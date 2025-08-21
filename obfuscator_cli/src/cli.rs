use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Apply obfuscation macros to Rust files")]
pub struct Cli {
    /// input
    #[arg(short, long)]
    pub input: PathBuf,

    /// output
    #[arg(short, long, required_unless_present = "init")]
    pub output: Option<PathBuf>,

    /// obfuscate as full project
    #[arg(long, default_value_t = false)]
    pub as_project: bool,

    /// format output files with rustfmt
    #[arg(long, default_value_t = false)]
    pub format: bool,

    /// Generate a default .obfuscate.toml file
    #[arg(long, default_value_t = false)]
    pub init: bool,

    /// Output result as JSON
    #[arg(long, default_value_t = false)]
    pub json: bool,

    /// Do not write to disk; print what would change
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Verbose logging (prints touched files/passes)
    #[arg(long, default_value_t = false)]
    pub verbose: bool,

    /// Show unified diff; optional context lines (default 3)
    #[arg(long)]
    pub diff: Option<Option<usize>>,
}
