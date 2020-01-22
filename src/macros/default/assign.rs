use std::collections::HashMap;

use crate::macros::{Macro, MacroParse, ParsedMacro};
use crate::parser::ParseState;
use crate::tokenizer::{match_id, match_str, skip_whitespace, Match};
use crate::{evaluator, parser, Ctx};

#[derive(Debug)]
pub struct Assign;

impl Macro for Assign {
    fn match_input(&self, input: &str) -> Match {
        match_id(input)
            .map(|len| {
                let input = &input[len..];
                skip_whitespace(input) + len
            })
            .and_then(|len| {
                let input = &input[len..];
                match_str(input, "=").map(|m| m + len)
            })
    }

    fn parse<'a>(
        &self,
        input: &'a str,
        current_state: ParseState,
    ) -> Result<MacroParse<'a>, parser::Error> {
        if let ParseState::ExpectExpression = current_state {
            let len = match_id(input).unwrap();
            let id = &input[..len];
            Ok(MacroParse::after(
                AssignParsed { id },
                ParseState::ExpectExpression,
            ))
        } else {
            Err(parser::Error::ExpectedExpression)
        }
    }
}

#[derive(Debug)]
pub struct AssignParsed<'a> {
    id: &'a str,
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
    use crate::macros::Macro;
    use crate::tokenizer::Match;

    #[test]
    fn test_match_input() {
        let input = vec!["a = 10", "a = b", "a =", "10 = "];
        let expected = vec![Some(3usize), Some(3usize), Some(3usize), None];
        for (expected, input) in expected.into_iter().zip(input) {
            let result: Match = Assign.match_input(input);
            assert_eq!(expected, result);
        }
    }
}