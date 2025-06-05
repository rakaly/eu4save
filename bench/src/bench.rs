use criterion::{Criterion, Throughput};
use eu4save::flavor::Eu4Flavor;
use jomini::JominiDeserialize;
use serde::{
    de::{self, SeqAccess},
    Deserialize, Deserializer,
};
use std::{
    collections::HashMap,
    fmt,
    hash::{BuildHasherDefault, Hasher},
};

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

fn deserialize_ledger(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialize");

    #[derive(Debug, Clone, JominiDeserialize)]
    #[expect(dead_code)]
    struct CountryLedger {
        #[jomini(
            default = "array_default",
            deserialize_with = "deserialize_list",
            token = 0x287f
        )]
        income: [f32; 19],
        #[jomini(
            default = "array_default",
            deserialize_with = "deserialize_list",
            token = 0x2880
        )]
        expense: [f32; 38],
        #[jomini(alias = "lastmonthincome", token = 0x2879)]
        last_month_income: Option<f32>,
        #[jomini(
            alias = "lastmonthincometable",
            default = "array_default",
            deserialize_with = "deserialize_list",
            token = 0x287b
        )]
        last_month_income_table: [f32; 19],
        #[jomini(
            alias = "lastmonthexpensetable",
            default = "array_default",
            deserialize_with = "deserialize_list",
            token = 0x287c
        )]
        last_month_expense_table: [f32; 38],
        #[jomini(
            alias = "totalexpensetable",
            default = "array_default",
            deserialize_with = "deserialize_list",
            token = 0x34a4
        )]
        total_expense_table: [f32; 38],
        #[jomini(
            alias = "lastyearincome",
            default = "array_default",
            deserialize_with = "deserialize_list",
            token = 0x287d
        )]
        last_year_income: [f32; 19],
        #[jomini(
            alias = "lastyearexpense",
            default = "array_default",
            deserialize_with = "deserialize_list",
            token = 0x287e
        )]
        last_year_expense: [f32; 38],
    }

    let data = r#"
			income={
				160.500 539.364 817.406 52.862 20.882 1.665 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 5.652
			}
			expense={
				39.736 0.000 72.992 0.000 12.773 0.000 153.668 82.705 39.690 8.080 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000
			}
			lastmonthincome=1598.331
			lastmonthincometable={
				160.500 539.364 817.406 52.862 20.882 1.665 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 5.652
			}
			lastmonthexpense=409.644
			lastmonthexpensetable={
				39.736 0.000 72.992 0.000 12.773 0.000 153.668 82.705 39.690 8.080 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000
			}
			totalexpensetable={
				57281.152 8858.215 53707.281 151.200 35162.777 0.000 182451.328 41266.332 28286.047 10804.620 0.000 6979.546 15015.400 0.000 308928.188 0.000 65326.508 3621.690 16936.703 86527.047 0.000 0.000 0.000 0.000 0.000 0.000 0.000 6571.160 16624.076 157.956 0.000 36739.602 0.000 74.205 194.857 159112.000 27260.199 102.400
			}
			lastyearincome={
				1905.523 6395.835 9682.316 634.344 247.500 19.980 0.000 0.000 0.000 0.000 0.000 0.000 0.000 597.321 0.000 0.000 0.000 26.108 108.956
			}
			lastyearexpense={
				476.682 0.000 875.137 0.000 153.276 0.000 2102.549 992.460 426.870 80.840 0.000 0.000 575.700 0.000 21585.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 9108.000 0.000 0.000 0.000 0.000 0.000 0.000
			}
			last_months_recurring_income=1598.331
			last_months_recurring_expenses=409.644
"#;

    let data2 = r#"
			incom1={
				160.500 539.364 817.406 52.862 20.882 1.665 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 5.652
			}
			expens1={
				39.736 0.000 72.992 0.000 12.773 0.000 153.668 82.705 39.690 8.080 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000
			}
			lastmonthincom1=1598.331
			lastmonthincometabl1={
				160.500 539.364 817.406 52.862 20.882 1.665 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 5.652
			}
			lastmonthexpens1=409.644
			lastmonthexpensetabl1={
				39.736 0.000 72.992 0.000 12.773 0.000 153.668 82.705 39.690 8.080 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000
			}
			totalexpensetabl1={
				57281.152 8858.215 53707.281 151.200 35162.777 0.000 182451.328 41266.332 28286.047 10804.620 0.000 6979.546 15015.400 0.000 308928.188 0.000 65326.508 3621.690 16936.703 86527.047 0.000 0.000 0.000 0.000 0.000 0.000 0.000 6571.160 16624.076 157.956 0.000 36739.602 0.000 74.205 194.857 159112.000 27260.199 102.400
			}
			lastyearincom1={
				1905.523 6395.835 9682.316 634.344 247.500 19.980 0.000 0.000 0.000 0.000 0.000 0.000 0.000 597.321 0.000 0.000 0.000 26.108 108.956
			}
			lastyearexpens1={
				476.682 0.000 875.137 0.000 153.276 0.000 2102.549 992.460 426.870 80.840 0.000 0.000 575.700 0.000 21585.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 0.000 9108.000 0.000 0.000 0.000 0.000 0.000 0.000
			}
			last_months_recurring_incom1=1598.331
			last_months_recurring_expense1=409.644
