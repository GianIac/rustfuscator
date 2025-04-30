use std::{fmt, error};

#[derive(Debug)]
pub enum ObfuscatorError {
    EncryptionError,
}

impl fmt::Display for ObfuscatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObfuscatorError::EncryptionError => write!(f, "Encryption failed :( "),
        }
    }
}

impl error::Error for ObfuscatorError {}