use crate::{
    advance_day, apply_event, buy_object_for_sim, eligible_event_entries, enter_event_for_sim,
    join_quest_for_sim, legal_encounter_action_ids, legal_event_ids, new_game_seeded,
    resolve_encounter_for_sim, run_status, submit_event_for_sim_with_details, EventResult,
    EventStartResult, GameState, RunStatus,
};

#[derive(Debug, Clone)]
pub struct HeadlessGame {
    state: GameState,
}

impl HeadlessGame {
    pub fn new(dataset_path: impl Into<String>, seed: u64) -> Self {
        Self {
            state: new_game_seeded(dataset_path, seed),
        }
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    pub fn status(&self, goal_characteristic: &str, goal_value: f64, max_days: u32) -> RunStatus {
        run_status(&self.state, goal_characteristic, goal_value, max_days)
    }

    pub fn legal_actions(&self) -> Vec<String> {
        legal_event_ids(&self.state)
    }

    pub fn eligible_events(&self) -> Vec<(String, String)> {
        eligible_event_entries(&self.state)
    }

    pub fn encounter_actions(&self) -> Vec<String> {
        legal_encounter_action_ids(&self.state)
    }

    pub fn advance_day(&mut self) -> Result<(), String> {
        advance_day(&mut self.state)
    }

    pub fn apply_action(&mut self, event_id: &str) -> Result<EventStartResult, String> {
        apply_event(&mut self.state, event_id)
    }

    pub fn enter_event(&mut self, event_id: &str, object_id: &str) -> Result<(), String> {
        enter_event_for_sim(&mut self.state, event_id, object_id)
    }

    pub fn submit_event(
        &mut self,
        entry_id: &str,
        result: &str,
        player_position: Option<u32>,
    ) -> Result<EventResult, String> {
        submit_event_for_sim_with_details(&mut self.state, entry_id, result, player_position)
    }

    pub fn resolve_encounter(
        &mut self,
        action_id: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        resolve_encounter_for_sim(&mut self.state, action_id)
    }

    pub fn buy_object(&mut self, object_id: &str) -> Result<(), String> {
        buy_object_for_sim(&mut self.state, object_id)
    }

    pub fn join_quest(&mut self, quest_id: &str) -> Result<(), String> {
        join_quest_for_sim(&mut self.state, quest_id)
    }
}

#[cfg(test)]
mod tests {
    use super::HeadlessGame;

    #[test]
    fn seeded_headless_state_is_reproducible() {
        let left = HeadlessGame::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../dataset"), 42);
        let right = HeadlessGame::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../dataset"), 42);
        assert_eq!(left.state().rng_state, right.state().rng_state);
        assert_eq!(left.state().current_day, right.state().current_day);
        assert_eq!(left.legal_actions(), right.legal_actions());
    }
}
