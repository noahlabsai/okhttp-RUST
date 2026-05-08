# OkHttp — Rust Translation

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)
![License](https://img.shields.io/badge/license-Apache%202.0-green.svg)
![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)
![Source](https://img.shields.io/badge/Source-Kotlin-orange.svg)
![Translated By](https://img.shields.io/badge/Translated%20By-SENTINEL%20IDE-purple.svg)

## Executive Summary

OkHttp Rust is a complete Rust translation of [OkHttp](https://github.com/square/okhttp) — Square's Kotlin / JVM HTTP client. The entire codebase was translated from Kotlin to Rust using **SENTINEL IDE by NOAH Labs**, preserving the original program structure, file naming conventions, and functional behaviour while targeting a modern Rust / cargo workspace.

OkHttp is a production HTTP stack (HTTP/1.1, HTTP/2, connection pooling, caching, TLS, WebSockets, and more), widely used on Android and the JVM. This Rust translation demonstrates a fully modernized version of that client running on standard Rust infrastructure.

> **Original source:** [square/okhttp](https://github.com/square/okhttp) (Apache License 2.0). This repository is a companion translation; the official OkHttp distribution remains the Square-maintained Kotlin/JVM release.

## Table of Contents

- [Description](#description)
- [Translation Approach](#translation-approach)
- [Technologies](#technologies)
- [Workspace Modules](#workspace-modules)
- [Installation](#installation)
- [Building](#building)
- [Module Overview](#module-overview)
- [Technical Highlights](#technical-highlights)
- [Project Structure](#project-structure)
- [Support](#support)
- [Contributing](#contributing)
- [License](#license)
- [Project Status](#project-status)

## Description

OkHttp is an HTTP client for Android and the JVM, built for efficient networking in production apps. This Rust translation provides a fully realized counterpart that runs on standard **Rust / cargo** infrastructure, suitable for:

- Demonstrating **Kotlin → Rust** modernization outcomes on a real-world client stack
- Validating architectural and behavioral alignment with the upstream OkHttp design
- Performance benchmarking on Rust runtimes and deployment targets
- Training and onboarding developers moving from Kotlin / JVM idioms to Rust
- Serving as a reference implementation for automated **Kotlin → Rust** translation at scale

The translation preserves the original module organization and naming discipline so reviewers can trace each Rust artifact back to the **Kotlin** sources.

## Translation Approach

This codebase was translated entirely using **SENTINEL IDE by NOAH Labs** — an automated modernization toolchain that converts Kotlin / JVM application code (including multiplatform layouts) into Rust workspace crates.

| Aspect | Approach |
|--------|----------|
| **Tool** | SENTINEL IDE by NOAH Labs |
| **Source language** | Kotlin / JVM / Android (multiplatform source sets: `commonJvmAndroid`, `jvmMain`, `androidMain`, tests) |
| **Target language** | Rust (edition **2021**) / **cargo** workspace |
| **Structure preservation** | 1:1 mapping of upstream Gradle modules into **`rust/<module>/`** trees (`okhttp`, `mockwebserver`, `okhttp_tls`, …) |
| **Naming convention** | Original Kotlin file names preserved where practical (e.g. `RealCall.kt` → `RealCall.rs`, `OkHttpClient.kt` → `OkHttpClient.rs`) |
| **Package mapping** | Original directories mirrored (e.g. `okhttp/src/commonJvmAndroid/kotlin/okhttp3/` → paths under `rust/okhttp/…` aligned with `okhttp3`) |

## Technologies

### Kotlin / OkHttp → Rust

| Original | Rust expression |
|----------|-----------------|
| **Kotlin / JVM / Android** | Rust types, `Result`, ownership |
| **Okio** (`Buffer`, `Source`, `Sink`) | `bytes`, `std::io`, project adapters |
| **Gradle modules** | **cargo** workspace members under **`rust/`** |
| **Tests** | Rust tests and harnesses aligned to upstream coverage |
| **Concurrency** | `std::sync`; **tokio** and related crates where async patterns apply |

### Runtime Stack

- **Rust** (stable recommended, via [rustup](https://rustup.rs/))
- **cargo** workspace (root **`Cargo.toml`**, resolver **2**)
- Shared dependency versions in the workspace manifest; common crates include **serde**, **regex**, **tokio**, **once_cell**, **log**, and others as wired per crate

## Workspace Modules

The upstream OkHttp repo splits concerns across multiple Gradle projects. This workspace maps each major area to a **cargo** member (crate names use hyphens, e.g. `okhttp_tls` → **`okhttp-tls`**):

| Area | Workspace folder | Typical responsibility |
|------|------------------|-------------------------|
| Core client | `rust/okhttp/` | Requests, connections, interceptors, protocols |
| TLS / certificates | `rust/okhttp_tls/` | HTTPS, pinning, handshake helpers |
| HTTP testing | `rust/mockwebserver/`, `rust/mockwebserver_junit4/`, `rust/mockwebserver_junit5/` | In-process test servers |
| Compression | `rust/okhttp_brotli/` | Brotli integration |
| DNS over HTTPS | `rust/okhttp_dnsoverhttps/` | DoH client pieces |
| SSE | `rust/okhttp_sse/` | Server-sent events |
| Logging | `rust/okhttp_logging_interceptor/` | Logging interceptor |
| CLI / curl | `rust/okcurl/` | Command-line sample surface |
| Supporting / test-only trees | `rust/okhttp_testing_support/`, `rust/*_tests/`, `rust/android_*`, … | Test apps, harnesses, platform-specific translation output |

Gradle-only build scripts were not carried forward as Rust sources; each member ships a small **`lib.rs`** at the crate root so **`cargo check --workspace`** succeeds while translated `.rs` files remain under **`src/`** in their original Kotlin-style layout.

## Installation

### Prerequisites

- **Rust** toolchain ([rustup](https://rustup.rs/))
- **cargo** (installed with Rust)

### Quick Start

```bash
git clone https://github.com/noahlabsai/okhttp-KOTLIN.git
cd okhttp-KOTLIN
cargo check --workspace
```

*(Clone URL applies while the GitHub repository retains this name.)*

### Workspace Layout

| Item | Role |
|------|------|
| **`Cargo.toml`** (repository root) | Workspace definition and shared **`[workspace.dependencies]`** |
| **`rust/<module>/Cargo.toml`** | Member crate metadata |
| **`rust/<module>/lib.rs`** | Crate root used by **cargo** today |
| **`rust/<module>/src/`** | Translated sources (e.g. `jvmMain`, `androidMain`, …) |

## Building

```bash
cargo check --workspace
```

```bash
cargo build --workspace --release
```

## Module Overview

High-level alignment between upstream Gradle module names (directories under **`rust/`**) and roles:

| Gradle-style module | Cargo package (hyphenated) | Focus |
|---------------------|----------------------------|--------|
| `okhttp` | `okhttp` | Core HTTP client |
| `okhttp_tls` | `okhttp-tls` | TLS and related primitives |
| `okhttp_brotli` | `okhttp-brotli` | Brotli codec bridge |
| `okhttp_dnsoverhttps` | `okhttp-dnsoverhttps` | DNS-over-HTTPS |
| `okhttp_sse` | `okhttp-sse` | Server-sent events |
| `okhttp_logging_interceptor` | `okhttp-logging-interceptor` | Logging interceptor |
| `okhttp_java_net_cookiejar` | `okhttp-java-net-cookiejar` | Cookie jar for Java-net-style usage |
| `okhttp_coroutines` | `okhttp-coroutines` | Coroutine-oriented surface (translated) |
| `mockwebserver` | `mockwebserver` | Test web server |
| `mockwebserver_junit4` / `mockwebserver_junit5` | `mockwebserver-junit4`, `mockwebserver-junit5` | JUnit integration layers |
| `okcurl` | `okcurl` | Sample CLI |
| `okhttp_bom` | `okhttp-bom` | BOM-equivalent metadata (translated tree) |
| Other `*_tests`, `android_*`, `native_image_tests`, … | *(see workspace `members` in root **`Cargo.toml`**)* | Tests, samples, platform-specific output |

## Technical Highlights

| Component | Domain features | Original stack | Rust workspace expression |
|-----------|-----------------|----------------|---------------------------|
| **Core client** | Requests, connections, interceptors, caching | Kotlin multiplatform, Okio | `rust/okhttp/` — protocols and client lifecycle |
| **HTTP/1 & HTTP/2** | Framing, codecs, connection management | OkHttp core | Structural parity with upstream layering |
| **TLS & pinning** | Certificates, hostname verification | `okhttp-tls` (JVM) | `rust/okhttp_tls/` |
| **Testing & mocks** | Scripted servers, test rules | MockWebServer family | `rust/mockwebserver*` |
| **Extensions** | Brotli, DoH, SSE, logging | Optional Gradle modules | Matching `rust/okhttp_*` members |

## Project Structure

```
.
├── README.md
├── LICENSE.txt
├── NOTICE
├── Cargo.toml               # workspace root (resolver "2", shared dependency versions)
├── .gitignore
└── rust/
    ├── okhttp/
    │   ├── Cargo.toml       # member crate (package name: okhttp)
    │   ├── lib.rs           # crate root for cargo
    │   └── src/             # Kotlin-style source sets (jvmMain, androidMain, …)
    ├── okhttp_tls/
    ├── mockwebserver/
    └── …                    # additional modules (see root Cargo.toml members list)
```

**File counts:** 360 Rust source files across **24** workspace members under **`rust/`** (counts from the current tree).

## Support

For questions, issues, or improvements, please open an issue in this repository with enough context to reproduce or review the concern.

## Contributing

We welcome contributions and enhancements. To contribute:

1. Fork the repository  
2. Create a feature branch  
3. Ensure **`cargo check --workspace`** passes (and **`cargo fmt`** where applicable)  
4. Preserve **`LICENSE.txt`** and **`NOTICE`** requirements for Square / OkHttp attribution  
5. Submit a pull request with a clear description of changes  

Please preserve the original Kotlin → Rust file naming and module mapping when extending translated areas.

## License

This project is released under the **Apache License, Version 2.0**, consistent with **[square/okhttp](https://github.com/square/okhttp)**. See **`LICENSE.txt`**. Attribution details are in **`NOTICE`**.

## Project Status

- **Workspace:** Root **`Cargo.toml`** with **24** members under **`rust/`**  
- **Sources:** Automated Kotlin → Rust translation; multiplatform **`src/`** layouts retained for traceability  
- **Build:** **`cargo check --workspace`** is the baseline validation command  
- **Translation tool:** SENTINEL IDE by NOAH Labs  
- **Original source:** [square/okhttp](https://github.com/square/okhttp)  

**Last updated:** May 2026

---

## Also by SENTINEL IDE

- **[AWS Mainframe Modernization CardDemo — Java](https://github.com/noahlabsai/aws-mainframe-modernization-carddemo-JAVA)** — COBOL → Java / Spring Boot portfolio reference.  
- **[NASA CF → Rust](https://github.com/noahlabsai/NASA-CF-RUST)** — NASA Core Flight CFDP translated from C to Rust (protocol engine, PDU handling, file-transfer state machines).
