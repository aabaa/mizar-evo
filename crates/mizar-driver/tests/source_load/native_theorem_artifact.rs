use super::*;

#[test]
fn native_theorem_artifact_draft_matches_source_rows() {
    use mizar_artifact::verified_artifact::write_verified_artifact;
    use mizar_atp::problem::*;
    use mizar_ir::{projection::ProjectionExternalDependencyGap as Gap, storage::CollectInput};
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for (text, threshold) in [
        (original.to_owned(), usize::MAX),
        (original.replace("T:", "Renamed:"), 0),
        (format!("private {original}"), usize::MAX),
    ] {
        with_native_theorem_artifact_context(
            &text,
            false,
            threshold,
            "",
            |fixture, inputs, six, owner, refs, expected, _| {
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("explicit"),
                        max_steps: 64,
                    },
                };
                let verifier = if threshold == 0 {
                    VerifierPolicy::default()
                        .with_profile_id("artifact λ\0")
                        .with_checker_schema_version(7)
                } else {
                    VerifierPolicy::default()
                };
                let profile = LogicProfile::try_new(
                    "artifact-source",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let vc = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc, vc.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let root = fixture.root.join("actual-artifact");
                let before = (
                    six.clone(),
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    policy.clone(),
                    profile.clone(),
                    verifier.clone(),
                );
                let expected = expected.as_ref().unwrap().as_ref().unwrap();
                let (actual, _) = inputs
                    .project_native_theorem_artifact_draft(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        ("proof-witnesses/checked.json", &root),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(&actual, expected);
                assert_eq!(
                    write_verified_artifact(&actual.artifact).unwrap(),
                    write_verified_artifact(&expected.artifact).unwrap()
                );
                assert_eq!(
                    actual.artifact.compute_interface_hash().unwrap(),
                    actual.artifact.interface_hash
                );
                assert_eq!(
                    actual.artifact.compute_implementation_hash().unwrap(),
                    actual.artifact.implementation_hash
                );
                assert_eq!(
                    actual.external_dependency_gaps,
                    [
                        Gap::DriverSession,
                        Gap::DiagnosticsIntegration,
                        Gap::ProducerProjectionPayload,
                        Gap::ArtifactPublicationToken
                    ]
                );
                assert_eq!(
                    actual.artifact.exports.len(),
                    usize::from(!text.starts_with("private "))
                );
                assert!(
                    !actual.artifact.expressions.is_empty()
                        && !actual.artifact.obligations.is_empty()
                );
                assert!(
                    actual
                        .artifact
                        .exports
                        .iter()
                        .all(|export| export.proof_status.is_none())
                );
                assert_eq!(actual.artifact.expressions.len(), 9);
                assert_eq!(
                    actual.artifact.obligations[0].owner_origin_id.is_none(),
                    text.starts_with("private ")
                );
                assert_eq!(
                    actual.artifact.obligations[0].obligation_kind,
                    "theorem-proof-step"
                );
                assert_eq!(
                    actual.artifact.obligations[0]
                        .accepted_witness_obligation_id
                        .as_deref(),
                    Some(actual.artifact.obligations[0].obligation_id.as_str())
                );
                assert_eq!(
                    fs::read(root.join("proof-witnesses/checked.json")).unwrap(),
                    fs::read(
                        fixture
                            .root
                            .join("oracle-artifact/proof-witnesses/checked.json")
                    )
                    .unwrap()
                );
                assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
                assert_eq!(
                    (
                        six.clone(),
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone(),
                        policy,
                        profile,
                        verifier
                    ),
                    before
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (14, 0)
                );
            },
        );
    }
}

#[test]
fn native_theorem_artifact_draft_covers_real_expression_rows() {
    use mizar_atp::problem::*;
    use mizar_checker::{
        source_atomic_formula::SourceAtomicFormulaKind,
        source_term::SourcePrimaryTermKind,
        type_checker::{NormalizedTypeStatus, TypeHeadRef},
    };
    use mizar_ir::storage::CollectInput;
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for text in [
        original.replace(" X", " Bound"),
        format!(":: 日本語 λ\n{original}\n"),
    ] {
        with_native_theorem_artifact_context(
            &text,
            false,
            0,
            "",
            |fixture, inputs, six, owner, refs, expected, (inference, typed, metadata, source)| {
                let primary = typed.source_term().unwrap();
                let atomic = typed.source_atomic_formula().unwrap();
                let sites = primary
                    .terms()
                    .iter()
                    .map(|(_, row)| {
                        assert_eq!(row.kind(), SourcePrimaryTermKind::VariableReference);
                        row.site().clone()
                    })
                    .chain(atomic.formulas().iter().map(|(_, row)| {
                        assert_eq!(row.kind(), SourceAtomicFormulaKind::Equality);
                        row.site().clone()
                    }))
                    .collect::<std::collections::BTreeSet<_>>();
                let actual_sites = metadata
                    .expr_metadata()
                    .iter()
                    .map(|(_, row)| row.typed_site.clone())
                    .collect::<std::collections::BTreeSet<_>>();
                assert_eq!(sites, actual_sites);
                assert_eq!(metadata.expr_metadata().len(), sites.len());
                let table_before = (inference.clone(), typed.clone(), metadata.clone());
                let vc = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc, vc.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("explicit"),
                        max_steps: 64,
                    },
                };
                let verifier = VerifierPolicy::default()
                    .with_profile_id("artifact λ\0")
                    .with_checker_schema_version(7);
                let profile = LogicProfile::try_new(
                    "artifact-source",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let (actual, _) = inputs
                    .project_native_theorem_artifact_draft(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile, ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (
                            "proof-witnesses/checked.json",
                            &fixture.root.join("actual-artifact"),
                        ),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(&actual, expected.as_ref().unwrap().as_ref().unwrap());
                assert_eq!(
                    actual.artifact.expressions.len(),
                    metadata.expr_metadata().len()
                        + typed.source_type().unwrap().expressions().len()
                        + 1
                );
                let written = typed.source_type().unwrap();
                let mut expression_ids = metadata
                    .expr_metadata()
                    .iter()
                    .map(|(_, row)| row.expr.as_str().to_owned())
                    .collect::<std::collections::BTreeSet<_>>();
                for (root, row) in written.expressions().iter() {
                    let ordinal = written
                        .applications()
                        .iter()
                        .filter(|(_, application)| application.root() == root)
                        .map(|(_, application)| application.source_ordinal())
                        .min()
                        .unwrap();
                    let id = format!("source.theorem.type.{ordinal}");
                    assert!(expression_ids.insert(id.clone()));
                    let projected = actual
                        .artifact
                        .expressions
                        .iter()
                        .find(|value| value.expression_id == id)
                        .unwrap();
                    assert_eq!(
                        projected.expression_kind,
                        typed.nodes().node(row.site().node()).unwrap().kind.as_str()
                    );
                    assert_eq!(
                        projected.source_range.start_byte,
                        u64::try_from(row.source_range().start).unwrap()
                    );
                    assert_eq!(
                        projected.source_range.end_byte,
                        u64::try_from(row.source_range().end).unwrap()
                    );
                    assert_eq!(
                        projected.rendered_surface,
                        source
                            .get(row.source_range().start..row.source_range().end)
                            .unwrap()
                    );
                    assert_eq!(projected.inferred_type.as_deref(), Some("set"));
                    assert!(
                        projected.resolved_symbol.is_none()
                            && projected.inserted_coercions.is_empty()
                            && projected.active_thesis.is_none()
                            && projected.overload_resolution.is_none()
                    );
                }
                assert!(expression_ids.insert(format!(
                        "source.theorem.quantified.{}",
                        expected
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .artifact
                            .exports[0]
                            .fully_qualified_name
                    )));
                assert_eq!(
                    actual
                        .artifact
                        .expressions
                        .iter()
                        .map(|row| row.expression_id.clone())
                        .collect::<std::collections::BTreeSet<_>>(),
                    expression_ids
                );
                let mut typed_terms = 0;
                let mut untyped_formulas = 0;
                for (_, row) in metadata.expr_metadata().iter() {
                    let projected = actual
                        .artifact
                        .expressions
                        .iter()
                        .find(|value| value.expression_id == row.expr.as_str())
                        .unwrap();
                    assert_eq!(
                        projected.expression_kind,
                        typed
                            .nodes()
                            .node(row.typed_site.node())
                            .unwrap()
                            .kind
                            .as_str()
                    );
                    assert_eq!(
                        projected.rendered_surface,
                        source
                            .get(row.source_range.start..row.source_range.end)
                            .unwrap()
                    );
                    assert_eq!(
                        (
                            projected.source_range.start_byte,
                            projected.source_range.end_byte
                        ),
                        (
                            u64::try_from(row.source_range.start).unwrap(),
                            u64::try_from(row.source_range.end).unwrap()
                        )
                    );
                    if let Some(id) = row.final_type {
                        let ty = inference.normalized_types().get(id).unwrap();
                        assert_eq!(ty.status, NormalizedTypeStatus::Known);
                        assert_eq!(ty.head, TypeHeadRef::BuiltinSet);
                        assert!(
                            ty.args.is_empty()
                                && ty.attributes.positive().is_empty()
                                && ty.attributes.negative().is_empty()
                        );
                        assert_eq!(projected.inferred_type.as_deref(), Some("set"));
                        typed_terms += 1;
                    } else {
                        assert!(row.expr.as_str().starts_with("source.theorem.formula."));
                        assert_eq!(projected.inferred_type, None);
                        untyped_formulas += 1;
                    }
                    assert!(
                        projected.resolved_symbol.is_none()
                            && projected.inserted_coercions.is_empty()
                            && projected.overload_resolution.is_none()
                    );
                    assert_eq!(
                        projected.active_thesis,
                        expected
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .artifact
                            .expressions
                            .iter()
                            .find(|expected| expected.expression_id == row.expr.as_str())
                            .unwrap()
                            .active_thesis
                    );
                }
                assert!(typed_terms > 0 && untyped_formulas > 0);
                assert_eq!(
                    (inference.clone(), typed.clone(), metadata.clone()),
                    table_before
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (14, 0)
                );
            },
        );
    }
}

#[test]
fn native_theorem_artifact_draft_preserves_refusals_and_fresh_root() {
    use mizar_atp::problem::*;
    use mizar_ir::storage::CollectInput;
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in ["ordinary", "buffer", "late", "bytes", "link", "collected"] {
        let source = if case == "late" {
            text.replace("T:", "OutputSlotId:")
        } else {
            text.into()
        };
        let fault = if matches!(case, "bytes" | "link") {
            case
        } else {
            ""
        };
        with_native_theorem_artifact_context(
            &source,
            case == "buffer",
            usize::MAX,
            fault,
            |fixture, inputs, six, owner, refs, expected, _| {
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("explicit"),
                        max_steps: 64,
                    },
                };
                let verifier = VerifierPolicy::default();
                let profile = LogicProfile::try_new(
                    "artifact-source",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let vc = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc, vc.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let id = vcs.vcs()[0].id;
                let root = fixture.root.join("actual-artifact");
                let before = (
                    six.clone(),
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                );
                if case == "buffer" {
                    assert!(
                        inputs
                            .project_native_theorem_artifact_draft(
                                six,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", &root)
                            )
                            .unwrap()
                            .is_none()
                    );
                    assert!(!root.exists());
                } else if matches!(case, "bytes" | "link") {
                    assert!(expected.is_none());
                    assert!(
                        inputs
                            .project_native_theorem_artifact_draft(
                                six,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", &root)
                            )
                            .is_err()
                    );
                    assert!(!root.exists());
                } else if case == "collected" {
                    let keep = refs
                        .iter()
                        .filter(|row| {
                            row.output_kind() != &OutputKind::new("SourceTheoremCandidateCheck")
                        })
                        .map(|row| row.output())
                        .collect();
                    assert_eq!(
                        owner
                            .storage()
                            .collect(CollectInput {
                                snapshot: six.snapshot,
                                protected_outputs: keep
                            })
                            .outputs_dropped,
                        2
                    );
                    assert!(
                        inputs
                            .project_native_theorem_artifact_draft(
                                six,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", &root)
                            )
                            .is_err()
                    );
                    assert!(!root.exists());
                } else if case == "late" {
                    let error = expected.as_ref().unwrap().as_ref().unwrap_err();
                    assert!(error.contains("OutputSlotId"));
                    assert_eq!(
                        &inputs
                            .project_native_theorem_artifact_draft(
                                six,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", &root)
                            )
                            .unwrap_err(),
                        error
                    );
                    assert_eq!(
                        fs::read(root.join("proof-witnesses/checked.json")).unwrap(),
                        fs::read(
                            fixture
                                .root
                                .join("oracle-artifact/proof-witnesses/checked.json")
                        )
                        .unwrap()
                    );
                    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
                } else {
                    let (actual, _) = inputs
                        .project_native_theorem_artifact_draft(
                            six,
                            owner,
                            id,
                            (profile.clone(), ConcreteFormat::Tptp),
                            (&policy, &verifier),
                            ("proof-witnesses/checked.json", &root),
                        )
                        .unwrap()
                        .unwrap();
                    assert_eq!(&actual, expected.as_ref().unwrap().as_ref().unwrap());
                    let file_before = fs::read(root.join("proof-witnesses/checked.json")).unwrap();
                    let root_error = fs::create_dir(&root).unwrap_err().to_string();
                    assert_eq!(
                        inputs
                            .project_native_theorem_artifact_draft(
                                six,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", &root)
                            )
                            .unwrap_err(),
                        root_error
                    );
                    assert_eq!(
                        fs::read(root.join("proof-witnesses/checked.json")).unwrap(),
                        file_before
                    );
                    let existing_file = fixture.root.join("existing-file");
                    fs::write(&existing_file, b"keep").unwrap();
                    let file_error = fs::create_dir(&existing_file).unwrap_err().to_string();
                    assert_eq!(
                        inputs
                            .project_native_theorem_artifact_draft(
                                six,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", &existing_file)
                            )
                            .unwrap_err(),
                        file_error
                    );
                    assert_eq!(fs::read(&existing_file).unwrap(), b"keep");
                    #[cfg(unix)]
                    {
                        let link = fixture.root.join("existing-symlink");
                        std::os::unix::fs::symlink(&root, &link).unwrap();
                        let error = fs::create_dir(&link).unwrap_err().to_string();
                        assert_eq!(
                            inputs
                                .project_native_theorem_artifact_draft(
                                    six,
                                    owner,
                                    id,
                                    (profile.clone(), ConcreteFormat::Tptp),
                                    (&policy, &verifier),
                                    ("proof-witnesses/checked.json", &link)
                                )
                                .unwrap_err(),
                            error
                        );
                        assert_eq!(fs::read_link(&link).unwrap(), root);
                    }
                    let absent_root = fixture.root.join("must-not-create");
                    for extra in [false, true] {
                        let mut parents = six.parent_outputs().to_vec();
                        if extra {
                            let frontend = refs
                                .iter()
                                .find(|row| row.output_kind() == &OutputKind::new("FrontendOutput"))
                                .unwrap();
                            parents.push(
                                SealedParentOutputHandle::from_published_output(
                                    owner,
                                    six.snapshot,
                                    frontend.clone(),
                                )
                                .unwrap(),
                            );
                        } else {
                            parents.pop();
                        }
                        let wrong = PhaseInput::new(
                            six.work_unit.clone(),
                            PhaseDispatchInputBundle::new(
                                six.snapshot,
                                six.identities().input_hash(),
                                six.identities().dependency_hashes().to_vec(),
                                parents,
                            )
                            .unwrap(),
                        );
                        assert!(
                            inputs
                                .project_native_theorem_artifact_draft(
                                    &wrong,
                                    owner,
                                    id,
                                    (profile.clone(), ConcreteFormat::Tptp),
                                    (&policy, &verifier),
                                    ("proof-witnesses/checked.json", &absent_root)
                                )
                                .is_err()
                        );
                    }
                    let alien = publisher(six.snapshot, usize::MAX, &six.work_unit);
                    assert!(
                        inputs
                            .project_native_theorem_artifact_draft(
                                six,
                                &alien,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", &absent_root)
                            )
                            .is_err()
                    );
                    let mut bad_lock = inputs.build_plan.clone();
                    bad_lock.lockfile.packages[0].name.push_str("-changed");
                    let changed = SourceLoadInputs {
                        build_plan: &bad_lock,
                        ..inputs
                    };
                    let error = changed.validate_native_lock_content().unwrap_err();
                    assert_eq!(
                        changed
                            .project_native_theorem_artifact_draft(
                                six,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", &absent_root)
                            )
                            .unwrap_err(),
                        error
                    );
                    let mut bad_config = inputs.build_plan.clone();
                    bad_config.build_config.packages[0].config.incremental =
                        !bad_config.build_config.packages[0].config.incremental;
                    let changed = SourceLoadInputs {
                        build_plan: &bad_config,
                        ..inputs
                    };
                    let error = changed.validate_native_config_content().unwrap_err();
                    assert_eq!(
                        changed
                            .project_native_theorem_artifact_draft(
                                six,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", &absent_root)
                            )
                            .unwrap_err(),
                        error
                    );
                    let disabled = verifier.clone().with_kernel_evidence_formats([]);
                    let four_parents = six
                        .parent_outputs()
                        .iter()
                        .filter(|row| {
                            matches!(
                                row.as_output_ref().output_kind().as_str(),
                                "SourceTheoremCore"
                                    | "SourceTheoremVC"
                                    | "SourceTheoremDischarge"
                                    | "SourceTheoremCandidateCheck"
                            )
                        })
                        .cloned()
                        .collect();
                    let four = PhaseInput::new(
                        six.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            six.snapshot,
                            six.identities().input_hash(),
                            six.identities().dependency_hashes().to_vec(),
                            four_parents,
                        )
                        .unwrap(),
                    );
                    let error = inputs
                        .native_theorem_candidate_selection_with_core_vc_discharge_check_parents(
                            &four,
                            owner,
                            id,
                            (profile.clone(), ConcreteFormat::Tptp),
                            (&policy, &disabled),
                        )
                        .unwrap_err();
                    assert_eq!(
                        inputs
                            .project_native_theorem_artifact_draft(
                                six,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &disabled),
                                ("", &root)
                            )
                            .unwrap_err(),
                        error
                    );
                    assert!(!absent_root.exists());
                    owner.mark_obsolete(six.snapshot).unwrap();
                    assert!(
                        inputs
                            .project_native_theorem_artifact_draft(
                                six,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", &absent_root)
                            )
                            .is_err()
                    );
                    assert!(!absent_root.exists());
                }
                assert_eq!(
                    (
                        six.clone(),
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone()
                    ),
                    before
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (if case == "collected" { 12 } else { 14 }, 0)
                );
            },
        );
    }
}

