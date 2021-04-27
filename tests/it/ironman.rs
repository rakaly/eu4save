#![cfg(ironman)]

use crate::utils;
use eu4save::models::{GameDifficulty, ProvinceEvent, ProvinceEventValue, TaxManpowerModifier};
use eu4save::{
    query::{LedgerPoint, NationEvent, NationEventKind, NationEvents, PlayerHistory, Query},
    CountryTag, Encoding, Eu4Date, Eu4Extractor, Eu4ExtractorBuilder, FailedResolveStrategy,
    ProvinceId,
};
use paste::paste;
use std::io::Cursor;

#[test]
fn test_eu4_bin() {
    let data = utils::request("ragusa2.bin.eu4");
    let (save, encoding) = Eu4Extractor::extract_save(Cursor::new(&data[..])).unwrap();
    assert_eq!(encoding, Encoding::BinZip);
    assert_eq!(save.meta.player, "CRO".parse().unwrap());

    let (save2, _) = Eu4Extractor::extract_meta_optimistic(Cursor::new(&data[..])).unwrap();
    assert!(save2.game.is_none());

    let query = Query::from_save(save);
    let province_owners = query.province_owners();
    let nation_events = query.nation_events(&province_owners);
    let histories = query.player_histories(&nation_events);
    assert_eq!(
        query.starting_country(&histories),
        Some("RAG".parse().unwrap())
    );
    assert_eq!(query.players(), vec![]);

    let expected_histories = vec![PlayerHistory {
        history: NationEvents {
            initial: "RAG".parse().unwrap(),
            latest: "CRO".parse().unwrap(),
            stored: "CRO".parse().unwrap(),
            events: vec![NationEvent {
                date: Eu4Date::new(1769, 1, 2).unwrap(),
                kind: NationEventKind::TagSwitch("CRO".parse().unwrap()),
            }],
        },
        is_human: true,
        player_names: Vec::new(),
    }];
    assert_eq!(expected_histories, histories);
}

#[test]
fn test_eu4_kandy_bin() {
    let data = utils::request("kandy2.bin.eu4");
    let (save, encoding) = Eu4Extractor::extract_save(Cursor::new(&data[..])).unwrap();
    assert_eq!(encoding, Encoding::BinZip);
    assert_eq!(save.meta.player, "BHA".parse().unwrap());

    let query = Query::from_save(save);
    let province_owners = query.province_owners();
    let nation_events = query.nation_events(&province_owners);
    let histories = query.player_histories(&nation_events);

    let player = query
        .save()
        .game
        .countries
        .get(&"BHA".parse().unwrap())
        .unwrap();
    assert!(!player.completed_missions.is_empty());

    assert_eq!(
        query.country_tag_hex_color(&"BHA".parse().unwrap()),
        Some(String::from("#50a50a"))
    );

    assert_eq!(
        query
            .save()
            .game
            .provinces
            .get(&ProvinceId::from(1))
            .unwrap()
            .owner
            .as_ref()
            .unwrap(),
        &"SCA".parse().unwrap()
    );

    assert_eq!(
        query.starting_country(&histories),
        Some("KND".parse().unwrap())
    );
    assert_eq!(
        query
            .players()
            .iter()
            .map(|x| x.name.as_str())
            .collect::<Vec<_>>(),
        vec!["comagoosie"]
    );

    let subjects: Vec<CountryTag> = vec![
        "TEO".parse().unwrap(),
        "YOK".parse().unwrap(),
        "C21".parse().unwrap(),
        "C23".parse().unwrap(),
    ];

    assert_eq!(
        query
            .save()
            .game
            .countries
            .get(&"BHA".parse().unwrap())
            .unwrap()
            .subjects,
        subjects
    );

    // Testing binary encoded saves can extract province building history perfectly fine
    let london = query
        .save()
        .game
        .provinces
        .get(&ProvinceId::from(236))
        .unwrap();

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
    assert_eq!(date.game_fmt().as_str(), "1486.6.3");

    let building_date = query
        .province_building_history(london)
        .iter()
        .find(|x| x.building == "marketplace")
        .map(|x| x.date.game_fmt());
    assert_eq!(building_date, Some(String::from("1486.6.3")));

    let building_date = query
        .province_building_history(london)
        .iter()
        .find(|x| x.building == "fort_15th")
        .map(|x| x.date);
    assert_eq!(building_date, Some(query.save().game.start_date));

    assert_eq!(player.active_idea_groups.len(), 9);
    assert_eq!(
        player.active_idea_groups[8],
        (String::from("economic_ideas"), 2)
    );
}

