use jomini::FailedResolveStrategy;
use std::env;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_data = std::fs::read(&args[1]).unwrap();
    let melted = eu4save::melt(&file_data[..], FailedResolveStrategy::Error).unwrap();

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(&melted[..]).unwrap();
}
