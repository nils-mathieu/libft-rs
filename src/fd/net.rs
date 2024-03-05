//! Networking with file descriptors.

use core::ffi::c_int;

#[cfg(feature = "futures")]
use crate::futures;
use crate::net::{SocketAddr, SocketAddrFamily, SocketType};
use crate::{Errno, Fd, File, Result};

impl Fd {
    /// Creates a communication endpoint.
    ///
    /// # Arguments
    ///
    /// - `domain` - The domain to which the socket will bound to.
    ///
    /// - `ty` - The type of the socket.
    ///
    /// # Returns
    ///
    /// The created file descriptor.
    #[inline]
    pub fn socket(domain: SocketAddrFamily, ty: SocketType) -> Result<Self> {
        let fd = unsafe { libc::socket(domain.to_raw(), ty.to_raw(), 0) };

        if fd < 0 {
            Err(Errno::last())
        } else {
            Ok(Self(fd))
        }
    }

    /// Marks the socket referenced by this file descriptor as a passive socket.
    ///
    /// Passive sockets are used to listen for incoming connections coming from
    /// the network.
    ///
    /// This the socket must be of type [`SocketType::Stream`].
    ///
    /// # Arguments
    ///
    /// The `backlog` argument specifies the maximum number of pending connections
    /// that can be queued up before the kernel starts rejecting them.
    ///
    /// # Returns
    ///
    /// Nothing, or an error if the operation fails.
    #[inline]
    pub fn listen(self, backlog: usize) -> Result<()> {
        let ret = unsafe { libc::listen(self.0, backlog as c_int) };
        if ret == 0 {
            Ok(())
        } else {
            Err(Errno::last())
        }
    }

    /// Binds the socket referenced by this file descriptor to the provided address. Incoming
    /// connections will be accepted only if they are addressed to this address.
    ///
    /// # Arguments
    ///
    /// The `addr` argument specifies the address to bind to.
    ///
    /// # Returns
    ///
    /// Nothing, or an error if the operation fails.
    pub fn bind(self, addr: &SocketAddr) -> Result<()> {
        let mut addr_storage: libc::sockaddr_storage = unsafe { core::mem::zeroed() };

        let ret = unsafe {
            libc::bind(
                self.0,
                addr.write_raw(&mut addr_storage),
                addr.family().len_of_sockaddr(),
            )
        };

        if ret == 0 {
            Ok(())
        } else {
            Err(Errno::last())
        }
    }

    /// Accepts an incoming connection on this file descriptor.
    ///
    /// If no connection is available, this function will block until one becomes available.
    ///
    /// # Returns
    ///
    /// This function returns two values:
    ///
    /// - The file descriptor of the accepted connection. Reading from this file descriptor
    ///   allows to read data sent by the client. Writing to the file descriptor allows
    ///   sending data to the client.
    ///
    /// - The address of the client that connected to the socket.
    pub fn accept(self) -> Result<(File, SocketAddr)> {
        let mut addr_storage: libc::sockaddr_storage = unsafe { core::mem::zeroed() };
        let mut addr_len = core::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;

        let ret = unsafe {
            libc::accept(
                self.0,
                &mut addr_storage as *mut libc::sockaddr_storage as *mut libc::sockaddr,
                &mut addr_len,
            )
        };

        if ret == -1 {
            Err(Errno::last())
        } else {
            Ok((
                File::from_raw(ret),
                SocketAddr::from_raw(&addr_storage as *const _ as *const _),
            ))
        }
    }

    /// Like [`accept`](Self::accept), but returns a future that resolves when a connection is
    /// available.
    #[cfg(feature = "futures")]
    #[doc(alias = "accept")]
    #[inline]
    pub fn async_accept(self) -> futures::Accept {
        futures::Accept(self)
    }

    /// Connects this socket to a specific address.
    ///
    /// After a successful call to this function, the socket will be able to send and receive
    /// data to and from the specified address.
    ///
    /// # Arguments
    ///
    /// The `addr` argument specifies the address to connect to.
    pub fn connect(self, addr: SocketAddr) -> Result<()> {
        let mut addr_storage: libc::sockaddr_storage = unsafe { core::mem::zeroed() };

        let ret = unsafe {
            libc::connect(
                self.0,
                addr.write_raw(&mut addr_storage),
                addr.family().len_of_sockaddr(),
            )
        };

        if ret == 0 {
            Ok(())
        } else {
            Err(Errno::last())
        }
    }
}

impl File {
    /// See [`Fd::socket`].
    #[inline]
    pub fn socket(family: SocketAddrFamily, ty: SocketType) -> Result<Self> {
        Fd::socket(family, ty).map(Self)
    }
}
