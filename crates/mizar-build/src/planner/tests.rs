use super::{
    BuildConfig, DependencyKind, DependencySelection, LockSource, ManifestDependency,
    ManifestDiagnosticKind, ManifestValidationError, PackagePlanSource, PlanRequest, Solver,
    VerifierConfig, VersionComparison, VersionConstraint, WorkspacePackage,
    is_lowercase_snake_case_package_id, parse_lockfile, parse_package_manifest,
    parse_workspace_manifest, produce_build_plan, validate_lockfile_for_workspace,
    validate_package_id_spelling, validate_package_manifest,
};
use mizar_session::{ToolchainInfo, WorkspaceRoot};
use semver::Version;

#[test]
fn package_ids_accept_lowercase_snake_case() {
    for package_id in ["a", "mml", "mathcomp_mizar", "pkg1", "pkg_1_core2"] {
        assert!(
            is_lowercase_snake_case_package_id(package_id),
            "{package_id:?}"
        );
        validate_package_id_spelling(package_id).expect("valid package id");
    }
}

#[test]
fn package_ids_reject_hyphenated_or_normalized_spellings() {
    for package_id in [
        "mathcomp-mizar",
        "MathComp",
        "mathcomp__mizar",
        "mathcomp_",
        "_mathcomp",
        "mathcomp mizar",
        "mathcomp.mizar",
        "1mathcomp",
        "",
    ] {
        let error = validate_package_id_spelling(package_id).unwrap_err();
        assert!(
            matches!(
                error,
                ManifestValidationError::InvalidPackageId {
                    package_id: ref rejected,
                    expected: "[a-z][a-z0-9]*(?:_[a-z0-9]+)*",
                } if rejected == package_id
            ),
            "{package_id:?}: {error:?}"
        );
    }
}

#[test]
fn package_manifest_validation_preserves_spelling_without_hyphen_normalization() {
    let manifest = parse_package_manifest(
        r#"
        [package]
        name = "mathcomp_mizar"
        version = "1.2.3"
        "#,
    )
    .expect("valid manifest");
    let validated = validate_package_manifest(&manifest).expect("valid manifest");
    assert_eq!(validated.package_id.as_str(), "mathcomp_mizar");

    let error = parse_package_manifest(
        r#"
        [package]
        name = "mathcomp-mizar"
        version = "1.2.3"
        "#,
    )
    .unwrap_err();
    assert!(error.diagnostics().iter().any(|diagnostic| {
        matches!(
            diagnostic.kind,
            ManifestDiagnosticKind::InvalidPackageId {
                expected: "[a-z][a-z0-9]*(?:_[a-z0-9]+)*",
            }
        ) && diagnostic.value.as_deref() == Some("mathcomp-mizar")
    }));
}

#[test]
fn parses_valid_package_manifest_with_defaults_and_dependency_forms() {
    let manifest = parse_package_manifest(
        r#"
        [package]
        name = "algebra"
        version = "2.3.1"

        [dependencies]
        mml_core = "^1.0.0"
        topology = { version = "~0.9.0", features = ["metric", "compact"] }

        [dev-dependencies]
        test_utils = "0.2.0"
        "#,
    )
    .expect("valid package manifest");

    assert_eq!(manifest.name, "algebra");
    assert_eq!(manifest.version, Version::new(2, 3, 1));
    assert_eq!(manifest.edition.as_str(), "2025");
    assert_eq!(manifest.verifier, VerifierConfig::default());
    assert_eq!(manifest.build, BuildConfig::default());
    assert_eq!(
        manifest.dependencies,
        vec![
            ManifestDependency {
                package_id: "mml_core".to_owned(),
                version: VersionConstraint::Caret(Version::new(1, 0, 0)),
                kind: DependencyKind::Normal,
                features: Vec::new(),
            },
            ManifestDependency {
                package_id: "topology".to_owned(),
                version: VersionConstraint::Tilde(Version::new(0, 9, 0)),
                kind: DependencyKind::Normal,
                features: vec!["compact".to_owned(), "metric".to_owned()],
            },
        ]
    );
    assert_eq!(manifest.dev_dependencies[0].kind, DependencyKind::Dev);
}

