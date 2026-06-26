use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[test]
fn kernel_manifest_opts_into_workspace_lints() {
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
fn workspace_manifest_includes_mizar_kernel_once() {
    let manifest_path = workspace_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let workspace_members = workspace_members(&manifest);

    assert_eq!(
        workspace_members
            .iter()
            .filter(|member| member.as_str() == "crates/mizar-kernel")
            .count(),
        1,
        "{} must list crates/mizar-kernel exactly once in [workspace].members; found {workspace_members:?}",
        manifest_path.display()
    );
}

#[test]
fn kernel_manifest_keeps_task_one_package_metadata() {
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
            .any(|line| assignment_is(line, "name", "mizar_kernel")),
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
fn kernel_manifest_dependency_boundary_is_task_one_minimal() {
    let manifest_path = crate_root().join("Cargo.toml");
    let manifest = read_to_string(&manifest_path);
    let dependency_sections = dependency_sections(&manifest);

    assert_eq!(
        dependency_sections,
        [(
            "dependencies".to_owned(),
            vec![
                "batsat = { version = \"=0.6.0\", default-features = false }",
                "mizar-core = { path = \"../mizar-core\" }",
                "mizar-session = { path = \"../mizar-session\" }",
            ],
        )],
        "{} must keep the task-1 trusted-kernel dependency boundary exact: \
         production dependencies are the task-27 audited SAT checker plus \
         mizar-core and mizar-session only, and \
         dev/build/target dependency sections require a later explicit task",
        manifest_path.display()
    );
    let lockfile = read_to_string(&workspace_root().join("Cargo.lock"));
    assert_lock_package(
        &lockfile,
        "batsat",
        "0.6.0",
        "ec82b6bbce8ea42f5003417b699267860a9f4dd869fc9ba8faceac761d5afed1",
        &["bit-vec"],
    );
    assert_lock_package(
        &lockfile,
        "bit-vec",
        "0.5.1",
        "f59bbe95d4e52a6398ec21238d31577f2b28a9d86807f06ca59d191d8440d0bb",
        &[],
    );
    let kernel_lock = lock_package(&lockfile, "mizar-kernel").expect("mizar-kernel lock package");
    assert!(
        kernel_lock.contains("\"batsat\""),
        "mizar-kernel lockfile package must depend on audited batsat"
    );
}

#[test]
fn kernel_lib_exposes_only_current_spec_backed_modules() {
    let lib_path = crate_root().join("src/lib.rs");
    let source = read_to_string(&lib_path);
    let source_files = rust_source_files(&crate_root().join("src"))
        .into_iter()
        .map(|path| {
            path.strip_prefix(crate_root())
                .expect("source path lives in crate root")
                .display()
                .to_string()
        })
        .collect::<Vec<_>>();
    let declarations = public_semantic_declarations(&source);

    assert!(
        source.contains("#![forbid(unsafe_code)]"),
        "{} must forbid unsafe code at the trusted-kernel crate root",
        lib_path.display()
    );
    for marker in [
        "Trusted certificate checking boundary",
        "owns pipeline phase 14",
        "checks evidence only",
        "must not perform proof search",
        "overload resolution",
        "cluster search",
        "ATP search",
        "implicit coercion",
        "fallback inference",
        "global-state",
    ] {
        assert!(
            source.contains(marker),
            "{} must keep the task-1 trusted-kernel statement marker `{marker}`",
            lib_path.display()
        );
    }
    assert_eq!(
        declarations,
        [
            "12: pub mod certificate_parser;",
            "13: pub mod checker;",
            "14: pub mod clause;",
            "15: pub mod formula_evidence;",
            "16: pub mod rejection;",
            "17: pub mod resolution_trace;",
            "18: pub mod sat_checker;",
            "19: pub mod sat_encoding;",
            "20: pub mod substitution_checker;",
            "compact: pubmod",
        ],
        "{} must expose only current spec-backed kernel modules; found:\n{}",
        lib_path.display(),
        declarations.join("\n")
    );
    assert_eq!(
        source_files,
        [
            "src/certificate_parser/tests.rs",
            "src/certificate_parser.rs",
            "src/checker/tests.rs",
            "src/checker.rs",
            "src/clause/tests.rs",
            "src/clause.rs",
            "src/formula_evidence/tests.rs",
            "src/formula_evidence.rs",
            "src/lib.rs",
            "src/rejection/tests.rs",
            "src/rejection.rs",
            "src/resolution_trace/tests.rs",
            "src/resolution_trace.rs",
            "src/sat_checker/tests.rs",
            "src/sat_checker.rs",
            "src/sat_encoding/tests.rs",
            "src/sat_encoding.rs",
            "src/substitution_checker/tests.rs",
            "src/substitution_checker.rs",
        ],
        "kernel source modules require their \
         paired English/Japanese specs first, and private task-22 test modules \
         must stay under their spec-backed parent modules, found {source_files:?}"
    );
    for spec in [
        workspace_root().join("doc/design/mizar-kernel/en/certificate_parser.md"),
        workspace_root().join("doc/design/mizar-kernel/ja/certificate_parser.md"),
        workspace_root().join("doc/design/mizar-kernel/en/checker.md"),
        workspace_root().join("doc/design/mizar-kernel/ja/checker.md"),
        workspace_root().join("doc/design/mizar-kernel/en/clause.md"),
        workspace_root().join("doc/design/mizar-kernel/ja/clause.md"),
        workspace_root().join("doc/design/mizar-kernel/en/formula_evidence.md"),
        workspace_root().join("doc/design/mizar-kernel/ja/formula_evidence.md"),
        workspace_root().join("doc/design/mizar-kernel/en/rejection.md"),
        workspace_root().join("doc/design/mizar-kernel/ja/rejection.md"),
        workspace_root().join("doc/design/mizar-kernel/en/resolution_trace.md"),
        workspace_root().join("doc/design/mizar-kernel/ja/resolution_trace.md"),
        workspace_root().join("doc/design/mizar-kernel/en/sat_checker.md"),
        workspace_root().join("doc/design/mizar-kernel/ja/sat_checker.md"),
        workspace_root().join("doc/design/mizar-kernel/en/sat_encoding.md"),
        workspace_root().join("doc/design/mizar-kernel/ja/sat_encoding.md"),
        workspace_root().join("doc/design/mizar-kernel/en/substitution_checker.md"),
        workspace_root().join("doc/design/mizar-kernel/ja/substitution_checker.md"),
    ] {
        assert!(
            spec.exists(),
            "{} must exist before its matching source module is exposed",
            spec.display()
        );
    }
}

#[test]
fn kernel_public_enums_are_forward_compatible_and_documented() {
    let source_inventory = source_public_enum_inventory();
    assert!(
        !source_inventory.is_empty(),
        "mizar-kernel must keep an explicit public enum inventory"
    );

    for policy_doc in [
        workspace_root().join("doc/design/mizar-kernel/en/public_enum_policy.md"),
        workspace_root().join("doc/design/mizar-kernel/ja/public_enum_policy.md"),
    ] {
        let documented_inventory = documented_public_enum_inventory(&policy_doc);
        assert_eq!(
            documented_inventory,
            source_inventory,
            "{} must list exactly the public enums present in crates/mizar-kernel/src",
            policy_doc.display()
        );
    }

    let missing_markers = public_enums_without_non_exhaustive();
    assert!(
        missing_markers.is_empty(),
        "every mizar-kernel public enum must be immediately preceded by \
         #[non_exhaustive] per doc/design/mizar-kernel/en/public_enum_policy.md:\n{}",
        missing_markers.join("\n")
    );
}

#[test]
fn source_spec_audit_covers_public_surface_and_prohibitions() {
    let audit_paths = [
        workspace_root().join("doc/design/mizar-kernel/en/source_spec_audit.md"),
        workspace_root().join("doc/design/mizar-kernel/ja/source_spec_audit.md"),
    ];
    let lib_source = read_to_string(&crate_root().join("src/lib.rs"));
    let modules = public_module_exports(&lib_source);

    assert!(
        !modules.is_empty(),
        "mizar-kernel source/spec audit must track public module exports"
    );

    for audit_path in &audit_paths {
        let audit = read_to_string(audit_path);
        let documented_modules = documented_audit_modules(&audit);
        assert_eq!(
            documented_modules,
            modules,
            "{} must list exactly the current public module exports",
            audit_path.display()
        );

        for module in &modules {
            let section_heading = format!("### `{module}`");
            let module_section = markdown_section(&audit, &section_heading)
                .unwrap_or_else(|| panic!("{} missing {section_heading}", audit_path.display()));
            let source_path = format!("src/{module}.rs");
            let spec_path = format!("{module}.md");

            for marker in [format!("`{module}`"), format!("`{source_path}`"), spec_path] {
                assert!(
                    module_section.contains(&marker),
                    "{} section {section_heading} must mention {marker}",
                    audit_path.display()
                );
            }

            let source = read_to_string(&crate_root().join(&source_path));
            let unsupported_forms = unsupported_top_level_public_forms(&source);
            assert!(
                unsupported_forms.is_empty(),
                "{source_path} has public top-level API forms not yet represented \
                 by the Task20 exact inventory scanner:\n{}",
                unsupported_forms.join("\n")
            );
            let public_items = top_level_public_items(&source);
            let documented_items = documented_public_items(module_section);
            assert!(
                !public_items.is_empty(),
                "{source_path} must have an explicit public surface inventory"
            );
            assert_eq!(
                documented_items,
                public_items,
                "{} section {section_heading} must list exactly the current \
                 top-level public item inventory",
                audit_path.display()
            );
        }

        let gap_classification = markdown_section(&audit, "## Gap Classification")
            .unwrap_or_else(|| panic!("{} missing ## Gap Classification", audit_path.display()));
        for (marker, classes) in [
            (
                "Source-derived certificate",
                &["`external_dependency_gap`", "`deferred`"][..],
            ),
            (
                "ATP proof translation",
                &["`external_dependency_gap`", "`deferred`"],
            ),
            (
                "Cluster/reduction payload production",
                &["`external_dependency_gap`", "`deferred`"],
            ),
            (
                "Derived-fact payload schema",
                &["`external_dependency_gap`", "`deferred`"],
            ),
            (
                "Service-envelope normalization",
                &["`external_dependency_gap`", "`deferred`"],
            ),
            (
                "Downstream `mizar-proof`, `mizar-cache`, and `mizar-artifact`",
                &["`external_dependency_gap`", "`deferred`"],
            ),
            ("Downstream wildcard-arm checks", &["`deferred`"]),
            (
                "`source_undocumented_behavior`",
                &["`source_undocumented_behavior`"],
            ),
            ("`repo_metadata_conflict`", &["`repo_metadata_conflict`"]),
        ] {
            let row = gap_classification
                .lines()
                .find(|line| line.contains(marker))
                .unwrap_or_else(|| {
                    panic!(
                        "{} Gap Classification must include `{marker}`",
                        audit_path.display()
                    )
                });
            for class in classes {
                assert!(
                    row.contains(class),
                    "{} Gap Classification marker `{marker}` must include class {class}",
                    audit_path.display()
                );
            }
        }
    }

    for module in modules {
        for language in ["en", "ja"] {
            let spec_path = workspace_root()
                .join("doc/design/mizar-kernel")
                .join(language)
                .join(format!("{module}.md"));
            let spec = read_to_string(&spec_path);
            let trust_statement = markdown_section(&spec, "## Trust Statement")
                .unwrap_or_else(|| panic!("{} missing ## Trust Statement", spec_path.display()));
            let trust_statement = trust_statement
                .to_ascii_lowercase()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            for marker in [
                "trusted kernel code",
                "no proof search",
                "no sat solving",
                "no atp search or backend invocation",
                "no premise selection",
                "no overload resolution",
                "no cluster search",
                "no implicit coercion insertion",
                "no fallback inference",
                "no acceptance from backend-reported success alone",
                "no source loading",
                "no cache lookup",
                "no artifact lookup",
                "no wall-clock or random-state reads",
                "no unordered iteration dependence",
                "no hidden reads of mutable compiler-global state",
            ] {
                assert!(
                    trust_statement.contains(marker),
                    "{} trust statement must contain trusted-boundary marker `{marker}`",
                    spec_path.display()
                );
            }
        }
    }
}

#[test]
fn task_22_private_tests_are_tracked_and_source_spec_traceable() {
    for path in [
        "crates/mizar-kernel/src/certificate_parser/tests.rs",
        "crates/mizar-kernel/src/checker/tests.rs",
        "crates/mizar-kernel/src/clause/tests.rs",
        "crates/mizar-kernel/src/formula_evidence/tests.rs",
        "crates/mizar-kernel/src/rejection/tests.rs",
        "crates/mizar-kernel/src/resolution_trace/tests.rs",
        "crates/mizar-kernel/src/sat_checker/tests.rs",
        "crates/mizar-kernel/src/sat_encoding/tests.rs",
        "crates/mizar-kernel/src/substitution_checker/tests.rs",
        "doc/design/mizar-kernel/en/module_boundary_audit.md",
        "doc/design/mizar-kernel/ja/module_boundary_audit.md",
    ] {
        assert!(
            workspace_root().join(path).exists(),
            "task-22 split path must exist before commit: {path}"
        );
        assert!(
            git_tracks(path),
            "task-22 split path must be explicitly staged/tracked before \
             verification can pass: {path}"
        );
    }

    let expected_traceability_paths = [
        "crates/mizar-kernel/src/certificate_parser/tests.rs",
        "crates/mizar-kernel/src/checker/tests.rs",
        "crates/mizar-kernel/src/checker/tests.rs",
        "crates/mizar-kernel/src/checker/tests.rs",
        "crates/mizar-kernel/src/clause/tests.rs",
        "crates/mizar-kernel/src/formula_evidence/tests.rs",
        "crates/mizar-kernel/src/rejection/tests.rs",
        "crates/mizar-kernel/src/resolution_trace/tests.rs",
        "crates/mizar-kernel/src/sat_checker/tests.rs",
        "crates/mizar-kernel/src/sat_encoding/tests.rs",
        "crates/mizar-kernel/src/substitution_checker/tests.rs",
        "crates/mizar-kernel/tests/lint_policy.rs",
    ];

    for audit_path in [
        workspace_root().join("doc/design/mizar-kernel/en/source_spec_audit.md"),
        workspace_root().join("doc/design/mizar-kernel/ja/source_spec_audit.md"),
    ] {
        let audit = read_to_string(&audit_path);
        let documented_paths = documented_test_traceability_paths(&audit);
        assert_eq!(
            documented_paths,
            expected_traceability_paths,
            "{} must keep exact task-22 test traceability paths",
            audit_path.display()
        );
    }
}

#[test]
fn kernel_source_stays_off_producer_policy_cache_and_artifact_boundaries() {
    let forbidden = [
        "mizar_atp",
        "mizar_proof",
        "mizar_cache",
        "mizar_artifact",
        "AtpPortfolio",
        "BackendRunner",
        "ProofPolicy",
        "ProofWitnessStore",
        "ArtifactService",
        "Cache",
        "HashSet",
        "HashMap",
        "std::process",
        "std::net",
        "std::fs",
        "Command",
        "TcpStream",
        "UdpSocket",
        "mizar_checker",
        "mizar_resolve",
        "PremiseSelector",
        "premise_selection",
        "ProofSearch",
        "proof_search",
        "AtpSearch",
        "atp_search",
        "ClusterSearch",
        "cluster_search",
        "ImplicitCoercion",
        "insert_coercion",
        "FallbackInference",
        "fallback_inference",
        "HiddenBinderRepair",
        "hidden_binder_repair",
        "binder_repair",
        "SourceNameLookup",
        "source_name_lookup",
        "DisplayNameLookup",
        "display_name_lookup",
        "OmittedTemplateArgument",
        "omitted_template_argument",
        "SatSolver",
        "solve_sat",
        "OnceLock",
        "LazyLock",
        "Mutex",
        "RwLock",
        "AtomicBool",
        "AtomicI8",
        "AtomicI16",
        "AtomicI32",
        "AtomicUsize",
        "AtomicIsize",
        "AtomicU8",
        "AtomicU16",
        "AtomicU32",
        "AtomicU64",
        "AtomicI64",
        "AtomicU128",
        "AtomicI128",
        "AtomicPtr",
        "thread_local!",
        "static mut",
        "SystemTime",
        "Instant",
        "std::time",
        "thread_rng",
        "rand::",
        "batsat::dimacs",
        "batsat::drat",
        "callback",
        "Callback",
        "BasicCallbacks",
        "print_stats",
        "get_model",
        "unsat_core",
    ];
    let mut violations = Vec::new();

    for path in rust_source_files(&crate_root().join("src")) {
        let source = read_to_string(&path);
        for token in forbidden {
            if source.contains(token) {
                let display_path = path.strip_prefix(crate_root()).unwrap_or(&path);
                violations.push(format!("{}: {token}", display_path.display()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "mizar-kernel task-1 source must not couple to ATP/proof/cache/artifact \
         producers or nondeterministic helper surfaces:\n{}",
        violations.join("\n")
    );
}

#[test]
fn sat_checker_public_api_hides_dependency_types() {
    let source = read_to_string(&crate_root().join("src/sat_checker.rs"));
    let public_api = public_api_fragments(&source).join("\n");
    let dependency_symbols = imported_batsat_symbols(&source);
    assert!(
        !dependency_symbols.is_empty(),
        "sat_checker must keep audited batsat imports explicit for public API scanning"
    );
    let mut violations = dependency_symbols
        .into_iter()
        .filter(|symbol| contains_rust_identifier(&public_api, symbol))
        .collect::<Vec<_>>();
    if public_api.contains("batsat::") {
        violations.push("batsat::".to_owned());
    }

    assert!(
        violations.is_empty(),
        "sat_checker public API must not expose dependency-owned SAT types: {violations:?}"
    );
}

#[test]
fn encoded_sat_problem_fields_stay_private() {
    let source = read_to_string(&crate_root().join("src/sat_encoding.rs"));
    let body = struct_body(&source, "EncodedSatProblem");
    let field_violations = [
        "schema_version",
        "encoding_version",
        "target_vc",
        "atom_variables",
        "assertions",
        "clauses",
        "canonical_bytes",
    ]
    .iter()
    .filter_map(|field| {
        body.lines().find_map(|line| {
            let trimmed = line.trim_start();
            let marker = format!("{field}:");
            (trimmed.starts_with("pub") && trimmed.contains(&marker)).then_some(trimmed.to_owned())
        })
    })
    .collect::<Vec<_>>();
    let public_api = public_api_fragments(impl_body(&source, "EncodedSatProblem")).join("\n");
    let escape_violations = [
        "&mut",
        "_mut",
        "into_parts",
        "from_parts",
        "Vec<",
        "Box<[",
        "self) -> (",
    ]
    .into_iter()
    .filter(|token| public_api.contains(token))
    .collect::<Vec<_>>();

    assert!(
        field_violations.is_empty() && escape_violations.is_empty(),
        "EncodedSatProblem must expose read-only accessors, not public mutable \
         fields or mutable/reconstructable public escape hatches: fields={field_violations:?}, \
         public_api={escape_violations:?}"
    );
}

#[test]
fn parsed_kernel_evidence_fields_stay_read_only() {
    let source = read_to_string(&crate_root().join("src/formula_evidence.rs"));
    let body = struct_body(&source, "ParsedKernelEvidence");
    let field_violations = [
        "schema_version",
        "encoding_version",
        "kernel_profile",
        "target_vc",
        "symbol_manifest",
        "variable_manifest",
        "formulas",
        "substitutions",
        "provenance",
        "final_goal",
        "canonical_hash_input",
    ]
    .iter()
    .filter_map(|field| {
        body.lines().find_map(|line| {
            let trimmed = line.trim_start();
            let marker = format!("{field}:");
            (trimmed.starts_with("pub") && trimmed.contains(&marker)).then_some(trimmed.to_owned())
        })
    })
    .collect::<Vec<_>>();
    let public_api = public_api_fragments(impl_body(&source, "ParsedKernelEvidence")).join("\n");
    let escape_violations = [
        "&mut",
        "_mut",
        "into_parts",
        "from_parts",
        "Vec<",
        "Box<[",
        "self) -> (",
    ]
    .into_iter()
    .filter(|token| public_api.contains(token))
    .collect::<Vec<_>>();

    assert!(
        field_violations.is_empty() && escape_violations.is_empty(),
        "ParsedKernelEvidence must preserve parser-validated invariants with \
         read-only accessors, not public mutable fields or reconstructable public \
         escape hatches: fields={field_violations:?}, public_api={escape_violations:?}"
    );
}

#[test]
fn public_surface_scanner_catches_common_shapes() {
    let public_samples = [
        "pub mod clause;",
        "pub use clause::Clause;",
        "pub struct Clause;",
        "pub enum RejectionReason { Invalid }",
        "pub fn check() {}",
        "pub const VERSION: u32 = 1;",
        "pub static LIMIT: u32 = 1;",
        "pub type ClauseId = u32;",
        "pub trait Checker {}",
        "pub unsafe fn check() {}",
        "pub async fn check() {}",
        "pub extern crate mizar_core;",
        "pub macro sample() {}",
        "pub union RawClause { value: u32 }",
        "#[macro_export] macro_rules! kernel_macro { () => {}; }",
        "pub\nstruct Clause;",
        "pub(crate)\nfn helper() {}",
        "pub\nunsafe fn check() {}",
        "pub\nasync fn check() {}",
        "pub\nextern crate mizar_core;",
        "pub\nextern \"C\" fn check() {}",
        "pub\nmacro sample() {}",
        "pub\nunion RawClause { value: u32 }",
        "pub(crate)\nunsafe fn helper() {}",
        "pub(super)\nasync fn helper() {}",
        "pub(crate)\nextern \"C\" fn helper() {}",
        "#[macro_export]\nmacro_rules! kernel_macro { () => {}; }",
    ];

    for sample in public_samples {
        assert!(!public_semantic_declarations(sample).is_empty(), "{sample}");
    }

    for sample in [
        "#![forbid(unsafe_code)]",
        "//! task 1 intentionally exposes no public semantic surface",
        "const PRIVATE: u32 = 1;",
    ] {
        assert!(public_semantic_declarations(sample).is_empty(), "{sample}");
    }
}

#[test]
fn source_spec_public_inventory_scanner_is_exact() {
    let source = "\
pub struct DirectStruct;\n\
pub enum DirectEnum { Value }\n\
pub fn direct_fn() {}\n\
pub const DIRECT_CONST: u8 = 1;\n\
pub const fn direct_const_fn() -> u8 { 1 }\n\
pub type DirectType = u8;\n\
pub mod direct_mod {}\n\
pub unsafe fn direct_unsafe_fn() {}\n\
pub async fn direct_async_fn() {}\n\
pub extern \"C\" fn direct_extern_fn() {}\n\
pub\n\
struct SplitStruct;\n\
pub(crate) struct PrivateStruct;\n\
impl DirectStruct {\n\
    pub fn method_is_not_top_level() {}\n\
}\n";

    assert_eq!(
        top_level_public_items(source),
        [
            "DIRECT_CONST",
            "DirectEnum",
            "DirectStruct",
            "DirectType",
            "SplitStruct",
            "direct_async_fn",
            "direct_const_fn",
            "direct_extern_fn",
            "direct_fn",
            "direct_mod",
            "direct_unsafe_fn",
        ]
    );
    assert!(unsupported_top_level_public_forms(source).is_empty());

    let unsupported = "\
pub use crate::clause::Clause;\n\
pub macro sample() {}\n\
pub extern crate mizar_core;\n\
#[macro_export]\n\
macro_rules! kernel_macro { () => {}; }\n\
impl DirectStruct {\n\
    pub use is_not_valid_but_not_top_level;\n\
}\n";
    assert_eq!(
        unsupported_top_level_public_forms(unsupported),
        [
            "1: pub use crate::clause::Clause;",
            "2: pub macro sample() {}",
            "3: pub extern crate mizar_core;",
            "4: #[macro_export]",
        ]
    );
}

#[test]
fn dependency_scanner_catches_dependency_subtables() {
    let samples = [
        "[dependencies]",
        "[dependencies.mizar-atp]",
        "[dev-dependencies]",
        "[dev-dependencies.fixture]",
        "[build-dependencies]",
        "[build-dependencies.codegen]",
        "[target.'cfg(unix)'.dependencies]",
        "[target.'cfg(unix)'.dependencies.mizar-atp]",
        "[target.'cfg(unix)'.dev-dependencies.fixture]",
        "[target.'cfg(unix)'.build-dependencies.codegen]",
    ];

    for sample in samples {
        let section = section_name(sample).expect(sample);
        assert!(dependency_section(section), "{sample}");
    }

    for sample in ["[package]", "[lib]", "[lints]", "[workspace]"] {
        let section = section_name(sample).expect(sample);
        assert!(!dependency_section(section), "{sample}");
    }
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

fn workspace_members(manifest: &str) -> Vec<String> {
    let Some(workspace) = table_body(manifest, "workspace") else {
        return Vec::new();
    };
    let Some(members) = array_value(workspace, "members") else {
        return Vec::new();
    };

    quoted_strings(members)
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
        if let Some((_, lines)) = active.as_mut()
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
    line.strip_prefix('[')
        .and_then(|line| line.strip_suffix(']'))
        .map(str::trim)
}

fn dependency_section(section: &str) -> bool {
    const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

    section
        .split('.')
        .any(|segment| DEPENDENCY_TABLES.contains(&segment))
}

fn assert_lock_package(
    lockfile: &str,
    name: &str,
    version: &str,
    checksum: &str,
    dependencies: &[&str],
) {
    let package = lock_package(lockfile, name)
        .unwrap_or_else(|| panic!("Cargo.lock must contain package {name}"));
    assert!(
        package.contains(&format!("version = \"{version}\"")),
        "Cargo.lock package {name} must pin version {version}:\n{package}"
    );
    assert!(
        package.contains(&format!("checksum = \"{checksum}\"")),
        "Cargo.lock package {name} must pin audited checksum {checksum}:\n{package}"
    );
    for dependency in dependencies {
        assert!(
            package.contains(&format!("\"{dependency}\"")),
            "Cargo.lock package {name} must depend on {dependency}:\n{package}"
        );
    }
}

fn lock_package<'a>(lockfile: &'a str, name: &str) -> Option<&'a str> {
    lockfile.split("\n[[package]]").find(|package| {
        package
            .lines()
            .any(|line| line.trim() == format!("name = \"{name}\""))
    })
}

fn assignment_is(line: &str, key: &str, expected: &str) -> bool {
    line.split_once('=')
        .map(|(actual_key, actual_value)| {
            actual_key.trim() == key && actual_value.trim().trim_matches('"') == expected
        })
        .unwrap_or(false)
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

fn source_public_enum_inventory() -> Vec<String> {
    let src_root = crate_root().join("src");
    let mut inventory = Vec::new();
    for path in rust_source_files(&src_root) {
        let module = path
            .file_stem()
            .and_then(|name| name.to_str())
            .expect("source file has utf-8 stem");
        let source = read_to_string(&path);
        for line in source.lines() {
            if let Some(enum_name) = public_enum_name(line) {
                inventory.push(format!("{module}::{enum_name}"));
            }
        }
    }
    inventory.sort();
    inventory
}

fn documented_public_enum_inventory(path: &Path) -> Vec<String> {
    const START: &str = "<!-- public-enum-inventory:start -->";
    const END: &str = "<!-- public-enum-inventory:end -->";

    let document = read_to_string(path);
    let start = document
        .find(START)
        .unwrap_or_else(|| panic!("{} missing {START}", path.display()))
        + START.len();
    let rest = &document[start..];
    let end = rest
        .find(END)
        .unwrap_or_else(|| panic!("{} missing {END}", path.display()));
    rest[..end]
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "```text" && *line != "```")
        .map(str::to_owned)
        .collect()
}

fn public_enums_without_non_exhaustive() -> Vec<String> {
    let src_root = crate_root().join("src");
    let mut missing = Vec::new();
    for path in rust_source_files(&src_root) {
        let module = path
            .file_stem()
            .and_then(|name| name.to_str())
            .expect("source file has utf-8 stem");
        let source = read_to_string(&path);
        let lines = source.lines().collect::<Vec<_>>();
        for (index, line) in lines.iter().enumerate() {
            let Some(enum_name) = public_enum_name(line) else {
                continue;
            };
            if index == 0 || lines[index - 1].trim() != "#[non_exhaustive]" {
                let display_path = path.strip_prefix(crate_root()).unwrap_or(&path);
                missing.push(format!(
                    "{}:{} {module}::{enum_name}",
                    display_path.display(),
                    index + 1
                ));
            }
        }
    }
    missing.sort();
    missing
}

fn public_enum_name(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix("pub enum ")?;
    rest.split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .next()
        .filter(|name| !name.is_empty())
}

fn public_module_exports(source: &str) -> Vec<String> {
    let mut modules = source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod "))
        .filter_map(|rest| rest.strip_suffix(';'))
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    modules.sort();
    modules
}

fn documented_audit_modules(audit: &str) -> Vec<String> {
    let mut modules = audit
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("### `")?;
            rest.strip_suffix('`')
        })
        .map(str::to_owned)
        .collect::<Vec<_>>();
    modules.sort();
    modules
}

fn documented_public_items(module_section: &str) -> Vec<String> {
    let marker = "Covered top-level public items:";
    let start = module_section
        .find(marker)
        .unwrap_or_else(|| panic!("missing `{marker}` in audit module section"))
        + marker.len();
    let mut items = Vec::new();
    let mut in_list = false;
    for line in module_section[start..].lines() {
        let trimmed = line.trim();
        if let Some(item) = trimmed
            .strip_prefix("- `")
            .and_then(|item| item.strip_suffix('`'))
        {
            in_list = true;
            items.push(item.to_owned());
            continue;
        }
        if in_list && trimmed.is_empty() {
            break;
        }
    }
    items.sort();
    items
}

fn documented_test_traceability_paths(audit: &str) -> Vec<String> {
    let section = markdown_section(audit, "## Test Traceability")
        .expect("source/spec audit missing ## Test Traceability");
    section
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with('|')
                || trimmed.starts_with("|---")
                || trimmed.contains("Module / boundary")
            {
                return None;
            }
            let columns = trimmed.split('|').map(str::trim).collect::<Vec<_>>();
            let path = columns.get(2)?;
            path.strip_prefix('`')
                .and_then(|path| path.strip_suffix('`'))
                .map(str::to_owned)
        })
        .collect()
}

fn top_level_public_items(source: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut brace_depth = 0usize;
    let mut pending_public = false;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if brace_depth == 0 {
            if pending_public {
                if !trimmed.is_empty() && !trimmed.starts_with("#[") {
                    let normalized = format!("pub {trimmed}");
                    if let Some(item) = public_item_name(&normalized) {
                        items.push(item.to_owned());
                    }
                    pending_public = false;
                }
            } else if trimmed == "pub" {
                pending_public = true;
            } else if externally_public_line(trimmed)
                && let Some(item) = public_item_name(trimmed)
            {
                items.push(item.to_owned());
            }
        }

        let open = line.chars().filter(|character| *character == '{').count();
        let close = line.chars().filter(|character| *character == '}').count();
        brace_depth = brace_depth.saturating_add(open).saturating_sub(close);
        if line.trim_end().ends_with(';') && open == 0 && close == 0 {
            continue;
        }
    }
    items.sort();
    items
}

