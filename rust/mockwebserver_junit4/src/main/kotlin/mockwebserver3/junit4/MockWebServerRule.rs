use std::sync::Arc;
use crate::mockwebserver::src::main::kotlin::mockwebserver3::MockWebServer;
use crate::mockwebserver::src::test::java::mockwebserver3::CustomDispatcherTest::*;

// Trait representing the JUnit 4 ExternalResource behavior.
// In Rust, this is typically handled by a setup/teardown pattern in test frameworks.
pub trait ExternalResource {
    fn before(&self);
    fn after(&self);
}

/*
 * Runs MockWebServer for the duration of a single test method.
 *
 * In Java JUnit 4 tests (ie. tests annotated `@org.junit.Test`), use this by defining a field with
 * the `@Rule` annotation:
 *
 *