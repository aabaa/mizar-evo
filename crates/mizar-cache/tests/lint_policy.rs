use std::{collections::BTreeMap, fs, path::PathBuf};

#[test]
fn cache_manifest_opts_into_workspace_lints() {
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
fn workspace_manifest_includes_mizar_cache_once() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let members = workspace_members(&manifest);
    let occurrences = members
        .iter()
        .filter(|member| **member == "crates/mizar-cache")
        .count();

    assert_eq!(
        occurrences,
        1,
        "{} must list crates/mizar-cache exactly once in [workspace].members",
        manifest_path.display()
    );
}

#[test]
fn cache_manifest_keeps_task_one_package_metadata() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let package = section(&manifest, "package");
    let lib = section(&manifest, "lib");

    assert!(
        package
            .iter()
            .any(|line| assignment_is(line, "name", "mizar-cache")),
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
            .any(|line| assignment_is(line, "name", "mizar_cache")),
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
fn cache_manifest_dependency_boundary_matches_cache_key_builder() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [
            (
                "dependencies".to_owned(),
                vec![
                    "blake3 = \"1.8.5\"",
                    "mizar-artifact = { path = \"../mizar-artifact\" }",
                    "mizar-proof = { path = \"../mizar-proof\" }",
                    "mizar-session = { path = \"../mizar-session\" }",
                    "mizar-vc = { path = \"../mizar-vc\" }",
                ],
            ),
            (
                "dev-dependencies".to_owned(),
                vec!["mizar-core = { path = \"../mizar-core\" }"],
            ),
        ],
        "{} must keep production dependencies limited to the cache-key and \
         dependency-fingerprint hash implementation plus mizar-session, \
         mizar-artifact, mizar-proof, and mizar-vc, with mizar-core allowed \
         only as a task-5 dev dependency for VC projection fixtures; build/target \
         dependency sections require a later explicit task",
        manifest_path.display()
    );
}

#[test]
fn cache_lib_states_boundary_and_cache_key_module() {
    let lib_path = crate_root().join("src/lib.rs");
    let source = read_to_string(&lib_path);

    for marker in [
        "#![forbid(unsafe_code)]",
        "Internal build cache keys",
        "does not accept proofs",
        "Trusted acceptance comes only from `mizar-kernel`",
        "not promoted into kernel-verified status",
        "used_axioms",
    ] {
        assert!(
            source.contains(marker),
            "{} must keep the task-1 cache-boundary marker `{marker}`",
            lib_path.display()
        );
    }

    assert_eq!(
        public_api_declarations(&source),
        [
            "pub mod cache_key;",
            "pub mod dependency_fingerprint;",
            "pub mod cache_store;",
            "pub mod proof_reuse;",
            "pub mod cluster_db;"
        ],
        "{} must expose only the task-13 cache_key, dependency_fingerprint, \
         cache_store, proof_reuse, and cluster_db APIs",
        lib_path.display()
    );
}

#[test]
fn cache_key_api_does_not_expose_proof_authority_terms() {
    let cache_key_path = crate_root().join("src/cache_key.rs");
    let source = read_to_string(&cache_key_path);
    let public_surface = public_api_surface_lines(&source);

    for forbidden in [
        "KernelCheckResult",
        "ProofStatus",
        "used_axioms",
        "TrustedAcceptance",
        "Authority",
        "Accept",
    ] {
        assert!(
            public_surface
                .iter()
                .all(|declaration| !declaration.contains(forbidden)),
            "{} must not expose proof-authority projection terms through the \
             cache_key public API; found `{forbidden}` in {public_surface:?}",
            cache_key_path.display()
        );
    }
}

#[test]
fn cache_key_implementation_excludes_mutable_runtime_inputs() {
    let cache_key_path = crate_root().join("src/cache_key.rs");
    let source = read_to_string(&cache_key_path);
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation prefix exists");

    for forbidden in [
        "std::fs",
        "std::time",
        "SystemTime",
        "Instant",
        "read_dir",
        "cache_dir",
        "scheduler",
        "thread::current",
        "process::id",
        "temp_dir",
    ] {
        assert!(
            !implementation.contains(forbidden),
            "{} cache_key implementation must remain a pure projection and \
             exclude mutable runtime input `{forbidden}`",
            cache_key_path.display()
        );
    }
}

