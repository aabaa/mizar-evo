use super::*;

#[test]
fn native_theorem_core_caller_preserves_genuine_outputs() {
    use mizar_core::core_ir::{
        CoreCitation, CoreFormulaKind, CoreProofNodeKind, CoreProofStatus, ObligationSeedStatus,
    };
    for text in [
        include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        ),
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
    ] {
        for open in [false, true] {
            for threshold in [usize::MAX, 0] {
                let fixture = Fixture::new(text.as_bytes());
                let ids = InMemorySessionIdAllocator::new();
                let snapshots = SnapshotRegistry::new();
                let (mut draft, submit) = fixture.submit_request(vec![], vec![]);
                draft.source_inputs.versions.clear();
                let descriptor = SourceInput {
                    package_id: fixture.version.package_id.clone(),
                    module_path: fixture.version.module_path.clone(),
                    normalized_path: fixture.version.normalized_path.clone(),
                    edition: fixture.version.edition.clone(),
                    origin: if open {
                        SourceOriginInput::OpenBuffer {
                            uri: format!(
                                "file://{}",
                                fixture.root.join("alpha/src/main.miz").display()
                            ),
                            expected_version: 7,
                            actual_version: 7,
                            text: std::sync::Arc::from(text),
                        }
                    } else {
                        SourceOriginInput::Disk {
                            path: "src/main.miz".into(),
                        }
                    },
                };
                if open {
                    fixture.write(&[0xff]);
                }
                let session = draft
                    .capture_sources(
                        &ids,
                        &snapshots,
                        vec![(
                            DiskSourceLoader::new(fixture.root.join("alpha")),
                            descriptor.clone(),
                        )],
                    )
                    .unwrap();
                let snapshot = session.captured().snapshot().id;
                let module = mizar_build::module_index::ModuleId::new(
                    PackageId::new("alpha"),
                    ModulePath::new("main"),
                );
                let unit = WorkUnit::Module {
                    module: module.clone(),
                };
                let owner = publisher(snapshot, threshold, &unit);
                allow_frontend(&owner, &unit);
                let mut builder = PhaseRegistryBuilder::new();
                builder.register_source_load();
                builder.register_frontend(vec![]);
                let submission = CompilerDriver::new(builder.build().unwrap())
                    .with_output_publisher(owner.clone())
                    .submit_captured(session, &ids, &snapshots, submit)
                    .unwrap();
                let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
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
                let input = PhaseInput::new(
                    unit,
                    PhaseDispatchInputBundle::new(
                        snapshot,
                        source_input.identities().input_hash(),
                        vec![],
                        vec![
                            SealedParentOutputHandle::from_published_output(
                                &owner,
                                snapshot,
                                parent.clone(),
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                );
                let (_, loaded, captured_input) =
                    &inputs.captured.unwrap().native_sources().unwrap()[0];
                let source = mizar_frontend::source::source_unit_from_loaded(
                    loaded.clone(),
                    mizar_frontend::source::diagnostic_file_path(captured_input),
                );
                assert_eq!(source.source_text.as_ref(), text);
                let output = mizar_frontend::orchestration::Frontend::new(
                    mizar_frontend::source::FrontendSourceLoader::new(DiskSourceLoader::new(
                        fixture.root.join("alpha"),
                    )),
                    inputs.dependency_lexical_provider(&[]),
                    MizarParserSeam,
                )
                .run_loaded(source)
                .unwrap();
                assert!(output.diagnostics.is_empty());
                let typed = owner
                    .storage()
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        parent,
                        &OutputKind::new("FrontendOutput"),
                    )
                    .unwrap();
                assert_eq!(*owner.storage().get(&typed).unwrap(), output);
                let ast = output.ast.as_ref().unwrap();
                let resolver_module =
                    ResolverModuleId::new(module.package.clone(), module.path.clone());
                let (arena, env) = mizar_resolve::symbols::source_theorem_module_original_inputs(
                    ast,
                    &resolver_module,
                )
                .unwrap()
                .unwrap();
                let (scope, labels, resolved) =
                    mizar_resolve::symbols::source_proof_inputs(&arena, &env).unwrap();
                let metadata = mizar_checker::type_checker::check_source_theorem_metadata(
                    &arena, &env, &scope,
                )
                .unwrap();
                let check = mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &arena, &metadata.2, &scope, &env, &labels, &resolved, None,
                ).unwrap();
                assert!(!check.owners().is_empty());
                let expected =
                    mizar_core::elaborator::lower_source_theorem_skeletons(&check, None).unwrap();
                assert_eq!(expected.source_id(), loaded.source_id);
                assert_eq!(
                    expected.module_id(),
                    &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"))
                );
                assert_eq!(
                    expected.items().len(),
                    if text.starts_with("lemma") { 2 } else { 1 }
                );
                assert!(!expected.proofs().is_empty());
                assert!(
                    expected
                        .proofs()
                        .iter()
                        .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
                );
                assert!(expected.formulas().iter().any(|(_, formula)| matches!(&formula.kind, CoreFormulaKind::Forall { binders, .. } if !binders.is_empty() && binders.iter().all(|binder| binder.source_name.as_deref() == Some("X")))));
                assert!(expected.proof_nodes().iter().any(|(_, node)| matches!(&node.kind, CoreProofNodeKind::IntroduceBinder { binder, .. } if binder.source_name.as_deref() == Some("X"))));
                assert!(!expected.obligation_seeds().is_empty());
                for (_, seed) in expected.obligation_seeds().iter() {
                    assert_eq!(seed.status, ObligationSeedStatus::Active);
                    assert!(expected.items().get(seed.owner).is_some());
                    assert!(matches!(
                        &expected.formulas().get(seed.goal.unwrap()).unwrap().kind,
                        CoreFormulaKind::Equals { .. }
                    ));
                }
                if text.starts_with("lemma") {
                    assert!(expected.proof_nodes().iter().any(|(_, node)| matches!(&node.kind, CoreProofNodeKind::TerminalGoal { citations, .. } if citations.iter().any(|citation| matches!(citation, CoreCitation::Symbol(_))))));
                }
                let version = &inputs.snapshot.source_versions[0];
                assert_eq!(loaded.source_id, version.source_id);
                assert_eq!(loaded.source_hash, version.source_hash);
                assert_eq!(loaded.source_hash, mizar_session::source::hash_text(text));
                assert_eq!(loaded.origin, version.origin);
                assert_eq!(
                    matches!(loaded.origin, SourceOrigin::OpenBuffer { .. }),
                    open
                );
                let before = (
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    inputs.module_index.clone(),
                    loaded.clone(),
                    captured_input.clone(),
                    input.clone(),
                    parent.clone(),
                );
                for stage in 0..3 {
                    if stage == 1 {
                        fixture.write(b"open theorem Changed: 0 = 1;\n");
                    } else if stage == 2 {
                        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                    }
                    for _ in 0..2 {
                        assert_eq!(
                            inputs
                                .native_theorem_core_with_frontend_parent(&input, &owner)
                                .unwrap(),
                            Some(expected.clone()),
                            "open={open} threshold={threshold} stage={stage}"
                        );
                    }
                    assert_eq!(*owner.storage().get(&typed).unwrap(), output);
                }
                assert_eq!(
                    (
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone(),
                        inputs.module_index.clone(),
                        loaded.clone(),
                        captured_input.clone(),
                        input,
                        parent.clone()
                    ),
                    before
                );
                assert_eq!(captured_input, &descriptor);
            }
        }
    }
}

