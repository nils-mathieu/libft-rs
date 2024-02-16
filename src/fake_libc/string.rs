use libc::{c_char, c_int};

/// Computes the size of the provided null-terminated string.
pub unsafe fn strlen(x: *const c_char) -> usize {
    let mut len = 0;
    while unsafe { *x.add(len) != 0 } {
        len += 1;
    }
    len
}

/// Compares the provided strings lexicographically.
pub unsafe fn strcmp(l: *const c_char, r: *const c_char) -> c_int {
    let mut i = 0;
    loop {
        let li = unsafe { *l.add(i) };
        let ri = unsafe { *r.add(i) };
        if li == 0 || li != ri {
            return li as u8 as i32 - ri as u8 as i32;
        }
        i += 1;
    }
}

/// Returns a pointer to the first occurance of `c` in `s`, or `NULL` if `c` is not found.
pub unsafe fn strchr(mut s: *const c_char, c: c_int) -> *const c_char {
    loop {
        let si = unsafe { *s };
        if si == 0 {
            return core::ptr::null();
        }
        if si == c as c_char {
            return s;
        }
        s = unsafe { s.add(1) };
    }
}

/// Returns the length of the provided null-terminated string, or `n` if no null terminator is found
/// within the first `n` bytes.
pub unsafe fn strnlen(s: *const c_char, n: usize) -> usize {
    unsafe {
        let mut len = 0;
        while len < n && *s.add(len) != 0 {
            len += 1;
        }
        len
    }
}
