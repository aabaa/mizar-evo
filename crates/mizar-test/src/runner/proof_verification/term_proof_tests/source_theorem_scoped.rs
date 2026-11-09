use super::*;

#[test]
fn source_theorem_scoped_primary_retain_occurrences_and_lookup() -> Result<(), String> {
    use mizar_checker::{
        binding_env::*,
        source_term::{SourcePrimaryTermError as Error, SourcePrimaryTermProducer as Primary, *},
        type_checker::{
            SourceVariableSemanticsChecker as Semantics, TermFormulaChecker as Checker,
        },
        typed_ast::{TypedAst, TypedAstParts},
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
        symbols::{SignatureProjectionExtractor, source_proof_scope_maps},
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceInput,
        SourceOriginInput,
    };
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
    let path = root.join("src/scoped_primary.miz");
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
                        package_id: PackageId::new("scoped-primary-test"),
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
            let original = Checker::check_source_theorem_equalities(&source, symbols, &scope)?;
            let env = Semantics::source_theorem_binding_env(&source, symbols, &scope)?;
            let maps = source_proof_scope_maps(&source, symbols, &scope)?;
            let typed = &original.2;
            let old = typed.source_term().unwrap();
            let arena = typed.nodes();
            let build = Primary::build_from_scoped_theorem_occurrences;
            let before = (source.clone(), symbols.clone(), scope.clone(), env.clone());
            let ast_before = ast.snapshot_text();
            let capture_before = frontend.source.clone();
            let input = SourcePrimaryTermHandoffInput {
                source_id: old.source_id(),
                module_id: old.module_id().clone(),
                terms: old
                    .terms()
                    .iter()
                    .zip(scope.references())
                    .map(|((_, term), actual)| {
                        let context = env
                            .contexts()
                            .iter()
                            .find(|(_, row)| row.lexical_scope.as_ref() == Some(actual.scope()))
                            .unwrap()
                            .0;
                        SourcePrimaryTermInput {
                            site: term.site().clone(),
                            source_range: term.source_range(),
                            source_ordinal: term.source_ordinal(),
                            context,
                            recovery: term.recovery(),
                            spelling: term.spelling().into(),
                            kind: term.kind(),
                            role: term.role(),
                            parent: term.parent(),
                        }
                    })
                    .collect(),
                references: old
                    .references()
                    .iter()
                    .map(|(_, row)| SourcePrimaryTermReferenceInput {
                        term: row.term(),
                        binding: row.binding(),
                        role: row.role(),
                    })
                    .collect(),
                numeric_type_requests: Vec::new(),
            };
            let input_before = input.clone();
            let generic = Primary::build(input.clone(), &env, arena).map_err(|e| e.to_string())?;
            let output = build(input.clone(), &env, arena, &source, symbols, &scope)
                .map_err(|e| e.to_string())?;
            assert_eq!(output.source_id(), source.source_id());
            assert_eq!(output.module_id(), source.module());
            assert_eq!(output.references(), old.references());
            assert_eq!(output.numeric_type_requests(), old.numeric_type_requests());
            assert!(output.numeric_type_requests().is_empty());
            assert_eq!(output.terms().len(), old.terms().len());
            assert_eq!(output.terms(), generic.terms());
            assert_eq!(output.references().len(), scope.references().len());
            let mut matched = std::collections::BTreeSet::new();
            for ((id, term), (reference_id, row)) in
                output.terms().iter().zip(output.references().iter())
            {
                assert_eq!(row.term(), id);
                let node = arena.node(term.site().node()).unwrap();
                let actual = scope
                    .references()
                    .iter()
                    .find(|reference| {
                        source.resolved_node_for(reference.node()) == node.resolved_node
                    })
                    .unwrap();
                assert!(matched.insert(actual.id()));
                assert_eq!(maps.0.get(&actual.node()), Some(actual.scope()));
                assert_eq!(term.source_range(), actual.range());
                assert_eq!(term.spelling(), actual.spelling());
                let declaration = &scope.bindings()[actual.binding().index()];
                assert_eq!(
                    env.bindings().get(row.binding()).unwrap().identity,
                    BinderIdentity::ResolverLocal {
                        scope: declaration.scope().clone(),
                        ordinal: declaration.ordinal(),
                        declaration_range: declaration.range(),
                    }
                );
                assert_eq!(row.lexical_scope(), Some(actual.scope()));
                assert_eq!(row.use_ordinal(), actual.ordinal());
                let declaration_clock = env
                    .bindings()
                    .iter()
                    .filter(|(_, binding)| {
                        binding.declaration_range.end <= term.source_range().start
                    })
                    .count();
                let generic_row = generic.references().get(reference_id).unwrap();
                assert_eq!(generic_row.use_ordinal(), declaration_clock);
                different_clocks |= declaration_clock != row.use_ordinal();
                let context = env.contexts().get(term.context()).unwrap();
                assert_eq!(context.lexical_scope.as_ref(), Some(actual.scope()));
                assert_eq!(
                    env.lookup(&BindingLookupSite::new(
                        term.spelling(),
                        term.context(),
                        context.lexical_scope.clone(),
                        declaration_clock
                    ))
                    .unwrap(),
                    BindingLookupResult::Local(row.binding())
                );
                let binding = env.bindings().get(row.binding()).unwrap();
                let mut ancestor = Some(term.context());
                while ancestor != Some(binding.owner_context) {
                    ancestor = env.contexts().get(ancestor.unwrap()).unwrap().parent;
                    assert!(ancestor.is_some());
                }
            }
            assert_eq!(matched.len(), scope.references().len());
            assert!(
                case != 6
                    || env
                        .contexts()
                        .iter()
                        .any(|(_, context)| context.layer == BindingContextLayer::Proof
                            && context.bindings.is_empty())
            );
            assert_eq!(
                Primary::build_from_occurrences(input.clone(), &env, arena, &scope),
                Err(Error::InvalidTransaction)
            );
            let installed = TypedAst::try_new(TypedAstParts {
                source_id: typed.source_id(),
                module_id: typed.module_id().clone(),
                resolved_root: typed.resolved_root(),
                source_context: typed.source_context().cloned(),
                source_type: typed.source_type().cloned(),
                source_attribute: typed.source_attribute().cloned(),
                nodes: arena.clone(),
                contexts: typed.contexts().clone(),
                types: typed.types().clone(),
                facts: typed.facts().clone(),
                coercions: typed.coercions().clone(),
                initial_obligations: typed.initial_obligations().clone(),
                diagnostics: typed.diagnostics().clone(),
            })
            .and_then(|ast| ast.with_source_term(output.clone()))
            .map_err(|e| e.to_string())?;
            assert_eq!(installed.source_term(), Some(&output));
            assert_eq!(installed.nodes(), arena);
            let replay = build(input.clone(), &env, arena, &source, symbols, &scope)
                .map_err(|e| e.to_string())?;
            assert_eq!(replay, output);
            assert_eq!(
                Checker::check_source_theorem_equalities(&source, symbols, &scope)?,
                original
            );
            let signature = (output.debug_text(), env.debug_text());
            if let Some((source_id, previous)) = &disk_capture {
                assert_ne!(*source_id, output.source_id());
                assert_eq!(previous, &signature);
            } else {
                disk_capture = Some((output.source_id(), signature));
            }
            assert_eq!(input, input_before);
            assert_eq!(before, (source, symbols.clone(), scope, env));
            assert_eq!(ast_before, ast.snapshot_text());
            assert_eq!(capture_before, frontend.source);
        }
    }
    assert!(different_clocks);
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_scoped_primary_reject_mismatched_inputs() -> Result<(), String> {
    use mizar_checker::{
        binding_env::*,
        source_term::{
            SourcePrimaryTermError as Error, SourcePrimaryTermProducer as Primary,
            SourcePrimaryTermReferenceId as RefId, *,
        },
        type_checker::{
            SourceVariableSemanticsChecker as Semantics, TermFormulaChecker as Checker,
        },
        typed_ast::{NodeRecoveryState, TypedArena},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::{NamespacePath, SymbolEnv, SymbolEnvIndexes},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::SignatureProjectionExtractor,
    };
    use mizar_session::{ModulePath, PackageId};
    let ast = parse(
        "theorem Nested: for A being set for Unused being set holds for B being set holds A = B proof let L be set; thus L = L; end; theorem Empty: for Z being set holds Z = Z proof end;",
    );
    let module = ModuleId::new(
        PackageId::new("scoped-primary-test"),
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
    assert!(collected.diagnostics().is_empty());
    let symbols = collected.env();
    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&ast, &module, symbols),
    )
    .unwrap();
    let original = Checker::check_source_theorem_equalities(&source, symbols, &scope)?;
    let env = Semantics::source_theorem_binding_env(&source, symbols, &scope)?;
    let arena = original.2.nodes();
    let old = original.2.source_term().unwrap();
    let input = SourcePrimaryTermHandoffInput {
        source_id: old.source_id(),
        module_id: old.module_id().clone(),
        terms: old
            .terms()
            .iter()
            .zip(scope.references())
            .map(|((_, term), actual)| SourcePrimaryTermInput {
                site: term.site().clone(),
                source_range: term.source_range(),
                source_ordinal: term.source_ordinal(),
                context: env
                    .contexts()
                    .iter()
                    .find(|(_, row)| row.lexical_scope.as_ref() == Some(actual.scope()))
                    .unwrap()
                    .0,
                recovery: term.recovery(),
                spelling: term.spelling().into(),
                kind: term.kind(),
                role: term.role(),
                parent: term.parent(),
            })
            .collect(),
        references: old
            .references()
            .iter()
            .map(|(_, row)| SourcePrimaryTermReferenceInput {
                term: row.term(),
                binding: row.binding(),
                role: row.role(),
            })
            .collect(),
        numeric_type_requests: Vec::new(),
    };
    let before = (source.clone(), symbols.clone(), scope.clone());
    let build = Primary::build_from_scoped_theorem_occurrences;
    let scoped = |input| build(input, &env, arena, &source, symbols, &scope);
    let generic = Primary::build;
    let invalid_reference = |index| Error::InvalidReference {
        reference: RefId::new(index),
    };
    assert!(scoped(input.clone()).is_ok());
    let foreign_module = ModuleId::new(PackageId::new("foreign"), ModulePath::new("other"));
    let foreign_source = SurfaceResolvedArena::lower(&ast, &foreign_module).unwrap();
    let foreign_symbols = SymbolEnv::new(foreign_module.clone(), SymbolEnvIndexes::default());
    let foreign_scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&ast, &foreign_module, &foreign_symbols),
    )
    .unwrap();
    let empty_symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
    for (source, symbols, scope, env) in [
        (&foreign_source, symbols, &scope, &env),
        (&source, &empty_symbols, &scope, &env),
        (&source, symbols, &foreign_scope, &env),
        (&source, symbols, &scope, &original.0),
    ] {
        assert_eq!(
            build(input.clone(), env, arena, source, symbols, scope),
            Err(Error::InvalidTransaction)
        );
    }
    let mut broken = input.clone();
    broken.terms[0].source_ordinal = usize::MAX;
    let generic_error = generic(broken.clone(), &env, arena).unwrap_err();
    assert_eq!(scoped(broken.clone()).unwrap_err(), generic_error);
    let combined = build(
        broken,
        &original.0,
        arena,
        &foreign_source,
        &empty_symbols,
        &foreign_scope,
    );
    assert_eq!(combined, Err(Error::InvalidTransaction));
    let unused = env
        .bindings()
        .iter()
        .find(|(_, entry)| entry.spelling == "Unused")
        .unwrap()
        .0;
    let unused_context = env
        .contexts()
        .iter()
        .find(|(_, row)| {
            row.layer == BindingContextLayer::Proof
                && row.bindings.is_empty()
                && row.visible_bindings.is_empty()
        })
        .unwrap()
        .0;
    for change_binding in [true, false] {
        let mut bindings = BindingTable::new();
        for (id, entry) in env.bindings().iter() {
            bindings.insert(BindingDraft {
                spelling: if change_binding && id == unused {
                    "Hidden".into()
                } else {
                    entry.spelling.clone()
                },
                kind: entry.kind,
                identity: entry.identity.clone(),
                owner_context: entry.owner_context,
                declaration_range: entry.declaration_range,
                visible_after_ordinal: entry.visible_after_ordinal,
                type_site: entry.type_site.clone(),
                status: entry.status,
                captured: entry.captured.clone(),
                diagnostics: entry.diagnostics.clone(),
                recovery: entry.recovery,
            });
        }
        let mut contexts = BindingContextTable::new();
        for (id, row) in env.contexts().iter() {
            contexts.insert(BindingContextDraft {
                owner: row.owner.clone(),
                parent: row.parent,
                layer: row.layer,
                lexical_scope: row.lexical_scope.clone(),
                bindings: row.bindings.clone(),
                visible_bindings: row.visible_bindings.clone(),
                recovery: if !change_binding && id == unused_context {
                    BindingContextRecovery::Degraded
                } else {
                    row.recovery
                },
            });
        }
        let altered = BindingEnv::try_new(BindingEnvParts {
            source_id: env.source_id(),
            module_id: env.module_id().clone(),
            contexts,
            bindings,
            diagnostics: env.diagnostics().clone(),
        })
        .unwrap();
        assert!(generic(input.clone(), &altered, arena).is_ok());
        assert_eq!(
            build(input.clone(), &altered, arena, &source, symbols, &scope),
            Err(Error::InvalidTransaction)
        );
    }
    let mut ancestor = input.clone();
    let index = scope
        .references()
        .iter()
        .position(|actual| {
            let declared = scope.bindings()[actual.binding().index()].scope();
            let local = actual.scope();
            local != declared && local.path().starts_with(declared.path())
        })
        .unwrap();
    let parent = env
        .bindings()
        .get(input.references[index].binding)
        .unwrap()
        .owner_context;
    let mut cursor = env
        .contexts()
        .get(ancestor.terms[index].context)
        .unwrap()
        .parent;
    while cursor != Some(parent) {
        cursor = env.contexts().get(cursor.unwrap()).unwrap().parent;
    }
    ancestor.terms[index].context = parent;
    assert!(generic(ancestor.clone(), &env, arena).is_ok());
    assert_eq!(scoped(ancestor), Err(invalid_reference(index)));
    let (term, first) = old.terms().iter().next().unwrap();
    let target = first.site().node();
    let other = input.terms[1].site.node();
    let wrong = arena.node(other).unwrap().resolved_node;
    assert_ne!(arena.node(target).unwrap().resolved_node, wrong);
    for degraded in [false, true] {
        let mut altered = input.clone();
        if degraded {
            altered.terms[term.index()].recovery = SourcePrimaryTermRecovery::Degraded;
        }
        let changed_arena = TypedArena::try_new(
            arena.root(),
            arena
                .iter()
                .map(|(id, node)| match (id == target, degraded) {
                    (true, true) => node.clone().with_recovery(NodeRecoveryState::Degraded),
                    (true, false) => node.clone().with_resolved_node(wrong.unwrap()),
                    _ => node.clone(),
                })
                .collect(),
        )
        .unwrap();
        assert!(generic(altered.clone(), &env, &changed_arena).is_ok());
        let expected = if degraded {
            Error::InvalidTerm { term }
        } else {
            invalid_reference(0)
        };
        assert_eq!(
            build(altered, &env, &changed_arena, &source, symbols, &scope),
            Err(expected)
        );
    }
    let mut incomplete = input.clone();
    incomplete.terms.pop();
    incomplete.references.pop();
    assert!(generic(incomplete.clone(), &env, arena).is_ok());
    assert_eq!(
        scoped(incomplete.clone()),
        Err(invalid_reference(incomplete.references.len()))
    );
    let mut broken = [input.clone(), input.clone(), input.clone()];
    broken[0].references[0].binding = input.references[1].binding;
    broken[1].terms[0].spelling = "Missing".into();
    broken[2].terms[0].site = input.terms[1].site.clone();
    for broken in broken {
        let expected = generic(broken.clone(), &env, arena).unwrap_err();
        assert_eq!(scoped(broken).unwrap_err(), expected);
    }
    assert_eq!(before, (source.clone(), symbols.clone(), scope.clone()));
    Ok(())
}

#[test]
fn source_theorem_scoped_inference_retains_real_types_and_contexts() -> Result<(), String> {
    use mizar_checker::{
        binding_env::{BinderIdentity, BindingContextLayer},
        type_checker::{
            FormulaStatus, SourceVariableSemanticsChecker as Semantics,
            TermFormulaChecker as Checker, TermReference, TermStatus, TypeHeadRef,
        },
        typed_ast::{
            BindingTypeRef, TypeContextLayer, TypeEntryActual, TypeRole, TypeStatus, TypedAst,
            TypedAstParts, TypedSiteRef,
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
        symbols::{SignatureProjectionExtractor, source_proof_scope_maps},
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceInput,
        SourceOriginInput,
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
    let path = root.join("src/scoped_inference.miz");
    let ids = InMemorySessionIdAllocator::new();
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
                        package_id: PackageId::new("scoped-inference-test"),
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
            let baseline = Checker::check_source_theorem_equalities_with_local_contexts(
                &source, symbols, &scope,
            )?;
            let before = (
                source.clone(),
                symbols.clone(),
                scope.clone(),
                ast.snapshot_text(),
                frontend.source.clone(),
            );
            let maps = source_proof_scope_maps(&source, symbols, &scope)?;
            let lexical = Semantics::source_theorem_binding_env(&source, symbols, &scope)?;
            let output = Checker::check_source_theorem_equalities_with_scoped_inference(
                &source, symbols, &scope,
            )?;
            let (env, inference, typed, expressions) = &output;
            let base = &baseline.2;
            assert_eq!(env, &lexical);
            assert_eq!(inference.source_id(), source.source_id());
            assert_eq!(inference.module_id(), source.module());
            assert_eq!(inference.normalized_types(), baseline.1.normalized_types());
            assert_eq!(inference.type_entries(), baseline.1.type_entries());
            assert_eq!(inference.candidate_sets(), baseline.1.candidate_sets());
            assert_eq!(inference.facts(), baseline.1.facts());
            assert_eq!(inference.diagnostics(), baseline.1.diagnostics());
            assert_eq!(typed.nodes().len(), base.nodes().len());
            assert_eq!(typed.source_context(), base.source_context());
            assert_eq!(typed.source_type(), base.source_type());
            assert_eq!(typed.source_id(), source.source_id());
            assert_eq!(typed.module_id(), source.module());
            assert_eq!(typed.resolved_root(), base.resolved_root());
            assert_eq!(typed.nodes().root(), base.nodes().root());
            assert_eq!(typed.contexts(), base.contexts());
            assert_eq!(typed.types(), inference.type_entries());
            assert_eq!(typed.facts(), inference.facts());
            assert_eq!(typed.diagnostics(), inference.diagnostics());
            assert_eq!(typed.coercions(), base.coercions());
            assert_eq!(typed.initial_obligations(), base.initial_obligations());
            assert_eq!(expressions, &baseline.3);
            let primary = typed.source_term().unwrap();
            let old_primary = base.source_term().unwrap();
            let atomic = typed.source_atomic_formula().unwrap();
            let old_atomic = base.source_atomic_formula().unwrap();
            assert_eq!(primary.references(), old_primary.references());
            assert_eq!(
                primary.numeric_type_requests(),
                old_primary.numeric_type_requests()
            );
            assert_eq!(typed.source_attribute(), base.source_attribute());
            assert_eq!(inference.terms().len(), primary.terms().len());
            assert_eq!(inference.formulas().len(), atomic.formulas().len());
            assert_eq!(atomic.primary_term_fingerprint(), primary.debug_text());
            assert_ne!(
                atomic.primary_term_fingerprint(),
                old_atomic.primary_term_fingerprint()
            );
            assert_eq!(atomic.edges(), old_atomic.edges());
            assert_eq!(atomic.requests(), old_atomic.requests());
            assert_eq!(atomic.wrappers(), old_atomic.wrappers());
            assert_eq!(atomic.predicate_segments(), old_atomic.predicate_segments());
            assert_eq!(atomic.predicate_heads(), old_atomic.predicate_heads());
            assert_eq!(atomic.candidates(), old_atomic.candidates());
            assert_eq!(atomic.type_sites(), old_atomic.type_sites());
            assert_eq!(atomic.attributes(), old_atomic.attributes());
            assert_eq!(
                atomic.application_fingerprint(),
                old_atomic.application_fingerprint()
            );
            assert_eq!(
                atomic.structure_fingerprint(),
                old_atomic.structure_fingerprint()
            );
            assert_eq!(
                atomic.set_term_fingerprint(),
                old_atomic.set_term_fingerprint()
            );
            let typed_for = |surface| {
                let resolved = source.resolved_node_for(surface).unwrap();
                let matches = typed
                    .nodes()
                    .iter()
                    .filter(|(_, row)| row.resolved_node == Some(resolved))
                    .collect::<Vec<_>>();
                assert_eq!(matches.len(), 1);
                matches[0].0
            };
            let mut local_ids = BTreeMap::new();
            let mut binding_ids = BTreeMap::new();
            for view in ast.node_views() {
                let parent = maps.1.get(&view.id());
                if parent.is_some_and(|parent| maps.0[parent] == maps.0[&view.id()]) {
                    continue;
                }
                let owner = TypedSiteRef::Node(typed_for(view.id()));
                let (local_id, local) = typed
                    .contexts()
                    .iter()
                    .find(|(_, row)| row.owner == owner)
                    .unwrap();
                let scope_here = &maps.0[&view.id()];
                let (binding_id, _) = env
                    .contexts()
                    .iter()
                    .find(|(_, row)| {
                        if parent.is_none() {
                            row.parent.is_none()
                        } else {
                            row.lexical_scope.as_ref() == Some(scope_here)
                        }
                    })
                    .unwrap();
                let expected_layer = match view.kind() {
                    K::Root => TypeContextLayer::Module,
                    K::QuantifiedFormula(_) => TypeContextLayer::Expression,
                    K::ProofBlock => TypeContextLayer::Proof,
                    _ => panic!("unexpected lexical boundary"),
                };
                assert_eq!(local.layer, expected_layer);
                local_ids.insert(scope_here.clone(), local_id);
                binding_ids.insert(scope_here.clone(), binding_id);
            }
            assert_eq!(local_ids.len(), typed.contexts().len());
            assert_eq!(binding_ids.len(), env.contexts().len());
            for view in ast.node_views() {
                let scope_here = &maps.0[&view.id()];
                let node = typed.nodes().node(typed_for(view.id())).unwrap();
                assert_eq!(node.links.context, Some(local_ids[scope_here]));
                if let Some(parent) = maps.1.get(&view.id()) {
                    let outer = &maps.0[parent];
                    if outer != scope_here {
                        assert_eq!(
                            typed.contexts().get(local_ids[scope_here]).unwrap().parent,
                            Some(local_ids[outer])
                        );
                        assert_eq!(
                            env.contexts().get(binding_ids[scope_here]).unwrap().parent,
                            Some(binding_ids[outer])
                        );
                    }
                } else {
                    assert!(
                        typed
                            .contexts()
                            .get(local_ids[scope_here])
                            .unwrap()
                            .parent
                            .is_none()
                    );
                }
            }
            for (scope_here, local_id) in &local_ids {
                let expected = scope
                    .bindings()
                    .iter()
                    .filter(|decl| decl.scope() == scope_here)
                    .map(|decl| BindingTypeRef::Site(TypedSiteRef::Node(typed_for(decl.node()))))
                    .collect::<Vec<_>>();
                assert_eq!(typed.contexts().get(*local_id).unwrap().bindings, expected);
            }
            let mut seen = BTreeSet::new();
            for actual in scope.references() {
                let site = TypedSiteRef::Node(typed_for(actual.node()));
                let (id, term) = primary
                    .terms()
                    .iter()
                    .find(|(_, term)| term.site() == &site)
                    .unwrap();
                assert!(seen.insert(id));
                let old = old_primary.terms().get(id).unwrap();
                assert_eq!(term.site(), old.site());
                assert_eq!(term.source_range(), old.source_range());
                assert_eq!(term.source_ordinal(), old.source_ordinal());
                assert_eq!(term.recovery(), old.recovery());
                assert_eq!(term.spelling(), old.spelling());
                assert_eq!(term.kind(), old.kind());
                assert_eq!(term.role(), old.role());
                assert_eq!(term.parent(), old.parent());
                assert_eq!(term.context(), binding_ids[actual.scope()]);
                assert_eq!(term.source_range(), actual.range());
                assert_eq!(term.spelling(), actual.spelling());
                let declaration = &scope.bindings()[actual.binding().index()];
                let reference = primary
                    .references()
                    .iter()
                    .find(|(_, row)| row.term() == id)
                    .unwrap()
                    .1;
                let binding = env.bindings().get(reference.binding()).unwrap();
                assert_eq!(
                    binding.identity,
                    BinderIdentity::ResolverLocal {
                        scope: declaration.scope().clone(),
                        ordinal: declaration.ordinal(),
                        declaration_range: declaration.range(),
                    }
                );
                assert_eq!(reference.lexical_scope(), Some(actual.scope()));
                assert_eq!(reference.use_ordinal(), actual.ordinal());
                let mut ancestor = Some(term.context());
                let mut reached = false;
                while let Some(id) = ancestor {
                    reached |= id == binding.owner_context;
                    ancestor = env.contexts().get(id).unwrap().parent;
                }
                assert!(reached);
                let decl = ast
                    .node_views()
                    .find(|view| {
                        view.children().contains(&declaration.node())
                            && matches!(
                                view.kind(),
                                K::QuantifierVariableSegment | K::QualifiedVariableSegment
                            )
                    })
                    .unwrap();
                let type_child = *decl.children().last().unwrap();
                assert_eq!(
                    ast.node_view(type_child).unwrap().kind(),
                    &K::TypeExpression
                );
                let role_site = TypedSiteRef::Role {
                    node: typed_for(type_child),
                    role: TypeRole::new(format!("theorem.equality.use:{}", old.source_ordinal())),
                };
                let (_, role_entry) = baseline
                    .1
                    .type_entries()
                    .iter()
                    .find(|(_, row)| row.owner == role_site)
                    .unwrap();
                let (_, real_role) = inference
                    .type_entries()
                    .iter()
                    .find(|(_, row)| row.owner == role_site)
                    .unwrap();
                assert_eq!(real_role, role_entry);
                assert_eq!(real_role.status, TypeStatus::Known);
                let TypeEntryActual::Known(normalized) = real_role.actual else {
                    panic!("missing real declared type")
                };
                assert_eq!(
                    inference.normalized_types().get(normalized).unwrap().head,
                    TypeHeadRef::BuiltinSet
                );
                let checked = inference
                    .terms()
                    .iter()
                    .find(|(_, row)| row.site == site)
                    .unwrap()
                    .1;
                let mut expected = baseline
                    .1
                    .terms()
                    .iter()
                    .find(|(_, row)| row.site == site)
                    .unwrap()
                    .1
                    .clone();
                expected.context = binding_ids[actual.scope()];
                assert_eq!(checked, &expected);
                assert_eq!(
                    checked.reference,
                    Some(TermReference::Binding(reference.binding()))
                );
                assert_eq!(checked.status, TermStatus::Inferred);
                let node = typed.nodes().node(site.node()).unwrap();
                assert_eq!(node.links.type_entry, Some(checked.type_entry));
                let entry = typed.types().get(checked.type_entry).unwrap();
                assert_eq!(entry.owner, site);
                assert_eq!(entry.actual, real_role.actual);
                assert_eq!(node.links.context, Some(local_ids[actual.scope()]));
            }
            assert_eq!(seen.len(), primary.terms().len());
            let mut seen_formulas = BTreeSet::new();
            for view in ast
                .node_views()
                .filter(|view| view.kind() == &K::BuiltinPredicateApplication)
            {
                let site = TypedSiteRef::Node(typed_for(view.id()));
                let (id, formula) = atomic
                    .formulas()
                    .iter()
                    .find(|(_, row)| row.site() == &site)
                    .unwrap();
                assert!(seen_formulas.insert(id));
                let old = old_atomic.formulas().get(id).unwrap();
                assert_eq!(formula.site(), old.site());
                assert_eq!(formula.source_range(), old.source_range());
                assert_eq!(formula.source_ordinal(), old.source_ordinal());
                assert_eq!(formula.recovery(), old.recovery());
                assert_eq!(formula.spelling(), old.spelling());
                assert_eq!(formula.kind(), old.kind());
                assert_eq!(formula.context(), binding_ids[&maps.0[&view.id()]]);
                let checked = inference
                    .formulas()
                    .iter()
                    .find(|(_, row)| row.site == site)
                    .unwrap()
                    .1;
                let mut expected = baseline
                    .1
                    .formulas()
                    .iter()
                    .find(|(_, row)| row.site == site)
                    .unwrap()
                    .1
                    .clone();
                expected.context = binding_ids[&maps.0[&view.id()]];
                assert_eq!(checked, &expected);
                assert_eq!(checked.status, FormulaStatus::Checked);
                let operands = [view.children()[0], view.children()[2]].map(|child| {
                    let reference = ast.node_view(child).unwrap().children()[0];
                    TypedSiteRef::Node(typed_for(reference))
                });
                assert_eq!(checked.terms, operands);
                for operand in &checked.terms {
                    assert_eq!(
                        primary
                            .terms()
                            .iter()
                            .find(|(_, row)| row.site() == operand)
                            .unwrap()
                            .1
                            .context(),
                        formula.context()
                    );
                }
            }
            assert_eq!(seen_formulas.len(), atomic.formulas().len());
            for (id, old) in base.nodes().iter() {
                let actual = typed.nodes().node(id).unwrap();
                let mut expected = old.clone();
                if let Some((_, term)) = inference
                    .terms()
                    .iter()
                    .find(|(_, row)| row.site.node() == id)
                {
                    expected.links.type_entry = Some(term.type_entry);
                }
                assert_eq!(actual, &expected);
                let resolved = source.arena().node(actual.resolved_node.unwrap()).unwrap();
                assert_eq!(actual.anchor, *resolved.origin().anchor());
                assert_eq!(resolved.origin().source_id(), source.source_id());
            }
            for expression in expressions {
                assert_eq!(
                    expression.local_context,
                    typed
                        .nodes()
                        .node(expression.typed_site.node())
                        .unwrap()
                        .links
                        .context
                );
            }
            if case == 6 {
                assert!(
                    env.contexts()
                        .iter()
                        .any(|(_, context)| context.layer == BindingContextLayer::Proof
                            && context.bindings.is_empty())
                );
            }
            let installed = TypedAst::try_new(TypedAstParts {
                source_id: typed.source_id(),
                module_id: typed.module_id().clone(),
                resolved_root: typed.resolved_root(),
                source_context: typed.source_context().cloned(),
                source_type: typed.source_type().cloned(),
                source_attribute: typed.source_attribute().cloned(),
                nodes: typed.nodes().clone(),
                contexts: typed.contexts().clone(),
                types: typed.types().clone(),
                facts: typed.facts().clone(),
                coercions: typed.coercions().clone(),
                initial_obligations: typed.initial_obligations().clone(),
                diagnostics: typed.diagnostics().clone(),
            })
            .and_then(|ast| ast.with_source_term(primary.clone()))
            .and_then(|ast| ast.with_source_atomic_formula(atomic.clone()))
            .map_err(|e| e.to_string())?;
            assert_eq!(&installed, typed);
            assert_eq!(
                Checker::check_source_theorem_equalities_with_local_contexts(
                    &source, symbols, &scope
                )?,
                baseline
            );
            assert_eq!(
                Checker::check_source_theorem_equalities_with_scoped_inference(
                    &source, symbols, &scope
                )?,
                output
            );
            let signature = (
                env.debug_text(),
                inference.debug_text(),
                typed.debug_text(),
                primary.debug_text(),
                atomic.debug_text(),
                format!("{expressions:?}"),
            );
            if let Some((source_id, previous)) = &disk_capture {
                assert_ne!(*source_id, source.source_id());
                assert_eq!(previous, &signature);
            } else {
                disk_capture = Some((source.source_id(), signature));
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
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_scoped_inference_preserves_upstream_errors() {
    use mizar_checker::type_checker::TermFormulaChecker as Checker;
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
        PackageId::new("local-type-context-test"),
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
    let unsupported = parse(&text.replacen("thus X = X;", "thus X = 0;", 1));
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
        Checker::check_source_theorem_equalities(
            &unsupported_source,
            collected.env(),
            &unsupported_scope
        )
        .unwrap_err(),
        "theorems.equality.unsupported_source_types"
    );
    controls.push((
        "unsupported type",
        unsupported_source.clone(),
        collected.env().clone(),
        unsupported_scope.clone(),
    ));
    {
        let text = "theorem T: for X being set holds X = X proof let X be set; now let Y be set; thus Y = Y; end; thus X = X; end;";
        let ast = parse(text);
        let source = SurfaceResolvedArena::lower(&ast, &module).unwrap();
        let shells = DeclarationShellCollector::new(&ast, &module).collect();
        let collected = SignatureProjectionExtractor::new(
            &ast,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        let wider_scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(&ast, &module, collected.env()),
        )
        .unwrap();
        controls.push(("wider source", source, collected.env().clone(), wider_scope));
    }
    controls.push(("combined precedence", unsupported_source, empty, scope));
    for (control, source, symbols, scope) in controls {
        let before = (source.clone(), symbols.clone(), scope.clone());
        let expected =
            Checker::check_source_theorem_equalities_with_local_contexts(&source, &symbols, &scope)
                .unwrap_err();
        if control == "combined precedence" {
            assert_eq!(
                expected,
                validate_source_proof_occurrences(&source, &symbols, &scope).unwrap_err()
            );
        }
        assert_eq!(
            Checker::check_source_theorem_equalities_with_scoped_inference(
                &source, &symbols, &scope
            )
            .unwrap_err(),
            expected,
            "{control}"
        );
        assert_eq!(
            Checker::check_source_theorem_equalities_with_scoped_inference(
                &source, &symbols, &scope
            )
            .unwrap_err(),
            expected,
            "{control}"
        );
        assert_eq!(before, (source, symbols, scope));
    }
}

#[test]
fn source_theorem_scoped_skeleton_intake_preserves_genuine_consumers() -> Result<(), String> {
    use mizar_checker::{
        source_context::SourceItemVisibility,
        type_checker::{SourceVariableSemanticsChecker as Checker, check_source_theorem_metadata},
        typed_ast::{LocalTypeContextId, TypedNodeId},
    };
    use mizar_core::{
        core_ir::{
            CoreCitation, CoreFormulaKind, CoreProofNodeKind, CoreProofStatus, CoreSourceAnchor,
        },
        elaborator::lower_source_theorem_skeletons as lower,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        declarations::{DeclarationShellCollector, DeclarationShellVisibilityState},
        env::{ExportStatus, NamespacePath, Visibility},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::{SignatureProjectionExtractor, source_proof_inputs},
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceAnchor,
        SourceInput, SourceOrigin, SourceOriginInput, hash_text,
    };
    use mizar_syntax::SurfaceNodeKind as K;
    let private =
        "private theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let fixture = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let texts = [
        fixture.to_owned(),
        fixture
            .replace("Lem1", "Seed")
            .replace("let X be set;", "let Local be set;")
            .replace("thus X = X", "thus Local = Local"),
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;".into(),
        include_str!("../../../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz").to_owned(),
        private.to_owned(),
        private.replacen("private", "public", 1),
        private.replace("set", "object"),
        private.replace("theorem T", "open theorem T"),
        private.replace("thus X = X;", "thus X = ;"),
    ];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/scoped_intake.miz");
    let ids = InMemorySessionIdAllocator::new();
    let snapshot = snapshot_id(786);
    for (case, text) in texts.iter().enumerate() {
        for buffer in [false, true] {
            fs::write(&path, if buffer { "stale disk content" } else { text }).unwrap();
            let frontend = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
                super::super::super::ParseOnlyImportProvider,
                MizarParserSeam,
            )
            .run(
                SourceUnitRequest {
                    snapshot,
                    input: SourceInput {
                        package_id: PackageId::new("scoped-intake"),
                        module_path: ModulePath::new("owners"),
                        normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                        edition: Edition::new("2026"),
                        origin: if buffer {
                            SourceOriginInput::OpenBuffer {
                                uri: format!("file://{}", path.display()),
                                expected_version: 4,
                                actual_version: 4,
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
            assert_eq!(
                frontend.diagnostics.is_empty(),
                case != 8,
                "{:?}",
                frontend.diagnostics
            );
            assert_eq!(frontend.source.source_text.as_ref(), text);
            assert_eq!(frontend.source.source_hash, hash_text(text));
            assert_eq!(
                matches!(
                    frontend.source.origin,
                    SourceOrigin::OpenBuffer { version: 4 }
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
            let symbols = collected.env();
            if case == 8 {
                assert!(source_proof_inputs(&source, symbols).is_err());
                continue;
            }
            assert!(collected.diagnostics().is_empty());
            let (scope, labels, resolved) = source_proof_inputs(&source, symbols)?;
            let before = (
                source.clone(),
                symbols.clone(),
                scope.clone(),
                labels.clone(),
                resolved.clone(),
                ast.snapshot_text(),
                frontend.source.clone(),
            );
            if case >= 7 {
                let error = check_source_theorem_metadata(&source, symbols, &scope).unwrap_err();
                assert_eq!(
                    check_source_theorem_metadata(&source, symbols, &scope).unwrap_err(),
                    error
                );
                assert_eq!(
                    (
                        source.clone(),
                        symbols.clone(),
                        scope,
                        labels,
                        resolved,
                        ast.snapshot_text(),
                        frontend.source.clone()
                    ),
                    before
                );
                continue;
            }
            let output = check_source_theorem_metadata(&source, symbols, &scope)?;
            let (_, inference, typed, metadata) = &output;
            assert!(typed.source_context().is_some());
            assert!(!inference.terms().is_empty());
            assert!(!metadata.expr_metadata().is_empty());
            let output_before = output.clone();
            let expected = if case < 3 {
                Some(with_source_theorem_check(
                    ast,
                    &source,
                    symbols,
                    |legacy| {
                        assert!(legacy.typed_ast().source_context().is_none());
                        let core = lower(legacy, None)?;
                        let vcs = generate_core_vcs(&core, snapshot)?;
                        Ok((core, vcs, legacy.owners().to_vec()))
                    },
                )?)
            } else {
                None
            };
            let checked = Checker::check_theorem_skeletons(
                &source, typed, &scope, symbols, &labels, &resolved, None,
            )?;
            assert!(std::ptr::eq(checked.typed_ast(), typed));
            assert!(std::ptr::eq(checked.scope(), &scope));
            assert!(std::ptr::eq(checked.labels(), &labels));
            assert!(std::ptr::eq(checked.resolved_labels(), &resolved));
            if let Some((_, _, owners)) = &expected {
                assert_eq!(checked.owners(), owners);
            }
            let names = if case == 0 {
                vec!["Lem1", "UseLem1"]
            } else if case == 1 {
                vec!["Seed", "UseSeed"]
            } else if case == 3 {
                vec!["PrivT1", "UsePriv1"]
            } else {
                vec!["T"]
            };
            assert_eq!(
                checked
                    .owners()
                    .iter()
                    .map(|(owner, _)| owner.spelling.as_str())
                    .collect::<Vec<_>>(),
                names
            );
            for (index, (owner, visibility)) in checked.owners().iter().enumerate() {
                assert_eq!(
                    *visibility,
                    if index == 0 && matches!(case, 3 | 4 | 6) {
                        "private"
                    } else {
                        "public"
                    }
                );
                assert_eq!(owner.source_range.source_id, source.source_id());
                assert!(
                    text[owner.source_range.start..owner.source_range.end]
                        .contains(&owner.spelling)
                );
            }
            let core = lower(&checked, None)?;
            if let Some((expected, _, _)) = &expected {
                assert_eq!(&core, expected);
            }
            assert_eq!(core.source_id(), source.source_id());
            assert_eq!(core.module_id(), source.module());
            assert_eq!(core.proofs().len(), names.len());
            assert!(
                core.proofs()
                    .iter()
                    .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
            );
            if case >= 3 {
                let context = typed.source_context().unwrap();
                assert_eq!(typed.nodes().len(), ast.nodes().len());
                for (id, original) in source.arena().iter() {
                    let node = typed.nodes().node(TypedNodeId::new(id.index())).unwrap();
                    assert_eq!(node.resolved_node, Some(id));
                    assert_eq!(node.anchor, *original.origin().anchor());
                    assert!(
                        node.children
                            .iter()
                            .map(|id| id.index())
                            .eq(original.children().iter().map(|id| id.index()))
                    );
                }
                let wrappers = ast
                    .node_views()
                    .filter(|view| matches!(view.kind(), K::VisibleItem))
                    .collect::<Vec<_>>();
                assert_eq!(wrappers.len(), 1);
                let wrapper = wrappers[0];
                let [marker_id, inner_id] = wrapper.children() else {
                    panic!("unexpected visibility topology")
                };
                let marker = ast.node(*marker_id).unwrap();
                let inner = ast.node(*inner_id).unwrap();
                assert!(matches!(marker.kind, K::VisibilityMarker));
                assert!(matches!(inner.kind, K::TheoremItem));
                assert!(wrapper.range().start < inner.range.start);
                assert_eq!(shells.declarations()[0].range(), inner.range);
                assert_eq!(shells.declarations()[0].node_id(), *inner_id);
                assert_eq!(
                    shells.declarations()[0].visibility().marker_range(),
                    Some(marker.range)
                );
                assert_eq!(wrapper.range().end, inner.range.end);
                assert_eq!(
                    &text[marker.range.start..marker.range.end],
                    if case == 5 { "public" } else { "private" }
                );
                assert_eq!(&text[inner.range.start..inner.range.start + 7], "theorem");
                for id in [wrapper.id(), *marker_id, *inner_id] {
                    let resolved = source.resolved_node_for(id).unwrap();
                    assert_eq!(
                        typed
                            .nodes()
                            .node(TypedNodeId::new(resolved.index()))
                            .unwrap()
                            .links
                            .context,
                        Some(LocalTypeContextId::new(0))
                    );
                }
                assert_eq!(context.items().len(), shells.declarations().len());
                assert_eq!(core.items().len(), names.len());
                for (index, (((owner, visibility), shell), (_, item))) in checked
                    .owners()
                    .iter()
                    .zip(shells.declarations())
                    .zip(context.items().iter())
                    .enumerate()
                {
                    let private = index == 0 && case != 5;
                    let explicit = index == 0;
                    let expected_shell = if !explicit {
                        DeclarationShellVisibilityState::Unspecified
                    } else if private {
                        DeclarationShellVisibilityState::Private
                    } else {
                        DeclarationShellVisibilityState::Public
                    };
                    let expected_item = if !explicit {
                        SourceItemVisibility::Unspecified
                    } else if private {
                        SourceItemVisibility::Private
                    } else {
                        SourceItemVisibility::Public
                    };
                    assert_eq!(shell.visibility().state(), expected_shell);
                    assert_eq!(item.visibility, expected_item);
                    assert_eq!(item.shell, shell.id());
                    assert_eq!(item.source_range, shell.range());
                    assert_eq!(owner.source_range, shell.range());
                    assert_eq!(item.site, owner.site);
                    assert_eq!(
                        source.resolved_node_for(shell.node_id()),
                        typed.nodes().node(item.site.node()).unwrap().resolved_node
                    );
                    let symbol = symbols.symbols().get(&owner.symbol).unwrap();
                    assert_eq!(
                        symbol.visibility(),
                        if private {
                            Visibility::Private
                        } else {
                            Visibility::Public
                        }
                    );
                    assert_eq!(
                        symbol.export_status(),
                        if private {
                            ExportStatus::LocalOnly
                        } else {
                            ExportStatus::Exported
                        }
                    );
                    assert_eq!(
                        symbol.origin().anchor(),
                        &SourceAnchor::Range(shell.range())
                    );
                    let definition = symbols.definitions().by_symbol(&owner.symbol).unwrap();
                    assert_eq!(definition.visibility(), symbol.visibility());
                    let (core_id, core_item) = core.items().iter().nth(index).unwrap();
                    assert_eq!(core_item.symbol, owner.symbol);
                    assert_eq!(core_item.visibility.as_str(), *visibility);
                    assert_eq!(
                        core_item.source.anchor,
                        CoreSourceAnchor::SourceRange(shell.range())
                    );
                    assert_eq!(core.proofs().iter().nth(index).unwrap().1.item, core_id);
                }
                assert!(metadata.expr_metadata().iter().all(|(_, expression)| {
                    let node = typed.nodes().node(expression.typed_site.node()).unwrap();
                    expression.local_context == node.links.context
                        && expression.local_context.is_some()
                }));
                if case == 3 {
                    let first = core.items().iter().next().unwrap().0;
                    assert_eq!(
                        core.items().iter().nth(1).unwrap().1.dependencies,
                        vec![first]
                    );
                }
            }
            let binders = core
                .proof_nodes()
                .iter()
                .filter_map(|(_, node)| match &node.kind {
                    CoreProofNodeKind::IntroduceBinder { binder, .. } => Some(binder),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(binders.len(), names.len());
            assert!(binders.iter().all(|binder| binder.source_name.as_deref()
                == Some(if case == 1 { "Local" } else { "X" })));
            assert_eq!(core.obligation_seeds().len(), names.len());
            for (_, seed) in core.obligation_seeds().iter() {
                let goal = core.formulas().get(seed.goal.unwrap()).unwrap();
                assert!(matches!(goal.kind, CoreFormulaKind::Equals { .. }));
            }
            let citations = core
                .proof_nodes()
                .iter()
                .filter_map(|(_, node)| match &node.kind {
                    CoreProofNodeKind::TerminalGoal { citations, .. } => Some(citations.as_slice()),
                    _ => None,
                })
                .flatten()
                .collect::<Vec<_>>();
            if matches!(case, 0 | 1 | 3) {
                assert_eq!(labels.references().len(), 1);
                assert_eq!(
                    citations,
                    vec![&CoreCitation::Symbol(checked.owners()[0].0.symbol.clone())]
                );
            } else {
                assert!(labels.references().is_empty());
                assert!(citations.is_empty());
            }
            let vcs = generate_core_vcs(&core, snapshot)?;
            if let Some((_, expected_vcs, _)) = &expected {
                assert_eq!(&vcs, expected_vcs);
            }
            assert_eq!(vcs.vcs().len(), names.len());
            assert!(
                vcs.vcs()
                    .iter()
                    .all(|vc| vc.status == mizar_vc::vc_ir::VcStatus::Open)
            );
            if case >= 3 {
                for (vc, (_, proof)) in vcs.vcs().iter().zip(core.proofs().iter()) {
                    assert_eq!(
                        vc.anchor.owner,
                        mizar_vc::vc_ir::AnchorOwner::Theorem(proof.item)
                    );
                    let (_, seed) = core
                        .obligation_seeds()
                        .iter()
                        .find(|(_, seed)| seed.owner == proof.item)
                        .unwrap();
                    assert_eq!(
                        vc.goal,
                        mizar_vc::vc_ir::VcFormulaRef::Core(seed.goal.unwrap())
                    );
                }
            }
            if case == 6 {
                use mizar_vc::discharge::{DischargeInput, DischargePolicy, try_discharge};
                let guards = core
                    .formulas()
                    .iter()
                    .filter_map(|(_, row)| match &row.kind {
                        CoreFormulaKind::TypePred { ty, .. } => Some(ty.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                assert_eq!(guards, vec!["object", "object"]);
                let discharged = try_discharge(DischargeInput {
                    vc_set: &vcs,
                    policy: &DischargePolicy::default(),
                })
                .unwrap();
                assert!(discharged.evidence_records().is_empty());
                assert!(
                    discharged
                        .vc_set()
                        .vcs()
                        .iter()
                        .all(|vc| vc.status == mizar_vc::vc_ir::VcStatus::NeedsAtp)
                );
                assert_eq!(
                    mizar_core::elaborator::source_theorem_formula_payloads(&checked, &core, &[])
                        .unwrap_err(),
                    "unsupported source theorem formula payload"
                );
                assert_eq!(
                    mizar_vc::kernel_evidence_handoff::build_source_theorem_kernel_handoff(
                        &core,
                        &vcs,
                        vcs.vcs()[0].id
                    )
                    .unwrap_err(),
                    "theorem.handoff.invalid_projection"
                );
                assert!(
                    typed.source_composite_formula().is_none()
                        && typed.source_formula_composition().is_none()
                        && typed.source_statement().is_none()
                );
            }
            let replay = Checker::check_theorem_skeletons(
                &source, typed, &scope, symbols, &labels, &resolved, None,
            )?;
            assert_eq!(replay.owners(), checked.owners());
            assert_eq!(lower(&replay, None)?, core);
            assert_eq!(generate_core_vcs(&core, snapshot)?, vcs);
            assert_eq!(
                check_source_theorem_metadata(&source, symbols, &scope)?,
                output
            );
            assert_eq!(output, output_before);
            assert_eq!(
                (
                    source.clone(),
                    symbols.clone(),
                    scope,
                    labels,
                    resolved,
                    ast.snapshot_text(),
                    frontend.source.clone()
                ),
                before
            );
            if buffer {
                assert_eq!(fs::read_to_string(&path).unwrap(), "stale disk content");
            }
        }
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_scoped_skeleton_intake_preserves_refusals() -> Result<(), String> {
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker as Checker, check_source_theorem_metadata,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::NamespacePath,
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::{SignatureProjectionExtractor, source_proof_inputs},
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceInput,
        SourceOriginInput,
    };
    let singleton = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let incomplete = include_str!(
        "../../../../../../tests/miz/fail/theorems/fail_formula_statement_incomplete_proof_001.miz"
    );
    let texts = [
        singleton,
        incomplete,
        "theorem Empty: for X being set holds X = X proof end;",
        "theorem Pair: for A, B being set holds A = B proof let L, R be set; thus L = R; end;",
        "theorem Shadow: for A being set holds for A being set holds A = A proof let A be set; thus A = A; end;",
    ];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/scoped_refusals.miz");
    let ids = InMemorySessionIdAllocator::new();
    let mut genuine = None;
    for (case, text) in texts.iter().enumerate() {
        fs::write(&path, text).unwrap();
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: snapshot_id(787),
                input: SourceInput {
                    package_id: PackageId::new("scoped-refusals"),
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
        let (scope, labels, resolved) = source_proof_inputs(&source, symbols)?;
        let before = (
            source.clone(),
            symbols.clone(),
            scope.clone(),
            labels.clone(),
            resolved.clone(),
        );
        let metadata = check_source_theorem_metadata(&source, symbols, &scope)?;
        let typed = &metadata.2;
        let metadata_before = metadata.clone();
        if case == 0 {
            let flat = with_source_theorem_check(ast, &source, symbols, |legacy| {
                Ok(legacy.typed_ast().clone())
            })?;
            genuine = Some((
                source.clone(),
                symbols.clone(),
                scope.clone(),
                labels.clone(),
                resolved.clone(),
                flat,
                typed.clone(),
            ));
        } else {
            let expected =
                with_source_theorem_check(ast, &source, symbols, |_| Ok(())).unwrap_err();
            if case == 1 {
                assert_eq!(expected, "theorems.skeleton.incomplete_proof");
            }
            assert_eq!(
                Checker::check_theorem_skeletons(
                    &source, typed, &scope, symbols, &labels, &resolved, None
                )
                .unwrap_err(),
                expected
            );
            assert_eq!(
                Checker::check_theorem_skeletons(
                    &source, typed, &scope, symbols, &labels, &resolved, None
                )
                .unwrap_err(),
                expected
            );
        }
        if case == 1 {
            let (original, env, own_scope, own_labels, own_resolved, flat, scoped) =
                genuine.as_ref().unwrap();
            assert_ne!(original.source_id(), source.source_id());
            let controls = [
                (&source, env, own_scope, own_labels, own_resolved),
                (original, symbols, own_scope, own_labels, own_resolved),
                (original, env, &scope, own_labels, own_resolved),
                (original, env, own_scope, &labels, &resolved),
                (original, symbols, &scope, &labels, &resolved),
            ];
            for (control, (source, symbols, scope, labels, resolved)) in
                controls.into_iter().enumerate()
            {
                let inputs_before = (
                    source.clone(),
                    symbols.clone(),
                    scope.clone(),
                    labels.clone(),
                    resolved.clone(),
                    flat.clone(),
                    scoped.clone(),
                );
                let lower = mizar_checker::source_context::SourceBindingContextProducer::build_source_theorem_equalities(source, symbols, scope);
                let expected = Checker::check_theorem_skeletons(
                    source, flat, scope, symbols, labels, resolved, None,
                )
                .unwrap_err();
                let legacy_error = expected;
                let expected = if control == 2 {
                    assert_eq!(legacy_error, "formulas.typed_ast_invalid");
                    lower.as_ref().unwrap_err().clone()
                } else {
                    legacy_error
                };
                if std::ptr::eq(symbols, collected.env()) {
                    assert_eq!(expected, source_proof_inputs(source, symbols).unwrap_err());
                    assert_eq!(lower.unwrap_err(), expected);
                }
                assert_eq!(
                    Checker::check_theorem_skeletons(
                        source, scoped, scope, symbols, labels, resolved, None
                    )
                    .unwrap_err(),
                    expected
                );
                assert_eq!(
                    Checker::check_theorem_skeletons(
                        source, scoped, scope, symbols, labels, resolved, None
                    )
                    .unwrap_err(),
                    expected
                );
                assert_eq!(
                    (
                        source.clone(),
                        symbols.clone(),
                        scope.clone(),
                        labels.clone(),
                        resolved.clone(),
                        flat.clone(),
                        scoped.clone()
                    ),
                    inputs_before
                );
            }
        }
        assert_eq!(metadata, metadata_before);
        assert_eq!(
            (source.clone(), symbols.clone(), scope, labels, resolved,),
            before
        );
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_scoped_sequent_matches_genuine_problems() {
    use mizar_atp::{
        backend::{EncodedBackendProblem, EncodedBackendProblemParts},
        problem::*,
        translator::translate_source_theorem_sequent as sequent,
    };
    use mizar_core::core_ir::{CoreFormulaKind, CoreProofNodeKind, CoreTermKind};
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_theorem_vcs,
        kernel_evidence_handoff::build_source_theorem_discharge_kernel_handoff,
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcSchemaVersion, VcStatus},
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let baseline = step5c14_computation_core(text).unwrap();
    let module = baseline.module_id();
    let hex = |bytes: &[u8]| {
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let fields = |values: &[&str]| {
        values
            .iter()
            .map(|value| format!("{}:{value};", value.len()))
            .collect::<String>()
    };
    for (case, text) in [
        text.to_owned(),
        text.replace(" X", " Local"),
        format!(":: 日本語 λ\n\n{text}"),
        text.replace("T:", "Renamed:"),
        text.to_owned(),
        text.to_owned(),
        text.to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, collected) =
            super::super::super::tests::source_registration_owner_inputs(
                &text,
                module.package().as_str(),
                module.path().as_str(),
                usize::from(case == 4),
            );
        assert!(frontend.diagnostics.is_empty() && collected.diagnostics().is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        if case == 4 {
            assert_ne!(ast.source_id, baseline.source_id());
        }
        assert_eq!(source.source_id(), ast.source_id);
        let core = with_source_theorem_check(ast, &source, collected.env(), |check| {
            mizar_core::elaborator::lower_source_theorem_skeletons(check, None)
        })
        .unwrap();
        assert_eq!(core.source_id(), ast.source_id);
        let snapshot = snapshot_id(if case == 5 { 778 } else { 777 });
        let generation =
            GenerationSchemaVersion::new(if case == 5 { "" } else { "atp45-generation" });
        let schema = VcSchemaVersion::new(if case == 5 { "λ\n\0" } else { "atp45-schema" });
        let vcs = generate_source_theorem_vcs(&core, snapshot, &generation, &schema).unwrap();
        let (label, max_steps) = match case {
            5 => ("", 0),
            6 => ("λ\n\0", u32::MAX),
            _ => ("explicit", 64),
        };
        let policy = DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new(label),
                max_steps,
            },
        };
        let output = try_discharge(DischargeInput {
            vc_set: &vcs,
            policy: &policy,
        })
        .unwrap();
        let [vc] = output.vc_set().vcs() else {
            panic!("one processed theorem VC")
        };
        let id = vc.id;
        let handoff =
            build_source_theorem_discharge_kernel_handoff(&core, &vcs, id, &output, &policy)
                .unwrap();
        let immutable = (core.clone(), vcs.clone(), output.clone(), policy.clone());
        let proof = core.proofs().iter().next().unwrap().1;
        let CoreProofNodeKind::IntroduceBinder { binder, .. } =
            &core.proof_nodes().get(proof.root).unwrap().kind
        else {
            panic!()
        };
        let guard_ref = vc.local_context.entries()[0].formula.unwrap();
        let mizar_vc::vc_ir::VcFormulaRef::Core(guard_id) = guard_ref else {
            panic!()
        };
        let CoreFormulaKind::TypePred { subject, ty } =
            &core.formulas().get(guard_id).unwrap().kind
        else {
            panic!()
        };
        assert_eq!(ty.as_str(), "set");
        assert_eq!(
            core.terms().get(*subject).unwrap().kind,
            CoreTermKind::Var(binder.var)
        );
        let envelope = handoff.canonical_evidence();
        let guard = &envelope.formula_evidence()[0];
        let goal = envelope.final_goal();
        let guard_origin = envelope
            .provenance()
            .iter()
            .find(|row| row.provenance_id == guard.provenance_id())
            .unwrap();
        let goal_origin = envelope
            .provenance()
            .iter()
            .find(|row| row.provenance_id == goal.provenance_id)
            .unwrap();
        let target = envelope.target_vc();
        let common = format!(
            "mizar-atp/source-theorem-sequent/v1;source={};module={};binder={};target={}:{};handoff={};context={};guard-ref={};goal-ref={};guard-fingerprint={}:{};goal-fingerprint={}:{};guard-provenance={}:{};goal-provenance={}:{}",
            hex(format!("{:?}", core.source_id()).as_bytes()),
            hex(format!("{:?}", core.module_id()).as_bytes()),
            binder.var.index(),
            target.algorithm_id,
            hex(&target.digest),
            hex(handoff.canonical_hash().as_bytes()),
            hex(handoff.context_identity_hash().as_bytes()),
            hex(format!("{guard_ref:?}").as_bytes()),
            hex(format!("{:?}", vc.goal).as_bytes()),
            guard.formula_fingerprint().algorithm_id,
            hex(&guard.formula_fingerprint().digest),
            goal.formula_fingerprint.algorithm_id,
            hex(&goal.formula_fingerprint.digest),
            guard_origin.provenance_id,
            hex(&guard_origin.payload),
            goal_origin.provenance_id,
            hex(&goal_origin.payload)
        );
        let binder_source = format!("proof-let:{}", binder.var.index());
        let provenance = [
            (
                AtpSourceRef::LocalHypothesis(AtpSourceBinding::new(format!(
                    "local-context:{}",
                    vc.local_context.entries()[0].id.index() + 1
                ))),
                "guard",
            ),
            (
                AtpSourceRef::CheckerOwnedFact(AtpSourceBinding::new(&binder_source)),
                "binder",
            ),
            (
                AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("goal:1")),
                "closure",
            ),
        ]
        .into_iter()
        .enumerate()
        .map(|(id, (source, role))| {
            AtpProvenance::new(
                AtpProvenanceId::new(id as u32),
                source,
                format!("{common};role={role}"),
            )
        })
        .collect::<Vec<_>>();
        let local = AtpTerm::Variable("local:0".into());
        let closed = AtpFormulaTree::Forall {
            binders: vec![AtpBinder::new("local:0", None)],
            body: Box::new(AtpFormulaTree::Implies(
                Box::new(AtpFormulaTree::Atom(AtpAtom::new(
                    "type:set",
                    vec![local.clone()],
                ))),
                Box::new(AtpFormulaTree::Equality {
                    left: local.clone(),
                    right: local,
                }),
            )),
        };
        let predicate_key = fields(&[
            "symbol",
            "predicate",
            "1",
            &fields(&["mizar-symbol", "builtin-type:set"]),
            "type:set",
        ]);
        let binder_key = fields(&[
            "binder",
            &fields(&["generated-binder", &binder_source]),
            "1",
            "0",
            "0",
        ]);
        let predicate = format!("m_{}", hex(predicate_key.as_bytes()));
        assert_eq!(
            predicate,
            "m_363a73796d626f6c3b393a7072656469636174653b313a313b33363a31323a6d697a61722d73796d626f6c3b31363a6275696c74696e2d747970653a7365743b3b383a747970653a7365743b"
        );
        for (fragment, format, prefix) in [
            (LogicFragment::Fof, ConcreteFormat::Tptp, "V_"),
            (
                LogicFragment::SmtLibUninterpreted,
                ConcreteFormat::SmtLib,
                "v_",
            ),
        ] {
            let profile = LogicProfile::try_new(
                "caller-explicit",
                fragment,
                EqualitySupport::Supported,
                QuantifierPolicy::FirstOrder,
                SoftTypeStrategy::ScopedGuardPredicates,
                NativePropertySupport::Supported,
                [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                    .into_iter()
                    .collect(),
            )
            .unwrap();
            let expected = AtpProblem::try_new(AtpProblemParts {
                vc_id: id,
                target_binding: AtpTargetBinding::new(
                    AtpFingerprint::new(target.algorithm_id, target.digest.clone()).unwrap(),
                    format!(
                        "mizar-vc-kernel-evidence-handoff:{}",
                        hex(handoff.canonical_hash().as_bytes())
                    ),
                )
                .unwrap(),
                logic_profile: profile.clone(),
                expected_result: ExpectedBackendResult::Unsat,
                declarations: vec![
                    AtpDeclaration::new(
                        AtpDeclarationId::new(0),
                        AtpDeclarationKind::Predicate,
                        "type:set",
                        1,
                        AtpProvenanceId::new(0),
                    ),
                    AtpDeclaration::new(
                        AtpDeclarationId::new(1),
                        AtpDeclarationKind::GeneratedBinder,
                        "local:0",
                        0,
                        AtpProvenanceId::new(1),
                    ),
                ],
                axioms: vec![],
                conjecture: AtpFormula::new(
                    AtpFormulaId::new(0),
                    closed.clone(),
                    AtpProvenanceId::new(2),
                ),
                type_context: AtpTypeContext::new(vec![]),
                properties: vec![],
                symbol_map: vec![
                    AtpSymbolMapEntry::new(
                        "type:set",
                        AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("builtin-type:set")),
                    ),
                    AtpSymbolMapEntry::new(
                        "local:0",
                        AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new(&binder_source)),
                    ),
                ],
                provenance: provenance.clone(),
                diagnostics: vec![],
            })
            .unwrap();
            let actual = sequent(&core, &vcs, id, &output, &policy, profile.clone()).unwrap();
            assert_eq!(actual, expected);
            assert_eq!(
                actual,
                sequent(&core, &vcs, id, &output, &policy, profile.clone()).unwrap()
            );
            assert!(actual.observed_ground_terms().unwrap().is_empty());
            let variable = format!("{prefix}{}", hex(binder_key.as_bytes()));
            let (input_text, logic_fragment, label) = if format == ConcreteFormat::Tptp {
                (
                    format!(
                        "fof(conj_0, conjecture, (! [{variable}] : (({predicate}({variable}) => ({variable} = {variable}))))).\n"
                    ),
                    "fof",
                    "conj_0",
                )
            } else {
                (
                    format!(
                        "(set-logic UF)\n(declare-sort mizar_universe 0)\n(declare-fun {predicate} (mizar_universe) Bool)\n(assert (! (not (forall (({variable} mizar_universe)) (=> ({predicate} {variable}) (= {variable} {variable})))) :named neg_conj_0))\n(check-sat)\n"
                    ),
                    "smtlib-uninterpreted",
                    "neg_conj_0",
                )
            };
            let symbol_bindings = vec![
                format!(
                    "a=8:{};b={}:{};s=mizar=16:{}",
                    hex(b"type:set"),
                    predicate.len(),
                    hex(predicate.as_bytes()),
                    hex(b"builtin-type:set")
                ),
                format!(
                    "a=7:{};b={}:{};s=binder={}:{}",
                    hex(b"local:0"),
                    variable.len(),
                    hex(variable.as_bytes()),
                    binder_source.len(),
                    hex(binder_source.as_bytes())
                ),
            ];
            // Reuse the unchanged native producer only for its opaque provenance hash.
            let provenance_hash = EncodedBackendProblem::encode(&expected, format)
                .unwrap()
                .provenance_hash();
            let expected_encoded = EncodedBackendProblem::new(EncodedBackendProblemParts {
                problem_id: expected.problem_id(),
                target_binding: expected.target_binding().clone(),
                expected_result: ExpectedBackendResult::Unsat,
                concrete_format: format,
                logic_profile_name: "caller-explicit".to_owned(),
                logic_fragment: logic_fragment.to_owned(),
                input_text: input_text.into_bytes(),
                formula_labels: vec![label.to_owned()],
                symbol_bindings,
                provenance_hash,
            })
            .unwrap();
            assert_eq!(
                EncodedBackendProblem::encode(&actual, format).unwrap(),
                expected_encoded
            );
        }
        assert_eq!(vcs.vcs()[0].status, VcStatus::Open);
        assert!(
            core.proofs().iter().all(|(_, row)| row.status
                == mizar_core::core_ir::CoreProofStatus::PendingAutomaticProof)
        );
        assert!(
            core.obligation_seeds()
                .iter()
                .all(|(_, row)| row.status == mizar_core::core_ir::ObligationSeedStatus::Active)
        );
        assert_eq!((core, vcs, output, policy), immutable);
    }
}

#[test]
fn source_theorem_scoped_sequent_preserves_refusals() {
    use mizar_atp::{
        problem::*,
        translator::{translate_source_theorem_sequent as sequent, *},
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeInput, DischargePolicy, try_discharge},
        kernel_evidence_handoff::build_source_theorem_discharge_kernel_handoff as handoff,
        vc_ir::*,
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let core = step5c14_computation_core(text).unwrap();
    let vcs = generate_core_vcs(&core, snapshot_id(777)).unwrap();
    let id = vcs.vcs()[0].id;
    let policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("explicit"),
            max_steps: 64,
        },
    };
    let output = try_discharge(DischargeInput {
        vc_set: &vcs,
        policy: &policy,
    })
    .unwrap();
    let profile = LogicProfile::try_new(
        "source-refusals",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::ScopedGuardPredicates,
        NativePropertySupport::Unsupported,
        [ConcreteFormat::Tptp].into_iter().collect(),
    )
    .unwrap();
    let before = (
        core.clone(),
        vcs.clone(),
        output.clone(),
        policy.clone(),
        profile.clone(),
    );
    let good = sequent(&core, &vcs, id, &output, &policy, profile.clone()).unwrap();
    let equal_policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("different"),
            max_steps: 0,
        },
    };
    assert_eq!(
        output,
        try_discharge(DischargeInput {
            vc_set: &vcs,
            policy: &equal_policy
        })
        .unwrap()
    );
    assert_eq!(
        good,
        sequent(&core, &vcs, id, &output, &equal_policy, profile.clone()).unwrap()
    );
    let foreign = step5c14_computation_core(&text.replace("T:", "Foreign:")).unwrap();
    let foreign_vcs = generate_core_vcs(&foreign, snapshot_id(777)).unwrap();
    let foreign_output = try_discharge(DischargeInput {
        vc_set: &foreign_vcs,
        policy: &policy,
    })
    .unwrap();
    let later = generate_core_vcs(&core, snapshot_id(778)).unwrap();
    let later_output = try_discharge(DischargeInput {
        vc_set: &later,
        policy: &policy,
    })
    .unwrap();
    for wrong_output in [&foreign_output, &later_output] {
        assert_ne!(wrong_output, &output);
        let expected = handoff(&core, &vcs, id, wrong_output, &policy).unwrap_err();
        assert_eq!(
            sequent(&core, &vcs, id, wrong_output, &policy, profile.clone()).unwrap_err(),
            expected
        );
    }
    for (current_core, current_vcs, id) in [
        (&foreign, &vcs, id),
        (&core, &foreign_vcs, id),
        (&core, output.vc_set(), id),
        (&core, &vcs, VcId::new(usize::MAX)),
    ] {
        let expected =
            handoff(current_core, current_vcs, id, &foreign_output, &policy).unwrap_err();
        assert_eq!(
            sequent(
                current_core,
                current_vcs,
                id,
                &foreign_output,
                &policy,
                profile.clone()
            )
            .unwrap_err(),
            expected
        );
    }
    let cited = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for source in [
        text.replacen("theorem", "lemma", 1),
        text.replace("set", "object"),
        format!("{text} {}", text.replace("T:", "U:")),
        cited.to_owned(),
        "theorem C: 0 = 0 by computation(steps: 8);".to_owned(),
    ] {
        let unsupported = step5c14_computation_core(&source).unwrap();
        let vcs = if source.contains("computation") {
            mizar_vc::generator::generate_source_computation_request(
                &unsupported,
                snapshot_id(777),
                &GenerationSchemaVersion::new("mizar-vc-generation-step5c14-computation-v1"),
                &VcSchemaVersion::new("mizar-vc-vcset-step5c14-computation-v1"),
            )
            .unwrap()
        } else {
            generate_core_vcs(&unsupported, snapshot_id(777)).unwrap()
        };
        assert!(!vcs.vcs().is_empty());
        for vc in vcs.vcs() {
            let expected =
                handoff(&unsupported, &vcs, vc.id, &foreign_output, &policy).unwrap_err();
            assert_eq!(
                sequent(
                    &unsupported,
                    &vcs,
                    vc.id,
                    &foreign_output,
                    &policy,
                    profile.clone()
                )
                .unwrap_err(),
                expected
            );
        }
    }
    for invalid_profile in 0..5 {
        let mut mode = SoftTypeStrategy::ScopedGuardPredicates;
        let mut equality = EqualitySupport::Supported;
        let mut quantifiers = QuantifierPolicy::FirstOrder;
        let mut fragment = LogicFragment::Fof;
        let mut format = ConcreteFormat::Tptp;
        match invalid_profile {
            0 => mode = SoftTypeStrategy::GuardPredicates,
            1 => equality = EqualitySupport::Unsupported,
            2 => quantifiers = QuantifierPolicy::PropositionalOnly,
            3 => fragment = LogicFragment::TffLike,
            4 => format = ConcreteFormat::SmtLib,
            _ => unreachable!(),
        }
        let bad = LogicProfile::try_new(
            "bad-profile",
            fragment,
            equality,
            quantifiers,
            mode,
            NativePropertySupport::Unsupported,
            [format].into_iter().collect(),
        )
        .unwrap();
        assert_eq!(
            sequent(&core, &vcs, id, &output, &policy, bad.clone()).unwrap_err(),
            "theorem.atp.invalid_projection"
        );
        assert_eq!(
            sequent(&foreign, &vcs, id, &foreign_output, &policy, bad).unwrap_err(),
            handoff(&foreign, &vcs, id, &foreign_output, &policy).unwrap_err()
        );
    }
    let processed = output.vc_set();
    let actual_handoff = handoff(&core, &vcs, id, &output, &policy).unwrap();
    for soft_type_projections in [
        vec![],
        vec![AtpSoftTypeProjection {
            key: "guard".into(),
            representation: AtpSoftTypeRepresentation::GuardFormula(AtpFormulaTree::True),
            provenance: AtpProjectionProvenance::new(
                AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("control")),
                "control",
            ),
        }],
    ] {
        for result in [
            translate_declarations(AtpDeclarationTranslationInput {
                vc_set: processed,
                vc: id,
                kernel_handoff: &actual_handoff,
                logic_profile: profile.clone(),
                declaration_projections: vec![],
                soft_type_projections: soft_type_projections.clone(),
                diagnostics: vec![],
            })
            .map(|_| ()),
            translate_problem(AtpTranslationInput {
                vc_set: processed,
                vc: id,
                kernel_handoff: &actual_handoff,
                logic_profile: profile.clone(),
                declaration_projections: vec![],
                soft_type_projections,
                formula_projections: vec![],
                diagnostics: vec![],
            })
            .map(|_| ()),
        ] {
            assert!(matches!(
                result.unwrap_err(),
                AtpTranslationError::Problem {
                    source: AtpProblemError::UnsupportedProfileFeature {
                        feature: "scoped-guard generic translation"
                    }
                }
            ));
        }
    }
    assert_eq!(
        translate_source_existential_registration(&core, processed, id, profile.clone())
            .unwrap_err(),
        "registration.atp.invalid_projection"
    );
    assert_eq!((core, vcs, output, policy, profile), before);
}

#[test]
fn task257b5_native_universal_composition_preserves_original_owners_and_replay()
-> Result<(), String> {
    use mizar_checker::{
        binding_env::{BinderIdentity, BindingContextLayer, BindingContextOwner},
        cluster_trace::ClusterFactTable,
        overload_resolution::*,
        resolved_typed_ast::{ResolvedTypedAst, ResolvedTypedAstInputs},
        source_composite_formula::*,
        source_formula_composition::*,
        source_type::{SourceTypeHead, SourceTypeProducer},
        type_checker::{FormulaStatus, TermStatus, TypeHeadRef},
        typed_ast::{NodeRecoveryState, TypeEntryActual, TypeStatus, TypedSiteRef},
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::NamespacePath,
        names::{
            LocalTermBinding, SourceVariableBindingKind, SourceVariableScopeInput,
            SourceVariableScopeResolver,
        },
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::{SignatureProjectionExtractor, source_proof_scope_maps},
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceAnchor,
        SourceInput, SourceOriginInput,
    };
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceQuantifierKind};
    use std::collections::{BTreeMap, BTreeSet};

    let fixture = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    // Counts are literal source inventories: owners, universals, header binders,
    // all declarations, references, equality occurrences and written type nodes.
    let cases = [
        (fixture.to_owned(), (2, 2, 2, 4, 8, 4, 4)),
        (fixture.replace("Lem1", "Seed")
            .replace("for X being set holds X = X", "for Header being set holds Header = Header")
            .replace("let X be set;", "let Local be set;")
            .replace("thus X = X", "thus Local = Local"), (2, 2, 2, 4, 8, 4, 4)),
        (format!("\n:: original layout\n{}", fixture.replace("being set", "being\tset")
            .replace("proof", "proof\n:: proof layout\n")), (2, 2, 2, 4, 8, 4, 4)),
        ("theorem Pair: for A, Spare, B being set holds A = B proof let L, Unused, R be set; thus L = R; end;".into(), (1, 1, 3, 6, 4, 2, 2)),
        ("theorem Groups: for A being set, Spare, B being set holds A = B proof let L be set, Unused, R be set; thus L = R; end;".into(), (1, 1, 3, 6, 4, 2, 4)),
        ("theorem Nested: for A being set for Unused being set holds for B being set holds A = B proof let L be set; let Idle be set; let R be set; thus L = R; end;".into(), (1, 3, 3, 6, 4, 2, 6)),
        ("theorem Shadow: for A being set holds for A being set holds A = A proof let A be set; thus A = A; end;".into(), (1, 2, 2, 3, 4, 2, 3)),
    ];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let mut prepared = Vec::new();
    // Establish EVERY admitted source's unchanged lower before the new entrance.
    for (case, (text, counts)) in cases.iter().enumerate() {
        let path = root.join(format!("src/universal_{case}.miz"));
        fs::write(&path, text).unwrap();
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: snapshot_id(795),
                input: SourceInput {
                    package_id: PackageId::new("native-universal-composition"),
                    module_path: ModulePath::new(format!("owners_{case}")),
                    normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Disk { path },
                },
            },
            &ids,
        )
        .unwrap();
        assert!(
            frontend.diagnostics.is_empty(),
            "case {case}: {:?}",
            frontend.diagnostics
        );
        assert_eq!(frontend.source.source_text.as_ref(), text);
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
        let baseline =
            SourceTypeProducer::build_source_theorem_equalities(&source, symbols, &scope)?;
        let (env, inference, typed, metadata) = &baseline;
        let context = typed.source_context().unwrap();
        let types = typed.source_type().unwrap();
        let primary = typed.source_term().unwrap();
        let atomic = typed.source_atomic_formula().unwrap();
        let (owners, universals, binders, declarations, references, atoms, written_types) = *counts;
        assert_eq!(context.items().len(), owners);
        assert_eq!(context.declarations().len(), declarations);
        assert_eq!(scope.bindings().len(), declarations);
        assert_eq!(env.bindings().len(), declarations);
        assert_eq!(context.binding_env(), env);
        assert_eq!(env.contexts().len(), 1 + universals + owners);
        assert_eq!(primary.terms().len(), references);
        assert_eq!(primary.references().len(), references);
        assert_eq!(scope.references().len(), references);
        assert_eq!(atomic.formulas().len(), atoms);
        assert_eq!(types.applications().len(), declarations);
        assert_eq!(types.expressions().len(), written_types);
        assert_eq!(metadata.len(), references + atoms);
        assert!(typed.source_node_entries().is_none());
        assert!(typed.source_composite_formula().is_none());
        assert!(typed.source_formula_composition().is_none());
        assert!(env.diagnostics().is_empty());
        assert!(inference.diagnostics().is_empty());
        assert!(inference.candidate_sets().is_empty());
        assert!(inference.facts().is_empty());
        assert!(typed.facts().is_empty() && typed.coercions().is_empty());
        assert!(typed.initial_obligations().is_empty());
        assert!(
            inference
                .terms()
                .iter()
                .all(|(_, term)| term.status == TermStatus::Inferred && term.deferred.is_empty())
        );
        assert!(
            inference
                .formulas()
                .iter()
                .all(|(_, formula)| formula.status == FormulaStatus::Checked
                    && formula.deferred.is_empty())
        );
        for (_, entry) in inference.type_entries().iter() {
            assert_eq!(entry.status, TypeStatus::Known);
            let TypeEntryActual::Known(actual) = entry.actual else {
                panic!("non-Known lower type")
            };
            let actual = inference.normalized_types().get(actual).unwrap();
            assert_eq!(actual.head, TypeHeadRef::BuiltinSet);
            assert!(
                actual.args.is_empty()
                    && actual.attributes.positive().is_empty()
                    && actual.attributes.negative().is_empty()
            );
        }
        let maps = source_proof_scope_maps(&source, symbols, &scope)?;
        let typed_site = |id| {
            let resolved = source.resolved_node_for(id).unwrap();
            let matches = typed
                .nodes()
                .iter()
                .filter(|(_, row)| row.resolved_node == Some(resolved))
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1);
            TypedSiteRef::Node(matches[0].0)
        };
        let spelling = |id| {
            let range = ast.node_view(id).unwrap().range();
            let mut tokens = ast
                .node_views()
                .filter(|view| {
                    view.as_token().is_some()
                        && range.start <= view.range().start
                        && view.range().end <= range.end
                })
                .collect::<Vec<_>>();
            tokens.sort_by_key(|view| view.range().start);
            tokens
                .iter()
                .map(|view| view.as_token().unwrap().text.as_ref())
                .collect::<Vec<_>>()
                .join(" ")
        };
        let body_context = |id| {
            let local = maps.0.get(&id).unwrap();
            let matches = env
                .contexts()
                .iter()
                .filter(|(_, row)| row.lexical_scope.as_ref() == Some(local))
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1);
            matches[0].0
        };
        let mut quantifiers = ast
            .node_views()
            .filter(|view| view.kind() == &K::QuantifiedFormula(SurfaceQuantifierKind::Universal))
            .collect::<Vec<_>>();
        quantifiers.sort_by_key(|view| view.range().start);
        assert_eq!(quantifiers.len(), universals);
        let mut expected = SourceCompositeFormulaHandoffInput {
            source_id: source.source_id(),
            module_id: module.clone(),
            formulas: vec![],
            wrappers: vec![],
            roots: vec![],
            binders: vec![],
            type_sites: vec![],
            edges: vec![],
            requests: vec![],
        };
        let mut composition = SourceFormulaCompositionHandoffInput {
            source_id: source.source_id(),
            module_id: module.clone(),
            atomic_edges: vec![],
            bound_uses: vec![],
        };
        let mut binder_for = BTreeMap::new();
        let mut terminal_atoms = Vec::new();
        for (index, quantifier) in quantifiers.iter().enumerate() {
            let formula = SourceCompositeFormulaId::new(index);
            let body = body_context(quantifier.id());
            let incoming = env.contexts().get(body).unwrap().parent.unwrap();
            assert_eq!(
                env.contexts().get(body).unwrap().layer,
                BindingContextLayer::Expression
            );
            assert_eq!(
                env.contexts().get(body).unwrap().owner,
                BindingContextOwner::SourceFormula {
                    source_range: quantifier.range()
                }
            );
            let local = context.context_links().get(body).unwrap().local_context;
            assert_eq!(
                typed
                    .nodes()
                    .node(typed_site(quantifier.id()).node())
                    .unwrap()
                    .links
                    .context,
                Some(local)
            );
            let marker = quantifier
                .children()
                .iter()
                .filter_map(|id| {
                    let child = ast.node_view(*id).unwrap();
                    child
                        .as_token()
                        .filter(|token| matches!(token.text.as_ref(), "for" | "holds"))
                        .map(|token| token.text.to_string())
                })
                .collect::<Vec<_>>()
                .join(" ");
            expected.formulas.push(SourceCompositeFormulaInput {
                site: typed_site(quantifier.id()),
                source_range: quantifier.range(),
                source_ordinal: index,
                context: incoming,
                recovery: SourceCompositeFormulaRecovery::Normal,
                spelling: marker,
                kind: SourceCompositeFormulaKind::Universal,
            });
            let parent = ast
                .node_views()
                .find(|view| view.children().contains(&quantifier.id()))
                .unwrap();
            if let Some(parent_index) = quantifiers.iter().position(|view| view.id() == parent.id())
            {
                expected.edges.push(SourceFormulaEdgeInput {
                    parent: SourceCompositeFormulaId::new(parent_index),
                    ordinal: 0,
                    role: SourceFormulaEdgeRole::UniversalBody,
                    child: formula,
                });
            } else {
                assert_eq!(parent.kind(), &K::FormulaExpression);
                let owner = ast
                    .node_views()
                    .find(|view| view.children().contains(&parent.id()))
                    .unwrap();
                assert!(matches!(owner.kind(), K::TheoremItem | K::LemmaItem));
                let item = context
                    .items()
                    .iter()
                    .find(|(_, item)| item.site == typed_site(owner.id()))
                    .unwrap()
                    .1;
                assert_eq!(item.source_range, owner.range());
                assert!(item.parent.is_none());
                expected.roots.push(SourceFormulaRootInput {
                    formula,
                    ordinal: expected.roots.len(),
                    ownership: SourceFormulaRootOwnership::UnassignedStatement,
                });
            }
            expected.requests.push(SourceFormulaRequestInput {
                formula,
                ordinal: 0,
                kind: SourceFormulaRequestKind::QuantifierSemantics,
                binder: None,
                type_site: None,
            });
            let mut binder_ordinal = 0;
            for segment_id in quantifier.children() {
                let segment = ast.node_view(*segment_id).unwrap();
                if segment.kind() != &K::QuantifierVariableSegment {
                    continue;
                }
                let written = ast.node_view(*segment.children().last().unwrap()).unwrap();
                assert_eq!(written.kind(), &K::TypeExpression);
                let head = ast.node_view(written.children()[0]).unwrap();
                assert_eq!(head.kind(), &K::TypeHead);
                assert_eq!(spelling(written.id()), "set");
                let segment_marker = segment.children()[..segment.children().len() - 1]
                    .iter()
                    .map(|id| spelling(*id))
                    .collect::<Vec<_>>()
                    .join(" ");
                for name_id in &segment.children()[..segment.children().len() - 2] {
                    let name = ast.node_view(*name_id).unwrap();
                    if name.as_token().is_none_or(|token| {
                        token.kind != mizar_syntax::SurfaceTokenKind::Identifier
                    }) {
                        continue;
                    }
                    let original = scope
                        .bindings()
                        .iter()
                        .find(|binding| binding.node() == name.id())
                        .unwrap();
                    assert_eq!(original.kind(), SourceVariableBindingKind::Quantifier);
                    let matching = env
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
                    assert_eq!(matching.len(), 1);
                    let (binding, entry) = matching[0];
                    assert_eq!(entry.owner_context, body);
                    assert_eq!(entry.visible_after_ordinal, original.ordinal());
                    let declaration = context
                        .declarations()
                        .iter()
                        .find(|(_, row)| row.binding == binding)
                        .unwrap()
                        .1;
                    assert_eq!(declaration.site, typed_site(name.id()));
                    assert_eq!(declaration.declaration_range, name.range());
                    assert_eq!(declaration.written_type_range, written.range());
                    let application = types
                        .applications()
                        .iter()
                        .find(|(_, row)| row.binding() == binding)
                        .unwrap()
                        .1;
                    let written_type = types.expressions().get(application.root()).unwrap();
                    assert_eq!(written_type.site(), &typed_site(written.id()));
                    assert_eq!(written_type.head_site(), &typed_site(head.id()));
                    assert_eq!(written_type.source_range(), written.range());
                    assert_eq!(written_type.head_range(), head.range());
                    assert_eq!(written_type.head(), &SourceTypeHead::BuiltinSet);
                    let binder = SourceQuantifierBinderId::new(expected.binders.len());
                    let type_site = SourceBinderTypeSiteId::new(expected.type_sites.len());
                    binder_for.insert(original.id(), binder);
                    expected.binders.push(SourceQuantifierBinderInput {
                        formula,
                        ordinal: binder_ordinal,
                        segment_site: typed_site(segment.id()),
                        segment_range: segment.range(),
                        segment_spelling: segment_marker.clone(),
                        identifier_site: typed_site(name.id()),
                        identifier_range: name.range(),
                        identifier_spelling: original.spelling().into(),
                        local: LocalTermBinding::new(
                            original.spelling(),
                            original.scope().clone(),
                            original.range(),
                            original.ordinal(),
                        ),
                        binding,
                        body_context: body,
                        type_site,
                        recovery: SourceCompositeFormulaRecovery::Normal,
                    });
                    expected.type_sites.push(SourceBinderTypeSiteInput {
                        binder,
                        site: typed_site(written.id()),
                        source_range: written.range(),
                        spelling: "set".into(),
                        head_site: typed_site(head.id()),
                        head_range: head.range(),
                        head_spelling: "set".into(),
                        context: incoming,
                        recovery: SourceCompositeFormulaRecovery::Normal,
                        head: SourceBinderTypeHead::BuiltinSet,
                    });
                    expected.requests.push(SourceFormulaRequestInput {
                        formula,
                        ordinal: 1 + binder_ordinal,
                        kind: SourceFormulaRequestKind::BinderType,
                        binder: Some(binder),
                        type_site: Some(type_site),
                    });
                    binder_ordinal += 1;
                }
            }
            let child = ast
                .node_view(*quantifier.children().last().unwrap())
                .unwrap();
            if child.kind() == &K::BuiltinPredicateApplication {
                let atomic_id = atomic
                    .formulas()
                    .iter()
                    .find(|(_, row)| row.site() == &typed_site(child.id()))
                    .unwrap()
                    .0;
                let edge_id = SourceFormulaAtomicEdgeId::new(composition.atomic_edges.len());
                composition.atomic_edges.push(SourceFormulaAtomicEdgeInput {
                    formula,
                    ordinal: 0,
                    role: SourceFormulaAtomicEdgeRole::UniversalBody,
                    child: atomic_id,
                });
                terminal_atoms.push((edge_id, child.id()));
            } else {
                assert_eq!(
                    child.kind(),
                    &K::QuantifiedFormula(SurfaceQuantifierKind::Universal)
                );
            }
        }
        let mut use_ordinals = BTreeMap::new();
        for (edge, atom_id) in terminal_atoms {
            let atom = ast.node_view(atom_id).unwrap();
            for operand in [atom.children()[0], atom.children()[2]] {
                let expression = ast.node_view(operand).unwrap();
                assert_eq!(expression.kind(), &K::TermExpression);
                let term_id = expression.children()[0];
                let original = scope
                    .references()
                    .iter()
                    .find(|reference| reference.node() == term_id)
                    .unwrap();
                let binder = *binder_for.get(&original.binding()).unwrap();
                let (term_id, term) = primary
                    .terms()
                    .iter()
                    .find(|(_, row)| row.site() == &typed_site(term_id))
                    .unwrap();
                let (reference_id, reference) = primary
                    .references()
                    .iter()
                    .find(|(_, row)| row.term() == term_id)
                    .unwrap();
                assert_eq!(
                    reference.binding(),
                    expected.binders[binder.index()].binding
                );
                assert_eq!(reference.lexical_scope(), Some(original.scope()));
                assert_eq!(reference.use_ordinal(), original.ordinal());
                assert_eq!(term.source_range(), original.range());
                let ordinal = use_ordinals.entry(binder).or_insert(0);
                composition.bound_uses.push(SourceQuantifierBoundUseInput {
                    binder,
                    ordinal: *ordinal,
                    body_edge: edge,
                    term: term_id,
                    reference: reference_id,
                });
                *ordinal += 1;
            }
        }
        assert_eq!(expected.binders.len(), binders);
        assert_eq!(expected.roots.len(), owners);
        assert_eq!(expected.edges.len(), universals - owners);
        assert_eq!(expected.requests.len(), universals + binders);
        assert_eq!(composition.atomic_edges.len(), owners);
        assert_eq!(composition.bound_uses.len(), 2 * owners);
        let header_terms = composition
            .bound_uses
            .iter()
            .map(|row| row.term)
            .collect::<BTreeSet<_>>();
        for (_, reference) in primary.references().iter() {
            let term = primary.terms().get(reference.term()).unwrap();
            let original = scope
                .references()
                .iter()
                .find(|row| {
                    source.resolved_node_for(row.node())
                        == typed
                            .nodes()
                            .node(term.site().node())
                            .unwrap()
                            .resolved_node
                })
                .unwrap();
            assert_eq!(reference.use_ordinal(), original.ordinal());
            assert_eq!(reference.lexical_scope(), Some(original.scope()));
            let actual = &scope.bindings()[original.binding().index()];
            assert_eq!(
                header_terms.contains(&reference.term()),
                actual.kind() == SourceVariableBindingKind::Quantifier
            );
            let binding = env.bindings().get(reference.binding()).unwrap();
            assert_eq!(
                binding.identity,
                BinderIdentity::ResolverLocal {
                    scope: actual.scope().clone(),
                    ordinal: actual.ordinal(),
                    declaration_range: actual.range()
                }
            );
        }
        let mut with_rows = typed.clone();
        with_rows
            .install_source_nodes(&source)
            .map_err(|error| error.to_string())?;
        let rows = with_rows
            .source_node_entries()
            .unwrap()
            .cloned()
            .collect::<Vec<_>>();
        // Generic original rows own the reachable graph, not disconnected
        // speculative structural storage. Root token aliases are one node.
        let mut reachable = BTreeSet::new();
        let mut pending = vec![source.arena().root()];
        while let Some(site) = pending.pop() {
            if reachable.insert(site) {
                pending.extend_from_slice(source.arena().node(site).unwrap().children());
            }
        }
        assert_eq!(
            rows.iter().map(|row| row.site()).collect::<BTreeSet<_>>(),
            reachable
        );
        assert_eq!(rows.len(), reachable.len());
        assert!(
            source
                .arena()
                .iter()
                .filter(|(_, node)| matches!(node.kind(), K::Token(_)))
                .all(|(site, _)| reachable.contains(&site))
        );
        for row in &rows {
            let actual = source.arena().node(row.site()).unwrap();
            assert_eq!(row.kind(), actual.kind());
            assert_eq!(&SourceAnchor::Range(row.range()), actual.origin().anchor());
            assert_eq!(row.recovery(), NodeRecoveryState::Normal);
            let surface = ast
                .node_views()
                .find(|view| source.resolved_node_for(view.id()) == Some(row.site()))
                .unwrap();
            assert_eq!(row.spelling(), spelling(surface.id()));
            if let Some(parent) = row.parent() {
                let parent = source.arena().node(parent).unwrap();
                assert_eq!(parent.children()[row.child_ordinal()], row.site());
            } else {
                assert_eq!(row.site(), source.arena().root());
            }
        }
        prepared.push((
            source,
            symbols.clone(),
            scope,
            baseline,
            expected,
            composition,
            rows,
        ));
    }

    for (case, (source, symbols, scope, baseline, expected, expected_composition, rows)) in
        prepared.iter().enumerate()
    {
        let (env, inference, before, metadata) = baseline;
        let installed = before.clone().with_source_formula_composition_from_original(source, symbols, scope)
            .expect("native universal composition is unsupported only after every genuine lower prerequisite");
        let composite = installed.source_composite_formula().unwrap();
        let composition = installed.source_formula_composition().unwrap();
        assert_eq!(composite.binding_env(), env);
        assert_eq!(
            composite.binding_env(),
            installed.source_context().unwrap().binding_env()
        );
        assert_eq!(composite.source_id(), source.source_id());
        assert_eq!(composite.module_id(), source.module());
        assert_eq!(composition.source_id(), source.source_id());
        assert_eq!(composition.module_id(), source.module());
        assert_eq!(
            installed
                .source_node_entries()
                .unwrap()
                .cloned()
                .collect::<Vec<_>>(),
            *rows
        );
        assert_eq!(installed.nodes(), before.nodes());
        assert_eq!(installed.contexts(), before.contexts());
        assert_eq!(installed.types(), before.types());
        assert_eq!(installed.types(), inference.type_entries());
        assert_eq!(installed.facts(), before.facts());
        assert_eq!(installed.coercions(), before.coercions());
        assert_eq!(
            installed.initial_obligations(),
            before.initial_obligations()
        );
        assert_eq!(installed.diagnostics(), before.diagnostics());
        assert_eq!(installed.source_context(), before.source_context());
        assert_eq!(installed.source_type(), before.source_type());
        assert_eq!(installed.source_term(), before.source_term());
        assert_eq!(
            installed.source_atomic_formula(),
            before.source_atomic_formula()
        );
        assert!(installed.source_statement().is_none());
        assert!(installed.source_statement_references().is_none());
        assert!(installed.source_statement_witnesses().is_none());
        assert!(installed.source_proof_local_binding().is_none());
        assert_eq!(composite.formulas().len(), expected.formulas.len());
        for ((_, row), expected) in composite.formulas().iter().zip(&expected.formulas) {
            assert_eq!(
                (
                    row.site(),
                    row.source_range(),
                    row.source_ordinal(),
                    row.context(),
                    row.recovery(),
                    row.spelling(),
                    row.kind()
                ),
                (
                    &expected.site,
                    expected.source_range,
                    expected.source_ordinal,
                    expected.context,
                    expected.recovery,
                    expected.spelling.as_str(),
                    expected.kind
                )
            );
        }
        assert!(composite.wrappers().is_empty());
        assert_eq!(composite.roots().len(), expected.roots.len());
        for ((_, row), expected) in composite.roots().iter().zip(&expected.roots) {
            assert_eq!(
                (row.formula(), row.ordinal(), row.ownership()),
                (expected.formula, expected.ordinal, expected.ownership)
            );
        }
        assert_eq!(composite.binders().len(), expected.binders.len());
        for ((_, row), expected) in composite.binders().iter().zip(&expected.binders) {
            assert_eq!(
                (
                    row.formula(),
                    row.ordinal(),
                    row.segment_site(),
                    row.segment_range(),
                    row.segment_spelling()
                ),
                (
                    expected.formula,
                    expected.ordinal,
                    &expected.segment_site,
                    expected.segment_range,
                    expected.segment_spelling.as_str()
                )
            );
            assert_eq!(
                (
                    row.identifier_site(),
                    row.identifier_range(),
                    row.identifier_spelling(),
                    row.local(),
                    row.binding(),
                    row.body_context(),
                    row.type_site(),
                    row.recovery()
                ),
                (
                    &expected.identifier_site,
                    expected.identifier_range,
                    expected.identifier_spelling.as_str(),
                    &expected.local,
                    expected.binding,
                    expected.body_context,
                    expected.type_site,
                    expected.recovery
                )
            );
        }
        assert_eq!(composite.type_sites().len(), expected.type_sites.len());
        for ((_, row), expected) in composite.type_sites().iter().zip(&expected.type_sites) {
            assert_eq!(
                (
                    row.binder(),
                    row.site(),
                    row.source_range(),
                    row.spelling(),
                    row.head_site(),
                    row.head_range(),
                    row.head_spelling(),
                    row.context(),
                    row.recovery(),
                    row.head()
                ),
                (
                    expected.binder,
                    &expected.site,
                    expected.source_range,
                    expected.spelling.as_str(),
                    &expected.head_site,
                    expected.head_range,
                    expected.head_spelling.as_str(),
                    expected.context,
                    expected.recovery,
                    expected.head
                )
            );
        }
        assert_eq!(composite.edges().len(), expected.edges.len());
        for ((_, row), expected) in composite.edges().iter().zip(&expected.edges) {
            assert_eq!(
                (row.parent(), row.ordinal(), row.role(), row.child()),
                (
                    expected.parent,
                    expected.ordinal,
                    expected.role,
                    expected.child
                )
            );
        }
        assert_eq!(composite.requests().len(), expected.requests.len());
        for ((_, row), expected) in composite.requests().iter().zip(&expected.requests) {
            assert_eq!(
                (
                    row.formula(),
                    row.ordinal(),
                    row.kind(),
                    row.binder(),
                    row.type_site()
                ),
                (
                    expected.formula,
                    expected.ordinal,
                    expected.kind,
                    expected.binder,
                    expected.type_site
                )
            );
        }
        assert_eq!(
            composition.atomic_edges().len(),
            expected_composition.atomic_edges.len()
        );
        for ((_, row), expected) in composition
            .atomic_edges()
            .iter()
            .zip(&expected_composition.atomic_edges)
        {
            assert_eq!(
                (row.formula(), row.ordinal(), row.role(), row.child()),
                (
                    expected.formula,
                    expected.ordinal,
                    expected.role,
                    expected.child
                )
            );
        }
        assert_eq!(
            composition.bound_uses().len(),
            expected_composition.bound_uses.len()
        );
        for ((_, row), expected) in composition
            .bound_uses()
            .iter()
            .zip(&expected_composition.bound_uses)
        {
            assert_eq!(
                (
                    row.binder(),
                    row.ordinal(),
                    row.body_edge(),
                    row.term(),
                    row.reference()
                ),
                (
                    expected.binder,
                    expected.ordinal,
                    expected.body_edge,
                    expected.term,
                    expected.reference
                )
            );
        }
        assert_eq!(
            composition.primary_term_fingerprint(),
            installed.source_term().unwrap().debug_text()
        );
        assert_eq!(
            composition.atomic_formula_fingerprint(),
            installed.source_atomic_formula().unwrap().debug_text()
        );
        assert_eq!(
            composition.composite_formula_fingerprint(),
            composite.debug_text()
        );
        assert_eq!(installed.clone(), installed);
        assert!(
            installed
                .clone()
                .with_source_formula_composition_from_original(source, symbols, scope)
                .is_err()
        );
        assert!(
            installed
                .clone()
                .with_source_formula_composition(composite.clone(), composition.clone())
                .is_err()
        );
        assert!(
            before
                .clone()
                .with_source_composite_formula(composite.clone())
                .is_err()
        );
        assert!(
            before
                .clone()
                .with_source_formula_composition(composite.clone(), composition.clone())
                .is_err()
        );
        let mut preinstalled_rows = before.clone();
        preinstalled_rows.install_source_nodes(source).unwrap();
        assert!(
            preinstalled_rows
                .with_source_formula_composition_from_original(source, symbols, scope)
                .is_err()
        );
        let foreign = &prepared[(case + 1) % prepared.len()];
        assert!(
            before
                .clone()
                .with_source_formula_composition_from_original(&foreign.0, &foreign.1, &foreign.2)
                .is_err()
        );
        assert!(
            before
                .clone()
                .with_source_formula_composition_from_original(source, &foreign.1, scope)
                .is_err()
        );
        assert!(
            before
                .clone()
                .with_source_formula_composition_from_original(source, symbols, &foreign.2)
                .is_err()
        );
        let collection = OverloadCollectionOutput::collect([], []);
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability = CandidateViabilityOutput::filter(&expansion, []);
        let specificity = SpecificityGraphOutput::build(&viability, []);
        let selection = OverloadSelectionOutput::resolve(&specificity, []);
        let facts = ClusterFactTable::new();
        let assemble = |typed, expressions| {
            ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
                typed_ast: typed,
                cluster_facts: &facts,
                overload_collection: &collection,
                template_expansion: &expansion,
                viability: &viability,
                specificity: &specificity,
                overload_selection: &selection,
                expressions,
                node_hints: vec![],
                statement_semantics: None,
                statement_proofs: None,
            })
        };
        let resolved = assemble(&installed, metadata.clone()).map_err(|error| error.to_string())?;
        let mut reversed = metadata.clone();
        reversed.reverse();
        assert_eq!(assemble(&installed, reversed).unwrap(), resolved);
        assert_eq!(resolved.clone(), resolved);
        assert_eq!(resolved.source_context(), installed.source_context());
        assert_eq!(resolved.source_type(), installed.source_type());
        assert_eq!(resolved.source_term(), installed.source_term());
        assert_eq!(
            resolved.source_atomic_formula(),
            installed.source_atomic_formula()
        );
        assert_eq!(resolved.source_composite_formula(), Some(composite));
        assert_eq!(resolved.source_formula_composition(), Some(composition));
        assert!(resolved.source_statement().is_none());
        assert!(resolved.source_statement_references().is_none());
        assert!(resolved.source_statement_witnesses().is_none());
        assert!(resolved.diagnostics().is_empty());
        assert_eq!(resolved.nodes().len(), installed.nodes().len());
        assert_eq!(resolved.expr_metadata().len(), metadata.len());
        for input in metadata {
            let row = resolved.expr_metadata().get_by_expr(&input.expr).unwrap();
            assert_eq!(row.typed_site, input.typed_site);
            assert_eq!(row.local_context, input.local_context);
            assert_eq!(row.cluster_facts, input.cluster_facts);
            assert_eq!(
                resolved.expr_metadata().get_by_site(&input.typed_site),
                Some(row)
            );
            let node = installed.nodes().node(input.typed_site.node()).unwrap();
            assert_eq!(node.anchor, SourceAnchor::Range(row.source_range));
            let expected_type = installed
                .types()
                .iter()
                .find(|(_, entry)| entry.owner == input.typed_site)
                .map(|(_, entry)| {
                    let TypeEntryActual::Known(actual) = entry.actual else {
                        panic!("non-Known final type")
                    };
                    actual
                });
            assert_eq!(row.final_type, expected_type);
            assert!(
                row.visible_facts.is_empty()
                    && row.overload.is_none()
                    && row.inserted_views.is_empty()
                    && row.diagnostics.is_empty()
            );
        }
        let mut duplicate = metadata.clone();
        duplicate.push(duplicate[0].clone());
        assert!(assemble(&installed, duplicate).is_err());
        let mut wrong_site = metadata.clone();
        wrong_site[0].typed_site =
            TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(usize::MAX));
        assert!(assemble(&installed, wrong_site).is_err());
        let mut wrong_context = metadata.clone();
        wrong_context[0].local_context = Some(mizar_checker::typed_ast::LocalTypeContextId::new(
            usize::MAX,
        ));
        assert!(assemble(&installed, wrong_context).is_err());
        assert!(before.source_node_entries().is_none());
        assert!(
            before.source_composite_formula().is_none()
                && before.source_formula_composition().is_none()
        );
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn task258_native_let_statements_preserve_source_intent_and_installed_replay() -> Result<(), String>
{
    use mizar_checker::{
        binding_env::{BinderIdentity, BindingContextOwner, BindingTypeSite},
        cluster_trace::ClusterFactTable,
        overload_resolution::*,
        resolved_typed_ast::{ResolvedTypedAst, ResolvedTypedAstInputs},
        source_atomic_formula::{
            SourceAtomicFormulaId, SourceAtomicFormulaKind, SourceAtomicTermTarget,
        },
        source_composite_formula::*,
        source_statement::*,
        source_type::{SourceTypeHead, SourceTypeProducer},
        type_checker::{CheckedStatementOwner, FormulaStatus, TermStatus, TypeHeadRef},
        typed_ast::{TypeEntryActual, TypeStatus, TypedAst, TypedAstParts, TypedSiteRef},
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::{ContributionKind, ExportStatus, NamespacePath, Visibility},
        names::{SourceVariableBindingKind, SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::{SignatureProjectionExtractor, source_proof_scope_maps},
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceAnchor,
        SourceInput, SourceOriginInput,
    };
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceQuantifierKind};

    // EN04 4.2/4.5/4.6/4.7.1, EN14 14.4.1/14.5.2/14.7.5,
    // EN15 15.2.1/15.4.1/15.10/15.11.1 and EN16 16.1/16.2/16.4.
    // Literal source inventories: owners, universals, header bindings, Let
    // bindings, Let statements and written types. Let is not a proposition.
    let fixture =
        include_str!("../../../../../mizar-checker/tests/testdata/native_universal_statement.miz");
    let old_lemma = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let cases = [
        (fixture.to_owned(), (1, 1, 1, 1, 1, 2)),
        (fixture.replace("for X being set holds X = X", "for Header being set holds Header = Header"), (1, 1, 1, 1, 1, 2)),
        (fixture.replace("let X be set;", "let Local be set;").replace("thus X = X", "thus Local = Local"), (1, 1, 1, 1, 1, 2)),
        (format!("\n:: native layout\n{}", fixture.replace("being set", "being\tset").replace("proof", "proof\n:: local declaration\n")), (1, 1, 1, 1, 1, 2)),
        ("theorem Pair: for A, Spare, B being set holds A = B proof let L, Unused, R be set; thus L = R; end;".into(), (1, 1, 3, 3, 1, 2)),
        ("theorem Groups: for A being set, Spare, B being set holds A = B proof let L be set, Unused, R be set; thus L = R; end;".into(), (1, 1, 3, 3, 1, 4)),
        ("theorem Separate: for A being set holds A = A proof let L be set; let Idle be set; let R be set; thus L = R; end;".into(), (1, 1, 1, 3, 3, 4)),
        ("theorem Nested: for A being set for Unused being set holds for B being set holds A = B proof let L be set; let Idle be set; let R be set; thus L = R; end;".into(), (1, 3, 3, 3, 3, 6)),
        (old_lemma.to_owned(), (2, 2, 2, 2, 2, 4)),
    ];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let collection = OverloadCollectionOutput::collect([], []);
    let expansion = TemplateExpansionOutput::expand(&collection);
    let viability = CandidateViabilityOutput::filter(&expansion, []);
    let specificity = SpecificityGraphOutput::build(&viability, []);
    let selection = OverloadSelectionOutput::resolve(&specificity, []);
    let facts = ClusterFactTable::new();
    let assemble = |typed: &TypedAst, expressions| {
        ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast: typed,
            cluster_facts: &facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &selection,
            expressions,
            node_hints: vec![],
            statement_semantics: None,
            statement_proofs: None,
        })
    };
    let mut prepared = Vec::new();
    // EVERY positive morph reaches actual frontend, resolver, SourceType, B5
    // and Resolved success before the first unsupported statement call.
    for (case, (text, counts)) in cases.iter().enumerate() {
        let (owners, universals, header_bindings, let_bindings, let_events, written_types) =
            *counts;
        let path = root.join(format!("src/native_let_{case}.miz"));
        fs::write(&path, text).unwrap();
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: snapshot_id(796),
                input: SourceInput {
                    package_id: PackageId::new("native-let-statements"),
                    module_path: ModulePath::new(format!("native_let_{case}")),
                    normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Disk { path },
                },
            },
            &ids,
        )
        .unwrap();
        assert!(
            frontend.diagnostics.is_empty(),
            "case {case}: {:?}",
            frontend.diagnostics
        );
        assert_eq!(frontend.source.source_text.as_ref(), text);
        let ast = frontend.ast.as_ref().unwrap();
        let module = ModuleId::new(
            frontend.source.package_id.clone(),
            frontend.source.module_path.clone(),
        );
        let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
        source.validate_against(ast, &module).unwrap();
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
        let lower = SourceTypeProducer::build_source_theorem_equalities(&source, symbols, &scope)?;
        let (env, inference, primitive, metadata) = &lower;
        let context = primitive.source_context().unwrap();
        let types = primitive.source_type().unwrap();
        let primary = primitive.source_term().unwrap();
        let atomic = primitive.source_atomic_formula().unwrap();
        assert_eq!(context.items().len(), owners);
        assert_eq!(context.declarations().len(), header_bindings + let_bindings);
        assert_eq!(env.bindings().len(), header_bindings + let_bindings);
        assert_eq!(scope.bindings().len(), header_bindings + let_bindings);
        assert_eq!(context.binding_env(), env);
        assert_eq!(env.contexts().len(), 1 + universals + owners);
        assert_eq!(primary.terms().len(), 4 * owners);
        assert_eq!(primary.references().len(), 4 * owners);
        assert_eq!(scope.references().len(), 4 * owners);
        assert_eq!(atomic.formulas().len(), 2 * owners);
        assert_eq!(types.applications().len(), header_bindings + let_bindings);
        assert_eq!(types.expressions().len(), written_types);
        assert_eq!(metadata.len(), 6 * owners);
        assert!(primitive.source_node_entries().is_none());
        assert!(primitive.source_statement().is_none());
        assert!(!primitive.contexts().is_empty() && !primitive.types().is_empty());
        assert_eq!(primitive.types(), inference.type_entries());
        assert!(env.diagnostics().is_empty() && inference.diagnostics().is_empty());
        assert!(inference.candidate_sets().is_empty() && inference.facts().is_empty());
        assert!(primitive.facts().is_empty() && primitive.coercions().is_empty());
        assert!(primitive.initial_obligations().is_empty() && primitive.diagnostics().is_empty());
        assert!(
            inference
                .terms()
                .iter()
                .all(|(_, row)| row.status == TermStatus::Inferred && row.deferred.is_empty())
        );
        assert!(
            inference
                .formulas()
                .iter()
                .all(|(_, row)| row.status == FormulaStatus::Checked && row.deferred.is_empty())
        );
        for (_, row) in primitive.types().iter() {
            assert_eq!(row.status, TypeStatus::Known);
            let TypeEntryActual::Known(actual) = row.actual else {
                panic!("non-Known lower type")
            };
            let actual = inference.normalized_types().get(actual).unwrap();
            assert_eq!(actual.head, TypeHeadRef::BuiltinSet);
            assert!(
                actual.args.is_empty()
                    && actual.attributes.positive().is_empty()
                    && actual.attributes.negative().is_empty()
            );
        }
        let maps = source_proof_scope_maps(&source, symbols, &scope)?;
        let typed_site = |id| {
            let resolved = source.resolved_node_for(id).unwrap();
            let matches = primitive
                .nodes()
                .iter()
                .filter(|(_, row)| row.resolved_node == Some(resolved))
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1);
            TypedSiteRef::Node(matches[0].0)
        };
        let spelling = |id| {
            let range = ast.node_view(id).unwrap().range();
            let mut tokens = ast
                .node_views()
                .filter(|view| {
                    view.as_token().is_some()
                        && range.start <= view.range().start
                        && view.range().end <= range.end
                })
                .collect::<Vec<_>>();
            tokens.sort_by_key(|view| view.range().start);
            tokens
                .iter()
                .map(|view| view.as_token().unwrap().text.as_ref())
                .collect::<Vec<_>>()
                .join(" ")
        };
        let body_context = |id| {
            let local = maps.0.get(&id).unwrap();
            let matches = env
                .contexts()
                .iter()
                .filter(|(_, row)| row.lexical_scope.as_ref() == Some(local))
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1);
            matches[0].0
        };
        // Authenticate every source declaration/type application and reference;
        // these joins are independent of the statement producer's output.
        for original in scope.bindings() {
            let identity = BinderIdentity::ResolverLocal {
                scope: original.scope().clone(),
                ordinal: original.ordinal(),
                declaration_range: original.range(),
            };
            let matches = env
                .bindings()
                .iter()
                .filter(|(_, row)| row.identity == identity)
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1);
            let (binding, entry) = matches[0];
            assert_eq!(entry.visible_after_ordinal, original.ordinal());
            let declarations = context
                .declarations()
                .iter()
                .filter(|(_, row)| row.binding == binding)
                .collect::<Vec<_>>();
            assert_eq!(declarations.len(), 1);
            let declaration = declarations[0].1;
            assert_eq!(declaration.site, typed_site(original.node()));
            assert_eq!(declaration.spelling, original.spelling());
            assert_eq!(declaration.source_ordinal, original.ordinal());
            assert_eq!(declaration.declaration_range, original.range());
            assert_eq!(declaration.binding_context, entry.owner_context);
            // The primitive binding slot stays untyped; SourceType owns the written type.
            assert_eq!(entry.type_site, BindingTypeSite::Missing);
            let applications = types
                .applications()
                .iter()
                .filter(|(_, row)| row.binding() == binding)
                .collect::<Vec<_>>();
            assert_eq!(applications.len(), 1);
            let application = applications[0].1;
            assert_eq!(application.source_ordinal(), original.ordinal());
            let written = types.expressions().get(application.root()).unwrap();
            let syntax = ast
                .node_views()
                .find(|view| {
                    view.kind() == &K::TypeExpression
                        && view.range() == declaration.written_type_range
                })
                .unwrap();
            let head = ast.node_view(syntax.children()[0]).unwrap();
            assert_eq!(written.site(), &typed_site(syntax.id()));
            assert_eq!(written.source_range(), syntax.range());
            assert_eq!(written.head_site(), &typed_site(head.id()));
            assert_eq!(written.head_range(), head.range());
            assert_eq!(written.head(), &SourceTypeHead::BuiltinSet);
            assert_eq!(spelling(syntax.id()), "set");
        }
        for (ordinal, original) in scope.references().iter().enumerate() {
            let (term_id, term) = primary
                .terms()
                .iter()
                .find(|(_, row)| row.site() == &typed_site(original.node()))
                .unwrap();
            let (_, reference) = primary
                .references()
                .iter()
                .find(|(_, row)| row.term() == term_id)
                .unwrap();
            let binding = &scope.bindings()[original.binding().index()];
            assert_eq!(term.source_range(), original.range());
            assert_eq!(term.source_ordinal(), ordinal);
            assert_eq!(term.spelling(), binding.spelling());
            assert_eq!(reference.use_ordinal(), original.ordinal());
            assert_eq!(reference.lexical_scope(), Some(original.scope()));
            assert_eq!(
                env.bindings().get(reference.binding()).unwrap().identity,
                BinderIdentity::ResolverLocal {
                    scope: binding.scope().clone(),
                    ordinal: binding.ordinal(),
                    declaration_range: binding.range()
                }
            );
        }
        let baseline = primitive
            .clone()
            .with_source_formula_composition_from_original(&source, symbols, &scope)
            .expect("unchanged genuine B5 prerequisite");
        let composite = baseline.source_composite_formula().unwrap();
        let composition = baseline.source_formula_composition().unwrap();
        assert_eq!(composite.binding_env(), env);
        assert_eq!(composite.source_id(), source.source_id());
        assert_eq!(composite.module_id(), source.module());
        assert_eq!(composition.source_id(), source.source_id());
        assert_eq!(composition.module_id(), source.module());
        assert_eq!(baseline.nodes(), primitive.nodes());
        assert_eq!(baseline.contexts(), primitive.contexts());
        assert_eq!(baseline.types(), primitive.types());
        assert_eq!(baseline.source_context(), primitive.source_context());
        assert_eq!(baseline.source_type(), primitive.source_type());
        assert_eq!(baseline.source_term(), primitive.source_term());
        assert_eq!(
            baseline.source_atomic_formula(),
            primitive.source_atomic_formula()
        );
        assert!(baseline.source_statement().is_none());
        let mut quantifiers = ast
            .node_views()
            .filter(|view| view.kind() == &K::QuantifiedFormula(SurfaceQuantifierKind::Universal))
            .collect::<Vec<_>>();
        quantifiers.sort_by_key(|view| view.range().start);
        assert_eq!(quantifiers.len(), universals);
        assert_eq!(composite.formulas().len(), universals);
        assert_eq!(composite.roots().len(), owners);
        assert_eq!(composite.binders().len(), header_bindings);
        assert_eq!(composite.type_sites().len(), header_bindings);
        assert_eq!(composite.edges().len(), universals - owners);
        assert_eq!(composite.requests().len(), universals + header_bindings);
        assert_eq!(composition.atomic_edges().len(), owners);
        assert_eq!(composition.bound_uses().len(), 2 * owners);
        // The preceding B5 primary owns its complete graph oracle. Here check
        // the actual joins that the new statement must preserve and target.
        for (index, syntax) in quantifiers.iter().enumerate() {
            let id = SourceCompositeFormulaId::new(index);
            let row = composite.formulas().get(id).unwrap();
            let body = body_context(syntax.id());
            let incoming = env.contexts().get(body).unwrap().parent.unwrap();
            assert_eq!(row.site(), &typed_site(syntax.id()));
            assert_eq!(row.source_range(), syntax.range());
            assert_eq!(row.context(), incoming);
            assert_eq!(
                env.contexts().get(body).unwrap().owner,
                BindingContextOwner::SourceFormula {
                    source_range: syntax.range()
                }
            );
            for (binder_id, binder) in composite
                .binders()
                .iter()
                .filter(|(_, binder)| binder.formula() == id)
            {
                let original = scope
                    .bindings()
                    .iter()
                    .find(|original| typed_site(original.node()) == *binder.identifier_site())
                    .unwrap();
                assert_eq!(original.kind(), SourceVariableBindingKind::Quantifier);
                assert_eq!(original.scope(), maps.0.get(&syntax.id()).unwrap());
                assert_eq!(binder.body_context(), body);
                let declaration = context
                    .declarations()
                    .iter()
                    .find(|(_, row)| row.site == typed_site(original.node()))
                    .unwrap()
                    .1;
                assert_eq!(binder.binding(), declaration.binding);
                let application = types
                    .applications()
                    .iter()
                    .find(|(_, row)| row.binding() == binder.binding())
                    .unwrap()
                    .1;
                let written = types.expressions().get(application.root()).unwrap();
                let ty = composite.type_sites().get(binder.type_site()).unwrap();
                assert_eq!(ty.binder(), binder_id);
                assert_eq!(ty.site(), written.site());
                assert_eq!(ty.source_range(), declaration.written_type_range);
                assert_eq!(ty.context(), incoming);
            }
            let child = ast.node_view(*syntax.children().last().unwrap()).unwrap();
            if child.kind() == &K::BuiltinPredicateApplication {
                let (_, edge) = composition
                    .atomic_edges()
                    .iter()
                    .find(|(_, edge)| edge.formula() == id)
                    .unwrap();
                let (atom_id, atom) = atomic
                    .formulas()
                    .iter()
                    .find(|(_, row)| row.site() == &typed_site(child.id()))
                    .unwrap();
                assert_eq!(edge.child(), atom_id);
                assert_eq!(atom.context(), body);
            }
        }
        assert_eq!(composition.primary_term_fingerprint(), primary.debug_text());
        assert_eq!(
            composition.atomic_formula_fingerprint(),
            atomic.debug_text()
        );
        assert_eq!(
            composition.composite_formula_fingerprint(),
            composite.debug_text()
        );
        let baseline_resolved = assemble(&baseline, metadata.clone())
            .map_err(|error| format!("case {case} B5 Resolved prerequisite: {error}"))?;
        assert_eq!(baseline_resolved.expr_metadata().len(), metadata.len());
        assert_eq!(
            baseline_resolved.source_formula_composition(),
            Some(composition)
        );
        assert!(baseline_resolved.source_statement().is_none());
        assert!(
            baseline_resolved.checked_formulas().is_empty()
                && baseline_resolved.statement_semantics().is_empty()
                && baseline_resolved.checked_proofs().is_empty()
        );
        // The old two-owner lemma fixture has genuine B5/Resolved success; it
        // cannot supply the single ordinary theorem seal for this new route.
        let expected = if owners == 2 {
            assert_eq!(text, old_lemma);
            assert_eq!(
                symbols
                    .symbols()
                    .iter()
                    .filter(|entry| entry.visibility() == Visibility::Public)
                    .count(),
                2
            );
            None
        } else {
            let theorem = ast
                .node_views()
                .find(|view| view.kind() == &K::TheoremItem)
                .unwrap();
            let label = ast.node_view(theorem.children()[1]).unwrap();
            let label_text = label.as_token().unwrap().text.as_ref();
            let symbol = symbols
                .symbols()
                .iter()
                .find(|entry| entry.primary_spelling() == label_text)
                .unwrap();
            let checked = CheckedStatementOwner::validate_exact_local_theorem(
                symbols,
                symbol.symbol().clone(),
                source.source_id(),
                source.module(),
            )
            .unwrap();
            assert_eq!(checked.source_range(), theorem.range());
            assert_eq!(checked.origin(), symbol.origin());
            assert_eq!(
                checked.origin().anchor(),
                &SourceAnchor::Range(theorem.range())
            );
            assert_eq!(checked.visibility(), Visibility::Public);
            assert_eq!(checked.export_status(), ExportStatus::Exported);
            let definition = symbols.definitions().by_symbol(symbol.symbol()).unwrap();
            assert_eq!(definition.origin(), symbol.origin());
            assert_eq!(definition.contribution(), symbol.contribution());
            let contribution = symbols.contributions().get(symbol.contribution()).unwrap();
            assert_eq!(contribution.module(), source.module());
            assert_eq!(
                contribution.kind(),
                &ContributionKind::LocalSource {
                    source_id: source.source_id()
                }
            );
            assert!(contribution.effects().symbols().contains(symbol.symbol()));
            assert!(
                contribution
                    .effects()
                    .definitions()
                    .contains(&definition.id())
            );
            assert_eq!(contribution.anchor(), &SourceAnchor::Range(theorem.range()));
            let proof = ast
                .node_views()
                .find(|view| view.kind() == &K::ProofBlock)
                .unwrap();
            let conclusion = ast
                .node_views()
                .find(|view| view.kind() == &K::ConclusionStatement)
                .unwrap();
            assert_eq!(
                ast.node_views()
                    .filter(|view| view.kind() == &K::LetStatement)
                    .count(),
                let_events
            );
            assert_eq!(
                ast.node_views()
                    .filter(|view| view.kind() == &K::ConclusionStatement)
                    .count(),
                1
            );
            let proof_context = body_context(proof.id());
            let module_context = env
                .contexts()
                .iter()
                .find(|(_, row)| row.owner == BindingContextOwner::Module)
                .unwrap()
                .0;
            assert_eq!(
                env.contexts().get(proof_context).unwrap().parent,
                Some(module_context)
            );
            assert!(
                env.contexts()
                    .get(module_context)
                    .unwrap()
                    .visible_bindings
                    .is_empty()
            );
            let conclusion_atom = atomic
                .formulas()
                .get(SourceAtomicFormulaId::new(1))
                .unwrap();
            assert_eq!(conclusion_atom.context(), proof_context);
            assert_eq!(conclusion_atom.kind(), SourceAtomicFormulaKind::Equality);
            assert!(
                conclusion.range().start <= conclusion_atom.source_range().start
                    && conclusion_atom.source_range().end <= conclusion.range().end
            );
            let mut input = SourceStatementHandoffInput {
                source_id: source.source_id(),
                module_id: source.module().clone(),
                owners: vec![SourceTheoremOwnerInput {
                    symbol: symbol.symbol().clone(),
                    contribution: symbol.contribution(),
                    site: typed_site(theorem.id()),
                    source_range: theorem.range(),
                    spelling: label_text.into(),
                    role: SourceTheoremRole::Theorem,
                    status: SourceTheoremStatus::Unmodified,
                    recovery: SourceStatementRecovery::Normal,
                }],
                statements: vec![],
                contexts: vec![],
                input_facts: vec![],
                candidate_facts: vec![],
            };
            for (index, (syntax, binding_context, formula, source_ordinal, kind)) in [
                (
                    theorem,
                    module_context,
                    SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0)),
                    0,
                    SourceStatementKind::TheoremProposition,
                ),
                (
                    conclusion,
                    proof_context,
                    SourceStatementFormulaTarget::Atomic(SourceAtomicFormulaId::new(1)),
                    1 + let_events,
                    SourceStatementKind::Conclusion,
                ),
            ]
            .into_iter()
            .enumerate()
            {
                let statement = SourceStatementId::new(index);
                let context = SourceStatementContextId::new(index);
                input.statements.push(SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context,
                    formula,
                    site: typed_site(syntax.id()),
                    source_range: syntax.range(),
                    source_ordinal,
                    spelling: spelling(syntax.id()),
                    kind,
                    recovery: SourceStatementRecovery::Normal,
                });
                input.contexts.push(SourceStatementContextInput {
                    statement,
                    binding_context,
                    source_range: syntax.range(),
                    visible_bindings: env
                        .contexts()
                        .get(binding_context)
                        .unwrap()
                        .visible_bindings
                        .clone(),
                });
                input
                    .candidate_facts
                    .push(SourceStatementCandidateFactInput {
                        statement,
                        context,
                        ordinal: 0,
                        kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                        formula,
                    });
            }
            let mut locals = scope
                .bindings()
                .iter()
                .filter(|binding| binding.kind() == SourceVariableBindingKind::Let)
                .collect::<Vec<_>>();
            locals.sort_by_key(|binding| binding.range().start);
            assert_eq!(locals.len(), let_bindings);
            for (ordinal, local) in locals.iter().enumerate() {
                let declaration = context
                    .declarations()
                    .iter()
                    .find(|(_, row)| row.site == typed_site(local.node()))
                    .unwrap()
                    .1;
                assert_eq!(declaration.binding_context, proof_context);
                let uses = primary
                    .references()
                    .iter()
                    .filter(|(_, reference)| reference.binding() == declaration.binding)
                    .map(|(id, reference)| {
                        let term = primary.terms().get(reference.term()).unwrap();
                        assert_eq!(term.context(), proof_context);
                        assert!(
                            conclusion.range().start <= term.source_range().start
                                && term.source_range().end <= conclusion.range().end
                        );
                        assert!(atomic.edges().iter().any(|(_, edge)| edge.formula()
                            == SourceAtomicFormulaId::new(1)
                            && edge.target() == SourceAtomicTermTarget::Primary(reference.term())));
                        id
                    })
                    .collect::<Vec<_>>();
                let expected_uses = scope
                    .references()
                    .iter()
                    .filter(|reference| reference.binding() == local.id())
                    .map(|reference| {
                        primary
                            .references()
                            .iter()
                            .find(|(_, row)| {
                                primary.terms().get(row.term()).unwrap().site()
                                    == &typed_site(reference.node())
                            })
                            .unwrap()
                            .0
                    })
                    .collect::<Vec<_>>();
                assert_eq!(uses, expected_uses);
                input.input_facts.push(SourceStatementInputFactInput {
                    statement: SourceStatementId::new(1),
                    context: SourceStatementContextId::new(1),
                    ordinal,
                    kind: SourceStatementInputFactKind::LetTypeGuard,
                    binding: declaration.binding,
                    uses,
                });
            }
            assert_eq!(
                input.contexts[1].visible_bindings,
                input
                    .input_facts
                    .iter()
                    .map(|row| row.binding)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                input
                    .input_facts
                    .iter()
                    .map(|row| row.uses.len())
                    .sum::<usize>(),
                2
            );
            assert!(input.input_facts.iter().all(|row| {
                composite
                    .binders()
                    .iter()
                    .all(|(_, binder)| binder.binding() != row.binding)
            }));
            assert_eq!(
                composite
                    .formulas()
                    .get(SourceCompositeFormulaId::new(0))
                    .unwrap()
                    .context(),
                module_context
            );
            if let_bindings == 3 {
                assert!(input.input_facts[1].uses.is_empty());
            }
            if case == 0 {
                assert_eq!(input.statements[1].source_ordinal, 2);
                assert_eq!(locals[0].ordinal(), 1);
                assert_eq!(
                    input.input_facts[0]
                        .uses
                        .iter()
                        .map(|id| primary.references().get(*id).unwrap().use_ordinal())
                        .collect::<Vec<_>>(),
                    vec![2, 3]
                );
                let header = composite
                    .binders()
                    .get(SourceQuantifierBinderId::new(0))
                    .unwrap();
                assert_eq!(header.identifier_spelling(), locals[0].spelling());
                assert_ne!(header.binding(), input.input_facts[0].binding);
            }
            if case == 4 || case == 5 {
                assert_eq!(input.statements[1].source_ordinal, 2);
                assert_eq!(
                    locals.iter().map(|row| row.ordinal()).collect::<Vec<_>>(),
                    vec![3, 4, 5]
                );
            }
            if case == 6 || case == 7 {
                assert_eq!(input.statements[1].source_ordinal, 4);
                assert_ne!(input.statements[1].source_ordinal, locals[0].ordinal());
            }
            Some((input, checked))
        };
        prepared.push((
            source,
            symbols.clone(),
            scope,
            lower,
            baseline,
            baseline_resolved,
            expected,
        ));
    }
    assert_eq!(prepared.len(), 9);
    for (case, (source, symbols, scope, lower, baseline, baseline_resolved, expected)) in
        prepared.iter().enumerate()
    {
        let (env, inference, primitive, metadata) = lower;
        let Some((expected, checked)) = expected else {
            assert!(
                baseline
                    .clone()
                    .with_source_statement_from_original(source, symbols, scope)
                    .is_err()
            );
            assert_eq!(
                assemble(baseline, metadata.clone()).unwrap(),
                *baseline_resolved
            );
            continue;
        };
        let before = (
            source.clone(),
            symbols.clone(),
            scope.clone(),
            lower.clone(),
            baseline.clone(),
        );
        let installed = baseline.clone().with_source_statement_from_original(source, symbols, scope)
            .expect("CHECKER258 native Let statement unsupported after all genuine SourceType/B5/Resolved prerequisites");
        let statement = installed.source_statement().unwrap();
        assert_eq!(statement.source_id(), expected.source_id);
        assert_eq!(statement.module_id(), &expected.module_id);
        assert_eq!(statement.checked_owner(), checked);
        assert_eq!(statement.binding_env(), env);
        assert_eq!(statement.binding_fingerprint(), env.debug_text());
        assert_eq!(
            statement.primary_term_fingerprint(),
            primitive.source_term().unwrap().debug_text()
        );
        assert_eq!(
            statement.atomic_formula_fingerprint(),
            primitive.source_atomic_formula().unwrap().debug_text()
        );
        assert_eq!(
            statement.composite_formula_fingerprint(),
            Some(
                baseline
                    .source_composite_formula()
                    .unwrap()
                    .debug_text()
                    .as_str()
            )
        );
        assert_eq!(
            statement.formula_composition_fingerprint(),
            Some(
                baseline
                    .source_formula_composition()
                    .unwrap()
                    .debug_text()
                    .as_str()
            )
        );
        assert_eq!(statement.owners().len(), 1);
        assert_eq!(statement.statements().len(), 2);
        assert_eq!(statement.contexts().len(), 2);
        assert_eq!(statement.input_facts().len(), expected.input_facts.len());
        assert_eq!(statement.candidate_facts().len(), 2);
        for ((id, row), expected) in statement.owners().iter().zip(&expected.owners) {
            assert_eq!(id, SourceTheoremOwnerId::new(0));
            assert_eq!(row.symbol(), &expected.symbol);
            assert_eq!(row.contribution(), expected.contribution);
            assert_eq!(row.site(), &expected.site);
            assert_eq!(row.source_range(), expected.source_range);
            assert_eq!(row.spelling(), expected.spelling.as_str());
            assert_eq!(row.role(), expected.role);
            assert_eq!(row.status(), expected.status);
            assert_eq!(row.recovery(), expected.recovery);
            let symbol = symbols.symbols().get(row.symbol()).unwrap();
            assert_eq!(symbol.origin(), checked.origin());
            assert_eq!(row.contribution(), symbol.contribution());
        }
        for (index, ((id, row), expected)) in statement
            .statements()
            .iter()
            .zip(&expected.statements)
            .enumerate()
        {
            assert_eq!(id, SourceStatementId::new(index));
            assert_eq!(row.owner(), expected.owner);
            assert_eq!(row.context(), expected.context);
            assert_eq!(row.formula(), expected.formula);
            assert_eq!(row.site(), &expected.site);
            assert_eq!(row.source_range(), expected.source_range);
            assert_eq!(row.source_ordinal(), expected.source_ordinal);
            assert_eq!(row.spelling(), expected.spelling.as_str());
            assert_eq!(row.kind(), expected.kind);
            assert_eq!(row.recovery(), expected.recovery);
        }
        for (index, ((id, row), expected)) in statement
            .contexts()
            .iter()
            .zip(&expected.contexts)
            .enumerate()
        {
            assert_eq!(id, SourceStatementContextId::new(index));
            assert_eq!(row.statement(), expected.statement);
            assert_eq!(row.binding_context(), expected.binding_context);
            assert_eq!(row.source_range(), expected.source_range);
            assert_eq!(row.visible_bindings(), expected.visible_bindings.as_slice());
        }
        for (index, ((id, row), expected)) in statement
            .input_facts()
            .iter()
            .zip(&expected.input_facts)
            .enumerate()
        {
            assert_eq!(id, SourceStatementInputFactId::new(index));
            assert_eq!(row.statement(), expected.statement);
            assert_eq!(row.context(), expected.context);
            assert_eq!(row.ordinal(), expected.ordinal);
            assert_eq!(row.kind(), expected.kind);
            assert_eq!(row.binding(), expected.binding);
            assert_eq!(row.uses(), expected.uses.as_slice());
            let declaration = installed
                .source_context()
                .unwrap()
                .declarations()
                .iter()
                .find(|(_, declaration)| declaration.binding == row.binding())
                .unwrap()
                .1;
            let application = installed
                .source_type()
                .unwrap()
                .applications()
                .iter()
                .find(|(_, application)| application.binding() == row.binding())
                .unwrap()
                .1;
            assert_eq!(application.source_ordinal(), declaration.source_ordinal);
            assert_eq!(
                installed
                    .source_type()
                    .unwrap()
                    .expressions()
                    .get(application.root())
                    .unwrap()
                    .source_range(),
                declaration.written_type_range
            );
        }
        assert!(
            !statement
                .input_facts()
                .iter()
                .any(|(_, row)| row.statement() == SourceStatementId::new(0))
        );
        for (index, ((id, row), expected)) in statement
            .candidate_facts()
            .iter()
            .zip(&expected.candidate_facts)
            .enumerate()
        {
            assert_eq!(id, SourceStatementCandidateFactId::new(index));
            assert_eq!(row.statement(), expected.statement);
            assert_eq!(row.context(), expected.context);
            assert_eq!(row.ordinal(), expected.ordinal);
            assert_eq!(row.kind(), expected.kind);
            assert_eq!(row.formula(), expected.formula);
        }
        assert_eq!(installed.nodes(), baseline.nodes());
        assert_eq!(installed.contexts(), baseline.contexts());
        assert_eq!(installed.types(), baseline.types());
        assert_eq!(installed.types(), inference.type_entries());
        assert_eq!(installed.source_context(), baseline.source_context());
        assert_eq!(installed.source_type(), baseline.source_type());
        assert_eq!(installed.source_term(), baseline.source_term());
        assert_eq!(
            installed.source_atomic_formula(),
            baseline.source_atomic_formula()
        );
        assert_eq!(
            installed.source_composite_formula(),
            baseline.source_composite_formula()
        );
        assert_eq!(
            installed.source_formula_composition(),
            baseline.source_formula_composition()
        );
        assert_eq!(
            installed
                .source_node_entries()
                .unwrap()
                .cloned()
                .collect::<Vec<_>>(),
            baseline
                .source_node_entries()
                .unwrap()
                .cloned()
                .collect::<Vec<_>>()
        );
        assert_eq!(installed.facts(), baseline.facts());
        assert_eq!(installed.coercions(), baseline.coercions());
        assert_eq!(
            installed.initial_obligations(),
            baseline.initial_obligations()
        );
        assert_eq!(installed.diagnostics(), baseline.diagnostics());
        assert!(
            installed.source_statement_references().is_none()
                && installed.source_statement_witnesses().is_none()
        );
        assert!(
            installed.source_proof_local_binding().is_none()
                && installed.source_proof_local_let_binding().is_none()
                && installed.source_proof_local_let_type().is_none()
        );
        let fresh = SourceTypeProducer::build_source_theorem_equalities(source, symbols, scope)?
            .2
            .with_source_formula_composition_from_original(source, symbols, scope)
            .unwrap()
            .with_source_statement_from_original(source, symbols, scope)
            .unwrap();
        assert_eq!(fresh, installed);
        let resolved = assemble(&installed, metadata.clone())
            .map_err(|error| format!("case {case} installed replay: {error}"))?;
        assert_eq!(assemble(&fresh, metadata.clone()).unwrap(), resolved);
        let mut reversed = metadata.clone();
        reversed.reverse();
        assert_eq!(assemble(&installed, reversed).unwrap(), resolved);
        assert_eq!(resolved.source_statement(), Some(statement));
        assert_eq!(resolved.source_context(), installed.source_context());
        assert_eq!(resolved.source_type(), installed.source_type());
        assert_eq!(resolved.source_term(), installed.source_term());
        assert_eq!(
            resolved.source_atomic_formula(),
            installed.source_atomic_formula()
        );
        assert_eq!(
            resolved.source_composite_formula(),
            installed.source_composite_formula()
        );
        assert_eq!(
            resolved.source_formula_composition(),
            installed.source_formula_composition()
        );
        assert_eq!(resolved.nodes(), baseline_resolved.nodes());
        assert_eq!(resolved.expr_metadata(), baseline_resolved.expr_metadata());
        assert_eq!(resolved.expr_metadata().len(), metadata.len());
        for input in metadata {
            let row = resolved.expr_metadata().get_by_expr(&input.expr).unwrap();
            assert_eq!(row.typed_site.clone(), input.typed_site.clone());
            assert_eq!(row.local_context, input.local_context);
            assert_eq!(row.cluster_facts.clone(), input.cluster_facts.clone());
            assert_eq!(
                resolved.expr_metadata().get_by_site(&input.typed_site),
                Some(row)
            );
            let node = installed.nodes().node(input.typed_site.node()).unwrap();
            assert_eq!(node.anchor, SourceAnchor::Range(row.source_range));
            assert_eq!(node.links.context, input.local_context);
            let expected_type = installed
                .types()
                .iter()
                .find(|(_, entry)| entry.owner == input.typed_site)
                .map(|(_, entry)| {
                    let TypeEntryActual::Known(actual) = entry.actual else {
                        panic!("non-Known final type")
                    };
                    actual
                });
            assert_eq!(row.final_type, expected_type);
            assert!(
                row.visible_facts.is_empty()
                    && row.overload.is_none()
                    && row.inserted_views.is_empty()
                    && row.diagnostics.is_empty()
            );
        }
        assert!(
            resolved.checked_formulas().is_empty()
                && resolved.statement_semantics().is_empty()
                && resolved.checked_proofs().is_empty()
        );
        assert!(
            resolved.checked_proof_nodes().is_empty()
                && resolved.checked_terminal_goals().is_empty()
        );
        assert!(
            resolved.cluster_facts().is_empty()
                && resolved.inserted_coercions().is_empty()
                && resolved.diagnostics().is_empty()
        );
        assert!(
            resolved.source_statement_references().is_none()
                && resolved.source_statement_witnesses().is_none()
        );
        assert_eq!(resolved.clone(), resolved);
        // Every refusal starts from a genuine nonempty installed transaction.
        assert!(
            installed
                .clone()
                .with_source_statement_from_original(source, symbols, scope)
                .is_err()
        );
        assert!(
            installed
                .clone()
                .with_source_statement(statement.clone())
                .is_err()
        );
        assert!(
            baseline
                .clone()
                .with_source_statement(statement.clone())
                .is_err()
        );
        assert!(
            primitive
                .clone()
                .with_source_statement_from_original(source, symbols, scope)
                .is_err()
        );
        assert!(
            primitive
                .clone()
                .with_source_statement(statement.clone())
                .is_err()
        );
        assert!(
            primitive
                .clone()
                .with_source_formula_composition_statement(
                    baseline.source_composite_formula().unwrap().clone(),
                    baseline.source_formula_composition().unwrap().clone(),
                    statement.clone()
                )
                .is_err()
        );
        let mut rows_only = primitive.clone();
        rows_only.install_source_nodes(source).unwrap();
        assert!(
            rows_only
                .with_source_statement_from_original(source, symbols, scope)
                .is_err()
        );
        let detached = TypedAst::try_new(TypedAstParts {
            source_id: source.source_id(),
            module_id: source.module().clone(),
            resolved_root: primitive.resolved_root(),
            source_context: primitive.source_context().cloned(),
            source_type: primitive.source_type().cloned(),
            source_attribute: None,
            nodes: primitive.nodes().clone(),
            contexts: primitive.contexts().clone(),
            types: primitive.types().clone(),
            facts: primitive.facts().clone(),
            coercions: primitive.coercions().clone(),
            initial_obligations: primitive.initial_obligations().clone(),
            diagnostics: primitive.diagnostics().clone(),
        });
        assert!(
            detached
                .unwrap()
                .with_source_statement(statement.clone())
                .is_err()
        );
        let foreign = &prepared[(case + 1) % 8];
        assert!(
            baseline
                .clone()
                .with_source_statement_from_original(&foreign.0, &foreign.1, &foreign.2)
                .is_err()
        );
        assert!(
            foreign
                .4
                .clone()
                .with_source_statement_from_original(source, symbols, scope)
                .is_err()
        );
        assert!(
            baseline
                .clone()
                .with_source_statement_from_original(source, &foreign.1, scope)
                .is_err()
        );
        assert!(
            baseline
                .clone()
                .with_source_statement_from_original(source, symbols, &foreign.2)
                .is_err()
        );
        let mut duplicate = metadata.clone();
        duplicate.push(duplicate[0].clone());
        assert!(assemble(&installed, duplicate).is_err());
        let mut wrong_site = metadata.clone();
        wrong_site[0].typed_site =
            TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(usize::MAX));
        assert!(assemble(&installed, wrong_site).is_err());
        let mut wrong_context = metadata.clone();
        wrong_context[0].local_context = Some(mizar_checker::typed_ast::LocalTypeContextId::new(
            usize::MAX,
        ));
        assert!(assemble(&installed, wrong_context).is_err());
        assert_eq!(assemble(&installed, metadata.clone()).unwrap(), resolved);
        assert_eq!(
            (
                source.clone(),
                symbols.clone(),
                scope.clone(),
                lower.clone(),
                baseline.clone()
            ),
            before
        );
        assert!(baseline.source_statement().is_none());
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn task257b6_native_connective_lower_preserves_genuine_tables() -> Result<(), String> {
    use mizar_checker::{
        binding_env::{BinderIdentity, BindingContextOwner, BindingTypeSite},
        cluster_trace::ClusterFactTable,
        overload_resolution::*,
        resolved_typed_ast::{ResolvedTypedAst, ResolvedTypedAstInputs},
        source_atomic_formula::*,
        source_term::{
            SourcePrimaryTermKind, SourcePrimaryTermRecovery, SourcePrimaryTermReferenceRole,
            SourcePrimaryTermRole,
        },
        source_type::{SourceTypeHead, SourceTypeProducer},
        type_checker::{FormulaStatus, TermStatus, TypeHeadRef},
        typed_ast::{TypeEntryActual, TypeStatus, TypedSiteRef},
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
        symbols::{SignatureProjectionExtractor, source_proof_scope_maps},
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceInput,
        SourceOriginInput,
    };
    use mizar_syntax::SurfaceNodeKind as K;

    let fixture =
        include_str!("../../../../../mizar-checker/tests/testdata/native_connective_statement.miz");
    let old =
        include_str!("../../../../../mizar-checker/tests/testdata/native_universal_statement.miz");
    let theorem = |body: &str| {
        format!("theorem Shape: for x being set holds {body} proof let x be set; thus x = x; end;")
    };
    let mut cases = vec![old.to_owned(), fixture.to_owned(),
        fixture.replace("NativeConnectives", "Renamed").replace('x', "outer").replace('y', "inner"),
        format!("\n:: layout\n{}", fixture.replace("being set", "being\tset").replace("proof", "proof\n:: proof scope\n")),
        theorem("x = x & x = x"), theorem("x = x or x = x"),
        theorem("x = x implies x = x"), theorem("x = x iff x = x"), theorem("not x = x"),
        theorem("x = x & x = x & x = x"), theorem("x = x or x = x or x = x"),
        theorem("x = x implies x = x implies x = x"),
        theorem("(not x = x) & (not x = x)"),
        theorem("(not x = x) or (not x = x)"),
        theorem("(not x = x) implies (not x = x)"),
        theorem("not (((x = x & ((x = x)))))"),
        theorem("x = x & (for x being set holds x = x or x = x)"),
        "theorem Unused: for x, spare being set, idle being set holds ((x = x or x = x)) proof let x, unused be set, idle be set; thus x = x; end;".to_owned(),
    ];
    // All frontend/resolver prerequisites, including excluded source shapes,
    // finish before the first new lower call. These are genuine disk sources.
    let positive_count = cases.len();
    let nested_existential = theorem("ex y being set st y = x");
    let restricted_universal = "theorem Restricted: for x being set st x = x holds x = x proof let x be set; thus x = x; end;".to_owned();
    cases.extend([
        theorem("contradiction"), theorem("thesis"), theorem("x = x & ... & x = x"), theorem("x = x or ... or x = x"),
        nested_existential.clone(), theorem("1 = 1"),
        restricted_universal.clone(),
        "theorem Object: for x being object holds x = x & x = x proof let x be object; thus x = x; end;".into(),
        theorem("x = x & x = x").replace("thus x = x;", "thus (x = x);"),
        theorem("x = x & x = x").replace("thus x = x;", "thus x = x & x = x;"),
        "theorem Recovered: for x being set holds (x = x & x = x proof let x be set; thus x = x; end;".into(),
    ]);
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let mut prepared = Vec::new();
    let mut recovered = None;
    for (case, text) in cases.iter().enumerate() {
        let path = root.join(format!("src/connective_lower_{case}.miz"));
        fs::write(&path, text).unwrap();
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: snapshot_id(797),
                input: SourceInput {
                    package_id: PackageId::new("native-connective-lower"),
                    module_path: ModulePath::new(format!("lower_{case}")),
                    normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Disk { path },
                },
            },
            &ids,
        )
        .unwrap();
        let is_recovered = case == cases.len() - 1;
        assert_eq!(
            frontend.diagnostics.is_empty(),
            !is_recovered,
            "case {case}: {:?}",
            frontend.diagnostics
        );
        let ast = frontend.ast.as_ref().unwrap();
        let module = ModuleId::new(
            frontend.source.package_id.clone(),
            frontend.source.module_path.clone(),
        );
        let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
        source.validate_against(ast, &module).unwrap();
        let shells = DeclarationShellCollector::new(ast, &module).collect();
        let collected = SignatureProjectionExtractor::new(
            ast,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        if !is_recovered {
            assert!(collected.diagnostics().is_empty());
        }
        let symbols = collected.env().clone();
        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(ast, &module, &symbols),
        );
        if is_recovered {
            assert!(
                ast.node_views()
                    .any(|view| matches!(view.kind(), K::ErrorRecovery(_)))
            );
            recovered = Some((source, symbols, scope));
            continue;
        }
        let scope = scope.unwrap();
        prepared.push((frontend, source, symbols, scope));
    }
    let collection = OverloadCollectionOutput::collect([], []);
    let expansion = TemplateExpansionOutput::expand(&collection);
    let viability = CandidateViabilityOutput::filter(&expansion, []);
    let specificity = SpecificityGraphOutput::build(&viability, []);
    let selection = OverloadSelectionOutput::resolve(&specificity, []);
    let facts = ClusterFactTable::new();
    for (case, (frontend, source, symbols, scope)) in prepared.iter().enumerate() {
        if case >= positive_count {
            let result =
                SourceTypeProducer::build_source_theorem_equalities(source, symbols, scope);
            let expected = if cases[case] == nested_existential {
                Some((4, 2, 3, 4))
            } else if cases[case] == restricted_universal || case == positive_count + 7 {
                Some((6, 3, 2, 3))
            } else {
                None
            };
            if let Some((terms, formulas, bindings, contexts)) = expected {
                let (env, inference, typed, metadata) = result.unwrap();
                assert_eq!(typed.source_term().unwrap().terms().len(), terms);
                assert_eq!(inference.terms().len(), terms);
                assert_eq!(
                    typed.source_atomic_formula().unwrap().formulas().len(),
                    formulas
                );
                assert_eq!(inference.formulas().len(), formulas);
                assert_eq!(env.bindings().len(), bindings);
                assert_eq!(env.contexts().len(), contexts);
                assert_eq!(
                    typed.source_context().unwrap().declarations().len(),
                    bindings
                );
                assert_eq!(typed.source_type().unwrap().applications().len(), bindings);
                assert_eq!(metadata.len(), terms + formulas);
                assert!(inference.facts().is_empty() && typed.facts().is_empty());
                if case == positive_count + 7 {
                    let ast = frontend.ast.as_ref().unwrap();
                    let physical = ast
                        .node_views()
                        .filter(|row| row.kind() == &K::TypeExpression)
                        .collect::<Vec<_>>();
                    assert_eq!(physical.len(), 2);
                    for (_, row) in typed.source_type().unwrap().expressions().iter() {
                        assert_eq!(row.head(), &SourceTypeHead::BuiltinObject);
                        let syntax = physical
                            .iter()
                            .find(|syntax| syntax.range() == row.source_range())
                            .unwrap();
                        let head = ast.node_view(syntax.children()[0]).unwrap();
                        assert_eq!(row.head_range(), head.range());
                        assert_eq!(
                            ast.node_view(head.children()[0])
                                .unwrap()
                                .as_token()
                                .unwrap()
                                .text
                                .as_ref(),
                            "object"
                        );
                    }
                    for (_, row) in inference.type_entries().iter() {
                        let TypeEntryActual::Known(id) = row.actual else {
                            panic!("known Object type")
                        };
                        assert_eq!(
                            inference.normalized_types().get(id).unwrap().head,
                            TypeHeadRef::BuiltinObject
                        );
                    }
                    for (_, row) in typed.source_context().unwrap().declarations().iter() {
                        assert!(
                            physical
                                .iter()
                                .any(|syntax| syntax.range() == row.written_type_range)
                        );
                        assert_eq!(
                            env.bindings().get(row.binding).unwrap().owner_context,
                            row.binding_context
                        );
                    }
                    let composed = typed
                        .clone()
                        .with_source_formula_composition_from_original(source, symbols, scope)
                        .expect("RED: genuine Object connective composition");
                    let graph = composed.source_composite_formula().unwrap();
                    let pair = composed.source_formula_composition().unwrap();
                    assert_eq!(
                        (
                            graph.formulas().len(),
                            graph.binders().len(),
                            graph.type_sites().len(),
                            graph.edges().len(),
                            pair.atomic_edges().len(),
                            pair.bound_uses().len()
                        ),
                        (2, 1, 1, 1, 2, 4)
                    );
                    assert_eq!(
                        format!("{:?}", graph.type_sites().iter().next().unwrap().1.head()),
                        "BuiltinObject"
                    );
                }

                assert!(
                    typed.source_composite_formula().is_none()
                        && typed.source_formula_composition().is_none()
                        && typed.source_statement().is_none()
                );
            } else {
                assert_eq!(
                    result.unwrap_err(),
                    "theorems.equality.unsupported_source_types",
                    "negative {case}"
                );
            }
            continue;
        }
        let ast = frontend.ast.as_ref().unwrap();
        let lower = SourceTypeProducer::build_source_theorem_equalities(source, symbols, scope)
            .unwrap_or_else(|error| panic!("RED-L genuine case {case}: {error}"));
        let (env, inference, typed, metadata) = &lower;
        let context = typed.source_context().unwrap();
        let types = typed.source_type().unwrap();
        let primary = typed.source_term().unwrap();
        let atomic = typed.source_atomic_formula().unwrap();
        let maps = source_proof_scope_maps(source, symbols, scope)?;
        let site = |id| {
            let resolved = source.resolved_node_for(id).unwrap();
            let matches = typed
                .nodes()
                .iter()
                .filter(|(_, row)| row.resolved_node == Some(resolved))
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1);
            TypedSiteRef::Node(matches[0].0)
        };
        let spelling = |id| {
            let range = ast.node_view(id).unwrap().range();
            let mut tokens = ast
                .node_views()
                .filter(|view| {
                    view.as_token().is_some()
                        && range.start <= view.range().start
                        && view.range().end <= range.end
                })
                .collect::<Vec<_>>();
            tokens.sort_by_key(|view| view.range().start);
            tokens
                .iter()
                .map(|view| view.as_token().unwrap().text.as_ref())
                .collect::<Vec<_>>()
                .join(" ")
        };
        let lexical = |id| {
            let mut cursor = id;
            loop {
                if let Some(local) = maps.0.get(&cursor) {
                    return env
                        .contexts()
                        .iter()
                        .find(|(_, row)| row.lexical_scope.as_ref() == Some(local))
                        .unwrap()
                        .0;
                }
                cursor = ast
                    .node_views()
                    .find(|view| view.children().contains(&cursor))
                    .unwrap()
                    .id();
            }
        };
        assert_eq!(context.binding_env(), env);
        assert_eq!(context.source_id(), source.source_id());
        assert_eq!(context.module_id(), source.module());
        assert_eq!(types.source_id(), source.source_id());
        assert_eq!(types.module_id(), source.module());
        assert_eq!(primary.source_id(), source.source_id());
        assert_eq!(primary.module_id(), source.module());
        assert_eq!(atomic.source_id(), source.source_id());
        assert_eq!(atomic.module_id(), source.module());
        assert_eq!(typed.contexts(), context.local_contexts());
        assert_eq!(context.context_links().len(), env.contexts().len());
        for (id, _) in env.contexts().iter() {
            let link = context.context_links().get(id).unwrap();
            assert_eq!(link.binding_context, id);
            assert!(typed.contexts().get(link.local_context).is_some());
        }
        assert_eq!(context.items().len(), 1);
        assert_eq!(context.declarations().len(), scope.bindings().len());
        assert_eq!(env.bindings().len(), scope.bindings().len());
        assert_eq!(types.applications().len(), scope.bindings().len());
        assert_eq!(
            types.expressions().len(),
            ast.node_views()
                .filter(|view| view.kind() == &K::TypeExpression)
                .count()
        );
        assert!(
            types.arguments().is_empty()
                && types.definition_returns().is_empty()
                && types.mode_rhs().is_empty()
                && types.structure_members().is_empty()
        );
        assert_eq!(
            env.contexts().len(),
            2 + ast
                .node_views()
                .filter(|view| matches!(view.kind(), K::QuantifiedFormula(_)))
                .count()
        );
        assert_eq!(primary.terms().len(), scope.references().len());
        assert_eq!(primary.references().len(), scope.references().len());
        assert!(primary.numeric_type_requests().is_empty());
        assert!(typed.source_node_entries().is_none());
        assert!(
            typed.source_composite_formula().is_none()
                && typed.source_formula_composition().is_none()
                && typed.source_statement().is_none()
        );
        assert!(env.diagnostics().is_empty() && inference.diagnostics().is_empty());
        assert!(inference.candidate_sets().is_empty() && inference.facts().is_empty());
        assert!(
            typed.facts().is_empty()
                && typed.coercions().is_empty()
                && typed.initial_obligations().is_empty()
                && typed.diagnostics().is_empty()
        );
        assert_eq!(typed.types(), inference.type_entries());
        assert!(
            inference
                .terms()
                .iter()
                .all(|(_, row)| row.status == TermStatus::Inferred && row.deferred.is_empty())
        );
        assert!(
            inference
                .formulas()
                .iter()
                .all(|(_, row)| row.status == FormulaStatus::Checked && row.deferred.is_empty())
        );
        for (_, entry) in typed.types().iter() {
            assert_eq!(entry.status, TypeStatus::Known);
            let TypeEntryActual::Known(actual) = entry.actual else {
                panic!("unknown Set type")
            };
            let actual = inference.normalized_types().get(actual).unwrap();
            assert_eq!(actual.head, TypeHeadRef::BuiltinSet);
            assert!(
                actual.args.is_empty()
                    && actual.attributes.positive().is_empty()
                    && actual.attributes.negative().is_empty()
            );
        }
        for original in scope.bindings() {
            let identity = BinderIdentity::ResolverLocal {
                scope: original.scope().clone(),
                ordinal: original.ordinal(),
                declaration_range: original.range(),
            };
            let matches = env
                .bindings()
                .iter()
                .filter(|(_, row)| row.identity == identity)
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1);
            let (binding, entry) = matches[0];
            assert_eq!(entry.type_site, BindingTypeSite::Missing);
            assert_eq!(entry.visible_after_ordinal, original.ordinal());
            let declaration = context
                .declarations()
                .iter()
                .find(|(_, row)| row.binding == binding)
                .unwrap()
                .1;
            assert_eq!(declaration.site, site(original.node()));
            assert_eq!(declaration.spelling, original.spelling());
            assert_eq!(declaration.source_ordinal, original.ordinal());
            assert_eq!(declaration.declaration_range, original.range());
            assert_eq!(declaration.binding_context, entry.owner_context);
            let application = types
                .applications()
                .iter()
                .find(|(_, row)| row.binding() == binding)
                .unwrap()
                .1;
            assert_eq!(application.source_ordinal(), original.ordinal());
            let written = types.expressions().get(application.root()).unwrap();
            let syntax = ast
                .node_views()
                .find(|view| {
                    view.kind() == &K::TypeExpression
                        && view.range() == declaration.written_type_range
                })
                .unwrap();
            let head = ast.node_view(syntax.children()[0]).unwrap();
            assert_eq!(written.site(), &site(syntax.id()));
            assert_eq!(written.source_range(), syntax.range());
            assert_eq!(written.head_site(), &site(head.id()));
            assert_eq!(written.head_range(), head.range());
            assert_eq!(written.head(), &SourceTypeHead::BuiltinSet);
            assert_eq!(spelling(syntax.id()), "set");
        }
        for (ordinal, original) in scope.references().iter().enumerate() {
            let (term_id, term) = primary
                .terms()
                .iter()
                .find(|(_, row)| row.site() == &site(original.node()))
                .unwrap();
            let (_, reference) = primary
                .references()
                .iter()
                .find(|(_, row)| row.term() == term_id)
                .unwrap();
            let winner = &scope.bindings()[original.binding().index()];
            assert_eq!(term.source_range(), original.range());
            assert_eq!(term.source_ordinal(), ordinal);
            assert_eq!(term.spelling(), winner.spelling());
            assert_eq!(term.kind(), SourcePrimaryTermKind::VariableReference);
            assert_eq!(term.role(), SourcePrimaryTermRole::Value);
            assert_eq!(term.recovery(), SourcePrimaryTermRecovery::Normal);
            assert!(term.parent().is_none());
            assert_eq!(term.context(), lexical(original.node()));
            assert_eq!(reference.use_ordinal(), original.ordinal());
            assert_eq!(reference.role(), SourcePrimaryTermReferenceRole::Variable);
            assert_eq!(reference.lexical_scope(), Some(original.scope()));
            assert_eq!(
                env.bindings().get(reference.binding()).unwrap().identity,
                BinderIdentity::ResolverLocal {
                    scope: winner.scope().clone(),
                    ordinal: winner.ordinal(),
                    declaration_range: winner.range()
                }
            );
        }
        let mut atoms = ast
            .node_views()
            .filter(|view| view.kind() == &K::BuiltinPredicateApplication)
            .collect::<Vec<_>>();
        atoms.sort_by_key(|view| view.range().start);
        assert_eq!(atomic.formulas().len(), atoms.len());
        assert_eq!(atomic.edges().len(), 2 * atoms.len());
        assert_eq!(atomic.requests().len(), 2 * atoms.len());
        assert!(
            atomic.predicate_segments().is_empty()
                && atomic.predicate_heads().is_empty()
                && atomic.candidates().is_empty()
                && atomic.type_sites().is_empty()
                && atomic.attributes().is_empty()
        );
        assert_eq!(atomic.primary_term_fingerprint(), primary.debug_text());
        assert!(
            atomic.application_fingerprint().is_none()
                && atomic.structure_fingerprint().is_none()
                && atomic.set_term_fingerprint().is_none()
        );
        for (index, syntax) in atoms.iter().enumerate() {
            let id = SourceAtomicFormulaId::new(index);
            let row = atomic.formulas().get(id).unwrap();
            assert_eq!(row.site(), &site(syntax.id()));
            assert_eq!(row.source_range(), syntax.range());
            assert_eq!(row.source_ordinal(), index);
            assert_eq!(row.context(), lexical(syntax.id()));
            assert_eq!(row.recovery(), SourceAtomicFormulaRecovery::Normal);
            assert_eq!(row.spelling(), spelling(syntax.id()));
            assert_eq!(row.kind(), SourceAtomicFormulaKind::Equality);
            for (ordinal, child) in [syntax.children()[0], syntax.children()[2]]
                .into_iter()
                .enumerate()
            {
                let reference_node = ast.node_view(child).unwrap().children()[0];
                let target = primary
                    .terms()
                    .iter()
                    .find(|(_, row)| row.site() == &site(reference_node))
                    .unwrap()
                    .0;
                let edge_id = SourceAtomicEdgeId::new(2 * index + ordinal);
                let edge = atomic.edges().get(edge_id).unwrap();
                assert_eq!(
                    (edge.formula(), edge.ordinal(), edge.role(), edge.target()),
                    (
                        id,
                        ordinal,
                        if ordinal == 0 {
                            SourceAtomicEdgeRole::BuiltinLeftOperand
                        } else {
                            SourceAtomicEdgeRole::BuiltinRightOperand
                        },
                        SourceAtomicTermTarget::Primary(target)
                    )
                );
                let request = atomic
                    .requests()
                    .get(SourceAtomicRequestId::new(2 * index + ordinal))
                    .unwrap();
                assert_eq!(
                    (
                        request.formula(),
                        request.ordinal(),
                        request.kind(),
                        request.edge()
                    ),
                    (
                        id,
                        ordinal,
                        SourceAtomicRequestKind::OperandExpectedType,
                        Some(edge_id)
                    )
                );
                assert!(
                    request.candidate().is_none()
                        && request.type_site().is_none()
                        && request.attribute().is_none()
                );
            }
        }
        let mut expected_wrappers = Vec::new();
        for (index, syntax) in atoms.iter().enumerate() {
            let mut cursor = syntax.id();
            let mut groups = Vec::new();
            while let Some(parent) = ast
                .node_views()
                .find(|view| view.children().contains(&cursor))
            {
                match parent.kind() {
                    K::FormulaExpression => cursor = parent.id(),
                    K::ParenthesizedFormula => {
                        groups.push(parent);
                        cursor = parent.id();
                    }
                    _ => break,
                }
            }
            groups.reverse();
            for (ordinal, group) in groups.into_iter().enumerate() {
                expected_wrappers.push((
                    SourceAtomicFormulaId::new(index),
                    ordinal,
                    site(group.id()),
                    group.range(),
                    lexical(group.id()),
                    spelling(group.id()),
                ));
            }
        }
        assert_eq!(atomic.wrappers().len(), expected_wrappers.len());
        for ((_, row), (formula, ordinal, site, range, context, spelling)) in
            atomic.wrappers().iter().zip(expected_wrappers)
        {
            assert_eq!(
                (
                    row.formula(),
                    row.ordinal(),
                    row.site(),
                    row.source_range(),
                    row.context(),
                    row.recovery(),
                    row.spelling()
                ),
                (
                    formula,
                    ordinal,
                    &site,
                    range,
                    context,
                    SourceAtomicFormulaRecovery::Normal,
                    spelling.as_str()
                )
            );
            assert_eq!(
                typed.nodes().node(site.node()).unwrap().kind.as_str(),
                "source.formula.atomic.parenthesized"
            );
        }
        assert_eq!(metadata.len(), primary.terms().len() + atoms.len());
        for input in metadata {
            let node = typed.nodes().node(input.typed_site.node()).unwrap();
            assert_eq!(node.links.context, input.local_context);
            assert!(input.cluster_facts.is_empty());
        }
        let resolved = ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast: typed,
            cluster_facts: &facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &selection,
            expressions: metadata.clone(),
            node_hints: vec![],
            statement_semantics: None,
            statement_proofs: None,
        })
        .map_err(|error| error.to_string())?;
        assert_eq!(resolved.source_context(), Some(context));
        assert_eq!(resolved.source_type(), Some(types));
        assert_eq!(resolved.source_term(), Some(primary));
        assert_eq!(resolved.source_atomic_formula(), Some(atomic));
        assert!(
            resolved.checked_formulas().is_empty()
                && resolved.checked_proofs().is_empty()
                && resolved.statement_semantics().is_empty()
        );
        assert_eq!(
            SourceTypeProducer::build_source_theorem_equalities(source, symbols, scope)?,
            lower
        );
        if case == 0 {
            let installed = typed
                .clone()
                .with_source_formula_composition_from_original(source, symbols, scope)
                .expect("old B5 positive before unsupported lower");
            assert_eq!(
                installed
                    .source_composite_formula()
                    .unwrap()
                    .formulas()
                    .len(),
                1
            );
            assert_eq!(
                installed
                    .source_formula_composition()
                    .unwrap()
                    .bound_uses()
                    .len(),
                2
            );
        }
        let module_context = env
            .contexts()
            .iter()
            .find(|(_, row)| row.owner == BindingContextOwner::Module)
            .unwrap()
            .0;
        assert!(env.contexts().get(module_context).unwrap().parent.is_none());
    }
    // Recovered source is refused at its genuine resolver or lower owner; it
    // never supplies a lower RED receipt for the admitted connective family.
    let (source, symbols, scope) = recovered.unwrap();
    if let Ok(scope) = scope {
        assert!(
            SourceTypeProducer::build_source_theorem_equalities(&source, &symbols, &scope).is_err()
        );
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn task257b6_native_connective_composition_preserves_installed_replay() -> Result<(), String> {
    use SourceCompositeFormulaKind as C;
    use SourceFormulaAtomicEdgeRole as A;
    use SourceFormulaEdgeRole as E;
    use mizar_checker::{
        binding_env::BindingContextOwner,
        cluster_trace::ClusterFactTable,
        overload_resolution::*,
        resolved_typed_ast::{ResolvedTypedAst, ResolvedTypedAstInputs},
        source_atomic_formula::SourceAtomicFormulaId,
        source_composite_formula::*,
        source_formula_composition::*,
        source_statement::*,
        source_type::SourceTypeProducer,
        type_checker::CheckedStatementOwner,
        typed_ast::{NodeRecoveryState, TypeEntryActual, TypedAst, TypedAstParts, TypedSiteRef},
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::NamespacePath,
        names::{
            LocalTermBinding, SourceVariableBindingKind, SourceVariableScopeInput,
            SourceVariableScopeResolver,
        },
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::{SignatureProjectionExtractor, source_proof_scope_maps},
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceAnchor,
        SourceInput, SourceOriginInput,
    };
    use mizar_syntax::{SurfaceFormulaConnective, SurfaceNodeKind as K, SurfaceQuantifierKind};
    use std::collections::{BTreeMap, BTreeSet};

    // Source-derived tables, not a production graph dump. Formula indexes are
    // outer-before-inner source order; atom indexes are equality source order.
    // Each edge tuple is (parent, operand ordinal, role, child).
    let fixture =
        include_str!("../../../../../mizar-checker/tests/testdata/native_connective_statement.miz");
    let old =
        include_str!("../../../../../mizar-checker/tests/testdata/native_universal_statement.miz");
    let theorem = |body: &str| {
        format!("theorem Shape: for x being set holds {body} proof let x be set; thus x = x; end;")
    };
    let fixture_kinds = vec![
        C::Universal,
        C::Biconditional,
        C::Implication,
        C::Conjunction,
        C::Disjunction,
        C::Negation,
        C::Universal,
    ];
    let fixture_edges = vec![
        (0, 0, E::UniversalBody, 1),
        (1, 0, E::BiconditionalLeft, 2),
        (1, 1, E::BiconditionalRight, 6),
        (2, 0, E::ImplicationLeft, 3),
        (2, 1, E::ImplicationRight, 5),
        (3, 0, E::ConjunctionLeft, 4),
    ];
    let fixture_atoms = vec![
        (3, 1, A::ConjunctionRight, 2),
        (4, 0, A::DisjunctionLeft, 0),
        (4, 1, A::DisjunctionRight, 1),
        (5, 0, A::NegatedFormula, 3),
        (6, 0, A::UniversalBody, 4),
    ];
    let mut cases = vec![(
        old.to_owned(),
        vec![C::Universal],
        vec![],
        vec![(0, 0, A::UniversalBody, 0)],
        (0, 0),
    )];
    for text in [
        fixture.to_owned(),
        fixture
            .replace("NativeConnectives", "Renamed")
            .replace('x', "outer")
            .replace('y', "inner"),
        format!(
            "\n:: layout\n{}",
            fixture
                .replace("being set", "being\tset")
                .replace("proof", "proof\n:: proof scope\n")
        ),
    ] {
        cases.push((
            text,
            fixture_kinds.clone(),
            fixture_edges.clone(),
            fixture_atoms.clone(),
            (4, 1),
        ));
    }
    for (body, kind, left, right) in [
        (
            "x = x & x = x",
            C::Conjunction,
            A::ConjunctionLeft,
            A::ConjunctionRight,
        ),
        (
            "x = x or x = x",
            C::Disjunction,
            A::DisjunctionLeft,
            A::DisjunctionRight,
        ),
        (
            "x = x implies x = x",
            C::Implication,
            A::ImplicationLeft,
            A::ImplicationRight,
        ),
        (
            "x = x iff x = x",
            C::Biconditional,
            A::BiconditionalLeft,
            A::BiconditionalRight,
        ),
    ] {
        cases.push((
            theorem(body),
            vec![C::Universal, kind],
            vec![(0, 0, E::UniversalBody, 1)],
            vec![(1, 0, left, 0), (1, 1, right, 1)],
            (0, 0),
        ));
    }
    cases.push((
        theorem("not x = x"),
        vec![C::Universal, C::Negation],
        vec![(0, 0, E::UniversalBody, 1)],
        vec![(1, 0, A::NegatedFormula, 0)],
        (0, 0),
    ));
    for (body, kind, role, left, right) in [
        (
            "x = x & x = x & x = x",
            C::Conjunction,
            E::ConjunctionLeft,
            A::ConjunctionLeft,
            A::ConjunctionRight,
        ),
        (
            "x = x or x = x or x = x",
            C::Disjunction,
            E::DisjunctionLeft,
            A::DisjunctionLeft,
            A::DisjunctionRight,
        ),
    ] {
        cases.push((
            theorem(body),
            vec![C::Universal, kind, kind],
            vec![(0, 0, E::UniversalBody, 1), (1, 0, role, 2)],
            vec![(1, 1, right, 2), (2, 0, left, 0), (2, 1, right, 1)],
            (0, 0),
        ));
    }
    cases.push((
        theorem("x = x implies x = x implies x = x"),
        vec![C::Universal, C::Implication, C::Implication],
        vec![(0, 0, E::UniversalBody, 1), (1, 1, E::ImplicationRight, 2)],
        vec![
            (1, 0, A::ImplicationLeft, 0),
            (2, 0, A::ImplicationLeft, 1),
            (2, 1, A::ImplicationRight, 2),
        ],
        (0, 0),
    ));
    for (body, kind, left, right) in [
        (
            "(not x = x) & (not x = x)",
            C::Conjunction,
            E::ConjunctionLeft,
            E::ConjunctionRight,
        ),
        (
            "(not x = x) or (not x = x)",
            C::Disjunction,
            E::DisjunctionLeft,
            E::DisjunctionRight,
        ),
        (
            "(not x = x) implies (not x = x)",
            C::Implication,
            E::ImplicationLeft,
            E::ImplicationRight,
        ),
    ] {
        cases.push((
            theorem(body),
            vec![C::Universal, kind, C::Negation, C::Negation],
            vec![
                (0, 0, E::UniversalBody, 1),
                (1, 0, left, 2),
                (1, 1, right, 3),
            ],
            vec![(2, 0, A::NegatedFormula, 0), (3, 0, A::NegatedFormula, 1)],
            (2, 0),
        ));
    }
    cases.push((
        theorem("not (((x = x & ((x = x)))))"),
        vec![C::Universal, C::Negation, C::Conjunction],
        vec![(0, 0, E::UniversalBody, 1), (1, 0, E::NegatedFormula, 2)],
        vec![
            (2, 0, A::ConjunctionLeft, 0),
            (2, 1, A::ConjunctionRight, 1),
        ],
        (3, 2),
    ));
    cases.push((
        theorem("x = x & (for x being set holds x = x or x = x)"),
        vec![C::Universal, C::Conjunction, C::Universal, C::Disjunction],
        vec![
            (0, 0, E::UniversalBody, 1),
            (1, 1, E::ConjunctionRight, 2),
            (2, 0, E::UniversalBody, 3),
        ],
        vec![
            (1, 0, A::ConjunctionLeft, 0),
            (3, 0, A::DisjunctionLeft, 1),
            (3, 1, A::DisjunctionRight, 2),
        ],
        (1, 0),
    ));
    cases.push(("theorem Unused: for x, spare being set, idle being set holds ((x = x or x = x)) proof let x, unused be set, idle be set; thus x = x; end;".into(), vec![C::Universal, C::Disjunction], vec![(0, 0, E::UniversalBody, 1)], vec![(1, 0, A::DisjunctionLeft, 0), (1, 1, A::DisjunctionRight, 1)], (2, 0)));

    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let mut sources = Vec::new();
    for (case, (text, ..)) in cases.iter().enumerate() {
        let path = root.join(format!("src/connective_composition_{case}.miz"));
        fs::write(&path, text).unwrap();
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: snapshot_id(798),
                input: SourceInput {
                    package_id: PackageId::new("native-connective-composition"),
                    module_path: ModulePath::new(format!("composition_{case}")),
                    normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Disk { path },
                },
            },
            &ids,
        )
        .unwrap();
        assert!(
            frontend.diagnostics.is_empty(),
            "case {case}: {:?}",
            frontend.diagnostics
        );
        let ast = frontend.ast.as_ref().unwrap();
        let module = ModuleId::new(
            frontend.source.package_id.clone(),
            frontend.source.module_path.clone(),
        );
        let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
        source.validate_against(ast, &module).unwrap();
        let shells = DeclarationShellCollector::new(ast, &module).collect();
        let collected = SignatureProjectionExtractor::new(
            ast,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        assert!(collected.diagnostics().is_empty());
        let symbols = collected.env().clone();
        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(ast, &module, &symbols),
        )
        .unwrap();
        sources.push((frontend, source, symbols, scope));
    }
    // This separate loop is deliberate: all genuine lower positives, including
    // old B5, must pass before the first unchanged upper entrance can be RED-U.
    let mut lowers = Vec::new();
    for (case, (_, source, symbols, scope)) in sources.iter().enumerate() {
        let lower = SourceTypeProducer::build_source_theorem_equalities(source, symbols, scope)
            .unwrap_or_else(|error| panic!("not RED-U: lower case {case} failed: {error}"));
        if case == 0 {
            assert!(
                lower
                    .2
                    .clone()
                    .with_source_formula_composition_from_original(source, symbols, scope)
                    .is_ok()
            );
        }
        lowers.push(lower);
    }
    let collection = OverloadCollectionOutput::collect([], []);
    let expansion = TemplateExpansionOutput::expand(&collection);
    let viability = CandidateViabilityOutput::filter(&expansion, []);
    let specificity = SpecificityGraphOutput::build(&viability, []);
    let selection = OverloadSelectionOutput::resolve(&specificity, []);
    let facts = ClusterFactTable::new();
    let assemble = |typed: &TypedAst, expressions| {
        ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast: typed,
            cluster_facts: &facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &selection,
            expressions,
            node_hints: vec![],
            statement_semantics: None,
            statement_proofs: None,
        })
    };
    for (case, ((frontend, source, symbols, scope), (env, inference, primitive, metadata))) in
        sources.iter().zip(&lowers).enumerate()
    {
        let ast = frontend.ast.as_ref().unwrap();
        let (_, kinds, edge_specs, atom_specs, wrapper_counts) = &cases[case];
        let site = |id| {
            let resolved = source.resolved_node_for(id).unwrap();
            let matches = primitive
                .nodes()
                .iter()
                .filter(|(_, row)| row.resolved_node == Some(resolved))
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1);
            TypedSiteRef::Node(matches[0].0)
        };
        let spelling = |id| {
            let range = ast.node_view(id).unwrap().range();
            let mut tokens = ast
                .node_views()
                .filter(|view| {
                    view.as_token().is_some()
                        && range.start <= view.range().start
                        && view.range().end <= range.end
                })
                .collect::<Vec<_>>();
            tokens.sort_by_key(|view| view.range().start);
            tokens
                .iter()
                .map(|view| view.as_token().unwrap().text.as_ref())
                .collect::<Vec<_>>()
                .join(" ")
        };
        let maps = source_proof_scope_maps(source, symbols, scope)?;
        let body_context = |id| {
            env.contexts()
                .iter()
                .find(|(_, row)| row.lexical_scope.as_ref() == maps.0.get(&id))
                .unwrap()
                .0
        };
        let lexical = |id| {
            let mut cursor = id;
            loop {
                if maps.0.contains_key(&cursor) {
                    return body_context(cursor);
                }
                cursor = ast
                    .node_views()
                    .find(|view| view.children().contains(&cursor))
                    .unwrap()
                    .id();
            }
        };
        let mut syntax = ast
            .node_views()
            .filter(|view| {
                matches!(
                    view.kind(),
                    K::QuantifiedFormula(_) | K::BinaryFormula(_) | K::PrefixFormula(_)
                )
            })
            .collect::<Vec<_>>();
        syntax.sort_by_key(|view| (view.range().start, std::cmp::Reverse(view.range().end)));
        assert_eq!(syntax.len(), kinds.len());
        let mut expected = SourceCompositeFormulaHandoffInput {
            source_id: source.source_id(),
            module_id: source.module().clone(),
            formulas: vec![],
            wrappers: vec![],
            roots: vec![SourceFormulaRootInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                ownership: SourceFormulaRootOwnership::UnassignedStatement,
            }],
            binders: vec![],
            type_sites: vec![],
            edges: vec![],
            requests: vec![],
        };
        let primary = primitive.source_term().unwrap();
        let atomic = primitive.source_atomic_formula().unwrap();
        let mut binder_for = BTreeMap::new();
        for (index, (written, kind)) in syntax.iter().zip(kinds).enumerate() {
            let formula = SourceCompositeFormulaId::new(index);
            let (incoming, marker, request_kind) = match kind {
                C::Universal => {
                    assert_eq!(
                        written.kind(),
                        &K::QuantifiedFormula(SurfaceQuantifierKind::Universal)
                    );
                    let body = body_context(written.id());
                    assert_eq!(
                        env.contexts().get(body).unwrap().owner,
                        BindingContextOwner::SourceFormula {
                            source_range: written.range()
                        }
                    );
                    (
                        env.contexts().get(body).unwrap().parent.unwrap(),
                        written
                            .children()
                            .iter()
                            .filter_map(|id| {
                                ast.node_view(*id)
                                    .unwrap()
                                    .as_token()
                                    .filter(|token| matches!(token.text.as_ref(), "for" | "holds"))
                                    .map(|token| token.text.to_string())
                            })
                            .collect::<Vec<_>>()
                            .join(" "),
                        SourceFormulaRequestKind::QuantifierSemantics,
                    )
                }
                C::Negation => {
                    assert!(matches!(written.kind(), K::PrefixFormula(_)));
                    assert_eq!(spelling(written.children()[0]), "not");
                    (
                        lexical(written.id()),
                        "not".into(),
                        SourceFormulaRequestKind::NegationSemantics,
                    )
                }
                _ => {
                    let K::BinaryFormula(operator) = written.kind() else {
                        panic!("expected binary source")
                    };
                    assert!(!operator.repeated);
                    let (connective, marker) = match kind {
                        C::Conjunction => (SurfaceFormulaConnective::And, "&"),
                        C::Disjunction => (SurfaceFormulaConnective::Or, "or"),
                        C::Implication => (SurfaceFormulaConnective::Implies, "implies"),
                        C::Biconditional => (SurfaceFormulaConnective::Iff, "iff"),
                        _ => panic!("outside frozen family"),
                    };
                    assert_eq!(operator.connective, connective);
                    assert_eq!(spelling(written.children()[1]), marker);
                    (
                        lexical(written.id()),
                        marker.into(),
                        SourceFormulaRequestKind::ConnectiveSemantics,
                    )
                }
            };
            expected.formulas.push(SourceCompositeFormulaInput {
                site: site(written.id()),
                source_range: written.range(),
                source_ordinal: index,
                context: incoming,
                recovery: SourceCompositeFormulaRecovery::Normal,
                spelling: marker,
                kind: *kind,
            });
            expected.requests.push(SourceFormulaRequestInput {
                formula,
                ordinal: 0,
                kind: request_kind,
                binder: None,
                type_site: None,
            });
            if *kind == C::Universal {
                let body = body_context(written.id());
                let mut ordinal = 0;
                for segment in written
                    .children()
                    .iter()
                    .map(|id| ast.node_view(*id).unwrap())
                    .filter(|view| view.kind() == &K::QuantifierVariableSegment)
                {
                    let ty = ast.node_view(*segment.children().last().unwrap()).unwrap();
                    let head = ast.node_view(ty.children()[0]).unwrap();
                    assert_eq!(spelling(ty.id()), "set");
                    let descriptor = segment.children()[..segment.children().len() - 1]
                        .iter()
                        .map(|id| spelling(*id))
                        .collect::<Vec<_>>()
                        .join(" ");
                    for original in scope
                        .bindings()
                        .iter()
                        .filter(|binding| segment.children().contains(&binding.node()))
                    {
                        assert_eq!(original.kind(), SourceVariableBindingKind::Quantifier);
                        assert_eq!(Some(original.scope()), maps.0.get(&written.id()));
                        let declaration = primitive
                            .source_context()
                            .unwrap()
                            .declarations()
                            .iter()
                            .find(|(_, row)| row.site == site(original.node()))
                            .unwrap()
                            .1;
                        let binder = SourceQuantifierBinderId::new(expected.binders.len());
                        let type_site = SourceBinderTypeSiteId::new(expected.type_sites.len());
                        binder_for.insert(original.id(), binder);
                        expected.binders.push(SourceQuantifierBinderInput {
                            formula,
                            ordinal,
                            segment_site: site(segment.id()),
                            segment_range: segment.range(),
                            segment_spelling: descriptor.clone(),
                            identifier_site: site(original.node()),
                            identifier_range: original.range(),
                            identifier_spelling: original.spelling().into(),
                            local: LocalTermBinding::new(
                                original.spelling(),
                                original.scope().clone(),
                                original.range(),
                                original.ordinal(),
                            ),
                            binding: declaration.binding,
                            body_context: body,
                            type_site,
                            recovery: SourceCompositeFormulaRecovery::Normal,
                        });
                        expected.type_sites.push(SourceBinderTypeSiteInput {
                            binder,
                            site: site(ty.id()),
                            source_range: ty.range(),
                            spelling: "set".into(),
                            head_site: site(head.id()),
                            head_range: head.range(),
                            head_spelling: "set".into(),
                            context: incoming,
                            recovery: SourceCompositeFormulaRecovery::Normal,
                            head: SourceBinderTypeHead::BuiltinSet,
                        });
                        expected.requests.push(SourceFormulaRequestInput {
                            formula,
                            ordinal: 1 + ordinal,
                            kind: SourceFormulaRequestKind::BinderType,
                            binder: Some(binder),
                            type_site: Some(type_site),
                        });
                        ordinal += 1;
                    }
                }
            }
            let mut cursor = written.id();
            let mut groups = Vec::new();
            while let Some(parent) = ast
                .node_views()
                .find(|view| view.children().contains(&cursor))
            {
                match parent.kind() {
                    K::FormulaExpression => cursor = parent.id(),
                    K::ParenthesizedFormula => {
                        groups.push(parent);
                        cursor = parent.id();
                    }
                    _ => break,
                }
            }
            groups.reverse();
            let depth = groups.len();
            for (ordinal, group) in groups.into_iter().enumerate() {
                let mut descriptor = expected.formulas[index].spelling.clone();
                for _ in ordinal..depth {
                    descriptor = format!("( {descriptor} )");
                }
                expected.wrappers.push(SourceFormulaWrapperInput {
                    formula,
                    ordinal,
                    site: site(group.id()),
                    source_range: group.range(),
                    context: incoming,
                    recovery: SourceCompositeFormulaRecovery::Normal,
                    spelling: descriptor,
                });
            }
        }
        expected.edges = edge_specs
            .iter()
            .map(|(parent, ordinal, role, child)| SourceFormulaEdgeInput {
                parent: SourceCompositeFormulaId::new(*parent),
                ordinal: *ordinal,
                role: *role,
                child: SourceCompositeFormulaId::new(*child),
            })
            .collect();
        let mut composition = SourceFormulaCompositionHandoffInput {
            source_id: source.source_id(),
            module_id: source.module().clone(),
            atomic_edges: atom_specs
                .iter()
                .map(
                    |(parent, ordinal, role, atom)| SourceFormulaAtomicEdgeInput {
                        formula: SourceCompositeFormulaId::new(*parent),
                        ordinal: *ordinal,
                        role: *role,
                        child: SourceAtomicFormulaId::new(*atom),
                    },
                )
                .collect(),
            bound_uses: vec![],
        };
        let mut use_ordinals = BTreeMap::new();
        for original in scope
            .references()
            .iter()
            .filter(|reference| binder_for.contains_key(&reference.binding()))
        {
            let binder = binder_for[&original.binding()];
            let (term, _) = primary
                .terms()
                .iter()
                .find(|(_, row)| row.site() == &site(original.node()))
                .unwrap();
            let (reference, actual) = primary
                .references()
                .iter()
                .find(|(_, row)| row.term() == term)
                .unwrap();
            assert_eq!(actual.use_ordinal(), original.ordinal());
            let atom = atomic
                .formulas()
                .iter()
                .find(|(_, row)| {
                    row.source_range().start <= original.range().start
                        && original.range().end <= row.source_range().end
                })
                .unwrap()
                .0;
            let body_edge = SourceFormulaAtomicEdgeId::new(
                composition
                    .atomic_edges
                    .iter()
                    .position(|edge| edge.child == atom)
                    .unwrap(),
            );
            let ordinal = use_ordinals.entry(binder).or_insert(0);
            composition.bound_uses.push(SourceQuantifierBoundUseInput {
                binder,
                ordinal: *ordinal,
                body_edge,
                term,
                reference,
            });
            *ordinal += 1;
        }
        assert_eq!(expected.wrappers.len(), wrapper_counts.0);
        assert_eq!(atomic.wrappers().len(), wrapper_counts.1);
        assert_eq!(composition.bound_uses.len(), 2 * atom_specs.len());
        if (1..=3).contains(&case) {
            assert_eq!(
                composition
                    .bound_uses
                    .iter()
                    .map(|row| row.body_edge.index())
                    .collect::<Vec<_>>(),
                vec![1, 1, 2, 2, 0, 0, 3, 3, 4, 4]
            );
            assert_eq!(
                composition
                    .bound_uses
                    .iter()
                    .map(|row| row.binder.index())
                    .collect::<Vec<_>>(),
                vec![0, 0, 0, 0, 0, 0, 0, 0, 1, 0]
            );
        }
        let mut with_rows = primitive.clone();
        with_rows.install_source_nodes(source).unwrap();
        let rows = with_rows
            .source_node_entries()
            .unwrap()
            .cloned()
            .collect::<Vec<_>>();
        let mut reachable = BTreeSet::new();
        let mut pending = vec![source.arena().root()];
        while let Some(node) = pending.pop() {
            if reachable.insert(node) {
                pending.extend_from_slice(source.arena().node(node).unwrap().children());
            }
        }
        assert_eq!(
            rows.iter().map(|row| row.site()).collect::<BTreeSet<_>>(),
            reachable
        );
        assert_eq!(rows.len(), reachable.len());
        for row in &rows {
            let original = source.arena().node(row.site()).unwrap();
            let surface = ast
                .node_views()
                .find(|view| source.resolved_node_for(view.id()) == Some(row.site()))
                .unwrap();
            assert_eq!(row.kind(), original.kind());
            assert_eq!(
                &SourceAnchor::Range(row.range()),
                original.origin().anchor()
            );
            assert_eq!(row.spelling(), spelling(surface.id()));
            assert_eq!(row.recovery(), NodeRecoveryState::Normal);
            if let Some(parent) = row.parent() {
                assert_eq!(
                    source.arena().node(parent).unwrap().children()[row.child_ordinal()],
                    row.site()
                );
            } else {
                assert_eq!(row.site(), source.arena().root());
            }
        }
        let baseline = primitive
            .clone()
            .with_source_formula_composition_from_original(source, symbols, scope)
            .expect("RED-U only after every genuine lower positive passes");
        let composite = baseline.source_composite_formula().unwrap();
        let joined = baseline.source_formula_composition().unwrap();
        assert_eq!(composite.source_id(), expected.source_id);
        assert_eq!(composite.module_id(), &expected.module_id);
        assert_eq!(composite.binding_env(), env);
        assert_eq!(joined.source_id(), composition.source_id);
        assert_eq!(joined.module_id(), &composition.module_id);
        assert_eq!(composite.formulas().len(), expected.formulas.len());
        for ((_, row), expected) in composite.formulas().iter().zip(&expected.formulas) {
            assert_eq!(
                (
                    row.site(),
                    row.source_range(),
                    row.source_ordinal(),
                    row.context(),
                    row.recovery(),
                    row.spelling(),
                    row.kind()
                ),
                (
                    &expected.site,
                    expected.source_range,
                    expected.source_ordinal,
                    expected.context,
                    expected.recovery,
                    expected.spelling.as_str(),
                    expected.kind
                )
            );
        }
        assert_eq!(composite.wrappers().len(), expected.wrappers.len());
        for ((_, row), expected) in composite.wrappers().iter().zip(&expected.wrappers) {
            assert_eq!(
                (
                    row.formula(),
                    row.ordinal(),
                    row.site(),
                    row.source_range(),
                    row.context(),
                    row.recovery(),
                    row.spelling()
                ),
                (
                    expected.formula,
                    expected.ordinal,
                    &expected.site,
                    expected.source_range,
                    expected.context,
                    expected.recovery,
                    expected.spelling.as_str()
                )
            );
        }
        assert_eq!(composite.roots().len(), expected.roots.len());
        for ((_, row), expected) in composite.roots().iter().zip(&expected.roots) {
            assert_eq!(
                (row.formula(), row.ordinal(), row.ownership()),
                (expected.formula, expected.ordinal, expected.ownership)
            );
        }
        assert_eq!(composite.binders().len(), expected.binders.len());
        for ((_, row), expected) in composite.binders().iter().zip(&expected.binders) {
            assert_eq!(
                (
                    row.formula(),
                    row.ordinal(),
                    row.segment_site(),
                    row.segment_range(),
                    row.segment_spelling()
                ),
                (
                    expected.formula,
                    expected.ordinal,
                    &expected.segment_site,
                    expected.segment_range,
                    expected.segment_spelling.as_str()
                )
            );
            assert_eq!(
                (
                    row.identifier_site(),
                    row.identifier_range(),
                    row.identifier_spelling(),
                    row.local(),
                    row.binding(),
                    row.body_context(),
                    row.type_site(),
                    row.recovery()
                ),
                (
                    &expected.identifier_site,
                    expected.identifier_range,
                    expected.identifier_spelling.as_str(),
                    &expected.local,
                    expected.binding,
                    expected.body_context,
                    expected.type_site,
                    expected.recovery
                )
            );
        }
        assert_eq!(composite.type_sites().len(), expected.type_sites.len());
        for ((_, row), expected) in composite.type_sites().iter().zip(&expected.type_sites) {
            assert_eq!(
                (
                    row.binder(),
                    row.site(),
                    row.source_range(),
                    row.spelling(),
                    row.head_site(),
                    row.head_range(),
                    row.head_spelling(),
                    row.context(),
                    row.recovery(),
                    row.head()
                ),
                (
                    expected.binder,
                    &expected.site,
                    expected.source_range,
                    expected.spelling.as_str(),
                    &expected.head_site,
                    expected.head_range,
                    expected.head_spelling.as_str(),
                    expected.context,
                    expected.recovery,
                    expected.head
                )
            );
        }
        assert_eq!(composite.edges().len(), expected.edges.len());
        for ((_, row), expected) in composite.edges().iter().zip(&expected.edges) {
            assert_eq!(
                (row.parent(), row.ordinal(), row.role(), row.child()),
                (
                    expected.parent,
                    expected.ordinal,
                    expected.role,
                    expected.child
                )
            );
        }
        assert_eq!(composite.requests().len(), expected.requests.len());
        for ((_, row), expected) in composite.requests().iter().zip(&expected.requests) {
            assert_eq!(
                (
                    row.formula(),
                    row.ordinal(),
                    row.kind(),
                    row.binder(),
                    row.type_site()
                ),
                (
                    expected.formula,
                    expected.ordinal,
                    expected.kind,
                    expected.binder,
                    expected.type_site
                )
            );
        }
        assert_eq!(
            joined
                .atomic_edges()
                .iter()
                .map(|(_, row)| SourceFormulaAtomicEdgeInput::from(row))
                .collect::<Vec<_>>(),
            composition.atomic_edges
        );
        assert_eq!(
            joined
                .bound_uses()
                .iter()
                .map(|(_, row)| SourceQuantifierBoundUseInput::from(row))
                .collect::<Vec<_>>(),
            composition.bound_uses
        );
        assert_eq!(joined.primary_term_fingerprint(), primary.debug_text());
        assert_eq!(joined.atomic_formula_fingerprint(), atomic.debug_text());
        assert_eq!(
            joined.composite_formula_fingerprint(),
            composite.debug_text()
        );
        assert_eq!(baseline.nodes(), primitive.nodes());
        assert_eq!(baseline.contexts(), primitive.contexts());
        assert_eq!(baseline.types(), inference.type_entries());
        assert_eq!(baseline.source_context(), primitive.source_context());
        assert_eq!(baseline.source_type(), primitive.source_type());
        assert_eq!(baseline.source_term(), primitive.source_term());
        assert_eq!(
            baseline.source_atomic_formula(),
            primitive.source_atomic_formula()
        );
        assert_eq!(baseline.facts(), primitive.facts());
        assert_eq!(baseline.coercions(), primitive.coercions());
        assert_eq!(
            baseline.initial_obligations(),
            primitive.initial_obligations()
        );
        assert_eq!(baseline.diagnostics(), primitive.diagnostics());
        assert_eq!(
            baseline
                .source_node_entries()
                .unwrap()
                .cloned()
                .collect::<Vec<_>>(),
            rows
        );
        assert!(baseline.source_statement().is_none());
        let baseline_resolved = assemble(&baseline, metadata.clone()).unwrap();
        assert_eq!(
            baseline_resolved.source_composite_formula(),
            Some(composite)
        );
        assert_eq!(baseline_resolved.source_formula_composition(), Some(joined));

        // Native258 retains precisely two propositions. Quantifier guards stay
        // inside the header graph; only actual proof Let declarations add facts.
        let theorem = ast
            .node_views()
            .find(|view| view.kind() == &K::TheoremItem)
            .unwrap();
        let proof = ast
            .node_views()
            .find(|view| view.kind() == &K::ProofBlock)
            .unwrap();
        let conclusion = ast
            .node_views()
            .find(|view| view.kind() == &K::ConclusionStatement)
            .unwrap();
        let label = spelling(theorem.children()[1]);
        let symbol = symbols
            .symbols()
            .iter()
            .find(|entry| entry.primary_spelling() == label)
            .unwrap();
        let checked = CheckedStatementOwner::validate_exact_local_theorem(
            symbols,
            symbol.symbol().clone(),
            source.source_id(),
            source.module(),
        )
        .unwrap();
        assert_eq!(checked.source_range(), theorem.range());
        assert_eq!(checked.origin(), symbol.origin());
        let module_context = env
            .contexts()
            .iter()
            .find(|(_, row)| row.owner == BindingContextOwner::Module)
            .unwrap()
            .0;
        let proof_context = body_context(proof.id());
        assert_eq!(
            env.contexts().get(proof_context).unwrap().parent,
            Some(module_context)
        );
        assert_eq!(expected.formulas[0].context, module_context);
        let conclusion_atom = SourceAtomicFormulaId::new(atom_specs.len());
        assert_eq!(atomic.formulas().len(), atom_specs.len() + 1);
        assert_eq!(
            atomic.formulas().get(conclusion_atom).unwrap().context(),
            proof_context
        );
        let let_count = ast
            .node_views()
            .filter(|view| view.kind() == &K::LetStatement)
            .count();
        let mut statement_expected = SourceStatementHandoffInput {
            source_id: source.source_id(),
            module_id: source.module().clone(),
            owners: vec![SourceTheoremOwnerInput {
                symbol: symbol.symbol().clone(),
                contribution: symbol.contribution(),
                site: site(theorem.id()),
                source_range: theorem.range(),
                spelling: label,
                role: SourceTheoremRole::Theorem,
                status: SourceTheoremStatus::Unmodified,
                recovery: SourceStatementRecovery::Normal,
            }],
            statements: vec![],
            contexts: vec![],
            input_facts: vec![],
            candidate_facts: vec![],
        };
        for (index, (syntax, binding_context, formula, source_ordinal, kind)) in [
            (
                theorem,
                module_context,
                SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0)),
                0,
                SourceStatementKind::TheoremProposition,
            ),
            (
                conclusion,
                proof_context,
                SourceStatementFormulaTarget::Atomic(conclusion_atom),
                1 + let_count,
                SourceStatementKind::Conclusion,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let statement = SourceStatementId::new(index);
            let context = SourceStatementContextId::new(index);
            statement_expected.statements.push(SourceStatementInput {
                owner: SourceTheoremOwnerId::new(0),
                context,
                formula,
                site: site(syntax.id()),
                source_range: syntax.range(),
                source_ordinal,
                spelling: spelling(syntax.id()),
                kind,
                recovery: SourceStatementRecovery::Normal,
            });
            statement_expected
                .contexts
                .push(SourceStatementContextInput {
                    statement,
                    binding_context,
                    source_range: syntax.range(),
                    visible_bindings: env
                        .contexts()
                        .get(binding_context)
                        .unwrap()
                        .visible_bindings
                        .clone(),
                });
            statement_expected
                .candidate_facts
                .push(SourceStatementCandidateFactInput {
                    statement,
                    context,
                    ordinal: 0,
                    kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                    formula,
                });
        }
        for (ordinal, local) in scope
            .bindings()
            .iter()
            .filter(|binding| binding.kind() == SourceVariableBindingKind::Let)
            .enumerate()
        {
            let declaration = primitive
                .source_context()
                .unwrap()
                .declarations()
                .iter()
                .find(|(_, row)| row.site == site(local.node()))
                .unwrap()
                .1;
            assert_eq!(declaration.binding_context, proof_context);
            let uses = scope
                .references()
                .iter()
                .filter(|reference| reference.binding() == local.id())
                .map(|reference| {
                    primary
                        .references()
                        .iter()
                        .find(|(_, row)| {
                            primary.terms().get(row.term()).unwrap().site()
                                == &site(reference.node())
                        })
                        .unwrap()
                        .0
                })
                .collect::<Vec<_>>();
            assert!(
                expected
                    .binders
                    .iter()
                    .all(|binder| binder.binding != declaration.binding)
            );
            statement_expected
                .input_facts
                .push(SourceStatementInputFactInput {
                    statement: SourceStatementId::new(1),
                    context: SourceStatementContextId::new(1),
                    ordinal,
                    kind: SourceStatementInputFactKind::LetTypeGuard,
                    binding: declaration.binding,
                    uses,
                });
        }
        assert!(statement_expected.contexts[0].visible_bindings.is_empty());
        assert_eq!(
            statement_expected.contexts[1].visible_bindings,
            statement_expected
                .input_facts
                .iter()
                .map(|row| row.binding)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            statement_expected
                .input_facts
                .iter()
                .map(|row| row.uses.len())
                .sum::<usize>(),
            2
        );
        if case == cases.len() - 1 {
            assert_eq!(expected.binders.len(), 3);
            assert_eq!(statement_expected.input_facts.len(), 3);
            assert!(
                statement_expected.input_facts[1].uses.is_empty()
                    && statement_expected.input_facts[2].uses.is_empty()
            );
            assert!(
                composition
                    .bound_uses
                    .iter()
                    .all(|row| row.binder == SourceQuantifierBinderId::new(0))
            );
        }
        let installed = baseline
            .clone()
            .with_source_statement_from_original(source, symbols, scope)
            .expect("native258 connective header coexistence");
        let statement = installed.source_statement().unwrap();
        assert_eq!(statement.source_id(), statement_expected.source_id);
        assert_eq!(statement.module_id(), &statement_expected.module_id);
        assert_eq!(statement.checked_owner(), &checked);
        assert_eq!(statement.binding_env(), env);
        assert_eq!(statement.binding_fingerprint(), env.debug_text());
        assert_eq!(statement.primary_term_fingerprint(), primary.debug_text());
        assert_eq!(statement.atomic_formula_fingerprint(), atomic.debug_text());
        assert_eq!(
            statement.composite_formula_fingerprint(),
            Some(composite.debug_text().as_str())
        );
        assert_eq!(
            statement.formula_composition_fingerprint(),
            Some(joined.debug_text().as_str())
        );
        assert_eq!(statement.owners().len(), 1);
        for ((_, row), expected) in statement.owners().iter().zip(&statement_expected.owners) {
            assert_eq!(row.symbol(), &expected.symbol);
            assert_eq!(row.contribution(), expected.contribution);
            assert_eq!(
                (
                    row.site(),
                    row.source_range(),
                    row.spelling(),
                    row.role(),
                    row.status(),
                    row.recovery()
                ),
                (
                    &expected.site,
                    expected.source_range,
                    expected.spelling.as_str(),
                    expected.role,
                    expected.status,
                    expected.recovery
                )
            );
        }
        assert_eq!(statement.statements().len(), 2);
        for ((_, row), expected) in statement
            .statements()
            .iter()
            .zip(&statement_expected.statements)
        {
            assert_eq!(
                (
                    row.owner(),
                    row.context(),
                    row.formula(),
                    row.site(),
                    row.source_range(),
                    row.source_ordinal(),
                    row.spelling(),
                    row.kind(),
                    row.recovery()
                ),
                (
                    expected.owner,
                    expected.context,
                    expected.formula,
                    &expected.site,
                    expected.source_range,
                    expected.source_ordinal,
                    expected.spelling.as_str(),
                    expected.kind,
                    expected.recovery
                )
            );
        }
        assert_eq!(statement.contexts().len(), 2);
        for ((_, row), expected) in statement
            .contexts()
            .iter()
            .zip(&statement_expected.contexts)
        {
            assert_eq!(
                (
                    row.statement(),
                    row.binding_context(),
                    row.source_range(),
                    row.visible_bindings()
                ),
                (
                    expected.statement,
                    expected.binding_context,
                    expected.source_range,
                    expected.visible_bindings.as_slice()
                )
            );
        }
        assert_eq!(
            statement.input_facts().len(),
            statement_expected.input_facts.len()
        );
        for ((_, row), expected) in statement
            .input_facts()
            .iter()
            .zip(&statement_expected.input_facts)
        {
            assert_eq!(
                (
                    row.statement(),
                    row.context(),
                    row.ordinal(),
                    row.kind(),
                    row.binding(),
                    row.uses()
                ),
                (
                    expected.statement,
                    expected.context,
                    expected.ordinal,
                    expected.kind,
                    expected.binding,
                    expected.uses.as_slice()
                )
            );
        }
        assert_eq!(statement.candidate_facts().len(), 2);
        for ((_, row), expected) in statement
            .candidate_facts()
            .iter()
            .zip(&statement_expected.candidate_facts)
        {
            assert_eq!(
                (
                    row.statement(),
                    row.context(),
                    row.ordinal(),
                    row.kind(),
                    row.formula()
                ),
                (
                    expected.statement,
                    expected.context,
                    expected.ordinal,
                    expected.kind,
                    expected.formula
                )
            );
        }
        assert_eq!(installed.nodes(), baseline.nodes());
        assert_eq!(installed.contexts(), baseline.contexts());
        assert_eq!(installed.types(), baseline.types());
        assert_eq!(installed.source_context(), baseline.source_context());
        assert_eq!(installed.source_type(), baseline.source_type());
        assert_eq!(installed.source_term(), baseline.source_term());
        assert_eq!(
            installed.source_atomic_formula(),
            baseline.source_atomic_formula()
        );
        assert_eq!(installed.source_composite_formula(), Some(composite));
        assert_eq!(installed.source_formula_composition(), Some(joined));
        assert_eq!(
            installed
                .source_node_entries()
                .unwrap()
                .cloned()
                .collect::<Vec<_>>(),
            rows
        );
        assert_eq!(installed.facts(), baseline.facts());
        assert_eq!(installed.coercions(), baseline.coercions());
        assert_eq!(
            installed.initial_obligations(),
            baseline.initial_obligations()
        );
        assert_eq!(installed.diagnostics(), baseline.diagnostics());
        assert!(
            installed.source_statement_references().is_none()
                && installed.source_statement_witnesses().is_none()
                && installed.source_proof_local_binding().is_none()
                && installed.source_proof_local_let_binding().is_none()
                && installed.source_proof_local_let_type().is_none()
        );
        let fresh = SourceTypeProducer::build_source_theorem_equalities(source, symbols, scope)?
            .2
            .with_source_formula_composition_from_original(source, symbols, scope)
            .unwrap()
            .with_source_statement_from_original(source, symbols, scope)
            .unwrap();
        assert_eq!(fresh, installed);
        let resolved = assemble(&installed, metadata.clone()).unwrap();
        assert_eq!(assemble(&fresh, metadata.clone()).unwrap(), resolved);
        assert_eq!(resolved.source_statement(), Some(statement));
        assert_eq!(resolved.source_context(), installed.source_context());
        assert_eq!(resolved.source_type(), installed.source_type());
        assert_eq!(resolved.source_term(), installed.source_term());
        assert_eq!(
            resolved.source_atomic_formula(),
            installed.source_atomic_formula()
        );
        assert_eq!(resolved.source_composite_formula(), Some(composite));
        assert_eq!(resolved.source_formula_composition(), Some(joined));
        assert_eq!(resolved.nodes(), baseline_resolved.nodes());
        assert_eq!(resolved.expr_metadata(), baseline_resolved.expr_metadata());
        assert_eq!(resolved.expr_metadata().len(), metadata.len());
        for input in metadata {
            let row = resolved.expr_metadata().get_by_expr(&input.expr).unwrap();
            assert_eq!(row.typed_site, input.typed_site);
            assert_eq!(row.local_context, input.local_context);
            assert_eq!(row.cluster_facts, input.cluster_facts);
            assert_eq!(
                resolved.expr_metadata().get_by_site(&input.typed_site),
                Some(row)
            );
            assert_eq!(
                installed
                    .nodes()
                    .node(input.typed_site.node())
                    .unwrap()
                    .anchor,
                SourceAnchor::Range(row.source_range)
            );
            let expected_type = installed
                .types()
                .iter()
                .find(|(_, entry)| entry.owner == input.typed_site)
                .map(|(_, entry)| {
                    let TypeEntryActual::Known(actual) = entry.actual else {
                        panic!("unknown final type")
                    };
                    actual
                });
            assert_eq!(row.final_type, expected_type);
            assert!(
                row.visible_facts.is_empty()
                    && row.overload.is_none()
                    && row.inserted_views.is_empty()
                    && row.diagnostics.is_empty()
            );
        }
        assert!(
            resolved.checked_formulas().is_empty()
                && resolved.statement_semantics().is_empty()
                && resolved.checked_proofs().is_empty()
                && resolved.checked_proof_nodes().is_empty()
                && resolved.checked_terminal_goals().is_empty()
        );
        assert!(
            resolved.cluster_facts().is_empty()
                && resolved.inserted_coercions().is_empty()
                && resolved.diagnostics().is_empty()
        );
        assert!(
            resolved.source_statement_references().is_none()
                && resolved.source_statement_witnesses().is_none()
        );

        // Nonempty final success above precedes every refusal and mutation.
        let pristine = (
            source.clone(),
            symbols.clone(),
            scope.clone(),
            primitive.clone(),
            baseline.clone(),
            installed.clone(),
        );
        assert!(
            installed
                .clone()
                .with_source_formula_composition_from_original(source, symbols, scope)
                .is_err()
        );
        assert!(
            installed
                .clone()
                .with_source_statement_from_original(source, symbols, scope)
                .is_err()
        );
        assert!(
            baseline
                .clone()
                .with_source_statement(statement.clone())
                .is_err()
        );
        assert!(
            primitive
                .clone()
                .with_source_statement_from_original(source, symbols, scope)
                .is_err()
        );
        assert!(
            primitive
                .clone()
                .with_source_composite_formula(composite.clone())
                .is_err()
        );
        assert!(
            primitive
                .clone()
                .with_source_formula_composition(composite.clone(), joined.clone())
                .is_err()
        );
        assert!(
            primitive
                .clone()
                .with_source_formula_composition_statement(
                    composite.clone(),
                    joined.clone(),
                    statement.clone()
                )
                .is_err()
        );
        assert!(
            with_rows
                .with_source_formula_composition_from_original(source, symbols, scope)
                .is_err()
        );
        let foreign = &sources[(case + 1) % sources.len()];
        assert!(
            primitive
                .clone()
                .with_source_formula_composition_from_original(&foreign.1, &foreign.2, &foreign.3)
                .is_err()
        );
        assert!(
            primitive
                .clone()
                .with_source_formula_composition_from_original(source, &foreign.2, scope)
                .is_err()
        );
        assert!(
            primitive
                .clone()
                .with_source_formula_composition_from_original(source, symbols, &foreign.3)
                .is_err()
        );
        assert!(
            baseline
                .clone()
                .with_source_statement_from_original(&foreign.1, &foreign.2, &foreign.3)
                .is_err()
        );
        assert!(
            baseline
                .clone()
                .with_source_statement_from_original(source, &foreign.2, scope)
                .is_err()
        );
        assert!(
            baseline
                .clone()
                .with_source_statement_from_original(source, symbols, &foreign.3)
                .is_err()
        );
        let detached = TypedAst::try_new(TypedAstParts {
            source_id: source.source_id(),
            module_id: source.module().clone(),
            resolved_root: primitive.resolved_root(),
            source_context: primitive.source_context().cloned(),
            source_type: primitive.source_type().cloned(),
            source_attribute: None,
            nodes: primitive.nodes().clone(),
            contexts: primitive.contexts().clone(),
            types: primitive.types().clone(),
            facts: primitive.facts().clone(),
            coercions: primitive.coercions().clone(),
            initial_obligations: primitive.initial_obligations().clone(),
            diagnostics: primitive.diagnostics().clone(),
        })
        .unwrap();
        assert!(
            detached
                .with_source_formula_composition_from_original(source, symbols, scope)
                .is_err()
        );
        let mut enriched_nodes = primitive
            .nodes()
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        enriched_nodes.push(mizar_checker::typed_ast::TypedNode::new(
            "additional.source.node",
            SourceAnchor::Range(theorem.range()),
        ));
        let enriched_nodes =
            mizar_checker::typed_ast::TypedArena::try_new(primitive.nodes().root(), enriched_nodes)
                .unwrap();
        let enriched = TypedAst::try_new(TypedAstParts {
            source_id: source.source_id(),
            module_id: source.module().clone(),
            resolved_root: primitive.resolved_root(),
            source_context: primitive.source_context().cloned(),
            source_type: primitive.source_type().cloned(),
            source_attribute: None,
            nodes: enriched_nodes,
            contexts: primitive.contexts().clone(),
            types: primitive.types().clone(),
            facts: primitive.facts().clone(),
            coercions: primitive.coercions().clone(),
            initial_obligations: primitive.initial_obligations().clone(),
            diagnostics: primitive.diagnostics().clone(),
        })
        .unwrap()
        .with_source_term(primary.clone())
        .unwrap()
        .with_source_atomic_formula(atomic.clone())
        .unwrap();
        assert_eq!(enriched.nodes().len(), primitive.nodes().len() + 1);
        assert!(
            enriched
                .with_source_formula_composition_from_original(source, symbols, scope)
                .is_err()
        );
        let mut reversed = metadata.clone();
        reversed.reverse();
        assert_eq!(assemble(&installed, reversed).unwrap(), resolved);
        let mut duplicate = metadata.clone();
        duplicate.push(duplicate[0].clone());
        assert!(assemble(&installed, duplicate).is_err());
        let mut wrong_site = metadata.clone();
        wrong_site[0].typed_site =
            TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(usize::MAX));
        assert!(assemble(&installed, wrong_site).is_err());
        let mut wrong_context = metadata.clone();
        wrong_context[0].local_context = Some(mizar_checker::typed_ast::LocalTypeContextId::new(
            usize::MAX,
        ));
        assert!(assemble(&installed, wrong_context).is_err());
        // Raw B2/B3 constructors must continue refusing native transactions.
        assert!(
            SourceCompositeFormulaProducer::build(expected.clone(), env, primitive.nodes())
                .is_err()
        );
        assert!(
            SourceFormulaCompositionProducer::build(
                composition.clone(),
                primary,
                atomic,
                composite,
                primitive.nodes()
            )
            .is_err()
        );
        assert_eq!(assemble(&installed, metadata.clone()).unwrap(), resolved);
        assert_eq!(
            (
                source.clone(),
                symbols.clone(),
                scope.clone(),
                primitive.clone(),
                baseline.clone(),
                installed.clone()
            ),
            pristine
        );
        assert!(
            primitive.source_node_entries().is_none()
                && primitive.source_composite_formula().is_none()
                && primitive.source_formula_composition().is_none()
        );
        assert!(baseline.source_statement().is_none());
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn task257b7_native_quantifier_lower_preserves_genuine_tables() -> Result<(), String> {
    use mizar_checker::{
        binding_env::*,
        cluster_trace::ClusterFactTable,
        overload_resolution::*,
        resolved_typed_ast::{ResolvedTypedAst, ResolvedTypedAstInputs},
        source_atomic_formula::*,
        source_term::{
            SourcePrimaryTermKind, SourcePrimaryTermRecovery, SourcePrimaryTermReferenceRole,
            SourcePrimaryTermRole,
        },
        source_type::{SourceTypeHead, SourceTypeProducer},
        type_checker::{
            FormulaStatus, SourceVariableSemanticsChecker as Semantics, TermStatus, TypeHeadRef,
        },
        typed_ast::{
            NodeRecoveryState, TypeEntryActual, TypeStatus, TypedArena, TypedAst, TypedAstError,
            TypedAstParts, TypedNodeKind, TypedSiteRef,
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
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceAnchor,
        SourceInput, SourceOriginInput,
    };
    use mizar_syntax::SurfaceNodeKind as K;

    let fixture =
        include_str!("../../../../../mizar-checker/tests/testdata/native_quantifier_statement.miz");
    let old =
        include_str!("../../../../../mizar-checker/tests/testdata/native_connective_statement.miz");
    let theorem =
        |header: &str| format!("theorem Shape: {header} proof let x be set; thus x = x; end;");
    let mut cases = vec![old.to_owned(), fixture.to_owned(),
        fixture.replace("NativeQuantifiers", "Renamed").replace(" x", " outer").replace("(x", "(outer").replace(" y", " inner").replace(" z", " deep"),
        format!("\n:: layout\n{}", fixture.replace("being set", "being\tset").replace("proof", "proof\n:: proof scope\n")),
        fixture.replace("theorem NativeQuantifiers", "lemma NativeQuantifiers"),
        theorem("ex x being set st x = x"),
        theorem("for x being set st x = x holds x = x"),
        theorem("for x being set st (ex y being set st y = x) holds x = x"),
        theorem("for x being set ex y being set st y = x"),
        theorem("for x being set st x = x ex y being set st y = x"),
        theorem("ex x being set st for y being set st y = x holds y = x"),
        theorem("ex x being set st ex y being set st y = x"),
        theorem("for x being set st x = x for y being set st y = x holds y = x"),
        "theorem Unused: ex x, spare being set, idle being set st ((x = x)) proof let x, unused be set, idle be set; thus x = x; end;".into(),
        theorem("for x being set holds (ex y being set st y = x) & x = x"),
        theorem("for x being set st not x = x holds ex y being set st ((y = x implies y = y) iff (not y = x or y = x))"),
        format!("{}\n{}", theorem("for x being set st x = x holds x = x").replace("Shape", "First"), theorem("ex x being set st x = x").replace("theorem Shape", "lemma Second")),
        theorem("(for x being set holds x = x)"),
        theorem("(ex x being set st x = x)"),
        fixture.replace(": ", ": (((").replace("\nproof", ")))\nproof"),
        format!("\n:: grouped lemma\n{}", fixture.replace("theorem NativeQuantifiers", "lemma Grouped").replace(": ", ": ((").replace("\nproof", "))\nproof").replace("being set", "being\tset")),
        theorem("not (for x being set holds x = x)"),
        theorem("(for x being set holds x = x) & (ex y being set st y = y)"),
        theorem("((for x being set holds x = x) or (ex y being set st y = y))"),
        theorem("(((for x being set holds x = x) implies (ex y being set st y = y)))"),
        format!("\n:: connective lemma\n{}", theorem("(for x being set holds x = x) iff (ex y being set st y = y)").replace("theorem Shape", "lemma Renamed").replace("being set", "being\tset")),
        fixture.replace(": ", ": not (").replace("\nproof", ")\nproof"),
    ];
    let implicit = include_str!(
        "../../../../../mizar-checker/tests/testdata/native_implicit_quantifier_statement.miz"
    );
    cases.extend([
        implicit.to_owned(),
        implicit.replace("Implicit", "Renamed").replace(" x", " parcel").replace("set;", "set;\n:: layout\n"),
        implicit.replace("for x holds", "ex x st"),
        "reserve x, y for set; theorem Shared: for x, y holds x = y proof let x be set; thus x = x; end;".into(),
        "reserve x for set; reserve y for set; theorem Mixed: for x being set, y holds x = y proof let x be set; thus x = x; end;".into(),
        implicit.replace("for x holds", "for x holds ex x st"),
        "reserve x, y for set; theorem Restricted: for x st (ex y st y = x) holds x = x proof let x be set; thus x = x; end;".into(),
        "reserve x, y for set; lemma Fixed: (for x holds x = x) iff (ex y st y = y) proof let x be set; thus x = x; end;".into(),
        "reserve x, y for set; theorem Unused: for x, y holds x = x proof let x be set; thus x = x; end;".into(),
    ]);
    // EN04 §§4.3/4.5/4.6.1: defaults apply after their directive;
    // explicit binders need no default, and unused hints remain source items.
    let interleaved = "reserve x for set; theorem First: for x holds x = x proof let x be set; thus x = x; end; reserve y for set; lemma Second: ex y st y = y proof let y be set; thus y = y; end;";
    cases.extend([
        implicit.replace("for x holds", "for x being set holds"),
        format!("{} reserve y for set;", implicit.trim()),
        format!("{} {implicit}", theorem("for z being set holds z = z")),
        interleaved.into(),
        interleaved
            .replace(" x", " parcel")
            .replace(" y", " token")
            .replace("set;", "set;\n:: layout\n"),
    ]);
    // Manual (variable terms, equalities including proof, declarations, contexts).
    let counts = [
        (12, 6, 3, 4),
        (12, 6, 5, 6),
        (12, 6, 5, 6),
        (12, 6, 5, 6),
        (12, 6, 5, 6),
        (4, 2, 2, 3),
        (6, 3, 2, 3),
        (6, 3, 3, 4),
        (4, 2, 3, 4),
        (6, 3, 3, 4),
        (6, 3, 3, 4),
        (4, 2, 3, 4),
        (8, 4, 3, 4),
        (4, 2, 6, 3),
        (6, 3, 3, 4),
        (12, 6, 3, 4),
        (10, 5, 4, 5),
        (4, 2, 2, 3),
        (4, 2, 2, 3),
        (12, 6, 5, 6),
        (12, 6, 5, 6),
        (4, 2, 2, 3),
        (6, 3, 3, 4),
        (6, 3, 3, 4),
        (6, 3, 3, 4),
        (6, 3, 3, 4),
        (12, 6, 5, 6),
        (4, 2, 3, 3),
        (4, 2, 3, 3),
        (4, 2, 3, 3),
        (4, 2, 5, 3),
        (4, 2, 5, 3),
        (4, 2, 4, 4),
        (6, 3, 5, 4),
        (6, 3, 5, 4),
        (4, 2, 5, 3),
        (4, 2, 3, 3),
        (4, 2, 4, 3),
        (8, 4, 5, 5),
        (8, 4, 6, 5),
        (8, 4, 6, 5),
    ];
    let positive_count = cases.len();
    cases.extend([
        theorem("ex x being object st x = x").replace("let x be set", "let x be object"),
        theorem("ex x being set st 1 = 1"),
        theorem("ex x being set st x = x & ... & x = x"),
        theorem("ex x being set st x = x or ... or x = x"),
        theorem("ex x being set st contradiction"),
        theorem("ex x being set st thesis"),
        theorem("ex x being set st x = x").replace("thus x = x;", "thus (x = x);"),
        theorem("ex x being set st x = x").replace("thus x = x;", "thus (x |-> x) = x;"),
        theorem("ex x being set st x = x").replace("thus x = x;", "thus x = x & x = x;"),
        theorem("ex x being set st x = x").replace("thus x = x;", "set y = x; thus y = y;"),
    ]);
    cases.extend([implicit
        .replace("reserve x for set;", "reserve x, y for set;")
        .replace("holds x = x", "holds x = y")]);
    // Literal physical type paths start at the actual ItemList. Declaration
    // slots and reference winners are source-intent oracles, independent of receipts.
    let mixed = "theorem Mixed: for x being set, idle being object holds x = x proof let x be set, spare be object; thus x = x; end;";
    let grouped = "theorem Groups: for a, idle being object, b being set holds a = a & b = b proof let a, spare be object, b be set; thus a = a; thus b = b; end;";
    let object_let =
        "reserve x for object; theorem ObjectLet: for x holds x = x proof let x; thus x = x; end;";
    let object_profiles = [
        (cases[positive_count].clone(), vec!["object", "object"],
            vec![vec![0,3,0,1,2], vec![0,4,1,1,2]], vec![0,1], vec![0,0,1,1], (4,2,2,3)),
        (mixed.into(), vec!["set", "object", "set", "object"],
            vec![vec![0,3,0,1,2],vec![0,3,0,3,2],vec![0,4,1,1,2],vec![0,4,1,3,2]], vec![0,1,2,3],vec![0,0,2,2],(4,2,4,3)),
        ("theorem Inverse: for x being object, idle being set holds x = x proof let x be object, spare be set; thus x = x; end;".into(), vec!["object", "set", "object", "set"],
            vec![vec![0,3,0,1,2],vec![0,3,0,3,2],vec![0,4,1,1,2],vec![0,4,1,3,2]],vec![0,1,2,3],vec![0,0,2,2],(4,2,4,3)),
        (grouped.into(),vec!["object","set","object","set"],
            vec![vec![0,3,0,1,4],vec![0,3,0,3,2],vec![0,4,1,1,4],vec![0,4,1,3,2]],vec![0,0,1,2,2,3],vec![0,0,2,2,3,3,5,5],(8,4,6,3)),
        ("theorem Shadow: for x being object holds for x being set holds x = x proof let x be object; thus x = x; end;".into(),vec!["object","set","object"],
            vec![vec![0,3,0,1,2],vec![0,3,0,3,1,2],vec![0,4,1,1,2]],vec![0,1,2],vec![1,1,2,2],(4,2,3,4)),
        ("reserve x for set; theorem Override: for x being object holds x = x proof let x be object; thus x = x; end;".into(),vec!["set","object","object"],
            vec![vec![0,1,2],vec![1,3,0,1,2],vec![1,4,1,1,2]],vec![0,1,2],vec![1,1,2,2],(4,2,3,3)),
        (format!("\n:: Object layout\n{}",grouped.replace(" a,", " parcel,").replace(" a = a", " parcel = parcel").replace(" b being", " token being").replace(" b be", " token be").replace(" b = b", " token = token").replace("being object", "being\tobject").replace("proof", "proof\n:: locals\n")),vec!["object","set","object","set"],
            vec![vec![0,3,0,1,4],vec![0,3,0,3,2],vec![0,4,1,1,4],vec![0,4,1,3,2]],vec![0,0,1,2,2,3],vec![0,0,2,2,3,3,5,5],(8,4,6,3)),
        ("theorem Canonical: for idle being object, x being object holds x = x proof let spare be object, x be object; thus x = x; end;".into(),vec!["object","object","object","object"],
            vec![vec![0,3,0,1,2],vec![0,3,0,3,2],vec![0,4,1,1,2],vec![0,4,1,3,2]],vec![0,1,2,3],vec![1,1,3,3],(4,2,4,3)),
        ("theorem Restrict: for x being object st x = x holds x = x proof let x be object; thus x = x; end;".into(),vec!["object","object"],
            vec![vec![0,3,0,1,2],vec![0,4,1,1,2]],vec![0,1],vec![0,0,0,0,1,1],(6,3,2,3)),
        (implicit.replace("let x be set", "let x"),vec!["set"],
            vec![vec![0,1,2]],vec![0,0,0],vec![1,1,2,2],(4,2,3,3)),
        ("reserve x for object; theorem ObjectQuant: for x holds x = x proof let x be object; thus x = x; end;".into(),vec!["object","object"],
            vec![vec![0,1,2],vec![1,4,1,1,2]],vec![0,0,1],vec![1,1,2,2],(4,2,3,3)),
        (object_let.into(),vec!["object"],
            vec![vec![0,1,2]],vec![0,0,0],vec![1,1,2,2],(4,2,3,3)),
        ("reserve x for object; reserve y for set; theorem MixedDefault: for x, y holds x = x & y = y proof let x, y; thus x = x; thus y = y; end;".into(),vec!["object","set"],
            vec![vec![0,1,2],vec![1,1,2]],vec![0,1,0,1,0,1],vec![2,2,3,3,4,4,5,5],(8,4,6,3)),
        ("reserve x, y for object; theorem SharedDefault: for x, y holds x = y proof let x, y; thus x = x; end;".into(),vec!["object"],
            vec![vec![0,1,4]],vec![0,0,0,0,0,0],vec![2,3,4,4],(4,2,6,3)),
        ("reserve spare for object; reserve x for set; theorem UnusedDefault: for x holds x = x proof let x; thus x = x; end;".into(),vec!["object","set"],
            vec![vec![0,1,2],vec![1,1,2]],vec![0,1,1,1],vec![2,2,3,3],(4,2,4,3)),
        ("reserve x for object; theorem NestedDefault: for x holds for x being set holds x = x proof let x; thus x = x; end;".into(),vec!["object","set"],
            vec![vec![0,1,2],vec![1,3,0,3,1,2]],vec![0,0,1,0],vec![2,2,3,3],(4,2,4,4)),
        ("reserve x for object; theorem OverrideSet: for x being set holds x = x proof let x be set; thus x = x; end;".into(),vec!["object","set","set"],
            vec![vec![0,1,2],vec![1,3,0,1,2],vec![1,4,1,1,2]],vec![0,1,2],vec![1,1,2,2],(4,2,3,3)),
        ("reserve x for object; theorem LocalOverride: for x holds x = x proof let x be set; thus x = x; end;".into(),vec!["object","set"],
            vec![vec![0,1,2],vec![1,4,1,1,2]],vec![0,0,1],vec![1,1,2,2],(4,2,3,3)),
        ("reserve x for object; theorem HeaderOverride: for x being set holds x = x proof let x; thus x = x; end;".into(),vec!["object","set"],
            vec![vec![0,1,2],vec![1,3,0,1,2]],vec![0,1,0],vec![1,1,2,2],(4,2,3,3)),
        ("reserve x for set; theorem OppositeDefault: for x being object holds x = x proof let x; thus x = x; end;".into(),vec!["set","object"],
            vec![vec![0,1,2],vec![1,3,0,1,2]],vec![0,1,0],vec![1,1,2,2],(4,2,3,3)),
        (format!("\n:: default layout\n{}",object_let.replace(" x", " parcel").replace("for object", "for\tobject").replace("proof", "proof\n:: locals\n")),vec!["object"],
            vec![vec![0,1,2]],vec![0,0,0],vec![1,1,2,2],(4,2,3,3)),
    ];
    let object_cases = cases.len();
    cases.extend(
        object_profiles
            .iter()
            .skip(1)
            .map(|profile| profile.0.clone()),
    );
    let binder_cases = cases.len();
    let binder_proof = "theorem Binder: ex z being set st z = z proof let x, y, idle be set; thus (x |-> x) = x; end;";
    cases.extend([
        binder_proof.to_owned(),
        binder_proof.replace("(x |-> x)", "(x,y) |-> {x,y}"),
        binder_proof.replace("(x |-> x)", "(x,y) |-> x"),
        binder_proof
            .replace("let x, y, idle be set", "let x be set, y, idle be object")
            .replace("(x |-> x)", "(x,y) |-> y"),
        binder_proof.replace("x |-> x", "x |-> (x |-> x)"),
        binder_proof.replace("(x |-> x) = x", "(x |-> x) = (x |-> x)"),
        binder_proof.replace("x |-> x", "x |-> y"),
        binder_proof.replace("x |-> x", "x |-> ((y |-> x))"),
        format!("reserve x, y for object; {binder_proof}"),
        format!(
            "\n:: alpha/layout\n{}",
            binder_proof
                .replace(" x", " parcel")
                .replace("(x", "(parcel")
                .replace(" y", " token")
                .replace("set;", "set;\n:: declarations\n")
        ),
        "reserve x for set; theorem Head: ex z being set st (x |-> x) = x proof let x be set; thus x = x; end;".into(),
        binder_proof.replace("thus", "take x; thus"),
    ]);
    let distinct_reserves = cases.len();
    cases.push(format!(
        "reserve spare for set; reserve other for object; {binder_proof}"
    ));
    cases.push(binder_proof.replace("(x |-> x)", "((x,y) |-> {x,y})"));
    let unsupported_binder_cases = cases.len();
    cases.extend([
        binder_proof.replace("thus", "set local = x; thus"),
        binder_proof.replace("thus", "deffunc Local(n be set) -> set equals n; thus"),
        binder_proof.replace("thus (x |-> x) = x;", "now thus (x |-> x) = x; end;"),
        binder_proof.replace("x |-> x", "x |-> {y where y is set : y = x}"),
        binder_proof.replace("thus", "take local = x; thus"),
    ]);
    let duplicate_reserve = cases.len();
    cases.push(implicit.replace("reserve x for set;", "reserve x, x for set;"));
    let forward_default = cases.len();
    cases.push(format!(
        "{} reserve x for set;",
        implicit.replace("reserve x for set;", "")
    ));
    let missing_let_default = cases.len();
    cases.push(
        "theorem MissingLet: for x being set holds x = x proof let x; thus x = x; end;".into(),
    );
    let forward_let_default = cases.len();
    cases.push("theorem ForwardLet: for x being set holds x = x proof let x; thus x = x; end; reserve x for object;".into());
    let recovered_start = cases.len();
    cases.extend([
        theorem("for x being set holds (x = x"),
        theorem("ex x being set holds x = x"),
        theorem("for x being set st x = x"),
    ]);
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let mut prepared = Vec::new();
    let mut recovered = Vec::new();
    for (case, text) in cases.iter().enumerate() {
        let path = root.join(format!("src/quantifier_lower_{case}.miz"));
        fs::write(&path, text).unwrap();
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: snapshot_id(807),
                input: SourceInput {
                    package_id: PackageId::new("native-quantifier-lower"),
                    module_path: ModulePath::new(format!("lower_{case}")),
                    normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Disk { path },
                },
            },
            &ids,
        )
        .unwrap();
        if case == duplicate_reserve {
            assert!(
                frontend
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.message.contains("duplicate binding name"))
            );
            continue;
        }
        let is_recovered = case >= recovered_start;
        assert_eq!(
            frontend.diagnostics.is_empty(),
            !is_recovered,
            "case {case}: {:?}",
            frontend.diagnostics
        );
        let ast = frontend.ast.as_ref().unwrap();
        let module = ModuleId::new(
            frontend.source.package_id.clone(),
            frontend.source.module_path.clone(),
        );
        let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
        source.validate_against(ast, &module).unwrap();
        let shells = DeclarationShellCollector::new(ast, &module).collect();
        let collected = SignatureProjectionExtractor::new(
            ast,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        if !is_recovered {
            assert!(collected.diagnostics().is_empty());
        }
        let symbols = collected.env().clone();
        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(ast, &module, &symbols),
        );
        if case == forward_default || case == missing_let_default || case == forward_let_default {
            if case == missing_let_default || case == forward_let_default {
                let local = ast
                    .node_views()
                    .find(|view| view.kind() == &K::QualifiedVariableSegment)
                    .unwrap();
                assert!(
                    !local
                        .children()
                        .iter()
                        .any(|id| { ast.node_view(*id).unwrap().kind() == &K::TypeExpression })
                );
                let reserves = ast
                    .node_views()
                    .filter(|view| view.kind() == &K::ReserveSegment)
                    .collect::<Vec<_>>();
                assert_eq!(reserves.len(), usize::from(case == forward_let_default));
                if case == forward_let_default {
                    assert!(local.range().end < reserves[0].range().start);
                }
            }
            assert_eq!(
                scope.unwrap_err(),
                mizar_resolve::names::SourceVariableScopeError::UnreservedImplicitVariable
            );
            continue;
        }
        if is_recovered {
            assert!(
                ast.node_views()
                    .any(|view| matches!(view.kind(), K::ErrorRecovery(_)))
            );
            recovered.push((source, symbols, scope));
            continue;
        }
        let scope = scope.unwrap_or_else(|error| {
            panic!("case {case}: {error:?}: {text}: {}", ast.snapshot_text())
        });
        prepared.push((frontend, source, symbols, scope));
    }
    let collection = OverloadCollectionOutput::collect([], []);
    let expansion = TemplateExpansionOutput::expand(&collection);
    let viability = CandidateViabilityOutput::filter(&expansion, []);
    let specificity = SpecificityGraphOutput::build(&viability, []);
    let selection = OverloadSelectionOutput::resolve(&specificity, []);
    let facts = ClusterFactTable::new();
    // Flat projection is a transaction, including unsupported lexical families.
    // Complete this pass before asking the scoped producer for any metadata.
    for (frontend, source, symbols, scope) in &prepared {
        let ast = frontend.ast.as_ref().unwrap();
        if !ast
            .node_views()
            .any(|view| view.kind() == &K::BinderExpression)
        {
            continue;
        }
        source_proof_scope_maps(source, symbols, scope)?;
        let flat = Semantics::occurrence_binding_env(scope);
        assert_eq!(flat.source_id(), scope.source_id());
        assert_eq!(flat.module_id(), scope.module_id());
        assert!(flat.diagnostics().is_empty());
        assert_eq!(flat.contexts().len(), 1);
        let root = flat.contexts().get(BindingContextId::new(0)).unwrap();
        assert_eq!(root.owner, BindingContextOwner::Module);
        assert_eq!(root.layer, BindingContextLayer::Module);
        assert!(root.parent.is_none() && root.lexical_scope.is_none());
        assert_eq!(root.recovery, BindingContextRecovery::Normal);
        let ids = (0..scope.bindings().len())
            .map(BindingId::new)
            .collect::<Vec<_>>();
        assert_eq!(root.bindings, ids);
        assert_eq!(root.visible_bindings, ids);
        assert_eq!(flat.bindings().len(), scope.bindings().len());
        for ((id, row), declaration) in flat.bindings().iter().zip(scope.bindings()) {
            assert_eq!(id.index(), declaration.id().index());
            assert_eq!(row.id, id);
            assert_eq!(row.spelling, declaration.spelling());
            assert_eq!(
                format!("{:?}", row.kind),
                match declaration.kind() {
                    SourceVariableBindingKind::Reserve => "ReservedVariable",
                    SourceVariableBindingKind::Quantifier
                    | SourceVariableBindingKind::ComprehensionGenerator => "QuantifierBinder",
                    SourceVariableBindingKind::Let => "LetBinding",
                    SourceVariableBindingKind::FunctionBinder => "FunctionBinder",
                    SourceVariableBindingKind::InlineParameter => "DefinitionParameter",
                    SourceVariableBindingKind::Set | SourceVariableBindingKind::InlineFunctor =>
                        "LocalAbbreviation",
                    kind => panic!("uncovered genuine declaration {kind:?}"),
                }
            );
            let identity = |declaration: &mizar_resolve::names::SourceVariableBinding| {
                BinderIdentity::ResolverLocal {
                    scope: declaration.scope().clone(),
                    ordinal: declaration.ordinal(),
                    declaration_range: declaration.range(),
                }
            };
            assert_eq!(
                row.identity,
                if declaration.kind() == SourceVariableBindingKind::Reserve {
                    BinderIdentity::ReservedVariable {
                        spelling: declaration.spelling().into(),
                        declaration_range: declaration.range(),
                    }
                } else {
                    identity(declaration)
                }
            );
            assert_eq!(row.owner_context, BindingContextId::new(0));
            assert_eq!(row.declaration_range, declaration.range());
            assert_eq!(row.visible_after_ordinal, declaration.ordinal());
            assert_eq!(row.type_site, BindingTypeSite::Missing);
            assert_eq!(row.status, BindingStatus::Active);
            assert_eq!(row.recovery, BindingRecoveryState::Normal);
            assert!(row.diagnostics.is_empty());
            assert_eq!(
                row.captured,
                CapturedFreeVariables::new(
                    declaration
                        .captures()
                        .iter()
                        .map(|capture| identity(&scope.bindings()[capture.index()]))
                        .collect()
                )
            );
        }
    }
    let mut remapped_inference_id = false;
    let mut distinct_decl_namespace_id = false;
    for (case, (frontend, source, symbols, scope)) in prepared.iter().enumerate() {
        let object_index = if case == positive_count {
            Some(0)
        } else if (object_cases..binder_cases).contains(&case) {
            Some(case - object_cases + 1)
        } else {
            None
        };
        let object = object_index.map(|index| &object_profiles[index]);
        let expected_head = |binding: usize| {
            if object.is_some_and(|profile| profile.1[profile.3[binding]] == "object") {
                TypeHeadRef::BuiltinObject
            } else {
                TypeHeadRef::BuiltinSet
            }
        };
        if case >= positive_count && object.is_none() {
            if case == positive_count + 7 {
                let binder = frontend
                    .ast
                    .as_ref()
                    .unwrap()
                    .node_views()
                    .find(|view| view.kind() == &K::BinderExpression)
                    .unwrap();
                let fresh = scope
                    .bindings()
                    .iter()
                    .find(|binding| binding.node() == binder.children()[0])
                    .expect("real Binder head declaration before checker refusal");
                assert_eq!(fresh.kind(), SourceVariableBindingKind::FunctionBinder);
                assert_eq!(fresh.declared_type(), scope.bindings()[1].declared_type());
                let uses = scope
                    .references()
                    .iter()
                    .filter(|row| {
                        binder.range().start <= row.range().start
                            && row.range().end <= binder.range().end
                    })
                    .collect::<Vec<_>>();
                assert_eq!(uses.len(), 1);
                assert_eq!(uses[0].binding(), fresh.id());
                assert_ne!(fresh.id(), scope.bindings()[1].id());
            }
            let ast = frontend.ast.as_ref().unwrap();
            if ast
                .node_views()
                .any(|view| view.kind() == &K::BinderExpression)
            {
                let before = (
                    source.clone(),
                    symbols.clone(),
                    scope.clone(),
                    ast.snapshot_text(),
                    frontend.source.clone(),
                );
                let maps = source_proof_scope_maps(source, symbols, scope)?;
                let build = Semantics::source_theorem_binding_env;
                if case >= unsupported_binder_cases {
                    assert_eq!(
                        build(source, symbols, scope).unwrap_err(),
                        "theorems.bindings.invalid_lexical_association"
                    );
                } else {
                    // Fixture intent fixes boundaries and declaration/reference membership;
                    // neither the resolver maps nor the projected environment select them.
                    use SourceVariableBindingKind::{FunctionBinder, Let, Quantifier, Reserve};
                    use mizar_resolve::names::{LocalTermScope, SourceVariableTypeRadix as Radix};
                    let legacy = case < binder_cases;
                    let variant = if legacy { 0 } else { case - binder_cases };
                    let header_binder = variant == 10;
                    let reserve_count = match variant {
                        8 | 12 => 2,
                        10 => 1,
                        _ => 0,
                    };
                    let ast_root = ast.root().unwrap();
                    let compilation = ast.node_view(ast_root).unwrap().children().len() - 1;
                    let item = if variant == 12 {
                        2
                    } else {
                        usize::from(reserve_count > 0)
                    };
                    let prefix = [compilation, 0, item];
                    let syntax_at = |path: &[usize]| {
                        path.iter().fold(ast_root, |node, index| {
                            ast.node_view(node).unwrap().children()[*index]
                        })
                    };
                    let at = |path: &[usize]| syntax_at(&[prefix.as_slice(), path].concat());
                    let quantifier = at(&[3, 0]);
                    let proof = at(&[4]);
                    assert!(matches!(
                        ast.node_view(quantifier).unwrap().kind(),
                        K::QuantifiedFormula(_)
                    ));
                    assert_eq!(ast.node_view(proof).unwrap().kind(), &K::ProofBlock);
                    let mut expected = vec![
                        (
                            LocalTermScope::new(vec![]),
                            "Module".to_owned(),
                            BindingContextLayer::Module,
                            None,
                        ),
                        (
                            LocalTermScope::new(vec![0]),
                            format!(
                                "SourceFormula {{ source_range: {:?} }}",
                                ast.node_view(quantifier).unwrap().range()
                            ),
                            BindingContextLayer::Expression,
                            Some(LocalTermScope::new(vec![])),
                        ),
                        (
                            LocalTermScope::new(vec![4]),
                            format!(
                                "SourceStatement {{ source_range: {:?} }}",
                                ast.node_view(proof).unwrap().range()
                            ),
                            BindingContextLayer::Proof,
                            Some(LocalTermScope::new(vec![])),
                        ),
                    ];
                    let mut boundaries = vec![
                        (vec![], LocalTermScope::new(vec![])),
                        (
                            [prefix.as_slice(), &[3, 0]].concat(),
                            LocalTermScope::new(vec![0]),
                        ),
                        (
                            [prefix.as_slice(), &[4]].concat(),
                            LocalTermScope::new(vec![4]),
                        ),
                    ];
                    let conclusion = if variant == 11 { 3 } else { 2 };
                    let equation_path = if header_binder {
                        vec![3, 0, 3]
                    } else {
                        vec![4, conclusion, 1, 0, 0]
                    };
                    let mut binder_path = [equation_path.as_slice(), &[0, 0]].concat();
                    let multi = matches!(variant, 1..=3 | 13);
                    if !multi || variant == 13 {
                        binder_path.extend([1, 0]);
                    }
                    let first_binder = at(&binder_path);
                    let mut binders = vec![(
                        binder_path.clone(),
                        if multi {
                            vec![4, 6]
                        } else if header_binder {
                            vec![0, 2]
                        } else {
                            vec![4, 2]
                        },
                        if multi { 6 } else { 2 },
                    )];
                    if matches!(variant, 4 | 7) {
                        let suffix = if variant == 4 {
                            vec![2, 0, 1, 0]
                        } else {
                            vec![2, 0, 1, 0, 1, 0]
                        };
                        binders.push((
                            [binder_path.as_slice(), suffix.as_slice()].concat(),
                            vec![4, 2, 2],
                            2,
                        ));
                    } else if variant == 5 {
                        binders.push((
                            [equation_path.as_slice(), &[2, 0, 1, 0]].concat(),
                            vec![4, 3],
                            2,
                        ));
                    }
                    for (path, local, body_index) in &binders {
                        let binder = ast.node_view(at(path)).unwrap();
                        assert_eq!(binder.kind(), &K::BinderExpression);
                        assert_eq!(binder.children().len(), body_index + 1);
                        let parent = LocalTermScope::new(local[..local.len() - 1].to_vec());
                        expected.push((
                            LocalTermScope::new(local.clone()),
                            format!(
                                "SourceFunctionBinder {{ source_range: {:?} }}",
                                binder.range()
                            ),
                            BindingContextLayer::Expression,
                            Some(parent),
                        ));
                        boundaries.push((
                            [prefix.as_slice(), path.as_slice(), &[*body_index]].concat(),
                            LocalTermScope::new(local.clone()),
                        ));
                    }
                    // A plain tree census assigns each node to the deepest explicit
                    // fixture boundary and independently checks every upstream map row.
                    let mut expected_scopes = std::collections::BTreeMap::new();
                    let mut expected_parents = std::collections::BTreeMap::new();
                    let compilation_node = syntax_at(&[compilation]);
                    expected_scopes.insert(ast_root, LocalTermScope::new(vec![]));
                    expected_parents.insert(compilation_node, ast_root);
                    let mut pending = vec![(compilation_node, vec![compilation])];
                    while let Some((node, path)) = pending.pop() {
                        let local = boundaries
                            .iter()
                            .filter(|(boundary, _)| path.starts_with(boundary))
                            .max_by_key(|(boundary, _)| boundary.len())
                            .unwrap()
                            .1
                            .clone();
                        assert!(expected_scopes.insert(node, local).is_none());
                        for (index, child) in
                            ast.node_view(node).unwrap().children().iter().enumerate()
                        {
                            assert!(expected_parents.insert(*child, node).is_none());
                            let mut child_path = path.clone();
                            child_path.push(index);
                            pending.push((*child, child_path));
                        }
                    }
                    // The root's flat token overlay is metadata; only tokens
                    // unused by the structural tree retain the root as parent.
                    for token in &ast.node_view(ast_root).unwrap().children()[..compilation] {
                        assert!(ast.node_view(*token).unwrap().as_token().is_some());
                        expected_scopes
                            .entry(*token)
                            .or_insert_with(|| LocalTermScope::new(vec![]));
                        expected_parents.entry(*token).or_insert(ast_root);
                    }
                    assert_eq!(expected_scopes.len(), ast.node_views().count());
                    assert_eq!(maps.0, expected_scopes);
                    assert_eq!(maps.1, expected_parents);
                    let x = if variant == 9 { "parcel" } else { "x" };
                    let y = if variant == 9 { "token" } else { "y" };
                    let mut declarations = Vec::new();
                    let mut bind = |node, kind, spelling: &str, local: Vec<u32>, radix| {
                        declarations.push((
                            node,
                            kind,
                            spelling.to_owned(),
                            LocalTermScope::new(local),
                            radix,
                        ));
                    };
                    if variant == 8 {
                        bind(
                            syntax_at(&[compilation, 0, 0, 1, 0]),
                            Reserve,
                            "x",
                            vec![],
                            Radix::Object,
                        );
                        bind(
                            syntax_at(&[compilation, 0, 0, 1, 2]),
                            Reserve,
                            "y",
                            vec![],
                            Radix::Object,
                        );
                    } else if header_binder || variant == 12 {
                        bind(
                            syntax_at(&[compilation, 0, 0, 1, 0]),
                            Reserve,
                            if header_binder { "x" } else { "spare" },
                            vec![],
                            Radix::Set,
                        );
                        if variant == 12 {
                            bind(
                                syntax_at(&[compilation, 0, 1, 1, 0]),
                                Reserve,
                                "other",
                                vec![],
                                Radix::Object,
                            );
                        }
                    }
                    let quantifier_id = reserve_count;
                    bind(
                        at(&[3, 0, 1, 0]),
                        Quantifier,
                        if legacy { "x" } else { "z" },
                        vec![0],
                        Radix::Set,
                    );
                    if header_binder {
                        bind(
                            ast.node_view(first_binder).unwrap().children()[0],
                            FunctionBinder,
                            "x",
                            vec![0, 2],
                            Radix::Set,
                        );
                    }
                    let let_id = reserve_count + 1 + usize::from(header_binder);
                    bind(at(&[4, 1, 1, 0]), Let, x, vec![4], Radix::Set);
                    if !legacy && !header_binder {
                        let (y_path, idle_path) = if variant == 3 {
                            (vec![4, 1, 3, 0], vec![4, 1, 3, 2])
                        } else {
                            (vec![4, 1, 1, 2], vec![4, 1, 1, 4])
                        };
                        let radix = if variant == 3 {
                            Radix::Object
                        } else {
                            Radix::Set
                        };
                        bind(at(&y_path), Let, y, vec![4], radix);
                        bind(at(&idle_path), Let, "idle", vec![4], radix);
                    }
                    let fresh_id = let_id + if legacy || header_binder { 1 } else { 3 };
                    if !header_binder {
                        for (index, (path, local, _)) in binders.iter().enumerate() {
                            let binder = ast.node_view(at(path)).unwrap();
                            bind(
                                binder.children()[usize::from(multi)],
                                FunctionBinder,
                                if variant == 7 && index == 1 { y } else { x },
                                local.clone(),
                                Radix::Set,
                            );
                            if multi {
                                bind(
                                    binder.children()[3],
                                    FunctionBinder,
                                    y,
                                    local.clone(),
                                    if variant == 3 {
                                        Radix::Object
                                    } else {
                                        Radix::Set
                                    },
                                );
                            }
                        }
                    }
                    assert_eq!(scope.bindings().len(), declarations.len());
                    for (index, (row, (node, kind, spelling, local, radix))) in
                        scope.bindings().iter().zip(&declarations).enumerate()
                    {
                        let syntax = ast.node_view(*node).unwrap();
                        assert_eq!(syntax.as_token().unwrap().text.as_ref(), spelling);
                        assert_eq!(row.id().index(), index);
                        assert_eq!(row.ordinal(), index);
                        assert_eq!(row.node(), *node);
                        assert_eq!(row.range(), syntax.range());
                        assert_eq!(row.kind(), *kind);
                        assert_eq!(row.spelling(), spelling);
                        assert_eq!(row.scope(), local);
                        assert_eq!(row.declared_type().unwrap().radix(), *radix);
                        assert!(row.declared_type().unwrap().attributes().is_empty());
                        assert!(row.arity().is_none() && row.captures().is_empty());
                    }
                    let mut references = Vec::new();
                    if !header_binder {
                        references.extend([
                            (at(&[3, 0, 3, 0, 0]), quantifier_id, vec![0]),
                            (at(&[3, 0, 3, 2, 0]), quantifier_id, vec![0]),
                        ]);
                    }
                    if variant == 11 {
                        references.push((at(&[4, 2, 1, 0, 0]), let_id, vec![4]));
                    }
                    let (body_path, body_scope, _) = binders.last().unwrap();
                    let body_index = if multi { 6 } else { 2 };
                    let body = [body_path.as_slice(), &[body_index]].concat();
                    if header_binder {
                        references.push((at(&[body.as_slice(), &[0]].concat()), 2, vec![0, 2]));
                    } else if matches!(variant, 1 | 13) {
                        references.extend([
                            (
                                at(&[body.as_slice(), &[0, 1, 0]].concat()),
                                fresh_id,
                                body_scope.clone(),
                            ),
                            (
                                at(&[body.as_slice(), &[0, 3, 0]].concat()),
                                fresh_id + 1,
                                body_scope.clone(),
                            ),
                        ]);
                    } else {
                        if variant == 5 {
                            references.push((
                                at(&[binder_path.as_slice(), &[2, 0]].concat()),
                                fresh_id,
                                vec![4, 2],
                            ));
                        }
                        let selected = match variant {
                            3..=5 => fresh_id + 1,
                            6 => let_id + 1,
                            _ => fresh_id,
                        };
                        references.push((
                            at(&[body.as_slice(), &[0]].concat()),
                            selected,
                            body_scope.clone(),
                        ));
                    }
                    if variant != 5 {
                        references.push((
                            at(&[equation_path.as_slice(), &[2, 0]].concat()),
                            if header_binder { 0 } else { let_id },
                            if header_binder { vec![0] } else { vec![4] },
                        ));
                    }
                    if header_binder {
                        references.extend([
                            (at(&[4, 2, 1, 0, 0, 0, 0]), let_id, vec![4]),
                            (at(&[4, 2, 1, 0, 0, 2, 0]), let_id, vec![4]),
                        ]);
                    }
                    assert_eq!(scope.references().len(), references.len());
                    for (index, (row, (node, binding, local))) in
                        scope.references().iter().zip(&references).enumerate()
                    {
                        assert_eq!(row.id().index(), index);
                        assert_eq!(row.ordinal(), index);
                        assert_eq!(row.node(), *node);
                        assert_eq!(row.range(), ast.node_view(*node).unwrap().range());
                        assert_eq!(row.spelling(), declarations[*binding].2);
                        assert_eq!(row.scope(), &LocalTermScope::new(local.clone()));
                        assert_eq!(row.binding().index(), *binding);
                        assert_eq!(
                            row.kind(),
                            mizar_resolve::names::SourceVariableReferenceKind::Term
                        );
                    }
                    expected.sort_by(|left, right| {
                        (left.0.path().len(), left.0.path())
                            .cmp(&(right.0.path().len(), right.0.path()))
                    });
                    let scoped = build(source, symbols, scope)
                        .unwrap_or_else(|error| panic!("RED scoped Binder case {case}: {error}"));
                    let flat = Semantics::occurrence_binding_env(scope);
                    let context_ids = expected
                        .iter()
                        .enumerate()
                        .map(|(index, row)| (row.0.clone(), BindingContextId::new(index)))
                        .collect::<std::collections::BTreeMap<_, _>>();
                    assert_eq!(scoped.source_id(), source.source_id());
                    assert_eq!(scoped.module_id(), source.module());
                    assert_eq!(scoped.diagnostics(), flat.diagnostics());
                    assert_eq!(scoped.contexts().len(), expected.len());
                    assert_eq!(scoped.bindings().len(), flat.bindings().len());
                    for ((id, row), (local, owner, layer, parent)) in
                        scoped.contexts().iter().zip(&expected)
                    {
                        assert_eq!(row.id, id);
                        assert_eq!(id, context_ids[local]);
                        assert_eq!(format!("{:?}", row.owner), *owner);
                        assert_eq!(row.layer, *layer);
                        assert_eq!(row.parent, parent.as_ref().map(|scope| context_ids[scope]));
                        assert_eq!(row.lexical_scope.as_ref(), parent.as_ref().map(|_| local));
                        assert_eq!(row.recovery, BindingContextRecovery::Normal);
                        let mut ancestors = vec![local.clone()];
                        while let Some(parent) = &expected
                            .iter()
                            .find(|context| context.0 == *ancestors.last().unwrap())
                            .unwrap()
                            .3
                        {
                            ancestors.push(parent.clone());
                        }
                        let own = declarations
                            .iter()
                            .enumerate()
                            .filter(|(_, declaration)| &declaration.3 == local)
                            .map(|(index, _)| BindingId::new(index))
                            .collect::<Vec<_>>();
                        let visible = declarations
                            .iter()
                            .enumerate()
                            .filter(|(_, declaration)| ancestors.contains(&declaration.3))
                            .map(|(index, _)| BindingId::new(index))
                            .collect::<Vec<_>>();
                        assert_eq!(row.bindings, own);
                        assert_eq!(row.visible_bindings, visible);
                    }
                    for ((id, row), declaration) in scoped.bindings().iter().zip(scope.bindings()) {
                        let mut unchanged = flat.bindings().get(id).unwrap().clone();
                        unchanged.owner_context = context_ids[declaration.scope()];
                        assert_eq!(row, &unchanged);
                        assert_eq!(row.type_site, BindingTypeSite::Missing);
                        if declaration.kind() == SourceVariableBindingKind::FunctionBinder {
                            let binder = ast
                                .node_views()
                                .find(|view| {
                                    view.kind() == &K::BinderExpression
                                        && view.children().contains(&declaration.node())
                                })
                                .unwrap();
                            let body = *binder.children().last().unwrap();
                            assert_eq!(maps.0.get(&body), Some(declaration.scope()));
                            let incoming = maps.0.get(&declaration.node()).unwrap();
                            assert_eq!(incoming, &maps.0[&binder.id()]);
                            assert_ne!(incoming, declaration.scope());
                            let winner = scope
                                .bindings()
                                .iter()
                                .filter(|prior| {
                                    prior.spelling() == declaration.spelling()
                                        && incoming.path().starts_with(prior.scope().path())
                                        && prior.range().end <= declaration.range().start
                                })
                                .max_by_key(|prior| (prior.scope().path().len(), prior.ordinal()))
                                .unwrap();
                            let clock = scope
                                .bindings()
                                .iter()
                                .filter(|prior| prior.range().end <= declaration.range().start)
                                .count();
                            assert_eq!(
                                scoped
                                    .lookup(&BindingLookupSite::new(
                                        declaration.spelling(),
                                        context_ids[incoming],
                                        Some(incoming.clone()),
                                        clock
                                    ))
                                    .unwrap(),
                                BindingLookupResult::Local(BindingId::new(winner.id().index()))
                            );
                            assert_ne!(winner.id(), declaration.id());
                        }
                    }
                    let mut different_clocks = false;
                    for reference in scope.references() {
                        assert_eq!(maps.0.get(&reference.node()), Some(reference.scope()));
                        let clock = scope
                            .bindings()
                            .iter()
                            .filter(|binding| binding.range().end <= reference.range().start)
                            .count();
                        different_clocks |= clock != reference.ordinal();
                        assert_eq!(
                            scoped
                                .lookup(&BindingLookupSite::new(
                                    reference.spelling(),
                                    context_ids[reference.scope()],
                                    Some(reference.scope().clone()),
                                    clock
                                ))
                                .unwrap(),
                            BindingLookupResult::Local(BindingId::new(reference.binding().index()))
                        );
                    }
                    assert!(different_clocks);
                    use mizar_checker::{
                        source_term::{
                            SourcePrimaryTermError as PrimaryError, SourcePrimaryTermHandoffInput,
                            SourcePrimaryTermId, SourcePrimaryTermInput,
                            SourcePrimaryTermProducer as Primary, SourcePrimaryTermReferenceId,
                            SourcePrimaryTermReferenceInput,
                        },
                        typed_ast::{TypedNode, TypedNodeId, TypingState},
                    };
                    // These caller-owned leaves are untrusted inputs, not a typed phase.
                    let arena = TypedArena::try_new(
                        None,
                        references
                            .iter()
                            .map(|(node, _, _)| {
                                TypedNode::new(
                                    "source.term.variable-reference",
                                    SourceAnchor::Range(ast.node_view(*node).unwrap().range()),
                                )
                                .with_resolved_node(source.resolved_node_for(*node).unwrap())
                            })
                            .collect(),
                    )
                    .unwrap();
                    let input = SourcePrimaryTermHandoffInput {
                        source_id: source.source_id(),
                        module_id: source.module().clone(),
                        terms: references
                            .iter()
                            .enumerate()
                            .map(|(index, (node, binding, local))| SourcePrimaryTermInput {
                                site: TypedSiteRef::Node(TypedNodeId::new(index)),
                                source_range: ast.node_view(*node).unwrap().range(),
                                source_ordinal: index,
                                context: context_ids[&LocalTermScope::new(local.clone())],
                                recovery: SourcePrimaryTermRecovery::Normal,
                                spelling: declarations[*binding].2.clone(),
                                kind: SourcePrimaryTermKind::VariableReference,
                                role: SourcePrimaryTermRole::Value,
                                parent: None,
                            })
                            .collect(),
                        references: references
                            .iter()
                            .enumerate()
                            .map(|(index, (_, binding, _))| SourcePrimaryTermReferenceInput {
                                term: SourcePrimaryTermId::new(index),
                                binding: BindingId::new(*binding),
                                role: SourcePrimaryTermReferenceRole::Variable,
                            })
                            .collect(),
                        numeric_type_requests: Vec::new(),
                    };
                    let primary_before = (arena.clone(), scoped.clone());
                    let selects_fresh = references
                        .iter()
                        .any(|(_, binding, _)| declarations[*binding].1 == FunctionBinder);
                    let generic = Primary::build(input.clone(), &scoped, &arena);
                    assert_eq!(
                        generic.is_err(),
                        selects_fresh,
                        "Generic admission case {case}"
                    );
                    let primary_build = Primary::build_from_scoped_theorem_occurrences;
                    if header_binder {
                        // Active Reserve identity remains outside the scoped primary profile.
                        assert_eq!(
                            primary_build(input.clone(), &scoped, &arena, source, symbols, scope)
                                .unwrap_err(),
                            PrimaryError::InvalidReference {
                                reference: SourcePrimaryTermReferenceId::new(1)
                            }
                        );
                    } else {
                        let primary = primary_build(
                            input.clone(),
                            &scoped,
                            &arena,
                            source,
                            symbols,
                            scope,
                        )
                        .unwrap_or_else(|error| {
                            panic!(
                                "RED scoped primary Binder case {case} variant {variant}: {error:?}"
                            )
                        });
                        assert_eq!(primary.source_id(), source.source_id());
                        assert_eq!(primary.module_id(), source.module());
                        assert_eq!(primary.terms().len(), references.len());
                        assert_eq!(primary.references().len(), references.len());
                        assert!(primary.numeric_type_requests().is_empty());
                        for (index, (node, binding, local)) in references.iter().enumerate() {
                            let term = primary
                                .terms()
                                .get(SourcePrimaryTermId::new(index))
                                .unwrap();
                            assert_eq!(term.site(), &TypedSiteRef::Node(TypedNodeId::new(index)));
                            assert_eq!(term.source_range(), ast.node_view(*node).unwrap().range());
                            assert_eq!(term.source_ordinal(), index);
                            assert_eq!(
                                term.context(),
                                context_ids[&LocalTermScope::new(local.clone())]
                            );
                            assert_eq!(term.recovery(), SourcePrimaryTermRecovery::Normal);
                            assert_eq!(term.spelling(), declarations[*binding].2);
                            assert_eq!(term.kind(), SourcePrimaryTermKind::VariableReference);
                            assert_eq!(term.role(), SourcePrimaryTermRole::Value);
                            assert!(term.parent().is_none());
                            let (_, reference) = primary.references().iter().nth(index).unwrap();
                            assert_eq!(reference.term(), SourcePrimaryTermId::new(index));
                            assert_eq!(reference.binding(), BindingId::new(*binding));
                            assert_eq!(reference.role(), SourcePrimaryTermReferenceRole::Variable);
                            assert_eq!(
                                reference.lexical_scope(),
                                Some(&LocalTermScope::new(local.clone()))
                            );
                            assert_eq!(reference.use_ordinal(), index);
                        }
                        assert_eq!(
                            primary_build(input.clone(), &scoped, &arena, source, symbols, scope)
                                .unwrap(),
                            primary
                        );
                        if case == binder_cases {
                            for mutation in 0..6 {
                                let mut changed = input.clone();
                                let mut nodes = arena
                                    .iter()
                                    .map(|(_, node)| node.clone())
                                    .collect::<Vec<_>>();
                                match mutation {
                                    0 => {
                                        changed.terms.pop();
                                        changed.references.pop();
                                    }
                                    1 => changed.references.push(changed.references[2].clone()),
                                    2 => changed.references[2].binding = BindingId::new(let_id),
                                    3 => nodes[2].resolved_node = nodes[3].resolved_node,
                                    4 => {
                                        changed.terms[2].context =
                                            context_ids[&LocalTermScope::new(vec![4])]
                                    }
                                    _ => changed.terms[2].site = changed.terms[3].site.clone(),
                                }
                                let changed_arena = TypedArena::try_new(None, nodes).unwrap();
                                assert!(
                                    primary_build(
                                        changed,
                                        &scoped,
                                        &changed_arena,
                                        source,
                                        symbols,
                                        scope
                                    )
                                    .is_err(),
                                    "primary mutation {mutation}"
                                );
                            }
                            let mut ancestor = input.clone();
                            ancestor.terms[2].context = context_ids[&LocalTermScope::new(vec![4])];
                            ancestor.references[2].binding = BindingId::new(let_id);
                            assert!(Primary::build(ancestor.clone(), &scoped, &arena).is_ok());
                            assert!(
                                primary_build(ancestor, &scoped, &arena, source, symbols, scope)
                                    .is_err()
                            );
                            assert!(
                                primary_build(input.clone(), &flat, &arena, source, symbols, scope)
                                    .is_err()
                            );
                        }
                    }
                    assert_eq!((arena.clone(), scoped.clone()), primary_before);
                    assert!(
                        arena
                            .iter()
                            .all(|(_, node)| node.typing == TypingState::Unknown
                                && node.recovery == NodeRecoveryState::Normal
                                && node.children.is_empty()
                                && node.links.context.is_none()
                                && node.links.type_entry.is_none()
                                && node.links.facts.is_empty()
                                && node.links.coercions.is_empty()
                                && node.links.initial_obligations.is_empty()
                                && node.links.diagnostics.is_empty())
                    );

                    // Upstream replay errors win over metadata-only admission.
                    let (_, other_source, other_symbols, other_scope) = &prepared[0];
                    let empty = mizar_resolve::env::SymbolEnv::new(
                        source.module().clone(),
                        mizar_resolve::env::SymbolEnvIndexes::default(),
                    );
                    for (candidate, environment, occurrences) in [
                        (source, &empty, scope),
                        (other_source, symbols, scope),
                        (source, other_symbols, scope),
                        (source, symbols, other_scope),
                    ] {
                        assert!(
                            primary_build(
                                input.clone(),
                                &scoped,
                                &arena,
                                candidate,
                                environment,
                                occurrences
                            )
                            .is_err()
                        );
                        assert_eq!(
                            build(candidate, environment, occurrences).unwrap_err(),
                            source_proof_scope_maps(candidate, environment, occurrences)
                                .unwrap_err()
                        );
                    }
                    if case == binder_cases + 1 || case == distinct_reserves {
                        use mizar_syntax::SurfaceAstBuilder;
                        let binder = ast
                            .node_views()
                            .find(|view| view.kind() == &K::BinderExpression)
                            .unwrap();
                        let head = binder.children()[1];
                        let body = *binder.children().last().unwrap();
                        for mutation in if case == distinct_reserves {
                            4..5
                        } else {
                            0..4
                        } {
                            let mut builder = SurfaceAstBuilder::new(ast.source_id);
                            let mut rebuilt = Vec::new();
                            for view in ast.node_views() {
                                let children = view
                                    .children()
                                    .iter()
                                    .map(|id| rebuilt[id.index()])
                                    .collect();
                                rebuilt.push(if let Some(token) = view.as_token() {
                                    if mutation == 0 && view.id() == head {
                                        builder.add_recovered_token(
                                            token.kind,
                                            token.text.clone(),
                                            view.range(),
                                        )
                                    } else {
                                        let swap = (mutation == 1
                                            && binder.children().contains(&view.id()))
                                            || (mutation == 2
                                                && ast.node_view(body).unwrap().range().start
                                                    <= view.range().start
                                                && view.range().end
                                                    <= ast.node_view(body).unwrap().range().end)
                                            || mutation == 3;
                                        builder.add_token(
                                            token.kind,
                                            if mutation == 4 && token.text.as_ref() == "other" {
                                                "spare"
                                            } else if swap && token.text.as_ref() == "x" {
                                                "y"
                                            } else if swap && token.text.as_ref() == "y" {
                                                "x"
                                            } else {
                                                token.text.as_ref()
                                            },
                                            view.range(),
                                        )
                                    }
                                } else {
                                    builder.add_node(view.kind().clone(), view.range(), children)
                                });
                            }
                            let changed =
                                builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None);
                            let changed_source =
                                SurfaceResolvedArena::lower(&changed, source.module()).unwrap();
                            let shells =
                                DeclarationShellCollector::new(&changed, source.module()).collect();
                            let changed_symbols = SignatureProjectionExtractor::new(
                                &changed,
                                &shells,
                                NamespacePath::new(source.module().path().as_str()),
                            )
                            .collect(source.module())
                            .into_env();
                            if mutation == 0 {
                                assert_eq!(
                                    build(&changed_source, &changed_symbols, scope).unwrap_err(),
                                    source_proof_scope_maps(
                                        &changed_source,
                                        &changed_symbols,
                                        scope
                                    )
                                    .unwrap_err()
                                );
                            } else {
                                let changed_scope =
                                    SourceVariableScopeResolver::resolve_proof_occurrences(
                                        SourceVariableScopeInput::new(
                                            &changed,
                                            source.module(),
                                            &changed_symbols,
                                        ),
                                    )
                                    .unwrap();
                                source_proof_scope_maps(
                                    &changed_source,
                                    &changed_symbols,
                                    &changed_scope,
                                )?;
                                if mutation == 4 {
                                    let projected =
                                        Semantics::occurrence_binding_env(&changed_scope);
                                    assert_eq!(
                                        projected.bindings().len(),
                                        changed_scope.bindings().len()
                                    );
                                    assert_eq!(
                                        changed_scope
                                            .bindings()
                                            .iter()
                                            .filter(|binding| binding.kind()
                                                == SourceVariableBindingKind::Reserve
                                                && binding.spelling() == "spare")
                                            .count(),
                                        2
                                    );
                                    assert_eq!(
                                        build(&changed_source, &changed_symbols, &changed_scope)
                                            .unwrap_err(),
                                        "theorems.bindings.invalid_lexical_association"
                                    );
                                } else {
                                    assert!(
                                        build(&changed_source, &changed_symbols, &changed_scope)
                                            .is_ok()
                                    );
                                }
                                for (candidate, environment, occurrences) in [
                                    (source, symbols, &changed_scope),
                                    (&changed_source, &changed_symbols, scope),
                                ] {
                                    assert_eq!(
                                        build(candidate, environment, occurrences).unwrap_err(),
                                        source_proof_scope_maps(
                                            candidate,
                                            environment,
                                            occurrences
                                        )
                                        .unwrap_err()
                                    );
                                }
                            }
                            assert_eq!(build(source, symbols, scope)?, scoped);
                        }
                    }
                    assert_eq!(build(source, symbols, scope)?, scoped);
                    assert_eq!(mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities_with_scoped_inference(source, symbols, scope).unwrap_err(), "theorems.equality.unsupported_source_types");
                    if case == binder_cases + 8 {
                        use mizar_session::{
                            SessionIdAllocator, SnapshotInput, SnapshotRegistry, SourceVersion,
                            ToolchainInfo, WorkspaceRoot, hash_text,
                        };
                        let policy = mizar_proof::policy::VerifierPolicy::default();
                        let registry = SnapshotRegistry::new();
                        let unit = &frontend.source;
                        let (snapshot, _lease) = registry
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
                                    dependency_artifacts: vec![],
                                    lockfile_hash: hash_text(
                                        &fs::read_to_string(
                                            Path::new(env!("CARGO_MANIFEST_DIR"))
                                                .join("../../Cargo.lock"),
                                        )
                                        .unwrap(),
                                    ),
                                    toolchain: ToolchainInfo::new("fresh-binder-metadata-test"),
                                    verifier_config_hash: policy.policy_fingerprint().hash(),
                                },
                            )
                            .unwrap();
                        let pristine = snapshot.clone();
                        assert_eq!(mizar_proof::status::source_theorem_reflexivity_witness_draft_from_original(source, symbols, &snapshot, &policy, "").unwrap_err(), "theorems.equality.unsupported_source_types");
                        assert_eq!(snapshot, pristine);
                        assert!(!root.join("proof-witnesses").exists());
                    }
                }
                assert_eq!(
                    before,
                    (
                        source.clone(),
                        symbols.clone(),
                        scope.clone(),
                        ast.snapshot_text(),
                        frontend.source.clone()
                    )
                );
            }
            assert_eq!(
                SourceTypeProducer::build_source_theorem_equalities(source, symbols, scope)
                    .unwrap_err(),
                "theorems.equality.unsupported_source_types",
                "negative {case}"
            );
            continue;
        }
        let ast = frontend.ast.as_ref().unwrap();
        let lower = SourceTypeProducer::build_source_theorem_equalities(source, symbols, scope)
            .unwrap_or_else(|error| panic!("RED-L genuine case {case}: {error}"));
        let (env, inference, typed, metadata) = &lower;
        if scope
            .bindings()
            .iter()
            .any(|binding| binding.kind() == SourceVariableBindingKind::Reserve)
        {
            let (_, checked, declared, declared_metadata) = mizar_checker::type_checker::DeclarationChecker::check_source_theorem_equalities_with_typed_declarations(source, symbols, scope)?;
            assert_eq!(declared.source_context(), typed.source_context());
            assert_eq!(declared.source_type(), typed.source_type());
            assert_eq!(declared.source_term(), typed.source_term());
            assert_eq!(
                declared.source_atomic_formula(),
                typed.source_atomic_formula()
            );
            assert_eq!(&declared_metadata, metadata);
            assert_eq!(checked.terms(), inference.terms());
        }
        let context = typed.source_context().unwrap();
        let types = typed.source_type().unwrap();
        let primary = typed.source_term().unwrap();
        let atomic = typed.source_atomic_formula().unwrap();
        let maps = source_proof_scope_maps(source, symbols, scope)?;
        let site = |id| {
            let resolved = source.resolved_node_for(id).unwrap();
            let matches = typed
                .nodes()
                .iter()
                .filter(|(_, row)| row.resolved_node == Some(resolved))
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1);
            TypedSiteRef::Node(matches[0].0)
        };
        let spelling = |id| {
            let range = ast.node_view(id).unwrap().range();
            let mut tokens = ast
                .node_views()
                .filter(|view| {
                    view.as_token().is_some()
                        && range.start <= view.range().start
                        && view.range().end <= range.end
                })
                .collect::<Vec<_>>();
            tokens.sort_by_key(|view| view.range().start);
            tokens
                .iter()
                .map(|view| view.as_token().unwrap().text.as_ref())
                .collect::<Vec<_>>()
                .join(" ")
        };
        let lexical = |id| {
            let mut cursor = id;
            loop {
                if let Some(local) = maps.0.get(&cursor) {
                    return env
                        .contexts()
                        .iter()
                        .find(|(_, row)| row.lexical_scope.as_ref() == Some(local))
                        .unwrap()
                        .0;
                }
                cursor = ast
                    .node_views()
                    .find(|view| view.children().contains(&cursor))
                    .unwrap()
                    .id();
            }
        };
        assert_eq!(context.binding_env(), env);
        assert_eq!(context.source_id(), source.source_id());
        assert_eq!(context.module_id(), source.module());
        assert_eq!(types.source_id(), source.source_id());
        assert_eq!(types.module_id(), source.module());
        assert_eq!(primary.source_id(), source.source_id());
        assert_eq!(primary.module_id(), source.module());
        assert_eq!(atomic.source_id(), source.source_id());
        assert_eq!(atomic.module_id(), source.module());
        assert_eq!(typed.contexts(), context.local_contexts());
        assert_eq!(context.context_links().len(), env.contexts().len());
        for (id, _) in env.contexts().iter() {
            let link = context.context_links().get(id).unwrap();
            assert_eq!(link.binding_context, id);
            assert!(typed.contexts().get(link.local_context).is_some());
        }
        let (terms, atoms, bindings, contexts) =
            object.map_or_else(|| counts[case], |profile| profile.5);
        assert_eq!(
            (
                primary.terms().len(),
                atomic.formulas().len(),
                env.bindings().len(),
                env.contexts().len()
            ),
            (terms, atoms, bindings, contexts)
        );
        assert_eq!(
            (
                inference.terms().len(),
                inference.formulas().len(),
                metadata.len()
            ),
            (terms, atoms, terms + atoms)
        );
        let owners = ast
            .node_views()
            .filter(|view| matches!(view.kind(), K::TheoremItem | K::LemmaItem))
            .count();
        assert_eq!(
            context.items().len(),
            owners
                + ast
                    .node_views()
                    .filter(|view| view.kind() == &K::ReserveItem)
                    .count()
        );
        let mut items = ast
            .node_views()
            .filter(|view| matches!(view.kind(), K::ReserveItem | K::TheoremItem | K::LemmaItem))
            .collect::<Vec<_>>();
        items.sort_by_key(|view| view.range().start);
        for (ordinal, ((id, row), syntax)) in context.items().iter().zip(&items).enumerate() {
            assert_eq!(
                (row.shell_ordinal, row.source_range, row.site.clone()),
                (ordinal, syntax.range(), site(syntax.id()))
            );
            assert_eq!(
                row.predecessor,
                ordinal
                    .checked_sub(1)
                    .map(mizar_checker::source_context::SourceItemId::new)
            );
            assert_eq!(id.index(), ordinal);
            assert_eq!(
                row.role,
                match syntax.kind() {
                    K::ReserveItem => mizar_checker::source_context::SourceItemRole::Reserve,
                    K::TheoremItem => mizar_checker::source_context::SourceItemRole::Theorem,
                    K::LemmaItem => mizar_checker::source_context::SourceItemRole::Lemma,
                    _ => unreachable!(),
                }
            );
        }
        assert_eq!(
            context
                .declarations()
                .iter()
                .map(|(_, row)| row.declaration_range)
                .collect::<Vec<_>>(),
            scope
                .bindings()
                .iter()
                .map(|row| row.range())
                .collect::<Vec<_>>()
        );
        assert_eq!(context.declarations().len(), scope.bindings().len());
        assert_eq!(env.bindings().len(), scope.bindings().len());
        assert_eq!(types.applications().len(), scope.bindings().len());
        assert_eq!(
            types.expressions().len(),
            ast.node_views()
                .filter(|view| view.kind() == &K::TypeExpression)
                .count()
        );
        assert!(
            types.arguments().is_empty()
                && types.definition_returns().is_empty()
                && types.mode_rhs().is_empty()
                && types.structure_members().is_empty()
        );
        assert_eq!(
            env.contexts().len(),
            1 + owners
                + ast
                    .node_views()
                    .filter(|view| matches!(view.kind(), K::QuantifiedFormula(_)))
                    .count()
        );
        assert_eq!(primary.terms().len(), scope.references().len());
        assert_eq!(primary.references().len(), scope.references().len());
        assert!(primary.numeric_type_requests().is_empty());
        assert!(typed.source_node_entries().is_none());
        assert!(
            typed.source_composite_formula().is_none()
                && typed.source_formula_composition().is_none()
                && typed.source_statement().is_none()
        );
        assert!(env.diagnostics().is_empty() && inference.diagnostics().is_empty());
        assert!(inference.candidate_sets().is_empty() && inference.facts().is_empty());
        assert!(
            typed.facts().is_empty()
                && typed.coercions().is_empty()
                && typed.initial_obligations().is_empty()
                && typed.diagnostics().is_empty()
        );
        assert_eq!(typed.types(), inference.type_entries());
        assert!(
            inference
                .terms()
                .iter()
                .all(|(_, row)| row.status == TermStatus::Inferred && row.deferred.is_empty())
        );
        assert!(
            inference
                .formulas()
                .iter()
                .all(|(_, row)| row.status == FormulaStatus::Checked && row.deferred.is_empty())
        );
        for (_, entry) in typed.types().iter() {
            assert_eq!(entry.status, TypeStatus::Known);
            let TypeEntryActual::Known(actual) = entry.actual else {
                panic!("unknown Set type")
            };
            let actual = inference.normalized_types().get(actual).unwrap();
            if object.is_none() {
                assert_eq!(actual.head, TypeHeadRef::BuiltinSet);
            }
            assert!(
                actual.args.is_empty()
                    && actual.attributes.positive().is_empty()
                    && actual.attributes.negative().is_empty()
            );
        }
        for original in scope.bindings() {
            let identity = if original.kind() == SourceVariableBindingKind::Reserve {
                BinderIdentity::ReservedVariable {
                    spelling: original.spelling().into(),
                    declaration_range: original.range(),
                }
            } else {
                BinderIdentity::ResolverLocal {
                    scope: original.scope().clone(),
                    ordinal: original.ordinal(),
                    declaration_range: original.range(),
                }
            };
            let matches = env
                .bindings()
                .iter()
                .filter(|(_, row)| row.identity == identity)
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1);
            let (binding, entry) = matches[0];
            let segment = ast
                .node_views()
                .find(|view| {
                    matches!(
                        view.kind(),
                        K::ReserveSegment
                            | K::QuantifierVariableSegment
                            | K::QualifiedVariableSegment
                    ) && view.children().contains(&original.node())
                })
                .unwrap();
            assert_eq!(
                segment.kind(),
                &match original.kind() {
                    SourceVariableBindingKind::Reserve => K::ReserveSegment,
                    SourceVariableBindingKind::Quantifier => K::QuantifierVariableSegment,
                    SourceVariableBindingKind::Let => K::QualifiedVariableSegment,
                    kind => panic!("unexpected positive declaration {kind:?}"),
                }
            );
            let explicit = segment
                .children()
                .iter()
                .filter_map(|id| ast.node_view(*id))
                .find(|view| view.kind() == &K::TypeExpression);
            let default = || {
                ast.node_views()
                    .find(|view| {
                        view.kind() == &K::ReserveSegment
                            && view.range().end < original.range().start
                            && view
                                .children()
                                .iter()
                                .any(|id| spelling(*id) == original.spelling())
                    })
                    .unwrap()
            };
            let type_syntax = explicit.unwrap_or_else(|| {
                ast.node_view(*default().children().last().unwrap())
                    .unwrap()
            });
            assert_eq!(
                entry.type_site,
                if original.kind() == SourceVariableBindingKind::Reserve || explicit.is_none() {
                    BindingTypeSite::Source(type_syntax.range())
                } else {
                    BindingTypeSite::Missing
                }
            );
            assert_eq!(
                entry.status,
                mizar_checker::binding_env::BindingStatus::Active
            );
            assert_eq!(entry.visible_after_ordinal, original.ordinal());
            let declaration = context
                .declarations()
                .iter()
                .find(|(_, row)| row.binding == binding)
                .unwrap()
                .1;
            assert_eq!(declaration.site, site(original.node()));
            assert_eq!(declaration.spelling, original.spelling());
            assert_eq!(declaration.source_ordinal, original.ordinal());
            assert_eq!(declaration.declaration_range, original.range());
            assert_eq!(declaration.binding_context, entry.owner_context);
            assert_eq!(declaration.written_type_range, type_syntax.range());
            let predecessor = scope
                .bindings()
                .iter()
                .filter(|prior| {
                    prior.ordinal() < original.ordinal()
                        && prior.spelling() == original.spelling()
                        && prior.scope() != original.scope()
                        && original.scope().path().starts_with(prior.scope().path())
                })
                .max_by_key(|prior| prior.scope().path().len());
            assert_eq!(
                declaration.shadowed_binding.map(|id| env
                    .bindings()
                    .get(id)
                    .unwrap()
                    .declaration_range),
                predecessor.map(|prior| prior.range())
            );
            assert_eq!(
                declaration.role,
                match original.kind() {
                    SourceVariableBindingKind::Reserve =>
                        mizar_checker::source_context::SourceBindingSiteRole::ReserveDefault,
                    SourceVariableBindingKind::Quantifier =>
                        mizar_checker::source_context::SourceBindingSiteRole::QuantifierBinder,
                    _ => mizar_checker::source_context::SourceBindingSiteRole::ProofLet,
                }
            );
            let application = types
                .applications()
                .iter()
                .find(|(_, row)| row.binding() == binding)
                .unwrap()
                .1;
            assert_eq!(application.source_ordinal(), original.ordinal());
            if explicit.is_none() && original.kind() != SourceVariableBindingKind::Reserve {
                let matching = scope
                    .bindings()
                    .iter()
                    .filter(|hint| {
                        hint.kind() == SourceVariableBindingKind::Reserve
                            && hint.spelling() == original.spelling()
                            && hint.range().end < original.range().start
                    })
                    .collect::<Vec<_>>();
                assert_eq!(matching.len(), 1);
                let hint = matching[0];
                let hint_application = types
                    .applications()
                    .iter()
                    .find(|(_, row)| row.source_ordinal() == hint.ordinal())
                    .unwrap()
                    .1;
                assert_eq!(application.root(), hint_application.root());
            }
            let written = types.expressions().get(application.root()).unwrap();
            let syntax = ast
                .node_views()
                .find(|view| {
                    view.kind() == &K::TypeExpression
                        && view.range() == declaration.written_type_range
                })
                .unwrap();
            let head = ast.node_view(syntax.children()[0]).unwrap();
            assert_eq!(written.site(), &site(syntax.id()));
            assert_eq!(written.source_range(), syntax.range());
            assert_eq!(written.head_site(), &site(head.id()));
            assert_eq!(written.head_range(), head.range());
            let physical_context = if explicit.is_none() {
                let root = env
                    .contexts()
                    .iter()
                    .find(|(_, row)| row.owner == BindingContextOwner::Module)
                    .unwrap()
                    .0;
                context.context_links().get(root).unwrap().local_context
            } else {
                declaration.local_context
            };
            assert_eq!(
                typed
                    .nodes()
                    .node(written.site().node())
                    .unwrap()
                    .links
                    .context,
                Some(physical_context)
            );
            assert_eq!(
                typed
                    .nodes()
                    .node(written.head_site().node())
                    .unwrap()
                    .links
                    .context,
                Some(physical_context)
            );
            let head = if expected_head(original.id().index()) == TypeHeadRef::BuiltinObject {
                SourceTypeHead::BuiltinObject
            } else {
                SourceTypeHead::BuiltinSet
            };
            assert_eq!(written.head(), &head);
            assert_eq!(
                spelling(syntax.id()),
                if head == SourceTypeHead::BuiltinObject {
                    "object"
                } else {
                    "set"
                }
            );
        }
        for (ordinal, original) in scope.references().iter().enumerate() {
            let (term_id, term) = primary
                .terms()
                .iter()
                .find(|(_, row)| row.site() == &site(original.node()))
                .unwrap();
            let (_, reference) = primary
                .references()
                .iter()
                .find(|(_, row)| row.term() == term_id)
                .unwrap();
            let winner = &scope.bindings()[original.binding().index()];
            assert_eq!(term.source_range(), original.range());
            assert_eq!(term.source_ordinal(), ordinal);
            assert_eq!(term.spelling(), winner.spelling());
            assert_eq!(term.kind(), SourcePrimaryTermKind::VariableReference);
            assert_eq!(term.role(), SourcePrimaryTermRole::Value);
            assert_eq!(term.recovery(), SourcePrimaryTermRecovery::Normal);
            assert!(term.parent().is_none());
            assert_eq!(term.context(), lexical(original.node()));
            assert_eq!(reference.use_ordinal(), original.ordinal());
            assert_eq!(reference.role(), SourcePrimaryTermReferenceRole::Variable);
            assert_eq!(reference.lexical_scope(), Some(original.scope()));
            assert_eq!(
                env.bindings().get(reference.binding()).unwrap().identity,
                BinderIdentity::ResolverLocal {
                    scope: winner.scope().clone(),
                    ordinal: winner.ordinal(),
                    declaration_range: winner.range()
                }
            );
        }
        if let Some(profile) = object {
            use mizar_checker::type_checker::{
                DeclarationChecker, DeclarationStatus, check_source_theorem_metadata,
                check_source_theorem_registration_closure,
            };
            let compilation = *ast
                .node_view(ast.root().unwrap())
                .unwrap()
                .children()
                .last()
                .unwrap();
            let item_list = ast.node_view(compilation).unwrap().children()[0];
            let physical = profile
                .2
                .iter()
                .map(|path| {
                    path.iter().fold(item_list, |node, child| {
                        ast.node_view(node).unwrap().children()[*child]
                    })
                })
                .collect::<Vec<_>>();
            assert_eq!(types.expressions().len(), physical.len());
            assert_eq!(context.declarations().len(), profile.3.len());
            for (index, node) in physical.iter().enumerate() {
                let syntax = ast.node_view(*node).unwrap();
                assert_eq!(syntax.kind(), &K::TypeExpression);
                assert_eq!(spelling(*node), profile.1[index]);
                assert_eq!(syntax.children().len(), 1);
                let head = ast.node_view(syntax.children()[0]).unwrap();
                let token = ast.node_view(head.children()[0]).unwrap();
                assert_eq!(token.as_token().unwrap().text.as_ref(), profile.1[index]);
                let (_, written) = types
                    .expressions()
                    .iter()
                    .find(|(_, row)| row.site() == &site(*node))
                    .unwrap();
                assert_eq!(written.head_site(), &site(head.id()));
                assert_eq!(written.head_range(), token.range());
            }
            for (index, (_, declaration)) in context.declarations().iter().enumerate() {
                assert_eq!(
                    declaration.written_type_range,
                    ast.node_view(physical[profile.3[index]]).unwrap().range()
                );
            }
            assert_eq!(primary.references().len(), profile.4.len());
            for ((_, reference), binding) in primary.references().iter().zip(&profile.4) {
                assert_eq!(reference.binding().index(), *binding);
                let term = primary.terms().get(reference.term()).unwrap();
                let (_, inferred) = inference
                    .terms()
                    .iter()
                    .find(|(_, row)| &row.site == term.site())
                    .unwrap();
                let entry = inference.type_entries().get(inferred.type_entry).unwrap();
                let TypeEntryActual::Known(id) = entry.actual else {
                    panic!("known source head")
                };
                assert_eq!(
                    inference.normalized_types().get(id).unwrap().head,
                    expected_head(*binding)
                );
                let origin = physical[profile.3[*binding]];
                assert!(
                    inference
                        .type_entries()
                        .iter()
                        .any(|(_, row)| row.owner.node() == site(origin).node())
                );
            }
            let (direct_lower, checked) =
                DeclarationChecker::check_source_theorem_equalities(source, symbols, scope)?;
            assert_eq!(direct_lower, lower);
            let installed =
                DeclarationChecker::check_source_theorem_equalities_with_typed_declarations(
                    source, symbols, scope,
                )?;
            let (_, union, declared, declared_metadata) = &installed;
            assert_eq!(declared_metadata, metadata);
            assert_eq!(declared.source_context(), typed.source_context());
            assert_eq!(declared.source_type(), typed.source_type());
            assert_eq!(declared.source_term(), typed.source_term());
            assert_eq!(
                declared.source_atomic_formula(),
                typed.source_atomic_formula()
            );
            assert_eq!(declared.contexts(), typed.contexts());
            let heads = profile
                .1
                .iter()
                .map(|head| {
                    if *head == "object" {
                        TypeHeadRef::BuiltinObject
                    } else {
                        TypeHeadRef::BuiltinSet
                    }
                })
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(
                union
                    .normalized_types()
                    .iter()
                    .map(|(_, row)| row.head.clone())
                    .collect::<Vec<_>>(),
                heads.into_iter().collect::<Vec<_>>()
            );
            let remap = inference
                .normalized_types()
                .iter()
                .map(|(old, row)| {
                    let (id, canonical) = union
                        .normalized_types()
                        .iter()
                        .find(|(_, candidate)| candidate.head == row.head)
                        .unwrap();
                    assert_eq!(canonical.source, row.source);
                    assert_eq!(canonical.status, row.status);
                    assert!(canonical.args.is_empty() && canonical.attributes == row.attributes);
                    remapped_inference_id |= old != id;
                    (old, id)
                })
                .collect::<std::collections::BTreeMap<_, _>>();
            for (id, entry) in inference.type_entries().iter() {
                let mut expected = entry.clone();
                let TypeEntryActual::Known(actual) = entry.actual else {
                    panic!("known lower type")
                };
                expected.actual = TypeEntryActual::Known(remap[&actual]);
                expected.expected = entry.expected.map(|id| remap[&id]);
                assert_eq!(union.type_entries().get(id), Some(&expected));
                assert_eq!(declared.types().get(id), Some(&expected));
            }
            assert_eq!(union.type_entries().len(), inference.type_entries().len());
            for (id, term) in inference.terms().iter() {
                let mut expected = term.clone();
                expected.expected_type = term.expected_type.map(|id| remap[&id]);
                assert_eq!(union.terms().get(id), Some(&expected));
            }
            for (id, formula) in inference.formulas().iter() {
                let mut expected = formula.clone();
                expected.asserted_type = formula.asserted_type.map(|id| remap[&id]);
                for constraint in &mut expected.expected_types {
                    constraint.expected = remap[&constraint.expected];
                }
                assert_eq!(union.formulas().get(id), Some(&expected));
            }
            assert_eq!(checked.declarations().len(), profile.3.len());
            assert!(checked.diagnostics().is_empty() && checked.facts().is_empty());
            assert!(union.candidate_sets().is_empty() && union.facts().is_empty());
            for (_, declaration) in checked.declarations().iter() {
                assert_eq!(declaration.status, DeclarationStatus::Checked);
                let entry = checked
                    .type_entries()
                    .get(declaration.type_entry.unwrap())
                    .unwrap();
                let TypeEntryActual::Known(actual) = entry.actual else {
                    panic!("known declaration")
                };
                let head = expected_head(declaration.binding.index());
                assert_eq!(checked.normalized_types().get(actual).unwrap().head, head);
                if let Some((lower_id, _)) = inference
                    .normalized_types()
                    .iter()
                    .find(|(_, row)| row.head == head)
                {
                    distinct_decl_namespace_id |= lower_id != actual;
                }
                let node = declared.nodes().node(declaration.site.node()).unwrap();
                let installed_entry = declared
                    .types()
                    .get(node.links.type_entry.unwrap())
                    .unwrap();
                let TypeEntryActual::Known(installed_id) = installed_entry.actual else {
                    panic!("known installed declaration")
                };
                assert_eq!(
                    union.normalized_types().get(installed_id).unwrap().head,
                    head
                );
                assert_eq!(installed_entry.owner, entry.owner);
                assert_eq!(installed_entry.status, entry.status);
                assert_eq!(installed_entry.provenance, entry.provenance);
                assert_eq!(
                    installed_entry.expected.map(|id| &union
                        .normalized_types()
                        .get(id)
                        .unwrap()
                        .head),
                    entry
                        .expected
                        .map(|id| &checked.normalized_types().get(id).unwrap().head)
                );
            }
            let actual = check_source_theorem_metadata(source, symbols, scope)?;
            assert_eq!(
                (&actual.0, &actual.1, &actual.2),
                (&installed.0, &installed.1, &installed.2)
            );
            assert_eq!(actual.3.source_type(), declared.source_type());
            assert!(
                actual.3.checked_proofs().is_empty()
                    && actual.3.checked_formulas().is_empty()
                    && actual.3.statement_semantics().is_empty()
            );
            let closure =
                check_source_theorem_registration_closure(source, symbols, scope, declared)?;
            assert!(closure.closure_facts().is_empty());
            assert_eq!(
                check_source_theorem_metadata(source, symbols, scope)?,
                actual
            );
            let composed = typed
                .clone()
                .with_source_formula_composition_from_original(source, symbols, scope)
                .expect("RED: genuine primitive quantifier composition");
            let graph = composed.source_composite_formula().unwrap();
            let declarations = context
                .declarations()
                .iter()
                .filter(|(_, row)| {
                    row.role
                        == mizar_checker::source_context::SourceBindingSiteRole::QuantifierBinder
                })
                .collect::<Vec<_>>();
            assert_eq!(graph.binders().len(), declarations.len());
            assert_eq!(graph.type_sites().len(), declarations.len());
            for (binder_id, binder) in graph.binders().iter() {
                let (_, declaration) = declarations
                    .iter()
                    .find(|(_, row)| row.binding == binder.binding())
                    .unwrap();
                let physical = physical[profile.3[declaration.source_ordinal]];
                let syntax = ast.node_view(physical).unwrap();
                let head = ast.node_view(syntax.children()[0]).unwrap();
                let row = graph.type_sites().get(binder.type_site()).unwrap();
                assert_eq!(row.binder(), binder_id);
                assert_eq!(row.site(), &site(physical));
                assert_eq!(row.source_range(), syntax.range());
                assert_eq!(row.head_site(), &site(head.id()));
                assert_eq!(row.head_range(), head.range());
                let expected = profile.1[profile.3[declaration.source_ordinal]];
                assert_eq!(row.spelling(), expected);
                assert_eq!(row.head_spelling(), expected);
                assert_eq!(
                    format!("{:?}", row.head()),
                    if expected == "object" {
                        "BuiltinObject"
                    } else {
                        "BuiltinSet"
                    }
                );
                let original = scope
                    .bindings()
                    .iter()
                    .find(|original| site(original.node()) == declaration.site)
                    .unwrap();
                assert_eq!(original.kind(), SourceVariableBindingKind::Quantifier);
                if original.declared_type().is_none() {
                    assert!(syntax.range().end < declaration.declaration_range.start);
                }
                assert_eq!(
                    row.context(),
                    if original.declared_type().is_none() {
                        env.contexts()
                            .iter()
                            .find(|(_, row)| row.owner == BindingContextOwner::Module)
                            .unwrap()
                            .0
                    } else {
                        env.contexts()
                            .get(binder.body_context())
                            .unwrap()
                            .parent
                            .unwrap()
                    }
                );
            }
            assert_eq!(composed.nodes(), typed.nodes());
            assert_eq!(composed.contexts(), typed.contexts());
            assert_eq!(composed.types(), typed.types());
            assert_eq!(composed.source_type(), typed.source_type());
            assert_eq!(composed.source_context(), typed.source_context());
            let final_composed = ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
                typed_ast: &composed,
                cluster_facts: &facts,
                overload_collection: &collection,
                template_expansion: &expansion,
                viability: &viability,
                specificity: &specificity,
                overload_selection: &selection,
                expressions: metadata.clone(),
                node_hints: vec![],
                statement_semantics: None,
                statement_proofs: None,
            })
            .map_err(|error| error.to_string())?;
            assert_eq!(final_composed.source_composite_formula(), Some(graph));
            assert_eq!(
                final_composed.source_formula_composition(),
                composed.source_formula_composition()
            );
            assert!(
                final_composed.checked_proofs().is_empty()
                    && final_composed.checked_formulas().is_empty()
                    && final_composed.statement_semantics().is_empty()
            );
            assert!(
                composed
                    .clone()
                    .with_source_formula_composition_from_original(source, symbols, scope)
                    .is_err()
            );
            assert_eq!(
                DeclarationChecker::check_source_theorem_equalities_with_typed_declarations(
                    source, symbols, scope
                )?,
                installed
            );
        }
        let mut atoms = ast
            .node_views()
            .filter(|view| view.kind() == &K::BuiltinPredicateApplication)
            .collect::<Vec<_>>();
        atoms.sort_by_key(|view| view.range().start);
        assert_eq!(atomic.formulas().len(), atoms.len());
        assert_eq!(atomic.edges().len(), 2 * atoms.len());
        assert_eq!(atomic.requests().len(), 2 * atoms.len());
        assert!(
            atomic.predicate_segments().is_empty()
                && atomic.predicate_heads().is_empty()
                && atomic.candidates().is_empty()
                && atomic.type_sites().is_empty()
                && atomic.attributes().is_empty()
        );
        assert_eq!(atomic.primary_term_fingerprint(), primary.debug_text());
        assert!(
            atomic.application_fingerprint().is_none()
                && atomic.structure_fingerprint().is_none()
                && atomic.set_term_fingerprint().is_none()
        );
        for (index, syntax) in atoms.iter().enumerate() {
            let id = SourceAtomicFormulaId::new(index);
            let row = atomic.formulas().get(id).unwrap();
            assert_eq!(row.site(), &site(syntax.id()));
            assert_eq!(row.source_range(), syntax.range());
            assert_eq!(row.source_ordinal(), index);
            assert_eq!(row.context(), lexical(syntax.id()));
            assert_eq!(row.recovery(), SourceAtomicFormulaRecovery::Normal);
            assert_eq!(row.spelling(), spelling(syntax.id()));
            assert_eq!(row.kind(), SourceAtomicFormulaKind::Equality);
            for (ordinal, child) in [syntax.children()[0], syntax.children()[2]]
                .into_iter()
                .enumerate()
            {
                let reference_node = ast.node_view(child).unwrap().children()[0];
                let target = primary
                    .terms()
                    .iter()
                    .find(|(_, row)| row.site() == &site(reference_node))
                    .unwrap()
                    .0;
                let edge_id = SourceAtomicEdgeId::new(2 * index + ordinal);
                let edge = atomic.edges().get(edge_id).unwrap();
                assert_eq!(
                    (edge.formula(), edge.ordinal(), edge.role(), edge.target()),
                    (
                        id,
                        ordinal,
                        if ordinal == 0 {
                            SourceAtomicEdgeRole::BuiltinLeftOperand
                        } else {
                            SourceAtomicEdgeRole::BuiltinRightOperand
                        },
                        SourceAtomicTermTarget::Primary(target)
                    )
                );
                let request = atomic
                    .requests()
                    .get(SourceAtomicRequestId::new(2 * index + ordinal))
                    .unwrap();
                assert_eq!(
                    (
                        request.formula(),
                        request.ordinal(),
                        request.kind(),
                        request.edge()
                    ),
                    (
                        id,
                        ordinal,
                        SourceAtomicRequestKind::OperandExpectedType,
                        Some(edge_id)
                    )
                );
                assert!(
                    request.candidate().is_none()
                        && request.type_site().is_none()
                        && request.attribute().is_none()
                );
            }
        }
        let mut expected_wrappers = Vec::new();
        for (index, syntax) in atoms.iter().enumerate() {
            let mut cursor = syntax.id();
            let mut groups = Vec::new();
            while let Some(parent) = ast
                .node_views()
                .find(|view| view.children().contains(&cursor))
            {
                match parent.kind() {
                    K::FormulaExpression => cursor = parent.id(),
                    K::ParenthesizedFormula => {
                        groups.push(parent);
                        cursor = parent.id();
                    }
                    _ => break,
                }
            }
            groups.reverse();
            for (ordinal, group) in groups.into_iter().enumerate() {
                expected_wrappers.push((
                    SourceAtomicFormulaId::new(index),
                    ordinal,
                    site(group.id()),
                    group.range(),
                    lexical(group.id()),
                    spelling(group.id()),
                ));
            }
        }
        assert_eq!(atomic.wrappers().len(), expected_wrappers.len());
        for ((_, row), (formula, ordinal, site, range, context, spelling)) in
            atomic.wrappers().iter().zip(expected_wrappers)
        {
            assert_eq!(
                (
                    row.formula(),
                    row.ordinal(),
                    row.site(),
                    row.source_range(),
                    row.context(),
                    row.recovery(),
                    row.spelling()
                ),
                (
                    formula,
                    ordinal,
                    &site,
                    range,
                    context,
                    SourceAtomicFormulaRecovery::Normal,
                    spelling.as_str()
                )
            );
            assert_eq!(
                typed.nodes().node(site.node()).unwrap().kind.as_str(),
                "source.formula.atomic.parenthesized"
            );
        }
        assert_eq!(metadata.len(), primary.terms().len() + atoms.len());
        for input in metadata {
            let node = typed.nodes().node(input.typed_site.node()).unwrap();
            assert_eq!(node.links.context, input.local_context);
            assert!(input.cluster_facts.is_empty());
        }
        let resolved = ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast: typed,
            cluster_facts: &facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &selection,
            expressions: metadata.clone(),
            node_hints: vec![],
            statement_semantics: None,
            statement_proofs: None,
        })
        .map_err(|error| error.to_string())?;
        assert_eq!(resolved.source_context(), Some(context));
        assert_eq!(resolved.source_type(), Some(types));
        assert_eq!(resolved.source_term(), Some(primary));
        assert_eq!(resolved.source_atomic_formula(), Some(atomic));
        let parts = TypedAstParts {
            source_id: typed.source_id(),
            module_id: typed.module_id().clone(),
            resolved_root: typed.resolved_root(),
            source_context: typed.source_context().cloned(),
            source_type: typed.source_type().cloned(),
            source_attribute: None,
            nodes: typed.nodes().clone(),
            contexts: typed.contexts().clone(),
            types: typed.types().clone(),
            facts: typed.facts().clone(),
            coercions: typed.coercions().clone(),
            initial_obligations: typed.initial_obligations().clone(),
            diagnostics: typed.diagnostics().clone(),
        };
        let reconstructed = TypedAst::try_new(parts.clone())
            .unwrap()
            .with_source_term(primary.clone())
            .unwrap()
            .with_source_atomic_formula(atomic.clone())
            .unwrap();
        assert_eq!(&reconstructed, typed);
        if case != 0 {
            for (_, link) in context.context_links().iter() {
                let local = typed.contexts().get(link.local_context).unwrap();
                let owner = typed.nodes().node(local.owner.node()).unwrap();
                let other_kind = match owner.kind.as_str() {
                    "QuantifiedFormula(Universal)" => "QuantifiedFormula(Existential)",
                    "QuantifiedFormula(Existential)" => "QuantifiedFormula(Universal)",
                    _ => continue,
                };
                let opening = owner.children[0];
                let opening_row = typed.nodes().node(opening).unwrap();
                let SourceAnchor::Range(opening_range) = opening_row.anchor else {
                    panic!("opening range")
                };
                let keyword = if other_kind.ends_with("(Universal)") {
                    "for"
                } else {
                    "ex"
                };
                for fault in [
                    "owner kind",
                    "keyword",
                    "category",
                    "missing first",
                    "wrong first",
                    "owner range",
                    "opening range",
                    "opening context",
                    "opening recovery",
                    "opening children",
                ] {
                    let mut nodes = parts
                        .nodes
                        .iter()
                        .map(|(_, row)| row.clone())
                        .collect::<Vec<_>>();
                    match fault {
                        "owner kind" => {
                            nodes[local.owner.node().index()].kind = TypedNodeKind::new(other_kind)
                        }
                        "keyword" => {
                            nodes[opening.index()].kind = TypedNodeKind::new(format!(
                                "Token(SurfaceToken {{ kind: ReservedWord, text: {keyword:?} }})"
                            ))
                        }
                        "category" => {
                            nodes[opening.index()].kind = TypedNodeKind::new(
                                opening_row
                                    .kind
                                    .as_str()
                                    .replace("ReservedWord", "Identifier"),
                            )
                        }
                        "missing first" => nodes[local.owner.node().index()].children.clear(),
                        "wrong first" => nodes[local.owner.node().index()].children.swap(0, 1),
                        "owner range" => {
                            nodes[local.owner.node().index()].anchor = typed
                                .nodes()
                                .node(typed.nodes().root().unwrap())
                                .unwrap()
                                .anchor
                                .clone()
                        }
                        "opening range" => {
                            let mut range = opening_range;
                            range.start += 1;
                            nodes[opening.index()].anchor = SourceAnchor::Range(range);
                        }
                        "opening context" => {
                            nodes[opening.index()].links.context = typed
                                .nodes()
                                .node(typed.nodes().root().unwrap())
                                .unwrap()
                                .links
                                .context
                        }
                        "opening recovery" => {
                            nodes[opening.index()].recovery = NodeRecoveryState::Recovered
                        }
                        _ => nodes[opening.index()].children = vec![owner.children[1]],
                    }
                    let mut corrupt = parts.clone();
                    corrupt.nodes = TypedArena::try_new(parts.nodes.root(), nodes).unwrap();
                    assert_eq!(
                        TypedAst::try_new(corrupt).unwrap_err(),
                        TypedAstError::InvalidSourceContext,
                        "case {case}, {fault}"
                    );
                }
            }
        }

        assert!(
            resolved.checked_formulas().is_empty()
                && resolved.checked_proofs().is_empty()
                && resolved.statement_semantics().is_empty()
        );
        assert_eq!(
            SourceTypeProducer::build_source_theorem_equalities(source, symbols, scope)?,
            lower
        );
        let module_context = env
            .contexts()
            .iter()
            .find(|(_, row)| row.owner == BindingContextOwner::Module)
            .unwrap()
            .0;
        assert!(env.contexts().get(module_context).unwrap().parent.is_none());
    }
    assert!(remapped_inference_id && distinct_decl_namespace_id);
    // Recovered source is refused at its genuine resolver or lower owner; it
    // never supplies a lower RED receipt for the admitted quantifier family.
    assert_eq!(recovered.len(), 3);
    for (source, symbols, scope) in recovered {
        if let Ok(scope) = scope {
            assert!(
                SourceTypeProducer::build_source_theorem_equalities(&source, &symbols, &scope)
                    .is_err()
            );
        }
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn task257b7_native_quantifier_composition_preserves_installed_replay() -> Result<(), String> {
    use SourceCompositeFormulaKind as C;
    use SourceFormulaAtomicEdgeRole as A;
    use SourceFormulaEdgeRole as E;
    use mizar_checker::{
        binding_env::BindingContextOwner,
        cluster_trace::ClusterFactTable,
        overload_resolution::*,
        resolved_typed_ast::{ResolvedTypedAst, ResolvedTypedAstInputs},
        source_atomic_formula::SourceAtomicFormulaId,
        source_composite_formula::*,
        source_formula_composition::*,
        source_statement::*,
        source_type::SourceTypeProducer,
        type_checker::CheckedStatementOwner,
        typed_ast::{NodeRecoveryState, TypeEntryActual, TypedAst, TypedAstParts, TypedSiteRef},
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::{NamespacePath, SymbolEnv},
        names::{
            LocalTermBinding, ResolvedVariableScope, SourceVariableBindingKind,
            SourceVariableScopeInput, SourceVariableScopeResolver,
        },
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::{SignatureProjectionExtractor, source_proof_scope_maps},
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceAnchor,
        SourceInput, SourceOriginInput,
    };
    use mizar_syntax::{SurfaceFormulaConnective, SurfaceNodeKind as K, SurfaceQuantifierKind};
    use std::collections::{BTreeMap, BTreeSet};

    // Source-derived IDs and roles; no expected graph comes from producer output.
    let fixture =
        include_str!("../../../../../mizar-checker/tests/testdata/native_quantifier_statement.miz");
    let theorem =
        |header: &str| format!("theorem Shape: {header} proof let x be set; thus x = x; end;");
    let fk = vec![
        C::Universal,
        C::Conjunction,
        C::Existential,
        C::Existential,
        C::Disjunction,
        C::Universal,
    ];
    let fe = vec![
        (0, 0, E::UniversalRestriction, 1),
        (0, 1, E::UniversalBody, 3),
        (1, 1, E::ConjunctionRight, 2),
        (3, 0, E::ExistentialBody, 4),
        (4, 1, E::DisjunctionRight, 5),
    ];
    let fa = vec![
        (1, 0, A::ConjunctionLeft, 0),
        (2, 0, A::ExistentialBody, 1),
        (4, 0, A::DisjunctionLeft, 2),
        (5, 0, A::UniversalRestriction, 3),
        (5, 1, A::UniversalBody, 4),
    ];
    let mut cases = Vec::new();
    for text in [
        fixture.to_owned(),
        fixture
            .replace("NativeQuantifiers", "Renamed")
            .replace(" x", " outer")
            .replace("(x", "(outer")
            .replace(" y", " inner")
            .replace(" z", " deep"),
        format!(
            "\n:: layout\n{}",
            fixture
                .replace("being set", "being\tset")
                .replace("proof", "proof\n:: proof scope\n")
        ),
        fixture.replace("theorem NativeQuantifiers", "lemma NativeQuantifiers"),
    ] {
        cases.push((text, fk.clone(), fe.clone(), fa.clone(), (4, 1)));
    }
    cases.extend([
        (theorem("ex x being set st x = x"), vec![C::Existential], vec![], vec![(0,0,A::ExistentialBody,0)], (0,0)),
        (theorem("for x being set st x = x holds x = x"), vec![C::Universal], vec![], vec![(0,0,A::UniversalRestriction,0),(0,1,A::UniversalBody,1)], (0,0)),
        (theorem("for x being set st (ex y being set st y = x) holds x = x"), vec![C::Universal,C::Existential], vec![(0,0,E::UniversalRestriction,1)], vec![(0,1,A::UniversalBody,1),(1,0,A::ExistentialBody,0)], (1,0)),
        (theorem("for x being set ex y being set st y = x"), vec![C::Universal,C::Existential], vec![(0,0,E::UniversalBody,1)], vec![(1,0,A::ExistentialBody,0)], (0,0)),
        (theorem("for x being set st x = x ex y being set st y = x"), vec![C::Universal,C::Existential], vec![(0,1,E::UniversalBody,1)], vec![(0,0,A::UniversalRestriction,0),(1,0,A::ExistentialBody,1)], (0,0)),
        (theorem("ex x being set st for y being set st y = x holds y = x"), vec![C::Existential,C::Universal], vec![(0,0,E::ExistentialBody,1)], vec![(1,0,A::UniversalRestriction,0),(1,1,A::UniversalBody,1)], (0,0)),
        (theorem("ex x being set st ex y being set st y = x"), vec![C::Existential,C::Existential], vec![(0,0,E::ExistentialBody,1)], vec![(1,0,A::ExistentialBody,0)], (0,0)),
        (theorem("for x being set st x = x for y being set st y = x holds y = x"), vec![C::Universal,C::Universal], vec![(0,1,E::UniversalBody,1)], vec![(0,0,A::UniversalRestriction,0),(1,0,A::UniversalRestriction,1),(1,1,A::UniversalBody,2)], (0,0)),
        ("theorem Unused: ex x, spare being set, idle being set st ((x = x)) proof let x, unused be set, idle be set; thus x = x; end;".into(), vec![C::Existential], vec![], vec![(0,0,A::ExistentialBody,0)], (0,2)),
        (theorem("for x being set holds (ex y being set st y = x) & x = x"), vec![C::Universal,C::Conjunction,C::Existential], vec![(0,0,E::UniversalBody,1),(1,0,E::ConjunctionLeft,2)], vec![(1,1,A::ConjunctionRight,1),(2,0,A::ExistentialBody,0)], (1,0)),
        (theorem("for x being set st not x = x holds ex y being set st ((y = x implies y = y) iff (not y = x or y = x))"), vec![C::Universal,C::Negation,C::Existential,C::Biconditional,C::Implication,C::Disjunction,C::Negation], vec![(0,0,E::UniversalRestriction,1),(0,1,E::UniversalBody,2),(2,0,E::ExistentialBody,3),(3,0,E::BiconditionalLeft,4),(3,1,E::BiconditionalRight,5),(5,0,E::DisjunctionLeft,6)], vec![(1,0,A::NegatedFormula,0),(4,0,A::ImplicationLeft,1),(4,1,A::ImplicationRight,2),(5,1,A::DisjunctionRight,4),(6,0,A::NegatedFormula,3)], (3,0)),
        (theorem("(for x being set holds x = x)"), vec![C::Universal], vec![], vec![(0,0,A::UniversalBody,0)], (1,0)),
        (theorem("(ex x being set st x = x)"), vec![C::Existential], vec![], vec![(0,0,A::ExistentialBody,0)], (1,0)),
        (fixture.replace(": ", ": (((").replace("\nproof", ")))\nproof"), fk.clone(), fe.clone(), fa.clone(), (7,1)),
        (format!("\n:: grouped lemma\n{}", fixture.replace("theorem NativeQuantifiers", "lemma Grouped").replace(": ", ": ((").replace("\nproof", "))\nproof").replace("being set", "being\tset")), fk.clone(), fe.clone(), fa.clone(), (6,1)),
        (theorem("not (for x being set holds x = x)"), vec![C::Negation,C::Universal], vec![(0,0,E::NegatedFormula,1)], vec![(1,0,A::UniversalBody,0)], (1,0)),
        (theorem("(for x being set holds x = x) & (ex y being set st y = y)"), vec![C::Conjunction,C::Universal,C::Existential], vec![(0,0,E::ConjunctionLeft,1),(0,1,E::ConjunctionRight,2)], vec![(1,0,A::UniversalBody,0),(2,0,A::ExistentialBody,1)], (2,0)),
        (theorem("((for x being set holds x = x) or (ex y being set st y = y))"), vec![C::Disjunction,C::Universal,C::Existential], vec![(0,0,E::DisjunctionLeft,1),(0,1,E::DisjunctionRight,2)], vec![(1,0,A::UniversalBody,0),(2,0,A::ExistentialBody,1)], (3,0)),
        (theorem("(((for x being set holds x = x) implies (ex y being set st y = y)))"), vec![C::Implication,C::Universal,C::Existential], vec![(0,0,E::ImplicationLeft,1),(0,1,E::ImplicationRight,2)], vec![(1,0,A::UniversalBody,0),(2,0,A::ExistentialBody,1)], (4,0)),
        (format!("\n:: connective lemma\n{}", theorem("(for x being set holds x = x) iff (ex y being set st y = y)").replace("theorem Shape", "lemma Renamed").replace("being set", "being\tset")), vec![C::Biconditional,C::Universal,C::Existential], vec![(0,0,E::BiconditionalLeft,1),(0,1,E::BiconditionalRight,2)], vec![(1,0,A::UniversalBody,0),(2,0,A::ExistentialBody,1)], (2,0)),
        (fixture.replace(": ", ": not (").replace("\nproof", ")\nproof"), std::iter::once(C::Negation).chain(fk).collect(), std::iter::once((0,0,E::NegatedFormula,1)).chain(fe.into_iter().map(|(parent,ordinal,role,child)| (parent+1,ordinal,role,child+1))).collect(), fa.into_iter().map(|(parent,ordinal,role,child)| (parent+1,ordinal,role,child)).collect(), (5,1)),
    ]);
    let implicit = include_str!(
        "../../../../../mizar-checker/tests/testdata/native_implicit_quantifier_statement.miz"
    );
    cases.extend([
        (implicit.to_owned(), vec![C::Universal], vec![], vec![(0,0,A::UniversalBody,0)], (0,0)),
        (implicit.replace("Implicit", "Renamed").replace(" x", " parcel").replace("set;", "set;\n:: layout\n"), vec![C::Universal], vec![], vec![(0,0,A::UniversalBody,0)], (0,0)),
        (implicit.replace("for x holds", "ex x st"), vec![C::Existential], vec![], vec![(0,0,A::ExistentialBody,0)], (0,0)),
        ("reserve x, y for set; theorem Shared: for x, y holds x = y proof let x be set; thus x = x; end;".into(), vec![C::Universal], vec![], vec![(0,0,A::UniversalBody,0)], (0,0)),
        ("reserve x for set; reserve y for set; theorem Mixed: for x being set, y holds x = y proof let x be set; thus x = x; end;".into(), vec![C::Universal], vec![], vec![(0,0,A::UniversalBody,0)], (0,0)),
        (implicit.replace("for x holds", "for x holds ex x st"), vec![C::Universal,C::Existential], vec![(0,0,E::UniversalBody,1)], vec![(1,0,A::ExistentialBody,0)], (0,0)),
        ("reserve x, y for set; theorem Restricted: for x st (ex y st y = x) holds x = x proof let x be set; thus x = x; end;".into(), vec![C::Universal,C::Existential], vec![(0,0,E::UniversalRestriction,1)], vec![(0,1,A::UniversalBody,1),(1,0,A::ExistentialBody,0)], (1,0)),
        ("reserve x, y for set; lemma Fixed: (for x holds x = x) iff (ex y st y = y) proof let x be set; thus x = x; end;".into(), vec![C::Biconditional,C::Universal,C::Existential], vec![(0,0,E::BiconditionalLeft,1),(0,1,E::BiconditionalRight,2)], vec![(1,0,A::UniversalBody,0),(2,0,A::ExistentialBody,1)], (2,0)),
        ("reserve x, y for set; theorem Unused: for x, y holds x = x proof let x be set; thus x = x; end;".into(), vec![C::Universal], vec![], vec![(0,0,A::UniversalBody,0)], (0,0)),
    ]);
    let interleaved = "reserve x for set; theorem First: for x holds x = x proof let x be set; thus x = x; end; reserve y for set; lemma Second: ex y st y = y proof let y be set; thus y = y; end;";
    cases.extend([
        (
            implicit.replace("for x holds", "for x being set holds"),
            vec![C::Universal],
            vec![],
            vec![(0, 0, A::UniversalBody, 0)],
            (0, 0),
        ),
        (
            format!("{} reserve y for set;", implicit.trim()),
            vec![C::Universal],
            vec![],
            vec![(0, 0, A::UniversalBody, 0)],
            (0, 0),
        ),
        (
            format!("{} {implicit}", theorem("for z being set holds z = z")),
            vec![C::Universal, C::Universal],
            vec![],
            vec![(0, 0, A::UniversalBody, 0), (1, 0, A::UniversalBody, 2)],
            (0, 0),
        ),
    ]);
    for text in [
        interleaved.to_owned(),
        interleaved
            .replace(" x", " parcel")
            .replace(" y", " token")
            .replace("set;", "set;\n:: layout\n"),
    ] {
        cases.push((
            text,
            vec![C::Universal, C::Existential],
            vec![],
            vec![(0, 0, A::UniversalBody, 0), (1, 0, A::ExistentialBody, 2)],
            (0, 0),
        ));
    }
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let mut sources = Vec::new();
    for (case, (text, ..)) in cases.iter().enumerate() {
        let path = root.join(format!("src/quantifier_composition_{case}.miz"));
        fs::write(&path, text).unwrap();
        let frontend = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: snapshot_id(808),
                input: SourceInput {
                    package_id: PackageId::new("native-quantifier-composition"),
                    module_path: ModulePath::new(format!("composition_{case}")),
                    normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Disk { path },
                },
            },
            &ids,
        )
        .unwrap();
        assert!(
            frontend.diagnostics.is_empty(),
            "case {case}: {:?}",
            frontend.diagnostics
        );
        let ast = frontend.ast.as_ref().unwrap();
        let module = ModuleId::new(
            frontend.source.package_id.clone(),
            frontend.source.module_path.clone(),
        );
        let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
        source.validate_against(ast, &module).unwrap();
        let shells = DeclarationShellCollector::new(ast, &module).collect();
        let collected = SignatureProjectionExtractor::new(
            ast,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        assert!(collected.diagnostics().is_empty());
        let symbols = collected.env().clone();
        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(ast, &module, &symbols),
        )
        .unwrap();
        sources.push((frontend, source, symbols, scope));
    }
    let collection = OverloadCollectionOutput::collect([], []);
    let expansion = TemplateExpansionOutput::expand(&collection);
    let viability = CandidateViabilityOutput::filter(&expansion, []);
    let specificity = SpecificityGraphOutput::build(&viability, []);
    let selection = OverloadSelectionOutput::resolve(&specificity, []);
    let facts = ClusterFactTable::new();
    let assemble = |typed: &TypedAst, expressions| {
        ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
            typed_ast: typed,
            cluster_facts: &facts,
            overload_collection: &collection,
            template_expansion: &expansion,
            viability: &viability,
            specificity: &specificity,
            overload_selection: &selection,
            expressions,
            node_hints: vec![],
            statement_semantics: None,
            statement_proofs: None,
        })
    };
    // Keep deep native statement acquisition off the large oracle stack.
    let install_statement = |receiver: TypedAst,
                             original: &SurfaceResolvedArena,
                             symbol_env: &SymbolEnv,
                             variable_scope: &ResolvedVariableScope| {
        std::thread::scope(|worker| {
            worker
                .spawn(move || {
                    receiver.with_source_statement_from_original(
                        original,
                        symbol_env,
                        variable_scope,
                    )
                })
                .join()
                .expect("native statement acquisition panicked")
        })
    };
    // This separate loop is deliberate: all genuine lower positives, including
    // all canonical-part and final-metadata prerequisites must pass before the first unchanged upper entrance can be RED-U.
    let mut lowers = Vec::new();
    for (case, (_, source, symbols, scope)) in sources.iter().enumerate() {
        let lower = SourceTypeProducer::build_source_theorem_equalities(source, symbols, scope)
            .unwrap_or_else(|error| panic!("not RED-U: lower case {case} failed: {error}"));
        let parts = TypedAstParts {
            source_id: lower.2.source_id(),
            module_id: lower.2.module_id().clone(),
            resolved_root: lower.2.resolved_root(),
            source_context: lower.2.source_context().cloned(),
            source_type: lower.2.source_type().cloned(),
            source_attribute: None,
            nodes: lower.2.nodes().clone(),
            contexts: lower.2.contexts().clone(),
            types: lower.2.types().clone(),
            facts: lower.2.facts().clone(),
            coercions: lower.2.coercions().clone(),
            initial_obligations: lower.2.initial_obligations().clone(),
            diagnostics: lower.2.diagnostics().clone(),
        };
        let reconstructed = TypedAst::try_new(parts)
            .unwrap()
            .with_source_term(lower.2.source_term().unwrap().clone())
            .unwrap()
            .with_source_atomic_formula(lower.2.source_atomic_formula().unwrap().clone())
            .unwrap();
        assert_eq!(reconstructed, lower.2);
        assert_eq!(lower.2.source_context().unwrap().binding_env(), &lower.0);
        assert_eq!(
            lower.2.source_type().unwrap().applications().len(),
            scope.bindings().len()
        );
        let resolved_lower = assemble(&lower.2, lower.3.clone()).unwrap();
        assert_eq!(resolved_lower.expr_metadata().len(), lower.3.len());
        assert_eq!(resolved_lower.source_context(), lower.2.source_context());
        assert_eq!(resolved_lower.source_type(), lower.2.source_type());
        assert_eq!(resolved_lower.source_term(), lower.2.source_term());
        assert_eq!(
            resolved_lower.source_atomic_formula(),
            lower.2.source_atomic_formula()
        );
        assert!(
            resolved_lower.checked_formulas().is_empty()
                && resolved_lower.checked_proofs().is_empty()
        );
        lowers.push(lower);
    }
    // Keep the complete B6 baseline on its own default stack.
    std::thread::scope(|scope| {
        scope
            .spawn(task257b6_native_connective_composition_preserves_installed_replay)
            .join()
            .expect("old B6 baseline panicked")
    })?;
    for (case, ((frontend, source, symbols, scope), (env, inference, primitive, metadata))) in
        sources.iter().zip(&lowers).enumerate()
    {
        let ast = frontend.ast.as_ref().unwrap();
        let (_, kinds, edge_specs, atom_specs, wrapper_counts) = &cases[case];
        let site = |id| {
            let resolved = source.resolved_node_for(id).unwrap();
            let matches = primitive
                .nodes()
                .iter()
                .filter(|(_, row)| row.resolved_node == Some(resolved))
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1);
            TypedSiteRef::Node(matches[0].0)
        };
        let spelling = |id| {
            let range = ast.node_view(id).unwrap().range();
            let mut tokens = ast
                .node_views()
                .filter(|view| {
                    view.as_token().is_some()
                        && range.start <= view.range().start
                        && view.range().end <= range.end
                })
                .collect::<Vec<_>>();
            tokens.sort_by_key(|view| view.range().start);
            tokens
                .iter()
                .map(|view| view.as_token().unwrap().text.as_ref())
                .collect::<Vec<_>>()
                .join(" ")
        };
        let maps = source_proof_scope_maps(source, symbols, scope)?;
        let root_scope = maps.0.get(&ast.root().unwrap()).unwrap();
        assert!(root_scope.path().is_empty());
        assert_eq!(
            env.contexts()
                .iter()
                .filter(|(_, row)| row.owner == BindingContextOwner::Module)
                .count(),
            1
        );
        let body_context = |id| {
            let actual_scope = maps.0.get(&id).unwrap();
            env.contexts()
                .iter()
                .find(|(_, row)| {
                    row.lexical_scope.as_ref() == Some(actual_scope)
                        || (row.owner == BindingContextOwner::Module
                            && row.lexical_scope.is_none()
                            && actual_scope == root_scope)
                })
                .unwrap()
                .0
        };
        let lexical = |id| {
            let mut cursor = id;
            loop {
                if maps.0.contains_key(&cursor) {
                    return body_context(cursor);
                }
                cursor = ast
                    .node_views()
                    .find(|view| view.children().contains(&cursor))
                    .unwrap()
                    .id();
            }
        };
        let mut syntax = ast
            .node_views()
            .filter(|view| {
                matches!(
                    view.kind(),
                    K::QuantifiedFormula(_) | K::BinaryFormula(_) | K::PrefixFormula(_)
                )
            })
            .collect::<Vec<_>>();
        syntax.sort_by_key(|view| (view.range().start, std::cmp::Reverse(view.range().end)));
        assert_eq!(syntax.len(), kinds.len());
        let mut expected = SourceCompositeFormulaHandoffInput {
            source_id: source.source_id(),
            module_id: source.module().clone(),
            formulas: vec![],
            wrappers: vec![],
            roots: syntax
                .iter()
                .enumerate()
                .filter(|(_, written)| {
                    !syntax.iter().any(|outer| {
                        outer.id() != written.id()
                            && outer.range().start <= written.range().start
                            && written.range().end <= outer.range().end
                    })
                })
                .enumerate()
                .map(|(ordinal, (index, _))| SourceFormulaRootInput {
                    formula: SourceCompositeFormulaId::new(index),
                    ordinal,
                    ownership: SourceFormulaRootOwnership::UnassignedStatement,
                })
                .collect(),
            binders: vec![],
            type_sites: vec![],
            edges: vec![],
            requests: vec![],
        };
        let primary = primitive.source_term().unwrap();
        let atomic = primitive.source_atomic_formula().unwrap();
        let mut binder_for = BTreeMap::new();
        for (index, (written, kind)) in syntax.iter().zip(kinds).enumerate() {
            let formula = SourceCompositeFormulaId::new(index);
            let (incoming, marker, request_kind) = match kind {
                C::Universal | C::Existential => {
                    assert_eq!(
                        written.kind(),
                        &K::QuantifiedFormula(if *kind == C::Universal {
                            SurfaceQuantifierKind::Universal
                        } else {
                            SurfaceQuantifierKind::Existential
                        })
                    );
                    let body = body_context(written.id());
                    assert_eq!(
                        env.contexts().get(body).unwrap().owner,
                        BindingContextOwner::SourceFormula {
                            source_range: written.range()
                        }
                    );
                    (
                        env.contexts().get(body).unwrap().parent.unwrap(),
                        written
                            .children()
                            .iter()
                            .filter_map(|id| {
                                ast.node_view(*id)
                                    .unwrap()
                                    .as_token()
                                    .filter(|token| {
                                        matches!(token.text.as_ref(), "for" | "ex" | "st" | "holds")
                                    })
                                    .map(|token| token.text.to_string())
                            })
                            .collect::<Vec<_>>()
                            .join(" "),
                        SourceFormulaRequestKind::QuantifierSemantics,
                    )
                }
                C::Negation => {
                    assert!(matches!(written.kind(), K::PrefixFormula(_)));
                    assert_eq!(spelling(written.children()[0]), "not");
                    (
                        lexical(written.id()),
                        "not".into(),
                        SourceFormulaRequestKind::NegationSemantics,
                    )
                }
                _ => {
                    let K::BinaryFormula(operator) = written.kind() else {
                        panic!("expected binary source")
                    };
                    assert!(!operator.repeated);
                    let (connective, marker) = match kind {
                        C::Conjunction => (SurfaceFormulaConnective::And, "&"),
                        C::Disjunction => (SurfaceFormulaConnective::Or, "or"),
                        C::Implication => (SurfaceFormulaConnective::Implies, "implies"),
                        C::Biconditional => (SurfaceFormulaConnective::Iff, "iff"),
                        _ => panic!("outside frozen family"),
                    };
                    assert_eq!(operator.connective, connective);
                    assert_eq!(spelling(written.children()[1]), marker);
                    (
                        lexical(written.id()),
                        marker.into(),
                        SourceFormulaRequestKind::ConnectiveSemantics,
                    )
                }
            };
            expected.formulas.push(SourceCompositeFormulaInput {
                site: site(written.id()),
                source_range: written.range(),
                source_ordinal: index,
                context: incoming,
                recovery: SourceCompositeFormulaRecovery::Normal,
                spelling: marker,
                kind: *kind,
            });
            expected.requests.push(SourceFormulaRequestInput {
                formula,
                ordinal: 0,
                kind: request_kind,
                binder: None,
                type_site: None,
            });
            if matches!(kind, C::Universal | C::Existential) {
                let body = body_context(written.id());
                let mut ordinal = 0;
                for segment in written
                    .children()
                    .iter()
                    .map(|id| ast.node_view(*id).unwrap())
                    .filter(|view| view.kind() == &K::QuantifierVariableSegment)
                {
                    let explicit = segment
                        .children()
                        .iter()
                        .filter_map(|id| ast.node_view(*id))
                        .find(|view| view.kind() == &K::TypeExpression);
                    let descriptor = segment.children()
                        [..segment.children().len() - usize::from(explicit.is_some())]
                        .iter()
                        .map(|id| spelling(*id))
                        .collect::<Vec<_>>()
                        .join(" ");
                    for original in scope
                        .bindings()
                        .iter()
                        .filter(|binding| segment.children().contains(&binding.node()))
                    {
                        assert_eq!(original.kind(), SourceVariableBindingKind::Quantifier);
                        assert_eq!(Some(original.scope()), maps.0.get(&written.id()));
                        let declaration = primitive
                            .source_context()
                            .unwrap()
                            .declarations()
                            .iter()
                            .find(|(_, row)| row.site == site(original.node()))
                            .unwrap()
                            .1;
                        let ty = explicit.unwrap_or_else(|| {
                            let reserve = ast
                                .node_views()
                                .find(|view| {
                                    view.kind() == &K::ReserveSegment
                                        && view.range().end < original.range().start
                                        && view
                                            .children()
                                            .iter()
                                            .any(|id| spelling(*id) == original.spelling())
                                })
                                .unwrap();
                            ast.node_view(*reserve.children().last().unwrap()).unwrap()
                        });
                        let head = ast.node_view(ty.children()[0]).unwrap();
                        assert_eq!(spelling(ty.id()), "set");
                        let binder = SourceQuantifierBinderId::new(expected.binders.len());
                        let type_site = SourceBinderTypeSiteId::new(expected.type_sites.len());
                        binder_for.insert(original.id(), binder);
                        expected.binders.push(SourceQuantifierBinderInput {
                            formula,
                            ordinal,
                            segment_site: site(segment.id()),
                            segment_range: segment.range(),
                            segment_spelling: descriptor.clone(),
                            identifier_site: site(original.node()),
                            identifier_range: original.range(),
                            identifier_spelling: original.spelling().into(),
                            local: LocalTermBinding::new(
                                original.spelling(),
                                original.scope().clone(),
                                original.range(),
                                original.ordinal(),
                            ),
                            binding: declaration.binding,
                            body_context: body,
                            type_site,
                            recovery: SourceCompositeFormulaRecovery::Normal,
                        });
                        expected.type_sites.push(SourceBinderTypeSiteInput {
                            binder,
                            site: site(ty.id()),
                            source_range: ty.range(),
                            spelling: "set".into(),
                            head_site: site(head.id()),
                            head_range: head.range(),
                            head_spelling: "set".into(),
                            context: if explicit.is_some() {
                                incoming
                            } else {
                                lexical(ty.id())
                            },
                            recovery: SourceCompositeFormulaRecovery::Normal,
                            head: SourceBinderTypeHead::BuiltinSet,
                        });
                        expected.requests.push(SourceFormulaRequestInput {
                            formula,
                            ordinal: 1 + ordinal,
                            kind: SourceFormulaRequestKind::BinderType,
                            binder: Some(binder),
                            type_site: Some(type_site),
                        });
                        ordinal += 1;
                    }
                }
            }
            let mut cursor = written.id();
            let mut groups = Vec::new();
            while let Some(parent) = ast
                .node_views()
                .find(|view| view.children().contains(&cursor))
            {
                match parent.kind() {
                    K::FormulaExpression => cursor = parent.id(),
                    K::ParenthesizedFormula => {
                        groups.push(parent);
                        cursor = parent.id();
                    }
                    _ => break,
                }
            }
            groups.reverse();
            let depth = groups.len();
            for (ordinal, group) in groups.into_iter().enumerate() {
                let mut descriptor = expected.formulas[index].spelling.clone();
                for _ in ordinal..depth {
                    descriptor = format!("( {descriptor} )");
                }
                expected.wrappers.push(SourceFormulaWrapperInput {
                    formula,
                    ordinal,
                    site: site(group.id()),
                    source_range: group.range(),
                    context: incoming,
                    recovery: SourceCompositeFormulaRecovery::Normal,
                    spelling: descriptor,
                });
            }
        }
        expected.edges = edge_specs
            .iter()
            .map(|(parent, ordinal, role, child)| SourceFormulaEdgeInput {
                parent: SourceCompositeFormulaId::new(*parent),
                ordinal: *ordinal,
                role: *role,
                child: SourceCompositeFormulaId::new(*child),
            })
            .collect();
        let mut composition = SourceFormulaCompositionHandoffInput {
            source_id: source.source_id(),
            module_id: source.module().clone(),
            atomic_edges: atom_specs
                .iter()
                .map(
                    |(parent, ordinal, role, atom)| SourceFormulaAtomicEdgeInput {
                        formula: SourceCompositeFormulaId::new(*parent),
                        ordinal: *ordinal,
                        role: *role,
                        child: SourceAtomicFormulaId::new(*atom),
                    },
                )
                .collect(),
            bound_uses: vec![],
        };
        let mut use_ordinals = BTreeMap::new();
        for original in scope
            .references()
            .iter()
            .filter(|reference| binder_for.contains_key(&reference.binding()))
        {
            let binder = binder_for[&original.binding()];
            let (term, _) = primary
                .terms()
                .iter()
                .find(|(_, row)| row.site() == &site(original.node()))
                .unwrap();
            let (reference, actual) = primary
                .references()
                .iter()
                .find(|(_, row)| row.term() == term)
                .unwrap();
            assert_eq!(actual.use_ordinal(), original.ordinal());
            let atom = atomic
                .formulas()
                .iter()
                .find(|(_, row)| {
                    row.source_range().start <= original.range().start
                        && original.range().end <= row.source_range().end
                })
                .unwrap()
                .0;
            let body_edge = SourceFormulaAtomicEdgeId::new(
                composition
                    .atomic_edges
                    .iter()
                    .position(|edge| edge.child == atom)
                    .unwrap(),
            );
            let ordinal = use_ordinals.entry(binder).or_insert(0);
            composition.bound_uses.push(SourceQuantifierBoundUseInput {
                binder,
                ordinal: *ordinal,
                body_edge,
                term,
                reference,
            });
            *ordinal += 1;
        }
        assert_eq!(expected.wrappers.len(), wrapper_counts.0);
        assert_eq!(atomic.wrappers().len(), wrapper_counts.1);
        assert_eq!(composition.bound_uses.len(), 2 * atom_specs.len());
        if matches!(
            kinds[0],
            C::Conjunction | C::Disjunction | C::Implication | C::Biconditional
        ) {
            assert_eq!(expected.binders.len(), 2);
            let module_context = expected.formulas[0].context;
            assert_eq!(
                env.contexts().get(module_context).unwrap().owner,
                BindingContextOwner::Module
            );
            assert_ne!(
                expected.binders[0].body_context,
                expected.binders[1].body_context
            );
            for binder in &expected.binders {
                assert_eq!(
                    env.contexts().get(binder.body_context).unwrap().parent,
                    Some(module_context)
                );
            }
            assert_eq!(
                composition
                    .bound_uses
                    .iter()
                    .map(|row| (row.binder.index(), row.ordinal, row.body_edge.index()))
                    .collect::<Vec<_>>(),
                vec![(0, 0, 0), (0, 1, 0), (1, 0, 1), (1, 1, 1)]
            );
            let proof_local = scope
                .bindings()
                .iter()
                .find(|row| row.kind() == SourceVariableBindingKind::Let)
                .unwrap();
            let proof_declaration = primitive
                .source_context()
                .unwrap()
                .declarations()
                .iter()
                .find(|(_, row)| row.site == site(proof_local.node()))
                .unwrap()
                .1;
            assert!(
                expected
                    .binders
                    .iter()
                    .all(|row| row.binding != proof_declaration.binding
                        && row.body_context != proof_declaration.binding_context)
            );
            assert_eq!(
                env.contexts()
                    .get(proof_declaration.binding_context)
                    .unwrap()
                    .parent,
                Some(module_context)
            );
        }
        if case < 4 {
            assert_eq!(expected.binders.len(), 4);
            assert_eq!(expected.requests.len(), 10);
            assert_eq!((env.contexts().len(), env.bindings().len()), (6, 5));
            assert_eq!((atomic.formulas().len(), primary.terms().len()), (6, 12));
            assert_eq!(
                composition
                    .bound_uses
                    .iter()
                    .map(|row| row.binder.index())
                    .collect::<Vec<_>>(),
                vec![0, 0, 1, 0, 2, 2, 3, 2, 3, 2]
            );
            assert_eq!(
                composition
                    .bound_uses
                    .iter()
                    .map(|row| row.ordinal)
                    .collect::<Vec<_>>(),
                vec![0, 1, 0, 2, 0, 1, 0, 2, 1, 3]
            );
            let y = expected.binders[1].body_context;
            let shadow = expected.binders[2].body_context;
            assert_ne!(y, shadow);
            assert_eq!(
                env.contexts().get(y).unwrap().parent,
                Some(expected.binders[0].body_context)
            );
            assert_eq!(
                env.contexts().get(shadow).unwrap().parent,
                Some(expected.binders[0].body_context)
            );
            assert_eq!(
                env.contexts()
                    .get(expected.binders[3].body_context)
                    .unwrap()
                    .parent,
                Some(shadow)
            );
        }
        if case == 6 || case == 13 {
            assert_eq!(
                composition
                    .bound_uses
                    .iter()
                    .map(|row| row.body_edge.index())
                    .collect::<Vec<_>>(),
                vec![1, 1, 0, 0]
            );
            assert_eq!(
                composition
                    .bound_uses
                    .iter()
                    .map(|row| row.binder.index())
                    .collect::<Vec<_>>(),
                vec![1, 0, 0, 0]
            );
            assert_eq!(
                composition
                    .bound_uses
                    .iter()
                    .map(|row| row.ordinal)
                    .collect::<Vec<_>>(),
                vec![0, 0, 1, 2]
            );
        }
        if case == 12 {
            assert_eq!(expected.binders.len(), 3);
            assert_eq!(expected.type_sites.len(), 3);
            assert_eq!(expected.requests.len(), 4);
            assert!(
                composition
                    .bound_uses
                    .iter()
                    .all(|row| row.binder == SourceQuantifierBinderId::new(0))
            );
        }
        let mut with_rows = primitive.clone();
        with_rows.install_source_nodes(source).unwrap();
        let rows = with_rows
            .source_node_entries()
            .unwrap()
            .cloned()
            .collect::<Vec<_>>();
        let mut reachable = BTreeSet::new();
        let mut pending = vec![source.arena().root()];
        while let Some(node) = pending.pop() {
            if reachable.insert(node) {
                pending.extend_from_slice(source.arena().node(node).unwrap().children());
            }
        }
        assert_eq!(
            rows.iter().map(|row| row.site()).collect::<BTreeSet<_>>(),
            reachable
        );
        assert_eq!(rows.len(), reachable.len());
        for row in &rows {
            let original = source.arena().node(row.site()).unwrap();
            let surface = ast
                .node_views()
                .find(|view| source.resolved_node_for(view.id()) == Some(row.site()))
                .unwrap();
            assert_eq!(row.kind(), original.kind());
            assert_eq!(
                &SourceAnchor::Range(row.range()),
                original.origin().anchor()
            );
            assert_eq!(row.spelling(), spelling(surface.id()));
            assert_eq!(row.recovery(), NodeRecoveryState::Normal);
            if let Some(parent) = row.parent() {
                assert_eq!(
                    source.arena().node(parent).unwrap().children()[row.child_ordinal()],
                    row.site()
                );
            } else {
                assert_eq!(row.site(), source.arena().root());
            }
        }
        let baseline = primitive
            .clone()
            .with_source_formula_composition_from_original(source, symbols, scope)
            .expect("RED-U only after every genuine lower positive passes");
        let composite = baseline.source_composite_formula().unwrap();
        let joined = baseline.source_formula_composition().unwrap();
        assert_eq!(composite.source_id(), expected.source_id);
        assert_eq!(composite.module_id(), &expected.module_id);
        assert_eq!(composite.binding_env(), env);
        assert_eq!(joined.source_id(), composition.source_id);
        assert_eq!(joined.module_id(), &composition.module_id);
        assert_eq!(composite.formulas().len(), expected.formulas.len());
        for ((_, row), expected) in composite.formulas().iter().zip(&expected.formulas) {
            assert_eq!(
                (
                    row.site(),
                    row.source_range(),
                    row.source_ordinal(),
                    row.context(),
                    row.recovery(),
                    row.spelling(),
                    row.kind()
                ),
                (
                    &expected.site,
                    expected.source_range,
                    expected.source_ordinal,
                    expected.context,
                    expected.recovery,
                    expected.spelling.as_str(),
                    expected.kind
                )
            );
        }
        assert_eq!(composite.wrappers().len(), expected.wrappers.len());
        for ((_, row), expected) in composite.wrappers().iter().zip(&expected.wrappers) {
            assert_eq!(
                (
                    row.formula(),
                    row.ordinal(),
                    row.site(),
                    row.source_range(),
                    row.context(),
                    row.recovery(),
                    row.spelling()
                ),
                (
                    expected.formula,
                    expected.ordinal,
                    &expected.site,
                    expected.source_range,
                    expected.context,
                    expected.recovery,
                    expected.spelling.as_str()
                )
            );
        }
        assert_eq!(composite.roots().len(), expected.roots.len());
        for ((_, row), expected) in composite.roots().iter().zip(&expected.roots) {
            assert_eq!(
                (row.formula(), row.ordinal(), row.ownership()),
                (expected.formula, expected.ordinal, expected.ownership)
            );
        }
        assert_eq!(composite.binders().len(), expected.binders.len());
        for ((_, row), expected) in composite.binders().iter().zip(&expected.binders) {
            assert_eq!(
                (
                    row.formula(),
                    row.ordinal(),
                    row.segment_site(),
                    row.segment_range(),
                    row.segment_spelling()
                ),
                (
                    expected.formula,
                    expected.ordinal,
                    &expected.segment_site,
                    expected.segment_range,
                    expected.segment_spelling.as_str()
                )
            );
            assert_eq!(
                (
                    row.identifier_site(),
                    row.identifier_range(),
                    row.identifier_spelling(),
                    row.local(),
                    row.binding(),
                    row.body_context(),
                    row.type_site(),
                    row.recovery()
                ),
                (
                    &expected.identifier_site,
                    expected.identifier_range,
                    expected.identifier_spelling.as_str(),
                    &expected.local,
                    expected.binding,
                    expected.body_context,
                    expected.type_site,
                    expected.recovery
                )
            );
        }
        assert_eq!(composite.type_sites().len(), expected.type_sites.len());
        for ((_, row), expected) in composite.type_sites().iter().zip(&expected.type_sites) {
            assert_eq!(
                (
                    row.binder(),
                    row.site(),
                    row.source_range(),
                    row.spelling(),
                    row.head_site(),
                    row.head_range(),
                    row.head_spelling(),
                    row.context(),
                    row.recovery(),
                    row.head()
                ),
                (
                    expected.binder,
                    &expected.site,
                    expected.source_range,
                    expected.spelling.as_str(),
                    &expected.head_site,
                    expected.head_range,
                    expected.head_spelling.as_str(),
                    expected.context,
                    expected.recovery,
                    expected.head
                )
            );
        }
        assert_eq!(composite.edges().len(), expected.edges.len());
        for ((_, row), expected) in composite.edges().iter().zip(&expected.edges) {
            assert_eq!(
                (row.parent(), row.ordinal(), row.role(), row.child()),
                (
                    expected.parent,
                    expected.ordinal,
                    expected.role,
                    expected.child
                )
            );
        }
        assert_eq!(composite.requests().len(), expected.requests.len());
        for ((_, row), expected) in composite.requests().iter().zip(&expected.requests) {
            assert_eq!(
                (
                    row.formula(),
                    row.ordinal(),
                    row.kind(),
                    row.binder(),
                    row.type_site()
                ),
                (
                    expected.formula,
                    expected.ordinal,
                    expected.kind,
                    expected.binder,
                    expected.type_site
                )
            );
        }
        assert_eq!(
            joined
                .atomic_edges()
                .iter()
                .map(|(_, row)| SourceFormulaAtomicEdgeInput::from(row))
                .collect::<Vec<_>>(),
            composition.atomic_edges
        );
        assert_eq!(
            joined
                .bound_uses()
                .iter()
                .map(|(_, row)| SourceQuantifierBoundUseInput::from(row))
                .collect::<Vec<_>>(),
            composition.bound_uses
        );
        assert_eq!(joined.primary_term_fingerprint(), primary.debug_text());
        assert_eq!(joined.atomic_formula_fingerprint(), atomic.debug_text());
        assert_eq!(
            joined.composite_formula_fingerprint(),
            composite.debug_text()
        );
        assert_eq!(baseline.nodes(), primitive.nodes());
        assert_eq!(baseline.contexts(), primitive.contexts());
        assert_eq!(baseline.types(), inference.type_entries());
        assert_eq!(baseline.source_context(), primitive.source_context());
        assert_eq!(baseline.source_type(), primitive.source_type());
        assert_eq!(baseline.source_term(), primitive.source_term());
        assert_eq!(
            baseline.source_atomic_formula(),
            primitive.source_atomic_formula()
        );
        assert_eq!(baseline.facts(), primitive.facts());
        assert_eq!(baseline.coercions(), primitive.coercions());
        assert_eq!(
            baseline.initial_obligations(),
            primitive.initial_obligations()
        );
        assert_eq!(baseline.diagnostics(), primitive.diagnostics());
        assert_eq!(
            baseline
                .source_node_entries()
                .unwrap()
                .cloned()
                .collect::<Vec<_>>(),
            rows
        );
        assert!(baseline.source_statement().is_none());
        let baseline_resolved = assemble(&baseline, metadata.clone()).unwrap();
        assert_eq!(
            baseline_resolved.source_composite_formula(),
            Some(composite)
        );
        assert_eq!(baseline_resolved.source_formula_composition(), Some(joined));

        let theorem = ast
            .node_views()
            .find(|view| matches!(view.kind(), K::TheoremItem | K::LemmaItem))
            .unwrap();
        let implicit_family = ast.node_views().any(|view| view.kind() == &K::ReserveItem);
        if implicit_family {
            let (actual_scope, labels, resolution) =
                mizar_resolve::symbols::source_proof_inputs(source, symbols)?;
            assert_eq!(&actual_scope, scope);
            use mizar_session::{
                SessionIdAllocator, SnapshotInput, SnapshotRegistry, SourceVersion, ToolchainInfo,
                WorkspaceRoot, hash_text,
            };
            let policy = mizar_proof::policy::VerifierPolicy::default();
            let registry = SnapshotRegistry::new();
            let unit = &frontend.source;
            let (snapshot, _lease) = registry
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
                        dependency_artifacts: vec![],
                        lockfile_hash: hash_text(
                            &fs::read_to_string(
                                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock"),
                            )
                            .unwrap(),
                        ),
                        toolchain: ToolchainInfo::new("native-reserve-placement-test"),
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
                "theorems.equality.unsupported_source_types"
            );
            assert_eq!(snapshot, snapshot_before);
            assert!(!root.join("proof-witnesses").exists());
            assert!(mizar_checker::type_checker::SourceVariableSemanticsChecker::check_proof_organization(&baseline, scope, symbols, &labels, &resolution).is_err());
            assert!(mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(source, &baseline, scope, symbols, &labels, &resolution, None).is_err());
        }
        let statement_admitted = !implicit_family
            && kinds[0] == C::Universal
            && theorem.kind() == &K::TheoremItem
            && !expected
                .wrappers
                .iter()
                .any(|row| row.formula == SourceCompositeFormulaId::new(0));
        let installed = if statement_admitted {
            // Native258 retains precisely two propositions. Quantifier guards stay
            // inside the header graph; only actual proof Let declarations add facts.
            let proof = ast
                .node_views()
                .find(|view| view.kind() == &K::ProofBlock)
                .unwrap();
            let conclusion = ast
                .node_views()
                .find(|view| view.kind() == &K::ConclusionStatement)
                .unwrap();
            let label = spelling(theorem.children()[1]);
            let symbol = symbols
                .symbols()
                .iter()
                .find(|entry| entry.primary_spelling() == label)
                .unwrap();
            let checked = CheckedStatementOwner::validate_exact_local_theorem(
                symbols,
                symbol.symbol().clone(),
                source.source_id(),
                source.module(),
            )
            .unwrap();
            assert_eq!(checked.source_range(), theorem.range());
            assert_eq!(checked.origin(), symbol.origin());
            let module_context = env
                .contexts()
                .iter()
                .find(|(_, row)| row.owner == BindingContextOwner::Module)
                .unwrap()
                .0;
            let proof_context = body_context(proof.id());
            assert_eq!(
                env.contexts().get(proof_context).unwrap().parent,
                Some(module_context)
            );
            assert_eq!(expected.formulas[0].context, module_context);
            let conclusion_atom = SourceAtomicFormulaId::new(atom_specs.len());
            assert_eq!(atomic.formulas().len(), atom_specs.len() + 1);
            assert_eq!(
                atomic.formulas().get(conclusion_atom).unwrap().context(),
                proof_context
            );
            let let_count = ast
                .node_views()
                .filter(|view| view.kind() == &K::LetStatement)
                .count();
            let mut statement_expected = SourceStatementHandoffInput {
                source_id: source.source_id(),
                module_id: source.module().clone(),
                owners: vec![SourceTheoremOwnerInput {
                    symbol: symbol.symbol().clone(),
                    contribution: symbol.contribution(),
                    site: site(theorem.id()),
                    source_range: theorem.range(),
                    spelling: label,
                    role: SourceTheoremRole::Theorem,
                    status: SourceTheoremStatus::Unmodified,
                    recovery: SourceStatementRecovery::Normal,
                }],
                statements: vec![],
                contexts: vec![],
                input_facts: vec![],
                candidate_facts: vec![],
            };
            for (index, (syntax, binding_context, formula, source_ordinal, kind)) in [
                (
                    theorem,
                    module_context,
                    SourceStatementFormulaTarget::Composite(SourceCompositeFormulaId::new(0)),
                    0,
                    SourceStatementKind::TheoremProposition,
                ),
                (
                    conclusion,
                    proof_context,
                    SourceStatementFormulaTarget::Atomic(conclusion_atom),
                    1 + let_count,
                    SourceStatementKind::Conclusion,
                ),
            ]
            .into_iter()
            .enumerate()
            {
                let statement = SourceStatementId::new(index);
                let context = SourceStatementContextId::new(index);
                statement_expected.statements.push(SourceStatementInput {
                    owner: SourceTheoremOwnerId::new(0),
                    context,
                    formula,
                    site: site(syntax.id()),
                    source_range: syntax.range(),
                    source_ordinal,
                    spelling: spelling(syntax.id()),
                    kind,
                    recovery: SourceStatementRecovery::Normal,
                });
                statement_expected
                    .contexts
                    .push(SourceStatementContextInput {
                        statement,
                        binding_context,
                        source_range: syntax.range(),
                        visible_bindings: env
                            .contexts()
                            .get(binding_context)
                            .unwrap()
                            .visible_bindings
                            .clone(),
                    });
                statement_expected
                    .candidate_facts
                    .push(SourceStatementCandidateFactInput {
                        statement,
                        context,
                        ordinal: 0,
                        kind: SourceStatementCandidateFactKind::UnverifiedProposition,
                        formula,
                    });
            }
            for (ordinal, local) in scope
                .bindings()
                .iter()
                .filter(|binding| binding.kind() == SourceVariableBindingKind::Let)
                .enumerate()
            {
                let declaration = primitive
                    .source_context()
                    .unwrap()
                    .declarations()
                    .iter()
                    .find(|(_, row)| row.site == site(local.node()))
                    .unwrap()
                    .1;
                assert_eq!(declaration.binding_context, proof_context);
                let uses = scope
                    .references()
                    .iter()
                    .filter(|reference| reference.binding() == local.id())
                    .map(|reference| {
                        primary
                            .references()
                            .iter()
                            .find(|(_, row)| {
                                primary.terms().get(row.term()).unwrap().site()
                                    == &site(reference.node())
                            })
                            .unwrap()
                            .0
                    })
                    .collect::<Vec<_>>();
                assert!(
                    expected
                        .binders
                        .iter()
                        .all(|binder| binder.binding != declaration.binding)
                );
                statement_expected
                    .input_facts
                    .push(SourceStatementInputFactInput {
                        statement: SourceStatementId::new(1),
                        context: SourceStatementContextId::new(1),
                        ordinal,
                        kind: SourceStatementInputFactKind::LetTypeGuard,
                        binding: declaration.binding,
                        uses,
                    });
            }
            assert!(statement_expected.contexts[0].visible_bindings.is_empty());
            assert_eq!(
                statement_expected.contexts[1].visible_bindings,
                statement_expected
                    .input_facts
                    .iter()
                    .map(|row| row.binding)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                statement_expected
                    .input_facts
                    .iter()
                    .map(|row| row.uses.len())
                    .sum::<usize>(),
                2
            );
            let installed = install_statement(baseline.clone(), source, symbols, scope)
                .expect("native258 quantifier header coexistence");
            let statement = installed.source_statement().unwrap();
            assert_eq!(statement.source_id(), statement_expected.source_id);
            assert_eq!(statement.module_id(), &statement_expected.module_id);
            assert_eq!(statement.checked_owner(), &checked);
            assert_eq!(statement.binding_env(), env);
            assert_eq!(statement.binding_fingerprint(), env.debug_text());
            assert_eq!(statement.primary_term_fingerprint(), primary.debug_text());
            assert_eq!(statement.atomic_formula_fingerprint(), atomic.debug_text());
            assert_eq!(
                statement.composite_formula_fingerprint(),
                Some(composite.debug_text().as_str())
            );
            assert_eq!(
                statement.formula_composition_fingerprint(),
                Some(joined.debug_text().as_str())
            );
            assert_eq!(statement.owners().len(), 1);
            for ((_, row), expected) in statement.owners().iter().zip(&statement_expected.owners) {
                assert_eq!(row.symbol(), &expected.symbol);
                assert_eq!(row.contribution(), expected.contribution);
                assert_eq!(
                    (
                        row.site(),
                        row.source_range(),
                        row.spelling(),
                        row.role(),
                        row.status(),
                        row.recovery()
                    ),
                    (
                        &expected.site,
                        expected.source_range,
                        expected.spelling.as_str(),
                        expected.role,
                        expected.status,
                        expected.recovery
                    )
                );
            }
            assert_eq!(statement.statements().len(), 2);
            for ((_, row), expected) in statement
                .statements()
                .iter()
                .zip(&statement_expected.statements)
            {
                assert_eq!(
                    (
                        row.owner(),
                        row.context(),
                        row.formula(),
                        row.site(),
                        row.source_range(),
                        row.source_ordinal(),
                        row.spelling(),
                        row.kind(),
                        row.recovery()
                    ),
                    (
                        expected.owner,
                        expected.context,
                        expected.formula,
                        &expected.site,
                        expected.source_range,
                        expected.source_ordinal,
                        expected.spelling.as_str(),
                        expected.kind,
                        expected.recovery
                    )
                );
            }
            assert_eq!(statement.contexts().len(), 2);
            for ((_, row), expected) in statement
                .contexts()
                .iter()
                .zip(&statement_expected.contexts)
            {
                assert_eq!(
                    (
                        row.statement(),
                        row.binding_context(),
                        row.source_range(),
                        row.visible_bindings()
                    ),
                    (
                        expected.statement,
                        expected.binding_context,
                        expected.source_range,
                        expected.visible_bindings.as_slice()
                    )
                );
            }
            assert_eq!(
                statement.input_facts().len(),
                statement_expected.input_facts.len()
            );
            for ((_, row), expected) in statement
                .input_facts()
                .iter()
                .zip(&statement_expected.input_facts)
            {
                assert_eq!(
                    (
                        row.statement(),
                        row.context(),
                        row.ordinal(),
                        row.kind(),
                        row.binding(),
                        row.uses()
                    ),
                    (
                        expected.statement,
                        expected.context,
                        expected.ordinal,
                        expected.kind,
                        expected.binding,
                        expected.uses.as_slice()
                    )
                );
            }
            assert_eq!(statement.candidate_facts().len(), 2);
            for ((_, row), expected) in statement
                .candidate_facts()
                .iter()
                .zip(&statement_expected.candidate_facts)
            {
                assert_eq!(
                    (
                        row.statement(),
                        row.context(),
                        row.ordinal(),
                        row.kind(),
                        row.formula()
                    ),
                    (
                        expected.statement,
                        expected.context,
                        expected.ordinal,
                        expected.kind,
                        expected.formula
                    )
                );
            }
            assert_eq!(installed.nodes(), baseline.nodes());
            assert_eq!(installed.contexts(), baseline.contexts());
            assert_eq!(installed.types(), baseline.types());
            assert_eq!(installed.source_context(), baseline.source_context());
            assert_eq!(installed.source_type(), baseline.source_type());
            assert_eq!(installed.source_term(), baseline.source_term());
            assert_eq!(
                installed.source_atomic_formula(),
                baseline.source_atomic_formula()
            );
            assert_eq!(installed.source_composite_formula(), Some(composite));
            assert_eq!(installed.source_formula_composition(), Some(joined));
            assert_eq!(
                installed
                    .source_node_entries()
                    .unwrap()
                    .cloned()
                    .collect::<Vec<_>>(),
                rows
            );
            assert_eq!(installed.facts(), baseline.facts());
            assert_eq!(installed.coercions(), baseline.coercions());
            assert_eq!(
                installed.initial_obligations(),
                baseline.initial_obligations()
            );
            assert_eq!(installed.diagnostics(), baseline.diagnostics());
            assert!(
                installed.source_statement_references().is_none()
                    && installed.source_statement_witnesses().is_none()
                    && installed.source_proof_local_binding().is_none()
                    && installed.source_proof_local_let_binding().is_none()
                    && installed.source_proof_local_let_type().is_none()
            );
            installed
        } else {
            assert!(install_statement(baseline.clone(), source, symbols, scope).is_err());
            baseline.clone()
        };
        let fresh = SourceTypeProducer::build_source_theorem_equalities(source, symbols, scope)?
            .2
            .with_source_formula_composition_from_original(source, symbols, scope)
            .unwrap();
        let fresh = if statement_admitted {
            install_statement(fresh, source, symbols, scope).unwrap()
        } else {
            assert!(install_statement(fresh.clone(), source, symbols, scope).is_err());
            fresh
        };
        assert_eq!(fresh, installed);
        let resolved = assemble(&installed, metadata.clone()).unwrap();
        assert_eq!(assemble(&fresh, metadata.clone()).unwrap(), resolved);
        assert_eq!(resolved.source_statement(), installed.source_statement());
        assert_eq!(resolved.source_context(), installed.source_context());
        assert_eq!(resolved.source_type(), installed.source_type());
        assert_eq!(resolved.source_term(), installed.source_term());
        assert_eq!(
            resolved.source_atomic_formula(),
            installed.source_atomic_formula()
        );
        assert_eq!(resolved.source_composite_formula(), Some(composite));
        assert_eq!(resolved.source_formula_composition(), Some(joined));
        assert_eq!(resolved.nodes(), baseline_resolved.nodes());
        assert_eq!(resolved.expr_metadata(), baseline_resolved.expr_metadata());
        assert_eq!(resolved.expr_metadata().len(), metadata.len());
        for input in metadata {
            let row = resolved.expr_metadata().get_by_expr(&input.expr).unwrap();
            assert_eq!(row.typed_site, input.typed_site);
            assert_eq!(row.local_context, input.local_context);
            assert_eq!(row.cluster_facts, input.cluster_facts);
            assert_eq!(
                resolved.expr_metadata().get_by_site(&input.typed_site),
                Some(row)
            );
            assert_eq!(
                installed
                    .nodes()
                    .node(input.typed_site.node())
                    .unwrap()
                    .anchor,
                SourceAnchor::Range(row.source_range)
            );
            let expected_type = installed
                .types()
                .iter()
                .find(|(_, entry)| entry.owner == input.typed_site)
                .map(|(_, entry)| {
                    let TypeEntryActual::Known(actual) = entry.actual else {
                        panic!("unknown final type")
                    };
                    actual
                });
            assert_eq!(row.final_type, expected_type);
            assert!(
                row.visible_facts.is_empty()
                    && row.overload.is_none()
                    && row.inserted_views.is_empty()
                    && row.diagnostics.is_empty()
            );
        }
        assert!(
            resolved.checked_formulas().is_empty()
                && resolved.statement_semantics().is_empty()
                && resolved.checked_proofs().is_empty()
                && resolved.checked_proof_nodes().is_empty()
                && resolved.checked_terminal_goals().is_empty()
        );
        assert!(
            resolved.cluster_facts().is_empty()
                && resolved.inserted_coercions().is_empty()
                && resolved.diagnostics().is_empty()
        );
        assert!(
            resolved.source_statement_references().is_none()
                && resolved.source_statement_witnesses().is_none()
        );

        // Nonempty final success above precedes every refusal and mutation.
        let pristine = (
            source.clone(),
            symbols.clone(),
            scope.clone(),
            primitive.clone(),
            baseline.clone(),
            installed.clone(),
        );
        assert!(
            installed
                .clone()
                .with_source_formula_composition_from_original(source, symbols, scope)
                .is_err()
        );
        assert!(install_statement(installed.clone(), source, symbols, scope).is_err());
        if let Some(statement) = installed.source_statement() {
            assert!(
                baseline
                    .clone()
                    .with_source_statement(statement.clone())
                    .is_err()
            );
            assert!(
                primitive
                    .clone()
                    .with_source_formula_composition_statement(
                        composite.clone(),
                        joined.clone(),
                        statement.clone()
                    )
                    .is_err()
            );
        }
        assert!(install_statement(primitive.clone(), source, symbols, scope).is_err());
        assert!(
            primitive
                .clone()
                .with_source_composite_formula(composite.clone())
                .is_err()
        );
        assert!(
            primitive
                .clone()
                .with_source_formula_composition(composite.clone(), joined.clone())
                .is_err()
        );
        assert!(
            with_rows
                .with_source_formula_composition_from_original(source, symbols, scope)
                .is_err()
        );
        let foreign = &sources[(case + 1) % sources.len()];
        assert!(
            primitive
                .clone()
                .with_source_formula_composition_from_original(&foreign.1, &foreign.2, &foreign.3)
                .is_err()
        );
        assert!(
            primitive
                .clone()
                .with_source_formula_composition_from_original(source, &foreign.2, scope)
                .is_err()
        );
        assert!(
            primitive
                .clone()
                .with_source_formula_composition_from_original(source, symbols, &foreign.3)
                .is_err()
        );
        assert!(install_statement(baseline.clone(), &foreign.1, &foreign.2, &foreign.3).is_err());
        assert!(install_statement(baseline.clone(), source, &foreign.2, scope).is_err());
        assert!(install_statement(baseline.clone(), source, symbols, &foreign.3).is_err());
        let detached = TypedAst::try_new(TypedAstParts {
            source_id: source.source_id(),
            module_id: source.module().clone(),
            resolved_root: primitive.resolved_root(),
            source_context: primitive.source_context().cloned(),
            source_type: primitive.source_type().cloned(),
            source_attribute: None,
            nodes: primitive.nodes().clone(),
            contexts: primitive.contexts().clone(),
            types: primitive.types().clone(),
            facts: primitive.facts().clone(),
            coercions: primitive.coercions().clone(),
            initial_obligations: primitive.initial_obligations().clone(),
            diagnostics: primitive.diagnostics().clone(),
        })
        .unwrap();
        assert!(
            detached
                .with_source_formula_composition_from_original(source, symbols, scope)
                .is_err()
        );
        let mut enriched_nodes = primitive
            .nodes()
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        enriched_nodes.push(mizar_checker::typed_ast::TypedNode::new(
            "additional.source.node",
            SourceAnchor::Range(theorem.range()),
        ));
        let enriched_nodes =
            mizar_checker::typed_ast::TypedArena::try_new(primitive.nodes().root(), enriched_nodes)
                .unwrap();
        let enriched = TypedAst::try_new(TypedAstParts {
            source_id: source.source_id(),
            module_id: source.module().clone(),
            resolved_root: primitive.resolved_root(),
            source_context: primitive.source_context().cloned(),
            source_type: primitive.source_type().cloned(),
            source_attribute: None,
            nodes: enriched_nodes,
            contexts: primitive.contexts().clone(),
            types: primitive.types().clone(),
            facts: primitive.facts().clone(),
            coercions: primitive.coercions().clone(),
            initial_obligations: primitive.initial_obligations().clone(),
            diagnostics: primitive.diagnostics().clone(),
        })
        .unwrap()
        .with_source_term(primary.clone())
        .unwrap()
        .with_source_atomic_formula(atomic.clone())
        .unwrap();
        assert_eq!(enriched.nodes().len(), primitive.nodes().len() + 1);
        assert!(
            enriched
                .with_source_formula_composition_from_original(source, symbols, scope)
                .is_err()
        );
        let mut coherent_nodes = primitive
            .nodes()
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        let owner_id = expected.formulas[0].site.node();
        let owner = coherent_nodes[owner_id.index()].clone();
        let opening = owner.children[0];
        let root_is_universal = kinds[0] == C::Universal;
        coherent_nodes[owner_id.index()].kind =
            mizar_checker::typed_ast::TypedNodeKind::new(if root_is_universal {
                "QuantifiedFormula(Existential)"
            } else {
                "QuantifiedFormula(Universal)"
            });
        coherent_nodes[opening.index()].kind =
            mizar_checker::typed_ast::TypedNodeKind::new(format!(
                "Token(SurfaceToken {{ kind: ReservedWord, text: {:?} }})",
                if root_is_universal { "ex" } else { "for" }
            ));
        if let SourceAnchor::Range(mut range) = coherent_nodes[opening.index()].anchor {
            range.end = range.start + if root_is_universal { 2 } else { 3 };
            coherent_nodes[opening.index()].anchor = SourceAnchor::Range(range);
        }
        let coherent = TypedAst::try_new(TypedAstParts {
            source_id: primitive.source_id(),
            module_id: primitive.module_id().clone(),
            resolved_root: primitive.resolved_root(),
            source_context: primitive.source_context().cloned(),
            source_type: primitive.source_type().cloned(),
            source_attribute: None,
            nodes: mizar_checker::typed_ast::TypedArena::try_new(
                primitive.nodes().root(),
                coherent_nodes,
            )
            .unwrap(),
            contexts: primitive.contexts().clone(),
            types: primitive.types().clone(),
            facts: primitive.facts().clone(),
            coercions: primitive.coercions().clone(),
            initial_obligations: primitive.initial_obligations().clone(),
            diagnostics: primitive.diagnostics().clone(),
        })
        .unwrap()
        .with_source_term(primary.clone())
        .unwrap()
        .with_source_atomic_formula(atomic.clone())
        .unwrap();
        assert!(
            coherent
                .with_source_formula_composition_from_original(source, symbols, scope)
                .is_err()
        );
        let mut reversed = metadata.clone();
        reversed.reverse();
        assert_eq!(assemble(&installed, reversed).unwrap(), resolved);
        let mut duplicate = metadata.clone();
        duplicate.push(duplicate[0].clone());
        assert!(assemble(&installed, duplicate).is_err());
        let mut wrong_site = metadata.clone();
        wrong_site[0].typed_site =
            TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(usize::MAX));
        assert!(assemble(&installed, wrong_site).is_err());
        let mut wrong_context = metadata.clone();
        wrong_context[0].local_context = Some(mizar_checker::typed_ast::LocalTypeContextId::new(
            usize::MAX,
        ));
        assert!(assemble(&installed, wrong_context).is_err());
        // Raw B2/B3 constructors must continue refusing native transactions.
        assert!(
            SourceCompositeFormulaProducer::build(expected.clone(), env, primitive.nodes())
                .is_err()
        );
        assert!(
            SourceFormulaCompositionProducer::build(
                composition.clone(),
                primary,
                atomic,
                composite,
                primitive.nodes()
            )
            .is_err()
        );
        assert_eq!(assemble(&installed, metadata.clone()).unwrap(), resolved);
        assert_eq!(
            (
                source.clone(),
                symbols.clone(),
                scope.clone(),
                primitive.clone(),
                baseline.clone(),
                installed.clone()
            ),
            pristine
        );
        assert!(
            primitive.source_node_entries().is_none()
                && primitive.source_composite_formula().is_none()
                && primitive.source_formula_composition().is_none()
        );
        assert!(baseline.source_statement().is_none());
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}
