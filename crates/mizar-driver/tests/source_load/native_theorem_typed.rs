use super::*;

#[test]
fn native_theorem_typed_ast_storage_replay_preserves_genuine_outputs() {
    use mizar_checker::type_checker::DeclarationChecker as Checker;
    use mizar_driver::native_theorem_typed_ast_from_storage_bytes as decode;
    use mizar_driver::native_theorem_typed_ast_storage_bytes as encode;
    use mizar_resolve::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    use mizar_resolve::symbols::{
        source_theorem_module_candidates as candidates,
        source_theorem_module_original_inputs as original_inputs,
    };
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let capture = |fixture: &Fixture,
                   open: bool,
                   threshold: usize,
                   ids: &InMemorySessionIdAllocator,
                   snapshots: &SnapshotRegistry<InMemorySessionIdAllocator>| {
        if !open {
            let (_, _, adapter, parent, _) =
                native_disk_frontend_record_submission(fixture, ids, snapshots, threshold);
            assert_eq!(
                parent.placement() == &StoragePlacement::Resident,
                threshold != 0
            );
            return (*adapter
                .publisher()
                .storage()
                .get(&parent)
                .unwrap()
                .ast
                .as_ref()
                .unwrap())
            .clone();
        }
        let (mut draft, submit) = fixture.submit_request(vec![], vec![]);
        draft.source_inputs.versions.clear();
        let descriptor = SourceInput {
            package_id: fixture.version.package_id.clone(),
            module_path: fixture.version.module_path.clone(),
            normalized_path: fixture.version.normalized_path.clone(),
            edition: fixture.version.edition.clone(),
            origin: SourceOriginInput::OpenBuffer {
                uri: format!(
                    "file://{}",
                    fixture.root.join("alpha/src/main.miz").display()
                ),
                expected_version: 7,
                actual_version: 7,
                text: std::sync::Arc::from(std::str::from_utf8(&fixture.bytes).unwrap()),
            },
        };
        fixture.write(&[0xff]);
        let session = draft
            .capture_sources(
                ids,
                snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    descriptor,
                )],
            )
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        let work_unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        let owner = publisher(snapshot, threshold, &work_unit);
        allow_frontend(&owner, &work_unit);
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let submission = CompilerDriver::new(builder.build().unwrap())
            .with_output_publisher(owner.clone())
            .submit_captured(session, ids, snapshots, submit)
            .unwrap();
        let parent = submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .flat_map(|result| &result.output_refs)
            .find(|handle| handle.output_kind() == &OutputKind::new("FrontendOutput"))
            .unwrap();
        assert_eq!(
            parent.placement() == &StoragePlacement::Resident,
            threshold != 0
        );
        let retained = owner
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                parent,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let output = owner.storage().get(&retained).unwrap();
        (*output.ast.as_ref().unwrap()).clone()
    };
    let check = |ast: &<MizarParserSeam as ParserSeam>::Ast| {
        let (arena, env) = original_inputs(ast, &module).unwrap().unwrap();
        assert!(candidates(ast, &arena, &env).unwrap().is_some());
        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(ast, &module, &env),
        )
        .unwrap();
        Checker::check_source_theorem_equalities_with_typed_declarations(&arena, &env, &scope)
            .unwrap()
            .2
    };
    let texts = [
        include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        ),
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
        include_str!(
            "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
        ),
        "theorem T: for Y being set holds Y = Y\r\nproof\r\n  let Y be set;\r\n  thus Y = Y;\r\nend;\r\n",
    ];
    for text in texts {
        for open in [false, true] {
            for threshold in [usize::MAX, 0] {
                let fixture = Fixture::new(text.as_bytes());
                let ids = InMemorySessionIdAllocator::new();
                let snapshots = SnapshotRegistry::new();
                let asts = [
                    capture(&fixture, open, threshold, &ids, &snapshots),
                    capture(&fixture, open, threshold, &ids, &snapshots),
                ];
                assert_ne!(asts[0].source_id, asts[1].source_id);
                let mut baselines = Vec::new();
                for ast in asts {
                    let typed = check(&ast);
                    assert_eq!(typed.source_id(), ast.source_id);
                    assert_eq!(typed.module_id(), &module);
                    assert!(!typed.types().is_empty());
                    let declarations = typed.source_context().unwrap().declarations();
                    assert!(declarations.iter().next().is_some());
                    for (_, declaration) in declarations.iter() {
                        let node = typed.nodes().node(declaration.site.node()).unwrap();
                        assert_eq!(
                            node.typing,
                            mizar_checker::typed_ast::TypingState::Successful
                        );
                        assert!(node.links.type_entry.is_some());
                    }
                    let syntax = ast.canonical_bytes().unwrap();
                    let mut expected =
                        b"mizar-driver/native-theorem-typed-ast-storage/v1\0".to_vec();
                    for field in [b"alpha".as_slice(), b"main".as_slice(), syntax.as_slice()] {
                        expected.extend(u64::try_from(field.len()).unwrap().to_le_bytes());
                        expected.extend_from_slice(field);
                    }
                    baselines.push((ast, typed, expected));
                }
                let before = baselines.clone();
                for stage in 0..3 {
                    if stage == 1 {
                        fixture.write(b"open theorem Changed: 0 = 1;\n");
                    } else if stage == 2 {
                        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                    }
                    for (index, (ast, typed, bytes)) in baselines.iter().enumerate() {
                        let other = &baselines[1 - index];
                        assert_eq!(encode(ast, &module, typed).unwrap(), Some(bytes.clone()));
                        assert_eq!(
                            decode(bytes, other.0.source_id, &module).unwrap(),
                            Some(other.1.clone())
                        );
                        assert_eq!(encode(ast, &module, &other.1).unwrap(), None);
                    }
                    assert_eq!(baselines, before);
                }
            }
        }
    }
}

