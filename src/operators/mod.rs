//! Includes definition for deffest operator types.
//!
//! This module mostly reexports the relevant types from its submodule.
//! # Example
//!
//! You can easily define your own esoteric unary and binary operatos using types in this module:
//!
//! ```
//! # use std::collections::HashMap;
//! use rusty_yard::operators::{BiOp, UOp, binary::Associativity};
//! use rusty_yard::{Ctx, evaluator::eval_str_with_vars_and_ctx};
//!
//! let mut ctx = Ctx::empty();
//! let mut vars = HashMap::new();
//! // add new u_op to context
//! ctx.u_ops.push(UOp {
//!     token: "$$$".to_owned(),
//!     func: |a| 1000.0 * a,
//! });
//! // add new bi_op to context
//! ctx.bi_ops.push(BiOp {
//!     token: "crazy".to_owned(),
//!     precedence: 0,
//!     // use right associativity because why not?
//!     associativity: Associativity::RIGHT,
//!     func: |a, b| (a.powi(2) + b.powi(2)).sqrt()
//! });
//! assert_eq!(eval_str_with_vars_and_ctx("$$$(12 crazy 3 crazy 4)", &mut vars, &ctx), Ok(13_000.0))
//! //                                     ^      ^       ^ 1. 'crazy' is right associative (3 crazy 4) = 5 is first;
//! //                                     |      | 2. next this will be evaluated 12 crazy 5;
//! //                                     | 3. finally, $$$ is evaluated.
//! ```

pub use binary::BiOp;
pub use unary::UOp;

pub mod binary;
pub mod unary;
