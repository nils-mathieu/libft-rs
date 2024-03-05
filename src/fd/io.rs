use core::ffi::c_void;
use core::fmt;
use core::mem::MaybeUninit;

use libc::c_uint;

use super::{Mode, OpenFlags};
#[cfg(feature = "futures")]
use crate::futures;
use crate::{CharStar, Errno, Fd, File, Result};

impl Fd {
    /// Opens a file for reading only.
    ///
    /// # Returns
    ///
    /// A [`Fd`] instance representing the opened file.
    #[inline]
    pub fn open(path: &CharStar, flags: OpenFlags) -> Result<Self> {
        let res = unsafe { libc::open(path.as_ptr(), flags.bits()) };
        if res < 0 {
            Err(Errno::last())
        } else {
            Ok(Self(res))
        }
    }

    /// Opens a file with a specific mode value.
    ///
    /// This is only really useful when the `CREATE` flag is set.
    ///
    /// # Returns
    ///
    /// A [`Fd`] instance representing the opened file.
    #[inline]
    #[doc(alias = "open")]
    pub fn open_with_mode(path: &CharStar, flags: OpenFlags, mode: Mode) -> Result<Self> {
        let res = unsafe { libc::open(path.as_ptr(), flags.bits(), mode.bits() as c_uint) };
        if res < 0 {
            Err(Errno::last())
        } else {
            Ok(Self(res))
        }
    }

    /// Writes some amount of the provided buffer to the file descriptor.
    ///
    /// # Notes
    ///
    /// It's perfectly possible (and expected) for this function to write less bytes than
    /// the length of the provided buffer. For this reason, the number of bytes actually
    /// written should be checked after calling this function.
    ///
    /// # Returns
    ///
    /// The number of bytes written to the file descriptor.
    #[inline]
    pub fn write(self, data: &[u8]) -> Result<usize> {
        let res = unsafe { libc::write(self.0, data.as_ptr() as *const c_void, data.len()) };
        if res < 0 {
            Err(Errno::last())
        } else {
            Ok(res as usize)
        }
    }

    /// Like [`write`](Self::write), but async.
    #[inline]
    #[cfg(feature = "futures")]
    #[doc(alias = "write")]
    pub fn async_write(self, data: &[u8]) -> futures::Write {
        futures::Write { fd: self, data }
    }

    /// Writes the entire contents of the provided buffer to the file descriptor.
    ///
    /// This function simply calls [`write`](Fd::write) in a loop until the entire buffer
    /// has been written or an error occurs.
    #[doc(alias = "write")]
    pub fn write_all(self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            let n = self.write(buf)?;

            // SAFETY:
            //  We know that `write` wrote `n` bytes, so it's safe to advance the buffer
            //  by this many bytes.
            buf = unsafe { buf.get_unchecked(n..) };
        }

