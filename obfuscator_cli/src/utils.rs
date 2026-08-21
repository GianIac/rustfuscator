use anyhow::Result;
use cargo_metadata::MetadataCommand;
use std::fs;
use std::path::Path;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_virtual_and_package_manifests() {
        let dir = tempfile::tempdir().unwrap();
        let virtual_manifest = dir.path().join("virtual.toml");
        let package_manifest = dir.path().join("package.toml");
        fs::write(&virtual_manifest, "[workspace]\nmembers = []\n").unwrap();
        fs::write(
            &package_manifest,
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        assert!(is_virtual_manifest(&virtual_manifest).unwrap());
        assert!(!is_virtual_manifest(&package_manifest).unwrap());
    }

    #[test]
    fn reads_workspace_versions_and_reports_unknown_crates() {
        assert_eq!(
            get_local_crate_version("rust_code_obfuscator").unwrap(),
            Some("0.3.2".to_string())
        );
        assert_eq!(
            get_local_crate_version("definitely-not-a-crate").unwrap(),
            None
        );
    }
}
