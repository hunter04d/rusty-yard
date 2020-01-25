//! Provides high and low level api for expression execution.
//!
//! Functions in this module are using the [`context`](crate::Ctx) only for macro support,
//! because macros are allowed to query the contents of the context.
//! In most use cases it is possible to just pass [`empty`](crate::Ctx::empty) or [`default`](std::default::Default) context.
//!
//! # Example
//!
//! Here is a high level example:
//!
//! ```
//! use rusty_yard::evaluator::eval_str;
//! assert_eq!(eval_str("10 + 10 * 10"), Ok(110.0));
//! ```
//!

#![deny(missing_docs)]
use std::collections::HashMap;

use thiserror::Error;

use super::parser::{self, parse, ParserToken};
use super::tokenizer::tokenize;
use super::Ctx;

/// Represents the Error that can occur during the evaluation of the expression
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// Signifies that variable was not found in variable map
    #[error("Variable not found: {0}")]
    VarNotFound(String),
    /// Signifies that evaluation stack has empty when a value was expected
    #[error("Eval stack is empty during processing")]
    EmptyEvalStack,

    /// Signifies that an error occurred during expression parsing
    ///
    ///# Note
    ///
    ///This is only the case when one of the `eval_str` functions is called.
    #[error("Parser: {0}")]
    ParserError(#[from] parser::Error),

    /// Signifies that a function has been called with different number of parameters than expected
    ///
    /// # Note
    ///
    /// This error is likely picked up in ParserError case, however it still can occur if you pass the tokens manually to one of `eval` functions.
    #[error("Arity of function {id} mismatched during evaluation: expected: {expected}, actual: {actual}")]
    ArityMismatch {
        /// Identifier of the mismatched function
        id: String,
        /// Expected number of parameters to the function
        expected: usize,
        /// Actual number of parameters passed to the function
        actual: usize,
    },

    /// Catch-all case when something unexpected happened
    #[error("Ill formed token steam")]
    Other,
}

/// Result type of this module with [`evaluator::Error`](Error) as Error type
pub type Result = std::result::Result<f64, Error>;

