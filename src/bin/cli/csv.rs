use eu4save::{BasicTokenResolver, Eu4File, PdsDate};
use std::error::Error;

pub fn run(file_data: &[u8]) -> Result<(), Box<dyn Error>> {
    let file = Eu4File::from_slice(file_data)?;
    let file_data = std::fs::read("assets/eu4.txt").unwrap_or_default();
    let resolver = BasicTokenResolver::from_text_lines(file_data.as_slice())?;
    let save = file.parse_save(&resolver)?;

    println!("date,tag,prestige");
    for (tag, country) in &save.game.countries {
        if country.num_of_cities > 0 {
            println!("{},{},{}", save.meta.date.iso_8601(), tag, country.prestige);
        }
    }

    Ok(())
}
