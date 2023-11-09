//! Provides way to safely declare an entry point for the program.

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
/// The return value of the entry point function is the exit code of the program.
///
/// # Examples
///
/// ```
/// # use ft::entry_point;
/// # use ft::CharStar;
/// #
/// entry_point!(main);
///
/// fn main(argv: &'static [CharStar<'static>], envp: &'static [CharStar<'static>]) -> u8 {
///     // ...
///     0
/// }
/// ```
#[macro_export]
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
