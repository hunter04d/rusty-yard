use std::convert::TryFrom;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::fmt;


#[derive(Clone, Eq, PartialEq, Hash)]
pub struct BiOp<'a> {
    pub token: &'a str,
    pub precedence: u32,
    pub associativity: Associativity,
    pub func: fn(f64, f64) -> f64,
}

impl<'a> Debug for BiOp<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{{{}}}", self.token)
    }
}

pub const BINARY_OPERATOR_PLUS: BiOp = BiOp {
    token: "+",
    precedence: 0,
    associativity: Associativity::LEFT,
    func: |e1, e2| e1 + e2,
};
pub const BINARY_OPERATOR_MINUS: BiOp = BiOp {
    token: "-",
    precedence: 0,
    associativity: Associativity::LEFT,
    func: |e1, e2| e1 - e2,
};

pub const BINARY_OPERATOR_MULTIPLY: BiOp = BiOp {
    token: "*",
    precedence: 0,
    associativity: Associativity::LEFT,
    func: |e1, e2| e1 * e2,
};
pub const BINARY_OPERATOR_DIVIDE: BiOp = BiOp {
    token: "/",
    precedence: 0,
    associativity: Associativity::LEFT,
    func: |e1, e2| e1 / e2,
};

pub const BINARY_OPERATOR_POWER: BiOp = BiOp {
    token: "^",
    precedence: 0,
    associativity: Associativity::RIGHT,
    func: |e1, e2| e1.powf(e2),
};

pub fn default_operators() -> HashSet<BiOp<'static>> {
    let mut s = HashSet::new();
    s.insert(BINARY_OPERATOR_PLUS);
    s.insert(BINARY_OPERATOR_MINUS);
    s.insert(BINARY_OPERATOR_MULTIPLY);
    s.insert(BINARY_OPERATOR_DIVIDE);
    s.insert(BINARY_OPERATOR_POWER);
    s
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Associativity {
    LEFT,
    RIGHT,
}
