use super::Ctx;
use super::tokenizer::Match;
use std::collections::HashMap;

pub trait Macro {
    fn match_input(&self, input: &str) -> Match;

    fn parse<'a>(&self, input: &'a str) -> Box<dyn ParsedMacro + 'a>;
}

pub trait ParsedMacro {
    fn eval(&mut self, eval_stack: &mut Vec<f64>, variables: &mut HashMap<String, f64>, ctx: &Ctx);
}