#[test]
fn test_eu4_same_campaign_id() {
    let data = utils::request("ita2.eu4");
    let data2 = utils::request("ita2_later.eu4");
    let (save, _) = Eu4Extractor::extract_save(Cursor::new(&data[..])).unwrap();
    let (save2, _) = Eu4Extractor::extract_save(Cursor::new(&data2[..])).unwrap();
    assert_eq!(save.meta.campaign_id, save2.meta.campaign_id);
    assert!(save.meta.date < save2.meta.date);
}

#[test]
fn test_eu4_ita1() {
    let data = utils::request("ita1.eu4");
    let (save, encoding) = Eu4Extractor::extract_save(Cursor::new(&data[..])).unwrap();
    assert_eq!(encoding, Encoding::BinZip);
    assert_eq!(save.meta.player, "ITA".parse().unwrap());
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
        .map(|x| eu4save::dlc_id(x.as_str()))
        .all(|x| x.is_some());
    assert!(all_dlc_recognized);

    let query = Query::from_save(save);
    let province_owners = query.province_owners();
    let nation_events = query.nation_events(&province_owners);
    let histories = query.player_histories(&nation_events);
    assert_eq!(
        query.starting_country(&histories),
        Some("LAN".parse().unwrap())
    );
    assert_eq!(
        query
            .players()
            .iter()
            .map(|x| x.name.as_str())
            .collect::<Vec<_>>(),
        vec!["comagoosie"]
    );
}

#[test]
fn test_roundtrip_melt() {
    let data = utils::request("kandy2.bin.eu4");
    let (out, _unknown) = eu4save::melt(&data[..], eu4save::FailedResolveStrategy::Error).unwrap();
    let (save, encoding) = Eu4Extractor::extract_save(Cursor::new(&out[..])).unwrap();
    assert_eq!(encoding, Encoding::Text);
    assert_eq!(save.meta.player, "BHA".parse().unwrap());
}

macro_rules! ironman_test {
    ($name:ident, $fp:expr, $query:expr, $further:expr) => {
        paste! {
            #[test]
            fn [<test_ $name>]() {
                let data = utils::request($fp);

                // Ensure that every ironman can be melted with all tokens resolvable.
                // Deserialization will not try and resolve tokens that aren't used. Melting
                // ensures that every token is seen
                let (melted, _) = eu4save::melt(&data[..], FailedResolveStrategy::Error).unwrap();
                assert!(!melted.is_empty());

                let (save, encoding) = Eu4ExtractorBuilder::new()
                    .with_on_failed_resolve(FailedResolveStrategy::Error)
                    .extract_save(Cursor::new(&data[..]))
                    .unwrap();
                assert_eq!(encoding, Encoding::BinZip);
                let expected = $query;
                assert_eq!(save.meta.player, expected.player.parse::<CountryTag>().unwrap());
                assert_eq!(save.meta.date, expected.date);

                let version = &save.meta.savegame_version;
                let patch = format!(
                    "{}.{}.{}.{}",
                    version.first,
                    version.second,
                    version.third,
                    version.fourth
                );
                assert_eq!(patch.as_str(), expected.patch);

                let query = Query::from_save(save);
                let province_owners = query.province_owners();
                let nation_events = query.nation_events(&province_owners);
                let histories = query.player_histories(&nation_events);

                assert_eq!(
                    query.starting_country(&histories).unwrap(),
                    expected.starting.parse::<CountryTag>().unwrap(),
                );

                ($further)(query, melted.as_slice());
            }
        }
    };

    ($name:ident, $fp:expr, $query:expr) => {
        paste! {
            fn [<test_ $name _cb>](_q: Query, _data: &[u8]) {
            }

            ironman_test!($name, $fp, $query,  [<test_ $name _cb>]);
        }
    };
}

struct IronmanQuery {
    starting: &'static str,
    player: &'static str,
    patch: &'static str,
    date: Eu4Date,
}

ironman_test!(
    kandy2,
    "kandy2.bin.eu4",
    IronmanQuery {
        starting: "KND",
        player: "BHA",
        patch: "1.29.5.0",
        date: Eu4Date::parse_from_str("1804.12.09").unwrap()
    }
);

ironman_test!(
    tryone,
    "tryone.eu4",
    IronmanQuery {
        starting: "TYR",
        player: "TYR",
        patch: "1.30.3.0",
        date: Eu4Date::parse_from_str("1581.03.01").unwrap()
    }
);

