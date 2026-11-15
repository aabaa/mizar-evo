use super::*;

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
                .submit_captured(session, &ids, &snapshots, submit, Default::default())
                .unwrap();
            let (inputs, _) = native_disk_lookup_context(&submission, &ids);
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
            let typed = owner
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    parent,
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let retained = owner.storage().get(&typed).unwrap();
            let core =
                genuine_theorem_core_from_ast(retained.ast.as_ref().unwrap(), &module).unwrap();
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
                parent.clone(),
                core.clone(),
            );
            let other = Fixture::new(
                b"theorem Other: for Y being set holds Y = Y proof let Y be set; thus Y = Y; end;",
            );
            let (_, other_submission, other_adapter, other_handle, _) =
                native_disk_frontend_record_submission(&other, &ids, &snapshots, threshold);
            let (_, other_source) = native_disk_lookup_context(&other_submission, &ids);
            let other_owner = other_adapter.publisher();
            let other_output = other_owner.storage().get(&other_handle).unwrap();
            let other_core =
                genuine_theorem_core_from_ast(other_output.ast.as_ref().unwrap(), &module).unwrap();
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
    let (_, source_input) = native_disk_lookup_context(&submission, &ids);
    let owner = adapter.publisher();
    let snapshot = source_input.snapshot;
    let retained = owner.storage().get(&handle).unwrap();
    let ast = retained.ast.as_ref().unwrap();
    let core = genuine_theorem_core_from_ast(ast, &module).unwrap();
    let gs = GenerationSchemaVersion::new(" \n\t");
    let schema = VcSchemaVersion::new("");
    let vcs =
        mizar_vc::generator::generate_source_theorem_vcs(&core, snapshot, &gs, &schema).unwrap();
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for (text, is_none) in [
        (format!("{private}\nreserve Y for set;\n"), true),
        (private.replacen("thus X = X;", "thus X = 0;", 1), false),
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