        Ok(())
    }

    /// Like [`write_all`](Self::write_all), but async.
    #[inline]
    #[cfg(feature = "futures")]
    #[doc(alias = "write")]
    pub fn async_write_all(self, data: &[u8]) -> futures::WriteAll {
        futures::WriteAll { fd: self, data }
    }

    /// Writes the provided arguments to the file descriptor.
    ///
    /// This is a convenience wrapper around [`write_all`](Fd::write_all) that takes a
    /// [`fmt::Arguments`] instance instead of a byte slice.
    #[doc(alias = "write")]
    pub fn write_fmt(self, arguments: fmt::Arguments) -> Result<()> {
        /// An adapter that implements [`fmt::Write`] but makes sure to
        /// keep track of I/O errors instead of discarding them.
        struct Adapter {
            /// The file descriptor to write to.
            fd: Fd,
            /// An error that might have occurred while writing.
            err: Errno,
        }

        impl fmt::Write for Adapter {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                match self.fd.write_all(s.as_bytes()) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.err = e;
                        Err(fmt::Error)
                    }
                }
            }
        }

        let mut output = Adapter {
            fd: self,
            err: Errno::SUCCESS,
        };

        match fmt::write(&mut output, arguments) {
            Ok(()) => Ok(()),

            // NOTE:
            //  It's possible to determine whether the error came from the file descriptor
            //  or from the formatting by checking the value of `output.err`. However,
            //  it's not clear what we should do with this information right now.`
            Err(_) => Err(output.err),
        }
    }

    /// Reads some amount of data from the file descriptor into the provided buffer.
    ///
    /// # Notes
    ///
    /// It's perfectly possible (and expected) for this function to read less bytes than
    /// the length of the provided buffer. For this reason, the number of bytes actually
    /// read should be checked after calling this function.
    ///
    /// # Returns
    ///
    /// This function returns the number of bytes thtat were read into the input buffer. This
    /// number is always at most `buf`.
    ///
    /// If the function returns `0`, then the file descriptor has been exhausted and no more
    /// data is available for reading.
    ///
    /// Note that if the input bufferh as a size of zero, this function will always return `0`,
    /// even if more data is available for reading.
    #[inline]
    pub fn read(self, buf: &mut [MaybeUninit<u8>]) -> Result<usize> {
        let res = unsafe { libc::read(self.0, buf.as_mut_ptr() as *mut c_void, buf.len()) };
        if res < 0 {
            Err(Errno::last())
        } else {
            Ok(res as usize)
        }
    }

    /// Like [`read`](Self::read), but async.
    #[inline]
    #[cfg(feature = "futures")]
    #[doc(alias = "read")]
    pub fn async_read(self, buf: &mut [MaybeUninit<u8>]) -> futures::Read {
        futures::Read { fd: self, buf }
    }

    /// Reads a single byte from the file descriptor.
    ///
    /// If the file descriptor is exhausted, this function will return `None`.
    #[inline]
    #[doc(alias = "read")]
    pub fn read_one(self) -> Result<Option<u8>> {
        let mut buffer = MaybeUninit::uninit();

        match self.read(core::slice::from_mut(&mut buffer)) {
            Ok(0) => Ok(None),
            Ok(1) => Ok(Some(unsafe { buffer.assume_init() })),
            Ok(_) => unsafe { core::hint::unreachable_unchecked() },
            Err(e) => Err(e),
        }
    }

    /// Like [`read_one`](Self::read_one), but async.
    #[doc(alias = "read")]
    #[inline]
    #[cfg(feature = "futures")]
    pub fn async_read_one(self) -> futures::ReadOne {
        futures::ReadOne(self)
    }

    /// Performs a single read in the provided buffer's spare capacity.
    ///
    /// If the buffer has no more spare capacity, this function will
    /// *not* attempt to allocate more memory.
    ///
    /// The number of bytes read is returned.
    #[cfg(feature = "alloc")]
    #[inline]
    #[doc(alias = "read")]
    pub fn read_once_to_vec(self, vec: &mut alloc::vec::Vec<u8>) -> Result<usize> {
        let spare_cap = vec.spare_capacity_mut();
        match self.read(spare_cap) {
            Ok(count) => {
                unsafe { vec.set_len(vec.len() + count) };
                Ok(count)
            }
            Err(e) => Err(e),
        }
    }

    /// Like [`read_once_to_vec`](Self::read_once_to_vec), but async.
    #[cfg(all(feature = "futures", feature = "alloc"))]
    #[doc(alias = "read")]
    #[inline]
    pub fn async_read_once_to_vec(self, buf: &mut alloc::vec::Vec<u8>) -> futures::ReadOnceToVec {
        futures::ReadOnceToVec { fd: self, buf }
    }

    /// Reads the contents of the whole file until end-of-file or until an error occurs.
    #[cfg(feature = "alloc")]
    #[doc(alias = "read")]
    pub fn read_to_vec(self, vec: &mut alloc::vec::Vec<u8>) -> Result<()> {
        let mut batch_size = 64;

        // Read the whole file into a vector.
        loop {
            if vec.spare_capacity_mut().len() < batch_size && vec.try_reserve(batch_size).is_err() {
                break Err(Errno::NOMEM);
            }

            match self.read_once_to_vec(vec) {
                Ok(0) => break Ok(()),
                Ok(_) => {}
                Err(e) => break Err(e),
            }

            batch_size = batch_size.saturating_mul(2);
        }
    }

    /// Like [`read_to_vec`](Self::read_to_vec), but async.
    #[cfg(feature = "futures")]
    #[doc(alias = "read")]
    pub fn async_read_to_vec(self, vec: &mut alloc::vec::Vec<u8>) -> futures::ReadToVec {
        futures::ReadToVec::new(self, vec)
    }

    /// Attempts to perform a write on this file descriptor.
    ///
    /// If the operation would block, this function returns `Pending` and schedules the current
    /// task to be woken up when the file descriptor is ready for writing.
    #[cfg(feature = "rt-single-thread")]
    pub fn poll_write(
        self,
        data: &[u8],
        cx: &mut core::task::Context,
    ) -> core::task::Poll<Result<usize>> {
        match self.write(data) {
            Ok(count) => core::task::Poll::Ready(Ok(count)),
            Err(Errno::WOULDBLOCK) => {
                match crate::runtime::wake_me_up_on_io(
                    crate::fd::poll::PollFd::new(self, crate::fd::poll::PollFlags::OUT),
                    cx.waker().clone(),
                ) {
                    Ok(()) => core::task::Poll::Pending,
                    Err(err) => core::task::Poll::Ready(Err(err.into())),
                }
            }
            Err(err) => core::task::Poll::Ready(Err(err)),
        }
    }

    /// Attempts to perform a read on this file descriptor.
    ///
    /// If the operation would block, this function returns `Pending` and schedules the current
    /// task to be woken up when the file descriptor is ready for reading.
    #[cfg(feature = "rt-single-thread")]
    pub fn poll_read(
        self,
        data: &mut [MaybeUninit<u8>],
        cx: &mut core::task::Context,
    ) -> core::task::Poll<Result<usize>> {
        match self.read(data) {
            Ok(count) => core::task::Poll::Ready(Ok(count)),
            Err(Errno::WOULDBLOCK) => {
                match crate::runtime::wake_me_up_on_io(
                    crate::fd::poll::PollFd::new(self, crate::fd::poll::PollFlags::IN),
                    cx.waker().clone(),
                ) {
                    Ok(()) => core::task::Poll::Pending,
                    Err(err) => core::task::Poll::Ready(Err(err.into())),
                }
            }
            Err(err) => core::task::Poll::Ready(Err(err)),
        }
    }
}

impl File {
    /// Opens a file for reading.
    #[inline]
    pub fn open(path: &CharStar) -> Result<Self> {
        Fd::open(path, OpenFlags::READ_ONLY).map(Self)
    }

    /// Creates a new file for writing, truncating it if it already exists.
    #[inline]
    #[doc(alias = "open")]
    pub fn create(path: &CharStar) -> Result<Self> {
        Fd::open_with_mode(
            path,
            OpenFlags::WRITE_ONLY | OpenFlags::CREATE | OpenFlags::TRUNCATE,
            Mode::OWNER_READ | Mode::OWNER_WRITE,
        )
        .map(Self)
    }
}

/// Prints the provided message to the standard output stream, ignoring any errors that might
/// occur.
#[macro_export]
macro_rules! printf {
    ($($a:tt)*) => {{
        let _ = $crate::Fd::STDOUT.write_fmt(format_args!($($a)*));
    }};
}

/// Prints the provided message to the standard error stream, ignoring any errors that might
/// occur.
#[macro_export]
macro_rules! eprintf {
    ($($a:tt)*) => {{
        let _ = $crate::Fd::STDERR.write_fmt(format_args!($($a)*));
    }};
}
