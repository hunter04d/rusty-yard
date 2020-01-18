use std::collections::HashMap;

use shunting_yard::evaluator;
use shunting_yard::evaluator::eval_str_with_vars_and_ctx;
use shunting_yard::Ctx;

#[test]
fn test_macro_assign() -> Result<(), evaluator::Error> {
    let input = "a = 10";
    let ctx = Ctx::default_with_macros();
    let mut vars = HashMap::<String, f64>::new();
    let res = eval_str_with_vars_and_ctx(input, &mut vars, &ctx)?;
    assert_eq!(vars.get("a"), Some(&10.0));
    assert_eq!(res, 10.0);
    Ok(())
}
