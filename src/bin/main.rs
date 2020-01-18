use std::collections::HashMap;
use std::io::{stdin, stdout, Write};

use shunting_yard::{evaluator, Ctx};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vars = HashMap::new();
    let ctx = Ctx::default_with_macros();
    loop {
        print!(">> ");
        stdout().flush()?;
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        match evaluator::eval_str_with_vars_and_ctx(&input, &mut vars, &ctx) {
            Ok(res) => println!("{}", res),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
