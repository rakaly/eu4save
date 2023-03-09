use crate::{
    models::{WarEvent, WarHistory},
    Eu4Date,
};
use serde::{
    de::{self, Error},
    Deserialize, Deserializer,
};
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
                let hint = map.size_hint().unwrap_or_default();
                let estimate = hint.max(8);
                let mut events = Vec::with_capacity(estimate);

                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "name" => name = map.next_value()?,
                        "war_goal" => war_goal = map.next_value()?,
                        "succession" => succession = map.next_value()?,
                        x => {
                            let date = Eu4Date::parse(x).map_err(A::Error::custom)?;
                            let seed = ExtendVec {
                                date,
                                events: &mut events,
                            };
                            map.next_value_seed(seed)?;
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

// https://docs.rs/serde/latest/serde/de/trait.DeserializeSeed.html
struct ExtendVec<'a> {
    date: Eu4Date,
    events: &'a mut Vec<(Eu4Date, WarEvent)>,
}

impl<'de, 'a> de::DeserializeSeed<'de> for ExtendVec<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ExtendVecVisitor<'a> {
            date: Eu4Date,
            events: &'a mut Vec<(Eu4Date, WarEvent)>,
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
                        "add_attacker" => WarEvent::AddAttacker(map.next_value()?),
                        "add_defender" => WarEvent::AddDefender(map.next_value()?),
                        "rem_attacker" => WarEvent::RemoveAttacker(map.next_value()?),
                        "rem_defender" => WarEvent::RemoveDefender(map.next_value()?),
                        "battle" => WarEvent::Battle(map.next_value()?),
                        _ => {
                            return Err(de::Error::custom(format!("unknown battle key: {}", &key)))
                        }
                    };

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