#[test]
fn parses_short_caret_and_tilde_dependency_constraints() {
    let manifest = parse_package_manifest(
        r#"
        [package]
        name = "short_versions"
        version = "1.0.0"

        [dependencies]
        algebra = "^1.0"
        topology = "~0.9"
        "#,
    )
    .expect("short caret and tilde constraints are valid");

    assert_eq!(
        manifest.dependencies[0].version,
        VersionConstraint::Caret(Version::new(1, 0, 0))
    );
    assert_eq!(
        manifest.dependencies[1].version,
        VersionConstraint::Tilde(Version::new(0, 9, 0))
    );
}

#[test]
fn duplicate_manifest_dependencies_across_normal_and_dev_are_rejected() {
    let diagnostics = parse_package_manifest(
        r#"
        [package]
        name = "algebra"
        version = "1.0.0"

        [dependencies]
        test_utils = "1.0.0"

        [dev-dependencies]
        test_utils = "1.0.0"
        "#,
    )
    .unwrap_err();

    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(diagnostic.kind, ManifestDiagnosticKind::DuplicateDependency)
            && diagnostic.location.key == "dev-dependencies.test_utils"
    }));
}

#[test]
fn parses_verifier_and_build_config() {
    let manifest = parse_package_manifest(
        r#"
        [package]
        name = "algebra"
        version = "2.3.1"
        edition = "2025"

        [verifier]
        max_axioms = 64
        atp_timeout = 10
        default_solver = "z3"
        require_kernel_certificates = false

        [build]
        incremental = false
        cache_dir = "cache"
        artifact_dir = "out"
        "#,
    )
    .expect("valid manifest");

    assert_eq!(manifest.verifier.max_axioms, 64);
    assert_eq!(manifest.verifier.atp_timeout, 10);
    assert_eq!(manifest.verifier.default_solver, Solver::Z3);
    assert!(!manifest.verifier.require_kernel_certificates);
    assert!(!manifest.build.incremental);
    assert_eq!(manifest.build.cache_dir, "cache");
    assert_eq!(manifest.build.artifact_dir, "out");
}

#[test]
fn invalid_manifest_reports_deterministic_error_order() {
    let diagnostics = parse_package_manifest(
        r#"
        unexpected = true

        [build]
        artifact_dir = "../out"
        cache_dir = "/tmp/cache"

        [package]
        version = "not-semver"
        name = "bad-name"

        [verifier]
        default_solver = "missing"
        "#,
    )
    .unwrap_err()
    .into_diagnostics();

    let keys = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.location.key.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        keys,
        vec![
            "build.artifact_dir",
            "build.cache_dir",
            "package.name",
            "package.version",
            "unexpected",
            "verifier.default_solver",
        ]
    );
}

#[test]
fn workspace_manifest_validates_members_and_root_member() {
    let manifest = parse_workspace_manifest(
        r#"
        [workspace]
        members = ["topology", ".", "algebra"]
        "#,
    )
    .expect("valid workspace manifest");
    assert_eq!(manifest.members, vec![".", "algebra", "topology"]);

    let diagnostics = parse_workspace_manifest(
        r#"
        [workspace]
        members = ["algebra", "algebra", "../outside", ""]
        "#,
    )
    .unwrap_err()
    .into_diagnostics();
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic.kind,
        ManifestDiagnosticKind::DuplicateWorkspaceMember
    )));
    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| matches!(
                diagnostic.kind,
                ManifestDiagnosticKind::InvalidWorkspaceMemberPath
            ))
            .count(),
        2
    );
}

#[test]
fn lockfile_parses_sources_and_validates_internal_dependency_targets() {
    let lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.2.3"
        source = { kind = "workspace", path = "algebra" }
        dependencies = [{ name = "mml_core", version = "1.0.0" }]

        [[package]]
        name = "mml_core"
        version = "1.0.0"
        source = { kind = "registry", registry = "default", checksum = "sha256:abcd" }
        dependencies = []
        "#,
    )
    .expect("valid lockfile");

    assert_eq!(lockfile.schema_version, 1);
    assert_eq!(lockfile.packages[0].name, "algebra");
    assert_eq!(
        lockfile.packages[0].source,
        LockSource::Workspace {
            path: "algebra".to_owned()
        }
    );
    assert_eq!(lockfile.packages[1].name, "mml_core");

    let diagnostics = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.2.3"
        source = { kind = "workspace", path = "algebra" }
        dependencies = [{ name = "missing_dep", version = "1.0.0" }]
        "#,
    )
    .unwrap_err();
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| matches!(
        diagnostic.kind,
        ManifestDiagnosticKind::UnknownLockedDependency
    )));
}

