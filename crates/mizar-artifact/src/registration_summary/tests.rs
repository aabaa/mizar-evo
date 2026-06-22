use super::{
    ActivatedRegistrationSummary, ArtifactHashClass, ArtifactHashRef, DependencyRegistrationRef,
    REGISTRATION_SUMMARY_SCHEMA_FAMILY, RegistrationAcceptedStatus, RegistrationContributionKind,
    RegistrationContributionSummary, RegistrationKind, RegistrationPatternSummary,
    RegistrationSummary, RegistrationSummaryError, RegistrationSummaryReadOptions,
    RegistrationTraceArtifactRef, RegistrationTraceKind, RegistrationVisibility,
    SuppliedTraceArtifactRef, current_schema_version, read_registration_summary,
    registration_interface_projection_json, registration_summary_json,
    registration_summary_json_unchecked, write_registration_summary,
};
use crate::{
    module_summary::{ModuleSummaryIdentity, SourceRangeSummary},
    store::{
        CanonicalHashDomain, CanonicalJson, HashClass, SchemaVersion, SchemaVersionError,
        canonical_json_string,
    },
};
use mizar_session::Hash;

#[test]
fn registration_summary_round_trips_through_canonical_json() {
    let summary = sample_summary();
    let json = registration_summary_json(&summary).expect("canonical registration JSON");
    let bytes = write_registration_summary(&summary).expect("canonical registration summary bytes");

    assert_eq!(bytes, canonical_json_string(&json).into_bytes());
    assert_eq!(
        read_registration_summary(&json, RegistrationSummaryReadOptions::default())
            .expect("valid summary"),
        summary
    );
}

#[test]
fn registration_summary_writer_and_hash_are_deterministic_for_identical_inputs() {
    let summary = sample_summary();
    let first_json = registration_summary_json(&summary).expect("first canonical JSON");
    let first_bytes = write_registration_summary(&summary).expect("first canonical bytes");
    let first_projection = registration_interface_projection_json(&summary)
        .expect("first registration interface projection");
    let first_hash = summary
        .compute_registration_interface_hash()
        .expect("first hash");

    for _ in 0..3 {
        let json = registration_summary_json(&summary).expect("repeated canonical JSON");
        assert_eq!(json, first_json);
        assert_eq!(canonical_json_string(&json).into_bytes(), first_bytes);
        assert_eq!(
            write_registration_summary(&summary).expect("repeated canonical bytes"),
            first_bytes
        );
        assert_eq!(
            registration_interface_projection_json(&summary)
                .expect("repeated registration interface projection"),
            first_projection
        );
        assert_eq!(
            summary
                .compute_registration_interface_hash()
                .expect("repeated hash"),
            first_hash
        );
    }
}

#[test]
fn writer_sorts_collections_and_reader_rejects_unsorted_arrays() {
    let mut summary = sample_summary();
    summary.activated_registrations.reverse();
    summary.trace_artifacts.reverse();
    summary.dependency_registrations.reverse();
    summary.activated_registrations[0].trace_ids.reverse();
    summary.trace_artifacts[0]
        .used_by_registration_origin_ids
        .reverse();
    summary
        .refresh_registration_interface_hash()
        .expect("refresh hash");

    let json = registration_summary_json(&summary).expect("writer sorts collections");
    let text = canonical_json_string(&json);
    assert!(
        text.find("\"trigger_key\":\"alpha\"")
            .expect("alpha trigger")
            < text.find("\"trigger_key\":\"zeta\"").expect("zeta trigger")
    );
    assert!(
        text.find("\"trace_id\":\"cluster:alpha\"")
            .expect("cluster trace")
            < text
                .find("\"trace_id\":\"reduction:zeta\"")
                .expect("reduction trace")
    );

    assert_unsorted_collection_rejected(
        json.clone(),
        &["activated_registrations"],
        "$.activated_registrations",
    );
    assert_unsorted_collection_rejected(json.clone(), &["trace_artifacts"], "$.trace_artifacts");
    assert_unsorted_collection_rejected(
        json.clone(),
        &["dependency_registrations"],
        "$.dependency_registrations",
    );
    assert_unsorted_collection_rejected(
        json.clone(),
        &["activated_registrations", "trace_ids"],
        "$.activated_registrations[0].trace_ids",
    );
    assert_unsorted_collection_rejected(
        json,
        &["trace_artifacts", "used_by_registration_origin_ids"],
        "$.trace_artifacts[0].used_by_registration_origin_ids",
    );
}

