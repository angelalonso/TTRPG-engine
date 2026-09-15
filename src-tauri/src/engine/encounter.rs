use super::loader::{EncounterActionData, EncounterAttributeData, EncounterConfigData, EncounterOpponentData, GameCatalog};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterLogEntry {
    pub turn_number: u32, pub actor: String, pub action_id: String, pub success: bool,
    pub effects_applied: HashMap<String, f64>, pub text: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterResult {
    pub encounter_id: String,
    pub opponent_id: String,
    pub outcome: String, pub final_attribute_values: HashMap<String, HashMap<String, f64>>,
    pub consequences_applied: Vec<String>, pub full_log: Vec<EncounterLogEntry>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterState {
    pub encounter_id: String, pub opponent_id: String, pub turn: u32,
    pub current_actor: String, pub attributes: HashMap<String, HashMap<String, f64>>,
    pub cooldowns: HashMap<String, u32>, pub log: Vec<EncounterLogEntry>,
    pub finished: bool, pub outcome: Option<String>,
}

fn pairs(value: &str) -> HashMap<String, f64> {
    value.split(';').filter_map(|entry| {
        let mut parts = entry.splitn(2, ':');
        Some((parts.next()?.trim().to_string(), parts.next()?.trim().parse().ok()?))
    }).collect()
}
fn config<'a>(catalog: &'a GameCatalog, id: &str) -> Result<&'a EncounterConfigData, String> {
    catalog.encounter_configs.iter().find(|v| v.encounter_id == id).ok_or_else(|| format!("Unknown encounter: {id}"))
}
fn attr<'a>(catalog: &'a GameCatalog, id: &str) -> Option<&'a EncounterAttributeData> {
    catalog.encounter_attributes.iter().find(|v| v.attribute_id == id)
}
fn has_object(inventory: &[String], id: &str) -> bool {
    inventory.iter().any(|owned| owned == id || owned.starts_with(&format!("{id}_")))
}
fn available(catalog: &GameCatalog, state: &EncounterState, actor: &str, inventory: &[String]) -> Vec<EncounterActionData> {
    let profile_ids = if actor == "opponent" {
        catalog.encounter_opponents.iter().find(|v| v.opponent_id == state.opponent_id)
            .map(|v| v.available_action_ids.split(';').map(str::trim).collect::<Vec<_>>()).unwrap_or_default()
    } else { Vec::new() };
    catalog.encounter_actions.iter().filter(|a| {
        (actor == "player" || profile_ids.is_empty() || profile_ids.iter().any(|id| *id == a.action_id)) &&
        (a.usable_by == actor || a.usable_by == "both" || a.usable_by.is_empty()) &&
        state.cooldowns.get(&a.action_id).copied().unwrap_or(0) == 0 &&
        (a.requires_attribute_id.is_empty() || state.attributes[actor].get(&a.requires_attribute_id).copied().unwrap_or(0.0) >= a.requires_attribute_min.unwrap_or(0.0))
    }).filter(|a| a.requires_object_id.is_empty() || has_object(inventory, &a.requires_object_id) ||
        catalog.encounter_objects.iter().any(|o| o.object_id == a.requires_object_id && o.enables_action_id == a.action_id && has_object(inventory, &o.object_id))).filter(|a| {
        a.resource_cost_attribute_id.is_empty() || state.attributes[actor].get(&a.resource_cost_attribute_id).copied().unwrap_or(0.0) >= a.resource_cost_amount
    }).cloned().collect()
}
pub fn start(catalog: &GameCatalog, encounter_id: &str, opponent_id: &str, player_attributes: &HashMap<String, f64>) -> Result<EncounterState, String> {
    let opponent = catalog.encounter_opponents.iter().find(|v| v.opponent_id == opponent_id).ok_or_else(|| format!("Unknown opponent: {opponent_id}"))?;
    let rules = config(catalog, encounter_id)?;
    let mut attributes = HashMap::new();
    let mut player = player_attributes.clone();
    player.extend(pairs(&rules.player_starting_attributes));
    attributes.insert("player".into(), player);
    attributes.insert("opponent".into(), pairs(&opponent.starting_attributes));
    let current_actor = match rules.turn_order.as_str() {
        "opponent_first" => "opponent",
        "random" => if rand::rng().random::<bool>() { "player" } else { "opponent" },
        value if value.starts_with("initiative_attribute:") => {
            let id = value.trim_start_matches("initiative_attribute:");
            if attributes["player"].get(id).copied().unwrap_or(0.0) >= attributes["opponent"].get(id).copied().unwrap_or(0.0) { "player" } else { "opponent" }
        }
        _ => "player",
    }.into();
    Ok(EncounterState { encounter_id: encounter_id.into(), opponent_id: opponent_id.into(), turn: 0, current_actor, attributes, cooldowns: HashMap::new(), log: vec![], finished: false, outcome: None })
}
fn select_opponent(actions: &[EncounterActionData], profile: &EncounterOpponentData, state: &EncounterState) -> usize {
    if profile.strategy == "aggressive" {
        actions.iter().enumerate().max_by(|(_, a), (_, b)| a.effect_on_success.abs().partial_cmp(&b.effect_on_success.abs()).unwrap_or(std::cmp::Ordering::Equal)).map(|(i, _)| i).unwrap_or(0)
    } else if profile.strategy == "defensive" {
        actions.iter().enumerate().max_by(|(_, a), (_, b)| a.effect_on_success.partial_cmp(&b.effect_on_success).unwrap_or(std::cmp::Ordering::Equal)).map(|(i, _)| i).unwrap_or(0)
    } else {
        let _ = state;
        rand::rng().random_range(0..actions.len())
    }
}
pub fn play_turn(catalog: &GameCatalog, state: &mut EncounterState, action_id: Option<&str>, inventory: &mut Vec<String>) -> Result<(), String> {
    if state.finished { return Ok(()); }
    let actor = state.current_actor.clone();
    let attack_defense = config(catalog, &state.encounter_id)?.mode.eq_ignore_ascii_case("attack_defense");
    let actions = available(catalog, state, &actor, inventory);
    if actions.is_empty() {
        state.turn += 1;
        state.log.push(EncounterLogEntry {
            turn_number: state.turn, actor: actor.clone(), action_id: "pass".into(), success: true,
            effects_applied: HashMap::new(), text: format!("{actor} takes no available action."),
        });
        state.current_actor = if actor == "player" { "opponent".into() } else { "player".into() };
        return Ok(());
    }
    let action = if actor == "player" {
        actions.into_iter().find(|a| Some(a.action_id.as_str()) == action_id).ok_or_else(|| "Action is unavailable".to_string())?
    } else {
        let profile = catalog.encounter_opponents.iter().find(|v| v.opponent_id == state.opponent_id).unwrap();
        actions.get(select_opponent(&actions, profile, state)).cloned().ok_or_else(|| "No available opponent action".to_string())?
    };
    let modifier = if action.success_modifier_attribute_id.is_empty() { 0.0 } else { state.attributes[&actor].get(&action.success_modifier_attribute_id).copied().unwrap_or(0.0) * action.success_modifier_scale };
    let object_bonus: f64 = catalog.encounter_objects.iter().filter(|o| has_object(inventory, &o.object_id) &&
        (o.enables_action_id.is_empty() || o.enables_action_id == action.action_id)).map(|o| o.success_rate_bonus).sum();
    let rate = (action.base_success_rate + modifier + object_bonus).clamp(0.0, 1.0);
    let success = rand::rng().random::<f64>() < rate;
    let before_defence = if attack_defense && actor == "player" && success {
        rand::rng().random_range(0.0..=action.result_max.max(1.0))
    } else {
        0.0
    };
    if !action.resource_cost_attribute_id.is_empty() { *state.attributes.get_mut(&actor).unwrap().entry(action.resource_cost_attribute_id.clone()).or_default() -= action.resource_cost_amount; }
    let (target, delta) = if attack_defense && actor == "player" {
        let maximum = action.result_max.max(1.0);
        if !success || before_defence <= maximum * 0.1 {
            (&action.effect_on_failure_target, action.effect_on_failure)
        } else {
            let defence_rate = available(catalog, state, "opponent", inventory).iter()
                .map(|defender| defender.base_success_rate)
                .fold(0.0, f64::max)
                .clamp(0.0, 1.0);
            let reduction = if before_defence >= maximum { 0.0 } else {
                (1.0 - action.defense_reduction.max(defence_rate)).clamp(0.0, 1.0)
            };
            (&action.effect_on_success_target, action.effect_on_success * reduction)
        }
    } else if success {
        (&action.effect_on_success_target, action.effect_on_success)
    } else {
        (&action.effect_on_failure_target, action.effect_on_failure)
    };
    let target_actor = if target == "self" { actor.clone() } else { if actor == "player" { "opponent".into() } else { "player".into() } };
    let mut effects = HashMap::new();
    if !action.target_attribute_id.is_empty() {
        if let Some(value) = state.attributes.get_mut(&target_actor).and_then(|m| m.get_mut(&action.target_attribute_id)) {
            let bounds = attr(catalog, &action.target_attribute_id);
            let next = (*value + delta).clamp(bounds.map(|b| b.min_value).unwrap_or(f64::NEG_INFINITY), bounds.map(|b| b.max_value).unwrap_or(f64::INFINITY));
            *value = next; effects.insert(format!("{target_actor}.{}", action.target_attribute_id), delta);
        }
    }
    if action.consumes_object && !action.requires_object_id.is_empty() { if let Some(i) = inventory.iter().position(|id| id == &action.requires_object_id) { inventory.remove(i); } }
    for cooldown in state.cooldowns.values_mut() { *cooldown = cooldown.saturating_sub(1); }
    if action.cooldown_turns > 0 { state.cooldowns.insert(action.action_id.clone(), action.cooldown_turns); }
    state.turn += 1;
    let template = if success { &action.flavor_text_success } else { &action.flavor_text_failure };
    let text = if attack_defense && actor == "player" {
        format!(
            "{} (before defence: {:.1}/{:.1}; damage: {:.1})",
            template.replace("{actor}", &actor).replace("{target}", &target_actor).replace("{amount}", &delta.abs().to_string()),
            before_defence,
            action.result_max.max(1.0),
            delta.abs(),
        )
    } else {
        template.replace("{actor}", &actor).replace("{target}", &target_actor).replace("{amount}", &delta.abs().to_string())
    };
    state.log.push(EncounterLogEntry { turn_number: state.turn, actor: actor.clone(), action_id: action.action_id, success, effects_applied: effects, text });
    let config = config(catalog, &state.encounter_id)?;
    let defeated = |side: &str| catalog.encounter_attributes.iter().any(|a| a.is_loss_condition && state.attributes[side].get(&a.attribute_id).copied().unwrap_or(a.min_value) <= a.min_value);
    if defeated("player") || defeated("opponent") { state.finished = true; state.outcome = Some(if defeated("player") && defeated("opponent") { "draw".into() } else if defeated("opponent") { "win".into() } else { "lose".into() }); }
    else if config.max_turns > 0 && state.turn >= config.max_turns {
        state.finished = true;
        state.outcome = Some(if let Some(id) = config.tiebreaker.strip_prefix("highest_remaining:") {
            let player = state.attributes["player"].get(id).copied().unwrap_or(0.0);
            let opponent = state.attributes["opponent"].get(id).copied().unwrap_or(0.0);
            if player == opponent { "draw" } else if player > opponent { "win" } else { "lose" }
        } else { "draw" }.into());
    }
    state.current_actor = if actor == "player" { "opponent".into() } else { "player".into() };
    Ok(())
}
pub fn result(state: &EncounterState) -> Option<EncounterResult> {
    state.outcome.as_ref().map(|outcome| EncounterResult {
        encounter_id: state.encounter_id.clone(),
        opponent_id: state.opponent_id.clone(),
        outcome: outcome.clone(),
        final_attribute_values: state.attributes.clone(),
        consequences_applied: vec![],
        full_log: state.log.clone(),
    })
}
