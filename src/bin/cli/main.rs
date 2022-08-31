use std::{env, error::Error, io::Read};

mod csv;
mod debug_save;
mod deducer;
mod fmt;
mod json;
mod melt;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let stdin = std::io::stdin();
    let mut lock = stdin.lock();
    let mut buf = Vec::new();
    lock.read_to_end(&mut buf)?;

    match args[1].as_str() {
        "csv" => csv::run(&buf),
        "debug" => debug_save::run(&buf),
        "deducer" => deducer::run(&buf),
        "fmt" => fmt::run(&buf),
        "json" => json::run(&buf),
        "melt" => melt::run(&buf),
        x => panic!("unrecognized argument: {}", x),
    }?;

    Ok(())
}
