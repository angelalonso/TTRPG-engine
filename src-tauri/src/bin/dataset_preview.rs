use std::env;

use serde::Serialize;
use serde_json::Value;
use ttrpg_engine_lib::engine::conditions::{ConditionSet, ExplanationNode};
use ttrpg_engine_lib::engine::facts::{
    characteristic, evaluate_fact, event_completed, object_count, quest_joined, ObjectQuery,
    StateFact,
};
use ttrpg_engine_lib::engine::loader::{validate_dataset_directory, DatasetValidationReport};
use ttrpg_engine_lib::engine::modifiers::NumericModifierContribution;
use ttrpg_engine_lib::engine::schema::capability_metadata;
use ttrpg_engine_lib::{new_game_seeded, numeric_modifier_breakdown_for_sim};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Validate,
    Capabilities,
    Explain,
    Condition,
    Modifier,
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

fn usage() -> &'static str {
    "dataset_preview [--dataset PATH] [--seed N] (--validate | --capabilities | --explain FACT | --condition GROUP | --modifier TARGET BASE)\n\
FACT forms: characteristic:<id>==<number>, object_count:<id>==<number>, \
object_type:<type>==<number>, active_event_count[:<id>]==<number>, \
event_completed:<id>[:success|failure]==0|1, quest_joined:<id>==0|1, \
calendar_day==<number>, age_days==<number>\n\
CONDITION form: --condition CONDITION_GROUP_ID\n\
MODIFIER form: --modifier TARGET BASE"
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
            "--help" | "-h" => return Err(usage().to_string()),
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let mode = mode.ok_or_else(|| {
        "one of --validate, --capabilities, --explain, --condition, or --modifier is required"
            .to_string()
    })?;
    if mode != Mode::Explain && expression.is_some() {
        return Err("an explanation expression is only valid with --explain".to_string());
    }
    Ok(Arguments {
        dataset,
        seed,
        mode,
        expression,
        condition_group,
        modifier_target,
        modifier_base,
    })
}

fn condition(args: &Arguments, group: &str) -> Result<ConditionResponse, String> {
    let game = new_game_seeded(&args.dataset, args.seed);
    let set = ConditionSet::from_rows(&game.catalog.condition_groups, &game.catalog.conditions)
        .map_err(|errors| format!("invalid condition contract: {errors:?}"))?;
    let explanation = set
        .evaluate(&game, group)
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
        return Err("choose exactly one of --validate, --capabilities, or --explain".to_string());
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
    let game = new_game_seeded(&args.dataset, args.seed);
    let result = evaluate_fact(&game, fact.clone()).map_err(|error| format!("{error:?}"))?;
    let explanation = match &fact {
        StateFact::Characteristic { id, value } => {
            let actual = characteristic(&game, id).map_err(|error| format!("{error:?}"))?;
            format!("characteristic {id} is {actual}; expected {value}")
        }
        StateFact::ObjectCount { query, count } => {
            let actual = object_count(&game, query).map_err(|error| format!("{error:?}"))?;
            format!(
                "object count for {} is {actual}; expected {count}",
                query.definition_id.as_deref().unwrap_or("<any>")
            )
        }
        StateFact::EventCompleted {
            event_id, success, ..
        } => {
            let actual = event_completed(&game, event_id, Some(*success))
                .map_err(|error| format!("{error:?}"))?;
            format!("event {event_id} completed with expected result {success}; actual {actual}")
        }
        StateFact::QuestJoined { quest_id, joined } => {
            let actual = quest_joined(&game, quest_id).map_err(|error| format!("{error:?}"))?;
            format!("quest {quest_id} joined state is {actual}; expected {joined}")
        }
        StateFact::ActiveEventCount { event_id, count } => {
            let actual =
                ttrpg_engine_lib::engine::facts::active_event_count(&game, event_id.as_deref());
            format!(
                "active event count for {} is {actual}; expected {count}",
                event_id.as_deref().unwrap_or("<any>")
            )
        }
        StateFact::CalendarDay(day) => {
            format!("calendar day is {}; expected {day}", game.current_day)
        }
        StateFact::AgeDays(days) => {
            format!("age is {} days; expected {days}", game.player.age_days)
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
            })
        );
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
}
