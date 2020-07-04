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
    fn match_input(&self, input: &str, ctx: &Ctx) -> Option<Match<()>> {
        let Match(id, c) = match_id(input, ctx)?;
        if id == "=" {
            None
        } else if let Some(c) = id.find('=') {
            Some(Match((), c + '='.len_utf8()))
        } else if id.ends_with('=') {
            Some(Match((), c))
        } else {
            let whitespace = skip_whitespace(&input[c..]);
            let Match(_, eq_len) = match_str(&input[(c + whitespace)..], "=")?;
            Some(Match((), c + whitespace + eq_len))
        }
    }

    fn parse<'a>(
        &self,
        input: &'a str,
        ctx: &Ctx,
        current_state: ParseState,
    ) -> Result<MacroParse<'a>, parser::Error> {
        if let ParseState::Operator = current_state {
            Err(parser::Error::ExpectedExpression)
        } else {
            let Match(id, len) = match_id(input, ctx).unwrap();
            let len = id.find('=').unwrap_or(len);
            Ok(MacroParse::after(
                AssignParsed { id: &id[..len] },
                ParseState::Expression,
            ))
        }
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
    use super::Assign;
    use crate::macros::{ApplyMode, Macro, MacroParse};
    use crate::parser::ParseState;
    use crate::{parser, Ctx};
    #[test]
    fn test_match_input() {
        let input_expected = &[
            ("a = 10", Some(3usize)),
            ("a = b", Some(3)),
            ("a =", Some(3)),
            ("a=10", Some(2)),
            ("a=b", Some(2)),
            ("a=", Some(2)),
            ("a==", Some(2)),
            ("a= b", Some(2)),
            ("a= 10", Some(2)),
            ("10= ", None),
            ("10= ", None),
            ("a", None),
            ("=", None),
        ];
        let ctx = &Ctx::empty();
        for (input, expected) in input_expected {
            let result = Assign.match_input(input, ctx).map(|m| m.1);
            assert_eq!(result, *expected, "input was {}", input);
        }
    }

    #[test]
    fn test_parse_ok() {
        let input = &["a = ", "a="];
        let expected_state = ParseState::Expression;
        let expected_binding = ApplyMode::After;
        let ctx = &Ctx::empty();
        for input in input {
            let result = Assign.parse(input, &ctx, ParseState::Expression);
            assert!(result.is_ok(), "input = {}", input);
            let MacroParse {
                result: _,
                mode,
                state_after,
            } = result.unwrap();
            assert_eq!(state_after, expected_state);
            assert_eq!(mode, expected_binding);
        }
    }

    #[test]
    fn test_parse_err() {
        let input = "a = ";
        let ctx = &Ctx::empty();
        let result = Assign.parse(input, ctx, ParseState::Operator);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), parser::Error::ExpectedExpression);
    }

    #[test]
    #[should_panic]
    fn test_parse_panic() {
        let ctx = &Ctx::empty();
        let input = "1 won't bind";
        Assign
            .parse(input, ctx, ParseState::Expression)
            .expect("Panics before");
    }
}