#[test]
fn native_theorem_core_caller_preserves_refusals() {
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, case) in [
        (format!("{private}\nreserve Y for set;\n"), "reserve"),
        (private.replace("by PrivT1", "by Missing"), "citation"),
        (private.replace("thus X = X;", "thus Y = Y;"), "unbound"),
        (private.replacen("thus X = X;", "thus X = 0;", 1), "checker"),
        (
            "theorem Empty: for A being set holds A = A proof end;".to_owned(),
            "skeleton",
        ),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (_, submission, adapter, handle, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let owner = adapter.publisher();
        let input = PhaseInput::new(
            source_input.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                source_input.snapshot,
                source_input.identities().input_hash(),
                vec![],
                vec![
                    SealedParentOutputHandle::from_published_output(
                        owner,
                        source_input.snapshot,
                        handle.erase(),
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        );
        let retained = owner.storage().get(&handle).unwrap();
        let (_, loaded, descriptor) = &inputs.captured.unwrap().native_sources().unwrap()[0];
        let before = (
            inputs.snapshot.clone(),
            inputs.build_plan.clone(),
            inputs.module_index.clone(),
            loaded.clone(),
            descriptor.clone(),
            input.clone(),
            handle.clone(),
            (*retained).clone(),
        );
        let metadata = inputs.native_theorem_metadata_with_frontend_parent(&input, owner);
        let expected = if case == "skeleton" {
            let metadata = metadata.unwrap().unwrap();
            let WorkUnit::Module { module } = &input.work_unit else {
                unreachable!()
            };
            let resolver_module =
                ResolverModuleId::new(module.package.clone(), module.path.clone());
            let (arena, env) = mizar_resolve::symbols::source_theorem_module_original_inputs(
                retained.ast.as_ref().unwrap(),
                &resolver_module,
            )
            .unwrap()
            .unwrap();
            let (scope, labels, resolved) =
                mizar_resolve::symbols::source_proof_inputs(&arena, &env).unwrap();
            let error = mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                &arena, &metadata.2, &scope, &env, &labels, &resolved, None,
            ).unwrap_err();
            Err(error)
        } else if case == "unbound" || case == "checker" {
            Err(metadata.unwrap_err())
        } else {
            assert_eq!(metadata, Ok(None));
            Ok(None)
        };
        assert_eq!(
            inputs.native_theorem_core_with_frontend_parent(&input, owner),
            expected,
            "{case}"
        );
        assert_eq!(
            (
                inputs.snapshot.clone(),
                inputs.build_plan.clone(),
                inputs.module_index.clone(),
                loaded.clone(),
                descriptor.clone(),
                input,
                handle.clone(),
                (*owner.storage().get(&handle).unwrap()).clone()
            ),
            before
        );
    }
    let text = "theorem Empty: for A being set holds A = A proof end;";
    let fixture = Fixture::new(text.as_bytes());
    let foreign = Fixture::new(text.as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (_, foreign_submission, foreign_adapter, foreign_handle, _) =
        native_disk_frontend_record_submission(&foreign, &ids, &snapshots, 0);
    for case in ["absent", "foreign", "hash", "obsolete", "collected"] {
        let (_, submission, adapter, handle, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let owner = adapter.publisher();
        let mut parents = vec![
            SealedParentOutputHandle::from_published_output(
                owner,
                source_input.snapshot,
                handle.erase(),
            )
            .unwrap(),
        ];
        let mut snapshot = source_input.snapshot;
        let mut hash = source_input.identities().input_hash();
        match case {
            "absent" => parents.clear(),
            "foreign" => {
                snapshot = foreign_submission.session.captured().snapshot().id;
                parents = vec![
                    SealedParentOutputHandle::from_published_output(
                        foreign_adapter.publisher(),
                        snapshot,
                        foreign_handle.erase(),
                    )
                    .unwrap(),
                ];
            }
            "hash" => hash = Hash::from_bytes([0x61; 32]),
            "obsolete" => owner.mark_obsolete(snapshot).unwrap(),
            "collected" => {
                owner.storage().collect(mizar_ir::storage::CollectInput {
                    snapshot,
                    protected_outputs: vec![],
                });
            }
            _ => unreachable!(),
        }
        let input = PhaseInput::new(
            source_input.work_unit.clone(),
            PhaseDispatchInputBundle::new(snapshot, hash, vec![], parents).unwrap(),
        );
        let (_, loaded, descriptor) = &inputs.captured.unwrap().native_sources().unwrap()[0];
        let before = (
            inputs.snapshot.clone(),
            inputs.build_plan.clone(),
            inputs.module_index.clone(),
            loaded.clone(),
            descriptor.clone(),
            input.clone(),
            handle.clone(),
        );
        let expected = inputs
            .native_theorem_metadata_with_frontend_parent(&input, owner)
            .unwrap_err();
        assert_eq!(
            inputs.native_theorem_core_with_frontend_parent(&input, owner),
            Err(expected),
            "{case}"
        );
        assert_eq!(
            (
                inputs.snapshot.clone(),
                inputs.build_plan.clone(),
                inputs.module_index.clone(),
                loaded.clone(),
                descriptor.clone(),
                input,
                handle
            ),
            before
        );
    }
}

#[test]
fn native_theorem_core_storage_replay_preserves_genuine_outputs() {
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_driver::{native_theorem_core_from_storage_bytes, native_theorem_core_storage_bytes};
    use mizar_resolve::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    for threshold in [usize::MAX, 0] {
        let mut distinct_storage = Vec::new();
        for text in [
            include_str!(
                "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
            ),
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            "theorem T: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
            "theorem T: for X being set holds X = X\nproof\n  let X be set;\n  thus X = X;\nend;\n",
        ] {
            let fixture = Fixture::new(text.as_bytes());
            let ids = InMemorySessionIdAllocator::new();
            let snapshots = SnapshotRegistry::new();
            let captures = [
                native_disk_frontend_record_submission(&fixture, &ids, &snapshots, threshold),
                native_disk_frontend_record_submission(&fixture, &ids, &snapshots, threshold),
            ];
            let mut baselines = Vec::new();
            for (_, submission, adapter, handle, _) in &captures {
                let (inputs, _) = native_disk_lookup_context(submission, &ids);
                let retained = adapter.publisher().storage().get(handle).unwrap();
                assert_eq!(
                    handle.erase().placement() == &StoragePlacement::Resident,
                    threshold != 0
                );
                assert!(retained.diagnostics.is_empty());
                let ast = retained.ast.as_ref().unwrap();
                let (_, loaded, _) = &inputs.captured.unwrap().native_sources().unwrap()[0];
                assert_eq!(loaded.text.as_ref(), text);
                assert_eq!(ast.source_id, loaded.source_id);
                let (arena, env) =
                    mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &module)
                        .unwrap()
                        .unwrap();
                assert!(
                    mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, &env,)
                        .unwrap()
                        .is_some()
                );
                let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                    SourceVariableScopeInput::new(ast, &module, &env),
                )
                .unwrap();
                let metadata = mizar_checker::type_checker::check_source_theorem_metadata(
                    &arena, &env, &scope,
                )
                .unwrap();
                let (scope, labels, resolved) =
                    mizar_resolve::symbols::source_proof_inputs(&arena, &env).unwrap();
                let check = mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &arena, &metadata.2, &scope, &env, &labels, &resolved, None,
                ).unwrap();
                assert!(!check.owners().is_empty());
                let core =
                    mizar_core::elaborator::lower_source_theorem_skeletons(&check, None).unwrap();
                assert_eq!(core.source_id(), loaded.source_id);
                assert_eq!(core.module_id(), &module);
                assert!(!core.proofs().is_empty());
                assert!(
                    core.proofs()
                        .iter()
                        .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
                );
                assert!(!core.obligation_seeds().is_empty());
                assert!(
                    core.obligation_seeds()
                        .iter()
                        .all(|(_, seed)| seed.status == ObligationSeedStatus::Active)
                );
                let syntax = ast.canonical_bytes().unwrap();
                let mut expected_bytes = b"mizar-driver/native-theorem-core-storage/v1\0".to_vec();
                for field in [b"alpha".as_slice(), b"main".as_slice(), syntax.as_slice()] {
                    expected_bytes
                        .extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
                    expected_bytes.extend_from_slice(field);
                }
                baselines.push((core, expected_bytes, ast.clone()));
            }
            assert_ne!(baselines[0].0.source_id(), baselines[1].0.source_id());
            assert_eq!(baselines[0].1, baselines[1].1);
            assert!(!distinct_storage.contains(&baselines[0].1));
            distinct_storage.push(baselines[0].1.clone());
            for stage in 0..3 {
                if stage == 1 {
                    fixture.write(b"open theorem Changed: 0 = 1;\n");
                } else if stage == 2 {
                    fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                }
                for (core, bytes, ast) in &baselines {
                    let before = (ast.clone(), module.clone(), core.clone(), bytes.clone());
                    for _ in 0..2 {
                        assert_eq!(
                            native_theorem_core_storage_bytes(ast, &module, core).unwrap(),
                            Some(bytes.clone())
                        );
                        assert_eq!(
                            native_theorem_core_from_storage_bytes(bytes, ast.source_id, &module)
                                .unwrap(),
                            Some(core.clone())
                        );
                        assert_eq!(
                            native_theorem_core_from_storage_bytes(
                                bytes,
                                baselines[1].0.source_id(),
                                &module
                            )
                            .unwrap(),
                            Some(baselines[1].0.clone())
                        );
                    }
                    assert_eq!(
                        native_theorem_core_storage_bytes(
                            ast,
                            &module,
                            &baselines
                                [1 - usize::from(ast.source_id == baselines[1].0.source_id())]
                            .0
                        )
                        .unwrap(),
                        None
                    );
                    assert_eq!(
                        (ast.clone(), module.clone(), core.clone(), bytes.clone()),
                        before
                    );
                }
            }
        }
    }
}

