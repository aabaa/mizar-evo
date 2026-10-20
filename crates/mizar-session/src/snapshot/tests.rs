use super::{
    BuildSnapshot, DependencyArtifactRef, Edition, GeneratedSourceKind, ModulePath, PackageId,
    RetentionReason, SnapshotError, SnapshotInput, SnapshotRegistry, SnapshotRegistryState,
    SourceOrigin, SourceVersion, ToolchainInfo, WorkspaceRoot, sort_source_versions_canonical,
};
use crate::source::hash_text;
use crate::{
    BuildRequestId, BuildSessionId, BuildSnapshotId, DiskSourceLoader, Hash, IdError,
    InMemorySessionIdAllocator, NormalizedPath, SessionIdAllocator, SnapshotLeaseId, SourceAnchor,
    SourceId, SourceInput, SourceLoadError, SourceMapId, SourceOriginInput, SourcePathError,
    SourceRange, normalize_source_path,
};
use std::path::Path;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

#[test]
fn capture_binds_all_origins_maps_and_raw_descriptors_to_the_actual_snapshot() {
    let fixture = SourcePathFixture::new();
    fixture.write("src/seed.miz", "seed");
    fixture.write("src/disk.miz", "\u{feff}disk\r\nz\r");
    fixture.write("src/editor.miz", "disk is overridden");
    fixture.write("src/generated.miz", "");
    let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
    let request = request_id();
    let (seed, _, seed_sources) = registry
        .load_snapshot(
            request,
            fixture.capture_context(),
            vec![fixture.capture_source(
                "seed",
                SourceOriginInput::Disk {
                    path: "src/seed.miz".into(),
                },
            )],
        )
        .unwrap();
    let anchor = SourceAnchor::Range(SourceRange {
        source_id: seed_sources[0].0.source_id,
        start: 0,
        end: 4,
    });
    let raw: Arc<str> = Arc::from("\u{feff}open\r\n😀");
    let generated: Arc<str> = Arc::from("\u{feff}generated\r\n\r");
    let uri = format!("file://{}", fixture.root().join("src/editor.miz").display());
    let disk_path = fixture.root().join("src/disk.miz");
    let mut disk = fixture.capture_source(
        "disk",
        SourceOriginInput::Disk {
            path: disk_path.clone(),
        },
    );
    let mut open = fixture.capture_source(
        "editor",
        SourceOriginInput::OpenBuffer {
            uri: uri.clone(),
            expected_version: 3,
            actual_version: 3,
            text: raw.clone(),
        },
    );
    std::mem::swap(&mut disk.1.normalized_path, &mut open.1.normalized_path);
    let context = fixture.capture_context();
    let (snapshot, lease, captured) = registry
        .load_snapshot(
            request,
            context.clone(),
            vec![
                fixture.capture_source(
                    "generated",
                    SourceOriginInput::Generated {
                        generator: GeneratedSourceKind::new("macro"),
                        text: generated.clone(),
                        anchor: Some(anchor.clone()),
                    },
                ),
                disk,
                open,
            ],
        )
        .unwrap();
    assert_eq!(
        captured
            .iter()
            .map(|(source, _)| source.module_path.as_str())
            .collect::<Vec<_>>(),
        ["generated", "disk", "editor"]
    );
    assert_eq!(
        registry.allocator.source_targets.lock().unwrap().as_slice(),
        [seed.id, snapshot.id, snapshot.id, snapshot.id]
    );
    assert_eq!(lease.snapshot, snapshot.id);
    assert_eq!(lease.reason, RetentionReason::ActiveBuild);
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
        1
    );
    assert_eq!(registry.get(snapshot.id), Some(snapshot.clone()));
    assert!(!registry.is_current_for_request(seed.id, request));
    assert!(registry.is_current_for_request(snapshot.id, request));
    for (source, input) in &captured {
        assert_eq!(source.normalized_path, input.normalized_path);
        assert_eq!(
            source.normalized_path.as_str(),
            format!("src/{}.miz", source.module_path.as_str())
        );
        assert_eq!(source.line_map.source_id(), source.source_id);
        assert_eq!(source.line_map.text_hash(), source.source_hash);
        assert_eq!(source.source_hash, hash_text(&source.text));
        if let Some(map) = &source.loading_map {
            assert_eq!(map.source_id, source.source_id);
            assert_eq!(map.loaded_text_hash, source.source_hash);
            assert_eq!(
                map.original_offset_for_loaded(source.source_id, 0).unwrap(),
                3
            );
        }
    }
    assert_eq!(captured[0].0.text.as_ref(), "\u{feff}generated\r\n\r");
    assert!(Arc::ptr_eq(&captured[0].0.text, &generated));
    assert_eq!(captured[0].0.generated_anchor, Some(anchor.clone()));
    assert_eq!(
        captured[0].0.origin,
        SourceOrigin::Generated {
            generator: GeneratedSourceKind::new("macro")
        }
    );
    assert_eq!(captured[0].0.loading_map, None);
    assert!(
        matches!(&captured[0].1.origin, SourceOriginInput::Generated { generator, text, anchor: Some(actual) }
        if generator.as_str() == "macro" && Arc::ptr_eq(text, &generated) && *actual == anchor)
    );
    assert_eq!(captured[1].0.text.as_ref(), "disk\nz\r");
    assert_eq!(captured[1].0.origin, SourceOrigin::Disk);
    assert_eq!(
        captured[1].1.origin,
        SourceOriginInput::Disk { path: disk_path }
    );
    assert_eq!(
        captured[1].0.loading_map.as_ref().unwrap().origin,
        crate::LoadingOrigin::DiskBytes {
            normalized_path: captured[1].0.normalized_path.clone()
        }
    );
    assert_eq!(captured[2].0.text.as_ref(), "open\n😀");
    assert_eq!(
        captured[2].0.origin,
        SourceOrigin::OpenBuffer { version: 3 }
    );
    assert_eq!(
        captured[2].0.loading_map.as_ref().unwrap().origin,
        crate::LoadingOrigin::OpenBufferText {
            uri: uri.clone(),
            version: 3
        }
    );
    assert!(
        matches!(&captured[2].1.origin, SourceOriginInput::OpenBuffer { uri: actual, expected_version: 3, actual_version: 3, text }
        if *actual == uri && Arc::ptr_eq(text, &raw))
    );
    assert_eq!(
        captured[2]
            .0
            .loading_map
            .as_ref()
            .unwrap()
            .original_offset_for_loaded(captured[2].0.source_id, 5)
            .unwrap(),
        9
    );
    let (equivalent, _) = registry
        .create_snapshot(
            request,
            SnapshotInput {
                workspace_root: context.0,
                source_versions: captured
                    .iter()
                    .map(|(source, _)| SourceVersion {
                        source_id: source.source_id,
                        package_id: source.package_id.clone(),
                        module_path: source.module_path.clone(),
                        normalized_path: source.normalized_path.clone(),
                        source_hash: source.source_hash,
                        edition: source.edition.clone(),
                        origin: source.origin.clone(),
                    })
                    .collect(),
                dependency_artifacts: context.1,
                lockfile_hash: context.2,
                toolchain: context.3,
                verifier_config_hash: context.4,
            },
        )
        .unwrap();
    assert_eq!(equivalent, snapshot);
}

#[test]
fn capture_preserves_pair_order_and_uses_every_canonical_context_field_and_edition() {
    let fixture = SourcePathFixture::new();
    fixture.write("src/alpha.miz", "alpha\r\n");
    fixture.write("src/beta.miz", "beta");
    let sources = vec![
        fixture.capture_source(
            "beta",
            SourceOriginInput::Disk {
                path: "src/beta.miz".into(),
            },
        ),
        fixture.capture_source(
            "alpha",
            SourceOriginInput::Disk {
                path: "src/alpha.miz".into(),
            },
        ),
    ];
    let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
    let context = fixture.capture_context();
    let (original, _, pairs) = registry
        .load_snapshot(request_id(), context.clone(), sources.clone())
        .unwrap();
    assert_eq!(pairs[0].0.module_path.as_str(), "beta");
    let mut reversed = sources.clone();
    reversed.reverse();
    let (same, _, pairs) = registry
        .load_snapshot(request_id(), context.clone(), reversed)
        .unwrap();
    assert_eq!(same.id, original.id);
    assert_eq!(pairs[0].0.module_path.as_str(), "alpha");
    for field in 0..7 {
        let mut changed = context.clone();
        let mut inputs = sources.clone();
        match field {
            0 => {
                changed.0 =
                    WorkspaceRoot::new(fixture.root().parent().unwrap().display().to_string())
            }
            1 => changed.1[0].content_hash = hash_text("dependency-v2"),
            2 => changed.1[0].artifact = "renamed-dependency.vo".into(),
            3 => changed.2 = hash_text("lockfile-v2"),
            4 => changed.3 = ToolchainInfo::new("toolchain-v2"),
            5 => changed.4 = hash_text("verifier-v2"),
            _ => inputs[0].1.edition = Edition::new("2027"),
        }
        let (snapshot, _, _) = registry
            .load_snapshot(request_id(), changed, inputs)
            .unwrap();
        assert_ne!(snapshot.id, original.id, "canonical field {field}");
        let targets = registry.allocator.source_targets.lock().unwrap();
        assert_eq!(&targets[targets.len() - 2..], [snapshot.id, snapshot.id]);
    }
}

#[test]
fn capture_errors_preflight_the_whole_batch_without_allocating_or_mutating_registry() {
    let fixture = SourcePathFixture::new();
    fixture.write("src/good.miz", "good");
    fixture.write("src/bad.miz", "bad");
    std::fs::write(fixture.root().join("src/bad.miz"), [0xff]).unwrap();
    let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
    let good = fixture.capture_source(
        "good",
        SourceOriginInput::Disk {
            path: "src/good.miz".into(),
        },
    );
    let context = fixture.capture_context();
    let request = request_id();
    let (previous, lease, _) = registry
        .load_snapshot(request, context.clone(), vec![good.clone()])
        .unwrap();
    let before = {
        let state = registry.state.lock().unwrap();
        (
            state.snapshots.clone(),
            state.current_by_request.clone(),
            state.leases.clone(),
            state.lease_counts.clone(),
        )
    };
    let targets = registry.allocator.source_targets.lock().unwrap().clone();
    let leases = registry.allocator.lease_allocations.load(Ordering::Relaxed);
    let uri = format!("file://{}", fixture.root().join("src/bad.miz").display());
    let mut cases = Vec::new();
    for (origin, expected) in [
        (
            SourceOriginInput::Disk {
                path: "src/bad.miz".into(),
            },
            SourceLoadError::InvalidUtf8 {
                path: Some(fixture.root().join("src/bad.miz")),
            },
        ),
        (
            SourceOriginInput::OpenBuffer {
                uri: uri.clone(),
                expected_version: 2,
                actual_version: 1,
                text: Arc::from("buffer"),
            },
            SourceLoadError::StaleLspDocumentVersion {
                expected: 2,
                actual: 1,
            },
        ),
        (
            SourceOriginInput::OpenBuffer {
                uri: uri.clone(),
                expected_version: -1,
                actual_version: -1,
                text: Arc::from("buffer"),
            },
            SourceLoadError::StaleLspDocumentVersion {
                expected: 0,
                actual: -1,
            },
        ),
        (
            SourceOriginInput::OpenBuffer {
                uri: "untitled:bad".into(),
                expected_version: 0,
                actual_version: 0,
                text: Arc::from("buffer"),
            },
            SourceLoadError::UnmappedOpenBufferUri {
                uri: "untitled:bad".into(),
            },
        ),
        (
            SourceOriginInput::Generated {
                generator: GeneratedSourceKind::new(" "),
                text: Arc::from("generated"),
                anchor: None,
            },
            SourceLoadError::GeneratedSourceWithoutMetadata {
                module_path: ModulePath::new("bad"),
            },
        ),
    ] {
        cases.push((
            context.clone(),
            vec![good.clone(), fixture.capture_source("bad", origin)],
            SnapshotError::SourceLoading { error: expected },
        ));
    }
    for field in 0..8 {
        let mut context = context.clone();
        let mut input = good.clone();
        let expected = match field {
            0 => {
                context.0 = WorkspaceRoot::new(" ");
                SnapshotError::InvalidWorkspaceRoot {
                    workspace_root: context.0.clone(),
                }
            }
            1 => {
                context.1[0].artifact = " ".into();
                SnapshotError::MissingDependencyArtifact {
                    artifact: " ".into(),
                }
            }
            2 => {
                context.1[0].content_hash = zero_hash();
                SnapshotError::MissingDependencyArtifact {
                    artifact: context.1[0].artifact.clone(),
                }
            }
            3 => {
                context.2 = zero_hash();
                SnapshotError::UnsupportedLockfileMetadata {
                    metadata: "missing-lockfile-hash".into(),
                }
            }
            4 => {
                context.3 = ToolchainInfo::new(" ");
                SnapshotError::UnsupportedToolchainMetadata {
                    metadata: " ".into(),
                }
            }
            5 => {
                input.1.package_id = PackageId::new(" ");
                SnapshotError::InvalidPackageId {
                    package_id: input.1.package_id.clone(),
                }
            }
            6 => {
                input.1.module_path = ModulePath::new("theorem");
                SnapshotError::InvalidModulePath {
                    package_id: input.1.package_id.clone(),
                    module_path: input.1.module_path.clone(),
                }
            }
            _ => {
                input.1.edition = Edition::new(" ");
                SnapshotError::InvalidEdition {
                    package_id: input.1.package_id.clone(),
                    module_path: input.1.module_path.clone(),
                    edition: input.1.edition.clone(),
                }
            }
        };
        let sources = if field >= 5 {
            vec![good.clone(), input]
        } else {
            vec![input]
        };
        cases.push((context, sources, expected));
    }
    cases.push((
        context.clone(),
        vec![good.clone(), good.clone()],
        SnapshotError::DuplicateSourceVersionIdentity {
            package_id: good.1.package_id.clone(),
            module_path: good.1.module_path.clone(),
            normalized_path: good.1.normalized_path.clone(),
            source_hash: hash_text("good"),
        },
    ));
    let mut duplicate = good.clone();
    duplicate.1.origin = SourceOriginInput::Generated {
        generator: GeneratedSourceKind::new("macro"),
        text: Arc::from("other content"),
        anchor: None,
    };
    cases.push((
        context,
        vec![good, duplicate],
        SnapshotError::DuplicateModulePath {
            package_id: PackageId::new("mml"),
            module_path: ModulePath::new("good"),
        },
    ));
    for (context, sources, expected) in cases {
        let error = registry
            .load_snapshot(request, context, sources)
            .unwrap_err();
        assert_eq!(error, expected);
        if let SnapshotError::SourceLoading { error: native } = &error {
            assert_eq!(
                std::error::Error::source(&error).unwrap().to_string(),
                native.to_string()
            );
            assert!(error.to_string().contains(&native.to_string()));
        }
        assert_eq!(*registry.allocator.source_targets.lock().unwrap(), targets);
        assert_eq!(
            registry.allocator.lease_allocations.load(Ordering::Relaxed),
            leases
        );
        let state = registry.state.lock().unwrap();
        assert_eq!(
            (
                state.snapshots.clone(),
                state.current_by_request.clone(),
                state.leases.clone(),
                state.lease_counts.clone()
            ),
            before
        );
    }
    registry.release_lease(previous.id, lease.lease_id).unwrap();
}

