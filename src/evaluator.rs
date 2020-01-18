use std::collections::HashMap;

use thiserror::Error;

use super::{
    parser::{self, parse, ParserToken},
    tokenizer::tokenize,
    Ctx,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Variable not found: {0}")]
    VarNotFound(String),
    #[error("Eval stack is empty during processing")]
    EmptyEvalStack,
    #[error("Parser: {0}")]
    ParserError(#[from] parser::Error),

    #[error("Arity of function {id} mismatched during evaluation: expected: {expected}, actual: {actual}")]
    ArityMismatch {
        id: String,
        expected: usize,
        actual: usize,
    },

    #[error("Ill formed token steam")]
    Other,
}
pub type Result = std::result::Result<f64, Error>;

fn eval_internal(
    tokens: &Vec<ParserToken>,
    variables: &mut HashMap<String, f64>,
    ctx: &Ctx,
) -> Result {
    let mut eval_stack: Vec<f64> = Vec::new();
    let mut iter = tokens.iter();
    while let Some(token) = iter.next() {
        match *token {
            ParserToken::Num(n) => {
                eval_stack.push(n);
            }
            ParserToken::Id(id) => {
                let value = variables
                    .get(id)
                    .ok_or_else(|| Error::VarNotFound(id.into()))?;
                eval_stack.push(*value);
            }
            ParserToken::UOp(op) => {
                let operand = eval_stack.pop().ok_or(Error::EmptyEvalStack)?;
                let func = op.func;
                eval_stack.push(func(operand));
            }
            ParserToken::BiOp(op) => {
                let right = eval_stack.pop().ok_or(Error::EmptyEvalStack)?;
                let left = eval_stack.pop().ok_or(Error::EmptyEvalStack)?;
                let func = op.func;
                let eval = func(left, right);
                eval_stack.push(eval);
            }
            ParserToken::Func(func, call_args) => {
                let arity = func.arity;
                if arity != 0 && arity != call_args {
                    return Err(Error::ArityMismatch {
                        id: func.token.clone(),
                        expected: arity,
                        actual: call_args,
                    });
                }
                let temp = &eval_stack[(eval_stack.len() - call_args)..];
                let eval = func.call(temp).expect(
                    "Number of actual arguments matches the number of params to the function",
                );
                for _ in 0..call_args {
                    eval_stack.pop();
                }
                eval_stack.push(eval);
            }
            ParserToken::Macro(ref m) => {
                m.eval(&mut eval_stack, variables, ctx)?;
            }
        }
    }
    eval_stack.pop().ok_or(Error::Other)
}

pub fn eval_with_vars(tokens: &Vec<ParserToken>, variables: &mut HashMap<String, f64>) -> Result {
    eval_internal(tokens, variables, &Ctx::default())
}

pub fn eval_str(input: &str) -> Result {
    eval_str_with_vars_and_ctx(input, &mut HashMap::new(), &Ctx::default())
}

pub fn eval_str_with_vars(input: &str, variables: &mut HashMap<String, f64>) -> Result {
    eval_str_with_vars_and_ctx(input, variables, &Ctx::default())
}

pub fn eval_str_with_vars_and_ctx(
    input: &str,
    variables: &mut HashMap<String, f64>,
    ctx: &Ctx,
) -> Result {
    let tokens = tokenize(input, ctx);
    let parsed = parse(&tokens, ctx)?;
    eval_internal(&parsed, variables, ctx)
}

#[cfg(test)]
mod tests {
    use crate::functions::FN_SUM;
    use crate::operators::binary::PLUS;

    use super::ParserToken::*;
    use super::*;

    // TODO: more tests cases
    #[test]
    fn test_eval() -> std::result::Result<(), Error> {
        let mut vars = HashMap::new();

        vars.insert("a".into(), 10.0);
        vars.insert("b".into(), 20.0);
        vars.insert("c".into(), 30.0);

        let expected = vec![1.0, 10.0, 15.0, 3.0];

        let input = vec![
            vec![Num(1.0)],
            vec![Id("a")],
            vec![Id("a"), Num(5.0), BiOp(&PLUS)],
            vec![Num(1.0), Num(1.0), Num(1.0), Func(&FN_SUM, 3)],
        ];

        for (expected, input) in expected.into_iter().zip(input) {
            let result = eval_with_vars(&input, &mut vars)?;
            assert_eq!(expected, result);
        }
        Ok(())
    }
}