#[test]
fn trace_references_resolve_by_hash_when_supplied() {
    let summary = sample_summary();
    let json = registration_summary_json(&summary).expect("canonical JSON");
    let cluster = &summary.trace_artifacts[0];
    let reduction = &summary.trace_artifacts[1];
    let supplied = [supplied_trace(cluster), supplied_trace(reduction)];

    assert!(
        read_registration_summary(
            &json,
            RegistrationSummaryReadOptions {
                supplied_trace_artifacts: &supplied,
                ..RegistrationSummaryReadOptions::default()
            },
        )
        .is_ok()
    );

    let bad_artifact_hash = hash_ref(
        ArtifactHashClass::Artifact,
        "mizar-artifact/resolution-trace",
        88,
    );
    let bad_supplied = [SuppliedTraceArtifactRef {
        trace_id: cluster.trace_id.as_str(),
        artifact_hash: &bad_artifact_hash,
        trace_replay_hash: &cluster.trace_replay_hash,
        diagnostic_hash: cluster.diagnostic_hash.as_ref(),
    }];

    assert!(matches!(
        read_registration_summary(
            &json,
            RegistrationSummaryReadOptions {
                supplied_trace_artifacts: &bad_supplied,
                ..RegistrationSummaryReadOptions::default()
            },
        ),
        Err(RegistrationSummaryError::SuppliedTraceArtifactMismatch {
            field: "artifact_hash",
            ..
        })
    ));

    let bad_trace_replay_hash = hash_ref(
        ArtifactHashClass::Interface,
        "mizar-artifact/resolution-trace",
        89,
    );
    let bad_supplied = [SuppliedTraceArtifactRef {
        trace_id: cluster.trace_id.as_str(),
        artifact_hash: &cluster.artifact_hash,
        trace_replay_hash: &bad_trace_replay_hash,
        diagnostic_hash: cluster.diagnostic_hash.as_ref(),
    }];

    assert!(matches!(
        read_registration_summary(
            &json,
            RegistrationSummaryReadOptions {
                supplied_trace_artifacts: &bad_supplied,
                ..RegistrationSummaryReadOptions::default()
            },
        ),
        Err(RegistrationSummaryError::SuppliedTraceArtifactMismatch {
            field: "trace_replay_hash",
            ..
        })
    ));

    let bad_diagnostic_hash = hash_ref(
        ArtifactHashClass::Diagnostic,
        "mizar-artifact/resolution-trace",
        90,
    );
    let bad_supplied = [SuppliedTraceArtifactRef {
        trace_id: cluster.trace_id.as_str(),
        artifact_hash: &cluster.artifact_hash,
        trace_replay_hash: &cluster.trace_replay_hash,
        diagnostic_hash: Some(&bad_diagnostic_hash),
    }];

    assert!(matches!(
        read_registration_summary(
            &json,
            RegistrationSummaryReadOptions {
                supplied_trace_artifacts: &bad_supplied,
                ..RegistrationSummaryReadOptions::default()
            },
        ),
        Err(RegistrationSummaryError::SuppliedTraceArtifactMismatch {
            field: "diagnostic_hash",
            ..
        })
    ));

    let bad_supplied = [SuppliedTraceArtifactRef {
        trace_id: cluster.trace_id.as_str(),
        artifact_hash: &cluster.artifact_hash,
        trace_replay_hash: &cluster.trace_replay_hash,
        diagnostic_hash: None,
    }];

    assert!(matches!(
        read_registration_summary(
            &json,
            RegistrationSummaryReadOptions {
                supplied_trace_artifacts: &bad_supplied,
                ..RegistrationSummaryReadOptions::default()
            },
        ),
        Err(RegistrationSummaryError::SuppliedTraceArtifactMismatch {
            field: "diagnostic_hash",
            ..
        })
    ));

    let duplicate_supplied = [supplied_trace(cluster), supplied_trace(cluster)];
    assert!(matches!(
        read_registration_summary(
            &json,
            RegistrationSummaryReadOptions {
                supplied_trace_artifacts: &duplicate_supplied,
                ..RegistrationSummaryReadOptions::default()
            },
        ),
        Err(RegistrationSummaryError::DuplicateEntry { path, .. })
            if path == "supplied_trace_artifacts"
    ));

    let unknown_supplied = [SuppliedTraceArtifactRef {
        trace_id: "trace:unknown",
        artifact_hash: &cluster.artifact_hash,
        trace_replay_hash: &cluster.trace_replay_hash,
        diagnostic_hash: cluster.diagnostic_hash.as_ref(),
    }];
    assert!(matches!(
        read_registration_summary(
            &json,
            RegistrationSummaryReadOptions {
                supplied_trace_artifacts: &unknown_supplied,
                ..RegistrationSummaryReadOptions::default()
            },
        ),
        Err(RegistrationSummaryError::UnknownSuppliedTraceArtifact { trace_id })
            if trace_id == "trace:unknown"
    ));
}

