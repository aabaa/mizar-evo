use super::*;
use mizar_artifact::{
    module_summary::{
        DependencyInterfaceRef, ExportedLabelSummary, ExportedSymbolSummary,
        LexicalContributionSummary, ModuleLexicalSummary, ModuleReexportSummary,
        ProofStatusSummary, SourceRangeSummary, current_schema_version,
    },
    registration_summary::{
        ActivatedRegistrationSummary, ArtifactHashClass, ArtifactHashRef,
        RegistrationAcceptedStatus, RegistrationContributionKind, RegistrationContributionSummary,
        RegistrationKind, RegistrationPatternSummary, RegistrationSummary, RegistrationVisibility,
    },
};
use mizar_core::{
    control_flow::ObligationHandoffId,
    core_ir::{
        CoreFormulaId, CoreItemId, CoreLabelRef, CoreProvenance, CoreProvenancePhase,
        CoreSourceRef, LocalProofOrProgramPath, NormalizedSemanticOrigin, ObligationSeedId,
        ObligationSeedStatus,
    },
};
use mizar_session::{BuildSnapshotId, InMemorySessionIdAllocator, SessionIdAllocator, SourceRange};
use mizar_vc::{
    dependency_slice::{DependencySliceInput, try_compute_dependency_slices},
    vc_ir::{
        AnchorCompleteness, AnchorLabel, AnchorLabelRole, AnchorOwner, CanonicalSortKey,
        ContextEntry, ContextEntryId, ContextEntryKind, GenerationSchemaVersion, HashMarker,
        LocalContext, PremiseRef, SeedAccounting, SeedOriginRef, SeedVcMapping, SeedVcRef,
        VcFormulaRef, VcId, VcIr, VcKind, VcModuleRef, VcProvenance, VcProvenancePhase,
        VcSchemaVersion, VcSet, VcSetParts, VcSourceRef, VcStatus, VcText,
    },
};

#[test]
fn footprint_is_deterministic_and_canonicalizes_vectors() {
    let mut first = request();
    first.fingerprints.push(first.fingerprints[0].clone());
    first.slices.push(first.slices[0].clone());
    first
        .compatibility_fields
        .push(first.compatibility_fields[0].clone());
    first
        .proof_reuse_validation
        .push(first.proof_reuse_validation[0].clone());
    first.unknown_markers.reverse();
    first.fingerprints.reverse();
    first.slices.reverse();
    first.compatibility_fields.reverse();
    first.proof_reuse_validation.reverse();

    let first = reusable(first);
    let second = reusable(request());

    assert_eq!(first.final_hash, second.final_hash);
    assert_eq!(first.fingerprints, second.fingerprints);
    assert_eq!(first.slices, second.slices);
    assert_eq!(first.compatibility_fields, second.compatibility_fields);
    assert_eq!(first.proof_reuse_validation, second.proof_reuse_validation);
}

#[test]
fn unknown_markers_are_sorted_and_deduplicated() {
    let mut first = request();
    first.unknown_markers = vec![
        unknown_marker("trace", "pkg::A", "unknown trace coverage"),
        unknown_marker("import", "pkg::A", "unknown imported interface"),
        unknown_marker("trace", "pkg::A", "unknown trace coverage"),
    ];
    first.unknown_markers.reverse();
    let mut second = request();
    second.unknown_markers = vec![
        unknown_marker("import", "pkg::A", "unknown imported interface"),
        unknown_marker("trace", "pkg::A", "unknown trace coverage"),
    ];

    let first = uncacheable(first);
    let second = uncacheable(second);

    assert_eq!(first.final_hash, second.final_hash);
    assert_eq!(first.unknown_markers, second.unknown_markers);
    assert_eq!(
        first.unknown_markers,
        [
            unknown_marker("import", "pkg::A", "unknown imported interface"),
            unknown_marker("trace", "pkg::A", "unknown trace coverage"),
        ]
    );
}

#[test]
fn duplicate_identity_with_different_payload_forces_uncacheable_diagnostic() {
    assert_conflicting_duplicate("fingerprints", |request| {
        let mut conflicting = request.fingerprints[0].clone();
        conflicting.value_hash = hash(99);
        request.fingerprints.push(conflicting);
    });
    assert_conflicting_duplicate("slices", |request| {
        let mut conflicting = request.slices[0].clone();
        conflicting.digest = hash(98);
        request.slices.push(conflicting);
    });
    assert_conflicting_duplicate("compatibility_fields", |request| {
        let mut conflicting = request.compatibility_fields[0].clone();
        conflicting.value = "nightly".to_owned();
        request.compatibility_fields.push(conflicting);
    });
    assert_conflicting_duplicate("proof_reuse_validation", |request| {
        let mut conflicting = request.proof_reuse_validation[0].clone();
        conflicting
            .validation_hash
            .as_mut()
            .expect("validation hash")
            .digest = hash(97);
        request.proof_reuse_validation.push(conflicting);
    });
    assert_conflicting_duplicate(
        "proof_reuse_validation.metadata_schema_versions",
        |request| {
            request.proof_reuse_validation[0]
                .metadata_schema_versions
                .push(NamedSchemaVersion {
                    schema_family: "mizar-proof/reuse-metadata".to_owned(),
                    name: "proof-reuse".to_owned(),
                    version: SchemaVersion::new("mizar-proof/reuse-metadata/v2"),
                });
        },
    );
}

