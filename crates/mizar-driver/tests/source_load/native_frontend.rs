use super::*;

#[test]
fn native_frontend_resolver_binding_preserves_scheduled_captured_pairs() {
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for text in [
        private.replace("by PrivT1", "by PrivT1, PrivT1"),
        private.replace("theorem UsePriv1", "lemma UsePriv1"),
        private.replace("\ntheorem UsePriv1", "\nprivate theorem UsePriv1"),
        include_str!("../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz").to_owned(),
        format!("\u{feff}{}\r\nopen theorem Numeral: 0 = 0;\r\n", private.replace("holds X = X", "holds X ::= 日本語 formula =:: = X").replace('\n', "\r\n")),
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
                            uri: format!("file://{}", fixture.root.join("alpha/src/main.miz").display()),
                            expected_version: 7,
                            actual_version: 7,
                            text: std::sync::Arc::from(text.as_str()),
                        }
                    } else {
                        SourceOriginInput::Disk { path: "src/main.miz".into() }
                    },
                };
                if open {
                    fixture.write(&[0xff]);
                }
                let session = draft.capture_sources(
                    &ids, &snapshots,
                    vec![(DiskSourceLoader::new(fixture.root.join("alpha")), descriptor.clone())],
                ).unwrap();
                let snapshot = session.captured().snapshot().id;
                let module = mizar_build::module_index::ModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
                let unit = WorkUnit::Module { module: module.clone() };
                let owner = publisher(snapshot, threshold, &unit);
                allow_frontend(&owner, &unit);
                let mut builder = PhaseRegistryBuilder::new();
                builder.register_source_load();
                builder.register_frontend(vec![]);
                let mut driver = CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
                let submission = driver.submit_captured(session, &ids, &snapshots, submit).unwrap();
                let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
                let parent = submission.scheduler_run.as_ref().unwrap().phase_results.values().flatten()
                    .flat_map(|result| &result.output_refs)
                    .find(|handle| handle.output_kind() == &OutputKind::new("FrontendOutput")).unwrap();
                assert_eq!(parent.placement() == &StoragePlacement::Resident, threshold != 0);
                let input = PhaseInput::new(unit, PhaseDispatchInputBundle::new(
                    snapshot, source_input.identities().input_hash(), vec![],
                    vec![SealedParentOutputHandle::from_published_output(&owner, snapshot, parent.clone()).unwrap()],
                ).unwrap());
                let (_, loaded, captured_input) = &inputs.captured.unwrap().native_sources().unwrap()[0];
                let source = mizar_frontend::source::source_unit_from_loaded(
                    loaded.clone(), mizar_frontend::source::diagnostic_file_path(captured_input),
                );
                assert_eq!(source.source_text.as_ref(), text.trim_start_matches('\u{feff}').replace("\r\n", "\n"));
                let output = mizar_frontend::orchestration::Frontend::new(
                    mizar_frontend::source::FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root.join("alpha"))),
                    inputs.dependency_lexical_provider(&[]), MizarParserSeam,
                ).run_loaded(source).unwrap();
                assert!(output.diagnostics.is_empty());
                let typed = owner.storage().typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    parent, &OutputKind::new("FrontendOutput"),
                ).unwrap();
                assert_eq!(*owner.storage().get(&typed).unwrap(), output);
                let ast = output.ast.as_ref().unwrap();
                let resolver_module = ResolverModuleId::new(module.package.clone(), module.path.clone());
                let shells = DeclarationShellCollector::new(ast, &resolver_module).collect();
                let collection = SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new("main")).collect(&resolver_module);
                assert!(collection.diagnostics().is_empty());
                let arena = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(ast, &resolver_module).unwrap();
                let expected = mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, collection.env()).unwrap().unwrap();
                assert_eq!(expected.0.source_id(), loaded.source_id);
                assert_eq!(expected.0.module_id(), &resolver_module);
                if text.contains("by PrivT1, PrivT1") {
                    assert_eq!(expected.0.label_refs().len(), 2);
                    assert_eq!(expected.1.declaration_dependencies().iter().count(), 2);
                }
                let before = (inputs.snapshot.clone(), inputs.build_plan.clone(), inputs.module_index.clone(),
                    loaded.clone(), captured_input.clone(), output.clone(), arena.clone(), collection.env().clone(), input.clone());
                for stage in 0..3 {
                    if stage == 1 {
                        fixture.write(b"open theorem Changed: 0 = 1;\n");
                    } else if stage == 2 {
                        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                    }
                    assert_eq!(inputs.native_theorem_module_candidates_with_frontend_parent(&input, &owner).unwrap(),
                        Some(expected.clone()), "open={open} threshold={threshold} stage={stage}");
                }
                mizar_resolve::symbols::validate_source_symbol_env(&arena, collection.env()).unwrap();
                assert_eq!((inputs.snapshot.clone(), inputs.build_plan.clone(), inputs.module_index.clone(),
                    loaded.clone(), captured_input.clone(), output.clone(), arena.clone(), collection.env().clone(), input.clone()), before);
                assert_eq!(captured_input, &descriptor);
            }
        }
    }
}

