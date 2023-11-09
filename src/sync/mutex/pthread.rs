use super::Lock;

/// A raw mutex based on `pthread`'s.
pub struct PthreadLock(libc::pthread_mutex_t);

unsafe impl Send for PthreadLock {}
unsafe impl Sync for PthreadLock {}

unsafe impl Lock for PthreadLock {
    const UNLOCKED: Self = Self(libc::PTHREAD_MUTEX_INITIALIZER);

    // I don't actually know whether pthread allows its mutexes to be
    // used on multiple threads if the before-after ordering can be
    // guaranteed.
    type GuardMarker = *mut ();

    fn try_lock(&self) -> bool {
        unsafe { libc::pthread_mutex_trylock(&self.0 as *const _ as *mut _) == 0 }
    }

    fn lock(&self) {
        let ret = unsafe { libc::pthread_mutex_lock(&self.0 as *const _ as *mut _) };
        debug_assert_eq!(ret, 0);
    }

    fn is_locked(&self) -> bool {
        if self.try_lock() {
            unsafe { self.unlock() };
            true
        } else {
            false
        }
    }

    unsafe fn unlock(&self) {
        let ret = unsafe { libc::pthread_mutex_unlock(&self.0 as *const _ as *mut _) };
        debug_assert_eq!(ret, 0);
    }
}

impl Drop for PthreadLock {
    fn drop(&mut self) {
        let ret = unsafe { libc::pthread_mutex_destroy(&mut self.0 as *mut _ as *mut _) };
        debug_assert_eq!(ret, 0);
    }
}
