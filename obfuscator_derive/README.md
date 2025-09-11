# obfuscator_derive

[![crates.io](https://img.shields.io/crates/v/rust_code_obfuscator.svg)](https://crates.io/crates/rust_code_obfuscator)
[![Contributing](https://img.shields.io/badge/docs-contributing-blueviolet?logo=github)](./CONTRIBUTING.md)
[![Whitepaper](https://img.shields.io/badge/docs-whitepaper-lightgrey?logo=readthedocs)](https://github.com/GianIac/rustfuscator/blob/main/WHITEPAPER.md)
[![Obfuscation Guide](https://img.shields.io/badge/docs-obfuscation_fundamentals-blue?logo=rust)](https://gianiac.github.io/rustfuscator/obfuscation_fundamentals.html)

> Procedural macro definitions for the [Rustfuscator](https://github.com/gianiac/rustfuscator) — a Rust obfuscation framework for control flow, syntax, and literal protection.

---

## 🧠 What is `obfuscator_derive`?

This crate provides the **procedural macros** used by [Rustfuscator](https://github.com/gianiac/rustfuscator).  
It defines attribute macros like `#[obfuscate]` that mark functions or modules for transformation during compilation.

> ⚠️ This crate is not meant to be used directly. Use [`rust_code_obfuscator`](https://github.com/gianiac/rustfuscator)) unless you're developing internals or writing custom tooling.

### Usage notes

- `#[derive(Obfuscate)]` supports fields of type `String` and `u32`.
- Invalid field types produce a compile-time error pointing to the offending field.
- The derive internally uses `rust_code_obfuscator::crypto::default_key()`; no user key plumbing required.
