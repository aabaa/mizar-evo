use super::*;

#[test]
fn native_theorem_vc_caller_preserves_genuine_raw_generation() {
    use mizar_core::core_ir::{
        CoreCitation, CoreFormulaKind, CoreItemKind, CoreProofNodeKind, CoreProofStatus,
        ObligationSeedStatus,
    };
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion, VcStatus};
    let lemma = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for text in [
        lemma.to_owned(),
        lemma
            .replace("let X be set;", "let Local be set;")
            .replace("thus X = X", "thus Local = Local"),
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;".into(),
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
                            text: std::sync::Arc::from(text.as_str()),
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
                let core = inputs
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
                let (_, loaded, captured_input) =
                    &inputs.captured.unwrap().native_sources().unwrap()[0];
                assert_eq!(loaded.text.as_ref(), text);
                assert_eq!(captured_input, &descriptor);
                assert_eq!(loaded.source_hash, mizar_session::source::hash_text(&text));
                assert_eq!(
                    loaded.source_id,
                    inputs.snapshot.source_versions[0].source_id
                );
                assert_eq!(
                    matches!(loaded.origin, SourceOrigin::OpenBuffer { .. }),
                    open
                );
                assert_eq!(retained.ast.as_ref().unwrap().source_id, loaded.source_id);
                assert_eq!(core.source_id(), loaded.source_id);
                assert_eq!(
                    core.module_id(),
                    &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"),)
                );
                let names = if text.starts_with("lemma") {
                    vec!["Lem1", "UseLem1"]
                } else {
                    vec!["T"]
                };
                assert_eq!(core.items().len(), names.len());
                assert!(core.items().iter().zip(&names).all(|((_, item), name)| {
                    item.symbol
                        .local()
                        .as_str()
                        .contains(&format!(":name={name}:"))
                }));
                assert_eq!(core.proofs().len(), names.len());
                assert!(
                    core.proofs()
                        .iter()
                        .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
                );
                assert_eq!(core.obligation_seeds().len(), names.len());
                for (_, seed) in core.obligation_seeds().iter() {
                    assert_eq!(seed.status, ObligationSeedStatus::Active);
                    assert!(core.items().get(seed.owner).is_some());
                    assert!(matches!(
                        core.formulas().get(seed.goal.unwrap()).unwrap().kind,
                        CoreFormulaKind::Equals { .. }
                    ));
                }
                let citations = core
                    .proof_nodes()
                    .iter()
                    .filter_map(|(_, node)| match &node.kind {
                        CoreProofNodeKind::TerminalGoal { citations, .. } => Some(citations),
                        _ => None,
                    })
                    .flatten()
                    .cloned()
                    .collect::<Vec<_>>();
                if names.len() == 2 {
                    let (_, first) = core.items().iter().next().unwrap();
                    assert_eq!(first.kind, CoreItemKind::Lemma);
                    assert_eq!(citations, vec![CoreCitation::Symbol(first.symbol.clone())]);
                } else {
                    assert!(citations.is_empty());
                }
                let before = (
                    inputs.snapshot.clone(),
                    inputs.build_plan.clone(),
                    inputs.module_index.clone(),
                    loaded.clone(),
                    captured_input.clone(),
                    (*retained).clone(),
                    input.clone(),
                    parent.clone(),
                    core.clone(),
                );
                for (generation, schema) in [
                    ("driver-native-generation", "driver-native-vc"),
                    (" \nraw generation\t", ""),
                ] {
                    let generation = GenerationSchemaVersion::new(generation);
                    let schema = VcSchemaVersion::new(schema);
                    let schemas_before = (generation.clone(), schema.clone());
                    let expected = mizar_vc::generator::generate_source_theorem_vcs(
                        &core,
                        snapshot,
                        &generation,
                        &schema,
                    )
                    .unwrap();
                    assert_eq!(expected.source(), loaded.source_id);
                    assert_eq!(expected.snapshot(), snapshot);
                    assert_eq!(expected.module().as_str(), "package=5:alpha;module=4:main");
                    assert_eq!(expected.schema_version(), &schema);
                    assert_eq!(expected.vcs().len(), names.len());
                    assert_eq!(
                        expected.seed_accounting().len(),
                        core.obligation_seeds().len()
                    );
                    assert!(
                        expected
                            .seed_accounting()
                            .iter()
                            .all(|seed| seed.seed_status == ObligationSeedStatus::Active)
                    );
                    assert!(expected.vcs().iter().all(|vc| vc.status == VcStatus::Open
                        && vc.anchor.generation_schema_version == generation));
                    for stage in 0..3 {
                        if stage == 1 {
                            fixture.write(b"open theorem Changed: 0 = 1;\n");
                        } else if stage == 2 {
                            fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                        }
                        for _ in 0..2 {
                            assert_eq!(
                                inputs
                                    .native_theorem_vcs_with_frontend_parent(
                                        &input,
                                        &owner,
                                        &generation,
                                        &schema,
                                    )
                                    .unwrap(),
                                Some(expected.clone())
                            );
                        }
                    }
                    assert_eq!((generation, schema), schemas_before);
                    // Restore a disk entry for the next schema's edit/removal replay.
                    fixture.write(&[0xff]);
                }
                assert_eq!(
                    (
                        inputs.snapshot.clone(),
                        inputs.build_plan.clone(),
                        inputs.module_index.clone(),
                        loaded.clone(),
                        captured_input.clone(),
                        (*owner.storage().get(&typed).unwrap()).clone(),
                        input,
                        parent.clone(),
                        core,
                    ),
                    before
                );
                let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                    snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.outputs_dropped, 2);
                assert_eq!(collected.abandoned_slots_dropped, 0);
            }
        }
    }
}