#[test]
fn registration_interface_hash_ignores_source_and_trace_byte_metadata() {
    let mut baseline = sample_summary();
    let baseline_hash = baseline
        .refresh_registration_interface_hash()
        .expect("baseline hash");
    let mut changed_metadata = baseline.clone();
    changed_metadata.source_hash = hash(99);
    changed_metadata.activated_registrations[0].source_range = Some(SourceRangeSummary {
        start_byte: 100,
        end_byte: 120,
    });
    changed_metadata.trace_artifacts[0].artifact_path = "traces/renamed.cluster.json".to_owned();
    changed_metadata.trace_artifacts[0].artifact_hash = hash_ref(
        ArtifactHashClass::Artifact,
        "mizar-artifact/resolution-trace",
        91,
    );
    changed_metadata.trace_artifacts[0].diagnostic_hash = Some(hash_ref(
        ArtifactHashClass::Diagnostic,
        "mizar-artifact/resolution-trace",
        92,
    ));

    assert_eq!(
        changed_metadata
            .refresh_registration_interface_hash()
            .expect("metadata-only hash"),
        baseline_hash
    );
}

#[test]
fn registration_interface_hash_uses_declared_registration_summary_domain() {
    let summary = sample_summary();
    let projection = registration_interface_projection_json(&summary).expect("projection");
    let expected = CanonicalHashDomain::new(
        HashClass::Interface,
        REGISTRATION_SUMMARY_SCHEMA_FAMILY,
        current_schema_version(),
    )
    .hash(&projection, &[]);

    assert_eq!(
        summary
            .compute_registration_interface_hash()
            .expect("summary hash"),
        expected
    );
    assert_ne!(
        CanonicalHashDomain::new(
            HashClass::Artifact,
            REGISTRATION_SUMMARY_SCHEMA_FAMILY,
            current_schema_version(),
        )
        .hash(&projection, &[]),
        expected
    );
    assert_ne!(
        CanonicalHashDomain::new(
            HashClass::Interface,
            "mizar-artifact/other-summary",
            current_schema_version(),
        )
        .hash(&projection, &[]),
        expected
    );
}

#[test]
fn registration_interface_hash_changes_for_importer_visible_projection_changes() {
    assert_registration_hash_changes("module identity", |summary| {
        summary.module.module_path = "Renamed.Hidden".to_owned();
    });
    assert_registration_hash_changes("registration contribution", |summary| {
        summary.activated_registrations[0]
            .generated_contribution
            .fingerprint = hash_ref(ArtifactHashClass::Interface, "mizar-artifact/checker", 51);
    });
    assert_registration_hash_changes("verifier policy", |summary| {
        summary.activated_registrations[0].verifier_policy_fingerprint = hash_ref(
            ArtifactHashClass::Interface,
            "mizar-artifact/verifier-policy",
            52,
        );
    });
    assert_registration_hash_changes("trace replay hash", |summary| {
        summary.trace_artifacts[0].trace_replay_hash = hash_ref(
            ArtifactHashClass::Interface,
            "mizar-artifact/resolution-trace",
            53,
        );
    });
    assert_registration_hash_changes("dependency registration", |summary| {
        summary.dependency_registrations[0].registration_interface_hash = hash(54);
    });
}

