use std::collections::HashSet;
use std::io::{self, Error, ErrorKind};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::cache::FaultHidingSink::*;

// Mocking okio types as they are external dependencies in the original Kotlin code.
// In a real production environment, these would be imported from the okio crate.
pub type Path = PathBuf;

pub trait Sink: Send + Sync {
    fn write(&mut self, source: &mut Buffer, byte_count: i64) -> io::Result<()>;
}


pub trait FileSystem: Send + Sync {
    fn atomic_move(&self, source: &Path, target: &Path) -> io::Result<()>;
    fn delete(&self, path: &Path, must_exist: bool) -> io::Result<()>;
    fn delete_recursively(&self, file_or_directory: &Path, must_exist: bool) -> io::Result<()>;
    fn appending_sink(&self, file: &Path, must_exist: bool) -> io::Result<Box<dyn Sink>>;
    fn sink(&self, file: &Path, must_create: bool) -> io::Result<Box<dyn Sink>>;
}

// ForwardingFileSystem equivalent: a wrapper that delegates to an underlying FileSystem.
pub struct ForwardingFileSystem {
    delegate: Arc<dyn FileSystem>,
}

impl ForwardingFileSystem {
    pub fn new(delegate: Arc<dyn FileSystem>) -> Self {
        Self { delegate }
    }
}

impl FileSystem for ForwardingFileSystem {
    fn atomic_move(&self, source: &Path, target: &Path) -> io::Result<()> {
        self.delegate.atomic_move(source, target)
    }
    fn delete(&self, path: &Path, must_exist: bool) -> io::Result<()> {
        self.delegate.delete(path, must_exist)
    }
    fn delete_recursively(&self, file_or_directory: &Path, must_exist: bool) -> io::Result<()> {
        self.delegate.delete_recursively(file_or_directory, must_exist)
    }
    fn appending_sink(&self, file: &Path, must_exist: bool) -> io::Result<Box<dyn Sink>> {
        self.delegate.appending_sink(file, must_exist)
    }
    fn sink(&self, file: &Path, must_create: bool) -> io::Result<Box<dyn Sink>> {
        self.delegate.sink(file, must_create)
    }
}

// ForwardingSink equivalent: a wrapper that delegates to an underlying Sink.

impl ForwardingSink {
    pub fn new(delegate: Box<dyn Sink>) -> Self {
        Self { delegate }
    }
}

impl Sink for ForwardingSink {
    fn write(&mut self, source: &mut Buffer, byte_count: i64) -> io::Result<()> {
        self.delegate.write(source, byte_count)
    }
}

pub struct FaultyFileSystem {
    inner: ForwardingFileSystem,
    write_faults: Arc<Mutex<HashSet<Path>>>,
    delete_faults: Arc<Mutex<HashSet<Path>>>,
    rename_faults: Arc<Mutex<HashSet<Path>>>,
}

impl FaultyFileSystem {
    pub fn new(delegate: Option<Arc<dyn FileSystem>>) -> Self {
        let delegate = delegate.expect("delegate must not be null");
        Self {
            inner: ForwardingFileSystem::new(delegate),
            write_faults: Arc::new(Mutex::new(HashSet::new())),
            delete_faults: Arc::new(Mutex::new(HashSet::new())),
            rename_faults: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn set_faulty_write(&self, file: Path, faulty: bool) {
        let mut faults = self.write_faults.lock().unwrap();
        if faulty {
            faults.insert(file);
        } else {
            faults.remove(&file);
        }
    }

    pub fn set_faulty_delete(&self, file: Path, faulty: bool) {
        let mut faults = self.delete_faults.lock().unwrap();
        if faulty {
            faults.insert(file);
        } else {
            faults.remove(&file);
        }
    }

    pub fn set_faulty_rename(&self, file: Path, faulty: bool) {
        let mut faults = self.rename_faults.lock().unwrap();
        if faulty {
            faults.insert(file);
        } else {
            faults.remove(&file);
        }
    }
}

impl FileSystem for FaultyFileSystem {
    fn atomic_move(&self, source: &Path, target: &Path) -> io::Result<()> {
        let faults = self.rename_faults.lock().unwrap();
        if faults.contains(source) || faults.contains(target) {
            return Err(Error::new(ErrorKind::Other, "boom!"));
        }
        self.inner.atomic_move(source, target)
    }

    fn delete(&self, path: &Path, must_exist: bool) -> io::Result<()> {
        let faults = self.delete_faults.lock().unwrap();
        if faults.contains(path) {
            return Err(Error::new(ErrorKind::Other, "boom!"));
        }
        self.inner.delete(path, must_exist)
    }

    fn delete_recursively(&self, file_or_directory: &Path, must_exist: bool) -> io::Result<()> {
        let faults = self.delete_faults.lock().unwrap();
        if faults.contains(file_or_directory) {
            return Err(Error::new(ErrorKind::Other, "boom!"));
        }
        self.inner.delete_recursively(file_or_directory, must_exist)
    }

    fn appending_sink(&self, file: &Path, must_exist: bool) -> io::Result<Box<dyn Sink>> {
        let sink = self.inner.appending_sink(file, must_exist)?;
        Ok(Box::new(FaultySink {
            inner: ForwardingSink::new(sink),
            file: file.clone(),
            write_faults: Arc::clone(&self.write_faults),
        }))
    }

    fn sink(&self, file: &Path, must_create: bool) -> io::Result<Box<dyn Sink>> {
        let sink = self.inner.sink(file, must_create)?;
        Ok(Box::new(FaultySink {
            inner: ForwardingSink::new(sink),
            file: file.clone(),
            write_faults: Arc::clone(&self.write_faults),
        }))
    }
}

struct FaultySink {
    inner: ForwardingSink,
    file: Path,
    write_faults: Arc<Mutex<HashSet<Path>>>,
}

impl Sink for FaultySink {
    fn write(&mut self, source: &mut Buffer, byte_count: i64) -> io::Result<()> {
        let faults = self.write_faults.lock().unwrap();
        if faults.contains(&self.file) {
            Err(Error::new(ErrorKind::Other, "boom!"))
        } else {
            self.inner.write(source, byte_count)
        }
    }
}
