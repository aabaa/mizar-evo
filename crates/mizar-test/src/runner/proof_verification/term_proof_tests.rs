use super::*;

fn config() -> crate::harness::DiscoveryConfig {
    crate::harness::DiscoveryConfig {
        workspace_root: Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap(),
        tests_root: "tests".into(),
        manifest_path: "tests/coverage/spec_trace.toml".into(),
        profile: crate::harness::TestProfile::Fast,
        validation_mode: crate::harness::ValidationMode::Development,
    }
}

fn parse(source: &str) -> mizar_syntax::SurfaceAst {
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceInput,
        SourceOriginInput,
    };
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/membership.miz");
    fs::write(&path, source).unwrap();
    let output = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
        super::super::ParseOnlyImportProvider,
        MizarParserSeam,
    )
    .run(
        SourceUnitRequest {
            snapshot: snapshot_id(777),
            input: SourceInput {
                package_id: PackageId::new("membership-test"),
                module_path: ModulePath::new("membership"),
                normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                edition: Edition::new("2026"),
                origin: SourceOriginInput::Disk { path: path.clone() },
            },
        },
        &InMemorySessionIdAllocator::new(),
    )
    .unwrap();
    fs::remove_file(path).unwrap();
    fs::remove_dir(root.join("src")).unwrap();
    fs::remove_dir(root).unwrap();
    // Exercise the checker even when frontend diagnostics would already reject.
    output.ast.unwrap()
}

fn check_theorem_source(
    source: &str,
    phase: PipelinePhase,
) -> Result<
    (
        Vec<mizar_proof::policy::CandidatePolicyClass>,
        Option<VcSet>,
    ),
    String,
> {
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::super::formula_statement::STEP5C10_CASES[0].0)
        .unwrap();
    let ast = parse(source);
    let resolver = resolver_symbol_collection(&config.workspace_root, case, &ast);
    if !resolver.detail_keys.is_empty() {
        return Err(format!("resolver: {:?}", resolver.detail_keys));
    }
    theorem_ast_output(
        &ast,
        &resolver.module,
        &resolver.env,
        phase,
        snapshot_id(777),
        None,
    )
}

fn with_step5c14_theorem_check<T>(
    text: &str,
    consume: impl FnOnce(&mizar_checker::type_checker::SourceTheoremCheck<'_>) -> Result<T, String>,
) -> Result<T, String> {
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    let config = config();
    let plan = crate::harness::build_test_plan(&config).unwrap();
    let case = plan
        .cases
        .iter()
        .find(|case| case.id.0 == super::super::formula_statement::STEP5C10_CASES[0].0)
        .unwrap();
    let ast = parse(text);
    let resolver = resolver_symbol_collection(&config.workspace_root, case, &ast);
    if !resolver.detail_keys.is_empty() {
        return Err(format!("{:?}", resolver.detail_keys));
    }
    let source = SurfaceResolvedArena::lower(&ast, &resolver.module).map_err(|e| e.to_string())?;
    with_source_theorem_check(&ast, &source, &resolver.env, consume)
}

fn with_source_theorem_check<T>(
    ast: &mizar_syntax::SurfaceAst,
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    symbols: &mizar_resolve::env::SymbolEnv,
    consume: impl FnOnce(&mizar_checker::type_checker::SourceTheoremCheck<'_>) -> Result<T, String>,
) -> Result<T, String> {
    use mizar_checker::type_checker::SourceVariableSemanticsChecker;
    use mizar_resolve::{
        labels::{LabelResolver, ProofLabelSourceCollector},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
    };
    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(ast, source.module(), symbols),
    )
    .map_err(|e| format!("{e:?}"))?;
    let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
    let typed = super::super::type_elaboration::step5c8_formula_typed_ast(
        ast,
        source.module(),
        symbols,
        &scope,
        &bindings,
        true,
    )?;
    let owner = symbols
        .symbols()
        .iter()
        .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
        .or_else(|| {
            symbols
                .symbols()
                .iter()
                .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Lemma)
        })
        .ok_or("no theorem")?;
    let namespace = mizar_resolve::env::NamespacePath::new(source.module().path().as_str());
    let labels = ProofLabelSourceCollector::new(
        ast,
        source.module(),
        namespace.clone(),
        owner.contribution(),
        source,
    )
    .and_then(|collector| collector.collect_with_theorem_owners(symbols))
    .map_err(|e| e.to_string())?;
    let resolved = LabelResolver::new(labels.projections()).resolve(
        source.module(),
        &namespace,
        labels.references(),
    );
    let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
        source, &typed, &scope, symbols, &labels, &resolved, None,
    )?;
    consume(&check)
}

