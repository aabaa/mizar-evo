use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Debug)]
struct LegacyCompactionManifest {
    batches: BTreeMap<String, LegacyCompactionBatch>,
    tasks: BTreeMap<String, LegacyCompactionTask>,
    redirects: Vec<LegacyCompactionRedirect>,
    indexes: Vec<LegacyCompactionIndex>,
    raw_rows_by_batch: BTreeMap<String, Vec<String>>,
}

#[derive(Debug)]
struct LegacyCompactionBatch {
    id: String,
    contract_en: PathBuf,
    contract_ja: PathBuf,
    inventory_sha256: String,
    task_count: usize,
    redirect_count: usize,
    source_count: usize,
    index_count: usize,
}

#[derive(Debug)]
struct LegacyCompactionTask {
    batch_id: String,
    id: String,
    contract_en: PathBuf,
    contract_ja: PathBuf,
}

#[derive(Debug)]
struct LegacyCompactionRedirect {
    batch_id: String,
    task_id: String,
    language: String,
    source_path: PathBuf,
    heading_level: usize,
    legacy_heading: String,
    replacement: String,
    previous_heading: String,
    next_heading: String,
}

#[derive(Debug)]
struct LegacyCompactionIndex {
    batch_id: String,
    indexed_id: String,
    language: String,
    plan_path: PathBuf,
    row: String,
}

#[test]
fn mizar_test_manifest_opts_into_workspace_lints() {
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
fn mizar_test_allow_exceptions_are_documented_inline() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in mizar_test_rust_target_files(&root) {
        collect_undocumented_allows(&root, &path, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "intentional lint allow exceptions need an adjacent reason:\n{}",
        violations.join("\n")
    );
}

#[test]
fn public_enums_are_non_exhaustive_and_documented() {
    let root = crate_root();
    let workspace = workspace_root();
    let policies = public_enum_policies();
    let expected = policies
        .iter()
        .map(|policy| (policy.module.to_owned(), policy.name.to_owned()))
        .collect::<BTreeSet<_>>();
    let mut observed = BTreeSet::new();

    let mut source_paths = Vec::new();
    collect_rust_files(&root.join("src"), &mut source_paths);
    source_paths.sort();

    for source_path in source_paths {
        let Some(module) = source_path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let source = read_to_string(&source_path);
        for enum_name in public_enum_names(&source) {
            observed.insert((module.to_owned(), enum_name));
        }
    }

    assert_eq!(
        observed, expected,
        "public enum policy inventory must match crates/mizar-test/src"
    );

    for policy in policies {
        let source_path = root.join("src").join(format!("{}.rs", policy.module));
        let source = read_to_string(&source_path);
        assert!(
            public_enum_has_non_exhaustive(&source, policy.name),
            "{} must mark public enum {} as #[non_exhaustive]",
            source_path.display(),
            policy.name
        );

        for doc_path in [policy.en_doc, policy.ja_doc] {
            let doc_path = workspace.join(doc_path);
            let doc = read_to_string(&doc_path);
            assert!(
                doc_has_public_enum_policy(&doc, policy.name),
                "{} must document {} as #[non_exhaustive]",
                doc_path.display(),
                policy.name
            );
        }
    }
}

#[test]
fn task_contracts_are_recursively_paired_and_supported_links_resolve() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        markdown_link_destinations("``code ` [ignored](ignored.md) ` code`` [owner](owner.md)"),
        vec!["owner.md"]
    );
    assert_legacy_heading_boundary_vectors();
    assert_legacy_document_evidence_vectors();
    assert_legacy_path_scoped_heading_vectors();
    assert_eq!(
        markdown_link_destinations("```text\n~~~\n[ignored](ignored.md)\n```\n[owner](owner.md)\n"),
        vec!["owner.md"]
    );
    assert_eq!(
        markdown_h2_section(
            "```md\n## Task Index\n| fenced |\n```\n## Task Index\n| visible |\n## Next\n",
            "Task Index"
        ),
        Some("## Task Index\n| visible |\n")
    );

    let workspace = workspace_root();
    let manifest_document =
        read_to_string(&workspace.join("doc/design/task_contracts/legacy_compactions.tsv"));
    assert_legacy_manifest_mutation_vectors(&workspace, &manifest_document);
    let contract_root = workspace.join("doc/design/task_contracts");
    let en_root = contract_root.join("en");
    let ja_root = contract_root.join("ja");
    let en_paths = relative_markdown_paths(&en_root);
    let ja_paths = relative_markdown_paths(&ja_root);

    assert_eq!(
        en_paths, ja_paths,
        "task-contract EN/JA trees must contain identical relative Markdown paths"
    );

    let mut ids = BTreeMap::new();
    let mut violations = Vec::new();
    validate_legacy_compaction_manifest(&workspace, &mut violations);
    for relative_path in &en_paths {
        let Some(task_id) = relative_path.file_stem().and_then(|stem| stem.to_str()) else {
            violations.push(format!(
                "{}: task-contract filename must be UTF-8",
                relative_path.display()
            ));
            continue;
        };
        if !valid_task_contract_id(task_id) {
            violations.push(format!(
                "{}: task id must match [A-Za-z0-9][A-Za-z0-9._-]*",
                relative_path.display()
            ));
        }
        if let Some(previous) = ids.insert(task_id.to_owned(), relative_path.clone()) {
            violations.push(format!(
                "{}: duplicate task id {task_id} also used by {}",
                relative_path.display(),
                previous.display()
            ));
        }

        for (language, root, marker, counterpart_root) in [
            (
                "en",
                en_root.as_path(),
                "Canonical language: English",
                ja_root.as_path(),
            ),
            (
                "ja",
                ja_root.as_path(),
                "canonical English:",
                en_root.as_path(),
            ),
        ] {
            let path = root.join(relative_path);
            let document = read_to_string(&path);
            let title_prefix = format!("# Task {task_id}:");
            if !document
                .lines()
                .next()
                .is_some_and(|title| title.starts_with(&title_prefix))
            {
                violations.push(format!(
                    "{}: first heading must start with {title_prefix:?}",
                    path.display()
                ));
            }
            if !document.contains(marker) {
                violations.push(format!(
                    "{}: missing {language} canonical/companion marker {marker:?}",
                    path.display()
                ));
            }

            let counterpart = counterpart_root.join(relative_path);
            let counterpart = fs::canonicalize(&counterpart).unwrap_or_else(|error| {
                panic!(
                    "failed to resolve paired contract {}: {error}",
                    counterpart.display()
                )
            });
            let links = markdown_link_destinations(&document);
            if !links.iter().any(|destination| {
                markdown_target_path(&path, destination)
                    .and_then(|target| fs::canonicalize(target).ok())
                    .is_some_and(|target| target == counterpart)
            }) {
                violations.push(format!(
                    "{}: missing reciprocal link to {}",
                    path.display(),
                    counterpart.display()
                ));
            }

            validate_crate_plan_backlinks(&path, &links, &mut violations);
            validate_local_markdown_links(&path, &document, &mut violations);
        }
    }

    assert!(
        violations.is_empty(),
        "task-contract policy violations:\n{}",
        violations.join("\n")
    );
}

#[test]
fn allow_detector_covers_common_attribute_shapes() {
    let samples = [
        "#[allow(dead_code)]",
        "# [allow(dead_code)]",
        "#[ allow(dead_code)]",
        "#[allow (dead_code)]",
        "#![allow(dead_code)]",
        "#! [allow(dead_code)]",
        "# ! [allow(dead_code)]",
        "#[cfg_attr(test, allow(dead_code))]",
        "#[\n    cfg_attr(test,\n        allow(dead_code)\n    )\n]",
    ];

    for sample in samples {
        assert!(is_allow_attribute(sample), "{sample}");
    }
    assert!(!is_allow_attribute("#[doc = \"allow(dead_code)\"]"));
}

#[test]
fn allow_scanner_does_not_skip_after_non_allow_attribute_with_bracket_in_string() {
    let source = "#[doc = \"open [bracket\"]\n#[allow(dead_code)]\nfn sample() {}\n";

    assert_eq!(undocumented_allow_line_numbers(source), vec![2]);
}

#[test]
fn allow_scanner_keeps_multiline_cfg_attr_open_past_bracket_in_string() {
    let source = "#[\n    cfg_attr(\n        test,\n        doc = \"close ] bracket\",\n        allow(dead_code)\n    )\n]\nfn sample() {}\n";

    assert_eq!(undocumented_allow_line_numbers(source), vec![1]);
}

#[test]
fn allow_scanner_keeps_multiline_cfg_attr_open_past_bracket_in_comment() {
    let source = "#[cfg_attr(\n    test, // ]\n    allow(dead_code)\n)]\nfn sample() {}\n";

    assert_eq!(undocumented_allow_line_numbers(source), vec![1]);
}

#[test]
fn allow_scanner_keeps_multiline_cfg_attr_open_past_bracket_in_raw_string() {
    let source = "#[cfg_attr(\n    test,\n    doc = r#\"close \" ] bracket\"#,\n    allow(dead_code)\n)]\nfn sample() {}\n";

    assert_eq!(undocumented_allow_line_numbers(source), vec![1]);
}

#[test]
fn allow_rationale_must_be_in_comment_text() {
    let source = "#[allow(dead_code)]\nconst reason: &str = \"not a comment\";\n";
    assert_eq!(undocumented_allow_line_numbers(source), vec![1]);

    let source = "#[allow(dead_code)]\nconst S: &str = \"// reason: not a comment\";\n";
    assert_eq!(undocumented_allow_line_numbers(source), vec![1]);

    let source = "#[allow(dead_code)] // reason: compatibility fixture\nfn sample() {}\n";
    assert!(undocumented_allow_line_numbers(source).is_empty());
}

