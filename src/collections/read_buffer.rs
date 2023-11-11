//! Defines the [`ReadBuffer`] collection.

use core::mem::MaybeUninit;
use core::ptr::NonNull;

use crate::malloc::Result;

/// A collection of buffers that can be used to read data from a file descriptor efficiently.
///
/// # Representation
///
/// The buffer is made of three distinct parts:
///
/// ```text
/// +-------------------+-------------------+-------------------+
/// | Consumed          | Pending           | Uninitialized     |
/// +-------------------+-------------------+-------------------+
/// ^                   ^                   ^                   ^
/// data                tail                head                capacity
/// ```
///
/// - The **consumed** part of the buffer contains data that has already been read from the
///   file descriptor and consumed by the user. This part may be overwritten by the next
///   read operation.
///
/// - The **pending** part of the buffer contains data that has already been read from the
///   file descriptor but not yet consumed by the user. This part won't be overwritten by
///   the next read operation.
///
/// - The **uninitialized** part of the buffer contains data that has not been read from the
///   file descriptor yet.
pub struct ReadBuffer {
    /// The data buffer used to read data from the file descriptor.
    data: NonNull<u8>,
    /// The total capacity of the data buffer.
    capacity: usize,
    /// The head of the data buffer.
    ///
    /// Every byte before this index is guaranteed to be filled with some data.
    head: usize,
    /// The tail of the data buffer.
    ///
    /// Every byte after this index has not been consumed yet and must not be overwritten.
    tail: usize,
}

impl ReadBuffer {
    /// Creates a new [`ReadBuffer`] instance.
    pub const fn new() -> Self {
        Self {
            data: NonNull::dangling(),
            capacity: 0,
            head: 0,
            tail: 0,
        }
    }

    /// Creates a new [`ReadBuffer`] instance with the provided capacity.
    pub fn with_capacity(cap: usize) -> Result<Self> {
        let mut readbuf = Self::new();
        readbuf.reserve(cap)?;
        Ok(readbuf)
    }

    /// Returns the number of bytes that are currently pending in the buffer.
    ///
    /// Pending bytes are the part of the buffer that has been read, but not yet consumed.
    #[inline]
    pub fn pending(&self) -> usize {
        self.head - self.tail
    }

    /// Assumes that `count` additional bytes have been written to the buffer in its
    /// uninitialized part.
    ///
    /// # Safety
    ///
    /// This function assumes that `count` bytes really have been written to the buffer.
    #[inline]
    pub unsafe fn assume_init(&mut self, count: usize) {
        debug_assert!(self.head + count <= self.capacity);
        self.head += count;
    }

    /// Makes sure that the buffer has enough spare capacity to hold `count` additional bytes.
    ///
    /// # Errors
    ///
    /// This function may fail if the system ran out of memory. In that case, [`OutOfMemory`]
    /// is returned.
    pub fn reserve(&mut self, count: usize) -> Result<()> {
        // There's three cases to consider:
        // 1. The buffer has enough spare capacity right now.
        // 2. The buffer would have enough spare capacity if we overwrote the consumed part.
        // 3. The buffer would *not* have enough spare capacity even if we overwrote the
        //    consumed part. In that case, we need to reallocate the buffer.

        let uninit = self.capacity - self.head;
        if count <= uninit {
            // Case 1: we have enough spare capacity.
            return Ok(());
        }

        let after_move = uninit + self.tail;

        if count <= after_move {
            // Case 2: we have enough spare capacity if we move the consumed part.

            unsafe {
                core::ptr::copy(
                    self.data.as_ptr().add(self.tail),
                    self.data.as_ptr(),
                    self.head - self.tail,
                );
            }

            self.head -= self.tail;
            self.tail = 0;

            return Ok(());
        }

        // Case 3: we have to reallocate the buffer.

        let new_capacity = after_move + count;
        let new_data = crate::malloc::allocate(new_capacity)?;
        let new_capacity = new_data.len();
        let new_data = new_data.as_non_null_ptr();

        if self.capacity != 0 {
            // We have to relocate the data to the new buffer and
            // deallocate the old buffer.

            unsafe {
                core::ptr::copy(
                    self.data.as_ptr().add(self.tail),
                    new_data.as_ptr(),
                    self.head - self.tail,
                );

                crate::malloc::deallocate(self.data);
            }
        }

        self.data = new_data;
        self.capacity = new_capacity;
        self.head -= self.tail;
        self.tail = 0;

        Ok(())
    }

    /// Returns the part of the buffer that has not been initialized yet.
    ///
    /// This function is usually coupled with a call to [`assume_init`] to
    /// mark the returned bytes as initialized.
    ///
    /// [`assume_init`]: ReadBuffer::assume_init
    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        let len = self.capacity - self.head;
        let ptr = unsafe { self.data.as_ptr().add(self.head) };
        unsafe { core::slice::from_raw_parts_mut(ptr.cast(), len) }
    }

    /// Consumes `count` bytes from the buffer, marking them as "unneeded".
    ///
    /// The consumed bytes may be overwritten by the next read operation.
    ///
    /// # Safety
    ///
    /// `count` must be less than or equal to `pending()`.
    ///
    /// # Returns
    ///
    /// This function returns the part of the buffer that has been consumed.
    pub unsafe fn consume_unchecked(&mut self, count: usize) -> &mut [u8] {
        debug_assert!(self.tail + count <= self.head);

        let consumed_ptr = self.data.as_ptr().add(self.tail);

        self.tail += count;

        // We just consumed the whole buffer, we can reset
        // the whole thing to its initial state wihout
        // copying anything!
        if self.tail == self.head {
            self.tail = 0;
            self.head = 0;
        }

        core::slice::from_raw_parts_mut(consumed_ptr, count)
    }

    /// Fills the buffer with additional data by reading from the provided file descriptor,
    /// returning the part of the buffer that has been filled.
    pub fn fill_with_fd(&mut self, fd: crate::Fd) -> crate::Result<&[u8]> {
        let count = fd.read(self.spare_capacity_mut())?;
        let ptr = unsafe { self.data.as_ptr().add(self.head) };
        unsafe { self.assume_init(count) };
        Ok(unsafe { core::slice::from_raw_parts(ptr, count) })
    }
}

impl Drop for ReadBuffer {
    fn drop(&mut self) {
        if self.capacity != 0 {
            unsafe {
                crate::malloc::deallocate(self.data);
            }
        }
    }
}
