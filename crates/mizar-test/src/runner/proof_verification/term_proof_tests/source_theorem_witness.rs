use super::*;

#[test]
fn source_theorem_witness_file_matches_genuine_artifact_transport() -> Result<(), String> {
    use mizar_artifact::{
        proof_witness::{EvidenceKind, ProofStatus, ProofWitnessRef},
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{
            CanonicalJson, PublishedArtifactPath, PublishedArtifactReadOptions, SchemaVersion,
            artifact_hash_domain, canonical_json_bytes, read_published_artifact,
            write_published_artifact,
        },
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_proof::{
        policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
        status::{
            ArtifactPublicationGap, ProjectedProofStatus,
            source_theorem_reflexivity_witness_draft_from_original as acquire,
            source_theorem_reflexivity_witness_file_candidate as candidate,
        },
        witness_store::{read_source_theorem_witness_file as recover, stage},
    };
    use mizar_resolve::{resolved_ast::ModuleId, symbols::source_theorem_module_original_inputs};
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SnapshotRegistry, SourceInput, SourceOrigin, SourceOriginInput,
        ToolchainInfo, WorkspaceRoot, hash_text,
    };
    use mizar_vc::{dependency_slice::DependencySliceCompleteness, vc_ir::VcStatus};

    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/owners.miz");
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let lockfile_hash = hash_text(include_str!("../../../../../../Cargo.lock"));
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let domain = artifact_hash_domain(
        "mizar-proof/source-theorem-witness-file",
        SchemaVersion::new(1, 0),
    );
    for (index, text) in [
        original.to_owned(),
        original.to_owned(),
        original.replace("T:", "Renamed:").replace(
            "for X being set holds X = X",
            "for Bound being set holds Bound = Bound",
        ),
        original.replace(
            "let X be set; thus X = X",
            "let Local be set; thus Local = Local",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        fs::write(
            &path,
            if index % 2 == 0 {
                &text
            } else {
                "stale disk content"
            },
        )
        .unwrap();
        let input = SourceInput {
            package_id: PackageId::new("witness-file-test"),
            module_path: ModulePath::new("owners"),
            normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
            edition: Edition::new("2026"),
            origin: if index % 2 == 0 {
                SourceOriginInput::Disk { path: path.clone() }
            } else {
                SourceOriginInput::OpenBuffer {
                    uri: format!("file://{}", path.display()),
                    expected_version: 3,
                    actual_version: 3,
                    text: text.clone().into(),
                }
            },
        };
        let (snapshot, lease, mut loaded) = registry
            .load_snapshot(
                ids.next_request_id().unwrap(),
                (
                    WorkspaceRoot::new(root.to_str().unwrap()),
                    Vec::new(),
                    lockfile_hash,
                    ToolchainInfo::new("caller-recorded-witness-file-test"),
                    Policy::default().policy_fingerprint().hash(),
                ),
                vec![(DiskSourceLoader::new(&root), input.clone())],
            )
            .unwrap();
        let (loaded, received) = loaded.pop().unwrap();
        assert_eq!(received, input);
        assert_eq!(loaded.source_hash, hash_text(&text));
        assert_eq!(snapshot.source_versions[0].source_hash, loaded.source_hash);
        assert_eq!(matches!(loaded.origin, SourceOrigin::Disk), index % 2 == 0);
        let unit = source_unit_from_loaded(loaded, path.clone());
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run_loaded(unit.clone())
        .unwrap();
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        let module = ModuleId::new(input.package_id.clone(), input.module_path.clone());
        let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
        let before = (source.clone(), symbols.clone(), snapshot.clone());
        for policy in [
            Policy::default(),
            Policy::release().with_require_kernel_certificates(true),
            Policy::development().with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
            Policy::default().with_profile_id("witness-file-policy"),
            Policy::default().with_checker_schema_version(2),
        ]
        .into_iter()
        .take(if index == 0 { 5 } else { 1 })
        {
            let witness_path = format!("proof-witnesses/file-{index}/checked.json");
            let supplied = (policy.clone(), witness_path.clone());
            let mut previous = None;
            for _ in 0..2 {
                let expected = acquire(&source, &symbols, &snapshot, &policy, &witness_path)?;
                let actual = candidate(&source, &symbols, &snapshot, &policy, &witness_path)?;
                assert_eq!(actual.0, expected);
                let draft = &expected.2;
                let metadata = draft.kernel_acceptance();
                let mut acceptance = std::collections::BTreeMap::from([
                    (
                        "checker_schema_version".to_owned(),
                        CanonicalJson::string(metadata.checker_schema_version.to_string()),
                    ),
                    (
                        "evidence_schema_version".to_owned(),
                        CanonicalJson::string("2.0"),
                    ),
                    ("formula_context_hash".to_owned(), CanonicalJson::Null),
                ]);
                for (key, hash) in [
                    (
                        "kernel_profile_fingerprint",
                        &metadata.kernel_profile_fingerprint,
                    ),
                    (
                        "verifier_policy_fingerprint",
                        &metadata.verifier_policy_fingerprint,
                    ),
                    ("target_binding_hash", &metadata.target_binding_hash),
                    ("formula_evidence_hash", &metadata.formula_evidence_hash),
                    (
                        "substitution_evidence_hash",
                        &metadata.substitution_evidence_hash,
                    ),
                    ("provenance_hash", &metadata.provenance_hash),
                    ("accepted_result_hash", &metadata.accepted_result_hash),
                ] {
                    acceptance.insert(
                        key.to_owned(),
                        CanonicalJson::string(hash.to_artifact_hash_string()),
                    );
                }
                let mut hex = String::new();
                for byte in draft.payload_bytes() {
                    hex.push_str(&format!("{byte:02x}"));
                }
                let mut file = std::collections::BTreeMap::new();
                for (key, value) in [
                    ("schema_version", "1.0".to_owned()),
                    (
                        "schema_family",
                        "mizar-proof/source-theorem-witness-file".to_owned(),
                    ),
                    ("obligation_id", draft.obligation_id().to_owned()),
                    (
                        "obligation_fingerprint",
                        draft.obligation_fingerprint().to_artifact_hash_string(),
                    ),
                    (
                        "payload_schema_family",
                        "mizar-proof/source-theorem-reflexivity-checked-wire".to_owned(),
                    ),
                    ("payload_schema_version", "2.0".to_owned()),
                    ("payload_bytes_hex", hex),
                ] {
                    file.insert(key.to_owned(), CanonicalJson::string(value));
                }
                file.insert(
                    "kernel_acceptance".to_owned(),
                    CanonicalJson::Object(acceptance),
                );
                let file = CanonicalJson::Object(file);
                let hash = ArtifactHashRef::new(
                    ArtifactHashClass::Artifact,
                    "mizar-proof/source-theorem-witness-file",
                    SchemaVersion::new(1, 0),
                    domain.hash(&file, &[]),
                );
                let reference = ProofWitnessRef {
                    schema_version: SchemaVersion::new(3, 0),
                    obligation_id: draft.obligation_id().to_owned(),
                    obligation_fingerprint: draft.obligation_fingerprint().clone(),
                    proof_status: ProofStatus::DischargedBuiltin,
                    evidence_kind: EvidenceKind::FormulaSubstitutionKernelEvidence,
                    witness_path: witness_path.clone(),
                    witness_artifact_hash: hash.clone(),
                    kernel_acceptance: metadata.clone(),
                };
                assert_eq!(actual.1, file);
                assert_eq!(actual.2, reference);
                assert_eq!(
                    actual.0.0.projected_status(),
                    ProjectedProofStatus::Accepted
                );
                assert_eq!(
                    actual.0.0.artifact_publication().gap(),
                    Some(ArtifactPublicationGap::DischargedBuiltinWitnessUnsupported)
                );
                assert_eq!(actual.0.1.status(), &VcStatus::Open);
                assert_eq!(
                    actual.0.1.completeness(),
                    DependencySliceCompleteness::IncompleteUncacheable
                );
                assert!(actual.0.1.requires_cache_miss());
                assert!(!actual.0.0.reuse_metadata().cache_reuse_predicate_complete());
                assert!(actual.0.0.trusted_used_axioms().is_none());
                assert!(draft.provenance().trusted_used_axioms_ref().is_none());
                assert!(draft.provenance().advisory_backend_ref().is_none());
                let staged = stage(draft.clone()).unwrap();
                assert!(staged.artifact_reference_candidate().is_none());
                assert_ne!(staged.staged_payload_hash(), &hash);
                assert!(!root.join(&witness_path).exists());
                let transport_root = root.join("unpublished-witness-files");
                let transport_path = PublishedArtifactPath::new(&witness_path).unwrap();
                let written = write_published_artifact(
                    &transport_root,
                    &transport_path,
                    &actual.1,
                    &domain,
                    &[],
                )
                .unwrap();
                assert_eq!(written.artifact_hash, hash.digest);
                assert_eq!(
                    fs::read(transport_root.join(&witness_path)).unwrap(),
                    canonical_json_bytes(&file)
                );
                let read = read_published_artifact(
                    &transport_root,
                    &transport_path,
                    PublishedArtifactReadOptions {
                        artifact_hash_domain: Some(&domain),
                        expected_artifact_hash: Some(hash.digest),
                        ..PublishedArtifactReadOptions::default()
                    },
                )
                .unwrap();
                assert_eq!(read.value, file);
                assert_eq!(read.artifact_hash, Some(hash.digest));
                let received_before = (read.clone(), actual.clone());
                assert_eq!(recover(&read, &actual.2)?, draft.payload_bytes());
                assert_eq!(recover(&read, &actual.2)?, draft.payload_bytes());
                assert_eq!((read, actual.clone()), received_before);
                if let Some(previous) = previous {
                    assert_eq!(actual, previous);
                }
                previous = Some(actual);
            }
            assert_eq!((policy, witness_path), supplied);
        }
        assert_eq!((source, symbols, snapshot.clone()), before);
        assert_eq!(registry.get(snapshot.id), Some(snapshot.clone()));
        registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_witness_file_rejects_invalid_schema_and_associations() -> Result<(), String> {
    use mizar_artifact::{
        proof_witness::{EvidenceKind, ProofStatus},
        registration_summary::ArtifactHashClass,
        store::{
            CanonicalJson, MinorVersionPolicy, PublishedArtifactPath, PublishedArtifactReadOptions,
            SchemaVersion, SchemaVersionSupport, artifact_hash_domain, read_published_artifact,
            write_published_artifact,
        },
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_proof::{
        policy::VerifierPolicy as Policy,
        status::{
            source_theorem_reflexivity_witness_draft_from_original as acquire,
            source_theorem_reflexivity_witness_file_candidate as candidate,
        },
        witness_store::read_source_theorem_witness_file as recover,
    };
    use mizar_resolve::{resolved_ast::ModuleId, symbols::source_theorem_module_original_inputs};
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SnapshotRegistry, SourceInput, SourceOriginInput, ToolchainInfo,
        WorkspaceRoot, hash_text,
    };

    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/owners.miz");
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    fs::write(&path, original).unwrap();
    let input = SourceInput {
        package_id: PackageId::new("witness-file-errors"),
        module_path: ModulePath::new("owners"),
        normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
        edition: Edition::new("2026"),
        origin: SourceOriginInput::Disk { path: path.clone() },
    };
    let lockfile_hash = hash_text(include_str!("../../../../../../Cargo.lock"));
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let policy = Policy::default();
    let (snapshot, lease, mut loaded) = registry
        .load_snapshot(
            ids.next_request_id().unwrap(),
            (
                WorkspaceRoot::new(root.to_str().unwrap()),
                Vec::new(),
                lockfile_hash,
                ToolchainInfo::new("caller-recorded-witness-file-errors"),
                policy.policy_fingerprint().hash(),
            ),
            vec![(DiskSourceLoader::new(&root), input.clone())],
        )
        .unwrap();
    let (loaded, received) = loaded.pop().unwrap();
    assert_eq!(received, input);
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
        super::super::super::ParseOnlyImportProvider,
        MizarParserSeam,
    )
    .run_loaded(source_unit_from_loaded(loaded, path))
    .unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.as_ref().unwrap();
    let module = ModuleId::new(input.package_id, input.module_path);
    let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
    let witness_path = "proof-witnesses/errors/checked.json";
    let before = (
        source.clone(),
        symbols.clone(),
        snapshot.clone(),
        policy.clone(),
    );
    let expected = acquire(&source, &symbols, &snapshot, &policy, witness_path)?;
    let actual = candidate(&source, &symbols, &snapshot, &policy, witness_path)?;
    assert_eq!(actual.0, expected);
    let domain = artifact_hash_domain(
        "mizar-proof/source-theorem-witness-file",
        SchemaVersion::new(1, 0),
    );
    let transport_root = root.join("unpublished-witness-file-errors");
    let transport_path = PublishedArtifactPath::new(witness_path).unwrap();
    let read_options = PublishedArtifactReadOptions {
        artifact_hash_domain: Some(&domain),
        ..PublishedArtifactReadOptions::default()
    };
    let CanonicalJson::Object(fields) = &actual.1 else {
        panic!("genuine object")
    };
    let support = SchemaVersionSupport::new(
        "mizar-proof/source-theorem-witness-file",
        1,
        0,
        MinorVersionPolicy::UpToSupported,
    );
    let mut mutations = vec![CanonicalJson::Null, CanonicalJson::string("not an object")];
    for header in [
        None,
        Some(CanonicalJson::Null),
        Some(CanonicalJson::string("bad")),
        Some(CanonicalJson::string("0.0")),
        Some(CanonicalJson::string("2.0")),
        Some(CanonicalJson::string("1.1")),
    ] {
        let mut value = fields.clone();
        if let Some(header) = header {
            value.insert("schema_version".into(), header);
        } else {
            value.remove("schema_version");
        }
        value.insert("payload_bytes_hex".into(), CanonicalJson::Null);
        value.insert("kernel_acceptance".into(), CanonicalJson::Null);
        mutations.push(CanonicalJson::Object(value));
    }
    for key in fields.keys() {
        let mut missing = fields.clone();
        missing.remove(key);
        mutations.push(CanonicalJson::Object(missing));
        let mut wrong_type = fields.clone();
        wrong_type.insert(key.clone(), CanonicalJson::Null);
        mutations.push(CanonicalJson::Object(wrong_type));
    }
    for (key, replacement) in [
        (
            "schema_family",
            CanonicalJson::string("foreign/witness-file"),
        ),
        ("unknown", CanonicalJson::Null),
        ("obligation_id", CanonicalJson::string("")),
        ("obligation_id", CanonicalJson::string("foreign-obligation")),
        (
            "obligation_fingerprint",
            CanonicalJson::string("invalid-hash"),
        ),
        (
            "payload_schema_family",
            CanonicalJson::string("mizar-proof/witness-payload"),
        ),
        ("payload_schema_version", CanonicalJson::string("1.0")),
    ] {
        let mut value = fields.clone();
        value.insert(key.into(), replacement);
        mutations.push(CanonicalJson::Object(value));
    }
    for hex in ["", "0", "AA", "0g", "00 0", "é", "0é0", "００"] {
        let mut value = fields.clone();
        value.insert("payload_bytes_hex".into(), CanonicalJson::string(hex));
        mutations.push(CanonicalJson::Object(value));
    }
    let CanonicalJson::Object(metadata) = &fields["kernel_acceptance"] else {
        panic!("genuine metadata")
    };
    for key in metadata.keys() {
        let mut missing = metadata.clone();
        missing.remove(key);
        let mut value = fields.clone();
        value.insert("kernel_acceptance".into(), CanonicalJson::Object(missing));
        mutations.push(CanonicalJson::Object(value));
    }
    for (key, value) in [
        ("unknown", CanonicalJson::Null),
        (
            "formula_context_hash",
            CanonicalJson::string("invalid-hash"),
        ),
        (
            "verifier_policy_fingerprint",
            CanonicalJson::string("invalid-policy"),
        ),
        ("evidence_schema_version", CanonicalJson::string("bad")),
    ] {
        let mut nested = metadata.clone();
        nested.insert(key.into(), value);
        let mut value = fields.clone();
        value.insert("kernel_acceptance".into(), CanonicalJson::Object(nested));
        mutations.push(CanonicalJson::Object(value));
    }
    for value in mutations {
        let written =
            write_published_artifact(&transport_root, &transport_path, &value, &domain, &[])
                .unwrap();
        let read = read_published_artifact(&transport_root, &transport_path, read_options).unwrap();
        assert_eq!(read.artifact_hash, Some(written.artifact_hash));
        let mut expected_reference = actual.2.clone();
        expected_reference.witness_artifact_hash.digest = written.artifact_hash;
        let received = (read.clone(), expected_reference.clone());
        let error = recover(&read, &expected_reference).unwrap_err();
        if let CanonicalJson::Object(value) = &value {
            let header = match value.get("schema_version") {
                Some(CanonicalJson::String(version)) => Some(version.as_str()),
                None => None,
                Some(_) => Some(""),
            };
            if let Err(expected) = support.check_at_path(header, witness_path) {
                assert_eq!(error, expected.to_string());
            } else if value.get("schema_family") != fields.get("schema_family") {
                for context in [
                    "mizar-proof/source-theorem-witness-file",
                    "1.0",
                    "1.0..=1.0",
                    witness_path,
                ] {
                    assert!(error.contains(context), "{error}");
                }
            }
        }
        assert_eq!((read, expected_reference), received);
    }
    write_published_artifact(&transport_root, &transport_path, &actual.1, &domain, &[]).unwrap();
    let read = read_published_artifact(
        &transport_root,
        &transport_path,
        PublishedArtifactReadOptions {
            artifact_hash_domain: Some(&domain),
            expected_artifact_hash: Some(actual.2.witness_artifact_hash.digest),
            ..PublishedArtifactReadOptions::default()
        },
    )
    .unwrap();
    let mut changed = read.clone();
    let CanonicalJson::Object(changed_fields) = &mut changed.value else {
        panic!("genuine object")
    };
    assert_ne!(
        changed_fields["payload_bytes_hex"],
        CanonicalJson::string("00")
    );
    changed_fields.insert("payload_bytes_hex".into(), CanonicalJson::string("00"));
    assert_eq!(
        recover(&changed, &actual.2).unwrap_err(),
        "source theorem witness file read hash mismatch"
    );
    for hash in [None, Some(hash_text("wrong-read-hash"))] {
        let mut wrong = read.clone();
        wrong.artifact_hash = hash;
        assert_eq!(
            recover(&wrong, &actual.2).unwrap_err(),
            "source theorem witness file read hash mismatch"
        );
    }
    let foreign_path = PublishedArtifactPath::new("proof-witnesses/foreign/checked.json").unwrap();
    write_published_artifact(&transport_root, &foreign_path, &actual.1, &domain, &[]).unwrap();
    let foreign_read =
        read_published_artifact(&transport_root, &foreign_path, read_options).unwrap();
    assert!(recover(&foreign_read, &actual.2).is_err());
    for index in 0..11 {
        let mut reference = actual.2.clone();
        match index {
            0 => reference.schema_version = SchemaVersion::new(2, 0),
            1 => reference.obligation_id.push_str("-foreign"),
            2 => reference.obligation_fingerprint.digest = hash_text("foreign-original"),
            3 => reference.proof_status = ProofStatus::KernelVerified,
            4 => reference.evidence_kind = EvidenceKind::BuiltinCertificate,
            5 => reference.witness_path = foreign_path.as_str().into(),
            6 => reference.witness_artifact_hash.digest = hash_text("foreign-file"),
            7 => {
                reference
                    .kernel_acceptance
                    .verifier_policy_fingerprint
                    .digest = hash_text("foreign-policy")
            }
            8 => {
                reference.kernel_acceptance.formula_context_hash =
                    Some(reference.obligation_fingerprint.clone())
            }
            9 => reference.witness_artifact_hash.schema_family = "foreign/file".into(),
            10 => reference.witness_artifact_hash.class = ArtifactHashClass::Interface,
            _ => unreachable!(),
        }
        assert!(recover(&read, &reference).is_err());
    }
    for index in 0..4 {
        let mut reference = actual.2.clone();
        let mut value = fields.clone();
        if index < 3 {
            match index {
                0 => reference.obligation_fingerprint.schema_family = "foreign/original".into(),
                1 => reference.obligation_fingerprint.schema_version = SchemaVersion::new(2, 0),
                2 => reference.obligation_fingerprint.class = ArtifactHashClass::Artifact,
                _ => unreachable!(),
            }
            value.insert(
                "obligation_fingerprint".into(),
                CanonicalJson::string(reference.obligation_fingerprint.to_artifact_hash_string()),
            );
        } else {
            reference.kernel_acceptance.evidence_schema_version = SchemaVersion::new(1, 0);
            let mut nested = metadata.clone();
            nested.insert(
                "evidence_schema_version".into(),
                CanonicalJson::string("1.0"),
            );
            value.insert("kernel_acceptance".into(), CanonicalJson::Object(nested));
        }
        let value = CanonicalJson::Object(value);
        reference.witness_artifact_hash.digest = domain.hash(&value, &[]);
        write_published_artifact(&transport_root, &transport_path, &value, &domain, &[]).unwrap();
        let read = read_published_artifact(&transport_root, &transport_path, read_options).unwrap();
        assert!(recover(&read, &reference).is_err());
    }
    let disabled = Policy::default().with_kernel_evidence_formats([]);
    for policy in [&policy, &disabled] {
        for path in ["", "../escape.json", witness_path] {
            if policy == &before.3 && path == witness_path {
                continue;
            }
            let expected = acquire(&source, &symbols, &snapshot, policy, path).unwrap_err();
            assert_eq!(
                candidate(&source, &symbols, &snapshot, policy, path).unwrap_err(),
                expected
            );
        }
    }
    let foreign_ast = parse(&original.replace("T:", "Foreign:"));
    let (_, foreign) = source_theorem_module_original_inputs(&foreign_ast, &module)?.unwrap();
    let expected = acquire(&source, &foreign, &snapshot, &disabled, "").unwrap_err();
    assert_eq!(
        candidate(&source, &foreign, &snapshot, &disabled, "").unwrap_err(),
        expected
    );
    assert_eq!((source, symbols, snapshot.clone(), policy), before);
    assert_eq!(registry.get(snapshot.id), Some(snapshot.clone()));
    registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    fs::remove_dir_all(root).unwrap();
    Ok(())
}
