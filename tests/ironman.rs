#![cfg(ironman)]

use eu4save::{
    query::{CountryQuery, Query},
    CountryTag, Encoding, Eu4Date, Eu4Extractor, Eu4ExtractorBuilder, FailedResolveStrategy,
    GameDifficulty, ProvinceEvent, ProvinceEventValue, ProvinceId, TaxManpowerModifier,
};
use paste::paste;
use std::collections::HashSet;
use std::io::Cursor;

mod utils;

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

    // Testing binary encoded saves can extract province building history perfectly fine
    let london = query
        .save
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
    assert_eq!(date.eu4_fmt().as_str(), "1486.6.3");

    let building_date = query
        .province_building_history(london)
        .get("marketplace")
        .map(|x| x.eu4_fmt());
    assert_eq!(building_date, Some(String::from("1486.6.3")));

    let building_date = query
        .province_building_history(london)
        .get("fort_15th")
        .cloned();
    assert_eq!(building_date, Some(query.save.game.start_date));
}

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

#[test]
fn test_roundtrip_melt() {
    let data = utils::request("kandy2.bin.eu4");
    let out = eu4save::melt(&data[..], eu4save::FailedResolveStrategy::Error).unwrap();
    let extractor = Eu4Extractor::default();
    let (save, encoding) = extractor.extract_save(Cursor::new(&out[..])).unwrap();
    assert_eq!(encoding, Encoding::Text);
    assert_eq!(save.meta.player, CountryTag::from("BHA"));
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
                let melted = eu4save::melt(&data[..], FailedResolveStrategy::Error).unwrap();
                assert!(!melted.is_empty());

                let extractor = Eu4ExtractorBuilder::new()
                    .with_on_failed_resolve(FailedResolveStrategy::Error)
                    .build();
                let (save, encoding) = extractor.extract_save(Cursor::new(&data[..])).unwrap();
                assert_eq!(encoding, Encoding::BinZip);
                let expected = $query;
                assert_eq!(save.meta.player, CountryTag::from(expected.player));
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

                assert_eq!(
                    query.starting_country.as_ref().unwrap(),
                    &CountryTag::from(expected.starting)
                );

                ($further)(query);
            }
        }
    };

    ($name:ident, $fp:expr, $query:expr) => {
        paste! {
            fn [<test_ $name _cb>](_q: Query) {
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
    true_heir_of_timur,
    "sis.eu4",
    IronmanQuery {
        starting: "SIS",
        player: "MUG",
        patch: "1.30.3.0",
        date: Eu4Date::parse_from_str("1508.04.27").unwrap()
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
    |query: Query| {
        assert_eq!(
            query.players.iter().cloned().collect::<Vec<_>>(),
            vec![String::from("comagoosie")]
        );
    }
);
