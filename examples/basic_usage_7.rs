use rust_code_obfuscator::{obfuscate_string, obfuscate_flow};

fn main() {
    // Uso di obfuscate_string!
    let secret = obfuscate_string!("Questa è una stringa segreta");
    println!("{}", secret);

    // Uso di obfuscate_flow! con if
    if 2 + 2 == 4 {
        println!("{}", obfuscate_string!("Condizione verificata!"));
    }

    // Uso di obfuscate_flow! con for
    for i in 0..3 {
        println!("{}", obfuscate_string!("Iterazione!"));
    }

    // Uso di obfuscate_flow! con match
    let n = 1;
    match n {
        0 => println!("{}", obfuscate_string!("Zero")),
        1 => println!("{}", obfuscate_string!("Uno")),
        _ => println!("{}", obfuscate_string!("Altro")),
    }

    // Uso di obfuscate_flow! con while
    let mut count = 0;
    obfuscate_flow!();
    while count < 1 {
        println!("{}", obfuscate_string!("Un solo giro!"));
        count += 1;
    }
}
