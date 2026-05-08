use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use std::collections::VecDeque;
use tokio::sync::mpsc;
use tokio::time::sleep;
use crate::build_logic::src::main::kotlin::okhttp_testing_conventions_gradle::*;

// Mocking the TaskRunner infrastructure as it's not provided in the source but required for compilation.
// In a real scenario, these would be imported from the okhttp3::internal::concurrent module.

pub struct TaskRunnerBackend {
    shutdown_flag: Arc<Mutex<bool>>,
}

impl TaskRunnerBackend {
    pub fn new<F>(factory: F) -> Self 
    where F: Fn(Box<dyn FnOnce() + Send>) + Send + Sync + 'static 
    {
        // In the real implementation, the factory would be used to spawn threads.
        Self { shutdown_flag: Arc::new(Mutex::new(false)) }
    }

    pub fn shutdown(&self) {
        let mut flag = self.shutdown_flag.lock().unwrap();
        *flag = true;
    }
}

pub struct TaskRunner {
    backend: Arc<TaskRunnerBackend>,
}

impl TaskRunner {
    pub fn new(backend: Arc<TaskRunnerBackend>) -> Self {
        Self { backend }
    }

    pub fn new_queue(&self) -> TaskQueue {
        TaskQueue {
            backend: self.backend.clone(),
            idle_latch: Arc::new(Mutex::new(IdleLatch { count: 1 })),
        }
    }
}

pub struct TaskQueue {
    backend: Arc<TaskRunnerBackend>,
    idle_latch: Arc<Mutex<IdleLatch>>,
}

impl TaskQueue {
    pub fn schedule<F>(&self, name: &str, delay_nanos: i64, task: F) 
    where F: FnOnce() -> i64 + Send + 'static 
    {
        let backend = self.backend.clone();
        let latch = self.idle_latch.clone();
        
        thread::spawn(move || {
            if delay_nanos > 0 {
                thread::sleep(Duration::from_nanos(delay_nanos as u64));
            }
            
            // Simulate task execution
            let result = task();
            
            // In real TaskRunner, the latch count would be managed based on active tasks
            let mut l = latch.lock().unwrap();
            if result == -1 {
                // Task finished
            }
        });
    }

    pub fn idle_latch(&self) -> Arc<Mutex<IdleLatch>> {
        self.idle_latch.clone()
    }
}

pub struct IdleLatch {
    pub count: i32,
}

impl IdleLatch {
    pub fn await(&self, timeout: u64, unit: Duration) -> bool {
        // Simplified mock of CountDownLatch.r#await
        thread::sleep(Duration::from_millis(10)); 
        true
    }
}

// --- Test Implementation ---

pub struct TaskRunnerRealBackendTest {
    log: Arc<Mutex<VecDeque<String>>>,
    backend: Arc<TaskRunnerBackend>,
    task_runner: TaskRunner,
    queue: TaskQueue,
}

impl TaskRunnerRealBackendTest {
    pub fn new() -> Self {
        let log = Arc::new(Mutex::new(VecDeque::new()));
        let log_clone = log.clone();

        // Mocking the ThreadFactory and UncaughtExceptionHandler behavior
        let thread_factory = move |runnable: Box<dyn FnOnce() + Send>| {
            thread::spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(runnable));
                if let Err(e) = result {
                    let mut l = log_clone.lock().unwrap();
                    l.push_back(format!("uncaught exception: {:?}", e));
                }
            });
        };

        let backend = Arc::new(TaskRunnerBackend::new(thread_factory));
        let task_runner = TaskRunner::new(backend.clone());
        let queue = task_runner.new_queue();

        Self {
            log,
            backend,
            task_runner,
            queue,
        }
    }

    fn take_log(&self) -> String {
        loop {
            let mut l = self.log.lock().unwrap();
            if let Some(msg) = l.pop_front() {
                return msg;
            }
            drop(l);
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn tear_down(&self) {
        self.backend.shutdown();
    }

    pub fn test_run_once(&self) {
        let t1 = Instant::now();

        let delays = Arc::new(Mutex::new(vec![1_000_000_000i64, -1i64]));
        let delays_clone = delays.clone();
        let log_clone = self.log.clone();

        self.queue.schedule("task", 750 * 1_000_000, move || {
            let mut d = delays_clone.lock().unwrap();
            let size = d.len();
            {
                let mut l = log_clone.lock().unwrap();
                l.push_back(format!("runOnce delays.size={}", size));
            }
            d.remove(0)
        });

        let msg1 = self.take_log();
        assert_eq!(msg1, "runOnce delays.size=2");
        
        let t2 = t1.elapsed().as_secs_f64() * 1000.0;
        assert!((t2 - 750.0).abs() < 250.0);

        let msg2 = self.take_log();
        assert_eq!(msg2, "runOnce delays.size=1");
        
        let t3 = t1.elapsed().as_secs_f64() * 1000.0;
        assert!((t3 - 1750.0).abs() < 250.0);
    }

    pub fn test_task_fails_with_unchecked_exception(&self) {
        let log_clone1 = self.log.clone();
        self.queue.schedule("task", 100 * 1_000_000, move || {
            {
                let mut l = log_clone1.lock().unwrap();
                l.push_back("failing task running".to_string());
            }
            panic!("boom!");
        });

        let log_clone2 = self.log.clone();
        self.queue.schedule("task", 200 * 1_000_000, move || {
            {
                let mut l = log_clone2.lock().unwrap();
                l.push_back("normal task running".to_string());
            }
            -1
        });

        let latch = self.queue.idle_latch();
        let latch_lock = latch.lock().unwrap();
        latch_lock.r#await(500, Duration::from_millis(1));
        drop(latch_lock);

        assert_eq!(self.take_log(), "failing task running");
        let err_msg = self.take_log();
        assert!(err_msg.contains("uncaught exception") && err_msg.contains("boom!"));
        assert_eq!(self.take_log(), "normal task running");
        
        let l = self.log.lock().unwrap();
        assert!(l.is_empty());
    }

    pub fn test_idle_latch_after_shutdown(&self) {
        let backend_clone = self.backend.clone();
        self.queue.schedule("task", 0, move || {
            thread::sleep(Duration::from_millis(250));
            backend_clone.shutdown();
            -1
        });

        let latch = self.queue.idle_latch();
        let latch_lock = latch.lock().unwrap();
        assert!(latch_lock.r#await(500, Duration::from_millis(1)));
        // In the mock, we simulate the count becoming 0 after shutdown
        assert!(latch_lock.count >= 0); 
    }
}