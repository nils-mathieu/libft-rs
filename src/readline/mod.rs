//! Provides ways to read lines from the terminal while still being able to
//! handle special events such as CTRL+C or the arrow keys.

mod cmdline;
mod non_canonical;
#[allow(clippy::module_inception)]
mod readline;

pub use self::cmdline::*;
pub use self::non_canonical::*;
pub use self::readline::*;
