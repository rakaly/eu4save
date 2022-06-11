use crate::utils;
use eu4save::{
    file::Eu4FileEntryName,
    models::{CountryEvent, Meta},
    query::{
        BuildingConstruction, BuildingEvent, NationEvent, NationEventKind, NationEvents,
        PlayerHistory, Query,
    },
    Encoding, EnvTokens, Eu4Date, Eu4File, PdsDate, ProvinceId,
};
use std::io::{Cursor, Read};
use std::{collections::HashMap, error::Error};

#[test]
fn test_eu4_text() -> Result<(), Box<dyn Error>> {
    let data = utils::request("eng.txt.eu4.zip");
    let reader = Cursor::new(&data[..]);
    let mut zip = zip::ZipArchive::new(reader)?;
    let mut zip_file = zip.by_index(0)?;
    let mut buffer = Vec::with_capacity(0);
    zip_file.read_to_end(&mut buffer)?;

    let file = Eu4File::from_slice(&buffer)?;
    let save = file.deserializer().build_save(&EnvTokens)?;

    assert_eq!(file.encoding(), Encoding::Text);
    assert_eq!(save.meta.player, "ENG".parse()?);

    let query = Query::from_save(save);
    let province_owners = query.province_owners();
    let nation_events = query.nation_events(&province_owners);
    let histories = query.player_histories(&nation_events);

    assert_eq!(query.starting_country(&histories), Some("ENG".parse()?));
    assert_eq!(query.players(), vec![]);
    let london = query
        .save()
        .game
        .provinces
        .get(&ProvinceId::from(236))
        .unwrap();
    assert_eq!(london.buildings.get("fort_15th"), Some(&true));
    assert_eq!(
        london.building_builders.get("fort_15th"),
        Some(&"ENG".parse()?)
    );

    let expected_histories = vec![PlayerHistory {
        history: NationEvents {
            initial: "ENG".parse().unwrap(),
            latest: "ENG".parse().unwrap(),
            stored: "ENG".parse().unwrap(),
            events: vec![],
        },
        is_human: true,
        player_names: Vec::new(),
    }];

    let reb_decision = query
        .country(&"REB".parse().unwrap())
        .unwrap()
        .decision_seed;
    assert_eq!(reb_decision, 684859145);

    assert_eq!(
        query.save().meta.date.iso_8601().to_string(),
        String::from("1444-12-04")
    );
    let inherit = query.inherit(&query.save_country(&"ENG".parse().unwrap()).unwrap());

    assert_eq!(inherit.subtotal, 6836);
    assert_eq!(inherit.inheritance_value, 80);
    assert_eq!(inherit.start_t0_year, 1464);
    assert_eq!(inherit.end_t0_year, 1538);
    assert_eq!(inherit.start_t1_year, 1539);
    assert_eq!(inherit.end_t1_year, 1543);
    assert_eq!(inherit.start_t2_year, 1444);
    assert_eq!(inherit.end_t2_year, 1463);

    assert_eq!(histories, expected_histories);

    Ok(())
}

#[test]
fn test_eu4_compressed_text() -> Result<(), Box<dyn Error>> {
    let data = utils::request("eng.txt.compressed.eu4");
    let file = Eu4File::from_slice(&data)?;
    let save = file.deserializer().build_save(&EnvTokens)?;
    assert_eq!(file.encoding(), Encoding::TextZip);
    assert_eq!(save.meta.player, "ENG".parse()?);

    Ok(())
}

#[test]
fn test_eu4_compressed_text_raw() -> Result<(), Box<dyn Error>> {
    let data = utils::request("eng.txt.compressed.eu4");
    let file = Eu4File::from_slice(&data)?;
    let mut entries = file.entries();
    let meta_entry = entries.next_entry().unwrap();
    assert_eq!(meta_entry.name(), Some(Eu4FileEntryName::Meta));
    let mut zip_sink = Vec::new();
    let meta_parsed = meta_entry.parse(&mut zip_sink)?;
    let meta: Meta = meta_parsed.deserializer().build(&EnvTokens)?;
    assert_eq!(file.encoding(), Encoding::TextZip);
    assert_eq!(meta.player, "ENG".parse()?);
    Ok(())
}

