use criterion::{Criterion, Throughput};

fn parse_save(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_save");

    let data = std::fs::read("../assets/saves/mp_Uesugi.eu4").unwrap();
    let zip = rawzip::ZipArchive::from_slice(&data).unwrap();
    let mut entries = zip.entries();
    let mut total_size = 0;
    while let Some(entry) = entries.next_entry().unwrap() {
        total_size += entry.uncompressed_size_hint();
    }

    let file_data = std::fs::read("../assets/eu4.txt").unwrap_or_default();
    let segments = eu4save::SegmentedResolver::parse(file_data.as_slice()).unwrap();

    group.throughput(Throughput::Bytes(total_size as u64));
    group.bench_function("text", |b| {
        b.iter(|| {
            let file = eu4save::Eu4File::from_slice(&data).unwrap();
            let _ = file.parse_save(&segments.resolver());
        })
    });

    let data = std::fs::read("../assets/saves/kandy2.bin.eu4").unwrap();
    let zip = rawzip::ZipArchive::from_slice(&data).unwrap();
    let mut entries = zip.entries();
    let mut total_size = 0;
    while let Some(entry) = entries.next_entry().unwrap() {
        total_size += entry.uncompressed_size_hint();
    }

    group.throughput(Throughput::Bytes(total_size as u64));
    group.bench_function("binary", |b| {
        b.iter(|| {
            let file = eu4save::Eu4File::from_slice(&data).unwrap();
            let _ = file.parse_save(&segments.resolver());
        })
    });

    group.bench_function("debug", |b| {
        b.iter(|| {
            let file = eu4save::Eu4File::from_slice(&data).unwrap();
            let eu4save::file::Eu4SliceFileKind::Zip(zip) = file.kind() else {
                return;
            };

            let resolver = segments.resolver();
            let meta = zip.get(eu4save::file::Eu4FileEntryName::Meta).unwrap();
            let mut deser = eu4save::file::Eu4Modeller::from_reader(meta, &resolver);
            let mut track = serde_path_to_error::Track::new();
            let deser = serde_path_to_error::Deserializer::new(&mut deser, &mut track);
            let mut erased = <dyn erased_serde::Deserializer>::erase(deser);
            let meta = erased_serde::deserialize(&mut erased).unwrap();

            let gamestate = zip.get(eu4save::file::Eu4FileEntryName::Gamestate).unwrap();
            let mut deser = eu4save::file::Eu4Modeller::from_reader(gamestate, &resolver);
            let mut track = serde_path_to_error::Track::new();
            let deser = serde_path_to_error::Deserializer::new(&mut deser, &mut track);
            let mut erased = <dyn erased_serde::Deserializer>::erase(deser);
            let game = erased_serde::deserialize(&mut erased).unwrap();

            let _ = eu4save::models::Eu4Save { meta, game };
        })
    });
    group.finish();
}

criterion::criterion_group!(benches, parse_save);
criterion::criterion_main!(benches);
