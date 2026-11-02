use super::*;

#[test]
fn native_theorem_summary_publication_seals_whole_genuine_summary_and_bytes() {
    use mizar_artifact::{
        module_summary::{ModuleSummaryReadOptions, read_module_summary, write_module_summary},
        store::parse_canonical_json_bytes,
    };
    use mizar_resolve::{env::SymbolEnv, resolved_ast::ResolvedAst};
    let private = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    let lemma = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let statuses = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_theorem_status_open_assumed_001.miz"
    );
    for (text, open, threshold) in [
        (private, false, usize::MAX),
        (lemma, false, usize::MAX),
        (statuses, true, 0),
    ] {
        with_native_summary_parent(
            text,
            open,
            threshold,
            |fixture, inputs, input, owner, outputs, (session, ids, snapshots)| {
                let WorkUnit::Module { module } = &input.work_unit else {
                    unreachable!()
                };
                let version = &inputs.snapshot.source_versions[0];
                let parent = outputs
                    .iter()
                    .find(|output| output.phase() == &IrPipelinePhase::new("ModuleResolve"))
                    .unwrap();
                let expected_pair = inputs
                    .native_theorem_module_candidates(module)
                    .unwrap()
                    .unwrap();
                let typed = owner
                    .storage()
                    .typed_handle::<(ResolvedAst, SymbolEnv)>(parent, parent.output_kind())
                    .unwrap();
                assert_eq!(*owner.storage().get(&typed).unwrap(), expected_pair);
                assert_eq!(expected_pair.0.source_id(), version.source_id);
                let expected = inputs
                    .native_theorem_module_summary_candidate(module)
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    expected.module,
                    ModuleSummaryIdentity {
                        package_id: "alpha".into(),
                        package_version: Some("0.1.0".into()),
                        lockfile_identity: None,
                        module_path: "main".into(),
                        language_edition: "2025".into(),
                    }
                );
                assert_eq!(expected.source_hash, version.source_hash);
                assert_eq!(expected.lexical_summary.fingerprint, None);
                assert!(
                    expected
                        .exported_symbols
                        .iter()
                        .all(|row| row.proof_status.is_none())
                );
                assert!(!expected.exported_symbols.is_empty());
                for row in &expected.exported_symbols {
                    assert!(row.source_range.start_byte < row.source_range.end_byte);
                    assert!(row.source_range.end_byte <= text.len() as u64);
                }
                let bytes = write_module_summary(&expected).unwrap();
                let parsed = parse_canonical_json_bytes(&bytes, "summary baseline").unwrap();
                assert_eq!(
                    read_module_summary(
                        &parsed,
                        ModuleSummaryReadOptions {
                            expected_module: Some(&expected.module),
                            expected_interface_hash: Some(expected.interface_hash),
                            ..ModuleSummaryReadOptions::default()
                        }
                    )
                    .unwrap(),
                    expected
                );
                let source_named = vec![mizar_ir::identity::NamedInputHash {
                    name: "source".into(),
                    domain: SOURCE_UNIT_CACHE_KEY_VERSION.into(),
                    digest: source_key(version),
                }];
                assert_eq!(parent.lineage().named_input_hashes, source_named);
                assert_eq!(
                    parent.schema_version(),
                    mizar_ir::storage::SchemaVersion::new(1)
                );
                let [frontend_id] = parent.lineage().parents.as_slice() else {
                    panic!("one Frontend ancestor")
                };
                let frontend = owner.registry().output_lineage(*frontend_id).unwrap();
                assert_eq!(frontend.snapshot, input.snapshot);
                assert_eq!(frontend.phase, IrPipelinePhase::new("Frontend"));
                assert_eq!(frontend.work_unit, *parent.work_unit());
                assert_eq!(frontend.output_kind, OutputKind::new("FrontendOutput"));
                let mut content = blake3::Hasher::new();
                for field in [
                    b"mizar-ir/publisher-content-hash/v1".as_slice(),
                    bytes.as_slice(),
                ] {
                    content.update(&(field.len() as u64).to_le_bytes());
                    content.update(field);
                }
                content.update(&1_u64.to_le_bytes());
                content.update(&("parent_content".len() as u64).to_le_bytes());
                content.update(b"parent_content");
                content.update(parent.content_hash().as_bytes());
                content.update(&1_u64.to_le_bytes());
                for field in [
                    "source",
                    SOURCE_UNIT_CACHE_KEY_VERSION,
                    "mizar-ir/publisher-named-input-hash/v1",
                ] {
                    content.update(&(field.len() as u64).to_le_bytes());
                    content.update(field.as_bytes());
                }
                content.update(source_key(version).as_bytes());
                let expected_content = Hash::from_bytes(*content.finalize().as_bytes());
                let mut blob = blake3::Hasher::new();
                let domain = b"mizar-ir/content-blob/v1";
                blob.update(&(domain.len() as u64).to_le_bytes());
                blob.update(domain);
                blob.update(&1_u32.to_le_bytes());
                blob.update(&(bytes.len() as u64).to_le_bytes());
                blob.update(&bytes);
                let expected_blob = Hash::from_bytes(*blob.finalize().as_bytes());
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("SignatureCollection"),
                    OutputKind::new("SourceTheoremModuleSummary"),
                    parent.work_unit().clone(),
                ));
                if open {
                    fixture.write(&[0xff]);
                } else {
                    fixture.write(b"open theorem Changed: 0 = 1;\n");
                }
                fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                assert_eq!(
                    inputs.native_theorem_module_candidates(module).unwrap(),
                    Some(expected_pair)
                );
                assert_eq!(
                    inputs
                        .native_theorem_module_summary_candidate(module)
                        .unwrap(),
                    Some(expected.clone())
                );
                let sealed = inputs
                    .publish_native_theorem_module_summary(input, owner)
                    .unwrap()
                    .unwrap();
                assert_eq!(*owner.storage().get(&sealed).unwrap(), expected);
                assert_eq!(sealed.snapshot(), input.snapshot);
                assert_eq!(sealed.phase(), &IrPipelinePhase::new("SignatureCollection"));
                assert_eq!(
                    sealed.output_kind(),
                    &OutputKind::new("SourceTheoremModuleSummary")
                );
                assert_eq!(sealed.work_unit(), &IrWorkUnit::new("\"alpha\":\"main\""));
                assert_eq!(
                    sealed.schema_version(),
                    mizar_ir::storage::SchemaVersion::new(1)
                );
                assert_eq!(sealed.lineage().parents, vec![parent.output()]);
                assert_eq!(sealed.lineage().named_input_hashes, source_named);
                assert_eq!(sealed.content_hash(), expected_content);
                assert_eq!(
                    *owner.storage().side_tables(&sealed).unwrap(),
                    mizar_ir::storage::IrSideTables::default()
                );
                match sealed.placement() {
                    StoragePlacement::Resident => assert_eq!(threshold, usize::MAX),
                    StoragePlacement::Blob { blob, len } => {
                        assert_eq!(threshold, 0);
                        assert_eq!(*len, bytes.len());
                        assert_eq!(blob.hash(), expected_blob);
                    }
                    _ => panic!("unexpected storage placement"),
                }
                owner
                    .validate_published_output(input.snapshot, sealed.any())
                    .unwrap();
                assert_eq!(
                    owner
                        .validate_current_output(input.snapshot, sealed.any())
                        .is_ok(),
                    !open
                );
                assert!(
                    owner
                        .validate_published_output(snapshot_id(0xf8), sealed.any())
                        .is_err()
                );
                let replay_owner = publisher(
                    input.snapshot,
                    if threshold == 0 { usize::MAX } else { 0 },
                    &input.work_unit,
                );
                allow_frontend(&replay_owner, &input.work_unit);
                for (phase, kind) in [
                    ("ModuleResolve", "SourceTheoremModuleCandidates"),
                    ("SignatureCollection", "SourceTheoremModuleSummary"),
                ] {
                    replay_owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new(phase),
                        OutputKind::new(kind),
                        parent.work_unit().clone(),
                    ));
                }
                let mut builder = PhaseRegistryBuilder::new();
                builder
                    .register_source_load()
                    .register_frontend(vec![])
                    .register_source_theorem_module_resolver();
                let (_, submit) = fixture.submit_request(vec![], vec![]);
                let replay = CompilerDriver::new(builder.build().unwrap())
                    .with_output_publisher(replay_owner.clone())
                    .submit_captured(session.clone(), ids, snapshots, submit)
                    .unwrap();
                assert_eq!(replay.session.captured(), session.captured());
                let replay_run = replay.scheduler_run.as_ref().unwrap();
                let replay_parent = replay_run
                    .phase_results
                    .values()
                    .flatten()
                    .flat_map(|result| &result.output_refs)
                    .find(|output| output.phase() == &IrPipelinePhase::new("ModuleResolve"))
                    .unwrap();
                let (replay_inputs, _) = native_disk_lookup_context(&replay, ids);
                let replay_parent = SealedParentOutputHandle::from_published_output(
                    &replay_owner,
                    input.snapshot,
                    replay_parent.clone(),
                )
                .unwrap();
                let replay_input = PhaseInput::new(
                    input.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        input.snapshot,
                        input.identities().input_hash(),
                        vec![],
                        vec![replay_parent],
                    )
                    .unwrap(),
                );
                let replayed = replay_inputs
                    .publish_native_theorem_module_summary(&replay_input, &replay_owner)
                    .unwrap()
                    .unwrap();
                let actual = replay_owner.storage().get(&replayed).unwrap();
                assert_eq!(*actual, expected);
                assert_eq!(write_module_summary(&actual).unwrap(), bytes);
                assert_eq!(replayed.content_hash(), expected_content);
                assert_eq!(replayed.output(), sealed.output());
                match replayed.placement() {
                    StoragePlacement::Resident => assert_eq!(threshold, 0),
                    StoragePlacement::Blob { blob, len } => {
                        assert_eq!(threshold, usize::MAX);
                        assert_eq!(*len, bytes.len());
                        assert_eq!(blob.hash(), expected_blob);
                    }
                    _ => panic!("unexpected replay placement"),
                }
            },
        );
    }
}

