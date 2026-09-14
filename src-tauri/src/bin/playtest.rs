use economy_engine_lib::{advance_day, apply_action, legal_action_ids, new_game, GameState};
use serde::Serialize;
use std::env;

#[derive(Serialize, Clone)]
struct Run {
    run: usize, seed: u64, outcome: String, days: u32, budget: f64, stamina: f64,
    charisma: f64, loss_cause: String, actions: String,
}

fn arg(args: &[String], name: &str, default: &str) -> String {
    args.windows(2).find(|w| w[0] == name).map(|w| w[1].clone()).unwrap_or_else(|| default.into())
}
fn metric(state: &GameState, stat: &str) -> f64 { state.player.characteristics.get(stat).copied().unwrap_or(0.0) }
fn percentile(rows: &[Run], outcome: &str, p: f64) -> u32 {
    let mut values: Vec<u32> = rows.iter().filter(|r| r.outcome == outcome).map(|r| r.days).collect();
    if values.is_empty() { return 0; }
    values.sort_unstable();
    values[((values.len() - 1) as f64 * p).round() as usize]
}
trait Policy { fn choose(&mut self, game: &GameState, actions: &[String]) -> usize; }
struct RandomPolicy { rng: u64 }
struct GreedyPolicy;
impl Policy for RandomPolicy {
    fn choose(&mut self, _: &GameState, actions: &[String]) -> usize {
        self.rng = self.rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.rng as usize) % actions.len()
    }
}
impl Policy for GreedyPolicy {
    fn choose(&mut self, game: &GameState, actions: &[String]) -> usize {
        actions.iter().enumerate().max_by(|(_, a), (_, b)| {
            let score = |id: &String| game.catalog.actions.iter().find(|x| &x.id == id).map(|x| x.payout - x.base_cost).unwrap_or(0.0);
            score(a).partial_cmp(&score(b)).unwrap_or(std::cmp::Ordering::Equal)
        }).map(|(i, _)| i).unwrap_or(0)
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let runs: usize = arg(&args, "--runs", "100").parse().unwrap_or(100);
    let policy = arg(&args, "--policy", "random");
    let dataset = arg(&args, "--dataset", "dataset");
    let max_days: u32 = arg(&args, "--max-days", "2000").parse().unwrap_or(2000);
    let seed: u64 = arg(&args, "--seed", "1").parse().unwrap_or(1);
    let win_stat = arg(&args, "--win-stat", "charisma");
    let win_target: f64 = arg(&args, "--win-target", "1000").parse().unwrap_or(1000.0);
    let output = arg(&args, "--output", "");
    let mut rows = Vec::with_capacity(runs);
    for run in 0..runs {
        let run_seed = seed.wrapping_add(run as u64);
        let mut policy_impl: Box<dyn Policy> = if policy.eq_ignore_ascii_case("greedy") {
            Box::new(GreedyPolicy)
        } else { Box::new(RandomPolicy { rng: run_seed ^ 0x9e3779b97f4a7c15 }) };
        let mut game = new_game(dataset.clone());
        let mut history = Vec::new();
        let mut outcome = "timeout".to_string();
        while game.current_day.saturating_sub(1) < max_days {
            if metric(&game, &win_stat) >= win_target { outcome = "win".into(); break; }
            let budget = metric(&game, "budget");
            let stamina = metric(&game, "stamina");
            if budget < 0.0 && stamina == 0.0 { outcome = "loss".into(); break; }
            let ids = legal_action_ids(&game);
            if !ids.is_empty() {
                let idx = policy_impl.choose(&game, &ids);
                let id = ids[idx].clone();
                if apply_action(&mut game, &id).is_ok() { history.push(id); }
            }
            if advance_day(&mut game).is_err() { break; }
        }
        if outcome == "timeout" && metric(&game, &win_stat) >= win_target { outcome = "win".into(); }
        let cause = if metric(&game, "budget") < 0.0 && metric(&game, "stamina") == 0.0 { "both" }
            else if metric(&game, "budget") < 0.0 { "budget" }
            else if metric(&game, "stamina") == 0.0 { "stamina" } else { "" };
        rows.push(Run { run, seed: run_seed, outcome, days: game.current_day.saturating_sub(1),
            budget: metric(&game, "budget"), stamina: metric(&game, "stamina"), charisma: metric(&game, "charisma"),
            loss_cause: cause.into(), actions: history.iter().rev().take(5).cloned().collect::<Vec<_>>().join("|") });
    }
    if !output.is_empty() {
        if output.ends_with(".json") {
            std::fs::write(&output, serde_json::to_string_pretty(&rows).unwrap()).expect("write output");
        } else {
            let mut csv = String::from("run,seed,outcome,days,budget,stamina,charisma,loss_cause,actions\n");
            for r in &rows { csv.push_str(&format!("{},{},{},{},{},{},{},{},{}\n", r.run,r.seed,r.outcome,r.days,r.budget,r.stamina,r.charisma,r.loss_cause,r.actions)); }
            std::fs::write(&output, csv).expect("write output");
        }
    }
    let wins = rows.iter().filter(|r| r.outcome == "win").count();
    let losses = rows.iter().filter(|r| r.outcome == "loss").count();
    println!("runs={} wins={} ({:.1}%) losses={} ({:.1}%) timeouts={} ({:.1}%)",
        runs, wins, wins as f64 * 100.0 / runs.max(1) as f64, losses, losses as f64 * 100.0 / runs.max(1) as f64,
        runs - wins - losses, (runs - wins - losses) as f64 * 100.0 / runs.max(1) as f64);
    println!("days-to-win p10/p50/p90={}/{}/{}; days-to-loss p10/p50/p90={}/{}/{}",
        percentile(&rows, "win", 0.10), percentile(&rows, "win", 0.50), percentile(&rows, "win", 0.90),
        percentile(&rows, "loss", 0.10), percentile(&rows, "loss", 0.50), percentile(&rows, "loss", 0.90));
    for cause in ["budget", "stamina", "both"] {
        let count = rows.iter().filter(|r| r.outcome == "loss" && r.loss_cause == cause).count();
        if count > 0 { println!("loss-cause {}={}", cause, count); }
    }
}
