use super::*;

#[test]
fn native_disk_cache_request_projects_real_storage_and_replays_without_local_ids() {
    use mizar_cache::cache_key::{
        CACHE_KEY_SCHEMA_VERSION, CacheKeyBuildOutcome, CacheKeyBuilder, FootprintCompleteness,
    };
    use mizar_proof::policy::VerifierPolicy;
    let policy = VerifierPolicy::release();
    let mut previous = Vec::new();
    for raw in ["\u{feff}definition\r\nend;\r\n", "definition\nend;\n"] {
        let mut replays = Vec::new();
        for replay in 0..2 {
            let fixture = Fixture::new(raw.as_bytes());
            let ids = InMemorySessionIdAllocator::new();
            let source_ids = InMemorySessionIdAllocator::new();
            for _ in 0..replay {
                source_ids.next_source_id(snapshot_id(0x62)).unwrap();
            }
            let snapshots = SnapshotRegistry::with_allocator(source_ids);
            let (mut draft, input) = fixture.submit_request(vec![], vec![]);
            draft.source_inputs.versions.clear();
            let session = draft
                .capture_sources(
                    &ids,
                    &snapshots,
                    vec![(
                        DiskSourceLoader::new(fixture.root.join("alpha")),
                        SourceInput {
                            package_id: fixture.version.package_id.clone(),
                            module_path: fixture.version.module_path.clone(),
                            normalized_path: fixture.version.normalized_path.clone(),
                            edition: fixture.version.edition.clone(),
                            origin: SourceOriginInput::Disk {
                                path: "src/main.miz".into(),
                            },
                        },
                    )],
                )
                .unwrap();
            let version = session.captured().snapshot().source_versions[0].clone();
            let unit = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    version.package_id.clone(),
                    version.module_path.clone(),
                ),
            };
            let owner = publisher(session.captured().snapshot().id, 0, &unit);
            allow_frontend(&owner, &unit);
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            let mut driver =
                CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
            fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
            let submission = driver
                .submit_captured(session, &ids, &snapshots, input)
                .unwrap();
            let captured = submission.session.captured();
            let phase_input = PhaseInput::new(
                unit,
                PhaseDispatchInputBundle::without_parent_outputs(
                    captured.snapshot().id,
                    source_key(&version),
                    vec![],
                ),
            );
            let request = SourceLoadInputs {
                snapshot: captured.snapshot(),
                captured: Some(captured),
                build_plan: submission.build_plan.as_ref().unwrap(),
                module_index: submission.module_index.as_ref().unwrap(),
                allocator: &ids,
            }
            .prepare_native_disk_cache_request(&phase_input, &policy)
            .unwrap();
            let result = submission
                .scheduler_run
                .as_ref()
                .unwrap()
                .phase_results
                .values()
                .flatten()
                .find(|result| {
                    result
                        .output_refs
                        .iter()
                        .any(|handle| handle.output_kind() == &OutputKind::new("SourceUnit"))
                })
                .unwrap();
            assert_eq!(result.status, PhaseStatus::Complete);
            let handle = &result.output_refs[0];
            let source = owner
                .storage()
                .get(
                    &owner
                        .storage()
                        .typed_handle::<SourceUnit>(handle, &OutputKind::new("SourceUnit"))
                        .unwrap(),
                )
                .unwrap();
            let side = owner.storage().side_tables_by_ref(handle).unwrap();
            assert_eq!(request.phase.as_str(), handle.phase().as_str());
            assert_eq!(request.work_unit.as_str(), handle.work_unit().as_str());
            let identity = request.source_identity.as_ref().unwrap();
            assert_eq!(identity.package_id, source.package_id);
            assert_eq!(identity.module_path, source.module_path);
            assert_eq!(identity.normalized_source_path, source.normalized_path);
            assert_eq!(identity.source_hash, source.source_hash);
            assert_eq!(identity.language_edition, source.edition);
            assert_eq!(request.input_hashes.len(), 3);
            assert_eq!(request.input_hashes[0].name, "source");
            assert_eq!(
                request.input_hashes[0].domain,
                SOURCE_UNIT_CACHE_KEY_VERSION
            );
            assert_eq!(
                request.input_hashes[0].digest,
                SourceUnitCacheKey::from_source(&source).stable_hash()
            );
            assert_eq!(request.input_hashes[1].name, "source-storage");
            assert_eq!(
                request.input_hashes[1].domain,
                "mizar-frontend/source-unit-storage-side-table/v1"
            );
            assert_eq!(request.input_hashes[1].digest, side.source_maps[0].digest);
            assert_eq!(request.input_hashes[2].name, "captured-lockfile");
            assert_eq!(
                request.input_hashes[2].domain,
                "mizar-driver/captured-lockfile-identity/v1"
            );
            assert_eq!(
                request.input_hashes[2].digest,
                captured.snapshot().lockfile_hash
            );
            assert_eq!(
                request.config_hash,
                captured.snapshot().verifier_config_hash
            );
            assert_eq!(
                request.policy_fingerprint.hash(),
                policy.policy_fingerprint().hash()
            );
            assert_ne!(request.config_hash, request.policy_fingerprint.hash());
            assert_eq!(
                request.cache_schema_version.as_str(),
                CACHE_KEY_SCHEMA_VERSION
            );
            let descriptor = driver
                .registry()
                .descriptor_for_phase(PipelinePhase::SourceLoad)
                .unwrap();
            assert_eq!(
                request
                    .schema_versions
                    .iter()
                    .map(|entry| (
                        entry.schema_family.as_str(),
                        entry.name.as_str(),
                        entry.version.as_str(),
                    ))
                    .collect::<Vec<_>>(),
                vec![
                    (
                        "phase-output",
                        "SourceUnit",
                        descriptor.schema_version.as_str()
                    ),
                    ("source-key", "SourceUnit", SOURCE_UNIT_CACHE_KEY_VERSION),
                    ("source-storage", "Disk", "mizar-frontend-source-disk-v1"),
                ]
            );
            let validation = &request.validation_inputs;
            assert_eq!(
                validation.cache_schema_compatibility.as_str(),
                CACHE_KEY_SCHEMA_VERSION
            );
            assert_eq!(validation.toolchain_compatibility.len(), 1);
            assert_eq!(
                validation.toolchain_compatibility[0].family,
                "mizar-session/toolchain"
            );
            assert_eq!(validation.toolchain_compatibility[0].field_name, "identity");
            assert_eq!(
                validation.toolchain_compatibility[0].value,
                captured.snapshot().toolchain.identity()
            );
            assert_eq!(
                validation
                    .policy_compatibility
                    .iter()
                    .map(|field| (
                        field.family.as_str(),
                        field.field_name.as_str(),
                        field.value.clone(),
                    ))
                    .collect::<Vec<_>>(),
                vec![
                    (
                        "mizar-proof/verifier-policy",
                        "schema_version",
                        policy.schema_version().to_string()
                    ),
                    (
                        "mizar-proof/verifier-policy",
                        "profile_id",
                        policy.profile_id().to_owned()
                    ),
                    (
                        "mizar-proof/verifier-policy",
                        "checker_schema_version",
                        policy.checker_schema_version().to_string()
                    ),
                    (
                        "mizar-proof/verifier-policy",
                        "fingerprint",
                        policy
                            .policy_fingerprint()
                            .hash()
                            .as_bytes()
                            .iter()
                            .map(|byte| format!("{byte:02x}"))
                            .collect::<String>()
                    ),
                ]
            );
            assert_eq!(
                validation.footprint_completeness,
                FootprintCompleteness::Complete
            );
            assert!(!validation.uncacheable);
            assert!(request.dependency_hashes.is_empty() && request.dependency_slices.is_empty());
            assert!(validation.dependency_artifacts.is_empty());
            assert!(validation.dependency_slice_fingerprints.is_empty());
            assert!(validation.proof_reuse_schema_versions.is_empty());
            assert!(validation.proof_reuse_evidence_identities.is_empty());
            assert!(validation.diagnostic_refs.is_empty());
            assert_eq!(validation.canonical_vc_fingerprint, None);
            assert_eq!(validation.local_context_fingerprint, None);
            assert_eq!(validation.obligation_anchor_fingerprint, None);
            assert_eq!(validation.selected_proof_witness_hash, None);
            assert_eq!(validation.deterministic_discharge_hash, None);
            assert_eq!(validation.proof_reuse_validation_hash, None);
            assert_eq!(
                driver
                    .registry()
                    .cache_key_for_phase(PipelinePhase::SourceLoad, &phase_input)
                    .unwrap()
                    .intent,
                PhaseCacheIntent::NoKey {
                    reason: "current disk source maps require a real source load".into(),
                }
            );
            let CacheKeyBuildOutcome::Cacheable(key) =
                CacheKeyBuilder::new(request.clone()).build()
            else {
                panic!("bound source-only request must construct a cacheable key");
            };
            replays.push((request, key.final_hash, source.source_id));
        }
        assert_eq!(replays[0].0, replays[1].0);
        assert_eq!(replays[0].1, replays[1].1);
        assert_ne!(replays[0].2, replays[1].2);
        previous.push(replays.remove(0));
    }
    assert_eq!(previous[0].0.source_identity, previous[1].0.source_identity);
    assert_eq!(previous[0].0.input_hashes[0], previous[1].0.input_hashes[0]);
    assert_ne!(previous[0].0.input_hashes[1], previous[1].0.input_hashes[1]);
    assert_ne!(previous[0].1, previous[1].1);
}

#[test]
fn native_disk_cache_request_keeps_captured_and_explicit_policy_identities_distinct() {
    use mizar_cache::cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder};
    use mizar_proof::policy::VerifierPolicy;
    let fixture = Fixture::new(b"definition\nend;\n");
    let mut baseline = None;
    for mode in [
        "baseline",
        "text",
        "config",
        "lock",
        "toolchain",
        "profile",
        "settings",
    ] {
        fixture.write(if mode == "text" {
            b"definition\n\nend;\n"
        } else {
            b"definition\nend;\n"
        });
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut draft, mut input) = fixture.submit_request(vec![], vec![]);
        draft.source_inputs.versions.clear();
        if mode == "config" {
            draft.verifier_config = VerifierConfigInput::new(hash(3));
        }
        if mode == "lock" {
            draft.dependency_inputs.lockfile_hash = hash(4);
        }
        if mode == "toolchain" {
            draft.dependency_inputs.toolchain = ToolchainInfo::new("different-toolchain");
            input.plan_request.toolchain = draft.dependency_inputs.toolchain.clone();
        }
        let session = draft
            .capture_sources(
                &ids,
                &snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    SourceInput {
                        package_id: fixture.version.package_id.clone(),
                        module_path: fixture.version.module_path.clone(),
                        normalized_path: fixture.version.normalized_path.clone(),
                        edition: fixture.version.edition.clone(),
                        origin: SourceOriginInput::Disk {
                            path: "src/main.miz".into(),
                        },
                    },
                )],
            )
            .unwrap();
        let mut driver = CompilerDriver::new(source_registry());
        let submission = driver
            .submit_captured(session, &ids, &snapshots, input)
            .unwrap();
        let captured = submission.session.captured();
        let phase_input = PhaseInput::new(
            source_task(&submission).unit.clone(),
            PhaseDispatchInputBundle::without_parent_outputs(
                captured.snapshot().id,
                source_key(&captured.snapshot().source_versions[0]),
                vec![],
            ),
        );
        let policy = match mode {
            "profile" => VerifierPolicy::release().with_profile_id("alternate-release"),
            "settings" => VerifierPolicy::release().with_require_kernel_certificates(false),
            _ => VerifierPolicy::release(),
        };
        let request = SourceLoadInputs {
            snapshot: captured.snapshot(),
            captured: Some(captured),
            allocator: &ids,
            build_plan: submission.build_plan.as_ref().unwrap(),
            module_index: submission.module_index.as_ref().unwrap(),
        }
        .prepare_native_disk_cache_request(&phase_input, &policy)
        .unwrap();
        let CacheKeyBuildOutcome::Cacheable(key) = CacheKeyBuilder::new(request.clone()).build()
        else {
            panic!("bound native Disk request");
        };
        if let Some((old, old_key)) = &baseline {
            assert_ne!(&request, old, "{mode}");
            assert_ne!(&key.final_hash, old_key, "{mode}");
            if mode == "config" {
                assert_ne!(request.config_hash, old.config_hash);
                assert_eq!(request.policy_fingerprint, old.policy_fingerprint);
            } else if mode == "profile" || mode == "settings" {
                assert_eq!(request.config_hash, old.config_hash);
                assert_ne!(request.policy_fingerprint, old.policy_fingerprint);
            } else {
                assert_eq!(request.config_hash, old.config_hash);
                assert_eq!(request.policy_fingerprint, old.policy_fingerprint);
            }
            match mode {
                "text" => assert_ne!(request.source_identity, old.source_identity),
                "lock" => assert_ne!(request.input_hashes[2], old.input_hashes[2]),
                "toolchain" => assert_ne!(
                    request.validation_inputs.toolchain_compatibility,
                    old.validation_inputs.toolchain_compatibility
                ),
                "settings" => assert_eq!(
                    request.validation_inputs.policy_compatibility[1],
                    old.validation_inputs.policy_compatibility[1]
                ),
                _ => {}
            }
        } else {
            baseline = Some((request, key.final_hash));
        }
    }
}

