use aes_gcm::{
    aead::{rand_core::RngCore, Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};

pub const AES_KEY: &[u8; 32] = b"01234567890123456789012345678901";

pub fn encrypt_string(input: &str, key: &[u8; 32]) -> Result<(Vec<u8>, [u8; 12]), ObfuscatorError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| ObfuscatorError::EncryptionError)?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, input.as_bytes())
        .map_err(|_| ObfuscatorError::EncryptionError)?;
    Ok((ciphertext, nonce_bytes))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn crete_new_key() -> [u8; 32] {
        let new_key: [u8; 32] = core::array::from_fn(|i| i as u8);
        new_key
    }

    fn create_string_to_encrypt() -> &'static str {
        "secret data"
    }

    fn create_number_to_encrypt() -> u32 {
        1234
    }

    #[test]
    fn encrypt_and_decrypt_str() {
        let data_to_encrypt = create_string_to_encrypt();
        let my_new_key: [u8; 32] = crete_new_key();

        let encrypted: (Vec<u8>, [u8; 12]) = encrypt_string(data_to_encrypt, &my_new_key);
        let decrypted = decrypt_string(&encrypted.0, &encrypted.1, &my_new_key);

        assert_eq!(data_to_encrypt, decrypted);
    }

    #[test]
    fn encrypt_and_decrypt_u32() {
        let data_to_encrypt: u32 = create_number_to_encrypt();
        let my_new_key: [u8; 32] = crete_new_key();

        let encrypted: (Vec<u8>, [u8; 12]) = encrypt_u32(data_to_encrypt, &my_new_key);
        let decrypted = decrypt_u32(&encrypted.0, &encrypted.1, &my_new_key);

        assert_eq!(data_to_encrypt, decrypted);
    }

    #[test]
    fn encrypt_and_decrypt_with_global_aes_key() {
        let string_to_encrypt = create_string_to_encrypt();
        let number_to_encrypt: u32 = create_number_to_encrypt();

        let encrypted: (Vec<u8>, [u8; 12]) = encrypt_string(string_to_encrypt, &AES_KEY);
        let decrypted = decrypt_string(&encrypted.0, &encrypted.1, &AES_KEY);
        assert_eq!(string_to_encrypt, decrypted);

        let encrypted: (Vec<u8>, [u8; 12]) = encrypt_u32(number_to_encrypt, &AES_KEY);
        let decrypted = decrypt_u32(&encrypted.0, &encrypted.1, &AES_KEY);
        assert_eq!(number_to_encrypt, decrypted);
    }

    #[test]
    #[should_panic]
    fn try_decrypt_with_used_nonce() {
        let data_to_encrypt = create_string_to_encrypt();
        let my_new_key: [u8; 32] = crete_new_key();

        let encrypted_1: (Vec<u8>, [u8; 12]) = encrypt_string(data_to_encrypt, &my_new_key);
        let encrypted_2: (Vec<u8>, [u8; 12]) = encrypt_string(data_to_encrypt, &my_new_key);
        let decrypted = decrypt_string(&encrypted_2.0, &encrypted_1.1, &my_new_key);

        assert_eq!(data_to_encrypt, decrypted);
    }

    #[test]
    #[should_panic]
    fn decrypt_with_wrong_key_should_panic() {
        let key_ok: [u8; 32] = core::array::from_fn(|i| i as u8);
        let key_bad: [u8; 32] = [0u8; 32];
        let (ct, nonce) = encrypt_string("secret", &key_ok);
        let _ = decrypt_string(&ct, &nonce, &key_bad); // expected panic
    }

    #[test]
    fn encrypt_and_decrypt_unicode() {
        let key: [u8; 32] = core::array::from_fn(|i| i as u8);
        let s = "café Καλημέρα — 数据";
        let (ct, nonce) = encrypt_string(s, &key);
        assert_eq!(s, decrypt_string(&ct, &nonce, &key));
    }
}
