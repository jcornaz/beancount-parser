#![allow(missing_docs)]

use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use beancount_parser::BeancountFile;

const SAMPLE: &str = include_str!("../tests/samples/official.beancount");

pub fn run_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse official example");
    group.significance_level(0.01);
    group.throughput(Throughput::Bytes(SAMPLE.len() as u64));
    group.measurement_time(Duration::from_secs(10));
    group.bench_function("parse", |b| b.iter(|| SAMPLE.parse::<BeancountFile<f64>>()));
    group.finish();
}

criterion_group!(benches, run_bench);
criterion_main!(benches);
