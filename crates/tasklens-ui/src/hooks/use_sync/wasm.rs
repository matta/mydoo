use super::{INITIAL_RETRY_DELAY_MS, SYNC_SERVER_URL_KEY, SyncStatus, next_retry_delay_ms};
use dioxus::prelude::*;
use futures::{Sink, SinkExt, Stream, StreamExt};
use gloo_storage::{LocalStorage, Storage};
use std::pin::Pin;
use std::task::{Context, Poll};
use tasklens_store::store::AppStore;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

/// Build and run the browser sync client with reconnect behavior.
pub(super) fn use_sync_client_impl(store: Signal<AppStore>) -> Signal<SyncStatus> {
    let status = use_signal(|| SyncStatus::Disconnected);
    let retry_trigger = use_signal(|| 0u64);

    install_visibility_retry_effect(status, retry_trigger);
    spawn_sync_loop(store, status, retry_trigger);

    status
}

/// Trigger a reconnect attempt when the tab becomes visible again.
fn install_visibility_retry_effect(status: Signal<SyncStatus>, mut retry_trigger: Signal<u64>) {
    use_effect(move || {
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            let closure = Closure::wrap(Box::new(move || {
                if let Some(document) = web_sys::window().and_then(|window| window.document())
                    && !document.hidden()
                    && status.read().is_disconnected()
                {
                    tracing::info!("App became visible, triggering sync retry");
                    retry_trigger.with_mut(|counter| *counter += 1);
                }
            }) as Box<dyn FnMut()>);

            match document.add_event_listener_with_callback(
                "visibilitychange",
                closure.as_ref().unchecked_ref(),
            ) {
                Ok(_) => closure.forget(),
                Err(error) => {
                    tracing::warn!("Failed to add visibilitychange listener: {:?}", error)
                }
            }
        }
    });
}

/// Run the reconnect loop that keeps the sync channel alive while app is open.
fn spawn_sync_loop(
    store: Signal<AppStore>,
    mut status: Signal<SyncStatus>,
    retry_trigger: Signal<u64>,
) {
    use_future(move || async move {
        let _ = retry_trigger();
        let repo = wait_for_repo(store).await;
        let mut retry_delay_ms = INITIAL_RETRY_DELAY_MS;

        loop {
            let Some(url) = sync_server_url() else {
                status.set(SyncStatus::Disconnected);
                gloo_timers::future::TimeoutFuture::new(5_000).await;
                continue;
            };

            if connect_once(&repo, &url, status).await {
                retry_delay_ms = INITIAL_RETRY_DELAY_MS;
            }

            tracing::info!("Sync: reconnecting in {}ms", retry_delay_ms);
            gloo_timers::future::TimeoutFuture::new(retry_delay_ms).await;
            retry_delay_ms = next_retry_delay_ms(retry_delay_ms);
        }
    });
}

/// Poll until the app store has initialized a `samod::Repo`.
async fn wait_for_repo(store: Signal<AppStore>) -> samod::Repo {
    loop {
        if let Some(repo) = store.read().repo.clone() {
            return repo;
        }
        gloo_timers::future::TimeoutFuture::new(100).await;
    }
}

/// Read the persisted sync endpoint from browser local storage.
fn sync_server_url() -> Option<String> {
    match LocalStorage::get::<String>(SYNC_SERVER_URL_KEY) {
        Ok(url) if !url.is_empty() => Some(url),
        _ => None,
    }
}

/// Attempt one websocket connection and bridge it to repo sync.
async fn connect_once(repo: &samod::Repo, url: &str, mut status: Signal<SyncStatus>) -> bool {
    status.set(SyncStatus::Connecting);

    let websocket = match gloo_net::websocket::futures::WebSocket::open(url) {
        Ok(websocket) => websocket,
        Err(error) => {
            tracing::error!("Failed to open websocket: {:?}", error);
            status.set(SyncStatus::Error(format!("{:?}", error)));
            return false;
        }
    };

    let (write, read) = websocket.split();

    // Convert gloo websocket events to the byte stream expected by `samod::Repo`.
    let stream = read.filter_map(|message| async move {
        match message {
            Ok(gloo_net::websocket::Message::Bytes(bytes)) => Some(Ok(bytes)),
            Ok(gloo_net::websocket::Message::Text(_)) => {
                tracing::warn!("Received unexpected text message on sync websocket");
                None
            }
            Err(error) => Some(Err(SyncError(error.to_string()))),
        }
    });

    let sink = write
        .with(|bytes: Vec<u8>| async move {
            Ok::<_, gloo_net::websocket::WebSocketError>(gloo_net::websocket::Message::Bytes(bytes))
        })
        .sink_map_err(|error| SyncError(error.to_string()));

    let stream = UnsafeSend(Box::pin(stream));
    let sink = UnsafeSend(Box::pin(sink));

    match repo.connect(stream, sink, samod::ConnDirection::Outgoing) {
        Ok(connection) => {
            status.set(SyncStatus::Connected);
            let reason = connection.finished().await;
            tracing::warn!("Sync connection finished: {:?}", reason);
            status.set(SyncStatus::Disconnected);
            true
        }
        Err(error) => {
            tracing::error!("Failed to connect: {:?}", error);
            status.set(SyncStatus::Error(format!("{:?}", error)));
            false
        }
    }
}

/// Wrapper for browser-only stream/sink types that are single-threaded by design.
struct UnsafeSend<T>(T);

// SAFETY: `wasm32-unknown-unknown` runs in a single-threaded event loop for this app.
unsafe impl<T> Send for UnsafeSend<T> {}
// SAFETY: same rationale as `Send`; no cross-thread sharing occurs in this runtime.
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

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Item) -> Result<(), Self::Error> {
        Pin::new(&mut self.0).start_send(item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_close(cx)
    }
}

#[derive(Debug)]
struct SyncError(String);

impl std::fmt::Display for SyncError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl std::error::Error for SyncError {}
