use std::{error, fmt};

#[derive(Debug)]
pub enum ObfuscatorError {
    EncryptionError,
    InvalidFileExtension,
}

impl fmt::Display for ObfuscatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObfuscatorError::EncryptionError => write!(f, "Encryption failed :( "),
            ObfuscatorError::InvalidFileExtension => {
                write!(f, "An invalid file extension was used.")
            }
        }
    }
}

impl error::Error for ObfuscatorError {}
