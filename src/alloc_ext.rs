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
