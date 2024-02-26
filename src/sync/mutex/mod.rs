//! Mutual exclusion primitives.
//!
//! This module provides mutual exclusion primitives that can be used to protect data structures
//! from concurrent access.

use core::cell::UnsafeCell;
use core::ops::Deref;
use core::ops::DerefMut;

mod noblock;
mod pthread;
mod spin;

pub use self::noblock::*;
pub use self::pthread::*;
pub use self::spin::*;

/// Describes the core functionality of a mutex.
///
/// # Safety
///
/// The mutex must really provide mutual exclusion. At any given point, only one execution context
/// may hold the lock.
pub unsafe trait RawMutex {
    /// The initial state of the raw mutex.
    ///
    /// In this state, the mutex is not currently locked.
    const UNLOCKED: Self;

    /// Attempts to lock the mutex immediately, without blocking.
    ///
    /// # Returns
    ///
    /// This function returns `true` if the mutex has been acquired, or `false` if another
    /// execution context already holds the lock.
    fn try_lock(&self) -> bool;

    /// Locks the mutex.
    ///
    /// This function blocks the current thread until the mutex has been acquired.
    fn lock(&self);

    /// Unlocks the mutex.
    ///
    /// # Safety
    ///
    /// This function may only be called if the current execution context holds the lock.
    unsafe fn unlock(&self);
}

/// A mutual exclusion primitive that can be used to protect a `T` from concurrent access.
///
/// This type ensures that the data is only accessed by one thread at any given time.
pub struct Mutex<T: ?Sized, M: RawMutex = PthreadMutex> {
    mutex: M,
    data: UnsafeCell<T>,
}

impl<T: ?Sized, M: RawMutex> Mutex<T, M> {
    /// Creates a new [`Mutex<T, M>`] instance.
    #[inline]
    pub const fn new(value: T) -> Self
    where
        T: Sized,
    {
        Self {
            mutex: M::UNLOCKED,
            data: UnsafeCell::new(value),
        }
    }

    /// Locks the mutex and returns a guard that releases the lock when dropped.
    ///
    /// If the lock cannot be acquired immediately, this function will block the current thread
    /// until it can acquire the lock.
    #[inline]
    pub fn lock(&self) -> MutexGuard<'_, T, M> {
        self.mutex.lock();
        MutexGuard {
            mutex: &self.mutex,
            data: unsafe { &mut *self.data.get() },
        }
    }

    /// Attempts to lock the mutex immediately, without blocking.
    ///
    /// If the mutex cannot be acquired immediately, this function returns `None`.
    #[inline]
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T, M>> {
        if self.mutex.try_lock() {
            Some(MutexGuard {
                mutex: &self.mutex,
                data: unsafe { &mut *self.data.get() },
            })
        } else {
            None
        }
    }
}

unsafe impl<T: ?Sized + Send, M: RawMutex> Sync for Mutex<T, M> {}
unsafe impl<T: ?Sized + Send, M: RawMutex> Send for Mutex<T, M> {}

/// A guard that releases the lock of a [`Mutex`] when dropped.
pub struct MutexGuard<'a, T: ?Sized, M: RawMutex> {
    mutex: &'a M,
    data: &'a mut T,
}

impl<T: ?Sized, M: RawMutex> Deref for MutexGuard<'_, T, M> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.data
    }
}

impl<T: ?Sized, M: RawMutex> DerefMut for MutexGuard<'_, T, M> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<T: ?Sized, M: RawMutex> Drop for MutexGuard<'_, T, M> {
    #[inline]
    fn drop(&mut self) {
        unsafe { self.mutex.unlock() }
    }
}