#[test]
fn native_disk_cache_request_rejects_unbound_dispatch_capture_roots_and_policy_metadata() {
    use mizar_proof::policy::{
        DEFAULT_CHECKER_SCHEMA_VERSION, POLICY_SCHEMA_VERSION, VerifierPolicy,
    };
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (mut draft, input) = fixture.submit_request(vec![], vec![]);
    draft.source_inputs.versions.clear();
    let descriptor = SourceInput {
        package_id: fixture.version.package_id.clone(),
        module_path: fixture.version.module_path.clone(),
        normalized_path: fixture.version.normalized_path.clone(),
        edition: fixture.version.edition.clone(),
        origin: SourceOriginInput::Disk {
            path: "src/main.miz".into(),
        },
    };
    let session = draft
        .clone()
        .capture_sources(
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                descriptor.clone(),
            )],
        )
        .unwrap();
    let mut foreign_draft = draft.clone();
    foreign_draft.verifier_config = VerifierConfigInput::new(hash(9));
    let foreign = foreign_draft
        .capture_sources(
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                descriptor.clone(),
            )],
        )
        .unwrap();
    let mut driver = CompilerDriver::new(source_registry());
    let submission = driver
        .submit_captured(session, &ids, &snapshots, input)
        .unwrap();
    let captured = submission.session.captured();
    let phase_input = PhaseInput::new(
        source_task(&submission).unit.clone(),
        PhaseDispatchInputBundle::without_parent_outputs(
            captured.snapshot().id,
            source_key(&captured.snapshot().source_versions[0]),
            vec![],
        ),
    );
    let inputs = SourceLoadInputs {
        snapshot: captured.snapshot(),
        captured: Some(captured),
        allocator: &ids,
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: submission.module_index.as_ref().unwrap(),
    };
    let policy = VerifierPolicy::release();
    assert!(
        inputs
            .prepare_native_disk_cache_request(&phase_input, &policy)
            .is_ok()
    );
    for rejected in [
        SourceLoadInputs {
            captured: None,
            ..inputs
        },
        SourceLoadInputs {
            captured: Some(foreign.captured()),
            ..inputs
        },
    ] {
        assert!(
            rejected
                .prepare_native_disk_cache_request(&phase_input, &policy)
                .is_err()
        );
    }
    let legacy = fixture.submit(&ids, &SnapshotRegistry::new());
    let legacy_capture = legacy.session.captured();
    let legacy_input = PhaseInput::new(
        phase_input.work_unit.clone(),
        PhaseDispatchInputBundle::without_parent_outputs(
            legacy_capture.snapshot().id,
            source_key(&legacy_capture.snapshot().source_versions[0]),
            vec![],
        ),
    );
    assert!(
        SourceLoadInputs {
            snapshot: legacy_capture.snapshot(),
            captured: Some(legacy_capture),
            build_plan: legacy.build_plan.as_ref().unwrap(),
            module_index: legacy.module_index.as_ref().unwrap(),
            allocator: &ids,
        }
        .prepare_native_disk_cache_request(&legacy_input, &policy)
        .is_err()
    );
    for unsupported in [
        policy.clone().with_profile_id(" \t\n"),
        policy
            .clone()
            .with_schema_version(POLICY_SCHEMA_VERSION + 1),
        policy
            .clone()
            .with_checker_schema_version(DEFAULT_CHECKER_SCHEMA_VERSION + 1),
    ] {
        assert!(
            inputs
                .prepare_native_disk_cache_request(&phase_input, &unsupported)
                .is_err()
        );
    }
    for mode in [
        "missing",
        "duplicate",
        "source-id",
        "source-hash",
        "origin",
        "workspace",
        "config",
        "lock",
        "toolchain",
    ] {
        let mut snapshot = captured.snapshot().clone();
        match mode {
            "missing" => snapshot.source_versions.clear(),
            "duplicate" => snapshot
                .source_versions
                .push(snapshot.source_versions[0].clone()),
            "source-id" => {
                ids.next_source_id(snapshot.id).unwrap();
                snapshot.source_versions[0].source_id = ids.next_source_id(snapshot.id).unwrap();
            }
            "source-hash" => snapshot.source_versions[0].source_hash = hash(8),
            "origin" => {
                snapshot.source_versions[0].origin = SourceOrigin::OpenBuffer { version: 1 }
            }
            "workspace" => snapshot.workspace_root = WorkspaceRoot::new("/foreign"),
            "config" => snapshot.verifier_config_hash = hash(8),
            "lock" => snapshot.lockfile_hash = hash(8),
            "toolchain" => snapshot.toolchain = ToolchainInfo::new("foreign"),
            _ => unreachable!(),
        }
        assert!(
            SourceLoadInputs {
                snapshot: &snapshot,
                ..inputs
            }
            .prepare_native_disk_cache_request(&phase_input, &policy)
            .is_err(),
            "{mode}"
        );
    }
    for mode in [
        "duplicate-package",
        "root",
        "source-root",
        "manifest",
        "edition",
        "toolchain",
    ] {
        let mut plan = inputs.build_plan.clone();
        match mode {
            "duplicate-package" => plan.packages.push(plan.packages[0].clone()),
            "root" | "source-root" | "manifest" => {
                let mizar_build::planner::PackagePlanSource::Workspace {
                    root,
                    source_root,
                    manifest_path,
                } = &mut plan.packages[0].source
                else {
                    panic!("workspace package");
                };
                match mode {
                    "root" => *root = "..".into(),
                    "source-root" => *source_root = "foreign/src".into(),
                    "manifest" => *manifest_path = "foreign/mizar.pkg".into(),
                    _ => unreachable!(),
                }
            }
            "edition" => plan.packages[0].edition = Edition::new("foreign"),
            "toolchain" => plan.toolchain = ToolchainInfo::new("foreign"),
            _ => unreachable!(),
        }
        assert!(
            SourceLoadInputs {
                build_plan: &plan,
                ..inputs
            }
            .prepare_native_disk_cache_request(&phase_input, &policy)
            .is_err(),
            "{mode}"
        );
    }
    for mode in [
        "missing-module",
        "duplicate-module",
        "duplicate-index-package",
        "path",
    ] {
        let mut index = inputs.module_index.clone();
        match mode {
            "missing-module" => index.modules.clear(),
            "duplicate-module" => index.modules.push(index.modules[0].clone()),
            "duplicate-index-package" => index.packages.push(index.packages[0].clone()),
            "path" => {
                let ModuleIndexLocation::WorkspaceFile {
                    source_relative_path,
                    ..
                } = &mut index.modules[0].location
                else {
                    panic!("workspace source");
                };
                *source_relative_path = "foreign.miz".into();
            }
            _ => unreachable!(),
        }
        assert!(
            SourceLoadInputs {
                module_index: &index,
                ..inputs
            }
            .prepare_native_disk_cache_request(&phase_input, &policy)
            .is_err(),
            "{mode}"
        );
    }
    for (snapshot, unit, digest, dependencies) in [
        (
            snapshot_id(0x71),
            phase_input.work_unit.clone(),
            source_key(&fixture.version),
            vec![],
        ),
        (
            phase_input.snapshot,
            WorkUnit::Workspace,
            source_key(&fixture.version),
            vec![],
        ),
        (
            phase_input.snapshot,
            phase_input.work_unit.clone(),
            hash(9),
            vec![],
        ),
        (
            phase_input.snapshot,
            phase_input.work_unit.clone(),
            source_key(&fixture.version),
            vec![hash(9)],
        ),
        (
            phase_input.snapshot,
            WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("foreign"),
                    ModulePath::new("main"),
                ),
            },
            source_key(&fixture.version),
            vec![],
        ),
    ] {
        let rejected = PhaseInput::new(
            unit,
            PhaseDispatchInputBundle::without_parent_outputs(snapshot, digest, dependencies),
        );
        assert!(
            inputs
                .prepare_native_disk_cache_request(&rejected, &policy)
                .is_err()
        );
    }
    let owner = publisher(phase_input.snapshot, 0, &phase_input.work_unit);
    let source_result = driver
        .registry()
        .execute_phase_with_resources(
            PipelinePhase::SourceLoad,
            phase_input.clone(),
            PhaseExecutionResources {
                output_publisher: Some(owner.clone()),
                source_load: Some(inputs),
                diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                    DiagnosticPhase::SourceLoad,
                    phase_input.snapshot,
                    "frontend.source_load",
                ))),
                ..PhaseExecutionResources::default()
            },
        )
        .unwrap()
        .result;
    assert_eq!(source_result.status, PhaseStatus::Complete);
    let parent = SealedParentOutputHandle::from_published_output(
        &owner,
        phase_input.snapshot,
        source_result.output_refs[0].clone(),
    )
    .unwrap();
    let rejected = PhaseInput::new(
        phase_input.work_unit.clone(),
        PhaseDispatchInputBundle::new(
            phase_input.snapshot,
            source_key(&captured.snapshot().source_versions[0]),
            vec![],
            vec![parent],
        )
        .unwrap(),
    );
    assert!(
        inputs
            .prepare_native_disk_cache_request(&rejected, &policy)
            .is_err()
    );
    for origin in [
        SourceOriginInput::OpenBuffer {
            uri: format!(
                "file://{}",
                fixture.root.join("alpha/src/main.miz").display()
            ),
            expected_version: 1,
            actual_version: 1,
            text: std::sync::Arc::from("definition\nend;\n"),
        },
        SourceOriginInput::Generated {
            generator: mizar_session::GeneratedSourceKind::new("cache-request-test"),
            text: std::sync::Arc::from("definition\nend;\n"),
            anchor: None,
        },
    ] {
        let native = draft
            .clone()
            .capture_sources(
                &ids,
                &snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    SourceInput {
                        origin,
                        ..descriptor.clone()
                    },
                )],
            )
            .unwrap();
        let snapshot = native.captured().snapshot();
        let rejected = PhaseInput::new(
            phase_input.work_unit.clone(),
            PhaseDispatchInputBundle::without_parent_outputs(
                snapshot.id,
                source_key(&snapshot.source_versions[0]),
                vec![],
            ),
        );
        assert!(
            SourceLoadInputs {
                snapshot,
                captured: Some(native.captured()),
                ..inputs
            }
            .prepare_native_disk_cache_request(&rejected, &policy)
            .is_err()
        );
    }
}

#[test]
fn native_disk_cache_record_resident_round_trips_real_store_with_owner_bytes() {
    use mizar_cache::{
        cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder},
        cache_store::{CacheInsertOutcome, CacheLookupOutcome, CacheStoreRoot},
    };
    use mizar_ir::cache_adapter::{
        CacheAdapterCacheability, EncodeCacheRecordInput, EncodeCacheRecordOutcome,
    };
    let fixture = Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (driver, submission, owner, descriptor, handle) =
        native_disk_record_submission(&fixture, &ids, &snapshots, usize::MAX);
    let captured = submission.session.captured();
    let version = captured.snapshot().source_versions[0].clone();
    let snapshot = captured.snapshot().id;
    let unit = WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            version.package_id.clone(),
            version.module_path.clone(),
        ),
    };
    let phase_input = PhaseInput::new(
        unit.clone(),
        PhaseDispatchInputBundle::without_parent_outputs(snapshot, source_key(&version), vec![]),
    );
    let inputs = SourceLoadInputs {
        snapshot: captured.snapshot(),
        captured: Some(captured),
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: submission.module_index.as_ref().unwrap(),
        allocator: &ids,
    };
    let source = owner.storage().get(&handle).unwrap();
    let adapter = mizar_ir::cache_adapter::IrCacheAdapter::new(owner.clone());
    let policy = mizar_proof::policy::VerifierPolicy::release();

    assert_eq!(handle.any().placement(), &StoragePlacement::Resident);
    assert_eq!(
        source.line_map,
        LineMap::new(source.source_id, "definition\nend;\n")
    );
    assert_eq!(source.loading_map, fixture.loading_map);
    assert_eq!(source.source_id, version.source_id);
    let request = inputs
        .prepare_native_disk_cache_request(&phase_input, &policy)
        .unwrap();
    let CacheKeyBuildOutcome::Cacheable(key) = CacheKeyBuilder::new(request.clone()).build() else {
        panic!("genuine source-only key");
    };
    let EncodeCacheRecordOutcome::Encoded(record) = inputs
        .encode_native_disk_cache_record(&phase_input, &policy, &adapter, handle.clone())
        .unwrap()
    else {
        panic!("bound resident encoding");
    };
    let disk = source.canonical_disk_bytes().unwrap();
    assert_eq!(
        SourceUnit::from_canonical_storage_bytes(&disk, version.source_id, &descriptor).unwrap(),
        *source
    );
    let expected = adapter.encode(EncodeCacheRecordInput {
        handle: handle.clone(),
        cache_key: key.clone(),
        produced_by: request.validation_inputs.toolchain_compatibility.clone(),
        canonical_payload: SourceUnitCacheKey::from_source(&source)
            .stable_hash()
            .as_bytes()
            .to_vec(),
        storage_payload: disk,
        cacheability: CacheAdapterCacheability::Cacheable,
    });
    assert_eq!(EncodeCacheRecordOutcome::Encoded(record.clone()), expected);
    assert_eq!(record.header.key, key);
    assert_eq!(
        record.header.produced_by,
        request.validation_inputs.toolchain_compatibility
    );
    assert!(!record.header.uncacheable);
    let store = CacheStoreRoot::new(fixture.root.join("cache"));
    assert!(matches!(store.lookup(&key), CacheLookupOutcome::Miss(_)));
    assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
    let CacheLookupOutcome::Hit(readback) = store.lookup(&key) else {
        panic!("actual stored hit");
    };
    assert_eq!(readback, record);
    assert_eq!(
        store.insert(&readback).unwrap(),
        CacheInsertOutcome::AlreadyPresent
    );
    for (phase, reason) in [
        (
            PipelinePhase::SourceLoad,
            "current disk source maps require a real source load",
        ),
        (
            PipelinePhase::Frontend,
            "frontend inputs require a live source and dependency run",
        ),
    ] {
        assert_eq!(
            driver
                .registry()
                .cache_key_for_phase(phase, &phase_input)
                .unwrap()
                .intent,
            PhaseCacheIntent::NoKey {
                reason: reason.into()
            }
        );
    }
}

#[test]
fn native_disk_cache_record_blob_replay_relocates_ids_and_separates_raw_maps() {
    use mizar_ir::cache_adapter::EncodeCacheRecordOutcome;
    let mut formats = Vec::new();
    for raw in ["\u{feff}definition\r\nend;\r\n", "definition\nend;\n"] {
        let mut replays = Vec::new();
        for replay in 0..2 {
            let fixture = Fixture::new(raw.as_bytes());
            let ids = InMemorySessionIdAllocator::new();
            let source_ids = InMemorySessionIdAllocator::new();
            for _ in 0..replay {
                source_ids.next_source_id(snapshot_id(0x62)).unwrap();
            }
            let snapshots = SnapshotRegistry::with_allocator(source_ids);
            let (_driver, submission, owner, descriptor, handle) =
                native_disk_record_submission(&fixture, &ids, &snapshots, 0);
            let captured = submission.session.captured();
            let version = captured.snapshot().source_versions[0].clone();
            let snapshot = captured.snapshot().id;
            let unit = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    version.package_id.clone(),
                    version.module_path.clone(),
                ),
            };
            let phase_input = PhaseInput::new(
                unit.clone(),
                PhaseDispatchInputBundle::without_parent_outputs(
                    snapshot,
                    source_key(&version),
                    vec![],
                ),
            );
            let inputs = SourceLoadInputs {
                snapshot: captured.snapshot(),
                captured: Some(captured),
                build_plan: submission.build_plan.as_ref().unwrap(),
                module_index: submission.module_index.as_ref().unwrap(),
                allocator: &ids,
            };
            let source = owner.storage().get(&handle).unwrap();
            let adapter = mizar_ir::cache_adapter::IrCacheAdapter::new(owner.clone());
            let policy = mizar_proof::policy::VerifierPolicy::release();
            assert!(matches!(
                handle.any().placement(),
                StoragePlacement::Blob { .. }
            ));
            assert_eq!(
                source.line_map,
                LineMap::new(source.source_id, "definition\nend;\n")
            );
            assert_eq!(
                source.loading_map,
                fixture
                    .loading_map
                    .as_ref()
                    .map(|expected_map| LoadingMap::new(
                        source.source_id,
                        "definition\nend;\n",
                        expected_map.origin.clone(),
                        expected_map.segments.clone()
                    ))
            );
            let disk = source.canonical_disk_bytes().unwrap();
            let relocated = SourceUnit::from_canonical_storage_bytes(
                &disk,
                fixture.version.source_id,
                &descriptor,
            )
            .unwrap();
            assert!(source.matches_canonical_storage(&relocated, &descriptor));
            let EncodeCacheRecordOutcome::Encoded(record) = inputs
                .encode_native_disk_cache_record(&phase_input, &policy, &adapter, handle.clone())
                .unwrap()
            else {
                panic!("bound blob encoding");
            };
            replays.push((
                record,
                source.source_id,
                source.source_hash,
                disk,
                owner.storage().side_tables(&handle).unwrap(),
            ));
        }
        assert_eq!(replays[0].0, replays[1].0);
        assert_ne!(replays[0].1, replays[1].1);
        assert_eq!(replays[0].3, replays[1].3);
        formats.push(replays.remove(0));
    }
    assert_eq!(formats[0].2, formats[1].2);
    assert_eq!(
        formats[0].0.header.key.source_identity,
        formats[1].0.header.key.source_identity
    );
    assert_ne!(formats[0].3, formats[1].3);
    assert_ne!(formats[0].4.source_maps, formats[1].4.source_maps);
    assert_ne!(
        formats[0].0.header.key.final_hash,
        formats[1].0.header.key.final_hash
    );
    assert_ne!(formats[0].0.output, formats[1].0.output);
}

#[test]
fn native_disk_cache_record_rejects_physical_current_capture_and_output_bindings() {
    use mizar_ir::{
        cache_adapter::{EncodeCacheRecordOutcome, IrCacheAdapter},
        storage::CollectInput,
    };
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (_driver, submission, owner, _descriptor, handle) =
        native_disk_record_submission(&fixture, &ids, &snapshots, usize::MAX);
    let captured = submission.session.captured();
    let version = captured.snapshot().source_versions[0].clone();
    let snapshot = captured.snapshot().id;
    let unit = WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            version.package_id.clone(),
            version.module_path.clone(),
        ),
    };
    let phase_input = PhaseInput::new(
        unit.clone(),
        PhaseDispatchInputBundle::without_parent_outputs(snapshot, source_key(&version), vec![]),
    );
    let inputs = SourceLoadInputs {
        snapshot: captured.snapshot(),
        captured: Some(captured),
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: submission.module_index.as_ref().unwrap(),
        allocator: &ids,
    };
    let source = owner.storage().get(&handle).unwrap();
    let adapter = mizar_ir::cache_adapter::IrCacheAdapter::new(owner.clone());
    let policy = mizar_proof::policy::VerifierPolicy::release();

    assert!(matches!(
        inputs
            .encode_native_disk_cache_record(&phase_input, &policy, &adapter, handle.clone())
            .unwrap(),
        EncodeCacheRecordOutcome::Encoded(_)
    ));
    assert!(
        SourceLoadInputs {
            captured: None,
            ..inputs
        }
        .encode_native_disk_cache_record(&phase_input, &policy, &adapter, handle.clone())
        .is_err()
    );
    let foreign = IrCacheAdapter::new(publisher(snapshot, usize::MAX, &unit));
    assert!(
        inputs
            .encode_native_disk_cache_record(&phase_input, &policy, &foreign, handle.clone())
            .is_err()
    );
    let unpublished = IrCacheAdapter::new(std::sync::Arc::new(PhaseOutputPublisher::new(
        owner.storage().clone(),
        owner.registry().clone(),
    )));
    unpublished.publisher().register_current_snapshot(snapshot);
    assert!(
        inputs
            .encode_native_disk_cache_record(&phase_input, &policy, &unpublished, handle.clone())
            .is_err()
    );
    for (raw, relocate) in [
        ("definition\nend;\n", true),
        ("\u{feff}definition\r\nend;\r\n", false),
    ] {
        fixture.write(raw.as_bytes());
        let other_ids = InMemorySessionIdAllocator::new();
        let source_ids = InMemorySessionIdAllocator::new();
        if relocate {
            source_ids.next_source_id(snapshot_id(0x62)).unwrap();
        }
        let other_snapshots = SnapshotRegistry::with_allocator(source_ids);
        let (_, other_submission, other_owner, _, other_handle) =
            native_disk_record_submission(&fixture, &other_ids, &other_snapshots, usize::MAX);
        let other_capture = other_submission.session.captured();
        assert_eq!(other_capture.snapshot().id, snapshot);
        let other_source = other_owner.storage().get(&other_handle).unwrap();
        assert_eq!(other_source.source_hash, source.source_hash);
        if relocate {
            assert_ne!(other_source.source_id, source.source_id);
            assert_eq!(
                other_source.canonical_disk_bytes(),
                source.canonical_disk_bytes()
            );
        } else {
            assert_eq!(other_source.source_id, source.source_id);
            assert_ne!(
                other_source.canonical_disk_bytes(),
                source.canonical_disk_bytes()
            );
        }
        assert!(
            other_owner
                .validate_current_output(snapshot, other_handle.any())
                .is_ok()
        );
        assert!(
            other_owner
                .storage()
                .validate_handle(other_handle.any())
                .is_ok()
        );
        let other_adapter = IrCacheAdapter::new(other_owner);
        assert!(
            inputs
                .encode_native_disk_cache_record(
                    &phase_input,
                    &policy,
                    &other_adapter,
                    other_handle
                )
                .is_err()
        );
    }
    let wrong_dispatch = PhaseInput::new(
        unit,
        PhaseDispatchInputBundle::without_parent_outputs(snapshot, hash(9), vec![]),
    );
    assert!(
        inputs
            .encode_native_disk_cache_record(&wrong_dispatch, &policy, &adapter, handle.clone())
            .is_err()
    );
    owner.mark_obsolete(snapshot).unwrap();
    assert!(owner.storage().validate_handle(handle.any()).is_ok());
    assert!(
        inputs
            .encode_native_disk_cache_record(&phase_input, &policy, &adapter, handle.clone())
            .is_err()
    );

    fixture.write(b"definition\nend;\n");
    let collected_ids = InMemorySessionIdAllocator::new();
    let collected_snapshots = SnapshotRegistry::new();
    let (_, collected_submission, collected_owner, _, collected_handle) =
        native_disk_record_submission(&fixture, &collected_ids, &collected_snapshots, usize::MAX);
    let collected_capture = collected_submission.session.captured();
    let collected_snapshot = collected_capture.snapshot();
    let collected_input = PhaseInput::new(
        phase_input.work_unit.clone(),
        PhaseDispatchInputBundle::without_parent_outputs(
            collected_snapshot.id,
            source_key(&collected_snapshot.source_versions[0]),
            vec![],
        ),
    );
    let collected_inputs = SourceLoadInputs {
        snapshot: collected_snapshot,
        captured: Some(collected_capture),
        build_plan: collected_submission.build_plan.as_ref().unwrap(),
        module_index: collected_submission.module_index.as_ref().unwrap(),
        allocator: &collected_ids,
    };
    let collected_adapter = IrCacheAdapter::new(collected_owner.clone());
    assert!(matches!(
        collected_inputs
            .encode_native_disk_cache_record(
                &collected_input,
                &policy,
                &collected_adapter,
                collected_handle.clone()
            )
            .unwrap(),
        EncodeCacheRecordOutcome::Encoded(_)
    ));
    assert!(
        collected_owner
            .validate_current_output(collected_snapshot.id, collected_handle.any())
            .is_ok()
    );
    assert!(
        collected_owner
            .storage()
            .collect(CollectInput {
                snapshot: collected_snapshot.id,
                protected_outputs: vec![]
            })
            .outputs_dropped
            > 0
    );
    assert!(
        collected_owner
            .validate_current_output(collected_snapshot.id, collected_handle.any())
            .is_ok()
    );
    assert!(
        collected_owner
            .storage()
            .validate_handle(collected_handle.any())
            .is_err()
    );
    assert!(
        collected_inputs
            .encode_native_disk_cache_record(
                &collected_input,
                &policy,
                &collected_adapter,
                collected_handle
            )
            .is_err()
    );
}

