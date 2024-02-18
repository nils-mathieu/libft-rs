//! Provides functions to work with C-like strings.

use core::ffi::{c_char, c_int};
use core::fmt;
use core::iter::FusedIterator;

#[cfg(feature = "restrict-functions")]
use crate::fake_libc as c;
use crate::utils::display_bytes;
#[cfg(not(feature = "restrict-functions"))]
use libc as c;

/// Creates a [`CharStar`] instance from a string literal.
///
/// # Examples
///
/// ```ignore
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
    #[inline]
    pub const unsafe fn from_ptr<'a>(data: *const c_char) -> &'a Self {
        unsafe { &*(data as *const Self) }
    }

    /// Creates a new [`CharStar`] instance from the provided bytes.
    ///
    /// If the provided slice does not include a null character, [`None`] is returned.
    #[inline]
    pub fn from_bytes_until_nul(bytes: &[u8]) -> Option<&Self> {
        if bytes.contains(&b'\0') {
            Some(unsafe { Self::from_ptr(bytes.as_ptr() as *const c_char) })
        } else {
            None
        }
    }

    /// Returns a pointer to the first byte of the string.
    #[inline]
    pub const fn as_ptr(&self) -> *const c_char {
        self as *const Self as *const c_char
    }

    /// Returns whether the string contains no bytes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        unsafe { *self.as_ptr() == 0 }
    }

    /// Returns the length of the string, not including the terminating null byte.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { c::strlen(self.as_ptr()) }
    }

    /// Returns the length of the string, or `max` if the string is longer than `max`. The
    /// terminating null byte is not included in the length.
    #[inline]
    pub fn len_bounded(&self, max: usize) -> usize {
        unsafe { c::strnlen(self.as_ptr(), max) }
    }

    /// Returns whether the length of the string is strictly less than `len`.
    #[inline]
    pub fn len_less_than(&self, len: usize) -> bool {
        self.len_bounded(len) != len
    }

    /// Returns the bytes of the string, not including the terminating null byte.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        let len = self.len();
        unsafe { core::slice::from_raw_parts(self.as_ptr() as *const u8, len) }
    }

    /// Returns the bytes of the string, or the first `max` bytes if the string is longer than
    /// `max`. The terminating null byte is not included in the returned slice.
    #[inline]
    pub fn as_bytes_bounded(&self, max: usize) -> &[u8] {
        let len = self.len_bounded(max);
        unsafe { core::slice::from_raw_parts(self.as_ptr() as *const u8, len) }
    }

    /// Attempts to convert this [`CharStar`] into a [`&str`].
    ///
    /// # Errors
    ///
    /// This function fails with [`None`] if the string is not valid UTF-8.
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        core::str::from_utf8(self.as_bytes()).ok()
    }

    /// Returns whether this [`CharStar`] starts with the provided prefix.
    pub fn starts_with(&self, prefix: &[u8]) -> bool {
        let len = self.as_bytes_bounded(prefix.len());
        len == prefix
    }

    /// If the string starts with the provided prefix, this function returns a
    /// [`CharStar`] that does not include it.
    ///
    /// Otherwise, it returns [`None`].
    pub fn strip_prefix(&self, prefix: &[u8]) -> Option<&Self> {
        if self.starts_with(prefix) {
            Some(unsafe { self.split_at_unchecked(prefix.len()).1 })
        } else {
            None
        }
    }

    /// Returns a [`CharStar`] starting at the first byte of the string equal to `c`.
    ///
    /// If the character is not found, returns [`None`].
    #[inline]
    pub fn advance_at_char(&self, c: u8) -> Option<&Self> {
        let p = unsafe { c::strchr(self.as_ptr(), c as c_int) };

        if p.is_null() {
            None
        } else {
            Some(unsafe { Self::from_ptr(p) })
        }
    }

    /// Splits the string at the first byte equal to `c`.
    ///
    /// If the character is not found, returns [`None`].
    pub fn split_at_char(&self, c: u8) -> Option<(&[u8], &Self)> {
        let p = unsafe { c::strchr(self.as_ptr(), c as c_int) };

        if p.is_null() {
            None
        } else {
            unsafe {
                let len = p.offset_from(self.as_ptr()) as usize;
                let init = core::slice::from_raw_parts(self.as_ptr() as *const u8, len);
                let rest = Self::from_ptr(p);
                Some((init, rest))
            }
        }
    }

    /// Splits the string at the provided index. The index is not checked to ensure that it
    /// is within the bounds of the string.
    ///
    /// # Safety
    ///
    /// The provided index must be within the bounds of the string (meaning `<= len`).
    pub const unsafe fn split_at_unchecked(&self, index: usize) -> (&[u8], &Self) {
        let p = self.as_ptr();

        unsafe {
            (
                core::slice::from_raw_parts(p as *const u8, index),
                Self::from_ptr(p.add(index)),
            )
        }
    }

    /// Returns the index of the first byte of the string equal to `c`.
    #[inline]
    pub fn index_of(&self, c: u8) -> Option<usize> {
        let p = unsafe { c::strchr(self.as_ptr(), c as c_int) };

        if p.is_null() {
            None
        } else {
            Some(unsafe { p.offset_from(self.as_ptr()) as usize })
        }
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
    #[inline]
    pub fn iter(&self) -> Iter {
        self.into_iter()
    }

    /// Split the string using the provided byte delimiter, returning an iterator
    /// over the resulting substrings.
    #[inline]
    pub fn split(&self, byte: u8) -> Split {
        Split {
            charstar: self,
            byte,
        }
    }
}

impl fmt::Debug for CharStar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(display_bytes(self.as_bytes()), f)
    }
}

impl fmt::Display for CharStar {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(display_bytes(self.as_bytes()), f)
    }
}

impl PartialEq<CharStar> for CharStar {
    #[inline]
    fn eq(&self, other: &CharStar) -> bool {
        unsafe { c::strcmp(self.as_ptr(), other.as_ptr()) == 0 }
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
    #[inline]
    fn partial_cmp(&self, other: &CharStar) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CharStar {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        unsafe { c::strcmp(self.as_ptr(), other.as_ptr()).cmp(&0) }
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

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter(self)
    }
}

/// An iterator over the bytes of a [`CharStar`], split at a given byte.
pub struct Split<'a> {
    charstar: &'a CharStar,
    byte: u8,
}

impl<'a> Iterator for Split<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (init, rest) = self.charstar.split_at_char(self.byte)?;
        self.charstar = rest;
        Some(init)
    }
}
