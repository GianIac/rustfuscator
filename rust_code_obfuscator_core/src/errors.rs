use std::{error, fmt, path::PathBuf};

#[derive(Debug)]
pub enum ObfuscatorError {
    EncryptionError,
    InvalidFileExtension { path: PathBuf },
}

impl fmt::Display for ObfuscatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObfuscatorError::EncryptionError => write!(f, "Encryption failed :("),
            ObfuscatorError::InvalidFileExtension { path } => {
                write!(f, "Invalid file extension for: {}", path.display())
            }
        }
    }
}

impl error::Error for ObfuscatorError {}
