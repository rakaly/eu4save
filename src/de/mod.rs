mod alternating_key_values;
mod country_events;
mod country_history;
mod gameplay_settings;
mod leader_kind;
mod province_event_value;
mod province_events;
mod province_history;
mod token_bool;
mod vec_overflow_byte;
mod vec_pair;
mod war_events;
mod war_history;
mod yes_map;

pub(crate) use alternating_key_values::*;
pub(crate) use token_bool::*;
pub(crate) use vec_overflow_byte::*;
pub(crate) use vec_pair::*;
pub(crate) use yes_map::*;

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
