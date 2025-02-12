use criterion::{BenchmarkId, Criterion, Throughput};

fn parse_save(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_save");

    let data = std::fs::read("../assets/saves/mp_Uesugi.eu4").unwrap();
    let zip = rawzip::ZipArchive::from_slice(&data).unwrap();
    let mut entries = zip.entries();
    let mut total_size = 0;
    while let Some(entry) = entries.next_entry().unwrap() {
        total_size += entry.uncompressed_size_hint();
    }

    let file_data = std::fs::read("../assets/eu4.txt").unwrap();
    let tokens = eu4save::BasicTokenResolver::from_text_lines(file_data.as_slice()).unwrap();

    group.throughput(Throughput::Bytes(total_size as u64));
    group.bench_function("text", |b| {
        b.iter(|| {
            let file = eu4save::Eu4File::from_slice(&data).unwrap();
            let _ = file.parse_save(&tokens);
        })
    });
    group.finish();
}

criterion::criterion_group!(benches, parse_save);
criterion::criterion_main!(benches);
