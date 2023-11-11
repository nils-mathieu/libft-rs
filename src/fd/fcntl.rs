//! Functions related to the `fcntl` syscall.

use bitflags::bitflags;
use core::ffi::c_int;

use crate::{Errno, Fd, Result};

bitflags! {
    /// Flags that can be passed to [`Fd::open`].
    ///
    /// Describes how the file should be opened.
    pub struct OpenFlags: c_int {
        /// The file should be opened for reading only.
        const READ_ONLY = libc::O_RDONLY;
        /// The file should be opened for writing only.
        const WRITE_ONLY = libc::O_WRONLY;
        /// The file should be opened for reading and writing.
        const READ_WRITE = libc::O_RDWR;
        /// The file should be created if it doesn't exist.
        const CREATE = libc::O_CREAT;
        /// The file should be truncated to zero size if it already exists.
        const TRUNCATE = libc::O_TRUNC;
        /// The file should be appended to if it already exists.
        const APPEND = libc::O_APPEND;
        /// The file should be opened in non-blocking mode.
        const NON_BLOCKING = libc::O_NONBLOCK;
    }
}

impl Fd {
    /// Sets the [`OpenFlags`] for this file descriptor.
    #[inline]
    pub fn set_flags(self, flags: OpenFlags) -> Result<()> {
        let ret = unsafe { libc::fcntl(self.0, libc::F_SETFL, flags.bits()) };
        if ret == 0 {
            Ok(())
        } else {
            Err(Errno::last())
        }
    }

    /// Gets the [`OpenFlags`] for this file descriptor.
    #[inline]
    pub fn get_flags(self) -> Result<OpenFlags> {
        let ret = unsafe { libc::fcntl(self.0, libc::F_GETFL) };
        if ret < 0 {
            Err(Errno::last())
        } else {
            Ok(OpenFlags::from_bits_retain(ret))
        }
    }
}
