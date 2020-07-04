//! This module contains the necessary types to implement your own macros.
use std::collections::HashMap;
use std::fmt::Debug;

use crate::macros::ApplyMode::Before;
use crate::parser::ParseState;
use crate::{evaluator, parser};

use super::tokenizer::Match;
use super::Ctx;

pub mod default;

/// Specifies how the macro should be parsed in relation to other tokens.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ApplyMode {
    /// The macro will be put directly into output queue immediately after it was parsed
    ///
    /// # Example
    ///
    /// ```text
    /// .macro is some macro
    /// input = ".macro 120"
    /// the parsed token stream = "<Macro>(.macro) <Num>(120)"
    /// ```
    Before,
    /// The macro will be put into operator stack.
    ///
    /// Expression after the macro will be evaluated first by [`evaluator`](crate::evaluator).
    ///
    /// # Example
    ///
    /// ```text
    /// .macro is some macro
    /// input = ".macro 120"
    /// the parsed token stream = "<Num>(120) Macro>(.macro)"
    /// ```
    After,
}

impl Default for ApplyMode {
    /// The default and more natural apply mode is [`ApplyMode::Before`](ApplyMode::Before)
    fn default() -> Self {
        Before
    }
}

/// The result of parsing the macro
///
/// Contains information on how the parser should continue parsing after this macro has been parsed.
#[derive(Debug)]
pub struct MacroParse<'a> {
    pub(crate) result: Box<dyn ParsedMacro + 'a>,
    pub(crate) mode: ApplyMode,
    pub(crate) state_after: ParseState,
    // other fields are possible
}

impl<'a> MacroParse<'a> {
    /// Creates parsed macro with [`ApplyMode::Before`](ApplyMode::Before)
    ///
    /// `expected_state` is the state the macro expects the parser to be after the parsing of this macro
    pub fn before(result: impl ParsedMacro + 'a, expected_state: ParseState) -> Self {
        MacroParse {
            result: Box::new(result),
            mode: ApplyMode::Before,
            state_after: expected_state,
        }
    }

    /// Creates parsed macro with [`ApplyMode::Before`](ApplyMode::Before)
    ///
    /// `expected_state` is the state the macro expects the parser to be after the parsing of this macro
    pub fn after(result: impl ParsedMacro + 'a, expected_state: ParseState) -> Self {
        MacroParse {
            result: Box::new(result),
            mode: ApplyMode::After,
            state_after: expected_state,
        }
    }
}

/// Implement this trait (+ [`Debug`](std::fmt::Debug) to create your own macro).
pub trait Macro: Debug {
    /// Match the start of the `input` with this macro.
    ///
    /// Returns [`Some(length of the match)`](std::option::Option::Some) if the start of the `input` matched this macro
    /// and [`None`](std::option::Option::None) when input hasn't matched this macro.
    fn match_input(&self, input: &str, ctx: &Ctx) -> Option<Match<()>>;

    /// Parse this macro
    ///
    /// `input` contains exactly the string that was matched using [`match_input`](Macro::match_input) function.
    ///
    /// `current_state` contains the current state of the parser.
    fn parse<'a>(
        &self,
        input: &'a str,
        ctx: &Ctx,
        current_state: ParseState,
    ) -> Result<MacroParse<'a>, parser::Error>;
}

/// Represents the Parsed macro.
///
/// Types implementing this trait should contain all the information necessary to evaluate this macro.
///
/// Don't forget to derive or implement [`Debug`](std::fmt::Debug).
pub trait ParsedMacro: Debug {
    /// Evaluate this parsed macro
    ///
    /// Arguments contain the current state of the evaluator.
    fn eval(
        &self,
        eval_stack: &mut Vec<f64>,
        variables: &mut HashMap<String, f64>,
        ctx: &Ctx,
    ) -> Result<(), evaluator::Error>;
}