fn unsupported_top_level_public_forms(source: &str) -> Vec<String> {
    let mut unsupported = Vec::new();
    let mut brace_depth = 0usize;
    let mut pending_public = None;

    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if brace_depth == 0 {
            if let Some(public_line) = pending_public {
                if !trimmed.is_empty() && !trimmed.starts_with("#[") {
                    let normalized = format!("pub {trimmed}");
                    if unsupported_public_form(&normalized) {
                        unsupported.push(format!("{}: pub {}", public_line, trimmed));
                    }
                    pending_public = None;
                }
            } else if trimmed == "pub" {
                pending_public = Some(index + 1);
            } else if trimmed.starts_with("#[macro_export]") || unsupported_public_form(trimmed) {
                unsupported.push(format!("{}: {trimmed}", index + 1));
            }
        }

        let open = line.chars().filter(|character| *character == '{').count();
        let close = line.chars().filter(|character| *character == '}').count();
        brace_depth = brace_depth.saturating_add(open).saturating_sub(close);
    }

    unsupported
}

fn public_item_name(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("pub ")?;
    let rest = rest
        .strip_prefix("unsafe ")
        .or_else(|| rest.strip_prefix("async "))
        .unwrap_or(rest);
    let rest = rest.strip_prefix("extern \"C\" ").unwrap_or(rest);

    if let Some(rest) = rest.strip_prefix("const fn ") {
        return rest
            .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
            .next()
            .filter(|name| !name.is_empty());
    }

    for prefix in [
        "struct ", "enum ", "fn ", "const ", "static ", "type ", "trait ", "union ", "mod ",
    ] {
        if let Some(name) = rest.strip_prefix(prefix) {
            return name
                .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
                .next()
                .filter(|name| !name.is_empty());
        }
    }

    None
}

