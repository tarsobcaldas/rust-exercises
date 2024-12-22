use std::io::prelude::*;
use std::io::{stdin, stdout};

fn map_array(array: Vec<isize>, op: &str, n: isize) -> Vec<isize> {
    let mut result: Vec<isize> = Vec::new();
    for &num in array.iter() {
        match op {
            "add" | "+" => result.push(num + n),
            "sub" | "-" => result.push(num - n),
            "mul" | "*" => result.push(num * n),
            "div" | "/" => result.push(num / n),
            _ => eprintln!("Invalid operation"),
        }
    }
    result
}

fn main() {
    loop {
        let mut input = String::new();
        print!("Enter array numbers (space separated): ");
        stdout().flush().unwrap();
        stdin().read_line(&mut input).expect("Failed to read line");
        let mut numbers: Vec<isize> = Vec::new();
        for num in input.trim().split_whitespace() {
            numbers.push(num.parse().unwrap());
        }

        print!("Enter basic operation (either by first three letters or by symbol): ");
        stdout().flush().unwrap();
        input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");
        let op = input.trim().to_lowercase();

        print!("Enter number: ");
        stdout().flush().unwrap();
        input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");
        let n: isize = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid input");
                return;
            }
        };

        let result: Vec<isize> = map_array(numbers, &op.trim(), n);
        print!("Result: [");
        for (i, &num) in result.iter().enumerate() {
            if i != result.len() - 1 {
                print!("{}, ", num);
            } else {
                println!("{}]", num);
            }
        }
        stdout().flush().unwrap();
    }
}
