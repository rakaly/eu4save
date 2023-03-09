use crate::{
    models::{CountryEvent, CountryHistory},
    Eu4Date,
};
use serde::{de, Deserialize, Deserializer};
use std::fmt;

impl<'de> Deserialize<'de> for CountryHistory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CountryHistoryVisitor;

        impl<'de> de::Visitor<'de> for CountryHistoryVisitor {
            type Value = CountryHistory;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct CountryHistory with arbitrary fields")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                // Hmm empty object
                let abc = seq.next_element::<&str>()?;
                if abc.is_some() {
                    return Err(de::Error::custom("unexpected sequence!"));
                }

                Ok(CountryHistory::default())
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut government = None;
                let mut technology_group = None;
                let mut primary_culture = None;
                let mut religion = None;
                let mut add_government_reform = Vec::new();
                let mut events = Vec::new();

                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "government" => government = map.next_value()?,
                        "technology_group" => technology_group = map.next_value()?,
                        "primary_culture" => primary_culture = map.next_value()?,
                        "religion" => religion = map.next_value()?,
                        "add_government_reform" => add_government_reform.push(map.next_value()?),
                        x => {
                            if let Ok(date) = Eu4Date::parse(x) {
                                let seed = ExtendVec {
                                    date,
                                    events: &mut events,
                                };
                                map.next_value_seed(seed)?;
                            }
                        }
                    }
                }

                events.shrink_to_fit();
                Ok(CountryHistory {
                    government,
                    technology_group,
                    primary_culture,
                    religion,
                    add_government_reform,
                    events,
                })
            }
        }

        deserializer.deserialize_map(CountryHistoryVisitor)
    }
}

// https://docs.rs/serde/latest/serde/de/trait.DeserializeSeed.html
struct ExtendVec<'a> {
    date: Eu4Date,
    events: &'a mut Vec<(Eu4Date, CountryEvent)>,
}

impl<'de, 'a> de::DeserializeSeed<'de> for ExtendVec<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ExtendVecVisitor<'a> {
            date: Eu4Date,
            events: &'a mut Vec<(Eu4Date, CountryEvent)>,
        }

        impl<'de, 'a> de::Visitor<'de> for ExtendVecVisitor<'a> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "province events")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                // Hmm empty object
                let abc = seq.next_element::<&str>()?;
                if abc.is_some() {
                    return Err(de::Error::custom("unexpected sequence!"));
                }

                Ok(())
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                while let Some(key) = map.next_key::<&str>()? {
                    let val = match key {
                        "monarch" => CountryEvent::Monarch(map.next_value()?),
                        "monarch_heir" => CountryEvent::MonarchHeir(map.next_value()?),
                        "monarch_consort" => CountryEvent::MonarchConsort(map.next_value()?),
                        "heir" => CountryEvent::Heir(map.next_value()?),
                        "queen" => CountryEvent::Queen(map.next_value()?),
                        "union" => CountryEvent::Union(map.next_value()?),
                        "capital" => CountryEvent::Capital(map.next_value()?),
                        "leader" => CountryEvent::Leader(map.next_value()?),
                        "remove_accepted_culture" => {
                            CountryEvent::RemoveAcceptedCulture(map.next_value()?)
                        }
                        "changed_country_name_from" => {
                            CountryEvent::ChangedCountryNameFrom(map.next_value()?)
                        }
                        "changed_country_adjective_from" => {
                            CountryEvent::ChangedCountryAdjectiveFrom(map.next_value()?)
                        }
                        "changed_country_mapcolor_from" => {
                            CountryEvent::ChangedCountryMapColorFrom(map.next_value()?)
                        }
                        "changed_tag_from" => CountryEvent::ChangedTagFrom(map.next_value()?),
                        "religion" => CountryEvent::Religion(map.next_value()?),
                        _ => continue, /*panic!("unknown: {}", &key)*/
                    };

                    // Most countries tend to have 32 around events
                    if self.events.is_empty() {
                        self.events.reserve(64);
                    }

                    self.events.push((self.date, val));
                }

                Ok(())
            }
        }

        deserializer.deserialize_map(ExtendVecVisitor {
            date: self.date,
            events: self.events,
        })
    }
}
