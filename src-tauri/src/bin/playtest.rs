use serde::Serialize;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ttrpg_engine_lib::{
    apply_override, apply_event, advance_day, buy_object_for_sim, eligible_event_entries,
    enter_event_for_sim, legal_event_ids, legal_encounter_action_ids, new_game_seeded, roll, run_status,
    resolve_encounter_for_sim,
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

#[derive(Clone, Copy)]
enum Decision {
    Action(usize),
    Event(usize),
    Purchase(usize),
    Wait,
}

trait Strategy {
    fn choose(&mut self, game: &mut GameState, actions: &[String], events: &[(String, String)], purchases: &[String]) -> Decision;
}

struct RandomStrategy;
struct GreedyStrategy;
struct RequiredStrategy;

impl Strategy for RandomStrategy {
    fn choose(&mut self, game: &mut GameState, actions: &[String], events: &[(String, String)], purchases: &[String]) -> Decision {
        let total = actions.len() + events.len() + purchases.len();
        if total == 0 { return Decision::Wait; }
        let pick = (roll(game) * total as f64).floor() as usize % total;
        if pick < events.len() { Decision::Event(pick) }
        else if pick < events.len() + actions.len() { Decision::Action(pick - events.len()) }
        else { Decision::Purchase(pick - events.len() - actions.len()) }
    }
}

impl Strategy for GreedyStrategy {
    fn choose(&mut self, game: &mut GameState, actions: &[String], events: &[(String, String)], purchases: &[String]) -> Decision {
        if !events.is_empty() { return Decision::Event(0); }
        if let Some((index, _)) = actions.iter().enumerate().max_by(|(_, left), (_, right)| {
            action_value(game, left).partial_cmp(&action_value(game, right)).unwrap_or(std::cmp::Ordering::Equal)
        }) {
            return Decision::Action(index);
        }
        purchases.first().map(|_| Decision::Purchase(0)).unwrap_or(Decision::Wait)
    }
}

impl Strategy for RequiredStrategy {
    fn choose(&mut self, _game: &mut GameState, _actions: &[String], events: &[(String, String)], purchases: &[String]) -> Decision {
        if !events.is_empty() { Decision::Event(0) }
        else if !purchases.is_empty() { Decision::Purchase(0) }
        else { Decision::Wait }
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
}

struct Reporter {
    file: Option<std::fs::File>,
    verbosity: String,
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
            let _ = writeln!(file, "{message}");
        }
    }
}

fn arg(args: &[String], name: &str, default: &str) -> String {
    args.windows(2).find(|pair| pair[0] == name).map(|pair| pair[1].clone()).unwrap_or_else(|| default.into())
}

fn values(args: &[String], name: &str) -> Vec<String> {
    args.windows(2).filter(|pair| pair[0] == name).map(|pair| pair[1].clone()).collect()
}

fn metric(game: &GameState, id: &str) -> f64 {
    game.player.characteristics.get(id).copied().unwrap_or(0.0)
}

fn parse_strategy(value: &str) -> StrategyKind {
    match value.to_ascii_lowercase().as_str() {
        "greedy" => StrategyKind::Greedy,
        "required" | "required-only" => StrategyKind::Required,
        _ => StrategyKind::Random,
    }
}

fn action_value(game: &GameState, id: &str) -> f64 {
    game.catalog.events.iter().find(|action| action.id == id)
        .map(|action| action.success_rate * action.payout - action.base_cost - action.stamina_cost * 10.0)
        .unwrap_or(f64::MIN)
}

fn purchase_candidates(game: &GameState) -> Vec<String> {
    let mut candidates: Vec<_> = game.catalog.objects.iter()
        .filter(|object| object.object_type == "vehicle")
        .filter(|object| object.price <= metric(game, "budget"))
        .filter(|object| !game.player.inventory.iter().any(|owned| owned.id == object.id || owned.id.starts_with(&format!("{}_", object.id))))
        .map(|object| (object.id.clone(), object.price))
        .collect();
    candidates.sort_by(|left, right| left.1.partial_cmp(&right.1).unwrap_or(std::cmp::Ordering::Equal));
    candidates.into_iter().map(|(id, _)| id).take(8).collect()
}

