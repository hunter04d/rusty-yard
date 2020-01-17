use std::collections::VecDeque;

use ParseState::{ExpectExpression, ExpectOperator};

use super::operators::binary::Associativity;
use super::operators::{BiOp, UOp};
use super::{tokenizer::Token, Ctx};
use crate::shunting_yard::functions::Func;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Error;

pub fn parse<'a>(tokens: &Vec<Token<'a>>, ctx: &'a Ctx) -> Result<VecDeque<ParserToken<'a>>, Error> {
    let mut queue = VecDeque::new();
    let mut operator_stack: Vec<OperatorStackValue> = Vec::new();
    let mut parse_state: ParseState = ExpectExpression;
    let mut arity = Vec::new();
    let mut iter = tokens.iter().peekable();
    while let Some(current_token) = iter.next() {
        match *current_token {
            Token::Num(num) if parse_state == ExpectExpression => {
                queue.push_back(ParserToken::Num(num));
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
                        queue.push_back(ParserToken::Id(id.clone()));
                    }
                    // is operator
                    Some(sv) => {
                        if let OperatorStackValue::Func(_) = sv {
                            if let Some(Token::OpenParen) = iter.peek() {
                                arity.push(1usize);
                            } else {
                                return Err(Error);
                            }
                        }
                        parse_state = ExpectExpression;
                        operator_stack.push(sv);
                    }
                }
            }
            Token::OpenParen => {
                operator_stack.push(OperatorStackValue::LeftParen);
            }
            Token::ClosedParen => {
                pop_operator_stack(&mut operator_stack, &mut queue, &mut arity, true)?;
                operator_stack.pop().unwrap();
            }
            Token::Comma => {
                pop_operator_stack(&mut operator_stack, &mut queue, &mut arity, false)?;
                // TODO: arity stack
                parse_state = ExpectExpression;
                let a = arity
                    .last_mut()
                    .expect("Comma can only be used in functions, arity should contain value");
                *a += 1;
            }
            _ => {
                return Err(Error);
            }
        }
    }
    if parse_state == ExpectExpression {
        return Err(Error);
    }
    while let Some(v) = operator_stack.pop() {
        if let OperatorStackValue::LeftParen = v {
            return Err(Error);
        } // TODO: function arity check
        let token = to_parser_token(v, &mut arity).unwrap();
        queue.push_back(token);
    }
    Ok(queue)
}

fn pop_operator_stack<'a>(
    operator_stack: &mut Vec<OperatorStackValue<'a>>,
    queue: &mut VecDeque<ParserToken<'a>>,
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
            // TODO: function arity check
            let token = to_parser_token(el, arity).unwrap();
            queue.push_back(token);
        }
    }
    return if found_left_paren || !expect_left_paren {
        Ok(())
    } else {
        Err(Error)
    };
}

fn find_biop<'a, 'b>(
    ctx: &'a Ctx,
    id: &str,
    queue: &mut VecDeque<ParserToken<'b>>,
    operator_stack: &mut Vec<OperatorStackValue<'b>>,
) -> Option<OperatorStackValue<'a>> {
    // binary operator
    let b_op = ctx.bi_ops.iter().find(|op| op.token == id)?;
    while let Some(top_of_stack) = operator_stack.last() {
        match *top_of_stack {
            OperatorStackValue::UOp(op) => {
                queue.push_back(ParserToken::UOp(op));
                operator_stack.pop();
            }
            OperatorStackValue::BiOp(op)
                if op.precedence > b_op.precedence
                    || (op.precedence == b_op.precedence
                        && op.associativity == Associativity::LEFT) =>
            {
                let pt = op.into();
                queue.push_back(pt);
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
    use super::*;
}
