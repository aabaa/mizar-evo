use super::*;

#[test]
fn source_theorem_typed_inference_retains_real_site_types_and_consumers() -> Result<(), String> {
    use mizar_checker::{type_checker::*, typed_ast::*};
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
        "theorem Different: for A, B being set holds A = B proof let L, R be set; thus L = R; end;".into(),
    ];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/typed_inference.miz");
    let ids = InMemorySessionIdAllocator::new();
    for (case, text) in sources.iter().enumerate() {
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
                        package_id: PackageId::new("typed-inference-test"),
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
            let original =
                TermFormulaChecker::check_source_theorem_equalities(&source, symbols, &scope)?;
            let (bindings, inference, typed) =
                TermFormulaChecker::check_source_theorem_equalities_with_typed_tables(
                    &source, symbols, &scope,
                )?;
            assert_eq!(bindings, original.0);
            assert_eq!(inference, original.1);
            assert_eq!(
                bindings,
                SourceVariableSemanticsChecker::occurrence_binding_env(&scope)
            );
            let base = &original.2;
            assert_eq!(typed.source_id(), base.source_id());
            assert_eq!(typed.module_id(), base.module_id());
            assert_eq!(typed.resolved_root(), base.resolved_root());
            assert_eq!(typed.nodes().root(), base.nodes().root());
            assert_eq!(typed.nodes().len(), base.nodes().len());
            assert_eq!(typed.contexts(), base.contexts());
            assert_eq!(typed.types(), inference.type_entries());
            assert_eq!(typed.facts(), inference.facts());
            assert_eq!(typed.diagnostics(), inference.diagnostics());
            assert_eq!(typed.coercions(), base.coercions());
            assert_eq!(typed.initial_obligations(), base.initial_obligations());
            assert_eq!(typed.source_context(), base.source_context());
            assert_eq!(typed.source_type(), base.source_type());
            assert_eq!(typed.source_attribute(), base.source_attribute());
            assert_eq!(typed.source_term(), base.source_term());
            assert_eq!(typed.source_atomic_formula(), base.source_atomic_formula());
            assert!(
                typed.contexts().is_empty()
                    && typed.facts().is_empty()
                    && typed.coercions().is_empty()
                    && typed.initial_obligations().is_empty()
                    && typed.diagnostics().is_empty()
            );
            for (id, node) in base.nodes().iter() {
                let mut expected = node.clone();
                if let Some((_, term)) = inference
                    .terms()
                    .iter()
                    .find(|(_, term)| term.site.node() == id)
                {
                    assert_eq!(term.status, TermStatus::Inferred);
                    expected.typing = TypingState::Successful;
                    expected.links.type_entry = Some(term.type_entry);
                }
                if inference.formulas().iter().any(|(_, formula)| {
                    formula.site.node() == id && formula.status == FormulaStatus::Checked
                }) {
                    expected.typing = TypingState::Successful;
                    assert!(expected.links.type_entry.is_none());
                }
                assert_eq!(typed.nodes().node(id), Some(&expected));
            }
            let primary = typed.source_term().unwrap();
            assert_eq!(inference.terms().len(), scope.references().len());
            assert_eq!(primary.terms().len(), scope.references().len());
            assert_eq!(typed.types().len(), inference.terms().len() * 2);
            for ((id, term), occurrence) in primary.terms().iter().zip(scope.references()) {
                assert_eq!(term.site().node().index(), occurrence.node().index());
                assert_eq!(term.source_range(), occurrence.range());
                let reference = primary
                    .references()
                    .iter()
                    .find(|(_, reference)| reference.term() == id)
                    .unwrap()
                    .1;
                assert_eq!(reference.binding().index(), occurrence.binding().index());
                assert_eq!(reference.lexical_scope(), Some(occurrence.scope()));
                let checked = inference
                    .terms()
                    .iter()
                    .find(|(_, checked)| &checked.site == term.site())
                    .unwrap()
                    .1;
                assert_eq!(
                    checked.reference,
                    Some(TermReference::Binding(reference.binding()))
                );
                let node = typed.nodes().node(checked.site.node()).unwrap();
                assert_eq!(node.links.type_entry, Some(checked.type_entry));
                assert_eq!(node.typing, TypingState::Successful);
                assert!(node.links.context.is_none());
                let entry = typed.types().get(checked.type_entry).unwrap();
                assert_eq!(
                    Some(entry),
                    original.1.type_entries().get(checked.type_entry)
                );
                assert_eq!(entry.owner, checked.site);
                assert_eq!(entry.status, TypeStatus::Known);
                let TypeEntryActual::Known(actual) = entry.actual else {
                    panic!("missing normalized result type")
                };
                let normalized = inference.normalized_types().get(actual).unwrap();
                assert_eq!(normalized.head, TypeHeadRef::BuiltinSet);
                assert_eq!(normalized.status, NormalizedTypeStatus::Known);
                assert_eq!(normalized.source.spelling, "set");
                assert!(
                    normalized.args.is_empty()
                        && normalized.attributes.positive().is_empty()
                        && normalized.attributes.negative().is_empty()
                );
                let binding = &scope.bindings()[occurrence.binding().index()];
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
                    .find(|view| matches!(view.kind(), K::TypeExpression))
                    .unwrap();
                let role = TypedSiteRef::Role {
                    node: TypedNodeId::new(written.id().index()),
                    role: TypeRole::new(format!("theorem.equality.use:{}", id.index())),
                };
                let rows = typed
                    .types()
                    .iter()
                    .filter(|(_, row)| row.owner == role)
                    .collect::<Vec<_>>();
                let [(role_id, declared)] = rows.as_slice() else {
                    panic!("missing unique written type entry")
                };
                assert_ne!(*role_id, checked.type_entry);
                assert_eq!(Some(*declared), original.1.type_entries().get(*role_id));
                assert_eq!(declared.actual, entry.actual);
                assert_eq!(declared.status, TypeStatus::Known);
                assert_eq!(&text[written.range().start..written.range().end], "set");
            }
            for (_, formula) in inference.formulas().iter() {
                assert_eq!(formula.status, FormulaStatus::Checked);
                assert_eq!(formula.kind, FormulaKind::Equality);
                let node = typed.nodes().node(formula.site.node()).unwrap();
                assert_eq!(node.typing, TypingState::Successful);
                assert!(node.links.type_entry.is_none() && node.links.context.is_none());
                assert!(
                    formula
                        .terms
                        .iter()
                        .all(|site| inference.terms().iter().any(|(_, term)| &term.site == site))
                );
            }
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
            let old_check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                &source, base, &scope, symbols, &labels, &resolved, None,
            );
            let new_check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                &source, &typed, &scope, symbols, &labels, &resolved, None,
            );
            if case < 2 {
                let old_check = old_check?;
                let new_check = new_check?;
                assert!(std::ptr::eq(new_check.typed_ast(), &typed));
                assert!(std::ptr::eq(old_check.typed_ast(), base));
                assert_eq!(new_check.owners(), old_check.owners());
                assert_eq!(new_check.scope(), old_check.scope());
                assert_eq!(new_check.labels(), old_check.labels());
                assert_eq!(new_check.resolved_labels(), old_check.resolved_labels());
                let expected = lower_source_theorem_skeletons(&old_check, None)?;
                let core = lower_source_theorem_skeletons(&new_check, None)?;
                assert_eq!(core, expected);
                assert_eq!(core.source_id(), source.source_id());
                assert_eq!(core.module_id(), source.module());
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
                assert_eq!(labels.references().len(), 1);
                assert_eq!(core, lower_source_theorem_skeletons(&new_check, None)?);
                assert_eq!(vcs, generate_core_vcs(&core, snapshot_id(777))?);
            } else {
                assert_eq!(new_check.unwrap_err(), old_check.unwrap_err());
                assert!(inference.formulas().iter().any(|(_, formula)| {
                    let references = formula
                        .terms
                        .iter()
                        .map(|site| {
                            inference
                                .terms()
                                .iter()
                                .find(|(_, term)| &term.site == site)
                                .unwrap()
                                .1
                                .reference
                                .clone()
                        })
                        .collect::<Vec<_>>();
                    references[0] != references[1]
                }));
                if case < 5 {
                    assert!(scope.bindings().iter().any(|binding| {
                        !scope
                            .references()
                            .iter()
                            .any(|reference| reference.binding() == binding.id())
                    }));
                }
            }
            let original_replay =
                TermFormulaChecker::check_source_theorem_equalities(&source, symbols, &scope)?;
            assert_eq!(original_replay, original);
            let replay = TermFormulaChecker::check_source_theorem_equalities_with_typed_tables(
                &source, symbols, &scope,
            )?;
            assert_eq!(replay, (bindings, inference, typed));
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
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_typed_inference_preserves_upstream_errors() {
    use mizar_checker::type_checker::TermFormulaChecker;
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::{NamespacePath, SymbolEnv, SymbolEnvIndexes},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::{SignatureProjectionExtractor, validate_source_proof_occurrences},
    };
    use mizar_session::{ModulePath, PackageId};
    use mizar_syntax::{SurfaceAstBuilder, SurfaceNodeKind as K};
    let text = "theorem T: for X being set, Unused being set holds X = X proof let X be set; thus X = X; end;";
    let ast = parse(text);
    let module = ModuleId::new(
        PackageId::new("typed-inference-test"),
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
    assert!(TermFormulaChecker::check_source_theorem_equalities(&source, symbols, &scope).is_ok());
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
    let unsupported = parse(&text.replace("set", "object"));
    let unsupported_source = SurfaceResolvedArena::lower(&unsupported, &module).unwrap();
    let shells = DeclarationShellCollector::new(&unsupported, &module).collect();
    let collected = SignatureProjectionExtractor::new(
        &unsupported,
        &shells,
        NamespacePath::new(module.path().as_str()),
    )
    .collect(&module);
    let unsupported_scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&unsupported, &module, collected.env()),
    )
    .unwrap();
    assert_eq!(
        validate_source_proof_occurrences(&unsupported_source, collected.env(), &unsupported_scope),
        Ok(())
    );
    controls.push((
        "unsupported type",
        unsupported_source.clone(),
        collected.env().clone(),
        unsupported_scope.clone(),
    ));
    controls.push(("combined precedence", unsupported_source, empty, scope));
    for (control, source, symbols, scope) in controls {
        let before = (source.clone(), symbols.clone(), scope.clone());
        let expected =
            TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope)
                .unwrap_err();
        if control == "combined precedence" {
            assert_eq!(
                expected,
                validate_source_proof_occurrences(&source, &symbols, &scope).unwrap_err()
            );
        }
        assert_eq!(
            TermFormulaChecker::check_source_theorem_equalities_with_typed_tables(
                &source, &symbols, &scope
            )
            .unwrap_err(),
            expected,
            "{control}"
        );
        let expected_replay =
            TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope)
                .unwrap_err();
        assert_eq!(
            TermFormulaChecker::check_source_theorem_equalities_with_typed_tables(
                &source, &symbols, &scope
            )
            .unwrap_err(),
            expected_replay,
            "{control}"
        );
        assert_eq!(before, (source, symbols, scope));
    }
}

