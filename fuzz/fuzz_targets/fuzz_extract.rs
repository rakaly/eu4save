#![no_main]
use eu4save::EnvTokens;
use libfuzzer_sys::fuzz_target;

fn run(data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let file = eu4save::Eu4File::from_slice(&data)?;
    let mut zip_sink = Vec::new();
    let parsed_file = file.parse(&mut zip_sink)?;

    match parsed_file.kind() {
        eu4save::file::Eu4ParsedFileKind::Text(x) => {
            x.reader().json().to_writer(std::io::sink())?;
        }
        eu4save::file::Eu4ParsedFileKind::Binary(x) => {
            x.melter().melt(&EnvTokens)?;
        }
    }

    let _meta: Result<eu4save::models::Meta, _> = parsed_file.deserializer().build(&EnvTokens);
    let _game: Result<eu4save::models::GameState, _> = parsed_file.deserializer().build(&EnvTokens);

    Ok(())
}

fuzz_target!(|data: &[u8]| {
    let _ = run(data);
});
