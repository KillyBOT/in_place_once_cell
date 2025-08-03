use std::cell::UnsafeCell;

pub struct InPlaceOnceCell<T> {
    inner: UnsafeCell<T>,
}
