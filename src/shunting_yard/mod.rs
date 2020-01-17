use std::collections::HashSet;

use functions::Func;
use operators::{binary, unary};
use operators::{BiOp, UOp};

pub mod evaluator;
pub mod functions;
pub mod operators;
pub mod parser;
pub mod tokenizer;

pub struct Ctx {
    pub bi_ops: HashSet<BiOp>,
    pub u_ops: HashSet<UOp>,
    pub fns: HashSet<Func>,
}

impl Ctx {
    #[allow(dead_code)]
    pub fn new(bi_ops: HashSet<BiOp>, u_ops: HashSet<UOp>, fns: HashSet<Func>) -> Self {
        Self { bi_ops, u_ops, fns }
    }

    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self {
            bi_ops: HashSet::new(),
            u_ops: HashSet::new(),
            fns: HashSet::new(),
        }
    }
}

impl Default for Ctx {
    fn default() -> Self {
        Self {
            bi_ops: binary::default_operators(),
            u_ops: unary::default_operators(),
            fns: functions::default_functions(),
        }
    }
}
