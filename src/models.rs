use crate::de::*;
use crate::{CountryTag, Eu4Date, ProvinceId};
use jomini::JominiDeserialize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Eu4SaveMeta {
    #[serde(flatten)]
    pub meta: Meta,

    #[serde(flatten)]
    pub game: Option<GameState>,
}

#[derive(Debug, Clone, JominiDeserialize, Serialize)]
pub struct Meta {
    pub campaign_id: String,
    pub save_game: String,
    pub player: CountryTag,
    pub displayed_country_name: String,
    pub campaign_length: i32,
    pub date: Eu4Date,
    #[jomini(default)]
    pub is_ironman: bool,
    #[jomini(default, alias = "multi_player")]
    #[serde(alias = "multi_player")]
    pub multiplayer: bool,
    pub not_observer: bool,
    #[jomini(default)]
    pub dlc_enabled: Vec<String>,
    #[jomini(default)]
    pub mod_enabled: Vec<String>,
    #[jomini(default)]
    pub mods_enabled_names: Vec<ModName>,
    #[jomini(take_last)]
    pub checksum: String,
    pub savegame_version: SavegameVersion,
    #[jomini(default)]
    pub is_random_new_world: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModName {
    pub filename: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Eu4Save {
    #[serde(flatten)]
    pub meta: Meta,

    #[serde(flatten)]
    pub game: GameState,
}

#[derive(Debug, Clone, JominiDeserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct GameState {
    #[jomini(default)]
    pub players_countries: Vec<String>,
    pub current_age: String,
    pub start_date: Eu4Date,
    pub map_area_data: HashMap<String, MapAreaDatum>,
    pub military_hegemon: Option<Hegemon>,
    pub naval_hegemon: Option<Hegemon>,
    pub economic_hegemon: Option<Hegemon>,
    pub religion_instance_data: HashMap<String, ReligionInstanceDatum>,
    pub empire: Option<HRE>,
    pub countries: HashMap<CountryTag, Country>,
    pub provinces: HashMap<ProvinceId, Province>,
    pub income_statistics: LedgerData,
    pub nation_size_statistics: LedgerData,
    pub score_statistics: LedgerData,
    pub inflation_statistics: LedgerData,
    #[jomini(duplicated, alias = "active_war")]
    pub active_wars: Vec<ActiveWar>,
    #[jomini(duplicated, alias = "previous_war")]
    pub previous_wars: Vec<PreviousWar>,
    #[jomini(default)]
    pub achievement_ok: bool,
    #[jomini(default)]
    pub achievement: Vec<i32>,
    #[jomini(default)]
    pub completed_achievements: Vec<i32>,
    #[jomini(alias = "gameplaysettings")]
    pub gameplay_settings: GameplaySettings,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SavegameVersion {
    pub first: u16,
    pub second: u16,
    pub third: u16,
    #[serde(alias = "forth")]
    pub fourth: u16,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct GameplaySettings {
    #[serde(alias = "setgameplayoptions")]
    pub options: GameplayOptions,
}

#[derive(Debug, Clone, JominiDeserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct MapAreaDatum {
    pub state: Option<MapAreaState>,
    #[jomini(default, duplicated, alias = "investments")]
    pub investments: Vec<TradeCompanyInvestment>,
}

#[derive(Debug, Clone, JominiDeserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct MapAreaState {
    pub area: String,
    #[jomini(duplicated, alias = "country_state")]
    pub country_states: Vec<CountryState>,
}

#[derive(Debug, Clone, JominiDeserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct TradeCompanyInvestment {
    pub tag: CountryTag,
    #[jomini(default)]
    pub investments: Vec<String>,
}

