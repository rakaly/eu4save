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
        "csv" => csv::run(&args[2]),
        "debug" => debug_save::run(&args[2]),
        "deducer" => deducer::run(&args[2]),
        "fmt" => fmt::run(&args[2]),
        "json" => json::run(&args[2]),
        "melt" => melt::run(&args[2]),
        x => panic!("unrecognized argument: {}", x),
    }?;

    Ok(())
}
