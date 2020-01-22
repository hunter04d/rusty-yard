use std::fmt::{self, Debug, Formatter};

use lazy_static::lazy_static;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct BiOp {
    pub token: String,
    pub precedence: u32,
    pub associativity: Associativity,
    pub func: fn(f64, f64) -> f64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Associativity {
    LEFT,
    RIGHT,
}

impl Debug for BiOp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{{{}}}", self.token)
    }
}

lazy_static! {
    pub static ref PLUS: BiOp = BiOp {
        token: "+".to_owned(),
        precedence: 0,
        associativity: Associativity::LEFT,
        func: |e1, e2| e1 + e2,
    };
    pub static ref MINUS: BiOp = BiOp {
        token: "-".to_owned(),
        precedence: 0,
        associativity: Associativity::LEFT,
        func: |e1, e2| e1 - e2,
    };
    pub static ref MULTIPLY: BiOp = BiOp {
        token: "*".to_owned(),
        precedence: 1,
        associativity: Associativity::LEFT,
        func: |e1, e2| e1 * e2,
    };
    pub static ref DIVIDE: BiOp = BiOp {
        token: "/".to_owned(),
        precedence: 1,
        associativity: Associativity::LEFT,
        func: |e1, e2| e1 / e2,
    };
    pub static ref POWER: BiOp = BiOp {
        token: "^".to_owned(),
        precedence: 2,
        associativity: Associativity::RIGHT,
        func: |e1, e2| e1.powf(e2),
    };
}
pub fn default_operators() -> Vec<BiOp> {
    vec![
        PLUS.clone(),
        MINUS.clone(),
        MULTIPLY.clone(),
        DIVIDE.clone(),
        POWER.clone(),
    ]
}
