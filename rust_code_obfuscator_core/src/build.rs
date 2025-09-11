use std::{env, fs, path::PathBuf};

fn main() {
    let key_hex = env::var("OBFUSCATOR_KEY_HEX").ok();

    let key_bytes: [u8; 32] = if let Some(hex) = key_hex {
        let hex = hex.trim();
        assert!(
            hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()),
            "OBFUSCATOR_KEY_HEX must be 64 hex chars (32 bytes)"
        );
        let mut out = [0u8; 32];
        for i in 0..32 {
            let b = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16)
                .expect("Invalid hex in OBFUSCATOR_KEY_HEX");
            out[i] = b;
        }
        out
    } else {
        // Random per-build key
        use rand::RngCore;
        let mut tmp = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut tmp);
        tmp
    };

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let key_path = out_dir.join("obf_key.bin");
    fs::write(&key_path, &key_bytes).expect("Failed to write obf_key.bin");

    let hex = key_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    println!("cargo:rustc-env=OBF_KEY_HEX={}", hex);

    println!("cargo:rerun-if-env-changed=OBFUSCATOR_KEY_HEX");
}
