use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

const PUBLIC_ENUM_DOCS: &[(&str, &str, &str)] = &[
    (
        "cache_adapter.rs",
        "CacheAdapterCacheability",
        "cache_adapter.md",
    ),
    ("cache_adapter.rs", "CacheAdapterMiss", "cache_adapter.md"),
    (
        "cache_adapter.rs",
        "CacheRehydrateOutcome",
        "cache_adapter.md",
    ),
    (
        "cache_adapter.rs",
        "EncodeCacheRecordOutcome",
        "cache_adapter.md",
    ),
    ("identity.rs", "IdentityError", "identity.md"),
    ("publisher.rs", "OutputOrigin", "publisher.md"),
    ("projection.rs", "ProjectionError", "projection.md"),
    (
        "projection.rs",
        "ProjectionExternalDependencyGap",
        "projection.md",
    ),
    ("publisher.rs", "PublicationTarget", "publisher.md"),
    ("publisher.rs", "PublishError", "publisher.md"),
    ("storage.rs", "StorageError", "storage.md"),
    ("storage.rs", "StoragePlacement", "storage.md"),
];

#[test]
fn ir_manifest_opts_into_workspace_lints() {
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
fn workspace_manifest_includes_mizar_ir_once() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let occurrences = manifest
        .lines()
        .filter(|line| line.trim() == "\"crates/mizar-ir\",")
        .count();

    assert_eq!(
        occurrences,
        1,
        "{} must list crates/mizar-ir exactly once in [workspace].members",
        manifest_path.display()
    );
}

#[test]
fn ir_manifest_keeps_task_one_package_metadata() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let package = section(&manifest, "package");
    let lib = section(&manifest, "lib");

    assert!(
        package
            .iter()
            .any(|line| assignment_is(line, "name", "mizar-ir")),
        "{} must keep the package name stable",
        manifest_path.display()
    );
    assert!(
        package
            .iter()
            .any(|line| assignment_is(line, "version", "0.1.0")),
        "{} must keep the task-1 crate version explicit until a release-policy \
         task changes it",
        manifest_path.display()
    );
    for key in [
        "edition.workspace",
        "license.workspace",
        "repository.workspace",
    ] {
        assert!(
            package.iter().any(|line| assignment_is(line, key, "true")),
            "{} must inherit {key} from the workspace",
            manifest_path.display()
        );
    }
    assert!(
        lib.iter()
            .any(|line| assignment_is(line, "name", "mizar_ir")),
        "{} must keep the library crate name stable",
        manifest_path.display()
    );
    assert!(
        lib.iter()
            .any(|line| assignment_is(line, "path", "src/lib.rs")),
        "{} must keep the task-1 library entry point at src/lib.rs",
        manifest_path.display()
    );
}

#[test]
fn ir_manifest_dependency_boundary_stays_narrow() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [(
            "dependencies".to_owned(),
            vec![
                "blake3 = \"1.8.5\"",
                "mizar-artifact = { path = \"../mizar-artifact\" }",
                "mizar-cache = { path = \"../mizar-cache\" }",
                "mizar-session = { path = \"../mizar-session\" }",
            ],
        )],
        "{} task 12 must keep production dependencies limited to deterministic \
         hashing, mizar-session, mizar-cache validation, and mizar-artifact \
         stable schemas; mizar-driver/mizar-diagnostics must remain absent",
        manifest_path.display()
    );

    for (section, lines) in &dependency_sections {
        for forbidden in ["mizar-build", "mizar-diagnostics", "mizar-driver"] {
            assert!(
                lines.iter().all(|line| !line.contains(forbidden)),
                "{} must not list premature downstream crate `{forbidden}` in [{section}]",
                manifest_path.display()
            );
        }
    }
}

#[test]
fn ir_lib_states_boundary_and_external_gaps() {
    let lib_path = crate_root().join("src/lib.rs");
    let source = read_to_string(&lib_path);

    for marker in [
        "#![forbid(unsafe_code)]",
        "Compiler-internal IR storage",
        "`mizar-ir` owns sealed internal phase-output storage",
        "consumes snapshot identity from `mizar-session`",
        "cache-key construction",
        "dependency fingerprints",
        "proof-reuse validation",
        "proof acceptance",
        "trusted",
        "kernel acceptance",
        "external dependency gaps",
    ] {
        assert!(
            source.contains(marker),
            "{} must keep the task-1 boundary marker `{marker}`",
            lib_path.display()
        );
    }
}

#[test]
fn public_forward_compatible_enums_are_marked_non_exhaustive() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in rust_source_files(&root.join("src")) {
        let source = read_to_string(&path);
        collect_public_enums_without_non_exhaustive(&root, &path, &source, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "public enum forward-compatibility policy requires #[non_exhaustive] on:\n{}",
        violations.join("\n")
    );
}

