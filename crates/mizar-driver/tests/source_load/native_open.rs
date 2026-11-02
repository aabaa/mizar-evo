use super::*;

#[test]
fn native_open_buffer_payloads_publish_internal_blobs_with_genuine_parent_authority() {
    use mizar_frontend::{
        cache_key::{ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION, TOKEN_STREAM_CACHE_KEY_VERSION},
        orchestration::Frontend,
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_ir::{
        dispatch_input::SealedParentOutputHandle,
        identity::NamedInputHash,
        publisher::{OutputOrigin, PublicationTarget, PublishError, PublishOutputInput},
        storage::{BlobDecodeError, BlobDecoder, IrSideTables, SchemaVersion, SideTableRecord},
    };
    use std::sync::{Arc, atomic::AtomicUsize};
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let disk_submission = fixture.submit(&ids, &snapshots);
    let disk_context = &disk_submission.session.captured().snapshot();
    let package_root = fixture.root.join("alpha");
    let input = SourceInput {
        package_id: fixture.version.package_id.clone(),
        module_path: fixture.version.module_path.clone(),
        normalized_path: fixture.version.normalized_path.clone(),
        edition: fixture.version.edition.clone(),
        origin: SourceOriginInput::OpenBuffer {
            uri: format!("file://{}", package_root.join("src/main.miz").display()),
            expected_version: 7,
            actual_version: 7,
            text: Arc::from("\u{feff}definition\r\nend;\r\n"),
        },
    };
    let (snapshot, _lease, captured) = snapshots
        .load_snapshot(
            ids.next_request_id().unwrap(),
            (
                disk_context.workspace_root.clone(),
                disk_context.dependency_artifacts.clone(),
                disk_context.lockfile_hash,
                disk_context.toolchain.clone(),
                disk_context.verifier_config_hash,
            ),
            vec![(DiskSourceLoader::new(&package_root), input)],
        )
        .unwrap();
    let (loaded, captured_input) = captured.into_iter().next().unwrap();
    let source = source_unit_from_loaded(loaded, package_root.join("src/main.miz"));
    assert_eq!(snapshot.source_versions[0].source_id, source.source_id);
    assert!(matches!(
        source.origin,
        SourceOrigin::OpenBuffer { version: 7 }
    ));
    let inputs = SourceLoadInputs {
        captured: None,
        snapshot: &snapshot,
        build_plan: disk_submission.build_plan.as_ref().unwrap(),
        module_index: disk_submission.module_index.as_ref().unwrap(),
        allocator: &ids,
    };
    let output = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&package_root)),
        inputs.dependency_lexical_provider(&[]),
        MizarParserSeam,
    )
    .run_loaded(source.clone())
    .unwrap();
    assert!(output.ast.is_some());
    assert!(output.diagnostics.is_empty());
    let unit = &source_task(&disk_submission).unit;
    let publisher = publisher(snapshot.id, 0, unit);
    allow_frontend(&publisher, unit);
    let WorkUnit::Module { module } = unit else {
        unreachable!()
    };
    let ir_unit = IrWorkUnit::new(format!(
        "{:?}:{:?}",
        module.package.as_str(),
        module.path.as_str()
    ));
    let source_id = source.source_id;
    let expected_hash = source.source_hash;
    let source_key = SourceUnitCacheKey::from_source(&source).stable_hash();
    let source_bytes = source.canonical_open_buffer_bytes(&captured_input).unwrap();
    let source_storage_hash = Hash::from_bytes(blake3::derive_key(
        "mizar-frontend/source-unit-storage-side-table/v1",
        &source_bytes,
    ));
    let source_decodes = Arc::new(AtomicUsize::new(0));
    let mut source_handle = None;
    for target in [
        PublicationTarget::CurrentPackage,
        PublicationTarget::InternalOnly,
    ] {
        let decode_input = captured_input.clone();
        let decodes = source_decodes.clone();
        let phase = IrPipelinePhase::new("SourceLoad");
        let kind = OutputKind::new("SourceUnit");
        let result = publisher.publish(PublishOutputInput {
            slot: publisher.allocate(
                snapshot.id,
                phase.clone(),
                ir_unit.clone(),
                kind.clone(),
                SchemaVersion::new(1),
            ),
            snapshot: snapshot.id,
            phase,
            work_unit: ir_unit.clone(),
            output_kind: kind,
            schema_version: SchemaVersion::new(1),
            payload: source.clone(),
            canonical_payload: Some(source_key.as_bytes().to_vec()),
            storage_payload: Some(source_bytes.clone()),
            decode: BlobDecoder::new(move |bytes| {
                decodes.fetch_add(1, Ordering::SeqCst);
                SourceUnit::from_canonical_storage_bytes(bytes, source_id, &decode_input)
                    .filter(|source| source.source_hash == expected_hash)
                    .ok_or_else(|| BlobDecodeError::new("invalid native OpenBuffer source"))
            }),
            parents: vec![],
            named_input_hashes: vec![NamedInputHash {
                name: "source".into(),
                domain: SOURCE_UNIT_CACHE_KEY_VERSION.into(),
                digest: source_key,
            }],
            side_tables: IrSideTables {
                source_maps: vec![SideTableRecord::new(
                    "source-storage",
                    source.normalized_path.as_str(),
                    source_storage_hash,
                )],
                ..IrSideTables::default()
            },
            origin: OutputOrigin::OpenBuffer,
            target,
        });
        if target == PublicationTarget::CurrentPackage {
            assert!(matches!(result, Err(PublishError::OpenBufferOutput { .. })));
        } else {
            source_handle = Some(result.unwrap());
        }
    }
    let source_handle = source_handle.unwrap();
    let source_parent = SealedParentOutputHandle::from_published_output(
        &publisher,
        snapshot.id,
        source_handle.erase(),
    )
    .unwrap();
    let output_bytes = output.canonical_open_buffer_bytes(&captured_input).unwrap();
    let mut semantic_bytes = b"mizar-frontend/output-publication/v1\0".to_vec();
    semantic_bytes.extend_from_slice(
        output
            .cache_keys
            .ast
            .as_ref()
            .unwrap()
            .stable_hash()
            .as_bytes(),
    );
    let output_storage_hash = Hash::from_bytes(blake3::derive_key(
        "mizar-frontend/output-storage-side-table/v1",
        &output_bytes,
    ));
    let output_decodes = Arc::new(AtomicUsize::new(0));
    let decode_input = captured_input;
    let decodes = output_decodes.clone();
    let phase = IrPipelinePhase::new("Frontend");
    let kind = OutputKind::new("FrontendOutput");
    let output_handle = publisher
        .publish(PublishOutputInput {
            slot: publisher.allocate(
                snapshot.id,
                phase.clone(),
                ir_unit.clone(),
                kind.clone(),
                SchemaVersion::new(1),
            ),
            snapshot: snapshot.id,
            phase,
            work_unit: ir_unit,
            output_kind: kind,
            schema_version: SchemaVersion::new(1),
            payload: output.clone(),
            canonical_payload: Some(semantic_bytes),
            storage_payload: Some(output_bytes),
            decode: BlobDecoder::new(move |bytes| {
                decodes.fetch_add(1, Ordering::SeqCst);
                FrontendOutput::from_canonical_storage_bytes(bytes, source_id, &decode_input)
                    .filter(|output| output.source.source_hash == expected_hash)
                    .ok_or_else(|| BlobDecodeError::new("invalid native OpenBuffer frontend"))
            }),
            parents: vec![source_parent.into_output_ref()],
            named_input_hashes: vec![
                NamedInputHash {
                    name: "active-lexical-environment".into(),
                    domain: ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION.into(),
                    digest: output.cache_keys.active_lexical_environment.stable_hash(),
                },
                NamedInputHash {
                    name: "tokens".into(),
                    domain: TOKEN_STREAM_CACHE_KEY_VERSION.into(),
                    digest: output.cache_keys.tokens.stable_hash(),
                },
            ],
            side_tables: IrSideTables {
                source_maps: vec![SideTableRecord::new(
                    "frontend-storage",
                    source.normalized_path.as_str(),
                    output_storage_hash,
                )],
                ..IrSideTables::default()
            },
            origin: OutputOrigin::OpenBuffer,
            target: PublicationTarget::InternalOnly,
        })
        .unwrap();
    for handle in [source_handle.erase(), output_handle.erase()] {
        assert!(matches!(handle.placement(), StoragePlacement::Blob { .. }));
        assert!(matches!(
            publisher.validate_current_output(snapshot.id, &handle),
            Err(PublishError::OutputNotCurrentPackage { .. })
        ));
        assert!(
            SealedParentOutputHandle::from_current_output(&publisher, snapshot.id, handle.clone())
                .is_err()
        );
        assert!(
            SealedParentOutputHandle::from_published_output(&publisher, snapshot.id, handle)
                .is_ok()
        );
    }
    assert_eq!(source_decodes.load(Ordering::SeqCst), 0);
    assert_eq!(output_decodes.load(Ordering::SeqCst), 0);
    fixture.write(b"changed disk source");
    fs::remove_file(package_root.join("src/main.miz")).unwrap();
    assert_eq!(*publisher.storage().get(&source_handle).unwrap(), source);
    assert_eq!(*publisher.storage().get(&output_handle).unwrap(), output);
    assert_eq!(source_decodes.load(Ordering::SeqCst), 1);
    assert_eq!(output_decodes.load(Ordering::SeqCst), 1);
}

