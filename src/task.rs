use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

use futures_util::StreamExt;
use pc_keyboard::DecodedKey;

use crate::{keyboard, print};

mod basic_executor;
mod executor;
pub(crate) mod scancode_queue;

pub use basic_executor::BasicExecutor;
pub use executor::Executor;
use scancode_queue::ScancodeQueue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Task<'f> {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()> + 'f>>,
}

impl<'f> Task<'f> {
    pub fn new(future: impl Future<Output = ()> + 'f) -> Self {
        Self {
            id: TaskId::new(),
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
