use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

#[test]
fn workspace_lists_driver_member() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);

    assert!(
        manifest.contains("\"crates/mizar-driver\""),
        "{} must list crates/mizar-driver as a workspace member",
        manifest_path.display()
    );
}

#[test]
fn driver_manifest_declares_expected_package_and_library() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let package = section(&manifest, "package");
    let lib = section(&manifest, "lib");

    assert_section_assignment(&package, &manifest_path, "package", "name", "mizar-driver");
    assert_section_assignment(&package, &manifest_path, "package", "version", "0.1.0");
    assert_section_assignment(
        &package,
        &manifest_path,
        "package",
        "edition.workspace",
        "true",
    );
    assert_section_assignment(
        &package,
        &manifest_path,
        "package",
        "license.workspace",
        "true",
    );
    assert_section_assignment(
        &package,
        &manifest_path,
        "package",
        "repository.workspace",
        "true",
    );
    assert_section_assignment(&lib, &manifest_path, "lib", "name", "mizar_driver");
    assert_section_assignment(&lib, &manifest_path, "lib", "path", "src/lib.rs");
}

#[test]
fn driver_manifest_opts_into_workspace_lints() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let lints = section(&manifest, "lints");

    assert!(
        lints
            .iter()
            .any(|line| assignment_is(line, "workspace", "true")),
        "{} must keep [lints] workspace = true so cargo build/test and clippy \
         use the shared lint policy",
        manifest_path.display()
    );
}

#[test]
fn driver_dependency_boundary_is_exact_for_scaffold_task() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_section_names(&manifest);
    let expected_sections = vec!["dependencies".to_owned()];
    let dependencies = section(&manifest, "dependencies");
    let actual = dependency_names(&dependencies);
    let expected = BTreeSet::from([
        "mizar-build".to_owned(),
        "mizar-diagnostics".to_owned(),
        "mizar-ir".to_owned(),
        "mizar-session".to_owned(),
        "salsa".to_owned(),
    ]);

    assert_eq!(
        dependency_sections,
        expected_sections,
        "{} must keep D-001 dependencies only in [dependencies]; \
         dev/build/target-specific dependency tables are later-task scope",
        manifest_path.display()
    );
    assert_eq!(
        actual,
        expected,
        "{} must keep D-001 dependencies limited to the real owner seams \
         named by the crate plan",
        manifest_path.display()
    );
    assert_dependency_path(
        &dependencies,
        &manifest_path,
        "mizar-build",
        "../mizar-build",
    );
    assert_dependency_path(
        &dependencies,
        &manifest_path,
        "mizar-diagnostics",
        "../mizar-diagnostics",
    );
    assert_dependency_path(&dependencies, &manifest_path, "mizar-ir", "../mizar-ir");
    assert_dependency_path(
        &dependencies,
        &manifest_path,
        "mizar-session",
        "../mizar-session",
    );
    assert!(
        dependencies
            .iter()
            .any(|line| line.starts_with("salsa =") && line.contains("\"0.27.2\"")),
        "{} must depend on salsa only for the driver-owned registry query boundary",
        manifest_path.display()
    );
}

#[test]
fn dependency_section_detector_covers_cargo_table_forms() {
    let manifest = r#"
[dependencies]
mizar-session = { path = "../mizar-session" }

[dependencies.fake]
path = "../fake"

[dev-dependencies.fake]
path = "../fake"

[target.'cfg(unix)'.dependencies.fake] # inline comment
path = "../fake"
"#;

    assert_eq!(
        dependency_section_names(manifest),
        vec![
            "dependencies".to_owned(),
            "dependencies.fake".to_owned(),
            "dev-dependencies.fake".to_owned(),
            "target.'cfg(unix)'.dependencies.fake".to_owned(),
        ]
    );
}

#[test]
fn driver_source_surface_matches_completed_tasks() {
    let root = crate_root();
    let source_files = rust_source_files(&root);
    let expected_files = vec![
        root.join("src/cli/output.rs"),
        root.join("src/cli.rs"),
        root.join("src/driver/event_log.rs"),
        root.join("src/driver/scheduler.rs"),
        root.join("src/driver/tests.rs"),
        root.join("src/driver/watch.rs"),
        root.join("src/driver.rs"),
        root.join("src/events.rs"),
        root.join("src/lib.rs"),
        root.join("src/registry/catalog.rs"),
        root.join("src/registry.rs"),
        root.join("src/request.rs"),
    ];

    assert_eq!(
        source_files, expected_files,
        "driver source files must match the completed task surface"
    );
}

#[test]
fn driver_lib_exposes_only_completed_modules() {
    let root = crate_root();
    let lib_rs = root.join("src/lib.rs");
    let source = read_to_string(&lib_rs);
    let mut violations = Vec::new();

    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if lib_has_unexpected_code_line(trimmed) {
            violations.push(format!("src/lib.rs:{}", index + 1));
        }
    }

    assert!(
        violations.is_empty(),
        "driver lib must expose only modules whose paired specs have landed:\n{}",
        violations.join("\n")
    );
}

