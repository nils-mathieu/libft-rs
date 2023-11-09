//! Functions to work with file descriptors.

use core::ffi::c_int;

mod io;

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
    pub const fn from_raw(fd: c_int) -> Self {
        Self(fd)
    }

    /// Returns the raw file descriptor number represented by this [`Fd`] instance.
    pub const fn to_raw(self) -> c_int {
        self.0
    }
}
