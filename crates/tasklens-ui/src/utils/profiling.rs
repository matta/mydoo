//! Profiling utilities for UI debugging

use std::sync::atomic::{AtomicUsize, Ordering};

#[allow(dead_code)]
static RECONCILE_DEPTH: AtomicUsize = AtomicUsize::new(0);

#[allow(dead_code)]
pub struct ProfilingGuard {
    pub depth: usize,
    pub ind: String,
    pub start_ms: f64,
}

impl Drop for ProfilingGuard {
    fn drop(&mut self) {
        RECONCILE_DEPTH.fetch_sub(1, Ordering::SeqCst);
    }
}

#[allow(dead_code)]
pub fn depth_enter() -> ProfilingGuard {
    let depth = RECONCILE_DEPTH.fetch_add(1, Ordering::SeqCst);
    ProfilingGuard {
        depth,
        ind: "  ".repeat(depth),
        start_ms: now_ms(),
    }
}

#[allow(dead_code)]
pub fn now_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0.0
    }
}