#[test]
fn conflicting_duplicate_diagnostic_hash_is_order_independent() {
    let mut slice_first = request();
    let mut slice_conflict = slice_first.slices[0].clone();
    slice_conflict.completeness = DependencyFootprintCompleteness::IncompleteUncacheable;
    let mut slice_second = slice_first.clone();
    slice_first.slices.push(slice_conflict.clone());
    slice_second.slices.insert(0, slice_conflict);
    assert_eq!(
        uncacheable(slice_first).final_hash,
        uncacheable(slice_second).final_hash
    );

    let mut proof_first = request();
    let mut validation_only = proof_first.proof_reuse_validation[0].clone();
    validation_only.witness_or_discharge_hash = None;
    let mut witness_only = proof_first.proof_reuse_validation[0].clone();
    witness_only.validation_hash = None;
    proof_first.proof_reuse_validation = vec![validation_only.clone(), witness_only.clone()];
    let mut proof_second = request();
    proof_second.proof_reuse_validation = vec![witness_only, validation_only];
    assert_eq!(
        uncacheable(proof_first).final_hash,
        uncacheable(proof_second).final_hash
    );

    let mut fingerprint_first = request();
    let mut hidden_payload_conflict = fingerprint_first.fingerprints[0].clone();
    hidden_payload_conflict.importer_visible = false;
    let mut fingerprint_second = fingerprint_first.clone();
    fingerprint_first
        .fingerprints
        .push(hidden_payload_conflict.clone());
    fingerprint_second
        .fingerprints
        .insert(0, hidden_payload_conflict);
    assert_eq!(
        uncacheable(fingerprint_first).final_hash,
        uncacheable(fingerprint_second).final_hash
    );
}

#[test]
fn semantic_fingerprint_fields_change_final_hash() {
    let base = reusable(request()).final_hash;
    let cases = vec![
        mutate(|request| request.owner.package_id = "other".to_owned()),
        mutate(|request| request.phase = PipelinePhase::new("proof")),
        mutate(|request| request.fingerprints[0].target = FingerprintTargetKind::Definition),
        mutate(|request| request.fingerprints[0].identity.target_name = "other".to_owned()),
        mutate(|request| request.fingerprints[0].value_domain = "other-domain".to_owned()),
        mutate(|request| request.fingerprints[0].value_hash = hash(42)),
        mutate(|request| request.slices[0].name = "other-vc".to_owned()),
        mutate(|request| request.slices[0].digest = hash(43)),
        mutate(|request| request.compatibility_fields[0].value = "other".to_owned()),
        mutate(|request| {
            request.proof_reuse_validation[0]
                .validation_hash
                .as_mut()
                .expect("validation hash")
                .digest = hash(44);
        }),
    ];

    for case in cases {
        assert_ne!(base, reusable(case).final_hash);
    }
}

#[test]
fn non_interface_summary_metadata_is_excluded_from_importer_visible_fingerprint() {
    let base_module = module_summary(hash(11));
    let mut changed_module = module_summary(hash(11));
    changed_module.source_hash = hash(90);
    changed_module.exported_symbols[0].source_range = SourceRangeSummary {
        start_byte: 100,
        end_byte: 120,
    };
    changed_module.exported_labels[0].source_range = SourceRangeSummary {
        start_byte: 200,
        end_byte: 220,
    };
    assert_eq!(
        DependencyFingerprint::from_module_summary(&base_module),
        DependencyFingerprint::from_module_summary(&changed_module)
    );

    let base_registration = registration_summary(hash(12));
    let mut changed_registration = registration_summary(hash(12));
    changed_registration.source_hash = hash(91);
    changed_registration.activated_registrations[0].source_range = Some(SourceRangeSummary {
        start_byte: 300,
        end_byte: 320,
    });
    assert_eq!(
        DependencyFingerprint::from_registration_summary(&base_registration),
        DependencyFingerprint::from_registration_summary(&changed_registration)
    );
}

#[test]
fn interface_change_invalidates_importer_visible_fingerprint() {
    let mut changed = request();
    changed.fingerprints[0].value_hash = hash(50);

    assert_ne!(reusable(request()).final_hash, reusable(changed).final_hash);
}