#[test]
fn manifest_target_path_parser_covers_custom_cargo_targets() {
    let root = Path::new("crate");
    let manifest = r#"
[package]
build = "build/custom.rs"

[lib] # library target
path = "lib/custom.rs"

[[bin]] # binary target
name = "mizar-test"
path = "tools/mizar_test.rs"

[[test]]
name = "custom-test"
path = "custom/test_entry.rs"
"#;

    assert_eq!(
        explicit_manifest_target_paths_from_manifest(root, manifest),
        vec![
            PathBuf::from("crate/build/custom.rs"),
            PathBuf::from("crate/lib/custom.rs"),
            PathBuf::from("crate/tools/mizar_test.rs"),
            PathBuf::from("crate/custom/test_entry.rs"),
        ]
    );
}

#[test]
fn explicit_manifest_target_parent_directory_is_scanned_for_modules() {
    let root = std::env::temp_dir().join(format!("mizar_test_lint_policy_{}", std::process::id()));
    remove_dir_if_exists(&root);
    create_dir(&root.join("tools"));
    write_test_file(
        &root.join("Cargo.toml"),
        r#"
[[bin]]
name = "mizar-test"
path = "tools/mizar_test.rs"
"#,
    );
    write_test_file(
        &root.join("tools/mizar_test.rs"),
        "mod helper;\nfn main() {}\n",
    );
    write_test_file(
        &root.join("tools/helper.rs"),
        "#[allow(dead_code)]\nfn helper() {}\n",
    );

    let files = mizar_test_rust_target_files(&root);

    assert!(files.contains(&root.join("tools/helper.rs")));
    remove_dir_if_exists(&root);
}

#[test]
fn implicit_cargo_target_locations_are_scanned() {
    let root = std::env::temp_dir().join(format!(
        "mizar_test_lint_policy_implicit_{}",
        std::process::id()
    ));
    remove_dir_if_exists(&root);
    create_dir(&root.join("examples"));
    create_dir(&root.join("benches"));
    write_test_file(&root.join("Cargo.toml"), "[package]\nname = \"sample\"\n");
    write_test_file(&root.join("build.rs"), "fn main() {}\n");
    write_test_file(&root.join("examples/demo.rs"), "fn main() {}\n");
    write_test_file(&root.join("benches/smoke.rs"), "fn main() {}\n");

    let files = mizar_test_rust_target_files(&root);

    assert!(files.contains(&root.join("build.rs")));
    assert!(files.contains(&root.join("examples/demo.rs")));
    assert!(files.contains(&root.join("benches/smoke.rs")));
    remove_dir_if_exists(&root);
}

#[test]
fn allow_scanner_ignores_attribute_text_inside_multiline_strings() {
    let ordinary_string = "const S: &str = \"\n#[allow(dead_code)]\n\";\n";
    assert!(undocumented_allow_line_numbers(ordinary_string).is_empty());

    let lifetime_string = "const S: &'static str = \"\n#[allow(dead_code)]\n\";\n";
    assert!(undocumented_allow_line_numbers(lifetime_string).is_empty());

    let raw_string = r##"const S: &str = r#"
#[allow(dead_code)]
"#;
"##;
    assert!(undocumented_allow_line_numbers(raw_string).is_empty());
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

#[derive(Clone, Copy)]
struct PublicEnumPolicy {
    module: &'static str,
    name: &'static str,
    en_doc: &'static str,
    ja_doc: &'static str,
}

fn public_enum_policies() -> Vec<PublicEnumPolicy> {
    vec![
        PublicEnumPolicy {
            module: "diagnostic",
            name: "ValidationSeverity",
            en_doc: "doc/design/mizar-test/en/harness.md",
            ja_doc: "doc/design/mizar-test/ja/harness.md",
        },
        PublicEnumPolicy {
            module: "expectation",
            name: "TestKind",
            en_doc: "doc/design/mizar-test/en/expectation_schema.md",
            ja_doc: "doc/design/mizar-test/ja/expectation_schema.md",
        },
        PublicEnumPolicy {
            module: "expectation",
            name: "ExpectedOutcome",
            en_doc: "doc/design/mizar-test/en/expectation_schema.md",
            ja_doc: "doc/design/mizar-test/ja/expectation_schema.md",
        },
        PublicEnumPolicy {
            module: "expectation",
            name: "PipelinePhase",
            en_doc: "doc/design/mizar-test/en/expectation_schema.md",
            ja_doc: "doc/design/mizar-test/ja/expectation_schema.md",
        },
        PublicEnumPolicy {
            module: "expectation",
            name: "Architecture22Gate",
            en_doc: "doc/design/mizar-test/en/expectation_schema.md",
            ja_doc: "doc/design/mizar-test/ja/expectation_schema.md",
        },
        PublicEnumPolicy {
            module: "harness",
            name: "TestProfile",
            en_doc: "doc/design/mizar-test/en/harness.md",
            ja_doc: "doc/design/mizar-test/ja/harness.md",
        },
        PublicEnumPolicy {
            module: "harness",
            name: "ValidationMode",
            en_doc: "doc/design/mizar-test/en/harness.md",
            ja_doc: "doc/design/mizar-test/ja/harness.md",
        },
        PublicEnumPolicy {
            module: "harness",
            name: "HarnessError",
            en_doc: "doc/design/mizar-test/en/harness.md",
            ja_doc: "doc/design/mizar-test/ja/harness.md",
        },
        PublicEnumPolicy {
            module: "runner",
            name: "ParseOnlyCaseStatus",
            en_doc: "doc/design/mizar-test/en/harness.md",
            ja_doc: "doc/design/mizar-test/ja/harness.md",
        },
        PublicEnumPolicy {
            module: "runner",
            name: "DeclarationSymbolCaseStatus",
            en_doc: "doc/design/mizar-test/en/harness.md",
            ja_doc: "doc/design/mizar-test/ja/harness.md",
        },
        PublicEnumPolicy {
            module: "runner",
            name: "TypeElaborationCaseStatus",
            en_doc: "doc/design/mizar-test/en/harness.md",
            ja_doc: "doc/design/mizar-test/ja/harness.md",
        },
        PublicEnumPolicy {
            module: "runner",
            name: "ProofVerificationCaseStatus",
            en_doc: "doc/design/mizar-test/en/harness.md",
            ja_doc: "doc/design/mizar-test/ja/harness.md",
        },
        PublicEnumPolicy {
            module: "snapshot",
            name: "SnapshotKind",
            en_doc: "doc/design/mizar-test/en/snapshot.md",
            ja_doc: "doc/design/mizar-test/ja/snapshot.md",
        },
        PublicEnumPolicy {
            module: "snapshot",
            name: "ParallelismProfile",
            en_doc: "doc/design/mizar-test/en/snapshot.md",
            ja_doc: "doc/design/mizar-test/ja/snapshot.md",
        },
        PublicEnumPolicy {
            module: "snapshot",
            name: "SnapshotUpdateReason",
            en_doc: "doc/design/mizar-test/en/snapshot.md",
            ja_doc: "doc/design/mizar-test/ja/snapshot.md",
        },
        PublicEnumPolicy {
            module: "snapshot",
            name: "SnapshotUpdateMode",
            en_doc: "doc/design/mizar-test/en/snapshot.md",
            ja_doc: "doc/design/mizar-test/ja/snapshot.md",
        },
        PublicEnumPolicy {
            module: "snapshot",
            name: "SnapshotBaselineStatus",
            en_doc: "doc/design/mizar-test/en/snapshot.md",
            ja_doc: "doc/design/mizar-test/ja/snapshot.md",
        },
        PublicEnumPolicy {
            module: "snapshot",
            name: "SnapshotBaselineError",
            en_doc: "doc/design/mizar-test/en/snapshot.md",
            ja_doc: "doc/design/mizar-test/ja/snapshot.md",
        },
        PublicEnumPolicy {
            module: "snapshot",
            name: "SnapshotError",
            en_doc: "doc/design/mizar-test/en/snapshot.md",
            ja_doc: "doc/design/mizar-test/ja/snapshot.md",
        },
        PublicEnumPolicy {
            module: "staged_model",
            name: "Stage",
            en_doc: "doc/design/mizar-test/en/staged_model.md",
            ja_doc: "doc/design/mizar-test/ja/staged_model.md",
        },
        PublicEnumPolicy {
            module: "toml_lite",
            name: "TomlValue",
            en_doc: "doc/design/mizar-test/en/expectation_schema.md",
            ja_doc: "doc/design/mizar-test/ja/expectation_schema.md",
        },
        PublicEnumPolicy {
            module: "traceability",
            name: "RequirementStatus",
            en_doc: "doc/design/mizar-test/en/traceability.md",
            ja_doc: "doc/design/mizar-test/ja/traceability.md",
        },
        PublicEnumPolicy {
            module: "traceability",
            name: "CoverageShape",
            en_doc: "doc/design/mizar-test/en/traceability.md",
            ja_doc: "doc/design/mizar-test/ja/traceability.md",
        },
    ]
}

fn public_enum_names(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("pub enum "))
        .filter_map(|rest| {
            rest.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
                .next()
        })
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

fn public_enum_has_non_exhaustive(source: &str, name: &str) -> bool {
    let Some(position) = source.find(&format!("pub enum {name}")) else {
        return false;
    };
    for line in source[..position].lines().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.starts_with("#[") {
            return false;
        }
        if trimmed == "#[non_exhaustive]" {
            return true;
        }
    }
    false
}

fn doc_has_public_enum_policy(doc: &str, name: &str) -> bool {
    let row_prefix = format!("| `{name}` |");
    doc.lines()
        .any(|line| line.contains(&row_prefix) && line.contains("`#[non_exhaustive]`"))
}

fn collect_undocumented_allows(root: &Path, path: &Path, violations: &mut Vec<String>) {
    let source = read_to_string(path);

    for line_number in undocumented_allow_line_numbers(&source) {
        let display_path = path.strip_prefix(root).unwrap_or(path);
        violations.push(format!("{}:{line_number}", display_path.display()));
    }
}