fn externally_public_line(line: &str) -> bool {
    line.starts_with("pub ")
        && !line.starts_with("pub(crate)")
        && !line.starts_with("pub(super)")
        && !line.starts_with("pub(in ")
}

fn unsupported_public_form(line: &str) -> bool {
    line.starts_with("pub use ")
        || line.starts_with("pub macro ")
        || line.starts_with("pub extern crate ")
}

fn public_api_fragments(source: &str) -> Vec<String> {
    let mut fragments = Vec::new();
    let lines = source.lines().collect::<Vec<_>>();
    let mut index = 0usize;

    while index < lines.len() {
        let trimmed = lines[index].trim_start();
        if externally_public_signature_line(trimmed) {
            let (fragment, next_index) = collect_public_signature(&lines, index);
            fragments.push(fragment);
            index = next_index;
        } else if externally_public_block_line(trimmed) {
            let (fragment, next_index) = collect_public_block(&lines, index);
            fragments.push(fragment);
            index = next_index;
        } else {
            index += 1;
        }
    }

    fragments
}

fn externally_public_signature_line(line: &str) -> bool {
    externally_public_line(line)
        && (line.starts_with("pub fn ")
            || line.starts_with("pub const fn ")
            || line.starts_with("pub async fn ")
            || line.starts_with("pub unsafe fn ")
            || line.starts_with("pub type ")
            || line.starts_with("pub const ")
            || line.starts_with("pub static "))
}

