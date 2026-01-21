use samod::runtime::LocalRuntimeHandle;
use std::future::Future;
use std::pin::Pin;

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug)]
pub struct WasmRuntime;

#[cfg(target_arch = "wasm32")]
impl LocalRuntimeHandle for WasmRuntime {
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + 'static>>) {
        wasm_bindgen_futures::spawn_local(future);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug)]
pub struct WasmRuntime;

#[cfg(not(target_arch = "wasm32"))]
impl LocalRuntimeHandle for WasmRuntime {
    fn spawn(&self, _future: Pin<Box<dyn Future<Output = ()> + 'static>>) {
        // No-op or panic? Let's go with no-op for compilation
    }
}