#[test]
fn native_theorem_artifact_main_matches_real_draft() {
    use mizar_artifact::{
        store::{
            FieldPath, PublishedArtifactPath, PublishedArtifactReadOptions, artifact_hash_domain,
            canonical_json_bytes, read_published_artifact, write_published_artifact,
        },
        verified_artifact::{
            VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifactReadOptions,
            artifact_hash_excluded_paths, read_verified_artifact, verified_artifact_json,
        },
    };
    use mizar_atp::problem::*;
    use mizar_ir::{projection::ProjectionExternalDependencyGap as Gap, storage::CollectInput};
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    for (text, threshold) in [
        (
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            usize::MAX,
        ),
        (
            ":: main transport\ntheorem Renamed: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
            0,
        ),
        (
            "private theorem Hidden: for Bound being set holds Bound = Bound proof let Local be set; thus Local  =\n    Local; end;",
            0,
        ),
    ] {
        with_native_theorem_artifact_context(
            text,
            false,
            threshold,
            "",
            |fixture, inputs, six, owner, refs, expected, _| {
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("explicit"),
                        max_steps: 64,
                    },
                };
                let verifier = if threshold == 0 {
                    VerifierPolicy::default()
                        .with_profile_id("artifact λ\0")
                        .with_checker_schema_version(7)
                } else {
                    VerifierPolicy::default()
                };
                let profile = LogicProfile::try_new(
                    "artifact-source",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let vc = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc, vc.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let before = (
                    six.clone(),
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    policy.clone(),
                    profile.clone(),
                    verifier.clone(),
                    fs::read(fixture.root.join("alpha/src/main.miz")).unwrap(),
                );
                let witness_path = "proof-witnesses/checked.json";
                let baseline_root = fixture.root.join("baseline-main");
                let expected = expected.as_ref().unwrap().as_ref().unwrap();
                let (baseline, _) = inputs
                    .project_native_theorem_artifact_draft(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (witness_path, &baseline_root),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(&baseline, expected);
                assert_eq!(
                    baseline.external_dependency_gaps,
                    [
                        Gap::DriverSession,
                        Gap::DiagnosticsIntegration,
                        Gap::ProducerProjectionPayload,
                        Gap::ArtifactPublicationToken,
                    ]
                );
                assert_eq!(
                    baseline.artifact.exports.len(),
                    usize::from(!text.starts_with("private "))
                );
                assert_eq!(
                    baseline.artifact.obligations[0].owner_origin_id.is_none(),
                    text.starts_with("private ")
                );
                assert_eq!(baseline.artifact.expressions.len(), 9);
                assert!(!baseline.artifact.expressions.is_empty());
                assert!(!baseline.artifact.obligations.is_empty());
                let main_path = PublishedArtifactPath::new("main.mizir.json").unwrap();
                let value = verified_artifact_json(&baseline.artifact).unwrap();
                let domain = artifact_hash_domain(
                    VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                    baseline.artifact.schema_version,
                );
                let excluded = artifact_hash_excluded_paths();
                assert_eq!(
                    excluded,
                    [
                        FieldPath::new(["verified_at"]).unwrap(),
                        FieldPath::new(["provenance", "cache_key"]).unwrap(),
                    ]
                );
                let expected_root = fixture.root.join("expected-main");
                fs::create_dir(&expected_root).unwrap();
                let written = write_published_artifact(
                    &expected_root,
                    &main_path,
                    &value,
                    &domain,
                    &excluded,
                )
                .unwrap();
                assert_eq!(written.path, main_path);
                assert_eq!(written.artifact_hash, domain.hash(&value, &excluded));
                let expected_read = read_published_artifact(
                    &expected_root,
                    &main_path,
                    PublishedArtifactReadOptions {
                        artifact_hash_domain: Some(&domain),
                        hash_excluded_paths: &excluded,
                        expected_artifact_hash: Some(written.artifact_hash),
                    },
                )
                .unwrap();
                let options = VerifiedArtifactReadOptions {
                    artifact_path: Some(main_path.as_str()),
                    expected_module: Some(&baseline.artifact.module),
                    expected_interface_hash: Some(baseline.artifact.interface_hash),
                    expected_implementation_hash: Some(baseline.artifact.implementation_hash),
                };
                assert_eq!(
                    read_verified_artifact(&expected_read.value, options).unwrap(),
                    baseline.artifact
                );
                let root = fixture.root.join("actual-main");
                let (actual, actual_write, _) = inputs
                    .write_native_theorem_artifact_draft(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (witness_path, main_path.as_str(), &root),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(actual, baseline);
                assert_eq!(actual_write, written);
                assert_ne!(actual_write.artifact_hash, actual.artifact.interface_hash);
                assert_ne!(
                    actual_write.artifact_hash,
                    actual.artifact.implementation_hash
                );
                let bytes = fs::read(root.join(main_path.as_str())).unwrap();
                assert_eq!(bytes, canonical_json_bytes(&value));
                assert_eq!(
                    bytes,
                    fs::read(expected_root.join(main_path.as_str())).unwrap()
                );
                let read = read_published_artifact(
                    &root,
                    &main_path,
                    PublishedArtifactReadOptions {
                        artifact_hash_domain: Some(&domain),
                        hash_excluded_paths: &excluded,
                        expected_artifact_hash: Some(actual_write.artifact_hash),
                    },
                )
                .unwrap();
                assert_eq!(read, expected_read);
                assert_eq!(
                    read_verified_artifact(&read.value, options).unwrap(),
                    actual.artifact
                );
                let [reference] = actual.artifact.proof_witnesses.as_slice() else {
                    panic!()
                };
                assert_eq!(reference.witness_path, witness_path);
                let witness = fs::read(root.join(witness_path)).unwrap();
                assert_eq!(witness, fs::read(baseline_root.join(witness_path)).unwrap());
                assert_eq!(
                    witness,
                    fs::read(fixture.root.join("oracle-artifact").join(witness_path)).unwrap()
                );
                let file_ref = refs
                    .iter()
                    .find(|row| {
                        row.output_kind() == &OutputKind::new("SourceTheoremCandidateWitnessFile")
                    })
                    .unwrap();
                let physical_file = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<CanonicalJson>(file_ref, file_ref.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                assert_eq!(witness, canonical_json_bytes(&physical_file));
                let witness_domain = artifact_hash_domain(
                    reference.witness_artifact_hash.schema_family.clone(),
                    reference.witness_artifact_hash.schema_version,
                );
                let witness_read = read_published_artifact(
                    &root,
                    &PublishedArtifactPath::new(witness_path).unwrap(),
                    PublishedArtifactReadOptions {
                        artifact_hash_domain: Some(&witness_domain),
                        expected_artifact_hash: Some(reference.witness_artifact_hash.digest),
                        ..PublishedArtifactReadOptions::default()
                    },
                )
                .unwrap();
                assert_eq!(witness_read.value, *physical_file);
                assert!(
                    !mizar_proof::witness_store::read_source_theorem_witness_file(
                        &witness_read,
                        reference
                    )
                    .unwrap()
                    .is_empty()
                );
                assert_ne!(
                    actual_write.artifact_hash,
                    reference.witness_artifact_hash.digest
                );
                let mut entries = fs::read_dir(&root)
                    .unwrap()
                    .map(|entry| entry.unwrap().file_name().into_string().unwrap())
                    .collect::<Vec<_>>();
                entries.sort();
                assert_eq!(entries, ["main.mizir.json", "proof-witnesses"]);
                assert_eq!(
                    fs::read_dir(root.join("proof-witnesses")).unwrap().count(),
                    1
                );
                for reference in refs {
                    owner.storage().validate_handle(reference).unwrap();
                    owner
                        .validate_current_output(six.snapshot, reference)
                        .unwrap();
                }
                assert_eq!(
                    (
                        six.clone(),
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone(),
                        policy,
                        profile,
                        verifier,
                        fs::read(fixture.root.join("alpha/src/main.miz")).unwrap(),
                    ),
                    before
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (14, 0)
                );
            },
        );
    }
}

#[test]
fn native_theorem_artifact_main_preserves_refusals() {
    use mizar_artifact::store::PublishedArtifactPath;
    use mizar_atp::problem::*;
    use mizar_ir::storage::CollectInput;
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for open in [false, true] {
        with_native_theorem_artifact_context(
            text,
            open,
            usize::MAX,
            "",
            |fixture, inputs, six, owner, refs, expected, _| {
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("explicit"),
                        max_steps: 64,
                    },
                };
                let verifier = VerifierPolicy::default();
                let profile = LogicProfile::try_new(
                    "artifact-source",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let vc = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc, vc.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let id = vcs.vcs()[0].id;
                let before = (
                    six.clone(),
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                );
                let baseline_root = fixture.root.join("baseline-refusals");
                let baseline = inputs
                    .project_native_theorem_artifact_draft(
                        six,
                        owner,
                        id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        ("proof-witnesses/checked.json", &baseline_root),
                    )
                    .unwrap();
                let root = fixture.root.join("existing-refusals");
                fs::create_dir(&root).unwrap();
                fs::write(root.join("sentinel"), b"keep").unwrap();
                if open {
                    assert_eq!(baseline, None);
                    assert!(!baseline_root.exists());
                    assert_eq!(
                        inputs
                            .write_native_theorem_artifact_draft(
                                six,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", "../invalid", &root),
                            )
                            .unwrap(),
                        None
                    );
                } else {
                    let (baseline, _) = baseline.unwrap();
                    assert_eq!(&baseline, expected.as_ref().unwrap().as_ref().unwrap());
                    let mut parents = six.parent_outputs().to_vec();
                    parents.pop();
                    let missing = PhaseInput::new(
                        six.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            six.snapshot,
                            six.identities().input_hash(),
                            six.identities().dependency_hashes().to_vec(),
                            parents,
                        )
                        .unwrap(),
                    );
                    let error = inputs
                        .project_native_theorem_artifact_draft(
                            &missing,
                            owner,
                            id,
                            (profile.clone(), ConcreteFormat::Tptp),
                            (&policy, &verifier),
                            ("proof-witnesses/checked.json", &baseline_root),
                        )
                        .unwrap_err();
                    assert_eq!(
                        inputs
                            .write_native_theorem_artifact_draft(
                                &missing,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", "../invalid", &root),
                            )
                            .unwrap_err(),
                        error
                    );
                    let disabled = verifier.clone().with_kernel_evidence_formats([]);
                    let error = inputs
                        .project_native_theorem_artifact_draft(
                            six,
                            owner,
                            id,
                            (profile.clone(), ConcreteFormat::Tptp),
                            (&policy, &disabled),
                            ("proof-witnesses/checked.json", &baseline_root),
                        )
                        .unwrap_err();
                    assert_eq!(
                        inputs
                            .write_native_theorem_artifact_draft(
                                six,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &disabled),
                                ("proof-witnesses/checked.json", "../invalid", &root),
                            )
                            .unwrap_err(),
                        error
                    );
                    for (index, main) in ["", "../main.mizir.json", "main.json"]
                        .into_iter()
                        .enumerate()
                    {
                        let path_root = fixture.root.join(format!("invalid-main-{index}"));
                        let expected_error =
                            if main.ends_with(".json") && !main.ends_with(".mizir.json") {
                                "native theorem artifact main requires a .mizir.json main path"
                                    .to_owned()
                            } else {
                                PublishedArtifactPath::new(main).unwrap_err().to_string()
                            };
                        assert_eq!(
                            inputs
                                .write_native_theorem_artifact_draft(
                                    six,
                                    owner,
                                    id,
                                    (profile.clone(), ConcreteFormat::Tptp),
                                    (&policy, &verifier),
                                    ("proof-witnesses/checked.json", main, &path_root),
                                )
                                .unwrap_err(),
                            expected_error
                        );
                        assert_eq!(
                            fs::read(path_root.join("proof-witnesses/checked.json")).unwrap(),
                            fs::read(baseline_root.join("proof-witnesses/checked.json")).unwrap(),
                        );
                        assert_eq!(fs::read_dir(&path_root).unwrap().count(), 1);
                        assert_eq!(
                            fs::read_dir(path_root.join("proof-witnesses"))
                                .unwrap()
                                .count(),
                            1
                        );
                    }
                }
                assert_eq!(fs::read(root.join("sentinel")).unwrap(), b"keep");
                assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
                assert_eq!(
                    (
                        six.clone(),
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone()
                    ),
                    before
                );
                for reference in refs {
                    owner.storage().validate_handle(reference).unwrap();
                    owner
                        .validate_published_output(six.snapshot, reference)
                        .unwrap();
                }
                if !open {
                    for parent in six.parent_outputs() {
                        owner
                            .validate_current_output(six.snapshot, parent.as_output_ref())
                            .unwrap();
                    }
                }
                let collected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (14, 0)
                );
            },
        );
    }
}

