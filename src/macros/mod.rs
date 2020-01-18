use std::collections::HashMap;
use std::fmt::Debug;

use crate::macros::ApplyMode::Before;
use crate::parser::ParseState;
use crate::{evaluator, parser};

use super::tokenizer::Match;
use super::Ctx;

pub mod default;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ApplyMode {
    Before,
    After,
}

impl Default for ApplyMode {
    fn default() -> Self {
        Before
    }
}

pub struct MacroParse<'a> {
    pub(crate) result: Box<dyn ParsedMacro + 'a>,
    pub(crate) mode: ApplyMode,
    pub(crate) state_after: ParseState,
    // other fields are possible
}

impl<'a> MacroParse<'a> {
    pub fn before(result: impl ParsedMacro + 'a, expected_state: ParseState) -> Self {
        MacroParse {
            result: Box::new(result),
            mode: ApplyMode::Before,
            state_after: expected_state,
        }
    }
    pub fn after(result: impl ParsedMacro + 'a, expected_state: ParseState) -> Self {
        MacroParse {
            result: Box::new(result),
            mode: ApplyMode::After,
            state_after: expected_state,
        }
    }
}

pub trait Macro: Debug {
    fn match_input(&self, input: &str) -> Match;

    fn parse<'a>(
        &self,
        input: &'a str,
        current_state: ParseState,
    ) -> Result<MacroParse<'a>, parser::Error>;
}

pub trait ParsedMacro: Debug {
    fn eval(
        &self,
        eval_stack: &mut Vec<f64>,
        variables: &mut HashMap<String, f64>,
        ctx: &Ctx,
    ) -> Result<(), evaluator::Error>;
}
