use std::{
    fs,
    path::{Component, PathBuf},
    sync::{Arc, Barrier},
    thread,
};

use super::*;
use crate::cache_key::{
    CacheKeyBuilder, CacheKeyRequest, DependencyArtifactAvailability, DependencyHash,
    FootprintCompleteness, PipelinePhase, PolicyFingerprint, ProofReuseEvidenceIdentity, WorkUnit,
};

#[test]
fn record_store_round_trips_inline_output() {
    let root = test_root("round_trip");
    let store = CacheStoreRoot::new(&root);
    let key = key(FootprintCompleteness::Complete, "dev", false);
    let record = CacheRecord::new_inline(
        key.clone(),
        key.validation_inputs.toolchain_compatibility.clone(),
        b"cached output".to_vec(),
    );

    assert_eq!(
        store.insert(&record).expect("insert"),
        CacheInsertOutcome::Inserted
    );
    assert_eq!(
        store.insert(&record).expect("repeat insert"),
        CacheInsertOutcome::AlreadyPresent
    );

    let CacheLookupOutcome::Hit(hit) = store.lookup(&key) else {
        panic!("expected cache hit");
    };
    assert_eq!(hit.header.key, key);
    assert_eq!(hit.output, b"cached output");
    cleanup(root);
}

#[test]
fn blob_record_round_trips_by_content_digest() {
    let root = test_root("blob_round_trip");
    let store = CacheStoreRoot::new(&root);
    let key = key(FootprintCompleteness::Complete, "dev", false);
    let output = b"large cached output bytes".to_vec();
    let record = CacheRecord::new_blob(
        key.clone(),
        key.validation_inputs.toolchain_compatibility.clone(),
        output.clone(),
    );
    let blob = record
        .header
        .output
        .blob_ref()
        .expect("blob record has ref")
        .clone();
    assert_eq!(CACHE_BLOB_HASH_FAMILY, "blake3");
    assert_eq!(blob.hash_family, CACHE_BLOB_HASH_FAMILY);
    assert_eq!(blob.digest, hash_hex(output_hash(&output)));

    assert_eq!(
        store.insert(&record).expect("insert blob record"),
        CacheInsertOutcome::Inserted
    );
    assert_eq!(
        store.insert(&record).expect("repeat blob insert"),
        CacheInsertOutcome::AlreadyPresent
    );
    assert_eq!(fs::read(store.blob_path(&blob)).expect("read blob"), output);

    let CacheLookupOutcome::Hit(hit) = store.lookup(&key) else {
        panic!("expected blob cache hit");
    };
    assert_eq!(hit.header.output, record.header.output);
    assert_eq!(hit.output, output);
    cleanup(root);
}

