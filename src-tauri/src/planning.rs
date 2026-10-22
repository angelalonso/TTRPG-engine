//! Read-only planning using the same transition and eligibility rules as play.

use crate::{
    advance_one_day, apply_bound_effects, apply_numeric_modifier_target, buy_object_for_sim,
    characteristic_value, event_upfront_stamina_cost, join_quest_for_sim, label,
    missing_object_prerequisites, obligation_due_on_day, owned_matches_definition,
    sponsor_entry_fee_covered, validate_event_entry, validate_player_started_event, ActiveEvent,
    GameState, ObligationState, QuestMembership,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceProjection {
    pub id: String,
    pub name: String,
    pub current: f64,
    pub projected: f64,
    pub minimum: f64,
    pub first_negative_day: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionDay {
    pub day: u32,
    pub values: BTreeMap<String, f64>,
    pub changes: BTreeMap<String, f64>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceForecast {
    pub start_day: u32,
    pub through_day: u32,
    pub requested_days: u32,
    pub resources: Vec<ResourceProjection>,
    pub days: Vec<ProjectionDay>,
    pub terminal_day: Option<u32>,
    pub omitted_uncertain_rules: usize,
}

fn projection_state(game: &GameState) -> GameState {
    let mut staged = game.clone();
    staged.catalog.cost_rules.retain(|rule| rule.probability >= 1.0);
    staged.catalog.effect_bindings.retain(|binding| binding.probability >= 1.0);
    if staged.catalog.labels.values.contains_key("sickness_daily_probability") {
        staged.catalog.labels.values.insert("sickness_daily_probability".into(), "0".into());
    }
    staged
}

pub fn resource_forecast(game: &GameState, days: u32) -> Result<ResourceForecast, String> {
    if !(1..=365).contains(&days) {
        return Err("Forecast horizon must be between 1 and 365 days".into());
    }
    if game.current_day.checked_add(days).is_none() {
        return Err("Forecast exceeds the supported calendar".into());
    }
    let mut staged = projection_state(game);
    let mut ids = [
        game.catalog.resource_roles.currency.clone(),
        game.catalog.resource_roles.recovery.clone(),
    ].into_iter().flatten().collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    let mut resources = ids.iter().map(|id| {
        let current = characteristic_value(game, id);
        ResourceProjection {
            id: id.clone(),
            name: game.catalog.player_characteristics.iter()
                .find(|definition| definition.id == *id)
                .map(|definition| definition.name.clone()).unwrap_or_else(|| id.clone()),
            current, projected: current, minimum: current,
            first_negative_day: (current < 0.0).then_some(game.current_day),
        }
    }).collect::<Vec<_>>();
    let mut result_days = Vec::new();
    let mut terminal_day = game.player.dead.then_some(game.current_day);
    for _ in 0..days {
        if staged.player.dead {
            break;
        }
        let previous = ids.iter().map(|id| (id.clone(), characteristic_value(&staged, id)))
            .collect::<BTreeMap<_, _>>();
        let log_start = staged.event_log.len();
        let alert_start = staged.pending_alerts.len();
        advance_one_day(&mut staged)?;
        let values = ids.iter().map(|id| (id.clone(), characteristic_value(&staged, id)))
            .collect::<BTreeMap<_, _>>();
        let changes = values.iter().map(|(id, value)| (id.clone(), value - previous[id]))
            .collect();
        let mut notes = staged.event_log[log_start..].iter()
            .map(|entry| entry.event.clone()).collect::<Vec<_>>();
        notes.extend(staged.pending_alerts[alert_start..].iter().map(|alert| alert.message.clone()));
        notes.sort();
        notes.dedup();
        for resource in &mut resources {
            let value = values[&resource.id];
            resource.projected = value;
            resource.minimum = resource.minimum.min(value);
            if value < 0.0 && resource.first_negative_day.is_none() {
                resource.first_negative_day = Some(staged.current_day);
            }
        }
        result_days.push(ProjectionDay { day: staged.current_day, values, changes, notes });
        if staged.player.dead {
            terminal_day = Some(staged.current_day);
        }
    }
    Ok(ResourceForecast {
        start_day: game.current_day, through_day: staged.current_day, requested_days: days,
        resources, days: result_days, terminal_day,
        omitted_uncertain_rules: game.catalog.cost_rules.len() - staged.catalog.cost_rules.len()
            + game.catalog.effect_bindings.len() - staged.catalog.effect_bindings.len(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObligationPreview {
    pub id: String,
    pub resource: String,
    pub amount: f64,
    pub interval: u32,
    pub interval_unit: String,
    pub due_days: String,
    pub first_due_day: Option<u32>,
    pub max_payments: u32,
    pub fault_limit: u32,
    pub fault_consequence: String,
    pub consequence_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentPreview {
    pub event_id: String,
    pub current_day: u32,
    pub requires_confirmation: bool,
    pub upfront_currency: f64,
    pub upfront_recovery: f64,
    pub conditional_payout: f64,
    pub obligations: Vec<ObligationPreview>,
    pub accepted_forecast: Option<ResourceForecast>,
}

pub fn commitment_preview(game: &GameState, event_id: &str) -> Result<CommitmentPreview, String> {
    let event = game.catalog.events.iter().find(|event| event.id == event_id)
        .ok_or_else(|| "Event not found in catalog".to_string())?;
    validate_player_started_event(game, event)?;
    let active = ActiveEvent {
        event_id: event.id.clone(), start_day: game.current_day,
        obligation_payments: 0, obligation_faults: 0, obligation_states: HashMap::new(),
    };
    let obligations = game.catalog.obligations.iter().filter(|entry| entry.event_id == event_id)
        .map(|entry| {
            let search_days = crate::obligation_interval_days(entry).saturating_add(7);
            let first_due_day = (1..=search_days).find_map(|offset| {
                let day = game.current_day.checked_add(offset)?;
                obligation_due_on_day(entry, &active, &ObligationState::default(), day).then_some(day)
            });
            ObligationPreview {
                id: entry.id.clone(), resource: entry.resource.clone(), amount: entry.amount,
                interval: entry.interval, interval_unit: entry.interval_unit.clone(),
                due_days: entry.due_days.clone(), first_due_day,
                max_payments: entry.max_payments, fault_limit: entry.fault_limit,
                fault_consequence: entry.fault_consequence.clone(),
                consequence_message: entry.limit_message.clone(),
            }
        }).collect::<Vec<_>>();
    let upfront_recovery = event_upfront_stamina_cost(game, event);
    let conditional_payout = if event.payout_freq_type.eq_ignore_ascii_case("recurring") {
        0.0
    } else {
        apply_numeric_modifier_target(game, "action_payout", event.payout)?.max(0.0)
    };
    let requires_confirmation = !obligations.is_empty() || event.base_cost > 0.0;
    let accepted_forecast = if obligations.is_empty() {
        None
    } else {
        let mut staged = projection_state(game);
        crate::adjust_characteristic(&mut staged, "budget", -event.base_cost + conditional_payout);
        crate::adjust_characteristic(&mut staged, "stamina", -upfront_recovery);
        staged.player.last_event_day = Some(staged.current_day);
        apply_bound_effects(&mut staged, "event_started", event_id, "")?;
        staged.player.active_events.push(active);
        Some(resource_forecast(&staged, 30)?)
    };
    Ok(CommitmentPreview {
        event_id: event_id.into(), current_day: game.current_day, requires_confirmation,
        upfront_currency: event.base_cost, upfront_recovery, conditional_payout,
        obligations, accepted_forecast,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestJoinEligibility {
    pub quest_id: String,
    pub joined: bool,
    pub can_join: bool,
    pub reason: String,
}

pub fn quest_join_eligibility(game: &GameState, quest_id: &str) -> QuestJoinEligibility {
    let joined = game.quest_memberships.iter().any(|membership| membership.quest_id == quest_id);
    let result = join_quest_for_sim(&mut game.clone(), quest_id);
    QuestJoinEligibility {
        quest_id: quest_id.into(), joined, can_join: result.is_ok(),
        reason: result.err().unwrap_or_default(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedAcquisition {
    pub object_id: String,
    pub name: String,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPreparation {
    pub selection_name: String,
    pub purchases: Vec<PlannedAcquisition>,
    pub acquisition_cost: f64,
    pub entry_fee: f64,
    pub total_cost: f64,
    pub shortfall: f64,
    pub ready_after_preparation: bool,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedEvent {
    pub event_id: String,
    pub name: String,
    pub next_day: u32,
    pub preparations: Vec<EntryPreparation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestEntryPlan {
    pub quest_id: String,
    pub current_day: u32,
    pub eligibility: QuestJoinEligibility,
    pub join_fee: f64,
    pub events: Vec<PlannedEvent>,
}

fn ensure_object(
    staged: &mut GameState,
    id: &str,
    purchases: &mut Vec<PlannedAcquisition>,
    visiting: &mut HashSet<String>,
) -> Result<(), String> {
    if staged.player.inventory.iter().any(|object| owned_matches_definition(object, id)) {
        return Ok(());
    }
    if !visiting.insert(id.into()) {
        return Err(format!("Cyclic object prerequisite: {id}"));
    }
    let definition = staged.catalog.objects.iter().find(|object| object.id == id).cloned()
        .ok_or_else(|| format!("Unknown object prerequisite: {id}"))?;
    for prerequisite in missing_object_prerequisites(staged, &definition) {
        ensure_object(staged, &prerequisite, purchases, visiting)?;
    }
    let before = characteristic_value(staged, "budget");
    buy_object_for_sim(staged, id)?;
    purchases.push(PlannedAcquisition {
        object_id: id.into(), name: definition.name,
        cost: before - characteristic_value(staged, "budget"),
    });
    visiting.remove(id);
    Ok(())
}

fn prepare_group(staged: &mut GameState, group: &str, purchases: &mut Vec<PlannedAcquisition>) -> Result<(), String> {
    let ids = group.split('|').map(str::trim).filter(|id| !id.is_empty()).collect::<Vec<_>>();
    let alternatives = staged.catalog.objects.iter().filter(|object| ids.iter().any(|id| {
        object.id == *id || object.requirement_group == *id
    })).map(|object| object.id.clone()).collect::<Vec<_>>();
    if staged.player.inventory.iter().any(|owned| alternatives.iter().any(|id| owned_matches_definition(owned, id))) {
        return Ok(());
    }
    let mut choices = Vec::new();
    let mut errors = Vec::new();
    for id in alternatives {
        let mut candidate = staged.clone();
        let mut additions = Vec::new();
        match ensure_object(&mut candidate, &id, &mut additions, &mut HashSet::new()) {
            Ok(()) => choices.push((candidate, additions)),
            Err(error) => errors.push(error),
        }
    }
    choices.sort_by(|left, right| {
        let cost = |items: &[PlannedAcquisition]| items.iter().map(|item| item.cost).sum::<f64>();
        cost(&left.1).total_cmp(&cost(&right.1))
    });
    if let Some((candidate, additions)) = choices.into_iter().next() {
        *staged = candidate;
        purchases.extend(additions);
        Ok(())
    } else {
        Err(format!("{group}: {}", if errors.is_empty() { "no matching definitions".into() } else { errors.join("; ") }))
    }
}

pub fn quest_entry_plan(game: &GameState, quest_id: &str) -> Result<QuestEntryPlan, String> {
    let quest = game.catalog.quests.iter().find(|quest| quest.id == quest_id)
        .ok_or_else(|| "Quest not found in catalog".to_string())?;
    if game.days_per_year == 0 {
        return Err("Calendar must contain at least one day".into());
    }
    let eligibility = quest_join_eligibility(game, quest_id);
    let join_fee = if eligibility.joined { 0.0 } else { quest.join_fee };
    let object_groups = label(&game.catalog, &format!("quest_planning_object_groups:{quest_id}"),
        &label(&game.catalog, "quest_planning_object_groups", ""));
    let available_funding = characteristic_value(game, "budget");
    // Synthetic funding is only used to estimate prerequisites; shortfall is reported separately.
    let estimate_funding = game.catalog.objects.iter().map(|object| object.price.max(object.license_fee).max(0.0))
        .sum::<f64>() + game.catalog.events.iter().map(|event| event.entry_fee.max(0.0)).sum::<f64>()
        + join_fee + available_funding.max(0.0) + 1.0;
    let mut events = Vec::new();
    for event in game.catalog.events.iter().filter(|event| event.quest_id == quest_id && event.day_of_year > 0) {
        let year_start = (game.current_day.saturating_sub(1) / game.days_per_year) * game.days_per_year;
        let mut next_day = year_start.checked_add(event.day_of_year).ok_or("Calendar overflow")?;
        if next_day < game.current_day {
            next_day = next_day.checked_add(game.days_per_year).ok_or("Calendar overflow")?;
        }
        let mut candidates = game.catalog.objects.iter().filter(|object| {
            event.required_object_ids.trim().is_empty()
                || event.required_object_ids.split(';').any(|id| id.trim() == object.id)
        }).map(|object| Some(object.id.clone())).collect::<Vec<_>>();
        candidates.push(None);
        let mut preparations = Vec::new();
        for candidate in candidates {
            let mut staged = projection_state(game);
            if let Some(resource) = staged.catalog.resource_roles.currency.clone() {
                staged.player.characteristics.insert(resource, estimate_funding);
            }
            let mut purchases = Vec::new();
            let mut blockers = Vec::new();
            for id in [&quest.required_license_id, &event.required_license_id].into_iter().filter(|id| !id.trim().is_empty()) {
                if let Err(error) = ensure_object(&mut staged, id, &mut purchases, &mut HashSet::new()) {
                    blockers.push(error);
                }
            }
            for group in object_groups.split(';').map(str::trim).filter(|group| !group.is_empty()) {
                if let Err(error) = prepare_group(&mut staged, group, &mut purchases) {
                    blockers.push(error);
                }
            }
            if let Some(id) = &candidate {
                if let Err(error) = ensure_object(&mut staged, id, &mut purchases, &mut HashSet::new()) {
                    blockers.push(error);
                }
            }
            if !eligibility.joined {
                if let Err(error) = join_quest_for_sim(&mut staged, quest_id) {
                    blockers.push(error);
                    staged.quest_memberships.push(QuestMembership { quest_id: quest_id.into(), joined_day: game.current_day });
                }
            }
            staged.current_day = next_day;
            let selection = candidate.as_ref().and_then(|id| staged.player.inventory.iter()
                .find(|owned| owned_matches_definition(owned, id))).map(|owned| owned.id.as_str()).unwrap_or("");
            if let Err(error) = validate_event_entry(&staged, event, selection) {
                blockers.push(error);
            }
            blockers.sort();
            blockers.dedup();
            let acquisition_cost = purchases.iter().map(|item| item.cost).sum::<f64>();
            let entry_fee = if sponsor_entry_fee_covered(game, event) { 0.0 } else { event.entry_fee };
            let total_cost = acquisition_cost + join_fee + entry_fee;
            let selection_name = candidate.as_ref().and_then(|id| game.catalog.objects.iter().find(|object| object.id == *id))
                .map(|object| object.name.clone()).unwrap_or_else(|| label(&game.catalog, "planning_no_selection", "No selected object"));
            preparations.push(EntryPreparation {
                selection_name, purchases, acquisition_cost, entry_fee, total_cost,
                shortfall: (total_cost - available_funding).max(0.0),
                ready_after_preparation: blockers.is_empty(), blockers,
            });
        }
        preparations.sort_by(|left, right| right.ready_after_preparation.cmp(&left.ready_after_preparation)
            .then_with(|| left.total_cost.total_cmp(&right.total_cost)));
        preparations.truncate(3);
        events.push(PlannedEvent { event_id: event.id.clone(), name: event.name.clone(), next_day, preparations });
    }
    events.sort_by_key(|event| event.next_day);
    Ok(QuestEntryPlan { quest_id: quest_id.into(), current_day: game.current_day, eligibility, join_fee, events })
}
