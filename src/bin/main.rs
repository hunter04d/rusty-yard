use std::collections::HashMap;
use std::io::{stdin, stdout, Write};

use rusty_yard::functions::Func;
use rusty_yard::{evaluator, parser, tokenizer, Ctx};

#[cfg_attr(tarpaulin, skip)]
/// Simple read, eval, print loop
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vars = HashMap::new();
    let mut ctx = Ctx::default_with_macros();
    ctx.fns.push(Func {
        token: "pi".to_string(),
        arity: Some(0),
        func: |_| std::f64::consts::PI,
    });

    loop {
        print!("> ");
        stdout().flush()?;
        let mut input = String::new();
        stdin().read_line(&mut input)?;

        let tokens = tokenizer::tokenize(&input, &ctx);
        let parsed = parser::parse(&tokens, &ctx);
        match parsed {
            Ok(tokens) => {
                let result = evaluator::eval_with_vars_and_ctx(&tokens, &mut vars, &ctx);
                match result {
                    Ok(n) => println!("{}", n),
                    Err(e) => println!("{}", e),
                }
            }
            Err(pe) => pe.report_to(&mut stdout(), &tokens)?,
        }
    }
}