#[test]
fn blob_lookup_fails_closed_for_missing_corrupt_or_unsupported_blob() {
    let root = test_root("blob_miss");
    let store = CacheStoreRoot::new(&root);
    let key = key(FootprintCompleteness::Complete, "dev", false);
    let output = b"blob output".to_vec();
    let record = CacheRecord::new_blob(
        key.clone(),
        key.validation_inputs.toolchain_compatibility.clone(),
        output.clone(),
    );
    store.insert(&record).expect("insert blob record");
    let blob = record
        .header
        .output
        .blob_ref()
        .expect("blob record has ref")
        .clone();

    fs::remove_file(store.blob_path(&blob)).expect("delete blob");
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
    );

    store.write_blob(&output).expect("restore blob");
    fs::write(store.blob_path(&blob), b"wrong bytes").expect("corrupt blob");
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
    );

    let mut unsupported_family = record.clone();
    let CacheOutputDescriptor::Blob {
        blob: unsupported_blob,
        ..
    } = &mut unsupported_family.header.output
    else {
        panic!("blob descriptor expected");
    };
    unsupported_blob.hash_family = "unknown-family".to_owned();
    write_raw(&store, &key, encode_record(&unsupported_family));
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::UnknownSchema)
    );

    let mut uppercase_digest = record.clone();
    let CacheOutputDescriptor::Blob {
        blob: uppercase_blob,
        ..
    } = &mut uppercase_digest.header.output
    else {
        panic!("blob descriptor expected");
    };
    uppercase_blob.digest = uppercase_blob.digest.to_ascii_uppercase();
    write_raw(&store, &key, encode_record(&uppercase_digest));
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::UnknownSchema)
    );

    let mut short_digest = record.clone();
    let CacheOutputDescriptor::Blob {
        blob: short_blob, ..
    } = &mut short_digest.header.output
    else {
        panic!("blob descriptor expected");
    };
    short_blob.digest = "abcd".to_owned();
    write_raw(&store, &key, encode_record(&short_digest));
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::UnknownSchema)
    );

    let mut path_like_digest = record.clone();
    let CacheOutputDescriptor::Blob {
        blob: path_like_blob,
        ..
    } = &mut path_like_digest.header.output
    else {
        panic!("blob descriptor expected");
    };
    path_like_blob.digest = "../escape".to_owned();
    assert!(
        !store
            .blob_path(path_like_blob)
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    );
    write_raw(&store, &key, encode_record(&path_like_digest));
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::UnknownSchema)
    );

    let unsafe_ref = CacheBlobRef {
        hash_family: "..".to_owned(),
        digest: ".".to_owned(),
    };
    assert!(
        !store
            .blob_path(&unsafe_ref)
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    );

    let alternate = b"alternate blob".to_vec();
    let alternate_blob = store.write_blob(&alternate).expect("write alternate blob");
    let mut mismatched_digest = record.clone();
    let CacheOutputDescriptor::Blob {
        blob: mismatched_blob,
        ..
    } = &mut mismatched_digest.header.output
    else {
        panic!("blob descriptor expected");
    };
    *mismatched_blob = alternate_blob;
    write_raw(&store, &key, encode_record(&mismatched_digest));
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
    );

    fs::remove_file(store.blob_path(&blob)).expect("remove corrupt expected blob");
    store.write_blob(&output).expect("restore expected blob");
    let mut mismatched_len = record.clone();
    let CacheOutputDescriptor::Blob { byte_len, .. } = &mut mismatched_len.header.output else {
        panic!("blob descriptor expected");
    };
    *byte_len += 1;
    write_raw(&store, &key, encode_record(&mismatched_len));
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
    );

    let header = canonical_json_bytes(&header_json(&record.header));
    write_raw(&store, &key, encode_raw_record(&header, b"nonempty"));
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
    );
    cleanup(root);
}

#[test]
fn blob_writers_converge_and_reject_divergent_existing_digest() {
    let root = test_root("blob_writers");
    let store = CacheStoreRoot::new(&root);
    let output = b"shared blob output".to_vec();

    let first = store.write_blob(&output).expect("write blob");
    let second = store.write_blob(&output).expect("repeat write");
    assert_eq!(first, second);
    assert_eq!(
        fs::read(store.blob_path(&first)).expect("read blob"),
        output
    );

    let temporary_dir = root.join("tmp");
    fs::create_dir_all(&temporary_dir).expect("mkdir temp");
    store
        .write_blob_via_unique_temp(&first, &store.blob_path(&first), &temporary_dir, &output)
        .expect("identical final-link collision converges");

    let concurrent_output = b"parallel shared blob output".to_vec();
    let expected_concurrent = CacheBlobRef::for_output(&concurrent_output);
    let writers = 8;
    let barrier = Arc::new(Barrier::new(writers));
    let mut handles = Vec::new();
    for _ in 0..writers {
        let store = store.clone();
        let barrier = Arc::clone(&barrier);
        let output = concurrent_output.clone();
        handles.push(thread::spawn(move || {
            barrier.wait();
            store.write_blob(&output).expect("concurrent blob write")
        }));
    }
    for handle in handles {
        assert_eq!(
            handle.join().expect("writer thread joins"),
            expected_concurrent
        );
    }
    assert_eq!(
        fs::read(store.blob_path(&expected_concurrent)).expect("read concurrent blob"),
        concurrent_output
    );

    fs::write(store.blob_path(&first), b"divergent bytes").expect("corrupt existing blob");
    assert!(matches!(
        store.write_blob(&output),
        Err(CacheStoreError::DivergentRecord { .. })
    ));
    assert!(matches!(
        store.write_blob_via_unique_temp(&first, &store.blob_path(&first), &temporary_dir, &output),
        Err(CacheStoreError::DivergentRecord { .. })
    ));
    cleanup(root);
}

