use std::io::prelude::*;
use std::io::{stdin, stdout};

fn main() {
    fn check_prime(n: u32) -> bool {
        if n <= 1 {
            return false;
        } else if n <= 3 {
            return true;
        } else if n % 2 == 0 || n % 3 == 0 {
            return false;
        }

        let mut i = 5;
        while i <= (n as f64).sqrt() as u32 {
            if n % i == 0 || n % (i + 2) == 0 {
                return false;
            }
            i += 6;
        }

        true
    }

    loop {
        let mut input = String::new();
        print!("Enter number: ");
        stdout().flush().unwrap();
        stdin().read_line(&mut input).expect("Failed to read line");

        let n: u32 = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid input");
                return;
            }
        };
        let result: bool = check_prime(n);

        if result {
            println!("{} is a prime number", n);
        } else {
            println!("{} is not a prime number", n);
        }
    }
}
