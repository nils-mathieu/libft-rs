/// A mutex implementation based on `libpthread`.
#[doc(alias = "pthread_mutex_t")]
pub struct PthreadMutex(libc::pthread_mutex_t);

impl PthreadMutex {
    /// Returns a raw pointer to the inner `pthread_mutex_t`.
    #[inline]
    pub const fn as_ptr(&self) -> *const libc::pthread_mutex_t {
        &self.0
    }
}

unsafe impl super::RawMutex for PthreadMutex {
    const UNLOCKED: Self = PthreadMutex(libc::PTHREAD_MUTEX_INITIALIZER);

    #[inline]
    fn lock(&self) {
        let ret = unsafe { libc::pthread_mutex_lock(self.as_ptr().cast_mut()) };
        assert!(ret == 0, "pthread_mutex_lock failed");
    }

    #[inline]
    unsafe fn unlock(&self) {
        unsafe { libc::pthread_mutex_unlock(self.as_ptr().cast_mut()) };
    }

    #[inline]
    fn try_lock(&self) -> bool {
        unsafe { libc::pthread_mutex_trylock(self.as_ptr().cast_mut()) == 0 }
    }
}
