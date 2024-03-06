use core::task::Waker;

use alloc::collections::BinaryHeap;
use alloc::vec::Vec;

use crate::fd::PollFd;
use crate::malloc::OutOfMemory;
use crate::{Clock, Instant};

/// A task that is waiting for an instant to be reached before waking up.
///
/// This type features a custom `PartialEq`, `Eq`, `PartialOrd` and `Ord` implementation
/// based on its `instant` field. Note that *earlier* means *larger*.
struct Sleeper {
    /// The task to wake up.
    pub waker: Waker,
    /// The instant to reach.
    pub instant: Instant,
}

impl PartialEq for Sleeper {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}

impl Eq for Sleeper {}

impl PartialOrd for Sleeper {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Sleeper {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        other.instant.cmp(&self.instant) // earilier means larger
    }
}

/// Holds a list of [`Waker`] that are waiting for a specific I/O event to occur before waking
/// up their associated task.
pub struct TaskWaker {
    /// Tasks that are blocked on I/O.
    io: Vec<PollFd>,
    /// Wakers associated with tasks in `io`.
    io_wakers: Vec<Waker>,
    /// Tasks that are waiting for an alarm.
    sleepers: BinaryHeap<Sleeper>,
}

impl TaskWaker {
    /// Creates a new [`TaskWaker`] instance.
    #[inline]
    pub const fn new() -> Self {
        Self {
            io: Vec::new(),
            io_wakers: Vec::new(),
            sleepers: BinaryHeap::new(),
        }
    }

    /// Clears the list of wakers and I/O events.
    pub fn clear(&mut self) {
        self.io.clear();
        self.io_wakers.clear();
        self.sleepers.clear();
    }

    /// Requests a wake up when the provided `PollFd` instance becomes ready.
    pub fn wake_me_up_on_io(&mut self, fd: PollFd, waker: Waker) -> Result<(), OutOfMemory> {
        self.io.try_reserve(1)?;
        self.io_wakers.try_reserve(1)?;

        self.io.push(fd);
        self.io_wakers.push(waker);

        Ok(())
    }

    /// Requests a wake up when the provided instant is reached.
    ///
    /// Note that it is possible for the task to be woken up *after* that alarm.
    pub fn wake_me_up_on_time(&mut self, at: Instant, waker: Waker) -> Result<(), OutOfMemory> {
        self.sleepers.try_reserve(1)?;
        self.sleepers.push(Sleeper { instant: at, waker });
        Ok(())
    }

    /// Returns whether no tasks are waiting.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.io.is_empty() && self.sleepers.is_empty()
    }

    /// Blocks the current thread until any of the tasks this [`TaskWaker`] manages becomes
    /// ready.
    ///
    /// This function takes care of calling the `wake` function on all wakers that are
    /// ready.
    ///
    /// # Notes
    ///
    /// If no tasks are currently waiting, this function blocks forever.
    pub fn wait_any(&mut self) -> crate::Result<()> {
        // Compute the timeout for the `poll` call.
        // Use saturating semantics to make sure that if any task must be woken up right now,
        // the timeout of 0.
        let timeout = self
            .sleepers
            .peek()
            .map(|s| s.instant.saturating_sub(Clock::MONOTONIC.get()));

        let mut ready = crate::fd::poll(&mut self.io, timeout)?;

        // Wake up all the tasks that are waiting for an alarm if the alarm has already
        // been reached.
        let now = Clock::MONOTONIC.get();
        while let Some(sleeper) = self.sleepers.peek() {
            if now >= sleeper.instant {
                let sleeper = unsafe { self.sleepers.pop().unwrap_unchecked() };
                sleeper.waker.wake();
            } else {
                break;
            }
        }

        debug_assert!(self.io.len() == self.io_wakers.len());

        // Wake up tasks that were waiting on I/O.
        let mut index = 0;
        while ready > 0 {
            debug_assert!(index < self.io.len(), "{} < {}", index, self.io.len());

            unsafe {
                let pollfd = self.io.get_unchecked(index);
                if pollfd.ready() {
                    swap_remove_unchecked(&mut self.io, index);
                    swap_remove_unchecked(&mut self.io_wakers, index).wake();
                    ready -= 1;
                } else {
                    index += 1;
                }
            }
        }

        Ok(())
    }
}

/// Performs a "swap remove" operation on the provided vector without checking whether the
/// provided index is valid.
unsafe fn swap_remove_unchecked<T>(v: &mut Vec<T>, index: usize) -> T {
    let p = v.as_mut_ptr();

    unsafe {
        let new_len = v.len() - 1;

        let ret = p.add(index).read();

        if new_len != index {
            p.add(index).write(p.add(new_len).read());
        }

        v.set_len(new_len);

        ret
    }
}
