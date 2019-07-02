use std::convert::TryFrom;
use crate::tokenizer::Token;

#[derive(Debug, Copy, Clone)]
pub enum BinaryOperator {
    PLUS,
    MINUS,
    MULTIPLY,
    DIVIDE,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Associativity {
    LEFT,
    RIGHT,
}

impl BinaryOperator {
    pub fn precedence(&self) -> u32 {
        match self {
            BinaryOperator::PLUS |
            BinaryOperator::MINUS => 1,
            BinaryOperator::MULTIPLY |
            BinaryOperator::DIVIDE => 2,
        }
    }

    pub fn associativity(&self) -> Associativity {
        match self {
            BinaryOperator::PLUS |
            BinaryOperator::MINUS |
            BinaryOperator::MULTIPLY |
            BinaryOperator::DIVIDE => Associativity::LEFT
        }
    }
}

impl TryFrom<&Token> for BinaryOperator {
    type Error = String;

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        match value {
            Token::Plus => Ok(BinaryOperator::PLUS),
            Token::Minus => Ok(BinaryOperator::MINUS),
            Token::Star => Ok(BinaryOperator::MULTIPLY),
            Token::Slash => Ok(BinaryOperator::DIVIDE),
            other => Err(format!("Can't map Token {:?} into Binary operator", other))
        }
    }
}