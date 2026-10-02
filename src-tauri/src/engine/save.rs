use serde_json::{Map, Value};
use std::collections::HashSet;

pub const CURRENT_SAVE_VERSION: u32 = 2;
pub const CURRENT_SCHEMA_VERSION: u32 = 1;
const LEGACY_SAVE_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SaveVersion {
    Legacy,
    Current,
}

fn parse_version(
    value: &Map<String, Value>,
    field: &str,
    default: u32,
    current: u32,
) -> Result<u32, String> {
    let Some(raw) = value.get(field) else {
        return Ok(default);
    };
    let version = raw
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| format!("Save field '{field}' must be a non-negative integer"))?;
    if version > current {
        return Err(format!(
            "Save uses newer unsupported {field} {version}; supported version is {current}"
        ));
    }
    Ok(version)
}

fn parse_save_version(value: &Map<String, Value>) -> Result<SaveVersion, String> {
    match parse_version(
        value,
        "save_version",
        LEGACY_SAVE_VERSION,
        CURRENT_SAVE_VERSION,
    )? {
        LEGACY_SAVE_VERSION => Ok(SaveVersion::Legacy),
        CURRENT_SAVE_VERSION => Ok(SaveVersion::Current),
        version => Err(format!(
            "Save uses unsupported save_version {version}; supported versions are {LEGACY_SAVE_VERSION} and {CURRENT_SAVE_VERSION}"
        )),
    }
}

fn move_alias(map: &mut Map<String, Value>, canonical: &str, alias: &str) {
    if !map.contains_key(canonical) {
        if let Some(value) = map.remove(alias) {
            map.insert(canonical.to_string(), value);
        }
    } else {
        map.remove(alias);
    }
}

fn catalog_object_ids(root: &Map<String, Value>) -> Vec<String> {
    root.get("catalog")
        .and_then(Value::as_object)
        .and_then(|catalog| catalog.get("objects"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|object| object.get("id").and_then(Value::as_str))
        .map(str::to_string)
        .collect()
}

fn resolve_legacy_definition_id(
    legacy_id: &str,
    object_ids: &[String],
    index: usize,
) -> Result<String, String> {
    if legacy_id.is_empty() || object_ids.is_empty() {
        return Ok(legacy_id.to_string());
    }
    if object_ids.iter().any(|id| id == legacy_id) {
        return Ok(legacy_id.to_string());
    }

    let matches = object_ids
        .iter()
        .filter(|id| legacy_id.starts_with(&format!("{id}_")))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => Ok(legacy_id.to_string()),
        [definition_id] => Ok((*definition_id).clone()),
        _ => Err(format!(
            "Save inventory entry {index} has ambiguous legacy object identity '{legacy_id}'; add definition_id"
        )),
    }
}

fn migrate_inventory(root: &mut Map<String, Value>) -> Result<(), String> {
    let object_ids = catalog_object_ids(root);
    let mut next_counter = root
        .get("next_object_instance_id")
        .and_then(Value::as_u64)
        .unwrap_or(1)
        .max(1);
    let Some(inventory) = root
        .get_mut("player")
        .and_then(Value::as_object_mut)
        .and_then(|player| player.get_mut("inventory"))
        .and_then(Value::as_array_mut)
    else {
        return Ok(());
    };

    let mut used_instance_ids = HashSet::new();

    for (index, object) in inventory.iter_mut().enumerate() {
        let object = object
            .as_object_mut()
            .ok_or_else(|| format!("Save inventory entry {index} is not an object"))?;
        let legacy_id = object
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let definition_id = object
            .get("definition_id")
            .and_then(Value::as_str)
            .filter(|id| !id.trim().is_empty())
            .map(str::to_string)
            .map(Ok)
            .unwrap_or_else(|| resolve_legacy_definition_id(&legacy_id, &object_ids, index))?;

        if definition_id.is_empty() {
            return Err(format!(
                "Save inventory entry {index} has no object identity"
            ));
        }
        object.insert(
            "definition_id".to_string(),
            Value::String(definition_id.clone()),
        );

        let mut instance_id = object
            .get("instance_id")
            .and_then(Value::as_str)
            .filter(|id| !id.trim().is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| legacy_id.clone());
        if instance_id.is_empty() || used_instance_ids.contains(&instance_id) {
            loop {
                instance_id = format!("{definition_id}_{next_counter}");
                next_counter = next_counter.saturating_add(1);
                if !used_instance_ids.contains(&instance_id) {
                    break;
                }
            }
        }
        used_instance_ids.insert(instance_id.clone());
        object.insert(
            "instance_id".to_string(),
            Value::String(instance_id.clone()),
        );
        object.insert("id".to_string(), Value::String(instance_id.clone()));

        if let Some(number) = instance_id
            .rsplit_once('_')
            .and_then(|(_, suffix)| suffix.parse::<u64>().ok())
        {
            next_counter = next_counter.max(number.saturating_add(1));
        }
    }
    root.insert(
        "next_object_instance_id".to_string(),
        Value::Number(next_counter.into()),
    );
    Ok(())
}

