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

/// Contains the state of the async runtime.
struct Runtime<'a> {
    tasks: Tasks<'a>,
    waker: TaskWaker,
}

/// Calls the provided closure with a reference to the current runtime.
///
/// # Panics
///
/// This function panics in debug builds if called reentrantly.
///
/// # Safety
///
/// This function must not be called reentrantly.
#[track_caller]
#[cfg_attr(not(debug_assertions), inline(always))]
unsafe fn with_runtime<F, R>(f: F) -> R
where
    F: FnOnce(&mut Runtime<'static>) -> R,
{
    #[thread_local]
    static RUNTIME: UnsafeCell<Runtime<'static>> = UnsafeCell::new(Runtime {
        tasks: Tasks::new(),
        waker: TaskWaker::new(),
    });

    #[thread_local]
    #[cfg(debug_assertions)]
    static BORROWED: Cell<bool> = Cell::new(false);

    #[cfg(debug_assertions)]
    assert!(
        !BORROWED.replace(true),
        "attempted to reentrantly borrow the runtime"
    );

    #[cfg(debug_assertions)]
    let _guard = crate::utils::Guard::new((), |_| BORROWED.set(false));

    // SAFETY:
    //  The caller must make sure not to call this function reentrantly.
    f(unsafe { &mut *RUNTIME.get() })
}

/// Attempts to spawn a boxed task on the current runtime.
///
/// # Errors
///
/// This function fails if memory cannot be allocated to spawn the created task.
pub fn try_spawn_boxed(task: DynTask<'static>) -> Result<(), OutOfMemory> {
    unsafe { with_runtime(|rt| rt.tasks.insert(task)) }
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
        with_runtime(|rt| {
            if !rt.waker.is_empty() {
                rt.waker.wait_any()
            } else {
                Ok(())
            }
        })?;
    }

    while let Some((id, mut task)) = unsafe { with_runtime(|rt| rt.tasks.take_any_ready_task()) } {
        let waker = waker_from_task_id(id);
        let mut ctx = Context::from_waker(&waker);
        match task.as_mut().poll(&mut ctx) {
            Poll::Pending => {
                // The task is pending.
                unsafe { with_runtime(|rt| rt.tasks.put_back_waiting(id, task)) };
            }
            Poll::Ready(()) => {
                unsafe { with_runtime(|rt| rt.tasks.put_back_nothing(id)) };
                drop(task);
            }
        }
    }

    Ok(unsafe { with_runtime(|rt| rt.tasks.len()) })
}

/// Schedules the provided waker to be consumed when `pollfd` becomes ready.
#[inline]
pub fn wake_me_up_on_io(pollfd: PollFd, waker: Waker) -> Result<(), OutOfMemory> {
    unsafe { with_runtime(move |rt| rt.waker.wake_me_up_on_io(pollfd, waker)) }
}

/// Schedules the provided waker to be consumed when `instant` is reached.
#[inline]
pub fn wake_me_up_on_time(at: Instant, waker: Waker) -> Result<(), OutOfMemory> {
    unsafe { with_runtime(move |rt| rt.waker.wake_me_up_on_time(at, waker)) }
}

/// Wakes a task up manually.
///
/// If the provided task does not exist, or if it isn't pending, this function does nothing.
pub(super) fn wake_up_manual(id: TaskId) {
    unsafe { with_runtime(|rt| rt.tasks.mark_ready(id)) };
}
