use core::ffi::{c_char, c_int};

use crate::CharStar;

/// Computes the size of a null-terminated array.
///
/// # Safety
///
/// This function assumes that `array` contains a null pointer. This null pointer must be within
/// the allocated block of memory that `array` is a part of.
unsafe fn null_terminated_array_len(array: *const *const c_char) -> usize {
    let mut p = array;
    while !(*p).is_null() {
        p = p.add(1);
    }
    p.offset_from(array) as usize
}

/// The raw entry point called by the [`entry_point`] macro.
///
/// # Safety
///
/// Both `argv` and `envp` are assumed to be null-terminated arrays of null-terminated strings.
/// The data referenced by these arrays and strings must remain valid for the lifetime of the
/// entire program.
pub unsafe fn call<F>(f: F, argv: *const *const c_char, envp: *const *const c_char) -> c_int
where
    F: FnOnce(&'static [CharStar<'static>], &'static [CharStar<'static>]) -> u8,
{
    let argc = null_terminated_array_len(argv);
    let args = core::slice::from_raw_parts(argv as *const CharStar, argc);
    let envc = null_terminated_array_len(envp);
    let env = core::slice::from_raw_parts(envp as *const CharStar, envc);
    f(args, env) as c_int
}
