//! Provides functions to work with C-like strings.

use core::ffi::c_char;
use core::fmt;
use core::marker::PhantomData;

/// Represents a C-like string (null-terminated array of bytes).
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct CharStar<'a> {
    /// A pointer to the first byte of the string.
    data: *const c_char,
    /// The lifetime of the string.
    ///
    /// The bytes referenced by `data` are guaranteed to be valid for the lifetime `'a`.
    _lifetime: PhantomData<&'a [c_char]>,
}

unsafe impl Sync for CharStar<'_> {}
unsafe impl Send for CharStar<'_> {}

impl<'a> CharStar<'a> {
    /// Creates a new [`CharStar`] instance from the provided pointer.
    ///
    /// # Safety
    ///
    /// This function assumes that `data` is a null-terminated string. At some points
    /// in the data it references, there is a null byte (`0x00`) marking the end of the string.
    ///
    /// The memory referenced by `data`, up and including the null byte, must remain valid and
    /// borrowed for the lifetime `'a`.
    pub const unsafe fn new(data: *const c_char) -> Self {
        Self {
            data,
            _lifetime: PhantomData,
        }
    }

    /// Returns whether the string contains no bytes.
    pub fn is_empty(self) -> bool {
        unsafe { *self.data == 0 }
    }

    /// Returns the length of the string.
    pub fn len(self) -> usize {
        unsafe { libc::strlen(self.data) }
    }

    /// Returns the bytes of the string, not including the terminating null byte.
    pub fn as_bytes(self) -> &'a [u8] {
        let len = self.len();
        unsafe { core::slice::from_raw_parts(self.data as *const u8, len) }
    }

    /// Attempts to convert this [`CharStar`] into a [`&str`].
    ///
    /// # Errors
    ///
    /// This function fails with [`None`] if the string is not valid UTF-8.
    pub fn to_str(self) -> Option<&'a str> {
        core::str::from_utf8(self.as_bytes()).ok()
    }
}

impl<'a> fmt::Debug for CharStar<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_str() {
            Some(s) => fmt::Debug::fmt(s, f),
            None => f.write_str(INVALID_UTF8),
        }
    }
}

impl<'a> fmt::Display for CharStar<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.to_str().unwrap_or(INVALID_UTF8))
    }
}

/// A string to use when invalid UTF-8 is encountered.
const INVALID_UTF8: &str = "<invalid utf-8>";
