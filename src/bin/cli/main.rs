use std::{env, error::Error};

mod csv;
mod debug_save;
mod deducer;
mod fmt;
mod json;
mod melt;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    match args[1].as_str() {
        "csv" => csv::run(args[2].as_str()),
        "debug" => debug_save::run(args[2].as_str()),
        "deducer" => deducer::run(args[2].as_str()),
        "fmt" => fmt::run(args[2].as_str()),
        "json" => json::run(args[2].as_str()),
        "melt" => melt::run(args[2].as_str()),
        x => panic!("unrecognized argument: {}", x),
    }?;

    Ok(())
}
