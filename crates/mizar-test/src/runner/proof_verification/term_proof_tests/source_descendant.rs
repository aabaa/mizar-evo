use super::*;

#[test]
fn source_descendant_statement_original_capture_transports_native_pair() -> Result<(), String> {
    use mizar_checker::{
        binding_env::*, cluster_trace::ClusterFactTable, overload_resolution::*,
        resolved_typed_ast::*, source_atomic_formula::*, source_statement::*, source_term::*,
        type_checker::TermFormulaChecker, typed_ast::*,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        labels::{LabelProjectionSource, LabelReferenceScope},
        resolved_ast::{LabelResolution, ModuleId, NodeReferenceKey, NodeResolutionState},
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
    let original = concat!(
        "reserve x for set;\n",
        "theorem FormulaStatementAncestorLabelDescendantCitationSmoke: x = x proof\n",
        "  A: x = x proof\n",
        "    thus x = x;\n",
        "  end;\n",
        "  thus x = x proof\n",
        "    thus x = x by A;\n",
        "  end;\n",
        "end;\n"
    );
    let texts = [original.to_owned(), original.replace("FormulaStatementAncestorLabelDescendantCitationSmoke", "RenamedDescendant").replace("x", "parcel").replace("A:", "Evidence:").replace("by A", "by Evidence"), "reserve\n parcel\tfor :: default\n set;\ntheorem Layout: parcel = parcel proof\n Evidence: parcel = parcel proof :: nested\n thus parcel = parcel; end;\n thus parcel = parcel proof\n thus parcel = parcel by Evidence; end; end;".to_owned()];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src/b5a.miz");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let mut cases = Vec::new();
    for text in texts {
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
                        package_id: PackageId::new("descendant-statement-test"),
                        module_path: ModulePath::new("b5a"),
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
            let (scopes, parents) = source_proof_scope_maps(&source, &symbols, &scope)?;
            source.validate_against(ast, &module).unwrap();
            assert_eq!(
                validate_source_proof_occurrences(&source, &symbols, &scope),
                Ok(())
            );
            assert_eq!(
                (
                    source.arena().len(),
                    scopes.len(),
                    parents.len(),
                    scope.bindings().len(),
                    scope.references().len()
                ),
                (93, 93, 92, 1, 10)
            );
            assert_eq!(
                (
                    labels.projections().len(),
                    labels.references().len(),
                    resolution.index().len(),
                    resolution.ids().len()
                ),
                (2, 1, 2, 1)
            );
            assert!(!resolution.has_unresolved() && resolution.diagnostics().is_empty());
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
            cases.push((
                frontend, source, symbols, scope, labels, resolution, scopes, parents, before,
            ));
        }
    }
    assert_eq!(cases.len(), 6);
    let bases = cases
        .iter()
        .map(|(_, source, symbols, scope, ..)| {
            TermFormulaChecker::check_source_theorem_equalities(source, symbols, scope)
        })
        .collect::<Vec<_>>();
    // Stage 1 requires all genuine resolver batches above before future native-base success.
    assert!(
        bases.iter().all(Result::is_ok),
        "native descendant bases: {:?}",
        bases
            .iter()
            .filter_map(|result| result.as_ref().err())
            .collect::<Vec<_>>()
    );
    for ((frontend, _, _, scope, _, _, scopes, parents, _), result) in cases.iter().zip(&bases) {
        let (bindings, inference, base) = result.as_ref().unwrap();
        let ast = frontend.ast.as_ref().unwrap();
        let primary = base.source_term().unwrap();
        let atomic = base.source_atomic_formula().unwrap();
        assert_eq!(
            (
                bindings.contexts().len(),
                bindings.bindings().len(),
                primary.terms().len(),
                primary.references().len(),
                atomic.formulas().len(),
                atomic.edges().len()
            ),
            (4, 1, 10, 10, 5, 10)
        );
        let (binding_id, binding) = bindings.bindings().iter().next().unwrap();
        assert_eq!(
            (
                binding_id.index(),
                binding.owner_context.index(),
                binding.kind,
                binding.status
            ),
            (0, 0, BindingKind::ReservedVariable, BindingStatus::Active)
        );
        assert_eq!(binding.type_site, BindingTypeSite::Missing);
        assert_eq!(
            binding.identity,
            BinderIdentity::ReservedVariable {
                spelling: scope.bindings()[0].spelling().to_owned(),
                declaration_range: scope.bindings()[0].range()
            }
        );
        assert!(binding.captured.identities().is_empty() && binding.diagnostics.is_empty());
        let mut proofs = ast
            .node_views()
            .filter(|view| matches!(view.kind(), K::ProofBlock))
            .collect::<Vec<_>>();
        proofs.sort_by_key(|view| view.range().start);
        assert_eq!(proofs.len(), 3);
        for (id, context) in bindings.contexts().iter() {
            let index = id.index();
            assert_eq!(
                context.parent.map(|parent| parent.index()),
                [None, Some(0), Some(1), Some(1)][index]
            );
            assert_eq!(context.visible_bindings, [binding_id]);
            if index == 0 {
                assert_eq!(context.owner, BindingContextOwner::Module);
                assert_eq!(context.bindings, [binding_id]);
                assert!(context.lexical_scope.is_none());
            } else {
                let proof = &proofs[index - 1];
                assert_eq!(
                    context.owner,
                    BindingContextOwner::SourceStatement {
                        source_range: proof.range()
                    }
                );
                assert_eq!(context.lexical_scope.as_ref(), scopes.get(&proof.id()));
                assert_eq!(
                    context.lexical_scope.as_ref().unwrap().path(),
                    [&[4][..], &[4, 1][..], &[4, 2][..]][index - 1]
                );
                assert!(context.bindings.is_empty());
                assert_eq!(
                    ast.node(parents[&proof.id()]).unwrap().kind,
                    [K::TheoremItem, K::CompactStatement, K::ConclusionStatement][index - 1]
                );
            }
        }
        for (((term_id, term), (reference_id, reference)), (ordinal, occurrence)) in primary
            .terms()
            .iter()
            .zip(primary.references().iter())
            .zip(scope.references().iter().enumerate())
        {
            assert_eq!(
                (
                    term_id.index(),
                    reference_id.index(),
                    term.source_ordinal(),
                    reference.use_ordinal(),
                    occurrence.ordinal()
                ),
                (ordinal, ordinal, ordinal, ordinal, ordinal)
            );
            assert_eq!(term.site().node().index(), occurrence.node().index());
            assert_eq!(
                (term.source_range(), term.spelling()),
                (occurrence.range(), occurrence.spelling())
            );
            assert_eq!(
                term.context().index(),
                [0, 0, 1, 1, 2, 2, 1, 1, 3, 3][ordinal]
            );
            assert_eq!(
                (
                    reference.term(),
                    reference.binding(),
                    reference.lexical_scope()
                ),
                (term_id, binding_id, Some(occurrence.scope()))
            );
        }
        let mut equalities = ast
            .node_views()
            .filter(|view| matches!(view.kind(), K::BuiltinPredicateApplication))
            .collect::<Vec<_>>();
        equalities.sort_by_key(|view| view.range().start);
        assert_eq!(equalities.len(), 5);
        for ((id, formula), original) in atomic.formulas().iter().zip(equalities) {
            let ordinal = id.index();
            assert_eq!(formula.site().node().index(), original.id().index());
            assert_eq!(
                (
                    formula.source_range(),
                    formula.source_ordinal(),
                    formula.context().index(),
                    formula.kind(),
                    formula.recovery()
                ),
                (
                    original.range(),
                    ordinal,
                    [0, 1, 2, 1, 3][ordinal],
                    SourceAtomicFormulaKind::Equality,
                    SourceAtomicFormulaRecovery::Normal
                )
            );
            let mut tokens = ast
                .node_views()
                .filter(|token| {
                    matches!(token.kind(), K::Token(_))
                        && original.range().start <= token.range().start
                        && token.range().end <= original.range().end
                })
                .collect::<Vec<_>>();
            tokens.sort_by_key(|token| token.range().start);
            assert_eq!(
                formula.spelling(),
                tokens
                    .iter()
                    .map(|token| ast.node(token.id()).unwrap().token_text().unwrap())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            let edges = atomic
                .edges()
                .iter()
                .filter(|(_, edge)| edge.formula() == id)
                .collect::<Vec<_>>();
            assert_eq!(edges.len(), 2);
            for (operand, (_, edge)) in edges.into_iter().enumerate() {
                let occurrence = &scope.references()[2 * ordinal + operand];
                assert_eq!(
                    ast.node(original.children()[2 * operand]).unwrap().children,
                    [occurrence.node()]
                );
                assert_eq!(
                    (edge.ordinal(), edge.role(), edge.target()),
                    (
                        operand,
                        if operand == 0 {
                            SourceAtomicEdgeRole::BuiltinLeftOperand
                        } else {
                            SourceAtomicEdgeRole::BuiltinRightOperand
                        },
                        SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                            2 * ordinal + operand
                        ))
                    )
                );
            }
        }
        assert!(
            inference.facts().is_empty()
                && base.types().is_empty()
                && base.facts().is_empty()
                && base.initial_obligations().is_empty()
        );
    }
    let collection = OverloadCollectionOutput::collect([], []);
    let expansion = TemplateExpansionOutput::expand(&collection);
    let viability = CandidateViabilityOutput::filter(&expansion, []);
    let graphs = SpecificityGraphOutput::build(&viability, []);
    let selection = OverloadSelectionOutput::resolve(&graphs, []);
    let cluster_facts = ClusterFactTable::new();
    for (
        case,
        ((frontend, source, symbols, scope, labels, resolution, scopes, parents, before), result),
    ) in cases.into_iter().zip(bases).enumerate()
    {
        let (bindings, _, base) = result.unwrap();
        // Stage 2 starts only after every genuine 4-context/10-reference/5-equality base passed.
        let installed = base
            .clone()
            .with_source_statement_from_original(&source, &symbols, &scope);
        assert!(installed.is_ok(), "native descendant pair: {installed:?}");
        let installed = installed.unwrap();
        let statements = installed.source_statement().expect("statement half");
        let references = installed
            .source_statement_references()
            .expect("reference half");
        assert_eq!(statements.binding_env(), &bindings);
        assert_eq!(
            (
                statements.owners().len(),
                statements.statements().len(),
                statements.contexts().len(),
                statements.input_facts().len(),
                statements.candidate_facts().len()
            ),
            (1, 5, 5, 5, 5)
        );
        let ast = frontend.ast.as_ref().unwrap();
        let mut sites = ast
            .node_views()
            .filter(|view| {
                matches!(
                    view.kind(),
                    K::TheoremItem | K::CompactStatement | K::ConclusionStatement
                )
            })
            .collect::<Vec<_>>();
        sites.sort_by_key(|view| view.range().start);
        assert_eq!(sites.len(), 5);
        for ((id, row), site) in statements.statements().iter().zip(&sites) {
            let ordinal = id.index();
            assert_eq!(
                (
                    row.owner().index(),
                    row.context().index(),
                    row.source_ordinal(),
                    row.site().node().index(),
                    row.source_range()
                ),
                (0, ordinal, ordinal, site.id().index(), site.range())
            );
            assert_eq!(
                row.kind(),
                if ordinal == 0 {
                    SourceStatementKind::TheoremProposition
                } else if ordinal == 1 {
                    SourceStatementKind::ProofStepProposition
                } else {
                    SourceStatementKind::Conclusion
                }
            );
            assert_eq!(row.recovery(), SourceStatementRecovery::Normal);
            assert_eq!(
                row.formula(),
                SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(ordinal))
            );
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
            let context = statements.contexts().get(row.context()).unwrap();
            assert_eq!(
                (
                    context.statement(),
                    context.binding_context().index(),
                    context.source_range(),
                    context.visible_bindings()
                ),
                (
                    id,
                    [0, 1, 2, 1, 3][ordinal],
                    site.range(),
                    &[BindingId::new(0)][..]
                )
            );
            let (_, guard) = statements.input_facts().iter().nth(ordinal).unwrap();
            assert_eq!(
                (
                    guard.statement(),
                    guard.context(),
                    guard.ordinal(),
                    guard.kind(),
                    guard.binding()
                ),
                (
                    id,
                    row.context(),
                    0,
                    SourceStatementInputFactKind::ReservedTypeGuard,
                    BindingId::new(0)
                )
            );
            assert_eq!(
                guard.uses(),
                [
                    SourcePrimaryTermReferenceId::new(2 * ordinal),
                    SourcePrimaryTermReferenceId::new(2 * ordinal + 1)
                ]
            );
            let (_, candidate) = statements.candidate_facts().iter().nth(ordinal).unwrap();
            assert_eq!(
                (
                    candidate.statement(),
                    candidate.context(),
                    candidate.ordinal(),
                    candidate.kind(),
                    candidate.formula()
                ),
                (
                    id,
                    row.context(),
                    0,
                    SourceStatementCandidateFactKind::UnverifiedProposition,
                    row.formula()
                )
            );
        }
        assert_eq!(references.label_resolution(), &resolution);
        assert_eq!(references.resolver_ast().label_refs(), resolution.table());
        assert_eq!(references.reference_candidate(), &labels.references()[0]);
        let projection = labels
            .projections()
            .iter()
            .find(|projection| {
                matches!(
                    projection.source(),
                    LabelProjectionSource::CurrentModule {
                        proof_scope: Some(_),
                        ..
                    }
                )
            })
            .unwrap();
        assert_eq!(references.label_projection(), projection);
        assert!(
            matches!(projection.source(), LabelProjectionSource::CurrentModule { proof_scope: Some(scope), visible_after_ordinal: 2 } if scope.path() == [0])
        );
        assert!(labels.projections().iter().any(|projection| matches!(
            projection.source(),
            LabelProjectionSource::CurrentModule {
                proof_scope: None,
                visible_after_ordinal: 4
            }
        )));
        let (label_id, label) = references.labels().iter().next().unwrap();
        let (_, citation) = references.citations().iter().next().unwrap();
        assert_eq!(
            (
                references.labels().len(),
                references.citations().len(),
                label.statement().index(),
                label.source_ordinal(),
                label.visible_after_ordinal(),
                citation.statement().index(),
                citation.context().index(),
                citation.ordinal(),
                references.reference_candidate().ordinal()
            ),
            (1, 1, 1, 0, 2, 4, 4, 0, 4)
        );
        assert_eq!(label.proof_scope().path(), [0]);
        let LabelReferenceScope::Unqualified {
            proof_scope: Some(use_scope),
        } = labels.references()[0].scope()
        else {
            panic!("descendant citation scope")
        };
        assert_eq!(citation.proof_scope(), use_scope);
        assert_eq!(citation.proof_scope().path(), [0, 1]);
        assert_ne!(citation.proof_scope(), label.proof_scope());
        assert_eq!(
            citation.target(),
            SourceStatementCitationTarget::Local(label_id)
        );
        assert_eq!(
            citation.source_range(),
            labels.references()[0].site().range()
        );
        assert!(
            sites[4].range().start <= citation.source_range().start
                && citation.source_range().end <= sites[4].range().end
        );
        let resolved_ref = resolution.table().get(citation.label_ref()).unwrap();
        assert!(
            matches!(resolved_ref.resolution(), LabelResolution::Resolved(winner) if winner.origin() == projection.origin_path())
        );
        let snapshot = references.resolver_ast();
        assert_eq!(
            (snapshot.nodes().root(), snapshot.nodes().len()),
            (source.arena().root(), source.arena().len())
        );
        assert!(
            snapshot.name_refs().is_empty()
                && snapshot.imports().imports().next().is_none()
                && snapshot.imports().exports().next().is_none()
        );
        let citation_node = labels.references()[0].site().node();
        let label_node = ast
            .node_views()
            .find(|view| {
                view.range() == projection.declaration_range() && matches!(view.kind(), K::Token(_))
            })
            .unwrap();
        assert_ne!(
            projection.origin().structural_path(),
            source
                .arena()
                .node(source.resolved_node_for(label_node.id()).unwrap())
                .unwrap()
                .origin()
                .structural_path()
        );
        assert_ne!(
            labels.references()[0].origin().structural_path(),
            source
                .arena()
                .node(citation_node)
                .unwrap()
                .origin()
                .structural_path()
        );
        for (id, original) in source.arena().iter() {
            let expected = if id == citation_node {
                original
                    .clone()
                    .with_reference_key(NodeReferenceKey::Label(citation.label_ref()))
                    .with_resolution(NodeResolutionState::Resolved)
            } else {
                original.clone()
            };
            assert_eq!(snapshot.nodes().node(id), Some(&expected));
            let typed = installed
                .nodes()
                .node(TypedNodeId::new(id.index()))
                .unwrap();
            let expected_kind = match original.kind() {
                K::TermReference => "source.term.variable-reference".to_owned(),
                K::BuiltinPredicateApplication => "source.formula.atomic.equality".to_owned(),
                kind => format!("{kind:?}"),
            };
            assert_eq!(typed.kind.as_str(), expected_kind);
            assert_eq!(
                (typed.resolved_node, &typed.anchor, typed.recovery),
                (
                    Some(id),
                    original.origin().anchor(),
                    NodeRecoveryState::Normal
                )
            );
            assert!(
                typed
                    .children
                    .iter()
                    .map(|child| child.index())
                    .eq(original.children().iter().map(|child| child.index()))
            );
        }
        let clone = installed.clone();
        let mut final_baseline = None;
        for typed in std::iter::once(&installed).chain((case == 0).then_some(&clone)) {
            let resolved = ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
                typed_ast: typed,
                cluster_facts: &cluster_facts,
                overload_collection: &collection,
                template_expansion: &expansion,
                viability: &viability,
                specificity: &graphs,
                overload_selection: &selection,
                expressions: Vec::new(),
                node_hints: Vec::new(),
                statement_semantics: None,
                statement_proofs: None,
            })
            .map_err(|error| error.to_string())?;
            assert_eq!(resolved.source_statement(), Some(statements));
            assert_eq!(resolved.source_statement_references(), Some(references));
            assert!(
                resolved.checked_formulas().is_empty()
                    && resolved.statement_semantics().is_empty()
                    && resolved.checked_proofs().is_empty()
            );
            if let Some(previous) = &final_baseline {
                assert_eq!(&resolved, previous);
            }
            final_baseline = Some(resolved);
        }
        assert_eq!(
            installed,
            base.with_source_statement_from_original(&source, &symbols, &scope)
                .unwrap()
        );
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
fn source_descendant_statement_original_capture_preserves_boundary() -> Result<(), String> {
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
        resolved_ast::{LabelResolution, ModuleId},
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
    let text = concat!(
        "reserve x for set;\n",
        "theorem FormulaStatementAncestorLabelDescendantCitationSmoke: x = x proof\n",
        "  A: x = x proof\n",
        "    thus x = x;\n",
        "  end;\n",
        "  thus x = x proof\n",
        "    thus x = x by A;\n",
        "  end;\n",
        "end;\n"
    );
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src/b5a.miz");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let mut captures = Vec::new();
    for current in [text.to_owned(), text.replace("Smoke", "Foreign"), text.replace("by A", "by Missing"), "reserve x for set; theorem Nested: x = x proof A: x = x proof thus x = x; end; thus x = x by A; end;".to_owned()] {
        fs::write(&path, &current).unwrap();
        let frontend = Frontend::new(FrontendSourceLoader::new(DiskSourceLoader::new(&root)), super::super::super::ParseOnlyImportProvider, MizarParserSeam).run(SourceUnitRequest { snapshot: snapshot_id(778), input: SourceInput { package_id: PackageId::new("descendant-statement-boundary"), module_path: ModulePath::new("b5a"), normalized_path: mizar_session::normalize_path(&root, &path).unwrap(), edition: Edition::new("2026"), origin: SourceOriginInput::Disk { path: path.clone() } } }, &ids).unwrap();
        assert!(frontend.diagnostics.is_empty());
        assert!(frontend.preprocessed.import_stubs.is_empty());
        captures.push(frontend);
    }
    let ast = captures[0].ast.as_ref().unwrap();
    let module = ModuleId::new(
        captures[0].source.package_id.clone(),
        captures[0].source.module_path.clone(),
    );
    let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
    let (scope, labels, resolution) = source_proof_inputs(&source, &symbols)?;
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
    let statements = installed.source_statement().expect("native statement half");
    let references = installed
        .source_statement_references()
        .expect("native reference half");
    let unsupported = "theorems.equality.unsupported_source_types";
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
    assert_eq!(
        SourceVariableSemanticsChecker::source_theorem_binding_env(&source, &symbols, &scope)
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
                toolchain: ToolchainInfo::new("native-descendant-statement-test"),
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
        .find(|view| matches!(view.kind(), K::TheoremItem))
        .unwrap();
    // Exact original B5A bytes must not collide with a surviving legacy profile on any root envelope.
    for control in [
        "rooted",
        "rootless",
        "resolved-root-only",
        "typed-root-only",
        "changed",
        "enriched",
    ] {
        let mut nodes = base
            .nodes()
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        if control == "changed" {
            nodes[theorem.id().index()].children.reverse();
        }
        let reconstructed = TypedAst::try_new(TypedAstParts {
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
        .unwrap()
        .with_source_term(base.source_term().unwrap().clone())
        .unwrap()
        .with_source_atomic_formula(base.source_atomic_formula().unwrap().clone())
        .unwrap();
        assert_eq!(reconstructed.source_term(), base.source_term());
        assert_eq!(
            reconstructed.source_atomic_formula(),
            base.source_atomic_formula()
        );
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
            reconstructed
                .clone()
                .with_source_statement(statements.clone()),
            Err(TypedAstError::InvalidSourceStatement),
            "detached {control}"
        );
        assert!(
            reconstructed
                .with_source_statement_references(statements.clone(), references.clone())
                .is_err()
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
    let (unknown, unknown_env) =
        source_theorem_module_original_inputs(captures[2].ast.as_ref().unwrap(), &module)?.unwrap();
    let (unknown_scope, unknown_labels, unresolved) = source_proof_inputs(&unknown, &unknown_env)?;
    assert_eq!(
        validate_source_proof_occurrences(&unknown, &unknown_env, &unknown_scope),
        Ok(())
    );
    assert_eq!(
        (
            unknown_scope.references().len(),
            unknown_labels.projections().len(),
            unknown_labels.references().len()
        ),
        (10, 2, 1)
    );
    assert!(matches!(
        unresolved
            .table()
            .get(unresolved.ids()[0])
            .unwrap()
            .resolution(),
        LabelResolution::Unresolved(_)
    ));
    assert_eq!(
        TermFormulaChecker::check_source_theorem_equalities(&unknown, &unknown_env, &unknown_scope)
            .unwrap_err(),
        unsupported
    );
    assert_eq!(
        base.clone()
            .with_source_statement_from_original(&unknown, &unknown_env, &unknown_scope),
        Err(TypedAstError::InvalidSourceStatement)
    );
    let authentication =
        validate_source_proof_occurrences(&unknown, &empty, &unknown_scope).unwrap_err();
    assert_eq!(
        TermFormulaChecker::check_source_theorem_equalities(&unknown, &empty, &unknown_scope)
            .unwrap_err(),
        authentication
    );
    assert_eq!(
        base.clone()
            .with_source_statement_from_original(&source, &symbols, &scope),
        Ok(installed.clone())
    );
    let (b1, b1_env) =
        source_theorem_module_original_inputs(captures[3].ast.as_ref().unwrap(), &module)?.unwrap();
    let (b1_scope, _, _) = source_proof_inputs(&b1, &b1_env)?;
    let b1_base = TermFormulaChecker::check_source_theorem_equalities(&b1, &b1_env, &b1_scope)?.2;
    let b1_pair = b1_base
        .clone()
        .with_source_statement_from_original(&b1, &b1_env, &b1_scope)
        .unwrap();
    assert!(
        base.clone()
            .with_source_statement_references(
                statements.clone(),
                b1_pair.source_statement_references().unwrap().clone()
            )
            .is_err()
    );
    assert!(
        b1_base
            .clone()
            .with_source_statement_references(
                b1_pair.source_statement().unwrap().clone(),
                references.clone()
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
    for control in ["canonical", "metadata", "canonical"] {
        let replayed = base
            .clone()
            .with_source_statement_from_original(&source, &symbols, &scope)
            .unwrap();
        assert_eq!(replayed, installed);
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
                    typed_site: inference.formulas().iter().next().unwrap().1.site.clone(),
                    local_context: None,
                    cluster_facts: Vec::new(),
                }]
            } else {
                Vec::new()
            },
            node_hints: Vec::new(),
            statement_semantics: None,
            statement_proofs: None,
        });
        if control == "canonical" {
            let resolved = result.map_err(|error| error.to_string())?;
            assert_eq!(resolved.source_statement(), Some(statements));
            assert_eq!(resolved.source_statement_references(), Some(references));
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
            assert!(result.is_err(), "final accepted {control}");
        }
    }
    assert_eq!((source, symbols, scope, labels, resolution, base), before);
    assert_eq!(registry.get(snapshot.id), Some(snapshot.clone()));
    registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    fs::remove_dir_all(root).unwrap();
    Ok(())
}
