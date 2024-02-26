//! Functions related to the `fcntl` syscall.

use bitflags::bitflags;
use core::ffi::c_int;

use crate::{Errno, Fd, Result};

bitflags! {
    /// Flags that can be passed to [`Fd::open`].
    ///
    /// Describes how the file should be opened.
    #[derive(Clone, Copy, Default, PartialEq, Eq)]
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

bitflags! {
    /// The mode of a file.
    #[derive(Clone, Copy, Default, PartialEq, Eq)]
    pub struct Mode: libc::mode_t {
        /// The file can be read by the owner.
        const OWNER_READ = libc::S_IRUSR;
        /// The file can be written by the owner.
        const OWNER_WRITE = libc::S_IWUSR;
        /// The file can be executed by the owner.
        const OWNER_EXECUTE = libc::S_IXUSR;
        /// The file can be read by the group.
        const GROUP_READ = libc::S_IRGRP;
        /// The file can be written by the group.
        const GROUP_WRITE = libc::S_IWGRP;
        /// The file can be executed by the group.
        const GROUP_EXECUTE = libc::S_IXGRP;
        /// The file can be read by others.
        const OTHER_READ = libc::S_IROTH;
        /// The file can be written by others.
        const OTHER_WRITE = libc::S_IWOTH;
        /// The file can be executed by others.
        const OTHER_EXECUTE = libc::S_IXOTH;
    }
}

impl Fd {
    /// Sets the [`OpenFlags`] for this file descriptor.
    #[inline]
    #[doc(alias = "fcntl")]
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
    #[doc(alias = "fcntl")]
    pub fn get_flags(self) -> Result<OpenFlags> {
        let ret = unsafe { libc::fcntl(self.0, libc::F_GETFL) };
        if ret < 0 {
            Err(Errno::last())
        } else {
            Ok(OpenFlags::from_bits_retain(ret))
        }
    }
}
