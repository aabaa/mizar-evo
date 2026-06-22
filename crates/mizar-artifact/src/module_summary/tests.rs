use super::{
    DependencyInterfaceRef, ExportedLabelSummary, ExportedSymbolSummary,
    LexicalContributionSummary, MODULE_SUMMARY_SCHEMA_FAMILY, ModuleLexicalSummary,
    ModuleReexportSummary, ModuleSummary, ModuleSummaryError, ModuleSummaryIdentity,
    ModuleSummaryReadOptions, ProofStatusSummary, SourceRangeSummary, current_schema_version,
    interface_hash_string, interface_projection_json, module_summary_json,
    module_summary_json_unchecked, read_module_summary, write_module_summary,
};
use crate::store::{
    CanonicalHashDomain, CanonicalJson, HashClass, SchemaVersionError, canonical_json_string,
};
use mizar_session::Hash;

#[test]
fn module_summary_round_trips_through_canonical_json() {
    let summary = sample_summary();
    let json = module_summary_json(&summary).expect("canonical module summary JSON");
    let bytes = write_module_summary(&summary).expect("canonical module summary bytes");

    assert_eq!(bytes, canonical_json_string(&json).into_bytes());
    assert_eq!(
        read_module_summary(&json, ModuleSummaryReadOptions::default()).expect("valid summary"),
        summary
    );
}

#[test]
fn module_summary_writer_and_hash_are_deterministic_for_identical_inputs() {
    let summary = sample_summary();
    let first_json = module_summary_json(&summary).expect("first canonical JSON");
    let first_bytes = write_module_summary(&summary).expect("first canonical bytes");
    let first_projection = interface_projection_json(&summary).expect("first interface projection");
    let first_hash = summary.compute_interface_hash().expect("first hash");

    for _ in 0..3 {
        let json = module_summary_json(&summary).expect("repeated canonical JSON");
        assert_eq!(json, first_json);
        assert_eq!(canonical_json_string(&json).into_bytes(), first_bytes);
        assert_eq!(
            write_module_summary(&summary).expect("repeated canonical bytes"),
            first_bytes
        );
        assert_eq!(
            interface_projection_json(&summary).expect("repeated interface projection"),
            first_projection
        );
        assert_eq!(
            summary.compute_interface_hash().expect("repeated hash"),
            first_hash
        );
    }
}

#[test]
fn writer_sorts_collections_and_reader_rejects_unsorted_arrays() {
    let mut summary = sample_summary();
    summary.exported_symbols.reverse();
    summary.exported_labels.reverse();
    summary.lexical_summary.contributions.reverse();
    summary.reexports.reverse();
    summary.dependency_interfaces.reverse();
    summary.refresh_interface_hash().expect("refresh hash");

    let json = module_summary_json(&summary).expect("writer sorts collections");
    let text = canonical_json_string(&json);
    assert!(
        text.find("Alpha.one").expect("alpha symbol") < text.find("Zeta.one").expect("zeta symbol")
    );
    assert!(
        text.find("\"label\":\"A1\"").expect("label A")
            < text.find("\"label\":\"Z1\"").expect("label Z")
    );

    let mut unsorted = json.clone();
    reverse_array_field(&mut unsorted, "exported_symbols");
    assert!(matches!(
        read_module_summary(&unsorted, ModuleSummaryReadOptions::default()),
        Err(ModuleSummaryError::UnsortedCollection { path }) if path == "$.exported_symbols"
    ));
}

#[test]
fn interface_hash_ignores_source_metadata() {
    let mut baseline = sample_summary();
    let baseline_hash = baseline
        .refresh_interface_hash()
        .expect("baseline interface hash");
    let mut changed_metadata = baseline.clone();
    changed_metadata.source_hash = hash(99);
    changed_metadata.exported_symbols[0].source_range = SourceRangeSummary {
        start_byte: 100,
        end_byte: 120,
    };
    changed_metadata.exported_labels[0].source_range = SourceRangeSummary {
        start_byte: 200,
        end_byte: 202,
    };

    assert_eq!(
        changed_metadata
            .refresh_interface_hash()
            .expect("metadata-only interface hash"),
        baseline_hash
    );
}