#[test]
fn native_disk_expected_cache_payload_rebinds_current_resident_source() {
    use mizar_cache::cache_store::{CacheInsertOutcome, CacheLookupOutcome, CacheStoreRoot};
    use mizar_ir::{
        cache_adapter::{
            CacheAdapterMiss, CacheRehydrateOutcome, EncodeCacheRecordOutcome, IrCacheAdapter,
            RehydrateCacheHitInput,
        },
        publisher::PublishError,
        storage::{BlobDecodeError, BlobDecoder},
    };
    let fixtures = [
        Fixture::new(b"definition\nend;\n"),
        Fixture::new(b"definition\nend;\n"),
    ];
    let ids = InMemorySessionIdAllocator::new();
    let old_snapshots = SnapshotRegistry::new();
    let source_ids = InMemorySessionIdAllocator::new();
    source_ids.next_source_id(snapshot_id(0x62)).unwrap();
    let current_snapshots = SnapshotRegistry::with_allocator(source_ids);
    let (_, old_submission, old_owner, _, old_handle) =
        native_disk_record_submission(&fixtures[0], &ids, &old_snapshots, usize::MAX);
    let (driver, current_submission, current_owner, descriptor, current_handle) =
        native_disk_record_submission(&fixtures[1], &ids, &current_snapshots, usize::MAX);
    let old_capture = old_submission.session.captured();
    let current_capture = current_submission.session.captured();
    let unit = source_task(&current_submission).unit.clone();
    let policy = mizar_proof::policy::VerifierPolicy::release();
    let mut records = Vec::new();
    for (submission, owner, handle) in [
        (&old_submission, &old_owner, &old_handle),
        (&current_submission, &current_owner, &current_handle),
    ] {
        let captured = submission.session.captured();
        let input = PhaseInput::new(
            unit.clone(),
            PhaseDispatchInputBundle::without_parent_outputs(
                captured.snapshot().id,
                source_key(&captured.snapshot().source_versions[0]),
                vec![],
            ),
        );
        let inputs = SourceLoadInputs {
            snapshot: captured.snapshot(),
            captured: Some(captured),
            build_plan: submission.build_plan.as_ref().unwrap(),
            module_index: submission.module_index.as_ref().unwrap(),
            allocator: &ids,
        };
        let EncodeCacheRecordOutcome::Encoded(record) = inputs
            .encode_native_disk_cache_record(
                &input,
                &policy,
                &IrCacheAdapter::new(owner.clone()),
                handle.clone(),
            )
            .unwrap()
        else {
            panic!("genuine native record");
        };
        records.push(record);
    }
    assert_ne!(old_capture.snapshot().id, current_capture.snapshot().id);
    let old_source = old_owner.storage().get(&old_handle).unwrap();
    let current_source = current_owner.storage().get(&current_handle).unwrap();
    assert_ne!(old_source.source_id, current_source.source_id);
    assert_eq!(records[0], records[1]);
    let key = records[1].header.key.clone();
    let store = CacheStoreRoot::new(fixtures[1].root.join("cache"));
    let expected_bytes = SourceUnitCacheKey::from_source(&current_source)
        .stable_hash()
        .as_bytes()
        .to_vec();
    let expected_tables = current_owner
        .storage()
        .side_tables(&current_handle)
        .unwrap();
    let snapshot = current_capture.snapshot().id;
    for mode in ["miss", "denied", "obsolete", "resident"] {
        let target = if mode == "denied" {
            let target = std::sync::Arc::new(PhaseOutputPublisher::new(
                std::sync::Arc::new(IrStorageService::new()),
                std::sync::Arc::new(SnapshotHandleRegistry::new()),
            ));
            target.register_current_snapshot(snapshot);
            target
        } else {
            publisher(snapshot, usize::MAX, &unit)
        };
        if mode == "obsolete" {
            target.mark_obsolete(snapshot).unwrap();
        }
        if mode == "denied" {
            assert_eq!(
                store.insert(&records[0]).unwrap(),
                CacheInsertOutcome::Inserted
            );
        }
        let lookup = store.lookup(&key);
        if mode == "miss" {
            assert!(matches!(lookup, CacheLookupOutcome::Miss(_)));
        } else {
            assert!(matches!(lookup, CacheLookupOutcome::Hit(_)));
        }
        let source = current_source.clone();
        let descriptor = descriptor.clone();
        let adapter = IrCacheAdapter::new(target.clone());
        let outcome = adapter.rehydrate_with_expected_payload(
            RehydrateCacheHitInput {
                lookup,
                snapshot,
                phase: current_handle.phase().clone(),
                work_unit: current_handle.work_unit().clone(),
                output_kind: current_handle.output_kind().clone(),
                schema_version: current_handle.schema_version(),
                parents: vec![],
                named_input_hashes: current_handle.lineage().named_input_hashes.clone(),
                decode: BlobDecoder::new(move |bytes| {
                    SourceUnit::from_canonical_storage_bytes(bytes, source.source_id, &descriptor)
                        .filter(|decoded| decoded.matches_canonical_storage(&source, &descriptor))
                        .ok_or_else(|| BlobDecodeError::new("current source storage mismatch"))
                }),
            },
            &expected_bytes,
            &expected_tables,
        );
        match mode {
            "miss" => assert!(matches!(
                outcome,
                CacheRehydrateOutcome::Miss(CacheAdapterMiss::Cache(_))
            )),
            "denied" => assert!(
                matches!(outcome, CacheRehydrateOutcome::Miss(CacheAdapterMiss::Publisher { error }) if matches!(*error, PublishError::WorkUnitNotAllowed { .. }))
            ),
            "obsolete" => assert!(
                matches!(outcome, CacheRehydrateOutcome::Miss(CacheAdapterMiss::Publisher { error }) if matches!(*error, PublishError::ObsoleteSnapshot { .. }))
            ),
            _ => {
                let CacheRehydrateOutcome::Rehydrated(handle) = outcome else {
                    panic!("bound resident rehydration");
                };
                assert_eq!(handle.snapshot(), snapshot);
                assert_ne!(handle.output(), old_handle.output());
                assert_eq!(handle.placement(), &StoragePlacement::Resident);
                assert_eq!(*target.storage().get(&handle).unwrap(), *current_source);
                assert_eq!(
                    target.storage().side_tables(&handle).unwrap(),
                    expected_tables
                );
                assert!(
                    target
                        .validate_current_output(snapshot, handle.any())
                        .is_ok()
                );
            }
        }
        assert_eq!(
            adapter.successful_rehydrations(),
            u64::from(mode == "resident")
        );
    }
    let input = PhaseInput::new(
        unit,
        PhaseDispatchInputBundle::without_parent_outputs(
            snapshot,
            source_key(&current_capture.snapshot().source_versions[0]),
            vec![],
        ),
    );
    for (phase, reason) in [
        (
            PipelinePhase::SourceLoad,
            "current disk source maps require a real source load",
        ),
        (
            PipelinePhase::Frontend,
            "frontend inputs require a live source and dependency run",
        ),
    ] {
        assert_eq!(
            driver
                .registry()
                .cache_key_for_phase(phase, &input)
                .unwrap()
                .intent,
            PhaseCacheIntent::NoKey {
                reason: reason.into()
            }
        );
    }
}

#[test]
fn native_disk_expected_cache_payload_rejects_current_bytes_and_raw_maps_before_decode() {
    use mizar_cache::cache_store::{CacheInsertOutcome, CacheLookupOutcome, CacheStoreRoot};
    use mizar_ir::{
        cache_adapter::{
            CacheAdapterMiss, CacheRehydrateOutcome, EncodeCacheRecordOutcome, IrCacheAdapter,
            RehydrateCacheHitInput,
        },
        storage::{BlobDecodeError, BlobDecoder},
    };
    use std::sync::{Arc, atomic::AtomicUsize};
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let old_fixture = Fixture::new(b"definition\nend;\n");
    let (_, old_submission, old_owner, _, old_handle) =
        native_disk_record_submission(&old_fixture, &ids, &snapshots, usize::MAX);
    let old_capture = old_submission.session.captured();
    let unit = source_task(&old_submission).unit.clone();
    let input = PhaseInput::new(
        unit.clone(),
        PhaseDispatchInputBundle::without_parent_outputs(
            old_capture.snapshot().id,
            source_key(&old_capture.snapshot().source_versions[0]),
            vec![],
        ),
    );
    let inputs = SourceLoadInputs {
        snapshot: old_capture.snapshot(),
        captured: Some(old_capture),
        build_plan: old_submission.build_plan.as_ref().unwrap(),
        module_index: old_submission.module_index.as_ref().unwrap(),
        allocator: &ids,
    };
    let EncodeCacheRecordOutcome::Encoded(record) = inputs
        .encode_native_disk_cache_record(
            &input,
            &mizar_proof::policy::VerifierPolicy::release(),
            &IrCacheAdapter::new(old_owner.clone()),
            old_handle.clone(),
        )
        .unwrap()
    else {
        panic!("genuine native record");
    };
    let store = CacheStoreRoot::new(old_fixture.root.join("cache"));
    assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
    let old_source = old_owner.storage().get(&old_handle).unwrap();
    for (raw, expected_miss) in [
        ("definition\nend;\n@", CacheAdapterMiss::PayloadHashMismatch),
        (
            "\u{feff}definition\r\nend;\r\n",
            CacheAdapterMiss::SideTableHashMismatch,
        ),
    ] {
        let fixture = Fixture::new(raw.as_bytes());
        let (_, _submission, owner, _, handle) =
            native_disk_record_submission(&fixture, &ids, &snapshots, usize::MAX);
        let source = owner.storage().get(&handle).unwrap();
        let expected_bytes = SourceUnitCacheKey::from_source(&source)
            .stable_hash()
            .as_bytes()
            .to_vec();
        let expected_tables = owner.storage().side_tables(&handle).unwrap();
        assert_ne!(
            expected_tables,
            old_owner.storage().side_tables(&old_handle).unwrap()
        );
        if expected_miss == CacheAdapterMiss::SideTableHashMismatch {
            assert_eq!(source.source_hash, old_source.source_hash);
            assert_eq!(source.source_text, old_source.source_text);
        } else {
            assert_ne!(source.source_hash, old_source.source_hash);
        }
        let target = publisher(handle.snapshot(), usize::MAX, &unit);
        let adapter = IrCacheAdapter::new(target.clone());
        let decodes = Arc::new(AtomicUsize::new(0));
        let calls = decodes.clone();
        // The old-key hit is intentionally offered against different authentic current expectations.
        let lookup = store.lookup(&record.header.key);
        assert!(matches!(lookup, CacheLookupOutcome::Hit(_)));
        let outcome = adapter.rehydrate_with_expected_payload(
            RehydrateCacheHitInput {
                lookup,
                snapshot: handle.snapshot(),
                phase: handle.phase().clone(),
                work_unit: handle.work_unit().clone(),
                output_kind: handle.output_kind().clone(),
                schema_version: handle.schema_version(),
                parents: vec![],
                named_input_hashes: handle.lineage().named_input_hashes.clone(),
                decode: BlobDecoder::<SourceUnit>::new(move |_| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Err(BlobDecodeError::new("must reject before decoder"))
                }),
            },
            &expected_bytes,
            &expected_tables,
        );
        assert_eq!(outcome, CacheRehydrateOutcome::Miss(expected_miss));
        assert_eq!(decodes.load(Ordering::SeqCst), 0);
        assert_eq!(adapter.successful_rehydrations(), 0);
        assert!(
            target
                .validate_current_output(handle.snapshot(), handle.any())
                .is_err()
        );
        // Equal first slot IDs in independent fresh stores detect any hidden allocation.
        let baseline = publisher(handle.snapshot(), usize::MAX, &unit);
        assert_eq!(
            target
                .allocate::<SourceUnit>(
                    handle.snapshot(),
                    handle.phase().clone(),
                    handle.work_unit().clone(),
                    handle.output_kind().clone(),
                    handle.schema_version()
                )
                .slot(),
            baseline
                .allocate::<SourceUnit>(
                    handle.snapshot(),
                    handle.phase().clone(),
                    handle.work_unit().clone(),
                    handle.output_kind().clone(),
                    handle.schema_version()
                )
                .slot(),
        );
    }
}

#[test]
fn native_disk_expected_cache_blob_source_feeds_registered_frontend_like_clean_run() {
    use mizar_cache::{
        cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder},
        cache_store::{CacheInsertOutcome, CacheLookupOutcome, CacheStoreRoot},
    };
    use mizar_ir::{
        cache_adapter::{
            CacheRehydrateOutcome, EncodeCacheRecordOutcome, IrCacheAdapter, RehydrateCacheHitInput,
        },
        storage::{BlobDecodeError, BlobDecoder},
    };
    let fixtures = [
        Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes()),
        Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes()),
    ];
    let ids = InMemorySessionIdAllocator::new();
    let old_snapshots = SnapshotRegistry::new();
    let source_ids = InMemorySessionIdAllocator::new();
    source_ids.next_source_id(snapshot_id(0x62)).unwrap();
    let current_snapshots = SnapshotRegistry::with_allocator(source_ids);
    let (_, old_submission, old_owner, _, old_handle) =
        native_disk_record_submission(&fixtures[0], &ids, &old_snapshots, 0);
    let (driver, submission, owner, descriptor, clean_source_handle) =
        native_disk_record_submission(&fixtures[1], &ids, &current_snapshots, 0);
    let unit = source_task(&submission).unit.clone();
    let policy = mizar_proof::policy::VerifierPolicy::release();
    let mut records = Vec::new();
    let mut keys = Vec::new();
    for (submission, owner, handle) in [
        (&old_submission, &old_owner, &old_handle),
        (&submission, &owner, &clean_source_handle),
    ] {
        let captured = submission.session.captured();
        let input = PhaseInput::new(
            unit.clone(),
            PhaseDispatchInputBundle::without_parent_outputs(
                captured.snapshot().id,
                source_key(&captured.snapshot().source_versions[0]),
                vec![],
            ),
        );
        let inputs = SourceLoadInputs {
            snapshot: captured.snapshot(),
            captured: Some(captured),
            build_plan: submission.build_plan.as_ref().unwrap(),
            module_index: submission.module_index.as_ref().unwrap(),
            allocator: &ids,
        };
        let CacheKeyBuildOutcome::Cacheable(key) = CacheKeyBuilder::new(
            inputs
                .prepare_native_disk_cache_request(&input, &policy)
                .unwrap(),
        )
        .build() else {
            panic!("genuine current source key");
        };
        keys.push(key);
        let EncodeCacheRecordOutcome::Encoded(record) = inputs
            .encode_native_disk_cache_record(
                &input,
                &policy,
                &IrCacheAdapter::new(owner.clone()),
                handle.clone(),
            )
            .unwrap()
        else {
            panic!("genuine native blob record");
        };
        records.push(record);
    }
    assert_eq!(keys[0], keys[1]);
    assert_eq!(records[0], records[1]);
    let source = owner.storage().get(&clean_source_handle).unwrap();
    assert_ne!(
        source.source_id,
        old_owner.storage().get(&old_handle).unwrap().source_id
    );
    assert_ne!(clean_source_handle.snapshot(), old_handle.snapshot());
    let expected_bytes = SourceUnitCacheKey::from_source(&source)
        .stable_hash()
        .as_bytes()
        .to_vec();
    let expected_tables = owner.storage().side_tables(&clean_source_handle).unwrap();
    let store = CacheStoreRoot::new(fixtures[1].root.join("cache"));
    assert_eq!(
        store.insert(&records[0]).unwrap(),
        CacheInsertOutcome::Inserted
    );
    let lookup = store.lookup(&keys[1]);
    assert!(matches!(lookup, CacheLookupOutcome::Hit(_)));
    let target = publisher(clean_source_handle.snapshot(), 0, &unit);
    let adapter = IrCacheAdapter::new(target.clone());
    let decoded_source = source.clone();
    let CacheRehydrateOutcome::Rehydrated(handle) = adapter.rehydrate_with_expected_payload(
        RehydrateCacheHitInput {
            lookup,
            snapshot: clean_source_handle.snapshot(),
            phase: clean_source_handle.phase().clone(),
            work_unit: clean_source_handle.work_unit().clone(),
            output_kind: clean_source_handle.output_kind().clone(),
            schema_version: clean_source_handle.schema_version(),
            parents: vec![],
            named_input_hashes: clean_source_handle.lineage().named_input_hashes.clone(),
            decode: BlobDecoder::new(move |bytes| {
                SourceUnit::from_canonical_storage_bytes(
                    bytes,
                    decoded_source.source_id,
                    &descriptor,
                )
                .filter(|decoded| decoded.matches_canonical_storage(&decoded_source, &descriptor))
                .ok_or_else(|| BlobDecodeError::new("current source storage mismatch"))
            }),
        },
        &expected_bytes,
        &expected_tables,
    ) else {
        panic!("bound blob rehydration");
    };
    assert_eq!(adapter.successful_rehydrations(), 1);
    assert!(matches!(handle.placement(), StoragePlacement::Blob { .. }));
    assert_eq!(*target.storage().get(&handle).unwrap(), *source);
    assert_eq!(
        target.storage().side_tables(&handle).unwrap(),
        expected_tables
    );
    let snapshot = submission.session.captured().snapshot();
    let frontend_input = PhaseInput::new(
        unit,
        PhaseDispatchInputBundle::new(
            snapshot.id,
            source_key(&snapshot.source_versions[0]),
            vec![],
            vec![
                SealedParentOutputHandle::from_current_output(&target, snapshot.id, handle.erase())
                    .unwrap(),
            ],
        )
        .unwrap(),
    );
    allow_frontend(&target, &frontend_input.work_unit);
    assert_eq!(
        driver
            .registry()
            .cache_key_for_phase(PipelinePhase::Frontend, &frontend_input)
            .unwrap()
            .intent,
        PhaseCacheIntent::NoKey {
            reason: "frontend inputs require a live source and dependency run".into()
        }
    );
    let replay = driver
        .registry()
        .execute_phase_with_resources(
            PipelinePhase::Frontend,
            frontend_input,
            PhaseExecutionResources {
                diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                    DiagnosticPhase::Frontend,
                    snapshot.id,
                    "frontend.run_loaded",
                ))),
                output_publisher: Some(target.clone()),
                source_load: Some(SourceLoadInputs {
                    snapshot,
                    captured: Some(submission.session.captured()),
                    build_plan: submission.build_plan.as_ref().unwrap(),
                    module_index: submission.module_index.as_ref().unwrap(),
                    allocator: &ids,
                }),
                ..PhaseExecutionResources::default()
            },
        )
        .unwrap()
        .result;
    let clean = submission
        .scheduler_run
        .as_ref()
        .unwrap()
        .phase_results
        .values()
        .flatten()
        .find(|result| {
            result
                .output_refs
                .iter()
                .any(|handle| handle.output_kind() == &OutputKind::new("FrontendOutput"))
        })
        .unwrap();
    assert_eq!(replay.status, PhaseStatus::Complete);
    assert_eq!(replay.diagnostics, clean.diagnostics);
    let replay_handle = target
        .storage()
        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
            &replay.output_refs[0],
            &OutputKind::new("FrontendOutput"),
        )
        .unwrap();
    let clean_handle = owner
        .storage()
        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
            &clean.output_refs[0],
            &OutputKind::new("FrontendOutput"),
        )
        .unwrap();
    assert_eq!(
        *target.storage().get(&replay_handle).unwrap(),
        *owner.storage().get(&clean_handle).unwrap()
    );
    assert_eq!(
        target.storage().side_tables(&replay_handle).unwrap(),
        owner.storage().side_tables(&clean_handle).unwrap()
    );
    assert_eq!(replay_handle.lineage().parents, vec![handle.output()]);
}

