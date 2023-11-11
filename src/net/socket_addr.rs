//! Defines the [`SocketAddr`] type.

use core::ffi::c_int;
use core::fmt;
use core::mem::size_of;

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
            family => unreachable!("unknown socket address family: {family}"),
        }
    }

    /// Returns the raw value of the socket address.
    pub(crate) fn write_raw(&self, storage: &mut libc::sockaddr_storage) -> *mut libc::sockaddr {
        match self {
            Self::V4(ip, port) => {
                let ret = unsafe { &mut *(storage as *mut _ as *mut libc::sockaddr_in) };
                ret.sin_family = libc::AF_INET as _;

                #[cfg(target_os = "macos")]
                {
                    ret.sin_len = core::mem::size_of::<libc::sockaddr_in>() as _;
                }

                ret.sin_addr.s_addr = u32::from_be_bytes(*ip);
                ret.sin_port = port.to_be();
            }
            Self::V6(ip, port) => {
                let ret = unsafe { &mut *(storage as *mut _ as *mut libc::sockaddr_in6) };
                ret.sin6_family = libc::AF_INET6 as _;

                #[cfg(target_os = "macos")]
                {
                    ret.sin6_len = core::mem::size_of::<libc::sockaddr_in6>() as _;
                }

                ret.sin6_addr.s6_addr = *ip;
                ret.sin6_port = port.to_be();
            }
        }

        storage as *mut _ as *mut _
    }
}

impl fmt::Display for SocketAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::V4(ip, port) => format_ipv4(ip, port, f),
            Self::V6(ip, port) => format_ipv6(ip, port, f),
        }
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
    #[inline]
    pub fn to_raw(self) -> c_int {
        self as _
    }

    /// Returns the size of a [`libc::sockaddr`] object with this family.
    pub(crate) fn len_of_sockaddr(self) -> libc::socklen_t {
        match self {
            Self::V4 => size_of::<libc::sockaddr_in>() as _,
            Self::V6 => size_of::<libc::sockaddr_in6>() as _,
        }
    }
}

/// Formats an IPv4 using the provided formatter.
fn format_ipv4(ip: [u8; 4], port: u16, f: &mut fmt::Formatter) -> fmt::Result {
    let [a, b, c, d] = ip;
    write!(f, "{a}.{b}.{c}.{d}:{port}")
}

/// Formats an IPv6 using the provided formatter.
fn format_ipv6(ip: [u8; 16], port: u16, f: &mut fmt::Formatter) -> fmt::Result {
    let ip: [u16; 8] = core::array::from_fn(|i| u16::from_be_bytes([ip[i * 2], ip[i * 2 + 1]]));

    // Look for the longest sequence of zeros.
    let mut best = 0..0;

    let mut i = 0;
    let mut current_start = 0;
    while i < 8 {
        if ip[i] == 0 {
            current_start = i;

            while i < 8 && ip[i] == 0 {
                i += 1;
            }

            if i - current_start > best.len() {
                best = current_start..i;
            }
        }

        i += 1;
        current_start += 1;
    }

    // Write the address.
    write!(f, "[")?;
    if best.start == best.end {
        for (i, n) in ip.iter().enumerate() {
            write!(f, "{n}")?;

            if i != 7 {
                write!(f, ":")?;
            }
        }
    } else {
        if best.start == 0 {
            write!(f, ":")?;
        }
        for n in &ip[0..best.start] {
            write!(f, "{n}:")?;
        }
        for n in &ip[best.end..8] {
            write!(f, ":{n}")?;
        }
        if best.end == 8 {
            write!(f, ":")?;
        }
    }
    write!(f, "]:{port}")?;

    Ok(())
}
