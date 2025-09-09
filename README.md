# 🦀 rustfuscator

[![crates.io](https://img.shields.io/crates/v/rust_code_obfuscator.svg)](https://crates.io/crates/rust_code_obfuscator)
[![Contributing](https://img.shields.io/badge/docs-contributing-blueviolet?logo=github)](./CONTRIBUTING.md)
[![Whitepaper](https://img.shields.io/badge/docs-whitepaper-lightgrey?logo=readthedocs)](https://github.com/GianIac/rustfuscator/blob/main/WHITEPAPER.md)
[![Obfuscation Guide](https://img.shields.io/badge/docs-obfuscation_fundamentals-blue?logo=rust)](https://gianiac.github.io/rustfuscator/obfuscation_fundamentals.html)

**Obfuscation-first CLI and library for Rust.**  
Protect your source code from reverse engineering by encrypting string literals, injecting opaque control-flow, and rewriting AST logic — with full automation CLI, macros or a derive.

---

## Features

- Full CLI: obfuscate files, folders, or full Cargo projects
- Configurable via `.obfuscate.toml`
- Output formatting (`--format`)
- No runtime dependency or unpacking
- Obfuscate string literals at compile-time (`obfuscate_string!`)
- Insert control-flow breaking statements (`obfuscate_flow!`)
- Derive macro for struct encryption (`#[derive(Obfuscate)]`)

## How Obfuscation Works

The rustfuscator CLI does not obfuscate your compiled binary directly.

Instead, it rewrites your Rust source code to wrap strings and logic in macros like obfuscate_string!() and obfuscate_flow!(). These macros perform real obfuscation by:

Encrypting values at compile-time using the cryptify crate.
Embedding encrypted data into the binary instead of plaintext.
Generating runtime decryption logic, so your binary still works as expected.
This approach is:

Fully integrated with the Rust compiler
Transparent to your runtime logic
Compatible with all platforms supported by Rust

> ➡️ Important: After using the CLI, you still need to build your code with cargo build to produce the final obfuscated binary.

##### If you have time read --> [![Obfuscation Guide](https://img.shields.io/badge/docs-obfuscation_fundamentals-blue?logo=rust)](https://gianiac.github.io/rustfuscator/obfuscation_fundamentals.html) :)

---

## Installation

Once published:

`cargo install rustfuscator`

Or clone and build:

`git clone https://github.com/GianIac/rustfuscator`

`cd rustfuscator`

`cargo install --path obfuscator_cli`

## Command-Line Interface

The CLI is the most powerful part of rustfuscator. You can obfuscate anything from a single file to a full Rust project.

Obfuscate a single file

`obfuscator_cli --input ./src/main.rs --output ./obf`

Obfuscate a full source folder

`obfuscator_cli --input ./src --output ./obf_src`

Obfuscate an entire Cargo project

`obfuscator_cli \
  --input ./my_project \
  --output ./my_project_obf \
  --as-project \
  --format`
  
Automatically:

- Copies your project structure
- Patches Cargo.toml
- Applies macro-based obfuscation
- Optionally formats the output

## Configuration with .obfuscate.toml
Generate a default config:

`obfuscator_cli --input ./my_project --init`

Example file:

> The `.obfuscate.toml` file controls what parts of your code get obfuscated and how.  
> This file is especially useful when obfuscating full projects.

<pre><code>
  
  [obfuscation]
  strings = true
  min_string_length = 4
  ignore_strings = ["DEBUG", "LOG"]
  control_flow = true
  skip_files = ["src/main.rs"]
  skip_attributes = true 
  
  [identifiers]
  rename = false
  preserve = ["main"]
  
  [include]
  files = ["**/*.rs"]
  exclude = ["target/**", "tests/**"]
 
</code></pre>

## Library Usage

Add to your Cargo.toml:

`[dependencies] -->
rust_code_obfuscator = "0.2.8"
cryptify = "3.1.1"`

Use it:

<pre><code>
use rust_code_obfuscator::{obfuscate_string, obfuscate_flow};

fn main() {
    let secret = obfuscate_string!("hidden string");
    obfuscate_flow!();
    println!("{}", secret);
}
</code></pre>

Derive Struct Encryption

<pre><code>
use rust_code_obfuscator::Obfuscate;

#[derive(Obfuscate)]
struct MyData {
    name: String,
    age: u32,
}
</code></pre>

## Project Layout

<pre><code>
rustfuscator/
├── rust_code_obfuscator/     # Core library
├── obfuscator_derive/        # Proc macro derive
├── obfuscator_cli/           # CLI interface
├── examples/                 # Basic example use cases
├── README.md
├── Cargo.toml (workspace)
</code></pre>

## Disclaimer

Obfuscation does not guarantee complete protection.
It significantly increases the complexity of reverse engineering, but should be combined with:

- Binary stripping
- Anti-debugging
- Compiler flags
- Other hardening strategies

```markdown
For stronger binary protection, consider compiling with:

RUSTFLAGS="-C strip=debuginfo -C opt-level=z -C panic=abort" cargo build --release
```

## Author Note

The first commits signed by `user <user@local>` are authored by me (GianIac).

## License

MIT License © 2025 Gianfranco Iaculo
