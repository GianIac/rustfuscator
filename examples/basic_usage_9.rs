use rust_code_obfuscator::obfuscate_string;

fn greet(user: &str) -> String {
    format!("Hello, {}!", user)
}

fn main() {
    // Usable as &str directly
    let name = obfuscate_string!("Alice");
    println!("Name: {}", name); // Debug + Display

    // Assignment to &str
    let name_ref: &str = &name;
    assert_eq!(name_ref, "Alice");

    // Passing to a function that accepts &str
    let msg = greet(&name);
    assert_eq!(msg, "Hello, Alice!");

    // Comparison in if
    if &*name == "Alice" {
        println!("Match succeeded");
    }

    // Match pattern
    match &*obfuscate_string!("ADMIN") {
        "ADMIN" => println!("Role match ok"),
        _ => println!("Role mismatch"),
    }

    println!("All obfuscate_string! tests passed.");
}
