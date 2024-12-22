use std::io::{self, Write};

fn map_array<F>(array: Vec<isize>, op: F) -> Vec<isize>
where
    F: Fn(isize) -> isize,
{
    array.into_iter().map(op).collect()
}

fn main() {
    loop {
        let mut input = String::new();
        print!("Enter array numbers (space separated): ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let numbers: Vec<isize> = input
            .split_whitespace()
            .map(|num| num.parse().expect("Invalid number"))
            .collect();

        print!("Enter basic operation (either by first three letters or by symbol): ");
        io::stdout().flush().unwrap();
        input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let op = input.trim().to_lowercase();

        print!("Enter number: ");
        io::stdout().flush().unwrap();
        input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let n: isize = input.trim().parse().expect("Invalid input");

        let result: Vec<isize> = match op.as_str() {
            "add" | "+" => map_array(numbers, |x| x + n),
            "sub" | "-" => map_array(numbers, |x| x - n),
            "mul" | "*" => map_array(numbers, |x| x * n),
            "div" | "/" => map_array(numbers, |x| x / n),
            _ => {
                eprintln!("Invalid operation");
                continue;
            }
        };

        println!("Result: {:?}", result);
    }
}
