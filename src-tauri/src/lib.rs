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
    Placement(String), // "First", "Second", "Third", "Unplaced"
    DNF { dnf: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarData {
    pub id: String,
    pub name: String,
    pub price: u64,
    pub engine_rebuild_cost: u64,
    pub gearbox_maint_cost: u64,
    pub oil_change_cost: u64,
    pub tire_set_cost: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionData {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub base_cost: u64,
    pub risk_factor: f64,
    pub success_rate: f64,
    pub payout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceData {
    pub id: String,
    pub name: String,
    pub day_of_year: u32,
    pub entry_fee: u64,
    pub prize_pool: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameCatalog {
    pub cars: Vec<CarData>,
    pub actions: Vec<ActionData>,
    pub races: Vec<RaceData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedCar {
    pub id: String,
    pub name: String,
    pub price: u64,
    pub engine_rebuild_cost: u64,
    pub gearbox_maint_cost: u64,
    pub oil_change_cost: u64,
    pub tire_set_cost: u64,
    pub needs_oil_change: bool,
    pub needs_engine_rebuild: bool,
    pub needs_gearbox_maint: bool,
    pub tire_sets_available: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub age_days: u32,
    pub budget: u64,
    pub cars: Vec<OwnedCar>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: Player,
    pub catalog: GameCatalog,
    pub time_speed: TimeSpeed,
    pub current_day: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceResult {
    pub race_name: String,
    pub position: RacePosition,
    pub entry_fee_paid: u64,
    pub prize_awarded: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action_name: String,
    pub success: bool,
    pub payout_received: u64,
    pub cost_paid: u64,
    pub message: String,
}

// Managed Application State
pub struct AppState(pub Mutex<GameState>);

// ============================================================================
// Initial Catalog Seeding
// ============================================================================

fn create_initial_state() -> GameState {
    let catalog = GameCatalog {
        cars: vec![
            CarData {
                id: "car_mx5".into(),
                name: "MX-5 Cup Car".into(),
                price: 15_000,
                engine_rebuild_cost: 3_500,
                gearbox_maint_cost: 1_200,
                oil_change_cost: 150,
                tire_set_cost: 600,
            },
            CarData {
                id: "car_gt4".into(),
                name: "Cayman GT4 Clubsport".into(),
                price: 85_000,
                engine_rebuild_cost: 12_000,
                gearbox_maint_cost: 4_500,
                oil_change_cost: 400,
                tire_set_cost: 1_800,
            },
            CarData {
                id: "car_gt3".into(),
                name: "911 GT3 R".into(),
                price: 250_000,
                engine_rebuild_cost: 35_000,
                gearbox_maint_cost: 14_000,
                oil_change_cost: 800,
                tire_set_cost: 3_200,
            },
        ],
        actions: vec![
            ActionData {
                id: "job_mechanic".into(),
                name: "Pit Crew Freelance Shift".into(),
                r#type: "Labor".into(),
                base_cost: 0,
                risk_factor: 0.05,
                success_rate: 0.95,
                payout: 450,
            },
            ActionData {
                id: "job_instructor".into(),
                name: "Track Day Instructor".into(),
                r#type: "Coaching".into(),
                base_cost: 50,
                risk_factor: 0.10,
                success_rate: 0.88,
                payout: 1_200,
            },
            ActionData {
                id: "job_sponsor".into(),
                name: "Sponsorship Pitch".into(),
                r#type: "Business".into(),
                base_cost: 250,
                risk_factor: 0.40,
                success_rate: 0.60,
                payout: 8_500,
            },
        ],
        races: vec![
            RaceData {
                id: "race_spring_sprint".into(),
                name: "Spring Sprint Trophy".into(),
                day_of_year: 45,
                entry_fee: 500,
                prize_pool: 3_000,
            },
            RaceData {
                id: "race_summer_endurance".into(),
                name: "Midsummer 500".into(),
                day_of_year: 180,
                entry_fee: 2_500,
                prize_pool: 18_000,
            },
            RaceData {
                id: "race_autumn_gp".into(),
                name: "Autumn Grand Prix".into(),
                day_of_year: 290,
                entry_fee: 10_000,
                prize_pool: 75_000,
            },
        ],
    };

    GameState {
        current_day: 1,
        time_speed: TimeSpeed::Paused,
        player: Player {
            age_days: 18 * 365,
            budget: 20_000,
            cars: vec![],
        },
        catalog,
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
fn tick_game_day(state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    game.current_day += 1;
    game.player.age_days += 1;

    let current_day = game.current_day;

    // Daily wear progression for owned vehicles
    for car in &mut game.player.cars {
        if current_day % 60 == 0 {
            car.needs_oil_change = true;
        }
        if current_day % 180 == 0 {
            car.needs_gearbox_maint = true;
        }
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
            let total_cost = game.player.cars[car_idx].tire_set_cost * sets as u64;
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

    let success = action.success_rate >= 0.50;
    let payout = if success { action.payout } else { 0 };

    game.player.budget += payout;

    let message = if success {
        format!("Successfully completed '{}' and earned £{}.", action.name, payout)
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

#[tauri::command]
fn enter_race(
    car_id: String,
    race_id: String,
    state: State<'_, AppState>,
) -> Result<RaceResult, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;

    let current_day_of_year = ((game.current_day - 1) % 365) + 1;

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

    game.player.cars[car_idx].needs_oil_change = true;

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
            enter_race
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
