use super::*;

#[test]
fn source_theorem_equality_producer_preserves_real_typed_inputs_and_consumers() -> Result<(), String>
{
    use mizar_checker::{type_checker::*, typed_ast::*};
    use mizar_core::elaborator::lower_source_theorem_skeletons;
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::NamespacePath,
        labels::{LabelResolver, ProofLabelSourceCollector},
        names::{SourceVariableBindingKind, SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::SignatureProjectionExtractor,
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceInput,
        SourceOrigin, SourceOriginInput, hash_text,
    };
    use mizar_syntax::SurfaceNodeKind as K;
    let fixture = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let sources = [
        fixture.to_owned(),
        fixture.replace("Lem1", "Seed").replace("let X be set;", "let Local be set;").replace("thus X = X", "thus Local = Local"),
        "theorem Pair: for A, Spare, B being set holds A = B proof let L, Unused, R be set; thus L = R; end;".into(),
        "theorem Nested: for A being set for Unused being set holds for B being set holds A = B proof let L be set; let Idle be set; let R be set; thus L = R; end;".into(),
        "theorem Groups: for A being set, Unused, B being set holds A = B proof let L be set, Idle, R be set; thus L = R; end;".into(),
    ];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/owners.miz");
    let ids = InMemorySessionIdAllocator::new();
    for (case, text) in sources.iter().enumerate() {
        for buffer in [false, true] {
            fs::write(&path, if buffer { "stale disk content" } else { text }).unwrap();
            let input = SourceInput {
                package_id: PackageId::new("equality-test"),
                module_path: ModulePath::new("owners"),
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
            };
            let frontend = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
                super::super::super::ParseOnlyImportProvider,
                MizarParserSeam,
            )
            .run(
                SourceUnitRequest {
                    snapshot: snapshot_id(777),
                    input,
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
            assert_eq!(frontend.source.source_hash, hash_text(text));
            assert_eq!(
                matches!(
                    frontend.source.origin,
                    SourceOrigin::OpenBuffer { version: 3 }
                ),
                buffer
            );
            let ast = frontend.ast.as_ref().unwrap();
            let module = ModuleId::new(
                frontend.source.package_id.clone(),
                frontend.source.module_path.clone(),
            );
            let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
            let shells = DeclarationShellCollector::new(ast, &module).collect();
            let collected = SignatureProjectionExtractor::new(
                ast,
                &shells,
                NamespacePath::new(module.path().as_str()),
            )
            .collect(&module);
            assert!(collected.diagnostics().is_empty());
            let symbols = collected.env();
            let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                SourceVariableScopeInput::new(ast, &module, symbols),
            )
            .unwrap();
            let before = (
                source.clone(),
                symbols.clone(),
                scope.clone(),
                ast.snapshot_text(),
                frontend.source.clone(),
            );
            let (bindings, inference, typed) =
                TermFormulaChecker::check_source_theorem_equalities(&source, symbols, &scope)?;
            let replay =
                TermFormulaChecker::check_source_theorem_equalities(&source, symbols, &scope)?;
            assert_eq!(bindings, replay.0);
            assert_eq!(inference, replay.1);
            assert_eq!(typed, replay.2);
            assert_eq!(
                bindings,
                SourceVariableSemanticsChecker::occurrence_binding_env(&scope)
            );
            assert_eq!(typed.source_id(), ast.source_id);
            assert_eq!(typed.module_id(), &module);
            assert_eq!(typed.nodes().len(), ast.nodes().len());
            assert_eq!(
                typed.nodes().root().unwrap().index(),
                ast.root().unwrap().index()
            );
            assert_eq!(typed.resolved_root(), Some(source.arena().root()));
            let oracle = super::super::super::type_elaboration::step5c8_formula_typed_ast(
                ast, &module, symbols, &scope, &bindings, true,
            )?;
            for (id, neutral) in source.arena().iter() {
                let actual = typed.nodes().node(TypedNodeId::new(id.index())).unwrap();
                let expected = oracle
                    .nodes()
                    .node(TypedNodeId::new(id.index()))
                    .unwrap()
                    .clone()
                    .with_resolved_node(id);
                assert_eq!(actual, &expected);
                assert_eq!(actual.resolved_node, Some(id));
                assert_eq!(actual.anchor, *neutral.origin().anchor());
                assert!(
                    actual
                        .children
                        .iter()
                        .map(|id| id.index())
                        .eq(neutral.children().iter().map(|id| id.index()))
                );
                assert_eq!(actual.recovery, NodeRecoveryState::Normal);
                assert_eq!(actual.typing, TypingState::Unknown);
                assert_eq!(actual.links, TypedNodeLinks::default());
            }
            let declarations = ast.node_views().filter(|view| matches!(view.kind(), K::QuantifierVariableSegment | K::QualifiedVariableSegment))
                .flat_map(|view| view.child_views().filter(|child| matches!(child.kind(), K::Token(token) if token.kind == mizar_syntax::SurfaceTokenKind::Identifier)).map(|child| child.id())).collect::<Vec<_>>();
            assert_eq!(
                scope
                    .bindings()
                    .iter()
                    .map(|binding| binding.node())
                    .collect::<Vec<_>>(),
                declarations
            );
            for binding in scope.bindings() {
                assert_eq!(binding.range(), ast.node(binding.node()).unwrap().range);
                assert_eq!(
                    ast.node(binding.node()).unwrap().token_text(),
                    Some(binding.spelling())
                );
            }
            let primary = typed.source_term().unwrap();
            let atomic = typed.source_atomic_formula().unwrap();
            assert_eq!(Some(primary), oracle.source_term());
            assert_eq!(Some(atomic), oracle.source_atomic_formula());
            assert_eq!(primary.terms().len(), scope.references().len());
            assert_eq!(primary.references().len(), scope.references().len());
            assert!(primary.numeric_type_requests().is_empty());
            assert_eq!(inference.terms().len(), scope.references().len());
            assert!(
                inference.diagnostics().is_empty()
                    && inference.facts().is_empty()
                    && inference.candidate_sets().is_empty()
            );
            let mut declared_ranges = Vec::new();
            let mut expected_roles = Vec::new();
            for ((term_id, term), occurrence) in primary.terms().iter().zip(scope.references()) {
                assert_eq!(term.source_ordinal(), term_id.index());
                assert_eq!(term.site().node().index(), occurrence.node().index());
                assert_eq!(term.source_range(), occurrence.range());
                assert_eq!(term.spelling(), occurrence.spelling());
                let reference = primary
                    .references()
                    .iter()
                    .find(|(_, reference)| reference.term() == term_id)
                    .unwrap()
                    .1;
                assert_eq!(reference.binding().index(), occurrence.binding().index());
                assert_eq!(reference.lexical_scope(), Some(occurrence.scope()));
                assert_eq!(reference.use_ordinal(), occurrence.ordinal());
                let checked = inference
                    .terms()
                    .iter()
                    .find(|(_, checked)| checked.site == *term.site())
                    .unwrap()
                    .1;
                assert_eq!(
                    checked.reference,
                    Some(TermReference::Binding(reference.binding()))
                );
                assert_eq!(checked.status, TermStatus::Inferred);
                assert_eq!(checked.kind, TermKind::Variable);
                assert!(checked.deferred.is_empty() && checked.candidate_set.is_none());
                let entry = inference.type_entries().get(checked.type_entry).unwrap();
                let TypeEntryActual::Known(actual) = entry.actual else {
                    panic!("unknown actual type")
                };
                assert_eq!(entry.owner, checked.site);
                assert_eq!(entry.status, TypeStatus::Known);
                let normalized = inference.normalized_types().get(actual).unwrap();
                assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
                assert_eq!(normalized.status, NormalizedTypeStatus::Known);
                assert!(
                    normalized.args.is_empty()
                        && normalized.attributes.positive().is_empty()
                        && normalized.attributes.negative().is_empty()
                );
                assert_eq!(normalized.source.spelling, "set");
                let binding = scope
                    .bindings()
                    .iter()
                    .find(|binding| binding.id() == occurrence.binding())
                    .unwrap();
                let segment = ast
                    .node_views()
                    .find(|view| {
                        matches!(
                            view.kind(),
                            K::QuantifierVariableSegment | K::QualifiedVariableSegment
                        ) && view.children().contains(&binding.node())
                    })
                    .unwrap();
                let written = segment
                    .child_views()
                    .filter(|view| matches!(view.kind(), K::TypeExpression))
                    .collect::<Vec<_>>();
                let [written] = written.as_slice() else {
                    panic!("missing unique declared type")
                };
                let owner = TypedSiteRef::Role {
                    node: TypedNodeId::new(written.id().index()),
                    role: TypeRole::new(format!("theorem.equality.use:{}", term_id.index())),
                };
                let rows = inference
                    .type_entries()
                    .iter()
                    .filter(|(_, row)| row.owner == owner)
                    .collect::<Vec<_>>();
                let [(_, declared)] = rows.as_slice() else {
                    panic!("missing unique type input row")
                };
                assert_eq!(declared.actual, entry.actual);
                assert_eq!(declared.status, TypeStatus::Known);
                expected_roles.push(owner);
                declared_ranges.push(written.range());
            }
            let mut actual_roles = inference
                .type_entries()
                .iter()
                .filter(|(_, entry)| matches!(entry.owner, TypedSiteRef::Role { .. }))
                .map(|(_, entry)| entry.owner.clone())
                .collect::<Vec<_>>();
            actual_roles.sort();
            expected_roles.sort();
            assert_eq!(actual_roles, expected_roles);
            let representative = declared_ranges
                .iter()
                .min_by_key(|range| (range.start, range.end))
                .unwrap();
            for (_, normalized) in inference.normalized_types().iter() {
                assert_eq!(normalized.source.range, *representative);
            }
            for quantified in scope
                .bindings()
                .iter()
                .filter(|binding| binding.kind() == SourceVariableBindingKind::Quantifier)
            {
                for local in scope
                    .bindings()
                    .iter()
                    .filter(|binding| binding.kind() == SourceVariableBindingKind::Let)
                {
                    assert_ne!(quantified.id(), local.id());
                    assert_ne!(quantified.node(), local.node());
                    assert_ne!(quantified.scope(), local.scope());
                }
            }
            let atoms = ast
                .node_views()
                .filter(|view| matches!(view.kind(), K::BuiltinPredicateApplication))
                .collect::<Vec<_>>();
            assert_eq!(atomic.formulas().len(), atoms.len());
            assert_eq!(atomic.edges().len(), atoms.len() * 2);
            assert_eq!(atomic.requests().len(), atoms.len() * 2);
            assert_eq!(inference.formulas().len(), atoms.len());
            for atom in atoms {
                let expected = [atom.children()[0], atom.children()[2]].map(|id| {
                    TypedSiteRef::Node(TypedNodeId::new(ast.node(id).unwrap().children[0].index()))
                });
                let checked = inference
                    .formulas()
                    .iter()
                    .find(|(_, formula)| formula.site.node().index() == atom.id().index())
                    .unwrap()
                    .1;
                assert_eq!(checked.terms, expected);
                assert_eq!(checked.source_range, atom.range());
                assert_eq!(checked.kind, FormulaKind::Equality);
                assert_eq!(checked.status, FormulaStatus::Checked);
                assert!(
                    checked.deferred.is_empty()
                        && checked.facts.is_empty()
                        && checked.candidate_set.is_none()
                );
                let (id, _) = atomic
                    .formulas()
                    .iter()
                    .find(|(_, formula)| formula.site() == &checked.site)
                    .unwrap();
                let edges = atomic
                    .edges()
                    .iter()
                    .filter(|(_, edge)| edge.formula() == id)
                    .map(|(_, edge)| {
                        let mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                            term,
                        ) = edge.target()
                        else {
                            panic!("foreign operand")
                        };
                        primary.terms().get(term).unwrap().site().clone()
                    })
                    .collect::<Vec<_>>();
                assert_eq!(edges, expected);
            }
            assert!(
                typed.types().is_empty()
                    && typed.facts().is_empty()
                    && typed.initial_obligations().is_empty()
                    && typed.diagnostics().is_empty()
            );
            let owner = symbols.symbols().iter().next().unwrap();
            let namespace = NamespacePath::new(module.path().as_str());
            let labels = ProofLabelSourceCollector::new(
                ast,
                &module,
                namespace.clone(),
                owner.contribution(),
                &source,
            )
            .and_then(|collector| collector.collect_with_theorem_owners(symbols))
            .unwrap();
            let resolved = LabelResolver::new(labels.projections()).resolve(
                &module,
                &namespace,
                labels.references(),
            );
            let checked = SourceVariableSemanticsChecker::check_theorem_skeletons(
                &source, &typed, &scope, symbols, &labels, &resolved, None,
            );
            if case < 2 {
                let checked = checked?;
                let legacy = SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &source, &oracle, &scope, symbols, &labels, &resolved, None,
                )?;
                let core = lower_source_theorem_skeletons(&checked, None)?;
                let expected = lower_source_theorem_skeletons(&legacy, None)?;
                assert_eq!(core, expected);
                let vcs = generate_core_vcs(&core, snapshot_id(777))?;
                assert_eq!(vcs, generate_core_vcs(&expected, snapshot_id(777))?);
                assert_eq!(vcs.vcs().len(), 2);
                assert!(
                    vcs.vcs()
                        .iter()
                        .all(|vc| vc.status == mizar_vc::vc_ir::VcStatus::Open)
                );
                assert_eq!(core.source_id(), source.source_id());
                assert_eq!(core.module_id(), source.module());
                assert_eq!(labels.references().len(), 1);
                assert_ne!(vcs.vcs()[0].local_context, vcs.vcs()[1].local_context);
            } else {
                assert!(checked.is_err());
                assert!(scope.bindings().iter().any(|binding| {
                    !scope
                        .references()
                        .iter()
                        .any(|reference| reference.binding() == binding.id())
                }));
                assert!(inference.formulas().iter().any(|(_, formula)| {
                    let references = formula
                        .terms
                        .iter()
                        .map(|site| {
                            scope
                                .references()
                                .iter()
                                .find(|reference| reference.node().index() == site.node().index())
                                .unwrap()
                                .binding()
                        })
                        .collect::<Vec<_>>();
                    references[0] != references[1]
                }));
            }
            assert_eq!(
                before,
                (
                    source,
                    symbols.clone(),
                    scope,
                    ast.snapshot_text(),
                    frontend.source.clone()
                )
            );
        }
    }
    fs::remove_file(path).unwrap();
    fs::remove_dir(root.join("src")).unwrap();
    fs::remove_dir(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_equality_producer_transports_reserved_source() -> Result<(), String> {
    use mizar_checker::{
        binding_env::{BinderIdentity, BindingKind},
        resolved_typed_ast::{ExprId, ExpressionMetadataInput},
        source_atomic_formula::*,
        source_term::*,
        type_checker::*,
        typed_ast::*,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        names::{SourceVariableBindingKind, SourceVariableTypeRadix},
        resolved_ast::ModuleId,
        symbols::{
            source_proof_inputs, source_theorem_module_original_inputs,
            validate_source_proof_occurrences,
        },
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceInput,
        SourceOrigin, SourceOriginInput, hash_text,
    };
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind};
    let fixture = include_str!(
        "../../../../../../tests/miz/pass/types/pass_type_elaboration_reserved_variable_equality_001.miz"
    );
    let texts = [
        fixture.to_owned(),
        fixture.replace(" x", " Parcel").replace("ReservedVariableEqualityPayloadBoundary", "RenamedReserve"),
        "reserve\n  arbitrary\tfor :: written type follows\n set;\ntheorem Layout: :: proposition follows\n arbitrary\t=\n arbitrary;\n".to_owned(),
    ];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src").join("reserved.miz");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let mut cases = Vec::new();
    for text in &texts {
        for buffer in [false, true] {
            fs::write(&path, if buffer { "stale disk content" } else { text }).unwrap();
            let frontend = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
                super::super::super::ParseOnlyImportProvider,
                MizarParserSeam,
            )
            .run(
                SourceUnitRequest {
                    snapshot: snapshot_id(777),
                    input: SourceInput {
                        package_id: PackageId::new("reserved-equality-test"),
                        module_path: ModulePath::new("reserved"),
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
            assert_eq!(frontend.source.source_hash, hash_text(text));
            assert_eq!(
                matches!(
                    frontend.source.origin,
                    SourceOrigin::OpenBuffer { version: 3 }
                ),
                buffer
            );
            let ast = frontend.ast.as_ref().unwrap();
            let module = ModuleId::new(
                frontend.source.package_id.clone(),
                frontend.source.module_path.clone(),
            );
            let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
            let (scope, _, _) = source_proof_inputs(&source, &symbols)?;
            assert_eq!(
                validate_source_proof_occurrences(&source, &symbols, &scope),
                Ok(())
            );
            source.validate_against(ast, &module).unwrap();
            assert_eq!(source.source_id(), frontend.source.source_id);
            assert_eq!(scope.source_id(), source.source_id());
            assert_eq!(scope.module_id(), &module);
            let mut reachable = std::collections::BTreeSet::new();
            let mut pending = vec![ast.root().unwrap()];
            while let Some(id) = pending.pop() {
                if !reachable.insert(id) {
                    continue; // Root also lists the original token nodes.
                }
                let view = ast.node(id).unwrap();
                assert!(!view.recovered);
                pending.extend(view.children.iter().copied());
            }
            assert_eq!(reachable.len(), ast.nodes().len());
            let item_list = ast
                .node_views()
                .find(|view| matches!(view.kind(), K::ItemList))
                .unwrap();
            assert_eq!(item_list.children().len(), 2);
            let reserve = ast.node(item_list.children()[0]).unwrap();
            let theorem = ast.node(item_list.children()[1]).unwrap();
            assert_eq!(reserve.kind, K::ReserveItem);
            assert_eq!(theorem.kind, K::TheoremItem);
            let segment = ast
                .node_views()
                .find(|view| matches!(view.kind(), K::ReserveSegment))
                .unwrap();
            assert!(reserve.children.contains(&segment.id()));
            let [binding] = scope.bindings() else {
                panic!("one real reserve binding")
            };
            assert_eq!(binding.kind(), SourceVariableBindingKind::Reserve);
            assert_eq!(segment.children()[0], binding.node());
            let binder = ast.node(binding.node()).unwrap();
            assert_eq!(binder.range, binding.range());
            assert_eq!(binder.token_text(), Some(binding.spelling()));
            let declared = binding.declared_type().unwrap();
            assert_eq!(declared.radix(), SourceVariableTypeRadix::Set);
            assert!(declared.attributes().is_empty());
            let written = ast.node(*segment.children().last().unwrap()).unwrap();
            assert_eq!(written.kind, K::TypeExpression);
            assert_eq!(written.children.len(), 1);
            let head = ast.node(written.children[0]).unwrap();
            assert_eq!(head.kind, K::TypeHead);
            assert_eq!(head.children.len(), 1);
            let set = ast.node(head.children[0]).unwrap();
            assert!(
                matches!(&set.kind, K::Token(token) if token.kind == SurfaceTokenKind::ReservedWord && token.text.as_ref() == "set")
            );
            let equality = ast
                .node_views()
                .find(|view| matches!(view.kind(), K::BuiltinPredicateApplication))
                .unwrap();
            assert_eq!(scope.references().len(), 2);
            assert_ne!(scope.references()[0].id(), scope.references()[1].id());
            assert!(binding.range().end <= scope.references()[0].range().start);
            for (ordinal, reference) in scope.references().iter().enumerate() {
                assert_eq!(reference.binding(), binding.id());
                assert_eq!(reference.ordinal(), ordinal);
                assert_eq!(reference.spelling(), binding.spelling());
                let term = ast.node(equality.children()[ordinal * 2]).unwrap();
                assert_eq!(term.kind, K::TermExpression);
                assert_eq!(term.children, vec![reference.node()]);
                let occurrence = ast.node(reference.node()).unwrap();
                assert_eq!(occurrence.kind, K::TermReference);
                assert_eq!(occurrence.children.len(), 1);
                let token = ast.node(occurrence.children[0]).unwrap();
                assert_eq!(token.range, reference.range());
                assert_eq!(token.token_text(), Some(binding.spelling()));
            }
            let identifiers = ast.node_views().filter(|view| matches!(view.kind(), K::Token(token) if token.kind == SurfaceTokenKind::Identifier)).count();
            assert_eq!(identifiers, 4); // Reserve binder, theorem label and two uses.
            let before = (
                source.clone(),
                symbols.clone(),
                scope.clone(),
                ast.snapshot_text(),
                frontend.source.clone(),
            );
            // Collect every public native result after independently checking readiness.
            let result =
                TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope);
            cases.push((frontend, source, symbols, scope, before, result));
        }
    }
    let errors = cases
        .iter()
        .filter_map(|(_, _, _, _, _, result)| result.as_ref().err())
        .collect::<Vec<_>>();
    assert!(
        errors.is_empty(),
        "native reserved equality success REDs: {errors:?}"
    );
    for (frontend, source, symbols, scope, before, result) in cases {
        let (bindings, inference, typed) = result?;
        let ast = frontend.ast.as_ref().unwrap();
        assert_eq!(
            bindings,
            SourceVariableSemanticsChecker::occurrence_binding_env(&scope)
        );
        assert_eq!(bindings.bindings().len(), 1);
        let (_, projected) = bindings.bindings().iter().next().unwrap();
        let binding = &scope.bindings()[0];
        assert_eq!(projected.kind, BindingKind::ReservedVariable);
        assert_eq!(
            projected.identity,
            BinderIdentity::ReservedVariable {
                spelling: binding.spelling().into(),
                declaration_range: binding.range()
            }
        );
        assert!(bindings.diagnostics().is_empty());
        assert_eq!(typed.source_id(), source.source_id());
        assert_eq!(typed.module_id(), source.module());
        assert_eq!(typed.nodes().len(), source.arena().len());
        assert_eq!(typed.resolved_root(), Some(source.arena().root()));
        assert_eq!(
            typed.nodes().root().unwrap().index(),
            source.arena().root().index()
        );
        for (id, original) in source.arena().iter() {
            let actual = typed.nodes().node(TypedNodeId::new(id.index())).unwrap();
            let expected_kind = match original.kind() {
                K::TermReference => "source.term.variable-reference".to_owned(),
                K::BuiltinPredicateApplication => "source.formula.atomic.equality".to_owned(),
                kind => format!("{kind:?}"),
            };
            assert_eq!(actual.kind.as_str(), expected_kind.as_str());
            assert_eq!(actual.resolved_node, Some(id));
            assert_eq!(actual.anchor, *original.origin().anchor());
            assert!(
                actual
                    .children
                    .iter()
                    .map(|child| child.index())
                    .eq(original.children().iter().map(|child| child.index()))
            );
            assert_eq!(actual.recovery, NodeRecoveryState::Normal);
            assert_eq!(actual.typing, TypingState::Unknown);
            assert_eq!(actual.links, TypedNodeLinks::default());
        }
        assert!(
            typed.types().is_empty()
                && typed.facts().is_empty()
                && typed.coercions().is_empty()
                && typed.initial_obligations().is_empty()
                && typed.diagnostics().is_empty()
        );
        assert!(typed.source_context().is_none() && typed.source_type().is_none());
        let primary = typed.source_term().unwrap();
        let atomic = typed.source_atomic_formula().unwrap();
        assert_eq!(
            (primary.source_id(), primary.module_id()),
            (source.source_id(), source.module())
        );
        assert_eq!(
            (atomic.source_id(), atomic.module_id()),
            (source.source_id(), source.module())
        );
        assert_eq!(
            (
                primary.terms().len(),
                primary.references().len(),
                inference.terms().len()
            ),
            (2, 2, 2)
        );
        assert!(primary.numeric_type_requests().is_empty());
        assert_eq!(
            (
                atomic.formulas().len(),
                atomic.edges().len(),
                atomic.requests().len(),
                inference.formulas().len()
            ),
            (1, 2, 2, 1)
        );
        assert!(
            atomic.wrappers().is_empty()
                && atomic.candidates().is_empty()
                && atomic.predicate_segments().is_empty()
                && atomic.predicate_heads().is_empty()
                && atomic.type_sites().is_empty()
                && atomic.attributes().is_empty()
        );
        assert!(
            inference.diagnostics().is_empty()
                && inference.candidate_sets().is_empty()
                && inference.facts().is_empty()
        );
        let segment = ast
            .node_views()
            .find(|view| matches!(view.kind(), K::ReserveSegment))
            .unwrap();
        let written = ast.node(*segment.children().last().unwrap()).unwrap();
        for ((id, term), occurrence) in primary.terms().iter().zip(scope.references()) {
            assert_eq!(term.site().node().index(), occurrence.node().index());
            assert_eq!(term.source_range(), occurrence.range());
            assert_eq!(term.source_ordinal(), occurrence.ordinal());
            assert_eq!(term.spelling(), occurrence.spelling());
            assert_eq!(term.kind(), SourcePrimaryTermKind::VariableReference);
            assert_eq!(term.role(), SourcePrimaryTermRole::Value);
            assert_eq!(term.recovery(), SourcePrimaryTermRecovery::Normal);
            assert!(term.parent().is_none());
            let reference = primary
                .references()
                .iter()
                .find(|(_, row)| row.term() == id)
                .unwrap()
                .1;
            assert_eq!(reference.binding(), projected.id);
            assert_eq!(reference.role(), SourcePrimaryTermReferenceRole::Variable);
            assert_eq!(reference.lexical_scope(), Some(occurrence.scope()));
            assert_eq!(reference.use_ordinal(), occurrence.ordinal());
            let checked = inference
                .terms()
                .iter()
                .find(|(_, row)| &row.site == term.site())
                .unwrap()
                .1;
            assert_eq!(checked.status, TermStatus::Inferred);
            assert_eq!(checked.kind, TermKind::Variable);
            assert_eq!(
                checked.reference,
                Some(TermReference::Binding(projected.id))
            );
            assert!(checked.deferred.is_empty() && checked.candidate_set.is_none());
            let entry = inference.type_entries().get(checked.type_entry).unwrap();
            assert_eq!(entry.owner, checked.site);
            assert_eq!(entry.status, TypeStatus::Known);
            let TypeEntryActual::Known(actual) = entry.actual else {
                panic!("real known set")
            };
            let normalized = inference.normalized_types().get(actual).unwrap();
            assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
            assert_eq!(normalized.status, NormalizedTypeStatus::Known);
            assert!(
                normalized.args.is_empty()
                    && normalized.attributes.positive().is_empty()
                    && normalized.attributes.negative().is_empty()
            );
            assert_eq!(normalized.source.spelling, "set");
            assert_eq!(normalized.source.range, written.range);
            let owner = TypedSiteRef::Role {
                node: TypedNodeId::new(segment.children().last().unwrap().index()),
                role: TypeRole::new(format!("theorem.equality.use:{}", id.index())),
            };
            let declared = inference
                .type_entries()
                .iter()
                .filter(|(_, row)| row.owner == owner)
                .collect::<Vec<_>>();
            assert_eq!(declared.len(), 1);
            assert_eq!(declared[0].1.actual, entry.actual);
        }
        let (formula_id, formula) = atomic.formulas().iter().next().unwrap();
        let equality = ast
            .node_views()
            .find(|view| matches!(view.kind(), K::BuiltinPredicateApplication))
            .unwrap();
        assert_eq!(formula.site().node().index(), equality.id().index());
        assert_eq!(formula.source_range(), equality.range());
        assert_eq!(formula.source_ordinal(), 0);
        assert_eq!(formula.kind(), SourceAtomicFormulaKind::Equality);
        assert_eq!(formula.recovery(), SourceAtomicFormulaRecovery::Normal);
        assert_eq!(
            formula.spelling(),
            format!("{} = {}", binding.spelling(), binding.spelling())
        );
        let (_, checked) = inference.formulas().iter().next().unwrap();
        assert_eq!(checked.site, *formula.site());
        assert_eq!(checked.source_range, formula.source_range());
        assert_eq!(checked.kind, FormulaKind::Equality);
        assert_eq!(checked.status, FormulaStatus::Checked);
        assert!(
            checked.deferred.is_empty()
                && checked.facts.is_empty()
                && checked.candidate_set.is_none()
        );
        for (ordinal, ((edge_id, edge), (_, request))) in atomic
            .edges()
            .iter()
            .zip(atomic.requests().iter())
            .enumerate()
        {
            assert_eq!(edge.formula(), formula_id);
            assert_eq!(edge.ordinal(), ordinal);
            assert_eq!(
                edge.role(),
                if ordinal == 0 {
                    SourceAtomicEdgeRole::BuiltinLeftOperand
                } else {
                    SourceAtomicEdgeRole::BuiltinRightOperand
                }
            );
            let SourceAtomicTermTarget::Primary(term) = edge.target() else {
                panic!("native primary operand")
            };
            assert_eq!(
                checked.terms[ordinal],
                *primary.terms().get(term).unwrap().site()
            );
            assert_eq!(request.formula(), formula_id);
            assert_eq!(request.ordinal(), ordinal);
            assert_eq!(request.kind(), SourceAtomicRequestKind::OperandExpectedType);
            assert_eq!(request.edge(), Some(edge_id));
            assert!(
                request.candidate().is_none()
                    && request.type_site().is_none()
                    && request.attribute().is_none()
            );
        }
        let tables = TermFormulaChecker::check_source_theorem_equalities_with_typed_tables(
            &source, &symbols, &scope,
        )?;
        assert_eq!((&tables.0, &tables.1), (&bindings, &inference));
        assert_eq!(tables.2.types(), inference.type_entries());
        assert_eq!(tables.2.source_term(), Some(primary));
        assert_eq!(tables.2.source_atomic_formula(), Some(atomic));
        assert_eq!(tables.2.source_id(), typed.source_id());
        assert_eq!(tables.2.module_id(), typed.module_id());
        assert_eq!(tables.2.resolved_root(), typed.resolved_root());
        assert_eq!(tables.2.nodes().root(), typed.nodes().root());
        assert_eq!(tables.2.contexts(), typed.contexts());
        assert_eq!(tables.2.facts(), typed.facts());
        assert_eq!(tables.2.coercions(), typed.coercions());
        assert_eq!(tables.2.initial_obligations(), typed.initial_obligations());
        assert_eq!(tables.2.diagnostics(), typed.diagnostics());
        assert_eq!(tables.2.source_context(), typed.source_context());
        assert_eq!(tables.2.source_type(), typed.source_type());
        assert_eq!(tables.2.source_attribute(), typed.source_attribute());
        for (id, original) in typed.nodes().iter() {
            let mut expected = original.clone();
            if let Some((_, term)) = inference
                .terms()
                .iter()
                .find(|(_, term)| term.site.node() == id)
            {
                expected.typing = TypingState::Successful;
                expected.links.type_entry = Some(term.type_entry);
            }
            if inference
                .formulas()
                .iter()
                .any(|(_, formula)| formula.site.node() == id)
            {
                expected.typing = TypingState::Successful;
            }
            assert_eq!(tables.2.nodes().node(id), Some(&expected));
        }
        for (_, term) in inference.terms().iter() {
            let actual = tables.2.nodes().node(term.site.node()).unwrap();
            assert_eq!(actual.typing, TypingState::Successful);
            assert_eq!(actual.links.type_entry, Some(term.type_entry));
            assert_eq!(
                tables.2.types().get(term.type_entry).unwrap().owner,
                term.site
            );
        }
        assert_eq!(
            tables.2.nodes().node(formula.site().node()).unwrap().typing,
            TypingState::Successful
        );
        let metadata =
            TermFormulaChecker::check_source_theorem_equalities_with_expression_metadata(
                &source, &symbols, &scope,
            )?;
        assert_eq!(
            (&metadata.0, &metadata.1, &metadata.2),
            (&tables.0, &tables.1, &tables.2)
        );
        let expected = primary
            .terms()
            .iter()
            .map(|(_, row)| ("term", row.source_ordinal(), row.site()))
            .chain(
                atomic
                    .formulas()
                    .iter()
                    .map(|(_, row)| ("formula", row.source_ordinal(), row.site())),
            )
            .map(|(kind, ordinal, site)| ExpressionMetadataInput {
                expr: ExprId::new(format!("source.theorem.{kind}.{ordinal}")),
                typed_site: site.clone(),
                local_context: None,
                cluster_facts: Vec::new(),
            })
            .collect::<Vec<_>>();
        assert_eq!(metadata.3, expected);
        assert_eq!(metadata.3.len(), 3);
        assert_eq!(
            metadata,
            TermFormulaChecker::check_source_theorem_equalities_with_expression_metadata(
                &source, &symbols, &scope
            )?
        );
        assert_eq!(
            (bindings, inference, typed),
            TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope)?
        );
        assert_eq!(
            (source, symbols, scope, ast.snapshot_text(), frontend.source),
            before
        );
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_equality_producer_preserves_reserved_boundaries() -> Result<(), String> {
    use mizar_checker::{
        source_context::SourceBindingContextProducer,
        source_type::SourceTypeProducer,
        type_checker::{
            DeclarationChecker, SourceVariableSemanticsChecker, TermFormulaChecker,
            check_source_theorem_metadata,
        },
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_proof::{
        policy::VerifierPolicy as Policy,
        status::source_theorem_reflexivity_witness_draft_from_original as acquire,
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::{NamespacePath, SymbolEnv, SymbolEnvIndexes},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::{
            SignatureProjectionExtractor, source_proof_inputs,
            source_theorem_module_original_inputs, validate_source_proof_occurrences,
        },
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SnapshotRegistry, SourceInput, SourceOriginInput, ToolchainInfo,
        WorkspaceRoot, hash_text,
    };
    use mizar_syntax::{SurfaceAstBuilder, SurfaceNodeKind as K, SurfaceTokenKind};
    let text = include_str!(
        "../../../../../../tests/miz/pass/types/pass_type_elaboration_reserved_variable_equality_001.miz"
    );
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    let path = root.join("src").join("reserved.miz");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, text).unwrap();
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let policies = [
        Policy::default(),
        Policy::default().with_kernel_evidence_formats([]),
    ];
    let input = SourceInput {
        package_id: PackageId::new("reserved-boundary-test"),
        module_path: ModulePath::new("reserved"),
        normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
        edition: Edition::new("2026"),
        origin: SourceOriginInput::Disk { path: path.clone() },
    };
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
                ToolchainInfo::new("caller-supplied-reserved-boundary-test-toolchain"),
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
    let module = ModuleId::new(input.package_id.clone(), input.module_path.clone());
    let (source, symbols) = source_theorem_module_original_inputs(ast, &module)?.unwrap();
    let (scope, labels, resolved) = source_proof_inputs(&source, &symbols)?;
    let before = (
        source.clone(),
        symbols.clone(),
        scope.clone(),
        snapshot.clone(),
        policies.clone(),
        ast.snapshot_text(),
    );
    // Upper refusals are meaningful only after this real native lower success.
    let (_, _, typed) =
        TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope)?;
    let unsupported = "theorems.equality.unsupported_source_types";
    assert_eq!(
        SourceVariableSemanticsChecker::source_theorem_binding_env(&source, &symbols, &scope)
            .unwrap_err(),
        unsupported
    );
    assert_eq!(
        TermFormulaChecker::check_source_theorem_equalities_with_local_contexts(
            &source, &symbols, &scope
        )
        .unwrap_err(),
        unsupported
    );
    assert_eq!(
        TermFormulaChecker::check_source_theorem_equalities_with_scoped_inference(
            &source, &symbols, &scope
        )
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
    assert_eq!(
        DeclarationChecker::check_source_theorem_equalities_with_typed_declarations(
            &source, &symbols, &scope
        )
        .unwrap_err(),
        unsupported
    );
    assert_eq!(
        check_source_theorem_metadata(&source, &symbols, &scope).unwrap_err(),
        unsupported
    );
    assert!(
        SourceVariableSemanticsChecker::check_theorem_skeletons(
            &source, &typed, &scope, &symbols, &labels, &resolved, None
        )
        .is_err()
    );
    assert_eq!(
        acquire(&source, &symbols, &snapshot, &policies[1], "").unwrap_err(),
        unsupported
    );
    let empty = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
    let foreign_ast = parse(&text.replace("ReservedVariableEqualityPayloadBoundary", "Foreign"));
    let foreign_module = ModuleId::new(PackageId::new("foreign"), ModulePath::new("elsewhere"));
    let foreign_source = SurfaceResolvedArena::lower(&foreign_ast, &foreign_module).unwrap();
    let shells = DeclarationShellCollector::new(&foreign_ast, &foreign_module).collect();
    let foreign_env = SignatureProjectionExtractor::new(
        &foreign_ast,
        &shells,
        NamespacePath::new(foreign_module.path().as_str()),
    )
    .collect(&foreign_module);
    let foreign_scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&foreign_ast, &foreign_module, foreign_env.env()),
    )
    .unwrap();
    for (current, env, occurrences) in [
        (&source, &empty, &scope),
        (&source, foreign_env.env(), &scope),
        (&foreign_source, &symbols, &foreign_scope),
        (&source, &symbols, &foreign_scope),
    ] {
        let expected = validate_source_proof_occurrences(current, env, occurrences).unwrap_err();
        assert_eq!(
            TermFormulaChecker::check_source_theorem_equalities(current, env, occurrences)
                .unwrap_err(),
            expected
        );
        assert_eq!(
            SourceVariableSemanticsChecker::source_theorem_binding_env(current, env, occurrences)
                .unwrap_err(),
            expected
        );
        let expected_proof = source_proof_inputs(current, env);
        if let Err(expected) = expected_proof {
            assert_eq!(
                acquire(current, env, &snapshot, &policies[1], "").unwrap_err(),
                expected
            );
        }
    }
    for changed in [
        text.replace("reserve x for set;", ""),
        "theorem Later: x = x; reserve x for set;".into(),
        text.replace("reserve x", "reserve x, y"),
        format!("reserve y for set;\n{text}"),
        format!("{text}\ntheorem Extra: x = x;"),
        text.replace("for set", "for object"),
        text.replace("for set", "for non empty set"),
        text.replace("x = x", "x = y"),
        "reserve x for set; reserve y for set; theorem Different: x = y;".into(),
        text.replace("theorem", "public theorem"),
        text.replace("theorem", "private theorem"),
        text.replace("theorem", "open theorem"),
        text.replace("theorem", "assumed theorem"),
        text.replace("theorem", "lemma"),
        text.replace("x = x;", "x = x by Missing;"),
        text.replace("x = x;", "x = x proof thus x = x; end;"),
        text.replace("x = x", "x in x"),
        text.replace("x = x", "x = 0"),
        text.replace("x = x", "x = f(x)"),
        text.replace("x = x", "x = x & x = x"),
        text.replace("x = x", "x ="),
    ] {
        let altered = parse(&changed);
        let altered_source = SurfaceResolvedArena::lower(&altered, &module).unwrap();
        let shells = DeclarationShellCollector::new(&altered, &module).collect();
        let env = SignatureProjectionExtractor::new(
            &altered,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        let occurrences = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(&altered, &module, env.env()),
        );
        let input_scope = occurrences.as_ref().unwrap_or(&scope);
        let error = TermFormulaChecker::check_source_theorem_equalities(
            &altered_source,
            env.env(),
            input_scope,
        )
        .unwrap_err();
        if let Err(expected) =
            validate_source_proof_occurrences(&altered_source, env.env(), input_scope)
        {
            assert_eq!(error, expected, "{changed}");
        }
        // One unsupported type plus an empty environment fixes lower-error priority.
        if changed.contains("for object") {
            let expected = validate_source_proof_occurrences(&altered_source, &empty, input_scope)
                .unwrap_err();
            assert_eq!(
                TermFormulaChecker::check_source_theorem_equalities(
                    &altered_source,
                    &empty,
                    input_scope
                )
                .unwrap_err(),
                expected
            );
            let expected = source_proof_inputs(&altered_source, &empty).unwrap_err();
            assert_eq!(
                acquire(&altered_source, &empty, &snapshot, &policies[1], "").unwrap_err(),
                expected
            );
        }
    }
    for control in [
        "recovered operator",
        "wrong operator kind",
        "malformed wrapper",
        "root-only token",
    ] {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = Vec::new();
        for view in ast.node_views() {
            let mut children = view
                .children()
                .iter()
                .map(|id| rebuilt[id.index()])
                .collect::<Vec<_>>();
            if control == "malformed wrapper" && matches!(view.kind(), K::TypeExpression) {
                children.clear();
            }
            if control == "root-only token" && matches!(view.kind(), K::Root) {
                let semicolon = ast.node_views().find(|view| matches!(view.kind(), K::Token(token) if token.kind == SurfaceTokenKind::ReservedSymbol && token.text.as_ref() == ";")).unwrap();
                children.push(builder.add_token(
                    SurfaceTokenKind::ReservedSymbol,
                    ";",
                    semicolon.range(),
                ));
            }
            rebuilt.push(match view.kind() {
                K::Token(token)
                    if token.text.as_ref() == "=" && control == "recovered operator" =>
                {
                    builder.add_recovered_token(token.kind, token.text.clone(), view.range())
                }
                K::Token(token)
                    if token.text.as_ref() == "=" && control == "wrong operator kind" =>
                {
                    builder.add_token(
                        SurfaceTokenKind::ReservedWord,
                        token.text.clone(),
                        view.range(),
                    )
                }
                K::Token(token) => builder.add_token(token.kind, token.text.clone(), view.range()),
                kind => builder.add_node(kind.clone(), view.range(), children),
            });
        }
        let changed = builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None);
        let changed_source = SurfaceResolvedArena::lower(&changed, &module).unwrap();
        let shells = DeclarationShellCollector::new(&changed, &module).collect();
        let env = SignatureProjectionExtractor::new(
            &changed,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        let occurrences = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(&changed, &module, env.env()),
        );
        let input_scope = occurrences.as_ref().unwrap_or(&scope);
        let error = TermFormulaChecker::check_source_theorem_equalities(
            &changed_source,
            env.env(),
            input_scope,
        )
        .unwrap_err();
        if let Err(expected) =
            validate_source_proof_occurrences(&changed_source, env.env(), input_scope)
        {
            assert_eq!(error, expected);
        } else {
            assert_eq!(error, unsupported);
        }
    }
    assert_eq!(
        (
            source,
            symbols,
            scope,
            snapshot.clone(),
            policies,
            ast.snapshot_text()
        ),
        before
    );
    assert!(!root.join("proof-witnesses").exists());
    assert_eq!(registry.get(snapshot.id), Some(snapshot.clone()));
    registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_equality_producer_rejects_foreign_and_unsupported_sources() {
    use mizar_checker::type_checker::TermFormulaChecker;
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::{NamespacePath, SymbolEnv, SymbolEnvIndexes},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::SignatureProjectionExtractor,
    };
    use mizar_session::{ModulePath, PackageId};
    use mizar_syntax::{SurfaceAstBuilder, SurfaceNodeKind as K, SurfaceTokenKind};
    let text = "theorem T: for X being set, Unused being set holds X = X proof let X be set; thus X = X; end;";
    let ast = parse(text);
    let module = ModuleId::new(PackageId::new("equality-test"), ModulePath::new("owners"));
    let source = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let collected = SignatureProjectionExtractor::new(
        &ast,
        &shells,
        NamespacePath::new(module.path().as_str()),
    )
    .collect(&module);
    let symbols = collected.env();
    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&ast, &module, symbols),
    )
    .unwrap();
    assert!(TermFormulaChecker::check_source_theorem_equalities(&source, symbols, &scope).is_ok());
    let empty = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
    assert_eq!(
        TermFormulaChecker::check_source_theorem_equalities(&source, &empty, &scope).unwrap_err(),
        mizar_resolve::symbols::validate_source_proof_occurrences(&source, &empty, &scope)
            .unwrap_err()
    );
    let thesis = ast
        .node_views()
        .find(|view| matches!(view.kind(), K::QuantifiedFormula(_)))
        .unwrap();
    let equality = ast.node(*thesis.children().last().unwrap()).unwrap();
    let term = ast.node(equality.children[0]).unwrap();
    let reference = ast.node(term.children[0]).unwrap();
    assert!(matches!(reference.kind, K::TermReference));
    let reference_token = reference.children[0];
    for control in [
        "unused declaration",
        "reference only",
        "wrong operator token",
        "recovered term",
    ] {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = Vec::new();
        for view in ast.node_views() {
            let children = view
                .children()
                .iter()
                .map(|id| rebuilt[id.index()])
                .collect();
            rebuilt.push(match view.kind() {
                K::Token(token) if control == "reference only" && view.id() == reference_token => {
                    builder.add_token(token.kind, "Unused", view.range())
                }
                K::Token(token)
                    if control == "unused declaration" && token.text.as_ref() == "Unused" =>
                {
                    builder.add_token(token.kind, "Hidden", view.range())
                }
                K::Token(token)
                    if control == "wrong operator token" && token.text.as_ref() == "=" =>
                {
                    builder.add_token(SurfaceTokenKind::ReservedWord, "=", view.range())
                }
                K::Token(token) if control == "recovered term" && token.text.as_ref() == "=" => {
                    builder.add_recovered_token(token.kind, token.text.clone(), view.range())
                }
                K::Token(token) => builder.add_token(token.kind, token.text.clone(), view.range()),
                kind => builder.add_node(kind.clone(), view.range(), children),
            });
        }
        let changed = builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None);
        if matches!(control, "unused declaration" | "reference only") {
            let foreign = SourceVariableScopeResolver::resolve_proof_occurrences(
                SourceVariableScopeInput::new(&changed, &module, symbols),
            )
            .unwrap();
            assert_eq!(foreign.source_id(), scope.source_id());
            if control == "unused declaration" {
                assert_eq!(foreign.references(), scope.references());
                assert_ne!(foreign.bindings(), scope.bindings());
            } else {
                assert_eq!(foreign.bindings(), scope.bindings());
                assert_ne!(foreign.references(), scope.references());
            }
            assert_eq!(
                TermFormulaChecker::check_source_theorem_equalities(&source, symbols, &foreign)
                    .unwrap_err(),
                mizar_resolve::symbols::validate_source_proof_occurrences(
                    &source, symbols, &foreign
                )
                .unwrap_err()
            );
        }
        let changed_source = SurfaceResolvedArena::lower(&changed, &module).unwrap();
        let shells = DeclarationShellCollector::new(&changed, &module).collect();
        let env = SignatureProjectionExtractor::new(
            &changed,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        if control != "unused declaration" {
            assert!(
                TermFormulaChecker::check_source_theorem_equalities(
                    &changed_source,
                    env.env(),
                    &scope
                )
                .is_err(),
                "{control}"
            );
        }
    }
    let unused_object = text.replace("Unused being set", "Unused being object");
    let all_object = text.replace("set", "object");
    let private_object = text
        .replace("theorem T", "private theorem T")
        .replace("set", "object");
    let unresolved = text.replace("holds X = X", "holds X = Y");
    let conjunction = text.replace("holds X = X", "holds X = X & X = X");
    let unused_reserve = format!("reserve Z for set; {text}");
    for changed in [
        unused_object.clone(),
        all_object.clone(),
        text.replace("set", "non empty set"),
        text.replace("thus X = X", "thus X in X"),
        text.replace("thus X = X", "thus X = 0"),
        text.replace("thus X = X", "thus X = f(X)"),
        conjunction.clone(),
        text.replace("for X being set, Unused being set", "ex X being set"),
        private_object.clone(),
        text.replace("theorem T", "public open theorem T"),
        "open theorem T: for X being set holds X = X;".into(),
        "assumed theorem T: for X being set holds X = X;".into(),
        "theorem T: 0 = 0 by computation(steps: 8);".into(),
        unused_reserve.clone(),
        format!("import mml.test; {text}"),
        format!("definition let Z be set; end; {text}"),
        text.replace("thus X = X", "thus X ="),
        unresolved.clone(),
        format!("{text} theorem Extra: 0 = 0 by computation(steps: 8);"),
    ] {
        let altered = parse(&changed);
        let altered_source = SurfaceResolvedArena::lower(&altered, &module).unwrap();
        let shells = DeclarationShellCollector::new(&altered, &module).collect();
        let env = SignatureProjectionExtractor::new(
            &altered,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        let occurrences = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(&altered, &module, env.env()),
        );
        let occurrence_input = if changed == conjunction {
            occurrences
                .as_ref()
                .expect("fixed conjunction must resolve")
        } else if changed == unused_reserve {
            occurrences
                .as_ref()
                .expect("unused Reserve must resolve freshly")
        } else if changed == unused_object {
            occurrences
                .as_ref()
                .expect("unused object declaration must resolve")
        } else {
            occurrences.as_ref().unwrap_or(&scope)
        };
        if env.env() != symbols {
            assert!(
                TermFormulaChecker::check_source_theorem_equalities(&source, env.env(), &scope)
                    .is_err(),
                "foreign env: {changed}"
            );
        }
        let result = TermFormulaChecker::check_source_theorem_equalities(
            &altered_source,
            env.env(),
            occurrence_input,
        );
        if changed == unused_reserve {
            // EN04 §4.3: the unused default does not exclude an explicitly typed quantifier.
            assert!(
                TermFormulaChecker::check_source_theorem_equalities(
                    &altered_source,
                    env.env(),
                    &scope
                )
                .is_err()
            );
            let (_, inference, typed) = result.unwrap();
            assert_eq!(
                (inference.terms().len(), inference.formulas().len()),
                (4, 2)
            );
            assert_eq!(typed.source_term().unwrap().terms().len(), 4);
            assert_eq!(typed.source_atomic_formula().unwrap().formulas().len(), 2);
            assert!(typed.source_type().is_none());
            assert!(inference.facts().is_empty() && typed.facts().is_empty());
            continue;
        }
        if [&unused_object, &all_object, &private_object].contains(&&changed) {
            let (_, inference, typed) = result.unwrap();
            assert_eq!(
                (inference.terms().len(), inference.formulas().len()),
                (4, 2)
            );
            assert_eq!(typed.source_term().unwrap().terms().len(), 4);
            assert!(inference.facts().is_empty() && typed.facts().is_empty());
            for (_, term) in inference.terms().iter() {
                let entry = inference.type_entries().get(term.type_entry).unwrap();
                let mizar_checker::typed_ast::TypeEntryActual::Known(id) = entry.actual else {
                    panic!("known builtin type")
                };
                assert_eq!(
                    inference.normalized_types().get(id).unwrap().head,
                    if changed == unused_object {
                        mizar_checker::type_checker::TypeHeadRef::BuiltinSet
                    } else {
                        mizar_checker::type_checker::TypeHeadRef::BuiltinObject
                    }
                );
            }
            assert_eq!(
                TermFormulaChecker::check_source_theorem_equalities(
                    &altered_source,
                    &empty,
                    occurrence_input
                )
                .unwrap_err(),
                mizar_resolve::symbols::validate_source_proof_occurrences(
                    &altered_source,
                    &empty,
                    occurrence_input
                )
                .unwrap_err()
            );
            continue;
        }
        if changed == conjunction {
            let (_, inference, typed) = result.unwrap();
            assert_eq!(typed.source_term().unwrap().terms().len(), 6);
            assert_eq!(inference.terms().len(), 6);
            assert_eq!(typed.source_atomic_formula().unwrap().formulas().len(), 3);
            assert_eq!(inference.formulas().len(), 3);
            assert!(inference.facts().is_empty() && typed.facts().is_empty());
            assert!(
                typed.source_composite_formula().is_none()
                    && typed.source_formula_composition().is_none()
                    && typed.source_statement().is_none()
            );
            continue;
        }
        let error = result.unwrap_err();
        if changed == unresolved {
            assert!(occurrences.is_err());
            assert_eq!(
                error,
                mizar_resolve::symbols::validate_source_proof_occurrences(
                    &altered_source,
                    env.env(),
                    occurrence_input,
                )
                .unwrap_err()
            );
        }
    }
    let foreign_ast = parse(&text.replace("T:", "Foreign:"));
    let foreign_module = ModuleId::new(PackageId::new("other"), ModulePath::new("foreign"));
    let foreign_source = SurfaceResolvedArena::lower(&foreign_ast, &foreign_module).unwrap();
    assert!(
        TermFormulaChecker::check_source_theorem_equalities(&foreign_source, symbols, &scope)
            .is_err()
    );
    assert!(
        TermFormulaChecker::check_source_theorem_equalities(
            &source,
            symbols,
            &SourceVariableScopeResolver::resolve_occurrences(SourceVariableScopeInput::new(
                &ast, &module, symbols
            ))
            .unwrap()
        )
        .is_ok()
    );

    let set = ast
        .node_views()
        .find(|view| {
            matches!(view.kind(), K::Token(token) if token.kind == SurfaceTokenKind::ReservedWord && token.text.as_ref() == "set")
        })
        .unwrap();
    let mut builder = SurfaceAstBuilder::new(ast.source_id);
    let token = builder.add_token(SurfaceTokenKind::ReservedWord, "set", set.range());
    let malformed = builder.finish(Some(token), None);
    let shells = DeclarationShellCollector::new(&malformed, &module).collect();
    let env = SignatureProjectionExtractor::new(
        &malformed,
        &shells,
        NamespacePath::new(module.path().as_str()),
    )
    .collect(&module);
    let malformed_scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&malformed, &module, env.env()),
    )
    .unwrap();
    let malformed_source = SurfaceResolvedArena::lower(&malformed, &module).unwrap();
    assert_eq!(
        mizar_resolve::symbols::validate_source_proof_occurrences(
            &malformed_source,
            env.env(),
            &malformed_scope,
        ),
        Ok(())
    );
    assert_eq!(
        TermFormulaChecker::check_source_theorem_equalities(
            &malformed_source,
            env.env(),
            &malformed_scope,
        )
        .unwrap_err(),
        "theorems.equality.unsupported_source_types"
    );
}
