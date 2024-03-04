//! Functions to work with file descriptors.

use core::ffi::c_int;
use core::ops::Deref;

use crate::{Errno, Result};

mod fcntl;
mod io;
mod net;
mod poll;
mod select;

pub use self::fcntl::*;
pub use self::poll::*;
pub use self::select::*;

/// A file descriptor.
///
/// # Notes
///
/// This type makes no assumptions about the value or state of the file descriptor it represents
/// (no I/O safety). It is up to the user to ensure that the file descriptor is valid and that
/// it is used correctly.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Fd(c_int);

impl Fd {
    /// The file descriptor usually used to read data from the standard input stream.
    pub const STDIN: Self = Self(libc::STDIN_FILENO);
    /// The file descriptor usually used to write data to the standard output stream.
    pub const STDOUT: Self = Self(libc::STDOUT_FILENO);
    /// The file descriptor usually used to write data to the standard error stream.
    pub const STDERR: Self = Self(libc::STDERR_FILENO);

    /// Creates a new [`Fd`] instance from the provided raw file descriptor.
    #[inline(always)]
    pub const fn from_raw(fd: c_int) -> Self {
        Self(fd)
    }

    /// Returns the raw file descriptor number represented by this [`Fd`] instance.
    #[inline(always)]
    pub const fn to_raw(self) -> c_int {
        self.0
    }

    /// Closes the file descriptor.
    #[inline]
    pub fn close(self) -> Result<()> {
        let ret = unsafe { libc::close(self.0) };
        if ret == 0 {
            Ok(())
        } else {
            Err(Errno::last())
        }
    }

    /// Returns whether this file descriptor is a TTY.
    ///
    /// # Remarks
    ///
    /// This function returns `false` if the file descriptor is not a TTY, or if the file
    /// descriptor is not valid.
    #[inline]
    #[doc(alias = "isatty")]
    pub fn is_a_tty(self) -> bool {
        unsafe { libc::isatty(self.0) == 1 }
    }
}

/// A RAII wrapper around a file descriptor.
///
/// When a [`File`] is dropped, the underlying file descriptor is automatically closed.
pub struct File(Fd);

impl File {
    /// Creates a new [`File`] instance from the provided raw file descriptor.
    pub const fn from_raw(fd: c_int) -> Self {
        Self(Fd::from_raw(fd))
    }

    /// Creates a new [`File`] instance from the provided [`Fd`] instance.
    pub const fn from_fd(fd: Fd) -> Self {
        Self(fd)
    }

    /// Leaks this [`File`], returning the underlying [`Fd`] while making sure
    /// that the file descriptor is not closed when this [`File`] is dropped.
    pub fn leak(this: Self) -> Fd {
        let fd = this.0;
        core::mem::forget(this);
        fd
    }
}

impl Deref for File {
    type Target = Fd;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for File {
    fn drop(&mut self) {
        let _ = self.0.close();
    }
}
