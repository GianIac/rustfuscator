use anyhow::Result;
use std::path::Path;
use std::fs;
use cargo_metadata::MetadataCommand;

#[allow(dead_code)]
/// Returns (is_virtual_manifest, root_package_path)
pub fn is_virtual_manifest(path: &Path) -> Result<bool> {
    let manifest = fs::read_to_string(path)?;
    Ok(manifest.contains("[workspace]"))
}

#[allow(dead_code)]
/// Returns the version of a crate dependency from the workspace root
pub fn get_local_crate_version(crate_name: &str) -> Result<Option<String>> {
    let metadata = MetadataCommand::new().exec()?;
    for package in metadata.packages {
        if package.name.as_str() == crate_name {
            return Ok(Some(package.version.to_string()));
        }
    }
    Ok(None)
}
