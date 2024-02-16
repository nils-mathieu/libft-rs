use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::*;

/// A cell that can be written to only once.
pub struct OnceCell<T> {
    /// Whether the cell is initialized or not.
    initialized: AtomicBool,
    /// The protected value.
    value: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Sync + Send> Sync for OnceCell<T> {}
unsafe impl<T: Send> Send for OnceCell<T> {}

impl<T> OnceCell<T> {
    /// Creates a new [`OnceCell`] without initializing it.
    pub const fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Creates a new [`OnceCell`] with the given value.
    pub const fn with_value(value: T) -> Self {
        Self {
            initialized: AtomicBool::new(true),
            value: UnsafeCell::new(MaybeUninit::new(value)),
        }
    }

    /// If the cell is currently initialized, returns a reference to the inner
    /// value. Otherwise, returns `None`.
    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if *self.initialized.get_mut() {
            Some(unsafe { self.value.get_mut().assume_init_mut() })
        } else {
            None
        }
    }

    /// Returns the inner value of the cell, or `None` if it has not been
    /// initialized yet.
    #[inline]
    pub fn into_inner(mut self) -> Option<T> {
        if *self.initialized.get_mut() {
            Some(unsafe { self.value.into_inner().assume_init() })
        } else {
            None
        }
    }

    /// Takes the value off the cell, leaving it uninitialized.
    #[inline]
    pub fn take(&mut self) -> Option<T> {
        core::mem::take(self).into_inner()
    }

    /// Returns whether the cell is initialized or not.
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Acquire)
    }

    /// Sets the value of the cell without checking whether it has been initialized
    /// or not previously.
    ///
    /// # Safety
    ///
    /// The cell must not be initialized.
    #[inline]
    pub unsafe fn set_unchecked(&self, value: T) {
        unsafe { (*self.value.get()).write(value) };
        self.initialized.store(true, Release);
    }

    /// Returns a reference to the inner value of the cell without checking
    /// whether it has been initialized or not previously.
    ///
    /// # Safety
    ///
    /// The cell must be initialized.
    #[inline]
    pub unsafe fn get_unchecked(&self) -> &T {
        unsafe { (*self.value.get()).assume_init_ref() }
    }

    /// Returns the inner value of the cell, or initializes it with the given
    /// function if it is not initialized yet.
    #[inline]
    pub fn get_or_init_with<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        unsafe {
            if !self.is_initialized() {
                self.set_unchecked(f());
            }
            self.get_unchecked()
        }
    }

    /// Attempts to read the inner value of the cell.
    ///
    /// If the cell has not been initialized yet, this method returns `None`.
    #[inline]
    pub fn get(&self) -> Option<&T> {
        if self.is_initialized() {
            Some(unsafe { self.get_unchecked() })
        } else {
            None
        }
    }

    /// Attempts to write the inner valu of the cell. If the cell has already
    /// been initialized, this method returns `Err(value)`, where `value` is the
    /// value that was not written.
    ///
    /// On success, the method returns a reference to the written value.
    #[inline]
    pub fn set(&self, value: T) -> Result<(), T> {
        if self.is_initialized() {
            Err(value)
        } else {
            unsafe { self.set_unchecked(value) };
            Ok(())
        }
    }
}

impl<T> Default for OnceCell<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
