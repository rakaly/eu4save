use eu4save::{EnvTokens, Eu4File};
use std::error::Error;

pub fn run(file_path: &str) -> Result<(), Box<dyn Error>> {
    let data = std::fs::read(file_path)?;
    let file = Eu4File::from_slice(&data)?;
    let save = file.deserializer().build_save(&EnvTokens)?;

    let query = eu4save::query::Query::from_save(save);
    let owners = query.province_owners();
    let nation_events = query.nation_events(&owners);
    let player = query.player_histories(&nation_events);
    let ledger = query.nation_size_statistics_ledger(&player[0].history);
    println!("{}", ledger.len());
    Ok(())
}
