//! Generic, deterministic quest scoring.
//!
//! This module deliberately operates on small data-transfer types instead of
//! `GameState` or racing-specific result types. Adapters can translate their
//! event history into these types without changing the legacy runtime paths.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestScoringMode {
    Ranked,
    Points,
    NonCompetitive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TiePolicy {
    Competition,
    Dense,
}

impl Default for TiePolicy {
    fn default() -> Self {
        Self::Competition
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoringRules {
    pub mode: QuestScoringMode,
    #[serde(default)]
    pub position_points: HashMap<u32, f64>,
    #[serde(default)]
    pub success_points: f64,
    #[serde(default)]
    pub failure_points: f64,
    #[serde(default)]
    pub tie_policy: TiePolicy,
}

impl Default for ScoringRules {
    fn default() -> Self {
        Self {
            mode: QuestScoringMode::Points,
            position_points: HashMap::new(),
            success_points: 1.0,
            failure_points: 0.0,
            tie_policy: TiePolicy::Competition,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestDefinition {
    pub id: String,
    pub rules: ScoringRules,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventHistoryRecord {
    pub quest_id: String,
    pub run_id: String,
    pub participant_id: String,
    pub event_id: String,
    pub sequence: u32,
    pub result: String,
    pub position: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Standing {
    pub participant_id: String,
    pub points: f64,
    pub events: u32,
    pub successes: u32,
    pub best_position: Option<u32>,
    pub rank: u32,
}

pub fn calculate_standings(
    quest: &QuestDefinition,
    run_id: &str,
    history: &[EventHistoryRecord],
) -> Vec<Standing> {
    let mut totals: HashMap<String, StandingAccumulator> = HashMap::new();

    for record in history.iter().filter(|record| {
        record.quest_id == quest.id && record.run_id == run_id && !record.participant_id.is_empty()
    }) {
        let entry = totals.entry(record.participant_id.clone()).or_default();
        entry.events += 1;
        let successful = is_success(&record.result);
        if successful {
            entry.successes += 1;
        }
        entry.points += points_for(record, &quest.rules, successful);
        entry.best_position = match (entry.best_position, record.position) {
            (Some(current), Some(candidate)) => Some(current.min(candidate)),
            (None, Some(candidate)) => Some(candidate),
            (current, None) => current,
        };
    }

    let mut standings: Vec<Standing> = totals
        .into_iter()
        .map(|(participant_id, total)| Standing {
            participant_id,
            points: normalize(total.points),
            events: total.events,
            successes: total.successes,
            best_position: total.best_position,
            rank: 0,
        })
        .collect();

    standings.sort_by(compare_standings);
    assign_ranks(&mut standings, quest.rules.tie_policy);
    standings
}

#[derive(Default)]
struct StandingAccumulator {
    points: f64,
    events: u32,
    successes: u32,
    best_position: Option<u32>,
}

fn points_for(record: &EventHistoryRecord, rules: &ScoringRules, successful: bool) -> f64 {
    match rules.mode {
        QuestScoringMode::Ranked => record
            .position
            .and_then(|position| rules.position_points.get(&position).copied())
            .unwrap_or({
                if successful {
                    rules.success_points
                } else {
                    rules.failure_points
                }
            }),
        QuestScoringMode::Points => {
            if successful {
                rules.success_points
            } else {
                rules.failure_points
            }
        }
        QuestScoringMode::NonCompetitive => 0.0,
    }
}

fn is_success(result: &str) -> bool {
    matches!(
        result.trim().to_ascii_lowercase().as_str(),
        "success" | "succeeded" | "complete" | "completed" | "win" | "won"
    )
}

fn normalize(value: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

fn compare_standings(left: &Standing, right: &Standing) -> Ordering {
    right
        .points
        .total_cmp(&left.points)
        .then_with(|| right.successes.cmp(&left.successes))
        .then_with(|| right.events.cmp(&left.events))
        .then_with(|| match (left.best_position, right.best_position) {
            (Some(left), Some(right)) => left.cmp(&right),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        })
        .then_with(|| left.participant_id.cmp(&right.participant_id))
}

fn assign_ranks(standings: &mut [Standing], tie_policy: TiePolicy) {
    let mut previous: Option<Standing> = None;
    let mut previous_rank = 0;
    let mut distinct_rank = 0;
    for (index, standing) in standings.iter_mut().enumerate() {
        let tied = previous
            .as_ref()
            .is_some_and(|previous| same_score(previous, standing));
        if !tied {
            distinct_rank += 1;
        }
        standing.rank = match tie_policy {
            TiePolicy::Competition => {
                if tied {
                    previous_rank
                } else {
                    index as u32 + 1
                }
            }
            TiePolicy::Dense => distinct_rank,
        };
        previous_rank = standing.rank;
        previous = Some(standing.clone());
    }
}

fn same_score(left: &Standing, right: &Standing) -> bool {
    left.points.total_cmp(&right.points) == Ordering::Equal
        && left.successes == right.successes
        && left.events == right.events
        && left.best_position == right.best_position
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quest(mode: QuestScoringMode, tie_policy: TiePolicy) -> QuestDefinition {
        QuestDefinition {
            id: "quest".into(),
            rules: ScoringRules {
                mode,
                position_points: HashMap::from([(1, 10.0), (2, 6.0), (3, 3.0)]),
                success_points: 2.0,
                failure_points: -1.0,
                tie_policy,
            },
        }
    }

    fn result(
        participant_id: &str,
        event_id: &str,
        sequence: u32,
        result: &str,
        position: Option<u32>,
    ) -> EventHistoryRecord {
        EventHistoryRecord {
            quest_id: "quest".into(),
            run_id: "run-1".into(),
            participant_id: participant_id.into(),
            event_id: event_id.into(),
            sequence,
            result: result.into(),
            position,
        }
    }

    #[test]
    fn ranked_scores_accumulate_across_events_not_last_result() {
        let history = vec![
            result("alice", "round-1", 1, "success", Some(1)),
            result("alice", "round-2", 2, "success", Some(2)),
            result("bob", "round-1", 1, "success", Some(2)),
            result("bob", "round-2", 2, "success", Some(1)),
        ];

        let standings = calculate_standings(
            &quest(QuestScoringMode::Ranked, TiePolicy::Competition),
            "run-1",
            &history,
        );

        assert_eq!(standings[0].participant_id, "alice");
        assert_eq!(standings[0].points, 16.0);
        assert_eq!(standings[1].points, 16.0);
        assert_eq!(standings[0].rank, 1);
        assert_eq!(standings[1].rank, 1);
    }

    #[test]
    fn tie_breaking_is_deterministic_after_equal_points() {
        let history = vec![
            result("zara", "round-1", 1, "success", None),
            result("anna", "round-1", 1, "success", None),
        ];

        let standings = calculate_standings(
            &quest(QuestScoringMode::Points, TiePolicy::Competition),
            "run-1",
            &history,
        );

        assert_eq!(
            standings
                .iter()
                .map(|standing| standing.participant_id.as_str())
                .collect::<Vec<_>>(),
            ["anna", "zara"]
        );
        assert_eq!(standings[0].rank, 1);
        assert_eq!(standings[1].rank, 1);
    }

    #[test]
    fn dense_ties_do_not_leave_gaps() {
        let history = vec![
            result("alice", "round-1", 1, "success", None),
            result("bob", "round-1", 1, "success", None),
            result("cara", "round-1", 1, "failure", None),
        ];

        let standings = calculate_standings(
            &quest(QuestScoringMode::Points, TiePolicy::Dense),
            "run-1",
            &history,
        );

        assert_eq!(
            standings
                .iter()
                .map(|standing| standing.rank)
                .collect::<Vec<_>>(),
            [1, 1, 2]
        );
    }

    #[test]
    fn noncompetitive_mode_tracks_participation_without_points() {
        let history = vec![
            result("alice", "round-1", 1, "success", Some(1)),
            result("alice", "round-2", 2, "failure", Some(2)),
        ];

        let standings = calculate_standings(
            &quest(QuestScoringMode::NonCompetitive, TiePolicy::Competition),
            "run-1",
            &history,
        );

        assert_eq!(standings[0].points, 0.0);
        assert_eq!(standings[0].events, 2);
        assert_eq!(standings[0].successes, 1);
    }

    #[test]
    fn filters_other_quests_and_runs() {
        let mut unrelated = result("bob", "round-1", 1, "success", None);
        unrelated.quest_id = "other".into();
        let mut other_run = result("cara", "round-1", 1, "success", None);
        other_run.run_id = "run-2".into();

        let standings = calculate_standings(
            &quest(QuestScoringMode::Points, TiePolicy::Competition),
            "run-1",
            &[
                unrelated,
                other_run,
                result("alice", "round-1", 1, "success", None),
            ],
        );

        assert_eq!(standings.len(), 1);
        assert_eq!(standings[0].participant_id, "alice");
    }
}
