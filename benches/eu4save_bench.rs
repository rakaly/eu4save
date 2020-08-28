use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use eu4save::Eu4Date;

const METADATA_BIN: &'static [u8] = include_bytes!("../tests/fixtures/meta.bin");

pub fn melt_benchmark(c: &mut Criterion) {
    let data = &METADATA_BIN[..];
    let mut group = c.benchmark_group("melt");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function("metadata", |b| {
        b.iter(|| eu4save::melt(data, eu4save::FailedResolveStrategy::Ignore).unwrap())
    });
    group.finish();
}

pub fn eu4_date_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("eu4date-parse");
    group.bench_function("valid-date", |b| {
        b.iter(|| Eu4Date::parse_from_str("1444.11.11").unwrap())
    });
    group.bench_function("invalid-date", |b| {
        b.iter(|| Eu4Date::parse_from_str("marketplace").is_none())
    });
    group.bench_function("long-invalid-date", |b| {
        b.iter(|| Eu4Date::parse_from_str("incidents_bur_inheritance.5").is_none())
    });
    group.finish();
}

criterion_group!(benches, melt_benchmark, eu4_date_benchmark);
criterion_main!(benches);
