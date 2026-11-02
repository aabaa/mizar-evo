use super::*;

#[test]
fn committed_source_theorem_witness_matches_genuine_staged_candidate() {
    use mizar_artifact::{
        manifest::{
            ManifestCommitOptions, ManifestFileReadOptions, ManifestProofWitnessEntry,
            ManifestTransaction, artifact_manifest_path, read_manifest_file,
        },
        proof_witness::ProofStatus,
        store::{
            PublishedArtifactPath, PublishedArtifactReadOptions, artifact_hash_domain,
            canonical_json_bytes, read_published_artifact, write_published_artifact,
        },
        verified_artifact::{
            ObligationStatus, VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifactReadOptions,
            artifact_hash_excluded_paths, read_verified_artifact, verified_artifact_json,
        },
    };
    use mizar_atp::problem::*;
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker, check_source_theorem_metadata,
    };
    use mizar_proof::{
        policy::VerifierPolicy,
        selection::ProofWinnerClass,
        status::{
            ArtifactPublicationGap, ArtifactStatusPublication, ProjectedProofStatus,
            source_theorem_candidate_witness_file_candidate,
        },
        witness_store::{
            CommittedWitnessPublicationProof, ProofWitnessStoreError, publish_ref,
            read_source_theorem_witness_file,
        },
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::{source_proof_inputs, source_theorem_module_original_inputs},
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcSchemaVersion, VcSet},
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
            |fixture, inputs, six, owner, refs, expected, _| {
                let frontend = refs
                    .iter()
                    .find(|row| row.output_kind() == &OutputKind::new("FrontendOutput"))
                    .unwrap();
                let frontend = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                frontend,
                                frontend.output_kind(),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                let ast = frontend.ast.as_ref().unwrap();
                assert_eq!(frontend.source.source_text.as_ref(), text);
                assert_eq!(ast.source_id, inputs.snapshot.source_versions[0].source_id);
                let module =
                    ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                let (arena, env) = source_theorem_module_original_inputs(ast, &module)
                    .unwrap()
                    .unwrap();
                let variables = SourceVariableScopeResolver::resolve_proof_occurrences(
                    SourceVariableScopeInput::new(ast, &module, &env),
                )
                .unwrap();
                let metadata = check_source_theorem_metadata(&arena, &env, &variables).unwrap();
                let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
                let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &arena,
                    &metadata.2,
                    &scope,
                    &env,
                    &labels,
                    &resolved,
                    None,
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
                let witness_path = "proof-witnesses/checked.json";
                let ((status, _, draft), file_value, reference) =
                    source_theorem_candidate_witness_file_candidate(
                        &arena,
                        &check,
                        &env,
                        inputs.snapshot,
                        vcs.vcs()[0].id,
                        (
                            &GenerationSchemaVersion::new("λ\n\0"),
                            &VcSchemaVersion::new(""),
                            profile.clone(),
                            ConcreteFormat::Tptp,
                        ),
                        (&policy, &verifier),
                        witness_path,
                    )
                    .unwrap();
                assert_eq!(status.projected_status(), ProjectedProofStatus::Accepted);
                assert_eq!(draft.selected_class(), ProofWinnerClass::KernelVerified);
                assert_eq!(reference.proof_status, ProofStatus::KernelVerified);
                assert_eq!(
                    status.artifact_publication(),
                    ArtifactStatusPublication::ExternalDependencyGap(
                        ArtifactPublicationGap::MissingKernelVerifiedWitness
                    )
                );
                let payload = draft.payload_bytes().to_vec();
                let staged_root = fixture.root.join("staged-witness");
                let file_domain = artifact_hash_domain(
                    reference.witness_artifact_hash.schema_family.clone(),
                    reference.witness_artifact_hash.schema_version,
                );
                fs::create_dir_all(&staged_root).unwrap();
                let written = write_published_artifact(
                    &staged_root,
                    &PublishedArtifactPath::new(witness_path).unwrap(),
                    &file_value,
                    &file_domain,
                    &[],
                )
                .unwrap();
                assert_eq!(
                    written.artifact_hash,
                    reference.witness_artifact_hash.digest
                );
                let file_bytes = fs::read(staged_root.join(witness_path)).unwrap();
                assert_eq!(file_bytes, canonical_json_bytes(&file_value));
                let file = read_published_artifact(
                    &staged_root,
                    &PublishedArtifactPath::new(witness_path).unwrap(),
                    PublishedArtifactReadOptions {
                        artifact_hash_domain: Some(&file_domain),
                        expected_artifact_hash: Some(reference.witness_artifact_hash.digest),
                        ..PublishedArtifactReadOptions::default()
                    },
                )
                .unwrap();
                assert_eq!(
                    read_source_theorem_witness_file(&file, &reference).unwrap(),
                    payload
                );
                let staged = draft.stage_source_theorem_file(&file, &reference).unwrap();
                assert_eq!(staged.artifact_reference_candidate(), Some(&reference));
                assert_eq!(staged.selected_class(), ProofWinnerClass::KernelVerified);
                assert_eq!(
                    staged.staged_payload_hash(),
                    &reference.witness_artifact_hash
                );
                assert_eq!(
                    staged.provenance().build_snapshot_fingerprint(),
                    inputs.snapshot.id.fingerprint()
                );
                let projected_root = fixture.root.join("alpha/projected-witness");
                let (projected, projected_stage) = inputs
                    .project_native_theorem_artifact_draft(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (witness_path, &projected_root),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(&projected, expected.as_ref().unwrap().as_ref().unwrap());
                assert_eq!(projected_stage, staged);
                let written_root = fixture.root.join("alpha/written-witness");
                let (written_draft, _, written_stage) = inputs
                    .write_native_theorem_artifact_draft(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (witness_path, "main.mizir.json", &written_root),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(written_draft, projected);
                assert_eq!(written_stage, staged);
                let root = fixture.root.join("alpha/committed-witness");
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("ArtifactCommit"),
                    OutputKind::new("SourceTheoremArtifactData"),
                    IrWorkUnit::new("\"alpha\":\"main\""),
                ));
                let (artifact, main_write, commit, actual_published, output) = inputs
                    .commit_native_theorem_artifact_manifest(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (witness_path, "main.mizir.json", &root),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(&artifact, expected.as_ref().unwrap().as_ref().unwrap());
                assert_eq!(
                    artifact.artifact.proof_witnesses.as_slice(),
                    std::slice::from_ref(&reference)
                );
                assert_eq!(
                    artifact.artifact.obligations[0].status,
                    ObligationStatus::Accepted
                );
                assert_eq!(
                    artifact.artifact.obligations[0]
                        .accepted_witness_obligation_id
                        .as_deref(),
                    Some(reference.obligation_id.as_str())
                );
                assert_eq!(
                    artifact.artifact.source_hash,
                    inputs.snapshot.source_versions[0].source_hash
                );
                assert_eq!(
                    artifact.artifact.exports.is_empty(),
                    text.starts_with("private ")
                );
                let main_domain = artifact_hash_domain(
                    VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                    artifact.artifact.schema_version,
                );
                let excluded = artifact_hash_excluded_paths();
                let main = read_published_artifact(
                    &root,
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
                        &main.value,
                        VerifiedArtifactReadOptions {
                            artifact_path: Some(main_write.path.as_str()),
                            expected_module: Some(&artifact.artifact.module),
                            expected_interface_hash: Some(artifact.artifact.interface_hash),
                            expected_implementation_hash: Some(
                                artifact.artifact.implementation_hash
                            ),
                        },
                    )
                    .unwrap(),
                    artifact.artifact
                );
                assert_eq!(
                    fs::read(root.join(main_write.path.as_str())).unwrap(),
                    canonical_json_bytes(&verified_artifact_json(&artifact.artifact).unwrap())
                );
                let manifest = read_manifest_file(
                    &root,
                    ManifestFileReadOptions {
                        validate_references: true,
                        validate_witness_files: true,
                        validate_development_artifacts: true,
                        expected_manifest_hash: Some(commit.write().artifact_hash),
                    },
                )
                .unwrap();
                assert_eq!(&manifest.manifest, commit.manifest());
                let [entry] = manifest.manifest.modules.as_slice() else {
                    panic!("one committed module")
                };
                assert_eq!(entry.module, artifact.artifact.module);
                assert_eq!(entry.artifact_hash.digest, main_write.artifact_hash);
                assert_eq!(
                    entry.proof_witnesses,
                    [ManifestProofWitnessEntry {
                        obligation_id: reference.obligation_id.clone(),
                        obligation_fingerprint: reference.obligation_fingerprint.clone(),
                        witness_path: reference.witness_path.clone(),
                        witness_artifact_hash: reference.witness_artifact_hash.clone(),
                    }]
                );
                let expected_snapshot = inputs.snapshot.id.fingerprint();
                let proof = CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                    &commit,
                    &root,
                    &artifact.artifact.module,
                    expected_snapshot,
                )
                .unwrap();
                assert_eq!(proof.module_entry(), entry);
                assert_eq!(proof.committed_main_artifact_hash(), &entry.artifact_hash);
                assert_eq!(
                    proof.verified_artifact_witnesses(),
                    artifact.artifact.proof_witnesses
                );
                assert_eq!(proof.manifest_witnesses(), entry.proof_witnesses);
                assert_eq!(proof.build_snapshot_fingerprint(), expected_snapshot);
                let published = publish_ref(&staged, &proof).unwrap();
                assert_eq!(actual_published, published);
                assert_eq!(published.reference(), &reference);
                assert_eq!(published.provenance(), staged.provenance());
                assert_eq!(
                    published.committed_main_artifact_hash(),
                    &entry.artifact_hash
                );
                let stale = CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                    &commit,
                    &root,
                    &artifact.artifact.module,
                    Hash::from_bytes([0x43; 32]),
                )
                .unwrap();
                assert_eq!(
                    publish_ref(&staged, &stale),
                    Err(ProofWitnessStoreError::StaleBuildSnapshot)
                );
                for field in [
                    "package_id",
                    "package_version",
                    "lockfile_identity",
                    "module_path",
                    "language_edition",
                ] {
                    let mut wrong_module = artifact.artifact.module.clone();
                    match field {
                        "package_id" => wrong_module.package_id.push_str("-other"),
                        "package_version" => wrong_module.package_version = Some("99.0.0".into()),
                        "lockfile_identity" => {
                            wrong_module.lockfile_identity = Some("other-lock".into())
                        }
                        "module_path" => wrong_module.module_path.push_str(".other"),
                        "language_edition" => wrong_module.language_edition = "9999".into(),
                        _ => unreachable!(),
                    }
                    assert!(
                        CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                            &commit,
                            &root,
                            &wrong_module,
                            expected_snapshot,
                        )
                        .is_err(),
                        "{field}"
                    );
                }
                assert!(
                    CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                        &commit,
                        &fixture.root.join("absent-root"),
                        &artifact.artifact.module,
                        expected_snapshot,
                    )
                    .is_err()
                );
                let other_root = fixture.root.join("alpha/other-committed-witness");
                let (other_artifact, other_main, _) = inputs
                    .write_native_theorem_artifact_draft(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        ("proof-witnesses/other.json", "main.mizir.json", &other_root),
                    )
                    .unwrap()
                    .unwrap();
                let mut other_entry = entry.clone();
                other_entry.artifact_hash.digest = other_main.artifact_hash;
                other_entry.interface_hash.digest = other_artifact.artifact.interface_hash;
                other_entry.implementation_hash.digest =
                    other_artifact.artifact.implementation_hash;
                other_entry.proof_witnesses = other_artifact
                    .artifact
                    .proof_witnesses
                    .iter()
                    .map(|witness| ManifestProofWitnessEntry {
                        obligation_id: witness.obligation_id.clone(),
                        obligation_fingerprint: witness.obligation_fingerprint.clone(),
                        witness_path: witness.witness_path.clone(),
                        witness_artifact_hash: witness.witness_artifact_hash.clone(),
                    })
                    .collect();
                let mut other_seed = commit.manifest().clone();
                other_seed.artifact_root = "other-committed-witness".into();
                let mut other_transaction =
                    ManifestTransaction::begin(&other_root, other_seed, None).unwrap();
                other_transaction.stage_module(other_entry).unwrap();
                let other_commit = other_transaction
                    .commit(ManifestCommitOptions::default())
                    .unwrap();
                for (wrong_commit, wrong_root) in [(&commit, &other_root), (&other_commit, &root)] {
                    assert!(
                        CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                            wrong_commit,
                            wrong_root,
                            &artifact.artifact.module,
                            expected_snapshot,
                        )
                        .is_err()
                    );
                }
                let other_proof =
                    CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                        &other_commit,
                        &other_root,
                        &other_artifact.artifact.module,
                        expected_snapshot,
                    )
                    .unwrap();
                assert_eq!(
                    publish_ref(&staged, &other_proof),
                    Err(ProofWitnessStoreError::MissingCommittedWitnessReference {
                        obligation_id: reference.obligation_id.clone(),
                    })
                );
                let copied_root = fixture.root.join("identical-copy");
                fs::create_dir_all(copied_root.join("proof-witnesses")).unwrap();
                for path in [
                    artifact_manifest_path().as_str(),
                    main_write.path.as_str(),
                    witness_path,
                ] {
                    fs::copy(root.join(path), copied_root.join(path)).unwrap();
                }
                let copied_proof =
                    CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                        &commit,
                        &copied_root,
                        &artifact.artifact.module,
                        expected_snapshot,
                    )
                    .unwrap();
                assert_eq!(copied_proof, proof);
                assert_eq!(publish_ref(&staged, &copied_proof).unwrap(), published);
                let main_bytes = fs::read(root.join(main_write.path.as_str())).unwrap();
                let mut diagnostic_change = artifact.artifact.clone();
                diagnostic_change.verified_at = Some("2030-01-01T00:00:00Z".into());
                diagnostic_change.provenance.cache_key =
                    Some("different diagnostics-only cache key".into());
                let changed_main = verified_artifact_json(&diagnostic_change).unwrap();
                assert_eq!(
                    main_domain.hash(&changed_main, &excluded),
                    main_write.artifact_hash
                );
                assert_eq!(
                    main_domain.hash_input(&changed_main, &excluded),
                    main_domain.hash_input(
                        &verified_artifact_json(&artifact.artifact).unwrap(),
                        &excluded
                    )
                );
                assert_ne!(canonical_json_bytes(&changed_main), main_bytes);
                fs::write(
                    root.join(main_write.path.as_str()),
                    canonical_json_bytes(&changed_main),
                )
                .unwrap();
                let excluded_proof =
                    CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                        &commit,
                        &root,
                        &artifact.artifact.module,
                        expected_snapshot,
                    )
                    .unwrap();
                assert_eq!(excluded_proof, proof);
                assert_eq!(publish_ref(&staged, &excluded_proof).unwrap(), published);
                fs::write(root.join(main_write.path.as_str()), &main_bytes).unwrap();
                for case in ["missing-coverage", "extra-coverage", "wrong-witness-hash"] {
                    let mut wrong_entry = entry.clone();
                    match case {
                        "missing-coverage" => wrong_entry.proof_witnesses.clear(),
                        "extra-coverage" => {
                            let mut extra = wrong_entry.proof_witnesses[0].clone();
                            extra.obligation_id.push_str("-extra");
                            extra.witness_path = "proof-witnesses/extra.json".into();
                            wrong_entry.proof_witnesses.push(extra);
                            wrong_entry
                                .proof_witnesses
                                .sort_by(|a, b| a.obligation_id.cmp(&b.obligation_id));
                        }
                        "wrong-witness-hash" => {
                            wrong_entry.proof_witnesses[0].witness_artifact_hash.digest =
                                Hash::from_bytes([0x44; 32])
                        }
                        _ => unreachable!(),
                    }
                    let mut transaction =
                        ManifestTransaction::begin(&root, commit.manifest().clone(), None).unwrap();
                    assert!(
                        transaction
                            .stage_module(wrong_entry)
                            .and_then(|()| transaction.commit(ManifestCommitOptions::default()))
                            .is_err(),
                        "{case}"
                    );
                    assert_eq!(
                        CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                            &commit,
                            &root,
                            &artifact.artifact.module,
                            expected_snapshot,
                        )
                        .unwrap(),
                        proof
                    );
                }
                for case in ["unsupported-payload-profile", "internal-reference-mismatch"] {
                    let challenge_root = fixture.root.join(case);
                    fs::create_dir_all(challenge_root.join("proof-witnesses")).unwrap();
                    for path in [
                        artifact_manifest_path().as_str(),
                        main_write.path.as_str(),
                        witness_path,
                    ] {
                        fs::copy(root.join(path), challenge_root.join(path)).unwrap();
                    }
                    let mut challenge_file = file_value.clone();
                    let CanonicalJson::Object(fields) = &mut challenge_file else {
                        panic!("genuine FILE object")
                    };
                    let (field, value) = if case == "unsupported-payload-profile" {
                        (
                            "payload_schema_family",
                            "mizar-proof/unsupported-checked-wire".to_owned(),
                        )
                    } else {
                        (
                            "obligation_id",
                            format!("{}-mismatch", reference.obligation_id),
                        )
                    };
                    fields.insert(field.into(), CanonicalJson::string(value));
                    let challenge_write = write_published_artifact(
                        &challenge_root,
                        &PublishedArtifactPath::new(witness_path).unwrap(),
                        &challenge_file,
                        &file_domain,
                        &[],
                    )
                    .unwrap();
                    let mut challenge_main = artifact.artifact.clone();
                    challenge_main.proof_witnesses[0]
                        .witness_artifact_hash
                        .digest = challenge_write.artifact_hash;
                    challenge_main.refresh_hashes().unwrap();
                    let challenge_main_write = write_published_artifact(
                        &challenge_root,
                        &main_write.path,
                        &verified_artifact_json(&challenge_main).unwrap(),
                        &main_domain,
                        &excluded,
                    )
                    .unwrap();
                    let mut challenge_entry = entry.clone();
                    challenge_entry.proof_witnesses[0].witness_artifact_hash = challenge_main
                        .proof_witnesses[0]
                        .witness_artifact_hash
                        .clone();
                    challenge_entry.artifact_hash.digest = challenge_main_write.artifact_hash;
                    challenge_entry.interface_hash.digest = challenge_main.interface_hash;
                    challenge_entry.implementation_hash.digest = challenge_main.implementation_hash;
                    let mut transaction = ManifestTransaction::begin(
                        &challenge_root,
                        commit.manifest().clone(),
                        None,
                    )
                    .unwrap();
                    transaction.stage_module(challenge_entry).unwrap();
                    let challenge_commit = transaction
                        .commit(ManifestCommitOptions::default())
                        .unwrap();
                    let challenge_manifest = read_manifest_file(
                        &challenge_root,
                        ManifestFileReadOptions {
                            validate_references: true,
                            validate_witness_files: true,
                            validate_development_artifacts: true,
                            expected_manifest_hash: Some(challenge_commit.write().artifact_hash),
                        },
                    )
                    .unwrap();
                    assert_eq!(&challenge_manifest.manifest, challenge_commit.manifest());
                    let challenge_read = read_published_artifact(
                        &challenge_root,
                        &PublishedArtifactPath::new(witness_path).unwrap(),
                        PublishedArtifactReadOptions {
                            artifact_hash_domain: Some(&file_domain),
                            expected_artifact_hash: Some(challenge_write.artifact_hash),
                            ..PublishedArtifactReadOptions::default()
                        },
                    )
                    .unwrap();
                    assert!(
                        read_source_theorem_witness_file(
                            &challenge_read,
                            &challenge_main.proof_witnesses[0],
                        )
                        .is_err(),
                        "{case}"
                    );
                    assert!(
                        CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                            &challenge_commit,
                            &challenge_root,
                            &artifact.artifact.module,
                            expected_snapshot,
                        )
                        .is_err(),
                        "{case}"
                    );
                }
                let manifest_path = artifact_manifest_path();
                for (path, original) in [
                    (
                        manifest_path.as_str(),
                        fs::read(root.join(manifest_path.as_str())).unwrap(),
                    ),
                    (main_write.path.as_str(), main_bytes.clone()),
                    (witness_path, file_bytes.clone()),
                ] {
                    for missing in [false, true] {
                        if missing {
                            fs::remove_file(root.join(path)).unwrap();
                        } else {
                            fs::write(root.join(path), b"null").unwrap();
                        }
                        if path == witness_path && !missing {
                            // Ordinary manifest validation proves reachability; concrete FILE integrity remains separate.
                            assert!(
                                read_manifest_file(
                                    &root,
                                    ManifestFileReadOptions {
                                        validate_references: true,
                                        validate_witness_files: true,
                                        validate_development_artifacts: true,
                                        expected_manifest_hash: Some(commit.write().artifact_hash),
                                    }
                                )
                                .is_ok()
                            );
                        }
                        assert!(
                            CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                                &commit,
                                &root,
                                &artifact.artifact.module,
                                expected_snapshot,
                            )
                            .is_err(),
                            "{path}, missing={missing}"
                        );
                        assert_eq!(publish_ref(&staged, &proof).unwrap(), published);
                        assert_eq!(*owner.storage().get(&output).unwrap(), artifact.artifact);
                        fs::write(root.join(path), &original).unwrap();
                        assert_eq!(
                            CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                                &commit,
                                &root,
                                &artifact.artifact.module,
                                expected_snapshot,
                            )
                            .unwrap(),
                            proof
                        );
                    }
                }
                assert_eq!(fs::read(root.join(witness_path)).unwrap(), file_bytes);
                assert_eq!(
                    fs::read(fixture.root.join("alpha/src/main.miz")).unwrap(),
                    text.as_bytes()
                );
                owner.mark_obsolete(six.snapshot).unwrap();
                fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                assert!(
                    owner
                        .validate_current_output(six.snapshot, output.any())
                        .is_err()
                );
                assert_eq!(*owner.storage().get(&output).unwrap(), artifact.artifact);
            },
        );
    }
}
