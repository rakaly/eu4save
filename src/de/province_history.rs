use crate::{Eu4Date, ProvinceHistory};
use serde::{de, Deserialize, Deserializer};
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
                let abc = seq.next_element::<String>()?;
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
                let mut events = Vec::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "owner" => owner = map.next_value()?,
                        "base_tax" => base_tax = map.next_value()?,
                        "base_production" => base_production = map.next_value()?,
                        "base_manpower" => base_manpower = map.next_value()?,
                        x => {
                            if let Some(date) = Eu4Date::parse_from_str(x) {
                                let event = map.next_value()?;
                                events.push((date, event));
                            }
                        }
                    }
                }

                Ok(ProvinceHistory {
                    owner,
                    base_tax,
                    base_production,
                    base_manpower,
                    events,
                })
            }
        }

        deserializer.deserialize_map(ProvinceHistoryVisitor)
    }
}
