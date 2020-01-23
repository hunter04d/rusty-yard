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
    ExpectExpression,
    /// The state signaling that the next token is expected to be an operator
    ExpectOperator,
}

/// Parses the input tokens into steam of [`ParserTokens`](ParserToken) in Reverse polish notation order
pub fn parse<'a, 'tokens>(
    tokens: &'tokens [Token<'a>],
    ctx: &'a Ctx,
) -> Result<Vec<ParserToken<'a>>, Error> {
    let mut queue = Vec::new();
    let mut operator_stack: Vec<OperatorStackValue> = Vec::new();
    let mut parse_state: ParseState = ExpectExpression;
    let mut arity = Vec::new();
    let mut iter = tokens.iter().peekable();
    while let Some(current_token) = iter.next() {
        match *current_token {
            Token::Num(num) => {
                queue.push(ParserToken::Num(num));
                parse_state = ExpectOperator;
            }
            Token::Id(id) => {
                let next_stack_value: Option<OperatorStackValue> = find_uop(ctx, id, parse_state)
                    .or_else(|| find_biop(ctx, id, &mut queue, &mut operator_stack))
                    .or_else(|| find_func(ctx, id, parse_state));
                match next_stack_value {
                    // is variable
                    None => {
                        parse_state = ExpectOperator;
                        queue.push(ParserToken::Id(id));
                    }
                    // is operator
                    Some(sv) => {
                        if let OperatorStackValue::Func(_) = sv {
                            if let Some(Token::OpenParen) = iter.peek() {
                                arity.push(1usize);
                            } else {
                                return Err(Error::NoLeftParenAfterFnId);
                            }
                        }
                        parse_state = ExpectExpression;
                        operator_stack.push(sv);
                    }
                }
            }
            Token::OpenParen => {
                if let ParseState::ExpectOperator = parse_state {
                    return Err(Error::ExpectedOperator);
                }
                operator_stack.push(OperatorStackValue::LeftParen);
            }
            Token::ClosedParen => {
                pop_operator_stack(&mut operator_stack, &mut queue, &mut arity, true)?;
                operator_stack.pop().unwrap();
            }
            Token::Comma => {
                pop_operator_stack(&mut operator_stack, &mut queue, &mut arity, false)?;
                parse_state = ExpectExpression;
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
    if parse_state == ExpectExpression {
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

/// Parses the input string into a stream of [`ParsedTokens`](ParserToken).
///
/// This tokenizes the input first using [`tokenizer::tokenize`](crate::tokenizer::tokenize)
/// and then parses it using [`parse`](parse).
///
/// This uses the ctx provided as the last parameter.
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
        } else {
            // unwrap: safe because operator stack value is never LeftParen and is always present
            let el = operator_stack.pop().unwrap();
            let token = to_parser_token(el, arity).unwrap();
            check_arity(&token)?;
            queue.push(token);
        }
    }
    if found_left_paren || !expect_left_paren {
        Ok(())
    } else {
        Err(Error::MismatchedRightParen)
    }
}

fn find_biop<'a, 'b>(
    ctx: &'a Ctx,
    id: &str,
    queue: &mut Vec<ParserToken<'b>>,
    operator_stack: &mut Vec<OperatorStackValue<'b>>,
) -> Option<OperatorStackValue<'a>> {
    // binary operator
    let b_op = ctx.bi_ops.iter().find(|op| op.token == id)?;
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
    Some(OperatorStackValue::BiOp(b_op))
}

fn find_uop<'a>(ctx: &'a Ctx, id: &str, parse_state: ParseState) -> Option<OperatorStackValue<'a>> {
    let u_op = ctx.u_ops.iter().find(|op| op.token == id)?;
    match parse_state {
        ExpectExpression => Some(OperatorStackValue::UOp(u_op)),
        ExpectOperator => None,
    }
}

fn find_func<'a>(
    ctx: &'a Ctx,
    id: &str,
    parse_state: ParseState,
) -> Option<OperatorStackValue<'a>> {
    let func = ctx.fns.iter().find(|op| op.token == id)?;
    match parse_state {
        ExpectExpression => Some(OperatorStackValue::Func(func)),
        ExpectOperator => None, // does this make sense?
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
        let input = vec![
            vec![Token::Num(10.0), Token::Id("bi_op"), Token::Id("10")],
            vec![Token::Id("u_op"), Token::Num(10.0)],
            vec![
                Token::Id("a"),
                Token::Id("bi_op"),
                Token::Id("b"),
                Token::Id("bi_op"),
                Token::Id("c"),
            ],
        ];
        let expected: Vec<_> = vec![
            vec![Num(10.0), Id("10"), BiOp(&bi_op)],
            vec![Num(10.0), UOp(&u_op)],
            vec![Id("a"), Id("b"), BiOp(&bi_op), Id("c"), BiOp(&bi_op)],
        ];
        for (expected, input) in expected.into_iter().zip(input) {
            let actual = parse(&input, &ctx)
                .expect("Parse succeeded")
                .into_iter()
                .collect::<Vec<_>>();
            assert_eq!(expected, actual);
        }
        Ok(())
    }
}
