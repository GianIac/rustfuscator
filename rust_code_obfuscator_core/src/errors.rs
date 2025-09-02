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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_encryption_error() {
        let encryption_err = ObfuscatorError::EncryptionError;

        let displayed = encryption_err.to_string();

        assert! {
            displayed.contains("Encryption failed :(")
        }
    }

    #[test]
    fn display_inv_file_ext() {
        let test_file_path = "./test.txt";

        let inv_file_ext_err = ObfuscatorError::InvalidFileExtension {
            path: PathBuf::from(test_file_path),
        };

        let displayed = inv_file_ext_err.to_string();

        assert! {
            displayed.contains(test_file_path),
            "Expected output contains: '{}',got: {}",
            test_file_path,
            displayed
        }
    }
}