ironman_test!(
    modded,
    "modded.eu4",
    IronmanQuery {
        starting: "MOS",
        player: "MOS",
        patch: "1.30.4.0",
        date: Eu4Date::parse_from_str("1446.03.16").unwrap()
    }
);

fn trycone_expected_histories() -> Vec<PlayerHistory> {
    vec![PlayerHistory {
        history: NationEvents {
            initial: "TYR".parse().unwrap(),
            latest: "GBR".parse().unwrap(),
            stored: "GBR".parse().unwrap(),
            events: vec![
                NationEvent {
                    date: Eu4Date::new(1518, 1, 29).unwrap(),
                    kind: NationEventKind::TagSwitch("IRE".parse().unwrap()),
                },
                NationEvent {
                    date: Eu4Date::new(1606, 8, 4).unwrap(),
                    kind: NationEventKind::TagSwitch("GBR".parse().unwrap()),
                },
            ],
        },
        is_human: true,
        player_names: vec![String::from("comagoosie")],
    }]
}

fn leinster_history() -> NationEvents {
    NationEvents {
        initial: "LEI".parse().unwrap(),
        latest: "LEI".parse().unwrap(),
        stored: "LEI".parse().unwrap(),
        events: vec![
            NationEvent {
                date: Eu4Date::new(1451, 8, 2).unwrap(),
                kind: NationEventKind::Annexed,
            },
            NationEvent {
                date: Eu4Date::new(1588, 6, 15).unwrap(),
                kind: NationEventKind::Appeared,
            },
            NationEvent {
                date: Eu4Date::new(1605, 2, 15).unwrap(),
                kind: NationEventKind::Annexed,
            },
            NationEvent {
                date: Eu4Date::new(1716, 2, 9).unwrap(),
                kind: NationEventKind::Appeared,
            },
        ],
    }
}

ironman_test!(
    trycone,
    "trycone.eu4",
    IronmanQuery {
        starting: "TYR",
        player: "GBR",
        patch: "1.30.4.0",
        date: Eu4Date::parse_from_str("1725.05.12").unwrap()
    },
    |query: Query, _melted_data: &[u8]| {
        let province_owners = query.province_owners();
        let nation_events = query.nation_events(&province_owners);
        let lei = "LEI".parse().unwrap();
        let lei_events = nation_events.iter().find(|x| x.initial == lei).unwrap();
        assert_eq!(lei_events, &leinster_history());
        let histories = query.player_histories(&nation_events);
        assert_eq!(&histories, &trycone_expected_histories());
        let tag_resolver = query.tag_resolver(&nation_events);
        assert_eq!(
            tag_resolver.resolve("IRE".parse().unwrap(), Eu4Date::new(1529, 3, 1).unwrap()),
            "GBR".parse().unwrap()
        );

        let lei_income = query.income_statistics_ledger(&lei_events);
        assert_eq!(
            lei_income,
            vec![
                LedgerPoint {
                    tag: lei,
                    year: 1445,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1446,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1447,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1448,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1449,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1450,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1451,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1589,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1590,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1591,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1592,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1593,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1594,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1595,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1596,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1597,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1598,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1599,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1600,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1601,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1602,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1603,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1604,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1605,
                    value: 1
                },
                LedgerPoint {
                    tag: lei,
                    year: 1717,
                    value: 3
                },
                LedgerPoint {
                    tag: lei,
                    year: 1718,
                    value: 3
                },
                LedgerPoint {
                    tag: lei,
                    year: 1719,
                    value: 3
                },
                LedgerPoint {
                    tag: lei,
                    year: 1720,
                    value: 3
                },
                LedgerPoint {
                    tag: lei,
                    year: 1721,
                    value: 3
                },
                LedgerPoint {
                    tag: lei,
                    year: 1722,
                    value: 3
                },
                LedgerPoint {
                    tag: lei,
                    year: 1723,
                    value: 3
                },
                LedgerPoint {
                    tag: lei,
                    year: 1724,
                    value: 3
                },
                LedgerPoint {
                    tag: lei,
                    year: 1725,
                    value: 3
                },
            ]
        );

        let income = query.income_statistics_ledger(&histories[0].history);
        assert_eq!(
            income[0],
            LedgerPoint {
                tag: "TYR".parse().unwrap(),
                year: 1445,
                value: 1
            }
        );
        assert!(income.contains(&LedgerPoint {
            tag: "TYR".parse().unwrap(),
            year: 1518,
            value: 8
        }));
        assert!(income.contains(&LedgerPoint {
            tag: "IRE".parse().unwrap(),
            year: 1519,
            value: 9
        }));
        assert!(income.contains(&LedgerPoint {
            tag: "IRE".parse().unwrap(),
            year: 1606,
            value: 70
        }));
        assert!(income.contains(&LedgerPoint {
            tag: "GBR".parse().unwrap(),
            year: 1607,
            value: 69
        }));
        assert_eq!(
            income.last().unwrap(),
            &LedgerPoint {
                tag: "GBR".parse().unwrap(),
                year: 1725,
                value: 717
            }
        );
    }
);