"#;

    let binary_data = include_bytes!("../../assets/ledger.bin");

    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_function("matching-ledger", |b| {
        b.iter(|| {
            let reader = jomini::text::TokenReader::from_slice(data.as_bytes());
            let _: CountryLedger = jomini::TextDeserializer::from_windows1252_reader(reader)
                .deserialize()
                .unwrap();
        })
    });

    let resolver = HashMap::<u16, String>::new();
    group.bench_function("binary-ledger", |b| {
        b.iter(|| {
            let _: CountryLedger = jomini::BinaryDeserializer::builder_flavor(Eu4Flavor::new())
                .deserialize_slice(&binary_data[..], &resolver)
                .unwrap();
        })
    });

    group.bench_function("skip-ledger", |b| {
        b.iter(|| {
            let reader = jomini::text::TokenReader::from_slice(data2.as_bytes());
            let _: CountryLedger = jomini::TextDeserializer::from_windows1252_reader(reader)
                .deserialize()
                .unwrap();
        })
    });

    group.bench_function("parser", |b| {
        b.iter(|| {
            let mut reader = jomini::text::TokenReader::from_slice(data.as_bytes());
            while let Ok(Some(_)) = reader.next() {}
        })
    });

    group.finish();
}

fn country_tag_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("country_tag_hashing");

    let data = std::fs::read("../assets/saves/mp_Uesugi.eu4").unwrap();
    let file_data = std::fs::read("../assets/eu4.txt").unwrap_or_default();
    let segments = eu4save::SegmentedResolver::parse(file_data.as_slice()).unwrap();
    let file = eu4save::Eu4File::from_slice(&data).unwrap();
    let save = file.parse_save(&segments.resolver()).unwrap();
    let country_tags = save
        .game
        .countries
        .iter()
        .map(|(tag, _)| *tag)
        .collect::<Vec<_>>();

    group.throughput(Throughput::Elements(country_tags.len() as u64));
    group.bench_function("default-hash", |b| {
        let map = country_tags
            .iter()
            .enumerate()
            .map(|(i, tag)| (*tag, i))
            .collect::<HashMap<_, _>>();
        b.iter(|| {
            let mut sum = 0;
            for tag in &country_tags {
                sum += map.get(tag).unwrap_or(&0);
            }
            std::hint::black_box(sum)
        })
    });

    type FnvBuildHasher = BuildHasherDefault<FnvHasher>;
    pub type FnvHashMap<K, V> = HashMap<K, V, FnvBuildHasher>;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FnvHasher(u64);

    impl Default for FnvHasher {
        #[inline]
        fn default() -> FnvHasher {
            FnvHasher(0xcbf29ce484222325)
        }
    }

    impl Hasher for FnvHasher {
        #[inline]
        fn finish(&self) -> u64 {
            self.0
        }

        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            let FnvHasher(mut hash) = *self;

            for byte in bytes {
                hash ^= *byte as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }

            *self = FnvHasher(hash);
        }
    }

    group.bench_function("fnv-hash", |b| {
        let map = country_tags
            .iter()
            .enumerate()
            .map(|(i, tag)| (*tag, i))
            .collect::<FnvHashMap<_, _>>();
        b.iter(|| {
            let mut sum = 0;
            for tag in &country_tags {
                sum += map.get(tag).unwrap_or(&0);
            }
            std::hint::black_box(sum)
        })
    });
}

criterion::criterion_group!(benches, parse_save, deserialize_ledger, country_tag_hashing);
criterion::criterion_main!(benches);

fn collect_into_default<'de, A, T, const N: usize>(
    mut seq: A,
) -> Result<[T; N], <A as SeqAccess<'de>>::Error>
where
    A: SeqAccess<'de>,
    T: Default + Copy + Deserialize<'de>,
{
    let mut result = [T::default(); N];
    for i in 0..N {
        let Some(x) = seq.next_element::<T>()? else {
            return Ok(result);
        };
        result[i] = x;
    }

    // If the sequence is not finished, we need to consume the rest of the elements
    // so that we drive a potential parser that underlies the deserializer
    while let Some(_x) = seq.next_element::<de::IgnoredAny>()? {}
    Ok(result)
}

fn deserialize_list<'de, D, const N: usize>(deserializer: D) -> Result<[f32; N], D::Error>
where
    D: Deserializer<'de>,
{
    struct ListVisitor<const N: usize>;

    impl<'de, const N: usize> de::Visitor<'de> for ListVisitor<N> {
        type Value = [f32; N];

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a seq of bytes allowed to overflow")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            collect_into_default(seq)
        }
    }

    deserializer.deserialize_seq(ListVisitor)
}

fn array_default<const N: usize>() -> [f32; N] {
    [0.0; N]
}