#[test]
fn native_theorem_core_storage_replay_preserves_refusals() {
    use mizar_driver::{native_theorem_core_from_storage_bytes, native_theorem_core_storage_bytes};
    use mizar_resolve::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let valid = Fixture::new(
        b"theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
    );
    let (_, submission, adapter, handle, _) =
        native_disk_frontend_record_submission(&valid, &ids, &snapshots, 0);
    let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
    let owner = adapter.publisher();
    let input = PhaseInput::new(
        source_input.work_unit.clone(),
        PhaseDispatchInputBundle::new(
            source_input.snapshot,
            source_input.identities().input_hash(),
            vec![],
            vec![
                SealedParentOutputHandle::from_published_output(
                    owner,
                    source_input.snapshot,
                    handle.erase(),
                )
                .unwrap(),
            ],
        )
        .unwrap(),
    );
    let foreign_core = inputs
        .native_theorem_core_with_frontend_parent(&input, owner)
        .unwrap()
        .unwrap();
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
        (
            "theorem Empty: for A being set holds A = A proof end;".to_owned(),
            "empty",
        ),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let (_, _, adapter, handle, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let output = adapter.publisher().storage().get(&handle).unwrap();
        let ast = output.ast.as_ref().unwrap();
        let (arena, env) =
            mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &module)
                .unwrap()
                .unwrap();
        let candidate = mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, &env);
        let expected: Result<Option<mizar_core::core_ir::CoreIr>, String> = match candidate {
            Ok(None) => {
                assert!(matches!(case, "reserve" | "citation"));
                Ok(None)
            }
            Err(error) => {
                assert_eq!(case, "unbound");
                Err(error)
            }
            Ok(Some(_)) => {
                let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                    SourceVariableScopeInput::new(ast, &module, &env),
                )
                .unwrap();
                let metadata = mizar_checker::type_checker::check_source_theorem_metadata(
                    &arena, &env, &scope,
                );
                if case == "unsupported-type" {
                    Err(metadata.unwrap_err())
                } else {
                    assert_eq!(case, "empty");
                    let metadata = metadata.unwrap();
                    let (scope, labels, resolved) =
                        mizar_resolve::symbols::source_proof_inputs(&arena, &env).unwrap();
                    let error = mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                        &arena, &metadata.2, &scope, &env, &labels, &resolved, None,
                    ).unwrap_err();
                    Err(error)
                }
            }
        };
        let syntax = ast.canonical_bytes().unwrap();
        let mut bytes = b"mizar-driver/native-theorem-core-storage/v1\0".to_vec();
        for field in [b"alpha".as_slice(), b"main".as_slice(), syntax.as_slice()] {
            bytes.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
            bytes.extend_from_slice(field);
        }
        let before = (
            ast.clone(),
            module.clone(),
            foreign_core.clone(),
            bytes.clone(),
        );
        assert_eq!(
            native_theorem_core_from_storage_bytes(&bytes, ast.source_id, &module),
            expected,
            "{case}"
        );
        assert_eq!(
            native_theorem_core_storage_bytes(ast, &module, &foreign_core),
            expected.map(|_| None::<Vec<u8>>),
            "{case}"
        );
        assert_eq!(
            (ast.clone(), module.clone(), foreign_core.clone(), bytes),
            before
        );
    }
    let output = owner.storage().get(&handle).unwrap();
    let ast = output.ast.as_ref().unwrap();
    let syntax = ast.canonical_bytes().unwrap();
    let domain = b"mizar-driver/native-theorem-core-storage/v1\0";
    let mut bytes = domain.to_vec();
    for field in [b"alpha".as_slice(), b"main".as_slice(), syntax.as_slice()] {
        bytes.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
        bytes.extend_from_slice(field);
    }
    let mut corruptions = vec![
        Vec::new(),
        bytes[..domain.len() + 3].to_vec(),
        bytes[..bytes.len() - 1].to_vec(),
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
    let mut bad_syntax = bytes.clone();
    bad_syntax[bytes.len() - syntax.len()] = b'!';
    corruptions.push(bad_syntax);
    let mut oversized = bytes.clone();
    oversized.resize(32 * 1024 * 1024 + 1, 0);
    corruptions.push(oversized);
    for malformed in corruptions {
        assert_eq!(
            native_theorem_core_from_storage_bytes(&malformed, ast.source_id, &module),
            Ok(None)
        );
    }
    for wrong_module in [
        ResolverModuleId::new(PackageId::new("foreign"), ModulePath::new("main")),
        ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("other")),
    ] {
        assert_eq!(
            native_theorem_core_from_storage_bytes(&bytes, ast.source_id, &wrong_module),
            Ok(None)
        );
        assert_eq!(
            native_theorem_core_storage_bytes(ast, &wrong_module, &foreign_core),
            Ok(None)
        );
    }
    let oversized_module = ResolverModuleId::new(
        PackageId::new("x".repeat(32 * 1024 * 1024)),
        ModulePath::new("main"),
    );
    assert_eq!(
        native_theorem_core_storage_bytes(ast, &oversized_module, &foreign_core),
        Ok(None)
    );
}

#[test]
fn native_theorem_core_semantic_payload_preserves_genuine_outputs() {
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_driver::{native_theorem_core_semantic_payload, native_theorem_core_storage_bytes};
    use mizar_resolve::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    for threshold in [usize::MAX, 0] {
        let mut contents = Vec::new();
        let mut storage = Vec::new();
        for text in [
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            "theorem T: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
            "theorem T: for X being set holds X = X\nproof\n  let X be set;\n  thus X = X;\nend;\n",
            "theorem Renamed: for X being set holds X = X proof let X be set; thus X = X; end;",
        ] {
            let fixture = Fixture::new(text.as_bytes());
            let ids = InMemorySessionIdAllocator::new();
            let snapshots = SnapshotRegistry::new();
            let captures = [
                native_disk_frontend_record_submission(&fixture, &ids, &snapshots, threshold),
                native_disk_frontend_record_submission(&fixture, &ids, &snapshots, threshold),
            ];
            let mut baselines = Vec::new();
            for (_, submission, adapter, handle, _) in &captures {
                let (inputs, _) = native_disk_lookup_context(submission, &ids);
                let output = adapter.publisher().storage().get(handle).unwrap();
                assert_eq!(
                    handle.erase().placement() == &StoragePlacement::Resident,
                    threshold != 0
                );
                assert!(output.diagnostics.is_empty());
                let ast = output.ast.as_ref().unwrap();
                let (_, loaded, _) = &inputs.captured.unwrap().native_sources().unwrap()[0];
                assert_eq!(loaded.text.as_ref(), text);
                assert_eq!(ast.source_id, loaded.source_id);
                let (arena, env) =
                    mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &module)
                        .unwrap()
                        .unwrap();
                assert!(
                    mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, &env)
                        .unwrap()
                        .is_some()
                );
                let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                    SourceVariableScopeInput::new(ast, &module, &env),
                )
                .unwrap();
                let metadata = mizar_checker::type_checker::check_source_theorem_metadata(
                    &arena, &env, &scope,
                )
                .unwrap();
                let (scope, labels, resolved) =
                    mizar_resolve::symbols::source_proof_inputs(&arena, &env).unwrap();
                let check = mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &arena, &metadata.2, &scope, &env, &labels, &resolved, None,
                ).unwrap();
                assert!(!check.owners().is_empty());
                let core =
                    mizar_core::elaborator::lower_source_theorem_skeletons(&check, None).unwrap();
                assert_eq!(core.source_id(), loaded.source_id);
                assert_eq!(core.module_id(), &module);
                assert!(!core.proofs().is_empty());
                assert!(
                    core.proofs()
                        .iter()
                        .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
                );
                assert!(!core.obligation_seeds().is_empty());
                assert!(
                    core.obligation_seeds()
                        .iter()
                        .all(|(_, seed)| seed.status == ObligationSeedStatus::Active)
                );
                let bytes =
                    mizar_core::elaborator::source_theorem_core_semantic_payload(&check, &core)
                        .unwrap();
                let source_storage = native_theorem_core_storage_bytes(ast, &module, &core)
                    .unwrap()
                    .unwrap();
                baselines.push((ast.clone(), core, bytes, source_storage));
            }
            assert_ne!(baselines[0].0.source_id, baselines[1].0.source_id);
            assert_ne!(baselines[0].1, baselines[1].1);
            assert_eq!(baselines[0].2, baselines[1].2);
            assert_eq!(baselines[0].3, baselines[1].3);
            assert!(!storage.contains(&baselines[0].3));
            storage.push(baselines[0].3.clone());
            contents.push(baselines[0].2.clone());
            for stage in 0..3 {
                if stage == 1 {
                    fixture.write(b"open theorem Changed: 0 = 1;\n");
                } else if stage == 2 {
                    fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                }
                for (ast, core, bytes, _) in &baselines {
                    let before = (ast.clone(), module.clone(), core.clone());
                    for _ in 0..2 {
                        assert_eq!(
                            native_theorem_core_semantic_payload(ast, &module, core),
                            Ok(Some(bytes.clone()))
                        );
                    }
                    assert_eq!((ast.clone(), module.clone(), core.clone()), before);
                }
            }
        }
        assert_eq!(contents[0], contents[1]);
        assert_eq!(contents[0], contents[2]);
        assert_ne!(contents[0], contents[3]);
    }
}

