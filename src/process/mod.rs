//! Functions and types to work with processes.

mod pid;
mod signal;

pub use self::pid::*;
pub use self::signal::*;

use crate::CharStar;
use crate::{Errno, Result};

/// Aborts the current process.
#[inline]
pub fn abort() -> ! {
    unsafe { libc::abort() }
}

/// Pauses the current process until a signal is received.
#[inline]
pub fn pause() {
    unsafe { libc::pause() };
}

/// The result of a [`fork`] operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fork {
    /// The current process is the parent.
    ///
    /// The PID of the child process is associated with this variant.
    Parent(Pid),
    /// The current process is the child.
    Child,
}

/// Forks the current process, cloning all of its resources.
///
/// Depending on whether the function returns in the parent process, or in the
/// child process, the return value will be either `Fork::Parent(child_pid)` or
/// `Fork::Child`, respectively.
#[inline]
pub fn fork() -> Result<Fork> {
    match unsafe { libc::fork() } {
        -1 => Err(Errno::last()),
        0 => Ok(Fork::Child),
        pid => Ok(Fork::Parent(Pid::from_raw(pid))),
    }
}

/// Replace the current process with a new one.
///
/// # Arguments
///
/// * `path` - The path to the new process image.
///
/// * `args` - The arguments to pass to the new process.
///
/// * `envp` - The environment variables to pass to the new process.
///
/// # Safety
///
/// `args` must be a null-terminated array of null-terminated strings.
///
/// `envp` must be a null-terminated array of null-terminated strings.
///
/// # Returns
///
/// On success, this function does not return. On error, it returns the error code.
#[inline]
#[doc(alias = "execve")]
pub unsafe fn exec(
    path: &CharStar,
    args: *const *const CharStar,
    envp: *const *const CharStar,
) -> Result<!> {
    unsafe { libc::execve(path.as_ptr(), args.cast(), envp.cast()) };
    Err(Errno::last())
}
