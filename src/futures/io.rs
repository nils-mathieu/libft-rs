use core::future::Future;
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::task::{ready, Context, Poll};

use crate::fd::{PollFd, PollFlags};
use crate::{Errno, Fd, Result};

fn poll_write(fd: Fd, data: &[u8], cx: &mut Context) -> Poll<Result<usize>> {
    match fd.write(data) {
        Ok(count) => Poll::Ready(Ok(count)),
        Err(Errno::WOULDBLOCK) => {
            match crate::runtime::wake_me_up_on_io(
                PollFd::new(fd, PollFlags::OUT),
                cx.waker().clone(),
            ) {
                Ok(()) => Poll::Pending,
                Err(err) => Poll::Ready(Err(err.into())),
            }
        }
        Err(err) => Poll::Ready(Err(err)),
    }
}

fn poll_read(fd: Fd, data: &mut [MaybeUninit<u8>], cx: &mut Context) -> Poll<Result<usize>> {
    match fd.read(data) {
        Ok(count) => Poll::Ready(Ok(count)),
        Err(Errno::WOULDBLOCK) => {
            match crate::runtime::wake_me_up_on_io(
                PollFd::new(fd, PollFlags::IN),
                cx.waker().clone(),
            ) {
                Ok(()) => Poll::Pending,
                Err(err) => Poll::Ready(Err(err.into())),
            }
        }
        Err(err) => Poll::Ready(Err(err)),
    }
}

/// A future that completes when some data can be written to a file descriptor.
///
/// # Notes
///
/// This future assumes that the file descriptor is in non-blocking mode.
#[derive(Debug, Clone)]
#[doc(alias = "write")]
pub struct Write<'a> {
    /// The file descriptor that will be written.
    ///
    /// It is expected to be in non-blocking mode.
    pub fd: Fd,
    /// The data to write.
    pub data: &'a [u8],
}

impl<'a> Future for Write<'a> {
    type Output = Result<usize>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        poll_write(self.fd, self.data, cx)
    }
}

/// A future that completes once all the data it references has been written to a file descriptor.
///
/// # Notes
///
/// This future assumes that the file descriptor is in non-blocking mode.
#[doc(alias = "write")]
#[derive(Debug, Clone)]
pub struct WriteAll<'a> {
    /// The file descriptor that will be written.
    ///
    /// This file descriptor is expected to be in non-blocking mode.
    pub fd: Fd,
    /// The data that must still be written.
    pub data: &'a [u8],
}

impl<'a> Future for WriteAll<'a> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        while !self.data.is_empty() {
            match ready!(poll_write(self.fd, self.data, cx)) {
                Ok(count) => self.data = unsafe { self.data.get_unchecked(count..) },
                Err(err) => return Poll::Ready(Err(err)),
            }
        }

        Poll::Ready(Ok(()))
    }
}

/// Reads data from a file descriptor.
///
/// # Notes
///
/// This future assumes that the file descriptor is in non-blocking mode.
#[doc(alias = "write")]
#[derive(Debug)]
pub struct Read<'a> {
    /// The file descriptor that will be read.
    ///
    /// It is expected to be in non-blocking mode.
    pub fd: Fd,
    /// The buffer to read the data into.
    pub buf: &'a mut [MaybeUninit<u8>],
}

impl<'a> Future for Read<'a> {
    type Output = Result<usize>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        poll_read(self.fd, self.buf, cx)
    }
}
