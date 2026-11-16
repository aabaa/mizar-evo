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
    let mut first_file_staged = None;
    let mut first_bound_projection = None;
    let mut did_publication_round_trip = false;
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
                let file_staged = draft.clone().stage_source_theorem_file(&read, &actual.2)?;
                assert_eq!(file_staged.artifact_reference_candidate(), Some(&actual.2));
                let bound = actual
                    .0
                    .0
                    .clone()
                    .with_source_theorem_witness(&file_staged)?;
                assert_eq!(
                    bound.artifact_publication(),
                    mizar_proof::status::ArtifactStatusPublication::Publishable(
                        mizar_proof::status::CurrentArtifactObligationStatus::Accepted
                    )
                );
                assert_eq!(
                    bound.accepted_witness_obligation_id(),
                    Some(actual.2.obligation_id.as_str())
                );
                assert_eq!(bound.reuse_metadata(), actual.0.0.reuse_metadata());
                assert_eq!(
                    bound.trusted_used_axioms(),
                    file_staged.provenance().trusted_used_axioms_ref()
                );
                assert!(
                    bound
                        .reuse_metadata()
                        .selected_proof_witness_hash()
                        .is_none()
                );
                let mut wrong_identity = actual.2.clone();
                wrong_identity.obligation_id.push_str("-other");
                assert!(
                    draft
                        .clone()
                        .stage_source_theorem_file(&read, &wrong_identity)
                        .is_err()
                );
                let mut wrong_status = actual.2.clone();
                wrong_status.proof_status = ProofStatus::KernelVerified;
                assert!(
                    draft
                        .clone()
                        .stage_source_theorem_file(&read, &wrong_status)
                        .is_err()
                );
                if index == 0 && first_bound_projection.is_none() {
                    first_bound_projection = Some(actual.0.0.clone());
                    first_file_staged = Some(file_staged.clone());
                } else if index <= 1 {
                    let baseline = first_bound_projection.as_ref().unwrap();
                    let matches_identity = baseline == &actual.0.0;
                    assert_eq!(
                        baseline
                            .clone()
                            .with_source_theorem_witness(&file_staged)
                            .is_ok(),
                        matches_identity
                    );
                } else {
                    assert!(
                        actual
                            .0
                            .0
                            .clone()
                            .with_source_theorem_witness(first_file_staged.as_ref().unwrap())
                            .is_err()
                    );
                }

                if index == 0 && previous.is_none() && !did_publication_round_trip {
                    did_publication_round_trip = true;
                    use mizar_artifact::manifest::ManifestProofWitnessEntry;
                    use mizar_artifact::{
                        manifest::{
                            ArtifactManifest, ManifestCommitOptions, ManifestFileReadOptions,
                            ManifestProvenance, ManifestTransaction, ModuleArtifactEntry,
                            PackageIdentity, current_schema_version as manifest_version,
                            read_manifest_file,
                        },
                        module_summary::ModuleSummaryIdentity,
                        registration_summary::{ArtifactHashClass, ArtifactHashRef},
                        store::{
                            artifact_hash_domain, canonical_json_bytes, write_published_artifact,
                        },
                        verified_artifact::{
                            BuildProvenance, ObligationMetadata, ObligationStatus,
                            VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifact,
                            VerifiedArtifactReadOptions, artifact_hash_excluded_paths,
                            builtin_verified_artifact_json, read_builtin_verified_artifact,
                            read_verified_artifact,
                        },
                    };

                    let reuse = bound.reuse_metadata();
                    let interface_ref = |family: &str, digest| {
                        ArtifactHashRef::new(
                            ArtifactHashClass::Interface,
                            family,
                            SchemaVersion::new(1, 0),
                            digest,
                        )
                    };
                    let module = ModuleSummaryIdentity {
                        package_id: input.package_id.as_str().to_owned(),
                        package_version: None,
                        lockfile_identity: None,
                        module_path: input.module_path.as_str().to_owned(),
                        language_edition: "2026".to_owned(),
                    };
                    let source_hash = snapshot.source_versions[0].source_hash;
                    let mut artifact = VerifiedArtifact {
                        schema_version: SchemaVersion::new(2, 0),
                        module: module.clone(),
                        source_file: "src/owners.miz".to_owned(),
                        source_hash,
                        verified_at: None,
                        interface_hash: hash_text("integration interface"),
                        implementation_hash: hash_text("integration implementation"),
                        exports: Vec::new(),
                        expressions: Vec::new(),
                        statements: Vec::new(),
                        obligations: vec![ObligationMetadata {
                            obligation_id: bound.obligation_id().to_owned(),
                            obligation_anchor: Some(reuse.obligation_anchor().as_str().to_owned()),
                            owner_origin_id: None,
                            source_range: mizar_artifact::module_summary::SourceRangeSummary {
                                start_byte: 0,
                                end_byte: original.len() as u64,
                            },
                            obligation_kind: "source_theorem_reflexivity".to_owned(),
                            statement_summary: original.to_owned(),
                            obligation_fingerprint: actual.2.obligation_fingerprint.clone(),
                            vc_fingerprint: interface_ref(
                                "mizar-proof/source-theorem-vc",
                                reuse.vc_fingerprint(),
                            ),
                            local_context_fingerprint: interface_ref(
                                "mizar-proof/source-theorem-context",
                                reuse.local_context_fingerprint(),
                            ),
                            dependency_slice_fingerprint: interface_ref(
                                "mizar-proof/source-theorem-dependency-slice",
                                reuse.dependency_slice_fingerprint(),
                            ),
                            verifier_policy_fingerprint: actual
                                .2
                                .kernel_acceptance
                                .verifier_policy_fingerprint
                                .clone(),
                            status: ObligationStatus::Accepted,
                            accepted_witness_obligation_id: bound
                                .accepted_witness_obligation_id()
                                .map(str::to_owned),
                            deterministic_discharge_hash: None,
                            diagnostic_ref: None,
                        }],
                        proof_witnesses: vec![actual.2.clone()],
                        diagnostics: Vec::new(),
                        provenance: BuildProvenance {
                            toolchain: "source-theorem-witness-test".to_owned(),
                            language_edition: "2026".to_owned(),
                            lockfile_hash: ArtifactHashRef::new(
                                ArtifactHashClass::Artifact,
                                "mizar-build/lockfile",
                                SchemaVersion::new(1, 0),
                                lockfile_hash,
                            ),
                            verifier_config_hash: actual
                                .2
                                .kernel_acceptance
                                .verifier_policy_fingerprint
                                .clone(),
                            dependency_artifact_hashes: Vec::new(),
                            cache_key: None,
                            captured_source: None,
                        },
                    };
                    let (interface_hash, implementation_hash) = artifact
                        .refresh_hashes()
                        .map_err(|error| error.to_string())?;
                    let main_path = "main.mizir.json";
                    let main_json = builtin_verified_artifact_json(&artifact)
                        .map_err(|error| error.to_string())?;
                    let main_domain = artifact_hash_domain(
                        VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                        SchemaVersion::new(2, 0),
                    );
                    let main_write = write_published_artifact(
                        &transport_root,
                        &PublishedArtifactPath::new(main_path).unwrap(),
                        &main_json,
                        &main_domain,
                        &artifact_hash_excluded_paths(),
                    )
                    .unwrap();
                    assert_eq!(
                        fs::read(transport_root.join(main_path)).unwrap(),
                        canonical_json_bytes(&main_json)
                    );
                    let main_read = read_published_artifact(
                        &transport_root,
                        &PublishedArtifactPath::new(main_path).unwrap(),
                        PublishedArtifactReadOptions {
                            artifact_hash_domain: Some(&main_domain),
                            hash_excluded_paths: &artifact_hash_excluded_paths(),
                            expected_artifact_hash: Some(main_write.artifact_hash),
                        },
                    )
                    .unwrap();
                    assert_eq!(
                        read_builtin_verified_artifact(
                            &main_read.value,
                            VerifiedArtifactReadOptions {
                                artifact_path: Some(main_path),
                                expected_module: Some(&module),
                                expected_interface_hash: Some(interface_hash),
                                expected_implementation_hash: Some(implementation_hash),
                            },
                        )
                        .unwrap(),
                        artifact
                    );
                    assert!(
                        read_verified_artifact(
                            &main_read.value,
                            VerifiedArtifactReadOptions::default(),
                        )
                        .is_err()
                    );
                    let module_entry = ModuleArtifactEntry {
                        module: module.clone(),
                        source_file: artifact.source_file.clone(),
                        source_hash,
                        artifact_file: main_path.to_owned(),
                        artifact_hash: ArtifactHashRef::new(
                            ArtifactHashClass::Artifact,
                            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                            SchemaVersion::new(2, 0),
                            main_write.artifact_hash,
                        ),
                        interface_hash: ArtifactHashRef::new(
                            ArtifactHashClass::Interface,
                            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                            SchemaVersion::new(2, 0),
                            interface_hash,
                        ),
                        implementation_hash: ArtifactHashRef::new(
                            ArtifactHashClass::Implementation,
                            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                            SchemaVersion::new(2, 0),
                            implementation_hash,
                        ),
                        module_summary_file: None,
                        module_summary_hash: None,
                        module_summary_interface_hash: None,
                        registration_summary_file: None,
                        registration_summary_hash: None,
                        registration_interface_hash: None,
                        proof_witnesses: vec![ManifestProofWitnessEntry {
                            obligation_id: actual.2.obligation_id.clone(),
                            obligation_fingerprint: actual.2.obligation_fingerprint.clone(),
                            witness_path: actual.2.witness_path.clone(),
                            witness_artifact_hash: actual.2.witness_artifact_hash.clone(),
                        }],
                        diagnostics_hash: None,
                    };
                    let manifest = ArtifactManifest {
                        schema_version: manifest_version(),
                        package: PackageIdentity {
                            package_id: module.package_id.clone(),
                            package_version: None,
                            lockfile_identity: None,
                        },
                        artifact_root: "test-artifact-root".to_owned(),
                        lockfile_hash: artifact.provenance.lockfile_hash.clone(),
                        toolchain: artifact.provenance.toolchain.clone(),
                        language_edition: "2026".to_owned(),
                        verifier_config_hash: artifact.provenance.verifier_config_hash.clone(),
                        modules: Vec::new(),
                        development_artifacts: Vec::new(),
                        provenance: ManifestProvenance {
                            generated_by: "mizar-test".to_owned(),
                            manifest_policy: "source-theorem-reflexivity".to_owned(),
                            transaction_format: "manifest-transaction-v1".to_owned(),
                        },
                    };
                    let mut transaction =
                        ManifestTransaction::begin(&transport_root, manifest, None).unwrap();
                    transaction.stage_module(module_entry.clone()).unwrap();
                    let commit = transaction
                        .commit(ManifestCommitOptions::default())
                        .unwrap();
                    let committed = read_manifest_file(
                        &transport_root,
                        ManifestFileReadOptions {
                            validate_references: true,
                            validate_witness_files: true,
                            validate_development_artifacts: true,
                            expected_manifest_hash: Some(commit.write().artifact_hash),
                        },
                    )
                    .unwrap();
                    assert_eq!(committed.manifest, *commit.manifest());
                    for altered_witnesses in [
                        Vec::new(),
                        vec![ManifestProofWitnessEntry {
                            witness_path: "proof-witnesses/other.json".to_owned(),
                            ..module_entry.proof_witnesses[0].clone()
                        }],
                    ] {
                        let mut altered_entry = module_entry.clone();
                        altered_entry.proof_witnesses = altered_witnesses;
                        let mut transaction = ManifestTransaction::begin(
                            &transport_root,
                            commit.manifest().clone(),
                            None,
                        )
                        .unwrap();
                        transaction.stage_module(altered_entry).unwrap();
                        assert!(
                            transaction
                                .commit(ManifestCommitOptions::default())
                                .is_err()
                        );
                    }
                    let token = mizar_proof::witness_store::CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                        &commit,
                        &transport_root,
                        &module,
                        file_staged.provenance().build_snapshot_fingerprint(),
                    )
                    .unwrap();
                    let published =
                        mizar_proof::witness_store::publish_ref(&file_staged, &token).unwrap();
                    assert_eq!(published.reference(), &actual.2);
                    assert_eq!(published.provenance(), file_staged.provenance());
                    assert!(matches!(
                        mizar_proof::witness_store::publish_ref(&staged, &token),
                        Err(mizar_proof::witness_store::ProofWitnessStoreError::UnsupportedWitnessPublication { .. })
                    ));
                    let algorithm_text = include_str!("../../../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz")
                        .replacen("    assert x = a;\n", "", 1);
                    let algorithm_policy =
                        Policy::default().with_instance_search_limits(Some([0, 0, 3, 0]));
                    super::with_source_algorithm_state_draft_inputs(
                        &algorithm_text,
                        &algorithm_policy,
                        |source, check, symbols, build| {
                            let product = mizar_proof::status::
                                try_source_algorithm_state_transitivity_witness_file_candidate(
                                    source,
                                    check,
                                    symbols,
                                    build,
                                    &algorithm_policy,
                                    "proof-witnesses/algorithm-state.json",
                                )
                                .unwrap()
                                .unwrap();
                            let file = &product.1;
                            let reference = &product.2;
                            let state_root = transport_root.join("algorithm-state-controls");
                            let state_domain = artifact_hash_domain(
                                "mizar-proof/source-algorithm-state-witness-file",
                                SchemaVersion::new(1, 0),
                            );
                            let state_path =
                                PublishedArtifactPath::new(&reference.witness_path).unwrap();
                            write_published_artifact(
                                &state_root,
                                &state_path,
                                file,
                                &state_domain,
                                &[],
                            )
                            .unwrap();
                            let state_read = read_published_artifact(
                                &state_root,
                                &state_path,
                                PublishedArtifactReadOptions {
                                    artifact_hash_domain: Some(&state_domain),
                                    expected_artifact_hash: Some(
                                        reference.witness_artifact_hash.digest,
                                    ),
                                    ..PublishedArtifactReadOptions::default()
                                },
                            )
                            .unwrap();
                            let staged_state = product
                                .0
                                .2
                                .stage_source_algorithm_state_file(&state_read, reference)
                                .unwrap();
                            assert!(
                                product
                                    .0
                                    .0
                                    .clone()
                                    .with_source_theorem_witness(&staged_state)
                                    .is_err()
                            );
                            assert!(
                                mizar_proof::witness_store::publish_ref(&staged_state, &token)
                                    .is_err()
                            );
                        },
                    );
                    let stale = mizar_proof::witness_store::CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                        &commit,
                        &transport_root,
                        &module,
                        hash_text("stale expected snapshot"),
                    )
                    .unwrap();
                    assert_eq!(
                        mizar_proof::witness_store::publish_ref(&file_staged, &stale),
                        Err(mizar_proof::witness_store::ProofWitnessStoreError::StaleBuildSnapshot)
                    );
                    let witness_file = transport_root.join(&actual.2.witness_path);
                    let original_bytes = fs::read(&witness_file).unwrap();
                    fs::write(&witness_file, [original_bytes.as_slice(), b" "].concat()).unwrap();
                    assert!(mizar_proof::witness_store::publish_ref(&file_staged, &token).is_ok());
                    assert!(mizar_proof::witness_store::CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                        &commit,
                        &transport_root,
                        &module,
                        file_staged.provenance().build_snapshot_fingerprint(),
                    )
                    .is_err());
                    fs::write(&witness_file, original_bytes).unwrap();
                    assert!(
                        mizar_proof::witness_store::CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                            &commit,
                            &transport_root,
                            &module,
                            file_staged.provenance().build_snapshot_fingerprint(),
                        )
                        .is_ok()
                    );

                    let mut no_witness_artifact = artifact.clone();
                    no_witness_artifact.proof_witnesses.clear();
                    no_witness_artifact.obligations[0].status = ObligationStatus::Open;
                    no_witness_artifact.obligations[0].accepted_witness_obligation_id = None;
                    let (no_witness_interface, no_witness_implementation) = no_witness_artifact
                        .refresh_hashes()
                        .map_err(|error| error.to_string())?;
                    let no_witness_json = builtin_verified_artifact_json(&no_witness_artifact)
                        .map_err(|error| error.to_string())?;
                    let no_witness_write = write_published_artifact(
                        &transport_root,
                        &PublishedArtifactPath::new(main_path).unwrap(),
                        &no_witness_json,
                        &main_domain,
                        &artifact_hash_excluded_paths(),
                    )
                    .unwrap();
                    let mut no_witness_entry = module_entry.clone();
                    no_witness_entry.artifact_hash.digest = no_witness_write.artifact_hash;
                    no_witness_entry.interface_hash.digest = no_witness_interface;
                    no_witness_entry.implementation_hash.digest = no_witness_implementation;
                    no_witness_entry.proof_witnesses.clear();
                    let mut transaction = ManifestTransaction::begin(
                        &transport_root,
                        commit.manifest().clone(),
                        None,
                    )
                    .unwrap();
                    transaction.stage_module(no_witness_entry).unwrap();
                    let no_witness_commit = transaction
                        .commit(ManifestCommitOptions::default())
                        .unwrap();
                    assert!(mizar_proof::witness_store::CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                        &no_witness_commit,
                        &transport_root,
                        &module,
                        file_staged.provenance().build_snapshot_fingerprint(),
                    )
                    .is_err());
                }
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
