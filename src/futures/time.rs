use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::Instant;

/// A [`Future`] that resolves once a certain point in time is reached.
#[derive(Debug, Clone)]
pub struct Sleep {
    instant: Instant,
    once: bool,
}

impl Sleep {
    /// Creates a new [`Sleep`] future.
    #[inline]
    pub const fn new(at: Instant) -> Self {
        Self {
            instant: at,
            once: false,
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.once {
            Poll::Ready(())
        } else {
            crate::runtime::wake_me_up_on_time(self.instant, cx.waker().clone()).unwrap(); // out of memory
            Poll::Pending
        }
    }
}
