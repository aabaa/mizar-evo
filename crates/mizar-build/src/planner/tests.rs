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

#[test]
fn computation_replay_parsing_preserves_defaults_and_boolean_type_errors() {
    let package = "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n";
    let default = VerifierConfig::default();
    assert!(!default.computation_replay);
    assert_eq!(
        (
            default.max_axioms,
            default.atp_timeout,
            default.default_solver,
            default.require_kernel_certificates
        ),
        (128, 30, Solver::Auto, true)
    );
    for (settings, replay) in [
        ("", false),
        ("[verifier]\n", false),
        ("[verifier]\nmax_axioms = 128\n", false),
        ("[verifier]\ncomputation_replay = false\n", false),
        ("[verifier]\ncomputation_replay = true\n", true),
    ] {
        let manifest = parse_package_manifest(&format!("{package}{settings}")).unwrap();
        assert_eq!(
            manifest.verifier,
            VerifierConfig {
                computation_replay: replay,
                ..default.clone()
            }
        );
    }
    for (value, kind) in [
        ("1", "integer"),
        ("\"true\"", "string"),
        ("[]", "array"),
        ("{}", "table"),
    ] {
        let errors = parse_package_manifest(&format!(
            "{package}[verifier]\ncomputation_replay = {value}\n"
        ))
        .unwrap_err();
        assert_eq!(
            errors.diagnostics(),
            &[super::ManifestDiagnostic::new(
                "<package manifest>",
                "verifier.computation_replay",
                ManifestDiagnosticKind::InvalidType,
                Some(kind.to_owned()),
            )]
        );
    }
}

#[test]
fn computation_replay_stays_package_local_and_changes_only_typed_configuration() {
    let lock = parse_lockfile("schema_version = 1\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"app\" }\ndependencies = [{ name = \"core\", version = \"1.0.0\" }]\n[[package]]\nname = \"core\"\nversion = \"1.0.0\"\nsource = { kind = \"workspace\", path = \"core\" }\ndependencies = []\n").unwrap();
    let mut clean = None;
    for (replay, core_replay) in [(false, true), (true, false), (true, true)] {
        let core = workspace_package(
            "core",
            &format!(
                "[package]\nname = \"core\"\nversion = \"1.0.0\"\n[verifier]\ncomputation_replay = {core_replay}\n"
            ),
        );
        let app = workspace_package(
            "app",
            &format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\n[dependencies]\ncore = \"1.0.0\"\n[verifier]\nmax_axioms = 64\ncomputation_replay = {replay}\n"
            ),
        );
        let plan = produce_build_plan(
            plan_request(DependencySelection::Normal),
            vec![app.clone(), core.clone()],
            lock.clone(),
        )
        .unwrap();
        let reversed = produce_build_plan(
            plan_request(DependencySelection::Normal),
            vec![core.clone(), app],
            lock.clone(),
        )
        .unwrap();
        assert_eq!(plan, reversed);
        assert_eq!(
            plan.packages
                .iter()
                .map(|package| package.package_id.as_str())
                .collect::<Vec<_>>(),
            ["core", "app"]
        );
        assert_eq!(plan.verifier_config.packages.len(), plan.packages.len());
        for (package, aggregate) in plan.packages.iter().zip(&plan.verifier_config.packages) {
            assert_eq!(package.package_id.as_str(), aggregate.package_id);
            assert_eq!(package.verifier_config, aggregate.config);
            assert_eq!(
                package.verifier_config.computation_replay,
                if package.package_id.as_str() == "core" {
                    core_replay
                } else {
                    replay
                }
            );
        }
        assert_eq!(plan.packages[1].verifier_config.max_axioms, 64);
        let mut normalized = plan;
        for package in &mut normalized.packages {
            package.verifier_config.computation_replay = false;
        }
        for aggregate in &mut normalized.verifier_config.packages {
            aggregate.config.computation_replay = false;
        }
        if let Some(expected) = &clean {
            assert_eq!(&normalized, expected);
        } else {
            clean = Some(normalized);
        }
    }
}

#[test]
fn workspace_lock_paths_bind_root_and_nested_member_coordinates() {
    for (member, foreign) in [(".", "packages/library"), ("packages/library", ".")] {
        let package =
            workspace_package(member, "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n");
        for path in [member, foreign] {
            let lock = parse_lockfile(&format!("schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = {{ kind = \"workspace\", path = \"{path}\" }}\ndependencies = []\n")).unwrap();
            validate_lockfile_for_workspace(std::slice::from_ref(&package.manifest), &lock)
                .unwrap();
            let result = produce_build_plan(
                plan_request(DependencySelection::Normal),
                vec![package.clone()],
                lock,
            );
            if path == member {
                let plan = result.unwrap();
                assert!(
                    matches!(&plan.packages[0].source, PackagePlanSource::Workspace { root, .. } if root == member)
                );
            } else {
                assert_eq!(
                    result.unwrap_err().diagnostics(),
                    &[super::ManifestDiagnostic::new(
                        "<lockfile>",
                        "alpha.source.path",
                        ManifestDiagnosticKind::InvalidLockSource,
                        Some(format!("{path} != {member}"))
                    )]
                );
            }
        }
    }
}