#[test]
fn lockfile_workspace_validation_reports_missing_and_mismatched_versions() {
    let algebra = parse_package_manifest(
        r#"
        [package]
        name = "algebra"
        version = "1.2.3"
        "#,
    )
    .expect("valid manifest");
    let topology = parse_package_manifest(
        r#"
        [package]
        name = "topology"
        version = "0.1.0"
        "#,
    )
    .expect("valid manifest");
    let lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.2.4"
        source = { kind = "workspace", path = "algebra" }
        dependencies = []
        "#,
    )
    .expect("valid lockfile");

    let diagnostics = validate_lockfile_for_workspace(&[algebra, topology], &lockfile).unwrap_err();
    assert_eq!(
        diagnostics
            .diagnostics()
            .iter()
            .map(|diagnostic| &diagnostic.kind)
            .collect::<Vec<_>>(),
        vec![
            &ManifestDiagnosticKind::LockVersionMismatch,
            &ManifestDiagnosticKind::MissingLockPackage,
        ]
    );
}

#[test]
fn lockfile_workspace_validation_requires_workspace_source_kind() {
    let algebra = parse_package_manifest(
        r#"
        [package]
        name = "algebra"
        version = "1.2.3"
        "#,
    )
    .expect("valid manifest");
    let lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.2.3"
        source = { kind = "registry", registry = "default", checksum = "sha256:abcd" }
        dependencies = []
        "#,
    )
    .expect("valid lockfile");

    let diagnostics = validate_lockfile_for_workspace(&[algebra], &lockfile).unwrap_err();
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(diagnostic.kind, ManifestDiagnosticKind::InvalidLockSource)
            && diagnostic.location.key == "algebra.source.kind"
    }));
}

#[test]
fn lockfile_workspace_validation_rejects_nonmember_workspace_sources() {
    let algebra = parse_package_manifest(
        r#"
        [package]
        name = "algebra"
        version = "1.2.3"

        [dependencies]
        local_dep = "1.0.0"
        "#,
    )
    .expect("valid manifest");
    let lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.2.3"
        source = { kind = "workspace", path = "algebra" }
        dependencies = [{ name = "local_dep", version = "1.0.0" }]

        [[package]]
        name = "local_dep"
        version = "1.0.0"
        source = { kind = "workspace", path = "local_dep" }
        dependencies = []
        "#,
    )
    .expect("valid lockfile");

    let diagnostics = validate_lockfile_for_workspace(&[algebra], &lockfile).unwrap_err();
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(diagnostic.kind, ManifestDiagnosticKind::InvalidLockSource)
            && diagnostic.location.key == "local_dep.source.kind"
            && diagnostic.value.as_deref() == Some("workspace")
    }));
}

#[test]
fn lockfile_workspace_validation_checks_manifest_dependency_versions() {
    let algebra = parse_package_manifest(
        r#"
        [package]
        name = "algebra"
        version = "1.0.0"

        [dependencies]
        mml_core = "^1.0.0"
        "#,
    )
    .expect("valid manifest");
    let valid_lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = [{ name = "mml_core", version = "1.2.0" }]

        [[package]]
        name = "mml_core"
        version = "1.2.0"
        source = { kind = "registry", registry = "default", checksum = "sha256:abcd" }
        dependencies = []
        "#,
    )
    .expect("valid lockfile");
    validate_lockfile_for_workspace(std::slice::from_ref(&algebra), &valid_lockfile)
        .expect("dependency satisfies caret constraint");

    let invalid_lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = [{ name = "mml_core", version = "2.0.0" }]

        [[package]]
        name = "mml_core"
        version = "2.0.0"
        source = { kind = "registry", registry = "default", checksum = "sha256:abcd" }
        dependencies = []
        "#,
    )
    .expect("internally consistent lockfile");
    let diagnostics = validate_lockfile_for_workspace(&[algebra], &invalid_lockfile)
        .expect_err("manifest dependency constraint rejects lock version");
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(
            diagnostic.kind,
            ManifestDiagnosticKind::InvalidDependencyVersion
        ) && diagnostic.location.key == "algebra.dependencies.mml_core"
    }));
}

