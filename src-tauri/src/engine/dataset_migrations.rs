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
        (0, 1) | (1, 2) | (2, 3) | (3, 4) | (4, 5) | (5, 6)
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
        _ => Err(format!(
            "No dataset migration is defined from revision {from} to {to}"
        )),
    }
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
}