#[test]
fn native_disk_cache_lookup_rebinds_resident_and_blob_for_clean_frontend() {
    use mizar_cache::cache_store::{CacheInsertOutcome, CacheLookupOutcome, CacheStoreRoot};
    use mizar_ir::cache_adapter::{
        CacheRehydrateOutcome, EncodeCacheRecordOutcome, IrCacheAdapter,
    };
    for threshold in [usize::MAX, 0] {
        let fixtures = [
            Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes()),
            Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes()),
        ];
        let ids = InMemorySessionIdAllocator::new();
        let old_snapshots = SnapshotRegistry::new();
        let source_ids = InMemorySessionIdAllocator::new();
        source_ids.next_source_id(snapshot_id(0x62)).unwrap();
        let current_snapshots = SnapshotRegistry::with_allocator(source_ids);
        let (_, old_submission, old_owner, _, old_handle) =
            native_disk_record_submission(&fixtures[0], &ids, &old_snapshots, threshold);
        let (driver, submission, owner, _, clean_source) =
            native_disk_record_submission(&fixtures[1], &ids, &current_snapshots, threshold);
        let (old_inputs, old_input) = native_disk_lookup_context(&old_submission, &ids);
        let (inputs, input) = native_disk_lookup_context(&submission, &ids);
        let policy = mizar_proof::policy::VerifierPolicy::release();
        let EncodeCacheRecordOutcome::Encoded(record) = old_inputs
            .encode_native_disk_cache_record(
                &old_input,
                &policy,
                &IrCacheAdapter::new(old_owner.clone()),
                old_handle.clone(),
            )
            .unwrap()
        else {
            panic!("genuine native producer");
        };
        assert_ne!(old_input.snapshot, input.snapshot);
        assert_ne!(
            old_inputs.snapshot.source_versions[0].source_id,
            inputs.snapshot.source_versions[0].source_id
        );
        let store = CacheStoreRoot::new(fixtures[1].root.join("cache"));
        assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
        assert!(matches!(
            store.lookup(&record.header.key),
            CacheLookupOutcome::Hit(_)
        ));
        let target = publisher(input.snapshot, threshold, &input.work_unit);
        let adapter = IrCacheAdapter::new(target.clone());
        let CacheRehydrateOutcome::Rehydrated(handle) = inputs
            .rehydrate_native_disk_cache(&input, &policy, &store, &adapter)
            .unwrap()
        else {
            panic!("current native source reconstruction");
        };
        assert_eq!(handle.snapshot(), input.snapshot);
        assert_eq!(handle.placement(), clean_source.placement());
        assert_eq!(
            handle.lineage().named_input_hashes,
            clean_source.lineage().named_input_hashes
        );
        assert!(handle.lineage().parents.is_empty());
        assert_eq!(
            *target.storage().get(&handle).unwrap(),
            *owner.storage().get(&clean_source).unwrap()
        );
        assert_eq!(
            target.storage().side_tables(&handle).unwrap(),
            owner.storage().side_tables(&clean_source).unwrap()
        );
        assert_eq!(adapter.successful_rehydrations(), 1);
        for (phase, reason) in [
            (
                PipelinePhase::SourceLoad,
                "current disk source maps require a real source load",
            ),
            (
                PipelinePhase::Frontend,
                "frontend inputs require a live source and dependency run",
            ),
        ] {
            assert_eq!(
                driver
                    .registry()
                    .cache_key_for_phase(phase, &input)
                    .unwrap()
                    .intent,
                PhaseCacheIntent::NoKey {
                    reason: reason.into()
                }
            );
        }
        let frontend_input = PhaseInput::new(
            input.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                input.snapshot,
                input.identities().input_hash(),
                vec![],
                vec![
                    SealedParentOutputHandle::from_current_output(
                        &target,
                        input.snapshot,
                        handle.erase(),
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        );
        allow_frontend(&target, &frontend_input.work_unit);
        let replay = driver
            .registry()
            .execute_phase_with_resources(
                PipelinePhase::Frontend,
                frontend_input,
                PhaseExecutionResources {
                    diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                        DiagnosticPhase::Frontend,
                        input.snapshot,
                        "frontend.run_loaded",
                    ))),
                    output_publisher: Some(target.clone()),
                    source_load: Some(inputs),
                    ..PhaseExecutionResources::default()
                },
            )
            .unwrap()
            .result;
        let clean = submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .find(|result| {
                result
                    .output_refs
                    .iter()
                    .any(|output| output.output_kind() == &OutputKind::new("FrontendOutput"))
            })
            .unwrap();
        assert_eq!(replay.status, PhaseStatus::Complete);
        assert_eq!(replay.diagnostics, clean.diagnostics);
        assert!(replay.diagnostics.is_empty()); // This valid fixture covers the empty diagnostic case.
        let replay_handle = target
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                &replay.output_refs[0],
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let clean_handle = owner
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                &clean.output_refs[0],
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        assert_eq!(
            *target.storage().get(&replay_handle).unwrap(),
            *owner.storage().get(&clean_handle).unwrap()
        );
        assert_eq!(
            target.storage().side_tables(&replay_handle).unwrap(),
            owner.storage().side_tables(&clean_handle).unwrap()
        );
        assert_eq!(replay_handle.lineage().parents, vec![handle.output()]);
    }
}

#[test]
fn native_disk_cache_lookup_misses_changed_source_maps_context_and_policy() {
    use mizar_cache::{
        cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder},
        cache_store::{CacheLookupOutcome, CacheStoreRoot},
    };
    use mizar_ir::cache_adapter::{
        CacheAdapterMiss, CacheRehydrateOutcome, EncodeCacheRecordOutcome, IrCacheAdapter,
    };
    let old_fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (_, old_submission, old_owner, _, old_handle) =
        native_disk_record_submission(&old_fixture, &ids, &snapshots, usize::MAX);
    let (old_inputs, old_input) = native_disk_lookup_context(&old_submission, &ids);
    let baseline_policy = mizar_proof::policy::VerifierPolicy::release();
    let baseline_request = old_inputs
        .prepare_native_disk_cache_request(&old_input, &baseline_policy)
        .unwrap();
    let EncodeCacheRecordOutcome::Encoded(record) = old_inputs
        .encode_native_disk_cache_record(
            &old_input,
            &baseline_policy,
            &IrCacheAdapter::new(old_owner),
            old_handle,
        )
        .unwrap()
    else {
        panic!("genuine native producer");
    };
    let store = CacheStoreRoot::new(old_fixture.root.join("cache"));
    store.insert(&record).unwrap();
    for mode in [
        "source",
        "maps",
        "config",
        "lock",
        "toolchain",
        "profile",
        "settings",
    ] {
        let raw = match mode {
            "source" => "definition\n\nend;\n",
            "maps" => "\u{feff}definition\r\nend;\r\n",
            _ => "definition\nend;\n",
        };
        let fixture = Fixture::new(raw.as_bytes());
        let (mut draft, mut submit_input) = fixture.submit_request(vec![], vec![]);
        draft.source_inputs.versions.clear();
        match mode {
            "config" => draft.verifier_config = VerifierConfigInput::new(hash(3)),
            "lock" => draft.dependency_inputs.lockfile_hash = hash(4),
            "toolchain" => {
                draft.dependency_inputs.toolchain = ToolchainInfo::new("different-toolchain");
                submit_input.plan_request.toolchain = draft.dependency_inputs.toolchain.clone();
            }
            _ => {}
        }
        let session = draft
            .capture_sources(
                &ids,
                &snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    SourceInput {
                        package_id: fixture.version.package_id.clone(),
                        module_path: fixture.version.module_path.clone(),
                        normalized_path: fixture.version.normalized_path.clone(),
                        edition: fixture.version.edition.clone(),
                        origin: SourceOriginInput::Disk {
                            path: "src/main.miz".into(),
                        },
                    },
                )],
            )
            .unwrap();
        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
        let mut driver = CompilerDriver::new(source_registry());
        let submission = driver
            .submit_captured(session, &ids, &snapshots, submit_input)
            .unwrap();
        let (inputs, input) = native_disk_lookup_context(&submission, &ids);
        let policy = match mode {
            "profile" => baseline_policy.clone().with_profile_id("alternate-release"),
            "settings" => baseline_policy
                .clone()
                .with_require_kernel_certificates(false),
            _ => baseline_policy.clone(),
        };
        let request = inputs
            .prepare_native_disk_cache_request(&input, &policy)
            .unwrap();
        if mode == "config" {
            assert_ne!(request.config_hash, baseline_request.config_hash);
            assert_eq!(
                request.policy_fingerprint,
                baseline_request.policy_fingerprint
            );
        } else if mode == "profile" || mode == "settings" {
            assert_eq!(request.config_hash, baseline_request.config_hash);
            assert_ne!(
                request.policy_fingerprint,
                baseline_request.policy_fingerprint
            );
        } else if mode == "maps" {
            assert_eq!(request.source_identity, baseline_request.source_identity);
            assert_ne!(request.input_hashes[1], baseline_request.input_hashes[1]);
        }
        let CacheKeyBuildOutcome::Cacheable(key) = CacheKeyBuilder::new(request).build() else {
            panic!("current native key");
        };
        assert_ne!(key, record.header.key, "{mode}");
        assert!(matches!(store.lookup(&key), CacheLookupOutcome::Miss(_)));
        let target = publisher(input.snapshot, usize::MAX, &input.work_unit);
        let adapter = IrCacheAdapter::new(target);
        assert!(matches!(
            inputs
                .rehydrate_native_disk_cache(&input, &policy, &store, &adapter)
                .unwrap(),
            CacheRehydrateOutcome::Miss(CacheAdapterMiss::Cache(_))
        ));
        assert_eq!(adapter.successful_rehydrations(), 0);
    }
}

#[test]
fn native_disk_cache_lookup_rejects_binding_currentness_and_foreign_producer() {
    use mizar_cache::cache_store::{CacheLookupOutcome, CacheStoreRoot};
    use mizar_ir::cache_adapter::{
        CacheAdapterMiss, CacheRehydrateOutcome, EncodeCacheRecordOutcome, IrCacheAdapter,
    };
    let fixtures = [
        Fixture::new(b"definition\nend;\n"),
        Fixture::new(b"definition\nend;\n"),
    ];
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (_, submission, owner, _, source_handle) =
        native_disk_record_submission(&fixtures[0], &ids, &snapshots, usize::MAX);
    let (_, foreign_submission, _, _, _) =
        native_disk_record_submission(&fixtures[1], &ids, &snapshots, usize::MAX);
    let (inputs, input) = native_disk_lookup_context(&submission, &ids);
    let policy = mizar_proof::policy::VerifierPolicy::release();
    let EncodeCacheRecordOutcome::Encoded(record) = inputs
        .encode_native_disk_cache_record(
            &input,
            &policy,
            &IrCacheAdapter::new(owner.clone()),
            source_handle.clone(),
        )
        .unwrap()
    else {
        panic!("genuine native producer");
    };
    let store = CacheStoreRoot::new(fixtures[0].root.join("cache"));
    store.insert(&record).unwrap();
    for mode in [
        "missing",
        "foreign",
        "plan",
        "unregistered",
        "obsolete",
        "denied",
        "producer",
    ] {
        let target = if mode == "unregistered" || mode == "denied" {
            let target = std::sync::Arc::new(PhaseOutputPublisher::new(
                std::sync::Arc::new(IrStorageService::new()),
                std::sync::Arc::new(SnapshotHandleRegistry::new()),
            ));
            if mode == "denied" {
                target.register_current_snapshot(input.snapshot);
            }
            target
        } else {
            publisher(input.snapshot, usize::MAX, &input.work_unit)
        };
        if mode == "obsolete" {
            target.mark_obsolete(input.snapshot).unwrap();
        }
        let mut plan = submission.build_plan.as_ref().unwrap().clone();
        if mode == "plan" {
            plan.workspace_root = foreign_submission
                .build_plan
                .as_ref()
                .unwrap()
                .workspace_root
                .clone();
        }
        let bound_inputs = SourceLoadInputs {
            captured: match mode {
                "missing" => None,
                "foreign" => Some(foreign_submission.session.captured()),
                _ => inputs.captured,
            },
            build_plan: &plan,
            ..inputs
        };
        let adversarial_store;
        let selected_store = if mode == "producer" {
            let mut modified = record.clone();
            // Only this negative record changes genuine transport metadata.
            let other = ToolchainInfo::new("other-native-producer");
            modified.header.produced_by[0].value = other.identity().to_owned();
            adversarial_store =
                CacheStoreRoot::new(fixtures[0].root.join("foreign-producer-cache"));
            adversarial_store.insert(&modified).unwrap();
            assert!(matches!(
                adversarial_store.lookup(&record.header.key),
                CacheLookupOutcome::Hit(_)
            ));
            &adversarial_store
        } else {
            &store
        };
        let adapter = IrCacheAdapter::new(target.clone());
        let result =
            bound_inputs.rehydrate_native_disk_cache(&input, &policy, selected_store, &adapter);
        if mode == "denied" {
            assert!(matches!(
                result.unwrap(),
                CacheRehydrateOutcome::Miss(CacheAdapterMiss::Publisher { .. })
            ));
        } else {
            assert!(result.is_err(), "{mode}: {result:?}");
        }
        assert_eq!(adapter.successful_rehydrations(), 0);
        assert!(
            target
                .validate_current_output(input.snapshot, source_handle.any())
                .is_err()
        );
    }
}