fn externally_public_block_line(line: &str) -> bool {
    externally_public_line(line)
        && (line.starts_with("pub struct ")
            || line.starts_with("pub enum ")
            || line.starts_with("pub trait ")
            || line.starts_with("pub union "))
}

fn collect_public_signature(lines: &[&str], start: usize) -> (String, usize) {
    let mut fragment = String::new();
    let mut index = start;

    while index < lines.len() {
        let line = lines[index];
        if let Some((signature, _)) = line.split_once('{') {
            fragment.push_str(signature);
            fragment.push('\n');
            return (fragment, index + 1);
        }

        fragment.push_str(line);
        fragment.push('\n');
        if line.contains(';') {
            return (fragment, index + 1);
        }
        index += 1;
    }

    (fragment, index)
}

fn collect_public_block(lines: &[&str], start: usize) -> (String, usize) {
    let mut fragment = String::new();
    let mut brace_depth = 0usize;
    let mut saw_brace = false;
    let mut index = start;

    while index < lines.len() {
        let line = lines[index];
        fragment.push_str(line);
        fragment.push('\n');

        let open = line.chars().filter(|character| *character == '{').count();
        let close = line.chars().filter(|character| *character == '}').count();
        saw_brace |= open > 0;
        brace_depth = brace_depth.saturating_add(open).saturating_sub(close);

        index += 1;
        if (saw_brace && brace_depth == 0) || (!saw_brace && line.contains(';')) {
            break;
        }
    }

    (fragment, index)
}