#[test]
fn driver_private_helper_modules_stay_private() {
    let root = crate_root();
    let helper_modules = [
        ("src/cli.rs", "output"),
        ("src/driver.rs", "event_log"),
        ("src/driver.rs", "scheduler"),
        ("src/driver.rs", "watch"),
        ("src/registry.rs", "catalog"),
    ];

    for (module_file, module_name) in helper_modules {
        let path = root.join(module_file);
        let source = read_to_string(&path);
        assert!(
            source
                .lines()
                .any(|line| line.trim() == format!("mod {module_name};")),
            "{module_file} must declare private helper module {module_name}"
        );
        assert!(
            !source.lines().any(|line| {
                let trimmed = line.trim();
                trimmed == format!("pub mod {module_name};")
                    || trimmed == format!("pub(crate) mod {module_name};")
            }),
            "{module_file} must not expose helper module {module_name}"
        );
    }
}

#[test]
fn driver_does_not_expose_later_task_modules_yet() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in rust_source_files(&root) {
        let source = read_to_string(&path);
        for forbidden in ["pub mod watch"] {
            if source.contains(forbidden) {
                let relative = path.strip_prefix(&root).unwrap_or(&path);
                violations.push(format!("{} contains {forbidden}", relative.display()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "later task modules require their own spec/implementation tasks:\n{}",
        violations.join("\n")
    );
}

#[test]
fn lib_code_line_detector_blocks_unexpected_item_forms() {
    for line in [
        "pub(crate) mod request;",
        "impl Driver {}",
        "macro_rules! driver {}",
        "extern crate fake;",
        "pub fn run() {}",
    ] {
        assert!(lib_has_unexpected_code_line(line), "{line}");
    }

    for line in [
        "",
        "//! docs",
        "// comment",
        "#![forbid(unsafe_code)]",
        "#[test]",
        "pub mod cli;",
        "pub mod driver;",
        "pub mod events;",
        "pub mod request;",
        "pub mod registry;",
    ] {
        assert!(!lib_has_unexpected_code_line(line), "{line}");
    }
}

#[test]
fn phase_owner_crates_do_not_depend_on_driver_or_salsa() {
    let workspace = workspace_root();
    let owner_crates = [
        "mizar-lexer",
        "mizar-syntax",
        "mizar-parser",
        "mizar-frontend",
        "mizar-resolve",
        "mizar-checker",
        "mizar-core",
        "mizar-vc",
        "mizar-atp",
        "mizar-kernel",
        "mizar-proof",
        "mizar-artifact",
        "mizar-doc",
    ];
    let expected_missing = BTreeSet::from(["mizar-doc"]);
    let mut missing = BTreeSet::new();
    let mut violations = Vec::new();

    for crate_name in owner_crates {
        let manifest_path = workspace.join("crates").join(crate_name).join("Cargo.toml");
        if !manifest_path.exists() {
            missing.insert(crate_name);
            continue;
        }
        let manifest = read_to_string(&manifest_path);
        let dependency_names = dependency_section_names(&manifest)
            .into_iter()
            .flat_map(|section_name| dependency_names(&section(&manifest, &section_name)))
            .collect::<BTreeSet<_>>();
        for forbidden in ["mizar-driver", "salsa"] {
            if dependency_names.contains(forbidden) {
                violations.push(format!(
                    "{} depends on {forbidden}",
                    manifest_path.display()
                ));
            }
        }
    }

    assert_eq!(
        missing, expected_missing,
        "phase owner dependency scan must cover every existing owner crate and only skip documented absent owners"
    );
    assert!(
        violations.is_empty(),
        "phase owner crates must not gain driver or salsa dependencies through the registry:\n{}",
        violations.join("\n")
    );
}

#[test]
fn workspace_lint_baseline_denies_rustc_warnings_and_clippy_all() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let rust_lints = section(&manifest, "workspace.lints.rust");
    let clippy_lints = section(&manifest, "workspace.lints.clippy");

    assert!(
        rust_lints
            .iter()
            .any(|line| assignment_is(line, "warnings", "deny")),
        "{} must deny rustc warnings in the shared lint baseline",
        manifest_path.display()
    );
    assert!(
        clippy_lints
            .iter()
            .any(|line| assignment_is(line, "all", "deny")),
        "{} must deny clippy::all in the shared lint baseline",
        manifest_path.display()
    );
}

#[test]
fn public_driver_enums_are_forward_compatible() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in rust_target_files(&root) {
        collect_public_enums_without_non_exhaustive(&root, &path, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "public driver enums must stay forward-compatible; document any \
         exhaustive exception in the owning module spec before changing this \
         guard:\n{}",
        violations.join("\n")
    );
}

#[test]
fn driver_allow_exceptions_are_documented_inline() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in rust_target_files(&root) {
        collect_undocumented_allows(&root, &path, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "intentional lint allow exceptions need an adjacent reason:\n{}",
        violations.join("\n")
    );
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> PathBuf {
    crate_root()
        .parent()
        .and_then(Path::parent)
        .expect("crate lives under crates/<name>")
        .to_path_buf()
}

fn read_to_string(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn section<'a>(manifest: &'a str, section: &str) -> Vec<&'a str> {
    let header = format!("[{section}]");
    let mut lines = Vec::new();
    let mut active = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            active = trimmed == header;
            continue;
        }
        if active && !trimmed.is_empty() && !trimmed.starts_with('#') {
            lines.push(trimmed);
        }
    }
    lines
}

