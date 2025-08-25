use anyhow::Result;
use rust_code_obfuscator_core::errors::ObfuscatorError;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn gather_rust_files(input: &Path) -> Result<Vec<PathBuf>> {
    if input.is_file() {
        accept_only_rust_files(input)?;
        Ok(vec![input.to_path_buf()])
    } else {
        let files: Vec<PathBuf> = WalkDir::new(input)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
            .map(|e| e.path().to_path_buf())
            .collect();
        Ok(files)
    }
}

pub fn write_transformed(dest: &Path, content: &str, format: bool) -> Result<()> {
    accept_only_rust_files(dest)?;
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

fn accept_only_rust_files(file: &Path) -> Result<(), ObfuscatorError> {
    if file.extension() == Some(OsStr::new("rs")) {
        return Ok(());
    } else {
        return Err(ObfuscatorError::InvalidFileExtension);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    use std::io::Read;

    fn get_valid_file_names() -> Vec<&'static str> {
        vec!["file_1.rs", "file_2.rs"]
    }

    fn get_invalid_file_names() -> Vec<&'static str> {
        vec![
            "file.r",
            "file.s",
            "file.sr",
            "file.rsrs",
            "file.rs_rs",
            ".rs",
            "file.txt",
        ]
    }

    fn get_folder_paths() -> Vec<&'static str> {
        vec![
            "folder_1",
            "folder_1/folder_1_1",
            "folder_1/folder_1_1/folder_1_1_1",
            "folder_rs",
            "rs",
        ]
    }

    fn create_test_files() -> std::io::Result<(TempDir, PathBuf, Vec<PathBuf>, Vec<PathBuf>)> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path().to_path_buf();

        // Create subdirectories
        let mut sub_dirs: Vec<PathBuf> = Vec::new();
        for path in get_folder_paths() {
            let path_buf = root.join(path);
            fs::create_dir_all(&path_buf)?;
            sub_dirs.push(path_buf);
        }

        // Create test files
        let mut valid_file_paths: Vec<PathBuf> = Vec::new();
        let mut invalid_file_paths: Vec<PathBuf> = Vec::new();
        for dir in sub_dirs {
            for valid_file in get_valid_file_names() {
                let file_path: PathBuf = dir.join(valid_file);
                valid_file_paths.push(file_path.clone());
                File::create(file_path)?;
            }
            for invalid_file in get_invalid_file_names() {
                let file_path: PathBuf = dir.join(invalid_file);
                invalid_file_paths.push(file_path.clone());
                File::create(file_path)?;
            }
        }
        Ok((temp_dir, root, valid_file_paths, invalid_file_paths))
    }

    #[test]
    fn gather_rust_files_with_folder_input() {
        let (_temp_dir, root, mut valid_file_paths, _) = create_test_files().unwrap();
        let results: Vec<PathBuf> = gather_rust_files(&root).unwrap();

        // check wether the number of items in `results` equals number of necessary valid files in `valid_file_paths`
        assert_eq!(results.len(), valid_file_paths.len());

        // check wether each item in `results` can be found in `valid_file_paths`
        for result in results {
            if valid_file_paths.contains(&result) {
                if let Some(pos) = valid_file_paths.iter().position(|x| *x == result) {
                    valid_file_paths.remove(pos);
                }
            }
        }

        assert_eq!(valid_file_paths.len(), 0);
    }

    #[test]
    fn gather_rust_files_with_valid_file_input() {
        let (_temp_dir, _root, valid_file_paths, _) = create_test_files().unwrap();

        for f_path in &valid_file_paths {
            let results: Vec<PathBuf> = gather_rust_files(f_path).unwrap();
            assert_eq!(results.len(), 1);
            assert!(results.contains(f_path));
        }
    }

    #[test]
    fn gather_rust_files_with_invalid_file_input() {
        let (_temp_dir, _root, _, invalid_file_paths) = create_test_files().unwrap();

        for f_path in &invalid_file_paths {
            match gather_rust_files(f_path) {
                Ok(_) => panic!("It should panic"),
                Err(e) => {
                    if e.downcast_ref::<ObfuscatorError>()
                        .is_some_and(|err| matches!(err, ObfuscatorError::InvalidFileExtension))
                    {
                        ()
                    } else {
                        panic!("Unexpected error: {:?}", e);
                    }
                }
            }
        }
    }

    #[test]
    fn write_transformed_invalid_extension() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        let content = "simple content";

        for invalid_file in get_invalid_file_names() {
            let dest = root.join(invalid_file);
            match write_transformed(&dest, content, false) {
                Ok(_) => panic!("It should panic"),
                Err(e) => {
                    if e.downcast_ref::<ObfuscatorError>()
                        .is_some_and(|err| matches!(err, ObfuscatorError::InvalidFileExtension))
                    {
                        ();
                    } else {
                        panic!("Unexpected error: {:?}", e);
                    }
                }
            }
        }
    }

    #[test]
    fn try_write_transformed() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        let content = "simple content";

        let dest = root.join("valid_file.rs");
        write_transformed(&dest, content, false).unwrap();

        let mut file = File::open(dest).unwrap();
        let mut saved_content = String::new();
        file.read_to_string(&mut saved_content).unwrap();
        assert_eq!(content, saved_content);
    }
}
