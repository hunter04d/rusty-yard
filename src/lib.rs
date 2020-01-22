use functions::Func;
use macros::Macro;
use operators::{binary, unary};
use operators::{BiOp, UOp};

use crate::macros::default::Assign;

// reason api not stable
#[allow(clippy::implicit_hasher)]
pub mod evaluator;
pub mod functions;
pub mod macros;
pub mod operators;
pub mod parser;
pub mod tokenizer;

pub struct Ctx {
    pub bi_ops: Vec<BiOp>,
    pub u_ops: Vec<UOp>,
    pub fns: Vec<Func>,
    pub macros: Vec<Box<dyn Macro>>,
}

impl Ctx {
    pub fn new(bi_ops: Vec<BiOp>, u_ops: Vec<UOp>, fns: Vec<Func>) -> Self {
        Self {
            bi_ops,
            u_ops,
            fns,
            macros: Vec::new(),
        }
    }

    pub fn empty() -> Self {
        Self {
            bi_ops: Vec::new(),
            u_ops: Vec::new(),
            fns: Vec::new(),
            macros: Vec::new(),
        }
    }

    pub fn default_with_macros() -> Self {
        Self {
            macros: vec![Box::new(Assign)],
            ..Default::default()
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
