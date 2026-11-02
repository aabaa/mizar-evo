use super::*;

#[test]
fn native_theorem_registration_closure_storage_replay_preserves_genuine_outputs() {
    use mizar_checker::{
        cluster_trace::{ClusterClosureStatus, ClusterTraceBuilder},
        registration_resolution::RegistrationDatabase,
        type_checker::{DeclarationChecker, check_source_theorem_registration_closure as close},
    };
    use mizar_driver::{
        native_theorem_registration_closure_from_storage_bytes as decode,
        native_theorem_registration_closure_storage_bytes as encode,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        source::{FrontendSourceLoader, diagnostic_file_path, source_unit_from_loaded},
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput as Input, SourceVariableScopeResolver as Resolver},
        symbols::{
            source_theorem_module_candidates as candidates,
            source_theorem_module_original_inputs as original_inputs,
        },
    };
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let check = |ast: &<MizarParserSeam as ParserSeam>::Ast| {
        let (arena, env) = original_inputs(ast, &module).unwrap().unwrap();
        assert!(candidates(ast, &arena, &env).unwrap().is_some());
        let scope = Resolver::resolve_proof_occurrences(Input::new(ast, &module, &env)).unwrap();
        let typed = DeclarationChecker::check_source_theorem_equalities_with_typed_declarations(
            &arena, &env, &scope,
        )
        .unwrap()
        .2;
        let database = RegistrationDatabase::from_symbol_env(&env, []);
        let oracle =
            ClusterTraceBuilder::default().close(&database, ast.source_id, module.clone(), [], []);
        let closure = close(&arena, &env, &scope, &typed).unwrap();
        assert_eq!(closure, oracle);
        assert_eq!(closure.status(), ClusterClosureStatus::Complete);
        assert_eq!(closure.trace().source_id(), ast.source_id);
        assert_eq!(closure.trace().module_id(), &module);
        assert_eq!(
            closure
                .trace()
                .traversal_profile()
                .cache_key_material()
                .as_str(),
            "order=cluster-trace-order-v1;max_depth=128;max_generated=4096"
        );
        assert!(closure.closure_facts().is_empty());
        assert!(closure.diagnostics().is_empty());
        closure
    };
    for text in [
        include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        ),
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
        include_str!(
            "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
        ),
        "theorem Renamed: for Y being set holds Y = Y\r\nproof\r\n  let Y be set;\r\n  thus Y = Y;\r\nend;\r\n",
    ] {
        for open in [false, true] {
            for threshold in [usize::MAX, 0] {
                with_native_summary_parent(
                    text,
                    open,
                    threshold,
                    |fixture, inputs, _, owner, outputs, (session, ids, snapshots)| {
                        let parent = outputs
                            .iter()
                            .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                            .unwrap();
                        assert_eq!(
                            parent.placement() == &StoragePlacement::Resident,
                            threshold != 0
                        );
                        let handle = owner
                            .storage()
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                parent,
                                parent.output_kind(),
                            )
                            .unwrap();
                        let retained = owner.storage().get(&handle).unwrap();
                        let ast = retained.ast.as_ref().unwrap();
                        let closure = check(ast);
                        let syntax = ast.canonical_bytes().unwrap();
                        let profile = closure
                            .trace()
                            .traversal_profile()
                            .cache_key_material()
                            .as_str();
                        let mut bytes =
                            b"mizar-driver/native-theorem-registration-closure-storage/v1\0"
                                .to_vec();
                        for field in [
                            b"alpha".as_slice(),
                            b"main".as_slice(),
                            syntax.as_slice(),
                            profile.as_bytes(),
                        ] {
                            bytes.extend_from_slice(&(field.len() as u64).to_le_bytes());
                            bytes.extend_from_slice(field);
                        }
                        assert_eq!(encode(ast, &module, &closure).unwrap(), Some(bytes.clone()));
                        let (_, _, descriptor) =
                            &inputs.captured.unwrap().native_sources().unwrap()[0];
                        let (mut draft, _) = fixture.submit_request(vec![], vec![]);
                        draft.source_inputs.versions.clear();
                        let relocated = draft
                            .capture_sources(
                                ids,
                                snapshots,
                                vec![(
                                    DiskSourceLoader::new(fixture.root.join("alpha")),
                                    descriptor.clone(),
                                )],
                            )
                            .unwrap();
                        let second_inputs = SourceLoadInputs {
                            snapshot: relocated.captured().snapshot(),
                            captured: Some(relocated.captured()),
                            ..inputs
                        };
                        let (_, loaded, descriptor) =
                            &relocated.captured().native_sources().unwrap()[0];
                        let second = Frontend::new(
                            FrontendSourceLoader::new(DiskSourceLoader::new(
                                fixture.root.join("alpha"),
                            )),
                            second_inputs.dependency_lexical_provider(&[]),
                            MizarParserSeam,
                        )
                        .run_loaded(source_unit_from_loaded(
                            loaded.clone(),
                            diagnostic_file_path(descriptor),
                        ))
                        .unwrap();
                        assert!(second.diagnostics.is_empty());
                        let second_ast = second.ast.as_ref().unwrap();
                        assert_ne!(second_ast.source_id, ast.source_id);
                        let second_closure = check(second_ast);
                        assert_ne!(second_closure, closure);
                        assert_eq!(
                            encode(second_ast, &module, &second_closure).unwrap(),
                            Some(bytes.clone())
                        );
                        assert_eq!(
                            decode(&bytes, second_ast.source_id, &module).unwrap(),
                            Some(second_closure)
                        );
                        let before = (
                            ast.clone(),
                            module.clone(),
                            closure.clone(),
                            session.captured().clone(),
                        );
                        for stage in 0..3 {
                            if stage == 1 {
                                fixture.write(if open {
                                    &[0xff]
                                } else {
                                    b"open theorem Changed: 0 = 1;\n"
                                });
                            } else if stage == 2 {
                                fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                            }
                            for _ in 0..2 {
                                assert_eq!(
                                    encode(ast, &module, &closure).unwrap(),
                                    Some(bytes.clone())
                                );
                                assert_eq!(
                                    decode(&bytes, ast.source_id, &module).unwrap(),
                                    Some(closure.clone())
                                );
                            }
                            assert_eq!(*owner.storage().get(&handle).unwrap(), *retained);
                        }
                        assert_eq!(
                            (
                                ast.clone(),
                                module.clone(),
                                closure,
                                session.captured().clone()
                            ),
                            before
                        );
                    },
                );
            }
        }
    }
}

