mod cli;
mod file_io;
mod processor;

use clap::Parser;
use anyhow::{Result, bail};

use cli::Cli;
use file_io::{gather_rust_files, write_transformed};
use processor::process_file;

use std::fs;

fn main() -> Result<()> {
    let args = Cli::parse();

    if !args.input.exists() {
        bail!("Input path '{}' does not exist.", args.input.display());
    }

    if !args.output.exists() {
        fs::create_dir_all(&args.output)?;
    }

    let files = gather_rust_files(&args.input)?;
    for file_path in files {
        println!("Processing file: {:?}", file_path);

        let transformed = process_file(&file_path)?;

        let relative = if args.input.is_file() {
            std::path::Path::new(file_path.file_name().unwrap())
        } else {
            file_path.strip_prefix(&args.input)?
        };

        let output_path = args.output.join(relative);
        write_transformed(&output_path, &transformed)?;
    }

    Ok(())
}
