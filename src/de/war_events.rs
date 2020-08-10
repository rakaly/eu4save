use crate::{WarEvent, WarEvents};
use serde::{de, Deserialize, Deserializer};
use std::fmt;

impl<'de> Deserialize<'de> for WarEvents {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WarEventsVisitor;

        impl<'de> de::Visitor<'de> for WarEventsVisitor {
            type Value = WarEvents;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct WarEvents")
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

                while let Some(key) = map.next_key::<String>()? {
                    let val = match key.as_str() {
                        "add_attacker" => WarEvent::AddAttacker(map.next_value()?),
                        "add_defender" => WarEvent::AddDefender(map.next_value()?),
                        "rem_attacker" => WarEvent::RemoveAttacker(map.next_value()?),
                        "rem_defender" => WarEvent::RemoveDefender(map.next_value()?),
                        "battle" => WarEvent::Battle(map.next_value()?),
                        _ => {
                            return Err(de::Error::custom(format!("unknown battle key: {}", &key)))
                        }
                    };

                    values.push(val);
                }

                Ok(WarEvents(values))
            }
        }

        deserializer.deserialize_map(WarEventsVisitor)
    }
}
