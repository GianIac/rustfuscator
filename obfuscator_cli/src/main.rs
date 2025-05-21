mod cli;
mod file_io;
mod processor;
mod project_mode;

use clap::Parser;
use cli::Cli;
use anyhow::{Result, bail};
use std::fs;

fn main() -> Result<()> {
    let args = Cli::parse();

    if !args.input.exists() {
        bail!("Input path '{}' does not exist.", args.input.display());
    }

    if args.as_project {
        project_mode::process_project(&args.input, &args.output)?;
    } else {
        if !args.output.exists() {
            fs::create_dir_all(&args.output)?;
        }

        let files = file_io::gather_rust_files(&args.input)?;
        for file_path in files {
            let transformed = processor::process_file(&file_path)?;
            let relative = if args.input.is_file() {
                std::path::Path::new(file_path.file_name().unwrap())
            } else {
                file_path.strip_prefix(&args.input)?
            };
            let output_path = args.output.join(relative);
            file_io::write_transformed(&output_path, &transformed)?;
        }
    }

    Ok(())
}
