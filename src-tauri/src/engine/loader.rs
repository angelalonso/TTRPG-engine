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
    pub fn load_embedded() -> Result<Self, Box<dyn Error>> {
        let cars_raw = include_str!("../../data/cars.csv");
        let actions_raw = include_str!("../../data/actions.csv");
        let races_raw = include_str!("../../data/races.csv");

        Ok(Self {
            cars: parse_csv_str(cars_raw)?,
            actions: parse_csv_str(actions_raw)?,
            races: parse_csv_str(races_raw)?,
        })
    }

    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Result<Self, Box<dyn Error>> {
        let base = dir.as_ref();

        Ok(Self {
            cars: parse_csv_file(&base.join("cars.csv"))?,
            actions: parse_csv_file(&base.join("actions.csv"))?,
            races: parse_csv_file(&base.join("races.csv"))?,
        })
    }
}

pub fn parse_csv_str<T>(contents: &str) -> Result<Vec<T>, Box<dyn Error>>
where
    T: serde::de::DeserializeOwned,
{
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(contents.as_bytes());

    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: T = result?;
        records.push(record);
    }
    Ok(records)
}

pub fn parse_csv_file<T, P: AsRef<Path>>(path: P) -> Result<Vec<T>, Box<dyn Error>>
where
    T: serde::de::DeserializeOwned,
{
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)?;

    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: T = result?;
        records.push(record);
    }
    Ok(records)
}
