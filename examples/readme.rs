use std::collections::HashMap;

use rusty_yard::{evaluator, Ctx};
use rusty_yard::operators::UOp;

fn main() {
    simple();
    with_variables();
    with_context();
    macros();
}

fn simple() {
    let result = evaluator::eval_str("10 + 10 * 10").unwrap();

    assert_eq!(110.0, result);
    println!("simple example {}", result);
}

fn with_variables() {
    let mut vars = HashMap::new();
    vars.insert("a".to_owned(), 1.0);
    vars.insert("b".to_owned(), 2.0);
    vars.insert("c".to_owned(), 3.0);
    // vars is mut because macros can modify the content of the map
    let result = evaluator::eval_str_with_vars("a + b * c", &mut vars).unwrap();

    assert_eq!(7.0, result);
    println!("example with variables: {}", result);
}

fn with_context() {
    let mut vars = HashMap::new();
    // default ctx with operators one might expect for shunting yard
    let mut ctx = Ctx::default();
    // add $$$ operator with some action
    ctx.u_ops.insert(UOp {
        token: "$$$".to_owned(),
        func: |v| v * 1000.0,
    });

    let result = evaluator::eval_str_with_vars_and_ctx("$$$42.0", &mut vars, &ctx).unwrap();

    assert_eq!(42.0 * 1000.0, result);
    println!("example with custom unary operator from ctx: {}", result)
}

fn macros() {
    let mut vars = HashMap::new();
    // use default ctx with macros
    let ctx = Ctx::default_with_macros();

    // this is why vars is &mut
    // currently, only assign macro is defined
    let result = evaluator::eval_str_with_vars_and_ctx("a = 22.0 + 20.0", &mut vars, &ctx).unwrap();

    assert_eq!(42.0, vars["a"]);
    println!("macro example: a = {}", result)
}
