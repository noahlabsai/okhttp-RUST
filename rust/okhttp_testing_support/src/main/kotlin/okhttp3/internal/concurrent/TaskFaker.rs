use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicI32, Ordering};

// Static instance counter for logger naming
static INSTANCE_COUNT: AtomicI32 = AtomicI32::new(0);

// Represents a task that can be scheduled and run by the TaskFaker.
trait SerialTask: Send + Sync {
    fn is_ready(&self) -> bool {
        true
    }
    fn start(&self, faker: &Arc<Mutex<TaskFakerInner>>);
}

// The task representing the main test thread.
struct TestThreadSerialTask;
impl SerialTask for TestThreadSerialTask {
    fn start(&self, _faker: &Arc<Mutex<TaskFakerInner>>) {
        panic!("unexpected call");
    }
}

// A task that wraps a closure to be executed on a thread pool.
struct RunnableSerialTask {
    runnable: Box<dyn FnOnce() + Send>,
}
impl SerialTask for RunnableSerialTask {

}

// Specialized implementation for RunnableSerialTask to handle the async execution.
struct RunnableSerialTaskExecutor {
    runnable: Box<dyn FnOnce() + Send>,
    faker_inner: Arc<Mutex<TaskFakerInner>>,
    task_runner: Arc<TaskRunner>,
}

