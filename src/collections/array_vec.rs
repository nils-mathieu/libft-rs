//! An implementation of vector using a fixed-size array.

use core::borrow::{Borrow, BorrowMut};
use core::mem::{ManuallyDrop, MaybeUninit};
use core::ops::{Deref, DerefMut};

/// A vector using a fixed-size array as backing storage.
pub struct ArrayVec<T, const N: usize> {
    /// The backing array.
    array: [MaybeUninit<T>; N],
    /// The number of initialized elements in the vector.
    init: u8,
}

impl<T, const N: usize> ArrayVec<T, N> {
    /// Creates a new [`ArrayVec`] instance.
    pub const fn new() -> Self {
        assert!(N <= u8::MAX as usize, "N must be less than u8::MAX");

        Self {
            array: unsafe { MaybeUninit::uninit().assume_init() },
            init: 0,
        }
    }

    /// Returns whether the collection is full.
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.init as usize >= N
    }

    /// Pushes a new value into the vector without checking if enough space is available.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the vector has enough space to push a new value.
    #[inline(always)]
    pub unsafe fn push_unchecked(&mut self, value: T) {
        unsafe { (*self.array.as_mut_ptr().add(self.init as usize)).write(value) };
        self.init += 1;
    }

    /// Attempts to push a value into the vector.
    ///
    /// If the vector is already full, the value is returned back to the caller.
    #[inline]
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.is_full() {
            Err(value)
        } else {
            unsafe { self.push_unchecked(value) };
            Ok(())
        }
    }

    /// Pushes a new value into the vector.
    ///
    /// # Panics
    ///
    /// This function panics if the vector is already full.
    #[inline]
    pub fn push(&mut self, value: T) {
        if self.try_push(value).is_err() {
            panic!("ArrayVec is full");
        }
    }

    /// Pops the last element from the vector.
    pub fn pop(&mut self) -> Option<T> {
        if self.init == 0 {
            None
        } else {
            self.init -= 1;
            unsafe { Some((*self.array.as_mut_ptr().add(self.init as usize)).assume_init_read()) }
        }
    }

    /// Clears the vector, dropping all its elements.
    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    /// Attempts to remove the element at `index` from the vector, replacing it with the last
    /// element to avoid copies.
    ///
    /// If the order must be preserved, use [`remove`] instead.
    pub fn swap_remove(&mut self, index: usize) -> Option<T> {
        if index >= self.init as usize {
            return None;
        }

        self.init -= 1;
        unsafe {
            let last = &*self.array.as_ptr().add(self.init as usize);
            let hole = &mut *self.array.as_mut_ptr().add(index);

            let ret = hole.assume_init_read();
            hole.write(last.assume_init_read());
            Some(ret)
        }
    }

    /// Removes the element at `index` from the vector, shifting all elements after it to the
    /// left.
    ///
    /// If the order does not matter, use [`swap_remove`] instead.
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.init as usize {
            return None;
        }

        self.init -= 1;
        unsafe {
            let ret = (*self.array.as_ptr().add(index)).assume_init_read();

            core::ptr::copy(
                self.array.as_ptr().add(index).add(1),
                self.array.as_mut_ptr().add(index),
                self.init as usize - index,
            );

            Some(ret)
        }
    }

    /// Returns the inner array of the vector.
    #[inline]
    pub fn into_array(self) -> [MaybeUninit<T>; N] {
        let this = ManuallyDrop::new(self);
        unsafe { core::ptr::read(&this.array) }
    }
}

impl<T, const N: usize> Drop for ArrayVec<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, const N: usize> Deref for ArrayVec<T, N> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.array.as_ptr().cast(), self.init as usize) }
    }
}

impl<T, const N: usize> DerefMut for ArrayVec<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        let init = self.init as usize;
        unsafe { core::slice::from_raw_parts_mut(self.array.as_mut_ptr().cast(), init) }
    }
}

impl<T, const N: usize> Borrow<[T]> for ArrayVec<T, N> {
    #[inline]
    fn borrow(&self) -> &[T] {
        self.deref()
    }
}

impl<T, const N: usize> BorrowMut<[T]> for ArrayVec<T, N> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T] {
        self.deref_mut()
    }
}

impl<T, const N: usize> AsRef<[T]> for ArrayVec<T, N> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.deref()
    }
}

impl<T, const N: usize> AsMut<[T]> for ArrayVec<T, N> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.deref_mut()
    }
}

impl<T, const N: usize> IntoIterator for ArrayVec<T, N> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            tail: self.init as usize,
            head: 0,
            array: self.into_array(),
        }
    }
}

/// An iterator over the elements of an [`ArrayVec`].
pub struct IntoIter<T, const N: usize> {
    array: [MaybeUninit<T>; N],
    head: usize,
    tail: usize,
}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.head >= self.tail {
            return None;
        }

        unsafe {
            let ret = (*self.array.as_ptr().add(self.head)).assume_init_read();
            self.head += 1;
            Some(ret)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<T, const N: usize> ExactSizeIterator for IntoIter<T, N> {
    #[inline]
    fn len(&self) -> usize {
        self.tail - self.head
    }
}

impl<T, const N: usize> DoubleEndedIterator for IntoIter<T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.head >= self.tail {
            return None;
        }

        self.tail -= 1;
        unsafe { Some((*self.array.as_ptr().add(self.tail)).assume_init_read()) }
    }
}

impl<T, const N: usize> Drop for IntoIter<T, N> {
    fn drop(&mut self) {
        while self.next().is_some() {}
    }
}
