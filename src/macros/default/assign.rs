use std::collections::HashMap;

use crate::macros::{Macro, MacroParse, ParsedMacro};
use crate::parser::ParseState;
use crate::tokenizer::{match_id, match_str, skip_whitespace, Match};
use crate::{evaluator, parser, Ctx};

/// The assign macro.
///
/// # Matching
///
/// This macro matches the following input:
/// ```text
/// {id}<spaces>=
/// ```
///
/// # Evaluation
///
/// This macro assigns the matched identifier the result of expression on the left of `=`
/// and returns that expression.
#[derive(Debug)]
pub struct Assign;

impl Macro for Assign {
    fn match_input(&self, input: &str) -> Match {
        let id_len = match_id(input)?;

        if (&input[..id_len]).ends_with('=') {
            return if id_len != 1 { Some(id_len) } else { None };
        }
        let whitespace = skip_whitespace(&input[id_len..]);
        let eq_len = match_str(&input[(id_len + whitespace)..], "=")?;
        Some(id_len + whitespace + eq_len)
    }

    fn parse<'a>(
        &self,
        input: &'a str,
        current_state: ParseState,
    ) -> Result<MacroParse<'a>, parser::Error> {
        if let ParseState::Operator = current_state {
            return Err(parser::Error::ExpectedExpression);
        }
        let len = match_id(input).unwrap();
        let id = if input.len() == len {
            &input[..len - 1]
        } else {
            &input[..len]
        };
        Ok(MacroParse::after(
            AssignParsed { id },
            ParseState::Expression,
        ))
    }
}

/// Parsed assign macro
#[derive(Debug)]
pub struct AssignParsed<'a> {
    id: &'a str,
}

impl<'a> AssignParsed<'a> {
    /// Creates a new instance of this parsed macro
    ///
    /// `id` is the name of the variable to assign the value into
    ///
    /// # Note
    ///
    /// In the sequence of parser tokens this macro comes **after**
    /// the expression which value will be assigned to macros variable.
    #[cfg_attr(tarpaulin, skip)]
    pub fn new(id: &'a str) -> Self {
        Self { id }
    }
}

impl<'a> ParsedMacro for AssignParsed<'a> {
    fn eval(
        &self,
        eval_stack: &mut Vec<f64>,
        variables: &mut HashMap<String, f64>,
        _ctx: &Ctx,
    ) -> Result<(), evaluator::Error> {
        let expr = *eval_stack.last().ok_or(evaluator::Error::EmptyEvalStack)?;
        variables.insert(self.id.into(), expr);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::macros::default::Assign;
    use crate::macros::{ApplyMode, Macro};
    use crate::parser;
    use crate::parser::ParseState;
    #[test]
    fn test_match_input() {
        let input_expected = &[
            ("a = 10", Some(3usize)),
            ("a = b", Some(3usize)),
            ("a =", Some(3usize)),
            // TODO v0.3: context aware tokenization
            ("a=10", None),
            ("a=b", None),
            ("a=", Some(2usize)),
            ("a==", Some(3usize)),
            ("=", None),
            // Rest of the cases are fine
            ("a= b", Some(2usize)),
            ("a= 10", Some(2usize)),
            ("10= ", None),
            ("10= ", None),
            ("a", None),
        ];
        for (input, expected) in input_expected {
            let result = Assign.match_input(input);
            assert_eq!(result, *expected, "input was {}", input);
        }
    }

    #[test]
    fn test_parse_ok() {
        let input = &["a = ", "a="];
        let expected_state = ParseState::Expression;
        let expected_binding = ApplyMode::After;
        for input in input {
            let result = Assign.parse(input, ParseState::Expression);
            assert!(result.is_ok(), "input = {}", input);
            let result = result.unwrap();
            assert_eq!(result.state_after, expected_state);
            assert_eq!(result.mode, expected_binding);
        }
    }

    #[test]
    fn test_parse_err() {
        let input = "a = ";
        let result = Assign.parse(input, ParseState::Operator);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), parser::Error::ExpectedExpression);
    }

    #[test]
    #[should_panic]
    fn test_parse_panic() {
        let input = "1 won't bind";
        Assign
            .parse(input, ParseState::Expression)
            .expect("Panics before");
    }
}
