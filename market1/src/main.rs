pub mod inventory;
pub mod product;
pub mod repl;
pub mod warehouse;

use crate::repl::run;
use std::env::args;

fn main() {
    let args: Vec<String> = args().collect();
    if let Err(e) = run(args) {
        eprintln!("Error: {}", e);
    }
}