#[test]
fn public_enum_forward_compatibility_decisions_are_documented() {
    let root = crate_root();
    let actual = public_enums(&root.join("src"));
    let expected = PUBLIC_ENUM_DOCS
        .iter()
        .map(|(source, enum_name, _doc)| ((*source).to_owned(), (*enum_name).to_owned()))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual, expected,
        "every public enum must have a task-15 decision in its owning EN/JA module spec"
    );

    for (_source, enum_name, doc_file) in PUBLIC_ENUM_DOCS {
        for language in ["en", "ja"] {
            let path = workspace_root()
                .join("doc/design/mizar-ir")
                .join(language)
                .join(doc_file);
            let document = read_to_string(&path);
            assert!(
                document.contains(enum_name),
                "{} must record the task-15 decision for `{enum_name}`",
                path.display()
            );
            assert!(
                enum_policy_is_documented(&document, enum_name),
                "{} must document `{enum_name}` and `#[non_exhaustive]` in the same policy paragraph",
                path.display()
            );
        }
    }
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> PathBuf {
    crate_root()
        .parent()
        .and_then(|path| path.parent())
        .expect("crate lives under workspace/crates/mizar-ir")
        .to_path_buf()
}

fn read_to_string(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    })
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(path) = stack.pop() {
        if path.is_dir() {
            let mut entries = fs::read_dir(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .map(|entry| {
                    entry
                        .unwrap_or_else(|error| {
                            panic!(
                                "failed to read directory entry under {}: {error}",
                                path.display()
                            )
                        })
                        .path()
                })
                .collect::<Vec<_>>();
            entries.sort();
            stack.extend(entries.into_iter().rev());
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }

    files
}

fn collect_public_enums_without_non_exhaustive(
    root: &Path,
    path: &Path,
    source: &str,
    violations: &mut Vec<String>,
) {
    let lines = source.lines().collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("pub enum ")
            && !has_preceding_non_exhaustive_attribute(&lines, index)
        {
            let relative = path.strip_prefix(root).unwrap_or(path);
            violations.push(format!("{}:{}", relative.display(), index + 1));
        }
    }
}

fn has_preceding_non_exhaustive_attribute(lines: &[&str], enum_index: usize) -> bool {
    for line in lines[..enum_index].iter().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("///")
            || trimmed.starts_with("//!")
            || trimmed.starts_with("//")
        {
            continue;
        }
        if trimmed.starts_with("#[") || trimmed.starts_with("# [") {
            if trimmed.contains("non_exhaustive") {
                return true;
            }
            continue;
        }
        break;
    }

    false
}

fn public_enums(root: &Path) -> BTreeSet<(String, String)> {
    let mut enums = BTreeSet::new();
    for path in rust_source_files(root) {
        let source = read_to_string(&path);
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        for line in source.lines() {
            if let Some(name) = public_enum_name(line) {
                enums.insert((relative.clone(), name));
            }
        }
    }
    enums
}

fn public_enum_name(line: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix("pub enum ")?;
    let name = rest
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect::<String>();
    (!name.is_empty()).then_some(name)
}

fn enum_policy_is_documented(document: &str, enum_name: &str) -> bool {
    document
        .split("\n\n")
        .any(|paragraph| paragraph.contains(enum_name) && paragraph.contains("#[non_exhaustive]"))
}

fn section<'a>(manifest: &'a str, name: &str) -> Vec<&'a str> {
    let header = format!("[{name}]");
    let mut in_section = false;
    let mut lines = Vec::new();

    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed == header;
            continue;
        }
        if in_section && !trimmed.is_empty() && !trimmed.starts_with('#') {
            lines.push(trimmed);
        }
    }

    lines
}

fn dependency_sections(manifest: &str) -> Vec<(String, Vec<&str>)> {
    let mut sections = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_lines = Vec::new();

    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if let Some(name) = current_name.take() {
                sections.push((name, current_lines));
                current_lines = Vec::new();
            }

            let name = trimmed.trim_start_matches('[').trim_end_matches(']');
            if name.ends_with("dependencies") {
                current_name = Some(name.to_owned());
            }
            continue;
        }

        if current_name.is_some() && !trimmed.is_empty() && !trimmed.starts_with('#') {
            current_lines.push(trimmed);
        }
    }

    if let Some(name) = current_name {
        sections.push((name, current_lines));
    }

    sections
}

fn assignment_is(line: &str, key: &str, value: &str) -> bool {
    let Some((actual_key, actual_value)) = line.split_once('=') else {
        return false;
    };
    actual_key.trim() == key && actual_value.trim().trim_matches('"') == value
}
