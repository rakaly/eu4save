/*!
EU4 Save is a library to ergonomically work with EU4 saves (ironman + mp).

```rust
use eu4save::{Eu4Extractor, Encoding, CountryTag};
use std::io::Cursor;

let data = std::fs::read("assets/saves/eng.txt.compressed.eu4")?;
let (save, encoding) = Eu4Extractor::extract_save(Cursor::new(&data[..]))?;
assert_eq!(encoding, Encoding::TextZip);
assert_eq!(save.meta.player, "ENG".parse()?);
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
let (save, _encoding) = Eu4Extractor::extract_save(Cursor::new(&data[..]))?;
let save_query = Query::from_save(save);
let trade = save_query.country(&"ENG".parse()?)
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
*/

mod country_tag;
mod de;
mod dlc;
mod errors;
mod eu4date;
mod extraction;
pub(crate) mod flavor;
mod melt;
/// Repository of raw structs extracted from a save file
pub mod models;
mod province_id;
/// Ergonomic module for querying info from a save file
pub mod query;
mod tag_resolver;
pub mod tokens;

pub use country_tag::*;
pub use dlc::*;
pub use errors::*;
pub use eu4date::*;
pub use extraction::*;
pub use jomini::FailedResolveStrategy;
pub use melt::*;
pub use province_id::*;
pub use tag_resolver::*;
