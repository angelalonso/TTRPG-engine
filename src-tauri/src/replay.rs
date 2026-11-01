use crate::GameState;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub const TRAJECTORY_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    pub version: u32,
    pub dataset_path: String,
    pub dataset_revision: u32,
    pub initial_state: GameState,
    pub steps: Vec<TrajectoryStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryStep {
    pub index: usize,
    pub action_key: String,
    pub replayable: bool,
    pub state_before_hash: String,
    pub state_after_hash: String,
    pub error: Option<String>,
}

#[derive(Debug, Default)]
pub struct Recorder {
    trajectory: Option<Trajectory>,
}

impl Recorder {
    pub fn start(&mut self, state: &GameState) {
        self.trajectory = Some(Trajectory {
            version: TRAJECTORY_VERSION,
            dataset_path: state.dataset_path.clone(),
            dataset_revision: state.dataset_revision_number,
            initial_state: state.clone(),
            steps: Vec::new(),
        });
    }

    pub fn is_active(&self) -> bool {
        self.trajectory.is_some()
    }

    pub fn record(
        &mut self,
        action_key: impl Into<String>,
        before: &GameState,
        after: &GameState,
        replayable: bool,
        error: Option<String>,
    ) {
        let Some(trajectory) = self.trajectory.as_mut() else {
            return;
        };
        trajectory.steps.push(TrajectoryStep {
            index: trajectory.steps.len(),
            action_key: action_key.into(),
            replayable,
            state_before_hash: state_hash(before),
            state_after_hash: state_hash(after),
            error,
        });
    }

    pub fn stop(&mut self) -> Option<Trajectory> {
        self.trajectory.take()
    }

    pub fn snapshot(&self) -> Option<Trajectory> {
        self.trajectory.clone()
    }
}

pub fn state_hash(state: &GameState) -> String {
    let bytes = serde_json::to_vec(state).expect("game state should serialize");
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

pub fn write(path: impl AsRef<Path>, trajectory: &Trajectory) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(trajectory).map_err(|error| error.to_string())?;
    fs::write(path, bytes).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{state_hash, Recorder, TRAJECTORY_VERSION};
    use crate::new_game_seeded;

    #[test]
    fn recorder_keeps_initial_state_and_transition_hashes() {
        let mut state = new_game_seeded("gtr2career", 42);
        let before = state.clone();
        let mut recorder = Recorder::default();
        recorder.start(&state);
        state.current_day += 1;
        recorder.record("wait", &before, &state, true, None);
        let trace = recorder.stop().expect("trace should exist");
        assert_eq!(trace.version, TRAJECTORY_VERSION);
        assert_eq!(trace.steps.len(), 1);
        assert_eq!(trace.steps[0].state_before_hash, state_hash(&before));
        assert_ne!(
            trace.steps[0].state_before_hash,
            trace.steps[0].state_after_hash
        );
    }
}