#[test]
fn native_theorem_vc_caller_preserves_refusals() {
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, case) in [
        (format!("{private}\nreserve Y for set;\n"), "reserve"),
        (private.replace("set", "object"), "metadata"),
        (
            "theorem Empty: for A being set holds A = A proof end;".to_owned(),
            "empty",
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
                        .native_theorem_metadata_with_frontend_parent(&input, owner,)
                        .unwrap_err())
                );
                Err(old.unwrap_err())
            }
            "empty" => {
                assert!(
                    inputs
                        .native_theorem_metadata_with_frontend_parent(&input, owner,)
                        .unwrap()
                        .is_some()
                );
                Err(old.unwrap_err())
            }
            _ => unreachable!(),
        };
        assert_eq!(
            inputs.native_theorem_vcs_with_frontend_parent(
                &input,
                owner,
                &GenerationSchemaVersion::new(" \n"),
                &VcSchemaVersion::new(""),
            ),
            expected,
            "{case}"
        );
        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
            snapshot: source_input.snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(collected.outputs_dropped, 2, "{case}");
        assert_eq!(collected.abandoned_slots_dropped, 0, "{case}");
    }
    for case in ["absent", "obsolete", "collected"] {
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
        let mut parents = vec![
            SealedParentOutputHandle::from_published_output(owner, snapshot, handle.erase())
                .unwrap(),
        ];
        match case {
            "absent" => parents.clear(),
            "obsolete" => owner.mark_obsolete(snapshot).unwrap(),
            "collected" => {
                let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                    snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.outputs_dropped, 2);
                assert_eq!(collected.abandoned_slots_dropped, 0);
            }
            _ => unreachable!(),
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
        let expected = inputs
            .native_theorem_core_with_frontend_parent(&input, owner)
            .unwrap_err();
        assert_eq!(
            inputs
                .native_theorem_vcs_with_frontend_parent(
                    &input,
                    owner,
                    &GenerationSchemaVersion::new(""),
                    &VcSchemaVersion::new(" \t"),
                )
                .unwrap_err(),
            expected,
            "{case}"
        );
        let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
            snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(
            collected.outputs_dropped,
            if case == "collected" { 0 } else { 2 }
        );
        assert_eq!(collected.abandoned_slots_dropped, 0);
    }
}

