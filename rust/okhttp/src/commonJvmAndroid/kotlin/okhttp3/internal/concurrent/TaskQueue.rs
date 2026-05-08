use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Condvar;
use std::sync::Mutex as StdMutex;

use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::concurrent::Task::*;
use crate::build_logic::src::main::kotlin::okhttp_testing_conventions_gradle::*;

// Mocking CountDownLatch as it's a Java utility
pub struct CountDownLatch {
    count: StdMutex<usize>,
    cvar: Condvar,
}

impl CountDownLatch {
    pub fn new(count: usize) -> Self {
        Self {
            count: StdMutex::new(count),
            cvar: Condvar::new(),
        }
    }

    pub fn count_down(&self) {
        let mut count = self.count.lock().unwrap();
        if *count > 0 {
            *count -= 1;
            if *count == 0 {
                self.cvar.notify_all();
            }
        }
    }

    pub fn await_latch(&self) {
        let mut count = self.count.lock().unwrap();
        while *count > 0 {
            count = self.cvar.wait(count).unwrap();
        }
    }
}

#[derive(Debug)]
pub struct RejectedExecutionException;
impl std::fmt::Display for RejectedExecutionException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RejectedExecutionException")
    }
}
impl std::error::Error for RejectedExecutionException {}

// Forward declarations for TaskRunner and its components to ensure compilability
pub trait TaskLogger {
    fn task_log<T: Task + ?Sized>(&self, task: &T, queue: &TaskQueue, message: impl FnOnce() -> String);
}

pub trait Backend {
    fn nano_time(&self) -> i64;
}

pub struct TaskRunner {
    pub logger: Box<dyn TaskLogger>,
    pub backend: Box<dyn Backend>,
    // In a real implementation, this would be a Mutex or similar
    lock: StdMutex<()>, 
}

impl TaskRunner {
    pub fn with_lock<F, R>(&self, f: F) -> R 
    where F: FnOnce() -> R {
        let _guard = self.lock.lock().unwrap();
        f()
    }

    pub fn kick_coordinator(&self, queue: &TaskQueue) {
        // Implementation for kicking the coordinator
    }

    pub fn assert_lock_not_held(&self) {
        // Implementation to check if current thread holds the lock
    }
}

pub struct TaskQueue {
    task_runner: Rc<TaskRunner>,
    name: String,
    shutdown: RefCell<bool>,
    active_task: RefCell<Option<Rc<RefCell<dyn Task>>>>,
    future_tasks: RefCell<Vec<Rc<RefCell<dyn Task>>>>,
    cancel_active_task: RefCell<bool>,
}

impl TaskQueue {
    pub fn new(task_runner: Rc<TaskRunner>, name: String) -> Self {
        Self {
            task_runner,
            name,
            shutdown: RefCell::new(false),
            active_task: RefCell::new(None),
            future_tasks: RefCell::new(Vec::new()),
            cancel_active_task: RefCell::new(false),
        }
    }

    pub fn scheduled_tasks(&self) -> Vec<Rc<RefCell<dyn Task>>> {
        self.task_runner.with_lock(|| {
            self.future_tasks.borrow().clone()
        })
    }

    pub fn schedule(&self, task: Rc<RefCell<dyn Task>>, delay_nanos: i64) -> Result<(), Box<dyn std::error::Error>> {
        self.task_runner.with_lock(|| {
            if *self.shutdown.borrow() {
                if task.borrow().cancelable() {
                    self.task_runner.logger.task_log(&*task.borrow(), self, || "schedule canceled (queue is shutdown)".to_string());
                    return Ok(());
                }
                self.task_runner.logger.task_log(&*task.borrow(), self, || "schedule failed (queue is shutdown)".to_string());
                return Err(Box::new(RejectedExecutionException));
            }

            if self.schedule_and_decide(task.clone(), delay_nanos, false) {
                self.task_runner.kick_coordinator(self);
            }
            Ok(())
        })
    }