pub fn migrate_payload(mut payload: Value) -> Result<Value, String> {
    let root = payload
        .as_object_mut()
        .ok_or_else(|| "Save payload must be a JSON object".to_string())?;
    let save_version = parse_save_version(root)?;
    parse_version(
        root,
        "schema_version",
        CURRENT_SCHEMA_VERSION,
        CURRENT_SCHEMA_VERSION,
    )?;

    if save_version == SaveVersion::Legacy {
        move_alias(root, "quest_memberships", "championship_memberships");
        move_alias(
            root,
            "pending_sponsor_event_id",
            "pending_sponsor_action_id",
        );
        if let Some(player) = root.get_mut("player").and_then(Value::as_object_mut) {
            move_alias(player, "active_events", "active_actions");
            move_alias(player, "last_event_day", "last_action_day");
            if let Some(events) = player
                .get_mut("active_events")
                .and_then(Value::as_array_mut)
            {
                for event in events {
                    if let Some(event) = event.as_object_mut() {
                        move_alias(event, "event_id", "action_id");
                    }
                }
            }
        }
        if let Some(memberships) = root
            .get_mut("quest_memberships")
            .and_then(Value::as_array_mut)
        {
            for membership in memberships {
                if let Some(membership) = membership.as_object_mut() {
                    move_alias(membership, "quest_id", "championship_id");
                }
            }
        }
    }

    migrate_inventory(root)?;
    root.insert(
        "save_version".to_string(),
        Value::Number(CURRENT_SAVE_VERSION.into()),
    );
    root.insert(
        "schema_version".to_string(),
        Value::Number(CURRENT_SCHEMA_VERSION.into()),
    );
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::{migrate_payload, CURRENT_SAVE_VERSION, CURRENT_SCHEMA_VERSION};
    use serde_json::json;

    #[test]
    fn migrates_legacy_aliases_and_object_identity() {
        let payload = json!({
            "save_version": 1,
            "schema_version": CURRENT_SCHEMA_VERSION,
            "player": {
                "active_actions": [{"action_id": "race", "start_day": 2}],
                "last_action_day": 2,
                "inventory": [
                    {"id": "gloves", "object_type": "equipment"},
                    {"id": "gloves_7", "object_type": "equipment"}
                ]
            },
            "championship_memberships": [{"championship_id": "cup", "joined_day": 1}],
            "pending_sponsor_action_id": "sponsor",
            "catalog": {"objects": [{"id": "gloves"}]}
        });

        let migrated = migrate_payload(payload).expect("legacy payload should migrate");
        assert_eq!(migrated["save_version"], CURRENT_SAVE_VERSION);
        assert_eq!(migrated["player"]["active_events"][0]["event_id"], "race");
        assert_eq!(migrated["quest_memberships"][0]["quest_id"], "cup");
        assert_eq!(
            migrated["player"]["inventory"][0]["definition_id"],
            "gloves"
        );
        assert_ne!(
            migrated["player"]["inventory"][0]["instance_id"],
            migrated["player"]["inventory"][1]["instance_id"]
        );
        assert!(migrated["next_object_instance_id"].as_u64().unwrap() > 7);
    }

    #[test]
    fn rejects_newer_save_and_schema_versions() {
        let newer_save = json!({"save_version": CURRENT_SAVE_VERSION + 1});
        let newer_schema = json!({"schema_version": CURRENT_SCHEMA_VERSION + 1});
        assert!(migrate_payload(newer_save)
            .expect_err("newer save should fail")
            .contains("newer unsupported save_version"));
        assert!(migrate_payload(newer_schema)
            .expect_err("newer schema should fail")
            .contains("newer unsupported schema_version"));
    }

    #[test]
    fn accepts_explicit_current_version_without_legacy_aliases() {
        let payload = json!({
            "save_version": CURRENT_SAVE_VERSION,
            "schema_version": CURRENT_SCHEMA_VERSION,
            "player": {
                "active_actions": [{"action_id": "legacy-name"}],
                "inventory": []
            }
        });

        let migrated = migrate_payload(payload).expect("current payload should be accepted");
        assert!(migrated["player"]["active_events"].is_null());
        assert_eq!(
            migrated["player"]["active_actions"][0]["action_id"],
            "legacy-name"
        );
    }

    #[test]
    fn rejects_unknown_older_save_versions_and_invalid_version_types() {
        let unsupported = json!({"save_version": 0});
        let invalid = json!({"save_version": "2"});

        assert!(migrate_payload(unsupported)
            .expect_err("unknown older version should fail")
            .contains("unsupported save_version 0"));
        assert!(migrate_payload(invalid)
            .expect_err("non-numeric version should fail")
            .contains("must be a non-negative integer"));
    }

    #[test]
    fn rejects_ambiguous_legacy_inventory_identity() {
        let payload = json!({
            "player": {
                "inventory": [{"id": "gloves_special_7"}]
            },
            "catalog": {
                "objects": [{"id": "gloves"}, {"id": "gloves_special"}]
            }
        });

        let error = migrate_payload(payload).expect_err("ambiguous identity should fail");
        assert!(error.contains("ambiguous legacy object identity"));
        assert!(error.contains("add definition_id"));
    }

    #[test]
    fn prefers_exact_legacy_inventory_identity_over_prefix_matches() {
        let payload = json!({
            "player": {
                "inventory": [{"id": "gloves"}]
            },
            "catalog": {
                "objects": [{"id": "gloves"}, {"id": "gloves_special"}]
            }
        });

        let migrated = migrate_payload(payload).expect("exact identity should migrate");
        assert_eq!(
            migrated["player"]["inventory"][0]["definition_id"],
            "gloves"
        );
    }
}
