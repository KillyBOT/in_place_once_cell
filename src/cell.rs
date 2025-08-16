use std::cell::{Cell, UnsafeCell};
use std::fmt;

// TODO: Add more documentation

// TODO: Once `#![feature(never_type)]` is stabilized, remove this
enum Never {}

/// A cell that can only be mutated once.
pub struct InPlaceOnceCell<T> {
    value: UnsafeCell<T>,
    is_mutated: Cell<bool>,
}

impl<T> InPlaceOnceCell<T> {
    /// Creates a new cell that has not been mutated.
    #[inline]
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            is_mutated: Cell::new(false),
        }
    }

    #[inline]
    #[must_use]
    fn is_mutated(&self) -> bool {
        self.is_mutated.get()
    }

    /// # Safety
    ///
    /// The cell must be mutated.
    #[inline]
    unsafe fn get_unchecked(&self) -> &T {
        debug_assert!(self.is_mutated());
        unsafe { &*self.value.get() }
    }

    /// # Safety
    ///
    /// The cell must be mutated.
    #[inline]
    unsafe fn get_mut_unchecked(&mut self) -> &mut T {
        debug_assert!(self.is_mutated());
        self.value.get_mut()
    }

    /// Gets the reference to the underlying value.
    ///
    /// Returns `None` if the cell is not mutated.
    #[inline]
    pub fn get(&self) -> Option<&T> {
        if self.is_mutated() {
            // SAFETY: `self.is_initialized() == true`, so always safe.
            Some(unsafe { self.get_unchecked() })
        } else {
            None
        }
    }

    /// Gets a mutable reference to the underlying value.
    ///
    /// Returns `None` if the cell is not mutated.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_mutated() {
            Some(self.value.get_mut())
        } else {
            None
        }
    }

    /// Gets the contents of the cell, mutating it with `f()` if the cell was never mutated.
    ///
    /// # Panics
    ///
    /// If `f()` panics, the panic is propagated to the caller, and the cell is never fully mutated.
    ///
    /// `f()` is not guaranteed to be reentrant, so calling the function after a panic also may
    /// result in a panic.
    #[inline]
    pub fn get_or_mutate<F>(&self, f: F) -> &T
    where
        F: FnOnce(&mut T),
    {
        match self.get_or_try_mutate(|val: &mut T| {
            f(val);
            Ok::<(), Never>(())
        }) {
            Ok(..) => {}
        }

        // SAFETY: the previous code is guaranteed to mutate the cell
        unsafe { self.get_unchecked() }
    }

    #[inline]
    pub fn get_mut_or_mutate<F>(&mut self, f: F) -> &mut T
    where
        F: FnOnce(&mut T),
    {
        match self.get_or_try_mutate(|val: &mut T| {
            f(val);
            Ok::<(), Never>(())
        }) {
            Ok(..) => {}
        }

        // SAFETY: the previous code is guaranteed to mutate the cell
        unsafe { self.get_mut_unchecked() }
    }

    /// Gets the contents of the cell, mutating it with `f(&mut T)` if the cell was never mutated.
    ///
    /// Returns an error if the cell was uninitialized and `f()` returns an error.
    pub fn get_or_try_mutate<F, E>(&self, f: F) -> Result<&T, E>
    where
        F: FnOnce(&mut T) -> Result<(), E>,
    {
        if let Some(val) = self.get() {
            return Ok(val);
        }

        self.try_mutate(f)?;
        // SAFETY: `try_mutate`, if it does not fail, is guaranteed to make the cell mutated.
        Ok(unsafe { self.get_unchecked() })
    }

    /// Gets the mutable contents of the cell, mutating it with `f(&mut T)` if the cell was never
    /// mutated.
    ///
    /// Returns an error if the cell was uninitialized and `f()` returns an error.
    pub fn get_mut_or_try_mutate<F, E>(&mut self, f: F) -> Result<&mut T, E>
    where
        F: FnOnce(&mut T) -> Result<(), E>,
    {
        // Stupid but we can't borrow `self`.
        if self.is_mutated() {
            // SAFETY: `is_mutated` ensures that `get_mut_unchecked` is safe.
            return Ok(unsafe { self.get_mut_unchecked() });
        }

        self.try_mutate(f)?;
        // SAFETY: `try_mutate`, if it does not fail, is guaranteed to make the cell mutated.
        Ok(unsafe { self.get_mut_unchecked() })
    }

    // It's most likely that the value is already initialized.
    #[cold]
    fn try_mutate<F, E>(&self, f: F) -> Result<(), E>
    where
        F: FnOnce(&mut T) -> Result<(), E>,
    {
        // SAFETY: `try_init` is only called in `get_*_or_try_mutate`, meaning `self.inner` will
        // always contain a valid non-null value that has not yet been mutated.
        let inner_mut_ref = unsafe { &mut *self.value.get() };
        f(inner_mut_ref)?;

        self.is_mutated.set(true);

        Ok(())
    }

    /// Consumes the cell, returning the wrapped value. Note that this occurs even when the cell
    /// was never mutated.
    #[inline]
    pub fn into_inner(self) -> T {
        // TODO: Make this a `pub const fn`.
        self.value.into_inner()
    }
}

impl<T: Default> Default for InPlaceOnceCell<T> {
    #[inline]
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: fmt::Debug> fmt::Debug for InPlaceOnceCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_tuple("InPlaceOnceCell");
        match self.get() {
            Some(v) => d.field(v),
            None => d.field(&format_args!("<untouched>")),
        };
        d.finish()
    }
}

impl<T: PartialEq> PartialEq for InPlaceOnceCell<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: Eq> Eq for InPlaceOnceCell<T> {}

impl<T> From<T> for InPlaceOnceCell<T> {
    /// Creates a new `InPlaceOnceCell<T>` containing `value`. This new cell is not yet mutated.
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
