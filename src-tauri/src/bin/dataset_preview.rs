use std::collections::HashMap;
use std::env;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ttrpg_engine_lib::engine::conditions::{ConditionSet, ExplanationNode};
use ttrpg_engine_lib::engine::facts::{
    characteristic, evaluate_fact, event_completed, object_count, quest_joined, ObjectQuery,
    StateFact,
};
use ttrpg_engine_lib::engine::loader::{
    validate_dataset_directory, DatasetValidationReport, UnsupportedField,
};
use ttrpg_engine_lib::engine::modifiers::NumericModifierContribution;
use ttrpg_engine_lib::engine::schema::capability_metadata;
use ttrpg_engine_lib::headless::HeadlessGame;
use ttrpg_engine_lib::{
    new_game_seeded, numeric_modifier_breakdown_for_sim, EventHistory, OwnedObject, QuestMembership,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Validate,
    Capabilities,
    Explain,
    Condition,
    Modifier,
    Execute,
}

#[derive(Debug, PartialEq)]
enum Operation {
    AdvanceDay,
    Buy(String),
    Sell(String),
    Start(String),
    Enter {
        event_id: String,
        object_id: String,
    },
    Rent {
        event_id: String,
        object_id: String,
    },
    JoinQuest(String),
    ResolveEncounter(Option<String>),
    Submit {
        entry_id: String,
        result: String,
        player_position: Option<u32>,
    },
    Sequence(Vec<Operation>),
}

#[derive(Debug, PartialEq)]
struct Arguments {
    dataset: String,
    seed: u64,
    mode: Mode,
    expression: Option<String>,
    condition_group: Option<String>,
    modifier_target: Option<String>,
    modifier_base: Option<f64>,
    operation: Option<Operation>,
    sample_state: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct SampleState {
    #[serde(default)]
    characteristics: HashMap<String, f64>,
    #[serde(default)]
    inventory: Vec<String>,
    #[serde(default)]
    history: Vec<SampleHistory>,
    #[serde(default)]
    quests: Vec<String>,
    current_day: Option<u32>,
    age_days: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
struct SampleHistory {
    event_id: String,
    #[serde(default = "default_sample_result")]
    result: String,
    #[serde(default)]
    outcome: String,
    #[serde(default)]
    object_id: String,
    entered_day: Option<u32>,
}

fn default_sample_result() -> String {
    "success".to_string()
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
    usage: &'static str,
}

#[derive(Debug, Serialize)]
struct ValidationResponse {
    ok: bool,
    mode: &'static str,
    dataset: String,
    errors: Vec<String>,
    warnings: Vec<String>,
    unsupported_fields: Vec<UnsupportedField>,
}

#[derive(Debug, Serialize)]
struct ExplanationResponse {
    ok: bool,
    mode: &'static str,
    dataset: String,
    seed: u64,
    expression: String,
    fact: Value,
    result: bool,
    explanation: String,
}

#[derive(Debug, Serialize)]
struct ConditionResponse {
    ok: bool,
    mode: &'static str,
    dataset: String,
    seed: u64,
    group: String,
    explanation: ExplanationNode,
}

#[derive(Debug, Serialize)]
struct ModifierResponse {
    ok: bool,
    mode: &'static str,
    dataset: String,
    seed: u64,
    target: String,
    base: f64,
    contributions: Vec<NumericModifierContribution>,
    final_value: f64,
}

#[derive(Debug, PartialEq, Serialize)]
struct StateDiff {
    path: String,
    before: Option<Value>,
    after: Option<Value>,
}

#[derive(Debug, Serialize)]
struct ExecuteResponse {
    ok: bool,
    mode: &'static str,
    dataset: String,
    seed: u64,
    operation: String,
    diagnostics: Vec<String>,
    result: Option<Value>,
    state_diff: Vec<StateDiff>,
}

fn usage() -> &'static str {
    "dataset_preview [--dataset PATH] [--seed N] [--sample-state PATH] (--validate | --capabilities | --explain FACT | --condition GROUP | --modifier TARGET BASE | --execute OPERATION)\n\
FACT forms: characteristic:<id>==<number>, object_count:<id>==<number>, \
object_type:<type>==<number>, active_event_count[:<id>]==<number>, \
event_completed:<id>[:success|failure]==0|1, quest_joined:<id>==0|1, \
calendar_day==<number>, age_days==<number>\n\
CONDITION form: --condition CONDITION_GROUP_ID\n\
MODIFIER form: --modifier TARGET BASE\n\
SAMPLE STATE JSON: {\"characteristics\": {\"budget\": 100}, \"inventory\": [\"object_id\"], \
\"history\": [{\"event_id\": \"event_id\", \"result\": \"success\"}], \
\"quests\": [\"quest_id\"], \"current_day\": 3, \"age_days\": 365}\n\
OPERATION forms: advance_day | buy:<object_id> | sell:<object_id> | \
start:<event_id> | enter:<event_id>:<object_id> | rent:<event_id>:<object_id> | \
join_quest:<quest_id> | encounter[:<action_id>] | \
submit:<entry_id>:<result>[:<player_position>]\
SEQUENCE form: --execute-sequence OPERATION;OPERATION;..."
}

fn parse_args<I>(args: I) -> Result<Arguments, String>
where
    I: IntoIterator<Item = String>,
{
    let mut dataset = "dataset".to_string();
    let mut seed = 1;
    let mut mode = None;
    let mut expression = None;
    let mut condition_group = None;
    let mut modifier_target = None;
    let mut modifier_base = None;
    let mut operation = None;
    let mut sample_state = None;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dataset" => {
                dataset = args
                    .next()
                    .ok_or_else(|| "--dataset requires a path".to_string())?;
            }
            "--seed" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--seed requires an unsigned integer".to_string())?;
                seed = value
                    .parse()
                    .map_err(|_| format!("invalid --seed value: {value}"))?;
            }
            "--sample-state" => {
                let path = args
                    .next()
                    .ok_or_else(|| "--sample-state requires a JSON path".to_string())?;
                sample_state = Some(path);
            }
            "--validate" => {
                select_mode(&mut mode, Mode::Validate)?;
            }
            "--capabilities" => {
                select_mode(&mut mode, Mode::Capabilities)?;
            }
            "--explain" | "--fact" => {
                select_mode(&mut mode, Mode::Explain)?;
                expression = Some(
                    args.next()
                        .ok_or_else(|| "--explain requires a fact expression".to_string())?,
                );
            }
            "--condition" => {
                select_mode(&mut mode, Mode::Condition)?;
                condition_group = Some(
                    args.next()
                        .ok_or_else(|| "--condition requires a group id".to_string())?,
                );
            }
            "--modifier" => {
                select_mode(&mut mode, Mode::Modifier)?;
                modifier_target = Some(
                    args.next()
                        .ok_or_else(|| "--modifier requires a target".to_string())?,
                );
                let base = args
                    .next()
                    .ok_or_else(|| "--modifier requires a base value".to_string())?;
                modifier_base = Some(
                    base.parse()
                        .map_err(|_| format!("invalid modifier base value: {base}"))?,
                );
            }
            "--execute" | "--propose" | "--proposed-operation" => {
                select_mode(&mut mode, Mode::Execute)?;
                operation =
                    Some(parse_operation(&args.next().ok_or_else(|| {
                        "--execute requires an operation".to_string()
                    })?)?);
            }
            "--execute-sequence" => {
                select_mode(&mut mode, Mode::Execute)?;
                let expression = args
                    .next()
                    .ok_or_else(|| "--execute-sequence requires operations".to_string())?;
                let operations = expression
                    .split(';')
                    .map(parse_operation)
                    .collect::<Result<Vec<_>, _>>()?;
                if operations.is_empty() {
                    return Err("--execute-sequence requires at least one operation".to_string());
                }
                operation = Some(Operation::Sequence(operations));
            }
            "--help" | "-h" => return Err(usage().to_string()),
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let mode = mode.ok_or_else(|| {
        "one of --validate, --capabilities, --explain, --condition, --modifier, or --execute is required"
            .to_string()
    })?;
    if mode != Mode::Explain && expression.is_some() {
        return Err("an explanation expression is only valid with --explain".to_string());
    }
    if sample_state.is_some() && !matches!(mode, Mode::Explain | Mode::Condition | Mode::Execute) {
        return Err(
            "--sample-state is only valid with --explain, --condition, or --execute".to_string(),
        );
    }
    Ok(Arguments {
        dataset,
        seed,
        mode,
        expression,
        condition_group,
        modifier_target,
        modifier_base,
        operation,
        sample_state,
    })
}

