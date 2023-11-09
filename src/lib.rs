//! My own standard library for 42 school projects.

#![no_std]
#![feature(lang_items)]
#![feature(extern_types)]
#![allow(internal_features)]

pub mod charstar;
pub use self::charstar::CharStar;

pub mod fd;
pub use self::fd::{Fd, File};

pub mod net;

mod errno;
pub use self::errno::{Errno, Result};

pub mod mmap;
pub mod process;

pub mod sync;
pub use self::sync::mutex::Mutex;

mod entry_point;

#[doc(hidden)]
pub mod __private;

#[cfg(feature = "panic-handler")]
mod panic;
#[cfg(feature = "panic-handler")]
pub use self::panic::set_panic_handler;
