use criterion::{criterion_group, criterion_main, Criterion};

#[cfg(feature = "unstable")]
use beancount_parser::parse;
use beancount_parser::Parser;

const SAMPLE: &str = include_str!("../tests/samples/official.beancount");

pub fn run_bench(c: &mut Criterion) {
    c.bench_function("nom", |b| {
        b.iter(|| Parser::new(SAMPLE).collect::<Result<Vec<_>, _>>().unwrap())
    });

    #[cfg(feature = "unstable")]
    c.bench_function("pest", |b| {
        b.iter(|| parse(SAMPLE).unwrap().collect::<Vec<_>>())
    });
}

criterion_group!(benches, run_bench);
criterion_main!(benches);
