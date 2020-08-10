use eu4save::{
    query::{CountryQuery, Query},
    CountryEvent, CountryTag, Encoding, Eu4Extractor, GameDifficulty, ProvinceId,
    TaxManpowerModifier,
};
use std::collections::HashSet;
use std::error::Error;
use std::io::{Cursor, Read};

mod utils;

#[test]
fn test_eu4_text() {
    let data = utils::request("eng.txt.eu4.zip");
    let reader = Cursor::new(&data[..]);
    let mut zip = zip::ZipArchive::new(reader).unwrap();
    let mut zip_file = zip.by_index(0).unwrap();
    let mut buffer = Vec::with_capacity(0);
    zip_file.read_to_end(&mut buffer).unwrap();
    let extractor = Eu4Extractor::default();
    let (save, encoding) = extractor.extract_save(Cursor::new(&buffer)).unwrap();
    assert_eq!(encoding, Encoding::Text);
    assert_eq!(save.meta.player, CountryTag::from("ENG"));

    let query = Query::from_save(save);
    assert_eq!(query.starting_country, Some(CountryTag::from("ENG")));
    assert_eq!(
        query.players.iter().cloned().collect::<Vec<String>>(),
        Vec::<String>::new()
    );

    let (save, _) = extractor
        .extract_meta_optimistic(Cursor::new(&buffer))
        .unwrap();
    assert!(save.game.is_some());
}

#[test]
fn test_eu4_compressed_text() {
    let data = utils::request("eng.txt.compressed.eu4");
    let extractor = Eu4Extractor::default();
    let (save, encoding) = extractor.extract_save(Cursor::new(&data[..])).unwrap();
    assert_eq!(encoding, Encoding::TextZip);
    assert_eq!(save.meta.player, CountryTag::from("ENG"));

    let (save, _) = extractor
        .extract_meta_optimistic(Cursor::new(&data[..]))
        .unwrap();
    assert!(save.game.is_none());
}

#[cfg(feature = "mmap")]
#[test]
fn test_eu4_compressed_text_mmap() {
    use eu4save::Extraction;
    let data = utils::request("eng.txt.compressed.eu4");
    let extractor = Eu4Extractor::new(Extraction::MmapTemporaries);
    let (save, encoding) = extractor.extract_save(Cursor::new(&data[..])).unwrap();
    assert_eq!(encoding, Encoding::TextZip);
    assert_eq!(save.meta.player, CountryTag::from("ENG"));
}

#[cfg(ironman)]
#[test]
fn test_eu4_bin() {
    let data = utils::request("ragusa2.bin.eu4");
    let extractor = Eu4Extractor::default();
    let (save, encoding) = extractor.extract_save(Cursor::new(&data[..])).unwrap();
    assert_eq!(encoding, Encoding::BinZip);
    assert_eq!(save.meta.player, CountryTag::from("CRO"));

    let (save2, _) = extractor
        .extract_meta_optimistic(Cursor::new(&data[..]))
        .unwrap();
    assert!(save2.game.is_none());

    let query = Query::from_save(save);
    assert_eq!(query.starting_country, Some(CountryTag::from("RAG")));
    assert_eq!(
        query.players.iter().cloned().collect::<Vec<String>>(),
        Vec::<String>::new()
    );

    let mut players = HashSet::new();
    players.insert(CountryTag::from("RAG"));
    players.insert(CountryTag::from("CRO"));
    assert_eq!(query.player_countries, players);
}

#[test]
pub fn parse_multiplayer_saves() -> Result<(), Box<dyn Error>> {
    let data = utils::request("mp_Uesugi.eu4");
    let extractor = Eu4Extractor::default();
    let (save, _encoding) = extractor.extract_save(Cursor::new(&data[..])).unwrap();
    assert!(save.meta.multiplayer);
    Ok(())
}

#[cfg(ironman)]
#[test]
fn test_eu4_kandy_bin() {
    let data = utils::request("kandy2.bin.eu4");
    let extractor = Eu4Extractor::default();
    let (save, encoding) = extractor.extract_save(Cursor::new(&data[..])).unwrap();
    assert_eq!(encoding, Encoding::BinZip);
    assert_eq!(save.meta.player, CountryTag::from("BHA"));

    let query = Query::from_save(save);
    let mut players = HashSet::new();
    players.insert(CountryTag::from("KND"));
    players.insert(CountryTag::from("BHA"));
    assert_eq!(query.player_countries, players);
    assert!(!query
        .save
        .game
        .countries
        .get(&CountryTag::from("BHA"))
        .unwrap()
        .completed_missions
        .is_empty());

    assert_eq!(
        query.country_tag_hex_color(&CountryTag::from("BHA")),
        Some(String::from("#50a50a"))
    );

    assert_eq!(
        query
            .save
            .game
            .provinces
            .get(&ProvinceId::from(1))
            .unwrap()
            .owner
            .as_ref()
            .unwrap(),
        &CountryTag::from("SCA")
    );

    assert_eq!(query.starting_country, Some(CountryTag::from("KND")));
    assert_eq!(
        query.players.iter().cloned().collect::<Vec<_>>(),
        vec![String::from("comagoosie")]
    );

    let subjects: Vec<CountryTag> = vec![
        CountryTag::from("TEO"),
        CountryTag::from("YOK"),
        CountryTag::from("C21"),
        CountryTag::from("C23"),
    ];

    assert_eq!(
        query
            .save
            .game
            .countries
            .get(&CountryTag::from("BHA"))
            .unwrap()
            .subjects,
        subjects
    );

    let blank: Vec<String> = Vec::new();
    let ledgers = query.annual_ledgers(&[CountryQuery::Greats], &blank, &blank);

    // When querying for great powers in the ledger and a current great power is from a reformed
    // country (like russia or great britain), ensure that their predecessor is included.
    let mos = ledgers.income.iter().find(|&x| x.name.as_str() == "MOS");
    assert!(mos.is_some());

    // I had a score of zero in 1450, but the ledger doesn't report zero values
    let knd_score = ledgers.score.iter().find(|&l| {
        l.name.as_str() == "KND" && l.data.iter().find(|(x, y)| *x == 1450 && *y == 0).is_some()
    });
    assert!(knd_score.is_some());
}

