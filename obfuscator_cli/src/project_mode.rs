use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use toml_edit::{DocumentMut, Item, Table, value};

use crate::config::ObfuscateConfig;
use crate::file_io::write_transformed;
use crate::processor::process_file;
use crate::utils::{is_virtual_manifest, get_local_crate_version};

pub fn process_project(input: &Path, output: &Path, format: bool, config: &ObfuscateConfig) -> Result<()> {
    copy_full_structure(input, output)?;
    transform_rust_files(output, config, format)?;
    patch_cargo_toml(output)?;

    if format {
        ensure_rustfmt_installed()?;
        format_rust_files(output)?;
    }

    Ok(())
}

fn copy_full_structure(input: &Path, output: &Path) -> Result<()> {
    let options = fs_extra::dir::CopyOptions {
        copy_inside: true,
        overwrite: true,
        content_only: false,
        ..Default::default()
    };
    fs_extra::dir::copy(input, output, &options)
        .with_context(|| format!("Error copying from {} to {}", input.display(), output.display()))?;
    Ok(())
}

fn transform_rust_files(project_root: &Path, config: &ObfuscateConfig, format: bool) -> Result<()> {
    for entry in WalkDir::new(project_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let file_path = entry.path();
        let relative = file_path.strip_prefix(project_root)?;
        let transformed = process_file(file_path, relative, config, false)?;

        println!("Writing {}", file_path.display());

        write_transformed(&file_path, &transformed, format)?;
    }
    Ok(())
}

fn patch_cargo_toml(project_root: &Path) -> Result<()> {
    for entry in WalkDir::new(project_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_name() == "Cargo.toml")
    {
        let cargo_path = entry.path();

        let content = fs::read_to_string(cargo_path)?;
        let mut doc = content.parse::<DocumentMut>()?;

        // Skip virtual manifests
        if !doc.contains_key("package") {
            println!("Skipping patch: {} is a virtual manifest", cargo_path.display());
            continue;
        }

        if doc.get("dependencies").is_none() {
            doc["dependencies"] = Item::Table(Table::new());
        }

        let deps = doc["dependencies"]
            .as_table_mut()
            .context("Expected [dependencies] to be a table")?;

        // Only insert if not already present
        if !deps.contains_key("rust_code_obfuscator") {
            deps.insert("rust_code_obfuscator", value("0.2.6"));
        }
        if !deps.contains_key("cryptify") {
            deps.insert("cryptify", value("3.1.1"));
        }

        fs::write(cargo_path, doc.to_string())?;
        println!("✓ Patched dependencies in {}", cargo_path.display());
    }

    Ok(())
}

fn ensure_rustfmt_installed() -> Result<()> {
    let rustfmt_check = std::process::Command::new("rustfmt")
        .arg("--version")
        .output();

    if rustfmt_check.is_err() {
        bail!(
            "`rustfmt` is not installed.\n\
             To enable formatting, run:\n  rustup component add rustfmt"
        );
    }

    Ok(())
}

fn format_rust_files(project_root: &Path) -> Result<()> {
    for entry in WalkDir::new(project_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let path = entry.path();
        let result = std::process::Command::new("rustfmt")
            .arg(path)
            .output();

        if let Err(e) = result {
            eprintln!("Warning: Failed to format {}: {}", path.display(), e);
        }
    }
    Ok(())
}
