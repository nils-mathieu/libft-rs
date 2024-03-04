use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::fd::{PollFd, PollFlags};
use crate::net::SocketAddr;
use crate::{Errno, Fd, File, Result};

/// A [`Future`] that waits until a connection is available to accept.
///
/// The inner file descriptor is expected to be a socket opened in non-blocking mode.
#[derive(Debug, Clone)]
pub struct Accept(pub Fd);

impl Future for Accept {
    type Output = Result<(File, SocketAddr)>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0.accept() {
            Ok(ret) => Poll::Ready(Ok(ret)),
            Err(Errno::WOULDBLOCK) => {
                match crate::runtime::wake_me_up_on_io(
                    PollFd::new(self.0, PollFlags::IN),
                    cx.waker().clone(),
                ) {
                    Ok(()) => Poll::Pending,
                    Err(err) => Poll::Ready(Err(err.into())),
                }
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
