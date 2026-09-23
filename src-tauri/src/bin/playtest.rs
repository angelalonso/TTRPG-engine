use serde::{Deserialize, Serialize};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ttrpg_engine_lib::{
    advance_day, apply_event, apply_override, buy_object_for_sim, eligible_event_entries,
    enter_event_for_sim, join_quest_for_sim, legal_encounter_action_ids, legal_event_ids,
    new_game_seeded, resolve_encounter_for_sim, roll, run_status,
    submit_event_for_sim_with_details, GameState, RunStatus,
};

#[derive(Clone, Copy)]
enum StrategyKind {
    Random,
    Greedy,
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
    speed: Option<String>,
    pace_ms: Option<u64>,
    overrides: Option<Vec<String>>,
    outcomes: Option<Vec<String>>,
    output: Option<String>,
    log: Option<String>,
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
    ) -> Decision;
}

struct RandomStrategy;
struct GreedyStrategy;
struct RequiredStrategy;

impl Strategy for RandomStrategy {
    fn choose(
        &mut self,
        game: &mut GameState,
        actions: &[String],
        events: &[(String, String)],
        purchases: &[String],
        quests: &[String],
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

impl Strategy for RequiredStrategy {
    fn choose(
        &mut self,
        _game: &mut GameState,
        _actions: &[String],
        events: &[(String, String)],
        purchases: &[String],
        quests: &[String],
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
}

impl Reporter {
    fn write(&mut self, level: &str, message: &str) {
        if (self.verbosity == "summary" && level != "summary")
            || (self.verbosity == "run" && level == "trace")
        {
            return;
        }
        println!("{message}");
        if let Some(file) = &mut self.file {
            if let Err(error) = writeln!(file, "{message}") {
                eprintln!("playtest log write failed: {error}");
            }
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
    format!(
        "state day={} rng={} characteristics=[{}] inventory={inventory:?} pending={pending:?} active_encounter={}",
        game.current_day,
        game.rng_state,
        characteristics.join(","),
        game.active_encounter.is_some()
    )
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
        "required" | "required-only" => StrategyKind::Required,
        _ => StrategyKind::Random,
    }
}

fn action_value(game: &GameState, id: &str) -> f64 {
    game.catalog
        .events
        .iter()
        .find(|action| action.id == id)
        .map(|action| {
            action.success_rate * action.payout - action.base_cost - action.stamina_cost * 10.0
        })
        .unwrap_or(f64::MIN)
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
    let action_cost = if event.event_type.eq_ignore_ascii_case("work")
        && ((game.current_day.saturating_sub(1) % 7) + 1) <= 5
    {
        game.catalog
            .labels
            .values
            .get("work_day_stamina_cost")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(30.0)
    } else {
        0.0
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
    action_cost + event_cost
}

fn stamina_safe_actions(game: &GameState, actions: Vec<String>, min_stamina: f64) -> Vec<String> {
    let stamina = metric(game, "stamina");
    actions
        .into_iter()
        .filter(|id| stamina - event_stamina_cost(game, id) >= min_stamina)
        .collect()
}

fn stamina_safe_entries(
    game: &GameState,
    entries: Vec<(String, String)>,
    min_stamina: f64,
) -> Vec<(String, String)> {
    let stamina = metric(game, "stamina");
    entries
        .into_iter()
        .filter(|(event_id, _)| stamina - event_stamina_cost(game, event_id) >= min_stamina)
        .collect()
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
    candidates.into_iter().map(|(id, _)| id).take(8).collect()
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
        if game.active_encounter.is_some() {
            let encounter_actions = legal_encounter_action_ids(&game);
            let action_id = encounter_actions.first().map(String::as_str);
            if let Err(error) = resolve_encounter_for_sim(&mut game, action_id) {
                reporter.write("run", &format!("seed={seed} encounter_error={error}"));
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
            let actions = stamina_safe_actions(&game, legal_event_ids(&game), config.min_stamina);
            let entries =
                stamina_safe_entries(&game, eligible_event_entries(&game), config.min_stamina);
            let purchases = purchase_candidates(&game);
            let quests = if entries.is_empty() {
                quest_candidates(&game)
            } else {
                vec![]
            };
            reporter.write(
                "trace",
                &format!(
                    "seed={seed} day={} possibilities=events:{:?};actions:{:?};purchases:{:?};quests:{:?}",
                    game.current_day, entries, actions, purchases, quests
                ),
            );
            let rng_before_decision = game.rng_state;
            let decision = strategy.choose(&mut game, &actions, &entries, &purchases, &quests);
            reporter.write(
                "deep-trace",
                &format!(
                    "seed={seed} {} candidates events={entries:?} actions={actions:?} purchases={purchases:?} quests={quests:?} decision={decision:?} rng_before={rng_before_decision} rng_after={}",
                    deep_trace_state(&game),
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
                                let (result, position) =
                                    outcome_for(&mut game, event_id, config.outcome_rules);
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
                        Err(error) => reporter.write(
                            "run",
                            &format!(
                                "seed={seed} event={} object={} enter_error={error}",
                                event_id, object_id
                            ),
                        ),
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
                        Err(error) => reporter.write(
                            "run",
                            &format!("seed={seed} action={action_id} apply_error={error}"),
                        ),
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
                        Err(error) => reporter.write(
                            "run",
                            &format!("seed={seed} purchase={object_id} buy_error={error}"),
                        ),
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
                        Err(error) => reporter.write(
                            "run",
                            &format!("seed={seed} quest={quest_id} join_error={error}"),
                        ),
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
