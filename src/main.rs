mod tokenizer;
mod shunting_yard;
mod evaluator;

use tokenizer::tokenize;
use crate::shunting_yard::parse;
use std::collections::HashMap;

fn main() {
    let output = tokenize("bla + (2 + 2)");
    println!("{:?}", output);
    let output = parse(&output).unwrap();
    println!("{:?}", output);
    let mut vars = HashMap::new();
    vars.insert("bla".to_owned(), 1.0);
    let output = evaluator::eval_with(output, &vars);
    println!("res = {}", output.unwrap());
}
