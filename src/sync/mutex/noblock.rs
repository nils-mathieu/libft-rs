use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::{Acquire, Relaxed, Release};

/// A mutex implementation that crashes the program if the lock is already held.
pub struct NoBlockMutex {
    locked: AtomicBool,
}

unsafe impl super::RawMutex for NoBlockMutex {
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
        assert!(self.try_lock(), "mutex is already locked");
    }

    #[inline]
    unsafe fn unlock(&self) {
        self.locked.store(false, Release);
    }
}