#[test]
fn blob_insert_rejects_mismatched_descriptor_without_trusting_record() {
    let root = test_root("blob_insert_mismatch");
    let store = CacheStoreRoot::new(&root);
    let key = key(FootprintCompleteness::Complete, "dev", false);
    let mut record = CacheRecord::new_blob(
        key,
        vec![CompatibilityField {
            family: "mizar".to_owned(),
            field_name: "version".to_owned(),
            value: "dev".to_owned(),
        }],
        b"blob output".to_vec(),
    );
    let CacheOutputDescriptor::Blob { blob, .. } = &mut record.header.output else {
        panic!("blob descriptor expected");
    };
    blob.digest = hash_hex(hash(99));

    assert!(matches!(
        store.insert(&record),
        Err(CacheStoreError::InvalidRecord {
            reason: CacheMiss::CorruptRecord
        })
    ));
    cleanup(root);
}

#[test]
fn conservative_complete_footprint_is_reusable() {
    let root = test_root("conservative_complete");
    let store = CacheStoreRoot::new(&root);
    let key = key(FootprintCompleteness::ConservativeComplete, "dev", false);
    let record = CacheRecord::new_inline(
        key.clone(),
        key.validation_inputs.toolchain_compatibility.clone(),
        b"coarse but complete".to_vec(),
    );

    assert_eq!(
        store.insert(&record).expect("insert"),
        CacheInsertOutcome::Inserted
    );
    assert!(matches!(store.lookup(&key), CacheLookupOutcome::Hit(_)));
    cleanup(root);
}

#[test]
fn incompatible_headers_miss_instead_of_erroring() {
    let root = test_root("incompatible_headers");
    let store = CacheStoreRoot::new(&root);
    let key = key(FootprintCompleteness::Complete, "dev", false);

    let mut unknown_schema = CacheRecord::new_inline(
        key.clone(),
        key.validation_inputs.toolchain_compatibility.clone(),
        b"schema".to_vec(),
    );
    unknown_schema.header.cache_record_schema_version = SchemaVersion::new("future");
    write_raw(&store, &key, encode_record(&unknown_schema));
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::UnknownSchema)
    );

    let mut unknown_toolchain = CacheRecord::new_inline(
        key.clone(),
        vec![CompatibilityField {
            family: "mizar".to_owned(),
            field_name: "version".to_owned(),
            value: "unknown".to_owned(),
        }],
        b"toolchain".to_vec(),
    );
    unknown_toolchain.header.cache_record_schema_version =
        SchemaVersion::new(CACHE_RECORD_SCHEMA_VERSION);
    write_raw(&store, &key, encode_record(&unknown_toolchain));
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::UnknownToolchain)
    );

    let mut key_schema_record = CacheRecord::new_inline(
        key.clone(),
        key.validation_inputs.toolchain_compatibility.clone(),
        b"key schema".to_vec(),
    );
    key_schema_record.header.key.cache_schema_version = SchemaVersion::new("future");
    write_raw(&store, &key, encode_record(&key_schema_record));
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::UnknownSchema)
    );

    let mut stale_requested_key = key.clone();
    stale_requested_key.cache_schema_version = SchemaVersion::new("future");
    let stale_requested_record = CacheRecord::new_inline(
        stale_requested_key.clone(),
        stale_requested_key
            .validation_inputs
            .toolchain_compatibility
            .clone(),
        b"stale key".to_vec(),
    );
    write_raw(
        &store,
        &stale_requested_key,
        encode_record(&stale_requested_record),
    );
    assert_eq!(
        store.lookup(&stale_requested_key),
        CacheLookupOutcome::Miss(CacheMiss::UnknownSchema)
    );

    let mut policy_key = key.clone();
    policy_key.validation_inputs.policy_compatibility[0].value = "incompatible".to_owned();
    let policy_record = CacheRecord::new_inline(
        policy_key.clone(),
        policy_key.validation_inputs.toolchain_compatibility.clone(),
        b"policy".to_vec(),
    );
    write_raw(&store, &policy_key, encode_record(&policy_record));
    assert_eq!(
        store.lookup(&policy_key),
        CacheLookupOutcome::Miss(CacheMiss::PolicyIncompatible)
    );

    let mut output_schema_key = key.clone();
    output_schema_key.schema_versions[0].version = SchemaVersion::new("unsupported");
    let output_schema_record = CacheRecord::new_inline(
        output_schema_key.clone(),
        output_schema_key
            .validation_inputs
            .toolchain_compatibility
            .clone(),
        b"output schema".to_vec(),
    );
    write_raw(
        &store,
        &output_schema_key,
        encode_record(&output_schema_record),
    );
    assert_eq!(
        store.lookup(&output_schema_key),
        CacheLookupOutcome::Miss(CacheMiss::UnknownSchema)
    );
    cleanup(root);
}

