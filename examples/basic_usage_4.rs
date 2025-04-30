use rust_code_obfuscator::{obfuscate_flow, obfuscate_string};

fn login(user: &str, pass: &str) -> bool {
    obfuscate_flow!(); // change flow

    user == obfuscate_string!("admin") && pass == obfuscate_string!("password123!")
}

fn main() {
    let u = "admin";
    let p = "password123";

    if login(u, p) {
        println!("{}", obfuscate_string!("Access OK!"));
    } else {
        println!("{}", obfuscate_string!("Access denied..."));
    }
}