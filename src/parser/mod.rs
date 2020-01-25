//! Parsing module
//!
//! Exposes a function and associated types that parse the [`Tokens`](crate::tokenizer::Token)
//! into the stream of [`ParserTokens`](ParserToken) in [reverse polish notation](https://en.wikipedia.org/wiki/Reverse_Polish_notation).
//!
//! The parser implementation uses the [`context`](crate::Ctx) to categorize input tokens of [`Token::Id`](crate::tokenizer::Token::Id) into VariableId, Function, Binary Operator and others.
pub use error::Error;
pub use token::ParserToken;
use ParseState::*;

use super::functions::Func;
use super::macros::{ApplyMode, ParsedMacro};
use super::operators::binary::Associativity;
use super::operators::{BiOp, UOp};
use super::tokenizer::{self, get_token_text, Token};
use super::Ctx;

mod error;
mod token;

#[derive(Debug)]
enum OperatorStackValue<'a> {
    LeftParen,
    BiOp(&'a BiOp),
    UOp(&'a UOp),
    Func(&'a Func),
    Macro(Box<dyn ParsedMacro + 'a>),
}

fn to_parser_token<'a>(
    sv: OperatorStackValue<'a>,
    arity: &mut Vec<usize>,
) -> Result<ParserToken<'a>, &'static str> {
    match sv {
        OperatorStackValue::LeftParen => Err("Left Parent cannot be in output queue"),
        OperatorStackValue::BiOp(b) => Ok(ParserToken::BiOp(b)),
        OperatorStackValue::UOp(u) => Ok(ParserToken::UOp(u)),
        OperatorStackValue::Func(f) => Ok(ParserToken::Func(f, arity.pop().unwrap())),
        OperatorStackValue::Macro(m) => Ok(ParserToken::Macro(m)),
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
    fn expect(self, state_to_expect: ParseState) -> Result<(), Error> {
        if self == state_to_expect {
            Ok(())
        } else if let Expression = self {
            Err(Error::ExpectedExpression)
        } else {
            Err(Error::ExpectedOperator)
        }
    }
}

/// Parses the input tokens into steam of [`ParserTokens`](ParserToken) in Reverse polish notation order
pub fn parse<'a, 'tokens>(
    tokens: &'tokens [Token<'a>],
    ctx: &'a Ctx,
) -> Result<Vec<ParserToken<'a>>, Error> {
    if tokens.is_empty() {
        return Ok(Vec::new());
    }
    let mut queue = Vec::new();
    let mut operator_stack: Vec<OperatorStackValue> = Vec::new();
    let mut parse_state: ParseState = Expression;
    let mut arity = Vec::new();
    let mut iter = tokens.iter().peekable();
    while let Some(current_token) = iter.next() {
        match *current_token {
            Token::Num(num) => {
                parse_state.expect(Expression)?;
                parse_state = Operator;
                queue.push(ParserToken::Num(num));
            }
            Token::Id(id) => {
                if let Some(u_op) = find_uop(ctx, id, parse_state) {
                    operator_stack.push(OperatorStackValue::UOp(u_op));
                } else if let Some(bi_op) = find_biop(ctx, id) {
                    parse_state.expect(Operator)?;
                    push_to_output(&mut queue, &mut operator_stack, bi_op);
                    parse_state = Expression;
                    operator_stack.push(OperatorStackValue::BiOp(bi_op));
                } else if let Some(func) = find_func(ctx, id, parse_state) {
                    if let Some(Token::OpenParen) = iter.peek() {
                        arity.push(0usize);
                        operator_stack.push(OperatorStackValue::Func(func))
                    } else {
                        // TODO v0.3: might be better to match id, to that fn(), and fn are different
                        return Err(Error::NoLeftParenAfterFnId);
                    }
                } else {
                    // variable
                    parse_state.expect(Expression)?;
                    parse_state = Operator;
                    queue.push(ParserToken::Id(id));
                }
            }
            Token::OpenParen => {
                parse_state.expect(Expression)?;
                operator_stack.push(OperatorStackValue::LeftParen);
            }
            Token::ClosedParen => {
                if parse_state == Expression {
                    // operator or left parent
                    //  (10 + ) or ()
                    // -------^-or--^
                    // |
                    // we are here
                    if let Some(OperatorStackValue::LeftParen) = operator_stack.last() {
                        // pop the left paren
                        operator_stack.pop().unwrap();
                    } else {
                        // operator before right paren is an error
                        return Err(Error::OperatorAtTheEnd);
                    }
                } else {
                    pop_operator_stack(&mut operator_stack, &mut queue, &mut arity, true)?;
                    // pop left paren
                    operator_stack.pop().unwrap();

                    if let Some(OperatorStackValue::Func(_)) = operator_stack.last() {
                        let a = arity.last_mut().unwrap();
                        *a += 1;
                    }
                }
                parse_state = Operator;
            }
            Token::Comma => {
                parse_state.expect(Operator)?;
                parse_state = Expression;
                pop_operator_stack(&mut operator_stack, &mut queue, &mut arity, false)?;

                let a = arity.last_mut().ok_or(Error::CommaOutsideFn)?;
                *a += 1;
            }
            Token::Macro { defn, text } => {
                let parse_result = defn.parse(text, parse_state)?;
                parse_state = parse_result.state_after;
                match parse_result.mode {
                    ApplyMode::Before => queue.push(ParserToken::Macro(parse_result.result)),
                    ApplyMode::After => {
                        operator_stack.push(OperatorStackValue::Macro(parse_result.result))
                    }
                };
            }
            ref token => {
                return Err(Error::BadToken(get_token_text(token)));
            }
        }
    }
    if let Expression = parse_state {
        return Err(Error::OperatorAtTheEnd);
    }
    while let Some(v) = operator_stack.pop() {
        if let OperatorStackValue::LeftParen = v {
            return Err(Error::MismatchedLeftParen);
        }
        let token = to_parser_token(v, &mut arity).unwrap();
        check_arity(&token)?;
        queue.push(token);
    }
    Ok(queue)
}

fn push_to_output<'a>(
    queue: &mut Vec<ParserToken<'a>>,
    operator_stack: &mut Vec<OperatorStackValue<'a>>,
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
pub fn parse_str<'a>(input: &'a str, ctx: &'a Ctx) -> Result<Vec<ParserToken<'a>>, Error> {
    let tokens = tokenizer::tokenize(input, ctx);
    parse(&tokens, ctx)
}

fn check_arity(token: &ParserToken) -> Result<(), Error> {
    if let ParserToken::Func(func, n_args) = token {
        if func.arity != 0 && func.arity != *n_args {
            return Err(Error::ArityMismatch {
                id: func.token.to_owned(),
                expected: func.arity,
                actual: *n_args,
            });
        }
    }
    Ok(())
}

fn pop_operator_stack<'a>(
    operator_stack: &mut Vec<OperatorStackValue<'a>>,
    queue: &mut Vec<ParserToken<'a>>,
    arity: &mut Vec<usize>,
    expect_left_paren: bool,
) -> Result<(), Error> {
    let mut found_left_paren = false;
    while let Some(v) = operator_stack.last() {
        if let OperatorStackValue::LeftParen = v {
            found_left_paren = true;
            break;
        }
        // unwrap: safe because operator stack value is never LeftParen and is always present
        let el = operator_stack.pop().unwrap();
        let token = to_parser_token(el, arity).unwrap();
        check_arity(&token)?;
        queue.push(token);
    }
    if found_left_paren || !expect_left_paren {
        Ok(())
    } else {
        Err(Error::MismatchedRightParen)
    }
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
    fn test_parse() -> Result<(), Error> {
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
        let result = parse(&[Token::BadToken(s.clone())], &ctx).unwrap_err();
        assert_eq!(
            std::mem::discriminant(&result),
            std::mem::discriminant(&Error::BadToken(s))
        );
    }
}
