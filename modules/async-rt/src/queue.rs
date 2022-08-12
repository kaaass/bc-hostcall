//! 本文件来自于 https://github.com/rustwasm/wasm-bindgen，遵照 MIT license 引入

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

struct QueueState {
    // The queue of Tasks which are to be run in order. In practice this is all the
    // synchronous work of futures, and each `Task` represents calling `poll` on
    // a future "at the right time".
    tasks: RefCell<VecDeque<Rc<crate::task::Task>>>,
}

impl QueueState {
    fn run_all(&self) {
        // Stop when all tasks that have been scheduled before this tick have been run.
        // Tasks that are scheduled while running tasks will run on the next tick.
        let mut task_count_left = self.tasks.borrow().len();
        while task_count_left > 0 {
            task_count_left -= 1;
            let task = match self.tasks.borrow_mut().pop_front() {
                Some(task) => task,
                None => break,
            };
            task.run();
        }

        // All of the Tasks have been run, so it's now possible to schedule the
        // next tick again
    }
}

pub(crate) struct Queue {
    state: Rc<QueueState>,
    closure: Box<dyn Fn()>,
}

impl Queue {
    // Schedule a task to run on the next tick
    pub(crate) fn schedule_task(&self, task: Rc<crate::task::Task>) {
        self.state.tasks.borrow_mut().push_back(task);
    }
    // Append a task to the currently running queue, or schedule it
    pub(crate) fn push_task(&self, task: Rc<crate::task::Task>) {
        // It would make sense to run this task on the same tick.  For now, we
        // make the simplifying choice of always scheduling tasks for a future tick.
        self.schedule_task(task)
    }
}

impl Queue {
    fn new() -> Self {
        let state = Rc::new(QueueState {
            tasks: RefCell::new(VecDeque::new()),
        });

        Self {
            closure: {
                let state = Rc::clone(&state);

                // This closure will only be called on the next microtask event
                // tick
                Box::new(move || state.run_all())
            },

            state,
        }
    }

    pub fn run_all(&self) {
        (self.closure.as_ref())();
    }
}

thread_local! {
    pub(crate) static QUEUE: Queue = Queue::new();
}
