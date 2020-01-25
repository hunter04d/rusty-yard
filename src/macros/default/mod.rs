//! Provides some default macros, and their parsed variants.
pub use assign::Assign;
// TODO v0.3: move to mod parsed
use crate::macros::Macro;
pub use assign::AssignParsed;

mod assign;

/// Get the list of default macros
///
/// This includes all macros from [`macros::default`](self) module
pub fn default_macros() -> Vec<Box<dyn Macro>> {
    vec![Box::new(Assign)]
}
