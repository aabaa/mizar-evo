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
            if case >= 6 {
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
                    if index == 0 && matches!(case, 3 | 4) {
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
