use core::ffi::c_int;

use bitflags::bitflags;

use crate::{Result, Signal};

bitflags! {
    /// The options that can be passed to [`wait`] and friends.
    ///
    /// [`wait`]: Pid::wait
    pub struct WaitOptions: c_int {
        /// Prevents the function from blocking if no child process has
        /// exited.
        const NOHANG = libc::WNOHANG;
        /// Causes the function to return if a child process has
        /// stopped.
        const UNTRACED = libc::WUNTRACED;
        /// Causes the function to return if a child process has been
        /// contined.
        const CONTINUED = libc::WCONTINUED;
    }
}

/// Represents the exit status of a process.
#[derive(Debug, Clone, Copy)]
pub enum ExitStatus {
    /// The process has exited normally.
    ///
    /// The associated value is the exit code of the process.
    Exited(u8),
    /// The process was terminated by a signal.
    Signaled {
        /// The signal that produced the termination.
        signal: Signal,
        /// Whether the core of the process was dumped.
        core_dumped: bool,
    },
    /// Whether the process was stopped by delivery of a signal.
    ///
    /// This is only possible when the `UNTRACED` option is used.
    Stopped(Signal),
    /// Whether the process was continued by delivery of a signal.
    Continued,
}

impl ExitStatus {
    /// Creates a new [`ExitStatus`] by parsing the provided raw status
    /// code.
    pub const fn from_raw(status: c_int) -> Self {
        if libc::WIFEXITED(status) {
            Self::Exited(libc::WEXITSTATUS(status) as u8)
        } else if libc::WIFSIGNALED(status) {
            Self::Signaled {
                signal: Signal::from_raw(libc::WTERMSIG(status)),
                core_dumped: libc::WCOREDUMP(status),
            }
        } else if libc::WIFSTOPPED(status) {
            Self::Stopped(Signal::from_raw(libc::WSTOPSIG(status)))
        } else {
            Self::Continued
        }
    }
}

/// Represents a process identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pid(pub libc::pid_t);

impl Pid {
    /// Creates a new [`Pid`] instance from the provided raw value.
    #[inline]
    pub const fn from_raw(pid: libc::pid_t) -> Self {
        Self(pid)
    }

    /// Returns the raw value of the process identifier.
    #[inline]
    pub const fn as_raw(&self) -> libc::pid_t {
        self.0
    }

    /// Blocks the current thread until *any* of the child processes of
    /// the current process have changed state.
    #[inline]
    #[doc(alias = "waitpid")]
    pub fn wait_any(opts: WaitOptions) -> Result<(Pid, ExitStatus)> {
        let mut status = 0;
        let ret = unsafe { libc::waitpid(-1, &mut status, opts.bits()) };

        if ret == -1 {
            Err(crate::Errno::last())
        } else {
            Ok((Pid::from_raw(ret), ExitStatus::from_raw(status)))
        }
    }

    /// Waits until this process changes state.
    ///
    /// # Remarks
    ///
    /// This function assumes that the PID stored in `self` is an actual
    /// child process (and not a special value, such as `-1`).
    #[inline]
    #[doc(alias = "waitpid")]
    pub fn wait(self, opts: WaitOptions) -> Result<ExitStatus> {
        let mut status = 0;
        let ret = unsafe { libc::waitpid(self.as_raw(), &mut status, opts.bits()) };

        if ret == -1 {
            Err(crate::Errno::last())
        } else {
            Ok(ExitStatus::from_raw(status))
        }
    }

    /// Sends a signal to the current process.
    #[inline]
    pub fn kill(self, signal: Signal) -> Result<()> {
        let ret = unsafe { libc::kill(self.as_raw(), signal.as_raw()) };

        if ret == -1 {
            Err(crate::Errno::last())
        } else {
            Ok(())
        }
    }
}
