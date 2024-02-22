//! Signals.

/// A signal that can be sent to a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Signal(libc::c_int);

/// A signal that can be sent to a process.
impl Signal {
    /// The `SIGKILL` signal.
    pub const KILL: Self = Self(libc::SIGKILL);
    /// The `SIGTERM` signal.
    pub const TERM: Self = Self(libc::SIGTERM);
    /// The `SIGSTOP` signal.
    pub const STOP: Self = Self(libc::SIGSTOP);
    /// The `SIGCONT` signal.
    pub const CONT: Self = Self(libc::SIGCONT);
    /// The `SIGINT` signal.
    pub const INT: Self = Self(libc::SIGINT);
    /// The `SIGQUIT` signal.
    pub const QUIT: Self = Self(libc::SIGQUIT);
}

impl Signal {
    /// Creates a new `Signal` from the provided raw signal number.
    #[inline]
    pub const fn from_raw(signal: libc::c_int) -> Self {
        Self(signal)
    }

    /// Returns the raw signal number.
    #[inline]
    pub const fn as_raw(&self) -> libc::c_int {
        self.0
    }

    /// Sends this signal to the current process.
    #[inline]
    pub fn raise(self) {
        unsafe { libc::raise(self.as_raw()) };
    }

    /// Marks this signal as being ignored.
    #[inline]
    pub fn set_handler_ignore(self) -> OpaqueSigHandler {
        let ret = unsafe { libc::signal(self.as_raw(), libc::SIG_IGN) };
        debug_assert!(ret != libc::SIG_ERR);
        OpaqueSigHandler(ret)
    }

    /// Sets the handler for this signal to the default handler.
    #[inline]
    pub fn set_handler_default(self) -> OpaqueSigHandler {
        let ret = unsafe { libc::signal(self.as_raw(), libc::SIG_DFL) };
        debug_assert!(ret != libc::SIG_ERR);
        OpaqueSigHandler(ret)
    }

    /// Sets the handler for this signal to the provided handler.
    #[inline]
    pub fn set_handler(self, handler: extern "C" fn()) -> OpaqueSigHandler {
        let ret = unsafe { libc::signal(self.as_raw(), handler as _) };
        debug_assert!(ret != libc::SIG_ERR);
        OpaqueSigHandler(ret)
    }
}

/// An opaque signal handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpaqueSigHandler(libc::sighandler_t);
