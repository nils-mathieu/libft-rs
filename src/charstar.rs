//! Provides functions to work with C-like strings.

use core::ffi::c_char;
use core::fmt;
use core::iter::FusedIterator;

/// Creates a [`CharStar`] instance from a string literal.
///
/// # Examples
///
/// ```
/// # use ft::charstar;
/// #
/// let s = charstar!("Hello, world!");
/// assert_eq!(s.to_str(), Some("Hello, world!"));
/// ```
#[macro_export]
macro_rules! charstar {
    ($s:literal) => {
        unsafe {
            $crate::CharStar::from_ptr(concat!($s, "\0").as_ptr() as *const ::core::ffi::c_char)
        }
    };
}

extern "C" {
    /// This type is the only thing that [`CharStar`] contains. It ensures that the
    /// type cannot be constructed on the stack (because it has no fixed size). Instead,
    /// its size is derived from the pointer it is constructed from.
    type Unsized;
}

/// Represents a C-like string (null-terminated array of bytes).
#[repr(transparent)]
pub struct CharStar(Unsized);

unsafe impl Sync for CharStar {}
unsafe impl Send for CharStar {}

impl CharStar {
    /// Creates a new [`CharStar`] instance from the provided pointer.
    ///
    /// # Safety
    ///
    /// This function assumes that `data` is a null-terminated string. At some points
    /// in the data it references, there is a null byte (`0x00`) marking the end of the string.
    ///
    /// The memory referenced by `data`, up and including the null byte, must remain valid and
    /// borrowed for the lifetime `'a`.
    pub const unsafe fn from_ptr<'a>(data: *const c_char) -> &'a Self {
        &*(data as *const Self)
    }

    /// Returns a pointer to the first byte of the string.
    pub const fn as_ptr(&self) -> *const c_char {
        self as *const Self as *const c_char
    }

    /// Returns whether the string contains no bytes.
    pub fn is_empty(&self) -> bool {
        unsafe { *self.as_ptr() == 0 }
    }

    /// Returns the length of the string, not including the terminating null byte.
    pub fn len(&self) -> usize {
        unsafe { libc::strlen(self.as_ptr()) }
    }

    /// Returns the length of the string, or `max` if the string is longer than `max`. The
    /// terminating null byte is not included in the length.
    pub fn len_bounded(&self, max: usize) -> usize {
        unsafe { libc::strnlen(self.as_ptr(), max) }
    }

    /// Returns the bytes of the string, not including the terminating null byte.
    pub fn as_bytes(&self) -> &[u8] {
        let len = self.len();
        unsafe { core::slice::from_raw_parts(self.as_ptr() as *const u8, len) }
    }

    /// Returns the bytes of the string, or the first `max` bytes if the string is longer than
    /// `max`. The terminating null byte is not included in the returned slice.
    pub fn as_bytes_bounded(&self, max: usize) -> &[u8] {
        let len = self.len_bounded(max);
        unsafe { core::slice::from_raw_parts(self.as_ptr() as *const u8, len) }
    }

    /// Attempts to convert this [`CharStar`] into a [`&str`].
    ///
    /// # Errors
    ///
    /// This function fails with [`None`] if the string is not valid UTF-8.
    pub fn as_str(&self) -> Option<&str> {
        core::str::from_utf8(self.as_bytes()).ok()
    }

    /// Returns whether this [`CharStar`] starts with the provided prefix.
    pub fn starts_with(&self, prefix: &[u8]) -> bool {
        let len = self.as_bytes_bounded(prefix.len());
        len == prefix
    }

    /// If this [`CharStar`] is not empty, returns the first byte of the string and a
    /// [`CharStar`] containing the remaining bytes.
    pub fn split_first(&self) -> Option<(u8, &Self)> {
        let p = self.as_ptr();
        let b = unsafe { *p };

        if b == 0 {
            None
        } else {
            let rest = unsafe { Self::from_ptr(p.add(1)) };
            Some((b as u8, rest))
        }
    }

    /// Creates an iterator over the bytes of the string.
    pub fn iter(&self) -> Iter {
        self.into_iter()
    }
}

impl fmt::Debug for CharStar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Some(s) => fmt::Debug::fmt(s, f),
            None => f.write_str(INVALID_UTF8),
        }
    }
}

impl fmt::Display for CharStar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_str().unwrap_or(INVALID_UTF8))
    }
}

impl PartialEq<CharStar> for CharStar {
    fn eq(&self, other: &CharStar) -> bool {
        unsafe { libc::strcmp(self.as_ptr(), other.as_ptr()) == 0 }
    }
}

impl PartialEq<str> for CharStar {
    fn eq(&self, other: &str) -> bool {
        self.iter().eq(other.bytes())
    }
}

impl PartialEq<[u8]> for CharStar {
    fn eq(&self, other: &[u8]) -> bool {
        self.iter().eq(other.iter().copied())
    }
}

impl PartialEq<CharStar> for str {
    fn eq(&self, other: &CharStar) -> bool {
        self.bytes().eq(other.iter())
    }
}

impl PartialEq<CharStar> for [u8] {
    fn eq(&self, other: &CharStar) -> bool {
        self.iter().copied().eq(other.iter())
    }
}

impl Eq for CharStar {}

impl PartialOrd<CharStar> for CharStar {
    fn partial_cmp(&self, other: &CharStar) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CharStar {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        unsafe { libc::strcmp(self.as_ptr(), other.as_ptr()).cmp(&0) }
    }
}

/// An iterator over the bytes of a [`CharStar`].
#[derive(Debug, Clone)]
pub struct Iter<'a>(&'a CharStar);

impl Iterator for Iter<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.split_first() {
            Some((b, rest)) => {
                self.0 = rest;
                Some(b)
            }
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.0.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for Iter<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for Iter<'_> {}

impl<'a> IntoIterator for &'a CharStar {
    type Item = u8;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self)
    }
}

/// A string to use when invalid UTF-8 is encountered.
const INVALID_UTF8: &str = "<invalid utf-8>";