#[test]
fn incompatible_version_reads_fail_cleanly() {
    let summary = sample_summary();
    let mut json = registration_summary_json(&summary).expect("canonical JSON");
    replace_object_field(&mut json, "schema_version", CanonicalJson::string("2.0"));

    assert!(matches!(
        read_registration_summary(
            &json,
            RegistrationSummaryReadOptions {
                artifact_path: Some("build/registration-summary.json"),
                ..RegistrationSummaryReadOptions::default()
            },
        ),
        Err(RegistrationSummaryError::SchemaVersion(
            SchemaVersionError::MajorMismatch { .. }
        ))
    ));
}

#[test]
fn reader_rejects_hash_and_module_mismatches() {
    let summary = sample_summary();
    let json = registration_summary_json(&summary).expect("canonical JSON");
    let other_module = ModuleSummaryIdentity {
        package_id: "other-package".to_owned(),
        ..summary.module.clone()
    };
    assert!(matches!(
        read_registration_summary(
            &json,
            RegistrationSummaryReadOptions {
                expected_module: Some(&other_module),
                ..RegistrationSummaryReadOptions::default()
            }
        ),
        Err(RegistrationSummaryError::ModuleIdentityMismatch { .. })
    ));

    assert!(matches!(
        read_registration_summary(
            &json,
            RegistrationSummaryReadOptions {
                expected_registration_interface_hash: Some(hash(88)),
                ..RegistrationSummaryReadOptions::default()
            }
        ),
        Err(RegistrationSummaryError::ExpectedRegistrationInterfaceHashMismatch { .. })
    ));

    let mut bad_hash_json = json;
    replace_object_field(
        &mut bad_hash_json,
        "registration_interface_hash",
        CanonicalJson::string(format!(
            "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact/registration-summary:1.0:{}",
            "77".repeat(Hash::BYTE_LEN)
        )),
    );
    assert!(matches!(
        read_registration_summary(&bad_hash_json, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::RegistrationInterfaceHashMismatch { .. })
    ));
}

#[test]
fn reader_rejects_invalid_hash_domains_and_digests() {
    let summary = sample_summary();
    let json = registration_summary_json(&summary).expect("canonical JSON");
    let digest = "11".repeat(Hash::BYTE_LEN);

    for bad_hash in [
        format!(
            "mizar-artifact/other-framed-hash/v1:interface:mizar-artifact/registration-summary:1.0:{digest}"
        ),
        format!(
            "mizar-artifact/artifact-framed-hash-text/v1:artifact:mizar-artifact/registration-summary:1.0:{digest}"
        ),
        format!(
            "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact/other-summary:1.0:{digest}"
        ),
        format!(
            "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact/registration-summary:1.1:{digest}"
        ),
        format!(
            "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact/registration-summary:1.0:{}",
            "GG".repeat(Hash::BYTE_LEN)
        ),
    ] {
        let mut bad_json = json.clone();
        replace_object_field(
            &mut bad_json,
            "registration_interface_hash",
            CanonicalJson::string(bad_hash),
        );

        assert!(matches!(
            read_registration_summary(&bad_json, RegistrationSummaryReadOptions::default()),
            Err(RegistrationSummaryError::InvalidHash { path, .. })
                if path == "$.registration_interface_hash"
        ));
    }

    let mut bad_pattern_family = json;
    replace_nested_object_field(
        &mut bad_pattern_family,
        &["activated_registrations", "normalized_pattern"],
        "fingerprint",
        CanonicalJson::string(format!(
            "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact//bad:1.0:{digest}"
        )),
    );

    assert!(matches!(
        read_registration_summary(&bad_pattern_family, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::InvalidHash { path, .. })
            if path == "$.activated_registrations[0].normalized_pattern.fingerprint"
    ));
}

