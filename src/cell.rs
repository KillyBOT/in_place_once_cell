use std::cell::{Cell, UnsafeCell};
use std::mem;

/// A cell that can only be mutated once.
pub struct InPlaceOnceCell<T> {
    is_initialized: Cell<bool>,
    inner: UnsafeCell<T>,
}

impl<T> InPlaceOnceCell<T> {
    /// Creates a new uninitialized cell.
    #[inline]
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self {
            is_initialized: Cell::new(false),
            inner: UnsafeCell::new(value),
        }
    }

    #[inline]
    #[must_use]
    fn is_initialized(&self) -> bool {
        self.is_initialized.get()
    }

    /// # Safety
    ///
    /// The cell must be initialized.
    #[inline]
    unsafe fn get_unchecked(&self) -> &T {
        debug_assert!(self.is_initialized());
        unsafe { &*self.inner.get() }
    }

    /// Gets the reference to the underlying value.
    ///
    /// Returns `None` if the cell is uninitialized.
    #[inline]
    pub fn get(&self) -> Option<&T> {
        if self.is_initialized() {
            // SAFETY: `self.is_initialized() == true`, so always safe
            Some(unsafe { self.get_unchecked() })
        } else {
            None
        }
    }

    /// Gets a mutable reference to the underlying value.
    ///
    /// Returns `None` if the cell is not initialized.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_initialized() {
            Some(self.inner.get_mut())
        } else {
            None
        }
    }

    ///
    #[inline]
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce(&mut T),
    {
        todo!()
    }

    #[inline]
    pub fn get_mut_or_init<F>(&mut self, f: F) -> &mut T
    where
        F: FnOnce(&mut T),
    {
        todo!()
    }

    /// Gets the contents of the cell, initializing it with `f(&mut T)` if the cell was
    /// uninitialized.
    ///
    /// Returns an error if the cell was uninitialized and `f()` returns an error.
    pub fn get_or_try_init<F, E>(&self, f: F) -> Result<&T, E>
    where
        F: FnOnce(&mut T) -> Result<(), E>,
    {
        todo!()
    }

    pub fn get_mut_or_try_init<F, E>(&mut self, f: F) -> Result<&mut T, E>
    where
        F: FnOnce(&mut T) -> Result<(), E>,
    {
        todo!()
    }

    // It's most likely that the value is already initialized.
    #[cold]
    fn try_init<F, E>(&self, f: F) -> Result<(), E>
    where
        F: FnOnce(&mut T) -> Result<(), E>,
    {
        // SAFETY: `try_init` is only called in `get_*_or_try_init`, meaning `self.inner` will
        // always be non-null
        let inner_mut_ref = unsafe { &mut *self.inner.get() };
        f(inner_mut_ref)
    }

    #[inline]
    pub fn into_inner(self) -> T {
        // TODO: Make this a `pub const fn`.
        self.inner.into_inner()
    }
}

// // `UnsafeCell` is not `Sync`, so neither is this. Technically this isn't needed.
// impl<T> !Sync for InPlaceOnceCell<T> {}
