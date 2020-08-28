use crate::{models::WarHistory, Eu4Date};
use serde::{de, Deserialize, Deserializer};
use std::fmt;

impl<'de> Deserialize<'de> for WarHistory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WarHistoryVisitor;

        impl<'de> de::Visitor<'de> for WarHistoryVisitor {
            type Value = WarHistory;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct WarHistory with arbitrary fields")
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

                Ok(WarHistory::default())
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut name = None;
                let mut war_goal = None;
                let mut succession = None;
                let mut events = Vec::new();

                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "name" => name = map.next_value()?,
                        "war_goal" => war_goal = map.next_value()?,
                        "succession" => succession = map.next_value()?,
                        x => {
                            let date = Eu4Date::parse_from_str(x)
                                .ok_or_else(|| de::Error::custom(format!("invalid date: {}", x)))?;
                            let event = map.next_value()?;
                            events.push((date, event));
                        }
                    }
                }

                Ok(WarHistory {
                    name,
                    war_goal,
                    succession,
                    events,
                })
            }
        }

        deserializer.deserialize_map(WarHistoryVisitor)
    }
}