#[test]
fn lockfile_dependency_edges_require_exact_locked_versions() {
    let diagnostics = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = [{ name = "mml_core", version = "9.9.9" }]

        [[package]]
        name = "mml_core"
        version = "1.0.0"
        source = { kind = "registry", registry = "default", checksum = "sha256:abcd" }
        dependencies = []
        "#,
    )
    .unwrap_err();
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(diagnostic.kind, ManifestDiagnosticKind::LockVersionMismatch)
            && diagnostic.location.key == "algebra.dependencies.mml_core"
    }));
}

#[test]
fn lockfile_rejects_duplicate_dependency_names() {
    let diagnostics = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = [
            { name = "mml_core", version = "1.0.0" },
            { name = "mml_core", version = "1.0.0" },
        ]

        [[package]]
        name = "mml_core"
        version = "1.0.0"
        source = { kind = "registry", registry = "default", checksum = "sha256:abcd" }
        dependencies = []
        "#,
    )
    .unwrap_err();

    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(diagnostic.kind, ManifestDiagnosticKind::DuplicateDependency)
            && diagnostic.location.key == "package[0].dependencies.mml_core"
    }));
}

#[test]
fn required_field_and_toml_errors_are_reported() {
    assert!(parse_package_manifest("not =").is_err_and(|diagnostics| {
        diagnostics
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind, ManifestDiagnosticKind::InvalidToml))
    }));

    let package_diagnostics = parse_package_manifest(
        r#"
        [package]
        name = "algebra"
        "#,
    )
    .unwrap_err();
    assert!(
        package_diagnostics
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.location.key == "package.version"
                && matches!(diagnostic.kind, ManifestDiagnosticKind::MissingField))
    );

    let workspace_diagnostics = parse_workspace_manifest("[workspace]\n").unwrap_err();
    assert!(
        workspace_diagnostics
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.location.key == "workspace.members"
                && matches!(diagnostic.kind, ManifestDiagnosticKind::MissingField))
    );

    let lockfile_diagnostics = parse_lockfile("schema_version = 1\n").unwrap_err();
    assert!(
        lockfile_diagnostics
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.location.key == "package"
                && matches!(diagnostic.kind, ManifestDiagnosticKind::MissingField))
    );
}

#[test]
fn paths_reject_host_specific_or_noncanonical_spelling() {
    let workspace_diagnostics = parse_workspace_manifest(
        r#"
        [workspace]
        members = ["foo\\bar", "foo//bar"]
        "#,
    )
    .unwrap_err();
    assert_eq!(
        workspace_diagnostics
            .diagnostics()
            .iter()
            .filter(|diagnostic| matches!(
                diagnostic.kind,
                ManifestDiagnosticKind::InvalidWorkspaceMemberPath
            ))
            .count(),
        2
    );

    let build_diagnostics = parse_package_manifest(
        r#"
        [package]
        name = "algebra"
        version = "1.0.0"

        [build]
        cache_dir = "C:\\cache"
        "#,
    )
    .unwrap_err();
    assert!(
        build_diagnostics
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.location.key == "build.cache_dir"
                && matches!(diagnostic.kind, ManifestDiagnosticKind::InvalidBuildPath))
    );

    parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "root_pkg"
        version = "1.0.0"
        source = { kind = "workspace", path = "." }
        dependencies = []
        "#,
    )
    .expect("root workspace lock path is valid");
}

#[test]
fn lockfile_reports_missing_dependency_array() {
    let diagnostics = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        "#,
    )
    .unwrap_err();
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        diagnostic.location.key == "package[0].dependencies"
            && matches!(diagnostic.kind, ManifestDiagnosticKind::MissingField)
    }));
}

#[test]
fn duplicate_lock_package_diagnostics_are_order_independent() {
    let first = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "2.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = []

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = []
        "#,
    )
    .unwrap_err()
    .into_diagnostics();
    let second = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = []

        [[package]]
        name = "algebra"
        version = "2.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = []
        "#,
    )
    .unwrap_err()
    .into_diagnostics();

    let duplicate_values = |diagnostics: Vec<super::ManifestDiagnostic>| {
        diagnostics
            .into_iter()
            .filter(|diagnostic| {
                matches!(
                    diagnostic.kind,
                    ManifestDiagnosticKind::DuplicateLockPackage
                )
            })
            .map(|diagnostic| diagnostic.value)
            .collect::<Vec<_>>()
    };
    assert_eq!(duplicate_values(first), duplicate_values(second));
}

