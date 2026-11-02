use super::*;

#[test]
fn source_singleton_enumeration_witness_original_capture_retains_lower() -> Result<(), String> {
    use mizar_checker::{
        binding_env::*,
        source_atomic_formula::*,
        source_set_term::*,
        source_statement::*,
        source_term::*,
        type_checker::{
            FormulaStatus, NormalizedTypeStatus, SourceVariableSemanticsChecker,
            TermFormulaChecker, TermKind, TermReference, TermStatus, TypeHeadRef,
        },
        typed_ast::{
            NodeRecoveryState, TypeEntryActual, TypeEntryDraft, TypeProvenance, TypeRuleId,
            TypeStatus, TypeTable, TypedArena, TypedAst, TypedAstError, TypedAstParts,
            TypedNodeLinks, TypedSiteRef, TypingState,
        },
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
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SnapshotInput, SnapshotRegistry, SourceInput, SourceOriginInput,
        SourceVersion, ToolchainInfo, WorkspaceRoot, hash_text,
    };
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind};
    // EN13.4.1/13.8.6: the element is a reference; the singleton has its own set type.
    let original = "reserve x for set;\ntheorem SingletonEnumeration: x = x proof\n take {x};\n thus x = x;\nend;\n";
    let texts = [original.to_owned(),
        original.replace("SingletonEnumeration", "RenamedEnumeration").replace('x', "parcel"),
        "reserve\n parcel\tfor :: written type\n set;\ntheorem LayoutEnumeration: parcel = parcel proof :: source only\n take { :: element\n parcel\n }; thus parcel = parcel; end;".to_owned()];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src/singletonenumeration.miz");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let capture = |text: &str, buffer: bool, foreign: bool| {
        fs::write(&path, if buffer { "stale disk" } else { text }).unwrap();
        Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: snapshot_id(779),
                input: SourceInput {
                    package_id: PackageId::new(if foreign {
                        "foreign-singleton-test"
                    } else {
                        "singleton-test"
                    }),
                    module_path: ModulePath::new(if foreign {
                        "foreignsingleton"
                    } else {
                        "singletonenumeration"
                    }),
                    normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2026"),
                    origin: if buffer {
                        SourceOriginInput::OpenBuffer {
                            uri: format!("file://{}", path.display()),
                            expected_version: 3,
                            actual_version: 3,
                            text: text.to_owned().into(),
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
    let mut cases = Vec::new();
    for text in &texts {
        for buffer in [false, true] {
            let frontend = capture(text, buffer, false);
            assert!(
                frontend.diagnostics.is_empty(),
                "{:?}",
                frontend.diagnostics
            );
            assert!(frontend.preprocessed.import_stubs.is_empty());
            assert_eq!(frontend.source.source_text.as_ref(), text);
            if buffer {
                assert_eq!(fs::read_to_string(&path).unwrap(), "stale disk");
            }
            let ast = frontend.ast.as_ref().unwrap();
            assert!(ast.node_views().all(|node| !node.is_recovered()));
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
                (source.source_id(), source.module()),
                (frontend.source.source_id, &module)
            );
            assert_eq!(
                (source.arena().len(), scopes.len(), parents.len()),
                (ast.nodes().len(), ast.nodes().len(), ast.nodes().len() - 1)
            );
            let root_id = ast.root().unwrap();
            assert_eq!(
                source.resolved_node_for(root_id),
                Some(source.arena().root())
            );
            assert!(!parents.contains_key(&root_id));
            for view in ast.node_views() {
                let row = source
                    .arena()
                    .node(source.resolved_node_for(view.id()).unwrap())
                    .unwrap();
                assert_eq!(row.kind(), view.kind());
                assert_eq!(
                    row.origin().anchor(),
                    &mizar_session::SourceAnchor::Range(view.range())
                );
                assert_eq!(
                    row.children(),
                    view.children()
                        .iter()
                        .map(|id| source.resolved_node_for(*id).unwrap())
                        .collect::<Vec<_>>()
                );
                assert!(scopes.contains_key(&view.id()));
                for child in view.children() {
                    if view.kind() != &K::Root
                        || !matches!(ast.node(*child).unwrap().kind, K::Token(_))
                    {
                        assert_eq!(parents.get(child), Some(&view.id()));
                    }
                }
            }
            for (kind, count) in [
                (K::ReserveItem, 1),
                (K::ReserveSegment, 1),
                (K::TypeExpression, 1),
                (K::TheoremItem, 1),
                (K::ProofBlock, 1),
                (K::TakeStatement, 1),
                (K::Witness, 1),
                (K::SetEnumeration, 1),
                (K::TermReference, 5),
                (K::BuiltinPredicateApplication, 2),
                (K::ConclusionStatement, 1),
                (K::ParenthesizedTerm, 0),
                (K::NumeralTerm, 0),
                (K::Reference, 0),
            ] {
                assert_eq!(
                    ast.node_views().filter(|view| view.kind() == &kind).count(),
                    count
                );
            }
            assert_eq!(ast.node_views().filter(|view| matches!(view.kind(), K::Token(token) if token.kind == SurfaceTokenKind::Identifier)).count(), 7);
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
            let proof = ast
                .node_views()
                .find(|view| view.kind() == &K::ProofBlock)
                .unwrap();
            let take = ast
                .node_views()
                .find(|view| view.kind() == &K::TakeStatement)
                .unwrap();
            assert_eq!(ast.node(parents[&proof.id()]).unwrap().kind, K::TheoremItem);
            assert_eq!(parents[&take.id()], proof.id());
            assert_eq!(take.children().len(), 3);
            let witness = ast.node(take.children()[1]).unwrap();
            assert_eq!(witness.kind, K::Witness);
            let [expression] = witness.children.as_slice() else {
                panic!("one unnamed witness")
            };
            assert_eq!(parents[expression], take.children()[1]);
            let expression_node = ast.node(*expression).unwrap();
            assert_eq!(expression_node.kind, K::TermExpression);
            let [enumeration] = expression_node.children.as_slice() else {
                panic!("one bare singleton")
            };
            assert_eq!(parents[enumeration], *expression);
            let enumeration_node = ast.node(*enumeration).unwrap();
            assert_eq!(enumeration_node.kind, K::SetEnumeration);
            let [open, element, close] = enumeration_node.children.as_slice() else {
                panic!("one element")
            };
            for (delimiter, spelling) in [(open, "{"), (close, "}")] {
                let token = ast.node(*delimiter).unwrap();
                assert_eq!(parents[delimiter], *enumeration);
                assert_eq!(token.token_text(), Some(spelling));
                assert!(
                    matches!(&token.kind, K::Token(token) if token.kind == SurfaceTokenKind::ReservedSymbol)
                );
            }
            assert_eq!(parents[element], *enumeration);
            let element_node = ast.node(*element).unwrap();
            assert_eq!(element_node.kind, K::TermExpression);
            assert_eq!(element_node.children, [scope.references()[2].node()]);
            for (ordinal, occurrence) in scope.references().iter().enumerate() {
                assert_eq!(occurrence.ordinal(), ordinal);
                assert_eq!(occurrence.binding(), scope.bindings()[0].id());
                assert_eq!(occurrence.spelling(), scope.bindings()[0].spelling());
                let term = ast.node(occurrence.node()).unwrap();
                assert_eq!(term.kind, K::TermReference);
                assert_eq!(term.children.len(), 1);
                let token = ast.node(term.children[0]).unwrap();
                assert_eq!(
                    (token.range, token.token_text()),
                    (occurrence.range(), Some(occurrence.spelling()))
                );
                assert_eq!(scopes.get(&occurrence.node()), Some(occurrence.scope()));
                assert_eq!(
                    occurrence.scope().path(),
                    if ordinal < 2 {
                        &[][..]
                    } else {
                        scopes[&proof.id()].path()
                    }
                );
                assert_eq!(ordinal < 2, occurrence.range().start < proof.range().start);
            }
            assert!(
                scopes
                    .iter()
                    .filter(|(id, _)| {
                        let range = ast.node(**id).unwrap().range;
                        take.range().start <= range.start && range.end <= take.range().end
                    })
                    .all(|(_, scope)| scope == &scopes[&proof.id()])
            );
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
    // Prepare all actual captures and collect every public lower result before future success.
    let lowers = cases
        .iter()
        .map(|(_, source, symbols, scope, ..)| {
            TermFormulaChecker::check_source_theorem_equalities(source, symbols, scope)
        })
        .collect::<Vec<_>>();
    let errors = lowers
        .iter()
        .filter_map(|result| result.as_ref().err())
        .collect::<Vec<_>>();
    assert!(
        errors.is_empty(),
        "native singleton enumeration lower: {errors:?}"
    );
    for ((frontend, source, symbols, scope, labels, resolution, scopes, parents, before), result) in
        cases.iter().zip(&lowers)
    {
        let (bindings, inference, base) = result.as_ref().unwrap();
        let ast = frontend.ast.as_ref().unwrap();
        let primary = base.source_term().unwrap();
        let set = base.source_set_term().unwrap();
        let atomic = base.source_atomic_formula().unwrap();
        let typed = base
            .nodes()
            .iter()
            .map(|(id, row)| (row.resolved_node.unwrap(), id))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            (typed.len(), base.nodes().len()),
            (ast.nodes().len(), ast.nodes().len())
        );
        assert_eq!(base.resolved_root(), Some(source.arena().root()));
        assert_eq!(base.nodes().root(), Some(typed[&source.arena().root()]));
        assert_eq!(
            (base.source_id(), base.module_id()),
            (source.source_id(), source.module())
        );
        for view in ast.node_views() {
            let resolved = source.resolved_node_for(view.id()).unwrap();
            let row = base.nodes().node(typed[&resolved]).unwrap();
            let kind = match view.kind() {
                K::TermReference => "source.term.variable-reference".to_owned(),
                K::SetEnumeration => "source.term.set.enumeration".to_owned(),
                K::BuiltinPredicateApplication => "source.formula.atomic.equality".to_owned(),
                kind => format!("{kind:?}"),
            };
            assert_eq!(row.kind.as_str(), kind);
            assert_eq!(row.resolved_node, Some(resolved));
            assert_eq!(row.anchor, mizar_session::SourceAnchor::Range(view.range()));
            assert_eq!(
                row.children,
                view.children()
                    .iter()
                    .map(|id| typed[&source.resolved_node_for(*id).unwrap()])
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                (row.recovery, row.typing, row.links.clone()),
                (
                    NodeRecoveryState::Normal,
                    TypingState::Unknown,
                    TypedNodeLinks::default()
                )
            );
        }
        assert_eq!(
            (
                bindings.contexts().len(),
                bindings.bindings().len(),
                primary.terms().len(),
                primary.references().len(),
                primary.numeric_type_requests().len()
            ),
            (2, 1, 5, 5, 0)
        );
        assert_eq!(
            (bindings.source_id(), bindings.module_id()),
            (source.source_id(), source.module())
        );
        assert_eq!(
            (primary.source_id(), primary.module_id()),
            (source.source_id(), source.module())
        );
        assert!(bindings.diagnostics().is_empty());
        let (binding_id, binding) = bindings.bindings().iter().next().unwrap();
        assert_eq!(binding.kind, BindingKind::ReservedVariable);
        assert_eq!(binding.status, BindingStatus::Active);
        assert_eq!(
            binding.identity,
            BinderIdentity::ReservedVariable {
                spelling: scope.bindings()[0].spelling().to_owned(),
                declaration_range: scope.bindings()[0].range()
            }
        );
        assert_eq!(binding.visible_after_ordinal, scope.bindings()[0].ordinal());
        assert_eq!(binding.type_site, BindingTypeSite::Missing);
        assert!(binding.captured.identities().is_empty() && binding.diagnostics.is_empty());
        let proof = ast
            .node_views()
            .find(|view| view.kind() == &K::ProofBlock)
            .unwrap();
        let module_context = bindings
            .contexts()
            .iter()
            .find(|(_, row)| row.owner == BindingContextOwner::Module)
            .unwrap()
            .0;
        let proof_context = bindings
            .contexts()
            .iter()
            .find(|(_, row)| row.owner != BindingContextOwner::Module)
            .unwrap()
            .0;
        assert_eq!(binding.owner_context, module_context);
        for (id, context) in bindings.contexts().iter() {
            assert_eq!(context.visible_bindings, [binding_id]);
            assert_eq!(context.recovery, BindingContextRecovery::Normal);
            if id == module_context {
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
                assert_eq!(context.lexical_scope.as_ref(), scopes.get(&proof.id()));
                assert!(context.bindings.is_empty());
            }
        }
        for (((id, term), (_, reference)), occurrence) in primary
            .terms()
            .iter()
            .zip(primary.references().iter())
            .zip(scope.references())
        {
            assert_eq!(
                term.site().node(),
                typed[&source.resolved_node_for(occurrence.node()).unwrap()]
            );
            assert_eq!(
                (term.source_range(), term.spelling(), term.source_ordinal()),
                (occurrence.range(), occurrence.spelling(), id.index())
            );
            assert_eq!(
                (term.kind(), term.role(), term.recovery()),
                (
                    SourcePrimaryTermKind::VariableReference,
                    SourcePrimaryTermRole::Value,
                    SourcePrimaryTermRecovery::Normal
                )
            );
            assert!(term.parent().is_none());
            assert_eq!(
                term.context(),
                if id.index() < 2 {
                    module_context
                } else {
                    proof_context
                }
            );
            assert_eq!(
                (reference.term(), reference.binding(), reference.role()),
                (id, binding_id, SourcePrimaryTermReferenceRole::Variable)
            );
            assert_eq!(reference.use_ordinal(), occurrence.ordinal());
            assert_eq!(reference.lexical_scope(), Some(occurrence.scope()));
        }
        assert_eq!(
            (set.source_id(), set.module_id()),
            (source.source_id(), source.module())
        );
        assert_eq!(set.primary_term_fingerprint(), primary.debug_text());
        assert!(set.application_fingerprint().is_none() && set.structure_fingerprint().is_none());
        assert_eq!(
            (set.terms().len(), set.edges().len(), set.requests().len()),
            (1, 1, 1)
        );
        assert!(
            set.wrappers().is_empty()
                && set.generators().is_empty()
                && set.type_sites().is_empty()
                && set.conditions().is_empty()
        );
        let enumeration = ast
            .node_views()
            .find(|view| view.kind() == &K::SetEnumeration)
            .unwrap();
        let (set_id, set_row) = set.terms().iter().next().unwrap();
        assert_eq!(set_id.index(), 0);
        assert_eq!(
            set_row.site().node(),
            typed[&source.resolved_node_for(enumeration.id()).unwrap()]
        );
        assert_eq!(set_row.source_range(), enumeration.range());
        assert_eq!(
            (
                set_row.source_ordinal(),
                set_row.context(),
                set_row.kind(),
                set_row.recovery()
            ),
            (
                0,
                proof_context,
                SourceSetTermKind::Enumeration,
                SourceSetTermRecovery::Normal
            )
        );
        assert_eq!(
            set_row.spelling(),
            format!("{{ {} }}", scope.bindings()[0].spelling())
        );
        let (_, edge) = set.edges().iter().next().unwrap();
        assert_eq!(
            (edge.term(), edge.ordinal(), edge.role(), edge.target()),
            (
                set_id,
                0,
                SourceSetEdgeRole::EnumerationElement,
                SourceSetTarget::Primary(SourcePrimaryTermId::new(2))
            )
        );
        let element = primary.terms().get(SourcePrimaryTermId::new(2)).unwrap();
        assert_eq!(element.source_range(), scope.references()[2].range());
        assert!(
            set_row.source_range().start < element.source_range().start
                && element.source_range().end < set_row.source_range().end
        );
        let (_, request) = set.requests().iter().next().unwrap();
        assert_eq!(
            (
                request.term(),
                request.ordinal(),
                request.kind(),
                request.generator(),
                request.type_site()
            ),
            (set_id, 0, SourceSetRequestKind::ResultType, None, None)
        );
        assert_eq!(
            (atomic.source_id(), atomic.module_id()),
            (source.source_id(), source.module())
        );
        assert_eq!(atomic.primary_term_fingerprint(), primary.debug_text());
        assert!(
            atomic.application_fingerprint().is_none()
                && atomic.structure_fingerprint().is_none()
                && atomic.set_term_fingerprint().is_none()
        );
        assert_eq!(
            (
                atomic.formulas().len(),
                atomic.edges().len(),
                atomic.requests().len()
            ),
            (2, 4, 4)
        );
        assert!(
            atomic.wrappers().is_empty()
                && atomic.predicate_segments().is_empty()
                && atomic.predicate_heads().is_empty()
                && atomic.candidates().is_empty()
                && atomic.type_sites().is_empty()
                && atomic.attributes().is_empty()
        );
        let mut equalities = ast
            .node_views()
            .filter(|view| view.kind() == &K::BuiltinPredicateApplication)
            .collect::<Vec<_>>();
        equalities.sort_by_key(|view| view.range().start);
        for ((id, formula), equality) in atomic.formulas().iter().zip(equalities) {
            assert_eq!(
                formula.site().node(),
                typed[&source.resolved_node_for(equality.id()).unwrap()]
            );
            assert_eq!(formula.source_range(), equality.range());
            assert_eq!(
                (formula.source_ordinal(), formula.kind(), formula.recovery()),
                (
                    id.index(),
                    SourceAtomicFormulaKind::Equality,
                    SourceAtomicFormulaRecovery::Normal
                )
            );
            assert_eq!(
                formula.context(),
                if id.index() == 0 {
                    module_context
                } else {
                    proof_context
                }
            );
            let checked = inference
                .formulas()
                .iter()
                .find(|(_, row)| row.site == *formula.site())
                .unwrap()
                .1;
            assert_eq!(
                (checked.status, checked.context),
                (FormulaStatus::Checked, formula.context())
            );
            assert!(
                checked.deferred.is_empty()
                    && checked.facts.is_empty()
                    && checked.candidate_set.is_none()
            );
            assert_eq!(checked.terms.len(), 2);
            for ordinal in 0..2 {
                let primary_id = SourcePrimaryTermId::new(if id.index() == 0 {
                    ordinal
                } else {
                    3 + ordinal
                });
                let term = primary.terms().get(primary_id).unwrap();
                let operand = ast.node(equality.children()[2 * ordinal]).unwrap().children[0];
                assert_eq!(
                    term.site().node(),
                    typed[&source.resolved_node_for(operand).unwrap()]
                );
                assert_eq!(checked.terms[ordinal], *term.site());
            }
        }
        for (index, ((_, edge), (_, request))) in atomic
            .edges()
            .iter()
            .zip(atomic.requests().iter())
            .enumerate()
        {
            assert_eq!(
                edge.target(),
                SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(if index < 2 {
                    index
                } else {
                    index + 1
                }))
            );
            assert_eq!(
                (edge.formula(), edge.ordinal()),
                (SourceAtomicFormulaId::new(index / 2), index % 2)
            );
            assert_eq!(
                edge.role(),
                if index % 2 == 0 {
                    SourceAtomicEdgeRole::BuiltinLeftOperand
                } else {
                    SourceAtomicEdgeRole::BuiltinRightOperand
                }
            );
            assert_eq!(
                (
                    request.formula(),
                    request.ordinal(),
                    request.kind(),
                    request.edge()
                ),
                (
                    edge.formula(),
                    edge.ordinal(),
                    SourceAtomicRequestKind::OperandExpectedType,
                    Some(SourceAtomicEdgeId::new(index))
                )
            );
            assert!(
                request.candidate().is_none()
                    && request.type_site().is_none()
                    && request.attribute().is_none()
            );
        }
        assert_eq!(
            (
                inference.terms().len(),
                inference.formulas().len(),
                inference.type_entries().len(),
                inference.normalized_types().len()
            ),
            (6, 2, 12, 1)
        );
        let (normalized_id, normalized) = inference.normalized_types().iter().next().unwrap();
        assert_eq!(
            (&normalized.head, normalized.status),
            (&TypeHeadRef::BuiltinSet, NormalizedTypeStatus::Known)
        );
        assert!(
            normalized.args.is_empty()
                && normalized.attributes.positive().is_empty()
                && normalized.attributes.negative().is_empty()
        );
        let written_type = ast
            .node_views()
            .find(|view| view.kind() == &K::TypeExpression)
            .unwrap();
        assert_eq!(
            ast.node(parents[&written_type.id()]).unwrap().kind,
            K::ReserveSegment
        );
        assert_eq!(
            (normalized.source.range, normalized.source.spelling.as_str()),
            (written_type.range(), "set")
        );
        let written_node = typed[&source.resolved_node_for(written_type.id()).unwrap()];
        let mut roles = std::collections::BTreeSet::new();
        let mut intrinsic = Vec::new();
        for (id, entry) in inference.type_entries().iter() {
            assert_eq!(
                (entry.status, entry.actual),
                (TypeStatus::Known, TypeEntryActual::Known(normalized_id))
            );
            match &entry.owner {
                TypedSiteRef::Role { node, role } => {
                    assert_eq!(*node, written_node);
                    assert!(roles.insert(role.as_str().to_owned()));
                    assert_eq!(
                        entry.provenance,
                        TypeProvenance::Inferred(TypeRuleId::new("type-expression-normalization"))
                    );
                }
                site if site == set_row.site()
                    && entry.provenance
                        == TypeProvenance::Inferred(TypeRuleId::new(
                            "type-expression-normalization",
                        )) =>
                {
                    assert_eq!(
                        base.nodes().node(site.node()).unwrap().anchor,
                        mizar_session::SourceAnchor::Range(enumeration.range())
                    );
                    intrinsic.push(id);
                }
                _ => {}
            }
        }
        assert_eq!(
            roles,
            (0..5)
                .map(|ordinal| format!("theorem.equality.use:{ordinal}"))
                .collect()
        );
        assert_eq!(intrinsic.len(), 1);
        let mut checked_entries = std::collections::BTreeSet::new();
        for (site, context, kind, reference) in primary
            .terms()
            .iter()
            .map(|(_, term)| {
                (
                    term.site(),
                    term.context(),
                    TermKind::Variable,
                    Some(TermReference::Binding(binding_id)),
                )
            })
            .chain(std::iter::once((
                set_row.site(),
                proof_context,
                TermKind::SetEnumeration,
                None,
            )))
        {
            let checked = inference
                .terms()
                .iter()
                .find(|(_, row)| &row.site == site)
                .unwrap()
                .1;
            assert_eq!(
                (
                    checked.context,
                    checked.kind,
                    checked.reference.clone(),
                    checked.status
                ),
                (context, kind, reference, TermStatus::Inferred)
            );
            assert!(checked.deferred.is_empty() && checked.candidate_set.is_none());
            let entry = inference.type_entries().get(checked.type_entry).unwrap();
            assert_eq!(&entry.owner, site);
            assert!(checked_entries.insert(checked.type_entry));
        }
        assert_eq!(checked_entries.len(), 6);
        assert!(!checked_entries.contains(&intrinsic[0]));
        assert_eq!(
            inference
                .type_entries()
                .iter()
                .filter(|(_, entry)| &entry.owner == set_row.site())
                .count(),
            2
        );
        assert!(
            inference.diagnostics().is_empty()
                && inference.candidate_sets().is_empty()
                && inference.facts().is_empty()
        );
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
                && base.source_evidence().is_none()
                && base.source_template().is_none()
                && base.source_application().is_none()
                && base.source_structure().is_none()
                && base.source_attribute_definition().is_none()
                && base.source_predicate_definition().is_none()
                && base.source_functor_definition().is_none()
                && base.source_mode_definition().is_none()
                && base.source_structure_definition().is_none()
                && base.source_composite_formula().is_none()
                && base.source_formula_composition().is_none()
                && base.source_statement().is_none()
                && base.source_statement_witnesses().is_none()
                && base.source_statement_references().is_none()
                && base.source_proof_local_declaration().is_none()
        );
        // Reinstall actual clones through the public transaction, retaining the whole primary fingerprint.
        let rebuilt = TypedAst::try_new(TypedAstParts {
            source_id: base.source_id(),
            module_id: base.module_id().clone(),
            resolved_root: base.resolved_root(),
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: base.nodes().clone(),
            contexts: base.contexts().clone(),
            types: base.types().clone(),
            facts: base.facts().clone(),
            coercions: base.coercions().clone(),
            initial_obligations: base.initial_obligations().clone(),
            diagnostics: base.diagnostics().clone(),
        })
        .unwrap()
        .with_source_term(primary.clone())
        .unwrap()
        .with_source_set_term(set.clone())
        .unwrap()
        .with_source_atomic_formula(atomic.clone())
        .unwrap();
        assert_eq!(&rebuilt, base);
        let resolved = super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
            &base.clone(),
            Vec::new(),
        )?;
        assert_eq!(resolved.source_set_term(), Some(&set.clone()));
        assert_eq!(resolved.source_term(), Some(&primary.clone()));
        assert_eq!(resolved.source_atomic_formula(), Some(&atomic.clone()));
        assert!(
            resolved.checked_formulas().is_empty()
                && resolved.statement_semantics().is_empty()
                && resolved.checked_proofs().is_empty()
        );
        assert_eq!(
            TermFormulaChecker::check_source_theorem_equalities(source, symbols, scope).as_ref(),
            Ok(result.as_ref().unwrap())
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
    // Collect every own-source attempt before the future-success assertion.
    let pairs = cases
        .iter()
        .zip(&lowers)
        .map(|((_, source, symbols, scope, ..), lower)| {
            lower
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
        "native singleton enumeration witness original pair: {errors:?}"
    );
    // All pair tables and source joins precede every original-statement final call.
    for (
        (
            (frontend, source, symbols, scope, labels, resolution, scopes, parents, inputs_before),
            lower,
        ),
        pair,
    ) in cases.iter().zip(&lowers).zip(&pairs)
    {
        let (bindings, _, base) = lower.as_ref().unwrap();
        let installed = pair.as_ref().unwrap();
        let before = (base.clone(), installed.clone());
        assert_eq!(installed.source_id(), base.source_id());
        assert_eq!(installed.module_id(), base.module_id());
        assert_eq!(installed.resolved_root(), base.resolved_root());
        assert_eq!(installed.nodes(), base.nodes());
        assert_eq!(installed.source_term(), base.source_term());
        assert_eq!(installed.source_set_term(), base.source_set_term());
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
        let set = base.source_set_term().unwrap();
        let atomic = base.source_atomic_formula().unwrap();
        assert_eq!(statement.source_id(), source.source_id());
        assert_eq!(statement.module_id(), source.module());
        assert_eq!(statement.binding_env(), bindings);
        assert_eq!(statement.binding_fingerprint(), bindings.debug_text());
        assert_eq!(statement.primary_term_fingerprint(), primary.debug_text());
        assert_eq!(statement.atomic_formula_fingerprint(), atomic.debug_text());
        assert!(statement.composite_formula_fingerprint().is_none());
        assert!(statement.formula_composition_fingerprint().is_none());
        assert_eq!(witnesses.source_id(), source.source_id());
        assert_eq!(witnesses.module_id(), source.module());
        assert_eq!(witnesses.statement_fingerprint(), statement.debug_text());
        assert_eq!(witnesses.primary_term_fingerprint(), primary.debug_text());
        assert_eq!(
            witnesses.set_term_fingerprint(),
            Some(set.debug_text().as_str())
        );
        assert!(witnesses.application_fingerprint().is_none());
        assert!(witnesses.structure_fingerprint().is_none());
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
        let ast = frontend.ast.as_ref().unwrap();
        let typed = base
            .nodes()
            .iter()
            .map(|(id, row)| (row.resolved_node.unwrap(), id))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut sites = ast
            .node_views()
            .filter(|view| matches!(view.kind(), K::TheoremItem | K::ConclusionStatement))
            .collect::<Vec<_>>();
        sites.sort_by_key(|view| view.range().start);
        let (owner_id, owner) = statement.owners().iter().next().unwrap();
        assert_eq!(owner_id.index(), 0);
        let projection = &labels.projections()[0];
        let symbol = symbols
            .symbols()
            .iter()
            .find(|entry| entry.contribution() == projection.contribution())
            .unwrap();
        assert_eq!(owner.symbol(), symbol.symbol());
        assert_eq!(owner.symbol(), statement.checked_owner().symbol());
        assert_eq!(owner.contribution(), projection.contribution());
        assert_eq!(owner.spelling(), projection.primary_spelling());
        assert_eq!(owner.source_range(), sites[0].range());
        assert_eq!(
            owner.site().node(),
            typed[&source.resolved_node_for(sites[0].id()).unwrap()]
        );
        assert_eq!(owner.role(), SourceTheoremRole::Theorem);
        assert_eq!(owner.status(), SourceTheoremStatus::Unmodified);
        assert_eq!(owner.recovery(), SourceStatementRecovery::Normal);
        let (witness_id, witness) = witnesses.witnesses().iter().next().unwrap();
        assert_eq!(witness_id.index(), 0);
        let take = ast
            .node_views()
            .find(|view| view.kind() == &K::TakeStatement)
            .unwrap();
        let witness_site = ast.node(take.children()[1]).unwrap();
        let expression = ast.node(witness_site.children[0]).unwrap();
        let (set_id, enumeration) = set.terms().iter().next().unwrap();
        assert_eq!(
            witness.term(),
            SourceStatementWitnessTermTarget::SetTerm(set_id)
        );
        assert_eq!(set_id.index(), 0);
        assert_eq!(
            enumeration.site().node(),
            typed[&source.resolved_node_for(expression.children[0]).unwrap()]
        );
        assert_eq!(witness.owner(), owner_id);
        assert_eq!(witness.binding_context(), enumeration.context());
        assert_eq!((witness.source_ordinal(), witness.ordinal()), (1, 0));
        assert_eq!(
            witness.take_site().node(),
            typed[&source.resolved_node_for(take.id()).unwrap()]
        );
        assert_eq!(witness.take_range(), take.range());
        assert_eq!(
            witness.site().node(),
            typed[&source.resolved_node_for(take.children()[1]).unwrap()]
        );
        assert_eq!(witness.source_range(), witness_site.range);
        assert_eq!(witness.spelling(), enumeration.spelling());
        assert_eq!(witness.kind(), SourceStatementWitnessKind::Unnamed);
        assert_eq!(witness.recovery(), SourceStatementRecovery::Normal);
        assert!(witness.name().is_none());
        assert_eq!(
            bindings
                .contexts()
                .get(witness.binding_context())
                .unwrap()
                .lexical_scope
                .as_ref(),
            scopes.get(&take.id())
        );
        let (_, edge) = set.edges().iter().next().unwrap();
        let SourceSetTarget::Primary(element_id) = edge.target() else {
            panic!("actual singleton element")
        };
        let element = primary.terms().get(element_id).unwrap();
        assert_eq!(
            (edge.term(), edge.ordinal(), edge.role()),
            (set_id, 0, SourceSetEdgeRole::EnumerationElement)
        );
        assert_eq!(element_id.index(), 2);
        let actual_enumeration = ast.node(expression.children[0]).unwrap();
        let actual_element = ast.node(actual_enumeration.children[1]).unwrap();
        assert_eq!(
            element.site().node(),
            typed[&source
                .resolved_node_for(actual_element.children[0])
                .unwrap()]
        );
        assert_eq!(
            enumeration.spelling(),
            format!("{{ {} }}", element.spelling())
        );
        assert_eq!(element.context(), witness.binding_context());
        assert_eq!(primary.references().len(), 5);
        for ((id, reference), occurrence) in primary.references().iter().zip(scope.references()) {
            let term = primary.terms().get(reference.term()).unwrap();
            assert_eq!(reference.use_ordinal(), occurrence.ordinal());
            assert_eq!(reference.lexical_scope(), Some(occurrence.scope()));
            assert_eq!(
                (term.source_ordinal(), term.source_range(), term.spelling()),
                (id.index(), occurrence.range(), occurrence.spelling())
            );
        }
        let equalities = (0..2)
            .map(|index| {
                let left = primary
                    .terms()
                    .get(SourcePrimaryTermId::new([0, 3][index]))
                    .unwrap();
                let right = primary
                    .terms()
                    .get(SourcePrimaryTermId::new([1, 4][index]))
                    .unwrap();
                let spelling = format!("{} = {}", left.spelling(), right.spelling());
                assert_eq!(
                    atomic
                        .formulas()
                        .get(SourceAtomicFormulaId::new(index))
                        .unwrap()
                        .spelling(),
                    spelling
                );
                spelling
            })
            .collect::<Vec<_>>();
        for (index, ((id, row), site)) in statement.statements().iter().zip(sites).enumerate() {
            assert_eq!(
                (id.index(), row.context().index(), row.source_ordinal()),
                (index, index, [0, 2][index])
            );
            assert_eq!(row.owner(), owner_id);
            assert_eq!(
                row.site().node(),
                typed[&source.resolved_node_for(site.id()).unwrap()]
            );
            assert_eq!(row.source_range(), site.range());
            assert_eq!(
                row.kind(),
                if index == 0 {
                    SourceStatementKind::TheoremProposition
                } else {
                    SourceStatementKind::Conclusion
                }
            );
            assert_eq!(row.recovery(), SourceStatementRecovery::Normal);
            assert_eq!(
                row.formula(),
                SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(index))
            );
            assert_eq!(
                row.spelling(),
                if index == 0 {
                    format!(
                        "theorem {} : {} proof take {} ; thus {} ; end ;",
                        owner.spelling(),
                        equalities[0],
                        witness.spelling(),
                        equalities[1]
                    )
                } else {
                    format!("thus {} ;", equalities[1])
                }
            );
            let context = statement.contexts().get(row.context()).unwrap();
            assert_eq!(context.statement(), id);
            assert_eq!(context.source_range(), row.source_range());
            assert_eq!(context.binding_context().index(), index);
            assert_eq!(context.visible_bindings(), &[BindingId::new(0)]);
            let binding_context = bindings.contexts().get(context.binding_context()).unwrap();
            assert_eq!(binding_context.visible_bindings, context.visible_bindings());
            if index == 1 {
                assert_eq!(binding_context.parent, Some(BindingContextId::new(0)));
                assert_eq!(
                    binding_context.lexical_scope.as_ref(),
                    scopes.get(&take.id())
                );
            }
            let (fact_id, fact) = statement.input_facts().iter().nth(index).unwrap();
            assert_eq!(fact_id.index(), index);
            assert_eq!(
                (
                    fact.statement(),
                    fact.context(),
                    fact.ordinal(),
                    fact.binding()
                ),
                (id, row.context(), 0, BindingId::new(0))
            );
            assert_eq!(fact.kind(), SourceStatementInputFactKind::ReservedTypeGuard);
            assert_eq!(
                fact.uses().iter().map(|id| id.index()).collect::<Vec<_>>(),
                [[0, 1], [3, 4]][index]
            );
            let (candidate_id, candidate) = statement.candidate_facts().iter().nth(index).unwrap();
            assert_eq!(candidate_id.index(), index);
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
                .with_source_statement_from_original(source, symbols, scope),
            Ok(installed.clone())
        );
        assert_eq!((base, installed), (&before.0, &before.1));
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
                &inputs_before.0,
                &inputs_before.1,
                &inputs_before.2,
                &inputs_before.3,
                &inputs_before.4,
                &inputs_before.5,
                &inputs_before.6,
                inputs_before.7.clone(),
                &inputs_before.8
            )
        );
    }
    let finals = pairs
        .iter()
        .map(|pair| {
            super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
                pair.as_ref().unwrap(),
                Vec::new(),
            )
        })
        .collect::<Vec<_>>();
    let errors = finals
        .iter()
        .filter_map(|result| result.as_ref().err())
        .collect::<Vec<_>>();
    assert!(
        errors.is_empty(),
        "native singleton enumeration witness original final: {errors:?}"
    );
    for (((case, lower), pair), resolved) in cases.iter().zip(&lowers).zip(&pairs).zip(&finals) {
        let (frontend, source, symbols, scope, labels, resolution, scopes, parents, before) = case;
        let base = &lower.as_ref().unwrap().2;
        let installed = pair.as_ref().unwrap();
        let resolved = resolved.as_ref().unwrap();
        assert_eq!(resolved.source_id(), source.source_id());
        assert_eq!(resolved.module_id(), source.module());
        assert_eq!(resolved.nodes().len(), base.nodes().len());
        assert_eq!(
            resolved
                .nodes()
                .node(resolved.nodes().root().unwrap())
                .unwrap()
                .typed_node,
            base.nodes().root().unwrap()
        );
        let mut seen = std::collections::BTreeSet::new();
        for (_, node) in resolved.nodes().iter() {
            assert!(seen.insert(node.typed_node));
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
            assert_eq!(original.typing, TypingState::Unknown);
            assert_eq!(original.links, TypedNodeLinks::default());
        }
        assert_eq!(seen, base.nodes().iter().map(|(id, _)| id).collect());
        assert_eq!(resolved.source_statement(), installed.source_statement());
        assert_eq!(
            resolved.source_statement_witnesses(),
            installed.source_statement_witnesses()
        );
        assert_eq!(resolved.source_term(), installed.source_term());
        assert_eq!(resolved.source_set_term(), installed.source_set_term());
        assert_eq!(
            resolved.source_atomic_formula(),
            installed.source_atomic_formula()
        );
        assert!(
            resolved.source_context().is_none()
                && resolved.source_type().is_none()
                && resolved.source_attribute().is_none()
                && resolved.source_evidence().is_none()
                && resolved.source_template().is_none()
                && resolved.source_application().is_none()
                && resolved.source_structure().is_none()
                && resolved.source_attribute_definition().is_none()
                && resolved.source_predicate_definition().is_none()
                && resolved.source_functor_definition().is_none()
                && resolved.source_mode_definition().is_none()
                && resolved.source_structure_definition().is_none()
                && resolved.source_composite_formula().is_none()
                && resolved.source_formula_composition().is_none()
                && resolved.source_statement_references().is_none()
                && resolved.source_proof_local_declaration().is_none()
                && resolved.source_property_implementation().is_none()
                && resolved.source_nested_fraenkel_capture_identity().is_none()
                && resolved.source_condition_formula_composition().is_none()
                && resolved.source_predicate_chain_composition().is_none()
                && resolved.source_proof_local_let_binding().is_none()
                && resolved.source_proof_local_let_type().is_none()
                && resolved.source_proof_local_given_binding().is_none()
                && resolved.source_proof_local_given_type().is_none()
                && resolved.source_proof_local_given_use_type().is_none()
                && resolved.source_proof_local_given_use_term().is_none()
                && resolved
                    .source_proof_local_given_condition_binding()
                    .is_none()
                && resolved.source_proof_local_given_condition_type().is_none()
                && resolved
                    .source_proof_local_given_condition_use_term()
                    .is_none()
                && resolved
                    .source_proof_local_given_descendant_binding()
                    .is_none()
                && resolved
                    .source_proof_local_given_descendant_type()
                    .is_none()
                && resolved
                    .source_proof_local_given_descendant_use_term()
                    .is_none()
        );
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
            super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
                &installed.clone(),
                Vec::new()
            )?,
            *resolved
        );
        let replay = base
            .clone()
            .with_source_statement_from_original(source, symbols, scope)
            .unwrap();
        assert_eq!(&replay, installed);
        assert_eq!(
            super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
                &replay,
                Vec::new()
            )?,
            *resolved
        );
        assert_eq!(
            TermFormulaChecker::check_source_theorem_equalities(source, symbols, scope).as_ref(),
            Ok(lower.as_ref().unwrap())
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
                frontend.ast.as_ref().unwrap().snapshot_text(),
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
    // Every full lower and original-pair oracle precedes the higher Reserve/Proof refusals.
    for ((frontend, source, symbols, scope, labels, resolution, scopes, parents, before), result) in
        cases.iter().zip(&lowers)
    {
        let pristine = || {
            let index = cases
                .iter()
                .position(|case| case.1.source_id() == source.source_id())
                .unwrap();
            assert_eq!(
                result
                    .as_ref()
                    .unwrap()
                    .2
                    .clone()
                    .with_source_statement_from_original(source, symbols, scope)
                    .as_ref(),
                Ok(pairs[index].as_ref().unwrap())
            );
            assert_eq!(
                super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
                    &pairs[index].as_ref().unwrap().clone(),
                    Vec::new()
                )
                .as_ref(),
                Ok(finals[index].as_ref().unwrap())
            );
            assert_eq!(
                TermFormulaChecker::check_source_theorem_equalities(source, symbols, scope)
                    .as_ref(),
                Ok(result.as_ref().unwrap())
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
                    frontend.ast.as_ref().unwrap().snapshot_text(),
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
        };
        let unsupported = "theorems.equality.unsupported_source_types";
        assert_eq!(
            TermFormulaChecker::check_source_theorem_equalities_with_typed_tables(
                source, symbols, scope
            )
            .unwrap_err(),
            unsupported
        );
        pristine();
        assert_eq!(
            TermFormulaChecker::check_source_theorem_equalities_with_expression_metadata(
                source, symbols, scope
            )
            .unwrap_err(),
            unsupported
        );
        pristine();
        assert_eq!(
            SourceVariableSemanticsChecker::source_theorem_binding_env(source, symbols, scope)
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
                    toolchain: ToolchainInfo::new("native-singleton-lower-boundary"),
                    verifier_config_hash: policy.policy_fingerprint().hash(),
                },
            )
            .unwrap();
        let snapshot_before = snapshot.clone();
        assert_eq!(
            mizar_proof::status::source_theorem_reflexivity_witness_draft_from_original(
                source, symbols, &snapshot, &policy, ""
            )
            .unwrap_err(),
            unsupported
        );
        assert_eq!(snapshot, snapshot_before);
        assert!(!root.join("proof-witnesses").exists());
        registry.release_lease(snapshot.id, lease.lease_id).unwrap();
        pristine();
    }
    let pristine = || {
        for (
            (frontend, source, symbols, scope, labels, resolution, scopes, parents, before),
            result,
        ) in cases.iter().zip(&lowers)
        {
            let index = cases
                .iter()
                .position(|case| case.1.source_id() == source.source_id())
                .unwrap();
            assert_eq!(
                result
                    .as_ref()
                    .unwrap()
                    .2
                    .clone()
                    .with_source_statement_from_original(source, symbols, scope)
                    .as_ref(),
                Ok(pairs[index].as_ref().unwrap())
            );
            assert_eq!(
                super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
                    &pairs[index].as_ref().unwrap().clone(),
                    Vec::new()
                )
                .as_ref(),
                Ok(finals[index].as_ref().unwrap())
            );
            assert_eq!(
                TermFormulaChecker::check_source_theorem_equalities(source, symbols, scope)
                    .as_ref(),
                Ok(result.as_ref().unwrap())
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
                    frontend.ast.as_ref().unwrap().snapshot_text(),
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
    };
    let target = &cases[0];
    let base = &lowers[0].as_ref().unwrap().2;
    let installed = pairs[0].as_ref().unwrap();
    let statement = installed.source_statement().unwrap();
    let witnesses = installed.source_statement_witnesses().unwrap();
    assert_eq!(
        installed
            .clone()
            .with_source_statement_from_original(&target.1, &target.2, &target.3),
        Err(TypedAstError::InvalidSourceStatement)
    );
    pristine();
    // A coherent lower from another capture is still a stale receiver for this source.
    assert_eq!(
        lowers[2]
            .as_ref()
            .unwrap()
            .2
            .clone()
            .with_source_statement_from_original(&target.1, &target.2, &target.3),
        Err(TypedAstError::InvalidSourceStatement)
    );
    pristine();
    // Negative receivers alone use public reconstruction; no native pair can travel in public parts.
    for control in [
        "missing-primary",
        "missing-atomic",
        "missing-set",
        "enriched",
        "resolved-rootless",
        "typed-rootless",
        "rootless",
        "punctuation",
        "substituted-primary",
        "substituted-atomic",
        "substituted-set",
    ] {
        let mut nodes = base
            .nodes()
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        if control == "punctuation" {
            let ast = target.0.ast.as_ref().unwrap();
            let token = ast
                .node_views()
                .find(|view| matches!(view.kind(), K::Token(token) if token.text.as_ref() == ";"))
                .unwrap();
            let resolved = target.1.resolved_node_for(token.id()).unwrap();
            let row = nodes
                .iter_mut()
                .find(|row| row.resolved_node == Some(resolved))
                .unwrap();
            assert_eq!(row.kind.as_str(), format!("{:?}", token.kind()));
            row.kind = mizar_checker::typed_ast::TypedNodeKind::new(
                "Token(SurfaceToken { kind: ReservedSymbol, text: \",\" })",
            );
        }
        let mut types = base.types().clone();
        if control == "enriched" {
            let entry = lowers[0]
                .as_ref()
                .unwrap()
                .1
                .type_entries()
                .iter()
                .next()
                .unwrap()
                .1;
            types = TypeTable::new();
            types.insert(TypeEntryDraft {
                owner: entry.owner.clone(),
                expected: entry.expected,
                actual: entry.actual,
                status: entry.status,
                provenance: entry.provenance.clone(),
            });
        }
        let mut receiver = TypedAst::try_new(TypedAstParts {
            source_id: base.source_id(),
            module_id: base.module_id().clone(),
            resolved_root: if ["resolved-rootless", "rootless"].contains(&control) {
                None
            } else {
                base.resolved_root()
            },
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: TypedArena::try_new(
                if ["typed-rootless", "rootless"].contains(&control) {
                    None
                } else {
                    base.nodes().root()
                },
                nodes,
            )
            .unwrap(),
            contexts: base.contexts().clone(),
            types,
            facts: base.facts().clone(),
            coercions: base.coercions().clone(),
            initial_obligations: base.initial_obligations().clone(),
            diagnostics: base.diagnostics().clone(),
        })
        .unwrap();
        let alternate = &lowers[2].as_ref().unwrap().2;
        if control == "substituted-primary" {
            assert_eq!(
                receiver.with_source_term(alternate.source_term().unwrap().clone()),
                Err(TypedAstError::InvalidSourceTerm)
            );
            pristine();
            continue;
        }
        if control != "missing-primary" {
            receiver = receiver
                .with_source_term(base.source_term().unwrap().clone())
                .unwrap();
            if control == "substituted-atomic" {
                assert_eq!(
                    receiver.with_source_atomic_formula(
                        alternate.source_atomic_formula().unwrap().clone()
                    ),
                    Err(TypedAstError::InvalidSourceAtomicFormula)
                );
                pristine();
                continue;
            }
            if control != "missing-atomic" {
                receiver = receiver
                    .with_source_atomic_formula(base.source_atomic_formula().unwrap().clone())
                    .unwrap();
            }
            if control == "substituted-set" {
                assert_eq!(
                    receiver.with_source_set_term(alternate.source_set_term().unwrap().clone()),
                    Err(TypedAstError::InvalidSourceSetTerm)
                );
                pristine();
                continue;
            }
            if ["resolved-rootless", "typed-rootless", "rootless"].contains(&control) {
                let before = receiver.clone();
                assert!(
                    receiver
                        .nodes()
                        .iter()
                        .any(|(_, row)| row.kind.as_str() == "TheoremItem")
                );
                assert_eq!(
                    receiver.clone().with_source_statement(statement.clone()),
                    Err(TypedAstError::InvalidSourceStatement)
                );
                assert_eq!(receiver, before);
                pristine();
                assert_eq!(
                    receiver
                        .clone()
                        .with_source_statement_witnesses(statement.clone(), witnesses.clone()),
                    Err(TypedAstError::InvalidSourceStatement)
                );
                assert_eq!(receiver, before);
                pristine();
                assert_eq!(
                    receiver.clone().with_source_set_term_statement_witnesses(
                        base.source_set_term().unwrap().clone(),
                        statement.clone(),
                        witnesses.clone()
                    ),
                    Err(TypedAstError::InvalidSourceStatement)
                );
                assert_eq!(receiver, before);
                pristine();
            }
            if control != "missing-set" {
                receiver = receiver
                    .with_source_set_term(base.source_set_term().unwrap().clone())
                    .unwrap();
            }
        }
        let before = receiver.clone();
        assert_ne!(&receiver, base, "actual receiver change {control}");
        // Punctuation keeps a valid graph and all lower handoffs: the canonical-base gate rejects it.
        assert_eq!(
            receiver
                .clone()
                .with_source_statement_from_original(&target.1, &target.2, &target.3),
            Err(TypedAstError::InvalidSourceStatement),
            "canonical receiver {control}"
        );
        assert_eq!(receiver, before);
        pristine();
    }
    for (name, witness) in [
        ("grouped singleton", "({x})"),
        ("grouped element", "{(x)}"),
        ("multiple elements", "{x,x}"),
        ("nested element", "{{x}}"),
        ("numeric element", "{0}"),
        ("named", "item = {x}"),
        ("multiple witnesses", "{x},{x}"),
        ("choice", "the set"),
        ("qua", "x qua set"),
        ("comprehension", "{x where x is set}"),
    ] {
        let text = original.replace("{x}", witness);
        let frontend = capture(&text, false, false);
        assert!(
            frontend.diagnostics.is_empty(),
            "{name}: {:?}",
            frontend.diagnostics
        );
        assert!(frontend.preprocessed.import_stubs.is_empty());
        assert_eq!(frontend.source.source_text.as_ref(), text);
        let ast = frontend.ast.as_ref().unwrap();
        assert!(ast.node_views().all(|node| !node.is_recovered()), "{name}");
        let module = ModuleId::new(
            frontend.source.package_id.clone(),
            frontend.source.module_path.clone(),
        );
        let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
        let (scope, labels, resolution) = source_proof_inputs(&source, &symbols)?;
        source.validate_against(ast, &module).unwrap();
        validate_source_proof_occurrences(&source, &symbols, &scope)?;
        let before = (
            source.clone(),
            symbols.clone(),
            scope.clone(),
            labels.clone(),
            resolution.clone(),
            ast.snapshot_text(),
            frontend.source.clone(),
        );
        let lower = TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope);
        if name == "multiple elements" {
            // EN13.4.1/13.8.6 retain both occurrences in the finite term list.
            let (_, inference, base) = lower.as_ref().map_err(|error| error.to_owned())?;
            let primary = base.source_term().unwrap();
            let set = base.source_set_term().unwrap();
            assert_eq!(
                (
                    primary.terms().len(),
                    primary.references().len(),
                    set.edges().len()
                ),
                (6, 6, 2)
            );
            assert_eq!(set.primary_term_fingerprint(), primary.debug_text());
            assert_eq!(set.terms().iter().next().unwrap().1.spelling(), "{ x , x }");
            assert!(inference.facts().is_empty() && inference.diagnostics().is_empty());
            let pair = base
                .clone()
                .with_source_statement_from_original(&source, &symbols, &scope)
                .expect("finite native pair");
            let resolved =
                super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
                    &pair,
                    Vec::new(),
                )?;
            assert_eq!(pair.source_set_term(), base.source_set_term());
            assert_eq!(resolved.source_set_term(), pair.source_set_term());
            assert_eq!(resolved.source_statement(), pair.source_statement());
            assert_eq!(
                resolved.source_statement_witnesses(),
                pair.source_statement_witnesses()
            );
            assert!(
                resolved.checked_formulas().is_empty()
                    && resolved.statement_semantics().is_empty()
                    && resolved.checked_proofs().is_empty()
            );
            assert_eq!(
                base.clone()
                    .with_source_statement_from_original(&source, &symbols, &scope),
                Ok(pair)
            );
            assert_eq!(
                TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope)
                    .as_ref(),
                Ok(lower.as_ref().unwrap())
            );
        } else if name == "choice" {
            let (bindings, inference, base) = lower.as_ref().map_err(|error| error.to_owned())?;
            assert_eq!(
                (inference.source_id(), inference.module_id()),
                (source.source_id(), source.module())
            );
            let primary = base.source_term().unwrap();
            let set = base.source_set_term().unwrap();
            assert_eq!(
                (
                    bindings.bindings().len(),
                    primary.terms().len(),
                    primary.references().len(),
                    set.terms().len(),
                    set.type_sites().len(),
                    set.requests().len()
                ),
                (1, 4, 4, 1, 1, 2)
            );
            assert_eq!(set.primary_term_fingerprint(), primary.debug_text());
            assert!(
                set.application_fingerprint().is_none() && set.structure_fingerprint().is_none()
            );
            assert!(
                set.edges().is_empty()
                    && set.wrappers().is_empty()
                    && set.generators().is_empty()
                    && set.conditions().is_empty()
            );
            let (choice_id, choice) = set.terms().iter().next().unwrap();
            assert_eq!(
                (
                    choice.kind(),
                    choice.spelling(),
                    choice.source_ordinal(),
                    choice.recovery()
                ),
                (
                    SourceSetTermKind::Choice,
                    "the set",
                    0,
                    SourceSetTermRecovery::Normal
                )
            );
            let actual_choice = ast
                .node_views()
                .find(|view| view.kind() == &K::ChoiceTerm)
                .unwrap();
            let target_id = actual_choice.children()[1];
            let target = ast.node(target_id).unwrap();
            assert_eq!(target.kind, K::TypeExpression);
            let head_id = target.children[0];
            assert_eq!(ast.node(head_id).unwrap().kind, K::TypeHead);
            let (site_id, site) = set.type_sites().iter().next().unwrap();
            assert_eq!(
                site.owner(),
                SourceSetTypeOwner::Term {
                    term: choice_id,
                    role: SourceSetTypeRole::ChoiceTarget
                }
            );
            assert_eq!(
                (
                    site.source_range(),
                    site.head_range(),
                    site.head(),
                    site.spelling(),
                    site.head_spelling(),
                    site.context(),
                    site.recovery()
                ),
                (
                    target.range,
                    ast.node(head_id).unwrap().range,
                    SourceSetTypeHead::BuiltinSet,
                    "set",
                    "set",
                    choice.context(),
                    SourceSetTermRecovery::Normal
                )
            );
            for (typed_site, actual_id, actual_range) in [
                (choice.site(), actual_choice.id(), actual_choice.range()),
                (site.site(), target_id, target.range),
                (site.head_site(), head_id, ast.node(head_id).unwrap().range),
            ] {
                let node = base.nodes().node(typed_site.node()).unwrap();
                assert_eq!(node.resolved_node, source.resolved_node_for(actual_id));
                assert_eq!(
                    node.anchor,
                    mizar_session::SourceAnchor::Range(actual_range)
                );
            }
            for ((_, request), (ordinal, kind, type_site)) in set.requests().iter().zip([
                (0, SourceSetRequestKind::ChoiceNonempty, Some(site_id)),
                (1, SourceSetRequestKind::ResultType, None),
            ]) {
                assert_eq!(
                    (
                        request.term(),
                        request.ordinal(),
                        request.kind(),
                        request.generator(),
                        request.type_site()
                    ),
                    (choice_id, ordinal, kind, None, type_site)
                );
            }
            for (((id, term), (_, reference)), occurrence) in primary
                .terms()
                .iter()
                .zip(primary.references().iter())
                .zip(scope.references())
            {
                assert_eq!(
                    (
                        term.kind(),
                        term.source_ordinal(),
                        term.source_range(),
                        term.spelling()
                    ),
                    (
                        SourcePrimaryTermKind::VariableReference,
                        id.index(),
                        occurrence.range(),
                        occurrence.spelling()
                    )
                );
                assert_eq!(
                    (
                        reference.term(),
                        reference.use_ordinal(),
                        reference.lexical_scope()
                    ),
                    (id, occurrence.ordinal(), Some(occurrence.scope()))
                );
                assert_eq!(
                    base.nodes().node(term.site().node()).unwrap().resolved_node,
                    source.resolved_node_for(occurrence.node())
                );
            }
            assert_eq!(
                (
                    inference.terms().len(),
                    inference.formulas().len(),
                    inference.type_entries().len(),
                    inference.normalized_types().len()
                ),
                (5, 2, 10, 1)
            );
            let (normalized_id, normalized) = inference.normalized_types().iter().next().unwrap();
            assert_eq!(
                (&normalized.head, normalized.status),
                (&TypeHeadRef::BuiltinSet, NormalizedTypeStatus::Known)
            );
            assert!(
                normalized.args.is_empty()
                    && normalized.attributes.positive().is_empty()
                    && normalized.attributes.negative().is_empty()
            );
            assert!(
                inference
                    .type_entries()
                    .iter()
                    .all(|(_, row)| row.status == TypeStatus::Known
                        && row.actual == TypeEntryActual::Known(normalized_id))
            );
            assert_eq!(
                inference
                    .type_entries()
                    .iter()
                    .filter(|(_, row)| row.owner.node() == site.site().node())
                    .count(),
                1
            );
            for (_, row) in inference.terms().iter() {
                assert_eq!(row.status, TermStatus::Inferred);
                assert!(row.deferred.is_empty() && row.candidate_set.is_none());
                assert_eq!(
                    inference.type_entries().get(row.type_entry).unwrap().owner,
                    row.site
                );
                if &row.site == choice.site() {
                    assert_eq!(
                        (row.kind, row.context, row.reference.clone()),
                        (TermKind::Choice, choice.context(), None)
                    );
                } else {
                    assert_eq!(row.kind, TermKind::Variable);
                    assert!(matches!(row.reference, Some(TermReference::Binding(_))));
                }
            }
            assert_eq!(
                inference
                    .terms()
                    .iter()
                    .filter(|(_, row)| &row.site == choice.site())
                    .count(),
                1
            );
            for (_, row) in inference.formulas().iter() {
                assert_eq!(row.status, FormulaStatus::Checked);
                assert!(
                    row.deferred.is_empty() && row.facts.is_empty() && row.candidate_set.is_none()
                );
            }
            assert!(
                inference.diagnostics().is_empty()
                    && inference.facts().is_empty()
                    && inference.candidate_sets().is_empty()
            );
            assert!(
                base.types().is_empty()
                    && base.contexts().is_empty()
                    && base.facts().is_empty()
                    && base.coercions().is_empty()
                    && base.initial_obligations().is_empty()
                    && base.diagnostics().is_empty()
            );
            let base_before = base.clone();
            let pair = base
                .clone()
                .with_source_statement_from_original(&source, &symbols, &scope)
                .unwrap();
            let resolved =
                super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
                    &pair,
                    Vec::new(),
                )?;
            assert_eq!(resolved.source_statement(), pair.source_statement());
            assert_eq!(
                resolved.source_statement_witnesses(),
                pair.source_statement_witnesses()
            );
            assert_eq!(resolved.source_set_term(), base.source_set_term());
            assert!(
                resolved.checked_formulas().is_empty()
                    && resolved.statement_semantics().is_empty()
                    && resolved.checked_proofs().is_empty()
            );
            assert_eq!(
                base.clone()
                    .with_source_statement_from_original(&source, &symbols, &scope),
                Ok(pair.clone())
            );
            assert_eq!(
                super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
                    &pair.clone(),
                    Vec::new()
                )?,
                resolved
            );
            assert_eq!(base, &base_before);
            assert_eq!(
                TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope)
                    .as_ref(),
                Ok(lower.as_ref().unwrap())
            );
        } else {
            assert_eq!(
                lower.unwrap_err(),
                "theorems.equality.unsupported_source_types",
                "{name}"
            );
        }
        assert_eq!(
            (
                source,
                symbols,
                scope,
                labels,
                resolution,
                ast.snapshot_text(),
                frontend.source
            ),
            before
        );
        pristine();
    }
    let recovered_text = original.replace("take {x};", "take {x}");
    let recovered = capture(&recovered_text, false, false);
    assert_eq!(recovered.source.source_text.as_ref(), recovered_text);
    assert!(recovered.diagnostics.iter().any(|diagnostic| matches!(&diagnostic.code,
        mizar_frontend::orchestration::DiagnosticCode::Syntax(code) if code.as_ref() == "missing_semicolon")));
    let ast = recovered.ast.as_ref().unwrap();
    let module = ModuleId::new(
        recovered.source.package_id.clone(),
        recovered.source.module_path.clone(),
    );
    let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
    let (scope, labels, resolution) = source_proof_inputs(&source, &symbols)?;
    let before = (
        source.clone(),
        symbols.clone(),
        scope.clone(),
        labels.clone(),
        resolution.clone(),
        ast.snapshot_text(),
        recovered.source.clone(),
    );
    assert_eq!(
        TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope).unwrap_err(),
        "theorems.equality.unsupported_source_types"
    );
    assert_eq!(
        (
            source,
            symbols,
            scope,
            labels,
            resolution,
            ast.snapshot_text(),
            recovered.source
        ),
        before
    );
    pristine();
    let alias = capture(
        &original.replace("take {x};", "take {unbound};"),
        false,
        false,
    );
    assert!(alias.diagnostics.is_empty() && alias.preprocessed.import_stubs.is_empty());
    let ast = alias.ast.as_ref().unwrap();
    let module = ModuleId::new(
        alias.source.package_id.clone(),
        alias.source.module_path.clone(),
    );
    let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
    let before = (
        source.clone(),
        symbols.clone(),
        ast.snapshot_text(),
        alias.source.clone(),
    );
    assert_eq!(
        source_proof_inputs(&source, &symbols).unwrap_err(),
        "source proof occurrences do not match the original source"
    );
    assert_eq!((source, symbols, ast.snapshot_text(), alias.source), before);
    pristine();
    let foreign = capture(original, true, true);
    assert!(foreign.diagnostics.is_empty() && foreign.preprocessed.import_stubs.is_empty());
    assert_eq!(foreign.source.source_text.as_ref(), original);
    assert_eq!(fs::read_to_string(&path).unwrap(), "stale disk");
    let ast = foreign.ast.as_ref().unwrap();
    let module = ModuleId::new(
        foreign.source.package_id.clone(),
        foreign.source.module_path.clone(),
    );
    let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
    let (scope, labels, resolution) = source_proof_inputs(&source, &symbols)?;
    source.validate_against(ast, &module).unwrap();
    validate_source_proof_occurrences(&source, &symbols, &scope)?;
    let foreign_lower =
        TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope)?;
    let own = &cases[0];
    let base = &lowers[0].as_ref().unwrap().2;
    assert_ne!(source.source_id(), own.1.source_id());
    assert_ne!(source.module(), own.1.module());
    let before = (
        source.clone(),
        symbols.clone(),
        scope.clone(),
        labels.clone(),
        resolution.clone(),
        ast.snapshot_text(),
        foreign.source.clone(),
        foreign_lower.clone(),
    );
    for (source, symbols, scope) in [
        (&own.1, &symbols, &own.3),
        (&own.1, &own.2, &scope),
        (&source, &own.2, &scope),
    ] {
        assert!(
            TermFormulaChecker::check_source_theorem_equalities(source, symbols, scope).is_err()
        );
        assert_eq!(
            base.clone()
                .with_source_statement_from_original(source, symbols, scope),
            Err(TypedAstError::InvalidSourceStatement)
        );
        pristine();
    }
    assert_eq!(
        base.clone()
            .with_source_statement_from_original(&source, &symbols, &scope),
        Err(TypedAstError::InvalidSourceStatement)
    );
    pristine();
    let foreign_pair = foreign_lower
        .2
        .clone()
        .with_source_statement_from_original(&source, &symbols, &scope)
        .unwrap();
    let foreign_final = super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
        &foreign_pair,
        Vec::new(),
    )?;
    assert_eq!(
        foreign_pair.source_set_term(),
        foreign_lower.2.source_set_term()
    );
    assert_eq!(
        foreign_final.source_statement(),
        foreign_pair.source_statement()
    );
    assert_eq!(
        foreign_final.source_statement_witnesses(),
        foreign_pair.source_statement_witnesses()
    );
    assert_eq!(
        foreign_lower
            .2
            .clone()
            .with_source_statement_from_original(&source, &symbols, &scope),
        Ok(foreign_pair.clone())
    );
    assert_eq!(
        super::super::super::type_elaboration::assemble_empty_resolved_typed_ast(
            &foreign_pair.clone(),
            Vec::new()
        )?,
        foreign_final
    );
    assert_eq!(
        TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope).as_ref(),
        Ok(&foreign_lower)
    );
    assert_eq!(
        (
            source,
            symbols,
            scope,
            labels,
            resolution,
            ast.snapshot_text(),
            foreign.source,
            foreign_lower
        ),
        before
    );
    pristine();
    fs::remove_dir_all(root).unwrap();
    Ok(())
}
