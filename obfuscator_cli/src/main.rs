mod cli;
mod file_io;
mod processor;

use cli::Cli;
use file_io::{gather_rust_files, write_transformed};
use processor::process_file;

use clap::Parser; 
use anyhow::Result;

fn main() -> Result<()> {
    let args = Cli::parse();

    let files = gather_rust_files(&args.input)?;
    for file_path in files {
        let transformed = process_file(&file_path)?;
        let relative = file_path.strip_prefix(&args.input)?;
        let output_path = args.output.join(relative);
        write_transformed(&output_path, &transformed)?;
    }

    Ok(())
}