#[test]
fn native_theorem_summary_publication_refuses_real_parent_association_failures() {
    let text = include_str!(
        "../../../../tests/miz/pass/resolve/pass_declaration_symbol_private_theorem_visibility_001.miz"
    );
    for case in [
        "missing",
        "extra",
        "source-parent",
        "frontend-parent",
        "foreign-owner",
        "wrong-unit",
    ] {
        with_native_summary_parent(text, false, 0, |_, inputs, input, owner, outputs, _| {
            let parent = |phase: &str| {
                outputs
                    .iter()
                    .find(|output| output.phase() == &IrPipelinePhase::new(phase))
                    .unwrap()
                    .clone()
            };
            let parents = match case {
                "missing" => vec![],
                "extra" => vec![parent("ModuleResolve"), parent("Frontend")],
                "source-parent" => vec![parent("SourceLoad")],
                "frontend-parent" => vec![parent("Frontend")],
                _ => vec![parent("ModuleResolve")],
            }
            .into_iter()
            .map(|output| {
                SealedParentOutputHandle::from_published_output(owner, input.snapshot, output)
                    .unwrap()
            })
            .collect();
            let candidate = PhaseInput::new(
                if case == "wrong-unit" {
                    WorkUnit::Package {
                        package_id: PackageId::new("alpha"),
                    }
                } else {
                    input.work_unit.clone()
                },
                PhaseDispatchInputBundle::new(
                    input.snapshot,
                    input.identities().input_hash(),
                    vec![],
                    parents,
                )
                .unwrap(),
            );
            owner.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("SignatureCollection"),
                OutputKind::new("SourceTheoremModuleSummary"),
                IrWorkUnit::new("\"alpha\":\"main\""),
            ));
            let foreign = PhaseOutputPublisher::new(
                owner.storage().clone(),
                std::sync::Arc::new(SnapshotHandleRegistry::new()),
            );
            foreign.register_current_snapshot(input.snapshot);
            let selected = if case == "foreign-owner" {
                &foreign
            } else {
                owner
            };
            assert!(
                inputs
                    .publish_native_theorem_module_summary(&candidate, selected)
                    .is_err(),
                "{case}"
            );
            let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                snapshot: input.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(collected.outputs_dropped, 3, "{case}");
            assert_eq!(collected.abandoned_slots_dropped, 0, "{case}");
        });
    }
}

