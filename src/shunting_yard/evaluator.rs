use super::parser::ParserToken;
use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
pub struct Error(String);

fn report_other() -> Error {
    Error("ill formed token stream".to_owned())
}

fn report_not_found(v: &str) -> Error {
    Error(format!("Variable not found {}", v))
}

pub fn eval_with(
    tokens: &VecDeque<ParserToken>,
    variables: &HashMap<String, f64>,
) -> Result<f64, Error> {
    let mut eval_stack: Vec<f64> = Vec::new();
    let mut iter = tokens.iter();
    while let Some(token) = iter.next() {
        match *token {
            ParserToken::Num(n) => {
                eval_stack.push(n);
            }
            ParserToken::Id(ref id) => {
                let value = variables.get(id).ok_or_else(|| (report_not_found(id)))?;
                eval_stack.push(*value);
            }
            ParserToken::UOp(op) => {
                let operand = eval_stack.pop().ok_or_else(report_other)?;
                let func = op.func;
                eval_stack.push(func(operand));
            }
            ParserToken::BiOp(op) => {
                let right = eval_stack.pop().ok_or_else(report_other)?;
                let left = eval_stack.pop().ok_or_else(report_other)?;
                let func = op.func;
                let eval = func(left, right);
                eval_stack.push(eval);
            }
            ParserToken::Func(func, call_args) => {
                let arity = func.arity;
                if arity != 0 && arity != call_args {
                    return Err(Error("function arity and call_args differ".to_owned()));
                }
                let func = func.func;
                // TODO: variable length functions
                let temp = &eval_stack[eval_stack.len() - call_args..];
                let eval = func(temp);
                for _ in 0..call_args {
                    eval_stack.pop();
                }
                eval_stack.push(eval);
            }
        }
    }
    eval_stack
        .pop()
        .ok_or(Error("ill formed token stream".to_owned()))
}

#[allow(dead_code)]
pub fn eval_with_vars(
    tokens: &VecDeque<ParserToken>,
    variables: &HashMap<String, f64>,
) -> Result<f64, Error> {
    eval_with(tokens, variables)
}
