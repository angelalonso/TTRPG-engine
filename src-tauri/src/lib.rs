pub mod engine;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use engine::loader::{ActionData, EventData, GameCatalog};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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
    pub cost_5: String,
    pub cost_6: String,
    pub cost_7: String,
    pub cost_8: String,
    pub cost_9: String,
    pub cost_10: String,
    pub cost_11: String,
    pub cost_12: String,
    pub cost_13: String,
    pub cost_14: String,
    pub cost_15: String,
    pub service_1_needed: bool,
    pub service_2_needed: bool,
    pub service_3_needed: bool,
    pub service_4_needed: bool,
    pub service_1_interval_days: u32,
    pub service_2_interval_days: u32,
    pub service_3_interval_days: u32,
    pub service_4_interval_days: u32,
    pub service_5_interval_days: u32,
    pub service_6_interval_days: u32,
    pub service_7_interval_days: u32,
    pub service_8_interval_days: u32,
    pub service_9_interval_days: u32,
    pub service_10_interval_days: u32,
    pub service_11_interval_days: u32,
    pub service_12_interval_days: u32,
    pub service_13_interval_days: u32,
    pub service_14_interval_days: u32,
    pub service_15_interval_days: u32,
    pub resale_initial_percent: f64,
    pub resale_annual_percent: f64,
    pub resale_min_percent: f64,
    pub purchase_day: u32,
    pub description_html: String,
    #[serde(default)]
    pub license_level: u32,
    #[serde(default)]
    pub license_previous_id: String,
    #[serde(default)]
    pub requires_object_ids: String,
    #[serde(default)]
    pub license_fee: f64,
    #[serde(default)]
    pub unavailable_until_day: u32,
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
    pub characteristics: std::collections::HashMap<String, f64>,
    pub inventory: Vec<OwnedObject>,
    pub active_actions: Vec<ActiveAction>,
    #[serde(default)]
    pub last_action_day: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: Player,
    pub catalog: GameCatalog,
    pub dataset_path: String,
    pub time_speed: TimeSpeed,
    pub current_day: u32,
    pub days_per_year: u32,
    pub pending_alerts: Vec<GameAlert>,
    pub cost_ledger: Vec<CostOccurrence>,
    #[serde(default)]
    pub pending_events: Vec<PendingEvent>,
    #[serde(default)]
    pub event_history: Vec<EventHistory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingEvent {
    pub id: String,
    pub event_id: String,
    pub object_id: String,
    pub entered_day: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHistory {
    pub id: String,
    pub event_id: String,
    pub object_id: String,
    pub entered_day: u32,
    pub result: String,
    pub outcome: String,
    pub reward_awarded: f64,
    #[serde(default)]
    pub charisma_reward_awarded: f64,
    #[serde(default)]
    pub damage_type: String,
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
    pub charisma_reward_awarded: f64,
    pub message: String,
    pub damage_type: String,
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
    damage_type: Option<String>,
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

fn event_duration_days(event: &EventData) -> u32 {
    match event.duration_unit.trim().to_lowercase().as_str() {
        "day" | "days" => event.duration_value,
        "week" | "weeks" => event.duration_value.saturating_mul(7),
        "hour" | "hours" | "minute" | "minutes" => 0,
        _ => 0,
    }
}

fn label(catalog: &GameCatalog, key: &str, fallback: &str) -> String {
    catalog.labels.get(key, fallback)
}

fn config_u32(catalog: &GameCatalog, key: &str, fallback: u32) -> u32 {
    catalog
        .labels
        .values
        .get(key)
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(fallback)
}

fn config_f64(catalog: &GameCatalog, key: &str, fallback: f64) -> f64 {
    catalog
        .labels
        .values
        .get(key)
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(fallback)
}

fn weekday(day: u32) -> u32 {
    ((day - 1) % 7) + 1
}

fn characteristic_value(player: &Player, id: &str) -> f64 {
    player.characteristics.get(id).copied().unwrap_or_default()
}

fn adjust_characteristic(game: &mut GameState, id: &str, amount: f64) {
    let definition = game.catalog.player_characteristics.iter().find(|entry| entry.id == id);
    let value = (characteristic_value(&game.player, id) + amount).clamp(
        definition.map(|entry| entry.min_value).unwrap_or(f64::NEG_INFINITY),
        definition.map(|entry| entry.max_value).unwrap_or(f64::INFINITY),
    );
    game.player.characteristics.insert(id.to_string(), value);
}

fn initial_characteristics(catalog: &GameCatalog) -> std::collections::HashMap<String, f64> {
    catalog
        .player_characteristics
        .iter()
        .map(|characteristic| (characteristic.id.clone(), characteristic.value))
        .collect()
}

fn merge_characteristics(
    current: &mut std::collections::HashMap<String, f64>,
    definitions: &[engine::loader::PlayerCharacteristicData],
) {
    for characteristic in definitions {
        current
            .entry(characteristic.id.clone())
            .or_insert(characteristic.value);
    }
}

fn cost_id(object: &OwnedObject, service_type: ServiceType) -> Option<&str> {
    match service_type {
        ServiceType::Service1 => Some(&object.cost_1),
        ServiceType::Service2 => Some(&object.cost_2),
        ServiceType::Service3 => Some(&object.cost_3),
        ServiceType::Service4 => Some(&object.cost_4),
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

fn missing_object_prerequisites(game: &GameState, object: &engine::loader::ObjectData) -> Vec<String> {
    let owns = |id: &str| {
        game.player
            .inventory
            .iter()
            .any(|owned| owned.id == id || owned.id.starts_with(&format!("{id}_")))
    };
    let mut requirements: Vec<String> = object
        .requires_object_ids
        .split(';')
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .filter(|id| !owns(id))
        .map(str::to_string)
        .collect();
    if !object.license_previous_id.trim().is_empty()
        && !owns(&object.license_previous_id)
    {
        requirements.push(object.license_previous_id.clone());
    }
    requirements
}

fn mark_object_service_needed(
    game: &mut GameState,
    object_id: &str,
    cost_id: &str,
) -> Result<(), String> {
    let object = game
        .player
        .inventory
        .iter_mut()
        .find(|object| object.id == object_id)
        .ok_or_else(|| format!("Object '{}' not found for service cost", object_id))?;
    if object.cost_1 == cost_id {
        object.service_1_needed = true;
    } else if object.cost_2 == cost_id {
        object.service_2_needed = true;
    } else if object.cost_3 == cost_id {
        object.service_3_needed = true;
    } else if object.cost_4 == cost_id {
        object.service_4_needed = true;
    } else {
        return Err(format!(
            "Service cost '{}' is not mapped to one of the object's service slots",
            cost_id
        ));
    }
    Ok(())
}

fn object_requirement_error(
    game: &GameState,
    object: &OwnedObject,
) -> Option<String> {
    let mut requirements = Vec::new();
    if object.unavailable_until_day > game.current_day {
        requirements.push(format!(
            "unavailable for {} more day(s)",
            object.unavailable_until_day - game.current_day
        ));
    }
    for (needed, cost_id) in [
        (object.service_1_needed, object.cost_1.as_str()),
        (object.service_2_needed, object.cost_2.as_str()),
        (object.service_3_needed, object.cost_3.as_str()),
        (object.service_4_needed, object.cost_4.as_str()),
    ] {
        if needed {
            let cost_name = game
                .catalog
                .costs
                .iter()
                .find(|cost| cost.id == cost_id)
                .map(|cost| cost.name.as_str())
                .unwrap_or(cost_id);
            requirements.push(format!("service required: {}", cost_name));
        }
    }
    if requirements.is_empty() {
        None
    } else {
        Some(requirements.join("; "))
    }
}

fn normalized(value: &str) -> String {
    value.trim().to_lowercase()
}

fn format_cost_message(
    template: &str,
    cost_name: &str,
    object_id: &str,
    currency: &str,
    amount: f64,
) -> String {
    template
        .replace("{cost_name}", cost_name)
        .replace("{object_id}", object_id)
        .replace("{currency}", currency)
        .replace("{amount}", &format!("{amount:.2}"))
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
            "inventory_count" => Some(game.player.inventory.len().to_string()),
            characteristic => game
                .player
                .characteristics
                .get(characteristic)
                .map(ToString::to_string),
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
    if !rule.damage_type.trim().is_empty()
        && normalized(&rule.damage_type)
            != normalized(context.damage_type.as_deref().unwrap_or_default())
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
        let object_scoped = normalized(&rule.trigger_type) == "day_elapsed"
            && game.catalog.cost_conditions.iter().any(|condition| {
                condition.rule_id == rule.id && normalized(&condition.subject_type) == "object"
            });
        let contexts: Vec<TriggerContext> = if object_scoped {
            game.player
                .inventory
                .iter()
                .map(|object| TriggerContext {
                    source_type: "object".into(),
                    source_id: object.id.clone(),
                    object_type: Some(object.object_type.clone()),
                    ..context.clone()
                })
                .collect()
        } else {
            vec![context.clone()]
        };

        for rule_context in contexts {
            if !rule_matches(game, &rule, &rule_context) {
                continue;
            }
            if normalized(&rule.trigger_type) == "day_elapsed"
                && (rule.interval_days == 0 || day % rule.interval_days != 0)
            {
                continue;
            }
            if rule.probability <= 0.0
                || rng.random::<f64>() > rule.probability.clamp(0.0, 1.0)
            {
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
            let object_service = normalized(&rule.resolution_mode) == "object_service";
            if object_service {
                mark_object_service_needed(
                    game,
                    &rule_context.source_id,
                    &rule.cost_id,
                )?;
            }
            if rule.unavailable_days > 0 && !rule_context.source_id.is_empty() {
                if let Some(object) = game
                    .player
                    .inventory
                    .iter_mut()
                    .find(|object| object.id == rule_context.source_id)
                {
                    object.unavailable_until_day = day.saturating_add(rule.unavailable_days);
                }
            }
            let immediate = normalized(&rule.charge_mode) == "immediate";
            let charged = !object_service
                && immediate
                && characteristic_value(&game.player, "budget") >= amount;
            if charged {
                adjust_characteristic(&mut *game, "budget", -amount);
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
                source_type: if object_service {
                    "object_service".into()
                } else {
                    rule_context.source_type.clone()
                },
                source_id: rule_context.source_id.clone(),
            });

            let cost_name = game
                .catalog
                .costs
                .iter()
                .find(|cost| cost.id == rule.cost_id)
                .map(|cost| cost.name.clone())
                .unwrap_or_else(|| rule.cost_id.clone());
            let currency = label(&game.catalog, "currency_symbol", "$");
            let custom_message = rule
                .message
                .replace("{cost_name}", &cost_name)
                .replace("{object_id}", &rule_context.source_id)
                .replace("{currency}", &currency)
                .replace("{amount}", &format!("{amount:.2}"));
            game.pending_alerts.push(GameAlert {
                id: format!("alert_{}", occurrence_id),
                title: if object_service {
                    "Service Required".into()
                } else if charged {
                    "Cost Applied".into()
                } else {
                    "Cost Pending".into()
                },
                message: if !custom_message.trim().is_empty() {
                    custom_message
                } else if charged {
                    format!("{} cost of {}{:.2} was applied.", cost_name, currency, amount)
                } else if object_service {
                    if rule.pending_message.trim().is_empty() {
                        format!(
                            "{} is required for {} and remains unpaid until completed in the garage.",
                            cost_name, rule_context.source_id
                        )
                    } else {
                        format_cost_message(
                            &rule.pending_message,
                            &cost_name,
                            &rule_context.source_id,
                            &currency,
                            amount,
                        )
                    }
                } else {
                    format!(
                        "{} cost of {}{:.2} is pending until sufficient funds are available.",
                        cost_name, currency, amount
                    )
                },
            });
        }
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
    let catalog = GameCatalog::load_from_directory(&dataset_path);
    GameState {
        current_day: 1,
        days_per_year: config_u32(&catalog, "days_per_year", 365).max(1),
        time_speed: TimeSpeed::Paused,
        player: Player {
            age_days: config_u32(&catalog, "starting_age_days", 0),
            characteristics: initial_characteristics(&catalog),
            inventory: vec![],
            active_actions: vec![],
            last_action_day: None,
        },
        catalog,
        dataset_path,
        pending_alerts: vec![],
        cost_ledger: vec![],
        pending_events: vec![],
        event_history: vec![],
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

fn save_file_path(dataset_path: &str) -> Result<PathBuf, String> {
    let dataset = Path::new(dataset_path)
        .canonicalize()
        .map_err(|error| format!("Dataset folder cannot be resolved: {error}"))?;
    if !dataset.is_dir() {
        return Err("Dataset path is not a folder".into());
    }
    Ok(dataset.join("saves").join("savegame.json"))
}

#[tauri::command]
fn save_game(state: State<'_, AppState>) -> Result<String, String> {
    let game = state.0.lock().map_err(|e| e.to_string())?.clone();
    let path = save_file_path(&game.dataset_path)?;
    let parent = path.parent().ok_or_else(|| "Invalid save path".to_string())?;
    std::fs::create_dir_all(parent).map_err(|error| format!("Cannot create save folder: {error}"))?;
    let data = serde_json::to_vec_pretty(&game).map_err(|error| format!("Cannot encode save: {error}"))?;
    std::fs::write(&path, data)
        .map_err(|error| format!("Cannot write save: {error}"))?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
fn load_game(state: State<'_, AppState>) -> Result<GameState, String> {
    let current_path = state.0.lock().map_err(|e| e.to_string())?.dataset_path.clone();
    let path = save_file_path(&current_path)?;
    let data = std::fs::read(&path).map_err(|error| format!("Cannot read save: {error}"))?;
    let loaded: GameState = serde_json::from_slice(&data).map_err(|error| format!("Cannot decode save: {error}"))?;
    let current_canonical = Path::new(&current_path)
        .canonicalize()
        .map_err(|error| format!("Dataset folder cannot be resolved: {error}"))?;
    let saved_canonical = Path::new(&loaded.dataset_path)
        .canonicalize()
        .map_err(|error| format!("Save belongs to an unavailable dataset: {error}"))?;
    if current_canonical != saved_canonical {
        return Err("This save belongs to a different dataset".into());
    }
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    *game = loaded.clone();
    Ok(loaded)
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
    if game.cost_ledger[index].source_type == "object_service" {
        return Err("This cost must be completed through the object's service action".into());
    }
    let amount = game.cost_ledger[index].amount;
    if characteristic_value(&game.player, "budget") < amount {
        return Err("Insufficient funds to pay pending cost".into());
    }
    adjust_characteristic(&mut *game, "budget", -amount);
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
    let characteristic_definitions = game.catalog.player_characteristics.clone();
    merge_characteristics(
        &mut game.player.characteristics,
        &characteristic_definitions,
    );
    game.days_per_year = config_u32(&game.catalog, "days_per_year", 365).max(1);
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
    advance_one_day(&mut game)?;
    Ok(game.clone())
}

fn advance_one_day(game: &mut GameState) -> Result<(), String> {
    let previous_day = game.current_day;
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
    evaluate_cost_rules(game, &daily_context, current_day)?;

    for object in &mut game.player.inventory {
        if object.service_1_interval_days > 0
            && current_day % object.service_1_interval_days == 0
        {
            object.service_1_needed = true;
        }
        if object.service_2_interval_days > 0
            && current_day % object.service_2_interval_days == 0
        {
            object.service_2_needed = true;
        }
        if object.service_3_interval_days > 0
            && current_day % object.service_3_interval_days == 0
        {
            object.service_3_needed = true;
        }
        if object.service_4_interval_days > 0
            && current_day % object.service_4_interval_days == 0
        {
            object.service_4_needed = true;
        }
    }

    let had_action = game.player.last_action_day == Some(previous_day);
    let mut daily_stamina_use = 0.0;
    for active in &game.player.active_actions {
        if let Some(action) = game.catalog.actions.iter().find(|a| a.id == active.action_id) {
            daily_stamina_use += action.stamina_cost.max(0.0);
        }
    }
    if daily_stamina_use > 0.0 {
        adjust_characteristic(&mut *game, "stamina", -daily_stamina_use);
    }
    if !had_action {
        let recovery = if matches!(weekday(current_day), 6 | 7) {
            config_f64(&game.catalog, "weekend_stamina_recovery", 1.0)
        } else {
            config_f64(&game.catalog, "daily_stamina_recovery", 1.0)
        };
        adjust_characteristic(&mut *game, "stamina", recovery);
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
    adjust_characteristic(&mut *game, "budget", total_payout);

    let day_of_year = ((current_day - 1) % game.days_per_year) + 1;
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
    Ok(())
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
    let acquisition_cost = if object.object_type == "license" && object.license_fee > 0.0 {
        object.license_fee
    } else {
        object.price
    };
    if characteristic_value(&game.player, "budget") < acquisition_cost {
        return Err("Insufficient funds to acquire object".into());
    }
    let missing = missing_object_prerequisites(&game, &object);
    if !missing.is_empty() {
        return Err(format!(
            "Cannot acquire '{}': required object(s) missing: {}",
            object.name,
            missing.join(", ")
        ));
    }

    adjust_characteristic(&mut *game, "budget", -acquisition_cost);
    let owned = OwnedObject {
        id: format!("{}_{}", object.id, game.player.inventory.len() + 1),
        object_type: object.object_type.clone(),
        name: object.name,
        price: object.price,
        cost_1: object.cost_1,
        cost_2: object.cost_2,
        cost_3: object.cost_3,
        cost_4: object.cost_4,
        cost_5: object.cost_5,
        cost_6: object.cost_6,
        cost_7: object.cost_7,
        cost_8: object.cost_8,
        cost_9: object.cost_9,
        cost_10: object.cost_10,
        cost_11: object.cost_11,
        cost_12: object.cost_12,
        cost_13: object.cost_13,
        cost_14: object.cost_14,
        cost_15: object.cost_15,
        service_1_needed: false,
        service_2_needed: false,
        service_3_needed: false,
        service_4_needed: false,
        service_1_interval_days: object.service_1_interval_days,
        service_2_interval_days: object.service_2_interval_days,
        service_3_interval_days: object.service_3_interval_days,
        service_4_interval_days: object.service_4_interval_days,
        service_5_interval_days: object.service_5_interval_days,
        service_6_interval_days: object.service_6_interval_days,
        service_7_interval_days: object.service_7_interval_days,
        service_8_interval_days: object.service_8_interval_days,
        service_9_interval_days: object.service_9_interval_days,
        service_10_interval_days: object.service_10_interval_days,
        service_11_interval_days: object.service_11_interval_days,
        service_12_interval_days: object.service_12_interval_days,
        service_13_interval_days: object.service_13_interval_days,
        service_14_interval_days: object.service_14_interval_days,
        service_15_interval_days: object.service_15_interval_days,
        resale_initial_percent: object.resale_initial_percent,
        resale_annual_percent: object.resale_annual_percent,
        resale_min_percent: object.resale_min_percent,
        purchase_day: game.current_day,
        description_html: object.description_html,
        license_level: object.license_level,
        license_previous_id: object.license_previous_id,
        requires_object_ids: object.requires_object_ids,
        license_fee: object.license_fee,
        unavailable_until_day: 0,
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
    };
    if matches!(service_type, ServiceType::Service1 | ServiceType::Service2 | ServiceType::Service3 | ServiceType::Service4)
        && !needed
    {
        return Err("That service is not currently required".into());
    }
    let pending_occurrence = game.cost_ledger.iter().position(|occurrence| {
        occurrence.status == "pending"
            && occurrence.source_type == "object_service"
            && occurrence.source_id == object_id
            && occurrence.cost_id == cost_id(
                &game.player.inventory[index],
                service_type,
            )
            .unwrap_or_default()
    });
    let payable_cost = pending_occurrence
        .map(|occurrence| game.cost_ledger[occurrence].amount)
        .unwrap_or(cost);
    if characteristic_value(&game.player, "budget") < payable_cost {
        return Err("Insufficient funds for service".into());
    }
    adjust_characteristic(&mut *game, "budget", -payable_cost);
    if let Some(occurrence) = pending_occurrence {
        game.cost_ledger[occurrence].status = "charged".into();
    }
    let object = &mut game.player.inventory[index];
    match service_type {
        ServiceType::Service1 => object.service_1_needed = false,
        ServiceType::Service2 => object.service_2_needed = false,
        ServiceType::Service3 => object.service_3_needed = false,
        ServiceType::Service4 => object.service_4_needed = false,
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
    if characteristic_value(&game.player, "budget") < action.base_cost {
        return Err("Insufficient funds to start action".into());
    }
    if characteristic_value(&game.player, "stamina") < action.stamina_cost {
        return Err("Not enough stamina to start action".into());
    }
    if action.payout_freq_type.eq_ignore_ascii_case("recurring")
        && game.player.active_actions.iter().any(|active| active.action_id == action.id)
    {
        return Err("This action is already active".into());
    }

    adjust_characteristic(&mut *game, "budget", -action.base_cost);
    adjust_characteristic(&mut *game, "stamina", -action.stamina_cost);
    game.player.last_action_day = Some(game.current_day);
    let mut rng = rand::rng();
    let success = rng.random::<f64>() <= action.success_rate;
    let payout = if success && !action.payout_freq_type.eq_ignore_ascii_case("recurring") {
        action.payout
    } else {
        0.0
    };
    adjust_characteristic(&mut *game, "budget", payout);
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
            if action.payout_freq_type.eq_ignore_ascii_case("recurring") {
                format!(
                    "Started '{}'. It pays {} every {}.",
                    action.name, action.payout, action.payout_freq_unit
                )
            } else {
                format!("Completed '{}'.", action.name)
            }
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
) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    let day_of_year = ((game.current_day - 1) % game.days_per_year) + 1;
    if !matches!(weekday(day_of_year), 5 | 6 | 7) {
        return Err("Events can only be scheduled on days 5, 6, or 7 of the week".into());
    }
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

    if characteristic_value(&game.player, "budget") < event.entry_fee {
        return Err("Insufficient funds for event entry".into());
    }
    if !event.required_license_id.trim().is_empty()
        && !game.player.inventory.iter().any(|object| {
            object.id == event.required_license_id
                || object.id.starts_with(&format!("{}_", event.required_license_id))
        })
    {
        return Err(format!(
            "Cannot enter '{}': required licence '{}' is missing.",
            event.name, event.required_license_id
        ));
    }
    let index = game
        .player
        .inventory
        .iter()
        .position(|object| object.id == object_id)
        .ok_or_else(|| "Object not found in inventory".to_string())?;
    if let Some(requirements) = object_requirement_error(
        &game,
        &game.player.inventory[index],
    ) {
        return Err(format!("Cannot enter '{}': {}.", event.name, requirements));
    }

    let entry_id = format!("event_entry_{}_{}", event.id, game.current_day);
    if game.pending_events.iter().any(|entry| entry.id == entry_id)
        || game.event_history.iter().any(|entry| entry.event_id == event.id && entry.entered_day == game.current_day)
    {
        return Err("This event has already been entered today".into());
    }
    adjust_characteristic(&mut *game, "budget", -event.entry_fee);
    let entered_day = game.current_day;
    let duration_days = event_duration_days(&event);
    game.pending_events.push(PendingEvent {
        id: entry_id,
        event_id: event.id,
        object_id,
        entered_day,
    });
    for _ in 0..duration_days {
        advance_one_day(&mut game)?;
    }
    Ok(game.clone())
}

#[tauri::command]
fn sell_object(object_id: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    let index = game
        .player
        .inventory
        .iter()
        .position(|object| object.id == object_id)
        .ok_or_else(|| "Object not found in inventory".to_string())?;
    let (price, purchase_day, initial, annual, minimum) = {
        let object = &game.player.inventory[index];
        (
            object.price,
            object.purchase_day,
            object.resale_initial_percent.clamp(0.0, 1.0),
            object.resale_annual_percent.clamp(0.0, 1.0),
            object.resale_min_percent.clamp(0.0, 1.0),
        )
    };
    let years_owned = game.current_day.saturating_sub(purchase_day) / game.days_per_year;
    let value = price * (initial * annual.powi(years_owned as i32)).max(minimum);
    adjust_characteristic(&mut *game, "budget", value);
    game.player.inventory.remove(index);
    Ok(game.clone())
}

#[tauri::command]
fn submit_event_result(
    entry_id: String,
    result: String,
    damage_type: String,
    state: State<'_, AppState>,
) -> Result<EventResult, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    let result = result.trim().to_string();
    let damage_type = damage_type.trim().to_string();
    if result.is_empty() {
        return Err("Enter an event result before submitting".into());
    }
    let entry_index = game
        .pending_events
        .iter()
        .position(|entry| entry.id == entry_id)
        .ok_or_else(|| "Pending event entry not found".to_string())?;
    let entry = game.pending_events.remove(entry_index);
    let event = game
        .catalog
        .events
        .iter()
        .find(|event| event.id == entry.event_id)
        .cloned()
        .ok_or_else(|| "Event not found in catalog".to_string())?;
    let success = matches!(
        normalized(&result).as_str(),
        "success" | "successful" | "win" | "won" | "1" | "yes" | "true"
    );
    let reward = if success { event.reward_pool } else { 0.0 };
    let charisma_reward = if success { event.charisma_reward } else { 0.0 };
    adjust_characteristic(&mut *game, "budget", reward);
    adjust_characteristic(&mut *game, "charisma", charisma_reward);
    let object_type = game
        .player
        .inventory
        .iter()
        .find(|object| object.id == entry.object_id)
        .map(|object| object.object_type.clone());
    let event_context = TriggerContext {
        trigger_type: "event_completed".into(),
        trigger_ref: event.id.clone(),
        source_type: "event".into(),
        source_id: entry.object_id.clone(),
        outcome: Some(if success { "success" } else { "failure" }.into()),
        tags: event_tags(&event),
        object_type,
        damage_type: Some(damage_type.clone()),
    };
    let current_day = game.current_day;
    evaluate_cost_rules(&mut game, &event_context, current_day)?;
    game.event_history.push(EventHistory {
        id: entry.id,
        event_id: event.id.clone(),
        object_id: entry.object_id,
        entered_day: entry.entered_day,
        result: result.clone(),
        outcome: if success { "Success".into() } else { "Unsuccessful".into() },
        reward_awarded: reward,
        charisma_reward_awarded: charisma_reward,
        damage_type: damage_type.clone(),
    });

    Ok(EventResult {
        event_name: event.name,
        outcome: if success { "Success".into() } else { "Unsuccessful".into() },
        entry_fee_paid: event.entry_fee,
        reward_awarded: reward,
        charisma_reward_awarded: charisma_reward,
        message: if success {
            format!(
                "The event was successful: {}. Rewards: {} budget and {} charisma.",
                result, reward, charisma_reward
            )
        } else {
            format!("The event ended without a reward: {}.", result)
        },
        damage_type,
    })
}

#[tauri::command]
fn load_description(path: String, state: State<'_, AppState>) -> Result<String, String> {
    let game = state.0.lock().map_err(|e| e.to_string())?;
    let base = std::path::Path::new(&game.dataset_path);
    let requested = base.join(&path);
    let canonical_base = base
        .canonicalize()
        .map_err(|error| format!("Dataset folder cannot be read: {}", error))?;
    let canonical_file = requested
        .canonicalize()
        .map_err(|error| format!("Description file cannot be read: {}", error))?;
    if !canonical_file.starts_with(&canonical_base) {
        return Err("Description path must remain inside the dataset folder".into());
    }
    let html = std::fs::read_to_string(&canonical_file)
        .map_err(|error| format!("Description file cannot be read: {}", error))?;
    embed_description_assets(&html, &canonical_file, &canonical_base)
}

fn embed_description_assets(
    html: &str,
    description_file: &std::path::Path,
    dataset_base: &std::path::Path,
) -> Result<String, String> {
    let mut output = String::with_capacity(html.len());
    let mut cursor = 0;

    while let Some(relative_start) = html[cursor..].find("src=") {
        let start = cursor + relative_start;
        output.push_str(&html[cursor..start + 4]);

        let quote = html.as_bytes().get(start + 4).copied().unwrap_or_default() as char;
        if quote != '"' && quote != '\'' {
            cursor = start + 4;
            continue;
        }

        let value_start = start + 5;
        let Some(relative_end) = html[value_start..].find(quote) else {
            output.push_str(&html[start + 4..]);
            return Ok(output);
        };
        let value_end = value_start + relative_end;
        let source = &html[value_start..value_end];

        if source.starts_with("data:")
            || source.starts_with("http://")
            || source.starts_with("https://")
            || source.starts_with("//")
            || source.starts_with('#')
        {
            output.push(quote);
            output.push_str(source);
            output.push(quote);
            cursor = value_end + 1;
            continue;
        }

        let description_relative = description_file
            .parent()
            .unwrap_or(dataset_base)
            .join(source);
        let asset_path = if source.starts_with("dataset/") {
            dataset_base.join(source.trim_start_matches("dataset/"))
        } else {
            description_relative
        };
        let canonical_asset = asset_path
            .canonicalize()
            .map_err(|error| format!("Description asset cannot be read ({}): {}", source, error))?;
        if !canonical_asset.starts_with(dataset_base) {
            return Err("Description asset path must remain inside the dataset folder".into());
        }
        let bytes = std::fs::read(&canonical_asset)
            .map_err(|error| format!("Description asset cannot be read ({}): {}", source, error))?;
        let mime = match canonical_asset
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            _ => "application/octet-stream",
        };
        let data_url = format!("data:{mime};base64,{}", BASE64.encode(bytes));
        output.push(quote);
        output.push_str(&data_url);
        output.push(quote);
        cursor = value_end + 1;
    }

    output.push_str(&html[cursor..]);
    Ok(output)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState(Mutex::new(create_initial_state())))
        .invoke_handler(tauri::generate_handler![
            get_game_state,
            get_catalog,
            save_game,
            load_game,
            set_time_speed,
            tick_game_day,
            pay_cost,
            buy_object,
            service_object,
            sell_object,
            perform_action,
            enter_event,
            submit_event_result,
            load_description,
            dismiss_alert,
            reload_dataset
        ])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