#[test]
fn native_theorem_registration_closure_storage_replay_preserves_refusals() {
    use mizar_checker::{
        cluster_trace::{ClusterTraceBuilder, ClusterTraversalConfig},
        registration_resolution::RegistrationDatabase,
        type_checker::{DeclarationChecker, check_source_theorem_registration_closure as close},
    };
    use mizar_driver::{
        native_theorem_registration_closure_from_storage_bytes as decode,
        native_theorem_registration_closure_storage_bytes as encode,
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput as Input, SourceVariableScopeResolver as Resolver},
        symbols::{
            source_theorem_module_candidates as candidates,
            source_theorem_module_original_inputs as original_inputs,
        },
    };
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let lower = |ast: &<MizarParserSeam as ParserSeam>::Ast,
                 module: &ResolverModuleId|
     -> Result<Option<_>, String> {
        let Some((arena, env)) = original_inputs(ast, module)? else {
            return Ok(None);
        };
        if candidates(ast, &arena, &env)?.is_none() {
            return Ok(None);
        }
        let scope = Resolver::resolve_proof_occurrences(Input::new(ast, module, &env))
            .map_err(|error| format!("{error:?}"))?;
        let typed = DeclarationChecker::check_source_theorem_equalities_with_typed_declarations(
            &arena, &env, &scope,
        )?
        .2;
        close(&arena, &env, &scope, &typed).map(Some)
    };
    let domain = b"mizar-driver/native-theorem-registration-closure-storage/v1\0";
    let frame = |syntax: &[u8], profile: &[u8]| {
        let mut bytes = domain.to_vec();
        for field in [b"alpha".as_slice(), b"main".as_slice(), syntax, profile] {
            bytes.extend_from_slice(&(field.len() as u64).to_le_bytes());
            bytes.extend_from_slice(field);
        }
        bytes
    };
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let valid = Fixture::new(
        b"theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
    );
    let (_, _, adapter, parent, _) =
        native_disk_frontend_record_submission(&valid, &ids, &snapshots, 0);
    let output = adapter.publisher().storage().get(&parent).unwrap();
    let ast = output.ast.as_ref().unwrap();
    let closure = lower(ast, &module).unwrap().unwrap();
    let syntax = ast.canonical_bytes().unwrap();
    let key = closure
        .trace()
        .traversal_profile()
        .cache_key_material()
        .as_str();
    let bytes = frame(&syntax, key.as_bytes());
    assert_eq!(encode(ast, &module, &closure), Ok(Some(bytes.clone())));
    let mut corruptions = vec![
        Vec::new(),
        bytes[..domain.len() + 3].to_vec(),
        bytes[..bytes.len() - 1].to_vec(),
        frame(&[0xff], key.as_bytes()),
        frame(&syntax, b"wrong-profile"),
    ];
    let mut bad_domain = bytes.clone();
    bad_domain[0] = b'x';
    corruptions.push(bad_domain);
    let mut bad_version = bytes.clone();
    bad_version[domain.len() - 2] = b'2';
    corruptions.push(bad_version);
    let mut bad_length = bytes.clone();
    bad_length[domain.len()..domain.len() + 8].copy_from_slice(&u64::MAX.to_le_bytes());
    corruptions.push(bad_length);
    let mut trailing = bytes.clone();
    trailing.push(0);
    corruptions.push(trailing);
    let mut oversized = bytes.clone();
    oversized.resize(32 * 1024 * 1024 + 1, 0);
    corruptions.push(oversized);
    for malformed in corruptions {
        assert_eq!(decode(&malformed, ast.source_id, &module), Ok(None));
    }
    for wrong_module in [
        ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("main")),
        ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("other")),
    ] {
        assert_eq!(decode(&bytes, ast.source_id, &wrong_module), Ok(None));
        assert_eq!(encode(ast, &wrong_module, &closure), Ok(None));
    }
    let (arena, env) = original_inputs(ast, &module).unwrap().unwrap();
    let database = RegistrationDatabase::from_symbol_env(&env, []);
    let mut config = ClusterTraversalConfig::default();
    config.max_cluster_depth += 1;
    let nondefault = ClusterTraceBuilder::new(config).close(
        &database,
        arena.source_id(),
        module.clone(),
        [],
        [],
    );
    assert_ne!(nondefault, closure);
    assert_eq!(encode(ast, &module, &nondefault), Ok(None));
    assert_eq!(
        decode(
            &frame(
                &syntax,
                nondefault
                    .trace()
                    .traversal_profile()
                    .cache_key_material()
                    .as_str()
                    .as_bytes()
            ),
            ast.source_id,
            &module
        ),
        Ok(None)
    );
    let changed = Fixture::new(
        b"theorem Different: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
    );
    let (_, _, other_adapter, other_parent, _) =
        native_disk_frontend_record_submission(&changed, &ids, &snapshots, 0);
    let other_output = other_adapter
        .publisher()
        .storage()
        .get(&other_parent)
        .unwrap();
    let other_ast = other_output.ast.as_ref().unwrap();
    assert_ne!(other_ast.source_id, ast.source_id);
    let foreign_closure = lower(other_ast, &module).unwrap().unwrap();
    assert_eq!(encode(ast, &module, &foreign_closure), Ok(None));
    let rebound = <MizarParserSeam as ParserSeam>::Ast::from_canonical_bytes(
        &other_ast.canonical_bytes().unwrap(),
        ast.source_id,
    )
    .unwrap();
    assert_ne!(rebound, *ast);
    assert_eq!(lower(&rebound, &module).unwrap(), Some(closure.clone()));
    assert!(encode(&rebound, &module, &closure).unwrap().is_some());
    let foreign_module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("other"));
    let foreign_value = lower(ast, &foreign_module).unwrap().unwrap();
    assert_eq!(encode(ast, &module, &foreign_value), Ok(None));
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let registration = include_str!(
        "../../../../tests/miz/pass/clusters/pass_advanced_semantics_existential_registration_001.miz"
    );
    for (text, case) in [
        (format!("{private}\nreserve Y for set;\n"), "reserve"),
        (private.replace("by PrivT1", "by Missing"), "citation"),
        (private.replace("thus X = X;", "thus Y = Y;"), "unbound"),
        (private.replace("set", "object"), "unsupported-type"),
        (registration.to_owned(), "registration"),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let (_, _, adapter, parent, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let retained = adapter.publisher().storage().get(&parent).unwrap();
        let ast = retained.ast.as_ref().unwrap();
        if case == "registration" {
            assert!(
                ast.node_views()
                    .any(|node| node.as_registration_block_item().is_some())
            );
        }
        let expected = lower(ast, &module);
        assert!(matches!(
            (&expected, case),
            (Ok(None), "reserve" | "citation" | "registration")
                | (Err(_), "unsupported-type" | "unbound")
        ));
        let syntax = ast.canonical_bytes().unwrap();
        let before = (ast.clone(), closure.clone());
        for profile in [key.as_bytes(), b"wrong-profile"] {
            assert_eq!(
                decode(&frame(&syntax, profile), ast.source_id, &module),
                expected,
                "{case}"
            );
        }
        assert_eq!(
            encode(ast, &module, &closure),
            expected.map(|_| None::<Vec<u8>>),
            "{case}"
        );
        assert_eq!((ast.clone(), closure.clone()), before);
    }
}

