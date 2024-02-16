//! My own standard library for 42 school projects.

#![no_std]
#![allow(internal_features)]
#![forbid(unsafe_op_in_unsafe_fn)]
#![feature(extern_types, panic_info_message)]
#![feature(slice_ptr_get)]
#![feature(lang_items)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod ctor;
mod entry_point;
mod errno;
#[cfg(feature = "restrict-functions")]
mod fake_libc;
#[cfg(feature = "global-allocator")]
mod global_allocator;
mod misc;
#[cfg(feature = "panic-handler")]
mod panic_handler;
mod time;

#[cfg(feature = "alloc")]
pub mod alloc_ext;
pub mod charstar;
#[cfg(feature = "collections")]
pub mod collections;
pub mod fd;
pub mod malloc;
pub mod mmap;
pub mod net;
pub mod process;
pub mod sync;

pub use self::charstar::CharStar;
pub use self::errno::{Errno, Result};
pub use self::fd::{Fd, File};
pub use self::misc::*;
#[cfg(feature = "panic-handler")]
pub use self::panic_handler::set_panic_handler;
pub use self::process::Signal;
pub use self::sync::mutex::Mutex;
pub use self::sync::OnceCell;
pub use self::time::*;

#[doc(hidden)]
pub mod __private;

#[cfg(feature = "panic-eh-personality")]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