fn operation_part(value: &str, kind: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        Err(format!("{kind} operation requires an id"))
    } else if value.contains(':') {
        Err(format!("{kind} operation ids cannot contain ':'"))
    } else {
        Ok(value.to_string())
    }
}

fn parse_operation(expression: &str) -> Result<Operation, String> {
    let parts: Vec<_> = expression.split(':').collect();
    match parts.as_slice() {
        ["advance_day"] => Ok(Operation::AdvanceDay),
        ["buy", id] => Ok(Operation::Buy(operation_part(id, "buy")?)),
        ["sell", id] => Ok(Operation::Sell(operation_part(id, "sell")?)),
        ["start", id] | ["start_event", id] => Ok(Operation::Start(operation_part(id, "start")?)),
        ["enter", event_id, object_id] | ["enter_event", event_id, object_id] => {
            Ok(Operation::Enter {
                event_id: operation_part(event_id, "enter")?,
                object_id: operation_part(object_id, "enter")?,
            })
        }
        ["rent", event_id, object_id] | ["rent_event", event_id, object_id] => {
            Ok(Operation::Rent {
                event_id: operation_part(event_id, "rent")?,
                object_id: operation_part(object_id, "rent")?,
            })
        }
        ["encounter"] | ["resolve_encounter"] => Ok(Operation::ResolveEncounter(None)),
        ["encounter", action_id] | ["resolve_encounter", action_id] => Ok(
            Operation::ResolveEncounter(Some(operation_part(action_id, "encounter")?)),
        ),
        ["join_quest", id] | ["join", id] => {
            Ok(Operation::JoinQuest(operation_part(id, "join_quest")?))
        }
        ["submit", entry_id, result] => Ok(Operation::Submit {
            entry_id: operation_part(entry_id, "submit")?,
            result: operation_part(result, "submit")?,
            player_position: None,
        }),
        ["submit", entry_id, result, position] => {
            let player_position = position
                .parse::<u32>()
                .map_err(|_| "submit player position must be an unsigned integer".to_string())?;
            Ok(Operation::Submit {
                entry_id: operation_part(entry_id, "submit")?,
                result: operation_part(result, "submit")?,
                player_position: Some(player_position),
            })
        }
        _ => Err(format!(
            "unsupported operation syntax: {expression}; see --help for supported forms"
        )),
    }
}