#[test]
fn capture_acquires_later_disk_sources_before_the_first_id_and_next_capture_sees_edit() {
    let fixture = SourcePathFixture::new();
    fixture.write("src/alpha.miz", "alpha");
    fixture.write("src/beta.miz", "\u{feff}before\r\n");
    let sources = vec![
        fixture.capture_source(
            "alpha",
            SourceOriginInput::Disk {
                path: "src/alpha.miz".into(),
            },
        ),
        fixture.capture_source(
            "beta",
            SourceOriginInput::Disk {
                path: "src/beta.miz".into(),
            },
        ),
    ];
    let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
    *registry.allocator.source_mutation.lock().unwrap() =
        Some((fixture.root().join("src/beta.miz"), "after\r\n".into()));
    let request = request_id();
    let (before, _, captured) = registry
        .load_snapshot(request, fixture.capture_context(), sources.clone())
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(fixture.root().join("src/beta.miz")).unwrap(),
        "after\r\n"
    );
    assert_eq!(captured[1].0.text.as_ref(), "before\n");
    assert_eq!(captured[1].0.source_hash, hash_text("before\n"));
    let (after, _, captured) = registry
        .load_snapshot(request, fixture.capture_context(), sources)
        .unwrap();
    assert_eq!(captured[1].0.text.as_ref(), "after\n");
    assert_ne!(after.id, before.id);
    assert!(!registry.is_current_for_request(before.id, request));
    assert!(registry.is_current_for_request(after.id, request));
    assert_eq!(
        registry.allocator.source_targets.lock().unwrap().as_slice(),
        [before.id, before.id, after.id, after.id]
    );
}

#[test]
fn capture_same_normalized_id_keeps_distinct_raw_buffer_and_version_provenance() {
    let fixture = SourcePathFixture::new();
    fixture.write("src/buffer.miz", "disk");
    let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
    let uri = format!("file://{}", fixture.root().join("src/buffer.miz").display());
    let mut captured = Vec::new();
    for (version, raw) in [
        (1, "\u{feff}alpha\r\n"),
        (2, "alpha\n"),
        (3, "\u{feff}alpha\n"),
    ] {
        let raw: Arc<str> = Arc::from(raw);
        let (snapshot, lease, sources) = registry
            .load_snapshot(
                request_id(),
                fixture.capture_context(),
                vec![fixture.capture_source(
                    "buffer",
                    SourceOriginInput::OpenBuffer {
                        uri: uri.clone(),
                        expected_version: version,
                        actual_version: version,
                        text: raw.clone(),
                    },
                )],
            )
            .unwrap();
        assert_eq!(sources[0].0.text.as_ref(), "alpha\n");
        assert_eq!(sources[0].0.origin, SourceOrigin::OpenBuffer { version });
        assert!(
            matches!(&sources[0].1.origin, SourceOriginInput::OpenBuffer { text, actual_version, .. } if Arc::ptr_eq(text, &raw) && *actual_version == version)
        );
        captured.push((snapshot, lease, sources, raw));
    }
    for entry in &captured {
        assert_eq!(entry.0.id, captured[0].0.id);
    }
    assert_ne!(captured[0].2[0].0.source_id, captured[1].2[0].0.source_id);
    assert_eq!(captured[0].3.as_ref(), "\u{feff}alpha\r\n");
    assert_eq!(captured[1].3.as_ref(), "alpha\n");
    assert!(captured[0].2[0].0.loading_map.is_some());
    assert!(captured[1].2[0].0.loading_map.is_none());
    assert!(captured[2].2[0].0.loading_map.is_some());
    assert_eq!(
        registry.lease_count_for_test(captured[0].0.id, RetentionReason::ActiveBuild),
        3
    );
}

#[test]
fn capture_partial_source_allocation_failure_preserves_prior_registry_and_leases() {
    let fixture = SourcePathFixture::new();
    fixture.write("src/alpha.miz", "alpha");
    fixture.write("src/beta.miz", "beta");
    let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
    let request = request_id();
    let alpha = fixture.capture_source(
        "alpha",
        SourceOriginInput::Disk {
            path: "src/alpha.miz".into(),
        },
    );
    let (previous, lease, _) = registry
        .load_snapshot(request, fixture.capture_context(), vec![alpha.clone()])
        .unwrap();
    let before = {
        let state = registry.state.lock().unwrap();
        (
            state.snapshots.clone(),
            state.current_by_request.clone(),
            state.leases.clone(),
            state.lease_counts.clone(),
        )
    };
    registry
        .allocator
        .source_allocations_remaining
        .store(1, Ordering::Relaxed);
    let error = registry
        .load_snapshot(
            request,
            fixture.capture_context(),
            vec![
                alpha,
                fixture.capture_source(
                    "beta",
                    SourceOriginInput::Disk {
                        path: "src/beta.miz".into(),
                    },
                ),
            ],
        )
        .unwrap_err();
    assert_eq!(
        error,
        SnapshotError::SourceLoading {
            error: SourceLoadError::SourceIdAllocation {
                error: IdError::AllocatorOverflow
            }
        }
    );
    assert_eq!(
        registry.allocator.lease_allocations.load(Ordering::Relaxed),
        1
    );
    let state = registry.state.lock().unwrap();
    assert_eq!(
        (
            state.snapshots.clone(),
            state.current_by_request.clone(),
            state.leases.clone(),
            state.lease_counts.clone()
        ),
        before
    );
    drop(state);
    registry.release_lease(previous.id, lease.lease_id).unwrap();
}

#[test]
fn capture_lease_allocation_failure_preserves_prior_registered_snapshot_and_live_lease() {
    let fixture = SourcePathFixture::new();
    fixture.write("src/alpha.miz", "alpha");
    let registry = SnapshotRegistry::with_allocator(LeaseAllocatorFailsAfter::new(1));
    let source = fixture.capture_source(
        "alpha",
        SourceOriginInput::Disk {
            path: "src/alpha.miz".into(),
        },
    );
    let request = request_id();
    let (previous, lease, _) = registry
        .load_snapshot(request, fixture.capture_context(), vec![source.clone()])
        .unwrap();
    let before = {
        let state = registry.state.lock().unwrap();
        (
            state.snapshots.clone(),
            state.current_by_request.clone(),
            state.leases.clone(),
            state.lease_counts.clone(),
        )
    };
    fixture.write("src/alpha.miz", "edited");
    let error = registry
        .load_snapshot(request, fixture.capture_context(), vec![source])
        .unwrap_err();
    assert_eq!(
        error,
        SnapshotError::LeaseIdAllocation {
            error: IdError::AllocatorOverflow
        }
    );
    let state = registry.state.lock().unwrap();
    assert_eq!(
        (
            state.snapshots.clone(),
            state.current_by_request.clone(),
            state.leases.clone(),
            state.lease_counts.clone()
        ),
        before
    );
    drop(state);
    registry.release_lease(previous.id, lease.lease_id).unwrap();
}

#[test]
fn identical_canonical_snapshot_inputs_produce_identical_ids() {
    let input = snapshot_input();
    let same = snapshot_input();

    assert_eq!(
        input.build_snapshot_id_unchecked(),
        same.build_snapshot_id_unchecked()
    );
    assert_eq!(
        BuildSnapshot::from_input_unchecked(input).id,
        same.build_snapshot_id_unchecked()
    );
}

#[test]
fn source_summary_changes_change_snapshot_id() {
    let input = snapshot_input();
    let mut changed_package_id = snapshot_input();
    changed_package_id.source_versions[0].package_id = PackageId::new("archive");
    let mut changed_module_path = snapshot_input();
    changed_module_path.source_versions[0].module_path = ModulePath::new("different.module");
    let mut changed_normalized_path = snapshot_input();
    changed_normalized_path.source_versions[0].normalized_path =
        normalized_path("src/different.miz");
    let mut changed_source_hash = snapshot_input();
    changed_source_hash.source_versions[0].source_hash = hash(44);
    let mut changed_edition = snapshot_input();
    changed_edition.source_versions[0].edition = Edition::new("2027");

    assert_ne!(
        input.build_snapshot_id_unchecked(),
        changed_package_id.build_snapshot_id_unchecked()
    );
    assert_ne!(
        input.build_snapshot_id_unchecked(),
        changed_module_path.build_snapshot_id_unchecked()
    );
    assert_ne!(
        input.build_snapshot_id_unchecked(),
        changed_normalized_path.build_snapshot_id_unchecked()
    );
    assert_ne!(
        input.build_snapshot_id_unchecked(),
        changed_source_hash.build_snapshot_id_unchecked()
    );
    assert_ne!(
        input.build_snapshot_id_unchecked(),
        changed_edition.build_snapshot_id_unchecked()
    );
}

#[test]
fn dependency_artifact_changes_change_snapshot_id() {
    let input = snapshot_input();
    let mut changed_dependency_hash = snapshot_input();
    changed_dependency_hash.dependency_artifacts[0].content_hash = hash(55);
    let mut changed_dependency_identity = snapshot_input();
    changed_dependency_identity.dependency_artifacts[0].artifact = "kernel/different.vo".to_owned();

    assert_ne!(
        input.build_snapshot_id_unchecked(),
        changed_dependency_hash.build_snapshot_id_unchecked()
    );
    assert_ne!(
        input.build_snapshot_id_unchecked(),
        changed_dependency_identity.build_snapshot_id_unchecked()
    );
}

#[test]
fn lockfile_toolchain_and_verifier_config_changes_change_snapshot_id() {
    let input = snapshot_input();
    let mut changed_lockfile = snapshot_input();
    changed_lockfile.lockfile_hash = hash(66);
    let mut changed_toolchain = snapshot_input();
    changed_toolchain.toolchain = ToolchainInfo::new("mizar-2026.2");
    let mut changed_verifier_config = snapshot_input();
    changed_verifier_config.verifier_config_hash = hash(77);

    assert_ne!(
        input.build_snapshot_id_unchecked(),
        changed_lockfile.build_snapshot_id_unchecked()
    );
    assert_ne!(
        input.build_snapshot_id_unchecked(),
        changed_toolchain.build_snapshot_id_unchecked()
    );
    assert_ne!(
        input.build_snapshot_id_unchecked(),
        changed_verifier_config.build_snapshot_id_unchecked()
    );
}

