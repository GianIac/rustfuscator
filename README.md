# 🦀 rustfuscator

[![crates.io](https://img.shields.io/crates/v/rust_code_obfuscator.svg)](https://crates.io/crates/rust_code_obfuscator)
[![Contributing](https://img.shields.io/badge/docs-contributing-blueviolet?logo=github)](./CONTRIBUTING.md)
[![Whitepaper](https://img.shields.io/badge/docs-whitepaper-lightgrey?logo=readthedocs)](https://github.com/GianIac/rustfuscator/blob/main/WHITEPAPER.md)
[![Obfuscation Guide](https://img.shields.io/badge/docs-obfuscation_fundamentals-blue?logo=rust)](https://gianiac.github.io/rustfuscator/obfuscation_fundamentals.html)

Rustfuscator is an obfuscation-first CLI and library for Rust codebases. It rewrites source code and exposes macros for string literals, numeric literals, control-flow noise, logging literals, identifier renaming, and derive-based encrypted fields.

Obfuscation is pragmatic: it does not make software invulnerable, but it can raise the cost of casual static analysis and reverse engineering when it is scoped and tested carefully.

## Features

- CLI for files, folders, or Cargo projects.
- `.obfuscate.toml` configuration with include/exclude and per-file control-flow selection.
- Compile-time string literal obfuscation:
  - `obfuscate_string!("...")` returns `ObfStr`
  - `obfuscate_str!("...")` returns `&'static str`
- Lightweight integer literal obfuscation with `obfuscate_num!(...)`.
- Control-flow injection with `obfuscate_flow!`.
- Optional dummy branch injection with `obfuscate_dummy_branch!` and CLI `dummy_branches`.
- Logging macro literal rewriting for `println!`, `eprintln!`, `log::*`, and `tracing::*`.
- Identifier renaming strategies: `suffix`, `hash`, and `confuse`.
- `#[derive(Obfuscate)]` for structs with `String`, `bool`, and Rust integer primitive fields.
- Optional `secure_zeroize` feature for supported clear values and temporary clear buffers.
- Optional `verify_literals` feature for debug-only literal round-trip assertions.

## How It Works

The CLI does not obfuscate compiled binaries directly. It parses Rust source, rewrites selected constructs, and emits Rust code that uses the library macros.

The macro layer then performs the runtime behavior needed by the transformed source, such as decrypting string literals on demand or injecting flow noise. After running the CLI, build the transformed project with Cargo as usual.

## Installation

```bash
cargo install rustfuscator
```

Or from a local checkout:

```bash
git clone https://github.com/GianIac/rustfuscator
cd rustfuscator
cargo install --path obfuscator_cli
```

## CLI Usage

Obfuscate a single Rust file:

```bash
obfuscator_cli --input ./src/main.rs --output ./obf
```

Obfuscate a source folder:

```bash
obfuscator_cli --input ./src --output ./obf_src
```

Obfuscate a Cargo project:

```bash
obfuscator_cli \
  --input ./my_project \
  --output ./my_project_obf \
  --as-project \
  --format
```

Generate a default config:

```bash
obfuscator_cli --input ./my_project --init
```

Useful review flags:

```bash
obfuscator_cli --input ./src --output ./obf_src --dry-run --diff
obfuscator_cli --input ./src --output ./obf_src --verbose
```

## Configuration

Example `.obfuscate.toml`:

```toml
[obfuscation]
strings = true
min_string_length = 4
ignore_strings = ["DEBUG", "LOG"]
control_flow = true
control_flow_files = ["**/*.rs"]
dummy_branches = false
obfuscate_logging = true
skip_files = ["src/main.rs"]
skip_attributes = true

[identifiers]
rename = false
strategy = "suffix" # suffix | hash | confuse
preserve = ["main"]

[include]
files = ["**/*.rs"]
exclude = ["target/**", "tests/**"]

[logging_macros]
enabled = [
  "println",
  "eprintln",
  "log::info",
  "log::warn",
  "log::error",
  "tracing::info",
  "tracing::warn",
]
ignore_messages = ["DEBUG", "TRACE", "startup ok"]
```

## Library Usage

Add the library:

```toml
[dependencies]
rust_code_obfuscator = "0.3.0"
```

Use macros directly:

```rust
use rust_code_obfuscator::{
    obfuscate_dummy_branch, obfuscate_flow, obfuscate_num, obfuscate_str, obfuscate_string,
};

fn main() {
    let secret = obfuscate_string!("hidden string");
    let role = obfuscate_str!("admin");
    let threshold = obfuscate_num!(1337u32);

    obfuscate_flow!();
    obfuscate_dummy_branch!();

    println!("{secret} {role} {threshold}");
}
```

## Derive Usage

```rust
use rust_code_obfuscator::Obfuscate;

#[derive(Debug, PartialEq, Obfuscate)]
struct MyData {
    name: String,
    enabled: bool,
    age: u32,
}

fn main() {
    let obfuscated = ObfuscatedMyData::new_clear("Alice", true, 42);
    let clear = obfuscated.get_clear();

    assert_eq!(clear.age, 42);
}
```

Supported derive field types are `String`, `bool`, and Rust integer primitives. Floats, containers, and custom types are intentionally out of scope.

## Examples

Run the advanced macro example:

```bash
cargo run --example advanced_macro_usage
```

Other examples under `examples/` cover basic string obfuscation, control-flow injection, derive usage, and CLI transformation targets.

## Benchmarks

Benchmarks use Criterion and compare baseline code against macro-assisted code:

```bash
cargo bench --bench macro_overhead
```

The benchmark suite currently covers:

- normal arithmetic vs `obfuscate_flow!`
- plain string literal access vs `obfuscate_string!`
- plain integer literal access vs `obfuscate_num!`

Benchmark results are workload-specific; use them to estimate overhead for your own threat model and performance budget.

## Feature Flags

```toml
[dependencies]
rust_code_obfuscator = { version = "0.3.0", features = ["secure_zeroize", "verify_literals"] }
```

- `secure_zeroize`: zeroizes supported clear values and temporary clear buffers.
- `verify_literals`: enables debug-only round-trip assertions inside string literal macros.

## Project Layout

```text
rustfuscator/
├── src/                         # Public facade crate
├── rust_code_obfuscator_core/    # Core macros and crypto helpers
├── obfuscator_derive/            # #[derive(Obfuscate)]
├── obfuscator_cli/               # CLI source rewriter
├── examples/                     # Runnable examples
├── benches/                      # Criterion benchmarks
└── tests/                        # Integration tests
```

## Disclaimer

Rustfuscator is an obfuscation tool, not a guarantee of secrecy. Combine it with normal release hardening where appropriate:

```bash
RUSTFLAGS="-C strip=debuginfo -C opt-level=z -C panic=abort" cargo build --release
```

## Author Note

The first commits signed by `user <user@local>` are authored by Gianfranco Iaculo.

## License

MIT License © 2025 Gianfranco Iaculo