fn read_sample_state(path: &str) -> Result<SampleState, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("could not read sample state '{path}': {error}"))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("could not parse sample state '{path}': {error}"))
}

fn sample_owned_object(
    game: &HeadlessGame,
    definition_id: &str,
    index: usize,
) -> Result<OwnedObject, String> {
    let object = game
        .state()
        .catalog
        .objects
        .iter()
        .find(|object| object.id == definition_id)
        .ok_or_else(|| format!("sample inventory object not found: {definition_id}"))?;
    let mut value = serde_json::to_value(object).expect("object data must serialize");
    let fields = value
        .as_object_mut()
        .expect("object data must serialize as an object");
    fields.insert(
        "id".into(),
        format!("sample-{index}-{definition_id}").into(),
    );
    fields.insert("definition_id".into(), definition_id.into());
    fields.insert(
        "instance_id".into(),
        format!("sample-{index}-{definition_id}").into(),
    );
    if let Some(object_type) = fields.remove("type") {
        fields.insert("object_type".into(), object_type);
    }
    for service in 1..=4 {
        fields.insert(format!("service_{service}_needed"), false.into());
    }
    fields.insert("purchase_day".into(), game.state().current_day.into());
    fields.insert("expires_day".into(), 0.into());
    fields.insert("unavailable_until_day".into(), 0.into());
    fields.insert("loaned".into(), false.into());
    serde_json::from_value(value)
        .map_err(|error| format!("invalid sample inventory object '{definition_id}': {error}"))
}

fn stage_sample_state(game: &mut HeadlessGame, sample: &SampleState) -> Result<(), String> {
    if let Some(day) = sample.current_day {
        game.state_mut().current_day = day;
    }
    if let Some(age_days) = sample.age_days {
        game.state_mut().player.age_days = age_days;
    }
    for (id, value) in &sample.characteristics {
        game.state_mut()
            .player
            .characteristics
            .insert(id.clone(), *value);
    }
    if !sample.inventory.is_empty() {
        let inventory = sample
            .inventory
            .iter()
            .enumerate()
            .map(|(index, id)| sample_owned_object(game, id, index))
            .collect::<Result<Vec<_>, _>>()?;
        game.state_mut().player.inventory = inventory;
    }
    if !sample.quests.is_empty() {
        game.state_mut().quest_memberships = sample
            .quests
            .iter()
            .map(|quest_id| QuestMembership {
                quest_id: quest_id.clone(),
                joined_day: game.state().current_day,
            })
            .collect();
    }
    if !sample.history.is_empty() {
        game.state_mut().event_history = sample
            .history
            .iter()
            .enumerate()
            .map(|(index, history)| EventHistory {
                id: format!("sample-history-{index}"),
                event_id: history.event_id.clone(),
                object_id: history.object_id.clone(),
                entered_day: history.entered_day.unwrap_or(game.state().current_day),
                result: history.result.clone(),
                outcome: history.outcome.clone(),
                reward_awarded: 0.0,
                charisma_reward_awarded: 0.0,
                damage_type: String::new(),
            })
            .collect();
    }
    Ok(())
}

fn staged_game(args: &Arguments) -> Result<HeadlessGame, String> {
    let mut game = HeadlessGame::new(&args.dataset, args.seed);
    if let Some(path) = args.sample_state.as_deref() {
        let sample = read_sample_state(path)?;
        stage_sample_state(&mut game, &sample)?;
    }
    Ok(game)
}

fn condition(args: &Arguments, group: &str) -> Result<ConditionResponse, String> {
    let game = staged_game(args)?;
    let set = ConditionSet::from_rows(
        &game.state().catalog.condition_groups,
        &game.state().catalog.conditions,
    )
    .map_err(|errors| format!("invalid condition contract: {errors:?}"))?;
    let explanation = set
        .evaluate(game.state(), group)
        .map_err(|error| format!("condition evaluation failed: {error:?}"))?;
    Ok(ConditionResponse {
        ok: true,
        mode: "condition",
        dataset: args.dataset.clone(),
        seed: args.seed,
        group: group.to_string(),
        explanation,
    })
}

fn select_mode(mode: &mut Option<Mode>, next: Mode) -> Result<(), String> {
    if mode.replace(next).is_some() {
        return Err(
            "choose exactly one of --validate, --capabilities, --explain, --condition, --modifier, or --execute"
                .to_string(),
        );
    }
    Ok(())
}

