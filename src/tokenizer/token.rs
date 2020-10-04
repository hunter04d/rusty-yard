use crate::macros::Macro;

/// Represents a macro token, part of [`Token::Macro`](Token::Macro)
#[derive(Debug)]
pub struct MacroToken<'a, 'ctx> {
    pub text: &'a str,
    pub definition: &'ctx dyn Macro,
}

/// Represents tokenizers token, generally produced by [`tokenizer::tokenize`](super::tokenize).
#[derive(Debug)]
pub enum Token<'a, 'ctx> {
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
    /// Represents the bad token, i.e it could not be tokenized by any other rules.
    BadToken(&'a str),
    /// Macro token
    ///
    /// Macros are the fist to match, so you can override any default behavior of any other variants using macros.
    Macro(MacroToken<'a, 'ctx>),
}

impl Token<'_, '_> {
    /// Returns the text representation of the token
    pub fn token_text(&self) -> String {
        use Token::*;
        match self {
            OpenParen => String::from("("),
            ClosedParen => String::from(")"),
            Id(s) => String::from(*s),
            Num(n) => n.to_string(),
            BadToken(s) => s.to_string(),
            Comma => String::from(","),
            Macro(MacroToken { text, definition }) => format!("<MACRO {:?}>({})", definition, text),
        }
    }
}

impl PartialEq for Token<'_, '_> {
    #[cfg_attr(tarpaulin, skip)]
    fn eq(&self, other: &Self) -> bool {
        use Token::*;
        match (self, other) {
            (OpenParen, OpenParen) => true,
            (ClosedParen, ClosedParen) => true,
            (Comma, Comma) => true,
            (Id(s1), Id(s2)) => s1 == s2,
            (Num(f1), Num(f2)) => f1 == f2,
            (BadToken(b1), BadToken(b2)) => b1 == b2,
            (Macro(_), Macro(_)) => unimplemented!(),
            _ => false,
        }
    }
}
