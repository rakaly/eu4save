mod country_events;
mod country_history;
mod gameplay_settings;
mod leader_kind;
mod token_bool;
mod vec_pair;
mod war_events;
mod war_history;

pub(crate) use token_bool::*;
pub(crate) use vec_pair::*;

use serde::{Deserialize, Deserializer};

pub(crate) fn empty_string_is_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s))
    }
}
