//! Defines a global allocator that uses `malloc` and `free`.

use core::alloc::GlobalAlloc;
use core::ffi::c_void;
use core::mem::align_of;

#[global_allocator]
static MALLOC: Malloc = Malloc;

/// An implementation of [`GlobalAlloc`] that uses `malloc` and `free`.
struct Malloc;

unsafe impl GlobalAlloc for Malloc {
    #[inline]
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        if layout.align() > align_of::<usize>() {
            return core::ptr::null_mut();
        }

        unsafe { libc::malloc(layout.size()) as *mut u8 }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        if layout.align() > align_of::<usize>() {
            return core::ptr::null_mut();
        }

        unsafe { libc::calloc(layout.size(), 1) as *mut u8 }
    }

    #[inline]
    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        _layout: core::alloc::Layout,
        new_size: usize,
    ) -> *mut u8 {
        unsafe { libc::realloc(ptr as *mut c_void, new_size) as *mut u8 }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        unsafe { libc::free(ptr as *mut c_void) };
    }
}
