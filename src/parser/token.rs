use crate::functions::Func;
use crate::macros::ParsedMacro;
use crate::operators::{BiOp, UOp};

#[derive(Debug)]
pub enum ParserToken<'a> {
    Num(f64),
    Id(&'a str),
    UOp(&'a UOp),
    BiOp(&'a BiOp),
    Func(&'a Func, usize),
    Macro(Box<dyn ParsedMacro + 'a>),
}

impl PartialEq for ParserToken<'_> {
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

impl<'a> From<&'a BiOp> for ParserToken<'a> {
    fn from(op: &'a BiOp) -> Self {
        ParserToken::BiOp(op)
    }
}

impl<'a> From<&'a UOp> for ParserToken<'a> {
    fn from(op: &'a UOp) -> Self {
        ParserToken::UOp(&op)
    }
}
