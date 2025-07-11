use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ObfuscateConfig {
    pub obfuscation: ObfuscationSection,
    pub identifiers: Option<IdentifiersSection>,
    pub include: Option<IncludeSection>,
}

#[derive(Debug, Deserialize)]
pub struct ObfuscationSection {
    pub strings: bool,
    pub min_string_length: Option<usize>,
    pub ignore_strings: Option<Vec<String>>,
    pub control_flow: bool,
    pub skip_files: Option<Vec<String>>,
    pub skip_attributes: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct IdentifiersSection {
    pub rename: bool,
    pub preserve: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct IncludeSection {
    pub files: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}