#[test]
fn native_theorem_core_semantic_payload_preserves_refusals() {
    use mizar_driver::native_theorem_core_semantic_payload;
    use mizar_resolve::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let closed = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut foreign_core = None;
    'cases: for (text, case) in [
        (closed.to_owned(), "foreign-seed"),
        (closed.to_owned(), "foreign"),
        (format!("{private}\nreserve Y for set;\n"), "reserve"),
        (private.replace("by PrivT1", "by Missing"), "citation"),
        (private.replace("thus X = X;", "thus Y = Y;"), "unbound"),
        (private.replacen("thus X = X;", "thus X = 0;", 1), "unsupported-type"),
        (
            "theorem Empty: for A being set holds A = A proof end;".to_owned(),
            "empty",
        ),
        (
            include_str!(
                "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
            )
            .to_owned(),
            "lemma",
        ),
        (closed.replace("set", "object"), "object"),
        (
            "open theorem T: for X being set holds X = X;".to_owned(),
            "open",
        ),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let (_, _, adapter, handle, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let output = adapter.publisher().storage().get(&handle).unwrap();
        assert!(output.diagnostics.is_empty(), "{case}");
        let ast = output.ast.as_ref().unwrap();
        let mut refusal_owner = "original";
        let expected = 'baseline: {
            let (arena, env) =
                match mizar_resolve::symbols::source_theorem_module_original_inputs(ast, &module) {
                    Ok(Some(pair)) => pair,
                    Ok(None) => break 'baseline Ok(None),
                    Err(error) => break 'baseline Err(error),
                };
            refusal_owner = "candidate";
            match mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, &env) {
                Ok(Some(_)) => {}
                Ok(None) => break 'baseline Ok(None),
                Err(error) => break 'baseline Err(error),
            }
            refusal_owner = "scope";
            let scope = match SourceVariableScopeResolver::resolve_proof_occurrences(
                SourceVariableScopeInput::new(ast, &module, &env),
            ) {
                Ok(scope) => scope,
                Err(error) => break 'baseline Err(format!("{error:?}")),
            };
            refusal_owner = "metadata";
            let metadata = match mizar_checker::type_checker::check_source_theorem_metadata(
                &arena, &env, &scope,
            ) {
                Ok(metadata) => metadata,
                Err(error) => break 'baseline Err(error),
            };
            refusal_owner = "proof inputs";
            let (scope, labels, resolved) =
                match mizar_resolve::symbols::source_proof_inputs(&arena, &env) {
                    Ok(inputs) => inputs,
                    Err(error) => break 'baseline Err(error),
                };
            refusal_owner = "skeleton";
            let check = match mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(
                &arena, &metadata.2, &scope, &env, &labels, &resolved, None,
            ) {
                Ok(check) => check,
                Err(error) => break 'baseline Err(error),
            };
            refusal_owner = "lowering";
            let core = match mizar_core::elaborator::lower_source_theorem_skeletons(&check, None) {
                Ok(core) => core,
                Err(error) => break 'baseline Err(error),
            };
            if case == "foreign-seed" {
                foreign_core = Some(core);
                continue 'cases;
            }
            refusal_owner = "semantic";
            let supplied = if case == "foreign" {
                foreign_core.as_ref().unwrap()
            } else {
                &core
            };
            let expected =
                mizar_core::elaborator::source_theorem_core_semantic_payload(&check, supplied)
                    .map(Some);
            assert!(expected.is_err(), "{case}");
            let before = (ast.clone(), module.clone(), supplied.clone());
            assert_eq!(
                native_theorem_core_semantic_payload(ast, &module, supplied),
                expected,
                "{case} at {refusal_owner}"
            );
            assert_eq!((ast.clone(), module.clone(), supplied.clone()), before);
            continue 'cases;
        };
        match case {
            "reserve" | "citation" => {
                assert_eq!(refusal_owner, "candidate");
                assert_eq!(expected, Ok(None));
            }
            "unbound" => {
                assert_eq!(refusal_owner, "candidate");
                assert!(expected.is_err());
            }
            "unsupported-type" => {
                assert_eq!(refusal_owner, "metadata");
                assert!(expected.is_err());
            }
            "empty" => {
                assert_eq!(refusal_owner, "skeleton");
                assert!(expected.is_err());
            }
            "object" | "open" | "lemma" => {
                assert!(expected.is_err() || expected == Ok(None));
            }
            _ => panic!("unexpected {case} refusal at {refusal_owner}"),
        }
        let supplied = foreign_core.as_ref().unwrap();
        let before = (ast.clone(), module.clone(), supplied.clone());
        assert_eq!(
            native_theorem_core_semantic_payload(ast, &module, supplied),
            expected,
            "{case} at {refusal_owner}"
        );
        assert_eq!((ast.clone(), module.clone(), supplied.clone()), before);
    }
}

