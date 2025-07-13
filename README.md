# 🦀 rustfuscator

[![crates.io](https://img.shields.io/crates/v/rust_code_obfuscator.svg)](https://crates.io/crates/rust_code_obfuscator)
[![docs.rs](https://img.shields.io/docsrs/rust_code_obfuscator)](https://docs.rs/rust_code_obfuscator)

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

`rustfuscator --input ./src/main.rs --output ./obf`

Obfuscate a full source folder

`rustfuscator --input ./src --output ./obf_src`

Obfuscate an entire Cargo project

`rustfuscator \
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

`rustfuscator --input ./my_project --init`

Example file:

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
  exclude = ["target/**", "tests/**"]
</code></pre>

## Library Usage

Add to your Cargo.toml:

`[dependencies] -->
rust_code_obfuscator = "0.2.1"`

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

## Author Note

Commits signed by `user <user@local>` are authored by me (GianIac). This is due to a local Git config and will be corrected going forward.

## License

MIT License © 2025 Gianfranco Iaculo
