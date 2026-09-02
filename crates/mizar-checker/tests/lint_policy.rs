use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

#[test]
fn checker_manifest_opts_into_workspace_lints() {
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
fn checker_manifest_keeps_task_one_package_metadata() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let package = section(&manifest, "package");
    let lib = section(&manifest, "lib");

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
            .any(|line| assignment_is(line, "name", "mizar_checker")),
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
fn checker_manifest_dependency_boundary_is_task258b1_scoped() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [
            (
                "dependencies".to_owned(),
                vec![
                    "mizar-lexer = { path = \"../mizar-lexer\" }",
                    "mizar-resolve = { path = \"../mizar-resolve\" }",
                    "mizar-session = { path = \"../mizar-session\" }",
                ],
            ),
            (
                "dev-dependencies".to_owned(),
                vec!["mizar-syntax = { path = \"../mizar-syntax\" }"],
            ),
        ],
        "{} must keep the established production dependencies and only the \
         Task-258B1 test-only syntax dependency until another task-scoped \
         checker spec expands the crate boundary",
        manifest_path.display()
    );
}

#[test]
fn checker_task258b1_syntax_dependency_remains_test_only() {
    let root = crate_root();
    let manifest = read_to_string(&root.join("Cargo.toml"));
    assert!(
        section(&manifest, "dependencies")
            .iter()
            .all(|line| !line.contains("mizar-syntax")),
        "Task-258B1 must not add mizar-syntax to production dependencies"
    );
    assert_eq!(
        section(&manifest, "dev-dependencies"),
        ["mizar-syntax = { path = \"../mizar-syntax\" }"]
    );
    let source = read_to_string(&root.join("src/source_statement.rs"));
    assert_eq!(
        source.matches("use mizar_syntax as syntax;").count(),
        1,
        "only the Task-258B1 checker test fixture may alias mizar-syntax"
    );
}

#[test]
fn checker_source_does_not_import_syntax_directly() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in checker_src_files(&root) {
        let source = read_to_string(&path);
        if source.contains("mizar_syntax::") || source.contains("extern crate mizar_syntax") {
            let display_path = path.strip_prefix(&root).unwrap_or(&path);
            violations.push(display_path.display().to_string());
        }
    }

    assert!(
        violations.is_empty(),
        "mizar-checker must keep checker-local semantic source-shape and \
         binding-env boundaries instead of importing mizar-syntax directly:\n{}",
        violations.join("\n")
    );
}

#[test]
fn overload_collection_stays_on_explicit_payload_boundary() {
    let root = crate_root();
    let path = root.join("src/overload_resolution.rs");
    let source = read_to_string(&path);
    let forbidden = [
        "mizar_resolve::env",
        "SymbolEnv",
        "ResolvedAst",
        "mizar_syntax::",
        "extern crate mizar_syntax",
    ];
    let violations = forbidden
        .iter()
        .copied()
        .filter(|token| source.contains(token))
        .collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "{} must collect explicit overload payloads without resolver-global \
         scans, resolved-AST walks, or direct syntax parsing:\n{}",
        path.display(),
        violations.join("\n")
    );
}

#[test]
fn resolved_typed_ast_stays_on_checker_output_boundary() {
    let root = crate_root();
    let path = root.join("src/resolved_typed_ast.rs");
    let source = read_to_string(&path);
    let forbidden = [
        "mizar_resolve::env",
        "SymbolEnv",
        "ResolvedAst",
        "mizar_syntax::",
        "extern crate mizar_syntax",
        "std::fs::",
        "File::create",
        "OpenOptions",
        "artifact",
        "Artifact",
        "ClusterTrace::",
        "ClusterClosure",
        "fire_cluster",
        "replay",
    ];
    let violations = forbidden
        .iter()
        .copied()
        .filter(|token| source.contains(token))
        .collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "{} must assemble from checker output tables without resolver-global \
         scans, direct syntax parsing, artifact emission, or cluster firing/replay:\n{}",
        path.display(),
        violations.join("\n")
    );
}

