use crate::{models::CountryHistory, Eu4Date};
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
                let mut add_government_reform = Vec::new();
                let mut events = Vec::new();

                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "government" => government = map.next_value()?,
                        "technology_group" => technology_group = map.next_value()?,
                        "primary_culture" => primary_culture = map.next_value()?,
                        "add_government_reform" => add_government_reform.push(map.next_value()?),
                        x => {
                            if let Ok(date) = Eu4Date::parse(x) {
                                let event = map.next_value()?;
                                events.push((date, event));
                            }
                        }
                    }
                }

                Ok(CountryHistory {
                    government,
                    technology_group,
                    primary_culture,
                    add_government_reform,
                    events,
                })
            }
        }

        deserializer.deserialize_map(CountryHistoryVisitor)
    }
}
