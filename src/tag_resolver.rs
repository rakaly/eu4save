use crate::{
    query::{NationEventKind, NationEvents},
    CountryTag, Eu4Date,
};
use std::collections::HashMap;

/// Tracks tag progression throughput history
///
/// The tag resolver is important to answering questions like "where are they
/// now?" when looking at historic events like province changes or wars. When
/// countries tag switch the tag resolver is able to connect historic events
/// to country's current selves. For instance in a TYR -> IRE -> GBR game, the
/// provinces and wars gained while playing as TYR should still be aggregated
/// under the current GBR tag.
#[derive(Debug)]
pub struct TagResolver {
    initial_to_stored: HashMap<CountryTag, CountryTag>,
    switches: HashMap<CountryTag, Vec<(Eu4Date, CountryTag)>>,
}

impl TagResolver {
    pub fn create(nation_events: &[NationEvents]) -> Self {
        let mut initial_to_stored = HashMap::with_capacity(nation_events.len());
        for nation in nation_events.iter() {
            initial_to_stored.insert(nation.initial, nation.stored);
        }

        let mut switches: HashMap<_, Vec<_>> = HashMap::new();
        for nation in nation_events.iter().filter(|x| x.initial != x.stored) {
            for event in &nation.events {
                if let NationEventKind::TagSwitch(to) = event.kind {
                    switches
                        .entry(to)
                        .or_insert_with(Vec::new)
                        .push((event.date, nation.stored));
                }
            }
        }

        for entries in switches.values_mut() {
            entries.sort_by_key(|(date, _)| *date);
        }

        TagResolver {
            initial_to_stored,
            switches,
        }
    }

    /// Given a date and tag associated with the date, return the current tag where
    /// the argument is stored.
    pub fn resolve(&self, tag: CountryTag, date: Eu4Date) -> CountryTag {
        self.switches
            .get(&tag)
            .and_then(|dates| {
                dates
                    .iter()
                    .take_while(|(change_date, _)| *change_date < date)
                    .last()
            })
            .map(|(_, stored)| *stored)
            .or_else(|| self.initial_to_stored.get(&tag).cloned())
            .unwrap_or(tag)
    }
}
