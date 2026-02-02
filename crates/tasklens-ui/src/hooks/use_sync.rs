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

pub fn use_sync_client(#[allow(unused_variables)] store: Signal<AppStore>) -> Signal<SyncStatus> {
    #[allow(unused_mut)]
    let mut status = use_signal(|| SyncStatus::Disconnected);

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
            // Wait for store to have a repo
            loop {
                let repo_opt = store.read().repo.clone();
                if let Some(repo) = repo_opt {
                    // Check for Sync URL
                    if let Ok(url) = LocalStorage::get::<String>(SYNC_SERVER_URL_KEY)
                        && !url.is_empty()
                    {
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
                    }
                    break;
                }
                gloo_timers::future::TimeoutFuture::new(100).await;
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            tracing::warn!("Sync client is not yet implemented for desktop targets.");
        }
    });

    status
}
