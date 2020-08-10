#![cfg(ironman)]

use eu4save::{query::Query, CountryTag, Encoding, Eu4ExtractorBuilder, FailedResolveStrategy, Eu4Date};
use paste::paste;
use std::io::Cursor;

mod utils;

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