#[test]
fn native_theorem_vc_storage_replays_genuine_graphs() {
    use mizar_core::core_ir::{
        CoreCitation, CoreFormulaKind, CoreProofNodeKind, CoreProofStatus, ObligationSeedStatus,
    };
    use mizar_driver::{
        native_theorem_core_from_storage_bytes, native_theorem_core_storage_bytes,
        native_theorem_vc_from_storage_bytes, native_theorem_vc_storage_bytes,
    };
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion, VcStatus};
    let lemma = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    for text in [
        lemma.to_owned(),
        lemma
            .replace("let X be set;", "let Local be set;")
            .replace("thus X = X", "thus Local = Local"),
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;".into(),
    ] {
        for (open, threshold) in [
            (false, usize::MAX),
            (false, 0),
            (true, usize::MAX),
            (true, 0),
        ] {
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
                        text: std::sync::Arc::from(text.as_str()),
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
            let core = inputs
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
            let (_, loaded, captured) = &inputs.captured.unwrap().native_sources().unwrap()[0];
            assert_eq!(loaded.text.as_ref(), text);
            assert_eq!(captured, &descriptor);
            assert_eq!(
                matches!(loaded.origin, SourceOrigin::OpenBuffer { .. }),
                open
            );
            assert_eq!(core.source_id(), loaded.source_id);
            assert_eq!(core.module_id(), &module);
            let names = if text.starts_with("lemma") {
                vec!["Lem1", "UseLem1"]
            } else {
                vec!["T"]
            };
            assert_eq!(core.items().len(), names.len());
            assert_eq!(core.proofs().len(), names.len());
            assert!(
                core.proofs()
                    .iter()
                    .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
            );
            assert_eq!(core.obligation_seeds().len(), names.len());
            for (_, seed) in core.obligation_seeds().iter() {
                assert_eq!(seed.status, ObligationSeedStatus::Active);
                assert!(core.items().get(seed.owner).is_some());
                assert!(matches!(
                    core.formulas().get(seed.goal.unwrap()).unwrap().kind,
                    CoreFormulaKind::Equals { .. }
                ));
            }
            let citations = core
                .proof_nodes()
                .iter()
                .filter_map(|(_, node)| match &node.kind {
                    CoreProofNodeKind::TerminalGoal { citations, .. } => Some(citations),
                    _ => None,
                })
                .flatten()
                .cloned()
                .collect::<Vec<_>>();
            let expected_citations = if names.len() == 2 {
                vec![CoreCitation::Symbol(
                    core.items().iter().next().unwrap().1.symbol.clone(),
                )]
            } else {
                vec![]
            };
            assert_eq!(citations, expected_citations);
            let ast = retained.ast.as_ref().unwrap();
            let before = (
                inputs.snapshot.clone(),
                (*retained).clone(),
                input.clone(),
                parent.clone(),
                core.clone(),
            );
            let other = Fixture::new(
                b"theorem Other: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
            );
            let (_, other_submission, other_adapter, other_handle, _) =
                native_disk_frontend_record_submission(&other, &ids, &snapshots, threshold);
            let (other_inputs, other_source) = native_disk_lookup_context(&other_submission, &ids);
            let other_owner = other_adapter.publisher();
            let other_input = PhaseInput::new(
                other_source.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    other_source.snapshot,
                    other_source.identities().input_hash(),
                    vec![],
                    vec![
                        SealedParentOutputHandle::from_published_output(
                            other_owner,
                            other_source.snapshot,
                            other_handle.erase(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            );
            let other_core = other_inputs
                .native_theorem_core_with_frontend_parent(&other_input, other_owner)
                .unwrap()
                .unwrap();
            assert_ne!(other_core.source_id(), loaded.source_id);
            assert_ne!(other_source.snapshot, snapshot);
            for (gs, schema) in [
                ("driver-native-generation", "driver-native-vc"),
                (" \n世代\t\0", ""),
            ] {
                let gs = GenerationSchemaVersion::new(gs);
                let schema = VcSchemaVersion::new(schema);
                let expected =
                    mizar_vc::generator::generate_source_theorem_vcs(&core, snapshot, &gs, &schema)
                        .unwrap();
                assert_eq!(
                    inputs
                        .native_theorem_vcs_with_frontend_parent(&input, &owner, &gs, &schema)
                        .unwrap(),
                    Some(expected.clone())
                );
                assert_eq!(expected.source(), loaded.source_id);
                assert_eq!(expected.snapshot(), snapshot);
                assert_eq!(expected.module().as_str(), "package=5:alpha;module=4:main");
                assert_eq!(expected.schema_version(), &schema);
                assert_eq!(expected.vcs().len(), names.len());
                assert_eq!(expected.seed_accounting().len(), names.len());
                assert!(
                    expected
                        .seed_accounting()
                        .iter()
                        .all(|seed| seed.seed_status == ObligationSeedStatus::Active)
                );
                assert!(expected.vcs().iter().all(|vc| vc.status == VcStatus::Open
                    && vc.anchor.generation_schema_version == gs
                    && !vc.provenance.is_empty()));
                let core_bytes = native_theorem_core_storage_bytes(ast, &module, &core)
                    .unwrap()
                    .unwrap();
                let rebound_core = native_theorem_core_from_storage_bytes(
                    &core_bytes,
                    other_core.source_id(),
                    &module,
                )
                .unwrap()
                .unwrap();
                let rebound = mizar_vc::generator::generate_source_theorem_vcs(
                    &rebound_core,
                    other_source.snapshot,
                    &gs,
                    &schema,
                )
                .unwrap();
                let foreign = mizar_vc::generator::generate_source_theorem_vcs(
                    &other_core,
                    other_source.snapshot,
                    &gs,
                    &schema,
                )
                .unwrap();
                assert_ne!(foreign, expected);
                let mut framed = b"mizar-driver/native-theorem-vc-storage/v1\0".to_vec();
                for field in [
                    gs.as_str().as_bytes(),
                    schema.as_str().as_bytes(),
                    core_bytes.as_slice(),
                ] {
                    framed.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
                    framed.extend_from_slice(field);
                }
                for stage in 0..3 {
                    if stage == 1 {
                        fixture.write(b"open theorem Changed: 0 = 1;\n");
                    }
                    if stage == 2 {
                        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                    }
                    for _ in 0..2 {
                        let bytes = native_theorem_vc_storage_bytes(
                            ast, &module, &core, &expected, snapshot, &gs,
                        )
                        .unwrap()
                        .unwrap();
                        assert_eq!(bytes, framed);
                        for (source, context, graph) in [
                            (loaded.source_id, snapshot, &expected),
                            (other_core.source_id(), other_source.snapshot, &rebound),
                        ] {
                            assert_eq!(
                                native_theorem_vc_from_storage_bytes(
                                    &bytes, source, &module, context
                                )
                                .unwrap(),
                                Some(graph.clone())
                            );
                        }
                    }
                }
                let changed = GenerationSchemaVersion::new("different generation");
                for (candidate, context, supplied) in [
                    (&foreign, snapshot, &gs),
                    (&expected, snapshot, &changed),
                    (&expected, other_source.snapshot, &gs),
                ] {
                    assert_eq!(
                        native_theorem_vc_storage_bytes(
                            ast, &module, &core, candidate, context, supplied
                        ),
                        Ok(None)
                    );
                }
                fixture.write(&[0xff]);
            }
            assert_eq!(
                (
                    inputs.snapshot.clone(),
                    (*owner.storage().get(&typed).unwrap()).clone(),
                    input,
                    parent.clone(),
                    core
                ),
                before
            );
            for (storage, snapshot) in [
                (owner.storage(), snapshot),
                (other_owner.storage(), other_source.snapshot),
            ] {
                let collected = storage.collect(mizar_ir::storage::CollectInput {
                    snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.outputs_dropped, 2);
                assert_eq!(collected.abandoned_slots_dropped, 0);
            }
        }
    }
}

#[test]
fn native_theorem_vc_storage_preserves_refusals() {
    use mizar_driver::{
        native_theorem_core_from_storage_bytes, native_theorem_core_storage_bytes,
        native_theorem_vc_from_storage_bytes, native_theorem_vc_storage_bytes,
    };
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};
    let module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let fixture = Fixture::new(
        b"theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
    );
    let (_, submission, adapter, handle, _) =
        native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
    let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
    let owner = adapter.publisher();
    let snapshot = source_input.snapshot;
    let input = PhaseInput::new(
        source_input.work_unit.clone(),
        PhaseDispatchInputBundle::new(
            snapshot,
            source_input.identities().input_hash(),
            vec![],
            vec![
                SealedParentOutputHandle::from_published_output(owner, snapshot, handle.erase())
                    .unwrap(),
            ],
        )
        .unwrap(),
    );
    let core = inputs
        .native_theorem_core_with_frontend_parent(&input, owner)
        .unwrap()
        .unwrap();
    let gs = GenerationSchemaVersion::new(" \n\t");
    let schema = VcSchemaVersion::new("");
    let vcs =
        mizar_vc::generator::generate_source_theorem_vcs(&core, snapshot, &gs, &schema).unwrap();
    let retained = owner.storage().get(&handle).unwrap();
    let ast = retained.ast.as_ref().unwrap();
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, is_none) in [
        (format!("{private}\nreserve Y for set;\n"), true),
        (private.replace("set", "object"), false),
    ] {
        let refused = Fixture::new(text.as_bytes());
        let (_, _, refused_adapter, refused_handle, _) =
            native_disk_frontend_record_submission(&refused, &ids, &snapshots, 0);
        let refused_owner = refused_adapter.publisher();
        let output = refused_owner.storage().get(&refused_handle).unwrap();
        let refused_ast = output.ast.as_ref().unwrap();
        let old = native_theorem_core_storage_bytes(refused_ast, &module, &core);
        if is_none {
            assert_eq!(old, Ok(None));
        } else {
            assert!(old.is_err());
        }
        let oversized = GenerationSchemaVersion::new("x".repeat(32 * 1024 * 1024));
        for supplied_generation in [&gs, &oversized] {
            assert_eq!(
                native_theorem_vc_storage_bytes(
                    refused_ast,
                    &module,
                    &core,
                    &vcs,
                    snapshot,
                    supplied_generation
                ),
                old
            );
        }
        let mut inner = b"mizar-driver/native-theorem-core-storage/v1\0".to_vec();
        let syntax = refused_ast.canonical_bytes().unwrap();
        for field in [b"alpha".as_slice(), b"main".as_slice(), syntax.as_slice()] {
            inner.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
            inner.extend_from_slice(field);
        }
        let decoded =
            native_theorem_core_from_storage_bytes(&inner, refused_ast.source_id, &module);
        assert_eq!(
            decoded.as_ref().map(Option::is_none),
            old.as_ref().map(Option::is_none)
        );
        let mut outer = b"mizar-driver/native-theorem-vc-storage/v1\0".to_vec();
        for field in [
            gs.as_str().as_bytes(),
            schema.as_str().as_bytes(),
            inner.as_slice(),
        ] {
            outer.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
            outer.extend_from_slice(field);
        }
        assert_eq!(
            native_theorem_vc_from_storage_bytes(&outer, refused_ast.source_id, &module, snapshot),
            decoded.map(|_| None)
        );
        let collected = refused_owner
            .storage()
            .collect(mizar_ir::storage::CollectInput {
                snapshot: refused_handle.erase().snapshot(),
                protected_outputs: vec![],
            });
        assert_eq!(collected.outputs_dropped, 2);
        assert_eq!(collected.abandoned_slots_dropped, 0);
    }
    let inner = native_theorem_core_storage_bytes(ast, &module, &core)
        .unwrap()
        .unwrap();
    let domain = b"mizar-driver/native-theorem-vc-storage/v1\0";
    let mut bytes = domain.to_vec();
    for field in [
        gs.as_str().as_bytes(),
        schema.as_str().as_bytes(),
        inner.as_slice(),
    ] {
        bytes.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
        bytes.extend_from_slice(field);
    }
    let mut corruptions = vec![
        Vec::new(),
        bytes[..domain.len() + 3].to_vec(),
        bytes[..bytes.len() - 1].to_vec(),
    ];
    let mut wrong_domain = bytes.clone();
    wrong_domain[0] = b'x';
    corruptions.push(wrong_domain);
    let mut wrong_version = bytes.clone();
    wrong_version[domain.len() - 2] = b'2';
    corruptions.push(wrong_version);
    let mut length = bytes.clone();
    length[domain.len()..domain.len() + 8].copy_from_slice(&u64::MAX.to_le_bytes());
    corruptions.push(length);
    let mut trailing = bytes.clone();
    trailing.push(0);
    corruptions.push(trailing);
    for fields in [
        [&[0xff][..], schema.as_str().as_bytes(), inner.as_slice()],
        [gs.as_str().as_bytes(), &[0xff][..], inner.as_slice()],
    ] {
        let mut invalid_utf8 = domain.to_vec();
        for field in fields {
            invalid_utf8.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
            invalid_utf8.extend_from_slice(field);
        }
        corruptions.push(invalid_utf8);
    }
    let mut oversized = bytes.clone();
    oversized.resize(32 * 1024 * 1024 + 1, 0);
    corruptions.push(oversized);
    for malformed in corruptions {
        assert_eq!(
            native_theorem_vc_from_storage_bytes(&malformed, ast.source_id, &module, snapshot),
            Ok(None)
        );
    }
    let oversized = GenerationSchemaVersion::new("x".repeat(32 * 1024 * 1024));
    assert_eq!(
        native_theorem_vc_storage_bytes(ast, &module, &core, &vcs, snapshot, &oversized),
        Ok(None)
    );
    let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
        snapshot,
        protected_outputs: vec![],
    });
    assert_eq!(collected.outputs_dropped, 2);
    assert_eq!(collected.abandoned_slots_dropped, 0);
}

