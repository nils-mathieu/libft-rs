use libc::{c_char, c_int, c_void};

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

/// Copies `n` bytes from `src` to `dst`.
pub unsafe fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    let mut d = dst.cast::<u8>();
    let mut s = src.cast::<u8>();
    let mut n = n;

    unsafe {
        while n > 0 {
            *d = *s;
            d = d.add(1);
            s = s.add(1);
            n -= 1;
        }
    }

    dst
}

/// Sets the first `n` bytes of `s` to `c`.
pub unsafe fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void {
    let mut s = s.cast::<u8>();
    let mut n = n;

    unsafe {
        while n > 0 {
            *s = c as u8;
            s = s.add(1);
            n -= 1;
        }
    }

    s.cast::<c_void>()
}

/// Compares the first `n` bytes of `l` and `r`.
pub unsafe fn memcmp(l: *const c_void, r: *const c_void, n: usize) -> c_int {
    let mut l = l.cast::<u8>();
    let mut r = r.cast::<u8>();
    let mut n = n;

    unsafe {
        while n > 0 {
            let li = *l;
            let ri = *r;
            if li != ri {
                return li as i32 - ri as i32;
            }
            l = l.add(1);
            r = r.add(1);
            n -= 1;
        }
    }

    0
}

/// Copies `n` bytes from `src` to `dst`, handling overlapping regions.
pub unsafe fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    let mut d = dst.cast::<u8>();
    let mut s = src.cast::<u8>();
    let mut n = n;

    unsafe {
        if d.cast_const() < s {
            while n > 0 {
                *d = *s;
                d = d.add(1);
                s = s.add(1);
                n -= 1;
            }
        } else {
            d = d.add(n);
            s = s.add(n);
            while n > 0 {
                d = d.sub(1);
                s = s.sub(1);
                *d = *s;
                n -= 1;
            }
        }
    }

    dst
}

/// Sets the first `n` bytes of `s` to zero.
#[inline]
pub unsafe fn bzero(s: *mut c_void, n: usize) {
    unsafe {
        memset(s, 0, n);
    }
}
