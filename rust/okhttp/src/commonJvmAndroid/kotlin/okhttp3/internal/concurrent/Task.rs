use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use crate::build_logic::src::main::kotlin::okhttp_testing_conventions_gradle::*;

// A unit of work that can be executed one or more times.
//
// Recurrence
// ----------
//
// Tasks control their recurrence schedule. The `run_once` function returns -1 to signify that the
// task should not be executed again. Otherwise it returns a delay until the next execution.
//
// A task has at most one next execution. If the same task instance is scheduled multiple times, the
// earliest one wins. This applies to both executions scheduled with `TaskRunner::Queue::schedule` and
// those implied by the returned execution delay.
//
// Cancellation
// ------------
//
// Tasks may be canceled while they are waiting to be executed, or while they are executing.
//
// Canceling a task that is waiting to execute prevents that upcoming execution. Canceling a task
// that is currently executing does not impact the ongoing run, but it does prevent a recurrence
// from being scheduled.
//
// Tasks may opt-out of cancellation with `cancelable = false`. Such tasks will recur until they
// decide not to by returning -1.
//
// Task Queues
// -----------
//
// Tasks are bound to the `TaskQueue` they are scheduled in. Each queue is sequential and the tasks
// within it never execute concurrently. It is an error to use a task in multiple queues.
pub trait Task {
    // Returns the name of the task.
    fn name(&self) -> &str;

    // Returns whether the task is cancelable.
    fn cancelable(&self) -> bool;

    // Returns the delay in nanoseconds until the next execution, or -1 to not reschedule.
    fn run_once(&mut self) -> i64;

    // Internal access to the queue the task is bound to.
    fn queue(&self) -> Option<Rc<RefCell<TaskQueue>>>;
    fn set_queue(&mut self, queue: Option<Rc<RefCell<TaskQueue>>>);

    // Internal access to the next execution time.
    fn next_execute_nano_time(&self) -> i64;
    fn set_next_execute_nano_time(&mut self, time: i64);

    // Initializes the queue for the task.
    fn init_queue(&mut self, queue: Rc<RefCell<TaskQueue>>) {
        let current_queue = self.queue();
        
        // Check if it's already in this queue
        if let Some(ref q) = current_queue {
            if Rc::ptr_eq(q, &queue) {
                return;
            }
        }

        // Ensure it's not in any other queue
        if current_queue.is_some() {
            panic!("task is in multiple queues");
        }
        
        self.set_queue(Some(queue));
    }
}

// Base implementation for Task to avoid repeating common fields in every concrete Task.
// Since Rust doesn't have abstract classes, we use a helper struct that concrete tasks can compose.
#[derive(Debug, Clone)]
pub struct TaskBase {
    pub name: String,
    pub cancelable: bool,
    pub queue: Option<Rc<RefCell<TaskQueue>>>,
    pub next_execute_nano_time: i64,
}

impl TaskBase {
    pub fn new(name: String, cancelable: bool) -> Self {
        Self {
            name,
            cancelable,
            queue: None,
            next_execute_nano_time: -1,
        }
    }
}

impl fmt::Display for TaskBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

// generated-compatibility for TaskQueue as it is referenced by Task.
// The actual implementation would be in the TaskQueue file.
pub struct TaskQueue {
    // Queue implementation details...
}

// Example of how a concrete Task would be implemented using the Task trait and TaskBase.
// 
// pub struct MyTask {
//     base: TaskBase,
// }
// 
// impl Task for MyTask {
//     fn name(&self) -> &str { &self.base.name }
//     fn cancelable(&self) -> bool { self.base.cancelable }
//     fn run_once(&mut self) -> i64 {
//         // business logic here
//         -1
//     }
//     fn queue(&self) -> Option<Rc<RefCell<TaskQueue>>> { self.base.queue.clone() }
//     fn set_queue(&mut self, queue: Option<Rc<RefCell<TaskQueue>>>) { self.base.queue = queue; }
//     fn next_execute_nano_time(&self) -> i64 { self.base.next_execute_nano_time }
//     fn set_next_execute_nano_time(&mut self, time: i64) { self.base.next_execute_nano_time = time; }
// }
)}