fn imported_batsat_symbols(source: &str) -> Vec<String> {
    let uncommented = source
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .collect::<Vec<_>>()
        .join("\n");
    let mut symbols = Vec::new();
    let mut rest = uncommented.as_str();

    while let Some(start) = rest.find("use batsat::") {
        let import = &rest[start + "use batsat::".len()..];
        let Some(end) = import.find(';') else {
            break;
        };
        let statement = import[..end].trim();
        if let Some(group) = statement
            .strip_prefix('{')
            .and_then(|statement| statement.strip_suffix('}'))
        {
            for item in group.split(',') {
                if let Some(symbol) = imported_symbol_name(item) {
                    symbols.push(symbol);
                }
            }
        } else if let Some(symbol) = imported_symbol_name(statement) {
            symbols.push(symbol);
        }
        rest = &import[end + 1..];
    }

    symbols.sort();
    symbols.dedup();
    symbols
}

fn imported_symbol_name(item: &str) -> Option<String> {
    let item = item.trim();
    if item.is_empty() {
        return None;
    }
    let name = item
        .rsplit_once(" as ")
        .map(|(_, alias)| alias)
        .unwrap_or_else(|| item.rsplit("::").next().unwrap_or(item))
        .trim();
    (!name.is_empty()).then(|| name.to_owned())
}

