//! Provides an async executor.

mod task_waker;
mod tasks;
mod waker;

#[cfg(feature = "rt-single-thread")]
mod single_thread;

#[cfg(feature = "rt-single-thread")]
pub use self::single_thread::*;

use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

/// A task function.
pub type DynTask<'a> = Pin<Box<dyn 'a + Future<Output = ()>>>;
