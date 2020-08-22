use eu4save::{
    query::Query, CountryEvent, CountryTag, Encoding, Eu4Extractor, ProvinceEvent,
    ProvinceEventValue, ProvinceId,
};
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

    let london = query
        .save
        .game
        .provinces
        .get(&ProvinceId::from(236))
        .unwrap();
    assert_eq!(london.buildings.get("fort_15th"), Some(&true));
    assert_eq!(
        london.building_builders.get("fort_15th"),
        Some(&CountryTag::from("ENG"))
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

#[test]
pub fn parse_multiplayer_saves() -> Result<(), Box<dyn Error>> {
    let data = utils::request("mp_Uesugi.eu4");
    let extractor = Eu4Extractor::default();
    let (save, _encoding) = extractor.extract_save(Cursor::new(&data[..])).unwrap();
    assert!(save.meta.multiplayer);

    // Testing text encoded saves can extract province building history perfectly fine
    let london = save.game.provinces.get(&ProvinceId::from(236)).unwrap();
    let (date, _marketplace, val) = london
        .history
        .events
        .iter()
        .flat_map(|(date, events)| events.0.iter().map(move |event| (date.clone(), event)))
        .filter_map(|(date, event)| match event {
            ProvinceEvent::KV((key, value)) => Some((date, key, value)),
            _ => None,
        })
        .find(|(_date, key, _value)| key.as_str() == "marketplace")
        .unwrap();
    assert!(matches!(val, ProvinceEventValue::Bool(v) if v == &true));
    assert_eq!(date.eu4_fmt().as_str(), "1506.5.25");
    Ok(())
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