#[test]
fn native_theorem_artifact_main_preserves_witness_and_fresh_root() {
    use mizar_artifact::store::{
        PublishedArtifactPath, PublishedArtifactReadOptions, artifact_hash_domain,
        read_published_artifact,
    };
    use mizar_atp::problem::*;
    use mizar_ir::storage::CollectInput;
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    with_native_theorem_artifact_context(
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
        false,
        usize::MAX,
        "",
        |fixture, inputs, six, owner, refs, expected, _| {
            let policy = DischargePolicy {
                computation_limit: ComputationLimit {
                    policy: PolicyKey::new("explicit"),
                    max_steps: 64,
                },
            };
            let verifier = VerifierPolicy::default();
            let profile = LogicProfile::try_new(
                "artifact-source",
                LogicFragment::Fof,
                EqualitySupport::Supported,
                QuantifierPolicy::FirstOrder,
                SoftTypeStrategy::ScopedGuardPredicates,
                NativePropertySupport::Supported,
                [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                    .into_iter()
                    .collect(),
            )
            .unwrap();
            let vc = refs
                .iter()
                .find(|row| row.output_kind() == &OutputKind::new("SourceTheoremVC"))
                .unwrap();
            let vcs = owner
                .storage()
                .get(
                    &owner
                        .storage()
                        .typed_handle::<VcSet>(vc, vc.output_kind())
                        .unwrap(),
                )
                .unwrap();
            let id = vcs.vcs()[0].id;
            let before = (
                six.clone(),
                inputs.snapshot.clone(),
                inputs.build_plan.clone(),
                fs::read(fixture.root.join("alpha/src/main.miz")).unwrap(),
            );
            let baseline_root = fixture.root.join("baseline-fresh");
            let witness_path = "proof-witnesses/checked.json";
            let (baseline, _) = inputs
                .project_native_theorem_artifact_draft(
                    six,
                    owner,
                    id,
                    (profile.clone(), ConcreteFormat::Tptp),
                    (&policy, &verifier),
                    (witness_path, &baseline_root),
                )
                .unwrap()
                .unwrap();
            assert_eq!(&baseline, expected.as_ref().unwrap().as_ref().unwrap());
            let witness_bytes = fs::read(baseline_root.join(witness_path)).unwrap();
            for (index, (witness, main)) in [
                (
                    "proof-witnesses/checked.mizir.json",
                    "proof-witnesses/checked.mizir.json",
                ),
                (
                    "proof-witnesses/nested.mizir.json/checked.json",
                    "proof-witnesses/nested.mizir.json",
                ),
                (witness_path, "proof-witnesses/checked.json/main.mizir.json"),
            ]
            .into_iter()
            .enumerate()
            {
                let alternate_root = fixture.root.join(format!("baseline-collision-{index}"));
                let (alternate, _) = inputs
                    .project_native_theorem_artifact_draft(
                        six,
                        owner,
                        id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (witness, &alternate_root),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    alternate.external_dependency_gaps,
                    baseline.external_dependency_gaps
                );
                assert_eq!(alternate.required_outputs, baseline.required_outputs);
                assert_eq!(
                    fs::read(alternate_root.join(witness)).unwrap(),
                    witness_bytes
                );
                let reference = &alternate.artifact.proof_witnesses[0];
                assert_eq!(reference.witness_path, witness);
                assert_eq!(
                    reference.witness_artifact_hash,
                    baseline.artifact.proof_witnesses[0].witness_artifact_hash
                );
                let root = fixture.root.join(format!("collision-{index}"));
                let error = inputs
                    .write_native_theorem_artifact_draft(
                        six,
                        owner,
                        id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (witness, main, &root),
                    )
                    .unwrap_err();
                if index < 2 {
                    assert_eq!(
                        error,
                        "native theorem artifact main requires a fresh main path"
                    );
                    assert!(fs::symlink_metadata(root.join(main)).is_ok());
                } else {
                    let io = fs::symlink_metadata(root.join(main)).unwrap_err();
                    assert_ne!(io.kind(), std::io::ErrorKind::NotFound);
                    assert_eq!(error, io.to_string());
                }
                assert_eq!(fs::read(root.join(witness)).unwrap(), witness_bytes);
                let domain = artifact_hash_domain(
                    reference.witness_artifact_hash.schema_family.clone(),
                    reference.witness_artifact_hash.schema_version,
                );
                let read = read_published_artifact(
                    &root,
                    &PublishedArtifactPath::new(witness).unwrap(),
                    PublishedArtifactReadOptions {
                        artifact_hash_domain: Some(&domain),
                        expected_artifact_hash: Some(reference.witness_artifact_hash.digest),
                        ..PublishedArtifactReadOptions::default()
                    },
                )
                .unwrap();
                assert!(
                    !mizar_proof::witness_store::read_source_theorem_witness_file(&read, reference)
                        .unwrap()
                        .is_empty()
                );
                assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
                assert_eq!(
                    fs::read_dir(root.join(witness).parent().unwrap())
                        .unwrap()
                        .count(),
                    1
                );
            }
            let root = fixture.root.join("fresh-main");
            let (actual, _, _) = inputs
                .write_native_theorem_artifact_draft(
                    six,
                    owner,
                    id,
                    (profile.clone(), ConcreteFormat::Tptp),
                    (&policy, &verifier),
                    (witness_path, "main.mizir.json", &root),
                )
                .unwrap()
                .unwrap();
            assert_eq!(actual, baseline);
            let main_bytes = fs::read(root.join("main.mizir.json")).unwrap();
            let error = fs::create_dir(&root).unwrap_err().to_string();
            assert_eq!(
                inputs
                    .write_native_theorem_artifact_draft(
                        six,
                        owner,
                        id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (witness_path, "main.mizir.json", &root),
                    )
                    .unwrap_err(),
                error
            );
            assert_eq!(fs::read(root.join("main.mizir.json")).unwrap(), main_bytes);
            assert_eq!(fs::read(root.join(witness_path)).unwrap(), witness_bytes);
            assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
            let existing_file = fixture.root.join("existing-main-root-file");
            fs::write(&existing_file, b"keep").unwrap();
            let error = fs::create_dir(&existing_file).unwrap_err().to_string();
            assert_eq!(
                inputs
                    .write_native_theorem_artifact_draft(
                        six,
                        owner,
                        id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (witness_path, "main.mizir.json", &existing_file),
                    )
                    .unwrap_err(),
                error
            );
            assert_eq!(fs::read(&existing_file).unwrap(), b"keep");
            #[cfg(unix)]
            {
                let link = fixture.root.join("existing-main-root-link");
                std::os::unix::fs::symlink(&root, &link).unwrap();
                let error = fs::create_dir(&link).unwrap_err().to_string();
                assert_eq!(
                    inputs
                        .write_native_theorem_artifact_draft(
                            six,
                            owner,
                            id,
                            (profile.clone(), ConcreteFormat::Tptp),
                            (&policy, &verifier),
                            (witness_path, "main.mizir.json", &link),
                        )
                        .unwrap_err(),
                    error
                );
                assert_eq!(fs::read_link(&link).unwrap(), root);
                assert_eq!(fs::read(root.join("main.mizir.json")).unwrap(), main_bytes);
                assert_eq!(fs::read(root.join(witness_path)).unwrap(), witness_bytes);
            }
            assert_eq!(
                (
                    six.clone(),
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    fs::read(fixture.root.join("alpha/src/main.miz")).unwrap()
                ),
                before
            );
            for reference in refs {
                owner.storage().validate_handle(reference).unwrap();
                owner
                    .validate_current_output(six.snapshot, reference)
                    .unwrap();
            }
            let collected = owner.storage().collect(CollectInput {
                snapshot: six.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(
                (collected.outputs_dropped, collected.abandoned_slots_dropped),
                (14, 0)
            );
        },
    );
}

#[test]
fn native_theorem_artifact_written_types_match_source_handoffs() {
    use mizar_artifact::verified_artifact::verified_artifact_json;
    use mizar_atp::problem::*;
    use mizar_checker::{
        source_context::SourceBindingSiteRole,
        type_checker::{NormalizedTypeStatus, TypeHeadRef},
        typed_ast::{TypeEntryActual, TypeStatus, TypingState},
    };
    use mizar_ir::{projection::ProjectionExternalDependencyGap as Gap, storage::CollectInput};
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for threshold in [usize::MAX, 0] {
        with_native_theorem_artifact_context(
            text,
            false,
            threshold,
            "",
            |fixture, inputs, six, owner, refs, expected, (inference, typed, metadata, source)| {
                let types = typed.source_type().unwrap();
                let context = typed.source_context().unwrap();
                let before = (
                    inference.clone(),
                    typed.clone(),
                    metadata.clone(),
                    six.clone(),
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    fs::read(fixture.root.join("alpha/src/main.miz")).unwrap(),
                );
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("explicit"),
                        max_steps: 64,
                    },
                };
                let verifier = if threshold == 0 {
                    VerifierPolicy::default()
                        .with_profile_id("artifact λ\0")
                        .with_checker_schema_version(7)
                } else {
                    VerifierPolicy::default()
                };
                let profile = LogicProfile::try_new(
                    "artifact-source",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let vc = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc, vc.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let (actual, _) = inputs
                    .project_native_theorem_artifact_draft(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile, ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (
                            "proof-witnesses/checked.json",
                            &fixture.root.join("written-handoffs"),
                        ),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(&actual, expected.as_ref().unwrap().as_ref().unwrap());
                assert_eq!(
                    actual.external_dependency_gaps,
                    [
                        Gap::DriverSession,
                        Gap::DiagnosticsIntegration,
                        Gap::ProducerProjectionPayload,
                        Gap::ArtifactPublicationToken
                    ]
                );
                assert_eq!(
                    verified_artifact_json(&actual.artifact).unwrap(),
                    verified_artifact_json(&expected.as_ref().unwrap().as_ref().unwrap().artifact)
                        .unwrap()
                );
                assert_eq!(types.expressions().len(), 2);
                assert_eq!(types.applications().len(), context.declarations().len());
                let mut applications = std::collections::BTreeSet::new();
                let mut declarations = std::collections::BTreeSet::new();
                let mut raw_sites = std::collections::BTreeSet::new();
                let mut checked_ids = std::collections::BTreeSet::new();
                let mut expected_ids = metadata
                    .expr_metadata()
                    .iter()
                    .map(|(_, row)| row.expr.as_str().to_owned())
                    .collect::<std::collections::BTreeSet<_>>();
                let mut proof_let_roots = std::collections::BTreeSet::new();
                for (root, expression) in types.expressions().iter() {
                    assert!(raw_sites.insert(expression.site().clone()));
                    let mut participants = Vec::new();
                    for (application_id, application) in types
                        .applications()
                        .iter()
                        .filter(|(_, row)| row.root() == root)
                    {
                        assert!(applications.insert(application_id));
                        let matching = context
                            .declarations()
                            .iter()
                            .filter(|(_, row)| {
                                row.binding == application.binding()
                                    && row.source_ordinal == application.source_ordinal()
                            })
                            .collect::<Vec<_>>();
                        let [(declaration_id, declaration)] = matching.as_slice() else {
                            panic!("unique actual application/declaration join")
                        };
                        assert!(declarations.insert(*declaration_id));
                        assert_eq!(declaration.written_type_range, expression.source_range());
                        let node = typed.nodes().node(declaration.site.node()).unwrap();
                        assert_eq!(node.typing, TypingState::Successful);
                        let entry = typed.types().get(node.links.type_entry.unwrap()).unwrap();
                        assert_eq!(entry.owner, declaration.site);
                        assert_eq!(entry.status, TypeStatus::Known);
                        let TypeEntryActual::Known(id) = entry.actual else {
                            panic!("actual known declaration type")
                        };
                        let normalized = inference.normalized_types().get(id).unwrap();
                        assert_eq!(normalized.status, NormalizedTypeStatus::Known);
                        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
                        assert!(
                            normalized.args.is_empty()
                                && normalized.attributes.positive().is_empty()
                                && normalized.attributes.negative().is_empty()
                        );
                        checked_ids.insert(id);
                        participants.push(application.source_ordinal());
                        if declaration.role == SourceBindingSiteRole::ProofLet {
                            proof_let_roots.insert(root);
                        }
                    }
                    let ordinal = participants.into_iter().min().unwrap();
                    let id = format!("source.theorem.type.{ordinal}");
                    assert!(expected_ids.insert(id.clone()));
                    let rows = actual
                        .artifact
                        .expressions
                        .iter()
                        .filter(|row| row.expression_id == id)
                        .collect::<Vec<_>>();
                    let [row] = rows.as_slice() else {
                        panic!("exactly one artifact row per raw written root")
                    };
                    let raw = typed.nodes().node(expression.site().node()).unwrap();
                    let head = typed.nodes().node(expression.head_site().node()).unwrap();
                    assert_ne!(expression.site(), expression.head_site());
                    assert_eq!(raw.kind.as_str(), "TypeExpression");
                    assert_eq!(head.kind.as_str(), "TypeHead");
                    assert_eq!(raw.children, [expression.head_site().node()]);
                    assert_eq!(row.expression_kind, raw.kind.as_str());
                    assert_eq!(
                        row.source_range.start_byte,
                        u64::try_from(expression.source_range().start).unwrap()
                    );
                    assert_eq!(
                        row.source_range.end_byte,
                        u64::try_from(expression.source_range().end).unwrap()
                    );
                    assert_eq!(
                        row.rendered_surface,
                        source
                            .get(expression.source_range().start..expression.source_range().end)
                            .unwrap()
                    );
                    assert_eq!(row.inferred_type.as_deref(), Some("set"));
                    assert!(
                        row.resolved_symbol.is_none()
                            && row.inserted_coercions.is_empty()
                            && row.active_thesis.is_none()
                            && row.overload_resolution.is_none()
                    );
                }
                assert_eq!(applications.len(), types.applications().len());
                assert_eq!(declarations.len(), context.declarations().len());
                assert_eq!(raw_sites.len(), types.expressions().len());
                assert_eq!(checked_ids.len(), 1);
                assert_eq!(proof_let_roots.len(), 1);
                assert_eq!(
                    actual.artifact.expressions.len(),
                    metadata.expr_metadata().len() + raw_sites.len() + 1
                );
                assert!(expected_ids.insert(format!(
                        "source.theorem.quantified.{}",
                        expected
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .artifact
                            .exports[0]
                            .fully_qualified_name
                    )));
                assert_eq!(
                    actual
                        .artifact
                        .expressions
                        .iter()
                        .map(|row| row.expression_id.clone())
                        .collect::<std::collections::BTreeSet<_>>(),
                    expected_ids
                );
                for (_, row) in metadata.expr_metadata().iter() {
                    let projected = actual
                        .artifact
                        .expressions
                        .iter()
                        .find(|value| value.expression_id == row.expr.as_str())
                        .unwrap();
                    assert_eq!(
                        projected.expression_kind,
                        typed
                            .nodes()
                            .node(row.typed_site.node())
                            .unwrap()
                            .kind
                            .as_str()
                    );
                    assert_eq!(
                        projected.source_range.start_byte,
                        u64::try_from(row.source_range.start).unwrap()
                    );
                    assert_eq!(
                        projected.source_range.end_byte,
                        u64::try_from(row.source_range.end).unwrap()
                    );
                    assert_eq!(
                        projected.rendered_surface,
                        source
                            .get(row.source_range.start..row.source_range.end)
                            .unwrap()
                    );
                    assert_eq!(
                        projected.inferred_type.as_deref(),
                        row.final_type.map(|_| "set")
                    );
                    assert!(
                        row.overload.is_none()
                            && row.inserted_views.is_empty()
                            && row.diagnostics.is_empty()
                    );
                }
                assert!(
                    actual
                        .artifact
                        .exports
                        .iter()
                        .all(|row| row.proof_status.is_none())
                );
                for reference in refs {
                    owner.storage().validate_handle(reference).unwrap();
                    owner
                        .validate_current_output(six.snapshot, reference)
                        .unwrap();
                }
                assert_eq!(
                    (
                        inference.clone(),
                        typed.clone(),
                        metadata.clone(),
                        six.clone(),
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone(),
                        fs::read(fixture.root.join("alpha/src/main.miz")).unwrap()
                    ),
                    before
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (14, 0)
                );
            },
        );
    }
}

#[test]
fn native_theorem_artifact_written_types_preserve_raw_roots_and_main() {
    use mizar_artifact::{
        store::{
            PublishedArtifactPath, PublishedArtifactReadOptions, artifact_hash_domain,
            canonical_json_bytes, read_published_artifact,
        },
        verified_artifact::{
            VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifactReadOptions,
            artifact_hash_excluded_paths, read_verified_artifact, verified_artifact_json,
        },
    };
    use mizar_atp::problem::*;
    use mizar_checker::typed_ast::TypeEntryActual;
    use mizar_ir::{projection::ProjectionExternalDependencyGap as Gap, storage::CollectInput};
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for text in [
        original.to_owned(),
        format!(
            ":: 日本語 λ\n{}\n",
            original.replace(" X", " Bound").replace("T:", "Renamed:")
        ),
    ] {
        with_native_theorem_artifact_context(
            &text,
            false,
            0,
            "",
            |fixture, inputs, six, owner, refs, expected, (inference, typed, metadata, source)| {
                let types = typed.source_type().unwrap();
                let context = typed.source_context().unwrap();
                let roots = types
                    .expressions()
                    .iter()
                    .map(|(_, row)| row)
                    .collect::<Vec<_>>();
                assert_eq!(roots.len(), 2);
                assert_ne!(roots[0].site(), roots[1].site());
                assert_ne!(roots[0].source_range(), roots[1].source_range());
                let mut denotations = std::collections::BTreeSet::new();
                for (_, application) in types.applications().iter() {
                    let declaration = context
                        .declarations()
                        .iter()
                        .find(|(_, row)| {
                            row.binding == application.binding()
                                && row.source_ordinal == application.source_ordinal()
                        })
                        .unwrap()
                        .1;
                    let node = typed.nodes().node(declaration.site.node()).unwrap();
                    let entry = typed.types().get(node.links.type_entry.unwrap()).unwrap();
                    let TypeEntryActual::Known(id) = entry.actual else {
                        panic!("known type")
                    };
                    denotations.insert(id);
                }
                assert_eq!(denotations.len(), 1);
                let before = (
                    inference.clone(),
                    typed.clone(),
                    metadata.clone(),
                    six.clone(),
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    fs::read(fixture.root.join("alpha/src/main.miz")).unwrap(),
                );
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("explicit"),
                        max_steps: 64,
                    },
                };
                let verifier = VerifierPolicy::default()
                    .with_profile_id("artifact λ\0")
                    .with_checker_schema_version(7);
                let profile = LogicProfile::try_new(
                    "artifact-source",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let vc = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc, vc.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let expected = expected.as_ref().unwrap().as_ref().unwrap();
                let root = fixture.root.join("written-main");
                let path = PublishedArtifactPath::new("resolved/main.mizir.json").unwrap();
                let witness_path = "proof-witnesses/checked.json";
                let (actual, written, _) = inputs
                    .write_native_theorem_artifact_draft(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile, ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (witness_path, path.as_str(), &root),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(&actual, expected);
                assert_eq!(
                    actual.external_dependency_gaps,
                    [
                        Gap::DriverSession,
                        Gap::DiagnosticsIntegration,
                        Gap::ProducerProjectionPayload,
                        Gap::ArtifactPublicationToken
                    ]
                );
                let value = verified_artifact_json(&expected.artifact).unwrap();
                let domain = artifact_hash_domain(
                    VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                    expected.artifact.schema_version,
                );
                let excluded = artifact_hash_excluded_paths();
                assert_eq!(written.path, path);
                assert_eq!(written.artifact_hash, domain.hash(&value, &excluded));
                assert_eq!(
                    fs::read(root.join(path.as_str())).unwrap(),
                    canonical_json_bytes(&value)
                );
                let read = read_published_artifact(
                    &root,
                    &path,
                    PublishedArtifactReadOptions {
                        artifact_hash_domain: Some(&domain),
                        hash_excluded_paths: &excluded,
                        expected_artifact_hash: Some(written.artifact_hash),
                    },
                )
                .unwrap();
                assert_eq!(read.value, value);
                assert_eq!(
                    read_verified_artifact(
                        &read.value,
                        VerifiedArtifactReadOptions {
                            artifact_path: Some(path.as_str()),
                            expected_module: Some(&expected.artifact.module),
                            expected_interface_hash: Some(expected.artifact.interface_hash),
                            expected_implementation_hash: Some(
                                expected.artifact.implementation_hash
                            )
                        }
                    )
                    .unwrap(),
                    expected.artifact
                );
                let type_rows = actual
                    .artifact
                    .expressions
                    .iter()
                    .filter(|row| row.expression_id.starts_with("source.theorem.type."))
                    .collect::<Vec<_>>();
                assert_eq!(type_rows.len(), roots.len());
                assert_ne!(type_rows[0].expression_id, type_rows[1].expression_id);
                assert_ne!(type_rows[0].source_range, type_rows[1].source_range);
                for (id, expression) in types.expressions().iter() {
                    let ordinal = types
                        .applications()
                        .iter()
                        .filter(|(_, row)| row.root() == id)
                        .map(|(_, row)| row.source_ordinal())
                        .min()
                        .unwrap();
                    let row = type_rows
                        .iter()
                        .find(|row| row.expression_id == format!("source.theorem.type.{ordinal}"))
                        .unwrap();
                    assert_eq!(
                        row.expression_kind,
                        typed
                            .nodes()
                            .node(expression.site().node())
                            .unwrap()
                            .kind
                            .as_str()
                    );
                    assert_eq!(
                        row.source_range.start_byte,
                        u64::try_from(expression.source_range().start).unwrap()
                    );
                    assert_eq!(
                        row.source_range.end_byte,
                        u64::try_from(expression.source_range().end).unwrap()
                    );
                    assert_eq!(
                        row.rendered_surface,
                        source
                            .get(expression.source_range().start..expression.source_range().end)
                            .unwrap()
                    );
                    assert_eq!(row.inferred_type.as_deref(), Some("set"));
                    assert!(row.active_thesis.is_none());
                }
                let file_ref = refs
                    .iter()
                    .find(|row| {
                        row.output_kind() == &OutputKind::new("SourceTheoremCandidateWitnessFile")
                    })
                    .unwrap();
                let physical = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<CanonicalJson>(file_ref, file_ref.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                assert_eq!(
                    fs::read(root.join(witness_path)).unwrap(),
                    canonical_json_bytes(&physical)
                );
                assert_eq!(
                    fs::read(root.join(witness_path)).unwrap(),
                    fs::read(fixture.root.join("oracle-artifact").join(witness_path)).unwrap()
                );
                assert_eq!(actual.artifact.obligations, expected.artifact.obligations);
                assert_eq!(
                    actual.artifact.proof_witnesses,
                    expected.artifact.proof_witnesses
                );
                assert_eq!(actual.artifact.provenance, expected.artifact.provenance);
                assert!(
                    actual
                        .artifact
                        .exports
                        .iter()
                        .all(|row| row.proof_status.is_none())
                );
                for reference in refs {
                    owner.storage().validate_handle(reference).unwrap();
                    owner
                        .validate_current_output(six.snapshot, reference)
                        .unwrap();
                }
                assert_eq!(
                    (
                        inference.clone(),
                        typed.clone(),
                        metadata.clone(),
                        six.clone(),
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone(),
                        fs::read(fixture.root.join("alpha/src/main.miz")).unwrap()
                    ),
                    before
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (14, 0)
                );
            },
        );
    }
}

#[test]
fn native_theorem_artifact_written_types_preserve_profiles_and_refusals() {
    use mizar_atp::problem::*;
    use mizar_ir::storage::CollectInput;
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in ["baseline", "buffer", "bytes", "link"] {
        let fault = if matches!(case, "bytes" | "link") {
            case
        } else {
            ""
        };
        with_native_theorem_artifact_context(
            text,
            case == "buffer",
            0,
            fault,
            |fixture, inputs, six, owner, refs, expected, (inference, typed, metadata, _)| {
                let before = (
                    inference.clone(),
                    typed.clone(),
                    metadata.clone(),
                    six.clone(),
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    fs::read(fixture.root.join("alpha/src/main.miz")).unwrap(),
                );
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("explicit"),
                        max_steps: 64,
                    },
                };
                let verifier = VerifierPolicy::default()
                    .with_profile_id("artifact λ\0")
                    .with_checker_schema_version(7);
                let profile = LogicProfile::try_new(
                    "artifact-source",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let vc = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc, vc.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let root = fixture.root.join("written-profile");
                let result = inputs.project_native_theorem_artifact_draft(
                    six,
                    owner,
                    vcs.vcs()[0].id,
                    (profile.clone(), ConcreteFormat::Tptp),
                    (&policy, &verifier),
                    ("proof-witnesses/checked.json", &root),
                );
                if case == "baseline" {
                    let (actual, _) = result.unwrap().unwrap();
                    assert_eq!(&actual, expected.as_ref().unwrap().as_ref().unwrap());
                    assert_eq!(
                        actual
                            .artifact
                            .expressions
                            .iter()
                            .filter(|row| row.expression_id.starts_with("source.theorem.type."))
                            .count(),
                        typed.source_type().unwrap().expressions().len()
                    );
                    let bytes = fs::read(root.join("proof-witnesses/checked.json")).unwrap();
                    let error = fs::create_dir(&root).unwrap_err().to_string();
                    assert_eq!(
                        inputs
                            .project_native_theorem_artifact_draft(
                                six,
                                owner,
                                vcs.vcs()[0].id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", &root)
                            )
                            .unwrap_err(),
                        error
                    );
                    assert_eq!(
                        fs::read(root.join("proof-witnesses/checked.json")).unwrap(),
                        bytes
                    );
                    let absent = fixture.root.join("written-refusal-must-stay-absent");
                    let mut bad_lock = inputs.build_plan.clone();
                    bad_lock.lockfile.packages[0].name.push_str("-changed");
                    let changed = SourceLoadInputs {
                        build_plan: &bad_lock,
                        ..inputs
                    };
                    let error = changed.validate_native_lock_content().unwrap_err();
                    assert_eq!(
                        changed
                            .project_native_theorem_artifact_draft(
                                six,
                                owner,
                                vcs.vcs()[0].id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", &absent)
                            )
                            .unwrap_err(),
                        error
                    );
                    let mut bad_config = inputs.build_plan.clone();
                    bad_config.build_config.packages[0].config.incremental =
                        !bad_config.build_config.packages[0].config.incremental;
                    let changed = SourceLoadInputs {
                        build_plan: &bad_config,
                        ..inputs
                    };
                    let error = changed.validate_native_config_content().unwrap_err();
                    assert_eq!(
                        changed
                            .project_native_theorem_artifact_draft(
                                six,
                                owner,
                                vcs.vcs()[0].id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", &absent)
                            )
                            .unwrap_err(),
                        error
                    );
                    assert!(!absent.exists());
                } else if case == "buffer" {
                    assert!(result.unwrap().is_none());
                    assert!(!root.exists());
                    assert!(
                        inputs
                            .write_native_theorem_artifact_draft(
                                six,
                                owner,
                                vcs.vcs()[0].id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", "main.json", &root)
                            )
                            .unwrap()
                            .is_none()
                    );
                    assert!(!root.exists());
                } else {
                    assert!(expected.is_none());
                    let error = result.unwrap_err();
                    assert!(!root.exists());
                    assert_eq!(
                        inputs
                            .write_native_theorem_artifact_draft(
                                six,
                                owner,
                                vcs.vcs()[0].id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", "main.json", &root)
                            )
                            .unwrap_err(),
                        error
                    );
                    assert!(!root.exists());
                }
                for reference in refs {
                    owner.storage().validate_handle(reference).unwrap();
                    owner
                        .validate_published_output(six.snapshot, reference)
                        .unwrap();
                }
                if case != "buffer" {
                    assert_eq!(six.parent_outputs().len(), 6);
                    for parent in six.parent_outputs() {
                        owner
                            .validate_current_output(six.snapshot, parent.as_output_ref())
                            .unwrap();
                    }
                }
                assert_eq!(
                    (
                        inference.clone(),
                        typed.clone(),
                        metadata.clone(),
                        six.clone(),
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone(),
                        fs::read(fixture.root.join("alpha/src/main.miz")).unwrap()
                    ),
                    before
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (14, 0)
                );
            },
        );
    }
}

#[test]
fn native_theorem_artifact_statements_match_checked_formula() {
    use mizar_artifact::{
        store::SchemaVersion,
        verified_artifact::{
            VerifiedArtifactReadOptions, read_verified_artifact, verified_artifact_json,
        },
    };
    use mizar_atp::problem::*;
    use mizar_ir::{projection::ProjectionExternalDependencyGap as Gap, storage::CollectInput};
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    for (text, formula, thesis, threshold) in [
        (
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            "for X being set holds X = X",
            "X = X",
            usize::MAX,
        ),
        (
            ":: 日本語 λ\ntheorem Renamed: for Bound being set holds Bound = Bound proof let Bound be set; thus Bound = Bound; end;\n",
            "for Bound being set holds Bound = Bound",
            "Bound = Bound",
            0,
        ),
        (
            "theorem Spaced:\n  for X being set\n    holds X = X\nproof\n  let X be set;\n  thus X = X;\nend;",
            "for X being set\n    holds X = X",
            "X = X",
            usize::MAX,
        ),
        (
            "theorem LocalRename: for Bound being set holds Bound = Bound proof let Local be set; thus Local  =\n    Local; end;",
            "for Bound being set holds Bound = Bound",
            "Local = Local",
            0,
        ),
        (
            "private theorem Hidden: for Bound being set holds Bound = Bound proof let Local be set; thus Local  =\n    Local; end;",
            "for Bound being set holds Bound = Bound",
            "Local = Local",
            0,
        ),
    ] {
        with_native_theorem_artifact_context(
            text,
            false,
            threshold,
            "",
            |fixture, inputs, six, owner, refs, expected, (inference, typed, metadata, source)| {
                let before = (
                    inference.clone(),
                    typed.clone(),
                    metadata.clone(),
                    six.clone(),
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    fs::read(fixture.root.join("alpha/src/main.miz")).unwrap(),
                );
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("explicit"),
                        max_steps: 64,
                    },
                };
                let verifier = if threshold == 0 {
                    VerifierPolicy::default()
                        .with_profile_id("artifact λ\0")
                        .with_checker_schema_version(7)
                } else {
                    VerifierPolicy::default()
                };
                let profile = LogicProfile::try_new(
                    "artifact-source",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let vc = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc, vc.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let root = fixture.root.join("statement-formula");
                let expected = expected.as_ref().unwrap().as_ref().unwrap();
                let (actual, _) = inputs
                    .project_native_theorem_artifact_draft(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        ("proof-witnesses/checked.json", &root),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    actual.artifact.exports.len(),
                    usize::from(!text.starts_with("private "))
                );
                assert_eq!(
                    actual.artifact.obligations[0].owner_origin_id.is_none(),
                    text.starts_with("private ")
                );
                assert_eq!(actual.artifact.expressions.len(), 9);
                let source_items = typed
                    .source_context()
                    .unwrap()
                    .items()
                    .iter()
                    .collect::<Vec<_>>();
                let [(_, source_item)] = source_items.as_slice() else {
                    panic!("one actual source owner")
                };
                let quantified_id = format!(
                    "source.theorem.quantified.{}",
                    expected.artifact.statements[0].item
                );
                assert_eq!(
                    actual.artifact.expressions.len(),
                    metadata.expr_metadata().len()
                        + typed.source_type().unwrap().expressions().len()
                        + 1,
                    "missing {quantified_id}"
                );
                let expected_theses = expected
                    .artifact
                    .expressions
                    .iter()
                    .filter_map(|row| {
                        row.active_thesis
                            .as_deref()
                            .map(|goal| (row.expression_id.as_str(), goal))
                    })
                    .collect::<std::collections::BTreeMap<_, _>>();
                assert_eq!(expected_theses.len(), 3);
                let actual_theses = actual
                    .artifact
                    .expressions
                    .iter()
                    .filter_map(|row| {
                        row.active_thesis
                            .as_deref()
                            .map(|goal| (row.expression_id.as_str(), goal))
                    })
                    .collect::<std::collections::BTreeMap<_, _>>();
                assert_eq!(
                    actual_theses.len(),
                    3,
                    "missing terminal thesis sites: {expected_theses:?}"
                );
                assert_eq!(actual_theses, expected_theses);
                assert_eq!(&actual, expected);
                let quantified = actual
                    .artifact
                    .expressions
                    .iter()
                    .find(|row| row.expression_id == quantified_id)
                    .unwrap();
                assert_eq!(quantified.expression_kind, "QuantifiedFormula(Universal)");
                assert_eq!(quantified.rendered_surface, formula);
                let quantified_start = source.find(formula).unwrap();
                assert_eq!(
                    (
                        quantified.source_range.start_byte,
                        quantified.source_range.end_byte
                    ),
                    (
                        u64::try_from(quantified_start).unwrap(),
                        u64::try_from(quantified_start + formula.len()).unwrap()
                    )
                );
                assert!(
                    quantified.inferred_type.is_none()
                        && quantified.resolved_symbol.is_none()
                        && quantified.active_thesis.is_none()
                        && quantified.overload_resolution.is_none()
                        && quantified.inserted_coercions.is_empty()
                );

                assert_eq!(actual.artifact.schema_version, SchemaVersion::new(1, 1));
                let [statement] = actual.artifact.statements.as_slice() else {
                    panic!("one checked theorem statement")
                };
                assert_eq!(statement.elaborated, formula);
                assert_eq!(statement.item, expected.artifact.statements[0].item);
                let start = source.find(formula).unwrap();
                let end = start + formula.len();
                assert_eq!(
                    statement.source_range.start_byte,
                    u64::try_from(start).unwrap()
                );
                assert_eq!(statement.source_range.end_byte, u64::try_from(end).unwrap());
                assert_eq!(source.get(start..end), Some(formula));
                assert!(start > source_item.source_range.start);
                assert!(end < source_item.source_range.end);
                assert!(
                    statement.reserve_binders.is_empty() && statement.resolved_symbols.is_empty()
                );
                let expected_statement = CanonicalJson::object([
                    (
                        "item",
                        CanonicalJson::string(&expected.artifact.statements[0].item),
                    ),
                    (
                        "source_range",
                        CanonicalJson::object([
                            (
                                "start_byte",
                                CanonicalJson::Integer(i64::try_from(start).unwrap()),
                            ),
                            (
                                "end_byte",
                                CanonicalJson::Integer(i64::try_from(end).unwrap()),
                            ),
                        ])
                        .unwrap(),
                    ),
                    ("elaborated", CanonicalJson::string(formula)),
                    ("reserve_binders", CanonicalJson::Array(vec![])),
                    (
                        "resolved_symbols",
                        CanonicalJson::object(Vec::<(String, CanonicalJson)>::new()).unwrap(),
                    ),
                ])
                .unwrap();
                let json = verified_artifact_json(&actual.artifact).unwrap();
                let CanonicalJson::Object(fields) = &json else {
                    panic!("normal artifact object")
                };
                assert_eq!(
                    fields.get("statements"),
                    Some(&CanonicalJson::Array(vec![expected_statement]))
                );
                assert_eq!(
                    read_verified_artifact(&json, VerifiedArtifactReadOptions::default()).unwrap(),
                    actual.artifact
                );
                assert_eq!(
                    actual.external_dependency_gaps,
                    [
                        Gap::DriverSession,
                        Gap::DiagnosticsIntegration,
                        Gap::ProducerProjectionPayload,
                        Gap::ArtifactPublicationToken
                    ]
                );
                assert!(actual.artifact.expressions.iter().all(|row| {
                    row.active_thesis
                        .as_deref()
                        .is_none_or(|goal| goal == thesis)
                }));
                assert_eq!(
                    actual.artifact.source_hash,
                    inputs.snapshot.source_versions[0].source_hash
                );
                assert_eq!(
                    fs::read(root.join("proof-witnesses/checked.json")).unwrap(),
                    fs::read(
                        fixture
                            .root
                            .join("oracle-artifact/proof-witnesses/checked.json")
                    )
                    .unwrap()
                );
                assert_eq!(refs.len(), 13);
                for reference in refs {
                    owner.storage().validate_handle(reference).unwrap();
                    owner
                        .validate_current_output(six.snapshot, reference)
                        .unwrap();
                }
                assert_eq!(
                    (
                        inference.clone(),
                        typed.clone(),
                        metadata.clone(),
                        six.clone(),
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone(),
                        fs::read(fixture.root.join("alpha/src/main.miz")).unwrap()
                    ),
                    before
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (14, 0)
                );
            },
        );
    }
}

#[test]
fn native_theorem_artifact_statements_preserve_main_and_identity() {
    use mizar_artifact::{
        store::{
            PublishedArtifactPath, PublishedArtifactReadOptions, artifact_hash_domain,
            canonical_json_bytes, read_published_artifact,
        },
        verified_artifact::{
            VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifactReadOptions,
            artifact_hash_excluded_paths, read_verified_artifact, verified_artifact_json,
        },
    };
    use mizar_atp::problem::*;
    use mizar_ir::storage::CollectInput;
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    let text = ":: statement transport λ\ntheorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for threshold in [usize::MAX, 0] {
        with_native_theorem_artifact_context(
            text,
            false,
            threshold,
            "",
            |fixture, inputs, six, owner, refs, expected, (inference, typed, metadata, source)| {
                let before = (
                    inference.clone(),
                    typed.clone(),
                    metadata.clone(),
                    six.clone(),
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    fs::read(fixture.root.join("alpha/src/main.miz")).unwrap(),
                );
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("explicit"),
                        max_steps: 64,
                    },
                };
                let verifier = if threshold == 0 {
                    VerifierPolicy::default()
                        .with_profile_id("artifact λ\0")
                        .with_checker_schema_version(7)
                } else {
                    VerifierPolicy::default()
                };
                let profile = LogicProfile::try_new(
                    "artifact-source",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let vc = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc, vc.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let expected = expected.as_ref().unwrap().as_ref().unwrap();
                let root = fixture.root.join("statement-main");
                let witness_path = "proof-witnesses/checked.json";
                let main_path = PublishedArtifactPath::new("main.mizir.json").unwrap();
                let (actual, written, _) = inputs
                    .write_native_theorem_artifact_draft(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (witness_path, main_path.as_str(), &root),
                    )
                    .unwrap()
                    .unwrap();
                let quantified_id = format!(
                    "source.theorem.quantified.{}",
                    expected.artifact.exports[0].fully_qualified_name
                );
                assert_eq!(
                    actual.artifact.expressions.len(),
                    metadata.expr_metadata().len()
                        + typed.source_type().unwrap().expressions().len()
                        + 1,
                    "missing {quantified_id}"
                );
                let expected_theses = expected
                    .artifact
                    .expressions
                    .iter()
                    .filter_map(|row| {
                        row.active_thesis
                            .as_deref()
                            .map(|goal| (row.expression_id.as_str(), goal))
                    })
                    .collect::<std::collections::BTreeMap<_, _>>();
                assert_eq!(expected_theses.len(), 3);
                let actual_theses = actual
                    .artifact
                    .expressions
                    .iter()
                    .filter_map(|row| {
                        row.active_thesis
                            .as_deref()
                            .map(|goal| (row.expression_id.as_str(), goal))
                    })
                    .collect::<std::collections::BTreeMap<_, _>>();
                assert_eq!(
                    actual_theses.len(),
                    3,
                    "missing terminal thesis sites: {expected_theses:?}"
                );
                assert_eq!(actual_theses, expected_theses);
                assert_eq!(&actual, expected);
                let json = verified_artifact_json(&expected.artifact).unwrap();
                let domain = artifact_hash_domain(
                    VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                    expected.artifact.schema_version,
                );
                let exclusions = artifact_hash_excluded_paths();
                assert_eq!(written.artifact_hash, domain.hash(&json, &exclusions));
                assert_eq!(
                    fs::read(root.join(main_path.as_str())).unwrap(),
                    canonical_json_bytes(&json)
                );
                let read = read_published_artifact(
                    &root,
                    &main_path,
                    PublishedArtifactReadOptions {
                        artifact_hash_domain: Some(&domain),
                        hash_excluded_paths: &exclusions,
                        expected_artifact_hash: Some(written.artifact_hash),
                    },
                )
                .unwrap();
                assert_eq!(read.value, json);
                let artifact = read_verified_artifact(
                    &read.value,
                    VerifiedArtifactReadOptions {
                        artifact_path: Some(main_path.as_str()),
                        expected_module: Some(&expected.artifact.module),
                        expected_interface_hash: Some(expected.artifact.interface_hash),
                        expected_implementation_hash: Some(expected.artifact.implementation_hash),
                    },
                )
                .unwrap();
                assert_eq!(artifact, expected.artifact);
                assert_eq!(
                    artifact
                        .expressions
                        .iter()
                        .filter_map(|row| row.active_thesis.as_deref())
                        .collect::<Vec<_>>(),
                    ["X = X"; 3]
                );
                let quantified = artifact
                    .expressions
                    .iter()
                    .find(|row| row.expression_id == quantified_id)
                    .unwrap();
                assert_eq!(quantified.expression_kind, "QuantifiedFormula(Universal)");
                assert_eq!(quantified.rendered_surface, "for X being set holds X = X");
                let quantified_start = source.find("for X being set holds X = X").unwrap();
                assert_eq!(
                    (
                        quantified.source_range.start_byte,
                        quantified.source_range.end_byte
                    ),
                    (
                        u64::try_from(quantified_start).unwrap(),
                        u64::try_from(quantified_start + "for X being set holds X = X".len())
                            .unwrap()
                    )
                );
                assert!(
                    quantified.inferred_type.is_none()
                        && quantified.resolved_symbol.is_none()
                        && quantified.active_thesis.is_none()
                        && quantified.overload_resolution.is_none()
                        && quantified.inserted_coercions.is_empty()
                );

                assert_eq!(artifact.statements.len(), 1);
                let statement = &artifact.statements[0];
                assert_eq!(statement.elaborated, "for X being set holds X = X");
                assert_eq!(
                    source.get(
                        usize::try_from(statement.source_range.start_byte).unwrap()
                            ..usize::try_from(statement.source_range.end_byte).unwrap()
                    ),
                    Some(statement.elaborated.as_str())
                );
                assert!(
                    artifact
                        .exports
                        .iter()
                        .all(|row| row.proof_status.is_none())
                );
                let [witness] = artifact.proof_witnesses.as_slice() else {
                    panic!("one authentic witness")
                };
                assert_eq!(
                    artifact.obligations[0]
                        .accepted_witness_obligation_id
                        .as_deref(),
                    Some(artifact.obligations[0].obligation_id.as_str())
                );
                let file_ref = refs
                    .iter()
                    .find(|row| {
                        row.output_kind() == &OutputKind::new("SourceTheoremCandidateWitnessFile")
                    })
                    .unwrap();
                let file = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<CanonicalJson>(file_ref, file_ref.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let witness_bytes = fs::read(root.join(witness_path)).unwrap();
                assert_eq!(witness_bytes, canonical_json_bytes(&file));
                assert_eq!(
                    witness_bytes,
                    fs::read(fixture.root.join("oracle-artifact").join(witness_path)).unwrap()
                );
                let witness_domain = artifact_hash_domain(
                    witness.witness_artifact_hash.schema_family.clone(),
                    witness.witness_artifact_hash.schema_version,
                );
                let witness_read = read_published_artifact(
                    &root,
                    &PublishedArtifactPath::new(witness_path).unwrap(),
                    PublishedArtifactReadOptions {
                        artifact_hash_domain: Some(&witness_domain),
                        expected_artifact_hash: Some(witness.witness_artifact_hash.digest),
                        ..PublishedArtifactReadOptions::default()
                    },
                )
                .unwrap();
                assert_eq!(witness_read.value, *file);
                assert!(
                    !mizar_proof::witness_store::read_source_theorem_witness_file(
                        &witness_read,
                        witness
                    )
                    .unwrap()
                    .is_empty()
                );
                assert_ne!(written.artifact_hash, witness.witness_artifact_hash.digest);
                assert_ne!(written.artifact_hash, artifact.interface_hash);
                assert_ne!(written.artifact_hash, artifact.implementation_hash);
                assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
                assert_eq!(
                    fs::read_dir(root.join("proof-witnesses")).unwrap().count(),
                    1
                );
                let error = fs::create_dir(&root).unwrap_err().to_string();
                assert_eq!(
                    inputs
                        .write_native_theorem_artifact_draft(
                            six,
                            owner,
                            vcs.vcs()[0].id,
                            (profile, ConcreteFormat::Tptp),
                            (&policy, &verifier),
                            (witness_path, main_path.as_str(), &root),
                        )
                        .unwrap_err(),
                    error
                );
                assert_eq!(
                    fs::read(root.join(main_path.as_str())).unwrap(),
                    canonical_json_bytes(&json)
                );
                assert_eq!(fs::read(root.join(witness_path)).unwrap(), witness_bytes);
                for reference in refs {
                    owner.storage().validate_handle(reference).unwrap();
                    owner
                        .validate_current_output(six.snapshot, reference)
                        .unwrap();
                }
                assert_eq!(
                    (
                        inference.clone(),
                        typed.clone(),
                        metadata.clone(),
                        six.clone(),
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone(),
                        fs::read(fixture.root.join("alpha/src/main.miz")).unwrap()
                    ),
                    before
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (14, 0)
                );
            },
        );
    }
}

#[test]
fn native_theorem_artifact_statements_preserve_refusals() {
    use mizar_atp::problem::*;
    use mizar_ir::storage::CollectInput;
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in ["baseline", "buffer", "bytes", "link"] {
        let fault = if matches!(case, "bytes" | "link") {
            case
        } else {
            ""
        };
        with_native_theorem_artifact_context(
            text,
            case == "buffer",
            0,
            fault,
            |fixture, inputs, six, owner, refs, expected, (inference, typed, metadata, source)| {
                let before = (
                    inference.clone(),
                    typed.clone(),
                    metadata.clone(),
                    six.clone(),
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    fs::read(fixture.root.join("alpha/src/main.miz")).unwrap(),
                );
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("explicit"),
                        max_steps: 64,
                    },
                };
                let verifier = VerifierPolicy::default()
                    .with_profile_id("artifact λ\0")
                    .with_checker_schema_version(7);
                let profile = LogicProfile::try_new(
                    "artifact-source",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let vc = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc, vc.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let id = vcs.vcs()[0].id;
                let root = fixture.root.join("statement-refusal");
                let result = inputs.project_native_theorem_artifact_draft(
                    six,
                    owner,
                    id,
                    (profile.clone(), ConcreteFormat::Tptp),
                    (&policy, &verifier),
                    ("proof-witnesses/checked.json", &root),
                );
                if case == "baseline" {
                    let (actual, _) = result.unwrap().unwrap();
                    let expected = expected.as_ref().unwrap().as_ref().unwrap();
                    let quantified_id = format!(
                        "source.theorem.quantified.{}",
                        expected.artifact.exports[0].fully_qualified_name
                    );
                    assert_eq!(
                        actual.artifact.expressions.len(),
                        metadata.expr_metadata().len()
                            + typed.source_type().unwrap().expressions().len()
                            + 1,
                        "missing {quantified_id}"
                    );
                    let expected_theses = expected
                        .artifact
                        .expressions
                        .iter()
                        .filter_map(|row| {
                            row.active_thesis
                                .as_deref()
                                .map(|goal| (row.expression_id.as_str(), goal))
                        })
                        .collect::<std::collections::BTreeMap<_, _>>();
                    assert_eq!(expected_theses.len(), 3);
                    let actual_theses = actual
                        .artifact
                        .expressions
                        .iter()
                        .filter_map(|row| {
                            row.active_thesis
                                .as_deref()
                                .map(|goal| (row.expression_id.as_str(), goal))
                        })
                        .collect::<std::collections::BTreeMap<_, _>>();
                    assert_eq!(
                        actual_theses.len(),
                        3,
                        "missing terminal thesis sites: {expected_theses:?}"
                    );
                    assert_eq!(actual_theses, expected_theses);
                    assert_eq!(&actual, expected);
                    assert_eq!(
                        actual
                            .artifact
                            .expressions
                            .iter()
                            .filter_map(|row| row.active_thesis.as_deref())
                            .collect::<Vec<_>>(),
                        ["X = X"; 3]
                    );
                    let quantified = actual
                        .artifact
                        .expressions
                        .iter()
                        .find(|row| row.expression_id == quantified_id)
                        .unwrap();
                    assert_eq!(quantified.expression_kind, "QuantifiedFormula(Universal)");
                    assert_eq!(quantified.rendered_surface, "for X being set holds X = X");
                    let quantified_start = source.find("for X being set holds X = X").unwrap();
                    assert_eq!(
                        (
                            quantified.source_range.start_byte,
                            quantified.source_range.end_byte
                        ),
                        (
                            u64::try_from(quantified_start).unwrap(),
                            u64::try_from(quantified_start + "for X being set holds X = X".len())
                                .unwrap()
                        )
                    );
                    assert!(
                        quantified.inferred_type.is_none()
                            && quantified.resolved_symbol.is_none()
                            && quantified.active_thesis.is_none()
                            && quantified.overload_resolution.is_none()
                            && quantified.inserted_coercions.is_empty()
                    );

                    assert_eq!(actual.artifact.statements.len(), 1);
                    let four = PhaseInput::new(
                        six.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            six.snapshot,
                            six.identities().input_hash(),
                            six.identities().dependency_hashes().to_vec(),
                            six.parent_outputs()
                                .iter()
                                .filter(|parent| {
                                    matches!(
                                        parent.as_output_ref().output_kind().as_str(),
                                        "SourceTheoremCore"
                                            | "SourceTheoremVC"
                                            | "SourceTheoremDischarge"
                                            | "SourceTheoremCandidateCheck"
                                    )
                                })
                                .cloned()
                                .collect(),
                        )
                        .unwrap(),
                    );
                    let wrong = verifier.clone().with_kernel_evidence_formats([]);
                    let lower = inputs
                        .native_theorem_candidate_witness_file_with_core_vc_discharge_check_parents(
                            &four,
                            owner,
                            id,
                            (profile.clone(), ConcreteFormat::Tptp),
                            (&policy, &wrong),
                            "proof-witnesses/checked.json",
                        )
                        .map(|value| value.is_some());
                    assert_ne!(lower, Ok(true));
                    let absent = fixture.root.join("statement-policy");
                    assert_eq!(
                        inputs
                            .project_native_theorem_artifact_draft(
                                six,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &wrong),
                                ("proof-witnesses/checked.json", &absent),
                            )
                            .map(|value| value.is_some()),
                        lower
                    );
                    assert_eq!(
                        inputs
                            .write_native_theorem_artifact_draft(
                                six,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &wrong),
                                ("proof-witnesses/checked.json", "main.mizir.json", &absent),
                            )
                            .map(|value| value.is_some()),
                        lower
                    );
                    assert!(!absent.exists());
                    for lock in [true, false] {
                        let mut changed_plan = inputs.build_plan.clone();
                        if lock {
                            changed_plan.lockfile.packages[0].name.push_str("-changed");
                        } else {
                            changed_plan.build_config.packages[0].config.incremental =
                                !changed_plan.build_config.packages[0].config.incremental;
                        }
                        let changed = SourceLoadInputs {
                            build_plan: &changed_plan,
                            ..inputs
                        };
                        let lower = if lock {
                            changed.validate_native_lock_content()
                        } else {
                            changed.validate_native_config_content()
                        }
                        .unwrap_err();
                        let absent = fixture.root.join(format!("statement-config-{lock}"));
                        assert_eq!(
                            changed
                                .write_native_theorem_artifact_draft(
                                    six,
                                    owner,
                                    id,
                                    (profile.clone(), ConcreteFormat::Tptp),
                                    (&policy, &verifier),
                                    ("proof-witnesses/checked.json", "main.mizir.json", &absent),
                                )
                                .unwrap_err(),
                            lower
                        );
                        assert!(!absent.exists());
                    }
                } else {
                    let lower = result.map(|value| value.is_some());
                    if case == "buffer" {
                        assert_eq!(lower, Ok(false));
                    } else {
                        assert!(expected.is_none());
                        assert!(lower.is_err());
                    }
                    assert_eq!(
                        inputs
                            .write_native_theorem_artifact_draft(
                                six,
                                owner,
                                id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                ("proof-witnesses/checked.json", "main.mizir.json", &root),
                            )
                            .map(|value| value.is_some()),
                        lower
                    );
                    assert!(!root.exists());
                }
                for reference in refs {
                    owner.storage().validate_handle(reference).unwrap();
                    owner
                        .validate_published_output(six.snapshot, reference)
                        .unwrap();
                }
                if case != "buffer" {
                    for parent in six.parent_outputs() {
                        owner
                            .validate_current_output(six.snapshot, parent.as_output_ref())
                            .unwrap();
                    }
                }
                assert_eq!(
                    (
                        inference.clone(),
                        typed.clone(),
                        metadata.clone(),
                        six.clone(),
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone(),
                        fs::read(fixture.root.join("alpha/src/main.miz")).unwrap()
                    ),
                    before
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (14, 0)
                );
            },
        );
    }
}