#[test]
fn native_theorem_typed_ast_storage_replay_preserves_refusals() {
    use mizar_checker::type_checker::DeclarationChecker as Checker;
    use mizar_driver::native_theorem_typed_ast_from_storage_bytes as decode;
    use mizar_driver::native_theorem_typed_ast_storage_bytes as encode;
    use mizar_resolve::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    use mizar_resolve::symbols::{
        source_theorem_module_candidates as candidates,
        source_theorem_module_original_inputs as original_inputs,
    };
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let lower = |ast: &<MizarParserSeam as ParserSeam>::Ast| {
        let Some((arena, env)) = original_inputs(ast, &module)? else {
            return Ok(None);
        };
        if candidates(ast, &arena, &env)?.is_none() {
            return Ok(None);
        }
        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(ast, &module, &env),
        )
        .map_err(|error| format!("{error:?}"))?;
        Checker::check_source_theorem_equalities_with_typed_declarations(&arena, &env, &scope)
            .map(|checked| Some(checked.2))
    };
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let valid = Fixture::new(
        include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        )
        .as_bytes(),
    );
    let (_, _, adapter, parent, _) =
        native_disk_frontend_record_submission(&valid, &ids, &snapshots, 0);
    let owner = adapter.publisher();
    let output = owner.storage().get(&parent).unwrap();
    let valid_ast = output.ast.as_ref().unwrap();
    let valid_typed = lower(valid_ast).unwrap().unwrap();
    let valid_bytes = encode(valid_ast, &module, &valid_typed).unwrap().unwrap();
    let domain = b"mizar-driver/native-theorem-typed-ast-storage/v1\0";
    let frame = |syntax: &[u8]| {
        let mut bytes = domain.to_vec();
        for field in [b"alpha".as_slice(), b"main".as_slice(), syntax] {
            bytes.extend(u64::try_from(field.len()).unwrap().to_le_bytes());
            bytes.extend_from_slice(field);
        }
        bytes
    };
    let mut corruptions = vec![
        Vec::new(),
        valid_bytes[..domain.len() + 3].to_vec(),
        valid_bytes[..valid_bytes.len() - 1].to_vec(),
    ];
    let mut bad_domain = valid_bytes.clone();
    bad_domain[0] = b'x';
    corruptions.push(bad_domain);
    let mut bad_version = valid_bytes.clone();
    bad_version[domain.len() - 2] = b'2';
    corruptions.push(bad_version);
    let mut bad_length = valid_bytes.clone();
    bad_length[domain.len()..domain.len() + 8].copy_from_slice(&u64::MAX.to_le_bytes());
    corruptions.push(bad_length);
    let mut trailing = valid_bytes.clone();
    trailing.push(0);
    corruptions.push(trailing);
    corruptions.push(frame(&[0xff]));
    let mut oversized = valid_bytes.clone();
    oversized.resize(32 * 1024 * 1024 + 1, 0);
    corruptions.push(oversized);
    for malformed in corruptions {
        assert_eq!(decode(&malformed, valid_ast.source_id, &module), Ok(None));
    }
    for wrong_module in [
        ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("main")),
        ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("other")),
    ] {
        assert_eq!(
            decode(&valid_bytes, valid_ast.source_id, &wrong_module),
            Ok(None)
        );
        assert_eq!(encode(valid_ast, &wrong_module, &valid_typed), Ok(None));
    }
    let oversized_module = ResolverModuleId::new(
        PackageId::new("x".repeat(32 * 1024 * 1024)),
        ModulePath::new("main"),
    );
    assert_eq!(encode(valid_ast, &oversized_module, &valid_typed), Ok(None));
    let changed = Fixture::new(b"theorem T: for X being set holds X = X proof end;");
    let (_, _, changed_adapter, changed_parent, _) =
        native_disk_frontend_record_submission(&changed, &ids, &snapshots, 0);
    let changed_ast = changed_adapter
        .publisher()
        .storage()
        .get(&changed_parent)
        .unwrap();
    let foreign_syntax = changed_ast.ast.as_ref().unwrap().canonical_bytes().unwrap();
    let same_source_ast = <MizarParserSeam as ParserSeam>::Ast::from_canonical_bytes(
        &foreign_syntax,
        valid_ast.source_id,
    )
    .unwrap();
    let same_source_typed = lower(&same_source_ast).unwrap().unwrap();
    assert_eq!(same_source_typed.source_id(), valid_typed.source_id());
    assert_ne!(same_source_typed, valid_typed);
    assert_eq!(encode(valid_ast, &module, &same_source_typed), Ok(None));
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, case) in [
        (format!("{private}\nreserve Y for set;\n"), "reserve"),
        (private.replace("by PrivT1", "by Missing"), "citation"),
        (private.replace("thus X = X;", "thus Y = Y;"), "unbound"),
        (
            private.replacen("thus X = X;", "thus X = 0;", 1),
            "unsupported-type",
        ),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let (_, _, adapter, parent, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let retained = adapter.publisher().storage().get(&parent).unwrap();
        let ast = retained.ast.as_ref().unwrap();
        let syntax = ast.canonical_bytes().unwrap();
        let bytes = frame(&syntax);
        let expected = lower(ast);
        assert!(matches!(
            (&expected, case),
            (Ok(None), "reserve" | "citation") | (Err(_), "unbound" | "unsupported-type")
        ));
        assert_eq!(decode(&bytes, ast.source_id, &module), expected, "{case}");
        assert_eq!(
            encode(ast, &module, &valid_typed),
            expected.map(|_| None::<Vec<u8>>),
            "{case}"
        );
    }
}

