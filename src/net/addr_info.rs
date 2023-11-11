//! Provides the [`AddrInfo`] type.

use core::ffi::c_int;

use bitflags::bitflags;

use crate::{CharStar, Errno, Result};

use super::{SocketAddr, SocketAddrFamily, SocketType};

bitflags! {
    /// Some flags associated with an address.
    pub struct AddrFlags: c_int {
        /// The address is suitable for binding to, listening passively on, and waiting
        /// for incoming connections.
        const PASSIVE = libc::AI_PASSIVE;
    }
}

/// Stores information about an address.
pub struct AddrInfo(libc::addrinfo);

impl AddrInfo {
    /// Looks up information about a specific address, eventually filtering by address family,
    /// flags, socket type.
    ///
    /// # Arguments
    ///
    /// - `hostname`: The hostname to look up. If `None`, the address will be unspecified.
    ///
    /// - `service`: The service to look up. If `None`, the service will be unspecified.
    ///
    /// - `addr_family`: The address family to look up. If `None`, the address family will be
    ///   unspecified.
    ///
    /// - `socket_type`: The socket type to look up. If `None`, the socket type will be unspecified.
    ///
    /// - `flags`: The flags to use when looking up the address.
    ///
    /// # Returns
    ///
    /// An iterator over matching addresses.
    pub fn lookup(
        hostname: Option<&CharStar>,
        service: Option<&CharStar>,
        addr_family: Option<SocketAddrFamily>,
        socket_type: Option<SocketType>,
        flags: AddrFlags,
    ) -> Result<LookupAddrInfo> {
        let mut hints = unsafe { core::mem::zeroed::<libc::addrinfo>() };
        hints.ai_family = addr_family.map_or(libc::AF_UNSPEC, SocketAddrFamily::to_raw);
        hints.ai_socktype = socket_type.map_or(0, SocketType::to_raw);
        hints.ai_flags = flags.bits();

        let hostname = hostname.map_or(core::ptr::null(), CharStar::as_ptr);
        let service = service.map_or(core::ptr::null(), CharStar::as_ptr);

        let mut res = core::ptr::null_mut();
        let ret = unsafe { libc::getaddrinfo(hostname, service, &hints, &mut res) };

        if ret == 0 {
            Ok(LookupAddrInfo {
                head: res,
                cur: res,
            })
        } else {
            Err(Errno::from_raw(ret))
        }
    }

    /// Returns the [`SocketAddr`] associated with this [`AddrInfo`].
    pub fn to_addr(&self) -> SocketAddr {
        match self.0.ai_family {
            libc::AF_INET => {
                let addr = unsafe { *(self.0.ai_addr as *const libc::sockaddr_in) };
                SocketAddr::V4(addr.sin_addr.s_addr.to_be_bytes(), addr.sin_port.to_be())
            }
            libc::AF_INET6 => {
                let addr = unsafe { *(self.0.ai_addr as *const libc::sockaddr_in6) };
                SocketAddr::V6(addr.sin6_addr.s6_addr, addr.sin6_port.to_be())
            }
            _ => unreachable!(),
        }
    }
}

/// An iterator over the [`AddrInfo`]s instances returned by [`AddrInfo::lookup`].
pub struct LookupAddrInfo {
    head: *mut libc::addrinfo,
    cur: *mut libc::addrinfo,
}

impl Iterator for LookupAddrInfo {
    type Item = AddrInfo;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur.is_null() {
            return None;
        }

        let cur = unsafe { *self.cur };
        self.cur = cur.ai_next;

        Some(AddrInfo(cur))
    }
}

impl Drop for LookupAddrInfo {
    #[inline]
    fn drop(&mut self) {
        unsafe { libc::freeaddrinfo(self.head) };
    }
}
