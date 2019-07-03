mod shunting_yard;

use std::collections::HashMap;
use crate::shunting_yard::{evaluator, Ctx};
use crate::shunting_yard::parser::parse;
use crate::shunting_yard::tokenizer::tokenize;

fn main() {
    let ctx = Ctx::default();
    let output = tokenize("-(1) + 1", &ctx);
    println!("{:?}", output);
    let output = parse(&output, &ctx).unwrap();
    println!("{:?}", output);
    let mut vars = HashMap::new();
    vars.insert("bla".to_owned(), 1.0);
    let output = evaluator::eval_with_vars(&output, &vars);
    println!("res = {}", output.unwrap());
}
