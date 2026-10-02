use super::expressions::Expression;
use super::expressions::FactProvider;
use super::loader::NumericModifierData;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NumericModifierContribution {
    pub id: String,
    pub operation: String,
    pub value: f64,
    pub result_after: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NumericModifierResult {
    pub base: f64,
    pub contributions: Vec<NumericModifierContribution>,
    pub final_value: f64,
}

pub fn apply_numeric_modifiers<P, F>(
    base: f64,
    modifiers: &[NumericModifierData],
    facts: &P,
    mut condition_passes: F,
) -> Result<NumericModifierResult, String>
where
    P: FactProvider,
    F: FnMut(&str) -> Result<bool, String>,
{
    if !base.is_finite() {
        return Err("Numeric modifier base value must be finite".into());
    }

    let mut ids = HashSet::new();
    let mut ordered = Vec::with_capacity(modifiers.len());
    for (index, modifier) in modifiers.iter().enumerate() {
        let id = modifier.id.trim();
        if id.is_empty() {
            return Err(format!("Numeric modifier at index {index} has an empty id"));
        }
        if !ids.insert(id.to_ascii_lowercase()) {
            return Err(format!("Numeric modifier '{id}' is duplicated"));
        }

        let operation = normalized_operation(&modifier.operation).ok_or_else(|| {
            format!(
                "Modifier '{id}' uses unsupported operation '{}'",
                modifier.operation.trim()
            )
        })?;
        if modifier.value.trim().is_empty() {
            return Err(format!("Modifier '{id}' has an empty value expression"));
        }
        for (name, bound) in [("minimum", modifier.minimum), ("maximum", modifier.maximum)] {
            if bound.is_some_and(|value| !value.is_finite()) {
                return Err(format!("Modifier '{id}' has a non-finite {name} bound"));
            }
        }
        if let (Some(minimum), Some(maximum)) = (modifier.minimum, modifier.maximum) {
            if minimum > maximum {
                return Err(format!("Modifier '{id}' has minimum greater than maximum"));
            }
        }
        ordered.push((index, modifier, operation));
    }

    ordered.sort_by(|left, right| {
        left.1
            .priority
            .cmp(&right.1.priority)
            .then_with(|| operation_order(left.2).cmp(&operation_order(right.2)))
            .then_with(|| {
                left.1
                    .id
                    .trim()
                    .to_ascii_lowercase()
                    .cmp(&right.1.id.trim().to_ascii_lowercase())
            })
            .then_with(|| left.0.cmp(&right.0))
    });

    let mut value = base;
    let mut contributions = Vec::new();
    for (_, modifier, operation) in ordered {
        let id = modifier.id.trim();
        let condition_group = modifier.condition_group.trim();
        if !condition_group.is_empty() {
            let passes = condition_passes(condition_group).map_err(|error| {
                format!(
                    "Modifier '{id}' condition group '{condition_group}' could not be evaluated: {error}"
                )
            })?;
            if !passes {
                continue;
            }
        }
        let expression = Expression::parse(modifier.value.trim())
            .map_err(|error| format!("Modifier '{id}' has invalid value: {error}"))?;
        let amount = expression
            .evaluate(facts)
            .map_err(|error| format!("Modifier '{id}' could not be evaluated: {error}"))?;
        if !amount.is_finite() {
            return Err(format!("Modifier '{id}' evaluated to a non-finite value"));
        }
        value = match operation {
            "set" => amount,
            "add" => value + amount,
            "multiply" => value * amount,
            _ => unreachable!("modifier operations are validated before evaluation"),
        };
        if let Some(minimum) = modifier.minimum {
            value = value.max(minimum);
        }
        if let Some(maximum) = modifier.maximum {
            value = value.min(maximum);
        }
        if !value.is_finite() {
            return Err(format!("Modifier '{id}' produced a non-finite result"));
        }
        contributions.push(NumericModifierContribution {
            id: id.to_owned(),
            operation: operation.to_owned(),
            value: amount,
            result_after: value,
        });
    }

    Ok(NumericModifierResult {
        base,
        contributions,
        final_value: value,
    })
}

fn normalized_operation(operation: &str) -> Option<&'static str> {
    match operation.trim().to_ascii_lowercase().as_str() {
        "set" => Some("set"),
        "add" => Some("add"),
        "multiply" => Some("multiply"),
        _ => None,
    }
}

fn operation_order(operation: &str) -> u8 {
    match operation {
        "set" => 0,
        "add" => 1,
        "multiply" => 2,
        _ => unreachable!("modifier operations are validated before ordering"),
    }
}

