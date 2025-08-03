use std::cell::UnsafeCell;
use std::sync::Once;

pub struct InPlaceOnceLock<T> {
    once: Once,
    inner: UnsafeCell<T>,
}

impl<T> InPlaceOnceLock<T> {
    #[inline]
    #[must_use]
    pub const fn new(value: T) -> InPlaceOnceLock<T> {
        InPlaceOnceLock {
            once: Once::new(),
            inner: UnsafeCell::new(value),
        }
    }
}