#[test]
fn native_theorem_vc_publication_seals_genuine_graph_and_streams() {
    use mizar_core::core_ir::{CoreProofStatus, ObligationSeedStatus};
    use mizar_driver::{
        native_theorem_core_semantic_payload, native_theorem_vc_from_storage_bytes,
        native_theorem_vc_storage_bytes,
    };
    use mizar_vc::vc_ir::{GenerationSchemaVersion, HashMarker, VcSchemaVersion, VcStatus};
    for (text, generation, schema) in [
        (
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            "native-generation",
            "native-vc",
        ),
        (
            "theorem T: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
            " \n世代\t\0",
            "",
        ),
        (
            "theorem T: for X being set holds X = X\nproof\n  let X be set;\n  thus X = X;\nend;\n",
            "native-generation",
            "native-vc",
        ),
    ] {
        let generation = GenerationSchemaVersion::new(generation);
        let schema = VcSchemaVersion::new(schema);
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
                    IrPipelinePhase::new("VcGenerate"),
                    OutputKind::new("SourceTheoremVC"),
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
                let core = inputs
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
                assert_eq!(
                    matches!(loaded.origin, SourceOrigin::OpenBuffer { .. }),
                    open
                );
                assert_eq!(core.source_id(), loaded.source_id);
                assert_eq!(core.module_id(), &module);
                assert_eq!(core.proofs().len(), 1);
                assert!(
                    core.proofs()
                        .iter()
                        .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
                );
                assert_eq!(core.obligation_seeds().len(), 1);
                assert!(
                    core.obligation_seeds()
                        .iter()
                        .all(|(_, seed)| seed.status == ObligationSeedStatus::Active)
                );
                let expected = mizar_vc::generator::generate_source_theorem_vcs(
                    &core,
                    snapshot,
                    &generation,
                    &schema,
                )
                .unwrap();
                assert_eq!(
                    inputs
                        .native_theorem_vcs_with_frontend_parent(
                            &input,
                            &owner,
                            &generation,
                            &schema,
                        )
                        .unwrap(),
                    Some(expected.clone())
                );
                assert_eq!(expected.source(), loaded.source_id);
                assert_eq!(expected.snapshot(), snapshot);
                assert_eq!(expected.module().as_str(), "package=5:alpha;module=4:main");
                assert_eq!(expected.schema_version(), &schema);
                assert_eq!(expected.vcs().len(), 1);
                assert_eq!(expected.seed_accounting().len(), 1);
                assert!(
                    expected
                        .seed_accounting()
                        .iter()
                        .all(|seed| seed.seed_status == ObligationSeedStatus::Active)
                );
                for vc in expected.vcs() {
                    assert_eq!(vc.status, VcStatus::Open);
                    assert!(matches!(vc.source.primary.anchor,
                        mizar_core::core_ir::CoreSourceAnchor::SourceRange(range)
                        if range.source_id == loaded.source_id && range.start < range.end));
                    assert!(matches!(expected.seed_accounting()[0].mapping,
                        mizar_vc::vc_ir::SeedVcMapping::One { vc: id } if id == vc.id));
                    assert_eq!(vc.anchor.generation_schema_version, generation);
                    assert!(!vc.provenance.is_empty());
                    assert!(matches!(
                        vc.anchor.canonical_goal_hash,
                        HashMarker::ConservativeUnknown { .. }
                    ));
                    assert!(matches!(
                        vc.anchor.canonical_context_hash,
                        HashMarker::ConservativeUnknown { .. }
                    ));
                }
                let core_semantic = native_theorem_core_semantic_payload(ast, &module, &core)
                    .unwrap()
                    .unwrap();
                let mut semantic = b"mizar-driver/native-theorem-vc-content/v1\0".to_vec();
                for field in [
                    generation.as_str().as_bytes(),
                    schema.as_str().as_bytes(),
                    core_semantic.as_slice(),
                ] {
                    semantic.extend_from_slice(&u64::try_from(field.len()).unwrap().to_le_bytes());
                    semantic.extend_from_slice(field);
                }
                let bytes = native_theorem_vc_storage_bytes(
                    ast,
                    &module,
                    &core,
                    &expected,
                    snapshot,
                    &generation,
                )
                .unwrap()
                .unwrap();
                assert_eq!(
                    native_theorem_vc_from_storage_bytes(
                        &bytes,
                        loaded.source_id,
                        &module,
                        snapshot
                    )
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
                    .publish_native_theorem_vcs(&input, &owner, &generation, &schema)
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
                            .native_theorem_vcs_with_frontend_parent(
                                &input,
                                &owner,
                                &generation,
                                &schema,
                            )
                            .unwrap(),
                        Some(expected.clone())
                    );
                    assert_eq!(
                        native_theorem_core_semantic_payload(ast, &module, &core).unwrap(),
                        Some(core_semantic.clone())
                    );
                    assert_eq!(
                        native_theorem_vc_storage_bytes(
                            ast,
                            &module,
                            &core,
                            &expected,
                            snapshot,
                            &generation
                        )
                        .unwrap(),
                        Some(bytes.clone())
                    );
                    assert_eq!(
                        native_theorem_vc_from_storage_bytes(
                            &bytes,
                            loaded.source_id,
                            &module,
                            snapshot
                        )
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
                            .publish_native_theorem_vcs(&input, &owner, &generation, &schema)
                            .unwrap_err(),
                        duplicate
                    );
                    assert_eq!(sealed.any(), &original_handle);
                    assert_eq!(*owner.storage().get(&sealed).unwrap(), expected);
                    assert_eq!(sealed.snapshot(), snapshot);
                    assert_eq!(sealed.phase(), &IrPipelinePhase::new("VcGenerate"));
                    assert_eq!(sealed.output_kind(), &OutputKind::new("SourceTheoremVC"));
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
                                "generate-theorem-vc-storage",
                                version.normalized_path.as_str(),
                                Hash::from_bytes(blake3::derive_key(
                                    "mizar-driver/native-theorem-vc-storage-side-table/v1",
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
                let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                    snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.outputs_dropped, 3);
                assert_eq!(collected.abandoned_slots_dropped, 3);
            }
        }
    }
}