fn parse_fact(expression: &str) -> Result<StateFact, String> {
    let (kind, comparison) = expression
        .split_once("==")
        .ok_or_else(|| "fact must use ==, for example characteristic:budget==0".to_string())?;
    let value = comparison
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("fact value is not a number: {}", comparison.trim()))?;
    let kind = kind.trim();

    if let Some(id) = kind.strip_prefix("characteristic:") {
        let id = id.trim();
        if id.is_empty() {
            return Err("characteristic fact requires an id".to_string());
        }
        return Ok(StateFact::Characteristic {
            id: id.to_string(),
            value,
        });
    }
    if let Some(id) = kind.strip_prefix("object_count:") {
        let id = id.trim();
        if id.is_empty() {
            return Err("object_count fact requires an object definition id".to_string());
        }
        let count = non_negative_count(value)?;
        return Ok(StateFact::ObjectCount {
            query: ObjectQuery {
                definition_id: Some(id.to_string()),
                ..ObjectQuery::default()
            },
            count,
        });
    }
    if let Some(object_type) = kind.strip_prefix("object_type:") {
        let object_type = object_type.trim();
        if object_type.is_empty() {
            return Err("object_type fact requires a type".to_string());
        }
        return Ok(StateFact::ObjectCount {
            query: ObjectQuery {
                object_type: Some(object_type.to_string()),
                ..ObjectQuery::default()
            },
            count: non_negative_count(value)?,
        });
    }
    if let Some(event) = kind.strip_prefix("event_completed:") {
        let (event_id, result) = event.split_once(':').unwrap_or((event, ""));
        if event_id.trim().is_empty() {
            return Err("event_completed fact requires an event id".to_string());
        }
        let success = match result.trim().to_ascii_lowercase().as_str() {
            "" | "success" => true,
            "failure" => false,
            other => return Err(format!("unsupported event result: {other}")),
        };
        return Ok(StateFact::EventCompleted {
            event_id: event_id.trim().to_string(),
            success,
        });
    }
    if let Some(quest_id) = kind.strip_prefix("quest_joined:") {
        return Ok(StateFact::QuestJoined {
            quest_id: required_id(quest_id, "quest_joined")?,
            joined: boolean_value(value)?,
        });
    }
    if let Some(event_id) = kind.strip_prefix("active_event_count:") {
        return Ok(StateFact::ActiveEventCount {
            event_id: Some(required_id(event_id, "active_event_count")?),
            count: non_negative_count(value)?,
        });
    }
    if kind == "active_event_count" {
        return Ok(StateFact::ActiveEventCount {
            event_id: None,
            count: non_negative_count(value)?,
        });
    }
    if kind == "calendar_day" {
        return Ok(StateFact::CalendarDay(non_negative_count(value)? as u32));
    }
    if kind == "age_days" {
        return Ok(StateFact::AgeDays(non_negative_count(value)? as u32));
    }
    Err(format!("unsupported fact: {kind}"))
}

fn required_id(value: &str, kind: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        Err(format!("{kind} fact requires an id"))
    } else {
        Ok(value.to_string())
    }
}

fn boolean_value(value: f64) -> Result<bool, String> {
    match value {
        0.0 => Ok(false),
        1.0 => Ok(true),
        _ => Err("boolean facts must use 0 or 1".to_string()),
    }
}

fn non_negative_count(value: f64) -> Result<usize, String> {
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 {
        return Err("count and day values must be non-negative integers".to_string());
    }
    if value > usize::MAX as f64 {
        return Err("count is too large".to_string());
    }
    Ok(value as usize)
}

fn fact_json(fact: &StateFact) -> Value {
    serde_json::to_value(fact).expect("StateFact must serialize")
}

fn explain(args: &Arguments, expression: &str) -> Result<ExplanationResponse, String> {
    let fact = parse_fact(expression)?;
    let game = staged_game(args)?;
    let result = evaluate_fact(game.state(), fact.clone()).map_err(|error| format!("{error:?}"))?;
    let explanation = match &fact {
        StateFact::Characteristic { id, value } => {
            let actual = characteristic(game.state(), id).map_err(|error| format!("{error:?}"))?;
            format!("characteristic {id} is {actual}; expected {value}")
        }
        StateFact::ObjectCount { query, count } => {
            let actual = object_count(game.state(), query).map_err(|error| format!("{error:?}"))?;
            format!(
                "object count for {} is {actual}; expected {count}",
                query.definition_id.as_deref().unwrap_or("<any>")
            )
        }
        StateFact::EventCompleted {
            event_id, success, ..
        } => {
            let actual = event_completed(game.state(), event_id, Some(*success))
                .map_err(|error| format!("{error:?}"))?;
            format!("event {event_id} completed with expected result {success}; actual {actual}")
        }
        StateFact::QuestJoined { quest_id, joined } => {
            let actual =
                quest_joined(game.state(), quest_id).map_err(|error| format!("{error:?}"))?;
            format!("quest {quest_id} joined state is {actual}; expected {joined}")
        }
        StateFact::ActiveEventCount { event_id, count } => {
            let actual = ttrpg_engine_lib::engine::facts::active_event_count(
                game.state(),
                event_id.as_deref(),
            );
            format!(
                "active event count for {} is {actual}; expected {count}",
                event_id.as_deref().unwrap_or("<any>")
            )
        }
        StateFact::CalendarDay(day) => {
            format!(
                "calendar day is {}; expected {day}",
                game.state().current_day
            )
        }
        StateFact::AgeDays(days) => {
            format!(
                "age is {} days; expected {days}",
                game.state().player.age_days
            )
        }
    };
    Ok(ExplanationResponse {
        ok: true,
        mode: "explain",
        dataset: args.dataset.clone(),
        seed: args.seed,
        expression: expression.to_string(),
        fact: fact_json(&fact),
        result,
        explanation,
    })
}

