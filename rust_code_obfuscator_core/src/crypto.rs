use aes_gcm::{
    aead::{rand_core::RngCore, Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use zeroize::Zeroize;

use crate::errors::ObfuscatorError;

const KEY_LEN: usize = 32;
const SALT_LEN: usize = 16;

/// Optional compile-time key material set by build.rs.
///
/// The AES key is not embedded as a single static byte/string sequence. It is
/// reconstructed from masked shares when `default_key()` is called.
const OBF_KEY_SHARE_A_HEX: Option<&'static str> = option_env!("OBF_KEY_SHARE_A_HEX");
const OBF_KEY_SHARE_B_HEX: Option<&'static str> = option_env!("OBF_KEY_SHARE_B_HEX");
const OBF_KEY_SHARE_C_HEX: Option<&'static str> = option_env!("OBF_KEY_SHARE_C_HEX");
const OBF_KEY_SALT_HEX: Option<&'static str> = option_env!("OBF_KEY_SALT_HEX");

#[derive(Clone)]
pub struct Key([u8; KEY_LEN]);

impl Key {
    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }
}

impl Drop for Key {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

fn parse_hex<const N: usize>(s: &str) -> Result<[u8; N], ObfuscatorError> {
    if s.len() != N * 2 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ObfuscatorError::EncryptionError);
    }
    let mut out = [0u8; N];
    for i in 0..N {
        out[i] = u8::from_str_radix(&s[2 * i..2 * i + 2], 16)
            .map_err(|_| ObfuscatorError::EncryptionError)?;
    }
    Ok(out)
}

/// Default key derived from build-time key shares.
///
/// Falls back to all-zeros only in editor/rust-analyzer contexts where build.rs
/// might not have run.
pub fn default_key() -> Key {
    derive_default_key().unwrap_or_else(|_| Key([0u8; KEY_LEN]))
}

fn derive_default_key() -> Result<Key, ObfuscatorError> {
    let mut share_a =
        parse_hex::<KEY_LEN>(OBF_KEY_SHARE_A_HEX.ok_or(ObfuscatorError::EncryptionError)?)?;
    let mut share_b =
        parse_hex::<KEY_LEN>(OBF_KEY_SHARE_B_HEX.ok_or(ObfuscatorError::EncryptionError)?)?;
    let mut share_c =
        parse_hex::<KEY_LEN>(OBF_KEY_SHARE_C_HEX.ok_or(ObfuscatorError::EncryptionError)?)?;
    let mut salt =
        parse_hex::<SALT_LEN>(OBF_KEY_SALT_HEX.ok_or(ObfuscatorError::EncryptionError)?)?;

    let key = core::array::from_fn(|i| {
        share_c[i]
            ^ share_a[i].rotate_left(rotation(i))
            ^ share_b[i].wrapping_add(offset(i))
            ^ salt[i % SALT_LEN].rotate_right(rotation(KEY_LEN - 1 - i))
    });

    share_a.zeroize();
    share_b.zeroize();
    share_c.zeroize();
    salt.zeroize();

    Ok(Key(key))
}

fn rotation(index: usize) -> u32 {
    (index % 7 + 1) as u32
}

fn offset(index: usize) -> u8 {
    (index as u8).wrapping_mul(17).wrapping_add(91)
}

pub fn encrypt_string(input: &str, key: &Key) -> Result<(Vec<u8>, [u8; 12]), ObfuscatorError> {
    let cipher =
        Aes256Gcm::new_from_slice(key.as_bytes()).map_err(|_| ObfuscatorError::EncryptionError)?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, input.as_bytes())
        .map_err(|_| ObfuscatorError::EncryptionError)?;
    Ok((ciphertext, nonce_bytes))
}

pub fn decrypt_string(data: &[u8], nonce: &[u8; 12], key: &Key) -> Result<String, ObfuscatorError> {
    let cipher =
        Aes256Gcm::new_from_slice(key.as_bytes()).map_err(|_| ObfuscatorError::EncryptionError)?;
    let nonce = Nonce::from_slice(nonce);
    let plaintext = cipher
        .decrypt(nonce, data)
        .map_err(|_| ObfuscatorError::EncryptionError)?;
    match String::from_utf8(plaintext) {
        Ok(value) => Ok(value),
        Err(err) => {
            #[cfg(feature = "secure_zeroize")]
            {
                let mut bytes = err.into_bytes();
                bytes.zeroize();
            }
            #[cfg(not(feature = "secure_zeroize"))]
            let _ = err;
            Err(ObfuscatorError::EncryptionError)
        }
    }
}

pub fn encrypt_u32(input: u32, key: &Key) -> Result<(Vec<u8>, [u8; 12]), ObfuscatorError> {
    let clear = input.to_string();
    let encrypted = encrypt_string(&clear, key);
    #[cfg(feature = "secure_zeroize")]
    {
        let mut clear = clear;
        clear.zeroize();
    }
    encrypted
}

pub fn decrypt_u32(data: &[u8], nonce: &[u8; 12], key: &Key) -> Result<u32, ObfuscatorError> {
    let s = decrypt_string(data, nonce, key)?;
    let parsed = s.parse().map_err(|_| ObfuscatorError::EncryptionError);
    #[cfg(feature = "secure_zeroize")]
    {
        let mut s = s;
        s.zeroize();
    }
    parsed
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
    fn default_key_is_injected_by_build_script() {
        assert!(
            OBF_KEY_SHARE_A_HEX.is_some()
                && OBF_KEY_SHARE_B_HEX.is_some()
                && OBF_KEY_SHARE_C_HEX.is_some()
                && OBF_KEY_SALT_HEX.is_some(),
            "obfuscated key shares must be injected by the crate build script"
        );
    }

    #[test]
    fn raw_default_key_is_not_embedded_as_single_hex_secret() {
        assert!(option_env!("OBF_KEY_HEX").is_none());
    }

    #[test]
    fn decrypt_with_wrong_key_should_err() {
        let k_ok = Key(core::array::from_fn(|i| i as u8));
        let k_bad = Key([0u8; 32]);
        let (ct, nonce) = encrypt_string("secret", &k_ok).unwrap();
        assert!(decrypt_string(&ct, &nonce, &k_bad).is_err());
    }
}