#[test]
fn workspace_root_change_changes_snapshot_id() {
    let input = snapshot_input();
    let mut changed_workspace = snapshot_input();
    changed_workspace.workspace_root = WorkspaceRoot::new("other-workspace");

    assert_ne!(
        input.build_snapshot_id_unchecked(),
        changed_workspace.build_snapshot_id_unchecked()
    );
}

#[test]
fn source_and_dependency_insertion_order_do_not_change_snapshot_id() {
    let input = snapshot_input();
    let mut reordered = snapshot_input();
    reordered.source_versions.reverse();
    reordered.dependency_artifacts.reverse();

    assert_eq!(
        input.build_snapshot_id_unchecked(),
        reordered.build_snapshot_id_unchecked()
    );

    let snapshot = reordered.build_snapshot_unchecked();
    assert_eq!(
        canonical_summary(&snapshot.source_versions),
        vec![
            ("mml", "alpha", "src/alpha.miz", 1),
            ("mml", "beta", "src/beta.miz", 2),
        ]
    );
    assert_eq!(
        snapshot
            .dependency_artifacts
            .iter()
            .map(|artifact| artifact.artifact.as_str())
            .collect::<Vec<_>>(),
        vec!["kernel/base.vo", "kernel/order.vo"]
    );
}

#[test]
fn source_identity_tie_breakers_do_not_make_hashing_insertion_order_dependent() {
    let snapshot_id = snapshot_id(13);
    let ids = InMemorySessionIdAllocator::new();
    let alpha_path = source_version(
        ids.next_source_id(snapshot_id).unwrap(),
        "mml",
        "same.module",
        "src/alpha.miz",
        hash(8),
        SourceOrigin::Disk,
    );
    let beta_path = source_version(
        ids.next_source_id(snapshot_id).unwrap(),
        "mml",
        "same.module",
        "src/beta.miz",
        hash(1),
        SourceOrigin::Disk,
    );
    let path_order = snapshot_input_with_sources(vec![beta_path.clone(), alpha_path.clone()]);
    let reverse_path_order = snapshot_input_with_sources(vec![alpha_path, beta_path]);

    assert_eq!(
        path_order.build_snapshot_id_unchecked(),
        reverse_path_order.build_snapshot_id_unchecked()
    );
    assert_eq!(
        canonical_summary(&path_order.build_snapshot_unchecked().source_versions),
        vec![
            ("mml", "same.module", "src/alpha.miz", 8),
            ("mml", "same.module", "src/beta.miz", 1),
        ]
    );

    let lower_hash = source_version(
        ids.next_source_id(snapshot_id).unwrap(),
        "mml",
        "same.module",
        "src/same.miz",
        hash(2),
        SourceOrigin::Disk,
    );
    let higher_hash = source_version(
        ids.next_source_id(snapshot_id).unwrap(),
        "mml",
        "same.module",
        "src/same.miz",
        hash(9),
        SourceOrigin::Disk,
    );
    let hash_order = snapshot_input_with_sources(vec![higher_hash.clone(), lower_hash.clone()]);
    let reverse_hash_order = snapshot_input_with_sources(vec![lower_hash, higher_hash]);

    assert_eq!(
        hash_order.build_snapshot_id_unchecked(),
        reverse_hash_order.build_snapshot_id_unchecked()
    );
    assert_eq!(
        canonical_summary(&hash_order.build_snapshot_unchecked().source_versions),
        vec![
            ("mml", "same.module", "src/same.miz", 2),
            ("mml", "same.module", "src/same.miz", 9),
        ]
    );
}

#[test]
fn dependency_content_hash_tie_breaker_does_not_make_hashing_insertion_order_dependent() {
    let mut input = snapshot_input();
    input.dependency_artifacts = vec![
        DependencyArtifactRef::new("kernel/same.vo", hash(9)),
        DependencyArtifactRef::new("kernel/same.vo", hash(2)),
    ];
    let mut reverse_order = snapshot_input();
    reverse_order.dependency_artifacts = input
        .dependency_artifacts
        .iter()
        .cloned()
        .rev()
        .collect::<Vec<_>>();

    assert_eq!(
        input.build_snapshot_id_unchecked(),
        reverse_order.build_snapshot_id_unchecked()
    );
    assert_eq!(
        dependency_summary(&input.build_snapshot_unchecked().dependency_artifacts),
        vec![("kernel/same.vo", 2), ("kernel/same.vo", 9)]
    );
}

#[test]
fn session_local_source_ids_and_origins_are_absent_from_snapshot_hash() {
    let snapshot_id = snapshot_id(10);
    let ids = InMemorySessionIdAllocator::new();
    let first_source = ids.next_source_id(snapshot_id).unwrap();
    let second_source = ids.next_source_id(snapshot_id).unwrap();
    let mut first = snapshot_input_with_sources(vec![source_version(
        first_source,
        "mml",
        "alpha",
        "src/alpha.miz",
        hash(1),
        SourceOrigin::Disk,
    )]);
    let second = snapshot_input_with_sources(vec![source_version(
        second_source,
        "mml",
        "alpha",
        "src/alpha.miz",
        hash(1),
        SourceOrigin::OpenBuffer { version: 99 },
    )]);
    first.workspace_root = WorkspaceRoot::new("workspace");

    assert_ne!(
        first.source_versions[0].source_id,
        second.source_versions[0].source_id
    );
    assert_ne!(
        first.source_versions[0].origin,
        second.source_versions[0].origin
    );
    assert_eq!(
        first.build_snapshot_id_unchecked(),
        second.build_snapshot_id_unchecked()
    );
}

#[test]
fn equal_source_canonical_keys_do_not_make_hashing_insertion_order_dependent() {
    let snapshot_id = snapshot_id(11);
    let ids = InMemorySessionIdAllocator::new();
    let old_edition = source_version_with_edition(
        ids.next_source_id(snapshot_id).unwrap(),
        "mml",
        "same.module",
        "src/same.miz",
        hash(9),
        Edition::new("2025"),
        SourceOrigin::Disk,
    );
    let new_edition = source_version_with_edition(
        ids.next_source_id(snapshot_id).unwrap(),
        "mml",
        "same.module",
        "src/same.miz",
        hash(9),
        Edition::new("2026"),
        SourceOrigin::Generated {
            generator: GeneratedSourceKind::new("different-origin"),
        },
    );

    assert_eq!(
        old_edition.canonical_sort_key(),
        new_edition.canonical_sort_key()
    );

    let insertion_order =
        snapshot_input_with_sources(vec![old_edition.clone(), new_edition.clone()]);
    let reverse_order = snapshot_input_with_sources(vec![new_edition, old_edition]);

    assert_eq!(
        insertion_order.build_snapshot_id_unchecked(),
        reverse_order.build_snapshot_id_unchecked()
    );
}

#[test]
fn equal_source_identity_summaries_do_not_make_hashing_insertion_order_dependent() {
    let snapshot_id = snapshot_id(12);
    let ids = InMemorySessionIdAllocator::new();
    let disk = source_version(
        ids.next_source_id(snapshot_id).unwrap(),
        "mml",
        "same.module",
        "src/same.miz",
        hash(9),
        SourceOrigin::Disk,
    );
    let open_buffer = source_version(
        ids.next_source_id(snapshot_id).unwrap(),
        "mml",
        "same.module",
        "src/same.miz",
        hash(9),
        SourceOrigin::OpenBuffer { version: 99 },
    );

    assert_eq!(disk.canonical_sort_key(), open_buffer.canonical_sort_key());
    assert_eq!(disk.edition, open_buffer.edition);
    assert_ne!(disk.source_id, open_buffer.source_id);
    assert_ne!(disk.origin, open_buffer.origin);

    let insertion_order = snapshot_input_with_sources(vec![disk.clone(), open_buffer.clone()]);
    let reverse_order = snapshot_input_with_sources(vec![open_buffer, disk]);

    assert_eq!(
        insertion_order.build_snapshot_id_unchecked(),
        reverse_order.build_snapshot_id_unchecked()
    );
}

#[test]
fn source_versions_sort_deterministically_by_canonical_key() {
    let snapshot_id = snapshot_id(1);
    let ids = InMemorySessionIdAllocator::new();
    let first_source = ids.next_source_id(snapshot_id).unwrap();
    let second_source = ids.next_source_id(snapshot_id).unwrap();
    let third_source = ids.next_source_id(snapshot_id).unwrap();
    let fourth_source = ids.next_source_id(snapshot_id).unwrap();

    let versions = vec![
        source_version(
            third_source,
            "pkg-b",
            "alpha",
            "src/alpha.miz",
            hash(1),
            SourceOrigin::Disk,
        ),
        source_version(
            first_source,
            "pkg-a",
            "beta",
            "src/beta.miz",
            hash(3),
            SourceOrigin::OpenBuffer { version: 7 },
        ),
        source_version(
            fourth_source,
            "pkg-a",
            "alpha",
            "src/alpha.miz",
            hash(9),
            SourceOrigin::Generated {
                generator: GeneratedSourceKind::new("test-generator"),
            },
        ),
        source_version(
            second_source,
            "pkg-a",
            "alpha",
            "src/alpha.miz",
            hash(2),
            SourceOrigin::Disk,
        ),
    ];
    let mut insertion_order = versions.clone();
    let mut reverse_order = versions.into_iter().rev().collect::<Vec<_>>();

    sort_source_versions_canonical(&mut insertion_order);
    sort_source_versions_canonical(&mut reverse_order);

    assert_eq!(
        canonical_summary(&insertion_order),
        vec![
            ("pkg-a", "alpha", "src/alpha.miz", 2),
            ("pkg-a", "alpha", "src/alpha.miz", 9),
            ("pkg-a", "beta", "src/beta.miz", 3),
            ("pkg-b", "alpha", "src/alpha.miz", 1),
        ]
    );
    assert_eq!(
        canonical_summary(&insertion_order),
        canonical_summary(&reverse_order)
    );
}

#[test]
fn source_version_canonical_key_uses_all_specified_fields_in_order() {
    let snapshot_id = snapshot_id(2);
    let ids = InMemorySessionIdAllocator::new();
    let base_source = ids.next_source_id(snapshot_id).unwrap();
    let package_tie_breaker_source = ids.next_source_id(snapshot_id).unwrap();
    let module_tie_breaker_source = ids.next_source_id(snapshot_id).unwrap();
    let path_tie_breaker_source = ids.next_source_id(snapshot_id).unwrap();
    let hash_tie_breaker_source = ids.next_source_id(snapshot_id).unwrap();

    let base = source_version(
        base_source,
        "pkg-a",
        "alpha",
        "src/alpha.miz",
        hash(1),
        SourceOrigin::Disk,
    );
    let package_tie_breaker = source_version(
        package_tie_breaker_source,
        "pkg-b",
        "alpha",
        "src/alpha.miz",
        hash(0),
        SourceOrigin::Disk,
    );
    let module_tie_breaker = source_version(
        module_tie_breaker_source,
        "pkg-a",
        "beta",
        "src/alpha.miz",
        hash(0),
        SourceOrigin::Disk,
    );
    let path_tie_breaker = source_version(
        path_tie_breaker_source,
        "pkg-a",
        "alpha",
        "src/beta.miz",
        hash(0),
        SourceOrigin::Disk,
    );
    let hash_tie_breaker = source_version(
        hash_tie_breaker_source,
        "pkg-a",
        "alpha",
        "src/alpha.miz",
        hash(2),
        SourceOrigin::Disk,
    );

    assert!(base.canonical_sort_key() < package_tie_breaker.canonical_sort_key());
    assert!(base.canonical_sort_key() < module_tie_breaker.canonical_sort_key());
    assert!(base.canonical_sort_key() < path_tie_breaker.canonical_sort_key());
    assert!(base.canonical_sort_key() < hash_tie_breaker.canonical_sort_key());
}

#[test]
fn source_versions_sort_by_normalized_path_before_source_hash() {
    let snapshot_id = snapshot_id(3);
    let ids = InMemorySessionIdAllocator::new();
    let mut versions = vec![
        source_version(
            ids.next_source_id(snapshot_id).unwrap(),
            "pkg",
            "same.module",
            "src/beta.miz",
            hash(0),
            SourceOrigin::Disk,
        ),
        source_version(
            ids.next_source_id(snapshot_id).unwrap(),
            "pkg",
            "same.module",
            "src/alpha.miz",
            hash(255),
            SourceOrigin::Disk,
        ),
    ];

    sort_source_versions_canonical(&mut versions);

    assert_eq!(
        canonical_summary(&versions),
        vec![
            ("pkg", "same.module", "src/alpha.miz", 255),
            ("pkg", "same.module", "src/beta.miz", 0),
        ]
    );
}