fn contains_rust_identifier(source: &str, identifier: &str) -> bool {
    source
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .any(|token| token == identifier)
}

fn struct_body<'a>(source: &'a str, name: &str) -> &'a str {
    let marker = format!("pub struct {name} {{");
    braced_body_after_marker(source, &marker)
        .unwrap_or_else(|| panic!("missing or unterminated public struct {name}"))
}

fn impl_body<'a>(source: &'a str, name: &str) -> &'a str {
    let marker = format!("impl {name} {{");
    braced_body_after_marker(source, &marker)
        .unwrap_or_else(|| panic!("missing or unterminated impl {name}"))
}

fn braced_body_after_marker<'a>(source: &'a str, marker: &str) -> Option<&'a str> {
    let start = source.find(marker)?;
    let body_start = start + marker.len();
    let rest = &source[body_start..];
    let mut depth = 1usize;

    for (index, character) in rest.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&rest[..index]);
                }
            }
            _ => {}
        }
    }

    None
}

fn markdown_section<'a>(document: &'a str, heading: &str) -> Option<&'a str> {
    let start = document.find(heading)?;
    let start_level = heading
        .chars()
        .take_while(|character| *character == '#')
        .count();
    let body_start = start + heading.len();
    let rest = &document[body_start..];
    let end = rest
        .match_indices('\n')
        .filter_map(|(index, _)| {
            let line = rest[index + 1..].lines().next().unwrap_or_default();
            let level = line
                .chars()
                .take_while(|character| *character == '#')
                .count();
            (level > 0 && level <= start_level).then_some(index)
        })
        .next()
        .unwrap_or(rest.len());

    Some(&document[start..body_start + end])
}

