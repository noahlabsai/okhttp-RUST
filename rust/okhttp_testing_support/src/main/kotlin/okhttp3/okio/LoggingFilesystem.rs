use okio::{FileSystem, Path, Sink, Source};
use std::sync::Arc;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::UrlComponentEncodingTester::*;
use crate::okhttp::src::jvmTest::kotlin::okhttp3::internal::io::FaultyFileSystem::*;
use crate::okhttp_testing_support::src::main::kotlin::okhttp3::TestUtilJvm::*;

// LoggingFilesystem is a decorator for FileSystem that logs every operation to stdout.
// In Kotlin, this extends ForwardingFileSystem. In Rust, we use a composition pattern
// with an Arc<dyn FileSystem> to mimic the delegation behavior.
#[derive(Debug, Clone)]
pub struct LoggingFilesystem {
    file_system: Arc<dyn FileSystem>,
}

impl LoggingFilesystem {
    pub fn new(file_system: Arc<dyn FileSystem>) -> Self {
        Self { file_system }
    }

    pub fn log(&self, line: &str) {
        println!("{}", line);
    }
}

impl FileSystem for LoggingFilesystem {
    fn appending_sink(&self, path: &Path, must_exist: bool) -> std::io::Result<Sink> {
        self.log(&format!("appendingSink({})", path));
        self.file_system.appending_sink(path, must_exist)
    }

    fn atomic_move(&self, source: &Path, target: &Path) -> std::io::Result<()> {
        self.log(&format!("atomicMove({}, {})", source, target));
        self.file_system.atomic_move(source, target)
    }

    fn create_directory(&self, dir: &Path, must_create: bool) -> std::io::Result<()> {
        self.log(&format!("createDirectory({})", dir));
        self.file_system.create_directory(dir, must_create)
    }

    fn delete(&self, path: &Path, must_exist: bool) -> std::io::Result<()> {
        self.log(&format!("delete({})", path));
        self.file_system.delete(path, must_exist)
    }

    fn sink(&self, path: &Path, must_create: bool) -> std::io::Result<Sink> {
        self.log(&format!("sink({})", path));
        self.file_system.sink(path, must_create)
    }

    fn source(&self, path: &Path) -> std::io::Result<Source> {
        self.log(&format!("source({})", path));
        self.file_system.source(path)
    }
}
