# rust_code_obfuscator_core

[![crates.io](https://img.shields.io/crates/v/rust_code_obfuscator.svg)](https://crates.io/crates/rust_code_obfuscator)
[![lib.rs](https://img.shields.io/badge/lib.rs-rust_code_obfuscator-orange?logo=rust)](https://lib.rs/crates/rust_code_obfuscator)
[![docs.rs](https://img.shields.io/docsrs/rust_code_obfuscator)](https://docs.rs/rust_code_obfuscator)

> Core engine for the [Rustfuscator](https://github.com/gianiac/rustfuscator) — a control flow and syntax obfuscation tool for Rust codebases.

---

## 🧠 What is `rust_code_obfuscator_core`?

This crate is the **core logic** behind [Rustfuscator](https://github.com/gianiac/rustfuscator), a Rust code obfuscation tool that transforms readable Rust code into functionally equivalent but *harder-to-analyze* and *harder-to-reverse-engineer* output.

It is **not** a standalone crate — it powers the procedural macros and CLI-level abstractions of the [Rustfuscator project](https://github.com/gianiac/rustfuscator).
