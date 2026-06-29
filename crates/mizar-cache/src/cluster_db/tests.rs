use super::*;

#[test]
fn accepted_contribution_writes_origin_and_indexes() {
    let mut db = ClusterDbIndex::new();
    let report = db
        .apply_module_update(
            "pkg",
            "alpha",
            vec![record(
                "origin-a",
                0,
                vec![entry(ClusterIndexEntryKind::Graph, "T", "A")],
            )],
        )
        .expect("accepted contribution is inserted");

    assert_eq!(report.inserted_origins, ["origin-a"]);
    assert_eq!(report.rebuilt_origins, ["origin-a"]);

    let snapshot = db.snapshot();
    assert_eq!(snapshot.origins.len(), 1);
    assert_eq!(snapshot.indexes.graph_rows.len(), 1);
    assert_eq!(snapshot.indexes.graph_rows[0].origin_key, "origin-a");
    assert!(snapshot.indexes.attr_index_rows.is_empty());
}

#[test]
fn accepted_contribution_populates_every_aggregate_index_kind() {
    let mut db = ClusterDbIndex::new();
    db.apply_module_update(
        "pkg",
        "alpha",
        vec![record(
            "origin-all",
            0,
            vec![
                entry(ClusterIndexEntryKind::Graph, "type", "graph"),
                entry(ClusterIndexEntryKind::SubsumptionDag, "symbol", "edge"),
                entry(ClusterIndexEntryKind::Attribute, "attr", "producer"),
                entry(ClusterIndexEntryKind::Type, "mode", "trigger"),
                entry(ClusterIndexEntryKind::Reduction, "lhs", "rule"),
            ],
        )],
    )
    .expect("accepted contribution is inserted into all indexes");

    let snapshot = db.snapshot();
    assert_eq!(snapshot.indexes.graph_rows.len(), 1);
    assert_eq!(snapshot.indexes.subsumption_dag_rows.len(), 1);
    assert_eq!(snapshot.indexes.attr_index_rows.len(), 1);
    assert_eq!(snapshot.indexes.type_index_rows.len(), 1);
    assert_eq!(snapshot.indexes.reduction_index_rows.len(), 1);
}

#[test]
fn non_visible_or_unaccepted_contributions_are_rejected() {
    for (label, mutate) in [
        (
            "private",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.accepted_visibility = ClusterContributionVisibility::Private;
            }) as Box<dyn Fn(&mut ClusterContributionRecord)>,
        ),
        (
            "local-only",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.accepted_visibility = ClusterContributionVisibility::LocalOnly;
            }),
        ),
        (
            "pending",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.accepted_status = ClusterContributionStatus::Pending;
            }),
        ),
        (
            "rejected",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.accepted_status = ClusterContributionStatus::Rejected;
            }),
        ),
        (
            "recovered",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.accepted_status = ClusterContributionStatus::Recovered;
            }),
        ),
        (
            "externally-attested",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.accepted_status = ClusterContributionStatus::ExternallyAttested;
            }),
        ),
        (
            "uncacheable",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.uncacheable = true;
            }),
        ),
    ] {
        let mut db = ClusterDbIndex::new();
        let mut candidate = record(
            &format!("origin-{label}"),
            0,
            vec![entry(ClusterIndexEntryKind::Attribute, "attr", label)],
        );
        mutate(&mut candidate);

        assert!(
            db.apply_module_update("pkg", "alpha", vec![candidate])
                .is_err(),
            "{label} contribution must not enter visible cluster-db indexes"
        );
        assert!(db.snapshot().origins.is_empty());
    }
}