#[test]
fn source_version_canonical_key_excludes_session_local_and_non_key_fields() {
    let snapshot_id = snapshot_id(4);
    let ids = InMemorySessionIdAllocator::new();
    let disk = source_version_with_edition(
        ids.next_source_id(snapshot_id).unwrap(),
        "pkg",
        "same.module",
        "src/same.miz",
        hash(7),
        Edition::new("2026"),
        SourceOrigin::Disk,
    );
    let open_buffer = source_version_with_edition(
        ids.next_source_id(snapshot_id).unwrap(),
        "pkg",
        "same.module",
        "src/same.miz",
        hash(7),
        Edition::new("future-test-edition"),
        SourceOrigin::OpenBuffer { version: 99 },
    );

    assert_ne!(disk.source_id, open_buffer.source_id);
    assert_ne!(disk.edition, open_buffer.edition);
    assert_ne!(disk.origin, open_buffer.origin);
    assert_eq!(disk.canonical_sort_key(), open_buffer.canonical_sort_key());
}

#[test]
fn source_version_records_source_id_origin_hash_and_path_identity() {
    let snapshot_id = snapshot_id(5);
    let source_id = InMemorySessionIdAllocator::new()
        .next_source_id(snapshot_id)
        .unwrap();
    let normalized_path = normalized_path("src/groups/basic.miz");
    let source_hash = hash(42);
    let version = SourceVersion {
        source_id,
        package_id: PackageId::new("mml"),
        module_path: ModulePath::new("groups.basic"),
        normalized_path: normalized_path.clone(),
        source_hash,
        edition: Edition::new("2026"),
        origin: SourceOrigin::Disk,
    };

    assert_eq!(version.source_id, source_id);
    assert_eq!(version.package_id.as_str(), "mml");
    assert_eq!(version.module_path.as_str(), "groups.basic");
    assert_eq!(version.normalized_path, normalized_path);
    assert_eq!(version.source_hash, source_hash);
    assert_eq!(version.edition.as_str(), "2026");
    assert_eq!(version.origin, SourceOrigin::Disk);
}

#[test]
fn source_origin_represents_disk_open_buffer_and_generated_sources() {
    let disk = SourceOrigin::Disk;
    let open_buffer = SourceOrigin::OpenBuffer { version: 11 };
    let generated = SourceOrigin::Generated {
        generator: GeneratedSourceKind::new("macro-expansion"),
    };

    assert_eq!(disk, SourceOrigin::Disk);
    assert_eq!(open_buffer, SourceOrigin::OpenBuffer { version: 11 });
    assert_eq!(
        generated,
        SourceOrigin::Generated {
            generator: GeneratedSourceKind::new("macro-expansion")
        }
    );
}

#[test]
fn created_snapshot_is_retrievable_and_returns_active_build_lease() {
    let registry = SnapshotRegistry::new();
    let request = request_id();
    let input = snapshot_input();
    let expected_snapshot_id = input.build_snapshot_id_unchecked();

    let (snapshot, lease) = registry.create_snapshot(request, input).unwrap();

    assert_eq!(snapshot.id, expected_snapshot_id);
    assert_eq!(registry.get(snapshot.id), Some(snapshot.clone()));
    assert_eq!(lease.snapshot, snapshot.id);
    assert_eq!(lease.reason, RetentionReason::ActiveBuild);
    assert!(registry.is_current_for_request(snapshot.id, request));
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
        1
    );
}

#[test]
fn repeated_snapshot_creation_returns_distinct_active_build_leases() {
    let registry = SnapshotRegistry::new();
    let request = request_id();

    let (first_snapshot, first_lease) =
        registry.create_snapshot(request, snapshot_input()).unwrap();
    let (second_snapshot, second_lease) =
        registry.create_snapshot(request, snapshot_input()).unwrap();

    assert_eq!(first_snapshot.id, second_snapshot.id);
    assert_eq!(first_lease.snapshot, first_snapshot.id);
    assert_eq!(second_lease.snapshot, second_snapshot.id);
    assert_eq!(first_lease.reason, RetentionReason::ActiveBuild);
    assert_eq!(second_lease.reason, RetentionReason::ActiveBuild);
    assert_ne!(first_lease.lease_id, second_lease.lease_id);
    assert_eq!(
        registry.lease_count_for_test(first_snapshot.id, RetentionReason::ActiveBuild),
        2
    );
}

#[test]
fn acquire_lease_increments_count_for_reason() {
    let registry = SnapshotRegistry::new();
    let (snapshot, _) = registry
        .create_snapshot(request_id(), snapshot_input())
        .unwrap();

    let lease = registry
        .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
        .unwrap();

    assert_eq!(lease.snapshot, snapshot.id);
    assert_eq!(lease.reason, RetentionReason::DiagnosticIndex);
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
        1
    );
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::DiagnosticIndex),
        1
    );
}

#[test]
fn acquire_lease_tracks_each_retention_reason_independently() {
    let registry = SnapshotRegistry::new();
    let (snapshot, _) = registry
        .create_snapshot(request_id(), snapshot_input())
        .unwrap();

    let leases = all_retention_reasons()
        .into_iter()
        .map(|reason| registry.acquire_lease(snapshot.id, reason).unwrap())
        .collect::<Vec<_>>();

    for reason in all_retention_reasons() {
        let expected_count = if reason == RetentionReason::ActiveBuild {
            2
        } else {
            1
        };
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, reason),
            expected_count,
            "lease count for {reason:?}"
        );
    }
    assert_eq!(
        registry.total_lease_count_for_test(snapshot.id),
        all_retention_reasons().len() + 1
    );

    for lease in leases {
        registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    }

    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
        1
    );
    for reason in all_retention_reasons()
        .into_iter()
        .filter(|reason| *reason != RetentionReason::ActiveBuild)
    {
        assert_eq!(
            registry.lease_count_for_test(snapshot.id, reason),
            0,
            "lease count after release for {reason:?}"
        );
    }
    assert_eq!(registry.total_lease_count_for_test(snapshot.id), 1);
}

#[test]
fn release_lease_decrements_count_for_reason() {
    let registry = SnapshotRegistry::new();
    let (snapshot, _) = registry
        .create_snapshot(request_id(), snapshot_input())
        .unwrap();
    let first = registry
        .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
        .unwrap();
    let second = registry
        .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
        .unwrap();

    registry.release_lease(snapshot.id, first.lease_id).unwrap();

    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::DiagnosticIndex),
        1
    );
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
        1
    );

    registry
        .release_lease(snapshot.id, second.lease_id)
        .unwrap();

    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::DiagnosticIndex),
        0
    );
}

#[test]
fn repeated_acquire_lease_returns_unique_ids_and_counts_by_reason() {
    let registry = SnapshotRegistry::new();
    let (snapshot, _) = registry
        .create_snapshot(request_id(), snapshot_input())
        .unwrap();

    let first = registry
        .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
        .unwrap();
    let second = registry
        .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
        .unwrap();
    let explanation = registry
        .acquire_lease(snapshot.id, RetentionReason::ExplanationRequest)
        .unwrap();

    assert_ne!(first.lease_id, second.lease_id);
    assert_ne!(first.lease_id, explanation.lease_id);
    assert_ne!(second.lease_id, explanation.lease_id);
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::DiagnosticIndex),
        2
    );
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ExplanationRequest),
        1
    );
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
        1
    );
    assert_eq!(registry.total_lease_count_for_test(snapshot.id), 4);
}

#[test]
fn acquire_lease_allocates_lease_id_without_holding_registry_mutex() {
    let registry = SnapshotRegistry::with_allocator(RegistryMutexProbeAllocator::new());
    registry.allocator.bind_state(&registry.state);
    let (snapshot, _) = registry
        .create_snapshot(request_id(), snapshot_input())
        .unwrap();
    registry
        .allocator
        .registry_mutex_available_checks
        .store(0, Ordering::Relaxed);

    let lease = registry
        .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
        .unwrap();

    assert_eq!(lease.snapshot, snapshot.id);
    assert_eq!(lease.reason, RetentionReason::DiagnosticIndex);
    assert_eq!(
        registry
            .allocator
            .registry_mutex_available_checks
            .load(Ordering::Relaxed),
        1
    );
}

#[test]
fn release_lease_only_decrements_the_released_reason() {
    let registry = SnapshotRegistry::new();
    let (snapshot, _) = registry
        .create_snapshot(request_id(), snapshot_input())
        .unwrap();
    let diagnostic = registry
        .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
        .unwrap();
    let explanation = registry
        .acquire_lease(snapshot.id, RetentionReason::ExplanationRequest)
        .unwrap();

    registry
        .release_lease(snapshot.id, diagnostic.lease_id)
        .unwrap();

    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::DiagnosticIndex),
        0
    );
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ExplanationRequest),
        1
    );
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
        1
    );
    assert_eq!(registry.total_lease_count_for_test(snapshot.id), 2);

    registry
        .release_lease(snapshot.id, explanation.lease_id)
        .unwrap();

    assert_eq!(registry.total_lease_count_for_test(snapshot.id), 1);
}

#[test]
fn release_active_build_lease_from_create_snapshot_is_accounted() {
    let registry = SnapshotRegistry::new();
    let (snapshot, active_build) = registry
        .create_snapshot(request_id(), snapshot_input())
        .unwrap();

    registry
        .release_lease(snapshot.id, active_build.lease_id)
        .unwrap();

    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
        0
    );
    assert_eq!(registry.get(snapshot.id), Some(snapshot));
}

#[test]
fn acquire_lease_for_unknown_snapshot_returns_unknown_snapshot_id() {
    let registry = SnapshotRegistry::new();
    let unknown = snapshot_id(201);

    let error = registry
        .acquire_lease(unknown, RetentionReason::ExplanationRequest)
        .unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::UnknownSnapshotId { snapshot_id } if snapshot_id == unknown
    ));
}

#[test]
fn acquire_lease_for_unknown_snapshot_does_not_allocate_lease_id() {
    let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
    let unknown = snapshot_id(203);

    let error = registry
        .acquire_lease(unknown, RetentionReason::PendingWrite)
        .unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::UnknownSnapshotId { snapshot_id } if snapshot_id == unknown
    ));
    assert_eq!(
        registry.allocator.lease_allocations.load(Ordering::Relaxed),
        0
    );
}

#[test]
fn release_lease_for_wrong_snapshot_returns_mismatch_without_changing_counts() {
    let registry = SnapshotRegistry::new();
    let (first_snapshot, first_lease) = registry
        .create_snapshot(request_id(), snapshot_input())
        .unwrap();
    let mut second_input = snapshot_input();
    second_input.source_versions[0].source_hash = hash(222);
    let (second_snapshot, _) = registry
        .create_snapshot(request_id(), second_input)
        .unwrap();

    let error = registry
        .release_lease(second_snapshot.id, first_lease.lease_id)
        .unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::LeaseReleaseMismatch {
            lease_id,
            expected_snapshot,
            actual_snapshot,
        } if lease_id == first_lease.lease_id
            && expected_snapshot == second_snapshot.id
            && actual_snapshot == first_snapshot.id
    ));
    assert_eq!(
        registry.lease_count_for_test(first_snapshot.id, RetentionReason::ActiveBuild),
        1
    );
    assert_eq!(
        registry.lease_count_for_test(second_snapshot.id, RetentionReason::ActiveBuild),
        1
    );

    registry
        .release_lease(first_snapshot.id, first_lease.lease_id)
        .unwrap();

    assert_eq!(
        registry.lease_count_for_test(first_snapshot.id, RetentionReason::ActiveBuild),
        0
    );
    assert_eq!(
        registry.lease_count_for_test(second_snapshot.id, RetentionReason::ActiveBuild),
        1
    );
}

#[test]
fn release_lease_for_unknown_snapshot_returns_unknown_snapshot_id() {
    let registry = SnapshotRegistry::new();
    let (snapshot, lease) = registry
        .create_snapshot(request_id(), snapshot_input())
        .unwrap();
    let unknown = snapshot_id(202);

    let error = registry.release_lease(unknown, lease.lease_id).unwrap_err();

    assert_ne!(snapshot.id, unknown);
    assert!(matches!(
        error,
        SnapshotError::UnknownSnapshotId { snapshot_id } if snapshot_id == unknown
    ));
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
        1
    );

    registry.release_lease(snapshot.id, lease.lease_id).unwrap();

    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
        0
    );
}

#[test]
fn double_release_returns_unknown_snapshot_lease_without_underflow() {
    let registry = SnapshotRegistry::new();
    let (snapshot, lease) = registry
        .create_snapshot(request_id(), snapshot_input())
        .unwrap();

    registry.release_lease(snapshot.id, lease.lease_id).unwrap();
    let error = registry
        .release_lease(snapshot.id, lease.lease_id)
        .unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::UnknownSnapshotLease { lease_id } if lease_id == lease.lease_id
    ));
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
        0
    );
}

