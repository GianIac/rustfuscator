use rust_code_obfuscator::{obfuscate_flow, obfuscate_str, obfuscate_string};

fn takes_str(s: &str) {
    println!("[takes_str] {s}");
}

fn greet(user: &str) -> String {
    format!("Hello, {}!", user)
}

fn login(user: &str, pass: &str) -> bool {
    obfuscate_flow!(); // Flow obfuscation inside the function

    // `obfuscate_string!` returns `ObfStr` (deref-coerces to `&str`)
    user == obfuscate_string!("admin") && pass == obfuscate_string!("password123")
}

fn main() {
    obfuscate_flow!(); // Flow obfuscation at entry

    // 1) Ergonomics: Display + `&str` usage via deref coercion
    let name = obfuscate_string!("Alice");
    println!("Name (Display): {name}");

    takes_str(&name);
    let msg = greet(&name);
    println!("Greet: {msg}");
    assert_eq!(msg, "Hello, Alice!");

    // 2) Comparisons (works if `ObfStr: PartialEq<&str>`)
    assert!(name == "Alice");
    println!("Comparison OK (name == \"Alice\")");

    // 3) Pattern matching: use `obfuscate_str!` (returns `&'static str`)
    match obfuscate_str!("ADMIN") {
        "ADMIN" => println!("Pattern match OK (ADMIN)"),
        _ => unreachable!("Unexpected value"),
    }

    // 4) Borrow-safe option usage (no `.as_deref()` on temporaries)
    let profile: Option<&str> = Some(obfuscate_str!("release"));
    assert_eq!(profile, Some("release"));
    println!("Profile: {:?}", profile);

    // 5) Realistic refactoring target: login checks against obfuscated literals
    let username = "admin";
    let password = "password123";

    if login(username, password) {
        println!("{}", obfuscate_string!("Access OK!"));
    } else {
        println!("{}", obfuscate_string!("Access denied..."));
    }

    // 6) Important note: caching is per call-site (two call-sites may yield different pointers)
    // So we validate content equality, not pointer equality.
    let a = obfuscate_str!("cached_literal");
    let b = obfuscate_str!("cached_literal");
    println!("Cache sanity: a={a}, b={b}");
    assert_eq!(a, "cached_literal");
    assert_eq!(b, "cached_literal");

    println!("All basic_usage_10 checks done");
}