#[test]
fn incomplete_origin_metadata_forces_rejection() {
    for (label, mutate) in [
        (
            "incomplete-footprint",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.footprint_completeness =
                    ClusterOriginFootprintCompleteness::IncompleteUncacheable;
            }) as Box<dyn Fn(&mut ClusterContributionRecord)>,
        ),
        (
            "missing-footprint-hash",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.origin_footprint_hash = None;
            }),
        ),
        (
            "missing-dependency-interface-hashes",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.dependency_interface_hashes = None;
            }),
        ),
        (
            "missing-trace-replay-hashes",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.trace_replay_hashes = None;
            }),
        ),
        (
            "missing-proof-identity",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.proof_backed = true;
                record.origin.accepted_witness_or_discharge_hash = None;
            }),
        ),
        (
            "missing-producer-schema",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.producer_schema_versions.clear();
            }),
        ),
        (
            "blank-origin-key",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.origin_key = " ".to_owned();
            }),
        ),
        (
            "blank-package-id",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.package_id = " ".to_owned();
            }),
        ),
        (
            "blank-module-path",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.module_path = " ".to_owned();
            }),
        ),
        (
            "blank-stable-contribution-id",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.stable_contribution_id = " ".to_owned();
            }),
        ),
        (
            "blank-label",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.label = " ".to_owned();
            }),
        ),
        (
            "blank-trace-hash-name",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.trace_replay_hashes = Some(vec![NamedHash {
                    name: String::new(),
                    domain: "test-domain/trace".to_owned(),
                    digest: hash(21),
                }]);
            }),
        ),
        (
            "blank-schema-family",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.producer_schema_versions = vec![NamedSchemaVersion {
                    schema_family: String::new(),
                    name: "registration-summary".to_owned(),
                    version: SchemaVersion::new("1.0"),
                }];
            }),
        ),
        (
            "unsupported-producer-schema",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.producer_schema_versions = vec![NamedSchemaVersion {
                    schema_family: "mizar-artifact/registration-summary".to_owned(),
                    name: "registration-summary".to_owned(),
                    version: SchemaVersion::new("unknown"),
                }];
            }),
        ),
        (
            "blank-proof-identity-name",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.accepted_witness_or_discharge_hash = Some(NamedHash {
                    name: String::new(),
                    domain: "test-domain/witness".to_owned(),
                    digest: hash(22),
                });
            }),
        ),
    ] {
        let mut db = ClusterDbIndex::new();
        let mut candidate = record(
            &format!("origin-{label}"),
            0,
            vec![entry(ClusterIndexEntryKind::Type, "type", label)],
        );
        mutate(&mut candidate);

        assert!(
            db.apply_module_update("pkg", "alpha", vec![candidate])
                .is_err(),
            "{label} must be rejected before visible indexing"
        );
        assert!(db.snapshot().origins.is_empty());
    }
}

#[test]
fn unknown_schema_or_toolchain_compatibility_forces_rejection() {
    for (label, mutate) in [
        (
            "missing-policy-compatibility",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.policy_compatibility.clear();
            }) as Box<dyn Fn(&mut ClusterContributionRecord)>,
        ),
        (
            "unknown-policy-compatibility",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.policy_compatibility[0].value = "unknown".to_owned();
            }),
        ),
        (
            "missing-schema-compatibility",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.schema_compatibility.clear();
            }),
        ),
        (
            "unknown-schema-compatibility",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.schema_compatibility[0].value = "unknown".to_owned();
            }),
        ),
        (
            "missing-toolchain-compatibility",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.toolchain_compatibility.clear();
            }),
        ),
        (
            "unsupported-toolchain-compatibility",
            Box::new(|record: &mut ClusterContributionRecord| {
                record.origin.toolchain_compatibility[0].value = "unsupported".to_owned();
            }),
        ),
    ] {
        let mut db = ClusterDbIndex::new();
        let mut candidate = record(
            &format!("origin-{label}"),
            0,
            vec![entry(ClusterIndexEntryKind::Type, "type", label)],
        );
        mutate(&mut candidate);

        assert!(matches!(
            db.apply_module_update("pkg", "alpha", vec![candidate]),
            Err(ClusterDbWriteRejection::UnknownCompatibility { .. })
        ));
        assert!(db.snapshot().origins.is_empty());
    }
}

#[test]
fn rename_or_removal_cleans_stale_origins_before_reuse() {
    let mut db = ClusterDbIndex::new();
    db.apply_module_update(
        "pkg",
        "alpha",
        vec![record(
            "old-origin",
            0,
            vec![entry(ClusterIndexEntryKind::Attribute, "attr", "old")],
        )],
    )
    .expect("initial insert");

    let report = db
        .apply_module_update(
            "pkg",
            "alpha",
            vec![record(
                "new-origin",
                1,
                vec![entry(ClusterIndexEntryKind::Attribute, "attr", "new")],
            )],
        )
        .expect("rename replaces module origins");

    assert_eq!(report.removed_origins, ["old-origin"]);
    assert_eq!(report.inserted_origins, ["new-origin"]);
    assert_eq!(report.rebuilt_origins, ["new-origin", "old-origin"]);
    assert!(db.origin("old-origin").is_none());
    assert!(db.origin("new-origin").is_some());

    let rows = db.snapshot().indexes.attr_index_rows;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].origin_key, "new-origin");
    assert_eq!(rows[0].secondary_key, "new");
}

