use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObjectData {
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: String,
    pub name: String,
    pub price: f64,
    #[serde(default)]
    pub cost_1: String,
    #[serde(default)]
    pub cost_2: String,
    #[serde(default)]
    pub cost_3: String,
    #[serde(default)]
    pub cost_4: String,
    #[serde(default)]
    pub cost_5: String,
    #[serde(default)]
    pub cost_6: String,
    #[serde(default)]
    pub cost_7: String,
    #[serde(default)]
    pub cost_8: String,
    #[serde(default)]
    pub cost_9: String,
    #[serde(default)]
    pub cost_10: String,
    #[serde(default)]
    pub cost_11: String,
    #[serde(default)]
    pub cost_12: String,
    #[serde(default)]
    pub cost_13: String,
    #[serde(default)]
    pub cost_14: String,
    #[serde(default)]
    pub cost_15: String,
    #[serde(default)]
    pub service_1_interval_days: u32,
    #[serde(default)]
    pub service_2_interval_days: u32,
    #[serde(default)]
    pub service_3_interval_days: u32,
    #[serde(default)]
    pub service_4_interval_days: u32,
    #[serde(default)]
    pub service_5_interval_days: u32,
    #[serde(default)]
    pub service_6_interval_days: u32,
    #[serde(default)]
    pub service_7_interval_days: u32,
    #[serde(default)]
    pub service_8_interval_days: u32,
    #[serde(default)]
    pub service_9_interval_days: u32,
    #[serde(default)]
    pub service_10_interval_days: u32,
    #[serde(default)]
    pub service_11_interval_days: u32,
    #[serde(default)]
    pub service_12_interval_days: u32,
    #[serde(default)]
    pub service_13_interval_days: u32,
    #[serde(default)]
    pub service_14_interval_days: u32,
    #[serde(default)]
    pub service_15_interval_days: u32,
    #[serde(default = "default_resale_initial_percent")]
    pub resale_initial_percent: f64,
    #[serde(default = "default_resale_annual_percent")]
    pub resale_annual_percent: f64,
    #[serde(default = "default_resale_min_percent")]
    pub resale_min_percent: f64,
    #[serde(default, alias = "description", alias = "description_path", alias = "html")]
    pub description_html: String,
    #[serde(default)]
    pub license_level: u32,
    #[serde(default)]
    pub license_previous_id: String,
    #[serde(default)]
    pub requires_object_ids: String,
    #[serde(default, deserialize_with = "deserialize_zero_f64")]
    pub license_fee: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostData {
    pub id: String,
    pub name: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerCharacteristicData {
    pub id: String,
    pub name: String,
    pub value: f64,
    #[serde(default = "default_characteristic_min", deserialize_with = "deserialize_min_bound")]
    pub min_value: f64,
    #[serde(default = "default_characteristic_max", deserialize_with = "deserialize_max_bound")]
    pub max_value: f64,
}

fn default_characteristic_min() -> f64 {
    f64::NEG_INFINITY
}

fn default_characteristic_max() -> f64 {
    f64::INFINITY
}

fn deserialize_min_bound<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<f64>::deserialize(deserializer)?.unwrap_or_else(default_characteristic_min))
}

fn deserialize_max_bound<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<f64>::deserialize(deserializer)?.unwrap_or_else(default_characteristic_max))
}

fn deserialize_zero_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<f64>::deserialize(deserializer)?.unwrap_or(0.0))
}

fn default_resale_initial_percent() -> f64 {
    0.9
}

fn default_resale_annual_percent() -> f64 {
    0.9
}

