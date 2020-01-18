use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

use lazy_static::lazy_static;

// Because func is magic we need to implement all markers our self
#[derive(Clone)]
pub struct Func {
    pub token: String,
    pub arity: usize,
    pub func: fn(&[f64]) -> f64,
}
#[derive(Debug)]
pub struct Error;

impl Func {
    pub(super) fn call(&self, args: &[f64]) -> Result<f64, Error> {
        if self.arity != 0 && args.len() != self.arity {
            Err(Error)
        } else {
            let func = self.func;
            Ok(func(args))
        }
    }
}

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

pub fn to_args_1(func: fn(f64) -> f64) -> impl Fn(&[f64]) -> f64 {
    move |args| func(args[0])
}

pub fn to_args_2(func: fn(f64, f64) -> f64) -> impl Fn(&[f64]) -> f64 {
    move |args| func(args[0], args[1])
}

lazy_static! {
    pub static ref FN_MAX: Func = Func {
        token: "max".to_owned(),
        arity: 2,
        func: |args| {
            let arg1 = args[0];
            let arg2 = args[1];
            arg1.max(arg2)
        },
    };
    pub static ref FN_SUM: Func = Func {
        token: "sum".to_owned(),
        arity: 0,
        func: |args| args.iter().sum(),
    };

     pub static ref FN_PROD: Func = Func {
        token: "prod".to_owned(),
        arity: 0,
        func: |args| args.iter().product(),
    };
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

pub fn default_functions() -> HashSet<Func> {
    let mut s = HashSet::new();
    s.insert(FN_MAX.clone());
    s.insert(FN_SUM.clone());
    s.insert(FN_PROD.clone());
    s.insert(FN_SUB.clone());
    s
}