#[test]
fn native_theorem_registration_closure_type_caller_preserves_genuine_outputs() {
    use mizar_checker::{
        cluster_trace::ClusterClosureStatus, registration_resolution::RegistrationDatabase,
        type_checker::check_source_theorem_registration_closure as close,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        source::{FrontendSourceLoader, diagnostic_file_path, source_unit_from_loaded},
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput as Input, SourceVariableScopeResolver as Resolver},
        symbols::{
            source_theorem_module_candidates as candidates,
            source_theorem_module_original_inputs as original_inputs,
        },
    };
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    for text in [
        include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        ),
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
    ] {
        for open in [false, true] {
            for threshold in [usize::MAX, 0] {
                with_native_summary_parent(
                    text,
                    open,
                    threshold,
                    |fixture, inputs, old, owner, _, (session, _, _)| {
                        for (phase, kind) in [
                            ("SignatureCollection", "SourceTheoremModuleSummary"),
                            ("TypeChecking", "SourceTheoremTypedAst"),
                        ] {
                            owner.allow_work_unit(AllowedWorkUnit::new(
                                IrPipelinePhase::new(phase),
                                OutputKind::new(kind),
                                IrWorkUnit::new("\"alpha\":\"main\""),
                            ));
                        }
                        let signature = inputs
                            .publish_native_theorem_module_summary(old, owner)
                            .unwrap()
                            .unwrap();
                        let type_input = PhaseInput::new(
                            old.work_unit.clone(),
                            PhaseDispatchInputBundle::new(
                                old.snapshot,
                                old.identities().input_hash(),
                                vec![],
                                vec![
                                    SealedParentOutputHandle::from_published_output(
                                        owner,
                                        old.snapshot,
                                        signature.erase(),
                                    )
                                    .unwrap(),
                                ],
                            )
                            .unwrap(),
                        );
                        let typed = inputs
                            .publish_native_theorem_typed_ast(&type_input, owner)
                            .unwrap()
                            .unwrap();
                        let input = PhaseInput::new(
                            old.work_unit.clone(),
                            PhaseDispatchInputBundle::new(
                                old.snapshot,
                                old.identities().input_hash(),
                                vec![],
                                vec![
                                    SealedParentOutputHandle::from_published_output(
                                        owner,
                                        old.snapshot,
                                        typed.erase(),
                                    )
                                    .unwrap(),
                                ],
                            )
                            .unwrap(),
                        );
                        assert_eq!(
                            typed.placement() == &StoragePlacement::Resident,
                            threshold != 0
                        );
                        let retained = owner.storage().get(&typed).unwrap();
                        let (_, loaded, descriptor) =
                            &inputs.captured.unwrap().native_sources().unwrap()[0];
                        let output = Frontend::new(
                            FrontendSourceLoader::new(DiskSourceLoader::new(
                                fixture.root.join("alpha"),
                            )),
                            inputs.dependency_lexical_provider(&[]),
                            MizarParserSeam,
                        )
                        .run_loaded(source_unit_from_loaded(
                            loaded.clone(),
                            diagnostic_file_path(descriptor),
                        ))
                        .unwrap();
                        assert!(output.diagnostics.is_empty());
                        assert!(output.cache_keys.ast.is_some());
                        let ast = output.ast.as_ref().unwrap();
                        let (arena, env) = original_inputs(ast, &module).unwrap().unwrap();
                        assert!(candidates(ast, &arena, &env).unwrap().is_some());
                        let scope =
                            Resolver::resolve_proof_occurrences(Input::new(ast, &module, &env))
                                .unwrap();
                        let expected = close(&arena, &env, &scope, &retained).unwrap();
                        let database = RegistrationDatabase::from_symbol_env(&env, []);
                        assert!(database.pending().is_empty());
                        assert!(database.activated().is_empty());
                        assert!(database.rejected().is_empty());
                        assert!(database.diagnostics().is_empty());
                        assert!(database.initial_obligations().is_empty());
                        assert!(retained.facts().is_empty());
                        assert!(retained.coercions().is_empty());
                        assert!(retained.initial_obligations().is_empty());
                        assert_eq!(expected.status(), ClusterClosureStatus::Complete);
                        assert_eq!(expected.trace().source_id(), loaded.source_id);
                        assert_eq!(expected.trace().module_id(), &module);
                        assert_eq!(
                            expected
                                .trace()
                                .traversal_profile()
                                .cache_key_material()
                                .as_str(),
                            "order=cluster-trace-order-v1;max_depth=128;max_generated=4096"
                        );
                        assert!(expected.closure_facts().is_empty());
                        assert!(expected.diagnostics().is_empty());
                        let before = (
                            input.clone(),
                            type_input.clone(),
                            typed.clone(),
                            (*retained).clone(),
                            session.captured().clone(),
                            expected.clone(),
                        );
                        for stage in 0..3 {
                            if stage == 1 {
                                fixture.write(if open {
                                    &[0xff]
                                } else {
                                    b"open theorem Changed: 0 = 1;\n"
                                });
                            } else if stage == 2 {
                                fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                            }
                            for _ in 0..2 {
                                assert_eq!(
                                    inputs
                                        .native_theorem_registration_closure_with_type_parent(
                                            &input, owner
                                        )
                                        .unwrap(),
                                    Some(expected.clone())
                                );
                            }
                            assert_eq!(*owner.storage().get(&typed).unwrap(), *retained);
                        }
                        assert_eq!(
                            (
                                input,
                                type_input,
                                typed.clone(),
                                (*owner.storage().get(&typed).unwrap()).clone(),
                                session.captured().clone(),
                                expected
                            ),
                            before
                        );
                        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                            snapshot: old.snapshot,
                            protected_outputs: vec![],
                        });
                        assert_eq!(collected.outputs_dropped, 5);
                        assert_eq!(collected.abandoned_slots_dropped, 0);
                    },
                );
            }
        }
    }
}

