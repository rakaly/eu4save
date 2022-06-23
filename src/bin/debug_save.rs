use eu4save::{
    models::{GameState, LedgerData},
    tokens::EnvTokens,
    Eu4File,
};
use serde::Deserialize;
use std::{env, time::Instant};

#[derive(Deserialize)]
struct MyGame {
    score_statistics: LedgerData,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let mut zip_sink = Vec::new();
    let data = std::fs::read(&args[1])?;
    let file = Eu4File::from_slice(&data)?;
    let file = file.parse(&mut zip_sink)?;

    let now = Instant::now();
    let _abc: MyGame = file.deserializer().build(&EnvTokens)?;
    let elapsed_time = now.elapsed();
    println!("Running score took {}us.", elapsed_time.as_micros());

    let now = Instant::now();
    let _abc: GameState = file.deserializer().build(&EnvTokens)?;
    let elapsed_time = now.elapsed();
    println!("Running gamestate took {}us.", elapsed_time.as_micros());

    // let file = File::open(&args[1])?;
    // let reader = BufReader::new(file);
    // let (save, _encoding) = Eu4Extractor::builder().extract_save(reader)?;

    // let query = eu4save::query::Query::from_save(save);
    // let owners = query.province_owners();
    // let nation_events = query.nation_events(&owners);
    // let player = query.player_histories(&nation_events);
    // let ledger = query.nation_size_statistics_ledger(&player[0].history);
    // println!("{}", ledger.len());
    Ok(())
}