#[test]
fn reader_rejects_duplicates_and_broken_trace_cross_references() {
    let summary = sample_summary();
    let json = registration_summary_json(&summary).expect("canonical JSON");

    assert_duplicate_collection_rejected(
        json.clone(),
        &["activated_registrations"],
        "$.activated_registrations",
    );
    assert_duplicate_collection_rejected(json.clone(), &["trace_artifacts"], "$.trace_artifacts");
    assert_duplicate_collection_rejected(
        json.clone(),
        &["dependency_registrations"],
        "$.dependency_registrations",
    );
    assert_duplicate_collection_rejected(
        json.clone(),
        &["trace_artifacts", "used_by_registration_origin_ids"],
        "$.trace_artifacts[0].used_by_registration_origin_ids",
    );
    assert_duplicate_collection_rejected(
        json.clone(),
        &["activated_registrations", "trace_ids"],
        "$.activated_registrations[0].trace_ids",
    );

    let mut missing_trace = json.clone();
    remove_first_array_item_at(&mut missing_trace, &["trace_artifacts"]);
    assert!(matches!(
        read_registration_summary(&missing_trace, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::TraceReferenceMismatch { path, .. })
            if path == "$.trace_artifacts"
    ));

    let mut extra_trace_summary = sample_summary();
    extra_trace_summary.trace_artifacts.push(trace(
        "cluster:unused",
        RegistrationTraceKind::Cluster,
        &[],
        66,
    ));
    let extra_trace_json =
        registration_summary_json_unchecked(&extra_trace_summary).expect("unchecked JSON");
    assert!(matches!(
        read_registration_summary(&extra_trace_json, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::TraceReferenceMismatch { path, .. })
            if path == "$.trace_artifacts"
    ));

    let mut wrong_used_by = json;
    replace_nested_object_field(
        &mut wrong_used_by,
        &["trace_artifacts"],
        "used_by_registration_origin_ids",
        CanonicalJson::array([CanonicalJson::string("registration:missing")]),
    );
    assert!(matches!(
        read_registration_summary(&wrong_used_by, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::TraceReferenceMismatch { .. })
    ));
}

#[test]
fn reader_rejects_nested_missing_and_unknown_fields() {
    let summary = sample_summary();
    let json = registration_summary_json(&summary).expect("canonical JSON");

    let mut missing_registration_field = json.clone();
    remove_nested_object_field(
        &mut missing_registration_field,
        &["activated_registrations"],
        "trigger_key",
    );
    assert!(matches!(
        read_registration_summary(
            &missing_registration_field,
            RegistrationSummaryReadOptions::default()
        ),
        Err(RegistrationSummaryError::MissingField { path })
            if path == "$.activated_registrations[0].trigger_key"
    ));

    let mut unknown_registration_field = json.clone();
    replace_nested_object_field(
        &mut unknown_registration_field,
        &["activated_registrations"],
        "raw_checker_state",
        CanonicalJson::bool(true),
    );
    assert!(matches!(
        read_registration_summary(
            &unknown_registration_field,
            RegistrationSummaryReadOptions::default()
        ),
        Err(RegistrationSummaryError::UnknownField { path, field })
            if path == "$.activated_registrations[0]" && field == "raw_checker_state"
    ));

    let mut missing_pattern_field = json.clone();
    remove_nested_object_field(
        &mut missing_pattern_field,
        &["activated_registrations", "normalized_pattern"],
        "fingerprint",
    );
    assert!(matches!(
        read_registration_summary(&missing_pattern_field, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::MissingField { path })
            if path == "$.activated_registrations[0].normalized_pattern.fingerprint"
    ));

    let mut unknown_pattern_field = json.clone();
    replace_nested_object_field(
        &mut unknown_pattern_field,
        &["activated_registrations", "normalized_pattern"],
        "raw_trace",
        CanonicalJson::bool(true),
    );
    assert!(matches!(
        read_registration_summary(&unknown_pattern_field, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::UnknownField { path, field })
            if path == "$.activated_registrations[0].normalized_pattern" && field == "raw_trace"
    ));

    let mut missing_trace_field = json.clone();
    remove_nested_object_field(&mut missing_trace_field, &["trace_artifacts"], "trace_kind");
    assert!(matches!(
        read_registration_summary(&missing_trace_field, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::MissingField { path })
            if path == "$.trace_artifacts[0].trace_kind"
    ));

    let mut unknown_trace_field = json.clone();
    replace_nested_object_field(
        &mut unknown_trace_field,
        &["trace_artifacts"],
        "scheduler_state",
        CanonicalJson::bool(true),
    );
    assert!(matches!(
        read_registration_summary(&unknown_trace_field, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::UnknownField { path, field })
            if path == "$.trace_artifacts[0]" && field == "scheduler_state"
    ));

    let mut missing_dependency_field = json.clone();
    remove_nested_object_field(
        &mut missing_dependency_field,
        &["dependency_registrations"],
        "registration_interface_hash",
    );
    assert!(matches!(
        read_registration_summary(&missing_dependency_field, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::MissingField { path })
            if path == "$.dependency_registrations[0].registration_interface_hash"
    ));

    let mut unknown_dependency_field = json;
    replace_nested_object_field(
        &mut unknown_dependency_field,
        &["dependency_registrations"],
        "cache_record",
        CanonicalJson::bool(true),
    );
    assert!(matches!(
        read_registration_summary(&unknown_dependency_field, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::UnknownField { path, field })
            if path == "$.dependency_registrations[0]" && field == "cache_record"
    ));
}

