# obfuscator_derive

> Procedural macro definitions for the [Rustfuscator](https://github.com/gianiac/rustfuscator) — a Rust obfuscation framework for control flow, syntax, and literal protection.

---

## 🧠 What is `obfuscator_derive`?

This crate provides the **procedural macros** used by [Rustfuscator](https://github.com/gianiac/rustfuscator).  
It defines attribute macros like `#[obfuscate]` that mark functions or modules for transformation during compilation.

> ⚠️ This crate is not meant to be used directly. Use [`rust_code_obfuscator`](https://github.com/gianiac/rustfuscator)) unless you're developing internals or writing custom tooling.
