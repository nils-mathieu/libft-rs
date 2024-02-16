use core::ffi::{c_int, c_void};

#[no_mangle]
unsafe extern "C" fn memcmp(l: *const c_void, r: *const c_void, n: usize) -> c_int {
    unsafe { super::memcmp(l, r, n) }
}

#[no_mangle]
unsafe extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    unsafe { super::memcpy(dst, src, n) }
}

#[no_mangle]
unsafe extern "C" fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void {
    unsafe { super::memset(s, c, n) }
}

#[no_mangle]
pub unsafe extern "C" fn memmove(s: *mut c_void, n: *const c_void, len: usize) -> *mut c_void {
    unsafe { super::memmove(s, n, len) }
}

#[no_mangle]
pub unsafe extern "C" fn bzero(s: *mut c_void, n: usize) {
    unsafe { super::bzero(s, n) }
}