#[cfg(ironman)]
#[test]
fn test_eu4_ita1() {
    let data = utils::request("ita1.eu4");
    let extractor = Eu4Extractor::default();
    let (save, encoding) = extractor.extract_save(Cursor::new(&data[..])).unwrap();
    assert_eq!(encoding, Encoding::BinZip);
    assert_eq!(save.meta.player, CountryTag::from("ITA"));
    let settings = &save.game.gameplay_settings.options;
    assert_eq!(settings.difficulty, GameDifficulty::Normal);
    assert_eq!(
        settings.tax_manpower_modifier,
        TaxManpowerModifier::Historical
    );

    let all_dlc_recognized = save
        .meta
        .dlc_enabled
        .iter()
        .map(|x| eu4save::dlc::dlc_id(x.as_str()))
        .all(|x| x.is_some());
    assert!(all_dlc_recognized);

    let query = Query::from_save(save);
    assert_eq!(query.starting_country, Some(CountryTag::from("LAN")));
    assert_eq!(
        query.players.iter().cloned().collect::<Vec<_>>(),
        vec![String::from("comagoosie")]
    );
}

#[cfg(ironman)]
#[test]
fn test_eu4_same_campaign_id() {
    let data = utils::request("ita2.eu4");
    let data2 = utils::request("ita2_later.eu4");
    let extractor = Eu4Extractor::default();
    let (save, _) = extractor.extract_save(Cursor::new(&data[..])).unwrap();
    let (save2, _) = extractor.extract_save(Cursor::new(&data2[..])).unwrap();
    assert_eq!(save.meta.campaign_id, save2.meta.campaign_id);
    assert!(save.meta.date < save2.meta.date);
}

#[cfg(ironman)]
#[test]
fn test_roundtrip_melt() {
    let data = utils::request("kandy2.bin.eu4");
    let out = eu4save::melt(&data[..], eu4save::FailedResolveStrategy::Error).unwrap();
    let extractor = Eu4Extractor::default();
    let (save, encoding) = extractor.extract_save(Cursor::new(&out[..])).unwrap();
    assert_eq!(encoding, Encoding::Text);
    assert_eq!(save.meta.player, CountryTag::from("BHA"));
}

#[test]
fn test_missing_leader_activation_save() {
    let data = utils::request("skan-cb25b0.eu4.zip");
    let reader = Cursor::new(&data[..]);
    let mut zip = zip::ZipArchive::new(reader).unwrap();
    let mut zip_file = zip.by_index(0).unwrap();
    let mut buffer = Vec::with_capacity(0);
    zip_file.read_to_end(&mut buffer).unwrap();

    let extractor = Eu4Extractor::default();
    let (save, encoding) = extractor.extract_save(Cursor::new(&buffer[..])).unwrap();
    assert_eq!(encoding, Encoding::Text);
    assert_eq!(save.meta.player, CountryTag::from("NED"));

    let none_activation: Vec<_> = save
        .game
        .countries
        .iter()
        .flat_map(|(_tag, country)| {
            country.history.events.iter().flat_map(|(_date, events)| {
                events.0.iter().filter_map(|event| match event {
                    CountryEvent::Monarch(x) => x.leader.as_ref(),
                    CountryEvent::Leader(x) => Some(x),
                    _ => None,
                })
            })
        })
        .filter(|x| x.activation.is_none())
        .collect();

    assert_eq!(none_activation.len(), 8);
}

#[test]
fn test_handle_heavily_nested_events() {
    // There are some events that nests inside itself every time it fires. One such event is
    // "flavor_eng.9880" or the symposium event for great britain where every ten years the event
    // fires. https://eu4.paradoxwikis.com/English_events#Symposium
    let data = utils::request("HashMP_Game56_S7End.eu4");
    let extractor = Eu4Extractor::default();
    let (_save, encoding) = extractor.extract_save(Cursor::new(&data[..])).unwrap();
    assert_eq!(encoding, Encoding::TextZip);
}