fn validation(args: &Arguments, report: DatasetValidationReport) -> ValidationResponse {
    ValidationResponse {
        ok: report.is_valid(),
        mode: "validate",
        dataset: args.dataset.clone(),
        errors: report.errors,
        warnings: report.warnings,
        unsupported_fields: report.unsupported_fields,
    }
}

fn modifier(args: &Arguments, target: &str, base: f64) -> Result<ModifierResponse, String> {
    let game = new_game_seeded(&args.dataset, args.seed);
    let result = numeric_modifier_breakdown_for_sim(&game, target, base)?;
    Ok(ModifierResponse {
        ok: true,
        mode: "modifier",
        dataset: args.dataset.clone(),
        seed: args.seed,
        target: target.to_string(),
        base: result.base,
        contributions: result.contributions,
        final_value: result.final_value,
    })
}

fn diff_values(path: &str, before: &Value, after: &Value, output: &mut Vec<StateDiff>) {
    match (before, after) {
        (Value::Object(left), Value::Object(right)) => {
            let keys: std::collections::BTreeSet<_> = left.keys().chain(right.keys()).collect();
            for key in keys {
                let child_path = if path.is_empty() {
                    key.to_string()
                } else {
                    format!("{path}.{key}")
                };
                match (left.get(key), right.get(key)) {
                    (Some(left), Some(right)) => diff_values(&child_path, left, right, output),
                    (left, right) if left != right => output.push(StateDiff {
                        path: child_path,
                        before: left.cloned(),
                        after: right.cloned(),
                    }),
                    _ => {}
                }
            }
        }
        (left, right) if left != right => output.push(StateDiff {
            path: path.to_string(),
            before: Some(left.clone()),
            after: Some(right.clone()),
        }),
        _ => {}
    }
}

fn execute(args: &Arguments, operation: &Operation) -> ExecuteResponse {
    let mut game = HeadlessGame::new(&args.dataset, args.seed);
    if let Some(path) = args.sample_state.as_deref() {
        let staging =
            read_sample_state(path).and_then(|sample| stage_sample_state(&mut game, &sample));
        if let Err(error) = staging {
            return ExecuteResponse {
                ok: false,
                mode: "execute",
                dataset: args.dataset.clone(),
                seed: args.seed,
                operation: operation.to_string(),
                diagnostics: vec![error],
                result: None,
                state_diff: Vec::new(),
            };
        }
    }
    let before = serde_json::to_value(game.state()).expect("game state must serialize");
    let mut diagnostics = Vec::new();
    let result = match operation {
        Operation::Sequence(operations) => {
            let mut results = Vec::with_capacity(operations.len());
            for (index, operation) in operations.iter().enumerate() {
                match apply_operation(&mut game, operation) {
                    Ok(result) => {
                        diagnostics.push(format!(
                            "step {}/{} applied: {operation}",
                            index + 1,
                            operations.len()
                        ));
                        results.push(result.unwrap_or(Value::Null));
                    }
                    Err(error) => {
                        diagnostics.push(format!(
                            "step {}/{} failed ({operation}): {error}",
                            index + 1,
                            operations.len()
                        ));
                        return ExecuteResponse {
                            ok: false,
                            mode: "execute",
                            dataset: args.dataset.clone(),
                            seed: args.seed,
                            operation: operation.to_string(),
                            diagnostics,
                            result: None,
                            state_diff: Vec::new(),
                        };
                    }
                }
            }
            Ok(Some(Value::Array(results)))
        }
        operation => apply_operation(&mut game, operation),
    };

    let (ok, result) = match result {
        Ok(result) => {
            diagnostics.push("operation applied to a cloned seeded state".to_string());
            (true, result)
        }
        Err(error) => {
            diagnostics.push(error);
            (false, None)
        }
    };
    let after = serde_json::to_value(game.state()).expect("game state must serialize");
    let mut state_diff = Vec::new();
    if ok {
        diff_values("", &before, &after, &mut state_diff);
        diagnostics.push(format!("{} state field(s) changed", state_diff.len()));
    }
    ExecuteResponse {
        ok,
        mode: "execute",
        dataset: args.dataset.clone(),
        seed: args.seed,
        operation: operation.to_string(),
        diagnostics,
        result,
        state_diff,
    }
}