#[test]
fn native_theorem_typed_ast_signature_caller_preserves_genuine_outputs() {
    use mizar_checker::{type_checker::DeclarationChecker, typed_ast::TypingState};
    use mizar_frontend::{orchestration::Frontend, source::FrontendSourceLoader};
    use mizar_resolve::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    use mizar_resolve::symbols::{
        source_theorem_module_candidates as candidates,
        source_theorem_module_original_inputs as original_inputs,
    };
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
                    |fixture, inputs, old_input, owner, outputs, (session, _, _)| {
                        let WorkUnit::Module { module } = &old_input.work_unit else {
                            unreachable!()
                        };
                        let resolved = outputs
                            .iter()
                            .find(|output| output.phase() == &IrPipelinePhase::new("ModuleResolve"))
                            .unwrap();
                        owner.allow_work_unit(AllowedWorkUnit::new(
                            IrPipelinePhase::new("SignatureCollection"),
                            OutputKind::new("SourceTheoremModuleSummary"),
                            resolved.work_unit().clone(),
                        ));
                        let summary = inputs
                            .publish_native_theorem_module_summary(old_input, owner)
                            .unwrap()
                            .unwrap();
                        assert_eq!(
                            summary.placement() == &StoragePlacement::Resident,
                            threshold != 0
                        );
                        let input = PhaseInput::new(
                            old_input.work_unit.clone(),
                            PhaseDispatchInputBundle::new(
                                old_input.snapshot,
                                old_input.identities().input_hash(),
                                vec![],
                                vec![
                                    SealedParentOutputHandle::from_published_output(
                                        owner,
                                        old_input.snapshot,
                                        summary.erase(),
                                    )
                                    .unwrap(),
                                ],
                            )
                            .unwrap(),
                        );
                        let (_, loaded, descriptor) =
                            &inputs.captured.unwrap().native_sources().unwrap()[0];
                        let source = mizar_frontend::source::source_unit_from_loaded(
                            loaded.clone(),
                            mizar_frontend::source::diagnostic_file_path(descriptor),
                        );
                        assert_eq!(source.source_text.as_ref(), text);
                        let output = Frontend::new(
                            FrontendSourceLoader::new(DiskSourceLoader::new(
                                fixture.root.join("alpha"),
                            )),
                            inputs.dependency_lexical_provider(&[]),
                            MizarParserSeam,
                        )
                        .run_loaded(source)
                        .unwrap();
                        assert!(output.diagnostics.is_empty());
                        assert!(output.cache_keys.ast.is_some());
                        let ast = output.ast.as_ref().unwrap();
                        let resolver_module =
                            ResolverModuleId::new(module.package.clone(), module.path.clone());
                        let (arena, env) = original_inputs(ast, &resolver_module).unwrap().unwrap();
                        assert!(candidates(ast, &arena, &env).unwrap().is_some());
                        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                            SourceVariableScopeInput::new(ast, &resolver_module, &env),
                        )
                        .unwrap();
                        let expected = DeclarationChecker::check_source_theorem_equalities_with_typed_declarations(
                            &arena, &env, &scope,
                        ).unwrap().2;
                        assert_eq!(expected.source_id(), loaded.source_id);
                        assert_eq!(expected.module_id(), &resolver_module);
                        assert_eq!(expected.module_id().package().as_str(), "alpha");
                        assert_eq!(expected.module_id().path().as_str(), "main");
                        assert!(!expected.types().is_empty());
                        let declarations = expected.source_context().unwrap().declarations();
                        assert!(declarations.iter().next().is_some());
                        for (_, declaration) in declarations.iter() {
                            let node = expected.nodes().node(declaration.site.node()).unwrap();
                            assert_eq!(node.typing, TypingState::Successful);
                            assert!(node.links.type_entry.is_some());
                        }
                        let original_summary = owner.storage().get(&summary).unwrap();
                        let before = (
                            session.captured().clone(),
                            old_input.clone(),
                            input.clone(),
                            summary.clone(),
                            loaded.clone(),
                            descriptor.clone(),
                            (*original_summary).clone(),
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
                                        .native_theorem_typed_ast_with_signature_parent(
                                            &input, owner
                                        )
                                        .unwrap(),
                                    Some(expected.clone())
                                );
                            }
                            assert_eq!(*owner.storage().get(&summary).unwrap(), *original_summary);
                        }
                        assert_eq!(
                            (
                                session.captured().clone(),
                                old_input.clone(),
                                input,
                                summary.clone(),
                                loaded.clone(),
                                descriptor.clone(),
                                (*owner.storage().get(&summary).unwrap()).clone(),
                            ),
                            before
                        );
                        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                            snapshot: old_input.snapshot,
                            protected_outputs: vec![],
                        });
                        assert_eq!(collected.outputs_dropped, 4);
                        assert_eq!(collected.abandoned_slots_dropped, 0);
                    },
                );
            }
        }
    }
}

