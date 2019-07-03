pub mod binary_operators;
pub mod unary_operators;
pub mod evaluator;
pub mod tokenizer;
pub mod parser;

use self::binary_operators::*;
use std::collections:: HashSet;

use crate::shunting_yard::unary_operators::UOp;

pub struct Ctx<'a> {
    pub(self) bi_ops: HashSet<BiOp<'a>>,
    pub(self) u_ops: HashSet<UOp<'a>>,
}

impl<'a> Ctx<'a> {
    pub fn new(bi_ops: HashSet<BiOp<'a>>, u_ops: HashSet<UOp<'a>>) -> Self {
       Self {bi_ops, u_ops}
    }

    pub fn default() -> Self {
        Self { bi_ops: binary_operators::default_operators(), u_ops: unary_operators::default_operators() }
    }
}
