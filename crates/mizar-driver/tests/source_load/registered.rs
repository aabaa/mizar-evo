use super::*;

#[test]
fn registered_source_theorem_artifact_commit_matches_real_owner_products() {
    use mizar_artifact::{
        manifest::{
            ArtifactManifest, ManifestCommitOptions, ManifestProofWitnessEntry, ManifestProvenance,
            ManifestTransaction, ModuleArtifactEntry, PackageIdentity, artifact_manifest_json,
            current_schema_version as manifest_schema, manifest_hash_domain,
        },
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{
            PublishedArtifactReadOptions, artifact_hash_domain, canonical_json_bytes,
            read_published_artifact,
        },
        verified_artifact::{
            VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifact, VerifiedArtifactReadOptions,
            artifact_hash_excluded_paths, read_verified_artifact, verified_artifact_json,
        },
    };
    use mizar_atp::problem::*;
    use mizar_driver::registry::PhaseOwner;
    use mizar_ir::{
        projection::ProjectionExternalDependencyGap as Gap,
        storage::{CollectInput, IrSideTables, SchemaVersion},
    };
    use mizar_proof::{
        policy::VerifierPolicy,
        witness_store::{CommittedWitnessPublicationProof, publish_ref},
    };
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
            |fixture, inputs, six, owner, refs, _, (inference, typed, metadata, source)| {
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
                let kernel = refs
                    .iter()
                    .find(|reference| {
                        reference.output_kind() == &OutputKind::new("SourceTheoremCandidateCheck")
                    })
                    .unwrap();
                let mut original_outputs = refs
                    .iter()
                    .map(|reference| owner.registry().output_lineage(reference.output()).unwrap())
                    .collect::<Vec<_>>();
                for parent in &kernel.lineage().parents {
                    if original_outputs
                        .iter()
                        .all(|output| output.output != *parent)
                    {
                        original_outputs.push(owner.registry().output_lineage(*parent).unwrap());
                    }
                }
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
                let vc = refs
                    .iter()
                    .find(|reference| {
                        reference.output_kind() == &OutputKind::new("SourceTheoremVC")
                    })
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
                let prefix = "proof-witnesses/service-λ";
                let kernel_hex = kernel
                    .content_hash()
                    .as_bytes()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                let witness_path = format!("{prefix}/{kernel_hex}.json");
                let baseline_root = fixture.root.join("alpha/service-artifact-baseline");
                let (draft, main_write, staged) = inputs
                    .write_native_theorem_artifact_draft(
                        six,
                        owner,
                        vcs.vcs()[0].id,
                        (profile.clone(), ConcreteFormat::Tptp),
                        (&policy, &verifier),
                        (&witness_path, "main.mizir.json", &baseline_root),
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    draft.external_dependency_gaps,
                    [
                        Gap::DriverSession,
                        Gap::DiagnosticsIntegration,
                        Gap::ProducerProjectionPayload,
                        Gap::ArtifactPublicationToken
                    ]
                );
                let artifact = &draft.artifact;
                let [witness] = artifact.proof_witnesses.as_slice() else {
                    panic!("one genuine witness")
                };
                assert_eq!(witness.witness_path, witness_path);
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
                    proof_witnesses: vec![ManifestProofWitnessEntry {
                        obligation_id: witness.obligation_id.clone(),
                        obligation_fingerprint: witness.obligation_fingerprint.clone(),
                        witness_path: witness.witness_path.clone(),
                        witness_artifact_hash: witness.witness_artifact_hash.clone(),
                    }],
                    diagnostics_hash: None,
                };
                let seed = ArtifactManifest {
                    schema_version: manifest_schema(),
                    package: PackageIdentity {
                        package_id: artifact.module.package_id.clone(),
                        package_version: artifact.module.package_version.clone(),
                        lockfile_identity: artifact.module.lockfile_identity.clone(),
                    },
                    artifact_root: "service-artifact-baseline".into(),
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
                let mut transaction =
                    ManifestTransaction::begin(&baseline_root, seed, None).unwrap();
                transaction.stage_module(entry).unwrap();
                let baseline_commit = transaction
                    .commit(ManifestCommitOptions::default())
                    .unwrap();
                let baseline_proof =
                    CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                        &baseline_commit,
                        &baseline_root,
                        &artifact.module,
                        six.snapshot.fingerprint(),
                    )
                    .unwrap();
                let baseline_published = publish_ref(&staged, &baseline_proof).unwrap();
                assert_eq!(baseline_published.reference(), witness);
                let main_bytes = fs::read(baseline_root.join(main_write.path.as_str())).unwrap();
                let file_bytes = fs::read(baseline_root.join(&witness_path)).unwrap();
                let value = verified_artifact_json(artifact).unwrap();
                let domain =
                    artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version);
                let excluded = artifact_hash_excluded_paths();
                assert_eq!(main_bytes, canonical_json_bytes(&value));
                assert_eq!(main_write.artifact_hash, domain.hash(&value, &excluded));
                let parent = "service-artifact-actual";
                let parent_root = fixture.root.join("alpha").join(parent);
                fs::create_dir(&parent_root).unwrap();
                let snapshot_hex = six
                    .snapshot
                    .fingerprint()
                    .as_bytes()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                let source_key_hex = six
                    .identities()
                    .input_hash()
                    .as_bytes()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                assert_eq!(
                    six.identities().input_hash(),
                    source_key(&inputs.snapshot.source_versions[0])
                );
                let child = format!("{snapshot_hex}-{source_key_hex}");
                let root = parent_root.join(&child);
                assert!(!root.exists());
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("ArtifactCommit"),
                    OutputKind::new("SourceTheoremArtifactData"),
                    kernel.work_unit().clone(),
                ));
                let mut builder = PhaseRegistryBuilder::new();
                builder
                    .register_source_theorem_artifact_commit(
                        policy.clone(),
                        profile.clone(),
                        ConcreteFormat::Tptp,
                        verifier.clone(),
                        (prefix.into(), "main.mizir.json".into(), parent.into()),
                    )
                    .unwrap();
                let registry = builder.build().unwrap();
                let descriptor = registry
                    .descriptor_for_phase(PipelinePhase::ArtifactCommit)
                    .unwrap();
                assert_eq!(descriptor.service_name, "SourceTheoremArtifactCommit");
                assert_eq!(descriptor.owner, PhaseOwner::MizarArtifact);
                assert_eq!(descriptor.phases, [PipelinePhase::ArtifactCommit]);
                assert_eq!(descriptor.output_kind, "SourceTheoremArtifactData");
                let cache = registry
                    .cache_key_for_phase(PipelinePhase::ArtifactCommit, six)
                    .unwrap();
                assert!(
                    matches!(&cache.intent, PhaseCacheIntent::NoKey { reason } if !reason.is_empty())
                );
                for variant in [
                    "parent", "prefix", "main", "policy", "verifier", "format", "profile",
                ] {
                    let mut configured_policy = policy.clone();
                    if variant == "policy" {
                        configured_policy.computation_limit.max_steps += 1;
                    }
                    let configured_verifier = if variant == "verifier" {
                        verifier
                            .clone()
                            .with_profile_id("other configured verifier")
                    } else {
                        verifier.clone()
                    };
                    let configured_profile = LogicProfile::try_new(
                        if variant == "profile" {
                            "other-artifact-source"
                        } else {
                            profile.name().as_str()
                        },
                        profile.fragment(),
                        profile.equality(),
                        profile.quantifiers(),
                        profile.soft_types(),
                        profile.native_properties(),
                        profile.concrete_formats().clone(),
                    )
                    .unwrap();
                    let mut staging = (prefix.into(), "main.mizir.json".into(), parent.into());
                    match variant {
                        "prefix" => staging.0 = "proof-witnesses/other".into(),
                        "main" => staging.1 = "other.mizir.json".into(),
                        "parent" => staging.2 = "another-artifact-parent".into(),
                        _ => {}
                    }
                    let mut configured = PhaseRegistryBuilder::new();
                    configured
                        .register_source_theorem_artifact_commit(
                            configured_policy,
                            configured_profile,
                            if variant == "format" {
                                ConcreteFormat::SmtLib
                            } else {
                                ConcreteFormat::Tptp
                            },
                            configured_verifier,
                            staging,
                        )
                        .unwrap();
                    let configured = configured.build().unwrap();
                    assert_eq!(
                        configured
                            .descriptor_for_phase(PipelinePhase::ArtifactCommit)
                            .unwrap()
                            == descriptor,
                        variant == "parent",
                        "{variant}"
                    );
                    let configured_cache = configured
                        .cache_key_for_phase(PipelinePhase::ArtifactCommit, six)
                        .unwrap();
                    assert_eq!(configured_cache.intent, cache.intent);
                    assert_eq!(
                        configured_cache.observation == cache.observation,
                        variant == "parent",
                        "{variant}"
                    );
                }
                for staging in [
                    ("other", "main.mizir.json", parent),
                    (prefix, "bad.json", parent),
                    (prefix, "main.mizir.json", "../outside"),
                ] {
                    let mut invalid = PhaseRegistryBuilder::new();
                    assert!(
                        invalid
                            .register_source_theorem_artifact_commit(
                                policy.clone(),
                                profile.clone(),
                                ConcreteFormat::Tptp,
                                verifier.clone(),
                                (staging.0.into(), staging.1.into(), staging.2.into()),
                            )
                            .is_err()
                    );
                    assert!(
                        invalid
                            .build()
                            .unwrap()
                            .descriptor_for_phase(PipelinePhase::ArtifactCommit)
                            .is_err()
                    );
                }
                let execution = registry
                    .execute_phase_with_resources(
                        PipelinePhase::ArtifactCommit,
                        six.clone(),
                        PhaseExecutionResources {
                            output_publisher: Some(owner.clone()),
                            source_load: Some(inputs),
                            ..PhaseExecutionResources::default()
                        },
                    )
                    .unwrap();
                let result = execution.result;
                assert_eq!(result.status, PhaseStatus::Complete);
                assert!(result.diagnostics.is_empty());
                assert_eq!(result.cache_observation, Some(execution.observation));
                let [output] = result.output_refs.as_slice() else {
                    panic!("one actual Artifact DATA output")
                };
                assert_eq!(output.snapshot(), six.snapshot);
                assert_eq!(output.phase(), &IrPipelinePhase::new("ArtifactCommit"));
                assert_eq!(output.work_unit(), kernel.work_unit());
                assert_eq!(
                    output.output_kind(),
                    &OutputKind::new("SourceTheoremArtifactData")
                );
                assert_eq!(output.schema_version(), SchemaVersion::new(1));
                let handle = owner
                    .storage()
                    .typed_handle::<VerifiedArtifact>(output, output.output_kind())
                    .unwrap();
                assert_eq!(*owner.storage().get(&handle).unwrap(), *artifact);
                assert_eq!(
                    *owner.storage().side_tables(&handle).unwrap(),
                    IrSideTables::default()
                );
                let mut parent_ids = six
                    .parent_outputs()
                    .iter()
                    .map(SealedParentOutputHandle::output)
                    .collect::<Vec<_>>();
                parent_ids
                    .sort_by(|left, right| left.hash().as_bytes().cmp(right.hash().as_bytes()));
                assert_eq!(output.lineage().parents, parent_ids);
                assert_eq!(
                    output.lineage().named_input_hashes,
                    kernel.lineage().named_input_hashes
                );
                owner.validate_current_output(six.snapshot, output).unwrap();
                match *output.placement() {
                    StoragePlacement::Resident => assert_eq!(threshold, usize::MAX),
                    StoragePlacement::Blob { len, .. } => {
                        assert_eq!(threshold, 0);
                        assert_eq!(len, main_bytes.len());
                    }
                    _ => unreachable!(),
                }
                assert_eq!(fs::read_dir(&parent_root).unwrap().count(), 1);
                assert_eq!(
                    fs::read(root.join(main_write.path.as_str())).unwrap(),
                    main_bytes
                );
                assert_eq!(fs::read(root.join(&witness_path)).unwrap(), file_bytes);
                let normal = read_published_artifact(
                    &root,
                    &main_write.path,
                    PublishedArtifactReadOptions {
                        artifact_hash_domain: Some(&domain),
                        hash_excluded_paths: &excluded,
                        expected_artifact_hash: Some(main_write.artifact_hash),
                    },
                )
                .unwrap();
                assert_eq!(
                    read_verified_artifact(
                        &normal.value,
                        VerifiedArtifactReadOptions {
                            artifact_path: Some(main_write.path.as_str()),
                            expected_module: Some(&artifact.module),
                            expected_interface_hash: Some(artifact.interface_hash),
                            expected_implementation_hash: Some(artifact.implementation_hash),
                        }
                    )
                    .unwrap(),
                    *artifact
                );
                let mut expected_manifest = baseline_commit.manifest().clone();
                expected_manifest.artifact_root = format!("{parent}/{child}");
                let manifest_value = artifact_manifest_json(&expected_manifest).unwrap();
                let manifest_hash = manifest_hash_domain(expected_manifest.schema_version)
                    .hash(&manifest_value, &[]);
                assert_ne!(manifest_hash, baseline_commit.write().artifact_hash);
                assert_eq!(
                    fs::read(root.join("artifact-manifest.json")).unwrap(),
                    canonical_json_bytes(&manifest_value)
                );
                let actual_commit = result
                    .artifact_commit
                    .as_ref()
                    .expect("same-call owner commit");
                assert_eq!(actual_commit.manifest(), &expected_manifest);
                assert_eq!(
                    actual_commit.write().path.as_str(),
                    "artifact-manifest.json"
                );
                assert_eq!(actual_commit.write().artifact_hash, manifest_hash);
                let actual_proof =
                    CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                        actual_commit,
                        &root,
                        &artifact.module,
                        six.snapshot.fingerprint(),
                    )
                    .unwrap();
                assert_eq!(
                    publish_ref(&staged, &actual_proof).unwrap(),
                    baseline_published
                );
                for original in &original_outputs {
                    assert_eq!(
                        owner.registry().output_lineage(original.output).as_ref(),
                        Some(original)
                    );
                    assert_ne!(original.output, output.output());
                }
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
                    (15, 0)
                );
            },
        );
    }
}