#[test]
pub fn parse_multiplayer_saves() -> Result<(), Box<dyn Error>> {
    let data = utils::request("mp_Uesugi.eu4");
    let file = Eu4File::from_slice(&data)?;
    let save = file.deserializer().build_save(&EnvTokens)?;
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
        .map(|x| x.date.game_fmt().to_string());
    assert_eq!(marketplace_date, Some(String::from("1506.5.25")));

    let fort_history: Vec<BuildingEvent> = history
        .iter()
        .filter(|x| x.building == "fort_15th")
        .cloned()
        .collect();
    let expected = vec![
        BuildingEvent {
            building: "fort_15th",
            date: Eu4Date::from_ymd(1444, 11, 11),
            action: BuildingConstruction::Constructed,
        },
        BuildingEvent {
            building: "fort_15th",
            date: Eu4Date::from_ymd(1476, 11, 2),
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

    let province_owners = query.province_owners();
    let nation_events = query.nation_events(&province_owners);

    let byz = nation_events
        .iter()
        .find(|x| x.initial.as_str() == "BYZ")
        .unwrap();
    assert_eq!(
        byz,
        &NationEvents {
            initial: "BYZ".parse().unwrap(),
            latest: "BYZ".parse().unwrap(),
            stored: "NPL".parse().unwrap(),
            events: vec![NationEvent {
                date: Eu4Date::from_ymd(1540, 9, 15),
                kind: NationEventKind::Annexed
            }],
        }
    );

    let hsn = nation_events
        .iter()
        .find(|x| x.initial.as_str() == "HSN")
        .unwrap();
    assert_eq!(
        hsn,
        &NationEvents {
            initial: "HSN".parse().unwrap(),
            latest: "BYZ".parse().unwrap(),
            stored: "BYZ".parse().unwrap(),
            events: vec![
                NationEvent {
                    date: Eu4Date::from_ymd(1701, 7, 11),
                    kind: NationEventKind::TagSwitch("NPL".parse().unwrap()),
                },
                NationEvent {
                    date: Eu4Date::from_ymd(1706, 10, 20),
                    kind: NationEventKind::TagSwitch("BYZ".parse().unwrap()),
                }
            ],
        }
    );

    let qom = nation_events
        .iter()
        .find(|x| x.initial.as_str() == "QOM")
        .unwrap();
    assert_eq!(
        qom,
        &NationEvents {
            initial: "QOM".parse().unwrap(),
            latest: "PER".parse().unwrap(),
            stored: "PER".parse().unwrap(),
            events: vec![
                NationEvent {
                    date: Eu4Date::from_ymd(1480, 1, 20),
                    kind: NationEventKind::TagSwitch("PER".parse().unwrap()),
                },
                NationEvent {
                    date: Eu4Date::from_ymd(1577, 4, 20),
                    kind: NationEventKind::Annexed,
                }
            ],
        }
    );

    let npl = nation_events.iter().find(|x| x.initial.as_str() == "NPL");
    assert_eq!(None, npl);

    let player_histories = query.player_histories(&nation_events);
    let histories: HashMap<_, _> = player_histories
        .iter()
        .map(|p| (p.history.latest, p))
        .collect();

    assert_eq!(histories.get(&"NPL".parse()?), None);

    assert_eq!(
        histories.get(&"SAX".parse()?).unwrap(),
        &&PlayerHistory {
            history: NationEvents {
                initial: "SAX".parse().unwrap(),
                latest: "SAX".parse().unwrap(),
                stored: "SAX".parse().unwrap(),
                events: vec![NationEvent {
                    date: Eu4Date::from_ymd(1653, 11, 25),
                    kind: NationEventKind::Annexed,
                }],
            },
            is_human: false,
            player_names: vec![String::from("Hose")],
        }
    );

    assert_eq!(
        histories.get(&"GER".parse()?).unwrap(),
        &&PlayerHistory {
            history: NationEvents {
                initial: "HSA".parse().unwrap(),
                latest: "GER".parse().unwrap(),
                stored: "GER".parse().unwrap(),
                events: vec![
                    NationEvent {
                        date: Eu4Date::from_ymd(1598, 11, 8),
                        kind: NationEventKind::TagSwitch("WES".parse().unwrap()),
                    },
                    NationEvent {
                        date: Eu4Date::from_ymd(1603, 11, 29),
                        kind: NationEventKind::TagSwitch("HAN".parse().unwrap()),
                    },
                    NationEvent {
                        date: Eu4Date::from_ymd(1737, 7, 20),
                        kind: NationEventKind::TagSwitch("GER".parse().unwrap()),
                    }
                ],
            },
            is_human: true,
            player_names: vec![String::from("Doge of Venice (Taran)")],
        }
    );

    assert_eq!(
        histories.get(&"FRA".parse()?).unwrap(),
        &&PlayerHistory {
            history: NationEvents {
                initial: "SCO".parse().unwrap(),
                latest: "FRA".parse().unwrap(),
                stored: "FRA".parse().unwrap(),
                events: vec![
                    NationEvent {
                        date: Eu4Date::from_ymd(1533, 1, 25),
                        kind: NationEventKind::TagSwitch("IRE".parse().unwrap()),
                    },
                    NationEvent {
                        date: Eu4Date::from_ymd(1644, 1, 11),
                        kind: NationEventKind::TagSwitch("FRA".parse().unwrap()),
                    }
                ]
            },
            is_human: true,
            player_names: vec![String::from("TheOnlySimen"), String::from("Strawman")],
        }
    );

    Ok(())
}

#[test]
fn test_missing_leader_activation_save() -> Result<(), Box<dyn Error>> {
    let data = utils::request("skan-cb25b0.eu4.zip");
    let reader = Cursor::new(&data[..]);
    let mut zip = zip::ZipArchive::new(reader)?;
    let mut zip_file = zip.by_index(0)?;
    let mut buffer = Vec::with_capacity(0);
    zip_file.read_to_end(&mut buffer)?;

    let file = Eu4File::from_slice(&buffer)?;
    let save = file.deserializer().build_save(&EnvTokens)?;
    assert_eq!(file.encoding(), Encoding::Text);
    assert_eq!(save.meta.player, "NED".parse()?);

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
    Ok(())
}

#[test]
fn test_handle_heavily_nested_events() {
    // There are some events that nests inside itself every time it fires. One such event is
    // "flavor_eng.9880" or the symposium event for great britain where every ten years the event
    // fires. https://eu4.paradoxwikis.com/English_events#Symposium
    let data = utils::request("HashMP_Game56_S7End.eu4");
    let file = Eu4File::from_slice(&data).unwrap();
    let _save = file.deserializer().build_save(&EnvTokens).unwrap();
    assert_eq!(file.encoding(), Encoding::TextZip);
}

#[test]
fn test_paperman_text() -> Result<(), Box<dyn Error>> {
    let data = utils::request("paperman.eu4.zip");
    let reader = Cursor::new(&data[..]);
    let mut zip = zip::ZipArchive::new(reader)?;
    let mut zip_file = zip.by_index(0)?;
    let mut buffer = Vec::with_capacity(0);
    zip_file.read_to_end(&mut buffer)?;
    let file = Eu4File::from_slice(&buffer).unwrap();
    let save = file.deserializer().build_save(&EnvTokens).unwrap();
    assert_eq!(file.encoding(), Encoding::Text);
    assert_eq!(save.meta.player, "GER".parse()?);
    Ok(())
}

#[test]
fn fix_crash_on_long_country_tag_debug_mode() {
    let data = include_bytes!("fixtures/crash1.bin");
    let file = Eu4File::from_slice(data).unwrap();
    let _save = file.deserializer().build_save(&EnvTokens);
}
