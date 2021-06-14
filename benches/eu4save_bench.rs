use criterion::{criterion_group, criterion_main, Criterion, Throughput};

const METADATA_BIN: &'static [u8] = include_bytes!("../tests/it/fixtures/meta.bin");

pub fn melt_benchmark(c: &mut Criterion) {
    let data = &METADATA_BIN[..];
    let mut group = c.benchmark_group("melt");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function("metadata", |b| {
        b.iter(|| {
            eu4save::Melter::new()
                .with_on_failed_resolve(eu4save::FailedResolveStrategy::Ignore)
                .melt(data)
                .unwrap()
        })
    });
    group.finish();
}

criterion_group!(benches, melt_benchmark);
criterion_main!(benches);