fn undocumented_allow_line_numbers(source: &str) -> Vec<usize> {
    let lines = source.lines().collect::<Vec<_>>();
    let line_starts_in_code = line_start_code_states(&lines);
    let mut violations = Vec::new();
    let mut line_index = 0;

    while line_index < lines.len() {
        if !line_starts_in_code[line_index] || !starts_attribute(lines[line_index]) {
            line_index += 1;
            continue;
        }

        let (attribute, end_line_index) = attribute_block(&lines, line_index);
        if is_allow_attribute(&attribute)
            && !has_adjacent_allow_rationale(&lines, line_index, end_line_index)
        {
            violations.push(line_index + 1);
        }
        line_index += 1;
    }

    violations
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LexState {
    Code,
    String { quote: u8, escaped: bool },
    RawString { hashes: usize },
    BlockComment { depth: usize },
}

fn line_start_code_states(lines: &[&str]) -> Vec<bool> {
    let mut state = LexState::Code;
    let mut starts = Vec::with_capacity(lines.len());

    for line in lines {
        starts.push(state == LexState::Code);
        advance_lex_state(line, &mut state);
    }

    starts
}

fn advance_lex_state(line: &str, state: &mut LexState) {
    let bytes = line.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        match *state {
            LexState::Code => {
                if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'/') {
                    break;
                }
                if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'*') {
                    *state = LexState::BlockComment { depth: 1 };
                    index += 2;
                    continue;
                }
                if bytes[index] == b'r'
                    && let Some((hashes, consumed)) = raw_string_start(bytes, index)
                {
                    *state = LexState::RawString { hashes };
                    index += consumed;
                    continue;
                }
                if bytes[index] == b'"' {
                    *state = LexState::String {
                        quote: bytes[index],
                        escaped: false,
                    };
                }
                index += 1;
            }
            LexState::String { quote, escaped } => {
                if escaped {
                    *state = LexState::String {
                        quote,
                        escaped: false,
                    };
                } else if bytes[index] == b'\\' {
                    *state = LexState::String {
                        quote,
                        escaped: true,
                    };
                } else if bytes[index] == quote {
                    *state = LexState::Code;
                }
                index += 1;
            }
            LexState::RawString { hashes } => {
                if raw_string_end(bytes, index, hashes) {
                    *state = LexState::Code;
                    index += 1 + hashes;
                } else {
                    index += 1;
                }
            }
            LexState::BlockComment { mut depth } => {
                if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'*') {
                    depth += 1;
                    *state = LexState::BlockComment { depth };
                    index += 2;
                } else if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
                    depth -= 1;
                    *state = if depth == 0 {
                        LexState::Code
                    } else {
                        LexState::BlockComment { depth }
                    };
                    index += 2;
                } else {
                    index += 1;
                }
            }
        }
    }
}

fn raw_string_start(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    if bytes.get(start) != Some(&b'r') {
        return None;
    }

    let mut index = start + 1;
    let mut hashes = 0;
    while bytes.get(index) == Some(&b'#') {
        hashes += 1;
        index += 1;
    }
    if bytes.get(index) != Some(&b'"') {
        return None;
    }

    Some((hashes, index - start + 1))
}

fn raw_string_end(bytes: &[u8], start: usize, hashes: usize) -> bool {
    bytes.get(start) == Some(&b'"')
        && (0..hashes).all(|offset| bytes.get(start + 1 + offset) == Some(&b'#'))
}

fn starts_attribute(line: &str) -> bool {
    let compact = line
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();

    compact.starts_with("#[") || compact.starts_with("#![")
}

fn attribute_block(lines: &[&str], start: usize) -> (String, usize) {
    let mut block = String::new();
    let mut bracket_depth = 0_i32;
    let mut saw_opening_bracket = false;
    let mut string_quote = None;
    let mut escaped = false;
    let mut in_block_comment = false;

    for (line_index, line) in lines.iter().enumerate().skip(start) {
        if !block.is_empty() {
            block.push('\n');
        }
        block.push_str(line);

        let mut characters = line.chars().peekable();
        while let Some(character) = characters.next() {
            if in_block_comment {
                if character == '*' && characters.next_if_eq(&'/').is_some() {
                    in_block_comment = false;
                }
                continue;
            }

            if let Some(quote) = string_quote {
                if escaped {
                    escaped = false;
                } else if character == '\\' {
                    escaped = true;
                } else if character == quote {
                    string_quote = None;
                }
                continue;
            }

            if character == '/' {
                if characters.next_if_eq(&'/').is_some() {
                    break;
                }
                if characters.next_if_eq(&'*').is_some() {
                    in_block_comment = true;
                    continue;
                }
            }

            if character == 'r' && skip_raw_string(&mut characters) {
                continue;
            }

            if character == '"' || character == '\'' {
                string_quote = Some(character);
                continue;
            }

            if character == '[' {
                bracket_depth += 1;
                saw_opening_bracket = true;
            } else if character == ']' && saw_opening_bracket {
                bracket_depth -= 1;
            }
        }

        if saw_opening_bracket && bracket_depth <= 0 {
            return (block, line_index);
        }
    }

    (block, lines.len().saturating_sub(1))
}

fn is_allow_attribute(attribute: &str) -> bool {
    let compact = compact_attribute_tokens(attribute);

    compact.starts_with("#[allow(")
        || compact.starts_with("#![allow(")
        || (compact.starts_with("#[cfg_attr(") && compact.contains(",allow("))
        || (compact.starts_with("#![cfg_attr(") && compact.contains(",allow("))
}

fn compact_attribute_tokens(attribute: &str) -> String {
    let mut compact = String::new();
    let mut characters = attribute.chars().peekable();
    let mut string_quote = None;
    let mut escaped = false;
    let mut in_block_comment = false;

    while let Some(character) = characters.next() {
        if in_block_comment {
            if character == '*' && characters.next_if_eq(&'/').is_some() {
                in_block_comment = false;
            }
            continue;
        }

        if let Some(quote) = string_quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == quote {
                string_quote = None;
            }
            continue;
        }

        if character == '/' {
            if characters.next_if_eq(&'/').is_some() {
                for next in characters.by_ref() {
                    if next == '\n' {
                        break;
                    }
                }
                continue;
            }
            if characters.next_if_eq(&'*').is_some() {
                in_block_comment = true;
                continue;
            }
        }

        if character == 'r' && skip_raw_string(&mut characters) {
            continue;
        }

        if character == '"' || character == '\'' {
            string_quote = Some(character);
            continue;
        }

        if !character.is_whitespace() {
            compact.push(character);
        }
    }

    compact
}

fn skip_raw_string(characters: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    let mut probe = characters.clone();
    let mut hashes = 0;
    while probe.next_if_eq(&'#').is_some() {
        hashes += 1;
    }
    if probe.next_if_eq(&'"').is_none() {
        return false;
    }

    for _ in 0..hashes {
        characters.next();
    }
    characters.next();

    while let Some(character) = characters.next() {
        if character != '"' {
            continue;
        }

        let mut closing_hashes = 0;
        while closing_hashes < hashes && characters.next_if_eq(&'#').is_some() {
            closing_hashes += 1;
        }
        if closing_hashes == hashes {
            break;
        }
    }

    true
}

fn has_adjacent_allow_rationale(lines: &[&str], start: usize, end: usize) -> bool {
    lines[start..=end]
        .iter()
        .any(|line| allow_rationale_in(line))
        || start
            .checked_sub(1)
            .is_some_and(|previous| allow_rationale_in(lines[previous]))
        || lines
            .get(end + 1)
            .is_some_and(|next| allow_rationale_in(next))
}

fn allow_rationale_in(line: &str) -> bool {
    let Some(comment) = comment_text(line) else {
        return false;
    };

    let lower = comment.to_ascii_lowercase();
    lower.contains("reason:") || lower.contains("rationale:")
}

fn comment_text(line: &str) -> Option<&str> {
    if let Some((before_comment, comment)) = line.split_once("//")
        && !before_comment.contains('"')
    {
        return Some(comment);
    }

    let trimmed = line.trim_start();
    if trimmed.starts_with("/*") || trimmed.starts_with('*') {
        Some(trimmed)
    } else {
        None
    }
}

fn mizar_test_rust_target_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    collect_rust_files(root, &mut files);
    add_explicit_manifest_target_files(root, &mut files);
    files.sort();
    files.dedup();
    files
}

fn add_explicit_manifest_target_files(root: &Path, files: &mut Vec<PathBuf>) {
    for target_path in explicit_manifest_target_paths(root) {
        if let Some(parent) = target_path.parent()
            && parent.exists()
        {
            collect_rust_files(parent, files);
        }
        files.push(target_path);
    }
}

fn explicit_manifest_target_paths(root: &Path) -> Vec<PathBuf> {
    let manifest = read_to_string(&root.join("Cargo.toml"));
    explicit_manifest_target_paths_from_manifest(root, &manifest)
}

fn explicit_manifest_target_paths_from_manifest(root: &Path, manifest: &str) -> Vec<PathBuf> {
    let mut section = "";
    let mut paths = Vec::new();

    for line in manifest.lines() {
        let trimmed = line.trim();
        if let Some(section_name) = manifest_section_name(trimmed) {
            section = section_name;
            continue;
        }

        if section == "package" {
            if let Some(path) = quoted_assignment(trimmed, "build") {
                paths.push(root.join(path));
            }
        } else if is_rust_target_section(section)
            && let Some(path) = quoted_assignment(trimmed, "path")
        {
            paths.push(root.join(path));
        }
    }

    paths
}

fn manifest_section_name(line: &str) -> Option<&str> {
    let header = line
        .split_once('#')
        .map_or(line, |(before_comment, _)| before_comment)
        .trim();

    if header.starts_with('[') && header.ends_with(']') {
        Some(header.trim_start_matches('[').trim_end_matches(']'))
    } else {
        None
    }
}

fn is_rust_target_section(section: &str) -> bool {
    matches!(section, "lib" | "bin" | "test" | "example" | "bench")
}

