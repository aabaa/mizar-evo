use std::{fs, path::PathBuf};

#[test]
fn atp_manifest_opts_into_workspace_lints() {
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
fn workspace_manifest_includes_mizar_atp_once() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let members = workspace_members(&manifest);
    let occurrences = members
        .iter()
        .filter(|member| **member == "crates/mizar-atp")
        .count();

    assert_eq!(
        occurrences,
        1,
        "{} must list crates/mizar-atp exactly once in [workspace].members",
        manifest_path.display()
    );
}

#[test]
fn atp_manifest_keeps_task_one_package_metadata() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let package = section(&manifest, "package");
    let lib = section(&manifest, "lib");

    assert!(
        package
            .iter()
            .any(|line| assignment_is(line, "name", "mizar-atp")),
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
            .any(|line| assignment_is(line, "name", "mizar_atp")),
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
fn atp_manifest_dependency_boundary_is_task_one_minimal() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [(
            "dependencies".to_owned(),
            vec![
                "mizar-core = { path = \"../mizar-core\" }",
                "mizar-kernel = { path = \"../mizar-kernel\" }",
                "mizar-session = { path = \"../mizar-session\" }",
                "mizar-vc = { path = \"../mizar-vc\" }",
            ],
        )],
        "{} must keep task-1 production dependencies limited to mizar-core, \
         mizar-kernel, mizar-session, and mizar-vc; dev/build/target \
         dependency sections require a later explicit task",
        manifest_path.display()
    );
}

#[test]
fn atp_lib_exposes_only_spec_backed_modules() {
    let lib_path = crate_root().join("src/lib.rs");
    let source = read_to_string(&lib_path);
    let expected_source = r#"//! ATP candidate-evidence production boundary.
//!
//! `mizar-atp` owns pipeline phase 13: translating `NeedsAtp` VC obligations
//! into backend-neutral ATP problems, running untrusted backends, and
//! collecting formula/substitution evidence candidates for `mizar-kernel`.
//!
//! This crate does not accept proofs, select trusted winners, call the kernel
//! as proof authority, or expose backend proof methods as trusted material.

#![forbid(unsafe_code)]

pub mod problem;
pub mod property_encoding;
pub mod translator;
"#;
    let source_files = rust_source_files(&crate_root().join("src"))
        .into_iter()
        .map(|path| {
            path.strip_prefix(crate_root())
                .expect("source path lives in crate root")
                .display()
                .to_string()
        })
        .collect::<Vec<_>>();

    assert_eq!(
        source,
        expected_source,
        "{} must expose only spec-backed mizar-atp modules",
        lib_path.display()
    );
    assert_eq!(
        source_files,
        [
            "src/lib.rs",
            "src/problem.rs",
            "src/property_encoding.rs",
            "src/translator.rs"
        ],
        "semantic ATP modules require paired specs before source; found {source_files:?}"
    );
}

#[test]
fn atp_crate_tree_contains_only_current_spec_backed_files() {
    let mut files = crate_files()
        .into_iter()
        .filter(|file| file != "Cargo.lock")
        .collect::<Vec<_>>();
    files.sort();

    assert_eq!(
        files,
        [
            "Cargo.toml",
            "src/lib.rs",
            "src/problem.rs",
            "src/property_encoding.rs",
            "src/translator.rs",
            "tests/lint_policy.rs"
        ],
        "mizar-atp crate files must stay limited to current spec-backed sources; \
         build scripts, examples, benches, extra tests, backend runners, \
         kernel/proof behavior, or other crate-root files require explicit spec \
         tasks; found {files:?}"
    );
}

