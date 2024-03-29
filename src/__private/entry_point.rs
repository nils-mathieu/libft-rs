use core::ffi::{c_char, c_int};

use crate::entry_point::Terminate;
use crate::CharStar;

/// Computes the size of a null-terminated array.
///
/// # Safety
///
/// This function assumes that `array` contains a null pointer. This null pointer must be within
/// the allocated block of memory that `array` is a part of.
unsafe fn null_terminated_array_len(array: *const *const c_char) -> usize {
    unsafe {
        let mut p = array;
        while !(*p).is_null() {
            p = p.add(1);
        }
        p.offset_from(array) as usize
    }
}

/// Calls the function `f` using the provided `argv` and `envp` pointers.
///
/// # Safety
///
/// Both `argv` and `envp` are assumed to be null-terminated arrays of null-terminated strings.
/// The data referenced by these arrays and strings must remain valid for the lifetime of the
/// entire program.
pub unsafe fn call<F, R>(f: F, argv: *const *const c_char, envp: *const *const c_char) -> c_int
where
    F: FnOnce(&'static [&'static CharStar], &'static [&'static CharStar]) -> R,
    R: Terminate,
{
    unsafe {
        let argc = null_terminated_array_len(argv);
        let args = core::slice::from_raw_parts(argv as *const &CharStar, argc);
        let envc = null_terminated_array_len(envp);
        let env = core::slice::from_raw_parts(envp as *const &CharStar, envc);
        f(args, env).terminate()
    }
}
