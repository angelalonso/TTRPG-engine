use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CarData {
    pub id: String,
    pub name: String,
    pub price: f64,
    pub engine_rebuild_cost: f64,
    pub gearbox_maint_cost: f64,
    pub oil_change_cost: f64,
    pub tire_set_cost: f64,
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
    pub payout_freq_type: String, // "once" | "recurring"
    pub payout_freq: u32,         // e.g. 1
    pub payout_freq_unit: String, // "day" | "month" | "year"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RaceData {
    pub id: String,
    pub name: String,
    pub day_of_year: u32,
    pub entry_fee: f64,
    pub prize_pool: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameCatalog {
    pub cars: Vec<CarData>,
    pub actions: Vec<ActionData>,
    pub races: Vec<RaceData>,
}

impl GameCatalog {
    /// Loads catalog dynamically from a directory. If any file is missing or corrupt, returns empty vectors.
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Self {
        let base = dir.as_ref();

        let cars = parse_csv_file(&base.join("cars.csv")).unwrap_or_default();
        let actions = parse_csv_file(&base.join("actions.csv")).unwrap_or_default();
        let races = parse_csv_file(&base.join("races.csv")).unwrap_or_default();

        Self { cars, actions, races }
    }
}

pub fn parse_csv_file<T, P: AsRef<Path>>(path: P) -> Result<Vec<T>, Box<dyn Error>>
where
    T: serde::de::DeserializeOwned,
{
    let path_ref = path.as_ref();
    if !path_ref.exists() {
        return Ok(Vec::new());
    }

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path_ref)?;

    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: T = result?;
        records.push(record);
    }
    Ok(records)
}
