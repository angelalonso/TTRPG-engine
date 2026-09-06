pub mod loader;
pub mod race;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod race_tests;

use loader::GameCatalog;
use rand::RngExt;
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActiveAction {
    pub action_id: String,
    pub start_day: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action_name: String,
    pub success: bool,
    pub payout_received: f64,
    pub cost_paid: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub age_days: u32,
    pub budget: f64,
    pub cars: Vec<Car>,
    pub active_actions: Vec<ActiveAction>,
}

impl Player {
    pub fn new(age_years: u32, budget: f64) -> Self {
        Self {
            age_days: age_years * 365,
            budget,
            cars: Vec::new(),
            active_actions: Vec::new(),
        }
    }

    pub fn execute_trading(&mut self, stake: f64, success_rate: f64, payout: f64, _risk_factor: f64) -> bool {
        if self.budget < stake {
            return false;
        }

        self.budget -= stake;

        let mut rng = rand::rng();
        let roll: f64 = rng.random();

        if roll <= success_rate {
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

pub fn calculate_interval_days(freq: u32, unit: &str) -> u32 {
    let multiplier = match unit.trim().to_lowercase().as_str() {
        "day" | "days" => 1,
        "month" | "months" => 30,
        "year" | "years" => 365,
        _ => 1,
    };
    freq * multiplier
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        let catalog = GameCatalog::load_from_directory("dataset");

        Self {
            player: Player::new(18, 2000.0),
            catalog,
            time_speed: TimeSpeed::Paused,
            current_day: 1,
            pending_events: Vec::new(),
        }
    }

    pub fn perform_action(&mut self, action_id: &str) -> Result<ActionResult, String> {
        let action = self
            .catalog
            .actions
            .iter()
            .find(|a| a.id == action_id)
            .cloned()
            .ok_or_else(|| format!("Action '{action_id}' not found"))?;

        if self.player.budget < action.base_cost {
            return Err(format!("Insufficient budget for {}", action.name));
        }

        self.player.budget -= action.base_cost;

        let mut rng = rand::rng();
        let roll: f64 = rng.random();

        if roll <= action.success_rate {
            let is_recurring = action.payout_freq_type.trim().eq_ignore_ascii_case("recurring");

            if !is_recurring {
                self.player.budget += action.payout;
                Ok(ActionResult {
                    action_name: action.name.clone(),
                    success: true,
                    payout_received: action.payout,
                    cost_paid: action.base_cost,
                    message: format!("Successfully performed {} and received £{:.2}!", action.name, action.payout),
                })
            } else if !self.player.active_actions.iter().any(|a| a.action_id == action.id) {
                self.player.active_actions.push(ActiveAction {
                    action_id: action.id.clone(),
                    start_day: self.current_day,
                });

                let interval = calculate_interval_days(action.payout_freq, &action.payout_freq_unit);

                Ok(ActionResult {
                    action_name: action.name.clone(),
                    success: true,
                    payout_received: 0.0,
                    cost_paid: action.base_cost,
                    message: format!(
                        "Started {}. Your first payout of £{:.2} arrives on Day {}.",
                        action.name,
                        action.payout,
                        self.current_day + interval
                    ),
                })
            } else {
                Err(format!("You are already active in {}", action.name))
            }
        } else {
            Ok(ActionResult {
                action_name: action.name.clone(),
                success: false,
                payout_received: 0.0,
                cost_paid: action.base_cost,
                message: format!("Failed to complete {}.", action.name),
            })
        }
    }

    pub fn tick_day(&mut self) -> Option<GameEvent> {
        self.current_day += 1;
        self.player.age_days += 1;

        if self.current_day.is_multiple_of(365) {
            for car in &mut self.player.cars {
                car.needs_engine_rebuild = true;
                car.needs_gearbox_maint = true;
            }
        }

        // Process recurring payouts on exact interval boundaries
        for active in &self.player.active_actions {
            if let Some(action) = self.catalog.actions.iter().find(|a| a.id == active.action_id) {
                if action.payout_freq_type.trim().eq_ignore_ascii_case("recurring") {
                    let interval = calculate_interval_days(action.payout_freq, &action.payout_freq_unit);
                    if interval > 0 {
                        let days_elapsed = self.current_day.saturating_sub(active.start_day);
                        if days_elapsed > 0 && days_elapsed.is_multiple_of(interval) {
                            self.player.budget += action.payout;
                        }
                    }
                }
            }
        }

        let mut rng = rand::rng();
        let chance: f64 = rng.random();

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