#[test]
fn native_theorem_registration_closure_type_caller_preserves_refusals() {
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in [
        "missing",
        "extra",
        "wrong-phase",
        "foreign-owner",
        "snapshot",
        "module",
        "hash",
        "obsolete",
        "collected",
    ] {
        with_native_summary_parent(text, false, 0, |_, inputs, old, owner, _, _| {
            for (phase, kind) in [
                ("SignatureCollection", "SourceTheoremModuleSummary"),
                ("TypeChecking", "SourceTheoremTypedAst"),
            ] {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new(phase),
                    OutputKind::new(kind),
                    IrWorkUnit::new("\"alpha\":\"main\""),
                ));
            }
            let seal = |parent| {
                SealedParentOutputHandle::from_published_output(owner, old.snapshot, parent)
                    .unwrap()
            };
            let signature = inputs
                .publish_native_theorem_module_summary(old, owner)
                .unwrap()
                .unwrap();
            let type_input = PhaseInput::new(
                old.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    old.snapshot,
                    old.identities().input_hash(),
                    vec![],
                    vec![seal(signature.erase())],
                )
                .unwrap(),
            );
            let typed = inputs
                .publish_native_theorem_typed_ast(&type_input, owner)
                .unwrap()
                .unwrap();
            let input = PhaseInput::new(
                if case == "module" {
                    WorkUnit::Module {
                        module: mizar_build::module_index::ModuleId::new(
                            PackageId::new("alpha"),
                            ModulePath::new("other"),
                        ),
                    }
                } else {
                    old.work_unit.clone()
                },
                PhaseDispatchInputBundle::new(
                    if case == "snapshot" {
                        snapshot_id(0xf8)
                    } else {
                        old.snapshot
                    },
                    if case == "hash" {
                        hash(0xf9)
                    } else {
                        old.identities().input_hash()
                    },
                    vec![],
                    match case {
                        "missing" | "snapshot" => vec![],
                        "extra" => vec![seal(typed.erase()), seal(signature.erase())],
                        "wrong-phase" => vec![seal(signature.erase())],
                        _ => vec![seal(typed.erase())],
                    },
                )
                .unwrap(),
            );
            if case == "obsolete" {
                owner.mark_obsolete(old.snapshot).unwrap();
            } else if case == "collected" {
                assert_eq!(
                    owner
                        .storage()
                        .collect(mizar_ir::storage::CollectInput {
                            snapshot: old.snapshot,
                            protected_outputs: vec![]
                        })
                        .outputs_dropped,
                    5
                );
            }
            let foreign = PhaseOutputPublisher::new(
                owner.storage().clone(),
                std::sync::Arc::new(SnapshotHandleRegistry::new()),
            );
            foreign.register_current_snapshot(old.snapshot);
            let selected = if case == "foreign-owner" {
                &foreign
            } else {
                owner.as_ref()
            };
            let error = inputs
                .native_theorem_registration_closure_with_type_parent(&input, selected)
                .unwrap_err();
            if matches!(case, "missing" | "extra" | "hash") {
                assert_eq!(
                    error,
                    "native theorem registration closure requires the leaf input profile"
                );
            } else if case == "wrong-phase" {
                assert_eq!(
                    error,
                    "native theorem registration closure parent binding mismatch"
                );
            } else if case == "snapshot" {
                assert_eq!(
                    error,
                    "native theorem registration closure snapshot mismatch"
                );
            }
            let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(
                collected.outputs_dropped,
                if case == "collected" { 0 } else { 5 }
            );
            assert_eq!(collected.abandoned_slots_dropped, 0);
        });
    }
    with_native_summary_parent(
        text,
        false,
        0,
        |_, inputs, old, owner, outputs, (_, ids, snapshots)| {
            use mizar_checker::type_checker::{
                DeclarationChecker, check_source_theorem_registration_closure as close,
            };
            use mizar_driver::{
                native_theorem_typed_ast_from_storage_bytes as decode,
                native_theorem_typed_ast_storage_bytes as encode,
            };
            use mizar_ir::{
                publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
                storage::{BlobDecodeError, BlobDecoder, IrSideTables, SchemaVersion},
            };
            use mizar_resolve::{
                names::{
                    SourceVariableScopeInput as Input, SourceVariableScopeResolver as Resolver,
                },
                symbols::{
                    source_theorem_module_candidates as candidates,
                    source_theorem_module_original_inputs as original_inputs,
                },
            };
            let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
            for (phase, kind) in [
                ("SignatureCollection", "SourceTheoremModuleSummary"),
                ("TypeChecking", "SourceTheoremTypedAst"),
            ] {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new(phase),
                    OutputKind::new(kind),
                    IrWorkUnit::new("\"alpha\":\"main\""),
                ));
            }
            let signature = inputs
                .publish_native_theorem_module_summary(old, owner)
                .unwrap()
                .unwrap();
            let frontend = outputs
                .iter()
                .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                .unwrap();
            let frontend = owner
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    frontend,
                    frontend.output_kind(),
                )
                .unwrap();
            let output = owner.storage().get(&frontend).unwrap();
            let ast = output.ast.as_ref().unwrap();
            let changed = Fixture::new(b"theorem Different: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;");
            let (_, _, other_owner, other_parent, _) =
                native_disk_frontend_record_submission(&changed, ids, snapshots, 0);
            let other = other_owner
                .publisher()
                .storage()
                .get(&other_parent)
                .unwrap();
            let other_ast = other.ast.as_ref().unwrap();
            assert_ne!(other_ast.source_id, ast.source_id);
            let rebound = <MizarParserSeam as ParserSeam>::Ast::from_canonical_bytes(
                &other_ast.canonical_bytes().unwrap(),
                ast.source_id,
            )
            .unwrap();
            let [
                (arena, env, scope, typed_a, closure_a),
                (_, _, _, typed_b, closure_b),
            ] = [ast, &rebound].map(|ast| {
                let (arena, env) = original_inputs(ast, &module).unwrap().unwrap();
                assert!(candidates(ast, &arena, &env).unwrap().is_some());
                let scope =
                    Resolver::resolve_proof_occurrences(Input::new(ast, &module, &env)).unwrap();
                let typed =
                    DeclarationChecker::check_source_theorem_equalities_with_typed_declarations(
                        &arena, &env, &scope,
                    )
                    .unwrap()
                    .2;
                let closure = close(&arena, &env, &scope, &typed).unwrap();
                (arena, env, scope, typed, closure)
            });
            assert_eq!(typed_b.source_id(), typed_a.source_id());
            assert_eq!(typed_b.module_id(), typed_a.module_id());
            assert_ne!(typed_a, typed_b);
            assert_eq!(closure_a, closure_b);
            let expected = close(&arena, &env, &scope, &typed_b).unwrap_err();
            assert_eq!(
                expected,
                "theorems.equality.invalid_registration_closure_association"
            );
            let bytes = encode(&rebound, &module, &typed_b).unwrap().unwrap();
            assert_eq!(
                decode(&bytes, ast.source_id, &module).unwrap(),
                Some(typed_b.clone())
            );
            let phase = IrPipelinePhase::new("TypeChecking");
            let kind = OutputKind::new("SourceTheoremTypedAst");
            let source_id = ast.source_id;
            let decoder_module = module.clone();
            let negative = owner
                .publish(PublishOutputInput {
                    slot: owner.allocate(
                        old.snapshot,
                        phase.clone(),
                        signature.work_unit().clone(),
                        kind.clone(),
                        SchemaVersion::new(1),
                    ),
                    snapshot: old.snapshot,
                    phase,
                    work_unit: signature.work_unit().clone(),
                    output_kind: kind,
                    schema_version: SchemaVersion::new(1),
                    payload: typed_b.clone(),
                    canonical_payload: Some(bytes.clone()),
                    storage_payload: Some(bytes),
                    decode: BlobDecoder::new(move |bytes| {
                        decode(bytes, source_id, &decoder_module)
                            .map_err(BlobDecodeError::new)?
                            .ok_or_else(|| BlobDecodeError::new("invalid negative typed replay"))
                    }),
                    parents: vec![signature.erase()],
                    named_input_hashes: signature.lineage().named_input_hashes.clone(),
                    side_tables: IrSideTables::default(),
                    origin: OutputOrigin::PackageSource,
                    target: PublicationTarget::CurrentPackage,
                })
                .unwrap();
            assert_eq!(*owner.storage().get(&negative).unwrap(), typed_b);
            let input = PhaseInput::new(
                old.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    old.snapshot,
                    old.identities().input_hash(),
                    vec![],
                    vec![
                        SealedParentOutputHandle::from_published_output(
                            owner,
                            old.snapshot,
                            negative.erase(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            );
            for _ in 0..2 {
                assert_eq!(
                    inputs
                        .native_theorem_registration_closure_with_type_parent(&input, owner)
                        .unwrap_err(),
                    expected
                );
            }
            let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(collected.outputs_dropped, 5);
            assert_eq!(collected.abandoned_slots_dropped, 0);
        },
    );
}

