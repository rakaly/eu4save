use eu4save::{
    models::CountryEvent,
    query::{CountryPlayed, PlayerHistory},
};
use eu4save::{
    query::{BuildingConstruction, BuildingEvent, Query},
    CountryTag, Encoding, Eu4Date, Eu4Extractor, ProvinceId,
};
use std::io::{Cursor, Read};
use std::{collections::HashMap, error::Error};

mod utils;

#[test]
fn test_eu4_text() {
    let data = utils::request("eng.txt.eu4.zip");
    let reader = Cursor::new(&data[..]);
    let mut zip = zip::ZipArchive::new(reader).unwrap();
    let mut zip_file = zip.by_index(0).unwrap();
    let mut buffer = Vec::with_capacity(0);
    zip_file.read_to_end(&mut buffer).unwrap();
    let (save, encoding) = Eu4Extractor::extract_save(Cursor::new(&buffer)).unwrap();
    assert_eq!(encoding, Encoding::Text);
    assert_eq!(save.meta.player, CountryTag::from("ENG"));

    let query = Query::from_save(save);
    assert_eq!(query.starting_country(), Some(&CountryTag::from("ENG")));
    assert_eq!(
        query
            .player_names()
            .iter()
            .cloned()
            .collect::<Vec<String>>(),
        Vec::<String>::new()
    );

    let london = query
        .save()
        .game
        .provinces
        .get(&ProvinceId::from(236))
        .unwrap();
    assert_eq!(london.buildings.get("fort_15th"), Some(&true));
    assert_eq!(
        london.building_builders.get("fort_15th"),
        Some(&CountryTag::from("ENG"))
    );

    let histories = vec![PlayerHistory {
        tag: CountryTag::from("ENG"),
        is_human: true,
        exists: true,
        player_names: Vec::new(),
        played_tags: vec![CountryPlayed {
            tag: CountryTag::from("ENG"),
            start: Eu4Date::new(1444, 11, 11).unwrap(),
            end: Eu4Date::new(1444, 12, 4).unwrap(),
        }],
    }];

    assert_eq!(query.player_histories(), histories);
    let (save, _) = Eu4Extractor::extract_meta_optimistic(Cursor::new(&buffer)).unwrap();
    assert!(save.game.is_some());
}

#[test]
fn test_eu4_compressed_text() {
    let data = utils::request("eng.txt.compressed.eu4");
    let (save, encoding) = Eu4Extractor::extract_save(Cursor::new(&data[..])).unwrap();
    assert_eq!(encoding, Encoding::TextZip);
    assert_eq!(save.meta.player, CountryTag::from("ENG"));

    let (save, _) = Eu4Extractor::extract_meta_optimistic(Cursor::new(&data[..])).unwrap();
    assert!(save.game.is_none());
}

#[cfg(feature = "mmap")]
#[test]
fn test_eu4_compressed_text_mmap() {
    use eu4save::Extraction;
    let data = utils::request("eng.txt.compressed.eu4");
    let (save, encoding) = Eu4Extractor::builder()
        .with_extraction(Extraction::MmapTemporaries)
        .extract_save(Cursor::new(&data[..]))
        .unwrap();
    assert_eq!(encoding, Encoding::TextZip);
    assert_eq!(save.meta.player, CountryTag::from("ENG"));
}

