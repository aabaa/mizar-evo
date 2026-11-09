use super::*;

#[test]
fn source_theorem_registration_closure_retains_native_inputs() -> Result<(), String> {
    use mizar_checker::{
        cluster_trace::{ClusterClosureStatus, ClusterTraceBuilder},
        registration_resolution::RegistrationDatabase,
        source_atomic_formula::SourceAtomicRequestKind,
        type_checker::{
            DeclarationChecker as Checker, check_source_theorem_registration_closure as check,
        },
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::NamespacePath,
        names::{SourceVariableScopeInput as Input, SourceVariableScopeResolver as Resolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::SignatureProjectionExtractor,
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceInput,
        SourceOrigin, SourceOriginInput, hash_text,
    };
    use mizar_syntax::SurfaceAst;
    let fixture = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/theorem_registration_closure.miz");
    let ids = InMemorySessionIdAllocator::new();
    let module = ModuleId::new(
        PackageId::new("theorem-closure-test"),
        ModulePath::new("owners"),
    );
    let lower = |ast: &SurfaceAst| {
        let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
        let shells = DeclarationShellCollector::new(ast, &module).collect();
        let collected =
            SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new("owners"))
                .collect(&module);
        assert!(collected.diagnostics().is_empty());
        let symbols = collected.env().clone();
        let scope =
            Resolver::resolve_proof_occurrences(Input::new(ast, &module, &symbols)).unwrap();
        (source, symbols, scope)
    };
    for text in [
        fixture.to_owned(),
        fixture.replace("Lem1", "Seed").replace('X', "Y"),
        "theorem Pair: for A, Spare, B being set holds A = B proof let L, Unused, R be set; thus L = R; end;".into(),
        "theorem Shadow: for A being set holds for A being set holds A = A proof end;".into(),
    ] {
        let mut disk: Option<SurfaceAst> = None;
        for buffer in [false, true] {
            fs::write(&path, if buffer { "stale disk content" } else { &text }).unwrap();
            let frontend = Frontend::new(FrontendSourceLoader::new(DiskSourceLoader::new(&root)), super::super::super::ParseOnlyImportProvider, MizarParserSeam).run(
                SourceUnitRequest { snapshot: snapshot_id(789), input: SourceInput {
                    package_id: module.package().clone(), module_path: module.path().clone(),
                    normalized_path: mizar_session::normalize_path(&root, &path).unwrap(), edition: Edition::new("2026"),
                    origin: if buffer { SourceOriginInput::OpenBuffer { uri: format!("file://{}", path.display()), expected_version: 3, actual_version: 3, text: text.clone().into() } } else { SourceOriginInput::Disk { path: path.clone() } },
                } }, &ids,
            ).unwrap();
            assert!(frontend.diagnostics.is_empty());
            assert_eq!(frontend.source.source_text.as_ref(), text);
            assert_eq!(frontend.source.source_hash, hash_text(&text));
            assert_eq!(matches!(frontend.source.origin, SourceOrigin::OpenBuffer { version: 3 }), buffer);
            let ast = frontend.ast.as_ref().unwrap();
            let (source, symbols, scope) = lower(ast);
            let baseline = Checker::check_source_theorem_equalities_with_typed_declarations(&source, &symbols, &scope)?;
            let (_, inference, typed, _) = &baseline;
            assert!(typed.facts().is_empty());
            assert!(typed.coercions().is_empty());
            assert!(typed.initial_obligations().is_empty());
            assert!(inference.facts().is_empty());
            assert!(inference.candidate_sets().is_empty());
            let requests = typed.source_atomic_formula().unwrap().requests();
            assert!(!requests.is_empty());
            assert!(requests.iter().all(|(_, request)| request.kind() == SourceAtomicRequestKind::OperandExpectedType));
            let database = RegistrationDatabase::from_symbol_env(&symbols, []);
            assert_eq!(database.module_id(), &module);
            assert!(database.pending().is_empty());
            assert!(database.activated().is_empty());
            assert!(database.rejected().is_empty());
            assert!(database.diagnostics().is_empty());
            assert!(database.initial_obligations().is_empty());
            let oracle = ClusterTraceBuilder::default().close(&database, source.source_id(), module.clone(), [], []);
            assert_eq!(oracle.status(), ClusterClosureStatus::Complete);
            assert_eq!(oracle.trace().source_id(), frontend.source.source_id);
            assert_eq!(oracle.trace().module_id(), &module);
            assert!(oracle.closure_facts().is_empty());
            assert!(oracle.diagnostics().is_empty());
            assert!(oracle.trace().steps().is_empty());
            assert!(oracle.trace().derived_facts().is_empty());
            let profile = oracle.trace().traversal_profile();
            assert_eq!(profile.ordering_version().as_str(), "cluster-trace-order-v1");
            assert_eq!(profile.cache_key_material().as_str(), "order=cluster-trace-order-v1;max_depth=128;max_generated=4096");
            assert_eq!(profile.max_cluster_depth(), 128);
            assert_eq!(profile.max_generated_facts(), 4096);
            assert!(!profile.bounded_saturation_reached());
            assert_eq!((profile.input_fact_count(), profile.derived_fact_count(), profile.cluster_step_count(), profile.reduction_step_count(), profile.diagnostic_count()), (0, 0, 0, 0, 0));
            if let Some(disk) = &disk {
                assert_ne!(disk.source_id, ast.source_id);
                let rebound = SurfaceAst::from_canonical_bytes(&disk.canonical_bytes().unwrap(), ast.source_id).unwrap();
                assert_eq!(rebound, *ast);
                let (rebound_source, rebound_symbols, rebound_scope) = lower(&rebound);
                let rebound_typed = Checker::check_source_theorem_equalities_with_typed_declarations(&rebound_source, &rebound_symbols, &rebound_scope)?.2;
                assert_eq!(rebound_typed, *typed);
                assert_eq!(check(&rebound_source, &rebound_symbols, &rebound_scope, &rebound_typed)?, oracle);
            } else { disk = Some(ast.clone()); }
            let before = (source.clone(), symbols.clone(), scope.clone(), baseline.clone(), database.clone(), ast.clone(), frontend.source.clone());
            for stage in 0..3 {
                if stage == 1 { fs::write(&path, "stale disk after capture").unwrap(); }
                else if stage == 2 { fs::remove_file(&path).unwrap(); }
                for _ in 0..2 { assert_eq!(check(&source, &symbols, &scope, typed)?, oracle); }
            }
            assert_eq!((source, symbols, scope, baseline.clone(), database, ast.clone(), frontend.source.clone()), before);
        }
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_registration_closure_preserves_upstream_errors() {
    use mizar_checker::type_checker::{
        DeclarationChecker as Checker, check_source_theorem_registration_closure as check,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::{NamespacePath, SymbolEnv, SymbolEnvIndexes},
        names::{SourceVariableScopeInput as Input, SourceVariableScopeResolver as Resolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::SignatureProjectionExtractor,
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, SourceInput, SourceOriginInput,
    };
    use mizar_syntax::SurfaceAst;
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let module = ModuleId::new(
        mizar_session::PackageId::new("closure-errors"),
        mizar_session::ModulePath::new("owners"),
    );
    let lower = |ast: &SurfaceAst, module: &ModuleId| {
        let source = SurfaceResolvedArena::lower(ast, module).unwrap();
        let shells = DeclarationShellCollector::new(ast, module).collect();
        let collected = SignatureProjectionExtractor::new(
            ast,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(module);
        let symbols = collected.env().clone();
        let scope = Resolver::resolve_proof_occurrences(Input::new(ast, module, &symbols)).unwrap();
        (source, symbols, scope)
    };
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/closure_errors.miz");
    let ids = InMemorySessionIdAllocator::new();
    let parse = |text: &str| {
        fs::write(&path, text).unwrap();
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: snapshot_id(790),
                input: SourceInput {
                    package_id: module.package().clone(),
                    module_path: module.path().clone(),
                    normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Disk { path: path.clone() },
                },
            },
            &ids,
        )
        .unwrap();
        assert!(
            frontend.diagnostics.is_empty(),
            "{:?}",
            frontend.diagnostics
        );
        frontend.ast.unwrap()
    };
    let original = parse(text);
    let (source, symbols, scope) = lower(&original, &module);
    let typed =
        Checker::check_source_theorem_equalities_with_typed_declarations(&source, &symbols, &scope)
            .unwrap()
            .2;
    let changed = parse(
        &text
            .replace("theorem T", "theorem Different")
            .replace('X', "Y"),
    );
    let rebound =
        SurfaceAst::from_canonical_bytes(&changed.canonical_bytes().unwrap(), original.source_id)
            .unwrap();
    let (other_source, other_symbols, other_scope) = lower(&rebound, &module);
    let other_typed = Checker::check_source_theorem_equalities_with_typed_declarations(
        &other_source,
        &other_symbols,
        &other_scope,
    )
    .unwrap()
    .2;
    assert_eq!(other_typed.source_id(), typed.source_id());
    assert_eq!(other_typed.module_id(), typed.module_id());
    assert_ne!(other_typed, typed);
    let before = (
        source.clone(),
        symbols.clone(),
        scope.clone(),
        typed.clone(),
        other_typed.clone(),
    );
    assert_eq!(
        check(&source, &symbols, &scope, &other_typed).unwrap_err(),
        "theorems.equality.invalid_registration_closure_association"
    );
    assert_eq!(
        check(&other_source, &other_symbols, &other_scope, &typed).unwrap_err(),
        "theorems.equality.invalid_registration_closure_association"
    );
    assert_eq!(
        (
            source.clone(),
            symbols.clone(),
            scope.clone(),
            typed.clone(),
            other_typed.clone()
        ),
        before
    );
    let empty = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
    let foreign_module = ModuleId::new(
        mizar_session::PackageId::new("foreign"),
        mizar_session::ModulePath::new("other"),
    );
    let (_, foreign_symbols, _) = lower(&original, &foreign_module);
    let foreign_ast = parse(text);
    assert_ne!(foreign_ast.source_id, original.source_id);
    let mut controls = vec![
        (source.clone(), empty.clone(), scope.clone()),
        (
            SurfaceResolvedArena::lower(&foreign_ast, &module).unwrap(),
            symbols.clone(),
            scope.clone(),
        ),
        (source.clone(), foreign_symbols, scope.clone()),
        (
            SurfaceResolvedArena::lower(&original, &foreign_module).unwrap(),
            symbols.clone(),
            scope.clone(),
        ),
        (source.clone(), symbols.clone(), other_scope.clone()),
    ];
    let private_control = controls.len();
    for (case, text) in [
        include_str!("../../../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz").to_owned(),
        format!("reserve spare for object; {text}"),
        format!("{text}\n{}", include_str!("../../../../../../tests/miz/pass/clusters/pass_advanced_semantics_existential_registration_001.miz")),
    ].into_iter().enumerate() {
        let ast = parse(&text);
        let (source, symbols, scope) = lower(&ast, &module);
        if case == 2 { assert!(!symbols.registrations().is_empty()); }
        controls.push((source.clone(), symbols, scope));
        controls.push((source, empty.clone(), other_scope.clone()));
    }
    for (index, (source, symbols, scope)) in controls.into_iter().enumerate() {
        let before = (
            source.clone(),
            symbols.clone(),
            scope.clone(),
            other_typed.clone(),
        );
        let lower = Checker::check_source_theorem_equalities_with_typed_declarations(
            &source, &symbols, &scope,
        );
        let expected = if index == private_control {
            assert_ne!(lower.unwrap().2, other_typed);
            "theorems.equality.invalid_registration_closure_association".to_owned()
        } else {
            lower.unwrap_err()
        };
        for _ in 0..2 {
            assert_eq!(
                check(&source, &symbols, &scope, &other_typed).unwrap_err(),
                expected
            );
        }
        assert_eq!((source, symbols, scope, other_typed.clone()), before);
    }
    fs::remove_dir_all(root).unwrap();
}
