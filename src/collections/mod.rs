//! Collections.

use core::fmt;

pub mod read_buffer;
pub use self::read_buffer::ReadBuffer;

/// An error that occurs when an allocation fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutOfMemory;

impl fmt::Display for OutOfMemory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("out of memory")
    }
}

/// The result type for memory-backed collections.
pub type Result<T> = ::core::result::Result<T, OutOfMemory>;
