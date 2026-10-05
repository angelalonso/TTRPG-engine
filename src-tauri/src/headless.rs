use crate::{
    advance_day, apply_event, buy_object_for_sim, eligible_event_entries, enter_event_for_sim,
    join_quest_for_sim, legal_encounter_action_ids, legal_event_ids, new_game_seeded,
    rent_event_for_sim, resolve_encounter_for_sim, run_status, sell_object_for_sim,
    submit_event_for_sim_with_details, EventEligibility, EventResult, EventStartResult, GameState,
    RunStatus,
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

    pub fn rent_event(&mut self, event_id: &str, object_id: &str) -> Result<(), String> {
        rent_event_for_sim(&mut self.state, event_id, object_id)
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

    pub fn finalize_quest_run(
        &mut self,
        run_id: &str,
        reward_id: &str,
    ) -> Result<Option<crate::engine::quest_runs::RewardReceipt>, String> {
        crate::finalize_quest_run_for_sim(&mut self.state, run_id, reward_id)
    }
}

#[cfg(test)]
mod tests {
    use super::HeadlessGame;
    use crate::{
        buy_object_for_sim, engine::loader::validate_dataset_directory,
        engine::loader::EventResultData, service_object_for_sim, EventHistory, GameState,
        PendingEvent, RunStatus, ServiceType,
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
    fn generic_quest_finalization_uses_configured_completion_and_receipts() {
        let mut game = HeadlessGame::new(cooking_path(), 42);
        let quest = game
            .state_mut()
            .catalog
            .quests
            .iter_mut()
            .find(|quest| quest.id == "recipe_quest")
            .expect("fixture quest should exist");
        quest.required_event_ids = "bake_pie".into();
        quest.completion_mode = "all_required".into();

        let definition = crate::engine::quest_runs::QuestDefinition {
            quest_id: "recipe_quest".into(),
            enrollment_policy: crate::engine::quest_runs::EnrollmentPolicy::Manual,
            repeat_policy: crate::engine::quest_runs::RepeatPolicy::Once,
            completion: crate::engine::quest_runs::CompletionRule::AllRequired,
            required_event_ids: vec!["bake_pie".into()],
            optional_event_ids: vec![],
        };
        let mut run = crate::engine::quest_runs::QuestRun::new(&definition, "recipe-run", 1, None)
            .expect("run should be valid");
        run.enroll().expect("run should enroll");
        run.record_result("bake_pie", true, 10.0)
            .expect("result should be recorded");
        game.state_mut().quest_runs.push(run);

        let receipt = game
            .finalize_quest_run("recipe-run", "recipe-trophy")
            .expect("finalization should succeed")
            .expect("completed run should issue a receipt");
        assert_eq!(receipt.source_run_id, "recipe-run");
        assert_eq!(receipt.reward_id, "recipe-trophy");
        assert_eq!(receipt.level_or_tier.as_deref(), Some("1"));
        assert_eq!(
            game.state().quest_runs[0].status,
            crate::engine::quest_runs::QuestRunStatus::Finalized
        );
        assert_eq!(game.state().reward_receipts.len(), 1);

        let retry = game
            .finalize_quest_run("recipe-run", "recipe-trophy")
            .expect("retry should be idempotent")
            .expect("retry should return the existing receipt");
        assert_eq!(retry, receipt);
        assert_eq!(game.state().reward_receipts.len(), 1);
    }

    #[test]
    fn generic_quest_finalization_records_failed_runs_without_rewards() {
        let mut game = HeadlessGame::new(cooking_path(), 42);
        let quest = game
            .state_mut()
            .catalog
            .quests
            .iter_mut()
            .find(|quest| quest.id == "recipe_quest")
            .expect("fixture quest should exist");
        quest.required_event_ids = "bake_pie".into();
        let definition = crate::engine::quest_runs::QuestDefinition {
            quest_id: "recipe_quest".into(),
            enrollment_policy: crate::engine::quest_runs::EnrollmentPolicy::Manual,
            repeat_policy: crate::engine::quest_runs::RepeatPolicy::Once,
            completion: crate::engine::quest_runs::CompletionRule::AllRequired,
            required_event_ids: vec!["bake_pie".into()],
            optional_event_ids: vec![],
        };
        let mut run = crate::engine::quest_runs::QuestRun::new(&definition, "failed-run", 1, None)
            .expect("run should be valid");
        run.enroll().expect("run should enroll");
        game.state_mut().quest_runs.push(run);

        assert!(game
            .finalize_quest_run("failed-run", "recipe-trophy")
            .expect("failed finalization should still succeed")
            .is_none());
        assert_eq!(
            game.state().quest_runs[0].status,
            crate::engine::quest_runs::QuestRunStatus::Finalized
        );
        assert!(game.state().reward_receipts.is_empty());
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
    fn seeded_replay_keeps_same_state_after_entry_and_result() {
        let mut left = HeadlessGame::new(cooking_path(), 42);
        let mut right = HeadlessGame::new(cooking_path(), 42);

        for game in [&mut left, &mut right] {
            game.join_quest("recipe_quest")
                .expect("quest should be joinable");
            game.buy_object("skillet")
                .expect("skillet should be affordable");
            game.enter_event("bake_pie", "skillet_1")
                .expect("scheduled event should be enterable");
            game.submit_event("event_entry_bake_pie_1", "success", None)
                .expect("result should resolve");
        }

        assert_eq!(
            serde_json::to_value(left.state()).expect("state should serialize"),
            serde_json::to_value(right.state()).expect("state should serialize")
        );
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
        assert_eq!(missing[0].reason_code, "event_not_found");
        assert!(!missing[0].failed_facts.is_empty());
    }

    #[test]
    fn rental_operation_uses_shared_atomic_boundary() {
        let mut game = HeadlessGame::new(dataset_path(), 3);
        let before = game.state().clone();

        let error = game
            .rent_event("race_thruxton_open", "missing-object")
            .expect_err("unknown rental object should be rejected");

        assert!(error.contains("not configured for rental"));
        assert_eq!(game.state().current_day, before.current_day);
        assert_eq!(
            game.state()
                .pending_events
                .iter()
                .map(|entry| (&entry.id, &entry.event_id, &entry.object_id))
                .collect::<Vec<_>>(),
            before
                .pending_events
                .iter()
                .map(|entry| (&entry.id, &entry.event_id, &entry.object_id))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            game.state().player.characteristics,
            before.player.characteristics
        );
    }

    #[test]
    fn valid_rental_entry_matches_shared_eligibility() {
        let mut game = HeadlessGame::new(dataset_path(), 11);
        game.state_mut()
            .player
            .characteristics
            .insert("budget".into(), 10_000.0);
        game.state_mut()
            .player
            .characteristics
            .insert("stamina".into(), 100.0);

        let event = game
            .state_mut()
            .catalog
            .events
            .iter_mut()
            .find(|event| event.id == "race_thruxton_open")
            .expect("configured race should exist");
        event.day_of_year = 1;
        event.quest_id.clear();
        event.required_license_id.clear();
        event.entry_fee = 10.0;
        event.stamina_cost = 1.0;
        event.tags = "rental".into();

        let object = game
            .state_mut()
            .catalog
            .objects
            .iter_mut()
            .find(|object| object.id == "alfa_mito")
            .expect("configured vehicle should exist");
        object.transfer_policy = "rental".into();
        object.price = 100.0;

        let options = game.event_eligibility("race_thruxton_open");
        let rental = options
            .iter()
            .find(|option| option.rented && option.selection_id == "alfa_mito")
            .expect("eligibility should expose the rental option");
        assert!(rental.available, "rental was rejected: {}", rental.reason);
        assert_eq!(rental.rental_cost, 4.0);
        assert!(rental.reason_code.is_empty());
        assert!(rental.failed_facts.is_empty());

        game.rent_event("race_thruxton_open", "alfa_mito")
            .expect("direct rental should accept the eligible option");
        assert!(game.state().pending_events.iter().any(|entry| {
            entry.event_id == "race_thruxton_open" && entry.object_id == "alfa_mito" && entry.rented
        }));
        game.state_mut().current_day = 1;
        let duplicate = game
            .event_eligibility("race_thruxton_open")
            .into_iter()
            .find(|option| option.rented && option.selection_id == "alfa_mito")
            .expect("rental option should remain discoverable after entry");
        assert!(!duplicate.available);
        assert_eq!(duplicate.reason_code, "duplicate_entry");
    }

    #[test]
    fn non_racing_rental_uses_configured_cost_without_stamina_resource() {
        let mut game = HeadlessGame::new(cooking_path(), 17);
        game.state_mut()
            .player
            .characteristics
            .insert("coins".into(), 20.0);

        let event = game
            .state_mut()
            .catalog
            .events
            .iter_mut()
            .find(|event| event.id == "bake_pie")
            .expect("cooking event should exist");
        event.quest_id.clear();
        event.required_object_ids.clear();
        event.tags = "cooking;rental".into();
        event.entry_fee = 2.0;
        event.stamina_cost = 0.0;

        let object = game
            .state_mut()
            .catalog
            .objects
            .iter_mut()
            .find(|object| object.id == "skillet")
            .expect("cooking object should exist");
        object.transfer_policy = "rental".into();
        object.rental_duration_days = 2;
        object.rental_cost = 7.5;

        let option = game
            .event_eligibility("bake_pie")
            .into_iter()
            .find(|option| option.rented && option.selection_id == "skillet")
            .expect("configured rental should be listed");
        assert!(option.available, "rental rejected: {}", option.reason);
        assert_eq!(option.rental_cost, 7.5);

        game.rent_event("bake_pie", "skillet")
            .expect("non-racing rental should not require stamina");
        assert_eq!(
            game.state()
                .player
                .characteristics
                .get("coins")
                .copied()
                .unwrap(),
            10.5
        );
        assert!(game.state().pending_events.iter().any(|entry| {
            entry.event_id == "bake_pie" && entry.object_id == "skillet" && entry.rented
        }));
    }

    #[test]
    fn non_racing_rental_expires_and_is_returned_after_configured_duration() {
        let mut game = HeadlessGame::new(cooking_path(), 23);
        game.state_mut()
            .player
            .characteristics
            .insert("coins".into(), 20.0);
        let event = game
            .state_mut()
            .catalog
            .events
            .iter_mut()
            .find(|event| event.id == "bake_pie")
            .expect("cooking event should exist");
        event.quest_id.clear();
        event.required_object_ids.clear();
        event.tags = "cooking;rental".into();
        event.entry_fee = 0.0;
        event.stamina_cost = 0.0;
        let object = game
            .state_mut()
            .catalog
            .objects
            .iter_mut()
            .find(|object| object.id == "skillet")
            .expect("cooking object should exist");
        object.transfer_policy = "rental".into();
        object.rental_duration_days = 2;
        object.rental_cost = 1.0;

        game.rent_event("bake_pie", "skillet")
            .expect("configured rental should start");
        let pending = game
            .state()
            .pending_events
            .iter()
            .find(|entry| entry.rented)
            .expect("rental should be represented separately from inventory");
        assert_eq!(pending.rental_expires_day, 3);

        game.advance_day().expect("first day should advance");
        assert!(game.state().pending_events.iter().any(|entry| entry.rented));
        game.advance_day().expect("expiry day should advance");
        assert!(!game.state().pending_events.iter().any(|entry| entry.rented));
        assert!(game
            .state()
            .pending_alerts
            .iter()
            .any(|alert| alert.title == "Rental Expired"));
    }

    #[test]
    fn rental_entry_rejects_missing_declared_currency_without_mutation() {
        let mut game = HeadlessGame::new(dataset_path(), 11);
        game.state_mut()
            .player
            .characteristics
            .insert("stamina".into(), 100.0);
        game.state_mut().player.characteristics.remove("budget");

        let event = game
            .state_mut()
            .catalog
            .events
            .iter_mut()
            .find(|event| event.id == "race_thruxton_open")
            .expect("configured race should exist");
        event.day_of_year = 1;
        event.quest_id.clear();
        event.required_license_id.clear();
        event.entry_fee = 10.0;
        event.stamina_cost = 1.0;
        event.tags = "rental".into();

        let object = game
            .state_mut()
            .catalog
            .objects
            .iter_mut()
            .find(|object| object.id == "alfa_mito")
            .expect("configured vehicle should exist");
        object.transfer_policy = "rental".into();
        object.price = 100.0;

        let before = serde_json::to_value(game.state()).expect("state should serialize");
        let error = game
            .rent_event("race_thruxton_open", "alfa_mito")
            .expect_err("rental without currency must fail explicitly");

        assert!(error.contains("dataset resource"));
        assert_eq!(
            serde_json::to_value(game.state()).expect("state should serialize"),
            before
        );
    }

    #[test]
    fn failed_encounter_turn_preserves_active_encounter_and_rng() {
        let mut game = HeadlessGame::new(dataset_path(), 19);
        let encounter = crate::engine::encounter::start(
            &game.state().catalog,
            "honda_showdown",
            "honda_rival",
            &game.state().player.characteristics,
            game.state().rng_state,
        )
        .expect("configured encounter should start");
        game.state_mut().active_encounter = Some(encounter);
        let before = game.state().clone();

        let error = game
            .resolve_encounter(Some("missing-action"))
            .expect_err("unknown encounter action should fail");

        assert!(error.contains("action") || error.contains("Action"));
        assert_eq!(game.state().rng_state, before.rng_state);
        assert_eq!(
            serde_json::to_value(game.state().active_encounter.as_ref())
                .expect("encounter should serialize"),
            serde_json::to_value(before.active_encounter.as_ref())
                .expect("encounter should serialize")
        );
    }

    #[test]
    fn encounter_win_applies_configured_custom_event_consequence() {
        let mut game = HeadlessGame::new(dataset_path(), 23);
        game.state_mut()
            .catalog
            .encounter_configs
            .iter_mut()
            .find(|config| config.encounter_id == "honda_showdown")
            .expect("configured encounter should exist")
            .mode = "standard".into();
        let action = game
            .state_mut()
            .catalog
            .encounter_actions
            .iter_mut()
            .find(|action| action.action_id == "lower_cost")
            .expect("configured player action should exist");
        action.base_success_rate = 1.0;
        action.effect_on_success = -100.0;

        let encounter = crate::engine::encounter::start(
            &game.state().catalog,
            "honda_showdown",
            "honda_rival",
            &game.state().player.characteristics,
            game.state().rng_state,
        )
        .expect("configured encounter should start");
        game.state_mut().active_encounter = Some(encounter);
        game.state_mut()
            .active_encounter
            .as_mut()
            .expect("active encounter should be stored")
            .current_actor = "player".into();

        let result = game
            .resolve_encounter(Some("lower_cost"))
            .expect("deterministic winning turn should resolve");

        assert_eq!(result["outcome"], "win");
        assert!(result["consequences_applied"]
            .as_array()
            .expect("consequences should be listed")
            .iter()
            .any(|value| value.as_str().unwrap_or_default().starts_with("Activated ")));
        assert!(game
            .state()
            .player
            .active_events
            .iter()
            .any(|active| active.event_id == "act_sponsor_honda_civic_national"));
        assert!(game.state().pending_sponsor_event_id.is_none());
        assert!(game
            .state()
            .quest_memberships
            .iter()
            .any(|membership| membership.quest_id == "honda_civic_fm_national"));
    }

    #[test]
    fn encounter_loss_applies_configured_attribute_consequences() {
        let mut game = HeadlessGame::new(dataset_path(), 23);
        let action = game
            .state_mut()
            .catalog
            .encounter_actions
            .iter_mut()
            .find(|action| action.action_id == "lower_cost")
            .expect("configured player action should exist");
        action.base_success_rate = 0.0;
        let stamina_before = game.state().player.characteristics["stamina"];
        let charisma_before = game.state().player.characteristics["charisma"];

        let mut encounter = crate::engine::encounter::start(
            &game.state().catalog,
            "honda_showdown",
            "honda_rival",
            &game.state().player.characteristics,
            game.state().rng_state,
        )
        .expect("configured encounter should start");
        encounter
            .attributes
            .get_mut("player")
            .expect("player encounter attributes should exist")
            .insert("resistance".into(), 0.0);
        game.state_mut().active_encounter = Some(encounter);
        game.state_mut()
            .active_encounter
            .as_mut()
            .expect("active encounter should be stored")
            .current_actor = "player".into();

        let result = game
            .resolve_encounter(Some("lower_cost"))
            .expect("deterministic losing turn should resolve");

        assert_eq!(result["outcome"], "lose");
        assert_eq!(
            game.state().player.characteristics["stamina"],
            stamina_before - 5.0
        );
        assert_eq!(
            game.state().player.characteristics["charisma"],
            charisma_before - 1.0
        );
        assert!(game.state().active_encounter.is_none());
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
    fn every_service_slot_can_be_cleared_through_headless_service_operation() {
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
        for slot in 1..=15 {
            match slot {
                1 => {
                    object.cost_1 = "service_1_id".into();
                    object.service_1_needed = true;
                }
                2 => {
                    object.cost_2 = "service_1_id".into();
                    object.service_2_needed = true;
                }
                3 => {
                    object.cost_3 = "service_1_id".into();
                    object.service_3_needed = true;
                }
                4 => {
                    object.cost_4 = "service_1_id".into();
                    object.service_4_needed = true;
                }
                5 => {
                    object.cost_5 = "service_1_id".into();
                    object.service_5_needed = true;
                }
                6 => {
                    object.cost_6 = "service_1_id".into();
                    object.service_6_needed = true;
                }
                7 => {
                    object.cost_7 = "service_1_id".into();
                    object.service_7_needed = true;
                }
                8 => {
                    object.cost_8 = "service_1_id".into();
                    object.service_8_needed = true;
                }
                9 => {
                    object.cost_9 = "service_1_id".into();
                    object.service_9_needed = true;
                }
                10 => {
                    object.cost_10 = "service_1_id".into();
                    object.service_10_needed = true;
                }
                11 => {
                    object.cost_11 = "service_1_id".into();
                    object.service_11_needed = true;
                }
                12 => {
                    object.cost_12 = "service_1_id".into();
                    object.service_12_needed = true;
                }
                13 => {
                    object.cost_13 = "service_1_id".into();
                    object.service_13_needed = true;
                }
                14 => {
                    object.cost_14 = "service_1_id".into();
                    object.service_14_needed = true;
                }
                15 => {
                    object.cost_15 = "service_1_id".into();
                    object.service_15_needed = true;
                }
                _ => unreachable!(),
            }
        }
        let object_id = game.state().player.inventory[0].id.clone();

        for (slot, service_type) in [
            (1, ServiceType::Service1),
            (2, ServiceType::Service2),
            (3, ServiceType::Service3),
            (4, ServiceType::Service4),
            (5, ServiceType::Service5),
            (6, ServiceType::Service6),
            (7, ServiceType::Service7),
            (8, ServiceType::Service8),
            (9, ServiceType::Service9),
            (10, ServiceType::Service10),
            (11, ServiceType::Service11),
            (12, ServiceType::Service12),
            (13, ServiceType::Service13),
            (14, ServiceType::Service14),
            (15, ServiceType::Service15),
        ] {
            service_object_for_sim(game.state_mut(), &object_id, service_type)
                .unwrap_or_else(|error| panic!("service slot {slot} should resolve: {error}"));
        }
        let object = &game.state().player.inventory[0];
        assert!(!object.service_1_needed && !object.service_2_needed);
        assert!(!object.service_3_needed && !object.service_4_needed);
        assert!(!object.service_5_needed && !object.service_6_needed);
        assert!(!object.service_7_needed && !object.service_8_needed);
        assert!(!object.service_9_needed && !object.service_10_needed);
        assert!(!object.service_11_needed && !object.service_12_needed);
        assert!(!object.service_13_needed && !object.service_14_needed);
        assert!(!object.service_15_needed);
    }

    #[test]
    fn finite_lifetime_object_expires_during_headless_day_advance() {
        let mut game = HeadlessGame::new(dataset_path(), 5);
        game.state_mut()
            .player
            .characteristics
            .insert("budget".into(), 1_000.0);
        game.state_mut()
            .catalog
            .objects
            .iter_mut()
            .find(|object| object.id == "gloves")
            .expect("gloves should exist")
            .lifetime_days = 1;

        game.buy_object("gloves").expect("gloves should be buyable");
        assert_eq!(game.state().player.inventory.len(), 1);
        game.advance_day().expect("expiry day should process");

        assert!(game.state().player.inventory.is_empty());
        assert!(game
            .state()
            .pending_alerts
            .iter()
            .any(|alert| alert.id.starts_with("expired_equipment_")));
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
            player_position: 0,
            pole_position: false,
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
            rental_expires_day: 0,
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
            rental_expires_day: 0,
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
            rental_expires_day: 0,
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
    fn reported_failure_result_uses_normalized_result_override() {
        let mut game = HeadlessGame::new(cooking_path(), 1);
        game.state_mut()
            .catalog
            .event_results
            .push(EventResultData {
                result_id: "pie_failure_override".into(),
                event_id: "bake_pie".into(),
                event_tags: "cooking".into(),
                reported_result: "failed".into(),
                probability: 1.0,
                reward_pool_delta: 2.0,
                effects: String::new(),
                message: "The pie needs more work.".into(),
            });
        let current_day = game.state().current_day;
        game.state_mut().pending_events.push(PendingEvent {
            id: "pending-failure-override".into(),
            event_id: "bake_pie".into(),
            object_id: String::new(),
            entered_day: current_day,
            rented: false,
            rental_expires_day: 0,
        });

        let result = game
            .submit_event("pending-failure-override", "failure", None)
            .expect("failure result should resolve");

        assert_eq!(result.outcome, "Unsuccessful");
        assert_eq!(result.reward_awarded, 2.0);
        assert_eq!(result.message, "The pie needs more work.");
        assert!(game.state().pending_events.is_empty());
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
    fn undeclared_terminal_resources_do_not_end_run() {
        let mut game = HeadlessGame::new(dataset_path(), 1);
        game.state_mut().catalog.resource_roles.currency = None;
        game.state_mut().catalog.resource_roles.recovery = None;
        game.state_mut()
            .player
            .characteristics
            .insert("budget".into(), -1.0);
        game.state_mut()
            .player
            .characteristics
            .insert("stamina".into(), 0.0);

        assert_eq!(
            game.status("charisma", f64::INFINITY, 100),
            RunStatus::Ongoing
        );
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
    fn paid_entry_and_quest_operations_reject_missing_declared_currency_atomically() {
        let mut game = HeadlessGame::new(cooking_path(), 1);
        game.state_mut()
            .catalog
            .quests
            .iter_mut()
            .find(|quest| quest.id == "recipe_quest")
            .expect("cooking quest should exist")
            .join_fee = 1.0;
        game.state_mut()
            .catalog
            .events
            .iter_mut()
            .find(|event| event.id == "bake_pie")
            .expect("cooking event should exist")
            .entry_fee = 1.0;
        let event = game
            .state_mut()
            .catalog
            .events
            .iter_mut()
            .find(|event| event.id == "bake_pie")
            .expect("cooking event should exist");
        event.quest_id.clear();
        event.required_object_ids.clear();
        game.state_mut().player.characteristics.remove("coins");

        let before = serde_json::to_value(game.state()).expect("state should serialize");
        let quest_error = game
            .join_quest("recipe_quest")
            .expect_err("quest joining without currency must fail explicitly");
        assert!(quest_error.contains("dataset resource"));
        assert_eq!(
            serde_json::to_value(game.state()).expect("state should serialize"),
            before
        );

        let entry_error = game
            .enter_event("bake_pie", "")
            .expect_err("event entry without currency must fail explicitly");
        assert!(entry_error.contains("dataset resource"));
        assert_eq!(
            serde_json::to_value(game.state()).expect("state should serialize"),
            before
        );
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
    fn cooking_fixture_uses_grouped_requirements_and_consumes_ingredients() {
        let mut game = HeadlessGame::new(cooking_path(), 11);
        let unavailable = game.event_eligibility("bake_with_ingredients");
        assert!(unavailable.iter().any(|option| !option.available));

        game.join_quest("recipe_quest")
            .expect("recipe quest should be joinable");
        game.buy_object("skillet")
            .expect("the skillet should be affordable");
        assert!(game
            .event_eligibility("bake_with_ingredients")
            .iter()
            .any(|option| !option.available));
        game.buy_object("flour")
            .expect("flour should be affordable");
        game.buy_object("egg").expect("egg should be affordable");
        game.enter_event("bake_with_ingredients", "skillet_1")
            .expect("grouped requirements should accept the ingredients");
        game.submit_event("event_entry_bake_with_ingredients_1", "success", None)
            .expect("ingredient result should resolve");

        assert!(!game
            .state()
            .player
            .inventory
            .iter()
            .any(|object| object.definition_id == "flour"));
        assert!(!game
            .state()
            .player
            .inventory
            .iter()
            .any(|object| object.definition_id == "egg"));
        assert_eq!(
            game.state()
                .player
                .inventory
                .iter()
                .filter(|object| object.definition_id == "pie")
                .count(),
            1
        );
        assert_eq!(
            game.state().player.characteristics.get("culinary_skill"),
            Some(&2.0)
        );
    }

    #[test]
    fn cooking_fixture_uses_configured_rental_and_returns_the_object() {
        let mut game = HeadlessGame::new(cooking_path(), 11);
        game.rent_event("rent_skillet", "skillet")
            .expect("configured cooking rental should start");

        let rental = game
            .state()
            .pending_events
            .iter()
            .find(|entry| entry.rented)
            .expect("rental should be represented as a pending event");
        assert_eq!(rental.object_id, "skillet");
        assert_eq!(rental.rental_expires_day, 3);
        assert!(!game
            .state()
            .player
            .inventory
            .iter()
            .any(|object| object.definition_id == "skillet"));

        game.advance_day().expect("rental day should advance");
        game.advance_day()
            .expect("rental expiry should be processed");
        assert!(!game.state().pending_events.iter().any(|entry| entry.rented));
        assert!(game
            .state()
            .pending_alerts
            .iter()
            .any(|alert| alert.title == "Rental Expired"));
    }

    #[test]
    fn cooking_fixture_consumption_state_survives_save_and_replay() {
        let mut direct = HeadlessGame::new(cooking_path(), 31);
        direct
            .join_quest("recipe_quest")
            .expect("recipe quest should be joinable");
        for object_id in ["skillet", "flour", "egg"] {
            direct
                .buy_object(object_id)
                .expect("fixture ingredient should be affordable");
        }
        direct
            .enter_event("bake_with_ingredients", "skillet_1")
            .expect("grouped requirements should pass");
        direct
            .submit_event("event_entry_bake_with_ingredients_1", "success", None)
            .expect("saved replay source should resolve");

        let encoded = serde_json::to_string(direct.state()).expect("state should serialize");
        let restored: GameState = serde_json::from_str(&encoded).expect("state should deserialize");
        assert_eq!(
            serde_json::to_value(&restored).expect("restored state should serialize"),
            serde_json::to_value(direct.state()).expect("direct state should serialize")
        );
        assert!(!restored
            .player
            .inventory
            .iter()
            .any(|object| matches!(object.definition_id.as_str(), "flour" | "egg")));
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
