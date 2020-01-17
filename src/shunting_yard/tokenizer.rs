#[derive(Debug)]
pub enum Token<'a> {
    OpenParen,
    ClosedParen,
    Comma,
    Id(&'a str),
    Num(f64),
    // Bad token might merge 2 tokens separated by whitespace, as such it has to allocate
    // example: \x01<space>\x01 results in one bad token <BAD TOKEN>(\x01\x01)
    BadToken(String),
}

#[allow(dead_code)]
pub fn get_token_text(token: &Token) -> String {
    match token {
        Token::OpenParen => String::from("("),
        Token::ClosedParen => String::from(")"),
        Token::Id(s) => String::from(*s),
        Token::Num(n) => n.to_string(),
        Token::BadToken(s) => format!("<BAD TOKEN>({})", s),
        Token::Comma => String::from(","),
    }
}

use crate::shunting_yard::Ctx;

type Match<T> = Option<(T, usize)>;

pub fn tokenize<'a>(input: &'a str, ctx: &Ctx) -> Vec<Token<'a>> {
    if !input.is_ascii() {
        panic!()
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
                if let Some((op, n)) = match_op(&input[index..], &ctx) {
                    consumed = n;
                    Token::Id(op)
                } else if let Some((id, n)) = match_id(&input[index..]) {
                    consumed = n;
                    Token::Id(id)
                } else if let Some((num, n)) = match_number(&input[index..]) {
                    consumed = n;
                    Token::Num(num)
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

fn match_id(text: & str) -> Match<&str> {
    let extra_chars = "$@_{}[]:-.";
    let is_extra = |ch: char| extra_chars.chars().any(|v| v == ch);
    let is_valid_first_char = |ch: char| ch.is_ascii_alphabetic() || is_extra(ch);
    let is_valid_char = |ch: char| ch.is_ascii_alphanumeric() || is_extra(ch);

    let mut iterator = text.chars();
    if let Some(ch)  = iterator.next() {
        if is_valid_first_char(ch) {
            let mut index = 1usize;
            while let Some(ch) = iterator.next() {
                if is_valid_char(ch) {
                    index += 1;
                    continue;
                }
                break;
            }
            return Some((&text[..index], index));
        }
    }
    None
}

fn match_op<'a>(text: &'a str, ctx: &Ctx) -> Match<&'a str> {
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
    matched_bi_op.or_else(matched_u_op).map(|index| {
        (&text[..index], index)
    })
}

fn match_number(text: &str) -> Match<f64> {
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
            let num_res = text[..index].parse();
            return num_res.ok().map(|num| (num, index));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::approx_eq;
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn test_match_numbers(f in prop::num::f64::NORMAL) {
            let str = f.to_string();
            let res = match_number(&str).unwrap();
            prop_assert!(approx_eq!(f64, f, res.0));
            prop_assert_eq!(str.len(), res.1);
        }
    }

    #[test]
    fn test_match_number_fails() {
        let str = "not a number";
        let res = match_number(str);
        assert_eq!(None, res);
    }

    #[test]
    fn test_bad_token() {
        let ctx = &Ctx::empty();
        let s = "\x01\x01";
        let res = tokenize(s, ctx);
        assert_eq!(1, res.len());
        if let Token::BadToken(bad_token) = &res[0] {
            assert_eq!(s, *bad_token);
        } else {
            assert!(false, "Not <Bad Token>");
        }
    }
}