#[test]
fn dependency_fingerprint_api_does_not_expose_proof_authority_terms() {
    let dependency_fingerprint_path = crate_root().join("src/dependency_fingerprint.rs");
    let source = read_to_string(&dependency_fingerprint_path);
    let public_surface = public_api_surface_lines(&source);

    for forbidden in [
        "KernelCheckResult",
        "ProofStatus",
        "used_axioms",
        "TrustedAcceptance",
        "Authority",
    ] {
        assert!(
            public_surface
                .iter()
                .all(|declaration| !declaration.contains(forbidden)),
            "{} must not expose proof-authority projection terms through the \
             dependency_fingerprint public API; found `{forbidden}` in \
             {public_surface:?}",
            dependency_fingerprint_path.display()
        );
    }
}

#[test]
fn dependency_fingerprint_implementation_excludes_mutable_runtime_inputs() {
    let dependency_fingerprint_path = crate_root().join("src/dependency_fingerprint.rs");
    let source = read_to_string(&dependency_fingerprint_path);
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation prefix exists");

    for forbidden in [
        "std::fs",
        "std::time",
        "SystemTime",
        "Instant",
        "read_dir",
        "cache_dir",
        "scheduler",
        "thread::current",
        "process::id",
        "temp_dir",
        "KernelCheckResult",
        "used_axioms",
    ] {
        assert!(
            !implementation.contains(forbidden),
            "{} dependency_fingerprint implementation must remain a pure \
             projection and exclude mutable runtime or proof-authority input \
             `{forbidden}`",
            dependency_fingerprint_path.display()
        );
    }
}

#[test]
fn cache_store_api_does_not_expose_proof_authority_terms() {
    let cache_store_path = crate_root().join("src/cache_store.rs");
    let source = read_to_string(&cache_store_path);
    let public_surface = public_api_surface_lines(&source);

    for forbidden in [
        "KernelCheckResult",
        "ProofStatus",
        "used_axioms",
        "TrustedAcceptance",
        "Authority",
        "Accept",
    ] {
        assert!(
            public_surface
                .iter()
                .all(|declaration| !declaration.contains(forbidden)),
            "{} must not expose proof-authority projection terms through the \
             cache_store public API; found `{forbidden}` in {public_surface:?}",
            cache_store_path.display()
        );
    }
}

#[test]
fn cache_store_implementation_keeps_boundary_terms_out_of_reuse_logic() {
    let cache_store_path = crate_root().join("src/cache_store.rs");
    let source = read_to_string(&cache_store_path);
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation prefix exists");

    for forbidden in [
        "SystemTime",
        "Instant",
        "scheduler",
        "thread::current",
        "process::id",
        "temp_dir",
        "KernelCheckResult",
        "used_axioms",
    ] {
        assert!(
            !implementation.contains(forbidden),
            "{} cache_store implementation must not use mutable runtime or \
             proof-authority input `{forbidden}` for reusable cache decisions",
            cache_store_path.display()
        );
    }
}

#[test]
fn proof_reuse_api_does_not_expose_authority_results_or_publication_tokens() {
    let proof_reuse_path = crate_root().join("src/proof_reuse.rs");
    let source = read_to_string(&proof_reuse_path);
    let public_surface = public_api_surface_lines(&source);

    for forbidden in [
        "KernelCheckResult",
        "ProofStatusProjection",
        "TrustedUsedAxiomsRef",
        "ArtifactStatusPublication",
        "CommittedWitnessPublication",
        "ProofWitnessPublishedRef",
        "ProofWitnessPublication",
        "ProofWitnessRef",
        "PublicationToken",
        "ResolutionTrace",
        "TraceConstruction",
        "TraceConstructor",
        "ClusterTraceBuilder",
        "ReductionSelector",
        "Scheduler",
        "IrCacheAdapter",
        "scheduler_hook",
        "trace_constructor",
        "cluster_trace_builder",
        "reduction_selector",
        "select_reduction",
        "infer_trace",
        "artifact_publication",
        "proof_authority",
        "ir_cache_adapter",
        "mizar_ir",
        "witness_store",
    ] {
        assert!(
            public_surface
                .iter()
                .all(|declaration| !declaration.contains(forbidden)),
            "{} must not expose proof authority, publication, scheduler, or IR \
             adapter APIs through proof_reuse; found `{forbidden}` in \
             {public_surface:?}",
            proof_reuse_path.display()
        );
    }
}