#[test]
fn implementation_only_change_does_not_change_importer_visible_subset() {
    let mut base = request();
    base.fingerprints.push(implementation_fingerprint(hash(70)));
    let mut changed = request();
    changed
        .fingerprints
        .push(implementation_fingerprint(hash(71)));

    let base_importer = importer_visible_fingerprint_hashes(&reusable(base));
    let changed_importer = importer_visible_fingerprint_hashes(&reusable(changed));

    assert_eq!(base_importer, changed_importer);
}

#[test]
fn slice_change_flips_only_dependent_slice_hash_at_current_granularity() {
    let mut base_request = request();
    base_request
        .slices
        .push(slice("vc-slice", "pkg::A", "obligation-2", hash(32)));
    let mut changed_request = base_request.clone();
    changed_request.slices[0].digest = hash(33);

    let base = reusable(base_request);
    let changed = reusable(changed_request);

    let base_named = slices_by_name(&base);
    let changed_named = slices_by_name(&changed);
    assert_ne!(
        base_named.get("obligation-1"),
        changed_named.get("obligation-1")
    );
    assert_eq!(
        base_named.get("obligation-2"),
        changed_named.get("obligation-2")
    );
    assert_ne!(base.final_hash, changed.final_hash);
}

#[test]
fn vc_dependency_slice_projection_consumes_producer_slice_fingerprint() {
    let vc_set = vc_fixture_set();
    let slice_set = try_compute_dependency_slices(DependencySliceInput {
        vc_set: &vc_set,
        discharge_output: None,
    })
    .expect("dependency slices");
    let producer_slice = slice_set.slice_for(VcId::new(0)).expect("vc slice");
    let projected = DependencySliceFingerprint::from_vc_slice(
        "vc-slice",
        "pkg::A",
        "obligation-1",
        producer_slice,
    );

    assert_eq!(projected.digest, producer_slice.fingerprint().hash());
    assert_eq!(projected.domain, DEPENDENCY_SLICE_SCHEMA_VERSION);
    assert_eq!(
        projected.completeness,
        match producer_slice.completeness() {
            DependencySliceCompleteness::Complete => DependencyFootprintCompleteness::Complete,
            DependencySliceCompleteness::IncompleteUncacheable => {
                DependencyFootprintCompleteness::IncompleteUncacheable
            }
            _ => DependencyFootprintCompleteness::IncompleteUncacheable,
        }
    );
}

#[test]
fn missing_unknown_and_uncacheable_inputs_force_miss() {
    for case in [
        mutate(|request| {
            request.requested_completeness = DependencyFootprintCompleteness::IncompleteUncacheable
        }),
        mutate(|request| {
            request.unknown_markers.push(UnknownDependencyMarker {
                family: "import".to_owned(),
                owner: "pkg::A".to_owned(),
                reason: "producer reported unknown coverage".to_owned(),
            });
        }),
        mutate(|request| {
            request.slices[0].completeness = DependencyFootprintCompleteness::IncompleteUncacheable;
        }),
        mutate(|request| request.uncacheable = true),
    ] {
        let footprint = uncacheable(case);
        assert_eq!(
            footprint.completeness,
            DependencyFootprintCompleteness::IncompleteUncacheable
        );
        assert!(footprint.uncacheable);
    }

    let mut missing_required_families = request();
    missing_required_families.requested_completeness = DependencyFootprintCompleteness::Complete;
    missing_required_families.slices.clear();
    missing_required_families.compatibility_fields.clear();
    missing_required_families.proof_reuse_validation.clear();
    let footprint = uncacheable(missing_required_families);
    assert_marker_family(&footprint, "compatibility");
    assert_marker_family(&footprint, "vc_slice");
    assert_marker_family(&footprint, "proof_reuse_identity");

    let mut unknown_toolchain = request();
    unknown_toolchain.compatibility_fields[0].value = "unknown".to_owned();
    assert!(uncacheable(unknown_toolchain).uncacheable);

    let mut empty_toolchain = request();
    empty_toolchain.compatibility_fields[0].value.clear();
    assert!(uncacheable(empty_toolchain).uncacheable);
}

#[test]
fn proof_reuse_validation_failures_force_miss_without_granting_trust() {
    for state in [
        ProofReuseValidationState::Mismatched,
        ProofReuseValidationState::Missing,
        ProofReuseValidationState::ExternalOnly,
        ProofReuseValidationState::UnsupportedEvidenceKind("backend-log".to_owned()),
    ] {
        let mut request = request();
        request.proof_reuse_validation[0].state = state;
        let footprint = uncacheable(request);
        assert!(footprint.uncacheable);
    }

    let mut missing_hash = request();
    missing_hash.proof_reuse_validation[0].validation_hash = None;
    assert!(uncacheable(missing_hash).uncacheable);

    let mut missing_witness = request();
    missing_witness.proof_reuse_validation[0].witness_or_discharge_hash = None;
    assert!(uncacheable(missing_witness).uncacheable);

    let mut missing_schema = request();
    missing_schema.proof_reuse_validation[0]
        .metadata_schema_versions
        .clear();
    assert!(uncacheable(missing_schema).uncacheable);
}

