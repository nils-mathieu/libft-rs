//! Signals.

/// A signal that can be sent to a process.
pub enum Signal {
    /// The `SIGKILL` signal.
    Kill = libc::SIGKILL as _,
    /// The `SIGTERM` signal.
    Terminate = libc::SIGTERM as _,
    /// The `SIGSTOP` signal.
    Stop = libc::SIGSTOP as _,
    /// The `SIGCONT` signal.
    Continue = libc::SIGCONT as _,
    /// The `SIGINT` signal.
    Interrupt = libc::SIGINT as _,
}

impl Signal {
    /// Sends this signal to the current process.
    #[inline]
    pub fn raise(self) {
        unsafe { libc::raise(self as _) };
    }

    /// Marks this signal as being ignored.
    #[inline]
    pub fn set_handler_ignore(self) {
        unsafe { libc::signal(self as _, libc::SIG_IGN) };
    }

    /// Sets the handler for this signal to the default handler.
    #[inline]
    pub fn set_handler_default(self) {
        unsafe { libc::signal(self as _, libc::SIG_DFL) };
    }

    /// Sets the handler for this signal to the provided handler.
    #[inline]
    pub fn set_handler(self, handler: extern "C" fn()) {
        unsafe { libc::signal(self as _, handler as _) };
    }
}
