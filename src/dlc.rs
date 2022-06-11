/// Map dlc name to its id
///
/// Every dlc has an id. This id is not present in the save file but it can be useful
/// to have the id so one doesn't need to compare strings.
///
/// For more information on IDs see the wiki:
/// <https://eu4.paradoxwikis.com/Downloadable_content>
///
/// ```rust
/// assert_eq!(eu4save::dlc_id("Dharma"), Some(90))
/// ```
pub fn dlc_id(name: &str) -> Option<i32> {
    match name {
        "Conquest of Paradise" => Some(10),
        "Wealth of Nations" => Some(18),
        "Res Publica" => Some(21),
        "Art of War" => Some(27),
        "El Dorado" => Some(33),
        "Common Sense" => Some(39),
        "The Cossacks" => Some(46),
        "Mare Nostrum" => Some(55),
        "Rights of Man" => Some(60),
        "Mandate of Heaven" => Some(66),
        "Third Rome" => Some(72),
        "Cradle of Civilization" => Some(77),
        "Rule Britannia" => Some(84),
        "Dharma" => Some(90),
        "Golden Century" => Some(95),
        "Emperor" => Some(101),
        "Leviathan" => Some(106),
        "Origins" => Some(110),
        _ => None,
    }
}