#[test]
fn proof_reuse_implementation_has_no_downstream_stub_or_timing_inputs() {
    let proof_reuse_path = crate_root().join("src/proof_reuse.rs");
    let source = read_to_string(&proof_reuse_path);
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation prefix exists");

    for forbidden in [
        "std::fs",
        "std::time",
        "SystemTime",
        "Instant",
        "thread::current",
        "process::id",
        "mizar_build",
        "mizar_ir",
        "schedule_task",
        "scheduler_hook",
        "publish_ref",
        "artifact_publication",
        "proof_authority",
        "ir_cache_adapter",
        "KernelCheckResult",
        "project_status",
        "CommittedWitnessPublication",
        "ProofWitnessPublishedRef",
        "ProofWitnessPublication",
        "ProofWitnessRef",
        "ArtifactStatusPublication",
        "PublicationToken",
        "ResolutionTrace",
        "TraceConstruction",
        "TraceConstructor",
        "ClusterTraceBuilder",
        "ReductionSelector",
        "trace_constructor",
        "cluster_trace_builder",
        "reduction_selector",
        "select_reduction",
        "infer_trace",
        "witness_store",
    ] {
        assert!(
            !implementation.contains(forbidden),
            "{} proof_reuse implementation must remain local validation only \
             and must not add downstream stubs, publication shortcuts, proof \
             authority calls, or timing input `{forbidden}`",
            proof_reuse_path.display()
        );
    }
}

#[test]
fn cluster_db_api_does_not_expose_proof_authority_or_downstream_stubs() {
    let cluster_db_path = crate_root().join("src/cluster_db.rs");
    let source = read_to_string(&cluster_db_path);
    let public_surface = public_api_surface_lines(&source);

    for forbidden in [
        "KernelCheckResult",
        "ProofStatusProjection",
        "TrustedUsedAxiomsRef",
        "ArtifactStatusPublication",
        "CommittedWitnessPublication",
        "ProofWitnessPublishedRef",
        "ProofWitnessPublication",
        "ProofWitnessRef",
        "PublicationToken",
        "ResolutionTrace",
        "TraceConstruction",
        "TraceConstructor",
        "ClusterTraceBuilder",
        "ReductionSelector",
        "Scheduler",
        "IrCacheAdapter",
        "scheduler_hook",
        "trace_constructor",
        "cluster_trace_builder",
        "reduction_selector",
        "select_reduction",
        "infer_trace",
        "artifact_publication",
        "proof_authority",
        "ir_cache_adapter",
        "mizar_ir",
        "witness_store",
    ] {
        assert!(
            public_surface
                .iter()
                .all(|declaration| !declaration.contains(forbidden)),
            "{} must not expose proof authority, publication, scheduler, or IR \
             adapter APIs through cluster_db; found `{forbidden}` in \
             {public_surface:?}",
            cluster_db_path.display()
        );
    }
}

#[test]
fn cluster_db_implementation_has_no_downstream_stub_or_timing_inputs() {
    let cluster_db_path = crate_root().join("src/cluster_db.rs");
    let source = read_to_string(&cluster_db_path);
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation prefix exists");

    for forbidden in [
        "std::fs",
        "std::time",
        "SystemTime",
        "Instant",
        "thread::current",
        "process::id",
        "mizar_build",
        "mizar_ir",
        "schedule_task",
        "scheduler_hook",
        "publish_ref",
        "artifact_publication",
        "proof_authority",
        "ir_cache_adapter",
        "KernelCheckResult",
        "project_status",
        "CommittedWitnessPublication",
        "ProofWitnessPublishedRef",
        "ProofWitnessPublication",
        "ProofWitnessRef",
        "ArtifactStatusPublication",
        "PublicationToken",
        "ResolutionTrace",
        "TraceConstruction",
        "TraceConstructor",
        "ClusterTraceBuilder",
        "ReductionSelector",
        "trace_constructor",
        "cluster_trace_builder",
        "reduction_selector",
        "select_reduction",
        "infer_trace",
        "witness_store",
    ] {
        assert!(
            !implementation.contains(forbidden),
            "{} cluster_db implementation must remain accepted-origin indexing \
             only and must not add downstream stubs, publication shortcuts, \
             proof authority calls, or timing input `{forbidden}`",
            cluster_db_path.display()
        );
    }
}

#[test]
fn cache_crate_tree_contains_only_task_twenty_files() {
    let mut files = crate_files();
    files.sort();

    assert_eq!(
        files,
        [
            "Cargo.toml",
            "src/cache_key.rs",
            "src/cache_store.rs",
            "src/cluster_db.rs",
            "src/dependency_fingerprint.rs",
            "src/lib.rs",
            "src/proof_reuse.rs",
            "tests/determinism_suite.rs",
            "tests/incremental_contract.rs",
            "tests/lint_policy.rs"
        ],
        "mizar-cache task 20 contains only the crate manifest, root module, \
         cache_key implementation, dependency_fingerprint implementation, \
         cache_store implementation, proof_reuse implementation, cluster_db \
         implementation, determinism suite, incremental contract suite, and \
         lint guard; other behavior modules, build scripts, examples, benches, \
         or extra tests require later explicit tasks; found {files:?}"
    );
}

