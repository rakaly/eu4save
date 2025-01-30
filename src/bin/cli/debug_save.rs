use eu4save::{BasicTokenResolver, Eu4File};
use std::{error::Error, time::Instant};

pub fn run(path: &str) -> Result<(), Box<dyn Error>> {
    let file = std::fs::File::open(path)?;
    let file = Eu4File::from_file(file)?;

    let start = Instant::now();
    let file_data = std::fs::read("assets/eu4.txt").unwrap_or_default();
    let resolver = BasicTokenResolver::from_text_lines(file_data.as_slice())?;
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
    println!("{}", ledger.len());
    Ok(())
}
