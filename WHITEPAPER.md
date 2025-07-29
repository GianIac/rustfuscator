# Rustfuscator: A Modular Framework for Rust Code Obfuscation

## Table of Contents

1. [Introduction](#1-introduction)
2. [Goals & Scope](#2-goals--scope)
3. [Architecture Overview](#3-architecture-overview)
4. [Obfuscation Techniques](#4-obfuscation-techniques)
5. [Configuration Format](#5-configuration-format)
6. [CLI Usage](#6-cli-usage)
7. [Threat Model](#7-threat-model)
8. [Benchmarks](#8-benchmarks)
10. [License](#9-license)
11. [References](#10-references)

---

## 1. Introduction

**Rustfuscator** is a modular, developer-friendly toolchain for obfuscating Rust code at compile-time. Unlike traditional binary-level obfuscation tools, Rustfuscator operates directly on **Rust source code**, transforming functions and literals into semantically equivalent but syntactically obscure versions using procedural macros and AST rewriting.

Its primary audience includes:

* Developers wishing to protect sensitive logic
* Security researchers interested in obfuscation tooling
* Companies aiming to reduce reverse engineering risk in closed-source Rust binaries

Rustfuscator integrates cleanly with the Cargo toolchain, allowing both macro-based and CLI-based workflows.

---

## 2. Goals & Scope

### Primary Goals

* **Source-level protection**: Transform Rust source code into harder-to-analyze equivalents
* **Configurable security**: Provide opt-in transformations (string encryption, control flow, renaming)
* **Macro + CLI UX**: Allow local and batch obfuscation via Rust macros or the CLI
* **Tooling integration**: Work well with rustfmt, Git, CI/CD pipelines, and `cargo` commands
* **Transparency**: Be fully auditable and open-source

### Out-of-scope (Current)

* Obfuscation during runtime execution (but not now for the moment)
* Anti-debugging or anti-VM techniques
* Post-compilation tamper detection

---

## 3. Architecture Overview

### High-Level View

```
Rustfuscator (Repo Root)
├── rust_code_obfuscator             # Library crate: entrypoint macro layer
│   └── src/lib.rs                   # Re-exports derive + core macros
├── rust_code_obfuscator_core       # Engine crate: logic, crypto, macros
│   └── src/{crypto, obfuscator.rs ...} # Macro internals, AES logic, flow injection
├── obfuscator_derive               # #[derive(Obfuscate)] support
│   └── src/lib.rs                  # Generates wrapper types for encrypted structs
├── obfuscator_cli                  # Command-line interface for batch obfuscation
│   └── src/{cli.rs, processor.rs ...}  # Config loader, file traversal, code rewriting
├── examples/                       # Basic usage demonstrations
└── .obfuscate.toml                 # Generated config file
```

### Interaction Summary

* `rust_code_obfuscator`: User-facing macros
* `rust_code_obfuscator_core`: Implements `obfuscate_string!`, `obfuscate_flow!`, etc.
* `obfuscator_derive`: Custom `#[derive(Obfuscate)]` for struct literal encryption
* `obfuscator_cli`: Allows project-wide transformations, file scanning, and formatting

---

## 4. Obfuscation Techniques

### 4.0 Internal Strategy: What Uses `cryptify`, What Doesn’t

Rustfuscator is built on a **hybrid model**: some transformations rely directly on the [`cryptify`](https://crates.io/crates/cryptify) library, while others are implemented natively in the core engine. This division ensures strong cryptographic handling where needed and full control over macros, naming, and CLI behavior elsewhere.

#### ✅ Powered by `cryptify`

* `obfuscate_string!`: wraps and encrypts string literals via `cryptify::encrypted_string!`
* `obfuscate_flow!`: injects opaque branches and junk code via `cryptify::flow::inject`

#### 🧠 Implemented in Rustfuscator Core

* `#[derive(Obfuscate)]`: macro expansion into encrypted struct fields
* CLI file scanning and rewrite engine
* Identifier renaming logic
* TOML configuration loader and application

This layered model means Rustfuscator can build upon proven cryptographic foundations (`cryptify`) while maintaining fine-grained control over usability, extensibility, and automation.

### 4.1 String Encryption (AES-GCM)

* Encrypts string literals at compile-time via `aes-gcm`
* AES key is statically embedded via `AES_KEY`
* Generates `(Vec<u8>, [u8; 12])` ciphertexts with a random nonce
* `obfuscate_string!(...)` expands to encrypted buffers with runtime decryption

> 🔐 **Note:** The AES-based string encryption was originally implemented as a challenge module to explore compile-time literal protection in a pure-Rust environment. While it provides basic confidentiality against static string scans, it is **not** meant to replace strong runtime encryption or secure key handling. In most real-world reverse engineering scenarios, a well-crafted control-flow obfuscation often contributes more to security than static encryption alone.
>
> This functionality is powered by the excellent [`cryptify`](https://crates.io/crates/cryptify) library, which handles both literal encryption and flow injection. We consider `cryptify` a reliable and high-quality dependency that balances ergonomics and obfuscation power.
>
> Future improvements may include **key splitting**, **dynamic key derivation**, or integration with environment-based secrets. However, the architectural **core of Rustfuscator remains in its CLI-driven, macro-enabled obfuscation passes**, particularly those related to **string rewriting and control-flow injection** using `cryptify` as the backend.

```rust
let msg = obfuscate_string!("Sensitive Info");
```

### 4.2 Control-flow Injection

* Injects misleading instructions via `obfuscate_flow!()` macro
* Adds junk branches inside `if`, `match`, `loop`, and `while`
* Aims to confuse static analysis tools or pattern matchers

```rust
if x > 0 {
    obfuscate_flow!();
    do_something();
}
```

### 4.3 Identifier Renaming

* Controlled via TOML config
* Appends numeric/random suffixes to function and variable names
* Preserves user-defined identifiers via whitelist (e.g., `main`, `entry_point`)

### 4.4 Struct Encryption with `#[derive(Obfuscate)]`

Expands structs into encrypted versions:

```rust
#[derive(Obfuscate)]
struct Credentials {
    username: String,
    pin: u32,
}
```

Expands to:

```rust
struct ObfuscatedCredentials {
    username: (Vec<u8>, [u8; 12]),
    pin: (Vec<u8>, [u8; 12]),
}

impl ObfuscatedCredentials {
    pub fn new_clear(...) -> Self { ... }
    pub fn get_clear(&self) -> Credentials { ... }
}
```

---

## 5. Configuration Format

Generated by:

```bash
rustfuscator --init --input .
```

Result:

```toml
[obfuscation]
strings = true
control_flow = true
min_string_length = 4
ignore_strings = ["DEBUG"]
skip_attributes = true

[identifiers]
rename = true
preserve = ["main"]

[include]
files = ["**/*.rs"]
exclude = ["target/**"]
```

This configuration format is modular, declarative, and easily versionable across CI/CD pipelines.

---

## 6. CLI Usage

The **CLI** is the operational heart of Rustfuscator, enabling scalable and repeatable obfuscation across entire codebases.

### 🛠 Basic Command

```bash
rustfuscator --input ./src --output ./obf_src --format
```

### 📋 Options Summary

* `--input`: Path to source file/folder
* `--output`: Where transformed files are written
* `--format`: Runs `rustfmt` on output
* `--init`: Generates `.obfuscate.toml` in input directory
* `--as-project`: Performs deep copy and patches Cargo.toml
* `--json`: Outputs JSON versions of transformed files

### 🔄 Example Workflow

```bash
rustfuscator --init --input ./mycrate
rustfuscator --input ./mycrate --output ./mycrate_obf --as-project --format
```

* The CLI scans for `.rs` files using `walkdir`
* Applies obfuscation transformations based on the `.obfuscate.toml`
* Renames identifiers, rewrites control flow, and encrypts literals
* Outputs fully usable and buildable Rust source code

### ✅ Value Proposition

* Fast, zero-dependency CLI
* Rustfmt integration preserves developer experience
* Obfuscation is reproducible and configurable
* Perfect for automation pipelines or obfuscation-as-a-step in CI

---

## 7. Threat Model

### Assumptions

* Attacker has access to compiled binary
* May use tools like `objdump`, Ghidra, or decompilers

### Provided Protections

* Obfuscated control flow frustrates basic symbolic reasoning
* Encrypted string literals break string-matching heuristics
* Identifier renaming obscures semantic meaning

### Limitations

* AES key is static (can be extracted with effort)
* No runtime behavior modification
* No memory-level or control-flow graph integrity protections (yet)

---

## 8. Benchmarks

| Feature             | Overhead                | Runtime overhead            |
| ------------------- | ----------------------- |-----------------------------|
| String encryption   | +3–10% binary size      | ~1–8% (depending on usage)  |
| Control-flow inject | \~0–2% runtime slowdown | ~0–2%                       |
| Identifier renaming | Build-time only         |  0%                         |

System: x86\_64 Linux / Rust 1.76.0, built in release mode with default config.
### Benchmarks notes 
- String encryption is performed at compile-time; strings are stored encrypted in the binary and decrypted inline at runtime. No plaintext strings are embedded in the executable.
- Currently, each use of obfuscate_string!() performs decryption on the fly. This ensures strings aren't kept in memory unnecessarily, at the cost of increased CPU usage when used frequently.
- Caching is not yet implemented, but we're actively working on a mechanism to optionally cache decrypted strings using once_cell or equivalent. In the future, this may be configurable based on user preference (performance vs. runtime secrecy).

---

## 9. License

MIT License
© 2025 Gianfranco Iaculo

---

## 10. References

* [https://github.com/GianIac/rustfuscator](https://github.com/GianIac/rustfuscator)
* https://crates.io/crates/rust_code_obfuscator
* [https://lib.rs/crates/rust\_code\_obfuscator](https://lib.rs/crates/rust_code_obfuscator)
* https://www.cs.auckland.ac.nz/~cthombor/Pubs/01027797a.pdf
