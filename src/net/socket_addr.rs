//! Defines the [`SocketAddr`] type.

use core::ffi::c_int;

use super::{AddrFlags, AddrInfo, SocketType};
use crate::{CharStar, Result};

/// A socket address that can be used when connecting to a server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SocketAddr {
    /// An IP-v4 socket address.
    V4([u8; 4], u16),
    /// An IP-v6 socket address.
    V6([u8; 16], u16),
}

impl SocketAddr {
    /// Returns an iterator over the addresses associated with the given hostname.
    pub fn lookup_host(hostname: &CharStar) -> Result<impl Iterator<Item = Self>> {
        AddrInfo::lookup(
            Some(hostname),
            None,
            None,
            Some(SocketType::Stream),
            AddrFlags::empty(),
        )
        .map(|lookup| lookup.map(|addr| addr.to_addr()))
    }

    /// Returns the port of the socket address.
    pub fn port(&self) -> u16 {
        match self {
            Self::V4(_, port) => *port,
            Self::V6(_, port) => *port,
        }
    }

    /// Returns the family of the socket address.
    pub fn family(&self) -> SocketAddrFamily {
        match self {
            Self::V4(..) => SocketAddrFamily::V4,
            Self::V6(..) => SocketAddrFamily::V6,
        }
    }

    /// Creates a new [`SocketAddr`] instance from the provided raw value.
    ///
    /// # Panics
    ///
    /// This function panics if the provided raw value is not a valid socket address.
    pub(crate) fn from_raw(addr: *const libc::sockaddr) -> Self {
        match unsafe { (*addr).sa_family as c_int } {
            libc::AF_INET => {
                let addr = unsafe { *(addr as *const libc::sockaddr_in) };
                Self::V4(addr.sin_addr.s_addr.to_be_bytes(), addr.sin_port.to_be())
            }
            libc::AF_INET6 => {
                let addr = unsafe { *(addr as *const libc::sockaddr_in6) };
                Self::V6(addr.sin6_addr.s6_addr, addr.sin6_port.to_be())
            }
            _ => unreachable!("invalid socket address"),
        }
    }

    /// Returns the raw value of the socket address.
    pub(crate) fn write_raw(&self, storage: &mut libc::sockaddr_storage) -> *mut libc::sockaddr {
        match self {
            Self::V4(ip, port) => {
                let ret = unsafe { &mut *(storage as *mut _ as *mut libc::sockaddr_in) };
                ret.sin_family = libc::AF_INET as _;
                ret.sin_len = core::mem::size_of::<libc::sockaddr_in>() as _;
                ret.sin_addr.s_addr = u32::from_be_bytes(*ip);
                ret.sin_port = port.to_be();
            }
            Self::V6(ip, port) => {
                let ret = unsafe { &mut *(storage as *mut _ as *mut libc::sockaddr_in6) };
                ret.sin6_family = libc::AF_INET6 as _;
                ret.sin6_len = core::mem::size_of::<libc::sockaddr_in6>() as _;
                ret.sin6_addr.s6_addr = *ip;
                ret.sin6_port = port.to_be();
            }
        }

        storage as *mut _ as *mut _
    }
}

/// The family of a [`SocketAddr`].
pub enum SocketAddrFamily {
    /// An IP-v4 socket address.
    V4 = libc::AF_INET as _,
    /// An IP-v6 socket address.
    V6 = libc::AF_INET6 as _,
}

impl SocketAddrFamily {
    /// Returns the raw value of the socket address family.
    pub fn to_raw(self) -> c_int {
        self as _
    }
}
