use eu4save::{query::Query, CountryTag, Eu4Extractor};
use std::error::Error;
use std::fs;
use std::io::Cursor;
use walkdir::WalkDir;

/*
#[test]
pub fn ironman_saves_detected() -> Result<(), Box<dyn Error>> {
    for entry in WalkDir::new("assets/saves/ironman") {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            println!("parsing {}", path.display());
            let data = fs::read(path)?;
            let extractor = Eu4Extractor::default();
            let (_save, encoding) = extractor.extract_save(Cursor::new(&data[..]))?;
            assert_eq!(encoding, eu4save::Encoding::BinZip);
        }
    }

    Ok(())
}
*/
/*
#[test]
fn test_true_heir_of_timur() {*/
    // Not only is this true heir of timur but the uploader used the intermediate tag of delhi to
    // further form the mughals, "annexed Delhi, flipped to Kashimiri, and formed Delhi to get a
    // bunch of free cores". So this save is especially important to ensure that start and end
    // country detection is working correctly.
    // https://www.reddit.com/r/eu4/comments/hnwnd2/sistan_true_heir_of_timur_in_1508/
/*    let data = fs::read("assets/saves/ironman/sis.eu4").unwrap();
    let extractor = Eu4Extractor::default();
    let (save, _encoding) = extractor.extract_save(Cursor::new(&data[..])).unwrap();
    let query = Query::from_save(save);
    assert_eq!(
        query.starting_country.as_ref().unwrap(),
        &CountryTag::from("SIS")
    );
}*/