#[test]
fn unsupported_footprint_schema_produces_no_footprint() {
    let mut request = request();
    request.schema_version = SchemaVersion::new("mizar-cache/dependency-fingerprint-schema/v2");

    let rejection = match DependencyFootprintBuilder::new(request).build() {
        DependencyFootprintBuildOutcome::NoFootprint(rejection) => rejection,
        outcome => panic!("expected no footprint, got {outcome:?}"),
    };

    assert!(matches!(
        rejection,
        DependencyFootprintBuildRejection::UnsupportedSchema { .. }
    ));
}

#[test]
fn trigger_evaluator_reuses_for_non_semantic_changes() {
    for change_kind in [
        FingerprintChangeKind::CommentOnly,
        FingerprintChangeKind::DiagnosticWordingOnly,
        FingerprintChangeKind::RuntimeObservationOnly,
    ] {
        let decision = trigger(change_kind, FingerprintTargetKind::Source);
        assert_eq!(decision.trigger, RebuildTrigger::ReuseAllowed);
        assert!(!decision.conservative);
    }
}

#[test]
fn trigger_evaluator_rebuilds_dependents_for_visible_semantic_changes() {
    for (change_kind, target) in [
        (
            FingerprintChangeKind::SourceTokenAst,
            FingerprintTargetKind::Source,
        ),
        (
            FingerprintChangeKind::ModuleInterface,
            FingerprintTargetKind::ModuleInterface,
        ),
        (
            FingerprintChangeKind::RegistrationInterface,
            FingerprintTargetKind::RegistrationInterface,
        ),
        (
            FingerprintChangeKind::ClusterReductionVisibleOrigin,
            FingerprintTargetKind::ClusterTrace,
        ),
        (
            FingerprintChangeKind::ExportedSemantic,
            FingerprintTargetKind::TheoremStatement,
        ),
    ] {
        let decision = trigger(change_kind, target);
        assert_eq!(decision.trigger, RebuildTrigger::RebuildDependents);
        assert!(!decision.conservative);
    }
}

#[test]
fn trigger_evaluator_refreshes_local_or_affected_phase_for_local_and_config_changes() {
    for (change_kind, target) in [
        (
            FingerprintChangeKind::ModuleImplementationOnly,
            FingerprintTargetKind::ModuleImplementation,
        ),
        (
            FingerprintChangeKind::ProofBodyOnly,
            FingerprintTargetKind::ProofBody,
        ),
        (
            FingerprintChangeKind::Policy,
            FingerprintTargetKind::PolicyToolchain,
        ),
        (
            FingerprintChangeKind::Toolchain,
            FingerprintTargetKind::PolicyToolchain,
        ),
        (
            FingerprintChangeKind::SchemaVersion,
            FingerprintTargetKind::PolicyToolchain,
        ),
        (
            FingerprintChangeKind::Lockfile,
            FingerprintTargetKind::LockfileManifest,
        ),
        (
            FingerprintChangeKind::Manifest,
            FingerprintTargetKind::LockfileManifest,
        ),
    ] {
        let decision = trigger(change_kind, target);
        assert_eq!(decision.trigger, RebuildTrigger::RebuildPhase);
        assert!(!decision.conservative);
    }
}

#[test]
fn trigger_evaluator_misses_for_incomplete_unknown_or_uncacheable_inputs() {
    for change_kind in [
        FingerprintChangeKind::IncompleteFootprint,
        FingerprintChangeKind::UnknownSchema,
        FingerprintChangeKind::UnknownToolchain,
        FingerprintChangeKind::UncacheableMarker,
        FingerprintChangeKind::MissingProofReuseValidation,
    ] {
        let decision = trigger(change_kind, FingerprintTargetKind::ProofReuseIdentity);
        assert_eq!(decision.trigger, RebuildTrigger::UncacheableMiss);
        assert!(!decision.conservative);
    }
}

