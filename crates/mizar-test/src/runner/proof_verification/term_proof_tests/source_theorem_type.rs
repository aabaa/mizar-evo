use super::*;

#[test]
fn source_theorem_type_handoff_retains_real_written_types() -> Result<(), String> {
    use mizar_checker::{
        binding_env::BinderIdentity,
        source_context::SourceBindingContextProducer as ContextProducer,
        source_type::{
            SourceTypeApplicationForm, SourceTypeApplicationInput, SourceTypeExpressionId,
            SourceTypeExpressionInput, SourceTypeHandoffInput, SourceTypeHead,
            SourceTypeProducer as Producer,
        },
        typed_ast::{
            NodeRecoveryState, TypedArena, TypedAst, TypedAstError, TypedAstParts, TypedNodeKind,
            TypedSiteRef,
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
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::SignatureProjectionExtractor,
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceInput,
        SourceOrigin, SourceOriginInput, hash_text,
    };
    use mizar_syntax::SurfaceNodeKind as K;
    use std::collections::{BTreeMap, BTreeSet};
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
    let path = root.join("src/theorem_type.miz");
    let ids = InMemorySessionIdAllocator::new();
    for text in cases {
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
                    snapshot: snapshot_id(780),
                    input: SourceInput {
                        package_id: PackageId::new("theorem-type-test"),
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
            let baseline =
                ContextProducer::build_source_theorem_equalities(&source, symbols, &scope)?;
            // Derive the oracle before the new producer, from original syntax and scope identity.
            let typed = &baseline.2;
            let mut expected = SourceTypeHandoffInput {
                source_id: source.source_id(),
                module_id: module.clone(),
                applications: vec![],
                expressions: vec![],
                arguments: vec![],
            };
            let mut first_use = BTreeMap::new();
            let mut original_types = BTreeSet::new();
            let mut original_heads = BTreeSet::new();
            for original in scope.bindings() {
                let bindings = baseline
                    .0
                    .bindings()
                    .iter()
                    .filter(|(_, entry)| {
                        matches!(&entry.identity, BinderIdentity::ResolverLocal {
                        scope, ordinal, declaration_range
                    } if scope == original.scope() && *ordinal == original.ordinal()
                        && *declaration_range == original.range())
                    })
                    .collect::<Vec<_>>();
                assert_eq!(bindings.len(), 1);
                let segments = ast
                    .node_views()
                    .filter(|view| {
                        view.children().contains(&original.node())
                            && matches!(
                                view.kind(),
                                K::QuantifierVariableSegment | K::QualifiedVariableSegment
                            )
                    })
                    .collect::<Vec<_>>();
                assert_eq!(segments.len(), 1);
                let written = ast
                    .node_view(*segments[0].children().last().unwrap())
                    .unwrap();
                assert_eq!(written.kind(), &K::TypeExpression);
                assert_eq!(written.children().len(), 1);
                let head = ast.node_view(written.children()[0]).unwrap();
                assert_eq!(head.kind(), &K::TypeHead);
                assert_eq!(&text[written.range().start..written.range().end], "set");
                assert_eq!(&text[head.range().start..head.range().end], "set");
                let resolved = source.resolved_node_for(written.id()).unwrap();
                let head_resolved = source.resolved_node_for(head.id()).unwrap();
                let sites = typed
                    .nodes()
                    .iter()
                    .filter(|(_, row)| row.resolved_node == Some(resolved))
                    .collect::<Vec<_>>();
                let head_sites = typed
                    .nodes()
                    .iter()
                    .filter(|(_, row)| row.resolved_node == Some(head_resolved))
                    .collect::<Vec<_>>();
                assert_eq!(sites.len(), 1);
                assert_eq!(head_sites.len(), 1);
                let (site, node) = sites[0];
                let (head_site, head_node) = head_sites[0];
                assert_eq!(node.children, [head_site]);
                assert_eq!(node.links.context, head_node.links.context);
                assert_eq!(node.recovery, NodeRecoveryState::Normal);
                assert_eq!(head_node.recovery, NodeRecoveryState::Normal);
                let local = baseline
                    .2
                    .source_context()
                    .unwrap()
                    .context_links()
                    .get(bindings[0].1.owner_context)
                    .unwrap()
                    .local_context;
                assert_eq!(node.links.context, Some(local));
                let next_id = SourceTypeExpressionId::new(expected.expressions.len());
                let root = *first_use.entry(written.id()).or_insert(next_id);
                if root == next_id {
                    expected.expressions.push(SourceTypeExpressionInput {
                        source_id: source.source_id(),
                        module_id: module.clone(),
                        site: TypedSiteRef::Node(site),
                        source_range: written.range(),
                        spelling: text[written.range().start..written.range().end].to_owned(),
                        head_site: TypedSiteRef::Node(head_site),
                        head_range: head.range(),
                        head_spelling: text[head.range().start..head.range().end].to_owned(),
                        form: SourceTypeApplicationForm::Bare,
                        head: SourceTypeHead::BuiltinSet,
                        recovery: NodeRecoveryState::Normal,
                    });
                }
                original_types.insert(written.id());
                original_heads.insert(head.id());
                expected.applications.push(SourceTypeApplicationInput {
                    binding: bindings[0].0,
                    source_ordinal: original.ordinal(),
                    root,
                });
            }
            assert_eq!(
                original_types,
                ast.node_views()
                    .filter(|v| v.kind() == &K::TypeExpression)
                    .map(|v| v.id())
                    .collect()
            );
            assert_eq!(
                original_heads,
                ast.node_views()
                    .filter(|v| v.kind() == &K::TypeHead)
                    .map(|v| v.id())
                    .collect()
            );
            let result = Producer::build_source_theorem_equalities(&source, symbols, &scope)?;
            assert_eq!(
                result,
                Producer::build_source_theorem_equalities(&source, symbols, &scope)?
            );
            assert_eq!(result.0, baseline.0);
            assert_eq!(result.1, baseline.1);
            assert_eq!(result.3, baseline.3);
            let handoff = result.2.source_type().unwrap();
            assert_eq!(handoff.source_id(), expected.source_id);
            assert_eq!(handoff.module_id(), &expected.module_id);
            assert_eq!(handoff.applications().len(), expected.applications.len());
            assert_eq!(handoff.expressions().len(), expected.expressions.len());
            assert!(handoff.arguments().is_empty());
            assert!(handoff.definition_returns().is_empty());
            assert!(handoff.mode_rhs().is_empty());
            assert!(handoff.structure_members().is_empty());
            for ((id, actual), wanted) in handoff.applications().iter().zip(&expected.applications)
            {
                assert_eq!(id.index(), actual.id().index());
                assert_eq!(actual.binding(), wanted.binding);
                assert_eq!(actual.source_ordinal(), wanted.source_ordinal);
                assert_eq!(actual.root(), wanted.root);
                let declaration = result
                    .2
                    .source_context()
                    .unwrap()
                    .declarations()
                    .iter()
                    .nth(id.index())
                    .unwrap()
                    .1;
                assert_eq!(actual.binding(), declaration.binding);
                assert_eq!(actual.source_ordinal(), declaration.source_ordinal);
                assert_eq!(
                    handoff
                        .expressions()
                        .get(actual.root())
                        .unwrap()
                        .source_range(),
                    declaration.written_type_range
                );
            }
            for ((id, actual), wanted) in handoff.expressions().iter().zip(&expected.expressions) {
                assert_eq!(
                    id.index(),
                    expected
                        .expressions
                        .iter()
                        .position(|row| row.site == wanted.site)
                        .unwrap()
                );
                assert_eq!(actual.source_id(), wanted.source_id);
                assert_eq!(actual.module_id(), &wanted.module_id);
                assert_eq!(actual.site(), &wanted.site);
                assert_eq!(actual.source_range(), wanted.source_range);
                assert_eq!(actual.spelling(), wanted.spelling);
                assert_eq!(actual.head_site(), &wanted.head_site);
                assert_eq!(actual.head_range(), wanted.head_range);
                assert_eq!(actual.head_spelling(), wanted.head_spelling);
                assert_eq!(actual.form(), wanted.form);
                assert_eq!(actual.head(), &wanted.head);
                assert_eq!(actual.recovery(), wanted.recovery);
            }
            assert!(Producer::build(expected, &result.0, symbols, result.2.nodes()).is_err());
            let parts = TypedAstParts {
                source_id: result.2.source_id(),
                module_id: result.2.module_id().clone(),
                resolved_root: result.2.resolved_root(),
                source_context: result.2.source_context().cloned(),
                source_type: Some(handoff.clone()),
                source_attribute: result.2.source_attribute().cloned(),
                nodes: result.2.nodes().clone(),
                contexts: result.2.contexts().clone(),
                types: result.2.types().clone(),
                facts: result.2.facts().clone(),
                coercions: result.2.coercions().clone(),
                initial_obligations: result.2.initial_obligations().clone(),
                diagnostics: result.2.diagnostics().clone(),
            };
            for with_type in [false, true] {
                let mut rebuilt = parts.clone();
                if !with_type {
                    rebuilt.source_type = None;
                }
                let installed = TypedAst::try_new(rebuilt)
                    .and_then(|ast| ast.with_source_term(typed.source_term().unwrap().clone()))
                    .and_then(|ast| {
                        ast.with_source_atomic_formula(
                            typed.source_atomic_formula().unwrap().clone(),
                        )
                    })
                    .unwrap();
                assert_eq!(installed, if with_type { &result.2 } else { typed }.clone());
            }
            let first = handoff.expressions().iter().next().unwrap().1;
            let alternative_context = parts
                .contexts
                .iter()
                .find(|(id, _)| {
                    Some(*id) != parts.nodes.node(first.site().node()).unwrap().links.context
                })
                .unwrap()
                .0;
            for fault in [
                "type kind",
                "head kind",
                "missing child",
                "other head",
                "type context",
                "head context",
            ] {
                if fault == "other head" && handoff.expressions().len() < 2 {
                    continue;
                }
                let mut nodes = parts
                    .nodes
                    .iter()
                    .map(|(_, row)| row.clone())
                    .collect::<Vec<_>>();
                match fault {
                    "type kind" => {
                        nodes[first.site().node().index()].kind = TypedNodeKind::new("Term")
                    }
                    "head kind" => {
                        nodes[first.head_site().node().index()].kind = TypedNodeKind::new("Term")
                    }
                    "missing child" => nodes[first.site().node().index()].children.clear(),
                    "other head" => {
                        nodes[first.site().node().index()].children = vec![
                            handoff
                                .expressions()
                                .iter()
                                .nth(1)
                                .unwrap()
                                .1
                                .head_site()
                                .node(),
                        ]
                    }
                    "type context" => {
                        nodes[first.site().node().index()].links.context = Some(alternative_context)
                    }
                    _ => {
                        nodes[first.head_site().node().index()].links.context =
                            Some(alternative_context)
                    }
                }
                let mut corrupt = parts.clone();
                corrupt.nodes = TypedArena::try_new(parts.nodes.root(), nodes).unwrap();
                assert_eq!(
                    TypedAst::try_new(corrupt).unwrap_err(),
                    TypedAstError::InvalidSourceType,
                    "{fault}"
                );
                assert_eq!(
                    TypedAst::try_new(parts.clone()).unwrap().source_type(),
                    Some(handoff)
                );
            }
            let shape = handoff.debug_text();
            if let Some(previous) = &graph {
                assert_eq!(&shape, previous);
            }
            graph = Some(shape);
            assert_eq!((source.clone(), symbols.clone(), scope.clone()), before);
            assert_eq!(ast.snapshot_text(), ast_before);
            assert_eq!(frontend.source, capture_before);
        }
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_type_handoff_preserves_upstream_errors() {
    use mizar_checker::{
        source_context::SourceBindingContextProducer as ContextProducer,
        source_type::SourceTypeProducer as Producer,
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
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SourceInput, SourceOriginInput,
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
    let path = root.join("src/theorem_type_errors.miz");
    fs::write(&path, text).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
        super::super::super::ParseOnlyImportProvider,
        MizarParserSeam,
    )
    .run(
        SourceUnitRequest {
            snapshot: snapshot_id(781),
            input: SourceInput {
                package_id: PackageId::new("theorem-type-errors"),
                module_path: ModulePath::new("owners"),
                normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                edition: Edition::new("2026"),
                origin: SourceOriginInput::Disk { path: path.clone() },
            },
        },
        &ids,
    )
    .unwrap();
    assert!(frontend.diagnostics.is_empty());
    let capture_before = frontend.source.clone();
    let ast = frontend.ast.as_ref().unwrap();
    let ast_before = ast.snapshot_text();
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
    assert!(ContextProducer::build_source_theorem_equalities(&source, symbols, &scope).is_ok());
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
    for control in ["foreign scope", "recovered source", "foreign source"] {
        let source_id = if control == "foreign source" {
            ids.next_source_id(snapshot_id(781)).unwrap()
        } else {
            ast.source_id
        };
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut rebuilt = Vec::new();
        for view in ast.node_views() {
            let mut range = view.range();
            range.source_id = source_id;
            let children = view
                .children()
                .iter()
                .map(|id| rebuilt[id.index()])
                .collect();
            rebuilt.push(match view.kind() {
                K::Token(token) if control == "foreign scope" && token.text.as_ref() == "X" => {
                    builder.add_token(token.kind, "Y", range)
                }
                K::Token(token) if control == "recovered source" && token.text.as_ref() == "=" => {
                    builder.add_recovered_token(token.kind, token.text.clone(), range)
                }
                K::Token(token) => builder.add_token(token.kind, token.text.clone(), range),
                kind => builder.add_node(kind.clone(), range, children),
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
        let expected = ContextProducer::build_source_theorem_equalities(&source, &symbols, &scope)
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
    assert_eq!(frontend.source, capture_before);
    assert_eq!(ast.snapshot_text(), ast_before);
    fs::remove_dir_all(root).unwrap();
}
