use super::*;

#[test]
fn source_theorem_lexical_contexts_retain_real_scopes_and_consumers() -> Result<(), String> {
    use mizar_checker::{
        binding_env::{
            BindingContextId, BindingContextLayer, BindingContextOwner, BindingContextRecovery,
            BindingId, BindingLookupResult, BindingLookupSite,
        },
        type_checker::{
            SourceVariableSemanticsChecker as Semantics, TermFormulaChecker as Checker,
        },
    };
    use mizar_core::{
        core_ir::{CoreProofStatus, ObligationSeedStatus},
        elaborator::lower_source_theorem_skeletons,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::NamespacePath,
        labels::{LabelResolver, ProofLabelSourceCollector},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::{
            SignatureProjectionExtractor, source_proof_scope_maps,
            validate_source_proof_occurrences,
        },
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
    let sources = [
        fixture.to_owned(),
        fixture.replace("Lem1", "Seed").replace("let X be set;", "let Local be set;").replace("thus X = X", "thus Local = Local"),
        "theorem Pair: for A, Spare, B being set holds A = B proof let L, Unused, R be set; thus L = R; end;".into(),
        "theorem Nested: for A being set for Unused being set holds for B being set holds A = B proof let L be set; let Idle be set; let R be set; thus L = R; end;".into(),
        "theorem Groups: for A being set, Unused, B being set holds A = B proof let L be set, Idle, R be set; thus L = R; end;".into(),
        "theorem Shadow: for A being set holds for A being set holds A = A proof let A be set; thus A = A; end;".into(),
        "theorem Unused: for A being set holds A = A proof end;".into(),
    ];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/theorem_contexts.miz");
    let ids = InMemorySessionIdAllocator::new();
    let mut different_clocks = false;
    for (case, text) in sources.iter().enumerate() {
        let mut disk_capture = None;
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
                        package_id: PackageId::new("lexical-context-test"),
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
            let baseline = Checker::check_source_theorem_equalities(&source, symbols, &scope)?;
            validate_source_proof_occurrences(&source, symbols, &scope)?;
            let maps = source_proof_scope_maps(&source, symbols, &scope)?;
            let env = Semantics::source_theorem_binding_env(&source, symbols, &scope)?;
            assert_eq!(env.source_id(), source.source_id());
            assert_eq!(env.module_id(), source.module());
            assert_eq!(env.diagnostics(), baseline.0.diagnostics());
            assert_eq!(maps.0.len(), ast.nodes().len());
            assert_eq!(
                maps.0.keys().copied().collect::<BTreeSet<_>>(),
                ast.node_views().map(|view| view.id()).collect()
            );
            let mut actual_parents = BTreeMap::new();
            for view in ast.node_views() {
                for child in view.children() {
                    if matches!(view.kind(), K::Root)
                        && matches!(ast.node_view(*child).unwrap().kind(), K::Token(_))
                    {
                        continue;
                    }
                    assert!(actual_parents.insert(*child, view.id()).is_none());
                }
            }
            assert_eq!(maps.1, actual_parents);
            let root_node = ast.root().unwrap();
            assert!(!maps.1.contains_key(&root_node));
            assert!(maps.0[&root_node].path().is_empty());
            assert_eq!(maps.1.len() + 1, maps.0.len());
            let mut boundaries = Vec::new();
            for view in ast.node_views() {
                let actual = source
                    .arena()
                    .node(source.resolved_node_for(view.id()).unwrap())
                    .unwrap();
                assert_eq!(actual.kind(), view.kind());
                assert_eq!(actual.origin().source_id(), ast.source_id);
                assert_eq!(actual.origin().module_id(), &module);
                assert_eq!(
                    actual.origin().anchor(),
                    &mizar_session::SourceAnchor::Range(view.range())
                );
                assert_eq!(
                    actual
                        .children()
                        .iter()
                        .map(|id| id.index())
                        .collect::<Vec<_>>(),
                    view.children()
                        .iter()
                        .map(|id| id.index())
                        .collect::<Vec<_>>()
                );
                assert_eq!(view.range().source_id, ast.source_id);
                let Some(parent) = maps.1.get(&view.id()) else {
                    continue;
                };
                let local = &maps.0[&view.id()];
                let outer = &maps.0[parent];
                if matches!(view.kind(), K::QuantifiedFormula(_) | K::ProofBlock) {
                    assert_ne!(local, outer);
                    assert!(local.path().starts_with(outer.path()));
                    assert_eq!(local.path().len(), outer.path().len() + 1);
                    boundaries.push((view.id(), local.clone(), outer.clone()));
                } else {
                    assert_eq!(local, outer);
                }
            }
            boundaries.sort_by(|left, right| {
                (left.1.path().len(), left.1.path()).cmp(&(right.1.path().len(), right.1.path()))
            });
            assert_eq!(env.contexts().len(), boundaries.len() + 1);
            let context_for = |local: &mizar_resolve::names::LocalTermScope| {
                if local.path().is_empty() {
                    BindingContextId::new(0)
                } else {
                    env.contexts()
                        .iter()
                        .find(|(_, row)| row.lexical_scope.as_ref() == Some(local))
                        .unwrap()
                        .0
                }
            };
            let module_context = env.contexts().get(BindingContextId::new(0)).unwrap();
            assert_eq!(module_context.owner, BindingContextOwner::Module);
            assert_eq!(module_context.layer, BindingContextLayer::Module);
            assert_eq!(module_context.parent, None);
            assert_eq!(module_context.lexical_scope, None);
            for (index, (node, local, outer)) in boundaries.iter().enumerate() {
                let id = BindingContextId::new(index + 1);
                assert_eq!(context_for(local), id);
                let row = env.contexts().get(id).unwrap();
                let view = ast.node_view(*node).unwrap();
                assert_eq!(row.lexical_scope.as_ref(), Some(local));
                assert_eq!(row.parent, Some(context_for(outer)));
                let (owner, layer) = match view.kind() {
                    K::QuantifiedFormula(_) => (
                        BindingContextOwner::SourceFormula {
                            source_range: view.range(),
                        },
                        BindingContextLayer::Expression,
                    ),
                    K::ProofBlock => (
                        BindingContextOwner::SourceStatement {
                            source_range: view.range(),
                        },
                        BindingContextLayer::Proof,
                    ),
                    _ => unreachable!(),
                };
                assert_eq!(row.owner, owner);
                assert_eq!(row.layer, layer);
            }
            assert_eq!(env.bindings().len(), baseline.0.bindings().len());
            assert_eq!(env.bindings().len(), scope.bindings().len());
            for (id, entry) in env.bindings().iter() {
                let old = baseline.0.bindings().get(id).unwrap();
                let declaration = scope
                    .bindings()
                    .iter()
                    .find(|binding| binding.range() == old.declaration_range)
                    .unwrap();
                assert_eq!(maps.0.get(&declaration.node()), Some(declaration.scope()));
                assert_eq!(entry.owner_context, context_for(declaration.scope()));
                let mut expected = old.clone();
                expected.owner_context = entry.owner_context;
                assert_eq!(entry, &expected);
                assert_eq!(
                    &text[entry.declaration_range.start..entry.declaration_range.end],
                    entry.spelling
                );
            }
            for (id, row) in env.contexts().iter() {
                assert_eq!(row.id, id);
                assert_eq!(row.recovery, BindingContextRecovery::Normal);
                let local = row.lexical_scope.as_ref().unwrap_or(&maps.0[&root_node]);
                let own = scope
                    .bindings()
                    .iter()
                    .filter(|binding| binding.scope() == local)
                    .map(|binding| BindingId::new(binding.id().index()))
                    .collect::<Vec<_>>();
                let visible = scope
                    .bindings()
                    .iter()
                    .filter(|binding| local.path().starts_with(binding.scope().path()))
                    .map(|binding| BindingId::new(binding.id().index()))
                    .collect::<Vec<_>>();
                assert_eq!(row.bindings, own);
                assert_eq!(row.visible_bindings, visible);
                assert!(
                    row.visible_bindings
                        .windows(2)
                        .all(|pair| pair[0] < pair[1])
                );
                for (_, other) in env.contexts().iter() {
                    if other.id != id
                        && !local.path().starts_with(
                            other
                                .lexical_scope
                                .as_ref()
                                .map_or(&[][..], |scope| scope.path()),
                        )
                    {
                        assert!(
                            other
                                .bindings
                                .iter()
                                .all(|binding| !row.visible_bindings.contains(binding))
                        );
                    }
                }
            }
            for reference in scope.references() {
                assert_eq!(maps.0.get(&reference.node()), Some(reference.scope()));
                let ordinal = scope
                    .bindings()
                    .iter()
                    .filter(|binding| binding.range().end <= reference.range().start)
                    .count();
                different_clocks |= ordinal != reference.ordinal();
                let lookup = BindingLookupSite::new(
                    reference.spelling(),
                    context_for(reference.scope()),
                    Some(reference.scope().clone()),
                    ordinal,
                );
                let expected = baseline
                    .0
                    .bindings()
                    .iter()
                    .find(|(_, binding)| {
                        binding.declaration_range
                            == scope.bindings()[reference.binding().index()].range()
                    })
                    .unwrap()
                    .0;
                assert_eq!(
                    env.lookup(&lookup).unwrap(),
                    BindingLookupResult::Local(expected)
                );
            }
            if (2..=4).contains(&case) {
                assert!(scope.bindings().iter().any(|binding| {
                    !scope
                        .references()
                        .iter()
                        .any(|reference| reference.binding() == binding.id())
                }));
            }
            if case == 5 {
                let nested = scope
                    .references()
                    .iter()
                    .find(|reference| reference.scope().path().len() > 1)
                    .unwrap();
                let winner = &scope.bindings()[nested.binding().index()];
                assert_eq!(winner.scope(), nested.scope());
                assert!(
                    scope
                        .bindings()
                        .iter()
                        .any(|binding| binding.spelling() == winner.spelling()
                            && binding.scope() != winner.scope()
                            && winner.scope().path().starts_with(binding.scope().path()))
                );
            }
            if case == 6 {
                assert!(
                    env.contexts()
                        .iter()
                        .any(|(_, row)| row.layer == BindingContextLayer::Proof
                            && row.bindings.is_empty()
                            && row.visible_bindings.is_empty())
                );
            }
            let after = Checker::check_source_theorem_equalities(&source, symbols, &scope)?;
            assert_eq!(after, baseline);
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
            let label_before = (labels.clone(), resolved.clone());
            let old_check = Semantics::check_theorem_skeletons(
                &source,
                &baseline.2,
                &scope,
                symbols,
                &labels,
                &resolved,
                None,
            );
            let new_check = Semantics::check_theorem_skeletons(
                &source, &after.2, &scope, symbols, &labels, &resolved, None,
            );
            if case < 2 {
                let old_check = old_check?;
                let new_check = new_check?;
                assert_eq!(new_check.owners(), old_check.owners());
                assert_eq!(new_check.scope(), old_check.scope());
                assert_eq!(new_check.labels(), old_check.labels());
                assert_eq!(new_check.resolved_labels(), old_check.resolved_labels());
                assert_eq!(new_check.typed_ast(), old_check.typed_ast());
                let expected = lower_source_theorem_skeletons(&old_check, None)?;
                let core = lower_source_theorem_skeletons(&new_check, None)?;
                assert_eq!(core, expected);
                assert!(
                    core.proofs()
                        .iter()
                        .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
                );
                assert_eq!(core.obligation_seeds().len(), 2);
                assert!(
                    core.obligation_seeds()
                        .iter()
                        .all(|(_, seed)| seed.status == ObligationSeedStatus::Active
                            && seed.goal.is_some())
                );
                let vcs = generate_core_vcs(&core, snapshot_id(777))?;
                assert_eq!(vcs, generate_core_vcs(&expected, snapshot_id(777))?);
                assert_eq!(vcs.vcs().len(), 2);
                assert!(
                    vcs.vcs()
                        .iter()
                        .all(|vc| vc.status == mizar_vc::vc_ir::VcStatus::Open)
                );
                assert_ne!(vcs.vcs()[0].local_context, vcs.vcs()[1].local_context);
            } else {
                assert_eq!(new_check.unwrap_err(), old_check.unwrap_err());
            }
            validate_source_proof_occurrences(&source, symbols, &scope)?;
            assert_eq!(source_proof_scope_maps(&source, symbols, &scope)?, maps);
            assert_eq!(
                Semantics::source_theorem_binding_env(&source, symbols, &scope)?,
                env
            );
            let signature = (maps, env.debug_text());
            if let Some((source_id, previous)) = &disk_capture {
                assert_ne!(*source_id, env.source_id());
                assert_eq!(previous, &signature);
            } else {
                disk_capture = Some((env.source_id(), signature));
            }
            assert_eq!(label_before, (labels, resolved));
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
    assert!(different_clocks);
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_lexical_contexts_preserve_upstream_errors() {
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker as Semantics, TermFormulaChecker as Checker,
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::{NamespacePath, SymbolEnv, SymbolEnvIndexes},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::{
            SignatureProjectionExtractor, source_proof_scope_maps,
            validate_source_proof_occurrences,
        },
    };
    use mizar_session::{ModulePath, PackageId};
    use mizar_syntax::{SurfaceAstBuilder, SurfaceNodeKind as K};
    let text = "theorem T: for X being set, Unused being set holds X = X proof let X be set; thus X = X; end;";
    let ast = parse(text);
    let module = ModuleId::new(
        PackageId::new("lexical-context-test"),
        ModulePath::new("owners"),
    );
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
    let empty = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
    let foreign_module = ModuleId::new(PackageId::new("foreign"), ModulePath::new("other"));
    let mut controls = vec![
        (
            "foreign environment",
            source.clone(),
            empty.clone(),
            scope.clone(),
        ),
        (
            "foreign source",
            SurfaceResolvedArena::lower(&ast, &foreign_module).unwrap(),
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
                K::Token(token)
                    if control == "foreign scope" && token.text.as_ref() == "Unused" =>
                {
                    builder.add_token(token.kind, "Hidden", view.range())
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
            assert_eq!(foreign.source_id(), scope.source_id());
            assert_eq!(foreign.references(), scope.references());
            assert_ne!(foreign.bindings(), scope.bindings());
            controls.push((control, source.clone(), symbols.clone(), foreign));
        } else {
            controls.push((
                control,
                SurfaceResolvedArena::lower(&changed, &module).unwrap(),
                symbols.clone(),
                scope.clone(),
            ));
        }
    }
    for text in [text.replace("set", "object"), String::new()] {
        let broader = parse(&text);
        let broader_source = SurfaceResolvedArena::lower(&broader, &module).unwrap();
        let shells = DeclarationShellCollector::new(&broader, &module).collect();
        let collected = SignatureProjectionExtractor::new(
            &broader,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        let broader_scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(&broader, &module, collected.env()),
        )
        .unwrap();
        let before = (
            broader_source.clone(),
            collected.env().clone(),
            broader_scope.clone(),
            broader.snapshot_text(),
        );
        validate_source_proof_occurrences(&broader_source, collected.env(), &broader_scope)
            .unwrap();
        let expected = Checker::check_source_theorem_equalities(
            &broader_source,
            collected.env(),
            &broader_scope,
        )
        .unwrap_err();
        let maps =
            source_proof_scope_maps(&broader_source, collected.env(), &broader_scope).unwrap();
        assert_eq!(maps.0.len(), broader.nodes().len());
        assert_eq!(maps.1.len() + 1, maps.0.len());
        assert!(!maps.1.contains_key(&broader.root().unwrap()));
        assert_eq!(
            Semantics::source_theorem_binding_env(&broader_source, collected.env(), &broader_scope)
                .unwrap_err(),
            expected
        );
        validate_source_proof_occurrences(&broader_source, collected.env(), &broader_scope)
            .unwrap();
        assert_eq!(
            source_proof_scope_maps(&broader_source, collected.env(), &broader_scope).unwrap(),
            maps
        );
        assert_eq!(
            Checker::check_source_theorem_equalities(
                &broader_source,
                collected.env(),
                &broader_scope
            )
            .unwrap_err(),
            expected
        );
        assert_eq!(
            Semantics::source_theorem_binding_env(&broader_source, collected.env(), &broader_scope)
                .unwrap_err(),
            expected
        );
        assert_eq!(
            before,
            (
                broader_source.clone(),
                collected.env().clone(),
                broader_scope.clone(),
                broader.snapshot_text()
            )
        );
        controls.push((
            "combined precedence",
            broader_source,
            empty.clone(),
            scope.clone(),
        ));
    }
    for (control, source, symbols, scope) in controls {
        let before = (source.clone(), symbols.clone(), scope.clone());
        let raw_error = validate_source_proof_occurrences(&source, &symbols, &scope).unwrap_err();
        let checker_error =
            Checker::check_source_theorem_equalities(&source, &symbols, &scope).unwrap_err();
        assert_eq!(
            source_proof_scope_maps(&source, &symbols, &scope).unwrap_err(),
            raw_error,
            "{control}"
        );
        assert_eq!(
            Semantics::source_theorem_binding_env(&source, &symbols, &scope).unwrap_err(),
            checker_error,
            "{control}"
        );
        if control == "combined precedence" {
            assert_eq!(checker_error, raw_error);
        }
        assert_eq!(
            validate_source_proof_occurrences(&source, &symbols, &scope).unwrap_err(),
            raw_error
        );
        assert_eq!(
            source_proof_scope_maps(&source, &symbols, &scope).unwrap_err(),
            raw_error
        );
        assert_eq!(
            Checker::check_source_theorem_equalities(&source, &symbols, &scope).unwrap_err(),
            checker_error
        );
        assert_eq!(
            Semantics::source_theorem_binding_env(&source, &symbols, &scope).unwrap_err(),
            checker_error
        );
        assert_eq!(before, (source, symbols, scope));
    }
}
