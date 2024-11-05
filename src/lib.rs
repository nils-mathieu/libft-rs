//! My own standard library for 42 school projects.

#![no_std]
#![allow(internal_features)]
#![forbid(unsafe_op_in_unsafe_fn)]
#![feature(extern_types)]
#![feature(slice_ptr_get)]
#![feature(lang_items)]
#![feature(thread_local)]
#![feature(allocator_api)]
#![feature(never_type)]
#![feature(result_flattening)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod alloc_ext;
mod ctor;
mod errno;
#[cfg(feature = "restrict-functions")]
mod fake_libc;
#[cfg(feature = "global-allocator")]
mod global_allocator;
mod memchr_ext;
mod misc;
#[cfg(feature = "panic-handler")]
mod panic_handler;
mod time;
mod user;

pub mod ansi;
pub mod charstar;
#[cfg(feature = "collections")]
pub mod collections;
pub mod dylib;
pub mod entry_point;
pub mod fd;
#[cfg(feature = "futures")]
pub mod futures;
pub mod malloc;
pub mod mmap;
pub mod net;
pub mod process;
#[cfg(feature = "readline")]
pub mod readline;
#[cfg(feature = "rt-single-thread")]
pub mod runtime;
pub mod sync;
pub mod termios;
pub mod utils;

#[cfg(feature = "alloc")]
pub use self::alloc_ext::*;
pub use self::charstar::CharStar;
pub use self::errno::{Errno, Result};
pub use self::fd::{Fd, File};
pub use self::memchr_ext::*;
pub use self::misc::*;
#[cfg(feature = "panic-handler")]
pub use self::panic_handler::set_panic_handler;
pub use self::process::{Pid, Signal};
pub use self::sync::mutex::Mutex;
pub use self::time::*;
pub use self::user::*;

#[doc(hidden)]
pub mod __private;

#[cfg(feature = "panic-eh-personality")]
#[lang = "eh_personality"]
extern "C" fn eh_personality() -> ! {
    unreachable!();
}

#[cfg(feature = "panic-eh-personality")]
#[no_mangle]
extern "C-unwind" fn _Unwind_Resume() -> ! {
    unreachable!();
}