#[test]
fn native_theorem_summary_publication_refuses_resources_and_currentness_before_success() {
    let text = include_str!(
        "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for case in [
        "source-hash",
        "dependencies",
        "snapshot-dependency",
        "index-dependency",
        "no-capture",
        "obsolete",
        "collected",
        "denied-phase",
        "denied-kind",
    ] {
        with_native_summary_parent(text, false, 0, |_, inputs, input, owner, _, _| {
            let WorkUnit::Module { module } = &input.work_unit else {
                unreachable!()
            };
            let mut snapshot = inputs.snapshot.clone();
            if case == "snapshot-dependency" {
                snapshot
                    .dependency_artifacts
                    .push(DependencyArtifactRef::new("dep/summary.json", hash(0x71)));
            }
            let mut index = inputs.module_index.clone();
            if case == "index-dependency" {
                index.dependency_summaries.push(DependencyModuleSummaryRef {
                    module: module.clone(),
                    artifact: "dep/summary.json".into(),
                    content_hash: hash(0x74),
                });
            }
            let inputs = SourceLoadInputs {
                snapshot: &snapshot,
                module_index: &index,
                captured: if case == "no-capture" {
                    None
                } else {
                    inputs.captured
                },
                ..inputs
            };
            let old_pair = inputs.native_theorem_module_candidates(module);
            let old_summary = inputs.native_theorem_module_summary_candidate(module);
            if matches!(case, "no-capture" | "snapshot-dependency") {
                assert!(old_pair.is_err());
                assert!(old_summary.is_err());
            } else {
                assert!(old_pair.as_ref().unwrap().is_some());
                assert!(old_summary.as_ref().unwrap().is_some());
            }
            let candidate = PhaseInput::new(
                input.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    input.snapshot,
                    if case == "source-hash" {
                        hash(0x72)
                    } else {
                        input.identities().input_hash()
                    },
                    if case == "dependencies" {
                        vec![hash(0x73)]
                    } else {
                        vec![]
                    },
                    input.parent_outputs().to_vec(),
                )
                .unwrap(),
            );
            owner.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new(if case == "denied-phase" {
                    "WrongPhase"
                } else {
                    "SignatureCollection"
                }),
                OutputKind::new(if case == "denied-kind" {
                    "WrongKind"
                } else {
                    "SourceTheoremModuleSummary"
                }),
                IrWorkUnit::new("\"alpha\":\"main\""),
            ));
            if case == "obsolete" {
                owner.mark_obsolete(input.snapshot).unwrap();
            }
            if case == "collected" {
                assert_eq!(
                    owner
                        .storage()
                        .collect(mizar_ir::storage::CollectInput {
                            snapshot: input.snapshot,
                            protected_outputs: vec![]
                        })
                        .outputs_dropped,
                    3
                );
            }
            let actual = inputs
                .publish_native_theorem_module_summary(&candidate, owner)
                .unwrap_err();
            if case == "no-capture" {
                assert_eq!(actual, old_pair.unwrap_err());
            } else if case.starts_with("denied-") {
                assert_eq!(
                    actual,
                    mizar_ir::publisher::PublishError::WorkUnitNotAllowed {
                        phase: IrPipelinePhase::new("SignatureCollection"),
                        work_unit: IrWorkUnit::new("\"alpha\":\"main\""),
                        output_kind: OutputKind::new("SourceTheoremModuleSummary"),
                    }
                    .to_string()
                );
            }
            let collected = owner.storage().collect(mizar_ir::storage::CollectInput {
                snapshot: input.snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(
                collected.outputs_dropped,
                if case == "collected" { 0 } else { 3 },
                "{case}"
            );
            assert_eq!(
                collected.abandoned_slots_dropped,
                usize::from(case.starts_with("denied-")),
                "{case}"
            );
        });
    }
}
