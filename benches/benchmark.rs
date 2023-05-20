use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use beancount_parser_2::parse;
use rust_decimal::Decimal;

const SAMPLE: &str = include_str!("../tests/samples/official.beancount");

pub fn run_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse official example");
    group.significance_level(0.01);
    group.throughput(Throughput::Bytes(SAMPLE.len() as u64));
    group.measurement_time(Duration::from_secs(10));
    group.bench_function("parse decimal", |b| b.iter(|| parse::<Decimal>(SAMPLE)));
    group.bench_function("parse f64", |b| b.iter(|| parse::<f64>(SAMPLE)));
    group.bench_function("parse f32", |b| b.iter(|| parse::<f32>(SAMPLE)));
    group.finish();
}

criterion_group!(benches, run_bench);
criterion_main!(benches);
