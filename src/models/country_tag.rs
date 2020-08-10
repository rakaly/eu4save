use serde::{de, Deserialize, Deserializer, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Hash, Eq, PartialEq)]
pub struct CountryTag(String);

impl CountryTag {
    pub fn new(x: String) -> Self {
        debug_assert!(
            x.len() == 3,
            "expected country tag {} to be 3 characters",
            x
        );
        CountryTag(x)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl<'a> From<&'a str> for CountryTag {
    fn from(x: &'a str) -> Self {
        CountryTag::from(String::from(x))
    }
}

impl From<String> for CountryTag {
    fn from(x: String) -> Self {
        CountryTag::new(x)
    }
}

impl Into<String> for CountryTag {
    fn into(self) -> String {
        self.0
    }
}

impl fmt::Display for CountryTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for CountryTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CountryTagVisitor;

        impl<'de> de::Visitor<'de> for CountryTagVisitor {
            type Value = CountryTag;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct CountryTag")
            }

            fn visit_str<A>(self, v: &str) -> Result<Self::Value, A>
            where
                A: de::Error,
            {
                if v.len() != 3 {
                    Err(de::Error::custom(
                        "a country tag should be a sequence of 3 letters",
                    ))
                } else {
                    Ok(CountryTag::from(v))
                }
            }
        }

        deserializer.deserialize_str(CountryTagVisitor)
    }
}
