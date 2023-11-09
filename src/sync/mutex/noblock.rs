use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::{Acquire, Relaxed, Release};

use super::Lock;

const LOCKED: bool = false;
const UNLOCKED: bool = true;

/// A [`Lock`] implementation that never blocks.
///
/// If a lock cannot be acquired, the lock panics instead.
pub struct NoBlockLock(AtomicBool);

unsafe impl Lock for NoBlockLock {
    #[allow(clippy::declare_interior_mutable_const)]
    const UNLOCKED: Self = Self(AtomicBool::new(UNLOCKED));

    type GuardMarker = ();

    fn is_locked(&self) -> bool {
        self.0.load(Relaxed) == LOCKED
    }

    fn lock(&self) {
        if !self.try_lock() {
            panic!("lock already taken");
        }
    }

    fn try_lock(&self) -> bool {
        self.0
            .compare_exchange_weak(UNLOCKED, LOCKED, Acquire, Relaxed)
            .is_ok()
    }

    unsafe fn unlock(&self) {
        self.0.store(UNLOCKED, Release);
    }
}
