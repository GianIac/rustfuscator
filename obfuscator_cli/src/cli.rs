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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_and_explicit_diff_context() {
        let default_context = Cli::try_parse_from([
            "obfuscator_cli",
            "--input",
            "src",
            "--output",
            "out",
            "--diff",
        ])
        .unwrap();
        assert_eq!(default_context.diff, Some(None));

        let explicit_context = Cli::try_parse_from([
            "obfuscator_cli",
            "--input",
            "src",
            "--output",
            "out",
            "--diff=7",
        ])
        .unwrap();
        assert_eq!(explicit_context.diff, Some(Some(7)));
    }

    #[test]
    fn init_does_not_require_output_but_normal_runs_do() {
        let init = Cli::try_parse_from(["obfuscator_cli", "--input", "project", "--init"]).unwrap();
        assert!(init.output.is_none());
        assert!(init.init);

        let missing_output = Cli::try_parse_from(["obfuscator_cli", "--input", "project"]);
        assert!(missing_output.is_err());
    }
}
