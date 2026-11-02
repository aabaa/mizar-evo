use super::*;

#[test]
fn import_discovery_preserves_legacy_submit_and_supplied_overlay_precedence() {
    for mode in ["legacy", "complete", "package_only"] {
        let fixture = WorkspaceLeafFixture::new(
            b"import alpha.leaf;\ntheorem T: a combine b = a;\n",
            b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n",
            None,
            usize::MAX,
        );
        let overlay = match mode {
            "complete" => complete_leaf_import_overlay(),
            "package_only" => ModuleDependencyOverlay::package_only(Vec::new()),
            _ => ModuleDependencyOverlay::unavailable(),
        };
        let (request, input) = fixture.scheduled_request(overlay, Vec::new(), Vec::new());
        let mut driver = fixture.scheduled_driver();
        if mode == "legacy" {
            let error = driver
                .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
                .unwrap_err();
            assert!(matches!(error, DriverSubmitError::TaskGraph { .. }));
            continue;
        }
        let submission = driver
            .submit_with_import_discovery(request, &fixture.ids, &SnapshotRegistry::new(), input)
            .unwrap();
        let graph = submission.task_graph.as_ref().unwrap();
        let main = fixture.task(TaskKind::Frontend, "main");
        let leaf = fixture.task(TaskKind::Frontend, "leaf");
        let semantic = graph
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::ModuleResolve && task.unit == main.unit)
            .unwrap();
        assert_eq!(
            semantic.dependency_coverage,
            if mode == "complete" {
                DependencyCoverage::Complete
            } else {
                DependencyCoverage::PackageConservative
            }
        );
        assert_eq!(
            graph
                .edges()
                .iter()
                .filter(|edge| { edge.dependent == main.id && edge.dependency == leaf.id })
                .count(),
            usize::from(mode == "complete")
        );
        let run = submission.scheduler_run.as_ref().unwrap();
        let result = &run.phase_results[&main.id][0];
        assert_eq!(
            result.status,
            if mode == "complete" {
                PhaseStatus::Complete
            } else {
                PhaseStatus::Blocking
            }
        );
    }
}

