use std::error::Error;

use eu4save::{
    file::{Eu4FileEntryName, Eu4ParsedFile, Eu4ParsedFileKind, Eu4Text},
    models::SavegameVersion,
    EnvTokens, Eu4File,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct MyMeta {
    savegame_version: SavegameVersion,
}

fn json_to_stdout(file: &Eu4Text) {
    let _ = file.reader().json().to_writer(std::io::stdout());
}

fn parsed_file_to_json(file: &Eu4ParsedFile) -> Result<(), Box<dyn std::error::Error>> {
    // if the save is binary, melt it, as the JSON API only works with text
    match file.kind() {
        Eu4ParsedFileKind::Text(text) => json_to_stdout(text),
        Eu4ParsedFileKind::Binary(binary) => {
            let melted = binary.melter().verbatim(true).melt(&EnvTokens)?;
            json_to_stdout(&Eu4Text::from_slice(melted.data())?);
        }
    };

    Ok(())
}

pub fn run(file_path: &str) -> Result<(), Box<dyn Error>> {
    let data = std::fs::read(file_path).unwrap();

    let file = Eu4File::from_slice(&data)?;
    let mut entries = file.entries();
    let mut zip_sink = Vec::new();
    while let Some(entry) = entries.next_entry() {
        if matches!(entry.name(), Some(Eu4FileEntryName::Meta) | None) {
            let parsed_file = entry.parse(&mut zip_sink)?;
            let meta: MyMeta = parsed_file.deserializer().build(&EnvTokens)?;

            if meta.savegame_version.second < 29 {
                println!("detected save game earlier than 1.29");
                return Ok(());
            }

            // if this is an uncompressed save, it could be quite large, so we
            // want to avoid reparsing it. Otherwise, we'll process the rest of
            // the zip data
            if !file.encoding().is_zip() {
                parsed_file_to_json(&parsed_file)?;
            } else {
                zip_sink.clear();
                let parsed_file = file.parse(&mut zip_sink)?;
                parsed_file_to_json(&parsed_file)?;
            }

            return Ok(());
        }
    }

    Ok(())
}
