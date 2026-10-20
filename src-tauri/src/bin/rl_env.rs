use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use ttrpg_engine_lib::rl::{
    ActionDefinition, GoalSpec, Observation, RlConfig, RlEnvironment, StepResult,
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
    let args: Vec<String> = std::env::args().collect();
    let dataset = argument(&args, "--dataset", "gtr2career");
    let goal = argument(&args, "--goal", "charisma>=100");
    let max_days = argument(&args, "--max-days", "7300")
        .parse::<u32>()
        .unwrap_or(7300);
    let max_steps = argument(&args, "--max-steps", "14600")
        .parse::<u32>()
        .unwrap_or(14600);
    let initial_seed = argument(&args, "--seed", "1").parse::<u64>().unwrap_or(1);
    let config = RlConfig {
        goal: GoalSpec::parse(&goal),
        max_days,
        max_steps,
        ..RlConfig::default()
    };
    let mut environment = RlEnvironment::new(dataset, initial_seed, config);
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
