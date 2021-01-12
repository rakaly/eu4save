use crate::models::{
    Country, CountryEvent, Eu4Save, LedgerData, LedgerDatum, Province, ProvinceEvent,
    ProvinceEventValue,
};
use crate::{CountryTag, Eu4Date};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy)]
pub enum CountryQuery {
    Players,
    Greats,
    //    PastGreat
    //    GreatPowersAndPlayers,
    //    AllGreatPowersAndPlayers,
    //    All,
}

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
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum BuildingConstruction {
    Constructed,
    Destroyed,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BuildingEvent<'a> {
    pub building: &'a str,
    pub date: Eu4Date,
    pub action: BuildingConstruction,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CountryPlayed {
    pub tag: CountryTag,
    pub start: Eu4Date,
    pub end: Eu4Date,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PlayerHistory {
    pub tag: CountryTag,

    /// Whether the player is currently in the session
    pub is_human: bool,

    /// Names of the players (may be empty)
    pub player_names: Vec<String>,

    /// Whether the country owns any provinces
    pub exists: bool,

    /// A history of tag switches for said country
    pub played_tags: Vec<CountryPlayed>,
}

#[derive(Debug)]
pub struct Query {
    save: Eu4Save,
    players: OnceCell<HashSet<String>>,
    player_histories: OnceCell<Vec<PlayerHistory>>,
    player_countries: OnceCell<HashSet<CountryTag>>,
    starting_country: OnceCell<Option<CountryTag>>,
    buildings: OnceCell<HashSet<String>>,
}

impl Query {
    pub fn from_save(save: Eu4Save) -> Self {
        Query {
            save,
            players: OnceCell::default(),
            player_histories: OnceCell::default(),
            player_countries: OnceCell::default(),
            starting_country: OnceCell::default(),
            buildings: OnceCell::default(),
        }
    }

    pub fn save(&self) -> &Eu4Save {
        &self.save
    }

    /// Returns a set of the names of the players who participated in a playthrough.
    /// May be blank for single player run through where the player is detected as
    /// "Player".
    pub fn player_names(&self) -> &HashSet<String> {
        self.players.get_or_init(|| {
            self.player_histories()
                .iter()
                .flat_map(|x| x.player_names.clone().into_iter())
                .collect()
        })
    }

    /// Provides a rich structure that contains the following:
    ///
    /// - A list countries that had a player at one point or another
    /// - What country the player is playing
    /// - Whether that player is currently in the session
    /// - The names of the players in the session
    /// - A list of all prior tags that country had been
    pub fn player_histories(&self) -> &[PlayerHistory] {
        self.player_histories
            .get_or_init(|| calc_histories(&self.save))
    }

    /// Returns all the tags that players touched in a playthrough. Don't rely on this
    /// for accurate calculations as countries form new countries that may have
    /// pre-existed (think of a cilli into croatia run) and relying on this would
    /// meddle up any calculations. Use `player_histories` instead.
    pub fn player_countries(&self) -> &HashSet<CountryTag> {
        self.player_countries.get_or_init(|| {
            self.player_histories()
                .iter()
                .filter(|x| x.is_human)
                .flat_map(|x| x.played_tags.iter().map(|x| x.tag.clone()))
                .collect()
        })
    }

    /// Return the starting country in single player playthroughs. If playing in multiplayer or if
    /// the starting country can't be determined then none is returned.
    pub fn starting_country(&self) -> Option<&CountryTag> {
        match self.player_histories() {
            [player] => player.played_tags.get(0).map(|x| &x.tag),
            _ => None,
        }
    }

    pub fn country_tag_hex_color(&self, country_tag: &CountryTag) -> Option<String> {
        self.save
            .game
            .countries
            .get(&country_tag)
            .map(|x| self.country_color_to_hex(x))
    }

    pub fn country_color_to_hex(&self, country: &Country) -> String {
        let colors = &country.colors.country_color;
        format!("#{:02x}{:02x}{:02x}", colors[0], colors[1], colors[2])
    }

    fn add_previous_countries(
        &self,
        tag: &CountryTag,
        country: &Country,
        mut set: &mut HashSet<CountryTag>,
    ) {
        set.insert(tag.clone());
        for prev_tag in &country.previous_country_tags {
            if set.contains(&prev_tag) {
                continue;
            }

            if let Some(prev_country) = self.save.game.countries.get(&prev_tag) {
                self.add_previous_countries(&prev_tag, prev_country, &mut set);
            }
        }
    }

    pub fn filter_countries<P: AsRef<str>>(
        &self,
        query: &[CountryQuery],
        includes: &[P],
        excludes: &[P],
    ) -> HashSet<CountryTag> {
        let mut filter_set = HashSet::new();
        let player_countries = self.player_countries();
        for q in query {
            match q {
                CountryQuery::Players => {
                    filter_set = filter_set.union(player_countries).cloned().collect();
                }
                CountryQuery::Greats => {
                    for (tag, country) in &self.save.game.countries {
                        if country.is_great_power {
                            self.add_previous_countries(&tag, &country, &mut filter_set);
                        }
                    }
                }
            }
        }

        for include in includes {
            filter_set.insert(CountryTag::from(include.as_ref()));
        }

        for exclude in excludes {
            filter_set.remove(&CountryTag::from(exclude.as_ref()));
        }

        filter_set
    }

    fn ledger_statistics<'a>(
        &self,
        filter_set: &HashSet<CountryTag>,
        data: &'a LedgerData,
    ) -> Vec<&'a LedgerDatum> {
        data.ledger
            .iter()
            .filter(|x| filter_set.contains(&x.name))
            .collect()
    }

    pub fn annual_ledgers<P: AsRef<str>>(
        &self,
        query: &[CountryQuery],
        includes: &[P],
        excludes: &[P],
    ) -> AnnualLedgers {
        let filter_set = self.filter_countries(&query, includes, excludes);
        let size = self.ledger_statistics(&filter_set, &self.save.game.nation_size_statistics);
        let score = self.ledger_statistics(&filter_set, &self.save.game.score_statistics);
        let inflation = self.ledger_statistics(&filter_set, &self.save.game.inflation_statistics);
        let income = self.ledger_statistics(&filter_set, &self.save.game.income_statistics);

        // Figure out the timeline of countries as the game doesn't write out ledger statistics for
        // those that report zero inflation, score, etc.
        let income_years: HashMap<CountryTag, HashSet<u16>> = income
            .iter()
            .map(|ledg| {
                (
                    ledg.name.clone(),
                    ledg.data.iter().map(|(x, _)| *x).collect(),
                )
            })
            .collect();

        let mut adjusted_scores: Vec<LedgerDatum> = score.into_iter().cloned().collect();
        for entry in &mut adjusted_scores {
            if let Some(years) = income_years.get(&entry.name) {
                let score_years: HashSet<u16> = entry.data.iter().map(|(x, _)| *x).collect();
                let diff = years - &score_years;
                entry.data.extend(diff.iter().map(|x| (*x, 0)));
            }
        }

        let mut adjusted_inflation: Vec<LedgerDatum> = inflation.into_iter().cloned().collect();
        for entry in &mut adjusted_inflation {
            if let Some(years) = income_years.get(&entry.name) {
                let inflation_years: HashSet<u16> = entry.data.iter().map(|(x, _)| *x).collect();
                let diff = years - &inflation_years;
                entry.data.extend(diff.iter().map(|x| (*x, 0)));
            }
        }

        AnnualLedgers {
            size: size.into_iter().cloned().collect(),
            income: income.into_iter().cloned().collect(),
            inflation: adjusted_inflation,
            score: adjusted_scores,
        }
    }

    /*        match query {
        CountryQuery::Players => self
            .save
            .game
            .income_statistics
            .ledger
            .iter()
            .filter(|x| self.player_countries.contains(x.name.as_str()))
            .collect(),
        CountryQuery::All => self.save.game.income_statistics.ledger.iter().collect(),
        CountryQuery::GreatPowersAndPlayers => {
            let great_tags: HashSet<_> = self
                .save
                .game
                .countries
                .iter()
                .filter_map(|(tag, country)| {
                    if country.is_great_power {
                        Some(tag.as_str())
                    } else {
                        None
                    }
                })
                .collect();

            self.save
                .game
                .income_statistics
                .ledger
                .iter()
                .filter(|x| {
                    self.player_countries.contains(x.name.as_str())
                        || great_tags.contains(x.name.as_str())
                })
                .collect()
        }
        CountryQuery::AllGreatPowersAndPlayers => {
            let great_tags: HashSet<_> = self
                .save
                .game
                .countries
                .iter()
                .filter_map(|(tag, country)| {
                    if country.is_great_power
                        || country
                            .flags
                            .iter()
                            .find(|&(flag, _)| flag == "became_great_power_flag")
                            .is_some()
                    {
                        Some(tag.as_str())
                    } else {
                        None
                    }
                })
                .collect();

            self.save
                .game
                .income_statistics
                .ledger
                .iter()
                .filter(|x| {
                    self.player_countries.contains(x.name.as_str())
                        || great_tags.contains(x.name.as_str())
                })
                .collect()
        }
    }*/

    pub fn country_income_breakdown(&self, country: &Country) -> CountryIncomeLedger {
        let ledger = &country.ledger.lastmonthincometable;
        CountryIncomeLedger {
            taxation: *ledger.get(0).unwrap_or(&0.0),
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
            .map(|(tag, country)| (tag.clone(), self.country_income_breakdown(country)))
            .collect()
    }

    pub fn countries_expense_breakdown(&self) -> HashMap<CountryTag, CountryExpenseLedger> {
        self.save
            .game
            .countries
            .iter()
            .filter(|(_, country)| country.num_of_cities > 0)
            .map(|(tag, country)| (tag.clone(), self.country_expense_breakdown(country)))
            .collect()
    }

    pub fn countries_total_expense_breakdown(&self) -> HashMap<CountryTag, CountryExpenseLedger> {
        self.save
            .game
            .countries
            .iter()
            .filter(|(_, c)| c.ledger.totalexpensetable.iter().any(|&x| x > 0.0))
            .map(|(tag, country)| (tag.clone(), self.country_total_expense_breakdown(country)))
            .collect()
    }

    fn expense_ledger_breakdown(&self, ledger: &[f32]) -> CountryExpenseLedger {
        CountryExpenseLedger {
            advisor_maintenance: *ledger.get(0).unwrap_or(&0.0),
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
            other: *ledger.get(3).unwrap_or(&0.0)
                + *ledger.get(15).unwrap_or(&0.0)
                + *ledger.get(24).unwrap_or(&0.0)
                + *ledger.get(25).unwrap_or(&0.0)
                + *ledger.get(29).unwrap_or(&0.0)
                + *ledger.get(32).unwrap_or(&0.0)
                + ledger.get(35..).iter().flat_map(|x| x.iter()).sum::<f32>(),
        }
    }

    pub fn country_expense_breakdown(&self, country: &Country) -> CountryExpenseLedger {
        self.expense_ledger_breakdown(&country.ledger.lastmonthexpensetable)
    }

    pub fn country_total_expense_breakdown(&self, country: &Country) -> CountryExpenseLedger {
        self.expense_ledger_breakdown(&country.ledger.totalexpensetable)
    }

    fn mana_spent_indexed(&self, data: &[(i32, i32)]) -> CountryManaSpend {
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
            buy_native_advancement: find_index(25, data),
            increse_tariffs: find_index(26, data),
            promote_merc: find_index(27, data),
            decrease_tariffs: find_index(28, data),
            move_trade_port: find_index(29, data),
            create_trade_post: find_index(30, data),
            siege_sorties: find_index(31, data),
            buy_religious_reform: find_index(32, data),
            set_primary_culture: find_index(33, data),
            add_accepted_culture: find_index(34, data),
            remove_accepted_culture: find_index(35, data),
            strengthen_government: find_index(36, data),
            boost_militarization: find_index(37, data),
            artillery_barrage: find_index(39, data),
            establish_siberian_frontier: find_index(40, data),
            government_interaction: find_index(41, data),
            naval_barrage: find_index(43, data),
            create_leader: find_index(46, data),
            enforce_culture: find_index(47, data),
            effect: find_index(48, data),
            minority_expulsion: find_index(49, data),
            other: find_index(38, data)
                + find_index(42, data)
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

        let over_time = province
            .history
            .events
            .iter()
            .map(|(date, events)| {
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
            })
            .flatten();

        initial_buildings.chain(over_time).collect()
    }
}

fn find_index(index: i32, data: &[(i32, i32)]) -> i32 {
    data.iter()
        .find(|&(ind, _)| *ind == index)
        .map(|(_, val)| *val)
        .unwrap_or(0)
}

fn calc_histories(save: &Eu4Save) -> Vec<PlayerHistory> {
    let mut result = Vec::with_capacity(save.game.players_countries.len());

    let mut latest_ownership = HashMap::with_capacity(save.game.countries.len());
    for prov in save.game.provinces.values() {
        let mut old_owner = None;
        if let Some(initial) = prov.history.owner.as_ref() {
            latest_ownership
                .entry(initial)
                .and_modify(|d| *d = std::cmp::max(*d, save.game.start_date))
                .or_insert(save.game.start_date);
            old_owner = Some(initial);
        }

        for (date, events) in &prov.history.events {
            for event in &events.0 {
                if let ProvinceEvent::Owner(new_owner) = event {
                    if let Some(old) = old_owner.replace(new_owner) {
                        latest_ownership
                            .entry(old)
                            .and_modify(|d| *d = std::cmp::max(*d, *date))
                            .or_insert(*date);
                    }
                }
            }
        }
    }

    for (tag, country) in save.game.countries.iter().filter(|(_, c)| c.was_player) {
        let exists = country.num_of_cities != 0;
        let end_date = if exists {
            &save.meta.date
        } else {
            latest_ownership.get(tag).unwrap_or(&save.meta.date)
        };

        let mut players = Vec::new();
        for entry in save.game.players_countries.chunks_exact(2) {
            let player_name = &entry[0];
            if player_name == "Player" && !save.meta.multiplayer {
                continue;
            }

            let country_tag = &entry[1];

            if country_tag == tag.as_str() {
                players.push(player_name.clone())
            }
        }

        let played_tags = trace_country(save, &tag, country, *end_date);
        result.push(PlayerHistory {
            tag: tag.clone(),
            is_human: country.human,
            player_names: players,
            exists,
            played_tags,
        });
    }

    result
}

fn trace_country(
    save: &Eu4Save,
    tag: &CountryTag,
    country: &Country,
    end: Eu4Date,
) -> Vec<CountryPlayed> {
    let mut played = Vec::new();
    let history = country
        .history
        .events
        .iter()
        .rev()
        .skip_while(|(d, _)| d > &end);
    let mut current_tag = tag;
    let mut current_date = end;
    for (date, events) in history {
        for event in events.0.iter().rev() {
            if let CountryEvent::ChangedTagFrom(prev) = event {
                played.push(CountryPlayed {
                    tag: current_tag.clone(),
                    start: *date,
                    end: current_date,
                });
                current_date = *date;
                current_tag = prev;
            }
        }
    }

    played.push(CountryPlayed {
        tag: current_tag.clone(),
        start: save.game.start_date,
        end: current_date,
    });

    played.reverse();
    played
}