fn collect_rust_source_files(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path.to_path_buf());
        }
        return;
    }

    for entry in fs::read_dir(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())) {
        let entry = entry.unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        collect_rust_source_files(&entry.path(), files);
    }
}

fn git_tracks(relative_path: &str) -> bool {
    Command::new("git")
        .arg("ls-files")
        .arg("--error-unmatch")
        .arg(relative_path)
        .current_dir(workspace_root())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap_or_else(|error| panic!("git ls-files failed for {relative_path}: {error}"))
        .success()
}

fn public_semantic_declarations(source: &str) -> Vec<String> {
    let mut declarations = source
        .lines()
        .enumerate()
        .filter(|(_, line)| public_semantic_declaration(line))
        .map(|(index, line)| format!("{}: {}", index + 1, line.trim()))
        .collect::<Vec<_>>();
    let compact_source = compact_non_comment_source(source);
    for marker in [
        "#[macro_export]",
        "pubmod",
        "pubuse",
        "pubstruct",
        "pubenum",
        "pubfn",
        "pubconst",
        "pubstatic",
        "pubtype",
        "pubtrait",
        "pubunsafe",
        "pubasync",
        "pubextern",
        "pubexterncrate",
        "pubmacro",
        "pubunion",
        "pub(crate)mod",
        "pub(crate)use",
        "pub(crate)struct",
        "pub(crate)enum",
        "pub(crate)fn",
        "pub(crate)const",
        "pub(crate)static",
        "pub(crate)type",
        "pub(crate)trait",
        "pub(crate)unsafe",
        "pub(crate)async",
        "pub(crate)extern",
        "pub(crate)externcrate",
        "pub(crate)macro",
        "pub(crate)union",
        "pub(super)mod",
        "pub(super)use",
        "pub(super)struct",
        "pub(super)enum",
        "pub(super)fn",
        "pub(super)const",
        "pub(super)static",
        "pub(super)type",
        "pub(super)trait",
        "pub(super)unsafe",
        "pub(super)async",
        "pub(super)extern",
        "pub(super)externcrate",
        "pub(super)macro",
        "pub(super)union",
        "pub(in",
    ] {
        if compact_source.contains(marker) {
            declarations.push(format!("compact: {marker}"));
        }
    }
    declarations.sort();
    declarations.dedup();
    declarations
}

