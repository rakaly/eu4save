use eu4save::Eu4Extractor;
use std::io::Cursor;

#[test]
fn fix_crash_on_long_country_tag_debug_mode() {
    let data = include_bytes!("fixtures/crash1.bin");
    let extractor = Eu4Extractor::default();
    let _ = extractor.extract_save(Cursor::new(&data[..]));
}
