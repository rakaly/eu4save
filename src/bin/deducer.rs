use eu4save::{
    models::{Country, Eu4Save},
    Eu4Extractor,
};
use std::collections::HashSet;
use std::env;
use std::io::Cursor;

#[derive(Debug)]
struct Deduce {
    country: String,
    index: usize,
    value: f32,
}

fn deduce_vec(save: &Eu4Save, f: impl Fn(&Country) -> &[f32]) {
    let mut ded = Vec::new();
    let mut found_indices = HashSet::new();
    let mut max_indices = 0;

    for (tag, country) in &save.game.countries {
        max_indices = std::cmp::max(max_indices, f(country).len());
        for (i, &value) in f(country).iter().enumerate() {
            if value != 0.0 && !found_indices.contains(&i) {
                found_indices.insert(i);
                ded.push(Deduce {
                    index: i,
                    value,
                    country: tag.to_string(),
                });
            }
        }
    }

    ded.sort_by_key(|x| x.index);
    println!("tag\tindex\tvalue");
    for item in &ded {
        println!("{}\t{}\t{}", item.country, item.index, item.value);
    }

    let mut missing_indices = Vec::new();
    for i in 0..max_indices {
        if ded.iter().find(|&x| i == x.index).is_none() {
            missing_indices.push(i);
        }
    }

    println!("missing {:?}", missing_indices);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let data = std::fs::read(&args[1])?;
    let (save, _encoding) = Eu4Extractor::extract_save(Cursor::new(&data[..]))?;
    deduce_vec(&save, |c| &c.ledger.income);
    deduce_vec(&save, |c| &c.ledger.expense);

    Ok(())
}