#[test]
fn checker_public_semantic_api_matches_documented_modules() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in checker_src_files(&root) {
        let source = read_to_string(&path);
        for (line_index, line) in source.lines().enumerate() {
            if public_declaration_name(line).is_some()
                && !public_checker_api_is_documented(&root, &path, line)
            {
                let display_path = path.strip_prefix(&root).unwrap_or(&path);
                violations.push(format!("{}:{}", display_path.display(), line_index + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "public checker APIs require their owning module spec first:\n{}",
        violations.join("\n")
    );
}

#[test]
fn checker_public_enums_are_forward_compatible_and_documented() {
    let root = crate_root();
    let docs_root = workspace_root().join("doc/design/mizar-checker");
    let modules = [
        ("src/typed_ast.rs", "typed_ast.md"),
        ("src/binding_env.rs", "binding_env.md"),
        ("src/source_context.rs", "source_context.md"),
        ("src/source_type.rs", "source_type.md"),
        ("src/source_attribute.rs", "source_attribute.md"),
        (
            "src/source_attribute_definition.rs",
            "source_attribute_definition.md",
        ),
        ("src/source_evidence.rs", "source_evidence.md"),
        ("src/source_term.rs", "source_term.md"),
        ("src/source_application.rs", "source_application.md"),
        ("src/source_atomic_formula.rs", "source_atomic_formula.md"),
        (
            "src/source_composite_formula.rs",
            "source_composite_formula.md",
        ),
        (
            "src/source_formula_composition.rs",
            "source_formula_composition.md",
        ),
        (
            "src/source_predicate_definition.rs",
            "source_predicate_definition.md",
        ),
        (
            "src/source_functor_definition.rs",
            "source_functor_definition.md",
        ),
        (
            "src/source_property_implementation.rs",
            "source_property_implementation.md",
        ),
        ("src/source_mode_definition.rs", "source_mode_definition.md"),
        ("src/source_set_term.rs", "source_set_term.md"),
        ("src/source_statement.rs", "source_statement.md"),
        (
            "src/source_proof_local_declaration.rs",
            "source_proof_local_declaration.md",
        ),
        ("src/source_structure.rs", "source_structure.md"),
        (
            "src/source_structure_definition.rs",
            "source_structure_definition.md",
        ),
        ("src/source_template.rs", "source_template.md"),
        (
            "src/source_template_type_parameter_association.rs",
            "source_template_type_parameter_association.md",
        ),
        ("src/type_checker.rs", "type_checker.md"),
        (
            "src/registration_resolution.rs",
            "registration_resolution.md",
        ),
        ("src/cluster_trace.rs", "cluster_trace.md"),
        ("src/overload_resolution.rs", "overload_resolution.md"),
        ("src/resolved_typed_ast.rs", "resolved_typed_ast.md"),
    ];
    let mut violations = Vec::new();

    for (source_path, spec_name) in modules {
        let source_path = root.join(source_path);
        let source = read_to_string(&source_path);
        let public_enums = public_enums(&source);
        let public_enum_names = public_enums
            .iter()
            .map(|public_enum| public_enum.name.as_str())
            .collect::<BTreeSet<_>>();
        assert!(
            !public_enums.is_empty(),
            "{} should have checker-owned public enums for task-31 policy coverage",
            source_path.display()
        );

        let en_path = docs_root.join("en").join(spec_name);
        let ja_path = docs_root.join("ja").join(spec_name);
        let en_doc = read_to_string(&en_path);
        let ja_doc = read_to_string(&ja_path);
        let en_section = public_enum_policy_section(&en_doc).unwrap_or_else(|| {
            panic!(
                "{} must contain a Public Enum Policy section",
                en_path.display()
            )
        });
        let ja_section = public_enum_policy_section(&ja_doc).unwrap_or_else(|| {
            panic!(
                "{} must contain a Public Enum Policy section",
                ja_path.display()
            )
        });
        let en_policy_enums = public_enum_policy_entries(en_section);
        let ja_policy_enums = public_enum_policy_entries(ja_section);
        push_duplicate_policy_entries(&en_path, &en_policy_enums, &mut violations);
        push_duplicate_policy_entries(&ja_path, &ja_policy_enums, &mut violations);

        if !en_section.contains("No exhaustive public enum exceptions are owned by this module.") {
            violations.push(format!(
                "{}: Public Enum Policy must state there are no exhaustive public enum exceptions",
                en_path.display()
            ));
        }
        if !ja_section.contains("この module が所有する exhaustive public enum exception はない。")
        {
            violations.push(format!(
                "{}: Public Enum Policy must state there are no exhaustive public enum exceptions",
                ja_path.display()
            ));
        }

        for documented_enum in &en_policy_enums {
            if !public_enum_names.contains(documented_enum.as_str()) {
                violations.push(format!(
                    "{}: Public Enum Policy table must not include unknown `{}` row",
                    en_path.display(),
                    documented_enum
                ));
            }
        }
        for documented_enum in &ja_policy_enums {
            if !public_enum_names.contains(documented_enum.as_str()) {
                violations.push(format!(
                    "{}: Public Enum Policy table must not include unknown `{}` row",
                    ja_path.display(),
                    documented_enum
                ));
            }
        }

        for public_enum in &public_enums {
            if !public_enum.has_non_exhaustive {
                violations.push(format!(
                    "{}:{} public enum {} must be #[non_exhaustive]",
                    source_path.display(),
                    public_enum.line_number,
                    public_enum.name
                ));
            }
            if !en_policy_enums.contains(&public_enum.name) {
                violations.push(format!(
                    "{}: Public Enum Policy table must include `{}` as a first-column entry",
                    en_path.display(),
                    public_enum.name
                ));
            }
            if !ja_policy_enums.contains(&public_enum.name) {
                violations.push(format!(
                    "{}: Public Enum Policy table must include `{}` as a first-column entry",
                    ja_path.display(),
                    public_enum.name
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "checker public enum policy/source drift:\n{}",
        violations.join("\n")
    );
}

#[test]
fn checker_task33c_public_family_remains_scalar_and_non_installing() {
    let source = read_to_string(&crate_root().join("src/source_formula_composition.rs"));
    assert_eq!(
        source
            .matches("pub struct SourceNestedFraenkelCaptureGraphOwnerHandoff")
            .count(),
        1,
        "Task33C must expose one scalar graph-owner handoff"
    );
    assert_eq!(
        source
            .matches("pub enum SourceNestedFraenkelCaptureGraphOwnerError")
            .count(),
        1,
        "Task33C must expose one graph-owner error family"
    );
    assert_eq!(
        source
            .matches("pub struct SourceNestedFraenkelCaptureGraphOwnerProducer")
            .count(),
        1,
        "Task33C must expose one graph-owner producer"
    );
    assert!(
        !source.contains("SourceNestedFraenkelCaptureGraphOwnerId"),
        "Task33C must not introduce an owner id"
    );
    assert!(
        !source.contains("with_source_nested_fraenkel_capture_graph_owner"),
        "Task33C must not add an installer"
    );
}

#[test]
fn checker_live_source_inventory_matches_repository() {
    let root = crate_root();
    let path = workspace_root().join("doc/design/mizar-checker/checker_source_inventory.tsv");
    let violations = validate_checker_source_inventory(&root, &read_to_string(&path));

    assert!(
        violations.is_empty(),
        "checker live source inventory drift:\n{}",
        violations.join("\n")
    );
}

#[test]
fn checker_bilingual_sync_audit_covers_design_doc_pairs() {
    let docs_root = workspace_root().join("doc/design/mizar-checker");
    let en_dir = docs_root.join("en");
    let ja_dir = docs_root.join("ja");
    let expected_pairs = markdown_file_names(&en_dir);
    let ja_pairs = markdown_file_names(&ja_dir);
    let mut violations = Vec::new();

    if !expected_pairs.contains("bilingual_sync_audit.md") {
        violations.push(format!(
            "{}: task 33 audit must include its own English document",
            en_dir.display()
        ));
    }
    for missing in expected_pairs.difference(&ja_pairs) {
        violations.push(format!(
            "{}: missing Japanese companion for `{missing}`",
            ja_dir.display()
        ));
    }
    for stale in ja_pairs.difference(&expected_pairs) {
        violations.push(format!(
            "{}: stale Japanese companion `{stale}` has no English canonical file",
            ja_dir.display()
        ));
    }

    for audit_path in [
        docs_root.join("en/bilingual_sync_audit.md"),
        docs_root.join("ja/bilingual_sync_audit.md"),
    ] {
        let audit = read_to_string(&audit_path);
        let rows = bilingual_pair_audit_rows(&audit_path, &audit, &mut violations);
        push_bilingual_pair_inventory_drift(&audit_path, &expected_pairs, &rows, &mut violations);

        for row in &rows {
            let expected_en_companion = format!("../ja/{}", row.pair);
            let expected_ja_companion = format!("../en/{}", row.pair);
            if row.en_companion != expected_en_companion {
                violations.push(format!(
                    "{}: Pair Inventory row `{}` must use EN companion `{}`",
                    audit_path.display(),
                    row.pair,
                    expected_en_companion
                ));
            }
            if row.ja_companion != expected_ja_companion {
                violations.push(format!(
                    "{}: Pair Inventory row `{}` must use JA companion `{}`",
                    audit_path.display(),
                    row.pair,
                    expected_ja_companion
                ));
            }
            if row.comparison_basis.trim().is_empty() {
                violations.push(format!(
                    "{}: Pair Inventory row `{}` must record a comparison basis",
                    audit_path.display(),
                    row.pair
                ));
            }
            if row.sync_debt != "none" {
                violations.push(format!(
                    "{}: Pair Inventory row `{}` must not defer bilingual sync debt: {}",
                    audit_path.display(),
                    row.pair,
                    row.sync_debt
                ));
            }
        }
    }

    for pair in &expected_pairs {
        let en_path = en_dir.join(pair);
        let ja_path = ja_dir.join(pair);
        let en_doc = read_to_string(&en_path);
        let ja_doc = read_to_string(&ja_path);
        let en_link = format!("../ja/{pair}");
        let ja_link = format!("../en/{pair}");

        if !en_doc.contains(&en_link) {
            violations.push(format!(
                "{}: English canonical document must link to `{en_link}`",
                en_path.display()
            ));
        }
        if !ja_doc.contains(&ja_link) {
            violations.push(format!(
                "{}: Japanese companion must link to `{ja_link}`",
                ja_path.display()
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "checker bilingual documentation sync audit drift:\n{}",
        violations.join("\n")
    );
}

#[test]
fn checker_live_source_inventory_boundary_matches_repository() {
    let root = crate_root();
    let path = workspace_root().join("doc/design/mizar-checker/checker_source_inventory.tsv");
    let violations = validate_checker_source_inventory(&root, &read_to_string(&path));

    assert!(
        violations.is_empty(),
        "checker live module-boundary inventory drift:\n{}",
        violations.join("\n")
    );
}

#[test]
fn checker_live_source_inventory_rejects_in_memory_mutations() {
    let root = crate_root();
    let path = workspace_root().join("doc/design/mizar-checker/checker_source_inventory.tsv");
    let inventory = read_to_string(&path);
    let lines = inventory.lines().collect::<Vec<_>>();
    let mut missing_row = lines.clone();
    missing_row.remove(4);
    let mut extra_row = inventory.clone();
    extra_row.push_str("src/extra.rs\t1\ttest-support\tdoc/design/mizar-checker/en/00.crate_plan.md\tno\tno\tkeep-test-support\tnone\n");
    let mut duplicate_row = lines.clone();
    duplicate_row.insert(5, lines[4]);
    let mut reordered_rows = lines.clone();
    reordered_rows.swap(4, 5);
    let seven_field_row = lines[4]
        .rsplit_once('\t')
        .expect("inventory row has a public-surface field")
        .0;

    let mutations = [
        (
            "missing generator comment",
            inventory.replacen(
                "# generator: crates/mizar-checker/tests/lint_policy.rs::checker_live_source_inventory_matches_repository\n",
                "",
                1,
            ),
        ),
        (
            "malformed generator comment",
            inventory.replacen("# generator:", "# generator-mismatch:", 1),
        ),
        (
            "missing source-of-truth comment",
            inventory.replacen(
                "# source-of-truth: crates/mizar-checker/{src,tests,benches,examples}/**/*.rs plus crates/mizar-checker/build.rs when present\n",
                "",
                1,
            ),
        ),
        (
            "malformed source-of-truth comment",
            inventory.replacen("# source-of-truth:", "# source:", 1),
        ),
        ("missing schema", inventory.replacen("schema\t1\n", "", 1)),
        (
            "malformed schema",
            inventory.replacen("schema\t1\n", "schema\t2\n", 1),
        ),
        (
            "missing header",
            inventory.replacen(&format!("{CHECKER_SOURCE_INVENTORY_HEADER}\n"), "", 1),
        ),
        (
            "malformed header",
            inventory.replacen("path\tlines\tboundary", "path\tline_count\tboundary", 1),
        ),
        (
            "missing final newline",
            inventory.trim_end_matches('\n').to_owned(),
        ),
        (
            "seven-field row",
            inventory.replacen(lines[4], seven_field_row, 1),
        ),
        (
            "nine-field row",
            inventory.replacen(lines[4], &format!("{}\textra", lines[4]), 1),
        ),
        (
            "empty field",
            mutate_first_row_cell(&inventory, 2, ""),
        ),
        (
            "nonnumeric line count",
            mutate_first_row_cell(&inventory, 1, "NaN"),
        ),
        ("missing row", format!("{}\n", missing_row.join("\n"))),
        ("extra row", extra_row),
        ("duplicate row", format!("{}\n", duplicate_row.join("\n"))),
        (
            "reordered rows",
            format!("{}\n", reordered_rows.join("\n")),
        ),
        (
            "stale line count",
            mutate_first_row_cell(&inventory, 1, "0"),
        ),
        (
            "archive owner",
            mutate_first_row_cell(&inventory, 3, "doc/design/archive/stale.md"),
        ),
        (
            "traversal owner",
            mutate_first_row_cell(&inventory, 3, "../outside.md"),
        ),
        (
            "wrong owner",
            mutate_first_row_cell(
                &inventory,
                3,
                "doc/design/mizar-checker/en/todo.md",
            ),
        ),
        (
            "nonexistent owner",
            mutate_first_row_cell(
                &inventory,
                3,
                "doc/design/mizar-checker/en/no-such-owner.md",
            ),
        ),
        (
            "wrong boundary",
            mutate_first_row_cell(&inventory, 2, "wrong-boundary"),
        ),
        (
            "open split finding",
            mutate_first_row_cell(&inventory, 4, "yes"),
        ),
        (
            "open hard-gate finding",
            mutate_first_row_cell(&inventory, 5, "yes"),
        ),
        (
            "wrong decision",
            mutate_first_row_cell(&inventory, 6, "split-required"),
        ),
        (
            "wrong public surface",
            mutate_first_row_cell(&inventory, 7, "none"),
        ),
        (
            "wrong crate module exports",
            mutate_inventory_row_cell(&inventory, "src/lib.rs", 7, "modules:"),
        ),
    ];

    for (label, mutated) in mutations {
        assert!(
            !validate_checker_source_inventory(&root, &mutated).is_empty(),
            "inventory mutation `{label}` must fail closed"
        );
    }
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
fn checker_allow_exceptions_are_documented_inline() {
    let root = crate_root();
    let mut violations = Vec::new();

    for path in checker_rust_target_files(&root) {
        collect_undocumented_allows(&root, &path, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "intentional lint allow exceptions need an adjacent reason:\n{}",
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

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> PathBuf {
    crate_root()
        .parent()
        .and_then(|path| path.parent())
        .expect("crate lives under crates/<name>")
        .to_path_buf()
}

fn read_to_string(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
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

fn dependency_sections(manifest: &str) -> Vec<(String, Vec<&str>)> {
    let mut sections = Vec::new();
    let mut active = None;

    for line in manifest.lines() {
        let trimmed = line.trim();
        if let Some(section_name) = section_name(trimmed) {
            if let Some(section) = active.take() {
                sections.push(section);
            }
            active =
                dependency_section(section_name).then(|| (section_name.to_owned(), Vec::new()));
            continue;
        }

        if let Some((_, lines)) = &mut active
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
        {
            lines.push(trimmed);
        }
    }

    if let Some(section) = active {
        sections.push(section);
    }

    sections
}

fn section_name(line: &str) -> Option<&str> {
    line.strip_prefix('[')?.strip_suffix(']')
}

fn dependency_section(section_name: &str) -> bool {
    matches!(
        section_name,
        "dependencies" | "dev-dependencies" | "build-dependencies"
    ) || section_name.starts_with("dependencies.")
        || section_name.starts_with("dev-dependencies.")
        || section_name.starts_with("build-dependencies.")
        || section_name.ends_with(".dependencies")
        || section_name.ends_with(".dev-dependencies")
        || section_name.ends_with(".build-dependencies")
        || section_name.contains(".dependencies.")
        || section_name.contains(".dev-dependencies.")
        || section_name.contains(".build-dependencies.")
}

fn assignment_is(line: &str, key: &str, value: &str) -> bool {
    let Some((lhs, rhs)) = line.split_once('=') else {
        return false;
    };

    lhs.trim() == key && rhs.trim().trim_matches('"') == value
}

fn collect_undocumented_allows(root: &Path, path: &Path, violations: &mut Vec<String>) {
    let source = read_to_string(path);

    for line_number in undocumented_allow_line_numbers(&source) {
        let display_path = path.strip_prefix(root).unwrap_or(path);
        violations.push(format!("{}:{line_number}", display_path.display()));
    }
}

fn public_declaration_name(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let rest = trimmed
        .strip_prefix("pub ")
        .or_else(|| trimmed.strip_prefix("pub("))?;
    rest.split(|character: char| {
        character.is_whitespace() || matches!(character, '(' | ')' | ':' | '<' | '{')
    })
    .find(|part| !part.is_empty())
}

fn public_checker_api_is_documented(root: &Path, path: &Path, line: &str) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    if matches!(
        relative,
        path if path == Path::new("src/typed_ast.rs")
            || path == Path::new("src/binding_env.rs")
            || path == Path::new("src/source_context.rs")
            || path == Path::new("src/source_type.rs")
            || path == Path::new("src/source_attribute.rs")
            || path == Path::new("src/source_attribute_definition.rs")
            || path == Path::new("src/source_evidence.rs")
            || path == Path::new("src/source_term.rs")
            || path == Path::new("src/source_application.rs")
            || path == Path::new("src/source_atomic_formula.rs")
            || path == Path::new("src/source_composite_formula.rs")
            || path == Path::new("src/source_formula_composition.rs")
            || path == Path::new("src/source_predicate_definition.rs")
            || path == Path::new("src/source_functor_definition.rs")
            || path == Path::new("src/source_property_implementation.rs")
            || path == Path::new("src/source_mode_definition.rs")
            || path == Path::new("src/source_set_term.rs")
            || path == Path::new("src/source_statement.rs")
            || path == Path::new("src/source_proof_local_declaration.rs")
            || path == Path::new("src/source_structure.rs")
            || path == Path::new("src/source_structure_definition.rs")
            || path == Path::new("src/source_template.rs")
            || path == Path::new("src/source_template_type_parameter_association.rs")
            || path == Path::new("src/type_checker.rs")
            || path == Path::new("src/registration_resolution.rs")
            || path == Path::new("src/cluster_trace.rs")
            || path == Path::new("src/overload_resolution.rs")
            || path == Path::new("src/resolved_typed_ast.rs")
    ) {
        return true;
    }
    relative == Path::new("src/lib.rs")
        && matches!(
            line.trim(),
            "pub mod typed_ast;"
                | "pub mod binding_env;"
                | "pub mod source_context;"
                | "pub mod source_type;"
                | "pub mod source_attribute;"
                | "pub mod source_attribute_definition;"
                | "pub mod source_evidence;"
                | "pub mod source_term;"
                | "pub mod source_application;"
                | "pub mod source_atomic_formula;"
                | "pub mod source_composite_formula;"
                | "pub mod source_formula_composition;"
                | "pub mod source_predicate_definition;"
                | "pub mod source_functor_definition;"
                | "pub mod source_property_implementation;"
                | "pub mod source_mode_definition;"
                | "pub mod source_set_term;"
                | "pub mod source_statement;"
                | "pub mod source_proof_local_declaration;"
                | "pub mod source_structure;"
                | "pub mod source_structure_definition;"
                | "pub mod source_template;"
                | "pub mod source_template_type_parameter_association;"
                | "pub mod type_checker;"
                | "pub mod registration_resolution;"
                | "pub mod cluster_trace;"
                | "pub mod overload_resolution;"
                | "pub mod resolved_typed_ast;"
        )
}

fn public_module_exports(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("pub mod ")
                .and_then(|rest| rest.strip_suffix(';'))
                .map(str::to_owned)
        })
        .collect()
}

fn public_surface_names(source: &str) -> Vec<String> {
    let mut names = BTreeSet::new();

    for line in source.lines() {
        if let Some(name) = macro_public_newtype_name(line.trim()) {
            names.insert(name.to_owned());
            continue;
        }

        let Some(rest) = line.strip_prefix("pub ") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let Some(kind) = parts.next() else {
            continue;
        };
        if !matches!(kind, "struct" | "enum" | "type" | "trait" | "fn" | "const") {
            continue;
        }
        let Some(raw_name) = parts.next() else {
            continue;
        };
        let name = raw_name
            .split(['<', '(', '{', ':', '=', ';'])
            .find(|part| !part.is_empty());
        if let Some(name) = name {
            names.insert(name.to_owned());
        }
    }

    for invocation in source.split("table!(").skip(1) {
        if let Some(name) = invocation
            .trim_start()
            .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
            .next()
            .filter(|name| !name.is_empty())
        {
            names.insert(name.to_owned());
        }
    }

    names.into_iter().collect()
}

const CHECKER_SOURCE_INVENTORY_GENERATOR: &str = "# generator: crates/mizar-checker/tests/lint_policy.rs::checker_live_source_inventory_matches_repository";
const CHECKER_SOURCE_INVENTORY_SOURCE: &str = "# source-of-truth: crates/mizar-checker/{src,tests,benches,examples}/**/*.rs plus crates/mizar-checker/build.rs when present";
const CHECKER_SOURCE_INVENTORY_SCHEMA: &str = "schema\t1";
const CHECKER_SOURCE_INVENTORY_HEADER: &str =
    "path\tlines\tboundary\towner_doc\tsplit_required\thard_gate_finding\tdecision\tpublic_surface";
const CHECKER_PLAN_OWNER: &str = "doc/design/mizar-checker/en/00.crate_plan.md";

#[derive(Debug, Clone, PartialEq, Eq)]
struct CheckerSourceInventoryRow {
    path: String,
    lines: usize,
    boundary: String,
    owner_doc: String,
    split_required: String,
    hard_gate_finding: String,
    decision: String,
    public_surface: String,
}

fn validate_checker_source_inventory(root: &Path, document: &str) -> Vec<String> {
    let inventory_path = Path::new("checker_source_inventory.tsv");
    let mut violations = Vec::new();
    let lines = document.lines().collect::<Vec<_>>();
    let required_prefix = [
        CHECKER_SOURCE_INVENTORY_GENERATOR,
        CHECKER_SOURCE_INVENTORY_SOURCE,
        CHECKER_SOURCE_INVENTORY_SCHEMA,
        CHECKER_SOURCE_INVENTORY_HEADER,
    ];

    if !document.ends_with('\n') {
        violations.push("checker source inventory must end with a newline".to_owned());
    }
    for (index, expected) in required_prefix.iter().enumerate() {
        if lines.get(index) != Some(expected) {
            violations.push(format!(
                "checker source inventory line {} must be exactly `{expected}`",
                index + 1
            ));
        }
    }

    let mut rows = Vec::new();
    for (index, line) in lines.iter().enumerate().skip(required_prefix.len()) {
        let cells = line.split('\t').collect::<Vec<_>>();
        if cells.len() != 8 {
            violations.push(format!(
                "checker source inventory line {} must have exactly 8 tab-separated fields",
                index + 1
            ));
            continue;
        }
        if cells.iter().any(|cell| cell.is_empty()) {
            violations.push(format!(
                "checker source inventory line {} must not contain an empty field",
                index + 1
            ));
        }
        let parsed_lines = cells[1].parse::<usize>().unwrap_or_else(|_| {
            violations.push(format!(
                "checker source inventory line {} has a non-numeric line count",
                index + 1
            ));
            usize::MAX
        });
        rows.push(CheckerSourceInventoryRow {
            path: cells[0].to_owned(),
            lines: parsed_lines,
            boundary: cells[2].to_owned(),
            owner_doc: cells[3].to_owned(),
            split_required: cells[4].to_owned(),
            hard_gate_finding: cells[5].to_owned(),
            decision: cells[6].to_owned(),
            public_surface: cells[7].to_owned(),
        });
    }

    let expected_rows = checker_rust_target_files(root)
        .iter()
        .map(|path| expected_checker_source_inventory_row(root, path))
        .collect::<Vec<_>>();
    let actual_paths = rows.iter().map(|row| row.path.clone()).collect::<Vec<_>>();
    let expected_paths = expected_rows
        .iter()
        .map(|row| row.path.clone())
        .collect::<Vec<_>>();
    push_duplicate_entries(
        inventory_path,
        "checker source inventory rows",
        &actual_paths,
        &mut violations,
    );

    let actual_set = actual_paths.iter().cloned().collect::<BTreeSet<_>>();
    let expected_set = expected_paths.iter().cloned().collect::<BTreeSet<_>>();
    for missing in expected_set.difference(&actual_set) {
        violations.push(format!("checker source inventory must include `{missing}`"));
    }
    for extra in actual_set.difference(&expected_set) {
        violations.push(format!(
            "checker source inventory must not include stale `{extra}`"
        ));
    }
    if actual_paths != expected_paths {
        violations
            .push("checker source inventory rows must use exact lexical path order".to_owned());
    }

    let workspace = root
        .parent()
        .and_then(Path::parent)
        .expect("checker crate lives under crates/<name>");
    for row in &rows {
        if invalid_inventory_relative_path(&row.path) {
            violations.push(format!(
                "checker source inventory path `{}` must be a normalized relative path",
                row.path
            ));
        }
        if invalid_inventory_relative_path(&row.owner_doc)
            || row.owner_doc.starts_with("doc/design/archive/")
        {
            violations.push(format!(
                "checker source inventory owner `{}` must be a normalized live path",
                row.owner_doc
            ));
        } else if !workspace.join(&row.owner_doc).is_file() {
            violations.push(format!(
                "checker source inventory owner `{}` must exist",
                row.owner_doc
            ));
        }
        if row.split_required != "no" {
            violations.push(format!(
                "checker source inventory row `{}` must keep split_required=no",
                row.path
            ));
        }
        if row.hard_gate_finding != "no" {
            violations.push(format!(
                "checker source inventory row `{}` must keep hard_gate_finding=no",
                row.path
            ));
        }
    }

    for (index, (actual, expected)) in rows.iter().zip(&expected_rows).enumerate() {
        if actual != expected {
            violations.push(format!(
                "checker source inventory row {} mismatch: expected {expected:?}, found {actual:?}",
                index + 1
            ));
        }
    }
    if rows.len() != expected_rows.len() {
        violations.push(format!(
            "checker source inventory must contain {} rows, not {}",
            expected_rows.len(),
            rows.len()
        ));
    }

    violations
}

fn expected_checker_source_inventory_row(root: &Path, path: &Path) -> CheckerSourceInventoryRow {
    let relative = relative_path_string(root, path);
    let source = read_to_string(path);
    let (boundary, owner_doc, decision, public_surface) = if relative == "src/lib.rs" {
        (
            "crate-exports".to_owned(),
            CHECKER_PLAN_OWNER.to_owned(),
            "keep-current-boundary".to_owned(),
            format!("modules:{}", public_module_exports(&source).join(",")),
        )
    } else if relative.starts_with("src/") {
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("Rust source path has a UTF-8 stem");
        let module_owner = format!("doc/design/mizar-checker/en/{stem}.md");
        let workspace = root
            .parent()
            .and_then(Path::parent)
            .expect("checker crate lives under crates/<name>");
        let owner_doc = if workspace.join(&module_owner).is_file() {
            module_owner
        } else {
            CHECKER_PLAN_OWNER.to_owned()
        };
        (
            format!("module:{stem}"),
            owner_doc,
            "keep-current-boundary".to_owned(),
            format!("items:{}", public_surface_names(&source).join(",")),
        )
    } else {
        (
            "test-support".to_owned(),
            CHECKER_PLAN_OWNER.to_owned(),
            "keep-test-support".to_owned(),
            "none".to_owned(),
        )
    };

    CheckerSourceInventoryRow {
        path: relative,
        lines: source.lines().count(),
        boundary,
        owner_doc,
        split_required: "no".to_owned(),
        hard_gate_finding: "no".to_owned(),
        decision,
        public_surface,
    }
}

fn invalid_inventory_relative_path(path: &str) -> bool {
    path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.split('/').any(|component| component == "..")
}

fn mutate_first_row_cell(document: &str, column: usize, replacement: &str) -> String {
    let first_path = document
        .lines()
        .nth(4)
        .and_then(|line| line.split('\t').next())
        .expect("inventory has a first data row");
    mutate_inventory_row_cell(document, first_path, column, replacement)
}

fn mutate_inventory_row_cell(
    document: &str,
    target_path: &str,
    column: usize,
    replacement: &str,
) -> String {
    let mut lines = document.lines().map(str::to_owned).collect::<Vec<_>>();
    let target = lines
        .iter()
        .position(|line| line.split('\t').next() == Some(target_path))
        .expect("target inventory row exists");
    let mut cells = lines[target]
        .split('\t')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    cells[column] = replacement.to_owned();
    lines[target] = cells.join("\t");
    format!("{}\n", lines.join("\n"))
}

fn markdown_file_names(dir: &Path) -> BTreeSet<String> {
    let entries = fs::read_dir(dir).unwrap_or_else(|error| panic!("{}: {error}", dir.display()));
    let mut names = BTreeSet::new();

    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("{}: {error}", dir.display()));
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        names.insert(name.to_owned());
    }

    names
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BilingualPairAuditRow {
    pair: String,
    en_companion: String,
    ja_companion: String,
    comparison_basis: String,
    sync_debt: String,
}

fn bilingual_pair_audit_rows(
    path: &Path,
    document: &str,
    violations: &mut Vec<String>,
) -> Vec<BilingualPairAuditRow> {
    let Some(section) = markdown_heading_section(document, "## Pair Inventory") else {
        violations.push(format!(
            "{}: bilingual sync audit must contain `## Pair Inventory`",
            path.display()
        ));
        return Vec::new();
    };
    let mut rows = Vec::new();
    let mut saw_header = false;
    let mut saw_delimiter = false;

    for line in section.lines() {
        let Some(cells) = markdown_table_cells(line) else {
            continue;
        };
        if cells
            == [
                "Pair",
                "EN companion",
                "JA companion",
                "Comparison basis",
                "Sync debt",
            ]
        {
            saw_header = true;
            continue;
        }
        if cells == ["---", "---", "---", "---", "---"] {
            saw_delimiter = true;
            continue;
        }
        if cells.len() != 5 {
            violations.push(format!(
                "{}: Pair Inventory row must have exactly 5 columns: {}",
                path.display(),
                line.trim()
            ));
            continue;
        }
        let Some(pair) = single_code_span(&cells[0]) else {
            violations.push(format!(
                "{}: Pair Inventory first column must be a code-spanned filename: {}",
                path.display(),
                line.trim()
            ));
            continue;
        };
        let Some(en_companion) = single_code_span(&cells[1]) else {
            violations.push(format!(
                "{}: Pair Inventory EN companion must be code-spanned for `{pair}`",
                path.display()
            ));
            continue;
        };
        let Some(ja_companion) = single_code_span(&cells[2]) else {
            violations.push(format!(
                "{}: Pair Inventory JA companion must be code-spanned for `{pair}`",
                path.display()
            ));
            continue;
        };
        rows.push(BilingualPairAuditRow {
            pair,
            en_companion,
            ja_companion,
            comparison_basis: cells[3].clone(),
            sync_debt: cells[4].clone(),
        });
    }

    if !saw_header {
        violations.push(format!(
            "{}: Pair Inventory must use exact header `| Pair | EN companion | JA companion | Comparison basis | Sync debt |`",
            path.display()
        ));
    }
    if !saw_delimiter {
        violations.push(format!(
            "{}: Pair Inventory must use exact delimiter `|---|---|---|---|---|`",
            path.display()
        ));
    }

    rows
}

fn markdown_table_cells(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return None;
    }

    Some(
        trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .map(str::to_owned)
            .collect(),
    )
}

fn single_code_span(cell: &str) -> Option<String> {
    cell.strip_prefix('`')?.strip_suffix('`').map(str::to_owned)
}

fn push_bilingual_pair_inventory_drift(
    path: &Path,
    expected: &BTreeSet<String>,
    rows: &[BilingualPairAuditRow],
    violations: &mut Vec<String>,
) {
    let actual = rows.iter().map(|row| row.pair.clone()).collect::<Vec<_>>();
    let actual_set = actual.iter().cloned().collect::<BTreeSet<_>>();
    push_duplicate_entries(path, "Pair Inventory rows", &actual, violations);

    for missing in expected.difference(&actual_set) {
        violations.push(format!(
            "{}: Pair Inventory must include `{missing}`",
            path.display()
        ));
    }
    for stale in actual_set.difference(expected) {
        violations.push(format!(
            "{}: Pair Inventory must not include stale `{stale}`",
            path.display()
        ));
    }
}

fn push_duplicate_entries(
    path: &Path,
    label: &str,
    entries: &[String],
    violations: &mut Vec<String>,
) {
    let mut seen = BTreeSet::new();

    for entry in entries {
        if !seen.insert(entry) {
            violations.push(format!(
                "{}: {label} must not duplicate `{entry}`",
                path.display()
            ));
        }
    }
}

fn macro_public_newtype_name(line: &str) -> Option<&str> {
    line.strip_prefix("dense_id!(")
        .or_else(|| line.strip_prefix("string_key!("))
        .and_then(|rest| rest.split_once(')'))
        .map(|(name, _)| name)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PublicEnum {
    name: String,
    line_number: usize,
    has_non_exhaustive: bool,
}

fn public_enums(source: &str) -> Vec<PublicEnum> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut public_enums = Vec::new();

    for (line_index, line) in lines.iter().enumerate() {
        let Some(name) = plain_public_enum_name(line) else {
            continue;
        };
        public_enums.push(PublicEnum {
            name: name.to_owned(),
            line_number: line_index + 1,
            has_non_exhaustive: has_non_exhaustive_attribute_before(&lines, line_index),
        });
    }

    public_enums
}

fn plain_public_enum_name(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix("pub enum ")?;
    rest.split(|character: char| character.is_whitespace() || matches!(character, '<' | '{' | '('))
        .find(|part| !part.is_empty())
}

fn has_non_exhaustive_attribute_before(lines: &[&str], enum_line_index: usize) -> bool {
    for line in lines[..enum_line_index].iter().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "#[non_exhaustive]" {
            return true;
        }
        if trimmed.starts_with("#[") || trimmed.starts_with("#!") || trimmed.starts_with(']') {
            continue;
        }
        break;
    }

    false
}

fn public_enum_policy_section(document: &str) -> Option<&str> {
    let start = document
        .lines()
        .scan(0, |offset, line| {
            let current = *offset;
            *offset += line.len() + 1;
            Some((current, line))
        })
        .find_map(|(offset, line)| (line.trim() == "## Public Enum Policy").then_some(offset))?;
    let rest = &document[start..];
    let end = rest
        .lines()
        .scan(0, |offset, line| {
            let current = *offset;
            *offset += line.len() + 1;
            Some((current, line))
        })
        .skip(1)
        .find_map(|(offset, line)| {
            (line.starts_with("## ") && line.trim() != "## Public Enum Policy").then_some(offset)
        })
        .unwrap_or(rest.len());

    Some(&rest[..end])
}

fn public_enum_policy_entries(section: &str) -> Vec<String> {
    section
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed.strip_prefix("| `")?;
            let (name, _) = rest.split_once("` |")?;
            Some(name.to_owned())
        })
        .collect()
}

fn push_duplicate_policy_entries(path: &Path, entries: &[String], violations: &mut Vec<String>) {
    let mut seen = BTreeSet::new();

    for entry in entries {
        if !seen.insert(entry) {
            violations.push(format!(
                "{}: Public Enum Policy table must not duplicate `{}` rows",
                path.display(),
                entry
            ));
        }
    }
}

fn markdown_heading_section<'a>(document: &'a str, heading: &str) -> Option<&'a str> {
    let start = document
        .lines()
        .scan(0, |offset, line| {
            let current = *offset;
            *offset += line.len() + 1;
            Some((current, line))
        })
        .find_map(|(offset, line)| (line.trim() == heading).then_some(offset))?;
    let rest = &document[start..];
    let heading_level = markdown_heading_level(heading)?;
    let end = rest
        .lines()
        .scan(0, |offset, line| {
            let current = *offset;
            *offset += line.len() + 1;
            Some((current, line))
        })
        .skip(1)
        .find_map(|(offset, line)| {
            let level = markdown_heading_level(line)?;
            (level <= heading_level).then_some(offset)
        })
        .unwrap_or(rest.len());

    Some(&rest[..end])
}

fn markdown_heading_level(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    let level = trimmed.bytes().take_while(|byte| *byte == b'#').count();
    (level > 0 && trimmed.as_bytes().get(level) == Some(&b' ')).then_some(level)
}

fn undocumented_allow_line_numbers(source: &str) -> Vec<usize> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut violations = Vec::new();
    let mut line_index = 0;

    while line_index < lines.len() {
        if !starts_attribute(lines[line_index]) {
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

fn checker_rust_target_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for relative_dir in ["src", "tests", "benches", "examples"] {
        collect_rs_files(&root.join(relative_dir), &mut files);
    }
    let build_rs = root.join("build.rs");
    if build_rs.exists() {
        files.push(build_rs);
    }
    files.sort();
    files
}

fn relative_path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn checker_src_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rs_files(&root.join("src"), &mut files);
    files.sort();
    files
}

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}