fn true_heir_expected_histories() -> Vec<PlayerHistory> {
    vec![PlayerHistory {
        history: NationEvents {
            initial: "SIS".parse().unwrap(),
            latest: "MUG".parse().unwrap(),
            stored: "MUG".parse().unwrap(),
            events: vec![
                NationEvent {
                    date: Eu4Date::new(1467, 12, 3).unwrap(),
                    kind: NationEventKind::TagSwitch("DLH".parse().unwrap()),
                },
                NationEvent {
                    date: Eu4Date::new(1467, 12, 3).unwrap(),
                    kind: NationEventKind::TagSwitch("MUG".parse().unwrap()),
                },
            ],
        },
        is_human: true,
        player_names: vec![String::from("lambdax.x")],
    }]
}

ironman_test!(
    true_heir_of_timur,
    "sis.eu4",
    IronmanQuery {
        starting: "SIS",
        player: "MUG",
        patch: "1.30.3.0",
        date: Eu4Date::parse_from_str("1508.04.27").unwrap()
    },
    |query: Query, _melted_data: &[u8]| {
        let province_owners = query.province_owners();
        let nation_events = query.nation_events(&province_owners);
        let histories = query.player_histories(&nation_events);
        assert_eq!(histories, true_heir_expected_histories());
    }
);

ironman_test!(
    revolution_center_cologne,
    "cologne2.eu4",
    IronmanQuery {
        starting: "KOL",
        player: "KOL",
        patch: "1.30.3.0",
        date: Eu4Date::parse_from_str("1821.01.03").unwrap()
    }
);

ironman_test!(
    ita2,
    "ita2_later13.eu4",
    IronmanQuery {
        starting: "MLO",
        player: "ITA",
        patch: "1.29.6.0",
        date: Eu4Date::parse_from_str("1547.03.05").unwrap()
    },
    |query: Query, _melted_data: &[u8]| {
        assert_eq!(
            query
                .players()
                .iter()
                .map(|x| x.name.as_str())
                .collect::<Vec<_>>(),
            vec!["comagoosie"]
        );
    }
);

ironman_test!(
    burg,
    "burg.eu4",
    IronmanQuery {
        starting: "BUR",
        player: "BUR",
        patch: "1.30.4.0",
        date: Eu4Date::parse_from_str("1821.01.03").unwrap()
    }
);

ironman_test!(
    no_dlc,
    "no-dlc.eu4",
    IronmanQuery {
        starting: "BYZ",
        player: "BYZ",
        patch: "1.30.4.0",
        date: Eu4Date::parse_from_str("1513.05.25").unwrap()
    }
);

ironman_test!(
    chinese_supplementary,
    "chinese-supplementary.eu4",
    IronmanQuery {
        starting: "SZO",
        player: "SZO",
        patch: "1.30.4.0",
        date: Eu4Date::parse_from_str("1800.01.01").unwrap()
    },
    |query: Query, _melted_data: &[u8]| {
        assert_eq!(
            query.save().meta.displayed_country_name.as_bytes(),
            b"\x10(\xe2\x80\x9e\x10bS\x10PO"
        );
    }
);

fn cilli_history() -> NationEvents {
    NationEvents {
        initial: "CLI".parse().unwrap(),
        latest: "CRO".parse().unwrap(),
        stored: "CRO".parse().unwrap(),
        events: vec![NationEvent {
            date: Eu4Date::new(1509, 6, 6).unwrap(),
            kind: NationEventKind::TagSwitch("CRO".parse().unwrap()),
        }],
    }
}

