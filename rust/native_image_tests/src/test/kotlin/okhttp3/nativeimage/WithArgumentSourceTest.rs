/*
 * Copyright (C) 2020 Square, Inc.
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

// Mocking JUnit 5 / Parameterized Test infrastructure as it doesn't exist natively in Rust.
// In a real Rust project, one would use `test-case` or `parameterized` crates.

pub struct ParameterDeclarations;
pub struct ExtensionContext;

#[derive(Debug, Clone, PartialEq)]
pub struct Arguments(pub Vec<Box<dyn Any>>);

impl Arguments {
    pub fn of<T: 'static>(val: T) -> Self {
        Arguments(vec![Box::new(val)])
    }
}

pub trait ArgumentsProvider {
    fn provide_arguments(
        &self,
        parameters: Option<&ParameterDeclarations>,
        context: Option<&ExtensionContext>,
    ) -> Vec<Arguments>;
}

// This enforces us having the params classes on the classpath to workaround
// https://github.com/graalvm/native-build-tools/issues/745
pub struct WithArgumentSourceTest;

impl WithArgumentSourceTest {
    // Equivalent to @ParameterizedTest @ArgumentsSource(FakeArgumentsProvider::class)
    pub fn passing_test(&self, value: i32) {
        // assertk.assertThat(value).isGreaterThan(0)
        assert!(value > 0, "Expected value to be greater than 0, but was {}", value);
    }

    // Simulation of the JUnit runner executing the parameterized test
    pub fn run_passing_test_parameterized(&self) {
        let provider = FakeArgumentsProvider;
        let args_list = provider.provide_arguments(None, None);
        
        for args in args_list {
            if let Some(val_any) = args.0.get(0) {
                if let Some(value) = val_any.downcast_ref::<i32>() {
                    self.passing_test(*value);
                }
            }
        }
    }
}

pub struct FakeArgumentsProvider;

impl ArgumentsProvider for FakeArgumentsProvider {
    fn provide_arguments(
        &self,
        _parameters: Option<&ParameterDeclarations>,
        _context: Option<&ExtensionContext>,
    ) -> Vec<Arguments> {
        // listOf(Arguments.of(1), Arguments.of(2)).stream()
        vec![
            Arguments::of(1),
            Arguments::of(2),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

    #[test]
    fn test_with_argument_source() {
        let test_suite = WithArgumentSourceTest;
        test_suite.run_passing_test_parameterized();
    }
}
}
