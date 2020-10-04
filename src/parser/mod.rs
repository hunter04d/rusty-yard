//! Parsing module
//!
//! Exposes a function and associated types that parse the [`Tokens`](crate::tokenizer::Token)
//! into the stream of [`ParserTokens`](ParserToken) in [reverse polish notation](https://en.wikipedia.org/wiki/Reverse_Polish_notation).
//!
//! The parser implementation uses the [`context`](crate::Ctx) to categorize input tokens of [`Token::Id`](crate::tokenizer::Token::Id) into VariableId, Function, Binary Operator and others.
pub use error::{Error, ErrorKind};
pub use token::ParserToken;
use ParseState::*;

use super::functions::Func;
use super::macros::{ApplyMode, ParsedMacro};
use super::operators::binary::Associativity;
use super::operators::{BiOp, UOp};
use super::tokenizer::{self, Token};
use super::Ctx;
use crate::macros::MacroParse;
use crate::Pos;

mod error;
mod token;

#[derive(Debug)]
enum OperatorStackValue<'a, 'ctx> {
    LeftParen,
    BiOp(&'ctx BiOp),
    UOp(&'ctx UOp),
    Func(&'ctx Func, usize),
    Macro(Box<dyn ParsedMacro + 'a>),
}

fn to_parser_token<'a, 'ctx>(
    sv: OperatorStackValue<'a, 'ctx>,
) -> Result<ParserToken<'a, 'ctx>, &'static str> {
    use OperatorStackValue::*;
    match sv {
        LeftParen => Err("Left Parent cannot be in output queue"),
        BiOp(b) => Ok(ParserToken::BiOp(b)),
        UOp(u) => Ok(ParserToken::UOp(u)),
        Func(f, n_args) => Ok(ParserToken::Func(f, n_args)),
        Macro(m) => Ok(ParserToken::Macro(m)),
    }
}

/// Represents the current state of the parser
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ParseState {
    /// The state signaling that the next token is expected to be an expression
    Expression,
    /// The state signaling that the next token is expected to be an operator
    Operator,
}

impl ParseState {
    fn expect(self, state_to_expect: ParseState, pos: Pos) -> Result<(), Error> {
        if self == state_to_expect {
            Ok(())
        } else if let Expression = self {
            Err(Error {
                pos,
                kind: ErrorKind::ExpectedExpression,
            })
        } else {
            Err(Error {
                pos,
                kind: ErrorKind::ExpectedOperator,
            })
        }
    }
}

/// Parses the input tokens into steam of [`ParserTokens`](ParserToken) in Reverse polish notation order
pub fn parse<'a, 'ctx>(
    tokens: &[Token<'a, 'ctx>],
    ctx: &'ctx Ctx,
) -> Result<Vec<ParserToken<'a, 'ctx>>, Error> {
    if tokens.is_empty() {
        return Ok(Vec::new());
    }
    let mut queue = Vec::new();
    let mut operator_stack: Vec<OperatorStackValue> = Vec::new();
    let mut parse_state: ParseState = Expression;
    let mut iter = tokens
        .iter()
        .enumerate()
        .map(|(i, t)| (Pos(i), t))
        .peekable();
    while let Some((pos, current_token)) = iter.next() {
        match &*current_token {
            Token::Num(num) => {
                parse_state.expect(Expression, pos)?;
                parse_state = Operator;
                queue.push(ParserToken::Num(*num));
            }
            Token::Id(id) => {
                if let Some(u_op) = find_uop(ctx, id, parse_state) {
                    operator_stack.push(OperatorStackValue::UOp(u_op));
                } else if let Some(bi_op) = find_biop(ctx, id) {
                    parse_state.expect(Operator, pos)?;
                    push_to_output(&mut queue, &mut operator_stack, bi_op);
                    parse_state = Expression;
                    operator_stack.push(OperatorStackValue::BiOp(bi_op));
                } else if let Some(func) = find_func(ctx, id, parse_state) {
                    if let Some((_, Token::OpenParen)) = iter.peek() {
                        operator_stack.push(OperatorStackValue::Func(func, 0usize))
                    } else {
                        // TODO v0.3: might be better to match id, to that fn(), and fn are different
                        return Err(ErrorKind::NoLeftParenAfterFnId.with_pos(pos));
                    }
                } else {
                    // variable
                    parse_state.expect(Expression, pos)?;
                    parse_state = Operator;
                    queue.push(ParserToken::Id(id));
                }
            }
            Token::OpenParen => {
                parse_state.expect(Expression, pos)?;
                operator_stack.push(OperatorStackValue::LeftParen);
            }
            Token::ClosedParen => {
                if parse_state == Expression {
                    // operator or left parent or empty parens
                    // (10 + )
                    // |-----^
                    // |or
                    // |<fn_name>()
                    // |----------^
                    // |or
                    // |()
                    // |-^
                    // |
                    // =we are here

                    // pop the left paren
                    if let Some(OperatorStackValue::LeftParen) = operator_stack.pop() {
                        if let Some(OperatorStackValue::Func(_, _)) = operator_stack.last() {
                            let func_token =
                                to_parser_token(operator_stack.pop().unwrap()).unwrap();
                            queue.push(func_token);
                        } else {
                            return Err(ErrorKind::EmptyParensNotFnCall.with_pos(pos));
                        }
                    } else {
                        // operator before right paren is an error
                        return Err(ErrorKind::OperatorAtTheEnd.with_pos(pos));
                    }
                } else {
                    let found_left_paren = pop_operator_stack(&mut operator_stack, &mut queue)
                        .map_err(|e| e.with_pos(pos))?;
                    if !found_left_paren {
                        return Err(ErrorKind::MismatchedRightParen.with_pos(pos));
                    }
                    if let Some(OperatorStackValue::Func(_, n_args)) = operator_stack.last_mut() {
                        *n_args += 1;
                    }
                }
                parse_state = Operator;
            }
            Token::Comma => {
                parse_state.expect(Operator, pos)?;
                parse_state = Expression;
                let found_left_paren = pop_operator_stack(&mut operator_stack, &mut queue)
                    .map_err(|e| e.with_pos(pos))?;
                match operator_stack.last_mut() {
                    Some(OperatorStackValue::Func(_, n_args)) if found_left_paren => {
                        *n_args += 1;
                        // return left paren into the stack
                        operator_stack.push(OperatorStackValue::LeftParen);
                    }
                    _ => {
                        return Err(ErrorKind::CommaOutsideFn.with_pos(pos));
                    }
                }
            }
            Token::Macro(m) => {
                let MacroParse {
                    result,
                    mode,
                    state_after,
                } = m
                    .definition
                    .parse(m.text, ctx, parse_state)
                    .map_err(|e| e.with_pos(pos))?;
                parse_state = state_after;
                match mode {
                    ApplyMode::Before => queue.push(ParserToken::Macro(result)),
                    ApplyMode::After => operator_stack.push(OperatorStackValue::Macro(result)),
                };
            }
            Token::BadToken(token) => {
                return Err(ErrorKind::BadToken(String::from(*token)).with_pos(pos));
            }
        }
    }
    let end_pos = Pos(tokens.len() - 1);
    if let Expression = parse_state {
        return Err(ErrorKind::OperatorAtTheEnd.with_pos(end_pos));
    }
    let found_left_paren =
        pop_operator_stack(&mut operator_stack, &mut queue).map_err(|e| e.with_pos(end_pos))?;
    if found_left_paren {
        Err(ErrorKind::MismatchedLeftParen.with_pos(end_pos))
    } else {
        Ok(queue)
    }
}

fn push_to_output<'a, 'ctx>(
    queue: &mut Vec<ParserToken<'a, 'ctx>>,
    operator_stack: &mut Vec<OperatorStackValue<'a, 'ctx>>,
    b_op: &BiOp,
) {
    while let Some(top_of_stack) = operator_stack.last() {
        match *top_of_stack {
            OperatorStackValue::UOp(op) => {
                queue.push(ParserToken::UOp(op));
                operator_stack.pop();
            }
            OperatorStackValue::BiOp(op)
                if op.precedence > b_op.precedence
                    || (op.precedence == b_op.precedence
                        && op.associativity == Associativity::LEFT) =>
            {
                let pt = op.into();
                queue.push(pt);
                operator_stack.pop();
            }
            _ => {
                break;
            }
        }
    }
}

/// Parses the input string into a stream of [`ParsedTokens`](ParserToken).
///
/// This tokenizes the input first using [`tokenizer::tokenize`](crate::tokenizer::tokenize)
/// and then parses it using [`parse`](parse).
///
/// This uses the ctx provided as the last parameter.
#[cfg_attr(tarpaulin, skip)]
pub fn parse_str<'a, 'ctx>(
    input: &'a str,
    ctx: &'ctx Ctx,
) -> Result<Vec<ParserToken<'a, 'ctx>>, Error> {
    let tokens = tokenizer::tokenize(input, ctx);
    parse(&tokens, ctx)
}

fn check_arity(token: &ParserToken) -> Result<(), ErrorKind> {
    if let ParserToken::Func(func, n_args) = token {
        if let Some(arity) = func.arity {
            if arity != *n_args {
                return Err(ErrorKind::ArityMismatch {
                    id: func.token.to_owned(),
                    expected: arity,
                    actual: *n_args,
                });
            }
        }
    }
    Ok(())
}

fn pop_operator_stack<'a, 'ctx>(
    operator_stack: &mut Vec<OperatorStackValue<'a, 'ctx>>,
    queue: &mut Vec<ParserToken<'a, 'ctx>>,
) -> Result<bool, ErrorKind> {
    while let Some(v) = operator_stack.pop() {
        if let OperatorStackValue::LeftParen = v {
            return Ok(true);
        }
        // unwrap: safe because operator stack value is never LeftParen
        let token = to_parser_token(v).unwrap();
        check_arity(&token)?;
        queue.push(token);
    }
    Ok(false)
}

#[inline]
fn find_biop<'a>(ctx: &'a Ctx, id: &str) -> Option<&'a BiOp> {
    ctx.bi_ops.iter().find(|op| op.token == id)
}

#[inline]
fn find_uop<'a>(ctx: &'a Ctx, id: &str, parse_state: ParseState) -> Option<&'a UOp> {
    let u_op = ctx.u_ops.iter().find(|op| op.token == id)?;
    match parse_state {
        Expression => Some(u_op),
        Operator => None,
    }
}

