use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::{
    future::Future,
    task::{Context, Poll, Waker},
};

use crossbeam_queue::ArrayQueue;
use x86_64::instructions::interrupts::{self, enable_interrupts_and_hlt};

use super::{Task, TaskId};

pub struct Executor<'f> {
    tasks: BTreeMap<TaskId, Task<'f>>,
    task_id_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl<'f> Executor<'f> {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_id_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, future: impl Future<Output = ()> + 'f) {
        let task = Task::new(future);
        let task_id = task.id;
        if self.tasks.insert(task_id, task).is_some() {
            panic!("task with id {:?} already exists", task_id);
        }
        self.task_id_queue
            .push(task_id)
            .expect("task queue is full");
    }

    // TODO: Since this takes &mut self, we can't call spawn() while the executor is running.
    // One solution to this is to share ownership of the tasks and tasks_id_queue fields with
    // some other structure and spawn tasks from there instead.
    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            // Interrupts may occur after polling tasks, so carefully check the queue
            interrupts::disable();
            if self.task_id_queue.is_empty() {
                enable_interrupts_and_hlt();
            } else {
                interrupts::enable();
            }
        }
    }

    fn run_ready_tasks(&mut self) {
        let Self {
            tasks,
            task_id_queue,
            waker_cache,
        } = self;

        while let Ok(task_id) = task_id_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists
            };
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| Waker::from(TaskWaker::new(task_id, Arc::clone(task_id_queue))));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }
}

struct TaskWaker {
    task_id: TaskId,
    task_id_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_id_queue: Arc<ArrayQueue<TaskId>>) -> Self {
        TaskWaker {
            task_id,
            task_id_queue,
        }
    }

    fn wake_task(&self) {
        self.task_id_queue
            .push(self.task_id)
            .expect("task queue is full");
    }
}

impl From<TaskWaker> for Waker {
    fn from(task_waker: TaskWaker) -> Self {
        Waker::from(Arc::new(task_waker))
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
