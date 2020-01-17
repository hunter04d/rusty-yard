mod shunting_yard;

use shunting_yard::{evaluator, parser::parse, tokenizer::tokenize, Ctx};
use std::collections::HashMap;

fn main() {
    let ctx = Ctx::default();
    let output = tokenize("max(1, 2)", &ctx);
    println!("{:?}", output);
    let output = parse(&output, &ctx).unwrap();
    println!("{:?}", output);
    let mut vars = HashMap::new();
    vars.insert("bla".to_string(), 1.0);
    let output = evaluator::eval_with_vars(&output, &vars).unwrap();
    println!("res = {}", output);
}
