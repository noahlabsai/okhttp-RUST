/*
 * Copyright (c) aQute SARL (2000, 2021). All Rights Reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::any::Any;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

// Mocking/Importing Gradle-like types as per the provided translation memory and context
// In a real production system, these would be imported from the project's Gradle-Rust bridge
use crate::build_logic::src::main::kotlin::JavaModules::*;
use crate::build_logic::src::main::kotlin::Osgi::*;

// BND specific imports (simulated based on Kotlin source)
mod bnd_osgi {
    use std::path::PathBuf;
    use std::io::{Result as IoResult, Write};

    pub struct Constants;
    impl Constants {
        pub const BUNDLE_SYMBOLICNAME: &'static str = "Bundle-SymbolicName";
        pub const BUNDLE_VERSION: &'static str = "Bundle-Version";
        pub const NOBUNDLES: &'static str = "-nobundles";
        pub const REPRODUCIBLE: &'static str = "-reproducible";
        pub const COMPRESSION: &'static str = "-compression";
    }

    pub struct Processor {
        pub properties: HashMap<String, Box<dyn Any>>,
    }

    impl Processor {
        pub fn new(properties: HashMap<String, Box<dyn Any>>, _ignore: bool) -> Self {
            Self { properties }
        }
        pub fn load_properties(&mut self, _file: &std::path::Path) -> &mut Self { self }
        pub fn store<W: Write>(&mut self, _writer: &mut W, _null: Option<String>) -> std::io::Result<()> { Ok(()) }
    }


    impl Builder {
        pub fn new(processor: Processor) -> Self {
            Self { ok: true }
        }
        pub fn set_properties(&mut self, _file: &std::path::Path, _dir: &std::path::Path) {}
        pub fn set_property(&mut self, _key: &str, _value: &str) {}
        pub fn is(&self, key: &str) -> bool {
            // Logic to check if property key is set to true/present
            false
        }
        pub fn get_property(&self, _key: &str) -> Option<String> { None }
        pub fn set_jar(&mut self, _jar: &mut Jar) {}
        pub fn build(&mut self) -> Jar { Jar { name: String::new(), file: PathBuf::new() } }
        pub fn get_errors(&self) -> Vec<String> { Vec::new() }
        pub fn get_warnings(&self) -> Vec<String> { Vec::new() }
    }

    impl Drop for Builder {

    }

    pub struct Jar {
        pub name: String,
        pub file: PathBuf,
    }

    impl Jar {
        pub enum Compression { STORE, DEFLATE }
        pub fn new(name: &str, file: PathBuf) -> Self {
            Self { name: name.to_string(), file }
        }

impl Default for Compression {
    fn default() -> Self {
        Compression::STORE
    }
}

pub const STORE: Compression = Compression::STORE;
pub const pub: Compression = Compression::pub;
pub const Self: Compression = Compression::Self;
        pub fn update_modified(&mut self, _time: i64, _msg: &str) {}
        pub fn write(&self, _dest: &PathBuf) -> std::io::Result<()> { Ok(()) }
    }
}

use bnd_osgi::*;
use crate::android_test::build_gradle::*;
use crate::android_test::src::androidTest::java::okhttp::android::test::sni::SniOverrideTest::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::FormBody::*;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZipEntryCompression {
    STORED,
    DEFLATED,
}

impl Default for ZipEntryCompression {
    fn default() -> Self {
        ZipEntryCompression::STORED
    }
}

pub const STORED: ZipEntryCompression = ZipEntryCompression::STORED;
pub const DEFLATED: ZipEntryCompression = ZipEntryCompression::DEFLATED;

impl BndBuildAction {
    pub fn new(
        extension: &BundleTaskExtension,
        task: &GradleJar,
        source_set: Vec<PathBuf>,
    ) -> Self {
        // Symbolic name default logic
        let base_name = &task.archive_base_name;
        let classifier = &task.archive_classifier;
        let symbolic_name = if classifier.is_empty() {
            base_name.clone()
        } else {
            format!("{}-{}", base_name, classifier)
        };

        // Version logic
        let version = task.archive_version.clone();
        let osgi_version = if version == "0" {
            "0".to_string()
        } else {
            // Simplified MavenVersion.parseMavenString(version).osGiVersion.toString()
            version 
        };

        Self {
            properties: extension.properties.clone(),
            classpath: extension.classpath.clone(),
            sourcepath: source_set,
            bundle_symbolic_name: symbolic_name,
            bundle_version: osgi_version,
            bndfile: extension.bndfile.clone(),
            bnd: extension.bnd.clone(),
            layout_project_dir: task.project_layout_dir.clone(),
            entry_compression: task.entry_compression,
            preserve_file_timestamps: task.is_preserve_file_timestamps,
        }
    }

    pub fn execute(&self, task: &mut GradleJar) -> Result<(), Box<dyn std::error::Error>> {
        let temporary_dir = &task.temporary_dir;
        let project_dir = &self.layout_project_dir;

        let mut gradle_properties = HashMap::new();
        for (k, v) in &self.properties {
            gradle_properties.insert(k.clone(), v.clone());
        }

        // Set default values if not present
        if !gradle_properties.contains_key(Constants::BUNDLE_SYMBOLICNAME) {
            gradle_properties.insert(Constants::BUNDLE_SYMBOLICNAME.to_string(), Box::new(self.bundle_symbolic_name.clone()));
        }
        if !gradle_properties.contains_key(Constants::BUNDLE_VERSION) {
            gradle_properties.insert(Constants::BUNDLE_VERSION.to_string(), Box::new(self.bundle_version.clone()));
        }

        // Builder block
        let mut builder = Builder::new(Processor::new(gradle_properties.clone(), false));
        
        let temporary_bnd_file = std::env::temp_dir().join(format!("bnd_{}.bnd", SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()));
        
        {
            let mut writer = File::create(&temporary_bnd_file)?;
            if let Some(ref bnd_file_val) = self.bndfile {
                if bnd_file_val.exists() && bnd_file_val.is_file() {
                    let mut p = Processor::new(gradle_properties.clone(), false);
                    p.load_properties(bnd_file_val).store(&mut writer, None)?;
                }
            } else {
                let bnd_val = &self.bnd;
                if !bnd_val.is_empty() {
                    // Simplified UTF8Properties logic
                    writer.write_all(bnd_val.as_bytes())?;
                }
            }
        }

        builder.set_properties(&temporary_bnd_file, project_dir);
        builder.set_property("project.output", &temporary_dir.to_string_lossy());

        if builder.is(Constants::NOBUNDLES) {
            return Ok(());
        }

        let archive_file = &task.archive_file;
        let archive_file_name = &task.archive_file_name;

        let archive_copy_file = temporary_dir.join(archive_file_name);
        std::fs::copy(archive_file, &archive_copy_file)?;

        let mut bundle_jar = Jar::new(archive_file_name, archive_copy_file);

        if builder.get_property(Constants::REPRODUCIBLE).is_none() && !self.preserve_file_timestamps {
            builder.set_property(Constants::REPRODUCIBLE, "true");
        }

        if builder.get_property(Constants::COMPRESSION).is_none() {
            let comp_val = match self.entry_compression {
                ZipEntryCompression::STORED => "STORE",
                ZipEntryCompression::DEFLATED => "DEFLATE",
            };
            builder.set_property(Constants::COMPRESSION, comp_val);
        }

        let metadata = std::fs::metadata(archive_file)?;
        let last_modified = metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as i64;
        bundle_jar.update_modified(last_modified, "time of Jar task generated jar");
        
        builder.set_jar(&mut bundle_jar);

        let valid_classpath: Vec<PathBuf> = self.classpath.iter()
            .filter(|p| p.exists() && (p.is_dir() || self.is_zip(p)))
            .cloned()
            .collect();
        
        let cp_path = valid_classpath.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>().join(std::env::joiners::JOINER);
        builder.set_property("project.buildpath", &cp_path);
        // builder.set_classpath(valid_classpath) - simulated

        let valid_sourcepath: Vec<PathBuf> = self.sourcepath.iter()
            .filter(|p| p.exists())
            .cloned()
            .collect();
        
        let sp_path = valid_sourcepath.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>().join(std::env::joiners::JOINER);
        builder.set_property("project.sourcepath", &sp_path);
        // builder.set_sourcepath(valid_sourcepath) - simulated

        let built_jar = builder.build();
        if !builder.ok {
            for err in builder.get_errors() {
                eprintln!("Error: {}", err);
            }
            for warn in builder.get_warnings() {
                eprintln!("Warning: {}", warn);
            }
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Bundle {} has errors", archive_file_name))));
        }

        built_jar.write(archive_file)?;
        
        // Set last modified to now
        let now = SystemTime::now();
        File::set_times(archive_file, now, now)?;

        Ok(())
    }

    fn is_zip(&self, file: &Path) -> bool {
        // In Rust, we'd use the `zip` crate to check if it's a valid zip
        // Simulating the try-catch ZipFile(file).close()
        match File::open(file) {
            Ok(_) => true, // Simplified: assume if it opens it might be a zip
            Err(_) => false,
        }
    }

    pub fn install_workaround(project: &Project) -> Rc<SourceSet> {
        let source_sets = project.source_sets();
        if let Some(existing_main) = source_sets.get_by_name("main") {
            return existing_main;
        }

        let jvm_main = source_sets.get_by_name("jvmMain")
            .expect("jvmMain source set not found");

        // In Rust, we cannot create an anonymous object implementing a trait with delegation 
        // as easily as Kotlin's `by` keyword. We create a wrapper struct.
        let main_source_set = Rc::new(FakeMainSourceSet {
            inner: jvm_main.clone(),
        });

        // source_sets.add(main_source_set.clone());
        // project.tasks.named(...).configure_each(...)
        
        main_source_set
    }
}

struct FakeMainSourceSet {
    inner: Rc<SourceSet>,
}

impl SourceSet for FakeMainSourceSet {
    fn get_name(&self) -> String { "main".to_string() }
    fn get_process_resources_task_name(&self) -> String {
        format!("{}ForFakeMain", self.inner.get_process_resources_task_name())
    }
    fn get_compile_java_task_name(&self) -> String {
        format!("{}ForFakeMain", self.inner.get_compile_java_task_name())
    }
    fn get_classes_task_name(&self) -> String {
        format!("{}ForFakeMain", self.inner.get_classes_task_name())
    }
    fn get_compile_only_configuration_name(&self) -> String {
        format!("{}ForFakeMain", self.inner.get_compile_only_configuration_name())
    }
    fn get_compile_classpath_configuration_name(&self) -> String {
        format!("{}ForFakeMain", self.inner.get_compile_classpath_configuration_name())
    }
    fn get_implementation_configuration_name(&self) -> String {
        format!("{}ForFakeMain", self.inner.get_implementation_configuration_name())
    }
    fn get_annotation_processor_configuration_name(&self) -> String {
        format!("{}ForFakeMain", self.inner.get_annotation_processor_configuration_name())
    }
    fn get_runtime_classpath_configuration_name(&self) -> String {
        format!("{}ForFakeMain", self.inner.get_runtime_classpath_configuration_name())
    }
    fn get_runtime_only_configuration_name(&self) -> String {
        format!("{}ForFakeMain", self.inner.get_runtime_only_configuration_name())
    }
    fn get_task_name(&self, verb: Option<&str>, target: Option<&str>) -> String {
        format!("{}ForFakeMain", self.inner.get_task_name(verb, target))
    }
}