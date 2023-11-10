//! Time-related types and functions.

use core::ops::{Add, Sub};
use core::time::Duration;

use crate::{Errno, Result};

/// A clock that measures time since some epoch, goes at some rate.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Clock(libc::clockid_t);

impl Clock {
    /// A clock that measures time since the Unix epoch (January 1, 1970).
    ///
    /// This clock is subject to NTP adjustments.
    ///
    /// Modifying this clock requires superuser privileges.
    pub const REALTIME: Clock = Clock(libc::CLOCK_MONOTONIC);

    /// Non-adjustable clock that measures time since some unspecified starting point.
    ///
    /// This clock is not affected by discontinuous jumps in the system time (e.g., if the
    /// system administrator manually changes the clock), but is affected by the
    /// incremental adjustments performed by NTP.
    pub const MONOTONIC: Clock = Clock(libc::CLOCK_MONOTONIC);

    /// Returns the current instant associated with this clock.
    pub fn get(self) -> Result<Instant> {
        let mut timespec = unsafe { core::mem::zeroed() };
        let ret = unsafe { libc::clock_gettime(self.0, &mut timespec) };
        if ret == 0 {
            Ok(Instant(Duration::new(
                timespec.tv_sec as u64,
                timespec.tv_nsec as u32,
            )))
        } else {
            Err(Errno::last())
        }
    }
}

/// An instant in time.
///
/// # Clocks
///
/// Depending on which clock they come from, `Instant`s may or may not be
/// comparable to one another. For example, `Instant`s from the `Monotonic`
/// clock are only comparable to other `Instant`s from the same `Monotonic`
/// clock.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Instant(pub Duration);

impl Instant {
    /// Returns the earliest possible instant.
    pub const EPOCH: Instant = Instant(Duration::ZERO);

    /// An [`Instant`] that represents the far future.
    pub const FAR_FUTURE: Instant = Instant(Duration::from_secs(u64::MAX));

    /// Returns the amount of time elapsed since this instant was created.
    ///
    /// If the operation overflows, it saturates at `Instant::FAR_FUTURE`.
    pub fn saturating_add(self, rhs: Duration) -> Instant {
        Instant(self.0.saturating_add(rhs))
    }

    /// Returns the amount of time elapsed since this instant was created.
    ///
    /// If `rhs` is earlier than `self`, the result is the amount of time between
    /// the two events.
    ///
    /// If `self` is earlier than `rhs`, the result is `Duration::ZERO`.
    pub fn saturating_sub(self, rhs: Instant) -> Duration {
        self.0.saturating_sub(rhs.0)
    }
}

impl Sub for Instant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0.saturating_sub(rhs.0)
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Self::Output {
        Instant(self.0 + rhs)
    }
}
