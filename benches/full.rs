use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusty_yard::evaluator::eval_str_with_vars_and_ctx;
use rusty_yard::Ctx;
use std::collections::HashMap;
use std::iter::{once, repeat};

pub fn bench_default_ctx(c: &mut Criterion) {
    let one_operator = black_box(repeat("1.0").take(1000).collect::<Vec<_>>().join(" + "));
    let two_operators = black_box::<String>(
        repeat("10.0").take(500).collect::<Vec<_>>().join(" + ")
            + " * "
            + &repeat("20.0").take(500).collect::<Vec<_>>().join(" * "),
    );
    let one_operator_right_associative =
        black_box::<String>(repeat("1.0").take(1000).collect::<Vec<_>>().join(" ^ "));
    let mut vars = HashMap::new();
    let ctx = Ctx::default();
    let mut eval = |s: &str| eval_str_with_vars_and_ctx(s, &mut vars, &ctx);
    let mut g = c.benchmark_group("Operator");
    g.bench_function("one operator", |b| {
        b.iter(|| eval(&one_operator));
    });
    g.bench_function("two operators", |b| {
        b.iter(|| eval(&two_operators));
    });
    g.bench_function("one operator right assosiative", |b| {
        b.iter(|| eval(&one_operator_right_associative));
    });
    g.finish()
}

pub fn bench_nested_expression(c: &mut Criterion) {
    let no_fn = repeat("(")
        .take(1000)
        .chain(once("10.0"))
        .chain(repeat(")").take(1000))
        .collect::<Vec<_>>()
        .join("");
    let with_fn = repeat("sum(")
        .take(1000)
        .chain(once("10.0"))
        .chain(repeat(")").take(1000))
        .collect::<Vec<_>>()
        .join("");
    let mut vars = HashMap::new();
    let ctx = Ctx::default();
    let mut eval = |s: &str| eval_str_with_vars_and_ctx(s, &mut vars, &ctx);
    let mut g = c.benchmark_group("nested");
    g.bench_function("no functions", |b| {
        b.iter(|| eval(&no_fn));
    });
    g.bench_function("using functions", |b| {
        b.iter(|| eval(&with_fn));
    });
}

criterion_group!(benches, bench_default_ctx, bench_nested_expression);
criterion_main!(benches);
