use eu4save::{models, EnvTokens, Eu4File};
use std::{error::Error, time::Instant};

pub fn run(data: &[u8]) -> Result<(), Box<dyn Error>> {
    let file = Eu4File::from_slice(data)?;
    let mut sink = Vec::new();

    let start = Instant::now();
    let parsed = file.parse(&mut sink)?;
    let after_parse = Instant::now();
    println!("parse: {}ms", after_parse.duration_since(start).as_millis());

    let game: models::GameState = parsed.deserializer().build(&EnvTokens)?;
    let meta: models::Meta = parsed.deserializer().build(&EnvTokens)?;
    let save = models::Eu4Save { game, meta };
    let after_de = Instant::now();
    println!(
        "deserialize: {}ms",
        after_de.duration_since(after_parse).as_millis()
    );

    let query = eu4save::query::Query::from_save(save);
    let owners = query.province_owners();
    let nation_events = query.nation_events(&owners);
    let player = query.player_histories(&nation_events);
    let ledger = query.nation_size_statistics_ledger(&player[0].history);
    let after_rest = Instant::now();
    println!(
        "rest: {}ms",
        after_rest.duration_since(after_de).as_millis()
    );
    println!("{}", ledger.len());
    Ok(())
}
