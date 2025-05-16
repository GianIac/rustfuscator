use anyhow::{Context, Result};
use regex::{Regex, Captures};
use std::fs;
use std::path::Path;

pub fn process_file(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Errore nella lettura del file: {}", path.display()))?;

    let content = obfuscate_strings(&content);
    let content = obfuscate_flows(&content);

    Ok(content)
}

fn obfuscate_strings(input: &str) -> String {
    let re = Regex::new(r#"(?P<full>"([^"\\]|\\.){4,}")"#).unwrap();

    let re_macro = Regex::new(r#"(println!|eprintln!|format!)\s*\("#).unwrap();
    let re_obf = Regex::new(r#"obfuscate_string!\s*\("#).unwrap();

    let mut result = String::with_capacity(input.len());
    let mut last_end = 0;

    for cap in re.captures_iter(input) {
        let mat = cap.name("full").unwrap();
        let start = mat.start();
        let end = mat.end();
        let prefix = &input[last_end..start];

        if re_obf.is_match(&input[start.saturating_sub(25)..start])
            || re_macro.is_match(prefix)
        {
            result.push_str(&input[last_end..end]);
        } else {
            result.push_str(prefix);
            result.push_str(&format!("obfuscate_string!({})", mat.as_str()));
        }

        last_end = end;
    }

    result.push_str(&input[last_end..]);
    result
}

fn obfuscate_flows(input: &str) -> String {
    let re = Regex::new(
        r#"(?m)^(\s*)(if\s*\(.*?\)|else\s+if\s*\(.*?\)|while\s*\(.*?\)|for\s+.*?in\s+.*?|loop\s*\{|match\s*\(.*?\))"#
    ).unwrap();

    let mut result = String::with_capacity(input.len());
    let mut last_end = 0;

    for cap in re.captures_iter(input) {
        let full = cap.get(0).unwrap();
        let indent = cap.get(1).map_or("", |m| m.as_str());

        result.push_str(&input[last_end..full.start()]);
        result.push_str(&format!("{indent}obfuscate_flow!();\n"));
        result.push_str(full.as_str());

        last_end = full.end();
    }

    result.push_str(&input[last_end..]);
    result
}