fn apply_operation(
    game: &mut HeadlessGame,
    operation: &Operation,
) -> Result<Option<Value>, String> {
    match operation {
        Operation::AdvanceDay => game.advance_day().map(|_| None),
        Operation::Buy(object_id) => game.buy_object(object_id).map(|_| None),
        Operation::Sell(object_id) => game.sell_object(object_id).map(|_| None),
        Operation::Start(event_id) => game
            .apply_action(event_id)
            .map(|value| Some(serde_json::to_value(value).expect("event result must serialize"))),
        Operation::Enter {
            event_id,
            object_id,
        } => game.enter_event(event_id, object_id).map(|_| None),
        Operation::Rent {
            event_id,
            object_id,
        } => game.rent_event(event_id, object_id).map(|_| None),
        Operation::JoinQuest(quest_id) => game.join_quest(quest_id).map(|_| None),
        Operation::ResolveEncounter(action_id) => {
            game.resolve_encounter(action_id.as_deref()).map(Some)
        }
        Operation::Submit {
            entry_id,
            result,
            player_position,
        } => game
            .submit_event(entry_id, result, *player_position)
            .map(|value| Some(serde_json::to_value(value).expect("event result must serialize"))),
        Operation::Sequence(_) => Err("nested operation sequences are not supported".to_string()),
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AdvanceDay => write!(formatter, "advance_day"),
            Self::Buy(id) => write!(formatter, "buy:{id}"),
            Self::Sell(id) => write!(formatter, "sell:{id}"),
            Self::Start(id) => write!(formatter, "start:{id}"),
            Self::Enter {
                event_id,
                object_id,
            } => write!(formatter, "enter:{event_id}:{object_id}"),
            Self::Rent {
                event_id,
                object_id,
            } => write!(formatter, "rent:{event_id}:{object_id}"),
            Self::JoinQuest(id) => write!(formatter, "join_quest:{id}"),
            Self::ResolveEncounter(action_id) => match action_id {
                Some(action_id) => write!(formatter, "encounter:{action_id}"),
                None => write!(formatter, "encounter"),
            },
            Self::Submit {
                entry_id,
                result,
                player_position,
            } => match player_position {
                Some(position) => write!(formatter, "submit:{entry_id}:{result}:{position}"),
                None => write!(formatter, "submit:{entry_id}:{result}"),
            },
            Self::Sequence(operations) => {
                let expression = operations
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(";");
                write!(formatter, "sequence:{expression}")
            }
        }
    }
}

fn print_error(error: String) -> ! {
    println!(
        "{}",
        serde_json::to_string_pretty(&ErrorResponse {
            ok: false,
            error,
            usage: usage(),
        })
        .expect("error response must serialize")
    );
    std::process::exit(2);
}

