use crate::engine::loader::{ActionData, CarData, GameCatalog};
use crate::engine::race::{enter_race, perform_maintenance, validate_race_entry, MaintenanceType, RaceResult};
use crate::engine::{Car, GameState, TimeSpeed};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_game_state(state: State<'_, AppState>) -> GameState {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_time_speed(speed: TimeSpeed, state: State<'_, AppState>) {
    let mut game = state.0.lock().unwrap();
    game.time_speed = speed;
}

#[tauri::command]
pub fn tick_game_day(state: State<'_, AppState>) -> GameState {
    let mut game = state.0.lock().unwrap();
    if game.time_speed != TimeSpeed::Paused {
        game.tick_day();
    }
    game.clone()
}

#[tauri::command]
pub fn get_catalog(state: State<'_, AppState>) -> GameCatalog {
    let game = state.0.lock().unwrap();
    game.catalog.clone()
}

#[tauri::command]
pub fn execute_action(action_id: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().unwrap();

    let action = game
        .catalog
        .actions
        .iter()
        .find(|a| a.id == action_id)
        .cloned()
        .ok_or_else(|| format!("Action '{action_id}' not found in catalog"))?;

    if game.player.budget < action.base_cost {
        return Err("Insufficient budget to execute action".into());
    }

    game.player.budget -= action.base_cost;

    let roll: f64 = rand::random();
    if roll <= action.success_rate {
        game.player.budget += action.payout;
    }

    Ok(game.clone())
}

#[tauri::command]
pub fn buy_car(car_id: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().unwrap();

    let car_data = game
        .catalog
        .cars
        .iter()
        .find(|c| c.id == car_id)
        .cloned()
        .ok_or_else(|| format!("Car '{car_id}' not found"))?;

    if game.player.budget < car_data.price {
        return Err("Cannot afford this car".into());
    }

    game.player.budget -= car_data.price;

    let owned_car = Car {
        id: car_data.id,
        name: car_data.name,
        price: car_data.price,
        engine_rebuild_cost: car_data.engine_rebuild_cost,
        gearbox_maint_cost: car_data.gearbox_maint_cost,
        oil_change_cost: car_data.oil_change_cost,
        tire_set_cost: car_data.tire_set_cost,
        needs_oil_change: false,
        needs_engine_rebuild: false,
        needs_gearbox_maint: false,
        tire_sets_available: 4,
    };

    game.player.cars.push(owned_car);
    Ok(game.clone())
}

#[tauri::command]
pub fn maintain_car_cmd(
    car_id: String,
    maintenance_type: MaintenanceType,
    state: State<'_, AppState>,
) -> Result<GameState, String> {
    let mut game = state.0.lock().unwrap();
    let player = &mut game.player;

    let car = player
        .cars
        .iter_mut()
        .find(|c| c.id == car_id)
        .ok_or_else(|| "Car not found in player garage".to_string())?;

    perform_maintenance(player, car, maintenance_type)?;
    Ok(game.clone())
}

#[tauri::command]
pub fn enter_race_cmd(
    car_id: String,
    race_id: String,
    state: State<'_, AppState>,
) -> Result<RaceResult, String> {
    let mut game = state.0.lock().unwrap();
    let current_day = game.current_day;

    let race = game
        .catalog
        .races
        .iter()
        .find(|r| r.id == race_id)
        .cloned()
        .ok_or_else(|| "Race event not found in catalog".to_string())?;

    let player = &mut game.player;
    let car = player
        .cars
        .iter_mut()
        .find(|c| c.id == car_id)
        .ok_or_else(|| "Selected car not found in garage".to_string())?;

    validate_race_entry(player, car, &race, current_day)?;

    let roll: f64 = rand::random();
    let result = enter_race(player, car, &race, roll)?;

    Ok(result)
}
