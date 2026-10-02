use crate::engine::conditions::{ConditionError, ConditionSet, ExplanationNode};
use crate::engine::loader::{ConditionData, ConditionGroupData, RequirementBindingData};
use crate::GameState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequirementOperation {
    Acquire,
    Sell,
    Use,
    Consume,
    Rent,
    Loan,
    Return,
    EventStart,
    EventEntry,
    EncounterAction,
    QuestJoin,
}

impl RequirementOperation {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "acquire" => Some(Self::Acquire),
            "sell" => Some(Self::Sell),
            "use" => Some(Self::Use),
            "consume" => Some(Self::Consume),
            "rent" => Some(Self::Rent),
            "loan" => Some(Self::Loan),
            "return" => Some(Self::Return),
            "event_start" => Some(Self::EventStart),
            "event_entry" => Some(Self::EventEntry),
            "encounter_action" => Some(Self::EncounterAction),
            "quest_join" => Some(Self::QuestJoin),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Acquire => "acquire",
            Self::Sell => "sell",
            Self::Use => "use",
            Self::Consume => "consume",
            Self::Rent => "rent",
            Self::Loan => "loan",
            Self::Return => "return",
            Self::EventStart => "event_start",
            Self::EventEntry => "event_entry",
            Self::EncounterAction => "encounter_action",
            Self::QuestJoin => "quest_join",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequirementDiagnostic {
    pub code: String,
    pub binding_id: Option<String>,
    pub operation: String,
    pub target_ref: String,
    pub requirement_group: Option<String>,
    pub failed_conditions: Vec<String>,
    pub explanation: Option<ExplanationNode>,
    pub error: Option<ConditionError>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequirementEvaluation {
    pub passed: bool,
    pub diagnostics: Vec<RequirementDiagnostic>,
}

pub fn evaluate_requirements(
    game: &GameState,
    bindings: &[RequirementBindingData],
    groups: &[ConditionGroupData],
    conditions: &[ConditionData],
    operation: RequirementOperation,
    target_ref: &str,
) -> RequirementEvaluation {
    let operation_name = operation.as_str().to_string();
    let matching = bindings
        .iter()
        .filter(|binding| {
            RequirementOperation::parse(&binding.operation) == Some(operation)
                && binding.target_ref.trim() == target_ref.trim()
        })
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return RequirementEvaluation {
            passed: true,
            diagnostics: Vec::new(),
        };
    }

    let condition_set = match ConditionSet::from_rows(groups, conditions) {
        Ok(set) => set,
        Err(errors) => {
            return RequirementEvaluation {
                passed: false,
                diagnostics: errors
                    .into_iter()
                    .map(|error| RequirementDiagnostic {
                        code: "invalid_condition_set".into(),
                        binding_id: None,
                        operation: operation_name.clone(),
                        target_ref: target_ref.into(),
                        requirement_group: None,
                        failed_conditions: Vec::new(),
                        explanation: None,
                        error: Some(error),
                    })
                    .collect(),
            }
        }
    };

    let mut diagnostics = Vec::new();
    for binding in matching {
        match condition_set.evaluate(game, binding.requirement_group.trim()) {
            Ok(explanation) if explanation.passed => {}
            Ok(explanation) => diagnostics.push(RequirementDiagnostic {
                code: "requirement_not_satisfied".into(),
                binding_id: Some(binding.id.clone()),
                operation: operation_name.clone(),
                target_ref: target_ref.into(),
                requirement_group: Some(binding.requirement_group.clone()),
                failed_conditions: failed_condition_ids(&explanation),
                explanation: Some(explanation),
                error: None,
            }),
            Err(error) => diagnostics.push(RequirementDiagnostic {
                code: "condition_evaluation_failed".into(),
                binding_id: Some(binding.id.clone()),
                operation: operation_name.clone(),
                target_ref: target_ref.into(),
                requirement_group: Some(binding.requirement_group.clone()),
                failed_conditions: Vec::new(),
                explanation: None,
                error: Some(error),
            }),
        }
    }
    RequirementEvaluation {
        passed: diagnostics.is_empty(),
        diagnostics,
    }
}

fn failed_condition_ids(node: &ExplanationNode) -> Vec<String> {
    let mut ids = node
        .children
        .iter()
        .flat_map(failed_condition_ids)
        .collect::<Vec<_>>();
    if !node.passed && node.kind == "CONDITION" {
        ids.push(node.id.clone());
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::loader::{ConditionData, ConditionGroupData, RequirementBindingData};
    use crate::new_game;

    fn rows() -> (
        Vec<RequirementBindingData>,
        Vec<ConditionGroupData>,
        Vec<ConditionData>,
    ) {
        (
            vec![RequirementBindingData {
                id: "use_tool".into(),
                operation: "use".into(),
                requirement_group: "skilled".into(),
                target_ref: "tool".into(),
            }],
            vec![ConditionGroupData {
                id: "skilled".into(),
                operator: "all".into(),
                children: String::new(),
                source_row: 2,
            }],
            vec![ConditionData {
                id: "skill".into(),
                group_id: "skilled".into(),
                subject_type: "characteristic".into(),
                subject_ref: "budget".into(),
                operator: ">=".into(),
                value: "5".into(),
                source_row: 3,
            }],
        )
    }

    #[test]
    fn evaluates_use_bindings_and_reports_failed_conditions() {
        let mut game = new_game(concat!(env!("CARGO_MANIFEST_DIR"), "/../dataset"));
        let characteristic = game.catalog.player_characteristics[0].id.clone();
        let (bindings, groups, mut conditions) = rows();
        conditions[0].subject_ref = characteristic.clone();
        game.player.characteristics.insert(characteristic, 2.0);

        let result = evaluate_requirements(
            &game,
            &bindings,
            &groups,
            &conditions,
            RequirementOperation::Use,
            "tool",
        );

        assert!(!result.passed);
        assert_eq!(result.diagnostics[0].code, "requirement_not_satisfied");
        assert_eq!(
            result.diagnostics[0].binding_id.as_deref(),
            Some("use_tool")
        );
        assert_eq!(result.diagnostics[0].failed_conditions, vec!["skill"]);
        assert_eq!(
            result.diagnostics[0]
                .explanation
                .as_ref()
                .map(|explanation| explanation.passed),
            Some(false)
        );
    }

    #[test]
    fn consume_operation_is_evaluated_by_the_same_shared_path() {
        let mut game = new_game(concat!(env!("CARGO_MANIFEST_DIR"), "/../dataset"));
        let characteristic = game.catalog.player_characteristics[0].id.clone();
        let (mut bindings, groups, mut conditions) = rows();
        bindings[0].operation = "consume".into();
        conditions[0].subject_ref = characteristic.clone();
        game.player.characteristics.insert(characteristic, 5.0);

        let result = evaluate_requirements(
            &game,
            &bindings,
            &groups,
            &conditions,
            RequirementOperation::Consume,
            "tool",
        );

        assert!(result.passed);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn unrelated_operation_and_target_are_unrestricted() {
        let game = new_game(concat!(env!("CARGO_MANIFEST_DIR"), "/../dataset"));
        let (bindings, groups, conditions) = rows();
        let result = evaluate_requirements(
            &game,
            &bindings,
            &groups,
            &conditions,
            RequirementOperation::Sell,
            "tool",
        );
        assert!(result.passed);
        assert!(result.diagnostics.is_empty());
    }
}