#[test]
fn workspace_lock_paths_report_only_mismatched_members_in_canonical_order() {
    let alpha = workspace_package(
        "nested/first",
        "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n",
    );
    let zeta = workspace_package(
        "nested/second",
        "[package]\nname = \"zeta\"\nversion = \"0.1.0\"\n",
    );
    for (alpha_path, zeta_path, mismatches) in [
        (
            "nested/second",
            "nested/first",
            vec![
                ("alpha", "nested/second != nested/first"),
                ("zeta", "nested/first != nested/second"),
            ],
        ),
        (
            "nested/first",
            "other",
            vec![("zeta", "other != nested/second")],
        ),
    ] {
        let lock = parse_lockfile(&format!("schema_version = 1\n[[package]]\nname = \"zeta\"\nversion = \"0.1.0\"\nsource = {{ kind = \"workspace\", path = \"{zeta_path}\" }}\ndependencies = []\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = {{ kind = \"workspace\", path = \"{alpha_path}\" }}\ndependencies = []\n")).unwrap();
        let first = produce_build_plan(
            plan_request(DependencySelection::Normal),
            vec![zeta.clone(), alpha.clone()],
            lock.clone(),
        )
        .unwrap_err();
        let mut reversed_lock = lock;
        reversed_lock.packages.reverse();
        let second = produce_build_plan(
            plan_request(DependencySelection::Normal),
            vec![alpha.clone(), zeta.clone()],
            reversed_lock,
        )
        .unwrap_err();
        assert_eq!(first, second);
        let expected = mismatches
            .into_iter()
            .map(|(name, value)| {
                super::ManifestDiagnostic::new(
                    "<lockfile>",
                    format!("{name}.source.path"),
                    ManifestDiagnosticKind::InvalidLockSource,
                    Some(value.into()),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(first.diagnostics(), expected);
    }
}

#[test]
fn workspace_lock_paths_preserve_missing_version_and_kind_errors() {
    let package = workspace_package(
        "actual",
        "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n",
    );
    for text in [
        "schema_version = 1\npackage = []\n",
        "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.2.0\"\nsource = { kind = \"workspace\", path = \"different\" }\ndependencies = []\n",
        "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"sha256:alpha\" }\ndependencies = []\n",
    ] {
        let lock = parse_lockfile(text).unwrap();
        let expected =
            validate_lockfile_for_workspace(std::slice::from_ref(&package.manifest), &lock)
                .unwrap_err();
        let actual = produce_build_plan(
            plan_request(DependencySelection::Normal),
            vec![package.clone()],
            lock,
        )
        .unwrap_err();
        assert_eq!(actual, expected);
        assert!(
            !actual
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.location.key.ends_with(".source.path"))
        );
    }
}

#[test]
fn lock_content_hash_matches_independent_literal_profile_and_frame() {
    use super::{LockedDependency, LockedPackage, Lockfile, lockfile_content_hash};
    let lock = Lockfile {
        schema_version: u64::MAX,
        toolchain_version: None,
        packages: vec![
            LockedPackage {
                name: "alpha".into(),
                version: Version::parse("2.0.0+meta").unwrap(),
                source: LockSource::Registry {
                    registry: "default".into(),
                    checksum: "x".into(),
                },
                dependencies: vec![],
            },
            LockedPackage {
                name: "alpha".into(),
                version: Version::parse("10.2.3-rc.1+build.7").unwrap(),
                source: LockSource::Workspace { path: ".".into() },
                dependencies: vec![
                    LockedDependency {
                        name: "beta".into(),
                        version: Version::parse("2.4.6-rc.2+edge.9").unwrap(),
                    },
                    LockedDependency {
                        name: "beta".into(),
                        version: Version::new(10, 0, 0),
                    },
                ],
            },
        ],
    };
    let json = "[\"18446744073709551615\",[[\"alpha\",\"10.2.3-rc.1+build.7\",[\"workspace\",\".\"],[[\"beta\",\"10.0.0\"],[\"beta\",\"2.4.6-rc.2+edge.9\"]]],[\"alpha\",\"2.0.0+meta\",[\"registry\",\"default\",\"x\"],[]]]]\n";
    let expected = mizar_session::hash_text(
        "mizar-build/parsed-lock-content/v1\n[\"18446744073709551615\",[[\"alpha\",\"10.2.3-rc.1+build.7\",[\"workspace\",\".\"],[[\"beta\",\"10.0.0\"],[\"beta\",\"2.4.6-rc.2+edge.9\"]]],[\"alpha\",\"2.0.0+meta\",[\"registry\",\"default\",\"x\"],[]]]]\n",
    );
    let actual = lockfile_content_hash(&lock);
    assert_eq!(actual, expected);
    for wrong_frame in [
        json.to_owned(),
        format!("mizar-build/parsed-lock-content/v1{json}"),
        format!("mizar-build/parsed-lock-content/v1\n{}", json.trim_end()),
        format!("mizar-build/parsed-lock-content/v2\n{json}"),
    ] {
        assert_ne!(actual, mizar_session::hash_text(&wrong_frame));
    }
}

#[test]
fn lock_content_hash_retains_every_typed_field_and_unambiguous_strings() {
    use super::lockfile_content_hash;
    let unknown = parse_lockfile("schema_version = 1\ntoolchain = \"unsupported\"\npackage = []\n")
        .unwrap_err();
    assert_eq!(
        unknown.diagnostics(),
        &[super::ManifestDiagnostic::new(
            "<lockfile>",
            "toolchain",
            ManifestDiagnosticKind::UnknownField,
            None
        )]
    );
    let base = parse_lockfile("schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"1.2.3-rc.1+build.7\"\nsource = { kind = \"workspace\", path = \"actual\" }\ndependencies = [{ name = \"beta\", version = \"2.0.0\" }]\n[[package]]\nname = \"beta\"\nversion = \"2.0.0\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"x\" }\ndependencies = []\n").unwrap();
    let baseline = lockfile_content_hash(&base);
    for field in [
        "schema",
        "schema-high-bit",
        "package-name",
        "version",
        "prerelease",
        "build",
        "workspace-path",
        "source-tag",
        "registry",
        "checksum",
        "dependency-name",
        "dependency-version",
        "package-count",
        "dependency-count",
    ] {
        let mut changed = base.clone();
        match field {
            "schema" => changed.schema_version = u64::MAX,
            "schema-high-bit" => changed.schema_version = 1 + (1 << 63),
            "package-name" => changed.packages[0].name = "quote\"\n\\😀".into(),
            "version" => changed.packages[0].version.major = 3,
            "prerelease" => {
                changed.packages[0].version.pre = semver::Prerelease::new("rc.2").unwrap()
            }
            "build" => {
                changed.packages[0].version.build = semver::BuildMetadata::new("build.8").unwrap()
            }
            "workspace-path" => {
                changed.packages[0].source = LockSource::Workspace {
                    path: "different".into(),
                }
            }
            "source-tag" => {
                changed.packages[0].source = LockSource::Registry {
                    registry: "actual".into(),
                    checksum: "".into(),
                }
            }
            "registry" => {
                changed.packages[1].source = LockSource::Registry {
                    registry: "other".into(),
                    checksum: "x".into(),
                }
            }
            "checksum" => {
                changed.packages[1].source = LockSource::Registry {
                    registry: "default".into(),
                    checksum: "y".into(),
                }
            }
            "dependency-name" => changed.packages[0].dependencies[0].name = "other".into(),
            "dependency-version" => {
                changed.packages[0].dependencies[0].version =
                    Version::parse("2.0.0-rc.1+metadata").unwrap()
            }
            "package-count" => {
                changed.packages.pop();
            }
            "dependency-count" => changed.packages[0].dependencies.clear(),
            _ => unreachable!(),
        }
        assert_ne!(lockfile_content_hash(&changed), baseline, "{field}");
    }
    let mut left = base.clone();
    left.packages[1].source = LockSource::Registry {
        registry: "ab".into(),
        checksum: "c".into(),
    };
    let mut right = left.clone();
    right.packages[1].source = LockSource::Registry {
        registry: "a".into(),
        checksum: "bc".into(),
    };
    assert_ne!(lockfile_content_hash(&left), lockfile_content_hash(&right));
    left.packages[0].name = "a\"b".into();
    right = left.clone();
    right.packages[0].name = "a\\\"b".into();
    assert_ne!(lockfile_content_hash(&left), lockfile_content_hash(&right));
}

#[test]
fn lock_content_hash_orders_complete_tuples_and_retains_duplicate_multiplicity() {
    use super::{LockedDependency, lockfile_content_hash};
    let mut lock = parse_lockfile("schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"1.0.0\"\nsource = { kind = \"workspace\", path = \"actual\" }\ndependencies = []\n").unwrap();
    lock.packages[0].dependencies = vec![
        LockedDependency {
            name: "same".into(),
            version: Version::new(2, 0, 0),
        },
        LockedDependency {
            name: "same".into(),
            version: Version::new(10, 0, 0),
        },
    ];
    let mut other = lock.packages[0].clone();
    other.source = LockSource::Workspace {
        path: "other".into(),
    };
    lock.packages.push(other);
    let mut edge_only = lock.packages[0].clone();
    edge_only.dependencies.pop();
    lock.packages.push(edge_only);
    let baseline = lockfile_content_hash(&lock);
    let mut reversed = lock.clone();
    reversed.packages.reverse();
    for package in &mut reversed.packages {
        package.dependencies.reverse();
    }
    assert_eq!(lockfile_content_hash(&reversed), baseline);
    let mut duplicate = lock.clone();
    duplicate.packages.push(duplicate.packages[0].clone());
    assert_ne!(lockfile_content_hash(&duplicate), baseline);
    duplicate = lock.clone();
    let dependency = duplicate.packages[0].dependencies[0].clone();
    duplicate.packages[0].dependencies.push(dependency);
    assert_ne!(lockfile_content_hash(&duplicate), baseline);
}

#[test]
fn config_content_hash_matches_independent_literal_profile_and_solver_labels() {
    use super::{
        PackageBuildConfig, PackageVerifierConfig, WorkspaceBuildConfig, WorkspaceVerifierConfig,
        workspace_config_content_hash,
    };
    let mut verifier = WorkspaceVerifierConfig {
        packages: vec![PackageVerifierConfig {
            package_id: "quote\"\\\n😀".into(),
            config: VerifierConfig {
                max_axioms: u32::MAX,
                atp_timeout: 1 << 31,
                default_solver: Solver::Cvc5,
                computation_replay: true,
                require_kernel_certificates: false,
            },
        }],
    };
    let build = WorkspaceBuildConfig {
        packages: vec![PackageBuildConfig {
            package_id: "other".into(),
            config: BuildConfig {
                incremental: false,
                cache_dir: "a\tb\r\0".into(),
                artifact_dir: "é/\"\\😀".into(),
            },
        }],
    };
    let json = r#"[[["quote\"\\\n😀",4294967295,2147483648,"cvc5",true,false]],[["other",false,"a\tb\r\u0000","é/\"\\😀"]]]"#;
    let frame = format!("mizar-build/package-config-content/v1\n{json}\n");
    let actual = workspace_config_content_hash(&verifier, &build);
    assert_eq!(actual, mizar_session::hash_text(&frame));
    verifier.packages[0].package_id = "alpha".into();
    verifier.packages[0].config = VerifierConfig::default();
    let empty = WorkspaceBuildConfig { packages: vec![] };
    for (solver, label) in [
        (Solver::Auto, "auto"),
        (Solver::Vampire, "vampire"),
        (Solver::E, "e"),
        (Solver::Cvc5, "cvc5"),
        (Solver::Z3, "z3"),
    ] {
        verifier.packages[0].config.default_solver = solver;
        assert_eq!(
            workspace_config_content_hash(&verifier, &empty),
            mizar_session::hash_text(&format!(
                "mizar-build/package-config-content/v1\n[[[\"alpha\",128,30,\"{label}\",false,true]],[]]\n"
            ))
        );
    }
    verifier.packages[0].config.default_solver = Solver::Auto;
    for (max_axioms, atp_timeout) in [(0, 0), (1 << 31, u32::MAX), (u32::MAX, 1 << 31)] {
        verifier.packages[0].config.max_axioms = max_axioms;
        verifier.packages[0].config.atp_timeout = atp_timeout;
        assert_eq!(
            workspace_config_content_hash(&verifier, &empty),
            mizar_session::hash_text(&format!(
                "mizar-build/package-config-content/v1\n[[[\"alpha\",{max_axioms},{atp_timeout},\"auto\",false,true]],[]]\n"
            ))
        );
    }
}

#[test]
fn config_content_hash_retains_each_field_and_unambiguous_strings() {
    use super::{
        PackageBuildConfig, PackageVerifierConfig, WorkspaceBuildConfig, WorkspaceVerifierConfig,
        workspace_config_content_hash,
    };
    let verifier = WorkspaceVerifierConfig {
        packages: vec![PackageVerifierConfig {
            package_id: "alpha".into(),
            config: VerifierConfig::default(),
        }],
    };
    let build = WorkspaceBuildConfig {
        packages: vec![PackageBuildConfig {
            package_id: "alpha".into(),
            config: BuildConfig::default(),
        }],
    };
    let baseline = workspace_config_content_hash(&verifier, &build);
    for field in [
        "verifier-id",
        "max-zero",
        "max-high-bit",
        "max-u32",
        "timeout-zero",
        "timeout-high-bit",
        "timeout-u32",
        "solver",
        "replay",
        "certificates",
        "build-id",
        "incremental",
        "cache-dir",
        "artifact-dir",
    ] {
        let (mut v, mut b) = (verifier.clone(), build.clone());
        match field {
            "verifier-id" => v.packages[0].package_id = "quote\"\\\n😀".into(),
            "max-zero" => v.packages[0].config.max_axioms = 0,
            "max-high-bit" => v.packages[0].config.max_axioms = 1 << 31,
            "max-u32" => v.packages[0].config.max_axioms = u32::MAX,
            "timeout-zero" => v.packages[0].config.atp_timeout = 0,
            "timeout-high-bit" => v.packages[0].config.atp_timeout = 1 << 31,
            "timeout-u32" => v.packages[0].config.atp_timeout = u32::MAX,
            "solver" => v.packages[0].config.default_solver = Solver::Vampire,
            "replay" => v.packages[0].config.computation_replay = true,
            "certificates" => v.packages[0].config.require_kernel_certificates = false,
            "build-id" => b.packages[0].package_id.clear(),
            "incremental" => b.packages[0].config.incremental = false,
            "cache-dir" => b.packages[0].config.cache_dir = "quote\"\\\n😀".into(),
            "artifact-dir" => b.packages[0].config.artifact_dir.clear(),
            _ => unreachable!(),
        }
        assert_ne!(workspace_config_content_hash(&v, &b), baseline, "{field}");
    }
    for (left_cache, left_artifact, right_cache, right_artifact) in [
        ("ab", "c", "a", "bc"),
        ("a\"b", "c", "a\\\"b", "c"),
        ("é", "c", "e\u{301}", "c"),
    ] {
        let mut first = build.clone();
        first.packages[0].config.cache_dir = left_cache.into();
        first.packages[0].config.artifact_dir = left_artifact.into();
        let mut second = first.clone();
        second.packages[0].config.cache_dir = right_cache.into();
        second.packages[0].config.artifact_dir = right_artifact.into();
        assert_ne!(
            workspace_config_content_hash(&verifier, &first),
            workspace_config_content_hash(&verifier, &second)
        );
    }
}

#[test]
fn config_content_hash_sorts_full_independent_records_and_preserves_association_and_counts() {
    use super::{
        PackageBuildConfig, PackageVerifierConfig, WorkspaceBuildConfig, WorkspaceVerifierConfig,
        workspace_config_content_hash,
    };
    let mut verifier = WorkspaceVerifierConfig {
        packages: [2, 10]
            .map(|max_axioms| PackageVerifierConfig {
                package_id: "alpha".into(),
                config: VerifierConfig {
                    max_axioms,
                    ..VerifierConfig::default()
                },
            })
            .into(),
    };
    let mut build = WorkspaceBuildConfig {
        packages: ["z", "a"]
            .map(|cache_dir| PackageBuildConfig {
                package_id: "alpha".into(),
                config: BuildConfig {
                    cache_dir: cache_dir.into(),
                    ..BuildConfig::default()
                },
            })
            .into(),
    };
    let baseline = workspace_config_content_hash(&verifier, &build);
    assert_eq!(
        baseline,
        mizar_session::hash_text(
            "mizar-build/package-config-content/v1\n[[[\"alpha\",10,30,\"auto\",false,true],[\"alpha\",2,30,\"auto\",false,true]],[[\"alpha\",true,\"a\",\"build\"],[\"alpha\",true,\"z\",\"build\"]]]\n"
        )
    );
    for reverse_verifier in [false, true] {
        for reverse_build in [false, true] {
            let (mut v, mut b) = (verifier.clone(), build.clone());
            if reverse_verifier {
                v.packages.reverse();
            }
            if reverse_build {
                b.packages.reverse();
            }
            assert_eq!(workspace_config_content_hash(&v, &b), baseline);
        }
    }
    for duplicate_verifier in [false, true] {
        let (mut v, mut b) = (verifier.clone(), build.clone());
        if duplicate_verifier {
            v.packages.push(v.packages[0].clone());
        } else {
            b.packages.push(b.packages[0].clone());
        }
        assert_ne!(workspace_config_content_hash(&v, &b), baseline);
    }
    verifier.packages[1].package_id = "zeta".into();
    build.packages[1].package_id = "zeta".into();
    let associated = workspace_config_content_hash(&verifier, &build);
    for swap_verifier in [false, true] {
        let (mut v, mut b) = (verifier.clone(), build.clone());
        if swap_verifier {
            v.packages.swap(0, 1);
            v.packages[0].package_id = "alpha".into();
            v.packages[1].package_id = "zeta".into();
        } else {
            b.packages.swap(0, 1);
            b.packages[0].package_id = "alpha".into();
            b.packages[1].package_id = "zeta".into();
        }
        assert_ne!(workspace_config_content_hash(&v, &b), associated);
    }
    let (empty_v, empty_b) = (
        WorkspaceVerifierConfig { packages: vec![] },
        WorkspaceBuildConfig { packages: vec![] },
    );
    let empty = workspace_config_content_hash(&empty_v, &empty_b);
    assert_eq!(
        empty,
        mizar_session::hash_text("mizar-build/package-config-content/v1\n[[],[]]\n")
    );
    let verifier_only = workspace_config_content_hash(&verifier, &empty_b);
    let build_only = workspace_config_content_hash(&empty_v, &build);
    assert_ne!(verifier_only, build_only);
    assert_ne!(verifier_only, empty);
    assert_ne!(build_only, empty);
    assert_ne!(verifier_only, associated);
    assert_ne!(build_only, associated);
}

#[test]
fn config_content_hash_uses_effective_plan_defaults_without_other_manifest_identity() {
    use super::workspace_config_content_hash;
    let lock = parse_lockfile("schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n").unwrap();
    for suffix in [
        "",
        "[verifier]\nmax_axioms = 128\natp_timeout = 30\ndefault_solver = \"auto\"\ncomputation_replay = false\nrequire_kernel_certificates = true\n[build]\nincremental = true\ncache_dir = \".mizar-cache\"\nartifact_dir = \"build\"\n",
        "authors = [\"different\"]\nlicense = \"MIT\"\ndescription = \"separate metadata\"\n",
    ] {
        let plan = produce_build_plan(
            plan_request(DependencySelection::Normal),
            vec![workspace_package(
                "alpha",
                &format!("[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n{suffix}"),
            )],
            lock.clone(),
        )
        .unwrap();
        let digest = workspace_config_content_hash(&plan.verifier_config, &plan.build_config);
        assert_eq!(
            digest,
            mizar_session::hash_text(
                "mizar-build/package-config-content/v1\n[[[\"alpha\",128,30,\"auto\",false,true]],[[\"alpha\",true,\".mizar-cache\",\"build\"]]]\n"
            )
        );
    }
}

#[test]
fn config_content_hash_orders_ids_before_config_and_complete_tied_tails() {
    use super::{
        PackageBuildConfig, PackageVerifierConfig, WorkspaceBuildConfig, WorkspaceVerifierConfig,
        workspace_config_content_hash,
    };
    let verifier = WorkspaceVerifierConfig {
        packages: [("alpha", 2, 2), ("zeta", 10, 30), ("alpha", 2, 10)]
            .map(
                |(package_id, max_axioms, atp_timeout)| PackageVerifierConfig {
                    package_id: package_id.into(),
                    config: VerifierConfig {
                        max_axioms,
                        atp_timeout,
                        ..VerifierConfig::default()
                    },
                },
            )
            .into(),
    };
    let build = WorkspaceBuildConfig {
        packages: [
            ("alpha", "z", "z"),
            ("zeta", "a", "build"),
            ("alpha", "z", "a"),
        ]
        .map(|(package_id, cache_dir, artifact_dir)| PackageBuildConfig {
            package_id: package_id.into(),
            config: BuildConfig {
                cache_dir: cache_dir.into(),
                artifact_dir: artifact_dir.into(),
                ..BuildConfig::default()
            },
        })
        .into(),
    };
    let expected = mizar_session::hash_text(
        "mizar-build/package-config-content/v1\n[[[\"alpha\",2,10,\"auto\",false,true],[\"alpha\",2,2,\"auto\",false,true],[\"zeta\",10,30,\"auto\",false,true]],[[\"alpha\",true,\"z\",\"a\"],[\"alpha\",true,\"z\",\"z\"],[\"zeta\",true,\"a\",\"build\"]]]\n",
    );
    for reverse_verifier in [false, true] {
        for reverse_build in [false, true] {
            let (mut v, mut b) = (verifier.clone(), build.clone());
            if reverse_verifier {
                v.packages.reverse();
            }
            if reverse_build {
                b.packages.reverse();
            }
            assert_eq!(workspace_config_content_hash(&v, &b), expected);
        }
    }
}

#[test]
fn package_toolchain_constraint_parses_existing_forms_and_validated_clone() {
    use super::VersionComparator;
    let package = "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n";
    let full = Version::parse("1.4.2-rc.1+build.7").unwrap();
    for (text, expected) in [
        (None, None),
        (
            Some("1.4.2"),
            Some(VersionConstraint::Exact(Version::new(1, 4, 2))),
        ),
        (
            Some("1.4.2-rc.1+build.7"),
            Some(VersionConstraint::Exact(full.clone())),
        ),
        (
            Some("^1"),
            Some(VersionConstraint::Caret(Version::new(1, 0, 0))),
        ),
        (
            Some(" ^1.4 "),
            Some(VersionConstraint::Caret(Version::new(1, 4, 0))),
        ),
        (
            Some("^1.4.2-rc.1+build.7"),
            Some(VersionConstraint::Caret(full.clone())),
        ),
        (
            Some("~1"),
            Some(VersionConstraint::Tilde(Version::new(1, 0, 0))),
        ),
        (
            Some("~1.4"),
            Some(VersionConstraint::Tilde(Version::new(1, 4, 0))),
        ),
        (
            Some("~1.4.2-rc.1+build.7"),
            Some(VersionConstraint::Tilde(full.clone())),
        ),
        (
            Some(">=1.4, <2, >1.0.0, <=1.4.2-rc.1+build.7"),
            Some(VersionConstraint::Range(vec![
                VersionComparator {
                    op: VersionComparison::GreaterEqual,
                    version: Version::new(1, 4, 0),
                },
                VersionComparator {
                    op: VersionComparison::Less,
                    version: Version::new(2, 0, 0),
                },
                VersionComparator {
                    op: VersionComparison::Greater,
                    version: Version::new(1, 0, 0),
                },
                VersionComparator {
                    op: VersionComparison::LessEqual,
                    version: full,
                },
            ])),
        ),
    ] {
        let suffix = text.map_or_else(String::new, |text| format!("toolchain = {text:?}\n"));
        let manifest = parse_package_manifest(&format!("{package}{suffix}")).unwrap();
        assert_eq!(manifest.toolchain_constraint, expected, "{text:?}");
        let validated = validate_package_manifest(&manifest).unwrap();
        assert_eq!(validated.manifest, manifest);
        assert_eq!(validated.manifest.toolchain_constraint, expected);
        let lock = parse_lockfile("schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n").unwrap();
        let plan = produce_build_plan(
            plan_request(DependencySelection::Normal),
            vec![WorkspacePackage {
                member_path: "alpha".into(),
                manifest,
            }],
            lock,
        )
        .unwrap();
        assert_eq!(plan.packages[0].toolchain_constraint, expected);
    }
}

#[test]
fn package_toolchain_constraint_errors_preserve_internal_locations_values_and_barrier() {
    let package = "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n";
    for (value, kind, rejected) in [
        ("\"\"", ManifestDiagnosticKind::InvalidVersionConstraint, ""),
        (
            "\"1.4\"",
            ManifestDiagnosticKind::InvalidVersionConstraint,
            "1.4",
        ),
        (
            "\"*\"",
            ManifestDiagnosticKind::InvalidVersionConstraint,
            "*",
        ),
        (
            "\"^bad\"",
            ManifestDiagnosticKind::InvalidVersionConstraint,
            "bad.0.0",
        ),
        (
            "\"~1.2.bad\"",
            ManifestDiagnosticKind::InvalidVersionConstraint,
            "1.2.bad",
        ),
        (
            "\">=1.4, wat\"",
            ManifestDiagnosticKind::InvalidVersionConstraint,
            "wat",
        ),
        (
            "\">=1.4,\"",
            ManifestDiagnosticKind::InvalidVersionConstraint,
            "",
        ),
        ("true", ManifestDiagnosticKind::InvalidType, "boolean"),
        ("1", ManifestDiagnosticKind::InvalidType, "integer"),
        ("1.4", ManifestDiagnosticKind::InvalidType, "float"),
        ("[]", ManifestDiagnosticKind::InvalidType, "array"),
        ("{}", ManifestDiagnosticKind::InvalidType, "table"),
        (
            "2025-01-01",
            ManifestDiagnosticKind::InvalidType,
            "datetime",
        ),
    ] {
        let error = parse_package_manifest(&format!("{package}toolchain = {value}\n")).unwrap_err();
        assert_eq!(
            error.diagnostics(),
            &[super::ManifestDiagnostic::new(
                "<package manifest>",
                "package.toolchain",
                kind,
                Some(rejected.into()),
            )],
            "{value}"
        );
    }
    let errors = parse_package_manifest(&format!(
        "{package}toolchain_extra = \"^1.4\"\ntoolchain = false\n"
    ))
    .unwrap_err();
    assert_eq!(
        errors.diagnostics(),
        &[
            super::ManifestDiagnostic::new(
                "<package manifest>",
                "package.toolchain",
                ManifestDiagnosticKind::InvalidType,
                Some("boolean".into())
            ),
            super::ManifestDiagnostic::new(
                "<package manifest>",
                "package.toolchain_extra",
                ManifestDiagnosticKind::UnknownField,
                None
            ),
        ]
    );
}

#[test]
fn package_toolchain_constraints_remain_package_local_with_registry_metadata_absent() {
    let alpha = workspace_package(
        "alpha",
        "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\ntoolchain = \"<1\"\n[dependencies]\ndep = \"1.0.0\"\n",
    );
    let beta = workspace_package(
        "beta",
        "[package]\nname = \"beta\"\nversion = \"0.1.0\"\ntoolchain = \">=2\"\n",
    );
    let lock = parse_lockfile("schema_version = 1\n[[package]]\nname = \"beta\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"beta\" }\ndependencies = []\n[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"sha256:dep\" }\ndependencies = []\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = [{ name = \"dep\", version = \"1.0.0\" }]\n").unwrap();
    let request = plan_request(DependencySelection::Normal);
    let plan = produce_build_plan(
        request.clone(),
        vec![beta.clone(), alpha.clone()],
        lock.clone(),
    )
    .unwrap();
    assert_eq!(
        plan,
        produce_build_plan(request.clone(), vec![alpha, beta], lock.clone()).unwrap()
    );
    assert_eq!(plan.toolchain, request.toolchain);
    assert_eq!(plan.lockfile, lock);
    assert_eq!(
        plan.packages
            .iter()
            .map(|package| package.package_id.as_str())
            .collect::<Vec<_>>(),
        ["beta", "dep", "alpha"]
    );
    assert_eq!(
        plan.packages[2].toolchain_constraint,
        Some(VersionConstraint::Range(vec![super::VersionComparator {
            op: VersionComparison::Less,
            version: Version::new(1, 0, 0)
        }]))
    );
    assert_eq!(
        plan.packages[0].toolchain_constraint,
        Some(VersionConstraint::Range(vec![super::VersionComparator {
            op: VersionComparison::GreaterEqual,
            version: Version::new(2, 0, 0)
        }]))
    );
    assert!(matches!(
        plan.packages[1].source,
        PackagePlanSource::Registry { .. }
    ));
    assert_eq!(plan.packages[1].toolchain_constraint, None);
}

#[test]
fn locked_toolchain_metadata_parses_exact_versions_and_retains_plan_inputs() {
    let package = workspace_package(
        "alpha",
        "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\ntoolchain = \"<1\"\n",
    );
    let body = "[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n";
    for version in [None, Some("1.4.2"), Some("10.2.3-rc.1+build.7")] {
        let field = version.map_or(String::new(), |v| format!("toolchain_version = \"{v}\"\n"));
        let lock = parse_lockfile(&format!("schema_version = 1\n{field}{body}")).unwrap();
        assert_eq!(
            lock.toolchain_version,
            version.map(|v| Version::parse(v).unwrap())
        );
        validate_lockfile_for_workspace(std::slice::from_ref(&package.manifest), &lock).unwrap();
        let request = plan_request(DependencySelection::Normal);
        let plan =
            produce_build_plan(request.clone(), vec![package.clone()], lock.clone()).unwrap();
        assert_eq!(plan.lockfile, lock);
        assert_eq!(plan.toolchain, request.toolchain);
        assert_eq!(
            plan.packages[0].toolchain_constraint,
            package.manifest.toolchain_constraint
        );
    }
}

#[test]
fn locked_toolchain_metadata_errors_preserve_locations_values_order_and_barrier() {
    for (value, kind, rejected) in [
        ("\"\"", ManifestDiagnosticKind::InvalidVersion, ""),
        ("\"1\"", ManifestDiagnosticKind::InvalidVersion, "1"),
        ("\"1.4\"", ManifestDiagnosticKind::InvalidVersion, "1.4"),
        (
            "\"^1.4.2\"",
            ManifestDiagnosticKind::InvalidVersion,
            "^1.4.2",
        ),
        (
            "\"~1.4.2\"",
            ManifestDiagnosticKind::InvalidVersion,
            "~1.4.2",
        ),
        (
            "\">=1.0, <2.0\"",
            ManifestDiagnosticKind::InvalidVersion,
            ">=1.0, <2.0",
        ),
        (
            "\" 1.4.2\"",
            ManifestDiagnosticKind::InvalidVersion,
            " 1.4.2",
        ),
        (
            "\"1.4.2 \"",
            ManifestDiagnosticKind::InvalidVersion,
            "1.4.2 ",
        ),
        (
            "\"v1.4.2\"",
            ManifestDiagnosticKind::InvalidVersion,
            "v1.4.2",
        ),
        (
            "\"1.4.2-01\"",
            ManifestDiagnosticKind::InvalidVersion,
            "1.4.2-01",
        ),
        ("true", ManifestDiagnosticKind::InvalidType, "boolean"),
        ("1", ManifestDiagnosticKind::InvalidType, "integer"),
        ("1.4", ManifestDiagnosticKind::InvalidType, "float"),
        ("[]", ManifestDiagnosticKind::InvalidType, "array"),
        ("{}", ManifestDiagnosticKind::InvalidType, "table"),
        (
            "2025-01-01",
            ManifestDiagnosticKind::InvalidType,
            "datetime",
        ),
    ] {
        let error = parse_lockfile(&format!(
            "schema_version = 1\ntoolchain_version = {value}\npackage = []\n"
        ))
        .unwrap_err();
        assert_eq!(
            error.diagnostics(),
            &[super::ManifestDiagnostic::new(
                "<lockfile>",
                "toolchain_version",
                kind,
                Some(rejected.into()),
            )],
            "{value}"
        );
    }
    let errors = parse_lockfile(
        "toolchain_version = false\nschema_version = 2\nextra = true\npackage = []\n",
    )
    .unwrap_err();
    assert_eq!(
        errors.diagnostics(),
        &[
            super::ManifestDiagnostic::new(
                "<lockfile>",
                "extra",
                ManifestDiagnosticKind::UnknownField,
                None
            ),
            super::ManifestDiagnostic::new(
                "<lockfile>",
                "schema_version",
                ManifestDiagnosticKind::InvalidLockfileSchema,
                Some("2".into())
            ),
            super::ManifestDiagnostic::new(
                "<lockfile>",
                "toolchain_version",
                ManifestDiagnosticKind::InvalidType,
                Some("boolean".into())
            ),
        ]
    );
}

#[test]
fn locked_toolchain_metadata_hash_matches_independent_frames_and_all_field_invariants() {
    use super::{LockedDependency, lockfile_content_hash};
    let mut lock = parse_lockfile("schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"10.2.3-rc.1+build.7\"\nsource = { kind = \"workspace\", path = \".\" }\ndependencies = [{ name = \"beta\", version = \"2.4.6-rc.2+edge.9\" }]\n[[package]]\nname = \"beta\"\nversion = \"2.4.6-rc.2+edge.9\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"x\" }\ndependencies = []\n").unwrap();
    lock.schema_version = u64::MAX;
    let old = "[\"18446744073709551615\",[[\"alpha\",\"10.2.3-rc.1+build.7\",[\"workspace\",\".\"],[[\"beta\",\"2.4.6-rc.2+edge.9\"]]],[\"beta\",\"2.4.6-rc.2+edge.9\",[\"registry\",\"default\",\"x\"],[]]]]\n";
    lock.toolchain_version = Some(Version::parse("4.5.6-rc.7+build.8").unwrap());
    let new = "[\"18446744073709551615\",[[\"alpha\",\"10.2.3-rc.1+build.7\",[\"workspace\",\".\"],[[\"beta\",\"2.4.6-rc.2+edge.9\"]]],[\"beta\",\"2.4.6-rc.2+edge.9\",[\"registry\",\"default\",\"x\"],[]]],\"4.5.6-rc.7+build.8\"]\n";
    let baseline = lockfile_content_hash(&lock);
    assert_eq!(
        baseline,
        mizar_session::hash_text(&format!("mizar-build/parsed-lock-content/v2\n{new}"))
    );
    for wrong in [
        new.to_owned(),
        format!("mizar-build/parsed-lock-content/v1\n{new}"),
        format!("mizar-build/parsed-lock-content/v2{new}"),
        format!("mizar-build/parsed-lock-content/v2\n{}", new.trim_end()),
        format!("mizar-build/parsed-lock-content/v2\n{old}"),
    ] {
        assert_ne!(baseline, mizar_session::hash_text(&wrong));
    }
    for field in 0..19 {
        let mut changed = lock.clone();
        match field {
            0 => changed.schema_version = 1,
            1 => changed.schema_version ^= 1 << 63,
            2 => changed.packages[0].name = "quote\"\n\\😀".into(),
            3 => changed.packages[0].version.major += 1,
            4 => changed.packages[0].version.pre = semver::Prerelease::new("rc.2").unwrap(),
            5 => changed.packages[0].version.build = semver::BuildMetadata::new("build.8").unwrap(),
            6 => {
                changed.packages[0].source = LockSource::Workspace {
                    path: "other".into(),
                }
            }
            7 => {
                changed.packages[0].source = LockSource::Registry {
                    registry: ".".into(),
                    checksum: "".into(),
                }
            }
            8 => {
                changed.packages[1].source = LockSource::Registry {
                    registry: "other".into(),
                    checksum: "x".into(),
                }
            }
            9 => {
                changed.packages[1].source = LockSource::Registry {
                    registry: "default".into(),
                    checksum: "y".into(),
                }
            }
            10 => changed.packages[0].dependencies[0].name = "other".into(),
            11 => changed.packages[0].dependencies[0].version.major += 1,
            12 => {
                changed.packages[0].dependencies[0].version.pre =
                    semver::Prerelease::new("rc.3").unwrap()
            }
            13 => {
                changed.packages[0].dependencies[0].version.build =
                    semver::BuildMetadata::new("edge.10").unwrap()
            }
            14 => {
                changed.packages.pop();
            }
            15 => changed.packages[0].dependencies.clear(),
            16 => changed.toolchain_version.as_mut().unwrap().major += 1,
            17 => {
                changed.toolchain_version.as_mut().unwrap().pre =
                    semver::Prerelease::new("rc.8").unwrap()
            }
            18 => {
                changed.toolchain_version.as_mut().unwrap().build =
                    semver::BuildMetadata::new("build.9").unwrap()
            }
            _ => unreachable!(),
        }
        assert_ne!(lockfile_content_hash(&changed), baseline, "field {field}");
    }
    lock.packages[0].dependencies.push(LockedDependency {
        name: "beta".into(),
        version: Version::new(10, 0, 0),
    });
    let mut other = lock.packages[0].clone();
    other.source = LockSource::Workspace {
        path: "other".into(),
    };
    lock.packages.push(other);
    let mut edge_only = lock.packages[0].clone();
    edge_only.dependencies.pop();
    lock.packages.push(edge_only);
    let baseline = lockfile_content_hash(&lock);
    let mut reversed = lock.clone();
    reversed.packages.reverse();
    for package in &mut reversed.packages {
        package.dependencies.reverse();
    }
    assert_eq!(lockfile_content_hash(&reversed), baseline);
    let mut duplicate = lock.clone();
    duplicate.packages.push(duplicate.packages[0].clone());
    assert_ne!(lockfile_content_hash(&duplicate), baseline);
    duplicate = lock.clone();
    let dependency = duplicate.packages[0].dependencies[0].clone();
    duplicate.packages[0].dependencies.push(dependency);
    assert_ne!(lockfile_content_hash(&duplicate), baseline);
}

#[test]
fn constraint_bound_overflow_preserves_public_lock_validation() {
    let cases: &[(&str, &[(&str, bool)])] = &[
        (
            "^18446744073709551615.4.5",
            &[
                ("18446744073709551615.4.5", true),
                ("18446744073709551615.5.0", true),
                ("18446744073709551615.4.4", false),
                ("18446744073709551615.4.5-rc.1", false),
                ("18446744073709551615.4.5+build.1", true),
            ],
        ),
        (
            "^0.18446744073709551615.4",
            &[
                ("0.18446744073709551615.4", true),
                ("0.18446744073709551615.5", true),
                ("0.18446744073709551615.3", false),
                ("1.0.0", false),
            ],
        ),
        (
            "^0.0.18446744073709551615",
            &[
                ("0.0.18446744073709551615", true),
                ("0.0.18446744073709551614", false),
                ("0.0.18446744073709551615-rc.1", false),
                ("0.0.18446744073709551615+build.1", true),
                ("0.1.0", false),
                ("1.0.0", false),
            ],
        ),
        (
            "~1.18446744073709551615.4",
            &[
                ("1.18446744073709551615.4", true),
                ("1.18446744073709551615.5", true),
                ("1.18446744073709551615.3", false),
                ("2.0.0", false),
            ],
        ),
        (
            "^18446744073709551614.4.5",
            &[
                ("18446744073709551614.4.5", true),
                ("18446744073709551614.4.4", false),
                ("18446744073709551615.0.0-rc.1", true),
                ("18446744073709551615.0.0", false),
                ("18446744073709551615.0.0+build.1", false),
            ],
        ),
        (
            "^0.18446744073709551614.4",
            &[
                ("0.18446744073709551614.4", true),
                ("0.18446744073709551614.3", false),
                ("0.18446744073709551615.0-rc.1", true),
                ("0.18446744073709551615.0", false),
            ],
        ),
        (
            "^0.0.18446744073709551614",
            &[
                ("0.0.18446744073709551614", true),
                ("0.0.18446744073709551613", false),
                ("0.0.18446744073709551615-rc.1", true),
                ("0.0.18446744073709551615", false),
            ],
        ),
        (
            "~1.18446744073709551614.4",
            &[
                ("1.18446744073709551614.4", true),
                ("1.18446744073709551614.3", false),
                ("1.18446744073709551615.0-rc.1", true),
                ("1.18446744073709551615.0", false),
            ],
        ),
    ];
    for &(constraint, versions) in cases {
        let manifest = parse_package_manifest(&format!(
            r#"
            [package]
            name = "alpha"
            version = "1.0.0"
            [dependencies]
            dep = "{constraint}"
            "#
        ))
        .expect("valid manifest constraint");
        for &(version, expected) in versions {
            let lock = parse_lockfile(&format!(
                r#"
                schema_version = 1
                [[package]]
                name = "alpha"
                version = "1.0.0"
                source = {{ kind = "workspace", path = "alpha" }}
                dependencies = [{{ name = "dep", version = "{version}" }}]
                [[package]]
                name = "dep"
                version = "{version}"
                source = {{ kind = "registry", registry = "default", checksum = "sha256:abcd" }}
                dependencies = []
                "#
            ))
            .expect("internally consistent lockfile");
            let result = validate_lockfile_for_workspace(std::slice::from_ref(&manifest), &lock);
            assert_eq!(result.is_ok(), expected, "{constraint} against {version}");
            if let Err(diagnostics) = result {
                assert_eq!(diagnostics.diagnostics().len(), 1);
                let diagnostic = &diagnostics.diagnostics()[0];
                assert_eq!(
                    diagnostic.kind,
                    ManifestDiagnosticKind::InvalidDependencyVersion
                );
                assert_eq!(diagnostic.location.path, "<lockfile>");
                assert_eq!(diagnostic.location.key, "alpha.dependencies.dep");
            }
        }
    }
}