ironman_test!(
    dont_be,
    "dont_be.eu4",
    IronmanQuery {
        starting: "CLI",
        player: "CRO",
        patch: "1.30.1.0",
        date: Eu4Date::parse_from_str("1509.06.10").unwrap()
    },
    |query: Query, _melted_data: &[u8]| {
        let province_owners = query.province_owners();
        let nation_events = query.nation_events(&province_owners);
        let histories = query.player_histories(&nation_events);
        assert_eq!(&histories[0].history, &cilli_history());
        let cli = "CLI".parse().unwrap();

        let my_score = query.score_statistics_ledger(&histories[0].history);
        let mut expected_score = Vec::new();
        for i in 1445..1510 {
            expected_score.push(LedgerPoint {
                tag: cli,
                year: i,
                value: 0,
            });
        }
        assert_eq!(my_score, expected_score);

        let my_inflation = query.inflation_statistics_ledger(&histories[0].history);
        let mut expected_inflation = Vec::new();
        for i in 1445..1479 {
            expected_inflation.push(LedgerPoint {
                tag: cli,
                year: i,
                value: 0,
            });
        }

        #[rustfmt::skip]
        expected_inflation.extend_from_slice(&[
            LedgerPoint { tag: cli, year: 1479, value: 1 },
            LedgerPoint { tag: cli, year: 1480, value: 1 },
            LedgerPoint { tag: cli, year: 1481, value: 1 },
            LedgerPoint { tag: cli, year: 1482, value: 1 },
            LedgerPoint { tag: cli, year: 1483, value: 1 },
            LedgerPoint { tag: cli, year: 1484, value: 1 },
            LedgerPoint { tag: cli, year: 1485, value: 1 },
            LedgerPoint { tag: cli, year: 1486, value: 1 },
            LedgerPoint { tag: cli, year: 1487, value: 1 },
            LedgerPoint { tag: cli, year: 1488, value: 1 },
            LedgerPoint { tag: cli, year: 1489, value: 2 },
            LedgerPoint { tag: cli, year: 1490, value: 2 },
            LedgerPoint { tag: cli, year: 1491, value: 2 },
            LedgerPoint { tag: cli, year: 1492, value: 2 },
            LedgerPoint { tag: cli, year: 1493, value: 2 },
            LedgerPoint { tag: cli, year: 1494, value: 2 },
            LedgerPoint { tag: cli, year: 1495, value: 2 },
            LedgerPoint { tag: cli, year: 1496, value: 2 },
            LedgerPoint { tag: cli, year: 1497, value: 3 },
            LedgerPoint { tag: cli, year: 1498, value: 3 },
            LedgerPoint { tag: cli, year: 1499, value: 3 },
            LedgerPoint { tag: cli, year: 1500, value: 3 },
            LedgerPoint { tag: cli, year: 1501, value: 3 },
            LedgerPoint { tag: cli, year: 1502, value: 3 },
            LedgerPoint { tag: cli, year: 1503, value: 3 },
            LedgerPoint { tag: cli, year: 1504, value: 3 },
            LedgerPoint { tag: cli, year: 1505, value: 3 },
            LedgerPoint { tag: cli, year: 1506, value: 3 },
            LedgerPoint { tag: cli, year: 1507, value: 3 },
            LedgerPoint { tag: cli, year: 1508, value: 3 },
            LedgerPoint { tag: cli, year: 1509, value: 3 },
        ]);
        assert_eq!(my_inflation, expected_inflation);

        let my_income = query.income_statistics_ledger(&histories[0].history);

        #[rustfmt::skip]
        let expected_income = vec![
            LedgerPoint { tag: cli, year: 1445, value: 1 },
            LedgerPoint { tag: cli, year: 1446, value: 1 },
            LedgerPoint { tag: cli, year: 1447, value: 1 },
            LedgerPoint { tag: cli, year: 1448, value: 1 },
            LedgerPoint { tag: cli, year: 1449, value: 1 },
            LedgerPoint { tag: cli, year: 1450, value: 2 },
            LedgerPoint { tag: cli, year: 1451, value: 2 },
            LedgerPoint { tag: cli, year: 1452, value: 3 },
            LedgerPoint { tag: cli, year: 1453, value: 3 },
            LedgerPoint { tag: cli, year: 1454, value: 3 },
            LedgerPoint { tag: cli, year: 1455, value: 3 },
            LedgerPoint { tag: cli, year: 1456, value: 5 },
            LedgerPoint { tag: cli, year: 1457, value: 5 },
            LedgerPoint { tag: cli, year: 1458, value: 5 },
            LedgerPoint { tag: cli, year: 1459, value: 5 },
            LedgerPoint { tag: cli, year: 1460, value: 4 },
            LedgerPoint { tag: cli, year: 1461, value: 5 },
            LedgerPoint { tag: cli, year: 1462, value: 5 },
            LedgerPoint { tag: cli, year: 1463, value: 5 },
            LedgerPoint { tag: cli, year: 1464, value: 5 },
            LedgerPoint { tag: cli, year: 1465, value: 4 },
            LedgerPoint { tag: cli, year: 1466, value: 4 },
            LedgerPoint { tag: cli, year: 1467, value: 4 },
            LedgerPoint { tag: cli, year: 1468, value: 4 },
            LedgerPoint { tag: cli, year: 1469, value: 4 },
            LedgerPoint { tag: cli, year: 1470, value: 4 },
            LedgerPoint { tag: cli, year: 1471, value: 4 },
            LedgerPoint { tag: cli, year: 1472, value: 4 },
            LedgerPoint { tag: cli, year: 1473, value: 4 },
            LedgerPoint { tag: cli, year: 1474, value: 4 },
            LedgerPoint { tag: cli, year: 1475, value: 4 },
            LedgerPoint { tag: cli, year: 1476, value: 4 },
            LedgerPoint { tag: cli, year: 1477, value: 4 },
            LedgerPoint { tag: cli, year: 1478, value: 4 },
            LedgerPoint { tag: cli, year: 1479, value: 4 },
            LedgerPoint { tag: cli, year: 1480, value: 5 },
            LedgerPoint { tag: cli, year: 1481, value: 5 },
            LedgerPoint { tag: cli, year: 1482, value: 5 },
            LedgerPoint { tag: cli, year: 1483, value: 5 },
            LedgerPoint { tag: cli, year: 1484, value: 5 },
            LedgerPoint { tag: cli, year: 1485, value: 5 },
            LedgerPoint { tag: cli, year: 1486, value: 5 },
            LedgerPoint { tag: cli, year: 1487, value: 5 },
            LedgerPoint { tag: cli, year: 1488, value: 5 },
            LedgerPoint { tag: cli, year: 1489, value: 5 },
            LedgerPoint { tag: cli, year: 1490, value: 5 },
            LedgerPoint { tag: cli, year: 1491, value: 5 },
            LedgerPoint { tag: cli, year: 1492, value: 5 },
            LedgerPoint { tag: cli, year: 1493, value: 5 },
            LedgerPoint { tag: cli, year: 1494, value: 5 },
            LedgerPoint { tag: cli, year: 1495, value: 6 },
            LedgerPoint { tag: cli, year: 1496, value: 6 },
            LedgerPoint { tag: cli, year: 1497, value: 5 },
            LedgerPoint { tag: cli, year: 1498, value: 6 },
            LedgerPoint { tag: cli, year: 1499, value: 5 },
            LedgerPoint { tag: cli, year: 1500, value: 5 },
            LedgerPoint { tag: cli, year: 1501, value: 5 },
            LedgerPoint { tag: cli, year: 1502, value: 6 },
            LedgerPoint { tag: cli, year: 1503, value: 6 },
            LedgerPoint { tag: cli, year: 1504, value: 6 },
            LedgerPoint { tag: cli, year: 1505, value: 6 },
            LedgerPoint { tag: cli, year: 1506, value: 6 },
            LedgerPoint { tag: cli, year: 1507, value: 6 },
            LedgerPoint { tag: cli, year: 1508, value: 6 },
            LedgerPoint { tag: cli, year: 1509, value: 6 },
        ];

        assert_eq!(my_income, expected_income);
    }
);

ironman_test!(
    non_ironman_binary,
    "non-ironman-binary.eu4",
    IronmanQuery {
        starting: "CAS",
        player: "CAS",
        patch: "1.30.6.0",
        date: Eu4Date::parse_from_str("1505.11.25").unwrap()
    },
    |query: Query, _melted_data: &[u8]| {
        assert!(!query.save().meta.is_ironman);
    }
);

ironman_test!(
    patch_131,
    "1.31.0.eu4",
    IronmanQuery {
        starting: "ENG",
        player: "ENG",
        patch: "1.31.0.0",
        date: Eu4Date::parse_from_str("1444.11.11").unwrap()
    },
    |_query: Query, melted_data: &[u8]| {
        // Find inukshuk
        twoway::find_bytes(melted_data, b"date_built=-2000.1.1").unwrap();
    }
);
