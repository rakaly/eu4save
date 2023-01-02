use eu4save::{EnvTokens, Eu4File, PdsDate};
use std::error::Error;

pub fn run(file_data: &[u8]) -> Result<(), Box<dyn Error>> {
    let file = Eu4File::from_slice(file_data)?;
    let save = file.parse_save(&EnvTokens)?;

    println!("date,tag,prestige");
    for (tag, country) in &save.game.countries {
        if country.num_of_cities > 0 {
            println!("{},{},{}", save.meta.date.iso_8601(), tag, country.prestige);
        }
    }

    Ok(())
}
