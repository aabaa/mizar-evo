use super::*;

#[test]
fn source_theorem_final_metadata_retains_native_inputs() -> Result<(), String> {
    use mizar_checker::{
        source_atomic_formula::SourceAtomicRequestKind,
        type_checker::{
            AttributeSet, DeclarationChecker as Checker, NormalizedTypeStatus, TypeHeadRef,
            check_source_theorem_metadata as check,
        },
        typed_ast::{TypeEntryActual, TypedSiteRef},
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
    use std::collections::{BTreeMap, BTreeSet};
    let fixture = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let cases = [fixture.to_owned(),
        "theorem Pair: for A, Spare, B being set holds A = B proof let L, Unused, R be set; thus L = R; end;".into(),
        "theorem Shadow: for A being set holds for A being set holds A = A proof end;".into()];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/theorem_metadata.miz");
    let ids = InMemorySessionIdAllocator::new();
    for text in cases {
        let mut disk_metadata = None;
        for buffer in [false, true] {
            fs::write(&path, if buffer { "stale disk content" } else { &text }).unwrap();
            let frontend = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
                super::super::super::ParseOnlyImportProvider,
                MizarParserSeam,
            )
            .run(
                SourceUnitRequest {
                    snapshot: snapshot_id(784),
                    input: SourceInput {
                        package_id: PackageId::new("theorem-metadata-test"),
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
            let scope =
                Resolver::resolve_proof_occurrences(Input::new(ast, &module, symbols)).unwrap();
            let before = (source.clone(), symbols.clone(), scope.clone());
            // Capture every original source site before invoking the new entry.
            let source_sites = ast
                .node_views()
                .map(|view| {
                    (
                        source.resolved_node_for(view.id()).unwrap(),
                        (
                            view.range(),
                            view.children()
                                .iter()
                                .map(|child| source.resolved_node_for(*child).unwrap())
                                .collect::<Vec<_>>(),
                        ),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let binding_sites = scope
                .bindings()
                .iter()
                .map(|row| (source.resolved_node_for(row.node()).unwrap(), row.range()))
                .collect::<BTreeMap<_, _>>();
            let reference_sites = scope
                .references()
                .iter()
                .map(|row| (source.resolved_node_for(row.node()).unwrap(), row.range()))
                .collect::<BTreeMap<_, _>>();
            let baseline = Checker::check_source_theorem_equalities_with_typed_declarations(
                &source, symbols, &scope,
            )?;
            let result = check(&source, symbols, &scope)?;
            assert_eq!(result.0, baseline.0);
            assert_eq!(result.1, baseline.1);
            assert_eq!(result.2, baseline.2);
            let (_, inference, typed, metadata) = &result;
            assert_eq!(metadata.source_id(), source.source_id());
            assert_eq!(metadata.module_id(), &module);
            assert_eq!(metadata.source_context(), typed.source_context());
            assert_eq!(metadata.source_type(), typed.source_type());
            assert_eq!(metadata.source_term(), typed.source_term());
            assert_eq!(
                metadata.source_atomic_formula(),
                typed.source_atomic_formula()
            );
            assert_eq!(
                typed.contexts(),
                typed.source_context().unwrap().local_contexts()
            );
            let atomic = metadata.source_atomic_formula().unwrap();
            assert!(!atomic.requests().is_empty());
            for (_, request) in atomic.requests().iter() {
                assert_eq!(request.kind(), SourceAtomicRequestKind::OperandExpectedType);
            }
            for (_, formula) in inference.formulas().iter() {
                assert!(formula.expected_types.is_empty());
            }
            for (_, term) in inference.terms().iter() {
                assert!(term.expected_type.is_none());
            }
            let node_map = metadata
                .nodes()
                .iter()
                .map(|(id, row)| (row.typed_node, id))
                .collect::<BTreeMap<_, _>>();
            assert_eq!(node_map.len(), typed.nodes().len());
            assert_eq!(
                metadata.nodes().root(),
                typed.nodes().root().map(|id| node_map[&id])
            );
            let mut covered = BTreeSet::new();
            for (id, row) in metadata.nodes().iter() {
                assert_eq!(node_map[&row.typed_node], id);
                let original = typed.nodes().node(row.typed_node).unwrap();
                let resolved = original.resolved_node.unwrap();
                let (range, children) = &source_sites[&resolved];
                assert!(covered.insert(resolved));
                assert_eq!(row.source_range, *range);
                assert_eq!(
                    row.children,
                    original
                        .children
                        .iter()
                        .map(|child| node_map[child])
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    original
                        .children
                        .iter()
                        .map(|child| typed.nodes().node(*child).unwrap().resolved_node.unwrap())
                        .collect::<Vec<_>>(),
                    *children
                );
                let site = TypedSiteRef::Node(row.typed_node);
                let type_rows = typed
                    .types()
                    .iter()
                    .filter(|(_, entry)| entry.owner == site)
                    .collect::<Vec<_>>();
                let expected = type_rows.iter().find_map(|(_, entry)| match entry.actual {
                    TypeEntryActual::Known(id) => Some(id),
                    _ => None,
                });
                assert_eq!(row.final_type, expected);
                if binding_sites.contains_key(&resolved) || reference_sites.contains_key(&resolved)
                {
                    let normalized = inference
                        .normalized_types()
                        .get(row.final_type.unwrap())
                        .unwrap();
                    assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
                    assert_eq!(normalized.status, NormalizedTypeStatus::Known);
                    assert!(normalized.args.is_empty());
                    assert_eq!(normalized.attributes, AttributeSet::empty());
                }
                assert!(row.diagnostics.is_empty());
                assert_eq!(row.metadata, metadata.expr_metadata().id_by_site(&site));
            }
            assert_eq!(covered.len(), source_sites.len());
            let primary = metadata.source_term().unwrap();
            assert_eq!(primary.references().len(), reference_sites.len());
            for (_, term) in primary.terms().iter() {
                let resolved = typed
                    .nodes()
                    .node(term.site().node())
                    .unwrap()
                    .resolved_node
                    .unwrap();
                assert_eq!(reference_sites[&resolved], term.source_range());
                assert_eq!(
                    &text[term.source_range().start..term.source_range().end],
                    term.spelling()
                );
            }
            let expected_sites = primary
                .terms()
                .iter()
                .map(|(_, row)| row.site().clone())
                .chain(atomic.formulas().iter().map(|(_, row)| row.site().clone()))
                .collect::<BTreeSet<_>>();
            assert_eq!(metadata.expr_metadata().len(), expected_sites.len());
            let mut expression_rows = Vec::new();
            for (id, row) in metadata.expr_metadata().iter() {
                let input = baseline
                    .3
                    .iter()
                    .find(|input| input.expr == row.expr)
                    .unwrap();
                assert_eq!(row.typed_site, input.typed_site);
                assert_eq!(metadata.expr_metadata().id_by_expr(&row.expr), Some(id));
                assert_eq!(metadata.expr_metadata().get_by_expr(&row.expr), Some(row));
                assert_eq!(
                    metadata.expr_metadata().get_by_site(&row.typed_site),
                    Some(row)
                );
                let node = typed.nodes().node(row.typed_site.node()).unwrap();
                let resolved = node.resolved_node.unwrap();
                assert_eq!(row.source_range, source_sites[&resolved].0);
                assert_eq!(
                    row.local_context,
                    input.local_context.or(node.links.context)
                );
                let context = typed.contexts().get(row.local_context.unwrap()).unwrap();
                assert!(row.visible_facts.is_empty());
                assert!(row.cluster_facts.is_empty());
                assert!(row.inserted_views.is_empty());
                assert!(row.overload.is_none());
                assert!(row.diagnostics.is_empty());
                assert_eq!(
                    row.final_type,
                    metadata
                        .nodes()
                        .node(node_map[&row.typed_site.node()])
                        .unwrap()
                        .final_type
                );
                if atomic
                    .formulas()
                    .iter()
                    .any(|(_, formula)| formula.site() == &row.typed_site)
                {
                    assert!(row.final_type.is_none());
                }
                expression_rows.push((
                    row.expr.clone(),
                    row.source_range.start,
                    row.source_range.end,
                    row.final_type
                        .map(|id| inference.normalized_types().get(id).unwrap().head.clone()),
                    context.layer,
                ));
            }
            assert!(metadata.collection_candidates().is_empty());
            assert!(metadata.expanded_candidates().is_empty());
            assert!(metadata.template_expansions().is_empty());
            assert!(metadata.viable_candidates().is_empty());
            assert!(metadata.viability_decisions().is_empty());
            assert!(metadata.specificity_graphs().is_empty());
            assert!(metadata.resolved_overloads().is_empty());
            assert!(metadata.inserted_coercions().is_empty());
            assert!(metadata.cluster_facts().is_empty());
            assert!(metadata.diagnostics().is_empty());
            assert!(metadata.checked_formulas().is_empty());
            assert!(metadata.statement_semantics().is_empty());
            assert!(metadata.checked_proofs().is_empty());
            assert!(metadata.checked_proof_nodes().is_empty());
            assert!(metadata.checked_terminal_goals().is_empty());
            assert!(typed.facts().is_empty());
            assert!(typed.coercions().is_empty());
            assert!(typed.initial_obligations().is_empty());
            if let Some((source_id, expected_rows)) = &disk_metadata {
                assert_ne!(*source_id, metadata.source_id());
                assert_eq!(&expression_rows, expected_rows);
            } else {
                disk_metadata = Some((metadata.source_id(), expression_rows));
            }
            assert_eq!(result, check(&source, symbols, &scope)?);
            assert_eq!((source.clone(), symbols.clone(), scope.clone()), before);
            assert_eq!(ast.snapshot_text(), ast_before);
            assert_eq!(frontend.source, capture_before);
        }
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_final_metadata_preserves_upstream_errors() {
    use mizar_checker::type_checker::{
        DeclarationChecker as Checker, check_source_theorem_metadata as check,
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
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SourceInput, SourceOriginInput,
    };
    use mizar_syntax::{SurfaceAstBuilder, SurfaceNodeKind as K};
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let registration = include_str!(
        "../../../../../../tests/miz/pass/clusters/pass_advanced_semantics_existential_registration_001.miz"
    );
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/theorem_metadata_errors.miz");
    let ids = InMemorySessionIdAllocator::new();
    for (ordinal, text) in [
        text.to_owned(),
        text.replace("set", "object"),
        format!("{text}\n{registration}"),
    ]
    .into_iter()
    .enumerate()
    {
        fs::write(&path, &text).unwrap();
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: snapshot_id(785),
                input: SourceInput {
                    package_id: PackageId::new("theorem-metadata-errors"),
                    module_path: ModulePath::new("owners"),
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
        let symbols = collected.env();
        let scope = Resolver::resolve_proof_occurrences(Input::new(ast, &module, symbols)).unwrap();
        let empty = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let mut controls = vec![(source.clone(), empty.clone(), scope.clone())];
        if ordinal == 0 {
            let foreign_module = ModuleId::new(PackageId::new("foreign"), ModulePath::new("other"));
            controls.push((
                SurfaceResolvedArena::lower(ast, &foreign_module).unwrap(),
                symbols.clone(),
                scope.clone(),
            ));
            for control in ["foreign scope", "recovered source", "foreign source"] {
                let source_id = if control == "foreign source" {
                    ids.next_source_id(snapshot_id(785)).unwrap()
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
                        K::Token(token)
                            if control == "foreign scope" && token.text.as_ref() == "X" =>
                        {
                            builder.add_token(token.kind, "Y", range)
                        }
                        K::Token(token)
                            if control == "recovered source" && token.text.as_ref() == "=" =>
                        {
                            builder.add_recovered_token(token.kind, token.text.clone(), range)
                        }
                        K::Token(token) => builder.add_token(token.kind, token.text.clone(), range),
                        kind => builder.add_node(kind.clone(), range, children),
                    });
                }
                let changed = builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None);
                if control == "foreign scope" {
                    let foreign =
                        Resolver::resolve_proof_occurrences(Input::new(&changed, &module, symbols))
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
        } else {
            controls.push((source.clone(), symbols.clone(), scope.clone()));
            if ordinal == 2 {
                assert!(!symbols.registrations().is_empty());
            }
        }
        for (source, symbols, scope) in controls {
            let before = (source.clone(), symbols.clone(), scope.clone());
            let expected = Checker::check_source_theorem_equalities_with_typed_declarations(
                &source, &symbols, &scope,
            )
            .unwrap_err();
            assert_eq!(check(&source, &symbols, &scope).unwrap_err(), expected);
            assert_eq!((source, symbols, scope), before);
        }
        assert_eq!(frontend.source, capture_before);
        assert_eq!(ast.snapshot_text(), ast_before);
    }
    fs::remove_dir_all(root).unwrap();
}
