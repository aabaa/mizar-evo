use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use mizar_test::{
    DiscoveryConfig, TestProfile, ValidationMode, ValidationSeverity, build_test_plan,
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
    if args.first().is_none_or(|arg| arg != "plan") {
        return Err("usage: mizar-test plan [--tests-root tests] [--manifest tests/coverage/spec_trace.toml] [--workspace-root .]".to_owned());
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
    let plan = build_test_plan(&config).map_err(|error| error.to_string())?;

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

fn next_value(args: &[String], idx: usize, name: &str) -> Result<String, String> {
    args.get(idx)
        .cloned()
        .ok_or_else(|| format!("missing value for `{name}`"))
}
