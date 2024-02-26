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

    /// Sets the handler for this signal to the provided handler.
    ///
    /// # Returns
    ///
    /// The previous signal handler is returned.
    ///
    /// # Panics
    ///
    /// This function panics if `self` is not a valid signal.
    #[inline]
    #[track_caller]
    #[doc(alias = "signal")]
    pub fn set_handler(self, handler: SigHandler) -> SigHandler {
        handler
            .install(self)
            .unwrap_or_else(|| panic!("{:?} is not a valid signal", self))
    }

    /// Like [`set_handler`], but takes a function pointer directly.
    ///
    /// [`set_handler`]: Self::set_handler
    #[inline]
    #[track_caller]
    #[doc(alias = "signal")]
    pub fn set_handler_fn(self, f: extern "C" fn(Signal)) -> SigHandler {
        self.set_handler(SigHandler::from_fn(f))
    }
}

/// An opaque signal handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SigHandler(libc::sighandler_t);

impl SigHandler {
    /// The default signal handler.
    pub const DEFAULT: Self = Self(libc::SIG_DFL);

    /// The signal handler that ignores the signal.
    pub const IGNORE: Self = Self(libc::SIG_IGN);

    /// Creates a new [`SigHandler`] from the provided raw signal handler.
    #[inline]
    pub fn from_raw(handler: libc::sighandler_t) -> Self {
        Self(handler)
    }

    /// Returns the raw signal handler.
    #[inline]
    pub fn as_raw(&self) -> libc::sighandler_t {
        self.0
    }

    /// Creates a new [`SigHandler`] from the provided function pointer.
    #[inline]
    pub fn from_fn(f: extern "C" fn(Signal)) -> Self {
        Self(f as _)
    }

    /// Installs this signal handler for the provided signal.
    ///
    /// # Errors
    ///
    /// This function fails if the provided [`Signal`] is invalid.
    #[inline]
    #[doc(alias = "signal")]
    pub fn install(self, signal: Signal) -> Option<SigHandler> {
        let ret = unsafe { libc::signal(signal.as_raw(), self.0) };

        if ret == libc::SIG_ERR {
            None
        } else {
            Some(SigHandler(ret))
        }
    }

    /// Returns a guard that installs this [`SigHandler`] when dropped.
    #[inline]
    pub fn guard(self, signal: Signal) -> SigHandlerGuard {
        SigHandlerGuard(signal, self)
    }
}

impl From<extern "C" fn(Signal)> for SigHandler {
    #[inline]
    fn from(f: extern "C" fn(Signal)) -> Self {
        Self::from_fn(f)
    }
}

/// A guard that installs a specific [`SigHandler`] when dropped.
#[derive(Debug)]
pub struct SigHandlerGuard(Signal, SigHandler);

impl Drop for SigHandlerGuard {
    fn drop(&mut self) {
        self.0.set_handler(self.1);
    }
}
