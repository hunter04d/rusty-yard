//! This crate provides some functions to [tokenize](crate::tokenizer), [parse](crate::parser), and [evaluate](crate::evaluator) expressions.
//!
//! It uses [reverse polish notation](https://en.wikipedia.org/wiki/Reverse_Polish_notation) and
//! [shunting yard algorithm](https://en.wikipedia.org/wiki/Shunting-yard_algorithm]) to do so.
//!
//! See [evaluator](crate::evaluator) documentation to get started with high level api that allows you to evaluate strings directly.
#![deny(missing_docs)]
use functions::Func;
use macros::{default::default_macros, Macro};
use operators::{binary, unary, BiOp, UOp};
use std::fmt;
use std::fmt::{Display, Formatter};

// reason api not stable
#[allow(clippy::implicit_hasher)]
pub mod evaluator;
pub mod functions;
pub mod macros;
pub mod operators;
pub mod parser;
pub mod tokenizer;

/// The context of the expression
///
/// It is used to make [tokenization](crate::tokenizer) more resalable form human perspective and
/// to actually parse the expression into a steam of tokens that can be executed by [`evaluator`](crate::evaluator).
pub struct Ctx {
    /// Binary operators
    pub bi_ops: Vec<BiOp>,
    /// Unary operators
    pub u_ops: Vec<UOp>,
    /// Functions that this context contains
    pub fns: Vec<Func>,
    /// Macros that this context contains
    pub macros: Vec<Box<dyn Macro>>,
}

impl Ctx {
    /// Creates new context with context items passes as the parameters.
    pub fn new(bi_ops: Vec<BiOp>, u_ops: Vec<UOp>, fns: Vec<Func>) -> Self {
        Self {
            bi_ops,
            u_ops,
            fns,
            macros: Vec::new(),
        }
    }

    /// Creates new empty context.
    pub fn empty() -> Self {
        Self {
            bi_ops: Vec::new(),
            u_ops: Vec::new(),
            fns: Vec::new(),
            macros: Vec::new(),
        }
    }

    /// Creates new default context that is similar to the one produced by [`default`](std::default::Default::default) but also has default macros enabled.
    ///
    /// Macros are formed from [`default_macros`](crate::macros::default::default_macros) function.
    pub fn default_with_macros() -> Self {
        Self {
            macros: default_macros(),
            ..Default::default()
        }
    }
}

impl Default for Ctx {
    /// Creates new default `Ctx`.
    ///
    /// This uses:
    ///
    /// - [binary::default_operators](crate::operators::binary::default_operators) to populate binary operators;
    /// - [unary::default_operators](crate::operators::unary::default_operators) to populate binary operators;
    /// - [functions::default_functions](crate::functions::default_functions) to populate functions.
    fn default() -> Self {
        Self {
            bi_ops: binary::default_operators(),
            u_ops: unary::default_operators(),
            fns: functions::default_functions(),
            macros: Vec::new(),
        }
    }
}

/// Position is the token stream
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Pos(pub usize);

impl Display for Pos {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.0)
    }
}
