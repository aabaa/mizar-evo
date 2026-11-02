use super::*;

#[test]
fn source_nested_statement_original_capture_transports_native_pair() -> Result<(), String> {
    use mizar_checker::{
        binding_env::*, cluster_trace::ClusterFactTable, overload_resolution::*,
        resolved_typed_ast::*, source_atomic_formula::SourceAtomicTermTarget, source_statement::*,
        type_checker::TermFormulaChecker,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        labels::LabelProjectionSource,
        resolved_ast::{ModuleId, NodeReferenceKey, NodeResolutionState},
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
    // Spec 4.3/15.10/16.4: transport preserves reservations and local labels without proving them.
    let original = concat!(
        "reserve x for set;\n",
        "theorem FormulaStatementNestedContextSmoke: x = x proof\n",
        "  A: x = x proof\n",
        "    thus x = x;\n",
        "  end;\n",
        "  thus x = x by A;\n",
        "end;\n"
    );
    let texts = [original.to_owned(), original.replace("FormulaStatementNestedContextSmoke", "RenamedNested").replace("x", "parcel").replace("A:", "Evidence:").replace("by A", "by Evidence"), "reserve\n parcel\tfor :: type\n set;\ntheorem Layout: parcel = parcel proof\n Evidence: parcel = parcel proof :: nested\n thus parcel = parcel; end;\n thus parcel = parcel by Evidence; end;".to_owned()];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src/b1.miz");
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
                    snapshot: snapshot_id(777),
                    input: SourceInput {
                        package_id: PackageId::new("nested-statement-test"),
                        module_path: ModulePath::new("b1"),
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
                (source.arena().len(), scopes.len(), parents.len()),
                (77, 77, 76)
            );
            assert_eq!(
                (
                    scope.bindings().len(),
                    scope.references().len(),
                    labels.projections().len(),
                    labels.references().len()
                ),
                (1, 8, 2, 1)
            );
            assert_eq!(
                scope
                    .references()
                    .iter()
                    .map(|row| row.ordinal())
                    .collect::<Vec<_>>(),
                (0..8).collect::<Vec<_>>()
            );
            assert_eq!(resolution.index().len(), 2);
            assert_eq!(resolution.ids().len(), 1);
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
    // Stage 1 RED follows complete real resolver readiness for every capture.
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
        "native nested equality bases: {errors:?}"
    );
    for ((frontend, source, _, scope, _, _, scopes, parents, _), result) in cases.iter().zip(&bases)
    {
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
                atomic.formulas().len()
            ),
            (3, 1, 8, 8, 4)
        );
        let (binding_id, binding) = bindings.bindings().iter().next().unwrap();
        assert_eq!(binding_id.index(), 0);
        assert_eq!(binding.kind, BindingKind::ReservedVariable);
        assert_eq!(binding.owner_context.index(), 0);
        assert_eq!(
            binding.identity,
            BinderIdentity::ReservedVariable {
                spelling: scope.bindings()[0].spelling().to_owned(),
                declaration_range: scope.bindings()[0].range()
            }
        );
        assert_eq!(
            (binding.type_site.clone(), binding.status),
            (BindingTypeSite::Missing, BindingStatus::Active)
        );
        assert!(binding.captured.identities().is_empty() && binding.diagnostics.is_empty());
        let mut proofs = ast
            .node_views()
            .filter(|view| matches!(view.kind(), K::ProofBlock))
            .collect::<Vec<_>>();
        proofs.sort_by_key(|view| view.range().start);
        for (context_id, context) in bindings.contexts().iter() {
            let index = context_id.index();
            assert_eq!(context.visible_bindings, [binding_id]);
            assert_eq!(context.recovery, BindingContextRecovery::Normal);
            if index == 0 {
                assert_eq!(context.owner, BindingContextOwner::Module);
                assert!(context.parent.is_none() && context.lexical_scope.is_none());
                assert_eq!(context.bindings, [binding_id]);
                assert_eq!(context.layer, BindingContextLayer::Module);
            } else {
                let proof = &proofs[index - 1];
                assert_eq!(
                    context.owner,
                    BindingContextOwner::SourceStatement {
                        source_range: proof.range()
                    }
                );
                assert_eq!(context.parent.unwrap().index(), index - 1);
                assert_eq!(context.layer, BindingContextLayer::Proof);
                assert!(context.bindings.is_empty());
                assert_eq!(context.lexical_scope.as_ref(), scopes.get(&proof.id()));
                let parent = ast.node(parents[&proof.id()]).unwrap();
                assert_eq!(
                    parent.kind,
                    if index == 1 {
                        K::TheoremItem
                    } else {
                        K::CompactStatement
                    }
                );
            }
        }
        assert_eq!(
            bindings
                .contexts()
                .get(BindingContextId::new(1))
                .unwrap()
                .lexical_scope
                .as_ref()
                .unwrap()
                .path(),
            [4]
        );
        assert_eq!(
            bindings
                .contexts()
                .get(BindingContextId::new(2))
                .unwrap()
                .lexical_scope
                .as_ref()
                .unwrap()
                .path(),
            [4, 1]
        );
        for (((term_id, term), (_, reference)), (ordinal, occurrence)) in primary
            .terms()
            .iter()
            .zip(primary.references().iter())
            .zip(scope.references().iter().enumerate())
        {
            assert_eq!(term.site().node().index(), occurrence.node().index());
            assert_eq!(
                (term.source_range(), term.source_ordinal(), term.spelling()),
                (occurrence.range(), ordinal, occurrence.spelling())
            );
            assert_eq!(term.context().index(), [0, 0, 1, 1, 2, 2, 1, 1][ordinal]);
            assert_eq!(reference.term(), term_id);
            assert_eq!(reference.binding(), binding_id);
            assert_eq!(reference.lexical_scope(), Some(occurrence.scope()));
            assert_eq!(reference.use_ordinal(), ordinal);
            assert_eq!(scopes.get(&occurrence.node()), Some(occurrence.scope()));
        }
        let mut equalities = ast
            .node_views()
            .filter(|view| matches!(view.kind(), K::BuiltinPredicateApplication))
            .collect::<Vec<_>>();
        equalities.sort_by_key(|view| view.range().start);
        assert_eq!(equalities.len(), 4);
        assert_eq!(atomic.edges().len(), 8);
        for ((formula_id, formula), equality) in atomic.formulas().iter().zip(equalities) {
            let ordinal = formula_id.index();
            assert_eq!(formula.site().node().index(), equality.id().index());
            assert_eq!(formula.source_range(), equality.range());
            assert_eq!(formula.source_ordinal(), ordinal);
            assert_eq!(formula.context().index(), [0, 1, 2, 1][ordinal]);
            assert_eq!(
                formula.kind(),
                mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality
            );
            assert_eq!(
                formula.recovery(),
                mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal
            );
            let mut tokens = ast
                .node_views()
                .filter(|token| {
                    matches!(token.kind(), K::Token(_))
                        && equality.range().start <= token.range().start
                        && token.range().end <= equality.range().end
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
                .filter(|(_, edge)| edge.formula() == formula_id)
                .collect::<Vec<_>>();
            assert_eq!(edges.len(), 2);
            for (operand, (_, edge)) in edges.into_iter().enumerate() {
                let occurrence = &scope.references()[2 * ordinal + operand];
                let original_term = ast.node(equality.children()[2 * operand]).unwrap();
                assert_eq!(original_term.children, [occurrence.node()]);
                assert_eq!(edge.ordinal(), operand);
                assert_eq!(
                    edge.role(),
                    if operand == 0 {
                        mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand
                    } else {
                        mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand
                    }
                );
                let expected_term =
                    mizar_checker::source_term::SourcePrimaryTermId::new(2 * ordinal + operand);
                assert_eq!(
                    edge.target(),
                    SourceAtomicTermTarget::Primary(expected_term)
                );
                assert_eq!(
                    primary
                        .terms()
                        .get(expected_term)
                        .unwrap()
                        .site()
                        .node()
                        .index(),
                    occurrence.node().index()
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
        // Stage 2 RED begins only after all six genuine native bases above succeeded.
        let installed = base
            .clone()
            .with_source_statement_from_original(&source, &symbols, &scope);
        assert!(
            installed.is_ok(),
            "native nested paired installation: {installed:?}"
        );
        let installed = installed.unwrap();
        let statements = installed.source_statement().expect("native statement half");
        let references = installed
            .source_statement_references()
            .expect("native reference half");
        assert_eq!(statements.binding_env(), &bindings);
        assert_eq!(
            (
                statements.owners().len(),
                statements.statements().len(),
                statements.contexts().len(),
                statements.input_facts().len(),
                statements.candidate_facts().len()
            ),
            (1, 4, 4, 4, 4)
        );
        assert_eq!(statements.binding_fingerprint(), bindings.debug_text());
        assert_eq!(
            statements.primary_term_fingerprint(),
            base.source_term().unwrap().debug_text()
        );
        assert_eq!(
            statements.atomic_formula_fingerprint(),
            base.source_atomic_formula().unwrap().debug_text()
        );
        assert_eq!(references.statement_fingerprint(), statements.debug_text());
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
        assert_eq!(sites.len(), 4);
        let (_, owner) = statements.owners().iter().next().unwrap();
        assert_eq!(owner.site().node().index(), sites[0].id().index());
        assert_eq!(owner.source_range(), sites[0].range());
        assert_eq!(owner.symbol(), statements.checked_owner().symbol());
        let owner_projection = labels
            .projections()
            .iter()
            .find(|projection| {
                matches!(
                    projection.source(),
                    LabelProjectionSource::CurrentModule {
                        proof_scope: None,
                        ..
                    }
                )
            })
            .unwrap();
        assert_eq!(owner.spelling(), owner_projection.primary_spelling());
        assert_eq!(owner.contribution(), owner_projection.contribution());
        assert_eq!(
            (owner.role(), owner.status(), owner.recovery()),
            (
                SourceTheoremRole::Theorem,
                SourceTheoremStatus::Unmodified,
                SourceStatementRecovery::Normal
            )
        );
        for ((id, row), site) in statements.statements().iter().zip(sites) {
            let ordinal = id.index();
            assert_eq!(row.owner().index(), 0);
            assert_eq!(row.context().index(), ordinal);
            assert_eq!(row.source_ordinal(), ordinal);
            assert_eq!(row.site().node().index(), site.id().index());
            assert_eq!(row.source_range(), site.range());
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
            let formula_id =
                mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(ordinal);
            assert_eq!(
                row.formula(),
                SourceStatementFormulaTarget::Atomic(formula_id)
            );
            let context = statements.contexts().get(row.context()).unwrap();
            assert_eq!(context.statement(), id);
            assert_eq!(context.source_range(), site.range());
            assert_eq!(context.binding_context().index(), [0, 1, 2, 1][ordinal]);
            assert_eq!(context.visible_bindings(), &[BindingId::new(0)]);
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
                    mizar_checker::source_term::SourcePrimaryTermReferenceId::new(2 * ordinal),
                    mizar_checker::source_term::SourcePrimaryTermReferenceId::new(2 * ordinal + 1)
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
        let projections = labels
            .projections()
            .iter()
            .filter(|projection| {
                matches!(
                    projection.source(),
                    LabelProjectionSource::CurrentModule {
                        proof_scope: Some(_),
                        ..
                    }
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(projections.len(), 1);
        let projection = projections[0];
        assert_eq!(references.label_projection(), projection);
        assert!(labels.projections().iter().any(|projection| matches!(
            projection.source(),
            LabelProjectionSource::CurrentModule {
                proof_scope: None,
                visible_after_ordinal: 3
            }
        )));
        assert!(
            matches!(projection.source(), LabelProjectionSource::CurrentModule { proof_scope: Some(scope), visible_after_ordinal: 2 } if scope.path() == [0])
        );
        let (label_id, label) = references.labels().iter().next().unwrap();
        assert_eq!(references.labels().len(), 1);
        assert_eq!(
            (
                label.statement().index(),
                label.context().index(),
                label.candidate().index(),
                label.source_ordinal(),
                label.visible_after_ordinal()
            ),
            (1, 1, 1, 0, 2)
        );
        assert_eq!(
            (label.origin_path(), label.source_range(), label.spelling()),
            (
                projection.origin_path(),
                projection.declaration_range(),
                projection.primary_spelling()
            )
        );
        assert_eq!(label.proof_scope().path(), [0]);
        assert_eq!(
            (label.kind(), label.recovery()),
            (
                SourceStatementLabelKind::ProofStep,
                SourceStatementRecovery::Normal
            )
        );
        let (_, citation) = references.citations().iter().next().unwrap();
        assert_eq!(references.citations().len(), 1);
        assert_eq!(
            (
                citation.statement().index(),
                citation.context().index(),
                citation.ordinal(),
                citation.target(),
                citation.label_ref()
            ),
            (
                3,
                3,
                0,
                SourceStatementCitationTarget::Local(label_id),
                resolution.ids()[0]
            )
        );
        assert_eq!(
            (citation.source_range(), citation.proof_scope().path()),
            (labels.references()[0].site().range(), &[0][..])
        );
        assert_eq!(
            (citation.kind(), citation.recovery()),
            (
                SourceStatementCitationKind::SimpleLocal,
                SourceStatementRecovery::Normal
            )
        );
        assert_eq!(references.reference_candidate().ordinal(), 3);
        let resolved_ref = resolution.table().get(citation.label_ref()).unwrap();
        assert_eq!(resolved_ref.origin(), labels.references()[0].origin());
        assert!(
            matches!(resolved_ref.resolution(), mizar_resolve::resolved_ast::LabelResolution::Resolved(target) if target.origin() == projection.origin_path() && target.kind() == mizar_resolve::resolved_ast::LabelKind::ProofStep)
        );
        let snapshot = references.resolver_ast();
        assert_eq!(
            (snapshot.nodes().root(), snapshot.nodes().len()),
            (source.arena().root(), source.arena().len())
        );
        // Empty ordinary-name/import tables express this bounded label-only snapshot, not phase completion.
        assert!(
            snapshot.name_refs().is_empty()
                && snapshot.imports().imports().next().is_none()
                && snapshot.imports().exports().next().is_none()
        );
        let citation_site = labels.references()[0].site().node();
        let label_site = ast
            .node_views()
            .find(|view| {
                view.range() == projection.declaration_range() && matches!(view.kind(), K::Token(_))
            })
            .unwrap();
        assert_ne!(
            projection.origin().structural_path(),
            source
                .arena()
                .node(source.resolved_node_for(label_site.id()).unwrap())
                .unwrap()
                .origin()
                .structural_path()
        );
        assert_ne!(
            labels.references()[0].origin().structural_path(),
            source
                .arena()
                .node(citation_site)
                .unwrap()
                .origin()
                .structural_path()
        );
        for (id, original_node) in source.arena().iter() {
            let expected = if id == citation_site {
                original_node
                    .clone()
                    .with_reference_key(NodeReferenceKey::Label(resolution.ids()[0]))
                    .with_resolution(NodeResolutionState::Resolved)
            } else {
                original_node.clone()
            };
            assert_eq!(snapshot.nodes().node(id), Some(&expected));
        }
        assert_eq!(installed.nodes(), base.nodes());
        let mut final_baseline = None;
        for typed in std::iter::once(&installed).chain((case == 0).then_some(&installed.clone())) {
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
            assert!(resolved.nodes().root().is_some());
            assert!(
                resolved.expr_metadata().is_empty()
                    && resolved.checked_formulas().is_empty()
                    && resolved.statement_semantics().is_empty()
                    && resolved.checked_proofs().is_empty()
            );
            if let Some(baseline) = &final_baseline {
                assert_eq!(&resolved, baseline);
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
fn source_nested_statement_original_capture_preserves_boundary() -> Result<(), String> {
    use mizar_checker::{
        cluster_trace::ClusterFactTable,
        overload_resolution::*,
        resolved_typed_ast::*,
        source_context::SourceBindingContextProducer,
        source_type::SourceTypeProducer,
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
        SessionIdAllocator, SnapshotInput, SnapshotRegistry, SourceAnchor, SourceInput,
        SourceOriginInput, SourceVersion, ToolchainInfo, WorkspaceRoot, hash_text,
    };
    use mizar_syntax::SurfaceNodeKind as K;
    let text = concat!(
        "reserve x for set;\n",
        "theorem FormulaStatementNestedContextSmoke: x = x proof\n",
        "  A: x = x proof\n",
        "    thus x = x;\n",
        "  end;\n",
        "  thus x = x by A;\n",
        "end;\n"
    );
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src/b1.miz");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let mut captures = Vec::new();
    for current in [
        text.to_owned(),
        text.replace("Smoke", "Foreign"),
        text.replace("by A", "by Missing"),
        text.replace("for set", "for object"),
        text.replace("thus x = x;", "thus x =;"),
        "reserve x for set; theorem Plain: x = x;".to_owned(),
        "theorem Quantified: for x being set holds x = x proof let y be set; thus y = y; end;"
            .to_owned(),
    ] {
        fs::write(&path, &current).unwrap();
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: snapshot_id(777),
                input: SourceInput {
                    package_id: PackageId::new("nested-statement-boundary"),
                    module_path: ModulePath::new("b1"),
                    normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Disk { path: path.clone() },
                },
            },
            &ids,
        )
        .unwrap();
        captures.push(frontend);
    }
    let ast = captures[0].ast.as_ref().unwrap();
    assert!(captures[0].diagnostics.is_empty());
    let module = ModuleId::new(
        captures[0].source.package_id.clone(),
        captures[0].source.module_path.clone(),
    );
    let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
    let (scope, labels, resolution) = source_proof_inputs(&source, &symbols)?;
    assert_eq!(
        validate_source_proof_occurrences(&source, &symbols, &scope),
        Ok(())
    );
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
    let statement = installed.source_statement().expect("statement half");
    let references = installed
        .source_statement_references()
        .expect("reference half");
    assert_eq!(
        installed
            .clone()
            .with_source_statement_from_original(&source, &symbols, &scope),
        Err(TypedAstError::InvalidSourceStatement)
    );
    assert_eq!(
        base.clone()
            .with_source_statement_from_original(&source, &symbols, &scope),
        Ok(installed.clone())
    );
    // Higher wrappers and Reserve/proof guards must refuse after genuine B1 base success.
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
    assert_eq!(
        SourceBindingContextProducer::build_source_theorem_equalities(&source, &symbols, &scope)
            .unwrap_err(),
        unsupported
    );
    assert_eq!(
        SourceTypeProducer::build_source_theorem_equalities(&source, &symbols, &scope).unwrap_err(),
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
                toolchain: ToolchainInfo::new("native-nested-statement-test"),
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
    let theorem = ast
        .node_views()
        .find(|view| matches!(view.kind(), K::TheoremItem))
        .unwrap();
    let label = ast.node_views().find(|view| matches!(view.kind(), K::Token(token) if token.text.as_ref() == "FormulaStatementNestedContextSmoke")).unwrap();
    for control in [
        "rooted",
        "rootless",
        "resolved-root-only",
        "typed-root-only",
        "token",
        "child",
        "range",
        "enriched",
    ] {
        let mut nodes = base
            .nodes()
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        match control {
            "token" => {
                nodes[label.id().index()].kind = TypedNodeKind::new("changed-token-envelope")
            }
            "child" => nodes[theorem.id().index()].children.reverse(),
            "range" => nodes[label.id().index()].anchor = SourceAnchor::Range(theorem.range()),
            _ => {}
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
                .with_source_statement(statement.clone()),
            Err(TypedAstError::InvalidSourceStatement),
            "detached statement {control}"
        );
        assert!(
            reconstructed
                .clone()
                .with_source_statement_references(statement.clone(), references.clone())
                .is_err(),
            "detached pair {control}"
        );
        assert_eq!(
            base.clone()
                .with_source_statement_from_original(&source, &symbols, &scope),
            Ok(installed.clone())
        );
    }
    let (foreign_source, foreign_symbols) =
        source_theorem_module_original_inputs(captures[1].ast.as_ref().unwrap(), &module)?.unwrap();
    let (foreign_scope, _, _) = source_proof_inputs(&foreign_source, &foreign_symbols)?;
    let empty = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
    for (current, env, occurrences) in [
        (&foreign_source, &symbols, &scope),
        (&source, &foreign_symbols, &scope),
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
    for (control, capture) in ["unknown label", "unsupported type", "recovered equality"]
        .into_iter()
        .zip(&captures[2..5])
    {
        let current_ast = capture.ast.as_ref().unwrap();
        let (current, env) = source_theorem_module_original_inputs(current_ast, &module)?.unwrap();
        match source_proof_inputs(&current, &env) {
            Ok((occurrences, current_labels, resolved)) => {
                assert_eq!(
                    validate_source_proof_occurrences(&current, &env, &occurrences),
                    Ok(())
                );
                let own = TermFormulaChecker::check_source_theorem_equalities(
                    &current,
                    &env,
                    &occurrences,
                );
                if control == "unknown label" {
                    assert_eq!(
                        (
                            occurrences.references().len(),
                            current_labels.projections().len(),
                            current_labels.references().len()
                        ),
                        (8, 2, 1)
                    );
                    assert!(resolved.has_unresolved());
                    let [reference_id] = resolved.ids() else {
                        panic!("one genuine unresolved citation")
                    };
                    let candidate = &current_labels.references()[0];
                    assert_eq!(
                        resolved.table().get(*reference_id).unwrap().resolution(),
                        &mizar_resolve::resolved_ast::LabelResolution::Unresolved(
                            mizar_resolve::resolved_ast::UnresolvedLabelRef::new(
                                "Missing",
                                candidate.site().range(),
                                candidate.expectation(),
                            )
                        )
                    );
                    assert_eq!(own.as_ref().unwrap_err(), unsupported);
                }
                if control == "unsupported type" {
                    let authentication =
                        validate_source_proof_occurrences(&current, &empty, &occurrences)
                            .unwrap_err();
                    assert_eq!(
                        TermFormulaChecker::check_source_theorem_equalities(
                            &current,
                            &empty,
                            &occurrences
                        )
                        .unwrap_err(),
                        authentication
                    );
                }
                assert_eq!(own.unwrap_err(), unsupported, "{control}");
                assert_eq!(
                    base.clone()
                        .with_source_statement_from_original(&current, &env, &occurrences),
                    Err(TypedAstError::InvalidSourceStatement),
                    "{control}"
                );
            }
            Err(_) => {
                assert_eq!(control, "recovered equality");
                assert!(!capture.diagnostics.is_empty());
                // Recovered syntax cannot supply its own occurrence batch; keep this foreign-scope refusal explicit.
                assert!(validate_source_proof_occurrences(&current, &env, &scope).is_err());
                assert_eq!(
                    base.clone()
                        .with_source_statement_from_original(&current, &env, &scope),
                    Err(TypedAstError::InvalidSourceStatement)
                );
            }
        }
        assert_eq!(
            base.clone()
                .with_source_statement_from_original(&source, &symbols, &scope),
            Ok(installed.clone())
        );
    }
    for (index, capture) in captures[5..].iter().enumerate() {
        let (current, env) =
            source_theorem_module_original_inputs(capture.ast.as_ref().unwrap(), &module)?.unwrap();
        let (occurrences, _, _) = source_proof_inputs(&current, &env)?;
        let (_, _, own_base) =
            TermFormulaChecker::check_source_theorem_equalities(&current, &env, &occurrences)?;
        assert!(
            TermFormulaChecker::check_source_theorem_equalities_with_expression_metadata(
                &current,
                &env,
                &occurrences
            )
            .is_ok()
        );
        if index == 0 {
            // Omitted outer proof is the existing A positive, with one statement and no reference half.
            let a = own_base
                .clone()
                .with_source_statement_from_original(&current, &env, &occurrences)
                .unwrap();
            assert_eq!(a.source_statement().unwrap().statements().len(), 1);
            assert!(a.source_statement_references().is_none());
            assert!(
                own_base
                    .with_source_statement_references(
                        a.source_statement().unwrap().clone(),
                        references.clone(),
                    )
                    .is_err()
            );
            assert_eq!(
                installed
                    .clone()
                    .with_source_statement_from_original(&current, &env, &occurrences),
                Err(TypedAstError::InvalidSourceStatement)
            );
        }
        assert_eq!(
            base.clone()
                .with_source_statement_from_original(&source, &symbols, &scope),
            Ok(installed.clone())
        );
    }
    let collection = OverloadCollectionOutput::collect([], []);
    let expansion = TemplateExpansionOutput::expand(&collection);
    let viability = CandidateViabilityOutput::filter(&expansion, []);
    let graphs = SpecificityGraphOutput::build(&viability, []);
    let selection = OverloadSelectionOutput::resolve(&graphs, []);
    let cluster_facts = ClusterFactTable::new();
    let mut final_baseline = None;
    for control in [
        "canonical",
        "metadata",
        "canonical",
        "hint",
        "canonical",
        "semantics",
        "canonical",
        "proofs",
        "canonical",
    ] {
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
            node_hints: if control == "hint" {
                vec![ResolvedNodeKindHint {
                    typed_node: TypedNodeId::new(theorem.id().index()),
                    kind: ResolvedNodeKindHintKind::ResolvedUse {
                        symbol: owner.symbol().clone(),
                    },
                }]
            } else {
                Vec::new()
            },
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

#[test]
fn source_nested_parenthesized_witness_statement_original_capture_transports_native_pair()
-> Result<(), String> {
    use super::super::super::type_elaboration::assemble_empty_resolved_typed_ast;
    use mizar_checker::{
        binding_env::*,
        source_atomic_formula::*,
        source_statement::*,
        source_term::*,
        type_checker::{
            FormulaStatus, NormalizedTypeStatus, TermFormulaChecker, TermKind, TermReference,
            TermStatus, TypeHeadRef,
        },
        typed_ast::{TypeEntryActual, TypeStatus},
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
    // EN13.1.3 preserves the actual child's type; EN15/16 leave existential-goal checking downstream.
    let original = "reserve x for set;\ntheorem FormulaStatementNestedParenthesizedWitnessSmoke: x = x proof\n  take ((x));\n  thus x = x;\nend;\n";
    assert_eq!(original.len(), 121);
    let texts = [original.to_owned(), original.replace("FormulaStatementNestedParenthesizedWitnessSmoke", "RenamedNestedParenthesis").replace("x", "parcel"), "reserve\n parcel\tfor :: type hint\n set;\ntheorem LayoutNestedParenthesis: parcel = parcel proof :: syntax only\n take ( ( parcel ) ); thus parcel = parcel; end;".to_owned()];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src/nestedparenthesized.miz");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let mut cases = Vec::new();
    for (depth, text) in [2usize, 3, 5, 8].into_iter().flat_map(|depth| {
        texts.iter().map(move |text| {
            let (old, leaf, spaced) = if text.contains("take ( (") {
                ("( ( parcel ) )", "parcel", true)
            } else if text.contains("((parcel))") {
                ("((parcel))", "parcel", false)
            } else {
                ("((x))", "x", false)
            };
            let group = if spaced {
                format!("{}{leaf}{}", "( ".repeat(depth), " )".repeat(depth))
            } else {
                format!("{}{leaf}{}", "(".repeat(depth), ")".repeat(depth))
            };
            (depth, text.replace(old, &group))
        })
    }) {
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
                        package_id: PackageId::new("nested-parenthesized-witness-test"),
                        module_path: ModulePath::new("nestedparenthesized"),
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
            assert!(ast.node_views().all(|node| !node.is_recovered()));
            let module = ModuleId::new(
                frontend.source.package_id.clone(),
                frontend.source.module_path.clone(),
            );
            let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
            assert_eq!(source.source_id(), frontend.source.source_id);
            let (scope, labels, resolution) = source_proof_inputs(&source, &symbols)?;
            source.validate_against(ast, &module).unwrap();
            validate_source_proof_occurrences(&source, &symbols, &scope)?;
            let (scopes, parents) = source_proof_scope_maps(&source, &symbols, &scope)?;
            assert_eq!(
                (source.arena().len(), scopes.len(), parents.len()),
                (49 + 4 * depth, 49 + 4 * depth, 48 + 4 * depth)
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
            assert!(symbols.labels().is_empty() && symbols.imports().is_empty());
            assert!(!resolution.has_unresolved() && resolution.diagnostics().is_empty());
            assert_eq!(
                scope
                    .references()
                    .iter()
                    .map(|row| row.ordinal())
                    .collect::<Vec<_>>(),
                [0, 1, 2, 3, 4]
            );
            let projection = &labels.projections()[0];
            let owner = symbols
                .symbols()
                .iter()
                .find(|entry| entry.contribution() == projection.contribution())
                .unwrap();
            assert_eq!(owner.kind(), mizar_resolve::env::SymbolKind::Theorem);
            assert_eq!(owner.primary_spelling(), projection.primary_spelling());
            assert_eq!(owner.origin(), projection.origin());
            for (kind, count) in [
                (K::ReserveItem, 1),
                (K::ReserveSegment, 1),
                (K::TypeExpression, 1),
                (K::TheoremItem, 1),
                (K::ConclusionStatement, 1),
                (K::BuiltinPredicateApplication, 2),
                (K::ProofBlock, 1),
                (K::TakeStatement, 1),
                (K::Witness, 1),
                (K::ParenthesizedTerm, depth),
                (K::TermReference, 5),
                (K::CompactStatement, 0),
                (K::Reference, 0),
                (K::NumeralTerm, 0),
            ] {
                assert_eq!(
                    ast.node_views().filter(|node| node.kind() == &kind).count(),
                    count
                );
            }
            let proof = ast
                .node_views()
                .find(|node| node.kind() == &K::ProofBlock)
                .unwrap();
            let take = ast
                .node_views()
                .find(|node| node.kind() == &K::TakeStatement)
                .unwrap();
            assert_eq!(ast.node(parents[&proof.id()]).unwrap().kind, K::TheoremItem);
            assert_eq!(parents[&take.id()], proof.id());
            assert_eq!(take.children().len(), 3);
            let witness = ast.node(take.children()[1]).unwrap();
            assert_eq!(witness.kind, K::Witness);
            assert_eq!(witness.children.len(), 1);
            let mut expression_id = witness.children[0];
            let mut expression_parent = take.children()[1];
            for _ in 0..depth {
                let expression = ast.node(expression_id).unwrap();
                assert_eq!(expression.kind, K::TermExpression);
                assert_eq!(parents[&expression_id], expression_parent);
                assert_eq!(scopes[&expression_id], scopes[&proof.id()]);
                let [wrapper_id] = expression.children.as_slice() else {
                    panic!("one witness group")
                };
                let wrapper = ast.node(*wrapper_id).unwrap();
                assert_eq!(wrapper.kind, K::ParenthesizedTerm);
                assert_eq!(wrapper.children.len(), 3);
                assert_eq!(parents[wrapper_id], expression_id);
                assert_eq!(scopes[wrapper_id], scopes[&proof.id()]);
                assert!(
                    !scope
                        .references()
                        .iter()
                        .any(|row| row.node() == *wrapper_id)
                );
                for (delimiter, spelling) in
                    [(wrapper.children[0], "("), (wrapper.children[2], ")")]
                {
                    assert_eq!(ast.node(delimiter).unwrap().token_text(), Some(spelling));
                    assert_eq!(parents[&delimiter], *wrapper_id);
                    assert_eq!(scopes[&delimiter], scopes[&proof.id()]);
                }
                expression_parent = *wrapper_id;
                expression_id = wrapper.children[1];
            }
            let leaf_expression = ast.node(expression_id).unwrap();
            assert_eq!(leaf_expression.kind, K::TermExpression);
            assert_eq!(parents[&expression_id], expression_parent);
            assert_eq!(scopes[&expression_id], scopes[&proof.id()]);
            let [child_id] = leaf_expression.children.as_slice() else {
                panic!("one reserved leaf at the expected depth")
            };
            assert_eq!(ast.node(*child_id).unwrap().kind, K::TermReference);
            let reference = scope
                .references()
                .iter()
                .find(|row| row.node() == *child_id)
                .unwrap();
            assert_eq!(reference.ordinal(), 2);
            assert_eq!(reference.spelling(), scope.bindings()[0].spelling());
            assert_eq!(parents[child_id], expression_id);
            assert_eq!(scopes[child_id], scopes[&proof.id()]);
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
            cases.push((
                frontend, source, symbols, scope, labels, resolution, scopes, parents, before,
                depth,
            ));
        }
    }
    assert_eq!(cases.len(), 24);
    // First runtime RED follows readiness of every genuine capture and all 24 real lower calls.
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
        "native nested parenthesized witness equality bases: {errors:?}"
    );
    for (
        (frontend, source, symbols, scope, labels, resolution, scopes, parents, before, depth),
        result,
    ) in cases.iter().zip(&bases)
    {
        let depth = *depth;
        let (bindings, inference, base) = result.as_ref().unwrap();
        let ast = frontend.ast.as_ref().unwrap();
        let primary = base.source_term().unwrap();
        let atomic = base.source_atomic_formula().unwrap();
        let typed_by_resolved = base
            .nodes()
            .iter()
            .map(|(id, node)| (node.resolved_node.unwrap(), id))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(typed_by_resolved.len(), base.nodes().len());
        assert_eq!(base.nodes().len(), ast.nodes().len());
        assert_eq!(base.resolved_root(), Some(source.arena().root()));
        assert_eq!(
            base.nodes().root(),
            Some(typed_by_resolved[&source.resolved_node_for(ast.root().unwrap()).unwrap()])
        );
        for surface in ast.node_views() {
            let resolved = source.resolved_node_for(surface.id()).unwrap();
            let node = base.nodes().node(typed_by_resolved[&resolved]).unwrap();
            let kind = match surface.kind() {
                K::TermReference => "source.term.variable-reference".to_owned(),
                K::ParenthesizedTerm => "source.term.parenthesized".to_owned(),
                K::BuiltinPredicateApplication => "source.formula.atomic.equality".to_owned(),
                kind => format!("{kind:?}"),
            };
            assert_eq!(node.kind.as_str(), kind);
            assert_eq!(node.resolved_node, Some(resolved));
            assert_eq!(
                node.anchor,
                mizar_session::SourceAnchor::Range(surface.range())
            );
            assert_eq!(
                node.children,
                surface
                    .children()
                    .iter()
                    .map(|child| { typed_by_resolved[&source.resolved_node_for(*child).unwrap()] })
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                node.recovery,
                mizar_checker::typed_ast::NodeRecoveryState::Normal
            );
            assert_eq!(node.typing, mizar_checker::typed_ast::TypingState::Unknown);
            assert_eq!(
                node.links,
                mizar_checker::typed_ast::TypedNodeLinks::default()
            );
        }
        assert_eq!(
            [
                bindings.contexts().len(),
                bindings.bindings().len(),
                primary.terms().len(),
                primary.references().len(),
                primary.numeric_type_requests().len(),
                atomic.formulas().len(),
                atomic.edges().len(),
                atomic.requests().len(),
                inference.terms().len(),
                inference.formulas().len()
            ],
            [2, 1, depth + 5, 5, 0, 2, 4, 4, depth + 5, 2]
        );
        let (binding_id, binding) = bindings.bindings().iter().next().unwrap();
        assert_eq!(binding.kind, BindingKind::ReservedVariable);
        assert_eq!(binding.status, BindingStatus::Active);
        assert_eq!(binding.visible_after_ordinal, scope.bindings()[0].ordinal());
        assert!(binding.captured.identities().is_empty());
        let proof = ast
            .node_views()
            .find(|node| node.kind() == &K::ProofBlock)
            .unwrap();
        assert_eq!(ast.node(parents[&proof.id()]).unwrap().kind, K::TheoremItem);
        let module_context = bindings
            .contexts()
            .iter()
            .find(|(_, context)| context.owner == BindingContextOwner::Module)
            .unwrap()
            .0;
        for (_, context) in bindings.contexts().iter() {
            assert_eq!(context.visible_bindings, [binding_id]);
            if context.owner == BindingContextOwner::Module {
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
                assert_eq!(context.parent, Some(module_context));
                assert!(context.bindings.is_empty());
                assert_eq!(context.lexical_scope.as_ref(), scopes.get(&proof.id()));
            }
        }
        let take_site = ast
            .node_views()
            .find(|node| node.kind() == &K::TakeStatement)
            .unwrap();
        let witness_site = ast.node(take_site.children()[1]).unwrap();
        let mut expression_site = ast.node(witness_site.children[0]).unwrap();
        let mut wrapper_sites = Vec::new();
        for _ in 0..depth {
            assert_eq!(expression_site.kind, K::TermExpression);
            assert_eq!(expression_site.children.len(), 1);
            let wrapper_site_id = expression_site.children[0];
            let wrapper_site = ast.node(wrapper_site_id).unwrap();
            assert_eq!(wrapper_site.kind, K::ParenthesizedTerm);
            wrapper_sites.push(wrapper_site_id);
            expression_site = ast.node(wrapper_site.children[1]).unwrap();
        }
        assert_eq!(expression_site.kind, K::TermExpression);
        assert_eq!(expression_site.children.len(), 1);
        let child_site = expression_site.children[0];
        assert_eq!(ast.node(child_site).unwrap().kind, K::TermReference);
        let wrapper_id = SourcePrimaryTermId::new(2);
        let child_id = SourcePrimaryTermId::new(depth + 2);
        let wrapper = primary.terms().get(wrapper_id).unwrap();
        let child = primary.terms().get(child_id).unwrap();
        assert_eq!(
            child.site().node(),
            typed_by_resolved[&source.resolved_node_for(child_site).unwrap()]
        );
        for (offset, site_id) in wrapper_sites.iter().enumerate() {
            let site = ast.node(*site_id).unwrap();
            let id = SourcePrimaryTermId::new(offset + 2);
            let row = primary.terms().get(id).unwrap();
            let immediate_child = primary
                .terms()
                .get(SourcePrimaryTermId::new(offset + 3))
                .unwrap();
            assert_eq!(
                row.site().node(),
                typed_by_resolved[&source.resolved_node_for(*site_id).unwrap()]
            );
            assert_eq!(row.source_range(), site.range);
            assert_eq!(row.kind(), SourcePrimaryTermKind::Parenthesized);
            assert_eq!(
                row.parent(),
                (offset > 0).then(|| SourcePrimaryTermId::new(offset + 1))
            );
            assert_eq!(immediate_child.parent(), Some(id));
            assert_eq!(row.context(), immediate_child.context());
            assert_eq!(row.context(), child.context());
            assert_eq!(
                row.spelling(),
                format!("( {} )", immediate_child.spelling())
            );
            assert!(
                !primary
                    .references()
                    .iter()
                    .any(|(_, reference)| reference.term() == id)
            );
        }
        assert_eq!(
            bindings
                .contexts()
                .get(wrapper.context())
                .unwrap()
                .lexical_scope
                .as_ref(),
            scopes.get(&proof.id())
        );
        for ((id, row), ordinal) in primary.terms().iter().zip(0..depth + 5) {
            assert_eq!(row.source_ordinal(), ordinal);
            assert_eq!(row.role(), SourcePrimaryTermRole::Value);
            assert_eq!(row.recovery(), SourcePrimaryTermRecovery::Normal);
            assert_eq!(
                row.parent(),
                if (3..=depth + 2).contains(&id.index()) {
                    Some(SourcePrimaryTermId::new(id.index() - 1))
                } else {
                    None
                }
            );
        }
        for ((reference_id, reference), occurrence) in
            primary.references().iter().zip(scope.references())
        {
            let target = [0, 1, depth + 2, depth + 3, depth + 4][reference_id.index()];
            assert_eq!(reference.term().index(), target);
            let term = primary.terms().get(reference.term()).unwrap();
            assert_eq!(
                term.site().node(),
                typed_by_resolved[&source.resolved_node_for(occurrence.node()).unwrap()]
            );
            assert_eq!(
                (term.source_range(), term.spelling()),
                (occurrence.range(), occurrence.spelling())
            );
            assert_eq!(term.source_ordinal(), target);
            assert_eq!(term.kind(), SourcePrimaryTermKind::VariableReference);
            assert_eq!(reference.binding(), binding_id);
            assert_eq!(reference.use_ordinal(), occurrence.ordinal());
            assert_eq!(reference.lexical_scope(), Some(occurrence.scope()));
            assert_eq!(scopes.get(&occurrence.node()), Some(occurrence.scope()));
            let context = bindings.contexts().get(term.context()).unwrap();
            if occurrence.range().start < proof.range().start {
                assert_eq!(context.owner, BindingContextOwner::Module);
            } else {
                assert_eq!(context.lexical_scope.as_ref(), Some(occurrence.scope()));
            }
        }
        assert_eq!(
            primary
                .references()
                .get(SourcePrimaryTermReferenceId::new(2))
                .unwrap()
                .use_ordinal(),
            2
        );
        assert_eq!(child.source_ordinal(), depth + 2);
        // Checked rows are joined by actual site, never by source-primary or reference IDs.
        let written_type = ast
            .node_views()
            .find(|node| node.kind() == &K::TypeExpression)
            .unwrap();
        assert_eq!(
            ast.node(parents[&written_type.id()]).unwrap().kind,
            K::ReserveSegment
        );
        let child_checked = inference
            .terms()
            .iter()
            .find(|(_, term)| term.site == *child.site())
            .unwrap()
            .1;
        assert_eq!(child_checked.kind, TermKind::Variable);
        assert_eq!(
            child_checked.reference,
            Some(TermReference::Binding(binding_id))
        );
        let child_type = inference
            .type_entries()
            .get(child_checked.type_entry)
            .unwrap();
        assert_eq!(child_type.owner, *child.site());
        for offset in 0..depth {
            let row = primary
                .terms()
                .get(SourcePrimaryTermId::new(offset + 2))
                .unwrap();
            let immediate_child = primary
                .terms()
                .get(SourcePrimaryTermId::new(offset + 3))
                .unwrap();
            let checked = inference
                .terms()
                .iter()
                .find(|(_, term)| term.site == *row.site())
                .unwrap()
                .1;
            let checked_child = inference
                .terms()
                .iter()
                .find(|(_, term)| term.site == *immediate_child.site())
                .unwrap()
                .1;
            assert_eq!(checked.kind, TermKind::Parenthesized);
            assert_eq!(checked.reference, None);
            assert_ne!(checked.type_entry, checked_child.type_entry);
            let entry = inference.type_entries().get(checked.type_entry).unwrap();
            let child_entry = inference
                .type_entries()
                .get(checked_child.type_entry)
                .unwrap();
            assert_eq!(entry.owner, *row.site());
            assert_eq!(child_entry.owner, *immediate_child.site());
            assert_eq!(entry.actual, child_entry.actual);
            assert_eq!(entry.actual, child_type.actual);
        }
        let TypeEntryActual::Known(normalized_id) = child_type.actual else {
            panic!("known preserved type")
        };
        let normalized = inference.normalized_types().get(normalized_id).unwrap();
        assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
        assert_eq!(normalized.status, NormalizedTypeStatus::Known);
        assert_eq!(normalized.source.range, written_type.range());
        assert_eq!(normalized.source.spelling, "set");
        let written_type_node =
            typed_by_resolved[&source.resolved_node_for(written_type.id()).unwrap()];
        let mut type_roles = std::collections::BTreeSet::new();
        for (_, entry) in inference.type_entries().iter() {
            if let mizar_checker::typed_ast::TypedSiteRef::Role { node, role } = &entry.owner {
                assert_eq!(*node, written_type_node);
                assert!(type_roles.insert(role.as_str().to_owned()));
                assert_eq!(entry.actual, TypeEntryActual::Known(normalized_id));
                assert_eq!(entry.status, TypeStatus::Known);
            }
        }
        assert_eq!(type_roles.len(), depth + 5);
        assert_eq!(inference.type_entries().len(), 2 * (depth + 5));
        for occurrence in scope.references() {
            assert!(type_roles.contains(&format!("theorem.equality.use:{}", occurrence.ordinal())));
        }
        assert!(type_roles.contains("theorem.equality.parenthesized"));
        for offset in 1..depth {
            assert!(type_roles.contains(&format!("theorem.equality.parenthesized:{offset}")));
        }
        assert!(
            normalized.args.is_empty()
                && normalized.attributes.positive().is_empty()
                && normalized.attributes.negative().is_empty()
        );
        assert_eq!(inference.normalized_types().len(), 1);
        let mut checked_owners = std::collections::BTreeSet::new();
        let mut checked_entries = std::collections::BTreeSet::new();
        for (_, term) in primary.terms().iter() {
            let checked = inference
                .terms()
                .iter()
                .find(|(_, row)| row.site == *term.site())
                .unwrap()
                .1;
            assert_eq!(checked.context, term.context());
            if term.kind() == SourcePrimaryTermKind::VariableReference {
                assert_eq!(checked.kind, TermKind::Variable);
                assert_eq!(checked.reference, Some(TermReference::Binding(binding_id)));
            } else {
                assert_eq!(checked.kind, TermKind::Parenthesized);
                assert_eq!(checked.reference, None);
            }
            assert_eq!(checked.status, TermStatus::Inferred);
            assert!(checked.deferred.is_empty() && checked.candidate_set.is_none());
            let entry = inference.type_entries().get(checked.type_entry).unwrap();
            assert_eq!(entry.owner, *term.site());
            assert!(checked_owners.insert(entry.owner.clone()));
            assert!(checked_entries.insert(checked.type_entry));
            assert_eq!(entry.status, TypeStatus::Known);
            assert_eq!(entry.actual, TypeEntryActual::Known(normalized_id));
        }
        assert_eq!(checked_owners.len(), depth + 5);
        assert_eq!(checked_entries.len(), depth + 5);
        assert!(
            inference
                .type_entries()
                .iter()
                .all(|(_, row)| row.status == TypeStatus::Known)
        );
        assert!(
            inference.diagnostics().is_empty()
                && inference.candidate_sets().is_empty()
                && inference.facts().is_empty()
        );
        let mut equalities = ast
            .node_views()
            .filter(|node| node.kind() == &K::BuiltinPredicateApplication)
            .collect::<Vec<_>>();
        equalities.sort_by_key(|node| node.range().start);
        for ((id, formula), equality) in atomic.formulas().iter().zip(equalities) {
            assert_eq!(
                formula.site().node(),
                typed_by_resolved[&source.resolved_node_for(equality.id()).unwrap()]
            );
            assert_eq!(
                (formula.source_range(), formula.source_ordinal()),
                (equality.range(), id.index())
            );
            assert_eq!(formula.kind(), SourceAtomicFormulaKind::Equality);
            let checked = inference
                .formulas()
                .iter()
                .find(|(_, row)| row.site == *formula.site())
                .unwrap()
                .1;
            assert_eq!(checked.status, FormulaStatus::Checked);
            assert!(
                checked.deferred.is_empty()
                    && checked.facts.is_empty()
                    && checked.candidate_set.is_none()
            );
            let edges = atomic
                .edges()
                .iter()
                .filter(|(_, edge)| edge.formula() == id)
                .collect::<Vec<_>>();
            assert_eq!(edges.len(), 2);
            for (ordinal, (_, edge)) in edges.into_iter().enumerate() {
                let operand = ast.node(equality.children()[2 * ordinal]).unwrap().children[0];
                let (term, term_row) = primary
                    .terms()
                    .iter()
                    .find(|(_, row)| {
                        row.site().node()
                            == typed_by_resolved[&source.resolved_node_for(operand).unwrap()]
                    })
                    .unwrap();
                assert_eq!(edge.target(), SourceAtomicTermTarget::Primary(term));
                assert_eq!(edge.ordinal(), ordinal);
                assert_eq!(checked.terms[ordinal], *term_row.site());
                assert_ne!(term, wrapper_id);
                assert!(!(2..depth + 2).contains(&term.index()));
                assert_ne!(term, child_id);
            }
        }
        assert_eq!(
            atomic
                .edges()
                .iter()
                .map(|(_, edge)| match edge.target() {
                    SourceAtomicTermTarget::Primary(id) => id.index(),
                    _ => usize::MAX,
                })
                .collect::<Vec<_>>(),
            [0, 1, depth + 3, depth + 4]
        );
        let mut requested = std::collections::BTreeSet::new();
        for (_, request) in atomic.requests().iter() {
            assert_eq!(request.kind(), SourceAtomicRequestKind::OperandExpectedType);
            let edge_id = request.edge().unwrap();
            assert!(requested.insert(edge_id));
            let edge = atomic.edges().get(edge_id).unwrap();
            assert_eq!(
                (request.formula(), request.ordinal()),
                (edge.formula(), edge.ordinal())
            );
            assert!(
                request.candidate().is_none()
                    && request.type_site().is_none()
                    && request.attribute().is_none()
            );
        }
        assert_eq!(requested, atomic.edges().iter().map(|(id, _)| id).collect());
        assert!(
            base.types().is_empty()
                && base.contexts().is_empty()
                && base.facts().is_empty()
                && base.coercions().is_empty()
                && base.initial_obligations().is_empty()
                && base.diagnostics().is_empty()
        );
        assert!(
            base.source_context().is_none()
                && base.source_type().is_none()
                && base.source_attribute().is_none()
                && base.source_statement().is_none()
                && base.source_statement_witnesses().is_none()
                && base.source_statement_references().is_none()
                && base.source_proof_local_declaration().is_none()
        );
        assert_eq!(base.source_id(), source.source_id());
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
                &frontend.source
            ),
            (
                &before.0,
                &before.1,
                &before.2,
                &before.3,
                &before.4,
                &before.5,
                &before.6,
                before.7.clone(),
                &before.8
            )
        );
    }
    // Every complete base oracle precedes all 24 future-success paired installation attempts.
    let pairs = cases
        .iter()
        .zip(&bases)
        .map(|((_, source, symbols, scope, ..), result)| {
            result
                .as_ref()
                .unwrap()
                .2
                .clone()
                .with_source_statement_from_original(source, symbols, scope)
        })
        .collect::<Vec<_>>();
    let errors = pairs
        .iter()
        .filter_map(|result| result.as_ref().err())
        .collect::<Vec<_>>();
    assert!(
        errors.is_empty(),
        "native nested parenthesized witness original installation: {errors:?}"
    );
    for (
        (
            (frontend, source, symbols, scope, labels, resolution, scopes, parents, before, depth),
            result,
        ),
        pair,
    ) in cases.into_iter().zip(bases).zip(pairs)
    {
        let (bindings, _, base) = result.unwrap();
        let installed = pair.unwrap();
        assert_eq!(installed.nodes(), base.nodes());
        assert_eq!(installed.source_term(), base.source_term());
        assert_eq!(
            installed.source_atomic_formula(),
            base.source_atomic_formula()
        );
        assert!(
            installed.types().is_empty()
                && installed.contexts().is_empty()
                && installed.facts().is_empty()
                && installed.coercions().is_empty()
                && installed.initial_obligations().is_empty()
                && installed.diagnostics().is_empty()
        );
        let statement = installed.source_statement().unwrap();
        let witnesses = installed.source_statement_witnesses().unwrap();
        let primary = base.source_term().unwrap();
        assert_eq!(
            (
                statement.owners().len(),
                statement.statements().len(),
                statement.contexts().len(),
                statement.input_facts().len(),
                statement.candidate_facts().len(),
                witnesses.witnesses().len(),
                witnesses.names().len()
            ),
            (1, 2, 2, 2, 2, 1, 0)
        );
        assert_eq!(statement.binding_env(), &bindings);
        assert!(
            installed.source_statement_references().is_none()
                && installed.source_proof_local_declaration().is_none()
        );
        let ast = frontend.ast.as_ref().unwrap();
        let typed_by_resolved = base
            .nodes()
            .iter()
            .map(|(id, node)| (node.resolved_node.unwrap(), id))
            .collect::<std::collections::BTreeMap<_, _>>();
        let take = ast
            .node_views()
            .find(|node| node.kind() == &K::TakeStatement)
            .unwrap();
        let site = ast.node(take.children()[1]).unwrap();
        let wrapper_site = ast.node(site.children[0]).unwrap().children[0];
        let wrapper = primary.terms().get(SourcePrimaryTermId::new(2)).unwrap();
        let child = primary
            .terms()
            .get(SourcePrimaryTermId::new(depth + 2))
            .unwrap();
        let row = witnesses
            .witnesses()
            .get(SourceStatementWitnessId::new(0))
            .unwrap();
        assert_eq!(
            (
                row.owner().index(),
                row.binding_context().index(),
                row.source_ordinal(),
                row.ordinal()
            ),
            (0, 1, 1, 0)
        );
        assert_eq!(
            row.term(),
            SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2))
        );
        assert_eq!(
            (
                row.take_site().node(),
                row.take_range(),
                row.site().node(),
                row.source_range()
            ),
            (
                typed_by_resolved[&source.resolved_node_for(take.id()).unwrap()],
                take.range(),
                typed_by_resolved[&source.resolved_node_for(take.children()[1]).unwrap()],
                site.range
            )
        );
        assert_eq!(
            wrapper.site().node(),
            typed_by_resolved[&source.resolved_node_for(wrapper_site).unwrap()]
        );
        for offset in 0..depth {
            assert_eq!(
                primary
                    .terms()
                    .get(SourcePrimaryTermId::new(offset + 3))
                    .unwrap()
                    .parent(),
                Some(SourcePrimaryTermId::new(offset + 2))
            );
        }
        assert_eq!(child.source_ordinal(), depth + 2);
        assert_eq!(
            bindings
                .contexts()
                .get(row.binding_context())
                .unwrap()
                .lexical_scope
                .as_ref(),
            scopes.get(&take.id())
        );
        assert_eq!(row.spelling(), wrapper.spelling());
        assert_eq!(row.kind(), SourceStatementWitnessKind::Unnamed);
        assert_eq!(row.recovery(), SourceStatementRecovery::Normal);
        assert!(row.name().is_none());
        assert!(
            !primary
                .references()
                .iter()
                .any(|(_, reference)| reference.term() == SourcePrimaryTermId::new(2))
        );
        let mut sites = ast
            .node_views()
            .filter(|site| matches!(site.kind(), K::TheoremItem | K::ConclusionStatement))
            .collect::<Vec<_>>();
        sites.sort_by_key(|site| site.range().start);
        let (_, owner) = statement.owners().iter().next().unwrap();
        assert_eq!(
            (owner.site().node(), owner.source_range()),
            (
                typed_by_resolved[&source.resolved_node_for(sites[0].id()).unwrap()],
                sites[0].range()
            )
        );
        assert_eq!(owner.spelling(), labels.projections()[0].primary_spelling());
        assert_eq!(owner.contribution(), labels.projections()[0].contribution());
        assert_eq!(owner.symbol(), statement.checked_owner().symbol());
        let atomic = base.source_atomic_formula().unwrap();
        assert_eq!(
            statement
                .statements()
                .get(SourceStatementId::new(0))
                .unwrap()
                .spelling(),
            format!(
                "theorem {} : {} proof take {} ; thus {} ; end ;",
                owner.spelling(),
                atomic
                    .formulas()
                    .get(SourceAtomicFormulaId::new(0))
                    .unwrap()
                    .spelling(),
                row.spelling(),
                atomic
                    .formulas()
                    .get(SourceAtomicFormulaId::new(1))
                    .unwrap()
                    .spelling()
            )
        );
        for (ordinal, ((id, row), site)) in statement.statements().iter().zip(sites).enumerate() {
            assert_eq!(
                (id.index(), row.context().index(), row.source_ordinal()),
                (ordinal, ordinal, [0, 2][ordinal])
            );
            assert_eq!(
                (row.site().node(), row.source_range()),
                (
                    typed_by_resolved[&source.resolved_node_for(site.id()).unwrap()],
                    site.range()
                )
            );
            assert_eq!(
                row.formula(),
                SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(ordinal))
            );
            let context = statement.contexts().get(row.context()).unwrap();
            assert_eq!(context.visible_bindings(), &[BindingId::new(0)]);
            let (_, fact) = statement.input_facts().iter().nth(ordinal).unwrap();
            assert_eq!(
                (
                    fact.statement(),
                    fact.context(),
                    fact.ordinal(),
                    fact.binding().index()
                ),
                (id, row.context(), 0, 0)
            );
            assert_eq!(fact.kind(), SourceStatementInputFactKind::ReservedTypeGuard);
            assert_eq!(
                fact.uses().iter().map(|id| id.index()).collect::<Vec<_>>(),
                if ordinal == 0 { vec![0, 1] } else { vec![3, 4] }
            );
            let (_, candidate) = statement.candidate_facts().iter().nth(ordinal).unwrap();
            assert_eq!(
                (
                    candidate.statement(),
                    candidate.context(),
                    candidate.ordinal(),
                    candidate.formula()
                ),
                (id, row.context(), 0, row.formula())
            );
            assert_eq!(
                candidate.kind(),
                SourceStatementCandidateFactKind::UnverifiedProposition
            );
        }
        assert_eq!(
            base.clone()
                .with_source_statement_from_original(&source, &symbols, &scope),
            Ok(installed.clone())
        );
        let resolved = assemble_empty_resolved_typed_ast(&installed, Vec::new())?;
        assert_eq!(
            assemble_empty_resolved_typed_ast(&installed.clone(), Vec::new())?,
            resolved
        );
        assert_eq!(resolved.source_statement(), Some(statement));
        assert_eq!(resolved.source_statement_witnesses(), Some(witnesses));
        assert!(
            resolved.source_statement_references().is_none()
                && resolved.source_proof_local_declaration().is_none()
        );
        assert_eq!(resolved.nodes().len(), 49 + 4 * depth);
        assert_eq!(
            resolved
                .nodes()
                .node(resolved.nodes().root().unwrap())
                .unwrap()
                .typed_node,
            base.nodes().root().unwrap()
        );
        for (_, node) in resolved.nodes().iter() {
            let original = base.nodes().node(node.typed_node).unwrap();
            assert_eq!(
                mizar_session::SourceAnchor::Range(node.source_range),
                original.anchor
            );
            assert_eq!(
                node.children
                    .iter()
                    .map(|child| resolved.nodes().node(*child).unwrap().typed_node)
                    .collect::<Vec<_>>(),
                original.children
            );
            assert_eq!(
                node.recovery,
                mizar_checker::resolved_typed_ast::ResolvedNodeRecovery::Normal
            );
            assert!(
                node.final_type.is_none() && node.metadata.is_none() && node.diagnostics.is_empty()
            );
        }
        assert!(
            resolved.expr_metadata().is_empty()
                && resolved.collection_candidates().is_empty()
                && resolved.expanded_candidates().is_empty()
                && resolved.template_expansions().is_empty()
                && resolved.viable_candidates().is_empty()
                && resolved.viability_decisions().is_empty()
                && resolved.specificity_graphs().is_empty()
                && resolved.resolved_overloads().is_empty()
                && resolved.inserted_coercions().is_empty()
                && resolved.cluster_facts().is_empty()
                && resolved.diagnostics().is_empty()
                && resolved.checked_formulas().is_empty()
                && resolved.statement_semantics().is_empty()
                && resolved.checked_proofs().is_empty()
                && resolved.checked_proof_nodes().is_empty()
                && resolved.checked_terminal_goals().is_empty()
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
fn source_nested_parenthesized_witness_statement_original_capture_preserves_boundary()
-> Result<(), String> {
    use super::super::super::type_elaboration::assemble_empty_resolved_typed_ast;
    use mizar_checker::{
        source_statement::*,
        type_checker::{SourceVariableSemanticsChecker, TermFormulaChecker},
        typed_ast::*,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        env::*,
        resolved_ast::*,
        symbols::{
            source_proof_inputs, source_proof_scope_maps, source_theorem_module_original_inputs,
            validate_source_proof_occurrences,
        },
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SnapshotInput, SnapshotRegistry, SourceInput, SourceOriginInput,
        SourceVersion, ToolchainInfo, WorkspaceRoot, hash_text,
    };
    use mizar_syntax::SurfaceNodeKind as K;
    let text = "reserve x for set;\ntheorem FormulaStatementNestedParenthesizedWitnessSmoke: x = x proof\n  take ((x));\n  thus x = x;\nend;\n";
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src/nestedparenthesized.miz");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let capture = |current: &str, buffer: bool| {
        fs::write(&path, if buffer { "stale disk" } else { current }).unwrap();
        Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: snapshot_id(778),
                input: SourceInput {
                    package_id: PackageId::new("nested-parenthesized-witness-boundary"),
                    module_path: ModulePath::new("nestedparenthesized"),
                    normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2026"),
                    origin: if buffer {
                        SourceOriginInput::OpenBuffer {
                            uri: format!("file://{}", path.display()),
                            expected_version: 3,
                            actual_version: 3,
                            text: current.to_owned().into(),
                        }
                    } else {
                        SourceOriginInput::Disk { path: path.clone() }
                    },
                },
            },
            &ids,
        )
        .unwrap()
    };
    let frontend = capture(text, false);
    assert!(frontend.diagnostics.is_empty() && frontend.preprocessed.import_stubs.is_empty());
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
        (57, 57, 56)
    );
    assert_eq!(
        scope
            .references()
            .iter()
            .map(|row| row.ordinal())
            .collect::<Vec<_>>(),
        [0, 1, 2, 3, 4]
    );
    assert!(symbols.labels().is_empty());
    assert_eq!(
        (labels.projections().len(), labels.references().len()),
        (1, 0)
    );
    let (_, inference, base) =
        TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope)?;
    let installed = base
        .clone()
        .with_source_statement_from_original(&source, &symbols, &scope)
        .unwrap();
    let resolved = assemble_empty_resolved_typed_ast(&installed, Vec::new())?;
    let statement = installed.source_statement().unwrap();
    let witnesses = installed.source_statement_witnesses().unwrap();
    assert_eq!(
        (
            base.nodes().len(),
            witnesses.witnesses().len(),
            witnesses.names().len()
        ),
        (57, 1, 0)
    );
    assert_eq!(resolved.source_statement(), Some(statement));
    assert_eq!(resolved.source_statement_witnesses(), Some(witnesses));
    assert!(
        installed.source_proof_local_declaration().is_none()
            && resolved.source_proof_local_declaration().is_none()
    );
    assert!(
        installed.source_statement_references().is_none()
            && resolved.source_statement_references().is_none()
    );
    assert!(
        installed.types().is_empty()
            && installed.facts().is_empty()
            && installed.contexts().is_empty()
            && installed.coercions().is_empty()
            && installed.initial_obligations().is_empty()
            && installed.diagnostics().is_empty()
    );
    assert!(
        resolved.expr_metadata().is_empty()
            && resolved.checked_formulas().is_empty()
            && resolved.statement_semantics().is_empty()
            && resolved.checked_proofs().is_empty()
            && resolved.checked_proof_nodes().is_empty()
            && resolved.checked_terminal_goals().is_empty()
    );
    let snapshot_inputs = || {
        (
            source.clone(),
            symbols.clone(),
            scope.clone(),
            labels.clone(),
            resolution.clone(),
            scopes.clone(),
            parents.clone(),
            (
                inference.clone(),
                base.clone(),
                installed.clone(),
                resolved.clone(),
            ),
            ast.snapshot_text(),
            frontend.source.clone(),
        )
    };
    let before = snapshot_inputs();
    let pristine = || {
        assert_eq!(snapshot_inputs(), before);
        let replay = base
            .clone()
            .with_source_statement_from_original(&source, &symbols, &scope)
            .unwrap();
        assert_eq!(replay, installed);
        assert_eq!(replay.debug_text(), installed.debug_text());
        assert_eq!(
            assemble_empty_resolved_typed_ast(&replay, Vec::new()).unwrap(),
            resolved
        );
        assert!(!root.join("proof-witnesses").exists());
    };
    let rejected_pair = |result: Result<TypedAst, TypedAstError>| {
        assert_eq!(result, Err(TypedAstError::InvalidSourceStatement));
        pristine();
    };
    let unsupported = "theorems.equality.unsupported_source_types";
    // Higher owners are tested only after genuine base, pair and final success.
    assert_eq!(
        SourceVariableSemanticsChecker::source_theorem_binding_env(&source, &symbols, &scope)
            .unwrap_err(),
        unsupported
    );
    pristine();
    assert_eq!(
        TermFormulaChecker::check_source_theorem_equalities_with_typed_tables(
            &source, &symbols, &scope
        )
        .unwrap_err(),
        unsupported
    );
    pristine();
    assert_eq!(
        TermFormulaChecker::check_source_theorem_equalities_with_expression_metadata(
            &source, &symbols, &scope
        )
        .unwrap_err(),
        unsupported
    );
    pristine();
    let policy = mizar_proof::policy::VerifierPolicy::default();
    let registry = SnapshotRegistry::new();
    let unit = &frontend.source;
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
                toolchain: ToolchainInfo::new("native-nested-parenthesized-witness-boundary"),
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
    pristine();

    let owner = symbols
        .symbols()
        .iter()
        .find(|entry| entry.kind() == SymbolKind::Theorem)
        .unwrap();
    assert!(symbols.labels().is_empty());
    let mut import_nodes = ResolvedArenaBuilder::new();
    let import_owner = import_nodes
        .push(ResolvedNode::new(
            K::ImportAliasDecl,
            Vec::new(),
            owner.origin().clone(),
        ))
        .unwrap();
    let mizar_session::SourceAnchor::Range(import_range) = owner.origin().anchor() else {
        panic!("actual owner range")
    };
    let mut imports = ResolvedImports::new();
    let import = imports.push_import(ResolvedImport::new(
        import_owner,
        *import_range,
        "import boundary.fixture;",
        None,
        ImportResolution::Resolved(module.clone()),
        owner.origin().clone(),
    ));
    let env_error = "source symbol environment does not match its declaration collection";
    for mutation in [
        "missing",
        "second-theorem",
        "path",
        "kind",
        "visibility",
        "export",
        "recovered",
        "import",
        "extra-y",
    ] {
        let mut entries = SymbolIndex::new();
        for entry in symbols.symbols().iter() {
            if entry.symbol() != owner.symbol() {
                entries.insert(entry.clone());
                continue;
            }
            if mutation == "missing" {
                continue;
            }
            let mut origin = entry.origin().clone();
            if mutation == "path" {
                let mut path = origin.structural_path().to_vec();
                path.push(99);
                origin = SemanticOrigin::new(
                    origin.source_id(),
                    origin.module_id().clone(),
                    origin.anchor().clone(),
                    path,
                );
            } else if mutation == "recovered" {
                origin = origin.recovered();
            } else if mutation == "import" {
                origin = origin.with_import_edge(import);
            }
            let mut replacement = SymbolEntry::new(
                entry.symbol().clone(),
                if mutation == "kind" {
                    SymbolKind::Builtin
                } else {
                    entry.kind()
                },
                entry.namespace().clone(),
                entry.primary_spelling(),
                origin,
                entry.contribution(),
            )
            .with_visibility(if mutation == "visibility" {
                Visibility::Private
            } else {
                entry.visibility()
            })
            .with_export_status(if mutation == "export" {
                ExportStatus::LocalOnly
            } else {
                entry.export_status()
            })
            .with_relations(entry.relations().to_vec());
            if let Some(notation) = entry.notation_spelling() {
                replacement = replacement.with_notation_spelling(notation);
            }
            if let Some(signature) = entry.signature() {
                replacement = replacement.with_signature(signature.clone());
            }
            entries.insert(replacement);
        }
        if ["second-theorem", "extra-y"].contains(&mutation) {
            let spelling = if mutation == "extra-y" {
                "y"
            } else {
                "OtherTheorem"
            };
            let symbol = SymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("boundary/{spelling}")),
                FullyQualifiedName::new(format!("{}::{spelling}", module.path().as_str())),
            );
            assert!(symbols.symbols().get(&symbol).is_none());
            entries.insert(
                SymbolEntry::new(
                    symbol,
                    if mutation == "extra-y" {
                        SymbolKind::Builtin
                    } else {
                        SymbolKind::Theorem
                    },
                    owner.namespace().clone(),
                    spelling,
                    owner.origin().clone(),
                    owner.contribution(),
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        let mutated = SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                imports: symbols.imports().clone(),
                exports: symbols.exports().clone(),
                symbols: entries,
                labels: symbols.labels().clone(),
                definitions: symbols.definitions().clone(),
                overloads: symbols.overloads().clone(),
                registrations: symbols.registrations().clone(),
                lexical_summaries: symbols.lexical_summaries().clone(),
                namespace_graph: symbols.namespace_graph().clone(),
                declaration_dependencies: symbols.declaration_dependencies().clone(),
                contributions: symbols.contributions().clone(),
                module_summaries: symbols.module_summaries().clone(),
            },
        );
        assert_ne!(mutated, symbols, "actual env mutation {mutation}");
        let mutated_before = mutated.clone();
        assert_eq!(
            source_proof_inputs(&source, &mutated).unwrap_err(),
            env_error,
            "proof inputs {mutation}"
        );
        assert_eq!(mutated, mutated_before);
        pristine();
        assert_eq!(
            TermFormulaChecker::check_source_theorem_equalities(&source, &mutated, &scope)
                .unwrap_err(),
            env_error,
            "base authentication {mutation}"
        );
        assert_eq!(mutated, mutated_before);
        pristine();
        assert_eq!(
            base.clone()
                .with_source_statement_from_original(&source, &mutated, &scope),
            Err(TypedAstError::InvalidSourceStatement),
            "pair authentication {mutation}"
        );
        assert_eq!(mutated, mutated_before);
        pristine();
    }

    // Establish successful public reconstruction before all 57 x 5 mutations.
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
    assert_eq!(
        reconstruct(
            nodes.clone(),
            base.resolved_root(),
            base.nodes().root(),
            true,
            false
        )
        .unwrap(),
        base
    );
    assert_eq!(
        reconstruct(
            nodes.clone(),
            base.resolved_root(),
            base.nodes().root(),
            true,
            false
        )
        .unwrap()
        .with_source_statement_from_original(&source, &symbols, &scope),
        Ok(installed.clone())
    );
    let unowned_tokens = ast
        .node_views()
        .filter(|site| {
            matches!(site.kind(), K::Token(_))
                && !base
                    .source_term()
                    .unwrap()
                    .terms()
                    .iter()
                    .any(|(_, row)| row.site().node().index() == site.id().index())
                && !base
                    .source_atomic_formula()
                    .unwrap()
                    .formulas()
                    .iter()
                    .any(|(_, row)| row.site().node().index() == site.id().index())
        })
        .map(|site| site.id().index())
        .collect::<std::collections::BTreeSet<_>>();
    for wrapper in ast
        .node_views()
        .filter(|site| site.kind() == &K::ParenthesizedTerm)
    {
        assert!(unowned_tokens.contains(&wrapper.children()[0].index()));
        assert!(unowned_tokens.contains(&wrapper.children().last().unwrap().index()));
    }
    let mut attempts = 0;
    let mut canonical_tokens = 0;
    for node in 0..nodes.len() {
        for mutation in ["anchor-end", "recovered", "degraded", "kind", "children"] {
            let mut current = nodes.clone();
            match mutation {
                "anchor-end" => {
                    let mizar_session::SourceAnchor::Range(mut range) =
                        current[node].anchor.clone()
                    else {
                        panic!("source range")
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
            assert_ne!(current, nodes);
            let mutated_before = current.clone();
            let stage = match reconstruct(
                current.clone(),
                base.resolved_root(),
                base.nodes().root(),
                true,
                false,
            ) {
                Ok(rebuilt) => {
                    let rebuilt_before = rebuilt.clone();
                    assert_eq!(
                        rebuilt
                            .clone()
                            .with_source_statement_from_original(&source, &symbols, &scope),
                        Err(TypedAstError::InvalidSourceStatement),
                        "node={node} mutation={mutation}"
                    );
                    assert_eq!(rebuilt, rebuilt_before);
                    "canonical"
                }
                Err((stage, error)) => {
                    match stage {
                        "primary" => assert_eq!(error, TypedAstError::InvalidSourceTerm),
                        "atomic" => {
                            assert_eq!(error, TypedAstError::InvalidSourceAtomicFormula)
                        }
                        _ => panic!(
                            "unexpected stage node={node} mutation={mutation}: {stage} {error:?}"
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
                canonical_tokens += 1;
            }
            assert_eq!(current, mutated_before);
            attempts += 1;
            pristine();
        }
    }
    assert_eq!(attempts, 285);
    assert_eq!(canonical_tokens, unowned_tokens.len() * 5);

    let raw_input = SourceStatementWitnessHandoffInput {
        source_id: base.source_id(),
        module_id: module.clone(),
        witnesses: witnesses
            .witnesses()
            .iter()
            .map(|(_, row)| SourceStatementWitnessInput {
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
            })
            .collect(),
        names: Vec::new(),
    };
    assert!(!inference.type_entries().is_empty());
    for control in [
        "rooted",
        "rootless",
        "resolved-root-only",
        "typed-root-only",
        "enriched",
        "missing",
        "missing-atomic",
    ] {
        let reconstructed = reconstruct(
            nodes.clone(),
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
        let current_before = reconstructed.clone();
        let raw_before = raw_input.clone();
        assert_eq!(
            SourceStatementWitnessProducer::build(
                raw_input.clone(),
                statement,
                base.source_term().unwrap(),
                reconstructed.nodes()
            ),
            Err(SourceStatementWitnessError::DependencyMismatch),
            "raw {control}"
        );
        assert_eq!(raw_input, raw_before);
        assert_eq!(reconstructed, current_before);
        pristine();
        assert_eq!(
            reconstructed
                .clone()
                .with_source_statement_from_original(&source, &symbols, &scope),
            if control == "rooted" {
                Ok(installed.clone())
            } else {
                Err(TypedAstError::InvalidSourceStatement)
            },
            "original {control}"
        );
        assert_eq!(reconstructed, current_before);
        pristine();
        assert_eq!(
            reconstructed
                .clone()
                .with_source_statement(statement.clone()),
            Err(TypedAstError::InvalidSourceStatement),
            "missing companion {control}"
        );
        assert_eq!(reconstructed, current_before);
        pristine();
        assert_eq!(
            reconstructed
                .clone()
                .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
            Err(TypedAstError::InvalidSourceStatement),
            "detached {control}"
        );
        assert_eq!(reconstructed, current_before);
        assert_eq!(raw_input, raw_before);
        pristine();
    }
    rejected_pair(
        installed
            .clone()
            .with_source_statement_from_original(&source, &symbols, &scope),
    );

    let foreign_frontend = capture(
        &text
            .replace(
                "NestedParenthesizedWitnessSmoke",
                "ForeignNestedParenthesized",
            )
            .replace("take ((x));", "take    ( ( x ) );"),
        false,
    );
    assert!(foreign_frontend.diagnostics.is_empty());
    let foreign_ast = foreign_frontend.ast.as_ref().unwrap();
    let (foreign, foreign_env) =
        source_theorem_module_original_inputs(foreign_ast, &module)?.unwrap();
    let (foreign_scope, foreign_labels, foreign_resolution) =
        source_proof_inputs(&foreign, &foreign_env)?;
    assert_ne!(foreign.source_id(), source.source_id());
    assert_ne!(foreign, source);
    assert_ne!(foreign_env, symbols);
    assert_ne!(foreign_scope, scope);
    let foreign_base = TermFormulaChecker::check_source_theorem_equalities(
        &foreign,
        &foreign_env,
        &foreign_scope,
    )?
    .2;
    let foreign_pair = foreign_base
        .clone()
        .with_source_statement_from_original(&foreign, &foreign_env, &foreign_scope)
        .unwrap();
    let foreign_final = assemble_empty_resolved_typed_ast(&foreign_pair, Vec::new())?;
    assert_eq!(
        (
            foreign_pair
                .source_statement_witnesses()
                .unwrap()
                .witnesses()
                .len(),
            foreign_pair
                .source_statement_witnesses()
                .unwrap()
                .names()
                .len()
        ),
        (1, 0)
    );
    assert!(
        foreign_pair.source_proof_local_declaration().is_none()
            && foreign_final.source_proof_local_declaration().is_none()
    );
    let snapshot_foreign = || {
        (
            foreign.clone(),
            foreign_env.clone(),
            foreign_scope.clone(),
            foreign_labels.clone(),
            foreign_resolution.clone(),
            foreign_base.clone(),
            foreign_pair.clone(),
            foreign_final.clone(),
            foreign_ast.snapshot_text(),
            foreign_frontend.source.clone(),
        )
    };
    let foreign_before = snapshot_foreign();
    let foreign_pristine = || {
        assert_eq!(snapshot_foreign(), foreign_before);
        assert_eq!(
            foreign_base.clone().with_source_statement_from_original(
                &foreign,
                &foreign_env,
                &foreign_scope
            ),
            Ok(foreign_pair.clone())
        );
        assert_eq!(
            assemble_empty_resolved_typed_ast(&foreign_pair, Vec::new()).unwrap(),
            foreign_final
        );
        pristine();
    };
    for (current, env, occurrences) in [
        (&foreign, &symbols, &scope),
        (&source, &foreign_env, &scope),
        (&source, &symbols, &foreign_scope),
        (&foreign, &foreign_env, &foreign_scope),
    ] {
        rejected_pair(
            base.clone()
                .with_source_statement_from_original(current, env, occurrences),
        );
        foreign_pristine();
    }
    rejected_pair(
        foreign_base
            .clone()
            .with_source_statement_from_original(&source, &symbols, &scope),
    );
    foreign_pristine();
    let empty = reconstruct(
        nodes.clone(),
        base.resolved_root(),
        base.nodes().root(),
        false,
        false,
    )
    .unwrap();
    assert_eq!(
        empty
            .clone()
            .with_source_term(foreign_base.source_term().unwrap().clone()),
        Err(TypedAstError::InvalidSourceTerm)
    );
    foreign_pristine();
    assert_eq!(
        empty
            .with_source_term(base.source_term().unwrap().clone())
            .unwrap()
            .with_source_atomic_formula(foreign_base.source_atomic_formula().unwrap().clone()),
        Err(TypedAstError::InvalidSourceAtomicFormula)
    );
    foreign_pristine();
    for (statements, companion) in [
        (
            statement.clone(),
            foreign_pair.source_statement_witnesses().unwrap().clone(),
        ),
        (
            foreign_pair.source_statement().unwrap().clone(),
            witnesses.clone(),
        ),
        (
            foreign_pair.source_statement().unwrap().clone(),
            foreign_pair.source_statement_witnesses().unwrap().clone(),
        ),
    ] {
        rejected_pair(
            base.clone()
                .with_source_statement_witnesses(statements, companion),
        );
        foreign_pristine();
    }
    foreign_pristine();
    // Triple grouping has its own native base; other mutations remain bounded-source refusals.
    for (case, current) in [
        ("triple", text.replace("take ((x));", "take (((x)));")),
        (
            "triple-buffer",
            text.replace("take ((x));", "take (((x)));"),
        ),
        ("numeral", text.replace("take ((x));", "take ((101));")),
        ("named", text.replace("take ((x));", "take y = ((x));")),
        ("multiple", text.replace("take ((x));", "take ((x)), x;")),
        (
            "theorem-group",
            text.replace(": x = x proof", ": (x) = x proof"),
        ),
        (
            "conclusion-group",
            text.replace("thus x = x;", "thus (x) = x;"),
        ),
        ("alias-rhs", text.replace("take ((x));", "take ((y));")),
        (
            "extra-take",
            text.replace("take ((x));", "take ((x)); take x;"),
        ),
        (
            "extra-conclusion",
            text.replace("thus x = x;", "thus x = x; thus x = x;"),
        ),
        (
            "second-reserve",
            text.replace("reserve x for set;", "reserve x, y for set;"),
        ),
        ("recovery", text.replace("take ((x));", "take ((x))")),
    ] {
        assert_ne!(current, text, "{case} source mutation");
        let negative = capture(&current, case == "triple-buffer");
        if case == "recovery" {
            assert!(negative.diagnostics.iter().any(|diagnostic| matches!(&diagnostic.code, mizar_frontend::orchestration::DiagnosticCode::Syntax(code) if code.as_ref() == "missing_semicolon")));
        } else {
            assert!(
                negative.diagnostics.is_empty(),
                "{case}: {:?}",
                negative.diagnostics
            );
        }
        assert_eq!(negative.source.source_text.as_ref(), current);
        let negative_ast = negative.ast.as_ref().unwrap();
        if case.starts_with("triple") {
            assert_eq!(negative_ast.nodes().len(), 61);
            assert_eq!(
                negative_ast
                    .node_views()
                    .filter(|node| node.kind() == &K::ParenthesizedTerm)
                    .count(),
                3
            );
        }
        let (original, env) =
            source_theorem_module_original_inputs(negative_ast, &module)?.unwrap();
        let snapshot_negative = || {
            (
                original.clone(),
                env.clone(),
                negative_ast.snapshot_text(),
                negative.source.clone(),
            )
        };
        let negative_before = snapshot_negative();
        if case.starts_with("alias-") {
            assert_eq!(
                source_proof_inputs(&original, &env).unwrap_err(),
                "source proof occurrences do not match the original source",
                "{case}"
            );
        } else {
            let (occurrences, collected, resolution) = source_proof_inputs(&original, &env)?;
            let inputs_before = (occurrences.clone(), collected.clone(), resolution.clone());
            let lower =
                TermFormulaChecker::check_source_theorem_equalities(&original, &env, &occurrences);
            if case.starts_with("triple") {
                let (_, inference, own_base) = lower.unwrap();
                let primary = own_base.source_term().unwrap();
                assert_eq!(
                    (
                        own_base.nodes().len(),
                        primary.terms().len(),
                        primary.references().len()
                    ),
                    (61, 8, 5)
                );
                assert_eq!(
                    primary
                        .references()
                        .iter()
                        .map(|(_, row)| row.term().index())
                        .collect::<Vec<_>>(),
                    [0, 1, 5, 6, 7]
                );
                assert_eq!(
                    primary
                        .references()
                        .iter()
                        .map(|(_, row)| row.use_ordinal())
                        .collect::<Vec<_>>(),
                    [0, 1, 2, 3, 4]
                );
                assert_eq!(inference.terms().len(), 8);
                assert!(inference.terms().iter().all(|(_, row)| row.status
                    == mizar_checker::type_checker::TermStatus::Inferred
                    && row.deferred.is_empty()));
                assert_eq!(inference.formulas().len(), 2);
                assert!(inference.formulas().iter().all(|(_, row)| row.status
                    == mizar_checker::type_checker::FormulaStatus::Checked
                    && row.deferred.is_empty()));
                assert!(
                    inference.diagnostics().is_empty()
                        && inference.candidate_sets().is_empty()
                        && inference.facts().is_empty()
                );
                let written = negative_ast
                    .node_views()
                    .find(|node| node.kind() == &K::TypeExpression)
                    .unwrap();
                let written_node = own_base
                    .nodes()
                    .iter()
                    .find(|(_, node)| {
                        node.resolved_node == original.resolved_node_for(written.id())
                    })
                    .unwrap()
                    .0;
                assert_eq!(inference.normalized_types().len(), 1);
                let (normalized_id, normalized) =
                    inference.normalized_types().iter().next().unwrap();
                assert_eq!(
                    normalized.head,
                    mizar_checker::type_checker::TypeHeadRef::BuiltinSet
                );
                assert_eq!(normalized.source.range, written.range());
                let mut roles = std::collections::BTreeSet::new();
                for (_, entry) in inference.type_entries().iter() {
                    assert_eq!(entry.status, TypeStatus::Known);
                    assert_eq!(entry.actual, TypeEntryActual::Known(normalized_id));
                    if let TypedSiteRef::Role { node, role } = &entry.owner {
                        assert_eq!(*node, written_node);
                        assert!(roles.insert(role.as_str().to_owned()));
                    }
                }
                assert_eq!(roles.len(), 8);
                let own_pair = own_base
                    .with_source_statement_from_original(&original, &env, &occurrences)
                    .unwrap();
                let own_final = assemble_empty_resolved_typed_ast(&own_pair, Vec::new())?;
                assert_eq!(own_final.nodes().len(), 61);
                assert_eq!(own_final.source_statement(), own_pair.source_statement());
                assert_eq!(
                    own_final.source_statement_witnesses(),
                    own_pair.source_statement_witnesses()
                );
                assert_eq!(
                    assemble_empty_resolved_typed_ast(&own_pair.clone(), Vec::new())?,
                    own_final
                );
            } else {
                assert_eq!(lower.unwrap_err(), unsupported, "{case}");
            }
            assert_eq!(snapshot_negative(), negative_before);
            assert_eq!(
                (&occurrences, &collected, &resolution),
                (&inputs_before.0, &inputs_before.1, &inputs_before.2)
            );
            pristine();
            assert_eq!(
                base.clone()
                    .with_source_statement_from_original(&original, &env, &occurrences),
                Err(TypedAstError::InvalidSourceStatement),
                "{case}"
            );
            assert_eq!((occurrences, collected, resolution), inputs_before);
        }
        assert_eq!(snapshot_negative(), negative_before);
        pristine();
    }
    assert_eq!(registry.get(snapshot.id), Some(snapshot.clone()));
    registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    fs::remove_dir_all(root).unwrap();
    Ok(())
}
