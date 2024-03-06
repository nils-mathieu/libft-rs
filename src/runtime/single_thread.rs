use alloc::boxed::Box;
use core::cell::{Cell, UnsafeCell};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use super::task_waker::TaskWaker;
use super::tasks::{TaskId, Tasks};
use super::waker::waker_from_task_id;
use super::DynTask;
use crate::fd::PollFd;
use crate::malloc::OutOfMemory;
use crate::Instant;

/// A [`core::cell::RefCell<T>`] that's only checked in debug builds.
struct UnsafeRefCell<T> {
    /// The protected value.
    value: UnsafeCell<T>,
    /// Whether the cell is currently borrowed.
    borrowed: Cell<bool>,
}

impl<T> UnsafeRefCell<T> {
    /// Creates a new [`UnsafeRefCell`] instance.
    #[inline]
    pub const fn new(val: T) -> Self {
        Self {
            value: UnsafeCell::new(val),
            borrowed: Cell::new(false),
        }
    }

    /// Borrows the value temporarily.
    ///
    /// # Safety
    ///
    /// This function must not be called reentrantly.
    ///
    /// # Panics
    ///
    /// This function panics in debug builds if called reentrantly.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[track_caller]
    pub unsafe fn update<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        #[cfg(debug_assertions)]
        assert!(
            !self.borrowed.replace(true),
            "attempted to reentrantly borrow the runtime"
        );

        #[cfg(debug_assertions)]
        let _guard = crate::utils::Guard::new((), |_| self.borrowed.set(false));

        unsafe { f(&mut *self.value.get()) }
    }
}

/// The list of all tasks running on the current thread.
#[thread_local]
static TASKS: UnsafeRefCell<Tasks<'static>> = UnsafeRefCell::new(Tasks::new());

/// The task waker responsnible for waking tasks up and blocking until they're ready
/// to work.
#[thread_local]
static WAKER: UnsafeRefCell<TaskWaker> = UnsafeRefCell::new(TaskWaker::new());

/// Attempts to spawn a boxed task on the current runtime.
///
/// # Errors
///
/// This function fails if memory cannot be allocated to spawn the created task.
pub fn try_spawn_boxed(task: DynTask<'static>) -> Result<(), OutOfMemory> {
    unsafe { TASKS.update(|tasks| tasks.insert(task)) }
}

/// Attempts to spawn a task on the current runtime.
///
/// # Errors
///
/// This function fails if memory cannot be allocated to spawn the created task.
pub fn try_spawn(task: impl 'static + Future<Output = ()>) -> Result<(), OutOfMemory> {
    // SAFETY:
    //  A box can always be safely pinned.
    try_spawn_boxed(unsafe { Pin::new_unchecked(Box::try_new(task)?) })
}

/// Spawns a task on the current runtime.
///
/// # Panics
///
/// This function panics if memory cannot be allocated to spawn the created task.
#[track_caller]
pub fn spawn_boxed(task: DynTask<'static>) {
    try_spawn_boxed(task).expect("failed to spawn task")
}

/// Spawns a task on the current runtime.
///
/// # Panics
///
/// This function panics if memory cannot be allocated to spawn the created task.
#[track_caller]
pub fn spawn(task: impl 'static + Future<Output = ()>) {
    try_spawn(task).expect("failed to spawn task")
}

/// Execute all pending tasks until the runtime is idle.
///
/// # Blocking behavior
///
/// If no tasks are currently ready, this function blocks until at least one can make progress.
///
/// # Returns
///
/// The function returns the number of tasks that are still pending.
pub fn run_until_idle() -> crate::Result<usize> {
    unsafe {
        WAKER.update(|waker| {
            if !waker.is_empty() {
                waker.wait_any()
            } else {
                Ok(())
            }
        })?;
    }

    while let Some((id, mut task)) = unsafe { TASKS.update(|tasks| tasks.take_any_ready_task()) } {
        let waker = waker_from_task_id(id);
        let mut ctx = Context::from_waker(&waker);
        match task.as_mut().poll(&mut ctx) {
            Poll::Pending => {
                // The task is pending.
                unsafe { TASKS.update(|tasks| tasks.put_back_waiting(id, task)) };
            }
            Poll::Ready(()) => {
                unsafe { TASKS.update(|tasks| tasks.put_back_nothing(id)) };
                drop(task);
            }
        }
    }

    Ok(unsafe { TASKS.update(|tasks| tasks.len()) })
}

/// Schedules the provided waker to be consumed when `pollfd` becomes ready.
#[inline]
pub fn wake_me_up_on_io(pollfd: PollFd, waker: Waker) -> Result<(), OutOfMemory> {
    unsafe { WAKER.update(move |w| w.wake_me_up_on_io(pollfd, waker)) }
}

/// Schedules the provided waker to be consumed when `instant` is reached.
#[inline]
pub fn wake_me_up_on_time(at: Instant, waker: Waker) -> Result<(), OutOfMemory> {
    unsafe { WAKER.update(move |w| w.wake_me_up_on_time(at, waker)) }
}

/// Clears the runtime.
///
/// This removes all tasks and wakers from the runtime. Pending tasks are dropped, and wakers
/// are consumed.
pub fn clear() {
    unsafe {
        TASKS.update(|tasks| tasks.clear());
        WAKER.update(|waker| waker.clear());
    }
}

/// Wakes a task up manually.
///
/// If the provided task does not exist, or if it isn't pending, this function does nothing.
#[inline]
pub(super) fn wake_up_manual(id: TaskId) {
    unsafe { TASKS.update(|tasks| tasks.mark_ready(id)) };
}