#[test]
fn rebuild_report_touches_only_changed_origins() {
    let mut db = ClusterDbIndex::new();
    db.apply_module_update(
        "pkg",
        "alpha",
        vec![
            record(
                "stable-origin",
                0,
                vec![entry(ClusterIndexEntryKind::Graph, "T", "stable")],
            ),
            record(
                "changed-origin",
                1,
                vec![entry(ClusterIndexEntryKind::Reduction, "f", "old-rule")],
            ),
        ],
    )
    .expect("initial insert");

    let report = db
        .apply_module_update(
            "pkg",
            "alpha",
            vec![
                record(
                    "stable-origin",
                    0,
                    vec![entry(ClusterIndexEntryKind::Graph, "T", "stable")],
                ),
                record(
                    "changed-origin",
                    1,
                    vec![entry(ClusterIndexEntryKind::Reduction, "f", "new-rule")],
                ),
            ],
        )
        .expect("changed origin replaced");

    assert_eq!(report.unchanged_origins, ["stable-origin"]);
    assert_eq!(report.replaced_origins, ["changed-origin"]);
    assert_eq!(report.rebuilt_origins, ["changed-origin"]);

    let snapshot = db.snapshot();
    assert_eq!(snapshot.indexes.graph_rows[0].origin_key, "stable-origin");
    assert_eq!(snapshot.indexes.reduction_index_rows.len(), 1);
    assert_eq!(
        snapshot.indexes.reduction_index_rows[0].secondary_key,
        "new-rule"
    );
}

#[test]
fn deterministic_ordering_is_independent_of_write_order() {
    let first = record(
        "b-origin",
        1,
        vec![
            entry(ClusterIndexEntryKind::Type, "T", "b"),
            entry(ClusterIndexEntryKind::SubsumptionDag, "root", "b"),
        ],
    );
    let second = record(
        "a-origin",
        0,
        vec![
            entry(ClusterIndexEntryKind::Attribute, "A", "a"),
            entry(ClusterIndexEntryKind::Graph, "T", "a"),
        ],
    );

    let mut left = ClusterDbIndex::new();
    left.apply_module_update("pkg", "alpha", vec![first.clone(), second.clone()])
        .expect("left update");

    let mut right = ClusterDbIndex::new();
    right
        .apply_module_update("pkg", "alpha", vec![second, first])
        .expect("right update");

    assert_eq!(left.snapshot(), right.snapshot());
    assert_eq!(left.snapshot().origins[0].origin.origin_key, "a-origin");
    assert_eq!(left.snapshot().origins[1].origin.origin_key, "b-origin");
}

#[test]
fn deterministic_ordering_sorts_multiple_rows_in_each_index_bucket() {
    let mut db = ClusterDbIndex::new();
    db.apply_module_update(
        "pkg",
        "alpha",
        vec![
            record(
                "z-origin",
                2,
                vec![
                    entry(ClusterIndexEntryKind::Graph, "T", "z"),
                    entry(ClusterIndexEntryKind::SubsumptionDag, "sym", "z"),
                    entry(ClusterIndexEntryKind::Attribute, "attr", "z"),
                    entry(ClusterIndexEntryKind::Type, "type", "z"),
                    entry(ClusterIndexEntryKind::Reduction, "lhs", "z"),
                ],
            ),
            record(
                "a-origin",
                0,
                vec![
                    entry(ClusterIndexEntryKind::Graph, "T", "a"),
                    entry(ClusterIndexEntryKind::SubsumptionDag, "sym", "a"),
                    entry(ClusterIndexEntryKind::Attribute, "attr", "a"),
                    entry(ClusterIndexEntryKind::Type, "type", "a"),
                    entry(ClusterIndexEntryKind::Reduction, "lhs", "a"),
                ],
            ),
        ],
    )
    .expect("multi-row insertion succeeds");

    let snapshot = db.snapshot();
    assert_eq!(
        origin_keys(&snapshot.indexes.graph_rows),
        vec!["a-origin", "z-origin"]
    );
    assert_eq!(
        origin_keys(&snapshot.indexes.subsumption_dag_rows),
        vec!["a-origin", "z-origin"]
    );
    assert_eq!(
        origin_keys(&snapshot.indexes.attr_index_rows),
        vec!["a-origin", "z-origin"]
    );
    assert_eq!(
        origin_keys(&snapshot.indexes.type_index_rows),
        vec!["a-origin", "z-origin"]
    );
    assert_eq!(
        origin_keys(&snapshot.indexes.reduction_index_rows),
        vec!["a-origin", "z-origin"]
    );
}

