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
//! "a+b" will as well:
//!
//! ```
//! # use rusty_yard::tokenizer::{tokenize, Token};
//! use rusty_yard::Ctx;
//! assert_eq!(tokenize("a+b", &Ctx::default()), vec![Token::Id("a"), Token::Id("+"), Token::Id("b")])
//! ```
//!
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
use crate::tokenizer::token::MacroToken;

mod token;

/// Represents a match from one of the match functions
///
/// This type is [`None`](std::option::Option::None) when input hasn't match and
/// [`Some(number_of_chars_matched)`](std::option::Option::Some) if we matched
///
/// # Note for macros
/// It is technically possible to have a 0 sized match to alter the behavior of existing token.
#[derive(Debug)]
pub struct Match<T>(pub T, pub usize);

/// Tokenizes the input string into Tokens.
///
/// Each token reuses memory from the input string when possible.
///
/// # Panics
///
/// This function will panic is input in not an ascii string.\
/// TODO: add unicode support.
pub fn tokenize<'a, 'ctx>(input: &'a str, ctx: &'ctx Ctx) -> Vec<Token<'a, 'ctx>> {
    if !input.is_ascii() {
        panic!("Input contains non ascii characters");
    }
    let mut output = Vec::new();
    let whitespace_to_skip = skip_whitespace(input);
    let mut text = &input[whitespace_to_skip..];
    while !text.is_empty() {
        let (token, consumed) = if text.starts_with('(') {
            (Token::OpenParen, '('.len_utf8())
        } else if text.starts_with(')') {
            (Token::ClosedParen, ')'.len_utf8())
        } else if text.starts_with(',') {
            (Token::Comma, ','.len_utf8())
        } else if let Some(Match(m, c)) = match_macros(text, &ctx) {
            let token = MacroToken {
                text: &text[..c],
                definition: m,
            };
            (Token::Macro(token), c)
        } else if let Some(Match(n, c)) = match_number(text) {
            (Token::Num(n), c)
        } else if let Some(Match(id, c)) = match_op(text, ctx).or_else(|| match_id(text, ctx)) {
            (Token::Id(id), c)
        } else {
            let c = text
                .chars()
                .take_while(|c| !c.is_ascii_whitespace())
                .map(|c| c.len_utf8())
                .sum();
            (Token::BadToken(&text[..c]), c)
        };
        output.push(token);
        text = &text[consumed..];
        let whitespace_to_skip = skip_whitespace(text);
        text = &text[whitespace_to_skip..];
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
pub fn match_id<'a>(text: &'a str, ctx: &'_ Ctx) -> Option<Match<&'a str>> {
    fn is_disallowed(ch: &char) -> bool {
        const DISALLOWED_CHARS: &[char] = &['(', ')', ','];
        DISALLOWED_CHARS.iter().any(|v| v == ch)
    }
    fn is_valid_first_char(ch: &char) -> bool {
        ch.is_ascii_graphic() && !ch.is_ascii_digit() && !is_disallowed(ch)
    }
    fn is_valid_char(ch: &char) -> bool {
        ch.is_ascii_graphic() && !is_disallowed(ch)
    }

    let mut iterator = text.chars();
    let first = iterator.next().filter(is_valid_first_char)?;
    let full_len = first.len_utf8()
        + iterator
            .take_while(is_valid_char)
            .map(char::len_utf8)
            .sum::<usize>();
    let text = &text[..full_len];
    let len = ctx
        .u_ops
        .iter()
        .find_map(|op| text.find(&op.token))
        .or_else(|| ctx.bi_ops.iter().find_map(|op| text.find(&op.token)))
        .unwrap_or(full_len);
    Some(Match(&text[..len], len))
}

/// Matches one of the macros from 'ctx' against the start of input `text`.
///
/// Returns [`Some(matched macro, length of the match)`](std::option::Option::Some) if we matched
/// and [`None`](std::option::Option::None) when input hasn't matched any of the macros.
pub fn match_macros<'a>(text: &str, ctx: &'a Ctx) -> Option<Match<&'a dyn Macro>> {
    ctx.macros.iter().find_map(|m| {
        let Match((), c) = m.match_input(text, ctx)?;
        Some(Match(m.as_ref(), c))
    })
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
pub fn match_op<'a>(text: &'a str, ctx: &Ctx) -> Option<Match<&'a str>> {
    let matched_bi_op = match_bi_op(text, &ctx.bi_ops).map(|m| m.1);
    let matched_u_op = || match_u_op(text, &ctx.u_ops).map(|m| m.1);
    matched_bi_op
        .or_else(matched_u_op)
        .map(|c| Match(&text[..c], c))
}