#[test]
fn native_theorem_core_publication_seals_genuine_core_and_streams() {
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_driver::{
        native_theorem_core_from_storage_bytes, native_theorem_core_semantic_payload,
        native_theorem_core_storage_bytes,
    };
    for text in [
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
        "theorem T: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
        "theorem T: for X being set holds X = X\nproof\n  let X be set;\n  thus X = X;\nend;\n",
    ] {
        for open in [false, true] {
            for threshold in [usize::MAX, 0] {
                let fixture = Fixture::new(text.as_bytes());
                let ids = InMemorySessionIdAllocator::new();
                let snapshots = SnapshotRegistry::new();
                let (mut draft, submit) = fixture.submit_request(vec![], vec![]);
                draft.source_inputs.versions.clear();
                let descriptor = SourceInput {
                    package_id: fixture.version.package_id.clone(),
                    module_path: fixture.version.module_path.clone(),
                    normalized_path: fixture.version.normalized_path.clone(),
                    edition: fixture.version.edition.clone(),
                    origin: if open {
                        SourceOriginInput::OpenBuffer {
                            uri: format!(
                                "file://{}",
                                fixture.root.join("alpha/src/main.miz").display()
                            ),
                            expected_version: 7,
                            actual_version: 7,
                            text: std::sync::Arc::from(text),
                        }
                    } else {
                        SourceOriginInput::Disk {
                            path: "src/main.miz".into(),
                        }
                    },
                };
                if open {
                    fixture.write(&[0xff]);
                }
                let session = draft
                    .capture_sources(
                        &ids,
                        &snapshots,
                        vec![(
                            DiskSourceLoader::new(fixture.root.join("alpha")),
                            descriptor.clone(),
                        )],
                    )
                    .unwrap();
                let snapshot = session.captured().snapshot().id;
                let unit = WorkUnit::Module {
                    module: mizar_build::module_index::ModuleId::new(
                        PackageId::new("alpha"),
                        ModulePath::new("main"),
                    ),
                };
                let ir_unit = IrWorkUnit::new("\"alpha\":\"main\"");
                let owner = publisher(snapshot, threshold, &unit);
                allow_frontend(&owner, &unit);
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("Elaboration"),
                    OutputKind::new("SourceTheoremCore"),
                    ir_unit.clone(),
                ));
                let mut builder = PhaseRegistryBuilder::new();
                builder.register_source_load();
                builder.register_frontend(vec![]);
                let submission = CompilerDriver::new(builder.build().unwrap())
                    .with_output_publisher(owner.clone())
                    .submit_captured(session, &ids, &snapshots, submit)
                    .unwrap();
                let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
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
                let input = PhaseInput::new(
                    unit,
                    PhaseDispatchInputBundle::new(
                        snapshot,
                        source_input.identities().input_hash(),
                        vec![],
                        vec![
                            SealedParentOutputHandle::from_published_output(
                                &owner,
                                snapshot,
                                parent.clone(),
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                );
                let expected = inputs
                    .native_theorem_core_with_frontend_parent(&input, &owner)
                    .unwrap()
                    .unwrap();
                let typed = owner
                    .storage()
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        parent,
                        &OutputKind::new("FrontendOutput"),
                    )
                    .unwrap();
                let retained = owner.storage().get(&typed).unwrap();
                assert!(retained.diagnostics.is_empty());
                let ast = retained.ast.as_ref().unwrap();
                let module =
                    ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                let version = &inputs.snapshot.source_versions[0];
                let (_, loaded, captured_input) =
                    &inputs.captured.unwrap().native_sources().unwrap()[0];
                assert_eq!(loaded.text.as_ref(), text);
                assert_eq!(ast.source_id, loaded.source_id);
                assert_eq!(expected.source_id(), loaded.source_id);
                assert_eq!(expected.module_id(), &module);
                assert!(!expected.proofs().is_empty());
                assert!(
                    expected
                        .proofs()
                        .iter()
                        .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
                );
                assert!(!expected.obligation_seeds().is_empty());
                assert!(
                    expected
                        .obligation_seeds()
                        .iter()
                        .all(|(_, seed)| seed.status == ObligationSeedStatus::Active)
                );
                let semantic = native_theorem_core_semantic_payload(ast, &module, &expected)
                    .unwrap()
                    .unwrap();
                let bytes = native_theorem_core_storage_bytes(ast, &module, &expected)
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    native_theorem_core_from_storage_bytes(&bytes, loaded.source_id, &module)
                        .unwrap(),
                    Some(expected.clone())
                );
                assert_ne!(semantic, bytes);
                let mut hasher = blake3::Hasher::new();
                for field in [
                    b"mizar-ir/publisher-content-hash/v1".as_slice(),
                    semantic.as_slice(),
                ] {
                    hasher.update(&(field.len() as u64).to_le_bytes());
                    hasher.update(field);
                }
                hasher.update(&1_u64.to_le_bytes());
                hasher.update(&("parent_content".len() as u64).to_le_bytes());
                hasher.update(b"parent_content");
                hasher.update(parent.content_hash().as_bytes());
                hasher.update(&1_u64.to_le_bytes());
                for field in [
                    "source",
                    SOURCE_UNIT_CACHE_KEY_VERSION,
                    "mizar-ir/publisher-named-input-hash/v1",
                ] {
                    hasher.update(&(field.len() as u64).to_le_bytes());
                    hasher.update(field.as_bytes());
                }
                hasher.update(source_key(version).as_bytes());
                let expected_content = Hash::from_bytes(*hasher.finalize().as_bytes());
                let mut blob_hasher = blake3::Hasher::new();
                let domain = b"mizar-ir/content-blob/v1";
                blob_hasher.update(&(domain.len() as u64).to_le_bytes());
                blob_hasher.update(domain);
                blob_hasher.update(&1_u32.to_le_bytes());
                blob_hasher.update(&(bytes.len() as u64).to_le_bytes());
                blob_hasher.update(&bytes);
                let expected_blob = Hash::from_bytes(*blob_hasher.finalize().as_bytes());
                let before = (
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    inputs.module_index.clone(),
                    loaded.clone(),
                    captured_input.clone(),
                    (*retained).clone(),
                    input.clone(),
                    parent.clone(),
                );
                let sealed = inputs
                    .publish_native_theorem_core(&input, &owner)
                    .unwrap()
                    .unwrap();
                let original_handle = sealed.erase();
                for stage in 0..3 {
                    if stage == 1 {
                        fixture.write(b"open theorem Changed: 0 = 1;\n");
                    } else if stage == 2 {
                        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                    }
                    assert_eq!(
                        inputs
                            .native_theorem_core_with_frontend_parent(&input, &owner)
                            .unwrap(),
                        Some(expected.clone())
                    );
                    assert_eq!(
                        native_theorem_core_semantic_payload(ast, &module, &expected).unwrap(),
                        Some(semantic.clone())
                    );
                    assert_eq!(
                        native_theorem_core_storage_bytes(ast, &module, &expected).unwrap(),
                        Some(bytes.clone())
                    );
                    assert_eq!(
                        native_theorem_core_from_storage_bytes(&bytes, loaded.source_id, &module)
                            .unwrap(),
                        Some(expected.clone())
                    );
                    let duplicate = mizar_ir::publisher::PublishError::Storage {
                        error: Box::new(mizar_ir::storage::StorageError::OutputAlreadyStored {
                            output: sealed.output(),
                        }),
                    }
                    .to_string();
                    assert_eq!(
                        inputs
                            .publish_native_theorem_core(&input, &owner)
                            .unwrap_err(),
                        duplicate
                    );
                    assert_eq!(sealed.any(), &original_handle);
                    assert_eq!(*owner.storage().get(&sealed).unwrap(), expected);
                    assert_eq!(sealed.snapshot(), snapshot);
                    assert_eq!(sealed.phase(), &IrPipelinePhase::new("Elaboration"));
                    assert_eq!(sealed.output_kind(), &OutputKind::new("SourceTheoremCore"));
                    assert_eq!(sealed.work_unit(), &ir_unit);
                    assert_eq!(
                        sealed.schema_version(),
                        mizar_ir::storage::SchemaVersion::new(1)
                    );
                    assert_eq!(sealed.lineage().parents, vec![parent.output()]);
                    assert_eq!(
                        sealed.lineage().named_input_hashes,
                        vec![mizar_ir::identity::NamedInputHash {
                            name: "source".to_owned(),
                            domain: SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                            digest: source_key(version),
                        }]
                    );
                    assert_eq!(sealed.content_hash(), expected_content);
                    match sealed.placement() {
                        StoragePlacement::Resident => assert_eq!(threshold, usize::MAX),
                        StoragePlacement::Blob { blob, len } => {
                            assert_eq!(threshold, 0);
                            assert_eq!(*len, bytes.len());
                            assert_eq!(blob.hash(), expected_blob);
                        }
                        _ => panic!("unexpected storage placement"),
                    }
                    assert_eq!(
                        *owner.storage().side_tables(&sealed).unwrap(),
                        mizar_ir::storage::IrSideTables {
                            source_maps: vec![mizar_ir::storage::SideTableRecord::new(
                                "elaborate-theorem-core-storage",
                                version.normalized_path.as_str(),
                                Hash::from_bytes(blake3::derive_key(
                                    "mizar-driver/native-theorem-core-storage-side-table/v1",
                                    &bytes
                                )),
                            )],
                            ..mizar_ir::storage::IrSideTables::default()
                        }
                    );
                    assert!(
                        owner
                            .validate_published_output(snapshot, sealed.any())
                            .is_ok()
                    );
                    assert_eq!(
                        owner
                            .validate_current_output(snapshot, sealed.any())
                            .is_ok(),
                        !open
                    );
                }
                assert_eq!(
                    (
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone(),
                        inputs.module_index.clone(),
                        loaded.clone(),
                        captured_input.clone(),
                        (*owner.storage().get(&typed).unwrap()).clone(),
                        input.clone(),
                        parent.clone()
                    ),
                    before
                );
                assert_eq!(captured_input, &descriptor);
            }
        }
    }
}

#[test]
fn native_theorem_core_publication_preserves_refusals_and_publisher_boundaries() {
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, case) in [
        (format!("{private}\nreserve Y for set;\n"), "reserve"),
        (private.replacen("thus X = X;", "thus X = 0;", 1), "metadata"),
        (
            "theorem Empty: for A being set holds A = A proof end;".to_owned(),
            "empty",
        ),
        (
            include_str!(
                "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
            )
            .to_owned(),
            "lemma",
        ),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (_, submission, adapter, handle, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let owner = adapter.publisher();
        owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new("Elaboration"),
            OutputKind::new("SourceTheoremCore"),
            IrWorkUnit::new("\"alpha\":\"main\""),
        ));
        let input = PhaseInput::new(
            source_input.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                source_input.snapshot,
                source_input.identities().input_hash(),
                vec![],
                vec![
                    SealedParentOutputHandle::from_published_output(
                        owner,
                        source_input.snapshot,
                        handle.erase(),
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        );
        let old = inputs.native_theorem_core_with_frontend_parent(&input, owner);
        let expected = match case {
            "reserve" => {
                assert_eq!(old, Ok(None));
                Ok(None)
            }
            "metadata" => {
                assert_eq!(
                    old,
                    Err(inputs
                        .native_theorem_metadata_with_frontend_parent(&input, owner)
                        .unwrap_err())
                );
                Err(old.unwrap_err())
            }
            "empty" => {
                assert!(
                    inputs
                        .native_theorem_metadata_with_frontend_parent(&input, owner)
                        .unwrap()
                        .is_some()
                );
                Err(old.unwrap_err())
            }
            "lemma" => {
                let core = old.unwrap().unwrap();
                let retained = owner.storage().get(&handle).unwrap();
                let module =
                    ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                Err(mizar_driver::native_theorem_core_semantic_payload(
                    retained.ast.as_ref().unwrap(),
                    &module,
                    &core,
                )
                .unwrap_err())
            }
            _ => unreachable!(),
        };
        let actual = inputs
            .publish_native_theorem_core(&input, owner)
            .map(|output| output.map(|output| (*owner.storage().get(&output).unwrap()).clone()));
        assert_eq!(actual, expected, "{case}");
        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
            snapshot: source_input.snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(collected.outputs_dropped, 2, "{case}");
        assert_eq!(collected.abandoned_slots_dropped, 0, "{case}");
    }
    for case in [
        "absent",
        "obsolete",
        "collected",
        "denied-kind",
        "denied-phase",
    ] {
        let fixture = Fixture::new(
            b"theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
        );
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (_, submission, adapter, handle, _) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let owner = adapter.publisher();
        let snapshot = source_input.snapshot;
        owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new(if case == "denied-phase" {
                "Frontend"
            } else {
                "Elaboration"
            }),
            OutputKind::new(if case == "denied-kind" {
                "WrongKind"
            } else {
                "SourceTheoremCore"
            }),
            IrWorkUnit::new("\"alpha\":\"main\""),
        ));
        let mut parents = vec![
            SealedParentOutputHandle::from_published_output(owner, snapshot, handle.erase())
                .unwrap(),
        ];
        match case {
            "absent" => parents.clear(),
            "obsolete" => owner.mark_obsolete(snapshot).unwrap(),
            "collected" => {
                owner.storage().collect(mizar_ir::storage::CollectInput {
                    snapshot,
                    protected_outputs: vec![],
                });
            }
            _ => {}
        }
        let input = PhaseInput::new(
            source_input.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                snapshot,
                source_input.identities().input_hash(),
                vec![],
                parents,
            )
            .unwrap(),
        );
        let old = inputs.native_theorem_core_with_frontend_parent(&input, owner);
        let actual = inputs
            .publish_native_theorem_core(&input, owner)
            .unwrap_err();
        if case.starts_with("denied-") {
            assert!(old.unwrap().is_some());
            assert_eq!(
                actual,
                mizar_ir::publisher::PublishError::WorkUnitNotAllowed {
                    phase: IrPipelinePhase::new("Elaboration"),
                    work_unit: IrWorkUnit::new("\"alpha\":\"main\""),
                    output_kind: OutputKind::new("SourceTheoremCore"),
                }
                .to_string()
            );
        } else {
            assert_eq!(actual, old.unwrap_err(), "{case}");
        }
        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
            snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(
            collected.outputs_dropped,
            if case == "collected" { 0 } else { 2 }
        );
    }
}

