use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::{Acquire, Relaxed, Release};

/// A mutex implementation that spins until it can acquire the lock.
pub struct SpinMutex {
    locked: AtomicBool,
}

unsafe impl super::RawMutex for SpinMutex {
    #[allow(clippy::declare_interior_mutable_const)]
    const UNLOCKED: Self = Self {
        locked: AtomicBool::new(false),
    };

    #[inline]
    fn try_lock(&self) -> bool {
        self.locked
            .compare_exchange(false, true, Acquire, Relaxed)
            .is_ok()
    }

    #[inline]
    fn lock(&self) {
        if !self.try_lock() {
            cold_lock(&self.locked)
        }
    }

    #[inline]
    unsafe fn unlock(&self) {
        self.locked.store(false, Release);
    }
}

/// Spins until the lock can be acquired.
fn cold_lock(locked: &AtomicBool) {
    while locked
        .compare_exchange_weak(false, true, Acquire, Relaxed)
        .is_err()
    {
        while locked.load(Relaxed) {
            core::hint::spin_loop();
        }
    }
}
