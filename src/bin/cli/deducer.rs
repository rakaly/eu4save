use eu4save::{CountryTag, EnvTokens, Eu4File};
use std::{collections::HashSet, error::Error, fmt::Display};

#[derive(Debug)]
struct Deduce<N> {
    country: CountryTag,
    index: usize,
    value: N,
}

fn deduce_vec<'a, N>(iter: impl Iterator<Item = (CountryTag, &'a [N])>)
where
    N: 'a + PartialEq + Default + Display,
{
    let mut ded = Vec::new();
    let mut found_indices = HashSet::new();
    let mut max_indices = 0;
    let default_val = N::default();

    for (tag, vals) in iter {
        max_indices = std::cmp::max(max_indices, vals.len());
        for (i, value) in vals.iter().enumerate() {
            if value.ne(&default_val) && !found_indices.contains(&i) {
                found_indices.insert(i);
                ded.push(Deduce {
                    index: i,
                    value,
                    country: tag,
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
        if !ded.iter().any(|x| i == x.index) {
            missing_indices.push(i);
        }
    }
}

pub fn run(data: &[u8]) -> Result<(), Box<dyn Error>> {
    let file = Eu4File::from_slice(&data)?;
    let save = file.deserializer().build_save(&EnvTokens)?;
    deduce_vec(
        save.game
            .countries
            .iter()
            .map(|(tag, c)| (*tag, c.ledger.income.as_slice())),
    );
    deduce_vec(
        save.game
            .countries
            .iter()
            .map(|(tag, c)| (*tag, c.ledger.expense.as_slice())),
    );
    deduce_vec(
        save.game
            .countries
            .iter()
            .filter(|(_tag, c)| c.num_of_cities > 0)
            .map(|(tag, c)| (*tag, c.losses.members.as_slice())),
    );

    Ok(())
}