#[test]
fn native_theorem_core_metadata_caller_preserves_genuine_outputs() {
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker, check_source_theorem_metadata as metadata,
    };
    use mizar_core::core_ir::{
        CoreCitation, CoreFormulaKind, CoreProofNodeKind, CoreProofStatus, ObligationSeedStatus,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        source::{FrontendSourceLoader, diagnostic_file_path, source_unit_from_loaded},
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput as Input, SourceVariableScopeResolver as Resolver},
        symbols::{source_proof_inputs, source_theorem_module_original_inputs as original_inputs},
    };
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    for text in [
        include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        ),
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
    ] {
        for (open, threshold) in [(false, usize::MAX), (true, 0)] {
            with_native_summary_parent(
                text,
                open,
                threshold,
                |fixture, inputs, old, owner, _, (session, _, _)| {
                    for (phase, kind) in [
                        ("SignatureCollection", "SourceTheoremModuleSummary"),
                        ("TypeChecking", "SourceTheoremTypedAst"),
                        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                        ("OverloadResolution", "SourceTheoremMetadata"),
                    ] {
                        owner.allow_work_unit(AllowedWorkUnit::new(
                            IrPipelinePhase::new(phase),
                            OutputKind::new(kind),
                            IrWorkUnit::new("\"alpha\":\"main\""),
                        ));
                    }
                    let child = |parents: Vec<AnyPhaseOutputRef>| {
                        PhaseInput::new(
                            old.work_unit.clone(),
                            PhaseDispatchInputBundle::new(
                                old.snapshot,
                                old.identities().input_hash(),
                                vec![],
                                parents
                                    .into_iter()
                                    .map(|parent| {
                                        SealedParentOutputHandle::from_published_output(
                                            owner,
                                            old.snapshot,
                                            parent,
                                        )
                                        .unwrap()
                                    })
                                    .collect(),
                            )
                            .unwrap(),
                        )
                    };
                    let signature = inputs
                        .publish_native_theorem_module_summary(old, owner)
                        .unwrap()
                        .unwrap();
                    let type_input = child(vec![signature.erase()]);
                    let typed = inputs
                        .publish_native_theorem_typed_ast(&type_input, owner)
                        .unwrap()
                        .unwrap();
                    let reg_input = child(vec![typed.erase()]);
                    let registration = inputs
                        .publish_native_theorem_registration_closure(&reg_input, owner)
                        .unwrap()
                        .unwrap();
                    let meta_input = child(vec![typed.erase(), registration.erase()]);
                    let parent = inputs
                        .publish_native_theorem_metadata(&meta_input, owner)
                        .unwrap()
                        .unwrap();
                    let input = child(vec![parent.erase()]);
                    assert_eq!(parent.placement() == &StoragePlacement::Resident, !open);
                    let retained = owner.storage().get(&parent).unwrap();
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
                    let ast = output.ast.as_ref().unwrap();
                    let (arena, env) = original_inputs(ast, &module).unwrap().unwrap();
                    let scope = Resolver::resolve_proof_occurrences(Input::new(ast, &module, &env))
                        .unwrap();
                    let independent = metadata(&arena, &env, &scope).unwrap();
                    assert_eq!(*retained, independent);
                    assert!(!independent.1.normalized_types().is_empty());
                    assert!(
                        !independent
                            .2
                            .source_atomic_formula()
                            .unwrap()
                            .requests()
                            .is_empty()
                    );
                    let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
                    let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                        &arena,
                        &independent.2,
                        &scope,
                        &env,
                        &labels,
                        &resolved,
                        None,
                    )
                    .unwrap();
                    assert!(!check.owners().is_empty());
                    let expected =
                        mizar_core::elaborator::lower_source_theorem_skeletons(&check, None)
                            .unwrap();
                    assert_eq!(expected.source_id(), loaded.source_id);
                    assert_eq!(expected.module_id(), &module);
                    assert!(!expected.items().is_empty());
                    assert!(!expected.proofs().is_empty());
                    assert!(
                        expected.proofs().iter().all(
                            |(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof
                        )
                    );
                    assert!(expected.formulas().iter().any(|(_, formula)| matches!(&formula.kind, CoreFormulaKind::Forall { binders, .. } if !binders.is_empty() && binders.iter().all(|binder| binder.source_name.as_deref() == Some("X")))));
                    assert!(expected.proof_nodes().iter().any(|(_, node)| matches!(&node.kind, CoreProofNodeKind::IntroduceBinder { binder, .. } if binder.source_name.as_deref() == Some("X"))));
                    assert!(!expected.obligation_seeds().is_empty());
                    for (_, seed) in expected.obligation_seeds().iter() {
                        assert_eq!(seed.status, ObligationSeedStatus::Active);
                        assert!(expected.items().get(seed.owner).is_some());
                        assert!(matches!(
                            &expected.formulas().get(seed.goal.unwrap()).unwrap().kind,
                            CoreFormulaKind::Equals { .. }
                        ));
                    }
                    if text.starts_with("lemma") {
                        assert!(expected.proof_nodes().iter().any(|(_, node)| matches!(&node.kind, CoreProofNodeKind::TerminalGoal { citations, .. } if citations.iter().any(|citation| matches!(citation, CoreCitation::Symbol(_))))));
                    }
                    let before = (
                        input.clone(),
                        parent.clone(),
                        (*retained).clone(),
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
                                inputs
                                    .native_theorem_core_with_metadata_parent(&input, owner)
                                    .unwrap(),
                                Some(expected.clone())
                            );
                        }
                        assert_eq!(*owner.storage().get(&parent).unwrap(), *retained);
                    }
                    assert_eq!(
                        (
                            input,
                            parent.clone(),
                            (*owner.storage().get(&parent).unwrap()).clone(),
                            session.captured().clone(),
                        ),
                        before
                    );
                    let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(collected.outputs_dropped, 7);
                    assert_eq!(collected.abandoned_slots_dropped, 0);
                },
            );
        }
    }
}