/// Matches the start of the input `text` against one of [BiOps](crate::operators::binary)
///
/// Returns [`Some(length of the match)`](std::option::Option::Some) if we matched
/// and [`None`](std::option::Option::None) when input hasn't matched any BiOp.
pub fn match_bi_op<'a>(text: &str, bi_ops: &'a [BiOp]) -> Option<Match<&'a BiOp>> {
    bi_ops
        .iter()
        .find(|op| text.starts_with(&op.token))
        .map(|op| Match(op, op.token.len()))
}

/// Matches the start of the input `text` against one of [UOps](crate::operators::unary)
///
/// Returns [`Some(matched macro, length of the match)`](std::option::Option::Some) if we matched
/// and [`None`](std::option::Option::None) when input hasn't matched any UOp.
pub fn match_u_op<'a>(text: &str, u_ops: &'a [UOp]) -> Option<Match<&'a UOp>> {
    u_ops
        .iter()
        .find(|op| text.starts_with(&op.token))
        .map(|op| Match(op, op.token.len()))
}

/// Matches the start of 'text' with the definition of number in this crate.
///
/// Returns [`Some(length of the match)`](std::option::Option::Some) if we matched
/// and [`None`](std::option::Option::None) when input hasn't a number.
pub fn match_number(text: &str) -> Option<Match<f64>> {
    let mut iterator = text.chars();
    let first_char = iterator.next().filter(char::is_ascii_digit)?;
    let mut index = first_char.len_utf8();
    let mut seen_dot = false;
    for ch in iterator {
        if ch.is_ascii_digit() {
            index += ch.len_utf8();
            continue;
        }
        if ch == '.' {
            if seen_dot {
                return None;
            }
            seen_dot = true;
            index += ch.len_utf8();
            continue;
        }
        break;
    }
    let num: f64 = text[..index].parse().ok()?;
    Some(Match(num, index))
}

/// Matches the start of 'text' string `str_to_match`.
///
/// Returns [`Some(number_of_chars_matched)`](std::option::Option::Some) if we matched
/// and [`None`](std::option::Option::None) when input hasn't match the string.
#[cfg_attr(tarpaulin, skip)]
pub fn match_str<'a>(text: &'a str, str_to_match: &str) -> Option<Match<&'a str>> {
    if text.starts_with(str_to_match) {
        Some(Match(&text[..str_to_match.len()], str_to_match.len()))
    } else {
        None
    }
}
/// Returns the number of whitespaces that are at the beginning of input 'text'
///
/// This is useful in implementing your own macros
#[cfg_attr(tarpaulin, skip)]
pub fn skip_whitespace(text: &str) -> usize {
    text.chars()
        .take_while(|ch| ch.is_ascii_whitespace())
        .map(|ch| ch.len_utf8())
        .sum()
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
            prop_assert_eq!(str.len(), res.1);
        }
        #[test]
        fn test_match_ids(s in r#"[a-zA-z](?:[a-zA-Z]|[0-9])*"#) {
            let ctx = &Ctx::empty();
            let res = match_id(&s, ctx);
            prop_assert!(res.is_some());
            let res = res.unwrap();
            prop_assert_eq!(s.len(), res.1);
        }
    }

    // TODO: more tests cases
    #[test]
    fn test_tokenize() {
        let ctx = Ctx::empty();
        let input_expected = &[
            ("1.0 op 1.0", vec![Num(1.0), Id("op"), Num(1.0)]),
            ("- 1.0", vec![Id("-"), Num(1.0)]),
            ("pi()", vec![Id("pi"), OpenParen, ClosedParen]),
            ("1 + ", vec![Num(1.0), Id("+")]),
        ];
        for (input, expected) in input_expected {
            let output = tokenize(input, &ctx);
            assert_eq!(output, *expected);
        }
    }

    #[test]
    fn test_match_number_fails() {
        let str = "not a number";
        let res = match_number(str);
        assert!(res.is_none())
    }
}
