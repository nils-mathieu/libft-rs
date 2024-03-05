use core::future::Future;
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::task::{ready, Context, Poll};

use crate::fd::{PollFd, PollFlags};
use crate::{Errno, Fd, Result};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

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
        self.fd.poll_write(self.data, cx)
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
            match ready!(self.fd.poll_write(self.data, cx)) {
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
#[doc(alias = "read")]
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
        self.fd.poll_read(self.buf, cx)
    }
}

/// A future that completes once exactly one byte has been read from a file descriptor.
///
/// # Notes
///
/// This future assumes that the file descriptor is in non-blocking mode.
#[doc(alias = "read")]
#[derive(Debug, Clone)]
pub struct ReadOne(pub Fd);

impl Future for ReadOne {
    type Output = Result<Option<u8>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut buf = MaybeUninit::uninit();

        match ready!(self.0.poll_read(core::slice::from_mut(&mut buf), cx)) {
            Ok(0) => Poll::Ready(Ok(None)),
            Ok(_) => Poll::Ready(Ok(Some(unsafe { buf.assume_init() }))),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

/// A future that reads data from a file descriptor into a vector. A single read is performed,
/// without attempting to reallocate the vector.
///
/// # Notes
///
/// This future assumes that the file descriptor is in non-blocking mode.
#[cfg(feature = "alloc")]
#[doc(alias = "read")]
#[derive(Debug)]
pub struct ReadOnceToVec<'a> {
    /// The file descriptor that will be read.
    ///
    /// It is expected to be in non-blocking mode.
    pub fd: Fd,
    /// The buffer to read the data into.
    pub buf: &'a mut Vec<u8>,
}

#[cfg(feature = "alloc")]
impl<'a> Future for ReadOnceToVec<'a> {
    type Output = Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.fd.poll_read(self.buf.spare_capacity_mut(), cx)) {
            Ok(count) => {
                let len = self.buf.len();
                unsafe { self.buf.set_len(len + count) };
                Poll::Ready(Ok(count))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

/// A future that reads data from a file descriptor into a vector. The vector is reallocated as
/// necessary to accommodate the data.
///
/// # Notes
///
/// This future assumes that the file descriptor is in non-blocking mode.
#[cfg(feature = "alloc")]
#[doc(alias = "read")]
#[derive(Debug)]
pub struct ReadToVec<'a> {
    fd: Fd,
    buf: &'a mut Vec<u8>,
    batch_size: usize,
}

impl<'a> ReadToVec<'a> {
    /// Creates a new `ReadToVec` future.
    pub fn new(fd: Fd, buf: &'a mut Vec<u8>) -> Self {
        Self {
            fd,
            buf,
            batch_size: 64,
        }
    }
}

#[cfg(feature = "alloc")]
impl Future for ReadToVec<'_> {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        loop {
            match this.buf.try_reserve(this.batch_size) {
                Ok(()) => {}
                Err(_) => break Poll::Ready(Err(Errno::NOMEM)),
            }

            let read_size = this.buf.spare_capacity_mut().len();

            match ready!(this.fd.poll_read(this.buf.spare_capacity_mut(), cx)) {
                Ok(0) => break Poll::Ready(Ok(())),
                Ok(count) => {
                    let len = this.buf.len();
                    unsafe { this.buf.set_len(len + count) }

                    if count < read_size {
                        match crate::runtime::wake_me_up_on_io(
                            PollFd::new(this.fd, PollFlags::IN),
                            cx.waker().clone(),
                        ) {
                            Ok(_) => break Poll::Pending,
                            Err(err) => break Poll::Ready(Err(err.into())),
                        }
                    }
                }
                Err(err) => break Poll::Ready(Err(err)),
            }
        }
    }
}
