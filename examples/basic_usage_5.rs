use rust_code_obfuscator::Obfuscate;

#[derive(Obfuscate)]
struct User {
    username: String,
    age: u32,
}

fn main() {
    let obf = ObfuscatedUser::new_clear("McFratm", 28);
    let user = obf.get_clear();

    println!("Decrypted: {}, {}", user.username, user.age);
}