#[test]
fn duplicate_conflicting_origin_rejects_without_mutation() {
    let mut db = ClusterDbIndex::new();
    db.apply_module_update(
        "pkg",
        "alpha",
        vec![record(
            "existing",
            0,
            vec![entry(ClusterIndexEntryKind::Graph, "T", "old")],
        )],
    )
    .expect("initial insert");
    let before = db.snapshot();

    let first = record(
        "dup",
        1,
        vec![entry(ClusterIndexEntryKind::Attribute, "attr", "one")],
    );
    let second = record(
        "dup",
        1,
        vec![entry(ClusterIndexEntryKind::Attribute, "attr", "two")],
    );

    let result = db.apply_module_update("pkg", "alpha", vec![first, second]);

    assert!(matches!(
        result,
        Err(ClusterDbWriteRejection::ConflictingDuplicateOrigin { .. })
    ));
    assert_eq!(db.snapshot(), before);
}

#[test]
fn cross_module_origin_key_collision_rejects_without_mutation() {
    let mut db = ClusterDbIndex::new();
    db.apply_module_update(
        "pkg",
        "alpha",
        vec![record(
            "shared-origin",
            0,
            vec![entry(ClusterIndexEntryKind::Graph, "T", "alpha")],
        )],
    )
    .expect("initial insert");
    let before = db.snapshot();

    let mut colliding = record(
        "shared-origin",
        0,
        vec![entry(ClusterIndexEntryKind::Graph, "T", "beta")],
    );
    colliding.origin.module_path = "beta".to_owned();

    let result = db.apply_module_update("pkg", "beta", vec![colliding]);

    assert!(matches!(
        result,
        Err(ClusterDbWriteRejection::OriginKeyCollision { .. })
    ));
    assert_eq!(db.snapshot(), before);
}

#[test]
fn identical_duplicate_origin_is_coalesced() {
    let mut db = ClusterDbIndex::new();
    let candidate = record(
        "dup",
        0,
        vec![entry(ClusterIndexEntryKind::Attribute, "attr", "same")],
    );

    let report = db
        .apply_module_update("pkg", "alpha", vec![candidate.clone(), candidate])
        .expect("identical duplicate coalesces");

    assert_eq!(report.inserted_origins, ["dup"]);
    assert_eq!(db.snapshot().origins.len(), 1);
}

#[test]
fn import_scoped_view_filters_to_visible_origins() {
    let mut db = ClusterDbIndex::new();
    db.apply_module_update(
        "pkg",
        "alpha",
        vec![record(
            "origin-a",
            0,
            vec![
                entry(ClusterIndexEntryKind::Graph, "type", "a-graph"),
                entry(ClusterIndexEntryKind::SubsumptionDag, "symbol", "a-edge"),
                entry(ClusterIndexEntryKind::Attribute, "attr", "a-attr"),
                entry(ClusterIndexEntryKind::Type, "mode", "a-type"),
                entry(ClusterIndexEntryKind::Reduction, "lhs", "a-rule"),
            ],
        )],
    )
    .expect("origin a inserted");
    db.apply_module_update(
        "pkg",
        "beta",
        vec![record_for(
            "origin-b",
            "pkg",
            "beta",
            0,
            vec![
                entry(ClusterIndexEntryKind::Graph, "type", "b-graph"),
                entry(ClusterIndexEntryKind::SubsumptionDag, "symbol", "b-edge"),
                entry(ClusterIndexEntryKind::Attribute, "attr", "b-attr"),
                entry(ClusterIndexEntryKind::Type, "mode", "b-type"),
                entry(ClusterIndexEntryKind::Reduction, "lhs", "b-rule"),
            ],
        )],
    )
    .expect("origin b inserted");

    let view = db
        .import_scoped_view(view_request(&["origin-a"]))
        .expect("visible origin view materializes");

    assert_eq!(view.visible_origin_keys, ["origin-a"]);
    assert_eq!(origin_keys(&view.indexes.graph_rows), ["origin-a"]);
    assert_eq!(
        origin_keys(&view.indexes.subsumption_dag_rows),
        ["origin-a"]
    );
    assert_eq!(origin_keys(&view.indexes.attr_index_rows), ["origin-a"]);
    assert_eq!(origin_keys(&view.indexes.type_index_rows), ["origin-a"]);
    assert_eq!(
        origin_keys(&view.indexes.reduction_index_rows),
        ["origin-a"]
    );
}

