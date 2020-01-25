use crate::functions::Func;
use crate::macros::ParsedMacro;
use crate::operators::{BiOp, UOp};

/// Represents the parser token.
///
/// Parser tokens make up the RPN token stream (`&[ParserToken]`) that can be evaluated using [`evaluator::eval`](crate::evaluator::eval)
/// and similar functions.
#[derive(Debug)]
pub enum ParserToken<'a> {
    /// Represents the primitive (number of type f64).
    Num(f64),
    /// Represents a variable identifier.
    Id(&'a str),
    /// Represents a [`Unary operator`](crate::operators::UOp).
    UOp(&'a UOp),
    /// Represents a [`Binary operator`](crate::operators::BiOp).
    BiOp(&'a BiOp),
    /// Represents the [`function`](crate::functions::Func).
    ///
    /// `.1` is the number of parameters the function has called with.
    /// It is equivalent to `Func.arity`, unless function is variadic.
    /// In that case it represents the actual number of parameters the function was called with.
    Func(&'a Func, usize),

    /// Represents a [`ParsedMacro`](crate::macros::ParsedMacro)
    Macro(Box<dyn ParsedMacro + 'a>),
}

impl PartialEq for ParserToken<'_> {
    #[cfg_attr(tarpaulin, skip)]
    fn eq(&self, other: &Self) -> bool {
        use ParserToken::*;
        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return false;
        }
        match (self, other) {
            (Num(n1), Num(n2)) => n1 == n2,
            (Id(id1), Id(id2)) => id1 == id2,
            (UOp(op1), UOp(op2)) => op1 == op2,
            (BiOp(op1), BiOp(op2)) => op1 == op2,
            (Func(f1, s1), Func(f2, s2)) => f1 == f2 && s1 == s2,
            (Macro(m1), Macro(m2)) => std::ptr::eq(m1.as_ref(), m2.as_ref()),
            _ => false,
        }
    }
}

impl From<f64> for ParserToken<'_> {
    #[inline]
    #[cfg_attr(tarpaulin, skip)]
    fn from(num: f64) -> Self {
        ParserToken::Num(num)
    }
}

impl<'a> From<&'a str> for ParserToken<'a> {
    #[inline]
    #[cfg_attr(tarpaulin, skip)]
    fn from(id: &'a str) -> Self {
        ParserToken::Id(id)
    }
}

impl<'a> From<&'a BiOp> for ParserToken<'a> {
    #[inline]
    #[cfg_attr(tarpaulin, skip)]
    fn from(op: &'a BiOp) -> Self {
        ParserToken::BiOp(op)
    }
}

impl<'a> From<&'a UOp> for ParserToken<'a> {
    #[inline]
    #[cfg_attr(tarpaulin, skip)]
    fn from(op: &'a UOp) -> Self {
        ParserToken::UOp(&op)
    }
}

impl<'a> From<(&'a Func, usize)> for ParserToken<'a> {
    #[inline]
    #[cfg_attr(tarpaulin, skip)]
    fn from(tuple: (&'a Func, usize)) -> Self {
        ParserToken::Func(tuple.0, tuple.1)
    }
}

impl<'a, Macro: ParsedMacro + 'a> From<Macro> for ParserToken<'a> {
    #[inline]
    #[cfg_attr(tarpaulin, skip)]
    fn from(m: Macro) -> Self {
        ParserToken::Macro(Box::new(m))
    }
}