#[test]
fn native_disk_cache_scheduled_results_match_clean_runs_and_release_owners() {
    use mizar_build::planner::{lockfile_content_hash, workspace_config_content_hash};
    use mizar_build::scheduler::SchedulerEventKind;
    use mizar_cache::cache_store::{CacheInsertOutcome, CacheStoreRoot};
    use mizar_ir::{
        cache_adapter::{EncodeCacheRecordOutcome, IrCacheAdapter},
        storage::CollectInput,
    };
    for threshold in [usize::MAX, 0] {
        for case in [
            "hit", "raw", "missing", "disabled", "provider", "config", "cancel", "obsolete",
        ] {
            let fixture = Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes());
            let ids = InMemorySessionIdAllocator::new();
            let snapshots = SnapshotRegistry::new();
            let policy = mizar_proof::policy::VerifierPolicy::release();
            let store = CacheStoreRoot::new(fixture.root.join("scheduled-cache"));
            let mut old_source = None;
            for old in [true, false] {
                fixture.write(if !old && case == "raw" {
                    b"definition\nend;\n"
                } else {
                    "\u{feff}definition\r\nend;\r\n".as_bytes()
                });
                let (mut draft, mut input) = fixture.submit_request(vec![], vec![]);
                let plan = produce_build_plan(
                    input.plan_request.clone(),
                    input.workspace_packages.clone(),
                    input.lockfile.clone(),
                )
                .unwrap();
                draft.source_inputs.versions.clear();
                draft.dependency_inputs.lockfile_hash = lockfile_content_hash(&plan.lockfile);
                draft.verifier_config = VerifierConfigInput::new(workspace_config_content_hash(
                    &plan.verifier_config,
                    &plan.build_config,
                ));
                if !old && case == "config" {
                    draft.verifier_config = VerifierConfigInput::new(hash(2));
                }
                let descriptor = SourceInput {
                    package_id: fixture.version.package_id.clone(),
                    module_path: fixture.version.module_path.clone(),
                    normalized_path: fixture.version.normalized_path.clone(),
                    edition: fixture.version.edition.clone(),
                    origin: SourceOriginInput::Disk {
                        path: "src/main.miz".into(),
                    },
                };
                let session = draft
                    .capture_sources(
                        &ids,
                        &snapshots,
                        vec![(
                            DiskSourceLoader::new(fixture.root.join("alpha")),
                            descriptor,
                        )],
                    )
                    .unwrap();
                let snapshot = session.captured().snapshot().id;
                let source_id = session.captured().snapshot().source_versions[0].source_id;
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
                let mut driver = CompilerDriver::new(builder.build().unwrap())
                    .with_output_publisher(owner.clone());
                fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
                if old {
                    let submission = driver
                        .submit_captured(session, &ids, &snapshots, input)
                        .unwrap();
                    let (inputs, phase_input) = native_disk_lookup_context(&submission, &ids);
                    assert_eq!(inputs.validate_native_lock_content(), Ok(()));
                    assert_eq!(inputs.validate_native_config_content(), Ok(()));
                    let result = submission
                        .scheduler_run
                        .as_ref()
                        .unwrap()
                        .phase_results
                        .get(&source_task(&submission).id)
                        .unwrap();
                    let handle = owner
                        .storage()
                        .typed_handle::<SourceUnit>(
                            &result[0].output_refs[0],
                            &OutputKind::new("SourceUnit"),
                        )
                        .unwrap();
                    let EncodeCacheRecordOutcome::Encoded(record) = inputs
                        .encode_native_disk_cache_record(
                            &phase_input,
                            &policy,
                            &IrCacheAdapter::new(owner),
                            handle,
                        )
                        .unwrap()
                    else {
                        panic!("real scheduled record");
                    };
                    if case != "missing" {
                        assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
                        assert!(matches!(
                            store.lookup(&record.header.key),
                            mizar_cache::cache_store::CacheLookupOutcome::Hit(_)
                        ));
                    }
                    old_source = Some((
                        source_id,
                        submission.session.captured().snapshot().source_versions[0].source_hash,
                    ));
                    continue;
                }
                assert_ne!(old_source.unwrap().0, source_id);
                assert_eq!(
                    old_source.unwrap().1,
                    session.captured().snapshot().source_versions[0].source_hash
                );
                let (_, clean_input) = fixture.submit_request(vec![], vec![]);
                let clean_owner = publisher(snapshot, threshold, &unit);
                allow_frontend(&clean_owner, &unit);
                let mut clean_builder = PhaseRegistryBuilder::new();
                clean_builder.register_source_load();
                clean_builder.register_frontend(vec![]);
                let clean = CompilerDriver::new(clean_builder.build().unwrap())
                    .with_output_publisher(clean_owner.clone())
                    .submit_captured(session.clone(), &ids, &snapshots, clean_input)
                    .unwrap();
                let source_task = source_task(&clean);
                let frontend_task = clean
                    .task_graph
                    .as_ref()
                    .unwrap()
                    .tasks()
                    .iter()
                    .find(|task| task.phases == [PipelinePhase::Frontend])
                    .unwrap();
                input.cache_policy = if case == "disabled" {
                    CacheSchedulingPolicy::Disabled
                } else {
                    CacheSchedulingPolicy::Enabled
                };
                // Even an otherwise viable caller hit must be replaced by this route.
                input.cache_decisions = CacheSchedulingPlan::new(vec![CacheTaskDecision::new(
                    source_task.id.clone(),
                    CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
                        vec![CacheOutputRef::new("caller", "fiction")],
                        vec![],
                    )),
                )]);
                if case == "provider" {
                    input.phase_dispatch_inputs = Some(Box::new(SuppliedInput::None));
                }
                if case == "cancel" {
                    input.cancellation.cancel_task(source_task.id.clone());
                }
                if case == "obsolete" {
                    input
                        .cancellation
                        .discard_obsolete_completed_task(source_task.id.clone());
                }
                let submission = driver
                    .submit_captured_with_native_disk_cache(
                        session, &ids, &snapshots, input, &policy, &store,
                    )
                    .unwrap();
                let run = submission.scheduler_run.as_ref().unwrap();
                let state = run
                    .task_states
                    .iter()
                    .find(|record| record.task_id == source_task.id)
                    .unwrap()
                    .state;
                let started = |id: &mizar_build::task_graph::TaskId| {
                    run.events.iter().any(|event| {
                        event.task_id.as_ref() == Some(id)
                            && event.kind == SchedulerEventKind::TaskStarted
                    })
                };
                if case == "cancel" || case == "obsolete" || case == "provider" {
                    assert_eq!(
                        state,
                        if case == "provider" {
                            TaskState::Blocked
                        } else {
                            TaskState::Cancelled
                        }
                    );
                    assert!(!run.phase_results.contains_key(&source_task.id));
                    assert!(!run.phase_results.contains_key(&frontend_task.id));
                } else {
                    assert_eq!(
                        state,
                        if case == "hit" {
                            TaskState::CacheHit
                        } else {
                            TaskState::Completed
                        }
                    );
                    assert_eq!(started(&source_task.id), case != "hit");
                    assert!(started(&frontend_task.id));
                    let mut actual_source = None;
                    for (id, kind) in [
                        (&source_task.id, "SourceUnit"),
                        (&frontend_task.id, "FrontendOutput"),
                    ] {
                        let actual = &run.phase_results[id][0];
                        let expected = &clean.scheduler_run.as_ref().unwrap().phase_results[id][0];
                        assert_eq!(actual.status, PhaseStatus::Complete);
                        assert_eq!(actual.diagnostics, expected.diagnostics);
                        if case == "hit" && kind == "SourceUnit" {
                            assert!(actual.cache_observation.is_none());
                        }
                        let output = &actual.output_refs[0];
                        assert_eq!(
                            output.content_hash(),
                            expected.output_refs[0].content_hash()
                        );
                        assert_eq!(
                            output.side_table_hash(),
                            expected.output_refs[0].side_table_hash()
                        );
                        assert_eq!(
                            matches!(output.placement(), StoragePlacement::Blob { .. }),
                            threshold == 0
                        );
                        if kind == "SourceUnit" {
                            let actual_handle = owner
                                .storage()
                                .typed_handle::<SourceUnit>(output, &OutputKind::new(kind))
                                .unwrap();
                            let expected_handle = clean_owner
                                .storage()
                                .typed_handle::<SourceUnit>(
                                    &expected.output_refs[0],
                                    &OutputKind::new(kind),
                                )
                                .unwrap();
                            assert_eq!(
                                *owner.storage().get(&actual_handle).unwrap(),
                                *clean_owner.storage().get(&expected_handle).unwrap()
                            );
                            assert_eq!(
                                owner.storage().side_tables(&actual_handle).unwrap(),
                                clean_owner.storage().side_tables(&expected_handle).unwrap()
                            );
                            assert!(output.lineage().parents.is_empty());
                            actual_source = Some(output.output());
                        } else {
                            let actual_handle = owner.storage().typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(output, &OutputKind::new(kind)).unwrap();
                            let expected_handle = clean_owner.storage().typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(&expected.output_refs[0], &OutputKind::new(kind)).unwrap();
                            assert_eq!(
                                *owner.storage().get(&actual_handle).unwrap(),
                                *clean_owner.storage().get(&expected_handle).unwrap()
                            );
                            assert_eq!(
                                owner.storage().side_tables(&actual_handle).unwrap(),
                                clean_owner.storage().side_tables(&expected_handle).unwrap()
                            );
                            assert_eq!(output.lineage().parents, vec![actual_source.unwrap()]);
                        }
                    }
                }
                assert_eq!(
                    submission.status,
                    if case == "provider" {
                        DriverSubmissionStatus::BlockedByPhaseDispatchGap
                    } else {
                        DriverSubmissionStatus::BlockedByMissingPhaseServices
                    }
                );
                let collected = owner.storage().collect(CollectInput {
                    snapshot,
                    protected_outputs: vec![],
                });
                assert_eq!(collected.retained_outputs, 0);
                assert_eq!(
                    collected.outputs_dropped,
                    if case == "provider" {
                        0
                    } else if case == "cancel" || case == "obsolete" {
                        1
                    } else {
                        2
                    }
                );
            }
        }
    }
}