#[test]
fn native_theorem_artifact_manifest_matches_real_transaction() {
    use mizar_artifact::{
        manifest::{
            ArtifactManifest, ManifestCommitOptions, ManifestFileReadOptions,
            ManifestProofWitnessEntry, ManifestProvenance, ManifestTransaction,
            ModuleArtifactEntry, PackageIdentity, artifact_manifest_json, artifact_manifest_path,
            current_schema_version as manifest_schema, manifest_hash_domain, read_manifest_file,
            write_artifact_manifest,
        },
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{
            PublishedArtifactReadOptions, artifact_hash_domain, canonical_json_bytes,
            read_published_artifact,
        },
        verified_artifact::{
            VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifactReadOptions,
            artifact_hash_excluded_paths, read_verified_artifact, verified_artifact_json,
        },
    };
    use mizar_atp::problem::*;
    use mizar_ir::{
        identity::NamedInputHash,
        projection::ProjectionExternalDependencyGap as Gap,
        storage::{CollectInput, IrSideTables, SchemaVersion},
    };
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    for (text, threshold) in [
        (
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            usize::MAX,
        ),
        (
            "private theorem Hidden: for Bound being set holds Bound = Bound proof let Local be set; thus Local  =\n    Local; end;",
            0,
        ),
    ] {
        with_native_theorem_artifact_context(
            text,
            false,
            threshold,
            "",
            |fixture, inputs, six, owner, refs, expected, (inference, typed, metadata, source)| {
                let mut original_outputs = refs
                    .iter()
                    .map(|reference| owner.registry().output_lineage(reference.output()).unwrap())
                    .collect::<Vec<_>>();
                let kernel = refs
                    .iter()
                    .find(|reference| {
                        reference.output_kind() == &OutputKind::new("SourceTheoremCandidateCheck")
                    })
                    .unwrap();
                let unlisted = kernel
                    .lineage()
                    .parents
                    .iter()
                    .filter(|parent| {
                        original_outputs
                            .iter()
                            .all(|output| output.output != **parent)
                    })
                    .collect::<Vec<_>>();
                let [candidate] = unlisted.as_slice() else {
                    panic!("one genuine inline ATP candidate parent")
                };
                let candidate = owner.registry().output_lineage(**candidate).unwrap();
                assert_eq!(candidate.phase, IrPipelinePhase::new("AtpSolve"));
                assert_eq!(
                    candidate.output_kind,
                    OutputKind::new("SourceTheoremCandidateInputs")
                );
                original_outputs.push(candidate);
                assert_eq!(original_outputs.len(), 14);
                let protected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: original_outputs
                        .iter()
                        .map(|output| output.output)
                        .collect(),
                });
                assert_eq!(
                    (
                        protected.outputs_dropped,
                        protected.protected_outputs,
                        protected.abandoned_slots_dropped
                    ),
                    (0, 14, 0)
                );
                let before = (
                    six.clone(),
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    inference.clone(),
                    typed.clone(),
                    metadata.clone(),
                    source.to_owned(),
                    fs::read(fixture.root.join("alpha/src/main.miz")).unwrap(),
                );
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("explicit"),
                        max_steps: 64,
                    },
                };
                let verifier = if threshold == 0 {
                    VerifierPolicy::default()
                        .with_profile_id("artifact λ\0")
                        .with_checker_schema_version(7)
                } else {
                    VerifierPolicy::default()
                };
                let profile = LogicProfile::try_new(
                    "artifact-source",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let vc = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc, vc.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let expected = expected.as_ref().unwrap().as_ref().unwrap();
                let root = fixture.root.join("alpha/manifest-baseline");
                assert!(!root.exists());
                let (draft, main_write, _) = inputs
                    .write_native_theorem_artifact_draft(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        ("proof-witnesses/checked.json", "main.mizir.json", &root),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(&draft, expected);
                assert_eq!(
                    draft.artifact.exports.len(),
                    usize::from(!text.starts_with("private "))
                );
                assert_eq!(
                    draft.artifact.obligations[0].owner_origin_id.is_none(),
                    text.starts_with("private ")
                );
                assert_eq!(draft.artifact.expressions.len(), 9);
                assert_eq!(
                    draft.external_dependency_gaps,
                    [
                        Gap::DriverSession,
                        Gap::DiagnosticsIntegration,
                        Gap::ProducerProjectionPayload,
                        Gap::ArtifactPublicationToken
                    ]
                );
                let package_root = fs::canonicalize(fixture.root.join("alpha")).unwrap();
                let canonical_root = fs::canonicalize(&root).unwrap();
                assert_eq!(
                    canonical_root.strip_prefix(&package_root).unwrap(),
                    Path::new("manifest-baseline")
                );
                let main_bytes = fs::read(root.join(main_write.path.as_str())).unwrap();
                let value = verified_artifact_json(&draft.artifact).unwrap();
                assert_eq!(main_bytes, canonical_json_bytes(&value));
                let main_domain = artifact_hash_domain(
                    VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                    draft.artifact.schema_version,
                );
                let excluded = artifact_hash_excluded_paths();
                assert_eq!(
                    main_write.artifact_hash,
                    main_domain.hash(&value, &excluded)
                );
                let main_read = read_published_artifact(
                    &canonical_root,
                    &main_write.path,
                    PublishedArtifactReadOptions {
                        artifact_hash_domain: Some(&main_domain),
                        hash_excluded_paths: &excluded,
                        expected_artifact_hash: Some(main_write.artifact_hash),
                    },
                )
                .unwrap();
                assert_eq!(
                    read_verified_artifact(
                        &main_read.value,
                        VerifiedArtifactReadOptions {
                            artifact_path: Some(main_write.path.as_str()),
                            expected_module: Some(&draft.artifact.module),
                            expected_interface_hash: Some(draft.artifact.interface_hash),
                            expected_implementation_hash: Some(draft.artifact.implementation_hash),
                        }
                    )
                    .unwrap(),
                    draft.artifact
                );
                let [witness] = draft.artifact.proof_witnesses.as_slice() else {
                    panic!("one genuine witness")
                };
                let witness_bytes = fs::read(root.join(&witness.witness_path)).unwrap();
                let witness_domain = artifact_hash_domain(
                    witness.witness_artifact_hash.schema_family.clone(),
                    witness.witness_artifact_hash.schema_version,
                );
                let witness_path =
                    mizar_artifact::store::PublishedArtifactPath::new(&witness.witness_path)
                        .unwrap();
                let witness_read = read_published_artifact(
                    &canonical_root,
                    &witness_path,
                    PublishedArtifactReadOptions {
                        artifact_hash_domain: Some(&witness_domain),
                        expected_artifact_hash: Some(witness.witness_artifact_hash.digest),
                        ..PublishedArtifactReadOptions::default()
                    },
                )
                .unwrap();
                assert!(
                    !mizar_proof::witness_store::read_source_theorem_witness_file(
                        &witness_read,
                        witness
                    )
                    .unwrap()
                    .is_empty()
                );
                let artifact = &draft.artifact;
                let entry = ModuleArtifactEntry {
                    module: artifact.module.clone(),
                    source_file: artifact.source_file.clone(),
                    source_hash: artifact.source_hash,
                    artifact_file: main_write.path.as_str().into(),
                    artifact_hash: ArtifactHashRef::new(
                        ArtifactHashClass::Artifact,
                        VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                        artifact.schema_version,
                        main_write.artifact_hash,
                    ),
                    interface_hash: ArtifactHashRef::new(
                        ArtifactHashClass::Interface,
                        VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                        artifact.schema_version,
                        artifact.interface_hash,
                    ),
                    implementation_hash: ArtifactHashRef::new(
                        ArtifactHashClass::Implementation,
                        VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                        artifact.schema_version,
                        artifact.implementation_hash,
                    ),
                    module_summary_file: None,
                    module_summary_hash: None,
                    module_summary_interface_hash: None,
                    registration_summary_file: None,
                    registration_summary_hash: None,
                    registration_interface_hash: None,
                    proof_witnesses: artifact
                        .proof_witnesses
                        .iter()
                        .map(|witness| ManifestProofWitnessEntry {
                            obligation_id: witness.obligation_id.clone(),
                            obligation_fingerprint: witness.obligation_fingerprint.clone(),
                            witness_path: witness.witness_path.clone(),
                            witness_artifact_hash: witness.witness_artifact_hash.clone(),
                        })
                        .collect(),
                    diagnostics_hash: None,
                };
                assert!(artifact.diagnostics.is_empty());
                let seed = ArtifactManifest {
                    schema_version: manifest_schema(),
                    package: PackageIdentity {
                        package_id: artifact.module.package_id.clone(),
                        package_version: artifact.module.package_version.clone(),
                        lockfile_identity: artifact.module.lockfile_identity.clone(),
                    },
                    artifact_root: "manifest-baseline".into(),
                    lockfile_hash: artifact.provenance.lockfile_hash.clone(),
                    toolchain: artifact.provenance.toolchain.clone(),
                    language_edition: artifact.provenance.language_edition.clone(),
                    verifier_config_hash: artifact.provenance.verifier_config_hash.clone(),
                    modules: vec![],
                    development_artifacts: vec![],
                    provenance: ManifestProvenance {
                        generated_by: concat!("mizar-driver/", env!("CARGO_PKG_VERSION")).into(),
                        manifest_policy: "source-theorem-unpublished".into(),
                        transaction_format: "mizar-artifact/manifest-transaction-v1".into(),
                    },
                };
                let guard = six.snapshot.to_published_schema_string().unwrap();
                let freshness = |actual_guard: Option<&str>| {
                    actual_guard == Some(guard.as_str())
                        && owner.validate_current_snapshot(six.snapshot).is_ok()
                        && six.parent_outputs().iter().all(|parent| {
                            owner
                                .storage()
                                .validate_handle(parent.as_output_ref())
                                .is_ok()
                                && owner
                                    .validate_current_output(six.snapshot, parent.as_output_ref())
                                    .is_ok()
                        })
                };
                assert!(freshness(Some(&guard)));
                let mut transaction =
                    ManifestTransaction::begin(&canonical_root, seed.clone(), Some(guard.clone()))
                        .unwrap();
                assert_eq!(transaction.base_manifest_hash(), None);
                assert_eq!(transaction.freshness_guard(), Some(guard.as_str()));
                transaction.stage_module(entry.clone()).unwrap();
                let commit = transaction
                    .commit(ManifestCommitOptions {
                        freshness_check: Some(&freshness),
                    })
                    .unwrap();
                let mut expected_manifest = seed;
                expected_manifest.modules.push(entry);
                assert_eq!(commit.manifest(), &expected_manifest);
                let manifest_json = artifact_manifest_json(&expected_manifest).unwrap();
                let manifest_bytes = write_artifact_manifest(&expected_manifest).unwrap();
                assert_eq!(manifest_bytes, canonical_json_bytes(&manifest_json));
                assert_eq!(commit.write().path, artifact_manifest_path());
                assert_eq!(
                    commit.write().artifact_hash,
                    manifest_hash_domain(expected_manifest.schema_version)
                        .hash(&manifest_json, &[])
                );
                assert_eq!(
                    fs::read(root.join(artifact_manifest_path().as_str())).unwrap(),
                    manifest_bytes
                );
                let read = read_manifest_file(
                    &canonical_root,
                    ManifestFileReadOptions {
                        validate_references: true,
                        validate_witness_files: true,
                        validate_development_artifacts: true,
                        expected_manifest_hash: Some(commit.write().artifact_hash),
                    },
                )
                .unwrap();
                assert_eq!(&read.manifest, commit.manifest());
                assert_eq!(read.path, commit.write().path);
                assert_eq!(read.artifact_hash, commit.write().artifact_hash);
                assert!(freshness(Some(&guard)));
                assert_eq!(
                    fs::read(root.join(main_write.path.as_str())).unwrap(),
                    main_bytes
                );
                assert_eq!(
                    fs::read(root.join(&witness.witness_path)).unwrap(),
                    witness_bytes
                );
                let mut entries = fs::read_dir(&root)
                    .unwrap()
                    .map(|entry| entry.unwrap().file_name().into_string().unwrap())
                    .collect::<Vec<_>>();
                entries.sort();
                assert_eq!(
                    entries,
                    [
                        "artifact-manifest.json",
                        "main.mizir.json",
                        "proof-witnesses"
                    ]
                );
                let actual_root = fixture.root.join("alpha/manifest-actual");
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("ArtifactCommit"),
                    OutputKind::new("SourceTheoremArtifactData"),
                    IrWorkUnit::new("\"alpha\":\"main\""),
                ));
                let (actual_draft, actual_main, actual_commit, _, output) = inputs
                    .commit_native_theorem_artifact_manifest(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile, ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (
                            "proof-witnesses/checked.json",
                            "main.mizir.json",
                            &actual_root,
                        ),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(actual_draft, draft);
                assert_eq!(actual_main, main_write);
                let mut actual_expected = expected_manifest.clone();
                actual_expected.artifact_root = "manifest-actual".into();
                assert_eq!(actual_commit.manifest(), &actual_expected);
                let actual_json = artifact_manifest_json(&actual_expected).unwrap();
                assert_eq!(
                    actual_commit.write().artifact_hash,
                    manifest_hash_domain(actual_expected.schema_version).hash(&actual_json, &[])
                );
                assert_ne!(
                    actual_commit.write().artifact_hash,
                    commit.write().artifact_hash
                );
                assert_eq!(
                    fs::read(actual_root.join(artifact_manifest_path().as_str())).unwrap(),
                    canonical_json_bytes(&actual_json)
                );
                let actual_read = read_manifest_file(
                    &actual_root,
                    ManifestFileReadOptions {
                        validate_references: true,
                        validate_witness_files: true,
                        validate_development_artifacts: true,
                        expected_manifest_hash: Some(actual_commit.write().artifact_hash),
                    },
                )
                .unwrap();
                assert_eq!(&actual_read.manifest, actual_commit.manifest());
                assert_eq!(
                    fs::read(actual_root.join(actual_main.path.as_str())).unwrap(),
                    main_bytes
                );
                assert_eq!(
                    fs::read(actual_root.join(&witness.witness_path)).unwrap(),
                    witness_bytes
                );
                assert_eq!(fs::read_dir(&actual_root).unwrap().count(), 3);
                let storage = owner.storage();
                let stored = storage.get(&output).unwrap();
                assert_eq!(*stored, actual_draft.artifact);
                assert_eq!(
                    *stored,
                    read_verified_artifact(
                        &main_read.value,
                        VerifiedArtifactReadOptions {
                            artifact_path: Some(main_write.path.as_str()),
                            expected_module: Some(&draft.artifact.module),
                            expected_interface_hash: Some(draft.artifact.interface_hash),
                            expected_implementation_hash: Some(draft.artifact.implementation_hash),
                        }
                    )
                    .unwrap()
                );
                assert_eq!(output.snapshot(), six.snapshot);
                assert_eq!(output.phase(), &IrPipelinePhase::new("ArtifactCommit"));
                assert_eq!(output.work_unit(), &IrWorkUnit::new("\"alpha\":\"main\""));
                assert_eq!(
                    output.output_kind(),
                    &OutputKind::new("SourceTheoremArtifactData")
                );
                assert_eq!(output.schema_version(), SchemaVersion::new(1));
                assert_eq!(
                    stored.schema_version,
                    mizar_artifact::store::SchemaVersion::new(1, 1)
                );
                assert_eq!(
                    original_outputs
                        .iter()
                        .map(|output| owner.registry().output_lineage(output.output).unwrap())
                        .collect::<Vec<_>>(),
                    original_outputs
                );
                assert!(
                    original_outputs
                        .iter()
                        .all(|old| old.output != output.output())
                );
                let mut parents = six
                    .parent_outputs()
                    .iter()
                    .map(|parent| parent.as_output_ref())
                    .collect::<Vec<_>>();
                parents.sort_by(|left, right| {
                    left.content_hash()
                        .as_bytes()
                        .cmp(right.content_hash().as_bytes())
                        .then_with(|| {
                            left.side_table_hash()
                                .as_bytes()
                                .cmp(right.side_table_hash().as_bytes())
                        })
                });
                let mut parent_ids = parents
                    .iter()
                    .map(|parent| parent.output())
                    .collect::<Vec<_>>();
                parent_ids
                    .sort_by(|left, right| left.hash().as_bytes().cmp(right.hash().as_bytes()));
                assert_eq!(output.lineage().parents, parent_ids);
                let source_key = source_key(&inputs.snapshot.source_versions[0]);
                assert_eq!(source_key, six.identities().input_hash());
                assert_eq!(
                    output.lineage().named_input_hashes,
                    [NamedInputHash {
                        name: "source".into(),
                        domain: SOURCE_UNIT_CACHE_KEY_VERSION.into(),
                        digest: source_key,
                    }]
                );
                let field = |hasher: &mut blake3::Hasher, bytes: &[u8]| {
                    hasher.update(&u64::try_from(bytes.len()).unwrap().to_le_bytes());
                    hasher.update(bytes);
                };
                let semantic = main_domain.hash_input(&value, &excluded).into_bytes();
                let mut content = blake3::Hasher::new();
                field(&mut content, b"mizar-ir/publisher-content-hash/v1");
                field(&mut content, &semantic);
                content.update(&6u64.to_le_bytes());
                for parent in parents {
                    field(&mut content, b"parent_content");
                    content.update(parent.content_hash().as_bytes());
                }
                content.update(&1u64.to_le_bytes());
                for name in [
                    "source",
                    SOURCE_UNIT_CACHE_KEY_VERSION,
                    "mizar-ir/publisher-named-input-hash/v1",
                ] {
                    field(&mut content, name.as_bytes());
                }
                content.update(source_key.as_bytes());
                assert_eq!(
                    output.content_hash(),
                    Hash::from_bytes(*content.finalize().as_bytes())
                );
                let mut side = blake3::Hasher::new();
                field(&mut side, b"mizar-ir/publisher-side-table-hash/v1");
                for category in [
                    "source_maps",
                    "diagnostics",
                    "explanation_refs",
                    "documentation_attachments",
                ] {
                    field(&mut side, category.as_bytes());
                    side.update(&0u64.to_le_bytes());
                }
                assert_eq!(
                    output.side_table_hash(),
                    Hash::from_bytes(*side.finalize().as_bytes())
                );
                assert_eq!(
                    *storage.side_tables(&output).unwrap(),
                    IrSideTables::default()
                );
                match *output.placement() {
                    StoragePlacement::Resident => assert_eq!(threshold, usize::MAX),
                    StoragePlacement::Blob { blob, len } => {
                        assert_eq!(threshold, 0);
                        let mut expected_blob = blake3::Hasher::new();
                        field(&mut expected_blob, b"mizar-ir/content-blob/v1");
                        expected_blob.update(&1u32.to_le_bytes());
                        field(&mut expected_blob, &main_bytes);
                        assert_eq!(
                            blob.hash(),
                            Hash::from_bytes(*expected_blob.finalize().as_bytes())
                        );
                        assert_eq!(len, main_bytes.len());
                    }
                    _ => unreachable!(),
                }
                owner
                    .validate_current_output(six.snapshot, output.any())
                    .unwrap();
                owner
                    .validate_published_output(six.snapshot, output.any())
                    .unwrap();
                assert!(freshness(Some(&guard)));
                for reference in refs {
                    owner.storage().validate_handle(reference).unwrap();
                    owner
                        .validate_published_output(six.snapshot, reference)
                        .unwrap();
                }
                assert_eq!(
                    (
                        six.clone(),
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone(),
                        inference.clone(),
                        typed.clone(),
                        metadata.clone(),
                        source.to_owned(),
                        fs::read(fixture.root.join("alpha/src/main.miz")).unwrap()
                    ),
                    before
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (15, 0)
                );
                assert!(storage.get(&output).is_err());
            },
        );
    }
}

