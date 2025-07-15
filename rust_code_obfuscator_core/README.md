# rust_code_obfuscator_core

[![crates.io](https://img.shields.io/crates/v/rust_code_obfuscator.svg)](https://crates.io/crates/rust_code_obfuscator)
[![Contributing](https://img.shields.io/badge/docs-contributing-blueviolet?logo=github)](./CONTRIBUTING.md)
[![Whitepaper](https://img.shields.io/badge/docs-whitepaper-lightgrey?logo=readthedocs)](https://github.com/GianIac/rustfuscator/blob/main/WHITEPAPER.md)
[![Obfuscation Guide](https://img.shields.io/badge/docs-obfuscation_fundamentals-blue?logo=rust)](https://gianiac.github.io/rustfuscator/obfuscation_fundamentals.html)

> Core engine for the [Rustfuscator](https://github.com/gianiac/rustfuscator) — a control flow and syntax obfuscation tool for Rust codebases.

---

## 🧠 What is `rust_code_obfuscator_core`?

This crate is the **core logic** behind [Rustfuscator](https://github.com/gianiac/rustfuscator), a Rust code obfuscation tool that transforms readable Rust code into functionally equivalent but *harder-to-analyze* and *harder-to-reverse-engineer* output.

It is **not** a standalone crate — it powers the procedural macros and CLI-level abstractions of the [Rustfuscator project](https://github.com/gianiac/rustfuscator).
