#[derive(Debug, PartialEq)]
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

type Match<T> = Option<(T, usize)>;

pub fn tokenize(input: &str) -> Vec<Token> {
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
                if let Some((num, n)) = match_number(&input[index..]) {
                    consumed = n;
                    Token::Num(num)
                } else if let Some((id, n)) = match_id(&input[index..]) {
                    consumed = n;
                    Token::Id(id)
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

fn match_id(text: &str) -> Match<&str> {
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
            return Some((&text[..index], index));
        }
    }
    None
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
    use super::Token::*;
    use super::*;
    use float_cmp::approx_eq;
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn test_match_numbers(f in prop::num::f64::NORMAL) {
            let str = f.to_string();
            let res = match_number(&str);
            prop_assert!(res.is_some());
            let res = res.unwrap();
            prop_assert!(approx_eq!(f64, f, res.0));
            prop_assert_eq!(str.len(), res.1);
        }
        #[test]
        fn test_match_ids(s in r#"[a-zA-z](?:[a-zA-Z]|[0-9])*"#) {
            let res = match_id(&s);
            prop_assert!(res.is_some());
            let res = res.unwrap();
            prop_assert_eq!(&s, res.0);
            prop_assert_eq!(s.len(), res.1);
        }
    }

    // TODO: more test cases
    #[test]
    fn test_tokenize() {
        let expected = vec![
            vec![Num(1.0), Id("op"), Num(1.0)],
            vec![Id("-"), Num(1.0)],
            vec![Id("pi"), OpenParen, ClosedParen],
        ];
        let input = vec!["1.0 op 1.0", "- 1.0", "pi()"];
        for (expected, input) in expected.into_iter().zip(input) {
            let output = tokenize(input);
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
        let res = tokenize(s);
        assert_eq!(1, res.len());
        if let Token::BadToken(bad_token) = &res[0] {
            assert_eq!(s, *bad_token);
        }
    }
}