#[test]
fn import_scoped_view_reuses_across_unrelated_origin_changes() {
    let mut db = ClusterDbIndex::new();
    db.apply_module_update(
        "pkg",
        "alpha",
        vec![record(
            "origin-a",
            0,
            vec![entry(ClusterIndexEntryKind::Attribute, "attr", "a")],
        )],
    )
    .expect("origin a inserted");
    db.apply_module_update(
        "pkg",
        "beta",
        vec![record_for(
            "origin-b",
            "pkg",
            "beta",
            0,
            vec![entry(ClusterIndexEntryKind::Reduction, "lhs", "old")],
        )],
    )
    .expect("origin b inserted");
    let before = db
        .import_scoped_view(view_request(&["origin-a"]))
        .expect("origin a view before unrelated change");

    db.apply_module_update(
        "pkg",
        "beta",
        vec![record_for(
            "origin-b",
            "pkg",
            "beta",
            0,
            vec![entry(ClusterIndexEntryKind::Reduction, "lhs", "new")],
        )],
    )
    .expect("unrelated origin b changed");

    let after = db
        .import_scoped_view(view_request(&["origin-a"]))
        .expect("origin a view after unrelated change");
    assert_eq!(after, before);
}

#[test]
fn visible_origin_change_invalidates_exactly_affected_views() {
    let mut db = ClusterDbIndex::new();
    db.apply_module_update(
        "pkg",
        "alpha",
        vec![record(
            "origin-a",
            0,
            vec![entry(ClusterIndexEntryKind::Graph, "type", "old")],
        )],
    )
    .expect("origin a inserted");
    db.apply_module_update(
        "pkg",
        "beta",
        vec![record_for(
            "origin-b",
            "pkg",
            "beta",
            0,
            vec![entry(ClusterIndexEntryKind::Type, "mode", "stable")],
        )],
    )
    .expect("origin b inserted");

    let before_a = db
        .import_scoped_view(view_request(&["origin-a"]))
        .expect("view a before");
    let before_b = db
        .import_scoped_view(view_request(&["origin-b"]))
        .expect("view b before");
    let before_both = db
        .import_scoped_view(view_request(&["origin-b", "origin-a"]))
        .expect("view both before");

    db.apply_module_update(
        "pkg",
        "alpha",
        vec![record(
            "origin-a",
            0,
            vec![entry(ClusterIndexEntryKind::Graph, "type", "new")],
        )],
    )
    .expect("origin a changed");

    let after_a = db
        .import_scoped_view(view_request(&["origin-a"]))
        .expect("view a after");
    let after_b = db
        .import_scoped_view(view_request(&["origin-b"]))
        .expect("view b after");
    let after_both = db
        .import_scoped_view(view_request(&["origin-a", "origin-b"]))
        .expect("view both after");

    assert_ne!(after_a, before_a);
    assert_eq!(after_a.key, before_a.key);
    assert_eq!(after_b, before_b);
    assert_ne!(after_both, before_both);
    assert_ne!(
        before_a.key.visible_origin_set_hash,
        before_both.key.visible_origin_set_hash
    );
}

