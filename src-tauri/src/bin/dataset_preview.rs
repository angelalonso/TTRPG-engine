use std::env;

use serde::Serialize;
use serde_json::Value;
use ttrpg_engine_lib::engine::facts::{
    characteristic, evaluate_fact, object_count, ObjectQuery, StateFact,
};
use ttrpg_engine_lib::engine::loader::{validate_dataset_directory, DatasetValidationReport};
use ttrpg_engine_lib::engine::schema::capability_metadata;
use ttrpg_engine_lib::new_game_seeded;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Validate,
    Capabilities,
    Explain,
}

#[derive(Debug, PartialEq, Eq)]
struct Arguments {
    dataset: String,
    seed: u64,
    mode: Mode,
    expression: Option<String>,
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

fn usage() -> &'static str {
    "dataset_preview [--dataset PATH] [--seed N] (--validate | --capabilities | --explain FACT)\n\
FACT forms: characteristic:<id>==<number>, object_count:<id>==<number>, \
calendar_day==<number>, age_days==<number>"
}

fn parse_args<I>(args: I) -> Result<Arguments, String>
where
    I: IntoIterator<Item = String>,
{
    let mut dataset = "dataset".to_string();
    let mut seed = 1;
    let mut mode = None;
    let mut expression = None;
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
            "--help" | "-h" => return Err(usage().to_string()),
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let mode = mode
        .ok_or_else(|| "one of --validate, --capabilities, or --explain is required".to_string())?;
    if mode != Mode::Explain && expression.is_some() {
        return Err("an explanation expression is only valid with --explain".to_string());
    }
    Ok(Arguments {
        dataset,
        seed,
        mode,
        expression,
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
    if kind == "calendar_day" {
        return Ok(StateFact::CalendarDay(non_negative_count(value)? as u32));
    }
    if kind == "age_days" {
        return Ok(StateFact::AgeDays(non_negative_count(value)? as u32));
    }
    Err(format!("unsupported fact: {kind}"))
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
        StateFact::CalendarDay(day) => {
            format!("calendar day is {}; expected {day}", game.current_day)
        }
        StateFact::AgeDays(days) => {
            format!("age is {} days; expected {days}", game.player.age_days)
        }
        _ => "fact evaluated".to_string(),
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
    }

    #[test]
    fn rejects_ambiguous_or_unsupported_requests() {
        assert!(parse_args(["--validate".into(), "--capabilities".into()]).is_err());
        assert!(parse_fact("characteristic:budget>=0").is_err());
    }
}