fn quoted_assignment<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let (lhs, rhs) = line.split_once('=')?;
    if lhs.trim() != key {
        return None;
    }

    let value = rhs.trim();
    quoted_value(value, '"').or_else(|| quoted_value(value, '\''))
}

fn quoted_value(value: &str, quote: char) -> Option<&str> {
    let rest = value.strip_prefix(quote)?;
    rest.split_once(quote).map(|(quoted, _)| quoted)
}

fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if dir.file_name().and_then(|name| name.to_str()) == Some("target") {
        return;
    }

    let entries = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()));

    for entry in entries {
        let entry =
            entry.unwrap_or_else(|error| panic!("failed to read {} entry: {error}", dir.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn relative_markdown_paths(root: &Path) -> BTreeSet<PathBuf> {
    let mut paths = Vec::new();
    collect_markdown_files(root, &mut paths);
    paths
        .into_iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to make {} relative to {}: {error}",
                        path.display(),
                        root.display()
                    )
                })
                .to_path_buf()
        })
        .collect()
}

fn collect_markdown_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()));

    for entry in entries {
        let entry =
            entry.unwrap_or_else(|error| panic!("failed to read {} entry: {error}", dir.display()));
        let file_type = entry.file_type().unwrap_or_else(|error| {
            panic!("failed to inspect {} entry type: {error}", dir.display())
        });
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            collect_markdown_files(&path, files);
        } else if file_type.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("md")
        {
            files.push(path);
        }
    }
}

fn valid_task_contract_id(task_id: &str) -> bool {
    let mut bytes = task_id.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn markdown_link_destinations(document: &str) -> Vec<String> {
    let mut destinations = Vec::new();
    let mut fence = None;

    for line in document.lines() {
        let trimmed = line.trim_start();
        if update_fence_state(trimmed, &mut fence) {
            continue;
        }
        if fence.is_some() {
            continue;
        }

        let visible = without_inline_code(line);
        let mut remainder = visible.as_str();
        while let Some(start) = remainder.find("](") {
            let after_open = &remainder[start + 2..];
            let Some(end) = after_open.find(')') else {
                break;
            };
            let raw = after_open[..end].trim();
            let destination = if let Some(angle) = raw.strip_prefix('<') {
                angle.split_once('>').map_or(angle, |(target, _)| target)
            } else {
                raw.split_whitespace().next().unwrap_or("")
            };
            if !destination.is_empty() {
                destinations.push(destination.to_owned());
            }
            remainder = &after_open[end + 1..];
        }
    }

    destinations
}

fn without_inline_code(line: &str) -> String {
    let mut visible = String::with_capacity(line.len());
    let mut delimiter = None;
    let mut characters = line.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '`' {
            let mut ticks = 1_usize;
            while characters.next_if_eq(&'`').is_some() {
                ticks += 1;
            }
            if delimiter == Some(ticks) {
                delimiter = None;
            } else if delimiter.is_none() {
                delimiter = Some(ticks);
            }
        } else if delimiter.is_none() {
            visible.push(character);
        }
    }
    visible
}

fn update_fence_state(line: &str, fence: &mut Option<(char, usize)>) -> bool {
    let Some(delimiter) = line
        .chars()
        .next()
        .filter(|character| matches!(character, '`' | '~'))
    else {
        return false;
    };
    let length = line
        .chars()
        .take_while(|character| *character == delimiter)
        .count();
    if length < 3 {
        return false;
    }
    let remainder = &line[length..];

    match *fence {
        None => {
            *fence = Some((delimiter, length));
            true
        }
        Some((active, minimum))
            if active == delimiter && length >= minimum && remainder.trim().is_empty() =>
        {
            *fence = None;
            true
        }
        Some(_) => false,
    }
}

fn markdown_target_path(source_path: &Path, destination: &str) -> Option<PathBuf> {
    if destination.starts_with("http://")
        || destination.starts_with("https://")
        || destination.starts_with("mailto:")
    {
        return None;
    }

    let target = destination
        .split_once('#')
        .map_or(destination, |(target, _)| target);
    if target.is_empty() {
        return Some(source_path.to_path_buf());
    }
    if Path::new(target)
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("md")
    {
        return None;
    }

    source_path.parent().map(|parent| parent.join(target))
}

fn validate_local_markdown_links(source_path: &Path, document: &str, violations: &mut Vec<String>) {
    for destination in markdown_link_destinations(document) {
        let Some(target_path) = markdown_target_path(source_path, &destination) else {
            continue;
        };
        if !target_path.is_file() {
            violations.push(format!(
                "{}: local Markdown target does not exist: {destination}",
                source_path.display()
            ));
            continue;
        }

        let Some((_, fragment)) = destination.split_once('#') else {
            continue;
        };
        if fragment.is_empty() {
            continue;
        }
        let target_document = if target_path == source_path {
            document.to_owned()
        } else {
            read_to_string(&target_path)
        };
        if !markdown_heading_slugs(&target_document).contains(fragment) {
            violations.push(format!(
                "{}: fragment #{fragment} does not exist in {}",
                source_path.display(),
                target_path.display()
            ));
        }
    }
}

fn validate_crate_plan_backlinks(
    contract_path: &Path,
    destinations: &[String],
    violations: &mut Vec<String>,
) {
    let contract = fs::canonicalize(contract_path).unwrap_or_else(|error| {
        panic!(
            "failed to resolve task contract {}: {error}",
            contract_path.display()
        )
    });
    let linked_plans = destinations
        .iter()
        .filter_map(|destination| markdown_target_path(contract_path, destination))
        .filter(|target| {
            target.file_name().and_then(|name| name.to_str()) == Some("00.crate_plan.md")
        })
        .filter_map(|target| fs::canonicalize(target).ok())
        .collect::<BTreeSet<_>>();
    if linked_plans.is_empty() {
        violations.push(format!(
            "{}: task contract must link at least one owning crate plan",
            contract_path.display()
        ));
    }

    let mut indexed_plans = BTreeSet::new();
    for plan_path in crate_plan_paths(&workspace_root().join("doc/design")) {
        let plan = read_to_string(&plan_path);
        let Some(task_index) = markdown_h2_section(&plan, "Task Index") else {
            continue;
        };
        let has_backlink = markdown_link_destinations(task_index)
            .iter()
            .any(|destination| {
                markdown_target_path(&plan_path, destination)
                    .and_then(|target| fs::canonicalize(target).ok())
                    .is_some_and(|target| target == contract)
            });
        if has_backlink {
            indexed_plans.insert(fs::canonicalize(&plan_path).unwrap_or_else(|error| {
                panic!(
                    "failed to resolve crate plan {}: {error}",
                    plan_path.display()
                )
            }));
        }
    }

    for plan_path in indexed_plans.difference(&linked_plans) {
        violations.push(format!(
            "{}: indexed owning crate plan {} is missing from contract links",
            contract_path.display(),
            plan_path.display()
        ));
    }
    for plan_path in linked_plans.difference(&indexed_plans) {
        if plan_path.is_file() {
            violations.push(format!(
                "{}: owning crate plan {} must link back to the contract",
                contract_path.display(),
                plan_path.display()
            ));
        }
    }
}

fn crate_plan_paths(design_root: &Path) -> Vec<PathBuf> {
    let mut plans = Vec::new();
    let canonical_design_root = fs::canonicalize(design_root).unwrap_or_else(|error| {
        panic!(
            "failed to resolve design root {}: {error}",
            design_root.display()
        )
    });
    let entries = fs::read_dir(design_root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", design_root.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            panic!("failed to read {} entry: {error}", design_root.display())
        });
        let file_type = entry.file_type().unwrap_or_else(|error| {
            panic!(
                "failed to inspect {} entry type: {error}",
                design_root.display()
            )
        });
        if !file_type.is_dir() || file_type.is_symlink() {
            continue;
        }
        for language in ["en", "ja"] {
            let plan = entry.path().join(language).join("00.crate_plan.md");
            let is_regular_file =
                fs::symlink_metadata(&plan).is_ok_and(|metadata| metadata.file_type().is_file());
            let is_contained = fs::canonicalize(&plan)
                .is_ok_and(|canonical| canonical.starts_with(&canonical_design_root));
            if is_regular_file && is_contained {
                plans.push(plan);
            }
        }
    }
    plans.sort();
    plans
}

fn validate_legacy_compaction_manifest(workspace: &Path, violations: &mut Vec<String>) {
    let manifest_path = workspace.join("doc/design/task_contracts/legacy_compactions.tsv");
    if !manifest_path.is_file() {
        violations.push(format!(
            "{}: legacy compaction manifest is missing",
            manifest_path.display()
        ));
        return;
    }
    let manifest_text = read_to_string(&manifest_path);
    violations.extend(legacy_compaction_manifest_violations(
        workspace,
        &manifest_text,
    ));
}

fn legacy_compaction_manifest_violations(workspace: &Path, document: &str) -> Vec<String> {
    let mut violations = Vec::new();
    if let Some(manifest) = parse_legacy_compaction_manifest(document, &mut violations) {
        validate_legacy_manifest_relations(workspace, &manifest, &mut violations);
        validate_legacy_manifest_documents(workspace, &manifest, &mut violations);
    }
    violations
}

