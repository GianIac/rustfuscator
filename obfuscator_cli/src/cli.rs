use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Apply obfuscation macros to Rust files")]
pub struct Cli {
    /// input
    #[arg(short, long)]
    pub input: PathBuf,

    /// output
    #[arg(short, long)]
    pub output: PathBuf,
}