#[test]
fn cache_source_spec_audit_covers_public_modules_and_gaps() {
    let en_path = workspace_root().join("doc/design/mizar-cache/en/source_spec_audit.md");
    let ja_path = workspace_root().join("doc/design/mizar-cache/ja/source_spec_audit.md");
    let en_audit = read_to_string(&en_path);
    let ja_audit = read_to_string(&ja_path);
    let en_ledger =
        read_to_string(&workspace_root().join("doc/design/mizar-cache/en/task_ledger.md"));
    let ja_ledger =
        read_to_string(&workspace_root().join("doc/design/mizar-cache/ja/task_ledger.md"));

    for marker in [
        "integration_readiness.md",
        "todo.md",
        "crates/mizar-cache/tests/lint_policy.rs",
        "crates/mizar-cache/tests/determinism_suite.rs",
        "crates/mizar-cache/tests/incremental_contract.rs",
        "external_dependency_gap",
        "deferred",
        "repo_metadata_conflict",
        "KernelCheckResult",
        "used_axioms",
    ] {
        assert!(
            en_audit.contains(marker),
            "{} must mention task-18 audit marker `{marker}`",
            en_path.display()
        );
        assert!(
            ja_audit.contains(marker),
            "{} must mention task-18 audit marker `{marker}`",
            ja_path.display()
        );
    }

    for source in rust_source_files() {
        let source_path = crate_root().join(&source);
        let source_text = read_to_string(&source_path);
        let documented_path = format!("crates/mizar-cache/{source}");
        assert_audit_marker(&en_audit, &en_path, &documented_path);
        assert_audit_marker(&ja_audit, &ja_path, &documented_path);

        for marker in public_api_audit_markers(&source_text) {
            assert_audit_marker(&en_audit, &en_path, &marker);
            assert_audit_marker(&ja_audit, &ja_path, &marker);
        }
    }

    let en_ledger_gaps = gap_class_map(&en_ledger);
    let ja_ledger_gaps = gap_class_map(&ja_ledger);
    let en_audit_gaps = gap_class_map(&en_audit);
    let ja_audit_gaps = gap_class_map(&ja_audit);
    assert_eq!(
        en_ledger_gaps, ja_ledger_gaps,
        "EN/JA mizar-cache task ledgers must agree on deferred/external gap IDs and classes"
    );
    assert_eq!(
        en_audit_gaps,
        en_ledger_gaps,
        "{} must repeat every ledger deferred/external gap ID with the same class",
        en_path.display()
    );
    assert_eq!(
        ja_audit_gaps,
        en_ledger_gaps,
        "{} must repeat every ledger deferred/external gap ID with the same class",
        ja_path.display()
    );

    for marker in source_spec_audit_test_markers() {
        assert!(
            test_names().iter().any(|name| name == marker),
            "task-18 audit test marker `{marker}` must name an existing test"
        );
        assert_audit_marker(&en_audit, &en_path, marker);
        assert_audit_marker(&ja_audit, &ja_path, marker);
    }

    for marker in [
        "Task 18 introduced this audit",
        "no unclassified",
        "Task 20 introduces no new gap IDs",
        "The cache remains an internal optimization owner",
    ] {
        assert!(
            en_audit.contains(marker),
            "{} must keep English task-18 audit conclusion marker `{marker}`",
            en_path.display()
        );
    }
    for marker in [
        "task 18 は public",
        "未分類",
        "task 20 の新規 gap ID は追加しない",
        "Cache は internal optimization owner",
    ] {
        assert!(
            ja_audit.contains(marker),
            "{} must keep Japanese task-18 audit conclusion marker `{marker}`",
            ja_path.display()
        );
    }
}

fn assert_audit_marker(audit: &str, path: &std::path::Path, marker: &str) {
    assert!(
        audit.contains(marker),
        "{} must mention task-18 audit marker `{marker}`",
        path.display()
    );
}

