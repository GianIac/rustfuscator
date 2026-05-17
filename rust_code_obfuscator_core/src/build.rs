use std::env;

const KEY_LEN: usize = 32;
const SALT_LEN: usize = 16;

fn main() {
    let key_hex = env::var("OBFUSCATOR_KEY_HEX").ok();

    let key_bytes: [u8; KEY_LEN] = if let Some(hex) = key_hex {
        let hex = hex.trim();
        assert!(
            hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()),
            "OBFUSCATOR_KEY_HEX must be 64 hex chars (32 bytes)"
        );
        let mut out = [0u8; KEY_LEN];
        for i in 0..KEY_LEN {
            let b = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16)
                .expect("Invalid hex in OBFUSCATOR_KEY_HEX");
            out[i] = b;
        }
        out
    } else {
        // Random per-build key
        rand::random()
    };

    let salt: [u8; SALT_LEN] = rand::random();
    let share_a: [u8; KEY_LEN] = rand::random();
    let share_b: [u8; KEY_LEN] = rand::random();
    let share_c = encode_key_share(&key_bytes, &share_a, &share_b, &salt);

    println!("cargo:rustc-env=OBF_KEY_SHARE_A_HEX={}", to_hex(&share_a));
    println!("cargo:rustc-env=OBF_KEY_SHARE_B_HEX={}", to_hex(&share_b));
    println!("cargo:rustc-env=OBF_KEY_SHARE_C_HEX={}", to_hex(&share_c));
    println!("cargo:rustc-env=OBF_KEY_SALT_HEX={}", to_hex(&salt));

    println!("cargo:rerun-if-env-changed=OBFUSCATOR_KEY_HEX");
}

fn encode_key_share(
    key: &[u8; KEY_LEN],
    share_a: &[u8; KEY_LEN],
    share_b: &[u8; KEY_LEN],
    salt: &[u8; SALT_LEN],
) -> [u8; KEY_LEN] {
    core::array::from_fn(|i| {
        key[i]
            ^ share_a[i].rotate_left(rotation(i))
            ^ share_b[i].wrapping_add(offset(i))
            ^ salt[i % SALT_LEN].rotate_right(rotation(KEY_LEN - 1 - i))
    })
}

fn rotation(index: usize) -> u32 {
    (index % 7 + 1) as u32
}

fn offset(index: usize) -> u8 {
    (index as u8).wrapping_mul(17).wrapping_add(91)
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
