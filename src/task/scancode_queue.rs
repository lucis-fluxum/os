use core::{
    pin::Pin,
    task::{Context, Poll},
};

use conquer_once::spin::Lazy;
use crossbeam_queue::{ArrayQueue, PopError};
use futures_util::{stream::Stream, task::AtomicWaker};
use log::warn;

// The queue must be accessible only through immutable borrows, so it is not stored in
// ScancodeQueue, which requires a mutable borrow in order to call next().
static SCANCODE_QUEUE: Lazy<ArrayQueue<u8>> = Lazy::new(|| ArrayQueue::new(100));
static WAKER: AtomicWaker = AtomicWaker::new();

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) {
    if Lazy::is_initialized(&SCANCODE_QUEUE) {
        SCANCODE_QUEUE
            .push(scancode)
            .unwrap_or_else(|_| warn!("Scancode queue is full; dropping keyboard input"));
        WAKER.wake();
    } else {
        warn!("Scancode queue is uninitialized!");
    }
}

pub(crate) struct ScancodeQueue;

impl Stream for ScancodeQueue {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<u8>> {
        if let Ok(scancode) = SCANCODE_QUEUE.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&context.waker());

        match SCANCODE_QUEUE.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(PopError) => Poll::Pending,
        }
    }
}
