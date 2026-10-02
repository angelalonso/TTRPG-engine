//! Generic quest-run lifecycle and progress tracking.
//!
//! This module deliberately knows nothing about a particular event type,
//! scoring system, or dataset. Adapters can translate their event results into
//! [`QuestRun::record_result`] calls and expose the resulting progress.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnrollmentPolicy {
    Manual,
    Automatic,
    Scheduled,
}

impl Default for EnrollmentPolicy {
    fn default() -> Self {
        Self::Manual
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatPolicy {
    Once,
    Repeatable,
    Periodic,
}

impl Default for RepeatPolicy {
    fn default() -> Self {
        Self::Once
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompletionRule {
    AllRequired,
    Points { target: f64 },
    NonCompetitive { required_successes: usize },
}

impl Default for CompletionRule {
    fn default() -> Self {
        Self::AllRequired
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestRunStatus {
    Planned,
    Enrolled,
    InProgress,
    Completed,
    Failed,
    Finalized,
}

impl Default for QuestRunStatus {
    fn default() -> Self {
        Self::Planned
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventProgressStatus {
    Required,
    Optional,
    Scheduled,
    Missed,
    PendingResult,
    Recorded,
}

impl Default for EventProgressStatus {
    fn default() -> Self {
        Self::Required
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestDefinition {
    #[serde(default)]
    pub quest_id: String,
    #[serde(default)]
    pub enrollment_policy: EnrollmentPolicy,
    #[serde(default)]
    pub repeat_policy: RepeatPolicy,
    #[serde(default)]
    pub completion: CompletionRule,
    #[serde(default)]
    pub required_event_ids: Vec<String>,
    #[serde(default)]
    pub optional_event_ids: Vec<String>,
}

impl QuestDefinition {
    fn validate(&self) -> Result<(), QuestRunError> {
        if self.quest_id.trim().is_empty() {
            return Err(QuestRunError::InvalidDefinition(
                "quest ID cannot be empty".into(),
            ));
        }
        if self
            .required_event_ids
            .iter()
            .any(|id| id.trim().is_empty())
            || self
                .optional_event_ids
                .iter()
                .any(|id| id.trim().is_empty())
        {
            return Err(QuestRunError::InvalidDefinition(
                "event IDs cannot be empty".into(),
            ));
        }
        if self.required_event_ids.iter().any(|id| {
            self.optional_event_ids
                .iter()
                .any(|optional| optional == id)
        }) {
            return Err(QuestRunError::InvalidDefinition(
                "an event cannot be both required and optional".into(),
            ));
        }
        match self.completion {
            CompletionRule::Points { target } if target <= 0.0 => {
                return Err(QuestRunError::InvalidDefinition(
                    "points target must be positive".into(),
                ));
            }
            CompletionRule::NonCompetitive {
                required_successes: 0,
            } => {
                return Err(QuestRunError::InvalidDefinition(
                    "required successes must be positive".into(),
                ));
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventProgress {
    #[serde(default)]
    pub event_id: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub status: EventProgressStatus,
    #[serde(default)]
    pub points: f64,
    #[serde(default)]
    pub successful: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestRun {
    #[serde(default)]
    pub run_id: String,
    #[serde(default)]
    pub quest_id: String,
    #[serde(default)]
    pub sequence: u32,
    #[serde(default)]
    pub period_key: Option<String>,
    #[serde(default)]
    pub status: QuestRunStatus,
    #[serde(default)]
    pub points: f64,
    #[serde(default)]
    pub events: BTreeMap<String, EventProgress>,
}

/// A durable, idempotent record of a reward issued for a quest run.
///
/// `source_run_id` and `reward_id` form the exactly-once identity. The
/// serialized `receipt_id` preserves that identity across save/load and gives
/// adapters a stable key without coupling them to a reward implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewardReceipt {
    #[serde(default)]
    pub receipt_id: String,
    #[serde(default)]
    pub source_run_id: String,
    #[serde(default)]
    pub reward_id: String,
    #[serde(default)]
    pub standing: Option<u32>,
    #[serde(default)]
    pub level_or_tier: Option<String>,
    #[serde(default)]
    pub source_metadata: BTreeMap<String, String>,
    #[serde(default)]
    pub issued_day: u32,
}

impl RewardReceipt {
    pub fn new(
        source_run_id: impl Into<String>,
        reward_id: impl Into<String>,
        standing: Option<u32>,
        level_or_tier: Option<String>,
        source_metadata: BTreeMap<String, String>,
        issued_day: u32,
    ) -> Self {
        let source_run_id = source_run_id.into();
        let reward_id = reward_id.into();
        let receipt_id = Self::identity_key(&source_run_id, &reward_id);
        Self {
            receipt_id,
            source_run_id,
            reward_id,
            standing,
            level_or_tier,
            source_metadata,
            issued_day,
        }
    }

    pub fn identity_key(source_run_id: &str, reward_id: &str) -> String {
        format!("{source_run_id}::{reward_id}")
    }

    pub fn has_identity(&self, source_run_id: &str, reward_id: &str) -> bool {
        self.receipt_id == Self::identity_key(source_run_id, reward_id)
            || (self.source_run_id == source_run_id && self.reward_id == reward_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestRunError {
    InvalidDefinition(String),
    EmptyRunId,
    UnknownEvent(String),
    InvalidTransition {
        operation: &'static str,
        status: QuestRunStatus,
    },
    DuplicateResult(String),
}

impl fmt::Display for QuestRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDefinition(message) => {
                write!(formatter, "invalid quest definition: {message}")
            }
            Self::EmptyRunId => write!(formatter, "run ID cannot be empty"),
            Self::UnknownEvent(event_id) => {
                write!(formatter, "event '{event_id}' is not in this quest")
            }
            Self::InvalidTransition { operation, status } => {
                write!(formatter, "cannot {operation} a run in {status:?} status")
            }
            Self::DuplicateResult(event_id) => {
                write!(formatter, "result for event '{event_id}' already exists")
            }
        }
    }
}

impl std::error::Error for QuestRunError {}

impl QuestRun {
    pub fn new(
        definition: &QuestDefinition,
        run_id: impl Into<String>,
        sequence: u32,
        period_key: Option<String>,
    ) -> Result<Self, QuestRunError> {
        definition.validate()?;
        let run_id = run_id.into();
        if run_id.trim().is_empty() {
            return Err(QuestRunError::EmptyRunId);
        }
        let mut events = BTreeMap::new();
        for event_id in definition
            .required_event_ids
            .iter()
            .chain(definition.optional_event_ids.iter())
        {
            events.insert(
                event_id.clone(),
                EventProgress {
                    event_id: event_id.clone(),
                    required: definition
                        .required_event_ids
                        .iter()
                        .any(|id| id == event_id),
                    status: if definition
                        .required_event_ids
                        .iter()
                        .any(|id| id == event_id)
                    {
                        EventProgressStatus::Required
                    } else {
                        EventProgressStatus::Optional
                    },
                    points: 0.0,
                    successful: None,
                },
            );
        }
        Ok(Self {
            run_id,
            quest_id: definition.quest_id.clone(),
            sequence,
            period_key,
            status: QuestRunStatus::Planned,
            points: 0.0,
            events,
        })
    }

    pub fn enroll(&mut self) -> Result<(), QuestRunError> {
        self.transition_allowed("enroll", &[QuestRunStatus::Planned])?;
        self.status = QuestRunStatus::Enrolled;
        Ok(())
    }

    pub fn schedule_event(&mut self, event_id: &str) -> Result<(), QuestRunError> {
        self.transition_allowed(
            "schedule an event",
            &[QuestRunStatus::Enrolled, QuestRunStatus::InProgress],
        )?;
        let event = self.event_mut(event_id)?;
        if matches!(
            event.status,
            EventProgressStatus::Required | EventProgressStatus::Optional
        ) {
            event.status = EventProgressStatus::Scheduled;
        }
        self.status = QuestRunStatus::InProgress;
        Ok(())
    }

    pub fn mark_missed(&mut self, event_id: &str) -> Result<(), QuestRunError> {
        self.transition_allowed(
            "mark an event missed",
            &[QuestRunStatus::Enrolled, QuestRunStatus::InProgress],
        )?;
        let event = self.event_mut(event_id)?;
        if event.status == EventProgressStatus::Recorded {
            return Err(QuestRunError::InvalidTransition {
                operation: "mark a recorded event missed",
                status: self.status.clone(),
            });
        }
        event.status = EventProgressStatus::Missed;
        self.status = QuestRunStatus::InProgress;
        Ok(())
    }

    pub fn record_result(
        &mut self,
        event_id: &str,
        successful: bool,
        points: f64,
    ) -> Result<(), QuestRunError> {
        self.transition_allowed(
            "record a result",
            &[QuestRunStatus::Enrolled, QuestRunStatus::InProgress],
        )?;
        let event = self.event_mut(event_id)?;
        if event.status == EventProgressStatus::Recorded {
            return Err(QuestRunError::DuplicateResult(event_id.into()));
        }
        event.status = EventProgressStatus::Recorded;
        event.successful = Some(successful);
        event.points = points;
        self.points += points;
        self.status = QuestRunStatus::InProgress;
        Ok(())
    }

    pub fn completion_ready(&self, rule: &CompletionRule) -> bool {
        match rule {
            CompletionRule::AllRequired => {
                self.events
                    .values()
                    .filter(|event| event.required)
                    .all(|event| {
                        event.status == EventProgressStatus::Recorded
                            && event.successful == Some(true)
                    })
            }
            CompletionRule::Points { target } => self.points >= *target,
            CompletionRule::NonCompetitive { required_successes } => {
                self.events
                    .values()
                    .filter(|event| event.successful == Some(true))
                    .count()
                    >= *required_successes
            }
        }
    }

    pub fn finalize(&mut self, rule: &CompletionRule) -> Result<(), QuestRunError> {
        self.transition_allowed(
            "finalize",
            &[QuestRunStatus::Enrolled, QuestRunStatus::InProgress],
        )?;
        if self.completion_ready(rule) {
            self.status = QuestRunStatus::Completed;
        } else {
            self.status = QuestRunStatus::Failed;
        }
        Ok(())
    }

    pub fn mark_finalized(&mut self) -> Result<(), QuestRunError> {
        self.transition_allowed(
            "mark finalized",
            &[QuestRunStatus::Completed, QuestRunStatus::Failed],
        )?;
        self.status = QuestRunStatus::Finalized;
        Ok(())
    }

    fn event_mut(&mut self, event_id: &str) -> Result<&mut EventProgress, QuestRunError> {
        self.events
            .get_mut(event_id)
            .ok_or_else(|| QuestRunError::UnknownEvent(event_id.into()))
    }

    fn transition_allowed(
        &self,
        operation: &'static str,
        allowed: &[QuestRunStatus],
    ) -> Result<(), QuestRunError> {
        if allowed.contains(&self.status) {
            Ok(())
        } else {
            Err(QuestRunError::InvalidTransition {
                operation,
                status: self.status.clone(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition() -> QuestDefinition {
        QuestDefinition {
            quest_id: "seasonal-circuit".into(),
            enrollment_policy: EnrollmentPolicy::Manual,
            repeat_policy: RepeatPolicy::Periodic,
            completion: CompletionRule::AllRequired,
            required_event_ids: vec!["opening".into(), "finale".into()],
            optional_event_ids: vec!["bonus".into()],
        }
    }

    #[test]
    fn tracks_required_optional_and_lifecycle_progress() {
        let definition = definition();
        let mut run = QuestRun::new(&definition, "run-2026", 1, Some("2026".into())).unwrap();

        assert_eq!(run.status, QuestRunStatus::Planned);
        assert!(!run.completion_ready(&definition.completion));
        run.enroll().unwrap();
        run.schedule_event("finale").unwrap();
        run.record_result("finale", true, 10.0).unwrap();
        assert!(!run.completion_ready(&definition.completion));
        run.mark_missed("opening").unwrap();
        run.finalize(&definition.completion).unwrap();

        assert_eq!(run.status, QuestRunStatus::Failed);
        assert_eq!(run.events["bonus"].status, EventProgressStatus::Optional);
        assert_eq!(run.events["finale"].points, 10.0);
    }

    #[test]
    fn supports_out_of_order_results_without_completing_missing_required_steps() {
        let definition = definition();
        let mut run = QuestRun::new(&definition, "run-1", 1, None).unwrap();
        run.enroll().unwrap();
        run.record_result("finale", true, 2.0).unwrap();
        assert!(!run.completion_ready(&definition.completion));
        run.record_result("opening", true, 3.0).unwrap();
        assert!(run.completion_ready(&definition.completion));
        run.finalize(&definition.completion).unwrap();
        assert_eq!(run.status, QuestRunStatus::Completed);
    }

    #[test]
    fn keeps_repeated_runs_separate_and_rejects_duplicate_results() {
        let definition = definition();
        let first = QuestRun::new(&definition, "run-2026", 1, Some("2026".into())).unwrap();
        let second = QuestRun::new(&definition, "run-2027", 2, Some("2027".into())).unwrap();
        assert_ne!(first.run_id, second.run_id);
        assert_ne!(first.period_key, second.period_key);

        let mut run = first;
        run.enroll().unwrap();
        run.record_result("opening", true, 1.0).unwrap();
        assert_eq!(
            run.record_result("opening", true, 1.0),
            Err(QuestRunError::DuplicateResult("opening".into()))
        );
    }

    #[test]
    fn supports_points_and_noncompetitive_completion() {
        let points = CompletionRule::Points { target: 12.0 };
        let noncompetitive = CompletionRule::NonCompetitive {
            required_successes: 2,
        };
        let definition = definition();
        let mut run = QuestRun::new(&definition, "run", 1, None).unwrap();
        run.enroll().unwrap();
        run.record_result("opening", false, 4.0).unwrap();
        run.record_result("bonus", true, 8.0).unwrap();
        assert!(run.completion_ready(&points));
        assert!(!run.completion_ready(&noncompetitive));
        run.record_result("finale", true, 0.0).unwrap();
        assert!(run.completion_ready(&noncompetitive));
    }

    #[test]
    fn rejects_ambiguous_definitions_and_unknown_events() {
        let mut invalid = definition();
        invalid.optional_event_ids.push("opening".into());
        assert!(matches!(
            QuestRun::new(&invalid, "run", 1, None),
            Err(QuestRunError::InvalidDefinition(_))
        ));

        let mut run = QuestRun::new(&definition(), "run", 1, None).unwrap();
        run.enroll().unwrap();
        assert_eq!(
            run.record_result("missing", true, 1.0),
            Err(QuestRunError::UnknownEvent("missing".into()))
        );
    }

    #[test]
    fn reward_receipt_identity_is_stable_and_serializable() {
        let receipt = RewardReceipt::new(
            "season-2026",
            "trophy",
            Some(1),
            Some("gold".into()),
            BTreeMap::from([("standing_source".into(), "authoritative".into())]),
            42,
        );
        assert_eq!(receipt.receipt_id, "season-2026::trophy");
        assert!(receipt.has_identity("season-2026", "trophy"));

        let restored: RewardReceipt =
            serde_json::from_value(serde_json::to_value(&receipt).unwrap()).unwrap();
        assert_eq!(restored, receipt);
    }

    #[test]
    fn quest_run_defaults_allow_partial_persisted_records() {
        let run: QuestRun = serde_json::from_value(serde_json::json!({
            "run_id": "run-1",
            "quest_id": "quest"
        }))
        .expect("missing optional persisted fields should use defaults");
        assert_eq!(run.status, QuestRunStatus::Planned);
        assert_eq!(run.points, 0.0);
        assert!(run.events.is_empty());
    }
}