#[inline]
fn find_func<'a>(ctx: &'a Ctx, id: &str, parse_state: ParseState) -> Option<&'a Func> {
    let func = ctx.fns.iter().find(|op| op.token == id)?;
    match parse_state {
        Expression => Some(func),
        Operator => None, // does this make sense?
    }
}

#[cfg(test)]
mod tests {
    use crate::operators;

    use super::ParserToken::*;
    use super::*;

    fn get_biop() -> operators::BiOp {
        operators::BiOp {
            token: "bi_op".to_owned(),
            precedence: 0,
            associativity: Associativity::LEFT,
            func: |_1, _2| 0.0,
        }
    }

    fn get_uop() -> operators::UOp {
        operators::UOp {
            token: "u_op".to_owned(),
            func: |_arg| 0.0,
        }
    }
    fn get_ctx() -> Ctx {
        let mut ctx = Ctx::empty();
        ctx.bi_ops.push(get_biop());
        ctx.u_ops.push(get_uop());
        ctx
    }

    // TODO: more tests cases
    #[test]
    fn test_parse() -> Result<(), ErrorKind> {
        let bi_op = get_biop();
        let u_op = get_uop();
        let ctx = get_ctx();
        let input_expected = &[
            (
                vec![Token::Num(10.0), Token::Id("bi_op"), Token::Id("10")],
                vec![Num(10.0), Id("10"), BiOp(&bi_op)],
            ),
            (
                vec![Token::Id("u_op"), Token::Num(10.0)],
                vec![Num(10.0), UOp(&u_op)],
            ),
            (
                vec![
                    Token::Id("a"),
                    Token::Id("bi_op"),
                    Token::Id("b"),
                    Token::Id("bi_op"),
                    Token::Id("c"),
                ],
                vec![Id("a"), Id("b"), BiOp(&bi_op), Id("c"), BiOp(&bi_op)],
            ),
        ];
        for (input, expected) in input_expected {
            let actual = parse(&input, &ctx).expect("Parse succeeded");
            assert_eq!(actual, *expected, "input was, {:?}", input);
        }
        Ok(())
    }

    #[test]
    fn test_parse_bad_token() {
        let s = "\x00".to_owned();
        let ctx = &get_ctx();
        let result = parse(&[Token::BadToken(&s)], &ctx).unwrap_err();
        assert_eq!(
            std::mem::discriminant(&result),
            std::mem::discriminant(&ErrorKind::BadToken(s))
        );
    }
}
