use core::ops::Sub;
use core::time::Duration;

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
    pub fn get(self) -> Instant {
        let mut timespec = unsafe { core::mem::zeroed() };
        unsafe { libc::clock_gettime(self.0, &mut timespec) };
        Instant(Duration::new(
            timespec.tv_sec as u64,
            timespec.tv_nsec as u32,
        ))
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
}

impl Sub for Instant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0.saturating_sub(rhs.0)
    }
}