#[test]
fn registered_source_theorem_artifact_commit_preserves_refusals() {
    use mizar_artifact::{
        manifest::{
            ManifestCommitOptions, ManifestFileReadOptions, ManifestTransaction, read_manifest_file,
        },
        verified_artifact::VerifiedArtifact,
    };
    use mizar_atp::problem::*;
    use mizar_ir::storage::CollectInput;
    use mizar_proof::{policy::VerifierPolicy, witness_store::CommittedWitnessPublicationProof};
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::PolicyKey,
    };
    for case in [
        "denied",
        "missing-parent",
        "escape",
        "package-root",
        "cancel",
        "foreign-token",
        "no-publisher",
        "no-inputs",
        "no-capture",
        "obsolete",
        "missing-role",
        "wrong-role",
        "wrong-unit",
        "policy",
        "repeat",
        "duplicate",
    ] {
        with_native_theorem_artifact_context(
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            false,
            usize::MAX,
            "",
            |fixture, mut inputs, six, owner, refs, _, _| {
                let before = (
                    six.clone(),
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    fs::read(fixture.root.join("alpha/src/main.miz")).unwrap(),
                );
                let kernel = refs
                    .iter()
                    .find(|reference| {
                        reference.output_kind() == &OutputKind::new("SourceTheoremCandidateCheck")
                    })
                    .unwrap();
                let mut original = refs
                    .iter()
                    .map(|reference| owner.registry().output_lineage(reference.output()).unwrap())
                    .collect::<Vec<_>>();
                for parent in &kernel.lineage().parents {
                    if original.iter().all(|output| output.output != *parent) {
                        original.push(owner.registry().output_lineage(*parent).unwrap());
                    }
                }
                assert_eq!(original.len(), 14);
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
                let prefix = "proof-witnesses/service-refusal";
                let witness_path = format!(
                    "{prefix}/{}.json",
                    kernel
                        .content_hash()
                        .as_bytes()
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>()
                );
                let child = format!(
                    "{}-{}",
                    six.snapshot
                        .fingerprint()
                        .as_bytes()
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>(),
                    six.identities()
                        .input_hash()
                        .as_bytes()
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>()
                );
                let mut parent = "service-artifact-refusal";
                let parent_root = fixture.root.join("alpha").join(parent);
                if matches!(case, "escape" | "package-root") {
                    #[cfg(unix)]
                    {
                        let target = if case == "escape" {
                            fixture.root.clone()
                        } else {
                            fixture.root.join("alpha")
                        };
                        std::os::unix::fs::symlink(target, &parent_root).unwrap();
                    }
                    #[cfg(not(unix))]
                    {
                        return;
                    }
                } else if case != "missing-parent" {
                    fs::create_dir(&parent_root).unwrap();
                }
                if case != "denied" {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("ArtifactCommit"),
                        OutputKind::new("SourceTheoremArtifactData"),
                        kernel.work_unit().clone(),
                    ));
                }
                let mut parents = six.parent_outputs().to_vec();
                if case == "missing-role" {
                    parents.retain(|parent| {
                        parent.as_output_ref().output_kind()
                            != &OutputKind::new("SourceTheoremCandidateWitnessFile")
                    });
                }
                if case == "wrong-role" {
                    let metadata = refs
                        .iter()
                        .find(|reference| {
                            reference.output_kind() == &OutputKind::new("SourceTheoremMetadata")
                        })
                        .unwrap();
                    parents.retain(|parent| {
                        parent.as_output_ref().output_kind()
                            != &OutputKind::new("SourceTheoremCandidateWitnessFile")
                    });
                    parents.push(
                        SealedParentOutputHandle::from_published_output(
                            owner,
                            six.snapshot,
                            metadata.clone(),
                        )
                        .unwrap(),
                    );
                }
                let query = PhaseInput::new(
                    if case == "wrong-unit" {
                        WorkUnit::Workspace
                    } else {
                        six.work_unit.clone()
                    },
                    PhaseDispatchInputBundle::new(
                        six.snapshot,
                        six.identities().input_hash(),
                        vec![],
                        parents,
                    )
                    .unwrap(),
                );
                let mut builder = PhaseRegistryBuilder::new();
                builder
                    .register_source_theorem_artifact_commit(
                        policy.clone(),
                        profile.clone(),
                        ConcreteFormat::Tptp,
                        verifier.clone(),
                        (prefix.into(), "main.mizir.json".into(), parent.into()),
                    )
                    .unwrap();
                let mut registry = builder.build().unwrap();
                let first_root = parent_root.join(&child);
                let previous = if matches!(case, "repeat" | "duplicate") {
                    let first = registry
                        .execute_phase_with_resources(
                            PipelinePhase::ArtifactCommit,
                            six.clone(),
                            PhaseExecutionResources {
                                output_publisher: Some(owner.clone()),
                                source_load: Some(inputs),
                                ..PhaseExecutionResources::default()
                            },
                        )
                        .unwrap()
                        .result;
                    assert_eq!(first.status, PhaseStatus::Complete);
                    assert!(first.artifact_commit.is_some());
                    let [output] = first.output_refs.as_slice() else {
                        panic!("one real Artifact output")
                    };
                    let value = owner
                        .storage()
                        .get(
                            &owner
                                .storage()
                                .typed_handle::<VerifiedArtifact>(output, output.output_kind())
                                .unwrap(),
                        )
                        .unwrap();
                    Some((
                        output.clone(),
                        value,
                        [
                            "artifact-manifest.json",
                            "main.mizir.json",
                            witness_path.as_str(),
                        ]
                        .map(|path| fs::read(first_root.join(path)).unwrap()),
                    ))
                } else {
                    None
                };
                if case == "duplicate" {
                    parent = "service-artifact-refusal-next";
                    fs::create_dir(fixture.root.join("alpha").join(parent)).unwrap();
                    let mut next = PhaseRegistryBuilder::new();
                    next.register_source_theorem_artifact_commit(
                        policy,
                        profile,
                        ConcreteFormat::Tptp,
                        verifier,
                        (prefix.into(), "main.mizir.json".into(), parent.into()),
                    )
                    .unwrap();
                    let next = next.build().unwrap();
                    assert_eq!(
                        next.descriptor_for_phase(PipelinePhase::ArtifactCommit)
                            .unwrap(),
                        registry
                            .descriptor_for_phase(PipelinePhase::ArtifactCommit)
                            .unwrap()
                    );
                    registry = next;
                }
                if case == "obsolete" {
                    owner.mark_obsolete(six.snapshot).unwrap();
                }
                if case == "no-capture" {
                    inputs.captured = None;
                }
                let result = registry
                    .execute_phase_with_resources(
                        PipelinePhase::ArtifactCommit,
                        query,
                        PhaseExecutionResources {
                            cancellation: matches!(case, "cancel" | "foreign-token").then_some(
                                CancellationToken {
                                    snapshot: if case == "cancel" {
                                        six.snapshot
                                    } else {
                                        snapshot_id(0xa9)
                                    },
                                    generation: CancellationGeneration::new(1),
                                    reason: CancellationReason::ExplicitRequest,
                                },
                            ),
                            output_publisher: (case != "no-publisher").then(|| owner.clone()),
                            source_load: (case != "no-inputs").then_some(inputs),
                            ..PhaseExecutionResources::default()
                        },
                    )
                    .unwrap()
                    .result;
                assert_eq!(
                    result.status,
                    if case == "cancel" {
                        PhaseStatus::Cancelled
                    } else {
                        PhaseStatus::Blocking
                    },
                    "{case}"
                );
                assert!(result.output_refs.is_empty(), "{case}");
                assert!(result.artifact_commit.is_none(), "{case}");
                assert!(result.diagnostics.is_empty());
                let root = fixture.root.join("alpha").join(parent).join(&child);
                if matches!(case, "denied" | "duplicate") {
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
                    assert_eq!(read.manifest.artifact_root, format!("{parent}/{child}"));
                    let [entry] = read.manifest.modules.as_slice() else {
                        panic!("one real committed module")
                    };
                    let mut transaction =
                        ManifestTransaction::begin(&root, read.manifest.clone(), None).unwrap();
                    transaction.stage_module(entry.clone()).unwrap();
                    let commit = transaction
                        .commit(ManifestCommitOptions::default())
                        .unwrap();
                    assert_eq!(commit.write().artifact_hash, read.artifact_hash);
                    let proof =
                        CommittedWitnessPublicationProof::from_source_theorem_manifest_commit(
                            &commit,
                            &root,
                            &entry.module,
                            six.snapshot.fingerprint(),
                        )
                        .unwrap();
                    assert_eq!(
                        proof.verified_artifact_witnesses()[0].witness_path,
                        witness_path
                    );
                } else if case != "repeat" {
                    assert!(!root.exists(), "{case}");
                }
                if let Some((output, value, bytes)) = previous {
                    assert_eq!(
                        owner
                            .storage()
                            .get(
                                &owner
                                    .storage()
                                    .typed_handle::<VerifiedArtifact>(&output, output.output_kind())
                                    .unwrap()
                            )
                            .unwrap(),
                        value
                    );
                    owner
                        .validate_current_output(six.snapshot, &output)
                        .unwrap();
                    assert_eq!(
                        [
                            "artifact-manifest.json",
                            "main.mizir.json",
                            witness_path.as_str()
                        ]
                        .map(|path| fs::read(first_root.join(path)).unwrap()),
                        bytes
                    );
                }
                for output in original {
                    assert_eq!(
                        owner.registry().output_lineage(output.output).as_ref(),
                        Some(&output)
                    );
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
                let collected = owner.storage().collect(CollectInput {
                    snapshot: six.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    (collected.outputs_dropped, collected.abandoned_slots_dropped),
                    (
                        if matches!(case, "repeat" | "duplicate") {
                            15
                        } else {
                            14
                        },
                        usize::from(matches!(case, "denied" | "duplicate"))
                    ),
                    "{case}"
                );
            },
        );
    }
}
