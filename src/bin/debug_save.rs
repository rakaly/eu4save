use eu4save::Eu4Extractor;
use std::env;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1])?;
    let reader = BufReader::new(file);
    let (save, _encoding) = Eu4Extractor::builder().extract_save(reader)?;

    let query = eu4save::query::Query::from_save(save);
    let owners = query.province_owners();
    let nation_events = query.nation_events(&owners);
    let player = query.player_histories(&nation_events);
    let ledger = query.nation_size_statistics_ledger(&player[0].history);
    println!("{}", ledger.len());
    Ok(())
}
