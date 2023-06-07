use crate::{
    models::{CountryTrade, TradeNode},
    CountryTag,
};
use serde::{de, Deserialize, Deserializer};
use std::fmt;

impl<'de> Deserialize<'de> for TradeNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TradeNodeVisitor;

        impl<'de> de::Visitor<'de> for TradeNodeVisitor {
            type Value = TradeNode;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct TradeNode")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut countries: Vec<_> = Vec::new();
                let mut country_section = false;
                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "highest_power" => {
                            country_section = true;
                            map.next_value::<de::IgnoredAny>()?;
                        }

                        // We need to know once the fixed fields are done as
                        // there are fields like "max" which may be accidentally
                        // interpretted as a country tag (country tag's can be
                        // lowercase).
                        x if country_section => {
                            if let Ok(tag) = CountryTag::create(x.as_bytes()) {
                                map.next_value_seed(ExtendVec {
                                    tag,
                                    countries: &mut countries,
                                })?;
                            } else {
                                map.next_value::<de::IgnoredAny>()?;
                            }
                        }
                        _ => {
                            map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(TradeNode { countries })
            }
        }

        deserializer.deserialize_map(TradeNodeVisitor)
    }
}

// https://docs.rs/serde/latest/serde/de/trait.DeserializeSeed.html
struct ExtendVec<'a> {
    tag: CountryTag,
    countries: &'a mut Vec<CountryTrade>,
}

impl<'de, 'a> de::DeserializeSeed<'de> for ExtendVec<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ExtendVecVisitor<'a> {
            tag: CountryTag,
            countries: &'a mut Vec<CountryTrade>,
        }

        impl<'de, 'a> de::Visitor<'de> for ExtendVecVisitor<'a> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "country trade node")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut privateer_money = 0.0;
                let mut should_extend = false;
                while let Some(key) = map.next_key::<&str>()? {
                    // Every country seems to have a max_demand field that seems
                    // useless so we only add a country to the array when it has
                    // an interesting field
                    match key {
                        "privateer_money" => privateer_money = map.next_value()?,
                        _ => {
                            map.next_value::<de::IgnoredAny>()?;
                            continue;
                        }
                    };

                    should_extend = true;
                }

                if should_extend {
                    self.countries.push(CountryTrade {
                        tag: self.tag,
                        privateer_money,
                    });
                }

                Ok(())
            }
        }

        deserializer.deserialize_map(ExtendVecVisitor {
            tag: self.tag,
            countries: self.countries,
        })
    }
}