#[test]
fn conservative_coarse_slices_overtrigger_without_false_negative_reuse() {
    let semantic = RebuildTriggerEvaluator::evaluate(RebuildTriggerInput {
        change_kind: FingerprintChangeKind::ExportedSemantic,
        target: FingerprintTargetKind::Definition,
        dependent_phase: PipelinePhase::new("proof"),
        slice_precision: DependencySlicePrecision::ConservativeCoarse,
    });

    assert_eq!(semantic.trigger, RebuildTrigger::RebuildDependents);
    assert!(semantic.conservative);

    let diagnostic = RebuildTriggerEvaluator::evaluate(RebuildTriggerInput {
        change_kind: FingerprintChangeKind::DiagnosticWordingOnly,
        target: FingerprintTargetKind::VcSlice,
        dependent_phase: PipelinePhase::new("proof"),
        slice_precision: DependencySlicePrecision::ConservativeCoarse,
    });

    assert_eq!(diagnostic.trigger, RebuildTrigger::ReuseAllowed);
    assert!(!diagnostic.conservative);
}

#[test]
fn trigger_evaluator_combines_rows_by_documented_precedence() {
    let rows = vec![
        trigger_input(
            FingerprintChangeKind::CommentOnly,
            FingerprintTargetKind::Source,
            DependencySlicePrecision::Exact,
        ),
        trigger_input(
            FingerprintChangeKind::ModuleImplementationOnly,
            FingerprintTargetKind::ModuleImplementation,
            DependencySlicePrecision::Exact,
        ),
        trigger_input(
            FingerprintChangeKind::ModuleInterface,
            FingerprintTargetKind::ModuleInterface,
            DependencySlicePrecision::Exact,
        ),
        trigger_input(
            FingerprintChangeKind::UnknownSchema,
            FingerprintTargetKind::PolicyToolchain,
            DependencySlicePrecision::Exact,
        ),
    ];
    let mut reversed = rows.clone();
    reversed.reverse();

    let first = RebuildTriggerEvaluator::evaluate_all(rows);
    let second = RebuildTriggerEvaluator::evaluate_all(reversed);

    assert_eq!(first, second);
    assert_eq!(first.trigger, RebuildTrigger::UncacheableMiss);
    assert!(!first.conservative);
    assert_eq!(first.row_count, 4);
}

#[test]
fn trigger_evaluator_combined_conservative_flag_tracks_strongest_coarse_rows() {
    let summary = RebuildTriggerEvaluator::evaluate_all([
        trigger_input(
            FingerprintChangeKind::ModuleInterface,
            FingerprintTargetKind::ModuleInterface,
            DependencySlicePrecision::ConservativeCoarse,
        ),
        trigger_input(
            FingerprintChangeKind::ProofBodyOnly,
            FingerprintTargetKind::ProofBody,
            DependencySlicePrecision::ConservativeCoarse,
        ),
    ]);

    assert_eq!(summary.trigger, RebuildTrigger::RebuildDependents);
    assert!(summary.conservative);
    assert_eq!(summary.row_count, 2);
}

#[test]
fn trigger_evaluator_combined_conservative_flag_ignores_lower_precedence_rows() {
    let mixed_equal_precedence = RebuildTriggerEvaluator::evaluate_all([
        trigger_input(
            FingerprintChangeKind::ModuleInterface,
            FingerprintTargetKind::ModuleInterface,
            DependencySlicePrecision::Exact,
        ),
        trigger_input(
            FingerprintChangeKind::ExportedSemantic,
            FingerprintTargetKind::TheoremStatement,
            DependencySlicePrecision::ConservativeCoarse,
        ),
    ]);
    assert_eq!(
        mixed_equal_precedence.trigger,
        RebuildTrigger::RebuildDependents
    );
    assert!(mixed_equal_precedence.conservative);

    let stronger_uncacheable = RebuildTriggerEvaluator::evaluate_all([
        trigger_input(
            FingerprintChangeKind::ExportedSemantic,
            FingerprintTargetKind::TheoremStatement,
            DependencySlicePrecision::ConservativeCoarse,
        ),
        trigger_input(
            FingerprintChangeKind::UncacheableMarker,
            FingerprintTargetKind::PolicyToolchain,
            DependencySlicePrecision::Exact,
        ),
    ]);
    assert_eq!(
        stronger_uncacheable.trigger,
        RebuildTrigger::UncacheableMiss
    );
    assert!(!stronger_uncacheable.conservative);
}

#[test]
fn summary_constructors_project_producer_hashes() {
    let module_summary = module_summary(hash(11));
    let module_fingerprint = DependencyFingerprint::from_module_summary(&module_summary);
    assert_eq!(
        module_fingerprint.target,
        FingerprintTargetKind::ModuleInterface
    );
    assert_eq!(module_fingerprint.value_hash, module_summary.interface_hash);
    assert!(module_fingerprint.importer_visible);

    let registration_summary = registration_summary(hash(12));
    let registration_fingerprint =
        DependencyFingerprint::from_registration_summary(&registration_summary);
    assert_eq!(
        registration_fingerprint.target,
        FingerprintTargetKind::RegistrationInterface
    );
    assert_eq!(
        registration_fingerprint.value_hash,
        registration_summary.registration_interface_hash
    );
    assert!(registration_fingerprint.importer_visible);
}