#[test]
fn native_theorem_registration_closure_publication_seals_genuine_payload_and_streams() {
    use mizar_checker::{
        cluster_trace::ClusterClosureStatus, registration_resolution::RegistrationDatabase,
        type_checker::check_source_theorem_registration_closure as close,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        source::{FrontendSourceLoader, diagnostic_file_path, source_unit_from_loaded},
    };
    use mizar_ir::{
        publisher::PublishError,
        storage::{IrSideTables, SchemaVersion, SideTableRecord, StorageError},
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput as Input, SourceVariableScopeResolver as Resolver},
        symbols::{
            source_theorem_module_original_inputs as original_inputs,
            source_theorem_module_semantic_payload as semantics,
        },
    };
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let field = |hasher: &mut blake3::Hasher, bytes: &[u8]| {
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    };
    for (open, threshold) in [(false, usize::MAX), (true, 0)] {
        let mut representations = Vec::new();
        for text in [
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            "theorem T: for X being set holds X = X\nproof\n  let X be set;\n  thus X = X;\nend;\n",
            "theorem Renamed: for X being set holds X = X proof let X be set; thus X = X; end;",
            include_str!(
                "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
            ),
        ] {
            with_native_summary_parent(
                text,
                open,
                threshold,
                |fixture, inputs, old, owner, _, (session, _, _)| {
                    let unit = IrWorkUnit::new("\"alpha\":\"main\"");
                    for (phase, kind) in [
                        ("SignatureCollection", "SourceTheoremModuleSummary"),
                        ("TypeChecking", "SourceTheoremTypedAst"),
                        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                    ] {
                        owner.allow_work_unit(AllowedWorkUnit::new(
                            IrPipelinePhase::new(phase),
                            OutputKind::new(kind),
                            unit.clone(),
                        ));
                    }
                    let signature = inputs
                        .publish_native_theorem_module_summary(old, owner)
                        .unwrap()
                        .unwrap();
                    let type_input = PhaseInput::new(
                        old.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            old.identities().input_hash(),
                            vec![],
                            vec![
                                SealedParentOutputHandle::from_published_output(
                                    owner,
                                    old.snapshot,
                                    signature.erase(),
                                )
                                .unwrap(),
                            ],
                        )
                        .unwrap(),
                    );
                    let parent = inputs
                        .publish_native_theorem_typed_ast(&type_input, owner)
                        .unwrap()
                        .unwrap();
                    let input = PhaseInput::new(
                        old.work_unit.clone(),
                        PhaseDispatchInputBundle::new(
                            old.snapshot,
                            old.identities().input_hash(),
                            vec![],
                            vec![
                                SealedParentOutputHandle::from_published_output(
                                    owner,
                                    old.snapshot,
                                    parent.erase(),
                                )
                                .unwrap(),
                            ],
                        )
                        .unwrap(),
                    );
                    let typed = owner.storage().get(&parent).unwrap();
                    let (_, loaded, descriptor) =
                        &inputs.captured.unwrap().native_sources().unwrap()[0];
                    let output = Frontend::new(
                        FrontendSourceLoader::new(DiskSourceLoader::new(
                            fixture.root.join("alpha"),
                        )),
                        inputs.dependency_lexical_provider(&[]),
                        MizarParserSeam,
                    )
                    .run_loaded(source_unit_from_loaded(
                        loaded.clone(),
                        diagnostic_file_path(descriptor),
                    ))
                    .unwrap();
                    assert!(output.diagnostics.is_empty());
                    assert!(output.cache_keys.ast.is_some());
                    let ast = output.ast.as_ref().unwrap();
                    let (arena, env) = original_inputs(ast, &module).unwrap().unwrap();
                    let scope = Resolver::resolve_proof_occurrences(Input::new(ast, &module, &env))
                        .unwrap();
                    let expected = close(&arena, &env, &scope, &typed).unwrap();
                    assert_eq!(expected.status(), ClusterClosureStatus::Complete);
                    assert_eq!(expected.trace().source_id(), loaded.source_id);
                    assert_eq!(expected.trace().module_id(), &module);
                    let profile = expected
                        .trace()
                        .traversal_profile()
                        .cache_key_material()
                        .as_str();
                    assert_eq!(
                        profile,
                        "order=cluster-trace-order-v1;max_depth=128;max_generated=4096"
                    );
                    assert!(expected.closure_facts().is_empty());
                    assert!(expected.diagnostics().is_empty());
                    assert!(typed.facts().is_empty());
                    assert!(typed.coercions().is_empty());
                    assert!(typed.initial_obligations().is_empty());
                    let database = RegistrationDatabase::from_symbol_env(&env, []);
                    assert!(database.pending().is_empty());
                    assert!(database.activated().is_empty());
                    assert!(database.rejected().is_empty());
                    assert!(database.diagnostics().is_empty());
                    assert!(database.initial_obligations().is_empty());
                    let opaque = semantics(ast, &module).unwrap().unwrap().2;
                    let mut content_bytes =
                        b"mizar-driver/native-theorem-registration-closure-content/v1\0".to_vec();
                    for value in [opaque.as_slice(), profile.as_bytes()] {
                        content_bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
                        content_bytes.extend_from_slice(value);
                    }
                    let syntax = ast.canonical_bytes().unwrap();
                    let mut bytes =
                        b"mizar-driver/native-theorem-registration-closure-storage/v1\0".to_vec();
                    for value in [
                        b"alpha".as_slice(),
                        b"main".as_slice(),
                        syntax.as_slice(),
                        profile.as_bytes(),
                    ] {
                        bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
                        bytes.extend_from_slice(value);
                    }
                    let version = &inputs.snapshot.source_versions[0];
                    let mut content = blake3::Hasher::new();
                    field(&mut content, b"mizar-ir/publisher-content-hash/v1");
                    field(&mut content, &content_bytes);
                    content.update(&1_u64.to_le_bytes());
                    field(&mut content, b"parent_content");
                    content.update(parent.content_hash().as_bytes());
                    content.update(&1_u64.to_le_bytes());
                    for value in [
                        "source",
                        SOURCE_UNIT_CACHE_KEY_VERSION,
                        "mizar-ir/publisher-named-input-hash/v1",
                    ] {
                        field(&mut content, value.as_bytes());
                    }
                    content.update(source_key(version).as_bytes());
                    let expected_content = Hash::from_bytes(*content.finalize().as_bytes());
                    let mut blob = blake3::Hasher::new();
                    field(&mut blob, b"mizar-ir/content-blob/v1");
                    blob.update(&1_u32.to_le_bytes());
                    field(&mut blob, &bytes);
                    let expected_blob = Hash::from_bytes(*blob.finalize().as_bytes());
                    let storage_digest = Hash::from_bytes(blake3::derive_key(
                        "mizar-driver/native-theorem-registration-closure-storage-side-table/v1",
                        &bytes,
                    ));
                    let side_tables = IrSideTables {
                        source_maps: vec![SideTableRecord::new(
                            "registration-theorem-closure-storage",
                            version.normalized_path.as_str(),
                            storage_digest,
                        )],
                        ..IrSideTables::default()
                    };
                    let mut side = blake3::Hasher::new();
                    field(&mut side, b"mizar-ir/publisher-side-table-hash/v1");
                    for category in [
                        "source_maps",
                        "diagnostics",
                        "explanation_refs",
                        "documentation_attachments",
                    ] {
                        field(&mut side, category.as_bytes());
                        side.update(&u64::from(category == "source_maps").to_le_bytes());
                        if category == "source_maps" {
                            field(&mut side, b"registration-theorem-closure-storage");
                            field(&mut side, version.normalized_path.as_str().as_bytes());
                            field(&mut side, b"side_table_digest");
                            side.update(storage_digest.as_bytes());
                        }
                    }
                    let expected_side = Hash::from_bytes(*side.finalize().as_bytes());
                    let before = (
                        input.clone(),
                        parent.clone(),
                        (*typed).clone(),
                        session.captured().clone(),
                        expected.clone(),
                    );
                    let sealed = inputs
                        .publish_native_theorem_registration_closure(&input, owner)
                        .unwrap()
                        .unwrap();
                    assert_eq!(sealed.snapshot(), input.snapshot);
                    assert_eq!(
                        sealed.phase(),
                        &IrPipelinePhase::new("RegistrationResolution")
                    );
                    assert_eq!(
                        sealed.output_kind(),
                        &OutputKind::new("SourceTheoremRegistrationClosure")
                    );
                    assert_eq!(sealed.schema_version(), SchemaVersion::new(1));
                    assert_eq!(sealed.work_unit(), &unit);
                    assert_eq!(sealed.lineage().parents, vec![parent.output()]);
                    assert_eq!(
                        sealed.lineage().named_input_hashes,
                        parent.lineage().named_input_hashes
                    );
                    assert_eq!(sealed.content_hash(), expected_content);
                    assert_eq!(sealed.side_table_hash(), expected_side);
                    assert_eq!(*owner.storage().side_tables(&sealed).unwrap(), side_tables);
                    match sealed.placement() {
                        StoragePlacement::Resident => assert!(!open),
                        StoragePlacement::Blob { blob, len } => {
                            assert!(open);
                            assert_eq!(blob.hash(), expected_blob);
                            assert_eq!(*len, bytes.len());
                        }
                        _ => panic!("unexpected placement"),
                    }
                    for stage in 0..3 {
                        if stage == 1 {
                            fixture.write(if open {
                                &[0xff]
                            } else {
                                b"open theorem Changed: 0 = 1;\n"
                            });
                        } else if stage == 2 {
                            fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                        }
                        assert_eq!(*owner.storage().get(&sealed).unwrap(), expected);
                        let duplicate = PublishError::Storage {
                            error: Box::new(StorageError::OutputAlreadyStored {
                                output: sealed.output(),
                            }),
                        }
                        .to_string();
                        assert_eq!(
                            inputs
                                .publish_native_theorem_registration_closure(&input, owner)
                                .unwrap_err(),
                            duplicate
                        );
                        owner
                            .validate_published_output(input.snapshot, sealed.any())
                            .unwrap();
                        assert_eq!(
                            owner
                                .validate_current_output(input.snapshot, sealed.any())
                                .is_ok(),
                            !open
                        );
                    }
                    assert_eq!(
                        (
                            input,
                            parent.clone(),
                            (*owner.storage().get(&parent).unwrap()).clone(),
                            session.captured().clone(),
                            expected
                        ),
                        before
                    );
                    let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(collected.outputs_dropped, 6);
                    assert_eq!(collected.abandoned_slots_dropped, 3);
                    representations.push((content_bytes, bytes));
                },
            );
        }
        assert_eq!(representations[0].0, representations[1].0);
        assert_ne!(representations[0].0, representations[2].0);
        assert_ne!(representations[0].1, representations[1].1);
    }
}

