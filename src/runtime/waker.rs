use core::task::{RawWaker, RawWakerVTable, Waker};

use super::tasks::TaskId;

/// Creates a [`RawWaker`] for the provided [`TaskId`].
#[inline]
pub fn raw_waker_from_task_id(task: TaskId) -> RawWaker {
    RawWaker::new(task as *const (), &VTABLE)
}

/// Creates a [`Waker`] for the provided [`TaskId`].
#[inline]
pub fn waker_from_task_id(task: TaskId) -> Waker {
    unsafe { Waker::from_raw(raw_waker_from_task_id(task)) }
}

const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

fn clone(this: *const ()) -> RawWaker {
    raw_waker_from_task_id(this as TaskId)
}

fn wake(this: *const ()) {
    super::wake_up_manual(this as TaskId);
}

fn wake_by_ref(this: *const ()) {
    super::wake_up_manual(this as TaskId);
}

fn drop(_this: *const ()) {}
