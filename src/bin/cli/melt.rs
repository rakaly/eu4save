use eu4save::{EnvTokens, Eu4File, FailedResolveStrategy};
use std::{error::Error, io::BufWriter};

pub fn run(file_data: &[u8]) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout();
    let handle = stdout.lock();
    let mut writer = BufWriter::new(handle);

    let file = Eu4File::from_slice(file_data)?;
    let mut melter = file.melter();
    melter.on_failed_resolve(FailedResolveStrategy::Error);
    melter.melt(&mut writer, &EnvTokens)?;

    Ok(())
}
