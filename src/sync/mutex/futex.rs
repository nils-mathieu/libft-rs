use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering::{self, Acquire, Relaxed, Release};

use super::Lock;

use State::{Contended, Locked, Unlocked};

/// The state of the [`FutexLock`].
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum State {
    /// The lock is unlocked.
    Unlocked,
    /// The lock held by some context, but they are alone and nobody is waiting
    /// for it to be unlocked.
    Locked,
    /// The lock is held by some context, and at least one thread is currently
    /// waiting for it to be unlocked.
    Contended,
}

/// An atomic version of [`State`].
struct AtomicState(AtomicU8);

impl AtomicState {
    /// Creates a new [`AtomicState`] instance.
    pub const fn new(state: State) -> Self {
        Self(AtomicU8::new(state as u8))
    }

    /// See [`AtomicU8::load`].
    pub fn load(&self, ord: Ordering) -> State {
        unsafe { core::mem::transmute(self.0.load(ord)) }
    }

    /// See [`AtomicU8::swap`].
    pub fn swap(&self, new: State, ord: Ordering) -> State {
        unsafe { core::mem::transmute(self.0.swap(new as _, ord)) }
    }

    /// See [`AtomicU8::compare_exchange_weak`].
    pub fn compare_exchange_weak(
        &self,
        current: State,
        new: State,
        success: Ordering,
        failure: Ordering,
    ) -> Result<State, State> {
        unsafe {
            core::mem::transmute(self.0.compare_exchange_weak(
                current as u8,
                new as u8,
                success,
                failure,
            ))
        }
    }
}

/// A [`Lock`] implementation that uses the [`libc::futex`] system call to block.
pub struct FutexLock(AtomicState);

unsafe impl Lock for FutexLock {
    #[allow(clippy::declare_interior_mutable_const)]
    const UNLOCKED: Self = Self(AtomicState::new(Unlocked));

    type GuardMarker = ();

    #[inline]
    fn is_locked(&self) -> bool {
        self.0.load(Relaxed) != Unlocked
    }

    #[inline]
    fn lock(&self) {}

    #[inline]
    fn try_lock(&self) -> bool {
        let mut state = self.0.load(Relaxed);

        loop {
            if state != Unlocked {
                break false;
            }

            match self
                .0
                .compare_exchange_weak(Unlocked, Locked, Acquire, Relaxed)
            {
                Ok(_) => break true,
                Err(new_state) => state = new_state,
            }
        }
    }

    #[inline]
    unsafe fn unlock(&self) {
        if self.0.swap(Unlocked, Release) == Contended {
            // There was some threads waiting for this lock.
        }
    }
}

impl FutexLock {
    /// Blocks the current thread until the lock is acquired.
    ///
    /// This function uses the futex API to block and only wakes up when an external call is
    /// made.
    #[cold]
    fn lock_contended(&self) {
        todo!();
    }
}