#[test]
fn view_request_fails_closed_for_missing_origins_and_unknown_compatibility() {
    let mut db = ClusterDbIndex::new();
    db.apply_module_update(
        "pkg",
        "alpha",
        vec![record(
            "origin-a",
            0,
            vec![entry(ClusterIndexEntryKind::Attribute, "attr", "a")],
        )],
    )
    .expect("origin inserted");

    assert!(matches!(
        db.import_scoped_view(view_request(&["missing-origin"])),
        Err(ClusterDbViewMiss::MissingVisibleOrigin { .. })
    ));

    for (field, mutate) in [
        (
            "importing_package_id",
            Box::new(|request: &mut ImportScopedViewRequest| {
                request.importing_package_id = " ".to_owned();
            }) as Box<dyn Fn(&mut ImportScopedViewRequest)>,
        ),
        (
            "importing_module_path",
            Box::new(|request: &mut ImportScopedViewRequest| {
                request.importing_module_path = " ".to_owned();
            }),
        ),
        (
            "import_closure_identity",
            Box::new(|request: &mut ImportScopedViewRequest| {
                request.import_closure_identity = " ".to_owned();
            }),
        ),
    ] {
        let mut missing_identity = view_request(&["origin-a"]);
        mutate(&mut missing_identity);
        assert!(matches!(
            db.import_scoped_view(missing_identity),
            Err(ClusterDbViewMiss::MissingRequiredIdentity {
                field: actual_field
            }) if actual_field == field
        ));
    }

    let mut unsupported_schema = view_request(&["origin-a"]);
    unsupported_schema.cluster_db_schema_version = SchemaVersion::new("unsupported");
    assert!(matches!(
        db.import_scoped_view(unsupported_schema),
        Err(ClusterDbViewMiss::UnsupportedSchema { .. })
    ));

    let mut unknown_policy = view_request(&["origin-a"]);
    unknown_policy.policy_compatibility[0].value = "unknown".to_owned();
    assert!(matches!(
        db.import_scoped_view(unknown_policy),
        Err(ClusterDbViewMiss::UnknownCompatibility {
            family: "policy_compatibility"
        })
    ));

    let mut unknown_schema_compatibility = view_request(&["origin-a"]);
    unknown_schema_compatibility.schema_compatibility[0].value = "unknown".to_owned();
    assert!(matches!(
        db.import_scoped_view(unknown_schema_compatibility),
        Err(ClusterDbViewMiss::UnknownCompatibility {
            family: "schema_compatibility"
        })
    ));

    let mut unsupported_producer_schema = view_request(&["origin-a"]);
    unsupported_producer_schema.producer_schema_versions[0].version = SchemaVersion::new("unknown");
    assert!(matches!(
        db.import_scoped_view(unsupported_producer_schema),
        Err(ClusterDbViewMiss::UnsupportedSchema { .. })
    ));

    let mut unknown_toolchain = view_request(&["origin-a"]);
    unknown_toolchain.toolchain_compatibility[0].value = "unsupported".to_owned();
    assert!(matches!(
        db.import_scoped_view(unknown_toolchain),
        Err(ClusterDbViewMiss::UnknownCompatibility {
            family: "toolchain_compatibility"
        })
    ));

    let mut missing_schema = view_request(&["origin-a"]);
    missing_schema.producer_schema_versions.clear();
    assert!(matches!(
        db.import_scoped_view(missing_schema),
        Err(ClusterDbViewMiss::UnknownCompatibility {
            family: "producer_schema_versions"
        })
    ));

    let mut missing_schema_compatibility = view_request(&["origin-a"]);
    missing_schema_compatibility.schema_compatibility.clear();
    assert!(matches!(
        db.import_scoped_view(missing_schema_compatibility),
        Err(ClusterDbViewMiss::UnknownCompatibility {
            family: "schema_compatibility"
        })
    ));

    let mut producer_schema_mismatch = view_request(&["origin-a"]);
    producer_schema_mismatch.producer_schema_versions[0].version = SchemaVersion::new("2.0");
    assert!(matches!(
        db.import_scoped_view(producer_schema_mismatch),
        Err(ClusterDbViewMiss::UnsupportedSchema { .. })
    ));

    let mut policy_mismatch = view_request(&["origin-a"]);
    policy_mismatch.policy_compatibility[0].value = "different-known".to_owned();
    assert!(matches!(
        db.import_scoped_view(policy_mismatch),
        Err(ClusterDbViewMiss::UnknownCompatibility {
            family: "policy_compatibility"
        })
    ));

    let mut schema_compatibility_mismatch = view_request(&["origin-a"]);
    schema_compatibility_mismatch.schema_compatibility[0].value = "different-known".to_owned();
    assert!(matches!(
        db.import_scoped_view(schema_compatibility_mismatch),
        Err(ClusterDbViewMiss::UnknownCompatibility {
            family: "schema_compatibility"
        })
    ));

    let mut toolchain_mismatch = view_request(&["origin-a"]);
    toolchain_mismatch.toolchain_compatibility[0].value = "different-known".to_owned();
    assert!(matches!(
        db.import_scoped_view(toolchain_mismatch),
        Err(ClusterDbViewMiss::UnknownCompatibility {
            family: "toolchain_compatibility"
        })
    ));

    let mut missing_traversal = view_request(&["origin-a"]);
    missing_traversal.traversal_profile.clear();
    assert!(matches!(
        db.import_scoped_view(missing_traversal),
        Err(ClusterDbViewMiss::UnknownCompatibility {
            family: "traversal_profile"
        })
    ));

    let mut mismatched_policy = view_request(&["origin-a"]);
    mismatched_policy.verifier_policy_fingerprint = PolicyFingerprint::new(hash(42));
    assert!(matches!(
        db.import_scoped_view(mismatched_policy),
        Err(ClusterDbViewMiss::UnknownCompatibility {
            family: "verifier_policy_fingerprint"
        })
    ));
}

