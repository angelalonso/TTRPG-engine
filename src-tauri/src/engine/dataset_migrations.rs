use serde_json::Value;

pub fn migration_steps(from: u32, to: u32) -> Result<Vec<u32>, String> {
    if from > to {
        return Err(format!(
            "Save uses newer dataset revision {from}; current revision is {to}"
        ));
    }

    let mut revision = from;
    let mut steps = Vec::new();
    while revision < to {
        if !has_migration(revision, revision + 1) {
            return Err(format!(
                "No dataset migration is defined from revision {revision} to {}",
                revision + 1
            ));
        }
        steps.push(revision);
        revision += 1;
    }
    Ok(steps)
}

pub fn migrate_payload(mut payload: Value, from: u32, to: u32) -> Result<Value, String> {
    let steps = migration_steps(from, to)?;
    for revision in steps {
        payload = migrate_step(payload, revision, revision + 1)?;
    }
    let root = payload
        .as_object_mut()
        .ok_or_else(|| "Save payload must be a JSON object".to_string())?;
    root.insert(
        "dataset_revision_number".to_string(),
        Value::Number(to.into()),
    );
    Ok(payload)
}

fn has_migration(from: u32, to: u32) -> bool {
    matches!(
        (from, to),
        (0, 1)
            | (1, 2)
            | (2, 3)
            | (3, 4)
            | (4, 5)
            | (5, 6)
            | (6, 7)
            | (7, 8)
            | (8, 9)
            | (9, 10)
            | (10, 11)
            | (11, 12)
            | (12, 13)
            | (13, 14)
            | (14, 15)
            | (15, 16)
            | (16, 17)
            | (17, 18)
            | (18, 19)
            | (19, 20)
            | (20, 21)
            | (21, 22)
            | (22, 23)
            | (23, 24)
            | (24, 25)
            | (25, 26)
            | (26, 27)
            | (27, 28)
            | (28, 29)
            | (29, 30)
            | (30, 31)
            | (31, 32)
            | (32, 33)
            | (33, 34)
            | (34, 35)
            | (35, 36)
    )
}

fn migrate_step(payload: Value, from: u32, to: u32) -> Result<Value, String> {
    match (from, to) {
        (0, 1) => migrate_legacy_dataset_revision(payload),
        (1, 2) => migrate_position_rewards_dataset_revision(payload),
        (2, 3) => migrate_economy_rebalance_dataset_revision(payload),
        (3, 4) => migrate_living_cost_dataset_revision(payload),
        (4, 5) => migrate_value_based_living_cost_dataset_revision(payload),
        (5, 6) => migrate_job_attempt_cost_dataset_revision(payload),
        (6, 7) => migrate_gtr2career_balance_dataset_revision(payload),
        (7, 8) => migrate_gtr2career_amateur_naming_dataset_revision(payload),
        (8, 9) => migrate_gtr2career_vehicle_costs_dataset_revision(payload),
        (9, 10) => migrate_gtr2career_race_costs_dataset_revision(payload),
        (10, 11) => migrate_gtr2career_insurance_readiness_dataset_revision(payload),
        (11, 12) => migrate_gtr2career_jobs_living_costs_dataset_revision(payload),
        (12, 13) => migrate_gtr2career_calendar_championship_dataset_revision(payload),
        (13, 14) => migrate_gtr2career_remove_hot_hatch_national_dataset_revision(payload),
        (14, 15) => migrate_gtr2career_xp_and_debt_dataset_revision(payload),
        (15, 16) => migrate_gtr2career_race_gear_descriptions_dataset_revision(payload),
        (16, 17) => migrate_gtr2career_race_gear_layout_dataset_revision(payload),
        (17, 18) => migrate_gtr2career_explanation_text_color_dataset_revision(payload),
        (18, 19) => migrate_gtr2career_disclaimer_dataset_revision(payload),
        (19, 20) => migrate_gtr2career_race_championship_descriptions_dataset_revision(payload),
        (20, 21) => migrate_gtr2career_license_insurance_descriptions_dataset_revision(payload),
        (21, 22) => migrate_gtr2career_championship_licence_requirements_dataset_revision(payload),
        (22, 23) => migrate_gtr2career_amateur_race_licence_requirements_dataset_revision(payload),
        (23, 24) => migrate_gtr2career_licence_progression_dataset_revision(payload),
        (24, 25) => migrate_gtr2career_helmet_requirements_dataset_revision(payload),
        (25, 26) => migrate_gtr2career_transport_finance_championship_dataset_revision(payload),
        (26, 27) => migrate_gtr2career_hot_hatch_200_dataset_revision(payload),
        (27, 28) => migrate_gtr2career_formula_ford_specification_dataset_revision(payload),
        (28, 29) => migrate_gtr2career_goodwood_festival_dataset_revision(payload),
        (29, 30) => migrate_gtr2career_trailer_catalog_dataset_revision(payload),
        (30, 31) => migrate_gtr2career_race_trailer_category_dataset_revision(payload),
        (31, 32) => migrate_gtr2career_trailer_image_assets_dataset_revision(payload),
        (32, 33) => migrate_gtr2career_remove_race_trailer_dataset_revision(payload),
        (33, 34) => migrate_gtr2career_formation_xp_dataset_revision(payload),
        (34, 35) => migrate_gtr2career_remove_orphaned_work_obligations_dataset_revision(payload),
        (35, 36) => migrate_gtr2career_catalog_visual_assets_dataset_revision(payload),
        _ => Err(format!(
            "No dataset migration is defined from revision {from} to {to}"
        )),
    }
}

