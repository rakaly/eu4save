use eu4save::{Eu4File, SegmentedResolver};
use std::{error::Error, time::Instant};

pub fn run(path: &str) -> Result<(), Box<dyn Error>> {
    let file = std::fs::File::open(path)?;
    let file = Eu4File::from_file(file)?;

    let start = Instant::now();
    let file_data = std::fs::read("assets/eu4.txt").unwrap_or_default();
    let resolver_builder = SegmentedResolver::parse(file_data.as_slice())?;
    let resolver = resolver_builder.resolver();
    let save = file.parse_save(&resolver)?;
    let after_parse = Instant::now();
    println!("parse: {}ms", after_parse.duration_since(start).as_millis());

    let query = eu4save::query::Query::from_save(save);
    let owners = query.province_owners();
    let nation_events = query.nation_events(&owners);
    let player = query.player_histories(&nation_events);
    let ledger = query.nation_size_statistics_ledger(&player[0].history);
    let after_rest = Instant::now();
    println!(
        "rest: {}ms",
        after_rest.duration_since(after_parse).as_millis()
    );
    println!("{}", query.save().game.provinces.len());
    println!("{}", query.save().game.countries.len());
    println!("{}", ledger.len());

    // Calculate stats number of non-zero history events
    let mut history_events = query
        .save()
        .game
        .countries
        .iter()
        .map(|(_, country)| country.history.events.len())
        .filter(|&x| x > 0)
        .collect::<Vec<_>>();
    history_events.sort_unstable();
    let sum = history_events.iter().sum::<usize>();
    println!(
        "median: {}, avg: {}",
        history_events[history_events.len() / 2],
        sum / history_events.len()
    );
    Ok(())
}
