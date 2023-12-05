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
                let abc = seq.next_element::<de::IgnoredAny>()?;
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

                while let Some(key) = map.next_key::<Phf>()? {
                    match key {
                        Phf::Owner => owner = map.next_value()?,
                        Phf::BaseTax => base_tax = map.next_value()?,
                        Phf::BaseProduction => base_production = map.next_value()?,
                        Phf::BaseManpower => base_manpower = map.next_value()?,
                        Phf::Religion => religion = map.next_value()?,
                        Phf::Date(date) => map.next_value_seed(ExtendVec {
                            date,
                            estimate,
                            events: &mut events,
                        })?,
                        Phf::Other(key) => {
                            if let x @ ProvinceEventValue::Bool(_) = map.next_value()? {
                                other.insert(key.to_string(), x);
                            }
                        }
                        _ => {
                            map.next_value::<de::IgnoredAny>()?;
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

enum Phf {
    AddCore,
    BaseManpower,
    BaseProduction,
    BaseTax,
    Capital,
    Culture,
    Date(Eu4Date),
    DiscoveredBy,
    Hre,
    Other(String),
    Owner,
    Religion,
    TradeGoods,
}

impl<'de> de::Deserialize<'de> for Phf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;
        impl de::Visitor<'_> for Visitor {
            type Value = Phf;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum province history field")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v {
                    "owner" => Ok(Phf::Owner),
                    "base_tax" => Ok(Phf::BaseTax),
                    "base_manpower" => Ok(Phf::BaseManpower),
                    "base_production" => Ok(Phf::BaseProduction),
                    "religion" => Ok(Phf::Religion),
                    "trade_goods" => Ok(Phf::TradeGoods),
                    "discovered_by" => Ok(Phf::DiscoveredBy),
                    "culture" => Ok(Phf::Culture),
                    "capital" => Ok(Phf::Capital),
                    "add_core" => Ok(Phf::AddCore),
                    "hre" => Ok(Phf::Hre),
                    x => {
                        if let Ok(date) = Eu4Date::parse(x) {
                            Ok(Phf::Date(date))
                        } else {
                            Ok(Phf::Other(String::from(x)))
                        }
                    }
                }
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

enum Pef {
    AddClaim,
    AddCore,
    Advisor,
    BaseManpower,
    BaseProduction,
    BaseTax,
    Controller,
    Culture,
    DiscoveredBy,
    IsCity,
    Other(String),
    Owner,
    Religion,
    RemoveClaim,
    RemoveCore,
    TradeGoods,
}

impl<'de> de::Deserialize<'de> for Pef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;
        impl de::Visitor<'_> for Visitor {
            type Value = Pef;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum province event field")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v {
                    "owner" => Ok(Pef::Owner),
                    "controller" => Ok(Pef::Controller),
                    "base_tax" => Ok(Pef::BaseTax),
                    "base_manpower" => Ok(Pef::BaseManpower),
                    "base_production" => Ok(Pef::BaseProduction),
                    "religion" => Ok(Pef::Religion),
                    "add_claim" => Ok(Pef::AddClaim),
                    "remove_claim" => Ok(Pef::RemoveClaim),
                    "add_core" => Ok(Pef::AddCore),
                    "remove_core" => Ok(Pef::RemoveCore),
                    "advisor" => Ok(Pef::Advisor),
                    "trade_goods" => Ok(Pef::TradeGoods),
                    "discovered_by" => Ok(Pef::DiscoveredBy),
                    "culture" => Ok(Pef::Culture),
                    "is_city" => Ok(Pef::IsCity),
                    x => Ok(Pef::Other(String::from(x))),
                }
            }
        }

        deserializer.deserialize_str(Visitor)
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
                let abc = seq.next_element::<de::IgnoredAny>()?;
                if abc.is_some() {
                    return Err(de::Error::custom("unexpected sequence!"));
                }

                Ok(())
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                while let Some(key) = map.next_key::<Pef>()? {
                    let val = match key {
                        Pef::Owner => ProvinceEvent::Owner(map.next_value()?),
                        Pef::Controller => ProvinceEvent::Controller(map.next_value()?),
                        Pef::BaseTax => ProvinceEvent::BaseTax(map.next_value()?),
                        Pef::BaseManpower => ProvinceEvent::BaseManpower(map.next_value()?),
                        Pef::BaseProduction => ProvinceEvent::BaseProduction(map.next_value()?),
                        Pef::Religion => ProvinceEvent::Religion(map.next_value()?),
                        Pef::Other(key) => {
                            if let x @ ProvinceEventValue::Bool(_) = map.next_value()? {
                                ProvinceEvent::KV((key, x))
                            } else {
                                continue;
                            }
                        }
                        _ => {
                            map.next_value::<de::IgnoredAny>()?;
                            continue;
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