fn parse_legacy_compaction_manifest(
    document: &str,
    violations: &mut Vec<String>,
) -> Option<LegacyCompactionManifest> {
    if document.contains('\r') {
        violations.push("legacy compaction manifest must use LF line endings".to_owned());
    }

    let data_rows = document
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<_>>();
    let Some((schema_line_number, schema)) = data_rows.first().copied() else {
        violations.push("legacy compaction manifest is empty".to_owned());
        return None;
    };
    if schema != "schema\t1" {
        violations.push(format!(
            "legacy compaction manifest:{}: first data row must be schema<TAB>1",
            schema_line_number + 1
        ));
    }

    for window in data_rows[1..].windows(2) {
        if window[0].1.as_bytes() >= window[1].1.as_bytes() {
            violations.push(format!(
                "legacy compaction manifest:{}: data rows must be strictly byte-sorted",
                window[1].0 + 1
            ));
        }
    }

    let mut manifest = LegacyCompactionManifest {
        batches: BTreeMap::new(),
        tasks: BTreeMap::new(),
        redirects: Vec::new(),
        indexes: Vec::new(),
        raw_rows_by_batch: BTreeMap::new(),
    };
    let mut redirect_keys = BTreeSet::new();
    let mut index_keys = BTreeSet::new();

    for (line_number, row) in data_rows.into_iter().skip(1) {
        let fields = row.split('\t').collect::<Vec<_>>();
        if fields.iter().any(|field| field.is_empty()) {
            violations.push(format!(
                "legacy compaction manifest:{}: fields must be nonempty",
                line_number + 1
            ));
            continue;
        }
        match fields.first().copied() {
            Some("batch") if fields.len() == 9 => {
                let counts = fields[5..9]
                    .iter()
                    .map(|field| parse_manifest_count(field, line_number, violations))
                    .collect::<Option<Vec<_>>>();
                let Some(counts) = counts else {
                    continue;
                };
                let batch = LegacyCompactionBatch {
                    id: fields[1].to_owned(),
                    contract_en: PathBuf::from(fields[2]),
                    contract_ja: PathBuf::from(fields[3]),
                    inventory_sha256: fields[4].to_owned(),
                    task_count: counts[0],
                    redirect_count: counts[1],
                    source_count: counts[2],
                    index_count: counts[3],
                };
                if !valid_task_contract_id(&batch.id) {
                    violations.push(format!(
                        "legacy compaction manifest:{}: invalid batch id {:?}",
                        line_number + 1,
                        batch.id
                    ));
                }
                if manifest.batches.insert(batch.id.clone(), batch).is_some() {
                    violations.push(format!(
                        "legacy compaction manifest:{}: duplicate batch id {}",
                        line_number + 1,
                        fields[1]
                    ));
                }
            }
            Some("task") if fields.len() == 5 => {
                let task = LegacyCompactionTask {
                    batch_id: fields[1].to_owned(),
                    id: fields[2].to_owned(),
                    contract_en: PathBuf::from(fields[3]),
                    contract_ja: PathBuf::from(fields[4]),
                };
                if !valid_task_contract_id(&task.id) {
                    violations.push(format!(
                        "legacy compaction manifest:{}: invalid task id {:?}",
                        line_number + 1,
                        task.id
                    ));
                }
                if manifest.tasks.insert(task.id.clone(), task).is_some() {
                    violations.push(format!(
                        "legacy compaction manifest:{}: task id {} belongs to multiple rows",
                        line_number + 1,
                        fields[2]
                    ));
                }
                manifest
                    .raw_rows_by_batch
                    .entry(fields[1].to_owned())
                    .or_default()
                    .push(row.to_owned());
            }
            Some("redirect") if fields.len() == 10 => {
                let Some(heading_level) = parse_manifest_count(fields[5], line_number, violations)
                else {
                    continue;
                };
                let redirect = LegacyCompactionRedirect {
                    batch_id: fields[1].to_owned(),
                    task_id: fields[2].to_owned(),
                    language: fields[3].to_owned(),
                    source_path: PathBuf::from(fields[4]),
                    heading_level,
                    legacy_heading: fields[6].to_owned(),
                    replacement: fields[7].to_owned(),
                    previous_heading: fields[8].to_owned(),
                    next_heading: fields[9].to_owned(),
                };
                let key = (redirect.source_path.clone(), redirect.task_id.clone());
                if !redirect_keys.insert(key) {
                    violations.push(format!(
                        "legacy compaction manifest:{}: duplicate redirect source/task",
                        line_number + 1
                    ));
                }
                manifest
                    .raw_rows_by_batch
                    .entry(redirect.batch_id.clone())
                    .or_default()
                    .push(row.to_owned());
                manifest.redirects.push(redirect);
            }
            Some("index") if fields.len() == 6 => {
                let index = LegacyCompactionIndex {
                    batch_id: fields[1].to_owned(),
                    indexed_id: fields[2].to_owned(),
                    language: fields[3].to_owned(),
                    plan_path: PathBuf::from(fields[4]),
                    row: fields[5].to_owned(),
                };
                let key = (
                    index.batch_id.clone(),
                    index.indexed_id.clone(),
                    index.language.clone(),
                    index.plan_path.clone(),
                );
                if !index_keys.insert(key) {
                    violations.push(format!(
                        "legacy compaction manifest:{}: duplicate logical index identity",
                        line_number + 1
                    ));
                }
                manifest
                    .raw_rows_by_batch
                    .entry(index.batch_id.clone())
                    .or_default()
                    .push(row.to_owned());
                manifest.indexes.push(index);
            }
            Some("schema") => violations.push(format!(
                "legacy compaction manifest:{}: schema record must occur exactly once and first",
                line_number + 1
            )),
            Some(kind) => violations.push(format!(
                "legacy compaction manifest:{}: unknown kind or wrong field count {kind:?}",
                line_number + 1
            )),
            None => unreachable!("nonempty row has a first field"),
        }
    }

    Some(manifest)
}

fn parse_manifest_count(
    field: &str,
    line_number: usize,
    violations: &mut Vec<String>,
) -> Option<usize> {
    match field.parse::<usize>() {
        Ok(value) => Some(value),
        Err(error) => {
            violations.push(format!(
                "legacy compaction manifest:{}: invalid count {field:?}: {error}",
                line_number + 1
            ));
            None
        }
    }
}

fn validate_legacy_manifest_relations(
    workspace: &Path,
    manifest: &LegacyCompactionManifest,
    violations: &mut Vec<String>,
) {
    for batch in manifest.batches.values() {
        validate_manifest_contract_path(workspace, &batch.contract_en, "en", &batch.id, violations);
        validate_manifest_contract_path(workspace, &batch.contract_ja, "ja", &batch.id, violations);
        let raw_rows = manifest
            .raw_rows_by_batch
            .get(&batch.id)
            .cloned()
            .unwrap_or_default();
        let canonical = if raw_rows.is_empty() {
            String::new()
        } else {
            format!("{}\n", raw_rows.join("\n"))
        };
        let observed_hash = sha256_hex(canonical.as_bytes());
        if observed_hash != batch.inventory_sha256 {
            violations.push(format!(
                "legacy compaction batch {}: inventory hash {} != {}",
                batch.id, observed_hash, batch.inventory_sha256
            ));
        }
        let tasks = manifest
            .tasks
            .values()
            .filter(|task| task.batch_id == batch.id)
            .count();
        let redirects = manifest
            .redirects
            .iter()
            .filter(|redirect| redirect.batch_id == batch.id)
            .collect::<Vec<_>>();
        let sources = redirects
            .iter()
            .map(|redirect| &redirect.source_path)
            .collect::<BTreeSet<_>>()
            .len();
        let indexes = manifest
            .indexes
            .iter()
            .filter(|index| index.batch_id == batch.id)
            .count();
        for (label, observed, expected) in [
            ("tasks", tasks, batch.task_count),
            ("redirects", redirects.len(), batch.redirect_count),
            ("source files", sources, batch.source_count),
            ("index rows", indexes, batch.index_count),
        ] {
            if observed != expected {
                violations.push(format!(
                    "legacy compaction batch {}: {label} count {observed} != {expected}",
                    batch.id
                ));
            }
        }
    }

    for batch_id in manifest.raw_rows_by_batch.keys() {
        if !manifest.batches.contains_key(batch_id) {
            violations.push(format!(
                "legacy compaction manifest: rows reference undeclared batch {batch_id}"
            ));
        }
    }
    for task in manifest.tasks.values() {
        if !manifest.batches.contains_key(&task.batch_id) {
            violations.push(format!(
                "legacy compaction task {} references undeclared batch {}",
                task.id, task.batch_id
            ));
        }
        validate_manifest_contract_path(workspace, &task.contract_en, "en", &task.id, violations);
        validate_manifest_contract_path(workspace, &task.contract_ja, "ja", &task.id, violations);
    }
    for redirect in &manifest.redirects {
        validate_manifest_redirect_relation(workspace, manifest, redirect, violations);
    }
    for index in &manifest.indexes {
        validate_manifest_index_relation(workspace, manifest, index, violations);
    }
}

fn validate_manifest_contract_path(
    workspace: &Path,
    relative_path: &Path,
    language: &str,
    id: &str,
    violations: &mut Vec<String>,
) {
    if !manifest_path_within_workspace(workspace, relative_path)
        || !relative_path.starts_with(format!("doc/design/task_contracts/{language}"))
        || relative_path.file_stem().and_then(|stem| stem.to_str()) != Some(id)
    {
        violations.push(format!(
            "legacy compaction {id}: invalid {language} contract path {}",
            relative_path.display()
        ));
        return;
    }
    let path = workspace.join(relative_path);
    if !path.is_file() {
        violations.push(format!(
            "legacy compaction {id}: missing {language} contract {}",
            path.display()
        ));
        return;
    }
    let document = read_to_string(&path);
    let title = document.lines().next().unwrap_or_default();
    if !title.starts_with(&format!("# Task {id}:")) {
        violations.push(format!(
            "{}: contract title must identify Task {id}",
            path.display()
        ));
    }
}

fn safe_manifest_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        && path.starts_with("doc/design")
        && !path.to_string_lossy().contains('\\')
}

fn manifest_path_within_workspace(workspace: &Path, relative_path: &Path) -> bool {
    if !safe_manifest_relative_path(relative_path) {
        return false;
    }
    let Ok(canonical_workspace) = fs::canonicalize(workspace) else {
        return false;
    };
    fs::canonicalize(workspace.join(relative_path))
        .is_ok_and(|path| path.starts_with(canonical_workspace))
}

