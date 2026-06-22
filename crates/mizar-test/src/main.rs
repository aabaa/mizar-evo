use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use mizar_test::{
    DiscoveryConfig, TestProfile, ValidationMode, ValidationSeverity, build_test_plan,
    run_declaration_symbol_corpus, run_parse_only_corpus, run_type_elaboration_corpus,
};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let Some(command) = args.first() else {
        return Err(usage());
    };
    if !matches!(
        command.as_str(),
        "plan" | "parse-only" | "declaration-symbol" | "type-elaboration"
    ) {
        return Err(usage());
    }

    let mut workspace_root = PathBuf::from(".");
    let mut tests_root = PathBuf::from("tests");
    let mut manifest_path = PathBuf::from("tests/coverage/spec_trace.toml");
    let mut idx = 1;
    while idx < args.len() {
        match args[idx].as_str() {
            "--workspace-root" => {
                idx += 1;
                workspace_root = next_value(&args, idx, "--workspace-root")?.into();
            }
            "--tests-root" => {
                idx += 1;
                tests_root = next_value(&args, idx, "--tests-root")?.into();
            }
            "--manifest" => {
                idx += 1;
                manifest_path = next_value(&args, idx, "--manifest")?.into();
            }
            "--validation-mode" => {
                idx += 1;
                let value = next_value(&args, idx, "--validation-mode")?;
                if value != "metadata" {
                    return Err(
                        "only `--validation-mode metadata` is implemented in the minimal crate"
                            .to_owned(),
                    );
                }
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
        idx += 1;
    }

    let config = DiscoveryConfig {
        workspace_root,
        tests_root,
        manifest_path,
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    match command.as_str() {
        "plan" => run_plan(&config),
        "parse-only" => run_parse_only(&config),
        "declaration-symbol" => run_declaration_symbol(&config),
        "type-elaboration" => run_type_elaboration(&config),
        _ => unreachable!("command was validated above"),
    }
}

fn run_plan(config: &DiscoveryConfig) -> Result<ExitCode, String> {
    let plan = build_test_plan(config).map_err(|error| error.to_string())?;

    for diagnostic in &plan.diagnostics {
        eprintln!("{diagnostic}");
    }

    println!("test cases: {}", plan.cases.len());
    println!("requirements: {}", plan.manifest.requirements.len());
    println!("errors: {}", plan.error_count());
    println!("warnings: {}", plan.warning_count());

    if plan
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == ValidationSeverity::Error)
    {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn run_parse_only(config: &DiscoveryConfig) -> Result<ExitCode, String> {
    let report = run_parse_only_corpus(config).map_err(|error| error.to_string())?;

    for diagnostic in &report.diagnostics {
        eprintln!("{diagnostic}");
    }

    println!("parse-only cases: {}", report.results.len());
    println!("passed: {}", report.passed_count());
    println!("failed: {}", report.failed_count());
    println!("errors: {}", report.error_count());
    println!("warnings: {}", report.warning_count());

    if report.error_count() > 0 {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn run_declaration_symbol(config: &DiscoveryConfig) -> Result<ExitCode, String> {
    let report = run_declaration_symbol_corpus(config).map_err(|error| error.to_string())?;

    for diagnostic in &report.diagnostics {
        eprintln!("{diagnostic}");
    }

    println!("declaration-symbol cases: {}", report.results.len());
    println!("passed: {}", report.passed_count());
    println!("failed: {}", report.failed_count());
    println!("errors: {}", report.error_count());
    println!("warnings: {}", report.warning_count());

    if report.error_count() > 0 {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn run_type_elaboration(config: &DiscoveryConfig) -> Result<ExitCode, String> {
    let report = run_type_elaboration_corpus(config).map_err(|error| error.to_string())?;

    for diagnostic in &report.diagnostics {
        eprintln!("{diagnostic}");
    }

    println!("type-elaboration cases: {}", report.results.len());
    println!("passed: {}", report.passed_count());
    println!("failed: {}", report.failed_count());
    println!("errors: {}", report.error_count());
    println!("warnings: {}", report.warning_count());

    if report.error_count() > 0 {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn usage() -> String {
    "usage: mizar-test <plan|parse-only|declaration-symbol|type-elaboration> [--tests-root tests] [--manifest tests/coverage/spec_trace.toml] [--workspace-root .]".to_owned()
}

fn next_value(args: &[String], idx: usize, name: &str) -> Result<String, String> {
    args.get(idx)
        .cloned()
        .ok_or_else(|| format!("missing value for `{name}`"))
}