#[test]
fn uncacheable_and_incomplete_records_miss_without_disk_trust() {
    let root = test_root("uncacheable");
    let store = CacheStoreRoot::new(&root);
    let base_key = key(FootprintCompleteness::Complete, "dev", false);
    let mut record = CacheRecord::new_inline(
        base_key.clone(),
        base_key.validation_inputs.toolchain_compatibility.clone(),
        b"uncacheable".to_vec(),
    );
    record.header.uncacheable = true;
    write_raw(&store, &base_key, encode_record(&record));
    assert_eq!(
        store.lookup(&base_key),
        CacheLookupOutcome::Miss(CacheMiss::Uncacheable)
    );

    let incomplete = key(FootprintCompleteness::IncompleteUncacheable, "dev", false);
    assert_eq!(
        store.lookup(&incomplete),
        CacheLookupOutcome::Miss(CacheMiss::IncompleteFootprint)
    );

    let unsupported = key(FootprintCompleteness::Unsupported, "dev", false);
    assert_eq!(
        store.lookup(&unsupported),
        CacheLookupOutcome::Miss(CacheMiss::UnsupportedFootprint)
    );

    let outcome = CacheKeyBuilder::new(request(
        FootprintCompleteness::IncompleteUncacheable,
        "dev",
        false,
    ))
    .build();
    assert_eq!(
        store.lookup_key_outcome(&outcome),
        CacheLookupOutcome::Miss(CacheMiss::Uncacheable)
    );
    cleanup(root);
}

