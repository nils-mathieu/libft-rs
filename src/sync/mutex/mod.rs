//! A mutual exclusion primitive useful for protecting shared data in a multi-threaded context.

use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

mod pthread;
pub use self::pthread::*;

mod noblock;
pub use self::noblock::*;

#[cfg(target_os = "linux")]
mod futex;
#[cfg(target_os = "linux")]
pub use self::futex::*;

/// A locking mechanism that a [`Mutex<T>`] may use to protect its inner value.
///
/// # Safety
///
/// The lock must actually be exclusive! When it is acquired using [`lock`] or
/// [`try_lock`], no one must be able to acquire it until it is released using [`unlock`].
///
/// [`lock`]: Lock::lock
/// [`try_lock`]: Lock::try_lock
/// [`unlock`]: Lock::unlock
pub unsafe trait Lock {
    /// An instance of the [`Lock`] that is unlocked.
    const UNLOCKED: Self;

    /// A marker type included in the [`Guard`] that indicates whether the
    /// [`Guard`] is safe to send between threads.
    ///
    /// Typically, this will be either `()` if the guard can be [`Send`], or
    /// something like `*mut ()` if it cannot.
    type GuardMarker;

    /// Attempts to lock the [`Lock`] without blocking.
    ///
    /// If the lock is not immidiately available, `false` is returned and the
    /// lock is *not* aquired.
    ///
    /// Otherwise, `true` is returned and the lock is aquired.
    fn try_lock(&self) -> bool;

    /// Locks the current thread until the lock is aquired.
    ///
    /// Once this function returns, the lock is guaranteed to be aquired.
    fn lock(&self);

    /// Returns whether the lock is currently locked or not.
    ///
    /// Note that this function cannot be used to actually acquire the lock.
    fn is_locked(&self) -> bool;

    /// Unlock the mutex.
    ///
    /// # Safety
    ///
    /// The mutex must be currently locked by the current context (i.e. the call
    /// must be paired with a successful call to [`lock`] or [`try_lock`]).
    ///
    /// [`lock`]: Lock::lock
    /// [`try_lock`]: Lock::try_lock
    unsafe fn unlock(&self);
}

#[cfg(target_os = "macos")]
pub type DefaultLock = PthreadLock;
#[cfg(target_os = "linux")]
pub type DefaultLock = FutexLock;

/// Allows access to a shared value only once at a time.
pub struct Mutex<T: ?Sized, L = DefaultLock> {
    /// The lock used to protect the inner value.
    lock: L,
    /// The inner value.
    value: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send, L: Send> Send for Mutex<T, L> {}
unsafe impl<T: ?Sized + Send, L: Sync> Sync for Mutex<T, L> {}

impl<T: ?Sized, L: Lock> Mutex<T, L> {
    /// Creates a new [`Mutex`] instance, initialized with the provided value.
    pub const fn new(value: T) -> Self
    where
        T: Sized,
    {
        Self {
            lock: L::UNLOCKED,
            value: UnsafeCell::new(value),
        }
    }

    /// Attempts to lock the mutex without blocking.
    ///
    /// If the value is not immediately available, `None` is returned.
    pub fn try_lock(&self) -> Option<Guard<T, L>> {
        if self.lock.try_lock() {
            Some(unsafe { Guard::new_unchecked(self) })
        } else {
            None
        }
    }

    /// Blocks the current thread until the mutex is locked.
    pub fn lock(&self) -> Guard<T, L> {
        self.lock.lock();
        unsafe { Guard::new_unchecked(self) }
    }

    /// Returns whether the mutex is currently locked or not.
    pub fn is_locked(&self) -> bool {
        self.lock.is_locked()
    }
}

/// A proof that a [`Mutex<T>`] is currently locked by the current context.
#[clippy::has_significant_drop]
pub struct Guard<'a, T: ?Sized, L: Lock> {
    mutex: &'a Mutex<T, L>,
    _marker: PhantomData<(&'a mut T, L::GuardMarker)>,
}

impl<'a, T: ?Sized, L: Lock> Guard<'a, T, L> {
    /// Creates a new [`Guard`].
    ///
    /// # Safety
    ///
    /// The mutex must be currently locked by the current context.
    unsafe fn new_unchecked(mutex: &'a Mutex<T, L>) -> Self {
        Self {
            mutex,
            _marker: PhantomData,
        }
    }
}

impl<'a, T: ?Sized, L: Lock> Deref for Guard<'a, T, L> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<'a, T: ?Sized, L: Lock> DerefMut for Guard<'a, T, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.value.get() }
    }
}

impl<'a, T: ?Sized, L: Lock> Drop for Guard<'a, T, L> {
    fn drop(&mut self) {
        // SAFETY:
        //  The guard ensures that the mutex is locked by the current context.
        unsafe { self.mutex.lock.unlock() }
    }
}
