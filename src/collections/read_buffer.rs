//! Defines the [`ReadBuffer`] collection.

use core::mem::MaybeUninit;
use core::ptr::NonNull;

#[cfg(feature = "futures")]
use crate::futures;
use crate::malloc::OutOfMemory;
use crate::{Errno, Fd, MemchrExt};

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

unsafe impl Send for ReadBuffer {}
unsafe impl Sync for ReadBuffer {}

impl Default for ReadBuffer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
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
    pub fn with_capacity(cap: usize) -> Result<Self, OutOfMemory> {
        let mut readbuf = Self::new();
        readbuf.reserve(cap)?;
        Ok(readbuf)
    }

    /// Returns the number of bytes that are currently pending in the buffer.
    ///
    /// Pending bytes are the part of the buffer that has been read, but not yet consumed.
    #[inline]
    pub fn pending(&self) -> &[u8] {
        unsafe {
            let p = self.data.as_ptr().add(self.tail);
            let len = self.head - self.tail;
            core::slice::from_raw_parts(p, len)
        }
    }

    /// Returns a slice over the part of the buffer that has been consumed.
    ///
    /// Note that this function is not reliable as the buffer may overwrite the consumed
    /// part when:
    ///
    /// 1. It needs to reallocate its internal buffer.
    ///
    /// 2. It needs to move the consumed part to make room for more data.
    #[inline]
    pub fn consumed(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.data.as_ptr(), self.tail) }
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
    ///
    /// [`OutOfMemory`]: crate::malloc::OutOfMemory
    pub fn reserve(&mut self, count: usize) -> Result<(), OutOfMemory> {
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
            // Case 2: we have enough spare capacity if we remove the consumed part.

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

        let new_capacity = self.head - self.tail + count;
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
    /// # Panics
    ///
    /// This function panics if `count` is greater than `pending().len()`.
    #[inline]
    pub fn consume(&mut self, count: usize) -> &mut [u8] {
        assert!(self.tail + count <= self.head);
        unsafe { self.consume_unchecked(count) }
    }

    /// Consumes `count` bytes from the buffer, marking them as "unneeded".
    ///
    /// The consumed bytes may be overwritten by the next read operation.
    ///
    /// # Safety
    ///
    /// `count` must be less than or equal to `pending().len()`.
    ///
    /// # Returns
    ///
    /// This function returns the part of the buffer that has been consumed.
    pub unsafe fn consume_unchecked(&mut self, count: usize) -> &mut [u8] {
        let ret =
            unsafe { core::slice::from_raw_parts_mut(self.data.as_ptr().add(self.tail), count) };

        self.tail += count;

        // We just consumed the whole buffer, we can reset
        // the whole thing to its initial state wihout
        // copying anything!
        if self.tail == self.head {
            self.tail = 0;
            self.head = 0;
        }

        ret
    }

    /// Fills the buffer with additional data by reading from the provided file descriptor,
    /// returning the part of the buffer that has been filled.
    ///
    /// # Remarks
    ///
    /// This function doesn't attempt to allocate more memory if the buffer
    /// is full. Instead, it will succeed with a size of 0.
    ///
    /// One should generall call [`reserve`] before calling this function to make sure
    /// that at least *some* additional space is available.
    ///
    /// [`reserve`]: ReadBuffer::reserve
    pub fn fill_with_fd(&mut self, fd: Fd) -> Result<usize, Errno> {
        let count = fd.read(self.spare_capacity_mut())?;

        unsafe {
            self.assume_init(count);
            Ok(count)
        }
    }

    /// Like [`fill_with_fd`](Self::fill_with_fd), but asynchronous.
    ///
    /// This function assumes that the provided file descriptor is non-blocking.
    #[inline]
    #[cfg(feature = "futures")]
    pub fn async_fill_with_fd(&mut self, fd: Fd) -> futures::FillWithFd {
        futures::FillWithFd { buf: self, fd }
    }

    /// Reads from the file descriptor until the given delimiter is found.
    ///
    /// The delimiter is included in the buffer.
    ///
    /// The buffer can be accessed through the `pending` method.
    ///
    /// # Remarks
    ///
    /// The current pending buffer is checked. If a delimiter is found in the current pending
    /// buffer, it will be immediately returned. The returned part of the buffer will be consumed
    /// and the next call to `read_until` will continue reading from the file descriptor (or parsing
    /// the existing pending buffer).
    pub fn read_until(&mut self, fd: Fd, delimiter: &[u8]) -> Result<&mut [u8], Errno> {
        if delimiter.is_empty() {
            return Ok(&mut []);
        }

        let mut batch_size = 64;

        loop {
            unsafe {
                if let Some(index) = self.pending().memchr(*delimiter.get_unchecked(0)) {
                    if self.pending().get_unchecked(index..).starts_with(delimiter) {
                        return Ok(self.consume_unchecked(index + delimiter.len()));
                    }
                }
            }

            self.reserve(batch_size)?;
            self.fill_with_fd(fd)?;

            batch_size = batch_size.saturating_mul(2);
        }
    }

    /// Reads exactly `count` bytes from the file descriptor (or from the pending buffer if it is
    /// not empty).
    ///
    /// # Remarks
    ///
    /// The current pending buffer is checked. If it contains at least `count` bytes, they will be
    /// immediately returned. The returned part of the buffer will be consumed and the next call to
    /// `read_exact` will continue reading from the file descriptor (or parsing the existing pending
    /// buffer).
    pub fn read_exact(&mut self, fd: Fd, count: usize) -> Result<&mut [u8], Errno> {
        loop {
            if self.pending().len() >= count {
                return unsafe { Ok(self.consume_unchecked(count)) };
            }

            self.reserve(count.saturating_sub(self.pending().len()))?;
            self.fill_with_fd(fd)?;
        }
    }

    /// Like [`read_until`](Self::read_until), but asynchronous.
    ///
    /// This function assumes that the provided file descriptor is non-blocking.
    #[inline]
    #[cfg(feature = "futures")]
    pub fn async_read_until<'a, 'd>(
        &'a mut self,
        fd: Fd,
        delimiter: &'d [u8],
    ) -> futures::ReadUntil<'a, 'd> {
        futures::ReadUntil::new(fd, self, delimiter)
    }

    /// Like [`read_exact`](Self::read_exact), but asynchronous.
    ///
    /// This function assumes that the provided file descriptor is non-blocking.
    #[inline]
    #[cfg(feature = "futures")]
    pub fn async_read_exact(&mut self, fd: Fd, count: usize) -> futures::ReadExact {
        futures::ReadExact::new(fd, self, count)
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