    pub fn schedule_with_block<F>(&self, name: String, delay_nanos: i64, block: F) -> Result<(), Box<dyn std::error::Error>> 
    where F: Fn() -> i64 + 'static {
        struct LambdaTask<F: Fn() -> i64> {
            name: String,
            block: F,
            queue: Option<Rc<RefCell<TaskQueue>>>,
            next_execute_nano_time: i64,
        }
        impl Task for LambdaTask<F> {
            fn name(&self) -> &str { &self.name }
            fn cancelable(&self) -> bool { true }
            fn run_once(&mut self) -> i64 { (self.block)() }
            fn queue(&self) -> Option<Rc<RefCell<TaskQueue>>> { self.queue.clone() }
            fn set_queue(&mut self, queue: Option<Rc<RefCell<TaskQueue>>>) { self.queue = queue; }
            fn next_execute_nano_time(&self) -> i64 { self.next_execute_nano_time }
            fn set_next_execute_nano_time(&mut self, time: i64) { self.next_execute_nano_time = time; }
        }

        let task = Rc::new(RefCell::new(LambdaTask {
            name,
            block,
            queue: None,
            next_execute_nano_time: 0,
        }));
        self.schedule(task, delay_nanos)
    }

    pub fn execute<F>(&self, name: String, delay_nanos: i64, cancelable: bool, block: F) -> Result<(), Box<dyn std::error::Error>> 
    where F: Fn() + 'static {
        struct ExecTask<F: Fn()> {
            name: String,
            cancelable: bool,
            block: F,
            queue: Option<Rc<RefCell<TaskQueue>>>,
            next_execute_nano_time: i64,
        }
        impl<F: Fn()> Task for ExecTask<F> {
            fn name(&self) -> &str { &self.name }
            fn cancelable(&self) -> bool { self.cancelable }
            fn run_once(&mut self) -> i64 {
                (self.block)();
                -1
            }
            fn queue(&self) -> Option<Rc<RefCell<TaskQueue>>> { self.queue.clone() }
            fn set_queue(&mut self, queue: Option<Rc<RefCell<TaskQueue>>>) { self.queue = queue; }
            fn next_execute_nano_time(&self) -> i64 { self.next_execute_nano_time }
            fn set_next_execute_nano_time(&mut self, time: i64) { self.next_execute_nano_time = time; }
        }

        let task = Rc::new(RefCell::new(ExecTask {
            name,
            cancelable,
            block,
            queue: None,
            next_execute_nano_time: 0,
        }));
        self.schedule(task, delay_nanos)
    }

    pub fn idle_latch(&self) -> Rc<CountDownLatch> {
        self.task_runner.with_lock(|| {
            if self.active_task.borrow().is_none() && self.future_tasks.borrow().is_empty() {
                return Rc::new(CountDownLatch::new(0));
            }

            if let Some(ref active) = *self.active_task.borrow() {
                if let Some(await_task) = active.borrow().as_any().downcast_ref::<AwaitIdleTask>() {
                    return Rc::new(await_task.latch.clone());
                }
            }

            for future_task in self.future_tasks.borrow().iter() {
                if let Some(await_task) = future_task.borrow().as_any().downcast_ref::<AwaitIdleTask>() {
                    return Rc::new(await_task.latch.clone());
                }
            }

            let new_task = Rc::new(RefCell::new(AwaitIdleTask::new()));
            let latch = new_task.borrow().latch.clone();
            if self.schedule_and_decide(new_task, 0, false) {
                self.task_runner.kick_coordinator(self);
            }
            latch
        })
    }

    fn schedule_and_decide(&self, task: Rc<RefCell<dyn Task>>, delay_nanos: i64, recurrence: bool) -> bool {
        // Note: Task::init_queue is handled by the trait implementation
        task.borrow_mut().init_queue(Rc::new(RefCell::new(self.clone_internal())));

        let now = self.task_runner.backend.nano_time();
        let execute_nano_time = now + delay_nanos;

        let mut future_tasks = self.future_tasks.borrow_mut();
        let existing_index = future_tasks.iter().position(|t| Rc::ptr_eq(t, &task));

        if let Some(idx) = existing_index {
            if task.borrow().next_execute_nano_time() <= execute_nano_time {
                self.task_runner.logger.task_log(&*task.borrow(), self, || "already scheduled".to_string());
                return false;
            }
            future_tasks.remove(idx);
        }

        task.borrow_mut().set_next_execute_nano_time(execute_nano_time);
        self.task_runner.logger.task_log(&*task.borrow(), self, || {
            if recurrence {
                format!("run again after {}", format_duration(execute_nano_time - now))
            } else {
                format!("scheduled after {}", format_duration(execute_nano_time - now))
            }
        });

        let insert_at = future_tasks.iter().position(|t| t.borrow().next_execute_nano_time() - now > delay_nanos)
            .unwrap_or(future_tasks.len());
        
        future_tasks.insert(insert_at, task);
        insert_at == 0
    }

    pub fn cancel_all(&self) {
        self.task_runner.assert_lock_not_held();
        self.task_runner.with_lock(|| {
            if self.cancel_all_and_decide() {
                self.task_runner.kick_coordinator(self);
            }
        });
    }

    pub fn shutdown(&self) {
        self.task_runner.assert_lock_not_held();
        self.task_runner.with_lock(|| {
            *self.shutdown.borrow_mut() = true;
            if self.cancel_all_and_decide() {
                self.task_runner.kick_coordinator(self);
            }
        });
    }

    fn cancel_all_and_decide(&self) -> bool {
        if let Some(ref active) = *self.active_task.borrow() {
            if active.borrow().cancelable() {
                *self.cancel_active_task.borrow_mut() = true;
            }
        }

        let mut tasks_canceled = false;
        let mut future_tasks = self.future_tasks.borrow_mut();
        for i in (0..future_tasks.len()).rev() {
            if future_tasks[i].borrow().cancelable() {
                self.task_runner.logger.task_log(&*future_tasks[i].borrow(), self, || "canceled".to_string());
                tasks_canceled = true;
                future_tasks.remove(i);
            }
        }
        tasks_canceled
    }

    // Helper to simulate the internal constructor/reference behavior
    fn clone_internal(&self) -> TaskQueue {
        // This is a simplification; in a real system, TaskQueue would likely be wrapped in Rc
        unimplemented!("TaskQueue internal cloning requires Rc wrapper")
    }
}

