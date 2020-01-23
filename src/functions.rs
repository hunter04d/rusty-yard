//! The module that deals with function.
//!
//! The main type in this module is [`Func`](Func). It allows you to define your own function with custom behaviour.
//!
//! # Example
//! ```
//! # use std::collections::HashMap;
//! use rusty_yard::{Ctx,functions::Func, evaluator::eval_str_with_vars_and_ctx};
//!
//! let exp = Func {
//!    token: "exp".to_owned(),
//!    arity: 1,
//!    func: |args| args[0].exp()
//! };
//! let mut vars = HashMap::new();
//! let mut ctx = Ctx::empty();
//! ctx.fns.push(exp);
//! assert_eq!(eval_str_with_vars_and_ctx("exp(1.0)", &mut vars, &ctx), Ok(std::f64::consts::E));
//! ```
//!
//! # Note
//!
//! A lot of functions are missing from [`default_functions`](default_functions) list.
//! Feel free to implement more of them.
#![deny(missing_docs)]

use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

use lazy_static::lazy_static;
use thiserror::Error;

/// Represents a function
#[derive(Clone)]
pub struct Func {
    /// Identifier of the function.
    pub token: String,
    /// Arity of the function.
    ///
    /// Set to 0 to make the function variadic.
    //TODO: 0 args function are missing this is a design flaw.
    pub arity: usize,

    /// The pointer to the function that implements the behaviour of the function.
    ///
    /// # Note
    ///
    /// [`evaluator`](crate::evaluator) will never pass any other number of parameters to the function other than arity.
    /// However, if the function is variadic `arity == 0` then any number of parameters,
    /// **including** 0 might be passed to the function by the evaluator.
    pub func: fn(&[f64]) -> f64,
}

/// Represents an error that can occur when calling [`Func::call`](Func::call).
#[derive(Debug, Error)]
#[error("Mismatched number of parameters when calling the function, expected: {expected}, actual: {actual}")]
pub struct Error {
    /// Expected number of parameters to the function.
    expected: usize,
    /// Actual number of parameters passed to the function.
    actual: usize,
}

impl Func {
    /// Call the function with the specified parameters.
    ///
    /// If number of parameters is equal to function arity, or function is variadic.
    /// returns [`Ok`](std::result::Result::Ok),
    /// otherwise [`Err`](std::result::Result::Err) with [`function::Error`](Error) type is returned.
    pub fn call(&self, args: &[f64]) -> Result<f64, Error> {
        if self.arity != 0 && args.len() != self.arity {
            Err(Error {
                expected: self.arity,
                actual: args.len(),
            })
        } else {
            let func = self.func;
            Ok(func(args))
        }
    }
}

// Because func is magic we need to implement all markers our self
impl PartialEq for Func {
    fn eq(&self, other: &Self) -> bool {
        self.token.eq(&other.token)
            && self.arity.eq(&other.arity)
            && self.func as usize == other.func as usize
    }
}

impl Hash for Func {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.token.hash(state);
        self.arity.hash(state);
        (self.func as usize).hash(state)
    }
}

impl Eq for Func {}

impl Debug for Func {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Fn {}({})", self.token, self.arity)
    }
}

#[allow(missing_docs)]
pub fn to_args_1(func: fn(f64) -> f64) -> impl Fn(&[f64]) -> f64 {
    move |args| func(args[0])
}

#[allow(missing_docs)]
pub fn to_args_2(func: fn(f64, f64) -> f64) -> impl Fn(&[f64]) -> f64 {
    move |args| func(args[0], args[1])
}

lazy_static! {
    /// max(a, b) function.
    ///
    /// # Implementation
    ///
    /// ```text
    /// a.max(b)
    /// ```
    pub static ref FN_MAX: Func = Func {
        token: "max".to_owned(),
        arity: 2,
        func: |args| {
            let arg1 = args[0];
            let arg2 = args[1];
            arg1.max(arg2)
        },
    };

    /// sum(..args) function.
    ///
    /// # Implementation
    ///
    /// ```text
    /// args.iter().sum()
    /// ```
    pub static ref FN_SUM: Func = Func {
        token: "sum".to_owned(),
        arity: 0,
        func: |args| args.iter().sum(),
    };

    /// prod(..args) function.
    ///
    /// # Implementation
    ///
    /// ```text
    /// args.iter().product()
    /// ```
    pub static ref FN_PROD: Func = Func {
        token: "prod".to_owned(),
        arity: 0,
        func: |args| args.iter().product(),
    };

    /// sub(a, b) function.
    ///
    /// # Implementation
    ///
    /// ```text
    /// a - b
    /// ```
    pub static ref FN_SUB: Func = Func {
        token: "sub".to_owned(),
        arity: 2,
        func: |args| {
            let arg1 = args[0];
            let arg2 = args[1];
            arg1 - arg2
        },
    };
}

/// Get the default functions list.
///
/// This includes all function from [`functions`](self) module.
pub fn default_functions() -> Vec<Func> {
    vec![
        FN_MAX.clone(),
        FN_SUM.clone(),
        FN_SUM.clone(),
        FN_PROD.clone(),
    ]
}
