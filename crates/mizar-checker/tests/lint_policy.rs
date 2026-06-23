use std::{
    collections::{BTreeMap, BTreeSet},
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
fn checker_manifest_dependency_boundary_is_task_one_minimal() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [(
            "dependencies".to_owned(),
            vec![
                "mizar-resolve = { path = \"../mizar-resolve\" }",
                "mizar-session = { path = \"../mizar-session\" }",
            ],
        )],
        "{} must depend only on mizar-resolve and mizar-session until a \
         task-scoped checker spec expands the crate boundary",
        manifest_path.display()
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
fn checker_source_spec_audit_covers_public_surface_and_gaps() {
    let root = crate_root();
    let docs_root = workspace_root().join("doc/design/mizar-checker");
    let en_path = docs_root.join("en/source_spec_audit.md");
    let ja_path = docs_root.join("ja/source_spec_audit.md");
    let en_audit = read_to_string(&en_path);
    let ja_audit = read_to_string(&ja_path);
    let modules = [
        ("src/typed_ast.rs", "typed_ast"),
        ("src/binding_env.rs", "binding_env"),
        ("src/type_checker.rs", "type_checker"),
        ("src/registration_resolution.rs", "registration_resolution"),
        ("src/cluster_trace.rs", "cluster_trace"),
        ("src/overload_resolution.rs", "overload_resolution"),
        ("src/resolved_typed_ast.rs", "resolved_typed_ast"),
    ];
    let mut violations = Vec::new();

    let public_modules = public_module_exports(&read_to_string(&root.join("src/lib.rs")));
    let public_module_set = public_modules.iter().cloned().collect::<BTreeSet<_>>();
    push_module_export_drift(
        &en_path,
        &public_module_set,
        &audit_module_exports(&en_audit),
        &mut violations,
    );
    push_module_export_drift(
        &ja_path,
        &public_module_set,
        &audit_module_exports(&ja_audit),
        &mut violations,
    );

    for (source_path, module_name) in modules {
        let source_path = root.join(source_path);
        let public_items = public_surface_names(&read_to_string(&source_path));
        assert!(
            !public_items.is_empty(),
            "{} should expose audited public items",
            source_path.display()
        );

        let heading = format!("### `{module_name}`");
        let en_section = markdown_heading_section(&en_audit, &heading).unwrap_or_else(|| {
            panic!(
                "{} must contain a source/spec audit section for `{module_name}`",
                en_path.display()
            )
        });
        let ja_section = markdown_heading_section(&ja_audit, &heading).unwrap_or_else(|| {
            panic!(
                "{} must contain a source/spec audit section for `{module_name}`",
                ja_path.display()
            )
        });

        let public_item_set = public_items.iter().cloned().collect::<BTreeSet<_>>();
        push_audit_inventory_drift(
            &en_path,
            module_name,
            &public_item_set,
            &audit_inventory_entries(en_section),
            &mut violations,
        );
        push_audit_inventory_drift(
            &ja_path,
            module_name,
            &public_item_set,
            &audit_inventory_entries(ja_section),
            &mut violations,
        );

        for public_item in &public_items {
            let needle = format!("`{public_item}`");
            if !en_section.contains(&needle) {
                violations.push(format!(
                    "{}: `{module_name}` audit must include public item {needle}",
                    en_path.display()
                ));
            }
            if !ja_section.contains(&needle) {
                violations.push(format!(
                    "{}: `{module_name}` audit must include public item {needle}",
                    ja_path.display()
                ));
            }
        }
    }

    let en_plan = read_to_string(&docs_root.join("en/00.crate_plan.md"));
    let expected_gap_ids = mc_gap_ids(&en_plan);
    let en_reconciled_gap_ids = reconciled_mc_gap_ids(&en_audit);
    let ja_reconciled_gap_ids = reconciled_mc_gap_ids(&ja_audit);
    push_gap_reconciliation_drift(
        &en_path,
        &expected_gap_ids,
        &en_reconciled_gap_ids,
        &mut violations,
    );
    push_gap_reconciliation_drift(
        &ja_path,
        &expected_gap_ids,
        &ja_reconciled_gap_ids,
        &mut violations,
    );

    assert!(
        violations.is_empty(),
        "checker source/spec audit drift:\n{}",
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
fn checker_module_boundary_audit_covers_source_layout() {
    let root = crate_root();
    let docs_root = workspace_root().join("doc/design/mizar-checker");
    let expected_files = checker_rust_target_files(&root)
        .into_iter()
        .map(|path| {
            let relative = relative_path_string(&root, &path);
            let lines = read_to_string(&path).lines().count();
            (relative, lines)
        })
        .collect::<BTreeMap<_, _>>();
    let required_classes = [
        "spec_gap",
        "test_gap",
        "design_drift",
        "source_drift",
        "source_undocumented_behavior",
        "boundary_violation",
        "external_dependency_gap",
        "deferred",
    ];
    let mut violations = Vec::new();

    for audit_path in [
        docs_root.join("en/module_boundary_audit.md"),
        docs_root.join("ja/module_boundary_audit.md"),
    ] {
        let audit = read_to_string(&audit_path);
        if !audit.contains("## Split Gate") {
            violations.push(format!(
                "{}: module-boundary audit must contain `## Split Gate`",
                audit_path.display()
            ));
        }
        let rows = module_boundary_audit_rows(&audit_path, &audit, &mut violations);
        push_source_layout_inventory_drift(&audit_path, &expected_files, &rows, &mut violations);

        for row in &rows {
            if let Some(expected_lines) = expected_files.get(&row.path)
                && row.lines != *expected_lines
            {
                violations.push(format!(
                    "{}: Source Layout Inventory row `{}` must record {expected_lines} lines, not {}",
                    audit_path.display(),
                    row.path,
                    row.lines
                ));
            }
            if row.boundary_label.trim().is_empty() {
                violations.push(format!(
                    "{}: Source Layout Inventory row `{}` must record a boundary label",
                    audit_path.display(),
                    row.path
                ));
            }
            if row.owning_specification.trim().is_empty() {
                violations.push(format!(
                    "{}: Source Layout Inventory row `{}` must record an owning specification",
                    audit_path.display(),
                    row.path
                ));
            }
            if row.split_required != "no" {
                violations.push(format!(
                    "{}: Source Layout Inventory row `{}` must not leave a required split unresolved: {}",
                    audit_path.display(),
                    row.path,
                    row.split_required
                ));
            }
            if row.hard_gate_finding != "no" {
                violations.push(format!(
                    "{}: Source Layout Inventory row `{}` must not leave a hard-gate finding unresolved: {}",
                    audit_path.display(),
                    row.path,
                    row.hard_gate_finding
                ));
            }
            if row.decision.trim().is_empty() {
                violations.push(format!(
                    "{}: Source Layout Inventory row `{}` must record a decision",
                    audit_path.display(),
                    row.path
                ));
            }
        }

        let classification = markdown_heading_section(&audit, "## Task 34 Classification")
            .unwrap_or_else(|| {
                panic!(
                    "{} must contain `## Task 34 Classification`",
                    audit_path.display()
                )
            });
        for class in required_classes {
            let needle = format!("| `{class}` |");
            if !classification.contains(&needle) {
                violations.push(format!(
                    "{}: Task 34 Classification must include `{class}`",
                    audit_path.display()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "checker module-boundary audit drift:\n{}",
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

fn audit_module_exports(document: &str) -> Vec<String> {
    let section = markdown_heading_section(document, "## Crate Module Exports")
        .expect("source/spec audit must contain Crate Module Exports section");
    let inventory = section
        .split("\nEvidence:")
        .next()
        .unwrap_or(section)
        .split("\n根拠:")
        .next()
        .unwrap_or(section);

    inventory
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed.strip_prefix("- `")?;
            let (name, _) = rest.split_once('`')?;
            Some(name.to_owned())
        })
        .collect()
}

fn push_module_export_drift(
    path: &Path,
    expected: &BTreeSet<String>,
    actual: &[String],
    violations: &mut Vec<String>,
) {
    let actual_set = actual.iter().cloned().collect::<BTreeSet<_>>();
    push_duplicate_entries(path, "Crate Module Exports inventory", actual, violations);

    for missing in expected.difference(&actual_set) {
        violations.push(format!(
            "{}: Crate Module Exports inventory must include `{missing}`",
            path.display()
        ));
    }
    for extra in actual_set.difference(expected) {
        violations.push(format!(
            "{}: Crate Module Exports inventory must not include stale `{extra}`",
            path.display()
        ));
    }
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

    names.into_iter().collect()
}

fn audit_inventory_entries(section: &str) -> Vec<String> {
    let inventory = section
        .split("\nCorrespondence:")
        .next()
        .unwrap_or(section)
        .split("\n対応:")
        .next()
        .unwrap_or(section);

    code_spans(
        inventory
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n")
            .as_str(),
    )
}

fn code_spans(text: &str) -> Vec<String> {
    let mut entries = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find('`') {
        rest = &rest[start + 1..];
        let Some(end) = rest.find('`') else {
            break;
        };
        entries.push(rest[..end].to_owned());
        rest = &rest[end + 1..];
    }

    entries
}

fn push_audit_inventory_drift(
    path: &Path,
    module_name: &str,
    expected: &BTreeSet<String>,
    actual: &[String],
    violations: &mut Vec<String>,
) {
    let actual_set = actual.iter().cloned().collect::<BTreeSet<_>>();
    push_duplicate_entries(
        path,
        &format!("`{module_name}` source/spec audit inventory"),
        actual,
        violations,
    );

    for missing in expected.difference(&actual_set) {
        violations.push(format!(
            "{}: `{module_name}` source/spec audit inventory must include `{missing}`",
            path.display()
        ));
    }
    for extra in actual_set.difference(expected) {
        violations.push(format!(
            "{}: `{module_name}` source/spec audit inventory must not include stale `{extra}`",
            path.display()
        ));
    }
}

fn reconciled_mc_gap_ids(document: &str) -> Vec<String> {
    let gap_section = markdown_heading_section(document, "## Gap Reconciliation")
        .or_else(|| markdown_heading_section(document, "## Gap Reconciliation"))
        .expect("source/spec audit must contain Gap Reconciliation section");
    let mut ids = Vec::new();

    for line in gap_section.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("| MC-G") else {
            continue;
        };
        let Some((digits, _)) = rest.split_once(" |") else {
            continue;
        };
        if digits.len() == 3 && digits.chars().all(|character| character.is_ascii_digit()) {
            ids.push(format!("MC-G{digits}"));
        }
    }

    for resolved_prefix in [
        "Resolved setup-history rows remain closed:",
        "Resolved setup-history row は closed のまま:",
    ] {
        let Some((_, resolved)) = gap_section.split_once(resolved_prefix) else {
            continue;
        };
        let resolved = resolved
            .split("\n## Task 32 Classification")
            .next()
            .unwrap_or(resolved);
        ids.extend(mc_gap_ids(resolved));
    }

    ids
}

fn push_gap_reconciliation_drift(
    path: &Path,
    expected: &BTreeSet<String>,
    actual: &[String],
    violations: &mut Vec<String>,
) {
    let actual_set = actual.iter().cloned().collect::<BTreeSet<_>>();
    push_duplicate_entries(path, "Gap Reconciliation MC-G rows", actual, violations);

    for missing in expected.difference(&actual_set) {
        violations.push(format!(
            "{}: Gap Reconciliation must include {missing}",
            path.display()
        ));
    }
    for extra in actual_set.difference(expected) {
        violations.push(format!(
            "{}: Gap Reconciliation must not include stale {extra}",
            path.display()
        ));
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModuleBoundaryAuditRow {
    path: String,
    lines: usize,
    boundary_label: String,
    owning_specification: String,
    split_required: String,
    hard_gate_finding: String,
    decision: String,
}

fn module_boundary_audit_rows(
    path: &Path,
    document: &str,
    violations: &mut Vec<String>,
) -> Vec<ModuleBoundaryAuditRow> {
    let Some(section) = markdown_heading_section(document, "## Source Layout Inventory") else {
        violations.push(format!(
            "{}: module-boundary audit must contain `## Source Layout Inventory`",
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
                "Path",
                "Lines",
                "Boundary label",
                "Owning specification",
                "Split required",
                "Hard-gate finding",
                "Decision",
            ]
        {
            saw_header = true;
            continue;
        }
        if cells == ["---", "---:", "---", "---", "---", "---", "---"] {
            saw_delimiter = true;
            continue;
        }
        if cells.len() != 7 {
            violations.push(format!(
                "{}: Source Layout Inventory row must have exactly 7 columns: {}",
                path.display(),
                line.trim()
            ));
            continue;
        }
        let Some(row_path) = single_code_span(&cells[0]) else {
            violations.push(format!(
                "{}: Source Layout Inventory path must be a code-spanned relative path: {}",
                path.display(),
                line.trim()
            ));
            continue;
        };
        let Ok(lines) = cells[1].parse::<usize>() else {
            violations.push(format!(
                "{}: Source Layout Inventory row `{row_path}` must record a numeric line count",
                path.display()
            ));
            continue;
        };
        rows.push(ModuleBoundaryAuditRow {
            path: row_path,
            lines,
            boundary_label: cells[2].clone(),
            owning_specification: cells[3].clone(),
            split_required: cells[4].clone(),
            hard_gate_finding: cells[5].clone(),
            decision: cells[6].clone(),
        });
    }

    if !saw_header {
        violations.push(format!(
            "{}: Source Layout Inventory must use exact header `| Path | Lines | Boundary label | Owning specification | Split required | Hard-gate finding | Decision |`",
            path.display()
        ));
    }
    if !saw_delimiter {
        violations.push(format!(
            "{}: Source Layout Inventory must use exact delimiter `|---|---:|---|---|---|---|---|`",
            path.display()
        ));
    }

    rows
}

fn push_source_layout_inventory_drift(
    path: &Path,
    expected: &BTreeMap<String, usize>,
    rows: &[ModuleBoundaryAuditRow],
    violations: &mut Vec<String>,
) {
    let actual = rows.iter().map(|row| row.path.clone()).collect::<Vec<_>>();
    let actual_set = actual.iter().cloned().collect::<BTreeSet<_>>();
    let expected_set = expected.keys().cloned().collect::<BTreeSet<_>>();
    push_duplicate_entries(path, "Source Layout Inventory rows", &actual, violations);

    for missing in expected_set.difference(&actual_set) {
        violations.push(format!(
            "{}: Source Layout Inventory must include `{missing}`",
            path.display()
        ));
    }
    for stale in actual_set.difference(&expected_set) {
        violations.push(format!(
            "{}: Source Layout Inventory must not include stale `{stale}`",
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

fn mc_gap_ids(document: &str) -> BTreeSet<String> {
    document
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '-'))
        .filter(|token| {
            token.len() == "MC-G000".len()
                && token.starts_with("MC-G")
                && token["MC-G".len()..]
                    .chars()
                    .all(|character| character.is_ascii_digit())
        })
        .map(str::to_owned)
        .collect()
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
