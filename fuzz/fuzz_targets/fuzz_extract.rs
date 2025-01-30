#![no_main]
use libfuzzer_sys::fuzz_target;
use eu4save::BasicTokenResolver;
use std::sync::LazyLock;

static TOKENS: LazyLock<BasicTokenResolver> = LazyLock::new(|| {
    let file_data = std::fs::read("assets/eu4.txt").unwrap();
    BasicTokenResolver::from_text_lines(file_data.as_slice()).unwrap()
});

fn run(data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let file = eu4save::Eu4File::from_slice(&data)?;
    let _ = file.parse_save(&*TOKENS);

    let mut zip_sink = Vec::new();
    let parsed_file = file.parse(&mut zip_sink)?;

    let mut sink = std::io::sink();
    let _ = file.melter().melt(&mut sink, &*TOKENS);

    match parsed_file.kind() {
        eu4save::file::Eu4ParsedFileKind::Text(x) => {
            x.reader().json().to_writer(std::io::sink())?;
        }
        _ => {}
    }

    Ok(())
}

fuzz_target!(|data: &[u8]| {
    let _ = run(data);
});
