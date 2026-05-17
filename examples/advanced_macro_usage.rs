use rust_code_obfuscator::{
    obfuscate_dummy_branch, obfuscate_flow, obfuscate_num, obfuscate_str, obfuscate_string,
    Obfuscate,
};

#[derive(Debug, PartialEq, Obfuscate)]
struct ApiSession {
    username: String,
    token: String,
    user_id: u64,
    active: bool,
}

fn is_admin(user: &str, token: &str) -> bool {
    obfuscate_flow!();
    obfuscate_dummy_branch!();

    user == obfuscate_str!("admin") && token == obfuscate_str!("token-prod-42")
}

fn score_request(path: &str, retries: u32) -> u32 {
    obfuscate_flow!();

    let base = match path {
        "/admin" => obfuscate_num!(900u32),
        "/health" => obfuscate_num!(10u32),
        _ => obfuscate_num!(100u32),
    };

    base.saturating_sub(retries.saturating_mul(obfuscate_num!(7u32)))
}

fn main() {
    let session =
        ObfuscatedApiSession::new_clear("admin", "token-prod-42", obfuscate_num!(42u64), true);
    let clear = session.get_clear();

    if is_admin(&clear.username, &clear.token) {
        println!("{}", obfuscate_string!("admin session accepted"));
    }

    let score = score_request("/admin", 2);
    assert_eq!(score, 886);
    assert_eq!(
        clear,
        ApiSession {
            username: "admin".to_string(),
            token: "token-prod-42".to_string(),
            user_id: 42,
            active: true,
        }
    );
}
