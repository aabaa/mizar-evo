use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn artifact_manifest_opts_into_workspace_lints() {
    let manifest = read_to_string(&crate_root().join("Cargo.toml"));
    let lints = section(&manifest, "lints");

    assert!(
        lints
            .iter()
            .any(|line| assignment_is(line, "workspace", "true")),
        "mizar-artifact must keep [lints] workspace = true"
    );
}

#[test]
fn artifact_depends_only_on_mizar_session() {
    let manifest = read_to_string(&crate_root().join("Cargo.toml"));
    let dependencies = section(&manifest, "dependencies");

    assert_eq!(
        dependencies
            .iter()
            .map(|line| line.trim())
            .collect::<Vec<_>>(),
        vec!["mizar-session = { path = \"../mizar-session\" }"],
        "mizar-artifact is a leaf schema/store crate and may depend only on mizar-session"
    );
}

#[test]
fn workspace_lint_baseline_denies_warnings_and_clippy_all() {
    let manifest = read_to_string(&workspace_root().join("Cargo.toml"));
    let rust_lints = section(&manifest, "workspace.lints.rust");
    let clippy_lints = section(&manifest, "workspace.lints.clippy");

    assert!(
        rust_lints
            .iter()
            .any(|line| assignment_is(line, "warnings", "deny")),
        "workspace rust lints must deny warnings"
    );
    assert!(
        clippy_lints
            .iter()
            .any(|line| assignment_is(line, "all", "deny")),
        "workspace clippy lints must deny clippy::all"
    );
}

#[test]
fn public_forward_compatible_enums_are_non_exhaustive() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in rust_files(&root.join("src")) {
        let source = read_to_string(&path);
        for enum_name in public_enum_names(&source) {
            if !enum_has_attribute(&source, &enum_name, "non_exhaustive") {
                let display = path.strip_prefix(&root).unwrap_or(&path);
                violations.push(format!("{}: pub enum {enum_name}", display.display()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "public schema/error enums must be forward-compatible:\n{}",
        violations.join("\n")
    );
}

#[test]
fn public_modules_link_to_english_design_specs() {
    let specs = [
        (
            "src/store.rs",
            "../../../../doc/design/mizar-artifact/en/store.md",
        ),
        (
            "src/module_summary.rs",
            "../../../../doc/design/mizar-artifact/en/module_summary.md",
        ),
        (
            "src/registration_summary.rs",
            "../../../../doc/design/mizar-artifact/en/registration_summary.md",
        ),
        (
            "src/proof_witness.rs",
            "../../../../doc/design/mizar-artifact/en/proof_witness.md",
        ),
        (
            "src/verified_artifact.rs",
            "../../../../doc/design/mizar-artifact/en/verified_artifact.md",
        ),
        (
            "src/manifest.rs",
            "../../../../doc/design/mizar-artifact/en/manifest.md",
        ),
    ];
    let root = crate_root();
    let mut violations = Vec::new();

    for (relative_path, spec_link) in specs {
        let source = read_to_string(&root.join(relative_path));
        if !source.contains(spec_link) {
            violations.push(format!("{relative_path}: missing {spec_link}"));
        }
    }

    assert!(
        violations.is_empty(),
        "public modules must link to their design specs:\n{}",
        violations.join("\n")
    );
}

#[test]
fn allow_exceptions_are_documented_inline() {
    let root = crate_root();
    let mut violations = Vec::new();
    let allow_pattern = ["allow", "("].concat();

    for path in rust_files(&root) {
        let source = read_to_string(&path);
        for (line_number, line) in source.lines().enumerate() {
            if line.contains(&allow_pattern) && !line.contains("reason:") {
                let display = path.strip_prefix(&root).unwrap_or(&path);
                violations.push(format!("{}:{}", display.display(), line_number + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "allow exceptions need an inline reason:\n{}",
        violations.join("\n")
    );
}

fn section<'a>(document: &'a str, name: &str) -> Vec<&'a str> {
    let header = format!("[{name}]");
    let mut in_section = false;
    let mut lines = Vec::new();

    for line in document.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed == header;
        } else if in_section && !trimmed.is_empty() && !trimmed.starts_with('#') {
            lines.push(line);
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

fn enum_has_attribute(source: &str, enum_name: &str, attribute_name: &str) -> bool {
    let enum_declaration = format!("pub enum {enum_name}");
    let lines = source.lines().collect::<Vec<_>>();

    for (line_index, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with(&enum_declaration) {
            return line_index > 0
                && lines[..line_index]
                    .iter()
                    .rev()
                    .take_while(|line| {
                        let trimmed = line.trim_start();
                        trimmed.starts_with("#[") || trimmed.is_empty()
                    })
                    .any(|line| line.contains(attribute_name));
        }
    }
    false
}

fn public_enum_names(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let rest = trimmed.strip_prefix("pub enum ")?;
            let name = rest
                .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
                .next()?;
            Some(name.to_owned())
        })
        .collect()
}

fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_files(root, &mut files);
    files
}

fn collect_rust_files(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            collect_rust_files(&entry.unwrap().path(), files);
        }
    } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
        files.push(path.to_owned());
    }
}

fn read_to_string(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    })
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> PathBuf {
    crate_root().parent().unwrap().parent().unwrap().to_owned()
}
