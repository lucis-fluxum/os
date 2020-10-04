use spinning_top::{Spinlock, SpinlockGuard};

/// A wrapper around `spinning_top::Spinlock` to permit trait implementations.
pub struct Mutex<T>(Spinlock<T>);

impl<T> Mutex<T> {
    pub const fn new(inner: T) -> Self {
        Self(Spinlock::new(inner))
    }

    pub fn lock(&self) -> SpinlockGuard<T> {
        self.0.lock()
    }
}
