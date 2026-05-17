use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::fmt;
use std::path::{Path, PathBuf};

use crate::config::ObfuscateConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustFile {
    pub path: PathBuf,
    pub relative_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedRustFile {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub reason: SkipReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    MatchedSkipFile,
    NotIncluded,
    Excluded,
}

impl fmt::Display for SkipReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkipReason::MatchedSkipFile => f.write_str("matched skip_files"),
            SkipReason::NotIncluded => f.write_str("not in include patterns"),
            SkipReason::Excluded => f.write_str("excluded by pattern"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilteredRustFiles {
    pub selected: Vec<RustFile>,
    pub skipped: Vec<SkippedRustFile>,
}

pub fn filter_rust_files(
    files: Vec<PathBuf>,
    root: &Path,
    config: &ObfuscateConfig,
) -> Result<FilteredRustFiles> {
    let include_set = match config
        .include
        .as_ref()
        .and_then(|include| include.files.as_ref())
    {
        Some(patterns) => Some(build_glob_set(patterns)?),
        None if config.include.is_some() => Some(build_glob_set(&[])?),
        None => None,
    };

    let exclude_set = match config
        .include
        .as_ref()
        .and_then(|include| include.exclude.as_ref())
    {
        Some(patterns) => Some(build_glob_set(patterns)?),
        None => None,
    };

    let mut selected = Vec::new();
    let mut skipped = Vec::new();

    for path in files {
        let relative_path = relative_path_for(&path, root);
        let reason = skip_reason(
            &path,
            &relative_path,
            config,
            include_set.as_ref(),
            exclude_set.as_ref(),
        );

        if let Some(reason) = reason {
            skipped.push(SkippedRustFile {
                path,
                relative_path,
                reason,
            });
        } else {
            selected.push(RustFile {
                path,
                relative_path,
            });
        }
    }

    Ok(FilteredRustFiles { selected, skipped })
}

fn build_glob_set(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(builder.build()?)
}

fn relative_path_for(path: &Path, root: &Path) -> PathBuf {
    if root.is_file() || path == root {
        return path
            .file_name()
            .map(PathBuf::from)
            .unwrap_or_else(|| path.to_path_buf());
    }

    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

fn skip_reason(
    path: &Path,
    relative_path: &Path,
    config: &ObfuscateConfig,
    include_set: Option<&GlobSet>,
    exclude_set: Option<&GlobSet>,
) -> Option<SkipReason> {
    if config
        .obfuscation
        .skip_files
        .as_ref()
        .is_some_and(|skip_files| {
            skip_files
                .iter()
                .any(|entry| path_ends_with(relative_path, entry))
        })
    {
        return Some(SkipReason::MatchedSkipFile);
    }

    if include_set.is_some_and(|set| !matches_path(set, path, relative_path)) {
        return Some(SkipReason::NotIncluded);
    }

    if exclude_set.is_some_and(|set| matches_path(set, path, relative_path)) {
        return Some(SkipReason::Excluded);
    }

    None
}

fn matches_path(set: &GlobSet, path: &Path, relative_path: &Path) -> bool {
    set.is_match(relative_path) || set.is_match(path)
}

fn path_ends_with(path: &Path, suffix: &str) -> bool {
    let path = path.to_string_lossy().replace('\\', "/");
    let suffix = suffix.replace('\\', "/");
    path.ends_with(&suffix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{IncludeSection, ObfuscationSection};

    fn cfg(
        skip_files: Option<Vec<String>>,
        files: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
    ) -> ObfuscateConfig {
        let include = if files.is_some() || exclude.is_some() {
            Some(IncludeSection { files, exclude })
        } else {
            None
        };

        ObfuscateConfig {
            obfuscation: ObfuscationSection {
                strings: true,
                min_string_length: None,
                ignore_strings: None,
                control_flow: false,
                control_flow_files: None,
                dummy_branches: None,
                obfuscate_logging: None,
                skip_files,
                skip_attributes: None,
            },
            identifiers: None,
            include,
            logging_macros: None,
        }
    }

    fn path_list(paths: &[&str]) -> Vec<PathBuf> {
        paths.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn filters_with_skip_include_and_exclude_rules() {
        let root = Path::new("/workspace");
        let files = path_list(&[
            "/workspace/src/lib.rs",
            "/workspace/src/main.rs",
            "/workspace/tests/integration.rs",
            "/workspace/examples/demo.rs",
        ]);

        let filtered = filter_rust_files(
            files,
            root,
            &cfg(
                Some(vec!["src/main.rs".to_string()]),
                Some(vec!["src/**/*.rs".to_string(), "tests/**/*.rs".to_string()]),
                Some(vec!["tests/**".to_string()]),
            ),
        )
        .unwrap();

        assert_eq!(
            filtered
                .selected
                .iter()
                .map(|file| file.relative_path.as_path())
                .collect::<Vec<_>>(),
            vec![Path::new("src/lib.rs")]
        );
        assert_eq!(
            filtered
                .skipped
                .iter()
                .map(|file| (&file.relative_path, &file.reason))
                .collect::<Vec<_>>(),
            vec![
                (&PathBuf::from("src/main.rs"), &SkipReason::MatchedSkipFile),
                (
                    &PathBuf::from("tests/integration.rs"),
                    &SkipReason::Excluded
                ),
                (&PathBuf::from("examples/demo.rs"), &SkipReason::NotIncluded),
            ]
        );
    }

    #[test]
    fn single_file_root_uses_file_name_as_relative_path() {
        let root = Path::new("/workspace/src/lib.rs");
        let files = path_list(&["/workspace/src/lib.rs"]);

        let filtered = filter_rust_files(files, root, &cfg(None, None, None)).unwrap();

        assert_eq!(filtered.selected[0].relative_path, Path::new("lib.rs"));
    }
}
