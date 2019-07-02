pub mod binary_operator;
pub mod unary_operator;

use self::binary_operator::*;
use crate::tokenizer;
use crate::tokenizer::Token;
use std::collections::VecDeque;

use std::convert::{TryFrom, TryInto};
use crate::shunting_yard::unary_operator::UnaryOperator;

#[derive(Debug)]
pub enum ParserToken {
    Number(f64),
    ID(String),
    UnaryOperator(UnaryOperator),
    BinaryOperator(BinaryOperator),
}

impl TryFrom<OperatorStackValue> for ParserToken {
    type Error = &'static str;

    fn try_from(value: OperatorStackValue) -> Result<Self, Self::Error> {
        match value {
            OperatorStackValue::LeftParen => Err("Left Parent cannot be in output queue"),
            OperatorStackValue::BinaryOperator(b) => Ok(ParserToken::BinaryOperator(b)),
            OperatorStackValue::UnaryOperator(u) => Ok(ParserToken::UnaryOperator(u))
        }
    }
}

#[derive(Debug)]
enum OperatorStackValue {
    LeftParen,
    BinaryOperator(BinaryOperator),
    UnaryOperator(UnaryOperator),
}


pub fn parse(tokens: &Vec<tokenizer::Token>) -> Result<VecDeque<ParserToken>, ()> {
    let mut queue = VecDeque::new();
    let mut operator_stack: Vec<OperatorStackValue> = Vec::new();
    for i in 0..tokens.len() {
        let current_token = &tokens[i];
        if let Token::Number(num) = current_token {
            queue.push_back(ParserToken::Number(num.clone()));
        } else if let Token::ID(id) = current_token {
            queue.push_back(ParserToken::ID(id.clone()));
        } else if let Ok(bo) = BinaryOperator::try_from(current_token.clone()) {
            while let Some(top_of_stack) = operator_stack.last() {
                match top_of_stack {
                    OperatorStackValue::BinaryOperator(op)
                    if op.precedence() > bo.precedence() ||
                        (op.precedence() == bo.precedence() && op.associativity() == Associativity::LEFT) => {
                        queue.push_back(ParserToken::BinaryOperator(op.clone()));
                        operator_stack.pop();
                    }
                    OperatorStackValue::UnaryOperator(op) => { unimplemented!("Unary operator") }
                    OperatorStackValue::LeftParen | _ => { break; }
                }
            }
            operator_stack.push(OperatorStackValue::BinaryOperator(bo));
        } else if let Token::OpenParen = current_token {
            operator_stack.push(OperatorStackValue::LeftParen);
        } else if let Token::ClosedParen = current_token {
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
                return Err(());
            }
        } else {
            return Err(());
        }
    }
    while let Some(v) = operator_stack.pop() {
        if let OperatorStackValue::LeftParen = v {
            return Err(());
        }
        queue.push_back(v.try_into().unwrap());
    }
    Ok(queue)
}