#[test]
fn native_theorem_core_metadata_caller_preserves_refusals() {
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in [
        "missing",
        "extra",
        "wrong",
        "hash",
        "foreign-owner",
        "obsolete",
        "collected",
        "empty",
    ] {
        let source = if case == "empty" {
            "theorem Empty: for A being set holds A = A proof end;"
        } else {
            text
        };
        with_native_summary_parent(source, false, 0, |_, inputs, old, owner, outputs, _| {
            for (phase, kind) in [
                ("SignatureCollection", "SourceTheoremModuleSummary"),
                ("TypeChecking", "SourceTheoremTypedAst"),
                ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                ("OverloadResolution", "SourceTheoremMetadata"),
            ] {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new(phase),
                    OutputKind::new(kind),
                    IrWorkUnit::new("\"alpha\":\"main\""),
                ));
            }
            let child = |parents: Vec<AnyPhaseOutputRef>, key| {
                PhaseInput::new(
                    old.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        key,
                        vec![],
                        parents
                            .into_iter()
                            .map(|parent| {
                                SealedParentOutputHandle::from_published_output(
                                    owner,
                                    old.snapshot,
                                    parent,
                                )
                                .unwrap()
                            })
                            .collect(),
                    )
                    .unwrap(),
                )
            };
            let signature = inputs
                .publish_native_theorem_module_summary(old, owner)
                .unwrap()
                .unwrap();
            let type_input = child(vec![signature.erase()], old.identities().input_hash());
            let typed = inputs
                .publish_native_theorem_typed_ast(&type_input, owner)
                .unwrap()
                .unwrap();
            let reg_input = child(vec![typed.erase()], old.identities().input_hash());
            let registration = inputs
                .publish_native_theorem_registration_closure(&reg_input, owner)
                .unwrap()
                .unwrap();
            let meta_input = child(
                vec![typed.erase(), registration.erase()],
                old.identities().input_hash(),
            );
            let parent = inputs
                .publish_native_theorem_metadata(&meta_input, owner)
                .unwrap()
                .unwrap();
            let input = child(
                match case {
                    "missing" => vec![],
                    "extra" => vec![parent.erase(), typed.erase()],
                    "wrong" => vec![typed.erase()],
                    _ => vec![parent.erase()],
                },
                if case == "hash" {
                    hash(0xd1)
                } else {
                    old.identities().input_hash()
                },
            );
            let expected = if case == "empty" {
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
                let retained = owner.storage().get(&frontend).unwrap();
                let module =
                    ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                let (arena, env) = mizar_resolve::symbols::source_theorem_module_original_inputs(
                    retained.ast.as_ref().unwrap(),
                    &module,
                )
                .unwrap()
                .unwrap();
                let metadata = owner.storage().get(&parent).unwrap();
                let (scope, labels, resolved) =
                    mizar_resolve::symbols::source_proof_inputs(&arena, &env).unwrap();
                Some(mizar_checker::type_checker::SourceVariableSemanticsChecker::check_theorem_skeletons(&arena, &metadata.2, &scope, &env, &labels, &resolved, None).unwrap_err())
            } else {
                None
            };
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
                    7
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
            let actual = inputs
                .native_theorem_core_with_metadata_parent(&input, selected)
                .unwrap_err();
            if let Some(expected) = expected {
                assert_eq!(actual, expected);
            }
            let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(
                collected.outputs_dropped,
                if case == "collected" { 0 } else { 7 }
            );
            assert_eq!(collected.abandoned_slots_dropped, 0);
        });
    }
    with_native_summary_parent(
        text,
        false,
        0,
        |_, inputs, old, owner, outputs, (_, ids, snapshots)| {
            use mizar_driver::{
                native_theorem_metadata_from_storage_bytes as decode,
                native_theorem_metadata_storage_bytes as encode,
            };
            use mizar_ir::{
                publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
                storage::{BlobDecodeError, BlobDecoder, IrSideTables, SchemaVersion},
            };
            use mizar_resolve::{
                names::{
                    SourceVariableScopeInput as Input, SourceVariableScopeResolver as Resolver,
                },
                symbols::source_theorem_module_original_inputs as original_inputs,
            };
            for (phase, kind) in [
                ("SignatureCollection", "SourceTheoremModuleSummary"),
                ("TypeChecking", "SourceTheoremTypedAst"),
                ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                ("OverloadResolution", "SourceTheoremMetadata"),
            ] {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new(phase),
                    OutputKind::new(kind),
                    IrWorkUnit::new("\"alpha\":\"main\""),
                ));
            }
            let child = |parents: Vec<AnyPhaseOutputRef>| {
                PhaseInput::new(
                    old.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        old.identities().input_hash(),
                        vec![],
                        parents
                            .into_iter()
                            .map(|parent| {
                                SealedParentOutputHandle::from_published_output(
                                    owner,
                                    old.snapshot,
                                    parent,
                                )
                                .unwrap()
                            })
                            .collect(),
                    )
                    .unwrap(),
                )
            };
            let signature = inputs
                .publish_native_theorem_module_summary(old, owner)
                .unwrap()
                .unwrap();
            let type_input = child(vec![signature.erase()]);
            let typed = inputs
                .publish_native_theorem_typed_ast(&type_input, owner)
                .unwrap()
                .unwrap();
            let reg_input = child(vec![typed.erase()]);
            let registration = inputs
                .publish_native_theorem_registration_closure(&reg_input, owner)
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
            let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
            let [metadata_a, metadata_b] = [ast, &rebound].map(|ast| {
                let (arena, env) = original_inputs(ast, &module).unwrap().unwrap();
                let scope =
                    Resolver::resolve_proof_occurrences(Input::new(ast, &module, &env)).unwrap();
                mizar_checker::type_checker::check_source_theorem_metadata(&arena, &env, &scope)
                    .unwrap()
            });
            assert_eq!(metadata_a.0.source_id(), metadata_b.0.source_id());
            assert_eq!(metadata_a.3.module_id(), metadata_b.3.module_id());
            assert_ne!(metadata_a, metadata_b);
            assert_eq!(encode(ast, &module, &metadata_b).unwrap(), None);
            let bytes = encode(&rebound, &module, &metadata_b).unwrap().unwrap();
            assert_eq!(
                decode(&bytes, ast.source_id, &module).unwrap(),
                Some(metadata_b.clone())
            );
            let phase = IrPipelinePhase::new("OverloadResolution");
            let kind = OutputKind::new("SourceTheoremMetadata");
            let source_id = ast.source_id;
            let negative = owner
                .publish(PublishOutputInput {
                    slot: owner.allocate(
                        old.snapshot,
                        phase.clone(),
                        typed.work_unit().clone(),
                        kind.clone(),
                        SchemaVersion::new(1),
                    ),
                    snapshot: old.snapshot,
                    phase,
                    work_unit: typed.work_unit().clone(),
                    output_kind: kind,
                    schema_version: SchemaVersion::new(1),
                    payload: metadata_b.clone(),
                    canonical_payload: Some(bytes.clone()),
                    storage_payload: Some(bytes),
                    decode: BlobDecoder::new(move |bytes| {
                        decode(bytes, source_id, &module)
                            .map_err(BlobDecodeError::new)?
                            .ok_or_else(|| BlobDecodeError::new("negative metadata replay"))
                    }),
                    parents: vec![typed.erase(), registration.erase()],
                    named_input_hashes: typed.lineage().named_input_hashes.clone(),
                    side_tables: IrSideTables::default(),
                    origin: OutputOrigin::PackageSource,
                    target: PublicationTarget::CurrentPackage,
                })
                .unwrap();
            assert_eq!(*owner.storage().get(&negative).unwrap(), metadata_b);
            let input = child(vec![negative.erase()]);
            for _ in 0..2 {
                assert_eq!(
                    inputs
                        .native_theorem_core_with_metadata_parent(&input, owner)
                        .unwrap(),
                    None
                );
            }
            let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(collected.outputs_dropped, 7);
            assert_eq!(collected.abandoned_slots_dropped, 0);
        },
    );
}