#[test]
fn build_plan_orders_dependencies_before_dependents_deterministically() {
    let algebra = workspace_package(
        "algebra",
        r#"
        [package]
        name = "algebra"
        version = "1.0.0"
        "#,
    );
    let topology = workspace_package(
        "topology",
        r#"
        [package]
        name = "topology"
        version = "1.0.0"

        [dependencies]
        algebra = "1.0.0"
        "#,
    );
    let logic = workspace_package(
        "logic",
        r#"
        [package]
        name = "logic"
        version = "1.0.0"
        "#,
    );
    let lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "topology"
        version = "1.0.0"
        source = { kind = "workspace", path = "topology" }
        dependencies = [{ name = "algebra", version = "1.0.0" }]

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = []

        [[package]]
        name = "logic"
        version = "1.0.0"
        source = { kind = "workspace", path = "logic" }
        dependencies = []
        "#,
    )
    .expect("valid lockfile");

    let plan = produce_build_plan(
        plan_request(DependencySelection::Normal),
        vec![topology, logic, algebra],
        lockfile,
    )
    .expect("valid plan");

    assert_eq!(
        plan.packages
            .iter()
            .map(|package| package.package_id.as_str())
            .collect::<Vec<_>>(),
        vec!["algebra", "logic", "topology"]
    );
    assert_eq!(plan.dependency_graph.edges.len(), 1);
    assert_eq!(plan.dependency_graph.edges[0].dependent, "topology");
    assert_eq!(plan.dependency_graph.edges[0].dependency, "algebra");
    assert!(matches!(
        plan.packages[0].source,
        PackagePlanSource::Workspace { .. }
    ));
}

#[test]
fn build_plan_is_equal_for_shuffled_equivalent_inputs() {
    let algebra = workspace_package(
        "algebra",
        r#"
        [build]
        artifact_dir = "build"

        [dependencies]
        topology = { features = ["compact", "metric"], version = "1.0.0" }

        [package]
        version = "1.0.0"
        name = "algebra"
        "#,
    );
    let topology = workspace_package(
        "topology",
        r#"
        [package]
        name = "topology"
        version = "1.0.0"
        "#,
    );
    let first_lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = [{ name = "topology", version = "1.0.0" }]

        [[package]]
        name = "topology"
        version = "1.0.0"
        source = { kind = "workspace", path = "topology" }
        dependencies = []
        "#,
    )
    .expect("valid lockfile");
    let second_lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        source = { path = "topology", kind = "workspace" }
        dependencies = []
        version = "1.0.0"
        name = "topology"

        [[package]]
        dependencies = [{ version = "1.0.0", name = "topology" }]
        source = { path = "algebra", kind = "workspace" }
        version = "1.0.0"
        name = "algebra"
        "#,
    )
    .expect("valid lockfile");

    let first = produce_build_plan(
        plan_request(DependencySelection::Normal),
        vec![algebra.clone(), topology.clone()],
        first_lockfile,
    )
    .expect("valid first plan");
    let second = produce_build_plan(
        plan_request(DependencySelection::Normal),
        vec![topology, algebra],
        second_lockfile,
    )
    .expect("valid second plan");

    assert_eq!(first, second);
}

#[test]
fn build_plan_rejects_duplicate_workspace_package_ids() {
    let algebra = workspace_package(
        "algebra",
        r#"
        [package]
        name = "algebra"
        version = "1.0.0"
        "#,
    );
    let duplicate_algebra = workspace_package(
        "algebra_copy",
        r#"
        [package]
        name = "algebra"
        version = "1.0.0"
        "#,
    );
    let lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = []
        "#,
    )
    .expect("valid lockfile");

    let diagnostics = produce_build_plan(
        plan_request(DependencySelection::Normal),
        vec![algebra, duplicate_algebra],
        lockfile,
    )
    .unwrap_err();
    assert_eq!(
        diagnostics
            .diagnostics()
            .iter()
            .filter(|diagnostic| {
                matches!(diagnostic.kind, ManifestDiagnosticKind::DuplicatePackageId)
                    && diagnostic.location.key == "algebra"
            })
            .count(),
        1
    );
}

