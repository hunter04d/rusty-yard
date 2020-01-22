use std::fmt::{self, Debug, Formatter};

use lazy_static::lazy_static;

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct UOp {
    pub token: String,
    pub func: fn(f64) -> f64,
}

impl Debug for UOp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{{{}}}", self.token)
    }
}
lazy_static! {
    pub static ref NEGATE: UOp = UOp {
        token: "-".to_owned(),
        func: |v| -v,
    };
    pub static ref PLUS: UOp = UOp {
        token: "+".to_owned(),
        func: |v| v,
    };
}

pub fn default_operators() -> Vec<UOp> {
    vec![PLUS.clone(), NEGATE.clone()]
}