#[test]
fn source_theorem_typed_declarations_retain_real_types() -> Result<(), String> {
    use mizar_checker::{
        binding_env::BinderIdentity,
        type_checker::{
            AttributeSet, DeclarationChecker as Checker, NormalizedTypeStatus, TypeHeadRef,
        },
        typed_ast::{TypeEntryActual, TypedSiteRef, TypingState},
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
        "theorem Canonical: for Unused being set, Used being set holds Used = Used proof end;".into(),
        "theorem Empty: for A being set holds A = A proof end;".into(),
    ];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/theorem_typed_declarations.miz");
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
                    snapshot: snapshot_id(782),
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
            let (baseline, checked) =
                Checker::check_source_theorem_equalities(&source, symbols, &scope)?;
            let typed = &baseline.2;
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
                let (binding, _) = matching[0];
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
                let site = TypedSiteRef::Node(typed_for(original.node()));
                let logical_type = TypedSiteRef::Role {
                    node: typed_for(written.id()),
                    role: format!("theorem.declaration.type:{}", binding.index()).into(),
                };
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
                assert!(expected.insert(binding, site).is_none());
            }
            assert_eq!(expected.len(), baseline.0.bindings().len());
            assert_eq!(
                raw_roots.len(),
                typed.source_type().unwrap().expressions().len()
            );
            if text.contains("Spare") {
                assert!(raw_roots.values().any(|(_, roles)| roles.len() == 3));
            }
            assert_eq!(checked.declarations().len(), expected.len());
            let checked_sites = checked
                .declarations()
                .iter()
                .map(|(_, row)| (row.binding, row.site.clone()))
                .collect::<BTreeMap<_, _>>();
            assert_eq!(checked_sites, expected);
            if text.contains("theorem Canonical:") {
                let declaration_source =
                    &checked.normalized_types().iter().next().unwrap().1.source;
                let inference_source = &baseline
                    .1
                    .normalized_types()
                    .iter()
                    .next()
                    .unwrap()
                    .1
                    .source;
                assert_ne!(declaration_source, inference_source);
                assert!(declaration_source.range.start < inference_source.range.start);
            }
            let result = Checker::check_source_theorem_equalities_with_typed_declarations(
                &source, symbols, &scope,
            )?;
            assert_eq!(result.0, baseline.0);
            assert_eq!(result.1, baseline.1);
            assert_eq!(result.3, baseline.3);
            let actual = &result.2;
            let old_len = typed.types().len();
            assert_eq!(actual.types().len(), old_len + checked.type_entries().len());
            assert_eq!(
                actual.types().iter().take(old_len).collect::<Vec<_>>(),
                typed.types().iter().collect::<Vec<_>>()
            );
            let appended = actual.types().iter().skip(old_len).collect::<Vec<_>>();
            let mut inserted_ids = BTreeMap::new();
            let mut covered = BTreeSet::new();
            for (old_id, entry) in checked.type_entries().iter() {
                let matching = appended
                    .iter()
                    .filter(|(_, row)| row.owner == entry.owner)
                    .collect::<Vec<_>>();
                assert_eq!(matching.len(), 1);
                let (inserted, row) = *matching[0];
                assert_eq!(row.id, inserted);
                assert!(covered.insert(inserted));
                assert!(inserted_ids.insert(old_id, inserted).is_none());
                assert_eq!(row.owner, entry.owner);
                assert_eq!(row.status, entry.status);
                assert_eq!(row.provenance, entry.provenance);
                assert_eq!(row.expected.is_some(), entry.expected.is_some());
                let TypeEntryActual::Known(old_actual) = entry.actual else {
                    panic!("missing genuine type")
                };
                let TypeEntryActual::Known(new_actual) = row.actual else {
                    panic!("missing installed type")
                };
                let references = std::iter::once((old_actual, new_actual))
                    .chain(entry.expected.zip(row.expected));
                for (old_reference, new_reference) in references {
                    let from = checked.normalized_types().get(old_reference).unwrap();
                    let to = result.1.normalized_types().get(new_reference).unwrap();
                    assert_eq!(from.head, TypeHeadRef::BuiltinSet);
                    assert_eq!(to.head, TypeHeadRef::BuiltinSet);
                    assert_eq!(from.status, NormalizedTypeStatus::Known);
                    assert_eq!(to.status, NormalizedTypeStatus::Known);
                    assert!(from.args.is_empty());
                    assert!(to.args.is_empty());
                    assert_eq!(from.attributes, AttributeSet::empty());
                    assert_eq!(to.attributes, AttributeSet::empty());
                    let targets = result
                        .1
                        .normalized_types()
                        .iter()
                        .filter(|(_, candidate)| {
                            candidate.head == TypeHeadRef::BuiltinSet
                                && candidate.status == NormalizedTypeStatus::Known
                                && candidate.args.is_empty()
                                && candidate.attributes == AttributeSet::empty()
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(targets.len(), 1);
                    assert_eq!(targets[0].0, new_reference);
                }
            }
            assert_eq!(covered, appended.iter().map(|(id, _)| *id).collect());
            let mut declaration_links = BTreeMap::new();
            for (_, declaration) in checked.declarations().iter() {
                let site = &expected[&declaration.binding];
                let inserted = inserted_ids[&declaration.type_entry.unwrap()];
                assert!(declaration_links.insert(site.node(), inserted).is_none());
                let node = actual.nodes().node(site.node()).unwrap();
                assert_eq!(node.typing, TypingState::Successful);
                assert_eq!(node.links.type_entry, Some(inserted));
                assert_eq!(actual.types().get(inserted).unwrap().owner, *site);
            }
            assert_eq!(actual.nodes().root(), typed.nodes().root());
            assert_eq!(actual.nodes().len(), typed.nodes().len());
            for (id, original) in typed.nodes().iter() {
                let mut expected_node = original.clone();
                if let Some(entry) = declaration_links.get(&id) {
                    expected_node.typing = TypingState::Successful;
                    expected_node.links.type_entry = Some(*entry);
                }
                assert_eq!(actual.nodes().node(id), Some(&expected_node));
            }
            assert_eq!(actual.source_id(), typed.source_id());
            assert_eq!(actual.module_id(), typed.module_id());
            assert_eq!(actual.resolved_root(), typed.resolved_root());
            assert_eq!(actual.contexts(), typed.contexts());
            assert_eq!(
                actual.contexts(),
                actual.source_context().unwrap().local_contexts()
            );
            assert_eq!(actual.facts(), typed.facts());
            assert_eq!(actual.coercions(), typed.coercions());
            assert_eq!(actual.initial_obligations(), typed.initial_obligations());
            assert_eq!(actual.diagnostics(), typed.diagnostics());
            assert_eq!(actual.source_context(), typed.source_context());
            assert_eq!(actual.source_type(), typed.source_type());
            assert_eq!(actual.source_attribute(), typed.source_attribute());
            assert_eq!(actual.source_evidence(), typed.source_evidence());
            assert_eq!(actual.source_term(), typed.source_term());
            assert_eq!(actual.source_template(), typed.source_template());
            assert_eq!(actual.source_application(), typed.source_application());
            assert_eq!(actual.source_structure(), typed.source_structure());
            assert_eq!(actual.source_set_term(), typed.source_set_term());
            assert_eq!(
                actual.source_atomic_formula(),
                typed.source_atomic_formula()
            );
            assert_eq!(
                actual.source_attribute_definition(),
                typed.source_attribute_definition()
            );
            assert_eq!(
                actual.source_predicate_definition(),
                typed.source_predicate_definition()
            );
            assert_eq!(
                actual.source_functor_definition(),
                typed.source_functor_definition()
            );
            assert_eq!(
                actual.source_mode_definition(),
                typed.source_mode_definition()
            );
            assert_eq!(
                actual.source_structure_definition(),
                typed.source_structure_definition()
            );
            assert_eq!(
                actual.source_composite_formula(),
                typed.source_composite_formula()
            );
            assert_eq!(
                actual.source_formula_composition(),
                typed.source_formula_composition()
            );
            assert_eq!(actual.source_statement(), typed.source_statement());
            assert_eq!(
                actual.source_statement_references(),
                typed.source_statement_references()
            );
            assert_eq!(
                actual.source_statement_witnesses(),
                typed.source_statement_witnesses()
            );
            assert_eq!(
                actual.source_proof_local_let_type(),
                typed.source_proof_local_let_type()
            );
            assert_eq!(
                actual.source_proof_local_given_type(),
                typed.source_proof_local_given_type()
            );
            assert_eq!(
                actual.source_property_implementation(),
                typed.source_property_implementation()
            );
            assert!(
                actual.source_nested_fraenkel_capture_identity()
                    == typed.source_nested_fraenkel_capture_identity()
            );
            assert_eq!(
                actual.source_condition_formula_composition(),
                typed.source_condition_formula_composition()
            );
            assert_eq!(
                actual.source_predicate_chain_composition(),
                typed.source_predicate_chain_composition()
            );
            assert_eq!(
                actual.source_proof_local_declaration(),
                typed.source_proof_local_declaration()
            );
            assert_eq!(
                actual.source_proof_local_let_binding(),
                typed.source_proof_local_let_binding()
            );
            assert_eq!(
                actual.source_proof_local_given_binding(),
                typed.source_proof_local_given_binding()
            );
            assert_eq!(
                actual.source_proof_local_given_use_type(),
                typed.source_proof_local_given_use_type()
            );
            assert_eq!(
                actual.source_proof_local_given_use_term(),
                typed.source_proof_local_given_use_term()
            );
            assert_eq!(
                actual.source_proof_local_given_condition_binding(),
                typed.source_proof_local_given_condition_binding()
            );
            assert_eq!(
                actual.source_proof_local_given_condition_type(),
                typed.source_proof_local_given_condition_type()
            );
            assert_eq!(
                actual.source_proof_local_given_condition_use_term(),
                typed.source_proof_local_given_condition_use_term()
            );
            assert_eq!(
                actual.source_proof_local_given_descendant_binding(),
                typed.source_proof_local_given_descendant_binding()
            );
            assert_eq!(
                actual.source_proof_local_given_descendant_type(),
                typed.source_proof_local_given_descendant_type()
            );
            assert_eq!(
                actual.source_proof_local_given_descendant_use_term(),
                typed.source_proof_local_given_descendant_use_term()
            );
            assert_eq!(
                result,
                Checker::check_source_theorem_equalities_with_typed_declarations(
                    &source, symbols, &scope
                )?
            );
            assert_eq!(
                (baseline, checked),
                Checker::check_source_theorem_equalities(&source, symbols, &scope)?
            );
            assert_eq!((source.clone(), symbols.clone(), scope.clone()), before);
            assert_eq!(ast.snapshot_text(), ast_before);
            assert_eq!(frontend.source, capture_before);
        }
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_typed_declarations_preserve_upstream_errors() {
    use mizar_checker::type_checker::DeclarationChecker as Checker;
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
    let path = root.join("src/theorem_typed_declaration_errors.miz");
    fs::write(&path, text).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
        super::super::super::ParseOnlyImportProvider,
        MizarParserSeam,
    )
    .run(
        SourceUnitRequest {
            snapshot: snapshot_id(783),
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
    assert!(Checker::check_source_theorem_equalities(&source, symbols, &scope).is_ok());
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
            ids.next_source_id(snapshot_id(783)).unwrap()
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
        let expected =
            Checker::check_source_theorem_equalities(&source, &symbols, &scope).unwrap_err();
        assert_eq!(
            Checker::check_source_theorem_equalities_with_typed_declarations(
                &source, &symbols, &scope
            )
            .unwrap_err(),
            expected
        );
        assert_eq!(
            Checker::check_source_theorem_equalities_with_typed_declarations(
                &source, &symbols, &scope
            )
            .unwrap_err(),
            expected
        );
        assert_eq!((source, symbols, scope), before);
    }
    assert_eq!(frontend.source, capture_before);
    assert_eq!(ast.snapshot_text(), ast_before);
    fs::remove_dir_all(root).unwrap();
}