#[test]
fn build_plan_revalidates_workspace_package_member_paths() {
    let algebra = workspace_package(
        "../algebra",
        r#"
        [package]
        name = "algebra"
        version = "1.0.0"
        "#,
    );
    let lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = []
        "#,
    )
    .expect("valid lockfile");

    let diagnostics = produce_build_plan(
        plan_request(DependencySelection::Normal),
        vec![algebra],
        lockfile,
    )
    .unwrap_err();
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(
            diagnostic.kind,
            ManifestDiagnosticKind::InvalidWorkspaceMemberPath
        ) && diagnostic.location.key == "algebra.member_path"
    }));
}

#[test]
fn build_plan_rejects_dependency_cycles() {
    let algebra = workspace_package(
        "algebra",
        r#"
        [package]
        name = "algebra"
        version = "1.0.0"

        [dependencies]
        topology = "1.0.0"
        "#,
    );
    let topology = workspace_package(
        "topology",
        r#"
        [package]
        name = "topology"
        version = "1.0.0"

        [dependencies]
        algebra = "1.0.0"
        "#,
    );
    let lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = [{ name = "topology", version = "1.0.0" }]

        [[package]]
        name = "topology"
        version = "1.0.0"
        source = { kind = "workspace", path = "topology" }
        dependencies = [{ name = "algebra", version = "1.0.0" }]
        "#,
    )
    .expect("valid lockfile");

    let diagnostics = produce_build_plan(
        plan_request(DependencySelection::Normal),
        vec![algebra, topology],
        lockfile,
    )
    .unwrap_err();
    assert!(
        diagnostics
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind, ManifestDiagnosticKind::DependencyCycle))
    );
}

#[test]
fn build_plan_rejects_dev_only_cycles_when_dev_dependencies_are_active() {
    let algebra = workspace_package(
        "algebra",
        r#"
        [package]
        name = "algebra"
        version = "1.0.0"

        [dev-dependencies]
        test_utils = "1.0.0"
        "#,
    );
    let test_utils = workspace_package(
        "test_utils",
        r#"
        [package]
        name = "test_utils"
        version = "1.0.0"

        [dependencies]
        algebra = "1.0.0"
        "#,
    );
    let lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = [{ name = "test_utils", version = "1.0.0" }]

        [[package]]
        name = "test_utils"
        version = "1.0.0"
        source = { kind = "workspace", path = "test_utils" }
        dependencies = [{ name = "algebra", version = "1.0.0" }]
        "#,
    )
    .expect("valid lockfile");

    produce_build_plan(
        plan_request(DependencySelection::Normal),
        vec![algebra.clone(), test_utils.clone()],
        lockfile.clone(),
    )
    .expect("normal plan ignores the dev edge that closes the cycle");

    let diagnostics = produce_build_plan(
        plan_request(DependencySelection::NormalAndDev),
        vec![algebra, test_utils],
        lockfile,
    )
    .unwrap_err();
    assert!(
        diagnostics
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind, ManifestDiagnosticKind::DependencyCycle))
    );
}

#[test]
fn build_plan_rejects_dev_only_registry_cycles_only_when_dev_dependencies_are_active() {
    let algebra = workspace_package(
        "algebra",
        r#"
        [package]
        name = "algebra"
        version = "1.0.0"

        [dev-dependencies]
        registry_a = "1.0.0"
        "#,
    );
    let lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = [{ name = "registry_a", version = "1.0.0" }]

        [[package]]
        name = "registry_a"
        version = "1.0.0"
        source = { kind = "registry", registry = "default", checksum = "sha256:a" }
        dependencies = [{ name = "registry_b", version = "1.0.0" }]

        [[package]]
        name = "registry_b"
        version = "1.0.0"
        source = { kind = "registry", registry = "default", checksum = "sha256:b" }
        dependencies = [{ name = "registry_a", version = "1.0.0" }]
        "#,
    )
    .expect("valid lockfile");

    let normal = produce_build_plan(
        plan_request(DependencySelection::Normal),
        vec![algebra.clone()],
        lockfile.clone(),
    )
    .expect("normal plan ignores the dev-only registry cycle");
    assert_eq!(
        normal
            .packages
            .iter()
            .map(|package| package.package_id.as_str())
            .collect::<Vec<_>>(),
        vec!["algebra"]
    );

    let diagnostics = produce_build_plan(
        plan_request(DependencySelection::NormalAndDev),
        vec![algebra],
        lockfile,
    )
    .unwrap_err();
    assert!(
        diagnostics
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind, ManifestDiagnosticKind::DependencyCycle))
    );
}

