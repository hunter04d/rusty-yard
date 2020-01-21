# Rusty Yard

## (name not final)

This library provides a generic implementation of the [shunting yard algorithm](https://en.wikipedia.org/wiki/Shunting-yard_algorithm])

## Disclaimer! API IS NOT STABLE

This is a simple project to learn the way of rust. As such the api can be rough in some places and is subject to change at any moment (patch versions are probably fine).

## Usage

For now the crate is only available via git:

```toml
[dependencies]
rusty_yard = { git = 'https://github.com/hunter04d/rusty-yard' }
```

Here is a simple example:

```rust
use rusty_yard::evaluator;

fn main() {
    let result = evaluator::eval_str("10 + 10 * 10").unwrap();

    assert_eq!(110.0, result);
}
```

You can also define variables using `HashMap<String, f64>` and evaluate the string with variables:

```rust
use std::collections::HashMap;

use rusty_yard::evaluator;

fn main() {

    let mut vars = HashMap::new();
    vars.insert("a".to_owned(), 1.0);
    vars.insert("b".to_owned(), 2.0);
    vars.insert("c".to_owned(), 3.0);
    // vars is mut because macros can modify the content of the map
    let result = evaluator::eval_str_with_vars("a + b * c", &mut vars).unwrap();

    assert_eq!(7.0, result);
}
```

### Custom context

The crate provides `Ctx` type with allows you to define your own operators / functions:

```rust
use std::collections::HashMap;

use rusty_yard::{evaluator, Ctx};
use rusty_yard::operators::UOp;

fn main() {

    let mut vars = HashMap::new();
    // default ctx with operators one might expect for shunting yard
    let mut ctx = Ctx::default();
    // add unary '$$$' operator with some action
    ctx.u_ops.insert(UOp {
        token: "$$$".to_owned(),
        func: |v| v * 1000.0,
    });
    // vars is mut because macros can modify the content of the map
    let result = evaluator::eval_str_with_vars_and_ctx("$$$42.0", &mut vars, &ctx).unwrap();

    assert_eq!(7.0, result);
}
```

### Macros

An interesting feature of this crate are macros. They allow you to hook into the execution of expression and to anything rust can do.

For example the `=` operator is defined as a macro:

```rust
use std::collections::HashMap;

use rusty_yard::{evaluator, Ctx};
use rusty_yard::operators::UOp;

fn main() {
    let mut vars = HashMap::new();
    // use default ctx with macros
    let ctx = Ctx::default_with_macros();

    // this is why vars is &mut
    // currently, only assign macro is defined
    let result = evaluator::eval_str_with_vars_and_ctx("a = 22.0 + 20.0", &mut vars, &ctx).unwrap();

    // a has been defined as 42.0
    assert_eq!(42.0, vars["a"]);
}
```

To implement your own macro you need to implement `Macro` and `ParsedMacro` trait. See [Assign macro](src/macros/default/assign.rs) for an example.

Note: the macros are even more experimental than the rest of the crate. Implementing your own macros is not recommended at this moment.

## This to do

- [ ] Document the crate
- [ ] More tests, a lot more tests
- [ ] Allow anything that implements `FromStr` to be used as primitive
- [ ] Allow customizing definition of id/num in tokenizer
- [ ] Helpers for matching, tokenizing, parsing, executing
- [ ] Think about the macro interface
