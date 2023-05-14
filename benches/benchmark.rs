use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use beancount_parser::Parser;
#[cfg(feature = "unstable")]
use beancount_parser::{parse, parse_to_vec};

const SAMPLE: &str = include_str!("../tests/samples/official.beancount");

pub fn run_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse official example");
    group.significance_level(0.01);
    group.throughput(Throughput::Bytes(SAMPLE.len() as u64));
    group.measurement_time(Duration::from_secs(10));
    group.bench_function("nom Parser", |b| {
        b.iter(|| Parser::new(SAMPLE).collect::<Result<Vec<_>, _>>().unwrap())
    });
    #[cfg(feature = "unstable")]
    group.bench_function("nom parse_to_vec", |b| {
        b.iter(|| parse_to_vec(SAMPLE).unwrap())
    });
    #[cfg(feature = "unstable")]
    group.bench_function("pest parse", |b| {
        b.iter(|| parse(SAMPLE).unwrap().collect::<Vec<_>>())
    });
    group.finish();
}

criterion_group!(benches, run_bench);
criterion_main!(benches);