fn step5c14_computation_core(text: &str) -> Result<mizar_core::core_ir::CoreIr, String> {
    with_step5c14_theorem_check(text, |check| {
        mizar_core::elaborator::lower_source_theorem_skeletons(check, None)
    })
}

fn with_source_algorithm_identity_inputs<R>(
    text: &str,
    package: &str,
    module: &str,
    skip: usize,
    f: impl FnOnce(
        &mizar_resolve::resolved_ast::SurfaceResolvedArena,
        &mizar_checker::typed_ast::TypedArena,
        &mizar_resolve::env::SymbolEnv,
        &mizar_checker::type_checker::SourceAlgorithmCheck<'_>,
    ) -> R,
) -> R {
    use mizar_checker::typed_ast::{TypedArena, TypedNode, TypedNodeId};
    let (frontend, source, collected) =
        super::super::tests::source_registration_owner_inputs(text, package, module, skip);
    assert!(frontend.diagnostics.is_empty());
    assert!(collected.diagnostics().is_empty());
    let typed = TypedArena::try_new(
        Some(TypedNodeId::new(source.arena().root().index())),
        source
            .arena()
            .iter()
            .map(|(id, node)| {
                TypedNode::new(format!("{:?}", node.kind()), node.origin().anchor().clone())
                    .with_resolved_node(id)
                    .with_children(
                        node.children()
                            .iter()
                            .map(|child| TypedNodeId::new(child.index()))
                            .collect(),
                    )
            })
            .collect(),
    )
    .unwrap();
    let check =
        mizar_checker::type_checker::check_source_algorithm_types(&source, &typed, collected.env())
            .unwrap();
    f(&source, &typed, collected.env(), &check)
}

