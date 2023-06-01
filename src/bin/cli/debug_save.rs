use eu4save::{models, EnvTokens, Eu4File};
use std::{error::Error, time::Instant};

pub fn run(data: &[u8]) -> Result<(), Box<dyn Error>> {
    let file = Eu4File::from_slice(data)?;
    let mut sink = Vec::new();

    let mut entries = file.entries();
    while let Some(entry) = entries.next_entry() {
        if entry.name() == Some(eu4save::file::Eu4FileEntryName::Gamestate) {
            let start = Instant::now();

            let mut dd = entry.deserializer(&mut sink, &EnvTokens).unwrap();
            let _game: models::GameState = dd.deserialize().unwrap();
            // let _game: models::GameState = serde_path_to_error::deserialize(&mut dd).map_err(|e| e.to_string()).unwrap();
            // let parsed = entry.parse(&mut sink)?;
            let after_parse = Instant::now();
            println!("parse: {}ms", after_parse.duration_since(start).as_millis());

            // let _game: models::GameState = parsed.deserializer(&EnvTokens).deserialize()?;
            // // let meta: models::Meta = parsed.deserializer(&EnvTokens).deserialize()?;
            // // let save = models::Eu4Save { game, meta };
            // let after_de = Instant::now();
            // println!(
            //     "deserialize: {}ms",
            //     after_de.duration_since(after_parse).as_millis()
            // );
        }
    }

    // let query = eu4save::query::Query::from_save(save);
    // let owners = query.province_owners();
    // let nation_events = query.nation_events(&owners);
    // let player = query.player_histories(&nation_events);
    // let ledger = query.nation_size_statistics_ledger(&player[0].history);
    // let after_rest = Instant::now();
    // println!(
    //     "rest: {}ms",
    //     after_rest.duration_since(after_de).as_millis()
    // );
    // println!("{}", ledger.len());
    Ok(())
}
