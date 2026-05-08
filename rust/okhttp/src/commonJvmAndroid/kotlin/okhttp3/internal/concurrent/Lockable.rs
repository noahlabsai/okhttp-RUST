use std::sync::{Mutex, MutexGuard};
use std::time::Duration;
use std::thread;

/// Marker trait for objects that use a locking mechanism.
/// In Rust, the JVM's `synchronized` behavior is typically represented by a Mutex.
pub trait Lockable: Send + Sync {}

/// Internal helper to simulate JVM's Object.wait() / notify() behavior.
/// Since Rust's Mutex does not have built-in wait/notify (Condition Variables are separate),
/// this trait defines the interface for those capabilities.
pub trait LockableExt {
    fn wait(&self);
    fn notify(&self);
    fn notify_all(&self);
    fn await_nanos(&self, nanos: i64);
}

/// Mock for the `assertionsEnabled` flag from the Kotlin source.
pub static ASSERTIONS_ENABLED: bool = cfg!(debug_assertions);

/// Internal function to check if the current thread holds the lock.
/// Note: Rust's std::sync::Mutex does not provide a `holdsLock` method.
/// In a production translation, this would require a custom Mutex wrapper or 
/// platform-specific implementation.
fn thread_holds_lock<T: Lockable>(lockable: &T) -> bool {
    // This is a semantic placeholder as Rust Mutexes are acquired via guards,
    // not by "locking the object" in the JVM sense.
    false 
}

pub trait LockableUtils {
    fn assert_lock_not_held(&self);
    fn assert_lock_held(&self);
}

impl<T: Lockable> LockableUtils for T {
    fn assert_lock_not_held(&self) {
        if ASSERTIONS_ENABLED && thread_holds_lock(self) {
            panic!(
                "Thread {:?} MUST NOT hold lock on {:?}",
                thread::current().id(),
                std::any::type_name::<T>()
            );
        }
    }

    fn assert_lock_held(&self) {
        if ASSERTIONS_ENABLED && !thread_holds_lock(self) {
            panic!(
                "Thread {:?} MUST hold lock on {:?}",
                thread::current().id(),
                std::any::type_name::<T>()
            );
        }
    }
}

/// Equivalent to Kotlin's `withLock` extension function.
/// In Rust, this is achieved by locking a Mutex and executing a closure.
pub fn with_lock<T, R, F>(mutex: &Mutex<T>, action: F) -> R 
where 
    F: FnOnce(&mut T) -> R 
{
    let mut guard = mutex.lock().expect("Mutex poisoned");
    action(&mut *guard)
}

/// Implementation of the wait/notify logic for types that wrap a Condvar and Mutex.
/// This is the idiomatic Rust way to achieve the behavior of JVM's synchronized wait/notify.
pub struct JvmLock {
    pub lock: Mutex<()>,
    // In a real implementation, a std::sync::Condvar would be here.
}

impl Lockable for JvmLock {}

impl LockableExt for JvmLock {
    fn wait(&self) {
        // Implementation would use Condvar::wait
    }

    fn notify(&self) {
        // Implementation would use Condvar::notify_one
    }

    fn notify_all(&self) {
        // Implementation would use Condvar::notify_all
    }

    fn await_nanos(&self, nanos: i64) {
        let ms = nanos / 1_000_000;
        let ns = nanos - (ms * 1_000_000);
        if ms > 0 || nanos > 0 {
            // Implementation would use Condvar::wait_timeout
            let _ = Duration::new(ms as u64, ns as u32);
        }
    }
}