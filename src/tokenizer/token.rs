use crate::macros::Macro;

#[derive(Debug)]
pub enum Token<'a> {
    OpenParen,
    ClosedParen,
    Comma,
    Id(&'a str),
    Num(f64),
    // TODO: is this reasonable behaviour?
    // Bad token might merge 2 tokens separated by whitespace, as such it has to allocate
    // example: \x01<space>\x01 results in one bad token <BAD TOKEN>(\x01\x01)
    BadToken(String),
    Macro { defn: &'a dyn Macro, text: &'a str },
}

impl PartialEq for Token<'_> {
    fn eq(&self, other: &Self) -> bool {
        use Token::*;
        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return false;
        }
        match (self, other) {
            (Id(s1), Id(s2)) => s1 == s2,
            (Num(f1), Num(f2)) => f1 == f2,
            (BadToken(b1), BadToken(b2)) => b1 == b2,
            (Macro { defn: d1, text: t1 }, Macro { defn: d2, text: t2 }) => {
                format!("{:?}", d1) == format!("{:?}", d2) && t1 == t2
            }
            _ => true,
        }
    }
}
