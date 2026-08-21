use rust_code_obfuscator::{obfuscate_flow, obfuscate_string};

fn main() {
    obfuscate_flow!();

    let secret = obfuscate_string!("Codice segreto!");
    println!("--> {}", secret);
}
