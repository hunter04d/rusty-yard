//! Provides definition on unary operator type.
//!
//! It also provides default operators that one might expect.
use std::fmt::{self, Debug, Formatter};

use lazy_static::lazy_static;

/// Represents the unary operator.
#[derive(Clone, Hash, Eq, PartialEq)]
pub struct UOp {
    /// operator's identifier.
    pub token: String,

    /// the function that is invoked by [`evaluator`](crate::evaluator) when evaluating this operator.
    pub func: fn(f64) -> f64,
}

impl Debug for UOp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("UOp").field("token", &self.token).finish()
    }
}
lazy_static! {

    /// `-a ("unary minus")` operator.
    ///
    /// # Implementation
    ///
    /// ```text
    /// return -a
    /// ```
    pub static ref NEGATE: UOp = UOp {
        token: "-".to_owned(),
        func: |v| -v,
    };

    /// `+a ("unary plus")` operator.
    ///
    /// # Implementation
    ///
    /// ```text
    /// return a
    /// ```
    pub static ref PLUS: UOp = UOp {
        token: "+".to_owned(),
        func: |v| v,
    };
}

/// Get the default unary operator list.
///
/// This includes all operators from [`this`](self) module.
pub fn default_operators() -> Vec<UOp> {
    vec![PLUS.clone(), NEGATE.clone()]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_debug() {
        let op = UOp {
            token: "#".to_owned(),
            func: |_| 0.0,
        };
        let dbg = format!("{:?}", op);
        assert!(dbg.contains("UOp"));
        assert!(dbg.contains("token") && dbg.contains("#"));
    }
}