#[test]
fn import_discovered_native_index_preserves_captured_import_runs_and_original_batches() {
    use mizar_diagnostics::aggregator::BuildDiagnosticIndex;
    for (origin, diagnostic, workers, order) in [
        ("disk", false, 1, CompletionOrder::Canonical),
        ("open", true, 4, CompletionOrder::Reverse),
        ("mixed", false, 4, CompletionOrder::Canonical),
    ] {
        let main = if diagnostic {
            "import alpha.leaf;\ntheorem T: ;\n"
        } else {
            "import alpha.leaf;\ntheorem T: a combine b = a;\n"
        };
        let leaf =
            "definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n";
        let fixture = Fixture::new(main.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut draft, _) = fixture.submit_request(vec![], vec![]);
        draft.source_inputs.versions.clear();
        let descriptors = [("main", main), ("leaf", leaf)]
            .into_iter()
            .map(|(module, text)| {
                let path = format!("src/{module}.miz");
                let raw = format!("\u{feff}{}", text.replace('\n', "\r\n"));
                fs::write(fixture.root.join("alpha").join(&path), &raw).unwrap();
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
                        origin: if origin == "disk" || origin == "mixed" && module == "leaf" {
                            SourceOriginInput::Disk { path: path.into() }
                        } else {
                            SourceOriginInput::OpenBuffer {
                                uri: format!(
                                    "file://{}",
                                    fixture.root.join("alpha").join(&path).display()
                                ),
                                expected_version: 7,
                                actual_version: 7,
                                text: raw.into(),
                            }
                        },
                    },
                )
            })
            .collect();
        let session = draft
            .capture_sources(&ids, &snapshots, descriptors)
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        for module in ["main", "leaf"] {
            let path = fixture.root.join(format!("alpha/src/{module}.miz"));
            fs::remove_file(path).unwrap();
        }
        let mut reference = None;
        for indexed in [false, true] {
            let (mut driver, owner) =
                import_discovered_native_index_driver(snapshot, &["main", "leaf"]);
            let (_, mut input) = fixture.submit_request(vec![], vec![]);
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
            if !indexed {
                reference = Some(
                    driver
                        .submit_captured_with_import_discovery(
                            session.clone(),
                            &ids,
                            &snapshots,
                            input,
                        )
                        .unwrap(),
                );
                continue;
            }
            let (submission, index) = driver
                .submit_captured_with_import_discovery_and_native_diagnostic_index(
                    session.clone(),
                    &ids,
                    &snapshots,
                    input,
                )
                .unwrap();
            assert_eq!(submission, reference.take().unwrap());
            let graph = submission.task_graph.as_ref().unwrap();
            let frontend = |name| {
                graph.tasks().iter().find(|task| task.kind == TaskKind::Frontend && matches!(&task.unit, WorkUnit::Module { module } if module.path.as_str() == name)).unwrap()
            };
            assert!(frontend("main").dependencies.contains(&frontend("leaf").id));
            assert!(
                graph
                    .tasks()
                    .iter()
                    .filter(|task| task.kind == TaskKind::ModuleResolve)
                    .all(|task| task.dependency_coverage
                        == DependencyCoverage::MissingModuleDependencyOverlay)
            );
            let run = submission.scheduler_run.as_ref().unwrap();
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(if diagnostic {
                    BuildSessionOutcome::Failed
                } else {
                    BuildSessionOutcome::Blocked
                })
            );
            let index = index.unwrap();
            assert_eq!(
                index,
                BuildDiagnosticIndex::from_batches(
                    snapshot,
                    run.phase_results
                        .values()
                        .flatten()
                        .flat_map(|result| result.diagnostics.clone())
                        .collect()
                )
                .unwrap()
            );
            assert_eq!(index.is_empty(), !diagnostic);
            for (_, captured, _) in session.captured().native_sources().unwrap() {
                let module = captured.module_path.as_str();
                let task = graph.tasks().iter().find(|task| task.kind == TaskKind::SourceLoad && matches!(&task.unit, WorkUnit::Module { module: id } if id.path.as_str() == module)).unwrap();
                let source_result = &run.phase_results[&task.id][0];
                let frontend_task = frontend(module);
                let frontend_result = &run.phase_results[&frontend_task.id][0];
                assert_eq!(source_result.status, PhaseStatus::Complete);
                assert_eq!(
                    frontend_result.status,
                    if diagnostic && module == "main" {
                        PhaseStatus::Recoverable
                    } else {
                        PhaseStatus::Complete
                    }
                );
                assert!(source_result.cache_observation.is_some());
                assert!(frontend_result.cache_observation.is_some());
                let source = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<SourceUnit>(
                                &source_result.output_refs[0],
                                &OutputKind::new("SourceUnit"),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                assert_eq!(source.source_id, captured.source_id);
                assert_eq!(source.source_hash, captured.source_hash);
                assert_eq!(source.origin, captured.origin);
                assert_eq!(source.source_text, captured.text);
                assert_eq!(
                    source.source_text.as_ref(),
                    if module == "main" { main } else { leaf }
                );
                assert_eq!(source.line_map, captured.line_map);
                assert_eq!(source.loading_map, captured.loading_map);
                assert!(source.loading_map.is_some());
                for phase in [PipelinePhase::SourceLoad, PipelinePhase::Frontend] {
                    let (dependencies, parents) = if phase == PipelinePhase::Frontend {
                        let mut dependencies = submission
                            .module_index
                            .as_ref()
                            .unwrap()
                            .dependency_summaries
                            .iter()
                            .map(|summary| summary.content_hash)
                            .collect::<Vec<_>>();
                        dependencies.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
                        let parents = frontend_task
                            .dependencies
                            .iter()
                            .flat_map(|id| run.phase_results.get(id).into_iter().flatten())
                            .filter(|result| result.status == PhaseStatus::Complete)
                            .flat_map(|result| &result.output_refs)
                            .filter(|parent| {
                                (parent.phase() == &IrPipelinePhase::new("SourceLoad")
                                    && parent.work_unit()
                                        == source_result.output_refs[0].work_unit()
                                    && parent.output_kind() == &OutputKind::new("SourceUnit"))
                                    || (parent.phase() == &IrPipelinePhase::new("Frontend")
                                        && parent.output_kind()
                                            == &OutputKind::new("FrontendOutput"))
                            })
                            .map(|parent| {
                                SealedParentOutputHandle::from_published_output(
                                    &owner,
                                    snapshot,
                                    parent.clone(),
                                )
                                .unwrap()
                            })
                            .collect();
                        (dependencies, parents)
                    } else {
                        (vec![], vec![])
                    };
                    let dispatch = PhaseInput::new(
                        task.unit.clone(),
                        PhaseDispatchInputBundle::new(
                            snapshot,
                            SourceUnitCacheKey::from_source(&source).stable_hash(),
                            dependencies,
                            parents,
                        )
                        .unwrap(),
                    );
                    assert!(
                        matches!(driver.registry().cache_key_for_phase(phase, &dispatch).unwrap().intent, PhaseCacheIntent::NoKey { reason } if !reason.is_empty())
                    );
                }
                let records = index.records().iter().filter(|record| matches!(record.primary_location(), DiagnosticPrimaryLocation::Span(span) if span.range().source_id == captured.source_id)).collect::<Vec<_>>();
                assert_eq!(!records.is_empty(), diagnostic && module == "main");
                for record in records {
                    let DiagnosticPrimaryLocation::Span(span) = record.primary_location() else {
                        unreachable!()
                    };
                    assert!(span.range().start <= span.range().end);
                    assert!(span.range().end <= source.source_text.len());
                    assert_eq!(record.phase(), DiagnosticPhase::Frontend);
                    assert_eq!(record.handle().snapshot, snapshot);
                }
            }
        }
    }
}

