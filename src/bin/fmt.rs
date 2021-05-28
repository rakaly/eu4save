use jomini::{TextTape, TextWriterBuilder};
use std::{
    env,
    io::{stdout, BufWriter, Write},
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_data = std::fs::read(&args[1]).unwrap();
    let header = b"EU4txt";
    if file_data.len() < header.len() {
        eprintln!("can only format plain eu4 file");
        std::process::exit(1);
    }

    let (file_header, data) = file_data.split_at(header.len());
    if file_header != header {
        eprintln!("can only format plain eu4 file");
        std::process::exit(1);
    }

    let tape = TextTape::from_slice(data).unwrap();
    let stdout = stdout();
    let stdout_lock = stdout.lock();
    let buf_stdout = BufWriter::new(stdout_lock);
    let mut writer = TextWriterBuilder::new().from_writer(buf_stdout);
    writer.write_tape(&tape).unwrap();
    let mut buf_stdout = writer.into_inner();
    buf_stdout.flush().unwrap();
}
