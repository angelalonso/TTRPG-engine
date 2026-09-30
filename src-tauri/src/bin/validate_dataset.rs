use std::env;
use ttrpg_engine_lib::engine::loader::validate_dataset_directory;

fn main() {
    let dataset = env::args().nth(1).unwrap_or_else(|| "dataset".into());
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
