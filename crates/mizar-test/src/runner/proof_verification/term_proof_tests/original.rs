use super::*;

#[test]
fn original_source_witness_draft_matches_genuine_pipeline() -> Result<(), String> {
    use mizar_core::elaborator::lower_source_theorem_skeletons;
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_proof::{
        policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
        status::{
            ArtifactPublicationGap, ProjectedProofStatus,
            source_theorem_reflexivity_witness_draft as legacy,
            source_theorem_reflexivity_witness_draft_from_original as acquire,
        },
    };
    use mizar_resolve::{resolved_ast::ModuleId, symbols::source_theorem_module_original_inputs};
    use mizar_session::{
        DependencyArtifactRef, DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath,
        PackageId, SessionIdAllocator, SnapshotRegistry, SourceInput, SourceOrigin,
        SourceOriginInput, ToolchainInfo, WorkspaceRoot, hash_text,
    };
    use mizar_vc::{
        dependency_slice::DependencySliceCompleteness,
        generator::generate_source_theorem_vcs,
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion, VcStatus},
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
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let lockfile_hash = hash_text(
        &fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock"))
            .unwrap(),
    );
    let mut captures = Vec::new();
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
            package_id: PackageId::new("original-draft-test"),
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
        // These supplied references are retained metadata, not authenticated dependencies.
        let dependencies = if index == 3 {
            vec![DependencyArtifactRef::new(
                "caller-recorded/source",
                hash_text(&text),
            )]
        } else {
            Vec::new()
        };
        let (snapshot, lease, mut loaded) = registry
            .load_snapshot(
                ids.next_request_id().unwrap(),
                (
                    WorkspaceRoot::new(root.to_str().unwrap()),
                    dependencies,
                    lockfile_hash,
                    ToolchainInfo::new(if index == 1 {
                        "caller-supplied-original-draft-variant-toolchain"
                    } else {
                        "caller-supplied-original-draft-test-toolchain"
                    }),
                    Policy::default().policy_fingerprint().hash(),
                ),
                vec![(DiskSourceLoader::new(&root), input.clone())],
            )
            .unwrap();
        assert_eq!(registry.get(snapshot.id), Some(snapshot.clone()));
        assert_eq!(lease.snapshot, snapshot.id);
        let (loaded, received) = loaded.pop().unwrap();
        assert_eq!(received, input);
        let version = &snapshot.source_versions[0];
        assert_eq!(version.source_id, loaded.source_id);
        assert_eq!(version.source_hash, hash_text(&text));
        assert_eq!(version.source_hash, loaded.source_hash);
        assert_eq!(version.origin, loaded.origin);
        assert_eq!(matches!(version.origin, SourceOrigin::Disk), index % 2 == 0);
        let unit = source_unit_from_loaded(loaded, path.clone());
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run_loaded(unit.clone())
        .unwrap();
        assert_eq!(frontend.source, unit);
        assert_eq!(frontend.source.source_text.as_ref(), text);
        assert!(frontend.diagnostics.is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        let module = ModuleId::new(input.package_id.clone(), input.module_path.clone());
        let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
        assert_eq!(source.source_id(), version.source_id);
        let before = (
            source.clone(),
            symbols.clone(),
            snapshot.clone(),
            unit.clone(),
            input.clone(),
            ast.snapshot_text(),
        );
        for policy in [
            Policy::default(),
            Policy::release().with_require_kernel_certificates(true),
            Policy::development().with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
            Policy::default().with_profile_id("original-draft-policy"),
        ]
        .into_iter()
        .take(if index == 0 { 4 } else { 1 })
        {
            let witness_path = format!("proof-witnesses/original-{index}/checked.bin");
            let supplied = (policy.clone(), witness_path.clone());
            let mut previous = None;
            for _ in 0..2 {
                // This existing helper uses the independent legacy typed-source route.
                let expected = with_source_theorem_check(ast, &source, &symbols, |check| {
                    let core = lower_source_theorem_skeletons(check, None)?;
                    let vcs = generate_source_theorem_vcs(
                        &core,
                        snapshot.id,
                        &GenerationSchemaVersion::new("source-theorem-local-v1"),
                        &VcSchemaVersion::new("vc-v1"),
                    )?;
                    let [vc] = vcs.vcs() else {
                        panic!("expected one genuine VC")
                    };
                    legacy(
                        &source,
                        check,
                        &symbols,
                        &snapshot,
                        vc.id,
                        &policy,
                        &witness_path,
                    )
                })?;
                let actual = acquire(&source, &symbols, &snapshot, &policy, &witness_path)?;
                assert_eq!(actual, expected);
                assert_eq!(actual.0.projected_status(), ProjectedProofStatus::Accepted);
                assert_eq!(
                    actual.0.artifact_publication().gap(),
                    Some(ArtifactPublicationGap::DischargedBuiltinWitnessUnsupported)
                );
                assert_eq!(actual.1.status(), &VcStatus::Open);
                assert_eq!(
                    actual.1.completeness(),
                    DependencySliceCompleteness::IncompleteUncacheable
                );
                assert!(actual.1.requires_cache_miss());
                assert!(!actual.0.reuse_metadata().cache_reuse_predicate_complete());
                assert!(actual.0.trusted_used_axioms().is_none());
                assert!(actual.2.provenance().trusted_used_axioms_ref().is_none());
                assert!(actual.2.provenance().advisory_backend_ref().is_none());
                assert_eq!(
                    actual.2.provenance().build_snapshot_fingerprint(),
                    snapshot.id.fingerprint()
                );
                assert_eq!(
                    actual.2.provenance().dependency_artifact_fingerprint(),
                    snapshot.dependency_artifact_fingerprint()
                );
                assert_eq!(
                    actual.2.provenance().verifier_policy_fingerprint(),
                    policy.policy_fingerprint().hash()
                );
                assert_eq!(actual.2.witness_path(), witness_path);
                assert!(!root.join(&witness_path).exists());
                if let Some(previous) = previous {
                    assert_eq!(actual, previous);
                }
                previous = Some(actual);
            }
            assert_eq!((policy, witness_path), supplied);
            captures.push(previous.unwrap());
        }
        assert_eq!(
            (
                source,
                symbols,
                snapshot.clone(),
                frontend.source,
                input,
                ast.snapshot_text()
            ),
            before
        );
        registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    }
    assert_ne!(
        captures[0].2.provenance().build_snapshot_fingerprint(),
        captures[4].2.provenance().build_snapshot_fingerprint()
    );
    assert_ne!(
        captures[0].0.reuse_metadata().obligation_anchor(),
        captures[5].0.reuse_metadata().obligation_anchor()
    );
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn original_source_witness_draft_preserves_errors() -> Result<(), String> {
    use mizar_checker::type_checker::{SourceVariableSemanticsChecker, TermFormulaChecker};
    use mizar_core::elaborator::lower_source_theorem_skeletons;
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_proof::{
        policy::VerifierPolicy as Policy,
        status::{
            source_theorem_obligation_identity, source_theorem_reflexivity_witness_draft as legacy,
            source_theorem_reflexivity_witness_draft_from_original as acquire,
        },
    };
    use mizar_resolve::{
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::{
            source_proof_inputs, source_theorem_module_candidates,
            source_theorem_module_original_inputs,
        },
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SnapshotRegistry, SourceInput, SourceOriginInput, ToolchainInfo,
        WorkspaceRoot, hash_text,
    };
    use mizar_vc::{
        generator::generate_source_theorem_vcs,
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/owners.miz");
    fs::write(&path, original).unwrap();
    let input = SourceInput {
        package_id: PackageId::new("original-draft-errors"),
        module_path: ModulePath::new("owners"),
        normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
        edition: Edition::new("2026"),
        origin: SourceOriginInput::Disk { path: path.clone() },
    };
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let policies = [
        Policy::default(),
        Policy::default().with_kernel_evidence_formats([]),
    ];
    let policies_before = policies.clone();
    let (snapshot, lease, mut loaded) = registry
        .load_snapshot(
            ids.next_request_id().unwrap(),
            (
                WorkspaceRoot::new(root.to_str().unwrap()),
                Vec::new(),
                hash_text(
                    &fs::read_to_string(
                        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock"),
                    )
                    .unwrap(),
                ),
                ToolchainInfo::new("caller-supplied-original-draft-test-toolchain"),
                policies[0].policy_fingerprint().hash(),
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
    .run_loaded(source_unit_from_loaded(loaded, path.clone()))
    .unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.as_ref().unwrap();
    let module = ModuleId::new(input.package_id, input.module_path);
    let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
    let before = (
        source.clone(),
        symbols.clone(),
        snapshot.clone(),
        frontend.source.clone(),
        ast.snapshot_text(),
    );
    with_source_theorem_check(ast, &source, &symbols, |check| {
        let core = lower_source_theorem_skeletons(check, None)?;
        let vcs = generate_source_theorem_vcs(
            &core,
            snapshot.id,
            &GenerationSchemaVersion::new("source-theorem-local-v1"),
            &VcSchemaVersion::new("vc-v1"),
        )?;
        let [vc] = vcs.vcs() else {
            panic!("expected one genuine VC")
        };
        assert!(
            source_theorem_obligation_identity(&source, check, &symbols, snapshot.id, vc.id)
                .is_ok()
        );
        for policy in &policies {
            for witness_path in ["", "../escape.bin", "proof-witnesses/errors/checked.bin"] {
                if policy == &policies[0] && witness_path == "proof-witnesses/errors/checked.bin" {
                    continue;
                }
                let expected = legacy(
                    &source,
                    check,
                    &symbols,
                    &snapshot,
                    vc.id,
                    policy,
                    witness_path,
                )
                .unwrap_err();
                assert_eq!(
                    acquire(&source, &symbols, &snapshot, policy, witness_path).unwrap_err(),
                    expected
                );
            }
        }
        Ok(())
    })?;
    let foreign_ast = parse(&original.replace("T:", "Foreign:"));
    let (_, foreign) = source_theorem_module_original_inputs(&foreign_ast, &module)?.unwrap();
    let (_, augmented) = source_theorem_module_candidates(ast, &source, &symbols)?.unwrap();
    assert_ne!(augmented, symbols);
    for env in [&foreign, &augmented] {
        let env_before = env.clone();
        let expected = source_proof_inputs(&source, env).unwrap_err();
        assert_eq!(
            acquire(&source, env, &snapshot, &policies[1], "").unwrap_err(),
            expected
        );
        assert_eq!(env, &env_before);
    }
    let cited = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for (index, text) in [
        original.replacen("theorem", "lemma", 1),
        cited.to_owned(),
        "theorem Pair: for A, B being set holds A = B proof let L, R be set; thus L = R; end;"
            .into(),
        original.replace("being set", "being object"),
        original.replace("thus X = X;", "thus X = X by Missing;"),
        original.replace("thus X = X;", "thus X = Missing;"),
        "theorem T: for X being set holds X = ;".into(),
    ]
    .into_iter()
    .enumerate()
    {
        let ast = parse(&text);
        let source = SurfaceResolvedArena::lower(&ast, &module).unwrap();
        let shells =
            mizar_resolve::declarations::DeclarationShellCollector::new(&ast, &module).collect();
        let collected = mizar_resolve::symbols::SignatureProjectionExtractor::new(
            &ast,
            &shells,
            mizar_resolve::env::NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        let symbols = collected.env();
        let before = (source.clone(), symbols.clone(), ast.snapshot_text());
        let expected = match source_proof_inputs(&source, symbols) {
            Err(error) => {
                assert!(index >= 5);
                error
            }
            Ok((scope, labels, resolved)) => {
                match TermFormulaChecker::check_source_theorem_equalities(&source, symbols, &scope)
                {
                    Err(error) => {
                        assert_eq!(index, 3);
                        error
                    }
                    Ok((_, inference, typed)) => {
                        assert!(inference.diagnostics().is_empty());
                        match SourceVariableSemanticsChecker::check_theorem_skeletons(
                            &source, &typed, &scope, symbols, &labels, &resolved, None,
                        ) {
                            Err(error) => {
                                assert!(index == 2 || index == 4);
                                error
                            }
                            Ok(check) => {
                                assert!(index < 2);
                                let core = lower_source_theorem_skeletons(&check, None)?;
                                let vcs = generate_source_theorem_vcs(
                                    &core,
                                    snapshot.id,
                                    &GenerationSchemaVersion::new("source-theorem-local-v1"),
                                    &VcSchemaVersion::new("vc-v1"),
                                )?;
                                if index == 1 {
                                    assert_eq!(vcs.vcs().len(), 2);
                                    "theorem.proof.invalid_source_witness_draft".to_owned()
                                } else {
                                    let [vc] = vcs.vcs() else {
                                        panic!("expected one lemma VC")
                                    };
                                    let identity_error = source_theorem_obligation_identity(
                                        &source,
                                        &check,
                                        symbols,
                                        snapshot.id,
                                        vc.id,
                                    )
                                    .unwrap_err();
                                    assert_eq!(
                                        legacy(
                                            &source,
                                            &check,
                                            symbols,
                                            &snapshot,
                                            vc.id,
                                            &policies[1],
                                            ""
                                        )
                                        .unwrap_err(),
                                        identity_error
                                    );
                                    identity_error
                                }
                            }
                        }
                    }
                }
            }
        };
        for policy in &policies {
            assert_eq!(
                acquire(&source, symbols, &snapshot, policy, "").unwrap_err(),
                expected,
                "case {index}"
            );
        }
        // A foreign environment takes precedence over later type/label/policy/path failures.
        let env_error = source_proof_inputs(&source, &foreign).unwrap_err();
        assert_eq!(
            acquire(&source, &foreign, &snapshot, &policies[1], "").unwrap_err(),
            env_error
        );
        assert_eq!((source, symbols.clone(), ast.snapshot_text()), before);
    }
    assert_eq!(
        (
            source,
            symbols,
            snapshot.clone(),
            frontend.source,
            ast.snapshot_text()
        ),
        before
    );
    assert_eq!(policies, policies_before);
    assert_eq!(registry.get(snapshot.id), Some(snapshot.clone()));
    registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    fs::remove_dir_all(root).unwrap();
    Ok(())
}
