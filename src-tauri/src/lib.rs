pub mod engine;
pub mod headless;
pub mod rng;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use engine::encounter::{EncounterResult, EncounterState};
use engine::loader::{EventData, GameCatalog, ObjectData, ObligationData};
use rand::RngExt;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[tauri::command]
fn toggle_alarm(event_id: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    if game.alarm_event_ids.iter().any(|id| id == &event_id) {
        game.alarm_event_ids.retain(|id| id != &event_id);
    } else {
        game.alarm_event_ids.push(event_id);
    }
    Ok(game.clone())
}

#[tauri::command]
fn set_popup_categories(
    categories: Vec<String>,
    state: State<'_, AppState>,
) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    game.popup_categories = categories;
    Ok(game.clone())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ServiceType {
    Service1,
    Service2,
    Service3,
    Service4,
    Service5,
    Service6,
    Service7,
    Service8,
    Service9,
    Service10,
    Service11,
    Service12,
    Service13,
    Service14,
    Service15,
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
    #[serde(default)]
    pub service_5_needed: bool,
    #[serde(default)]
    pub service_6_needed: bool,
    #[serde(default)]
    pub service_7_needed: bool,
    #[serde(default)]
    pub service_8_needed: bool,
    #[serde(default)]
    pub service_9_needed: bool,
    #[serde(default)]
    pub service_10_needed: bool,
    #[serde(default)]
    pub service_11_needed: bool,
    #[serde(default)]
    pub service_12_needed: bool,
    #[serde(default)]
    pub service_13_needed: bool,
    #[serde(default)]
    pub service_14_needed: bool,
    #[serde(default)]
    pub service_15_needed: bool,
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
    pub lifetime_days: u32,
    #[serde(default)]
    pub expires_day: u32,
    #[serde(default)]
    pub loaned: bool,
    #[serde(default)]
    pub unavailable_until_day: u32,
    #[serde(default)]
    pub trophy_championship: String,
    #[serde(default)]
    pub trophy_position: u32,
    #[serde(default)]
    pub trophy_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActiveEvent {
    #[serde(alias = "action_id")]
    pub event_id: String,
    pub start_day: u32,
    #[serde(default)]
    pub obligation_payments: u32,
    #[serde(default)]
    pub obligation_faults: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionshipCompetitor {
    pub name: String,
    pub position: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionshipResult {
    pub event_id: String,
    pub race_day: u32,
    pub player_position: u32,
    pub competitors: Vec<ChampionshipCompetitor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogEntry {
    pub id: String,
    pub day: u32,
    pub event: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAlert {
    pub id: String,
    pub title: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    #[serde(default)]
    pub name: String,
    pub age_days: u32,
    pub characteristics: std::collections::HashMap<String, f64>,
    pub inventory: Vec<OwnedObject>,
    #[serde(alias = "active_actions")]
    pub active_events: Vec<ActiveEvent>,
    #[serde(default)]
    #[serde(alias = "last_action_day")]
    pub last_event_day: Option<u32>,
    #[serde(default)]
    pub sickness_start_day: Option<u32>,
    #[serde(default)]
    pub sickness_salary_blocked_until_day: Option<u32>,
    #[serde(default)]
    pub dead: bool,
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
    #[serde(default, alias = "championship_memberships")]
    pub quest_memberships: Vec<QuestMembership>,
    #[serde(default)]
    pub championship_results: Vec<ChampionshipResult>,
    #[serde(default)]
    pub event_log: Vec<EventLogEntry>,
    #[serde(default)]
    pub last_race_day: Option<u32>,
    #[serde(default)]
    pub active_encounter: Option<EncounterState>,
    #[serde(default)]
    pub last_encounter_result: Option<EncounterResult>,
    #[serde(default)]
    #[serde(alias = "pending_sponsor_action_id")]
    pub pending_sponsor_event_id: Option<String>,
    #[serde(default)]
    pub rng_state: u64,
    #[serde(default)]
    pub alarm_event_ids: Vec<String>,
    #[serde(default)]
    pub popup_categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunStatus {
    Ongoing,
    GoalReached,
    DeadMoney,
    DeadStamina,
    Dead,
    MaxDays,
}

pub fn roll(game: &mut GameState) -> f64 {
    rng::next_f64(&mut game.rng_state)
}

pub fn run_status(
    game: &GameState,
    goal_characteristic: &str,
    goal_value: f64,
    max_days: u32,
) -> RunStatus {
    if game.player.dead {
        return RunStatus::Dead;
    }
    if game
        .player
        .characteristics
        .get(goal_characteristic)
        .copied()
        .unwrap_or(0.0)
        >= goal_value
    {
        return RunStatus::GoalReached;
    }
    if game
        .player
        .characteristics
        .get("budget")
        .copied()
        .unwrap_or(0.0)
        < 0.0
    {
        return RunStatus::DeadMoney;
    }
    if game
        .player
        .characteristics
        .get("stamina")
        .copied()
        .unwrap_or(0.0)
        <= 0.0
    {
        return RunStatus::DeadStamina;
    }
    if game.current_day.saturating_sub(1) >= max_days {
        return RunStatus::MaxDays;
    }
    RunStatus::Ongoing
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestMembership {
    #[serde(alias = "championship_id")]
    pub quest_id: String,
    pub joined_day: u32,
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
pub struct EventStartResult {
    pub event_name: String,
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
    pub sponsor_payment: f64,
    pub message: String,
    pub damage_type: String,
}

pub struct AppState(pub Mutex<GameState>);

const DEFAULT_THEME_COLORS: &[(&str, &str)] = &[
    ("app_background", "#0F172A"),
    ("surface_background", "#1E293B"),
    ("surface_border", "#334155"),
    ("control_border", "#475569"),
    ("control_background", "#334155"),
    ("primary_accent", "#2563EB"),
    ("primary_accent_border", "#93C5FD"),
    ("primary_text", "#F8FAFC"),
    ("secondary_text", "#E2E8F0"),
    ("muted_text", "#94A3B8"),
    ("subtle_text", "#CBD5E1"),
    ("link_text", "#BFDBFE"),
    ("dark_text", "#0F172A"),
    ("white_text", "#FFFFFF"),
    ("success_text", "#86EFAC"),
    ("success_background", "#166534"),
    ("warning_text", "#FBBF24"),
    ("warning_background", "#854D0E"),
    ("error_text", "#FCA5A5"),
    ("error_background", "#7F1D1D"),
    ("error_border", "#EF4444"),
    ("error_light_text", "#FEE2E2"),
    ("info_background", "#1E3A8A"),
    ("info_border", "#3B82F6"),
    ("info_text", "#BFDBFE"),
    ("progress_background", "#334155"),
    ("progress_high", "#22C55E"),
    ("progress_medium", "#EAB308"),
    ("progress_low", "#EF4444"),
    ("modal_overlay", "#020617"),
    ("modal_overlay_dark", "#000000"),
    ("light_surface", "#F8FAFC"),
    ("light_frame", "#FFFFFF"),
    ("danger_action", "#EF4444"),
    ("attention_text", "#F59E0B"),
    ("danger_text", "#B91C1C"),
    ("speed_normal_text", "#BBF7D0"),
    ("speed_fast_text", "#FEF08A"),
    ("speed_fastest_text", "#FED7AA"),
];

#[derive(Debug, Deserialize)]
struct ThemeColorRow {
    element_id: String,
    hex_color: String,
}

fn default_theme_colors() -> HashMap<String, String> {
    DEFAULT_THEME_COLORS
        .iter()
        .map(|(id, color)| ((*id).to_string(), (*color).to_string()))
        .collect()
}

fn read_theme_colors(dataset_path: &str) -> HashMap<String, String> {
    let path = Path::new(dataset_path).join("colors.csv");
    let mut colors = default_theme_colors();
    let Ok(mut reader) = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)
    else {
        return colors;
    };
    for row in reader.deserialize::<ThemeColorRow>().flatten() {
        if row.hex_color.len() == 7
            && row.hex_color.starts_with('#')
            && row.hex_color[1..]
                .chars()
                .all(|value| value.is_ascii_hexdigit())
        {
            colors.insert(row.element_id, row.hex_color.to_ascii_uppercase());
        }
    }
    colors
}

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
        "week" | "weeks" => 7,
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

fn sponsor_payout(action: &EventData, position: u32) -> f64 {
    let entries = action
        .sponsor_payouts
        .split(';')
        .filter_map(|entry| {
            let (key, value) = entry.split_once(':')?;
            Some((
                key.trim().to_ascii_lowercase(),
                value.trim().parse::<f64>().ok()?,
            ))
        })
        .collect::<Vec<_>>();
    entries
        .iter()
        .find(|(key, _)| key == &position.to_string())
        .map(|(_, amount)| *amount)
        .or_else(|| {
            entries
                .iter()
                .find(|(key, _)| key == "default")
                .map(|(_, amount)| *amount)
        })
        .unwrap_or(0.0)
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

#[tauri::command]
fn default_dataset_dialog_path() -> String {
    let mut candidates = Vec::new();
    if let Ok(dataset_path) = std::env::var("DATASET_PATH") {
        let configured = PathBuf::from(dataset_path);
        let absolute = if configured.is_absolute() {
            configured
        } else {
            std::env::current_dir()
                .map(|directory| directory.join(configured))
                .unwrap_or_default()
        };
        if absolute.is_dir() {
            return absolute.to_string_lossy().into_owned();
        }
        if let Some(parent) = absolute.parent() {
            candidates.push(parent.to_path_buf());
        }
    }
    if let Ok(executable) = std::env::current_exe() {
        if let Some(parent) = executable.parent() {
            candidates.push(parent.to_path_buf());
        }
    }
    if let Ok(current) = std::env::current_dir() {
        candidates.push(current);
    }

    for candidate in candidates {
        for directory in candidate.ancestors() {
            if directory.join("dataset").is_dir() {
                return directory.join("dataset").to_string_lossy().into_owned();
            }
        }
    }

    std::env::current_dir()
        .map(|directory| {
            let dataset = directory.join("dataset");
            if dataset.is_dir() {
                dataset.to_string_lossy().into_owned()
            } else {
                directory.to_string_lossy().into_owned()
            }
        })
        .unwrap_or_else(|_| ".".into())
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

fn obligation_interval_days(obligation: &ObligationData) -> u32 {
    calculate_interval_days(obligation.interval, &obligation.interval_unit)
}

fn event_has_obligation(game: &GameState, event_id: &str) -> bool {
    game.catalog
        .obligations
        .iter()
        .any(|obligation| obligation.event_id == event_id)
}

fn event_upfront_stamina_cost(game: &GameState, event: &EventData) -> f64 {
    if event_has_obligation(game, &event.id) {
        0.0
    } else {
        event.stamina_cost
    }
}

fn obligation_due_on_day(
    obligation: &ObligationData,
    active: &ActiveEvent,
    current_day: u32,
) -> bool {
    let interval = obligation_interval_days(obligation);
    let elapsed = current_day.saturating_sub(active.start_day);
    if interval == 0 || elapsed == 0 || elapsed % interval != 0 {
        return false;
    }
    if obligation.max_payments > 0 && active.obligation_payments >= obligation.max_payments {
        return false;
    }
    let due_days = obligation
        .due_days
        .split(';')
        .filter_map(|value| value.trim().parse::<u32>().ok())
        .collect::<Vec<_>>();
    due_days.is_empty() || due_days.contains(&weekday(current_day))
}

fn obligation_message(template: &str, active: &ActiveEvent, obligation: &ObligationData) -> String {
    template
        .replace("{faults}", &active.obligation_faults.to_string())
        .replace("{fault_limit}", &obligation.fault_limit.to_string())
        .replace("{payments}", &active.obligation_payments.to_string())
        .replace("{amount}", &obligation.amount.to_string())
}

fn process_obligations(game: &mut GameState, current_day: u32) -> Vec<String> {
    let obligations = game.catalog.obligations.clone();
    let mut failed_payout_events = Vec::new();
    let mut ended_events = Vec::new();

    for index in 0..game.player.active_events.len() {
        let active_snapshot = game.player.active_events[index].clone();
        let Some(obligation) = obligations
            .iter()
            .find(|obligation| obligation.event_id == active_snapshot.event_id)
        else {
            continue;
        };
        if obligation.skip_when_sick && game.player.sickness_start_day.is_some()
            || !obligation_due_on_day(obligation, &active_snapshot, current_day)
        {
            continue;
        }

        let can_pay = characteristic_value(&game.player, &obligation.resource) >= obligation.amount;
        if can_pay {
            adjust_characteristic(game, &obligation.resource, -obligation.amount);
            let active = &mut game.player.active_events[index];
            active.obligation_payments = active.obligation_payments.saturating_add(1);
            if obligation
                .completion_consequence
                .eq_ignore_ascii_case("end_event")
                && obligation.max_payments > 0
                && active.obligation_payments >= obligation.max_payments
            {
                ended_events.push(active.event_id.clone());
            }
            continue;
        }

        game.player.active_events[index].obligation_faults = game.player.active_events[index]
            .obligation_faults
            .saturating_add(1);
        let active_snapshot = game.player.active_events[index].clone();
        if obligation.fault_blocks_payout {
            failed_payout_events.push(active_snapshot.event_id.clone());
        }
        let fault_title = obligation.fault_title.clone();
        let fault_message =
            obligation_message(&obligation.fault_message, &active_snapshot, obligation);
        let fault_log = obligation_message(&obligation.fault_log, &active_snapshot, obligation);
        if !fault_log.is_empty() {
            log_event(game, fault_log);
        }
        game.pending_alerts.push(GameAlert {
            id: format!(
                "obligation_fault_{}_{}",
                active_snapshot.event_id, current_day
            ),
            title: fault_title,
            message: fault_message,
        });

        if obligation.fault_limit > 0 && active_snapshot.obligation_faults >= obligation.fault_limit
        {
            let limit_title = obligation.limit_title.clone();
            let limit_message =
                obligation_message(&obligation.limit_message, &active_snapshot, obligation);
            let limit_log = obligation_message(&obligation.limit_log, &active_snapshot, obligation);
            if !limit_log.is_empty() {
                log_event(game, limit_log);
            }
            game.pending_alerts.push(GameAlert {
                id: format!(
                    "obligation_limit_{}_{}",
                    active_snapshot.event_id, current_day
                ),
                title: limit_title,
                message: limit_message,
            });
            if obligation
                .fault_consequence
                .eq_ignore_ascii_case("end_event")
            {
                ended_events.push(active_snapshot.event_id.clone());
            } else if obligation.fault_consequence.eq_ignore_ascii_case("death") {
                game.player.dead = true;
                game.time_speed = TimeSpeed::Paused;
            }
        }
    }

    if !ended_events.is_empty() {
        game.player
            .active_events
            .retain(|active| !ended_events.iter().any(|id| id == &active.event_id));
    }
    failed_payout_events
}

fn characteristic_value(player: &Player, id: &str) -> f64 {
    player.characteristics.get(id).copied().unwrap_or_default()
}

fn adjust_characteristic(game: &mut GameState, id: &str, amount: f64) {
    let definition = game
        .catalog
        .player_characteristics
        .iter()
        .find(|entry| entry.id == id);
    let value = (characteristic_value(&game.player, id) + amount).clamp(
        definition
            .map(|entry| entry.min_value)
            .unwrap_or(f64::NEG_INFINITY),
        definition
            .map(|entry| entry.max_value)
            .unwrap_or(f64::INFINITY),
    );
    game.player.characteristics.insert(id.to_string(), value);
}

fn log_event(game: &mut GameState, event: impl Into<String>) {
    let id = format!("event_log_{}_{}", game.current_day, game.event_log.len());
    game.event_log.push(EventLogEntry {
        id,
        day: game.current_day,
        event: event.into(),
    });
}

fn popup_category_enabled(game: &GameState, category: &str) -> bool {
    game.popup_categories.iter().any(|entry| entry == category)
}

fn push_popup_alert(
    game: &mut GameState,
    category: &str,
    id: impl Into<String>,
    title: impl Into<String>,
    message: impl Into<String>,
) {
    if popup_category_enabled(game, category) {
        game.pending_alerts.push(GameAlert {
            id: id.into(),
            title: title.into(),
            message: message.into(),
        });
    }
}

fn initial_characteristics(catalog: &GameCatalog) -> std::collections::HashMap<String, f64> {
    catalog
        .player_characteristics
        .iter()
        .map(|characteristic| (characteristic.id.clone(), characteristic.value))
        .collect()
}

fn starting_age_days(catalog: &GameCatalog) -> u32 {
    catalog
        .player_characteristics
        .iter()
        .find(|characteristic| characteristic.id.eq_ignore_ascii_case("age"))
        .map(|characteristic| characteristic.value.max(0.0) as u32)
        .map(|years| years.saturating_mul(config_u32(catalog, "days_per_year", 365).max(1)))
        .unwrap_or_else(|| {
            catalog
                .labels
                .values
                .get("starting_age_days")
                .and_then(|value| value.parse::<u32>().ok())
                .unwrap_or_else(|| {
                    18u32.saturating_mul(config_u32(catalog, "days_per_year", 365).max(1))
                })
        })
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
        ServiceType::Service5 => Some(&object.cost_5),
        ServiceType::Service6 => Some(&object.cost_6),
        ServiceType::Service7 => Some(&object.cost_7),
        ServiceType::Service8 => Some(&object.cost_8),
        ServiceType::Service9 => Some(&object.cost_9),
        ServiceType::Service10 => Some(&object.cost_10),
        ServiceType::Service11 => Some(&object.cost_11),
        ServiceType::Service12 => Some(&object.cost_12),
        ServiceType::Service13 => Some(&object.cost_13),
        ServiceType::Service14 => Some(&object.cost_14),
        ServiceType::Service15 => Some(&object.cost_15),
    }
}

fn cost_amount(
    catalog: &GameCatalog,
    object: &OwnedObject,
    service_type: ServiceType,
) -> Result<f64, String> {
    let id =
        cost_id(object, service_type).ok_or_else(|| "Cost reference is missing".to_string())?;
    catalog
        .costs
        .iter()
        .find(|cost| cost.id == id)
        .map(|cost| cost.amount)
        .ok_or_else(|| format!("Cost '{}' not found in costs.csv", id))
}

fn missing_object_prerequisites(
    game: &GameState,
    object: &engine::loader::ObjectData,
) -> Vec<String> {
    let owns = |id: &str| {
        let required_group = game
            .catalog
            .objects
            .iter()
            .find(|candidate| candidate.id == id)
            .map(|candidate| candidate.requirement_group.trim())
            .filter(|group| !group.is_empty());
        game.player.inventory.iter().any(|owned| {
            owned.id == id
                || owned.id.starts_with(&format!("{id}_"))
                || required_group.is_some_and(|group| {
                    game.catalog
                        .objects
                        .iter()
                        .find(|candidate| {
                            owned.id == candidate.id
                                || owned.id.starts_with(&format!("{}_", candidate.id))
                        })
                        .is_some_and(|candidate| candidate.requirement_group.trim() == group)
                })
        })
    };
    let mut requirements: Vec<String> = object
        .requires_object_ids
        .split(';')
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .filter(|id| !owns(id))
        .map(str::to_string)
        .collect();
    if !object.license_previous_id.trim().is_empty() && !owns(&object.license_previous_id) {
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
    } else if object.cost_5 == cost_id {
        object.service_5_needed = true;
    } else if object.cost_6 == cost_id {
        object.service_6_needed = true;
    } else if object.cost_7 == cost_id {
        object.service_7_needed = true;
    } else if object.cost_8 == cost_id {
        object.service_8_needed = true;
    } else if object.cost_9 == cost_id {
        object.service_9_needed = true;
    } else if object.cost_10 == cost_id {
        object.service_10_needed = true;
    } else if object.cost_11 == cost_id {
        object.service_11_needed = true;
    } else if object.cost_12 == cost_id {
        object.service_12_needed = true;
    } else if object.cost_13 == cost_id {
        object.service_13_needed = true;
    } else if object.cost_14 == cost_id {
        object.service_14_needed = true;
    } else if object.cost_15 == cost_id {
        object.service_15_needed = true;
    } else {
        let dynamic_slot = [
            (&mut object.cost_6, &mut object.service_6_needed),
            (&mut object.cost_7, &mut object.service_7_needed),
            (&mut object.cost_8, &mut object.service_8_needed),
            (&mut object.cost_9, &mut object.service_9_needed),
            (&mut object.cost_10, &mut object.service_10_needed),
            (&mut object.cost_11, &mut object.service_11_needed),
            (&mut object.cost_12, &mut object.service_12_needed),
            (&mut object.cost_13, &mut object.service_13_needed),
            (&mut object.cost_14, &mut object.service_14_needed),
            (&mut object.cost_15, &mut object.service_15_needed),
        ]
        .into_iter()
        .find(|(slot, _)| slot.is_empty());
        let Some((slot, needed)) = dynamic_slot else {
            return Err(format!(
                "No free service slot is available for cost '{}'",
                cost_id
            ));
        };
        *slot = cost_id.to_string();
        *needed = true;
    }
    Ok(())
}

fn object_requirement_error(game: &GameState, object: &OwnedObject) -> Option<String> {
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
        (object.service_5_needed, object.cost_5.as_str()),
        (object.service_6_needed, object.cost_6.as_str()),
        (object.service_7_needed, object.cost_7.as_str()),
        (object.service_8_needed, object.cost_8.as_str()),
        (object.service_9_needed, object.cost_9.as_str()),
        (object.service_10_needed, object.cost_10.as_str()),
        (object.service_11_needed, object.cost_11.as_str()),
        (object.service_12_needed, object.cost_12.as_str()),
        (object.service_13_needed, object.cost_13.as_str()),
        (object.service_14_needed, object.cost_14.as_str()),
        (object.service_15_needed, object.cost_15.as_str()),
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
        "greater_than" => {
            actual.parse::<f64>().unwrap_or_default() > expected.parse::<f64>().unwrap_or_default()
        }
        "less_than" => {
            actual.parse::<f64>().unwrap_or_default() < expected.parse::<f64>().unwrap_or_default()
        }
        _ => false,
    }
}

fn condition_value(
    game: &GameState,
    context: &TriggerContext,
    subject_type: &str,
    subject_ref: &str,
) -> Option<String> {
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

fn rule_matches(
    game: &GameState,
    rule: &engine::loader::CostRule,
    context: &TriggerContext,
) -> bool {
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
    let selected_damage = context
        .damage_type
        .as_deref()
        .filter(|damage| !damage.eq_ignore_ascii_case("none") && !damage.trim().is_empty())
        .map(str::to_string);
    let automatic_damage =
        normalized(&context.trigger_type) == "event_completed" && selected_damage.is_none();
    let race_count = game
        .event_history
        .iter()
        .filter(|history| {
            game.catalog
                .events
                .iter()
                .find(|event| event.id == history.event_id)
                .is_some_and(|event| event.tags.split(';').any(|tag| normalized(tag) == "race"))
        })
        .count() as u32
        + 1;
    let automatic_damage_choice = if automatic_damage {
        let candidates: Vec<&engine::loader::CostRule> = rules
            .iter()
            .filter(|rule| {
                normalized(&rule.trigger_type) == "event_completed"
                    && !rule.damage_type.trim().is_empty()
                    && (rule.event_interval == 0 || race_count % rule.event_interval == 0)
                    && rule_matches(
                        game,
                        rule,
                        &TriggerContext {
                            damage_type: Some(rule.damage_type.clone()),
                            ..context.clone()
                        },
                    )
            })
            .collect();
        let total_probability: f64 = candidates
            .iter()
            .map(|rule| rule.probability.clamp(0.0, 1.0))
            .sum();
        if total_probability > 0.0 && roll(game) <= total_probability.min(1.0) {
            let mut pick = roll(game) * total_probability;
            candidates
                .iter()
                .find(|rule| {
                    pick -= rule.probability.clamp(0.0, 1.0);
                    pick <= 0.0
                })
                .map(|rule| rule.damage_type.clone())
        } else {
            None
        }
    } else {
        selected_damage.clone()
    };

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
            let mut evaluation_context = rule_context.clone();
            if !rule.damage_type.trim().is_empty() {
                evaluation_context.damage_type = automatic_damage_choice.clone();
            }
            if !rule_matches(game, &rule, &evaluation_context) {
                continue;
            }
            if normalized(&rule.trigger_type) == "day_elapsed"
                && (rule.interval_days == 0 || day % rule.interval_days != 0)
            {
                continue;
            }
            if normalized(&rule.trigger_type) == "day_elapsed" && rule.no_event_days > 0 {
                let days_without_event = game
                    .last_race_day
                    .map(|last_day| day.saturating_sub(last_day))
                    .unwrap_or(day);
                if days_without_event != rule.no_event_days {
                    continue;
                }
            }
            if normalized(&rule.trigger_type) == "event_completed"
                && !rule.damage_type.trim().is_empty()
                && selected_damage.is_none()
                && automatic_damage_choice.as_deref() != Some(rule.damage_type.as_str())
            {
                continue;
            }
            if normalized(&rule.trigger_type) == "event_completed"
                && !rule.damage_type.trim().is_empty()
                && selected_damage.is_some()
                && selected_damage.as_deref() != Some(rule.damage_type.as_str())
            {
                continue;
            }
            if normalized(&rule.trigger_type) == "event_completed"
                && !rule.damage_type.trim().is_empty()
                && selected_damage.is_none()
                && (rule.event_interval == 0 || race_count % rule.event_interval != 0)
            {
                continue;
            }
            if rule.probability <= 0.0
                || (!rule.damage_type.trim().is_empty() && selected_damage.is_some())
                || roll(game) > rule.probability.clamp(0.0, 1.0)
            {
                if !rule.damage_type.trim().is_empty() && selected_damage.is_some() {
                    // A player-selected damage type is authoritative for this race.
                } else {
                    continue;
                }
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
                mark_object_service_needed(game, &rule_context.source_id, &rule.cost_id)?;
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
                adjust_characteristic(game, "budget", -amount);
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
            let message = if !custom_message.trim().is_empty() {
                custom_message
            } else if charged {
                format!(
                    "{} cost of {}{:.2} was applied.",
                    cost_name, currency, amount
                )
            } else if object_service {
                if rule.pending_message.trim().is_empty() {
                    format!(
                        "{} is required for {} and remains unpaid until completed in the {}.",
                        cost_name,
                        rule_context.source_id,
                        label(&game.catalog, "inventory_name", "inventory").to_lowercase()
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
            };
            log_event(
                game,
                if charged {
                    format!("Cost applied: {} ({:.2})", cost_name, amount)
                } else {
                    format!("Cost pending: {} ({:.2})", cost_name, amount)
                },
            );
            push_popup_alert(
                game,
                "Costs applied",
                format!("alert_{}", occurrence_id),
                if object_service {
                    "Service Required"
                } else if charged {
                    "Cost Applied"
                } else {
                    "Cost Pending"
                },
                message,
            );
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
    let dataset_path = std::env::var("DATASET_PATH").unwrap_or_else(|_| "dataset".to_string());
    let catalog = GameCatalog::load_from_directory(&dataset_path);
    let pending_alerts = dataset_warning_alerts(&catalog);
    GameState {
        current_day: 1,
        days_per_year: config_u32(&catalog, "days_per_year", 365).max(1),
        time_speed: TimeSpeed::Paused,
        player: Player {
            name: String::new(),
            age_days: starting_age_days(&catalog),
            characteristics: initial_characteristics(&catalog),
            inventory: vec![],
            active_events: vec![],
            last_event_day: None,
            sickness_start_day: None,
            sickness_salary_blocked_until_day: None,
            dead: false,
        },
        catalog,
        dataset_path,
        pending_alerts,
        cost_ledger: vec![],
        pending_events: vec![],
        event_history: vec![],
        quest_memberships: vec![],
        championship_results: vec![],
        event_log: vec![],
        last_race_day: None,
        active_encounter: None,
        last_encounter_result: None,
        pending_sponsor_event_id: None,
        rng_state: rand::rng().random(),
        alarm_event_ids: vec![],
        popup_categories: vec![
            "Income".into(),
            "Costs applied".into(),
            "Event incoming".into(),
            "My Alarms".into(),
        ],
    }
}

/// Construct a UI-independent game.  The Tauri frontend uses the same state
/// shape, which also makes this useful to simulations and external tools.
pub fn new_game(dataset_path: impl Into<String>) -> GameState {
    new_game_seeded(dataset_path, rand::rng().random())
}

pub fn new_game_seeded(dataset_path: impl Into<String>, seed: u64) -> GameState {
    let dataset_path = dataset_path.into();
    let catalog = GameCatalog::load_from_directory(&dataset_path);
    let pending_alerts = dataset_warning_alerts(&catalog);
    GameState {
        current_day: 1,
        days_per_year: config_u32(&catalog, "days_per_year", 365).max(1),
        time_speed: TimeSpeed::Paused,
        player: Player {
            name: String::new(),
            age_days: starting_age_days(&catalog),
            characteristics: initial_characteristics(&catalog),
            inventory: vec![],
            active_events: vec![],
            last_event_day: None,
            sickness_start_day: None,
            sickness_salary_blocked_until_day: None,
            dead: false,
        },
        catalog,
        dataset_path,
        pending_alerts,
        cost_ledger: vec![],
        pending_events: vec![],
        event_history: vec![],
        quest_memberships: vec![],
        championship_results: vec![],
        event_log: vec![],
        last_race_day: None,
        active_encounter: None,
        last_encounter_result: None,
        pending_sponsor_event_id: None,
        rng_state: if seed == 0 { 1 } else { seed },
        alarm_event_ids: vec![],
        popup_categories: vec![
            "Income".into(),
            "Costs applied".into(),
            "Event incoming".into(),
            "My Alarms".into(),
        ],
    }
}

fn dataset_warning_alerts(catalog: &GameCatalog) -> Vec<GameAlert> {
    catalog
        .dataset_warnings
        .iter()
        .enumerate()
        .map(|(index, warning)| GameAlert {
            id: format!("dataset_warning_{index}"),
            title: "Dataset loading warning".into(),
            message: warning.clone(),
        })
        .collect()
}

pub fn legal_event_ids(game: &GameState) -> Vec<String> {
    if game.active_encounter.is_some() {
        return Vec::new();
    }
    game.catalog
        .events
        .iter()
        .filter(|action| action.day_of_year == 0)
        .filter(|action| {
            characteristic_value(&game.player, "budget") >= action.base_cost
                && characteristic_value(&game.player, "stamina")
                    >= event_upfront_stamina_cost(game, action)
                && (!action.event_type.eq_ignore_ascii_case("work")
                    || !game.player.active_events.iter().any(|active| {
                        game.catalog
                            .events
                            .iter()
                            .find(|candidate| candidate.id == active.event_id)
                            .is_some_and(|candidate| {
                                candidate.event_type.eq_ignore_ascii_case("work")
                            })
                    }))
                && (!action.payout_freq_type.eq_ignore_ascii_case("recurring")
                    || !game
                        .player
                        .active_events
                        .iter()
                        .any(|active| active.event_id == action.id))
                && (action.resolution_method != "encounter"
                    || (!action.encounter_id.trim().is_empty()
                        && game
                            .catalog
                            .encounter_configs
                            .iter()
                            .any(|config| config.encounter_id == action.encounter_id)))
        })
        .map(|action| action.id.clone())
        .collect()
}

pub fn eligible_event_entries(game: &GameState) -> Vec<(String, String)> {
    let day_of_year = ((game.current_day - 1) % game.days_per_year) + 1;
    game.catalog
        .events
        .iter()
        .filter(|event| {
            event.day_of_year == day_of_year
                && (event.quest_id.trim().is_empty()
                    || game
                        .quest_memberships
                        .iter()
                        .any(|membership| membership.quest_id == event.quest_id))
                && characteristic_value(&game.player, "budget") >= event.entry_fee
                && (event.required_license_id.trim().is_empty()
                    || game.player.inventory.iter().any(|object| {
                        object.id == event.required_license_id
                            || object
                                .id
                                .starts_with(&format!("{}_", event.required_license_id))
                    }))
                && !game.pending_events.iter().any(|entry| {
                    entry.event_id == event.id && entry.entered_day == game.current_day
                })
                && !game.event_history.iter().any(|entry| {
                    entry.event_id == event.id && entry.entered_day == game.current_day
                })
        })
        .flat_map(|event| {
            game.player.inventory.iter().filter_map(move |object| {
                if object.object_type != "vehicle"
                    || player_object_does_not_match_requirement(
                        game,
                        object,
                        &event.required_object_ids,
                    )
                    || object_requirement_error(game, object).is_some()
                {
                    return None;
                }
                Some((event.id.clone(), object.id.clone()))
            })
        })
        .collect()
}

fn player_object_does_not_match_requirement(
    game: &GameState,
    object: &OwnedObject,
    required_ids: &str,
) -> bool {
    let required = required_ids
        .split(';')
        .map(str::trim)
        .filter(|id| !id.is_empty());
    required.clone().next().is_some()
        && !required.into_iter().any(|id| {
            object.id == id
                || object.id.starts_with(&format!("{}_", id))
                || game
                    .catalog
                    .objects
                    .iter()
                    .find(|candidate| candidate.id == id)
                    .and_then(|required| {
                        let owned_definition = game.catalog.objects.iter().find(|candidate| {
                            object.id == candidate.id
                                || object.id.starts_with(&format!("{}_", candidate.id))
                        })?;
                        (!required.requirement_group.trim().is_empty()
                            && required.requirement_group == owned_definition.requirement_group)
                            .then_some(true)
                    })
                    .unwrap_or(false)
        })
}

pub fn apply_event(game: &mut GameState, event_id: &str) -> Result<EventStartResult, String> {
    let event = game
        .catalog
        .events
        .iter()
        .find(|event| event.id == event_id)
        .cloned()
        .ok_or_else(|| "Event not found in catalog".to_string())?;
    perform_event_inner(game, event)
}

fn perform_event_inner(
    game: &mut GameState,
    action: EventData,
) -> Result<EventStartResult, String> {
    if characteristic_value(&game.player, "budget") < action.base_cost {
        return Err("Insufficient funds to start action".into());
    }
    let upfront_stamina_cost = event_upfront_stamina_cost(game, &action);
    if characteristic_value(&game.player, "stamina") < upfront_stamina_cost {
        return Err("Not enough stamina to start action".into());
    }
    adjust_characteristic(game, "budget", -action.base_cost);
    adjust_characteristic(game, "stamina", -upfront_stamina_cost);
    game.player.last_event_day = Some(game.current_day);
    let success = roll(game) <= action.success_rate;
    let payout = if success && !action.payout_freq_type.eq_ignore_ascii_case("recurring") {
        action.payout
    } else {
        0.0
    };
    adjust_characteristic(game, "budget", payout);
    if success
        && (action.payout_freq_type.eq_ignore_ascii_case("recurring")
            || event_has_obligation(game, &action.id))
    {
        game.player.active_events.push(ActiveEvent {
            event_id: action.id.clone(),
            start_day: game.current_day,
            obligation_payments: 0,
            obligation_faults: 0,
        });
    }
    evaluate_cost_rules(
        game,
        &TriggerContext {
            trigger_type: "event_completed".into(),
            trigger_ref: action.id.clone(),
            source_type: "event".into(),
            source_id: action.id.clone(),
            outcome: Some(if success { "success" } else { "failure" }.into()),
            ..TriggerContext::default()
        },
        game.current_day,
    )?;
    log_event(
        game,
        if success {
            format!("Action completed: {}", action.name)
        } else {
            format!("Action failed: {}", action.name)
        },
    );
    Ok(EventStartResult {
        event_name: action.name.clone(),
        success,
        payout_received: payout,
        cost_paid: action.base_cost,
        message: format!(
            "{} '{}'.",
            if success { "Completed" } else { "Failed" },
            action.name
        ),
    })
}

pub fn advance_day(game: &mut GameState) -> Result<(), String> {
    advance_one_day(game)
}

pub fn enter_event_for_sim(
    game: &mut GameState,
    event_id: &str,
    object_id: &str,
) -> Result<(), String> {
    let event = game
        .catalog
        .events
        .iter()
        .find(|event| event.id == event_id)
        .cloned()
        .ok_or_else(|| "Event not found in catalog".to_string())?;
    if !eligible_event_entries(game)
        .iter()
        .any(|entry| entry == &(event_id.to_string(), object_id.to_string()))
    {
        return Err("Event is not currently eligible".into());
    }
    adjust_characteristic(game, "budget", -event.entry_fee);
    let id = format!("event_entry_{}_{}", event.id, game.current_day);
    game.pending_events.push(PendingEvent {
        id,
        event_id: event.id.clone(),
        object_id: object_id.into(),
        entered_day: game.current_day,
    });
    for _ in 0..event_duration_days(&event) {
        advance_one_day(game)?;
    }
    Ok(())
}

pub fn submit_event_for_sim(
    game: &mut GameState,
    entry_id: &str,
    result: &str,
) -> Result<EventResult, String> {
    submit_event_for_sim_with_details(game, entry_id, result, None)
}

pub fn submit_event_for_sim_with_details(
    game: &mut GameState,
    entry_id: &str,
    result: &str,
    player_position: Option<u32>,
) -> Result<EventResult, String> {
    let index = game
        .pending_events
        .iter()
        .position(|entry| entry.id == entry_id)
        .ok_or_else(|| "Pending event entry not found".to_string())?;
    let entry = game.pending_events.remove(index);
    let event = game
        .catalog
        .events
        .iter()
        .find(|event| event.id == entry.event_id)
        .cloned()
        .ok_or_else(|| "Event not found in catalog".to_string())?;
    let success = if event.resolution_method.eq_ignore_ascii_case("random") {
        roll(game) <= event.success_rate
    } else {
        matches!(
            normalized(result).as_str(),
            "success" | "successful" | "win" | "won" | "1" | "yes" | "true"
        )
    };
    let reward = if success { event.reward_pool } else { 0.0 };
    let charisma = if success { event.charisma_reward } else { 0.0 };
    adjust_characteristic(game, "budget", reward);
    adjust_characteristic(game, "charisma", charisma);
    game.event_history.push(EventHistory {
        id: entry.id.clone(),
        event_id: event.id.clone(),
        object_id: entry.object_id,
        entered_day: entry.entered_day,
        result: result.into(),
        outcome: if success { "Success" } else { "Unsuccessful" }.into(),
        reward_awarded: reward,
        charisma_reward_awarded: charisma,
        damage_type: String::new(),
    });
    if !event.quest_id.trim().is_empty() {
        if let Some(position) = player_position.filter(|position| *position > 0) {
            game.championship_results.push(ChampionshipResult {
                event_id: event.id.clone(),
                race_day: game.current_day,
                player_position: position,
                competitors: vec![],
            });
            let is_final_championship_race =
                event.tags.split(';').any(|tag| normalized(tag) == "finale")
                    || !game.catalog.events.iter().any(|candidate| {
                        candidate.quest_id == event.quest_id
                            && candidate.day_of_year > event.day_of_year
                    });
            if success && is_final_championship_race && matches!(position, 1..=3) {
                if let (Some(quest), Some(base_trophy)) = (
                    game.catalog
                        .quests
                        .iter()
                        .find(|quest| quest.id == event.quest_id)
                        .cloned(),
                    game.catalog
                        .objects
                        .iter()
                        .find(|object| object.object_type == "achievements")
                        .cloned(),
                ) {
                    let trophy_id =
                        format!("trophy_{}_{}_level_{}", quest.id, position, quest.level);
                    if !game
                        .player
                        .inventory
                        .iter()
                        .any(|object| object.id == trophy_id)
                    {
                        let mut trophy = base_trophy;
                        trophy.id = trophy_id;
                        trophy.name = format!(
                            "{} - {} place Trophy (Level {})",
                            quest.name, position, quest.level
                        );
                        trophy.trophy_championship = quest.name;
                        trophy.trophy_position = position;
                        trophy.trophy_level = quest.level;
                        let snapshot = game.clone();
                        game.player
                            .inventory
                            .push(build_owned_object(&trophy, &snapshot, false, 0));
                    }
                }
            }
        }
    }
    Ok(EventResult {
        event_name: event.name,
        outcome: if success { "Success" } else { "Unsuccessful" }.into(),
        entry_fee_paid: event.entry_fee,
        reward_awarded: reward,
        charisma_reward_awarded: charisma,
        sponsor_payment: 0.0,
        message: result.into(),
        damage_type: String::new(),
    })
}
#[tauri::command]
fn get_game_state(state: State<'_, AppState>) -> Result<GameState, String> {
    Ok(state.0.lock().map_err(|e| e.to_string())?.clone())
}

#[tauri::command]
fn start_encounter(
    encounter_id: String,
    opponent_id: String,
    state: State<'_, AppState>,
) -> Result<EncounterState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    let encounter = engine::encounter::start(
        &game.catalog,
        &encounter_id,
        &opponent_id,
        &game.player.characteristics,
        game.rng_state,
    )?;
    game.active_encounter = Some(encounter.clone());
    Ok(encounter)
}

pub fn resolve_encounter_for_sim(
    game: &mut GameState,
    action_id: Option<&str>,
) -> Result<serde_json::Value, String> {
    let mut encounter = game
        .active_encounter
        .take()
        .ok_or_else(|| "No active encounter".to_string())?;
    let mut inventory: Vec<String> = game.player.inventory.iter().map(|o| o.id.clone()).collect();
    engine::encounter::play_turn(&game.catalog, &mut encounter, action_id, &mut inventory)?;
    while !encounter.finished && encounter.current_actor == "opponent" {
        engine::encounter::play_turn(&game.catalog, &mut encounter, None, &mut inventory)?;
    }
    game.rng_state = encounter.rng_state;
    if let Some(mut result) = engine::encounter::result(&encounter) {
        let outcomes: Vec<_> = game
            .catalog
            .encounter_outcomes
            .iter()
            .filter(|o| {
                o.trigger.eq_ignore_ascii_case(&result.outcome)
                    && (o.applies_to_encounter_id.is_empty()
                        || o.applies_to_encounter_id == encounter.encounter_id)
            })
            .cloned()
            .collect();
        for outcome in outcomes {
            if roll(game) > outcome.probability {
                continue;
            }
            match outcome.consequence_type.to_ascii_lowercase().as_str() {
                "grant_object" => {
                    if let Some(def) = game
                        .catalog
                        .objects
                        .iter()
                        .find(|o| o.id == outcome.consequence_target)
                        .cloned()
                    {
                        let snapshot = game.clone();
                        game.player
                            .inventory
                            .push(build_owned_object(&def, &snapshot, false, 0));
                        result
                            .consequences_applied
                            .push(format!("Granted {}", outcome.consequence_target));
                    }
                }
                "attribute_delta" => {
                    let delta = outcome
                        .consequence_value
                        .strip_prefix("config:")
                        .map(|key| config_f64(&game.catalog, key, 0.0))
                        .or_else(|| outcome.consequence_value.parse::<f64>().ok());
                    if let Some(delta) = delta {
                        let (minimum, maximum) = game
                            .catalog
                            .player_characteristics
                            .iter()
                            .find(|entry| entry.id == outcome.consequence_target)
                            .map(|entry| (entry.min_value, entry.max_value))
                            .unwrap_or((f64::NEG_INFINITY, f64::INFINITY));
                        if let Some(value) = game
                            .player
                            .characteristics
                            .get_mut(&outcome.consequence_target)
                        {
                            *value = (*value + delta).clamp(minimum, maximum);
                            result
                                .consequences_applied
                                .push(format!("{} {:+}", outcome.consequence_target, delta));
                        }
                    }
                }
                "custom_event" => {
                    let target_action_id = game
                        .pending_sponsor_event_id
                        .as_deref()
                        .unwrap_or(&outcome.consequence_target);
                    if let Some(action) = game
                        .catalog
                        .events
                        .iter()
                        .find(|action| {
                            action.id == target_action_id
                                && action.event_type.eq_ignore_ascii_case("sponsor")
                        })
                        .cloned()
                    {
                        let current_day = game.current_day;
                        if !game
                            .quest_memberships
                            .iter()
                            .any(|membership| membership.quest_id == action.sponsor_quest_id)
                        {
                            game.quest_memberships.push(QuestMembership {
                                quest_id: action.sponsor_quest_id.clone(),
                                joined_day: current_day,
                            });
                        }
                        add_quest_events_to_alarms(game, &action.sponsor_quest_id);
                        if !game
                            .player
                            .active_events
                            .iter()
                            .any(|active| active.event_id == action.id)
                        {
                            game.player.active_events.push(ActiveEvent {
                                event_id: action.id.clone(),
                                start_day: current_day,
                                obligation_payments: 0,
                                obligation_faults: 0,
                            });
                        }
                        let sponsored_ids = std::iter::once(action.sponsor_object_id.clone())
                            .chain(
                                action
                                    .sponsor_equipment_ids
                                    .split(';')
                                    .map(str::trim)
                                    .filter(|id| !id.is_empty())
                                    .map(str::to_string),
                            );
                        for object_id in sponsored_ids {
                            if game.player.inventory.iter().any(|object| {
                                object.id == object_id
                                    || object.id.starts_with(&format!("{object_id}_"))
                            }) {
                                continue;
                            }
                            if let Some(definition) = game
                                .catalog
                                .objects
                                .iter()
                                .find(|object| object.id == object_id)
                                .cloned()
                            {
                                let snapshot = game.clone();
                                let is_sponsored_car = object_id == action.sponsor_object_id;
                                let next_year =
                                    ((game.current_day.saturating_sub(1) / game.days_per_year) + 1)
                                        .saturating_mul(game.days_per_year)
                                        .saturating_add(1);
                                let equipment_lifetime =
                                    if !is_sponsored_car && definition.lifetime_days > 0 {
                                        ((definition.lifetime_days as f64) * 1.5).ceil() as u32
                                    } else {
                                        0
                                    };
                                let expires_day = if is_sponsored_car {
                                    next_year
                                } else if equipment_lifetime > 0 {
                                    game.current_day.saturating_add(equipment_lifetime)
                                } else {
                                    0
                                };
                                let mut loaned = build_owned_object(
                                    &definition,
                                    &snapshot,
                                    is_sponsored_car,
                                    expires_day,
                                );
                                loaned.unavailable_until_day = game.current_day;
                                game.player.inventory.push(loaned);
                            }
                        }
                        result
                            .consequences_applied
                            .push(format!("Activated {}", action.name));
                    }
                }
                _ => {}
            }
        }
        game.last_encounter_result = Some(result.clone());
        game.pending_sponsor_event_id = None;
        return serde_json::to_value(result).map_err(|e| e.to_string());
    }
    game.active_encounter = Some(encounter.clone());
    serde_json::to_value(encounter).map_err(|e| e.to_string())
}

pub fn legal_encounter_action_ids(game: &GameState) -> Vec<String> {
    game.active_encounter
        .as_ref()
        .map(|encounter| {
            let inventory = game
                .player
                .inventory
                .iter()
                .map(|object| object.id.clone())
                .collect::<Vec<_>>();
            engine::encounter::available_player_action_ids(&game.catalog, encounter, &inventory)
        })
        .unwrap_or_default()
}

#[tauri::command]
fn resolve_encounter_turn(
    action_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    resolve_encounter_for_sim(&mut game, action_id.as_deref())
}

#[tauri::command]
fn retreat_encounter(state: State<'_, AppState>) -> Result<EncounterResult, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    let mut encounter = game
        .active_encounter
        .take()
        .ok_or_else(|| "No active encounter".to_string())?;
    let allow_retreat = game
        .catalog
        .encounter_configs
        .iter()
        .find(|c| c.encounter_id == encounter.encounter_id)
        .map(|c| c.allow_retreat)
        .unwrap_or(false);
    if !allow_retreat {
        game.active_encounter = Some(encounter);
        return Err("Retreat is not allowed for this encounter".into());
    }
    encounter.finished = true;
    encounter.outcome = Some("lose".into());
    let result = engine::encounter::result(&encounter)
        .ok_or_else(|| "Encounter did not resolve".to_string())?;
    game.last_encounter_result = Some(result.clone());
    Ok(result)
}

#[tauri::command]
fn get_catalog(state: State<'_, AppState>) -> Result<GameCatalog, String> {
    Ok(state.0.lock().map_err(|e| e.to_string())?.catalog.clone())
}

#[tauri::command]
fn get_theme_colors(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    let game = state.0.lock().map_err(|e| e.to_string())?;
    Ok(read_theme_colors(&game.dataset_path))
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

fn save_database_path(dataset_path: &str) -> Result<PathBuf, String> {
    let dataset = Path::new(dataset_path)
        .canonicalize()
        .map_err(|error| format!("Dataset folder cannot be resolved: {error}"))?;
    if !dataset.is_dir() {
        return Err("Dataset path is not a folder".into());
    }
    Ok(dataset.join("saves").join("savegame.db"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSlot {
    pub name: String,
}

fn save_database_path_for_slot(dataset_path: &str, slot: &str) -> Result<PathBuf, String> {
    let slot = slot.trim();
    if slot.is_empty()
        || slot == "."
        || slot == ".."
        || !slot.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
    {
        return Err("Save slot names may contain only letters, numbers, '.', '-' and '_'".into());
    }
    let dataset = Path::new(dataset_path)
        .canonicalize()
        .map_err(|error| format!("Dataset folder cannot be resolved: {error}"))?;
    if !dataset.is_dir() {
        return Err("Dataset path is not a folder".into());
    }
    Ok(dataset.join("saves").join(format!("{slot}.db")))
}

fn write_save(game: &GameState, path: &Path) -> Result<String, String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Invalid save path".to_string())?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("Cannot create save folder: {error}"))?;
    let payload =
        serde_json::to_string(game).map_err(|error| format!("Cannot encode save: {error}"))?;
    let connection =
        Connection::open(path).map_err(|error| format!("Cannot open save database: {error}"))?;
    connection.execute(
        "CREATE TABLE IF NOT EXISTS game_state (id INTEGER PRIMARY KEY CHECK (id = 1), payload TEXT NOT NULL)",
        [],
    ).map_err(|error| format!("Cannot initialize save database: {error}"))?;
    connection
        .execute(
            "INSERT INTO game_state (id, payload) VALUES (1, ?1)
         ON CONFLICT(id) DO UPDATE SET payload = excluded.payload",
            params![payload],
        )
        .map_err(|error| format!("Cannot write save database: {error}"))?;
    Ok(path.to_string_lossy().into_owned())
}

fn load_save_from_path(path: &Path) -> Result<GameState, String> {
    let connection =
        Connection::open(path).map_err(|error| format!("Cannot open save database: {error}"))?;
    let payload = connection
        .query_row("SELECT payload FROM game_state WHERE id = 1", [], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|error| format!("Cannot read save: {error}"))?;
    serde_json::from_str(&payload).map_err(|error| format!("Cannot decode save: {error}"))
}

#[tauri::command]
fn list_save_slots(dataset_path: String) -> Result<Vec<SaveSlot>, String> {
    let dataset = Path::new(&dataset_path)
        .canonicalize()
        .map_err(|error| format!("Dataset folder cannot be resolved: {error}"))?;
    let saves = dataset.join("saves");
    if !saves.exists() {
        return Ok(Vec::new());
    }
    let mut slots = std::fs::read_dir(saves)
        .map_err(|error| format!("Cannot read save folder: {error}"))?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("db") {
                return None;
            }
            Some(SaveSlot {
                name: path.file_stem()?.to_string_lossy().into_owned(),
            })
        })
        .collect::<Vec<_>>();
    slots.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(slots)
}

#[tauri::command]
fn latest_save_slot(dataset_path: String) -> Result<Option<SaveSlot>, String> {
    let dataset = Path::new(&dataset_path)
        .canonicalize()
        .map_err(|error| format!("Dataset folder cannot be resolved: {error}"))?;
    let saves = dataset.join("saves");
    if !saves.exists() {
        return Ok(None);
    }
    let latest = std::fs::read_dir(saves)
        .map_err(|error| format!("Cannot read save folder: {error}"))?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("db")
        })
        .filter_map(|entry| {
            let modified = entry.metadata().ok()?.modified().ok()?;
            Some((modified, entry))
        })
        .max_by_key(|(modified, _)| *modified);
    Ok(latest.and_then(|(_, entry)| {
        entry.path().file_stem().map(|name| SaveSlot {
            name: name.to_string_lossy().into_owned(),
        })
    }))
}

#[tauri::command]
fn start_new_game(
    dataset_path: String,
    player_name: String,
    state: State<'_, AppState>,
) -> Result<GameState, String> {
    let mut new_state = new_game(dataset_path);
    new_state.player.name = player_name.trim().to_string();
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    *game = new_state.clone();
    Ok(new_state)
}

#[tauri::command]
fn save_game_as(slot: String, state: State<'_, AppState>) -> Result<String, String> {
    let game = state.0.lock().map_err(|e| e.to_string())?.clone();
    let path = save_database_path_for_slot(&game.dataset_path, &slot)?;
    write_save(&game, &path)
}

#[tauri::command]
fn load_game_from(
    dataset_path: String,
    slot: String,
    state: State<'_, AppState>,
) -> Result<GameState, String> {
    let path = save_database_path_for_slot(&dataset_path, &slot)?;
    let loaded = load_save_from_path(&path)?;
    let selected = Path::new(&dataset_path)
        .canonicalize()
        .map_err(|error| format!("Dataset folder cannot be resolved: {error}"))?;
    let saved = Path::new(&loaded.dataset_path)
        .canonicalize()
        .map_err(|error| format!("Save belongs to an unavailable dataset: {error}"))?;
    if selected != saved {
        return Err("This save belongs to a different dataset".into());
    }
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    *game = loaded.clone();
    Ok(loaded)
}

#[tauri::command]
fn save_game(state: State<'_, AppState>) -> Result<String, String> {
    let game = state.0.lock().map_err(|e| e.to_string())?.clone();
    let path = save_database_path(&game.dataset_path)?;
    write_save(&game, &path)
}

#[tauri::command]
fn load_game(state: State<'_, AppState>) -> Result<GameState, String> {
    let current_path = state
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .dataset_path
        .clone();
    let database_path = save_database_path(&current_path)?;
    let loaded: GameState = if database_path.exists() {
        let connection = Connection::open(&database_path)
            .map_err(|error| format!("Cannot open save database: {error}"))?;
        let payload = connection
            .query_row("SELECT payload FROM game_state WHERE id = 1", [], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|error| format!("Cannot read save database: {error}"))?;
        serde_json::from_str(&payload).map_err(|error| format!("Cannot decode save: {error}"))?
    } else {
        let path = save_file_path(&current_path)?;
        let data = std::fs::read(&path).map_err(|error| format!("Cannot read save: {error}"))?;
        serde_json::from_slice(&data).map_err(|error| format!("Cannot decode save: {error}"))?
    };
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
fn join_quest(quest_id: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    join_quest_for_sim(&mut game, &quest_id)?;
    Ok(game.clone())
}

pub fn join_quest_for_sim(game: &mut GameState, quest_id: &str) -> Result<(), String> {
    if game
        .quest_memberships
        .iter()
        .any(|membership| membership.quest_id == quest_id)
    {
        return Err("You have already joined this quest".into());
    }
    let quest = game
        .catalog
        .quests
        .iter()
        .find(|quest| quest.id == quest_id)
        .cloned()
        .ok_or_else(|| "Quest not found in catalog".to_string())?;
    if quest.level > 1
        && !game.player.inventory.iter().any(|object| {
            object.object_type == "achievements" && object.trophy_level == quest.level - 1
        })
    {
        return Err(format!(
            "Cannot join '{}': a level {} trophy is required.",
            quest.name,
            quest.level - 1
        ));
    }
    if !quest.required_license_id.trim().is_empty()
        && !game.player.inventory.iter().any(|object| {
            object.id == quest.required_license_id
                || object
                    .id
                    .starts_with(&format!("{}_", quest.required_license_id))
        })
    {
        return Err(format!(
            "Cannot join '{}': required licence '{}' is missing.",
            quest.name, quest.required_license_id
        ));
    }
    if characteristic_value(&game.player, "budget") < quest.join_fee {
        return Err("Insufficient funds to join quest".into());
    }
    adjust_characteristic(game, "budget", -quest.join_fee);
    let joined_day = game.current_day;
    let quest_name = label(&game.catalog, "quest_name", "quest");
    log_event(
        game,
        format!("Joined {} '{}'", quest_name.to_lowercase(), quest.name),
    );
    game.quest_memberships.push(QuestMembership {
        quest_id: quest_id.to_string(),
        joined_day,
    });
    add_quest_events_to_alarms(game, quest_id);
    Ok(())
}

fn add_quest_events_to_alarms(game: &mut GameState, quest_id: &str) {
    let championship_alarm_ids: Vec<String> = game
        .catalog
        .events
        .iter()
        .filter(|event| event.quest_id == quest_id)
        .map(|event| event.id.clone())
        .filter(|id| !game.alarm_event_ids.contains(id))
        .collect();
    game.alarm_event_ids.extend(championship_alarm_ids);
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
    adjust_characteristic(&mut game, "budget", -amount);
    game.cost_ledger[index].status = "charged".into();
    let currency = label(&game.catalog, "currency_symbol", "$");
    log_event(&mut game, format!("Pending cost paid ({:.2})", amount));
    push_popup_alert(
        &mut game,
        "Costs applied",
        format!("paid_{}", cost_occurrence_id),
        "Pending Cost Paid",
        format!("Paid pending cost of {}{:.2}.", currency, amount),
    );
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
    let dataset_warnings = dataset_warning_alerts(&game.catalog);
    game.pending_alerts.extend(dataset_warnings);
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
    let mut expired_names = Vec::new();
    let mut expired_loaned_object_ids = Vec::new();
    game.player.inventory.retain(|object| {
        let expired = object.expires_day > 0 && object.expires_day <= current_day;
        if expired {
            expired_names.push(object.name.clone());
            if object.loaned {
                expired_loaned_object_ids.push(object.id.clone());
            }
        }
        !expired
    });
    if !expired_loaned_object_ids.is_empty() {
        game.player.active_events.retain(|active| {
            let Some(action) = game
                .catalog
                .events
                .iter()
                .find(|action| action.id == active.event_id)
            else {
                return true;
            };
            if !action.event_type.eq_ignore_ascii_case("sponsor") {
                return true;
            }
            let sponsor_object_expired = expired_loaned_object_ids.iter().any(|object_id| {
                object_id == &action.sponsor_object_id
                    || object_id.starts_with(&format!("{}_", action.sponsor_object_id))
            });
            !sponsor_object_expired
                || game.player.inventory.iter().any(|object| {
                    object.loaned
                        && (object.id == action.sponsor_object_id
                            || object
                                .id
                                .starts_with(&format!("{}_", action.sponsor_object_id)))
                })
        });
    }
    if !expired_names.is_empty() {
        game.pending_alerts.push(GameAlert {
            id: format!("expired_equipment_{current_day}"),
            title: "Equipment Expired".into(),
            message: format!(
                "The following equipment expired: {}.",
                expired_names.join(", ")
            ),
        });
    }

    let daily_context = TriggerContext {
        trigger_type: "day_elapsed".into(),
        trigger_ref: String::new(),
        source_type: "day".into(),
        source_id: current_day.to_string(),
        ..TriggerContext::default()
    };
    evaluate_cost_rules(game, &daily_context, current_day)?;

    for object in &mut game.player.inventory {
        if object.service_1_interval_days > 0 && current_day % object.service_1_interval_days == 0 {
            object.service_1_needed = true;
        }
        if object.service_2_interval_days > 0 && current_day % object.service_2_interval_days == 0 {
            object.service_2_needed = true;
        }
        if object.service_3_interval_days > 0 && current_day % object.service_3_interval_days == 0 {
            object.service_3_needed = true;
        }
        if object.service_4_interval_days > 0 && current_day % object.service_4_interval_days == 0 {
            object.service_4_needed = true;
        }
    }

    let had_event = game.player.last_event_day == Some(previous_day);
    if let Some(start_day) = game.player.sickness_start_day {
        let sickness_day = current_day.saturating_sub(start_day);
        if sickness_day < 2 {
            let current = characteristic_value(&game.player, "stamina");
            adjust_characteristic(game, "stamina", 10.0 - current);
        } else if sickness_day < 6 {
            let target = config_f64(&game.catalog, "sickness_recovery_stamina", 50.0);
            let current = characteristic_value(&game.player, "stamina");
            adjust_characteristic(game, "stamina", target - current);
        } else {
            adjust_characteristic(
                game,
                "stamina",
                config_f64(&game.catalog, "sickness_final_recovery", 50.0),
            );
            game.player.sickness_start_day = None;
        }
    }
    let failed_obligation_events = process_obligations(game, current_day);
    if game.player.sickness_start_day.is_none() && !had_event {
        let recovery = config_f64(&game.catalog, "nightly_stamina_recovery", 25.0);
        adjust_characteristic(&mut *game, "stamina", recovery);
    }

    if game.player.sickness_start_day.is_none() {
        let sickness_probability =
            config_f64(&game.catalog, "sickness_daily_probability", 0.001111111).clamp(0.0, 1.0);
        if roll(game) < sickness_probability {
            game.player.sickness_start_day = Some(current_day);
            game.player.sickness_salary_blocked_until_day = Some(current_day.saturating_add(6));
            let current = characteristic_value(&game.player, "stamina");
            adjust_characteristic(
                game,
                "stamina",
                config_f64(&game.catalog, "sickness_initial_stamina", 10.0) - current,
            );
            game.pending_alerts.push(GameAlert {
                id: format!("sickness_{current_day}"),
                title: label(&game.catalog, "sickness_event_name", "Sickness"),
                message: label(
                    &game.catalog,
                    "sickness_event_message",
                    "You are sick. Stamina falls to around 10, then recovers to around 50. Sick days do not count as missed work days.",
                ),
            });
        }
    }

    let mut total_payout = 0.0;
    let mut salary_events = Vec::new();
    let salary_blocked = game
        .player
        .sickness_salary_blocked_until_day
        .map(|day| current_day <= day)
        .unwrap_or(false);
    for active in &game.player.active_events {
        if let Some(action) = game.catalog.events.iter().find(|a| a.id == active.event_id) {
            if action.payout_freq_type.eq_ignore_ascii_case("recurring") {
                let interval =
                    calculate_interval_days(action.payout_freq, &action.payout_freq_unit);
                let elapsed = current_day.saturating_sub(active.start_day);
                let unpaid_job_week =
                    salary_blocked && action.event_type.eq_ignore_ascii_case("work");
                if interval > 0
                    && elapsed > 0
                    && elapsed % interval == 0
                    && !unpaid_job_week
                    && !failed_obligation_events.contains(&active.event_id)
                {
                    total_payout += action.payout;
                    salary_events.push(format!(
                        "Salary received from '{}': {}",
                        action.name, action.payout
                    ));
                }
            }
        }
    }
    adjust_characteristic(&mut *game, "budget", total_payout);
    for event in salary_events {
        log_event(game, event.clone());
        push_popup_alert(
            game,
            "Income",
            format!("income_{}_{}", current_day, game.pending_alerts.len()),
            "Income received",
            event,
        );
    }

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
        log_event(game, format!("Event incoming: {}", event.name));
        let is_alarm = game.alarm_event_ids.iter().any(|id| id == &event.id);
        let category = if is_alarm && popup_category_enabled(game, "My Alarms") {
            Some("My Alarms")
        } else if popup_category_enabled(game, "Event incoming") {
            Some("Event incoming")
        } else {
            None
        };
        if let Some(category) = category {
            push_popup_alert(
                game,
                category,
                format!("event_{}_{}", event.id, current_day),
                if category == "My Alarms" {
                    format!("My Alarm: {}", event_title)
                } else {
                    event_title
                },
                format!(
                    "Today is day {}: '{}' is scheduled.",
                    day_of_year, event.name
                ),
            );
        }
    }
    if !game.pending_alerts.is_empty() {
        game.time_speed = TimeSpeed::Paused;
    }
    Ok(())
}

#[tauri::command]
fn build_owned_object(
    object: &ObjectData,
    game: &GameState,
    loaned: bool,
    expires_day: u32,
) -> OwnedObject {
    OwnedObject {
        id: format!("{}_{}", object.id, game.player.inventory.len() + 1),
        object_type: object.object_type.clone(),
        name: object.name.clone(),
        price: object.price,
        cost_1: object.cost_1.clone(),
        cost_2: object.cost_2.clone(),
        cost_3: object.cost_3.clone(),
        cost_4: object.cost_4.clone(),
        cost_5: object.cost_5.clone(),
        cost_6: object.cost_6.clone(),
        cost_7: object.cost_7.clone(),
        cost_8: object.cost_8.clone(),
        cost_9: object.cost_9.clone(),
        cost_10: object.cost_10.clone(),
        cost_11: object.cost_11.clone(),
        cost_12: object.cost_12.clone(),
        cost_13: object.cost_13.clone(),
        cost_14: object.cost_14.clone(),
        cost_15: object.cost_15.clone(),
        service_1_needed: false,
        service_2_needed: false,
        service_3_needed: false,
        service_4_needed: false,
        service_5_needed: false,
        service_6_needed: false,
        service_7_needed: false,
        service_8_needed: false,
        service_9_needed: false,
        service_10_needed: false,
        service_11_needed: false,
        service_12_needed: false,
        service_13_needed: false,
        service_14_needed: false,
        service_15_needed: false,
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
        description_html: object.description_html.clone(),
        license_level: object.license_level,
        license_previous_id: object.license_previous_id.clone(),
        requires_object_ids: object.requires_object_ids.clone(),
        license_fee: object.license_fee,
        lifetime_days: object.lifetime_days,
        expires_day,
        loaned,
        unavailable_until_day: if object.availability_days > 0 {
            game.current_day.saturating_add(object.availability_days)
        } else {
            0
        },
        trophy_championship: object.trophy_championship.clone(),
        trophy_position: object.trophy_position,
        trophy_level: object.trophy_level,
    }
}

#[tauri::command]
fn buy_object(object_id: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    buy_object_for_sim(&mut game, &object_id)?;
    Ok(game.clone())
}

pub fn buy_object_for_sim(game: &mut GameState, object_id: &str) -> Result<(), String> {
    let object = game
        .catalog
        .objects
        .iter()
        .find(|object| object.id == object_id)
        .cloned()
        .ok_or_else(|| "Object not found in catalog".to_string())?;
    if object.id.starts_with("trophy_")
        || object.id == "business_proposal"
        || object.id == "lower_cost"
    {
        return Err(format!(
            "'{}' is earned through gameplay and cannot be bought",
            object.name
        ));
    }
    let acquisition_cost = if object.object_type == "license" && object.license_fee > 0.0 {
        object.license_fee
    } else {
        object.price
    };
    if object.object_type == "license"
        && game.player.inventory.iter().any(|owned| {
            owned.object_type == "license"
                && (owned.id == object.id || owned.id.starts_with(&format!("{}_", object.id)))
        })
    {
        return Err(format!(
            "Licence '{}' has already been purchased",
            object.name
        ));
    }
    if characteristic_value(&game.player, "budget") < acquisition_cost {
        return Err("Insufficient funds to acquire object".into());
    }

    let missing = missing_object_prerequisites(game, &object);
    if !missing.is_empty() {
        return Err(format!(
            "Cannot acquire '{}': required object(s) missing: {}",
            object.name,
            missing.join(", ")
        ));
    }

    adjust_characteristic(game, "budget", -acquisition_cost);
    if object.paddock_cred_bonus != 0.0 {
        adjust_characteristic(game, "charisma", object.paddock_cred_bonus);
    }
    let object_name = object.name.clone();
    let owned = build_owned_object(
        &object,
        game,
        false,
        if object.lifetime_days > 0 {
            game.current_day.saturating_add(object.lifetime_days)
        } else {
            0
        },
    );
    game.player.inventory.push(owned);
    log_event(
        game,
        format!("{} bought for {}", object_name, acquisition_cost),
    );
    let acquired_id = format!("{}_{}", object_id, game.player.inventory.len());
    let acquired_context = TriggerContext {
        trigger_type: "object_acquired".into(),
        trigger_ref: object_id.to_string(),
        source_type: "object".into(),
        source_id: acquired_id,
        object_type: Some(object.object_type),
        ..TriggerContext::default()
    };
    let current_day = game.current_day;
    evaluate_cost_rules(game, &acquired_context, current_day)?;
    Ok(())
}

pub fn apply_override(game: &mut GameState, path: &str, value: &str) -> Result<(), String> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.len() != 3 {
        return Err(format!("Override '{path}' must be <kind>.<id>.<parameter>"));
    }
    let parsed = value
        .parse::<f64>()
        .map_err(|_| format!("Override '{path}' requires a numeric value"))?;
    match (parts[0], parts[2]) {
        ("action", "success_rate" | "success_probability") => {
            let action = game
                .catalog
                .events
                .iter_mut()
                .find(|action| action.id == parts[1])
                .ok_or_else(|| format!("Unknown action '{}'", parts[1]))?;
            action.success_rate = parsed.clamp(0.0, 1.0);
        }
        ("event", "success_rate" | "success_probability") => {
            let event = game
                .catalog
                .events
                .iter_mut()
                .find(|event| event.id == parts[1])
                .ok_or_else(|| format!("Unknown event '{}'", parts[1]))?;
            event.success_rate = parsed.clamp(0.0, 1.0);
        }
        ("object", "price") => {
            let object = game
                .catalog
                .objects
                .iter_mut()
                .find(|object| object.id == parts[1])
                .ok_or_else(|| format!("Unknown object '{}'", parts[1]))?;
            object.price = parsed.max(0.0);
        }
        ("cost_rule", "probability") => {
            let rule = game
                .catalog
                .cost_rules
                .iter_mut()
                .find(|rule| rule.id == parts[1])
                .ok_or_else(|| format!("Unknown cost rule '{}'", parts[1]))?;
            rule.probability = parsed.clamp(0.0, 1.0);
        }
        _ => return Err(format!("Unsupported override '{path}'")),
    }
    Ok(())
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
            (
                cost_amount(&game.catalog, object, service_type)?,
                object.service_1_needed,
            )
        }
        ServiceType::Service2 => {
            let object = &game.player.inventory[index];
            (
                cost_amount(&game.catalog, object, service_type)?,
                object.service_2_needed,
            )
        }
        ServiceType::Service3 => {
            let object = &game.player.inventory[index];
            (
                cost_amount(&game.catalog, object, service_type)?,
                object.service_3_needed,
            )
        }
        ServiceType::Service4 => {
            let object = &game.player.inventory[index];
            (
                cost_amount(&game.catalog, object, service_type)?,
                object.service_4_needed,
            )
        }
        ServiceType::Service5 => {
            let object = &game.player.inventory[index];
            (
                cost_amount(&game.catalog, object, service_type)?,
                object.service_5_needed,
            )
        }
        ServiceType::Service6 => {
            let object = &game.player.inventory[index];
            (
                cost_amount(&game.catalog, object, service_type)?,
                object.service_6_needed,
            )
        }
        ServiceType::Service7 => {
            let object = &game.player.inventory[index];
            (
                cost_amount(&game.catalog, object, service_type)?,
                object.service_7_needed,
            )
        }
        ServiceType::Service8 => {
            let object = &game.player.inventory[index];
            (
                cost_amount(&game.catalog, object, service_type)?,
                object.service_8_needed,
            )
        }
        ServiceType::Service9 => {
            let object = &game.player.inventory[index];
            (
                cost_amount(&game.catalog, object, service_type)?,
                object.service_9_needed,
            )
        }
        ServiceType::Service10 => {
            let object = &game.player.inventory[index];
            (
                cost_amount(&game.catalog, object, service_type)?,
                object.service_10_needed,
            )
        }
        ServiceType::Service11 => {
            let object = &game.player.inventory[index];
            (
                cost_amount(&game.catalog, object, service_type)?,
                object.service_11_needed,
            )
        }
        ServiceType::Service12 => {
            let object = &game.player.inventory[index];
            (
                cost_amount(&game.catalog, object, service_type)?,
                object.service_12_needed,
            )
        }
        ServiceType::Service13 => {
            let object = &game.player.inventory[index];
            (
                cost_amount(&game.catalog, object, service_type)?,
                object.service_13_needed,
            )
        }
        ServiceType::Service14 => {
            let object = &game.player.inventory[index];
            (
                cost_amount(&game.catalog, object, service_type)?,
                object.service_14_needed,
            )
        }
        ServiceType::Service15 => {
            let object = &game.player.inventory[index];
            (
                cost_amount(&game.catalog, object, service_type)?,
                object.service_15_needed,
            )
        }
    };
    if !needed {
        return Err("That service is not currently required".into());
    }
    let pending_occurrence = game.cost_ledger.iter().position(|occurrence| {
        occurrence.status == "pending"
            && occurrence.source_type == "object_service"
            && occurrence.source_id == object_id
            && occurrence.cost_id
                == cost_id(&game.player.inventory[index], service_type).unwrap_or_default()
    });
    let payable_cost = pending_occurrence
        .map(|occurrence| game.cost_ledger[occurrence].amount)
        .unwrap_or(cost);
    if characteristic_value(&game.player, "budget") < payable_cost {
        return Err("Insufficient funds for service".into());
    }
    adjust_characteristic(&mut game, "budget", -payable_cost);
    if let Some(occurrence) = pending_occurrence {
        game.cost_ledger[occurrence].status = "charged".into();
    }
    let serviced_object_name = game.player.inventory[index].name.clone();
    let object = &mut game.player.inventory[index];
    match service_type {
        ServiceType::Service1 => object.service_1_needed = false,
        ServiceType::Service2 => object.service_2_needed = false,
        ServiceType::Service3 => object.service_3_needed = false,
        ServiceType::Service4 => object.service_4_needed = false,
        ServiceType::Service5 => object.service_5_needed = false,
        ServiceType::Service6 => object.service_6_needed = false,
        ServiceType::Service7 => object.service_7_needed = false,
        ServiceType::Service8 => object.service_8_needed = false,
        ServiceType::Service9 => object.service_9_needed = false,
        ServiceType::Service10 => object.service_10_needed = false,
        ServiceType::Service11 => object.service_11_needed = false,
        ServiceType::Service12 => object.service_12_needed = false,
        ServiceType::Service13 => object.service_13_needed = false,
        ServiceType::Service14 => object.service_14_needed = false,
        ServiceType::Service15 => object.service_15_needed = false,
    }
    log_event(&mut game, format!("{} serviced", serviced_object_name));
    Ok(game.clone())
}

#[tauri::command]
fn perform_event(event_id: String, state: State<'_, AppState>) -> Result<EventStartResult, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    let action: EventData = game
        .catalog
        .events
        .iter()
        .find(|action| action.id == event_id)
        .cloned()
        .ok_or_else(|| "Action not found in catalog".to_string())?;
    if characteristic_value(&game.player, "budget") < action.base_cost {
        return Err("Insufficient funds to start action".into());
    }

    let upfront_stamina_cost = event_upfront_stamina_cost(&game, &action);
    if characteristic_value(&game.player, "stamina") < upfront_stamina_cost {
        return Err("Not enough stamina to start action".into());
    }
    if action.event_type.eq_ignore_ascii_case("sponsor") {
        if action.sponsor_quest_id.trim().is_empty() || action.sponsor_object_id.trim().is_empty() {
            return Err("Sponsor action is missing its objective or sponsored object".into());
        }
        let already_member = game
            .quest_memberships
            .iter()
            .any(|membership| membership.quest_id == action.sponsor_quest_id);
        if !already_member {
            let quest = game
                .catalog
                .quests
                .iter()
                .find(|quest| quest.id == action.sponsor_quest_id)
                .ok_or_else(|| {
                    format!(
                        "Sponsor action references an unknown {}",
                        label(&game.catalog, "quest_name", "quest").to_lowercase()
                    )
                })?;
            if quest.level > 1
                && !game.player.inventory.iter().any(|object| {
                    object.object_type == "achievements" && object.trophy_level == quest.level - 1
                })
            {
                return Err(format!(
                    "Cannot join '{}': a level {} trophy is required.",
                    quest.name,
                    quest.level - 1
                ));
            }
            if !quest.required_license_id.trim().is_empty()
                && !game.player.inventory.iter().any(|object| {
                    object.id == quest.required_license_id
                        || object
                            .id
                            .starts_with(&format!("{}_", quest.required_license_id))
                })
            {
                return Err(format!(
                    "Cannot join '{}': required licence '{}' is missing.",
                    quest.name, quest.required_license_id
                ));
            }
        }
        if !game
            .catalog
            .objects
            .iter()
            .any(|object| object.id == action.sponsor_object_id)
        {
            return Err("Sponsor action references an unknown object".into());
        }
        for equipment_id in action
            .sponsor_equipment_ids
            .split(';')
            .map(str::trim)
            .filter(|id| !id.is_empty())
        {
            if !game
                .catalog
                .objects
                .iter()
                .any(|object| object.id == equipment_id)
            {
                return Err(format!(
                    "Sponsor action references unknown equipment '{}'",
                    equipment_id
                ));
            }
        }
    }
    let follow_up_encounter: Option<(String, String)> =
        if action.resolution_method != "encounter" && action.encounter_id.trim().is_empty() {
            None
        } else {
            let encounter_config = game
                .catalog
                .encounter_configs
                .iter()
                .find(|config| config.encounter_id == action.encounter_id)
                .ok_or_else(|| {
                    format!(
                        "Action '{}' references an unknown encounter '{}'",
                        action.name, action.encounter_id
                    )
                })?;
            let opponent_id = if encounter_config.opponent_id.trim().is_empty() {
                return Err(format!(
                    "Encounter '{}' does not define an opponent",
                    action.encounter_id
                ));
            } else {
                encounter_config.opponent_id.clone()
            };
            if !game
                .catalog
                .encounter_opponents
                .iter()
                .any(|opponent| opponent.opponent_id == opponent_id)
            {
                return Err(format!(
                    "Encounter '{}' references an unknown opponent '{}'",
                    action.encounter_id, opponent_id
                ));
            }
            Some((action.encounter_id.clone(), opponent_id))
        };
    if (action.payout_freq_type.eq_ignore_ascii_case("recurring")
        || event_has_obligation(&game, &action.id))
        && game
            .player
            .active_events
            .iter()
            .any(|active| active.event_id == action.id)
    {
        return Err("This action is already active".into());
    }
    if action.event_type.eq_ignore_ascii_case("work")
        && game.player.active_events.iter().any(|active| {
            game.catalog
                .events
                .iter()
                .find(|candidate| candidate.id == active.event_id)
                .map(|candidate| candidate.event_type.eq_ignore_ascii_case("work"))
                .unwrap_or(false)
        })
    {
        return Err("You can only have one job at a time".into());
    }

    adjust_characteristic(&mut game, "budget", -action.base_cost);
    adjust_characteristic(&mut game, "stamina", -upfront_stamina_cost);
    game.player.last_event_day = Some(game.current_day);
    let success = if action.event_type.eq_ignore_ascii_case("sponsor") {
        roll(&mut game) <= action.success_rate
    } else if action.resolution_method == "encounter" {
        true
    } else {
        roll(&mut game) <= action.success_rate
    };
    let payout = if success && !action.payout_freq_type.eq_ignore_ascii_case("recurring") {
        action.payout
    } else {
        0.0
    };
    adjust_characteristic(&mut game, "budget", payout);
    if success
        && (action.payout_freq_type.eq_ignore_ascii_case("recurring")
            || event_has_obligation(&game, &action.id))
        && !action.event_type.eq_ignore_ascii_case("sponsor")
    {
        let start_day = game.current_day;
        game.player.active_events.push(ActiveEvent {
            event_id: action.id.clone(),
            start_day,
            obligation_payments: 0,
            obligation_faults: 0,
        });
    }
    let action_context = TriggerContext {
        trigger_type: "event_completed".into(),
        trigger_ref: action.id.clone(),
        source_type: "event".into(),
        source_id: action.id.clone(),
        outcome: Some(if success { "success" } else { "failure" }.into()),
        ..TriggerContext::default()
    };
    let current_day = game.current_day;
    evaluate_cost_rules(&mut game, &action_context, current_day)?;
    log_event(
        &mut game,
        if success {
            format!("Action completed: {}", action.name)
        } else {
            format!("Action failed: {}", action.name)
        },
    );
    if success {
        if let Some((encounter_id, opponent_id)) = follow_up_encounter {
            if action.event_type.eq_ignore_ascii_case("sponsor") {
                game.pending_sponsor_event_id = Some(action.id.clone());
            }
            let encounter = engine::encounter::start(
                &game.catalog,
                &encounter_id,
                &opponent_id,
                &game.player.characteristics,
                game.rng_state,
            )?;
            game.active_encounter = Some(encounter);
        }
    }

    Ok(EventStartResult {
        event_name: action.name.clone(),
        success,
        payout_received: payout,
        cost_paid: action.base_cost,
        message: if success {
            if action.event_type.eq_ignore_ascii_case("sponsor") {
                format!(
                    "Started '{}'. Win the sponsor challenge to receive the {} deal.",
                    action.name,
                    label(&game.catalog, "quest_name", "quest").to_lowercase()
                )
            } else if action.payout_freq_type.eq_ignore_ascii_case("recurring") {
                format!(
                    "Started '{}'. It pays {} every {}.",
                    action.name, action.payout, action.payout_freq_unit
                )
            } else {
                format!("Completed '{}'.", action.name)
            }
        } else {
            label(
                &game.catalog,
                "action_failed_message",
                "Getting '{action}' did not work.",
            )
            .replace("{action}", &action.name)
        },
    })
}

#[tauri::command]
fn quit_event(event_id: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    let index = game
        .player
        .active_events
        .iter()
        .position(|active| active.event_id == event_id)
        .ok_or_else(|| "That recurring action is not active".to_string())?;
    game.player.active_events.remove(index);
    Ok(game.clone())
}

#[tauri::command]
fn enter_event(
    object_id: String,
    event_id: String,
    state: State<'_, AppState>,
) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    let day_of_year = ((game.current_day - 1) % game.days_per_year) + 1;
    let event = game
        .catalog
        .events
        .iter()
        .find(|event| event.id == event_id)
        .cloned()
        .ok_or_else(|| "Event not found in catalog".to_string())?;
    if !event.quest_id.trim().is_empty()
        && !game
            .quest_memberships
            .iter()
            .any(|membership| membership.quest_id == event.quest_id)
    {
        let quest_name = game
            .catalog
            .quests
            .iter()
            .find(|quest| quest.id == event.quest_id)
            .map(|quest| quest.name.as_str())
            .unwrap_or("the quest");
        return Err(format!("Join '{}' before entering its events", quest_name));
    }
    if event.day_of_year != day_of_year {
        return Err(format!(
            "Event is scheduled for day {}, today is day {}.",
            event.day_of_year, day_of_year
        ));
    }

    if characteristic_value(&game.player, "budget") < event.entry_fee {
        return Err("Insufficient funds for event entry".into());
    }
    let race_event = event.tags.split(';').any(|tag| normalized(tag) == "race");
    let event_stamina_cost = if race_event {
        config_f64(&game.catalog, "race_day_stamina_cost", 20.0)
            * event_duration_days(&event).max(1) as f64
    } else {
        event.stamina_cost.max(0.0)
    };
    if characteristic_value(&game.player, "stamina") < event_stamina_cost {
        return Err(format!(
            "Not enough stamina to enter '{}': {} required.",
            event.name, event_stamina_cost
        ));
    }
    if !event.required_license_id.trim().is_empty()
        && !game.player.inventory.iter().any(|object| {
            object.id == event.required_license_id
                || object
                    .id
                    .starts_with(&format!("{}_", event.required_license_id))
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
    if player_object_does_not_match_requirement(
        &game,
        &game.player.inventory[index],
        &event.required_object_ids,
    ) {
        return Err(format!(
            "Cannot enter '{}': an eligible {} is required (allowed: {}).",
            event.name,
            label(&game.catalog, "object_name", "object").to_lowercase(),
            event.required_object_ids
        ));
    }
    if let Some(requirements) = object_requirement_error(&game, &game.player.inventory[index]) {
        return Err(format!("Cannot enter '{}': {}.", event.name, requirements));
    }

    let entry_id = format!("event_entry_{}_{}", event.id, game.current_day);
    if game.pending_events.iter().any(|entry| entry.id == entry_id)
        || game
            .event_history
            .iter()
            .any(|entry| entry.event_id == event.id && entry.entered_day == game.current_day)
    {
        return Err("This event has already been entered today".into());
    }
    adjust_characteristic(&mut game, "budget", -event.entry_fee);
    adjust_characteristic(&mut game, "stamina", -event_stamina_cost);
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
    if game.player.inventory[index].loaned {
        return Err("Loaned sponsor objects cannot be sold".into());
    }
    if game.player.inventory[index].object_type == "license" {
        return Err("Licences cannot be resold".into());
    }
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
    let sold_object_name = game.player.inventory[index].name.clone();
    adjust_characteristic(&mut game, "budget", value);
    log_event(
        &mut game,
        format!("{} sold for {}", sold_object_name, value),
    );
    game.player.inventory.remove(index);
    Ok(game.clone())
}

#[tauri::command]
fn submit_event_result(
    entry_id: String,
    result: String,
    damage_type: String,
    player_position: Option<u32>,
    competitors: Vec<ChampionshipCompetitor>,
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
    let entry = game.pending_events[entry_index].clone();
    let event = game
        .catalog
        .events
        .iter()
        .find(|event| event.id == entry.event_id)
        .cloned()
        .ok_or_else(|| "Event not found in catalog".to_string())?;
    if !event.quest_id.trim().is_empty() {
        let player_position = player_position.unwrap_or(0);
        let mut positions = std::collections::HashSet::new();
        if player_position > 0 {
            positions.insert(player_position);
        }
        for competitor in &competitors {
            if competitor.name.trim().is_empty() || competitor.position == 0 {
                continue;
            }
            if !positions.insert(competitor.position) {
                return Err(format!(
                    "{} finishing position {} is already assigned",
                    label(&game.catalog, "quest_name", "Quest"),
                    competitor.position
                ));
            }
        }
    }
    game.pending_events.remove(entry_index);
    let reported_success = if event.resolution_method.eq_ignore_ascii_case("random") {
        roll(&mut game) <= event.success_rate
    } else {
        matches!(
            normalized(&result).as_str(),
            "success" | "successful" | "win" | "won" | "1" | "yes" | "true"
        )
    };
    let random_outcome = if reported_success {
        game.catalog
            .event_outcomes
            .clone()
            .into_iter()
            .find(|outcome| {
                outcome.event_id == event.id
                    && outcome.outcome_id.eq_ignore_ascii_case("failure")
                    && outcome.probability > 0.0
                    && roll(&mut game) < outcome.probability.clamp(0.0, 1.0)
            })
    } else {
        None
    };
    let random_result = if reported_success && random_outcome.is_none() {
        let tags = event_tags(&event);
        game.catalog
            .event_results
            .clone()
            .into_iter()
            .find(|candidate| {
                (candidate.event_id.trim().is_empty() || candidate.event_id == event.id)
                    && normalized(&candidate.reported_result) == "success"
                    && candidate
                        .event_tags
                        .split(';')
                        .map(str::trim)
                        .filter(|tag| !tag.is_empty())
                        .all(|required| {
                            tags.iter()
                                .any(|actual| normalized(actual) == normalized(required))
                        })
                    && roll(&mut game) < candidate.probability.clamp(0.0, 1.0)
            })
    } else {
        None
    };
    let success = reported_success && random_outcome.is_none() && random_result.is_none();
    let reward = if success {
        event.reward_pool
    } else if let Some(outcome) = &random_outcome {
        outcome.reward_pool_delta
    } else {
        random_result
            .as_ref()
            .map(|outcome| outcome.reward_pool_delta)
            .unwrap_or(0.0)
    };
    let charisma_reward = if let Some(outcome) = &random_outcome {
        outcome.charisma_reward_delta
    } else if success {
        event.charisma_reward
    } else {
        0.0
    };
    adjust_characteristic(&mut game, "budget", reward);
    adjust_characteristic(&mut game, "charisma", charisma_reward);
    if let Some(event_result) = &random_result {
        for effect in event_result.effects.split(';') {
            let mut parts = effect.splitn(2, ':');
            let Some(characteristic) = parts
                .next()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                continue;
            };
            let Some(delta) = parts
                .next()
                .and_then(|value| value.trim().parse::<f64>().ok())
            else {
                continue;
            };
            adjust_characteristic(&mut game, characteristic, delta);
        }
    }
    let sponsor_payment = if !event.quest_id.trim().is_empty() {
        let position = player_position.unwrap_or(0);
        let sponsor_action = game.player.active_events.iter().find_map(|active| {
            let action = game
                .catalog
                .events
                .iter()
                .find(|action| action.id == active.event_id)?;
            (action.event_type.eq_ignore_ascii_case("sponsor")
                && action.sponsor_quest_id == event.quest_id)
                .then_some(action)
        });
        let payment = sponsor_action
            .map(|action| sponsor_payout(action, position))
            .unwrap_or(0.0);
        adjust_characteristic(&mut game, "budget", payment);
        payment
    } else {
        0.0
    };
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
    if event.tags.split(';').any(|tag| normalized(tag) == "race") {
        game.last_race_day = Some(current_day);
    }
    if !event.quest_id.trim().is_empty() && player_position.unwrap_or(0) > 0 {
        game.championship_results.push(ChampionshipResult {
            event_id: event.id.clone(),
            race_day: entry.entered_day,
            player_position: player_position.unwrap_or(0),
            competitors,
        });
    }
    let is_final_championship_race = !event.quest_id.trim().is_empty()
        && (event.tags.split(';').any(|tag| normalized(tag) == "finale")
            || !game.catalog.events.iter().any(|candidate| {
                candidate.quest_id == event.quest_id && candidate.day_of_year > event.day_of_year
            }));
    if success && is_final_championship_race && matches!(player_position, Some(1..=3)) {
        if let Some(quest) = game
            .catalog
            .quests
            .iter()
            .find(|quest| quest.id == event.quest_id)
            .cloned()
        {
            if let Some(base_trophy) = game
                .catalog
                .objects
                .iter()
                .find(|object| object.object_type == "achievements")
                .cloned()
            {
                let position = player_position.unwrap_or_default();
                let trophy_id = format!("trophy_{}_{}_level_{}", quest.id, position, quest.level);
                if !game
                    .player
                    .inventory
                    .iter()
                    .any(|object| object.id == trophy_id)
                {
                    let mut trophy = base_trophy;
                    trophy.id = trophy_id;
                    trophy.name = format!(
                        "{} - {} place Trophy (Level {})",
                        quest.name, position, quest.level
                    );
                    trophy.trophy_championship = quest.name;
                    trophy.trophy_position = position;
                    trophy.trophy_level = quest.level;
                    let snapshot = game.clone();
                    game.player
                        .inventory
                        .push(build_owned_object(&trophy, &snapshot, false, 0));
                }
            }
        }
    }
    game.event_history.push(EventHistory {
        id: entry.id,
        event_id: event.id.clone(),
        object_id: entry.object_id,
        entered_day: entry.entered_day,
        result: result.clone(),
        outcome: if success {
            "Success".into()
        } else {
            "Unsuccessful".into()
        },
        reward_awarded: reward,
        charisma_reward_awarded: charisma_reward,
        damage_type: damage_type.clone(),
    });
    log_event(
        &mut game,
        format!("Event finished: {} ({})", event.name, result),
    );

    Ok(EventResult {
        event_name: event.name,
        outcome: if success {
            "Success".into()
        } else {
            "Unsuccessful".into()
        },
        entry_fee_paid: event.entry_fee,
        reward_awarded: reward,
        charisma_reward_awarded: charisma_reward,
        message: if let Some(outcome) = random_outcome {
            if outcome.message.trim().is_empty() {
                format!("The event went wrong: {}.", result)
            } else {
                outcome.message
            }
        } else if let Some(outcome) = random_result {
            if outcome.message.trim().is_empty() {
                format!("The event went wrong: {}.", result)
            } else {
                outcome.message
            }
        } else if success {
            format!(
                "The event was successful: {}. Rewards: {} budget and {} charisma{}.",
                result,
                reward,
                charisma_reward,
                if sponsor_payment > 0.0 {
                    format!("; sponsor payment {}", sponsor_payment)
                } else {
                    String::new()
                }
            )
        } else {
            format!(
                "The event ended without a reward: {}{}.",
                result,
                if sponsor_payment > 0.0 {
                    format!(" Sponsor payment: {}", sponsor_payment)
                } else {
                    String::new()
                }
            )
        },
        sponsor_payment,
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

#[tauri::command]
fn load_dataset_asset(path: String, state: State<'_, AppState>) -> Result<String, String> {
    let game = state.0.lock().map_err(|e| e.to_string())?;
    let base = Path::new(&game.dataset_path)
        .canonicalize()
        .map_err(|error| format!("Dataset folder cannot be read: {error}"))?;
    let canonical = base
        .join(path)
        .canonicalize()
        .map_err(|error| format!("Dataset asset cannot be read: {error}"))?;
    if !canonical.starts_with(&base) {
        return Err("Dataset asset path must remain inside the dataset folder".into());
    }
    let mime = match canonical
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => return Err("Unsupported dataset image format".into()),
    };
    let bytes = std::fs::read(&canonical)
        .map_err(|error| format!("Dataset asset cannot be read: {error}"))?;
    Ok(format!("data:{mime};base64,{}", BASE64.encode(bytes)))
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
            get_theme_colors,
            default_dataset_dialog_path,
            save_game,
            load_game,
            list_save_slots,
            latest_save_slot,
            start_new_game,
            save_game_as,
            load_game_from,
            set_time_speed,
            tick_game_day,
            pay_cost,
            buy_object,
            service_object,
            sell_object,
            perform_event,
            quit_event,
            enter_event,
            join_quest,
            submit_event_result,
            load_description,
            load_dataset_asset,
            dismiss_alert,
            toggle_alarm,
            set_popup_categories,
            reload_dataset,
            start_encounter,
            resolve_encounter_turn,
            retreat_encounter
        ])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
