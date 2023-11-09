use core::ffi::c_int;

/// The type of a socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SocketType {
    /// A stream socket (TCP).
    Stream = libc::SOCK_STREAM as _,
    /// A datagram socket (UDP).
    Datagram = libc::SOCK_DGRAM as _,
}

impl SocketType {
    /// Turns this [`SocketType`] into its raw value.
    pub fn to_raw(self) -> c_int {
        self as _
    }
}
