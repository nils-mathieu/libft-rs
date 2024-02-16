//! Defines functions from the C standard library to avoid using them directly and
//! depending on some potentially forbidden functions.

mod string;

pub use self::string::*;
