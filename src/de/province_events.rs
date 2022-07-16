use crate::models::{ProvinceEvent, ProvinceEvents};
use serde::{de, Deserialize, Deserializer};
use std::fmt;

impl<'de> Deserialize<'de> for ProvinceEvents {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ProvinceEventsVisitor;

        impl<'de> de::Visitor<'de> for ProvinceEventsVisitor {
            type Value = ProvinceEvents;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ProvinceEvents")
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

                Ok(ProvinceEvents(vec![]))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut values = if let Some(size) = map.size_hint() {
                    Vec::with_capacity(size)
                } else {
                    Vec::new()
                };

                while let Some(key) = map.next_key::<&str>()? {
                    let val = match key {
                        "owner" => ProvinceEvent::Owner(map.next_value()?),
                        "controller" => ProvinceEvent::Controller(map.next_value()?),
                        "base_tax" => ProvinceEvent::BaseTax(map.next_value()?),
                        "base_manpower" => ProvinceEvent::BaseManpower(map.next_value()?),
                        "base_production" => ProvinceEvent::BaseProduction(map.next_value()?),
                        "religion" => ProvinceEvent::Religion(map.next_value()?),
                        _ => ProvinceEvent::KV((key.to_string(), map.next_value()?)),
                    };

                    values.push(val);
                }

                Ok(ProvinceEvents(values))
            }
        }

        deserializer.deserialize_map(ProvinceEventsVisitor)
    }
}