#[test]
fn import_discovered_native_index_preserves_discovery_failure_and_supplied_overlays() {
    use mizar_build::{module_index::ModuleId, task_graph::ModuleDependencyCoverage};
    for mode in ["failure", "imports_only", "unavailable_edge"] {
        let main = if mode == "failure" {
            "import alpha.leaf;\nimport alpha.missing;\ndefinition\nend;\n"
        } else {
            "import alpha.leaf;\ndefinition\nend;\n"
        };
        let fixture = Fixture::new(main.as_bytes());
        fs::write(
            fixture.root.join("alpha/src/leaf.miz"),
            "definition\nend;\n",
        )
        .unwrap();
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut draft, _) = fixture.submit_request(vec![], vec![]);
        draft.source_inputs.versions.clear();
        let descriptors = ["main", "leaf"]
            .into_iter()
            .map(|module| {
                (
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    SourceInput {
                        package_id: PackageId::new("alpha"),
                        module_path: ModulePath::new(module),
                        normalized_path: normalize_path(
                            &fixture.root.join("alpha"),
                            Path::new(&format!("src/{module}.miz")),
                        )
                        .unwrap(),
                        edition: Edition::new("2025"),
                        origin: SourceOriginInput::Disk {
                            path: format!("src/{module}.miz").into(),
                        },
                    },
                )
            })
            .collect();
        let session = draft
            .capture_sources(&ids, &snapshots, descriptors)
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        let mut reference = None;
        for indexed in [false, true] {
            let (mut driver, _) =
                import_discovered_native_index_driver(snapshot, &["main", "leaf"]);
            let (_, mut input) = fixture.submit_request(vec![], vec![]);
            input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
                package_id: PackageId::new("alpha"),
                files: vec![
                    WorkspaceSourceFile::new("src/main.miz", "main.miz"),
                    WorkspaceSourceFile::new("src/leaf.miz", "leaf.miz"),
                ],
            }]);
            // Supplied overlays are adversarial caller inputs, never real Complete producers.
            input.dependency_overlay = match mode {
                "imports_only" => ModuleDependencyOverlay {
                    coverage: ModuleDependencyCoverage::ImportsOnly,
                    edges: vec![],
                },
                "unavailable_edge" => ModuleDependencyOverlay {
                    coverage: ModuleDependencyCoverage::Unavailable,
                    edges: vec![ModuleDependencyEdge {
                        dependent: ModuleId::new(PackageId::new("alpha"), ModulePath::new("main")),
                        dependency: ModuleId::new(PackageId::new("alpha"), ModulePath::new("leaf")),
                        kind: ModuleDependencyKind::ImportSummary,
                    }],
                },
                _ => ModuleDependencyOverlay::unavailable(),
            };
            let result = if indexed {
                driver.submit_captured_with_import_discovery_and_native_diagnostic_index(
                    session.clone(),
                    &ids,
                    &snapshots,
                    input,
                )
            } else {
                driver
                    .submit_captured_with_import_discovery(session.clone(), &ids, &snapshots, input)
                    .map(|submission| (submission, None))
            };
            if !indexed {
                reference = Some(result);
                continue;
            }
            match (reference.take().unwrap(), result) {
                (Err(expected), Err(actual)) => {
                    assert!(matches!(mode, "failure" | "unavailable_edge"));
                    assert_eq!(actual, expected);
                    let DriverSubmitError::TaskGraph { diagnostics, .. } = actual else {
                        panic!("discovery must fail closed before scheduling")
                    };
                    assert!(
                        diagnostics
                            .diagnostics()
                            .iter()
                            .any(|diagnostic| diagnostic.kind
                                == TaskGraphDiagnosticKind::MissingModuleDependencyOverlay)
                    );
                }
                (Ok((expected, _)), Ok((submission, index))) => {
                    assert!(!matches!(mode, "failure" | "unavailable_edge"));
                    assert_eq!(submission, expected);
                    let graph = submission.task_graph.as_ref().unwrap();
                    let frontend = |name| {
                        graph.tasks().iter().find(|task| task.kind == TaskKind::Frontend && matches!(&task.unit, WorkUnit::Module { module } if module.path.as_str() == name)).unwrap()
                    };
                    assert!(!frontend("main").dependencies.contains(&frontend("leaf").id));
                    assert!(
                        graph
                            .tasks()
                            .iter()
                            .filter(|task| task.kind == TaskKind::ModuleResolve)
                            .all(|task| task.dependency_coverage
                                == DependencyCoverage::MissingModuleDependencyOverlay)
                    );
                    assert!(index.is_some());
                }
                other => panic!("discovery reference mismatch: {other:?}"),
            }
        }
    }
}