#[test]
fn native_theorem_core_metadata_publication_seals_genuine_payload_and_streams() {
    use mizar_checker::type_checker::{
        SourceVariableSemanticsChecker, check_source_theorem_metadata as metadata,
    };
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_driver::native_theorem_core_storage_bytes;
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
        symbols::{source_proof_inputs, source_theorem_module_original_inputs as original_inputs},
    };
    let field = |hasher: &mut blake3::Hasher, bytes: &[u8]| {
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    };
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    for (open, threshold) in [(false, usize::MAX), (true, 0)] {
        let mut representations = Vec::new();
        for text in [
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            "theorem T: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
            "theorem T: for X being set holds X = X\nproof\n let X be set;\n thus X = X;\nend;",
            "theorem Renamed: for X being set holds X = X proof let X be set; thus X = X; end;",
        ] {
            with_native_summary_parent(
                text,
                open,
                threshold,
                |fixture, inputs, old, owner, _, (session, _, _)| {
                    for (phase, kind) in [
                        ("SignatureCollection", "SourceTheoremModuleSummary"),
                        ("TypeChecking", "SourceTheoremTypedAst"),
                        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                        ("OverloadResolution", "SourceTheoremMetadata"),
                        ("Elaboration", "SourceTheoremCore"),
                    ] {
                        owner.allow_work_unit(AllowedWorkUnit::new(
                            IrPipelinePhase::new(phase),
                            OutputKind::new(kind),
                            IrWorkUnit::new("\"alpha\":\"main\""),
                        ));
                    }
                    let child = |parents: Vec<AnyPhaseOutputRef>| {
                        PhaseInput::new(
                            old.work_unit.clone(),
                            PhaseDispatchInputBundle::new(
                                old.snapshot,
                                old.identities().input_hash(),
                                vec![],
                                parents
                                    .into_iter()
                                    .map(|parent| {
                                        SealedParentOutputHandle::from_published_output(
                                            owner,
                                            old.snapshot,
                                            parent,
                                        )
                                        .unwrap()
                                    })
                                    .collect(),
                            )
                            .unwrap(),
                        )
                    };
                    let signature = inputs
                        .publish_native_theorem_module_summary(old, owner)
                        .unwrap()
                        .unwrap();
                    let type_input = child(vec![signature.erase()]);
                    let typed = inputs
                        .publish_native_theorem_typed_ast(&type_input, owner)
                        .unwrap()
                        .unwrap();
                    let reg_input = child(vec![typed.erase()]);
                    let registration = inputs
                        .publish_native_theorem_registration_closure(&reg_input, owner)
                        .unwrap()
                        .unwrap();
                    let meta_input = child(vec![typed.erase(), registration.erase()]);
                    let parent = inputs
                        .publish_native_theorem_metadata(&meta_input, owner)
                        .unwrap()
                        .unwrap();
                    let input = child(vec![parent.erase()]);
                    let retained = owner.storage().get(&parent).unwrap();
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
                    let ast = output.ast.as_ref().unwrap();
                    let (arena, env) = original_inputs(ast, &module).unwrap().unwrap();
                    let scope = Resolver::resolve_proof_occurrences(Input::new(ast, &module, &env))
                        .unwrap();
                    let independent = metadata(&arena, &env, &scope).unwrap();
                    assert_eq!(*retained, independent);
                    let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
                    let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
                        &arena,
                        &independent.2,
                        &scope,
                        &env,
                        &labels,
                        &resolved,
                        None,
                    )
                    .unwrap();
                    assert!(!check.owners().is_empty());
                    let expected =
                        mizar_core::elaborator::lower_source_theorem_skeletons(&check, None)
                            .unwrap();
                    assert_eq!(expected.source_id(), loaded.source_id);
                    assert_eq!(expected.module_id(), &module);
                    assert!(!expected.proofs().is_empty());
                    assert!(
                        expected.proofs().iter().all(
                            |(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof
                        )
                    );
                    assert!(!expected.obligation_seeds().is_empty());
                    assert!(
                        expected
                            .obligation_seeds()
                            .iter()
                            .all(|(_, seed)| seed.status == ObligationSeedStatus::Active)
                    );
                    let semantic = mizar_core::elaborator::source_theorem_core_semantic_payload(
                        &check, &expected,
                    )
                    .unwrap();
                    assert!(semantic.starts_with(b"mizar-core/source-theorem-core-semantics/v1\0"));
                    let bytes = native_theorem_core_storage_bytes(ast, &module, &expected)
                        .unwrap()
                        .unwrap();
                    let version = &inputs.snapshot.source_versions[0];
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
                    let digest = Hash::from_bytes(blake3::derive_key(
                        "mizar-driver/native-theorem-core-storage-side-table/v1",
                        &bytes,
                    ));
                    let tables = IrSideTables {
                        source_maps: vec![SideTableRecord::new(
                            "elaborate-theorem-core-storage",
                            version.normalized_path.as_str(),
                            digest,
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
                            field(&mut side, b"elaborate-theorem-core-storage");
                            field(&mut side, version.normalized_path.as_str().as_bytes());
                            field(&mut side, b"side_table_digest");
                            side.update(digest.as_bytes());
                        }
                    }
                    let expected_side = Hash::from_bytes(*side.finalize().as_bytes());
                    let before = (
                        input.clone(),
                        (*retained).clone(),
                        session.captured().clone(),
                    );
                    let sealed = inputs
                        .publish_native_theorem_core_with_metadata_parent(&input, owner)
                        .unwrap()
                        .unwrap();
                    assert_eq!(sealed.snapshot(), old.snapshot);
                    assert_eq!(sealed.phase(), &IrPipelinePhase::new("Elaboration"));
                    assert_eq!(sealed.output_kind(), &OutputKind::new("SourceTheoremCore"));
                    assert_eq!(sealed.schema_version(), SchemaVersion::new(1));
                    assert_eq!(sealed.work_unit(), parent.work_unit());
                    assert_eq!(sealed.lineage().parents, vec![parent.output()]);
                    assert_eq!(
                        sealed.lineage().named_input_hashes,
                        parent.lineage().named_input_hashes
                    );
                    assert_eq!(sealed.content_hash(), expected_content);
                    assert_eq!(sealed.side_table_hash(), expected_side);
                    assert_eq!(*owner.storage().side_tables(&sealed).unwrap(), tables);
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
                                .publish_native_theorem_core_with_metadata_parent(&input, owner)
                                .unwrap_err(),
                            duplicate
                        );
                        owner
                            .validate_published_output(old.snapshot, sealed.any())
                            .unwrap();
                        assert_eq!(
                            owner
                                .validate_current_output(old.snapshot, sealed.any())
                                .is_ok(),
                            !open
                        );
                    }
                    assert_eq!(
                        (
                            input,
                            (*owner.storage().get(&parent).unwrap()).clone(),
                            session.captured().clone(),
                        ),
                        before
                    );
                    let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                        snapshot: old.snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(collected.outputs_dropped, 8);
                    assert_eq!(collected.abandoned_slots_dropped, 3);
                    representations.push((semantic, bytes));
                },
            );
        }
        assert_eq!(representations[0].0, representations[1].0);
        assert_eq!(representations[0].0, representations[2].0);
        assert_ne!(representations[0].0, representations[3].0);
        assert_ne!(representations[0].1, representations[1].1);
        assert_ne!(representations[0].1, representations[2].1);
    }
}

#[test]
fn native_theorem_core_metadata_publication_preserves_refusals() {
    use mizar_ir::{publisher::PublishError, storage::CollectInput};
    let ordinary = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    for case in [
        "missing",
        "wrong",
        "hash",
        "obsolete",
        "collected",
        "empty",
        "lemma",
        "denied",
        "conflict",
    ] {
        let text = match case {
            "empty" => "theorem Empty: for A being set holds A = A proof end;",
            "lemma" => include_str!(
                "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
            ),
            _ => ordinary,
        };
        with_native_summary_parent(text, false, 0, |_, inputs, old, owner, outputs, _| {
            for (phase, kind) in [
                ("SignatureCollection", "SourceTheoremModuleSummary"),
                ("TypeChecking", "SourceTheoremTypedAst"),
                ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                ("OverloadResolution", "SourceTheoremMetadata"),
            ] {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new(phase),
                    OutputKind::new(kind),
                    IrWorkUnit::new("\"alpha\":\"main\""),
                ));
            }
            let child = |parents: Vec<AnyPhaseOutputRef>| {
                PhaseInput::new(
                    old.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        old.snapshot,
                        old.identities().input_hash(),
                        vec![],
                        parents
                            .into_iter()
                            .map(|parent| {
                                SealedParentOutputHandle::from_published_output(
                                    owner,
                                    old.snapshot,
                                    parent,
                                )
                                .unwrap()
                            })
                            .collect(),
                    )
                    .unwrap(),
                )
            };
            let signature = inputs
                .publish_native_theorem_module_summary(old, owner)
                .unwrap()
                .unwrap();
            let type_input = child(vec![signature.erase()]);
            let typed = inputs
                .publish_native_theorem_typed_ast(&type_input, owner)
                .unwrap()
                .unwrap();
            let reg_input = child(vec![typed.erase()]);
            let registration = inputs
                .publish_native_theorem_registration_closure(&reg_input, owner)
                .unwrap()
                .unwrap();
            let meta_input = child(vec![typed.erase(), registration.erase()]);
            let parent = inputs
                .publish_native_theorem_metadata(&meta_input, owner)
                .unwrap()
                .unwrap();
            if case != "denied" {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("Elaboration"),
                    OutputKind::new("SourceTheoremCore"),
                    parent.work_unit().clone(),
                ));
            }
            let input = PhaseInput::new(
                old.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    old.snapshot,
                    if case == "hash" {
                        hash(0xe1)
                    } else {
                        old.identities().input_hash()
                    },
                    vec![],
                    match case {
                        "missing" => vec![],
                        "wrong" => vec![typed.erase()],
                        _ => vec![parent.erase()],
                    }
                    .into_iter()
                    .map(|parent| {
                        SealedParentOutputHandle::from_published_output(owner, old.snapshot, parent)
                            .unwrap()
                    })
                    .collect(),
                )
                .unwrap(),
            );
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
                    7
                );
            }
            let raw = inputs.native_theorem_core_with_metadata_parent(&input, owner);
            let expected = if case == "lemma" {
                let core = raw.unwrap().unwrap();
                let frontend = outputs
                    .iter()
                    .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                    .unwrap();
                let handle = owner
                    .storage()
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        frontend,
                        frontend.output_kind(),
                    )
                    .unwrap();
                let retained = owner.storage().get(&handle).unwrap();
                let ast = retained.ast.as_ref().unwrap();
                let module =
                    ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                assert!(
                    mizar_driver::native_theorem_core_storage_bytes(ast, &module, &core)
                        .unwrap()
                        .is_some()
                );
                mizar_driver::native_theorem_core_semantic_payload(ast, &module, &core).unwrap_err()
            } else if case == "denied" {
                assert!(raw.unwrap().is_some());
                PublishError::WorkUnitNotAllowed {
                    phase: IrPipelinePhase::new("Elaboration"),
                    work_unit: parent.work_unit().clone(),
                    output_kind: OutputKind::new("SourceTheoremCore"),
                }
                .to_string()
            } else if case == "conflict" {
                assert!(raw.unwrap().is_some());
                let frontend = outputs
                    .iter()
                    .find(|output| output.phase() == &IrPipelinePhase::new("Frontend"))
                    .unwrap();
                let frontend_input = child(vec![frontend.clone()]);
                let original = inputs
                    .publish_native_theorem_core(&frontend_input, owner)
                    .unwrap()
                    .unwrap();
                assert_eq!(original.lineage().parents, vec![frontend.output()]);
                let conflict = inputs
                    .publish_native_theorem_core_with_metadata_parent(&input, owner)
                    .unwrap_err();
                assert!(
                    conflict.contains("conflicting duplicate identity key")
                        && conflict.contains("phase_outputs")
                );
                conflict
            } else {
                raw.unwrap_err()
            };
            let actual = inputs
                .publish_native_theorem_core_with_metadata_parent(&input, owner)
                .unwrap_err();
            assert_eq!(actual, expected, "{case}");
            let collected = owner.storage().collect(CollectInput {
                snapshot: old.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(
                collected.outputs_dropped,
                match case {
                    "collected" => 0,
                    "conflict" => 8,
                    _ => 7,
                }
            );
            assert_eq!(
                collected.abandoned_slots_dropped,
                match case {
                    "denied" => 1,
                    "conflict" => 2,
                    _ => 0,
                }
            );
        });
    }
}