fn public_api_audit_markers(source: &str) -> Vec<String> {
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation prefix exists");
    let public_type_names = public_type_names(implementation);
    let mut markers = Vec::new();
    let mut depth = 0usize;
    let mut impl_owner: Option<String> = None;

    for raw_line in implementation.lines() {
        let line = raw_line.trim_start();
        if line.is_empty() || line.starts_with("//") || line.starts_with("#[") {
            continue;
        }

        if depth == 0 {
            if let Some(name) = public_item_name(line) {
                markers.push(name);
            }
            if let Some(owner) = impl_owner_name(line)
                && public_type_names.iter().any(|name| name == &owner)
            {
                impl_owner = Some(owner);
            }
        } else if let Some(owner) = &impl_owner
            && let Some(method) = public_fn_name(line)
        {
            markers.push(format!("{owner}::{method}"));
        }

        depth = depth
            .saturating_add(line.matches('{').count())
            .saturating_sub(line.matches('}').count());
        if depth == 0 {
            impl_owner = None;
        }
    }

    markers.sort();
    markers.dedup();
    markers
}

fn public_type_names(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            public_type_name(trimmed)
        })
        .collect()
}

fn public_type_name(line: &str) -> Option<String> {
    line.strip_prefix("pub struct ")
        .or_else(|| line.strip_prefix("pub enum "))
        .map(identifier_prefix)
}

fn public_item_name(line: &str) -> Option<String> {
    for prefix in [
        "pub const ",
        "pub enum ",
        "pub fn ",
        "pub mod ",
        "pub static ",
        "pub struct ",
        "pub trait ",
        "pub type ",
    ] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return Some(identifier_prefix(rest));
        }
    }
    None
}

fn public_fn_name(line: &str) -> Option<String> {
    line.strip_prefix("pub fn ")
        .or_else(|| line.strip_prefix("pub const fn "))
        .map(identifier_prefix)
}

fn impl_owner_name(line: &str) -> Option<String> {
    let rest = line.strip_prefix("impl ")?;
    if rest.contains(" for ") || rest.starts_with('<') {
        return None;
    }
    let owner = identifier_prefix(rest);
    (!owner.is_empty()).then_some(owner)
}

fn identifier_prefix(text: &str) -> String {
    text.split(|ch: char| !(ch == '_' || ch.is_ascii_alphanumeric()))
        .next()
        .expect("split always yields a prefix")
        .to_owned()
}

fn gap_class_map(document: &str) -> BTreeMap<String, String> {
    let mut gaps = BTreeMap::new();
    for line in document.lines() {
        let cells = line.split('|').map(str::trim).collect::<Vec<_>>();
        if cells.len() < 4 {
            continue;
        }
        let id = cells[1].trim_matches('`');
        let class = cells[2].trim_matches('`');
        if id.contains("-G") && matches!(class, "deferred" | "external_dependency_gap") {
            gaps.insert(id.to_owned(), class.to_owned());
        }
    }
    gaps
}