#[test]
fn reader_rejects_unaccepted_private_missing_and_unknown_fields() {
    let summary = sample_summary();
    let json = registration_summary_json(&summary).expect("canonical JSON");

    let mut unaccepted = json.clone();
    replace_nested_object_field(
        &mut unaccepted,
        &["activated_registrations"],
        "accepted_status",
        CanonicalJson::string("pending"),
    );
    assert!(matches!(
        read_registration_summary(&unaccepted, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::InvalidField { path, .. })
            if path == "$.activated_registrations[0].accepted_status"
    ));

    let mut private = json.clone();
    replace_nested_object_field(
        &mut private,
        &["activated_registrations"],
        "visibility",
        CanonicalJson::string("private"),
    );
    assert!(matches!(
        read_registration_summary(&private, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::InvalidField { path, .. })
            if path == "$.activated_registrations[0].visibility"
    ));

    let mut missing = json.clone();
    remove_object_field(&mut missing, "source_hash");
    assert!(matches!(
        read_registration_summary(&missing, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::MissingField { path }) if path == "$.source_hash"
    ));

    let mut unknown = json;
    insert_object_field(&mut unknown, "cache_record", CanonicalJson::bool(true));
    assert!(matches!(
        read_registration_summary(&unknown, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::UnknownField { path, field })
            if path == "$" && field == "cache_record"
    ));
}

fn sample_summary() -> RegistrationSummary {
    let module = identity("base", "Hidden");
    let dep_alpha = identity("dep-alpha", "AlphaDep");
    let dep_zeta = identity("dep-zeta", "ZetaDep");
    let mut zeta_registration = registration(
        "registration:zeta",
        RegistrationKind::Reduction,
        "zeta",
        "reduction:zeta",
        20,
    );
    zeta_registration
        .trace_ids
        .push("cluster:000-shared".to_owned());
    let mut alpha_registration = registration(
        "registration:alpha",
        RegistrationKind::Conditional,
        "alpha",
        "cluster:alpha",
        0,
    );
    alpha_registration
        .trace_ids
        .push("cluster:000-shared".to_owned());
    let mut summary = RegistrationSummary {
        schema_version: current_schema_version(),
        module: module.clone(),
        source_hash: hash(1),
        registration_interface_hash: hash(0),
        activated_registrations: vec![zeta_registration, alpha_registration],
        trace_artifacts: vec![
            trace(
                "reduction:zeta",
                RegistrationTraceKind::Reduction,
                &["registration:zeta"],
                6,
            ),
            trace(
                "cluster:alpha",
                RegistrationTraceKind::Cluster,
                &["registration:alpha"],
                7,
            ),
            trace(
                "cluster:000-shared",
                RegistrationTraceKind::Cluster,
                &["registration:zeta", "registration:alpha"],
                8,
            ),
        ],
        dependency_registrations: vec![
            DependencyRegistrationRef {
                module: dep_zeta,
                registration_interface_hash: hash(8),
            },
            DependencyRegistrationRef {
                module: dep_alpha,
                registration_interface_hash: hash(9),
            },
        ],
    };
    summary
        .refresh_registration_interface_hash()
        .expect("sample hash");
    registration_summary_json(&summary).expect("sample is canonical");
    read_registration_summary(
        &registration_summary_json(&summary).expect("sample JSON"),
        RegistrationSummaryReadOptions::default(),
    )
    .expect("sample round-trip")
}