#[test]
fn native_open_edits_match_independent_clean_real_prefixes_across_replay_controls() {
    let fixture = Fixture::new(b"definition\nend;\n");
    let leaf =
        "definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n";
    fs::write(fixture.root.join("alpha/src/leaf.miz"), leaf).unwrap();
    let mut retained = None;
    let mut baselines = Vec::new();
    let mut canonical_ids: Vec<Vec<SourceId>> = Vec::new();
    for (stage, workers, order, reverse) in [
        (0, 1, CompletionOrder::Canonical, false),
        (0, 1, CompletionOrder::Canonical, false),
        (0, 4, CompletionOrder::Reverse, false),
        (0, 4, CompletionOrder::Canonical, true),
        (1, 1, CompletionOrder::Canonical, false),
        (1, 1, CompletionOrder::Canonical, false),
        (1, 4, CompletionOrder::Reverse, false),
        (1, 4, CompletionOrder::Canonical, true),
        (2, 1, CompletionOrder::Canonical, false),
        (2, 1, CompletionOrder::Canonical, false),
        (2, 4, CompletionOrder::Reverse, false),
        (2, 4, CompletionOrder::Canonical, true),
    ] {
        let first = stage == baselines.len();
        let main = match stage {
            0 => "import alpha.leaf;\ntheorem T: a combine b = a;\n",
            1 => "import alpha.leaf;\ntheorem EditedT: a combine b = b;\n",
            _ => "import alpha.leaf;\ntheorem T: ;\n",
        };
        let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
        request.source_inputs.versions.clear();
        request.generation = BuildRequestGeneration::new(stage as u64);
        input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
            package_id: PackageId::new("alpha"),
            files: vec![
                WorkspaceSourceFile::new("src/main.miz", "main.miz"),
                WorkspaceSourceFile::new("src/leaf.miz", "leaf.miz"),
            ],
        }]);
        input.dependency_overlay = ModuleDependencyOverlay::unavailable();
        input.worker_count = workers;
        input.completion_order = order;
        let mut descriptors = ["leaf", "main"]
            .map(|module| {
                let path = format!("src/{module}.miz");
                let version = if module == "main" {
                    stage as i64 + 7
                } else {
                    7
                };
                (
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    SourceInput {
                        package_id: PackageId::new("alpha"),
                        module_path: ModulePath::new(module),
                        normalized_path: normalize_path(
                            &fixture.root.join("alpha"),
                            Path::new(&path),
                        )
                        .unwrap(),
                        edition: Edition::new("2025"),
                        origin: SourceOriginInput::OpenBuffer {
                            uri: format!(
                                "file://{}",
                                fixture.root.join("alpha").join(&path).display()
                            ),
                            expected_version: version,
                            actual_version: version,
                            text: std::sync::Arc::from(format!(
                                "\u{feff}{}",
                                if module == "main" { main } else { leaf }.replace('\n', "\r\n")
                            )),
                        },
                    },
                )
            })
            .to_vec();
        if reverse {
            descriptors.reverse();
        }
        let (mut driver, ids, snapshots, previous) = if first && stage > 0 {
            let (driver, ids, snapshots, owner, old_snapshot, output, unused) =
                retained.take().unwrap();
            (
                driver,
                ids,
                snapshots,
                Some((owner, old_snapshot, output, unused)),
            )
        } else {
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            (
                CompilerDriver::new(builder.build().unwrap()),
                InMemorySessionIdAllocator::new(),
                SnapshotRegistry::new(),
                None,
            )
        };
        let session = request
            .clone()
            .capture_sources(&ids, &snapshots, descriptors.clone())
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        let versions = &session.captured().snapshot().source_versions;
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        let owner = publisher(snapshot, 0, &unit);
        for module in ["leaf", "main"] {
            let unit = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("alpha"),
                    ModulePath::new(module),
                ),
            };
            owner.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("SourceLoad"),
                OutputKind::new("SourceUnit"),
                IrWorkUnit::new(format!("{:?}:{:?}", "alpha", module)),
            ));
            allow_frontend(&owner, &unit);
        }
        driver = driver.with_output_publisher(owner.clone());
        let mut obsolete = None;
        if let Some((old_owner, old_snapshot, old_output, unused)) = previous {
            let old_owner: std::sync::Arc<PhaseOutputPublisher> = old_owner;
            let old_output: AnyPhaseOutputRef = old_output;
            let unused: mizar_driver::request::BuildSession = unused;
            assert_ne!(snapshot, old_snapshot);
            old_owner
                .replace_current_snapshot(old_snapshot, snapshot)
                .unwrap();
            assert!(old_owner.validate_current_snapshot(old_snapshot).is_err());
            for target in [old_snapshot, snapshot] {
                assert!(
                    old_owner
                        .validate_published_output(target, &old_output)
                        .is_err()
                );
                assert!(
                    SealedParentOutputHandle::from_published_output(
                        &old_owner,
                        target,
                        old_output.clone()
                    )
                    .is_err()
                );
            }
            assert_eq!(unused.state(), BuildSessionState::SnapshotCaptured);
            assert!(driver.session(unused.id()).is_none());
            assert!(snapshots.is_current_for_request(old_snapshot, unused.request().id));
            let (_, obsolete_input) = fixture.submit_request(vec![], vec![]);
            // Submit below, after the edited generation has become current in the same Driver.
            obsolete = Some((unused, obsolete_input));
        }
        if first {
            canonical_ids.push(
                ["leaf", "main"]
                    .map(|module| {
                        versions
                            .iter()
                            .find(|version| version.module_path.as_str() == module)
                            .unwrap()
                            .source_id
                    })
                    .to_vec(),
            );
        }
        let submission = driver
            .submit_captured_with_import_discovery(session, &ids, &snapshots, input)
            .unwrap();
        assert_eq!(
            submission.status,
            if stage == 2 {
                DriverSubmissionStatus::SchedulerValidated
            } else {
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            }
        );
        assert_eq!(
            submission.session.state(),
            BuildSessionState::Finished(if stage == 2 {
                BuildSessionOutcome::Failed
            } else {
                BuildSessionOutcome::Blocked
            })
        );
        let snapshot = submission.session.captured().snapshot();
        let graph = submission.task_graph.as_ref().unwrap();
        let run = submission.scheduler_run.as_ref().unwrap();
        for task in graph.tasks().iter().filter(|task| {
            task.phases.iter().any(|phase| {
                !matches!(
                    phase,
                    PipelinePhase::PackageResolve
                        | PipelinePhase::SourceLoad
                        | PipelinePhase::Frontend
                )
            })
        }) {
            assert_eq!(
                run.task_states
                    .iter()
                    .find(|state| state.task_id == task.id)
                    .unwrap()
                    .state,
                TaskState::Blocked
            );
        }
        let mut projection = Vec::new();
        for (ordinal, module) in ["leaf", "main"].into_iter().enumerate() {
            let source_task = graph.tasks().iter().find(|task| task.kind == TaskKind::SourceLoad && matches!(&task.unit, WorkUnit::Module { module: id } if id.path.as_str() == module)).unwrap();
            let frontend_task = graph
                .tasks()
                .iter()
                .find(|task| task.kind == TaskKind::Frontend && task.unit == source_task.unit)
                .unwrap();
            assert_eq!(run.phase_results[&source_task.id].len(), 1);
            assert_eq!(run.phase_results[&frontend_task.id].len(), 1);
            let source_result = &run.phase_results[&source_task.id][0];
            let frontend_result = &run.phase_results[&frontend_task.id][0];
            let diagnostic_case = stage == 2 && module == "main";
            assert_eq!(source_result.output_refs.len(), 1);
            assert_eq!(
                frontend_result.output_refs.len(),
                usize::from(!diagnostic_case)
            );
            assert_eq!(source_result.status, PhaseStatus::Complete);
            assert_eq!(
                frontend_result.status,
                if diagnostic_case {
                    PhaseStatus::Recoverable
                } else {
                    PhaseStatus::Complete
                }
            );
            let descriptor = &descriptors
                .iter()
                .find(|(_, input)| input.module_path.as_str() == module)
                .unwrap()
                .1;
            let version = snapshot
                .source_versions
                .iter()
                .find(|version| version.module_path.as_str() == module)
                .unwrap();
            let source_ref = &source_result.output_refs[0];
            assert!(matches!(
                source_ref.placement(),
                StoragePlacement::Blob { .. }
            ));
            assert!(source_ref.lineage().parents.is_empty());
            let source = owner
                .storage()
                .get(
                    &owner
                        .storage()
                        .typed_handle::<SourceUnit>(source_ref, &OutputKind::new("SourceUnit"))
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(source.source_id, version.source_id);
            assert_eq!(source.source_hash, version.source_hash);
            assert_eq!(source.origin, version.origin);
            assert_eq!(
                source.source_text.as_ref(),
                if module == "main" { main } else { leaf }
            );
            let source_bytes = source.canonical_open_buffer_bytes(descriptor).unwrap();
            let cache_intents = [PipelinePhase::SourceLoad, PipelinePhase::Frontend].map(|phase| {
                let parents = if phase == PipelinePhase::Frontend {
                    frontend_task
                        .dependencies
                        .iter()
                        .flat_map(|id| run.phase_results.get(id).into_iter().flatten())
                        .filter(|result| result.status == PhaseStatus::Complete)
                        .flat_map(|result| &result.output_refs)
                        .map(|parent| {
                            SealedParentOutputHandle::from_published_output(
                                &owner,
                                snapshot.id,
                                parent.clone(),
                            )
                            .unwrap()
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                let input = PhaseInput::new(
                    source_task.unit.clone(),
                    PhaseDispatchInputBundle::new(
                        snapshot.id,
                        SourceUnitCacheKey::from_source(&source).stable_hash(),
                        vec![],
                        parents,
                    )
                    .unwrap(),
                );
                let intent = driver
                    .registry()
                    .cache_key_for_phase(phase, &input)
                    .unwrap()
                    .intent;
                assert!(
                    matches!(&intent, PhaseCacheIntent::NoKey { reason } if !reason.is_empty())
                );
                intent
            });
            let frontend = frontend_result.output_refs.first().map(|handle| {
                assert!(matches!(handle.placement(), StoragePlacement::Blob { .. }));
                let output = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                handle,
                                &OutputKind::new("FrontendOutput"),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                assert_eq!(output.source, *source);
                let parents = handle
                    .lineage()
                    .parents
                    .iter()
                    .map(|parent| {
                        let parent = owner.registry().output_lineage(*parent).unwrap();
                        (parent.phase, parent.work_unit, parent.output_kind)
                    })
                    .collect::<Vec<_>>();
                (
                    output.canonical_open_buffer_bytes(descriptor).unwrap(),
                    output.cache_keys.canonical_bytes().unwrap(),
                    parents,
                )
            });
            let diagnostics = source_result
                .diagnostics
                .iter()
                .chain(&frontend_result.diagnostics)
                .flat_map(|batch| batch.drafts())
                .map(|draft| {
                    assert_eq!(draft.source_snapshot(), snapshot.id);
                    let DiagnosticPrimaryLocation::Span(span) = draft.primary_location() else {
                        panic!("real parser span");
                    };
                    let SourceAnchor::Range(mut range) = *span.anchor() else {
                        panic!("real parser range");
                    };
                    source.line_map.validate_range(range).unwrap();
                    assert_eq!(range.source_id, version.source_id);
                    range.source_id = canonical_ids[stage][ordinal];
                    assert!(draft.secondary_spans().is_empty());
                    assert!(draft.notes().iter().all(|note| note.span().is_none()));
                    assert!(
                        draft
                            .details()
                            .entries()
                            .values()
                            .all(|value| matches!(value, DiagnosticDetailValue::String(_)))
                    );
                    assert!(draft.fixes().is_empty());
                    assert!(draft.explanation().is_none());
                    (
                        draft.source_snapshot(),
                        draft.code(),
                        draft.phase(),
                        draft.category(),
                        draft.stable_detail_key().to_owned(),
                        draft.message().to_owned(),
                        DiagnosticSpan::from_anchor(
                            SourceAnchor::Range(range),
                            span.role(),
                            span.label().map(str::to_owned),
                            span.freshness(),
                            span.zero_width(),
                        )
                        .unwrap(),
                        draft.notes().to_vec(),
                        draft.details().clone(),
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(diagnostics.is_empty(), !diagnostic_case);
            projection.push((
                module,
                snapshot.id,
                source_bytes,
                SourceUnitCacheKey::from_source(&source).stable_hash(),
                frontend,
                (source_result.status, frontend_result.status),
                diagnostics,
                cache_intents,
            ));
        }
        if let Some(expected) = baselines.get(stage) {
            assert_eq!(
                &projection, expected,
                "stage={stage}, workers={workers}, order={order:?}, reverse={reverse}"
            );
        } else {
            baselines.push(projection);
            if stage > 0 {
                assert_ne!(baselines[stage][1].1, baselines[stage - 1][1].1);
                assert_ne!(baselines[stage][1].2, baselines[stage - 1][1].2);
                if stage == 1 {
                    assert_ne!(baselines[stage][1].4, baselines[stage - 1][1].4);
                }
                let (unused, obsolete_input) = obsolete.take().unwrap();
                let obsolete = driver
                    .submit_captured(unused, &ids, &snapshots, obsolete_input)
                    .unwrap();
                assert_eq!(
                    obsolete.status,
                    DriverSubmissionStatus::SupersededBeforeSubmission
                );
                assert_eq!(
                    obsolete.session.state(),
                    BuildSessionState::Finished(BuildSessionOutcome::Superseded)
                );
                assert!(obsolete.scheduler_run.is_none());
                assert!(matches!(
                    obsolete.publication_decision,
                    mizar_driver::request::PublicationDecision::Suppressed(_)
                ));
            }
            let unused = request
                .capture_sources(&ids, &snapshots, descriptors)
                .unwrap();
            let output = run
                .phase_results
                .values()
                .flatten()
                .flat_map(|result| &result.output_refs)
                .find(|handle| handle.output_kind() == &OutputKind::new("SourceUnit"))
                .unwrap()
                .clone();
            retained = Some((driver, ids, snapshots, owner, snapshot.id, output, unused));
        }
    }
}

#[test]
fn native_open_buffer_input_feeds_protected_prefix_like_explicit_clean_input() {
    let fixture = Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes());
    let package_root = fixture.root.join("nested/member");
    fs::create_dir_all(package_root.join("src/groups")).unwrap();
    fs::write(package_root.join("src/groups/base.miz"), [0xff]).unwrap();
    let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
    input.workspace_packages[0].member_path = "nested/member".into();
    input.lockfile = parse_lockfile("schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"nested/member\" }\ndependencies = []\n").unwrap();
    let plan = produce_build_plan(
        input.plan_request.clone(),
        input.workspace_packages.clone(),
        input.lockfile.clone(),
    )
    .unwrap();
    input.source_layout = StaticSourceLayout::from_native_disk(&plan).unwrap();
    let index = build_module_index(&plan, &input.source_layout, &[]).unwrap();
    let loader = DiskSourceLoader::new(&package_root);
    let raw: std::sync::Arc<str> = "\u{feff}definition\r\nend;\r\n".into();
    let origin = SourceOriginInput::OpenBuffer {
        uri: format!(
            "file://localhost{}/src/groups/%62ase.miz",
            package_root.display()
        ),
        expected_version: 7,
        actual_version: 7,
        text: raw.clone(),
    };
    let acquired = index
        .native_open_buffer_input(
            &plan.workspace_root,
            &PackageId::new("alpha"),
            &loader,
            origin.clone(),
        )
        .unwrap();
    let explicit = SourceInput {
        package_id: PackageId::new("alpha"),
        module_path: ModulePath::new("groups.base"),
        normalized_path: normalize_path(&package_root, Path::new("src/groups/base.miz")).unwrap(),
        edition: Edition::new("2025"),
        origin,
    };
    assert_eq!(acquired, explicit);
    let SourceOriginInput::OpenBuffer { text, .. } = &acquired.origin else {
        panic!("open buffer")
    };
    assert!(std::sync::Arc::ptr_eq(text, &raw));
    request.source_inputs.versions.clear();
    let mut clean = None;
    for descriptor in [explicit, acquired] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let session = request
            .clone()
            .capture_sources(&ids, &snapshots, vec![(loader.clone(), descriptor)])
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("groups.base"),
            ),
        };
        let owner = publisher(snapshot, 0, &unit);
        allow_frontend(&owner, &unit);
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
        let (_, mut current) = fixture.submit_request(vec![], vec![]);
        current.workspace_packages = input.workspace_packages.clone();
        current.lockfile = input.lockfile.clone();
        current.source_layout = input.source_layout.clone();
        let submission = driver
            .submit_captured(session.clone(), &ids, &snapshots, current)
            .unwrap();
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        let results = submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .collect::<Vec<_>>();
        let refs = results
            .iter()
            .flat_map(|result| &result.output_refs)
            .collect::<Vec<_>>();
        assert_eq!(refs.len(), 2);
        assert!(results.iter().all(|result| result.diagnostics.is_empty()));
        let source_ref = *refs
            .iter()
            .find(|output| output.output_kind() == &OutputKind::new("SourceUnit"))
            .unwrap();
        let source = owner
            .storage()
            .get(
                &owner
                    .storage()
                    .typed_handle::<SourceUnit>(source_ref, &OutputKind::new("SourceUnit"))
                    .unwrap(),
            )
            .unwrap();
        let frontend_ref = *refs
            .iter()
            .find(|output| output.output_kind() == &OutputKind::new("FrontendOutput"))
            .unwrap();
        let frontend = owner
            .storage()
            .get(
                &owner
                    .storage()
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        frontend_ref,
                        &OutputKind::new("FrontendOutput"),
                    )
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(source.source_text.as_ref(), "definition\nend;\n");
        let map = source.loading_map.as_ref().unwrap();
        assert_eq!(map.segments, fixture.loading_map.as_ref().unwrap().segments);
        assert_eq!(
            map.origin,
            mizar_session::LoadingOrigin::OpenBufferText {
                uri: format!(
                    "file://localhost{}/src/groups/%62ase.miz",
                    package_root.display()
                ),
                version: 7,
            }
        );
        assert_eq!(source.line_map, fixture.line_map);
        assert_eq!(source.origin, SourceOrigin::OpenBuffer { version: 7 });
        assert_eq!(
            source.source_id,
            session.captured().snapshot().source_versions[0].source_id
        );
        let phase_input = PhaseInput::new(
            unit,
            PhaseDispatchInputBundle::without_parent_outputs(
                snapshot,
                SourceUnitCacheKey::from_source(&source).stable_hash(),
                vec![],
            ),
        );
        for phase in [PipelinePhase::SourceLoad, PipelinePhase::Frontend] {
            assert!(matches!(
                driver
                    .registry()
                    .cache_key_for_phase(phase, &phase_input)
                    .unwrap()
                    .intent,
                PhaseCacheIntent::NoKey { .. }
            ));
        }
        let actual = (
            submission.module_index.unwrap(),
            source.as_ref().clone(),
            frontend.as_ref().clone(),
        );
        if let Some(expected) = &clean {
            assert_eq!(&actual, expected);
        } else {
            clean = Some(actual);
        }
    }
}
