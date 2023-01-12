use crate::{
    models::{
        Country, CountryEvent, Eu4Save, LedgerData, LedgerDatum, Province, ProvinceEvent,
        ProvinceEventValue, WarEvent,
    },
    ProvinceId, TagResolver,
};
use crate::{CountryTag, Eu4Date, PdsDate};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU16,
};

#[derive(Debug)]
pub struct AnnualLedgers {
    pub score: Vec<LedgerDatum>,
    pub inflation: Vec<LedgerDatum>,
    pub size: Vec<LedgerDatum>,
    pub income: Vec<LedgerDatum>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CountryIncomeLedger {
    pub taxation: f32,
    pub production: f32,
    pub trade: f32,
    pub gold: f32,
    pub tariffs: f32,
    pub vassals: f32,
    pub harbor_fees: f32,
    pub subsidies: f32,
    pub war_reparations: f32,
    pub interest: f32,
    pub gifts: f32,
    pub events: f32,
    pub spoils_of_war: f32,
    pub treasure_fleet: f32,
    pub siphoning_income: f32,
    pub condottieri: f32,
    pub knowledge_sharing: f32,
    pub blockading_foreign_ports: f32,
    pub looting_foreign_cities: f32,
    pub other: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CountryExpenseLedger {
    pub advisor_maintenance: f32,
    pub interest: f32,
    pub state_maintenance: f32,
    pub subsidies: f32,
    pub war_reparations: f32,
    pub army_maintenance: f32,
    pub fleet_maintenance: f32,
    pub fort_maintenance: f32,
    pub colonists: f32,
    pub missionaries: f32,
    pub raising_armies: f32,
    pub building_fleets: f32,
    pub building_fortresses: f32,
    pub buildings: f32,
    pub repaid_loans: f32,
    pub gifts: f32,
    pub advisors: f32,
    pub events: f32,
    pub peace: f32,
    pub vassal_fee: f32,
    pub tariffs: f32,
    pub support_loyalists: f32,
    pub condottieri: f32,
    pub root_out_corruption: f32,
    pub embrace_institution: f32,
    pub knowledge_sharing: f32,
    pub trade_company_investments: f32,
    pub other: f32,
    pub ports_blockaded: f32,
    pub cities_looted: f32,
    pub monuments: f32,
    pub cot_upgrades: f32,
    pub colony_changes: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CountryManaUsage {
    pub adm: CountryManaSpend,
    pub dip: CountryManaSpend,
    pub mil: CountryManaSpend,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CountryManaSpend {
    pub buy_idea: i32,
    pub advance_tech: i32,
    pub boost_stab: i32,
    pub buy_general: i32,
    pub buy_admiral: i32,
    pub buy_conq: i32,
    pub buy_explorer: i32,
    pub develop_prov: i32,
    pub force_march: i32,
    pub assault: i32,
    pub seize_colony: i32,
    pub burn_colony: i32,
    pub attack_natives: i32,
    pub scorch_earth: i32,
    pub demand_non_wargoal_prov: i32,
    pub reduce_inflation: i32,
    pub move_capital: i32,
    pub make_province_core: i32,
    pub replace_rival: i32,
    pub change_gov: i32,
    pub change_culture: i32,
    pub harsh_treatment: i32,
    pub reduce_we: i32,
    pub boost_faction: i32,
    pub raise_war_taxes: i32,
    pub buy_native_advancement: i32,
    pub increse_tariffs: i32,
    pub promote_merc: i32,
    pub decrease_tariffs: i32,
    pub move_trade_port: i32,
    pub create_trade_post: i32,
    pub siege_sorties: i32,
    pub buy_religious_reform: i32,
    pub set_primary_culture: i32,
    pub add_accepted_culture: i32,
    pub remove_accepted_culture: i32,
    pub strengthen_government: i32,
    pub boost_militarization: i32,
    pub artillery_barrage: i32,
    pub establish_siberian_frontier: i32,
    pub government_interaction: i32,
    pub naval_barrage: i32,
    pub create_leader: i32,
    pub enforce_culture: i32,
    pub effect: i32,
    pub minority_expulsion: i32,
    pub other: i32,
    pub add_tribal_land: i32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum BuildingConstruction {
    Constructed,
    Destroyed,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct BuildingEvent<'a> {
    pub building: &'a str,
    pub date: Eu4Date,
    pub action: BuildingConstruction,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct PlayerHistory {
    pub history: NationEvents,

    /// Whether the player is currently in the session
    pub is_human: bool,

    /// Names of the players (may be empty)
    pub player_names: Vec<String>,
}

pub struct ProvinceOwners {
    /// Initial owners of provinces, index using province id
    pub initial: Vec<Option<CountryTag>>,

    /// Sorted by date and then province id    
    pub changes: Vec<ProvinceOwnerChange>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct ProvinceOwnerChange {
    pub province: ProvinceId,
    pub tag: CountryTag,
    pub date: Eu4Date,
}

pub struct ProvinceReligions {
    pub initial: Vec<Option<ReligionIndex>>,
    pub changes: Vec<ProvinceReligionChange>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProvinceReligionChange {
    pub province: ProvinceId,
    pub religion: ReligionIndex,
    pub date: Eu4Date,
}

pub struct ReligionLookup {
    religions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReligionIndex(NonZeroU16);

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Player {
    pub name: String,
    pub tag: CountryTag,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct NationEvents {
    /// The initial starting tag for a country. In a TYR -> IRE -> GBR run,
    /// this would be TYR
    pub initial: CountryTag,

    /// The latest tag that a country achieved. If DMS -> IRE but then
    /// IRE is annexed by SCO which culture shifts to form IRE then both
    /// initial tags of SCO and DMS will report a latest tag of IRE
    pub latest: CountryTag,

    /// The tag which the history of this country is stored under. For
    /// instance if ULM forms byzantium then the initial byzantium operator's
    /// history is stored under ULM
    pub stored: CountryTag,

    /// An ordered (by date) recounting of how the initial tag became the
    /// the latest tag. May be empty for nations that did not tag switch,
    /// get annexed, etc.
    pub events: Vec<NationEvent>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct NationEvent {
    pub date: Eu4Date,
    pub kind: NationEventKind,
}

impl NationEvent {
    pub fn as_tag_switch(&self) -> Option<(Eu4Date, CountryTag)> {
        if let NationEventKind::TagSwitch(to) = self.kind {
            Some((self.date, to))
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub enum NationEventKind {
    TagSwitch(CountryTag),
    Appeared,
    Annexed,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct LedgerPoint {
    pub tag: CountryTag,
    pub year: u16,
    pub value: i32,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct ResolvedWarParticipant {
    /// The tag as it appears in the war history
    pub tag: CountryTag,

    /// The actual location of the tag in country history
    pub stored: CountryTag,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct ResolvedWarParticipants {
    pub war: String,
    pub participants: Vec<ResolvedWarParticipant>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct SaveCountry<'a> {
    pub id: usize,
    pub tag: CountryTag,
    pub country: &'a Country,
}

#[derive(Debug, Clone, Copy)]
struct TagId {
    id: usize,
    tag: CountryTag,
}

#[derive(Debug, Clone, Serialize)]
pub struct Inheritance {
    pub start_t0_year: i16,
    pub end_t0_year: i16,
    pub start_t1_year: i16,
    pub end_t1_year: i16,
    pub start_t2_year: i16,
    pub end_t2_year: i16,
    pub t_value: u8,
    pub inheritance_value: u8,
    pub subtotal: i64,
    pub heretic_offset: i32,
    pub calculations: Vec<Calculation>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum TagDependency {
    Independent,
    Dependent(CountryTag),
}

#[derive(Debug, Clone, Serialize)]
pub struct Calculation {
    pub name: String,
    pub value: i64,
    pub dependency: TagDependency,
}

impl Calculation {
    pub fn new(name: &str, value: i64, dependency: TagDependency) -> Self {
        Self {
            name: String::from(name),
            value,
            dependency,
        }
    }
}

#[derive(Debug)]
pub struct Query {
    save: Eu4Save,
    tag_ids: Vec<TagId>,
    tag_lookup: HashMap<CountryTag, TagId>,
    buildings: OnceCell<HashSet<String>>,
}

impl Query {
    pub fn from_save(save: Eu4Save) -> Self {
        let tag_ids: Vec<_> = save
            .game
            .countries
            .iter()
            .enumerate()
            .map(|(i, (tag, _))| TagId { id: i, tag: *tag })
            .collect();

        let tag_lookup = tag_ids.iter().map(|id| (id.tag, *id)).collect();

        Query {
            save,
            tag_ids,
            tag_lookup,
            buildings: OnceCell::default(),
        }
    }

    pub fn save(&self) -> &Eu4Save {
        &self.save
    }

    pub fn countries(&self) -> impl Iterator<Item = SaveCountry> + '_ {
        self.save
            .game
            .countries
            .iter()
            .zip(self.tag_ids.iter())
            .map(|((_, country), tag_id)| SaveCountry {
                id: tag_id.id,
                tag: tag_id.tag,
                country,
            })
    }

    pub fn save_country(&self, tag: &CountryTag) -> Option<SaveCountry> {
        self.tag_lookup.get(tag).and_then(|tag_id| {
            self.save
                .game
                .countries
                .get(tag_id.id)
                .map(|(_, country)| SaveCountry {
                    id: tag_id.id,
                    tag: tag_id.tag,
                    country,
                })
        })
    }

    pub fn country(&self, tag: &CountryTag) -> Option<&Country> {
        self.save_country(tag).map(|x| x.country)
    }

    /// Returns a set of the names of the players who participated in a playthrough.
    /// May be blank for single player run through where the player is detected as
    /// "Player". No guarantees are given as to the state of each player's country.
    /// Annexed countries may still show the original player. It is undefined what
    /// happens to players when a formable nation is annexed and then reformed by
    /// another player.
    pub fn players(&self) -> Vec<Player> {
        players(&self.save)
    }

    /// Provides a rich structure that contains the following:
    ///
    /// - A list countries that had a player at one point or another
    /// - What country the player is playing
    /// - Whether that player is currently in the session
    /// - The names of the players in the session
    /// - A list of all prior tags that country had been
    pub fn player_histories(&self, nation_events: &[NationEvents]) -> Vec<PlayerHistory> {
        let save = self.save();
        let mut result = Vec::with_capacity(save.game.players_countries.len());
        let players = players(save);
        let mut leftovers = Vec::new();
        for (tag, country) in save.game.countries.iter().filter(|(_, c)| c.was_player) {
            let tag = *tag;
            let tag_players: Vec<_> = players
                .iter()
                .filter(|x| x.tag == tag)
                .map(|x| x.name.clone())
                .collect();
            let history = nation_events
                .iter()
                .find(|x| x.stored == tag)
                .cloned()
                .unwrap_or_else(|| NationEvents {
                    initial: tag,
                    latest: tag,
                    stored: tag,
                    events: Vec::new(),
                });

            let no_players = tag_players.is_empty();
            let history = PlayerHistory {
                history,
                is_human: country.human,
                player_names: tag_players,
            };

            if country.was_player && no_players {
                leftovers.push(history);
            } else {
                result.push(history);
            }
        }

        // Only for ironman will we try and resolve "release and play as" as the
        // save does not often paint an accurate picture of these transitions. And
        // we need to track these for achievements like spaghetti western.
        if result.len() == 1 && save.meta.is_ironman {
            for x in &mut result {
                if x.is_human && x.player_names.len() == 1 && x.history.stored == x.history.latest {
                    let country = self.country(&x.history.stored);
                    let rpa = country.map_or(false, |x| x.has_switched_nation);
                    let alive = country.map_or(false, |x| x.num_of_cities > 0);
                    if rpa && alive {
                        if let Some(end) = leftovers.pop() {
                            x.history.initial = end.history.initial;
                        }
                    }
                }
            }
        }

        result.append(&mut leftovers);

        result
    }

    /// Calculates the major events that befell countries (annexations, appearances, and tag switches)
    pub fn nation_events(&self, province_owners: &ProvinceOwners) -> Vec<NationEvents> {
        nation_events(&self.save, province_owners)
    }

    /// Aggregate when lands changed hands
    pub fn province_owners(&self) -> ProvinceOwners {
        province_owners(&self.save)
    }

    pub fn province_religions(&self, lookup: &ReligionLookup) -> ProvinceReligions {
        province_religions(&self.save, lookup)
    }

    pub fn religion_lookup(&self) -> ReligionLookup {
        let mut religions = self
            .save
            .game
            .religions
            .iter()
            .map(|(key, _religion)| key.clone())
            .collect::<Vec<_>>();
        religions.sort_unstable();
        ReligionLookup { religions }
    }

    /// Return the starting country in single player playthroughs. If playing in multiplayer or if
    /// the starting country can't be determined then none is returned.
    pub fn starting_country(&self, histories: &[PlayerHistory]) -> Option<CountryTag> {
        let mut preexisting = histories.iter().filter(|x| {
            !x.history
                .events
                .first()
                .map_or(false, |x| x.kind == NationEventKind::Appeared)
        });
        let first = preexisting.next();
        let second = preexisting.next();

        if second.is_some() {
            return None;
        }

        first.map(|x| x.history.initial).or(match histories {
            [player] => Some(player.history.initial),
            _ => None,
        })
    }

    pub fn tag_resolver(&self, nation_events: &[NationEvents]) -> TagResolver {
        TagResolver::create(nation_events)
    }

    fn inherit_subtotal(&self, country: &SaveCountry) -> (i64, Vec<Calculation>, i64) {
        let hre_ruler = self
            .save()
            .game
            .empire
            .as_ref()
            .and_then(|x| x.emperor)
            .and_then(|x| self.country(&x))
            .and_then(|x| x.monarch.as_ref())
            .map(|x| i64::from(x.id))
            .unwrap_or_default();

        let papacy_controller = self
            .save()
            .game
            .religion_instance_data
            .get("catholic")
            .and_then(|x| x.papacy.as_ref())
            .and_then(|x| self.save_country(&x.controller));

        let papacy_tag = papacy_controller
            .as_ref()
            .map(|x| x.tag)
            .unwrap_or_else(|| "---".parse().unwrap());
        let papacy_id = papacy_controller
            .as_ref()
            .map(|x| x.id as i64)
            .unwrap_or_default();

        let ruler = country
            .country
            .monarch
            .as_ref()
            .map(|x| i64::from(x.id))
            .unwrap_or_default();

        let previous_rulers = country
            .country
            .previous_monarchs
            .iter()
            .map(|x| i64::from(x.id))
            .sum::<i64>();

        let capital_province = i64::from(country.country.capital.as_u16());

        let provinces = i64::from(country.country.num_of_cities);

        #[rustfmt::skip]
        let calculations = vec![
            Calculation::new("Nation ID", country.id as i64, TagDependency::Dependent(country.tag)),
            Calculation::new("HRE Ruler ID", hre_ruler, TagDependency::Independent),
            Calculation::new("Curia Controller Nation ID", papacy_id, TagDependency::Dependent(papacy_tag)),
            Calculation::new("Ruler ID", ruler, TagDependency::Dependent(country.tag)),
            Calculation::new("Previous Ruler IDs", previous_rulers, TagDependency::Dependent(country.tag)),
            Calculation::new("Capital Province", capital_province, TagDependency::Dependent(country.tag)),
            Calculation::new("Owned Provinces", provinces, TagDependency::Dependent(country.tag)),
        ];

        let raw = hre_ruler
            + papacy_id
            + ruler
            + previous_rulers
            + capital_province
            + provinces
            + country.id as i64;

        (raw, calculations, papacy_id)
    }

    pub fn inherit(&self, country: &SaveCountry) -> Inheritance {
        let (subtotal, calculations, papacy_id) = self.inherit_subtotal(country);

        let year = i64::from(self.save().meta.date.year());
        let t_value = (subtotal + year) % 100;

        let t0_mod = (0 - t_value) % 100;
        let t1_mod = (75 - t_value) % 100;
        let t2_mod = (80 - t_value) % 100;

        // end date < year => +100
        // end date > year + 100 => -100
        let t0_offset = if year + t0_mod + 74 < year { 100 } else { 0 };

        let t1_offset = if year + t1_mod + 4 < year { 100 } else { 0 };

        let t2_offset = if year + t2_mod + 19 < year { 100 } else { 0 };

        let start_t0_year = year + t0_mod + t0_offset;
        let end_t0_year = year + t0_mod + t0_offset + 74;
        let start_t1_year = year + t1_mod + t1_offset;
        let end_t1_year = year + t1_mod + t1_offset + 4;
        let start_t2_year = year + t2_mod + t2_offset;
        let end_t2_year = year + t2_mod + t2_offset + 19;

        let is_catholic = country
            .country
            .religion
            .as_ref()
            .map_or(false, |x| x == "catholic");
        let heretic_offset = i64::from(is_catholic) * papacy_id;
        let inheritance_value = (subtotal + year - heretic_offset) % 100;

        Inheritance {
            start_t0_year: start_t0_year as i16,
            end_t0_year: end_t0_year as i16,
            start_t1_year: start_t1_year as i16,
            end_t1_year: end_t1_year as i16,
            start_t2_year: start_t2_year as i16,
            end_t2_year: end_t2_year as i16,
            t_value: t_value as u8,
            inheritance_value: inheritance_value as u8,
            heretic_offset: heretic_offset as i32,
            subtotal,
            calculations,
        }
    }

    pub fn resolved_war_participants(
        &self,
        tag_resolver: &TagResolver,
    ) -> Vec<ResolvedWarParticipants> {
        war_participants(&self.save, tag_resolver)
    }

    pub fn income_statistics_ledger(&self, nation: &NationEvents) -> Vec<LedgerPoint> {
        self.nation_ledger(nation, &self.save.game.income_statistics, |x| x / 12)
    }

    pub fn inflation_statistics_ledger(&self, nation: &NationEvents) -> Vec<LedgerPoint> {
        self.nation_ledger(nation, &self.save.game.inflation_statistics, |x| x)
    }

    pub fn score_statistics_ledger(&self, nation: &NationEvents) -> Vec<LedgerPoint> {
        self.nation_ledger(nation, &self.save.game.score_statistics, |x| x)
    }

    pub fn nation_size_statistics_ledger(&self, nation: &NationEvents) -> Vec<LedgerPoint> {
        self.nation_ledger(nation, &self.save.game.nation_size_statistics, |x| x)
    }

    fn nation_ledger<F: Fn(i32) -> i32>(
        &self,
        nation: &NationEvents,
        ledger: &LedgerData,
        f: F,
    ) -> Vec<LedgerPoint> {
        let time_range = self.save.game.start_date.days_until(&self.save.meta.date) / 365 + 1;
        let mut result = Vec::with_capacity(time_range as usize);

        #[derive(Debug)]
        struct NationChain {
            tag: CountryTag,
            start: Eu4Date,
            end: Eu4Date,
        }

        let mut chains: Vec<NationChain> = Vec::new();
        let mut current_tag = nation.initial;
        let mut start = self.save.game.start_date;
        let mut annexed = false;
        for event in &nation.events {
            match event.kind {
                NationEventKind::Annexed => {
                    chains.push(NationChain {
                        tag: current_tag,
                        start,
                        end: event.date,
                    });
                    annexed = true;
                }
                NationEventKind::Appeared => {
                    start = event.date;
                    annexed = false;
                }
                NationEventKind::TagSwitch(c) => {
                    chains.push(NationChain {
                        tag: current_tag,
                        start,
                        end: event.date,
                    });
                    annexed = false;
                    start = event.date;
                    current_tag = c;
                }
            }
        }

        if !annexed {
            chains.push(NationChain {
                tag: current_tag,
                start,
                end: self.save.meta.date,
            })
        }

        let ledger_chain = chains.iter().filter_map(|chain| {
            ledger
                .ledger
                .iter()
                .find(|datum| datum.name == chain.tag)
                .map(|ledger| (chain, ledger))
        });

        for (chain, ledger) in ledger_chain {
            let data = ledger
                .data
                .iter()
                .skip_while(|(year, _)| (*year as i16) < chain.start.year())
                .take_while(|(year, _)| (*year as i16) <= chain.end.year());

            let mut current = (chain.start.year() + 1) as u16;
            for &(x, y) in data {
                for year in current..x {
                    result.push(LedgerPoint {
                        tag: chain.tag,
                        year,
                        value: 0,
                    });
                }

                result.push(LedgerPoint {
                    tag: chain.tag,
                    year: x,
                    value: f(y),
                });
                current = x + 1;
            }

            for year in current..(chain.end.year() as u16) + 1 {
                result.push(LedgerPoint {
                    tag: chain.tag,
                    year,
                    value: 0,
                });
            }
        }

        result
    }

    pub fn country_tag_hex_color(&self, country_tag: &CountryTag) -> Option<String> {
        self.country(country_tag)
            .map(|x| self.country_color_to_hex(x))
    }

    pub fn country_color_to_hex(&self, country: &Country) -> String {
        let colors = &country.colors.country_color;
        format!("#{:02x}{:02x}{:02x}", colors[0], colors[1], colors[2])
    }

    pub fn country_income_breakdown(&self, country: &Country) -> CountryIncomeLedger {
        let ledger = &country.ledger.lastmonthincometable;
        CountryIncomeLedger {
            taxation: *ledger.first().unwrap_or(&0.0),
            production: *ledger.get(1).unwrap_or(&0.0),
            trade: *ledger.get(2).unwrap_or(&0.0),
            gold: *ledger.get(3).unwrap_or(&0.0),
            tariffs: *ledger.get(4).unwrap_or(&0.0),
            vassals: *ledger.get(5).unwrap_or(&0.0),
            harbor_fees: *ledger.get(6).unwrap_or(&0.0),
            subsidies: *ledger.get(7).unwrap_or(&0.0),
            war_reparations: *ledger.get(8).unwrap_or(&0.0),
            interest: *ledger.get(9).unwrap_or(&0.0),
            gifts: *ledger.get(10).unwrap_or(&0.0),
            events: *ledger.get(11).unwrap_or(&0.0),
            spoils_of_war: *ledger.get(12).unwrap_or(&0.0),
            treasure_fleet: *ledger.get(13).unwrap_or(&0.0),
            siphoning_income: *ledger.get(14).unwrap_or(&0.0),
            condottieri: *ledger.get(15).unwrap_or(&0.0),
            knowledge_sharing: *ledger.get(16).unwrap_or(&0.0),
            blockading_foreign_ports: *ledger.get(17).unwrap_or(&0.0),
            looting_foreign_cities: *ledger.get(18).unwrap_or(&0.0),
            other: ledger.get(19..).iter().flat_map(|x| x.iter()).sum(),
        }
    }

    pub fn countries_income_breakdown(&self) -> HashMap<CountryTag, CountryIncomeLedger> {
        self.save
            .game
            .countries
            .iter()
            .filter(|(_, country)| country.num_of_cities > 0)
            .map(|(tag, country)| (*tag, self.country_income_breakdown(country)))
            .collect()
    }

    pub fn countries_expense_breakdown(&self) -> HashMap<CountryTag, CountryExpenseLedger> {
        self.save
            .game
            .countries
            .iter()
            .filter(|(_, country)| country.num_of_cities > 0)
            .map(|(tag, country)| (*tag, self.country_expense_breakdown(country)))
            .collect()
    }

    pub fn countries_total_expense_breakdown(&self) -> HashMap<CountryTag, CountryExpenseLedger> {
        self.save
            .game
            .countries
            .iter()
            .filter(|(_, c)| c.ledger.totalexpensetable.iter().any(|&x| x > 0.0))
            .map(|(tag, country)| (*tag, self.country_total_expense_breakdown(country)))
            .collect()
    }

    fn expense_ledger_breakdown(&self, ledger: &[f32]) -> CountryExpenseLedger {
        CountryExpenseLedger {
            advisor_maintenance: *ledger.first().unwrap_or(&0.0),
            interest: *ledger.get(1).unwrap_or(&0.0),
            state_maintenance: *ledger.get(2).unwrap_or(&0.0),
            subsidies: *ledger.get(4).unwrap_or(&0.0),
            war_reparations: *ledger.get(5).unwrap_or(&0.0),
            army_maintenance: *ledger.get(6).unwrap_or(&0.0),
            fleet_maintenance: *ledger.get(7).unwrap_or(&0.0),
            fort_maintenance: *ledger.get(8).unwrap_or(&0.0),
            colonists: *ledger.get(9).unwrap_or(&0.0),
            missionaries: *ledger.get(10).unwrap_or(&0.0),
            raising_armies: *ledger.get(11).unwrap_or(&0.0),
            building_fleets: *ledger.get(12).unwrap_or(&0.0),
            building_fortresses: *ledger.get(13).unwrap_or(&0.0),
            buildings: *ledger.get(14).unwrap_or(&0.0),
            repaid_loans: *ledger.get(16).unwrap_or(&0.0),
            gifts: *ledger.get(17).unwrap_or(&0.0),
            advisors: *ledger.get(18).unwrap_or(&0.0),
            events: *ledger.get(19).unwrap_or(&0.0),
            peace: *ledger.get(20).unwrap_or(&0.0),
            vassal_fee: *ledger.get(21).unwrap_or(&0.0),
            tariffs: *ledger.get(22).unwrap_or(&0.0),
            support_loyalists: *ledger.get(23).unwrap_or(&0.0),
            condottieri: *ledger.get(26).unwrap_or(&0.0),
            root_out_corruption: *ledger.get(27).unwrap_or(&0.0),
            embrace_institution: *ledger.get(28).unwrap_or(&0.0),
            knowledge_sharing: *ledger.get(30).unwrap_or(&0.0),
            trade_company_investments: *ledger.get(31).unwrap_or(&0.0),
            ports_blockaded: *ledger.get(33).unwrap_or(&0.0),
            cities_looted: *ledger.get(34).unwrap_or(&0.0),
            monuments: *ledger.get(35).unwrap_or(&0.0),
            cot_upgrades: *ledger.get(36).unwrap_or(&0.0),
            colony_changes: *ledger.get(37).unwrap_or(&0.0),
            other: *ledger.get(3).unwrap_or(&0.0)
                + *ledger.get(15).unwrap_or(&0.0)
                + *ledger.get(24).unwrap_or(&0.0)
                + *ledger.get(25).unwrap_or(&0.0)
                + *ledger.get(29).unwrap_or(&0.0)
                + *ledger.get(32).unwrap_or(&0.0)
                + ledger.get(38..).iter().flat_map(|x| x.iter()).sum::<f32>(),
        }
    }

    pub fn country_expense_breakdown(&self, country: &Country) -> CountryExpenseLedger {
        self.expense_ledger_breakdown(&country.ledger.lastmonthexpensetable)
    }

    pub fn country_total_expense_breakdown(&self, country: &Country) -> CountryExpenseLedger {
        self.expense_ledger_breakdown(&country.ledger.totalexpensetable)
    }

    fn mana_spent_indexed(&self, data: &[(i32, i32)]) -> CountryManaSpend {
        let offset = i32::from(self.save().meta.savegame_version.second >= 31);
        let force_march = find_index(8, data) + find_index(45, data);
        CountryManaSpend {
            buy_idea: find_index(0, data),
            advance_tech: find_index(1, data),
            boost_stab: find_index(2, data),
            buy_general: find_index(3, data),
            buy_admiral: find_index(4, data),
            buy_conq: find_index(5, data),
            buy_explorer: find_index(6, data),
            develop_prov: find_index(7, data),
            force_march,
            assault: find_index(9, data),
            seize_colony: find_index(10, data),
            burn_colony: find_index(11, data),
            attack_natives: find_index(12, data),
            scorch_earth: find_index(13, data),
            demand_non_wargoal_prov: find_index(14, data),
            reduce_inflation: find_index(15, data),
            move_capital: find_index(16, data),
            make_province_core: find_index(17, data),
            replace_rival: find_index(18, data),
            change_gov: find_index(19, data),
            change_culture: find_index(20, data),
            harsh_treatment: find_index(21, data),
            reduce_we: find_index(22, data),
            boost_faction: find_index(23, data),
            raise_war_taxes: find_index(24, data),
            buy_native_advancement: if offset != 0 { 0 } else { find_index(25, data) },
            increse_tariffs: find_index(26 - offset, data),
            promote_merc: find_index(27 - offset, data),
            decrease_tariffs: find_index(28 - offset, data),
            move_trade_port: find_index(29 - offset, data),
            create_trade_post: find_index(30 - offset, data),
            siege_sorties: find_index(31 - offset, data),
            buy_religious_reform: find_index(32 - offset, data),
            set_primary_culture: find_index(33 - offset, data),
            add_accepted_culture: find_index(34 - offset, data),
            remove_accepted_culture: find_index(35 - offset, data),
            strengthen_government: find_index(36 - offset, data),
            boost_militarization: find_index(37 - offset, data),
            artillery_barrage: find_index(39 - offset, data),
            establish_siberian_frontier: find_index(40 - offset, data),
            government_interaction: find_index(41 - offset, data),
            naval_barrage: find_index(43 - offset, data),
            add_tribal_land: if offset != 1 {
                0
            } else {
                find_index(44 - offset, data)
            },
            create_leader: find_index(46, data),
            enforce_culture: find_index(47, data),
            effect: find_index(48, data),
            minority_expulsion: find_index(49, data),
            other: find_index(38 - offset, data)
                + find_index(42 - offset, data)
                + find_index(44, data)
                + data
                    .iter()
                    .filter(|(ind, _)| *ind > 49)
                    .map(|(_, val)| val)
                    .sum::<i32>(),
        }
    }

    pub fn country_mana_breakdown(&self, country: &Country) -> CountryManaUsage {
        CountryManaUsage {
            adm: self.mana_spent_indexed(&country.adm_spent_indexed),
            dip: self.mana_spent_indexed(&country.dip_spent_indexed),
            mil: self.mana_spent_indexed(&country.mil_spent_indexed),
        }
    }

    /// Return all unique buildings in the world that are built
    pub fn built_buildings(&self) -> &HashSet<String> {
        self.buildings.get_or_init(|| {
            self.save
                .game
                .provinces
                .values()
                .flat_map(|x| x.buildings.keys())
                .cloned()
                .collect()
        })
    }

    pub fn province_building_history<'a>(&'a self, province: &'a Province) -> Vec<BuildingEvent> {
        let buildings = self.built_buildings();
        let initial_buildings = province.history.other.iter().filter_map(|(key, _event)| {
            if buildings.contains(key) {
                Some(BuildingEvent {
                    building: key.as_str(),
                    date: self.save.game.start_date,
                    action: BuildingConstruction::Constructed,
                })
            } else {
                None
            }
        });

        let over_time = province.history.events.iter().flat_map(|(date, events)| {
            events.0.iter().filter_map(move |event| match event {
                ProvinceEvent::KV((key, value)) => {
                    let constructed = if let ProvinceEventValue::Bool(x) = value {
                        if buildings.contains(key) {
                            if *x {
                                BuildingConstruction::Constructed
                            } else {
                                BuildingConstruction::Destroyed
                            }
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    };

                    Some(BuildingEvent {
                        building: key.as_str(),
                        date: *date,
                        action: constructed,
                    })
                }
                _ => None,
            })
        });

        initial_buildings.chain(over_time).collect()
    }
}

fn find_index(index: i32, data: &[(i32, i32)]) -> i32 {
    data.iter()
        .find(|&(ind, _)| *ind == index)
        .map(|(_, val)| *val)
        .unwrap_or(0)
}

fn nation_events(save: &Eu4Save, province_owners: &ProvinceOwners) -> Vec<NationEvents> {
    struct CountryTagSwitchFrom {
        date: Eu4Date,
        from: CountryTag,
    }

    #[derive(Debug, PartialEq, Clone, Serialize)]
    struct CountryTagSwitch {
        date: Eu4Date,
        from: CountryTag,
        to: CountryTag,
        stored: CountryTag,
    }

    let mut nation_events = HashMap::with_capacity(save.game.countries.len());
    let mut all_switches = Vec::with_capacity(save.game.countries.len());
    let mut initial_to_stored = HashMap::with_capacity(save.game.countries.len());
    for (tag, country) in &save.game.countries {
        let tag = *tag;
        let mut country_tag_switches = Vec::new();

        for (date, events) in &country.history.events {
            for event in &events.0 {
                if let CountryEvent::ChangedTagFrom(from) = *event {
                    country_tag_switches.push(CountryTagSwitchFrom { date: *date, from });
                }
            }
        }

        let initial_tag = country_tag_switches
            .first()
            .map(|x| x.from)
            .or_else(|| country.previous_country_tags.first().cloned())
            .unwrap_or(tag);

        let latest = country
            .previous_country_tags
            .get(country_tag_switches.len())
            .cloned()
            .unwrap_or(tag);

        let tos = country_tag_switches
            .iter()
            .map(|x| x.from)
            .skip(1)
            .chain(std::iter::once(latest));

        let switches = country_tag_switches
            .iter()
            .zip(tos)
            .map(|(from, to)| CountryTagSwitch {
                date: from.date,
                stored: tag,
                from: from.from,
                to,
            })
            .collect::<Vec<_>>();

        all_switches.extend(switches.clone());

        if !switches.is_empty() || tag != latest || latest != initial_tag {
            let sws = nation_events.entry(tag).or_insert_with(Vec::new);
            let a = switches.iter().map(|x| NationEvent {
                date: x.date,
                kind: NationEventKind::TagSwitch(x.to),
            });
            sws.extend(a);
        }

        initial_to_stored.insert(initial_tag, tag);
    }

    let mut counts: HashMap<CountryTag, i32> = HashMap::with_capacity(save.game.countries.len());
    let mut owners: Vec<Option<CountryTag>> = vec![None; save.game.provinces.len() + 1];
    let initial_owners = province_owners
        .initial
        .iter()
        .enumerate()
        .filter_map(|(i, x)| x.map(|y| (i, y)));

    for (id, tag) in initial_owners {
        if let Some(&stored) = initial_to_stored.get(&tag) {
            owners[id] = Some(stored);
            *counts.entry(stored).or_insert(0) += 1;
        } else {
            debug_assert!(false)
        }
    }

    let mut tag_dater: HashMap<_, Vec<_>> = HashMap::new();
    for x in all_switches {
        tag_dater.entry(x.from).or_insert_with(Vec::new).push(x);
    }
    for x in tag_dater.values_mut() {
        x.sort_by_key(|x| x.date);
    }

    for change in &province_owners.changes {
        let store = tag_dater
            .get(&change.tag)
            .and_then(|x| x.iter().find(|x| x.date >= change.date))
            .map(|x| x.stored)
            .unwrap_or(change.tag);

        let prov_id = usize::from(change.province.as_u16());
        if let Some(old) = owners[prov_id].replace(store) {
            if let Some(count) = counts.get_mut(&old) {
                // There is a mod where the count dips below 0. I haven't seen it
                // dip below zero for a vanilla save, but we don't want to underflow
                // on any input so we bottom it out at zero.
                *count = std::cmp::max(0, *count - 1);
                if *count == 0 {
                    nation_events
                        .entry(old)
                        .or_insert_with(Vec::new)
                        .push(NationEvent {
                            date: change.date,
                            kind: NationEventKind::Annexed,
                        })
                }
            } else {
                debug_assert!(false, "tag of {} is not counted for", old);
            }
        }

        let new_count = counts.entry(store).or_insert(0);
        if *new_count == 0 && change.date > save.game.start_date {
            nation_events
                .entry(store)
                .or_insert_with(Vec::new)
                .push(NationEvent {
                    date: change.date,
                    kind: NationEventKind::Appeared,
                })
        }
        *new_count += 1;
    }

    for events in nation_events.values_mut() {
        events.sort_by_key(|x| x.date);
    }

    let mut result = Vec::with_capacity(save.game.countries.len());
    for (initial, stored) in initial_to_stored {
        let events = nation_events.remove(&stored).unwrap_or_default();
        let latest = events
            .iter()
            .filter_map(|x| match x.kind {
                NationEventKind::TagSwitch(t) => Some(t),
                _ => None,
            })
            .last()
            .unwrap_or(initial);

        // If a stored tag never owned a province in the entire playthrough, exclude it
        if counts.contains_key(&stored) {
            result.push(NationEvents {
                initial,
                latest,
                stored,
                events,
            });
        }
    }

    result
}

fn war_participants(save: &Eu4Save, tag_resolver: &TagResolver) -> Vec<ResolvedWarParticipants> {
    let active = save.game.active_wars.iter().map(|x| (&x.name, &x.history));
    let previous = save
        .game
        .previous_wars
        .iter()
        .map(|x| (&x.name, &x.history));
    let wars = active.chain(previous);

    let mut war_participants =
        Vec::with_capacity(save.game.active_wars.len() + save.game.previous_wars.len());
    for (name, war) in wars {
        let mut tags = Vec::new();
        for (date, events) in &war.events {
            for event in &events.0 {
                match event {
                    WarEvent::AddAttacker(x) | WarEvent::AddDefender(x) => {
                        let stored_tag = tag_resolver.resolve(*x, *date);
                        tags.push(ResolvedWarParticipant {
                            tag: *x,
                            stored: stored_tag.map(|x| x.stored).unwrap_or(*x),
                        });
                    }
                    _ => {}
                }
            }
        }

        war_participants.push(ResolvedWarParticipants {
            war: name.clone(),
            participants: tags,
        })
    }

    war_participants
}

fn province_owners(save: &Eu4Save) -> ProvinceOwners {
    let mut initial = vec![None; save.game.provinces.len() + 1];
    let mut owners = vec![None; save.game.provinces.len() + 1];
    let mut changes = Vec::with_capacity(save.game.provinces.len() * 2);
    for (&id, province) in &save.game.provinces {
        initial[usize::from(id.as_u16())] = province.history.owner;
        owners[usize::from(id.as_u16())] = province.history.owner;

        for (date, events) in &province.history.events {
            for event in &events.0 {
                if let ProvinceEvent::Owner(new_owner) = *event {
                    // Check to make sure the province really changed hands. Exclude
                    // change owner events if the owner didn't change hands.
                    // In the trycone save, Leinster is listed as the new owner of
                    // Laighin in 1444.11.12 even though they already owned it
                    let prov_id = usize::from(id.as_u16());
                    let old_owner = owners[prov_id].replace(new_owner);
                    if old_owner.map_or(true, |x| new_owner != x) {
                        changes.push(ProvinceOwnerChange {
                            date: *date,
                            province: id,
                            tag: new_owner,
                        });
                    }
                }
            }
        }
    }

    // Keep the sort stable so that when we come across a province history like
    // below, we keep "BUL" after "NAP", which could signify NAP conquering a
    // province and then immediately releasing bulgaria.
    // ```
    // 1477.2.27={
    //   owner="NAP"
    // }
    // 1477.2.27={
    //   owner="BUL"
    // }
    // ```
    changes.sort_by_key(|x| (x.date, x.province));

    ProvinceOwners { initial, changes }
}

fn players(save: &Eu4Save) -> Vec<Player> {
    let mut players = Vec::new();
    for entry in save.game.players_countries.chunks_exact(2) {
        let player_name = &entry[0];
        if player_name == "Player" && !save.meta.multiplayer {
            continue;
        }

        let country_tag = match entry[1].parse::<CountryTag>() {
            Ok(x) => x,
            _ => continue,
        };

        players.push(Player {
            name: player_name.clone(),
            tag: country_tag,
        })
    }

    players
}

fn province_religions(save: &Eu4Save, lookup: &ReligionLookup) -> ProvinceReligions {
    let mut initial = vec![None; save.game.provinces.len() + 1];
    let mut religions = vec![None; save.game.provinces.len() + 1];
    let mut changes = Vec::with_capacity(save.game.provinces.len());
    for (&id, province) in &save.game.provinces {
        let prov_id = usize::from(id.as_u16());
        let init = province
            .history
            .religion
            .as_ref()
            .and_then(|x| lookup.index(x));

        initial[prov_id] = init;
        religions[prov_id] = init;

        for (date, events) in &province.history.events {
            for event in &events.0 {
                if let ProvinceEvent::Religion(new_religion) = event {
                    if let Some(new_religion_index) = lookup.index(new_religion) {
                        let old_religion = religions[prov_id].replace(new_religion_index);
                        if old_religion.map_or(true, |x| new_religion_index != x) {
                            changes.push(ProvinceReligionChange {
                                date: *date,
                                province: id,
                                religion: new_religion_index,
                            });
                        }
                    }
                }
            }
        }
    }

    changes.sort_by_key(|x| (x.date, x.province));

    ProvinceReligions { initial, changes }
}

impl ReligionLookup {
    pub fn index(&self, religion: &String) -> Option<ReligionIndex> {
        self.religions
            .binary_search(religion)
            .ok()
            .and_then(|x| u16::try_from(x).ok())
            .map(|x| x + 1)
            .and_then(NonZeroU16::new)
            .map(ReligionIndex)
    }

    pub fn resolve(&self, index: ReligionIndex) -> &str {
        self.religions[usize::from(index.0.get() - 1)].as_str()
    }
}
