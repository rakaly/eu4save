use eu4save::{BasicTokenResolver, Eu4File, FailedResolveStrategy, MeltOptions};
use std::{error::Error, io::BufWriter};

pub fn run(path: &str) -> Result<(), Box<dyn Error>> {
    let file = std::fs::File::open(path)?;
    let mut file = Eu4File::from_file(file)?;

    let stdout = std::io::stdout();
    let handle = stdout.lock();
    let mut writer = BufWriter::new(handle);

    let file_data = std::fs::read("assets/eu4.txt").unwrap_or_default();
    let resolver = BasicTokenResolver::from_text_lines(file_data.as_slice())?;
    file.melt(
        MeltOptions::new().on_failed_resolve(FailedResolveStrategy::Error),
        resolver,
        &mut writer,
    )?;

    Ok(())
}
