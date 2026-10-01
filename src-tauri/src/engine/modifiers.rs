use super::expressions::Expression;
use super::expressions::FactProvider;
use super::loader::NumericModifierData;
use serde::Serialize;

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

    let mut ordered = modifiers.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        left.priority
            .cmp(&right.priority)
            .then_with(|| operation_order(&left.operation).cmp(&operation_order(&right.operation)))
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut value = base;
    let mut contributions = Vec::new();
    for modifier in ordered {
        if !modifier.condition_group.trim().is_empty()
            && !condition_passes(modifier.condition_group.trim())?
        {
            continue;
        }
        let expression = Expression::parse(modifier.value.trim())
            .map_err(|error| format!("Modifier '{}' has invalid value: {error}", modifier.id))?;
        let amount = expression.evaluate(facts).map_err(|error| {
            format!("Modifier '{}' could not be evaluated: {error}", modifier.id)
        })?;
        if !amount.is_finite() {
            return Err(format!(
                "Modifier '{}' evaluated to a non-finite value",
                modifier.id
            ));
        }
        value = match modifier.operation.trim().to_ascii_lowercase().as_str() {
            "set" => amount,
            "add" => value + amount,
            "multiply" => value * amount,
            operation => {
                return Err(format!(
                    "Modifier '{}' uses unsupported operation '{}'",
                    modifier.id, operation
                ))
            }
        };
        if let Some(minimum) = modifier.minimum {
            value = value.max(minimum);
        }
        if let Some(maximum) = modifier.maximum {
            value = value.min(maximum);
        }
        if !value.is_finite() {
            return Err(format!(
                "Modifier '{}' produced a non-finite result",
                modifier.id
            ));
        }
        contributions.push(NumericModifierContribution {
            id: modifier.id.clone(),
            operation: modifier.operation.clone(),
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

fn operation_order(operation: &str) -> u8 {
    match operation.trim().to_ascii_lowercase().as_str() {
        "set" => 0,
        "add" => 1,
        "multiply" => 2,
        _ => 3,
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
}
