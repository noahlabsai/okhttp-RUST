/*
 * Copyright (C) 2022 Square, Inc.
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

/// Mocking the GraalVM Native Image Feature API as it is a JVM-specific framework.
/// In a real Rust system, this would be replaced by the actual native image 
/// configuration or a similar trait-based plugin system.
pub trait Feature {
    fn before_analysis(&self, access: Option<&BeforeAnalysisAccess>);
}

/// Represents the access object provided by GraalVM during the beforeAnalysis phase.
#[derive(Debug, Clone, PartialEq)]
pub struct BeforeAnalysisAccess {
    // In the original JVM source, this object provides methods to register 
    // resources and reflection. We define it as a struct to maintain type fidelity.
    pub registration_id: String,
}

impl Default for BeforeAnalysisAccess {
    fn default() -> Self {
        Self {
            registration_id: String::new(),
        }
    }
}

/**
 * Automatic configuration of OkHttp for native images.
 *
 * Currently, includes all necessary resources.
 */
#[derive(Debug, Clone, PartialEq)]
pub struct OkHttpFeature;

impl OkHttpFeature {
    pub fn new() -> Self {
        Self
    }
}

impl Feature for OkHttpFeature {
    /// Implementation of the beforeAnalysis hook.
    /// The @IgnoreJRERequirement annotation is a build-time hint and does not 
    /// translate to runtime Rust logic.
    fn before_analysis(&self, _access: Option<&BeforeAnalysisAccess>) {
        // Kotlin: Unit (no-op)
    }
}