use std::collections::{VecDeque, HashMap, HashSet};
use crate::shunting_yard::binary_operators::BiOp;
use crate::shunting_yard::{binary_operators, Ctx};
use crate::shunting_yard::unary_operators::UOp;
use crate::shunting_yard::unary_operators;
use crate::shunting_yard::parser::ParserToken;

#[derive(Debug)]
pub struct Error(String);

fn report_other() -> String {
    "ill formed token stream".to_owned()
}

fn report_not_found(v: &str) -> String {
    format!("Variable not found {}", v)
}

pub fn eval_with(tokens: &VecDeque<ParserToken>, variables: &HashMap<String, f64>) -> Result<f64, Error> {
    let mut eval_stack: Vec<f64> = Vec::new();
    let mut iter = tokens.iter();
    while let Some(token) = iter.next() {
        match token {
            ParserToken::Num(n) => {
                eval_stack.push(*n);
            }
            ParserToken::Id(id) => {
                let value = variables
                    .get(id)
                    .ok_or(Error(report_not_found(id)))?;
                eval_stack.push(*value);
            }
            ParserToken::UOp(op) => {
                let operand = eval_stack.pop().ok_or(Error(report_other()))?;
                let func = op.func;
                eval_stack.push(func(operand));
            }
            ParserToken::BiOp(op) => {
                let right = eval_stack.pop().ok_or(Error(report_other()))?;
                let left = eval_stack.pop().ok_or(Error(report_other()))?;
                let func = op.func;
                let eval = func(left, right);
                eval_stack.push(eval);
            }
        }
    }
    eval_stack.pop().ok_or(Error("ill formed token stream".to_owned()))
}

#[allow(dead_code)]
pub fn eval_with_vars(tokens: &VecDeque<ParserToken>, variables: &HashMap<String, f64>) -> Result<f64, Error> {
    eval_with(tokens, variables)
}
