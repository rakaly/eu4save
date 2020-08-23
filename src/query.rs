use crate::{
    Country, CountryEvent, CountryTag, Eu4Date, Eu4Save, LedgerData, LedgerDatum, Province,
    ProvinceEvent,
};
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

#[derive(Debug)]
pub struct Query {
    pub save: Eu4Save,
    pub players: HashSet<String>,
    pub player_countries: HashSet<CountryTag>,
    pub starting_country: Option<CountryTag>,
}

impl Query {
    pub fn from_save(save: Eu4Save) -> Self {
        let players = calc_players(&save);
        let player_countries = calc_player_countries(&save);
        let starting_country = calc_starting_country(&save);
        Query {
            save,
            player_countries,
            players,
            starting_country,
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
        for q in query {
            match q {
                CountryQuery::Players => {
                    filter_set = filter_set.union(&self.player_countries).cloned().collect();
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

    pub fn province_building_history<'a>(
        &'a self,
        province: &'a Province,
    ) -> HashMap<&'a str, Eu4Date> {
        let buildings = &province.buildings;
        province
            .history
            .events
            .iter()
            .map(|(date, events)| {
                events.0.iter().filter_map(move |event| match event {
                    ProvinceEvent::KV((key, _value)) if buildings.contains_key(key) => {
                        Some((key.as_str(), date.clone()))
                    }
                    _ => None,
                })
            })
            .flatten()
            .collect()
    }
}

fn find_index(index: i32, data: &[(i32, i32)]) -> i32 {
    data.iter()
        .find(|&(ind, _)| *ind == index)
        .map(|(_, val)| *val)
        .unwrap_or(0)
}

fn calc_starting_country(save: &Eu4Save) -> Option<CountryTag> {
    if save.meta.multiplayer {
        return None;
    }

    let first = save
        .game
        .countries
        .iter()
        .find(|(_, country)| country.human);

    if let Some((first_tag, _country)) = first {
        let mut track_date = save.meta.date.clone();
        let mut tag = first_tag.clone();

        while let Some(country) = save.game.countries.get(first_tag) {
            let mut country_changes = country
                .history
                .events
                .iter()
                .rev()
                .filter(|(date, _events)| date < &track_date)
                .flat_map(|(date, events)| events.0.iter().map(move |event| (date.clone(), event)))
                .filter_map(|(date, event)| match event {
                    CountryEvent::ChangedTagFrom(x) => Some((date, x)),
                    _ => None,
                });

            if let Some((date, change_tag)) = country_changes.next() {
                track_date = date.clone();
                tag = change_tag.clone();
            } else {
                break;
            }
        }

        Some(tag)
    } else {
        None
    }
}

fn calc_players(save: &Eu4Save) -> HashSet<String> {
    save.game
        .players_countries
        .iter()
        .enumerate()
        .filter_map(|(i, x)| if i % 2 == 0 { Some(x) } else { None })
        .filter(|x| x.as_str() != "Player")
        .map(|x| x.to_string())
        .collect()
}

fn calc_player_countries(save: &Eu4Save) -> HashSet<CountryTag> {
    let mut player_countries: HashSet<_> = save
        .game
        .players_countries
        .iter()
        .enumerate()
        .filter_map(|(i, x)| if i % 2 == 1 { Some(x) } else { None })
        .map(|x| CountryTag::from(x.to_string()))
        .collect();

    loop {
        let size = player_countries.len();
        for (tag, country) in &save.game.countries {
            if country.was_player {
                player_countries.insert(tag.clone());
            }

            if player_countries.contains(&tag) {
                for prev_tag in &country.previous_country_tags {
                    player_countries.insert(prev_tag.clone());
                }
            }
        }

        if size == player_countries.len() {
            break;
        }
    }

    player_countries
}
