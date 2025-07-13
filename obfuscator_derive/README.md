# obfuscator_derive

[![crates.io](https://img.shields.io/crates/v/rust_code_obfuscator.svg)](https://crates.io/crates/rust_code_obfuscator)
[![lib.rs](https://img.shields.io/badge/lib.rs-rust_code_obfuscator-orange?logo=rust)](https://lib.rs/crates/rust_code_obfuscator)
[![docs.rs](https://img.shields.io/docsrs/rust_code_obfuscator)](https://docs.rs/rust_code_obfuscator)

> Procedural macro definitions for the [Rustfuscator](https://github.com/gianiac/rustfuscator) — a Rust obfuscation framework for control flow, syntax, and literal protection.

---

## 🧠 What is `obfuscator_derive`?

This crate provides the **procedural macros** used by [Rustfuscator](https://github.com/gianiac/rustfuscator).  
It defines attribute macros like `#[obfuscate]` that mark functions or modules for transformation during compilation.

> ⚠️ This crate is not meant to be used directly. Use [`rust_code_obfuscator`](https://github.com/gianiac/rustfuscator)) unless you're developing internals or writing custom tooling.
