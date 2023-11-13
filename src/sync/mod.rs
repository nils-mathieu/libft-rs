//! Syncronization primitives.

pub mod mutex;

mod once_cell;
pub use self::once_cell::*;
