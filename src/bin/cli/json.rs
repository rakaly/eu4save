use eu4save::{
    file::{Eu4FileEntryName, Eu4FsFileKind, Eu4ParsedText},
    BasicTokenResolver, Eu4File,
};
use std::{error::Error, io::Read};

fn json_to_stdout(file: &Eu4ParsedText) {
    let stdout = std::io::stdout();
    let _ = file.reader().json().to_writer(stdout.lock());
}

pub fn run(path: &str) -> Result<(), Box<dyn Error>> {
    let file = std::fs::File::open(path)?;
    let mut file = Eu4File::from_file(file)?;

    let file_data = std::fs::read("assets/eu4.txt").unwrap_or_default();
    let resolver = BasicTokenResolver::from_text_lines(file_data.as_slice())?;

    let melt_options = eu4save::MeltOptions::new().verbatim(true);
    match file.kind_mut() {
        Eu4FsFileKind::Text(x) => {
            let mut buf = Vec::new();
            x.read_to_end(&mut buf)?;
            let text = Eu4ParsedText::from_raw(&buf)?;
            json_to_stdout(&text);
        }
        Eu4FsFileKind::Binary(x) => {
            let mut buf = Vec::new();
            x.as_ref().melt(melt_options, resolver, &mut buf)?;
            let text = Eu4ParsedText::from_slice(&buf)?;
            json_to_stdout(&text);
        }
        Eu4FsFileKind::Zip(x) => {
            let mut meta = x.get(Eu4FileEntryName::Meta)?;
            let mut data = Vec::new();
            if x.encoding().is_binary() {
                meta.melt(melt_options, resolver, &mut data)?;
            } else {
                meta.read_to_end(&mut data)?;
            }

            let text = Eu4ParsedText::from_slice(&data)?;
            json_to_stdout(&text);
        }
    }

    Ok(())
}
