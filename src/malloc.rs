//! Memory allocation functions.

use core::fmt;
use core::ptr::NonNull;

/// An error that occurs when an allocation fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutOfMemory;

impl fmt::Display for OutOfMemory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("out of memory")
    }
}

impl From<OutOfMemory> for crate::Errno {
    #[inline]
    fn from(_value: OutOfMemory) -> Self {
        Self::NO_MEMORY
    }
}

/// The result type for memory-backed collections.
pub type Result<T> = ::core::result::Result<T, OutOfMemory>;

/// Allocates a block of memory of the given size.
///
/// The returned pointer is guaranteed to be aligned to the native word size of the target
/// architecture.
///
/// # Errors
///
/// This function may fail if the system ran out of memory. In that case, [`OutOfMemory`] is
/// returned.
///
/// # Remarks
///
/// The returned pointer should be freed at some point using [`deallocate`] or memory
/// will leak.
#[allow(unused_mut)]
pub fn allocate(mut size: usize) -> Result<NonNull<[u8]>> {
    #[cfg(all(target_os = "macos", not(feature = "restrict-functions")))]
    {
        size = unsafe { libc::malloc_good_size(size) };
    }

    let ptr = unsafe { libc::malloc(size) };

    if ptr.is_null() {
        return Err(OutOfMemory);
    }

    #[cfg(all(target_os = "macos", not(feature = "restrict-functions")))]
    {
        size = unsafe { libc::malloc_size(ptr) };
    }

    #[cfg(all(target_os = "linux", not(feature = "restrict-functions")))]
    {
        size = unsafe { libc::malloc_usable_size(ptr) };
    }

    Ok(NonNull::slice_from_raw_parts(
        unsafe { NonNull::new_unchecked(ptr.cast()) },
        size,
    ))
}

/// Deallocates the provided memory block.
///
/// # Safety
///
/// The provided pointer must come from a memory allocation function such as [`allocate`].
#[inline]
pub unsafe fn deallocate(ptr: NonNull<u8>) {
    unsafe { libc::free(ptr.as_ptr().cast()) }
}
