use samod::runtime::LocalRuntimeHandle;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Copy, Debug)]
pub struct DioxusRuntime;

impl LocalRuntimeHandle for DioxusRuntime {
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + 'static>>) {
        dioxus::prelude::spawn(future);
    }
}
