use serde::{Deserialize, Serialize};
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
    pub cost_1: String,
    pub cost_2: String,
    pub cost_3: String,
    pub cost_4: String,
    #[serde(default = "default_object_units")]
    pub units_available: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostData {
    pub id: String,
    pub name: String,
    pub amount: f64,
}

fn default_object_units() -> u32 {
    4
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionData {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub action_type: String,
    pub base_cost: f64,
    pub risk_factor: f64,
    pub success_rate: f64,
    pub payout: f64,
    pub payout_freq_type: String,
    pub payout_freq: u32,
    pub payout_freq_unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventData {
    pub id: String,
    pub name: String,
    pub day_of_year: u32,
    pub entry_fee: f64,
    pub reward_pool: f64,
    #[serde(default)]
    pub tags: String,
    #[serde(default = "default_event_units")]
    pub object_units_required: u32,
}

fn default_event_units() -> u32 {
    4
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
    pub objects: Vec<ObjectData>,
    pub costs: Vec<CostData>,
    pub cost_rules: Vec<CostRule>,
    pub cost_conditions: Vec<CostCondition>,
    pub actions: Vec<ActionData>,
    pub events: Vec<EventData>,
    pub labels: GameLabels,
}

impl GameCatalog {
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Self {
        let base = dir.as_ref();
        let labels = parse_config_file(base.join("config.csv")).unwrap_or_default();
        let objects = parse_csv_file(base.join("objects.csv")).unwrap_or_default();
        let costs = parse_csv_file(base.join("costs.csv")).unwrap_or_default();
        let cost_rules = parse_csv_file(base.join("cost_rules.csv")).unwrap_or_default();
        let cost_conditions =
            parse_csv_file(base.join("cost_rule_conditions.csv")).unwrap_or_default();
        let actions = parse_csv_file(base.join("actions.csv")).unwrap_or_default();
        let events = parse_csv_file(base.join("events.csv")).unwrap_or_default();

        Self {
            objects,
            costs,
            cost_rules,
            cost_conditions,
            actions,
            events,
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