#[derive(Debug, Clone, JominiDeserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct CountryState {
    pub prosperity: f32,
    pub country: CountryTag,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Hegemon {
    pub country: CountryTag,
    pub progress: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct ReligionInstanceDatum {
    #[serde(default)]
    pub defender: Option<CountryTag>,
    #[serde(default)]
    pub defender_date: Option<Eu4Date>,
}

#[derive(Debug, Clone, JominiDeserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct HRE {
    pub emperor: Option<CountryTag>,
    #[jomini(duplicated, alias = "passed_reform")]
    pub passed_reforms: Vec<String>,
    #[jomini(default)]
    pub electors: Vec<CountryTag>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub enum TaxManpowerModifier {
    Historical,
    Random,
    Equal,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub enum GameDifficulty {
    VeryEasy,
    Easy,
    Normal,
    Hard,
    VeryHard,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct GameplayOptions {
    pub difficulty: GameDifficulty,
    pub tax_manpower_modifier: TaxManpowerModifier,
}

#[derive(Debug, Clone, JominiDeserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct LedgerData {
    #[jomini(duplicated, alias = "ledger_data")]
    pub ledger: Vec<LedgerDatum>,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct LedgerDatum {
    pub name: CountryTag,
    #[serde(default, deserialize_with = "deserialize_vec_pair")]
    pub data: Vec<(u16, i32)>,
}

#[derive(Debug, Clone, JominiDeserialize, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Province {
    #[jomini(default, deserialize_with = "deserialize_vec_pair")]
    pub flags: Vec<(String, Eu4Date)>,
    pub name: String,
    pub owner: Option<CountryTag>,
    pub controller: Option<CountryTag>,
    pub previous_controller: Option<CountryTag>,
    #[jomini(default)]
    pub cores: Vec<CountryTag>,
    #[jomini(default)]
    pub claims: Vec<CountryTag>,
    pub institutions: Vec<f32>,
    pub trade: Option<String>,
    pub original_culture: Option<String>,
    pub culture: Option<String>,
    pub religion: Option<String>,
    pub original_religion: Option<String>,
    pub trade_goods: Option<String>,
    #[jomini(default, deserialize_with = "deserialize_alternating_key_values")]
    pub country_improve_count: HashMap<CountryTag, i32>,
    #[jomini(default)]
    pub latent_trade_goods: Vec<String>,
    #[jomini(default)]
    pub devastation: f32,
    #[jomini(default)]
    pub base_tax: f32,
    #[jomini(default)]
    pub base_production: f32,
    #[jomini(default)]
    pub base_manpower: f32,
    pub capital: Option<String>,
    #[jomini(default)]
    pub local_autonomy: f32,
    #[jomini(default)]
    pub is_city: bool,
    #[jomini(default, deserialize_with = "deserialize_token_bool")]
    pub active_trade_company: bool,
    #[jomini(default)]
    pub center_of_trade: u8,
    #[jomini(default)]
    pub trade_power: f32,
    #[jomini(default, deserialize_with = "deserialize_token_bool")]
    pub hre: bool,
    #[jomini(default, deserialize_with = "deserialize_yes_map")]
    pub buildings: HashMap<String, bool>,
    #[jomini(default)]
    pub building_builders: HashMap<String, CountryTag>,
    #[jomini(default, duplicated, alias = "modifier")]
    pub modifiers: Vec<Modifier>,
    #[jomini(default)]
    pub history: ProvinceHistory,
    #[jomini(default = "default_true")]
    pub ub: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Modifier {
    pub modifier: String,
    pub date: Eu4Date,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct ProvinceHistory {
    pub owner: Option<CountryTag>,
    pub base_tax: Option<f32>,
    pub base_production: Option<f32>,
    pub base_manpower: Option<f32>,
    pub other: HashMap<String, ProvinceEventValue>,
    pub events: Vec<(Eu4Date, ProvinceEvents)>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct ProvinceEvents(pub Vec<ProvinceEvent>);

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub enum ProvinceEvent {
    Owner(CountryTag),
    Controller(ControllerEvent),
    BaseTax(f32),
    BaseProduction(f32),
    BaseManpower(f32),
    KV((String, ProvinceEventValue)),
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct ControllerEvent {
    tag: CountryTag,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub enum ProvinceEventValue {
    String(String),
    Float(f32),
    Int(i32),
    Bool(bool),
    Object,
    Array,
}

#[derive(Debug, Clone, JominiDeserialize, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Country {
    #[jomini(default, deserialize_with = "deserialize_token_bool")]
    pub human: bool,
    #[jomini(default)]
    pub was_player: bool,
    #[jomini(default)]
    pub has_switched_nation: bool,
    #[jomini(default)]
    pub is_great_power: bool,
    #[jomini(default)]
    pub history: CountryHistory,
    #[jomini(duplicated)]
    pub previous_country_tags: Vec<CountryTag>,
    pub name: Option<String>,
    pub government_rank: i32,
    pub subject_focus: i32,
    pub trade_mission: f32,
    pub blockade_mission: f32,
    pub continent: Vec<i32>,
    pub institutions: Vec<i32>,
    pub capital: ProvinceId,
    pub original_capital: Option<ProvinceId>,
    pub trade_port: ProvinceId,
    #[jomini(default)]
    pub base_tax: f32,
    #[jomini(default)]
    pub development: f32,
    #[jomini(default)]
    pub prestige: f32,
    #[jomini(default)]
    pub stability: f32,
    #[jomini(default)]
    pub treasury: f32,
    #[jomini(default)]
    pub inflation: f32,
    #[jomini(default)]
    pub corruption: f32,
    #[jomini(default)]
    pub raw_development: f32,
    pub capped_development: f32,
    pub realm_development: f32,
    pub isolationism: i32,
    #[jomini(default)]
    pub manpower: f32,
    #[jomini(default)]
    pub max_manpower: f32,
    #[jomini(default)]
    pub sailors: f32,
    #[jomini(default)]
    pub max_sailors: f32,
    #[jomini(default, alias = "overextension_percentage")]
    pub overextension: f32,
    #[jomini(default)]
    pub innovativeness: f32,
    #[jomini(default)]
    pub religious_unity: f32,
    pub initialized_rivals: bool,
    pub national_focus: Option<String>,
    pub recalculate_strategy: bool,
    pub colors: CountryColors,
    pub dirty_colony: bool,
    pub primary_culture: Option<String>,
    pub dominant_culture: Option<String>,
    #[jomini(duplicated, alias = "accepted_culture")]
    pub accepted_cultures: Vec<String>,
    pub religion: Option<String>,
    pub dominant_religion: Option<String>,
    pub technology_group: Option<String>,
    pub unit_type: Option<String>,
    pub tribute_type: Option<i32>,
    pub technology: CountryTechnology,
    pub ledger: CountryLedger,
    #[jomini(duplicated, alias = "loan")]
    pub loans: Vec<Loan>,
    #[jomini(duplicated, alias = "estate")]
    pub estates: Vec<Estate>,
    #[jomini(default)]
    pub subjects: Vec<CountryTag>,
    #[jomini(default, deserialize_with = "deserialize_vec_pair")]
    pub flags: Vec<(String, Eu4Date)>,
    pub highest_possible_fort: Option<i32>,
    pub transfer_home_bonus: f32,
    #[jomini(duplicated, alias = "enemy")]
    pub enemies: Vec<String>,
    #[jomini(default)]
    pub current_power_projection: f32,
    #[jomini(default)]
    pub great_power_score: f32,
    #[jomini(default)]
    pub total_war_worth: u32,
    #[jomini(duplicated, alias = "army")]
    pub armies: Vec<Army>,
    #[jomini(duplicated, alias = "navy")]
    pub navies: Vec<Navy>,
    pub custom_nation_points: Option<f32>,
    #[jomini(default)]
    pub num_of_cities: i32,
    #[jomini(default)]
    pub num_of_total_ports: i32,
    #[jomini(default)]
    pub completed_missions: Vec<String>,
    #[jomini(default, deserialize_with = "deserialize_vec_pair")]
    pub active_idea_groups: Vec<(String, u8)>,
    #[jomini(default, deserialize_with = "deserialize_vec_pair")]
    pub adm_spent_indexed: Vec<(i32, i32)>,
    #[jomini(default, deserialize_with = "deserialize_vec_pair")]
    pub dip_spent_indexed: Vec<(i32, i32)>,
    #[jomini(default, deserialize_with = "deserialize_vec_pair")]
    pub mil_spent_indexed: Vec<(i32, i32)>,
    #[jomini(default)]
    pub losses: WarParticipantLosses,
    pub decision_seed: i32,
    #[jomini(duplicated, alias = "mercenary_company")]
    pub mercenary_companries: Vec<MercenaryCompany>,
    pub monarch: Option<ObjId>,
    pub heir: Option<ObjId>,
    #[jomini(duplicated, alias = "previous_monarch")]
    pub previous_monarchs: Vec<ObjId>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct CountryLedger {
    #[serde(default)]
    pub income: Vec<f32>,
    #[serde(default)]
    pub expense: Vec<f32>,
    pub lastmonthincome: Option<f32>,
    #[serde(default)]
    pub lastmonthincometable: Vec<f32>,
    #[serde(default)]
    pub lastmonthexpensetable: Vec<f32>,
    #[serde(default)]
    pub totalexpensetable: Vec<f32>,
    #[serde(default)]
    pub lastyearincome: Vec<f32>,
    #[serde(default)]
    pub lastyearexpense: Vec<f32>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct CountryHistory {
    pub government: Option<String>,
    pub technology_group: Option<String>,
    pub primary_culture: Option<String>,
    pub add_government_reform: Vec<String>,
    pub events: Vec<(Eu4Date, CountryEvents)>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct CountryEvents(pub Vec<CountryEvent>);

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub enum CountryEvent {
    Monarch(Monarch),
    Heir(Monarch),
    MonarchHeir(Monarch),
    MonarchConsort(Monarch),
    Queen(Monarch),
    Union(u32),
    Capital(u32),
    ChangedCountryNameFrom(String),
    ChangedCountryAdjectiveFrom(String),
    ChangedCountryMapColorFrom(Vec<u8>),
    ChangedTagFrom(CountryTag),
    Leader(Leader),
    RemoveAcceptedCulture(String),
}

impl CountryEvent {
    pub fn as_monarch(&self) -> Option<&Monarch> {
        match &self {
            CountryEvent::Monarch(x)
            | CountryEvent::Heir(x)
            | CountryEvent::MonarchHeir(x)
            | CountryEvent::MonarchConsort(x)
            | CountryEvent::Queen(x) => Some(x),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Monarch {
    pub id: ObjId,
    pub name: String,
    pub country: CountryTag,
    #[serde(alias = "DIP")]
    pub dip: u16,
    #[serde(alias = "ADM")]
    pub adm: u16,
    #[serde(alias = "MIL")]
    pub mil: u16,
    #[serde(default, deserialize_with = "deserialize_token_bool")]
    pub regent: bool,
    #[serde(default)]
    pub culture: Option<String>,
    #[serde(default)]
    pub religion: Option<String>,
    pub birth_date: Eu4Date,
    #[serde(default, deserialize_with = "deserialize_vec_pair")]
    pub personalities: Vec<(String, String)>,
    pub leader_id: Option<ObjId>,
    pub leader: Option<Leader>,
}

#[derive(Debug, Clone, Serialize)]
pub enum LeaderKind {
    Admiral,
    General,
    Explorer,
    Conquistador,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Leader {
    pub name: String,
    #[serde(alias = "type")]
    pub kind: LeaderKind,
    #[serde(default)]
    pub manuever: u16,
    #[serde(default)]
    pub shock: u16,
    #[serde(default)]
    pub fire: u16,
    #[serde(default)]
    pub siege: u16,
    pub monarch_id: Option<ObjId>,

    // While activation and id can be none, it is so rare that there
    // is a test case for it to prevent regression.
    pub activation: Option<Eu4Date>,
    pub id: Option<ObjId>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MercenaryCompany {
    pub id: ObjId,
    pub tag: String,
    pub progress: bool,
    pub manpower: f32,
    pub starting_manpower: f32,
    pub leader: Option<Leader>,
    pub unit: Option<ObjId>,
    pub hiring_date: Eu4Date,
    pub disband_date: Eu4Date,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct CountryColors {
    pub revolutionary_colors: Option<Vec<u8>>,
    #[serde(default, deserialize_with = "deserialize_vec_overflow_byte")]
    pub map_color: Vec<u8>,
    #[serde(default, deserialize_with = "deserialize_vec_overflow_byte")]
    pub country_color: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Loan {
    pub id: ObjId,
    pub lender: String,
    pub interest: f32,
    pub fixed_interest: bool,
    pub amount: i32,
    pub expiry_date: Eu4Date,
    pub spawned: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Estate {
    #[serde(alias = "type")]
    pub _type: String,
    pub loyalty: f32,
    #[serde(default)]
    pub territory: f32,
    #[serde(default)]
    pub provinces: Vec<ProvinceId>,
    #[serde(default)]
    pub active_influences: Vec<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CountryTechnology {
    pub adm_tech: u8,
    pub dip_tech: u8,
    pub mil_tech: u8,
}

#[derive(Debug, Clone, JominiDeserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Army {
    pub id: ObjId,
    pub name: String,
    pub location: ProvinceId,
    #[jomini(duplicated, alias = "regiment")]
    pub regiments: Vec<Regiment>,
    pub movement_progress_last_updated: Eu4Date,
    pub graphical_culture: String,
    #[jomini(default)]
    pub main_army: bool,
    #[jomini(default)]
    pub is_invading: bool,
    #[jomini(default)]
    pub visible_to_ai: bool,
}

#[derive(Debug, Clone, JominiDeserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Navy {
    pub id: ObjId,
    pub name: String,
    pub location: ProvinceId,
    pub previous: Option<ProvinceId>,
    pub previous_war: Option<i32>,
    #[jomini(duplicated, alias = "ship")]
    pub ships: Vec<Ship>,
    pub movement_progress_last_updated: Eu4Date,
    pub graphical_culture: String,
    pub active_fraction_last_month: f32,
    #[jomini(default)]
    pub attrition: bool,
    #[jomini(default)]
    pub visible_to_ai: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Ship {
    pub id: ObjId,
    pub name: String,
    pub home: ProvinceId,
    #[serde(alias = "type")]
    pub _type: String,
    pub morale: f32,
    #[serde(default = "default_strength")]
    pub strength: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ObjId {
    pub id: u32,
    #[serde(alias = "type")]
    pub _type: u32,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Regiment {
    pub id: ObjId,
    pub name: String,
    pub home: ProvinceId,
    #[serde(alias = "type")]
    pub _type: String,
    pub morale: f32,
    #[serde(default)]
    pub drill: f32,
    #[serde(default = "default_strength")]
    pub strength: f32,
}

fn default_strength() -> f32 {
    1.0
}

#[derive(Debug, Clone, JominiDeserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct ActiveWar {
    pub name: String,
    pub history: WarHistory,
    #[jomini(duplicated, default)]
    pub participants: Vec<WarParticipant>,
    pub original_attacker: CountryTag,
    pub original_defender: CountryTag,
}

#[derive(Debug, Clone, JominiDeserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct PreviousWar {
    pub name: String,
    pub history: WarHistory,
    #[jomini(duplicated, default)]
    pub participants: Vec<WarParticipant>,
    pub original_attacker: CountryTag,
    pub original_defender: CountryTag,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct WarParticipant {
    pub value: f32,
    pub tag: CountryTag,
    #[serde(default)]
    pub losses: WarParticipantLosses,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct WarParticipantLosses {
    #[serde(default)]
    pub members: Vec<u32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct WarGoal {
    #[serde(alias = "type")]
    pub _type: String,
    pub casus_belli: String,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct WarHistory {
    pub name: Option<String>,
    pub war_goal: Option<WarGoal>,
    pub succession: Option<String>,
    pub events: Vec<(Eu4Date, WarEvents)>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct WarEvents(pub Vec<WarEvent>);

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub enum WarEvent {
    AddAttacker(CountryTag),
    AddDefender(CountryTag),
    RemoveAttacker(CountryTag),
    RemoveDefender(CountryTag),
    Battle(Battle),
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Battle {
    pub name: String,
    pub location: ProvinceId,
    #[serde(alias = "result", deserialize_with = "deserialize_token_bool")]
    pub attacker_won: bool,
    pub attacker: BattleSide,
    pub defender: BattleSide,
    #[serde(default)]
    pub winner_alliance: f32,
    #[serde(default)]
    pub loser_alliance: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct BattleSide {
    #[serde(default)]
    pub cavalry: u32,
    #[serde(default)]
    pub infantry: u32,
    #[serde(default)]
    pub artillery: u32,
    #[serde(default)]
    pub heavy_ship: u32,
    #[serde(default)]
    pub light_ship: u32,
    #[serde(default)]
    pub galley: u32,
    #[serde(default)]
    pub transport: u32,
    pub losses: u32,
    pub country: CountryTag,

    #[serde(deserialize_with = "empty_string_is_none")]
    pub commander: Option<String>,
}

fn default_true() -> bool {
    true
}