fn public_semantic_declaration(line: &str) -> bool {
    let trimmed = line.trim();

    trimmed.starts_with("pub ")
        || trimmed.starts_with("pub(")
        || trimmed.starts_with("pub(crate)")
        || trimmed.starts_with("pub(super)")
        || trimmed.starts_with("pub(in ")
        || trimmed.starts_with("#[macro_export]")
}

fn table_body<'a>(document: &'a str, name: &str) -> Option<&'a str> {
    let header = format!("[{name}]");
    let start = document.find(&header)? + header.len();
    let rest = &document[start..];
    let end = rest
        .find("\n[")
        .map(|index| start + index)
        .unwrap_or(document.len());
    Some(&document[start..end])
}

fn array_value<'a>(table: &'a str, key: &str) -> Option<&'a str> {
    let key_offset = table.find(key)?;
    let after_key = &table[key_offset + key.len()..];
    let equals_offset = after_key.find('=')?;
    let after_equals = &after_key[equals_offset + 1..];
    let start_offset = after_equals.find('[')?;
    let after_start = &after_equals[start_offset + 1..];
    let end_offset = after_start.find(']')?;
    Some(&after_start[..end_offset])
}

fn quoted_strings(value: &str) -> Vec<String> {
    let mut strings = Vec::new();
    let mut rest = value;

    while let Some(start) = rest.find('"') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('"') else {
            break;
        };
        strings.push(after_start[..end].to_owned());
        rest = &after_start[end + 1..];
    }

    strings
}

fn compact_non_comment_source(source: &str) -> String {
    source
        .lines()
        .filter_map(|line| {
            line.split_once("//").map_or(Some(line), |(code, _)| {
                (!code.trim().is_empty()).then_some(code)
            })
        })
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}