fn validate_manifest_redirect_relation(
    workspace: &Path,
    manifest: &LegacyCompactionManifest,
    redirect: &LegacyCompactionRedirect,
    violations: &mut Vec<String>,
) {
    let Some(task) = manifest.tasks.get(&redirect.task_id) else {
        violations.push(format!(
            "legacy redirect {} references undeclared task {}",
            redirect.source_path.display(),
            redirect.task_id
        ));
        return;
    };
    if task.batch_id != redirect.batch_id {
        violations.push(format!(
            "legacy redirect {} binds task {} to wrong batch {}",
            redirect.source_path.display(),
            task.id,
            redirect.batch_id
        ));
    }
    if !matches!(redirect.language.as_str(), "en" | "ja")
        || manifest_path_language(&redirect.source_path) != Some(redirect.language.as_str())
    {
        violations.push(format!(
            "legacy redirect {} has mismatched language {}",
            redirect.source_path.display(),
            redirect.language
        ));
    }
    if !manifest_path_within_workspace(workspace, &redirect.source_path)
        || redirect
            .source_path
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("md")
    {
        violations.push(format!(
            "legacy redirect has unsafe source path {}",
            redirect.source_path.display()
        ));
    }
    if !(2..=6).contains(&redirect.heading_level)
        || atx_heading_level(&redirect.legacy_heading) != Some(redirect.heading_level)
    {
        violations.push(format!(
            "legacy redirect {} has invalid level/heading {:?}",
            redirect.source_path.display(),
            redirect.legacy_heading
        ));
    }

    let contract = if redirect.language == "en" {
        &task.contract_en
    } else {
        &task.contract_ja
    };
    let punctuation = if redirect.language == "en" {
        "."
    } else {
        "。"
    };
    let prefix = format!(
        "Completion evidence: [central Task-{} historical contract](",
        task.id
    );
    let suffix = format!("){}", punctuation);
    let Some(destination) = redirect
        .replacement
        .strip_prefix(&prefix)
        .and_then(|line| line.strip_suffix(&suffix))
    else {
        violations.push(format!(
            "legacy redirect {} is outside the reserved grammar: {:?}",
            redirect.source_path.display(),
            redirect.replacement
        ));
        return;
    };
    if !supported_relative_markdown_destination(destination) {
        violations.push(format!(
            "legacy redirect {} must use a relative Markdown path",
            redirect.source_path.display()
        ));
        return;
    }
    if destination.split_once('#').map(|(_, fragment)| fragment) != Some("completion-evidence") {
        violations.push(format!(
            "legacy redirect {} must target #completion-evidence",
            redirect.source_path.display()
        ));
    }
    let source = workspace.join(&redirect.source_path);
    let declared_contract = workspace.join(contract);
    let resolved =
        markdown_target_path(&source, destination).and_then(|path| fs::canonicalize(path).ok());
    let declared = fs::canonicalize(&declared_contract).ok();
    if resolved.is_none() || resolved != declared {
        violations.push(format!(
            "legacy redirect {} must target declared {} contract {}",
            redirect.source_path.display(),
            redirect.language,
            declared_contract.display()
        ));
    }
}

fn validate_manifest_index_relation(
    workspace: &Path,
    manifest: &LegacyCompactionManifest,
    index: &LegacyCompactionIndex,
    violations: &mut Vec<String>,
) {
    let Some(batch) = manifest.batches.get(&index.batch_id) else {
        violations.push(format!(
            "legacy index {} references undeclared batch {}",
            index.plan_path.display(),
            index.batch_id
        ));
        return;
    };
    let contract = if index.indexed_id == batch.id {
        if index.language == "en" {
            &batch.contract_en
        } else {
            &batch.contract_ja
        }
    } else {
        let Some(task) = manifest.tasks.get(&index.indexed_id) else {
            violations.push(format!(
                "legacy index {} references undeclared task {}",
                index.plan_path.display(),
                index.indexed_id
            ));
            return;
        };
        if task.batch_id != batch.id {
            violations.push(format!(
                "legacy index {} binds task {} to wrong batch {}",
                index.plan_path.display(),
                task.id,
                batch.id
            ));
        }
        if index.language == "en" {
            &task.contract_en
        } else {
            &task.contract_ja
        }
    };
    if !matches!(index.language.as_str(), "en" | "ja")
        || manifest_path_language(&index.plan_path) != Some(index.language.as_str())
        || !manifest_path_within_workspace(workspace, &index.plan_path)
        || index.plan_path.file_name().and_then(|name| name.to_str()) != Some("00.crate_plan.md")
    {
        violations.push(format!(
            "legacy index has invalid plan language/path {} ({})",
            index.plan_path.display(),
            index.language
        ));
    }
    let label = index.language.to_ascii_uppercase();
    let prefix = format!("| {} | [{label} contract](", index.indexed_id);
    let Some(destination) = index
        .row
        .strip_prefix(&prefix)
        .and_then(|row| row.strip_suffix(") |"))
    else {
        violations.push(format!(
            "legacy index {} has invalid exact row {:?}",
            index.plan_path.display(),
            index.row
        ));
        return;
    };
    let plan = workspace.join(&index.plan_path);
    let resolved =
        markdown_target_path(&plan, destination).and_then(|path| fs::canonicalize(path).ok());
    let declared = fs::canonicalize(workspace.join(contract)).ok();
    if resolved.is_none() || resolved != declared {
        violations.push(format!(
            "legacy index {} row must target declared contract {}",
            index.plan_path.display(),
            contract.display()
        ));
    }
}

fn manifest_path_language(path: &Path) -> Option<&'static str> {
    for component in path.components() {
        match component.as_os_str().to_str() {
            Some("en") => return Some("en"),
            Some("ja") => return Some("ja"),
            _ => {}
        }
    }
    path.starts_with("doc/design").then_some("en")
}

fn validate_legacy_manifest_documents(
    workspace: &Path,
    manifest: &LegacyCompactionManifest,
    violations: &mut Vec<String>,
) {
    let design_root = workspace.join("doc/design");
    let mut markdown_paths = Vec::new();
    collect_markdown_files(&design_root, &mut markdown_paths);
    markdown_paths.sort();

    let expected = manifest
        .redirects
        .iter()
        .map(|redirect| {
            (
                workspace.join(&redirect.source_path),
                redirect.replacement.clone(),
            )
        })
        .collect::<BTreeSet<_>>();
    if expected.len() != manifest.redirects.len() {
        violations.push("legacy compaction manifest has duplicate expanded redirects".to_owned());
    }
    let mut forbidden_by_path = BTreeMap::<PathBuf, BTreeSet<String>>::new();
    for redirect in &manifest.redirects {
        forbidden_by_path
            .entry(workspace.join(&redirect.source_path))
            .or_default()
            .insert(redirect.legacy_heading.clone());
    }
    let mut documents = BTreeMap::new();
    let mut heading_slugs = BTreeMap::new();
    for path in markdown_paths {
        let document = read_to_string(&path);
        if let Ok(canonical_path) = fs::canonicalize(&path) {
            heading_slugs.insert(canonical_path, markdown_heading_slugs(&document));
        }
        documents.insert(path, document);
    }

    let mut expected_by_path = BTreeMap::<PathBuf, BTreeSet<String>>::new();
    for (path, line) in &expected {
        expected_by_path
            .entry(path.clone())
            .or_default()
            .insert(line.clone());
        validate_cached_manifest_markdown_links(path, line, &heading_slugs, violations);
    }
    for (path, document) in &documents {
        let forbidden = forbidden_by_path.get(path).cloned().unwrap_or_default();
        let expected_lines = expected_by_path.get(path).cloned().unwrap_or_default();
        validate_legacy_redirect_document_evidence(
            path,
            document,
            &forbidden,
            &expected_lines,
            violations,
        );
    }
    for (path, expected_lines) in &expected_by_path {
        if !documents.contains_key(path) {
            let forbidden = forbidden_by_path.get(path).cloned().unwrap_or_default();
            validate_legacy_redirect_document_evidence(
                path,
                "",
                &forbidden,
                expected_lines,
                violations,
            );
        }
    }

    for redirect in &manifest.redirects {
        let path = workspace.join(&redirect.source_path);
        let Some(document) = documents.get(&path) else {
            violations.push(format!("{}: redirect source is missing", path.display()));
            continue;
        };
        match legacy_redirect_anchors(document, &redirect.replacement, redirect.heading_level) {
            Ok((previous, next)) => {
                if previous != redirect.previous_heading || next != redirect.next_heading {
                    violations.push(format!(
                        "{}: redirect anchors ({previous:?}, {next:?}) != ({:?}, {:?})",
                        path.display(),
                        redirect.previous_heading,
                        redirect.next_heading
                    ));
                }
            }
            Err(error) => violations.push(format!("{}: {error}", path.display())),
        }
    }

    for index in &manifest.indexes {
        let path = workspace.join(&index.plan_path);
        let Some(document) = documents.get(&path) else {
            violations.push(format!("{}: indexed plan is missing", path.display()));
            continue;
        };
        let Some(task_index) = markdown_h2_section(document, "Task Index") else {
            violations.push(format!("{}: missing Task Index section", path.display()));
            continue;
        };
        let count = visible_markdown_lines(task_index)
            .iter()
            .filter(|(_, line)| *line == index.row)
            .count();
        if count != 1 {
            violations.push(format!(
                "{}: Task Index row {:?} must occur exactly once, found {count}",
                path.display(),
                index.row
            ));
        }
    }
}

fn validate_legacy_redirect_document_evidence(
    path: &Path,
    document: &str,
    forbidden: &BTreeSet<String>,
    expected: &BTreeSet<String>,
    violations: &mut Vec<String>,
) {
    let mut observed = BTreeMap::new();
    for (line_number, line) in visible_markdown_lines(document) {
        if forbidden.contains(line) {
            violations.push(format!(
                "{}:{}: forbidden legacy completion heading {:?}",
                path.display(),
                line_number + 1,
                line
            ));
        }
        if reserved_completion_redirect(line) {
            *observed.entry(line.to_owned()).or_insert(0_usize) += 1;
        }
    }
    for line in expected {
        let count = observed.get(line).copied().unwrap_or(0);
        if count != 1 {
            violations.push(format!(
                "{}: manifest redirect {line:?} must occur exactly once, found {count}",
                path.display()
            ));
        }
    }
    for line in observed.keys() {
        if !expected.contains(line) {
            violations.push(format!(
                "{}: unexpected central historical-contract redirect {line:?}",
                path.display()
            ));
        }
    }
}

