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
    run(Cli::parse())
}

fn run(args: Cli) -> Result<()> {
    println!("Input path: {}", args.input.display());
    if args.init {
        generate_default_config(&args.input)?;
        println!("Generated default .obfuscate.toml");
        return Ok(());
    }

    if !args.input.exists() {
        bail!("Input path '{}' does not exist.", args.input.display());
    }

    let output = args.output.as_ref().expect("Missing --output");

    let config_path = config_path_for_input(&args.input);
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
    if args.verbose {
        println!("Configuration: {config:#?}");
    }

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

fn config_path_for_input(input: &Path) -> std::path::PathBuf {
    let config_root = if input.is_file() {
        input.parent().unwrap_or_else(|| Path::new("."))
    } else {
        input
    };
    config_root.join(".obfuscate.toml")
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

#[cfg(test)]
mod tests {
    use super::*;

    fn write_config(root: &Path, strings: bool) {
        fs::write(
            root.join(".obfuscate.toml"),
            format!("[obfuscation]\nstrings = {strings}\ncontrol_flow = false\n"),
        )
        .unwrap();
    }

    fn cli(input: std::path::PathBuf, output: std::path::PathBuf) -> Cli {
        Cli {
            input,
            output: Some(output),
            as_project: false,
            format: false,
            init: false,
            json: false,
            dry_run: false,
            verbose: true,
            diff: Some(None),
        }
    }

    #[test]
    fn single_file_uses_config_from_parent_directory() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input.rs");
        fs::write(&input, "fn main() {}").unwrap();

        assert_eq!(
            config_path_for_input(&input),
            dir.path().join(".obfuscate.toml")
        );
    }

    #[test]
    fn run_rejects_missing_input_and_missing_config() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.rs");
        let error = run(cli(missing.clone(), dir.path().join("out"))).unwrap_err();
        assert!(error.to_string().contains("does not exist"));

        let input = dir.path().join("input.rs");
        fs::write(&input, "fn main() {}").unwrap();
        let error = run(cli(input, dir.path().join("out"))).unwrap_err();
        assert!(error.to_string().contains("Missing .obfuscate.toml"));
    }

    #[test]
    fn init_creates_parseable_default_configuration() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("new-project");
        let mut args = cli(target.clone(), dir.path().join("unused"));
        args.init = true;

        run(args).unwrap();

        let config = fs::read_to_string(target.join(".obfuscate.toml")).unwrap();
        let parsed: ObfuscateConfig = toml::from_str(&config).unwrap();
        assert!(parsed.obfuscation.strings);
        assert!(parsed.obfuscation.control_flow);
    }

    #[test]
    fn single_file_run_transforms_and_writes_selected_source() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input.rs");
        let output = dir.path().join("out");
        fs::write(&input, "fn main() { let secret: &str = \"hello\"; }").unwrap();
        write_config(dir.path(), true);

        run(cli(input, output.clone())).unwrap();

        let transformed = fs::read_to_string(output.join("input.rs")).unwrap();
        assert!(transformed.contains("obfuscate_str!(\"hello\")"));
        assert!(transformed.contains("use rust_code_obfuscator::obfuscate_str;"));
    }

    #[test]
    fn dry_run_does_not_write_transformed_source() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input.rs");
        let output = dir.path().join("out");
        fs::write(&input, "fn main() { let secret: &str = \"hello\"; }").unwrap();
        write_config(dir.path(), true);
        let mut args = cli(input, output.clone());
        args.dry_run = true;
        args.verbose = false;
        args.diff = None;

        run(args).unwrap();

        assert!(!output.join("input.rs").exists());
    }

    #[test]
    fn project_dry_run_uses_project_pipeline_without_copying() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("project");
        let output = dir.path().join("out");
        fs::create_dir(&input).unwrap();
        fs::write(
            input.join("lib.rs"),
            "fn demo() { let value: &str = \"hello\"; }",
        )
        .unwrap();
        write_config(&input, true);
        let mut args = cli(input, output.clone());
        args.as_project = true;
        args.dry_run = true;

        run(args).unwrap();

        assert!(!output.exists());
    }
}
