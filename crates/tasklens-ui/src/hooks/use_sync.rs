use dioxus::prelude::*;
use futures::{Sink, SinkExt, Stream, StreamExt};
use gloo_storage::{LocalStorage, Storage};
use std::pin::Pin;
use std::task::{Context, Poll};
use tasklens_store::store::AppStore;
pub use tasklens_store::sync::SyncStatus;

pub const SYNC_SERVER_URL_KEY: &str = "tasklens_sync_server_url";

pub fn use_sync_client(store: Signal<AppStore>) -> Signal<SyncStatus> {
    let mut status = use_signal(|| SyncStatus::Disconnected);

    // Wrapper to satisfy Send bound on WASM (safe because single-threaded browser)
    struct UnsafeSend<T>(T);
    unsafe impl<T> Send for UnsafeSend<T> {}
    unsafe impl<T> Sync for UnsafeSend<T> {}
    impl<T> Unpin for UnsafeSend<T> {}

    impl<T: Stream + Unpin> Stream for UnsafeSend<T> {
        type Item = T::Item;
        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Pin::new(&mut self.0).poll_next(cx)
        }
    }

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

    #[derive(Debug)]
    struct SyncError(String);
    impl std::fmt::Display for SyncError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::error::Error for SyncError {}

    use_future(move || async move {
        // Wait for store to have a repo
        loop {
            let repo_opt = store.read().repo.clone();
            if let Some(repo) = repo_opt {
                // Check for Sync URL
                if let Ok(url) = LocalStorage::get::<String>(SYNC_SERVER_URL_KEY)
                    && !url.is_empty()
                {
                    tracing::info!("Starting sync with {}", url);
                    status.set(SyncStatus::Connecting);

                    match gloo_net::websocket::futures::WebSocket::open(&url) {
                        Ok(ws) => {
                            let (write, read) = ws.split();

                            // Map gloo_net messages to Vec<u8> for repo.connect
                            let stream = read.filter_map(|res| async move {
                                match res {
                                    Ok(gloo_net::websocket::Message::Bytes(b)) => {
                                        tracing::info!("Sync RX: {} bytes", b.len());
                                        Some(Ok(b))
                                    }
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
                                    tracing::info!("Sync TX: {} bytes", b.len());
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
    });

    status
}