#[test]
fn native_disk_cache_edits_match_independent_clean_prefixes_across_replay_controls() {
    use mizar_build::{
        planner::{lockfile_content_hash, workspace_config_content_hash},
        scheduler::SchedulerEventKind,
    };
    use mizar_cache::{
        cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder},
        cache_store::{CacheInsertOutcome, CacheLookupOutcome, CacheStoreRoot},
    };
    use mizar_ir::cache_adapter::{EncodeCacheRecordOutcome, IrCacheAdapter};
    for threshold in [usize::MAX, 0] {
        let fixture = Fixture::new(b"definition\nend;\n");
        let leaf =
            "definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n";
        fs::write(fixture.root.join("alpha/src/leaf.miz"), leaf).unwrap();
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let mut driver = CompilerDriver::new(builder.build().unwrap());
        let policy = mizar_proof::policy::VerifierPolicy::release();
        let store = CacheStoreRoot::new(fixture.root.join("equivalence-cache"));
        let mut baselines = Vec::new();
        let mut unused = None;
        // The ordinary warmup supplies both genuine records before the replay table.
        for iteration in 0_usize..=12 {
            let warmup = iteration == 0;
            let stage = iteration.saturating_sub(1) / 4;
            let control = iteration.saturating_sub(1) % 4;
            let (workers, order, reverse) = [
                (1, CompletionOrder::Canonical, false),
                (1, CompletionOrder::Canonical, false),
                (4, CompletionOrder::Reverse, false),
                (4, CompletionOrder::Canonical, true),
            ][control];
            let main = if stage == 0 {
                "import alpha.leaf;\ntheorem T: a combine b = a;\n"
            } else {
                "import alpha.leaf;\ntheorem T: ;\n"
            };
            let raw = if stage < 2 {
                format!("\u{feff}{}", main.replace('\n', "\r\n"))
            } else {
                main.to_owned()
            };
            fixture.write(raw.as_bytes());
            let mut clean_projection = None;
            let mut clean_ids = Vec::new();
            for clean in [true, false] {
                if warmup && clean {
                    continue;
                }
                let clean_allocator = InMemorySessionIdAllocator::new();
                // Advance the allocator actually owned by the clean capture registry.
                for _ in 0..64 {
                    clean_allocator.next_source_id(snapshot_id(0x62)).unwrap();
                }
                let clean_snapshots = SnapshotRegistry::with_allocator(clean_allocator);
                let selected_snapshots = if clean { &clean_snapshots } else { &snapshots };
                let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
                request.source_inputs.versions.clear();
                request.generation = BuildRequestGeneration::new(iteration as u64);
                let plan = produce_build_plan(
                    input.plan_request.clone(),
                    input.workspace_packages.clone(),
                    input.lockfile.clone(),
                )
                .unwrap();
                request.dependency_inputs.lockfile_hash = lockfile_content_hash(&plan.lockfile);
                request.verifier_config = VerifierConfigInput::new(workspace_config_content_hash(
                    &plan.verifier_config,
                    &plan.build_config,
                ));
                let mut files = vec![
                    WorkspaceSourceFile::new("src/main.miz", "main.miz"),
                    WorkspaceSourceFile::new("src/leaf.miz", "leaf.miz"),
                ];
                let mut descriptors = ["leaf", "main"]
                    .map(|module| {
                        let path = format!("src/{module}.miz");
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
                                origin: SourceOriginInput::Disk { path: path.into() },
                            },
                        )
                    })
                    .to_vec();
                if reverse {
                    descriptors.reverse();
                    files.reverse();
                }
                input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
                    package_id: PackageId::new("alpha"),
                    files,
                }]);
                input.dependency_overlay = complete_leaf_import_overlay();
                input.worker_count = workers;
                input.completion_order = order;
                input.cache_policy = CacheSchedulingPolicy::Enabled;
                let session = request
                    .clone()
                    .capture_sources(&ids, selected_snapshots, descriptors.clone())
                    .unwrap();
                let snapshot = session.captured().snapshot().id;
                let unit = WorkUnit::Module {
                    module: mizar_build::module_index::ModuleId::new(
                        PackageId::new("alpha"),
                        ModulePath::new("main"),
                    ),
                };
                let owner = publisher(snapshot, threshold, &unit);
                for module in ["leaf", "main"] {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("SourceLoad"),
                        OutputKind::new("SourceUnit"),
                        IrWorkUnit::new(format!("{:?}:{:?}", "alpha", module)),
                    ));
                    allow_frontend(
                        &owner,
                        &WorkUnit::Module {
                            module: mizar_build::module_index::ModuleId::new(
                                PackageId::new("alpha"),
                                ModulePath::new(module),
                            ),
                        },
                    );
                }
                let mut clean_builder = PhaseRegistryBuilder::new();
                clean_builder.register_source_load();
                clean_builder.register_frontend(vec![]);
                let mut clean_driver = CompilerDriver::new(clean_builder.build().unwrap())
                    .with_output_publisher(owner.clone());
                if !clean {
                    driver = driver.with_output_publisher(owner.clone());
                }
                let active = if clean {
                    &mut clean_driver
                } else {
                    &mut driver
                };
                let submission = if clean || warmup {
                    active.submit_captured(session, &ids, selected_snapshots, input)
                } else {
                    active.submit_captured_with_native_disk_cache(
                        session,
                        &ids,
                        selected_snapshots,
                        input,
                        &policy,
                        &store,
                    )
                }
                .unwrap();
                assert_eq!(
                    submission.status,
                    if stage == 0 {
                        DriverSubmissionStatus::BlockedByMissingPhaseServices
                    } else {
                        DriverSubmissionStatus::SchedulerValidated
                    }
                );
                assert_eq!(
                    submission.session.state(),
                    BuildSessionState::Finished(if stage == 0 {
                        BuildSessionOutcome::Blocked
                    } else {
                        BuildSessionOutcome::Failed
                    })
                );
                let captured = submission.session.captured();
                let graph = submission.task_graph.as_ref().unwrap();
                let run = submission.scheduler_run.as_ref().unwrap();
                let inputs = SourceLoadInputs {
                    snapshot: captured.snapshot(),
                    captured: Some(captured),
                    build_plan: submission.build_plan.as_ref().unwrap(),
                    module_index: submission.module_index.as_ref().unwrap(),
                    allocator: &ids,
                };
                assert_eq!(inputs.validate_native_lock_content(), Ok(()));
                assert_eq!(inputs.validate_native_config_content(), Ok(()));
                let mut projection = Vec::new();
                for (ordinal, module) in ["leaf", "main"].into_iter().enumerate() {
                    let source_task = graph.tasks().iter().find(|task| task.kind == TaskKind::SourceLoad && matches!(&task.unit, WorkUnit::Module { module: id } if id.path.as_str() == module)).unwrap();
                    let frontend_task = graph
                        .tasks()
                        .iter()
                        .find(|task| {
                            task.kind == TaskKind::Frontend && task.unit == source_task.unit
                        })
                        .unwrap();
                    let version = captured
                        .snapshot()
                        .source_versions
                        .iter()
                        .find(|version| version.module_path.as_str() == module)
                        .unwrap();
                    if clean {
                        clean_ids.push(version.source_id);
                    } else if !warmup {
                        assert_ne!(version.source_id, clean_ids[ordinal]);
                    }
                    let hit = !clean && !warmup && (stage == 0 || control > 0 || module == "leaf");
                    let failed = stage > 0 && module == "main";
                    let source_result = &run.phase_results[&source_task.id];
                    let frontend_result = &run.phase_results[&frontend_task.id];
                    assert_eq!(source_result.len(), 1);
                    assert_eq!(frontend_result.len(), 1);
                    let source_result = &source_result[0];
                    let frontend_result = &frontend_result[0];
                    for (task, state, started) in [
                        (
                            source_task,
                            if hit {
                                TaskState::CacheHit
                            } else {
                                TaskState::Completed
                            },
                            !hit,
                        ),
                        (
                            frontend_task,
                            if failed {
                                TaskState::Failed
                            } else {
                                TaskState::Completed
                            },
                            true,
                        ),
                    ] {
                        assert_eq!(
                            run.task_states
                                .iter()
                                .find(|record| record.task_id == task.id)
                                .unwrap()
                                .state,
                            state
                        );
                        assert_eq!(
                            run.events
                                .iter()
                                .any(|event| event.task_id.as_ref() == Some(&task.id)
                                    && event.kind == SchedulerEventKind::TaskStarted),
                            started
                        );
                    }
                    assert_eq!(source_result.status, PhaseStatus::Complete);
                    assert_eq!(
                        frontend_result.status,
                        if failed {
                            PhaseStatus::Recoverable
                        } else {
                            PhaseStatus::Complete
                        }
                    );
                    assert_eq!(source_result.output_refs.len(), 1);
                    assert_eq!(frontend_result.output_refs.len(), usize::from(!failed));
                    assert_eq!(source_result.cache_observation.is_some(), !hit);
                    assert!(frontend_result.cache_observation.is_some());
                    let source_ref = &source_result.output_refs[0];
                    assert!(source_ref.lineage().parents.is_empty());
                    let source_handle = owner
                        .storage()
                        .typed_handle::<SourceUnit>(source_ref, &OutputKind::new("SourceUnit"))
                        .unwrap();
                    let source = owner.storage().get(&source_handle).unwrap();
                    let expected = &captured
                        .native_sources()
                        .unwrap()
                        .iter()
                        .find(|(_, _, descriptor)| descriptor.module_path == version.module_path)
                        .unwrap()
                        .1;
                    assert_eq!(source.source_id, version.source_id);
                    assert_eq!(source.source_hash, version.source_hash);
                    assert_eq!(source.origin, SourceOrigin::Disk);
                    assert_eq!(
                        source.source_text.as_ref(),
                        if module == "main" { main } else { leaf }
                    );
                    assert_eq!(source.line_map, expected.line_map);
                    assert_eq!(source.loading_map, expected.loading_map);
                    assert_eq!(source.loading_map.is_some(), module == "main" && stage < 2);
                    let source_input = PhaseInput::new(
                        source_task.unit.clone(),
                        PhaseDispatchInputBundle::without_parent_outputs(
                            snapshot,
                            source_key(version),
                            vec![],
                        ),
                    );
                    let cache_request = inputs
                        .prepare_native_disk_cache_request(&source_input, &policy)
                        .unwrap();
                    let CacheKeyBuildOutcome::Cacheable(key) =
                        CacheKeyBuilder::new(cache_request).build()
                    else {
                        panic!("current native key");
                    };
                    if !clean {
                        assert_eq!(
                            matches!(store.lookup(&key), CacheLookupOutcome::Hit(_)),
                            hit
                        );
                        if !hit {
                            let EncodeCacheRecordOutcome::Encoded(record) = inputs
                                .encode_native_disk_cache_record(
                                    &source_input,
                                    &policy,
                                    &IrCacheAdapter::new(owner.clone()),
                                    source_handle,
                                )
                                .unwrap()
                            else {
                                panic!("real scheduled source record");
                            };
                            assert_eq!(record.header.key, key);
                            assert_eq!(
                                store.insert(&record).unwrap(),
                                CacheInsertOutcome::Inserted
                            );
                            assert!(matches!(store.lookup(&key), CacheLookupOutcome::Hit(_)));
                        }
                    }
                    let parents = frontend_task
                        .dependencies
                        .iter()
                        .flat_map(|id| run.phase_results.get(id).into_iter().flatten())
                        .filter(|result| result.status == PhaseStatus::Complete)
                        .flat_map(|result| &result.output_refs)
                        .map(|parent| {
                            SealedParentOutputHandle::from_published_output(
                                &owner,
                                snapshot,
                                parent.clone(),
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();
                    let mut expected_parents = parents
                        .iter()
                        .map(|parent| parent.output())
                        .collect::<Vec<_>>();
                    expected_parents
                        .sort_by(|left, right| left.hash().as_bytes().cmp(right.hash().as_bytes()));
                    let frontend_input = PhaseInput::new(
                        source_task.unit.clone(),
                        PhaseDispatchInputBundle::new(
                            snapshot,
                            source_key(version),
                            vec![],
                            parents,
                        )
                        .unwrap(),
                    );
                    for (phase, dispatch, reason) in [
                        (
                            PipelinePhase::SourceLoad,
                            &source_input,
                            "current disk source maps require a real source load",
                        ),
                        (
                            PipelinePhase::Frontend,
                            &frontend_input,
                            "frontend inputs require a live source and dependency run",
                        ),
                    ] {
                        assert_eq!(
                            active
                                .registry()
                                .cache_key_for_phase(phase, dispatch)
                                .unwrap()
                                .intent,
                            PhaseCacheIntent::NoKey {
                                reason: reason.into()
                            }
                        );
                    }
                    let frontend = frontend_result.output_refs.first().map(|handle| {
                        assert_eq!(handle.lineage().parents, expected_parents);
                        let output = owner.storage().get(&owner.storage().typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(handle, &OutputKind::new("FrontendOutput")).unwrap()).unwrap();
                        assert_eq!(output.source, *source);
                        if module == "main" {
                            assert_eq!(handle.lineage().parents.len(), 2);
                            assert!(output.tokens.tokens().iter().any(|token| token.text.as_ref() == "combine" && token.kind == TokenKind::UserSymbol));
                        }
                        (output.canonical_disk_bytes().unwrap(), output.cache_keys.canonical_bytes().unwrap())
                    });
                    for handle in source_result
                        .output_refs
                        .iter()
                        .chain(&frontend_result.output_refs)
                    {
                        assert_eq!(
                            matches!(handle.placement(), StoragePlacement::Blob { .. }),
                            threshold == 0
                        );
                        owner.validate_published_output(snapshot, handle).unwrap();
                    }
                    let diagnostics =
                        source_result
                            .diagnostics
                            .iter()
                            .chain(&frontend_result.diagnostics)
                            .flat_map(|batch| batch.drafts())
                            .map(|draft| {
                                assert_eq!(draft.source_snapshot(), snapshot);
                                let DiagnosticPrimaryLocation::Span(span) =
                                    draft.primary_location()
                                else {
                                    panic!("real parser span");
                                };
                                let SourceAnchor::Range(mut range) = *span.anchor() else {
                                    panic!("real parser range");
                                };
                                assert_eq!(range.source_id, source.source_id);
                                source.line_map.validate_range(range).unwrap();
                                let loaded_range = mizar_session::TextRange {
                                    start: range.start,
                                    end: range.end,
                                };
                                let original = source.loading_map.as_ref().map(|map| {
                                    assert_eq!(map.source_id, source.source_id);
                                    assert_eq!(map.loaded_text_hash, source.source_hash);
                                    assert_eq!(map.loaded_text_len, source.source_text.len());
                                    map.original_range_for_loaded(source.source_id, loaded_range)
                                        .unwrap()
                                });
                                let original_range =
                                    original.map(|mapped| mapped.original).unwrap_or_else(|| {
                                        assert_eq!(stage, 2);
                                        assert_eq!(raw, source.source_text.as_ref());
                                        loaded_range
                                    });
                                let raw_offset = |offset| {
                                    if stage < 2 {
                                        3 + offset
                                            + main.as_bytes()[..offset]
                                                .iter()
                                                .filter(|byte| **byte == b'\n')
                                                .count()
                                    } else {
                                        offset
                                    }
                                };
                                assert_eq!(
                                    original_range,
                                    mizar_session::TextRange {
                                        start: raw_offset(range.start),
                                        end: raw_offset(range.end)
                                    }
                                );
                                range.source_id = fixture.version.source_id;
                                assert!(draft.secondary_spans().is_empty());
                                assert!(draft.notes().iter().all(|note| note.span().is_none()));
                                assert!(draft.details().entries().values().all(|value| matches!(
                                    value,
                                    DiagnosticDetailValue::String(_)
                                )));
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
                                    original,
                                    original_range,
                                )
                            })
                            .collect::<Vec<_>>();
                    assert_eq!(diagnostics.is_empty(), !failed);
                    projection.push((
                        module,
                        snapshot,
                        source.canonical_disk_bytes().unwrap(),
                        source_key(version),
                        key,
                        frontend,
                        (source_result.status, frontend_result.status),
                        diagnostics,
                    ));
                }
                if clean {
                    clean_projection = Some(projection);
                } else if warmup {
                    unused = Some(
                        request
                            .capture_sources(&ids, &snapshots, descriptors)
                            .unwrap(),
                    );
                } else {
                    assert_eq!(Some(&projection), clean_projection.as_ref());
                    if let Some(baseline) = baselines.get(stage) {
                        assert_eq!(
                            &projection, baseline,
                            "stage={stage}, workers={workers}, order={order:?}, reverse={reverse}"
                        );
                    } else {
                        baselines.push(projection);
                        if stage > 0 {
                            assert_eq!(baselines[stage][0].2, baselines[stage - 1][0].2);
                            assert_eq!(baselines[stage][0].4, baselines[stage - 1][0].4);
                            assert_eq!(baselines[stage][0].5, baselines[stage - 1][0].5);
                            assert_ne!(baselines[stage][1].2, baselines[stage - 1][1].2);
                            assert_ne!(baselines[stage][1].4, baselines[stage - 1][1].4);
                            if stage == 2 {
                                assert_eq!(baselines[stage][1].3, baselines[stage - 1][1].3);
                            } else {
                                assert_ne!(baselines[stage][1].3, baselines[stage - 1][1].3);
                            }
                        }
                    }
                }
            }
        }
        let (_, input) = fixture.submit_request(vec![], vec![]);
        let unused = unused.unwrap();
        assert_eq!(unused.state(), BuildSessionState::SnapshotCaptured);
        assert!(driver.session(unused.id()).is_none());
        let obsolete = driver
            .submit_captured_with_native_disk_cache(
                unused, &ids, &snapshots, input, &policy, &store,
            )
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
}

#[test]
fn native_disk_frontend_record_transports_resident_blob_and_fresh_capture_replay() {
    use mizar_cache::cache_store::{CacheInsertOutcome, CacheLookupOutcome, CacheStoreRoot};
    use mizar_ir::cache_adapter::EncodeCacheRecordOutcome;
    for threshold in [usize::MAX, 0] {
        let fixture = Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let store = CacheStoreRoot::new(fixture.root.join("frontend-records"));
        let policy = mizar_proof::policy::VerifierPolicy::release();
        let mut previous = None;
        for _ in 0..2 {
            fixture.write("\u{feff}definition\r\nend;\r\n".as_bytes());
            let (driver, submission, adapter, handle, phase_input) =
                native_disk_frontend_record_submission(&fixture, &ids, &snapshots, threshold);
            let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
            let parent = phase_input.parent_outputs()[0].clone();
            let owner = adapter.publisher().clone();
            let output = owner.storage().get(&handle).unwrap();
            assert_eq!(
                output.source.loading_map,
                inputs.captured.unwrap().native_sources().unwrap()[0]
                    .1
                    .loading_map
            );
            assert_eq!(
                output
                    .source
                    .loading_map
                    .as_ref()
                    .unwrap()
                    .original_offset_for_loaded(output.source.source_id, 0)
                    .unwrap(),
                3
            );
            assert_eq!(
                output.source.line_map,
                LineMap::new(output.source.source_id, "definition\nend;\n")
            );
            assert_eq!(
                handle.any().placement() == &StoragePlacement::Resident,
                threshold != 0
            );
            assert_eq!(handle.lineage().parents, vec![parent.output()]);
            fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
            let EncodeCacheRecordOutcome::Encoded(record) = inputs
                .encode_native_disk_frontend_cache_record(
                    &phase_input,
                    &policy,
                    &adapter,
                    handle.clone(),
                )
                .unwrap()
            else {
                panic!("scheduled Frontend record");
            };
            // Reconstruct only this post-production profile from real retained inputs.
            let mut expected = inputs
                .prepare_native_disk_cache_request(&source_input, &policy)
                .unwrap();
            expected.phase = mizar_cache::cache_key::PipelinePhase::new("Frontend");
            let schema = expected
                .schema_versions
                .iter_mut()
                .find(|entry| entry.schema_family == "phase-output")
                .unwrap();
            schema.name = "FrontendOutput".to_owned();
            schema.version =
                mizar_cache::cache_key::SchemaVersion::new("mizar-frontend/output-publication/v1");
            let mut parent_bytes = parent.as_output_ref().content_hash().as_bytes().to_vec();
            parent_bytes.extend_from_slice(parent.as_output_ref().side_table_hash().as_bytes());
            for (name, domain, bytes) in [
                (
                    "frontend-cache-keys",
                    "mizar-driver/native-disk-frontend-cache-keys/v1",
                    output.cache_keys.canonical_bytes().unwrap(),
                ),
                (
                    "source-parent",
                    "mizar-driver/native-disk-frontend-source-parent/v1",
                    parent_bytes,
                ),
            ] {
                expected
                    .input_hashes
                    .push(mizar_cache::cache_key::NamedHash {
                        name: name.to_owned(),
                        domain: domain.to_owned(),
                        digest: Hash::from_bytes(blake3::derive_key(domain, &bytes)),
                    });
            }
            for (schema_family, name, version) in [
                (
                    "frontend-keys",
                    "FrontendCacheKeys",
                    "mizar-frontend/cache-keys/v1",
                ),
                ("frontend-storage", "Disk", "mizar-frontend-output-disk-v1"),
            ] {
                expected
                    .schema_versions
                    .push(mizar_cache::cache_key::NamedSchemaVersion {
                        schema_family: schema_family.to_owned(),
                        name: name.to_owned(),
                        version: mizar_cache::cache_key::SchemaVersion::new(version),
                    });
            }
            let mizar_cache::cache_key::CacheKeyBuildOutcome::Cacheable(expected_key) =
                mizar_cache::cache_key::CacheKeyBuilder::new(expected).build()
            else {
                panic!("supported profile");
            };
            assert_eq!(record.header.key, expected_key);
            assert_eq!(record.header.key.phase.as_str(), "Frontend");
            assert_eq!(
                record.header.produced_by,
                record.header.key.validation_inputs.toolchain_compatibility
            );
            if let Some((old_source, old_handle, old_input, old_adapter, old_record)) = &previous {
                let old_handle: &mizar_ir::storage::PhaseOutputRef<
                    FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>,
                > = old_handle;
                assert_ne!(*old_source, output.source.source_id);
                assert_eq!(old_handle.output(), handle.output());
                assert!(
                    inputs
                        .encode_native_disk_frontend_cache_record(
                            old_input,
                            &policy,
                            old_adapter,
                            old_handle.clone()
                        )
                        .is_err()
                );
                assert_eq!(*old_record, record);
                assert_eq!(
                    store.insert(&record).unwrap(),
                    CacheInsertOutcome::AlreadyPresent
                );
            } else {
                assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
            }
            let CacheLookupOutcome::Hit(readback) = store.lookup(&record.header.key) else {
                panic!("record transport");
            };
            assert_eq!(readback, record);
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
            previous = Some((
                output.source.source_id,
                handle.clone(),
                phase_input.clone(),
                adapter,
                record,
            ));
        }
    }
}

#[test]
fn native_disk_frontend_records_separate_semantic_raw_and_policy_context() {
    use mizar_cache::cache_store::{CacheInsertOutcome, CacheLookupOutcome, CacheStoreRoot};
    use mizar_ir::cache_adapter::EncodeCacheRecordOutcome;
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let threshold = 0;
    let store = CacheStoreRoot::new(fixture.root.join("frontend-records"));
    let mut records = Vec::new();
    let mut sources = Vec::new();
    for case in ["base", "raw", "semantic", "policy"] {
        fixture.write(match case {
            "raw" => "\u{feff}definition\r\nend;\r\n".as_bytes(),
            "semantic" => b"theorem T: a = a;\n",
            _ => b"definition\nend;\n",
        });
        let (_driver, submission, adapter, handle, phase_input) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, threshold);
        let (inputs, _) = native_disk_lookup_context(&submission, &ids);
        let owner = adapter.publisher().clone();
        let output = owner.storage().get(&handle).unwrap();
        let policy = if case == "policy" {
            mizar_proof::policy::VerifierPolicy::development()
        } else {
            mizar_proof::policy::VerifierPolicy::release()
        };
        let EncodeCacheRecordOutcome::Encoded(record) = inputs
            .encode_native_disk_frontend_cache_record(
                &phase_input,
                &policy,
                &adapter,
                handle.clone(),
            )
            .unwrap()
        else {
            panic!("actual changed-input record");
        };
        assert!(matches!(
            store.lookup(&record.header.key),
            CacheLookupOutcome::Miss(_)
        ));
        assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
        let CacheLookupOutcome::Hit(readback) = store.lookup(&record.header.key) else {
            panic!("actual record transport");
        };
        assert_eq!(record, readback);
        sources.push((
            output.source.source_hash,
            output.source.canonical_disk_bytes().unwrap(),
        ));
        records.push(record);
    }
    assert_eq!(sources[0].0, sources[1].0);
    assert_ne!(sources[0].1, sources[1].1);
    assert_ne!(sources[0].0, sources[2].0);
    assert_eq!(sources[0].1, sources[3].1);
    assert_eq!(sources[0].0, sources[3].0);
    for (index, record) in records.iter().enumerate() {
        for other in &records[index + 1..] {
            assert_ne!(record.header.key, other.header.key);
        }
    }
    assert_ne!(records[0].output, records[1].output);
    assert_ne!(records[0].output, records[2].output);
    assert_eq!(records[0].output, records[3].output);
}

