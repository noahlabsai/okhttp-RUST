//! Translated Kotlin/JVM sources from OkHttp live under `src/` using the original Gradle module layout
//! (e.g. `jvmMain`, `androidMain`). Integrating them into a single Rust module tree is ongoing.
//!
//! This file exists so the workspace crate builds cleanly with `cargo check`.

#![allow(dead_code, unused)]

/// Crate marker for workspace checks (`cargo check -p native-image-tests`).
pub const NATIVE_IMAGE_TESTS: &str = env!("CARGO_PKG_NAME");
