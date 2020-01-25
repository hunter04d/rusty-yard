use crate::macros::Macro;

/// Represent tokenizers token, generally produced by [`tokenizer::tokenize`](super::tokenize).
#[derive(Debug)]
pub enum Token<'a> {
    /// Open parenthesis ('(') token.
    OpenParen,
    /// Closes parenthesis (')') token.
    ClosedParen,
    /// Comma token (',').
    Comma,
    /// Identifier token.
    ///
    /// The definition is very relaxed by design (one or more characters that are `|char| char.is_ascii_graphic()` but not '(', ')', ',')
    Id(&'a str),
    /// Primitive (number).
    Num(f64),
    // TODO: is this reasonable behaviour?
    /// Represents the bad token, i.e it could not be tokenized by any other rules.
    ///
    /// Bad token might merge 2 tokens separated by whitespace, as such it has to allocate.
    /// # Example
    ///
    /// \x01<space>\x01 results in one bad token <BAD TOKEN>(\x01\x01)
    BadToken(String),

    /// Macro token
    ///
    /// Macros and the fist to match, so you can override any default behavior of any other variants using macros.
    Macro {
        /// Reference to macro definition
        defn: &'a dyn Macro,
        /// Text that matched using `Macro::match`(crate::macros::Macro::match).
        text: &'a str,
    },
}

impl PartialEq for Token<'_> {
    #[cfg_attr(tarpaulin, skip)]
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
