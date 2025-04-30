use rust_code_obfuscator::{obfuscate_string, obfuscate_flow};

fn main() {
    obfuscate_flow!();

    let secret = obfuscate_string!("Codice segreto!");
    println!("--> {}", secret);
}