/// The main evaluation logic
fn eval_internal(
    tokens: &[ParserToken],
    variables: &mut HashMap<String, f64>,
    ctx: &Ctx,
) -> Result {
    let mut eval_stack: Vec<f64> = Vec::new();
    for token in tokens {
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

/// Evaluate the input token stream and return the result of the evaluation.
///
/// Tokens can be produced by [`parse`](crate::parser::parse) or [`parse_str`](crate::parser::parse_str) function.
///
/// This uses the default context from `Ctx::default`.
/// # Note
///
/// Tokens need to be in [reverse polish notation](https://en.wikipedia.org/wiki/Reverse_Polish_notation).
///
/// # Example
///
/// ```
/// use rusty_yard::evaluator::eval;
/// use rusty_yard::parser::ParserToken;
/// use rusty_yard::operators::binary::PLUS;
///
/// let  result = eval(&[ParserToken::Num(3.0), ParserToken::Num(4.0), ParserToken::BiOp(&PLUS)]);
/// assert_eq!(result, Ok(7.0));
/// ```
#[cfg_attr(tarpaulin, skip)]
#[inline]
pub fn eval(tokens: &[ParserToken]) -> Result {
    eval_internal(tokens, &mut HashMap::new(), &Ctx::default())
}

/// Evaluate the input token stream with variables defined in `variables`.
///
/// Tokens can be produced by [`parse`](crate::parser::parse) or [`parse_str`](crate::parser::parse_str) function.
///
/// This uses the default context from `Ctx::default`.
///
/// # Note
///
/// Tokens need to be in [reverse polish notation](https://en.wikipedia.org/wiki/Reverse_Polish_notation).
///
/// # Example
///
/// ```
/// use rusty_yard::evaluator::eval_with_vars;
/// use rusty_yard::parser::ParserToken;
/// use rusty_yard::operators::binary::PLUS;
/// use std::collections::HashMap;
///
/// let mut vars = HashMap::new();
/// vars.insert("a".to_owned(), 3.0);
/// vars.insert("b".to_owned(), 4.0);
/// let result = eval_with_vars(&[ParserToken::Id("a"), ParserToken::Id("b"), ParserToken::BiOp(&PLUS)], &mut vars);
/// assert_eq!(result, Ok(7.0));
/// ```
#[cfg_attr(tarpaulin, skip)]
#[inline]
pub fn eval_with_vars(tokens: &[ParserToken], variables: &mut HashMap<String, f64>) -> Result {
    eval_internal(tokens, variables, &Ctx::default())
}

/// Evaluate the input token stream with variables defined in `variables` and custom [context](crate::Ctx).
///
/// Tokens can be produced by [`parse`](crate::parser::parse) or [`parse_str`](crate::parser::parse_str) function.
///
/// This uses the Context provided as input.
///
/// # Note
///
/// Tokens need to be in [reverse polish notation](https://en.wikipedia.org/wiki/Reverse_Polish_notation).
///
/// # Note
///
/// Ctx is mainly used during parsing. Since expression has already been parsed, ctx parameter is mostly useless (it can still be used in macros).
///
/// # Example
///
/// ```
/// use rusty_yard::evaluator::eval_with_vars_and_ctx;
/// use rusty_yard::parser::ParserToken;
/// use rusty_yard::operators::binary::PLUS;
/// use std::collections::HashMap;
/// use rusty_yard::Ctx;
/// use rusty_yard::macros::default::AssignParsed;
///
/// // use ctx that has default macros
/// let ctx = Ctx::default_with_macros();
/// let mut vars = HashMap::new();
/// vars.insert("a".to_owned(), 3.0);
/// let result = eval_with_vars_and_ctx(&[ParserToken::Num(7.0), ParserToken::Macro(Box::new(AssignParsed::new("a")))], &mut vars, &ctx);
/// assert_eq!(result, Ok(7.0));
/// assert_eq!(vars["a"], 7.0);
/// ```
#[cfg_attr(tarpaulin, skip)]
#[inline]
pub fn eval_with_vars_and_ctx(
    tokens: &[ParserToken],
    variables: &mut HashMap<String, f64>,
    ctx: &Ctx,
) -> Result {
    eval_internal(tokens, variables, ctx)
}

/// Evaluate the string with the expression inside
///
/// This uses the default context from `Ctx::default`
///
/// # Example
///
/// ```
/// use rusty_yard::evaluator:: eval_str;
/// use std::collections::HashMap;
///
/// let result = eval_str("3 + 4");
/// assert_eq!(result, Ok(7.0));
/// ```
#[cfg_attr(tarpaulin, skip)]
#[inline]
pub fn eval_str(input: &str) -> Result {
    eval_str_with_vars_and_ctx(input, &mut HashMap::new(), &Ctx::default())
}

/// Evaluate the string with the expression inside with variables defined in `variables`
///
/// This uses the default context from `Ctx::default`
///
/// # Example
///
/// ```
/// use rusty_yard::evaluator::eval_str_with_vars;
/// use std::collections::HashMap;
///
/// let mut vars = HashMap::new();
/// vars.insert("a".to_owned(), 3.0);
/// vars.insert("b".to_owned(), 4.0);
/// let result = eval_str_with_vars("a + b", &mut vars);
/// assert_eq!(result, Ok(7.0));
/// ```
#[cfg_attr(tarpaulin, skip)]
#[inline]
pub fn eval_str_with_vars(input: &str, variables: &mut HashMap<String, f64>) -> Result {
    eval_str_with_vars_and_ctx(input, variables, &Ctx::default())
}

/// Evaluate the input token stream with variables defined in `variables` and custom [context](crate::Ctx)..
///
/// This uses the Context provided as the last parameter.
///
/// # Example
///
/// ```
/// use rusty_yard::evaluator::eval_str_with_vars_and_ctx;
/// use rusty_yard::parser::ParserToken;
/// use rusty_yard::operators::binary::PLUS;
/// use std::collections::HashMap;
/// use rusty_yard::Ctx;
/// use rusty_yard::macros::default::AssignParsed;
///
/// // use ctx that has default macros
/// let ctx = Ctx::default_with_macros();
/// let mut vars = HashMap::new();
/// let result = eval_str_with_vars_and_ctx("a = 7.0", &mut vars, &ctx);
/// assert_eq!(result, Ok(7.0));
/// assert_eq!(vars["a"], 7.0);
/// ```
#[cfg_attr(tarpaulin, skip)]
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
    use crate::functions::{FN_SUB, FN_SUM};
    use crate::operators::{binary::PLUS as B_PLUS, unary::PLUS as U_PLUS};

    use super::ParserToken::*;
    use super::*;

    // TODO: more tests cases
    #[test]
    fn test_eval() {
        let mut vars = HashMap::new();

        vars.insert("a".into(), 10.0);
        vars.insert("b".into(), 20.0);
        vars.insert("c".into(), 30.0);

        let input_expected = &[
            (vec![Num(1.0)], Ok(1.0)),
            (vec![Id("a")], Ok(10.0)),
            (vec![Id("a"), Num(5.0), BiOp(&B_PLUS)], Ok(15.0)),
            (
                vec![Num(1.0), Num(1.0), Num(1.0), Func(&FN_SUM, 3)],
                Ok(3.0),
            ),
            (vec![Num(1.0), UOp(&U_PLUS)], Ok(1.0)),
            (
                vec![Num(2.0), Num(1.0), Func(&FN_SUB, 1)],
                Err(Error::ArityMismatch {
                    id: "sub".to_owned(),
                    expected: 2,
                    actual: 1,
                }),
            ),
        ];

        for (input, expected) in input_expected {
            let result = eval_with_vars(&input, &mut vars);
            assert_eq!(result, *expected);
        }
    }
}
