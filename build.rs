use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

fn main() {
    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("gen_tokens.rs");
    let mut writer = BufWriter::new(File::create(&out_path).unwrap());
    writeln!(writer, "match token {{").unwrap();

    println!("cargo:rerun-if-env-changed=EU4_IRONMAN_TOKENS");
    if let Ok(v) = env::var("EU4_IRONMAN_TOKENS") {
        println!("cargo:rustc-cfg=ironman");
        println!("cargo:rerun-if-changed={}", v);
        let file = File::open(&v).unwrap();
        let mut reader = BufReader::new(file);

        let mut line = String::new();
        while reader.read_line(&mut line).unwrap() != 0 {
            let (token_val, token_s) = line.split_once(' ').unwrap();
            writeln!(writer, "{} => Some(\"{}\"),", token_val, token_s.trim()).unwrap();
            line.clear();
        }
    }

    writeln!(writer, "_ => None,").unwrap();
    writeln!(writer, "}}").unwrap();
}
