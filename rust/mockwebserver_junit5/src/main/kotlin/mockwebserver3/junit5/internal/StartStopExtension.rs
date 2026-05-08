/*
 * Copyright (C) 2025 Square, Inc.
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
use std::sync::{Arc, Mutex};

// IMPORT PATHS
use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockWebServer;
use crate::mockwebserver_junit5::src::main::kotlin::mockwebserver3::junit5::StartStop;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::SuppressSignatureCheck;

/// Mocking JUnit 5 Extension interfaces as traits since they are external Java APIs.
pub trait BeforeAllCallback {
    fn before_all(&self, context: &ExtensionContext);
}

pub trait BeforeEachCallback {
    fn before_each(&self, context: &ExtensionContext);
}

/// Mocking JUnit 5 ExtensionContext and its associated types.
pub struct ExtensionContext {
    pub test_class: Option<Box<dyn Any>>,
    pub test_instance: Option<Box<dyn Any>>,
}

impl ExtensionContext {
    pub fn get_store(&self, _namespace: Namespace) -> Arc<Mutex<Store>> {
        // In a real JUnit 5 environment, this would return a store associated with the context.
        Arc::new(Mutex::new(Store::new()))
    }

    pub fn required_test_class(&self) -> &dyn Any {
        self.test_class.as_ref().expect("Required test class not found").as_ref()
    }

    pub fn test_instance(&self) -> Option<&dyn Any> {
        self.test_instance.as_ref().map(|i| i.as_ref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Namespace(pub String);

impl Namespace {
    pub fn create<T: 'static>() -> Self {
        Namespace(std::any::type_name::<T>().to_string())
    }
}

pub struct Store {
    data: HashMap<String, Box<dyn Any>>,
}

impl Store {
    fn new() -> Self {
        Store {
            data: HashMap::new(),
        }
    }

    pub fn put<K: std::fmt::Debug, V: 'static>(&mut self, key: &K, value: V) {
        self.data.insert(format!("{:?}", key), Box::new(value));
    }
}

/// Mocking the reflection utility `findAnnotatedFields`.
/// In Rust, reflection is not available at runtime in the same way as Java.
/// This implementation simulates the behavior of finding fields annotated with `StartStop`.
fn find_annotated_fields<T: 'static, A: 'static>(
    _class: &dyn Any,
    _annotation: std::any::TypeId,
    _filter: impl Fn(&Field) -> bool,
) -> Vec<Field> {
    // This is a mock implementation. In a real scenario, this would involve 
    // a registry of fields or a procedural macro generated metadata.
    Vec::new()
}

pub struct Field {
    pub name: String,
    pub is_static: bool,
    pub modifiers: i32,
}

impl Field {
    pub fn set_accessible(&mut self, _accessible: bool) {}
    pub fn get<T: 'static>(&self, _instance: Option<&dyn Any>) -> Option<T> {
        None
    }
}

/// Implements the policy specified by [StartStop].
#[derive(Debug, Clone, PartialEq)]
pub struct StartStopExtension;

impl BeforeAllCallback for StartStopExtension {
    fn before_all(&self, context: &ExtensionContext) {
        let store_mutex = context.get_store(Namespace::create::<StartStop>());
        let mut store = store_mutex.lock().unwrap();

        let static_fields = find_annotated_fields(
            context.required_test_class(),
            std::any::TypeId::of::<StartStop>(),
            |field| field.is_static,
        );

        for mut field in static_fields {
            field.set_accessible(true);
            if let Some(server) = field.get::<MockWebServer>(None) {
                // Put the instance in the store, so JUnit closes it for us in afterAll.
                store.put(&field.name, server.clone());
                
                // MockWebServer::start() is called.
                let _ = server.start(); 
            }
        }
    }
}

impl BeforeEachCallback for StartStopExtension {
    fn before_each(&self, context: &ExtensionContext) {
        let test_instance = context.test_instance();
        let store_mutex = context.get_store(Namespace::create::<StartStop>());
        let mut store = store_mutex.lock().unwrap();

        let instance_fields = find_annotated_fields(
            context.required_test_class(),
            std::any::TypeId::of::<StartStop>(),
            |field| !field.is_static,
        );

        for mut field in instance_fields {
            field.set_accessible(true);
            if let Some(server) = field.get::<MockWebServer>(test_instance) {
                // Put the instance in the store, so JUnit closes it for us in afterEach.
                store.put(&field.name, server.clone());

                let _ = server.start();
            }
        }
    }
}