#[test]
fn native_theorem_artifact_manifest_preserves_refusals() {
    use mizar_artifact::manifest::{ManifestFileReadOptions, read_manifest_file};
    use mizar_atp::problem::*;
    use mizar_ir::storage::CollectInput;
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    for case in [
        "buffer",
        "policy",
        "missing",
        "foreign",
        "obsolete",
        "collected",
        "existing",
        "outside",
        "main",
        "repeat",
        "collision",
        "collision-directory",
    ] {
        with_native_theorem_artifact_context(
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            case == "buffer",
            usize::MAX,
            "",
            |fixture, inputs, six, owner, refs, expected, _| {
                assert_eq!(expected.as_ref().unwrap().is_ok(), case != "buffer");
                let before = (
                    six.clone(),
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                );
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("explicit"),
                        max_steps: 64,
                    },
                };
                let verifier = if case == "policy" {
                    VerifierPolicy::default().with_kernel_evidence_formats([])
                } else {
                    VerifierPolicy::default()
                };
                let profile = LogicProfile::try_new(
                    "artifact-source",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let vc = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc, vc.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let input = if case == "missing" {
                    PhaseInput::new(
                        six.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            six.snapshot,
                            six.identities().input_hash(),
                            six.identities().dependency_hashes().to_vec(),
                            six.parent_outputs()[..5].to_vec(),
                        )
                        .unwrap(),
                    )
                } else {
                    six.clone()
                };
                let foreign = std::sync::Arc::new(PhaseOutputPublisher::new(
                    owner.storage().clone(),
                    std::sync::Arc::new(SnapshotHandleRegistry::new()),
                ));
                foreign.register_current_snapshot(six.snapshot);
                let selected = if case == "foreign" { &foreign } else { owner };
                if case == "obsolete" {
                    owner.mark_obsolete(six.snapshot).unwrap();
                }
                if case == "collected" {
                    assert_eq!(
                        owner
                            .storage()
                            .collect(CollectInput {
                                snapshot: six.snapshot,
                                protected_outputs: vec![]
                            })
                            .outputs_dropped,
                        14
                    );
                }
                let root = fixture.root.join(if case == "outside" {
                    "manifest-outside"
                } else {
                    "alpha/manifest-refused"
                });
                let baseline_root = fixture.root.join("alpha/manifest-lower");
                if case == "existing" {
                    for path in [&root, &baseline_root] {
                        fs::create_dir(path).unwrap();
                        fs::write(path.join("sentinel"), b"keep").unwrap();
                    }
                }
                let witness = match case {
                    "collision" => "artifact-manifest.json",
                    "collision-directory" => "artifact-manifest.json/checked.json",
                    _ => "proof-witnesses/checked.json",
                };
                let main = if matches!(case, "main" | "policy") {
                    "bad.json"
                } else {
                    "main.mizir.json"
                };
                let lower = inputs.write_native_theorem_artifact_draft(
                    &input,
                    selected,
                    vcs.vcs()[0].id,
                    (profile.clone(), ConcreteFormat::Tptp),
                    (&policy, &verifier),
                    (witness, main, &baseline_root),
                );
                if case == "repeat" {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("ArtifactCommit"),
                        OutputKind::new("SourceTheoremArtifactData"),
                        IrWorkUnit::new("\"alpha\":\"main\""),
                    ));
                }
                let actual = inputs.commit_native_theorem_artifact_manifest(
                    &input,
                    selected,
                    vcs.vcs()[0].id,
                    (profile.clone(), ConcreteFormat::Tptp),
                    (&policy, &verifier),
                    (witness, main, &root),
                );
                if case == "buffer" {
                    assert_eq!(lower.unwrap(), None);
                    assert_eq!(actual.unwrap(), None);
                    assert!(!root.exists());
                } else if matches!(
                    case,
                    "policy"
                        | "missing"
                        | "foreign"
                        | "obsolete"
                        | "collected"
                        | "existing"
                        | "main"
                        | "collision"
                        | "collision-directory"
                ) {
                    assert_eq!(actual.unwrap_err(), lower.unwrap_err());
                    if case == "main" {
                        assert!(root.join(witness).is_file());
                        assert!(!root.join(main).exists());
                    } else if case == "existing" {
                        assert_eq!(fs::read(root.join("sentinel")).unwrap(), b"keep");
                        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
                    } else {
                        assert!(!root.exists());
                    }
                } else {
                    let (baseline, baseline_write, _) = lower.unwrap().unwrap();
                    if case == "repeat" {
                        let (draft, write, commit, _, _) = actual.unwrap().unwrap();
                        assert_eq!(draft, baseline);
                        assert_eq!(write, baseline_write);
                        let manifest_bytes = fs::read(root.join("artifact-manifest.json")).unwrap();
                        let old = inputs
                            .write_native_theorem_artifact_draft(
                                six,
                                owner,
                                vcs.vcs()[0].id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                (witness, main, &root),
                            )
                            .unwrap_err();
                        assert_eq!(
                            inputs
                                .commit_native_theorem_artifact_manifest(
                                    six,
                                    owner,
                                    vcs.vcs()[0].id,
                                    (profile, ConcreteFormat::Tptp),
                                    (&policy, &verifier),
                                    (witness, main, &root)
                                )
                                .unwrap_err(),
                            old
                        );
                        assert_eq!(
                            fs::read(root.join("artifact-manifest.json")).unwrap(),
                            manifest_bytes
                        );
                        assert_eq!(
                            read_manifest_file(
                                &root,
                                ManifestFileReadOptions {
                                    validate_references: true,
                                    validate_witness_files: true,
                                    expected_manifest_hash: Some(commit.write().artifact_hash),
                                    ..ManifestFileReadOptions::default()
                                }
                            )
                            .unwrap()
                            .manifest,
                            *commit.manifest()
                        );
                    } else {
                        let error = actual.unwrap_err();
                        assert_eq!(
                            error,
                            "native theorem artifact manifest root must be below package root"
                        );
                        assert!(!root.join("artifact-manifest.json").exists());
                    }
                    assert_eq!(
                        fs::read(root.join(main)).unwrap(),
                        fs::read(baseline_root.join(main)).unwrap()
                    );
                    assert_eq!(
                        fs::read(root.join(witness)).unwrap(),
                        fs::read(baseline_root.join(witness)).unwrap()
                    );
                }
                assert_eq!(
                    (
                        six.clone(),
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone()
                    ),
                    before
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (
                        if case == "collected" {
                            0
                        } else if case == "repeat" {
                            15
                        } else {
                            14
                        },
                        0
                    )
                );
            },
        );
    }
}

