use crate::shunting_yard::{tokenizer, binary_operators, unary_operators, Ctx};
use std::collections::{HashSet, VecDeque};
use crate::shunting_yard::binary_operators::{BiOp, Associativity};
use crate::shunting_yard::unary_operators::UOp;
use crate::shunting_yard::tokenizer::Token;
use std::convert::{TryInto, TryFrom};
use crate::shunting_yard::parser::ParseState::{ExpectExpression, ExpectOperator};

#[derive(Debug)]
pub enum ParserToken<'a> {
    Num(f64),
    Id(String),
    UOp(UOp<'a>),
    BiOp(BiOp<'a>),
}

impl<'a> From<BiOp<'a>> for ParserToken<'a> {
    fn from(op: BiOp<'a>) -> Self {
        ParserToken::<'a>::BiOp(op)
    }
}

impl<'a> From<UOp<'a>> for ParserToken<'a> {
    fn from(op: UOp<'a>) -> Self {
        ParserToken::<'a>::UOp(op)
    }
}

impl<'a> TryFrom<OperatorStackValue<'a>> for ParserToken<'a> {
    type Error = &'static str;

    fn try_from(value: OperatorStackValue<'a>) -> Result<Self, Self::Error> {
        match value {
            OperatorStackValue::LeftParen => Err("Left Parent cannot be in output queue"),
            OperatorStackValue::BiOp(b) => Ok(ParserToken::BiOp(b)),
            OperatorStackValue::UOp(u) => Ok(ParserToken::UOp(u))
        }
    }
}

#[derive(Debug)]
enum OperatorStackValue<'a> {
    LeftParen,
    BiOp(BiOp<'a>),
    UOp(UOp<'a>),
}


#[derive(Debug, Eq, PartialEq)]
enum ParseState {
    ExpectExpression,
    ExpectOperator,
}

#[derive(Debug)]
pub struct Error;

pub fn parse<'a>(
    tokens: &Vec<tokenizer::Token>,
    ctx: &Ctx<'a>,
) -> Result<VecDeque<ParserToken<'a>>, Error> {
    let mut queue = VecDeque::new();
    let mut operator_stack: Vec<OperatorStackValue> = Vec::new();
    let mut parse_state = ExpectExpression;
    for i in 0..tokens.len() {
        let current_token = &tokens[i];
        match current_token {
            Token::Num(num) if parse_state == ExpectExpression => {
                queue.push_back(ParserToken::Num(num.clone()));
                parse_state = ExpectOperator;
            }
            Token::Id(id) => {
                // unary operator
                let next_stack_value = ctx.u_ops.iter().find(|op| op.token == id)
                    .and_then(|op| {
                        if parse_state == ExpectExpression {
                            Some(OperatorStackValue::UOp(op.clone()))
                        } else { None }
                    })
                    .or_else(|| {
                        // binary operator
                        let b_op = ctx.bi_ops.iter().find(|op| op.token == id)?;
                        while let Some(top_of_stack) = operator_stack.last() {
                            match top_of_stack {
                                OperatorStackValue::UOp(op) => {
                                    queue.push_back(ParserToken::UOp(op.clone()));
                                    operator_stack.pop();
                                }
                                OperatorStackValue::BiOp(op)
                                if op.precedence > b_op.precedence ||
                                    (op.precedence == b_op.precedence && op.associativity == Associativity::LEFT) => {
                                    queue.push_back(op.clone().into());
                                    operator_stack.pop();
                                }
                                OperatorStackValue::LeftParen | _ => { break; }
                            }
                        }
                        Some(OperatorStackValue::BiOp(b_op.clone()))
                    });
                match next_stack_value {
                    None => {
                        parse_state = ExpectOperator;
                        queue.push_back(ParserToken::Id(id.clone()));
                    }
                    Some(sv) => {
                        match sv {
                            _ => parse_state = ExpectExpression
                        }
                        operator_stack.push(sv);
                    }
                }
            }
            Token::OpenParen => {
                operator_stack.push(OperatorStackValue::LeftParen);
            }
            Token::ClosedParen => {
                let mut found_left_paren = false;
                while let Some(v) = operator_stack.pop() {
                    if let OperatorStackValue::LeftParen = v {
                        found_left_paren = true;
                        break;
                    } else {
                        queue.push_back(v.try_into().unwrap());
                    }
                }
                if !found_left_paren {
                    debug_assert!(operator_stack.is_empty());
                    return Err(Error);
                }
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
        }
        queue.push_back(v.try_into().unwrap());
    }
    Ok(queue)
}