#[test]
fn atp_property_encoding_module_has_paired_specs_and_excludes_trusted_material() {
    let en_spec = workspace_root().join("doc/design/mizar-atp/en/property_encoding.md");
    let ja_spec = workspace_root().join("doc/design/mizar-atp/ja/property_encoding.md");
    let source_path = crate_root().join("src/property_encoding.rs");
    let en = read_to_string(&en_spec);
    let ja = read_to_string(&ja_spec);
    let source = read_to_string(&source_path);

    for marker in [
        "Generated Binders",
        "Task 8 must not emit native declarations yet",
        "`connectedness` is permitted to use `AtpFormulaTree::Or`",
        "Task-8 Test Expectations",
    ] {
        assert!(
            en.contains(marker),
            "{} must keep task-7 property spec marker `{marker}`",
            en_spec.display()
        );
    }
    for marker in [
        "Generated binder",
        "Task 8 はまだ native declaration を emit してはならない",
        "`AtpFormulaTree::Or` を使ってよい",
        "Task 8 test expectations",
    ] {
        assert!(
            ja.contains(marker),
            "{} must keep task-7 Japanese property spec marker `{marker}`",
            ja_spec.display()
        );
    }
    for marker in [
        "pub fn encode_properties",
        "AtpPropertyEncodingStrategy::NativeDeclaration",
        "AtpPropertyEncodingError::NativeDeclarationDeferred",
        "AtpDeclarationKind::GeneratedBinder",
        "AtpSymbolSource::GeneratedBinder",
        "AtpSourceRef::EncodedProperty",
        "AtpFormulaTree::Or",
        "EncodedProperty::axiom",
    ] {
        assert!(
            source.contains(marker),
            "{} must implement property_encoding.md task-8 marker `{marker}`",
            source_path.display()
        );
    }
    for prohibited in [
        "std::process::Command",
        "mizar_kernel::",
        "resolution_trace",
        "MiniSAT",
        "DIMACS",
        "instantiated_formula",
        "backend_used_axioms",
        "kernel_verified",
    ] {
        assert!(
            !source.contains(prohibited),
            "{} must not expose prohibited trusted/backend material `{prohibited}`",
            source_path.display()
        );
    }
}