#[test]
fn build_plan_rejects_reachable_registry_cycles() {
    let algebra = workspace_package(
        "algebra",
        r#"
        [package]
        name = "algebra"
        version = "1.0.0"

        [dependencies]
        registry_a = "1.0.0"
        "#,
    );
    let lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = [{ name = "registry_a", version = "1.0.0" }]

        [[package]]
        name = "registry_a"
        version = "1.0.0"
        source = { kind = "registry", registry = "default", checksum = "sha256:a" }
        dependencies = [{ name = "registry_b", version = "1.0.0" }]

        [[package]]
        name = "registry_b"
        version = "1.0.0"
        source = { kind = "registry", registry = "default", checksum = "sha256:b" }
        dependencies = [{ name = "registry_a", version = "1.0.0" }]
        "#,
    )
    .expect("valid lockfile");

    let diagnostics = produce_build_plan(
        plan_request(DependencySelection::Normal),
        vec![algebra],
        lockfile,
    )
    .unwrap_err();
    assert!(
        diagnostics
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic.kind, ManifestDiagnosticKind::DependencyCycle))
    );
}

#[test]
fn build_plan_expands_registry_dependencies_that_sort_before_their_parent() {
    let algebra = workspace_package(
        "algebra",
        r#"
        [package]
        name = "algebra"
        version = "1.0.0"

        [dependencies]
        registry_z = "1.0.0"
        "#,
    );
    let first_lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = [{ name = "registry_z", version = "1.0.0" }]

        [[package]]
        name = "registry_z"
        version = "1.0.0"
        source = { kind = "registry", registry = "default", checksum = "sha256:z" }
        dependencies = [{ name = "registry_a", version = "1.0.0" }]

        [[package]]
        name = "registry_a"
        version = "1.0.0"
        source = { kind = "registry", registry = "default", checksum = "sha256:a" }
        dependencies = [{ name = "registry_b", version = "1.0.0" }]

        [[package]]
        name = "registry_b"
        version = "1.0.0"
        source = { kind = "registry", registry = "default", checksum = "sha256:b" }
        dependencies = []
        "#,
    )
    .expect("valid lockfile");
    let second_lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "registry_b"
        version = "1.0.0"
        source = { kind = "registry", registry = "default", checksum = "sha256:b" }
        dependencies = []

        [[package]]
        name = "registry_a"
        version = "1.0.0"
        source = { kind = "registry", registry = "default", checksum = "sha256:a" }
        dependencies = [{ name = "registry_b", version = "1.0.0" }]

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = [{ name = "registry_z", version = "1.0.0" }]

        [[package]]
        name = "registry_z"
        version = "1.0.0"
        source = { kind = "registry", registry = "default", checksum = "sha256:z" }
        dependencies = [{ name = "registry_a", version = "1.0.0" }]
        "#,
    )
    .expect("valid shuffled lockfile");

    let first = produce_build_plan(
        plan_request(DependencySelection::Normal),
        vec![algebra.clone()],
        first_lockfile,
    )
    .expect("valid first plan");
    let second = produce_build_plan(
        plan_request(DependencySelection::Normal),
        vec![algebra],
        second_lockfile,
    )
    .expect("valid second plan");
    assert_eq!(first, second);
    assert_eq!(
        first
            .packages
            .iter()
            .map(|package| package.package_id.as_str())
            .collect::<Vec<_>>(),
        vec!["registry_b", "registry_a", "registry_z", "algebra"]
    );
    assert!(
        first
            .dependency_graph
            .edges
            .iter()
            .any(|edge| { edge.dependent == "registry_a" && edge.dependency == "registry_b" })
    );
}

#[test]
fn build_plan_rejects_manifest_version_conflicts() {
    let algebra = workspace_package(
        "algebra",
        r#"
        [package]
        name = "algebra"
        version = "1.0.0"

        [dependencies]
        topology = "^1.0.0"
        "#,
    );
    let topology = workspace_package(
        "topology",
        r#"
        [package]
        name = "topology"
        version = "2.0.0"
        "#,
    );
    let lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = [{ name = "topology", version = "2.0.0" }]

        [[package]]
        name = "topology"
        version = "2.0.0"
        source = { kind = "workspace", path = "topology" }
        dependencies = []
        "#,
    )
    .expect("valid lockfile");

    let diagnostics = produce_build_plan(
        plan_request(DependencySelection::Normal),
        vec![algebra, topology],
        lockfile,
    )
    .unwrap_err();
    assert_eq!(
        diagnostics
            .diagnostics()
            .iter()
            .filter(|diagnostic| {
                matches!(
                    diagnostic.kind,
                    ManifestDiagnosticKind::InvalidDependencyVersion
                ) && diagnostic.location.key == "algebra.dependencies.topology"
            })
            .count(),
        1
    );
}

