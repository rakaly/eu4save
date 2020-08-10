#![no_main]
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let extractor = eu4save::Eu4Extractor::default();
    let _ = extractor.extract_save(Cursor::new(&data[..]));
});