#[test]
fn import_scoped_view_order_independent_of_visible_origin_order_and_write_order() {
    let origin_a = record(
        "origin-a",
        0,
        vec![
            entry(ClusterIndexEntryKind::Graph, "type", "a-graph"),
            entry(ClusterIndexEntryKind::SubsumptionDag, "symbol", "a-edge"),
            entry(ClusterIndexEntryKind::Attribute, "attr", "a-attr"),
            entry(ClusterIndexEntryKind::Type, "mode", "a-type"),
            entry(ClusterIndexEntryKind::Reduction, "lhs", "a-rule"),
        ],
    );
    let origin_b = record_for(
        "origin-b",
        "pkg",
        "beta",
        0,
        vec![
            entry(ClusterIndexEntryKind::Graph, "type", "b-graph"),
            entry(ClusterIndexEntryKind::SubsumptionDag, "symbol", "b-edge"),
            entry(ClusterIndexEntryKind::Attribute, "attr", "b-attr"),
            entry(ClusterIndexEntryKind::Type, "mode", "b-type"),
            entry(ClusterIndexEntryKind::Reduction, "lhs", "b-rule"),
        ],
    );

    let mut left = ClusterDbIndex::new();
    left.apply_module_update("pkg", "alpha", vec![origin_a.clone()])
        .expect("left origin a");
    left.apply_module_update("pkg", "beta", vec![origin_b.clone()])
        .expect("left origin b");

    let mut right = ClusterDbIndex::new();
    right
        .apply_module_update("pkg", "beta", vec![origin_b])
        .expect("right origin b");
    right
        .apply_module_update("pkg", "alpha", vec![origin_a])
        .expect("right origin a");

    let left_view = left
        .import_scoped_view(view_request(&["origin-b", "origin-a", "origin-a"]))
        .expect("left view");
    let right_view = right
        .import_scoped_view(view_request(&["origin-a", "origin-b"]))
        .expect("right view");

    assert_eq!(left_view, right_view);
    assert_eq!(left_view.visible_origin_keys, ["origin-a", "origin-b"]);
    assert_eq!(
        origin_keys(&left_view.indexes.graph_rows),
        ["origin-a", "origin-b"]
    );
    assert_eq!(
        origin_keys(&left_view.indexes.subsumption_dag_rows),
        ["origin-a", "origin-b"]
    );
    assert_eq!(
        origin_keys(&left_view.indexes.attr_index_rows),
        ["origin-a", "origin-b"]
    );
    assert_eq!(
        origin_keys(&left_view.indexes.type_index_rows),
        ["origin-a", "origin-b"]
    );
    assert_eq!(
        origin_keys(&left_view.indexes.reduction_index_rows),
        ["origin-a", "origin-b"]
    );
}

#[test]
fn import_scoped_view_does_not_infer_hidden_trace_steps() {
    let mut db = ClusterDbIndex::new();
    db.apply_module_update(
        "pkg",
        "alpha",
        vec![record(
            "origin-a",
            0,
            vec![entry(ClusterIndexEntryKind::Graph, "type", "graph-only")],
        )],
    )
    .expect("graph-only origin inserted");
    db.apply_module_update(
        "pkg",
        "beta",
        vec![record_for(
            "origin-b",
            "pkg",
            "beta",
            0,
            vec![entry(ClusterIndexEntryKind::Reduction, "lhs", "reduction")],
        )],
    )
    .expect("reduction origin inserted");

    let view = db
        .import_scoped_view(view_request(&["origin-a"]))
        .expect("view materializes");

    assert_eq!(origin_keys(&view.indexes.graph_rows), ["origin-a"]);
    assert!(view.indexes.reduction_index_rows.is_empty());
    assert!(view.indexes.subsumption_dag_rows.is_empty());
    assert!(view.indexes.attr_index_rows.is_empty());
    assert!(view.indexes.type_index_rows.is_empty());

    let reduction_only = db
        .import_scoped_view(view_request(&["origin-b"]))
        .expect("visible reduction-only view materializes");

    assert!(reduction_only.indexes.graph_rows.is_empty());
    assert!(reduction_only.indexes.subsumption_dag_rows.is_empty());
    assert!(reduction_only.indexes.attr_index_rows.is_empty());
    assert!(reduction_only.indexes.type_index_rows.is_empty());
    assert_eq!(
        origin_keys(&reduction_only.indexes.reduction_index_rows),
        ["origin-b"]
    );
}

