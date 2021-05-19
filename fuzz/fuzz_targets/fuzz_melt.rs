#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = eu4save::Melter::new().with_on_failed_resolve(eu4save::FailedResolveStrategy::Ignore).melt(data);
});