#[test]
fn build_plan_rejects_unsupported_editions() {
    let algebra = workspace_package(
        "algebra",
        r#"
        [package]
        name = "algebra"
        version = "1.0.0"
        edition = "2099"
        "#,
    );
    let lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = []
        "#,
    )
    .expect("valid lockfile");

    let diagnostics = produce_build_plan(
        plan_request(DependencySelection::Normal),
        vec![algebra],
        lockfile,
    )
    .unwrap_err();
    assert!(
        diagnostics.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic.kind,
            ManifestDiagnosticKind::UnsupportedEdition
        ))
    );
}

#[test]
fn build_plan_activates_dev_dependencies_only_when_requested() {
    let algebra = workspace_package(
        "algebra",
        r#"
        [package]
        name = "algebra"
        version = "1.0.0"

        [dev-dependencies]
        test_utils = "1.0.0"
        "#,
    );
    let test_utils = workspace_package(
        "test_utils",
        r#"
        [package]
        name = "test_utils"
        version = "1.0.0"
        "#,
    );
    let lockfile = parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "algebra"
        version = "1.0.0"
        source = { kind = "workspace", path = "algebra" }
        dependencies = [{ name = "test_utils", version = "1.0.0" }]

        [[package]]
        name = "test_utils"
        version = "1.0.0"
        source = { kind = "workspace", path = "test_utils" }
        dependencies = []
        "#,
    )
    .expect("valid lockfile");

    let normal = produce_build_plan(
        plan_request(DependencySelection::Normal),
        vec![algebra.clone(), test_utils.clone()],
        lockfile.clone(),
    )
    .expect("valid normal plan");
    assert!(normal.dependency_graph.edges.is_empty());
    let normal_algebra = normal
        .packages
        .iter()
        .find(|package| package.package_id.as_str() == "algebra")
        .expect("algebra package plan exists");
    assert_eq!(normal_algebra.dependencies.len(), 1);
    assert_eq!(normal_algebra.dependencies[0].kind, DependencyKind::Dev);

    let with_dev = produce_build_plan(
        plan_request(DependencySelection::NormalAndDev),
        vec![algebra, test_utils],
        lockfile,
    )
    .expect("valid dev plan");
    assert_eq!(with_dev.dependency_graph.edges.len(), 1);
    assert_eq!(with_dev.dependency_graph.edges[0].kind, DependencyKind::Dev);
    let dev_algebra = with_dev
        .packages
        .iter()
        .find(|package| package.package_id.as_str() == "algebra")
        .expect("algebra package plan exists");
    assert_eq!(normal_algebra.dependencies, dev_algebra.dependencies);
}

#[test]
fn parses_explicit_version_ranges() {
    let manifest = parse_package_manifest(
        r#"
        [package]
        name = "ranges"
        version = "1.0.0"

        [dependencies]
        algebra = ">= 1.0, < 2.0"
        "#,
    )
    .expect("valid range");

    let VersionConstraint::Range(comparators) = &manifest.dependencies[0].version else {
        panic!("expected range constraint");
    };
    assert_eq!(comparators[0].op, VersionComparison::GreaterEqual);
    assert_eq!(comparators[0].version, Version::new(1, 0, 0));
    assert_eq!(comparators[1].op, VersionComparison::Less);
    assert_eq!(comparators[1].version, Version::new(2, 0, 0));
}

fn workspace_package(member_path: &str, manifest: &str) -> WorkspacePackage {
    WorkspacePackage {
        member_path: member_path.to_owned(),
        manifest: parse_package_manifest(manifest).expect("valid package manifest"),
    }
}

fn plan_request(selection: DependencySelection) -> PlanRequest {
    PlanRequest {
        workspace_root: WorkspaceRoot::new("workspace"),
        dependency_selection: selection,
        toolchain: ToolchainInfo::new("mizar-evo-test"),
    }
}
