#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

pub(crate) async fn sleep(millis: u64) {
    #[cfg(target_arch = "wasm32")]
    {
        gloo_timers::future::TimeoutFuture::new(millis as u32).await;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(feature = "tokio")]
        tokio::time::sleep(Duration::from_millis(millis)).await;

        #[cfg(not(feature = "tokio"))]
        {
            // Fallback if tokio is not enabled (e.g. mobile?), though usually it should be.
            // But for now, we just panic or warn, or try std sleep (blocking).
            // Since we added tokio to desktop, this should cover the crash case.
            tracing::warn!(
                "Async sleep called without tokio runtime on non-wasm target. Blocking thread."
            );
            std::thread::sleep(Duration::from_millis(millis));
        }
    }
}
