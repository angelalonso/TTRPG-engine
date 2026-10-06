pub mod engine;
pub mod headless;
pub mod rng;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use engine::encounter::{EncounterResult, EncounterState};
use engine::loader::{
    EffectData, EventData, GameCatalog, ObjectData, ObligationData, TransferPolicy,
};
use engine::plugin::{PluginExecutionLimits, PluginOperation};
use engine::quest_runs::{
    CompletionRule, EnrollmentPolicy, QuestDefinition, QuestRun, QuestRunStatus, RepeatPolicy,
    RewardReceipt,
};
use rand::RngExt;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write as IoWrite};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Mutex;
use tauri::{Manager, State};

pub(crate) fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub dataset_path: String,
    pub fullscreen: bool,
    pub window_width: u32,
    pub window_height: u32,
    #[serde(default)]
    pub game_directory: String,
    #[serde(default)]
    pub results_directory: String,
    #[serde(default)]
    pub driver_names: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            dataset_path: "dataset".to_string(),
            fullscreen: false,
            window_width: 1440,
            window_height: 900,
            game_directory: r"C:\Program Files (x86)\Steam\steamapps\common\GTR 2 - FIA GT Racing Game".to_string(),
            results_directory: r"C:\Program Files (x86)\Steam\steamapps\common\GTR 2 - FIA GT Racing Game\UserData\Log\Results".to_string(),
            driver_names: Vec::new(),
        }
    }
}

fn app_config_path() -> PathBuf {
    PathBuf::from("cfg.yml")
}

fn read_app_config_file() -> AppConfig {
    let mut config = AppConfig::default();
    let Ok(contents) = std::fs::read_to_string(app_config_path()) else {
        return config;
    };
    for line in contents.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value
            .trim()
            .trim_matches(|character| character == '"' || character == '\'')
            .trim();
        match key.trim() {
            "dataset_path" if !value.is_empty() => config.dataset_path = value.to_string(),
            "fullscreen" => config.fullscreen = value.eq_ignore_ascii_case("true"),
            "window_width" => config.window_width = value.parse().unwrap_or(config.window_width).max(800),
            "window_height" => config.window_height = value.parse().unwrap_or(config.window_height).max(600),
            "game_directory" if !value.is_empty() => {
                config.game_directory = value.replace("\\\\", "\\")
            }
            "results_directory" if !value.is_empty() => {
                config.results_directory = value.replace("\\\\", "\\")
            }
            "driver_names" => {
                config.driver_names = value
                    .trim_matches(|character| character == '[' || character == ']')
                    .split(',')
                    .map(|name| name.trim().trim_matches('"').trim_matches('\'').to_string())
                    .filter(|name| !name.is_empty())
                    .collect();
            }
            _ => {}
        }
    }
    config
}

fn write_app_config_file(config: &AppConfig) -> Result<(), String> {
    let dataset_path = config
        .dataset_path
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let game_directory = config
        .game_directory
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let results_directory = config
        .results_directory
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let contents = format!(
        "dataset_path: \"{}\"\nfullscreen: {}\nwindow_width: {}\nwindow_height: {}\ngame_directory: \"{}\"\nresults_directory: \"{}\"\ndriver_names: [{}]\n",
        dataset_path,
        config.fullscreen,
        config.window_width,
        config.window_height,
        game_directory,
        results_directory,
        config.driver_names.iter().map(|name| format!("\"{}\"", name.replace('"', "\\\""))).collect::<Vec<_>>().join(", ")
    );
    std::fs::write(app_config_path(), contents)
        .map_err(|error| format!("Cannot save cfg.yml: {error}"))
}

#[tauri::command]
fn get_app_config() -> AppConfig {
    read_app_config_file()
}

#[tauri::command]
fn save_app_config(config: AppConfig) -> Result<AppConfig, String> {
    let mut config = config;
    if config.game_directory.trim().is_empty() {
        config.game_directory = read_app_config_file().game_directory;
    }
    if config.results_directory.trim().is_empty() {
        config.results_directory = read_app_config_file().results_directory;
    }
    write_app_config_file(&config)?;
    Ok(config)
}

