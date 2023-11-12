//! Miscellaneous functions.

/// Schedules a function to be called *after* main, when the program exits.
///
/// # Returns
///
/// This function returns whether the registration was successful. Specifically,
/// this function may fail if the maximum number of registered function has been
/// reached.
#[inline]
pub fn at_exit(f: extern "C" fn()) -> bool {
    unsafe { libc::atexit(f) == 0 }
}
