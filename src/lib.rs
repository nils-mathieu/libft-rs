//! My own standard library for 42 school projects.

#![no_std]
#![feature(lang_items)]
#![feature(extern_types)]
#![allow(internal_features)]

pub mod charstar;
pub use self::charstar::CharStar;

mod entry_point;

#[doc(hidden)]
pub mod __private;

#[cfg(feature = "panic-handler")]
mod panic;
