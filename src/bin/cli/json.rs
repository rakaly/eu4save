use eu4save::{
    file::{Eu4FileEntryName, Eu4ParsedText},
    EnvTokens, Eu4File,
};
use std::{error::Error, io::Cursor};

fn json_to_stdout(file: &Eu4ParsedText) {
    let stdout = std::io::stdout();
    let _ = file.reader().json().to_writer(stdout.lock());
}

pub fn run(data: &[u8]) -> Result<(), Box<dyn Error>> {
    let file = Eu4File::from_slice(data)?;
    let mut entries = file.entries();
    while let Some(entry) = entries.next_entry() {
        if matches!(entry.name(), Some(Eu4FileEntryName::Meta) | None) {
            if file.encoding().is_binary() || file.encoding().is_zip() {
                let mut out = Cursor::new(Vec::new());
                entry.melter().verbatim(true).melt(&mut out, &EnvTokens)?;
                let text = Eu4ParsedText::from_slice(out.get_ref().as_slice())?;
                json_to_stdout(&text);
            } else {
                let text = Eu4ParsedText::from_slice(data)?;
                json_to_stdout(&text);
            }

            return Ok(());
        }
    }

    Ok(())
}
