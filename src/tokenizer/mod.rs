//! Exposes function than tokenize the input into a sequence of tokens.
//!
//! Functions in this module use [`context`](crate::Ctx) to tokenize the input in somewhat way sensible to humans.
//!
//! # Example
//!
//! For example +a will be tokenized into `[Token::Id("+"), Token::Id("a")]`, despite the fact that `'+'` is a valid char for Token::Id:
//!
//! ```
//! # use rusty_yard::tokenizer::{tokenize, Token};
//! use rusty_yard::Ctx;
//! assert_eq!(tokenize("+a", &Ctx::default()), vec![Token::Id("+"), Token::Id("a")])
//! ```
//!
//! "a + b " will be tokenized as one might expect:
//!
//! ```
//! # use rusty_yard::tokenizer::{tokenize, Token};
//! use rusty_yard::Ctx;
//! assert_eq!(tokenize("a + b", &Ctx::default()), vec![Token::Id("a"), Token::Id("+"), Token::Id("b")])
//! ```
//!
//! However, a+b will just make one identifier:
//! TODO: Is this a resanoble behaivior?
//!
//! ```
//! # use rusty_yard::tokenizer::{tokenize, Token};
//! use rusty_yard::Ctx;
//! assert_eq!(tokenize("a+b", &Ctx::default()), vec![Token::Id("a+b")])
//! ```
//! # Note
//!
//! **[`Tokenizer`](crate::tokenizer) does not distinguish between different types of identifiers.**
//! They all are using [`Token::Id`](crate::tokenizer::Token::Id).
//!
//! It is the job of the [`parser`](crate::parser) to distinguish different identifiers.
pub use token::Token;

use crate::macros::Macro;

use super::Ctx;
use crate::operators::{BiOp, UOp};

mod token;

#[allow(missing_docs)]
pub fn get_token_text(token: &Token) -> String {
    match token {
        Token::OpenParen => String::from("("),
        Token::ClosedParen => String::from(")"),
        Token::Id(s) => String::from(*s),
        Token::Num(n) => n.to_string(),
        Token::BadToken(s) => format!("<BAD TOKEN>({})", s),
        Token::Comma => String::from(","),
        Token::Macro { defn, text } => format!("<MACRO {:?}>({})", defn, text),
    }
}

/// Represents a match from one of the match functions
///
/// This type is [`None`](std::option::Option::None) when input hasn't match and
/// [`Some(number_of_chars_matched)`](std::option::Option::Some) if we matched
///
/// # Note for macros
/// It is technically possible to have a 0 sized match to alter the behavior of existing token.
pub type Match = Option<usize>;

/// Tokenizes the input string into Tokens.
///
/// Each token reuses memory from the input string when possible.
///
/// # Panics
///
/// This function will panic is input in not an ascii string.\
/// TODO: add unicode support.
pub fn tokenize<'a>(input: &'a str, ctx: &'a Ctx) -> Vec<Token<'a>> {
    if !input.is_ascii() {
        panic!("Input contains non ascii characters");
    }
    let mut output = Vec::new();
    let mut iterator = input.chars();
    let mut index: usize = 0;
    while let Some(ch) = iterator.next() {
        if ch.is_ascii_whitespace() {
            index += 1;
            continue;
        }
        let mut consumed: usize = 1;
        let next_token = match ch {
            '(' => Token::OpenParen,
            ')' => Token::ClosedParen,
            ',' => Token::Comma,
            _ => {
                let text = &input[index..];
                if let Some((m, n)) = match_macro(text, ctx) {
                    consumed = n;
                    Token::Macro {
                        defn: m,
                        text: &text[..n],
                    }
                } else if let Some(n) = match_number(text) {
                    consumed = n;
                    Token::Num(
                        text[..n]
                            .parse()
                            .expect("Parsing a matched number should not fail"),
                    )
                } else if let Some(n) = match_op(text, ctx).or_else(|| match_id(text)) {
                    consumed = n;
                    Token::Id(&text[..n])
                } else {
                    Token::BadToken(ch.to_string())
                }
            }
        };
        // merge the trailing <Bad Token>s into one
        let mut last_merged = false;
        if let Some(last_token) = output.last_mut() {
            if let (Token::BadToken(s1), Token::BadToken(s2)) = (last_token, &next_token) {
                last_merged = true;
                *s1 += s2;
            }
        }
        if !last_merged {
            output.push(next_token);
        }
        if consumed > 1 {
            // Iteration consumes one char initially
            iterator.nth(consumed - 2);
        }
        index += consumed;
    }
    output
}

/// Matches the start of the `text` with the definition of id in this crate.
///
/// The definition of *identifier* very relaxed by design
/// (one or more characters that are `|char| char.is_ascii_graphic()` but not '(', ')', ',').
///
/// Returns [`Some(length of the match)`](std::option::Option::Some) if we matched
/// and [`None`](std::option::Option::None) when input hasn't matched an identifier.
#[allow(clippy::while_let_on_iterator)]
pub fn match_id(text: &str) -> Match {
    const DISALLOWED_CHARS: &str = "(),";
    let is_disallowed = |ch: char| DISALLOWED_CHARS.chars().any(|v| v == ch);
    let is_valid_first_char =
        |ch: char| ch.is_ascii_graphic() && !ch.is_ascii_digit() && !is_disallowed(ch);
    let is_valid_char = |ch: char| ch.is_ascii_graphic() && !is_disallowed(ch);

    let mut iterator = text.chars();
    if let Some(ch) = iterator.next() {
        if is_valid_first_char(ch) {
            let mut index = 1usize;
            while let Some(ch) = iterator.next() {
                if is_valid_char(ch) {
                    index += 1;
                    continue;
                }
                break;
            }
            return Some(index);
        }
    }
    None
}