#[test]
fn missing_dependency_and_proof_validation_inputs_miss_fail_closed() {
    let root = test_root("missing_validation");
    let store = CacheStoreRoot::new(&root);

    let mut dependency_key = key(FootprintCompleteness::Complete, "dev", false);
    dependency_key
        .validation_inputs
        .dependency_artifacts
        .push(DependencyArtifactAvailability {
            package_id: "dep".to_owned(),
            module_path: "dep.alpha".to_owned(),
            artifact_kind: "mizir".to_owned(),
            artifact_path: "build/missing.mizir.json".to_owned(),
            domain: "artifact".to_owned(),
            digest: hash(80),
        });
    let dependency_record = CacheRecord::new_inline(
        dependency_key.clone(),
        dependency_key
            .validation_inputs
            .toolchain_compatibility
            .clone(),
        b"dependency".to_vec(),
    );
    write_raw(&store, &dependency_key, encode_record(&dependency_record));
    assert_eq!(
        store.lookup(&dependency_key),
        CacheLookupOutcome::Miss(CacheMiss::DependencyUnavailable)
    );

    let mut mismatched_dependency_key = key(FootprintCompleteness::Complete, "dev", true);
    let mismatched_artifact_path = format!(
        "{}/deps/mismatch.mizir.json",
        root.file_name()
            .and_then(|name| name.to_str())
            .expect("test root has utf-8 file name")
    );
    let mismatched_artifact_on_disk = root.join("deps/mismatch.mizir.json");
    fs::create_dir_all(
        mismatched_artifact_on_disk
            .parent()
            .expect("artifact path has parent"),
    )
    .expect("mkdir dependency artifact parent");
    fs::write(&mismatched_artifact_on_disk, b"actual artifact bytes")
        .expect("write mismatched dependency artifact");
    mismatched_dependency_key
        .validation_inputs
        .dependency_artifacts
        .push(DependencyArtifactAvailability {
            package_id: "dep".to_owned(),
            module_path: "dep.beta".to_owned(),
            artifact_kind: "mizir".to_owned(),
            artifact_path: mismatched_artifact_path,
            domain: "artifact".to_owned(),
            digest: dependency_artifact_hash("artifact", b"expected artifact bytes"),
        });
    let mismatched_dependency_record = CacheRecord::new_inline(
        mismatched_dependency_key.clone(),
        mismatched_dependency_key
            .validation_inputs
            .toolchain_compatibility
            .clone(),
        b"mismatched dependency".to_vec(),
    );
    write_raw(
        &store,
        &mismatched_dependency_key,
        encode_record(&mismatched_dependency_record),
    );
    assert_eq!(
        store.lookup(&mismatched_dependency_key),
        CacheLookupOutcome::Miss(CacheMiss::DependencyUnavailable)
    );

    let mut proof_key = key(FootprintCompleteness::Complete, "dev", true);
    proof_key.validation_inputs.obligation_anchor_fingerprint = Some(hash(81));
    proof_key.validation_inputs.proof_reuse_schema_versions = vec![NamedSchemaVersion {
        schema_family: "proof-reuse".to_owned(),
        name: "metadata".to_owned(),
        version: SchemaVersion::new("1.0"),
    }];
    proof_key
        .validation_inputs
        .proof_reuse_evidence_identities
        .push(ProofReuseEvidenceIdentity {
            obligation_anchor_fingerprint: hash(81),
            evidence_kind: "kernel_verified".to_owned(),
            witness_or_discharge_domain: "witness".to_owned(),
            witness_or_discharge_digest: hash(82),
        });
    proof_key.validation_inputs.selected_proof_witness_hash = Some(named("witness", 82));
    proof_key.validation_inputs.proof_reuse_validation_hash = None;
    let proof_record = CacheRecord::new_inline(
        proof_key.clone(),
        proof_key.validation_inputs.toolchain_compatibility.clone(),
        b"proof".to_vec(),
    );
    write_raw(&store, &proof_key, encode_record(&proof_record));
    assert_eq!(
        store.lookup(&proof_key),
        CacheLookupOutcome::Miss(CacheMiss::ProofReuseInvalid)
    );

    let mut proof_phase_key = key(FootprintCompleteness::Complete, "dev", false);
    proof_phase_key.phase = PipelinePhase::new("proof");
    let proof_phase_record = CacheRecord::new_inline(
        proof_phase_key.clone(),
        proof_phase_key
            .validation_inputs
            .toolchain_compatibility
            .clone(),
        b"proof phase".to_vec(),
    );
    write_raw(&store, &proof_phase_key, encode_record(&proof_phase_record));
    assert_eq!(
        store.lookup(&proof_phase_key),
        CacheLookupOutcome::Miss(CacheMiss::ProofReuseInvalid)
    );

    let mut proof_phase_with_vc_key = key(FootprintCompleteness::Complete, "dev", true);
    proof_phase_with_vc_key.phase = PipelinePhase::new("proof");
    proof_phase_with_vc_key
        .validation_inputs
        .canonical_vc_fingerprint = Some(hash(83));
    proof_phase_with_vc_key
        .validation_inputs
        .local_context_fingerprint = Some(hash(84));
    proof_phase_with_vc_key
        .validation_inputs
        .dependency_slice_fingerprints
        .push(DependencySliceHash {
            slice_kind: "theorem".to_owned(),
            owner: "dep.alpha".to_owned(),
            name: "T1".to_owned(),
            domain: "domain/dependency-slice".to_owned(),
            digest: hash(85),
        });
    let proof_phase_with_vc_record = CacheRecord::new_inline(
        proof_phase_with_vc_key.clone(),
        proof_phase_with_vc_key
            .validation_inputs
            .toolchain_compatibility
            .clone(),
        b"proof phase with vc".to_vec(),
    );
    write_raw(
        &store,
        &proof_phase_with_vc_key,
        encode_record(&proof_phase_with_vc_record),
    );
    assert_eq!(
        store.lookup(&proof_phase_with_vc_key),
        CacheLookupOutcome::Miss(CacheMiss::ProofReuseInvalid)
    );
    cleanup(root);
}

