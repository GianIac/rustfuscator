use rust_code_obfuscator::obfuscate_string;

fn main() {
    let username = obfuscate_string!("admin");
    let password = obfuscate_string!("Secret123!");

    println!("User: {}", username);
    println!("Password {}", password);
}