fn migrate_gtr2career_remove_orphaned_work_obligations_dataset_revision(
    mut payload: Value,
) -> Result<Value, String> {
    const REMOVED_IDS: &[&str] = &[
        "act_freelance_writing",
        "act_tutoring",
        "act_pet_sitting",
        "act_photo_sessions",
        "act_gig_musician",
        "act_research_assistant",
        "act_virtual_assistant",
        "act_garden_maintenance",
        "act_translation",
        "act_consulting",
        "act_fitness_coaching",
        "work_act_freelance_writing",
        "work_act_tutoring",
        "work_act_pet_sitting",
        "work_act_photo_sessions",
        "work_act_gig_musician",
        "work_act_research_assistant",
        "work_act_virtual_assistant",
        "work_act_garden_maintenance",
        "work_act_translation",
        "work_act_consulting",
        "work_act_fitness_coaching",
    ];

    let Some(catalog) = payload
        .as_object_mut()
        .and_then(|root| root.get_mut("catalog"))
        .and_then(Value::as_object_mut)
    else {
        return Ok(payload);
    };

    for key in ["events", "activities", "obligations"] {
        if let Some(entries) = catalog.get_mut(key).and_then(Value::as_array_mut) {
            entries.retain(|entry| {
                entry
                    .get("id")
                    .and_then(Value::as_str)
                    .map(|id| !REMOVED_IDS.contains(&id))
                    .unwrap_or(true)
            });
        }
    }
    Ok(payload)
}

fn migrate_gtr2career_catalog_visual_assets_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    Ok(payload)
}

fn migrate_legacy_dataset_revision(payload: Value) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_position_rewards_dataset_revision(payload: Value) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_economy_rebalance_dataset_revision(payload: Value) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_living_cost_dataset_revision(payload: Value) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_value_based_living_cost_dataset_revision(payload: Value) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_job_attempt_cost_dataset_revision(payload: Value) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_balance_dataset_revision(payload: Value) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_amateur_naming_dataset_revision(mut payload: Value) -> Result<Value, String> {
    fn rename(value: &mut Value) {
        match value {
            Value::String(text) => {
                *text = text
                    .replace("uk_amateur_hot_hatch_", "uk_hot_hatch_")
                    .replace("UK Amateur Hot Hatch", "UK Hot Hatch")
                    .replace("Amateur - Volkswagen Scirocco (1981)", "VW Scirocco 1981")
                    .replace("Amateur - Fiat 500 Abarth (2010)", "Fiat 500 Abarth")
                    .replace("amateur_scirocco_", "scirocco_")
                    .replace("amateur_abarth_", "abarth_");
            }
            Value::Array(values) => values.iter_mut().for_each(rename),
            Value::Object(values) => values.values_mut().for_each(rename),
            _ => {}
        }
    }

    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    rename(&mut payload);
    Ok(payload)
}

