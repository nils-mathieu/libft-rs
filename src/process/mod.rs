//! Functions and types to work with processes.

/// Aborts the current process.
pub fn abort() -> ! {
    unsafe { libc::abort() }
}