fn test_names() -> Vec<String> {
    let mut names = Vec::new();
    for source in crate_files()
        .into_iter()
        .filter(|file| file.ends_with(".rs"))
    {
        let text = read_to_string(&crate_root().join(source));
        let mut previous_line_was_test_attr = false;
        for raw_line in text.lines() {
            let line = raw_line.trim_start();
            if line == "#[test]" {
                previous_line_was_test_attr = true;
                continue;
            }
            if previous_line_was_test_attr {
                if let Some(rest) = line.strip_prefix("fn ") {
                    names.push(identifier_prefix(rest));
                }
                previous_line_was_test_attr = false;
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

fn source_spec_audit_test_markers() -> &'static [&'static str] {
    &[
        "cache_key_api_does_not_expose_proof_authority_terms",
        "cache_key_implementation_excludes_mutable_runtime_inputs",
        "key_builder_is_deterministic_and_sorts_all_vectors",
        "every_semantic_field_changes_final_hash",
        "diagnostic_refs_participate_only_when_supplied_and_nondeterministic_inputs_are_absent",
        "dependency_fingerprint_api_does_not_expose_proof_authority_terms",
        "dependency_fingerprint_implementation_excludes_mutable_runtime_inputs",
        "non_interface_summary_metadata_is_excluded_from_importer_visible_fingerprint",
        "interface_change_invalidates_importer_visible_fingerprint",
        "implementation_only_change_does_not_change_importer_visible_subset",
        "missing_unknown_and_uncacheable_inputs_force_miss",
        "proof_reuse_validation_failures_force_miss_without_granting_trust",
        "unsupported_footprint_schema_produces_no_footprint",
        "cache_store_api_does_not_expose_proof_authority_terms",
        "cache_store_implementation_keeps_boundary_terms_out_of_reuse_logic",
        "cache_store_deletion_changes_only_lookup_availability",
        "trusted_incremental_contract_requires_complete_cross_module_validation",
        "missing_or_unknown_incremental_inputs_fail_closed_before_reuse",
        "proof_reuse_requires_each_architecture_22_validation_field",
        "dependency_footprint_projects_missing_and_external_proof_metadata_to_miss",
        "cache_deletion_and_diagnostic_order_are_non_semantic",
        "externally_attested_evidence_never_becomes_trusted_reuse",
        "proof_reuse_api_does_not_expose_authority_results_or_publication_tokens",
        "proof_reuse_implementation_has_no_downstream_stub_or_timing_inputs",
        "proof_reuse_validation_is_deterministic_and_never_promotes_external_evidence",
        "cluster_db_api_does_not_expose_proof_authority_or_downstream_stubs",
        "cluster_db_implementation_has_no_downstream_stub_or_timing_inputs",
        "public_cache_enums_are_forward_compatible_and_documented",
    ]
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
fn public_cache_enums_are_forward_compatible_and_documented() {
    let expected = public_enum_policy();
    let mut expected_pairs = expected
        .iter()
        .map(|entry| (entry.source.to_owned(), entry.name.to_owned()))
        .collect::<Vec<_>>();
    expected_pairs.sort();

    let mut actual_pairs = Vec::new();
    let mut violations = Vec::new();
    for source in rust_source_files() {
        let text = read_to_string(&crate_root().join(&source));
        for declaration in public_enum_declarations(&source, &text) {
            if !declaration.non_exhaustive {
                violations.push(format!(
                    "{}:{}: public enum `{}` must keep #[non_exhaustive]",
                    source, declaration.line_number, declaration.name
                ));
            }
            actual_pairs.push((source.clone(), declaration.name));
        }
    }
    actual_pairs.sort();

    assert_eq!(
        actual_pairs, expected_pairs,
        "each public mizar-cache enum must be classified by the task-17 \
         forward-compatibility policy"
    );

    for entry in expected {
        assert_documented_enum(
            entry.en_doc,
            "## Public Enum Policy",
            "No exhaustive public enum exceptions are owned by this module",
            entry.name,
        );
        assert_documented_enum(
            entry.ja_doc,
            "## 公開 enum policy",
            "この module が所有する exhaustive public enum exception はない",
            entry.name,
        );
    }

    assert!(
        violations.is_empty(),
        "public mizar-cache enum forward-compatibility policy drift:\n{}",
        violations.join("\n")
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

fn section<'a>(document: &'a str, name: &str) -> Vec<&'a str> {
    let header = format!("[{name}]");
    let mut in_section = false;
    let mut lines = Vec::new();

    for line in document.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed == header;
        } else if in_section && !trimmed.is_empty() && !trimmed.starts_with('#') {
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

fn workspace_members(manifest: &str) -> Vec<String> {
    let mut members = Vec::new();
    let mut in_members = false;

    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_members = false;
            continue;
        }
        if trimmed.starts_with("members") && trimmed.contains('[') {
            in_members = true;
            continue;
        }
        if in_members {
            if trimmed.starts_with(']') {
                break;
            }
            if let Some(member) = trimmed.trim_end_matches(',').trim().strip_prefix('"') {
                members.push(member.trim_end_matches('"').to_owned());
            }
        }
    }

    members
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

fn public_api_declarations(source: &str) -> Vec<String> {
    let public_markers = [
        "pub const ",
        "pub enum ",
        "pub fn ",
        "pub mod ",
        "pub static ",
        "pub struct ",
        "pub trait ",
        "pub type ",
        "pub use ",
    ];

    source
        .lines()
        .map(str::trim_start)
        .filter_map(|line| {
            public_markers
                .iter()
                .find(|marker| line.starts_with(**marker))
                .map(|_| line.to_owned())
        })
        .collect()
}

fn public_api_surface_lines(source: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut in_public_type = false;
    let mut depth = 0usize;

    for raw_line in source
        .split("#[cfg(test)]")
        .next()
        .expect("implementation prefix exists")
        .lines()
    {
        let line = raw_line.trim_start();
        if line.starts_with("//") || line.starts_with("#[") || line.is_empty() {
            continue;
        }

        if line.starts_with("pub ") {
            lines.push(line.to_owned());
            in_public_type = starts_public_type(line);
        } else if in_public_type {
            lines.push(line.to_owned());
        }

        if in_public_type {
            depth = depth
                .saturating_add(line.matches('{').count())
                .saturating_sub(line.matches('}').count());
            if depth == 0 {
                in_public_type = false;
            }
        }
    }

    lines
}

fn starts_public_type(line: &str) -> bool {
    line.starts_with("pub struct ")
        || line.starts_with("pub enum ")
        || line.starts_with("pub trait ")
}

fn crate_files() -> Vec<String> {
    let root = crate_root();
    let mut files = Vec::new();
    collect_files(&root, &root, &mut files);
    files
}

fn collect_files(root: &std::path::Path, dir: &std::path::Path, files: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|error| panic!("{}: {error}", dir.display())) {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, files);
        } else {
            files.push(
                path.strip_prefix(root)
                    .expect("file lives under crate root")
                    .display()
                    .to_string(),
            );
        }
    }
}

struct PublicEnumPolicy {
    source: &'static str,
    name: &'static str,
    en_doc: &'static str,
    ja_doc: &'static str,
}

struct PublicEnumDeclaration {
    name: String,
    line_number: usize,
    non_exhaustive: bool,
}

fn public_enum_policy() -> &'static [PublicEnumPolicy] {
    PUBLIC_ENUM_POLICY
}