#[test]
fn native_theorem_vc_publication_preserves_refusals_and_publisher_boundaries() {
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};
    let generation = GenerationSchemaVersion::new("native-generation");
    let schema = VcSchemaVersion::new("native-vc");
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, case) in [
        (format!("{private}\nreserve Y for set;\n"), "reserve"),
        (private.replace("set", "object"), "metadata"),
        (
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;"
                .to_owned(),
            "oversized",
        ),
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
            IrPipelinePhase::new("VcGenerate"),
            OutputKind::new("SourceTheoremVC"),
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
        let schema = if case == "oversized" {
            VcSchemaVersion::new("x".repeat(32 * 1024 * 1024))
        } else {
            schema.clone()
        };
        let old_core = inputs.native_theorem_core_with_frontend_parent(&input, owner);
        let old =
            inputs.native_theorem_vcs_with_frontend_parent(&input, owner, &generation, &schema);
        let expected = match case {
            "reserve" => {
                assert_eq!(old_core, Ok(None));
                assert_eq!(old, Ok(None));
                Ok(None)
            }
            "metadata" | "empty" => {
                let metadata = inputs.native_theorem_metadata_with_frontend_parent(&input, owner);
                if case == "metadata" {
                    assert_eq!(old_core.as_ref().unwrap_err(), &metadata.unwrap_err());
                } else {
                    assert!(metadata.unwrap().is_some());
                }
                assert_eq!(old.as_ref().unwrap_err(), old_core.as_ref().unwrap_err());
                Err(old.unwrap_err())
            }
            "lemma" => {
                let core = old_core.unwrap().unwrap();
                assert_eq!(
                    old.unwrap().unwrap(),
                    mizar_vc::generator::generate_source_theorem_vcs(
                        &core,
                        input.snapshot,
                        &generation,
                        &schema,
                    )
                    .unwrap()
                );
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
            "oversized" => {
                assert!(old_core.unwrap().is_some());
                let graph = old.unwrap().unwrap();
                assert_eq!(graph.vcs().len(), 1);
                assert_eq!(graph.schema_version(), &schema);
                drop(graph);
                Ok(None)
            }
            _ => unreachable!(),
        };
        let actual = inputs
            .publish_native_theorem_vcs(&input, owner, &generation, &schema)
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
                "VcGenerate"
            }),
            OutputKind::new(if case == "denied-kind" {
                "WrongKind"
            } else {
                "SourceTheoremVC"
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
        let old =
            inputs.native_theorem_vcs_with_frontend_parent(&input, owner, &generation, &schema);
        let actual = inputs
            .publish_native_theorem_vcs(&input, owner, &generation, &schema)
            .unwrap_err();
        if case.starts_with("denied-") {
            assert!(old.unwrap().is_some());
            assert_eq!(
                actual,
                mizar_ir::publisher::PublishError::WorkUnitNotAllowed {
                    phase: IrPipelinePhase::new("VcGenerate"),
                    work_unit: IrWorkUnit::new("\"alpha\":\"main\""),
                    output_kind: OutputKind::new("SourceTheoremVC"),
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
        assert_eq!(
            collected.abandoned_slots_dropped,
            usize::from(case.starts_with("denied-"))
        );
    }
}
