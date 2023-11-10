//! Ways to use the [`libc::select`] system call.

use core::time::Duration;

use crate::{Errno, Fd, Result};

/// A finite set of file descriptors.
pub struct FdSet(libc::fd_set);

impl FdSet {
    /// Creates a new, empty, [`FdSet`] instance.
    pub const fn new() -> Self {
        Self(unsafe { core::mem::zeroed() })
    }

    /// Returns whether a [`FdSet`] instance can hold the provided file descriptor.
    pub fn can_hold(fd: Fd) -> bool {
        fd.to_raw() < 1024
    }

    /// Inserts a file descriptor in this set.
    ///
    /// # Errors
    ///
    /// This function panics in debug builds when the provided file descriptor do not fit
    /// in the set.
    pub fn insert(&mut self, fd: Fd) {
        debug_assert!(Self::can_hold(fd));
        unsafe { libc::FD_SET(fd.to_raw(), &mut self.0) }
    }

    /// Removes a file descriptor from this set.
    ///
    /// # Panics
    ///
    /// This function panics in debug builds when the provided file descriptor do not fit
    /// in the set.
    pub fn remove(&mut self, fd: Fd) {
        debug_assert!(Self::can_hold(fd));
        unsafe { libc::FD_CLR(fd.to_raw(), &mut self.0) }
    }

    /// Returns whether `fd` is part of this set.
    ///
    /// # Panics
    ///
    /// This function panics in debug builds when the provided file descriptor do not fit
    /// in the set.
    pub fn contains(&self, fd: Fd) -> bool {
        debug_assert!(Self::can_hold(fd));
        unsafe { libc::FD_ISSET(fd.to_raw(), &self.0) }
    }

    /// Clears the set.
    pub fn clear(&mut self) {
        unsafe { core::ptr::write_bytes(&mut self.0, 0x00, 1) }
    }
}

/// Blocks the current thread until at least one of the file descriptors registered in the file
/// descriptor sets provided (`read`, `write` or `error`) becomes available.
///
/// If a `timeout` is provided, the function will additionally return with no available file
/// descriptor when the amount of time desired has elapsed.
///
/// # Arguments
///
/// - `highest` - The file descriptor with the highest numerical value that is part of one of the
///   sets. This can also be used to selectively remove higher file descriptors from the process.
///
/// - `read` - A set of file descriptors to wait for a read operation to become non-blocking.
///
/// - `write` - A set of file descriptors to wait for a write operation to become non-blocking.
///
/// - `error` - A set of file descriptors to wait for an error to occur.
///
/// - `timeout` - An optional timeout for the whole function.
///
/// # Returns
///
/// This function returns the number of file descriptors that have some work ready (reading,
/// writing, or an error). Note that if a file descriptor is present in multiple sets, it
/// will be counted multiple times.
///
/// On error, the error number is returned.
pub fn select(
    highest: Fd,
    read: &mut FdSet,
    write: &mut FdSet,
    error: &mut FdSet,
    timeout: Option<Duration>,
) -> Result<usize> {
    let mut timeval;

    let timeout = match timeout {
        Some(dur) => {
            const MAX_TIMEVAL: Duration = Duration::new(i64::MAX as u64, 999_999_000);

            if dur > MAX_TIMEVAL {
                timeval = libc::timeval {
                    tv_sec: i64::MAX,
                    tv_usec: 999_999,
                }
            } else {
                timeval = libc::timeval {
                    tv_sec: dur.as_secs() as i64,
                    tv_usec: dur.subsec_micros() as i64,
                };
            }

            &mut timeval
        }
        None => core::ptr::null_mut(),
    };

    let ret = unsafe {
        libc::select(
            highest.to_raw() + 1,
            &mut read.0,
            &mut write.0,
            &mut error.0,
            timeout,
        )
    };

    if ret == -1 {
        Err(Errno::last())
    } else {
        Ok(ret as usize)
    }
}