impl fmt::Display for TaskQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

struct AwaitIdleTask {
    latch: Rc<CountDownLatch>,
    name: String,
    cancelable: bool,
    queue: Option<Rc<RefCell<TaskQueue>>>,
    next_execute_nano_time: i64,
}

impl AwaitIdleTask {
    fn new() -> Self {
        Self {
            latch: Rc::new(CountDownLatch::new(1)),
            name: "okhttp awaitIdle".to_string(),
            cancelable: false,
            queue: None,
            next_execute_nano_time: 0,
        }
    }
}

impl Task for AwaitIdleTask {
    fn name(&self) -> &str { &self.name }
    fn cancelable(&self) -> bool { self.cancelable }
    fn run_once(&mut self) -> i64 {
        self.latch.count_down();
        -1
    }
    fn queue(&self) -> Option<Rc<RefCell<TaskQueue>>> { self.queue.clone() }
    fn set_queue(&mut self, queue: Option<Rc<RefCell<TaskQueue>>>) { self.queue = queue; }
    fn next_execute_nano_time(&self) -> i64 { self.next_execute_nano_time }
    fn set_next_execute_nano_time(&mut self, time: i64) { self.next_execute_nano_time = time; }
}

// Extension to Task to allow downcasting for AwaitIdleTask
trait TaskAny {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: Task + 'static> TaskAny for T {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

fn format_duration(nanos: i64) -> String {
    format!("{}ns", nanos)
}