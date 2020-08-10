use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Hash, Eq, PartialEq, Default, Deserialize)]
#[serde(from = "i32")]
pub struct ProvinceId(i32);

impl ProvinceId {
    pub fn new(x: i32) -> Self {
        ProvinceId(x.abs())
    }
}

impl From<i32> for ProvinceId {
    fn from(x: i32) -> Self {
        ProvinceId::new(x)
    }
}

impl fmt::Display for ProvinceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
