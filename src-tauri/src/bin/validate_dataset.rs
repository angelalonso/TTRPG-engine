use std::env;
use ttrpg_engine_lib::engine::loader::validate_dataset_directory;
use ttrpg_engine_lib::engine::schema::capability_metadata;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--capabilities") {
        println!(
            "{}",
            serde_json::to_string_pretty(&capability_metadata())
                .expect("capability metadata must serialize")
        );
        return;
    }

    let dataset = args.first().cloned().unwrap_or_else(|| "gtr2career".into());
    let report = validate_dataset_directory(&dataset);

    for warning in &report.warnings {
        eprintln!("warning: {warning}");
    }
    for error in &report.errors {
        eprintln!("error: {error}");
    }

    println!(
        "dataset={} errors={} warnings={}",
        dataset,
        report.errors.len(),
        report.warnings.len()
    );

    if !report.is_valid() {
        std::process::exit(1);
    }
}