fn validate_cached_manifest_markdown_links(
    source_path: &Path,
    document: &str,
    heading_slugs: &BTreeMap<PathBuf, BTreeSet<String>>,
    violations: &mut Vec<String>,
) {
    for destination in markdown_link_destinations(document) {
        let Some(target_path) = markdown_target_path(source_path, &destination) else {
            continue;
        };
        let Ok(canonical_target) = fs::canonicalize(&target_path) else {
            violations.push(format!(
                "{}: local Markdown target does not exist: {destination}",
                source_path.display()
            ));
            continue;
        };
        let Some((_, fragment)) = destination.split_once('#') else {
            continue;
        };
        if fragment.is_empty() {
            continue;
        }
        if !heading_slugs
            .get(&canonical_target)
            .is_some_and(|slugs| slugs.contains(fragment))
        {
            violations.push(format!(
                "{}: fragment #{fragment} does not exist in {}",
                source_path.display(),
                target_path.display()
            ));
        }
    }
}

fn visible_markdown_lines(document: &str) -> Vec<(usize, &str)> {
    let mut visible = Vec::new();
    let mut fence = None;
    for (line_number, line) in document.lines().enumerate() {
        if update_fence_state(line.trim_start(), &mut fence) {
            continue;
        }
        if fence.is_none() {
            visible.push((line_number, line));
        }
    }
    visible
}

fn reserved_completion_redirect(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("Completion evidence: [central Task-") else {
        return false;
    };
    let Some((task_id, destination)) = rest.split_once(" historical contract](") else {
        return false;
    };
    if !valid_task_contract_id(task_id) {
        return false;
    }
    let destination = destination
        .strip_suffix(").")
        .or_else(|| destination.strip_suffix(")。"));
    destination.is_some_and(|destination| {
        destination.ends_with(".md#completion-evidence")
            && supported_relative_markdown_destination(destination)
    })
}

fn supported_relative_markdown_destination(destination: &str) -> bool {
    let target = destination
        .split_once('#')
        .map_or(destination, |(target, _)| target);
    !target.is_empty()
        && !Path::new(target).is_absolute()
        && !destination.contains(['\\', '\r', '\n', '\t', ' '])
        && Path::new(target)
            .extension()
            .and_then(|extension| extension.to_str())
            == Some("md")
        && markdown_target_path(Path::new("source.md"), destination).is_some()
}

fn legacy_redirect_anchors(
    document: &str,
    replacement: &str,
    legacy_level: usize,
) -> Result<(String, String), String> {
    let visible = visible_markdown_lines(document);
    let matches = visible
        .iter()
        .enumerate()
        .filter(|(_, (_, line))| *line == replacement)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(format!(
            "redirect {replacement:?} must be unique outside code, found {}",
            matches.len()
        ));
    }
    let redirect_index = matches[0];
    let previous = visible[..redirect_index]
        .iter()
        .rev()
        .find_map(|(_, line)| {
            atx_heading_level(line)
                .filter(|level| *level <= legacy_level)
                .map(|_| (*line).to_owned())
        })
        .unwrap_or_else(|| "BOF".to_owned());
    let next = visible[redirect_index + 1..]
        .iter()
        .find_map(|(_, line)| {
            atx_heading_level(line)
                .filter(|level| *level <= legacy_level)
                .map(|_| (*line).to_owned())
        })
        .unwrap_or_else(|| "EOF".to_owned());
    Ok((previous, next))
}

fn atx_heading_level(line: &str) -> Option<usize> {
    let level = line.bytes().take_while(|byte| *byte == b'#').count();
    (1..=6)
        .contains(&level)
        .then(|| line.as_bytes().get(level))
        .flatten()
        .is_some_and(|byte| *byte == b' ')
        .then_some(level)
}

fn assert_legacy_heading_boundary_vectors() {
    for level in 2..=6 {
        let marker = "#".repeat(level);
        let document = format!("{marker} Previous\nredirect\n{marker} Next\n");
        assert_eq!(
            legacy_redirect_anchors(&document, "redirect", level),
            Ok((format!("{marker} Previous"), format!("{marker} Next")))
        );
    }
    assert_eq!(
        legacy_redirect_anchors("redirect\n", "redirect", 2),
        Ok(("BOF".to_owned(), "EOF".to_owned()))
    );
    let lower_level = "## Previous\n### Nested\nredirect\n### Still nested\n## Next\n";
    assert_eq!(
        legacy_redirect_anchors(lower_level, "redirect", 2),
        Ok(("## Previous".to_owned(), "## Next".to_owned()))
    );
    let higher_level = "### Previous\nredirect\n# Higher\n";
    assert_eq!(
        legacy_redirect_anchors(higher_level, "redirect", 3),
        Ok(("### Previous".to_owned(), "# Higher".to_owned()))
    );
    let fences =
        "## Same\n## Same\n```md\n## Ignored\n```\nredirect\n~~~md\n## Ignored too\n~~~\n## Next\n";
    assert_eq!(
        legacy_redirect_anchors(fences, "redirect", 2),
        Ok(("## Same".to_owned(), "## Next".to_owned()))
    );
}

fn assert_legacy_document_evidence_vectors() {
    let path = Path::new("fixture.md");
    let forbidden = BTreeSet::from(["## Legacy completion".to_owned()]);
    let redirect = "Completion evidence: [central Task-T historical contract](contract.md#completion-evidence).";
    let unexpected =
        "Completion evidence: [central Task-U historical contract](other.md#completion-evidence).";
    let expected = BTreeSet::from([redirect.to_owned()]);

    let accepted = format!(
        "{redirect}\nordinary prose mentions {unexpected}\n```md\n## Legacy completion\n{unexpected}\n```\n"
    );
    let mut violations = Vec::new();
    validate_legacy_redirect_document_evidence(
        path,
        &accepted,
        &forbidden,
        &expected,
        &mut violations,
    );
    assert!(
        violations.is_empty(),
        "ordinary prose and fenced evidence must be ignored: {violations:?}"
    );

    for (document, diagnostic) in [
        (
            "## Legacy completion\n",
            "forbidden legacy completion heading",
        ),
        ("", "must occur exactly once, found 0"),
        (
            &format!("{redirect}\n{redirect}\n"),
            "must occur exactly once, found 2",
        ),
        (
            &format!("{redirect}\n{unexpected}\n"),
            "unexpected central historical-contract redirect",
        ),
    ] {
        let mut violations = Vec::new();
        validate_legacy_redirect_document_evidence(
            path,
            document,
            &forbidden,
            &expected,
            &mut violations,
        );
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains(diagnostic)),
            "document mutation must report {diagnostic:?}, got {violations:?}"
        );
    }
}

fn assert_legacy_path_scoped_heading_vectors() {
    let workspace = std::env::temp_dir().join(format!(
        "mizar_test_legacy_path_scope_{}",
        std::process::id()
    ));
    remove_dir_if_exists(&workspace);

    let source_path = PathBuf::from("doc/design/component/en/source.md");
    let unrelated_path = PathBuf::from("doc/design/component/en/unrelated.md");
    let contract_path = PathBuf::from("doc/design/task_contracts/en/T.md");
    let legacy_heading = "## Legacy completion";
    let replacement = "Completion evidence: [central Task-T historical contract](../../task_contracts/en/T.md#completion-evidence).";
    create_dir(&workspace.join("doc/design/component/en"));
    create_dir(&workspace.join("doc/design/task_contracts/en"));
    write_test_file(
        &workspace.join(&contract_path),
        "# Task T: Historical record\n\n## Completion Evidence\n",
    );
    write_test_file(
        &workspace.join(&source_path),
        &format!("## Previous\n{replacement}\n## Next\n"),
    );
    write_test_file(
        &workspace.join(&unrelated_path),
        &format!("{legacy_heading}\n"),
    );

    let manifest = LegacyCompactionManifest {
        batches: BTreeMap::new(),
        tasks: BTreeMap::new(),
        redirects: vec![LegacyCompactionRedirect {
            batch_id: "BATCH".to_owned(),
            task_id: "T".to_owned(),
            language: "en".to_owned(),
            source_path: source_path.clone(),
            heading_level: 2,
            legacy_heading: legacy_heading.to_owned(),
            replacement: replacement.to_owned(),
            previous_heading: "## Previous".to_owned(),
            next_heading: "## Next".to_owned(),
        }],
        indexes: Vec::new(),
        raw_rows_by_batch: BTreeMap::new(),
    };

    let mut violations = Vec::new();
    validate_legacy_manifest_documents(&workspace, &manifest, &mut violations);
    assert!(
        violations.is_empty(),
        "an unrelated document may retain the same heading text: {violations:?}"
    );

    write_test_file(
        &workspace.join(&source_path),
        &format!("## Previous\n{replacement}\n## Next\n{legacy_heading}\n"),
    );
    let mut violations = Vec::new();
    validate_legacy_manifest_documents(&workspace, &manifest, &mut violations);
    assert!(
        violations.iter().any(|violation| {
            violation.contains("forbidden legacy completion heading")
                && violation.contains(&workspace.join(&source_path).display().to_string())
        }),
        "the declared source must reject its legacy heading: {violations:?}"
    );

    remove_dir_if_exists(&workspace);
}

