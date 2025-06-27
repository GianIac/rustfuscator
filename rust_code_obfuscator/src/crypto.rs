use aes_gcm::{
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
    Aes256Gcm, Nonce,
};

pub const AES_KEY: &[u8; 32] = b"01234567890123456789012345678901";

pub fn encrypt_string(input: &str, key: &[u8; 32]) -> (Vec<u8>, [u8; 12]) {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, input.as_bytes()).unwrap();
    (ciphertext, nonce_bytes)
}

pub fn decrypt_string(data: &[u8], nonce: &[u8; 12], key: &[u8; 32]) -> String {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce = Nonce::from_slice(nonce);
    let plaintext = cipher.decrypt(nonce, data).unwrap();
    String::from_utf8(plaintext).unwrap()
}

pub fn encrypt_u32(input: u32, key: &[u8; 32]) -> (Vec<u8>, [u8; 12]) {
    encrypt_string(&input.to_string(), key)
}

pub fn decrypt_u32(data: &[u8], nonce: &[u8; 12], key: &[u8; 32]) -> u32 {
    decrypt_string(data, nonce, key).parse().unwrap()
}
