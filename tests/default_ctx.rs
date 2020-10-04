use std::collections::HashMap;

use rusty_yard::evaluator::{eval_str_with_vars_and_ctx, Error::*};
use rusty_yard::functions::default_functions;
use rusty_yard::operators::{
    binary::{self, PLUS},
    unary,
};
use rusty_yard::parser;
use rusty_yard::Ctx;

#[inline]
fn bi_op_ctx() -> Ctx {
    let mut ctx = Ctx::empty();
    ctx.bi_ops = binary::default_operators();
    ctx
}

fn u_op_ctx() -> Ctx {
    let mut ctx = Ctx::empty();
    ctx.u_ops = unary::default_operators();
    ctx
}

fn func_ctx() -> Ctx {
    let mut ctx = Ctx::empty();
    ctx.fns = default_functions();
    ctx
}

#[inline]
fn vars() -> HashMap<String, f64> {
    let map = HashMap::new();
    map
}

#[test]
fn test_evaluation_results_bi_ops() {
    let ctx = bi_op_ctx();
    let mut vars = vars();
    let input_expected_pair = &[
        ("1 + 1", Ok(2.0)),
        ("1 - 1", Ok(0.0)),
        ("1 * 2 + 1", (Ok(3.0))),
        ("1 + 1 * 2", Ok(3.0)),
        ("1 + 1 + 1", Ok(3.0)),
        ("2 ^ 3 ^ 2", Ok(512.0)),
        ("1 * 1", Ok(1.0)),
        ("1 / 1", Ok(1.0)),
        ("1 ^ 1", Ok(1.0)),
        ("1 + 1 + 1", Ok(3.0)),
        ("1 * (2 + 2)", Ok(4.0)),
        ("(2 ^ 3) ^ 2", Ok(64.0)),
        ("", Err(Other)),
        (
            "1 + ",
            Err(ParserError(parser::ErrorKind::OperatorAtTheEnd)),
        ),
        (
            "+ 1",
            Err(ParserError(parser::ErrorKind::ExpectedExpression)),
        ),
        ("1 1", Err(ParserError(parser::ErrorKind::ExpectedOperator))),
        ("a a", Err(ParserError(parser::ErrorKind::ExpectedOperator))),
        ("1 a", Err(ParserError(parser::ErrorKind::ExpectedOperator))),
        ("a 1", Err(ParserError(parser::ErrorKind::ExpectedOperator))),
        (
            "1 + + 1",
            Err(ParserError(parser::ErrorKind::ExpectedExpression)),
        ),
    ];

    for (input, expected) in input_expected_pair {
        let result = eval_str_with_vars_and_ctx(input, &mut vars, &ctx);
        assert_eq!(result, *expected, "input was: {}", input);
    }
}

#[test]
fn test_evaluation_results_u_ops() {
    let ctx = u_op_ctx();
    let mut vars = vars();
    let input_expected_pair = &[
        ("+1", Ok(1.0)),
        ("++1", Ok(1.0)),
        ("+++1", Ok(1.0)),
        ("-1", Ok(-1.0)),
        ("--1", Ok(1.0)),
        ("---1", Ok(-1.0)),
        ("+-1", Ok(-1.0)),
        ("-+1", Ok(-1.0)),
        ("-+-1", Ok(1.0)),
        ("+-+1", Ok(-1.0)),
    ];
    for (input, expected) in input_expected_pair {
        let result = eval_str_with_vars_and_ctx(input, &mut vars, &ctx);
        assert_eq!(result, *expected, "input was: {}", input);
    }
}

#[test]
fn test_evaluation_results_funcs() {
    let mut ctx = func_ctx();
    ctx.bi_ops.push(PLUS.clone());
    let mut vars = vars();
    let input_expected_pair = &[
        ("max(1, 2)", Ok(2.0)),
        ("max(2, 1)", Ok(2.0)),
        ("sum()", Ok(0.0)),
        ("sum(1)", Ok(1.0)),
        ("sum(1, 1)", Ok(2.0)),
        ("sum(1, 1, 1)", Ok(3.0)),
        ("prod()", Ok(1.0)),
        ("prod(1)", Ok(1.0)),
        ("prod(1, 1)", Ok(1.0)),
        ("prod(1, 1, 1)", Ok(1.0)),
        ("sub(2, 1)", Ok(1.0)),
        //TODO: v0.3 this should change
        (
            "sum + 10",
            Err(ParserError(parser::ErrorKind::NoLeftParenAfterFnId)),
        ),
        (
            "sum(10 + )",
            Err(ParserError(parser::ErrorKind::OperatorAtTheEnd)),
        ),
        (
            "sub(1)",
            Err(ParserError(parser::ErrorKind::ArityMismatch {
                id: "sub".to_owned(),
                expected: 2,
                actual: 1,
            })),
        ),
        (
            "(1 + 1))",
            Err(ParserError(parser::ErrorKind::MismatchedRightParen)),
        ),
        (
            "((1 + 1)",
            Err(ParserError(parser::ErrorKind::MismatchedLeftParen)),
        ),
    ];
    for (input, expected) in input_expected_pair {
        let result = eval_str_with_vars_and_ctx(input, &mut vars, &ctx);
        assert_eq!(result, *expected, "input was: {}", input);
    }
}

#[test]
fn test_evaluation_results_all() {
    let ctx = Ctx::default();
    let mut vars = vars();
    let input_expected_pair = &[
        ("+1 + +2 + +3", Ok(6.0)),
        ("+1 + +2 * +3", Ok(7.0)),
        ("-+-1 + +-+2 * -3", Ok(7.0)),
    ];
    for (input, expected) in input_expected_pair {
        let result = eval_str_with_vars_and_ctx(input, &mut vars, &ctx);
        assert_eq!(result, *expected, "input was: {}", input);
    }
}
