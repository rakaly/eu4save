use crate::models::{CountryEvent, CountryEvents};
use serde::{de, Deserialize, Deserializer};
use std::fmt;

impl<'de> Deserialize<'de> for CountryEvents {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CountryEventsVisitor;

        impl<'de> de::Visitor<'de> for CountryEventsVisitor {
            type Value = CountryEvents;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct CountryEvents")
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

                Ok(CountryEvents(vec![]))
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
                        "monarch" => CountryEvent::Monarch(map.next_value()?),
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
                        _ => continue, /*panic!("unknown: {}", &key)*/
                    };

                    values.push(val);
                }

                Ok(CountryEvents(values))
            }
        }

        deserializer.deserialize_map(CountryEventsVisitor)
    }
}
