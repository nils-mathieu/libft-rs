//! Functions and types to work with processes.

mod signal;
pub use self::signal::*;

/// Aborts the current process.
pub fn abort() -> ! {
    unsafe { libc::abort() }
}