fn outcome_for(game: &mut GameState, event_id: &str, rules: &[OutcomeRule]) -> (String, Option<u32>) {
    let event = game.catalog.events.iter().find(|event| event.id == event_id);
    let event_type = event.map(|event| event.event_type.as_str()).unwrap_or("event");
    let rule = rules.iter().find(|rule| rule.selector == format!("event:{event_id}"))
        .or_else(|| rules.iter().find(|rule| rule.selector == format!("type:{event_type}")));
    if let Some(rule) = rule {
        if let Some(rank) = rule.fixed_rank { return ("success".into(), Some(rank)); }
    }
    if roll(game) < 0.5 {
        ("success".into(), Some((roll(game) * 5.0).floor() as u32 + 1))
    } else {
        ("failure".into(), None)
    }
}

fn run_one(
    dataset: &str,
    seed: u64,
    strategy_kind: StrategyKind,
    goal: (&str, f64),
    max_days: u32,
    overrides: &[String],
    outcome_rules: &[OutcomeRule],
    speed: &str,
    pace_ms: u64,
    reporter: &mut Reporter,
) -> RunRecord {
    let mut game = new_game_seeded(dataset, seed);
    for override_value in overrides {
        if let Some((path, value)) = override_value.split_once('=') {
            if let Err(error) = apply_override(&mut game, path, value) {
                reporter.write("run", &format!("seed={seed} override_error={error}"));
            }
        }
    }
    let mut strategy: Box<dyn Strategy> = match strategy_kind {
        StrategyKind::Greedy => Box::new(GreedyStrategy),
        StrategyKind::Required => Box::new(RequiredStrategy),
        StrategyKind::Random => Box::new(RandomStrategy),
    };
    let mut steps = 0;
    loop {
        let status = run_status(&game, goal.0, goal.1, max_days);
        if status != RunStatus::Ongoing { break; }
        let day_before = game.current_day;
        if game.active_encounter.is_some() {
            let encounter_actions = legal_encounter_action_ids(&game);
            let action_id = encounter_actions.first().map(String::as_str);
            if let Err(error) = resolve_encounter_for_sim(&mut game, action_id) {
                reporter.write("run", &format!("seed={seed} encounter_error={error}"));
                break;
            }
            reporter.write("trace", &format!("seed={seed} day={} encounter_action={}", game.current_day, action_id.unwrap_or("pass")));
        } else {
        let actions = legal_event_ids(&game);
        let entries = eligible_event_entries(&game);
        let purchases = if entries.is_empty() { purchase_candidates(&game) } else { vec![] };
        match strategy.choose(&mut game, &actions, &entries, &purchases) {
            Decision::Event(index) => {
                let (event_id, object_id) = &entries[index];
                if enter_event_for_sim(&mut game, event_id, object_id).is_ok() {
                    if let Some(pending) = game.pending_events.iter().rev().find(|entry| entry.event_id == *event_id) {
                        let pending_id = pending.id.clone();
                        let (result, position) = outcome_for(&mut game, event_id, outcome_rules);
                        let _ = submit_event_for_sim_with_details(&mut game, &pending_id, &result, position);
                        reporter.write("trace", &format!("seed={seed} day={} event={} result={}", game.current_day, event_id, result));
                    }
                }
            }
            Decision::Action(index) => {
                let action_id = &actions[index];
                if let Ok(result) = apply_event(&mut game, action_id) {
                    reporter.write("trace", &format!("seed={seed} day={} action={} success={}", game.current_day, action_id, result.success));
                }
            }
            Decision::Purchase(index) => {
                let object_id = &purchases[index];
                if buy_object_for_sim(&mut game, object_id).is_ok() {
                    reporter.write("trace", &format!("seed={seed} day={} purchase={}", game.current_day, object_id));
                }
            }
            Decision::Wait => {
                reporter.write("trace", &format!("seed={seed} day={} wait", game.current_day));
            }
        }
        }
        steps += 1;
        if speed == "paced" { sleep(Duration::from_millis(pace_ms)); }
        if steps > max_days.saturating_mul(2).max(1) { break; }
        if game.current_day == day_before && game.active_encounter.is_none() {
            if let Err(error) = advance_day(&mut game) {
                reporter.write("run", &format!("seed={seed} advance_error={error}"));
                break;
            }
        }
    }
    let status = run_status(&game, goal.0, goal.1, max_days);
    let outcome = match status {
        RunStatus::GoalReached => "goal",
        RunStatus::DeadMoney => "dead_money",
        RunStatus::DeadStamina => "dead_stamina",
        RunStatus::MaxDays => "timeout",
        RunStatus::Ongoing => "timeout",
    };
    RunRecord { seed, outcome: outcome.into(), days: game.current_day.saturating_sub(1), goal_value: metric(&game, goal.0), budget: metric(&game, "budget"), stamina: metric(&game, "stamina"), steps }
}

