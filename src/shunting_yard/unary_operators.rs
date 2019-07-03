use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::fmt;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct UOp<'a> {
    pub token: &'a str,
    pub func: fn(f64) -> f64,
}

impl<'a> Debug for UOp<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result{
        write!(f, "{{{}}}", self.token)
    }
}

const UNARY_OPERATOR_NEGATE: UOp = UOp {
    token: "-",
    func: |v| -v,
};
const UNARY_OPERATOR_PLUS: UOp = UOp {
    token: "+",
    func: |v| v,
};

pub fn default_operators() -> HashSet<UOp<'static>> {
    let mut s = HashSet::new();
    s.insert(UNARY_OPERATOR_NEGATE);
    s.insert(UNARY_OPERATOR_PLUS);
    s
}
