use super::*;

#[test]
fn source_witness_statement_original_capture_transports_native_pair() -> Result<(), String> {
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
    // EN04/15/16: these rows preserve source intent without accepting a witness or an existential proof.
    let original = "reserve x for set;\ntheorem FormulaStatementSingleWitnessSmoke: x = x proof\n  take x;\n  thus x = x;\nend;\n";
    let texts = [original.to_owned(), original.replace("FormulaStatementSingleWitnessSmoke", "RenamedWitness").replace("x", "parcel"), "reserve\n parcel\tfor :: type hint\n set;\ntheorem Layout: parcel = parcel proof :: syntax only\n take parcel; thus parcel = parcel; end;".to_owned()];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src/b3.miz");
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
                        package_id: PackageId::new("witness-statement-test"),
                        module_path: ModulePath::new("b3"),
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
                (49, 49, 48)
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
                (1, 5, 1, 0, 1, 0)
            );
            assert!(symbols.labels().is_empty());
            let projection = &labels.projections()[0];
            let owner = symbols
                .symbols()
                .iter()
                .find(|entry| entry.contribution() == projection.contribution())
                .unwrap();
            assert_eq!(owner.kind(), mizar_resolve::env::SymbolKind::Theorem);
            assert_eq!(owner.visibility(), mizar_resolve::env::Visibility::Public);
            assert_eq!(
                owner.export_status(),
                mizar_resolve::env::ExportStatus::Exported
            );
            assert_eq!(owner.primary_spelling(), projection.primary_spelling());
            assert_eq!(owner.origin(), projection.origin());
            let definition = symbols.definitions().by_symbol(owner.symbol()).unwrap();
            assert_eq!(
                definition.kind(),
                mizar_resolve::env::DefinitionKind::Theorem
            );
            assert_eq!(definition.contribution(), projection.contribution());
            let contribution = symbols
                .contributions()
                .get(projection.contribution())
                .unwrap();
            assert!(contribution.effects().symbols().contains(owner.symbol()));
            assert!(
                contribution
                    .effects()
                    .definitions()
                    .contains(&definition.id())
            );
            assert!(symbols.imports().is_empty());
            assert_eq!(
                projection.kind(),
                mizar_resolve::resolved_ast::LabelKind::Theorem
            );
            assert_eq!(projection.visibility(), owner.visibility());
            assert_eq!(projection.export_status(), owner.export_status());
            assert!(
                ast.node_views()
                    .all(|site| !matches!(site.kind(), K::Reference | K::CompactStatement))
            );

            assert!(!resolution.has_unresolved() && resolution.diagnostics().is_empty());
            assert_eq!(
                scope
                    .references()
                    .iter()
                    .map(|row| row.ordinal())
                    .collect::<Vec<_>>(),
                (0..5).collect::<Vec<_>>()
            );
            for (kind, count) in [
                (K::BuiltinPredicateApplication, 2),
                (K::ProofBlock, 1),
                (K::TakeStatement, 1),
                (K::Witness, 1),
                (K::CompactStatement, 0),
            ] {
                assert_eq!(
                    ast.node_views().filter(|node| node.kind() == &kind).count(),
                    count
                );
            }
            let proof = ast
                .node_views()
                .find(|site| site.kind() == &K::ProofBlock)
                .unwrap();
            let take = ast
                .node_views()
                .find(|site| site.kind() == &K::TakeStatement)
                .unwrap();
            assert_eq!(ast.node(parents[&proof.id()]).unwrap().kind, K::TheoremItem);
            assert_eq!(parents[&take.id()], proof.id());
            assert_eq!(scopes[&proof.id()].path(), [4]);
            for occurrence in scope.references() {
                assert_eq!(scopes.get(&occurrence.node()), Some(occurrence.scope()));
                assert_eq!(
                    occurrence.scope().path(),
                    if occurrence.range().start < proof.range().start {
                        &[][..]
                    } else {
                        scopes[&proof.id()].path()
                    }
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
                "B3 resolver ready: variant={variant} buffer={buffer} nodes=49 reserve=1 refs=0..5 equalities=2 proof=1 labels=1 citations=0"
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
        "native witness equality bases: {errors:?}"
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
            [2, 1, 5, 5, 2, 4]
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
                let operand_node = ast.node(equality.children()[2 * operand]).unwrap().children[0];
                let occurrence = scope
                    .references()
                    .iter()
                    .find(|row| row.node() == operand_node)
                    .unwrap();
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
        let take = ast
            .node_views()
            .find(|node| node.kind() == &K::TakeStatement)
            .unwrap();
        let witness = ast
            .node_views()
            .find(|node| node.kind() == &K::Witness)
            .unwrap();
        assert_eq!(parents[&take.id()], proof.id());
        assert_eq!(parents[&witness.id()], take.id());
        let operand = ast.node(witness.children()[0]).unwrap().children[0];
        let occurrence = scope
            .references()
            .iter()
            .find(|row| row.node() == operand)
            .unwrap();
        assert_eq!(occurrence.ordinal(), 2);
        assert_eq!(scopes.get(&operand), scopes.get(&proof.id()));
        assert_eq!(scopes[&proof.id()].path(), [4]);
        let (witness_term, term) = primary
            .terms()
            .iter()
            .find(|(_, term)| term.site().node().index() == operand.index())
            .unwrap();
        assert_eq!(term.context().index(), 1);
        assert!(
            atomic
                .edges()
                .iter()
                .all(|(_, edge)| edge.target() != SourceAtomicTermTarget::Primary(witness_term))
        );
        assert!(primary.numeric_type_requests().is_empty());
        assert_eq!(atomic.requests().len(), 4);
        let mut requested_edges = std::collections::BTreeSet::new();
        for (_, request) in atomic.requests().iter() {
            assert_eq!(request.kind(), SourceAtomicRequestKind::OperandExpectedType);
            let edge_id = request.edge().expect("equality operand request edge");
            assert!(requested_edges.insert(edge_id));
            let edge = atomic.edges().get(edge_id).unwrap();
            assert_eq!(request.formula(), edge.formula());
            assert_eq!(request.ordinal(), edge.ordinal());
            assert_ne!(edge.target(), SourceAtomicTermTarget::Primary(witness_term));
            assert!(
                request.candidate().is_none()
                    && request.type_site().is_none()
                    && request.attribute().is_none()
            );
        }
        assert_eq!(
            requested_edges,
            atomic.edges().iter().map(|(id, _)| id).collect()
        );
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
        "B3 native bases ready: all six have 2 equalities, 5 genuine references, and 2 actual contexts"
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
            "native witness original installation: {installed:?}"
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
            [1, 2, 2, 2, 2]
        );
        let witnesses = installed.source_statement_witnesses().unwrap();
        assert_eq!(witnesses.witnesses().len(), 1);
        assert!(witnesses.names().is_empty());
        let (_, witness) = witnesses.witnesses().iter().next().unwrap();
        let ast = frontend.ast.as_ref().unwrap();
        let take = ast
            .node_views()
            .find(|node| node.kind() == &K::TakeStatement)
            .unwrap();
        let witness_site = ast
            .node_views()
            .find(|node| node.kind() == &K::Witness)
            .unwrap();
        let operand = ast.node(witness_site.children()[0]).unwrap().children[0];
        let primary = base.source_term().unwrap();
        let (term, term_row) = primary
            .terms()
            .iter()
            .find(|(_, row)| row.site().node().index() == operand.index())
            .unwrap();
        assert_eq!(
            (
                witness.owner().index(),
                witness.binding_context().index(),
                witness.source_ordinal(),
                witness.ordinal()
            ),
            (0, 1, 1, 0)
        );
        assert_eq!(
            witness.term(),
            SourceStatementWitnessTermTarget::Primary(term)
        );
        assert_eq!(witness.take_site().node().index(), take.id().index());
        assert_eq!(witness.take_range(), take.range());
        assert_eq!(witness.site().node().index(), witness_site.id().index());
        assert_eq!(witness.source_range(), witness_site.range());
        assert_eq!(witness.spelling(), term_row.spelling());
        assert_eq!(witness.kind(), SourceStatementWitnessKind::Unnamed);
        assert_eq!(witness.recovery(), SourceStatementRecovery::Normal);
        assert!(witness.name().is_none());
        assert_eq!(statement.binding_env(), &bindings);
        let ast = frontend.ast.as_ref().unwrap();
        let mut sites = ast
            .node_views()
            .filter(|view| matches!(view.kind(), K::TheoremItem | K::ConclusionStatement))
            .collect::<Vec<_>>();
        sites.sort_by_key(|view| view.range().start);
        assert_eq!(sites.len(), 2);
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
                (0, ordinal, [0, 2][ordinal])
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
                    SourcePrimaryTermReferenceId::new([0, 3][ordinal]),
                    SourcePrimaryTermReferenceId::new([1, 4][ordinal])
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
            assert_eq!(resolved.source_statement_witnesses(), Some(witnesses));
            assert_eq!(typed.debug_text(), installed.debug_text());
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
fn source_witness_statement_original_capture_preserves_boundary() -> Result<(), String> {
    use super::super::super::type_elaboration::assemble_empty_resolved_typed_ast;
    use mizar_checker::{
        cluster_trace::ClusterFactTable,
        overload_resolution::*,
        resolved_typed_ast::*,
        source_statement::{
            SourceStatementWitnessError, SourceStatementWitnessHandoffInput,
            SourceStatementWitnessInput, SourceStatementWitnessProducer,
        },
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
    let text = "reserve x for set;\ntheorem FormulaStatementSingleWitnessSmoke: x = x proof\n  take x;\n  thus x = x;\nend;\n";
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src/b3.miz");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let mut captures = Vec::new();
    for current in [text.to_owned(), text.replace("SingleWitnessSmoke", "NamedWitnessSmoke").replace("take x;", "take     x;"), "reserve x for set; theorem Nested: x = x proof A: x = x proof thus x = x; end; thus x = x by A; end;".to_owned()] {
            fs::write(&path, current).unwrap();
            let frontend = Frontend::new(FrontendSourceLoader::new(DiskSourceLoader::new(&root)), super::super::super::ParseOnlyImportProvider, MizarParserSeam).run(SourceUnitRequest { snapshot: snapshot_id(778), input: SourceInput { package_id: PackageId::new("witness-statement-boundary"), module_path: ModulePath::new("b3"), normalized_path: mizar_session::normalize_path(&root, &path).unwrap(), edition: Edition::new("2026"), origin: SourceOriginInput::Disk { path: path.clone() } } }, &ids).unwrap();
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
    let witnesses = installed.source_statement_witnesses().unwrap();
    assert_eq!(witnesses.witnesses().len(), 1);
    assert!(witnesses.names().is_empty());
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
                toolchain: ToolchainInfo::new("native-witness-statement-test"),
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
    // Public reconstruction must genuinely succeed before any mutant is attempted.
    let reconstruct = |nodes, resolved_root, typed_root, attach: bool, enriched: bool| {
        let raw = TypedAst::try_new(TypedAstParts {
            source_id: base.source_id(),
            module_id: base.module_id().clone(),
            resolved_root,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: TypedArena::try_new(typed_root, nodes)
                .map_err(|error| ("arena", TypedAstError::Arena(error)))?,
            contexts: base.contexts().clone(),
            types: if enriched {
                inference.type_entries().clone()
            } else {
                base.types().clone()
            },
            facts: base.facts().clone(),
            coercions: base.coercions().clone(),
            initial_obligations: base.initial_obligations().clone(),
            diagnostics: base.diagnostics().clone(),
        })
        .map_err(|error| ("parts", error))?;
        if !attach {
            return Ok(raw);
        }
        raw.with_source_term(base.source_term().unwrap().clone())
            .map_err(|error| ("primary", error))?
            .with_source_atomic_formula(base.source_atomic_formula().unwrap().clone())
            .map_err(|error| ("atomic", error))
    };
    let nodes = base
        .nodes()
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    let reconstructed = reconstruct(
        nodes.clone(),
        base.resolved_root(),
        base.nodes().root(),
        true,
        false,
    )
    .expect("unmutated native public reconstruction must succeed");
    assert_eq!(reconstructed, base);
    assert_eq!(
        reconstructed.with_source_statement_from_original(&source, &symbols, &scope),
        Ok(installed.clone())
    );
    for control in [
        "rooted",
        "rootless",
        "resolved-root-only",
        "typed-root-only",
        "changed",
        "enriched",
        "missing",
        "missing-atomic",
    ] {
        let mut current = nodes.clone();
        if control == "changed" {
            current[theorem.id().index()].children.reverse();
        }
        let reconstructed = reconstruct(
            current,
            if ["rootless", "typed-root-only"].contains(&control) {
                None
            } else {
                base.resolved_root()
            },
            if ["rootless", "resolved-root-only"].contains(&control) {
                None
            } else {
                base.nodes().root()
            },
            !["missing", "missing-atomic"].contains(&control),
            control == "enriched",
        )
        .unwrap();
        let reconstructed = if control == "missing-atomic" {
            reconstructed
                .with_source_term(base.source_term().unwrap().clone())
                .unwrap()
        } else {
            reconstructed
        };
        let result = reconstructed
            .clone()
            .with_source_statement_from_original(&source, &symbols, &scope);
        assert_eq!(
            result,
            if control == "rooted" {
                Ok(installed.clone())
            } else {
                Err(TypedAstError::InvalidSourceStatement)
            },
            "{control}"
        );
        assert_eq!(
            reconstructed
                .clone()
                .with_source_statement(statement.clone()),
            Err(TypedAstError::InvalidSourceStatement),
            "missing companion {control}"
        );
        assert_eq!(
            reconstructed.with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement),
            "detached pair {control}"
        );
        assert_eq!(
            base.clone()
                .with_source_statement_from_original(&source, &symbols, &scope),
            Ok(installed.clone())
        );
    }
    assert_eq!(nodes.len(), 49);
    let unowned_tokens = ast
        .node_views()
        .filter(|site| {
            matches!(site.kind(), K::Token(_))
                && !base
                    .source_term()
                    .unwrap()
                    .terms()
                    .iter()
                    .any(|(_, term)| term.site().node().index() == site.id().index())
                && !base
                    .source_atomic_formula()
                    .unwrap()
                    .formulas()
                    .iter()
                    .any(|(_, formula)| formula.site().node().index() == site.id().index())
        })
        .map(|site| site.id().index())
        .collect::<std::collections::BTreeSet<_>>();
    let mut stages = std::collections::BTreeMap::new();
    let mut canonical_tokens = 0;
    for node in 0..nodes.len() {
        for mutation in ["anchor-end", "recovered", "degraded", "kind", "children"] {
            let mut current = nodes.clone();
            match mutation {
                "anchor-end" => {
                    let mizar_session::SourceAnchor::Range(mut range) =
                        current[node].anchor.clone()
                    else {
                        panic!("native source range")
                    };
                    range.end += 1;
                    current[node].anchor = mizar_session::SourceAnchor::Range(range);
                }
                "recovered" => current[node].recovery = NodeRecoveryState::Recovered,
                "degraded" => current[node].recovery = NodeRecoveryState::Degraded,
                "kind" => current[node].kind = TypedNodeKind::new("mutated.source.node"),
                "children" => {
                    if current[node].children.len() > 1 {
                        current[node].children.swap(0, 1);
                    } else if current[node].children.len() == 1 {
                        current[node].children.clear();
                    } else {
                        current[node]
                            .children
                            .push(TypedNodeId::new(usize::from(node == 0)));
                    }
                }
                _ => unreachable!(),
            }
            let attempt = reconstruct(
                current,
                base.resolved_root(),
                base.nodes().root(),
                true,
                false,
            );
            let stage = match attempt {
                Ok(rebuilt) => {
                    assert_eq!(
                        rebuilt.with_source_statement_from_original(&source, &symbols, &scope),
                        Err(TypedAstError::InvalidSourceStatement),
                        "node={node} mutation={mutation}"
                    );
                    if unowned_tokens.contains(&node) {
                        canonical_tokens += 1;
                    }
                    "canonical"
                }
                Err((stage, error)) => {
                    match stage {
                        "primary" => assert_eq!(error, TypedAstError::InvalidSourceTerm),
                        "atomic" => {
                            assert_eq!(error, TypedAstError::InvalidSourceAtomicFormula)
                        }
                        _ => panic!(
                            "unexpected reconstruction stage node={node} mutation={mutation}: {stage} {error:?}"
                        ),
                    }
                    stage
                }
            };
            if unowned_tokens.contains(&node) {
                assert_eq!(
                    stage, "canonical",
                    "unowned token node={node} mutation={mutation}"
                );
            }
            *stages.entry(stage).or_insert(0usize) += 1;
            eprintln!("B3 native arena node={node} mutation={mutation} refusal={stage}");
            assert_eq!(
                base.clone()
                    .with_source_statement_from_original(&source, &symbols, &scope),
                Ok(installed.clone())
            );
        }
    }
    assert_eq!(stages.values().sum::<usize>(), 245);
    assert!(
        canonical_tokens > 0,
        "unowned token mutants must reach whole-base comparison"
    );
    let (foreign, foreign_env) =
        source_theorem_module_original_inputs(captures[1].ast.as_ref().unwrap(), &module)?.unwrap();
    let (foreign_scope, _, _) = source_proof_inputs(&foreign, &foreign_env)?;
    let foreign_base = TermFormulaChecker::check_source_theorem_equalities(
        &foreign,
        &foreign_env,
        &foreign_scope,
    )?
    .2;
    let foreign_installed = foreign_base
        .clone()
        .with_source_statement_from_original(&foreign, &foreign_env, &foreign_scope)
        .unwrap();
    // Coherent source layout collides with the remaining legacy B3N owner summary.
    let foreign_statement = foreign_installed.source_statement().unwrap();
    let foreign_witnesses = foreign_installed.source_statement_witnesses().unwrap();
    assert_eq!(
        foreign_statement
            .owners()
            .iter()
            .next()
            .unwrap()
            .1
            .spelling(),
        "FormulaStatementNamedWitnessSmoke"
    );
    assert_eq!(
        (
            foreign_statement
                .owners()
                .iter()
                .next()
                .unwrap()
                .1
                .source_range()
                .start,
            foreign_statement
                .owners()
                .iter()
                .next()
                .unwrap()
                .1
                .source_range()
                .end
        ),
        (19, 106)
    );
    assert_eq!(
        assemble_empty_resolved_typed_ast(&foreign_installed, Vec::new())?
            .source_statement_witnesses(),
        Some(foreign_witnesses)
    );
    let (_, row) = foreign_witnesses.witnesses().iter().next().unwrap();
    let raw_input = SourceStatementWitnessHandoffInput {
        source_id: foreign_base.source_id(),
        module_id: foreign_base.module_id().clone(),
        witnesses: vec![SourceStatementWitnessInput {
            owner: row.owner(),
            binding_context: row.binding_context(),
            term: row.term(),
            take_site: row.take_site().clone(),
            take_range: row.take_range(),
            site: row.site().clone(),
            source_range: row.source_range(),
            source_ordinal: row.source_ordinal(),
            ordinal: row.ordinal(),
            spelling: row.spelling().to_owned(),
            kind: row.kind(),
            recovery: row.recovery(),
            name: row.name(),
        }],
        names: Vec::new(),
    };
    for rooted in [true, false] {
        let arena = TypedArena::try_new(
            if rooted {
                foreign_base.nodes().root()
            } else {
                None
            },
            foreign_base
                .nodes()
                .iter()
                .map(|(_, node)| node.clone())
                .collect(),
        )
        .unwrap();
        assert_eq!(
            SourceStatementWitnessProducer::build(
                raw_input.clone(),
                foreign_statement,
                foreign_base.source_term().unwrap(),
                &arena
            ),
            Err(SourceStatementWitnessError::DependencyMismatch)
        );
        let detached = TypedAst::try_new(TypedAstParts {
            source_id: foreign_base.source_id(),
            module_id: foreign_base.module_id().clone(),
            resolved_root: if rooted {
                foreign_base.resolved_root()
            } else {
                None
            },
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: arena,
            contexts: foreign_base.contexts().clone(),
            types: foreign_base.types().clone(),
            facts: foreign_base.facts().clone(),
            coercions: foreign_base.coercions().clone(),
            initial_obligations: foreign_base.initial_obligations().clone(),
            diagnostics: foreign_base.diagnostics().clone(),
        })
        .unwrap()
        .with_source_term(foreign_base.source_term().unwrap().clone())
        .unwrap()
        .with_source_atomic_formula(foreign_base.source_atomic_formula().unwrap().clone())
        .unwrap();
        assert_eq!(
            detached.with_source_statement_witnesses(
                foreign_statement.clone(),
                foreign_witnesses.clone()
            ),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            foreign_base.clone().with_source_statement_from_original(
                &foreign,
                &foreign_env,
                &foreign_scope
            ),
            Ok(foreign_installed.clone())
        );
    }
    for (statements, companion) in [
        (
            statement.clone(),
            foreign_installed
                .source_statement_witnesses()
                .unwrap()
                .clone(),
        ),
        (
            foreign_installed.source_statement().unwrap().clone(),
            witnesses.clone(),
        ),
        (
            foreign_installed.source_statement().unwrap().clone(),
            foreign_installed
                .source_statement_witnesses()
                .unwrap()
                .clone(),
        ),
    ] {
        assert_eq!(
            base.clone()
                .with_source_statement_witnesses(statements.clone(), companion.clone()),
            Err(TypedAstError::InvalidSourceStatement)
        );
        assert_eq!(
            installed
                .clone()
                .with_source_statement_witnesses(statements, companion),
            Err(TypedAstError::InvalidSourceStatement)
        );
    }
    let empty = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
    for (current, env, occurrences) in [
        (&foreign, &symbols, &scope),
        (&source, &foreign_env, &scope),
        (&source, &symbols, &foreign_scope),
        (&source, &empty, &scope),
        (&foreign, &foreign_env, &foreign_scope),
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
    for typed in [&base, &installed] {
        assert_eq!(
            typed.clone().with_source_statement_references(
                statement.clone(),
                b1_installed.source_statement_references().unwrap().clone()
            ),
            Err(TypedAstError::InvalidSourceStatement)
        );
    }
    assert_eq!(
        installed.clone().with_source_statement(statement.clone()),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        installed
            .clone()
            .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
        Err(TypedAstError::InvalidSourceStatement)
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
            assert_eq!(resolved.source_statement_witnesses(), Some(witnesses));
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
