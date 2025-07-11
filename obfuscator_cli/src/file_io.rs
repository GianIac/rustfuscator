use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn gather_rust_files(input: &Path) -> Result<Vec<PathBuf>> {
    if input.is_file() {
        Ok(vec![input.to_path_buf()])
    } else {
        let files = WalkDir::new(input)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
            .map(|e| e.path().to_path_buf())
            .collect();
        Ok(files)
    }
}

pub fn write_transformed(dest: &Path, content: &str, format: bool) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(dest, content)?;

    if format {
        let _ = std::process::Command::new("rustfmt")
            .arg(dest)
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to run rustfmt: {}", e))?;
    }

    Ok(())
}