impl RunnableSerialTaskExecutor {
    fn execute(self) {
        self.task_runner.assert_lock_not_held();
        
        // Verify current task is this one (simulated by checking state in inner)
        {
            let inner = self.faker_inner.lock().unwrap();
            if inner.current_task_id != 1 { // Simplified ID check
                // In a real port, we'd use pointer equality or unique IDs
            }
        }

        // Simulate the runnable execution
        (self.runnable)();

        // Cleanup and trigger next task
        let mut inner = self.faker_inner.lock().unwrap();
        inner.active_threads -= 1;
        inner.start_next_task();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ResumePriority {
    BeforeOtherTasks,
    AfterEnqueuedTasks,
    AfterOtherTasks,
}

// Internal state of TaskFaker to allow shared access between the coordinator and worker threads.
struct TaskFakerInner {
    is_parallel: bool,
    execute_call_count: i32,
    nano_time: i64,
    serial_task_queue: VecDeque<Arc<dyn SerialTask>>,
    current_task: Arc<dyn SerialTask>,
    waiting_coordinator_task: Option<Arc<dyn SerialTask>>,
    waiting_coordinator_interrupted: bool,
    waiting_coordinator_notified: bool,
    context_switch_count: i32,
    active_threads: i32,
    condvar: Condvar,
}

impl TaskFakerInner {
    fn start_next_task(&mut self) -> Option<Arc<dyn SerialTask>> {
        let index = self.serial_task_queue.iter().position(|t| t.is_ready());
        if index == None {
            return None;
        }
        let idx = index.unwrap();
        let next_task = self.serial_task_queue.remove(idx).unwrap();
        self.current_task = Arc::clone(&next_task);
        self.context_switch_count += 1;
        
        // We can't call start() here directly if it needs the lock, 
        // but the trait is designed to take the Arc<Mutex<Inner>>.
        // This is handled by the TaskFaker wrapper.
        Some(next_task)
    }
}

// The public API for TaskRunner.
pub struct TaskRunner {
    inner: Arc<Mutex<TaskFakerInner>>,
    backend: Arc<dyn TaskRunnerBackend>,
}

impl TaskRunner {

    fn with_lock<F, R>(&self, f: F) -> R 
    where F: FnOnce(&mut TaskFakerInner) -> R {
        let mut inner = self.inner.lock().unwrap();
        f(&mut inner)
    }

    fn wait(&self) {
        let inner = self.inner.lock().unwrap();
        inner.condvar.wait(inner).unwrap();
    }

    fn notify_all(&self) {
        let inner = self.inner.lock().unwrap();
        inner.condvar.notify_all();
    }

    fn active_queues(&self) -> Vec<i32> {
        // Simplified for translation
        Vec::new()
    }
}

trait TaskRunnerBackend: Send + Sync {
    fn execute(&self, task_runner: &TaskRunner, runnable: Box<dyn FnOnce() + Send>);
    fn nano_time(&self) -> i64;
    fn coordinator_notify(&self, task_runner: &TaskRunner);
    fn coordinator_wait(&self, task_runner: &TaskRunner, nanos: i64) -> Result<(), std::io::Error>;
}

pub struct TaskFaker {
    inner: Arc<Mutex<TaskFakerInner>>,
    task_runner: Arc<TaskRunner>,
    executor: std::sync::mpsc::Sender<Box<dyn FnOnce() + Send>>,
}

impl TaskFaker {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<Box<dyn FnOnce() + Send>>();
        
        // Simple thread pool simulation
        thread::spawn(move || {
            while let Ok(task) = rx.recv() {
                (task)();
            }
        });

        let inner = Arc::new(Mutex::new(TaskFakerInner {
            is_parallel: false,
            execute_call_count: 0,
            nano_time: 0,
            serial_task_queue: VecDeque::new(),
            current_task: Arc::new(TestThreadSerialTask),
            waiting_coordinator_task: None,
            waiting_coordinator_interrupted: false,
            waiting_coordinator_notified: false,
            context_switch_count: 0,
            active_threads: 0,
            condvar: Condvar::new(),
        }));

        // We use a recursive-like structure for the backend
        // In Rust, we define the backend as a separate struct to avoid circular Arc
        let backend = Arc::new(TaskFakerBackend {
            inner: Arc::clone(&inner),
            executor: tx.clone(),
        });

        let task_runner = Arc::new(TaskRunner {
            inner: Arc::clone(&inner),
            backend: backend,
        });

        TaskFaker {
            inner,
            task_runner,
            executor: tx,
        }
    }

    pub fn run_tasks(&self) {
        let time = self.task_runner.with_lock(|i| i.nano_time);
        self.advance_until(time);
    }

    pub fn advance_until(&self, new_time: i64) {
        self.task_runner.assert_lock_not_held();
        self.task_runner.with_lock(|inner| {
            // check(currentTask == TestThreadSerialTask)
            inner.nano_time = new_time;
            self.yield_until_internal(inner, ResumePriority::AfterOtherTasks, || true);
        });
    }

    pub fn assert_no_more_tasks(&self) {
        self.task_runner.assert_lock_not_held();
        self.task_runner.with_lock(|inner| {
            assert_eq!(inner.active_threads, 0);
        });
    }

    pub fn interrupt_coordinator_thread(&self) {
        self.task_runner.assert_lock_not_held();
        
        self.task_runner.with_lock(|inner| {
            // require(currentTask == TestThreadSerialTask)
            
            struct InterruptTask;
            impl SerialTask for InterruptTask {
                fn start(&self, faker: &Arc<Mutex<TaskFakerInner>>) {
                    let mut inner = faker.lock().unwrap();
                    inner.waiting_coordinator_interrupted = true;
                    if let Some(coord) = inner.waiting_coordinator_task.take() {
                        inner.current_task = coord;
                        // notify_all is called via the TaskRunner wrapper in real impl
                    } else {
                        panic!("no coordinator waiting");
                    }
                }
            }
            inner.serial_task_queue.push_back(Arc::new(InterruptTask));
        });

        self.run_tasks();
    }

    pub fn run_next_task(&self) {
        self.task_runner.assert_lock_not_held();
        self.task_runner.with_lock(|inner| {
            let before = inner.context_switch_count;
            self.yield_until_internal(inner, ResumePriority::BeforeOtherTasks, || {
                inner.context_switch_count > before
            });
        });
    }

    pub fn sleep(&self, duration_nanos: i64) {
        self.task_runner.with_lock(|inner| {
            let sleep_until = inner.nano_time + duration_nanos;
            self.yield_until_internal(inner, ResumePriority::AfterEnqueuedTasks, || {
                inner.nano_time >= sleep_until
            });
        });
    }

    pub fn yield_task(&self) {
        self.task_runner.assert_lock_not_held();
        self.task_runner.with_lock(|inner| {
            self.yield_until_internal(inner, ResumePriority::AfterEnqueuedTasks, || true);
        });
    }

    fn yield_until_internal<F>(&self, inner_mut: &mut TaskFakerInner, strategy: ResumePriority, condition: F) 
    where F: Fn() -> bool {
        let self_task = Arc::clone(&inner_mut.current_task);

        struct YieldTask<F> {
            cond: F,
            original_task: Arc<dyn SerialTask>,
            faker_inner: Arc<Mutex<TaskFakerInner>>,
        }
        impl<F: Fn() -> bool + Send + Sync> SerialTask for YieldTask<F> {
            fn is_ready(&self) -> bool { (self.cond)() }
            fn start(&self, faker: &Arc<Mutex<TaskFakerInner>>) {
                let mut inner = faker.lock().unwrap();
                inner.current_task = Arc::clone(&self.original_task);
                // notify_all logic
            }
        }

        // This is a simplified version of the tailrec yieldUntil
        // In Rust, we'd implement this as a loop.
        let yield_task = Arc::new(YieldTask {
            cond: condition,
            original_task: Arc::clone(&self_task),
            faker_inner: Arc::clone(&self.inner),
        });

        match strategy {
            ResumePriority::BeforeOtherTasks => inner_mut.serial_task_queue.push_front(Arc::clone(&yield_task) as Arc<dyn SerialTask>),
            _ => inner_mut.serial_task_queue.push_back(Arc::clone(&yield_task) as Arc<dyn SerialTask>),
        }

        let started = inner_mut.start_next_task();
        let other_started = started.as_ref().map_or(false, |t| !Arc::ptr_eq(t, &yield_task as &Arc<dyn SerialTask>));

        // Wait loop
        while inner_mut.current_task != self_task {
            // This requires dropping the lock and waiting on condvar
            // In this translation, we simulate the wait
            break; 
        }

        inner_mut.serial_task_queue.retain(|t| !Arc::ptr_eq(t, &yield_task as &Arc<dyn SerialTask>));

        if strategy == ResumePriority::AfterOtherTasks && other_started {
            // Recurse
        }
    }

    pub fn is_idle(&self) -> bool {
        self.task_runner.active_queues().is_empty()
    }
}

impl Drop for TaskFaker {

}

struct TaskFakerBackend {
    inner: Arc<Mutex<TaskFakerInner>>,
    executor: std::sync::mpsc::Sender<Box<dyn FnOnce() + Send>>,
}

impl TaskRunnerBackend for TaskFakerBackend {
    fn execute(&self, task_runner: &TaskRunner, runnable: Box<dyn FnOnce() + Send>) {
        let mut inner = self.inner.lock().unwrap();
        
        struct RunnableTask {
            runnable: Box<dyn FnOnce() + Send>,
        }
        impl SerialTask for RunnableTask {

        }

        inner.serial_task_queue.push_back(Arc::new(RunnableTask { runnable: runnable }) as Arc<dyn SerialTask>);
        inner.execute_call_count += 1;
        inner.is_parallel = inner.serial_task_queue.len() > 1;
    }

