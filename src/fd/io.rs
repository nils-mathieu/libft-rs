use core::ffi::c_void;
use core::fmt;
use core::mem::MaybeUninit;

use super::OpenFlags;
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
    pub fn write(self, buf: &[u8]) -> Result<usize> {
        let res = unsafe { libc::write(self.0, buf.as_ptr() as *const c_void, buf.len()) };
        if res < 0 {
            Err(Errno::last())
        } else {
            Ok(res as usize)
        }
    }

    /// Writes the entire contents of the provided buffer to the file descriptor.
    ///
    /// This function simply calls [`write`](Fd::write) in a loop until the entire buffer
    /// has been written or an error occurs.
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

    /// Writes the provided arguments to the file descriptor.
    ///
    /// This is a convenience wrapper around [`write_all`](Fd::write_all) that takes a
    /// [`fmt::Arguments`] instance instead of a byte slice.
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
}

impl File {
    /// Opens a file for reading.
    #[inline]
    pub fn open(path: &CharStar) -> Result<Self> {
        Fd::open(path, OpenFlags::READ_ONLY).map(Self)
    }

    /// Creates a new file for writing, truncating it if it already exists.
    #[inline]
    pub fn create(path: &CharStar) -> Result<Self> {
        Fd::open(
            path,
            OpenFlags::WRITE_ONLY | OpenFlags::CREATE | OpenFlags::TRUNCATE,
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
