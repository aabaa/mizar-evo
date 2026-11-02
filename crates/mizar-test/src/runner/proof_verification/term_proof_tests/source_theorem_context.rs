use super::*;

#[test]
fn source_theorem_context_handoff_retains_real_tables() -> Result<(), String> {
    use mizar_checker::{
        binding_env::{BinderIdentity, BindingContextOwner, BindingKind},
        source_context::{
            SourceBindingContextInput, SourceBindingContextProducer as Producer,
            SourceBindingSiteRole, SourceContextError, SourceItemInput, SourceItemRecovery,
            SourceItemRole, SourceItemVisibility,
        },
        type_checker::TermFormulaChecker as Checker,
        typed_ast::{
            LocalTypeContextDraft, LocalTypeContextTable, TypedArena, TypedAst, TypedAstError,
            TypedAstParts, TypedNodeKind, TypedSiteRef,
        },
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        declarations::{
            DeclarationShellCollector, DeclarationShellKind, DeclarationShellVisibilityState,
        },
        env::{NamespacePath, Visibility},
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
    let cases = [
        fixture.to_owned(),
        fixture.replace("Lem1", "Seed"),
        "theorem Pair: for A, Spare, B being set holds A = B proof let L, Unused, R be set; thus L = R; end;".into(),
        "theorem Shadow: for A being set holds for A being set holds A = A proof let A be set; thus A = A; end;".into(),
        "theorem Empty: for A being set holds A = A proof end;".into(),
    ];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/theorem_context.miz");
    let ids = InMemorySessionIdAllocator::new();
    for (case, text) in cases.into_iter().enumerate() {
        let mut graph = None;
        for buffer in [false, true] {
            fs::write(&path, if buffer { "stale disk content" } else { &text }).unwrap();
            let frontend = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
                super::super::super::ParseOnlyImportProvider,
                MizarParserSeam,
            )
            .run(
                SourceUnitRequest {
                    snapshot: snapshot_id(778),
                    input: SourceInput {
                        package_id: PackageId::new("theorem-context-test"),
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
            assert_eq!(frontend.source.source_hash, hash_text(&text));
            assert_eq!(
                matches!(
                    frontend.source.origin,
                    SourceOrigin::OpenBuffer { version: 3 }
                ),
                buffer
            );
            let capture_before = frontend.source.clone();
            let ast = frontend.ast.as_ref().unwrap();
            let ast_before = ast.snapshot_text();
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
            let before = (source.clone(), symbols.clone(), scope.clone());
            let baseline = Checker::check_source_theorem_equalities_with_scoped_inference(
                &source, symbols, &scope,
            )?;
            let typed = &baseline.2;
            let env = &baseline.0;
            let result = Producer::build_source_theorem_equalities(&source, symbols, &scope)?;
            assert_eq!(
                result,
                Producer::build_source_theorem_equalities(&source, symbols, &scope)?
            );
            assert_eq!(result.0, baseline.0);
            assert_eq!(result.1, baseline.1);
            assert_eq!(result.3, baseline.3);
            let handoff = result.2.source_context().unwrap();
            assert_eq!(handoff.source_id(), source.source_id());
            assert_eq!(handoff.module_id(), source.module());
            assert_eq!(handoff.binding_env(), &result.0);
            assert_eq!(handoff.local_contexts(), result.2.contexts());
            assert_eq!(handoff.items().len(), shells.declarations().len());
            assert_eq!(handoff.declarations().len(), scope.bindings().len());
            assert_eq!(handoff.context_links().len(), result.0.contexts().len());
            assert_eq!(result.2.nodes(), typed.nodes());
            assert_eq!(result.2.contexts(), typed.contexts());
            assert_eq!(result.2.types(), typed.types());
            assert_eq!(result.2.facts(), typed.facts());
            assert_eq!(result.2.diagnostics(), typed.diagnostics());
            assert_eq!(result.2.source_term(), typed.source_term());
            assert_eq!(
                result.2.source_atomic_formula(),
                typed.source_atomic_formula()
            );
            assert_eq!(result.2.source_id(), typed.source_id());
            assert_eq!(result.2.module_id(), typed.module_id());
            assert_eq!(result.2.resolved_root(), typed.resolved_root());
            assert_eq!(result.2.coercions(), typed.coercions());
            assert_eq!(result.2.initial_obligations(), typed.initial_obligations());
            assert_eq!(result.2.source_type(), typed.source_type());
            assert_eq!(result.2.source_attribute(), typed.source_attribute());
            for (id, item) in handoff.items().iter() {
                let shell = &shells.declarations()[id.index()];
                assert_eq!(shell.module(), &module);
                assert!(!shell.recovered());
                assert!(shell.parent().is_none());
                assert_eq!(
                    shell.visibility().state(),
                    DeclarationShellVisibilityState::Unspecified
                );
                assert!(shell.visibility().marker_range().is_none());
                assert!(shell.visibility().spelling().is_none());
                let anchor = mizar_session::SourceAnchor::Range(shell.range());
                let mut symbols_for_shell = symbols
                    .symbols()
                    .iter()
                    .filter(|entry| entry.origin().anchor() == &anchor);
                assert_eq!(
                    symbols_for_shell.next().unwrap().visibility(),
                    Visibility::Public
                );
                assert!(symbols_for_shell.next().is_none());
                assert_eq!(item.visibility, SourceItemVisibility::Unspecified);
                assert_eq!(item.recovery, SourceItemRecovery::Normal);
                assert!(item.local_scope.is_none());
                assert_eq!(item.shell, shell.id());
                assert_eq!(item.shell_ordinal, shell.ordinal());
                assert_eq!(item.source_range, shell.range());
                assert_eq!(item.parent, shell.parent());
                assert_eq!(
                    item.predecessor,
                    id.index()
                        .checked_sub(1)
                        .map(mizar_checker::source_context::SourceItemId::new)
                );
                assert_eq!(
                    item.role,
                    match shell.kind() {
                        DeclarationShellKind::Theorem => SourceItemRole::Theorem,
                        DeclarationShellKind::Lemma => SourceItemRole::Lemma,
                        _ => panic!("unsupported captured shell"),
                    }
                );
                let resolved = source.resolved_node_for(shell.node_id()).unwrap();
                let sites = typed
                    .nodes()
                    .iter()
                    .filter(|(_, node)| node.resolved_node == Some(resolved))
                    .collect::<Vec<_>>();
                assert_eq!(sites.len(), 1);
                let (site, node) = sites[0];
                assert_eq!(item.site, TypedSiteRef::Node(site));
                assert_eq!(node.anchor, anchor);
                assert_eq!(node.links.context, Some(item.local_context));
                let links = handoff
                    .context_links()
                    .iter()
                    .filter(|(_, link)| link.local_context == item.local_context)
                    .collect::<Vec<_>>();
                assert_eq!(links.len(), 1);
                assert_eq!(item.binding_context, links[0].1.binding_context);
                let context = env.contexts().get(item.binding_context).unwrap();
                assert_eq!(context.owner, BindingContextOwner::Module);
            }
            for (id, link) in handoff.context_links().iter() {
                assert_eq!(link.binding_context.index(), id);
                let binding = result.0.contexts().get(link.binding_context).unwrap();
                let local = result.2.contexts().get(link.local_context).unwrap();
                assert_eq!(
                    local.parent,
                    binding.parent.map(|parent| handoff
                        .context_links()
                        .get(parent)
                        .unwrap()
                        .local_context)
                );
                if id == 0 {
                    assert!(link.item.is_none());
                } else {
                    let item = handoff.items().get(link.item.unwrap()).unwrap();
                    let range = match binding.owner {
                        BindingContextOwner::SourceFormula { source_range }
                        | BindingContextOwner::SourceStatement { source_range } => source_range,
                        _ => panic!("unexpected theorem context"),
                    };
                    assert!(
                        item.source_range.start <= range.start
                            && range.end <= item.source_range.end
                    );
                    let node = typed.nodes().node(local.owner.node()).unwrap();
                    assert_eq!(node.anchor, mizar_session::SourceAnchor::Range(range));
                    assert_eq!(node.links.context, Some(link.local_context));
                }
            }
            let originals = scope
                .bindings()
                .iter()
                .map(|original| {
                    let matches = env
                        .bindings()
                        .iter()
                        .filter(|(_, entry)| {
                            entry.identity
                                == BinderIdentity::ResolverLocal {
                                    scope: original.scope().clone(),
                                    ordinal: original.ordinal(),
                                    declaration_range: original.range(),
                                }
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(matches.len(), 1);
                    (original, matches[0].0, matches[0].1)
                })
                .collect::<Vec<_>>();
            for (id, declaration) in handoff.declarations().iter() {
                let joined = originals
                    .iter()
                    .filter(|(_, binding, _)| *binding == declaration.binding)
                    .collect::<Vec<_>>();
                assert_eq!(joined.len(), 1);
                let (original, binding, entry) = *joined[0];
                assert_eq!(result.0.bindings().get(binding), Some(entry));
                assert_eq!(scope.bindings()[id.index()].id(), original.id());
                assert_eq!(entry.spelling, original.spelling());
                assert_eq!(
                    entry.kind,
                    match original.kind() {
                        SourceVariableBindingKind::Quantifier => BindingKind::QuantifierBinder,
                        SourceVariableBindingKind::Let => BindingKind::LetBinding,
                        _ => panic!("unsupported captured binding"),
                    }
                );
                let owners = shells
                    .declarations()
                    .iter()
                    .filter(|shell| {
                        shell.range().start <= original.range().start
                            && original.range().end <= shell.range().end
                    })
                    .collect::<Vec<_>>();
                assert_eq!(owners.len(), 1);
                let items = handoff
                    .items()
                    .iter()
                    .filter(|(_, item)| item.shell == owners[0].id())
                    .collect::<Vec<_>>();
                assert_eq!(items.len(), 1);
                assert_eq!(declaration.item, items[0].0);
                assert_eq!(declaration.source_ordinal, original.ordinal());
                assert_eq!(declaration.spelling, original.spelling());
                assert_eq!(declaration.declaration_range, original.range());
                assert_eq!(entry.owner_context, declaration.binding_context);
                assert_eq!(
                    declaration.role,
                    match original.kind() {
                        SourceVariableBindingKind::Quantifier =>
                            SourceBindingSiteRole::QuantifierBinder,
                        SourceVariableBindingKind::Let => SourceBindingSiteRole::ProofLet,
                        _ => panic!("unsupported captured binding"),
                    }
                );
                assert_eq!(
                    declaration.predecessor,
                    id.index()
                        .checked_sub(1)
                        .map(mizar_checker::source_context::SourceDeclarationId::new)
                );
                let segment = ast
                    .node_views()
                    .find(|view| {
                        view.children().contains(&original.node())
                            && matches!(
                                view.kind(),
                                K::QuantifierVariableSegment | K::QualifiedVariableSegment
                            )
                    })
                    .unwrap();
                let type_child = ast.node_view(*segment.children().last().unwrap()).unwrap();
                assert_eq!(type_child.kind(), &K::TypeExpression);
                assert_eq!(declaration.written_type_range, type_child.range());
                let resolved = source.resolved_node_for(original.node()).unwrap();
                let sites = typed
                    .nodes()
                    .iter()
                    .filter(|(_, row)| row.resolved_node == Some(resolved))
                    .collect::<Vec<_>>();
                assert_eq!(sites.len(), 1);
                assert_eq!(declaration.site, TypedSiteRef::Node(sites[0].0));
                assert_eq!(sites[0].1.links.context, Some(declaration.local_context));
                let link = handoff.context_links().get(entry.owner_context).unwrap();
                assert_eq!(link.item, Some(declaration.item));
                assert_eq!(link.local_context, declaration.local_context);
                // Spec 4.1/4.6: lexical ancestry comes from the original resolver,
                // independently of the handoff's binding-context parent graph.
                let expected = originals
                    .iter()
                    .filter(|(prior, _, _)| {
                        prior.spelling() == original.spelling()
                            && prior.ordinal() < original.ordinal()
                            && prior.scope().path().len() < original.scope().path().len()
                            && original.scope().path().starts_with(prior.scope().path())
                    })
                    .max_by_key(|(prior, _, _)| (prior.scope().path().len(), prior.ordinal()));
                assert_eq!(
                    declaration.shadowed_binding,
                    expected.map(|(_, binding, _)| *binding)
                );
                if case != 3 || original.kind() == SourceVariableBindingKind::Let {
                    assert!(expected.is_none());
                }
            }
            if case == 3 {
                let quantifiers = originals
                    .iter()
                    .filter(|(row, _, _)| row.kind() == SourceVariableBindingKind::Quantifier)
                    .collect::<Vec<_>>();
                assert_eq!(quantifiers.len(), 2);
                let (outer, outer_id, _) = *quantifiers[0];
                let (inner, inner_id, _) = *quantifiers[1];
                assert_eq!(outer.spelling(), inner.spelling());
                assert!(inner.scope().path().starts_with(outer.scope().path()));
                assert!(outer.scope().path().len() < inner.scope().path().len());
                let shadows = handoff
                    .declarations()
                    .iter()
                    .filter(|(_, row)| row.shadowed_binding.is_some())
                    .collect::<Vec<_>>();
                assert_eq!(shadows.len(), 1);
                assert_eq!(shadows[0].1.binding, inner_id);
                assert_eq!(shadows[0].1.shadowed_binding, Some(outer_id));
            }
            let shell = &shells.declarations()[0];
            let item = handoff.items().iter().next().unwrap().1;
            assert_eq!(
                Producer::build(SourceBindingContextInput {
                    source_id: source.source_id(),
                    module_id: module.clone(),
                    module_site: TypedSiteRef::Node(result.2.nodes().root().unwrap()),
                    items: vec![SourceItemInput {
                        shell: shell.id(),
                        shell_ordinal: shell.ordinal(),
                        role: item.role,
                        module_id: module.clone(),
                        source_range: shell.range(),
                        parent: shell.parent(),
                        visibility: SourceItemVisibility::Unspecified,
                        site: item.site.clone(),
                        local_scope: None,
                        recovery: SourceItemRecovery::Normal,
                    }],
                    bindings: vec![],
                })
                .unwrap_err(),
                SourceContextError::UnsupportedTaskShape
            );
            let shape = handoff
                .debug_text()
                .replace(&format!("{:?}", source.source_id()), "source");
            if let Some(previous) = &graph {
                assert_eq!(&shape, previous);
            }
            graph = Some(shape);
            assert_eq!((source.clone(), symbols.clone(), scope.clone()), before);
            assert_eq!(ast.snapshot_text(), ast_before);
            assert_eq!(frontend.source, capture_before);
            let parts = TypedAstParts {
                source_id: typed.source_id(),
                module_id: typed.module_id().clone(),
                resolved_root: typed.resolved_root(),
                source_context: Some(handoff.clone()),
                source_type: typed.source_type().cloned(),
                source_attribute: typed.source_attribute().cloned(),
                nodes: typed.nodes().clone(),
                contexts: typed.contexts().clone(),
                types: typed.types().clone(),
                facts: typed.facts().clone(),
                coercions: typed.coercions().clone(),
                initial_obligations: typed.initial_obligations().clone(),
                diagnostics: typed.diagnostics().clone(),
            };
            let installed = TypedAst::try_new(parts.clone())
                .and_then(|ast| ast.with_source_term(typed.source_term().unwrap().clone()))
                .and_then(|ast| {
                    ast.with_source_atomic_formula(typed.source_atomic_formula().unwrap().clone())
                })
                .unwrap();
            assert_eq!(installed, result.2);
            let boundary = handoff
                .context_links()
                .iter()
                .find(|(_, link)| link.item.is_some())
                .unwrap()
                .1;
            let local = result.2.contexts().get(boundary.local_context).unwrap();
            for fault in ["kind", "quantifier kind", "range", "context"] {
                let mut nodes = parts
                    .nodes
                    .iter()
                    .map(|(_, row)| row.clone())
                    .collect::<Vec<_>>();
                let node = &mut nodes[local.owner.node().index()];
                match fault {
                    "kind" => node.kind = TypedNodeKind::new("ProofBlock"),
                    "quantifier kind" => {
                        node.kind = TypedNodeKind::new("QuantifiedFormula(Existential)")
                    }
                    "range" => {
                        node.anchor = mizar_session::SourceAnchor::Range(
                            handoff
                                .items()
                                .get(boundary.item.unwrap())
                                .unwrap()
                                .source_range,
                        )
                    }
                    _ => {
                        node.links.context = Some(
                            handoff
                                .items()
                                .get(boundary.item.unwrap())
                                .unwrap()
                                .local_context,
                        )
                    }
                }
                let mut corrupt = parts.clone();
                corrupt.nodes = TypedArena::try_new(parts.nodes.root(), nodes).unwrap();
                assert_eq!(
                    TypedAst::try_new(corrupt).unwrap_err(),
                    TypedAstError::InvalidSourceContext,
                    "{fault}"
                );
            }
            let different_owner = TypedSiteRef::Node(parts.nodes.root().unwrap());
            assert_ne!(local.owner, different_owner);
            let mut contexts = LocalTypeContextTable::new();
            for (id, row) in parts.contexts.iter() {
                let mut expected = row.clone();
                if id == boundary.local_context {
                    expected.owner = different_owner.clone();
                }
                assert_eq!(
                    contexts.insert(LocalTypeContextDraft {
                        owner: expected.owner.clone(),
                        parent: row.parent,
                        layer: row.layer,
                        bindings: row.bindings.clone(),
                        introduced_assumptions: row.introduced_assumptions.clone(),
                        visible_facts: row.visible_facts.clone(),
                        recovery: row.recovery,
                    }),
                    id
                );
                assert_eq!(contexts.get(id), Some(&expected));
            }
            let mut corrupt = parts.clone();
            corrupt.contexts = contexts;
            assert_eq!(
                TypedAst::try_new(corrupt).unwrap_err(),
                TypedAstError::InvalidSourceContext
            );
        }
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_context_handoff_preserves_upstream_errors() {
    use mizar_checker::{
        source_context::SourceBindingContextProducer as Producer,
        type_checker::TermFormulaChecker as Checker,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::{NamespacePath, SymbolEnv, SymbolEnvIndexes},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::SignatureProjectionExtractor,
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceInput,
        SourceOriginInput,
    };
    use mizar_syntax::{SurfaceAstBuilder, SurfaceNodeKind as K};
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/theorem_context_errors.miz");
    fs::write(&path, text).unwrap();
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
        super::super::super::ParseOnlyImportProvider,
        MizarParserSeam,
    )
    .run(
        SourceUnitRequest {
            snapshot: snapshot_id(779),
            input: SourceInput {
                package_id: PackageId::new("theorem-context-errors"),
                module_path: ModulePath::new("owners"),
                normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                edition: Edition::new("2026"),
                origin: SourceOriginInput::Disk { path: path.clone() },
            },
        },
        &InMemorySessionIdAllocator::new(),
    )
    .unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.as_ref().unwrap();
    let module = ModuleId::new(
        frontend.source.package_id.clone(),
        frontend.source.module_path.clone(),
    );
    let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
    let shells = DeclarationShellCollector::new(ast, &module).collect();
    let collected =
        SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new(module.path().as_str()))
            .collect(&module);
    let symbols = collected.env();
    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(ast, &module, symbols),
    )
    .unwrap();
    assert!(
        Checker::check_source_theorem_equalities_with_scoped_inference(&source, symbols, &scope)
            .is_ok()
    );
    assert!(Producer::build_source_theorem_equalities(&source, symbols, &scope).is_ok());
    let empty = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
    let foreign_module = ModuleId::new(PackageId::new("foreign"), ModulePath::new("other"));
    let mut controls = vec![
        (source.clone(), empty.clone(), scope.clone()),
        (
            SurfaceResolvedArena::lower(ast, &foreign_module).unwrap(),
            symbols.clone(),
            scope.clone(),
        ),
    ];
    for control in ["foreign scope", "recovered source"] {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = Vec::new();
        for view in ast.node_views() {
            let children = view
                .children()
                .iter()
                .map(|id| rebuilt[id.index()])
                .collect();
            rebuilt.push(match view.kind() {
                K::Token(token) if control == "foreign scope" && token.text.as_ref() == "X" => {
                    builder.add_token(token.kind, "Y", view.range())
                }
                K::Token(token) if control == "recovered source" && token.text.as_ref() == "=" => {
                    builder.add_recovered_token(token.kind, token.text.clone(), view.range())
                }
                K::Token(token) => builder.add_token(token.kind, token.text.clone(), view.range()),
                kind => builder.add_node(kind.clone(), view.range(), children),
            });
        }
        let changed = builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None);
        if control == "foreign scope" {
            let foreign = SourceVariableScopeResolver::resolve_proof_occurrences(
                SourceVariableScopeInput::new(&changed, &module, symbols),
            )
            .unwrap();
            controls.push((source.clone(), symbols.clone(), foreign));
        } else {
            controls.push((
                SurfaceResolvedArena::lower(&changed, &module).unwrap(),
                symbols.clone(),
                scope.clone(),
            ));
        }
    }
    let unsupported = parse(&text.replace("set", "object"));
    let unsupported_source = SurfaceResolvedArena::lower(&unsupported, &module).unwrap();
    let unsupported_shells = DeclarationShellCollector::new(&unsupported, &module).collect();
    let unsupported_collected = SignatureProjectionExtractor::new(
        &unsupported,
        &unsupported_shells,
        NamespacePath::new(module.path().as_str()),
    )
    .collect(&module);
    let unsupported_scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&unsupported, &module, unsupported_collected.env()),
    )
    .unwrap();
    controls.push((
        unsupported_source.clone(),
        unsupported_collected.env().clone(),
        unsupported_scope,
    ));
    controls.push((unsupported_source, empty, scope.clone()));
    for (source, symbols, scope) in controls {
        let before = (source.clone(), symbols.clone(), scope.clone());
        let expected = Checker::check_source_theorem_equalities_with_scoped_inference(
            &source, &symbols, &scope,
        )
        .unwrap_err();
        assert_eq!(
            Producer::build_source_theorem_equalities(&source, &symbols, &scope).unwrap_err(),
            expected
        );
        assert_eq!(
            Producer::build_source_theorem_equalities(&source, &symbols, &scope).unwrap_err(),
            expected
        );
        assert_eq!((source, symbols, scope), before);
    }
    fs::remove_dir_all(root).unwrap();
}
