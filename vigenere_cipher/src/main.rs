use std::io::{prelude::*, stdin, stdout};

fn vigenere_cipher(text: &str, key: &str, encrypt: bool) -> String {
    let mut result = String::new();
    let mut key_iter = key.chars().cycle();
    for (_i, c) in text.chars().enumerate() {
        let shift = key_iter.next().unwrap() as u8 - 'a' as u8;
        let new_char = if c.is_ascii_alphabetic() {
            let base = if c.is_uppercase() { 'A' } else { 'a' } as u8;
            let new_char = if encrypt {
                (((c as u8 - base + shift) % 26) + base) as char
            } else {
                (((c as u8 - base + 26 - shift) % 26) + base) as char
            };
            new_char
        } else {
            c
        };
        result.push(new_char);
    }
    result
}

fn main() {
    let mut text = String::new();
    let mut key = String::new();
    print!("Enter text: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut text).unwrap();
    text = text.trim().to_string();

    print!("Enter key: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut key).unwrap();
    key = key.trim().to_string();

    print!("Encrypt (E) or decrypt? (D) ");
    stdout().flush().unwrap();
    let mut choice = String::new();
    stdin().read_line(&mut choice).unwrap();
    choice = choice.trim().to_string();
    match choice.as_str() {
        "E" => println!("Encrypted text: {}", vigenere_cipher(&text, &key, true)),
        "D" => println!("Decrypted text: {}", vigenere_cipher(&text, &key, false)),
        _ => println!("Invalid choice"),
    }
}