    fn nano_time(&self) -> i64 {
        self.inner.lock().unwrap().nano_time
    }

    fn coordinator_notify(&self, task_runner: &TaskRunner) {
        let mut inner = self.inner.lock().unwrap();
        assert!(inner.waiting_coordinator_task.is_some());

        struct NotifyTask;
        impl SerialTask for NotifyTask {
            fn start(&self, faker: &Arc<Mutex<TaskFakerInner>>) {
                let mut inner = faker.lock().unwrap();
                if let Some(coord) = inner.waiting_coordinator_task.take() {
                    inner.waiting_coordinator_notified = true;
                    inner.current_task = coord;
                }
            }
        }
        inner.serial_task_queue.push_back(Arc::new(NotifyTask) as Arc<dyn SerialTask>);
    }

    fn coordinator_wait(&self, task_runner: &TaskRunner, nanos: i64) -> Result<(), std::io::Error> {
        let mut inner = self.inner.lock().unwrap();
        assert!(inner.waiting_coordinator_task.is_none());
        if nanos == 0 { return Ok(()); }

        let wait_until = inner.nano_time + nanos;
        let self_task = Arc::clone(&inner.current_task);
        inner.waiting_coordinator_task = Some(Arc::clone(&self_task));
        inner.waiting_coordinator_notified = false;
        inner.waiting_coordinator_interrupted = false;

        // Yield until condition
        // (Implementation of yield_until logic)

        inner.waiting_coordinator_task = None;
        inner.waiting_coordinator_notified = false;
        if inner.waiting_coordinator_interrupted {
            inner.waiting_coordinator_interrupted = false;
            return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "Interrupted"));
        }
        Ok(())
    }
}

// Mock of the BlockingQueue for the translation
pub struct TaskFakerBlockingQueue<T> {
    delegate: Mutex<VecDeque<T>>,
    task_runner: Arc<TaskRunner>,
    edit_count: Mutex<i32>,
}

impl<T: Send> TaskFakerBlockingQueue<T> {
    pub fn poll(&self, timeout: i64, unit: Duration) -> Option<T> {
        self.task_runner.with_lock(|inner| {
            let wait_until = inner.nano_time + timeout; // simplified
            loop {
                if let Some(item) = self.delegate.lock().unwrap().pop_front() {
                    return Some(item);
                }
                if inner.nano_time >= wait_until {
                    return None;
                }
                let before = *self.edit_count.lock().unwrap();
                // yield_until logic
                break;
            }
            None
        })
    }

    pub fn put(&self, element: T) {
        self.task_runner.with_lock(|_inner| {
            self.delegate.lock().unwrap().push_back(element);
            *self.edit_count.lock().unwrap() += 1;
        });
    }
}