#[test]
fn corrupted_records_and_output_hash_mismatches_are_misses() {
    let root = test_root("corrupt");
    let store = CacheStoreRoot::new(&root);
    let key = key(FootprintCompleteness::Complete, "dev", false);
    write_raw(&store, &key, b"not a record".to_vec());
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
    );

    write_raw(&store, &key, encode_raw_record(b"{}\n", b"payload"));
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
    );

    write_raw(
        &store,
        &key,
        encode_raw_record(b"{\"a\":1,\"a\":2}\n", b"payload"),
    );
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
    );

    let record = CacheRecord::new_inline(
        key.clone(),
        key.validation_inputs.toolchain_compatibility.clone(),
        b"payload".to_vec(),
    );
    let mut bytes = encode_record(&record);
    let last = bytes.last_mut().expect("encoded record has payload byte");
    *last ^= 0xff;
    write_raw(&store, &key, bytes);
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
    );

    let mut truncated = encode_record(&record);
    truncated.pop();
    write_raw(&store, &key, truncated);
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
    );
    cleanup(root);
}

#[test]
fn exact_key_path_still_validates_embedded_key_and_diagnostic_refs() {
    let root = test_root("embedded_key");
    let store = CacheStoreRoot::new(&root);
    let requested = key(FootprintCompleteness::Complete, "dev", false);
    let embedded = key(FootprintCompleteness::Complete, "dev", true);
    let mismatched = CacheRecord::new_inline(
        embedded.clone(),
        embedded.validation_inputs.toolchain_compatibility.clone(),
        b"wrong key".to_vec(),
    );
    write_raw(&store, &requested, encode_record(&mismatched));
    assert_eq!(
        store.lookup(&requested),
        CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
    );
    fs::remove_file(store.record_path(&requested)).expect("remove mismatched raw record");

    let mut record = CacheRecord::new_inline(
        requested.clone(),
        requested.validation_inputs.toolchain_compatibility.clone(),
        b"diagnostic".to_vec(),
    );
    record.header.diagnostic_refs.push(DiagnosticRefHash {
        diagnostic_ref_kind: "cache_miss_explanation".to_owned(),
        diagnostic_ref_hash: hash(90),
    });
    store.insert(&record).expect("insert diagnostic record");
    let CacheLookupOutcome::Hit(hit) = store.lookup(&requested) else {
        panic!("expected hit");
    };
    assert_eq!(hit.header.diagnostic_refs, record.header.diagnostic_refs);
    cleanup(root);
}

#[test]
fn record_write_order_does_not_change_lookup_result() {
    let root = test_root("write_order");
    let store = CacheStoreRoot::new(&root);
    let first = key(FootprintCompleteness::Complete, "dev", false);
    let second = key(FootprintCompleteness::Complete, "dev", true);
    let first_record = CacheRecord::new_inline(
        first.clone(),
        first.validation_inputs.toolchain_compatibility.clone(),
        b"first".to_vec(),
    );
    let second_record = CacheRecord::new_inline(
        second.clone(),
        second.validation_inputs.toolchain_compatibility.clone(),
        b"second".to_vec(),
    );

    store.insert(&second_record).expect("insert second");
    store.insert(&first_record).expect("insert first");

    let CacheLookupOutcome::Hit(hit) = store.lookup(&first) else {
        panic!("expected first hit");
    };
    assert_eq!(hit.output, b"first");
    let CacheLookupOutcome::Hit(hit) = store.lookup(&second) else {
        panic!("expected second hit");
    };
    assert_eq!(hit.output, b"second");
    cleanup(root);
}

#[test]
fn divergent_same_key_insert_loses_without_overwriting_existing_record() {
    let root = test_root("divergent_insert");
    let store = CacheStoreRoot::new(&root);
    let key = key(FootprintCompleteness::Complete, "dev", false);
    let original = CacheRecord::new_inline(
        key.clone(),
        key.validation_inputs.toolchain_compatibility.clone(),
        b"original".to_vec(),
    );
    let divergent = CacheRecord::new_inline(
        key.clone(),
        key.validation_inputs.toolchain_compatibility.clone(),
        b"divergent".to_vec(),
    );

    assert_eq!(
        store.insert(&original).expect("insert original"),
        CacheInsertOutcome::Inserted
    );
    assert!(matches!(
        store.insert(&divergent),
        Err(CacheStoreError::DivergentRecord { .. })
    ));

    let CacheLookupOutcome::Hit(hit) = store.lookup(&key) else {
        panic!("expected original hit");
    };
    assert_eq!(hit.output, b"original");
    cleanup(root);
}