fn identity(package_id: &str, module_path: &str) -> ModuleSummaryIdentity {
    ModuleSummaryIdentity {
        package_id: package_id.to_owned(),
        package_version: Some("1.0.0".to_owned()),
        lockfile_identity: Some("lock:fixture".to_owned()),
        module_path: module_path.to_owned(),
        language_edition: "2026".to_owned(),
    }
}

fn registration(
    origin_id: &str,
    kind: RegistrationKind,
    trigger_key: &str,
    trace_id: &str,
    start_byte: u64,
) -> ActivatedRegistrationSummary {
    ActivatedRegistrationSummary {
        origin_id: origin_id.to_owned(),
        label: Some(format!("label:{trigger_key}")),
        registration_kind: kind,
        visibility: RegistrationVisibility::Public,
        namespace_path: vec!["Hidden".to_owned()],
        source_module: identity("base", "Hidden"),
        trigger_key: trigger_key.to_owned(),
        normalized_pattern: RegistrationPatternSummary {
            fingerprint: hash_ref(ArtifactHashClass::Interface, "mizar-artifact/checker", 2),
            type_head: Some("Hidden.Type".to_owned()),
            attribute: Some("non_empty".to_owned()),
            functor: None,
            term_head: None,
            parameters: vec!["T".to_owned()],
            guards: vec![hash_ref(
                ArtifactHashClass::Interface,
                "mizar-artifact/checker",
                3,
            )],
        },
        generated_contribution: RegistrationContributionSummary {
            kind: if kind == RegistrationKind::Reduction {
                RegistrationContributionKind::ReductionRule
            } else {
                RegistrationContributionKind::AttributeFact
            },
            summary: format!("generated contribution for {trigger_key}"),
            fingerprint: hash_ref(ArtifactHashClass::Interface, "mizar-artifact/checker", 4),
        },
        accepted_status: RegistrationAcceptedStatus::Accepted,
        verifier_policy_fingerprint: hash_ref(
            ArtifactHashClass::Interface,
            "mizar-artifact/verifier-policy",
            5,
        ),
        trace_ids: vec![trace_id.to_owned()],
        source_range: Some(SourceRangeSummary {
            start_byte,
            end_byte: start_byte + 10,
        }),
    }
}

fn trace(
    trace_id: &str,
    trace_kind: RegistrationTraceKind,
    used_by_origin_ids: &[&str],
    seed: u8,
) -> RegistrationTraceArtifactRef {
    RegistrationTraceArtifactRef {
        trace_id: trace_id.to_owned(),
        trace_kind,
        artifact_path: format!("traces/{trace_id}.json"),
        artifact_hash: hash_ref(
            ArtifactHashClass::Artifact,
            "mizar-artifact/resolution-trace",
            seed,
        ),
        trace_replay_hash: hash_ref(
            ArtifactHashClass::Interface,
            "mizar-artifact/resolution-trace",
            seed + 10,
        ),
        diagnostic_hash: Some(hash_ref(
            ArtifactHashClass::Diagnostic,
            "mizar-artifact/resolution-trace",
            seed + 20,
        )),
        used_by_registration_origin_ids: used_by_origin_ids
            .iter()
            .map(|origin_id| (*origin_id).to_owned())
            .collect(),
    }
}

fn supplied_trace(trace: &RegistrationTraceArtifactRef) -> SuppliedTraceArtifactRef<'_> {
    SuppliedTraceArtifactRef {
        trace_id: trace.trace_id.as_str(),
        artifact_hash: &trace.artifact_hash,
        trace_replay_hash: &trace.trace_replay_hash,
        diagnostic_hash: trace.diagnostic_hash.as_ref(),
    }
}

