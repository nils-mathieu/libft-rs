//! Implements the [`libc::poll`] system call.

use core::ffi::{c_int, c_short};
use core::time::Duration;

use bitflags::bitflags;

use crate::{Errno, Fd, Result};

bitflags! {
    /// Some events that can be waited on by [`poll`].
    #[derive(Copy, Clone, Debug)]
    pub struct PollFlags: c_short {
        /// Data may be read without blocking.
        const IN = libc::POLLIN;
        /// Priority data may be read without blocking.
        const PRI = libc::POLLPRI;
        /// Data may be written without blocking.
        const OUT = libc::POLLOUT;
        /// An error has occurred.
        const ERR = libc::POLLERR;
        /// A hangup has occurred.
        const HUP = libc::POLLHUP;
        /// Invalid polling request.
        const NVAL = libc::POLLNVAL;
    }
}

/// Describes a file descriptor being waited on.
#[derive(Copy, Clone)]
#[repr(transparent)]
#[doc(alias = "pollfd")]
pub struct PollFd(libc::pollfd);

impl PollFd {
    /// Creates a new [`PollFd`] instance.
    pub const fn new(fd: Fd, events: PollFlags) -> Self {
        Self(libc::pollfd {
            fd: fd.to_raw(),
            events: events.bits(),
            revents: 0,
        })
    }

    /// Returns the file descriptor being waited on.
    #[inline]
    pub fn fd(&self) -> Fd {
        Fd::from_raw(self.0.fd)
    }

    /// Returns the events being waited on.
    #[inline]
    pub fn events(&self) -> PollFlags {
        PollFlags::from_bits_retain(self.0.events)
    }

    /// Returns the events being waited on.
    #[inline]
    pub fn events_mut(&mut self) -> &mut PollFlags {
        unsafe { &mut *(&mut self.0.events as *mut c_short as *mut PollFlags) }
    }

    /// Returns the events that occurred on the file descriptor.
    #[inline]
    pub fn revents(&self) -> PollFlags {
        PollFlags::from_bits_retain(self.0.revents)
    }

    /// Returns the events that occurred on the file descriptor.
    #[inline]
    pub fn revents_mut(&mut self) -> &mut PollFlags {
        unsafe { &mut *(&mut self.0.revents as *mut c_short as *mut PollFlags) }
    }

    /// Returns whether this [`PollFd`] instance is ready (it has a non-zero `revents`).
    pub fn ready(&self) -> bool {
        self.revents().is_empty()
    }
}

/// Waits for a collection of file descriptors to become ready.
///
/// # Arguments
///
/// - `fds` - The file descriptors being waited on. Exactly which events are being waited on
///   is specified by the `events` field of each [`PollFd`] instance. The `revents` field
///   describes which events occurred on the file descriptors when the function returns.
///
/// - `timeout` - The maximum amount of time to wait for an event to occur. If `None`, then
///   the function will block indefinitely.
pub fn poll(fds: &mut [PollFd], timeout: Option<Duration>) -> Result<usize> {
    let timeout: c_int = timeout
        .and_then(|t| t.as_millis().try_into().ok())
        .unwrap_or(-1);

    let ret = unsafe { libc::poll(fds.as_mut_ptr().cast(), fds.len() as _, timeout) };

    if ret < 0 {
        Err(Errno::last())
    } else {
        Ok(ret as _)
    }
}
