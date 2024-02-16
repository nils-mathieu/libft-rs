/// A user identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Uid(libc::uid_t);

impl Uid {
    /// Returns the user ID of the root user.
    pub const ROOT: Self = Self(0);

    /// Returns the real user ID of the calling process.
    #[inline]
    pub fn current() -> Self {
        Self(unsafe { libc::getuid() })
    }

    /// Returns the effective user ID of the calling process.
    #[inline]
    pub fn effective() -> Self {
        Self(unsafe { libc::geteuid() })
    }

    /// Returns whether this user is the root user.
    #[inline(always)]
    pub const fn is_root(self) -> bool {
        self.0 == Self::ROOT.0
    }
}
