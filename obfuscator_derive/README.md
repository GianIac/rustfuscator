# obfuscator_derive

> Procedural macro definitions for the [Rustfuscator](https://lib.rs/crates/rust_code_obfuscator) — a Rust obfuscation framework for control flow, syntax, and literal protection.

---

## 🧠 What is `obfuscator_derive`?

This crate provides the **procedural macros** used by [`rust_code_obfuscator`](https://lib.rs/crates/rust_code_obfuscator).  
It defines attribute macros like `#[obfuscate]` that mark functions or modules for transformation during compilation.

Internally, it delegates obfuscation logic to [`rust_code_obfuscator_core`](https://lib.rs/crates/rust_code_obfuscator_core).

> ⚠️ This crate is not meant to be used directly. Use [`rust_code_obfuscator`](https://lib.rs/crates/rust_code_obfuscator) unless you're developing internals or writing custom tooling.
