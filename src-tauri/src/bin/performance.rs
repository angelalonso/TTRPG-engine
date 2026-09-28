//! Low-overhead performance probe for the engine.
//!
//! This binary measures the existing public engine operations from outside the
//! application. It does not add timers, logging, or allocations to production
//! code paths.

use serde::Serialize;
use std::env;
use std::fs;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use ttrpg_engine_lib::{
    eligible_event_entries, engine::loader::GameCatalog, enter_event_for_sim,
    legal_encounter_action_ids, legal_event_ids, new_game_seeded,
    submit_event_for_sim_with_details,
};

#[derive(Debug)]
struct Config {
    dataset: String,
    iterations: usize,
    warmup: usize,
    days: usize,
    output: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dataset: "dataset".into(),
            iterations: 10,
            warmup: 1,
            days: 365,
            output: Some("performance-results.json".into()),
        }
    }
}

#[derive(Debug, Serialize)]
struct Metric {
    count: usize,
    errors: usize,
    total_ns: u128,
    min_ns: u128,
    mean_ns: f64,
    median_ns: u128,
    p95_ns: u128,
    p99_ns: u128,
    max_ns: u128,
    operations_per_second: f64,
    samples_ns: Vec<u128>,
}

#[derive(Debug, Default)]
struct MetricBuilder {
    samples_ns: Vec<u128>,
    errors: usize,
}

impl MetricBuilder {
    fn record(&mut self, duration: Duration, error: bool) {
        self.samples_ns.push(duration.as_nanos());
        self.errors += usize::from(error);
    }

    fn finish(mut self) -> Metric {
        self.samples_ns.sort_unstable();
        let count = self.samples_ns.len();
        let total_ns = self.samples_ns.iter().sum();
        let percentile = |percent: usize| -> u128 {
            if count == 0 {
                return 0;
            }
            let index = ((count - 1) * percent).div_ceil(100);
            self.samples_ns[index.min(count - 1)]
        };
        Metric {
            count,
            errors: self.errors,
            total_ns,
            min_ns: self.samples_ns.first().copied().unwrap_or(0),
            mean_ns: if count == 0 {
                0.0
            } else {
                total_ns as f64 / count as f64
            },
            median_ns: percentile(50),
            p95_ns: percentile(95),
            p99_ns: percentile(99),
            max_ns: self.samples_ns.last().copied().unwrap_or(0),
            operations_per_second: if total_ns == 0 {
                0.0
            } else {
                count as f64 * 1_000_000_000.0 / total_ns as f64
            },
            samples_ns: self.samples_ns,
        }
    }
}

#[derive(Debug, Serialize)]
struct PerformanceReport {
    generated_at_unix_seconds: u64,
    dataset: String,
    iterations: usize,
    warmup: usize,
    days_per_iteration: usize,
    process_id: u32,
    peak_rss_bytes: Option<u64>,
    counters: Counters,
    metrics: std::collections::BTreeMap<String, Metric>,
}

#[derive(Debug, Default, Serialize)]
struct Counters {
    initialized_games: usize,
    advanced_days: usize,
    event_entries: usize,
    event_submissions: usize,
    encounter_queries: usize,
    largest_serialized_state_bytes: usize,
}

fn parse_args() -> Config {
    let mut config = Config::default();
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        let value = |name: &str, args: &mut std::iter::Skip<env::Args>| {
            args.next()
                .unwrap_or_else(|| panic!("missing value for {name}"))
        };
        match argument.as_str() {
            "--dataset" => config.dataset = value("--dataset", &mut args),
            "--iterations" => {
                config.iterations = value("--iterations", &mut args)
                    .parse()
                    .expect("--iterations must be a positive integer")
            }
            "--warmup" => {
                config.warmup = value("--warmup", &mut args)
                    .parse()
                    .expect("--warmup must be a non-negative integer")
            }
            "--days" => {
                config.days = value("--days", &mut args)
                    .parse()
                    .expect("--days must be a non-negative integer")
            }
            "--output" => config.output = Some(value("--output", &mut args)),
            "--no-output" => config.output = None,
            "--help" | "-h" => {
                println!(
                    "Usage: cargo run --manifest-path src-tauri/Cargo.toml --bin performance -- [OPTIONS]\n\
                     \n\
                     --dataset PATH       Dataset directory (default: dataset)\n\
                     --iterations N       Measured game iterations (default: 10)\n\
                     --warmup N           Unmeasured warmup iterations (default: 1)\n\
                     --days N             Days advanced per iteration (default: 365)\n\
                     --output PATH        JSON report path (default: performance-results.json)\n\
                     --no-output          Do not write a JSON report"
                );
                std::process::exit(0);
            }
            unknown => panic!("unknown option '{unknown}', use --help"),
        }
    }
    assert!(config.iterations > 0, "--iterations must be positive");
    config
}

fn current_rss_bytes() -> Option<u64> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("VmHWM:"))?;
    line.split_whitespace()
        .nth(1)?
        .parse::<u64>()
        .ok()
        .map(|kb| kb * 1024)
}

