//! Provides definitions of types used in defining what a binary operator is.
//!
//! It also provides default operators that one might expect.
use std::fmt::{self, Debug, Formatter};

use lazy_static::lazy_static;

/// Represent the binary operator.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct BiOp {
    /// operator's identifier.
    pub token: String,

    /// operator's precedence.
    pub precedence: u32,

    /// operator's associativity.
    pub associativity: Associativity,

    /// the function that is invoked by [`evaluator`](crate::evaluator) when evaluating this operator.
    pub func: fn(f64, f64) -> f64,
}

/// The associativity of the operator.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Associativity {
    /// Operator is left associative.
    LEFT,
    /// Operator is right associative.
    ///
    /// The only standard operator that is right associative is `power`.
    RIGHT,
}

impl Debug for BiOp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{{{}}}", self.token)
    }
}

lazy_static! {

    /// `a + b` operator.
    ///
    /// # Implementation
    ///
    /// ```text
    /// a + b
    /// ```
    pub static ref PLUS: BiOp = BiOp {
        token: "+".to_owned(),
        precedence: 0,
        associativity: Associativity::LEFT,
        func: |e1, e2| e1 + e2,
    };

    /// `a - b` operator.
    ///
    /// # Implementation
    ///
    /// ```text
    /// a - b
    /// ```
    pub static ref MINUS: BiOp = BiOp {
        token: "-".to_owned(),
        precedence: 0,
        associativity: Associativity::LEFT,
        func: |e1, e2| e1 - e2,
    };

    /// `a * b` operator.
    ///
    /// # Implementation
    ///
    /// ```text
    /// a * b
    /// ```
    pub static ref MULTIPLY: BiOp = BiOp {
        token: "*".to_owned(),
        precedence: 1,
        associativity: Associativity::LEFT,
        func: |e1, e2| e1 * e2,
    };

    /// `a / b` operator.
    ///
    /// # Implementation
    ///
    /// ```text
    /// a / b
    /// ```
    pub static ref DIVIDE: BiOp = BiOp {
        token: "/".to_owned(),
        precedence: 1,
        associativity: Associativity::LEFT,
        func: |e1, e2| e1 / e2,
    };

    /// `a ^ b ("power")` operator.
    ///
    /// # Implementation
    ///
    /// ```text
    /// a.powf(b)
    /// ```
    pub static ref POWER: BiOp = BiOp {
        token: "^".to_owned(),
        precedence: 2,
        associativity: Associativity::RIGHT,
        func: |e1, e2| e1.powf(e2),
    };
}

/// Get the default binary operator list.
///
/// This includes all operators from [`this`](self) module.
pub fn default_operators() -> Vec<BiOp> {
    vec![
        PLUS.clone(),
        MINUS.clone(),
        MULTIPLY.clone(),
        DIVIDE.clone(),
        POWER.clone(),
    ]
}