#[test]
fn native_theorem_artifact_manifest_preserves_commit_and_files() {
    use mizar_artifact::{
        manifest::{
            ManifestCommitOptions, ManifestError, ManifestFileReadOptions, ManifestTransaction,
            read_manifest_file,
        },
        store::{
            PublishedArtifactPath, PublishedArtifactReadOptions, artifact_hash_domain,
            read_published_artifact,
        },
    };
    use mizar_atp::problem::*;
    use mizar_ir::storage::CollectInput;
    use mizar_proof::policy::VerifierPolicy;
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    for case in ["obsolete", "main", "witness", "retained", "witness-content"] {
        with_native_theorem_artifact_context(
            "private theorem Hidden: for X being set holds X = X proof let X be set; thus X = X; end;",
            false,
            usize::MAX,
            "",
            |fixture, inputs, six, owner, refs, expected, _| {
                let before = (
                    six.clone(),
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                );
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("explicit"),
                        max_steps: 64,
                    },
                };
                let verifier = VerifierPolicy::default();
                let profile = LogicProfile::try_new(
                    "artifact-source",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let vc = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc, vc.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let root = fixture.root.join("alpha/manifest-files");
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("ArtifactCommit"),
                    OutputKind::new("SourceTheoremArtifactData"),
                    IrWorkUnit::new("\"alpha\":\"main\""),
                ));
                let (draft, write, commit, _, output) = inputs
                    .commit_native_theorem_artifact_manifest(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile, ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        ("proof-witnesses/checked.json", "main.mizir.json", &root),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(&draft, expected.as_ref().unwrap().as_ref().unwrap());
                let retained_manifest = commit.manifest().clone();
                let retained_write = commit.write().clone();
                let manifest_bytes = fs::read(root.join("artifact-manifest.json")).unwrap();
                let witness = &draft.artifact.proof_witnesses[0];
                let guard = six.snapshot.to_published_schema_string().unwrap();
                let freshness = |candidate: Option<&str>| {
                    candidate == Some(guard.as_str())
                        && owner.validate_current_snapshot(six.snapshot).is_ok()
                        && six.parent_outputs().iter().all(|parent| {
                            owner
                                .storage()
                                .validate_handle(parent.as_output_ref())
                                .is_ok()
                                && owner
                                    .validate_current_output(six.snapshot, parent.as_output_ref())
                                    .is_ok()
                        })
                };
                let mut transaction = ManifestTransaction::begin(
                    &root,
                    retained_manifest.clone(),
                    Some(guard.clone()),
                )
                .unwrap();
                let entry = retained_manifest.modules[0].clone();
                match case {
                    "obsolete" => {
                        owner.mark_obsolete(six.snapshot).unwrap();
                    }
                    "main" => fs::write(root.join(write.path.as_str()), b"{").unwrap(),
                    "witness" => fs::remove_file(root.join(&witness.witness_path)).unwrap(),
                    "retained" => fs::write(root.join("artifact-manifest.json"), b"{").unwrap(),
                    "witness-content" => {
                        fs::write(root.join(&witness.witness_path), b"{}").unwrap()
                    }
                    _ => unreachable!(),
                }
                transaction.stage_module(entry).unwrap();
                let result = transaction.commit(ManifestCommitOptions {
                    freshness_check: Some(&freshness),
                });
                if case == "witness-content" {
                    let next = result.unwrap();
                    assert_eq!(next.manifest(), &retained_manifest);
                    assert_eq!(next.write(), &retained_write);
                    assert!(
                        read_manifest_file(
                            &root,
                            ManifestFileReadOptions {
                                validate_references: true,
                                validate_witness_files: true,
                                validate_development_artifacts: true,
                                expected_manifest_hash: Some(retained_write.artifact_hash)
                            }
                        )
                        .is_ok()
                    );
                    let path = PublishedArtifactPath::new(&witness.witness_path).unwrap();
                    let domain = artifact_hash_domain(
                        witness.witness_artifact_hash.schema_family.clone(),
                        witness.witness_artifact_hash.schema_version,
                    );
                    assert!(
                        read_published_artifact(
                            &root,
                            &path,
                            PublishedArtifactReadOptions {
                                artifact_hash_domain: Some(&domain),
                                expected_artifact_hash: Some(witness.witness_artifact_hash.digest),
                                ..PublishedArtifactReadOptions::default()
                            }
                        )
                        .is_err()
                    );
                } else {
                    let error = result.unwrap_err();
                    if case == "obsolete" {
                        assert!(matches!(error, ManifestError::ObsoleteSnapshot { .. }));
                    }
                    if case != "retained" {
                        assert_eq!(
                            fs::read(root.join("artifact-manifest.json")).unwrap(),
                            manifest_bytes
                        );
                    }
                    if matches!(case, "main" | "witness" | "retained") {
                        assert!(
                            read_manifest_file(
                                &root,
                                ManifestFileReadOptions {
                                    validate_references: true,
                                    validate_witness_files: true,
                                    validate_development_artifacts: true,
                                    expected_manifest_hash: Some(retained_write.artifact_hash)
                                }
                            )
                            .is_err()
                        );
                    }
                }
                assert_eq!(*owner.storage().get(&output).unwrap(), draft.artifact);
                assert_eq!(
                    owner
                        .validate_current_output(six.snapshot, output.any())
                        .is_ok(),
                    case != "obsolete"
                );
                assert_eq!(commit.manifest(), &retained_manifest);
                assert_eq!(commit.write(), &retained_write);
                assert_eq!(
                    (
                        six.clone(),
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone()
                    ),
                    before
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (15, 0)
                );
            },
        );
    }
}

