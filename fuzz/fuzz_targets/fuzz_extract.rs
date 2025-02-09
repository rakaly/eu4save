#![no_main]
use eu4save::{file::Eu4ParsedText, BasicTokenResolver};
use libfuzzer_sys::fuzz_target;
use std::sync::LazyLock;

static TOKENS: LazyLock<BasicTokenResolver> = LazyLock::new(|| {
    let file_data = std::fs::read("assets/eu4.txt").unwrap();
    BasicTokenResolver::from_text_lines(file_data.as_slice()).unwrap()
});

fn run(data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let file = eu4save::Eu4File::from_slice(&data)?;

    let mut sink = std::io::sink();
    let _ = file.melt(eu4save::MeltOptions::new(), &*TOKENS, &mut sink);
    let _ = file.parse_save(&*TOKENS);
    let _ = file.size();
    let _ = file.encoding();

    match file.kind() {
        eu4save::file::Eu4SliceFileKind::Text(x) => {
            Eu4ParsedText::from_raw(x.get_ref())?
                .reader()
                .json()
                .to_writer(std::io::sink())?;
        }
        _ => {}
    }

    Ok(())
}

fuzz_target!(|data: &[u8]| {
    let _ = run(data);
});
