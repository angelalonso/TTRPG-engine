use crate::{
    advance_day, apply_event, buy_object_for_sim, eligible_event_entries, enter_event_for_sim,
    join_quest_for_sim, legal_encounter_action_ids, legal_event_ids, new_game_seeded,
    resolve_encounter_for_sim, run_status, sell_object_for_sim, submit_event_for_sim_with_details,
    EventEligibility, EventResult, EventStartResult, GameState, RunStatus,
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

    pub fn event_eligibility(&self, event_id: &str) -> Vec<EventEligibility> {
        crate::event_entry_eligibility(&self.state, event_id)
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

    pub fn sell_object(&mut self, object_id: &str) -> Result<(), String> {
        sell_object_for_sim(&mut self.state, object_id)
    }

    pub fn join_quest(&mut self, quest_id: &str) -> Result<(), String> {
        join_quest_for_sim(&mut self.state, quest_id)
    }
}

#[cfg(test)]
mod tests {
    use super::HeadlessGame;
    use crate::{
        buy_object_for_sim, engine::loader::validate_dataset_directory, EventHistory, GameState,
        PendingEvent, RunStatus,
    };

    fn dataset_path() -> &'static str {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../dataset")
    }

    fn pony_stable_path() -> &'static str {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/fixtures/pony_stable")
    }

    fn cooking_path() -> &'static str {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/fixtures/cooking")
    }

    #[test]
    fn seeded_headless_state_is_reproducible() {
        let left = HeadlessGame::new(dataset_path(), 42);
        let right = HeadlessGame::new(dataset_path(), 42);
        assert_eq!(left.state().rng_state, right.state().rng_state);
        assert_eq!(left.state().current_day, right.state().current_day);
        assert_eq!(left.legal_actions(), right.legal_actions());
    }

    #[test]
    fn restored_activity_can_be_started_through_headless_api() {
        let mut game = HeadlessGame::new(dataset_path(), 7);

        let result = game
            .apply_action("act_freelance_writing")
            .expect("restored activity should be startable");

        assert_eq!(result.event_name, "Freelance writing");
        assert!(!result.message.is_empty());
    }

    #[test]
    fn direct_player_started_calls_match_eligibility_listing() {
        let mut game = HeadlessGame::new(dataset_path(), 7);
        let scheduled = game
            .state()
            .catalog
            .events
            .iter()
            .find(|event| event.day_of_year > 0)
            .expect("dataset should contain a scheduled event")
            .id
            .clone();

        assert!(!game.legal_actions().contains(&scheduled));
        let error = game
            .apply_action(&scheduled)
            .expect_err("scheduled events cannot be started as actions");
        assert!(error.contains("scheduled"));
    }

    #[test]
    fn inventory_free_scheduled_events_use_an_empty_object_selection() {
        let mut game = HeadlessGame::new(cooking_path(), 3);
        let mut event = game
            .state()
            .catalog
            .events
            .iter()
            .find(|event| event.id == "bake_pie")
            .cloned()
            .expect("cooking event should exist");
        event.id = "free_lesson".into();
        event.name = "Free lesson".into();
        event.quest_id.clear();
        event.required_object_ids.clear();
        game.state_mut().catalog.events.push(event);

        assert!(game
            .eligible_events()
            .contains(&("free_lesson".into(), String::new())));
        game.enter_event("free_lesson", "")
            .expect("inventory-free event should be enterable");
    }

    #[test]
    fn structured_event_eligibility_reports_available_inventory_free_and_missing_events() {
        let mut game = HeadlessGame::new(cooking_path(), 3);
        let mut free_event = game
            .state()
            .catalog
            .events
            .iter()
            .find(|event| event.id == "bake_pie")
            .cloned()
            .expect("cooking event should exist");
        free_event.id = "free_lesson".into();
        free_event.required_object_ids.clear();
        free_event.quest_id.clear();
        let event = free_event.id.clone();
        game.state_mut().catalog.events.push(free_event);

        let options = game.event_eligibility(&event);
        assert!(options.iter().any(|option| {
            option.selection_id.is_empty() && option.available && !option.rented
        }));
        assert!(options.iter().all(|option| option.event_id == event));

        let missing = game.event_eligibility("missing-event");
        assert_eq!(missing.len(), 1);
        assert!(!missing[0].available);
        assert!(missing[0].reason.contains("not found"));
    }

    #[test]
    fn headless_catalog_matches_loaded_dataset() {
        let game = HeadlessGame::new(dataset_path(), 1);
        let catalog = &game.state().catalog;

        for object_id in [
            "alfa_mito",
            "citroen_ds3",
            "ford_fiesta",
            "seat_ibiza",
            "vauxhall_corsa",
            "peugeot_207",
            "toyota_yaris",
            "renault_clio",
        ] {
            assert!(catalog.objects.iter().any(|object| object.id == object_id));
        }
        assert!(catalog
            .events
            .iter()
            .any(|event| event.id == "act_freelance_writing"));
    }

    #[test]
    fn all_service_intervals_include_slot_fifteen() {
        let mut game = HeadlessGame::new(dataset_path(), 9);
        game.state_mut()
            .player
            .characteristics
            .insert("budget".into(), 100_000.0);
        game.buy_object("helmet")
            .expect("helmet should be affordable");
        let object = game
            .state_mut()
            .player
            .inventory
            .first_mut()
            .expect("purchased object");
        object.cost_15 = "service_1_id".into();
        object.service_15_interval_days = 2;
        game.state_mut().current_day = 1;

        game.advance_day().expect("day advancement should succeed");

        assert!(game.state().player.inventory[0].service_15_needed);
    }

    #[test]
    fn event_count_selectors_use_non_racing_tags() {
        let mut game = HeadlessGame::new(cooking_path(), 11);
        game.state_mut().event_history.push(EventHistory {
            id: "history-1".into(),
            event_id: "bake_pie".into(),
            object_id: String::new(),
            entered_day: 1,
            result: "success".into(),
            outcome: "success".into(),
            reward_awarded: 0.0,
            charisma_reward_awarded: 0.0,
            damage_type: String::new(),
        });

        assert_eq!(crate::selected_event_count(game.state(), "cooking"), 1);
        assert_eq!(crate::selected_event_count(game.state(), "race"), 0);
    }

    #[test]
    fn cooking_fixture_validates_without_legacy_characteristics() {
        assert!(validate_dataset_directory(cooking_path()).is_valid());
        let game = HeadlessGame::new(cooking_path(), 1);
        assert!(game
            .state()
            .catalog
            .player_characteristics
            .iter()
            .all(|entry| {
                !matches!(entry.id.as_str(), "budget" | "stamina" | "charisma" | "age")
            }));
    }

    #[test]
    fn failed_result_submission_preserves_pending_entry() {
        let mut game = HeadlessGame::new(dataset_path(), 1);
        let current_day = game.state().current_day;
        game.state_mut().pending_events.push(PendingEvent {
            id: "pending-invalid-event".into(),
            event_id: "missing-event".into(),
            object_id: String::new(),
            entered_day: current_day,
            rented: false,
        });

        let error = game
            .submit_event("pending-invalid-event", "success", None)
            .expect_err("unknown event should be rejected");

        assert!(error.contains("Event not found"));
        assert_eq!(game.state().pending_events.len(), 1);
    }

    #[test]
    fn completed_result_receipt_rejects_duplicate_submission() {
        let mut game = HeadlessGame::new(cooking_path(), 1);
        let current_day = game.state().current_day;
        game.state_mut().pending_events.push(PendingEvent {
            id: "pending-bake-pie".into(),
            event_id: "bake_pie".into(),
            object_id: String::new(),
            entered_day: current_day,
            rented: false,
        });

        game.submit_event("pending-bake-pie", "success", None)
            .expect("first result submission should succeed");
        assert!(game.state().pending_events.is_empty());
        let error = game
            .submit_event("pending-bake-pie", "success", None)
            .expect_err("duplicate result submission should be rejected");
        assert!(error.contains("Pending event entry not found"));
    }

    #[test]
    fn dataset_effect_binding_applies_typed_reward_atomically() {
        let mut game = HeadlessGame::new(cooking_path(), 1);
        let current_day = game.state().current_day;
        game.state_mut().pending_events.push(PendingEvent {
            id: "pending-typed-effect".into(),
            event_id: "bake_pie".into(),
            object_id: String::new(),
            entered_day: current_day,
            rented: false,
        });

        game.submit_event("pending-typed-effect", "success", None)
            .expect("typed effect binding should apply");

        assert_eq!(
            game.state()
                .player
                .inventory
                .iter()
                .filter(|object| object.definition_id == "flour")
                .count(),
            1
        );
    }

    #[test]
    fn repeated_acquisition_keeps_distinct_instance_ids_after_sale() {
        let mut game = HeadlessGame::new(dataset_path(), 1);
        game.state_mut()
            .player
            .characteristics
            .insert("budget".into(), 1_000.0);

        game.buy_object("gloves")
            .expect("first acquisition should work");
        game.buy_object("gloves")
            .expect("second acquisition should work");
        let first_id = game.state().player.inventory[0].id.clone();
        let second_id = game.state().player.inventory[1].id.clone();
        assert_ne!(first_id, second_id);

        game.sell_object(&first_id)
            .expect("ordinary object should be sellable");
        game.buy_object("gloves")
            .expect("reacquisition after sale should work");

        let ids: Vec<_> = game
            .state()
            .player
            .inventory
            .iter()
            .filter(|object| object.definition_id == "gloves")
            .map(|object| object.id.clone())
            .collect();
        assert_eq!(ids.len(), 2);
        assert_ne!(ids[0], ids[1]);
        assert!(ids.contains(&second_id));

        let encoded = serde_json::to_string(game.state()).expect("state should serialize");
        let mut restored: GameState =
            serde_json::from_str(&encoded).expect("state should deserialize");
        buy_object_for_sim(&mut restored, "gloves").expect("restored state should acquire");
        let restored_id = restored
            .player
            .inventory
            .last()
            .expect("restored acquisition should be present")
            .id
            .clone();
        assert!(!ids.contains(&restored_id));
    }

    #[test]
    fn configured_resource_roles_replace_legacy_ids() {
        let mut game = HeadlessGame::new(dataset_path(), 1);
        game.state_mut().catalog.resource_roles.currency = Some("coins".into());
        game.state_mut().catalog.resource_roles.recovery = Some("energy".into());
        game.state_mut()
            .catalog
            .terminal_conditions
            .currency_below_zero = true;
        game.state_mut()
            .catalog
            .terminal_conditions
            .recovery_at_or_below_zero = false;
        game.state_mut()
            .player
            .characteristics
            .retain(|id, _| id != "budget" && id != "stamina");
        game.state_mut()
            .player
            .characteristics
            .insert("coins".into(), 1_000.0);
        game.state_mut()
            .player
            .characteristics
            .insert("energy".into(), 0.0);

        game.buy_object("gloves")
            .expect("configured currency should pay for purchases");
        assert_eq!(
            game.status("charisma", f64::INFINITY, 100),
            RunStatus::Ongoing
        );
        assert!(!game.state().player.characteristics.contains_key("stamina"));
        assert!(!game.state().player.characteristics.contains_key("budget"));
    }

    #[test]
    fn paid_operations_reject_missing_declared_currency() {
        let mut game = HeadlessGame::new(cooking_path(), 1);
        game.state_mut().player.characteristics.remove("coins");
        let error = game
            .buy_object("skillet")
            .expect_err("acquisition without currency must fail explicitly");
        assert!(error.contains("dataset resource"));
    }

    #[test]
    fn undeclared_characteristic_effect_does_not_expand_player_state() {
        let mut game = HeadlessGame::new(dataset_path(), 1);
        game.state_mut().player.characteristics.remove("charisma");
        game.state_mut()
            .catalog
            .player_characteristics
            .retain(|entry| entry.id != "charisma" && entry.id != "stamina");
        game.state_mut().player.characteristics.remove("stamina");
        game.state_mut().catalog.resource_roles.recovery = Some("energy".into());
        game.state_mut()
            .player
            .characteristics
            .insert("energy".into(), 10.0);
        game.advance_day().expect("day advancement should succeed");
        assert!(!game.state().player.characteristics.contains_key("charisma"));
        assert!(!game.state().player.characteristics.contains_key("stamina"));
    }

    #[test]
    fn non_racing_fixtures_validate() {
        for path in [pony_stable_path(), cooking_path()] {
            let report = validate_dataset_directory(path);
            assert!(
                report.is_valid(),
                "fixture {path} should validate: {:?}",
                report.errors
            );
        }
    }

    #[test]
    fn pony_fixture_runs_recurring_upkeep_headlessly() {
        let mut game = HeadlessGame::new(pony_stable_path(), 11);
        game.buy_object("pony")
            .expect("pony fixture should allow buying the pony");
        game.apply_action("stable_care")
            .expect("stable care should start");
        assert_eq!(game.state().player.active_events.len(), 1);

        game.advance_day()
            .expect("advancing a recurring stable day should succeed");

        assert_eq!(
            game.state().player.characteristics.get("coins"),
            Some(&32.0)
        );
        assert_eq!(game.state().player.active_events.len(), 1);
    }

    #[test]
    fn cooking_fixture_runs_quest_event_and_effect_headlessly() {
        let mut game = HeadlessGame::new(cooking_path(), 11);
        assert!(!game.state().player.characteristics.contains_key("budget"));
        assert!(!game.state().player.characteristics.contains_key("stamina"));

        game.join_quest("recipe_quest")
            .expect("recipe quest should be joinable");
        game.buy_object("skillet")
            .expect("skillet should be affordable");
        game.enter_event("bake_pie", "skillet_1")
            .expect("the skillet should satisfy the event requirement");
        let result = game
            .submit_event("event_entry_bake_pie_1", "success", None)
            .expect("the cooking result should resolve");

        assert_eq!(result.reward_awarded, 8.0);
        assert_eq!(
            game.state().player.characteristics.get("coins"),
            Some(&24.0)
        );
        assert_eq!(
            game.state().player.characteristics.get("culinary_skill"),
            Some(&2.0)
        );
    }

    #[test]
    fn failed_late_result_effect_keeps_pending_state_and_rng() {
        let mut game = HeadlessGame::new(cooking_path(), 11);
        game.join_quest("recipe_quest")
            .expect("recipe quest should be joinable");
        game.buy_object("skillet")
            .expect("skillet should be affordable");
        game.enter_event("bake_pie", "skillet_1")
            .expect("the skillet should satisfy the event requirement");
        game.state_mut().catalog.event_results[0].effects =
            "culinary_skill:1;consume_object:missing:1".into();
        let pending_before = game.state().pending_events.clone();
        let rng_before = game.state().rng_state;
        let skill_before = game.state().player.characteristics["culinary_skill"];

        let error = game
            .submit_event("event_entry_bake_pie_1", "success", None)
            .expect_err("invalid late effect should reject the result");

        assert!(error.contains("missing"));
        let pending_after = game
            .state()
            .pending_events
            .iter()
            .map(|pending| {
                (
                    pending.id.as_str(),
                    pending.event_id.as_str(),
                    pending.object_id.as_str(),
                    pending.entered_day,
                    pending.rented,
                )
            })
            .collect::<Vec<_>>();
        let pending_before = pending_before
            .iter()
            .map(|pending| {
                (
                    pending.id.as_str(),
                    pending.event_id.as_str(),
                    pending.object_id.as_str(),
                    pending.entered_day,
                    pending.rented,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(pending_after, pending_before);
        assert_eq!(game.state().rng_state, rng_before);
        assert_eq!(
            game.state().player.characteristics["culinary_skill"],
            skill_before
        );
    }
}
