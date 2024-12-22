use rug::Integer;
use std::{io, io::prelude::*, time::Instant};

fn main() {
    fn calculate_fibonacci(n: u32) -> String {
        let mut table: Vec<Integer> = Vec::new();
        if table.len() == 0 {
            table.push(Integer::from(0));
            table.push(Integer::from(1));
        }
        while table.len() <= n as usize {
            let len = table.len();
            let next = &table[len - 1] + &table[len - 2];
            table.push(Integer::from(next));
        }
        return table[n as usize].to_string();
    }

    loop {
        print!("Enter a number to calculate the fibonacci number for: ");
        io::stdout().flush().unwrap();
        let mut number = String::new();

        io::stdin()
            .read_line(&mut number)
            .expect("Failed to read line");

        let number: u32 = match number.trim().parse() {
            Ok(num) => num,
            Err(_) => return,
        };

        let time = Instant::now();
        let fibonacci_number: String = calculate_fibonacci(number);
        let elapsed = time.elapsed();
        println!(
            "The fibonacci number is: {}, calculated in {:.2?}",
            fibonacci_number, elapsed
        );
    }
}
