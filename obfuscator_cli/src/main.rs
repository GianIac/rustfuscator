mod cli;
mod file_io;
mod processor;
mod project_mode;
mod config;

use anyhow::{bail, Result};
use clap::Parser;
use cli::Cli;
use config::ObfuscateConfig;
use std::{fs, path::Path};

fn main() -> Result<()> {
    let args = Cli::parse();

    println!("Input path: {}", args.input.display());
    if !args.input.exists() {
        bail!("Input path '{}' does not exist.", args.input.display());
    }

    if args.init {
        generate_default_config(&args.input)?;
        println!("Generated default .obfuscate.toml");
        return Ok(());
    }

    let output = args.output.as_ref().expect("Missing --output");

    let config_path = args.input.join(".obfuscate.toml");
    if !config_path.exists() {
        bail!(
            "Missing .obfuscate.toml in input directory.\n\
             Run `rustfuscator --init --input {}` to generate one.",
            args.input.display()
        );
    }

    println!("Loading config from: {}", config_path.display());
    let config_str = fs::read_to_string(&config_path)?;
    let config: ObfuscateConfig = toml::from_str(&config_str)?;
    dbg!(&config);

    if args.as_project {
        println!("Running in project mode...");
        project_mode::process_project(&args.input, output, args.format, &config)?;
    } else {
        println!("Running in single-file mode...");
        if !output.exists() {
            fs::create_dir_all(output)?;
        }

        let files = file_io::gather_rust_files(&args.input)?;
        println!("Found {} files", files.len());
        for file_path in files {
            println!("\nProcessing: {}", file_path.display());

            let relative = if args.input.is_file() {
                std::path::Path::new(file_path.file_name().unwrap())
            } else {
                file_path.strip_prefix(&args.input)?
            };

            dbg!(relative);

            let transformed = processor::process_file(&file_path, relative, &config)?;

            let output_path = output.join(relative);
            println!("Writing to: {}", output_path.display());
            file_io::write_transformed(&output_path, &transformed)?;
        }
    }

    Ok(())
}

fn generate_default_config(target: &Path) -> Result<()> {
    if !target.exists() {
        fs::create_dir_all(target)?;
    }

    let default = r#"
[obfuscation]
strings = true
min_string_length = 4
ignore_strings = ["DEBUG", "LOG"]
control_flow = true
skip_files = ["src/main.rs"]

[identifiers]
rename = false
preserve = ["main"]

[include]
files = ["**/*.rs"]
exclude = ["target/**", "tests/**"]
"#;

    let path = target.join(".obfuscate.toml");
    println!("Generating config at: {}", path.display());
    fs::write(&path, default.trim_start())?;
    Ok(())
}
