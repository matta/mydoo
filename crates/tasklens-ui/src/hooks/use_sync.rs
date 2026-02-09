use dioxus::prelude::*;
#[cfg(target_arch = "wasm32")]
use futures::{Sink, SinkExt, Stream, StreamExt};
#[cfg(target_arch = "wasm32")]
use gloo_storage::{LocalStorage, Storage};
#[cfg(target_arch = "wasm32")]
use std::pin::Pin;
#[cfg(target_arch = "wasm32")]
use std::task::{Context, Poll};
use tasklens_store::store::AppStore;
pub use tasklens_store::sync::SyncStatus;

#[cfg(target_arch = "wasm32")]
pub(crate) const SYNC_SERVER_URL_KEY: &str = "tasklens_sync_server_url";

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::Closure;

pub fn use_sync_client(#[allow(unused_variables)] store: Signal<AppStore>) -> Signal<SyncStatus> {
    #[allow(unused_mut)]
    let mut status = use_signal(|| SyncStatus::Disconnected);
    #[cfg(target_arch = "wasm32")]
    let mut retry_trigger = use_signal(|| 0u64);

    #[cfg(target_arch = "wasm32")]
    use_effect(move || {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                let closure = Closure::wrap(Box::new(move || {
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            if !document.hidden() && status.read().is_disconnected() {
                                tracing::info!("App became visible, triggering sync retry");
                                retry_trigger.with_mut(|v| *v += 1);
                            }
                        }
                    }
                }) as Box<dyn FnMut()>);

                let _ = document.add_event_listener_with_callback(
                    "visibilitychange",
                    closure.as_ref().unchecked_ref(),
                );
                closure.forget();
            }
        }
    });

    // Wrapper to satisfy Send bound on WASM (safe because single-threaded browser)
    #[cfg(target_arch = "wasm32")]
    struct UnsafeSend<T>(T);
    #[cfg(target_arch = "wasm32")]
    unsafe impl<T> Send for UnsafeSend<T> {}
    #[cfg(target_arch = "wasm32")]
    unsafe impl<T> Sync for UnsafeSend<T> {}
    #[cfg(target_arch = "wasm32")]
    impl<T> Unpin for UnsafeSend<T> {}

    #[cfg(target_arch = "wasm32")]
    impl<T: Stream + Unpin> Stream for UnsafeSend<T> {
        type Item = T::Item;
        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Pin::new(&mut self.0).poll_next(cx)
        }
    }

    #[cfg(target_arch = "wasm32")]
    impl<T: Sink<Item> + Unpin, Item> Sink<Item> for UnsafeSend<T> {
        type Error = T::Error;
        fn poll_ready(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut self.0).poll_ready(cx)
        }
        fn start_send(mut self: Pin<&mut Self>, item: Item) -> Result<(), Self::Error> {
            Pin::new(&mut self.0).start_send(item)
        }
        fn poll_flush(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut self.0).poll_flush(cx)
        }
        fn poll_close(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Pin::new(&mut self.0).poll_close(cx)
        }
    }

    #[cfg(target_arch = "wasm32")]
    #[derive(Debug)]
    struct SyncError(String);
    #[cfg(target_arch = "wasm32")]
    impl std::fmt::Display for SyncError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    #[cfg(target_arch = "wasm32")]
    impl std::error::Error for SyncError {}

    use_future(move || async move {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = retry_trigger(); // Reactivity
            let mut retry_delay = 1000;
            const MAX_DELAY: u32 = 30000;

            // Wait for store to have a repo
            let repo = loop {
                let repo_opt = store.read().repo.clone();
                if let Some(repo) = repo_opt {
                    break repo;
                }
                gloo_timers::future::TimeoutFuture::new(100).await;
            };

            // Reconnection Loop
            loop {
                // Check for Sync URL
                let url_res = LocalStorage::get::<String>(SYNC_SERVER_URL_KEY);
                let url = match url_res {
                    Ok(u) if !u.is_empty() => u,
                    _ => {
                        status.set(SyncStatus::Disconnected);
                        gloo_timers::future::TimeoutFuture::new(5000).await;
                        continue;
                    }
                };

                status.set(SyncStatus::Connecting);

                match gloo_net::websocket::futures::WebSocket::open(&url) {
                    Ok(ws) => {
                        let (write, read) = ws.split();

                        // Map gloo_net messages to Vec<u8> for repo.connect
                        let stream = read.filter_map(|res| async move {
                            match res {
                                Ok(gloo_net::websocket::Message::Bytes(b)) => Some(Ok(b)),
                                Ok(gloo_net::websocket::Message::Text(_)) => {
                                    tracing::warn!(
                                        "Received unexpected text message on sync websocket"
                                    );
                                    None
                                }
                                Err(e) => Some(Err(SyncError(e.to_string()))),
                            }
                        });

                        let sink = write
                            .with(|b: Vec<u8>| async move {
                                Ok::<_, gloo_net::websocket::WebSocketError>(
                                    gloo_net::websocket::Message::Bytes(b),
                                )
                            })
                            .sink_map_err(|e| SyncError(e.to_string()));

                        // Wrap in UnsafeSend to satisfy samod's Send bound
                        let stream = UnsafeSend(Box::pin(stream));
                        let sink = UnsafeSend(Box::pin(sink));

                        match repo.connect(stream, sink, samod::ConnDirection::Outgoing) {
                            Ok(conn) => {
                                status.set(SyncStatus::Connected);
                                retry_delay = 1000; // Reset delay on successful connection

                                // Wait for connection to finish (disconnect)
                                let reason = conn.finished().await;
                                tracing::warn!("Sync connection finished: {:?}", reason);
                                status.set(SyncStatus::Disconnected);
                            }
                            Err(e) => {
                                tracing::error!("Failed to connect: {:?}", e);
                                status.set(SyncStatus::Error(format!("{:?}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to open websocket: {:?}", e);
                        status.set(SyncStatus::Error(format!("{:?}", e)));
                    }
                }

                // Wait before retrying with exponential backoff
                tracing::info!("Sync: Reconnecting in {}ms", retry_delay);
                gloo_timers::future::TimeoutFuture::new(retry_delay).await;
                retry_delay = (retry_delay * 2).min(MAX_DELAY);
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            tracing::warn!("Sync client is not yet implemented for desktop targets.");
        }
    });

    status
}
