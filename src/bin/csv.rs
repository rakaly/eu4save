use eu4save::PdsDate;
use std::env;
use std::error::Error;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let file_data = std::fs::read(&args[1])?;
    let cursor = Cursor::new(&file_data);
    let (save, _) = eu4save::Eu4Extractor::builder().extract_save(cursor)?;

    println!("date,tag,prestige");
    for (tag, country) in &save.game.countries {
        if country.num_of_cities > 0 {
            println!("{},{},{}", save.meta.date.iso_8601(), tag, country.prestige);
        }
    }

    Ok(())
}