#[test]
pub fn parse_multiplayer_saves() -> Result<(), Box<dyn Error>> {
    let data = utils::request("mp_Uesugi.eu4");
    let (save, _encoding) = Eu4Extractor::extract_save(Cursor::new(&data[..])).unwrap();
    assert!(save.meta.multiplayer);

    let query = Query::from_save(save);

    let london = query
        .save()
        .game
        .provinces
        .get(&ProvinceId::from(236))
        .unwrap();
    let history = query.province_building_history(london);

    let marketplace_date = history
        .iter()
        .find(|x| x.building == "marketplace")
        .map(|x| x.date.game_fmt());
    assert_eq!(marketplace_date, Some(String::from("1506.5.25")));

    let fort_history: Vec<BuildingEvent> = history
        .iter()
        .filter(|x| x.building == "fort_15th")
        .cloned()
        .collect();
    let expected = vec![
        BuildingEvent {
            building: "fort_15th",
            date: Eu4Date::new(1444, 11, 11).unwrap(),
            action: BuildingConstruction::Constructed,
        },
        BuildingEvent {
            building: "fort_15th",
            date: Eu4Date::new(1476, 11, 2).unwrap(),
            action: BuildingConstruction::Destroyed,
        },
    ];

    assert_eq!(fort_history, expected);

    let stockholm = query
        .save()
        .game
        .provinces
        .get(&ProvinceId::from(1))
        .unwrap();
    let history2 = query.province_building_history(stockholm);
    assert_eq!(history2.iter().find(|x| x.building == "hre"), None);
    assert_eq!(history2.iter().find(|x| x.building == "is_city"), None);

    let histories: HashMap<_, _> = query
        .player_histories()
        .iter()
        .map(|p| (p.tag.clone(), p))
        .collect();

    assert_eq!(
        histories.get(&CountryTag::from("SAX")).unwrap(),
        &&PlayerHistory {
            tag: CountryTag::from("SAX"),
            is_human: false,
            exists: false,
            player_names: vec![String::from("Hose")],
            played_tags: vec![CountryPlayed {
                tag: CountryTag::from("SAX"),
                start: Eu4Date::new(1444, 11, 11).unwrap(),
                end: Eu4Date::new(1653, 11, 25).unwrap(),
            },],
        }
    );

    assert_eq!(
        histories.get(&CountryTag::from("GER")).unwrap(),
        &&PlayerHistory {
            tag: CountryTag::from("GER"),
            is_human: true,
            exists: true,
            player_names: vec![String::from("Doge of Venice (Taran)")],
            played_tags: vec![
                CountryPlayed {
                    tag: CountryTag::from("HSA"),
                    start: Eu4Date::new(1444, 11, 11).unwrap(),
                    end: Eu4Date::new(1598, 11, 8).unwrap(),
                },
                CountryPlayed {
                    tag: CountryTag::from("WES"),
                    start: Eu4Date::new(1598, 11, 8).unwrap(),
                    end: Eu4Date::new(1603, 11, 29).unwrap(),
                },
                CountryPlayed {
                    tag: CountryTag::from("HAN"),
                    start: Eu4Date::new(1603, 11, 29).unwrap(),
                    end: Eu4Date::new(1737, 7, 20).unwrap(),
                },
                CountryPlayed {
                    tag: CountryTag::from("GER"),
                    start: Eu4Date::new(1737, 7, 20).unwrap(),
                    end: Eu4Date::new(1817, 8, 31).unwrap(),
                },
            ],
        }
    );

    assert_eq!(
        histories.get(&CountryTag::from("FRA")).unwrap(),
        &&PlayerHistory {
            tag: CountryTag::from("FRA"),
            is_human: true,
            exists: true,
            player_names: vec![String::from("TheOnlySimen"), String::from("Strawman")],
            played_tags: vec![
                CountryPlayed {
                    tag: CountryTag::from("SCO"),
                    start: Eu4Date::new(1444, 11, 11).unwrap(),
                    end: Eu4Date::new(1533, 1, 25).unwrap(),
                },
                CountryPlayed {
                    tag: CountryTag::from("IRE"),
                    start: Eu4Date::new(1533, 1, 25).unwrap(),
                    end: Eu4Date::new(1644, 1, 11).unwrap(),
                },
                CountryPlayed {
                    tag: CountryTag::from("FRA"),
                    start: Eu4Date::new(1644, 1, 11).unwrap(),
                    end: Eu4Date::new(1817, 8, 31).unwrap(),
                }
            ]
        }
    );

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

    let (save, encoding) = Eu4Extractor::extract_save(Cursor::new(&buffer[..])).unwrap();
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
    let (_save, encoding) = Eu4Extractor::extract_save(Cursor::new(&data[..])).unwrap();
    assert_eq!(encoding, Encoding::TextZip);
}
