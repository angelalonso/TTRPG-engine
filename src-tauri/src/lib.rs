pub mod engine;

use engine::loader::{ActionData, EventData, GameCatalog};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeSpeed {
    Paused,
    OneDayEveryFiveSec,
    OneDayPerSec,
    OneWeekPerSec,
    RealTime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ServiceType {
    Service1,
    Service2,
    Service3,
    Service4,
    BuyUnits(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedObject {
    pub id: String,
    pub object_type: String,
    pub name: String,
    pub price: f64,
    pub cost_1: String,
    pub cost_2: String,
    pub cost_3: String,
    pub cost_4: String,
    pub service_1_needed: bool,
    pub service_2_needed: bool,
    pub service_3_needed: bool,
    pub service_4_needed: bool,
    pub units_available: u32,
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
    pub inventory: Vec<OwnedObject>,
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
    pub cost_ledger: Vec<CostOccurrence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOccurrence {
    pub id: String,
    pub cost_id: String,
    pub rule_id: String,
    pub amount: f64,
    pub created_day: u32,
    pub due_day: u32,
    pub status: String,
    pub source_type: String,
    pub source_id: String,
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
pub struct EventResult {
    pub event_name: String,
    pub outcome: String,
    pub entry_fee_paid: f64,
    pub reward_awarded: f64,
    pub message: String,
}

pub struct AppState(pub Mutex<GameState>);

#[derive(Debug, Clone, Default)]
struct TriggerContext {
    trigger_type: String,
    trigger_ref: String,
    source_type: String,
    source_id: String,
    outcome: Option<String>,
    tags: Vec<String>,
    object_type: Option<String>,
}

fn calculate_interval_days(freq: u32, unit: &str) -> u32 {
    let multiplier = match unit.trim().to_lowercase().as_str() {
        "day" | "days" => 1,
        "month" | "months" => 30,
        "year" | "years" => 365,
        _ => 1,
    };
    freq * multiplier
}

fn label(catalog: &GameCatalog, key: &str, fallback: &str) -> String {
    catalog.labels.get(key, fallback)
}

fn cost_id(object: &OwnedObject, service_type: ServiceType) -> Option<&str> {
    match service_type {
        ServiceType::Service1 => Some(&object.cost_1),
        ServiceType::Service2 => Some(&object.cost_2),
        ServiceType::Service3 => Some(&object.cost_3),
        ServiceType::Service4 | ServiceType::BuyUnits(_) => Some(&object.cost_4),
    }
}

fn cost_amount(catalog: &GameCatalog, object: &OwnedObject, service_type: ServiceType) -> Result<f64, String> {
    let id = cost_id(object, service_type).ok_or_else(|| "Cost reference is missing".to_string())?;
    catalog
        .costs
        .iter()
        .find(|cost| cost.id == id)
        .map(|cost| cost.amount)
        .ok_or_else(|| format!("Cost '{}' not found in costs.csv", id))
}

fn normalized(value: &str) -> String {
    value.trim().to_lowercase()
}

fn matches_value(actual: &str, operator: &str, expected: &str) -> bool {
    match normalized(operator).as_str() {
        "equals" | "eq" => normalized(actual) == normalized(expected),
        "not_equals" | "neq" => normalized(actual) != normalized(expected),
        "contains" => normalized(actual).contains(&normalized(expected)),
        "starts_with" => normalized(actual).starts_with(&normalized(expected)),
        "greater_than" => actual.parse::<f64>().unwrap_or_default() > expected.parse::<f64>().unwrap_or_default(),
        "less_than" => actual.parse::<f64>().unwrap_or_default() < expected.parse::<f64>().unwrap_or_default(),
        _ => false,
    }
}

fn condition_value(game: &GameState, context: &TriggerContext, subject_type: &str, subject_ref: &str) -> Option<String> {
    match normalized(subject_type).as_str() {
        "event" => match normalized(subject_ref).as_str() {
            "id" => Some(context.trigger_ref.clone()),
            "outcome" => context.outcome.clone(),
            "tag" => Some(context.tags.join(",")),
            _ => None,
        },
        "action" => match normalized(subject_ref).as_str() {
            "id" => Some(context.trigger_ref.clone()),
            "outcome" => context.outcome.clone(),
            _ => None,
        },
        "object" => match normalized(subject_ref).as_str() {
            "id" => Some(context.source_id.clone()),
            "type" => context.object_type.clone(),
            _ => None,
        },
        "player" => match normalized(subject_ref).as_str() {
            "budget" => Some(game.player.budget.to_string()),
            "inventory_count" => Some(game.player.inventory.len().to_string()),
            _ => None,
        },
        _ => None,
    }
}

fn rule_matches(game: &GameState, rule: &engine::loader::CostRule, context: &TriggerContext) -> bool {
    let trigger_type = normalized(&rule.trigger_type);
    let context_type = normalized(&context.trigger_type);
    if trigger_type != context_type {
        return false;
    }

    let reference = normalized(&rule.trigger_ref);
    if !reference.is_empty()
        && reference != normalized(&context.trigger_ref)
        && !context.tags.iter().any(|tag| normalized(tag) == reference)
    {
        return false;
    }

    game.catalog
        .cost_conditions
        .iter()
        .filter(|condition| condition.rule_id == rule.id)
        .all(|condition| {
            condition_value(
                game,
                context,
                &condition.subject_type,
                &condition.subject_ref,
            )
            .is_some_and(|actual| matches_value(&actual, &condition.operator, &condition.value))
        })
}

fn evaluate_cost_rules(
    game: &mut GameState,
    context: &TriggerContext,
    day: u32,
) -> Result<(), String> {
    let rules = game.catalog.cost_rules.clone();
    let mut rng = rand::rng();

    for rule in rules {
        if !rule_matches(game, &rule, context) {
            continue;
        }
        if normalized(&rule.trigger_type) == "day_elapsed"
            && (rule.interval_days == 0 || day % rule.interval_days != 0)
        {
            continue;
        }
        if rule.probability <= 0.0 || rng.random::<f64>() > rule.probability.clamp(0.0, 1.0) {
            continue;
        }

        let base_amount = game
            .catalog
            .costs
            .iter()
            .find(|cost| cost.id == rule.cost_id)
            .map(|cost| cost.amount)
            .ok_or_else(|| format!("Cost '{}' not found in costs.csv", rule.cost_id))?;
        let amount = base_amount * rule.amount_multiplier;
        let immediate = normalized(&rule.charge_mode) == "immediate";
        let charged = immediate && game.player.budget >= amount;
        if charged {
            game.player.budget -= amount;
        }

        let occurrence_id = format!("cost_{}_{}_{}", rule.id, day, game.cost_ledger.len());
        let status = if charged { "charged" } else { "pending" };
        game.cost_ledger.push(CostOccurrence {
            id: occurrence_id.clone(),
            cost_id: rule.cost_id.clone(),
            rule_id: rule.id.clone(),
            amount,
            created_day: day,
            due_day: if charged { day } else { day.saturating_add(1) },
            status: status.into(),
            source_type: context.source_type.clone(),
            source_id: context.source_id.clone(),
        });

        let cost_name = game
            .catalog
            .costs
            .iter()
            .find(|cost| cost.id == rule.cost_id)
            .map(|cost| cost.name.clone())
            .unwrap_or_else(|| rule.cost_id.clone());
        let currency = label(&game.catalog, "currency_symbol", "$");
        game.pending_alerts.push(GameAlert {
            id: format!("alert_{}", occurrence_id),
            title: if charged { "Cost Applied".into() } else { "Cost Pending".into() },
            message: if charged {
                format!("{} cost of {}{:.2} was applied.", cost_name, currency, amount)
            } else {
                format!(
                    "{} cost of {}{:.2} is pending until sufficient funds are available.",
                    cost_name, currency, amount
                )
            },
        });
    }
    Ok(())
}

fn event_tags(event: &EventData) -> Vec<String> {
    event
        .tags
        .split(';')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(str::to_string)
        .collect()
}

fn create_initial_state() -> GameState {
    let dataset_path = "dataset".to_string();
    GameState {
        current_day: 1,
        time_speed: TimeSpeed::Paused,
        player: Player {
            age_days: 18 * 365,
            budget: 20_000.0,
            inventory: vec![],
            active_actions: vec![],
        },
        catalog: GameCatalog::load_from_directory(&dataset_path),
        dataset_path,
        pending_alerts: vec![],
        cost_ledger: vec![],
    }
}

#[tauri::command]
fn get_game_state(state: State<'_, AppState>) -> Result<GameState, String> {
    Ok(state.0.lock().map_err(|e| e.to_string())?.clone())
}

#[tauri::command]
fn get_catalog(state: State<'_, AppState>) -> Result<GameCatalog, String> {
    Ok(state.0.lock().map_err(|e| e.to_string())?.catalog.clone())
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
    game.pending_alerts.retain(|alert| alert.id != alert_id);
    Ok(game.clone())
}

#[tauri::command]
fn pay_cost(cost_occurrence_id: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    let index = game
        .cost_ledger
        .iter()
        .position(|occurrence| occurrence.id == cost_occurrence_id)
        .ok_or_else(|| "Cost occurrence not found".to_string())?;
    if game.cost_ledger[index].status != "pending" {
        return Err("Cost has already been settled".into());
    }
    let amount = game.cost_ledger[index].amount;
    if game.player.budget < amount {
        return Err("Insufficient funds to pay pending cost".into());
    }
    game.player.budget -= amount;
    game.cost_ledger[index].status = "charged".into();
    let currency = label(&game.catalog, "currency_symbol", "$");
    game.pending_alerts.push(GameAlert {
        id: format!("paid_{}", cost_occurrence_id),
        title: "Pending Cost Paid".into(),
        message: format!("Paid pending cost of {}{:.2}.", currency, amount),
    });
    Ok(game.clone())
}

#[tauri::command]
fn reload_dataset(new_path: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    game.catalog = GameCatalog::load_from_directory(&new_path);
    game.dataset_path = new_path.clone();
    let current_day = game.current_day;
    game.pending_alerts.push(GameAlert {
        id: format!("dataset_reload_{}", current_day),
        title: "Dataset Reloaded".into(),
        message: format!("Loaded game data from '{}'.", new_path),
    });
    Ok(game.clone())
}

#[tauri::command]
fn tick_game_day(state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    if game.time_speed == TimeSpeed::Paused {
        return Ok(game.clone());
    }

    game.current_day += 1;
    game.player.age_days += 1;
    let current_day = game.current_day;

    let daily_context = TriggerContext {
        trigger_type: "day_elapsed".into(),
        trigger_ref: String::new(),
        source_type: "day".into(),
        source_id: current_day.to_string(),
        ..TriggerContext::default()
    };
    evaluate_cost_rules(&mut game, &daily_context, current_day)?;

    if game.current_day % 60 == 0 {
        for object in &mut game.player.inventory {
            object.service_1_needed = true;
        }
    }
    if game.current_day % 180 == 0 {
        for object in &mut game.player.inventory {
            object.service_2_needed = true;
        }
    }

    let mut total_payout = 0.0;
    for active in &game.player.active_actions {
        if let Some(action) = game.catalog.actions.iter().find(|a| a.id == active.action_id) {
            if action.payout_freq_type.eq_ignore_ascii_case("recurring") {
                let interval = calculate_interval_days(action.payout_freq, &action.payout_freq_unit);
                let elapsed = current_day.saturating_sub(active.start_day);
                if interval > 0 && elapsed > 0 && elapsed % interval == 0 {
                    total_payout += action.payout;
                }
            }
        }
    }
    game.player.budget += total_payout;

    let day_of_year = ((current_day - 1) % 365) + 1;
    let events: Vec<EventData> = game
        .catalog
        .events
        .iter()
        .filter(|event| event.day_of_year == day_of_year)
        .cloned()
        .collect();
    for event in events {
        let event_title = format!("{} Today", label(&game.catalog, "event_name", "Event"));
        game.pending_alerts.push(GameAlert {
            id: format!("event_{}_{}", event.id, current_day),
            title: event_title,
            message: format!(
                "Today is day {}: '{}' is scheduled.",
                day_of_year, event.name
            ),
        });
    }
    if total_payout > 0.0 || !game.pending_alerts.is_empty() {
        game.time_speed = TimeSpeed::Paused;
    }
    Ok(game.clone())
}

#[tauri::command]
fn buy_object(object_id: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    let object = game
        .catalog
        .objects
        .iter()
        .find(|object| object.id == object_id)
        .cloned()
        .ok_or_else(|| "Object not found in catalog".to_string())?;
    if game.player.budget < object.price {
        return Err("Insufficient funds to acquire object".into());
    }

    game.player.budget -= object.price;
    let owned = OwnedObject {
        id: format!("{}_{}", object.id, game.player.inventory.len() + 1),
        object_type: object.object_type.clone(),
        name: object.name,
        price: object.price,
        cost_1: object.cost_1,
        cost_2: object.cost_2,
        cost_3: object.cost_3,
        cost_4: object.cost_4,
        service_1_needed: false,
        service_2_needed: false,
        service_3_needed: false,
        service_4_needed: false,
        units_available: object.units_available,
    };
    game.player.inventory.push(owned);
    let acquired_id = format!("{}_{}", object_id, game.player.inventory.len());
    let acquired_context = TriggerContext {
        trigger_type: "object_acquired".into(),
        trigger_ref: object_id,
        source_type: "object".into(),
        source_id: acquired_id,
        object_type: Some(object.object_type),
        ..TriggerContext::default()
    };
    let current_day = game.current_day;
    evaluate_cost_rules(&mut game, &acquired_context, current_day)?;
    Ok(game.clone())
}

#[tauri::command]
fn service_object(
    object_id: String,
    service_type: ServiceType,
    state: State<'_, AppState>,
) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    let index = game
        .player
        .inventory
        .iter()
        .position(|object| object.id == object_id)
        .ok_or_else(|| "Object not found in inventory".to_string())?;
    let (cost, needed) = match service_type {
        ServiceType::Service1 => {
            let object = &game.player.inventory[index];
            (cost_amount(&game.catalog, object, service_type)?, object.service_1_needed)
        }
        ServiceType::Service2 => {
            let object = &game.player.inventory[index];
            (cost_amount(&game.catalog, object, service_type)?, object.service_2_needed)
        }
        ServiceType::Service3 => {
            let object = &game.player.inventory[index];
            (cost_amount(&game.catalog, object, service_type)?, object.service_3_needed)
        }
        ServiceType::Service4 => {
            let object = &game.player.inventory[index];
            (cost_amount(&game.catalog, object, service_type)?, object.service_4_needed)
        }
        ServiceType::BuyUnits(units) => {
            let object = &game.player.inventory[index];
            (cost_amount(&game.catalog, object, service_type)? * units as f64, true)
        }
    };
    if matches!(service_type, ServiceType::Service1 | ServiceType::Service2 | ServiceType::Service3 | ServiceType::Service4)
        && !needed
    {
        return Err("That service is not currently required".into());
    }
    if game.player.budget < cost {
        return Err("Insufficient funds for service".into());
    }
    game.player.budget -= cost;
    let object = &mut game.player.inventory[index];
    match service_type {
        ServiceType::Service1 => object.service_1_needed = false,
        ServiceType::Service2 => object.service_2_needed = false,
        ServiceType::Service3 => object.service_3_needed = false,
        ServiceType::Service4 => object.service_4_needed = false,
        ServiceType::BuyUnits(units) => object.units_available += units,
    }
    Ok(game.clone())
}

#[tauri::command]
fn perform_action(action_id: String, state: State<'_, AppState>) -> Result<ActionResult, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    let action: ActionData = game
        .catalog
        .actions
        .iter()
        .find(|action| action.id == action_id)
        .cloned()
        .ok_or_else(|| "Action not found in catalog".to_string())?;
    if game.player.budget < action.base_cost {
        return Err("Insufficient funds to start action".into());
    }
    if action.payout_freq_type.eq_ignore_ascii_case("recurring")
        && game.player.active_actions.iter().any(|active| active.action_id == action.id)
    {
        return Err("This action is already active".into());
    }

    game.player.budget -= action.base_cost;
    let mut rng = rand::rng();
    let success = rng.random::<f64>() <= action.success_rate;
    let payout = if success && !action.payout_freq_type.eq_ignore_ascii_case("recurring") {
        action.payout
    } else {
        0.0
    };
    game.player.budget += payout;
    if success && action.payout_freq_type.eq_ignore_ascii_case("recurring") {
        let start_day = game.current_day;
        game.player.active_actions.push(ActiveAction {
            action_id: action.id.clone(),
            start_day,
        });
    }
    let action_context = TriggerContext {
        trigger_type: "action_completed".into(),
        trigger_ref: action.id.clone(),
        source_type: "action".into(),
        source_id: action.id.clone(),
        outcome: Some(if success { "success" } else { "failure" }.into()),
        ..TriggerContext::default()
    };
    let current_day = game.current_day;
    evaluate_cost_rules(&mut game, &action_context, current_day)?;

    Ok(ActionResult {
        action_name: action.name.clone(),
        success,
        payout_received: payout,
        cost_paid: action.base_cost,
        message: if success {
            format!("Completed '{}'.", action.name)
        } else {
            format!("Could not complete '{}'.", action.name)
        },
    })
}

#[tauri::command]
fn enter_event(
    object_id: String,
    event_id: String,
    state: State<'_, AppState>,
) -> Result<EventResult, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    let day_of_year = ((game.current_day - 1) % 365) + 1;
    let event = game
        .catalog
        .events
        .iter()
        .find(|event| event.id == event_id)
        .cloned()
        .ok_or_else(|| "Event not found in catalog".to_string())?;
    if event.day_of_year != day_of_year {
        return Err(format!("Event is scheduled for day {}, today is day {}.", event.day_of_year, day_of_year));
    }
    if game.player.budget < event.entry_fee {
        return Err("Insufficient funds for event entry".into());
    }
    let index = game
        .player
        .inventory
        .iter()
        .position(|object| object.id == object_id)
        .ok_or_else(|| "Object not found in inventory".to_string())?;
    let object_ready = {
        let object = &game.player.inventory[index];
        object.units_available >= event.object_units_required
            && !object.service_1_needed
            && !object.service_2_needed
            && !object.service_3_needed
            && !object.service_4_needed
    };
    if !object_ready
    {
        return Err("Object requires service or additional units before this event".into());
    }

    game.player.budget -= event.entry_fee;
    let object = &mut game.player.inventory[index];
    object.units_available -= event.object_units_required;
    object.service_1_needed = true;
    let mut rng = rand::rng();
    let success = rng.random::<f64>() > 0.2;
    let reward = if success { event.reward_pool } else { 0.0 };
    game.player.budget += reward;
    let event_context = TriggerContext {
        trigger_type: "event_completed".into(),
        trigger_ref: event.id.clone(),
        source_type: "event".into(),
        source_id: event.id.clone(),
        outcome: Some(if success { "success" } else { "failure" }.into()),
        tags: event_tags(&event),
        object_type: Some(game.player.inventory[index].object_type.clone()),
    };
    let current_day = game.current_day;
    evaluate_cost_rules(&mut game, &event_context, current_day)?;

    Ok(EventResult {
        event_name: event.name,
        outcome: if success { "Success".into() } else { "Unsuccessful".into() },
        entry_fee_paid: event.entry_fee,
        reward_awarded: reward,
        message: if success {
            "The event was successful.".into()
        } else {
            "The event ended without a reward.".into()
        },
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState(Mutex::new(create_initial_state())))
        .invoke_handler(tauri::generate_handler![
            get_game_state,
            get_catalog,
            set_time_speed,
            tick_game_day,
            pay_cost,
            buy_object,
            service_object,
            perform_action,
            enter_event,
            dismiss_alert,
            reload_dataset
        ])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
