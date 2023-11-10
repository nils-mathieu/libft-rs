//! Ways to use the [`libc::select`] system call.

use core::ffi::c_int;
use core::ops::Range;
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

    /// Creates an iterator over the file descriptors of this set.
    ///
    /// # Arguments
    ///
    /// - `max` - The file descriptor of the file descriptor with the highest numerical value.
    ///   This can also be used to selectively ignore the file descriptors with a higher number.
    pub fn iter(&self, max: Fd) -> FdSetIter {
        FdSetIter {
            set: self,
            range: 0..max.to_raw().wrapping_add(1),
        }
    }
}

/// An iterator over the file descriptors set in an [`FdSet`].
pub struct FdSetIter<'a> {
    set: &'a FdSet,
    range: Range<c_int>,
}

impl<'a> Iterator for FdSetIter<'a> {
    type Item = Fd;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let fd = Fd::from_raw(self.range.next()?);
            if self.set.contains(fd) {
                break Some(fd);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_min, max) = self.range.size_hint();
        (0, max)
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
    read: Option<&mut FdSet>,
    write: Option<&mut FdSet>,
    error: Option<&mut FdSet>,
    timeout: Option<Duration>,
) -> Result<usize> {
    let mut timeval;
    let timeout = match timeout {
        Some(dur) => {
            const MAX_TIMEVAL: Duration = Duration::new(i64::MAX as u64, 999_999_000);

            if dur > MAX_TIMEVAL {
                core::ptr::null_mut()
            } else {
                timeval = libc::timeval {
                    tv_sec: dur.as_secs() as i64,
                    tv_usec: dur.subsec_micros() as _,
                };
                &mut timeval
            }
        }
        None => core::ptr::null_mut(),
    };

    let readfds = read.map_or_else(core::ptr::null_mut, |set| &mut set.0);
    let writefds = write.map_or_else(core::ptr::null_mut, |set| &mut set.0);
    let exceptfds = error.map_or_else(core::ptr::null_mut, |set| &mut set.0);

    let ret = unsafe {
        libc::select(
            highest.to_raw().wrapping_add(1),
            readfds,
            writefds,
            exceptfds,
            timeout,
        )
    };

    if ret == -1 {
        Err(Errno::last())
    } else {
        Ok(ret as usize)
    }
}
