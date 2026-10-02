use super::loader::PlayerCharacteristicData;

const SUPPORTED_LIFECYCLE_TRIGGERS: &[&str] = &[
    "event_attempt",
    "event_started",
    "event_entered",
    "event_success",
    "event_failure",
    "event_completed",
    "object_acquired",
    "object_sold",
    "daily",
    "day_elapsed",
    "obligation",
    "encounter_completed",
    "quest_joined",
    "quest_completed",
    "quest_finalized",
    "loan_returned",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleBindingDiagnostic {
    pub code: &'static str,
    pub binding_id: String,
    pub message: String,
}

impl LifecycleBindingDiagnostic {
    fn new(code: &'static str, binding_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code,
            binding_id: binding_id.into(),
            message: message.into(),
        }
    }
}

/// Validate lifecycle effect bindings without applying them.
///
/// The repeat key is deliberately stable and side-effect free so callers can
/// later use it for per-operation/per-event receipt tracking. Empty
/// `trigger_ref` values retain the existing meaning of "any matching trigger".
pub fn validate_lifecycle_effect_bindings(
    effects: &[super::loader::EffectData],
    bindings: &[super::loader::EffectBindingData],
) -> Result<(), Vec<LifecycleBindingDiagnostic>> {
    use std::collections::HashSet;

    let effect_ids: HashSet<&str> = effects.iter().map(|effect| effect.id.trim()).collect();
    let mut diagnostics = Vec::new();
    let mut binding_ids = HashSet::new();
    let mut repeat_keys = HashSet::new();

    for effect in effects {
        let id = effect.id.trim();
        if id.is_empty() {
            diagnostics.push(LifecycleBindingDiagnostic::new(
                "empty_effect_id",
                "",
                "Effect definition has an empty id",
            ));
        }
    }

    for binding in bindings {
        let binding_id = binding.id.trim();
        if binding_id.is_empty() {
            diagnostics.push(LifecycleBindingDiagnostic::new(
                "empty_binding_id",
                "",
                "Effect binding has an empty id",
            ));
        } else if !binding_ids.insert(binding_id) {
            diagnostics.push(LifecycleBindingDiagnostic::new(
                "duplicate_binding_id",
                binding_id,
                format!("Effect binding id '{binding_id}' is declared more than once"),
            ));
        }

        let effect_id = binding.effect_id.trim();
        if !effect_ids.contains(effect_id) {
            diagnostics.push(LifecycleBindingDiagnostic::new(
                "unknown_effect",
                binding_id,
                format!("Effect binding references unknown effect '{effect_id}'"),
            ));
        }

        let trigger = binding.trigger_type.trim().to_ascii_lowercase();
        if trigger.is_empty() {
            diagnostics.push(LifecycleBindingDiagnostic::new(
                "empty_trigger",
                binding_id,
                "Effect binding has an empty lifecycle trigger",
            ));
        } else if !SUPPORTED_LIFECYCLE_TRIGGERS.contains(&trigger.as_str()) {
            diagnostics.push(LifecycleBindingDiagnostic::new(
                "unsupported_trigger",
                binding_id,
                format!("Effect binding uses unsupported lifecycle trigger '{trigger}'"),
            ));
        }

        if !binding.probability.is_finite() {
            diagnostics.push(LifecycleBindingDiagnostic::new(
                "non_finite_probability",
                binding_id,
                "Effect binding probability must be finite",
            ));
        } else if !(0.0..=1.0).contains(&binding.probability) {
            diagnostics.push(LifecycleBindingDiagnostic::new(
                "probability_out_of_range",
                binding_id,
                format!(
                    "Effect binding probability {} must be within [0, 1]",
                    binding.probability
                ),
            ));
        }

        let reported_result = binding.reported_result.trim().to_ascii_lowercase();
        if !reported_result.is_empty() && !matches!(reported_result.as_str(), "success" | "failure")
        {
            diagnostics.push(LifecycleBindingDiagnostic::new(
                "unsupported_reported_result",
                binding_id,
                format!(
                    "Effect binding reported result '{reported_result}' must be 'success' or 'failure'"
                ),
            ));
        }

        let trigger_ref = binding.trigger_ref.trim().to_ascii_lowercase();
        let repeat_key = format!("{effect_id}|{trigger}|{trigger_ref}|{reported_result}");
        if !repeat_keys.insert(repeat_key) {
            diagnostics.push(LifecycleBindingDiagnostic::new(
                "duplicate_repeat_key",
                binding_id,
                "Effect binding duplicates another binding for the same effect, trigger, trigger reference, and result",
            ));
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

pub fn validate_characteristic_value(
    definitions: &[PlayerCharacteristicData],
    target: &str,
    value: f64,
) -> Result<(), String> {
    if !value.is_finite() {
        return Err(format!(
            "Characteristic effect for '{target}' must produce a finite value"
        ));
    }

    let definition = definitions
        .iter()
        .find(|definition| definition.id == target)
        .ok_or_else(|| {
            format!("Characteristic effect targets unknown characteristic '{target}'")
        })?;

    if value < definition.min_value || value > definition.max_value {
        return Err(format!(
            "Characteristic effect for '{}' produces {} outside [{}, {}]",
            target, value, definition.min_value, definition.max_value
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_characteristic_value, validate_lifecycle_effect_bindings};
    use crate::engine::loader::{EffectBindingData, EffectData, PlayerCharacteristicData};

    fn definition() -> PlayerCharacteristicData {
        PlayerCharacteristicData {
            id: "skill".into(),
            name: "Skill".into(),
            value: 5.0,
            min_value: 0.0,
            max_value: 10.0,
        }
    }

    #[test]
    fn characteristic_effect_accepts_values_within_configured_bounds() {
        let definitions = [definition()];
        assert!(validate_characteristic_value(&definitions, "skill", 0.0).is_ok());
        assert!(validate_characteristic_value(&definitions, "skill", 10.0).is_ok());
    }

    #[test]
    fn characteristic_effect_rejects_unknown_nonfinite_and_out_of_range_values() {
        let definitions = [definition()];
        assert!(validate_characteristic_value(&definitions, "missing", 1.0).is_err());
        assert!(validate_characteristic_value(&definitions, "skill", f64::NAN).is_err());
        assert!(validate_characteristic_value(&definitions, "skill", 11.0).is_err());
    }

    fn effect() -> EffectData {
        EffectData {
            id: "reward".into(),
            operation: "add_characteristic".into(),
            target: "skill".into(),
            value: "1".into(),
            quantity: String::new(),
        }
    }

    fn binding(id: &str) -> EffectBindingData {
        EffectBindingData {
            id: id.into(),
            effect_id: "reward".into(),
            trigger_type: "event_completed".into(),
            trigger_ref: "event-1".into(),
            reported_result: "success".into(),
            probability: 1.0,
        }
    }

    #[test]
    fn lifecycle_binding_validator_accepts_bounded_binding_and_wildcard_reference() {
        let effects = [effect()];
        let mut binding = binding("on-success");
        binding.trigger_ref.clear();
        assert!(validate_lifecycle_effect_bindings(&effects, &[binding]).is_ok());
    }

    #[test]
    fn lifecycle_binding_validator_reports_explicit_reference_trigger_and_probability_errors() {
        let effects = [effect()];
        let mut binding = binding("bad-binding");
        binding.effect_id = "missing".into();
        binding.trigger_type = "recursive_follow_up".into();
        binding.probability = f64::NAN;

        let diagnostics = validate_lifecycle_effect_bindings(&effects, &[binding]).unwrap_err();
        let codes: Vec<_> = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect();
        assert!(codes.contains(&"unknown_effect"));
        assert!(codes.contains(&"unsupported_trigger"));
        assert!(codes.contains(&"non_finite_probability"));
    }

    #[test]
    fn lifecycle_binding_validator_rejects_duplicate_repeat_keys() {
        let effects = [effect()];
        let bindings = [binding("first"), binding("second")];

        let diagnostics = validate_lifecycle_effect_bindings(&effects, &bindings).unwrap_err();
        assert_eq!(
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "duplicate_repeat_key")
                .count(),
            1
        );
        assert_eq!(diagnostics[0].binding_id, "second");
    }
}