fn assignment_is(line: &str, key: &str, value: &str) -> bool {
    let Some((lhs, rhs)) = line.split_once('=') else {
        return false;
    };
    lhs.trim() == key && rhs.trim().trim_matches('"') == value
}

fn assert_section_assignment(
    section: &[&str],
    manifest_path: &Path,
    section_name: &str,
    key: &str,
    value: &str,
) {
    assert!(
        section.iter().any(|line| assignment_is(line, key, value)),
        "{} must set [{}].{} = {}",
        manifest_path.display(),
        section_name,
        key,
        value
    );
}

fn dependency_names(lines: &[&str]) -> BTreeSet<String> {
    lines
        .iter()
        .filter_map(|line| {
            line.split_once('=')
                .map(|(lhs, _rhs)| lhs.trim().to_owned())
        })
        .collect()
}

fn dependency_section_names(manifest: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in manifest.lines() {
        let trimmed = strip_toml_comment(line).trim();
        let Some(name) = trimmed
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        else {
            continue;
        };
        if is_dependency_section_name(name) {
            names.push(name.to_owned());
        }
    }
    names
}

fn strip_toml_comment(line: &str) -> &str {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut previous = '\0';

    for (index, ch) in line.char_indices() {
        match ch {
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '"' if !in_single_quote && previous != '\\' => in_double_quote = !in_double_quote,
            '#' if !in_single_quote && !in_double_quote => return &line[..index],
            _ => {}
        }
        previous = ch;
    }

    line
}

fn is_dependency_section_name(name: &str) -> bool {
    name.split('.').any(|part| {
        matches!(
            part,
            "dependencies" | "dev-dependencies" | "build-dependencies"
        )
    })
}

fn assert_dependency_path(lines: &[&str], manifest_path: &Path, name: &str, path: &str) {
    let expected_name = format!("{name} =");
    let expected_path = format!("path = \"{path}\"");

    assert!(
        lines
            .iter()
            .any(|line| line.starts_with(&expected_name) && line.contains(&expected_path)),
        "{} must depend on {} through path {}",
        manifest_path.display(),
        name,
        path
    );
}

fn rust_target_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_files(&root.join("src"), &mut files);
    collect_rust_files(&root.join("tests"), &mut files);
    files.sort();
    files
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_files(&root.join("src"), &mut files);
    files.sort();
    files
}

fn collect_rust_files(path: &Path, files: &mut Vec<PathBuf>) {
    if !path.exists() {
        return;
    }

    if path.is_file() {
        if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path.to_path_buf());
        }
        return;
    }

    for entry in fs::read_dir(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())) {
        let entry = entry.unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        collect_rust_files(&entry.path(), files);
    }
}

fn collect_public_enums_without_non_exhaustive(
    root: &Path,
    path: &Path,
    violations: &mut Vec<String>,
) {
    let source = read_to_string(path);
    let mut previous_significant_line = "";

    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(name) = public_enum_name(trimmed)
            && !previous_significant_line.contains("#[non_exhaustive]")
        {
            let relative = path.strip_prefix(root).unwrap_or(path);
            violations.push(format!(
                "{}:{}: pub enum {name}",
                relative.display(),
                index + 1
            ));
        }
        if !trimmed.is_empty() {
            previous_significant_line = trimmed;
        }
    }
}

fn collect_undocumented_allows(root: &Path, path: &Path, violations: &mut Vec<String>) {
    let source = read_to_string(path);
    let mut previous_significant_line = "";

    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if is_allow_attribute(trimmed) && !previous_significant_line.contains("allow:") {
            let relative = path.strip_prefix(root).unwrap_or(path);
            violations.push(format!("{}:{}", relative.display(), index + 1));
        }
        if !trimmed.is_empty() {
            previous_significant_line = trimmed;
        }
    }
}

fn public_enum_name(line: &str) -> Option<&str> {
    let name = line.strip_prefix("pub enum ")?;
    name.split(|ch: char| ch == '{' || ch == '<' || ch.is_whitespace())
        .find(|part| !part.is_empty())
}

fn lib_has_unexpected_code_line(line: &str) -> bool {
    let ignored_prefixes = ["//", "#![", "#[", "//!"];
    line != "pub mod request;"
        && line != "pub mod cli;"
        && line != "pub mod driver;"
        && line != "pub mod events;"
        && line != "pub mod registry;"
        && !line.is_empty()
        && !ignored_prefixes
            .iter()
            .any(|prefix| line.starts_with(prefix))
}

fn is_allow_attribute(line: &str) -> bool {
    let compact = line
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    let is_attribute = compact.starts_with("#[") || compact.starts_with("#![");
    is_attribute
        && (compact.starts_with("#[allow(")
            || compact.starts_with("#![allow(")
            || compact.contains("cfg_attr(") && compact.contains("allow("))
}