fn key(completeness: FootprintCompleteness, toolchain_value: &str, alternate: bool) -> CacheKey {
    match CacheKeyBuilder::new(request(completeness, toolchain_value, alternate)).build() {
        CacheKeyBuildOutcome::Cacheable(key) | CacheKeyBuildOutcome::Uncacheable(key) => key,
        CacheKeyBuildOutcome::NoKey(rejection) => panic!("{rejection}"),
    }
}

fn request(
    completeness: FootprintCompleteness,
    toolchain_value: &str,
    alternate: bool,
) -> CacheKeyRequest {
    CacheKeyRequest {
        cache_schema_version: SchemaVersion::default(),
        phase: PipelinePhase::new("resolve"),
        work_unit: WorkUnit::new(if alternate {
            "module:alpha#resolve:2"
        } else {
            "module:alpha#resolve:1"
        }),
        source_identity: None,
        input_hashes: vec![named("source", if alternate { 42 } else { 2 })],
        dependency_hashes: vec![DependencyHash {
            dependency_kind: "interface".to_owned(),
            package_id: "dep".to_owned(),
            module_path: "dep.alpha".to_owned(),
            name: "artifact".to_owned(),
            domain: "domain/interface".to_owned(),
            digest: hash(3),
        }],
        dependency_slices: Vec::new(),
        config_hash: hash(4),
        schema_versions: vec![NamedSchemaVersion {
            schema_family: "resolve".to_owned(),
            name: "output".to_owned(),
            version: SchemaVersion::new("1.0"),
        }],
        policy_fingerprint: PolicyFingerprint::new(hash(5)),
        validation_inputs: validation_inputs(completeness, toolchain_value),
    }
}

fn validation_inputs(
    completeness: FootprintCompleteness,
    toolchain_value: &str,
) -> crate::cache_key::CacheValidationInputs {
    crate::cache_key::CacheValidationInputs {
        cache_schema_compatibility: SchemaVersion::default(),
        toolchain_compatibility: vec![CompatibilityField {
            family: "mizar".to_owned(),
            field_name: "version".to_owned(),
            value: toolchain_value.to_owned(),
        }],
        dependency_artifacts: Vec::new(),
        footprint_completeness: completeness,
        uncacheable: false,
        policy_compatibility: vec![CompatibilityField {
            family: "proof-policy".to_owned(),
            field_name: "require_kernel_certificates".to_owned(),
            value: "true".to_owned(),
        }],
        canonical_vc_fingerprint: None,
        local_context_fingerprint: None,
        dependency_slice_fingerprints: Vec::new(),
        obligation_anchor_fingerprint: None,
        selected_proof_witness_hash: None,
        deterministic_discharge_hash: None,
        proof_reuse_schema_versions: Vec::new(),
        proof_reuse_validation_hash: None,
        proof_reuse_evidence_identities: Vec::new(),
        diagnostic_refs: Vec::new(),
    }
}

fn named(name: &str, seed: u8) -> NamedHash {
    NamedHash {
        name: name.to_owned(),
        domain: format!("domain/{name}"),
        digest: hash(seed),
    }
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

fn write_raw(store: &CacheStoreRoot, key: &CacheKey, bytes: Vec<u8>) {
    let path = store.record_path(key);
    fs::create_dir_all(path.parent().expect("record path has parent")).expect("mkdir");
    fs::write(path, bytes).expect("write raw record");
}

fn encode_raw_record(header: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(CACHE_RECORD_MAGIC);
    encoded.extend_from_slice(&RECORD_FORMAT_VERSION.to_le_bytes());
    encoded.extend_from_slice(&(header.len() as u64).to_le_bytes());
    encoded.extend_from_slice(header);
    encoded.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    encoded.extend_from_slice(payload);
    encoded
}

fn test_root(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("mizar-cache-store-{name}-{}", std::process::id()));
    cleanup(root.clone());
    root
}

fn cleanup(root: PathBuf) {
    let _ = fs::remove_dir_all(root);
}
