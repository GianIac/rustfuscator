mod cli;
mod config;
mod file_filter;
mod file_io;
mod processor;
mod project_mode;
mod utils;

use anyhow::{bail, Result};
use clap::Parser;
use cli::Cli;
use config::ObfuscateConfig;
use file_filter::filter_rust_files;
use similar::TextDiff;
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
        // --diff          => Some(None)  -> use 3 lines of context
        // --diff=5        => Some(Some(5))
        // (no --diff) => None
        let diff_ctx = args.diff.map(|opt| opt.unwrap_or(3));
        project_mode::process_project(
            &args.input,
            output,
            args.format,
            &config,
            args.dry_run,
            diff_ctx,
            args.verbose,
        )?;
    } else {
        println!("Running in single-file mode...");
        if !output.exists() {
            fs::create_dir_all(output)?;
        }

        let files = file_io::gather_rust_files(&args.input)?;
        let files = filter_rust_files(files, &args.input, &config)?;
        println!(
            "Found {} Rust files ({} selected, {} skipped)",
            files.selected.len() + files.skipped.len(),
            files.selected.len(),
            files.skipped.len()
        );
        if args.verbose {
            for skipped in &files.skipped {
                println!(
                    "• [SKIP] {} ({})",
                    skipped.relative_path.display(),
                    skipped.reason
                );
            }
        }
        let diff_ctx = args.diff.map(|opt| opt.unwrap_or(3));
        for file in files.selected {
            let file_path = file.path;
            let relative = file.relative_path;
            println!("\nProcessing: {}", file_path.display());

            let (transformed, changed, before_opt) =
                processor::process_file(&file_path, &relative, &config, args.json)?;

            if args.json {
                continue;
            }

            if args.verbose {
                println!(
                    "• {} {}",
                    if changed { "[CHANGED]" } else { "[UNCHANGED]" },
                    relative.display()
                );
            }

            if changed {
                if let Some(ctx) = diff_ctx {
                    if let Some(before) = before_opt.as_ref() {
                        let diff = TextDiff::from_lines(before, &transformed);
                        let old = format!("{} (before)", relative.display());
                        let new = format!("{} (after)", relative.display());
                        println!(
                            "{}",
                            diff.unified_diff().context_radius(ctx).header(&old, &new)
                        );
                    }
                }
                if !args.dry_run {
                    let output_path = output.join(&relative);
                    println!("Writing to: {}", output_path.display());
                    file_io::write_transformed(&output_path, &transformed, args.format)?;
                }
            }
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
control_flow_files = ["**/*.rs"]
dummy_branches = false
obfuscate_logging = true
skip_files = ["src/main.rs"]
skip_attributes = true

[identifiers]
rename = false
strategy = "suffix"
preserve = ["main"]

[include]
files = ["**/*.rs"]
exclude = ["target/**", "tests/**"]

[logging_macros]
enabled = ["println", "eprintln", "log::info", "log::warn", "log::error", "tracing::info", "tracing::warn"]
ignore_messages = ["DEBUG", "TRACE", "startup ok"]
"#;

    let path = target.join(".obfuscate.toml");
    println!("Generating config at: {}", path.display());
    fs::write(&path, default.trim_start())?;
    Ok(())
}
