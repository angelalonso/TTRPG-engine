pub mod engine;

use engine::loader::GameCatalog;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

// ============================================================================
// Data Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeSpeed {
    Paused,
    OneDayEveryFiveSec,
    OneDayPerSec,
    OneWeekPerSec,
    RealTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaintenanceType {
    EngineRebuild,
    GearboxService,
    OilChange,
    BuyTires(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RacePosition {
    Placement(String),
    DNF { dnf: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedCar {
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
    pub tire_sets_available: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActiveAction {
    pub action_id: String,
    pub start_day: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAlert {
    pub id: String,
    pub title: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub age_days: u32,
    pub budget: f64,
    pub cars: Vec<OwnedCar>,
    pub active_actions: Vec<ActiveAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: Player,
    pub catalog: GameCatalog,
    pub dataset_path: String,
    pub time_speed: TimeSpeed,
    pub current_day: u32,
    pub pending_alerts: Vec<GameAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceResult {
    pub race_name: String,
    pub position: RacePosition,
    pub entry_fee_paid: f64,
    pub prize_awarded: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action_name: String,
    pub success: bool,
    pub payout_received: f64,
    pub cost_paid: f64,
    pub message: String,
}

// Managed Application State
pub struct AppState(pub Mutex<GameState>);

fn calculate_interval_days(freq: u32, unit: &str) -> u32 {
    let multiplier = match unit.trim().to_lowercase().as_str() {
        "day" | "days" => 1,
        "month" | "months" => 30,
        "year" | "years" => 365,
        _ => 1,
    };
    freq * multiplier
}

// ============================================================================
// Initial Catalog Seeding
// ============================================================================

fn create_initial_state() -> GameState {
    let dataset_path = "dataset".to_string();
    let catalog = GameCatalog::load_from_directory(&dataset_path);

    GameState {
        current_day: 1,
        time_speed: TimeSpeed::Paused,
        player: Player {
            age_days: 18 * 365,
            budget: 20_000.0,
            cars: vec![],
            active_actions: vec![],
        },
        dataset_path,
        catalog,
        pending_alerts: vec![],
    }
}

// ============================================================================
// Tauri v2 Command Handlers
// ============================================================================

#[tauri::command]
fn get_game_state(state: State<'_, AppState>) -> Result<GameState, String> {
    let game = state.0.lock().map_err(|e| e.to_string())?;
    Ok(game.clone())
}

#[tauri::command]
fn set_time_speed(speed: TimeSpeed, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    game.time_speed = speed;
    Ok(game.clone())
}

#[tauri::command]
fn dismiss_alert(alert_id: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    game.pending_alerts.retain(|a| a.id != alert_id);
    Ok(game.clone())
}

#[tauri::command]
fn reload_dataset(new_path: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    let catalog = GameCatalog::load_from_directory(&new_path);
    let current_day = game.current_day;
    game.catalog = catalog;
    game.dataset_path = new_path.clone();

    let alert = GameAlert {
        id: format!("dataset_reload_{}", current_day),
        title: "⚙️ Dataset Reloaded".into(),
        message: format!("Successfully reloaded game data from directory: '{}'.", new_path),
    };
    game.pending_alerts.push(alert);

    Ok(game.clone())
}

#[tauri::command]
fn tick_game_day(state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    game.current_day += 1;
    game.player.age_days += 1;

    let current_day = game.current_day;
    let current_day_of_year = ((current_day - 1) % 365) + 1;

    // 1. Scheduled Car Maintenance Check
    for car in &mut game.player.cars {
        if current_day % 60 == 0 {
            car.needs_oil_change = true;
        }
        if current_day % 180 == 0 {
            car.needs_gearbox_maint = true;
        }
    }

    // 2. Process Payday / Recurring Income
    let mut total_payout = 0.0;
    let mut paid_jobs = Vec::new();
    for active in &game.player.active_actions {
        if let Some(action) = game.catalog.actions.iter().find(|a| a.id == active.action_id) {
            if action.payout_freq_type.trim().eq_ignore_ascii_case("recurring") {
                let interval = calculate_interval_days(action.payout_freq, &action.payout_freq_unit);
                if interval > 0 {
                    let days_elapsed = current_day.saturating_sub(active.start_day);
                    if days_elapsed > 0 && days_elapsed % interval == 0 {
                        total_payout += action.payout;
                        paid_jobs.push((action.name.clone(), action.payout));
                    }
                }
            }
        }
    }

    if total_payout > 0.0 {
        game.player.budget += total_payout;
        for (job_name, payout) in paid_jobs {
            game.pending_alerts.push(GameAlert {
                id: format!("payday_{}_{}", job_name, current_day),
                title: "💰 Salary Payday!".into(),
                message: format!("You received your payday salary of £{:.2} from '{}'!", payout, job_name),
            });
        }
        game.time_speed = TimeSpeed::Paused;
    }

    // 3. Race Day Check
    let mut race_alerts = Vec::new();
    for race in &game.catalog.races {
        if race.day_of_year == current_day_of_year {
            race_alerts.push(GameAlert {
                id: format!("race_{}_{}", race.id, current_day),
                title: "🏁 Race Day Today!".into(),
                message: format!("Today is Day {} of the year: '{}' is taking place today!", current_day_of_year, race.name),
            });
        }
    }

    if !race_alerts.is_empty() {
        game.pending_alerts.extend(race_alerts);
        game.time_speed = TimeSpeed::Paused;
    }

    Ok(game.clone())
}

#[tauri::command]
fn buy_car(car_id: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;

    let car_spec = game
        .catalog
        .cars
        .iter()
        .find(|c| c.id == car_id)
        .cloned()
        .ok_or_else(|| "Car not found in catalog".to_string())?;

    if game.player.budget < car_spec.price {
        return Err("Insufficient funds to purchase vehicle".into());
    }

    game.player.budget -= car_spec.price;

    let new_owned_car = OwnedCar {
        id: format!("{}_{}", car_spec.id, game.player.cars.len() + 1),
        name: car_spec.name,
        price: car_spec.price,
        engine_rebuild_cost: car_spec.engine_rebuild_cost,
        gearbox_maint_cost: car_spec.gearbox_maint_cost,
        oil_change_cost: car_spec.oil_change_cost,
        tire_set_cost: car_spec.tire_set_cost,
        needs_oil_change: false,
        needs_engine_rebuild: false,
        needs_gearbox_maint: false,
        tire_sets_available: 4,
    };

    game.player.cars.push(new_owned_car);
    Ok(game.clone())
}

#[tauri::command]
fn maintain_car(
    car_id: String,
    maintenance_type: MaintenanceType,
    state: State<'_, AppState>,
) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;

    let car_idx = game
        .player
        .cars
        .iter()
        .position(|c| c.id == car_id)
        .ok_or_else(|| "Car not found in garage".to_string())?;

    match maintenance_type {
        MaintenanceType::OilChange => {
            if !game.player.cars[car_idx].needs_oil_change {
                return Err("Oil change is not required".into());
            }
            let cost = game.player.cars[car_idx].oil_change_cost;
            if game.player.budget < cost {
                return Err("Insufficient funds for oil change".into());
            }
            game.player.budget -= cost;
            game.player.cars[car_idx].needs_oil_change = false;
        }
        MaintenanceType::EngineRebuild => {
            if !game.player.cars[car_idx].needs_engine_rebuild {
                return Err("Engine rebuild is not required".into());
            }
            let cost = game.player.cars[car_idx].engine_rebuild_cost;
            if game.player.budget < cost {
                return Err("Insufficient funds for engine rebuild".into());
            }
            game.player.budget -= cost;
            game.player.cars[car_idx].needs_engine_rebuild = false;
        }
        MaintenanceType::GearboxService => {
            if !game.player.cars[car_idx].needs_gearbox_maint {
                return Err("Gearbox service is not required".into());
            }
            let cost = game.player.cars[car_idx].gearbox_maint_cost;
            if game.player.budget < cost {
                return Err("Insufficient funds for gearbox service".into());
            }
            game.player.budget -= cost;
            game.player.cars[car_idx].needs_gearbox_maint = false;
        }
        MaintenanceType::BuyTires(sets) => {
            let total_cost = game.player.cars[car_idx].tire_set_cost * sets as f64;
            if game.player.budget < total_cost {
                return Err("Insufficient funds for tire purchase".into());
            }
            game.player.budget -= total_cost;
            game.player.cars[car_idx].tire_sets_available += sets;
        }
    }

    Ok(game.clone())
}

#[tauri::command]
fn perform_action(
    action_id: String,
    state: State<'_, AppState>,
) -> Result<ActionResult, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;

    let action = game
        .catalog
        .actions
        .iter()
        .find(|a| a.id == action_id)
        .cloned()
        .ok_or_else(|| "Action not found in catalog".to_string())?;

    if game.player.budget < action.base_cost {
        return Err("Insufficient funds to start this activity".into());
    }

    game.player.budget -= action.base_cost;

    let is_recurring = action.payout_freq_type.trim().eq_ignore_ascii_case("recurring");

    if is_recurring {
        if game.player.active_actions.iter().any(|a| a.action_id == action.id) {
            return Err(format!("You already have an active contract/job for '{}'.", action.name));
        }

        let interval = calculate_interval_days(action.payout_freq, &action.payout_freq_unit);
        let start_day = game.current_day;

        game.player.active_actions.push(ActiveAction {
            action_id: action.id.clone(),
            start_day,
        });

        Ok(ActionResult {
            action_name: action.name.clone(),
            success: true,
            payout_received: 0.0,
            cost_paid: action.base_cost,
            message: format!(
                "Started '{}'! First salary of £{:.2} will arrive on Day {}.",
                action.name,
                action.payout,
                start_day + interval
            ),
        })
    } else {
        let success = action.success_rate >= 0.50;
        let payout = if success { action.payout } else { 0.0 };
        game.player.budget += payout;

        let message = if success {
            format!("Successfully completed '{}' and earned £{:.2}.", action.name, payout)
        } else {
            format!("Failed to complete '{}'. Base cost lost.", action.name)
        };

        Ok(ActionResult {
            action_name: action.name,
            success,
            payout_received: payout,
            cost_paid: action.base_cost,
            message,
        })
    }
}

#[tauri::command]
fn enter_race(
    car_id: String,
    race_id: String,
    state: State<'_, AppState>,
) -> Result<RaceResult, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;

    let current_day = game.current_day;
    let current_day_of_year = ((current_day - 1) % 365) + 1;

    let race = game
        .catalog
        .races
        .iter()
        .find(|r| r.id == race_id)
        .cloned()
        .ok_or_else(|| "Race event not found in catalog".to_string())?;

    if race.day_of_year != current_day_of_year {
        return Err(format!(
            "Race is scheduled for day {}, but today is day {}.",
            race.day_of_year, current_day_of_year
        ));
    }

    if game.player.budget < race.entry_fee {
        return Err("Insufficient funds for race entry fee".into());
    }

    let car_idx = game
        .player
        .cars
        .iter()
        .position(|c| c.id == car_id)
        .ok_or_else(|| "Vehicle not found in garage".to_string())?;

    if game.player.cars[car_idx].tire_sets_available < 4 {
        return Err("Vehicle requires at least 4 tire sets for race entry".into());
    }

    if game.player.cars[car_idx].needs_oil_change
        || game.player.cars[car_idx].needs_engine_rebuild
        || game.player.cars[car_idx].needs_gearbox_maint
    {
        return Err("Vehicle requires maintenance before racing".into());
    }

    game.player.budget -= race.entry_fee;
    game.player.cars[car_idx].tire_sets_available -= 4;

    let position = RacePosition::Placement("First".to_string());
    let prize = race.prize_pool;
    game.player.budget += prize;

    let car_name = game.player.cars[car_idx].name.clone();
    game.player.cars[car_idx].needs_oil_change = true;

    // Trigger Post-Race Maintenance Alert & Auto-Pause
    game.pending_alerts.push(GameAlert {
        id: format!("maint_{}_{}", car_id, current_day),
        title: "🔧 Post-Race Maintenance Required!".into(),
        message: format!("Race complete! '{}' requires an oil change and service before your next race.", car_name),
    });
    game.time_speed = TimeSpeed::Paused;

    Ok(RaceResult {
        race_name: race.name,
        position,
        entry_fee_paid: race.entry_fee,
        prize_awarded: prize,
        message: "Phenomenal drive! You took 1st place and claimed the winner's purse.".into(),
    })
}

// ============================================================================
// Tauri App Initialization
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState(Mutex::new(create_initial_state())))
        .invoke_handler(tauri::generate_handler![
            get_game_state,
            set_time_speed,
            tick_game_day,
            buy_car,
            maintain_car,
            perform_action,
            enter_race,
            dismiss_alert,
            reload_dataset
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
