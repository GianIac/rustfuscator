use anyhow::{bail, Context, Result};
use similar::TextDiff;
use std::fs;
use std::path::Path;
use toml_edit::{value, DocumentMut, Item, Table};
use walkdir::WalkDir;

use crate::config::ObfuscateConfig;
use crate::file_filter::filter_rust_files;
use crate::file_io::gather_rust_files;
use crate::file_io::write_transformed;
use crate::processor::process_file;

pub fn process_project(
    input: &Path,
    output: &Path,
    format: bool,
    config: &ObfuscateConfig,
    dry_run: bool,
    diff_ctx: Option<usize>,
    verbose: bool,
) -> Result<()> {
    if dry_run {
        println!("Dry run: scanning project without copying...");
        transform_rust_files(
            input, config, /*format=*/ false, /*dry_run=*/ true, diff_ctx, verbose,
        )?;
        return Ok(());
    }

    ensure_output_outside_input(input, output)?;
    copy_full_structure(input, output)?;
    transform_rust_files(
        output, config, format, /*dry_run=*/ false, diff_ctx, verbose,
    )?;
    patch_cargo_toml(output)?;

    if format {
        ensure_rustfmt_installed()?;
        format_rust_files(output)?;
    }

    Ok(())
}

fn ensure_output_outside_input(input: &Path, output: &Path) -> Result<()> {
    let input = input
        .canonicalize()
        .with_context(|| format!("Cannot resolve input path {}", input.display()))?;
    let output = if output.exists() {
        output
            .canonicalize()
            .with_context(|| format!("Cannot resolve output path {}", output.display()))?
    } else {
        let parent = output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let parent = parent
            .canonicalize()
            .with_context(|| format!("Cannot resolve output parent {}", parent.display()))?;
        parent.join(
            output
                .file_name()
                .context("Output path must have a final component")?,
        )
    };

    if output.starts_with(&input) {
        bail!(
            "Output directory '{}' must be outside input project '{}'",
            output.display(),
            input.display()
        );
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
    fs_extra::dir::copy(input, output, &options).with_context(|| {
        format!(
            "Error copying from {} to {}",
            input.display(),
            output.display()
        )
    })?;
    Ok(())
}

fn transform_rust_files(
    project_root: &Path,
    config: &ObfuscateConfig,
    format: bool,
    dry_run: bool,
    diff_ctx: Option<usize>,
    verbose: bool,
) -> Result<()> {
    let files = filter_rust_files(gather_rust_files(project_root)?, project_root, config)?;
    println!(
        "Found {} Rust files ({} selected, {} skipped)",
        files.selected.len() + files.skipped.len(),
        files.selected.len(),
        files.skipped.len()
    );
    if verbose {
        for skipped in &files.skipped {
            println!(
                "• [SKIP] {} ({})",
                skipped.relative_path.display(),
                skipped.reason
            );
        }
    }

    for file in files.selected {
        let file_path = file.path;
        let relative = file.relative_path;
        let (transformed, changed, before_opt) =
            process_file(&file_path, &relative, config, false)?;

        if verbose {
            println!(
                "• {} {}",
                if changed { "[CHANGED]" } else { "[UNCHANGED]" },
                relative.display()
            );
        }

        if changed {
            if let Some(ctx) = diff_ctx {
                if let Some(before) = before_opt.as_ref() {
                    let diff = TextDiff::from_lines(before, &transformed);
                    let old = format!("{} (before)", relative.display());
                    let new = format!("{} (after)", relative.display());
                    println!(
                        "{}",
                        diff.unified_diff().context_radius(ctx).header(&old, &new)
                    );
                }
            }
            if !dry_run {
                println!("Writing {}", file_path.display());
                write_transformed(&file_path, &transformed, format)?;
            }
        }
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
            println!(
                "Skipping patch: {} is a virtual manifest",
                cargo_path.display()
            );
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
            deps.insert("rust_code_obfuscator", value("0.3.2"));
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
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let path = entry.path();
        let result = std::process::Command::new("rustfmt").arg(path).output();

        if let Err(e) = result {
            eprintln!("Warning: Failed to format {}: {}", path.display(), e);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_format_rust_files() {
        let src: &str = r#"pub const TEST:    &str =     "test";"#;

        let file_name = "simple_file.rs";
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(file_name);
        std::fs::write(&path, src).unwrap();

        format_rust_files(dir.path()).unwrap();

        let formated_content = fs::read_to_string(path).unwrap();
        assert_eq!(formated_content.trim(), r#"pub const TEST: &str = "test";"#);
    }

    #[test]
    fn rejects_project_output_inside_input_tree() {
        let input = tempfile::tempdir().unwrap();
        let output = input.path().join("obfuscated");

        let error = ensure_output_outside_input(input.path(), &output).unwrap_err();
        assert!(error.to_string().contains("must be outside input project"));
    }

    #[test]
    fn accepts_sibling_project_output() {
        let root = tempfile::tempdir().unwrap();
        let input = root.path().join("input");
        fs::create_dir(&input).unwrap();

        ensure_output_outside_input(&input, &root.path().join("output")).unwrap();
    }

    #[test]
    fn rejects_existing_output_equal_to_input() {
        let input = tempfile::tempdir().unwrap();

        let error = ensure_output_outside_input(input.path(), input.path()).unwrap_err();
        assert!(error.to_string().contains("must be outside input project"));
    }

    #[test]
    fn patches_package_manifests_but_skips_virtual_manifests() {
        let root = tempfile::tempdir().unwrap();
        let member = root.path().join("member");
        fs::create_dir(&member).unwrap();
        fs::write(
            root.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"member\"]\n",
        )
        .unwrap();
        fs::write(
            member.join("Cargo.toml"),
            "[package]\nname = \"member\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        patch_cargo_toml(root.path()).unwrap();

        let workspace = fs::read_to_string(root.path().join("Cargo.toml")).unwrap();
        let package = fs::read_to_string(member.join("Cargo.toml")).unwrap();
        assert!(!workspace.contains("rust_code_obfuscator"));
        assert!(package.contains("rust_code_obfuscator = \"0.3.2\""));
        assert!(!package.contains("cryptify"));
    }

    #[test]
    fn preserves_existing_rustfuscator_dependency() {
        let root = tempfile::tempdir().unwrap();
        fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[dependencies]\nrust_code_obfuscator = \"0.2\"\n",
        )
        .unwrap();

        patch_cargo_toml(root.path()).unwrap();

        let package = fs::read_to_string(root.path().join("Cargo.toml")).unwrap();
        assert!(package.contains("rust_code_obfuscator = \"0.2\""));
        assert!(!package.contains("rust_code_obfuscator = \"0.3.2\""));
    }

    #[test]
    fn copies_nested_project_structure() {
        let root = tempfile::tempdir().unwrap();
        let input = root.path().join("input");
        let output = root.path().join("output");
        fs::create_dir_all(input.join("src/nested")).unwrap();
        fs::write(input.join("src/nested/lib.rs"), "pub fn demo() {}\n").unwrap();

        copy_full_structure(&input, &output).unwrap();

        assert_eq!(
            fs::read_to_string(output.join("src/nested/lib.rs")).unwrap(),
            "pub fn demo() {}\n"
        );
    }

    #[test]
    fn project_mode_copies_transforms_and_patches_a_complete_project() {
        let root = tempfile::tempdir().unwrap();
        let input = root.path().join("input");
        let output = root.path().join("output");
        fs::create_dir_all(input.join("src")).unwrap();
        fs::write(
            input.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        fs::write(
            input.join("src/lib.rs"),
            "pub fn secret() { let value: &str = \"hidden\"; }\n",
        )
        .unwrap();
        let config = ObfuscateConfig {
            obfuscation: crate::config::ObfuscationSection {
                strings: true,
                min_string_length: None,
                ignore_strings: None,
                control_flow: false,
                control_flow_files: None,
                dummy_branches: None,
                obfuscate_logging: None,
                skip_files: None,
                skip_attributes: None,
            },
            identifiers: None,
            include: None,
            logging_macros: None,
        };

        process_project(&input, &output, false, &config, false, Some(1), true).unwrap();

        let source = fs::read_to_string(output.join("src/lib.rs")).unwrap();
        let manifest = fs::read_to_string(output.join("Cargo.toml")).unwrap();
        assert!(source.contains("obfuscate_str!(\"hidden\")"), "{source}");
        assert!(manifest.contains("rust_code_obfuscator = \"0.3.2\""));
    }

    #[test]
    fn dry_run() {
        let src: &str = r#"pub const TEST: &str = "test";"#;

        let file_name = "simple_file.rs";
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(file_name);
        std::fs::write(&path, src).unwrap();

        let config = ObfuscateConfig {
            obfuscation: crate::config::ObfuscationSection {
                strings: true,
                min_string_length: None,
                ignore_strings: None,
                control_flow: true,
                control_flow_files: None,
                dummy_branches: None,
                obfuscate_logging: None,
                skip_files: None,
                skip_attributes: None,
            },
            identifiers: None,
            include: None,
            logging_macros: None,
        };

        let result = transform_rust_files(&path, &config, false, true, None, false);
        match result {
            Ok(_) => {}
            Err(_) => panic!("transform_rust_files fails with error"),
        }
        let formated_content = fs::read_to_string(path).unwrap();
        assert_eq!(formated_content.trim(), src);
    }

    #[test]
    fn try_transform_rust_files_with_no_format() {
        let src_1: &str = r#"
fn main() {let test: &str = "test";}
"#;
        let src_2: &str = r#"
fn main() {let test: &str = "test";loop {}}
"#;
        let src_3: &str = r#"
fn main() {
while true {}
for i in [] {}
let x = Some(1);
match x {
    None => None,
    Some(i) => {},
    _ => {},}}
"#;

        let file_name_1 = "simple_file_1.rs";
        let file_name_2 = "simple_file_2.rs";
        let file_name_3 = "simple_file_3.rs";
        let dir = tempfile::tempdir().unwrap();
        let path_1 = dir.path().join(file_name_1);
        let path_2 = dir.path().join(file_name_2);
        let path_3 = dir.path().join(file_name_3);
        std::fs::write(&path_1, src_1).unwrap();
        std::fs::write(&path_2, src_2).unwrap();
        std::fs::write(&path_3, src_3).unwrap();

        let config = ObfuscateConfig {
            obfuscation: crate::config::ObfuscationSection {
                strings: true,
                min_string_length: None,
                ignore_strings: None,
                control_flow: true,
                control_flow_files: None,
                dummy_branches: None,
                obfuscate_logging: None,
                skip_files: None,
                skip_attributes: None,
            },
            identifiers: None,
            include: None,
            logging_macros: None,
        };

        let result = transform_rust_files(dir.path(), &config, false, false, None, false);
        match result {
            Ok(_) => {}
            Err(_) => panic!("transform_rust_files fails with error"),
        }
        let formated_content = fs::read_to_string(path_1).unwrap();
        for line in formated_content.lines() {
            println!("{}", line);
        }
        let formated_content = fs::read_to_string(path_2).unwrap();
        for line in formated_content.lines() {
            println!("{}", line);
        }
        let formated_content = fs::read_to_string(path_3).unwrap();
        for line in formated_content.lines() {
            println!("{}", line);
        }
    }
}
