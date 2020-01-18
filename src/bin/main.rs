use std::io::{stdin, stdout, Write};

use shunting_yard::evaluator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        print!(">> ");
        stdout().flush()?;
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        match evaluator::eval_str(&input) {
            Ok(res) => println!("{}", res),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}