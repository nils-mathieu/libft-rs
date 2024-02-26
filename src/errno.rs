//! Defines the [`Errno`] type and its associated constants.

use core::ffi::c_int;
use core::fmt;
use core::mem::MaybeUninit;

use crate::CharStar;

/// The result type for functions that return a canonical [`Errno`] value.
pub type Result<T> = ::core::result::Result<T, Errno>;

/// The type of the `errno` global variable.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "errno")]
pub struct Errno(c_int);

impl Errno {
    /// Returns the value of the `errno` global variable on the current thread.
    #[inline]
    #[cfg(not(feature = "restrict-errno"))]
    pub fn last() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self(unsafe { *libc::__error() })
        }

        #[cfg(target_os = "linux")]
        {
            Self(unsafe { *libc::__errno_location() })
        }
    }

    /// Returns the `SUCCESS` error code.
    ///
    /// This function is used as a fallback when the `restrict-errno` feature is
    /// enabled.
    #[inline]
    #[cfg(feature = "restrict-errno")]
    pub fn last() -> Self {
        Self::SUCCESS
    }

    /// Sets the value of the `errno` global variable on the current thread.
    #[inline]
    #[cfg(not(feature = "restrict-errno"))]
    pub fn make_last(self) {
        #[cfg(target_os = "macos")]
        unsafe {
            *libc::__error() = self.0
        }

        #[cfg(target_os = "linux")]
        unsafe {
            *libc::__errno_location() = self.0
        }
    }

    /// Does nothing.
    ///
    /// This function is used as a fallback when the `restrict-errno` feature is
    /// enabled.
    #[inline]
    #[cfg(feature = "restrict-errno")]
    pub fn make_last(self) {}

    /// Creates a new [`Errno`] instance from the provided raw value.
    #[inline]
    pub fn from_raw(raw: c_int) -> Self {
        Self(raw)
    }

    /// Returns the raw value of this [`Errno`] instance.
    #[inline]
    pub fn to_raw(self) -> c_int {
        self.0
    }

    /// Writes a description of this error to the provided buffer.
    pub fn write_description(self, buf: &mut [MaybeUninit<u8>]) -> Option<&CharStar> {
        let ret = unsafe { libc::strerror_r(self.0, buf.as_mut_ptr().cast(), buf.len()) };

        if ret == 0 {
            Some(unsafe { CharStar::from_ptr(buf.as_ptr().cast()) })
        } else {
            None
        }
    }
}

/// A helper to help define constants for [`Errno`].
macro_rules! define_Errno_constants {
    ($(
        $(#[$($attrs:meta)*])*
        pub const $name:ident = $value:expr;
    )*) => {
        impl Errno {
            $(
                $(#[$($attrs)*])*
                pub const $name: Self = Self($value);
            )*
        }

        impl fmt::Debug for Errno {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "\"{self}\" (")?;

                match *self {
                    $(
                        Self::$name => f.write_str(stringify!($name))?,
                    )*
                    _ => f.debug_tuple("Errno").field(&self.0).finish()?,
                }

                write!(f, ")")
            }
        }
    };
}

impl fmt::Display for Errno {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf: [MaybeUninit<u8>; 32] = uninit_array();

        let desc = self
            .write_description(&mut buf)
            .and_then(|d| d.as_str())
            .unwrap_or("Unknown error");

        f.pad(desc)
    }
}

/// A helper to create an array of [`MaybeUninit`] values.
fn uninit_array<T, const N: usize>() -> [MaybeUninit<T>; N] {
    unsafe { MaybeUninit::uninit().assume_init() }
}

define_Errno_constants! {
    /// Indicates that no error occured.
    pub const SUCCESS = 0;
    /// Indicates that an invalid argument was provided.
    pub const INVAL = libc::EINVAL;
    /// Indicates that the provided buffer was too small.
    pub const WOULDBLOCK = libc::EWOULDBLOCK;
    /// Indicates that the operation was interrupted by a signal.
    pub const INTR = libc::EINTR;
    /// The system is out of memory.
    pub const NOMEM = libc::ENOMEM;
    /// The connection was reset by the peer.
    pub const CONNRESET = libc::ECONNRESET;
    /// The connection was aborted by the peer.
    pub const NOEXEC = libc::ENOEXEC;
}