fn record(
    origin_key: &str,
    declaration_order: u32,
    index_entries: Vec<ClusterIndexEntry>,
) -> ClusterContributionRecord {
    record_for(origin_key, "pkg", "alpha", declaration_order, index_entries)
}

fn record_for(
    origin_key: &str,
    package_id: &str,
    module_path: &str,
    declaration_order: u32,
    index_entries: Vec<ClusterIndexEntry>,
) -> ClusterContributionRecord {
    ClusterContributionRecord {
        schema_version: SchemaVersion::new(CLUSTER_DB_SCHEMA_VERSION),
        origin: ClusterContributionOrigin {
            origin_key: origin_key.to_owned(),
            package_id: package_id.to_owned(),
            module_path: module_path.to_owned(),
            stable_contribution_id: format!("{origin_key}-id"),
            label: format!("{origin_key}-label"),
            contribution_kind: ClusterContributionKind::ConditionalCluster,
            target_pattern_hash: hash(1),
            guard_hash: hash(2),
            declared_contribution_hash: hash(3),
            accepted_visibility: ClusterContributionVisibility::ImporterVisible,
            accepted_status: ClusterContributionStatus::Accepted,
            accepted_status_projection_hash: hash(4),
            accepted_witness_or_discharge_hash: Some(named_hash("witness", 5)),
            proof_backed: true,
            verifier_policy_fingerprint: PolicyFingerprint::new(hash(6)),
            policy_compatibility: vec![compat("verifier-policy", "known")],
            schema_compatibility: vec![compat("cluster-db-schema", "known")],
            toolchain_compatibility: vec![compat("producer-toolchain", "known")],
            producer_schema_versions: vec![schema("registration-summary")],
            trace_replay_hashes: Some(vec![named_hash("trace", 7)]),
            dependency_interface_hashes: Some(vec![named_hash("registration-interface", 8)]),
            origin_footprint_hash: Some(hash(9)),
            footprint_completeness: ClusterOriginFootprintCompleteness::Complete,
            uncacheable: false,
        },
        declaration_order,
        index_entries,
        diagnostic_refs: vec![named_hash("diagnostic", 10)],
    }
}

fn view_request(visible_origin_keys: &[&str]) -> ImportScopedViewRequest {
    ImportScopedViewRequest {
        importing_package_id: "consumer-pkg".to_owned(),
        importing_module_path: "consumer".to_owned(),
        import_closure_identity: "consumer/import-closure/v1".to_owned(),
        visible_origin_keys: visible_origin_keys
            .iter()
            .map(|origin_key| (*origin_key).to_owned())
            .collect(),
        verifier_policy_fingerprint: PolicyFingerprint::new(hash(6)),
        cluster_db_schema_version: SchemaVersion::new(CLUSTER_DB_SCHEMA_VERSION),
        producer_schema_versions: vec![schema("registration-summary")],
        policy_compatibility: vec![compat("verifier-policy", "known")],
        schema_compatibility: vec![compat("cluster-db-schema", "known")],
        toolchain_compatibility: vec![compat("producer-toolchain", "known")],
        traversal_profile: vec![compat("traversal-profile", "canonical")],
    }
}

fn entry(
    entry_kind: ClusterIndexEntryKind,
    primary_key: &str,
    secondary_key: &str,
) -> ClusterIndexEntry {
    ClusterIndexEntry {
        entry_kind,
        primary_key: primary_key.to_owned(),
        secondary_key: secondary_key.to_owned(),
        strategy_key: format!("{primary_key}->{secondary_key}"),
        contribution_fingerprint: hash(primary_key.as_bytes()[0] ^ secondary_key.as_bytes()[0]),
    }
}

fn schema(name: &str) -> NamedSchemaVersion {
    NamedSchemaVersion {
        schema_family: "mizar-artifact/registration-summary".to_owned(),
        name: name.to_owned(),
        version: SchemaVersion::new("1.0"),
    }
}

fn named_hash(name: &str, seed: u8) -> NamedHash {
    NamedHash {
        name: name.to_owned(),
        domain: format!("test-domain/{name}"),
        digest: hash(seed),
    }
}

fn compat(field_name: &str, value: &str) -> CompatibilityField {
    CompatibilityField {
        family: "cluster-db-test".to_owned(),
        field_name: field_name.to_owned(),
        value: value.to_owned(),
    }
}

fn origin_keys(rows: &[ClusterAggregateRow]) -> Vec<&str> {
    rows.iter().map(|row| row.origin_key.as_str()).collect()
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}