#[test]
fn atp_property_encoding_public_api_surface_is_task_eight_allowlist() {
    let source_path = crate_root().join("src/property_encoding.rs");
    let source = read_to_string(&source_path);
    let public_items = public_api_items(&source);
    let public_fields = public_struct_fields(&source);
    let public_functions = public_api_functions(&source);
    let expected = [
        "AtpPropertyBinderSort",
        "AtpPropertyEncodingBundle",
        "AtpPropertyEncodingError",
        "AtpPropertyEncodingInput",
        "AtpPropertyEncodingStrategy",
        "AtpPropertyFamily",
        "AtpPropertyProjection",
        "AtpPropertyTargetKind",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();

    assert_eq!(
        public_items,
        expected,
        "{} public API must stay limited to task-8 property projection, \
         axiom encoding, generated-binder bundle, and fail-closed error \
         shapes; concrete encoders, backend runners, proof policy, accepted \
         statuses, witnesses, and cache handles require explicit later specs",
        source_path.display()
    );

    assert_eq!(
        public_fields,
        [
            "AtpPropertyBinderSort::source",
            "AtpPropertyBinderSort::symbol",
            "AtpPropertyEncodingInput::existing_declarations",
            "AtpPropertyEncodingInput::existing_provenance",
            "AtpPropertyEncodingInput::existing_symbol_map",
            "AtpPropertyEncodingInput::logic_profile",
            "AtpPropertyEncodingInput::next_declaration_id",
            "AtpPropertyEncodingInput::next_property_id",
            "AtpPropertyEncodingInput::next_provenance_id",
            "AtpPropertyEncodingInput::property_projections",
            "AtpPropertyProjection::binder_sort",
            "AtpPropertyProjection::encoding_strategy",
            "AtpPropertyProjection::family",
            "AtpPropertyProjection::provenance_payload",
            "AtpPropertyProjection::source_property",
            "AtpPropertyProjection::target_arity",
            "AtpPropertyProjection::target_kind",
            "AtpPropertyProjection::target_source",
            "AtpPropertyProjection::target_symbol",
        ],
        "{} public struct fields must stay limited to structured property \
         inputs and id ranges; backend material, accepted statuses, witnesses, \
         and cache handles require explicit later specs",
        source_path.display()
    );

    assert_eq!(
        public_functions,
        [
            "AtpPropertyEncodingBundle::declarations",
            "AtpPropertyEncodingBundle::properties",
            "AtpPropertyEncodingBundle::provenance",
            "AtpPropertyEncodingBundle::symbol_map",
            "AtpPropertyFamily::as_str",
            "AtpPropertyFamily::is_empty",
            "AtpPropertyFamily::new",
            "encode_properties",
        ],
        "{} public functions and methods must stay limited to task-8 property \
         encoding construction/accessors",
        source_path.display()
    );
}

#[test]
fn atp_translator_module_has_paired_specs_and_excludes_trusted_material() {
    let en_spec = workspace_root().join("doc/design/mizar-atp/en/translator.md");
    let ja_spec = workspace_root().join("doc/design/mizar-atp/ja/translator.md");
    let source_path = crate_root().join("src/translator.rs");
    let en = read_to_string(&en_spec);
    let ja = read_to_string(&ja_spec);
    let source = read_to_string(&source_path);

    for marker in [
        "structured formula, declaration, and soft-type projection tables",
        "The source `VcIr` must be status `NeedsAtp`",
        "Duplicate declarations",
        "task 5 defines Rust projection input structs",
    ] {
        assert!(
            en.contains(marker),
            "{} must keep task-4 translator spec marker `{marker}`",
            en_spec.display()
        );
    }
    for marker in [
        "structured formula / declaration / soft-type projection table",
        "`NeedsAtp` status",
        "duplicate declaration",
        "task 5 は declaration / soft-type payload 用の Rust projection input",
    ] {
        assert!(
            ja.contains(marker),
            "{} must keep task-4 Japanese translator spec marker `{marker}`",
            ja_spec.display()
        );
    }
    for marker in [
        "pub fn translate_declarations",
        "pub fn translate_problem",
        "VcStatus::NeedsAtp",
        "targets_vc",
        "AtpDeclarationProjection",
        "AtpFormulaProjection",
        "AtpSymbolSourceProjection",
        "AtpSoftTypeProjection",
        "MissingSoftTypeGuard",
        "ExpectedBackendResult::Unsat",
        "AtpProblem::try_new",
    ] {
        assert!(
            source.contains(marker),
            "{} must implement translator.md task-5/task-6 marker `{marker}`",
            source_path.display()
        );
    }
    for prohibited in [
        "std::process::Command",
        "mizar_kernel::",
        "resolution_trace",
        "MiniSAT",
        "DIMACS",
        "instantiated_formula",
        "backend_used_axioms",
        "kernel_verified",
    ] {
        assert!(
            !source.contains(prohibited),
            "{} must not expose prohibited trusted/backend material `{prohibited}`",
            source_path.display()
        );
    }
}

#[test]
fn atp_translator_public_api_surface_is_task_six_allowlist() {
    let source_path = crate_root().join("src/translator.rs");
    let source = read_to_string(&source_path);
    let public_items = public_api_items(&source);
    let public_fields = public_struct_fields(&source);
    let public_functions = public_api_functions(&source);
    let expected = [
        "AtpDeclarationProjection",
        "AtpDeclarationTranslation",
        "AtpDeclarationTranslationInput",
        "AtpFormulaProjection",
        "AtpFormulaProjectionTarget",
        "AtpProjectionKey",
        "AtpProjectionProvenance",
        "AtpSoftTypeProjection",
        "AtpSoftTypeRepresentation",
        "AtpSymbolSourceProjection",
        "AtpTranslationError",
        "AtpTranslationInput",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();

    assert_eq!(
        public_items,
        expected,
        "{} public API must stay limited to task-5/task-6 translator \
         structured-projection and problem-construction shapes; backend \
         runners, proof methods, SAT \
         clauses, accepted statuses, witness types, and cache handles require \
         explicit later specs",
        source_path.display()
    );

    assert_eq!(
        public_fields,
        [
            "AtpDeclarationProjection::arity",
            "AtpDeclarationProjection::key",
            "AtpDeclarationProjection::kind",
            "AtpDeclarationProjection::provenance",
            "AtpDeclarationProjection::symbol",
            "AtpDeclarationProjection::symbol_source",
            "AtpDeclarationTranslationInput::declaration_projections",
            "AtpDeclarationTranslationInput::diagnostics",
            "AtpDeclarationTranslationInput::kernel_handoff",
            "AtpDeclarationTranslationInput::logic_profile",
            "AtpDeclarationTranslationInput::soft_type_projections",
            "AtpDeclarationTranslationInput::vc",
            "AtpDeclarationTranslationInput::vc_set",
            "AtpFormulaProjection::formula",
            "AtpFormulaProjection::handoff_formula_fingerprint",
            "AtpFormulaProjection::handoff_provenance_payload",
            "AtpFormulaProjection::provenance",
            "AtpFormulaProjection::source_identity",
            "AtpFormulaProjection::target",
            "AtpProjectionProvenance::payload",
            "AtpProjectionProvenance::source",
            "AtpSoftTypeProjection::key",
            "AtpSoftTypeProjection::provenance",
            "AtpSoftTypeProjection::representation",
            "AtpTranslationInput::declaration_projections",
            "AtpTranslationInput::diagnostics",
            "AtpTranslationInput::formula_projections",
            "AtpTranslationInput::kernel_handoff",
            "AtpTranslationInput::logic_profile",
            "AtpTranslationInput::soft_type_projections",
            "AtpTranslationInput::vc",
            "AtpTranslationInput::vc_set",
        ],
        "{} public struct fields must stay limited to structured translator \
         inputs and formula projection agreement material; backend material, \
         SAT material, accepted statuses, witnesses, and cache handles require \
         explicit later specs",
        source_path.display()
    );

    assert_eq!(
        public_functions,
        [
            "AtpDeclarationTranslation::declarations",
            "AtpDeclarationTranslation::diagnostics",
            "AtpDeclarationTranslation::logic_profile",
            "AtpDeclarationTranslation::provenance",
            "AtpDeclarationTranslation::symbol_map",
            "AtpDeclarationTranslation::target_binding",
            "AtpDeclarationTranslation::type_context",
            "AtpDeclarationTranslation::vc_id",
            "AtpFormulaProjectionTarget::imported",
            "AtpFormulaProjectionTarget::vc_formula",
            "AtpProjectionKey::as_str",
            "AtpProjectionKey::is_empty",
            "AtpProjectionKey::new",
            "AtpProjectionProvenance::new",
            "translate_declarations",
            "translate_problem",
        ],
        "{} public functions and methods must stay limited to task-5/task-6 \
         translator construction/accessors; backend runners, proof methods, \
         SAT/kernel checks, witnesses, and cache APIs require explicit later \
         specs",
        source_path.display()
    );
}

#[test]
fn atp_problem_module_has_paired_specs_and_excludes_trusted_backend_material() {
    let en_spec = workspace_root().join("doc/design/mizar-atp/en/problem.md");
    let ja_spec = workspace_root().join("doc/design/mizar-atp/ja/problem.md");
    let source_path = crate_root().join("src/problem.rs");
    let en = read_to_string(&en_spec);
    let ja = read_to_string(&ja_spec);
    let source = read_to_string(&source_path);

    for marker in [
        "backend-neutral ATP problem data model",
        "not kernel evidence",
        "not a SAT problem",
        "ExpectedBackendResult::Unsat",
        "not a provenance substitute",
    ] {
        assert!(
            en.contains(marker),
            "{} must keep task-2 problem spec marker `{marker}`",
            en_spec.display()
        );
    }
    for marker in [
        "backend-neutral ATP problem data model",
        "kernel evidence",
        "SAT problem",
        "ExpectedBackendResult::Unsat",
        "provenance の代替ではない",
    ] {
        assert!(
            ja.contains(marker),
            "{} must keep task-2 Japanese problem spec marker `{marker}`",
            ja_spec.display()
        );
    }
    for marker in [
        "pub struct AtpProblem",
        "ExpectedBackendResult::Unsat",
        "MissingSymbolMap",
        "MissingTypeContextBinding",
        "UnsupportedProfileFeature",
    ] {
        assert!(
            source.contains(marker),
            "{} must implement problem.md marker `{marker}`",
            source_path.display()
        );
    }
    for prohibited in [
        "std::process::Command",
        "mizar_kernel::",
        "resolution_trace",
        "MiniSAT",
        "DIMACS",
        "instantiated_formula",
        "backend_used_axioms",
        "kernel_verified",
    ] {
        assert!(
            !source.contains(prohibited),
            "{} must not expose prohibited trusted/backend material `{prohibited}`",
            source_path.display()
        );
    }
}

#[test]
fn atp_problem_public_api_surface_is_spec_backed_allowlist() {
    let source_path = crate_root().join("src/problem.rs");
    let source = read_to_string(&source_path);
    let public_items = public_api_items(&source);
    let public_fields = public_struct_fields(&source);
    let expected = [
        "AtpAtom",
        "AtpBinder",
        "AtpDeclaration",
        "AtpDeclarationId",
        "AtpDeclarationKind",
        "AtpDiagnostic",
        "AtpDiagnosticKey",
        "AtpDiagnosticMessage",
        "AtpFingerprint",
        "AtpFormula",
        "AtpFormulaId",
        "AtpFormulaTree",
        "AtpPayload",
        "AtpProblem",
        "AtpProblemError",
        "AtpProblemId",
        "AtpProblemParts",
        "AtpProfileName",
        "AtpPropertyId",
        "AtpProvenance",
        "AtpProvenanceId",
        "AtpRequiredProofStatus",
        "AtpSourceBinding",
        "AtpSourceRef",
        "AtpSymbolMapEntry",
        "AtpSymbolName",
        "AtpSymbolSource",
        "AtpTargetBinding",
        "AtpTerm",
        "AtpTypeContext",
        "AtpTypeGuard",
        "AtpTypeGuardId",
        "ConcreteFormat",
        "EncodedProperty",
        "EqualitySupport",
        "ExpectedBackendResult",
        "LogicFragment",
        "LogicProfile",
        "NativePropertySupport",
        "PropertyEncoding",
        "QuantifierPolicy",
        "SoftTypeStrategy",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();

    assert_eq!(
        public_items,
        expected,
        "{} public API must stay limited to problem.md data shapes; backend \
         logs, proof methods, SAT clauses, legacy certificates, accepted \
         statuses, used-axiom material, kernel checks, cache handles, and \
         witness types require explicit later specs",
        source_path.display()
    );

    assert_eq!(
        public_fields,
        [
            "AtpProblemParts::axioms",
            "AtpProblemParts::conjecture",
            "AtpProblemParts::declarations",
            "AtpProblemParts::diagnostics",
            "AtpProblemParts::expected_result",
            "AtpProblemParts::logic_profile",
            "AtpProblemParts::properties",
            "AtpProblemParts::provenance",
            "AtpProblemParts::symbol_map",
            "AtpProblemParts::target_binding",
            "AtpProblemParts::type_context",
            "AtpProblemParts::vc_id",
        ],
        "{} public struct fields must stay limited to problem.md construction \
         inputs; backend logs, proof methods, SAT clauses, legacy \
         certificates, accepted statuses, used-axiom material, kernel checks, \
         cache handles, and witness payloads require explicit later specs",
        source_path.display()
    );

    assert!(
        source.contains(
            "#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n\
             #[non_exhaustive]\n\
             pub enum ExpectedBackendResult {\n\
             \x20   Unsat,\n\
             }\n"
        ),
        "{} must keep `Unsat` as the only task-3 expected-result variant",
        source_path.display()
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

fn read_to_string(path: &std::path::Path) -> String {
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

fn workspace_members(manifest: &str) -> Vec<String> {
    let workspace = section(manifest, "workspace");
    let mut members = Vec::new();
    let mut in_members = false;

    for line in workspace {
        if !in_members {
            if let Some((lhs, rhs)) = line.split_once('=')
                && lhs.trim() == "members"
            {
                in_members = true;
                push_member_entries(rhs, &mut members);
                if rhs.contains(']') {
                    break;
                }
            }
            continue;
        }

        push_member_entries(line, &mut members);
        if line.contains(']') {
            break;
        }
    }

    members
}

fn push_member_entries(line: &str, members: &mut Vec<String>) {
    for entry in line.split(',') {
        let member = entry
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .trim()
            .trim_matches('"');
        if !member.is_empty() {
            members.push(member.to_owned());
        }
    }
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

fn public_api_items(source: &str) -> Vec<String> {
    let mut items = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(name) = item_name(trimmed.strip_prefix("pub struct ")) {
            items.push(name.to_owned());
        } else if let Some(name) = item_name(trimmed.strip_prefix("pub enum ")) {
            items.push(name.to_owned());
        } else if let Some(name) = macro_item_name(trimmed, "dense_id!(") {
            items.push(name.to_owned());
        } else if let Some(name) = macro_item_name(trimmed, "string_key!(") {
            items.push(name.to_owned());
        }
    }
    items.sort();
    items
}

fn public_struct_fields(source: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current_struct = None;

    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(name) = item_name(trimmed.strip_prefix("pub struct ")) {
            if trimmed.ends_with('{') {
                current_struct = Some(name.to_owned());
            }
            continue;
        }

        if trimmed == "}" {
            current_struct = None;
            continue;
        }

        if let Some(struct_name) = &current_struct
            && let Some(field) = item_name(trimmed.strip_prefix("pub "))
        {
            fields.push(format!("{struct_name}::{field}"));
        }
    }

    fields.sort();
    fields
}

fn public_api_functions(source: &str) -> Vec<String> {
    let mut functions = Vec::new();
    let mut current_impl = None;
    let mut depth = 0usize;

    for line in source.lines() {
        let trimmed = line.trim();
        if current_impl.is_none()
            && let Some(rest) = trimmed.strip_prefix("impl ")
            && !rest.contains(" for ")
            && trimmed.ends_with('{')
        {
            current_impl = Some((item_name(Some(rest)).expect("impl type").to_owned(), depth));
        }

        if let Some(name) = item_name(
            trimmed
                .strip_prefix("pub const fn ")
                .or_else(|| trimmed.strip_prefix("pub fn ")),
        ) {
            if let Some((impl_type, _)) = &current_impl {
                functions.push(format!("{impl_type}::{name}"));
            } else {
                functions.push(name.to_owned());
            }
        }

        depth = depth
            .saturating_add(line.chars().filter(|character| *character == '{').count())
            .saturating_sub(line.chars().filter(|character| *character == '}').count());
        if let Some((_, impl_start_depth)) = &current_impl
            && depth == *impl_start_depth
        {
            current_impl = None;
        }
    }

    functions.sort();
    functions
}

fn item_name(rest: Option<&str>) -> Option<&str> {
    let rest = rest?;
    let end = rest
        .find(|character: char| !(character == '_' || character.is_ascii_alphanumeric()))
        .unwrap_or(rest.len());
    (end > 0).then_some(&rest[..end])
}

fn macro_item_name<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(prefix)?;
    rest.strip_suffix(");")
}

fn rust_source_files(root: &std::path::Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_source_files(path: &std::path::Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())) {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn crate_files() -> Vec<String> {
    let root = crate_root();
    let mut files = Vec::new();
    collect_crate_files(&root, &root, &mut files);
    files
}

fn collect_crate_files(root: &std::path::Path, path: &std::path::Path, files: &mut Vec<String>) {
    for entry in fs::read_dir(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())) {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        let name = entry.file_name();
        if name == "target" {
            continue;
        }

        if path.is_dir() {
            collect_crate_files(root, &path, files);
        } else {
            files.push(
                path.strip_prefix(root)
                    .expect("crate file lives in crate root")
                    .display()
                    .to_string(),
            );
        }
    }
}
