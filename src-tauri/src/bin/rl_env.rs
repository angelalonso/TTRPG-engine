use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use ttrpg_engine_lib::rl::{
    ActionDefinition, GoalSpec, Observation, RewardConfig, RlConfig, RlEnvironment, StepResult,
};

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum Request {
    Actions,
    Observation,
    Reset { seed: u64 },
    Step { action_id: u32 },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Response {
    Actions { actions: Vec<ActionDefinition> },
    Observation { observation: Observation },
    Step { result: StepResult },
    Error { error: String },
}

fn write_response(response: Response) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &response)?;
    stdout.write_all(b"\n")?;
    stdout.flush()
}

fn main() {
    if let Err(error) = run() {
        eprintln!("RL configuration error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let dataset = argument(&args, "--dataset", "gtr2career");
    let goal = argument(&args, "--goal", "charisma>=100");
    let max_days = argument(&args, "--max-days", "7300")
        .parse::<u32>()
        .map_err(|_| "--max-days must be a positive integer")?;
    let max_steps = argument(&args, "--max-steps", "14600")
        .parse::<u32>()
        .map_err(|_| "--max-steps must be a positive integer")?;
    if max_days == 0 || max_steps == 0 {
        return Err("--max-days and --max-steps must be greater than zero".into());
    }
    let initial_seed = argument(&args, "--seed", "1")
        .parse::<u64>()
        .map_err(|_| "--seed must be a non-negative integer")?;
    let reward_path = argument(&args, "--reward-config", "");
    let reward: RewardConfig = if reward_path.is_empty() {
        RewardConfig::default()
    } else {
        serde_json::from_str(
            &std::fs::read_to_string(&reward_path)
                .map_err(|error| format!("{reward_path}: {error}"))?,
        )
        .map_err(|error| format!("{reward_path}: {error}"))?
    };
    if reward.goal <= 0.0
        || reward.progress_scale < 0.0
        || [
            reward.death,
            reward.day,
            reward.step,
            reward.invalid_action,
            reward.low_stamina_penalty,
            reward.low_budget_penalty,
        ]
        .iter()
        .any(|value| !value.is_finite() || *value > 0.0)
        || !reward.goal.is_finite()
        || !reward.progress_scale.is_finite()
        || !reward.low_stamina_threshold.is_finite()
        || !reward.low_budget_threshold.is_finite()
    {
        return Err("Reward must have a positive goal bonus, non-negative progress scale, finite thresholds and non-positive penalties".into());
    }
    let config = RlConfig {
        goal: GoalSpec::try_parse(&goal)?,
        max_days,
        max_steps,
        reward,
    };
    let mut environment = RlEnvironment::new(dataset, initial_seed, config);
    match &environment.config().goal {
        GoalSpec::Characteristic { id, .. }
            if !environment.observation().characteristics.contains_key(id) =>
        {
            return Err(format!("Unknown characteristic goal '{id}'"))
        }
        GoalSpec::Championship { id }
            if !environment
                .state()
                .catalog
                .quests
                .iter()
                .any(|quest| quest.id == *id) =>
        {
            return Err(format!("Unknown championship goal '{id}'"))
        }
        _ => {}
    }
    if environment.status() != ttrpg_engine_lib::rl::EpisodeStatus::Ongoing {
        return Err(
            "The initial state already ends the episode; choose a goal and limits requiring play"
                .into(),
        );
    }
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            break;
        };
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Request>(&line) {
            Ok(Request::Actions) => Response::Actions {
                actions: environment.actions().to_vec(),
            },
            Ok(Request::Observation) => Response::Observation {
                observation: environment.observation(),
            },
            Ok(Request::Reset { seed }) => {
                let config = environment_config(&environment);
                environment =
                    RlEnvironment::new(environment.state().dataset_path.clone(), seed, config);
                Response::Observation {
                    observation: environment.observation(),
                }
            }
            Ok(Request::Step { action_id }) => match environment.step(action_id) {
                Ok(result) => Response::Step { result },
                Err(error) => Response::Error { error },
            },
            Err(error) => Response::Error {
                error: format!("invalid request: {error}"),
            },
        };
        if write_response(response).is_err() {
            break;
        }
    }
    Ok(())
}

fn environment_config(environment: &RlEnvironment) -> RlConfig {
    environment.config().clone()
}

fn argument(args: &[String], name: &str, default: &str) -> String {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
        .unwrap_or_else(|| default.into())
}