#[test]
fn unknown_lease_id_returns_unknown_snapshot_lease() {
    let registry = SnapshotRegistry::new();
    let (snapshot, _) = registry
        .create_snapshot(request_id(), snapshot_input())
        .unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let _known_shape_but_not_this_lease = ids.next_lease_id(snapshot.id).unwrap();
    let unknown_lease = ids.next_lease_id(snapshot.id).unwrap();

    let error = registry
        .release_lease(snapshot.id, unknown_lease)
        .unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::UnknownSnapshotLease { lease_id } if lease_id == unknown_lease
    ));
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
        1
    );
}

#[test]
fn release_lease_after_final_reason_count_removes_the_snapshot_total() {
    let registry = SnapshotRegistry::new();
    let (snapshot, active_build) = registry
        .create_snapshot(request_id(), snapshot_input())
        .unwrap();

    registry
        .release_lease(snapshot.id, active_build.lease_id)
        .unwrap();

    assert_eq!(registry.total_lease_count_for_test(snapshot.id), 0);
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
        0
    );
    assert_eq!(registry.get(snapshot.id), Some(snapshot));
}

#[test]
fn acquire_lease_id_allocation_failure_does_not_change_registry_state() {
    let registry = SnapshotRegistry::with_allocator(LeaseAllocatorFailsAfter::new(1));
    let request = request_id();
    let (snapshot, _) = registry.create_snapshot(request, snapshot_input()).unwrap();
    let snapshot_before = registry.get(snapshot.id);
    let current_before = registry.is_current_for_request(snapshot.id, request);
    let active_build_count_before =
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild);
    let published_count_before =
        registry.lease_count_for_test(snapshot.id, RetentionReason::PublishedLspSnapshot);
    let total_count_before = registry.total_lease_count_for_test(snapshot.id);
    let live_lease_count_before = registry.live_lease_count_for_test(snapshot.id);

    let error = registry
        .acquire_lease(snapshot.id, RetentionReason::PublishedLspSnapshot)
        .unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::LeaseIdAllocation {
            error: IdError::AllocatorOverflow
        }
    ));
    assert_eq!(registry.get(snapshot.id), snapshot_before);
    assert_eq!(
        registry.is_current_for_request(snapshot.id, request),
        current_before
    );
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
        active_build_count_before
    );
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::PublishedLspSnapshot),
        published_count_before
    );
    assert_eq!(
        registry.total_lease_count_for_test(snapshot.id),
        total_count_before
    );
    assert_eq!(
        registry.live_lease_count_for_test(snapshot.id),
        live_lease_count_before
    );
}

#[test]
fn duplicate_acquire_lease_id_does_not_change_registry_state() {
    let duplicate_lease_id = InMemorySessionIdAllocator::new()
        .next_lease_id(snapshot_id(205))
        .unwrap();
    let registry =
        SnapshotRegistry::with_allocator(DuplicateLeaseAllocator::new(duplicate_lease_id));
    let request = request_id();
    let (snapshot, active_build) = registry.create_snapshot(request, snapshot_input()).unwrap();
    let snapshot_before = registry.get(snapshot.id);
    let current_before = registry.is_current_for_request(snapshot.id, request);
    let active_build_count_before =
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild);
    let diagnostic_count_before =
        registry.lease_count_for_test(snapshot.id, RetentionReason::DiagnosticIndex);
    let total_count_before = registry.total_lease_count_for_test(snapshot.id);
    let live_lease_count_before = registry.live_lease_count_for_test(snapshot.id);

    let error = registry
        .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
        .unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::DuplicateLeaseIdAllocation {
            lease_id,
            existing_snapshot,
            allocated_snapshot,
        } if lease_id == active_build.lease_id
            && existing_snapshot == snapshot.id
            && allocated_snapshot == snapshot.id
    ));
    assert_eq!(registry.get(snapshot.id), snapshot_before);
    assert_eq!(
        registry.is_current_for_request(snapshot.id, request),
        current_before
    );
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::ActiveBuild),
        active_build_count_before
    );
    assert_eq!(
        registry.lease_count_for_test(snapshot.id, RetentionReason::DiagnosticIndex),
        diagnostic_count_before
    );
    assert_eq!(
        registry.total_lease_count_for_test(snapshot.id),
        total_count_before
    );
    assert_eq!(
        registry.live_lease_count_for_test(snapshot.id),
        live_lease_count_before
    );
}

#[test]
fn duplicate_create_snapshot_lease_id_does_not_insert_snapshot_or_replace_current() {
    let duplicate_lease_id = InMemorySessionIdAllocator::new()
        .next_lease_id(snapshot_id(206))
        .unwrap();
    let registry =
        SnapshotRegistry::with_allocator(DuplicateLeaseAllocator::new(duplicate_lease_id));
    let request = request_id();
    let (accepted, active_build) = registry.create_snapshot(request, snapshot_input()).unwrap();
    let mut rejected_input = snapshot_input();
    rejected_input.source_versions[0].source_hash = hash(224);
    let rejected_id = rejected_input.build_snapshot_id_unchecked();
    let active_build_count_before =
        registry.lease_count_for_test(accepted.id, RetentionReason::ActiveBuild);
    let rejected_count_before = registry.total_lease_count_for_test(rejected_id);
    let live_lease_count_before = registry.live_lease_count_for_test(accepted.id);

    let error = registry
        .create_snapshot(request, rejected_input)
        .unwrap_err();

    assert_ne!(accepted.id, rejected_id);
    assert!(matches!(
        error,
        SnapshotError::DuplicateLeaseIdAllocation {
            lease_id,
            existing_snapshot,
            allocated_snapshot,
        } if lease_id == active_build.lease_id
            && existing_snapshot == accepted.id
            && allocated_snapshot == rejected_id
    ));
    assert_eq!(registry.get(accepted.id), Some(accepted.clone()));
    assert_eq!(registry.get(rejected_id), None);
    assert!(registry.is_current_for_request(accepted.id, request));
    assert!(!registry.is_current_for_request(rejected_id, request));
    assert_eq!(
        registry.lease_count_for_test(accepted.id, RetentionReason::ActiveBuild),
        active_build_count_before
    );
    assert_eq!(
        registry.total_lease_count_for_test(rejected_id),
        rejected_count_before
    );
    assert_eq!(
        registry.live_lease_count_for_test(accepted.id),
        live_lease_count_before
    );
}

#[test]
fn retrieved_snapshot_clone_cannot_mutate_registry_snapshot() {
    let registry = SnapshotRegistry::new();
    let request = request_id();
    let (snapshot, _) = registry.create_snapshot(request, snapshot_input()).unwrap();

    let mut retrieved = registry.get(snapshot.id).unwrap();
    retrieved.source_versions.clear();
    retrieved.dependency_artifacts.clear();

    assert_eq!(registry.get(snapshot.id), Some(snapshot));
}

#[test]
fn stale_snapshot_id_is_not_current_for_request() {
    let registry = SnapshotRegistry::new();
    let request = request_id();
    let (snapshot, _) = registry.create_snapshot(request, snapshot_input()).unwrap();
    let stale = snapshot_id(200);

    assert_ne!(snapshot.id, stale);
    assert!(!registry.is_current_for_request(stale, request));
}

#[test]
fn current_snapshot_is_tracked_independently_per_request_generation() {
    let registry = SnapshotRegistry::new();
    let ids = InMemorySessionIdAllocator::new();
    let first_request = ids.next_request_id().unwrap();
    let second_request = ids.next_request_id().unwrap();
    let (first_snapshot, _) = registry
        .create_snapshot(first_request, snapshot_input())
        .unwrap();
    let mut second_input = snapshot_input();
    second_input.source_versions[0].source_hash = hash(98);

    let (second_snapshot, _) = registry
        .create_snapshot(second_request, second_input)
        .unwrap();

    assert_ne!(first_request, second_request);
    assert_ne!(first_snapshot.id, second_snapshot.id);
    assert!(registry.is_current_for_request(first_snapshot.id, first_request));
    assert!(!registry.is_current_for_request(first_snapshot.id, second_request));
    assert!(registry.is_current_for_request(second_snapshot.id, second_request));
    assert!(!registry.is_current_for_request(second_snapshot.id, first_request));
}

#[test]
fn older_snapshot_is_not_current_after_new_snapshot_for_same_request() {
    let registry = SnapshotRegistry::new();
    let request = request_id();
    let (older, _) = registry.create_snapshot(request, snapshot_input()).unwrap();
    let mut newer_input = snapshot_input();
    newer_input.source_versions[0].source_hash = hash(99);

    let (newer, _) = registry.create_snapshot(request, newer_input).unwrap();

    assert_ne!(older.id, newer.id);
    assert!(!registry.is_current_for_request(older.id, request));
    assert!(registry.is_current_for_request(newer.id, request));
    assert_eq!(registry.get(older.id), Some(older));
}

#[test]
fn stale_but_registered_snapshot_can_still_acquire_lease_without_becoming_current() {
    let registry = SnapshotRegistry::new();
    let request = request_id();
    let (older, _) = registry.create_snapshot(request, snapshot_input()).unwrap();
    let mut newer_input = snapshot_input();
    newer_input.source_versions[0].source_hash = hash(100);
    let (newer, _) = registry.create_snapshot(request, newer_input).unwrap();

    let lease = registry
        .acquire_lease(older.id, RetentionReason::PhaseOutputReference)
        .unwrap();

    assert_eq!(lease.snapshot, older.id);
    assert!(!registry.is_current_for_request(older.id, request));
    assert!(registry.is_current_for_request(newer.id, request));
    assert_eq!(
        registry.lease_count_for_test(older.id, RetentionReason::PhaseOutputReference),
        1
    );
    assert_eq!(
        registry.lease_count_for_test(newer.id, RetentionReason::PhaseOutputReference),
        0
    );

    registry.release_lease(older.id, lease.lease_id).unwrap();

    assert!(!registry.is_current_for_request(older.id, request));
    assert!(registry.is_current_for_request(newer.id, request));
    assert_eq!(
        registry.lease_count_for_test(older.id, RetentionReason::PhaseOutputReference),
        0
    );
}

#[test]
fn rejected_snapshot_does_not_allocate_lease_insert_snapshot_or_replace_current() {
    let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
    let request = request_id();
    let (accepted, _) = registry.create_snapshot(request, snapshot_input()).unwrap();
    let snapshot_id = snapshot_id(16);
    let ids = InMemorySessionIdAllocator::new();
    let rejected_input = snapshot_input_with_sources(vec![
        source_version(
            ids.next_source_id(snapshot_id).unwrap(),
            "mml",
            "same.module",
            "src/alpha.miz",
            hash(1),
            SourceOrigin::Disk,
        ),
        source_version(
            ids.next_source_id(snapshot_id).unwrap(),
            "mml",
            "same.module",
            "src/beta.miz",
            hash(2),
            SourceOrigin::Disk,
        ),
    ]);
    let rejected_id = rejected_input.build_snapshot_id_unchecked();
    let lease_allocations_before = registry.allocator.lease_allocations.load(Ordering::Relaxed);

    let error = registry
        .create_snapshot(request, rejected_input)
        .unwrap_err();

    assert!(matches!(error, SnapshotError::DuplicateModulePath { .. }));
    assert_eq!(
        registry.allocator.lease_allocations.load(Ordering::Relaxed),
        lease_allocations_before
    );
    assert_eq!(registry.get(rejected_id), None);
    assert!(registry.is_current_for_request(accepted.id, request));
    assert!(!registry.is_current_for_request(rejected_id, request));
}

#[test]
fn unchecked_identity_helpers_do_not_create_registry_snapshots() {
    let registry = SnapshotRegistry::new();
    let request = request_id();
    let mut input = snapshot_input();
    input.toolchain = ToolchainInfo::new(" ");

    let unchecked_id = input.build_snapshot_id_unchecked();
    let unchecked_snapshot = input.clone().build_snapshot_unchecked();
    let error = registry.create_snapshot(request, input).unwrap_err();

    assert_eq!(unchecked_snapshot.id, unchecked_id);
    assert!(matches!(
        error,
        SnapshotError::UnsupportedToolchainMetadata { metadata } if metadata == " "
    ));
    assert_eq!(registry.get(unchecked_id), None);
    assert!(!registry.is_current_for_request(unchecked_id, request));
    assert_eq!(registry.total_lease_count_for_test(unchecked_id), 0);
}

#[test]
fn public_struct_literal_snapshot_is_only_a_detached_record() {
    let registry = SnapshotRegistry::new();
    let request = request_id();
    let snapshot = BuildSnapshot {
        id: snapshot_id(204),
        workspace_root: WorkspaceRoot::new("detached"),
        source_versions: Vec::new(),
        dependency_artifacts: Vec::new(),
        lockfile_hash: zero_hash(),
        toolchain: ToolchainInfo::new(" "),
        verifier_config_hash: zero_hash(),
    };

    let error = registry
        .acquire_lease(snapshot.id, RetentionReason::DiagnosticIndex)
        .unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::UnknownSnapshotId { snapshot_id } if snapshot_id == snapshot.id
    ));
    assert_eq!(registry.get(snapshot.id), None);
    assert!(!registry.is_current_for_request(snapshot.id, request));
    assert_eq!(registry.total_lease_count_for_test(snapshot.id), 0);
}

