//! Provides global constructor and destructor functions.

/// Declares a function that will be called before `main`.
#[macro_export]
macro_rules! ctor {
    ($f:ident) => {
        const _: () = {
            #[used]
            #[cfg_attr(target_os = "linux", link_section = ".init_array")]
            #[cfg_attr(target_os = "android", link_section = ".init_array")]
            #[cfg_attr(target_os = "freebsd", link_section = ".init_array")]
            #[cfg_attr(target_os = "netbsd", link_section = ".init_array")]
            #[cfg_attr(target_os = "openbsd", link_section = ".init_array")]
            #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
            #[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
            static STORAGE: extern "C" fn() = $f;
        };
    };
}