#[test]
fn native_theorem_artifact_ir_preserves_late_denial_and_duplicates() {
    use mizar_artifact::{
        manifest::{
            ManifestCommitOptions, ManifestFileReadOptions, ManifestTransaction, read_manifest_file,
        },
        store::{PublishedArtifactReadOptions, artifact_hash_domain, read_published_artifact},
        verified_artifact::{
            VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifactReadOptions,
            artifact_hash_excluded_paths, read_verified_artifact,
        },
    };
    use mizar_atp::problem::*;
    use mizar_ir::{
        identity::IdentityError,
        publisher::PublishError,
        storage::{CollectInput, StorageError},
    };
    use mizar_proof::{
        policy::VerifierPolicy,
        witness_store::{CommittedWitnessPublicationProof, publish_ref},
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{PolicyKey, VcSet},
    };
    for case in ["denied", "same", "different"] {
        with_native_theorem_artifact_context(
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            false,
            usize::MAX,
            "",
            |fixture, inputs, six, owner, refs, expected, (inference, typed, metadata, source)| {
                let mut original_outputs = refs
                    .iter()
                    .map(|reference| owner.registry().output_lineage(reference.output()).unwrap())
                    .collect::<Vec<_>>();
                let kernel = refs
                    .iter()
                    .find(|reference| {
                        reference.output_kind() == &OutputKind::new("SourceTheoremCandidateCheck")
                    })
                    .unwrap();
                let unlisted = kernel
                    .lineage()
                    .parents
                    .iter()
                    .filter(|parent| {
                        original_outputs
                            .iter()
                            .all(|output| output.output != **parent)
                    })
                    .collect::<Vec<_>>();
                let [candidate] = unlisted.as_slice() else {
                    panic!("one genuine inline ATP candidate parent")
                };
                let candidate = owner.registry().output_lineage(**candidate).unwrap();
                assert_eq!(candidate.phase, IrPipelinePhase::new("AtpSolve"));
                assert_eq!(
                    candidate.output_kind,
                    OutputKind::new("SourceTheoremCandidateInputs")
                );
                original_outputs.push(candidate);
                assert_eq!(original_outputs.len(), 14);
                let protected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: original_outputs
                        .iter()
                        .map(|output| output.output)
                        .collect(),
                });
                assert_eq!(
                    (
                        protected.outputs_dropped,
                        protected.protected_outputs,
                        protected.abandoned_slots_dropped
                    ),
                    (0, 14, 0)
                );
                let before = (
                    six.clone(),
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    inference.clone(),
                    typed.clone(),
                    metadata.clone(),
                    source.to_owned(),
                    fs::read(fixture.root.join("alpha/src/main.miz")).unwrap(),
                );
                let policy = DischargePolicy {
                    computation_limit: ComputationLimit {
                        policy: PolicyKey::new("explicit"),
                        max_steps: 64,
                    },
                };
                let verifier = VerifierPolicy::default();
                let profile = LogicProfile::try_new(
                    "artifact-source",
                    LogicFragment::Fof,
                    EqualitySupport::Supported,
                    QuantifierPolicy::FirstOrder,
                    SoftTypeStrategy::ScopedGuardPredicates,
                    NativePropertySupport::Supported,
                    [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                        .into_iter()
                        .collect(),
                )
                .unwrap();
                let vc = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("SourceTheoremVC"))
                    .unwrap();
                let vcs = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<VcSet>(vc, vc.output_kind())
                            .unwrap(),
                    )
                    .unwrap();
                let phase = IrPipelinePhase::new("ArtifactCommit");
                let kind = OutputKind::new("SourceTheoremArtifactData");
                let unit = IrWorkUnit::new("\"alpha\":\"main\"");
                let first_root = fixture.root.join("alpha/artifact-ir-first");
                let previous = if case == "denied" {
                    None
                } else {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        phase.clone(),
                        kind.clone(),
                        unit.clone(),
                    ));
                    Some(
                        inputs
                            .commit_native_theorem_artifact_manifest(
                                six,
                                owner,
                                vcs.vcs()[0].id,
                                (profile.clone(), ConcreteFormat::Tptp),
                                (&policy, &verifier),
                                (
                                    "proof-witnesses/checked.json",
                                    "main.mizir.json",
                                    &first_root,
                                ),
                            )
                            .unwrap()
                            .unwrap(),
                    )
                };
                if let Some((draft, _, _, _, output)) = &previous {
                    assert_eq!(draft, expected.as_ref().unwrap().as_ref().unwrap());
                    assert_eq!(*owner.storage().get(output).unwrap(), draft.artifact);
                }
                let previous_files = previous.as_ref().map(|_| {
                    [
                        "artifact-manifest.json",
                        "main.mizir.json",
                        "proof-witnesses/checked.json",
                    ]
                    .map(|path| fs::read(first_root.join(path)).unwrap())
                });
                let witness_path = if case == "different" {
                    "proof-witnesses/other.json"
                } else {
                    "proof-witnesses/checked.json"
                };
                let lower_root = fixture.root.join("alpha/artifact-ir-lower");
                let (baseline, baseline_write, staged) = inputs
                    .write_native_theorem_artifact_draft(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (witness_path, "main.mizir.json", &lower_root),
                    )
                    .unwrap()
                    .unwrap();
                if case != "different" {
                    assert_eq!(&baseline, expected.as_ref().unwrap().as_ref().unwrap());
                }
                let root = fixture.root.join("alpha/artifact-ir-attempt");
                let error = inputs
                    .commit_native_theorem_artifact_manifest(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile, ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (witness_path, "main.mizir.json", &root),
                    )
                    .unwrap_err();
                let expected_error = match case {
                    "denied" => PublishError::WorkUnitNotAllowed {
                        phase,
                        work_unit: unit,
                        output_kind: kind,
                    },
                    "same" => PublishError::Storage {
                        error: Box::new(StorageError::OutputAlreadyStored {
                            output: previous.as_ref().unwrap().4.output(),
                        }),
                    },
                    "different" => PublishError::Identity {
                        error: Box::new(IdentityError::ConflictingDuplicate {
                            collection: "phase_outputs",
                            key: "mizar-ir/phase-output-id/v1:phase=ArtifactCommit,work_unit=\"alpha\":\"main\",output_kind=SourceTheoremArtifactData".into(),
                        }),
                    },
                    _ => unreachable!(),
                };
                assert_eq!(error, expected_error.to_string(), "{case}");
                for path in ["main.mizir.json", witness_path] {
                    assert_eq!(
                        fs::read(root.join(path)).unwrap(),
                        fs::read(lower_root.join(path)).unwrap()
                    );
                }
                let domain = artifact_hash_domain(
                    VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                    baseline.artifact.schema_version,
                );
                let excluded = artifact_hash_excluded_paths();
                let main = read_published_artifact(
                    &root,
                    &baseline_write.path,
                    PublishedArtifactReadOptions {
                        artifact_hash_domain: Some(&domain),
                        hash_excluded_paths: &excluded,
                        expected_artifact_hash: Some(baseline_write.artifact_hash),
                    },
                )
                .unwrap();
                assert_eq!(
                    read_verified_artifact(
                        &main.value,
                        VerifiedArtifactReadOptions {
                            artifact_path: Some(baseline_write.path.as_str()),
                            expected_module: Some(&baseline.artifact.module),
                            expected_interface_hash: Some(baseline.artifact.interface_hash),
                            expected_implementation_hash: Some(
                                baseline.artifact.implementation_hash
                            ),
                        },
                    )
                    .unwrap(),
                    baseline.artifact
                );
                let manifest_bytes = fs::read(root.join("artifact-manifest.json")).unwrap();
                let read = read_manifest_file(
                    &root,
                    ManifestFileReadOptions {
                        validate_references: true,
                        validate_witness_files: true,
                        validate_development_artifacts: true,
                        expected_manifest_hash: None,
                    },
                )
                .unwrap();
                assert_eq!(read.manifest.artifact_root, "artifact-ir-attempt");
                assert_eq!(read.manifest.modules.len(), 1);
                assert_eq!(
                    read.manifest.modules[0].artifact_hash.digest,
                    baseline_write.artifact_hash
                );
                let mut transaction =
                    ManifestTransaction::begin(&root, read.manifest.clone(), None).unwrap();
                transaction
                    .stage_module(read.manifest.modules[0].clone())
                    .unwrap();
                let commit = transaction
                    .commit(ManifestCommitOptions::default())
                    .unwrap();
                assert_eq!(commit.write().artifact_hash, read.artifact_hash);
                assert_eq!(
                    fs::read(root.join("artifact-manifest.json")).unwrap(),
                    manifest_bytes
                );
                let proof = CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                    &commit,
                    &root,
                    &baseline.artifact.module,
                    six.snapshot.fingerprint(),
                )
                .unwrap();
                assert_eq!(
                    publish_ref(&staged, &proof).unwrap().reference(),
                    &baseline.artifact.proof_witnesses[0]
                );
                if let Some((draft, _, _, _, output)) = &previous {
                    assert_eq!(*owner.storage().get(output).unwrap(), draft.artifact);
                    owner
                        .validate_current_output(six.snapshot, output.any())
                        .unwrap();
                    assert_eq!(
                        [
                            "artifact-manifest.json",
                            "main.mizir.json",
                            "proof-witnesses/checked.json"
                        ]
                        .map(|path| fs::read(first_root.join(path)).unwrap()),
                        previous_files.unwrap()
                    );
                    assert_eq!(draft.artifact == baseline.artifact, case == "same");
                }
                assert_eq!(
                    original_outputs
                        .iter()
                        .map(|output| owner.registry().output_lineage(output.output).unwrap())
                        .collect::<Vec<_>>(),
                    original_outputs
                );
                for reference in refs {
                    owner.storage().validate_handle(reference).unwrap();
                    owner
                        .validate_current_output(six.snapshot, reference)
                        .unwrap();
                }
                assert_eq!(
                    (
                        six.clone(),
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone(),
                        inference.clone(),
                        typed.clone(),
                        metadata.clone(),
                        source.to_owned(),
                        fs::read(fixture.root.join("alpha/src/main.miz")).unwrap(),
                    ),
                    before
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (if case == "denied" { 14 } else { 15 }, 1)
                );
            },
        );
    }
}
