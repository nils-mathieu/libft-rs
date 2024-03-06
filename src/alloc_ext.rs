use alloc::string::String;
use alloc::vec::Vec;

use crate::malloc::OutOfMemory;

/// An extension trait for [`Vec<T>`] that defines non-panicking methods
/// for adding elements to the vector.
pub trait SafeVecExt {
    type Item;

    /// Attempts to push an element into the vector.
    ///
    /// If the vector cannot allocate for more memory, this function simply
    /// returns an error.
    fn try_push(&mut self, item: Self::Item) -> Result<(), OutOfMemory>;

    /// Attempts to extend the vector with the elements from the provided slice.
    ///
    /// If the vector cannot allocate for more memory, this function simply
    /// returns an error.
    fn try_extend_from_slice(&mut self, slice: &[Self::Item]) -> Result<(), OutOfMemory>
    where
        Self::Item: Clone;
}

impl<T> SafeVecExt for Vec<T> {
    type Item = T;

    fn try_push(&mut self, item: Self::Item) -> Result<(), OutOfMemory> {
        self.try_reserve(1)?;
        self.push(item);
        Ok(())
    }

    fn try_extend_from_slice(&mut self, slice: &[Self::Item]) -> Result<(), OutOfMemory>
    where
        Self::Item: Clone,
    {
        self.try_reserve(slice.len())?;
        self.extend_from_slice(slice);
        Ok(())
    }
}

/// An extension trait for [`String`] that defines non-panicking methods
/// for adding elements to the string.
pub trait SafeStringExt {
    /// Attempts to push a string slice into the string.
    fn try_write_fmt(&mut self, args: core::fmt::Arguments<'_>) -> core::fmt::Result;
}

impl SafeStringExt for String {
    fn try_write_fmt(&mut self, args: core::fmt::Arguments<'_>) -> core::fmt::Result {
        struct Wrapper(String);

        impl Wrapper {
            #[inline]
            fn wrap(s: &mut String) -> &mut Wrapper {
                unsafe { &mut *(s as *mut String as *mut Wrapper) }
            }
        }

        impl core::fmt::Write for Wrapper {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                self.0.try_reserve(s.len()).map_err(|_| core::fmt::Error)?;
                self.0.push_str(s);
                Ok(())
            }

            fn write_char(&mut self, c: char) -> core::fmt::Result {
                self.0
                    .try_reserve(c.len_utf8())
                    .map_err(|_| core::fmt::Error)?;
                self.0.push(c);
                Ok(())
            }
        }

        core::fmt::Write::write_fmt(Wrapper::wrap(self), args)
    }
}
