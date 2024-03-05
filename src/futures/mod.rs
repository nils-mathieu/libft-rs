#[cfg(not(feature = "rt-single-thread"))]
compile_error!("The feature `futures` cannot be used without a runtime.");

#[cfg(feature = "collections")]
mod collections;
mod io;
mod net;
mod time;

#[cfg(feature = "collections")]
pub use self::collections::*;
pub use self::io::*;
pub use self::net::*;
pub use self::time::*;
