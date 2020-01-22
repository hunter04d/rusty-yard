use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusty_yard::evaluator::eval_str_with_vars_and_ctx;
use rusty_yard::Ctx;
use std::collections::HashMap;
use std::iter::repeat;

pub fn bench_one_operator_default_ctx(c: &mut Criterion) {
    let input = black_box(repeat("10.0").take(1000).collect::<Vec<_>>().join(" + "));
    let mut vars = HashMap::new();
    let ctx = Ctx::default();
    c.bench_function("one operator [default ctx]", |b| {
        b.iter(|| eval_str_with_vars_and_ctx(&input, &mut vars, &ctx))
    });
}

pub fn bench_two_operators_default_ctx(c: &mut Criterion) {
    let input = black_box::<String>(
        repeat("10.0").take(500).collect::<Vec<_>>().join(" + ")
            + " * "
            + &repeat("20.0").take(500).collect::<Vec<_>>().join(" * "),
    );
    let mut vars = HashMap::new();
    let ctx = Ctx::default();
    c.bench_function("two operators [default ctx]", |b| {
        b.iter(|| eval_str_with_vars_and_ctx(&input, &mut vars, &ctx))
    });
}

criterion_group!(
    benches,
    bench_one_operator_default_ctx,
    bench_two_operators_default_ctx
);
criterion_main!(benches);
