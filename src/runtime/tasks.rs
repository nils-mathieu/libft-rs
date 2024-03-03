use alloc::vec::Vec;

use crate::alloc_ext::SafeVecExt;
use crate::malloc::OutOfMemory;

use super::DynTask;

/// A slot in the task queue.
enum TaskSlot<'a> {
    /// The task is waiting for some event. A waker has been dispatched for it and we are waiting
    /// for it to put the task back into the ready state.
    Waiting(DynTask<'a>),
    /// The task is ready to be run.
    Ready {
        /// The task to run.
        task: DynTask<'a>,
        /// The index of the next task in the ready list.
        next: TaskId,
    },
    /// The task is currently running. We needed to remove the task from the queue while it is
    /// running because we need access to the whole queue from *within* the task.
    Running,
    /// The task is finished.
    ///
    /// The next free slot is stored in the `usize`.
    Vacant {
        /// The index of the next free slot.
        next: TaskId,
    },
}

/// The ID of a task.
pub type TaskId = usize;

/// A list of tasks with stable indices.
pub struct Tasks<'a> {
    /// The tasks that are currently running.
    tasks: Vec<TaskSlot<'a>>,
    /// The head of the free list.
    ///
    /// When the free list is empty, this is set to `usize::MAX`.
    free_list_head: TaskId,
    /// The head of the list of free tasks.
    ///
    /// When the list of ready tasks is empty, this is set to `usize::MAX`.
    ready_list_head: TaskId,
    /// The number of tasks that are in the list.
    len: usize,
}

impl<'a> Tasks<'a> {
    /// Creates a new [`Tasks`] instance.
    pub const fn new() -> Self {
        Self {
            tasks: Vec::new(),
            free_list_head: usize::MAX,
            ready_list_head: usize::MAX,
            len: 0,
        }
    }

    /// Returns the number of tasks in the list.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Inserts a new task in the list.
    pub fn insert(&mut self, task: DynTask<'a>) -> Result<(), OutOfMemory> {
        let new_slot = TaskSlot::Ready {
            task,
            next: self.ready_list_head,
        };

        match self.tasks.get_mut(self.free_list_head) {
            Some(slot) => {
                debug_assert!(matches!(slot, TaskSlot::Vacant { .. }));

                self.ready_list_head = self.free_list_head;
                match core::mem::replace(slot, new_slot) {
                    TaskSlot::Vacant { next } => self.free_list_head = next,
                    _ => unsafe { core::hint::unreachable_unchecked() },
                };

                self.len += 1;

                Ok(())
            }
            None => {
                let new_head = self.tasks.len();
                self.tasks.try_push(new_slot)?;
                self.ready_list_head = new_head;

                self.len += 1;

                Ok(())
            }
        }
    }

    /// Removes a "ready" task from the list, marking it as "Running".
    pub fn take_any_ready_task(&mut self) -> Option<(TaskId, DynTask<'a>)> {
        let id = self.ready_list_head;

        let slot = self.tasks.get_mut(id)?;
        debug_assert!(matches!(slot, TaskSlot::Ready { .. }));

        match core::mem::replace(slot, TaskSlot::Running) {
            TaskSlot::Ready { task, next } => {
                self.ready_list_head = next;
                Some((id, task))
            }
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    /// Puts a running task back into the list.
    ///
    /// # Safety
    ///
    /// The task at `id` must be currently running.
    pub unsafe fn put_back_waiting(&mut self, id: TaskId, task: DynTask<'a>) {
        debug_assert!(id < self.tasks.len());
        let slot = unsafe { self.tasks.get_unchecked_mut(id) };
        debug_assert!(matches!(slot, TaskSlot::Running));

        // We know that the task is running, so we can skip any kind of dropping by forcing a write.
        unsafe { (slot as *mut TaskSlot).write(TaskSlot::Waiting(task)) };
    }

    /// Makes a running task free.
    ///
    /// This should be called when the task that was running is done.
    ///
    /// # Safety
    ///
    /// The task at `id` must be currently running.
    pub unsafe fn put_back_nothing(&mut self, id: TaskId) {
        debug_assert!(id < self.tasks.len());
        let slot = unsafe { self.tasks.get_unchecked_mut(id) };
        debug_assert!(matches!(slot, TaskSlot::Running));

        let new_slot = TaskSlot::Vacant {
            next: self.free_list_head,
        };
        self.free_list_head = id;

        // We know that the task is running, so we can skip any kind of dropping by forcing a write.
        unsafe { (slot as *mut TaskSlot).write(new_slot) };

        self.len -= 1;
    }

    /// Marks a task as ready.
    ///
    /// # Returns
    ///
    /// This function returns whether the task was successfully marked as ready. Specifically, if
    /// the task does not exist, or if the task is not in the "waiting" state, this function will
    /// return `false`.
    pub fn mark_ready(&mut self, id: TaskId) -> bool {
        let Some(slot) = self.tasks.get_mut(id) else {
            return false;
        };

        if !matches!(slot, TaskSlot::Waiting(..)) {
            return false;
        }

        match core::mem::replace(slot, TaskSlot::Running) {
            TaskSlot::Waiting(task) => {
                // We know that the slot is now in the `Running` state, ensuring that we don't
                // need to drop any value. This write ensures that Rust does not attempt
                // to drop anything.
                unsafe {
                    (slot as *mut TaskSlot).write(TaskSlot::Ready {
                        task,
                        next: self.ready_list_head,
                    });
                }

                self.ready_list_head = id;
            }
            _ => unsafe { core::hint::unreachable_unchecked() },
        }

        true
    }
}
