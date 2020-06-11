use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Task<'f> {
    future: Pin<Box<dyn Future<Output = ()> + 'f>>,
}

impl<'f> Task<'f> {
    pub fn new(future: impl Future<Output = ()> + 'f) -> Self {
        Self {
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