#[test]
fn lease_id_allocation_failure_does_not_insert_snapshot_or_mark_current() {
    let registry = SnapshotRegistry::with_allocator(LeaseFailingAllocator::new());
    let request = request_id();
    let input = snapshot_input();
    let snapshot_id = input.build_snapshot_id_unchecked();

    let error = registry.create_snapshot(request, input).unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::LeaseIdAllocation {
            error: IdError::AllocatorOverflow
        }
    ));
    assert_eq!(registry.get(snapshot_id), None);
    assert!(!registry.is_current_for_request(snapshot_id, request));
    assert_eq!(registry.total_lease_count_for_test(snapshot_id), 0);
}

#[test]
fn duplicate_module_path_is_rejected_before_snapshot_hashing() {
    let registry = SnapshotRegistry::new();
    let snapshot_id = snapshot_id(14);
    let ids = InMemorySessionIdAllocator::new();
    let duplicate_module = snapshot_input_with_sources(vec![
        source_version(
            ids.next_source_id(snapshot_id).unwrap(),
            "mml",
            "same.module",
            "src/alpha.miz",
            hash(1),
            SourceOrigin::Disk,
        ),
        source_version(
            ids.next_source_id(snapshot_id).unwrap(),
            "mml",
            "same.module",
            "src/beta.miz",
            hash(2),
            SourceOrigin::Disk,
        ),
    ]);

    let error = registry
        .create_snapshot(request_id(), duplicate_module)
        .unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::DuplicateModulePath {
            package_id,
            module_path,
        } if package_id.as_str() == "mml" && module_path.as_str() == "same.module"
    ));
}

#[test]
fn duplicate_source_version_identity_is_rejected_before_snapshot_hashing() {
    let registry = SnapshotRegistry::new();
    let snapshot_id = snapshot_id(15);
    let ids = InMemorySessionIdAllocator::new();
    let duplicate_identity = snapshot_input_with_sources(vec![
        source_version(
            ids.next_source_id(snapshot_id).unwrap(),
            "mml",
            "same.module",
            "src/same.miz",
            hash(7),
            SourceOrigin::Disk,
        ),
        source_version(
            ids.next_source_id(snapshot_id).unwrap(),
            "mml",
            "same.module",
            "src/same.miz",
            hash(7),
            SourceOrigin::OpenBuffer { version: 12 },
        ),
    ]);

    let error = registry
        .create_snapshot(request_id(), duplicate_identity)
        .unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::DuplicateSourceVersionIdentity {
            package_id,
            module_path,
            normalized_path,
            source_hash,
        } if package_id.as_str() == "mml"
            && module_path.as_str() == "same.module"
            && normalized_path.as_str() == "src/same.miz"
            && source_hash == hash(7)
    ));
}

#[test]
fn missing_dependency_artifact_is_rejected_by_creation_validation() {
    let registry = SnapshotRegistry::new();
    let mut input = snapshot_input();
    input.dependency_artifacts[0].artifact = "   ".to_owned();

    let error = registry.create_snapshot(request_id(), input).unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::MissingDependencyArtifact { artifact } if artifact == "   "
    ));
}

#[test]
fn dependency_artifact_with_missing_content_hash_is_rejected_by_creation_validation() {
    let registry = SnapshotRegistry::new();
    let mut input = snapshot_input();
    input.dependency_artifacts[0].content_hash = zero_hash();

    let error = registry.create_snapshot(request_id(), input).unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::MissingDependencyArtifact { artifact }
            if artifact == "kernel/order.vo"
    ));
}

#[test]
fn unsupported_lockfile_metadata_is_rejected_by_creation_validation() {
    let registry = SnapshotRegistry::new();
    let mut input = snapshot_input();
    input.lockfile_hash = zero_hash();

    let error = registry.create_snapshot(request_id(), input).unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::UnsupportedLockfileMetadata { metadata }
            if metadata == "missing-lockfile-hash"
    ));
}

#[test]
fn unsupported_toolchain_metadata_is_rejected_by_creation_validation() {
    let registry = SnapshotRegistry::new();
    let mut input = snapshot_input();
    input.toolchain = ToolchainInfo::new(" ");

    let error = registry.create_snapshot(request_id(), input).unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::UnsupportedToolchainMetadata { metadata } if metadata == " "
    ));
}

#[test]
fn invalid_workspace_root_is_rejected_before_snapshot_registration() {
    let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
    let request = request_id();
    let mut input = snapshot_input();
    input.workspace_root = WorkspaceRoot::new(" ");

    let error = registry.create_snapshot(request, input).unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::InvalidWorkspaceRoot { workspace_root }
            if workspace_root.as_str() == " "
    ));
    assert_eq!(
        registry.allocator.lease_allocations.load(Ordering::Relaxed),
        0
    );
}

#[test]
fn blank_package_id_is_rejected_before_snapshot_registration() {
    for invalid_package_id in [" ", "mml core"] {
        let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
        let request = request_id();
        let mut input = snapshot_input();
        input.source_versions[0].package_id = PackageId::new(invalid_package_id);

        let error = registry.create_snapshot(request, input).unwrap_err();

        assert!(
            matches!(
                error,
                SnapshotError::InvalidPackageId { package_id }
                    if package_id.as_str() == invalid_package_id
            ),
            "{invalid_package_id:?}"
        );
        assert_eq!(
            registry.allocator.lease_allocations.load(Ordering::Relaxed),
            0,
            "{invalid_package_id:?}"
        );
    }
}

#[test]
fn invalid_module_paths_are_rejected_before_duplicate_module_validation() {
    for invalid_module_path in [
        "",
        "groups..basic",
        "bad-name",
        "groups.bad-name.basic",
        "1bad",
        "groups.1bad.basic",
        "theorem",
        "groups.theorem.basic",
        "step",
        "groups.step.basic",
    ] {
        let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
        let request = request_id();
        let mut input = snapshot_input();
        input.source_versions[0].module_path = ModulePath::new(invalid_module_path);
        input.source_versions[1].module_path = ModulePath::new(invalid_module_path);

        let error = registry.create_snapshot(request, input).unwrap_err();

        assert!(
            matches!(
                error,
                SnapshotError::InvalidModulePath {
                    package_id,
                    module_path,
                } if package_id.as_str() == "mml"
                    && module_path.as_str() == invalid_module_path
            ),
            "{invalid_module_path:?}"
        );
        assert_eq!(
            registry.allocator.lease_allocations.load(Ordering::Relaxed),
            0,
            "{invalid_module_path:?}"
        );
    }
}

#[test]
fn blank_edition_is_rejected_before_snapshot_registration() {
    let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
    let request = request_id();
    let mut input = snapshot_input();
    input.source_versions[0].edition = Edition::new(" ");

    let error = registry.create_snapshot(request, input).unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::InvalidEdition {
            package_id,
            module_path,
            edition,
        } if package_id.as_str() == "mml"
            && module_path.as_str() == "beta"
            && edition.as_str() == " "
    ));
    assert_eq!(
        registry.allocator.lease_allocations.load(Ordering::Relaxed),
        0
    );
}

#[test]
fn generated_source_without_metadata_is_rejected_by_snapshot_creation() {
    let registry = SnapshotRegistry::with_allocator(CountingLeaseAllocator::new());
    let request = request_id();
    let mut input = snapshot_input();
    input.source_versions[0].origin = SourceOrigin::Generated {
        generator: GeneratedSourceKind::new(" "),
    };

    let error = registry.create_snapshot(request, input).unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::GeneratedSourceWithoutMetadata { module_path }
            if module_path.as_str() == "beta"
    ));
    assert_eq!(
        registry.allocator.lease_allocations.load(Ordering::Relaxed),
        0
    );
}

#[test]
fn non_negative_open_buffer_version_is_accepted_by_creation_validation() {
    let registry = SnapshotRegistry::new();
    let mut input = snapshot_input();
    input.source_versions[0].origin = SourceOrigin::OpenBuffer { version: 0 };

    let (snapshot, _) = registry.create_snapshot(request_id(), input).unwrap();

    assert!(
        snapshot
            .source_versions
            .iter()
            .any(|version| { matches!(version.origin, SourceOrigin::OpenBuffer { version: 0 }) })
    );
}

#[test]
fn negative_open_buffer_version_is_rejected_by_creation_validation() {
    let registry = SnapshotRegistry::new();
    let mut input = snapshot_input();
    input.source_versions[0].origin = SourceOrigin::OpenBuffer { version: -1 };

    let error = registry.create_snapshot(request_id(), input).unwrap_err();

    assert!(matches!(
        error,
        SnapshotError::StaleOpenBufferVersion {
            expected: 0,
            actual: -1
        }
    ));
}

#[test]
fn snapshot_error_basic_variants_exist() {
    let source_path_error = SourcePathError::UnsupportedExtension {
        path: Path::new("src/basic.txt").to_owned(),
    };
    let unknown = snapshot_id(6);
    let expected = snapshot_id(7);
    let actual = snapshot_id(8);
    let lease_id = InMemorySessionIdAllocator::new()
        .next_lease_id(expected)
        .unwrap();

    let invalid_source_path = SnapshotError::InvalidSourcePath {
        error: source_path_error,
    };
    let invalid_workspace_root = SnapshotError::InvalidWorkspaceRoot {
        workspace_root: WorkspaceRoot::new(" "),
    };
    let invalid_package_id = SnapshotError::InvalidPackageId {
        package_id: PackageId::new(""),
    };
    let invalid_module_path = SnapshotError::InvalidModulePath {
        package_id: PackageId::new("mml"),
        module_path: ModulePath::new(""),
    };
    let invalid_edition = SnapshotError::InvalidEdition {
        package_id: PackageId::new("mml"),
        module_path: ModulePath::new("groups.basic"),
        edition: Edition::new(" "),
    };
    let duplicate_module_path = SnapshotError::DuplicateModulePath {
        package_id: PackageId::new("mml"),
        module_path: ModulePath::new("groups.basic"),
    };
    let missing_dependency_artifact = SnapshotError::MissingDependencyArtifact {
        artifact: "dep.artifact".to_owned(),
    };
    let unsupported_lockfile_metadata = SnapshotError::UnsupportedLockfileMetadata {
        metadata: "lock-v99".to_owned(),
    };
    let unsupported_toolchain_metadata = SnapshotError::UnsupportedToolchainMetadata {
        metadata: "toolchain-v99".to_owned(),
    };
    let stale_open_buffer_version = SnapshotError::StaleOpenBufferVersion {
        expected: 12,
        actual: 11,
    };
    let generated_source_without_metadata = SnapshotError::GeneratedSourceWithoutMetadata {
        module_path: ModulePath::new("generated.basic"),
    };
    let unknown_snapshot_id = SnapshotError::UnknownSnapshotId {
        snapshot_id: unknown,
    };
    let lease_release_mismatch = SnapshotError::LeaseReleaseMismatch {
        lease_id,
        expected_snapshot: expected,
        actual_snapshot: actual,
    };
    let unknown_snapshot_lease = SnapshotError::UnknownSnapshotLease { lease_id };
    let duplicate_lease_id_allocation = SnapshotError::DuplicateLeaseIdAllocation {
        lease_id,
        existing_snapshot: expected,
        allocated_snapshot: actual,
    };
    let duplicate_source_version_identity = SnapshotError::DuplicateSourceVersionIdentity {
        package_id: PackageId::new("mml"),
        module_path: ModulePath::new("groups.basic"),
        normalized_path: normalized_path("src/groups/basic.miz"),
        source_hash: hash(42),
    };
    let lease_id_allocation = SnapshotError::LeaseIdAllocation {
        error: crate::IdError::AllocatorOverflow,
    };

    assert!(matches!(
        invalid_source_path,
        SnapshotError::InvalidSourcePath {
            error: SourcePathError::UnsupportedExtension { .. }
        }
    ));
    assert!(matches!(
        invalid_workspace_root,
        SnapshotError::InvalidWorkspaceRoot { workspace_root }
            if workspace_root.as_str() == " "
    ));
    assert!(matches!(
        invalid_package_id,
        SnapshotError::InvalidPackageId { package_id } if package_id.as_str().is_empty()
    ));
    assert!(matches!(
        invalid_module_path,
        SnapshotError::InvalidModulePath {
            package_id,
            module_path,
        } if package_id.as_str() == "mml" && module_path.as_str().is_empty()
    ));
    assert!(matches!(
        invalid_edition,
        SnapshotError::InvalidEdition {
            package_id,
            module_path,
            edition,
        } if package_id.as_str() == "mml"
            && module_path.as_str() == "groups.basic"
            && edition.as_str() == " "
    ));
    assert!(matches!(
        duplicate_module_path,
        SnapshotError::DuplicateModulePath {
            package_id,
            module_path
        } if package_id.as_str() == "mml" && module_path.as_str() == "groups.basic"
    ));
    assert!(matches!(
        missing_dependency_artifact,
        SnapshotError::MissingDependencyArtifact { artifact } if artifact == "dep.artifact"
    ));
    assert!(matches!(
        unsupported_lockfile_metadata,
        SnapshotError::UnsupportedLockfileMetadata { metadata } if metadata == "lock-v99"
    ));
    assert!(matches!(
        unsupported_toolchain_metadata,
        SnapshotError::UnsupportedToolchainMetadata { metadata } if metadata == "toolchain-v99"
    ));
    assert!(matches!(
        stale_open_buffer_version,
        SnapshotError::StaleOpenBufferVersion {
            expected: 12,
            actual: 11
        }
    ));
    assert!(matches!(
        generated_source_without_metadata,
        SnapshotError::GeneratedSourceWithoutMetadata { module_path }
            if module_path.as_str() == "generated.basic"
    ));
    assert!(matches!(
        unknown_snapshot_id,
        SnapshotError::UnknownSnapshotId { snapshot_id } if snapshot_id == unknown
    ));
    assert!(matches!(
        lease_release_mismatch,
        SnapshotError::LeaseReleaseMismatch {
            lease_id: actual_lease_id,
            expected_snapshot,
            actual_snapshot,
        } if actual_lease_id == lease_id
            && expected_snapshot == expected
            && actual_snapshot == actual
    ));
    assert!(matches!(
        unknown_snapshot_lease,
        SnapshotError::UnknownSnapshotLease {
            lease_id: actual_lease_id,
        } if actual_lease_id == lease_id
    ));
    assert!(matches!(
        duplicate_lease_id_allocation,
        SnapshotError::DuplicateLeaseIdAllocation {
            lease_id: actual_lease_id,
            existing_snapshot,
            allocated_snapshot,
        } if actual_lease_id == lease_id
            && existing_snapshot == expected
            && allocated_snapshot == actual
    ));
    assert!(matches!(
        duplicate_source_version_identity,
        SnapshotError::DuplicateSourceVersionIdentity {
            package_id,
            module_path,
            normalized_path,
            source_hash,
        } if package_id.as_str() == "mml"
            && module_path.as_str() == "groups.basic"
            && normalized_path.as_str() == "src/groups/basic.miz"
            && source_hash == hash(42)
    ));
    assert!(matches!(
        lease_id_allocation,
        SnapshotError::LeaseIdAllocation {
            error: crate::IdError::AllocatorOverflow,
        }
    ));
}