fn run_iteration(
    dataset: &str,
    seed: u64,
    days: usize,
    metrics: &mut std::collections::BTreeMap<String, MetricBuilder>,
    counters: &mut Counters,
) {
    GameCatalog::clear_cached_directory(dataset);
    let cold_catalog_start = Instant::now();
    let _ = GameCatalog::load_from_directory_cached(dataset);
    metrics
        .entry("cold_catalog_loading".into())
        .or_default()
        .record(cold_catalog_start.elapsed(), false);

    let init_start = Instant::now();
    let mut game = new_game_seeded(dataset, seed);
    metrics
        .entry("game_initialization".into())
        .or_default()
        .record(init_start.elapsed(), false);
    counters.initialized_games += 1;

    for _ in 0..days {
        let advance = metrics.entry("advance_day".into()).or_default();
        let start = Instant::now();
        let error = ttrpg_engine_lib::advance_day(&mut game).is_err();
        advance.record(start.elapsed(), error);
        counters.advanced_days += 1;

        let legal = metrics.entry("legal_event_ids".into()).or_default();
        let start = Instant::now();
        let _ = legal_event_ids(&game);
        legal.record(start.elapsed(), false);

        let eligible = metrics.entry("eligible_event_entries".into()).or_default();
        let start = Instant::now();
        let entries = eligible_event_entries(&game);
        eligible.record(start.elapsed(), false);

        if let Some((event_id, object_id)) = entries.first() {
            let enter = metrics.entry("enter_event".into()).or_default();
            let start = Instant::now();
            let enter_result = enter_event_for_sim(&mut game, event_id, object_id);
            enter.record(start.elapsed(), enter_result.is_err());
            if enter_result.is_ok() {
                counters.event_entries += 1;
                if let Some(entry) = game.pending_events.last().cloned() {
                    let submit = metrics.entry("submit_event_result".into()).or_default();
                    let start = Instant::now();
                    let submit_result =
                        submit_event_for_sim_with_details(&mut game, &entry.id, "success", None);
                    submit.record(start.elapsed(), submit_result.is_err());
                    if submit_result.is_ok() {
                        counters.event_submissions += 1;
                    }
                }
            }
        }

        let encounter = metrics
            .entry("legal_encounter_action_ids".into())
            .or_default();
        let start = Instant::now();
        let _ = legal_encounter_action_ids(&game);
        encounter.record(start.elapsed(), false);
    }

    let serialization = metrics
        .entry("serialize_game_state_json".into())
        .or_default();
    let start = Instant::now();
    let result = serde_json::to_vec(&game);
    let serialized_size = result.as_ref().map(Vec::len).unwrap_or(0);
    counters.largest_serialized_state_bytes =
        counters.largest_serialized_state_bytes.max(serialized_size);
    serialization.record(start.elapsed(), result.is_err());

    if game.pending_events.is_empty() {
        counters.encounter_queries += 1;
    }
}

fn print_report(report: &PerformanceReport) {
    println!(
        "Performance report: dataset={} iterations={} days/iteration={} peak_rss={}",
        report.dataset,
        report.iterations,
        report.days_per_iteration,
        report
            .peak_rss_bytes
            .map(|bytes| format!("{bytes} bytes"))
            .unwrap_or_else(|| "unavailable".into())
    );
    println!(
        "{:<32} {:>8} {:>12} {:>12} {:>12} {:>12} {:>12}",
        "operation", "count", "mean us", "median us", "p95 us", "max us", "ops/sec"
    );
    for (name, metric) in &report.metrics {
        println!(
            "{:<32} {:>8} {:>12.3} {:>12.3} {:>12.3} {:>12.3} {:>12.1}",
            name,
            metric.count,
            metric.mean_ns / 1_000.0,
            metric.median_ns as f64 / 1_000.0,
            metric.p95_ns as f64 / 1_000.0,
            metric.max_ns as f64 / 1_000.0,
            metric.operations_per_second
        );
    }
    println!(
        "Counters: games={} days={} entries={} submissions={} encounter_queries={}",
        report.counters.initialized_games,
        report.counters.advanced_days,
        report.counters.event_entries,
        report.counters.event_submissions,
        report.counters.encounter_queries
    );
    println!(
        "Largest serialized state: {} bytes",
        report.counters.largest_serialized_state_bytes
    );
}

fn main() {
    let config = parse_args();
    for warmup in 0..config.warmup {
        let mut metrics = std::collections::BTreeMap::new();
        let mut counters = Counters::default();
        run_iteration(
            &config.dataset,
            warmup as u64,
            config.days,
            &mut metrics,
            &mut counters,
        );
    }

    let mut metrics = std::collections::BTreeMap::new();
    let mut counters = Counters::default();
    for iteration in 0..config.iterations {
        run_iteration(
            &config.dataset,
            iteration as u64 + config.warmup as u64,
            config.days,
            &mut metrics,
            &mut counters,
        );
    }
    let report = PerformanceReport {
        generated_at_unix_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_secs(),
        dataset: config.dataset,
        iterations: config.iterations,
        warmup: config.warmup,
        days_per_iteration: config.days,
        process_id: std::process::id(),
        peak_rss_bytes: current_rss_bytes(),
        counters,
        metrics: metrics
            .into_iter()
            .map(|(name, metric)| (name, metric.finish()))
            .collect(),
    };
    print_report(&report);
    if let Some(path) = config.output {
        let json = serde_json::to_vec_pretty(&report).expect("serialize performance report");
        fs::write(&path, json).unwrap_or_else(|error| panic!("write report '{path}': {error}"));
        println!("Detailed JSON report written to {path}");
    }
}