fn request() -> DependencyFootprintRequest {
    DependencyFootprintRequest {
        schema_version: SchemaVersion::new(DEPENDENCY_FINGERPRINT_SCHEMA_VERSION),
        owner: FootprintOwner {
            package_id: "pkg".to_owned(),
            module_path: "pkg::A".to_owned(),
            origin_id: None,
            language_edition: Some("miz-2025".to_owned()),
            lockfile_identity: Some("lock-v1".to_owned()),
        },
        phase: PipelinePhase::new("vc"),
        fingerprints: vec![
            DependencyFingerprint::from_module_summary(&module_summary(hash(10))),
            DependencyFingerprint::from_registration_summary(&registration_summary(hash(20))),
        ],
        slices: vec![slice("vc-slice", "pkg::A", "obligation-1", hash(30))],
        compatibility_fields: vec![CompatibilityField {
            family: "toolchain".to_owned(),
            field_name: "rustc".to_owned(),
            value: "stable".to_owned(),
        }],
        proof_reuse_validation: vec![proof_reuse_validation()],
        unknown_markers: Vec::new(),
        requested_completeness: DependencyFootprintCompleteness::ConservativeComplete,
        uncacheable: false,
    }
}

fn reusable(request: DependencyFootprintRequest) -> DependencyFootprint {
    match DependencyFootprintBuilder::new(request).build() {
        DependencyFootprintBuildOutcome::Reusable(footprint) => footprint,
        outcome => panic!("expected reusable footprint, got {outcome:?}"),
    }
}

fn uncacheable(request: DependencyFootprintRequest) -> DependencyFootprint {
    match DependencyFootprintBuilder::new(request).build() {
        DependencyFootprintBuildOutcome::Uncacheable(footprint) => footprint,
        outcome => panic!("expected uncacheable footprint, got {outcome:?}"),
    }
}

fn assert_conflicting_duplicate(
    collection: &'static str,
    mutate: impl FnOnce(&mut DependencyFootprintRequest),
) {
    let mut request = request();
    mutate(&mut request);
    let footprint = uncacheable(request);

    assert_eq!(
        footprint.completeness,
        DependencyFootprintCompleteness::IncompleteUncacheable
    );
    assert!(footprint.uncacheable);
    assert!(
        footprint.unknown_markers.iter().any(|marker| {
            marker.family == "conflicting_duplicate" && marker.owner == collection
        }),
        "expected conflicting duplicate marker for {collection}, got {:?}",
        footprint.unknown_markers
    );
}

fn assert_marker_family(footprint: &DependencyFootprint, family: &str) {
    assert!(
        footprint
            .unknown_markers
            .iter()
            .any(|marker| marker.family == family),
        "expected unknown marker family {family}, got {:?}",
        footprint.unknown_markers
    );
}

fn trigger(
    change_kind: FingerprintChangeKind,
    target: FingerprintTargetKind,
) -> RebuildTriggerDecision {
    RebuildTriggerEvaluator::evaluate(trigger_input(
        change_kind,
        target,
        DependencySlicePrecision::Exact,
    ))
}

fn trigger_input(
    change_kind: FingerprintChangeKind,
    target: FingerprintTargetKind,
    slice_precision: DependencySlicePrecision,
) -> RebuildTriggerInput {
    RebuildTriggerInput {
        change_kind,
        target,
        dependent_phase: PipelinePhase::new("proof"),
        slice_precision,
    }
}

fn mutate(mut f: impl FnMut(&mut DependencyFootprintRequest)) -> DependencyFootprintRequest {
    let mut request = request();
    f(&mut request);
    request
}

fn importer_visible_fingerprint_hashes(footprint: &DependencyFootprint) -> Vec<Hash> {
    footprint
        .fingerprints
        .iter()
        .filter(|fingerprint| fingerprint.importer_visible)
        .map(|fingerprint| fingerprint.value_hash)
        .collect()
}

fn slices_by_name(footprint: &DependencyFootprint) -> BTreeMap<String, Hash> {
    footprint
        .slices
        .iter()
        .map(|slice| (slice.name.clone(), slice.digest))
        .collect()
}

fn slice(
    slice_kind: impl Into<String>,
    owner: impl Into<String>,
    name: impl Into<String>,
    digest: Hash,
) -> DependencySliceFingerprint {
    DependencySliceFingerprint {
        slice_kind: slice_kind.into(),
        owner: owner.into(),
        name: name.into(),
        domain: DEPENDENCY_SLICE_SCHEMA_VERSION.to_owned(),
        digest,
        completeness: DependencyFootprintCompleteness::Complete,
    }
}