#[test]
fn native_disk_frontend_record_rejects_current_physical_parent_capture_and_profiles() {
    use mizar_build::planner::{lockfile_content_hash, workspace_config_content_hash};
    use mizar_ir::cache_adapter::EncodeCacheRecordOutcome;
    use mizar_ir::{
        cache_adapter::IrCacheAdapter,
        publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
        storage::{BlobDecodeError, BlobDecoder, CollectInput, SchemaVersion, SideTableRecord},
    };
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let policy = mizar_proof::policy::VerifierPolicy::release();
    for case in [
        "capture",
        "dispatch",
        "dependency",
        "indexed-summary-module",
        "missing-parent",
        "wrong-parent",
        "wrong-frontend-side",
        "extra-frontend-side",
        "wrong-source-parent-side",
        "internal-source-parent",
        "foreign",
        "unpublished",
        "obsolete",
        "collected-output",
        "collected-parent",
        "lock",
        "config",
    ] {
        let threshold = if case == "collected-parent" {
            0
        } else {
            usize::MAX
        };
        let (_driver, submission, adapter, handle, phase_input) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, threshold);
        let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
        let snapshot = phase_input.snapshot;
        let unit = phase_input.work_unit.clone();
        let parent = phase_input.parent_outputs()[0].clone();
        let descriptor = inputs.captured.unwrap().native_sources().unwrap()[0]
            .2
            .clone();
        let owner = adapter.publisher().clone();
        assert!(matches!(
            inputs
                .encode_native_disk_frontend_cache_record(
                    &phase_input,
                    &policy,
                    &adapter,
                    handle.clone()
                )
                .unwrap(),
            EncodeCacheRecordOutcome::Encoded(_)
        ));
        let mut rejected_input = phase_input.clone();
        let mut rejected_handle = handle.clone();
        let mut rejected_inputs = inputs;
        let mut plan = inputs.build_plan.clone();
        let mut index = inputs.module_index.clone();
        let alternate;
        let selected_adapter = match case {
            "capture" => {
                rejected_inputs.captured = None;
                &adapter
            }
            "dispatch" | "dependency" | "missing-parent" | "wrong-parent" => {
                let parents = match case {
                    "missing-parent" => vec![],
                    "wrong-parent" => vec![
                        SealedParentOutputHandle::from_current_output(
                            &owner,
                            snapshot,
                            handle.erase(),
                        )
                        .unwrap(),
                    ],
                    _ => vec![parent.clone()],
                };
                rejected_input = PhaseInput::new(
                    unit.clone(),
                    PhaseDispatchInputBundle::new(
                        snapshot,
                        if case == "dispatch" {
                            hash(0xec)
                        } else {
                            source_input.identities().input_hash()
                        },
                        if case == "dependency" {
                            vec![hash(0xed)]
                        } else {
                            vec![]
                        },
                        parents,
                    )
                    .unwrap(),
                );
                &adapter
            }
            "wrong-frontend-side"
            | "extra-frontend-side"
            | "wrong-source-parent-side"
            | "internal-source-parent" => {
                // These republished handles are adversarial negative controls only.
                let output = owner.storage().get(&handle).unwrap();
                let genuine_owner = &owner;
                let owner = publisher(snapshot, usize::MAX, &unit);
                allow_frontend(&owner, &unit);
                let bound_parent;
                {
                    let source_handle = genuine_owner
                        .storage()
                        .typed_handle::<SourceUnit>(
                            parent.as_output_ref(),
                            &OutputKind::new("SourceUnit"),
                        )
                        .unwrap();
                    let source = genuine_owner.storage().get(&source_handle).unwrap();
                    let mut side_tables =
                        (*genuine_owner.storage().side_tables(&source_handle).unwrap()).clone();
                    if case == "wrong-source-parent-side" {
                        side_tables.source_maps[0].digest = hash(0xf0);
                    }
                    let decode_input = descriptor.clone();
                    let source_id = source.source_id;
                    let negative_source = owner
                        .publish(PublishOutputInput {
                            slot: owner.allocate(
                                snapshot,
                                source_handle.phase().clone(),
                                source_handle.work_unit().clone(),
                                OutputKind::new("SourceUnit"),
                                SchemaVersion::new(1),
                            ),
                            snapshot,
                            phase: source_handle.phase().clone(),
                            work_unit: source_handle.work_unit().clone(),
                            output_kind: OutputKind::new("SourceUnit"),
                            schema_version: SchemaVersion::new(1),
                            payload: (*source).clone(),
                            canonical_payload: Some(
                                source_key(&inputs.snapshot.source_versions[0])
                                    .as_bytes()
                                    .to_vec(),
                            ),
                            storage_payload: Some(source.canonical_disk_bytes().unwrap()),
                            decode: BlobDecoder::new(move |bytes| {
                                SourceUnit::from_canonical_storage_bytes(
                                    bytes,
                                    source_id,
                                    &decode_input,
                                )
                                .ok_or_else(|| {
                                    BlobDecodeError::new("invalid negative source bytes")
                                })
                            }),
                            parents: vec![],
                            named_input_hashes: source_handle.lineage().named_input_hashes.clone(),
                            side_tables,
                            origin: if case == "internal-source-parent" {
                                OutputOrigin::OpenBuffer
                            } else {
                                OutputOrigin::PackageSource
                            },
                            target: if case == "internal-source-parent" {
                                PublicationTarget::InternalOnly
                            } else {
                                PublicationTarget::CurrentPackage
                            },
                        })
                        .unwrap();
                    assert_eq!(*owner.storage().get(&negative_source).unwrap(), *source);
                    assert_eq!(
                        negative_source.lineage().parents,
                        source_handle.lineage().parents
                    );
                    assert_eq!(
                        negative_source.lineage().named_input_hashes,
                        source_handle.lineage().named_input_hashes
                    );
                    assert!(
                        owner
                            .validate_published_output(snapshot, negative_source.any())
                            .is_ok()
                    );
                    assert_eq!(
                        owner
                            .validate_current_output(snapshot, negative_source.any())
                            .is_ok(),
                        case != "internal-source-parent",
                    );
                    assert!(
                        owner
                            .storage()
                            .validate_handle(negative_source.any())
                            .is_ok()
                    );
                    bound_parent = SealedParentOutputHandle::from_published_output(
                        &owner,
                        snapshot,
                        negative_source.erase(),
                    )
                    .unwrap();
                }
                let mut side_tables =
                    (*genuine_owner.storage().side_tables(&handle).unwrap()).clone();
                if case == "wrong-frontend-side" {
                    side_tables.source_maps[0].digest = hash(0xf1);
                } else if case == "extra-frontend-side" {
                    side_tables
                        .documentation_attachments
                        .push(SideTableRecord::new(
                            "negative-only-extra",
                            "unexpected",
                            hash(0xf2),
                        ));
                }
                let mut semantic = b"mizar-frontend/output-publication/v1\0".to_vec();
                semantic.extend_from_slice(
                    output
                        .cache_keys
                        .ast
                        .as_ref()
                        .unwrap()
                        .stable_hash()
                        .as_bytes(),
                );
                let decode_input = descriptor.clone();
                let source_id = output.source.source_id;
                rejected_handle = owner
                    .publish(PublishOutputInput {
                        slot: owner.allocate(
                            snapshot,
                            handle.phase().clone(),
                            handle.work_unit().clone(),
                            OutputKind::new("FrontendOutput"),
                            SchemaVersion::new(1),
                        ),
                        snapshot,
                        phase: handle.phase().clone(),
                        work_unit: handle.work_unit().clone(),
                        output_kind: OutputKind::new("FrontendOutput"),
                        schema_version: SchemaVersion::new(1),
                        payload: (*output).clone(),
                        canonical_payload: Some(semantic),
                        storage_payload: Some(output.canonical_disk_bytes().unwrap()),
                        decode: BlobDecoder::new(move |bytes| {
                            FrontendOutput::from_canonical_storage_bytes(
                                bytes,
                                source_id,
                                &decode_input,
                            )
                            .ok_or_else(|| BlobDecodeError::new("invalid negative frontend bytes"))
                        }),
                        parents: vec![bound_parent.as_output_ref().clone()],
                        named_input_hashes: handle.lineage().named_input_hashes.clone(),
                        side_tables,
                        origin: OutputOrigin::PackageSource,
                        target: PublicationTarget::CurrentPackage,
                    })
                    .unwrap();
                assert_eq!(*owner.storage().get(&rejected_handle).unwrap(), *output);
                assert_eq!(
                    rejected_handle.lineage().parents,
                    vec![bound_parent.output()]
                );
                assert_eq!(
                    rejected_handle.lineage().named_input_hashes,
                    handle.lineage().named_input_hashes
                );
                assert!(
                    owner
                        .validate_current_output(snapshot, rejected_handle.any())
                        .is_ok()
                );
                assert!(
                    owner
                        .storage()
                        .validate_handle(rejected_handle.any())
                        .is_ok()
                );
                if case == "wrong-source-parent-side" {
                    assert_eq!(
                        owner.storage().side_tables(&rejected_handle).unwrap(),
                        genuine_owner.storage().side_tables(&handle).unwrap()
                    );
                }
                rejected_input = PhaseInput::new(
                    unit.clone(),
                    PhaseDispatchInputBundle::new(
                        snapshot,
                        source_input.identities().input_hash(),
                        vec![],
                        vec![bound_parent],
                    )
                    .unwrap(),
                );
                alternate = IrCacheAdapter::new(owner);
                &alternate
            }
            "indexed-summary-module" => {
                // Adversarial index entry is deliberately absent from both backing tables.
                let mut extra = index.modules[0].clone();
                extra.module = mizar_build::module_index::ModuleId::new(
                    PackageId::new("alpha"),
                    ModulePath::new("unbacked"),
                );
                extra.module_path = ModulePath::new("unbacked");
                extra.location = ModuleIndexLocation::DependencySummary {
                    artifact: "negative-only-unbacked-summary".to_owned(),
                    content_hash: hash(0xee),
                };
                index.modules.push(extra);
                assert!(index.dependency_summaries.is_empty());
                assert!(inputs.snapshot.dependency_artifacts.is_empty());
                rejected_inputs.module_index = &index;
                &adapter
            }
            "foreign" => {
                alternate = IrCacheAdapter::new(publisher(snapshot, threshold, &unit));
                &alternate
            }
            "unpublished" => {
                alternate = IrCacheAdapter::new(std::sync::Arc::new(PhaseOutputPublisher::new(
                    owner.storage().clone(),
                    owner.registry().clone(),
                )));
                alternate.publisher().register_current_snapshot(snapshot);
                &alternate
            }
            "obsolete" => {
                owner.mark_obsolete(snapshot).unwrap();
                &adapter
            }
            "collected-output" | "collected-parent" => {
                owner.storage().collect(CollectInput {
                    snapshot,
                    protected_outputs: if case == "collected-parent" {
                        vec![handle.output()]
                    } else {
                        vec![parent.output()]
                    },
                });
                let missing = if case == "collected-parent" {
                    parent.as_output_ref()
                } else {
                    handle.any()
                };
                assert!(owner.storage().validate_handle(missing).is_err());
                assert!(
                    owner
                        .validate_current_output(snapshot, handle.any())
                        .is_ok()
                );
                &adapter
            }
            "lock" | "config" => {
                if case == "lock" {
                    plan.lockfile.schema_version += 1;
                } else {
                    plan.verifier_config.packages[0].config.max_axioms += 1;
                }
                rejected_inputs.build_plan = &plan;
                &adapter
            }
            _ => unreachable!(),
        };
        let rejected = rejected_inputs.encode_native_disk_frontend_cache_record(
            &rejected_input,
            &policy,
            selected_adapter,
            rejected_handle,
        );
        match case {
            "wrong-frontend-side" | "extra-frontend-side" => {
                assert_eq!(
                    rejected.unwrap_err(),
                    "native Frontend encoding side tables mismatch"
                );
            }
            "wrong-source-parent-side" => {
                assert_eq!(
                    rejected.unwrap_err(),
                    "native Frontend encoding source parent lineage mismatch"
                );
            }
            _ => assert!(rejected.is_err(), "{case}"),
        }
    }
    for open in [false, true] {
        let raw = if open {
            "definition\nend;\n"
        } else {
            "import alpha.leaf;\ndefinition\nend;\n"
        };
        fixture.write(raw.as_bytes());
        fs::write(
            fixture.root.join("alpha/src/leaf.miz"),
            b"definition\nend;\n",
        )
        .unwrap();
        let (mut draft, input) = fixture.submit_request(vec![], vec![]);
        let plan = produce_build_plan(
            input.plan_request.clone(),
            input.workspace_packages.clone(),
            input.lockfile.clone(),
        )
        .unwrap();
        draft.source_inputs.versions.clear();
        draft.dependency_inputs.lockfile_hash = lockfile_content_hash(&plan.lockfile);
        draft.verifier_config = VerifierConfigInput::new(workspace_config_content_hash(
            &plan.verifier_config,
            &plan.build_config,
        ));
        let modules = if open {
            vec!["main"]
        } else {
            vec!["main", "leaf"]
        };
        let descriptors = modules
            .iter()
            .map(|module| {
                let path = format!("src/{module}.miz");
                (
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    SourceInput {
                        package_id: PackageId::new("alpha"),
                        module_path: ModulePath::new(*module),
                        normalized_path: normalize_path(
                            &fixture.root.join("alpha"),
                            Path::new(&path),
                        )
                        .unwrap(),
                        edition: Edition::new("2025"),
                        origin: if open {
                            SourceOriginInput::OpenBuffer {
                                uri: format!(
                                    "file://{}",
                                    fixture.root.join("alpha/src/main.miz").display()
                                ),
                                expected_version: 7,
                                actual_version: 7,
                                text: std::sync::Arc::from(raw),
                            }
                        } else {
                            SourceOriginInput::Disk { path: path.into() }
                        },
                    },
                )
            })
            .collect();
        let mut input = input;
        if !open {
            input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
                package_id: PackageId::new("alpha"),
                files: modules
                    .iter()
                    .map(|module| {
                        WorkspaceSourceFile::new(
                            format!("src/{module}.miz"),
                            format!("{module}.miz"),
                        )
                    })
                    .collect(),
            }]);
            input.dependency_overlay = complete_leaf_import_overlay();
        }
        let session = draft
            .capture_sources(&ids, &snapshots, descriptors)
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        let owner = publisher(snapshot, 0, &unit);
        for module in &modules {
            let unit = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("alpha"),
                    ModulePath::new(*module),
                ),
            };
            owner.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("SourceLoad"),
                OutputKind::new("SourceUnit"),
                IrWorkUnit::new(format!("{:?}:{:?}", "alpha", module)),
            ));
            allow_frontend(&owner, &unit);
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
        let submission = driver
            .submit_captured(session, &ids, &snapshots, input)
            .unwrap();
        let captured = submission.session.captured();
        let version = captured
            .snapshot()
            .source_versions
            .iter()
            .find(|version| version.module_path == ModulePath::new("main"))
            .unwrap();
        let refs = submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .flat_map(|result| &result.output_refs)
            .collect::<Vec<_>>();
        let work_unit = IrWorkUnit::new(format!("{:?}:{:?}", "alpha", "main"));
        let source = refs
            .iter()
            .find(|handle| {
                handle.work_unit() == &work_unit
                    && handle.output_kind() == &OutputKind::new("SourceUnit")
            })
            .unwrap();
        let handle = owner
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                refs.iter()
                    .find(|handle| {
                        handle.work_unit() == &work_unit
                            && handle.output_kind() == &OutputKind::new("FrontendOutput")
                    })
                    .unwrap(),
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        assert_eq!(
            !owner
                .storage()
                .get(&handle)
                .unwrap()
                .preprocessed
                .import_stubs
                .is_empty(),
            !open
        );
        let inputs = SourceLoadInputs {
            snapshot: captured.snapshot(),
            captured: Some(captured),
            build_plan: submission.build_plan.as_ref().unwrap(),
            module_index: submission.module_index.as_ref().unwrap(),
            allocator: &ids,
        };
        let phase_input = PhaseInput::new(
            unit,
            PhaseDispatchInputBundle::new(
                snapshot,
                source_key(version),
                vec![],
                vec![
                    SealedParentOutputHandle::from_published_output(
                        &owner,
                        snapshot,
                        (*source).clone(),
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        );
        assert!(
            inputs
                .encode_native_disk_frontend_cache_record(
                    &phase_input,
                    &policy,
                    &IrCacheAdapter::new(owner.clone()),
                    handle
                )
                .is_err()
        );
    }
}

#[test]
fn native_disk_frontend_lookup_reconstructs_resident_blob_and_clean_aggregate() {
    use mizar_cache::{
        cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder},
        cache_store::{CacheInsertOutcome, CacheLookupOutcome, CacheStoreRoot},
    };
    use mizar_ir::{
        cache_adapter::{CacheAdapterMiss, CacheRehydrateOutcome, EncodeCacheRecordOutcome},
        publisher::PublishError,
        storage::StorageError,
    };
    for threshold in [usize::MAX, 0] {
        let fixture = Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (driver, submission, oracle, handle, oracle_input) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, threshold);
        let (inputs, _) = native_disk_lookup_context(&submission, &ids);
        let policy = mizar_proof::policy::VerifierPolicy::release();
        let request = inputs
            .prepare_native_disk_frontend_cache_request(&oracle_input, &policy, &oracle, &handle)
            .unwrap();
        let CacheKeyBuildOutcome::Cacheable(key) = CacheKeyBuilder::new(request.clone()).build()
        else {
            panic!("current native leaf profile");
        };
        let EncodeCacheRecordOutcome::Encoded(record) = inputs
            .encode_native_disk_frontend_cache_record(
                &oracle_input,
                &policy,
                &oracle,
                handle.clone(),
            )
            .unwrap()
        else {
            panic!("genuine scheduled Frontend record");
        };
        assert_eq!(key, record.header.key);
        assert_eq!(
            record.header.produced_by,
            request.validation_inputs.toolchain_compatibility
        );
        assert_eq!(key.phase.as_str(), "Frontend");
        let store = CacheStoreRoot::new(fixture.root.join("retained-frontend"));
        assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
        assert!(matches!(store.lookup(&key), CacheLookupOutcome::Hit(found) if found == record));
        assert!(matches!(
            inputs
                .rehydrate_native_disk_frontend_cache(
                    &oracle_input,
                    &policy,
                    &store,
                    (&oracle_input, &oracle, &handle),
                    &oracle,
                )
                .unwrap(),
            CacheRehydrateOutcome::Miss(CacheAdapterMiss::Publisher { error })
                if matches!(error.as_ref(), PublishError::Storage { error }
                    if matches!(error.as_ref(), StorageError::OutputAlreadyStored { .. }))
        ));
        assert_eq!(oracle.successful_rehydrations(), 0);
        let expected = oracle.publisher().storage().get(&handle).unwrap();
        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
        let (clean, clean_input) = native_disk_frontend_lookup_target(&submission, &ids, threshold);
        allow_frontend(clean.publisher(), &clean_input.work_unit);
        let clean_result = driver
            .registry()
            .execute_phase_with_resources(
                PipelinePhase::Frontend,
                clean_input.clone(),
                PhaseExecutionResources {
                    diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                        DiagnosticPhase::Frontend,
                        clean_input.snapshot,
                        "frontend.run_loaded",
                    ))),
                    output_publisher: Some(clean.publisher().clone()),
                    source_load: Some(inputs),
                    ..PhaseExecutionResources::default()
                },
            )
            .unwrap()
            .result;
        assert!(clean_result.diagnostics.is_empty());
        let clean_handle = clean
            .publisher()
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                &clean_result.output_refs[0],
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let clean_output = clean.publisher().storage().get(&clean_handle).unwrap();
        assert_eq!(*clean_output, *expected);
        for _ in 0..2 {
            let (target, input) = native_disk_frontend_lookup_target(&submission, &ids, threshold);
            assert!(!std::sync::Arc::ptr_eq(
                target.publisher().storage(),
                oracle.publisher().storage()
            ));
            assert!(
                target
                    .publisher()
                    .storage()
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        handle.any(),
                        &OutputKind::new("FrontendOutput"),
                    )
                    .is_err()
            );
            allow_frontend(target.publisher(), &input.work_unit);
            assert_eq!(
                input.parent_outputs()[0].output(),
                oracle_input.parent_outputs()[0].output()
            );
            let CacheRehydrateOutcome::Rehydrated(rehydrated) = inputs
                .rehydrate_native_disk_frontend_cache(
                    &input,
                    &policy,
                    &store,
                    (&oracle_input, &oracle, &handle),
                    &target,
                )
                .unwrap()
            else {
                panic!("real retained lookup");
            };
            let actual = target.publisher().storage().get(&rehydrated).unwrap();
            assert_eq!(*actual, *clean_output);
            assert_eq!(
                actual.source.source_id,
                inputs.snapshot.source_versions[0].source_id
            );
            assert_eq!(
                actual.source.line_map,
                LineMap::new(actual.source.source_id, "definition\nend;\n")
            );
            assert_eq!(
                actual
                    .source
                    .loading_map
                    .as_ref()
                    .unwrap()
                    .original_offset_for_loaded(actual.source.source_id, 0)
                    .unwrap(),
                3
            );
            assert_eq!(rehydrated.lineage(), handle.lineage());
            assert_eq!(
                target
                    .publisher()
                    .storage()
                    .side_tables(&rehydrated)
                    .unwrap(),
                oracle.publisher().storage().side_tables(&handle).unwrap()
            );
            assert_eq!(target.successful_rehydrations(), 1);
            for (phase, reason) in [
                (
                    PipelinePhase::SourceLoad,
                    "current disk source maps require a real source load",
                ),
                (
                    PipelinePhase::Frontend,
                    "frontend inputs require a live source and dependency run",
                ),
            ] {
                assert_eq!(
                    driver
                        .registry()
                        .cache_key_for_phase(phase, &input)
                        .unwrap()
                        .intent,
                    PhaseCacheIntent::NoKey {
                        reason: reason.into()
                    }
                );
            }
            assert!(matches!(
                inputs
                    .rehydrate_native_disk_frontend_cache(
                        &input,
                        &policy,
                        &store,
                        (&oracle_input, &oracle, &handle),
                        &target,
                    )
                    .unwrap(),
                CacheRehydrateOutcome::Miss(CacheAdapterMiss::Publisher { error })
                    if matches!(error.as_ref(), PublishError::Storage { error }
                        if matches!(error.as_ref(), StorageError::OutputAlreadyStored { .. }))
            ));
            assert_eq!(target.successful_rehydrations(), 1);
        }
    }
}