fn configured_game_results_directory() -> String {
    let config = read_app_config_file();
    if !config.results_directory.trim().is_empty() {
        return config.results_directory;
    }
    PathBuf::from(config.game_directory)
        .join("UserData")
        .join("Log")
        .join("Results")
        .to_string_lossy()
        .into_owned()
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OwnedObject {
    pub id: String,
    #[serde(default)]
    pub definition_id: String,
    #[serde(default)]
    pub instance_id: String,
    pub object_type: String,
    pub name: String,
    pub price: f64,
    #[serde(default)]
    pub policy_version: u32,
    #[serde(default = "crate::default_true")]
    pub buyable: bool,
    #[serde(default = "crate::default_true")]
    pub sellable: bool,
    #[serde(default)]
    pub reward_only: bool,
    #[serde(default)]
    pub unique: bool,
    #[serde(default)]
    pub max_owned: u32,
    #[serde(default)]
    pub use_policy: String,
    #[serde(default)]
    pub consume_policy: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ObligationState {
    #[serde(default)]
    pub payments: u32,
    #[serde(default)]
    pub faults: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActiveEvent {
    #[serde(alias = "action_id")]
    pub event_id: String,
    pub start_day: u32,
    // Kept only so saves written before per-obligation state was introduced
    // remain readable. New saves persist obligation_states instead.
    #[serde(default, skip_serializing)]
    pub obligation_payments: u32,
    #[serde(default, skip_serializing)]
    pub obligation_faults: u32,
    #[serde(default)]
    pub obligation_states: HashMap<String, ObligationState>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChampionshipStanding {
    pub name: String,
    pub points: u32,
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
    #[serde(default = "default_save_version")]
    pub save_version: u32,
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
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
    pub quest_runs: Vec<QuestRun>,
    #[serde(default)]
    pub reward_receipts: Vec<RewardReceipt>,
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
    pub sponsor_contracts: Vec<SponsorContract>,
    #[serde(default)]
    pub manager_sponsor_offer_ids: Vec<String>,
    #[serde(default)]
    pub manager_sponsor_offer_cooldown_until_day: u32,
    #[serde(default)]
    pub rng_state: u64,
    #[serde(default)]
    pub alarm_event_ids: Vec<String>,
    #[serde(default)]
    pub popup_categories: Vec<String>,
    #[serde(default = "default_next_object_instance_id")]
    pub next_object_instance_id: u64,
}

impl GameState {
    pub fn has_reward_receipt(&self, source_run_id: &str, reward_id: &str) -> bool {
        self.reward_receipts
            .iter()
            .any(|receipt| receipt.has_identity(source_run_id, reward_id))
    }

    pub fn record_reward_receipt(&mut self, receipt: RewardReceipt) -> Result<(), String> {
        if self.has_reward_receipt(&receipt.source_run_id, &receipt.reward_id)
            || self
                .reward_receipts
                .iter()
                .any(|existing| existing.receipt_id == receipt.receipt_id)
        {
            return Err(format!(
                "Reward receipt '{}' has already been recorded",
                receipt.receipt_id
            ));
        }
        self.reward_receipts.push(receipt);
        Ok(())
    }
}

fn quest_run_definition(game: &GameState, quest_id: &str) -> Result<QuestDefinition, String> {
    let quest = game
        .catalog
        .quests
        .iter()
        .find(|quest| quest.id == quest_id)
        .ok_or_else(|| format!("Quest '{quest_id}' is not configured"))?;
    let split_ids = |value: &str| {
        value
            .split(';')
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .map(str::to_string)
            .collect()
    };
    let enrollment_policy = match quest.enrollment_policy.trim().to_ascii_lowercase().as_str() {
        "automatic" => EnrollmentPolicy::Automatic,
        "scheduled" => EnrollmentPolicy::Scheduled,
        _ => EnrollmentPolicy::Manual,
    };
    let repeat_policy = match quest.repeat_policy.trim().to_ascii_lowercase().as_str() {
        "repeatable" => RepeatPolicy::Repeatable,
        "periodic" => RepeatPolicy::Periodic,
        _ => RepeatPolicy::Once,
    };
    let required_event_ids: Vec<String> = split_ids(&quest.required_event_ids);
    let completion = match quest.completion_mode.trim().to_ascii_lowercase().as_str() {
        "points" => CompletionRule::Points {
            target: quest.success_points,
        },
        "noncompetitive" | "non_competitive" => CompletionRule::NonCompetitive {
            required_successes: required_event_ids.len().max(1),
        },
        _ => CompletionRule::AllRequired,
    };
    Ok(QuestDefinition {
        quest_id: quest.id.clone(),
        enrollment_policy,
        repeat_policy,
        completion,
        required_event_ids,
        optional_event_ids: split_ids(&quest.optional_event_ids),
    })
}

/// Finalize a generic persistent quest run and issue its reward at most once.
///
/// This is deliberately separate from the legacy racing finalization path.
/// The configured quest definition supplies the completion rule; the run and
/// receipt remain the durable source of truth across retries and reloads.
pub fn finalize_quest_run_for_sim(
    game: &mut GameState,
    run_id: &str,
    reward_id: &str,
) -> Result<Option<RewardReceipt>, String> {
    if run_id.trim().is_empty() || reward_id.trim().is_empty() {
        return Err("Quest run and reward IDs cannot be empty".into());
    }
    let run_index = game
        .quest_runs
        .iter()
        .position(|run| run.run_id == run_id)
        .ok_or_else(|| format!("Quest run '{run_id}' was not found"))?;
    if game.quest_runs[run_index].status == QuestRunStatus::Finalized {
        return game
            .reward_receipts
            .iter()
            .find(|receipt| receipt.has_identity(run_id, reward_id))
            .cloned()
            .map(Some)
            .ok_or_else(|| {
                format!("Quest run '{run_id}' is finalized without reward '{reward_id}'")
            });
    }

    let quest_id = game.quest_runs[run_index].quest_id.clone();
    let quest_level = game
        .catalog
        .quests
        .iter()
        .find(|quest| quest.id == quest_id)
        .map(|quest| quest.level)
        .ok_or_else(|| format!("Quest '{quest_id}' is not configured"))?;
    let definition = quest_run_definition(game, &quest_id)?;
    let mut staged = game.clone();
    let completion_status = {
        let run = staged
            .quest_runs
            .get_mut(run_index)
            .expect("quest run index was obtained from the original state");
        run.finalize(&definition.completion)
            .map_err(|error| error.to_string())?;
        run.status.clone()
    };

    let receipt = if completion_status == QuestRunStatus::Completed {
        if let Some(receipt) = staged
            .reward_receipts
            .iter()
            .find(|receipt| receipt.has_identity(run_id, reward_id))
            .cloned()
        {
            Some(receipt)
        } else {
            let receipt = RewardReceipt::new(
                run_id,
                reward_id,
                None,
                Some(quest_level.to_string()),
                std::collections::BTreeMap::from([
                    ("quest_id".into(), definition.quest_id.clone()),
                    ("source".into(), "quest-run-finalization".into()),
                ]),
                staged.current_day,
            );
            staged.record_reward_receipt(receipt.clone())?;
            Some(receipt)
        }
    } else {
        None
    };

    staged.quest_runs[run_index]
        .mark_finalized()
        .map_err(|error| error.to_string())?;
    *game = staged;
    Ok(receipt)
}

fn default_next_object_instance_id() -> u64 {
    1
}

fn default_save_version() -> u32 {
    engine::save::CURRENT_SAVE_VERSION
}

fn default_schema_version() -> u32 {
    engine::save::CURRENT_SCHEMA_VERSION
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
    if game.catalog.terminal_conditions.currency_below_zero
        && game
            .catalog
            .resource_roles
            .currency
            .as_ref()
            .and_then(|id| game.player.characteristics.get(id))
            .is_some_and(|value| *value < 0.0)
    {
        return RunStatus::DeadMoney;
    }
    if game.catalog.terminal_conditions.recovery_at_or_below_zero
        && game
            .catalog
            .resource_roles
            .recovery
            .as_ref()
            .and_then(|id| game.player.characteristics.get(id))
            .is_some_and(|value| *value <= 0.0)
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
    #[serde(default)]
    pub rented: bool,
    #[serde(default)]
    pub rental_expires_day: u32,
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
    #[serde(default)]
    pub player_position: u32,
    #[serde(default)]
    pub pole_position: bool,
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
pub struct EventEligibility {
    pub event_id: String,
    pub selection_id: String,
    pub definition_id: String,
    pub available: bool,
    pub rented: bool,
    pub reason: String,
    #[serde(default)]
    pub reason_code: String,
    #[serde(default)]
    pub failed_facts: Vec<String>,
    pub entry_fee: f64,
    pub stamina_cost: f64,
    pub rental_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectTransactionEligibility {
    pub object_id: String,
    pub owned_count: u32,
    pub can_acquire: bool,
    pub acquire_reason: String,
    pub can_sell: bool,
    pub sell_reason: String,
    pub buyable: bool,
    pub sellable: bool,
    pub reward_only: bool,
    pub max_owned: u32,
    pub transfer_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResult {
    pub event_name: String,
    pub outcome: String,
    pub entry_fee_paid: f64,
    pub reward_awarded: f64,
    pub charisma_reward_awarded: f64,
    pub sponsor_payment: f64,
    #[serde(default)]
    pub sponsor_bonus: f64,
    pub message: String,
    pub damage_type: String,
    #[serde(default)]
    pub championship_standings: Vec<ChampionshipStanding>,
}

pub struct AppState(pub Mutex<GameState>);

pub(crate) struct SponsorNegotiationProcessState(pub(crate) Mutex<Option<SponsorNegotiationProcess>>);

pub(crate) struct SponsorNegotiationProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    event_id: String,
    target_id: String,
    target_name: String,
    car_object_id: String,
    stamina_cost: f64,
}

#[derive(Debug, Clone, Serialize)]
struct SponsorNegotiationStart {
    state: serde_json::Value,
    game_state: GameState,
}

#[derive(Debug, Clone, Serialize)]
struct SponsorNegotiationActionResult {
    state: serde_json::Value,
    game_state: GameState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceResultsPluginResponse {
    result: String,
    player_position: u32,
    competitors: Vec<ChampionshipCompetitor>,
    #[serde(default)]
    damage_type: String,
    #[serde(default)]
    pole_position: bool,
    #[serde(default)]
    standings: Vec<ChampionshipStanding>,
    #[serde(default)]
    driver_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SponsorContract {
    pub id: String,
    pub sponsor_id: String,
    pub sponsor_name: String,
    pub sponsor_tier: String,
    pub scope: String,
    #[serde(default)]
    pub race_tier: String,
    #[serde(default)]
    pub target_id: String,
    #[serde(default)]
    pub target_name: String,
    pub signed_day: u32,
    pub expires_day: u32,
    pub initial_money: f64,
    pub monthly_payment: f64,
    pub entry_fees: bool,
    pub maintenance: bool,
    pub repair_coverage: f64,
    pub car: bool,
    #[serde(default)]
    pub car_object_id: String,
    pub gear: bool,
    pub result_bonus: f64,
    pub dnf_penalty: f64,
    #[serde(default)]
    pub last_payment_day: u32,
}

fn run_race_results_plugin(
    game: &GameState,
    event: &EventData,
    result: &str,
    player_position: Option<u32>,
    competitors: &[ChampionshipCompetitor],
    interactive: bool,
    autodetect: bool,
) -> Result<RaceResultsPluginResponse, String> {
    if !event.plugin_id.trim().is_empty() {
        let manifest = game
            .catalog
            .plugin_manifest(&event.plugin_id)?
            .ok_or_else(|| format!("Declared plugin '{}' was not found", event.plugin_id))?;
        let request_payload = serde_json::json!({
            "result": result,
            "interactive": interactive,
            "autodetect": autodetect,
            "dataset_path": game.dataset_path,
            "event": event,
            "description": std::fs::read_to_string(
                Path::new(&game.dataset_path).join(&event.description_html)
            ).unwrap_or_else(|_| event.description_html.clone()),
            "max_reward_position": event.position_rewards
                .split(';')
                .filter_map(|value| value.trim().parse::<u32>().ok())
                .max()
                .unwrap_or(0),
            "player_position": player_position,
            "player_name": game.player.name,
            "driver_names": read_app_config_file().driver_names,
            "competitors": competitors,
            "results_directory": configured_game_results_directory(),
        });
        let response = manifest
            .execute_operation(
                Path::new(&game.dataset_path),
                PluginOperation::ProvideResult,
                request_payload,
                PluginExecutionLimits::default(),
            )
            .map_err(|error| error.to_string())?;
        let result = response
            .results
            .first()
            .ok_or_else(|| "Declared plugin returned no result".to_string())?;
        return serde_json::from_value(result.value.clone())
            .map_err(|error| format!("Declared plugin returned an invalid race result: {error}"));
    }
    let configured_path = std::env::var("TTRPG_RACE_RESULTS_PLUGIN")
        .unwrap_or_else(|_| "dataset/plugins/race_results.py".into());
    let configured_path = PathBuf::from(configured_path);
    let plugin_path = if configured_path.exists() {
        configured_path
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dataset/plugins/race_results.py")
    };
    if !plugin_path.is_file() {
        return Err(format!(
            "Race-results plugin was not found at '{}'. Set TTRPG_RACE_RESULTS_PLUGIN to its path.",
            plugin_path.display()
        ));
    }

    let request = serde_json::json!({
        "result": result,
        "interactive": interactive,
        "autodetect": autodetect,
        "dataset_path": game.dataset_path,
        "event": event,
        "description": std::fs::read_to_string(
            Path::new(&game.dataset_path).join(&event.description_html)
        ).unwrap_or_else(|_| event.description_html.clone()),
        "max_reward_position": event.position_rewards
            .split(';')
            .filter_map(|entry| entry.split(':').next()?.trim().parse::<u32>().ok())
            .max()
            .unwrap_or(if event.reward_pool > 0.0 { 3 } else { 1 }),
        "damage_options": game.catalog.cost_rules.iter()
            .filter(|rule| !rule.damage_type.trim().is_empty())
            .map(|rule| {
                serde_json::json!({
                    "id": rule.damage_type,
                    "name": game.catalog.costs.iter()
                        .find(|cost| cost.id == rule.cost_id)
                        .map(|cost| cost.name.clone())
                        .unwrap_or_else(|| rule.damage_type.clone()),
                })
            })
            .collect::<Vec<_>>(),
        "player_position": player_position.unwrap_or(0),
        "player_name": game.player.name,
        "driver_names": read_app_config_file().driver_names,
        "competitors": competitors,
        "previous_results": game.championship_results,
        "events": game.catalog.events,
        "results_directory": configured_game_results_directory(),
    });
    let python = std::env::var("TTRPG_PYTHON").unwrap_or_else(|_| "python3".into());
    let mut child = Command::new(&python)
        .arg(&plugin_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Could not start race-results plugin with '{python}': {error}"))?;
    child
        .stdin
        .take()
        .ok_or_else(|| "Race-results plugin stdin was unavailable".to_string())?
        .write_all(request.to_string().as_bytes())
        .map_err(|error| format!("Could not send data to race-results plugin: {error}"))?;
    let output = child
        .wait_with_output()
        .map_err(|error| format!("Race-results plugin failed to finish: {error}"))?;
    let plugin_log = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !plugin_log.is_empty() {
        eprintln!("[race-results-plugin] {plugin_log}");
    }
    if !output.status.success() {
        let details = plugin_log;
        return Err(if details.is_empty() {
            format!("Race-results plugin exited with {}", output.status)
        } else {
            format!("Race-results plugin failed: {details}")
        });
    }
    let response: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("Race-results plugin returned invalid JSON: {error}"))?;
    if let Some(error) = response.get("error").and_then(serde_json::Value::as_str) {
        return Err(format!("Race-results plugin rejected the result: {error}"));
    }
    serde_json::from_value(response)
        .map_err(|error| format!("Race-results plugin returned an invalid response: {error}"))
}

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

fn refresh_manager_sponsor_offers(game: &mut GameState) {
    let manager_id = label(&game.catalog, "manager_object_id", "manager");
    let has_manager = game.player.inventory.iter().any(|object| {
        object.id == manager_id
            || object.definition_id == manager_id
            || object.instance_id == manager_id
    });
    if !has_manager {
        game.manager_sponsor_offer_ids.clear();
        return;
    }
    game.manager_sponsor_offer_ids.retain(|offer_id| {
        game.catalog.activities.iter().any(|activity| {
            !activity.scheduled
                && activity.id == *offer_id
                && game.catalog.events.iter().any(|event| {
                    event.id == activity.id && event.event_type.eq_ignore_ascii_case("sponsor")
                })
        })
    });
    if !game.manager_sponsor_offer_ids.is_empty()
        || game.current_day < game.manager_sponsor_offer_cooldown_until_day
    {
        return;
    }
    let mut candidates: Vec<(String, f64)> = game.catalog.activities.iter()
        .filter(|activity| !activity.scheduled)
        .filter_map(|activity| {
            let event = game.catalog.events.iter().find(|event| {
                event.id == activity.id && event.event_type.eq_ignore_ascii_case("sponsor")
            })?;
            let max_payout = event.sponsor_payouts
                .split(';')
                .filter_map(|entry| entry.split_once(':')?.1.trim().parse::<f64>().ok())
                .fold(0.0, f64::max);
            let car_value = game.catalog.objects.iter()
                .find(|object| object.id == event.sponsor_object_id)
                .map(|object| object.price)
                .unwrap_or(0.0);
            Some((activity.id.clone(), max_payout + car_value))
        })
        .collect();
    candidates.sort_by(|left, right| right.1.total_cmp(&left.1));
    game.manager_sponsor_offer_ids = candidates.into_iter()
        .take(4)
        .map(|(id, _)| id)
        .collect();
}

fn consume_manager_sponsor_offer(game: &mut GameState, event_id: &str) {
    if game.manager_sponsor_offer_ids.iter().any(|id| id == event_id) {
        game.manager_sponsor_offer_ids.retain(|id| id != event_id);
        game.manager_sponsor_offer_cooldown_until_day = game.current_day.saturating_add(14);
    }
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

#[tauri::command]
fn is_dataset_path(path: String) -> bool {
    let directory = Path::new(path.trim());
    directory.is_dir()
        && directory.join("objects.csv").is_file()
        && directory.join("events.csv").is_file()
}

fn config_f64(catalog: &GameCatalog, key: &str, fallback: f64) -> f64 {
    catalog
        .labels
        .values
        .get(key)
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(fallback)
}

fn has_configured_rule(catalog: &GameCatalog, keys: &[&str]) -> bool {
    keys.iter()
        .any(|key| catalog.labels.values.contains_key(*key))
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

fn job_start_blocked(game: &GameState, event: &EventData) -> bool {
    let Some(candidate_obligations) = obligations_for_event(game, &event.id) else {
        return false;
    };
    candidate_obligations.iter().any(|candidate| {
        let group = candidate.active_group.trim();
        if group.is_empty() {
            return false;
        }
        let active_in_group = game.player.active_events.iter().filter(|active| {
            obligations_for_event(game, &active.event_id).is_some_and(|obligations| {
                obligations
                    .iter()
                    .any(|obligation| obligation.active_group.trim().eq_ignore_ascii_case(group))
            })
        });
        candidate.active_exclusive
            || active_in_group.clone().any(|active| {
                obligations_for_event(game, &active.event_id).is_some_and(|obligations| {
                    obligations.iter().any(|obligation| {
                        obligation.active_group.trim().eq_ignore_ascii_case(group)
                            && obligation.active_exclusive
                    })
                })
            })
            || (candidate.active_group_limit > 0
                && active_in_group.count() >= candidate.active_group_limit as usize)
    })
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
    state: &ObligationState,
    current_day: u32,
) -> bool {
    let interval = obligation_interval_days(obligation);
    let elapsed = current_day.saturating_sub(active.start_day);
    if interval == 0 || elapsed == 0 || elapsed % interval != 0 {
        return false;
    }
    if obligation.max_payments > 0 && state.payments >= obligation.max_payments {
        return false;
    }
    let due_days = obligation
        .due_days
        .split(';')
        .filter_map(|value| value.trim().parse::<u32>().ok())
        .collect::<Vec<_>>();
    due_days.is_empty() || due_days.contains(&weekday(current_day))
}

fn obligation_message(
    template: &str,
    state: &ObligationState,
    obligation: &ObligationData,
) -> String {
    template
        .replace("{faults}", &state.faults.to_string())
        .replace("{fault_limit}", &obligation.fault_limit.to_string())
        .replace("{payments}", &state.payments.to_string())
        .replace("{amount}", &obligation.amount.to_string())
}

fn process_obligations(game: &mut GameState, current_day: u32) -> Vec<usize> {
    let obligations = game.catalog.obligations.clone();
    let mut failed_payout_events = Vec::new();
    let mut ended_event_indices = std::collections::HashSet::new();

    for index in 0..game.player.active_events.len() {
        let active_snapshot = game.player.active_events[index].clone();
        let event_obligations = obligations
            .iter()
            .filter(|obligation| obligation.event_id == active_snapshot.event_id)
            .collect::<Vec<_>>();
        if event_obligations.is_empty() {
            continue;
        }

        let legacy_state = if active_snapshot.obligation_states.is_empty() {
            Some(ObligationState {
                payments: active_snapshot.obligation_payments,
                faults: active_snapshot.obligation_faults,
            })
        } else {
            None
        };
        for obligation in event_obligations {
            let state = game.player.active_events[index]
                .obligation_states
                .entry(obligation.id.clone())
                .or_insert_with(|| legacy_state.clone().unwrap_or_default())
                .clone();
            if obligation.skip_when_sick && game.player.sickness_start_day.is_some()
                || !obligation_due_on_day(obligation, &active_snapshot, &state, current_day)
            {
                continue;
            }

            let can_pay = characteristic_value(game, &obligation.resource) >= obligation.amount;
            if can_pay {
                adjust_characteristic(game, &obligation.resource, -obligation.amount);
                game.player.active_events[index]
                    .obligation_states
                    .get_mut(&obligation.id)
                    .expect("obligation state was initialized")
                    .payments = state.payments.saturating_add(1);
                if obligation
                    .completion_consequence
                    .eq_ignore_ascii_case("end_event")
                    && obligation.max_payments > 0
                    && game.player.active_events[index]
                        .obligation_states
                        .get(&obligation.id)
                        .is_some_and(|state| state.payments >= obligation.max_payments)
                {
                    let completion_reward = game
                        .catalog
                        .events
                        .iter()
                        .find(|event| event.id == active_snapshot.event_id)
                        .map(|event| (event.name.clone(), event.charisma_reward));
                    if let Some((event_name, charisma_reward)) = completion_reward {
                        adjust_characteristic(game, "charisma", charisma_reward);
                        if charisma_reward != 0.0 {
                            log_event(
                                game,
                                format!(
                                    "{} completed: {} Paddock Cred awarded",
                                    event_name, charisma_reward
                                ),
                            );
                        }
                    }
                    ended_event_indices.insert(index);
                }
                continue;
            }

            let faults = state.faults.saturating_add(1);
            game.player.active_events[index]
                .obligation_states
                .get_mut(&obligation.id)
                .expect("obligation state was initialized")
                .faults = faults;
            if obligation.fault_blocks_payout {
                failed_payout_events.push(index);
            }
            let fault_title = obligation.fault_title.clone();
            let fault_message = obligation_message(&obligation.fault_message, &state, obligation);
            let fault_log = obligation_message(&obligation.fault_log, &state, obligation);
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

            if obligation.fault_limit > 0 && faults >= obligation.fault_limit {
                let limit_title = obligation.limit_title.clone();
                let limit_message =
                    obligation_message(&obligation.limit_message, &state, obligation);
                let limit_log = obligation_message(&obligation.limit_log, &state, obligation);
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
                    ended_event_indices.insert(index);
                } else if obligation.fault_consequence.eq_ignore_ascii_case("death") {
                    game.player.dead = true;
                    game.time_speed = TimeSpeed::Paused;
                }
            }
        }
    }

    if !ended_event_indices.is_empty() {
        game.player.active_events = game
            .player
            .active_events
            .drain(..)
            .enumerate()
            .filter_map(|(index, active)| (!ended_event_indices.contains(&index)).then_some(active))
            .collect();
    }
    failed_payout_events
}

fn obligations_for_event<'a>(
    game: &'a GameState,
    event_id: &str,
) -> Option<Vec<&'a ObligationData>> {
    let obligations = game
        .catalog
        .obligations
        .iter()
        .filter(|obligation| obligation.event_id == event_id)
        .collect::<Vec<_>>();
    (!obligations.is_empty()).then_some(obligations)
}

fn active_obligation_count(game: &GameState, event_id: &str) -> usize {
    game.player
        .active_events
        .iter()
        .filter(|active| active.event_id == event_id)
        .count()
}

fn required_event_type_is_active(game: &GameState, required_type: &str) -> bool {
    let required_type = normalized(required_type);
    !required_type.is_empty()
        && game.player.active_events.iter().any(|active| {
            game.catalog
                .events
                .iter()
                .find(|event| event.id == active.event_id)
                .is_some_and(|event| normalized(&event.event_type) == required_type)
        })
}

fn mark_event_services_needed(game: &mut GameState, object_id: &str, event: &EventData) {
    if object_id.trim().is_empty() {
        return;
    }
    let tags: Vec<String> = event_tags(event)
        .into_iter()
        .map(|tag| normalized(&tag))
        .collect();
    let configured_cost_ids: Vec<String> = game
        .player
        .inventory
        .iter()
        .find(|object| object.id == object_id)
        .map(|object| {
            [
                object.cost_1.as_str(),
                object.cost_2.as_str(),
                object.cost_3.as_str(),
                object.cost_4.as_str(),
                object.cost_5.as_str(),
                object.cost_6.as_str(),
                object.cost_7.as_str(),
                object.cost_8.as_str(),
                object.cost_9.as_str(),
                object.cost_10.as_str(),
                object.cost_11.as_str(),
                object.cost_12.as_str(),
                object.cost_13.as_str(),
                object.cost_14.as_str(),
                object.cost_15.as_str(),
            ]
            .into_iter()
            .filter(|cost_id| !cost_id.trim().is_empty())
            .map(str::to_string)
            .collect()
        })
        .unwrap_or_default();
    let service_costs: Vec<String> = configured_cost_ids
        .into_iter()
        .filter(|cost_id| {
            game.catalog
                .costs
                .iter()
                .find(|cost| cost.id == *cost_id)
                .is_some_and(|cost| {
                    !cost.event_tags.trim().is_empty()
                        && cost
                            .event_tags
                            .split(';')
                            .map(normalized)
                            .any(|tag| tags.iter().any(|event_tag| event_tag == &tag))
                })
        })
        .collect();
    for cost_id in service_costs {
        let _ = mark_object_service_needed(game, object_id, &cost_id, None);
    }
}

fn selected_event_count(game: &GameState, selector: &str) -> u32 {
    let selector = normalized(selector);
    game.event_history
        .iter()
        .filter(|history| {
            game.catalog
                .events
                .iter()
                .find(|event| event.id == history.event_id)
                .is_some_and(|event| {
                    selector.is_empty()
                        || event_tags(event)
                            .iter()
                            .any(|tag| normalized(tag) == selector)
                })
        })
        .count() as u32
}

fn selected_event_last_day(game: &GameState, selector: &str) -> Option<u32> {
    let selector = normalized(selector);
    game.event_history
        .iter()
        .filter(|history| {
            game.catalog
                .events
                .iter()
                .find(|event| event.id == history.event_id)
                .is_some_and(|event| {
                    selector.is_empty()
                        || event_tags(event)
                            .iter()
                            .any(|tag| normalized(tag) == selector)
                })
        })
        .map(|history| history.entered_day)
        .max()
}

pub fn characteristic_value(game: &GameState, id: &str) -> f64 {
    let role_id = resource_id(game, id);
    game.player
        .characteristics
        .get(role_id)
        .copied()
        .unwrap_or_default()
}

fn resource_id<'a>(game: &'a GameState, id: &'a str) -> &'a str {
    match id {
        "budget" => game.catalog.resource_roles.currency.as_deref(),
        "stamina" => game.catalog.resource_roles.recovery.as_deref(),
        "age" => game.catalog.resource_roles.age.as_deref(),
        _ => None,
    }
    .unwrap_or(id)
}

fn require_resource(
    game: &GameState,
    id: &str,
    amount: f64,
    operation: &str,
) -> Result<(), String> {
    if amount > 0.0
        && !game
            .player
            .characteristics
            .contains_key(resource_id(game, id))
    {
        return Err(format!(
            "Cannot {operation}: dataset resource '{}' is not declared",
            resource_id(game, id)
        ));
    }
    Ok(())
}

fn adjust_characteristic(game: &mut GameState, id: &str, amount: f64) {
    let role_id = resource_id(game, id);
    let definition = game
        .catalog
        .player_characteristics
        .iter()
        .find(|entry| entry.id == role_id);
    if definition.is_none() {
        return;
    }
    let value = (characteristic_value(game, id) + amount).clamp(
        definition
            .map(|entry| entry.min_value)
            .unwrap_or(f64::NEG_INFINITY),
        definition
            .map(|entry| entry.max_value)
            .unwrap_or(f64::INFINITY),
    );
    game.player
        .characteristics
        .insert(role_id.to_string(), value);
}

fn apply_event_effects(game: &mut GameState, effects: &str) -> Result<(), String> {
    let mut staged = game.clone();
    apply_event_effects_in_place(&mut staged, effects)?;
    *game = staged;
    Ok(())
}

fn apply_event_effects_in_place(game: &mut GameState, effects: &str) -> Result<(), String> {
    for raw_effect in effects.split(';') {
        let effect = raw_effect.trim();
        if effect.is_empty() {
            continue;
        }
        let parts = effect.split(':').map(str::trim).collect::<Vec<_>>();
        let is_operation = matches!(
            parts
                .first()
                .map(|value| value.to_ascii_lowercase())
                .as_deref(),
            Some(
                "add_characteristic"
                    | "characteristic_delta"
                    | "set_characteristic"
                    | "grant_object"
                    | "consume_object"
                    | "remove_object"
                    | "set_object_availability"
                    | "set_object_service"
            )
        );
        if parts.len() == 2 && !is_operation {
            let target = non_empty_effect_part(parts[0], effect)?;
            let amount = parse_effect_number(parts[1], effect)?;
            adjust_characteristic(game, target, amount);
            if !game
                .catalog
                .player_characteristics
                .iter()
                .any(|entry| entry.id == target)
                && !matches!(target, "budget" | "stamina" | "age")
            {
                return Err(format!(
                    "Effect '{effect}' targets unknown characteristic '{target}'"
                ));
            }
            continue;
        }
        if parts.len() < 2 || parts.len() > 3 {
            return Err(format!(
                "Effect '{effect}' must be operation:target[:value]"
            ));
        }

        let operation = parts[0].to_ascii_lowercase();
        let target = non_empty_effect_part(parts[1], effect)?;
        if matches!(
            operation.as_str(),
            "add_characteristic"
                | "characteristic_delta"
                | "set_characteristic"
                | "multiply_characteristic"
        ) && !game
            .catalog
            .player_characteristics
            .iter()
            .any(|entry| entry.id == target)
            && !matches!(target, "budget" | "stamina" | "age")
        {
            return Err(format!(
                "Effect '{effect}' targets unknown characteristic '{target}'"
            ));
        }
        match operation.as_str() {
            "add_characteristic" | "characteristic_delta" => {
                let amount = parse_effect_number(
                    parts
                        .get(2)
                        .copied()
                        .ok_or_else(|| format!("Effect '{effect}' is missing an amount"))?,
                    effect,
                )?;
                adjust_characteristic(game, target, amount);
            }
            "set_characteristic" => {
                let value = parse_effect_number(
                    parts
                        .get(2)
                        .copied()
                        .ok_or_else(|| format!("Effect '{effect}' is missing a value"))?,
                    effect,
                )?;
                let role_id = resource_id(game, target);
                if game
                    .catalog
                    .player_characteristics
                    .iter()
                    .any(|entry| entry.id == role_id)
                {
                    engine::effects::validate_characteristic_value(
                        &game.catalog.player_characteristics,
                        role_id,
                        value,
                    )
                    .map_err(|error| format!("Effect '{effect}' is invalid: {error}"))?;
                }
                let current = characteristic_value(game, target);
                adjust_characteristic(game, target, value - current);
            }
            "multiply_characteristic" => {
                let factor = parse_effect_number(
                    parts
                        .get(2)
                        .copied()
                        .ok_or_else(|| format!("Effect '{effect}' is missing a multiplier"))?,
                    effect,
                )?;
                let current = characteristic_value(game, target);
                let result = current * factor;
                let role_id = resource_id(game, target);
                if game
                    .catalog
                    .player_characteristics
                    .iter()
                    .any(|entry| entry.id == role_id)
                {
                    engine::effects::validate_characteristic_value(
                        &game.catalog.player_characteristics,
                        role_id,
                        result,
                    )
                    .map_err(|error| format!("Effect '{effect}' is invalid: {error}"))?;
                }
                adjust_characteristic(game, target, current * (factor - 1.0));
            }
            "grant_object" => {
                let quantity = parse_effect_quantity(parts.get(2).copied(), effect)?;
                let definition = game
                    .catalog
                    .objects
                    .iter()
                    .find(|object| object.id == target)
                    .cloned()
                    .ok_or_else(|| {
                        format!("Effect '{effect}' targets unknown object '{target}'")
                    })?;
                let policy = definition.policy();
                let quantity_u32 = u32::try_from(quantity)
                    .map_err(|_| format!("Effect '{effect}' quantity is too large"))?;
                let owned_count = game
                    .player
                    .inventory
                    .iter()
                    .filter(|owned| {
                        if definition.object_type.eq_ignore_ascii_case("insurance") {
                            owned.object_type.eq_ignore_ascii_case("insurance")
                        } else {
                            owned_matches_definition(owned, target)
                        }
                    })
                    .count() as u32;
                if policy.unique && owned_count.saturating_add(quantity_u32) > 1
                    || policy.max_owned > 0
                        && owned_count.saturating_add(quantity_u32) > policy.max_owned
                {
                    return Err(format!(
                        "Effect '{effect}' exceeds ownership limit for object '{target}'"
                    ));
                }
                for _ in 0..quantity {
                    let owned = build_owned_object(&definition, game, false, 0);
                    game.player.inventory.push(owned);
                }
            }
            "consume_object" | "remove_object" => {
                let quantity = parse_effect_quantity(parts.get(2).copied(), effect)?;
                let matching = game
                    .player
                    .inventory
                    .iter()
                    .enumerate()
                    .filter(|(_, owned)| owned_matches_definition(owned, target))
                    .map(|(index, _)| index)
                    .collect::<Vec<_>>();
                if matching.len() < quantity {
                    return Err(format!(
                        "Effect '{effect}' requires {quantity} owned instance(s) of '{target}', found {}",
                        matching.len()
                    ));
                }
                for index in matching.into_iter().take(quantity).rev() {
                    game.player.inventory.remove(index);
                }
            }
            "set_object_availability" => {
                let days = parse_effect_quantity(parts.get(2).copied(), effect)?;
                let until = game.current_day.saturating_add(days as u32);
                let object = game
                    .player
                    .inventory
                    .iter_mut()
                    .find(|owned| owned_matches_definition(owned, target))
                    .ok_or_else(|| {
                        format!("Effect '{effect}' targets an unowned object '{target}'")
                    })?;
                object.unavailable_until_day = until;
            }
            "set_object_service" => {
                let slot = parts
                    .get(2)
                    .copied()
                    .ok_or_else(|| format!("Effect '{effect}' is missing a service slot"))?
                    .strip_prefix("service_")
                    .and_then(|value| value.strip_suffix("_needed").or(Some(value)))
                    .and_then(|value| value.parse::<usize>().ok())
                    .filter(|slot| (1..=15).contains(slot))
                    .ok_or_else(|| {
                        format!("Effect '{effect}' requires service_1 through service_15")
                    })?;
                let object = game
                    .player
                    .inventory
                    .iter_mut()
                    .find(|owned| owned_matches_definition(owned, target))
                    .ok_or_else(|| {
                        format!("Effect '{effect}' targets an unowned object '{target}'")
                    })?;
                set_service_needed(object, slot, true);
            }
            _ => {
                return Err(format!(
                    "Effect '{effect}' uses unsupported operation '{operation}'"
                ));
            }
        }
    }
    Ok(())
}

fn apply_bound_effects(
    game: &mut GameState,
    trigger_type: &str,
    trigger_ref: &str,
    reported_result: &str,
) -> Result<(), String> {
    let bindings = game.catalog.effect_bindings.clone();
    let effects = game.catalog.effects.clone();
    let mut effect_strings = Vec::new();
    for binding in bindings.into_iter().filter(|binding| {
        normalized(&binding.trigger_type) == normalized(trigger_type)
            && (binding.trigger_ref.trim().is_empty()
                || normalized(&binding.trigger_ref) == normalized(trigger_ref))
            && (binding.reported_result.trim().is_empty()
                || normalized(&binding.reported_result) == normalized(reported_result))
    }) {
        if binding.probability <= 0.0
            || (binding.probability < 1.0 && roll(game) > binding.probability)
        {
            continue;
        }
        let effect = effects
            .iter()
            .find(|effect| effect.id == binding.effect_id)
            .ok_or_else(|| {
                format!(
                    "Effect binding '{}' references unknown effect '{}'",
                    binding.id, binding.effect_id
                )
            })?;
        let operation = normalized(&effect.operation);
        let target = effect.target.trim();
        if target.is_empty() {
            return Err(format!("Effect '{}' has an empty target", effect.id));
        }
        let encoded = match operation.as_str() {
            "add_characteristic" | "set_characteristic" | "multiply_characteristic" => {
                let value = effect.value.trim();
                if value.is_empty() {
                    return Err(format!("Effect '{}' is missing a value", effect.id));
                }
                let expression =
                    engine::expressions::Expression::parse(value).map_err(|error| {
                        format!(
                            "Effect '{}' has invalid value expression: {error}",
                            effect.id
                        )
                    })?;
                let evaluated =
                    expression
                        .evaluate(&numeric_modifier_facts(game))
                        .map_err(|error| {
                            format!("Effect '{}' could not be evaluated: {error}", effect.id)
                        })?;
                if !evaluated.is_finite() {
                    return Err(format!(
                        "Effect '{}' evaluated to a non-finite value",
                        effect.id
                    ));
                }
                format!("{}:{}:{}", operation, target, evaluated)
            }
            "grant_object" | "remove_object" | "consume_object" => {
                let quantity = effective_effect_quantity(game, effect, 1.0, false)?;
                format!("{}:{}:{}", operation, target, quantity)
            }
            "set_object_availability" => {
                let quantity = effective_effect_quantity(game, effect, 0.0, true)?;
                format!("{}:{}:{}", operation, target, quantity)
            }
            "set_object_service" => {
                format!("{}:{}:{}", operation, target, effect.value.trim())
            }
            _ => {
                return Err(format!(
                    "Effect '{}' uses unsupported operation '{}'",
                    effect.id, effect.operation
                ))
            }
        };
        effect_strings.push(encoded);
    }
    if effect_strings.is_empty() {
        return Ok(());
    }
    apply_event_effects(game, &effect_strings.join(";"))
}

fn effective_effect_quantity(
    game: &GameState,
    effect: &EffectData,
    default: f64,
    allow_zero: bool,
) -> Result<String, String> {
    let base = if effect.quantity.trim().is_empty() {
        default
    } else {
        parse_effect_number(effect.quantity.trim(), &effect.id)?
    };
    let modified =
        apply_numeric_modifier_target(game, &format!("effect_quantity:{}", effect.id), base)?;
    if modified < 0.0 || (!allow_zero && modified < 1.0) || modified.fract() != 0.0 {
        return Err(format!(
            "Effect '{}' quantity must be a {} integral value, got {}",
            effect.id,
            if allow_zero {
                "non-negative"
            } else {
                "positive"
            },
            modified
        ));
    }
    let quantity = usize::try_from(modified as u128)
        .map_err(|_| format!("Effect '{}' quantity is too large", effect.id))?;
    Ok(quantity.to_string())
}

fn non_empty_effect_part<'a>(value: &'a str, effect: &str) -> Result<&'a str, String> {
    if value.is_empty() {
        Err(format!("Effect '{effect}' has an empty target"))
    } else {
        Ok(value)
    }
}

fn parse_effect_number(value: &str, effect: &str) -> Result<f64, String> {
    let number = value
        .parse::<f64>()
        .map_err(|_| format!("Effect '{effect}' has invalid numeric value '{value}'"))?;
    if !number.is_finite() {
        return Err(format!("Effect '{effect}' must use a finite numeric value"));
    }
    Ok(number)
}

fn parse_effect_quantity(value: Option<&str>, effect: &str) -> Result<usize, String> {
    let value = value.unwrap_or("1");
    let number = parse_effect_number(value, effect)?;
    if number < 1.0 || number.fract() != 0.0 {
        return Err(format!(
            "Effect '{effect}' requires a positive integral quantity"
        ));
    }
    usize::try_from(number as u128).map_err(|_| format!("Effect '{effect}' quantity is too large"))
}

fn set_service_needed(object: &mut OwnedObject, slot: usize, needed: bool) {
    match slot {
        1 => object.service_1_needed = needed,
        2 => object.service_2_needed = needed,
        3 => object.service_3_needed = needed,
        4 => object.service_4_needed = needed,
        5 => object.service_5_needed = needed,
        6 => object.service_6_needed = needed,
        7 => object.service_7_needed = needed,
        8 => object.service_8_needed = needed,
        9 => object.service_9_needed = needed,
        10 => object.service_10_needed = needed,
        11 => object.service_11_needed = needed,
        12 => object.service_12_needed = needed,
        13 => object.service_13_needed = needed,
        14 => object.service_14_needed = needed,
        15 => object.service_15_needed = needed,
        _ => unreachable!("service slots are validated before mutation"),
    }
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

fn random_text(game: &mut GameState, variable: &str) -> Option<String> {
    let variants = game
        .catalog
        .texts
        .iter()
        .find(|entry| entry.variable == variable)?
        .values
        .clone();
    if variants.is_empty() {
        return None;
    }
    let index = (roll(game) * variants.len() as f64).floor() as usize;
    variants.get(index.min(variants.len() - 1)).cloned()
}

fn resolve_text_variables(game: &mut GameState, template: impl Into<String>) -> String {
    let mut result = template.into();
    let mut search_from = 0;
    while let Some(relative_start) = result[search_from..].find("{text:") {
        let start = search_from + relative_start;
        let Some(relative_end) = result[start..].find('}') else {
            break;
        };
        let end = start + relative_end;
        let variable = &result[start + 6..end];
        let Some(replacement) = random_text(game, variable.trim()) else {
            search_from = end + 1;
            continue;
        };
        result.replace_range(start..=end, &replacement);
        search_from = start + replacement.len();
    }
    result
}

fn push_popup_alert(
    game: &mut GameState,
    category: &str,
    id: impl Into<String>,
    title: impl Into<String>,
    message: impl Into<String>,
) {
    if popup_category_enabled(game, category) {
        let title = resolve_text_variables(game, title);
        let message = resolve_text_variables(game, message);
        game.pending_alerts.push(GameAlert {
            id: id.into(),
            title,
            message,
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
        .find(|characteristic| {
            catalog
                .resource_roles
                .age
                .as_deref()
                .is_some_and(|id| id.eq_ignore_ascii_case(&characteristic.id))
                || (catalog.resource_roles.age.is_none()
                    && characteristic.id.eq_ignore_ascii_case("age"))
        })
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
            owned_matches_definition(owned, id)
                || required_group.is_some_and(|group| {
                    game.catalog
                        .objects
                        .iter()
                        .find(|candidate| owned_matches_definition(owned, &candidate.id))
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
    service_slot: Option<usize>,
) -> Result<(), String> {
    let object = game
        .player
        .inventory
        .iter_mut()
        .find(|object| object.id == object_id)
        .ok_or_else(|| format!("Object '{}' not found for service cost", object_id))?;
    if let Some(slot) = service_slot {
        let (configured_cost, needed) = match slot {
            1 => (&object.cost_1, &mut object.service_1_needed),
            2 => (&object.cost_2, &mut object.service_2_needed),
            3 => (&object.cost_3, &mut object.service_3_needed),
            4 => (&object.cost_4, &mut object.service_4_needed),
            5 => (&object.cost_5, &mut object.service_5_needed),
            6 => (&object.cost_6, &mut object.service_6_needed),
            7 => (&object.cost_7, &mut object.service_7_needed),
            8 => (&object.cost_8, &mut object.service_8_needed),
            9 => (&object.cost_9, &mut object.service_9_needed),
            10 => (&object.cost_10, &mut object.service_10_needed),
            11 => (&object.cost_11, &mut object.service_11_needed),
            12 => (&object.cost_12, &mut object.service_12_needed),
            13 => (&object.cost_13, &mut object.service_13_needed),
            14 => (&object.cost_14, &mut object.service_14_needed),
            15 => (&object.cost_15, &mut object.service_15_needed),
            _ => unreachable!("service slots are validated during dataset loading"),
        };
        if configured_cost != cost_id {
            return Err(format!(
                "Cost rule for '{}' targets service slot {}, configured cost is '{}'",
                cost_id, slot, configured_cost
            ));
        }
        *needed = true;
        return Ok(());
    }

    let mut slots = [
        (&mut object.cost_1, &mut object.service_1_needed),
        (&mut object.cost_2, &mut object.service_2_needed),
        (&mut object.cost_3, &mut object.service_3_needed),
        (&mut object.cost_4, &mut object.service_4_needed),
        (&mut object.cost_5, &mut object.service_5_needed),
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
    ];
    if let Some((_, needed)) = slots.iter_mut().find(|(slot, _)| **slot == cost_id) {
        **needed = true;
    } else if let Some((slot, needed)) = slots.iter_mut().find(|(slot, _)| slot.is_empty()) {
        **slot = cost_id.to_string();
        **needed = true;
    } else {
        return Err(format!(
            "No free service slot is available for cost '{}'",
            cost_id
        ));
    }
    Ok(())
}

fn cost_is_cosmetic(game: &GameState, cost_id: &str) -> bool {
    game.catalog
        .costs
        .iter()
        .find(|cost| cost.id == cost_id)
        .is_some_and(|cost| cost.cosmetic)
}

fn event_is_motorsport(event: &EventData) -> bool {
    event
        .tags
        .split(';')
        .any(|tag| matches!(normalized(tag).as_str(), "race" | "track_day" | "trackday"))
}

fn event_allows_any_vehicle(event: &EventData) -> bool {
    let is_track_day = event
        .tags
        .split(';')
        .any(|tag| matches!(normalized(tag).as_str(), "track_day" | "trackday"));
    let is_open_race = normalized(&event.name).contains("open race");
    is_track_day || is_open_race
}

fn owns_object_id(game: &GameState, required_id: &str) -> bool {
    game.player
        .inventory
        .iter()
        .any(|owned| owned_matches_definition(owned, required_id))
}

fn owned_matches_definition(object: &OwnedObject, definition_id: &str) -> bool {
    if !object.definition_id.trim().is_empty() {
        return object.definition_id == definition_id;
    }
    object.id == definition_id
}

fn next_instance_id(game: &mut GameState, definition_id: &str) -> String {
    loop {
        let candidate = format!("{definition_id}_{}", game.next_object_instance_id);
        game.next_object_instance_id = game.next_object_instance_id.saturating_add(1);
        if !game.player.inventory.iter().any(|object| {
            object.id == candidate
                || (!object.instance_id.is_empty() && object.instance_id == candidate)
        }) {
            return candidate;
        }
    }
}

fn race_gear_ready(game: &GameState) -> bool {
    let groups = label(
        &game.catalog,
        "dashboard_readiness_object_groups",
        "helmet|helm;tracksuit|sponsored_track_suit;gloves|sponsored_gloves;shoes|sponsored_shoes",
    );
    groups
        .split(';')
        .map(|group| group.split('|').map(str::trim).filter(|id| !id.is_empty()))
        .filter(|group| group.clone().next().is_some())
        .all(|group| group.into_iter().any(|id| owns_object_id(game, id)))
}

fn rental_object_for(game: &GameState, object_id: &str) -> Option<engine::loader::ObjectData> {
    game.catalog
        .objects
        .iter()
        .find(|object| {
            object.id == object_id
                && (object.transfer_policy() == TransferPolicy::Rental
                    || (!object.has_explicit_transfer_policy()
                        && object.object_type.eq_ignore_ascii_case("vehicle")))
        })
        .cloned()
}

fn rental_expiry_day(game: &GameState, object_id: &str, entered_day: u32) -> u32 {
    rental_object_for(game, object_id)
        .map(|object| object.rental_duration_days.saturating_add(entered_day))
        .unwrap_or(0)
}

fn rental_event_error(game: &GameState, event: &EventData, object_id: &str) -> Result<f64, String> {
    validate_requirement_binding(game, "rent", &event.id)?;
    let car = rental_object_for(game, object_id)
        .ok_or_else(|| "The selected object is not configured for rental".to_string())?;
    let explicit_rental = car.transfer_policy() == TransferPolicy::Rental;
    if !explicit_rental && !event_allows_any_vehicle(event) {
        return Err("Only open races and track days offer car rental".into());
    }
    let day_of_year = ((game.current_day - 1) % game.days_per_year) + 1;
    if event.day_of_year != day_of_year {
        return Err(format!(
            "Event is scheduled for day {}, today is day {}.",
            event.day_of_year, day_of_year
        ));
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
    if !event.quest_id.trim().is_empty()
        && !game
            .quest_memberships
            .iter()
            .any(|membership| membership.quest_id == event.quest_id)
    {
        return Err("Join the event's championship before renting a car for it".into());
    }
    if !event.required_license_id.trim().is_empty()
        && !owns_object_id(game, event.required_license_id.trim())
    {
        return Err(format!(
            "Cannot enter '{}': required licence '{}' is missing.",
            event.name, event.required_license_id
        ));
    }
    if !explicit_rental && !race_gear_ready(game) {
        return Err("All required race gear must be owned before renting a car".into());
    }
    let rental_cost = if car.rental_cost > 0.0 {
        car.rental_cost
    } else {
        car.price / 25.0
    };
    let race_event = event.tags.split(';').any(|tag| normalized(tag) == "race");
    let event_stamina_cost = if race_event {
        config_f64(&game.catalog, "race_day_stamina_cost", 20.0)
            * event_duration_days(event).max(1) as f64
    } else {
        event.stamina_cost.max(0.0)
    };
    let entry_fee = if sponsor_entry_fee_covered(game, event) {
        0.0
    } else {
        event.entry_fee
    };
    require_resource(game, "budget", entry_fee + rental_cost, "rent a car")?;
    if event_stamina_cost > 0.0 {
        require_resource(game, "stamina", event_stamina_cost, "enter this event")?;
    }
    if characteristic_value(game, "budget") < entry_fee + rental_cost {
        return Err("Insufficient funds for event entry and car rental".into());
    }
    if event_stamina_cost > 0.0 && characteristic_value(game, "stamina") < event_stamina_cost {
        return Err(format!(
            "Not enough stamina to enter '{}': {} required.",
            event.name, event_stamina_cost
        ));
    }
    Ok(rental_cost)
}

fn object_has_needed_service(object: &OwnedObject, cost_id: &str) -> bool {
    [
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
    ]
    .iter()
    .any(|(needed, configured_cost)| *needed && *configured_cost == cost_id)
}

fn cosmetic_cred_penalty(game: &GameState, object: &OwnedObject, event: &EventData) -> f64 {
    if !event_is_motorsport(event) {
        return 0.0;
    }
    let has_cosmetic_damage = [
        object.cost_1.as_str(),
        object.cost_2.as_str(),
        object.cost_3.as_str(),
        object.cost_4.as_str(),
        object.cost_5.as_str(),
        object.cost_6.as_str(),
        object.cost_7.as_str(),
        object.cost_8.as_str(),
        object.cost_9.as_str(),
        object.cost_10.as_str(),
        object.cost_11.as_str(),
        object.cost_12.as_str(),
        object.cost_13.as_str(),
        object.cost_14.as_str(),
        object.cost_15.as_str(),
    ]
    .iter()
    .any(|cost_id| cost_is_cosmetic(game, cost_id) && object_has_needed_service(object, cost_id));
    if has_cosmetic_damage {
        config_f64(&game.catalog, "cosmetic_race_cred_penalty", 2.0)
    } else {
        0.0
    }
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
        if needed && !cost_is_cosmetic(game, cost_id) {
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

fn numeric_modifier_facts(game: &GameState) -> HashMap<String, f64> {
    let mut facts = HashMap::new();
    for characteristic in &game.catalog.player_characteristics {
        let value = game
            .player
            .characteristics
            .get(&characteristic.id)
            .copied()
            .unwrap_or(characteristic.value);
        facts.insert(characteristic.id.clone(), value);
        facts.insert(format!("characteristic({})", characteristic.id), value);
    }
    for object in &game.catalog.objects {
        let definition_count = game
            .player
            .inventory
            .iter()
            .filter(|owned| owned_matches_definition(owned, &object.id))
            .count() as f64;
        facts.insert(
            format!("inventory_count(definition,{})", object.id),
            definition_count,
        );
        facts.insert(
            format!("inventory_count(type,{})", object.object_type),
            game.player
                .inventory
                .iter()
                .filter(|owned| {
                    game.catalog
                        .objects
                        .iter()
                        .find(|definition| owned_matches_definition(owned, &definition.id))
                        .is_some_and(|definition| definition.object_type == object.object_type)
                })
                .count() as f64,
        );
    }
    facts.insert("calendar_day".into(), game.current_day as f64);
    facts.insert("age_days".into(), game.player.age_days as f64);
    facts
}

pub fn numeric_modifier_breakdown_for_sim(
    game: &GameState,
    target: &str,
    base: f64,
) -> Result<engine::modifiers::NumericModifierResult, String> {
    let modifiers = game
        .catalog
        .numeric_modifiers
        .iter()
        .filter(|modifier| normalized(&modifier.target) == normalized(target))
        .cloned()
        .collect::<Vec<_>>();
    if modifiers.is_empty() {
        return Ok(engine::modifiers::NumericModifierResult {
            base,
            contributions: Vec::new(),
            final_value: base,
        });
    }
    let conditions = if modifiers
        .iter()
        .any(|modifier| !modifier.condition_group.trim().is_empty())
    {
        Some(
            engine::conditions::ConditionSet::from_rows(
                &game.catalog.condition_groups,
                &game.catalog.conditions,
            )
            .map_err(|errors| format!("Invalid numeric modifier conditions: {errors:?}"))?,
        )
    } else {
        None
    };
    let facts = numeric_modifier_facts(game);
    let result = engine::modifiers::apply_numeric_modifiers(base, &modifiers, &facts, |group| {
        conditions
            .as_ref()
            .ok_or_else(|| format!("Modifier condition group '{group}' is unavailable"))?
            .evaluate(game, group)
            .map(|explanation| explanation.passed)
            .map_err(|error| format!("Cannot evaluate modifier condition '{group}': {error:?}"))
    })?;
    Ok(if normalized(target).contains("probability") {
        engine::modifiers::NumericModifierResult {
            final_value: result.final_value.clamp(0.0, 1.0),
            ..result
        }
    } else {
        result
    })
}

fn apply_numeric_modifier_target(game: &GameState, target: &str, base: f64) -> Result<f64, String> {
    Ok(numeric_modifier_breakdown_for_sim(game, target, base)?.final_value)
}

fn validate_requirement_binding(
    game: &GameState,
    operation: &str,
    target: &str,
) -> Result<(), String> {
    let bindings = game
        .catalog
        .requirement_bindings
        .iter()
        .filter(|binding| {
            normalized(&binding.operation) == normalized(operation)
                && binding.target_ref.trim() == target
        })
        .collect::<Vec<_>>();
    if bindings.is_empty() {
        return Ok(());
    }
    let conditions = engine::conditions::ConditionSet::from_rows(
        &game.catalog.condition_groups,
        &game.catalog.conditions,
    )
    .map_err(|errors| format!("Invalid requirement bindings: {errors:?}"))?;
    for binding in bindings {
        let explanation = conditions
            .evaluate(game, binding.requirement_group.trim())
            .map_err(|error| {
                format!(
                    "Cannot evaluate requirement '{}' for {} '{}': {error:?}",
                    binding.requirement_group, operation, target
                )
            })?;
        if !explanation.passed {
            return Err(format!(
                "Requirement '{}' is not satisfied for {} '{}'",
                binding.requirement_group, operation, target
            ));
        }
    }
    Ok(())
}

fn reported_result_matches(expected: &str, actual: &str) -> bool {
    fn canonical(value: &str) -> String {
        match normalized(value).as_str() {
            "success" | "successful" | "win" | "won" | "1" | "yes" | "true" => "success".into(),
            "failure" | "failed" | "unsuccessful" | "lose" | "lost" | "0" | "no" | "false" => {
                "failure".into()
            }
            value => value.to_string(),
        }
    }

    canonical(expected) == canonical(actual)
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
    let automatic_damage_choice = if automatic_damage {
        let candidates: Vec<&engine::loader::CostRule> = rules
            .iter()
            .filter(|rule| {
                normalized(&rule.trigger_type) == "event_completed"
                    && !rule.damage_type.trim().is_empty()
                    && (rule.event_interval == 0
                        || selected_event_count(game, &rule.trigger_ref).saturating_add(1)
                            % rule.event_interval
                            == 0)
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
                let days_without_event = selected_event_last_day(game, &rule.trigger_ref)
                    .or_else(|| {
                        if normalized(&rule.trigger_ref) == "race" {
                            game.last_race_day
                        } else {
                            None
                        }
                    })
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
                && (rule.event_interval == 0
                    || selected_event_count(game, &rule.trigger_ref).saturating_add(1)
                        % rule.event_interval
                        != 0)
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
                mark_object_service_needed(
                    game,
                    &rule_context.source_id,
                    &rule.cost_id,
                    rule.service_slot,
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
            let charged =
                !object_service && immediate && characteristic_value(game, "budget") >= amount;
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
    // The startup screen does not need a catalog. Loading it here caused the
    // default dataset to be parsed again when the user started a new game.
    let catalog = GameCatalog::default();
    let pending_alerts = Vec::new();
    GameState {
        save_version: engine::save::CURRENT_SAVE_VERSION,
        schema_version: engine::save::CURRENT_SCHEMA_VERSION,
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
        quest_runs: vec![],
        reward_receipts: vec![],
        championship_results: vec![],
        event_log: vec![],
        last_race_day: None,
        active_encounter: None,
        last_encounter_result: None,
        pending_sponsor_event_id: None,
        sponsor_contracts: vec![],
        manager_sponsor_offer_ids: vec![],
        manager_sponsor_offer_cooldown_until_day: 0,
        rng_state: rand::rng().random(),
        alarm_event_ids: vec![],
        popup_categories: vec![
            "Income".into(),
            "Costs applied".into(),
            "Event incoming".into(),
            "My Alarms".into(),
        ],
        next_object_instance_id: 1,
    }
}

/// Construct a UI-independent game.  The Tauri frontend uses the same state
/// shape, which also makes this useful to simulations and external tools.
pub fn new_game(dataset_path: impl Into<String>) -> GameState {
    new_game_seeded(dataset_path, rand::rng().random())
}

pub fn new_game_seeded(dataset_path: impl Into<String>, seed: u64) -> GameState {
    let dataset_path = dataset_path.into();
    let catalog = GameCatalog::load_from_directory_cached(&dataset_path);
    let pending_alerts = dataset_warning_alerts(&catalog);
    GameState {
        save_version: engine::save::CURRENT_SAVE_VERSION,
        schema_version: engine::save::CURRENT_SCHEMA_VERSION,
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
        quest_runs: vec![],
        reward_receipts: vec![],
        championship_results: vec![],
        event_log: vec![],
        last_race_day: None,
        active_encounter: None,
        last_encounter_result: None,
        pending_sponsor_event_id: None,
        sponsor_contracts: vec![],
        manager_sponsor_offer_ids: vec![],
        manager_sponsor_offer_cooldown_until_day: 0,
        rng_state: if seed == 0 { 1 } else { seed },
        alarm_event_ids: vec![],
        popup_categories: vec![
            "Income".into(),
            "Costs applied".into(),
            "Event incoming".into(),
            "My Alarms".into(),
        ],
        next_object_instance_id: 1,
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

fn catalog_reload_diagnostics(game: &GameState) -> Vec<String> {
    let object_ids = game
        .catalog
        .objects
        .iter()
        .map(|object| object.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let event_ids = game
        .catalog
        .events
        .iter()
        .map(|event| event.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let quest_ids = game
        .catalog
        .quests
        .iter()
        .map(|quest| quest.id.as_str())
        .collect::<std::collections::HashSet<_>>();
    let mut diagnostics = Vec::new();
    for object in &game.player.inventory {
        if !object_ids.contains(object.definition_id.as_str()) {
            diagnostics.push(format!(
                "Inventory object '{}' references missing definition '{}'; identity was preserved.",
                object.id, object.definition_id
            ));
        }
    }
    for event_id in game
        .player
        .active_events
        .iter()
        .map(|event| event.event_id.as_str())
        .chain(
            game.pending_events
                .iter()
                .map(|event| event.event_id.as_str()),
        )
        .chain(
            game.event_history
                .iter()
                .map(|event| event.event_id.as_str()),
        )
    {
        if !event_ids.contains(event_id) {
            diagnostics.push(format!(
                "Saved event reference '{}' is missing from the reloaded catalog.",
                event_id
            ));
        }
    }
    for membership in &game.quest_memberships {
        if !quest_ids.contains(membership.quest_id.as_str()) {
            diagnostics.push(format!(
                "Saved quest reference '{}' is missing from the reloaded catalog.",
                membership.quest_id
            ));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub fn legal_event_ids(game: &GameState) -> Vec<String> {
    if game.active_encounter.is_some() {
        return Vec::new();
    }
    game.catalog
        .events
        .iter()
        .filter(|action| action.day_of_year == 0)
        .filter(|action| validate_player_started_event(game, action).is_ok())
        .map(|action| action.id.clone())
        .collect()
}

fn validate_player_started_event(game: &GameState, event: &EventData) -> Result<(), String> {
    if event.day_of_year != 0 {
        return Err("This event is scheduled and cannot be started directly.".into());
    }
    validate_requirement_binding(game, "event_start", &event.id)?;
    require_resource(game, "budget", event.base_cost, "start this action")?;
    require_resource(
        game,
        "stamina",
        event_upfront_stamina_cost(game, event),
        "start this action",
    )?;
    if characteristic_value(game, "budget") < event.base_cost {
        return Err("Insufficient funds to start action".into());
    }
    if characteristic_value(game, "stamina") < event_upfront_stamina_cost(game, event) {
        return Err("Not enough stamina to start action".into());
    }
    if event.payout_freq_type.eq_ignore_ascii_case("recurring")
        && game
            .player
            .active_events
            .iter()
            .any(|active| active.event_id == event.id)
    {
        return Err("This action is already active".into());
    }
    if let Some(obligations) = obligations_for_event(game, &event.id) {
        for obligation in obligations {
            if !obligation.required_event_type.trim().is_empty()
                && !required_event_type_is_active(game, &obligation.required_event_type)
            {
                return Err(format!(
                    "This action requires an active {}.",
                    obligation.required_event_type
                ));
            }
            if obligation.max_active > 0
                && active_obligation_count(game, &event.id) >= obligation.max_active as usize
            {
                return Err(format!(
                    "You cannot have more than {} active instances of this action.",
                    obligation.max_active
                ));
            }
        }
    }
    if job_start_blocked(game, event) {
        return Err("This job cannot be started alongside your current jobs.".into());
    }
    let needs_encounter = event.resolution_method.eq_ignore_ascii_case("encounter")
        || !event.encounter_id.trim().is_empty();
    if needs_encounter
        && (event.encounter_id.trim().is_empty()
            || !game
                .catalog
                .encounter_configs
                .iter()
                .any(|config| config.encounter_id == event.encounter_id))
    {
        return Err(format!(
            "Action '{}' references an unknown encounter '{}'",
            event.name, event.encounter_id
        ));
    }
    Ok(())
}

fn validate_event_entry(
    game: &GameState,
    event: &EventData,
    object_id: &str,
) -> Result<f64, String> {
    validate_requirement_binding(game, "event_entry", &event.id)?;
    let day_of_year = ((game.current_day - 1) % game.days_per_year) + 1;
    if event.day_of_year != day_of_year {
        return Err(format!(
            "Event is scheduled for day {}, today is day {}.",
            event.day_of_year, day_of_year
        ));
    }
    if !event.quest_id.trim().is_empty()
        && !game
            .quest_memberships
            .iter()
            .any(|membership| membership.quest_id == event.quest_id)
    {
        return Err("Join the event's quest before entering it".into());
    }
    if event_is_motorsport(event)
        && event.tags.split(';').any(|tag| normalized(tag) == "race")
        && !game
            .player
            .inventory
            .iter()
            .any(|object| object.object_type.eq_ignore_ascii_case("insurance"))
    {
        return Err("Cannot enter a race without active insurance.".into());
    }
    if !event.requirement_group.trim().is_empty() && !game.catalog.condition_groups.is_empty() {
        let conditions = engine::conditions::ConditionSet::from_rows(
            &game.catalog.condition_groups,
            &game.catalog.conditions,
        )
        .map_err(|errors| format!("Invalid event requirements: {errors:?}"))?;
        let explanation = conditions
            .evaluate(game, event.requirement_group.trim())
            .map_err(|error| format!("Cannot evaluate event requirements: {error:?}"))?;
        if !explanation.passed {
            return Err(format!(
                "Event requirements are not satisfied: {}",
                explanation_summary(&explanation)
            ));
        }
    }
    let entry_fee = if sponsor_entry_fee_covered(game, event) {
        0.0
    } else {
        event.entry_fee
    };
    require_resource(game, "budget", entry_fee, "enter this event")?;
    if characteristic_value(game, "budget") < entry_fee {
        return Err("Insufficient funds for event entry".into());
    }

    fn explanation_summary(node: &engine::conditions::ExplanationNode) -> String {
        if node.children.is_empty() {
            return node.id.clone();
        }
        let children = node
            .children
            .iter()
            .map(explanation_summary)
            .collect::<Vec<_>>()
            .join(", ");
        format!("{} [{}]", node.kind, children)
    }
    let event_stamina_cost = if event_is_motorsport(event)
        && event.tags.split(';').any(|tag| normalized(tag) == "race")
    {
        config_f64(&game.catalog, "race_day_stamina_cost", 20.0)
            * event_duration_days(event).max(1) as f64
    } else {
        event.stamina_cost.max(0.0)
    };
    require_resource(game, "stamina", event_stamina_cost, "enter this event")?;
    if characteristic_value(game, "stamina") < event_stamina_cost {
        return Err(format!(
            "Not enough stamina to enter '{}': {} required.",
            event.name, event_stamina_cost
        ));
    }
    if !event.required_license_id.trim().is_empty()
        && !owns_object_id(game, event.required_license_id.trim())
    {
        return Err(format!(
            "Cannot enter '{}': required licence '{}' is missing.",
            event.name, event.required_license_id
        ));
    }
    let object = if object_id.trim().is_empty() {
        if event_is_motorsport(event) || !event.required_object_ids.trim().is_empty() {
            let required_objects = event
                .required_object_ids
                .split(';')
                .map(str::trim)
                .filter(|required_id| !required_id.is_empty())
                .map(|required_id| {
                    game.catalog
                        .objects
                        .iter()
                        .find(|definition| definition.id == required_id)
                        .map(|definition| definition.name.clone())
                        .unwrap_or_else(|| required_id.to_string())
                })
                .collect::<Vec<_>>();
            let requirement_detail = if required_objects.is_empty() {
                "a vehicle owned by the player is required".to_string()
            } else {
                format!(
                    "an eligible vehicle is required (allowed: {})",
                    required_objects.join(", ")
                )
            };
            return Err(format!(
                "Cannot enter '{}': {}.",
                event.name, requirement_detail
            ));
        }
        None
    } else {
        Some(
            game.player
                .inventory
                .iter()
                .find(|object| object.id == object_id)
                .ok_or_else(|| "Object not found in inventory".to_string())?,
        )
    };
    if event_is_motorsport(event) && object.is_some_and(|object| object.object_type != "vehicle") {
        return Err(format!(
            "Cannot enter '{}': a vehicle owned by the player is required.",
            event.name
        ));
    }
    if !event_allows_any_vehicle(event)
        && object.is_some_and(|object| {
            player_object_does_not_match_requirement(game, object, &event.required_object_ids)
        })
    {
        return Err(format!(
            "Cannot enter '{}': an eligible {} is required (allowed: {}).",
            event.name,
            label(&game.catalog, "object_name", "object").to_lowercase(),
            event.required_object_ids
        ));
    }
    if let Some(requirements) = object.and_then(|object| object_requirement_error(game, object)) {
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
    Ok(object
        .map(|object| cosmetic_cred_penalty(game, object, event))
        .unwrap_or(0.0))
}

pub fn eligible_event_entries(game: &GameState) -> Vec<(String, String)> {
    game.catalog
        .events
        .iter()
        .flat_map(|event| {
            let inventory_free = validate_event_entry(game, event, "")
                .ok()
                .map(|_| (event.id.clone(), String::new()));
            inventory_free
                .into_iter()
                .chain(game.player.inventory.iter().filter_map(move |object| {
                    validate_event_entry(game, event, &object.id)
                        .ok()
                        .map(|_| (event.id.clone(), object.id.clone()))
                }))
        })
        .collect()
}

pub fn object_transaction_eligibility(game: &GameState) -> Vec<ObjectTransactionEligibility> {
    game.catalog
        .objects
        .iter()
        .map(|definition| {
            let policy = definition.policy();
            let owned_count = game
                .player
                .inventory
                .iter()
                .filter(|owned| owned_matches_definition(owned, &definition.id))
                .count() as u32;
            let mut acquire_state = game.clone();
            let acquire_result = buy_object_in_place(&mut acquire_state, &definition.id);
            let (can_acquire, acquire_reason) = match acquire_result {
                Ok(()) => (true, String::new()),
                Err(error) => (false, error),
            };
            let owned = game
                .player
                .inventory
                .iter()
                .find(|owned| owned_matches_definition(owned, &definition.id));
            let (can_sell, sell_reason) = if let Some(owned) = owned {
                let mut sell_state = game.clone();
                match sell_object_in_place(&mut sell_state, &owned.id) {
                    Ok(()) => (true, String::new()),
                    Err(error) => (false, error),
                }
            } else {
                (false, "No owned instance is available to sell".into())
            };
            ObjectTransactionEligibility {
                object_id: definition.id.clone(),
                owned_count,
                can_acquire,
                acquire_reason,
                can_sell,
                sell_reason,
                buyable: policy.buyable,
                sellable: policy.sellable
                    && !definition.object_type.eq_ignore_ascii_case("insurance"),
                reward_only: policy.reward_only,
                max_owned: policy.max_owned,
                transfer_policy: definition.transfer_policy.clone(),
            }
        })
        .collect()
}

pub fn event_entry_eligibility(game: &GameState, event_id: &str) -> Vec<EventEligibility> {
    let Some(event) = game
        .catalog
        .events
        .iter()
        .find(|event| event.id == event_id)
    else {
        return vec![EventEligibility {
            event_id: event_id.to_string(),
            selection_id: String::new(),
            definition_id: String::new(),
            available: false,
            rented: false,
            reason: "Event not found in catalog".into(),
            reason_code: "event_not_found".into(),
            failed_facts: vec!["event not found".into()],
            entry_fee: 0.0,
            stamina_cost: 0.0,
            rental_cost: 0.0,
        }];
    };
    let stamina_cost = if event_is_motorsport(event)
        && event.tags.split(';').any(|tag| normalized(tag) == "race")
    {
        config_f64(&game.catalog, "race_day_stamina_cost", 20.0)
            * event_duration_days(event).max(1) as f64
    } else {
        event.stamina_cost.max(0.0)
    };
    let mut options = Vec::new();
    let mut add_option = |selection_id: String,
                          definition_id: String,
                          rented: bool,
                          result: Result<f64, String>,
                          rental_cost: f64| {
        let (available, reason, reason_code, failed_facts) = match result {
            Ok(_) => (true, String::new(), String::new(), Vec::new()),
            Err(error) => {
                let reason_code = eligibility_reason_code(&error);
                let failed_facts = vec![error.clone()];
                (false, error, reason_code, failed_facts)
            }
        };
        options.push(EventEligibility {
            event_id: event.id.clone(),
            selection_id,
            definition_id,
            available,
            rented,
            reason,
            reason_code,
            failed_facts,
            entry_fee: event.entry_fee,
            stamina_cost,
            rental_cost,
        });
    };

    add_option(
        String::new(),
        String::new(),
        false,
        validate_event_entry(game, event, ""),
        0.0,
    );
    for object in &game.player.inventory {
        let definition_id = if object.definition_id.is_empty() {
            object.id.clone()
        } else {
            object.definition_id.clone()
        };
        add_option(
            object.id.clone(),
            definition_id,
            false,
            validate_event_entry(game, event, &object.id),
            0.0,
        );
    }

    fn eligibility_reason_code(reason: &str) -> String {
        let normalized_reason = normalized(reason);
        [
            ("event_not_found", "event not found"),
            ("scheduled", "scheduled"),
            ("duplicate_entry", "already been entered"),
            ("membership_required", "join the event"),
            ("missing_license", "licence"),
            ("missing_object", "eligible object"),
            ("insufficient_resource", "insufficient"),
            ("missing_resource", "dataset resource"),
            ("unavailable_object", "unavailable"),
            ("service_required", "service"),
        ]
        .iter()
        .find(|(_, marker)| normalized_reason.contains(marker))
        .map(|(code, _)| (*code).to_string())
        .unwrap_or_else(|| "ineligible".into())
    }
    for object in &game.catalog.objects {
        if rental_object_for(game, &object.id).is_some() {
            let rental_cost = if object.rental_cost > 0.0 {
                object.rental_cost
            } else {
                object.price / 25.0
            };
            add_option(
                object.id.clone(),
                object.id.clone(),
                true,
                rental_event_error(game, event, &object.id),
                rental_cost,
            );
        }
    }
    options
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
            owned_matches_definition(object, id)
                || game
                    .catalog
                    .objects
                    .iter()
                    .find(|candidate| candidate.id == id)
                    .and_then(|required| {
                        let owned_definition =
                            game.catalog.objects.iter().find(|candidate| {
                                owned_matches_definition(object, &candidate.id)
                            })?;
                        (!required.requirement_group.trim().is_empty()
                            && required.requirement_group == owned_definition.requirement_group)
                            .then_some(true)
                    })
                    .unwrap_or(false)
        })
}

pub fn apply_event(game: &mut GameState, event_id: &str) -> Result<EventStartResult, String> {
    perform_event_for_sim(game, event_id)
}

pub fn advance_day(game: &mut GameState) -> Result<(), String> {
    let mut staged = game.clone();
    advance_one_day(&mut staged)?;
    refresh_manager_sponsor_offers(&mut staged);
    *game = staged;
    Ok(())
}

pub fn enter_event_for_sim(
    game: &mut GameState,
    event_id: &str,
    object_id: &str,
) -> Result<(), String> {
    let mut staged = game.clone();
    enter_event_in_place(&mut staged, event_id, object_id)?;
    *game = staged;
    Ok(())
}

fn enter_event_in_place(
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
    let cosmetic_penalty = validate_event_entry(game, &event, object_id)?;
    if !sponsor_entry_fee_covered(game, &event) {
        adjust_characteristic(game, "budget", -event.entry_fee);
    }
    let event_stamina_cost = if event_is_motorsport(&event)
        && event.tags.split(';').any(|tag| normalized(tag) == "race")
    {
        config_f64(&game.catalog, "race_day_stamina_cost", 20.0)
            * event_duration_days(&event).max(1) as f64
    } else {
        event.stamina_cost.max(0.0)
    };
    adjust_characteristic(game, "stamina", -event_stamina_cost);
    if cosmetic_penalty > 0.0 {
        adjust_characteristic(game, "charisma", -cosmetic_penalty);
    }
    let id = format!("event_entry_{}_{}", event.id, game.current_day);
    game.pending_events.push(PendingEvent {
        id,
        event_id: event.id.clone(),
        object_id: object_id.into(),
        entered_day: game.current_day,
        rented: false,
        rental_expires_day: 0,
    });
    apply_bound_effects(game, "event_entered", &event.id, "")?;
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
    resolve_event_result(
        game,
        entry_id.to_string(),
        result.to_string(),
        String::new(),
        player_position,
        Vec::new(),
        None,
    )
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
    let mut staged = game.clone();
    let result = resolve_encounter_in_place(&mut staged, action_id)?;
    *game = staged;
    Ok(result)
}

fn resolve_encounter_in_place(
    game: &mut GameState,
    action_id: Option<&str>,
) -> Result<serde_json::Value, String> {
    let mut encounter = game
        .active_encounter
        .take()
        .ok_or_else(|| "No active encounter".to_string())?;
    if let Some(action_id) = action_id {
        validate_requirement_binding(game, "encounter_action", action_id)?;
    }
    let mut encounter_catalog = game.catalog.clone();
    for action in &mut encounter_catalog.encounter_actions {
        action.base_success_rate = apply_numeric_modifier_target(
            game,
            &format!("encounter_action_success_probability:{}", action.action_id),
            action.base_success_rate,
        )?
        .clamp(0.0, 1.0);
        action.effect_on_success = apply_numeric_modifier_target(
            game,
            &format!("encounter_action_effect_success:{}", action.action_id),
            action.effect_on_success,
        )?;
        action.effect_on_failure = apply_numeric_modifier_target(
            game,
            &format!("encounter_action_effect_failure:{}", action.action_id),
            action.effect_on_failure,
        )?;
        action.resource_cost_amount = apply_numeric_modifier_target(
            game,
            &format!("encounter_action_resource_cost:{}", action.action_id),
            action.resource_cost_amount,
        )?
        .max(0.0);
    }
    let mut inventory: Vec<String> = game.player.inventory.iter().map(|o| o.id.clone()).collect();
    engine::encounter::play_turn(
        &encounter_catalog,
        &mut encounter,
        action_id,
        &mut inventory,
    )?;
    while !encounter.finished && encounter.current_actor == "opponent" {
        engine::encounter::play_turn(&encounter_catalog, &mut encounter, None, &mut inventory)?;
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
                        let owned = build_owned_object(&def, game, false, 0);
                        game.player.inventory.push(owned);
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
                                obligation_states: HashMap::new(),
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
                            if game
                                .player
                                .inventory
                                .iter()
                                .any(|object| owned_matches_definition(object, &object_id))
                            {
                                continue;
                            }
                            if let Some(definition) = game
                                .catalog
                                .objects
                                .iter()
                                .find(|object| object.id == object_id)
                                .cloned()
                            {
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
                                    game,
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
        apply_bound_effects(
            game,
            "encounter_completed",
            &encounter.encounter_id,
            &result.outcome,
        )?;
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
                .into_iter()
                .filter(|action_id| {
                    validate_requirement_binding(game, "encounter_action", action_id).is_ok()
                })
                .collect()
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetTable {
    pub file: String,
    pub headers: Vec<String>,
    pub rows: Vec<HashMap<String, String>>,
}

fn editor_dataset_path(dataset_path: &str) -> Result<PathBuf, String> {
    let dataset = Path::new(dataset_path)
        .canonicalize()
        .map_err(|error| format!("Dataset folder cannot be resolved: {error}"))?;
    if !dataset.is_dir() {
        return Err("Dataset path is not a folder".into());
    }
    Ok(dataset)
}

fn editor_table_path(dataset_path: &str, file: &str) -> Result<PathBuf, String> {
    let file = file.trim();
    if file.is_empty()
        || !file.ends_with(".csv")
        || file.contains('/')
        || file.contains('\\')
        || file == "."
        || file == ".."
    {
        return Err("Dataset table must be a CSV file in the selected dataset folder".into());
    }
    Ok(editor_dataset_path(dataset_path)?.join(file))
}

#[tauri::command]
fn list_dataset_tables(dataset_path: String) -> Result<Vec<DatasetTable>, String> {
    let dataset = editor_dataset_path(&dataset_path)?;
    let mut files = std::fs::read_dir(&dataset)
        .map_err(|error| format!("Cannot read dataset folder: {error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("csv"))
        .collect::<Vec<_>>();
    files.sort();
    let mut tables = Vec::new();
    for path in files {
        let file = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "Dataset contains a table with an invalid filename".to_string())?
            .to_string();
        let mut reader = csv::Reader::from_path(&path)
            .map_err(|error| format!("Cannot read {file}: {error}"))?;
        let headers = reader
            .headers()
            .map_err(|error| format!("Cannot read headers from {file}: {error}"))?
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let mut rows = Vec::new();
        for record in reader.records() {
            let record = record.map_err(|error| format!("Cannot read {file}: {error}"))?;
            let row = headers
                .iter()
                .enumerate()
                .map(|(index, header)| {
                    (
                        header.clone(),
                        record.get(index).unwrap_or_default().to_string(),
                    )
                })
                .collect::<HashMap<_, _>>();
            rows.push(row);
        }
        tables.push(DatasetTable { file, headers, rows });
    }
    Ok(tables)
}

#[tauri::command]
fn save_dataset_table(
    dataset_path: String,
    file: String,
    headers: Vec<String>,
    rows: Vec<HashMap<String, String>>,
) -> Result<(), String> {
    if headers.is_empty() || headers.iter().any(|header| header.trim().is_empty()) {
        return Err("A dataset table needs at least one non-empty column".into());
    }
    let path = editor_table_path(&dataset_path, &file)?;
    let temporary = path.with_extension("csv.tmp");
    let result = (|| {
        let mut writer = csv::Writer::from_path(&temporary)
            .map_err(|error| format!("Cannot open temporary table: {error}"))?;
        writer
            .write_record(&headers)
            .map_err(|error| format!("Cannot write table headers: {error}"))?;
        for row in rows {
            writer
                .write_record(headers.iter().map(|header| row.get(header).map(String::as_str).unwrap_or_default()))
                .map_err(|error| format!("Cannot write table row: {error}"))?;
        }
        writer
            .flush()
            .map_err(|error| format!("Cannot flush table: {error}"))?;
        std::fs::rename(&temporary, &path)
            .map_err(|error| format!("Cannot replace dataset table: {error}"))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
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
    decode_save_payload(&payload)
}

fn decode_save_payload(payload: &str) -> Result<GameState, String> {
    let value: serde_json::Value = serde_json::from_str(payload)
        .map_err(|error| format!("Cannot decode save JSON: {error}"))?;
    let migrated = engine::save::migrate_payload(value)?;
    serde_json::from_value(migrated).map_err(|error| format!("Cannot decode save state: {error}"))
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
    let mut loaded = loaded;
    refresh_manager_sponsor_offers(&mut loaded);
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    *game = loaded.clone();
    Ok(loaded)
}

#[tauri::command]
fn delete_save_slot(dataset_path: String, slot: String) -> Result<(), String> {
    let path = save_database_path_for_slot(&dataset_path, &slot)?;
    if !path.exists() {
        return Err("Save slot not found".into());
    }
    std::fs::remove_file(&path).map_err(|error| format!("Cannot delete save: {error}"))
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
        decode_save_payload(&payload)?
    } else {
        let path = save_file_path(&current_path)?;
        let data = std::fs::read(&path).map_err(|error| format!("Cannot read save: {error}"))?;
        decode_save_payload(
            std::str::from_utf8(&data)
                .map_err(|error| format!("Cannot decode save as UTF-8: {error}"))?,
        )?
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
    let mut loaded = loaded;
    refresh_manager_sponsor_offers(&mut loaded);
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
    let mut staged = game.clone();
    join_quest_in_place(&mut staged, quest_id)?;
    *game = staged;
    Ok(())
}

fn join_quest_in_place(game: &mut GameState, quest_id: &str) -> Result<(), String> {
    validate_requirement_binding(game, "quest_join", quest_id)?;
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
    require_resource(game, "budget", quest.join_fee, "join this quest")?;
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
        && !game
            .player
            .inventory
            .iter()
            .any(|object| owned_matches_definition(object, &quest.required_license_id))
    {
        return Err(format!(
            "Cannot join '{}': required licence '{}' is missing.",
            quest.name, quest.required_license_id
        ));
    }
    if characteristic_value(game, "budget") < quest.join_fee {
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
    apply_bound_effects(game, "quest_joined", quest_id, "")?;
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
    if characteristic_value(&game, "budget") < amount {
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
    GameCatalog::clear_cached_directory(&new_path);
    game.catalog = GameCatalog::load_from_directory_cached(&new_path);
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
    for (index, diagnostic) in catalog_reload_diagnostics(&game).into_iter().enumerate() {
        game.pending_alerts.push(GameAlert {
            id: format!("dataset_reload_diagnostic_{current_day}_{index}"),
            title: "Saved state reference warning".into(),
            message: diagnostic,
        });
    }
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
    record_missed_races(game, previous_day);
    game.current_day += 1;
    game.player.age_days += 1;
    let current_day = game.current_day;
    apply_sponsor_monthly_payments(game, current_day);
    let mut expired_names = Vec::new();
    let mut expired_loaned_object_ids = Vec::new();
    let mut expired_loaned_definition_ids = Vec::new();
    let mut expired_rental_ids = Vec::new();
    game.pending_events.retain(|entry| {
        let expired =
            entry.rented && entry.rental_expires_day > 0 && entry.rental_expires_day < current_day;
        if expired {
            expired_rental_ids.push(entry.id.clone());
        }

        !expired
    });
    if !expired_rental_ids.is_empty() {
        game.pending_alerts.push(GameAlert {
            id: format!("expired_rentals_{current_day}"),
            title: "Rental Expired".into(),
            message: format!(
                "{} rental event(s) expired and were returned.",
                expired_rental_ids.len()
            ),
        });
    }
    game.player.inventory.retain(|object| {
        let expired = object.expires_day > 0 && object.expires_day <= current_day;
        if expired {
            expired_names.push(object.name.clone());
            if object.loaned {
                expired_loaned_object_ids.push(object.id.clone());
                expired_loaned_definition_ids.push(object.definition_id.clone());
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
            let sponsor_object_expired = expired_loaned_definition_ids
                .iter()
                .any(|definition_id| definition_id == &action.sponsor_object_id);
            !sponsor_object_expired
                || game.player.inventory.iter().any(|object| {
                    object.loaned && owned_matches_definition(object, &action.sponsor_object_id)
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
    apply_bound_effects(game, "day_elapsed", &current_day.to_string(), "")?;

    for object in &mut game.player.inventory {
        for (interval, needed) in [
            (object.service_1_interval_days, &mut object.service_1_needed),
            (object.service_2_interval_days, &mut object.service_2_needed),
            (object.service_3_interval_days, &mut object.service_3_needed),
            (object.service_4_interval_days, &mut object.service_4_needed),
            (object.service_5_interval_days, &mut object.service_5_needed),
            (object.service_6_interval_days, &mut object.service_6_needed),
            (object.service_7_interval_days, &mut object.service_7_needed),
            (object.service_8_interval_days, &mut object.service_8_needed),
            (object.service_9_interval_days, &mut object.service_9_needed),
            (
                object.service_10_interval_days,
                &mut object.service_10_needed,
            ),
            (
                object.service_11_interval_days,
                &mut object.service_11_needed,
            ),
            (
                object.service_12_interval_days,
                &mut object.service_12_needed,
            ),
            (
                object.service_13_interval_days,
                &mut object.service_13_needed,
            ),
            (
                object.service_14_interval_days,
                &mut object.service_14_needed,
            ),
            (
                object.service_15_interval_days,
                &mut object.service_15_needed,
            ),
        ] {
            if interval > 0 && current_day % interval == 0 {
                *needed = true;
            }
        }
    }

    let had_event = game.player.last_event_day == Some(previous_day);
    let recovery_rules_configured = game.catalog.resource_roles.recovery.is_some()
        && has_configured_rule(
            &game.catalog,
            &[
                "nightly_stamina_recovery",
                "daily_stamina_recovery",
                "weekend_stamina_recovery",
            ],
        );
    let sickness_rules_configured = game.catalog.resource_roles.recovery.is_some()
        && game
            .catalog
            .labels
            .values
            .contains_key("sickness_daily_probability");
    if sickness_rules_configured {
        if let Some(start_day) = game.player.sickness_start_day {
            let sickness_day = current_day.saturating_sub(start_day);
            if sickness_day < 2 {
                let current = characteristic_value(game, "stamina");
                adjust_characteristic(game, "stamina", 10.0 - current);
            } else if sickness_day < 6 {
                let target = config_f64(&game.catalog, "sickness_recovery_stamina", 50.0);
                let current = characteristic_value(game, "stamina");
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
    }
    let failed_obligation_events = process_obligations(game, current_day);
    if recovery_rules_configured && game.player.sickness_start_day.is_none() && !had_event {
        let recovery_key = if weekday(current_day) >= 6 {
            "weekend_stamina_recovery"
        } else {
            "daily_stamina_recovery"
        };
        let recovery = config_f64(
            &game.catalog,
            recovery_key,
            config_f64(&game.catalog, "nightly_stamina_recovery", 25.0),
        );
        adjust_characteristic(&mut *game, "stamina", recovery);
    }

    if sickness_rules_configured && game.player.sickness_start_day.is_none() {
        let sickness_probability =
            config_f64(&game.catalog, "sickness_daily_probability", 0.001111111).clamp(0.0, 1.0);
        if roll(game) < sickness_probability {
            game.player.sickness_start_day = Some(current_day);
            game.player.sickness_salary_blocked_until_day = Some(current_day.saturating_add(6));
            let current = characteristic_value(game, "stamina");
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
    for (active_index, active) in game.player.active_events.iter().enumerate() {
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
                    && !failed_obligation_events.contains(&active_index)
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

fn record_missed_races(game: &mut GameState, race_day: u32) {
    let day_of_year = ((race_day.saturating_sub(1)) % game.days_per_year) + 1;
    let races = game
        .catalog
        .events
        .iter()
        .filter(|event| {
            event.day_of_year == day_of_year
                && event
                    .tags
                    .split(';')
                    .any(|tag| normalized(tag) == "race")
                && !event.quest_id.trim().is_empty()
                && game
                    .quest_memberships
                    .iter()
                    .any(|membership| membership.quest_id == event.quest_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut missed_sponsor_bonus = 0.0;
    let mut completed_contract_ids = Vec::new();
    for event in races {
        let was_entered = game
            .pending_events
            .iter()
            .any(|entry| entry.event_id == event.id && entry.entered_day == race_day)
            || game
                .event_history
                .iter()
                .any(|entry| entry.event_id == event.id && entry.entered_day == race_day);
        if was_entered {
            continue;
        }
        game.event_history.push(EventHistory {
            id: format!("event_missed_{}_{}", event.id, race_day),
            event_id: event.id.clone(),
            object_id: String::new(),
            entered_day: race_day,
            result: "DNF".into(),
            outcome: "Unsuccessful".into(),
            reward_awarded: 0.0,
            charisma_reward_awarded: 0.0,
            damage_type: String::new(),
            player_position: 0,
            pole_position: false,
        });
        for contract in &game.sponsor_contracts {
            if sponsor_contract_matches_event(contract, &event) {
                missed_sponsor_bonus -= contract.dnf_penalty;
                if contract.scope.eq_ignore_ascii_case("race") {
                    completed_contract_ids.push(contract.id.clone());
                }
            }
        }
        log_event(
            game,
            format!("Event finished: {} (DNF - not entered)", event.name),
        );
        game.pending_alerts.push(GameAlert {
            id: format!("event_missed_{}_{}", event.id, race_day),
            title: "Race missed".into(),
            message: format!(
                "You did not join '{}'. The result was recorded as DNF.",
                event.name
            ),
        });
    }
    if missed_sponsor_bonus != 0.0 {
        adjust_characteristic(game, "budget", missed_sponsor_bonus);
    }
    if !completed_contract_ids.is_empty() {
        game.sponsor_contracts
            .retain(|contract| !completed_contract_ids.iter().any(|id| id == &contract.id));
    }
}

fn sponsor_contract_matches_event(contract: &SponsorContract, event: &EventData) -> bool {
    match contract.scope.to_ascii_lowercase().as_str() {
        "year" => true,
        "championship" | "quest" => {
            !contract.target_id.is_empty() && contract.target_id == event.quest_id
        }
        _ => contract.target_id.is_empty() || contract.target_id == event.id,
    }
}

fn sponsor_entry_fee_covered(game: &GameState, event: &EventData) -> bool {
    game.sponsor_contracts
        .iter()
        .any(|contract| contract.entry_fees && sponsor_contract_matches_event(contract, event))
}

fn sponsor_repair_coverage(game: &GameState) -> f64 {
    game.sponsor_contracts
        .iter()
        .filter(|contract| contract.maintenance)
        .map(|contract| contract.repair_coverage)
        .sum::<f64>()
        .max(0.0)
}

fn apply_sponsor_monthly_payments(game: &mut GameState, current_day: u32) {
    if current_day == 0 || current_day % 30 != 0 {
        return;
    }
    let mut total = 0.0;
    for contract in &mut game.sponsor_contracts {
        if contract.monthly_payment > 0.0
            && current_day > contract.signed_day
            && (contract.expires_day == 0 || current_day < contract.expires_day)
            && contract.last_payment_day < current_day
        {
            total += contract.monthly_payment;
            contract.last_payment_day = current_day;
        }
    }
    if total > 0.0 {
        adjust_characteristic(game, "budget", total);
        game.pending_alerts.push(GameAlert {
            id: format!("sponsor_payment_{current_day}"),
            title: "Sponsor payment received".into(),
            message: format!("Monthly sponsor payments received: {:.0}.", total),
        });
    }
    game.sponsor_contracts.retain(|contract| {
        contract.expires_day == 0 || contract.expires_day > current_day
    });
}

#[tauri::command]
fn rent_event(
    object_id: String,
    event_id: String,
    state: State<'_, AppState>,
) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    rent_event_for_sim(&mut game, &event_id, &object_id)?;
    Ok(game.clone())
}

pub fn rent_event_for_sim(
    game: &mut GameState,
    event_id: &str,
    object_id: &str,
) -> Result<(), String> {
    let mut staged = game.clone();
    rent_event_in_place(&mut staged, event_id, object_id)?;
    *game = staged;
    Ok(())
}

fn rent_event_in_place(
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
    let rental_cost = rental_event_error(game, &event, object_id)?;
    let entry_id = format!("event_entry_{}_{}", event.id, game.current_day);
    if game.pending_events.iter().any(|entry| entry.id == entry_id)
        || game
            .event_history
            .iter()
            .any(|entry| entry.event_id == event.id && entry.entered_day == game.current_day)
    {
        return Err("This event has already been entered today".into());
    }
    let race_event = event.tags.split(';').any(|tag| normalized(tag) == "race");
    let event_stamina_cost = if race_event {
        config_f64(&game.catalog, "race_day_stamina_cost", 20.0)
            * event_duration_days(&event).max(1) as f64
    } else {
        event.stamina_cost.max(0.0)
    };
    let entered_day = game.current_day;
    let duration_days = event_duration_days(&event);
    let event_id = event.id.clone();
    let entry_fee = if sponsor_entry_fee_covered(game, &event) {
        0.0
    } else {
        event.entry_fee
    };
    require_resource(
        game,
        "budget",
        entry_fee + rental_cost,
        "rent this event",
    )?;
    if event_stamina_cost > 0.0 {
        require_resource(game, "stamina", event_stamina_cost, "enter this event")?;
    }
    adjust_characteristic(game, "budget", -(entry_fee + rental_cost));
    adjust_characteristic(game, "stamina", -event_stamina_cost);
    let rental_expires_day = rental_expiry_day(game, object_id, entered_day)
        .max(entered_day.saturating_add(duration_days));
    game.pending_events.push(PendingEvent {
        id: entry_id,
        event_id,
        object_id: object_id.to_string(),
        entered_day,
        rented: true,
        rental_expires_day,
    });
    apply_bound_effects(game, "event_entered", &event.id, "")?;
    for _ in 0..duration_days {
        advance_one_day(game)?;
    }
    Ok(())
}

#[tauri::command]
fn build_owned_object(
    object: &ObjectData,
    game: &mut GameState,
    loaned: bool,
    expires_day: u32,
) -> OwnedObject {
    let instance_id = next_instance_id(game, &object.id);
    let expires_day = if loaned && expires_day == 0 && object.rental_duration_days > 0 {
        game.current_day.saturating_add(object.rental_duration_days)
    } else {
        expires_day
    };
    OwnedObject {
        id: instance_id.clone(),
        definition_id: object.id.clone(),
        instance_id,
        object_type: object.object_type.clone(),
        name: object.name.clone(),
        price: object.price,
        policy_version: object.policy_version,
        buyable: object.buyable,
        sellable: object.sellable,
        reward_only: object.reward_only,
        unique: object.unique,
        max_owned: object.max_owned,
        use_policy: object.use_policy.clone(),
        consume_policy: object.consume_policy.clone(),
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

#[tauri::command]
fn switch_insurance(object_id: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    switch_insurance_for_sim(&mut game, &object_id)?;
    Ok(game.clone())
}

pub fn switch_insurance_for_sim(game: &mut GameState, object_id: &str) -> Result<(), String> {
    let mut staged = game.clone();
    let target = staged
        .catalog
        .objects
        .iter()
        .find(|object| object.id == object_id && object.object_type.eq_ignore_ascii_case("insurance"))
        .cloned()
        .ok_or_else(|| "Insurance policy not found".to_string())?;
    let current_index = staged
        .player
        .inventory
        .iter()
        .position(|object| object.object_type.eq_ignore_ascii_case("insurance"))
        .ok_or_else(|| "No active insurance policy to replace".to_string())?;
    let current = staged.player.inventory[current_index].clone();
    let current_price = staged
        .catalog
        .objects
        .iter()
        .find(|object| object.id == current.definition_id)
        .map(|object| object.price)
        .unwrap_or(current.price);
    let year_end = staged.current_day % staged.days_per_year.max(1) == 0;
    if target.price < current_price && !year_end {
        return Err("Insurance downgrades are only available at the end of the year".into());
    }
    if target.id == current.definition_id {
        return Err("This insurance policy is already active".into());
    }
    staged.player.inventory.remove(current_index);
    buy_object_in_place(&mut staged, object_id)?;
    log_event(
        &mut staged,
        format!("Insurance changed from '{}' to '{}'", current.name, target.name),
    );
    *game = staged;
    Ok(())
}

#[tauri::command]
fn terminate_insurance(object_id: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    terminate_insurance_for_sim(&mut game, &object_id)?;
    Ok(game.clone())
}

pub fn terminate_insurance_for_sim(game: &mut GameState, object_id: &str) -> Result<(), String> {
    let index = game
        .player
        .inventory
        .iter()
        .position(|object| {
            object.id == object_id && object.object_type.eq_ignore_ascii_case("insurance")
        })
        .ok_or_else(|| "Insurance policy not found".to_string())?;
    let policy_name = game.player.inventory[index].name.clone();
    game.player.inventory.remove(index);
    log_event(game, format!("Insurance policy '{}' terminated", policy_name));
    Ok(())
}

#[tauri::command]
fn get_object_transaction_eligibility(
    state: State<'_, AppState>,
) -> Result<Vec<ObjectTransactionEligibility>, String> {
    let game = state.0.lock().map_err(|e| e.to_string())?;
    Ok(object_transaction_eligibility(&game))
}

pub fn buy_object_for_sim(game: &mut GameState, object_id: &str) -> Result<(), String> {
    let mut staged = game.clone();
    buy_object_in_place(&mut staged, object_id)?;
    refresh_manager_sponsor_offers(&mut staged);
    *game = staged;
    Ok(())
}

fn buy_object_in_place(game: &mut GameState, object_id: &str) -> Result<(), String> {
    validate_requirement_binding(game, "acquire", object_id)?;
    let object = game
        .catalog
        .objects
        .iter()
        .find(|object| object.id == object_id)
        .cloned()
        .ok_or_else(|| "Object not found in catalog".to_string())?;
    let policy = object.policy();
    if !policy.buyable || policy.reward_only {
        return Err(format!(
            "'{}' is earned through gameplay and cannot be bought",
            object.name
        ));
    }
    let owned_count = game
        .player
        .inventory
        .iter()
        .filter(|owned| {
            if object.object_type.eq_ignore_ascii_case("insurance") {
                owned.object_type.eq_ignore_ascii_case("insurance")
            } else {
                owned_matches_definition(owned, &object.id)
            }
        })
        .count() as u32;
    if policy.unique && owned_count > 0 {
        if object.object_type.eq_ignore_ascii_case("license") {
            return Err(format!(
                "Licence '{}' has already been purchased",
                object.name
            ));
        }
        if object.object_type.eq_ignore_ascii_case("insurance") {
            return Err("Only one insurance policy can be active at a time".into());
        }
        return Err(format!("'{}' is unique and is already owned", object.name));
    }
    if policy.max_owned > 0 && owned_count >= policy.max_owned {
        return Err(format!(
            "Cannot acquire '{}': maximum owned count is {}",
            object.name, policy.max_owned
        ));
    }
    let acquisition_cost = if object.object_type == "license" && object.license_fee > 0.0 {
        object.license_fee
    } else {
        object.price
    };
    let acquisition_cost =
        apply_numeric_modifier_target(game, "object_acquisition_cost", acquisition_cost)?.max(0.0);
    require_resource(game, "budget", acquisition_cost, "acquire this object")?;
    if characteristic_value(game, "budget") < acquisition_cost {
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
    apply_bound_effects(game, "object_acquired", object_id, "")?;
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
            let matching_actions = game
                .catalog
                .events
                .iter_mut()
                .filter(|action| parts[1] == "*" || action.id == parts[1])
                .count();
            if matching_actions == 0 {
                return Err(format!("Unknown action '{}'", parts[1]));
            }
            for action in game
                .catalog
                .events
                .iter_mut()
                .filter(|action| parts[1] == "*" || action.id == parts[1])
            {
                action.success_rate = parsed.clamp(0.0, 1.0);
            }
        }
        ("event", "success_rate" | "success_probability") => {
            let matching_events = game
                .catalog
                .events
                .iter_mut()
                .filter(|event| parts[1] == "*" || event.id == parts[1])
                .count();
            if matching_events == 0 {
                return Err(format!("Unknown event '{}'", parts[1]));
            }
            for event in game
                .catalog
                .events
                .iter_mut()
                .filter(|event| parts[1] == "*" || event.id == parts[1])
            {
                event.success_rate = parsed.clamp(0.0, 1.0);
            }
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
    service_object_for_sim(&mut game, &object_id, service_type)?;
    Ok(game.clone())
}

pub fn service_object_for_sim(
    game: &mut GameState,
    object_id: &str,
    service_type: ServiceType,
) -> Result<(), String> {
    let mut staged = game.clone();
    service_object_in_place(&mut staged, object_id, service_type)?;
    *game = staged;
    Ok(())
}

fn service_object_in_place(
    game: &mut GameState,
    object_id: &str,
    service_type: ServiceType,
) -> Result<(), String> {
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
    let payable_cost = apply_numeric_modifier_target(game, "service_cost", payable_cost)?.max(0.0);
    let payable_cost = (payable_cost - sponsor_repair_coverage(game)).max(0.0);
    require_resource(game, "budget", payable_cost, "service this object")?;
    if characteristic_value(game, "budget") < payable_cost {
        return Err("Insufficient funds for service".into());
    }
    adjust_characteristic(game, "budget", -payable_cost);
    if let Some(occurrence) = pending_occurrence {
        game.cost_ledger[occurrence].amount = payable_cost;
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
    log_event(game, format!("{} serviced", serviced_object_name));
    Ok(())
}

pub fn perform_event_for_sim(
    game: &mut GameState,
    event_id: &str,
) -> Result<EventStartResult, String> {
    let mut staged = game.clone();
    let result = perform_event_in_place(&mut staged, event_id)?;
    *game = staged;
    Ok(result)
}

fn perform_event_in_place(
    game: &mut GameState,
    event_id: &str,
) -> Result<EventStartResult, String> {
    let action: EventData = game
        .catalog
        .events
        .iter()
        .find(|action| action.id == event_id)
        .cloned()
        .ok_or_else(|| "Action not found in catalog".to_string())?;
    validate_player_started_event(game, &action)?;
    let upfront_stamina_cost = event_upfront_stamina_cost(game, &action);
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
                && !game
                    .player
                    .inventory
                    .iter()
                    .any(|object| owned_matches_definition(object, &quest.required_license_id))
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
        if action.encounter_id.trim().is_empty()
            && (!action.resolution_method.eq_ignore_ascii_case("encounter")
                || action.event_type.eq_ignore_ascii_case("sponsor"))
        {
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
    if action.payout_freq_type.eq_ignore_ascii_case("recurring")
        && game
            .player
            .active_events
            .iter()
            .any(|active| active.event_id == action.id)
    {
        return Err("This action is already active".into());
    }
    if let Some(obligations) = obligations_for_event(game, &action.id) {
        for obligation in obligations {
            if !obligation.required_event_type.trim().is_empty()
                && !required_event_type_is_active(game, &obligation.required_event_type)
            {
                return Err(format!(
                    "This action requires an active {}.",
                    obligation.required_event_type
                ));
            }
            if obligation.max_active > 0
                && active_obligation_count(game, &action.id) >= obligation.max_active as usize
            {
                return Err(format!(
                    "You cannot have more than {} active instances of this action.",
                    obligation.max_active
                ));
            }
        }
    }
    if job_start_blocked(game, &action) {
        return Err("This job cannot be started alongside your current jobs.".into());
    }

    adjust_characteristic(game, "budget", -action.base_cost);
    adjust_characteristic(game, "stamina", -upfront_stamina_cost);
    game.player.last_event_day = Some(game.current_day);
    apply_bound_effects(game, "event_started", &action.id, "")?;
    let success = if action.resolution_method.eq_ignore_ascii_case("encounter") {
        true
    } else {
        let success_target = if action.event_type.eq_ignore_ascii_case("sponsor") {
            "sponsor_success_probability"
        } else {
            "action_success_probability"
        };
        let success_probability =
            apply_numeric_modifier_target(game, success_target, action.success_rate)?;
        roll(game) <= success_probability
    };
    let payout = if success && !action.payout_freq_type.eq_ignore_ascii_case("recurring") {
        action.payout
    } else {
        0.0
    };
    let payout = apply_numeric_modifier_target(game, "action_payout", payout)?.max(0.0);
    require_resource(game, "budget", payout, "apply this action's payout")?;
    adjust_characteristic(game, "budget", payout);
    if success
        && (action.payout_freq_type.eq_ignore_ascii_case("recurring")
            || event_has_obligation(game, &action.id))
        && !action.event_type.eq_ignore_ascii_case("sponsor")
    {
        let start_day = game.current_day;
        game.player.active_events.push(ActiveEvent {
            event_id: action.id.clone(),
            start_day,
            obligation_payments: 0,
            obligation_faults: 0,
            obligation_states: HashMap::new(),
        });
    }
    let action_context = TriggerContext {
        trigger_type: "event_completed".into(),
        trigger_ref: action.id.clone(),
        source_type: "event".into(),
        source_id: action.id.clone(),
        outcome: Some(if success { "success" } else { "failure" }.into()),
        tags: event_tags(&action),
        ..TriggerContext::default()
    };
    let current_day = game.current_day;
    evaluate_cost_rules(game, &action_context, current_day)?;
    log_event(
        game,
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

    let failure_template = label(
        &game.catalog,
        "action_failed_message",
        "Getting '{action}' did not work.",
    );
    let failure_message =
        resolve_text_variables(game, failure_template).replace("{action}", &action.name);
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
            failure_message
        },
    })
}

#[tauri::command]
fn perform_event(event_id: String, state: State<'_, AppState>) -> Result<EventStartResult, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    perform_event_for_sim(&mut game, &event_id)
}

#[tauri::command]
fn quit_event(event_id: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    quit_event_for_sim(&mut game, &event_id)?;
    Ok(game.clone())
}

pub fn quit_event_for_sim(game: &mut GameState, event_id: &str) -> Result<(), String> {
    let index = game
        .player
        .active_events
        .iter()
        .position(|active| active.event_id == event_id)
        .ok_or_else(|| "That recurring action is not active".to_string())?;
    game.player.active_events.remove(index);
    Ok(())
}

#[tauri::command]
fn enter_event(
    object_id: String,
    event_id: String,
    state: State<'_, AppState>,
) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    enter_event_for_sim(&mut game, &event_id, &object_id)?;
    Ok(game.clone())
}

#[tauri::command]
fn get_event_eligibility(
    event_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<EventEligibility>, String> {
    let game = state.0.lock().map_err(|e| e.to_string())?;
    Ok(event_entry_eligibility(&game, &event_id))
}

#[tauri::command]
fn sell_object(object_id: String, state: State<'_, AppState>) -> Result<GameState, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    sell_object_for_sim(&mut game, &object_id)?;
    Ok(game.clone())
}

pub fn sell_object_for_sim(game: &mut GameState, object_id: &str) -> Result<(), String> {
    let mut staged = game.clone();
    sell_object_in_place(&mut staged, object_id)?;
    *game = staged;
    Ok(())
}

fn sell_object_in_place(game: &mut GameState, object_id: &str) -> Result<(), String> {
    validate_requirement_binding(game, "sell", object_id)?;
    let index = game
        .player
        .inventory
        .iter()
        .position(|object| object.id == object_id)
        .ok_or_else(|| "Object not found in inventory".to_string())?;
    if game.player.inventory[index].loaned {
        return Err("Loaned sponsor objects cannot be sold".into());
    }
    if game.player.inventory[index]
        .object_type
        .eq_ignore_ascii_case("insurance")
    {
        return Err("Insurance policies cannot be sold".into());
    }
    let object_policy = game
        .catalog
        .objects
        .iter()
        .find(|definition| owned_matches_definition(&game.player.inventory[index], &definition.id))
        .map(ObjectData::policy);
    let sellable = object_policy
        .as_ref()
        .map(|policy| policy.sellable)
        .unwrap_or(game.player.inventory[index].sellable);
    if !sellable {
        if game.player.inventory[index]
            .object_type
            .eq_ignore_ascii_case("license")
        {
            return Err("Licences cannot be resold".into());
        }
        return Err("This object cannot be resold".into());
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
    adjust_characteristic(game, "budget", value);
    log_event(game, format!("{} sold for {}", sold_object_name, value));
    game.player.inventory.remove(index);
    apply_bound_effects(game, "object_sold", object_id, "")?;
    Ok(())
}

#[tauri::command]
fn open_race_results_plugin(
    entry_id: String,
    state: State<'_, AppState>,
) -> Result<RaceResultsPluginResponse, String> {
    let game = state.0.lock().map_err(|e| e.to_string())?;
    let entry = game
        .pending_events
        .iter()
        .find(|entry| entry.id == entry_id)
        .ok_or_else(|| "Pending event entry not found".to_string())?;
    let event = game
        .catalog
        .events
        .iter()
        .find(|event| event.id == entry.event_id)
        .cloned()
        .ok_or_else(|| "Event not found in catalog".to_string())?;
    let initial_competitors = game
        .championship_results
        .iter()
        .filter(|result| {
            game.catalog
                .events
                .iter()
                .find(|candidate| candidate.id == result.event_id)
                .map(|candidate| candidate.quest_id == event.quest_id)
                .unwrap_or(false)
        })
        .flat_map(|result| result.competitors.clone())
        .collect::<Vec<_>>();
    run_race_results_plugin(&game, &event, "", None, &initial_competitors, true, false)
}

#[tauri::command]
fn autodetect_race_results_plugin(
    entry_id: String,
    state: State<'_, AppState>,
) -> Result<RaceResultsPluginResponse, String> {
    let game = state.0.lock().map_err(|e| e.to_string())?;
    let entry = game
        .pending_events
        .iter()
        .find(|entry| entry.id == entry_id)
        .ok_or_else(|| "Pending event entry not found".to_string())?;
    let event = game
        .catalog
        .events
        .iter()
        .find(|event| event.id == entry.event_id)
        .cloned()
        .ok_or_else(|| "Event not found in catalog".to_string())?;
    let response = run_race_results_plugin(&game, &event, "", None, &[], false, true)?;
    Ok(response)
}

fn sponsor_value<'a>(agreement: &'a serde_json::Value, key: &str) -> Option<&'a serde_json::Value> {
    agreement.get("proposal").and_then(|proposal| proposal.get(key))
}

fn sponsor_number(agreement: &serde_json::Value, key: &str) -> f64 {
    sponsor_value(agreement, key)
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0)
        .max(0.0)
}

fn sponsor_bool(agreement: &serde_json::Value, key: &str) -> bool {
    sponsor_value(agreement, key)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

fn sponsor_contract_expiry(game: &GameState, scope: &str, target_id: &str) -> u32 {
    if scope.eq_ignore_ascii_case("year") {
        return game.current_day.saturating_add(game.days_per_year);
    }
    if scope.eq_ignore_ascii_case("championship") {
        let final_day = game
            .catalog
            .events
            .iter()
            .filter(|event| event.quest_id == target_id)
            .map(|event| event.day_of_year)
            .max()
            .unwrap_or_else(|| ((game.current_day - 1) % game.days_per_year) + 1);
        let current_day = ((game.current_day - 1) % game.days_per_year) + 1;
        let offset = if final_day >= current_day {
            final_day - current_day
        } else {
            game.days_per_year - current_day + final_day
        };
        return game.current_day.saturating_add(offset).saturating_add(1);
    }
    game.current_day.saturating_add(1)
}

fn apply_sponsor_agreement(
    game: &mut GameState,
    agreement: &serde_json::Value,
    stamina_cost: f64,
) -> Result<(), String> {
    let sponsor_id = agreement
        .get("sponsor_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let sponsor_name = agreement
        .get("sponsor_name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Sponsor")
        .to_string();
    let sponsor_tier = agreement
        .get("sponsor_tier")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("local")
        .to_string();
    let scope = agreement
        .get("scope")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("race")
        .to_string();
    let mut target_id = agreement
        .get("target_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let target_name = agreement
        .get("target_name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    if scope.eq_ignore_ascii_case("championship") || scope.eq_ignore_ascii_case("quest") {
        target_id = game
            .catalog
            .quests
            .iter()
            .find(|quest| quest.id == target_id || quest.name == target_name)
            .map(|quest| quest.id.clone())
            .or_else(|| {
                game.catalog
                    .events
                    .iter()
                    .find(|event| event.id == target_id && !event.quest_id.is_empty())
                    .map(|event| event.quest_id.clone())
            })
            .unwrap_or(target_id);
    }
    let initial_money = sponsor_number(agreement, "initial_money");
    let monthly_payment = sponsor_number(agreement, "monthly_payment");
    let entry_fees = sponsor_bool(agreement, "entry_fees");
    let maintenance = sponsor_bool(agreement, "maintenance");
    let repair_coverage = if maintenance {
        agreement
            .get("repair_value")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0)
            .max(0.0)
    } else {
        0.0
    };
    let car = sponsor_bool(agreement, "car");
    let gear = sponsor_bool(agreement, "gear");
    let result_bonus = sponsor_number(agreement, "result_bonus");
    let dnf_penalty = sponsor_number(agreement, "dnf_penalty");
    adjust_characteristic(game, "budget", initial_money);

    let requested_car_id = agreement
        .get("car_object_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let mut car_object_id = String::new();
    if car {
        car_object_id = game
            .catalog
            .objects
            .iter()
            .find(|object| {
                object.object_type.eq_ignore_ascii_case("vehicle")
                    && object.id == requested_car_id
            })
            .or_else(|| {
                game.catalog
                    .objects
                    .iter()
                    .find(|object| object.object_type.eq_ignore_ascii_case("vehicle"))
            })
            .map(|object| object.id.clone())
            .unwrap_or_default();
        if !car_object_id.is_empty() {
            if let Some(definition) = game
                .catalog
                .objects
                .iter()
                .find(|object| object.id == car_object_id)
                .cloned()
            {
                let expires_day = sponsor_contract_expiry(game, &scope, &target_id);
                let mut loaned = build_owned_object(&definition, game, true, expires_day);
                loaned.unavailable_until_day = game.current_day;
                game.player.inventory.push(loaned);
            }
        }
    }
    let expires_day = sponsor_contract_expiry(game, &scope, &target_id);
    let contract = SponsorContract {
        id: format!("sponsor_contract_{}_{}", game.current_day, game.sponsor_contracts.len() + 1),
        sponsor_id,
        sponsor_name: sponsor_name.clone(),
        sponsor_tier,
        scope: scope.clone(),
        race_tier: agreement
            .get("race_tier")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        target_id,
        target_name: target_name.clone(),
        signed_day: game.current_day,
        expires_day,
        initial_money,
        monthly_payment,
        entry_fees,
        maintenance,
        repair_coverage,
        car,
        car_object_id,
        gear,
        result_bonus,
        dnf_penalty,
        last_payment_day: game.current_day,
    };
    game.sponsor_contracts.push(contract);
    game.pending_alerts.push(GameAlert {
        id: format!("sponsor_signed_{}_{}", game.current_day, game.sponsor_contracts.len()),
        title: "Sponsor agreement signed".into(),
        message: format!(
            "{} agreement signed for {}. Negotiation cost {:.0} stamina; initial payment {:.0}.",
            sponsor_name, scope, stamina_cost, initial_money
        ),
    });
    Ok(())
}

fn enrich_sponsor_negotiation_state(
    state: &mut serde_json::Value,
    game: &GameState,
    selected_target: &str,
    selected_car: &str,
) {
    let scope = state
        .get("scope")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("race");
    let target_options: Vec<serde_json::Value> = if scope.eq_ignore_ascii_case("championship") {
        game.catalog
            .quests
            .iter()
            .map(|quest| serde_json::json!({ "id": quest.id, "name": quest.name }))
            .collect()
    } else if scope.eq_ignore_ascii_case("year") {
        Vec::new()
    } else {
        game.catalog
            .events
            .iter()
            .filter(|event| !event.event_type.eq_ignore_ascii_case("sponsor"))
            .map(|event| serde_json::json!({ "id": event.id, "name": event.name }))
            .collect()
    };
    let target_id = if scope.eq_ignore_ascii_case("championship") {
        game.catalog
            .quests
            .iter()
            .find(|quest| quest.name == selected_target || quest.id == selected_target)
            .map(|quest| quest.id.as_str())
    } else {
        game.catalog
            .events
            .iter()
            .find(|event| event.name == selected_target || event.id == selected_target)
            .map(|event| event.id.as_str())
    };
    let required_vehicle_ids: std::collections::HashSet<&str> = if scope.eq_ignore_ascii_case("year") {
        std::collections::HashSet::new()
    } else {
        game.catalog
            .events
            .iter()
            .filter(|event| {
                if scope.eq_ignore_ascii_case("championship") {
                    target_id.is_some_and(|id| event.quest_id == id)
                } else {
                    target_id.is_some_and(|id| event.id == id)
                }
            })
            .flat_map(|event| event.required_object_ids.split(';'))
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .collect()
    };
    let vehicle_options: Vec<serde_json::Value> = if !scope.eq_ignore_ascii_case("year")
        && target_id.is_none()
    {
        Vec::new()
    } else {
        game.catalog
            .objects
            .iter()
            .filter(|object| {
                object.object_type.eq_ignore_ascii_case("vehicle")
                    && (required_vehicle_ids.is_empty()
                        || required_vehicle_ids.contains(object.id.as_str()))
            })
            .map(|object| {
                serde_json::json!({
                    "id": object.id,
                    "name": object.name,
                    "price": object.price,
                })
            })
            .collect()
    };
    if let Some(object) = state.as_object_mut() {
        object.insert("target_options".into(), serde_json::Value::Array(target_options));
        object.insert("vehicle_options".into(), serde_json::Value::Array(vehicle_options));
        object.insert(
            "selected_target".into(),
            serde_json::Value::String(selected_target.to_string()),
        );
        object.insert(
            "selected_car".into(),
            serde_json::Value::String(selected_car.to_string()),
        );
    }
}

#[tauri::command]
fn open_sponsor_negotiation(
    event_id: String,
    sponsor_id: Option<String>,
    approach: String,
    state: State<'_, AppState>,
    plugin_state: State<'_, SponsorNegotiationProcessState>,
) -> Result<SponsorNegotiationStart, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    let mut active_process = plugin_state.0.lock().map_err(|e| e.to_string())?;
    if active_process.is_some() {
        return Err("A sponsor negotiation is already in progress".into());
    }
    let plugin_path = PathBuf::from(&game.dataset_path).join("plugins/sponsor_negotiator.py");
    if !plugin_path.is_file() {
        return Err(format!("Sponsor negotiation plugin was not found at '{}'", plugin_path.display()));
    }
    let python = std::env::var("TTRPG_PYTHON").unwrap_or_else(|_| "python3".into());
    let sponsor_label = sponsor_id.as_deref().unwrap_or("random").to_string();
    let scope = game
        .catalog
        .events
        .iter()
        .find(|event| event.id == event_id && event.event_type.eq_ignore_ascii_case("sponsor"))
        .map(|event| {
            if event.payout_freq_unit.eq_ignore_ascii_case("year") {
                "year"
            } else {
                "championship"
            }
        })
        .unwrap_or("race");
    let target_id = game
        .catalog
        .events
        .iter()
        .find(|event| event.id == event_id)
        .map(|event| {
            if scope == "championship" {
                event.quest_id.clone()
            } else {
                event.id.clone()
            }
        })
        .unwrap_or_default();
    let target_name = if scope == "championship" {
        game.catalog
            .quests
            .iter()
            .find(|quest| quest.id == target_id)
            .map(|quest| quest.name.clone())
            .unwrap_or_default()
    } else {
        game.catalog
            .events
            .iter()
            .find(|event| event.id == target_id)
            .map(|event| event.name.clone())
            .unwrap_or_default()
    };
    let stamina_cost = 8.0 + (roll(&mut game) * 9.0).floor();
    require_resource(
        &game,
        "stamina",
        stamina_cost,
        "attempt sponsor negotiations",
    )?;
    adjust_characteristic(&mut game, "stamina", -stamina_cost);
    let mut command = Command::new(&python);
    command
        .arg(&plugin_path)
        .arg("--json")
        .arg("--interactive")
        .args([
            "--dataset-path",
            &game.dataset_path,
            "--approach",
            if approach.eq_ignore_ascii_case("proposal") {
                "proposal"
            } else {
                "cold_call"
            },
            "--race-tier",
            "local",
            "--scope",
            scope,
            "--target-id",
            &target_id,
            "--target-name",
            &target_name,
        ]);
    if let Some(sponsor_id) = sponsor_id.filter(|value| !value.trim().is_empty()) {
        command.args(["--sponsor", sponsor_id.as_str()]);
    }
    let manager_id = game
        .catalog
        .labels
        .values
        .get("manager_object_id")
        .map(String::as_str)
        .unwrap_or("manager")
        .trim()
        .to_string();
    let has_manager = game.player.inventory.iter().any(|object| {
        object.id == manager_id
            || object.definition_id == manager_id
            || object.instance_id == manager_id
    });
    if has_manager {
        command.args(["--has-agent", "--agent-level", "1"]);
    }
    let mut child = match command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            return Err(format!(
                "Could not start sponsor negotiation plugin with '{python}': {error}"
            ));
        }
    };
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Sponsor negotiation plugin stdin was unavailable".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Sponsor negotiation plugin stdout was unavailable".to_string())?;
    let mut process = SponsorNegotiationProcess {
        child,
        stdin,
        stdout: BufReader::new(stdout),
        event_id,
        target_id,
        target_name: target_name.clone(),
        car_object_id: String::new(),
        stamina_cost,
    };
    let mut first_line = String::new();
    process
        .stdout
        .read_line(&mut first_line)
        .map_err(|error| format!("Could not read sponsor negotiation state: {error}"))?;
    if first_line.trim().is_empty() {
        let _ = process.child.kill();
        return Err("Sponsor negotiation plugin returned no initial state".into());
    }
    let mut initial_state: serde_json::Value = serde_json::from_str(first_line.trim())
        .map_err(|error| format!("Invalid sponsor negotiation state: {error}"))?;
    if initial_state.get("status").and_then(serde_json::Value::as_str) == Some("ERROR") {
        let _ = process.child.kill();
        return Err(initial_state
            .get("error")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Sponsor negotiation plugin failed")
            .to_string());
    }
    enrich_sponsor_negotiation_state(&mut initial_state, &game, &target_name, "No car included");
    consume_manager_sponsor_offer(&mut game, &process.event_id);
    *active_process = Some(process);
    eprintln!(
        "[sponsor-negotiator-plugin] started sponsor={} scope={} race_tier=local",
        sponsor_label, scope
    );
    Ok(SponsorNegotiationStart {
        state: initial_state,
        game_state: game.clone(),
    })
}

#[tauri::command]
fn sponsor_negotiation_action(
    action: serde_json::Value,
    state: State<'_, AppState>,
    plugin_state: State<'_, SponsorNegotiationProcessState>,
) -> Result<SponsorNegotiationActionResult, String> {
    let mut process_guard = plugin_state.0.lock().map_err(|e| e.to_string())?;
    let process = process_guard
        .as_mut()
        .ok_or_else(|| "No sponsor negotiation is in progress".to_string())?;
    let mut plugin_action = action;
    let action_name = plugin_action
        .get("action")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    if action_name == "select_target" {
        if let Some(target_name) = plugin_action
            .get("target_name")
            .and_then(serde_json::Value::as_str)
        {
            process.target_name = target_name.to_string();
        }
        process.car_object_id.clear();
    } else if action_name == "new_session" {
        process.target_name.clear();
        process.car_object_id.clear();
    }
    if let Some(car_object_id) = plugin_action
        .get("car_object_id")
        .and_then(serde_json::Value::as_str)
    {
        process.car_object_id = car_object_id.to_string();
    }
    if !action_name.is_empty() {
        plugin_action["action_id"] = serde_json::Value::String(action_name);
    }
    let request = serde_json::to_string(&plugin_action)
        .map_err(|error| format!("Invalid sponsor negotiation action: {error}"))?;
    process
        .stdin
        .write_all(request.as_bytes())
        .and_then(|_| process.stdin.write_all(b"\n"))
        .and_then(|_| process.stdin.flush())
        .map_err(|error| format!("Could not send sponsor negotiation action: {error}"))?;
    let mut line = String::new();
    process
        .stdout
        .read_line(&mut line)
        .map_err(|error| format!("Could not read sponsor negotiation response: {error}"))?;
    if line.trim().is_empty() {
        return Err("Sponsor negotiation plugin returned no response".into());
    }
    let mut response: serde_json::Value = serde_json::from_str(line.trim())
        .map_err(|error| format!("Invalid sponsor negotiation response: {error}"))?;
    if response.get("status").and_then(serde_json::Value::as_str) == Some("ERROR") {
        return Err(response
            .get("error")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Sponsor negotiation action failed")
            .to_string());
    }
    eprintln!(
        "[sponsor-negotiator-plugin] status={} log={}",
        response
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("UNKNOWN"),
        response
            .get("final_log")
            .or_else(|| response.get("last_action_log"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
    );
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    let selected_target = process.target_name.clone();
    let selected_car = game
        .catalog
        .objects
        .iter()
        .find(|object| object.id == process.car_object_id)
        .map(|object| object.name.as_str())
        .unwrap_or("No car included");
    enrich_sponsor_negotiation_state(&mut response, &game, &selected_target, selected_car);
    if response.get("status").and_then(serde_json::Value::as_str) == Some("SIGNED") {
        let mut agreement = response
            .get("agreement")
            .cloned()
            .ok_or_else(|| "Sponsor negotiation signed without an agreement".to_string())?;
        if let Some(agreement_object) = agreement.as_object_mut() {
            agreement_object.insert(
                "target_id".into(),
                serde_json::Value::String(process.target_id.clone()),
            );
            agreement_object.insert(
                "target_name".into(),
                serde_json::Value::String(process.target_name.clone()),
            );
            agreement_object.insert(
                "car_object_id".into(),
                serde_json::Value::String(process.car_object_id.clone()),
            );
        }
        let stamina_cost = process.stamina_cost;
        apply_sponsor_agreement(&mut game, &agreement, stamina_cost)?;
        if let Some(process) = process_guard.take() {
            let mut child = process.child;
            let _ = child.kill();
            let _ = child.wait();
        }
    }
    Ok(SponsorNegotiationActionResult {
        state: response,
        game_state: game.clone(),
    })
}

#[tauri::command]
fn close_sponsor_negotiation(
    plugin_state: State<'_, SponsorNegotiationProcessState>,
) -> Result<(), String> {
    let mut process_guard = plugin_state.0.lock().map_err(|e| e.to_string())?;
    if let Some(process) = process_guard.take() {
        let mut child = process.child;
        let _ = child.kill();
        let _ = child.wait();
    }
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
struct SponsorListItem {
    id: String,
    name: String,
    tier: String,
    interested_race_tiers: Vec<String>,
    preferred_categories: Vec<String>,
    base_cash: f64,
    monthly_payment: f64,
    repair_value: f64,
    brand: String,
}

#[tauri::command]
fn list_sponsors(state: State<'_, AppState>) -> Result<Vec<SponsorListItem>, String> {
    let game = state.0.lock().map_err(|e| e.to_string())?;
    let path = PathBuf::from(&game.dataset_path).join("sponsors.csv");
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(&path)
        .map_err(|error| format!("Cannot read sponsors.csv: {error}"))?;
    let headers = reader
        .headers()
        .map_err(|error| format!("Cannot read sponsors.csv headers: {error}"))?
        .clone();
    let mut sponsors = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|error| format!("Cannot parse sponsors.csv: {error}"))?;
        let value = |key: &str| {
            headers
                .iter()
                .position(|header| header == key)
                .and_then(|index| record.get(index))
                .unwrap_or("")
                .trim()
                .to_string()
        };
        let split = |key: &str| {
            value(key)
                .split(';')
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .map(String::from)
                .collect()
        };
        sponsors.push(SponsorListItem {
            id: value("id"),
            name: value("name"),
            tier: value("tier"),
            interested_race_tiers: split("interested_race_tiers"),
            preferred_categories: split("preferred_categories"),
            base_cash: value("base_cash").parse().unwrap_or(0.0),
            monthly_payment: value("monthly_payment").parse().unwrap_or(0.0),
            repair_value: value("repair_value").parse().unwrap_or(0.0),
            brand: value("brand"),
        });
    }
    Ok(sponsors)
}

fn resolve_event_result(
    game: &mut GameState,
    entry_id: String,
    result: String,
    damage_type: String,
    player_position: Option<u32>,
    competitors: Vec<ChampionshipCompetitor>,
    plugin_response: Option<RaceResultsPluginResponse>,
) -> Result<EventResult, String> {
    let mut staged = game.clone();
    let resolved = resolve_event_result_in_place(
        &mut staged,
        entry_id,
        result,
        damage_type,
        player_position,
        competitors,
        plugin_response,
    )?;
    *game = staged;
    Ok(resolved)
}

fn resolve_event_result_in_place(
    game: &mut GameState,
    entry_id: String,
    result: String,
    damage_type: String,
    player_position: Option<u32>,
    competitors: Vec<ChampionshipCompetitor>,
    plugin_response: Option<RaceResultsPluginResponse>,
) -> Result<EventResult, String> {
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
    let plugin_response = if event.tags.split(';').any(|tag| normalized(tag) == "race") {
        Some(plugin_response.unwrap_or(run_race_results_plugin(
            game,
            &event,
            &result,
            player_position,
            &competitors,
            false,
            false,
        )?))
    } else {
        None
    };
    if let Some(response) = plugin_response.as_ref() {
        let driver_name = response.driver_name.trim();
        if !driver_name.is_empty() {
            let mut config = read_app_config_file();
            if !config.driver_names.iter().any(|name| name.eq_ignore_ascii_case(driver_name)) {
                config.driver_names.push(driver_name.to_string());
                write_app_config_file(&config)?;
            }
        }
    }
    let (result, player_position, competitors, championship_standings, damage_type, pole_position) =
        if let Some(response) = plugin_response {
            (
                response.result,
                Some(response.player_position).filter(|position| *position > 0),
                response.competitors,
                response.standings,
                response.damage_type,
                response.pole_position,
            )
        } else {
            (
                result,
                player_position,
                competitors,
                Vec::new(),
                damage_type,
                false,
            )
        };
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
    let reported_success = if event.resolution_method.eq_ignore_ascii_case("random") {
        let success_probability =
            apply_numeric_modifier_target(game, "event_success_probability", event.success_rate)?;
        roll(game) <= success_probability
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
                    && roll(game) < outcome.probability.clamp(0.0, 1.0)
            })
    } else {
        None
    };
    let random_result = if random_outcome.is_none() {
        let tags = event_tags(&event);
        game.catalog
            .event_results
            .clone()
            .into_iter()
            .find(|candidate| {
                (candidate.event_id.trim().is_empty() || candidate.event_id == event.id)
                    && reported_result_matches(&candidate.reported_result, &result)
                    && candidate
                        .event_tags
                        .split(';')
                        .map(str::trim)
                        .filter(|tag| !tag.is_empty())
                        .all(|required| {
                            tags.iter()
                                .any(|actual| normalized(actual) == normalized(required))
                        })
                    && roll(game) < candidate.probability.clamp(0.0, 1.0)
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
    let reward = apply_numeric_modifier_target(game, "event_reward", reward)?.max(0.0);
    let charisma_reward = if let Some(outcome) = &random_outcome {
        outcome.charisma_reward_delta
    } else if success {
        event.charisma_reward
    } else {
        0.0
    };
    let charisma_reward =
        apply_numeric_modifier_target(game, "event_charisma_reward", charisma_reward)?.max(0.0);
    require_resource(game, "budget", reward, "apply this event reward")?;
    require_resource(
        game,
        "charisma",
        charisma_reward,
        "apply this event characteristic reward",
    )?;
    adjust_characteristic(game, "budget", reward);
    adjust_characteristic(game, "charisma", charisma_reward);
    if let Some(event_result) = &random_result {
        apply_event_effects(game, &event_result.effects)?;
    }
    apply_bound_effects(
        game,
        "event_completed",
        &event.id,
        if success { "success" } else { "failure" },
    )?;
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
        require_resource(game, "budget", payment, "apply this sponsor payment")?;
        adjust_characteristic(game, "budget", payment);
        payment
    } else {
        0.0
    };
    let mut sponsor_bonus = 0.0;
    let mut completed_contract_ids = Vec::new();
    for contract in &game.sponsor_contracts {
        if !sponsor_contract_matches_event(contract, &event) {
            continue;
        }
        let amount = if success {
            contract.result_bonus
        } else {
            -contract.dnf_penalty
        };
        sponsor_bonus += amount;
        if contract.scope.eq_ignore_ascii_case("race") {
            completed_contract_ids.push(contract.id.clone());
        }
    }
    if sponsor_bonus != 0.0 {
        adjust_characteristic(game, "budget", sponsor_bonus);
    }
    if !completed_contract_ids.is_empty() {
        game.sponsor_contracts
            .retain(|contract| !completed_contract_ids.iter().any(|id| id == &contract.id));
    }
    let object_type = game
        .player
        .inventory
        .iter()
        .find(|object| object.id == entry.object_id)
        .map(|object| object.object_type.clone())
        .or_else(|| {
            game.catalog
                .objects
                .iter()
                .find(|object| object.id == entry.object_id)
                .map(|object| object.object_type.clone())
        });
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
    if !entry.rented {
        evaluate_cost_rules(game, &event_context, current_day)?;
        mark_event_services_needed(game, &entry.object_id, &event);
    }
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
                    let owned = build_owned_object(&trophy, game, false, 0);
                    game.player.inventory.push(owned);
                }
            }
        }
    }
    game.pending_events.remove(entry_index);
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
        player_position: player_position.unwrap_or(0),
        pole_position,
    });
    log_event(
        game,
        format!(
            "Event finished: {} ({}){}",
            event.name,
            result,
            if pole_position { " (pole position)" } else { "" }
        ),
    );

    let entry_fee_covered = sponsor_entry_fee_covered(game, &event);
    Ok(EventResult {
        event_name: event.name,
        outcome: if success {
            "Success".into()
        } else {
            "Unsuccessful".into()
        },
        entry_fee_paid: if entry_fee_covered {
            0.0
        } else {
            event.entry_fee
        },
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
                format!(
                    "{}{}{}",
                    if sponsor_payment > 0.0 {
                        format!("; sponsor payment {}", sponsor_payment)
                    } else {
                        String::new()
                    },
                    if sponsor_bonus != 0.0 {
                        format!("; sponsor adjustment {}", sponsor_bonus)
                    } else {
                        String::new()
                    },
                    if pole_position { "; pole position" } else { "" }
                )
            )
        } else {
            format!(
                "The event ended without a reward: {}{}{}.",
                result,
                if sponsor_payment > 0.0 {
                    format!(" Sponsor payment: {}", sponsor_payment)
                } else {
                    String::new()
                },
                if sponsor_bonus != 0.0 {
                    format!(" Sponsor adjustment: {}", sponsor_bonus)
                } else {
                    String::new()
                }
            )
        },
        sponsor_payment,
        sponsor_bonus,
        damage_type,
        championship_standings,
    })
}

#[tauri::command]
fn submit_event_result(
    entry_id: String,
    result: String,
    damage_type: String,
    player_position: Option<u32>,
    competitors: Vec<ChampionshipCompetitor>,
    plugin_response: Option<RaceResultsPluginResponse>,
    state: State<'_, AppState>,
) -> Result<EventResult, String> {
    let mut game = state.0.lock().map_err(|e| e.to_string())?;
    resolve_event_result(
        &mut game,
        entry_id,
        result,
        damage_type,
        player_position,
        competitors,
        plugin_response,
    )
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
        .setup(|app| {
            let config = read_app_config_file();
            if let Some(window) = app.get_webview_window("main") {
                window
                    .set_size(tauri::Size::Logical(tauri::LogicalSize::new(
                        config.window_width as f64,
                        config.window_height as f64,
                    )))
                    .map_err(|error| format!("Cannot apply window size: {error}"))?;
                #[cfg(not(target_os = "android"))]
                window
                    .set_fullscreen(config.fullscreen)
                    .map_err(|error| format!("Cannot apply fullscreen setting: {error}"))?;
            }
            Ok(())
        })
        .manage(AppState(Mutex::new(create_initial_state())))
        .manage(SponsorNegotiationProcessState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            get_app_config,
            save_app_config,
            switch_insurance,
            terminate_insurance,
            open_race_results_plugin,
            autodetect_race_results_plugin,
            open_sponsor_negotiation,
            sponsor_negotiation_action,
            close_sponsor_negotiation,
            list_sponsors,
            get_game_state,
            get_catalog,
            get_theme_colors,
            list_dataset_tables,
            save_dataset_table,
            default_dataset_dialog_path,
            is_dataset_path,
            save_game,
            load_game,
            list_save_slots,
            latest_save_slot,
            start_new_game,
            save_game_as,
            load_game_from,
            delete_save_slot,
            set_time_speed,
            tick_game_day,
            pay_cost,
            buy_object,
            service_object,
            sell_object,
            perform_event,
            quit_event,
            enter_event,
            get_event_eligibility,
            rent_event,
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
            retreat_encounter,
            get_object_transaction_eligibility
        ])
        .run(tauri::generate_context!())
        .expect("error while running application");
}

#[cfg(test)]
mod tests {
    use super::{
        advance_day, advance_one_day, apply_bound_effects, apply_event_effects,
        apply_numeric_modifier_target, build_owned_object, buy_object_for_sim,
        mark_event_services_needed, mark_object_service_needed, new_game_seeded,
        process_obligations, sell_object_for_sim, validate_requirement_binding, ActiveEvent,
    };
    use crate::engine::loader::{
        ConditionData, ConditionGroupData, EffectBindingData, EffectData, NumericModifierData,
        RequirementBindingData,
    };
    use crate::engine::quest_runs::{QuestDefinition, RewardReceipt};
    use std::collections::{BTreeMap, HashMap};

    fn dataset_path() -> &'static str {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../dataset")
    }

    #[test]
    fn recurring_obligations_keep_independent_counters_and_write_legacy_fields_only_on_read() {
        let mut game = new_game_seeded(dataset_path(), 24);
        let mut first = game
            .catalog
            .obligations
            .iter()
            .find(|obligation| obligation.id == "formation_stamina")
            .cloned()
            .expect("dataset should contain formation stamina obligation");
        let mut second = first.clone();
        first.max_payments = 1;
        first.amount = 1.0;
        second.id = "formation_stamina_delivery".into();
        second.max_payments = 3;
        second.amount = 1.0;
        second.due_days = "2;3".into();
        game.catalog.obligations = vec![first, second];
        game.player.characteristics.insert("stamina".into(), 10.0);
        game.player.active_events.push(ActiveEvent {
            event_id: "act_formation".into(),
            start_day: 1,
            obligation_payments: 0,
            obligation_faults: 0,
            obligation_states: HashMap::new(),
        });

        process_obligations(&mut game, 2);
        process_obligations(&mut game, 3);

        let states = &game.player.active_events[0].obligation_states;
        assert_eq!(states["formation_stamina"].payments, 1);
        assert_eq!(states["formation_stamina_delivery"].payments, 2);

        let serialized = serde_json::to_value(&game.player.active_events[0])
            .expect("active event should serialize");
        assert!(serialized.get("obligation_payments").is_none());
        assert!(serialized.get("obligation_faults").is_none());

        let legacy: ActiveEvent = serde_json::from_value(serde_json::json!({
            "event_id": "act_formation",
            "start_day": 1,
            "obligation_payments": 4,
            "obligation_faults": 2
        }))
        .expect("legacy active event should remain readable");
        assert_eq!(
            legacy,
            ActiveEvent {
                event_id: "act_formation".into(),
                start_day: 1,
                obligation_payments: 4,
                obligation_faults: 2,
                obligation_states: HashMap::new(),
            }
        );
    }

    #[test]
    fn recurring_obligations_preserve_paid_and_faulted_status_across_save_replay() {
        let mut game = new_game_seeded(dataset_path(), 25);
        let mut paid = game
            .catalog
            .obligations
            .iter()
            .find(|obligation| obligation.id == "formation_stamina")
            .cloned()
            .expect("dataset should contain formation stamina obligation");
        paid.id = "paid_delivery".into();
        paid.amount = 2.0;
        paid.max_payments = 2;
        let mut faulted = paid.clone();
        faulted.id = "faulted_delivery".into();
        faulted.amount = 20.0;
        faulted.fault_limit = 0;
        game.catalog.obligations = vec![paid, faulted];
        game.player.characteristics.insert("stamina".into(), 5.0);
        game.player.active_events.push(ActiveEvent {
            event_id: "act_formation".into(),
            start_day: 1,
            obligation_payments: 0,
            obligation_faults: 0,
            obligation_states: HashMap::new(),
        });

        process_obligations(&mut game, 2);
        let saved = serde_json::to_string(&game).expect("state should serialize");
        let mut replayed = super::decode_save_payload(&saved).expect("save should load");
        process_obligations(&mut game, 3);
        process_obligations(&mut replayed, 3);

        let left = &game.player.active_events[0].obligation_states;
        let right = &replayed.player.active_events[0].obligation_states;
        assert_eq!(left, right);
        assert_eq!(left["paid_delivery"].payments, 2);
        assert_eq!(left["paid_delivery"].faults, 0);
        assert_eq!(left["faulted_delivery"].payments, 0);
        assert_eq!(left["faulted_delivery"].faults, 2);
    }

    #[test]
    fn quest_runs_and_reward_receipts_round_trip_through_save_payloads() {
        let mut game = new_game_seeded(dataset_path(), 21);
        let definition = QuestDefinition {
            quest_id: "seasonal-circuit".into(),
            enrollment_policy: crate::engine::quest_runs::EnrollmentPolicy::Manual,
            repeat_policy: crate::engine::quest_runs::RepeatPolicy::Periodic,
            completion: crate::engine::quest_runs::CompletionRule::AllRequired,
            required_event_ids: vec!["finale".into()],
            optional_event_ids: vec![],
        };
        game.quest_runs.push(
            crate::engine::quest_runs::QuestRun::new(
                &definition,
                "seasonal-circuit::2026",
                1,
                Some("2026".into()),
            )
            .expect("quest run should be valid"),
        );
        game.record_reward_receipt(RewardReceipt::new(
            "seasonal-circuit::2026",
            "gold-trophy",
            Some(1),
            Some("gold".into()),
            BTreeMap::from([("source".into(), "quest-finalization".into())]),
            game.current_day,
        ))
        .expect("first receipt should be recorded");

        let payload = serde_json::to_string(&game).expect("game should serialize");
        let restored = super::decode_save_payload(&payload).expect("save should load");
        assert_eq!(restored.quest_runs, game.quest_runs);
        assert_eq!(restored.reward_receipts, game.reward_receipts);
    }

    #[test]
    fn old_saves_without_quest_state_load_with_empty_collections() {
        let game = new_game_seeded(dataset_path(), 22);
        let mut payload = serde_json::to_value(&game).expect("game should serialize");
        let root = payload.as_object_mut().expect("state should be an object");
        root.remove("quest_runs");
        root.remove("reward_receipts");

        let restored = super::decode_save_payload(
            &serde_json::to_string(&payload).expect("payload should serialize"),
        )
        .expect("old save should load");
        assert!(restored.quest_runs.is_empty());
        assert!(restored.reward_receipts.is_empty());
    }

    #[test]
    fn reward_receipt_recording_is_idempotency_guarded() {
        let mut game = new_game_seeded(dataset_path(), 23);
        let receipt = RewardReceipt::new("run-1", "reward-1", None, None, BTreeMap::new(), 1);
        game.record_reward_receipt(receipt.clone())
            .expect("first receipt should be accepted");
        let error = game
            .record_reward_receipt(receipt)
            .expect_err("duplicate receipt should be rejected");
        assert!(error.contains("already been recorded"));
        assert_eq!(game.reward_receipts.len(), 1);
    }

    #[test]
    fn event_effects_consume_and_grant_ordered_instances() {
        let mut game = new_game_seeded(dataset_path(), 4);
        game.player.characteristics.insert("budget".into(), 1_000.0);
        super::buy_object_for_sim(&mut game, "gloves").expect("first gloves purchase");
        super::buy_object_for_sim(&mut game, "gloves").expect("second gloves purchase");

        apply_event_effects(&mut game, "consume_object:gloves:2;grant_object:helmet")
            .expect("valid effect group should apply");

        assert_eq!(
            game.player
                .inventory
                .iter()
                .filter(|object| object.definition_id == "gloves")
                .count(),
            0
        );
        assert_eq!(
            game.player
                .inventory
                .iter()
                .filter(|object| object.definition_id == "helmet")
                .count(),
            1
        );
    }

    #[test]
    fn object_transaction_eligibility_reports_policy_and_reason() {
        let mut game = new_game_seeded(dataset_path(), 16);
        game.player
            .characteristics
            .insert("budget".into(), 10_000.0);
        let eligibility = super::object_transaction_eligibility(&game);
        let gloves = eligibility
            .iter()
            .find(|entry| entry.object_id == "gloves")
            .expect("gloves should have transaction metadata");
        assert!(gloves.can_acquire);
        assert!(!gloves.can_sell);
        assert!(gloves.sell_reason.contains("No owned instance"));

        super::buy_object_for_sim(&mut game, "gloves").expect("gloves should be acquirable");
        let updated = super::object_transaction_eligibility(&game);
        let gloves = updated
            .iter()
            .find(|entry| entry.object_id == "gloves")
            .expect("gloves should have updated transaction metadata");
        assert_eq!(gloves.owned_count, 1);
        assert!(gloves.can_sell);
    }

    #[test]
    fn insurance_is_unique_across_policy_types() {
        let mut game = new_game_seeded(dataset_path(), 17);
        game.player
            .characteristics
            .insert("budget".into(), 10_000.0);

        super::buy_object_for_sim(&mut game, "insurance_basic")
            .expect("first insurance policy should be acquirable");
        let error = super::buy_object_for_sim(&mut game, "insurance_full")
            .expect_err("a second insurance policy should be rejected");

        assert!(error.contains("Only one insurance policy"));
    }

    #[test]
    fn insurance_upgrades_are_immediate_but_downgrades_wait_for_year_end() {
        let mut game = new_game_seeded(dataset_path(), 18);
        game.player
            .characteristics
            .insert("budget".into(), 10_000.0);
        super::buy_object_for_sim(&mut game, "insurance_basic")
            .expect("basic insurance should be acquirable");

        super::switch_insurance_for_sim(&mut game, "insurance_full")
            .expect("upgrades should be immediate");
        assert!(game
            .player
            .inventory
            .iter()
            .any(|object| object.definition_id == "insurance_full"));

        let error = super::switch_insurance_for_sim(&mut game, "insurance_third_party")
            .expect_err("downgrades should wait for year end");
        assert!(error.contains("end of the year"));

        game.current_day = game.days_per_year;
        super::switch_insurance_for_sim(&mut game, "insurance_third_party")
            .expect("downgrades should work at year end");
        assert!(game
            .player
            .inventory
            .iter()
            .any(|object| object.definition_id == "insurance_third_party"));
    }

    #[test]
    fn event_effects_reject_invalid_quantities_and_targets() {
        let mut game = new_game_seeded(dataset_path(), 5);
        let before = game.clone();
        assert!(apply_event_effects(&mut game, "grant_object:helmet:-1").is_err());
        assert!(apply_event_effects(&mut game, "consume_object:missing:1").is_err());
        assert!(apply_event_effects(&mut game, "unknown_operation:helmet:1").is_err());
        assert!(
            apply_event_effects(&mut game, "grant_object:helmet;consume_object:missing:1").is_err()
        );
        assert_eq!(
            game.player
                .inventory
                .iter()
                .map(|object| object.instance_id.as_str())
                .collect::<Vec<_>>(),
            before
                .player
                .inventory
                .iter()
                .map(|object| object.instance_id.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn characteristic_set_and_multiply_effects_validate_bounds_atomically() {
        let mut game = new_game_seeded(dataset_path(), 6);
        game.catalog
            .player_characteristics
            .iter_mut()
            .find(|entry| entry.id == "charisma")
            .expect("dataset should define charisma")
            .max_value = 5.0;
        let before = game.player.characteristics["charisma"];

        assert!(apply_event_effects(&mut game, "set_characteristic:charisma:6").is_err());
        assert_eq!(game.player.characteristics["charisma"], before);

        assert!(apply_event_effects(&mut game, "multiply_characteristic:charisma:6").is_err());
        assert_eq!(game.player.characteristics["charisma"], before);
    }

    #[test]
    fn numeric_modifier_facts_and_probability_targets_are_applied() {
        let mut game = new_game_seeded(dataset_path(), 8);
        game.player.characteristics.insert("charisma".into(), 10.0);
        game.catalog.numeric_modifiers.push(NumericModifierData {
            id: "charisma_probability_bonus".into(),
            target: "event_success_probability".into(),
            operation: "add".into(),
            value: "characteristic(\"charisma\") * 0.01".into(),
            priority: 5,
            condition_group: String::new(),
            minimum: None,
            maximum: Some(0.9),
        });

        let probability = apply_numeric_modifier_target(&game, "event_success_probability", 0.3)
            .expect("configured modifier should evaluate");
        assert_eq!(probability, 0.4);
    }

    #[test]
    fn requirement_binding_gates_its_declared_operation_target() {
        let mut game = new_game_seeded(dataset_path(), 9);
        game.catalog.condition_groups.push(ConditionGroupData {
            id: "budget_available".into(),
            operator: "all".into(),
            children: "condition:has_budget".into(),
            source_row: 2,
        });
        game.catalog.conditions.push(ConditionData {
            id: "has_budget".into(),
            group_id: "budget_available".into(),
            subject_type: "characteristic".into(),
            subject_ref: "budget".into(),
            operator: "greater_or_equal".into(),
            value: "0".into(),
            source_row: 2,
        });
        game.catalog
            .requirement_bindings
            .push(RequirementBindingData {
                id: "gloves_affordable".into(),
                operation: "acquire".into(),
                requirement_group: "budget_available".into(),
                target_ref: "gloves".into(),
            });
        validate_requirement_binding(&game, "acquire", "gloves")
            .expect("satisfied requirement should allow acquisition");
        game.player.characteristics.insert("budget".into(), -1.0);
        assert!(validate_requirement_binding(&game, "acquire", "gloves").is_err());
    }

    #[test]
    fn typed_bound_object_service_effect_applies_atomically() {
        let mut game = new_game_seeded(dataset_path(), 10);
        game.player
            .characteristics
            .insert("budget".into(), 10_000.0);
        buy_object_for_sim(&mut game, "gloves").expect("gloves should be acquirable");
        let object_id = game
            .player
            .inventory
            .iter()
            .find(|object| object.definition_id == "gloves")
            .expect("gloves should be owned")
            .definition_id
            .clone();
        game.catalog.effects.push(EffectData {
            id: "service_gloves".into(),
            operation: "set_object_service".into(),
            target: object_id,
            value: "service_1".into(),
            quantity: String::new(),
        });
        game.catalog.effect_bindings.push(EffectBindingData {
            id: "service_gloves_on_event".into(),
            effect_id: "service_gloves".into(),
            trigger_type: "event_completed".into(),
            trigger_ref: "bake_pie".into(),
            reported_result: "success".into(),
            probability: 1.0,
        });

        apply_bound_effects(&mut game, "event_completed", "bake_pie", "success")
            .expect("typed service effect should apply");
        assert!(
            game.player
                .inventory
                .iter()
                .find(|object| object.definition_id == "gloves")
                .expect("gloves should remain owned")
                .service_1_needed
        );
    }

    #[test]
    fn explicit_cost_rule_service_slot_15_does_not_use_first_free_slot() {
        let mut game = new_game_seeded(dataset_path(), 20);
        let definition = game
            .catalog
            .objects
            .iter()
            .find(|object| object.id == "gloves")
            .cloned()
            .expect("dataset should contain gloves");
        let mut object = build_owned_object(&definition, &mut game, false, 0);
        object.cost_15 = "slot_15_service".into();
        game.player.inventory.push(object);

        mark_object_service_needed(&mut game, "gloves_1", "slot_15_service", Some(15))
            .expect("explicit slot 15 should be accepted");

        let object = game
            .player
            .inventory
            .iter()
            .find(|object| object.id == "gloves_1")
            .expect("configured object should remain owned");
        assert!(object.service_15_needed);
        assert!(!object.service_1_needed);
    }

    #[test]
    fn explicit_cost_rule_service_slot_rejects_mismatched_object_cost() {
        let mut game = new_game_seeded(dataset_path(), 25);
        let definition = game
            .catalog
            .objects
            .iter()
            .find(|object| object.id == "gloves")
            .cloned()
            .expect("dataset should contain gloves");
        let mut object = build_owned_object(&definition, &mut game, false, 0);
        object.cost_15 = "configured_cost".into();
        let object_id = object.id.clone();
        game.player.inventory.push(object);

        let error = mark_object_service_needed(&mut game, &object_id, "different_cost", Some(15))
            .expect_err("explicit slot must agree with its configured cost");
        assert!(error.contains("service slot 15"));
        assert!(
            !game
                .player
                .inventory
                .iter()
                .find(|object| object.id == object_id)
                .expect("object should remain owned")
                .service_15_needed
        );
    }

    #[test]
    fn typed_object_effect_quantity_uses_numeric_modifier_breakdown() {
        let mut game = new_game_seeded(dataset_path(), 14);
        game.catalog.effects.push(EffectData {
            id: "grant_gloves".into(),
            operation: "grant_object".into(),
            target: "gloves".into(),
            value: String::new(),
            quantity: "1".into(),
        });
        game.catalog.effect_bindings.push(EffectBindingData {
            id: "grant_gloves_on_event".into(),
            effect_id: "grant_gloves".into(),
            trigger_type: "event_completed".into(),
            trigger_ref: "bake_pie".into(),
            reported_result: "success".into(),
            probability: 1.0,
        });
        game.catalog.numeric_modifiers.push(NumericModifierData {
            id: "extra_gloves".into(),
            target: "effect_quantity:grant_gloves".into(),
            operation: "add".into(),
            value: "2".into(),
            priority: 1,
            condition_group: String::new(),
            minimum: None,
            maximum: None,
        });

        apply_bound_effects(&mut game, "event_completed", "bake_pie", "success")
            .expect("typed grant effect should apply");
        assert_eq!(
            game.player
                .inventory
                .iter()
                .filter(|object| object.definition_id == "gloves")
                .count(),
            3
        );
    }

    #[test]
    fn typed_characteristic_effect_evaluates_shared_facts() {
        let mut game = new_game_seeded(dataset_path(), 15);
        game.player.characteristics.insert("budget".into(), 2_000.0);
        let initial_charisma = game.player.characteristics["charisma"];
        game.catalog.effects.push(EffectData {
            id: "scaled_charisma".into(),
            operation: "add_characteristic".into(),
            target: "charisma".into(),
            value: "characteristic(budget) / 100".into(),
            quantity: String::new(),
        });
        game.catalog.effect_bindings.push(EffectBindingData {
            id: "scaled_charisma_on_event".into(),
            effect_id: "scaled_charisma".into(),
            trigger_type: "event_completed".into(),
            trigger_ref: "bake_pie".into(),
            reported_result: "success".into(),
            probability: 1.0,
        });

        apply_bound_effects(&mut game, "event_completed", "bake_pie", "success")
            .expect("expression-based characteristic effect should apply");
        assert_eq!(
            game.player.characteristics["charisma"],
            initial_charisma + 20.0
        );
    }

    #[test]
    fn lifecycle_bindings_apply_on_acquisition_and_sale() {
        let mut game = new_game_seeded(dataset_path(), 11);
        game.player
            .characteristics
            .insert("budget".into(), 10_000.0);
        game.catalog.effects.extend([
            EffectData {
                id: "acquisition_bonus".into(),
                operation: "add_characteristic".into(),
                target: "budget".into(),
                value: "7".into(),
                quantity: String::new(),
            },
            EffectData {
                id: "sale_bonus".into(),
                operation: "add_characteristic".into(),
                target: "budget".into(),
                value: "11".into(),
                quantity: String::new(),
            },
        ]);
        game.catalog.effect_bindings.extend([
            EffectBindingData {
                id: "on_acquisition".into(),
                effect_id: "acquisition_bonus".into(),
                trigger_type: "object_acquired".into(),
                trigger_ref: "gloves".into(),
                reported_result: String::new(),
                probability: 1.0,
            },
            EffectBindingData {
                id: "on_sale".into(),
                effect_id: "sale_bonus".into(),
                trigger_type: "object_sold".into(),
                trigger_ref: "gloves".into(),
                reported_result: String::new(),
                probability: 1.0,
            },
        ]);

        let before_buy = game.player.characteristics["budget"];
        buy_object_for_sim(&mut game, "gloves").expect("gloves should be acquirable");
        let after_buy = game.player.characteristics["budget"];
        assert!((after_buy - (before_buy - 150.0 + 7.0)).abs() < f64::EPSILON);

        let object_id = game
            .player
            .inventory
            .iter()
            .find(|object| object.definition_id == "gloves")
            .expect("gloves should remain owned")
            .id
            .clone();
        sell_object_for_sim(&mut game, &object_id).expect("gloves should be sellable");
        assert!(game.player.characteristics["budget"] > after_buy + 11.0);
    }

    #[test]
    fn failed_lifecycle_effect_rolls_back_acquisition() {
        let mut game = new_game_seeded(dataset_path(), 13);
        game.player
            .characteristics
            .insert("budget".into(), 10_000.0);
        game.catalog.effects.push(EffectData {
            id: "invalid_acquisition_effect".into(),
            operation: "add_characteristic".into(),
            target: "missing_characteristic".into(),
            value: "1".into(),
            quantity: String::new(),
        });
        game.catalog.effect_bindings.push(EffectBindingData {
            id: "invalid_acquisition_binding".into(),
            effect_id: "invalid_acquisition_effect".into(),
            trigger_type: "object_acquired".into(),
            trigger_ref: "gloves".into(),
            reported_result: String::new(),
            probability: 1.0,
        });
        let before = serde_json::to_value(&game).expect("game should serialize");

        assert!(buy_object_for_sim(&mut game, "gloves").is_err());

        let after = serde_json::to_value(&game).expect("game should serialize");
        assert_eq!(after, before);
    }

    #[test]
    fn daily_lifecycle_binding_applies_after_day_advance() {
        let mut game = new_game_seeded(dataset_path(), 12);
        let initial_charisma = game.player.characteristics["charisma"];
        game.catalog.effects.push(EffectData {
            id: "daily_age_bonus".into(),
            operation: "add_characteristic".into(),
            target: "charisma".into(),
            value: "3".into(),
            quantity: String::new(),
        });
        game.catalog.effect_bindings.push(EffectBindingData {
            id: "on_day".into(),
            effect_id: "daily_age_bonus".into(),
            trigger_type: "day_elapsed".into(),
            trigger_ref: String::new(),
            reported_result: String::new(),
            probability: 1.0,
        });

        advance_one_day(&mut game).expect("day advance should succeed");
        assert_eq!(
            game.player.characteristics["charisma"],
            initial_charisma + 3.0
        );
    }

    #[test]
    fn day_advance_rolls_back_late_effect_validation_failure() {
        let mut game = new_game_seeded(dataset_path(), 18);
        game.catalog.effect_bindings.push(EffectBindingData {
            id: "missing_daily_effect_binding".into(),
            effect_id: "missing_daily_effect".into(),
            trigger_type: "day_elapsed".into(),
            trigger_ref: String::new(),
            reported_result: String::new(),
            probability: 1.0,
        });
        let before = serde_json::to_value(&game).expect("game state should serialize");

        let error = advance_day(&mut game).expect_err("missing effect should fail late");

        assert!(error.contains("references unknown effect"));
        assert_eq!(
            serde_json::to_value(&game).expect("game state should serialize"),
            before
        );
    }

    #[test]
    fn day_advance_rolls_back_late_effect_application_failure() {
        let mut game = new_game_seeded(dataset_path(), 19);
        let initial_charisma = game.player.characteristics["charisma"];
        game.catalog.effects.push(EffectData {
            id: "invalid_daily_effect".into(),
            operation: "add_characteristic".into(),
            target: "missing_characteristic".into(),
            value: "1".into(),
            quantity: String::new(),
        });
        game.catalog.effect_bindings.push(EffectBindingData {
            id: "invalid_daily_effect_binding".into(),
            effect_id: "invalid_daily_effect".into(),
            trigger_type: "day_elapsed".into(),
            trigger_ref: String::new(),
            reported_result: String::new(),
            probability: 1.0,
        });
        let before = serde_json::to_value(&game).expect("game state should serialize");

        let error = advance_day(&mut game).expect_err("invalid effect should fail late");

        assert!(error.contains("unknown characteristic"));
        assert_eq!(game.player.characteristics["charisma"], initial_charisma);
        assert_eq!(
            serde_json::to_value(&game).expect("game state should serialize"),
            before
        );
    }

    #[test]
    fn daily_processing_does_not_create_recovery_state_without_role() {
        let mut game = new_game_seeded(dataset_path(), 17);
        game.catalog.resource_roles.recovery = None;
        game.player.sickness_start_day = None;
        let stamina_before = game.player.characteristics.get("stamina").copied();

        advance_one_day(&mut game).expect("day advance should succeed");

        assert_eq!(game.player.sickness_start_day, None);
        assert_eq!(
            game.player.characteristics.get("stamina").copied(),
            stamina_before
        );
    }

    #[test]
    fn configured_recovery_status_replays_with_custom_resource_and_expiry() {
        let mut game = new_game_seeded(dataset_path(), 26);
        game.catalog.resource_roles.recovery = Some("energy".into());
        game.catalog
            .player_characteristics
            .push(crate::engine::loader::PlayerCharacteristicData {
                id: "energy".into(),
                name: "Energy".into(),
                value: 100.0,
                min_value: 0.0,
                max_value: 100.0,
            });
        game.player.characteristics.remove("stamina");
        game.player.characteristics.insert("energy".into(), 70.0);
        game.catalog
            .labels
            .values
            .insert("daily_stamina_recovery".into(), "0".into());
        game.catalog
            .labels
            .values
            .insert("sickness_daily_probability".into(), "1".into());
        game.catalog
            .labels
            .values
            .insert("sickness_initial_stamina".into(), "10".into());
        game.catalog
            .labels
            .values
            .insert("sickness_recovery_stamina".into(), "50".into());
        game.catalog
            .labels
            .values
            .insert("sickness_final_recovery".into(), "50".into());

        advance_one_day(&mut game).expect("configured status should start");
        assert_eq!(game.player.sickness_start_day, Some(2));
        assert_eq!(game.player.characteristics["energy"], 10.0);

        let saved = serde_json::to_string(&game).expect("state should serialize");
        let mut replayed = super::decode_save_payload(&saved).expect("save should load");
        advance_one_day(&mut game).expect("status should progress");
        advance_one_day(&mut replayed).expect("restored status should progress");
        assert_eq!(
            game.player.sickness_start_day,
            replayed.player.sickness_start_day
        );
        assert_eq!(
            game.player.characteristics["energy"],
            replayed.player.characteristics["energy"]
        );
    }

    #[test]
    fn recovery_role_without_recovery_rules_is_inert() {
        let mut game = new_game_seeded(dataset_path(), 27);
        game.catalog.resource_roles.recovery = Some("energy".into());
        game.catalog
            .player_characteristics
            .push(crate::engine::loader::PlayerCharacteristicData {
                id: "energy".into(),
                name: "Energy".into(),
                value: 100.0,
                min_value: 0.0,
                max_value: 100.0,
            });
        game.player.characteristics.remove("stamina");
        game.player.characteristics.insert("energy".into(), 70.0);
        for key in [
            "nightly_stamina_recovery",
            "daily_stamina_recovery",
            "weekend_stamina_recovery",
            "sickness_daily_probability",
        ] {
            game.catalog.labels.values.remove(key);
        }

        advance_one_day(&mut game).expect("day advance should succeed");

        assert_eq!(game.player.sickness_start_day, None);
        assert_eq!(game.player.characteristics["energy"], 70.0);
    }

    #[test]
    fn race_services_only_use_costs_configured_on_the_entered_vehicle() {
        let mut game = new_game_seeded(dataset_path(), 6);
        let fiesta_definition = game
            .catalog
            .objects
            .iter()
            .find(|object| object.id == "ford_fiesta")
            .cloned()
            .expect("dataset should contain the Ford Fiesta");
        let fiesta = build_owned_object(&fiesta_definition, &mut game, true, 0);
        let fiesta_id = fiesta.id.clone();
        let configured_costs = [
            fiesta.cost_1.clone(),
            fiesta.cost_2.clone(),
            fiesta.cost_3.clone(),
            fiesta.cost_4.clone(),
            fiesta.cost_5.clone(),
            fiesta.cost_6.clone(),
            fiesta.cost_7.clone(),
            fiesta.cost_8.clone(),
            fiesta.cost_9.clone(),
            fiesta.cost_10.clone(),
            fiesta.cost_11.clone(),
            fiesta.cost_12.clone(),
            fiesta.cost_13.clone(),
            fiesta.cost_14.clone(),
            fiesta.cost_15.clone(),
        ];
        game.player.inventory.push(fiesta);
        let race = game
            .catalog
            .events
            .iter()
            .find(|event| {
                !event.quest_id.trim().is_empty()
                    && event
                    .tags
                    .split(';')
                    .any(|tag| tag.trim().eq_ignore_ascii_case("race"))
            })
            .cloned()
            .expect("dataset should contain a race event");

        mark_event_services_needed(&mut game, &fiesta_id, &race);

        let fiesta = game
            .player
            .inventory
            .iter()
            .find(|object| object.id == fiesta_id)
            .expect("loaned Ford Fiesta should remain in inventory");
        let needed_costs = [
            (fiesta.service_1_needed, &fiesta.cost_1),
            (fiesta.service_2_needed, &fiesta.cost_2),
            (fiesta.service_3_needed, &fiesta.cost_3),
            (fiesta.service_4_needed, &fiesta.cost_4),
            (fiesta.service_5_needed, &fiesta.cost_5),
            (fiesta.service_6_needed, &fiesta.cost_6),
            (fiesta.service_7_needed, &fiesta.cost_7),
            (fiesta.service_8_needed, &fiesta.cost_8),
            (fiesta.service_9_needed, &fiesta.cost_9),
            (fiesta.service_10_needed, &fiesta.cost_10),
            (fiesta.service_11_needed, &fiesta.cost_11),
            (fiesta.service_12_needed, &fiesta.cost_12),
            (fiesta.service_13_needed, &fiesta.cost_13),
            (fiesta.service_14_needed, &fiesta.cost_14),
            (fiesta.service_15_needed, &fiesta.cost_15),
        ];
        assert!(needed_costs
            .iter()
            .filter(|(needed, _)| *needed)
            .all(|(_, cost_id)| configured_costs
                .iter()
                .any(|configured| configured == *cost_id)));
        assert!(needed_costs
            .iter()
            .filter(|(needed, _)| *needed)
            .all(|(_, cost_id)| !cost_id.starts_with("car_caterham_")));
    }

    #[test]
    fn unentered_races_are_recorded_as_dnf_when_the_day_ends() {
        let mut game = new_game_seeded(dataset_path(), 12);
        let race = game
            .catalog
            .events
            .iter()
            .find(|event| {
                !event.quest_id.trim().is_empty()
                    && event
                    .tags
                    .split(';')
                    .any(|tag| tag.trim().eq_ignore_ascii_case("race"))
            })
            .cloned()
            .expect("dataset should contain a race");
        let quest_id = race.quest_id.clone();
        game.quest_memberships.push(super::QuestMembership {
            quest_id,
            joined_day: race.day_of_year.saturating_sub(1),
        });
        game.current_day = race.day_of_year;
        game.pending_alerts.clear();

        advance_one_day(&mut game).expect("day advance should succeed");

        let history = game
            .event_history
            .iter()
            .find(|entry| entry.event_id == race.id && entry.entered_day == race.day_of_year)
            .expect("missed race should be recorded");
        assert_eq!(history.result, "DNF");
        assert!(game.pending_alerts.iter().any(|alert| {
            alert.title == "Race missed" && alert.message.contains(&race.name)
        }));
    }

    #[test]
    fn unentered_races_outside_joined_championships_are_not_recorded() {
        let mut game = new_game_seeded(dataset_path(), 13);
        let race = game
            .catalog
            .events
            .iter()
            .find(|event| {
                event.quest_id.is_empty()
                    && event
                        .tags
                        .split(';')
                        .any(|tag| tag.trim().eq_ignore_ascii_case("race"))
            })
            .cloned();
        let Some(race) = race else {
            return;
        };
        game.current_day = race.day_of_year;
        game.pending_alerts.clear();

        advance_one_day(&mut game).expect("day advance should succeed");

        assert!(!game
            .event_history
            .iter()
            .any(|entry| entry.event_id == race.id && entry.entered_day == race.day_of_year));
    }
}
