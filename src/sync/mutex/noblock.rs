use core::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use core::sync::atomic::{AtomicBool, AtomicPtr};

use super::Lock;

const LOCKED: bool = false;
const UNLOCKED: bool = true;

/// A [`Lock`] implementation that never blocks.
///
/// If a lock cannot be acquired, the lock panics instead.
pub struct NoBlockLock {
    /// The location of the last lock operation.
    #[cfg(debug_assertions)]
    location: AtomicPtr<core::panic::Location<'static>>,
    /// The current state of the lock.
    state: AtomicBool,
}

unsafe impl Lock for NoBlockLock {
    #[allow(clippy::declare_interior_mutable_const)]
    const UNLOCKED: Self = Self {
        #[cfg(debug_assertions)]
        location: AtomicPtr::new(core::ptr::null_mut()),
        state: AtomicBool::new(UNLOCKED),
    };

    type GuardMarker = ();

    #[inline]
    fn is_locked(&self) -> bool {
        self.state.load(Relaxed) == LOCKED
    }

    #[track_caller]
    #[inline]
    fn lock(&self) {
        if !self.try_lock() {
            #[cfg(debug_assertions)]
            panic!("lock already taken at {:?}", unsafe {
                *self.location.load(Relaxed)
            });

            #[cfg(not(debug_assertions))]
            panic!("lock already taken");
        }
    }

    #[inline]
    #[allow(clippy::let_and_return)]
    fn try_lock(&self) -> bool {
        let ret = self
            .state
            .compare_exchange_weak(UNLOCKED, LOCKED, Acquire, Relaxed)
            .is_ok();

        #[cfg(debug_assertions)]
        if ret {
            self.location.store(
                core::panic::Location::caller() as *const _ as *mut _,
                Relaxed,
            );
        }

        ret
    }

    #[inline]
    unsafe fn unlock(&self) {
        #[cfg(debug_assertions)]
        self.location.store(core::ptr::null_mut(), Relaxed);
        self.state.store(UNLOCKED, Release);
    }
}