const PUBLIC_ENUM_POLICY: &[PublicEnumPolicy] = &[
    cache_key_enum("FootprintCompleteness"),
    cache_key_enum("CacheKeyBuildOutcome"),
    cache_key_enum("CacheKeyBuildRejection"),
    cache_store_enum("CacheOutputDescriptor"),
    cache_store_enum("CacheLookupOutcome"),
    cache_store_enum("CacheInsertOutcome"),
    cache_store_enum("CacheMiss"),
    cache_store_enum("CacheStoreError"),
    dependency_fingerprint_enum("FingerprintTargetKind"),
    dependency_fingerprint_enum("ProofReuseValidationState"),
    dependency_fingerprint_enum("DependencyFootprintCompleteness"),
    dependency_fingerprint_enum("DependencyFootprintBuildOutcome"),
    dependency_fingerprint_enum("DependencyFootprintBuildRejection"),
    dependency_fingerprint_enum("RebuildTrigger"),
    dependency_fingerprint_enum("FingerprintChangeKind"),
    dependency_fingerprint_enum("DependencySlicePrecision"),
    proof_reuse_enum("ProofReuseValidationOutcome"),
    proof_reuse_enum("ProofReuseMissReason"),
    cluster_db_enum("ClusterContributionVisibility"),
    cluster_db_enum("ClusterContributionStatus"),
    cluster_db_enum("ClusterContributionKind"),
    cluster_db_enum("ClusterOriginFootprintCompleteness"),
    cluster_db_enum("ClusterIndexEntryKind"),
    cluster_db_enum("ClusterDbViewMiss"),
    cluster_db_enum("ClusterDbWriteRejection"),
];

const fn cache_key_enum(name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        source: "src/cache_key.rs",
        name,
        en_doc: "doc/design/mizar-cache/en/cache_key.md",
        ja_doc: "doc/design/mizar-cache/ja/cache_key.md",
    }
}

const fn cache_store_enum(name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        source: "src/cache_store.rs",
        name,
        en_doc: "doc/design/mizar-cache/en/cache_store.md",
        ja_doc: "doc/design/mizar-cache/ja/cache_store.md",
    }
}

const fn dependency_fingerprint_enum(name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        source: "src/dependency_fingerprint.rs",
        name,
        en_doc: "doc/design/mizar-cache/en/dependency_fingerprint.md",
        ja_doc: "doc/design/mizar-cache/ja/dependency_fingerprint.md",
    }
}

const fn proof_reuse_enum(name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        source: "src/proof_reuse.rs",
        name,
        en_doc: "doc/design/mizar-cache/en/proof_reuse.md",
        ja_doc: "doc/design/mizar-cache/ja/proof_reuse.md",
    }
}

const fn cluster_db_enum(name: &'static str) -> PublicEnumPolicy {
    PublicEnumPolicy {
        source: "src/cluster_db.rs",
        name,
        en_doc: "doc/design/mizar-cache/en/cluster_db.md",
        ja_doc: "doc/design/mizar-cache/ja/cluster_db.md",
    }
}

fn rust_source_files() -> Vec<String> {
    crate_files()
        .into_iter()
        .filter(|file| file.starts_with("src/") && file.ends_with(".rs"))
        .collect()
}

