use anyhow::{Result, Context};
use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use toml_edit::{Document, Item, Table, value};
use crate::file_io::write_transformed;
use crate::processor::process_file;

pub fn process_project(input: &Path, output: &Path) -> Result<()> {
    copy_full_structure(input, output)?;
    transform_rust_files(output)?;
    patch_cargo_toml(output)?;
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
        .with_context(|| format!("Error in the copy {} a {}", input.display(), output.display()))?;
    Ok(())
}

fn transform_rust_files(project_root: &Path) -> Result<()> {
    for entry in WalkDir::new(project_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let file_path = entry.path();
        let transformed = process_file(file_path)?;
        write_transformed(file_path, &transformed)?;
    }
    Ok(())
}

fn patch_cargo_toml(project_root: &Path) -> Result<()> {
    let cargo_path = project_root.join("Cargo.toml");
    if !cargo_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&cargo_path)?;
    let mut doc = content.parse::<Document>()?;

    if doc.get("dependencies").is_none() {
        doc["dependencies"] = Item::Table(Table::new());
    }

    let deps = doc["dependencies"].as_table_mut().unwrap();
    if deps.contains_key("rust_code_obfuscator") {
        return Ok(());
    }

    let mut subtable = Table::new();
    subtable["path"] = value("../../");
    
    deps.insert("rust_code_obfuscator", Item::Table(subtable));
    
    fs::write(&cargo_path, doc.to_string())?;
    Ok(())
}