#[test]
fn interface_hash_uses_declared_module_summary_domain() {
    let summary = sample_summary();
    let projection = interface_projection_json(&summary).expect("interface projection");
    let expected = CanonicalHashDomain::new(
        HashClass::Interface,
        MODULE_SUMMARY_SCHEMA_FAMILY,
        current_schema_version(),
    )
    .hash(&projection, &[]);

    assert_eq!(
        summary.compute_interface_hash().expect("summary hash"),
        expected
    );
    assert_ne!(
        CanonicalHashDomain::new(
            HashClass::Artifact,
            MODULE_SUMMARY_SCHEMA_FAMILY,
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
fn interface_hash_changes_for_importer_visible_projection_changes() {
    assert_interface_hash_changes("module identity", |summary| {
        summary.module.module_path = "Renamed.Hidden".to_owned();
    });
    assert_interface_hash_changes("symbol signature", |summary| {
        summary.exported_symbols[0].rendered_signature = "theorem Alpha.one: x = z".to_owned();
    });
    assert_interface_hash_changes("symbol interface fingerprint", |summary| {
        summary.exported_symbols[0].interface_fingerprint = hash(42);
    });
    assert_interface_hash_changes("proof status", |summary| {
        summary.exported_symbols[0].proof_status = Some(ProofStatusSummary::NotAccepted);
    });
    assert_interface_hash_changes("exported label", |summary| {
        summary.exported_labels[0].target_kind = "scheme".to_owned();
    });
    assert_interface_hash_changes("lexical fingerprint", |summary| {
        summary.lexical_summary.fingerprint = Some(hash(43));
    });
    assert_interface_hash_changes("lexical contribution", |summary| {
        summary.lexical_summary.contributions[0].payload = "infix alpha changed".to_owned();
    });
    assert_interface_hash_changes("reexport", |summary| {
        summary.reexports[0].exported_name = Some("Changed.Alpha".to_owned());
    });
    assert_interface_hash_changes("dependency interface", |summary| {
        summary.dependency_interfaces[0].interface_hash = hash(44);
    });
}

#[test]
fn interface_hash_changes_for_exported_interface_changes() {
    let mut baseline = sample_summary();
    let baseline_hash = baseline.refresh_interface_hash().expect("baseline hash");
    let mut changed = baseline.clone();
    changed.exported_symbols[0].rendered_signature = "theorem Alpha.one: x = z".to_owned();

    assert_ne!(
        changed.refresh_interface_hash().expect("changed hash"),
        baseline_hash
    );
}

#[test]
fn incompatible_version_reads_fail_cleanly() {
    let summary = sample_summary();
    let mut json = module_summary_json(&summary).expect("canonical JSON");
    replace_object_field(&mut json, "schema_version", CanonicalJson::string("2.0"));

    assert!(matches!(
        read_module_summary(
            &json,
            ModuleSummaryReadOptions {
                artifact_path: Some("build/dep.mizir.json"),
                ..ModuleSummaryReadOptions::default()
            },
        ),
        Err(ModuleSummaryError::SchemaVersion(
            SchemaVersionError::MajorMismatch { .. }
        ))
    ));
}

#[test]
fn reader_rejects_hash_and_module_mismatches() {
    let summary = sample_summary();
    let json = module_summary_json(&summary).expect("canonical JSON");
    let other_module = ModuleSummaryIdentity {
        package_id: "other-package".to_owned(),
        ..summary.module.clone()
    };
    assert!(matches!(
        read_module_summary(
            &json,
            ModuleSummaryReadOptions {
                expected_module: Some(&other_module),
                ..ModuleSummaryReadOptions::default()
            }
        ),
        Err(ModuleSummaryError::ModuleIdentityMismatch { .. })
    ));

    assert!(matches!(
        read_module_summary(
            &json,
            ModuleSummaryReadOptions {
                expected_interface_hash: Some(hash(88)),
                ..ModuleSummaryReadOptions::default()
            }
        ),
        Err(ModuleSummaryError::ExpectedInterfaceHashMismatch { .. })
    ));

    let mut bad_hash_json = json;
    replace_object_field(
        &mut bad_hash_json,
        "interface_hash",
        CanonicalJson::string(interface_hash_string(current_schema_version(), hash(77))),
    );
    assert!(matches!(
        read_module_summary(&bad_hash_json, ModuleSummaryReadOptions::default()),
        Err(ModuleSummaryError::InterfaceHashMismatch { .. })
    ));
}

#[test]
fn reader_rejects_wrong_hash_construction() {
    let summary = sample_summary();
    let mut json = module_summary_json(&summary).expect("canonical JSON");
    replace_object_field(
        &mut json,
        "interface_hash",
        CanonicalJson::string(format!(
            "mizar-artifact/artifact-framed-hash-text/v1:artifact:mizar-artifact/module-summary:1.0:{}",
            "11".repeat(Hash::BYTE_LEN)
        )),
    );

    assert!(matches!(
        read_module_summary(&json, ModuleSummaryReadOptions::default()),
        Err(ModuleSummaryError::InvalidHash { path, .. }) if path == "$.interface_hash"
    ));
}

#[test]
fn reader_rejects_hash_string_domain_and_digest_mismatches() {
    let summary = sample_summary();
    let json = module_summary_json(&summary).expect("canonical JSON");
    let digest = "11".repeat(Hash::BYTE_LEN);

    for bad_hash in [
        format!(
            "mizar-artifact/other-framed-hash/v1:interface:mizar-artifact/module-summary:1.0:{digest}"
        ),
        format!(
            "mizar-artifact/artifact-framed-hash-text/v1:artifact:mizar-artifact/module-summary:1.0:{digest}"
        ),
        format!(
            "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact/other-summary:1.0:{digest}"
        ),
        format!(
            "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact/module-summary:1.1:{digest}"
        ),
        format!(
            "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact/module-summary:1.0:{}",
            "gg".repeat(Hash::BYTE_LEN)
        ),
    ] {
        let mut bad_json = json.clone();
        replace_object_field(
            &mut bad_json,
            "interface_hash",
            CanonicalJson::string(bad_hash),
        );

        assert!(matches!(
            read_module_summary(&bad_json, ModuleSummaryReadOptions::default()),
            Err(ModuleSummaryError::InvalidHash { path, .. }) if path == "$.interface_hash"
        ));
    }

    let mut wrong_source_hash = json;
    replace_object_field(
        &mut wrong_source_hash,
        "source_hash",
        CanonicalJson::string(format!("wrong-source-hash/v1:{digest}")),
    );

    assert!(matches!(
        read_module_summary(&wrong_source_hash, ModuleSummaryReadOptions::default()),
        Err(ModuleSummaryError::InvalidHash { path, .. }) if path == "$.source_hash"
    ));
}

#[test]
fn reader_rejects_unsorted_collections() {
    let summary = sample_summary();
    let json = module_summary_json(&summary).expect("canonical JSON");

    assert_unsorted_collection_rejected(json.clone(), &["exported_symbols"], "$.exported_symbols");
    assert_unsorted_collection_rejected(json.clone(), &["exported_labels"], "$.exported_labels");
    assert_unsorted_collection_rejected(
        json.clone(),
        &["lexical_summary", "contributions"],
        "$.lexical_summary.contributions",
    );
    assert_unsorted_collection_rejected(json.clone(), &["reexports"], "$.reexports");
    assert_unsorted_collection_rejected(
        json,
        &["dependency_interfaces"],
        "$.dependency_interfaces",
    );
}

#[test]
fn reader_rejects_duplicate_collection_keys() {
    let summary = sample_summary();
    let json = module_summary_json(&summary).expect("canonical JSON");

    assert_duplicate_collection_rejected(json.clone(), &["exported_symbols"], "$.exported_symbols");
    assert_duplicate_collection_rejected(json.clone(), &["exported_labels"], "$.exported_labels");
    assert_duplicate_collection_rejected(
        json.clone(),
        &["lexical_summary", "contributions"],
        "$.lexical_summary.contributions",
    );
    assert_duplicate_collection_rejected(json.clone(), &["reexports"], "$.reexports");
    assert_duplicate_collection_rejected(
        json,
        &["dependency_interfaces"],
        "$.dependency_interfaces",
    );
}

#[test]
fn reader_rejects_subset_duplicate_keys() {
    let mut duplicate_label_key = sample_summary();
    duplicate_label_key.exported_labels[1].label =
        duplicate_label_key.exported_labels[0].label.clone();
    duplicate_label_key.exported_labels[1].owner_fully_qualified_name = duplicate_label_key
        .exported_labels[0]
        .owner_fully_qualified_name
        .clone();
    duplicate_label_key.exported_labels[1].origin_id =
        duplicate_label_key.exported_labels[0].origin_id.clone();
    duplicate_label_key.exported_labels[1].visibility = "protected".to_owned();
    duplicate_label_key.exported_labels[1].target_kind = "scheme".to_owned();
    duplicate_label_key
        .refresh_interface_hash()
        .expect("duplicate label hash");
    let duplicate_label_json =
        module_summary_json_unchecked(&duplicate_label_key).expect("unchecked duplicate JSON");

    assert!(matches!(
        read_module_summary(&duplicate_label_json, ModuleSummaryReadOptions::default()),
        Err(ModuleSummaryError::DuplicateEntry { path, .. }) if path == "$.exported_labels"
    ));

    let mut duplicate_dependency_key = sample_summary();
    duplicate_dependency_key.dependency_interfaces[1].module = duplicate_dependency_key
        .dependency_interfaces[0]
        .module
        .clone();
    duplicate_dependency_key.dependency_interfaces[1].interface_hash = hash(77);
    duplicate_dependency_key
        .refresh_interface_hash()
        .expect("duplicate dependency hash");
    let duplicate_dependency_json =
        module_summary_json_unchecked(&duplicate_dependency_key).expect("unchecked duplicate JSON");

    assert!(matches!(
        read_module_summary(&duplicate_dependency_json, ModuleSummaryReadOptions::default()),
        Err(ModuleSummaryError::DuplicateEntry { path, .. }) if path == "$.dependency_interfaces"
    ));
}

#[test]
fn reader_rejects_duplicate_identity_keys() {
    let mut summary = sample_summary();
    summary.exported_symbols[1].fully_qualified_name =
        summary.exported_symbols[0].fully_qualified_name.clone();
    summary.exported_symbols[1].origin_id = summary.exported_symbols[0].origin_id.clone();
    summary.refresh_interface_hash().expect("refresh hash");

    assert!(matches!(
        module_summary_json(&summary),
        Err(ModuleSummaryError::DuplicateEntry { path, .. }) if path == "$.exported_symbols"
    ));
}

fn sample_summary() -> ModuleSummary {
    let module = identity("base", "Hidden");
    let dep_alpha = identity("dep-alpha", "AlphaDep");
    let dep_zeta = identity("dep-zeta", "ZetaDep");
    let mut summary = ModuleSummary {
        schema_version: current_schema_version(),
        module: module.clone(),
        source_hash: hash(1),
        interface_hash: hash(0),
        exported_symbols: vec![
            symbol("Zeta.one", "symbol:zeta", 20, "theorem Zeta.one: x = x", 2),
            symbol(
                "Alpha.one",
                "symbol:alpha",
                0,
                "theorem Alpha.one: x = x",
                3,
            ),
        ],
        exported_labels: vec![label("Z1", "label:zeta", 25), label("A1", "label:alpha", 5)],
        lexical_summary: ModuleLexicalSummary {
            schema_version: "lexer-summary-v1".to_owned(),
            fingerprint: Some(hash(4)),
            contributions: vec![
                LexicalContributionSummary {
                    kind: "notation".to_owned(),
                    key: "zeta".to_owned(),
                    payload: "infix zeta".to_owned(),
                },
                LexicalContributionSummary {
                    kind: "notation".to_owned(),
                    key: "alpha".to_owned(),
                    payload: "infix alpha".to_owned(),
                },
            ],
        },
        reexports: vec![
            ModuleReexportSummary {
                target_module: dep_zeta.clone(),
                target_item_origin_id: Some("dep:zeta".to_owned()),
                exported_name: Some("Dep.Zeta".to_owned()),
                provenance_origin_id: Some("reexport:zeta".to_owned()),
            },
            ModuleReexportSummary {
                target_module: module,
                target_item_origin_id: Some("base:alpha".to_owned()),
                exported_name: Some("Base.Alpha".to_owned()),
                provenance_origin_id: Some("reexport:alpha".to_owned()),
            },
        ],
        dependency_interfaces: vec![
            DependencyInterfaceRef {
                module: dep_zeta,
                interface_hash: hash(5),
            },
            DependencyInterfaceRef {
                module: dep_alpha,
                interface_hash: hash(6),
            },
        ],
    };
    summary.refresh_interface_hash().expect("sample hash");
    module_summary_json(&summary).expect("sample is canonical");
    read_module_summary(
        &module_summary_json(&summary).expect("sample JSON"),
        ModuleSummaryReadOptions::default(),
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

fn symbol(
    fully_qualified_name: &str,
    origin_id: &str,
    start_byte: u64,
    rendered_signature: &str,
    fingerprint_seed: u8,
) -> ExportedSymbolSummary {
    ExportedSymbolSummary {
        origin_id: origin_id.to_owned(),
        fully_qualified_name: fully_qualified_name.to_owned(),
        namespace_path: vec!["Hidden".to_owned()],
        visibility: "public".to_owned(),
        declaration_kind: "theorem".to_owned(),
        source_range: SourceRangeSummary {
            start_byte,
            end_byte: start_byte + 10,
        },
        rendered_signature: rendered_signature.to_owned(),
        interface_fingerprint: hash(fingerprint_seed),
        proof_status: Some(ProofStatusSummary::Accepted),
    }
}

fn label(label: &str, origin_id: &str, start_byte: u64) -> ExportedLabelSummary {
    ExportedLabelSummary {
        origin_id: origin_id.to_owned(),
        label: label.to_owned(),
        owner_fully_qualified_name: "Hidden.owner".to_owned(),
        visibility: "public".to_owned(),
        source_range: SourceRangeSummary {
            start_byte,
            end_byte: start_byte + 2,
        },
        target_kind: "theorem".to_owned(),
    }
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

fn assert_interface_hash_changes(name: &str, mutate: impl FnOnce(&mut ModuleSummary)) {
    let baseline = sample_summary();
    let baseline_hash = baseline.interface_hash;
    let mut changed = baseline.clone();
    mutate(&mut changed);

    assert_ne!(
        changed
            .refresh_interface_hash()
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
        read_module_summary(&json, ModuleSummaryReadOptions::default()),
        Err(ModuleSummaryError::UnsortedCollection { path }) if path == expected_path
    ));
}

fn assert_duplicate_collection_rejected(
    mut json: CanonicalJson,
    fields: &[&str],
    expected_path: &str,
) {
    duplicate_first_array_item_at(&mut json, fields);

    assert!(matches!(
        read_module_summary(&json, ModuleSummaryReadOptions::default()),
        Err(ModuleSummaryError::DuplicateEntry { path, .. }) if path == expected_path
    ));
}

fn replace_object_field(value: &mut CanonicalJson, field: &str, replacement: CanonicalJson) {
    let CanonicalJson::Object(fields) = value else {
        panic!("expected object");
    };
    fields.insert(field.to_owned(), replacement);
}

fn reverse_array_field(value: &mut CanonicalJson, field: &str) {
    reverse_array_at(value, &[field]);
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

fn array_at_mut<'a>(value: &'a mut CanonicalJson, fields: &[&str]) -> &'a mut Vec<CanonicalJson> {
    let (last, parents) = fields.split_last().expect("field path must be non-empty");
    let mut current = value;
    for field in parents {
        current = object_field_mut(current, field);
    }
    let CanonicalJson::Array(values) = object_field_mut(current, last) else {
        panic!("expected array");
    };
    values
}

fn object_field_mut<'a>(value: &'a mut CanonicalJson, field: &str) -> &'a mut CanonicalJson {
    let CanonicalJson::Object(fields) = value else {
        panic!("expected object");
    };
    fields.get_mut(field).expect("object field")
}
