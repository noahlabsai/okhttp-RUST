use std::collections::HashMap;
use std::path::PathBuf;
use std::any::Any;
use crate::android_test::build_gradle::*;

/*
 * Copyright (C) 2025 Square, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *    https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#[derive(Debug, Clone, PartialEq)]
pub struct MultiReleaseExtension {
    pub target_versions: Vec<i32>,
}

impl MultiReleaseExtension {
    pub fn target_versions(&mut self, versions: Vec<i32>) {
        self.target_versions = versions;
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct JavaCompile {
    pub options: CompileOptions,
    pub modularity: Modularity,
    pub java_compiler: Option<JavaCompiler>,
    pub classpath: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Modularity {
    pub infer_module_path: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JavaCompiler {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KotlinJvmCompile {
    pub destination_directory: PathBuf,
    pub libraries: Vec<String>,
    pub sources: Vec<PathBuf>,
}

impl KotlinJvmCompile {
    pub fn source(&mut self, path: &str) {
        self.sources.push(PathBuf::from(path));
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JavaToolchainService {
    pub service_id: String,
}

impl JavaToolchainService {
    pub fn compiler_for(&self, toolchain: &Toolchain) -> JavaCompiler {
        JavaCompiler {
            name: format!("Compiler for {:?}", toolchain),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Toolchain {
    pub version: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JavaPluginExtension {
    pub toolchain: Toolchain,
}

pub struct Project {
    pub plugins: PluginManager,
    pub extensions: ExtensionManager,
    pub tasks: TaskManager,
}

impl Project {
    pub fn file(&self, path: &str) -> PathBuf {
        PathBuf::from(path)
    }
}

pub struct PluginManager {
    pub applied_plugins: Vec<String>,
}

impl PluginManager {
    pub fn apply(&mut self, plugin_id: &str) {
        self.applied_plugins.push(plugin_id.to_string());
    }
}

pub struct ExtensionManager {
    pub extensions: HashMap<String, Box<dyn Any>>,
}

impl ExtensionManager {
    pub fn get_by_type<T: 'static>(&self) -> Option<&T> {
        // In a real implementation, this would look up the type in the map
        None
    }
}

pub struct TaskManager {
    pub tasks: HashMap<String, Box<dyn Any>>,
}

impl TaskManager {
    pub fn named<T: 'static>(&self, name: &str) -> Option<&T> {
        self.tasks.get(name)?.downcast_ref::<T>()
    }
    pub fn get_by_name(&self, name: &str) -> Option<&Box<dyn Any>> {
        self.tasks.get(name)
    }
}

pub fn apply_java_modules(
    project: &mut Project,
    module_name: String,
    default_version: i32,
    java_module_version: i32,
    enable_validation: bool,
) {
    project.plugins.apply("me.champeau.mrjar");

    if let Some(mr_ext) = project.extensions.get_by_type::<MultiReleaseExtension>() {
        // In a real Gradle-like system, we would mutate the extension in place.
        // Since we are mocking, we simulate the call.
        let mut mr_ext_clone = MultiReleaseExtension {
            target_versions: vec![default_version, java_module_version],
        };
        mr_ext_clone.target_versions(vec![default_version, java_module_version]);
    }

    if let Some(java_compile) = project.tasks.named::<JavaCompile>("compileJava9Java") {
        let mut java_compile_mut = java_compile.clone();

        if let Some(kotlin_task_any) = project.tasks.get_by_name("compileKotlin") {
            if let Some(compile_kotlin_task) = kotlin_task_any.downcast_ref::<KotlinJvmCompile>() {
                let mut kotlin_task_mut = compile_kotlin_task.clone();

                if enable_validation {
                    kotlin_task_mut.source("src/main/java9");
                }

                java_compile_mut.options.compiler_args.push("-Xlint:-requires-transitive-automatic".to_string());

                let patch_arg = format!("{}={}", module_name, kotlin_task_mut.destination_directory.display());
                java_compile_mut.options.compiler_args.push("--patch-module".to_string());
                java_compile_mut.options.compiler_args.push(patch_arg);

                java_compile_mut.classpath = kotlin_task_mut.libraries.clone();
                java_compile_mut.modularity.infer_module_path = true;

                if let (Some(toolchains), Some(java_ext)) = (
                    project.extensions.get_by_type::<JavaToolchainService>(),
                    project.extensions.get_by_type::<JavaPluginExtension>(),
                ) {
                    java_compile_mut.java_compiler = Some(toolchains.compiler_for(&java_ext.toolchain));
                }
            }
        }
    }
}
