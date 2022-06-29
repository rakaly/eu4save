use eu4save::{EnvTokens, Eu4File};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let data = std::fs::read(&args[1])?;
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
