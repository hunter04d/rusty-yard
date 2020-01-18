use std::collections::HashSet;

use functions::Func;
use macros::Macro;
use operators::{binary, unary};
use operators::{BiOp, UOp};

pub mod evaluator;
pub mod functions;
pub mod macros;
pub mod operators;
pub mod parser;
pub mod tokenizer;

pub struct Ctx {
    pub bi_ops: HashSet<BiOp>,
    pub u_ops: HashSet<UOp>,
    pub fns: HashSet<Func>,
    pub macros: Vec<Box<dyn Macro>>,
}

impl Ctx {
    #[allow(dead_code)]
    pub fn new(bi_ops: HashSet<BiOp>, u_ops: HashSet<UOp>, fns: HashSet<Func>) -> Self {
        Self {
            bi_ops,
            u_ops,
            fns,
            macros: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self {
            bi_ops: HashSet::new(),
            u_ops: HashSet::new(),
            fns: HashSet::new(),
            macros: Vec::new(),
        }
    }
}

impl Default for Ctx {
    fn default() -> Self {
        Self {
            bi_ops: binary::default_operators(),
            u_ops: unary::default_operators(),
            fns: functions::default_functions(),
            macros: Vec::new(),
        }
    }
}
