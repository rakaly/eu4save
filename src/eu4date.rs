use jomini::common::Date;

/// Struct specialized to parsing, formatting, and manipulating dates in EU4
pub type Eu4Date = Date;

/// EU4's start date
pub fn eu4_start_date() -> Eu4Date {
    Date::new(1444, 11, 11).unwrap()
}
