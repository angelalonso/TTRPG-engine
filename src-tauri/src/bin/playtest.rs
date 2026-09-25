use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ttrpg_engine_lib::{
    advance_day, apply_event, apply_override, buy_object_for_sim, eligible_event_entries,
    enter_event_for_sim, join_quest_for_sim, legal_encounter_action_ids, legal_event_ids,
    new_game_seeded, quit_event_for_sim, resolve_encounter_for_sim, roll, run_status,
    service_object_for_sim, submit_event_for_sim_with_details, GameState, RunStatus, ServiceType,
};

#[derive(Clone, Copy)]
enum StrategyKind {
    Random,
    Greedy,
    GoalAware,
    Required,
}

#[derive(Clone)]
struct OutcomeRule {
    selector: String,
    fixed_rank: Option<u32>,
}

#[derive(Clone, Copy, Debug)]
enum Decision {
    Action(usize),
    Event(usize),
    Purchase(usize),
    JoinQuest(usize),
    QuitJob(usize),
    Wait,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum GoalSpec {
    Characteristic { id: String, value: f64 },
    Championship { id: String },
    Championships { level: u32, count: usize },
}

impl GoalSpec {
    fn parse(value: &str) -> Self {
        if let Some(id) = value.strip_prefix("championship:") {
            return Self::Championship {
                id: id.trim().into(),
            };
        }
        if let Some(spec) = value.strip_prefix("championships:") {
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

    fn reached(&self, game: &GameState) -> bool {
        match self {
            Self::Characteristic { id, value } => {
                game.player.characteristics.get(id).copied().unwrap_or(0.0) >= *value
            }
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

    fn description(&self) -> String {
        match self {
            Self::Characteristic { id, value } => format!("{id}>={value}"),
            Self::Championship { id } => format!("championship:{id}"),
            Self::Championships { level, count } => {
                format!("championships:level={level},count={count}")
            }
        }
    }

    fn prioritizes_championships(&self) -> bool {
        matches!(self, Self::Championship { .. } | Self::Championships { .. })
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
struct FileConfig {
    dataset: Option<String>,
    seed: Option<u64>,
    runs: Option<u32>,
    max_days: Option<u32>,
    max_turns: Option<u32>,
    strategy: Option<String>,
    goal: Option<String>,
    verbosity: Option<String>,
    deep_trace: Option<bool>,
    min_stamina: Option<f64>,
    fake_results: Option<bool>,
    speed: Option<String>,
    pace_ms: Option<u64>,
    overrides: Option<Vec<String>>,
    outcomes: Option<Vec<String>>,
    output: Option<String>,
    log: Option<String>,
    logs: Option<Vec<String>>,
    unique_paths: Option<bool>,
    too_easy_below_days: Option<u32>,
    hard_above_days: Option<u32>,
    near_impossible_above_days: Option<u32>,
}

trait Strategy {
    fn choose(
        &mut self,
        game: &mut GameState,
        actions: &[String],
        events: &[(String, String)],
        purchases: &[String],
        quests: &[String],
        active_jobs: &[String],
    ) -> Decision;
}

struct RandomStrategy;
struct GreedyStrategy;
struct GoalAwareStrategy {
    goal: GoalSpec,
    min_stamina: f64,
}
struct RequiredStrategy;

impl Strategy for RandomStrategy {
    fn choose(
        &mut self,
        game: &mut GameState,
        actions: &[String],
        events: &[(String, String)],
        purchases: &[String],
        quests: &[String],
        _active_jobs: &[String],
    ) -> Decision {
        let total = actions.len() + events.len() + purchases.len() + quests.len();
        if total == 0 {
            return Decision::Wait;
        }
        let pick = (roll(game) * total as f64).floor() as usize % total;
        if pick < events.len() {
            Decision::Event(pick)
        } else if pick < events.len() + actions.len() {
            Decision::Action(pick - events.len())
        } else if pick < events.len() + actions.len() + purchases.len() {
            Decision::Purchase(pick - events.len() - actions.len())
        } else {
            Decision::JoinQuest(pick - events.len() - actions.len() - purchases.len())
        }
    }
}

impl Strategy for GreedyStrategy {
    fn choose(
        &mut self,
        game: &mut GameState,
        actions: &[String],
        events: &[(String, String)],
        purchases: &[String],
        quests: &[String],
        _active_jobs: &[String],
    ) -> Decision {
        if !events.is_empty() {
            return Decision::Event(0);
        }
        if !purchases.is_empty() {
            return Decision::Purchase(0);
        }
        if let Some((index, _)) = actions.iter().enumerate().max_by(|(_, left), (_, right)| {
            action_value(game, left)
                .partial_cmp(&action_value(game, right))
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            return Decision::Action(index);
        }
        quests
            .first()
            .map(|_| Decision::JoinQuest(0))
            .unwrap_or(Decision::Wait)
    }
}

impl Strategy for GoalAwareStrategy {
    fn choose(
        &mut self,
        game: &mut GameState,
        actions: &[String],
        events: &[(String, String)],
        purchases: &[String],
        quests: &[String],
        active_jobs: &[String],
    ) -> Decision {
        if self.goal.prioritizes_championships() {
            if let Some(index) = actions.iter().position(|id| {
                id == "act_personal_loan"
                    && metric(game, "budget") < championship_funding_needed(game, &self.goal)
            }) {
                return Decision::Action(index);
            }
            if active_jobs.is_empty()
                && metric(game, "budget") < championship_funding_needed(game, &self.goal)
            {
                if let Some((index, _)) = actions
                    .iter()
                    .enumerate()
                    .filter(|(_, id)| {
                        game.catalog
                            .events
                            .iter()
                            .find(|event| event.id == **id)
                            .is_some_and(|event| {
                                event.event_type.eq_ignore_ascii_case("work")
                                    && (event.payout_freq_type.eq_ignore_ascii_case("recurring")
                                        || game
                                            .catalog
                                            .obligations
                                            .iter()
                                            .any(|obligation| obligation.event_id == event.id))
                            })
                    })
                    .max_by(|(_, left), (_, right)| {
                        goal_action_value(game, left, Some(&self.goal))
                            .partial_cmp(&goal_action_value(game, right, Some(&self.goal)))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                {
                    return Decision::Action(index);
                }
            }
        }
        if should_quit_job_for_goal_race(game, &self.goal, self.min_stamina, active_jobs) {
            return Decision::QuitJob(0);
        }
        if !events.is_empty() {
            return Decision::Event(0);
        }
        if !purchases.is_empty() {
            return Decision::Purchase(0);
        }
        if !quests.is_empty() {
            return Decision::JoinQuest(0);
        }
        if should_rest_for_goal_race(game, &self.goal, self.min_stamina) {
            return Decision::Wait;
        }
        actions
            .iter()
            .enumerate()
            .filter(|(_, id)| {
                !(self.goal.prioritizes_championships()
                    && !active_jobs.is_empty()
                    && game
                        .catalog
                        .events
                        .iter()
                        .find(|event| event.id == **id)
                        .is_some_and(|event| event.event_type.eq_ignore_ascii_case("work")))
            })
            .max_by(|(_, left), (_, right)| {
                goal_action_value(game, left, Some(&self.goal))
                    .partial_cmp(&goal_action_value(game, right, Some(&self.goal)))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(index, _)| Decision::Action(index))
            .unwrap_or(Decision::Wait)
    }
}

impl Strategy for RequiredStrategy {
    fn choose(
        &mut self,
        _game: &mut GameState,
        _actions: &[String],
        events: &[(String, String)],
        purchases: &[String],
        quests: &[String],
        _active_jobs: &[String],
    ) -> Decision {
        if !events.is_empty() {
            Decision::Event(0)
        } else if !quests.is_empty() {
            Decision::JoinQuest(0)
        } else if !purchases.is_empty() {
            Decision::Purchase(0)
        } else {
            Decision::Wait
        }
    }
}

#[derive(Serialize)]
struct RunRecord {
    seed: u64,
    outcome: String,
    days: u32,
    goal_value: f64,
    budget: f64,
    stamina: f64,
    steps: u32,
    goal: String,
    path: Vec<String>,
}

struct Reporter {
    file: Option<std::fs::File>,
    verbosity: String,
    logs: HashSet<String>,
}

struct RunConfig<'a> {
    dataset: &'a str,
    strategy_kind: StrategyKind,
    goal: &'a GoalSpec,
    max_days: u32,
    overrides: &'a [String],
    outcome_rules: &'a [OutcomeRule],
    speed: &'a str,
    pace_ms: u64,
    max_turns: u32,
    min_stamina: f64,
    fake_results: bool,
}

impl Reporter {
    fn write(&mut self, level: &str, message: &str) {
        let suppressed = match self.verbosity.as_str() {
            "summary" => level != "summary" && level != "log",
            "run" => level == "trace" || level == "deep-trace",
            "trace" => level == "deep-trace",
            _ => false,
        };
        if suppressed {
            return;
        }
        println!("{message}");
        if let Some(file) = &mut self.file {
            if let Err(error) = writeln!(file, "{message}") {
                eprintln!("playtest log write failed: {error}");
            }
        }
    }

    fn write_log(&mut self, kind: &str, message: &str) {
        if self.logs.contains(kind) {
            self.write("log", &format!("{kind} {message}"));
        }
    }
}

fn arg(args: &[String], name: &str, default: &str) -> String {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
        .unwrap_or_else(|| default.into())
}

fn values(args: &[String], name: &str) -> Vec<String> {
    args.windows(2)
        .filter(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
        .collect()
}

fn has_arg(args: &[String], name: &str) -> bool {
    args.iter().any(|arg| arg == name)
}

fn metric(game: &GameState, id: &str) -> f64 {
    game.player.characteristics.get(id).copied().unwrap_or(0.0)
}

fn trace_state(game: &GameState) -> String {
    format!(
        "budget={:.2} charisma={:.2} stamina={:.2}",
        metric(game, "budget"),
        metric(game, "charisma"),
        metric(game, "stamina")
    )
}

fn deep_trace_state(game: &GameState) -> String {
    let mut characteristics = game
        .player
        .characteristics
        .iter()
        .map(|(id, value)| format!("{id}={value:.2}"))
        .collect::<Vec<_>>();
    characteristics.sort();
    let inventory = game
        .player
        .inventory
        .iter()
        .map(|object| object.id.as_str())
        .collect::<Vec<_>>();
    let pending = game
        .pending_events
        .iter()
        .map(|event| format!("{}({})", event.event_id, event.object_id))
        .collect::<Vec<_>>();
    let memberships = game
        .quest_memberships
        .iter()
        .map(|membership| membership.quest_id.as_str())
        .collect::<Vec<_>>();
    let target_events = game
        .catalog
        .events
        .iter()
        .filter(|event| event.quest_id == "honda_civic_fm_national")
        .map(|event| format!("{}:{}", event.id, event.day_of_year))
        .collect::<Vec<_>>();
    let vehicle_details = game
        .player
        .inventory
        .iter()
        .filter(|object| object.object_type == "vehicle")
        .map(|object| {
            format!(
                "{}:unavailable_until={},services={:?}",
                object.id,
                object.unavailable_until_day,
                [
                    object.service_1_needed,
                    object.service_2_needed,
                    object.service_3_needed,
                    object.service_4_needed,
                    object.service_5_needed,
                ]
            )
        })
        .collect::<Vec<_>>();
    format!(
        "state day={} doy={} days_per_year={} rng={} characteristics=[{}] inventory={inventory:?} vehicles={vehicle_details:?} pending={pending:?} memberships={memberships:?} target_events={target_events:?} active_encounter={}",
        game.current_day,
        ((game.current_day - 1) % game.days_per_year) + 1,
        game.days_per_year,
        game.rng_state,
        characteristics.join(","),
        game.active_encounter.is_some()
    )
}

fn service_goal_race_vehicle(game: &mut GameState, goal: &GoalSpec) -> Result<Vec<String>, String> {
    if !goal.prioritizes_championships() {
        return Ok(vec![]);
    }
    let day_of_year = ((game.current_day - 1) % game.days_per_year) + 1;
    let race = game.catalog.events.iter().find(|event| {
        event.day_of_year == day_of_year
            && goal_championship_ids(game, goal).contains(&event.quest_id)
            && event
                .tags
                .split(';')
                .any(|tag| tag.eq_ignore_ascii_case("race"))
    });
    let Some(race) = race else {
        return Ok(vec![]);
    };
    let Some(vehicle) = game.player.inventory.iter().find(|object| {
        object.object_type == "vehicle"
            && race.required_object_ids.split(';').any(|required| {
                let required = required.trim();
                !required.is_empty()
                    && (object.id == required || object.id.starts_with(&format!("{required}_")))
            })
    }) else {
        return Ok(vec![]);
    };
    let object_id = vehicle.id.clone();
    let needed = [
        (vehicle.service_1_needed, ServiceType::Service1),
        (vehicle.service_2_needed, ServiceType::Service2),
        (vehicle.service_3_needed, ServiceType::Service3),
        (vehicle.service_4_needed, ServiceType::Service4),
        (vehicle.service_5_needed, ServiceType::Service5),
        (vehicle.service_6_needed, ServiceType::Service6),
        (vehicle.service_7_needed, ServiceType::Service7),
        (vehicle.service_8_needed, ServiceType::Service8),
        (vehicle.service_9_needed, ServiceType::Service9),
        (vehicle.service_10_needed, ServiceType::Service10),
        (vehicle.service_11_needed, ServiceType::Service11),
        (vehicle.service_12_needed, ServiceType::Service12),
        (vehicle.service_13_needed, ServiceType::Service13),
        (vehicle.service_14_needed, ServiceType::Service14),
        (vehicle.service_15_needed, ServiceType::Service15),
    ];
    let mut serviced = Vec::new();
    for (is_needed, service_type) in needed {
        if is_needed {
            service_object_for_sim(game, &object_id, service_type)?;
            serviced.push(format!("{object_id}:{service_type:?}"));
        }
    }
    Ok(serviced)
}

fn player_objects(game: &GameState) -> String {
    let mut objects = game
        .player
        .inventory
        .iter()
        .map(|object| object.id.clone())
        .collect::<Vec<_>>();
    objects.sort();
    format!("{objects:?}")
}

fn player_state(game: &GameState) -> String {
    format!(
        "stamina={:.2} paddock_cred={:.2} budget={:.2} active_jobs={:?} next_day_obligation_stamina={:.2} objects={}",
        metric(game, "stamina"),
        metric(game, "charisma"),
        metric(game, "budget"),
        active_work_jobs(game),
        stamina_obligations_due(game, game.current_day.saturating_add(1)),
        player_objects(game)
    )
}

fn decision_description(
    decision: Decision,
    actions: &[String],
    events: &[(String, String)],
    purchases: &[String],
    quests: &[String],
    active_jobs: &[String],
) -> String {
    match decision {
        Decision::Action(index) => format!(
            "action {}",
            actions.get(index).map(String::as_str).unwrap_or("unknown")
        ),
        Decision::Event(index) => {
            let (event, object) = events
                .get(index)
                .map(|entry| (entry.0.as_str(), entry.1.as_str()))
                .unwrap_or(("unknown", ""));
            if object.is_empty() {
                format!("event {event}")
            } else {
                format!("event {event} with {object}")
            }
        }
        Decision::Purchase(index) => format!(
            "buy {}",
            purchases
                .get(index)
                .map(String::as_str)
                .unwrap_or("unknown")
        ),
        Decision::JoinQuest(index) => format!(
            "join championship {}",
            quests.get(index).map(String::as_str).unwrap_or("unknown")
        ),
        Decision::QuitJob(index) => format!(
            "quit job {}",
            active_jobs
                .get(index)
                .map(String::as_str)
                .unwrap_or("unknown")
        ),
        Decision::Wait => "wait".into(),
    }
}

fn decision_reason(
    decision: Decision,
    goal: &GoalSpec,
    actions: &[String],
    events: &[(String, String)],
    purchases: &[String],
    quests: &[String],
    active_jobs: &[String],
) -> &'static str {
    match decision {
        Decision::JoinQuest(_) if goal.prioritizes_championships() => {
            "required by championship goal"
        }
        Decision::Event(_) if !events.is_empty() => "an eligible event is available",
        Decision::Purchase(_) if !purchases.is_empty() => "the next affordable required object",
        Decision::Action(_) if !actions.is_empty() => "best available action",
        Decision::JoinQuest(_) if !quests.is_empty() => "an eligible championship is available",
        Decision::QuitJob(_) if !active_jobs.is_empty() => {
            "a job obligation would prevent the goal race"
        }
        Decision::Wait => "no eligible action, event, purchase, or championship",
        _ => "strategy choice",
    }
}

fn decision_debug(
    decision: Decision,
    actions: &[String],
    events: &[(String, String)],
    purchases: &[String],
    quests: &[String],
    active_jobs: &[String],
) -> String {
    match decision {
        Decision::Action(index) => format!(
            "Action({:?})",
            actions.get(index).map(String::as_str).unwrap_or("unknown")
        ),
        Decision::Event(index) => format!(
            "Event({:?})",
            events
                .get(index)
                .map(|(event, object)| format!("{event}, {object}"))
                .unwrap_or_else(|| "unknown".into())
        ),
        Decision::Purchase(index) => format!(
            "Purchase({:?})",
            purchases
                .get(index)
                .map(String::as_str)
                .unwrap_or("unknown")
        ),
        Decision::JoinQuest(index) => format!(
            "JoinQuest({:?})",
            quests.get(index).map(String::as_str).unwrap_or("unknown")
        ),
        Decision::QuitJob(index) => format!(
            "QuitJob({:?})",
            active_jobs
                .get(index)
                .map(String::as_str)
                .unwrap_or("unknown")
        ),
        Decision::Wait => "Wait".into(),
    }
}

fn failed_quest_reason(game: &GameState, quest_id: &str) -> String {
    let Some(quest) = game
        .catalog
        .quests
        .iter()
        .find(|quest| quest.id == quest_id)
    else {
        return "unknown championship".into();
    };
    if quest.join_fee > metric(game, "budget") {
        return format!("requires {:.2} budget", quest.join_fee);
    }
    if !quest.required_license_id.trim().is_empty()
        && !game.player.inventory.iter().any(|owned| {
            owned.id == quest.required_license_id
                || owned
                    .id
                    .starts_with(&format!("{}_", quest.required_license_id))
        })
    {
        return format!("requires {}", quest.required_license_id);
    }
    for event in game
        .catalog
        .events
        .iter()
        .filter(|event| event.quest_id == quest_id)
    {
        if let Some(required) = event
            .required_object_ids
            .split(';')
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .find(|required| {
                !game.player.inventory.iter().any(|owned| {
                    owned.id == *required || owned.id.starts_with(&format!("{required}_"))
                })
            })
        {
            return format!("requires {required}");
        }
    }
    "not currently eligible".into()
}

fn normalize_verbosity(value: String, deep_trace: bool) -> String {
    if deep_trace || value.eq_ignore_ascii_case("deep_trace") {
        "deep-trace".into()
    } else {
        value.to_ascii_lowercase()
    }
}

fn parse_strategy(value: &str) -> StrategyKind {
    match value.to_ascii_lowercase().as_str() {
        "greedy" => StrategyKind::Greedy,
        "goal-aware" | "goal_aware" | "championship" | "planned" => StrategyKind::GoalAware,
        "required" | "required-only" => StrategyKind::Required,
        _ => StrategyKind::Random,
    }
}

fn action_value(game: &GameState, id: &str) -> f64 {
    goal_action_value(game, id, None)
}

fn championship_funding_needed(game: &GameState, goal: &GoalSpec) -> f64 {
    if !goal.prioritizes_championships() {
        return 0.0;
    }
    let target_ids = championship_quest_ids(game, goal);
    let mut needed = 0.0;
    for object_id in championship_purchase_plan(game, goal) {
        let owned = game.player.inventory.iter().any(|object| {
            object.id == object_id || object.id.starts_with(&format!("{object_id}_"))
        });
        if owned {
            continue;
        }
        if let Some(object) = game
            .catalog
            .objects
            .iter()
            .find(|object| object.id == object_id)
        {
            needed += if object.object_type == "license" && object.license_fee > 0.0 {
                object.license_fee
            } else {
                object.price
            };
        }
    }
    needed += game
        .catalog
        .quests
        .iter()
        .filter(|quest| target_ids.contains(&quest.id))
        .map(|quest| quest.join_fee)
        .sum::<f64>();
    needed + 250.0
}

fn goal_action_value(game: &GameState, id: &str, goal: Option<&GoalSpec>) -> f64 {
    game.catalog
        .events
        .iter()
        .find(|action| action.id == id)
        .map(|action| {
            let interval_days = if action.payout_freq_type.eq_ignore_ascii_case("recurring") {
                payout_interval_days(action.payout_freq, &action.payout_freq_unit)
            } else {
                1
            };
            let expected_income = action.success_rate * action.payout / interval_days as f64;
            let obligation_cost = game
                .catalog
                .obligations
                .iter()
                .filter(|obligation| obligation.event_id == action.id)
                .filter(|obligation| obligation.resource.eq_ignore_ascii_case("stamina"))
                .map(|obligation| obligation.amount)
                .sum::<f64>();
            let obligation_penalty = if goal.is_some_and(GoalSpec::prioritizes_championships) {
                obligation_cost * 20.0
            } else {
                obligation_cost * 5.0
            };
            expected_income
                - action.base_cost
                - event_upfront_cost(game, &action.id) * 10.0
                - obligation_penalty
        })
        .unwrap_or(f64::MIN)
}

fn payout_interval_days(value: u32, unit: &str) -> u32 {
    match unit.trim().to_ascii_lowercase().as_str() {
        "day" | "days" => value.max(1),
        "week" | "weeks" => value.max(1).saturating_mul(7),
        "month" | "months" => value.max(1).saturating_mul(30),
        "year" | "years" => value.max(1).saturating_mul(365),
        _ => 1,
    }
}

fn event_upfront_cost(game: &GameState, event_id: &str) -> f64 {
    let Some(event) = game
        .catalog
        .events
        .iter()
        .find(|event| event.id == event_id)
    else {
        return f64::INFINITY;
    };
    if game
        .catalog
        .obligations
        .iter()
        .any(|obligation| obligation.event_id == event.id)
    {
        0.0
    } else {
        event.stamina_cost.max(0.0)
    }
}

fn event_duration_days(duration_value: u32, duration_unit: &str) -> u32 {
    match duration_unit.trim().to_ascii_lowercase().as_str() {
        "day" | "days" => duration_value,
        "week" | "weeks" => duration_value.saturating_mul(7),
        _ => 0,
    }
}

fn event_stamina_cost(game: &GameState, event_id: &str) -> f64 {
    let Some(event) = game
        .catalog
        .events
        .iter()
        .find(|event| event.id == event_id)
    else {
        return f64::INFINITY;
    };
    let event_cost = if event
        .tags
        .split(';')
        .any(|tag| tag.trim().eq_ignore_ascii_case("race"))
    {
        let daily_cost = game
            .catalog
            .labels
            .values
            .get("race_day_stamina_cost")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(20.0);
        daily_cost * event_duration_days(event.duration_value, &event.duration_unit).max(1) as f64
    } else {
        event.stamina_cost.max(0.0)
    };
    event_upfront_cost(game, event_id) + event_cost
}

fn interval_days(interval: u32, unit: &str) -> u32 {
    payout_interval_days(interval, unit)
}

fn weekday(day: u32) -> u32 {
    ((day.saturating_sub(1)) % 7) + 1
}

fn obligation_due_on_day(
    start_day: u32,
    payments: u32,
    interval: u32,
    interval_unit: &str,
    due_days: &str,
    max_payments: u32,
    day: u32,
) -> bool {
    let interval = interval_days(interval, interval_unit);
    let elapsed = day.saturating_sub(start_day);
    if interval == 0 || elapsed == 0 || elapsed % interval != 0 {
        return false;
    }
    if max_payments > 0 && payments >= max_payments {
        return false;
    }
    let due_days = due_days
        .split(';')
        .filter_map(|value| value.trim().parse::<u32>().ok())
        .collect::<Vec<_>>();
    due_days.is_empty() || due_days.contains(&weekday(day))
}

fn stamina_obligations_due(game: &GameState, day: u32) -> f64 {
    game.player
        .active_events
        .iter()
        .map(|active| {
            game.catalog
                .obligations
                .iter()
                .filter(|obligation| {
                    obligation.event_id == active.event_id
                        && obligation.resource.eq_ignore_ascii_case("stamina")
                        && obligation_due_on_day(
                            active.start_day,
                            active.obligation_payments,
                            obligation.interval,
                            &obligation.interval_unit,
                            &obligation.due_days,
                            obligation.max_payments,
                            day,
                        )
                })
                .map(|obligation| obligation.amount)
                .sum::<f64>()
        })
        .sum()
}

fn candidate_obligations_due(game: &GameState, event_id: &str, day: u32) -> f64 {
    game.catalog
        .obligations
        .iter()
        .filter(|obligation| {
            obligation.event_id == event_id
                && obligation.resource.eq_ignore_ascii_case("stamina")
                && obligation_due_on_day(
                    game.current_day,
                    0,
                    obligation.interval,
                    &obligation.interval_unit,
                    &obligation.due_days,
                    obligation.max_payments,
                    day,
                )
        })
        .map(|obligation| obligation.amount)
        .sum()
}

fn active_work_jobs(game: &GameState) -> Vec<String> {
    let mut jobs = game
        .player
        .active_events
        .iter()
        .filter_map(|active| {
            game.catalog
                .events
                .iter()
                .find(|event| event.id == active.event_id)
                .filter(|event| event.event_type.eq_ignore_ascii_case("work"))
                .map(|event| event.id.clone())
        })
        .collect::<Vec<_>>();
    jobs.sort();
    jobs.dedup();
    jobs
}

fn projected_stamina_for_action(game: &GameState, event_id: &str, min_stamina: f64) -> bool {
    let next_day = game.current_day.saturating_add(1);
    let next_obligations = stamina_obligations_due(game, next_day)
        + candidate_obligations_due(game, event_id, next_day);
    metric(game, "stamina") - event_stamina_cost(game, event_id) - next_obligations >= min_stamina
}

fn projected_stamina_at_race(game: &GameState, race_day_of_year: u32, race_id: &str) -> f64 {
    let mut stamina = metric(game, "stamina");
    let current_day_of_year = ((game.current_day - 1) % game.days_per_year) + 1;
    let target_day = if race_day_of_year >= current_day_of_year {
        game.current_day + race_day_of_year - current_day_of_year
    } else {
        game.current_day + game.days_per_year - current_day_of_year + race_day_of_year
    };
    let recovery = game
        .catalog
        .labels
        .values
        .get("daily_stamina_recovery")
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(25.0);
    for day in (game.current_day + 1)..=target_day {
        stamina -= stamina_obligations_due(game, day);
        stamina += recovery;
    }
    stamina - event_stamina_cost(game, race_id)
}

fn goal_championship_ids(game: &GameState, goal: &GoalSpec) -> HashSet<String> {
    match goal {
        GoalSpec::Championship { id } => [id.clone()].into_iter().collect(),
        GoalSpec::Championships { level, count } => game
            .catalog
            .quests
            .iter()
            .filter(|quest| quest.level == *level)
            .take(*count)
            .map(|quest| quest.id.clone())
            .collect(),
        GoalSpec::Characteristic { .. } => HashSet::new(),
    }
}

fn should_rest_for_goal_race(game: &GameState, goal: &GoalSpec, min_stamina: f64) -> bool {
    if !goal.prioritizes_championships() {
        return false;
    }
    let target_quests = goal_championship_ids(game, goal);
    if target_quests.is_empty() {
        return false;
    }
    let day_of_year = ((game.current_day - 1) % game.days_per_year) + 1;
    let next_race = game
        .catalog
        .events
        .iter()
        .filter(|event| {
            target_quests.contains(&event.quest_id)
                && event
                    .tags
                    .split(';')
                    .any(|tag| tag.trim().eq_ignore_ascii_case("race"))
        })
        .map(|event| {
            let days_until = if event.day_of_year >= day_of_year {
                event.day_of_year - day_of_year
            } else {
                game.days_per_year - day_of_year + event.day_of_year
            };
            (event, days_until)
        })
        .min_by_key(|(_, days_until)| *days_until);
    let Some((race, days_until)) = next_race else {
        return false;
    };
    if days_until == 0 || days_until > 14 {
        return false;
    }
    let daily_obligations = (1..=days_until)
        .map(|offset| stamina_obligations_due(game, game.current_day + offset))
        .sum::<f64>();
    let minimum_daily_action_cost = game
        .catalog
        .events
        .iter()
        .filter(|event| event.day_of_year == 0)
        .map(|event| event_upfront_cost(game, &event.id))
        .filter(|cost| cost.is_finite())
        .min_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(1.0)
        .max(1.0);
    metric(game, "stamina")
        < event_stamina_cost(game, &race.id)
            + min_stamina
            + daily_obligations
            + minimum_daily_action_cost * days_until as f64
}

fn should_quit_job_for_goal_race(
    game: &GameState,
    goal: &GoalSpec,
    min_stamina: f64,
    active_jobs: &[String],
) -> bool {
    if !goal.prioritizes_championships() || active_jobs.is_empty() {
        return false;
    }
    let target_quests = goal_championship_ids(game, goal);
    let day_of_year = ((game.current_day - 1) % game.days_per_year) + 1;
    let Some(race) = game
        .catalog
        .events
        .iter()
        .filter(|event| {
            target_quests.contains(&event.quest_id)
                && event
                    .tags
                    .split(';')
                    .any(|tag| tag.trim().eq_ignore_ascii_case("race"))
        })
        .filter(|event| event.day_of_year >= day_of_year)
        .min_by_key(|event| event.day_of_year - day_of_year)
    else {
        return false;
    };
    projected_stamina_at_race(game, race.day_of_year, &race.id) < min_stamina
}

fn stamina_safe_actions(game: &GameState, actions: Vec<String>, min_stamina: f64) -> Vec<String> {
    actions
        .into_iter()
        .filter(|id| projected_stamina_for_action(game, id, min_stamina))
        .collect()
}

fn stamina_safe_entries(
    game: &GameState,
    entries: Vec<(String, String)>,
    min_stamina: f64,
) -> Vec<(String, String)> {
    entries
        .into_iter()
        .filter(|(event_id, _)| {
            let duration = event_duration_days_for_entry(game, event_id);
            let obligations = (1..=duration.max(1))
                .map(|offset| stamina_obligations_due(game, game.current_day + offset))
                .sum::<f64>();
            metric(game, "stamina") - event_stamina_cost(game, event_id) - obligations
                >= min_stamina
        })
        .collect()
}

fn event_duration_days_for_entry(game: &GameState, event_id: &str) -> u32 {
    game.catalog
        .events
        .iter()
        .find(|event| event.id == event_id)
        .map(|event| event_duration_days(event.duration_value, &event.duration_unit))
        .unwrap_or(1)
}

fn purchase_candidates(game: &GameState) -> Vec<String> {
    let mut candidates: Vec<_> = game
        .catalog
        .objects
        .iter()
        .filter(|object| {
            !object.id.starts_with("trophy_")
                && object.id != "trophies"
                && object.id != "business_proposal"
                && object.id != "lower_cost"
        })
        .filter(|object| {
            let price = if object.object_type == "license" && object.license_fee > 0.0 {
                object.license_fee
            } else {
                object.price
            };
            price <= metric(game, "budget")
        })
        .filter(|object| {
            !game.player.inventory.iter().any(|owned| {
                owned.id == object.id || owned.id.starts_with(&format!("{}_", object.id))
            })
        })
        .filter(|object| {
            object
                .requires_object_ids
                .split(';')
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .all(|required| {
                    game.player.inventory.iter().any(|owned| {
                        owned.id == required || owned.id.starts_with(&format!("{}_", required))
                    })
                })
        })
        .filter(|object| {
            object.license_previous_id.trim().is_empty()
                || game.player.inventory.iter().any(|owned| {
                    owned.id == object.license_previous_id
                        || owned
                            .id
                            .starts_with(&format!("{}_", object.license_previous_id))
                })
        })
        .map(|object| {
            let price = if object.object_type == "license" && object.license_fee > 0.0 {
                object.license_fee
            } else {
                object.price
            };
            (object.id.clone(), price)
        })
        .collect();
    candidates.sort_by(|left, right| {
        left.1
            .partial_cmp(&right.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.into_iter().map(|(id, _)| id).collect()
}

fn championship_quest_ids(game: &GameState, goal: &GoalSpec) -> HashSet<String> {
    match goal {
        GoalSpec::Championship { id } => [id.clone()].into_iter().collect(),
        GoalSpec::Championships { level, count } => game
            .catalog
            .quests
            .iter()
            .filter(|quest| quest.level == *level)
            .take(*count)
            .map(|quest| quest.id.clone())
            .collect(),
        GoalSpec::Characteristic { .. } => HashSet::new(),
    }
}

fn championship_purchase_plan(game: &GameState, goal: &GoalSpec) -> HashSet<String> {
    let quest_ids = championship_quest_ids(game, goal);
    let mut required_ids = HashSet::new();
    for event in game
        .catalog
        .events
        .iter()
        .filter(|event| quest_ids.contains(&event.quest_id))
    {
        required_ids.extend(
            event
                .required_object_ids
                .split(';')
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .map(str::to_string),
        );
        if !event.required_license_id.trim().is_empty() {
            required_ids.insert(event.required_license_id.clone());
        }
    }

    let mut pending: Vec<_> = required_ids.iter().cloned().collect();
    while let Some(id) = pending.pop() {
        let Some(object) = game.catalog.objects.iter().find(|object| object.id == id) else {
            continue;
        };
        for prerequisite in object
            .requires_object_ids
            .split(';')
            .map(str::trim)
            .filter(|id| !id.is_empty())
        {
            if required_ids.insert(prerequisite.to_string()) {
                pending.push(prerequisite.to_string());
            }
        }
        if !object.license_previous_id.trim().is_empty()
            && required_ids.insert(object.license_previous_id.clone())
        {
            pending.push(object.license_previous_id.clone());
        }
    }
    required_ids
}

fn championship_purchase_candidates(
    game: &GameState,
    goal: &GoalSpec,
    purchases: Vec<String>,
) -> Vec<String> {
    if !goal.prioritizes_championships() {
        return purchases;
    }
    let plan = championship_purchase_plan(game, goal);
    let quest_ids = championship_quest_ids(game, goal);
    let required_licenses: HashSet<_> = game
        .catalog
        .events
        .iter()
        .filter(|event| quest_ids.contains(&event.quest_id))
        .map(|event| event.required_license_id.as_str())
        .filter(|id| !id.trim().is_empty())
        .collect();
    let has_required_licenses = required_licenses.iter().all(|required| {
        game.player
            .inventory
            .iter()
            .any(|owned| owned.id == *required || owned.id.starts_with(&format!("{required}_")))
    });

    purchases
        .into_iter()
        .filter(|id| plan.contains(id))
        .filter(|id| {
            has_required_licenses
                || game
                    .catalog
                    .objects
                    .iter()
                    .find(|object| object.id == *id)
                    .is_none_or(|object| object.object_type != "vehicle")
        })
        .collect()
}

fn championship_quest_candidates(
    game: &GameState,
    goal: &GoalSpec,
    quests: Vec<String>,
) -> Vec<String> {
    if !goal.prioritizes_championships() {
        return quests;
    }
    let target_ids = championship_quest_ids(game, goal);
    quests
        .into_iter()
        .filter(|quest_id| target_ids.contains(quest_id))
        .filter(|quest_id| {
            let events: Vec<_> = game
                .catalog
                .events
                .iter()
                .filter(|event| event.quest_id == *quest_id)
                .collect();
            events.iter().all(|event| {
                let license_ready = event.required_license_id.trim().is_empty()
                    || game.player.inventory.iter().any(|owned| {
                        owned.id == event.required_license_id
                            || owned
                                .id
                                .starts_with(&format!("{}_", event.required_license_id))
                    });
                let car_ready = event
                    .required_object_ids
                    .split(';')
                    .map(str::trim)
                    .filter(|id| !id.is_empty())
                    .all(|required| {
                        game.player.inventory.iter().any(|owned| {
                            owned.id == required || owned.id.starts_with(&format!("{required}_"))
                        })
                    });
                license_ready && car_ready
            })
        })
        .collect()
}

fn quest_candidates(game: &GameState) -> Vec<String> {
    game.catalog
        .quests
        .iter()
        .filter(|quest| {
            !game
                .quest_memberships
                .iter()
                .any(|membership| membership.quest_id == quest.id)
        })
        .filter(|quest| quest.join_fee <= metric(game, "budget"))
        .filter(|quest| {
            quest.required_license_id.trim().is_empty()
                || game.player.inventory.iter().any(|object| {
                    object.id == quest.required_license_id
                        || object
                            .id
                            .starts_with(&format!("{}_", quest.required_license_id))
                })
        })
        .map(|quest| quest.id.clone())
        .collect()
}

fn outcome_for(
    game: &mut GameState,
    event_id: &str,
    rules: &[OutcomeRule],
) -> (String, Option<u32>) {
    let event = game
        .catalog
        .events
        .iter()
        .find(|event| event.id == event_id);
    let event_type = event
        .map(|event| event.event_type.as_str())
        .unwrap_or("event");
    let rule = rules
        .iter()
        .find(|rule| rule.selector == format!("event:{event_id}"))
        .or_else(|| {
            rules
                .iter()
                .find(|rule| rule.selector == format!("type:{event_type}"))
        });
    if let Some(rule) = rule {
        if let Some(rank) = rule.fixed_rank {
            return ("success".into(), Some(rank));
        }
    }
    (
        "success".into(),
        Some((roll(game) * 5.0).floor() as u32 + 1),
    )
}

fn run_one(seed: u64, config: &RunConfig<'_>, reporter: &mut Reporter) -> RunRecord {
    let mut game = new_game_seeded(config.dataset, seed);
    for override_value in config.overrides {
        if let Some((path, value)) = override_value.split_once('=') {
            if let Err(error) = apply_override(&mut game, path, value) {
                reporter.write("run", &format!("seed={seed} override_error={error}"));
            }
        }
    }
    let mut strategy: Box<dyn Strategy> = match config.strategy_kind {
        StrategyKind::Greedy => Box::new(GreedyStrategy),
        StrategyKind::GoalAware => Box::new(GoalAwareStrategy {
            goal: config.goal.clone(),
            min_stamina: config.min_stamina,
        }),
        StrategyKind::Required => Box::new(RequiredStrategy),
        StrategyKind::Random => Box::new(RandomStrategy),
    };
    let mut steps = 0;
    let mut path = Vec::new();
    loop {
        if config.goal.reached(&game) {
            break;
        }
        let status = run_status(&game, "", f64::INFINITY, config.max_days);
        if status != RunStatus::Ongoing {
            break;
        }
        let day_before = game.current_day;
        reporter.write_log(
            "player_objects",
            &format!(
                "seed={seed} day={} objects={} budget={:.2} stamina={:.2}",
                game.current_day,
                player_objects(&game),
                metric(&game, "budget"),
                metric(&game, "stamina")
            ),
        );
        reporter.write_log(
            "player",
            &format!(
                "seed={seed} day={} {}",
                game.current_day,
                player_state(&game)
            ),
        );
        if game.active_encounter.is_some() {
            let encounter_actions = legal_encounter_action_ids(&game);
            let action_id = encounter_actions.first().map(String::as_str);
            reporter.write_log(
                "decision",
                &format!(
                    "seed={seed} day={} encounter {} because it is the first legal encounter action",
                    game.current_day,
                    action_id.unwrap_or("pass")
                ),
            );
            if let Err(error) = resolve_encounter_for_sim(&mut game, action_id) {
                reporter.write("run", &format!("seed={seed} encounter_error={error}"));
                reporter.write_log(
                    "decision",
                    &format!(
                        "seed={seed} day={} encounter {}, failed: {error}",
                        game.current_day,
                        action_id.unwrap_or("pass")
                    ),
                );
                break;
            }
            path.push(format!(
                "day:{}:encounter:{}",
                game.current_day,
                action_id.unwrap_or("pass")
            ));
            reporter.write(
                "trace",
                &format!(
                    "seed={seed} day={} encounter_actions={encounter_actions:?} selected={} {}",
                    game.current_day,
                    action_id.unwrap_or("pass"),
                    trace_state(&game)
                ),
            );
            reporter.write(
                "deep-trace",
                &format!("seed={seed} {}", deep_trace_state(&game)),
            );
        } else {
            match service_goal_race_vehicle(&mut game, config.goal) {
                Ok(serviced) if !serviced.is_empty() => reporter.write_log(
                    "decision",
                    &format!(
                        "seed={seed} day={} serviced goal-race vehicle: {serviced:?}",
                        game.current_day
                    ),
                ),
                Ok(_) => {}
                Err(error) => {
                    reporter.write(
                        "run",
                        &format!("seed={seed} day={} service_error={error}", game.current_day),
                    );
                    break;
                }
            }
            let actions = stamina_safe_actions(&game, legal_event_ids(&game), config.min_stamina);
            let entries =
                stamina_safe_entries(&game, eligible_event_entries(&game), config.min_stamina);
            let purchases =
                championship_purchase_candidates(&game, config.goal, purchase_candidates(&game));
            let quests = if entries.is_empty() {
                championship_quest_candidates(&game, config.goal, quest_candidates(&game))
            } else {
                vec![]
            };
            let active_jobs = active_work_jobs(&game);
            reporter.write_log(
                "available_events",
                &format!("seed={seed} day={} events={entries:?}", game.current_day),
            );
            if config.goal.prioritizes_championships() && quests.is_empty() {
                for quest_id in championship_quest_ids(&game, config.goal) {
                    reporter.write_log(
                        "decision",
                        &format!(
                            "seed={seed} day={} join championship {quest_id}, failed: {}",
                            game.current_day,
                            failed_quest_reason(&game, &quest_id)
                        ),
                    );
                }
            }
            reporter.write(
                "trace",
                &format!(
                    "seed={seed} day={} possibilities=events:{:?};actions:{:?};purchases:{:?};quests:{:?}",
                    game.current_day, entries, actions, purchases, quests
                ),
            );
            let rng_before_decision = game.rng_state;
            let decision = if config.goal.prioritizes_championships() && !quests.is_empty() {
                Decision::JoinQuest(0)
            } else {
                strategy.choose(
                    &mut game,
                    &actions,
                    &entries,
                    &purchases,
                    &quests,
                    &active_jobs,
                )
            };
            reporter.write_log(
                "decision",
                &format!(
                    "seed={seed} day={} decision={} ({}) because {}",
                    game.current_day,
                    decision_debug(
                        decision,
                        &actions,
                        &entries,
                        &purchases,
                        &quests,
                        &active_jobs,
                    ),
                    decision_description(
                        decision,
                        &actions,
                        &entries,
                        &purchases,
                        &quests,
                        &active_jobs,
                    ),
                    decision_reason(
                        decision,
                        config.goal,
                        &actions,
                        &entries,
                        &purchases,
                        &quests,
                        &active_jobs
                    )
                ),
            );
            reporter.write(
                "deep-trace",
                &format!(
                    "seed={seed} {} candidates events={entries:?} actions={actions:?} purchases={purchases:?} quests={quests:?} decision={} rng_before={rng_before_decision} rng_after={}",
                    deep_trace_state(&game),
                    decision_debug(
                        decision,
                        &actions,
                        &entries,
                        &purchases,
                        &quests,
                        &active_jobs,
                    ),
                    game.rng_state
                ),
            );
            match decision {
                Decision::Event(index) => {
                    let (event_id, object_id) = &entries[index];
                    match enter_event_for_sim(&mut game, event_id, object_id) {
                        Ok(()) => {
                            if let Some(pending) = game
                                .pending_events
                                .iter()
                                .rev()
                                .find(|entry| entry.event_id == *event_id)
                            {
                                let pending_id = pending.id.clone();
                                let (result, position) = if config.fake_results
                                    && game.catalog.events.iter().any(|event| {
                                        event.id == *event_id
                                            && event.event_type.eq_ignore_ascii_case("race")
                                            && !event.quest_id.trim().is_empty()
                                    }) {
                                    ("success".into(), Some(1))
                                } else {
                                    outcome_for(&mut game, event_id, config.outcome_rules)
                                };
                                if let Err(error) = submit_event_for_sim_with_details(
                                    &mut game,
                                    &pending_id,
                                    &result,
                                    position,
                                ) {
                                    reporter.write(
                                        "run",
                                        &format!("seed={seed} event_result_error={error}"),
                                    );
                                }
                                reporter.write(
                                    "trace",
                                    &format!(
                                        "seed={seed} day={} event={} result={} {}",
                                        game.current_day,
                                        event_id,
                                        result,
                                        trace_state(&game)
                                    ),
                                );
                                path.push(format!(
                                    "day:{}:event:{}:{}:position={}",
                                    game.current_day,
                                    event_id,
                                    result,
                                    position.unwrap_or(0)
                                ));
                            }
                        }
                        Err(error) => {
                            reporter.write(
                                "run",
                                &format!(
                                    "seed={seed} event={} object={} enter_error={error}",
                                    event_id, object_id
                                ),
                            );
                            reporter.write_log(
                                "decision",
                                &format!(
                                    "seed={seed} day={} event {event_id}, failed: {error}",
                                    game.current_day
                                ),
                            );
                        }
                    }
                }
                Decision::Action(index) => {
                    let action_id = &actions[index];
                    match apply_event(&mut game, action_id) {
                        Ok(result) => {
                            path.push(format!(
                                "day:{}:action:{}:success={}",
                                game.current_day, action_id, result.success
                            ));
                            reporter.write(
                                "trace",
                                &format!(
                                    "seed={seed} day={} action={} success={} {}",
                                    game.current_day,
                                    action_id,
                                    result.success,
                                    trace_state(&game)
                                ),
                            );
                        }
                        Err(error) => {
                            reporter.write(
                                "run",
                                &format!("seed={seed} action={action_id} apply_error={error}"),
                            );
                            reporter.write_log(
                                "decision",
                                &format!(
                                    "seed={seed} day={} action {action_id}, failed: {error}",
                                    game.current_day
                                ),
                            );
                        }
                    }
                }
                Decision::Purchase(index) => {
                    let object_id = &purchases[index];
                    match buy_object_for_sim(&mut game, object_id) {
                        Ok(()) => {
                            path.push(format!("day:{}:purchase:{}", game.current_day, object_id));
                            reporter.write(
                                "trace",
                                &format!(
                                    "seed={seed} day={} purchase={} {}",
                                    game.current_day,
                                    object_id,
                                    trace_state(&game)
                                ),
                            );
                        }
                        Err(error) => {
                            reporter.write(
                                "run",
                                &format!("seed={seed} purchase={object_id} buy_error={error}"),
                            );
                            reporter.write_log(
                                "decision",
                                &format!(
                                    "seed={seed} day={} buy {object_id}, failed: {error}",
                                    game.current_day
                                ),
                            );
                        }
                    }
                }
                Decision::JoinQuest(index) => {
                    let quest_id = &quests[index];
                    match join_quest_for_sim(&mut game, quest_id) {
                        Ok(()) => {
                            path.push(format!("day:{}:join_quest:{}", game.current_day, quest_id));
                            reporter.write(
                                "trace",
                                &format!(
                                    "seed={seed} day={} join_quest={} {}",
                                    game.current_day,
                                    quest_id,
                                    trace_state(&game)
                                ),
                            );
                        }
                        Err(error) => {
                            reporter.write(
                                "run",
                                &format!("seed={seed} quest={quest_id} join_error={error}"),
                            );
                            reporter.write_log(
                                "decision",
                                &format!(
                                    "seed={seed} day={} join championship {quest_id}, failed: {error}",
                                    game.current_day
                                ),
                            );
                        }
                    }
                }
                Decision::QuitJob(index) => {
                    let job_id = &active_jobs[index];
                    match quit_event_for_sim(&mut game, job_id) {
                        Ok(()) => {
                            path.push(format!("day:{}:quit_job:{}", game.current_day, job_id));
                            reporter.write(
                                "trace",
                                &format!(
                                    "seed={seed} day={} quit_job={} {}",
                                    game.current_day,
                                    job_id,
                                    trace_state(&game)
                                ),
                            );
                        }
                        Err(error) => {
                            reporter.write(
                                "run",
                                &format!("seed={seed} job={job_id} quit_error={error}"),
                            );
                            reporter.write_log(
                                "decision",
                                &format!(
                                    "seed={seed} day={} quit job {job_id}, failed: {error}",
                                    game.current_day
                                ),
                            );
                        }
                    }
                }
                Decision::Wait => {
                    path.push(format!("day:{}:wait", game.current_day));
                    reporter.write(
                        "trace",
                        &format!(
                            "seed={seed} day={} wait {}",
                            game.current_day,
                            trace_state(&game)
                        ),
                    );
                }
            }
        }
        steps += 1;
        if config.speed == "paced" {
            sleep(Duration::from_millis(config.pace_ms));
        }
        if steps >= config.max_turns {
            break;
        }
        if game.current_day == day_before && game.active_encounter.is_none() {
            if let Err(error) = advance_day(&mut game) {
                reporter.write("run", &format!("seed={seed} advance_error={error}"));
                break;
            }
        }
    }
    let status = if config.goal.reached(&game) {
        RunStatus::GoalReached
    } else {
        run_status(&game, "", f64::INFINITY, config.max_days)
    };
    let outcome = match status {
        RunStatus::GoalReached => "goal",
        RunStatus::DeadMoney => "dead_money",
        RunStatus::DeadStamina => "dead_stamina",
        RunStatus::Dead => "dead",
        RunStatus::MaxDays => "timeout",
        RunStatus::Ongoing => "timeout",
    };
    RunRecord {
        seed,
        outcome: outcome.into(),
        days: game.current_day.saturating_sub(1),
        goal_value: match config.goal {
            GoalSpec::Characteristic { id, .. } => metric(&game, id),
            _ => game
                .player
                .inventory
                .iter()
                .filter(|object| object.object_type == "achievements")
                .count() as f64,
        },
        budget: metric(&game, "budget"),
        stamina: metric(&game, "stamina"),
        steps,
        goal: config.goal.description(),
        path,
    }
}

fn percentile(mut values: Vec<u32>, fraction: f64) -> u32 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    values[((values.len() - 1) as f64 * fraction).round() as usize]
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_config = args
        .windows(2)
        .find(|pair| pair[0] == "--config")
        .map(|pair| pair[1].clone())
        .map(|path| {
            let contents = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read playtest config '{path}': {error}"));
            serde_json::from_str::<FileConfig>(&contents)
                .unwrap_or_else(|error| panic!("parse playtest config '{path}': {error}"))
        })
        .unwrap_or_default();
    let dataset = if has_arg(&args, "--dataset") {
        arg(&args, "--dataset", "dataset")
    } else {
        file_config.dataset.unwrap_or_else(|| "dataset".into())
    };
    let runs: u32 = if has_arg(&args, "--runs") {
        arg(&args, "--runs", "1").parse().unwrap_or(1)
    } else {
        file_config.runs.unwrap_or(1)
    }
    .max(1);
    let max_days: u32 = if has_arg(&args, "--max-days") {
        arg(&args, "--max-days", "7300").parse().unwrap_or(7300)
    } else {
        file_config.max_days.unwrap_or(7300)
    };
    let strategy_name = if has_arg(&args, "--strategy") {
        arg(&args, "--strategy", "greedy")
    } else {
        file_config.strategy.unwrap_or_else(|| "greedy".into())
    };
    let strategy = parse_strategy(&strategy_name);
    let goal_text = if has_arg(&args, "--goal") {
        arg(&args, "--goal", "charisma>=100")
    } else {
        file_config.goal.unwrap_or_else(|| "charisma>=100".into())
    };
    let goal = GoalSpec::parse(&goal_text);
    let seed_base = if has_arg(&args, "--seed") {
        arg(&args, "--seed", "").parse().unwrap_or(0)
    } else {
        file_config.seed.unwrap_or(0)
    };
    let seed_base = if seed_base == 0 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    } else {
        seed_base
    };
    let verbosity = if has_arg(&args, "--verbosity") {
        arg(&args, "--verbosity", "summary")
    } else {
        file_config.verbosity.unwrap_or_else(|| "summary".into())
    };
    let verbosity = normalize_verbosity(
        verbosity,
        has_arg(&args, "--deep-trace") || file_config.deep_trace.unwrap_or(false),
    );
    let speed = if has_arg(&args, "--speed") {
        arg(&args, "--speed", "max")
    } else {
        file_config.speed.unwrap_or_else(|| "max".into())
    };
    let pace_ms = if has_arg(&args, "--pace-ms") {
        arg(&args, "--pace-ms", "250").parse().unwrap_or(250)
    } else {
        file_config.pace_ms.unwrap_or(250)
    };
    let max_turns: u32 = if has_arg(&args, "--max-turns") {
        arg(&args, "--max-turns", "0").parse().unwrap_or(0)
    } else {
        file_config.max_turns.unwrap_or(0)
    };
    let min_stamina = if has_arg(&args, "--min-stamina") {
        arg(&args, "--min-stamina", "10").parse().unwrap_or(10.0)
    } else {
        file_config.min_stamina.unwrap_or(10.0)
    }
    .max(0.0);
    let fake_results = if has_arg(&args, "--fake-results") {
        true
    } else {
        file_config.fake_results.unwrap_or(false)
    };
    let too_easy_below: u32 = if has_arg(&args, "--too-easy-below-days") {
        arg(&args, "--too-easy-below-days", "730")
            .parse()
            .unwrap_or(730)
    } else {
        file_config.too_easy_below_days.unwrap_or(730)
    };
    let hard_above: u32 = if has_arg(&args, "--hard-above-days") {
        arg(&args, "--hard-above-days", "5475")
            .parse()
            .unwrap_or(5475)
    } else {
        file_config.hard_above_days.unwrap_or(5475)
    };
    let near_impossible_above: u32 = if has_arg(&args, "--near-impossible-above-days") {
        arg(&args, "--near-impossible-above-days", "7300")
            .parse()
            .unwrap_or(7300)
    } else {
        file_config.near_impossible_above_days.unwrap_or(7300)
    };
    let overrides = if has_arg(&args, "--override") {
        values(&args, "--override")
    } else {
        file_config.overrides.unwrap_or_default()
    };
    let outcome_values = if has_arg(&args, "--outcome") {
        values(&args, "--outcome")
    } else {
        file_config.outcomes.unwrap_or_default()
    };
    let outcome_rules = outcome_values
        .into_iter()
        .filter_map(|value| {
            let (selector, mode) = value.split_once('=')?;
            Some(OutcomeRule {
                selector: selector.into(),
                fixed_rank: mode
                    .strip_prefix("fixed:")
                    .and_then(|rank| rank.parse().ok()),
            })
        })
        .collect::<Vec<_>>();
    let log_path = if has_arg(&args, "--log") {
        arg(&args, "--log", "")
    } else {
        file_config.log.unwrap_or_default()
    };
    let logs = file_config
        .logs
        .unwrap_or_default()
        .into_iter()
        .map(|value| value.to_ascii_lowercase().replace('-', "_"))
        .filter(|value| {
            matches!(
                value.as_str(),
                "player" | "player_objects" | "available_events" | "decision"
            )
        })
        .collect::<HashSet<_>>();
    let output_path = if has_arg(&args, "--output") {
        arg(&args, "--output", "")
    } else {
        file_config.output.unwrap_or_default()
    };
    let unique_paths = if has_arg(&args, "--unique-paths") {
        true
    } else {
        file_config.unique_paths.unwrap_or(false)
    };
    let file = if log_path.is_empty() {
        None
    } else {
        Some(
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(log_path)
                .expect("open log file"),
        )
    };
    let mut reporter = Reporter {
        file,
        verbosity: verbosity.clone(),
        logs,
    };
    let mut records = Vec::new();
    for offset in 0..runs {
        let turn_cap = if max_turns == 0 {
            max_days.saturating_mul(2).max(1)
        } else {
            max_turns
        };
        let config = RunConfig {
            dataset: &dataset,
            strategy_kind: strategy,
            goal: &goal,
            max_days,
            overrides: &overrides,
            outcome_rules: &outcome_rules,
            speed: &speed,
            pace_ms,
            max_turns: turn_cap,
            min_stamina,
            fake_results,
        };
        let mut seed = seed_base.wrapping_add(offset as u64);
        let mut retries = 0;
        let record = loop {
            let record = run_one(seed, &config, &mut reporter);
            let duplicate = unique_paths
                && records
                    .iter()
                    .any(|previous: &RunRecord| previous.path == record.path);
            if !duplicate || !unique_paths || retries >= 100 {
                break record;
            }
            seed = seed.wrapping_add(runs as u64);
            retries += 1;
        };
        reporter.write(
            "run",
            &format!(
                "seed={} outcome={} days={}",
                record.seed, record.outcome, record.days
            ),
        );
        records.push(record);
    }
    let wins: Vec<_> = records
        .iter()
        .filter(|record| record.outcome == "goal")
        .collect();
    let win_days = wins.iter().map(|record| record.days).collect::<Vec<_>>();
    let win_rate = wins.len() as f64 / records.len() as f64;
    let deaths = records
        .iter()
        .filter(|record| record.outcome == "dead_money" || record.outcome == "dead_stamina")
        .count();
    let timeouts = records
        .iter()
        .filter(|record| record.outcome == "timeout")
        .count();
    let mean_days = if win_days.is_empty() {
        0.0
    } else {
        win_days.iter().map(|days| *days as f64).sum::<f64>() / win_days.len() as f64
    };
    let difficulty =
        if win_days.is_empty() || percentile(win_days.clone(), 0.9) >= near_impossible_above {
            "near-impossible"
        } else if percentile(win_days.clone(), 0.5) >= hard_above {
            "hard"
        } else if percentile(win_days.clone(), 0.5) <= too_easy_below {
            "too-easy"
        } else {
            "moderate"
        };
    let summary = format!(
        "runs={} wins={} win_rate={:.1}% deaths={} death_rate={:.1}% dead_money={} dead_stamina={} timeouts={} timeout_rate={:.1}% days_min={} days_mean={:.1} days_p10={} days_p50={} days_p90={} days_max={} difficulty={}",
        records.len(), wins.len(), win_rate * 100.0,
        deaths, deaths as f64 / records.len() as f64 * 100.0,
        records.iter().filter(|record| record.outcome == "dead_money").count(),
        records.iter().filter(|record| record.outcome == "dead_stamina").count(),
        timeouts, timeouts as f64 / records.len() as f64 * 100.0,
        win_days.iter().min().copied().unwrap_or(0), mean_days,
        percentile(win_days.clone(), 0.1), percentile(win_days.clone(), 0.5),
        percentile(win_days.clone(), 0.9), win_days.iter().max().copied().unwrap_or(0), difficulty
    );
    reporter.write("summary", &summary);
    if !output_path.is_empty() {
        std::fs::write(
            output_path,
            serde_json::to_string_pretty(&records).expect("serialize run records"),
        )
        .expect("write run records");
    }
}