fn migrate_gtr2career_vehicle_costs_dataset_revision(payload: Value) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_race_costs_dataset_revision(payload: Value) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_insurance_readiness_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_jobs_living_costs_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_calendar_championship_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_remove_hot_hatch_national_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }

    Ok(payload)
}

fn migrate_gtr2career_xp_and_debt_dataset_revision(payload: Value) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_race_gear_descriptions_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_race_gear_layout_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_explanation_text_color_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_disclaimer_dataset_revision(payload: Value) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_race_championship_descriptions_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_license_insurance_descriptions_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_championship_licence_requirements_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_amateur_race_licence_requirements_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_licence_progression_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_helmet_requirements_dataset_revision(payload: Value) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_transport_finance_championship_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_hot_hatch_200_dataset_revision(mut payload: Value) -> Result<Value, String> {
    fn rename(value: &mut Value) {
        match value {
            Value::String(text) if text == "hh200" => *text = "caterham_320".to_string(),
            Value::Array(values) => values.iter_mut().for_each(rename),
            Value::Object(values) => values.values_mut().for_each(rename),
            _ => {}
        }
    }

    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    rename(&mut payload);
    Ok(payload)
}

fn migrate_gtr2career_formula_ford_specification_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_goodwood_festival_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_trailer_catalog_dataset_revision(payload: Value) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_race_trailer_category_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_trailer_image_assets_dataset_revision(
    payload: Value,
) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

fn migrate_gtr2career_remove_race_trailer_dataset_revision(
    mut payload: Value,
) -> Result<Value, String> {
    let root = payload
        .as_object_mut()
        .ok_or_else(|| "Save payload must be a JSON object".to_string())?;

    if let Some(player) = root.get_mut("player").and_then(Value::as_object_mut) {
        if let Some(inventory) = player.get_mut("inventory").and_then(Value::as_array_mut) {
            inventory.retain(|entry| {
                entry
                    .get("definition_id")
                    .and_then(Value::as_str)
                    != Some("race_trailer")
                    && entry.get("id").and_then(Value::as_str) != Some("race_trailer")
            });
        }
    }
    if let Some(objects) = root.get_mut("catalog").and_then(Value::as_object_mut) {
        if let Some(objects) = objects.get_mut("objects").and_then(Value::as_array_mut) {
            objects.retain(|object| {
                object.get("id").and_then(Value::as_str) != Some("race_trailer")
            });
        }
    }
    Ok(payload)
}

