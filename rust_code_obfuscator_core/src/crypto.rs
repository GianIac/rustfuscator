use aes_gcm::{
    aead::{rand_core::RngCore, Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use zeroize::Zeroize;

use crate::errors::ObfuscatorError;

/// Optional compile-time environment set by build.rs
const OBF_KEY_HEX: Option<&'static str> = option_env!("OBF_KEY_HEX");

#[derive(Clone)]
pub struct Key([u8; 32]);

impl Key {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Drop for Key {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

fn parse_hex32(s: &str) -> Result<[u8; 32], ObfuscatorError> {
    if s.len() != 64 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ObfuscatorError::EncryptionError);
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&s[2 * i..2 * i + 2], 16)
            .map_err(|_| ObfuscatorError::EncryptionError)?;
    }
    Ok(out)
}

/// Default key loaded at build-time (generated or from OBFUSCATOR_KEY_HEX via build.rs).
/// Falls back to all-zeros in editor/rust-analyzer contexts where build.rs might not have run.
pub fn default_key() -> Key {
    let k = match OBF_KEY_HEX {
        Some(hex) => parse_hex32(hex).unwrap_or([0u8; 32]),
        None => [0u8; 32], // dev-only fallback
    };
    Key(k)
}

pub fn encrypt_string(input: &str, key: &Key) -> Result<(Vec<u8>, [u8; 12]), ObfuscatorError> {
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|_| ObfuscatorError::EncryptionError)?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, input.as_bytes())
        .map_err(|_| ObfuscatorError::EncryptionError)?;
    Ok((ciphertext, nonce_bytes))
}

pub fn decrypt_string(
    data: &[u8],
    nonce: &[u8; 12],
    key: &Key,
) -> Result<String, ObfuscatorError> {
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|_| ObfuscatorError::EncryptionError)?;
    let nonce = Nonce::from_slice(nonce);
    let plaintext = cipher
        .decrypt(nonce, data)
        .map_err(|_| ObfuscatorError::EncryptionError)?;
    String::from_utf8(plaintext).map_err(|_| ObfuscatorError::EncryptionError)
}

pub fn encrypt_u32(input: u32, key: &Key) -> Result<(Vec<u8>, [u8; 12]), ObfuscatorError> {
    encrypt_string(&input.to_string(), key)
}

pub fn decrypt_u32(data: &[u8], nonce: &[u8; 12], key: &Key) -> Result<u32, ObfuscatorError> {
    let s = decrypt_string(data, nonce, key)?;
    s.parse().map_err(|_| ObfuscatorError::EncryptionError)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_new_key() -> Key {
        let k: [u8; 32] = core::array::from_fn(|i| i as u8);
        Key(k)
    }

    #[test]
    fn encrypt_and_decrypt_str() {
        let k = create_new_key();
        let (ct, nonce) = encrypt_string("secret data", &k).unwrap();
        let pt = decrypt_string(&ct, &nonce, &k).unwrap();
        assert_eq!(pt, "secret data");
    }

    #[test]
    fn encrypt_and_decrypt_u32() {
        let k = create_new_key();
        let (ct, nonce) = encrypt_u32(1234, &k).unwrap();
        let n = decrypt_u32(&ct, &nonce, &k).unwrap();
        assert_eq!(n, 1234);
    }

    #[test]
    fn encrypt_and_decrypt_with_default_key() {
        let k = default_key();
        let (ct, nonce) = encrypt_string("hello", &k).unwrap();
        let pt = decrypt_string(&ct, &nonce, &k).unwrap();
        assert_eq!(pt, "hello");
    }

    #[test]
    fn decrypt_with_wrong_key_should_err() {
        let k_ok = Key(core::array::from_fn(|i| i as u8));
        let k_bad = Key([0u8; 32]);
        let (ct, nonce) = encrypt_string("secret", &k_ok).unwrap();
        assert!(decrypt_string(&ct, &nonce, &k_bad).is_err());
    }
}
