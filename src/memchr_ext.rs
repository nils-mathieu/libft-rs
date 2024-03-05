#[cfg(feature = "restrict-functions")]
use crate::fake_libc as c;
#[cfg(not(feature = "restrict-functions"))]
use libc as c;

/// An extension trait for slices of bytes, providing a `memchr` method.
pub trait MemchrExt {
    /// Returns the index of the first occurrence of `byte` in `self`.
    fn memchr(&self, byte: u8) -> Option<usize>;
}

impl MemchrExt for [u8] {
    #[inline]
    fn memchr(&self, byte: u8) -> Option<usize> {
        let p = unsafe { c::memchr(self.as_ptr().cast(), byte as u32 as i32, self.len()) };

        if p.is_null() {
            None
        } else {
            Some(unsafe { p.byte_offset_from(self.as_ptr()) as usize })
        }
    }
}