#[test]
fn native_theorem_typed_ast_signature_caller_preserves_refusals() {
    use mizar_checker::type_checker::DeclarationChecker;
    use mizar_frontend::{orchestration::Frontend, source::FrontendSourceLoader};
    use mizar_resolve::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    use mizar_resolve::symbols::{
        source_theorem_module_candidates as candidates,
        source_theorem_module_original_inputs as original_inputs,
    };
    let lemma = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let unsupported = private.replacen("thus X = X;", "thus X = 0;", 1);
    for case in [
        "checker",
        "missing",
        "extra",
        "wrong-phase",
        "foreign-owner",
        "snapshot",
        "module",
        "package",
        "hash",
        "dependencies",
        "obsolete",
        "collected",
    ] {
        with_native_summary_parent(
            if case == "checker" {
                &unsupported
            } else {
                lemma
            },
            false,
            0,
            |fixture, inputs, old_input, owner, outputs, _| {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("SignatureCollection"),
                    OutputKind::new("SourceTheoremModuleSummary"),
                    IrWorkUnit::new("\"alpha\":\"main\""),
                ));
                let summary = inputs
                    .publish_native_theorem_module_summary(old_input, owner)
                    .unwrap()
                    .unwrap();
                let parent = SealedParentOutputHandle::from_published_output(
                    owner,
                    old_input.snapshot,
                    summary.erase(),
                )
                .unwrap();
                let wrong_parent = || {
                    SealedParentOutputHandle::from_published_output(
                        owner,
                        old_input.snapshot,
                        outputs
                            .iter()
                            .find(|output| output.phase() == &IrPipelinePhase::new("ModuleResolve"))
                            .unwrap()
                            .clone(),
                    )
                    .unwrap()
                };
                let input = PhaseInput::new(
                    match case {
                        "module" => WorkUnit::Module {
                            module: mizar_build::module_index::ModuleId::new(
                                PackageId::new("alpha"),
                                ModulePath::new("other"),
                            ),
                        },
                        "package" => WorkUnit::Package {
                            package_id: PackageId::new("alpha"),
                        },
                        _ => old_input.work_unit.clone(),
                    },
                    PhaseDispatchInputBundle::new(
                        if case == "snapshot" {
                            snapshot_id(0xf1)
                        } else {
                            old_input.snapshot
                        },
                        if case == "hash" {
                            hash(0xf2)
                        } else {
                            old_input.identities().input_hash()
                        },
                        if case == "dependencies" {
                            vec![hash(0xf3)]
                        } else {
                            vec![]
                        },
                        match case {
                            "missing" | "snapshot" => vec![],
                            "extra" => vec![parent, wrong_parent()],
                            "wrong-phase" => vec![wrong_parent()],
                            _ => vec![parent],
                        },
                    )
                    .unwrap(),
                );
                let expected_error = if case == "checker" {
                    let (_, loaded, descriptor) =
                        &inputs.captured.unwrap().native_sources().unwrap()[0];
                    let source = mizar_frontend::source::source_unit_from_loaded(
                        loaded.clone(),
                        mizar_frontend::source::diagnostic_file_path(descriptor),
                    );
                    let output = Frontend::new(
                        FrontendSourceLoader::new(DiskSourceLoader::new(
                            fixture.root.join("alpha"),
                        )),
                        inputs.dependency_lexical_provider(&[]),
                        MizarParserSeam,
                    )
                    .run_loaded(source)
                    .unwrap();
                    let ast = output.ast.as_ref().unwrap();
                    let module =
                        ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                    let (arena, env) = original_inputs(ast, &module).unwrap().unwrap();
                    assert!(candidates(ast, &arena, &env).unwrap().is_some());
                    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                        SourceVariableScopeInput::new(ast, &module, &env),
                    )
                    .unwrap();
                    Some(DeclarationChecker::check_source_theorem_equalities_with_typed_declarations(
                        &arena, &env, &scope,
                    ).unwrap_err())
                } else {
                    None
                };
                if case == "obsolete" {
                    owner.mark_obsolete(old_input.snapshot).unwrap();
                } else if case == "collected" {
                    assert_eq!(
                        owner
                            .storage()
                            .collect(mizar_ir::storage::CollectInput {
                                snapshot: old_input.snapshot,
                                protected_outputs: vec![],
                            })
                            .outputs_dropped,
                        4
                    );
                }
                let foreign = PhaseOutputPublisher::new(
                    owner.storage().clone(),
                    std::sync::Arc::new(SnapshotHandleRegistry::new()),
                );
                foreign.register_current_snapshot(old_input.snapshot);
                let selected = if case == "foreign-owner" {
                    &foreign
                } else {
                    owner.as_ref()
                };
                let actual = inputs
                    .native_theorem_typed_ast_with_signature_parent(&input, selected)
                    .unwrap_err();
                if let Some(expected) = expected_error {
                    assert_eq!(actual, expected);
                } else if matches!(case, "missing" | "extra" | "hash" | "dependencies") {
                    assert_eq!(
                        actual,
                        "native theorem TypedAst requires the leaf input profile"
                    );
                } else if case == "wrong-phase" {
                    assert_eq!(actual, "native theorem TypedAst parent binding mismatch");
                } else if case == "snapshot" {
                    assert_eq!(actual, "native theorem TypedAst snapshot mismatch");
                }
                let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                    snapshot: old_input.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    collected.outputs_dropped,
                    if case == "collected" { 0 } else { 4 }
                );
                assert_eq!(collected.abandoned_slots_dropped, 0);
            },
        );
    }
    with_native_summary_parent(
        lemma,
        false,
        usize::MAX,
        |_, inputs, old_input, owner, outputs, _| {
            use mizar_artifact::module_summary::{
                ModuleSummaryReadOptions, read_module_summary, write_module_summary,
            };
            use mizar_ir::{
                publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
                storage::{BlobDecodeError, BlobDecoder, IrSideTables, SchemaVersion},
            };
            let mut foreign_summary = None;
            with_native_summary_parent(
                "theorem Different: for X being set holds X = X proof let X be set; thus X = X; end;",
                false,
                usize::MAX,
                |_, other, input, _, _, _| {
                    let WorkUnit::Module { module } = &input.work_unit else {
                        unreachable!()
                    };
                    foreign_summary = other
                        .native_theorem_module_summary_candidate(module)
                        .unwrap();
                },
            );
            let mut foreign_summary = foreign_summary.unwrap();
            foreign_summary.source_hash = inputs.snapshot.source_versions[0].source_hash;
            let parent = outputs
                .iter()
                .find(|output| output.phase() == &IrPipelinePhase::new("ModuleResolve"))
                .unwrap();
            let phase = IrPipelinePhase::new("SignatureCollection");
            let kind = OutputKind::new("SourceTheoremModuleSummary");
            owner.allow_work_unit(AllowedWorkUnit::new(
                phase.clone(),
                kind.clone(),
                parent.work_unit().clone(),
            ));
            let bytes = write_module_summary(&foreign_summary).unwrap();
            let negative = owner
                .publish(PublishOutputInput {
                    slot: owner.allocate(
                        old_input.snapshot,
                        phase.clone(),
                        parent.work_unit().clone(),
                        kind.clone(),
                        SchemaVersion::new(1),
                    ),
                    snapshot: old_input.snapshot,
                    phase,
                    work_unit: parent.work_unit().clone(),
                    output_kind: kind,
                    schema_version: SchemaVersion::new(1),
                    payload: foreign_summary,
                    canonical_payload: Some(bytes.clone()),
                    storage_payload: Some(bytes),
                    decode: BlobDecoder::new(|bytes| {
                        let value = mizar_artifact::store::parse_canonical_json_bytes(
                            bytes,
                            "negative summary",
                        )
                        .map_err(|error| BlobDecodeError::new(error.to_string()))?;
                        read_module_summary(&value, ModuleSummaryReadOptions::default())
                            .map_err(|error| BlobDecodeError::new(error.to_string()))
                    }),
                    parents: vec![parent.clone()],
                    named_input_hashes: parent.lineage().named_input_hashes.clone(),
                    side_tables: IrSideTables::default(),
                    origin: OutputOrigin::PackageSource,
                    target: PublicationTarget::CurrentPackage,
                })
                .unwrap();
            let input = PhaseInput::new(
                old_input.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    old_input.snapshot,
                    old_input.identities().input_hash(),
                    vec![],
                    vec![
                        SealedParentOutputHandle::from_published_output(
                            owner,
                            old_input.snapshot,
                            negative.erase(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            );
            assert_eq!(
                inputs
                    .native_theorem_typed_ast_with_signature_parent(&input, owner)
                    .unwrap_err(),
                "native theorem TypedAst complete summary mismatch"
            );
            let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                snapshot: old_input.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(collected.outputs_dropped, 4);
            assert_eq!(collected.abandoned_slots_dropped, 0);
        },
    );
}

#[test]
fn native_theorem_typed_ast_publication_seals_genuine_payload_and_streams() {
    use mizar_checker::{type_checker::DeclarationChecker, typed_ast::TypingState};
    use mizar_driver::{
        native_theorem_typed_ast_from_storage_bytes as decode,
        native_theorem_typed_ast_storage_bytes as encode,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        source::{FrontendSourceLoader, diagnostic_file_path, source_unit_from_loaded},
    };
    use mizar_ir::{
        identity::NamedInputHash,
        publisher::PublishError,
        storage::{IrSideTables, SchemaVersion, SideTableRecord, StorageError},
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::{
            source_theorem_module_candidates as candidates,
            source_theorem_module_original_inputs as original_inputs,
            source_theorem_module_semantic_payload as semantic_payload,
        },
    };
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let field = |hasher: &mut blake3::Hasher, bytes: &[u8]| {
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    };
    let baseline = |ast: &<MizarParserSeam as ParserSeam>::Ast| {
        let (arena, env) = original_inputs(ast, &module).unwrap().unwrap();
        assert!(candidates(ast, &arena, &env).unwrap().is_some());
        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(ast, &module, &env),
        )
        .unwrap();
        let typed = DeclarationChecker::check_source_theorem_equalities_with_typed_declarations(
            &arena, &env, &scope,
        )
        .unwrap()
        .2;
        let opaque = semantic_payload(ast, &module).unwrap().unwrap().2;
        let mut semantic = b"mizar-driver/native-theorem-typed-ast-content/v1\0".to_vec();
        semantic.extend_from_slice(&(opaque.len() as u64).to_le_bytes());
        semantic.extend_from_slice(&opaque);
        let syntax = ast.canonical_bytes().unwrap();
        let mut bytes = b"mizar-driver/native-theorem-typed-ast-storage/v1\0".to_vec();
        for value in [b"alpha".as_slice(), b"main".as_slice(), syntax.as_slice()] {
            bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
            bytes.extend_from_slice(value);
        }
        assert_eq!(encode(ast, &module, &typed).unwrap(), Some(bytes.clone()));
        (typed, semantic, bytes)
    };
    for open in [false, true] {
        for threshold in [usize::MAX, 0] {
            let mut representations = Vec::new();
            for text in [
                "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
                "theorem T: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
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
                    |fixture, inputs, old, owner, _, (session, ids, snapshots)| {
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
                        let parent = inputs
                            .publish_native_theorem_module_summary(old, owner)
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
                        let (_, loaded, descriptor) =
                            &inputs.captured.unwrap().native_sources().unwrap()[0];
                        let frontend =
                            |native: SourceLoadInputs<'_>,
                             loaded: mizar_session::LoadedSource,
                             descriptor: &SourceInput| {
                                Frontend::new(
                                    FrontendSourceLoader::new(DiskSourceLoader::new(
                                        fixture.root.join("alpha"),
                                    )),
                                    native.dependency_lexical_provider(&[]),
                                    MizarParserSeam,
                                )
                                .run_loaded(source_unit_from_loaded(
                                    loaded,
                                    diagnostic_file_path(descriptor),
                                ))
                                .unwrap()
                            };
                        let output = frontend(inputs, loaded.clone(), descriptor);
                        assert!(output.diagnostics.is_empty());
                        assert!(output.cache_keys.ast.is_some());
                        let ast = output.ast.as_ref().unwrap();
                        let (expected, semantic, bytes) = baseline(ast);
                        assert_eq!(expected.source_id(), loaded.source_id);
                        assert_eq!(expected.module_id().package().as_str(), "alpha");
                        assert_eq!(expected.module_id().path().as_str(), "main");
                        assert!(!expected.types().is_empty());
                        let declarations = expected.source_context().unwrap().declarations();
                        assert!(declarations.iter().next().is_some());
                        for (_, declaration) in declarations.iter() {
                            let node = expected.nodes().node(declaration.site.node()).unwrap();
                            assert_eq!(node.typing, TypingState::Successful);
                            assert!(node.links.type_entry.is_some());
                        }
                        assert_eq!(
                            inputs
                                .native_theorem_typed_ast_with_signature_parent(&input, owner)
                                .unwrap(),
                            Some(expected.clone())
                        );
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
                        let (_, second, descriptor2) =
                            &relocated.captured().native_sources().unwrap()[0];
                        let second_inputs = SourceLoadInputs {
                            snapshot: relocated.captured().snapshot(),
                            captured: Some(relocated.captured()),
                            ..inputs
                        };
                        let second_output = frontend(second_inputs, second.clone(), descriptor2);
                        let second_ast = second_output.ast.as_ref().unwrap();
                        assert_ne!(second.source_id, loaded.source_id);
                        let (second_typed, second_semantic, second_bytes) = baseline(second_ast);
                        assert_ne!(second_typed, expected);
                        assert_eq!(second_semantic, semantic);
                        assert_eq!(second_bytes, bytes);
                        assert_eq!(
                            decode(&bytes, second.source_id, &module).unwrap(),
                            Some(second_typed)
                        );
                        let version = &inputs.snapshot.source_versions[0];
                        let named = vec![NamedInputHash {
                            name: "source".into(),
                            domain: SOURCE_UNIT_CACHE_KEY_VERSION.into(),
                            digest: source_key(version),
                        }];
                        let mut content = blake3::Hasher::new();
                        field(&mut content, b"mizar-ir/publisher-content-hash/v1");
                        field(&mut content, &semantic);
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
                            "mizar-driver/native-theorem-typed-ast-storage-side-table/v1",
                            &bytes,
                        ));
                        let side_tables = IrSideTables {
                            source_maps: vec![SideTableRecord::new(
                                "typecheck-theorem-typed-ast-storage",
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
                                field(&mut side, b"typecheck-theorem-typed-ast-storage");
                                field(&mut side, version.normalized_path.as_str().as_bytes());
                                field(&mut side, b"side_table_digest");
                                side.update(storage_digest.as_bytes());
                            }
                        }
                        let expected_side = Hash::from_bytes(*side.finalize().as_bytes());
                        let original_summary = owner.storage().get(&parent).unwrap();
                        let before = (
                            input.clone(),
                            parent.clone(),
                            (*original_summary).clone(),
                            session.captured().clone(),
                        );
                        let sealed = inputs
                            .publish_native_theorem_typed_ast(&input, owner)
                            .unwrap()
                            .unwrap();
                        assert_eq!(sealed.snapshot(), input.snapshot);
                        assert_eq!(sealed.phase(), &IrPipelinePhase::new("TypeChecking"));
                        assert_eq!(
                            sealed.output_kind(),
                            &OutputKind::new("SourceTheoremTypedAst")
                        );
                        assert_eq!(sealed.work_unit(), &unit);
                        assert_eq!(sealed.schema_version(), SchemaVersion::new(1));
                        assert_eq!(sealed.lineage().parents, vec![parent.output()]);
                        assert_eq!(sealed.lineage().named_input_hashes, named);
                        assert_eq!(sealed.content_hash(), expected_content);
                        assert_eq!(sealed.side_table_hash(), expected_side);
                        assert_eq!(*owner.storage().side_tables(&sealed).unwrap(), side_tables);
                        match sealed.placement() {
                            StoragePlacement::Resident => assert_eq!(threshold, usize::MAX),
                            StoragePlacement::Blob { blob, len } => {
                                assert_eq!(threshold, 0);
                                assert_eq!(*len, bytes.len());
                                assert_eq!(blob.hash(), expected_blob);
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
                            assert_eq!(
                                inputs
                                    .native_theorem_typed_ast_with_signature_parent(&input, owner)
                                    .unwrap(),
                                Some(expected.clone())
                            );
                            let duplicate = PublishError::Storage {
                                error: Box::new(StorageError::OutputAlreadyStored {
                                    output: sealed.output(),
                                }),
                            }
                            .to_string();
                            assert_eq!(
                                inputs
                                    .publish_native_theorem_typed_ast(&input, owner)
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
                                session.captured().clone()
                            ),
                            before
                        );
                        representations.push((semantic, bytes));
                    },
                );
            }
            assert_ne!(representations[0].0, representations[1].0);
            assert_eq!(representations[0].0, representations[2].0);
            assert_ne!(representations[0].0, representations[3].0);
            for other in &representations[1..] {
                assert_ne!(representations[0].1, other.1);
            }
        }
    }
}