fn default_resale_min_percent() -> f64 {
    0.1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionData {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub action_type: String,
    pub base_cost: f64,
    #[serde(default = "default_action_stamina_cost")]
    pub stamina_cost: f64,
    pub risk_factor: f64,
    pub success_rate: f64,
    pub payout: f64,
    pub payout_freq_type: String,
    pub payout_freq: u32,
    pub payout_freq_unit: String,
    #[serde(default, alias = "description", alias = "description_path", alias = "html")]
    pub description_html: String,
}

fn default_action_stamina_cost() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventData {
    pub id: String,
    pub name: String,
    pub day_of_year: u32,
    pub entry_fee: f64,
    pub reward_pool: f64,
    #[serde(default)]
    pub charisma_reward: f64,
    #[serde(default)]
    pub duration_value: u32,
    #[serde(default = "default_duration_unit")]
    pub duration_unit: String,
    #[serde(default)]
    pub tags: String,
    #[serde(default, alias = "description", alias = "description_path", alias = "html")]
    pub description_html: String,
    #[serde(default)]
    pub required_license_id: String,
    #[serde(default)]
    pub required_object_ids: String,
    #[serde(default)]
    pub championship_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChampionshipData {
    pub id: String,
    pub name: String,
    #[serde(default = "default_championship_success_points")]
    pub success_points: f64,
    #[serde(default)]
    pub failure_points: f64,
}

fn default_championship_success_points() -> f64 {
    10.0
}

fn default_duration_unit() -> String {
    "days".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostRule {
    pub id: String,
    pub cost_id: String,
    pub trigger_type: String,
    #[serde(default)]
    pub trigger_ref: String,
    #[serde(default = "default_multiplier")]
    pub amount_multiplier: f64,
    #[serde(default = "default_probability")]
    pub probability: f64,
    #[serde(default)]
    pub interval_days: u32,
    #[serde(default = "default_charge_mode")]
    pub charge_mode: String,
    #[serde(default = "default_resolution_mode")]
    pub resolution_mode: String,
    #[serde(default)]
    pub pending_message: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub damage_type: String,
    #[serde(default)]
    pub unavailable_days: u32,
}

fn default_multiplier() -> f64 {
    1.0
}

fn default_probability() -> f64 {
    1.0
}

fn default_charge_mode() -> String {
    "immediate".into()
}

fn default_resolution_mode() -> String {
    "charge".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostCondition {
    pub rule_id: String,
    pub subject_type: String,
    pub subject_ref: String,
    pub operator: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameLabels {
    #[serde(default)]
    pub values: HashMap<String, String>,
}

impl GameLabels {
    pub fn get(&self, key: &str, fallback: &str) -> String {
        self.values
            .get(key)
            .cloned()
            .unwrap_or_else(|| fallback.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameCatalog {
    pub player_characteristics: Vec<PlayerCharacteristicData>,
    pub objects: Vec<ObjectData>,
    pub costs: Vec<CostData>,
    pub cost_rules: Vec<CostRule>,
    pub cost_conditions: Vec<CostCondition>,
    pub actions: Vec<ActionData>,
    pub events: Vec<EventData>,
    #[serde(default)]
    pub championships: Vec<ChampionshipData>,
    pub labels: GameLabels,
}

impl GameCatalog {
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Self {
        let base = dir.as_ref();
        let labels = parse_config_file(base.join("config.csv")).unwrap_or_default();
        let player_characteristics =
            parse_csv_file(base.join("player.csv")).unwrap_or_default();
        let objects = parse_csv_file(base.join("objects.csv")).unwrap_or_default();
        let costs = parse_csv_file(base.join("costs.csv")).unwrap_or_default();
        let cost_rules = parse_csv_file(base.join("cost_rules.csv")).unwrap_or_default();
        let cost_conditions =
            parse_csv_file(base.join("cost_rule_conditions.csv")).unwrap_or_default();
        let actions = parse_csv_file(base.join("actions.csv")).unwrap_or_default();
        let events = parse_csv_file(base.join("events.csv")).unwrap_or_default();
        let championships = parse_csv_file(base.join("championships.csv")).unwrap_or_default();

        Self {
            player_characteristics,
            objects,
            costs,
            cost_rules,
            cost_conditions,
            actions,
            events,
            championships,
            labels,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ConfigRecord {
    variable: String,
    value: String,
}

fn parse_config_file<P: AsRef<Path>>(path: P) -> Result<GameLabels, Box<dyn Error>> {
    if !path.as_ref().exists() {
        return Ok(GameLabels::default());
    }

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)?;
    let mut values = HashMap::new();
    for result in reader.deserialize() {
        let record: ConfigRecord = result?;
        values.insert(record.variable, record.value);
    }
    Ok(GameLabels { values })
}

pub fn parse_csv_file<T, P: AsRef<Path>>(path: P) -> Result<Vec<T>, Box<dyn Error>>
where
    T: serde::de::DeserializeOwned,
{
    if !path.as_ref().exists() {
        return Ok(Vec::new());
    }

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)?;
    let mut records = Vec::new();
    for result in reader.deserialize() {
        records.push(result?);
    }
    Ok(records)
}
