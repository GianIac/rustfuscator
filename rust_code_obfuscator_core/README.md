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

### Note about Key management for crypto.rs

Now uses build-time key management, the key is injected via `build.rs`:
- If `OBFUSCATOR_KEY_HEX` (64 hex chars) is set, that value is used.
- Otherwise, a random 32-byte key is generated for the build.

A 256-bit AES key is provided at build time (not in source code).

- Deterministic builds:
  ```bash
  export OBFUSCATOR_KEY_HEX="00112233... (64 hex chars) ..."
  cargo build --release 
  ```
- If unset, a random key is generated per build.
- The runtime API returns Result (no unwrap() in crypto paths).
