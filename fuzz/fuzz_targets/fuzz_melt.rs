#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = eu4save::melt(&data, eu4save::FailedResolveStrategy::Ignore);
});
