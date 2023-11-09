//! Defines the [`Errno`] type and its associated constants.

use core::ffi::c_int;

/// The result type for functions that return a canonical [`Errno`] value.
pub type Result<T> = ::core::result::Result<T, Errno>;

/// The type of the `errno` global variable.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Errno(c_int);

impl Errno {
    /// Returns the value of the `errno` global variable on the current thread.
    pub fn last() -> Self {
        Self(unsafe { *libc::__error() })
    }

    /// Sets the value of the `errno` global variable on the current thread.
    pub fn make_last(self) {
        unsafe { *libc::__error() = self.0 };
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
    };
}

define_Errno_constants! {
    /// Indicates that no error occured.
    pub const SUCCESS = 0;
}
