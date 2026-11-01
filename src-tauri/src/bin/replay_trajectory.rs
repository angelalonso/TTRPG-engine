use std::env;
use std::fs;
use ttrpg_engine_lib::{
    replay::{state_hash, Trajectory},
    rl::{GoalSpec, RlConfig, RlEnvironment},
};

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| usage("missing trajectory path"));
    let bytes = fs::read(&path).unwrap_or_else(|error| {
        eprintln!("cannot read '{path}': {error}");
        std::process::exit(1);
    });
    let trajectory: Trajectory = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        eprintln!("cannot parse '{path}': {error}");
        std::process::exit(1);
    });
    let mut environment = RlEnvironment::from_state(
        trajectory.initial_state,
        RlConfig {
            goal: GoalSpec::Characteristic {
                id: "charisma".into(),
                value: f64::MAX,
            },
            ..RlConfig::default()
        },
    );
    let mut replayed = 0;
    for step in trajectory.steps {
        if !step.replayable {
            continue;
        }
        let Some(action_id) = environment.action_id_for_key(&step.action_key) else {
            eprintln!("action is not in the RL catalogue: {}", step.action_key);
            std::process::exit(1);
        };
        let result = environment.step(action_id).unwrap_or_else(|error| {
            eprintln!("replay failed at step {}: {error}", step.index);
            std::process::exit(1);
        });
        let actual_hash = state_hash(environment.state());
        if actual_hash != step.state_after_hash {
            eprintln!(
                "state mismatch at step {}: expected {}, got {}",
                step.index, step.state_after_hash, actual_hash
            );
            std::process::exit(1);
        }
        if result.info.error.is_some() {
            eprintln!("replay produced an invalid action at step {}", step.index);
            std::process::exit(1);
        }
        replayed += 1;
    }
    println!("Replayed {replayed} RL-compatible steps successfully.");
}

fn usage(message: &str) -> ! {
    eprintln!("{message}\nusage: replay_trajectory <trajectory.json>");
    std::process::exit(2);
}
