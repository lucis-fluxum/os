use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::StreamExt;
use pc_keyboard::DecodedKey;

use crate::{keyboard, print};

mod basic_executor;
pub(crate) mod scancode_queue;

pub use basic_executor::BasicExecutor;
use scancode_queue::ScancodeQueue;

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

pub async fn print_keypresses() {
    let mut scancode_queue = ScancodeQueue;
    while let Some(scancode) = scancode_queue.next().await {
        if let Ok(Some(key)) = keyboard::decode_key(scancode) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }
}