fn hash_ref(class: ArtifactHashClass, schema_family: &str, seed: u8) -> ArtifactHashRef {
    ArtifactHashRef::new(class, schema_family, SchemaVersion::new(1, 0), hash(seed))
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

fn assert_registration_hash_changes(name: &str, mutate: impl FnOnce(&mut RegistrationSummary)) {
    let baseline = sample_summary();
    let baseline_hash = baseline.registration_interface_hash;
    let mut changed = baseline.clone();
    mutate(&mut changed);

    assert_ne!(
        changed
            .refresh_registration_interface_hash()
            .unwrap_or_else(|error| panic!("{name}: {error}")),
        baseline_hash,
        "{name}"
    );
}

fn assert_unsorted_collection_rejected(
    mut json: CanonicalJson,
    fields: &[&str],
    expected_path: &str,
) {
    reverse_array_at(&mut json, fields);

    assert!(matches!(
        read_registration_summary(&json, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::UnsortedCollection { path }) if path == expected_path
    ));
}

fn assert_duplicate_collection_rejected(
    mut json: CanonicalJson,
    fields: &[&str],
    expected_path: &str,
) {
    duplicate_first_array_item_at(&mut json, fields);

    assert!(matches!(
        read_registration_summary(&json, RegistrationSummaryReadOptions::default()),
        Err(RegistrationSummaryError::DuplicateEntry { path, .. }) if path == expected_path
    ));
}

fn replace_object_field(value: &mut CanonicalJson, field: &str, replacement: CanonicalJson) {
    let CanonicalJson::Object(fields) = value else {
        panic!("expected object");
    };
    fields.insert(field.to_owned(), replacement);
}

fn insert_object_field(value: &mut CanonicalJson, field: &str, replacement: CanonicalJson) {
    replace_object_field(value, field, replacement);
}

fn remove_object_field(value: &mut CanonicalJson, field: &str) {
    let CanonicalJson::Object(fields) = value else {
        panic!("expected object");
    };
    fields.remove(field).expect("object field");
}

fn remove_nested_object_field(value: &mut CanonicalJson, fields: &[&str], field: &str) {
    let object = first_object_at_mut(value, fields);
    remove_object_field(object, field);
}

fn replace_nested_object_field(
    value: &mut CanonicalJson,
    fields: &[&str],
    field: &str,
    replacement: CanonicalJson,
) {
    let object = first_object_at_mut(value, fields);
    replace_object_field(object, field, replacement);
}

fn reverse_array_at(value: &mut CanonicalJson, fields: &[&str]) {
    let values = array_at_mut(value, fields);
    values.reverse();
}

fn duplicate_first_array_item_at(value: &mut CanonicalJson, fields: &[&str]) {
    let values = array_at_mut(value, fields);
    assert!(
        values.len() >= 2,
        "fixture array must contain at least two entries"
    );
    values[1] = values[0].clone();
}

fn remove_first_array_item_at(value: &mut CanonicalJson, fields: &[&str]) {
    let values = array_at_mut(value, fields);
    values.remove(0);
}

fn array_at_mut<'a>(value: &'a mut CanonicalJson, fields: &[&str]) -> &'a mut Vec<CanonicalJson> {
    let (last, parents) = fields.split_last().expect("field path must be non-empty");
    let mut current = value;
    for field in parents {
        current = first_object_at_mut(current, &[*field]);
    }
    let CanonicalJson::Array(values) = object_field_mut(current, last) else {
        panic!("expected array");
    };
    values
}

fn first_object_at_mut<'a>(value: &'a mut CanonicalJson, fields: &[&str]) -> &'a mut CanonicalJson {
    let mut current = value;
    for field in fields {
        current = object_field_mut(current, field);
        if let CanonicalJson::Array(values) = current {
            current = values.first_mut().expect("array item");
        }
    }
    current
}

fn object_field_mut<'a>(value: &'a mut CanonicalJson, field: &str) -> &'a mut CanonicalJson {
    let CanonicalJson::Object(fields) = value else {
        panic!("expected object");
    };
    fields.get_mut(field).expect("object field")
}
