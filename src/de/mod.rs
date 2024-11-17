mod alternating_key_values;
mod country_history;
mod gameplay_settings;
mod leader_kind;
mod list_overflow_byte;
mod map_pair;
mod positive_vec;
mod province_event_value;
mod province_history;
mod token_bool;
mod trade_node;
mod vec_pair;
mod war_history;
mod yes_map;

pub(crate) use alternating_key_values::*;
pub(crate) use list_overflow_byte::*;
pub(crate) use map_pair::*;
pub(crate) use positive_vec::*;
pub(crate) use token_bool::*;
pub use vec_pair::*;
pub(crate) use yes_map::*;

use crate::models::Eu4String;
use serde::{Deserialize, Deserializer};

pub(crate) fn empty_string_is_none<'de, D>(deserializer: D) -> Result<Option<Eu4String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Eu4String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s))
    }
}