#[test]
fn snapshot_error_exposes_source_path_error_as_error_source() {
    let invalid_source_path = SnapshotError::InvalidSourcePath {
        error: SourcePathError::UnsupportedExtension {
            path: Path::new("src/basic.txt").to_owned(),
        },
    };
    let missing_dependency_artifact = SnapshotError::MissingDependencyArtifact {
        artifact: "dep.artifact".to_owned(),
    };

    assert!(std::error::Error::source(&invalid_source_path).is_some());
    assert!(std::error::Error::source(&missing_dependency_artifact).is_none());
}

#[test]
fn source_version_canonical_order_does_not_require_source_id_ordering() {
    fn requires_ord<T: Ord>() {}

    requires_ord::<super::SourceVersionCanonicalKey<'_>>();

    let snapshot_id = snapshot_id(6);
    let ids = InMemorySessionIdAllocator::new();
    let low_allocated_source = ids.next_source_id(snapshot_id).unwrap();
    let high_allocated_source = ids.next_source_id(snapshot_id).unwrap();
    let mut versions = vec![
        source_version(
            low_allocated_source,
            "pkg",
            "zeta",
            "src/zeta.miz",
            hash(0),
            SourceOrigin::Disk,
        ),
        source_version(
            high_allocated_source,
            "pkg",
            "alpha",
            "src/alpha.miz",
            hash(0),
            SourceOrigin::Disk,
        ),
    ];

    sort_source_versions_canonical(&mut versions);

    assert_eq!(versions[0].source_id, high_allocated_source);
    assert_eq!(versions[0].module_path.as_str(), "alpha");
}

#[derive(Debug)]
struct CountingLeaseAllocator {
    inner: InMemorySessionIdAllocator,
    lease_allocations: AtomicUsize,
    source_targets: Mutex<Vec<BuildSnapshotId>>,
    source_allocations_remaining: AtomicUsize,
    source_mutation: Mutex<Option<(std::path::PathBuf, String)>>,
}

impl CountingLeaseAllocator {
    fn new() -> Self {
        Self {
            inner: InMemorySessionIdAllocator::new(),
            lease_allocations: AtomicUsize::new(0),
            source_targets: Mutex::new(Vec::new()),
            source_allocations_remaining: AtomicUsize::new(usize::MAX),
            source_mutation: Mutex::new(None),
        }
    }
}

impl SessionIdAllocator for CountingLeaseAllocator {
    fn next_session_id(&self) -> Result<BuildSessionId, IdError> {
        self.inner.next_session_id()
    }

    fn next_request_id(&self) -> Result<BuildRequestId, IdError> {
        self.inner.next_request_id()
    }

    fn next_source_id(&self, snapshot: BuildSnapshotId) -> Result<SourceId, IdError> {
        self.source_targets.lock().unwrap().push(snapshot);
        self.source_allocations_remaining
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |remaining| {
                remaining.checked_sub(1)
            })
            .map_err(|_| IdError::AllocatorOverflow)?;
        if let Some((path, text)) = self.source_mutation.lock().unwrap().take() {
            std::fs::write(path, text).unwrap();
        }
        self.inner.next_source_id(snapshot)
    }

    fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError> {
        self.inner.next_source_map_id(snapshot)
    }

    fn next_lease_id(&self, snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError> {
        self.lease_allocations.fetch_add(1, Ordering::Relaxed);
        self.inner.next_lease_id(snapshot)
    }
}

#[derive(Debug)]
struct RegistryMutexProbeAllocator {
    inner: InMemorySessionIdAllocator,
    state: AtomicPtr<std::sync::Mutex<SnapshotRegistryState>>,
    registry_mutex_available_checks: AtomicUsize,
}

impl RegistryMutexProbeAllocator {
    fn new() -> Self {
        Self {
            inner: InMemorySessionIdAllocator::new(),
            state: AtomicPtr::new(std::ptr::null_mut()),
            registry_mutex_available_checks: AtomicUsize::new(0),
        }
    }

    fn bind_state(&self, state: &std::sync::Mutex<SnapshotRegistryState>) {
        self.state
            .store(std::ptr::from_ref(state).cast_mut(), Ordering::Relaxed);
    }

    fn record_registry_mutex_available(&self) {
        let state = self.state.load(Ordering::Relaxed);
        assert!(
            !state.is_null(),
            "registry mutex probe allocator was not bound to registry state"
        );
        // SAFETY: `bind_state` stores a pointer to this test's registry state,
        // and the registry outlives every allocator call in the test.
        let state = unsafe { &*state };
        match state.try_lock() {
            Ok(_guard) => {
                self.registry_mutex_available_checks
                    .fetch_add(1, Ordering::Relaxed);
            }
            Err(std::sync::TryLockError::WouldBlock) => {
                panic!("registry mutex was held during lease id allocation");
            }
            Err(std::sync::TryLockError::Poisoned(_)) => {
                panic!("snapshot registry mutex poisoned");
            }
        }
    }
}

impl SessionIdAllocator for RegistryMutexProbeAllocator {
    fn next_session_id(&self) -> Result<BuildSessionId, IdError> {
        self.inner.next_session_id()
    }

    fn next_request_id(&self) -> Result<BuildRequestId, IdError> {
        self.inner.next_request_id()
    }

    fn next_source_id(&self, snapshot: BuildSnapshotId) -> Result<SourceId, IdError> {
        self.inner.next_source_id(snapshot)
    }

    fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError> {
        self.inner.next_source_map_id(snapshot)
    }

    fn next_lease_id(&self, snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError> {
        self.record_registry_mutex_available();
        self.inner.next_lease_id(snapshot)
    }
}

#[derive(Debug)]
struct DuplicateLeaseAllocator {
    inner: InMemorySessionIdAllocator,
    lease_id: SnapshotLeaseId,
}

impl DuplicateLeaseAllocator {
    fn new(lease_id: SnapshotLeaseId) -> Self {
        Self {
            inner: InMemorySessionIdAllocator::new(),
            lease_id,
        }
    }
}

impl SessionIdAllocator for DuplicateLeaseAllocator {
    fn next_session_id(&self) -> Result<BuildSessionId, IdError> {
        self.inner.next_session_id()
    }

    fn next_request_id(&self) -> Result<BuildRequestId, IdError> {
        self.inner.next_request_id()
    }

    fn next_source_id(&self, snapshot: BuildSnapshotId) -> Result<SourceId, IdError> {
        self.inner.next_source_id(snapshot)
    }

    fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError> {
        self.inner.next_source_map_id(snapshot)
    }

    fn next_lease_id(&self, _snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError> {
        Ok(self.lease_id)
    }
}

#[derive(Debug)]
struct LeaseFailingAllocator {
    inner: InMemorySessionIdAllocator,
}

impl LeaseFailingAllocator {
    fn new() -> Self {
        Self {
            inner: InMemorySessionIdAllocator::new(),
        }
    }
}

impl SessionIdAllocator for LeaseFailingAllocator {
    fn next_session_id(&self) -> Result<BuildSessionId, IdError> {
        self.inner.next_session_id()
    }

    fn next_request_id(&self) -> Result<BuildRequestId, IdError> {
        self.inner.next_request_id()
    }

    fn next_source_id(&self, snapshot: BuildSnapshotId) -> Result<SourceId, IdError> {
        self.inner.next_source_id(snapshot)
    }

    fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError> {
        self.inner.next_source_map_id(snapshot)
    }

    fn next_lease_id(&self, _snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError> {
        Err(IdError::AllocatorOverflow)
    }
}

#[derive(Debug)]
struct LeaseAllocatorFailsAfter {
    inner: InMemorySessionIdAllocator,
    successful_lease_allocations_remaining: AtomicUsize,
}

impl LeaseAllocatorFailsAfter {
    fn new(successful_lease_allocations: usize) -> Self {
        Self {
            inner: InMemorySessionIdAllocator::new(),
            successful_lease_allocations_remaining: AtomicUsize::new(successful_lease_allocations),
        }
    }
}

impl SessionIdAllocator for LeaseAllocatorFailsAfter {
    fn next_session_id(&self) -> Result<BuildSessionId, IdError> {
        self.inner.next_session_id()
    }

    fn next_request_id(&self) -> Result<BuildRequestId, IdError> {
        self.inner.next_request_id()
    }

    fn next_source_id(&self, snapshot: BuildSnapshotId) -> Result<SourceId, IdError> {
        self.inner.next_source_id(snapshot)
    }

    fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError> {
        self.inner.next_source_map_id(snapshot)
    }

