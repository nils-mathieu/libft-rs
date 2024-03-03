//! Time-related types and functions.

use core::ops::{Add, AddAssign, Sub};
use core::time::Duration;

#[cfg(feature = "futures")]
use crate::futures;

/// A clock that measures time since some epoch, goes at some rate.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[doc(alias = "clockid_t")]
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
    #[doc(alias = "clock_gettime")]
    pub fn get(self) -> Instant {
        let mut timespec = unsafe { core::mem::zeroed() };

        let ret = unsafe { libc::clock_gettime(self.0, &mut timespec) };
        debug_assert!(ret == 0);

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

    /// Computes `self + other`. If the result overflows, `None` is returned.
    #[inline]
    pub fn checked_add(self, other: Duration) -> Option<Self> {
        self.0.checked_add(other).map(Self)
    }

    /// Returns the amount of time elapsed since this instant was created.
    ///
    /// If the operation overflows, it saturates at `Instant::FAR_FUTURE`.
    #[inline]
    pub fn saturating_add(self, rhs: Duration) -> Instant {
        Instant(self.0.saturating_add(rhs))
    }

    /// Returns the amount of time elapsed since this instant was created.
    ///
    /// If `rhs` is earlier than `self`, the result is the amount of time between
    /// the two events.
    ///
    /// If `self` is earlier than `rhs`, the result is `Duration::ZERO`.
    #[inline]
    pub fn saturating_sub(self, rhs: Instant) -> Duration {
        self.0.saturating_sub(rhs.0)
    }
}

impl Sub for Instant {
    type Output = Duration;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.0.saturating_sub(rhs.0)
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    #[inline]
    fn add(self, rhs: Duration) -> Self::Output {
        Instant(self.0 + rhs)
    }
}

impl AddAssign<Duration> for Instant {
    #[inline]
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

/// Returns a [`Future`](core::future::Future) that sleeps for the requested amount of time.
#[cfg(feature = "futures")]
#[inline]
pub fn async_sleep(duration: Duration) -> futures::Sleep {
    let now = Clock::MONOTONIC.get();
    futures::Sleep::new(now.saturating_add(duration))
}

/// Returns a [`Future`](core::future::Future) that sleeps until the reuqested instant.
///
/// If the instant is in the past, the task will sleep for a single runtime tick. If this behavior
/// is not wanted, please check manually with `Clock::MONOTONIC.get()`.
#[cfg(feature = "futures")]
#[inline]
pub fn async_sleep_until(alarm: Instant) -> futures::Sleep {
    futures::Sleep::new(alarm)
}