#[test]
fn native_theorem_typed_ast_publication_preserves_refusals() {
    use mizar_ir::{publisher::PublishError, storage::CollectInput};
    let lemma = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let unsupported = private.replacen("thus X = X;", "thus X = 0;", 1);
    for case in [
        "unsupported-type",
        "missing",
        "wrong-phase",
        "hash",
        "obsolete",
        "collected",
        "foreign-owner",
        "denied",
    ] {
        with_native_summary_parent(
            if case == "unsupported-type" {
                &unsupported
            } else {
                lemma
            },
            false,
            0,
            |_, inputs, old, owner, _, _| {
                let unit = IrWorkUnit::new("\"alpha\":\"main\"");
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("SignatureCollection"),
                    OutputKind::new("SourceTheoremModuleSummary"),
                    unit.clone(),
                ));
                let parent = inputs
                    .publish_native_theorem_module_summary(old, owner)
                    .unwrap()
                    .unwrap();
                let input = PhaseInput::new(
                    old.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        if case == "hash" {
                            hash(0xf3)
                        } else {
                            old.identities().input_hash()
                        },
                        vec![],
                        if case == "missing" {
                            vec![]
                        } else if case == "wrong-phase" {
                            old.parent_outputs().to_vec()
                        } else {
                            vec![
                                SealedParentOutputHandle::from_published_output(
                                    owner,
                                    old.snapshot,
                                    parent.erase(),
                                )
                                .unwrap(),
                            ]
                        },
                    )
                    .unwrap(),
                );
                if case != "denied" {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("TypeChecking"),
                        OutputKind::new("SourceTheoremTypedAst"),
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
                        4
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
                let raw = inputs.native_theorem_typed_ast_with_signature_parent(&input, selected);
                let actual = inputs
                    .publish_native_theorem_typed_ast(&input, selected)
                    .unwrap_err();
                if case == "denied" {
                    assert!(raw.unwrap().is_some());
                    assert_eq!(
                        actual,
                        PublishError::WorkUnitNotAllowed {
                            phase: IrPipelinePhase::new("TypeChecking"),
                            work_unit: unit,
                            output_kind: OutputKind::new("SourceTheoremTypedAst"),
                        }
                        .to_string()
                    );
                } else {
                    assert_eq!(actual, raw.unwrap_err());
                }
                let collected = owner.storage().collect(CollectInput {
                    snapshot: old.snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(
                    collected.outputs_dropped,
                    if case == "collected" { 0 } else { 4 }
                );
                assert_eq!(
                    collected.abandoned_slots_dropped,
                    usize::from(case == "denied")
                );
            },
        );
    }
}
