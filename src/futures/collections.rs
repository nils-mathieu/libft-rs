use core::future::Future;
use core::pin::Pin;
use core::task::{ready, Context, Poll};

use crate::collections::ReadBuffer;
use crate::{Fd, MemchrExt, Result};

/// A [`Future`] that completes when some data can be written to a [`ReadBuffer`].
#[doc(alias = "fill_with_fd")]
pub struct FillWithFd<'a> {
    /// The file descriptor that will be read into the buffer.
    ///
    /// It is expected to be in non-blocking mode.
    pub fd: Fd,
    /// The buffer that will be filled with data.
    pub buf: &'a mut ReadBuffer,
}

impl<'a> Future for FillWithFd<'a> {
    type Output = Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        match ready!(this.fd.poll_read(this.buf.spare_capacity_mut(), cx)) {
            Ok(count) => {
                unsafe { this.buf.assume_init(count) };
                Poll::Ready(Ok(count))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

/// A future that reads a file descriptor until a delimiter is found.
///
/// See [`ReadBuffer::read_until`] for more information.
pub struct ReadUntil<'a, 'd> {
    fd: Fd,
    buf: Option<&'a mut ReadBuffer>,
    delimiter: &'d [u8],
    batch_size: usize,
    checked: usize,
}

impl<'a, 'd> ReadUntil<'a, 'd> {
    /// Creates a new `ReadUntil` future.
    pub fn new(fd: Fd, buf: &'a mut ReadBuffer, delimiter: &'d [u8]) -> Self {
        Self {
            fd,
            buf: Some(buf),
            delimiter,
            batch_size: 64,
            checked: 0,
        }
    }
}

impl<'a, 'd> Future for ReadUntil<'a, 'd> {
    type Output = Result<&'a mut [u8]>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.delimiter.is_empty() {
            return Poll::Ready(Ok(&mut []));
        }

        assert!(self.buf.is_some(), "polling after completion");

        let this = self.get_mut();
        let buf = unsafe { this.buf.as_mut().unwrap_unchecked() };

        loop {
            unsafe {
                if let Some(mut index) = buf
                    .pending()
                    .get_unchecked(this.checked..)
                    .memchr(*this.delimiter.get_unchecked(0))
                {
                    index += this.checked;

                    if buf
                        .pending()
                        .get_unchecked(index..)
                        .starts_with(this.delimiter)
                    {
                        return Poll::Ready(Ok(this
                            .buf
                            .take()
                            .unwrap_unchecked()
                            .consume_unchecked(index + this.delimiter.len())));
                    }
                }
            }

            this.checked = buf.pending().len();

            buf.reserve(this.batch_size)?;
            let count = ready!(this.fd.poll_read(buf.spare_capacity_mut(), cx))?;
            unsafe { buf.assume_init(count) };

            this.batch_size = this.batch_size.saturating_mul(2);
        }
    }
}

/// A future that reads a file descriptor until a delimiter is found.
///
/// See [`ReadBuffer::read_until`] for more information.
pub struct ReadExact<'a> {
    fd: Fd,
    buf: Option<&'a mut ReadBuffer>,
    count: usize,
}

impl<'a> ReadExact<'a> {
    /// Creates a new `ReadExact` future.
    pub fn new(fd: Fd, buf: &'a mut ReadBuffer, count: usize) -> Self {
        Self {
            fd,
            buf: Some(buf),
            count,
        }
    }
}

impl<'a> Future for ReadExact<'a> {
    type Output = Result<&'a mut [u8]>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        assert!(self.buf.is_some(), "polling after completion");

        let this = self.get_mut();
        let buf = unsafe { this.buf.as_mut().unwrap_unchecked() };

        loop {
            if buf.pending().len() > this.count {
                return Poll::Ready(Ok(unsafe {
                    this.buf
                        .take()
                        .unwrap_unchecked()
                        .consume_unchecked(this.count)
                }));
            }

            buf.reserve(this.count.saturating_sub(buf.pending().len()))?;
            let count = ready!(this.fd.poll_read(buf.spare_capacity_mut(), cx))?;
            unsafe { buf.assume_init(count) };
        }
    }
}
