# rust_code_obfuscator_core

> Core engine for the [Rustfuscator](https://lib.rs/crates/rust_code_obfuscator) — a control flow and syntax obfuscation tool for Rust codebases.

---

## 🧠 What is `rust_code_obfuscator_core`?

This crate is the **core logic** behind [Rustfuscator], a Rust code obfuscation tool that transforms readable Rust code into functionally equivalent but *harder-to-analyze* and *harder-to-reverse-engineer* output.

It is **not** a standalone crate — it powers the procedural macros and CLI-level abstractions of the [Rustfuscator project](https://lib.rs/crates/rust_code_obfuscator).
