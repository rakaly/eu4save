use eu4save::{EnvTokens, Eu4File, FailedResolveStrategy};
use std::env;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_data = std::fs::read(&args[1]).unwrap();
    let file = Eu4File::from_slice(&file_data).unwrap();
    let mut zip_sink = Vec::new();
    let parsed_file = file.parse(&mut zip_sink).unwrap();
    let binary = parsed_file.as_binary().unwrap();
    let out = binary
        .melter()
        .on_failed_resolve(FailedResolveStrategy::Error)
        .melt(&EnvTokens)
        .unwrap();

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(out.data()).unwrap();
}