fn public_enum_declarations(source: &str, text: &str) -> Vec<PublicEnumDeclaration> {
    let lines = text.lines().collect::<Vec<_>>();
    let tokens = rust_tokens(text);
    let mut declarations = Vec::new();

    for index in 0..tokens.len().saturating_sub(2) {
        if tokens[index].text != "pub"
            || tokens[index + 1].text != "enum"
            || !is_identifier(&tokens[index + 2].text)
        {
            continue;
        };
        declarations.push(PublicEnumDeclaration {
            name: tokens[index + 2].text.clone(),
            line_number: tokens[index].line_number,
            non_exhaustive: previous_attribute_is_non_exhaustive(
                &lines,
                tokens[index].line_number.saturating_sub(1),
            ),
        });
    }

    for declaration in &declarations {
        assert!(
            public_enum_policy()
                .iter()
                .any(|entry| entry.source == source && entry.name == declaration.name),
            "{source}: discovered unclassified public enum `{}`",
            declaration.name
        );
    }

    declarations
}

fn previous_attribute_is_non_exhaustive(lines: &[&str], index: usize) -> bool {
    lines[..index]
        .iter()
        .rev()
        .take_while(|line| {
            let trimmed = line.trim();
            trimmed.is_empty() || trimmed.starts_with("#[") || trimmed.starts_with("///")
        })
        .any(|line| line.trim() == "#[non_exhaustive]")
}

#[derive(Debug, Eq, PartialEq)]
struct RustToken {
    text: String,
    line_number: usize,
}

fn rust_tokens(source: &str) -> Vec<RustToken> {
    let chars = source.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut index = 0;
    let mut line_number = 1;

    while index < chars.len() {
        let ch = chars[index];
        if ch == '\n' {
            line_number += 1;
            index += 1;
            continue;
        }
        if ch.is_whitespace() {
            index += 1;
            continue;
        }
        if ch == '/' && chars.get(index + 1) == Some(&'/') {
            index += 2;
            while index < chars.len() && chars[index] != '\n' {
                index += 1;
            }
            continue;
        }
        if ch == '/' && chars.get(index + 1) == Some(&'*') {
            index += 2;
            while index < chars.len() {
                if chars[index] == '\n' {
                    line_number += 1;
                }
                if chars[index] == '*' && chars.get(index + 1) == Some(&'/') {
                    index += 2;
                    break;
                }
                index += 1;
            }
            continue;
        }
        if ch == '"' {
            index += 1;
            let mut escaped = false;
            while index < chars.len() {
                let literal_ch = chars[index];
                if literal_ch == '\n' {
                    line_number += 1;
                }
                if escaped {
                    escaped = false;
                } else if literal_ch == '\\' {
                    escaped = true;
                } else if literal_ch == '"' {
                    index += 1;
                    break;
                }
                index += 1;
            }
            continue;
        }
        if is_identifier_start(ch) {
            let start = index;
            index += 1;
            while index < chars.len() && is_identifier_continue(chars[index]) {
                index += 1;
            }
            tokens.push(RustToken {
                text: chars[start..index].iter().collect(),
                line_number,
            });
            continue;
        }

        tokens.push(RustToken {
            text: ch.to_string(),
            line_number,
        });
        index += 1;
    }

    tokens
}

fn is_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_identifier_start(first) && chars.all(is_identifier_continue)
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn assert_documented_enum(
    doc: &str,
    heading: &str,
    no_exhaustive_exception_text: &str,
    enum_name: &str,
) {
    let path = workspace_root().join(doc);
    let text = read_to_string(&path);
    assert!(
        public_enum_policy_section(&text, heading).is_some(),
        "{} must contain the task-17 public enum policy section",
        path.display()
    );
    let policy = public_enum_policy_section(&text, heading).expect("checked above");
    assert!(
        policy.contains(no_exhaustive_exception_text),
        "{} must state that there are no exhaustive public enum exceptions",
        path.display()
    );
    let row = format!("| `{enum_name}` |");
    assert!(
        policy
            .lines()
            .any(|line| line.contains(&row) && line.contains("#[non_exhaustive]")),
        "{} must document the #[non_exhaustive] decision for {enum_name}",
        path.display()
    );
}

fn public_enum_policy_section<'a>(document: &'a str, heading: &str) -> Option<&'a str> {
    let start = document.find(heading)?;
    let rest = &document[start..];
    let end = rest
        .find("\n## ")
        .map_or(document.len(), |offset| start + offset);
    Some(&document[start..end])
}
