use super::*;

#[test]
fn source_assumption_statement_original_capture_transports_native_rows() -> Result<(), String> {
    use super::super::super::type_elaboration::assemble_empty_resolved_typed_ast;
    use mizar_checker::{
        binding_env::*, source_atomic_formula::*, source_statement::*, source_term::*,
        type_checker::TermFormulaChecker,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        resolved_ast::ModuleId,
        symbols::{
            source_proof_inputs, source_proof_scope_maps, source_theorem_module_original_inputs,
            validate_source_proof_occurrences,
        },
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceInput,
        SourceOriginInput,
    };
    use mizar_syntax::SurfaceNodeKind as K;
    // EN04/15/16: these rows preserve source intent without accepting an assumption as a premise.
    let original = "reserve x for set;\ntheorem FormulaStatementSingleAssumptionSmoke: x = x proof\n  assume x = x;\n  thus x = x;\nend;\n";
    let texts = [original.to_owned(), original.replace("FormulaStatementSingleAssumptionSmoke", "RenamedAssumption").replace("x", "parcel"), "reserve\n parcel\tfor :: type hint\n set;\ntheorem Layout: parcel = parcel proof :: syntax only\n assume parcel = parcel; thus parcel = parcel; end;".to_owned()];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src/b2.miz");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let mut cases = Vec::new();
    for (variant, text) in texts.into_iter().enumerate() {
        for buffer in [false, true] {
            fs::write(&path, if buffer { "stale disk" } else { &text }).unwrap();
            let frontend = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
                super::super::super::ParseOnlyImportProvider,
                MizarParserSeam,
            )
            .run(
                SourceUnitRequest {
                    snapshot: snapshot_id(778),
                    input: SourceInput {
                        package_id: PackageId::new("assumption-statement-test"),
                        module_path: ModulePath::new("b2"),
                        normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                        edition: Edition::new("2026"),
                        origin: if buffer {
                            SourceOriginInput::OpenBuffer {
                                uri: format!("file://{}", path.display()),
                                expected_version: 3,
                                actual_version: 3,
                                text: text.clone().into(),
                            }
                        } else {
                            SourceOriginInput::Disk { path: path.clone() }
                        },
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
            assert!(frontend.preprocessed.import_stubs.is_empty());
            assert_eq!(frontend.source.source_text.as_ref(), text);
            let ast = frontend.ast.as_ref().unwrap();
            let module = ModuleId::new(
                frontend.source.package_id.clone(),
                frontend.source.module_path.clone(),
            );
            let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
            let (scope, labels, resolution) = source_proof_inputs(&source, &symbols)?;
            source.validate_against(ast, &module).unwrap();
            validate_source_proof_occurrences(&source, &symbols, &scope)?;
            let (scopes, parents) = source_proof_scope_maps(&source, &symbols, &scope)?;
            assert_eq!(
                (source.arena().len(), scopes.len(), parents.len()),
                (55, 55, 54)
            );
            assert_eq!(
                (
                    scope.bindings().len(),
                    scope.references().len(),
                    labels.projections().len(),
                    labels.references().len(),
                    resolution.index().len(),
                    resolution.ids().len()
                ),
                (1, 6, 1, 0, 1, 0)
            );
            assert!(!resolution.has_unresolved() && resolution.diagnostics().is_empty());
            assert_eq!(
                scope
                    .references()
                    .iter()
                    .map(|row| row.ordinal())
                    .collect::<Vec<_>>(),
                (0..6).collect::<Vec<_>>()
            );
            for (kind, count) in [
                (K::BuiltinPredicateApplication, 3),
                (K::ProofBlock, 1),
                (K::CompactStatement, 0),
            ] {
                assert_eq!(
                    ast.node_views().filter(|node| node.kind() == &kind).count(),
                    count
                );
            }
            let before = (
                source.clone(),
                symbols.clone(),
                scope.clone(),
                labels.clone(),
                resolution.clone(),
                scopes.clone(),
                parents.clone(),
                ast.snapshot_text(),
                frontend.source.clone(),
            );
            eprintln!(
                "B2 resolver ready: variant={variant} buffer={buffer} nodes=55 reserve=1 refs=0..6 equalities=3 proof=1 labels=1 citations=0"
            );
            cases.push((
                frontend, source, symbols, scope, labels, resolution, scopes, parents, before,
            ));
        }
    }
    assert_eq!(cases.len(), 6);
    // Stage 1 runtime RED is permitted only after every real capture is ready.
    let bases = cases
        .iter()
        .map(|(_, source, symbols, scope, ..)| {
            TermFormulaChecker::check_source_theorem_equalities(source, symbols, scope)
        })
        .collect::<Vec<_>>();
    let errors = bases
        .iter()
        .filter_map(|result| result.as_ref().err())
        .collect::<Vec<_>>();
    assert!(
        errors.is_empty(),
        "native assumption equality bases: {errors:?}"
    );
    for ((frontend, source, _, scope, _, _, scopes, parents, _), result) in cases.iter().zip(&bases)
    {
        let (bindings, inference, base) = result.as_ref().unwrap();
        let ast = frontend.ast.as_ref().unwrap();
        let primary = base.source_term().unwrap();
        let atomic = base.source_atomic_formula().unwrap();
        assert_eq!(
            [
                bindings.contexts().len(),
                bindings.bindings().len(),
                primary.terms().len(),
                primary.references().len(),
                atomic.formulas().len(),
                atomic.edges().len()
            ],
            [2, 1, 6, 6, 3, 6]
        );
        let (binding_id, binding) = bindings.bindings().iter().next().unwrap();
        assert_eq!(binding.kind, BindingKind::ReservedVariable);
        assert_eq!(binding.owner_context.index(), 0);
        assert_eq!(binding.status, BindingStatus::Active);
        let proof = ast
            .node_views()
            .find(|node| node.kind() == &K::ProofBlock)
            .unwrap();
        assert_eq!(ast.node(parents[&proof.id()]).unwrap().kind, K::TheoremItem);
        for (id, context) in bindings.contexts().iter() {
            assert_eq!(context.visible_bindings, [binding_id]);
            if id.index() == 0 {
                assert_eq!(context.owner, BindingContextOwner::Module);
                assert!(context.parent.is_none() && context.lexical_scope.is_none());
                assert_eq!(context.bindings, [binding_id]);
            } else {
                assert_eq!(
                    context.owner,
                    BindingContextOwner::SourceStatement {
                        source_range: proof.range()
                    }
                );
                assert_eq!(context.parent.unwrap().index(), 0);
                assert!(context.bindings.is_empty());
                assert_eq!(context.lexical_scope.as_ref(), scopes.get(&proof.id()));
            }
        }
        for (((term_id, term), (_, reference)), occurrence) in primary
            .terms()
            .iter()
            .zip(primary.references().iter())
            .zip(scope.references())
        {
            let ordinal = occurrence.ordinal();
            assert_eq!(term.site().node().index(), occurrence.node().index());
            assert_eq!(
                (term.source_range(), term.source_ordinal(), term.spelling()),
                (occurrence.range(), ordinal, occurrence.spelling())
            );
            assert_eq!(term.context().index(), usize::from(ordinal >= 2));
            assert_eq!(reference.term(), term_id);
            assert_eq!(reference.binding(), binding_id);
            assert_eq!(reference.use_ordinal(), ordinal);
            assert_eq!(reference.lexical_scope(), Some(occurrence.scope()));
            assert_eq!(scopes.get(&occurrence.node()), Some(occurrence.scope()));
            let context = bindings.contexts().get(term.context()).unwrap();
            if ordinal < 2 {
                assert!(occurrence.scope().path().is_empty());
            } else {
                assert_eq!(context.lexical_scope.as_ref(), Some(occurrence.scope()));
            }
        }
        let mut equalities = ast
            .node_views()
            .filter(|node| node.kind() == &K::BuiltinPredicateApplication)
            .collect::<Vec<_>>();
        equalities.sort_by_key(|node| node.range().start);
        for ((id, formula), equality) in atomic.formulas().iter().zip(equalities) {
            assert_eq!(formula.site().node().index(), equality.id().index());
            assert_eq!(
                (
                    formula.source_range(),
                    formula.source_ordinal(),
                    formula.context().index()
                ),
                (equality.range(), id.index(), usize::from(id.index() > 0))
            );
            assert_eq!(formula.kind(), SourceAtomicFormulaKind::Equality);
            let edges = atomic
                .edges()
                .iter()
                .filter(|(_, edge)| edge.formula() == id)
                .collect::<Vec<_>>();
            assert_eq!(edges.len(), 2);
            for (operand, (_, edge)) in edges.into_iter().enumerate() {
                let occurrence = &scope.references()[2 * id.index() + operand];
                assert_eq!(
                    ast.node(equality.children()[2 * operand]).unwrap().children,
                    [occurrence.node()]
                );
                assert_eq!(edge.ordinal(), operand);
                assert_eq!(
                    edge.role(),
                    if operand == 0 {
                        SourceAtomicEdgeRole::BuiltinLeftOperand
                    } else {
                        SourceAtomicEdgeRole::BuiltinRightOperand
                    }
                );
                assert_eq!(
                    edge.target(),
                    SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(occurrence.ordinal()))
                );
            }
        }
        assert!(inference.facts().is_empty() && inference.diagnostics().is_empty());
        assert!(
            base.types().is_empty()
                && base.contexts().is_empty()
                && base.facts().is_empty()
                && base.initial_obligations().is_empty()
        );
        assert!(base.source_statement().is_none() && base.source_statement_references().is_none());
        assert_eq!(base.source_id(), source.source_id());
    }
    eprintln!(
        "B2 native bases ready: all six have 3 equalities, 6 genuine references, and 2 actual contexts"
    );
    for ((frontend, source, symbols, scope, labels, resolution, scopes, parents, before), result) in
        cases.into_iter().zip(bases)
    {
        let (bindings, _, base) = result.unwrap();
        // Stage 2 runtime RED starts after all six native bases above passed.
        let installed = base
            .clone()
            .with_source_statement_from_original(&source, &symbols, &scope);
        assert!(
            installed.is_ok(),
            "native assumption original installation: {installed:?}"
        );
        let installed = installed.unwrap();
        let statement = installed.source_statement().unwrap();
        assert!(installed.source_statement_references().is_none());
        assert_eq!(
            [
                statement.owners().len(),
                statement.statements().len(),
                statement.contexts().len(),
                statement.input_facts().len(),
                statement.candidate_facts().len()
            ],
            [1, 3, 3, 3, 3]
        );
        assert_eq!(statement.binding_env(), &bindings);
        let ast = frontend.ast.as_ref().unwrap();
        let mut sites = ast
            .node_views()
            .filter(|view| {
                matches!(
                    view.kind(),
                    K::TheoremItem | K::AssumptionStatement | K::ConclusionStatement
                )
            })
            .collect::<Vec<_>>();
        sites.sort_by_key(|view| view.range().start);
        assert_eq!(sites.len(), 3);
        let (_, owner) = statement.owners().iter().next().unwrap();
        assert_eq!(owner.site().node().index(), sites[0].id().index());
        assert_eq!(owner.source_range(), sites[0].range());
        assert_eq!(owner.spelling(), labels.projections()[0].primary_spelling());
        assert_eq!(owner.contribution(), labels.projections()[0].contribution());
        assert_eq!(owner.symbol(), statement.checked_owner().symbol());
        assert_eq!(
            (owner.role(), owner.status(), owner.recovery()),
            (
                SourceTheoremRole::Theorem,
                SourceTheoremStatus::Unmodified,
                SourceStatementRecovery::Normal
            )
        );
        for ((id, row), site) in statement.statements().iter().zip(sites) {
            let ordinal = id.index();
            assert_eq!(
                (
                    row.owner().index(),
                    row.context().index(),
                    row.source_ordinal()
                ),
                (0, ordinal, ordinal)
            );
            assert_eq!(row.site().node().index(), site.id().index());
            assert_eq!(row.source_range(), site.range());
            if ordinal > 0 {
                assert_eq!(ast.node(parents[&site.id()]).unwrap().kind, K::ProofBlock);
            }
            assert_eq!(
                row.kind(),
                [
                    SourceStatementKind::TheoremProposition,
                    SourceStatementKind::Assumption,
                    SourceStatementKind::Conclusion
                ][ordinal]
            );
            assert_eq!(row.recovery(), SourceStatementRecovery::Normal);
            let mut tokens = ast
                .node_views()
                .filter(|token| {
                    matches!(token.kind(), K::Token(_))
                        && site.range().start <= token.range().start
                        && token.range().end <= site.range().end
                })
                .collect::<Vec<_>>();
            tokens.sort_by_key(|token| token.range().start);
            assert_eq!(
                row.spelling(),
                tokens
                    .iter()
                    .map(|token| ast.node(token.id()).unwrap().token_text().unwrap())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            assert_eq!(
                row.formula(),
                SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(ordinal))
            );
            let context = statement.contexts().get(row.context()).unwrap();
            assert_eq!(
                (
                    context.statement(),
                    context.source_range(),
                    context.binding_context().index()
                ),
                (id, site.range(), usize::from(ordinal > 0))
            );
            assert_eq!(context.visible_bindings(), &[BindingId::new(0)]);
            let (_, guard) = statement.input_facts().iter().nth(ordinal).unwrap();
            assert!(
                guard.statement() == id
                    && guard.context() == row.context()
                    && guard.ordinal() == 0
                    && guard.kind() == SourceStatementInputFactKind::ReservedTypeGuard
                    && guard.binding() == BindingId::new(0),
                "guard {ordinal}"
            );
            assert_eq!(
                guard.uses(),
                [
                    SourcePrimaryTermReferenceId::new(2 * ordinal),
                    SourcePrimaryTermReferenceId::new(2 * ordinal + 1)
                ]
            );
            let (_, candidate) = statement.candidate_facts().iter().nth(ordinal).unwrap();
            assert!(
                candidate.statement() == id
                    && candidate.context() == row.context()
                    && candidate.ordinal() == 0
                    && candidate.kind() == SourceStatementCandidateFactKind::UnverifiedProposition
                    && candidate.formula() == row.formula(),
                "candidate {ordinal}"
            );
        }
        assert_eq!(
            base.clone()
                .with_source_statement_from_original(&source, &symbols, &scope),
            Ok(installed.clone())
        );
        let mut baseline = None;
        for typed in [&installed, &installed.clone()] {
            let resolved = assemble_empty_resolved_typed_ast(typed, Vec::new())?;
            assert_eq!(resolved.source_statement(), Some(statement));
            assert!(resolved.source_statement_references().is_none());
            assert_eq!(resolved.nodes().len(), base.nodes().len());
            assert!(
                resolved.expr_metadata().is_empty()
                    && resolved.checked_formulas().is_empty()
                    && resolved.statement_semantics().is_empty()
                    && resolved.checked_proofs().is_empty()
                    && resolved.checked_proof_nodes().is_empty()
                    && resolved.checked_terminal_goals().is_empty()
            );
            if let Some(previous) = &baseline {
                assert_eq!(&resolved, previous);
            }
            baseline = Some(resolved);
        }
        assert_eq!(
            (
                source,
                symbols,
                scope,
                labels,
                resolution,
                scopes,
                parents,
                ast.snapshot_text(),
                frontend.source
            ),
            before
        );
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_assumption_statement_original_capture_preserves_boundary() -> Result<(), String> {
    use mizar_checker::{
        cluster_trace::ClusterFactTable,
        overload_resolution::*,
        resolved_typed_ast::*,
        type_checker::{SourceVariableSemanticsChecker, TermFormulaChecker},
        typed_ast::*,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        env::{SymbolEnv, SymbolEnvIndexes},
        resolved_ast::ModuleId,
        symbols::{
            source_proof_inputs, source_theorem_module_original_inputs,
            validate_source_proof_occurrences,
        },
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SnapshotInput, SnapshotRegistry, SourceInput, SourceOriginInput,
        SourceVersion, ToolchainInfo, WorkspaceRoot, hash_text,
    };
    use mizar_syntax::SurfaceNodeKind as K;
    let text = "reserve x for set;\ntheorem FormulaStatementSingleAssumptionSmoke: x = x proof\n  assume x = x;\n  thus x = x;\nend;\n";
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src/b2.miz");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let mut captures = Vec::new();
    for current in [text.to_owned(), text.replace("Smoke", "Foreign"), "reserve x for set; theorem Nested: x = x proof A: x = x proof thus x = x; end; thus x = x by A; end;".to_owned()] {
            fs::write(&path, current).unwrap();
            let frontend = Frontend::new(FrontendSourceLoader::new(DiskSourceLoader::new(&root)), super::super::super::ParseOnlyImportProvider, MizarParserSeam).run(SourceUnitRequest { snapshot: snapshot_id(778), input: SourceInput { package_id: PackageId::new("assumption-statement-boundary"), module_path: ModulePath::new("b2"), normalized_path: mizar_session::normalize_path(&root, &path).unwrap(), edition: Edition::new("2026"), origin: SourceOriginInput::Disk { path: path.clone() } } }, &ids).unwrap();
            assert!(frontend.diagnostics.is_empty());
            captures.push(frontend);
        }
    let ast = captures[0].ast.as_ref().unwrap();
    let module = ModuleId::new(
        captures[0].source.package_id.clone(),
        captures[0].source.module_path.clone(),
    );
    let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
    let (scope, labels, resolution) = source_proof_inputs(&source, &symbols)?;
    validate_source_proof_occurrences(&source, &symbols, &scope)?;
    let (_, inference, base) =
        TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope)?;
    let before = (
        source.clone(),
        symbols.clone(),
        scope.clone(),
        labels.clone(),
        resolution.clone(),
        base.clone(),
    );
    let installed = base
        .clone()
        .with_source_statement_from_original(&source, &symbols, &scope)
        .unwrap();
    let statement = installed.source_statement().unwrap();
    assert!(installed.source_statement_references().is_none());
    let unsupported = "theorems.equality.unsupported_source_types";
    // Higher owners remain closed after the genuine native base and installation succeeded.
    assert_eq!(
        SourceVariableSemanticsChecker::source_theorem_binding_env(&source, &symbols, &scope)
            .unwrap_err(),
        unsupported
    );
    assert_eq!(
        TermFormulaChecker::check_source_theorem_equalities_with_typed_tables(
            &source, &symbols, &scope
        )
        .unwrap_err(),
        unsupported
    );
    assert_eq!(
        TermFormulaChecker::check_source_theorem_equalities_with_expression_metadata(
            &source, &symbols, &scope
        )
        .unwrap_err(),
        unsupported
    );
    let policy = mizar_proof::policy::VerifierPolicy::default();
    let unit = &captures[0].source;
    let registry = SnapshotRegistry::new();
    let (snapshot, lease) = registry
        .create_snapshot(
            ids.next_request_id().unwrap(),
            SnapshotInput {
                workspace_root: WorkspaceRoot::new(root.to_str().unwrap()),
                source_versions: vec![SourceVersion {
                    source_id: unit.source_id,
                    package_id: unit.package_id.clone(),
                    module_path: unit.module_path.clone(),
                    normalized_path: unit.normalized_path.clone(),
                    source_hash: unit.source_hash,
                    edition: unit.edition.clone(),
                    origin: unit.origin.clone(),
                }],
                dependency_artifacts: Vec::new(),
                lockfile_hash: hash_text(
                    &fs::read_to_string(
                        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock"),
                    )
                    .unwrap(),
                ),
                toolchain: ToolchainInfo::new("native-assumption-statement-test"),
                verifier_config_hash: policy.policy_fingerprint().hash(),
            },
        )
        .unwrap();
    let snapshot_before = snapshot.clone();
    assert_eq!(
        mizar_proof::status::source_theorem_reflexivity_witness_draft_from_original(
            &source, &symbols, &snapshot, &policy, ""
        )
        .unwrap_err(),
        unsupported
    );
    assert_eq!(snapshot, snapshot_before);
    assert!(!root.join("proof-witnesses").exists());
    assert_eq!(
        installed
            .clone()
            .with_source_statement_from_original(&source, &symbols, &scope),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let theorem = ast
        .node_views()
        .find(|view| view.kind() == &K::TheoremItem)
        .unwrap();
    // Exact historical bytes may not reenter the retired raw classifier, including root hybrids.
    for control in [
        "rooted",
        "rootless",
        "resolved-root-only",
        "typed-root-only",
        "changed",
        "enriched",
        "missing",
    ] {
        let mut nodes = base
            .nodes()
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        if control == "changed" {
            nodes[theorem.id().index()].children.reverse();
        }
        let mut reconstructed = TypedAst::try_new(TypedAstParts {
            source_id: base.source_id(),
            module_id: base.module_id().clone(),
            resolved_root: if ["rootless", "typed-root-only"].contains(&control) {
                None
            } else {
                base.resolved_root()
            },
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: TypedArena::try_new(
                if ["rootless", "resolved-root-only"].contains(&control) {
                    None
                } else {
                    base.nodes().root()
                },
                nodes,
            )
            .unwrap(),
            contexts: base.contexts().clone(),
            types: if control == "enriched" {
                inference.type_entries().clone()
            } else {
                base.types().clone()
            },
            facts: base.facts().clone(),
            coercions: base.coercions().clone(),
            initial_obligations: base.initial_obligations().clone(),
            diagnostics: base.diagnostics().clone(),
        })
        .unwrap();
        if control != "missing" {
            reconstructed = reconstructed
                .with_source_term(base.source_term().unwrap().clone())
                .unwrap()
                .with_source_atomic_formula(base.source_atomic_formula().unwrap().clone())
                .unwrap();
        }
        let result = reconstructed
            .clone()
            .with_source_statement_from_original(&source, &symbols, &scope);
        if control == "rooted" {
            assert_eq!(result, Ok(installed.clone()));
        } else {
            assert_eq!(
                result,
                Err(TypedAstError::InvalidSourceStatement),
                "{control}"
            );
        }
        assert_eq!(
            reconstructed.with_source_statement(statement.clone()),
            Err(TypedAstError::InvalidSourceStatement),
            "detached {control}"
        );
        assert_eq!(
            base.clone()
                .with_source_statement_from_original(&source, &symbols, &scope),
            Ok(installed.clone())
        );
    }
    let (foreign, foreign_env) =
        source_theorem_module_original_inputs(captures[1].ast.as_ref().unwrap(), &module)?.unwrap();
    let (foreign_scope, _, _) = source_proof_inputs(&foreign, &foreign_env)?;
    let empty = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
    for (current, env, occurrences) in [
        (&foreign, &symbols, &scope),
        (&source, &foreign_env, &scope),
        (&source, &symbols, &foreign_scope),
        (&source, &empty, &scope),
    ] {
        assert_eq!(
            base.clone()
                .with_source_statement_from_original(current, env, occurrences),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            base.clone()
                .with_source_statement_from_original(&source, &symbols, &scope),
            Ok(installed.clone())
        );
    }
    let (b1, b1_env) =
        source_theorem_module_original_inputs(captures[2].ast.as_ref().unwrap(), &module)?.unwrap();
    let (b1_scope, _, _) = source_proof_inputs(&b1, &b1_env)?;
    let b1_base = TermFormulaChecker::check_source_theorem_equalities(&b1, &b1_env, &b1_scope)?.2;
    let b1_installed = b1_base
        .clone()
        .with_source_statement_from_original(&b1, &b1_env, &b1_scope)
        .unwrap();
    assert_eq!(
        base.clone()
            .with_source_statement_from_original(&b1, &b1_env, &b1_scope),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        b1_base.with_source_statement_from_original(&source, &symbols, &scope),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone()
            .with_source_statement(b1_installed.source_statement().unwrap().clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert!(
        base.clone()
            .with_source_statement_references(
                statement.clone(),
                b1_installed.source_statement_references().unwrap().clone()
            )
            .is_err()
    );
    assert!(
        installed
            .clone()
            .with_source_statement_references(
                statement.clone(),
                b1_installed.source_statement_references().unwrap().clone()
            )
            .is_err()
    );
    let collection = OverloadCollectionOutput::collect([], []);
    let expansion = TemplateExpansionOutput::expand(&collection);
    let viability = CandidateViabilityOutput::filter(&expansion, []);
    let graphs = SpecificityGraphOutput::build(&viability, []);
    let selection = OverloadSelectionOutput::resolve(&graphs, []);
    let cluster_facts = ClusterFactTable::new();
    let mut final_baseline = None;
    for control in ["canonical", "metadata", "semantics", "proofs", "canonical"] {
        let replayed = base
            .clone()
            .with_source_statement_from_original(&source, &symbols, &scope)
            .unwrap();
        assert_eq!(replayed, installed);
        let (formula_id, formula) = inference.formulas().iter().next().unwrap();
        let owner = statement.checked_owner();
        let result = ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast: &replayed,
            cluster_facts: &cluster_facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &graphs,
            overload_selection: &selection,
            expressions: if control == "metadata" {
                vec![ExpressionMetadataInput {
                    expr: ExprId::new("extra"),
                    typed_site: formula.site.clone(),
                    local_context: None,
                    cluster_facts: Vec::new(),
                }]
            } else {
                Vec::new()
            },
            node_hints: Vec::new(),
            statement_semantics: (control == "semantics").then_some(StatementSemanticInputs {
                owner,
                binding_env: statement.binding_env(),
                term_formula: &inference,
                rows: vec![StatementSemanticInput {
                    owner: owner.symbol().clone(),
                    owner_node: TypedNodeId::new(theorem.id().index()),
                    formula: formula_id,
                    formula_node: formula.site.node(),
                }],
            }),
            statement_proofs: (control == "proofs").then_some(StatementProofInputs {
                owner,
                rows: vec![StatementProofIntentInput {
                    id: StatementProofIntentId::new(0),
                    source_order: 0,
                    statement: StatementSemanticId::new(0),
                    source_id: source.source_id(),
                    module_id: module.clone(),
                    owner: owner.symbol().clone(),
                    owner_node: TypedNodeId::new(theorem.id().index()),
                    owner_range: owner.source_range(),
                    owner_origin: owner.origin().clone(),
                    owner_visibility: owner.visibility(),
                    owner_export_status: owner.export_status(),
                    formula: formula_id,
                    formula_site: formula.site.clone(),
                    formula_node: formula.site.node(),
                    formula_range: formula.source_range,
                    recovery: NodeRecoveryState::Normal,
                    policy: TheoremPolicyIntent::Unmodified,
                    justification: TheoremJustificationIntent::Omitted,
                }],
            }),
        });
        if control == "canonical" {
            let resolved = result.map_err(|error| error.to_string())?;
            assert_eq!(resolved.source_statement(), Some(statement));
            assert!(resolved.source_statement_references().is_none());
            assert!(
                resolved.checked_formulas().is_empty()
                    && resolved.statement_semantics().is_empty()
                    && resolved.checked_proofs().is_empty()
            );
            if let Some(previous) = &final_baseline {
                assert_eq!(&resolved, previous);
            }
            final_baseline = Some(resolved);
        } else {
            assert_eq!(
                result,
                Err(ResolvedTypedAstError::InvalidSourceStatement),
                "final accepted {control}"
            );
        }
    }
    assert_eq!((source, symbols, scope, labels, resolution, base), before);
    assert_eq!(registry.get(snapshot.id), Some(snapshot.clone()));
    registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    fs::remove_dir_all(root).unwrap();
    Ok(())
}
