pub mod loader;
pub mod race;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod race_tests;

use loader::GameCatalog;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeSpeed {
    Paused,
    OneDayEveryFiveSec,
    OneDayPerSec,
    OneWeekPerSec,
    RealTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Car {
    pub id: String,
    pub name: String,
    pub price: f64,
    pub engine_rebuild_cost: f64,
    pub gearbox_maint_cost: f64,
    pub oil_change_cost: f64,
    pub tire_set_cost: f64,
    pub needs_oil_change: bool,
    pub needs_engine_rebuild: bool,
    pub needs_gearbox_maint: bool,
    pub tire_sets_available: u8,
}

impl Car {
    pub fn is_race_ready(&self) -> bool {
        !self.needs_oil_change
            && !self.needs_engine_rebuild
            && !self.needs_gearbox_maint
            && self.tire_sets_available >= 4
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub id: String,
    pub title: String,
    pub description: String,
    pub financial_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub age_days: u32,
    pub budget: f64,
    pub cars: Vec<Car>,
}

impl Player {
    pub fn new(age_years: u32, budget: f64) -> Self {
        Self {
            age_days: age_years * 365,
            budget,
            cars: Vec::new(),
        }
    }

    pub fn execute_trading(&mut self, cost: f64, win_prob: f64, payout: f64, roll: f64) -> bool {
        if self.budget < cost {
            return false;
        }
        self.budget -= cost;
        if roll <= win_prob {
            self.budget += payout;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: Player,
    pub catalog: GameCatalog,
    pub time_speed: TimeSpeed,
    pub current_day: u32,
    pub pending_events: Vec<GameEvent>,
}

impl GameState {
    pub fn new() -> Self {
        let catalog = GameCatalog::load_embedded().unwrap_or_else(|err| {
            eprintln!("Failed to load embedded CSV catalog: {err}");
            GameCatalog::default()
        });

        Self {
            player: Player::new(18, 2000.0),
            catalog,
            time_speed: TimeSpeed::Paused,
            current_day: 1,
            pending_events: Vec::new(),
        }
    }

    pub fn tick_day(&mut self) -> Option<GameEvent> {
        self.current_day += 1;
        self.player.age_days += 1;

        if self.current_day % 365 == 0 {
            for car in &mut self.player.cars {
                car.needs_engine_rebuild = true;
                car.needs_gearbox_maint = true;
            }
        }

        let mut rng = rand::thread_rng();
        let chance: f64 = rng.gen();

        if chance < 0.02 {
            let event = GameEvent {
                id: "ev_injury".into(),
                title: "Convenience Store Accident".into(),
                description: "You slipped outside the store. Minor medical bill!".into(),
                financial_impact: -120.0,
            };

            self.player.budget += event.financial_impact;
            self.pending_events.push(event.clone());
            self.time_speed = TimeSpeed::RealTime;
            return Some(event);
        }

        None
    }
}
