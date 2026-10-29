use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Debug)]
struct LegacyCompactionManifest {
    batches: BTreeMap<String, LegacyCompactionBatch>,
    tasks: BTreeMap<String, LegacyCompactionTask>,
    task_refs: BTreeSet<(String, String)>,
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
fn syntax_smoke_ledger_matches_frozen_rows_and_sha256() {
    let workspace = workspace_root();
    let path = workspace.join("tests/coverage/syntax_smoke_expected_rejections.tsv");
    let actual = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read `{}`: {error}", path.display()));
    let expected = r#"case_id	source	syntax_diagnostic_codes	owner
fail_template_func_actual_result_widening_001	tests/miz/fail/templates/fail_template_func_actual_result_widening_001.miz	malformed_formula_expression,malformed_formula_expression,missing_semicolon	template-spec-decision
fail_template_inference_requires_explicit_qua_view_001	tests/miz/fail/templates/fail_template_inference_requires_explicit_qua_view_001.miz	malformed_term_expression,malformed_term_expression,malformed_type_expression,malformed_type_expression,malformed_term_expression,malformed_term_expression,malformed_term_expression,malformed_term_expression,malformed_term_expression,malformed_type_expression,malformed_type_expression,malformed_term_expression,malformed_term_expression,malformed_term_expression,malformed_term_expression,malformed_term_expression,malformed_type_expression,malformed_type_expression,malformed_term_expression,malformed_term_expression,malformed_type_expression,malformed_type_expression	template-spec-decision
fail_template_qua_view_attribute_leak_001	tests/miz/fail/templates/fail_template_qua_view_attribute_leak_001.miz	malformed_term_expression,malformed_term_expression,malformed_type_expression,malformed_type_expression,malformed_term_expression,malformed_term_expression,malformed_term_expression,malformed_term_expression,malformed_term_expression,malformed_type_expression,malformed_type_expression,malformed_term_expression,malformed_term_expression,malformed_term_expression,malformed_term_expression,malformed_term_expression,malformed_type_expression,malformed_type_expression,malformed_term_expression,malformed_term_expression,malformed_type_expression,malformed_type_expression,malformed_formula_expression,malformed_formula_expression,missing_semicolon,malformed_formula_expression,malformed_term_expression,malformed_term_expression,missing_semicolon,malformed_formula_expression,malformed_term_expression,malformed_term_expression,unexpected_top_level_token,unexpected_top_level_token	template-spec-decision
fail_template_type_actual_missing_existential_001	tests/miz/fail/templates/fail_template_type_actual_missing_existential_001.miz	malformed_formula_expression,malformed_formula_expression,missing_semicolon,unexpected_top_level_token,unexpected_top_level_token	template-spec-decision
fail_type_elaboration_local_attribute_forward_reference_gap_001	tests/miz/fail/types/fail_type_elaboration_local_attribute_forward_reference_gap_001.miz	malformed_type_expression	task-77
fail_type_elaboration_local_mode_forward_reference_gap_001	tests/miz/fail/types/fail_type_elaboration_local_mode_forward_reference_gap_001.miz	malformed_type_expression	task-75
fail_type_elaboration_local_structure_forward_reference_gap_001	tests/miz/fail/types/fail_type_elaboration_local_structure_forward_reference_gap_001.miz	malformed_type_expression	task-76
"#;

    assert_eq!(actual, expected, "syntax-smoke ledger rows drifted");
    assert_eq!(
        sha256_hex(actual.as_bytes()),
        "54bd225e86fffde5c3b114dcfd66bb5bfd18683cc96477078486ddfd9496b019"
    );
}

#[test]
fn documentation_volume_stays_within_baseline() {
    assert_doc_volume_matcher_vectors();

    let workspace = workspace_root();
    let ledger_path = workspace.join(DOC_VOLUME_BASELINE_PATH);
    let ledger = read_to_string(&ledger_path);
    let mut violations = Vec::new();
    let baseline = parse_doc_volume_baseline(&ledger, &mut violations);

    let design_root = workspace.join("doc/design");
    let mut files = Vec::new();
    collect_markdown_files(&design_root, &mut files);
    let mut documents = BTreeMap::new();
    for path in files {
        let relative = workspace_relative_slash_path(&workspace, &path);
        if relative.starts_with("doc/design/archive/") {
            continue;
        }
        documents.insert(relative, read_to_string(&path));
    }

    // 1. Ceremony tokens: evaluation scores, gate tallies, model routing, and
    //    digests belong in commit bodies, never in live design documents.
    for (relative, document) in &documents {
        if relative == DOC_VOLUME_PROTOCOL_PATH {
            continue;
        }
        let actual = ceremony_token_count(document);
        check_doc_volume_ratchet(
            &baseline,
            "ceremony_tokens",
            relative,
            actual,
            0,
            "ceremony tokens (scores, gate tallies, xhigh/model names, 64-hex digests); \
             move them to the commit body",
            &mut violations,
        );
    }

    // 2. Task contracts: new contracts stay at or below 60 lines per language.
    for (relative, document) in &documents {
        if !relative.starts_with("doc/design/task_contracts/") {
            continue;
        }
        let actual = document.lines().count();
        check_doc_volume_ratchet(
            &baseline,
            "contract_lines",
            relative,
            actual,
            DOC_VOLUME_CONTRACT_LINE_LIMIT,
            "lines; a task contract records decisions and boundaries only",
            &mut violations,
        );
    }

    // 3. Contract fan-out: a contract is linked from the roadmap, crate plans,
    //    crate todos, and the coverage audit only; module documents own their
    //    own facts and never point back at orchestration records.
    let contract_ids: BTreeSet<String> = documents
        .keys()
        .filter_map(|relative| relative.strip_prefix("doc/design/task_contracts/en/"))
        .filter_map(|rest| rest.strip_suffix(".md"))
        .map(|rest| rest.rsplit('/').next().unwrap_or(rest).to_owned())
        .collect();
    for id in &contract_ids {
        let needles = [
            format!("task_contracts/en/{id}.md"),
            format!("task_contracts/ja/{id}.md"),
        ];
        let actual = documents
            .iter()
            .filter(|(relative, _)| !relative.starts_with("doc/design/task_contracts/"))
            .filter(|(relative, _)| !contract_fanout_allowed(relative))
            .filter(|(_, document)| needles.iter().any(|needle| document.contains(needle)))
            .count();
        check_doc_volume_ratchet(
            &baseline,
            "contract_fanout",
            id,
            actual,
            0,
            "module documents link this contract; link it from todo.md, crate plans, \
             crate todos, or the coverage audit only",
            &mut violations,
        );
    }

    // 4. Stale baseline rows must be removed so the ratchet only tightens.
    for (kind, key) in baseline.keys() {
        let exists = match kind.as_str() {
            "contract_fanout" => contract_ids.contains(key),
            _ => documents.contains_key(key),
        };
        if !exists {
            violations.push(format!(
                "{DOC_VOLUME_BASELINE_PATH}: stale row {kind}\t{key}; remove it"
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "documentation volume policy violations (baseline rows in \
         {DOC_VOLUME_BASELINE_PATH} may only be lowered or removed; raising one requires \
         explicit user approval recorded in the commit body):\n{}",
        violations.join("\n")
    );
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
    assert_legacy_task_reference_vectors();
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
    let plan_backlinks = crate_plan_backlinks(&workspace.join("doc/design"));
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

            validate_crate_plan_backlinks(
                &plan_backlinks,
                &path,
                &en_root.join(relative_path),
                &links,
                &mut violations,
            );
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
fn task_contract_backlinks_distinguish_consumers_and_existing_todo_owners() {
    let workspace = std::env::temp_dir().join(format!(
        "mizar_test_contract_backlinks_{}",
        std::process::id()
    ));
    let canonical_contract = workspace.join("doc/design/task_contracts/en/nested/T.md");

    for (case, owner_file, flags) in [
        ("consumer reference", "plan", [true, true, false, true]),
        (
            "missing owner backlink",
            "plan",
            [false, true, false, false],
        ),
        ("missing owner link", "plan", [true, false, false, false]),
        ("indexed consumer", "plan", [true, true, true, true]),
        ("fenced consumer", "plan", [false, true, true, false]),
        ("existing TODO owner", "todo.md", [true, true, false, true]),
        (
            "missing TODO backlink",
            "todo.md",
            [false, true, false, false],
        ),
        (
            "TODO with existing plan",
            "todo.md",
            [true, true, false, false],
        ),
    ] {
        let [owner_backlink, owner_link, consumer_backlink, valid] = flags;
        let owner_file = if owner_file == "plan" {
            "00.crate_plan.md"
        } else {
            owner_file
        };
        remove_dir_if_exists(&workspace);
        for language in ["en", "ja"] {
            let contract =
                workspace.join(format!("doc/design/task_contracts/{language}/nested/T.md"));
            let owner = workspace.join(format!("doc/design/owner/{language}/{owner_file}"));
            let consumer =
                workspace.join(format!("doc/design/consumer/{language}/00.crate_plan.md"));
            create_dir(contract.parent().unwrap());
            create_dir(owner.parent().unwrap());
            create_dir(consumer.parent().unwrap());
            let backlink = format!("[T](../../task_contracts/{language}/nested/T.md)\n");
            let owner_heading = if owner_file == "todo.md" {
                "Tasks"
            } else {
                "Task Index"
            };
            write_test_file(
                &owner,
                &format!(
                    "## {owner_heading}\n{}",
                    if owner_backlink { &backlink } else { "" }
                ),
            );
            write_test_file(
                &consumer,
                &format!(
                    "## Task Index\n{}",
                    if consumer_backlink { &backlink } else { "" }
                ),
            );
            if case == "TODO with existing plan" {
                write_test_file(&owner.with_file_name("00.crate_plan.md"), "## Task Index\n");
            }
            let owner_link = if owner_link {
                format!("[owner](../../../owner/{language}/{owner_file})")
            } else {
                "owner".to_owned()
            };
            let consumer_link =
                format!("[consumer](../../../consumer/{language}/00.crate_plan.md)");
            let mut document = if language == "en" {
                format!("Owner: {owner_link}; consumers: {consumer_link}.\n")
            } else {
                format!("所有者: {owner_link}、{consumer_link}。\n")
            };
            if case == "fenced consumer" {
                document.push_str(&format!("```md\nconsumers: {owner_link}\n```\n"));
            }
            if case == "consumer reference" {
                write_test_file(&workspace.join("doc/design/todo.md"), "## Sequence\n");
                document.push_str("[sequence](../../../todo.md)\n");
            }
            write_test_file(&contract, &document);
            write_test_file(&contract.with_file_name("U.md"), "# Unrelated contract\n");
            let unrelated =
                workspace.join(format!("doc/design/unrelated/{language}/00.crate_plan.md"));
            create_dir(unrelated.parent().unwrap());
            write_test_file(
                &unrelated,
                &format!("## Task Index\n[U](../../task_contracts/{language}/nested/U.md)\n"),
            );
        }

        for language in ["en", "ja"] {
            let plan_backlinks = crate_plan_backlinks(&workspace.join("doc/design"));
            let contract =
                workspace.join(format!("doc/design/task_contracts/{language}/nested/T.md"));
            let links = markdown_link_destinations(&read_to_string(&contract));
            let mut violations = Vec::new();
            validate_crate_plan_backlinks(
                &plan_backlinks,
                &contract,
                &canonical_contract,
                &links,
                &mut violations,
            );
            assert_eq!(
                violations.is_empty(),
                valid,
                "{case} ({language}): {violations:?}"
            );

            if case == "indexed consumer" {
                let links = links
                    .into_iter()
                    .filter(|link| !link.contains("/consumer/"))
                    .collect::<Vec<_>>();
                let mut violations = Vec::new();
                validate_crate_plan_backlinks(
                    &plan_backlinks,
                    &contract,
                    &canonical_contract,
                    &links,
                    &mut violations,
                );
                assert!(
                    violations
                        .iter()
                        .any(|violation| violation.contains("indexed owning crate plan"))
                );
                let consumer =
                    workspace.join(format!("doc/design/consumer/{language}/00.crate_plan.md"));
                write_test_file(&consumer, "## Task Index\n");
                let plan_backlinks = crate_plan_backlinks(&workspace.join("doc/design"));
                let mut violations = Vec::new();
                validate_crate_plan_backlinks(
                    &plan_backlinks,
                    &contract,
                    &canonical_contract,
                    &links,
                    &mut violations,
                );
                assert!(violations.is_empty(), "{language}: {violations:?}");
            }
        }
    }
    remove_dir_if_exists(&workspace);
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
            name: "SyntaxSmokeCaseStatus",
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
            name: "FormulaStatementCaseStatus",
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
    plan_backlinks: &BTreeMap<PathBuf, BTreeSet<PathBuf>>,
    contract_path: &Path,
    canonical_contract_path: &Path,
    destinations: &[String],
    violations: &mut Vec<String>,
) {
    let contract = fs::canonicalize(contract_path).unwrap_or_else(|error| {
        panic!(
            "failed to resolve task contract {}: {error}",
            contract_path.display()
        )
    });
    let canonical_document = read_to_string(canonical_contract_path);
    let consumer_crates = visible_markdown_lines(&canonical_document)
        .into_iter()
        .filter_map(|(_, line)| {
            let lower = line.to_ascii_lowercase();
            ["consumers:", "consumer:"]
                .iter()
                .find_map(|marker| lower.find(marker).map(|start| start + marker.len()))
                .map(|start| line[start..].split(';').next().unwrap_or_default())
        })
        .flat_map(markdown_link_destinations)
        .filter_map(|destination| markdown_target_path(canonical_contract_path, &destination))
        .filter_map(|target| fs::canonicalize(target).ok())
        .filter(|target| {
            matches!(
                target.file_name().and_then(|name| name.to_str()),
                Some("00.crate_plan.md" | "todo.md")
            )
        })
        .filter_map(|target| {
            target
                .parent()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
        })
        .collect::<BTreeSet<_>>();
    let todo_paths = plan_backlinks
        .keys()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("todo.md"))
        .filter_map(|path| fs::canonicalize(path).ok())
        .collect::<BTreeSet<_>>();
    let linked_plans = destinations
        .iter()
        .filter_map(|destination| markdown_target_path(contract_path, destination))
        .filter_map(|target| fs::canonicalize(target).ok())
        .filter(|target| {
            target.file_name().and_then(|name| name.to_str()) == Some("00.crate_plan.md")
                || todo_paths.contains(target)
        })
        .collect::<BTreeSet<_>>();
    let mut indexed_plans = BTreeSet::new();
    for (plan_path, targets) in plan_backlinks {
        if targets.contains(&contract) {
            indexed_plans.insert(fs::canonicalize(plan_path).unwrap_or_else(|error| {
                panic!(
                    "failed to resolve crate plan {}: {error}",
                    plan_path.display()
                )
            }));
        }
    }

    if indexed_plans.intersection(&linked_plans).next().is_none() {
        violations.push(format!(
            "{}: task contract must link at least one owning crate plan or crate TODO",
            contract_path.display()
        ));
    }

    for plan_path in indexed_plans.difference(&linked_plans) {
        violations.push(format!(
            "{}: indexed owning crate plan {} is missing from contract links",
            contract_path.display(),
            plan_path.display()
        ));
    }
    for plan_path in linked_plans.difference(&indexed_plans) {
        if !plan_path
            .parent()
            .and_then(Path::parent)
            .is_some_and(|crate_path| consumer_crates.contains(crate_path))
        {
            violations.push(format!(
                "{}: owning crate plan {} must link back to the contract",
                contract_path.display(),
                plan_path.display()
            ));
        }
    }
}

fn crate_plan_backlinks(design_root: &Path) -> BTreeMap<PathBuf, BTreeSet<PathBuf>> {
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
            let plan = if plan.exists() {
                plan
            } else {
                plan.with_file_name("todo.md")
            };
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
        .into_iter()
        .map(|plan_path| {
            let plan = read_to_string(&plan_path);
            let task_index =
                if plan_path.file_name().and_then(|name| name.to_str()) == Some("todo.md") {
                    Some(plan.as_str())
                } else {
                    markdown_h2_section(&plan, "Task Index")
                };
            let targets = task_index
                .into_iter()
                .flat_map(markdown_link_destinations)
                .filter_map(|destination| markdown_target_path(&plan_path, &destination))
                .filter_map(|target| fs::canonicalize(target).ok())
                .collect();
            (plan_path, targets)
        })
        .collect()
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
    if schema != "schema\t2" {
        violations.push(format!(
            "legacy compaction manifest:{}: first data row must be schema<TAB>2",
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
        task_refs: BTreeSet::new(),
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
            Some("task_ref") if fields.len() == 3 => {
                for (label, id) in [("batch", fields[1]), ("task", fields[2])] {
                    if !valid_task_contract_id(id) {
                        violations.push(format!(
                            "legacy compaction manifest:{}: invalid task_ref {label} id {id:?}",
                            line_number + 1
                        ));
                    }
                }
                let key = (fields[1].to_owned(), fields[2].to_owned());
                if !manifest.task_refs.insert(key) {
                    violations.push(format!(
                        "legacy compaction manifest:{}: duplicate task_ref batch/task",
                        line_number + 1
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
            .count()
            + manifest
                .task_refs
                .iter()
                .filter(|(batch_id, _)| batch_id == &batch.id)
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
    for (batch_id, task_id) in &manifest.task_refs {
        if !manifest.batches.contains_key(batch_id) {
            violations.push(format!(
                "legacy compaction task_ref {batch_id}/{task_id} references undeclared batch"
            ));
        }
        let Some(task) = manifest.tasks.get(task_id) else {
            violations.push(format!(
                "legacy compaction task_ref {batch_id}/{task_id} references undeclared task"
            ));
            continue;
        };
        if task.batch_id == *batch_id {
            violations.push(format!(
                "legacy compaction task_ref {batch_id}/{task_id}: owner batch must use its task row"
            ));
        }
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
    if task.batch_id != redirect.batch_id
        && !manifest
            .task_refs
            .contains(&(redirect.batch_id.clone(), task.id.clone()))
    {
        violations.push(format!(
            "legacy redirect {} binds task {} to wrong batch {} without task_ref",
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
        task_refs: BTreeSet::new(),
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

fn assert_legacy_task_reference_vectors() {
    let workspace =
        std::env::temp_dir().join(format!("mizar_test_legacy_task_ref_{}", std::process::id()));
    remove_dir_if_exists(&workspace);

    for language in ["en", "ja"] {
        create_dir(&workspace.join(format!("doc/design/task_contracts/{language}")));
        create_dir(&workspace.join(format!("doc/design/component/{language}")));
        write_test_file(
            &workspace.join(format!("doc/design/task_contracts/{language}/OWNER.md")),
            "# Task OWNER: Owning batch\n\n## Completion Evidence\n",
        );
        write_test_file(
            &workspace.join(format!("doc/design/task_contracts/{language}/REF.md")),
            "# Task REF: Referencing batch\n\n## Completion Evidence\n",
        );
        write_test_file(
            &workspace.join(format!("doc/design/task_contracts/{language}/T.md")),
            "# Task T: Historical task\n\n## Completion Evidence\n",
        );
    }

    let replacement_en = "Completion evidence: [central Task-T historical contract](../../task_contracts/en/T.md#completion-evidence).";
    let replacement_ja = "Completion evidence: [central Task-T historical contract](../../task_contracts/ja/T.md#completion-evidence)。";
    write_test_file(
        &workspace.join("doc/design/component/en/ref.md"),
        &format!("## Previous EN\n{replacement_en}\n## Next EN\n"),
    );
    write_test_file(
        &workspace.join("doc/design/component/ja/ref.md"),
        &format!("## Previous JA\n{replacement_ja}\n## Next JA\n"),
    );
    let referenced_index = "| T | [EN contract](../../task_contracts/en/T.md) |";
    write_test_file(
        &workspace.join("doc/design/component/en/00.crate_plan.md"),
        &format!("## Task Index\n{referenced_index}\n"),
    );

    let task_row =
        "task\tOWNER\tT\tdoc/design/task_contracts/en/T.md\tdoc/design/task_contracts/ja/T.md"
            .to_owned();
    let task_ref_row = "task_ref\tREF\tT".to_owned();
    let redirect_en = format!(
        "redirect\tREF\tT\ten\tdoc/design/component/en/ref.md\t2\t## Legacy EN\t{replacement_en}\t## Previous EN\t## Next EN"
    );
    let redirect_ja = format!(
        "redirect\tREF\tT\tja\tdoc/design/component/ja/ref.md\t2\t## Legacy JA\t{replacement_ja}\t## Previous JA\t## Next JA"
    );

    let owner_inventory = format!("{task_row}\n");
    let mut ref_rows = vec![redirect_en, redirect_ja, task_ref_row];
    ref_rows.sort();
    let ref_inventory = format!("{}\n", ref_rows.join("\n"));
    let mut rows = vec![
        format!(
            "batch\tOWNER\tdoc/design/task_contracts/en/OWNER.md\tdoc/design/task_contracts/ja/OWNER.md\t{}\t1\t0\t0\t0",
            sha256_hex(owner_inventory.as_bytes())
        ),
        format!(
            "batch\tREF\tdoc/design/task_contracts/en/REF.md\tdoc/design/task_contracts/ja/REF.md\t{}\t1\t2\t2\t0",
            sha256_hex(ref_inventory.as_bytes())
        ),
        task_row,
    ];
    rows.extend(ref_rows);
    rows.sort();
    let manifest = format!("schema\t2\n{}\n", rows.join("\n"));

    let violations = legacy_compaction_manifest_violations(&workspace, &manifest);
    assert!(
        violations.is_empty(),
        "a second batch may reference one canonical historical task: {violations:?}"
    );

    let mutations = vec![
        (
            duplicate_first_manifest_row(&manifest, "task_ref"),
            "duplicate task_ref batch/task",
        ),
        (
            mutate_first_manifest_field(&manifest, "task_ref", 1, |_| "MISSING".to_owned()),
            "references undeclared batch",
        ),
        (
            mutate_first_manifest_field(&manifest, "task_ref", 2, |_| "MISSING".to_owned()),
            "references undeclared task",
        ),
        (
            mutate_first_manifest_field(&manifest, "task_ref", 1, |_| "OWNER".to_owned()),
            "owner batch must use its task row",
        ),
        (
            mutate_first_manifest_row(&manifest, "task_ref", |row| format!("{row}\textra")),
            "unknown kind or wrong field count",
        ),
        (
            mutate_first_manifest_field(&manifest, "task_ref", 1, |_| "bad/id".to_owned()),
            "invalid task_ref batch id",
        ),
        (
            mutate_first_manifest_field(&manifest, "task_ref", 2, |_| "bad/id".to_owned()),
            "invalid task_ref task id",
        ),
        (
            remove_first_manifest_row(&manifest, "task_ref"),
            "wrong batch REF without task_ref",
        ),
        (
            duplicate_first_manifest_row_with(&manifest, "redirect", |row| {
                let mut fields = row.split('\t').collect::<Vec<_>>();
                fields[1] = "OWNER";
                fields.join("\t")
            }),
            "duplicate redirect source/task",
        ),
        (
            append_manifest_row(
                &manifest,
                &format!(
                    "index\tREF\tT\ten\tdoc/design/component/en/00.crate_plan.md\t{referenced_index}"
                ),
            ),
            "binds task T to wrong batch REF",
        ),
    ];
    for (mutation, expected) in mutations {
        let violations = legacy_compaction_manifest_violations(&workspace, &mutation);
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains(expected)),
            "task_ref mutation must report {expected:?}, got:\n{}",
            violations.join("\n")
        );
    }

    remove_dir_if_exists(&workspace);
}

fn assert_legacy_manifest_mutation_vectors(workspace: &Path, manifest: &str) {
    let zero_hash = "0".repeat(64);
    let mutations = vec![
        (
            mutate_first_manifest_field(manifest, "schema", 1, |_| "1".to_owned()),
            "first data row must be schema<TAB>2",
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
            append_manifest_row(manifest, "schema\t2"),
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

fn remove_first_manifest_row(document: &str, kind: &str) -> String {
    let mut removed = false;
    let lines = document
        .lines()
        .filter(|line| {
            if !removed && line.split('\t').next() == Some(kind) {
                removed = true;
                false
            } else {
                true
            }
        })
        .collect::<Vec<_>>();
    assert!(removed, "manifest must contain a {kind} row");
    format!("{}\n", lines.join("\n"))
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
    for block in padded.as_chunks::<64>().0.iter() {
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

const DOC_VOLUME_BASELINE_PATH: &str = "tests/coverage/doc_volume_baseline.tsv";
const DOC_VOLUME_PROTOCOL_PATH: &str = "doc/design/autonomous_crate_development.md";
const DOC_VOLUME_CONTRACT_LINE_LIMIT: usize = 60;
const DOC_VOLUME_KINDS: [&str; 3] = ["ceremony_tokens", "contract_lines", "contract_fanout"];

fn parse_doc_volume_baseline(
    ledger: &str,
    violations: &mut Vec<String>,
) -> BTreeMap<(String, String), usize> {
    let mut rows = BTreeMap::new();
    let mut lines = ledger.lines();
    if lines.next() != Some("kind\tpath\tvalue") {
        violations.push(format!(
            "{DOC_VOLUME_BASELINE_PATH}: header must be `kind\\tpath\\tvalue`"
        ));
        return rows;
    }
    let mut previous: Option<(String, String)> = None;
    for (index, line) in lines.enumerate() {
        let line_number = index + 2;
        let fields: Vec<&str> = line.split('\t').collect();
        let [kind, key, value] = fields[..] else {
            violations.push(format!(
                "{DOC_VOLUME_BASELINE_PATH}:{line_number}: expected three tab-separated fields"
            ));
            continue;
        };
        if !DOC_VOLUME_KINDS.contains(&kind) {
            violations.push(format!(
                "{DOC_VOLUME_BASELINE_PATH}:{line_number}: unknown kind {kind:?}"
            ));
            continue;
        }
        let Ok(value) = value.parse::<usize>() else {
            violations.push(format!(
                "{DOC_VOLUME_BASELINE_PATH}:{line_number}: value must be a non-negative integer"
            ));
            continue;
        };
        let row_key = (kind.to_owned(), key.to_owned());
        if previous
            .as_ref()
            .is_some_and(|previous| *previous >= row_key)
        {
            violations.push(format!(
                "{DOC_VOLUME_BASELINE_PATH}:{line_number}: rows must be unique and sorted by kind then path"
            ));
        }
        previous = Some(row_key.clone());
        rows.insert(row_key, value);
    }
    rows
}

fn check_doc_volume_ratchet(
    baseline: &BTreeMap<(String, String), usize>,
    kind: &str,
    key: &str,
    actual: usize,
    default_limit: usize,
    what: &str,
    violations: &mut Vec<String>,
) {
    let limit = baseline
        .get(&(kind.to_owned(), key.to_owned()))
        .copied()
        .unwrap_or(default_limit);
    if actual > limit {
        violations.push(format!("{key}: {actual} > {limit} {what} [{kind}]"));
    }
}

fn workspace_relative_slash_path(workspace: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(workspace).unwrap_or_else(|error| {
        panic!(
            "failed to make {} relative to {}: {error}",
            path.display(),
            workspace.display()
        )
    });
    relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn contract_fanout_allowed(relative: &str) -> bool {
    if relative == "doc/design/todo.md" || relative == "doc/design/spec_coverage_audit.md" {
        return true;
    }
    let parts: Vec<&str> = relative.split('/').collect();
    if parts.last() == Some(&"00.crate_plan.md") {
        return true;
    }
    parts.len() == 5 && matches!(parts[3], "en" | "ja") && parts[4] == "todo.md"
}

/// Counts evaluation ceremony that must not live in design documents:
/// `quality score`, `score cap`, `xhigh`, `<digits>/100`, `hard gate(s) N/M`,
/// the model names `Luna`/`Sol`/`Terra` as whole words, and runs of 64 or
/// more lowercase hex digits.
fn ceremony_token_count(document: &str) -> usize {
    let lower = document.to_ascii_lowercase();
    lower.matches("quality score").count()
        + lower.matches("score cap").count()
        + lower.matches("xhigh").count()
        + score_fraction_count(document)
        + gate_fraction_count(&lower)
        + hex_digest_count(document)
        + whole_word_count(document, "Luna")
        + whole_word_count(document, "Sol")
        + whole_word_count(document, "Terra")
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn whole_word_count(document: &str, word: &str) -> usize {
    let bytes = document.as_bytes();
    document
        .match_indices(word)
        .filter(|(start, _)| {
            let end = start + word.len();
            let before_ok = *start == 0 || !is_word_byte(bytes[start - 1]);
            let after_ok = end >= bytes.len() || !is_word_byte(bytes[end]);
            before_ok && after_ok
        })
        .count()
}

fn score_fraction_count(document: &str) -> usize {
    let bytes = document.as_bytes();
    document
        .match_indices("/100")
        .filter(|(start, _)| {
            let end = start + 4;
            let digit_before = *start > 0 && bytes[start - 1].is_ascii_digit();
            let digit_after = end < bytes.len() && bytes[end].is_ascii_digit();
            digit_before && !digit_after
        })
        .count()
}

fn gate_fraction_count(lower: &str) -> usize {
    let bytes = lower.as_bytes();
    let mut count = 0;
    for (start, _) in lower.match_indices("hard gate") {
        let mut index = start + "hard gate".len();
        if bytes.get(index) == Some(&b's') {
            index += 1;
        }
        if bytes.get(index) != Some(&b' ') {
            continue;
        }
        index += 1;
        let digits_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == digits_start || bytes.get(index) != Some(&b'/') {
            continue;
        }
        index += 1;
        let denominator_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index > denominator_start {
            count += 1;
        }
    }
    count
}

fn hex_digest_count(document: &str) -> usize {
    let mut count = 0;
    let mut run = 0;
    for byte in document.bytes().chain(std::iter::once(b' ')) {
        if matches!(byte, b'0'..=b'9' | b'a'..=b'f') {
            run += 1;
        } else {
            if run >= 64 {
                count += 1;
            }
            run = 0;
        }
    }
    count
}

fn assert_doc_volume_matcher_vectors() {
    assert_eq!(ceremony_token_count("plain design prose about sets"), 0);
    assert_eq!(ceremony_token_count("Hard gates:\n"), 0);
    assert_eq!(ceremony_token_count("hard gates 9/9 pass"), 1);
    assert_eq!(ceremony_token_count("Hard gate 1/9 failed"), 1);
    assert_eq!(ceremony_token_count("final quality score 100/100"), 2);
    assert_eq!(ceremony_token_count("Score caps applied: none"), 1);
    assert_eq!(ceremony_token_count("ratio 3/1000 and 1/100"), 1);
    assert_eq!(
        ceremony_token_count("Sol xhigh parent; Luna `xhigh`; Terra high"),
        5
    );
    assert_eq!(ceremony_token_count("Solar Lunar Terrace Solution"), 0);
    assert_eq!(ceremony_token_count("Lunaはruntime"), 1);
    assert_eq!(
        ceremony_token_count(&format!("digest {} end", "a".repeat(64))),
        1
    );
    assert_eq!(
        ceremony_token_count(&format!("short {} end", "a".repeat(63))),
        0
    );
    assert!(contract_fanout_allowed("doc/design/todo.md"));
    assert!(contract_fanout_allowed(
        "doc/design/mizar-core/en/00.crate_plan.md"
    ));
    assert!(contract_fanout_allowed("doc/design/mizar-core/ja/todo.md"));
    assert!(!contract_fanout_allowed(
        "doc/design/mizar-core/en/elaborator.md"
    ));
    assert!(!contract_fanout_allowed(
        "doc/design/mizar-core/en/nested/todo.md"
    ));

    let mut violations = Vec::new();
    let rows = parse_doc_volume_baseline(
        "kind\tpath\tvalue\nceremony_tokens\ta.md\t3\ncontract_lines\tb.md\t70\n",
        &mut violations,
    );
    assert!(violations.is_empty(), "{violations:?}");
    assert_eq!(rows.len(), 2);
    let mut violations = Vec::new();
    parse_doc_volume_baseline(
        "kind\tpath\tvalue\ncontract_lines\tb.md\t70\nceremony_tokens\ta.md\t3\nbogus\tc\t1\n",
        &mut violations,
    );
    assert_eq!(violations.len(), 2, "{violations:?}");
    let mut violations = Vec::new();
    check_doc_volume_ratchet(
        &rows,
        "ceremony_tokens",
        "a.md",
        3,
        0,
        "tokens",
        &mut violations,
    );
    check_doc_volume_ratchet(
        &rows,
        "ceremony_tokens",
        "a.md",
        4,
        0,
        "tokens",
        &mut violations,
    );
    check_doc_volume_ratchet(
        &rows,
        "ceremony_tokens",
        "z.md",
        1,
        0,
        "tokens",
        &mut violations,
    );
    check_doc_volume_ratchet(
        &rows,
        "contract_lines",
        "new.md",
        60,
        60,
        "lines",
        &mut violations,
    );
    assert_eq!(violations.len(), 2, "{violations:?}");
}

#[test]
fn literal_source_offset_counter_uses_whole_endpoint_syntax() {
    let cases = [
        ("source id alone", "let _ = SourceId(7);", 0),
        (
            "raw range type",
            "let _ = r#SourceRange { source_id: id, start: 1, end };",
            1,
        ),
        (
            "raw constructor",
            "let _ = SourceRange::r#new(id, 1, end);",
            1,
        ),
        (
            "raw endpoint",
            "let _ = SourceRange { source_id: id, r#start: 1, end };",
            1,
        ),
        ("raw named array", "let r#expected_ranges = [1];", 1),
        (
            "integer source id alone",
            "let _ = SourceRange::new(12, start, end);",
            0,
        ),
        (
            "direct struct start",
            "let _ = SourceRange { source_id: id, start: 4, end };",
            1,
        ),
        (
            "mixed struct endpoint",
            "let _ = SourceRange { source_id: id, start, end: 9 };",
            1,
        ),
        (
            "qualified struct",
            "let _ = mizar_core::SourceRange { end: 8, source_id: id, start };",
            1,
        ),
        (
            "constructor start",
            "let _ = SourceRange::new(id, 5, end);",
            1,
        ),
        (
            "both constructor endpoints count once",
            "let _ = SourceRange::new(id, 5, 6);",
            1,
        ),
        (
            "radix separators suffix groups and negative",
            "let _ = SourceRange::new(make_id(), (-0x1_0_i32), (1_234usize));",
            1,
        ),
        (
            "arithmetic and casts are not whole literals",
            "let _ = SourceRange::new(id, 1 + 2, end as usize);",
            0,
        ),
        (
            "float is not an integer literal",
            "let _ = SourceRange::new(id, 1.0, end);",
            0,
        ),
        (
            "one whole endpoint still counts once",
            "let _ = SourceRange::new(id, 1 + 2, 3);",
            1,
        ),
        (
            "independent constructions",
            "let _ = SourceRange::new(id, 1, end); let _ = SourceRange { source_id: id, start, end: 2 };",
            2,
        ),
    ];
    for (case, source, expected) in cases {
        assert_eq!(
            literal_source_offset_count(&format!("fn body() {{ {source} }}")),
            Ok(expected),
            "{case}"
        );
    }
}

#[test]
fn literal_source_offset_counter_recognizes_only_local_forward_helpers() {
    let source = r#"
        fn forward(source: SourceId, begin: usize, finish: usize) -> SourceRange {
            SourceRange { source_id: source, start: (begin), end: finish }
        }
        fn mirror(source: SourceId, at: usize) -> SourceRange {
            SourceRange { source_id: source, start: at, end: at }
        }
        fn not_sole_tail(source: SourceId, begin: usize, finish: usize) -> SourceRange {
            let range = SourceRange { source_id: source, start: begin, end: finish };
            range
        }
        fn callers(id: SourceId, begin: usize, finish: usize) {
            let _ = forward(id, 3, finish);
            let _ = self::forward(id, begin, 4);
            let _ = mirror(id, 5);
            let _ = not_sole_tail(id, 6, 7);
        }
    "#;
    assert_eq!(literal_source_offset_count(source), Ok(3));
    let conditional = r#"
        #[cfg(feature = "alt")]
        fn forward(id: SourceId, start: usize, end: usize) -> SourceRange {
            SourceRange { source_id: id, start, end }
        }
        #[cfg(not(feature = "alt"))]
        fn forward(start: usize, end: usize, id: SourceId) -> SourceRange {
            SourceRange { source_id: id, start, end }
        }
        fn f(id: SourceId, begin: usize) { let _ = forward(id, begin, 1); }
    "#;
    assert_eq!(literal_source_offset_count(conditional), Ok(1));
    let raw_helper = r#"
        fn r#forward(id: SourceId, start: usize, end: usize) -> SourceRange {
            SourceRange { source_id: id, start, end }
        }
        fn f(id: SourceId, begin: usize) { let _ = forward(id, begin, 1); }
    "#;
    assert_eq!(literal_source_offset_count(raw_helper), Ok(1));
    let raw_shadow = r#"
        fn forward(id: SourceId, start: usize, end: usize) -> SourceRange {
            SourceRange { source_id: id, start, end }
        }
        fn f(r#forward: Callable, id: SourceId) { let _ = forward(id, 1, 2); }
    "#;
    assert!(literal_source_offset_count(raw_shadow).is_err());

    let shadowed_call = r#"
        fn forward(source: SourceId, begin: usize, finish: usize) -> SourceRange {
            SourceRange { source_id: source, start: begin, end: finish }
        }
        fn caller(id: SourceId) {
            let forward = opaque_value;
            let _ = forward(id, 1, 2);
        }
    "#;
    assert!(literal_source_offset_count(shadowed_call).is_err());

    let partial = r#"
        fn partial(id: SourceId, start: usize, end: usize) -> SourceRange {
            SourceRange { source_id: id, start, end: end + 1 }
        }
        fn caller() { let _ = partial(id, 7, end); }
    "#;
    assert_eq!(literal_source_offset_count(partial), Ok(0));

    let qualified_helper = r#"
        const fn qualified(source: SourceId, begin: usize, finish: usize) -> crate::SourceRange {
            return (crate::SourceRange { source_id: source, start: (begin), end: (finish) });
        }
    "#;
    for (call, expected) in [
        ("fn f() { let _ = qualified(99, 2, end); }", 1),
        ("fn f() { let _ = qualified(9, begin, end); }", 0),
        ("fn f() { let _ = object.qualified(id, 3, end); }", 0),
    ] {
        let source = format!("{qualified_helper}\n{call}");
        assert_eq!(literal_source_offset_count(&source), Ok(expected));
    }

    let module_maps = r#"
        mod first {
            fn forward(id: SourceId, start: usize, end: usize) -> SourceRange {
                SourceRange { source_id: id, start, end }
            }
            fn caller() { let _ = forward(id, 1, end); }
        }
        mod second {
            fn forward(_id: SourceId, _start: usize, _end: usize) -> usize { 0 }
            fn caller() { let _ = forward(id, 2, end); }
        }
    "#;
    assert_eq!(literal_source_offset_count(module_maps), Ok(1));

    let lexical_scopes = r#"
        fn forward(id: SourceId, start: usize, end: usize) -> SourceRange {
            SourceRange { source_id: id, start, end }
        }
        fn only_binding(forward: usize) { let _ = forward; }
        fn qualified_call(forward: impl Fn(SourceId, usize, usize) -> SourceRange, id: SourceId) {
            let _ = self::forward(id, 1, end);
        }
        fn block_local_function() { fn forward(_value: usize) {} forward(1); }
    "#;
    assert_eq!(literal_source_offset_count(lexical_scopes), Ok(1));

    let helper = "fn forward(id: SourceId, start: usize, end: usize) -> SourceRange { SourceRange { source_id: id, start, end } }";
    let shadows = [
        (
            "parameter",
            "fn f(forward: impl Fn(SourceId, usize, usize) -> SourceRange, id: SourceId) { let _ = forward(id, 1, end); }",
        ),
        (
            "closure parameter",
            "fn f(id: SourceId) { let _ = (|forward| forward(id, 1, end))(value); }",
        ),
        (
            "loop binding",
            "fn f(id: SourceId) { for forward in values { let _ = forward(id, 1, end); } }",
        ),
        (
            "match binding",
            "fn f(id: SourceId, value: usize) { match value { forward => { let _ = forward(id, 1, end); } } }",
        ),
        (
            "block import",
            "fn f(id: SourceId) { use other::forward; let _ = forward(id, 1, end); }",
        ),
        (
            "block const",
            "fn f(id: SourceId) { const forward: fn(SourceId, usize, usize) -> SourceRange = foreign::forward; let _ = forward(id, 1, end); }",
        ),
        (
            "block static",
            "fn f(id: SourceId) { static forward: fn(SourceId, usize, usize) -> SourceRange = foreign::forward; let _ = forward(id, 1, end); }",
        ),
        (
            "conditionally enabled block functions",
            r#"fn f(id: SourceId) {
                #[cfg(feature = "alt")]
                fn forward(id: SourceId, start: usize, end: usize) -> SourceRange {
                    SourceRange { source_id: id, start, end }
                }
                #[cfg(not(feature = "alt"))]
                fn forward(id: SourceId, start: usize, end: usize) -> usize { 0 }
                let _ = forward(id, 1, end);
            }"#,
        ),
    ];
    for (case, shadow) in shadows {
        let source = format!("{helper}\n{shadow}");
        syn::parse_file(&source).expect("valid helper shadow fixture");
        assert!(
            literal_source_offset_count(&source).is_err(),
            "ambiguous {case} shadow"
        );
    }
    let constructor = format!(
        "{helper}\nfn f(id: SourceId) {{ struct forward(SourceId, usize, usize); let _ = forward(id, 1, end); }}"
    );
    assert_eq!(literal_source_offset_count(&constructor), Ok(0));
}

#[test]
fn literal_source_offset_counter_counts_named_arrays_and_value_wrappers() {
    let source = r#"
        fn cases(flag: bool, value: usize) {
            let expected_ranges: [usize; 5] = [2, (3, 4), (value, 5), value, (value, value)];
            let expected_sites = [(1, 2), (value, 3), value];
            let unrelated = [4, (5, 6)];
            let expected_ranges = &({
                let scratch = [7, 8];
                let closure = || [9, 10];
                match flag {
                    true => [(value, 11)],
                    false => if flag { [12] } else { [value] },
                }
            });
        }
    "#;
    // The first named arrays contribute 3 + 2; the wrapped match/if branches
    // contribute 2. Scratch, closure, and unrelated arrays contribute 0.
    assert_eq!(literal_source_offset_count(source), Ok(7));
    for (source, expected) in [
        (
            "fn f(flag: bool) { let expected_ranges = match flag { true => [(1, 2)], false => unreachable!(\"handled above\") }; }",
            1,
        ),
        ("fn f() { let expected_sites = unreachable!(); }", 0),
        (
            "fn f(value: usize) { let expected_sites = [(#[cfg(test)] 1, value)]; }",
            0,
        ),
    ] {
        assert_eq!(literal_source_offset_count(source), Ok(expected));
    }
    assert!(
        literal_source_offset_count(
            "fn f(message: &str) { let expected_sites = unreachable!(message); }"
        )
        .is_err()
    );

    assert_eq!(
        literal_source_offset_count("fn f() { let ref expected_sites = [1]; }"),
        Ok(0)
    );
    assert_eq!(
        literal_source_offset_count("fn f() { let expected_sites @ _ = [1]; }"),
        Ok(0)
    );

    for (case, source, expected) in [
        (
            "tuple depth",
            "fn f(v: usize) { let expected_sites = [((1, 2), v)]; }",
            0,
        ),
        (
            "call arguments",
            "fn f() { let expected_sites = compute([1, 2]); }",
            0,
        ),
        (
            "nested named binding",
            "fn f() { let expected_sites = { let expected_ranges = [1]; [2] }; }",
            2,
        ),
        (
            "parameter/type positions",
            "fn f(expected_ranges: [usize; 2]) { let expected_sites: [usize; 1] = value; }",
            0,
        ),
    ] {
        assert_eq!(literal_source_offset_count(source), Ok(expected), "{case}");
    }
    assert_eq!(
        literal_source_offset_count(
            "fn f() { let expected_ranges = compute(); let expected_sites = [value, value]; }"
        ),
        Ok(0)
    );
    assert!(
        literal_source_offset_count("fn f() { let expected_ranges = opaque_value!(); }").is_err()
    );
}

#[test]
fn literal_source_offset_counter_visits_macro_payloads_not_patterns_or_text() {
    let source = r##"
        fn cases(id: SourceId, end: usize) {
            let _ = vec![SourceRange::new(id, 1, end), SourceRange { source_id: id, start: 2, end }];
            let _ = vec![SourceRange::new(id, 3, end); 4];
            let _ = matches!(
                SourceRange::new(id, 4, end),
                SourceRange { start: 90, end: 91, .. } if SourceRange::new(id, 5, end).start == 0
            );
            assert_eq!(SourceRange::new(id, 6, end).start, 0, "{}", SourceRange::new(id, 7, end).start);
            let _ = format!("{item:?}", item = SourceRange::new(id, 8, end));
            println!("{}", SourceRange { source_id: id, start: 9, end }.start);
            let ordinary = "SourceRange { start: 10, end: 11 }";
            let raw = r#"SourceRange::new(id, 12, 13)"#;
            let character = 'x';
            // SourceRange::new(id, 14, 15)
            /* SourceRange { start: 16, end: 17 } */
        }
    "##;
    // vec payloads 3, matches scrutinee/guard 2, assert args 2, named format
    // assignment 1, and println arg 1. Pattern and lexical decoys add 0.
    assert_eq!(literal_source_offset_count(source), Ok(9));
    let accepted = [
        ("assert!", "SourceRange::new(id, 1, end)", 1),
        (
            "assert_eq!",
            "SourceRange::new(id, 1, end), SourceRange::new(id, 2, end)",
            2,
        ),
        (
            "assert_ne!",
            "SourceRange::new(id, 1, end), SourceRange::new(id, 2, end)",
            2,
        ),
        ("debug_assert!", "SourceRange::new(id, 1, end)", 1),
        (
            "debug_assert_eq!",
            "SourceRange::new(id, 1, end), SourceRange::new(id, 2, end)",
            2,
        ),
        (
            "debug_assert_ne!",
            "SourceRange::new(id, 1, end), SourceRange::new(id, 2, end)",
            2,
        ),
        ("format!", "\"{:?}\", SourceRange::new(id, 1, end)", 1),
        ("format_args!", "\"{:?}\", SourceRange::new(id, 1, end)", 1),
        ("print!", "\"{:?}\", SourceRange::new(id, 1, end)", 1),
        ("println!", "\"{:?}\", SourceRange::new(id, 1, end)", 1),
        ("eprint!", "\"{:?}\", SourceRange::new(id, 1, end)", 1),
        ("eprintln!", "\"{:?}\", SourceRange::new(id, 1, end)", 1),
        (
            "write!",
            "writer, \"{:?}\", SourceRange::new(id, 1, end)",
            1,
        ),
        (
            "writeln!",
            "writer, \"{:?}\", SourceRange::new(id, 1, end)",
            1,
        ),
    ];
    for (name, args, expected) in accepted {
        let source = format!("fn f() {{ {name}({args}); }}");
        assert_eq!(literal_source_offset_count(&source), Ok(expected), "{name}");
    }
    for source in [
        "fn f() { unknown!(SourceRange); }",
        "fn f() { unknown!(\"SourceRange::new(id, 1, 2)\"); }",
        "fn f() { unknown!(r#\"expected_ranges = SourceRange\"#); }",
    ] {
        assert_eq!(literal_source_offset_count(source), Ok(0), "{source}");
    }
    let helper = "fn forward(id: SourceId, start: usize, end: usize) -> SourceRange { SourceRange { source_id: id, start, end } }";
    for (case, tokens) in [
        (
            "nested helper",
            "fn f() { unknown!({ ((forward(id, 1, end))) }); }",
        ),
        (
            "named assignment",
            "fn f() { unknown!({ let expected_ranges = [1]; }); }",
        ),
        (
            "named type",
            "fn f() { unknown!({ let expected_sites: [usize; 1]; }); }",
        ),
    ] {
        let source = format!("{helper}\n{tokens}");
        assert!(
            literal_source_offset_count(&source).is_err(),
            "unknown macro {case}"
        );
    }
    for source in [
        "fn f() { unknown!(SourceRange::new(id, 1, end)); }",
        "fn f() { unknown!(r#SourceRange::r#new(id, 1, end)); }",
        "fn f() { r#include!(\"missing.rs\"); }",
        "macro_rules! hidden { () => { SourceRange { source_id: id, start: 1, end: 2 } } }",
        "fn f() { let expected_ranges = [opaque_value!(); 3]; }",
        "fn f() { assert!(, SourceRange::new(id, 1, end)); }",
    ] {
        assert!(literal_source_offset_count(source).is_err(), "{source}");
    }
}

#[test]
fn literal_source_offset_counter_applies_three_valued_cfg() {
    let source = r#"
        #[cfg(test)] fn test_only() { let _ = SourceRange::new(id, 1, 2); }
        #[cfg(all(test, feature = "optional"))]
        fn definitely_off() { let _ = SourceRange::new(id, 3, 4); }
        #[cfg(any(test, feature = "optional"))]
        fn possibly_on() { let _ = SourceRange::new(id, 5, 6); }
        #[cfg(not(test))] fn production() { let _ = SourceRange::new(id, 7, 8); }
        fn local_items() {
            #[cfg(test)] let _ = SourceRange::new(id, 9, 10);
            #[cfg(feature = "optional")] let _ = SourceRange::new(id, 11, 12);
        }
    "#;
    // test is false; other predicates are unknown and included unless a false
    // test conjunct proves the complete cfg false.
    assert_eq!(literal_source_offset_count(source), Ok(3));
    assert_eq!(
        literal_source_offset_count(
            "#![cfg(test)]\nfn f() { let _ = SourceRange::new(id, 1, 2); }"
        ),
        Ok(0)
    );
    assert_eq!(
        literal_source_offset_count(
            "#![cfg(not(test))]\nfn f() { let _ = SourceRange::new(id, 1, 2); }"
        ),
        Ok(1)
    );
    assert_eq!(
        literal_source_offset_count(
            "fn f() { #[cfg(test)] { let _ = SourceRange::new(id, 1, 2); } #[cfg(feature = \"maybe\")] { let _ = SourceRange::new(id, 3, 4); } }"
        ),
        Ok(1)
    );
    for (source, expected) in [
        ("fn f() { #[cfg(test)] include!(\"missing.rs\"); }", 0),
        (
            "fn f() { #[cfg(test)] unknown!(SourceRange::new(id, 1, end)); }",
            0,
        ),
        (
            "fn f() { #[cfg(feature = \"maybe\")] println!(\"{:?}\", SourceRange::new(id, 2, end)); }",
            1,
        ),
    ] {
        assert_eq!(literal_source_offset_count(source), Ok(expected));
    }
}

#[test]
fn literal_source_offset_workspace_finds_production_sources() {
    let root = std::env::temp_dir().join(format!(
        "mizar_literal_offset_manifest_roots_{}",
        std::process::id()
    ));
    remove_dir_if_exists(&root);
    create_dir(&root.join("crates/demo/src/test_support"));
    create_dir(&root.join("crates/demo/src/inline"));
    create_dir(&root.join("crates/demo/src/child"));
    create_dir(&root.join("crates/demo/src/tree"));
    create_dir(&root.join("crates/demo/src/fragments"));
    create_dir(&root.join("crates/mizar-test/src"));
    write_test_file(
        &root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/demo\", \"crates/mizar-test\"]\nresolver = \"2\"\n",
    );
    write_test_file(
        &root.join("crates/demo/Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[lib]\npath = \"src/lib.rs\"\n[[bin]]\nname = \"demo-bin\"\npath = \"src/main.rs\"\n",
    );
    write_test_file(
        &root.join("crates/demo/src/lib.rs"),
        r#"
            mod child; mod mizar_test; mod tree;
            #[path = "../outside.rs"] mod outside;
            #[path = "../shared.rs"] mod production_shared;
            #[cfg(test)] #[path = "../shared.rs"] mod test_shared;
            #[cfg(test)] #[path = "test_support.rs"] mod tests;
            #[cfg(test)] #[path = "missing_test.rs"] mod absent_test;
            #[cfg(any(test, feature = "maybe"))] #[path = "../maybe.rs"] mod maybe;
            mod inline {
                #[path = "custom.rs"] mod child;
                fn f() { let _ = SourceRange::new(id, 17, end); }
                #[cfg(test)] mod gated { fn f() { let _ = SourceRange::new(id, 18, end); } }
            }
            #[cfg(test)]
            mod include_tests {
                include!("fragments/test_fragments.rs");
                include!("../shared.rs");
            }
            fn root_offset() { let _ = SourceRange::new(id, 1, end); }
        "#,
    );
    write_test_file(
        &root.join("crates/demo/src/main.rs"),
        "#[path = \"../shared.rs\"] mod binary_shared; fn main() { let _ = SourceRange::new(id, 2, end); }",
    );
    write_test_file(
        &root.join("crates/demo/src/child.rs"),
        "mod nested; fn f() { let _ = SourceRange::new(id, 3, end); }",
    );
    write_test_file(
        &root.join("crates/demo/src/child/nested.rs"),
        "fn f() { let _ = SourceRange::new(id, 21, end); }",
    );
    write_test_file(
        &root.join("crates/demo/src/tree/mod.rs"),
        "fn f() { let _ = SourceRange::new(id, 22, end); }",
    );
    write_test_file(&root.join("crates/demo/src/zero.rs"), "fn no_offset() {}");
    write_test_file(
        &root.join("crates/demo/src/fragments/test_fragments.rs"),
        "fn first_test_fragment() { let _ = SourceRange::new(id, 23, end); } include!(\"test_nested.rs\");",
    );
    write_test_file(
        &root.join("crates/demo/src/fragments/test_nested.rs"),
        "fn second_test_fragment() { let _ = SourceRange::new(id, 24, end); }",
    );
    write_test_file(
        &root.join("crates/demo/src/mizar_test.rs"),
        "fn f() { let _ = SourceRange::new(id, 4, end); }",
    );
    write_test_file(
        &root.join("crates/demo/src/orphan.rs"),
        "fn f() { let _ = SourceRange::new(id, 5, end); }",
    );
    write_test_file(
        &root.join("crates/demo/outside.rs"),
        "fn f() { let _ = SourceRange::new(id, 6, end); }",
    );
    write_test_file(
        &root.join("crates/demo/shared.rs"),
        "fn f() { let _ = SourceRange::new(id, 7, end); }",
    );
    write_test_file(
        &root.join("crates/demo/maybe.rs"),
        "fn f() { let _ = SourceRange::new(id, 8, end); }",
    );
    write_test_file(
        &root.join("crates/demo/src/test_support.rs"),
        "mod descendant; mod inline { fn f() { let _ = SourceRange::new(id, 9, 10); } } #[cfg(not(test))] #[path = \"test_support/production_after.rs\"] mod production_after; fn f() { let _ = SourceRange::new(id, 11, 12); } include!(\"external_fragment.rs\");",
    );
    write_test_file(
        &root.join("crates/demo/src/descendant.rs"),
        "fn f() { let _ = SourceRange::new(id, 13, 14); }",
    );
    write_test_file(
        &root.join("crates/demo/src/test_support/production_after.rs"),
        "fn f() { let _ = SourceRange::new(id, 15, 16); }",
    );

    write_test_file(
        &root.join("crates/demo/src/inline/custom.rs"),
        "fn f() { let _ = SourceRange::new(id, 19, end); }",
    );
    write_test_file(
        &root.join("crates/mizar-test/Cargo.toml"),
        "[package]\nname = \"mizar-test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write_test_file(
        &root.join("crates/mizar-test/src/lib.rs"),
        "fn f() { let _ = SourceRange::new(id, 20, end); }",
    );

    write_test_file(
        &root.join("crates/demo/src/external_fragment.rs"),
        "include!(\"external_nested.rs\"); fn first() { let _ = SourceRange::new(id, 25, end); }",
    );
    write_test_file(
        &root.join("crates/demo/src/external_nested.rs"),
        "fn second() { let _ = SourceRange::new(id, 26, end); }",
    );

    let actual = literal_source_offset_counts(&root).unwrap();
    assert_eq!(actual.get("crates/demo/src/zero.rs"), Some(&0));
    for test_only in [
        "crates/demo/src/test_support.rs",
        "crates/demo/src/descendant.rs",
        "crates/demo/src/test_support/production_after.rs",
        "crates/demo/src/fragments/test_fragments.rs",
        "crates/demo/src/fragments/test_nested.rs",
        "crates/demo/src/external_fragment.rs",
        "crates/demo/src/external_nested.rs",
    ] {
        assert!(
            !actual.contains_key(test_only),
            "test-only key leaked: {test_only}"
        );
    }
    let positive = actual
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .collect::<BTreeMap<_, _>>();
    let expected = BTreeMap::from([
        ("crates/demo/maybe.rs".to_owned(), 1),
        ("crates/demo/outside.rs".to_owned(), 1),
        ("crates/demo/shared.rs".to_owned(), 1),
        ("crates/demo/src/child.rs".to_owned(), 1),
        ("crates/demo/src/child/nested.rs".to_owned(), 1),
        ("crates/demo/src/lib.rs".to_owned(), 2),
        ("crates/demo/src/inline/custom.rs".to_owned(), 1),
        ("crates/demo/src/main.rs".to_owned(), 1),
        ("crates/demo/src/mizar_test.rs".to_owned(), 1),
        ("crates/demo/src/orphan.rs".to_owned(), 1),
        ("crates/demo/src/tree/mod.rs".to_owned(), 1),
        ("crates/mizar-test/src/lib.rs".to_owned(), 1),
    ]);
    assert_eq!(positive, expected);
    remove_dir_if_exists(&root);
}

#[test]
fn literal_source_offset_workspace_requires_builtin_test_include_proof() {
    let root = std::env::temp_dir().join(format!(
        "mizar_literal_offset_include_proof_{}",
        std::process::id()
    ));
    remove_dir_if_exists(&root);
    create_dir(&root.join("crates/demo/src"));
    write_test_file(
        &root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/demo\"]\nresolver = \"2\"\n",
    );
    write_test_file(
        &root.join("crates/demo/Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write_test_file(
        &root.join("crates/demo/src/orphan.rs"),
        "fn orphan() { let _ = SourceRange::new(id, 1, end); }",
    );
    write_test_file(
        &root.join("crates/demo/ambiguous_bindings.rs"),
        "use crate::foreign::*;",
    );

    write_test_file(
        &root.join("crates/demo/src/lib.rs"),
        r#"
            mod foreign { pub fn value() {} }
            #[cfg(test)]
            mod test_scope {
                fn local_names() {
                    use crate::foreign::*;
                    let include = false;
                    let _ = include;
                }
                fn include() {}
                use mizar_session::Hash;
                #[derive(Debug)] struct Ordinary;
                include!("orphan.rs");
            }
        "#,
    );
    let ordinary_local_names = literal_source_offset_counts(&root).unwrap();
    assert!(
        !ordinary_local_names.contains_key("crates/demo/src/orphan.rs"),
        "function-local names and an ordinary function do not shadow item include!"
    );

    let cases = [
        (
            "local no-op macro",
            "",
            "macro_rules! include { ($path:literal) => {}; }",
        ),
        (
            "renamed macro import",
            "#[macro_export] macro_rules! renamed_include { ($path:literal) => {}; }",
            "use crate::renamed_include as include;",
        ),
        (
            "nonliteral test include",
            "",
            "include!(concat!(\"or\", \"phan.rs\"));",
        ),
        (
            "qualified test include",
            "",
            "std::include!(\"orphan.rs\");",
        ),
        (
            "aliased test include",
            "",
            "use std::include as imported_include; imported_include!(\"orphan.rs\");",
        ),
        ("module glob", "", "use crate::foreign::*;"),
        (
            "macro_use",
            "",
            "#[macro_use] mod imported_macros { macro_rules! include { ($path:literal) => {}; } }",
        ),
        (
            "imported built-in-named procedural derive",
            "",
            "use custom_derive::Debug; #[derive(Debug)] struct Generated;",
        ),
        (
            "raw imported built-in-named procedural derive",
            "",
            "use custom_derive::r#Debug; #[derive(Debug)] struct Generated;",
        ),
        (
            "raw local no-op macro",
            "",
            "macro_rules! r#include { ($path:literal) => {}; }",
        ),
        (
            "imported Hash procedural derive",
            "",
            "use custom_derive::Hash; #[derive(Hash)] struct Generated;",
        ),
        ("opaque item macro", "", "opaque_item!();"),
        (
            "custom procedural attribute",
            "",
            "#[opaque_attribute] fn generated() {}",
        ),
        (
            "prior included fragment with glob",
            "",
            "include!(\"../ambiguous_bindings.rs\");",
        ),
    ];
    for (case, root_items, module_items) in cases {
        let source = format!(
            "{root_items}\nmod foreign {{ pub fn value() {{}} }}\n#[cfg(test)] mod test_scope {{ {module_items} include!(\"orphan.rs\"); }}"
        );
        write_test_file(&root.join("crates/demo/src/lib.rs"), &source);
        let actual = literal_source_offset_counts(&root);
        let orphan_remains_production = actual
            .as_ref()
            .ok()
            .and_then(|counts| counts.get("crates/demo/src/orphan.rs"))
            .copied()
            == Some(1);
        assert!(
            actual.is_err() || orphan_remains_production,
            "{case} cannot prove the orphan is test-only"
        );
    }
    remove_dir_if_exists(&root);
}

#[test]
fn literal_source_offset_workspace_fails_closed_on_unsupported_inputs() {
    let invalid_root = std::env::temp_dir().join(format!(
        "mizar_literal_offset_fail_closed_{}",
        std::process::id()
    ));
    remove_dir_if_exists(&invalid_root);
    create_dir(&invalid_root.join("crates/demo/src"));
    write_test_file(
        &invalid_root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/demo\"]\n",
    );
    write_test_file(
        &invalid_root.join("crates/demo/Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write_test_file(
        &invalid_root.join("crates/demo/src/lib.rs"),
        "#[cfg_attr(feature = \"maybe\", path = \"alternate.rs\")] mod selected;",
    );
    write_test_file(
        &invalid_root.join("crates/demo/src/alternate.rs"),
        "fn alternate() {}",
    );
    write_test_file(
        &invalid_root.join("crates/demo/src/selected.rs"),
        "fn selected() {}",
    );
    assert!(literal_source_offset_counts(&invalid_root).is_err());
    write_test_file(&invalid_root.join("crates/demo/src/lib.rs"), "fn broken( {");
    assert!(literal_source_offset_counts(&invalid_root).is_err());
    write_test_file(&invalid_root.join("crates/demo/src/lib.rs"), "mod missing;");
    assert!(literal_source_offset_counts(&invalid_root).is_err());
    write_test_file(
        &invalid_root.join("crates/demo/production_payload.rs"),
        "fn f() { let _ = SourceRange::new(id, 1, 2); }",
    );
    write_test_file(
        &invalid_root.join("crates/demo/src/lib.rs"),
        "include!(\"../production_payload.rs\");",
    );
    assert!(literal_source_offset_counts(&invalid_root).is_err());
    write_test_file(
        &invalid_root.join("crates/demo/src/lib.rs"),
        "std::include!(\"../production_payload.rs\");",
    );
    assert!(literal_source_offset_counts(&invalid_root).is_err());

    write_test_file(
        &invalid_root.join("crates/demo/test_payload.rs"),
        "fn f() { let _ = SourceRange::new(id, 3, 4); }",
    );
    write_test_file(
        &invalid_root.join("crates/demo/src/lib.rs"),
        "#[cfg(test)] include!(\"../test_payload.rs\");",
    );
    let test_only_include = literal_source_offset_counts(&invalid_root).unwrap();
    assert_eq!(test_only_include.get("crates/demo/src/lib.rs"), Some(&0));
    assert!(!test_only_include.contains_key("crates/demo/test_payload.rs"));
    write_test_file(
        &invalid_root.join("crates/demo/Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[[bin]]\nname = \"bad-root\"\npath = [\"src/main.rs\"]\n",
    );
    write_test_file(
        &invalid_root.join("crates/demo/src/main.rs"),
        "fn main() { let _ = SourceRange::new(id, 5, 6); }",
    );
    assert!(literal_source_offset_counts(&invalid_root).is_err());
    remove_dir_if_exists(&invalid_root);
}

#[test]
fn literal_source_offset_baseline_vectors_and_repository_ratchet() {
    let actual = BTreeMap::from([("crates/demo/src/lib.rs".to_owned(), 3)]);
    assert!(
        literal_source_offset_baseline_violations(
            "path	count\ncrates/demo/src/lib.rs	3\n",
            &actual
        )
        .is_empty()
    );
    assert!(
        literal_source_offset_baseline_violations(
            "path	count\ncrates/demo/src/lib.rs	4\n",
            &actual
        )
        .is_empty()
    );
    let zero_file = BTreeMap::from([
        ("crates/demo/src/lib.rs".to_owned(), 3),
        ("crates/demo/src/new.rs".to_owned(), 0),
    ]);
    assert!(
        literal_source_offset_baseline_violations(
            "path	count\ncrates/demo/src/lib.rs	3\n",
            &zero_file
        )
        .is_empty()
    );
    assert!(
        literal_source_offset_baseline_violations(
            "path	count\n",
            &BTreeMap::from([("crates/demo/src/new.rs".to_owned(), 0),])
        )
        .is_empty()
    );
    assert!(
        literal_source_offset_baseline_violations(
            "path	count\ncrates/demo/src/new.rs	1\n",
            &BTreeMap::from([("crates/demo/src/new.rs".to_owned(), 0),])
        )
        .is_empty()
    );

    for (baseline, current) in [
        (
            "path	count\ncrates/demo/src/lib.rs	3\n",
            BTreeMap::from([("crates/demo/src/lib.rs".to_owned(), 4)]),
        ),
        (
            "path	count\ncrates/demo/src/lib.rs	1\n",
            BTreeMap::from([
                ("crates/demo/src/lib.rs".to_owned(), 1),
                ("crates/demo/src/new.rs".to_owned(), 1),
            ]),
        ),
        (
            "path	count\ncrates/old/src/lib.rs	1\n",
            BTreeMap::from([("crates/demo/src/lib.rs".to_owned(), 1)]),
        ),
        (
            "path	count\ncrates/demo/src/lib.rs	0\n",
            BTreeMap::from([("crates/demo/src/lib.rs".to_owned(), 0)]),
        ),
        (
            "path	count\ncrates/demo/src/lib.rs	bad\n",
            BTreeMap::from([("crates/demo/src/lib.rs".to_owned(), 1)]),
        ),
        (
            "path	count\ncrates/demo/src/lib.rs	1	extra\n",
            BTreeMap::from([("crates/demo/src/lib.rs".to_owned(), 1)]),
        ),
        (
            "path	count\ncrates/demo/src/lib.rs	1\ncrates/demo/src/lib.rs	2\n",
            BTreeMap::from([("crates/demo/src/lib.rs".to_owned(), 2)]),
        ),
        (
            "path	count\ncrates/demo/src/z.rs	1\ncrates/demo/src/a.rs	1\n",
            BTreeMap::from([
                ("crates/demo/src/a.rs".to_owned(), 1),
                ("crates/demo/src/z.rs".to_owned(), 1),
            ]),
        ),
        ("wrong	count\n", BTreeMap::new()),
        ("path	count\n\n", BTreeMap::new()),
        ("path	count\n# comment\n", BTreeMap::new()),
    ] {
        assert!(
            !literal_source_offset_baseline_violations(baseline, &current).is_empty(),
            "invalid, stale, unsorted, or over-budget baseline was accepted: {baseline:?}"
        );
    }

    let workspace = workspace_root();
    let baseline =
        read_to_string(&workspace.join("tests/coverage/literal_source_offset_baseline.tsv"));
    let current = literal_source_offset_counts(&workspace)
        .unwrap_or_else(|error| panic!("production source scan failed: {error}"));
    let violations = literal_source_offset_baseline_violations(&baseline, &current);
    assert!(
        violations.is_empty(),
        "production literal source offsets exceed their reviewed ceilings:\n{}",
        violations.join("\n")
    );
}

fn literal_source_offset_count(source: &str) -> Result<usize, String> {
    let file = syn::parse_file(source).map_err(|error| error.to_string())?;
    let mut counter = LiteralOffsetCounter::default();
    if literal_offset_configuration(&file.attrs)?.0 {
        counter.module(&file.items);
    }
    counter.error.map_or(Ok(counter.count), Err)
}

fn literal_offset_ident(ident: &syn::Ident) -> String {
    use syn::ext::IdentExt;
    ident.unraw().to_string()
}

fn literal_offset_path_is(path: &syn::Path, name: &str) -> bool {
    path.get_ident()
        .is_some_and(|ident| literal_offset_ident(ident) == name)
}

macro_rules! literal_offset_attrs {
    ($node:expr, $kind:ident, $($variant:ident),+) => {
        match $node { $($kind::$variant(node) => &node.attrs,)+ _ => &[] }
    };
}

fn literal_offset_cfg(meta: &syn::Meta) -> Result<Option<bool>, String> {
    use syn::{Meta, Token, punctuated::Punctuated};
    match meta {
        Meta::Path(path) => Ok(if literal_offset_path_is(path, "test") {
            Some(false)
        } else {
            None
        }),
        Meta::NameValue(_) => Ok(None),
        Meta::List(list) => {
            let values = list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .map_err(|error| error.to_string())?;
            let values = values
                .iter()
                .map(literal_offset_cfg)
                .collect::<Result<Vec<_>, _>>()?;
            if literal_offset_path_is(&list.path, "not") && values.len() == 1 {
                return Ok(values[0].map(|value| !value));
            }
            let all = literal_offset_path_is(&list.path, "all");
            if !all && !literal_offset_path_is(&list.path, "any") {
                return Err("unsupported cfg predicate".into());
            }
            Ok(if values.contains(&Some(!all)) {
                Some(!all)
            } else if values.contains(&None) {
                None
            } else {
                Some(all)
            })
        }
    }
}

fn literal_offset_configuration(
    attrs: &[syn::Attribute],
) -> Result<(bool, Option<String>), String> {
    fn apply(
        meta: &syn::Meta,
        enabled: &mut bool,
        path: &mut Option<String>,
        uncertain: bool,
    ) -> Result<(), String> {
        use syn::{Meta, Token, punctuated::Punctuated};
        if literal_offset_path_is(meta.path(), "cfg_attr") {
            let Meta::List(list) = meta else {
                return Err("invalid cfg_attr".into());
            };
            let args = list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .map_err(|error| error.to_string())?;
            let mut args = args.iter();
            let condition = literal_offset_cfg(args.next().ok_or("missing cfg_attr condition")?)?;
            if condition != Some(false) {
                for attr in args {
                    apply(attr, enabled, path, uncertain || condition.is_none())?;
                }
            }
        } else if literal_offset_path_is(meta.path(), "cfg") {
            if uncertain {
                return Err("unknown cfg_attr affects cfg".into());
            }
            let Meta::List(list) = meta else {
                return Err("invalid cfg".into());
            };
            *enabled &= literal_offset_cfg(
                &list
                    .parse_args::<Meta>()
                    .map_err(|error| error.to_string())?,
            )? != Some(false);
        } else if literal_offset_path_is(meta.path(), "path") {
            if uncertain {
                return Err("unknown cfg_attr affects path".into());
            }
            let Meta::NameValue(value) = meta else {
                return Err("invalid path attribute".into());
            };
            let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(value),
                ..
            }) = &value.value
            else {
                return Err("unsupported module path attribute".into());
            };
            if path.replace(value.value()).is_some() {
                return Err("duplicate path attribute".into());
            }
        }
        Ok(())
    }
    let mut enabled = true;
    let mut path = None;
    for attr in attrs {
        apply(&attr.meta, &mut enabled, &mut path, false)?;
    }
    Ok((enabled, path))
}

fn literal_offset_unwrap(mut expr: &syn::Expr) -> &syn::Expr {
    loop {
        expr = match expr {
            syn::Expr::Paren(value) => &value.expr,
            syn::Expr::Group(value) => &value.expr,
            _ => return expr,
        };
    }
}
fn literal_offset_expr_attrs(expr: &syn::Expr) -> &[syn::Attribute] {
    use syn::Expr;
    literal_offset_attrs!(
        expr, Expr, Array, Assign, Async, Await, Binary, Block, Break, Call, Cast, Closure, Const,
        Continue, Field, ForLoop, Group, If, Index, Infer, Let, Lit, Loop, Macro, Match,
        MethodCall, Paren, Path, Range, RawAddr, Reference, Repeat, Return, Struct, Try, TryBlock,
        Tuple, Unary, Unsafe, While, Yield
    )
}
fn literal_offset_integer(expr: &syn::Expr) -> bool {
    if !literal_offset_configuration(literal_offset_expr_attrs(expr))
        .is_ok_and(|(enabled, _)| enabled)
    {
        return false;
    }
    match expr {
        syn::Expr::Paren(value) => literal_offset_integer(&value.expr),
        syn::Expr::Group(value) => literal_offset_integer(&value.expr),
        syn::Expr::Lit(value) => {
            matches!(&value.lit, syn::Lit::Int(value) if matches!(value.suffix(), "" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"))
        }
        syn::Expr::Unary(value) if matches!(value.op, syn::UnOp::Neg(_)) => {
            literal_offset_integer(&value.expr)
        }
        _ => false,
    }
}

fn literal_offset_binding(pat: &syn::Pat) -> Option<String> {
    match pat {
        syn::Pat::Ident(value) if value.by_ref.is_none() && value.subpat.is_none() => {
            Some(literal_offset_ident(&value.ident))
        }
        syn::Pat::Type(value) => literal_offset_binding(&value.pat),
        _ => None,
    }
}
fn literal_offset_bindings(pat: &syn::Pat, names: &mut BTreeSet<String>) {
    use syn::Pat;
    match pat {
        Pat::Ident(value) => {
            names.insert(literal_offset_ident(&value.ident));
            if let Some((_, pat)) = &value.subpat {
                literal_offset_bindings(pat, names);
            }
        }
        Pat::Type(value) => literal_offset_bindings(&value.pat, names),
        Pat::Reference(value) => literal_offset_bindings(&value.pat, names),
        Pat::Paren(value) => literal_offset_bindings(&value.pat, names),
        Pat::Tuple(value) => {
            for pat in &value.elems {
                literal_offset_bindings(pat, names);
            }
        }
        Pat::TupleStruct(value) => {
            for pat in &value.elems {
                literal_offset_bindings(pat, names);
            }
        }
        Pat::Slice(value) => {
            for pat in &value.elems {
                literal_offset_bindings(pat, names);
            }
        }
        Pat::Or(value) => {
            for pat in &value.cases {
                literal_offset_bindings(pat, names);
            }
        }
        Pat::Struct(value) => {
            for field in &value.fields {
                literal_offset_bindings(&field.pat, names);
            }
        }
        _ => {}
    }
}

#[derive(Default)]
struct LiteralOffsetCounter {
    count: usize,
    error: Option<String>,
    helpers: BTreeMap<String, Vec<usize>>,
    shadows: Vec<BTreeMap<String, bool>>,
}
impl LiteralOffsetCounter {
    fn enabled(&mut self, attrs: &[syn::Attribute]) -> bool {
        match literal_offset_configuration(attrs) {
            Ok((enabled, _)) => enabled && self.error.is_none(),
            Err(error) => {
                self.error = Some(error);
                false
            }
        }
    }
    fn module(&mut self, items: &[syn::Item]) {
        use syn::{Expr, Item, ReturnType, Stmt, Type, visit::Visit};
        let outer = std::mem::take(&mut self.helpers);
        let shadows = std::mem::take(&mut self.shadows);
        for item in items {
            let Item::Fn(function) = item else {
                continue;
            };
            if !self.enabled(&function.attrs) {
                continue;
            }
            let ReturnType::Type(_, result) = &function.sig.output else {
                continue;
            };
            let Type::Path(result) = &**result else {
                continue;
            };
            if result
                .path
                .segments
                .last()
                .is_none_or(|part| literal_offset_ident(&part.ident) != "SourceRange")
                || function.block.stmts.len() != 1
            {
                continue;
            }
            let Stmt::Expr(body, semi) = &function.block.stmts[0] else {
                continue;
            };
            let body = literal_offset_unwrap(body);
            let body = if let Expr::Return(value) = body {
                let Some(value) = &value.expr else {
                    continue;
                };
                literal_offset_unwrap(value)
            } else {
                if semi.is_some() {
                    continue;
                }
                body
            };
            let Expr::Struct(body) = body else {
                continue;
            };
            if body
                .path
                .segments
                .last()
                .is_none_or(|part| literal_offset_ident(&part.ident) != "SourceRange")
            {
                continue;
            }
            let params = function
                .sig
                .inputs
                .iter()
                .map(|arg| match arg {
                    syn::FnArg::Typed(arg) => match &*arg.pat {
                        syn::Pat::Ident(value) => Some(literal_offset_ident(&value.ident)),
                        _ => None,
                    },
                    _ => None,
                })
                .collect::<Vec<_>>();
            let mut endpoints = [None, None];
            for field in &body.fields {
                let endpoint = match &field.member {
                    syn::Member::Named(name) if literal_offset_ident(name) == "start" => 0,
                    syn::Member::Named(name) if literal_offset_ident(name) == "end" => 1,
                    _ => continue,
                };
                if let Expr::Path(value) = literal_offset_unwrap(&field.expr)
                    && let Some(name) = value.path.get_ident()
                    && let Some(index) = params
                        .iter()
                        .position(|param| param.as_deref() == Some(&literal_offset_ident(name)))
                {
                    endpoints[endpoint] = Some(index);
                }
            }
            if let [Some(start), Some(end)] = endpoints {
                self.helpers
                    .entry(literal_offset_ident(&function.sig.ident))
                    .or_default()
                    .extend([start, end]);
            }
        }
        for item in items {
            self.visit_item(item);
        }
        self.helpers = outer;
        self.shadows = shadows;
    }
    fn function(&mut self, signature: &syn::Signature, block: &syn::Block) {
        use syn::visit::Visit;
        self.shadows.push(BTreeMap::new());
        for input in &signature.inputs {
            if let syn::FnArg::Typed(input) = input {
                self.bind(&input.pat);
            }
        }
        self.visit_block(block);
        self.shadows.pop();
    }
    fn bind(&mut self, pat: &syn::Pat) {
        if let Some(scope) = self.shadows.last_mut() {
            let mut names = BTreeSet::new();
            literal_offset_bindings(pat, &mut names);
            scope.extend(names.into_iter().map(|name| (name, true)));
        }
    }
    fn named_value(&mut self, expr: &syn::Expr) {
        use syn::{Expr, Stmt};
        if !self.enabled(literal_offset_expr_attrs(expr)) {
            return;
        }
        match expr {
            Expr::Paren(value) => self.named_value(&value.expr),
            Expr::Group(value) => self.named_value(&value.expr),
            Expr::Array(array) => {
                self.count += array
                    .elems
                    .iter()
                    .filter(|element| {
                        let mut element = *element;
                        loop {
                            if !literal_offset_configuration(literal_offset_expr_attrs(element))
                                .is_ok_and(|(enabled, _)| enabled)
                            {
                                return false;
                            }
                            match element {
                                Expr::Paren(value) => element = &value.expr,
                                Expr::Group(value) => element = &value.expr,
                                Expr::Tuple(tuple) => {
                                    return tuple.elems.iter().any(literal_offset_integer);
                                }
                                _ => return literal_offset_integer(element),
                            }
                        }
                    })
                    .count()
            }
            Expr::Reference(value) => self.named_value(&value.expr),
            Expr::Block(value) => {
                if let Some(Stmt::Expr(tail, None)) = value.block.stmts.last() {
                    self.named_value(tail);
                }
            }
            Expr::If(value) => {
                if let Some(Stmt::Expr(tail, None)) = value.then_branch.stmts.last() {
                    self.named_value(tail);
                }
                if let Some((_, branch)) = &value.else_branch {
                    self.named_value(branch);
                }
            }
            Expr::Match(value) => {
                for arm in &value.arms {
                    if self.enabled(&arm.attrs) {
                        self.named_value(&arm.body);
                    }
                }
            }
            Expr::Macro(value) if literal_offset_path_is(&value.mac.path, "vec") => {
                match syn::parse_str::<Expr>(&format!("[{}]", value.mac.tokens)) {
                    Ok(value) => self.named_value(&value),
                    Err(error) => self.error = Some(error.to_string()),
                }
            }
            Expr::Macro(value)
                if literal_offset_path_is(&value.mac.path, "unreachable")
                    && (value.mac.tokens.is_empty()
                        || syn::parse2::<syn::LitStr>(value.mac.tokens.clone()).is_ok()) => {}
            Expr::Macro(_) | Expr::Repeat(_) => {
                self.error = Some("opaque macro or repeat in named array value".into())
            }
            _ => {}
        }
    }
    fn macro_targets(&self, mac: &syn::Macro) -> Result<bool, syn::Error> {
        use syn::{ext::IdentExt, parse::Parser};
        fn scan(
            input: syn::parse::ParseStream<'_>,
            helpers: &BTreeMap<String, Vec<usize>>,
        ) -> syn::Result<bool> {
            let mut found = false;
            while !input.is_empty() {
                if input.peek(syn::token::Paren) {
                    let inside;
                    syn::parenthesized!(inside in input);
                    found |= scan(&inside, helpers)?;
                } else if input.peek(syn::token::Brace) {
                    let inside;
                    syn::braced!(inside in input);
                    found |= scan(&inside, helpers)?;
                } else if input.peek(syn::token::Bracket) {
                    let inside;
                    syn::bracketed!(inside in input);
                    found |= scan(&inside, helpers)?;
                } else if input.peek(syn::Ident::peek_any) {
                    let ident = input.call(syn::Ident::parse_any)?;
                    let raw_keyword = ident.to_string().starts_with("r#");
                    let name = literal_offset_ident(&ident);
                    let look = input.fork();
                    if name == "SourceRange" {
                        found |= look.peek(syn::token::Brace);
                        if look.parse::<syn::Token![::]>().is_ok()
                            && look
                                .call(syn::Ident::parse_any)
                                .is_ok_and(|name| literal_offset_ident(&name) == "new")
                        {
                            found |= look.peek(syn::token::Paren);
                        }
                    }
                    found |= helpers.contains_key(&name) && input.peek(syn::token::Paren);
                    if name == "let" && !raw_keyword {
                        let _ = look.parse::<syn::Token![mut]>();
                        if look.call(syn::Ident::parse_any).is_ok_and(|name| {
                            matches!(
                                literal_offset_ident(&name).as_str(),
                                "expected_ranges" | "expected_sites"
                            )
                        }) {
                            found |= look.peek(syn::Token![=]) || look.peek(syn::Token![:]);
                        }
                    }
                } else {
                    input.step(|cursor| {
                        cursor
                            .token_tree()
                            .map(|(_, next)| ((), next))
                            .ok_or_else(|| cursor.error("invalid macro tokens"))
                    })?;
                }
            }
            Ok(found)
        }
        (|input: syn::parse::ParseStream<'_>| scan(input, &self.helpers)).parse2(mac.tokens.clone())
    }
}

impl<'ast> syn::visit::Visit<'ast> for LiteralOffsetCounter {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        use syn::{Item, visit};
        if matches!(item, Item::Verbatim(_)) {
            self.error = Some("unsupported item syntax".into());
            return;
        }
        if self.enabled(literal_offset_attrs!(
            item,
            Item,
            Const,
            Enum,
            ExternCrate,
            Fn,
            ForeignMod,
            Impl,
            Macro,
            Mod,
            Static,
            Struct,
            Trait,
            TraitAlias,
            Type,
            Union,
            Use
        )) {
            visit::visit_item(self, item);
        }
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if let Some((_, items)) = &item.content {
            self.module(items);
        }
    }
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.function(&item.sig, &item.block);
    }
    fn visit_impl_item(&mut self, item: &'ast syn::ImplItem) {
        use syn::{ImplItem, visit};
        if matches!(item, ImplItem::Verbatim(_)) {
            self.error = Some("unsupported item syntax".into());
            return;
        }
        if self.enabled(literal_offset_attrs!(
            item, ImplItem, Const, Fn, Type, Macro
        )) {
            if let ImplItem::Fn(item) = item {
                self.function(&item.sig, &item.block);
            } else {
                visit::visit_impl_item(self, item);
            }
        }
    }
    fn visit_trait_item(&mut self, item: &'ast syn::TraitItem) {
        use syn::{TraitItem, visit};
        if matches!(item, TraitItem::Verbatim(_)) {
            self.error = Some("unsupported item syntax".into());
            return;
        }
        if self.enabled(literal_offset_attrs!(
            item, TraitItem, Const, Fn, Type, Macro
        )) {
            if let TraitItem::Fn(item) = item {
                if let Some(block) = &item.default {
                    self.function(&item.sig, block);
                }
            } else {
                visit::visit_trait_item(self, item);
            }
        }
    }
    fn visit_stmt_macro(&mut self, statement: &'ast syn::StmtMacro) {
        if self.enabled(&statement.attrs) {
            self.visit_macro(&statement.mac);
        }
    }
    fn visit_block(&mut self, block: &'ast syn::Block) {
        fn imports(tree: &syn::UseTree, names: &mut BTreeMap<String, bool>) {
            match tree {
                syn::UseTree::Name(name) => {
                    names.insert(literal_offset_ident(&name.ident), true);
                }
                syn::UseTree::Rename(name) => {
                    names.insert(literal_offset_ident(&name.rename), true);
                }
                syn::UseTree::Path(path) => imports(&path.tree, names),
                syn::UseTree::Group(group) => {
                    for item in &group.items {
                        imports(item, names);
                    }
                }
                syn::UseTree::Glob(_) => {
                    names.insert("*".into(), true);
                }
            }
        }
        let mut names = BTreeMap::new();
        for statement in &block.stmts {
            if let syn::Stmt::Item(item) = statement {
                match item {
                    syn::Item::Fn(item) if self.enabled(&item.attrs) => {
                        let ambiguous = matches!(&item.sig.output, syn::ReturnType::Type(_, result) if matches!(&**result, syn::Type::Path(path) if path.path.segments.last().is_some_and(|part| literal_offset_ident(&part.ident) == "SourceRange")));
                        names
                            .entry(literal_offset_ident(&item.sig.ident))
                            .and_modify(|possible| *possible |= ambiguous)
                            .or_insert(ambiguous);
                    }
                    syn::Item::Use(item) if self.enabled(&item.attrs) => {
                        imports(&item.tree, &mut names)
                    }
                    syn::Item::Const(item) if self.enabled(&item.attrs) => {
                        names.insert(literal_offset_ident(&item.ident), true);
                    }
                    syn::Item::Static(item) if self.enabled(&item.attrs) => {
                        names.insert(literal_offset_ident(&item.ident), true);
                    }
                    syn::Item::Struct(item)
                        if self.enabled(&item.attrs)
                            && matches!(item.fields, syn::Fields::Unnamed(_)) =>
                    {
                        names
                            .entry(literal_offset_ident(&item.ident))
                            .or_insert(false);
                    }
                    _ => {}
                }
            }
        }
        self.shadows.push(names);
        syn::visit::visit_block(self, block);
        self.shadows.pop();
    }
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if !self.enabled(&local.attrs) {
            return;
        }
        if let Some(init) = &local.init {
            if literal_offset_binding(&local.pat)
                .is_some_and(|name| name == "expected_ranges" || name == "expected_sites")
            {
                self.named_value(&init.expr);
            }
            self.visit_expr(&init.expr);
            if let Some((_, diverge)) = &init.diverge {
                self.visit_expr(diverge);
            }
        }
        self.bind(&local.pat);
    }
    fn visit_field_value(&mut self, field: &'ast syn::FieldValue) {
        if self.enabled(&field.attrs) {
            self.visit_expr(&field.expr);
        }
    }
    fn visit_expr(&mut self, expr: &'ast syn::Expr) {
        use syn::{Expr, visit};
        if !self.enabled(literal_offset_expr_attrs(expr)) {
            return;
        }
        match expr {
            Expr::Struct(value) if value.path.segments.last().is_some_and(|part| literal_offset_ident(&part.ident) == "SourceRange")
                && value.fields.iter().any(|field| {
                    literal_offset_configuration(&field.attrs).is_ok_and(|(enabled, _)| enabled)
                        && matches!(&field.member, syn::Member::Named(name) if matches!(literal_offset_ident(name).as_str(), "start" | "end"))
                        && literal_offset_integer(&field.expr)
                }) => {
                self.count += 1;
            }
            Expr::Call(call) => {
                if let Expr::Path(path) = literal_offset_unwrap(&call.func) {
                    let segments = path.path.segments.iter().map(|part| literal_offset_ident(&part.ident)).collect::<Vec<_>>();
                    if segments.ends_with(&["SourceRange".into(), "new".into()]) {
                        if call.args.len() == 3 && call.args.iter().skip(1).any(literal_offset_integer) {
                            self.count += 1;
                        }
                    } else if path.qself.is_none() && path.path.leading_colon.is_none()
                        && (segments.len() == 1 || (segments.len() == 2 && segments[0] == "self")) {
                        let name = segments.last().unwrap();
                        if let Some(positions) = self.helpers.get(name) {
                            let shadow = if segments.len() == 1 {
                                self.shadows.iter().rev().find_map(|scope| scope.get(name).or_else(|| scope.get("*")))
                            } else { None };
                            if shadow == Some(&true) {
                                self.error = Some(format!("ambiguous shadowed range helper call: {name}"));
                            } else if shadow.is_none() && positions.iter().any(|index| call.args.iter().nth(*index).is_some_and(literal_offset_integer)) {
                                self.count += 1;
                            }
                        }
                    }
                }
            }
            Expr::Let(value) => {
                self.visit_expr(&value.expr);
                self.bind(&value.pat);
                return;
            }
            Expr::If(value) => {
                self.shadows.push(BTreeMap::new());
                self.visit_expr(&value.cond);
                self.visit_block(&value.then_branch);
                self.shadows.pop();
                if let Some((_, branch)) = &value.else_branch { self.visit_expr(branch); }
                return;
            }
            Expr::While(value) => {
                self.shadows.push(BTreeMap::new());
                self.visit_expr(&value.cond);
                self.visit_block(&value.body);
                self.shadows.pop();
                return;
            }
            Expr::Closure(value) => {
                self.shadows.push(BTreeMap::new());
                for pat in &value.inputs { self.bind(pat); }
                self.visit_expr(&value.body);
                self.shadows.pop();
                return;
            }
            Expr::ForLoop(value) => {
                self.visit_expr(&value.expr);
                self.shadows.push(BTreeMap::new());
                self.bind(&value.pat);
                self.visit_block(&value.body);
                self.shadows.pop();
                return;
            }
            Expr::Match(value) => {
                self.visit_expr(&value.expr);
                for arm in &value.arms {
                    if self.enabled(&arm.attrs) {
                        self.shadows.push(BTreeMap::new());
                        self.bind(&arm.pat);
                        if let Some((_, guard)) = &arm.guard { self.visit_expr(guard); }
                        self.visit_expr(&arm.body);
                        self.shadows.pop();
                    }
                }
                return;
            }
            Expr::Verbatim(_) => { self.error = Some("unsupported expression syntax".into()); return; }
            _ => {}
        }
        visit::visit_expr(self, expr);
    }
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        use syn::{Expr, Pat, Token, parse::Parser, punctuated::Punctuated};
        let name = literal_offset_ident(&mac.path.segments.last().unwrap().ident);
        if name == "include" {
            self.error = Some("production include! is unsupported".into());
            return;
        }
        let parsed = if name == "vec" {
            syn::parse_str::<Expr>(&format!("[{}]", mac.tokens)).map(|value| vec![value])
        } else if name == "matches" {
            (|input: syn::parse::ParseStream<'_>| {
                let value = input.parse::<Expr>()?;
                input.parse::<Token![,]>()?;
                input.call(Pat::parse_multi_with_leading_vert)?;
                let mut values = vec![value];
                if input.peek(Token![if]) {
                    input.parse::<Token![if]>()?;
                    values.push(input.parse::<Expr>()?);
                }
                let _ = input.parse::<Token![,]>();
                Ok(values)
            })
            .parse2(mac.tokens.clone())
        } else if matches!(
            name.as_str(),
            "assert"
                | "assert_eq"
                | "assert_ne"
                | "debug_assert"
                | "debug_assert_eq"
                | "debug_assert_ne"
                | "format"
                | "format_args"
                | "print"
                | "println"
                | "eprint"
                | "eprintln"
                | "write"
                | "writeln"
        ) {
            Punctuated::<Expr, Token![,]>::parse_terminated
                .parse2(mac.tokens.clone())
                .map(|values| values.into_iter().collect())
        } else {
            if self.macro_targets(mac).unwrap_or(true) {
                self.error = Some(format!("unsupported target-bearing macro: {name}"));
            }
            return;
        };
        match parsed {
            Ok(values) => {
                for value in &values {
                    self.visit_expr(value);
                }
            }
            Err(error) => {
                if self.macro_targets(mac).unwrap_or(true) {
                    self.error = Some(format!("malformed target-bearing macro {name}: {error}"));
                }
            }
        }
    }
}

fn literal_source_offset_counts(workspace: &Path) -> Result<BTreeMap<String, usize>, String> {
    fn derive_bit(name: &syn::Ident) -> u16 {
        match literal_offset_ident(name).as_str() {
            "Debug" => 1,
            "Clone" => 2,
            "Copy" => 4,
            "PartialEq" => 8,
            "Eq" => 16,
            "PartialOrd" => 32,
            "Ord" => 64,
            "Hash" => 128,
            "Default" => 256,
            _ => 0,
        }
    }
    fn import_hazards(tree: &syn::UseTree) -> (bool, u16) {
        match tree {
            syn::UseTree::Glob(_) => (true, 0),
            syn::UseTree::Name(name) => (
                literal_offset_ident(&name.ident) == "include",
                derive_bit(&name.ident),
            ),
            syn::UseTree::Rename(rename) => (
                literal_offset_ident(&rename.rename) == "include",
                derive_bit(&rename.rename),
            ),
            syn::UseTree::Path(path) => import_hazards(&path.tree),
            syn::UseTree::Group(group) => {
                group.items.iter().fold((false, 0), |(bad, bits), item| {
                    let (item_bad, item_bits) = import_hazards(item);
                    (bad || item_bad, bits | item_bits)
                })
            }
        }
    }
    fn attribute_hazards(attrs: &[syn::Attribute]) -> (bool, u16) {
        let mut bad = false;
        let mut derives = 0;
        for attr in attrs {
            let path = attr.path();
            if literal_offset_path_is(path, "derive") {
                let syn::Meta::List(list) = &attr.meta else {
                    return (true, derives);
                };
                let parsed = syn::parse::Parser::parse2(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                    list.tokens.clone(),
                );
                match parsed {
                    Ok(paths) => {
                        for derive in paths {
                            let bit = derive.get_ident().map_or(0, derive_bit);
                            bad |= bit == 0;
                            derives |= bit;
                        }
                    }
                    Err(_) => bad = true,
                }
                continue;
            }
            let ordinary = matches!(
                path.get_ident().map(literal_offset_ident).as_deref(),
                Some(
                    "cfg"
                        | "path"
                        | "test"
                        | "allow"
                        | "inline"
                        | "track_caller"
                        | "must_use"
                        | "non_exhaustive"
                        | "default"
                        | "macro_export"
                        | "doc"
                )
            );
            let rustfmt_skip = path.segments.len() == 2
                && literal_offset_ident(&path.segments[0].ident) == "rustfmt"
                && literal_offset_ident(&path.segments[1].ident) == "skip";
            bad |= !(ordinary || rustfmt_skip);
        }
        (bad, derives)
    }
    fn include_scope(
        items: &[syn::Item],
        physical: &Path,
        possible: bool,
        visiting: &mut BTreeSet<PathBuf>,
        includes: &mut Vec<(PathBuf, bool)>,
        imports: &mut u16,
        derives: &mut u16,
    ) -> Result<bool, String> {
        let mut clean = true;
        for item in items {
            let attrs = match item {
                syn::Item::Const(v) => &v.attrs,
                syn::Item::Enum(v) => &v.attrs,
                syn::Item::ExternCrate(v) => &v.attrs,
                syn::Item::Fn(v) => &v.attrs,
                syn::Item::ForeignMod(v) => &v.attrs,
                syn::Item::Impl(v) => &v.attrs,
                syn::Item::Macro(v) => &v.attrs,
                syn::Item::Mod(v) => &v.attrs,
                syn::Item::Static(v) => &v.attrs,
                syn::Item::Struct(v) => &v.attrs,
                syn::Item::Trait(v) => &v.attrs,
                syn::Item::TraitAlias(v) => &v.attrs,
                syn::Item::Type(v) => &v.attrs,
                syn::Item::Union(v) => &v.attrs,
                syn::Item::Use(v) => &v.attrs,
                _ => {
                    clean = false;
                    continue;
                }
            };
            let (bad, used) = attribute_hazards(attrs);
            clean &= !bad;
            *derives |= used;
            match item {
                syn::Item::Use(item) => {
                    let (bad, names) = import_hazards(&item.tree);
                    clean &= !bad;
                    *imports |= names;
                }
                syn::Item::ExternCrate(_) => clean = false,
                syn::Item::Macro(item) if literal_offset_path_is(&item.mac.path, "macro_rules") => {
                    clean &= item
                        .ident
                        .as_ref()
                        .is_some_and(|ident| literal_offset_ident(ident) != "include");
                }
                syn::Item::Macro(item) if literal_offset_path_is(&item.mac.path, "include") => {
                    let item_possible = possible && literal_offset_configuration(&item.attrs)?.0;
                    let Ok(path) = syn::parse2::<syn::LitStr>(item.mac.tokens.clone()) else {
                        clean = false;
                        continue;
                    };
                    let path = physical.parent().unwrap().join(path.value());
                    let path = fs::canonicalize(&path)
                        .map_err(|error| format!("{}: {error}", path.display()))?;
                    if !visiting.insert(path.clone()) {
                        return Err(format!("recursive literal include: {}", path.display()));
                    }
                    let source = fs::read_to_string(&path)
                        .map_err(|error| format!("{}: {error}", path.display()))?;
                    let file = syn::parse_file(&source)
                        .map_err(|error| format!("{}: {error}", path.display()))?;
                    let (bad, used) = attribute_hazards(&file.attrs);
                    clean &= !bad;
                    *derives |= used;
                    clean &= include_scope(
                        &file.items,
                        &path,
                        item_possible,
                        visiting,
                        includes,
                        imports,
                        derives,
                    )?;
                    visiting.remove(&path);
                    includes.push((path, item_possible));
                }
                syn::Item::Macro(_) => clean = false,
                _ => {}
            }
        }
        Ok(clean)
    }
    fn edges(
        items: &[syn::Item],
        physical: &Path,
        directory: &Path,
        path_directory: &Path,
        possible: bool,
        scope: (bool, u16, u16),
        result: &mut Vec<(PathBuf, bool, bool, bool, bool, u16, u16)>,
    ) -> Result<(), String> {
        let mut includes = Vec::new();
        let (inherited_clean, inherited_imports, inherited_derives) = scope;
        let mut imports = inherited_imports;
        let mut derives = inherited_derives;
        let clean = inherited_clean
            && include_scope(
                items,
                physical,
                possible,
                &mut BTreeSet::new(),
                &mut includes,
                &mut imports,
                &mut derives,
            )?;
        for (path, possible) in includes {
            result.push((path, possible, true, clean, true, imports, derives));
        }
        for item in items {
            let syn::Item::Mod(module) = item else {
                continue;
            };
            let (enabled, explicit) = literal_offset_configuration(&module.attrs)?;
            let possible = possible && enabled;
            if let Some((_, items)) = &module.content {
                let nested = explicit.map_or_else(
                    || directory.join(literal_offset_ident(&module.ident)),
                    |path| path_directory.join(path),
                );
                edges(
                    items,
                    physical,
                    &nested,
                    &nested,
                    possible,
                    (clean, imports, derives),
                    result,
                )?;
            } else {
                let path_loaded = explicit.is_some();
                let path = if let Some(explicit) = explicit {
                    path_directory.join(explicit)
                } else {
                    let simple =
                        directory.join(format!("{}.rs", literal_offset_ident(&module.ident)));
                    let nested = directory
                        .join(literal_offset_ident(&module.ident))
                        .join("mod.rs");
                    match (simple.exists(), nested.exists()) {
                        (true, false) => simple,
                        (false, true) => nested,
                        (false, false) => simple,
                        _ => return Err(format!("ambiguous module {}", simple.display())),
                    }
                };
                if path.exists() {
                    result.push((
                        fs::canonicalize(&path).map_err(|error| error.to_string())?,
                        possible,
                        path_loaded,
                        clean,
                        false,
                        imports,
                        derives,
                    ));
                } else {
                    result.push((path, possible, path_loaded, clean, false, imports, derives));
                }
            }
        }
        Ok(())
    }
    let workspace = fs::canonicalize(workspace).map_err(|error| error.to_string())?;
    let mut sources = Vec::new();
    let mut roots = BTreeSet::new();
    for entry in fs::read_dir(workspace.join("crates")).map_err(|error| error.to_string())? {
        let crate_path = entry.map_err(|error| error.to_string())?.path();
        if !crate_path.join("Cargo.toml").exists() {
            continue;
        }
        if crate_path.join("src").exists() {
            collect_rust_files(&crate_path.join("src"), &mut sources);
        }
        for name in ["lib.rs", "main.rs"] {
            let path = crate_path.join("src").join(name);
            if path.exists() {
                roots.insert(fs::canonicalize(path).map_err(|error| error.to_string())?);
            }
        }
        let manifest =
            fs::read_to_string(crate_path.join("Cargo.toml")).map_err(|error| error.to_string())?;
        let mut section = String::new();
        for line in manifest.lines() {
            if let Some(name) = manifest_section_name(line) {
                section = name.to_owned();
            } else if matches!(section.as_str(), "lib" | "bin")
                && line.trim_start().starts_with("path")
            {
                let path =
                    quoted_assignment(line, "path").ok_or("unsupported production root path")?;
                roots.insert(
                    fs::canonicalize(crate_path.join(path)).map_err(|error| error.to_string())?,
                );
            }
        }
    }
    let mut pending = sources
        .iter()
        .chain(roots.iter())
        .map(|path| {
            fs::canonicalize(path)
                .map(|path| (path, false))
                .map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut graph = BTreeMap::new();
    while let Some((path, path_loaded)) = pending.pop() {
        if graph.contains_key(&(path.clone(), path_loaded)) {
            continue;
        }
        if !path.starts_with(&workspace) {
            return Err(format!(
                "unsupported source outside workspace: {}",
                path.display()
            ));
        }
        let source =
            fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        let file =
            syn::parse_file(&source).map_err(|error| format!("{}: {error}", path.display()))?;
        let enabled = literal_offset_configuration(&file.attrs)?.0;
        let (attribute_bad, attribute_derives) = attribute_hazards(&file.attrs);
        let parent = path.parent().unwrap();
        let directory =
            if path_loaded || roots.contains(&path) || path.file_name().unwrap() == "mod.rs" {
                parent.to_path_buf()
            } else {
                parent.join(path.file_stem().unwrap())
            };
        let mut children = Vec::new();
        edges(
            &file.items,
            &path,
            &directory,
            parent,
            enabled,
            (!attribute_bad, 0, attribute_derives),
            &mut children,
        )?;
        pending.extend(
            children
                .iter()
                .filter(|(path, _, _, _, _, _, _)| path.exists())
                .map(|(path, _, path_loaded, _, _, _, _)| (path.clone(), *path_loaded)),
        );
        graph.insert((path, path_loaded), (source, enabled, children));
    }
    let referenced = graph
        .values()
        .flat_map(|(_, _, children)| children.iter().map(|(path, _, _, _, _, _, _)| path.clone()))
        .collect::<BTreeSet<_>>();
    let mut queue = roots
        .into_iter()
        .chain(
            sources
                .into_iter()
                .map(|path| fs::canonicalize(path).unwrap())
                .filter(|path| !referenced.contains(path)),
        )
        .map(|path| ((path, false), true, true, 0_u16, 0_u16))
        .collect::<Vec<_>>();
    let mut reached = BTreeSet::new();
    loop {
        while let Some((path, possible, clean, imports, derives)) = queue.pop() {
            let Some((_, enabled, children)) = graph.get(&path) else {
                if possible {
                    return Err(format!("missing production module {}", path.0.display()));
                }
                continue;
            };
            let possible = possible && *enabled;
            if !reached.insert((path, possible, clean, imports, derives)) {
                continue;
            }
            queue.extend(children.iter().filter_map(
                |(path, edge, path_loaded, scope_clean, include, scope_imports, scope_derives)| {
                    let clean = clean && *scope_clean;
                    let imports = imports | scope_imports;
                    let derives = derives | scope_derives;
                    (!*include || (clean && imports & derives == 0)).then_some((
                        (path.clone(), *path_loaded),
                        possible && *edge,
                        clean,
                        imports,
                        derives,
                    ))
                },
            ));
        }
        let Some(path) = graph.keys().find(|(path, _)| {
            !reached
                .iter()
                .any(|((reached, _), _, _, _, _)| reached == path)
        }) else {
            break;
        };
        queue.push((path.clone(), true, true, 0, 0));
    }
    let mut counts = BTreeMap::new();
    for (path, possible, _, _, _) in &reached {
        if !possible {
            continue;
        }
        let relative = workspace_relative_slash_path(&workspace, &path.0);
        if let std::collections::btree_map::Entry::Vacant(entry) = counts.entry(relative) {
            let count = literal_source_offset_count(&graph[path].0)
                .map_err(|error| format!("{}: {error}", path.0.display()))?;
            entry.insert(count);
        }
    }
    Ok(counts)
}

fn literal_source_offset_baseline_violations(
    baseline: &str,
    actual: &BTreeMap<String, usize>,
) -> Vec<String> {
    let mut violations = Vec::new();
    let mut lines = baseline.lines();
    if lines.next() != Some("path\tcount") {
        return vec!["literal offset baseline requires path<TAB>count header".into()];
    }
    let mut rows = BTreeMap::new();
    let mut previous = "";
    for line in lines {
        let fields = line.split('\t').collect::<Vec<_>>();
        let [path, count] = fields[..] else {
            violations.push(format!("malformed literal offset baseline row: {line:?}"));
            continue;
        };
        let Ok(count) = count.parse::<usize>() else {
            violations.push(format!("invalid literal offset count: {line:?}"));
            continue;
        };
        if count == 0
            || !fields[1].bytes().all(|byte| byte.is_ascii_digit())
            || path <= previous
            || !actual.contains_key(path)
        {
            violations.push(format!(
                "zero, unsorted, duplicate or stale literal offset baseline row: {line:?}"
            ));
        }
        previous = path;
        rows.insert(path, count);
    }
    for (path, count) in actual {
        let ceiling = rows.get(path.as_str()).copied().unwrap_or(0);
        if *count > ceiling {
            violations.push(format!(
                "{path}: {count} literal source offsets exceed ceiling {ceiling}"
            ));
        }
    }
    violations
}