fn proof_reuse_validation() -> ProofReuseValidationInput {
    ProofReuseValidationInput {
        name: "proof-reuse:obligation-1".to_owned(),
        state: ProofReuseValidationState::Complete,
        validation_hash: Some(NamedHash {
            name: "validation".to_owned(),
            domain: "mizar-proof/reuse-validation/v1".to_owned(),
            digest: hash(40),
        }),
        witness_or_discharge_hash: Some(NamedHash {
            name: "witness".to_owned(),
            domain: "mizar-proof/witness/v1".to_owned(),
            digest: hash(41),
        }),
        metadata_schema_versions: vec![NamedSchemaVersion {
            schema_family: "mizar-proof/reuse-metadata".to_owned(),
            name: "proof-reuse".to_owned(),
            version: SchemaVersion::new("mizar-proof/reuse-metadata/v1"),
        }],
    }
}

fn unknown_marker(
    family: impl Into<String>,
    owner: impl Into<String>,
    reason: impl Into<String>,
) -> UnknownDependencyMarker {
    UnknownDependencyMarker {
        family: family.into(),
        owner: owner.into(),
        reason: reason.into(),
    }
}

fn implementation_fingerprint(value_hash: Hash) -> DependencyFingerprint {
    DependencyFingerprint::module_implementation(
        FingerprintIdentity {
            package_id: "pkg".to_owned(),
            module_path: "pkg::A".to_owned(),
            origin_id: None,
            target_name: "module-implementation".to_owned(),
            schema_family: "mizar-artifact/module-implementation".to_owned(),
            language_edition: Some("miz-2025".to_owned()),
            lockfile_identity: Some("lock-v1".to_owned()),
        },
        "mizar-artifact/module-implementation",
        value_hash,
        SchemaVersion::new("mizar-artifact/module-implementation/v1"),
    )
}

fn module_summary(interface_hash: Hash) -> ModuleSummary {
    ModuleSummary {
        schema_version: current_schema_version(),
        module: module_identity(),
        source_hash: hash(1),
        interface_hash,
        exported_symbols: vec![ExportedSymbolSummary {
            origin_id: "th:1".to_owned(),
            fully_qualified_name: "pkg::A::T".to_owned(),
            namespace_path: vec!["pkg".to_owned(), "A".to_owned()],
            visibility: "public".to_owned(),
            declaration_kind: "theorem".to_owned(),
            source_range: source_range(),
            rendered_signature: "T: thesis".to_owned(),
            interface_fingerprint: hash(2),
            proof_status: Some(ProofStatusSummary::Accepted),
        }],
        exported_labels: vec![ExportedLabelSummary {
            origin_id: "label:1".to_owned(),
            label: "L1".to_owned(),
            owner_fully_qualified_name: "pkg::A::T".to_owned(),
            visibility: "public".to_owned(),
            source_range: source_range(),
            target_kind: "theorem".to_owned(),
        }],
        lexical_summary: ModuleLexicalSummary {
            schema_version: "lexical-v1".to_owned(),
            fingerprint: Some(hash(3)),
            contributions: vec![LexicalContributionSummary {
                kind: "notation".to_owned(),
                key: "foo".to_owned(),
                payload: "bar".to_owned(),
            }],
        },
        reexports: vec![ModuleReexportSummary {
            target_module: module_identity(),
            target_item_origin_id: Some("th:1".to_owned()),
            exported_name: Some("T".to_owned()),
            provenance_origin_id: Some("reexport:1".to_owned()),
        }],
        dependency_interfaces: vec![DependencyInterfaceRef {
            module: module_identity(),
            interface_hash: hash(4),
        }],
    }
}

fn registration_summary(registration_interface_hash: Hash) -> RegistrationSummary {
    RegistrationSummary {
        schema_version: mizar_artifact::registration_summary::current_schema_version(),
        module: module_identity(),
        source_hash: hash(5),
        registration_interface_hash,
        activated_registrations: vec![ActivatedRegistrationSummary {
            origin_id: "reg:1".to_owned(),
            label: Some("R1".to_owned()),
            registration_kind: RegistrationKind::Conditional,
            visibility: RegistrationVisibility::Public,
            namespace_path: vec!["pkg".to_owned(), "A".to_owned()],
            source_module: module_identity(),
            trigger_key: "cluster:foo".to_owned(),
            normalized_pattern: RegistrationPatternSummary {
                fingerprint: artifact_hash(hash(6)),
                type_head: Some("set".to_owned()),
                attribute: None,
                functor: None,
                term_head: None,
                parameters: vec!["x".to_owned()],
                guards: vec![artifact_hash(hash(7))],
            },
            generated_contribution: RegistrationContributionSummary {
                kind: RegistrationContributionKind::AttributeFact,
                summary: "cluster foo".to_owned(),
                fingerprint: artifact_hash(hash(8)),
            },
            accepted_status: RegistrationAcceptedStatus::Accepted,
            verifier_policy_fingerprint: artifact_hash(hash(9)),
            trace_ids: Vec::new(),
            source_range: Some(source_range()),
        }],
        trace_artifacts: Vec::new(),
        dependency_registrations: Vec::new(),
    }
}