#[test]
fn import_discovered_native_index_preserves_native_eligibility_currentness_and_errors() {
    for mode in [
        "current",
        "obsolete_publisher",
        "generated",
        "invalid_registry",
    ] {
        let fixture = Fixture::new(b"definition\nend;\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let invalid_registry = SnapshotRegistry::new();
        let (mut draft, _) = fixture.submit_request(vec![], vec![]);
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
                        origin: if mode == "generated" {
                            SourceOriginInput::Generated {
                                generator: mizar_session::GeneratedSourceKind::new(
                                    "import-index-guard",
                                ),
                                text: "definition\nend;\n".into(),
                                anchor: None,
                            }
                        } else {
                            SourceOriginInput::OpenBuffer {
                                uri: format!(
                                    "file://{}",
                                    fixture.root.join("alpha/src/main.miz").display()
                                ),
                                expected_version: 7,
                                actual_version: 7,
                                text: "definition\nend;\n".into(),
                            }
                        },
                    },
                )],
            )
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        let registry = if mode == "invalid_registry" {
            &invalid_registry
        } else {
            &snapshots
        };
        let mut reference = None;
        for indexed in [false, true] {
            let (mut driver, owner) = import_discovered_native_index_driver(snapshot, &["main"]);
            if mode == "obsolete_publisher" {
                owner.mark_obsolete(snapshot).unwrap();
            }
            let (_, mut input) = fixture.submit_request(vec![], vec![]);
            input.dependency_overlay = if mode == "generated" {
                // Only bypass discovery as caller input to isolate native guard rejection.
                ModuleDependencyOverlay {
                    coverage: mizar_build::task_graph::ModuleDependencyCoverage::ImportsOnly,
                    edges: vec![],
                }
            } else {
                ModuleDependencyOverlay::unavailable()
            };
            let result = if indexed {
                driver.submit_captured_with_import_discovery_and_native_diagnostic_index(
                    session.clone(),
                    &ids,
                    registry,
                    input,
                )
            } else {
                driver
                    .submit_captured_with_import_discovery(session.clone(), &ids, registry, input)
                    .map(|submission| (submission, None))
            };
            if !indexed {
                reference = Some(result);
                continue;
            }
            if mode == "invalid_registry" {
                let actual = result.unwrap_err();
                assert_eq!(actual, reference.take().unwrap().unwrap_err());
                assert!(matches!(
                    actual,
                    DriverSubmitError::InvalidCapturedSession { .. }
                ));
                continue;
            }
            let (submission, index) = result.unwrap();
            assert_eq!(submission, reference.take().unwrap().unwrap().0);
            assert_eq!(index.is_some(), mode == "current");
            assert_eq!(
                driver
                    .validate_current_native_diagnostics(session.id(), &snapshots)
                    .is_ok(),
                mode == "current"
            );
            if mode == "current" {
                assert_eq!(
                    submission.session.state(),
                    BuildSessionState::Finished(BuildSessionOutcome::Blocked)
                );
                assert!(submission.scheduler_run.is_some());
                let index = index.unwrap();
                assert!(index.is_empty());
            }
        }
    }
}
