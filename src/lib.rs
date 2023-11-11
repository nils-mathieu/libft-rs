//! My own standard library for 42 school projects.

#![no_std]
#![allow(internal_features)]
#![feature(lang_items, extern_types, panic_info_message)]

pub mod charstar;
pub use self::charstar::CharStar;

pub mod fd;
pub use self::fd::{Fd, File};

pub mod net;

mod errno;
pub use self::errno::{Errno, Result};

pub mod mmap;

pub mod process;
pub use self::process::Signal;

pub mod sync;
pub use self::sync::mutex::Mutex;

mod time;
pub use self::time::*;

mod entry_point;

#[doc(hidden)]
pub mod __private;

#[cfg(feature = "panic-handler")]
mod panic_handler;
#[cfg(feature = "panic-handler")]
pub use self::panic_handler::set_panic_handler;

#[cfg(feature = "global-allocator")]
mod global_allocator;