    fn next_lease_id(&self, snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError> {
        self.successful_lease_allocations_remaining
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |remaining| {
                remaining.checked_sub(1)
            })
            .map_err(|_| IdError::AllocatorOverflow)?;
        self.inner.next_lease_id(snapshot)
    }
}

fn source_version(
    source_id: crate::SourceId,
    package_id: &str,
    module_path: &str,
    normalized_path: &str,
    source_hash: Hash,
    origin: SourceOrigin,
) -> SourceVersion {
    source_version_with_edition(
        source_id,
        package_id,
        module_path,
        normalized_path,
        source_hash,
        Edition::new("2026"),
        origin,
    )
}

fn source_version_with_edition(
    source_id: crate::SourceId,
    package_id: &str,
    module_path: &str,
    normalized_path: &str,
    source_hash: Hash,
    edition: Edition,
    origin: SourceOrigin,
) -> SourceVersion {
    SourceVersion {
        source_id,
        package_id: PackageId::new(package_id),
        module_path: ModulePath::new(module_path),
        normalized_path: normalized_path_from_source_path(normalized_path),
        source_hash,
        edition,
        origin,
    }
}

fn snapshot_input() -> SnapshotInput {
    let snapshot_id = snapshot_id(9);
    let ids = InMemorySessionIdAllocator::new();
    snapshot_input_with_sources(vec![
        source_version(
            ids.next_source_id(snapshot_id).unwrap(),
            "mml",
            "beta",
            "src/beta.miz",
            hash(2),
            SourceOrigin::Disk,
        ),
        source_version(
            ids.next_source_id(snapshot_id).unwrap(),
            "mml",
            "alpha",
            "src/alpha.miz",
            hash(1),
            SourceOrigin::OpenBuffer { version: 12 },
        ),
    ])
}

fn snapshot_input_with_sources(source_versions: Vec<SourceVersion>) -> SnapshotInput {
    SnapshotInput {
        workspace_root: WorkspaceRoot::new("workspace"),
        source_versions,
        dependency_artifacts: vec![
            DependencyArtifactRef::new("kernel/order.vo", hash(4)),
            DependencyArtifactRef::new("kernel/base.vo", hash(3)),
        ],
        lockfile_hash: hash(5),
        toolchain: ToolchainInfo::new("mizar-2026.1"),
        verifier_config_hash: hash(6),
    }
}

fn canonical_summary(versions: &[SourceVersion]) -> Vec<(&str, &str, &str, u8)> {
    versions
        .iter()
        .map(|version| {
            (
                version.package_id.as_str(),
                version.module_path.as_str(),
                version.normalized_path.as_str(),
                version.source_hash.as_bytes()[0],
            )
        })
        .collect()
}

fn dependency_summary(artifacts: &[DependencyArtifactRef]) -> Vec<(&str, u8)> {
    artifacts
        .iter()
        .map(|artifact| {
            (
                artifact.artifact.as_str(),
                artifact.content_hash.as_bytes()[0],
            )
        })
        .collect()
}

fn all_retention_reasons() -> [RetentionReason; 8] {
    [
        RetentionReason::ActiveBuild,
        RetentionReason::CurrentWatchBaseline,
        RetentionReason::PublishedLspSnapshot,
        RetentionReason::OpenBufferOverlay,
        RetentionReason::DiagnosticIndex,
        RetentionReason::ExplanationRequest,
        RetentionReason::PhaseOutputReference,
        RetentionReason::PendingWrite,
    ]
}

fn hash(first_byte: u8) -> Hash {
    let mut bytes = [0; Hash::BYTE_LEN];
    bytes[0] = first_byte;
    Hash::from_bytes(bytes)
}

fn zero_hash() -> Hash {
    Hash::from_bytes([0; Hash::BYTE_LEN])
}

fn request_id() -> crate::BuildRequestId {
    InMemorySessionIdAllocator::new().next_request_id().unwrap()
}

fn snapshot_id(first_byte: u8) -> BuildSnapshotId {
    let mut serialized = String::from(
        "mizar-session-build-snapshot-v1:0000000000000000000000000000000000000000000000000000000000000000",
    );
    let hex = format!("{first_byte:02x}");
    serialized.replace_range(
        "mizar-session-build-snapshot-v1:".len().."mizar-session-build-snapshot-v1:".len() + 2,
        &hex,
    );
    BuildSnapshotId::from_published_schema_str(&serialized).unwrap()
}

fn normalized_path(path: &str) -> NormalizedPath {
    normalized_path_from_source_path(path)
}

fn normalized_path_from_source_path(path: &str) -> NormalizedPath {
    let fixture = SourcePathFixture::new();
    fixture.write(path, "");
    normalize_source_path(fixture.root(), Path::new(path)).unwrap()
}

struct SourcePathFixture {
    base: std::path::PathBuf,
    root: std::path::PathBuf,
}

impl SourcePathFixture {
    fn capture_context(
        &self,
    ) -> (
        WorkspaceRoot,
        Vec<DependencyArtifactRef>,
        Hash,
        ToolchainInfo,
        Hash,
    ) {
        (
            WorkspaceRoot::new(self.root.display().to_string()),
            vec![DependencyArtifactRef::new("dependency.vo", hash(23))],
            hash(24),
            ToolchainInfo::new("native-capture-test/v1"),
            hash(25),
        )
    }

    fn capture_source(
        &self,
        module: &str,
        origin: SourceOriginInput,
    ) -> (DiskSourceLoader, SourceInput) {
        (
            DiskSourceLoader::new(&self.root),
            SourceInput {
                package_id: PackageId::new("mml"),
                module_path: ModulePath::new(module),
                normalized_path: normalize_source_path(
                    &self.root,
                    Path::new(&format!("src/{module}.miz")),
                )
                .unwrap(),
                edition: Edition::new("2026"),
                origin,
            },
        )
    }

    fn new() -> Self {
        let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!(
            "mizar_session_snapshot_source_path_{}_{}",
            std::process::id(),
            id
        ));
        let root = base.join("package");
        std::fs::create_dir_all(root.join("src")).unwrap();
        Self { base, root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }
}

impl Drop for SourcePathFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.base);
    }
}

#[test]
fn capture_metadata_fingerprints_bind_retained_values_and_empty_dependencies() {
    let fixture = SourcePathFixture::new();
    fixture.write("src/main.miz", "disk\r\n");
    let registry = SnapshotRegistry::new();
    let request = request_id();
    let mut context = fixture.capture_context();
    context.1.clear();
    let (snapshot, _, retained) = registry
        .load_snapshot(
            request,
            context.clone(),
            vec![fixture.capture_source(
                "main",
                SourceOriginInput::Disk {
                    path: "src/main.miz".into(),
                },
            )],
        )
        .unwrap();
    assert!(snapshot.dependency_artifacts.is_empty());
    assert_eq!(retained[0].0.text.as_ref(), "disk\n");
    assert_eq!(
        snapshot.source_versions[0].source_hash,
        retained[0].0.source_hash
    );
    let before = snapshot.clone();
    let mut bytes = Vec::new();
    for (label, value) in [
        (
            b"domain".as_slice(),
            b"mizar-session/dependency-artifact-references/v1".as_slice(),
        ),
        (
            b"dependency-artifact-summaries".as_slice(),
            0u64.to_le_bytes().as_slice(),
        ),
    ] {
        bytes.extend_from_slice(&(label.len() as u64).to_le_bytes());
        bytes.extend_from_slice(label);
        bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
        bytes.extend_from_slice(value);
    }
    let empty = Hash::from_bytes(*blake3::hash(&bytes).as_bytes());
    assert_eq!(snapshot.dependency_artifact_fingerprint(), empty);
    fixture.write("src/main.miz", "later disk bytes");
    let raw: Arc<str> = Arc::from("editor\r\n");
    let sources = vec![fixture.capture_source(
        "main",
        SourceOriginInput::OpenBuffer {
            uri: format!("file://{}", fixture.root().join("src/main.miz").display()),
            expected_version: 4,
            actual_version: 4,
            text: raw.clone(),
        },
    )];
    let (current, _, loaded) = registry
        .load_snapshot(request, context.clone(), sources.clone())
        .unwrap();
    let (replayed, _, _) = registry
        .load_snapshot(request, context.clone(), sources)
        .unwrap();
    assert_eq!(replayed.id, current.id);
    assert_ne!(current.id, snapshot.id);
    assert_eq!(loaded[0].0.text.as_ref(), "editor\n");
    assert_eq!(loaded[0].0.origin, SourceOrigin::OpenBuffer { version: 4 });
    assert_eq!(raw.as_ref(), "editor\r\n");
    assert_eq!(retained[0].0.text.as_ref(), "disk\n");
    assert!(!registry.is_current_for_request(snapshot.id, request));
    assert!(registry.is_current_for_request(current.id, request));
    // Empty captured references do not establish a complete dependency closure.
    for value in [&snapshot, &current, &replayed] {
        assert_eq!(value.dependency_artifact_fingerprint(), empty);
        let published = value.id.to_published_schema_string().unwrap();
        let hex = published
            .strip_prefix("mizar-session-build-snapshot-v1:")
            .unwrap();
        let mut decoded = [0; Hash::BYTE_LEN];
        for (index, pair) in hex.as_bytes().chunks_exact(2).enumerate() {
            let digits = b"0123456789abcdef";
            let high = digits.iter().position(|digit| *digit == pair[0]).unwrap();
            let low = digits.iter().position(|digit| *digit == pair[1]).unwrap();
            decoded[index] = (high * 16 + low) as u8;
        }
        assert_eq!(value.id.fingerprint().as_bytes(), &decoded);
    }
    assert_eq!(snapshot, before);
    let mut detached = snapshot.clone();
    detached
        .dependency_artifacts
        .push(DependencyArtifactRef::new(
            "supplied-only.vo",
            hash_text("metadata"),
        ));
    assert_eq!(detached.id.fingerprint(), snapshot.id.fingerprint());
    assert_ne!(detached.dependency_artifact_fingerprint(), empty);
    assert_eq!(registry.get(detached.id), Some(snapshot));
}

#[test]
fn capture_metadata_fingerprints_bind_exact_dependency_references() {
    let fixture = SourcePathFixture::new();
    fixture.write("src/main.miz", "source");
    let registry = SnapshotRegistry::new();
    let sources = vec![fixture.capture_source(
        "main",
        SourceOriginInput::Disk {
            path: "src/main.miz".into(),
        },
    )];
    // These are supplied reference metadata, not authenticated files or a manifest.
    let first = hash_text("supplied content");
    let mut later_bytes = *first.as_bytes();
    later_bytes[Hash::BYTE_LEN - 1] ^= 1;
    let base = vec![
        DependencyArtifactRef::new("é:dep.vo", first),
        DependencyArtifactRef::new("a/dep.vo", first),
        DependencyArtifactRef::new("a/dep.vo", Hash::from_bytes(later_bytes)),
        DependencyArtifactRef::new("a/dep.vo", first),
    ];
    let mut context = fixture.capture_context();
    context.1 = base.clone();
    let (snapshot, _, _) = registry
        .load_snapshot(request_id(), context.clone(), sources.clone())
        .unwrap();
    let expected = snapshot.dependency_artifact_fingerprint();
    let before = snapshot.clone();
    let mut cases = vec![(base.clone(), true), (Vec::new(), false)];
    let mut reversed = base.clone();
    reversed.reverse();
    cases.push((reversed, true));
    let mut removed = base.clone();
    removed.pop();
    cases.push((removed, false));
    let mut duplicated = base.clone();
    duplicated.push(base[0].clone());
    cases.push((duplicated, false));
    let mut renamed = base.clone();
    renamed[0].artifact.push('x');
    cases.push((renamed, false));
    let mut reassociated = base.clone();
    let other = reassociated[2].content_hash;
    reassociated[2].content_hash = reassociated[0].content_hash;
    reassociated[0].content_hash = other;
    cases.push((reassociated, false));
    for index in 0..Hash::BYTE_LEN {
        let mut changed = base.clone();
        let mut bytes = *changed[0].content_hash.as_bytes();
        bytes[index] ^= 1;
        changed[0].content_hash = Hash::from_bytes(bytes);
        cases.push((changed, false));
    }
    for (references, same) in cases {
        let mut input = context.clone();
        input.1 = references.clone();
        let (value, _, _) = registry
            .load_snapshot(request_id(), input, sources.clone())
            .unwrap();
        let value_before = value.clone();
        // Independent profile oracle: no production encoder is called here.
        let mut ordered = references.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|reference| (&reference.artifact, reference.content_hash.as_bytes()));
        let count = (ordered.len() as u64).to_le_bytes();
        let mut parts = vec![
            (
                b"domain".as_slice(),
                b"mizar-session/dependency-artifact-references/v1".as_slice(),
            ),
            (
                b"dependency-artifact-summaries".as_slice(),
                count.as_slice(),
            ),
        ];
        for reference in ordered {
            parts.push((b"dependency/artifact", reference.artifact.as_bytes()));
            parts.push((
                b"dependency/content-hash",
                reference.content_hash.as_bytes(),
            ));
        }
        let mut encoded = Vec::new();
        for (label, value) in parts {
            encoded.extend_from_slice(&(label.len() as u64).to_le_bytes());
            encoded.extend_from_slice(label);
            encoded.extend_from_slice(&(value.len() as u64).to_le_bytes());
            encoded.extend_from_slice(value);
        }
        let actual = value.dependency_artifact_fingerprint();
        assert_eq!(actual, Hash::from_bytes(*blake3::hash(&encoded).as_bytes()));
        assert_eq!(actual == expected, same);
        assert_eq!(value.dependency_artifact_fingerprint(), actual);
        assert_eq!(value, value_before);
    }
    for field in 0..6 {
        let mut changed = context.clone();
        let mut inputs = sources.clone();
        match field {
            0 => {
                changed.0 =
                    WorkspaceRoot::new(fixture.root().parent().unwrap().display().to_string())
            }
            1 => changed.2 = hash_text("other lockfile"),
            2 => changed.3 = ToolchainInfo::new("other-toolchain"),
            3 => changed.4 = hash_text("other verifier"),
            4 => inputs[0].1.edition = Edition::new("2027"),
            _ => {
                inputs[0].1.origin = SourceOriginInput::OpenBuffer {
                    uri: format!("file://{}", fixture.root().join("src/main.miz").display()),
                    expected_version: 1,
                    actual_version: 1,
                    text: Arc::from("different source"),
                }
            }
        }
        let (value, _, _) = registry
            .load_snapshot(request_id(), changed, inputs)
            .unwrap();
        assert_ne!(value.id, snapshot.id);
        assert_eq!(value.dependency_artifact_fingerprint(), expected);
    }
    assert_eq!(snapshot, before);
}