fn percentile(mut values: Vec<u32>, fraction: f64) -> u32 {
    if values.is_empty() { return 0; }
    values.sort_unstable();
    values[((values.len() - 1) as f64 * fraction).round() as usize]
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dataset = arg(&args, "--dataset", "dataset");
    let runs: u32 = arg(&args, "--runs", "1").parse().unwrap_or(1).max(1);
    let max_days: u32 = arg(&args, "--max-days", "7300").parse().unwrap_or(7300);
    let strategy = parse_strategy(&arg(&args, "--strategy", "greedy"));
    let goal_text = arg(&args, "--goal", "charisma>=100");
    let (goal_id, goal_target) = goal_text.split_once(">=").map(|(id, target)| (id.trim(), target.trim().parse().unwrap_or(100.0))).unwrap_or(("charisma", 100.0));
    let seed_base = arg(&args, "--seed", "").parse().unwrap_or_else(|_| SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64);
    let verbosity = arg(&args, "--verbosity", "summary");
    let speed = arg(&args, "--speed", "max");
    let pace_ms = arg(&args, "--pace-ms", "250").parse().unwrap_or(250);
    let too_easy_below: u32 = arg(&args, "--too-easy-below-days", "730").parse().unwrap_or(730);
    let hard_above: u32 = arg(&args, "--hard-above-days", "5475").parse().unwrap_or(5475);
    let near_impossible_above: u32 = arg(&args, "--near-impossible-above-days", "7300").parse().unwrap_or(7300);
    let overrides = values(&args, "--override");
    let outcome_rules = values(&args, "--outcome").into_iter().filter_map(|value| {
        let (selector, mode) = value.split_once('=')?;
        Some(OutcomeRule { selector: selector.into(), fixed_rank: mode.strip_prefix("fixed:").and_then(|rank| rank.parse().ok()) })
    }).collect::<Vec<_>>();
    let log_path = arg(&args, "--log", "");
    let file = if log_path.is_empty() { None } else {
        Some(OpenOptions::new().create(true).truncate(true).write(true).open(log_path).expect("open log file"))
    };
    let mut reporter = Reporter { file, verbosity: verbosity.clone() };
    let mut records = Vec::new();
    for offset in 0..runs {
        let record = run_one(&dataset, seed_base.wrapping_add(offset as u64), strategy, (goal_id, goal_target), max_days, &overrides, &outcome_rules, &speed, pace_ms, &mut reporter);
        reporter.write("run", &format!("seed={} outcome={} days={}", record.seed, record.outcome, record.days));
        records.push(record);
    }
    let wins: Vec<_> = records.iter().filter(|record| record.outcome == "goal").collect();
    let win_days = wins.iter().map(|record| record.days).collect::<Vec<_>>();
    let win_rate = wins.len() as f64 / records.len() as f64;
    let difficulty = if win_days.is_empty() || percentile(win_days.clone(), 0.9) >= near_impossible_above { "near-impossible" }
        else if percentile(win_days.clone(), 0.5) >= hard_above { "hard" }
        else if percentile(win_days.clone(), 0.5) <= too_easy_below { "too-easy" }
        else { "moderate" };
    let summary = format!(
        "runs={} wins={} win_rate={:.1}% dead_money={} dead_stamina={} timeouts={} days_min={} days_p50={} days_p90={} days_max={} difficulty={}",
        records.len(), wins.len(), win_rate * 100.0,
        records.iter().filter(|record| record.outcome == "dead_money").count(),
        records.iter().filter(|record| record.outcome == "dead_stamina").count(),
        records.iter().filter(|record| record.outcome == "timeout").count(),
        win_days.iter().min().copied().unwrap_or(0), percentile(win_days.clone(), 0.5),
        percentile(win_days.clone(), 0.9), win_days.iter().max().copied().unwrap_or(0), difficulty
    );
    reporter.write("summary", &summary);
    if let Some(output) = args.windows(2).find(|pair| pair[0] == "--output").map(|pair| pair[1].clone()) {
        std::fs::write(output, serde_json::to_string_pretty(&records).expect("serialize run records")).expect("write run records");
    }
}