#[cfg(test)]
mod tests {
    use super::apply_numeric_modifiers;
    use crate::engine::loader::NumericModifierData;
    use std::collections::HashMap;

    #[test]
    fn modifiers_are_ordered_and_report_breakdown() {
        let modifiers = vec![
            NumericModifierData {
                id: "multiply".into(),
                target: "value".into(),
                operation: "multiply".into(),
                value: "2".into(),
                priority: 20,
                condition_group: String::new(),
                minimum: None,
                maximum: None,
            },
            NumericModifierData {
                id: "add".into(),
                target: "value".into(),
                operation: "add".into(),
                value: "3".into(),
                priority: 10,
                condition_group: String::new(),
                minimum: None,
                maximum: None,
            },
        ];

        let facts: HashMap<String, f64> = HashMap::new();
        let result = apply_numeric_modifiers(10.0, &modifiers, &facts, |_| Ok(true))
            .expect("valid modifiers should apply");
        assert_eq!(result.final_value, 26.0);
        assert_eq!(
            result
                .contributions
                .iter()
                .map(|entry| entry.id.as_str())
                .collect::<Vec<_>>(),
            vec!["add", "multiply"]
        );
    }

    #[test]
    fn conditional_and_clamped_modifiers_are_explicit() {
        let modifiers = vec![
            NumericModifierData {
                id: "skipped".into(),
                target: "value".into(),
                operation: "add".into(),
                value: "100".into(),
                priority: 1,
                condition_group: "blocked".into(),
                minimum: None,
                maximum: None,
            },
            NumericModifierData {
                id: "bounded".into(),
                target: "value".into(),
                operation: "add".into(),
                value: "10".into(),
                priority: 2,
                condition_group: String::new(),
                minimum: Some(0.0),
                maximum: Some(12.0),
            },
        ];

        let facts: HashMap<String, f64> = HashMap::new();
        let result =
            apply_numeric_modifiers(5.0, &modifiers, &facts, |group| Ok(group != "blocked"))
                .expect("valid modifiers should apply");
        assert_eq!(result.final_value, 12.0);
        assert_eq!(result.contributions.len(), 1);
    }

    #[test]
    fn breakdown_is_canonical_and_case_insensitive() {
        let modifiers = vec![
            NumericModifierData {
                id: " Zulu ".into(),
                target: "value".into(),
                operation: " ADD ".into(),
                value: "2".into(),
                priority: 1,
                condition_group: String::new(),
                minimum: None,
                maximum: None,
            },
            NumericModifierData {
                id: "alpha".into(),
                target: "value".into(),
                operation: "multiply".into(),
                value: "3".into(),
                priority: 1,
                condition_group: String::new(),
                minimum: None,
                maximum: None,
            },
        ];

        let facts: HashMap<String, f64> = HashMap::new();
        let result = apply_numeric_modifiers(4.0, &modifiers, &facts, |_| Ok(true))
            .expect("valid modifiers should apply");
        assert_eq!(result.final_value, 18.0);
        assert_eq!(result.contributions[0].id, "Zulu");
        assert_eq!(result.contributions[0].operation, "add");
        assert_eq!(result.contributions[1].id, "alpha");
        assert_eq!(result.contributions[1].operation, "multiply");
    }

    #[test]
    fn invalid_modifiers_are_rejected_before_conditions_run() {
        let modifiers = vec![NumericModifierData {
            id: "broken".into(),
            target: "value".into(),
            operation: "divide".into(),
            value: "100".into(),
            priority: 1,
            condition_group: "never".into(),
            minimum: None,
            maximum: None,
        }];
        let facts: HashMap<String, f64> = HashMap::new();
        let mut condition_called = false;
        let error = apply_numeric_modifiers(1.0, &modifiers, &facts, |_| {
            condition_called = true;
            Ok(false)
        })
        .expect_err("unsupported operations must fail validation");
        assert_eq!(
            error,
            "Modifier 'broken' uses unsupported operation 'divide'"
        );
        assert!(!condition_called);
    }

    #[test]
    fn condition_failures_include_modifier_diagnostics() {
        let modifiers = vec![NumericModifierData {
            id: "conditional".into(),
            target: "value".into(),
            operation: "add".into(),
            value: "1".into(),
            priority: 1,
            condition_group: "missing_group".into(),
            minimum: None,
            maximum: None,
        }];
        let facts: HashMap<String, f64> = HashMap::new();
        let error = apply_numeric_modifiers(1.0, &modifiers, &facts, |_| {
            Err("unknown condition group".into())
        })
        .expect_err("condition errors must be reported");
        assert_eq!(
            error,
            "Modifier 'conditional' condition group 'missing_group' could not be evaluated: unknown condition group"
        );
    }
}
