//! Provides way to safely declare an entry point for the program.

use core::ffi::c_int;

/// Types that can be used as the output of the `main` function.
pub trait Terminate {
    /// Turns the value into the exit code of the program.
    fn terminate(self) -> c_int;
}

impl Terminate for c_int {
    #[inline]
    fn terminate(self) -> c_int {
        self
    }
}

impl Terminate for u8 {
    #[inline]
    fn terminate(self) -> c_int {
        self as c_int
    }
}

impl Terminate for () {
    #[inline]
    fn terminate(self) -> c_int {
        0
    }
}

impl<A, B> Terminate for Result<A, B>
where
    A: Terminate,
    B: Terminate,
{
    #[inline]
    fn terminate(self) -> c_int {
        match self {
            Ok(ok) => ok.terminate(),
            Err(err) => err.terminate(),
        }
    }
}

/// Declares an entry point for the program.
///
/// This macro adds a `main` function with the appropriate signature and link name to the
/// program. After having been called by the C runtime, this `main` function will call the
/// function passed as argument to the macro.
///
/// # Entry Points
///
/// Any function that takes two arguments of type `&[CharStar]` and returns `u8` can be used
/// as an entry point.
///
/// The first argument is the `argv` array, which contains the command-line arguments passed
/// to the program. The first element of this array is usually the name of the program itself.
///
/// The second argument is the `envp` array, which contains the environment variables passed
/// to the program.
///
/// The return value of the entry point function can be anything that implements the [`Terminate`]
/// trait.
///
/// # Examples
///
/// ```ignore
/// # use ft::entry_point;
/// # use ft::CharStar;
/// #
/// entry_point!(main);
///
/// fn main(argv: &[&CharStar], envp: &[&CharStar]) -> u8 {
///     // ...
///     0
/// }
/// ```
#[macro_export]
#[doc(alias = "main")]
macro_rules! entry_point {
    ($f:expr) => {
        const _: () = {
            #[export_name = "main"]
            pub extern "C" fn __libft_main(
                _argc: ::core::ffi::c_int,
                argv: *const *const ::core::ffi::c_char,
                envp: *const *const ::core::ffi::c_char,
            ) -> ::core::ffi::c_int {
                unsafe { $crate::__private::entry_point::call($f, argv, envp) }
            }
        };
    };
}
