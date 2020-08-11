![ci](https://github.com/rakaly/eu4save/workflows/ci/badge.svg) [![](https://docs.rs/eu4save/badge.svg)](https://docs.rs/eu4save) [![Version](https://img.shields.io/crates/v/eu4save.svg?style=flat-square)](https://crates.io/crates/eu4save)

# EU4 Save

EU4 Save is a library to ergonomically work with EU4 saves (ironman + mp).

```rust
use eu4save::{Eu4Extractor, Encoding, CountryTag};
use std::io::Cursor;

let data = std::fs::read("assets/saves/eng.txt.compressed.eu4")?;
let extractor = Eu4Extractor::default();
let (save, encoding) = extractor.extract_save(Cursor::new(&data[..]))?;
assert_eq!(encoding, Encoding::TextZip);
assert_eq!(save.meta.player, CountryTag::from("ENG"));
# Ok::<(), Box<dyn std::error::Error>>(())
```

`Eu4Extractor` will deserialize both plaintext (used for mods, multiplayer,
non-ironman saves) and binary (ironman) encoded saves into the same structure.

## Querying

Even once decoded, the data might be too low level. For example a country can
have an income ledger that looks like:

```ignore
income = { 1.000 0 2.000 0.000 1.500 }
```

While the structure will decoded successfully into a vector, the context of
what each index means is missing. What value represents the income from
trade?

To help solve questions like these, the `Query` API was created

```rust
use eu4save::{Eu4Extractor, Encoding, CountryTag, query::Query};
use std::io::Cursor;

let data = std::fs::read("assets/saves/eng.txt.compressed.eu4")?;
let extractor = Eu4Extractor::default();
let (save, _encoding) = extractor.extract_save(Cursor::new(&data[..]))?;
let save_query = Query::from_save(save);
let trade = save_query.save.game.countries.get(&CountryTag::from("ENG"))
    .map(|country| save_query.country_income_breakdown(country))
    .map(|income| income.trade);
assert_eq!(Some(17.982), trade);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Ironman

By default, ironman saves will not be decoded properly.

To enable support, one must supply an environment variable
(`EU4_IRONMAN_TOKENS`) that points to a newline delimited
text file of token descriptions. For instance:

```ignore
0xffff my_test_token
0xeeee my_test_token2
```

PDS has declared that in order to comply with EU4's terms of use, the list of
tokens must not be shared. I am also restricted from divulging how the
list of tokens can be derived.

You may look to other projects EU4 ironman projects like ironmelt or paperman
for inspiration.