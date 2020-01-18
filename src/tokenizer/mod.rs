pub use token::Token;

use crate::macros::Macro;

use super::Ctx;

mod token;

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

// number of chars matches
pub type Match = Option<usize>;

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

pub fn match_macro<'a>(text: &str, ctx: &'a Ctx) -> Option<(&'a dyn Macro, usize)> {
    ctx.macros
        .iter()
        .find_map(|m| m.match_input(text).map(|len| (m.as_ref(), len)))
}

fn match_op(text: &str, ctx: &Ctx) -> Match {
    let matched_bi_op = ctx
        .bi_ops
        .iter()
        .find(|op| text.starts_with(&op.token))
        .map(|op| op.token.len());
    let matched_u_op = || {
        ctx.u_ops
            .iter()
            .find(|op| text.starts_with(&op.token))
            .map(|op| op.token.len())
    };
    matched_bi_op.or_else(matched_u_op).map(|s| s)
}

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

pub fn match_str(text: &str, str_to_match: &str) -> Match {
    if text.starts_with(str_to_match) {
        Some(str_to_match.len())
    } else {
        None
    }
}

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
