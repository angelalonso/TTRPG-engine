//! Reinforcement-learning environment primitives built on the headless engine.
//!
//! This module deliberately contains no neural-network or Python dependencies.
//! It provides a deterministic, inspectable contract that an external trainer
//! can consume.

use crate::headless::HeadlessGame;
use crate::{
    characteristic_value, eligible_event_entries, legal_encounter_action_ids, legal_event_ids,
    object_transaction_eligibility, run_status, GameState, RunStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

pub const OBSERVATION_SCHEMA_VERSION: u32 = 1;
pub const ACTION_CATALOGUE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Wait,
    StartActivity {
        event_id: String,
    },
    EnterEvent {
        event_id: String,
        object_id: String,
    },
    BuyObject {
        object_id: String,
    },
    SellObject {
        object_id: String,
    },
    JoinQuest {
        quest_id: String,
    },
    ResolveEncounter {
        action_id: String,
    },
    ResolvePendingEvent {
        event_id: String,
        result: String,
        position: Option<u32>,
    },
}

impl Action {
    pub fn key(&self) -> String {
        match self {
            Self::Wait => "wait".into(),
            Self::StartActivity { event_id } => format!("start_activity:{event_id}"),
            Self::EnterEvent {
                event_id,
                object_id,
            } => format!("enter_event:{event_id}:{object_id}"),
            Self::BuyObject { object_id } => format!("buy_object:{object_id}"),
            Self::SellObject { object_id } => format!("sell_object:{object_id}"),
            Self::JoinQuest { quest_id } => format!("join_quest:{quest_id}"),
            Self::ResolveEncounter { action_id } => format!("resolve_encounter:{action_id}"),
            Self::ResolvePendingEvent {
                event_id,
                result,
                position,
            } => format!(
                "resolve_pending_event:{event_id}:{result}:{}",
                position.map_or_else(|| "none".into(), |value| value.to_string())
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionDefinition {
    pub id: u32,
    pub key: String,
    pub action: Action,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Observation {
    pub schema_version: u32,
    pub current_day: u32,
    pub days_per_year: u32,
    pub characteristics: BTreeMap<String, f64>,
    pub inventory_ids: Vec<String>,
    pub quest_ids: Vec<String>,
    pub active_event_ids: Vec<String>,
    pub pending_event_ids: Vec<String>,
    pub event_history_count: usize,
    pub reward_receipt_count: usize,
    pub legal_action_mask: Vec<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EpisodeStatus {
    Ongoing,
    GoalReached,
    DeadMoney,
    DeadStamina,
    Dead,
    MaxDays,
    MaxSteps,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepInfo {
    pub action_key: String,
    pub days_elapsed: u32,
    pub status: EpisodeStatus,
    pub budget: f64,
    pub stamina: f64,
    pub goal_progress: f64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepResult {
    pub observation: Observation,
    pub reward: f32,
    pub terminated: bool,
    pub truncated: bool,
    pub info: StepInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RewardConfig {
    pub goal: f32,
    pub death: f32,
    pub day: f32,
    pub invalid_action: f32,
    pub progress_scale: f32,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            goal: 1_000.0,
            death: -1_000.0,
            day: -1.0,
            invalid_action: -10.0,
            progress_scale: 5.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RlConfig {
    pub goal: GoalSpec,
    pub max_days: u32,
    pub max_steps: u32,
    pub reward: RewardConfig,
}

impl Default for RlConfig {
    fn default() -> Self {
        Self {
            goal: GoalSpec::Characteristic {
                id: "charisma".into(),
                value: 100.0,
            },
            max_days: 7_300,
            max_steps: 14_600,
            reward: RewardConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GoalSpec {
    Characteristic { id: String, value: f64 },
    Championship { id: String },
    Championships { level: u32, count: usize },
}

impl GoalSpec {
    pub fn parse(value: &str) -> Self {
        if let Some(id) = value.strip_prefix("championship:") {
            return Self::Championship {
                id: id.trim().into(),
            };
        }
        if let Some(spec) = value
            .strip_prefix("championships:")
            .or_else(|| value.strip_prefix("trophy:"))
        {
            let mut level = 1;
            let mut count = 1;
            for part in spec.split(',') {
                if let Some((key, value)) = part.split_once('=') {
                    match key.trim() {
                        "level" => level = value.trim().parse().unwrap_or(1),
                        "count" => count = value.trim().parse().unwrap_or(1),
                        _ => {}
                    }
                }
            }
            return Self::Championships { level, count };
        }
        let (id, target) = value
            .split_once(">=")
            .map(|(id, target)| (id.trim(), target.trim().parse().unwrap_or(100.0)))
            .unwrap_or((value.trim(), 100.0));
        Self::Characteristic {
            id: id.into(),
            value: target,
        }
    }

    pub fn reached(&self, game: &GameState) -> bool {
        match self {
            Self::Characteristic { id, value } => characteristic_value(game, id) >= *value,
            Self::Championship { id } => game.player.inventory.iter().any(|object| {
                object.object_type == "achievements"
                    && (object.trophy_championship == *id || object.id.contains(id))
            }),
            Self::Championships { level, count } => {
                game.player
                    .inventory
                    .iter()
                    .filter(|object| {
                        object.object_type == "achievements" && object.trophy_level == *level
                    })
                    .count()
                    >= *count
            }
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::Characteristic { id, value } => format!("{id}>={value}"),
            Self::Championship { id } => format!("championship:{id}"),
            Self::Championships { level, count } => {
                format!("championships:level={level},count={count}")
            }
        }
    }

    pub fn prioritizes_championships(&self) -> bool {
        matches!(self, Self::Championship { .. } | Self::Championships { .. })
    }
}

#[derive(Debug, Clone)]
pub struct RlEnvironment {
    game: HeadlessGame,
    config: RlConfig,
    actions: Vec<ActionDefinition>,
    steps: u32,
    previous_progress: f64,
}

impl RlEnvironment {
    pub fn new(dataset_path: impl Into<String>, seed: u64, config: RlConfig) -> Self {
        let game = HeadlessGame::new(dataset_path, seed);
        let previous_progress = goal_progress(game.state(), &config);
        let actions = build_action_catalogue(game.state());
        Self {
            game,
            config,
            actions,
            steps: 0,
            previous_progress,
        }
    }

    pub fn state(&self) -> &GameState {
        self.game.state()
    }

    pub fn config(&self) -> &RlConfig {
        &self.config
    }

    pub fn actions(&self) -> &[ActionDefinition] {
        &self.actions
    }

    pub fn legal_action_mask(&self) -> Vec<bool> {
        if !matches!(self.status(), EpisodeStatus::Ongoing) {
            return vec![false; self.actions.len()];
        }
        legal_action_mask(&self.game, &self.actions)
    }

    pub fn observation(&self) -> Observation {
        observation(self.game.state(), self.legal_action_mask())
    }

    pub fn status(&self) -> EpisodeStatus {
        status(self.game.state(), &self.config, self.steps)
    }

    pub fn step(&mut self, action_id: u32) -> Result<StepResult, String> {
        let definition = self
            .actions
            .get(action_id as usize)
            .cloned()
            .ok_or_else(|| format!("RL action id {action_id} is outside the action catalogue"))?;
        let before_progress = self.previous_progress;
        let before_day = self.game.state().current_day;
        let action_key = definition.key.clone();
        let action_result = apply_action(&mut self.game, &definition.action);
        self.steps = self.steps.saturating_add(1);
        let after_progress = goal_progress(self.game.state(), &self.config);
        self.previous_progress = after_progress;
        let current_status = status(self.game.state(), &self.config, self.steps);
        let invalid = action_result.as_ref().err().cloned();
        let days_elapsed = self.game.state().current_day.saturating_sub(before_day);
        let mut reward = (after_progress - before_progress) as f32
            * self.config.reward.progress_scale
            + days_elapsed as f32 * self.config.reward.day;
        if invalid.is_some() {
            reward += self.config.reward.invalid_action;
        }
        if matches!(current_status, EpisodeStatus::GoalReached) {
            reward += self.config.reward.goal;
        } else if matches!(
            current_status,
            EpisodeStatus::Dead | EpisodeStatus::DeadMoney | EpisodeStatus::DeadStamina
        ) {
            reward += self.config.reward.death;
        }
        let info = StepInfo {
            action_key,
            days_elapsed,
            status: current_status.clone(),
            budget: characteristic_value(self.game.state(), "budget"),
            stamina: characteristic_value(self.game.state(), "stamina"),
            goal_progress: after_progress,
            error: invalid,
        };
        let terminated = matches!(
            current_status,
            EpisodeStatus::GoalReached
                | EpisodeStatus::Dead
                | EpisodeStatus::DeadMoney
                | EpisodeStatus::DeadStamina
        );
        let truncated = matches!(
            current_status,
            EpisodeStatus::MaxDays | EpisodeStatus::MaxSteps
        );
        Ok(StepResult {
            observation: self.observation(),
            reward,
            terminated,
            truncated,
            info,
        })
    }
}

fn build_action_catalogue(game: &GameState) -> Vec<ActionDefinition> {
    let mut actions = vec![Action::Wait];
    actions.extend(
        game.catalog
            .events
            .iter()
            .filter(|event| event.day_of_year == 0)
            .map(|event| Action::StartActivity {
                event_id: event.id.clone(),
            }),
    );
    for event in &game.catalog.events {
        actions.push(Action::EnterEvent {
            event_id: event.id.clone(),
            object_id: String::new(),
        });
        for object in &game.catalog.objects {
            actions.push(Action::EnterEvent {
                event_id: event.id.clone(),
                object_id: object.id.clone(),
            });
        }
    }
    actions.extend(game.catalog.objects.iter().map(|object| Action::BuyObject {
        object_id: object.id.clone(),
    }));
    actions.extend(
        game.catalog
            .objects
            .iter()
            .map(|object| Action::SellObject {
                object_id: object.id.clone(),
            }),
    );
    actions.extend(game.catalog.quests.iter().map(|quest| Action::JoinQuest {
        quest_id: quest.id.clone(),
    }));
    actions.extend(
        game.catalog
            .encounter_actions
            .iter()
            .map(|action| Action::ResolveEncounter {
                action_id: action.action_id.clone(),
            }),
    );
    for event in &game.catalog.events {
        for result in ["success", "failure"] {
            actions.push(Action::ResolvePendingEvent {
                event_id: event.id.clone(),
                result: result.into(),
                position: None,
            });
            for position in 1..=10 {
                actions.push(Action::ResolvePendingEvent {
                    event_id: event.id.clone(),
                    result: result.into(),
                    position: Some(position),
                });
            }
        }
    }
    actions
        .into_iter()
        .enumerate()
        .map(|(id, action)| ActionDefinition {
            id: id as u32,
            key: action.key(),
            action,
        })
        .collect()
}

fn legal_action_mask(game: &HeadlessGame, actions: &[ActionDefinition]) -> Vec<bool> {
    let legal_started_events = legal_event_ids(game.state())
        .into_iter()
        .collect::<HashSet<_>>();
    let legal_entries = eligible_event_entries(game.state());
    let legal_entry_keys = legal_entries
        .iter()
        .map(|(event_id, object_id)| format!("{event_id}\u{1f}{object_id}"))
        .collect::<HashSet<_>>();
    let legal_entry_definitions = legal_entries
        .iter()
        .filter_map(|(event_id, object_id)| {
            if object_id.is_empty() {
                return None;
            }
            game.state()
                .player
                .inventory
                .iter()
                .find(|owned| owned.id == *object_id)
                .and_then(|owned| {
                    game.state()
                        .catalog
                        .objects
                        .iter()
                        .find(|definition| {
                            owned.id == definition.id
                                || owned.id.starts_with(&format!("{}_", definition.id))
                        })
                        .map(|definition| format!("{event_id}\u{1f}{}", definition.id))
                })
        })
        .collect::<HashSet<_>>();
    let transactions = object_transaction_eligibility(game.state());
    let can_buy = transactions
        .iter()
        .filter(|transaction| transaction.can_acquire)
        .map(|transaction| transaction.object_id.as_str())
        .collect::<HashSet<_>>();
    let can_sell = transactions
        .iter()
        .filter(|transaction| transaction.can_sell)
        .map(|transaction| transaction.object_id.as_str())
        .collect::<HashSet<_>>();
    let legal_encounters = legal_encounter_action_ids(game.state())
        .into_iter()
        .collect::<HashSet<_>>();
    let pending_events = game
        .state()
        .pending_events
        .iter()
        .map(|event| event.event_id.as_str())
        .collect::<HashSet<_>>();

    actions
        .iter()
        .map(|definition| match &definition.action {
            Action::Wait => true,
            Action::StartActivity { event_id } => legal_started_events.contains(event_id),
            Action::EnterEvent {
                event_id,
                object_id,
            } => {
                let key = format!("{event_id}\u{1f}{object_id}");
                object_id.is_empty()
                    && legal_entry_keys.contains(&key)
                    || !object_id.is_empty() && legal_entry_definitions.contains(&key)
            }
            Action::BuyObject { object_id } => can_buy.contains(object_id.as_str()),
            Action::SellObject { object_id } => can_sell.contains(object_id.as_str()),
            Action::JoinQuest { quest_id: _ } => {
                let mut clone = game.clone();
                apply_action(&mut clone, &definition.action).is_ok()
            }
            Action::ResolveEncounter { action_id } => legal_encounters.contains(action_id),
            Action::ResolvePendingEvent { event_id, .. } => {
                pending_events.contains(event_id.as_str())
            }
        })
        .collect()
}

fn apply_action(game: &mut HeadlessGame, action: &Action) -> Result<(), String> {
    match action {
        Action::Wait => game.advance_day(),
        Action::StartActivity { event_id } => game.apply_action(event_id).map(|_| ()),
        Action::EnterEvent {
            event_id,
            object_id,
        } => {
            if object_id.is_empty() {
                game.enter_event(event_id, "").map(|_| ())
            } else {
                let actual_object_id = game
                    .state()
                    .player
                    .inventory
                    .iter()
                    .find(|object| {
                        object.id == *object_id || object.id.starts_with(&format!("{object_id}_"))
                    })
                    .map(|object| object.id.clone())
                    .ok_or_else(|| format!("No owned object matches '{object_id}'"))?;
                game.enter_event(event_id, &actual_object_id)
            }
        }
        Action::BuyObject { object_id } => game.buy_object(object_id),
        Action::SellObject { object_id } => {
            let owned = game
                .state()
                .player
                .inventory
                .iter()
                .find(|object| {
                    object.id == *object_id || object.id.starts_with(&format!("{object_id}_"))
                })
                .map(|object| object.id.clone())
                .ok_or_else(|| format!("No owned object matches '{object_id}'"))?;
            game.sell_object(&owned)
        }
        Action::JoinQuest { quest_id } => game.join_quest(quest_id),
        Action::ResolveEncounter { action_id } => {
            game.resolve_encounter(Some(action_id)).map(|_| ())
        }
        Action::ResolvePendingEvent {
            event_id,
            result,
            position,
        } => {
            let entry_id = game
                .state()
                .pending_events
                .iter()
                .find(|event| event.event_id == *event_id)
                .map(|event| event.id.clone())
                .ok_or_else(|| format!("No pending event matches '{event_id}'"))?;
            game.submit_event(&entry_id, result, *position).map(|_| ())
        }
    }
}

fn goal_progress(game: &GameState, config: &RlConfig) -> f64 {
    match &config.goal {
        GoalSpec::Characteristic { id, value } => {
            if *value <= 0.0 {
                return 1.0;
            }
            (characteristic_value(game, id) / value).clamp(0.0, 1.0)
        }
        GoalSpec::Championship { id } => {
            if config.goal.reached(game) {
                1.0
            } else {
                (game
                    .player
                    .inventory
                    .iter()
                    .filter(|object| {
                        object.object_type == "achievements"
                            && (object.trophy_championship == *id || object.id.contains(id))
                    })
                    .count() as f64)
                    .min(1.0)
            }
        }
        GoalSpec::Championships { level, count } => {
            if *count == 0 {
                return 1.0;
            }
            let completed = game
                .player
                .inventory
                .iter()
                .filter(|object| {
                    object.object_type == "achievements" && object.trophy_level == *level
                })
                .count();
            (completed as f64 / *count as f64).clamp(0.0, 1.0)
        }
    }
}

fn status(game: &GameState, config: &RlConfig, steps: u32) -> EpisodeStatus {
    if game.player.dead {
        return EpisodeStatus::Dead;
    }
    if config.goal.reached(game) {
        return EpisodeStatus::GoalReached;
    }
    match run_status(
        game,
        match &config.goal {
            GoalSpec::Characteristic { id, .. } => id,
            GoalSpec::Championship { .. } | GoalSpec::Championships { .. } => "",
        },
        match &config.goal {
            GoalSpec::Characteristic { value, .. } => *value,
            GoalSpec::Championship { .. } | GoalSpec::Championships { .. } => f64::MAX,
        },
        config.max_days,
    ) {
        RunStatus::DeadMoney => EpisodeStatus::DeadMoney,
        RunStatus::DeadStamina => EpisodeStatus::DeadStamina,
        RunStatus::Dead => EpisodeStatus::Dead,
        RunStatus::MaxDays => EpisodeStatus::MaxDays,
        RunStatus::GoalReached => EpisodeStatus::GoalReached,
        RunStatus::Ongoing if steps >= config.max_steps => EpisodeStatus::MaxSteps,
        RunStatus::Ongoing => EpisodeStatus::Ongoing,
    }
}

fn observation(game: &GameState, legal_action_mask: Vec<bool>) -> Observation {
    let mut characteristics = BTreeMap::new();
    for definition in &game.catalog.player_characteristics {
        characteristics.insert(
            definition.id.clone(),
            game.player
                .characteristics
                .get(&definition.id)
                .copied()
                .unwrap_or(definition.value),
        );
    }
    let mut inventory_ids = game
        .player
        .inventory
        .iter()
        .map(|object| object.id.clone())
        .collect::<Vec<_>>();
    inventory_ids.sort();
    let mut quest_ids = game
        .quest_memberships
        .iter()
        .map(|membership| membership.quest_id.clone())
        .collect::<Vec<_>>();
    quest_ids.sort();
    let mut active_event_ids = game
        .player
        .active_events
        .iter()
        .map(|event| event.event_id.clone())
        .collect::<Vec<_>>();
    active_event_ids.sort();
    let mut pending_event_ids = game
        .pending_events
        .iter()
        .map(|event| event.event_id.clone())
        .collect::<Vec<_>>();
    pending_event_ids.sort();
    Observation {
        schema_version: OBSERVATION_SCHEMA_VERSION,
        current_day: game.current_day,
        days_per_year: game.days_per_year,
        characteristics,
        inventory_ids,
        quest_ids,
        active_event_ids,
        pending_event_ids,
        event_history_count: game.event_history.len(),
        reward_receipt_count: game.reward_receipts.len(),
        legal_action_mask,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Action, EpisodeStatus, GoalSpec, RlConfig, RlEnvironment, ACTION_CATALOGUE_VERSION,
    };

    fn dataset_path() -> &'static str {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../gtr2career")
    }

    #[test]
    fn action_catalogue_has_stable_ids_and_wait() {
        let environment = RlEnvironment::new(dataset_path(), 42, RlConfig::default());
        assert_eq!(ACTION_CATALOGUE_VERSION, 1);
        assert_eq!(environment.actions()[0].id, 0);
        assert_eq!(environment.actions()[0].action, Action::Wait);
        assert_eq!(environment.actions()[0].key, "wait");
    }

    #[test]
    fn seeded_observations_are_reproducible() {
        let left = RlEnvironment::new(dataset_path(), 42, RlConfig::default());
        let right = RlEnvironment::new(dataset_path(), 42, RlConfig::default());
        assert_eq!(left.observation(), right.observation());
        assert_eq!(left.legal_action_mask(), right.legal_action_mask());
    }

    #[test]
    fn wait_is_legal_at_start_and_advances_time() {
        let mut environment = RlEnvironment::new(dataset_path(), 42, RlConfig::default());
        assert!(environment.legal_action_mask()[0]);
        let before = environment.state().current_day;
        let result = environment.step(0).expect("wait should be a valid action");
        assert_eq!(environment.state().current_day, before + 1);
        assert!(!result.terminated);
        assert_eq!(result.info.status, EpisodeStatus::Ongoing);
    }

    #[test]
    fn parses_supported_goal_forms() {
        assert_eq!(
            GoalSpec::parse("charisma>=100"),
            GoalSpec::Characteristic {
                id: "charisma".into(),
                value: 100.0
            }
        );
        assert_eq!(
            GoalSpec::parse("championship:honda_civic_fm_national"),
            GoalSpec::Championship {
                id: "honda_civic_fm_national".into()
            }
        );
        assert_eq!(
            GoalSpec::parse("trophy:level=1,count=2"),
            GoalSpec::Championships { level: 1, count: 2 }
        );
    }
}