fn migrate_gtr2career_formation_xp_dataset_revision(payload: Value) -> Result<Value, String> {
    if !payload.is_object() {
        return Err("Save payload must be a JSON object".to_string());
    }
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::{migrate_payload, migration_steps};

    #[test]
    fn migrates_serialized_legacy_dataset_fixture_one_step_at_a_time() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_0.json"))
                .expect("dataset revision fixture should be valid JSON");

        assert_eq!(migration_steps(0, 1).unwrap(), vec![0]);
        let migrated = migrate_payload(fixture, 0, 1).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 1);
        assert_eq!(migrated["dataset_revision"], "sha256:legacy");
    }

    #[test]
    fn chains_each_supported_intermediate_migration() {
        assert_eq!(migration_steps(0, 2).unwrap(), vec![0, 1]);
    }

    #[test]
    fn migrates_revision_one_fixture_to_the_position_rewards_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_1.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 1, 2).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 2);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-1");
    }

    #[test]
    fn rejects_newer_dataset_revisions() {
        let error = migration_steps(3, 2).expect_err("newer saves must not be downgraded");
        assert!(error.contains("newer dataset revision"));
    }

    #[test]
    fn migrates_revision_two_fixture_to_the_economy_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_2.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 2, 3).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 3);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-2");
    }

    #[test]
    fn migrates_revision_three_fixture_to_the_living_cost_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_3.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 3, 4).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 4);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-3");
    }

    #[test]
    fn migrates_revision_four_fixture_to_value_based_living_cost_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_4.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 4, 5).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 5);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-4");
    }

    #[test]
    fn migrates_revision_five_fixture_to_the_job_attempt_cost_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_5.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 5, 6).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 6);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-5");
    }

    #[test]
    fn migrates_revision_six_fixture_to_the_gtr2career_balance_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_6.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 6, 7).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 7);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-6");
    }

    #[test]
    fn migrates_revision_seven_fixture_to_the_gtr2career_naming_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_7.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 7, 8).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 8);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-7");
        assert_eq!(
            migrated["player"]["active_events"][0]["event_id"],
            "uk_hot_hatch_national_round_1"
        );
        assert_eq!(
            migrated["catalog"]["objects"][0]["cost_1"],
            "scirocco_tires"
        );
        assert_eq!(
            migrated["catalog"]["quests"][0]["name"],
            "UK Hot Hatch Championship - National All-Comers Series"
        );
    }

    #[test]
    fn migrates_revision_eight_fixture_to_the_gtr2career_vehicle_cost_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_8.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 8, 9).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 9);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-8");
        assert_eq!(
            migrated["catalog"]["objects"][0]["cost_1"],
            "scirocco_tires"
        );
        assert_eq!(
            migrated["catalog"]["objects"][1]["cost_1"],
            "abarth_tires"
        );
    }

    #[test]
    fn migrates_revision_nine_fixture_to_the_gtr2career_race_cost_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_9.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 9, 10).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 10);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-9");
        assert_eq!(
            migrated["catalog"]["objects"][0]["cost_1"],
            "scirocco_tires"
        );
        assert_eq!(
            migrated["catalog"]["objects"][1]["cost_1"],
            "hhcup_tires"
        );
    }

    #[test]
    fn migrates_revision_ten_fixture_to_the_insurance_readiness_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_10.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 10, 11).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 11);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-10");
        assert_eq!(
            migrated["catalog"]["objects"][0]["requirement_group"],
            "insurance"
        );
    }

    #[test]
    fn migrates_revision_eleven_fixture_to_the_jobs_living_costs_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_11.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 11, 12).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 12);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-11");
        assert_eq!(migrated["catalog"]["costs"][0]["id"], "living_cost");
        assert_eq!(migrated["catalog"]["events"][0]["id"], "act_delivery_courier");
    }

    #[test]
    fn migrates_revision_twelve_fixture_to_the_calendar_championship_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_12.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 12, 13).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 13);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-12");
    }

    #[test]
    fn migrates_revision_thirteen_fixture_to_the_removed_hot_hatch_national_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_13.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 13, 14).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 14);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-13");
    }

    #[test]
    fn migrates_revision_fourteen_fixture_to_the_xp_and_debt_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_14.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 14, 15).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 15);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-14");
        assert_eq!(
            migrated["player"]["budget_negative_since_day"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn migrates_revision_fifteen_fixture_to_the_race_gear_descriptions_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_15.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 15, 16).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 16);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-15");
        assert_eq!(migrated["player"]["budget_negative_since_day"], serde_json::Value::Null);
    }

    #[test]
    fn migrates_revision_sixteen_fixture_to_the_race_gear_layout_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_16.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 16, 17).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 17);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-16");
    }

    #[test]
    fn migrates_revision_seventeen_fixture_to_the_explanation_text_color_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_17.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 17, 18).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 18);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-17");
    }

    #[test]
    fn migrates_revision_eighteen_fixture_to_the_disclaimer_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_18.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 18, 19).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 19);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-18");
    }

    #[test]
    fn migrates_revision_nineteen_fixture_to_the_race_championship_descriptions_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_19.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 19, 20).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 20);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-19");
    }

    #[test]
    fn migrates_revision_twenty_fixture_to_the_license_insurance_descriptions_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_20.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 20, 21).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 21);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-20");
    }

    #[test]
    fn migrates_revision_twenty_one_fixture_to_the_championship_licence_requirements_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_21.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 21, 22).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 22);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-21");
    }

    #[test]
    fn migrates_revision_twenty_two_fixture_to_the_amateur_race_licence_requirements_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_22.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 22, 23).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 23);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-22");
    }

    #[test]
    fn migrates_revision_twenty_three_fixture_to_the_licence_progression_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_23.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 23, 24).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 24);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-23");
    }

    #[test]
    fn migrates_revision_twenty_four_fixture_to_the_helmet_requirements_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_24.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 24, 25).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 25);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-24");
        assert_eq!(migrated["player"]["name"], "Driver");
        assert_eq!(
            migrated["catalog"]["objects"][0]["id"],
            "license_basic"
        );
    }

    #[test]
    fn migrates_revision_twenty_five_fixture_to_the_transport_finance_championship_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_25.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 25, 26).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 26);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-25");
        assert_eq!(migrated["player"]["name"], "Driver");
        assert_eq!(
            migrated["catalog"]["objects"][0]["id"],
            "caterham_320"
        );
    }

    #[test]
    fn migrates_revision_twenty_six_fixture_to_the_hot_hatch_200_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_26.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 26, 27).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 27);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-26");
        assert_eq!(migrated["player"]["inventory"][0]["definition_id"], "caterham_320");
        assert_eq!(
            migrated["catalog"]["events"][0]["required_object_ids"],
            "caterham_320"
        );
    }

    #[test]
    fn migrates_revision_twenty_seven_fixture_to_the_formula_ford_specification_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_27.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 27, 28).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 28);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-27");
        assert_eq!(migrated["player"]["name"], "Driver");
    }

    #[test]
    fn migrates_revision_twenty_eight_fixture_to_the_goodwood_festival_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_28.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 28, 29).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 29);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-28");
        assert_eq!(migrated["player"]["name"], "Driver");
    }

    #[test]
    fn migrates_revision_twenty_nine_fixture_to_the_trailer_catalog_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_29.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 29, 30).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 30);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-29");
        assert_eq!(migrated["player"]["name"], "Driver");
    }

    #[test]
    fn migrates_revision_thirty_fixture_to_the_race_trailer_category_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_30.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 30, 31).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 31);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-30");
        assert_eq!(migrated["player"]["name"], "Driver");
    }

    #[test]
    fn migrates_revision_thirty_one_fixture_to_the_trailer_image_assets_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_31.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 31, 32).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 32);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-31");
        assert_eq!(migrated["player"]["name"], "Driver");
    }

    #[test]
    fn migrates_revision_thirty_two_fixture_without_the_removed_race_trailer() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_32.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 32, 33).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 33);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-32");
        assert_eq!(
            migrated["player"]["inventory"][0]["definition_id"],
            "open_trailer"
        );
        assert_eq!(
            migrated["player"]["inventory"].as_array().unwrap().len(),
            1
        );
        assert!(migrated["catalog"]["objects"]
            .as_array()
            .unwrap()
            .iter()
            .all(|object| object["id"] != "race_trailer"));
    }

    #[test]
    fn migrates_revision_thirty_three_fixture_to_the_formation_xp_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_33.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 33, 34).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 34);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-33");
        assert_eq!(migrated["player"]["name"], "Driver");
    }

    #[test]
    fn migrates_revision_thirty_four_fixture_without_removed_work_entries() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_34.json"))
                .expect("dataset revision fixture should be valid JSON");

        let migrated = migrate_payload(fixture, 34, 35).unwrap();
        assert_eq!(migrated["dataset_revision_number"], 35);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-34");
        assert!(migrated["catalog"]["events"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["id"] != "act_freelance_writing"));
        assert!(migrated["catalog"]["activities"]
            .as_array()
            .unwrap()
            .iter()
            .all(|activity| activity["id"] != "act_freelance_writing"));
        assert!(migrated["catalog"]["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|obligation| obligation["id"] != "work_act_freelance_writing"));
        assert_eq!(migrated["catalog"]["events"][0]["id"], "act_work");
        assert_eq!(migrated["catalog"]["obligations"][0]["id"], "work_act_work");
    }

    #[test]
    fn migrates_revision_thirty_five_fixture_to_catalog_visual_assets_revision() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/dataset_revision_35.json"))
                .expect("dataset revision fixture should be valid JSON");

        let mut expected = fixture.clone();
        expected["dataset_revision_number"] = serde_json::json!(36);
        let migrated = migrate_payload(fixture, 35, 36).unwrap();
        assert_eq!(migrated, expected);
        assert_eq!(migrated["dataset_revision_number"], 36);
        assert_eq!(migrated["dataset_revision"], "sha256:revision-35");
        assert_eq!(migrated["player"]["name"], "Driver");
        assert!(migrated["catalog"]["events"].is_array());
        assert!(migrated["catalog"]["quests"].is_array());
        assert_eq!(migration_steps(0, 36).unwrap(), (0..36).collect::<Vec<_>>());
        assert!(migration_steps(37, 36).is_err());
    }
}
