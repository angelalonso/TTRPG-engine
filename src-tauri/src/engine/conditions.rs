use crate::engine::facts::{
    characteristic, event_completed, object_count, quest_joined, FactError, ObjectQuery,
};
use crate::engine::loader::{ConditionData, ConditionGroupData};
use crate::GameState;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRow {
    pub table: String,
    pub row: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupOperator {
    All,
    Any,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterOrEqual,
    LessThan,
    LessOrEqual,
    Present,
    Absent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConditionSubject {
    Characteristic(String),
    ObjectCount(ObjectQuery),
    EventCompleted {
        event_id: String,
        success: Option<bool>,
    },
    QuestJoined(String),
    ActiveEventCount(Option<String>),
    CalendarDay,
    AgeDays,
    ObjectPresence(ObjectQuery),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConditionValue {
    Number(f64),
    Boolean(bool),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub id: String,
    pub subject: ConditionSubject,
    pub operator: ComparisonOperator,
    pub expected: Option<ConditionValue>,
    pub source: SourceRow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupChild {
    Group(String),
    Condition(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionGroup {
    pub id: String,
    pub operator: GroupOperator,
    pub children: Vec<GroupChild>,
    pub source: SourceRow,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ConditionSet {
    pub groups: Vec<ConditionGroup>,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExplanationValue {
    Number(f64),
    Boolean(bool),
    Text(String),
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExplanationNode {
    pub id: String,
    pub kind: String,
    pub passed: bool,
    pub actual: ExplanationValue,
    pub expected: Option<ExplanationValue>,
    pub source: SourceRow,
    pub children: Vec<ExplanationNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionError {
    DuplicateGroup(String),
    DuplicateCondition(String),
    MissingGroup(String),
    MissingCondition(String),
    EmptyGroup(String),
    NotRequiresOneChild(String),
    Cycle(Vec<String>),
    Fact(FactError),
    MissingExpected(String),
    UnexpectedExpected(String),
    InvalidNumericValue(String),
    InvalidBooleanValue(String),
    UnsupportedComparison {
        condition: String,
        operator: ComparisonOperator,
    },
}

impl From<FactError> for ConditionError {
    fn from(error: FactError) -> Self {
        Self::Fact(error)
    }
}

impl ConditionSet {
    pub fn validate(&self) -> Result<(), Vec<ConditionError>> {
        let mut errors = Vec::new();
        let mut groups = HashMap::new();
        for group in &self.groups {
            if groups.insert(group.id.as_str(), group).is_some() {
                errors.push(ConditionError::DuplicateGroup(group.id.clone()));
            }
        }
        let mut conditions = HashMap::new();
        for condition in &self.conditions {
            if conditions
                .insert(condition.id.as_str(), condition)
                .is_some()
            {
                errors.push(ConditionError::DuplicateCondition(condition.id.clone()));
            }
        }
        for group in &self.groups {
            if group.children.is_empty() {
                errors.push(ConditionError::EmptyGroup(group.id.clone()));
            }
            if group.operator == GroupOperator::Not && group.children.len() != 1 {
                errors.push(ConditionError::NotRequiresOneChild(group.id.clone()));
            }
            for child in &group.children {
                match child {
                    GroupChild::Group(id) if !groups.contains_key(id.as_str()) => {
                        errors.push(ConditionError::MissingGroup(id.clone()))
                    }
                    GroupChild::Condition(id) if !conditions.contains_key(id.as_str()) => {
                        errors.push(ConditionError::MissingCondition(id.clone()))
                    }
                    _ => {}
                }
            }
        }
        if errors.is_empty() {
            let mut visiting = HashSet::new();
            let mut visited = HashSet::new();
            for group in &self.groups {
                if let Err(cycle) =
                    detect_cycle(group.id.as_str(), &groups, &mut visiting, &mut visited)
                {
                    errors.push(ConditionError::Cycle(cycle));
                }
            }
        }
        if errors.is_empty() {
            for condition in &self.conditions {
                if let Err(error) = validate_condition(condition) {
                    errors.push(error);
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn evaluate(
        &self,
        game: &GameState,
        root_group: &str,
    ) -> Result<ExplanationNode, ConditionError> {
        self.validate().map_err(|errors| errors[0].clone())?;
        let groups = self
            .groups
            .iter()
            .map(|group| (group.id.as_str(), group))
            .collect::<HashMap<_, _>>();
        let conditions = self
            .conditions
            .iter()
            .map(|condition| (condition.id.as_str(), condition))
            .collect::<HashMap<_, _>>();
        let group = groups
            .get(root_group)
            .ok_or_else(|| ConditionError::MissingGroup(root_group.to_string()))?;
        evaluate_group(group, &groups, &conditions, game)
    }

    pub fn from_rows(
        groups: &[ConditionGroupData],
        conditions: &[ConditionData],
    ) -> Result<Self, Vec<ConditionError>> {
        let mut parsed_groups = groups
            .iter()
            .map(|row| {
                Ok(ConditionGroup {
                    id: row.id.clone(),
                    operator: parse_group_operator(&row.operator)
                        .map_err(|_| ConditionError::InvalidBooleanValue(row.operator.clone()))?,
                    children: row
                        .children
                        .split(';')
                        .map(str::trim)
                        .filter(|id| !id.is_empty())
                        .map(|id| {
                            if let Some(value) = id.strip_prefix("group:") {
                                GroupChild::Group(value.to_string())
                            } else {
                                GroupChild::Condition(
                                    id.strip_prefix("condition:").unwrap_or(id).to_string(),
                                )
                            }
                        })
                        .collect(),
                    source: SourceRow {
                        table: "condition_groups.csv".to_string(),
                        row: row.source_row,
                    },
                })
            })
            .collect::<Result<Vec<_>, ConditionError>>()
            .map_err(|error| vec![error])?;
        let parsed_conditions = conditions
            .iter()
            .map(parse_condition)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| vec![error])?;
        for condition in conditions {
            if !condition.group_id.trim().is_empty() {
                let group = parsed_groups
                    .iter_mut()
                    .find(|group| group.id == condition.group_id)
                    .ok_or_else(|| {
                        vec![ConditionError::MissingGroup(condition.group_id.clone())]
                    })?;
                let child = GroupChild::Condition(condition.id.clone());
                if !group.children.contains(&child) {
                    group.children.push(child);
                }
            }
        }
        let set = Self {
            groups: parsed_groups,
            conditions: parsed_conditions,
        };
        set.validate().map(|()| set)
    }
}

fn evaluate_group(
    group: &ConditionGroup,
    groups: &HashMap<&str, &ConditionGroup>,
    conditions: &HashMap<&str, &Condition>,
    game: &GameState,
) -> Result<ExplanationNode, ConditionError> {
    let children = group
        .children
        .iter()
        .map(|child| match child {
            GroupChild::Group(id) => evaluate_group(groups[id.as_str()], groups, conditions, game),
            GroupChild::Condition(id) => evaluate_condition(conditions[id.as_str()], game),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let passed = match group.operator {
        GroupOperator::All => children.iter().all(|child| child.passed),
        GroupOperator::Any => children.iter().any(|child| child.passed),
        GroupOperator::Not => !children[0].passed,
    };
    Ok(ExplanationNode {
        id: group.id.clone(),
        kind: format!("{:?}", group.operator).to_ascii_uppercase(),
        passed,
        actual: ExplanationValue::Boolean(passed),
        expected: Some(ExplanationValue::Boolean(true)),
        source: group.source.clone(),
        children,
    })
}

fn evaluate_condition(
    condition: &Condition,
    game: &GameState,
) -> Result<ExplanationNode, ConditionError> {
    let (actual, is_presence) = actual_value(game, &condition.subject)?;
    let passed = compare(
        &condition.id,
        condition.operator,
        &actual,
        condition.expected.as_ref(),
        is_presence,
    )?;
    Ok(ExplanationNode {
        id: condition.id.clone(),
        kind: "CONDITION".to_string(),
        passed,
        actual,
        expected: condition.expected.as_ref().map(to_explanation_value),
        source: condition.source.clone(),
        children: Vec::new(),
    })
}

fn actual_value(
    game: &GameState,
    subject: &ConditionSubject,
) -> Result<(ExplanationValue, bool), ConditionError> {
    let value = match subject {
        ConditionSubject::Characteristic(id) => ExplanationValue::Number(characteristic(game, id)?),
        ConditionSubject::ObjectCount(query) => {
            ExplanationValue::Number(object_count(game, query)? as f64)
        }
        ConditionSubject::EventCompleted { event_id, success } => {
            ExplanationValue::Boolean(event_completed(game, event_id, *success)?)
        }
        ConditionSubject::QuestJoined(id) => ExplanationValue::Boolean(quest_joined(game, id)?),
        ConditionSubject::ActiveEventCount(id) => ExplanationValue::Number(
            crate::engine::facts::active_event_count(game, id.as_deref()) as f64,
        ),
        ConditionSubject::CalendarDay => ExplanationValue::Number(game.current_day as f64),
        ConditionSubject::AgeDays => ExplanationValue::Number(game.player.age_days as f64),
        ConditionSubject::ObjectPresence(query) => {
            ExplanationValue::Boolean(object_count(game, query)? > 0)
        }
    };
    Ok((
        value,
        matches!(subject, ConditionSubject::ObjectPresence(_)),
    ))
}

fn compare(
    id: &str,
    operator: ComparisonOperator,
    actual: &ExplanationValue,
    expected: Option<&ConditionValue>,
    presence: bool,
) -> Result<bool, ConditionError> {
    if matches!(
        operator,
        ComparisonOperator::Present | ComparisonOperator::Absent
    ) {
        if !presence {
            return Err(ConditionError::UnsupportedComparison {
                condition: id.to_string(),
                operator,
            });
        }
        let actual = matches!(actual, ExplanationValue::Boolean(true));
        return Ok(if operator == ComparisonOperator::Present {
            actual
        } else {
            !actual
        });
    }
    let expected = expected.ok_or_else(|| ConditionError::MissingExpected(id.to_string()))?;
    match (actual, expected) {
        (ExplanationValue::Number(actual), ConditionValue::Number(expected)) => {
            Ok(match operator {
                ComparisonOperator::Equals => actual == expected,
                ComparisonOperator::NotEquals => actual != expected,
                ComparisonOperator::GreaterThan => actual > expected,
                ComparisonOperator::GreaterOrEqual => actual >= expected,
                ComparisonOperator::LessThan => actual < expected,
                ComparisonOperator::LessOrEqual => actual <= expected,
                _ => false,
            })
        }
        (ExplanationValue::Boolean(actual), ConditionValue::Boolean(expected)) => {
            Ok(match operator {
                ComparisonOperator::Equals => actual == expected,
                ComparisonOperator::NotEquals => actual != expected,
                _ => {
                    return Err(ConditionError::UnsupportedComparison {
                        condition: id.to_string(),
                        operator,
                    })
                }
            })
        }
        (ExplanationValue::Text(actual), ConditionValue::Text(expected)) => Ok(match operator {
            ComparisonOperator::Equals => actual == expected,
            ComparisonOperator::NotEquals => actual != expected,
            _ => {
                return Err(ConditionError::UnsupportedComparison {
                    condition: id.to_string(),
                    operator,
                })
            }
        }),
        (ExplanationValue::Number(_), _) => {
            Err(ConditionError::InvalidNumericValue(id.to_string()))
        }
        (ExplanationValue::Boolean(_), _) => {
            Err(ConditionError::InvalidBooleanValue(id.to_string()))
        }
        (ExplanationValue::Text(_), _) => Err(ConditionError::UnexpectedExpected(id.to_string())),
        (ExplanationValue::Unavailable, _) => {
            Err(ConditionError::UnexpectedExpected(id.to_string()))
        }
    }
}

fn validate_condition(condition: &Condition) -> Result<(), ConditionError> {
    if matches!(
        condition.operator,
        ComparisonOperator::Present | ComparisonOperator::Absent
    ) {
        if condition.expected.is_some() {
            return Err(ConditionError::UnexpectedExpected(condition.id.clone()));
        }
    } else if condition.expected.is_none() {
        return Err(ConditionError::MissingExpected(condition.id.clone()));
    }
    if matches!(
        condition.subject,
        ConditionSubject::Characteristic(_)
            | ConditionSubject::ObjectCount(_)
            | ConditionSubject::ActiveEventCount(_)
            | ConditionSubject::CalendarDay
            | ConditionSubject::AgeDays
    ) && !matches!(condition.expected, Some(ConditionValue::Number(_)))
    {
        return Err(ConditionError::InvalidNumericValue(condition.id.clone()));
    }
    Ok(())
}

fn detect_cycle(
    id: &str,
    groups: &HashMap<&str, &ConditionGroup>,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
) -> Result<(), Vec<String>> {
    if visiting.contains(id) {
        return Err(visiting.iter().cloned().collect());
    }
    if !visited.insert(id.to_string()) {
        return Ok(());
    }
    visiting.insert(id.to_string());
    for child in &groups[id].children {
        if let GroupChild::Group(child_id) = child {
            detect_cycle(child_id, groups, visiting, visited)?;
        }
    }
    visiting.remove(id);
    Ok(())
}

fn parse_group_operator(value: &str) -> Result<GroupOperator, ()> {
    match value.trim().to_ascii_uppercase().as_str() {
        "ALL" => Ok(GroupOperator::All),
        "ANY" => Ok(GroupOperator::Any),
        "NOT" => Ok(GroupOperator::Not),
        _ => Err(()),
    }
}

fn parse_condition(row: &ConditionData) -> Result<Condition, ConditionError> {
    let subject = match row.subject_type.trim().to_ascii_lowercase().as_str() {
        "characteristic" => ConditionSubject::Characteristic(row.subject_ref.clone()),
        "object_count" => ConditionSubject::ObjectCount(parse_object_query(&row.subject_ref)),
        "object_type" => ConditionSubject::ObjectCount(ObjectQuery {
            object_type: Some(row.subject_ref.trim().to_string()),
            ..ObjectQuery::default()
        }),
        "object_presence" | "presence" => {
            ConditionSubject::ObjectPresence(parse_object_query(&row.subject_ref))
        }
        "event_completed" | "event_history" => {
            let (event_id, result) = row
                .subject_ref
                .split_once(':')
                .unwrap_or((&row.subject_ref, ""));
            ConditionSubject::EventCompleted {
                event_id: event_id.trim().to_string(),
                success: match result.trim().to_ascii_lowercase().as_str() {
                    "success" => Some(true),
                    "failure" => Some(false),
                    _ => None,
                },
            }
        }
        "quest_joined" => ConditionSubject::QuestJoined(row.subject_ref.clone()),
        "active_event" | "active_event_count" => ConditionSubject::ActiveEventCount(
            (!row.subject_ref.trim().is_empty()).then(|| row.subject_ref.clone()),
        ),
        "calendar" | "calendar_day" => ConditionSubject::CalendarDay,
        "age" | "age_days" => ConditionSubject::AgeDays,
        _ => {
            return Err(ConditionError::InvalidBooleanValue(
                row.subject_type.clone(),
            ))
        }
    };
    let operator = match row.operator.trim() {
        "=" | "==" | "equals" => ComparisonOperator::Equals,
        "!=" | "not_equals" => ComparisonOperator::NotEquals,
        ">" | "greater_than" => ComparisonOperator::GreaterThan,
        ">=" | "greater_or_equal" => ComparisonOperator::GreaterOrEqual,
        "<" | "less_than" => ComparisonOperator::LessThan,
        "<=" | "less_or_equal" => ComparisonOperator::LessOrEqual,
        "present" => ComparisonOperator::Present,
        "absent" => ComparisonOperator::Absent,
        _ => return Err(ConditionError::InvalidBooleanValue(row.operator.clone())),
    };
    let expected = if matches!(
        operator,
        ComparisonOperator::Present | ComparisonOperator::Absent
    ) {
        None
    } else if let Ok(value) = row.value.trim().parse::<f64>() {
        Some(ConditionValue::Number(value))
    } else {
        match row.value.trim().to_ascii_lowercase().as_str() {
            "true" => Some(ConditionValue::Boolean(true)),
            "false" => Some(ConditionValue::Boolean(false)),
            value => Some(ConditionValue::Text(value.to_string())),
        }
    };
    Ok(Condition {
        id: row.id.clone(),
        subject,
        operator,
        expected,
        source: SourceRow {
            table: "conditions.csv".to_string(),
            row: row.source_row,
        },
    })
}

fn parse_object_query(value: &str) -> ObjectQuery {
    let mut query = ObjectQuery::default();
    for part in value.split(',').map(str::trim) {
        if let Some(value) = part.strip_prefix("definition=") {
            query.definition_id = Some(value.to_string());
        } else if let Some(value) = part.strip_prefix("type=") {
            query.object_type = Some(value.to_string());
        } else if let Some(value) = part.strip_prefix("loaned=") {
            query.include_loaned = value.eq_ignore_ascii_case("true") || value == "1";
        } else if let Some(value) = part.strip_prefix("usable=") {
            query.include_unusable = !(value.eq_ignore_ascii_case("true") || value == "1");
        }
    }
    query
}

fn to_explanation_value(value: &ConditionValue) -> ExplanationValue {
    match value {
        ConditionValue::Number(value) => ExplanationValue::Number(*value),
        ConditionValue::Boolean(value) => ExplanationValue::Boolean(*value),
        ConditionValue::Text(value) => ExplanationValue::Text(value.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{new_game, EventHistory};

    fn game(skill: f64, ingredients: usize, lesson: bool) -> GameState {
        let mut game = new_game(concat!(env!("CARGO_MANIFEST_DIR"), "/../dataset"));
        let characteristic = game.catalog.player_characteristics[0].id.clone();
        game.player
            .characteristics
            .insert(characteristic.clone(), skill);
        let object = game.catalog.objects[0].clone();
        game.player.inventory = (0..ingredients)
            .map(|index| crate::OwnedObject {
                id: format!("{}_{}", object.id, index),
                definition_id: object.id.clone(),
                instance_id: format!("{}_{}", object.id, index),
                object_type: object.object_type.clone(),
                name: object.name.clone(),
                ..Default::default()
            })
            .collect();
        let event_id = game.catalog.events[0].id.clone();
        if lesson {
            game.event_history.push(EventHistory {
                id: "history".to_string(),
                event_id,
                object_id: String::new(),
                entered_day: 0,
                result: "success".to_string(),
                outcome: String::new(),
                reward_awarded: 0.0,
                charisma_reward_awarded: 0.0,
                damage_type: String::new(),
            });
        }
        game
    }

    #[test]
    fn evaluates_all_any_not_with_numeric_bounds_and_presence() {
        let set = ConditionSet {
            groups: vec![
                ConditionGroup {
                    id: "root".into(),
                    operator: GroupOperator::Any,
                    children: vec![
                        GroupChild::Group("requirements".into()),
                        GroupChild::Condition("lesson".into()),
                    ],
                    source: SourceRow {
                        table: "condition_groups.csv".into(),
                        row: 2,
                    },
                },
                ConditionGroup {
                    id: "requirements".into(),
                    operator: GroupOperator::All,
                    children: vec![
                        GroupChild::Condition("skill".into()),
                        GroupChild::Condition("count".into()),
                    ],
                    source: SourceRow {
                        table: "condition_groups.csv".into(),
                        row: 3,
                    },
                },
            ],
            conditions: vec![
                Condition {
                    id: "skill".into(),
                    subject: ConditionSubject::Characteristic(
                        game(0.0, 0, false).catalog.player_characteristics[0]
                            .id
                            .clone(),
                    ),
                    operator: ComparisonOperator::GreaterOrEqual,
                    expected: Some(ConditionValue::Number(5.0)),
                    source: SourceRow {
                        table: "conditions.csv".into(),
                        row: 2,
                    },
                },
                Condition {
                    id: "count".into(),
                    subject: ConditionSubject::ObjectCount(ObjectQuery {
                        object_type: Some(
                            game(0.0, 0, false).catalog.objects[0].object_type.clone(),
                        ),
                        ..Default::default()
                    }),
                    operator: ComparisonOperator::GreaterOrEqual,
                    expected: Some(ConditionValue::Number(3.0)),
                    source: SourceRow {
                        table: "conditions.csv".into(),
                        row: 3,
                    },
                },
                Condition {
                    id: "lesson".into(),
                    subject: ConditionSubject::EventCompleted {
                        event_id: game(0.0, 0, false).catalog.events[0].id.clone(),
                        success: Some(true),
                    },
                    operator: ComparisonOperator::Equals,
                    expected: Some(ConditionValue::Boolean(true)),
                    source: SourceRow {
                        table: "conditions.csv".into(),
                        row: 4,
                    },
                },
            ],
        };
        let explanation = set.evaluate(&game(5.0, 3, false), "root").unwrap();
        assert!(explanation.passed);
        assert_eq!(
            explanation.children[0].children[0].actual,
            ExplanationValue::Number(5.0)
        );
        assert!(!set.evaluate(&game(4.0, 2, false), "root").unwrap().passed);
        assert!(set.evaluate(&game(4.0, 2, true), "root").unwrap().passed);
    }

    #[test]
    fn parses_declared_calendar_event_and_active_subjects() {
        let rows = vec![
            ConditionData {
                id: "history".into(),
                group_id: "root".into(),
                subject_type: "event_history".into(),
                subject_ref: "intro:success".into(),
                operator: "equals".into(),
                value: "true".into(),
                source_row: 2,
            },
            ConditionData {
                id: "active".into(),
                group_id: "root".into(),
                subject_type: "active_event".into(),
                subject_ref: "job".into(),
                operator: "greater_or_equal".into(),
                value: "1".into(),
                source_row: 3,
            },
            ConditionData {
                id: "calendar".into(),
                group_id: "root".into(),
                subject_type: "calendar".into(),
                subject_ref: String::new(),
                operator: "greater_or_equal".into(),
                value: "1".into(),
                source_row: 4,
            },
            ConditionData {
                id: "filtered_objects".into(),
                group_id: "root".into(),
                subject_type: "object_count".into(),
                subject_ref: "type=tool,loaned=true,usable=false".into(),
                operator: "greater_or_equal".into(),
                value: "1".into(),
                source_row: 5,
            },
        ];
        let groups = vec![ConditionGroupData {
            id: "root".into(),
            operator: "all".into(),
            children: String::new(),
            source_row: 1,
        }];
        let set = ConditionSet::from_rows(&groups, &rows)
            .expect("declared condition subjects should parse");
        assert!(matches!(
            set.conditions[0].subject,
            ConditionSubject::EventCompleted {
                success: Some(true),
                ..
            }
        ));
        assert!(matches!(
            set.conditions[1].subject,
            ConditionSubject::ActiveEventCount(Some(_))
        ));
        assert!(matches!(
            set.conditions[2].subject,
            ConditionSubject::CalendarDay
        ));
        let ConditionSubject::ObjectCount(query) = &set.conditions[3].subject else {
            panic!("expected object count subject");
        };
        assert!(query.include_loaned);
        assert!(query.include_unusable);
    }

    #[test]
    fn rejects_invalid_numeric_and_not_shapes() {
        let invalid = ConditionSet {
            groups: vec![ConditionGroup {
                id: "root".into(),
                operator: GroupOperator::Not,
                children: vec![
                    GroupChild::Condition("a".into()),
                    GroupChild::Condition("b".into()),
                ],
                source: SourceRow {
                    table: "groups".into(),
                    row: 1,
                },
            }],
            conditions: vec![Condition {
                id: "a".into(),
                subject: ConditionSubject::Characteristic("skill".into()),
                operator: ComparisonOperator::GreaterOrEqual,
                expected: Some(ConditionValue::Text("five".into())),
                source: SourceRow {
                    table: "conditions".into(),
                    row: 1,
                },
            }],
        };
        assert!(invalid.validate().is_err());
        let cycle = ConditionSet {
            groups: vec![
                ConditionGroup {
                    id: "a".into(),
                    operator: GroupOperator::All,
                    children: vec![GroupChild::Group("b".into())],
                    source: SourceRow {
                        table: "groups".into(),
                        row: 1,
                    },
                },
                ConditionGroup {
                    id: "b".into(),
                    operator: GroupOperator::All,
                    children: vec![GroupChild::Group("a".into())],
                    source: SourceRow {
                        table: "groups".into(),
                        row: 2,
                    },
                },
            ],
            conditions: Vec::new(),
        };
        assert!(
            matches!(cycle.validate(), Err(errors) if errors.iter().any(|error| matches!(error, ConditionError::Cycle(_))))
        );
    }

    #[test]
    fn condition_rows_attach_to_their_declared_group() {
        let set = ConditionSet::from_rows(
            &[ConditionGroupData {
                id: "root".into(),
                operator: "all".into(),
                children: String::new(),
                source_row: 2,
            }],
            &[ConditionData {
                id: "skill".into(),
                group_id: "root".into(),
                subject_type: "characteristic".into(),
                subject_ref: "budget".into(),
                operator: ">=".into(),
                value: "5".into(),
                source_row: 2,
            }],
        )
        .expect("condition group_id should establish membership");
        assert_eq!(
            set.groups[0].children,
            vec![GroupChild::Condition("skill".into())]
        );
    }
}
