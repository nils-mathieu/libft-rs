//! Virtual-to-physical memory mapping.

use core::ffi::{c_int, c_void};

use bitflags::bitflags;

use crate::{Errno, Fd, Result};

bitflags! {
    /// Protection flags for memory mapping.
    #[derive(Default, Debug, Clone, Copy)]
    pub struct Prot: c_int {
        /// Pages may be executed.
        const EXEC = libc::PROT_EXEC;
        /// Pages may be written.
        const WRITE = libc::PROT_WRITE;
        /// Pages may be read.
        const READ = libc::PROT_READ;
    }
}

/// Anonymously maps virtual pages to physical pages.
///
/// # Safety
///
/// Calling this function may violate memory safety if currently mapped pages are messed with.
#[doc(alias = "mmap")]
pub unsafe fn anonymous(close_to: *mut (), length: usize, prot: Prot) -> Result<*mut ()> {
    let ret = unsafe {
        libc::mmap(
            close_to as *mut c_void,
            length,
            prot.bits(),
            libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
            -1,
            0,
        )
    };

    if ret == libc::MAP_FAILED {
        Err(Errno::last())
    } else {
        Ok(ret as *mut ())
    }
}

/// Maps a file to memory.
///
/// # Safety
///
/// Calling this function may violate memory safety if currently mapped pages are messed with.
#[doc(alias = "mmap")]
pub unsafe fn file(
    close_to: *mut (),
    length: usize,
    prot: Prot,
    fd: Fd,
    offset: usize,
) -> Result<*mut ()> {
    let ret = unsafe {
        libc::mmap(
            close_to as *mut c_void,
            length,
            prot.bits(),
            libc::MAP_PRIVATE,
            fd.to_raw(),
            offset as libc::off_t,
        )
    };

    if ret == libc::MAP_FAILED {
        Err(Errno::last())
    } else {
        Ok(ret as *mut ())
    }
}

/// Unmaps a memory mapping.
///
/// # Safety
///
/// Calling this function may violate memory safety if currently mapped pages are messed with.
#[doc(alias = "munmap")]
pub unsafe fn unmap(addr: *mut (), length: usize) -> Result<()> {
    let ret = unsafe { libc::munmap(addr as *mut c_void, length) };

    if ret == 0 {
        Ok(())
    } else {
        Err(Errno::last())
    }
}