fn with_source_theorem_file_staging_inputs<F>(mut check: F) -> Result<(), String>
where
    F: FnMut(
        usize,
        &(
            (
                mizar_proof::status::ProofStatusProjection,
                mizar_vc::dependency_slice::DependencySlice,
                mizar_proof::witness_store::ProofWitnessDraft,
            ),
            mizar_artifact::store::CanonicalJson,
            mizar_artifact::proof_witness::ProofWitnessRef,
        ),
        &mizar_artifact::store::PublishedArtifactRead,
        &mizar_proof::witness_store::ProofWitnessStagedRef,
    ) -> Result<(), String>,
{
    use mizar_artifact::store::{
        PublishedArtifactPath, PublishedArtifactReadOptions, SchemaVersion, artifact_hash_domain,
        read_published_artifact, write_published_artifact,
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
        witness_store::{read_source_theorem_witness_file as recover, stage},
    };
    use mizar_resolve::{resolved_ast::ModuleId, symbols::source_theorem_module_original_inputs};
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SnapshotRegistry, SourceInput, SourceOrigin, SourceOriginInput,
        ToolchainInfo, WorkspaceRoot, hash_text,
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
    let renamed = original.replace("T:", "Renamed:").replace(
        "for X being set holds X = X",
        "for Bound being set holds Bound = Bound",
    );
    let local = original.replace(
        "let X be set; thus X = X",
        "let Local be set; thus Local = Local",
    );
    let policy = Policy::default();
    let cases = [
        (
            original,
            false,
            policy.clone(),
            "proof-witnesses/stage/a.json",
        ),
        (
            original,
            true,
            policy.clone(),
            "proof-witnesses/stage/a.json",
        ),
        (
            &renamed,
            false,
            policy.clone(),
            "proof-witnesses/stage/a.json",
        ),
        (
            &local,
            false,
            policy.clone(),
            "proof-witnesses/stage/a.json",
        ),
        (
            original,
            false,
            policy.clone().with_profile_id("staging-policy"),
            "proof-witnesses/stage/a.json",
        ),
        (original, false, policy, "proof-witnesses/stage/b.json"),
    ];
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let domain = artifact_hash_domain(
        "mizar-proof/source-theorem-witness-file",
        SchemaVersion::new(1, 0),
    );
    for (index, (text, open, policy, witness_path)) in cases.into_iter().enumerate() {
        fs::write(&path, if open { "stale disk content" } else { text }).unwrap();
        let input = SourceInput {
            package_id: PackageId::new("witness-staging-test"),
            module_path: ModulePath::new("owners"),
            normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
            edition: Edition::new("2026"),
            origin: if open {
                SourceOriginInput::OpenBuffer {
                    uri: format!("file://{}", path.display()),
                    expected_version: 3,
                    actual_version: 3,
                    text: text.to_owned().into(),
                }
            } else {
                SourceOriginInput::Disk { path: path.clone() }
            },
        };
        let (snapshot, lease, mut loaded) = registry
            .load_snapshot(
                ids.next_request_id().unwrap(),
                (
                    WorkspaceRoot::new(root.to_str().unwrap()),
                    Vec::new(),
                    hash_text(include_str!("../../../../../Cargo.lock")),
                    ToolchainInfo::new("caller-recorded-witness-staging-test"),
                    policy.policy_fingerprint().hash(),
                ),
                vec![(DiskSourceLoader::new(&root), input.clone())],
            )
            .unwrap();
        let (loaded, received) = loaded.pop().unwrap();
        assert_eq!(received, input);
        assert_eq!(loaded.source_hash, hash_text(text));
        assert_eq!(
            matches!(loaded.origin, SourceOrigin::OpenBuffer { .. }),
            open
        );
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run_loaded(source_unit_from_loaded(loaded, path.clone()))
        .unwrap();
        assert!(frontend.diagnostics.is_empty());
        let module = ModuleId::new(input.package_id, input.module_path);
        let (source, symbols) =
            source_theorem_module_original_inputs(frontend.ast.as_ref().unwrap(), &module)?
                .unwrap();
        let before = (
            source.clone(),
            symbols.clone(),
            snapshot.clone(),
            policy.clone(),
        );
        let original_product = acquire(&source, &symbols, &snapshot, &policy, witness_path)?;
        let actual = candidate(&source, &symbols, &snapshot, &policy, witness_path)?;
        assert_eq!(actual.0, original_product);
        let legacy = stage(actual.0.2.clone()).unwrap();
        assert!(legacy.artifact_reference_candidate().is_none());
        let transport_path = PublishedArtifactPath::new(witness_path).unwrap();
        let written = write_published_artifact(
            root.join("unpublished-staging-files"),
            &transport_path,
            &actual.1,
            &domain,
            &[],
        )
        .unwrap();
        assert_eq!(written.artifact_hash, actual.2.witness_artifact_hash.digest);
        let read = read_published_artifact(
            root.join("unpublished-staging-files"),
            &transport_path,
            PublishedArtifactReadOptions {
                artifact_hash_domain: Some(&domain),
                expected_artifact_hash: Some(written.artifact_hash),
                ..PublishedArtifactReadOptions::default()
            },
        )
        .unwrap();
        assert_eq!(read.value, actual.1);
        assert_eq!(recover(&read, &actual.2)?, actual.0.2.payload_bytes());
        let received_before = (actual.clone(), read.clone());
        check(index, &actual, &read, &legacy)?;
        assert_eq!((actual, read), received_before);
        assert_eq!((source, symbols, snapshot.clone(), policy), before);
        assert_eq!(registry.get(snapshot.id), Some(snapshot.clone()));
        registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

fn with_source_algorithm_state_draft_inputs<R>(
    text: &str,
    policy: &mizar_proof::policy::VerifierPolicy,
    f: impl FnOnce(
        &mizar_resolve::resolved_ast::SurfaceResolvedArena,
        &mizar_checker::type_checker::SourceAlgorithmCheck<'_>,
        &mizar_resolve::env::SymbolEnv,
        &mizar_session::BuildSnapshot,
    ) -> R,
) -> R {
    use mizar_checker::typed_ast::{TypedArena, TypedNode, TypedNodeId};
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::NamespacePath,
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::SignatureProjectionExtractor,
    };
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
    let path = root.join(
        "src/tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
    );
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, text).unwrap();
    let input = SourceInput {
        package_id: PackageId::new("mizar-test-corpus"),
        module_path: ModulePath::new(
            "tests.miz.pass.algorithms.pass_proof_verification_algorithm_var_const_assert_001",
        ),
        normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
        edition: Edition::new("2026"),
        origin: SourceOriginInput::Disk { path: path.clone() },
    };
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let (snapshot, _lease, loaded) = registry
        .load_snapshot(
            ids.next_request_id().unwrap(),
            (
                WorkspaceRoot::new(root.to_str().unwrap()),
                vec![],
                hash_text(include_str!("../../../../../Cargo.lock")),
                ToolchainInfo::new("caller-recorded-state-draft-test"),
                policy.policy_fingerprint().hash(),
            ),
            vec![(DiskSourceLoader::new(&root), input.clone())],
        )
        .unwrap();
    let [(loaded, _)] = <[_; 1]>::try_from(loaded).ok().unwrap();
    let [version] = snapshot.source_versions.as_slice() else {
        panic!("one source")
    };
    assert_eq!(version.source_id, loaded.source_id);
    let unit = source_unit_from_loaded(loaded, path);
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
        super::super::ParseOnlyImportProvider,
        MizarParserSeam,
    )
    .run_loaded(unit)
    .unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.as_ref().unwrap();
    assert_eq!(ast.source_id, version.source_id);
    let module = ModuleId::new(input.package_id, input.module_path);
    let shells = DeclarationShellCollector::new(ast, &module).collect();
    let collected =
        SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new(module.path().as_str()))
            .collect(&module);
    assert!(collected.diagnostics().is_empty());
    let symbols = collected.into_env();
    let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
    assert_eq!(source.source_id(), version.source_id);
    let typed = TypedArena::try_new(
        Some(TypedNodeId::new(source.arena().root().index())),
        source
            .arena()
            .iter()
            .map(|(id, node)| {
                TypedNode::new(format!("{:?}", node.kind()), node.origin().anchor().clone())
                    .with_resolved_node(id)
                    .with_children(
                        node.children()
                            .iter()
                            .map(|child| TypedNodeId::new(child.index()))
                            .collect(),
                    )
            })
            .collect(),
    )
    .unwrap();
    let check =
        mizar_checker::type_checker::check_source_algorithm_types(&source, &typed, &symbols)
            .unwrap();
    let result = f(&source, &check, &symbols, &snapshot);
    fs::remove_dir_all(root).unwrap();
    result
}

mod misc;
mod original;
mod source_algorithm_core;
mod source_algorithm_formula;
mod source_algorithm_return;
mod source_algorithm_state;
mod source_assumption;
mod source_builtin;
mod source_descendant;
mod source_empty;
mod source_mixed;
mod source_named;
mod source_nested;
mod source_parenthesized;
mod source_reserved;
mod source_singleton;
mod source_theorem_candidate;
mod source_theorem_canonical;
mod source_theorem_context;
mod source_theorem_declaration;
mod source_theorem_discharge;
mod source_theorem_equality;
mod source_theorem_expression;
mod source_theorem_final;
mod source_theorem_lexical;
mod source_theorem_local;
mod source_theorem_obligation;
mod source_theorem_reflexivity;
mod source_theorem_registration;
mod source_theorem_scoped;
mod source_theorem_type;
mod source_theorem_typed;
mod source_theorem_variable;
mod source_theorem_witness;
mod source_witness;
mod step5c10;
mod step5c14;
mod step5c5;
mod step5c7;
