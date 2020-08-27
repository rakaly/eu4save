#![no_main]
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let _ = eu4save::Eu4Extractor::extract_save(Cursor::new(&data[..]));
});