#[test]
fn native_theorem_registration_closure_publication_preserves_refusals() {
    use mizar_ir::{publisher::PublishError, storage::CollectInput};
    let text = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for case in [
        "missing",
        "wrong-phase",
        "hash",
        "obsolete",
        "collected",
        "foreign-owner",
        "denied",
    ] {
        with_native_summary_parent(text, false, 0, |_, inputs, old, owner, _, _| {
            let unit = IrWorkUnit::new("\"alpha\":\"main\"");
            for (phase, kind) in [
                ("SignatureCollection", "SourceTheoremModuleSummary"),
                ("TypeChecking", "SourceTheoremTypedAst"),
            ] {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new(phase),
                    OutputKind::new(kind),
                    unit.clone(),
                ));
            }
            let signature = inputs
                .publish_native_theorem_module_summary(old, owner)
                .unwrap()
                .unwrap();
            let type_input = PhaseInput::new(
                old.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    old.snapshot,
                    old.identities().input_hash(),
                    vec![],
                    vec![
                        SealedParentOutputHandle::from_published_output(
                            owner,
                            old.snapshot,
                            signature.erase(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            );
            let parent = inputs
                .publish_native_theorem_typed_ast(&type_input, owner)
                .unwrap()
                .unwrap();
            let input = PhaseInput::new(
                old.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    old.snapshot,
                    if case == "hash" {
                        hash(0xef)
                    } else {
                        old.identities().input_hash()
                    },
                    vec![],
                    if case == "missing" {
                        vec![]
                    } else {
                        vec![
                            SealedParentOutputHandle::from_published_output(
                                owner,
                                old.snapshot,
                                if case == "wrong-phase" {
                                    signature.erase()
                                } else {
                                    parent.erase()
                                },
                            )
                            .unwrap(),
                        ]
                    },
                )
                .unwrap(),
            );
            if case != "denied" {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("RegistrationResolution"),
                    OutputKind::new("SourceTheoremRegistrationClosure"),
                    unit.clone(),
                ));
            }
            if case == "obsolete" {
                owner.mark_obsolete(old.snapshot).unwrap();
            } else if case == "collected" {
                assert_eq!(
                    owner
                        .storage()
                        .collect(CollectInput {
                            snapshot: old.snapshot,
                            protected_outputs: vec![]
                        })
                        .outputs_dropped,
                    5
                );
            }
            let foreign = PhaseOutputPublisher::new(
                owner.storage().clone(),
                std::sync::Arc::new(SnapshotHandleRegistry::new()),
            );
            foreign.register_current_snapshot(old.snapshot);
            let selected = if case == "foreign-owner" {
                &foreign
            } else {
                owner.as_ref()
            };
            let raw = inputs.native_theorem_registration_closure_with_type_parent(&input, selected);
            let actual = inputs
                .publish_native_theorem_registration_closure(&input, selected)
                .unwrap_err();
            if case == "denied" {
                assert!(raw.unwrap().is_some());
                assert_eq!(
                    actual,
                    PublishError::WorkUnitNotAllowed {
                        phase: IrPipelinePhase::new("RegistrationResolution"),
                        work_unit: unit,
                        output_kind: OutputKind::new("SourceTheoremRegistrationClosure")
                    }
                    .to_string()
                );
            } else {
                assert_eq!(actual, raw.unwrap_err(), "{case}");
            }
            let collected = owner.storage().collect(CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(
                collected.outputs_dropped,
                if case == "collected" { 0 } else { 5 }
            );
            assert_eq!(
                collected.abandoned_slots_dropped,
                usize::from(case == "denied")
            );
        });
    }
}
