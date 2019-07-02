#[derive(Debug)]
pub enum Token {
    Plus,
    Minus,
    Star,
    Slash,
    OpenParen,
    ClosedParen,
    ID(String),
    Number(f64),
    BadToken(String),
}

#[allow(dead_code)]
pub fn get_token_text(token: &Token) -> String {
    match token {
        Token::Plus => String::from("+"),
        Token::Minus => String::from("-"),
        Token::Star => String::from("*"),
        Token::Slash => String::from("/"),
        Token::OpenParen => String::from("("),
        Token::ClosedParen => String::from(")"),
        Token::ID(s) => s.clone(),
        Token::Number(n) => n.to_string(),
        Token::BadToken(s) => format!("<BAD TOKEN>({})", s)
    }
}

use std::ops::AddAssign;
pub fn tokenize(input: &str) -> Vec<Token> {
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
        let mut consumed = 1usize;
        let next_token = match ch {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '(' => Token::OpenParen,
            ')' => Token::ClosedParen,
            _ => {
                if let Some((id, n)) = match_id(&input[index..]) {
                    consumed = n;
                    Token::ID(id)
                } else if let Some((num, n)) = match_number(&input[index..]) {
                    consumed = n;
                    Token::Number(num)
                } else {
                    Token::BadToken(ch.to_string())
                }
            }
        };
        let mut last_reassigned = false;
        if !output.is_empty() {
            let last_token = output.last_mut().unwrap();
            if let (Token::BadToken(s1), Token::BadToken(s2)) = (last_token, &next_token) {
                last_reassigned = true;
                s1.add_assign(s2);
            }
        }
        if !last_reassigned {
            output.push(next_token);
        }
        if consumed > 1 {
            // one was consumed initially
            iterator.nth(consumed - 2);
        }
        index += consumed;
    }
    output
}

fn match_id(text: &str) -> Option<(String, usize)> {
    let mut iterator = text.chars();
    if let Some(ch) = iterator.next() {
        if ch.is_ascii_alphabetic() {
            let mut index = 1usize;
            while let Some(ch) = iterator.next() {
                if ch.is_ascii_alphanumeric() {
                    index += 1;
                    continue;
                }
                break;
            }
            return Some((text[..index].to_string(), index));
        }
    }
    None
}

fn match_number(text: &str) -> Option<(f64, usize)> {
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