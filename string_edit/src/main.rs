use std::io::{prelude::*, stdin, stdout};

fn edit_string(string: &mut String, operation: &str) {
    let tmp: String;
    match operation {
        "reverse" => tmp = string.chars().rev().collect(),
        "capitalize" => tmp = string.to_uppercase(),
        "lowercase" => tmp = string.to_lowercase(),
        "switch_case" => tmp = string.chars().map(|c| {
            if c.is_uppercase() {
                c.to_lowercase().next().unwrap()
            } else {
                c.to_uppercase().next().unwrap()
            }
        }).collect(),
        _ => {
            eprintln!("Invalid operation");
            return;
        }
    }
    string.clear();
    string.push_str(&tmp);
}

fn change_string(string: &mut String, operation: &str, args: Option<Vec<String>>) {
    match operation {
        "change_letter" => {
            let args = args.unwrap();
            println!("{:?}", args);
            let index = args[0].parse::<usize>().unwrap();
            let new_char = args[1].chars().next().unwrap();
            string.remove(index);
            string.insert(index, new_char);
        }
        "change_word" => {
            let args = args.unwrap();
            let word = &args[0];
            let new_word = &args[1];
            let mut new_string = String::new();
            let mut iter = string.split_whitespace();
            loop {
                let current_word = iter.next();
                match current_word {
                    Some(current_word) => {
                        if current_word == word {
                            new_string.push_str(new_word);
                        } else {
                            new_string.push_str(current_word);
                        }
                        new_string.push(' ');
                    }
                    None => break,
                }
            }
            string.clear();
            string.push_str(&new_string.trim());
        }
        _ => println!("Invalid operation"),
    }
}

fn main() {
    loop {
        let mut string_input = String::new();
        print!("Enter string: ");
        stdout().flush().unwrap();
        stdin().read_line(&mut string_input)
            .expect("Failed to read line");
        let mut string = string_input.trim().to_string();
        let mut operation_input = String::new();
        print!("Enter operation: ");
        stdout().flush().unwrap();
        stdin().read_line(&mut operation_input)
            .expect("Failed to read line");
        let operation = operation_input.trim();
        if &operation[..2] == "ch" {
            print!("Enter arguments: ");
            stdout().flush().unwrap();
            let mut args_input = String::new();
            stdin().read_line(&mut args_input)
                .expect("Failed to read line");
            let args = args_input.trim().split_whitespace().map(|s| s.to_string()).collect();
            change_string(&mut string, operation, Some(args));
        } else {
            edit_string(&mut string, operation);
        }
        println!("{}", string);
    }
}
