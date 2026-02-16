#[cfg(not(target_arch = "wasm32"))]
use samod::runtime::LocalRuntimeHandle;
#[cfg(not(target_arch = "wasm32"))]
use std::future::Future;
#[cfg(not(target_arch = "wasm32"))]
use std::pin::Pin;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug)]
pub(crate) struct DioxusRuntime;

#[cfg(not(target_arch = "wasm32"))]
impl LocalRuntimeHandle for DioxusRuntime {
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + 'static>>) {
        dioxus::prelude::spawn(future);
    }
}
