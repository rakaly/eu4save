use crate::models::{ProvinceEvent, ProvinceEventValue};
use crate::{models::ProvinceHistory, Eu4Date};
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;

impl<'de> Deserialize<'de> for ProvinceHistory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ProvinceHistoryVisitor;

        impl<'de> de::Visitor<'de> for ProvinceHistoryVisitor {
            type Value = ProvinceHistory;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ProvinceHistory with arbitrary fields")
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

                Ok(ProvinceHistory::default())
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut owner = None;
                let mut base_tax = None;
                let mut base_production = None;
                let mut base_manpower = None;
                let mut religion = None;
                let mut events = Vec::new();
                let mut other = HashMap::new();
                let hint = map.size_hint().unwrap_or_default();
                let estimate = (hint * 3 / 4) + 8;

                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "owner" => owner = map.next_value()?,
                        "base_tax" => base_tax = map.next_value()?,
                        "base_production" => base_production = map.next_value()?,
                        "base_manpower" => base_manpower = map.next_value()?,
                        "religion" => religion = map.next_value()?,
                        x => {
                            if let Ok(date) = Eu4Date::parse(x) {
                                let seed = ExtendVec {
                                    date,
                                    estimate,
                                    events: &mut events,
                                };
                                map.next_value_seed(seed)?;
                            } else if let x @ ProvinceEventValue::Bool(_) = map.next_value()? {
                                other.insert(key.to_string(), x);
                            }
                        }
                    }
                }

                events.shrink_to_fit();
                Ok(ProvinceHistory {
                    owner,
                    base_tax,
                    base_production,
                    base_manpower,
                    religion,
                    events,
                    other,
                })
            }
        }

        deserializer.deserialize_map(ProvinceHistoryVisitor)
    }
}

// https://docs.rs/serde/latest/serde/de/trait.DeserializeSeed.html
struct ExtendVec<'a> {
    date: Eu4Date,
    estimate: usize,
    events: &'a mut Vec<(Eu4Date, ProvinceEvent)>,
}

impl<'de, 'a> de::DeserializeSeed<'de> for ExtendVec<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ExtendVecVisitor<'a> {
            date: Eu4Date,
            estimate: usize,
            events: &'a mut Vec<(Eu4Date, ProvinceEvent)>,
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
                        "owner" => ProvinceEvent::Owner(map.next_value()?),
                        "controller" => ProvinceEvent::Controller(map.next_value()?),
                        "base_tax" => ProvinceEvent::BaseTax(map.next_value()?),
                        "base_manpower" => ProvinceEvent::BaseManpower(map.next_value()?),
                        "base_production" => ProvinceEvent::BaseProduction(map.next_value()?),
                        "religion" => ProvinceEvent::Religion(map.next_value()?),
                        _ => {
                            if let x @ ProvinceEventValue::Bool(_) = map.next_value()? {
                                ProvinceEvent::KV((key.to_string(), x))
                            } else {
                                continue;
                            }
                        }
                    };

                    // Across a couple saves, the average number of events for
                    // provinces that have events is 32 though this tends to be
                    // dominated by outliers with the median being around 20
                    if self.events.is_empty() {
                        self.events.reserve(self.estimate);
                    }

                    self.events.push((self.date, val));
                }

                Ok(())
            }
        }

        deserializer.deserialize_map(ExtendVecVisitor {
            date: self.date,
            estimate: self.estimate,
            events: self.events,
        })
    }
}
