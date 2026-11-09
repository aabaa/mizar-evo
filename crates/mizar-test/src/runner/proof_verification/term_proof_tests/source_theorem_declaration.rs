use super::*;

#[test]
fn source_theorem_declaration_checking_retains_real_types() -> Result<(), String> {
    use mizar_checker::{
        binding_env::BinderIdentity,
        source_type::SourceTypeProducer as Producer,
        type_checker::{
            AttributeSet, DeclarationChecker as Checker, DeclarationKind, DeclarationStatus,
            NormalizedTypeStatus, TypeHeadRef,
        },
        typed_ast::{
            BindingTypeRef, ContextRecoveryState, SourceRangeKey, TypeContextLayer,
            TypeEntryActual, TypeProvenance, TypeRuleId, TypeStatus, TypedSiteRef,
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
        names::{SourceVariableBindingKind, SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::{SignatureProjectionExtractor, source_proof_scope_maps},
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
    let path = root.join("src/theorem_declarations.miz");
    let ids = InMemorySessionIdAllocator::new();
    for text in cases {
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
                        package_id: PackageId::new("theorem-declaration-test"),
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
            let baseline = Producer::build_source_theorem_equalities(&source, symbols, &scope)?;
            let typed = &baseline.2;
            let maps = source_proof_scope_maps(&source, symbols, &scope)?;
            let typed_for = |surface| {
                let resolved = source.resolved_node_for(surface).unwrap();
                let matching = typed
                    .nodes()
                    .iter()
                    .filter(|(_, row)| row.resolved_node == Some(resolved))
                    .collect::<Vec<_>>();
                assert_eq!(matching.len(), 1);
                matching[0].0
            };
            // Original syntax and resolver identities determine every expected declaration.
            let mut expected = BTreeMap::new();
            let mut raw_roots = BTreeMap::new();
            for original in scope.bindings() {
                let matching = baseline
                    .0
                    .bindings()
                    .iter()
                    .filter(|(_, binding)| {
                        matches!(&binding.identity, BinderIdentity::ResolverLocal {
                        scope, ordinal, declaration_range
                    } if scope == original.scope() && *ordinal == original.ordinal()
                        && *declaration_range == original.range())
                    })
                    .collect::<Vec<_>>();
                assert_eq!(matching.len(), 1);
                let (binding, entry) = matching[0];
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
                assert_eq!(&text[written.range().start..written.range().end], "set");
                let kind = match original.kind() {
                    SourceVariableBindingKind::Quantifier => DeclarationKind::QuantifiedVariable,
                    SourceVariableBindingKind::Let => DeclarationKind::Let,
                    _ => panic!("unexpected declaration"),
                };
                let site = TypedSiteRef::Node(typed_for(original.node()));
                let logical_type = TypedSiteRef::Role {
                    node: typed_for(written.id()),
                    role: format!("theorem.declaration.type:{}", binding.index()).into(),
                };
                assert_eq!(entry.declaration_range, original.range());
                assert_eq!(
                    baseline
                        .0
                        .contexts()
                        .get(entry.owner_context)
                        .unwrap()
                        .lexical_scope
                        .as_ref(),
                    Some(original.scope())
                );
                let applications = typed
                    .source_type()
                    .unwrap()
                    .applications()
                    .iter()
                    .filter(|(_, row)| row.binding() == binding)
                    .collect::<Vec<_>>();
                assert_eq!(applications.len(), 1);
                let raw = typed
                    .source_type()
                    .unwrap()
                    .expressions()
                    .get(applications[0].1.root())
                    .unwrap();
                assert_eq!(raw.site(), &TypedSiteRef::Node(typed_for(written.id())));
                assert_eq!(raw.source_range(), written.range());
                assert_eq!(raw.spelling(), "set");
                let root = raw_roots
                    .entry(written.id())
                    .or_insert((applications[0].1.root(), BTreeSet::new()));
                assert_eq!(root.0, applications[0].1.root());
                assert!(root.1.insert(logical_type.clone()));
                assert!(
                    expected
                        .insert(
                            binding,
                            (
                                entry.owner_context,
                                site,
                                kind,
                                logical_type,
                                original.range(),
                                written.range()
                            )
                        )
                        .is_none()
                );
            }
            assert_eq!(expected.len(), baseline.0.bindings().len());
            assert_eq!(
                raw_roots.len(),
                typed.source_type().unwrap().expressions().len()
            );
            if text.contains("Spare") {
                assert!(raw_roots.values().any(|(_, roles)| roles.len() == 3));
            }
            let mut expected_contexts = BTreeMap::new();
            for view in ast.node_views() {
                let parent = maps.1.get(&view.id());
                if parent.is_some_and(|parent| maps.0[parent] == maps.0[&view.id()]) {
                    continue;
                }
                let scope_here = &maps.0[&view.id()];
                let owner = TypedSiteRef::Node(typed_for(view.id()));
                let contexts = baseline
                    .0
                    .contexts()
                    .iter()
                    .filter(|(_, row)| {
                        if parent.is_none() {
                            row.parent.is_none()
                        } else {
                            row.lexical_scope.as_ref() == Some(scope_here)
                        }
                    })
                    .collect::<Vec<_>>();
                assert_eq!(contexts.len(), 1);
                let layer = match view.kind() {
                    K::Root => TypeContextLayer::Module,
                    K::QuantifiedFormula(_) => TypeContextLayer::Expression,
                    K::ProofBlock => TypeContextLayer::Proof,
                    _ => panic!("unexpected lexical boundary"),
                };
                let parent_scope = parent.map(|id| maps.0[id].clone());
                assert!(
                    expected_contexts
                        .insert(
                            scope_here.clone(),
                            (contexts[0].0, owner, layer, parent_scope)
                        )
                        .is_none()
                );
            }
            assert_eq!(expected_contexts.len(), baseline.0.contexts().len());
            let result = Checker::check_source_theorem_equalities(&source, symbols, &scope)?;
            assert_eq!(result.0, baseline);
            assert_eq!(
                result,
                Checker::check_source_theorem_equalities(&source, symbols, &scope)?
            );
            assert_eq!(
                Producer::build_source_theorem_equalities(&source, symbols, &scope)?,
                baseline
            );
            let checked = &result.1;
            assert!(checked.diagnostics().is_empty());
            assert!(checked.facts().is_empty());
            assert_eq!(checked.declarations().len(), expected.len());
            assert_eq!(checked.type_entries().len(), 2 * expected.len());
            let canonical_range = expected
                .values()
                .map(|row| row.5)
                .min_by_key(|range| (range.start, range.end))
                .unwrap();
            assert_eq!(checked.normalized_types().len(), 1);
            let (normalized_id, normalized) = checked.normalized_types().iter().next().unwrap();
            assert_eq!(normalized.id, normalized_id);
            assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
            assert_eq!(normalized.status, NormalizedTypeStatus::Known);
            assert!(normalized.args.is_empty());
            assert_eq!(normalized.attributes, AttributeSet::empty());
            assert_eq!(normalized.source.range, canonical_range);
            assert_eq!(normalized.source.spelling, "set");
            let mut covered_bindings = BTreeSet::new();
            let mut covered_entries = BTreeSet::new();
            let mut declarations_by_scope = BTreeMap::<_, BTreeSet<_>>::new();
            for (id, declaration) in checked.declarations().iter() {
                assert_eq!(declaration.id, id);
                assert!(covered_bindings.insert(declaration.binding));
                let (context, site, kind, logical_type, range, _) = &expected[&declaration.binding];
                assert_eq!(declaration.context, *context);
                assert_eq!(&declaration.site, site);
                assert_eq!(declaration.kind, *kind);
                assert_eq!(declaration.type_site.as_ref(), Some(logical_type));
                assert_eq!(declaration.status, DeclarationStatus::Checked);
                assert!(declaration.deferred.is_empty());
                assert!(declaration.facts.is_empty());
                let entry_id = declaration.type_entry.unwrap();
                assert!(covered_entries.insert(entry_id));
                let entry = checked.type_entries().get(entry_id).unwrap();
                assert_eq!(entry.id, entry_id);
                assert_eq!(&entry.owner, site);
                assert_eq!(entry.expected, None);
                assert_eq!(entry.status, TypeStatus::Known);
                assert_eq!(
                    entry.provenance,
                    TypeProvenance::Declared(SourceRangeKey::from(*range))
                );
                let TypeEntryActual::Known(actual) = entry.actual else {
                    panic!("missing type");
                };
                assert_eq!(checked.normalized_types().get(actual), Some(normalized));
                let logical_entries = checked
                    .type_entries()
                    .iter()
                    .filter(|(_, row)| &row.owner == logical_type)
                    .collect::<Vec<_>>();
                assert_eq!(logical_entries.len(), 1);
                let (logical_id, logical_entry) = logical_entries[0];
                assert!(covered_entries.insert(logical_id));
                assert_eq!(logical_entry.id, logical_id);
                assert_eq!(logical_entry.actual, entry.actual);
                assert_eq!(logical_entry.expected, None);
                assert_eq!(logical_entry.status, TypeStatus::Known);
                assert_eq!(
                    logical_entry.provenance,
                    TypeProvenance::Inferred(TypeRuleId::new("type-expression-normalization"))
                );
                declarations_by_scope
                    .entry(*context)
                    .or_default()
                    .insert(BindingTypeRef::Site(site.clone()));
            }
            assert_eq!(covered_bindings, expected.keys().copied().collect());
            assert_eq!(
                covered_entries,
                checked.type_entries().iter().map(|(id, _)| id).collect()
            );
            let mut local_for_scope = BTreeMap::new();
            for (scope_here, (binding_context, owner, layer, _)) in &expected_contexts {
                let matching = checked
                    .contexts()
                    .iter()
                    .filter(|(_, row)| &row.owner == owner)
                    .collect::<Vec<_>>();
                assert_eq!(matching.len(), 1);
                let (id, context) = matching[0];
                assert_eq!(context.id, id);
                assert_eq!(&context.layer, layer);
                assert_eq!(context.recovery, ContextRecoveryState::Normal);
                assert!(context.introduced_assumptions.is_empty());
                assert!(context.visible_facts.is_empty());
                assert_eq!(
                    context.bindings.iter().cloned().collect::<BTreeSet<_>>(),
                    declarations_by_scope
                        .get(binding_context)
                        .cloned()
                        .unwrap_or_default()
                );
                assert_eq!(
                    context.bindings.len(),
                    declarations_by_scope
                        .get(binding_context)
                        .map_or(0, BTreeSet::len)
                );
                local_for_scope.insert(scope_here, id);
            }
            assert_eq!(local_for_scope.len(), checked.contexts().len());
            for (scope_here, (_, _, _, parent)) in &expected_contexts {
                let context = checked.contexts().get(local_for_scope[scope_here]).unwrap();
                assert_eq!(
                    context.parent,
                    parent.as_ref().map(|scope| local_for_scope[scope])
                );
            }
            assert_eq!((source.clone(), symbols.clone(), scope.clone()), before);
            assert_eq!(ast.snapshot_text(), ast_before);
            assert_eq!(frontend.source, capture_before);
        }
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_declaration_checking_preserves_upstream_errors() {
    use mizar_checker::{
        source_type::SourceTypeProducer as Producer, type_checker::DeclarationChecker as Checker,
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
    let path = root.join("src/theorem_declaration_errors.miz");
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
    let unsupported = parse(&format!("reserve spare for object; {text}"));
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
        let expected =
            Producer::build_source_theorem_equalities(&source, &symbols, &scope).unwrap_err();
        assert_eq!(
            Checker::check_source_theorem_equalities(&source, &symbols, &scope).unwrap_err(),
            expected
        );
        assert_eq!(
            Checker::check_source_theorem_equalities(&source, &symbols, &scope).unwrap_err(),
            expected
        );
        assert_eq!((source, symbols, scope), before);
    }
    assert_eq!(frontend.source, capture_before);
    assert_eq!(ast.snapshot_text(), ast_before);
    fs::remove_dir_all(root).unwrap();
}
