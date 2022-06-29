use eu4save::{EnvTokens, Eu4File, PdsDate};
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let file_data = std::fs::read(&args[1])?;
    let file = Eu4File::from_slice(&file_data)?;
    let save = file.deserializer().build_save(&EnvTokens)?;

    println!("date,tag,prestige");
    for (tag, country) in &save.game.countries {
        if country.num_of_cities > 0 {
            println!("{},{},{}", save.meta.date.iso_8601(), tag, country.prestige);
        }
    }

    Ok(())
}
