use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

mod basic_executor;

pub use basic_executor::BasicExecutor;

pub struct Task<'t> {
    future: Pin<Box<dyn Future<Output = ()> + 't>>,
}

impl<'t> Task<'t> {
    pub fn new(future: impl Future<Output = ()> + 't) -> Self {
        Self {
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
