use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Hash, Eq, PartialEq, Deserialize)]
#[serde(from = "String")]
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