#[test]
fn native_frontend_resolver_binding_rejects_bad_associations_and_preserves_lower_results() {
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, case) in [
        (format!("{private}\nreserve Y for set;\n"), "unsupported"),
        (private.replace("by PrivT1", "by Missing"), "unresolved"),
        (private.replace("thus X = X;", "thus Y = Y;"), "error"),
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
        let output = owner.storage().get(&handle).unwrap();
        let ast = output.ast.as_ref().unwrap();
        let WorkUnit::Module { module } = &input.work_unit else {
            unreachable!()
        };
        let resolver_module = ResolverModuleId::new(module.package.clone(), module.path.clone());
        let shells = DeclarationShellCollector::new(ast, &resolver_module).collect();
        let collection =
            SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new("main"))
                .collect(&resolver_module);
        assert!(collection.diagnostics().is_empty());
        let arena = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(ast, &resolver_module)
            .unwrap();
        let lower =
            mizar_resolve::symbols::source_theorem_module_candidates(ast, &arena, collection.env());
        if case == "error" {
            assert!(lower.is_err());
        } else {
            assert_eq!(lower, Ok(None));
            if case == "unresolved" {
                assert!(
                    mizar_resolve::resolved_ast::source_theorem_resolved_ast(
                        ast,
                        &arena,
                        collection.env()
                    )
                    .unwrap()
                    .unwrap()
                    .label_refs()
                    .iter()
                    .any(|(_, row)| matches!(
                        row.resolution(),
                        mizar_resolve::resolved_ast::LabelResolution::Unresolved(_)
                    ))
                );
            }
        }
        assert_eq!(
            inputs.native_theorem_module_candidates_with_frontend_parent(&input, owner),
            lower
        );
    }
    let fixture = Fixture::new(private.as_bytes());
    let foreign = Fixture::new(private.as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (_, foreign_submission, foreign_adapter, foreign_handle, _) =
        native_disk_frontend_record_submission(&foreign, &ids, &snapshots, 0);
    for case in [
        "absent",
        "extra",
        "kind",
        "foreign",
        "snapshot",
        "work-unit",
        "module",
        "package",
        "hash",
        "dependencies",
        "capture",
        "foreign-capture",
        "ambiguous-source",
        "absent-index",
        "ambiguous-index",
        "foreign-index",
        "unpublished",
        "foreign-storage",
        "obsolete",
        "collected",
        "phase",
        "schema",
        "parent-unit",
        "nonproducer",
        "nonproducer-key",
        "no-ancestor",
        "extra-ancestor",
        "ancestor-phase",
        "ancestor-kind",
        "ancestor-unit",
        "ancestor-input",
        "ancestor-parent",
    ] {
        let (_, submission, adapter, handle, frontend_input) =
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
        let mut work_unit = source_input.work_unit.clone();
        let mut input_hash = source_input.identities().input_hash();
        let mut dependencies = vec![];
        let mut snapshot = inputs.snapshot.clone();
        let mut index = inputs.module_index.clone();
        let mut selected_inputs = inputs;
        let alternative;
        let selected_owner = if case == "unpublished" {
            alternative = std::sync::Arc::new(PhaseOutputPublisher::new(
                owner.storage().clone(),
                owner.registry().clone(),
            ));
            alternative.register_current_snapshot(source_input.snapshot);
            &alternative
        } else if case == "foreign-storage" {
            alternative = publisher(source_input.snapshot, 0, &work_unit);
            &alternative
        } else if matches!(
            case,
            "phase"
                | "schema"
                | "parent-unit"
                | "nonproducer"
                | "nonproducer-key"
                | "no-ancestor"
                | "extra-ancestor"
        ) || case.starts_with("ancestor-")
        {
            alternative = std::sync::Arc::new(PhaseOutputPublisher::new(
                owner.storage().clone(),
                std::sync::Arc::new(SnapshotHandleRegistry::new()),
            ));
            alternative.register_current_snapshot(source_input.snapshot);
            if !case.starts_with("ancestor-") {
                let ancestor = frontend_input.parent_outputs()[0].as_output_ref().lineage();
                alternative
                    .registry()
                    .register_output(mizar_ir::identity::OutputIdentityInput {
                        snapshot: ancestor.snapshot,
                        phase: ancestor.phase.clone(),
                        work_unit: ancestor.work_unit.clone(),
                        output_kind: ancestor.output_kind.clone(),
                        content_hash: ancestor.content_hash,
                        side_table_hash: ancestor.side_table_hash,
                        parents: ancestor.parents.clone(),
                        named_input_hashes: ancestor.named_input_hashes.clone(),
                    })
                    .unwrap();
            }
            &alternative
        } else {
            owner
        };
        match case {
            "absent" => parents.clear(),
            "extra" => parents.push(frontend_input.parent_outputs()[0].clone()),
            "kind" => parents = frontend_input.parent_outputs().to_vec(),
            "foreign" => {
                parents = vec![
                    SealedParentOutputHandle::from_published_output(
                        foreign_adapter.publisher(),
                        foreign_handle.snapshot(),
                        foreign_handle.erase(),
                    )
                    .unwrap(),
                ];
                snapshot = foreign_submission.session.captured().snapshot().clone();
            }
            "work-unit" => work_unit = WorkUnit::Workspace,
            "module" | "package" => {
                let WorkUnit::Module { module } = &mut work_unit else {
                    unreachable!()
                };
                if case == "module" {
                    module.path = ModulePath::new("absent");
                } else {
                    module.package = PackageId::new("foreign");
                }
            }
            "hash" => input_hash = Hash::from_bytes([0x46; 32]),
            "dependencies" => dependencies.push(input_hash),
            "capture" => selected_inputs.captured = None,
            "foreign-capture" => {
                selected_inputs.captured = Some(foreign_submission.session.captured())
            }
            "ambiguous-source" => {
                snapshot
                    .source_versions
                    .push(snapshot.source_versions[0].clone());
                selected_inputs.snapshot = &snapshot;
            }
            "absent-index" => {
                index.modules.clear();
                selected_inputs.module_index = &index;
            }
            "ambiguous-index" => {
                index.modules.push(index.modules[0].clone());
                selected_inputs.module_index = &index;
            }
            "foreign-index" => {
                index.modules[0].module.package = PackageId::new("foreign");
                selected_inputs.module_index = &index;
            }
            "obsolete" => owner.mark_obsolete(source_input.snapshot).unwrap(),
            "collected" => {
                owner.storage().collect(mizar_ir::storage::CollectInput {
                    snapshot: source_input.snapshot,
                    protected_outputs: vec![],
                });
            }
            "phase" | "schema" | "parent-unit" | "nonproducer" | "nonproducer-key"
            | "no-ancestor" | "extra-ancestor" | "ancestor-phase" | "ancestor-kind"
            | "ancestor-unit" | "ancestor-input" | "ancestor-parent" => {
                // Generic publications below are adversarial derivatives, never positive service producers.
                let mut actual = (*owner.storage().get(&handle).unwrap()).clone();
                let mut ancestors =
                    vec![frontend_input.parent_outputs()[0].as_output_ref().clone()];
                if case.starts_with("ancestor-") || case == "extra-ancestor" {
                    let original = owner
                        .storage()
                        .typed_handle::<SourceUnit>(&ancestors[0], &OutputKind::new("SourceUnit"))
                        .unwrap();
                    let source = (*owner.storage().get(&original).unwrap()).clone();
                    for step in 0..if case == "ancestor-parent" { 2 } else { 1 } {
                        let phase = if case == "ancestor-phase" {
                            handle.phase().clone()
                        } else {
                            original.phase().clone()
                        };
                        let kind = if case == "ancestor-kind" {
                            handle.output_kind().clone()
                        } else {
                            original.output_kind().clone()
                        };
                        let unit = if case == "ancestor-unit"
                            || case == "extra-ancestor"
                            || (case == "ancestor-parent" && step == 0)
                        {
                            IrWorkUnit::new("negative-only")
                        } else {
                            original.work_unit().clone()
                        };
                        selected_owner.allow_work_unit(AllowedWorkUnit::new(
                            phase.clone(),
                            kind.clone(),
                            unit.clone(),
                        ));
                        let mut named = original.lineage().named_input_hashes.clone();
                        if case == "ancestor-input" {
                            named[0].digest = Hash::from_bytes([0x47; 32]);
                        }
                        let bytes = source.canonical_disk_bytes().unwrap();
                        let decode_source = source.clone();
                        let descriptor = inputs.captured.unwrap().native_sources().unwrap()[0]
                            .2
                            .clone();
                        let negative = selected_owner
                            .publish(mizar_ir::publisher::PublishOutputInput {
                                slot: selected_owner.allocate(
                                    source_input.snapshot,
                                    phase.clone(),
                                    unit.clone(),
                                    kind.clone(),
                                    mizar_ir::storage::SchemaVersion::new(1),
                                ),
                                snapshot: source_input.snapshot,
                                phase,
                                work_unit: unit,
                                output_kind: kind,
                                schema_version: mizar_ir::storage::SchemaVersion::new(1),
                                payload: source.clone(),
                                canonical_payload: Some(bytes.clone()),
                                storage_payload: Some(bytes),
                                decode: mizar_ir::storage::BlobDecoder::new(move |bytes| {
                                    SourceUnit::from_canonical_storage_bytes(
                                        bytes,
                                        decode_source.source_id,
                                        &descriptor,
                                    )
                                    .ok_or_else(|| {
                                        mizar_ir::storage::BlobDecodeError::new("negative ancestry")
                                    })
                                }),
                                parents: if case == "ancestor-parent" && step == 1 {
                                    ancestors.clone()
                                } else {
                                    vec![]
                                },
                                named_input_hashes: named,
                                side_tables: (*owner.storage().side_tables(&original).unwrap())
                                    .clone(),
                                origin: mizar_ir::publisher::OutputOrigin::PackageSource,
                                target: mizar_ir::publisher::PublicationTarget::CurrentPackage,
                            })
                            .unwrap();
                        if case == "extra-ancestor" {
                            ancestors.push(negative.erase());
                        } else {
                            ancestors = vec![negative.erase()];
                        }
                    }
                } else if case == "no-ancestor" {
                    ancestors.clear();
                }
                if case == "nonproducer" {
                    actual.tokens.tokens[0].text = "negative-only-token".into();
                } else if case == "nonproducer-key" {
                    let original = owner.storage().get(&handle).unwrap();
                    actual.cache_keys.ast.as_mut().unwrap().parser_inputs_hash =
                        Hash::from_bytes([0x48; 32]);
                    assert_ne!(actual.cache_keys.ast, original.cache_keys.ast);
                    assert_eq!(actual.tokens, original.tokens);
                }
                let raw = actual.canonical_disk_bytes().unwrap();
                let descriptor = inputs.captured.unwrap().native_sources().unwrap()[0]
                    .2
                    .clone();
                let source_id = actual.source.source_id;
                let decoded =
                    FrontendOutput::from_canonical_storage_bytes(&raw, source_id, &descriptor)
                        .unwrap();
                assert_eq!(decoded, actual);
                assert_eq!(decoded.canonical_disk_bytes().unwrap(), raw);
                let phase = if case == "phase" {
                    IrPipelinePhase::new("SourceLoad")
                } else {
                    handle.phase().clone()
                };
                let unit = if case == "parent-unit" {
                    IrWorkUnit::new("negative-only")
                } else {
                    handle.work_unit().clone()
                };
                let schema =
                    mizar_ir::storage::SchemaVersion::new(if case == "schema" { 2 } else { 1 });
                selected_owner.allow_work_unit(AllowedWorkUnit::new(
                    phase.clone(),
                    handle.output_kind().clone(),
                    unit.clone(),
                ));
                let negative = selected_owner
                    .publish(mizar_ir::publisher::PublishOutputInput {
                        slot: selected_owner.allocate(
                            source_input.snapshot,
                            phase.clone(),
                            unit.clone(),
                            handle.output_kind().clone(),
                            schema,
                        ),
                        snapshot: source_input.snapshot,
                        phase,
                        work_unit: unit,
                        output_kind: handle.output_kind().clone(),
                        schema_version: schema,
                        payload: actual,
                        canonical_payload: Some(raw.clone()),
                        storage_payload: Some(raw),
                        decode: mizar_ir::storage::BlobDecoder::new(move |bytes| {
                            FrontendOutput::from_canonical_storage_bytes(
                                bytes,
                                source_id,
                                &descriptor,
                            )
                            .ok_or_else(|| {
                                mizar_ir::storage::BlobDecodeError::new("negative aggregate")
                            })
                        }),
                        parents: ancestors,
                        named_input_hashes: handle.lineage().named_input_hashes.clone(),
                        side_tables: (*owner.storage().side_tables(&handle).unwrap()).clone(),
                        origin: mizar_ir::publisher::OutputOrigin::PackageSource,
                        target: mizar_ir::publisher::PublicationTarget::CurrentPackage,
                    })
                    .unwrap();
                parents = vec![
                    SealedParentOutputHandle::from_published_output(
                        selected_owner,
                        source_input.snapshot,
                        negative.erase(),
                    )
                    .unwrap(),
                ];
            }
            _ => {}
        }
        let mut input = PhaseInput::new(
            work_unit,
            PhaseDispatchInputBundle::new(snapshot.id, input_hash, dependencies, parents).unwrap(),
        );
        if case == "snapshot" {
            input.snapshot = foreign_submission.session.captured().snapshot().id;
        }
        let error = selected_inputs
            .native_theorem_module_candidates_with_frontend_parent(&input, selected_owner)
            .unwrap_err();
        if matches!(case, "nonproducer" | "nonproducer-key") {
            assert_eq!(error, "native Frontend resolver complete output mismatch");
        }
    }
    // A real failed scheduled Frontend has no positive service parent to supply.
    let diagnostic = Fixture::new(b"theorem Broken: ;\n");
    let (mut draft, submit) = diagnostic.submit_request(vec![], vec![]);
    draft.source_inputs.versions.clear();
    let session = draft
        .capture_sources(
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(diagnostic.root.join("alpha")),
                SourceInput {
                    package_id: diagnostic.version.package_id.clone(),
                    module_path: diagnostic.version.module_path.clone(),
                    normalized_path: diagnostic.version.normalized_path.clone(),
                    edition: diagnostic.version.edition.clone(),
                    origin: SourceOriginInput::Disk {
                        path: "src/main.miz".into(),
                    },
                },
            )],
        )
        .unwrap();
    let unit = WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            PackageId::new("alpha"),
            ModulePath::new("main"),
        ),
    };
    let owner = publisher(session.captured().snapshot().id, 0, &unit);
    allow_frontend(&owner, &unit);
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(vec![]);
    let submission = CompilerDriver::new(builder.build().unwrap())
        .with_output_publisher(owner.clone())
        .submit_captured(session, &ids, &snapshots, submit)
        .unwrap();
    let (inputs, input) = native_disk_lookup_context(&submission, &ids);
    let results = submission
        .scheduler_run
        .as_ref()
        .unwrap()
        .phase_results
        .values()
        .flatten()
        .collect::<Vec<_>>();
    assert!(
        results
            .iter()
            .any(|result| result.status == PhaseStatus::Recoverable)
    );
    assert!(
        !results
            .iter()
            .flat_map(|result| &result.output_refs)
            .any(|output| output.output_kind() == &OutputKind::new("FrontendOutput"))
    );
    let (_, loaded, descriptor) = &inputs.captured.unwrap().native_sources().unwrap()[0];
    let output = mizar_frontend::orchestration::Frontend::new(
        mizar_frontend::source::FrontendSourceLoader::new(DiskSourceLoader::new(
            diagnostic.root.join("alpha"),
        )),
        inputs.dependency_lexical_provider(&[]),
        MizarParserSeam,
    )
    .run_loaded(mizar_frontend::source::source_unit_from_loaded(
        loaded.clone(),
        mizar_frontend::source::diagnostic_file_path(descriptor),
    ))
    .unwrap();
    assert!(!output.diagnostics.is_empty());
    assert!(
        inputs
            .native_theorem_module_candidates_with_frontend_parent(&input, &owner)
            .is_err()
    );
}
