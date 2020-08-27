use eu4save::Eu4Extractor;
use std::env;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1])?;
    let reader = BufReader::new(file);
    let (save, _encoding) = Eu4Extractor::extract_save(reader)?;
    print!("{:#?}", save.meta.date);
    Ok(())
}
