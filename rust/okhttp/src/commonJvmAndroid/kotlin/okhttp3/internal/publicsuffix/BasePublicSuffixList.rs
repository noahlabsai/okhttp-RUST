use std::sync::atomic::{AtomicBoolean, Ordering};
use std::sync::{Mutex};
use std::io::{Read, Error as IoError};
use okio::{ByteString, Source};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixList;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::RequestBody::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::publicsuffix::PublicSuffixDatabase::*;

// BasePublicSuffixList provides the common logic for loading the public suffix list.
// Since Kotlin's abstract class cannot be directly translated to a Rust struct,
// we use a trait for the abstract methods and a helper struct for the shared state.
pub trait BasePublicSuffixList: PublicSuffixList {
    fn list_source(&self) -> Box<dyn Source>;
    fn path(&self) -> Box<dyn std::any::Any>;
    fn get_state(&self) -> &BasePublicSuffixListState;
}

pub struct BasePublicSuffixListState {
    // True after we've attempted to read the list for the first time.
    pub list_read: AtomicBoolean,
    // Used for concurrent threads reading the list for the first time.
    // To mimic CountDownLatch(1), we use a Mutex and a boolean.
    pub read_complete: Mutex<bool>,
    // Guarded by the mutex in BasePublicSuffixListState.
    pub bytes: Mutex<Option<ByteString>>,
    pub exception_bytes: Mutex<Option<ByteString>>,
    pub read_failure: Mutex<Option<IoError>>,
}

impl BasePublicSuffixListState {
    pub fn new() -> Self {
        Self {
            list_read: AtomicBoolean::new(false),
            read_complete: Mutex::new(false),
            bytes: Mutex::new(None),
            exception_bytes: Mutex::new(None),
            read_failure: Mutex::new(None),
        }
    }
}

// Implementation of the shared logic for BasePublicSuffixList.
pub fn ensure_loaded_impl<T: BasePublicSuffixList>(list: &T) {
    let state = list.get_state();
    
    // if (!listRead.get() && listRead.compareAndSet(false, true))
    if !state.list_read.load(Ordering::SeqCst) 
       && state.list_read.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() 
    {
        read_the_list_uninterruptibly(list);
    } else {
        // readCompleteLatch.r#await()
        loop {
            let complete = state.read_complete.lock().unwrap();
            if *complete {
                break;
            }
            drop(complete);
            std::thread::yield_now();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    // Check if bytes are initialized
    let bytes_guard = state.bytes.lock().unwrap();
    if bytes_guard.is_none() {
        let failure = state.read_failure.lock().unwrap();
        let path_val = list.path();
        let err_msg = format!("Unable to load {:?} resource.", path_val);
        
        // Kotlin's IllegalStateException with initCause
        panic!("{}. Cause: {:?}", err_msg, *failure);
    }
}

fn read_the_list_uninterruptibly<T: BasePublicSuffixList>(list: &T) {
    let mut interrupted = false;
    loop {
        match read_the_list(list) {
            Ok(_) => {
                if interrupted {
                    // In Rust, we cannot easily set the thread interrupt flag like Java.
                    // We acknowledge the interruption here.
                }
                return;
            }
            Err(e) => {
                // Check if it's an "InterruptedIOException"
                if e.kind() == std::io::ErrorKind::Interrupted {
                    interrupted = true;
                    continue;
                } else {
                    let state = list.get_state();
                    let mut failure = state.read_failure.lock().unwrap();
                    *failure = Some(e);
                    
                    // Signal completion even on failure
                    let mut complete = state.read_complete.lock().unwrap();
                    *complete = true;
                    return;
                }
            }
        }
    }
}

fn read_the_list<T: BasePublicSuffixList>(list: &T) -> Result<(), IoError> {
    let mut source = list.list_source();
    
    // readInt()
    let mut int_buf = [0u8; 4];
    source.read_exact(&mut int_buf)?;
    let total_bytes = i32::from_be_bytes(int_buf);
    
    // readByteString(totalBytes)
    let mut list_bytes_vec = vec![0u8; total_bytes as usize];
    source.read_exact(&mut list_bytes_vec)?;
    let public_suffix_list_bytes = ByteString::from_slice(&list_bytes_vec);

    // readInt()
    let mut int_buf_exc = [0u8; 4];
    source.read_exact(&mut int_buf_exc)?;
    let total_exception_bytes = i32::from_be_bytes(int_buf_exc);
    
    // readByteString(totalExceptionBytes)
    let mut exc_bytes_vec = vec![0u8; total_exception_bytes as usize];
    source.read_exact(&mut exc_bytes_vec)?;
    let public_suffix_exception_list_bytes = ByteString::from_slice(&exc_bytes_vec);

    // synchronized(this)
    let state = list.get_state();
    {
        let mut b = state.bytes.lock().unwrap();
        *b = Some(public_suffix_list_bytes);
        let mut eb = state.exception_bytes.lock().unwrap();
        *eb = Some(public_suffix_exception_list_bytes);
    }

    // finally { readCompleteLatch.countDown() }
    let mut complete = state.read_complete.lock().unwrap();
    *complete = true;

    Ok(())
}