/// Matches one of the macros from 'ctx' against the start of input `text`.
///
/// Returns [`Some(matched macro, length of the match)`](std::option::Option::Some) if we matched
/// and [`None`](std::option::Option::None) when input hasn't matched any of the macros.
pub fn match_macro<'a>(text: &str, ctx: &'a Ctx) -> Option<(&'a dyn Macro, usize)> {
    ctx.macros
        .iter()
        .find_map(|m| match_single_macro(text, m.as_ref()))
}

/// Matches single macro against the start of the `text`.
///
/// Returns [`Some(matched macro, length of the match)`](std::option::Option::Some) if we matched
/// and [`None`](std::option::Option::None) when input hasn't matched this macro.
#[inline]
fn match_single_macro<'a>(text: &str, m: &'a dyn Macro) -> Option<(&'a dyn Macro, usize)> {
    m.match_input(text).map(|len| (m, len))
}

/// Matches the start of input `text` against either one of [Binary operators](crate::operators::binary) or
/// [Unary operators](crate::operators::unary) from input context
///
/// Returns [`Some(length of the match)`](std::option::Option::Some) if we matched
/// and [`None`](std::option::Option::None) when input hasn't matched any of UOp or BiOp from context.
///
/// # Note
///
/// Binary operators are matched first, then unary.
///
/// In most cases this implementation detail does not matter.
#[inline]
pub fn match_op(text: &str, ctx: &Ctx) -> Match {
    let matched_bi_op = match_bi_op(text, &ctx.bi_ops);
    let matched_u_op = || match_u_op(text, &ctx.u_ops);
    matched_bi_op.or_else(matched_u_op)
}

/// Matches the start of the input `text` against one of [BiOps](crate::operators::binary)
///
/// Returns [`Some(length of the match)`](std::option::Option::Some) if we matched
/// and [`None`](std::option::Option::None) when input hasn't matched any BiOp.
pub fn match_bi_op(text: &str, bi_ops: &Vec<BiOp>) -> Match {
    bi_ops
        .iter()
        .find(|op| text.starts_with(&op.token))
        .map(|op| op.token.len())
}

/// Matches the start of the input `text` against one of [UOps](crate::operators::unary)
///
/// Returns [`Some(matched macro, length of the match)`](std::option::Option::Some) if we matched
/// and [`None`](std::option::Option::None) when input hasn't matched any UOp.
pub fn match_u_op(text: &str, u_ops: &Vec<UOp>) -> Match {
    u_ops
        .iter()
        .find(|op| text.starts_with(&op.token))
        .map(|op| op.token.len())
}

/// Matches the start of 'text' with the definition of number in this crate.
///
/// Returns [`Some(length of the match)`](std::option::Option::Some) if we matched
/// and [`None`](std::option::Option::None) when input hasn't a number.
#[allow(clippy::while_let_on_iterator)]
pub fn match_number(text: &str) -> Match {
    let mut iterator = text.chars();
    if let Some(ch) = iterator.next() {
        if ch.is_ascii_digit() {
            let mut index = 1usize;
            let mut seen_dot = false;
            while let Some(ch) = iterator.next() {
                if ch.is_ascii_digit() {
                    index += 1;
                    continue;
                }
                if ch == '.' {
                    if !seen_dot {
                        seen_dot = true;
                        index += 1;
                        continue;
                    }
                    return None;
                }
                break;
            }
            return Some(index);
        }
    }
    None
}

/// Matches the start of 'text' string `str_to_match`.
///
/// Returns [`Some(number_of_chars_matched)`](std::option::Option::Some) if we matched
/// and [`None`](std::option::Option::None) when input hasn't match the string.
pub fn match_str(text: &str, str_to_match: &str) -> Match {
    if text.starts_with(str_to_match) {
        Some(str_to_match.len())
    } else {
        None
    }
}
/// Returns the number of whitespaces that are at the beginning of input 'text'
///
/// This is useful in implementing your own macros
pub fn skip_whitespace(text: &str) -> usize {
    text.chars()
        .take_while(|ch| ch.is_ascii_whitespace())
        .count()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::Token::*;
    use super::*;

    proptest! {
        #[test]
        fn test_match_numbers(f in prop::num::f64::NORMAL) {
            let str = f.to_string();
            let res = match_number(&str);
            prop_assert!(res.is_some());
            let res = res.unwrap();
            prop_assert_eq!(str.len(), res);
        }
        #[test]
        fn test_match_ids(s in r#"[a-zA-z](?:[a-zA-Z]|[0-9])*"#) {
            let res = match_id(&s);
            prop_assert!(res.is_some());
            let res = res.unwrap();
            prop_assert_eq!(s.len(), res);
        }
    }

    // TODO: more tests cases
    #[test]
    fn test_tokenize() {
        let ctx = Ctx::empty();
        let expected = vec![
            vec![Num(1.0), Id("op"), Num(1.0)],
            vec![Id("-"), Num(1.0)],
            vec![Id("pi"), OpenParen, ClosedParen],
        ];
        let input = vec!["1.0 op 1.0", "- 1.0", "pi()"];
        for (expected, input) in expected.into_iter().zip(input) {
            let output = tokenize(input, &ctx);
            assert_eq!(expected, output);
        }
    }

    #[test]
    fn test_match_number_fails() {
        let str = "not a number";
        let res = match_number(str);
        assert_eq!(None, res);
    }

    #[test]
    fn test_bad_token_merging() {
        let s = "\x01\x01";
        let ctx = Ctx::empty();
        let res = tokenize(s, &ctx);
        assert_eq!(1, res.len());
        if let Token::BadToken(bad_token) = &res[0] {
            assert_eq!(s, *bad_token);
        }
    }
}
