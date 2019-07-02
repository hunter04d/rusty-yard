use std::collections::{VecDeque, HashMap};
use crate::shunting_yard::ParserToken;
use crate::shunting_yard::binary_operator::BinaryOperator;

#[derive(Debug)]
pub struct Error(String);


pub fn eval_with(mut tokens: VecDeque<ParserToken>, variables: &HashMap<String, f64>) -> Result<f64, Error> {
    let mut eval_stack = Vec::new();
    while let Some(token) = tokens.pop_front() {
        match token {
            ParserToken::Number(n) => {
                eval_stack.push(n);
            }
            ParserToken::ID(id) => {
                let value = variables
                    .get(&id)
                    .ok_or(Error(format!("Variable not found {}", id)))?
                    .clone();
                eval_stack.push(value);
            }
            ParserToken::UnaryOperator(_) => { unimplemented!("Unary operators") }
            ParserToken::BinaryOperator(op) => {
                let right = eval_stack.pop().ok_or(Error("ill formed token stream".to_owned()))?;
                let left = eval_stack.pop().ok_or(Error("ill formed token stream".to_owned()))?;
                let value = match op {
                    BinaryOperator::PLUS => left + right,
                    BinaryOperator::MINUS => left - right,
                    BinaryOperator::MULTIPLY => left * right,
                    BinaryOperator::DIVIDE => left / right,
                };
                eval_stack.push(value);
            }
        }
    }
    eval_stack.pop().ok_or(Error("ill formed token stream".to_owned()))
}

pub fn eval(tokens: VecDeque<ParserToken>) -> Result<f64, Error> {
    eval_with(tokens, &HashMap::new())
}