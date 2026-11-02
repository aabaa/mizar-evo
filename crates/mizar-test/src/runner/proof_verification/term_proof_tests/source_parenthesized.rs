use super::*;

#[test]
fn source_parenthesized_witness_statement_original_capture_transports_native_pair()
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
    let original = "reserve x for set;\ntheorem FormulaStatementParenthesizedWitnessSmoke: x = x proof\n  take (x);\n  thus x = x;\nend;\n";
    let texts = [original.to_owned(), original.replace("FormulaStatementParenthesizedWitnessSmoke", "RenamedParenthesis").replace("x", "parcel"), "reserve\n parcel\tfor :: type hint\n set;\ntheorem LayoutParenthesis: parcel = parcel proof :: syntax only\n take ( parcel ); thus parcel = parcel; end;".to_owned()];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src/parenthesized.miz");
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
                        package_id: PackageId::new("parenthesized-witness-test"),
                        module_path: ModulePath::new("parenthesized"),
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
                (53, 53, 52)
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
                (K::BuiltinPredicateApplication, 2),
                (K::ProofBlock, 1),
                (K::TakeStatement, 1),
                (K::Witness, 1),
                (K::ParenthesizedTerm, 1),
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
            let expression = ast.node(witness.children[0]).unwrap();
            assert_eq!(expression.kind, K::TermExpression);
            let [wrapper_id] = expression.children.as_slice() else {
                panic!("one witness operand")
            };
            let wrapper = ast.node(*wrapper_id).unwrap();
            assert_eq!(wrapper.kind, K::ParenthesizedTerm);
            assert_eq!(wrapper.children.len(), 3);
            assert_eq!(
                ast.node(wrapper.children[0]).unwrap().token_text(),
                Some("(")
            );
            assert_eq!(
                ast.node(wrapper.children[2]).unwrap().token_text(),
                Some(")")
            );
            let child_expression = ast.node(wrapper.children[1]).unwrap();
            assert_eq!(child_expression.kind, K::TermExpression);
            let [child_id] = child_expression.children.as_slice() else {
                panic!("one inner reference")
            };
            assert_eq!(ast.node(*child_id).unwrap().kind, K::TermReference);
            let reference = scope
                .references()
                .iter()
                .find(|row| row.node() == *child_id)
                .unwrap();
            assert_eq!(reference.ordinal(), 2);
            assert_eq!(reference.spelling(), scope.bindings()[0].spelling());
            assert!(
                !scope
                    .references()
                    .iter()
                    .any(|row| row.node() == *wrapper_id)
            );
            assert_eq!(parents[wrapper_id], witness.children[0]);
            assert_eq!(parents[child_id], wrapper.children[1]);
            assert_eq!(parents[&wrapper.children[1]], *wrapper_id);
            assert_eq!(scopes[wrapper_id], scopes[&proof.id()]);
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
            ));
        }
    }
    assert_eq!(cases.len(), 6);
    // First runtime RED follows readiness of every genuine capture and all six real lower calls.
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
        "native parenthesized witness equality bases: {errors:?}"
    );
    for ((frontend, source, _, scope, _, _, scopes, parents, _), result) in cases.iter().zip(&bases)
    {
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
            [2, 1, 6, 5, 0, 2, 4, 4, 6, 2]
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
        let wrapper_site = ast
            .node_views()
            .find(|node| node.kind() == &K::ParenthesizedTerm)
            .unwrap();
        let child_site = ast.node(wrapper_site.children()[1]).unwrap().children[0];
        let wrapper_id = SourcePrimaryTermId::new(2);
        let child_id = SourcePrimaryTermId::new(3);
        let wrapper = primary.terms().get(wrapper_id).unwrap();
        let child = primary.terms().get(child_id).unwrap();
        assert_eq!(wrapper.site().node().index(), wrapper_site.id().index());
        assert_eq!(wrapper.source_range(), wrapper_site.range());
        assert_eq!(wrapper.kind(), SourcePrimaryTermKind::Parenthesized);
        assert_eq!(wrapper.parent(), None);
        assert_eq!(child.site().node().index(), child_site.index());
        assert_eq!(child.parent(), Some(wrapper_id));
        assert_eq!(wrapper.context(), child.context());
        assert_eq!(wrapper.context().index(), 1);
        assert_eq!(wrapper.spelling(), format!("( {} )", child.spelling()));
        assert!(
            !primary
                .references()
                .iter()
                .any(|(_, row)| row.term() == wrapper_id)
        );
        for ((id, row), ordinal) in primary.terms().iter().zip(0..6) {
            assert_eq!(row.source_ordinal(), ordinal);
            assert_eq!(row.role(), SourcePrimaryTermRole::Value);
            assert_eq!(row.recovery(), SourcePrimaryTermRecovery::Normal);
            assert_eq!(
                row.parent(),
                if id == child_id {
                    Some(wrapper_id)
                } else {
                    None
                }
            );
        }
        for ((reference_id, reference), occurrence) in
            primary.references().iter().zip(scope.references())
        {
            let target = [0, 1, 3, 4, 5][reference_id.index()];
            assert_eq!(reference.term().index(), target);
            let term = primary.terms().get(reference.term()).unwrap();
            assert_eq!(term.site().node().index(), occurrence.node().index());
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
        assert_eq!(child.source_ordinal(), 3);
        // Checked rows are joined by actual site, never by source-primary or reference IDs.
        let written_type = ast
            .node_views()
            .find(|node| node.kind() == &K::TypeExpression)
            .unwrap();
        let wrapper_checked = inference
            .terms()
            .iter()
            .find(|(_, term)| term.site == *wrapper.site())
            .unwrap()
            .1;
        let child_checked = inference
            .terms()
            .iter()
            .find(|(_, term)| term.site == *child.site())
            .unwrap()
            .1;
        assert_eq!(wrapper_checked.kind, TermKind::Parenthesized);
        assert_eq!(wrapper_checked.reference, None);
        assert_eq!(child_checked.kind, TermKind::Variable);
        assert_eq!(
            child_checked.reference,
            Some(TermReference::Binding(binding_id))
        );
        assert_ne!(wrapper_checked.type_entry, child_checked.type_entry);
        let wrapper_type = inference
            .type_entries()
            .get(wrapper_checked.type_entry)
            .unwrap();
        let child_type = inference
            .type_entries()
            .get(child_checked.type_entry)
            .unwrap();
        assert_eq!(wrapper_type.owner, *wrapper.site());
        assert_eq!(child_type.owner, *child.site());
        assert_eq!(wrapper_type.actual, child_type.actual);
        let TypeEntryActual::Known(normalized_id) = wrapper_type.actual else {
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
        assert_eq!(type_roles.len(), 6);
        for occurrence in scope.references() {
            assert!(type_roles.contains(&format!("theorem.equality.use:{}", occurrence.ordinal())));
        }
        assert!(
            normalized.args.is_empty()
                && normalized.attributes.positive().is_empty()
                && normalized.attributes.negative().is_empty()
        );
        assert_eq!(inference.normalized_types().len(), 1);
        for (_, term) in primary.terms().iter() {
            let checked = inference
                .terms()
                .iter()
                .find(|(_, row)| row.site == *term.site())
                .unwrap()
                .1;
            assert_eq!(checked.context, term.context());
            assert_eq!(checked.status, TermStatus::Inferred);
            assert!(checked.deferred.is_empty() && checked.candidate_set.is_none());
            let entry = inference.type_entries().get(checked.type_entry).unwrap();
            assert_eq!(entry.owner, *term.site());
            assert_eq!(entry.status, TypeStatus::Known);
            assert_eq!(entry.actual, TypeEntryActual::Known(normalized_id));
        }
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
            assert_eq!(formula.site().node().index(), equality.id().index());
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
                    .find(|(_, row)| row.site().node().index() == operand.index())
                    .unwrap();
                assert_eq!(edge.target(), SourceAtomicTermTarget::Primary(term));
                assert_eq!(edge.ordinal(), ordinal);
                assert_eq!(checked.terms[ordinal], *term_row.site());
                assert_ne!(term, wrapper_id);
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
            [0, 1, 4, 5]
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
            base.source_statement().is_none()
                && base.source_statement_references().is_none()
                && base.source_proof_local_declaration().is_none()
        );
        assert_eq!(base.source_id(), source.source_id());
    }
    // Every complete base oracle precedes all six future-success paired installation attempts.
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
        "native parenthesized witness original installation: {errors:?}"
    );
    for (
        ((frontend, source, symbols, scope, labels, resolution, scopes, parents, before), result),
        pair,
    ) in cases.into_iter().zip(bases).zip(pairs)
    {
        let (bindings, _, base) = result.unwrap();
        let installed = pair.unwrap();
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
        let take = ast
            .node_views()
            .find(|node| node.kind() == &K::TakeStatement)
            .unwrap();
        let site = ast.node(take.children()[1]).unwrap();
        let wrapper_site = ast.node(site.children[0]).unwrap().children[0];
        let wrapper = primary.terms().get(SourcePrimaryTermId::new(2)).unwrap();
        let child = primary.terms().get(SourcePrimaryTermId::new(3)).unwrap();
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
                row.take_site().node().index(),
                row.take_range(),
                row.site().node().index(),
                row.source_range()
            ),
            (
                take.id().index(),
                take.range(),
                take.children()[1].index(),
                site.range
            )
        );
        assert_eq!(wrapper.site().node().index(), wrapper_site.index());
        assert_eq!(child.parent(), Some(SourcePrimaryTermId::new(2)));
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
            (owner.site().node().index(), owner.source_range()),
            (sites[0].id().index(), sites[0].range())
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
                (row.site().node().index(), row.source_range()),
                (site.id().index(), site.range())
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
        assert_eq!(resolved.nodes().len(), 53);
        assert!(
            resolved.expr_metadata().is_empty()
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
fn source_parenthesized_witness_statement_original_capture_preserves_boundary() -> Result<(), String>
{
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
        },
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SnapshotInput, SnapshotRegistry, SourceInput, SourceOriginInput,
        SourceVersion, ToolchainInfo, WorkspaceRoot, hash_text,
    };
    use mizar_syntax::SurfaceNodeKind as K;
    let text = "reserve x for set;\ntheorem FormulaStatementParenthesizedWitnessSmoke: x = x proof\n  take (x);\n  thus x = x;\nend;\n";
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src/parenthesized.miz");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let capture = |current: &str| {
        fs::write(&path, current).unwrap();
        Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: snapshot_id(778),
                input: SourceInput {
                    package_id: PackageId::new("parenthesized-witness-boundary"),
                    module_path: ModulePath::new("parenthesized"),
                    normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Disk { path: path.clone() },
                },
            },
            &ids,
        )
        .unwrap()
    };
    let frontend = capture(text);
    assert!(frontend.diagnostics.is_empty() && frontend.preprocessed.import_stubs.is_empty());
    let ast = frontend.ast.as_ref().unwrap();
    let module = ModuleId::new(
        frontend.source.package_id.clone(),
        frontend.source.module_path.clone(),
    );
    let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
    let (scope, labels, resolution) = source_proof_inputs(&source, &symbols)?;
    let (scopes, parents) = source_proof_scope_maps(&source, &symbols, &scope)?;
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
        (53, 1, 0)
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
                toolchain: ToolchainInfo::new("native-parenthesized-witness-boundary"),
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

    // Establish successful public reconstruction before all 53 x 5 mutations.
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
    let wrapper = ast
        .node_views()
        .find(|site| site.kind() == &K::ParenthesizedTerm)
        .unwrap();
    assert!(unowned_tokens.contains(&wrapper.children()[0].index()));
    assert!(unowned_tokens.contains(&wrapper.children().last().unwrap().index()));
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
    assert_eq!(attempts, 265);
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
            .replace("ParenthesizedWitnessSmoke", "ForeignParenthesized")
            .replace("take (x);", "take    ( x );"),
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
    // These are unsupported bounded-source domains, not claims of language invalidity.
    for (case, current) in [
        ("nested", text.replace("take (x);", "take ((x));")),
        ("numeral", text.replace("take (x);", "take (101);")),
        ("named", text.replace("take (x);", "take y = (x);")),
        ("multiple", text.replace("take (x);", "take (x), x;")),
        (
            "theorem-group",
            text.replace(": x = x proof", ": (x) = x proof"),
        ),
        (
            "conclusion-group",
            text.replace("thus x = x;", "thus (x) = x;"),
        ),
        ("alias-rhs", text.replace("take (x);", "take (y);")),
        ("extra-take", text.replace("take (x);", "take (x); take x;")),
        (
            "extra-conclusion",
            text.replace("thus x = x;", "thus x = x; thus x = x;"),
        ),
        (
            "second-reserve",
            text.replace("reserve x for set;", "reserve x, y for set;"),
        ),
        ("recovery", text.replace("take (x);", "take (x)")),
    ] {
        assert_ne!(current, text, "{case} source mutation");
        let negative = capture(&current);
        if case == "recovery" {
            assert!(negative.diagnostics.iter().any(|diagnostic| matches!(&diagnostic.code, mizar_frontend::orchestration::DiagnosticCode::Syntax(code) if code.as_ref() == "missing_semicolon")));
        } else {
            assert!(
                negative.diagnostics.is_empty(),
                "{case}: {:?}",
                negative.diagnostics
            );
        }
        let negative_ast = negative.ast.as_ref().unwrap();
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
            if case == "nested" {
                use mizar_checker::type_checker::{
                    FormulaStatus, NormalizedTypeStatus, TermKind, TermStatus, TypeHeadRef,
                };
                let (_, inferred, own_base) = TermFormulaChecker::check_source_theorem_equalities(
                    &original,
                    &env,
                    &occurrences,
                )
                .expect("EN13.1.3 preserves the nested witness type");
                assert_eq!(
                    (
                        negative_ast.nodes().len(),
                        own_base.nodes().len(),
                        occurrences.references().len()
                    ),
                    (57, 57, 5)
                );
                let typed_by_resolved = own_base
                    .nodes()
                    .iter()
                    .map(|(id, node)| (node.resolved_node.unwrap(), id))
                    .collect::<std::collections::BTreeMap<_, _>>();
                assert_eq!(typed_by_resolved.len(), 57);
                assert_eq!(own_base.resolved_root(), Some(original.arena().root()));
                assert_eq!(
                    own_base.nodes().root(),
                    Some(
                        typed_by_resolved[&original
                            .resolved_node_for(negative_ast.root().unwrap())
                            .unwrap()]
                    )
                );
                for surface in negative_ast.node_views() {
                    let resolved = original.resolved_node_for(surface.id()).unwrap();
                    let node = own_base.nodes().node(typed_by_resolved[&resolved]).unwrap();
                    let kind = match surface.kind() {
                        K::TermReference => "source.term.variable-reference".to_owned(),
                        K::ParenthesizedTerm => "source.term.parenthesized".to_owned(),
                        K::BuiltinPredicateApplication => {
                            "source.formula.atomic.equality".to_owned()
                        }
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
                            .map(|child| typed_by_resolved
                                [&original.resolved_node_for(*child).unwrap()])
                            .collect::<Vec<_>>()
                    );
                    assert_eq!(node.recovery, NodeRecoveryState::Normal);
                    assert_eq!(node.typing, TypingState::Unknown);
                    assert_eq!(node.links, TypedNodeLinks::default());
                }
                let primary = own_base.source_term().unwrap();
                assert_eq!(
                    (
                        primary.terms().len(),
                        primary.references().len(),
                        primary.numeric_type_requests().len()
                    ),
                    (7, 5, 0)
                );
                assert_eq!(
                    (
                        inferred.terms().len(),
                        inferred.formulas().len(),
                        inferred.normalized_types().len()
                    ),
                    (7, 2, 1)
                );
                let first_checked = inferred.terms().iter().next().unwrap().1;
                let TypeEntryActual::Known(normalized_id) = inferred
                    .type_entries()
                    .get(first_checked.type_entry)
                    .unwrap()
                    .actual
                else {
                    panic!("known nested witness type")
                };
                for (id, term) in primary.terms().iter() {
                    assert_eq!(term.source_ordinal(), id.index());
                    assert_eq!(
                        term.parent().map(|parent| parent.index()),
                        match id.index() {
                            3 => Some(2),
                            4 => Some(3),
                            _ => None,
                        }
                    );
                    let checked = inferred
                        .terms()
                        .iter()
                        .find(|(_, checked)| checked.site == *term.site())
                        .unwrap()
                        .1;
                    assert_eq!(
                        checked.kind,
                        if [2, 3].contains(&id.index()) {
                            TermKind::Parenthesized
                        } else {
                            TermKind::Variable
                        }
                    );
                    assert_eq!(checked.status, TermStatus::Inferred);
                    assert_eq!(checked.context, term.context());
                    assert!(checked.deferred.is_empty() && checked.candidate_set.is_none());
                    if [2, 3].contains(&id.index()) {
                        assert!(checked.reference.is_none());
                        assert!(
                            !primary
                                .references()
                                .iter()
                                .any(|(_, reference)| reference.term() == id)
                        );
                    }
                    let entry = inferred.type_entries().get(checked.type_entry).unwrap();
                    assert_eq!(entry.owner, *term.site());
                    assert_eq!(entry.status, TypeStatus::Known);
                    assert_eq!(entry.actual, TypeEntryActual::Known(normalized_id));
                }
                for (((_, reference), occurrence), target) in primary
                    .references()
                    .iter()
                    .zip(occurrences.references())
                    .zip([0, 1, 4, 5, 6])
                {
                    assert_eq!(reference.term().index(), target);
                    assert_eq!(reference.use_ordinal(), occurrence.ordinal());
                    assert_eq!(reference.lexical_scope(), Some(occurrence.scope()));
                    let term = primary.terms().get(reference.term()).unwrap();
                    assert_eq!(
                        term.site().node(),
                        typed_by_resolved[&original.resolved_node_for(occurrence.node()).unwrap()]
                    );
                    assert_eq!(
                        (term.source_range(), term.spelling()),
                        (occurrence.range(), occurrence.spelling())
                    );
                }
                let written = negative_ast
                    .node_views()
                    .find(|node| node.kind() == &K::TypeExpression)
                    .unwrap();
                let written_node =
                    typed_by_resolved[&original.resolved_node_for(written.id()).unwrap()];
                let mut roles = std::collections::BTreeSet::new();
                for (_, entry) in inferred.type_entries().iter() {
                    if let TypedSiteRef::Role { node, role } = &entry.owner {
                        assert_eq!(*node, written_node);
                        assert!(roles.insert(role.as_str().to_owned()));
                        assert_eq!(entry.actual, TypeEntryActual::Known(normalized_id));
                    }
                }
                assert_eq!(roles.len(), 7);
                for occurrence in occurrences.references() {
                    assert!(
                        roles.contains(&format!("theorem.equality.use:{}", occurrence.ordinal()))
                    );
                }
                let normalized = inferred.normalized_types().get(normalized_id).unwrap();
                assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
                assert_eq!(normalized.status, NormalizedTypeStatus::Known);
                assert_eq!(
                    (normalized.source.range, normalized.source.spelling.as_str()),
                    (written.range(), "set")
                );
                assert!(
                    normalized.args.is_empty()
                        && normalized.attributes.positive().is_empty()
                        && normalized.attributes.negative().is_empty()
                );
                assert!(
                    inferred
                        .formulas()
                        .iter()
                        .all(|(_, formula)| formula.status == FormulaStatus::Checked)
                );
                assert!(
                    inferred.diagnostics().is_empty()
                        && inferred.facts().is_empty()
                        && inferred.candidate_sets().is_empty()
                );
                let own_pair = own_base
                    .clone()
                    .with_source_statement_from_original(&original, &env, &occurrences)
                    .expect("nested witness own canonical pair");
                assert_eq!(
                    own_base.with_source_statement_from_original(&original, &env, &occurrences),
                    Ok(own_pair.clone())
                );
                let own_witnesses = own_pair.source_statement_witnesses().unwrap();
                assert_eq!(
                    (own_witnesses.witnesses().len(), own_witnesses.names().len()),
                    (1, 0)
                );
                assert_eq!(
                    own_witnesses.witnesses().iter().next().unwrap().1.term(),
                    SourceStatementWitnessTermTarget::Primary(
                        mizar_checker::source_term::SourcePrimaryTermId::new(2)
                    )
                );
                assert_eq!(
                    own_pair
                        .source_statement()
                        .unwrap()
                        .input_facts()
                        .iter()
                        .map(|(_, fact)| fact
                            .uses()
                            .iter()
                            .map(|id| id.index())
                            .collect::<Vec<_>>())
                        .collect::<Vec<_>>(),
                    [vec![0, 1], vec![3, 4]]
                );
                assert_eq!(own_pair.source_atomic_formula().unwrap().edges().iter().map(|(_, edge)| match edge.target() { mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(id) => id.index(), _ => usize::MAX }).collect::<Vec<_>>(), [0, 1, 5, 6]);
                let own_final = assemble_empty_resolved_typed_ast(&own_pair, Vec::new()).unwrap();
                assert_eq!(
                    assemble_empty_resolved_typed_ast(&own_pair.clone(), Vec::new()).unwrap(),
                    own_final
                );
                assert_eq!(own_final.source_statement(), own_pair.source_statement());
                assert_eq!(
                    own_final.source_statement_witnesses(),
                    own_pair.source_statement_witnesses()
                );
                assert_eq!(own_final.nodes().len(), 57);
                assert!(
                    own_pair.types().is_empty()
                        && own_pair.contexts().is_empty()
                        && own_pair.facts().is_empty()
                        && own_pair.coercions().is_empty()
                        && own_pair.initial_obligations().is_empty()
                        && own_pair.diagnostics().is_empty()
                );
                assert!(
                    own_final.expr_metadata().is_empty()
                        && own_final.checked_formulas().is_empty()
                        && own_final.statement_semantics().is_empty()
                        && own_final.checked_proofs().is_empty()
                        && own_final.checked_proof_nodes().is_empty()
                        && own_final.checked_terminal_goals().is_empty()
                );
            } else {
                assert_eq!(
                    TermFormulaChecker::check_source_theorem_equalities(
                        &original,
                        &env,
                        &occurrences
                    )
                    .unwrap_err(),
                    unsupported,
                    "{case}"
                );
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
