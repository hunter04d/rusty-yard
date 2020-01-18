use ParseState::{ExpectExpression, ExpectOperator};

use super::functions::Func;
use super::operators::binary::Associativity;
use super::operators::{BiOp, UOp};
use super::{tokenizer::Token, Ctx};

#[derive(Debug, PartialEq)]
pub enum ParserToken<'a> {
    Num(f64),
    Id(&'a str),
    UOp(&'a UOp),
    BiOp(&'a BiOp),
    Func(&'a Func, usize),
}

impl<'a> From<&'a BiOp> for ParserToken<'a> {
    fn from(op: &'a BiOp) -> Self {
        ParserToken::BiOp(op)
    }
}

impl<'a> From<&'a UOp> for ParserToken<'a> {
    fn from(op: &'a UOp) -> Self {
        ParserToken::UOp(&op)
    }
}

#[derive(Debug)]
enum OperatorStackValue<'a> {
    LeftParen,
    BiOp(&'a BiOp),
    UOp(&'a UOp),
    Func(&'a Func),
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
    }
}

#[derive(Debug, Eq, PartialEq)]
enum ParseState {
    ExpectExpression,
    ExpectOperator,
}

use crate::tokenizer::get_token_text;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Parser Error")]
pub enum Error {
    #[error("Expected left paren after function id")]
    NoLeftParenAfterFnId,
    #[error("Bad token {0:?}")]
    BadToken(String),

    #[error("Operator at the end of the token stream")]
    OperatorAtTheEnd,

    #[error("Mismatched left paren in the token stream")]
    MismatchedLeftParen,

    #[error("Mismatched right paren in the token stream")]
    MismatchedRightParen,

    #[error("Arity of function {id} mismatched: expected: {expected}, actual: {actual}")]
    ArityMismatch {
        id: String,
        expected: usize,
        actual: usize,
    },
    #[error("Expected Operator, found expression")]
    ExpectedOperator,

    #[error("Comma can only be used in functions, arity stack is empty")]
    CommaOutsideFn,
}

pub fn parse<'a>(tokens: &Vec<Token<'a>>, ctx: &'a Ctx) -> Result<Vec<ParserToken<'a>>, Error> {
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
                let next_stack_value: Option<OperatorStackValue> = find_uop(ctx, id, &parse_state)
                    .or_else(|| find_biop(ctx, id, &mut queue, &mut operator_stack))
                    .or_else(|| find_func(ctx, id, &parse_state));
                match next_stack_value {
                    // is variable
                    None => {
                        parse_state = ExpectOperator;
                        queue.push(ParserToken::Id(id.clone()));
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

fn check_arity(token: &ParserToken) -> Result<(), Error> {
    if let ParserToken::Func(func, n_args) = token {
        if func.arity != 0 && func.arity != *n_args {
            return Err(Error::ArityMismatch {
                id: (&func.token).into(),
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
    return if found_left_paren || !expect_left_paren {
        Ok(())
    } else {
        Err(Error::MismatchedRightParen)
    };
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

fn find_uop<'a>(
    ctx: &'a Ctx,
    id: &str,
    parse_state: &ParseState,
) -> Option<OperatorStackValue<'a>> {
    let u_op = ctx.u_ops.iter().find(|op| op.token == id)?;
    match parse_state {
        ExpectExpression => Some(OperatorStackValue::UOp(u_op)),
        ExpectOperator => None,
    }
}

fn find_func<'a>(
    ctx: &'a Ctx,
    id: &str,
    parse_state: &ParseState,
) -> Option<OperatorStackValue<'a>> {
    let func = ctx.fns.iter().find(|op| op.token == id)?;
    match parse_state {
        ExpectExpression => Some(OperatorStackValue::Func(func)),
        ExpectOperator => None, // does this make sense?
    }
}

#[cfg(test)]
mod tests {

    use super::ParserToken::*;
    use super::*;
    use crate::operators;

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
        ctx.bi_ops.insert(get_biop());
        ctx.u_ops.insert(get_uop());
        ctx
    }

    // TODO: more test cases
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
