use rust_code_obfuscator::Obfuscate;

#[derive(Obfuscate)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub token: String,
    pub user_id: u32,
}

fn main() {
    let clear = Credentials {
        username: "user".to_string(),
        password: "pwd!".to_string(),
        token: "csadveea-token".to_string(),
        user_id: 42,
    };

    let obf = ObfuscatedCredentials::new_clear(
        &clear.username,
        &clear.password,
        &clear.token,
        clear.user_id,
    );

    let restored = obf.get_clear();

    println!("Decrypted credentials:");
    println!("Username: {}", restored.username);
    println!("Password: {}", restored.password);
    println!("Token: {}", restored.token);
    println!("User ID: {}", restored.user_id);
}