fn assert_legacy_manifest_mutation_vectors(workspace: &Path, manifest: &str) {
    let zero_hash = "0".repeat(64);
    let mutations = vec![
        (
            mutate_first_manifest_field(manifest, "schema", 1, |_| "2".to_owned()),
            "first data row must be schema<TAB>1",
        ),
        (
            mutate_first_manifest_field(manifest, "batch", 4, |_| zero_hash),
            "inventory hash",
        ),
        (
            mutate_first_manifest_field(manifest, "batch", 5, |_| "999".to_owned()),
            "tasks count",
        ),
        (
            mutate_first_manifest_field(manifest, "redirect", 4, |_| "/outside.md".to_owned()),
            "unsafe source path",
        ),
        (
            mutate_first_manifest_field(manifest, "redirect", 6, |_| {
                "# Wrong legacy level".to_owned()
            }),
            "invalid level/heading",
        ),
        (
            mutate_first_manifest_field(manifest, "redirect", 7, |_| {
                "not a reserved completion redirect".to_owned()
            }),
            "outside the reserved grammar",
        ),
        (
            mutate_first_manifest_field(manifest, "redirect", 7, |line| {
                let (prefix, destination) = line
                    .split_once("](")
                    .expect("reserved redirect has a destination");
                let (_, suffix) = destination
                    .split_once("#completion-evidence")
                    .expect("reserved redirect has the completion fragment");
                format!("{prefix}](missing.md#completion-evidence{suffix}")
            }),
            "must target declared",
        ),
        (
            mutate_first_manifest_field(manifest, "redirect", 7, |line| {
                line.replace("#completion-evidence", "#missing-fragment")
            }),
            "must target #completion-evidence",
        ),
        (
            mutate_first_manifest_field(manifest, "redirect", 7, |line| {
                line.replacen("](", "](/", 1)
            }),
            "must use a relative Markdown path",
        ),
        (
            mutate_first_manifest_field(manifest, "redirect", 8, |_| {
                "## Missing previous anchor".to_owned()
            }),
            "redirect anchors",
        ),
        (
            mutate_first_manifest_field(manifest, "index", 5, |_| {
                "| wrong | [EN contract](missing.md) |".to_owned()
            }),
            "invalid exact row",
        ),
        (
            swap_first_two_manifest_rows(manifest, "index"),
            "strictly byte-sorted",
        ),
        (
            duplicate_first_manifest_row(manifest, "batch"),
            "duplicate batch id",
        ),
        (
            duplicate_first_manifest_row(manifest, "task"),
            "belongs to multiple rows",
        ),
        (
            duplicate_first_manifest_row_with(manifest, "index", |duplicate| {
                duplicate.replace("](", "](./")
            }),
            "duplicate logical index identity",
        ),
        (
            append_manifest_row(manifest, "unknown\tfield"),
            "unknown kind or wrong field count",
        ),
        (
            mutate_first_manifest_row(manifest, "task", |row| format!("{row}\textra")),
            "unknown kind or wrong field count",
        ),
        (
            append_manifest_row(manifest, "schema\t1"),
            "schema record must occur exactly once and first",
        ),
        (
            manifest.replacen('\n', "\r\n", 1),
            "must use LF line endings",
        ),
        (
            mutate_first_manifest_field(manifest, "redirect", 1, |_| "UNDECLARED-BATCH".to_owned()),
            "wrong batch",
        ),
        (
            mutate_first_manifest_field(manifest, "redirect", 3, |_| "ja".to_owned()),
            "mismatched language",
        ),
    ];
    for (mutation, expected) in mutations {
        let violations = legacy_compaction_manifest_violations(workspace, &mutation);
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains(expected)),
            "legacy manifest mutation must report {expected:?}, got:\n{}",
            violations.join("\n")
        );
    }
}

fn mutate_first_manifest_row(
    document: &str,
    kind: &str,
    mutate: impl FnOnce(&str) -> String,
) -> String {
    let mut mutation = Some(mutate);
    let mut changed = false;
    let mut output = String::with_capacity(document.len());
    for segment in document.split_inclusive('\n') {
        let (line, newline) = segment
            .strip_suffix('\n')
            .map_or((segment, ""), |line| (line, "\n"));
        if !changed && line.split('\t').next() == Some(kind) {
            output.push_str(&mutation.take().expect("mutation is available")(line));
            changed = true;
        } else {
            output.push_str(line);
        }
        output.push_str(newline);
    }
    assert!(changed, "manifest must contain a {kind} row");
    output
}

fn duplicate_first_manifest_row(document: &str, kind: &str) -> String {
    mutate_first_manifest_row(document, kind, |row| format!("{row}\n{row}"))
}

fn duplicate_first_manifest_row_with(
    document: &str,
    kind: &str,
    mutate_duplicate: impl FnOnce(&str) -> String,
) -> String {
    mutate_first_manifest_row(document, kind, |row| {
        format!("{row}\n{}", mutate_duplicate(row))
    })
}

fn append_manifest_row(document: &str, row: &str) -> String {
    format!(
        "{}{row}\n",
        document.trim_end_matches('\n').to_owned() + "\n"
    )
}

fn swap_first_two_manifest_rows(document: &str, kind: &str) -> String {
    let mut lines = document.lines().map(str::to_owned).collect::<Vec<_>>();
    let positions = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.split('\t').next() == Some(kind))
        .map(|(index, _)| index)
        .take(2)
        .collect::<Vec<_>>();
    assert_eq!(positions.len(), 2, "manifest must contain two {kind} rows");
    lines.swap(positions[0], positions[1]);
    format!("{}\n", lines.join("\n"))
}

fn mutate_first_manifest_field(
    document: &str,
    kind: &str,
    field_index: usize,
    mutate: impl FnOnce(&str) -> String,
) -> String {
    let mut mutation = Some(mutate);
    let mut changed = false;
    let mut output = String::with_capacity(document.len());
    for segment in document.split_inclusive('\n') {
        let (line, newline) = segment
            .strip_suffix('\n')
            .map_or((segment, ""), |line| (line, "\n"));
        let mut fields = line.split('\t').map(str::to_owned).collect::<Vec<_>>();
        if !changed && fields.first().is_some_and(|field| field == kind) {
            let value = fields
                .get(field_index)
                .unwrap_or_else(|| panic!("{kind} field {field_index} must exist"));
            fields[field_index] = mutation.take().expect("mutation is available")(value);
            changed = true;
        }
        output.push_str(&fields.join("\t"));
        output.push_str(newline);
    }
    assert!(changed, "manifest must contain a {kind} row");
    output
}

fn sha256_hex(input: &[u8]) -> String {
    const INITIAL: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const ROUND: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let bit_length = (input.len() as u64).wrapping_mul(8);
    let mut padded = input.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_length.to_be_bytes());

    let mut hash = INITIAL;
    for block in padded.chunks_exact(64) {
        let mut words = [0_u32; 64];
        for (index, word) in words[..16].iter_mut().enumerate() {
            let offset = index * 4;
            *word = u32::from_be_bytes([
                block[offset],
                block[offset + 1],
                block[offset + 2],
                block[offset + 3],
            ]);
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = hash;
        for index in 0..64 {
            let upper = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choose = (e & f) ^ (!e & g);
            let temporary_one = h
                .wrapping_add(upper)
                .wrapping_add(choose)
                .wrapping_add(ROUND[index])
                .wrapping_add(words[index]);
            let lower = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temporary_two = lower.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temporary_one);
            d = c;
            c = b;
            b = a;
            a = temporary_one.wrapping_add(temporary_two);
        }
        for (slot, value) in hash.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }

    hash.iter()
        .map(|word| format!("{word:08x}"))
        .collect::<Vec<_>>()
        .join("")
}

fn markdown_h2_section<'a>(document: &'a str, heading: &str) -> Option<&'a str> {
    let marker = format!("## {heading}");
    let mut fence = None;
    let mut start = None;
    let mut offset = 0_usize;

    for segment in document.split_inclusive('\n') {
        let line = segment.strip_suffix('\n').unwrap_or(segment);
        let trimmed = line.trim_start();
        if update_fence_state(trimmed, &mut fence) || fence.is_some() {
            offset += segment.len();
            continue;
        }
        if start.is_none() && trimmed == marker {
            start = Some(offset);
        } else if start.is_some() && atx_heading_level(trimmed) == Some(2) {
            return start.map(|start| &document[start..offset]);
        }
        offset += segment.len();
    }

    start.map(|start| &document[start..])
}

fn markdown_heading_slugs(document: &str) -> BTreeSet<String> {
    let mut slugs = BTreeSet::new();
    let mut duplicate_counts = BTreeMap::new();
    let mut fence = None;

    for line in document.lines() {
        let trimmed = line.trim_start();
        if update_fence_state(trimmed, &mut fence) {
            continue;
        }
        if fence.is_some() {
            continue;
        }

        let level = trimmed.bytes().take_while(|byte| *byte == b'#').count();
        if !(1..=6).contains(&level) || trimmed.as_bytes().get(level) != Some(&b' ') {
            continue;
        }
        let heading = trimmed[level + 1..].trim().trim_end_matches('#').trim();
        let base = github_heading_slug(heading);
        let count = duplicate_counts.entry(base.clone()).or_insert(0_usize);
        let slug = if *count == 0 {
            base
        } else {
            format!("{base}-{count}")
        };
        *count += 1;
        slugs.insert(slug);
    }

    slugs
}

fn github_heading_slug(heading: &str) -> String {
    let mut slug = String::new();
    for character in heading.chars().flat_map(char::to_lowercase) {
        if character.is_alphanumeric() || matches!(character, '-' | '_') {
            slug.push(character);
        } else if character.is_whitespace() {
            slug.push('-');
        }
    }
    slug
}

fn read_to_string(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn create_dir(path: &Path) {
    fs::create_dir_all(path)
        .unwrap_or_else(|error| panic!("failed to create {}: {error}", path.display()));
}

fn write_test_file(path: &Path, contents: &str) {
    fs::write(path, contents)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", path.display()));
}

fn remove_dir_if_exists(path: &Path) {
    if path.exists() {
        fs::remove_dir_all(path)
            .unwrap_or_else(|error| panic!("failed to remove {}: {error}", path.display()));
    }
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> PathBuf {
    let root = crate_root();
    root.parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("mizar-test crate must live under crates/")
}
