use super::parser::ParserToken;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Error(String);

fn report_other() -> Error {
    Error("ill formed token stream".to_owned())
}

fn report_not_found(v: &str) -> Error {
    Error(format!("Variable not found {}", v))
}

fn eval_internal(
    tokens: &Vec<ParserToken>,
    variables: &HashMap<String, f64>,
) -> Result<f64, Error> {
    let mut eval_stack: Vec<f64> = Vec::new();
    let mut iter = tokens.iter();
    while let Some(token) = iter.next() {
        match *token {
            ParserToken::Num(n) => {
                eval_stack.push(n);
            }
            ParserToken::Id(id) => {
                let value = variables.get(id).ok_or_else(|| report_not_found(id))?;
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

pub fn eval_with_vars(
    tokens: &Vec<ParserToken>,
    variables: &HashMap<String, f64>,
) -> Result<f64, Error> {
    eval_internal(tokens, variables)
}

#[cfg(test)]
mod tests {
    use super::ParserToken::*;
    use super::*;
    use crate::shunting_yard::operators::binary::{PLUS};

    // TODO: more test cases
    #[test]
    fn test_eval() -> Result<(), Error> {
        let mut vars = HashMap::new();

        vars.insert("a".into(), 10.0);
        vars.insert("b".into(), 20.0);
        vars.insert("c".into(), 30.0);

        let expected = vec![1.0, 10.0, 15.0];

        let input = vec![vec![Num(1.0)], vec![Id("a")],
        vec![Id("a"), Num(5.0), BiOp(&PLUS)]];

        for (expected, input) in expected.into_iter().zip(input) {
            let result = eval_with_vars(&input, &vars)?;
            assert_eq!(expected, result);
        }
        Ok(())
    }
}