fn artifact_hash(digest: Hash) -> ArtifactHashRef {
    ArtifactHashRef::new(
        ArtifactHashClass::Interface,
        "test-artifact",
        mizar_artifact::registration_summary::current_schema_version(),
        digest,
    )
}

fn module_identity() -> ModuleSummaryIdentity {
    ModuleSummaryIdentity {
        package_id: "pkg".to_owned(),
        package_version: Some("0.1.0".to_owned()),
        lockfile_identity: Some("lock-v1".to_owned()),
        module_path: "pkg::A".to_owned(),
        language_edition: "miz-2025".to_owned(),
    }
}

fn source_range() -> SourceRangeSummary {
    SourceRangeSummary {
        start_byte: 0,
        end_byte: 1,
    }
}

fn vc_fixture_set() -> VcSet {
    let snapshot = BuildSnapshotId::from_published_schema_str(
        "mizar-session-build-snapshot-v1:\
             2222222222222222222222222222222222222222222222222222222222222222",
    )
    .expect("snapshot id");
    let source = InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .expect("source id");
    let handoff = ObligationHandoffId::new(0);
    let source_ref = core_source_ref(source);

    VcSet::try_new(VcSetParts {
        schema_version: VcSchemaVersion::new("mizar-cache-task-5-test-v1"),
        snapshot,
        source,
        module: VcModuleRef::new("pkg::A"),
        generated_formulas: Vec::new(),
        vcs: vec![VcIr {
            id: VcId::new(0),
            kind: VcKind::TheoremProofStep,
            source: VcSourceRef {
                primary: source_ref.clone(),
                related: Vec::new(),
            },
            seed: SeedVcRef { handoff },
            anchor: complete_anchor(source),
            local_context: LocalContext::try_new(
                vec![ContextEntry {
                    id: ContextEntryId::new(0),
                    sort_key: CanonicalSortKey::new("000-assumption"),
                    kind: ContextEntryKind::ProofAssumption,
                    formula: Some(VcFormulaRef::Core(CoreFormulaId::new(1))),
                    provenance: vec![vc_provenance("context")],
                }],
                Vec::new(),
            )
            .expect("local context"),
            premises: vec![PremiseRef::LocalContext(ContextEntryId::new(0))],
            goal: VcFormulaRef::Core(CoreFormulaId::new(0)),
            proof_hint: None,
            status: VcStatus::NeedsAtp,
            provenance: vec![vc_provenance("vc")],
        }],
        seed_accounting: vec![SeedAccounting {
            handoff,
            origin: SeedOriginRef::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
            seed_status: ObligationSeedStatus::Active,
            mapping: SeedVcMapping::One { vc: VcId::new(0) },
        }],
    })
    .expect("valid VC fixture")
}

fn complete_anchor(source: mizar_session::SourceId) -> mizar_vc::vc_ir::ObligationAnchor {
    mizar_vc::vc_ir::ObligationAnchor {
        owner: AnchorOwner::Theorem(CoreItemId::new(0)),
        kind: VcKind::TheoremProofStep,
        local_path: LocalProofOrProgramPath::new("proof/step/0"),
        label: Some(AnchorLabel {
            role: AnchorLabelRole::UserLabel,
            hint: Some(CoreLabelRef::new("A1")),
        }),
        semantic_origin: NormalizedSemanticOrigin::new("theorem:sample:proof-step:0"),
        source_range: Some(SourceRange {
            source_id: source,
            start: 0,
            end: 10,
        }),
        provenance: vec![vc_provenance("anchor")],
        source_shape_hash: HashMarker::Available(hash(81)),
        canonical_goal_hash: HashMarker::Available(hash(82)),
        canonical_context_hash: HashMarker::Available(hash(83)),
        generation_schema_version: GenerationSchemaVersion::new("mizar-cache-task-5-test"),
        completeness: AnchorCompleteness::Complete,
    }
}

fn core_source_ref(source: mizar_session::SourceId) -> CoreSourceRef {
    CoreSourceRef::direct(SourceRange {
        source_id: source,
        start: 0,
        end: 10,
    })
    .with_provenance(vec![CoreProvenance::new(
        CoreProvenancePhase::ProofSkeleton,
        "direct-source",
    )])
}

fn vc_provenance(key: &str) -> VcProvenance {
    VcProvenance {
        phase: VcProvenancePhase::Generator,
        key: VcText::new(key),
        core: None,
    }
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}