fn main() {
    let args = match parse_args(env::args().skip(1)) {
        Ok(args) => args,
        Err(error) => print_error(error),
    };

    match args.mode {
        Mode::Capabilities => {
            println!(
                "{}",
                serde_json::to_string_pretty(&capability_metadata())
                    .expect("capability metadata must serialize")
            );
        }
        Mode::Validate => {
            let report = validation(&args, validate_dataset_directory(&args.dataset));
            let ok = report.ok;
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("validation response must serialize")
            );
            if !ok {
                std::process::exit(1);
            }
        }
        Mode::Explain => {
            let expression = args
                .expression
                .as_deref()
                .expect("explain expression is required");
            match explain(&args, expression) {
                Ok(response) => println!(
                    "{}",
                    serde_json::to_string_pretty(&response)
                        .expect("explanation response must serialize")
                ),
                Err(error) => print_error(error),
            }
        }
        Mode::Modifier => {
            let target = args
                .modifier_target
                .as_deref()
                .expect("modifier target is required");
            let base = args.modifier_base.expect("modifier base is required");
            match modifier(&args, target, base) {
                Ok(response) => println!(
                    "{}",
                    serde_json::to_string_pretty(&response)
                        .expect("modifier response must serialize")
                ),
                Err(error) => print_error(error),
            }
        }
        Mode::Condition => {
            let group = args
                .condition_group
                .as_deref()
                .expect("condition group is required");
            match condition(&args, group) {
                Ok(response) => println!(
                    "{}",
                    serde_json::to_string_pretty(&response)
                        .expect("condition response must serialize")
                ),
                Err(error) => print_error(error),
            }
        }
        Mode::Execute => {
            let operation = args.operation.as_ref().expect("operation is required");
            let response = execute(&args, operation);
            let ok = response.ok;
            println!(
                "{}",
                serde_json::to_string_pretty(&response).expect("execute response must serialize")
            );
            if !ok {
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_seeded_explain_arguments() {
        assert_eq!(
            parse_args([
                "--dataset".into(),
                "fixture".into(),
                "--seed".into(),
                "42".into(),
                "--explain".into(),
                "calendar_day==1".into(),
            ]),
            Ok(Arguments {
                dataset: "fixture".into(),
                seed: 42,
                mode: Mode::Explain,
                expression: Some("calendar_day==1".into()),
                condition_group: None,
                modifier_target: None,
                modifier_base: None,
                operation: None,
                sample_state: None,
            })
        );
    }

    #[test]
    fn parses_optional_sample_state_path() {
        let parsed = parse_args([
            "--sample-state".into(),
            "sample.json".into(),
            "--explain".into(),
            "calendar_day==7".into(),
        ])
        .expect("sample state should be accepted");
        assert_eq!(parsed.sample_state.as_deref(), Some("sample.json"));
        assert_eq!(parsed.expression.as_deref(), Some("calendar_day==7"));
    }

    #[test]
    fn parses_modifier_arguments() {
        assert_eq!(
            parse_args(["--modifier".into(), "event_reward".into(), "100".into(),]),
            Ok(Arguments {
                dataset: "dataset".into(),
                seed: 1,
                mode: Mode::Modifier,
                expression: None,
                condition_group: None,
                modifier_target: Some("event_reward".into()),
                modifier_base: Some(100.0),
                operation: None,
                sample_state: None,
            })
        );
    }

    #[test]
    fn parses_condition_arguments() {
        assert_eq!(
            parse_args([
                "--seed".into(),
                "9".into(),
                "--condition".into(),
                "entry_ready".into(),
            ]),
            Ok(Arguments {
                dataset: "dataset".into(),
                seed: 9,
                mode: Mode::Condition,
                expression: None,
                condition_group: Some("entry_ready".into()),
                modifier_target: None,
                modifier_base: None,
                operation: None,
                sample_state: None,
            })
        );
    }

    #[test]
    fn parses_supported_basic_facts() {
        assert!(matches!(
            parse_fact("characteristic:budget==0"),
            Ok(StateFact::Characteristic { .. })
        ));
        assert!(matches!(
            parse_fact("object_count:vehicle==2"),
            Ok(StateFact::ObjectCount { count: 2, .. })
        ));
        assert!(matches!(
            parse_fact("age_days==365"),
            Ok(StateFact::AgeDays(365))
        ));
        assert!(matches!(
            parse_fact("object_type:vehicle==2"),
            Ok(StateFact::ObjectCount {
                query: ObjectQuery {
                    object_type: Some(_),
                    ..
                },
                count: 2
            })
        ));
        assert!(matches!(
            parse_fact("quest_joined:starter==1"),
            Ok(StateFact::QuestJoined { joined: true, .. })
        ));
        assert!(matches!(
            parse_fact("event_completed:demo:failure==1"),
            Ok(StateFact::EventCompleted { success: false, .. })
        ));
    }

    #[test]
    fn rejects_ambiguous_or_unsupported_requests() {
        assert!(parse_args(["--validate".into(), "--capabilities".into()]).is_err());
        assert!(parse_fact("characteristic:budget>=0").is_err());
    }

    #[test]
    fn parses_explicit_seeded_operations() {
        assert_eq!(
            parse_operation("enter:race:event-car"),
            Ok(Operation::Enter {
                event_id: "race".into(),
                object_id: "event-car".into()
            })
        );
        assert_eq!(
            parse_args([
                "--dataset".into(),
                "tests/fixtures/cooking".into(),
                "--seed".into(),
                "42".into(),
                "--execute-sequence".into(),
                "join_quest:recipe_quest;buy:skillet;enter:bake_pie:skillet_1;submit:event_entry_bake_pie_1:success".into(),
            ])
            .map(|args| args.operation),
            Ok(Some(Operation::Sequence(vec![
                Operation::JoinQuest("recipe_quest".into()),
                Operation::Buy("skillet".into()),
                Operation::Enter {
                    event_id: "bake_pie".into(),
                    object_id: "skillet_1".into(),
                },
                Operation::Submit {
                    entry_id: "event_entry_bake_pie_1".into(),
                    result: "success".into(),
                    player_position: None,
                },
            ])))
        );
        assert_eq!(
            parse_operation("submit:entry:success:3"),
            Ok(Operation::Submit {
                entry_id: "entry".into(),
                result: "success".into(),
                player_position: Some(3)
            })
        );
        assert_eq!(
            parse_operation("rent:event:object"),
            Ok(Operation::Rent {
                event_id: "event".into(),
                object_id: "object".into()
            })
        );
        assert_eq!(
            parse_operation("encounter:negotiate"),
            Ok(Operation::ResolveEncounter(Some("negotiate".into())))
        );
        assert_eq!(
            parse_operation("encounter"),
            Ok(Operation::ResolveEncounter(None))
        );
        assert_eq!(
            parse_args([
                "--seed".into(),
                "42".into(),
                "--execute".into(),
                "advance_day".into()
            ]),
            Ok(Arguments {
                dataset: "dataset".into(),
                seed: 42,
                mode: Mode::Execute,
                expression: None,
                condition_group: None,
                modifier_target: None,
                modifier_base: None,
                operation: Some(Operation::AdvanceDay),
                sample_state: None,
            })
        );
    }

    #[test]
    fn rejects_ambiguous_operation_ids() {
        assert!(parse_operation("buy:").is_err());
        assert!(parse_operation("buy:object:extra").is_err());
        assert!(parse_operation("enter:event:").is_err());
        assert!(parse_operation("submit:entry:success:nope").is_err());
        assert!(parse_operation("rent:event:").is_err());
        assert!(parse_operation("encounter:").is_err());
    }

    #[test]
    fn seeded_execute_is_reproducible_and_reports_diffs() {
        let args = Arguments {
            dataset: "dataset".into(),
            seed: 42,
            mode: Mode::Execute,
            expression: None,
            condition_group: None,
            modifier_target: None,
            modifier_base: None,
            operation: Some(Operation::AdvanceDay),
            sample_state: None,
        };

        let left = execute(&args, args.operation.as_ref().expect("operation"));
        let right = execute(&args, args.operation.as_ref().expect("operation"));

        assert!(left.ok);
        assert_eq!(
            serde_json::to_value(&left).expect("response should serialize"),
            serde_json::to_value(&right).expect("response should serialize")
        );
        assert!(left
            .state_diff
            .iter()
            .any(|diff| diff.path == "current_day"));
        assert!(left
            .diagnostics
            .iter()
            .any(|message| message.contains("cloned seeded state")));
    }

    #[test]
    fn failed_execute_returns_diagnostics_without_state_diff() {
        let args = Arguments {
            dataset: "dataset".into(),
            seed: 1,
            mode: Mode::Execute,
            expression: None,
            condition_group: None,
            modifier_target: None,
            modifier_base: None,
            operation: Some(Operation::Buy("missing-object".into())),
            sample_state: None,
        };

        let response = execute(&args, args.operation.as_ref().expect("operation"));

        assert!(!response.ok);
        assert!(response.result.is_none());
        assert!(response.state_diff.is_empty());
        assert!(response
            .diagnostics
            .iter()
            .any(|message| message.contains("not found") || message.contains("acquire")));
    }

    #[test]
    fn generic_sequence_matches_headless_operation_replay_and_reports_steps() {
        let dataset = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/fixtures/cooking");
        let operations = Operation::Sequence(vec![
            Operation::JoinQuest("recipe_quest".into()),
            Operation::Buy("skillet".into()),
            Operation::Enter {
                event_id: "bake_pie".into(),
                object_id: "skillet_1".into(),
            },
            Operation::Submit {
                entry_id: "event_entry_bake_pie_1".into(),
                result: "success".into(),
                player_position: None,
            },
        ]);
        let args = Arguments {
            dataset: dataset.into(),
            seed: 42,
            mode: Mode::Execute,
            expression: None,
            condition_group: None,
            modifier_target: None,
            modifier_base: None,
            operation: Some(operations),
            sample_state: None,
        };

        let response = execute(&args, args.operation.as_ref().expect("operation"));

        assert!(response.ok, "{:?}", response.diagnostics);
        assert!(response
            .diagnostics
            .iter()
            .any(|message| message.contains("step 4/4 applied")));
        assert!(response
            .state_diff
            .iter()
            .any(|diff| diff.path == "event_history"));
        assert!(response
            .state_diff
            .iter()
            .any(|diff| diff.path == "player.inventory"));
        assert!(response
            .state_diff
            .iter()
            .any(|diff| diff.path == "quest_memberships"));

        let mut headless = HeadlessGame::new(dataset, 42);
        headless
            .join_quest("recipe_quest")
            .expect("quest should join");
        headless.buy_object("skillet").expect("object should buy");
        headless
            .enter_event("bake_pie", "skillet_1")
            .expect("event should enter");
        headless
            .submit_event("event_entry_bake_pie_1", "success", None)
            .expect("event should submit");
        let expected_after =
            serde_json::to_value(headless.state()).expect("state should serialize");
        let expected_before = serde_json::to_value(HeadlessGame::new(dataset, 42).state())
            .expect("state should serialize");
        let mut expected_diff = Vec::new();
        diff_values("", &expected_before, &expected_after, &mut expected_diff);
        assert_eq!(
            response.state_diff, expected_diff,
            "preview sequence must expose the same state transition as headless replay"
        );
        assert_eq!(
            headless.state().event_history.len(),
            1,
            "sequence should use the same shared submit operation as headless runtime"
        );
    }

    #[test]
    fn stages_characteristics_history_and_quests_for_preview_evaluation() {
        let mut game = HeadlessGame::new("dataset", 7);
        let sample: SampleState = serde_json::from_value(serde_json::json!({
            "characteristics": {"budget": 250.0},
            "history": [{"event_id": "demo", "result": "failure"}],
            "quests": ["starter"],
            "current_day": 12,
            "age_days": 500
        }))
        .expect("sample state should deserialize");

        stage_sample_state(&mut game, &sample).expect("sample state should stage");

        assert_eq!(game.state().current_day, 12);
        assert_eq!(game.state().player.age_days, 500);
        assert_eq!(
            game.state().player.characteristics.get("budget"),
            Some(&250.0)
        );
        assert_eq!(game.state().event_history[0].event_id, "demo");
        assert_eq!(game.state().event_history[0].result, "failure");
        assert_eq!(game.state().quest_memberships[0].quest_id, "starter");
    }
}