#[test]
fn native_disk_frontend_lookup_misses_changed_source_raw_policy_and_context() {
    use mizar_cache::cache_store::{CacheInsertOutcome, CacheStoreRoot};
    use mizar_ir::cache_adapter::{CacheRehydrateOutcome, EncodeCacheRecordOutcome};
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let store = CacheStoreRoot::new(fixture.root.join("retained-frontend"));
    let release = mizar_proof::policy::VerifierPolicy::release();
    let mut baseline = None;
    for case in [
        "base",
        "raw",
        "semantic",
        "policy",
        "config-binding",
        "context",
    ] {
        fixture.write(match case {
            "raw" => "\u{feff}definition\r\nend;\r\n".as_bytes(),
            "semantic" => b"theorem T: a = a;\n",
            _ => b"definition\nend;\n",
        });
        let (_driver, submission, oracle, handle, oracle_input) = if case == "context" {
            use mizar_build::planner::{lockfile_content_hash, workspace_config_content_hash};
            let (mut draft, mut input) = fixture.submit_request(vec![], vec![]);
            input.workspace_packages[0].manifest.verifier.max_axioms += 1;
            let plan = produce_build_plan(
                input.plan_request.clone(),
                input.workspace_packages.clone(),
                input.lockfile.clone(),
            )
            .unwrap();
            draft.source_inputs.versions.clear();
            draft.dependency_inputs.lockfile_hash = lockfile_content_hash(&plan.lockfile);
            draft.verifier_config = VerifierConfigInput::new(workspace_config_content_hash(
                &plan.verifier_config,
                &plan.build_config,
            ));
            let session = draft
                .capture_sources(
                    &ids,
                    &snapshots,
                    vec![(
                        DiskSourceLoader::new(fixture.root.join("alpha")),
                        SourceInput {
                            package_id: fixture.version.package_id.clone(),
                            module_path: fixture.version.module_path.clone(),
                            normalized_path: fixture.version.normalized_path.clone(),
                            edition: fixture.version.edition.clone(),
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
            let mut driver =
                CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
            let submission = driver
                .submit_captured(session, &ids, &snapshots, input)
                .unwrap();
            let (_, source_input) = native_disk_lookup_context(&submission, &ids);
            let refs = submission
                .scheduler_run
                .as_ref()
                .unwrap()
                .phase_results
                .values()
                .flatten()
                .flat_map(|result| &result.output_refs)
                .collect::<Vec<_>>();
            let source = refs
                .iter()
                .find(|output| output.output_kind() == &OutputKind::new("SourceUnit"))
                .unwrap();
            let parent = SealedParentOutputHandle::from_current_output(
                &owner,
                source_input.snapshot,
                (*source).clone(),
            )
            .unwrap();
            let handle = owner
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    refs.iter()
                        .find(|output| output.output_kind() == &OutputKind::new("FrontendOutput"))
                        .unwrap(),
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let oracle_input = PhaseInput::new(
                unit,
                PhaseDispatchInputBundle::new(
                    source_input.snapshot,
                    source_input.identities().input_hash(),
                    vec![],
                    vec![parent],
                )
                .unwrap(),
            );
            (
                driver,
                submission,
                mizar_ir::cache_adapter::IrCacheAdapter::new(owner),
                handle,
                oracle_input,
            )
        } else {
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0)
        };
        let (inputs, _) = native_disk_lookup_context(&submission, &ids);
        let (target, input) = native_disk_frontend_lookup_target(&submission, &ids, 0);
        allow_frontend(target.publisher(), &input.work_unit);
        let policy = if case == "policy" {
            mizar_proof::policy::VerifierPolicy::development()
        } else {
            release.clone()
        };
        if case == "config-binding" {
            // Adversarial plan mutation is only a capture/config binding rejection.
            let mut plan = inputs.build_plan.clone();
            plan.verifier_config.packages[0].config.max_axioms += 1;
            let changed = SourceLoadInputs {
                build_plan: &plan,
                ..inputs
            };
            assert!(
                changed
                    .rehydrate_native_disk_frontend_cache(
                        &input,
                        &policy,
                        &store,
                        (&oracle_input, &oracle, &handle),
                        &target
                    )
                    .is_err()
            );
            continue;
        }
        let EncodeCacheRecordOutcome::Encoded(record) = inputs
            .encode_native_disk_frontend_cache_record(
                &oracle_input,
                &policy,
                &oracle,
                handle.clone(),
            )
            .unwrap()
        else {
            panic!("genuine changed-input record");
        };
        if case == "base" {
            assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
            baseline = Some(record.header.key.clone());
        } else {
            assert_ne!(&record.header.key, baseline.as_ref().unwrap());
            assert!(matches!(
                inputs
                    .rehydrate_native_disk_frontend_cache(
                        &input,
                        &policy,
                        &store,
                        (&oracle_input, &oracle, &handle),
                        &target,
                    )
                    .unwrap(),
                CacheRehydrateOutcome::Miss(_)
            ));
            assert_eq!(target.successful_rehydrations(), 0);
        }
    }
}

#[test]
fn native_disk_frontend_lookup_rejects_permissions_parents_and_coherent_wrong_aggregate() {
    use mizar_cache::cache_store::{CacheInsertOutcome, CacheStoreRoot};
    use mizar_ir::{
        cache_adapter::{
            CacheAdapterCacheability, CacheAdapterMiss, CacheRehydrateOutcome,
            EncodeCacheRecordInput, EncodeCacheRecordOutcome, IrCacheAdapter,
        },
        publisher::{OutputOrigin, PublicationTarget, PublishError, PublishOutputInput},
        storage::{BlobDecodeError, BlobDecoder, CollectInput, SchemaVersion},
    };
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let policy = mizar_proof::policy::VerifierPolicy::release();
    for case in [
        "permission",
        "foreign",
        "unpublished",
        "mixed-source",
        "obsolete",
        "collected",
        "oracle-obsolete",
        "oracle-collected",
        "mixed-capture",
        "missing-parent",
        "wrong-parent",
        "producer",
        "corrupt",
        "coherent",
    ] {
        let (_driver, submission, oracle, handle, oracle_input) =
            native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
        let (inputs, _) = native_disk_lookup_context(&submission, &ids);
        let (target, mut input) = native_disk_frontend_lookup_target(&submission, &ids, 0);
        if case != "permission" {
            allow_frontend(target.publisher(), &input.work_unit);
        }
        let EncodeCacheRecordOutcome::Encoded(mut record) = inputs
            .encode_native_disk_frontend_cache_record(
                &oracle_input,
                &policy,
                &oracle,
                handle.clone(),
            )
            .unwrap()
        else {
            panic!("genuine source record");
        };
        let alternate;
        let selected = if case == "foreign" {
            alternate = IrCacheAdapter::new(publisher(input.snapshot, 0, &input.work_unit));
            &alternate
        } else if case == "unpublished" {
            alternate = IrCacheAdapter::new(std::sync::Arc::new(PhaseOutputPublisher::new(
                target.publisher().storage().clone(),
                target.publisher().registry().clone(),
            )));
            alternate
                .publisher()
                .register_current_snapshot(input.snapshot);
            &alternate
        } else {
            &target
        };
        match case {
            "obsolete" => target.publisher().mark_obsolete(input.snapshot).unwrap(),
            "oracle-obsolete" => oracle.publisher().mark_obsolete(input.snapshot).unwrap(),
            "collected" | "oracle-collected" => {
                let owner = if case == "collected" {
                    target.publisher()
                } else {
                    oracle.publisher()
                };
                owner.storage().collect(CollectInput {
                    snapshot: input.snapshot,
                    protected_outputs: vec![],
                });
            }
            "missing-parent" | "wrong-parent" => {
                input = PhaseInput::new(
                    input.work_unit.clone(),
                    PhaseDispatchInputBundle::new(
                        input.snapshot,
                        input.identities().input_hash(),
                        vec![],
                        if case == "wrong-parent" {
                            vec![
                                SealedParentOutputHandle::from_current_output(
                                    oracle.publisher(),
                                    input.snapshot,
                                    handle.erase(),
                                )
                                .unwrap(),
                            ]
                        } else {
                            vec![]
                        },
                    )
                    .unwrap(),
                );
            }
            "producer" => {
                record.header.produced_by[0].value = "negative-only-other-producer".into()
            }
            "corrupt" => {
                *record = mizar_cache::cache_store::CacheRecord::new_inline(
                    record.header.key.clone(),
                    record.header.produced_by.clone(),
                    vec![],
                );
            }
            "coherent" => {
                // This is an adversarial aggregate, never a real Frontend producer.
                let actual = oracle.publisher().storage().get(&handle).unwrap();
                let mut wrong = (*actual).clone();
                wrong.tokens.tokens[0].text = "negative-only-token".into();
                let raw = wrong.canonical_disk_bytes().unwrap();
                assert_ne!(raw, actual.canonical_disk_bytes().unwrap());
                let descriptor = inputs.captured.unwrap().native_sources().unwrap()[0]
                    .2
                    .clone();
                let source_id = actual.source.source_id;
                let decoded =
                    FrontendOutput::from_canonical_storage_bytes(&raw, source_id, &descriptor)
                        .unwrap();
                assert_eq!(decoded, wrong);
                assert_eq!(decoded.canonical_disk_bytes().unwrap(), raw);
                let mut semantic = b"mizar-frontend/output-publication/v1\0".to_vec();
                semantic.extend_from_slice(
                    actual
                        .cache_keys
                        .ast
                        .as_ref()
                        .unwrap()
                        .stable_hash()
                        .as_bytes(),
                );
                let side = (*oracle.publisher().storage().side_tables(&handle).unwrap()).clone();
                let decode_input = descriptor;
                let negative = target
                    .publisher()
                    .publish(PublishOutputInput {
                        slot: target.publisher().allocate(
                            input.snapshot,
                            handle.phase().clone(),
                            handle.work_unit().clone(),
                            handle.output_kind().clone(),
                            SchemaVersion::new(1),
                        ),
                        snapshot: input.snapshot,
                        phase: handle.phase().clone(),
                        work_unit: handle.work_unit().clone(),
                        output_kind: handle.output_kind().clone(),
                        schema_version: SchemaVersion::new(1),
                        payload: wrong,
                        canonical_payload: Some(semantic.clone()),
                        storage_payload: Some(raw.clone()),
                        decode: BlobDecoder::new(move |bytes| {
                            FrontendOutput::from_canonical_storage_bytes(
                                bytes,
                                source_id,
                                &decode_input,
                            )
                            .ok_or_else(|| BlobDecodeError::new("negative aggregate"))
                        }),
                        parents: vec![input.parent_outputs()[0].as_output_ref().clone()],
                        named_input_hashes: handle.lineage().named_input_hashes.clone(),
                        side_tables: side.clone(),
                        origin: OutputOrigin::PackageSource,
                        target: PublicationTarget::CurrentPackage,
                    })
                    .unwrap();
                assert_eq!(
                    target
                        .publisher()
                        .storage()
                        .side_tables(&negative)
                        .unwrap()
                        .as_ref(),
                    &side
                );
                assert_eq!(negative.content_hash(), handle.content_hash());
                let EncodeCacheRecordOutcome::Encoded(adversarial) =
                    target.encode(EncodeCacheRecordInput {
                        handle: negative,
                        cache_key: record.header.key.clone(),
                        produced_by: record.header.produced_by.clone(),
                        canonical_payload: semantic,
                        storage_payload: raw,
                        cacheability: CacheAdapterCacheability::Cacheable,
                    })
                else {
                    panic!("self-consistent adversarial control");
                };
                record = adversarial;
            }
            _ => {}
        }
        let store = CacheStoreRoot::new(fixture.root.join(format!("retained-{case}")));
        assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
        if case == "mixed-source" {
            let (_, fresh_submission, _, _, _) =
                native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
            let (fresh_target, fresh_input) =
                native_disk_frontend_lookup_target(&fresh_submission, &ids, 0);
            allow_frontend(fresh_target.publisher(), &fresh_input.work_unit);
            assert_eq!(fresh_input.identities(), oracle_input.identities());
            assert!(
                inputs
                    .rehydrate_native_disk_frontend_cache(
                        &fresh_input,
                        &policy,
                        &store,
                        (&oracle_input, &oracle, &handle),
                        &fresh_target
                    )
                    .is_err()
            );
            assert_eq!(fresh_target.successful_rehydrations(), 0);
        } else if case == "mixed-capture" {
            let (_, fresh_submission, _, _, _) =
                native_disk_frontend_record_submission(&fixture, &ids, &snapshots, 0);
            let (fresh_inputs, _) = native_disk_lookup_context(&fresh_submission, &ids);
            assert_eq!(fresh_inputs.snapshot.id, inputs.snapshot.id);
            assert_ne!(
                fresh_inputs.snapshot.source_versions[0].source_id,
                inputs.snapshot.source_versions[0].source_id
            );
            assert!(
                fresh_inputs
                    .rehydrate_native_disk_frontend_cache(
                        &input,
                        &policy,
                        &store,
                        (&oracle_input, &oracle, &handle),
                        selected
                    )
                    .is_err()
            );
        } else {
            let outcome = inputs.rehydrate_native_disk_frontend_cache(
                &input,
                &policy,
                &store,
                (&oracle_input, &oracle, &handle),
                selected,
            );
            match case {
                "coherent" => assert!(
                    matches!(outcome.unwrap(), CacheRehydrateOutcome::Miss(CacheAdapterMiss::Decode { message }) if message == "native Frontend lookup storage mismatch")
                ),
                "corrupt" => assert!(matches!(
                    outcome.unwrap(),
                    CacheRehydrateOutcome::Miss(CacheAdapterMiss::CorruptRecord)
                )),
                "permission" => assert!(matches!(
                    outcome.unwrap(),
                    CacheRehydrateOutcome::Miss(CacheAdapterMiss::Publisher { error })
                        if matches!(error.as_ref(), PublishError::WorkUnitNotAllowed { .. })
                )),
                _ => assert!(outcome.is_err(), "{case}"),
            }
        }
        assert_eq!(selected.successful_rehydrations(), 0);
    }
}
