use super::*;
use crate::{
    cache_seam::{CacheDiagnosticRef, CacheFallbackReason, CacheOutputRef, CacheTaskDecision},
    failure_state::{BlockReason, FailureCategory},
    module_index::{ModuleId, ModuleIndex, ModuleIndexEntry, ModuleIndexLocation},
    planner::{
        BuildConfig, BuildPlan, DependencyGraph, Lockfile, PackagePlan, PackagePlanSource,
        VerifierConfig, WorkspaceBuildConfig, WorkspaceVerifierConfig,
    },
    resource::{ResourceAdmissionStatus, ResourceBudget, ResourceScope},
    task_graph::{
        BackendProfileId, DocumentationProfile, ModuleDependencyOverlay, PipelinePhase,
        ResourceClass, TaskGraphInput, TaskGraphProfile, VcOrderKey, VcTaskDescriptor,
        VcTaskDescriptorId, WorkUnit, build_task_graph,
    },
};
use mizar_session::{Edition, Hash, ModulePath, PackageId, ToolchainInfo, WorkspaceRoot};
use semver::Version;

#[test]
fn shuffled_completion_and_worker_count_collate_identical_results() {
    let graph = multi_module_graph();
    let (canonical, serial_batches) = run_scheduler_with_dispatch_batches(SchedulerInput {
        graph: graph.clone(),
        completion_order: CompletionOrder::Canonical,
        worker_count: 1,
        ..SchedulerInput::new(graph.clone())
    })
    .expect("scheduler run succeeds");
    let (shuffled, parallel_batches) = run_scheduler_with_dispatch_batches(SchedulerInput {
        graph: graph.clone(),
        completion_order: CompletionOrder::Reverse,
        worker_count: 2,
        ..SchedulerInput::new(graph)
    })
    .expect("scheduler run succeeds");

    assert_ne!(serial_batches, parallel_batches);
    assert!(parallel_batches.iter().any(|batch| batch.len() == 2));
    assert_eq!(canonical.task_states, shuffled.task_states);
    assert_eq!(canonical.results, shuffled.results);
    assert_eq!(canonical.failure_records, shuffled.failure_records);
    assert_eq!(canonical.blocked_records, shuffled.blocked_records);
    assert_eq!(canonical.events, shuffled.events);
    assert_eq!(canonical.diagnostics, shuffled.diagnostics);
}

#[test]
fn package_resolve_root_starts_completed_and_non_roots_progress() {
    let graph = sample_graph();
    let run = run_scheduler(SchedulerInput::new(graph)).expect("scheduler run succeeds");
    let root = run
        .task_states
        .iter()
        .find(|record| {
            run.results.iter().any(|result| {
                result.task_id == record.task_id
                    && result.output_refs[0].content == "PackageResolve"
            })
        })
        .expect("root state exists");

    assert_eq!(root.state, TaskState::Completed);
    assert!(
        run.task_states
            .iter()
            .all(|record| matches!(record.state, TaskState::Completed))
    );
}

#[test]
fn immutable_outputs_and_diagnostics_are_canonicalized() {
    let graph = sample_graph();
    let task = task_id_for_kind(&graph, TaskKind::Frontend);
    let run = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        task_outcomes: vec![SyntheticTaskOutcome {
            task_id: task.clone(),
            status: SyntheticTaskStatus::Complete,
            outputs: vec![
                SyntheticOutputRef::new("b", "second"),
                SyntheticOutputRef::new("a", "first"),
            ],
            diagnostics: vec![
                SchedulerDiagnosticRef::new("main", "W002", "later"),
                SchedulerDiagnosticRef::new("main", "W001", "earlier"),
            ],
        }],
        ..SchedulerInput::new(graph)
    })
    .expect("scheduler run succeeds");

    let result = result_for(&run, &task);
    assert_eq!(
        result.output_refs,
        vec![
            SyntheticOutputRef::new("a", "first"),
            SyntheticOutputRef::new("b", "second"),
        ]
    );
    assert_eq!(
        result.diagnostics,
        vec![
            SchedulerDiagnosticRef::new("main", "W001", "earlier"),
            SchedulerDiagnosticRef::new("main", "W002", "later"),
        ]
    );
}

#[test]
fn dependency_coverage_controls_readiness() {
    let package_conservative =
        graph_with_overlay(ModuleDependencyOverlay::package_only(Vec::new()))
            .expect("package-only graph builds");
    let package_run =
        run_scheduler(SchedulerInput::new(package_conservative)).expect("scheduler run succeeds");
    assert!(
        package_run
            .task_states
            .iter()
            .any(
                |record| record.dependency_coverage == DependencyCoverage::PackageConservative
                    && record.state == TaskState::Completed
            )
    );

    let missing_overlay = graph_with_overlay(ModuleDependencyOverlay::unavailable())
        .expect_err("graph reports missing overlay");
    assert!(
        missing_overlay
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind
                == crate::task_graph::TaskGraphDiagnosticKind::MissingModuleDependencyOverlay)
    );

    let mut scheduler_graph = sample_graph();
    let module_resolve = scheduler_graph
        .tasks
        .iter_mut()
        .find(|task| task.kind == TaskKind::ModuleResolve)
        .expect("module resolve task exists");
    let module_resolve_id = module_resolve.id.clone();
    module_resolve.dependency_coverage = DependencyCoverage::MissingModuleDependencyOverlay;
    let scheduler_run =
        run_scheduler(SchedulerInput::new(scheduler_graph)).expect("scheduler run succeeds");
    assert_eq!(
        state_for(&scheduler_run, &module_resolve_id),
        TaskState::Blocked
    );
    assert!(scheduler_run.diagnostics.iter().any(|diagnostic| {
        diagnostic.task_id.as_ref() == Some(&module_resolve_id)
            && diagnostic.kind == SchedulerDiagnosticKind::IncompleteDependencyCoverage
    }));
    assert!(scheduler_run.blocked_records.iter().any(|record| {
        record.task_id == module_resolve_id
            && record.reason == BlockReason::MissingDependencyCoverage
            && record.blocked_by.is_empty()
    }));
}

#[test]
fn missing_vc_descriptors_block_synthetic_scheduler_tasks() {
    let mut graph = sample_graph();
    let artifact = graph
        .tasks
        .iter_mut()
        .find(|task| task.kind == TaskKind::ArtifactCommit)
        .expect("artifact task exists");
    artifact.dependency_coverage = DependencyCoverage::MissingVcDescriptors;

    let run = run_scheduler(SchedulerInput::new(graph)).expect("scheduler run succeeds");
    assert!(run.task_states.iter().any(|record| {
        record.dependency_coverage == DependencyCoverage::MissingVcDescriptors
            && record.state == TaskState::Blocked
    }));
    assert!(run.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == SchedulerDiagnosticKind::IncompleteDependencyCoverage
    }));
    assert!(run.blocked_records.iter().any(|record| {
        record.reason == BlockReason::MissingDependencyCoverage && record.blocked_by.is_empty()
    }));
}

#[test]
fn scheduler_routes_all_expected_queues() {
    let run = run_scheduler(SchedulerInput::new(sample_graph())).expect("scheduler run succeeds");
    let queues = run
        .task_states
        .iter()
        .map(|record| record.queue)
        .collect::<BTreeSet<_>>();

    assert_eq!(
        queues,
        BTreeSet::from([
            SchedulerQueue::Coordinator,
            SchedulerQueue::SourceLocalCpu,
            SchedulerQueue::DeterministicProof,
            SchedulerQueue::AtpPortfolio,
            SchedulerQueue::AtpProcess,
            SchedulerQueue::Kernel,
            SchedulerQueue::IoCommit,
            SchedulerQueue::Documentation,
        ])
    );
}

#[test]
fn resource_budget_queues_source_work_without_changing_collation() {
    let graph = multi_module_graph();
    let mut budget = ResourceBudget::unbounded();
    budget.source_workers = 1;
    let (limited, batches) = run_scheduler_with_dispatch_batches(SchedulerInput {
        graph: graph.clone(),
        worker_count: 2,
        resource_budget: budget.clone(),
        ..SchedulerInput::new(graph.clone())
    })
    .expect("scheduler run succeeds");
    let (reverse_limited, reverse_batches) = run_scheduler_with_dispatch_batches(SchedulerInput {
        graph: graph.clone(),
        worker_count: 2,
        completion_order: CompletionOrder::Reverse,
        resource_budget: budget,
        ..SchedulerInput::new(graph.clone())
    })
    .expect("scheduler run succeeds");
    let serial = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        worker_count: 1,
        ..SchedulerInput::new(graph.clone())
    })
    .expect("scheduler run succeeds");

    assert_eq!(serial.task_states, limited.task_states);
    assert_eq!(serial.results, limited.results);
    assert_eq!(serial.events, limited.events);
    assert_eq!(serial.diagnostics, limited.diagnostics);
    assert_eq!(limited.task_states, reverse_limited.task_states);
    assert_eq!(limited.results, reverse_limited.results);
    assert_eq!(limited.events, reverse_limited.events);
    assert_eq!(limited.diagnostics, reverse_limited.diagnostics);
    assert_eq!(
        limited.resource_telemetry,
        reverse_limited.resource_telemetry
    );
    assert!(limited.resource_telemetry.iter().any(|telemetry| {
        telemetry.status == ResourceAdmissionStatus::Delayed
            && telemetry.blocking_scope == Some(ResourceScope::SourceWorkers)
    }));
    assert!(batches.iter().all(|batch| {
        batch
            .iter()
            .filter(|task_id| {
                matches!(
                    task_kind_for_id(&graph, task_id),
                    TaskKind::SourceLoad
                        | TaskKind::Frontend
                        | TaskKind::ModuleResolve
                        | TaskKind::CheckAndElaborate
                        | TaskKind::VcGenerate
                )
            })
            .count()
            <= 1
    }));
    assert_eq!(batches, reverse_batches);
}

#[test]
fn impossible_resource_request_blocks_with_stable_diagnostic() {
    let graph = sample_graph();
    let source = task_id_for_kind(&graph, TaskKind::SourceLoad);
    let mut budget = ResourceBudget::unbounded();
    budget.source_workers = 0;

    let run = run_scheduler(SchedulerInput {
        graph,
        resource_budget: budget,
        ..SchedulerInput::new(sample_graph())
    })
    .expect("scheduler run succeeds");

    assert_eq!(state_for(&run, &source), TaskState::Blocked);
    assert!(run.diagnostics.iter().any(|diagnostic| {
        diagnostic.task_id.as_ref() == Some(&source)
            && diagnostic.kind == SchedulerDiagnosticKind::ImpossibleResourceRequest
            && diagnostic.value.as_deref() == Some("source-workers")
    }));
    assert!(run.blocked_records.iter().any(|record| {
        record.task_id == source
            && record.reason == BlockReason::ImpossibleResourceRequest
            && record.blocked_by.is_empty()
    }));
    assert!(run.resource_telemetry.iter().any(|telemetry| {
        telemetry.task_id == source
            && telemetry.status == ResourceAdmissionStatus::Impossible
            && telemetry.blocking_scope == Some(ResourceScope::SourceWorkers)
    }));
}

#[test]
fn io_commit_permits_serialize_commit_work_without_publication_authority() {
    let graph = multi_module_graph();
    let mut budget = ResourceBudget::unbounded();
    budget.io_commits = 1;
    let (run, batches) = run_scheduler_with_dispatch_batches(SchedulerInput {
        graph: graph.clone(),
        worker_count: 4,
        resource_budget: budget,
        ..SchedulerInput::new(graph.clone())
    })
    .expect("scheduler run succeeds");

    assert!(batches.iter().all(|batch| {
        batch
            .iter()
            .filter(|task_id| task_kind_for_id(&graph, task_id) == TaskKind::ArtifactCommit)
            .count()
            <= 1
    }));
    assert!(run.resource_telemetry.iter().any(|telemetry| {
        telemetry.status == ResourceAdmissionStatus::Delayed
            && telemetry.blocking_scope == Some(ResourceScope::IoCommits)
    }));
    let debug = format!("{run:#?}").to_lowercase();
    assert!(!debug.contains("publicationtoken"));
    assert!(!debug.contains("trustedstatus"));
}

#[test]
fn backend_limits_bound_global_processes_and_obligation_fanout() {
    let graph = multi_backend_graph();
    let mut global_budget = ResourceBudget::unbounded();
    global_budget.atp_processes = 1;
    global_budget.backend_fanout = 2;
    let (global_limited, global_batches) = run_scheduler_with_dispatch_batches(SchedulerInput {
        graph: graph.clone(),
        worker_count: 4,
        resource_budget: global_budget,
        ..SchedulerInput::new(graph.clone())
    })
    .expect("scheduler run succeeds");

    assert_backend_batches_are_serial(&graph, &global_batches);
    assert!(global_limited.resource_telemetry.iter().any(|telemetry| {
        telemetry.status == ResourceAdmissionStatus::Delayed
            && telemetry.blocking_scope == Some(ResourceScope::AtpProcesses)
    }));

    let mut fanout_budget = ResourceBudget::unbounded();
    fanout_budget.atp_processes = 2;
    fanout_budget.backend_fanout = 1;
    let (fanout_limited, fanout_batches) = run_scheduler_with_dispatch_batches(SchedulerInput {
        graph: graph.clone(),
        worker_count: 4,
        resource_budget: fanout_budget,
        ..SchedulerInput::new(graph.clone())
    })
    .expect("scheduler run succeeds");

    assert_backend_batches_are_serial(&graph, &fanout_batches);
    assert!(fanout_limited.resource_telemetry.iter().any(|telemetry| {
        telemetry.status == ResourceAdmissionStatus::Delayed
            && matches!(
                telemetry.blocking_scope,
                Some(ResourceScope::BackendFanout { .. })
            )
    }));
}

#[test]
fn atp_portfolio_admission_does_not_consume_backend_process_slot() {
    let graph = sample_graph();
    let atp = task_id_for_kind(&graph, TaskKind::AtpSolve);
    let backend = task_id_for_kind(&graph, TaskKind::BackendRun);
    let mut budget = ResourceBudget::unbounded();
    budget.atp_portfolios = 1;
    budget.atp_processes = 0;

    let run = run_scheduler(SchedulerInput {
        graph,
        resource_budget: budget,
        ..SchedulerInput::new(sample_graph())
    })
    .expect("scheduler run succeeds");

    assert_eq!(state_for(&run, &atp), TaskState::Completed);
    assert_eq!(state_for(&run, &backend), TaskState::Blocked);
    assert!(run.resource_telemetry.iter().any(|telemetry| {
        telemetry.task_id == atp && telemetry.status == ResourceAdmissionStatus::Admitted
    }));
    assert!(run.resource_telemetry.iter().any(|telemetry| {
        telemetry.task_id == backend
            && telemetry.status == ResourceAdmissionStatus::Impossible
            && telemetry.blocking_scope == Some(ResourceScope::AtpProcesses)
    }));
}

#[test]
fn admitted_tasks_release_resources_exactly_once() {
    let graph = sample_graph();
    assert_all_admitted_tasks_release_once(
        run_scheduler(SchedulerInput::new(graph.clone())).expect("completed run succeeds"),
    );

    let atp = task_id_for_kind(&graph, TaskKind::AtpSolve);
    assert_all_admitted_tasks_release_once(
        run_scheduler(SchedulerInput {
            graph: graph.clone(),
            task_outcomes: vec![SyntheticTaskOutcome::skip(atp)],
            ..SchedulerInput::new(graph.clone())
        })
        .expect("skipped run succeeds"),
    );

    let frontend = task_id_for_kind(&graph, TaskKind::Frontend);
    assert_all_admitted_tasks_release_once(
        run_scheduler(SchedulerInput {
            graph: graph.clone(),
            task_outcomes: vec![SyntheticTaskOutcome::fail(
                frontend,
                vec![SchedulerDiagnosticRef::new(
                    "frontend",
                    "E001",
                    "frontend failed",
                )],
            )],
            ..SchedulerInput::new(graph)
        })
        .expect("failed run succeeds"),
    );
}

#[test]
fn priority_hints_affect_start_order_but_not_canonical_results() {
    let graph = multi_module_graph();
    let main_source = task_id_for_module_kind(&graph, TaskKind::SourceLoad, "app", "main");
    let util_source = task_id_for_module_kind(&graph, TaskKind::SourceLoad, "app", "util");
    let (hinted, hinted_batches) = run_scheduler_with_dispatch_batches(SchedulerInput {
        graph: graph.clone(),
        worker_count: 1,
        priority_hints: PriorityHints {
            preferred_tasks: vec![util_source.clone()],
        },
        ..SchedulerInput::new(graph.clone())
    })
    .expect("scheduler run succeeds");
    let (default, default_batches) = run_scheduler_with_dispatch_batches(SchedulerInput {
        graph,
        worker_count: 1,
        ..SchedulerInput::new(multi_module_graph())
    })
    .expect("scheduler run succeeds");

    assert_eq!(default_batches[0], vec![main_source]);
    assert_eq!(hinted_batches[0], vec![util_source]);
    assert_eq!(hinted.results, default.results);
    assert_eq!(hinted.task_states, default.task_states);
    assert_eq!(hinted.events, default.events);
}

#[test]
fn backend_and_kernel_completion_order_does_not_change_collation_or_authority() {
    let graph = multi_backend_graph();
    assert!(task_ids_for_kind(&graph, TaskKind::BackendRun).len() >= 2);
    assert!(task_ids_for_kind(&graph, TaskKind::KernelCheck).len() >= 2);

    let (reverse, batches) = run_scheduler_with_dispatch_batches(SchedulerInput {
        graph: graph.clone(),
        completion_order: CompletionOrder::Reverse,
        worker_count: 4,
        ..SchedulerInput::new(graph.clone())
    })
    .expect("scheduler run succeeds");
    let canonical =
        run_scheduler(SchedulerInput::new(graph.clone())).expect("scheduler run succeeds");

    assert_eq!(canonical.results, reverse.results);
    assert!(batches.iter().any(|batch| {
        batch
            .iter()
            .filter(|task_id| task_kind_for_id(&graph, task_id) == TaskKind::BackendRun)
            .count()
            >= 2
    }));
    assert!(batches.iter().any(|batch| {
        batch
            .iter()
            .filter(|task_id| task_kind_for_id(&graph, task_id) == TaskKind::KernelCheck)
            .count()
            >= 2
    }));
    let canonical_proof_results = canonical
        .results
        .iter()
        .filter(|result| {
            matches!(
                task_kind_for_id(&graph, &result.task_id),
                TaskKind::BackendRun | TaskKind::KernelCheck
            )
        })
        .map(|result| result.task_id.clone())
        .collect::<Vec<_>>();
    let reverse_proof_results = reverse
        .results
        .iter()
        .filter(|result| {
            matches!(
                task_kind_for_id(&graph, &result.task_id),
                TaskKind::BackendRun | TaskKind::KernelCheck
            )
        })
        .map(|result| result.task_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(canonical_proof_results, reverse_proof_results);
    let debug = format!("{reverse:#?}").to_lowercase();
    for forbidden in ["proofauthority", "proofacceptance", "trustedstatus"] {
        assert!(!debug.contains(forbidden));
    }
}

#[test]
fn skipped_atp_cascades_over_backend_subgraph_without_authority() {
    let graph = sample_graph();
    let atp = task_id_for_kind(&graph, TaskKind::AtpSolve);
    let backend = task_id_for_kind(&graph, TaskKind::BackendRun);
    let kernel = task_id_for_kind(&graph, TaskKind::KernelCheck);
    let artifact = task_id_for_kind(&graph, TaskKind::ArtifactCommit);
    let documentation = task_id_for_kind(&graph, TaskKind::DocumentationExtract);

    let run = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        task_outcomes: vec![SyntheticTaskOutcome::skip(atp.clone())],
        ..SchedulerInput::new(graph)
    })
    .expect("scheduler run succeeds");

    assert_eq!(state_for(&run, &atp), TaskState::Skipped);
    assert_eq!(state_for(&run, &backend), TaskState::Skipped);
    assert_eq!(state_for(&run, &kernel), TaskState::Skipped);
    assert_eq!(state_for(&run, &artifact), TaskState::Completed);
    assert_eq!(state_for(&run, &documentation), TaskState::Completed);
    assert!(result_for(&run, &backend).output_refs.is_empty());
    let debug = format!("{run:#?}").to_lowercase();
    for forbidden in ["proofauthority", "proofacceptance", "trustedstatus"] {
        assert!(!debug.contains(forbidden));
    }
}

#[test]
fn non_conditional_skips_do_not_unblock_artifacts_or_documentation() {
    let graph = sample_graph();
    let backend = task_id_for_kind(&graph, TaskKind::BackendRun);
    let kernel = task_id_for_kind(&graph, TaskKind::KernelCheck);
    let artifact = task_id_for_kind(&graph, TaskKind::ArtifactCommit);
    let documentation = task_id_for_kind(&graph, TaskKind::DocumentationExtract);

    let run = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        task_outcomes: vec![SyntheticTaskOutcome::skip(backend.clone())],
        ..SchedulerInput::new(graph)
    })
    .expect("scheduler run succeeds");

    assert_eq!(state_for(&run, &backend), TaskState::Skipped);
    assert_eq!(state_for(&run, &kernel), TaskState::Blocked);
    assert_eq!(state_for(&run, &artifact), TaskState::Blocked);
    assert_eq!(state_for(&run, &documentation), TaskState::Blocked);
    assert!(run.blocked_records.iter().any(|record| {
        record.task_id == kernel
            && record.reason == BlockReason::NoSchedulablePath
            && record.blocked_by.is_empty()
    }));

    let graph = sample_graph();
    let artifact = task_id_for_kind(&graph, TaskKind::ArtifactCommit);
    let documentation = task_id_for_kind(&graph, TaskKind::DocumentationExtract);
    let run = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        task_outcomes: vec![SyntheticTaskOutcome::skip(artifact.clone())],
        ..SchedulerInput::new(graph)
    })
    .expect("scheduler run succeeds");

    assert_eq!(state_for(&run, &artifact), TaskState::Skipped);
    assert_eq!(state_for(&run, &documentation), TaskState::Blocked);
}

#[test]
fn state_records_match_build_task_dependencies() {
    let graph = sample_graph();
    let dependencies = graph
        .tasks
        .iter()
        .map(|task| (task.id.clone(), task.dependencies.clone()))
        .collect::<BTreeMap<_, _>>();
    let run = run_scheduler(SchedulerInput::new(graph)).expect("scheduler run succeeds");

    for record in &run.task_states {
        assert_eq!(
            record.dependencies,
            dependencies
                .get(&record.task_id)
                .expect("state record task exists")
                .clone()
        );
    }
}

#[test]
fn disabled_cache_seam_never_produces_cache_hit_or_cache_identity() {
    for cache in [
        CacheSchedulingPolicy::Disabled,
        CacheSchedulingPolicy::Miss,
        CacheSchedulingPolicy::Unavailable,
        CacheSchedulingPolicy::ErrorAsMiss,
    ] {
        let graph = sample_graph();
        let run = run_scheduler(SchedulerInput {
            graph: graph.clone(),
            cache,
            ..SchedulerInput::new(graph)
        })
        .expect("scheduler run succeeds");
        assert!(
            run.task_states
                .iter()
                .all(|record| record.state != TaskState::CacheHit)
        );
        assert!(
            run.failure_records.is_empty(),
            "cache miss/unavailable/error-as-miss must not become failure"
        );
        let debug = format!("{run:#?}").to_lowercase();
        assert!(!debug.contains("cachekey"));
        assert!(!debug.contains("dependencyfingerprint"));
        assert!(!debug.contains("proofreuse"));
    }
}

#[test]
fn validated_cache_hit_skips_execution_and_publishes_clean_equivalent_outputs() {
    let graph = sample_graph();
    let frontend = task_id_for_kind(&graph, TaskKind::Frontend);
    let clean_output = SyntheticOutputRef::new("frontend-summary", "clean");
    let clean_diagnostic = SchedulerDiagnosticRef::new("frontend", "W001", "clean warning");
    let clean = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        task_outcomes: vec![SyntheticTaskOutcome {
            task_id: frontend.clone(),
            status: SyntheticTaskStatus::Complete,
            outputs: vec![clean_output.clone()],
            diagnostics: vec![clean_diagnostic.clone()],
        }],
        ..SchedulerInput::new(graph.clone())
    })
    .expect("clean run succeeds");

    let hit = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cache: CacheSchedulingPolicy::Enabled,
        cache_decisions: CacheSchedulingPlan::new(vec![CacheTaskDecision::new(
            frontend.clone(),
            CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
                vec![
                    CacheOutputRef::new("frontend-summary", "clean"),
                    CacheOutputRef::new("frontend-summary", "clean"),
                ],
                vec![CacheDiagnosticRef::new("frontend", "W001", "clean warning")],
            )),
        )]),
        task_outcomes: vec![SyntheticTaskOutcome::fail(
            frontend.clone(),
            vec![SchedulerDiagnosticRef::new(
                "frontend",
                "E999",
                "would fail if executed",
            )],
        )],
        ..SchedulerInput::new(graph.clone())
    })
    .expect("cache-hit run succeeds");

    assert_eq!(state_for(&hit, &frontend), TaskState::CacheHit);
    assert_eq!(
        result_for(&hit, &frontend).output_refs,
        result_for(&clean, &frontend).output_refs
    );
    assert_eq!(
        result_for(&hit, &frontend).diagnostics,
        result_for(&clean, &frontend).diagnostics
    );
    assert!(hit.failure_records.is_empty());
    assert!(hit.events.iter().all(|event| {
        event.task_id.as_ref() != Some(&frontend) || event.kind != SchedulerEventKind::TaskStarted
    }));
    assert!(
        hit.resource_telemetry
            .iter()
            .all(|telemetry| telemetry.task_id != frontend)
    );
    assert_eq!(
        state_for(&hit, &task_id_for_kind(&graph, TaskKind::ArtifactCommit)),
        TaskState::Completed
    );
}

#[test]
fn cache_hit_result_canonicalizes_public_payload_fields() {
    let graph = sample_graph();
    let frontend = task_id_for_kind(&graph, TaskKind::Frontend);
    let run = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cache: CacheSchedulingPolicy::Enabled,
        cache_decisions: CacheSchedulingPlan::new(vec![CacheTaskDecision::new(
            frontend.clone(),
            CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit {
                output_refs: vec![
                    CacheOutputRef::new("b", "second"),
                    CacheOutputRef::new("a", "first"),
                    CacheOutputRef::new("a", "first"),
                ],
                diagnostics: vec![
                    CacheDiagnosticRef::new("frontend", "W002", "later"),
                    CacheDiagnosticRef::new("frontend", "W001", "earlier"),
                    CacheDiagnosticRef::new("frontend", "W001", "earlier"),
                ],
            }),
        )]),
        ..SchedulerInput::new(graph)
    })
    .expect("cache-hit run succeeds");

    assert_eq!(
        result_for(&run, &frontend).output_refs,
        vec![
            SyntheticOutputRef::new("a", "first"),
            SyntheticOutputRef::new("b", "second"),
        ]
    );
    assert_eq!(
        result_for(&run, &frontend).diagnostics,
        vec![
            SchedulerDiagnosticRef::new("frontend", "W001", "earlier"),
            SchedulerDiagnosticRef::new("frontend", "W002", "later"),
        ]
    );
}

#[test]
fn obsolete_cache_hit_is_cancelled_without_publication_and_blocks_dependents() {
    let graph = sample_graph();
    let frontend = task_id_for_kind(&graph, TaskKind::Frontend);
    let artifact = task_id_for_kind(&graph, TaskKind::ArtifactCommit);
    let run = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cache: CacheSchedulingPolicy::Enabled,
        cache_decisions: CacheSchedulingPlan::new(vec![CacheTaskDecision::new(
            frontend.clone(),
            CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
                vec![CacheOutputRef::new("frontend", "stale cached output")],
                vec![CacheDiagnosticRef::new(
                    "frontend",
                    "W001",
                    "stale cached diagnostic",
                )],
            )),
        )]),
        cancellation: CancellationPolicy::default().with_obsolete_completed_task(frontend.clone()),
        ..SchedulerInput::new(graph)
    })
    .expect("obsolete cache-hit run succeeds");

    assert_eq!(state_for(&run, &frontend), TaskState::Cancelled);
    let result = result_for(&run, &frontend);
    assert_eq!(result.state, TaskState::Cancelled);
    assert!(result.output_refs.is_empty());
    assert!(result.diagnostics.is_empty());
    assert_eq!(state_for(&run, &artifact), TaskState::Blocked);
    assert!(run.blocked_records.iter().any(|record| {
        record.blocked_by.contains(&frontend) && record.reason == BlockReason::DependencyCancelled
    }));
}

#[test]
fn cache_fallback_decisions_execute_normally() {
    for outcome in [
        CacheSchedulingOutcome::Miss(CacheFallbackReason::Miss),
        CacheSchedulingOutcome::NoKey(CacheFallbackReason::NoKey),
        CacheSchedulingOutcome::Unavailable(CacheFallbackReason::Unavailable),
        CacheSchedulingOutcome::ErrorAsMiss(CacheFallbackReason::Error),
    ] {
        let graph = sample_graph();
        let frontend = task_id_for_kind(&graph, TaskKind::Frontend);
        let run = run_scheduler(SchedulerInput {
            graph: graph.clone(),
            cache: CacheSchedulingPolicy::Enabled,
            cache_decisions: CacheSchedulingPlan::new(vec![CacheTaskDecision::new(
                frontend.clone(),
                outcome,
            )]),
            task_outcomes: vec![SyntheticTaskOutcome::complete(
                frontend.clone(),
                vec![SyntheticOutputRef::new("frontend", "executed")],
            )],
            ..SchedulerInput::new(graph)
        })
        .expect("fallback run succeeds");

        assert_eq!(state_for(&run, &frontend), TaskState::Completed);
        assert_eq!(
            result_for(&run, &frontend).output_refs,
            vec![SyntheticOutputRef::new("frontend", "executed")]
        );
        assert!(
            run.events.iter().any(|event| {
                event.task_id.as_ref() == Some(&frontend)
                    && event.kind == SchedulerEventKind::TaskStarted
            }),
            "fallback decision must execute the task"
        );
    }
}

#[test]
fn disabled_cache_scheduling_ignores_validated_hits() {
    let graph = sample_graph();
    let frontend = task_id_for_kind(&graph, TaskKind::Frontend);
    let run = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cache: CacheSchedulingPolicy::Disabled,
        cache_decisions: CacheSchedulingPlan::new(vec![CacheTaskDecision::new(
            frontend.clone(),
            CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
                vec![CacheOutputRef::new("frontend", "cached")],
                Vec::new(),
            )),
        )]),
        task_outcomes: vec![SyntheticTaskOutcome::complete(
            frontend.clone(),
            vec![SyntheticOutputRef::new("frontend", "executed")],
        )],
        ..SchedulerInput::new(graph)
    })
    .expect("disabled cache run succeeds");

    assert_eq!(state_for(&run, &frontend), TaskState::Completed);
    assert_eq!(
        result_for(&run, &frontend).output_refs,
        vec![SyntheticOutputRef::new("frontend", "executed")]
    );
    assert!(
        run.task_states
            .iter()
            .all(|record| { record.task_id != frontend || record.state != TaskState::CacheHit })
    );
}

#[test]
fn cache_decision_boundary_rejects_duplicate_and_unknown_tasks() {
    let graph = sample_graph();
    let frontend = task_id_for_kind(&graph, TaskKind::Frontend);
    let unknown = TaskId::new_for_test("unknown-cache-decision-task");
    let diagnostics = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cache: CacheSchedulingPolicy::Enabled,
        cache_decisions: CacheSchedulingPlan::new(vec![
            CacheTaskDecision::new(
                frontend.clone(),
                CacheSchedulingOutcome::Miss(CacheFallbackReason::Miss),
            ),
            CacheTaskDecision::new(
                frontend.clone(),
                CacheSchedulingOutcome::NoKey(CacheFallbackReason::NoKey),
            ),
            CacheTaskDecision::new(
                unknown.clone(),
                CacheSchedulingOutcome::Unavailable(CacheFallbackReason::Unavailable),
            ),
        ]),
        ..SchedulerInput::new(graph)
    })
    .expect_err("invalid cache decision plan is rejected");

    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        diagnostic.task_id.as_ref() == Some(&frontend)
            && diagnostic.kind == SchedulerDiagnosticKind::DuplicateCacheDecision
    }));
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        diagnostic.task_id.as_ref() == Some(&unknown)
            && diagnostic.kind == SchedulerDiagnosticKind::UnknownCacheDecision
    }));
}

#[test]
fn cache_hit_collation_is_deterministic_across_worker_order() {
    let graph = multi_module_graph();
    let main = task_id_for_module_kind(&graph, TaskKind::Frontend, "app", "main");
    let util = task_id_for_module_kind(&graph, TaskKind::Frontend, "app", "util");
    let cache_decisions = CacheSchedulingPlan::new(vec![
        CacheTaskDecision::new(
            util.clone(),
            CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
                vec![CacheOutputRef::new("util", "cached")],
                Vec::new(),
            )),
        ),
        CacheTaskDecision::new(
            main.clone(),
            CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
                vec![CacheOutputRef::new("main", "cached")],
                Vec::new(),
            )),
        ),
    ]);
    let canonical = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cache: CacheSchedulingPolicy::Enabled,
        cache_decisions: cache_decisions.clone(),
        worker_count: 1,
        completion_order: CompletionOrder::Canonical,
        ..SchedulerInput::new(graph.clone())
    })
    .expect("canonical cache-hit run succeeds");
    let reverse = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cache: CacheSchedulingPolicy::Enabled,
        cache_decisions,
        worker_count: 4,
        completion_order: CompletionOrder::Reverse,
        ..SchedulerInput::new(graph)
    })
    .expect("reverse cache-hit run succeeds");

    assert_eq!(canonical.task_states, reverse.task_states);
    assert_eq!(canonical.results, reverse.results);
    assert_eq!(canonical.events, reverse.events);
    assert_eq!(canonical.resource_telemetry, reverse.resource_telemetry);
    assert_eq!(state_for(&canonical, &main), TaskState::CacheHit);
    assert_eq!(state_for(&canonical, &util), TaskState::CacheHit);
}

#[test]
fn failed_and_cancelled_dependencies_block_dependents_boundedly() {
    let graph = multi_module_graph();
    let frontend = task_id_for_module_kind(&graph, TaskKind::Frontend, "app", "main");
    let util_artifact = task_id_for_module_kind(&graph, TaskKind::ArtifactCommit, "app", "util");
    let run = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        task_outcomes: vec![SyntheticTaskOutcome::fail(
            frontend.clone(),
            vec![SchedulerDiagnosticRef::new(
                "main",
                "E001",
                "frontend failed",
            )],
        )],
        ..SchedulerInput::new(graph.clone())
    })
    .expect("scheduler run succeeds");

    assert_eq!(state_for(&run, &frontend), TaskState::Failed);
    assert_eq!(run.failure_records.len(), 1);
    assert_eq!(&run.failure_records[0].task_id, &frontend);
    assert_eq!(run.failure_records[0].category, FailureCategory::ParseError);
    assert_eq!(run.failure_records[0].diagnostic_code, "E001");
    let failed_result = result_for(&run, &frontend);
    assert!(failed_result.output_refs.is_empty());
    assert_eq!(
        failed_result.diagnostics,
        vec![SchedulerDiagnosticRef::new(
            "main",
            "E001",
            "frontend failed"
        )]
    );
    let blocked_by_frontend = run
        .task_states
        .iter()
        .filter(|record| {
            record.state == TaskState::Blocked && record.blocked_by.contains(&frontend)
        })
        .collect::<Vec<_>>();
    assert!(!blocked_by_frontend.is_empty());
    assert!(
        blocked_by_frontend
            .iter()
            .all(|record| { task_belongs_to_module(&graph, &record.task_id, "app", "main") })
    );
    assert_eq!(state_for(&run, &util_artifact), TaskState::Completed);
    assert!(run.blocked_records.iter().any(|record| {
        record.blocked_by == vec![frontend.clone()]
            && record.reason == BlockReason::DependencyFailed
    }));
    assert!(run.blocked_records.iter().any(|record| {
        record.reason == BlockReason::DependencyBlocked
            && record
                .blocked_by
                .iter()
                .all(|blocked_by| blocked_by != &frontend)
    }));

    let graph = multi_module_graph();
    let source = task_id_for_module_kind(&graph, TaskKind::SourceLoad, "app", "main");
    let frontend = task_id_for_module_kind(&graph, TaskKind::Frontend, "app", "main");
    let util_artifact = task_id_for_module_kind(&graph, TaskKind::ArtifactCommit, "app", "util");
    let run = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cancellation: CancellationPolicy::default().with_cancelled_task(source.clone()),
        task_outcomes: vec![SyntheticTaskOutcome {
            task_id: frontend.clone(),
            status: SyntheticTaskStatus::Complete,
            outputs: vec![SyntheticOutputRef::new("frontend", "should not publish")],
            diagnostics: vec![SchedulerDiagnosticRef::new(
                "main",
                "E002",
                "blocked frontend did not run",
            )],
        }],
        ..SchedulerInput::new(graph.clone())
    })
    .expect("scheduler run succeeds");
    assert_eq!(state_for(&run, &source), TaskState::Cancelled);
    assert!(
        run.failure_records.is_empty(),
        "cancellation must not create failure records"
    );
    let blocked_by_source = run
        .task_states
        .iter()
        .filter(|record| record.state == TaskState::Blocked && record.blocked_by.contains(&source))
        .collect::<Vec<_>>();
    assert!(!blocked_by_source.is_empty());
    assert!(
        blocked_by_source
            .iter()
            .all(|record| { task_belongs_to_module(&graph, &record.task_id, "app", "main") })
    );
    assert_eq!(state_for(&run, &util_artifact), TaskState::Completed);
    assert!(run.blocked_records.iter().any(|record| {
        record.blocked_by == vec![source.clone()]
            && record.reason == BlockReason::DependencyCancelled
    }));
    let blocked_frontend = result_for(&run, &frontend);
    assert!(blocked_frontend.output_refs.is_empty());
    assert!(blocked_frontend.diagnostics.is_empty());
}

#[test]
fn failure_records_are_deterministic_and_independent_failures_remain_visible() {
    let graph = multi_module_graph();
    let main = task_id_for_module_kind(&graph, TaskKind::Frontend, "app", "main");
    let util = task_id_for_module_kind(&graph, TaskKind::Frontend, "app", "util");
    let outcomes = vec![
        SyntheticTaskOutcome::fail(
            util.clone(),
            vec![SchedulerDiagnosticRef::new("util", "E020", "util failed")],
        ),
        SyntheticTaskOutcome::fail(
            main.clone(),
            vec![SchedulerDiagnosticRef::new("main", "E010", "main failed")],
        ),
    ];
    let canonical = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        task_outcomes: outcomes.clone(),
        worker_count: 1,
        ..SchedulerInput::new(graph.clone())
    })
    .expect("scheduler run succeeds");
    let reverse = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        task_outcomes: outcomes,
        worker_count: 3,
        completion_order: CompletionOrder::Reverse,
        ..SchedulerInput::new(graph)
    })
    .expect("scheduler run succeeds");

    assert_eq!(canonical.failure_records, reverse.failure_records);
    assert_eq!(canonical.blocked_records, reverse.blocked_records);
    assert_eq!(
        canonical
            .failure_records
            .iter()
            .map(|record| record.diagnostic_code.as_str())
            .collect::<Vec<_>>(),
        vec!["E010", "E020"]
    );
    assert_eq!(state_for(&canonical, &main), TaskState::Failed);
    assert_eq!(state_for(&canonical, &util), TaskState::Failed);
    assert!(
        canonical
            .blocked_records
            .iter()
            .all(|record| record.blocked_by.len() <= 1)
    );
}

#[test]
fn multiple_failed_predecessors_have_completion_order_independent_blockers() {
    let graph = diamond_failure_graph(DependencyCoverage::Complete);
    let left = TaskId::new_for_test("left");
    let right = TaskId::new_for_test("right");
    let join = TaskId::new_for_test("join");
    let outcomes = vec![
        SyntheticTaskOutcome::fail(
            left.clone(),
            vec![SchedulerDiagnosticRef::new("left", "E010", "left failed")],
        ),
        SyntheticTaskOutcome::fail(
            right.clone(),
            vec![SchedulerDiagnosticRef::new("right", "E020", "right failed")],
        ),
    ];
    let canonical = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        worker_count: 2,
        task_outcomes: outcomes.clone(),
        ..SchedulerInput::new(graph.clone())
    })
    .expect("scheduler run succeeds");
    let reverse = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        worker_count: 2,
        completion_order: CompletionOrder::Reverse,
        task_outcomes: outcomes,
        ..SchedulerInput::new(graph)
    })
    .expect("scheduler run succeeds");

    let canonical_block = blocked_record_for(&canonical, &join);
    let reverse_block = blocked_record_for(&reverse, &join);
    assert_eq!(canonical_block, reverse_block);
    assert_eq!(canonical_block.reason, BlockReason::DependencyFailed);
    assert_eq!(canonical_block.blocked_by, vec![left, right]);
}

#[test]
fn direct_scheduler_blocks_propagate_before_no_schedulable_fallback() {
    let graph = diamond_failure_graph(DependencyCoverage::MissingModuleDependencyOverlay);
    let left = TaskId::new_for_test("left");
    let join = TaskId::new_for_test("join");

    let run = run_scheduler(SchedulerInput::new(graph)).expect("scheduler run succeeds");

    let left_block = blocked_record_for(&run, &left);
    assert_eq!(left_block.reason, BlockReason::MissingDependencyCoverage);
    assert!(left_block.blocked_by.is_empty());
    let join_block = blocked_record_for(&run, &join);
    assert_eq!(join_block.reason, BlockReason::DependencyBlocked);
    assert_eq!(join_block.blocked_by, vec![left]);
}

#[test]
fn dependency_terminal_blocks_propagate_before_no_schedulable_fallback() {
    let root = TaskId::new_for_test("root");
    let left = TaskId::new_for_test("left");
    let right = TaskId::new_for_test("right");
    let join = TaskId::new_for_test("join");
    let tail = TaskId::new_for_test("tail");
    let graph = TaskGraph {
        version: crate::task_graph::TaskGraphVersion::current(),
        snapshot: snapshot(6),
        tasks: vec![
            scheduler_test_task(
                root.clone(),
                TaskKind::PackageResolve,
                Vec::new(),
                DependencyCoverage::Complete,
                PriorityClass::Root,
            ),
            scheduler_test_task(
                tail.clone(),
                TaskKind::DocumentationExtract,
                vec![join.clone()],
                DependencyCoverage::Complete,
                PriorityClass::Commit,
            ),
            scheduler_test_task(
                join.clone(),
                TaskKind::ArtifactCommit,
                vec![left.clone(), right.clone()],
                DependencyCoverage::Complete,
                PriorityClass::Commit,
            ),
            scheduler_test_task(
                left.clone(),
                TaskKind::Frontend,
                vec![root.clone()],
                DependencyCoverage::Complete,
                PriorityClass::Source,
            ),
            scheduler_test_task(
                right.clone(),
                TaskKind::ModuleResolve,
                vec![root],
                DependencyCoverage::Complete,
                PriorityClass::Source,
            ),
        ],
        edges: Vec::new(),
        diagnostics: Vec::new(),
    };

    let run = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cancellation: CancellationPolicy::default()
            .with_cancelled_task(left.clone())
            .with_cancelled_task(right.clone()),
        ..SchedulerInput::new(graph)
    })
    .expect("scheduler run succeeds");

    let join_block = blocked_record_for(&run, &join);
    assert_eq!(join_block.reason, BlockReason::DependencyCancelled);
    assert_eq!(join_block.blocked_by, vec![left, right]);
    let tail_block = blocked_record_for(&run, &tail);
    assert_eq!(tail_block.reason, BlockReason::DependencyBlocked);
    assert_eq!(tail_block.blocked_by, vec![join]);
}

#[test]
fn synthetic_cancellation_prevents_current_publication() {
    let graph = sample_graph();
    let artifact = task_id_for_kind(&graph, TaskKind::ArtifactCommit);
    let run = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cancellation: CancellationPolicy::default().with_cancelled_task(artifact.clone()),
        task_outcomes: vec![SyntheticTaskOutcome {
            task_id: artifact.clone(),
            status: SyntheticTaskStatus::Complete,
            outputs: vec![SyntheticOutputRef::new("artifact", "should not publish")],
            diagnostics: vec![SchedulerDiagnosticRef::new(
                "artifact",
                "W001",
                "should not publish",
            )],
        }],
        ..SchedulerInput::new(graph)
    })
    .expect("scheduler run succeeds");

    let result = result_for(&run, &artifact);
    assert_eq!(result.state, TaskState::Cancelled);
    assert!(result.output_refs.is_empty());
    assert!(result.diagnostics.is_empty());
}

#[test]
fn root_cancellation_prevents_current_publication_and_blocks_dependents() {
    let graph = sample_graph();
    let root = task_id_for_kind(&graph, TaskKind::PackageResolve);
    let source = task_id_for_kind(&graph, TaskKind::SourceLoad);
    let run = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cancellation: CancellationPolicy::default().with_cancelled_task(root.clone()),
        task_outcomes: vec![SyntheticTaskOutcome {
            task_id: root.clone(),
            status: SyntheticTaskStatus::Complete,
            outputs: vec![SyntheticOutputRef::new("root", "should not publish")],
            diagnostics: vec![SchedulerDiagnosticRef::new(
                "root",
                "W001",
                "should not publish",
            )],
        }],
        ..SchedulerInput::new(graph)
    })
    .expect("scheduler run succeeds");

    let result = result_for(&run, &root);
    assert_eq!(result.state, TaskState::Cancelled);
    assert!(result.output_refs.is_empty());
    assert!(result.diagnostics.is_empty());
    assert_eq!(state_for(&run, &source), TaskState::Blocked);
}

#[test]
fn ready_cancellation_prevents_start_and_current_publication() {
    let graph = sample_graph();
    let frontend = task_id_for_kind(&graph, TaskKind::Frontend);
    let artifact = task_id_for_kind(&graph, TaskKind::ArtifactCommit);
    let run = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cancellation: CancellationPolicy::default().with_ready_cancelled_task(frontend.clone()),
        task_outcomes: vec![SyntheticTaskOutcome {
            task_id: frontend.clone(),
            status: SyntheticTaskStatus::Complete,
            outputs: vec![SyntheticOutputRef::new("frontend", "should not publish")],
            diagnostics: vec![SchedulerDiagnosticRef::new(
                "frontend",
                "W001",
                "should not publish",
            )],
        }],
        ..SchedulerInput::new(graph)
    })
    .expect("scheduler run succeeds");

    let result = result_for(&run, &frontend);
    assert_eq!(result.state, TaskState::Cancelled);
    assert!(result.output_refs.is_empty());
    assert!(result.diagnostics.is_empty());
    assert_eq!(state_for(&run, &artifact), TaskState::Blocked);
    assert!(!run.events.iter().any(|event| {
        event.kind == SchedulerEventKind::TaskStarted && event.task_id.as_ref() == Some(&frontend)
    }));
}

#[test]
fn running_checkpoint_cancellation_releases_resources_once() {
    let graph = sample_graph();
    let frontend = task_id_for_kind(&graph, TaskKind::Frontend);
    let artifact = task_id_for_kind(&graph, TaskKind::ArtifactCommit);
    let run = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cancellation: CancellationPolicy::default()
            .with_checkpoint_cancelled_task(frontend.clone()),
        task_outcomes: vec![SyntheticTaskOutcome {
            task_id: frontend.clone(),
            status: SyntheticTaskStatus::Complete,
            outputs: vec![SyntheticOutputRef::new("frontend", "should not publish")],
            diagnostics: vec![SchedulerDiagnosticRef::new(
                "frontend",
                "W002",
                "should not publish",
            )],
        }],
        ..SchedulerInput::new(graph)
    })
    .expect("scheduler run succeeds");

    let result = result_for(&run, &frontend);
    assert_eq!(result.state, TaskState::Cancelled);
    assert!(result.output_refs.is_empty());
    assert!(result.diagnostics.is_empty());
    assert_eq!(state_for(&run, &artifact), TaskState::Blocked);
    assert!(run.events.iter().any(|event| {
        event.kind == SchedulerEventKind::TaskStarted && event.task_id.as_ref() == Some(&frontend)
    }));
    assert_eq!(
        telemetry_counts(&run, ResourceAdmissionStatus::Admitted).get(&frontend),
        Some(&1)
    );
    assert_eq!(
        telemetry_counts(&run, ResourceAdmissionStatus::Released).get(&frontend),
        Some(&1)
    );
}

#[test]
fn obsolete_completed_result_is_discarded_before_publication() {
    let graph = sample_graph();
    let artifact = task_id_for_kind(&graph, TaskKind::ArtifactCommit);
    let documentation = task_id_for_kind(&graph, TaskKind::DocumentationExtract);
    let run = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cancellation: CancellationPolicy::default().with_obsolete_completed_task(artifact.clone()),
        task_outcomes: vec![SyntheticTaskOutcome {
            task_id: artifact.clone(),
            status: SyntheticTaskStatus::Complete,
            outputs: vec![SyntheticOutputRef::new("artifact", "should not publish")],
            diagnostics: vec![SchedulerDiagnosticRef::new(
                "artifact",
                "W003",
                "should not publish",
            )],
        }],
        ..SchedulerInput::new(graph)
    })
    .expect("scheduler run succeeds");

    let result = result_for(&run, &artifact);
    assert_eq!(result.state, TaskState::Cancelled);
    assert!(result.output_refs.is_empty());
    assert!(result.diagnostics.is_empty());
    assert_eq!(state_for(&run, &documentation), TaskState::Blocked);
}

#[test]
fn current_snapshot_mismatch_cancels_snapshot_before_start() {
    let graph = sample_graph();
    let current_snapshot = snapshot(99);
    let source = task_id_for_kind(&graph, TaskKind::SourceLoad);
    let frontend = task_id_for_kind(&graph, TaskKind::Frontend);
    let run = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cancellation: CancellationPolicy::default().with_current_snapshot(current_snapshot),
        task_outcomes: vec![SyntheticTaskOutcome {
            task_id: source.clone(),
            status: SyntheticTaskStatus::Complete,
            outputs: vec![SyntheticOutputRef::new("source", "should not publish")],
            diagnostics: vec![SchedulerDiagnosticRef::new(
                "source",
                "W004",
                "should not publish",
            )],
        }],
        ..SchedulerInput::new(graph)
    })
    .expect("scheduler run succeeds");

    let result = result_for(&run, &source);
    assert_eq!(result.state, TaskState::Cancelled);
    assert!(result.output_refs.is_empty());
    assert!(result.diagnostics.is_empty());
    assert_eq!(state_for(&run, &frontend), TaskState::Cancelled);
    assert!(
        !run.events
            .iter()
            .any(|event| { event.kind == SchedulerEventKind::TaskStarted })
    );
}

#[test]
fn commit_boundary_cancels_before_start_but_not_after_transaction_start() {
    let graph = sample_graph();
    let artifact = task_id_for_kind(&graph, TaskKind::ArtifactCommit);
    let before_start = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cancellation: CancellationPolicy::default().with_cancelled_task(artifact.clone()),
        task_outcomes: vec![SyntheticTaskOutcome::complete(
            artifact.clone(),
            vec![SyntheticOutputRef::new("artifact", "should not publish")],
        )],
        ..SchedulerInput::new(graph.clone())
    })
    .expect("scheduler run succeeds");
    assert_eq!(state_for(&before_start, &artifact), TaskState::Cancelled);
    assert!(result_for(&before_start, &artifact).output_refs.is_empty());
    assert!(!before_start.events.iter().any(|event| {
        event.kind == SchedulerEventKind::TaskStarted && event.task_id.as_ref() == Some(&artifact)
    }));

    let after_start = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        cancellation: CancellationPolicy::default()
            .with_cancelled_task(artifact.clone())
            .with_commit_started_task(artifact.clone()),
        task_outcomes: vec![SyntheticTaskOutcome::complete(
            artifact.clone(),
            vec![SyntheticOutputRef::new(
                "artifact",
                "modeled committed output",
            )],
        )],
        ..SchedulerInput::new(graph)
    })
    .expect("scheduler run succeeds");
    let result = result_for(&after_start, &artifact);
    assert_eq!(result.state, TaskState::Completed);
    assert_eq!(
        result.output_refs,
        vec![SyntheticOutputRef::new(
            "artifact",
            "modeled committed output"
        )]
    );
    let debug = format!("{after_start:#?}").to_lowercase();
    assert!(!debug.contains("publicationtoken"));
    assert!(!debug.contains("trustedstatus"));
}

#[test]
fn cancellation_is_deterministic_across_workers_and_completion_order() {
    let graph = multi_module_graph();
    let frontend = task_id_for_module_kind(&graph, TaskKind::Frontend, "app", "main");
    let cancellation =
        CancellationPolicy::default().with_checkpoint_cancelled_task(frontend.clone());
    let canonical = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        worker_count: 1,
        cancellation: cancellation.clone(),
        ..SchedulerInput::new(graph.clone())
    })
    .expect("scheduler run succeeds");
    let reverse = run_scheduler(SchedulerInput {
        graph: graph.clone(),
        worker_count: 3,
        completion_order: CompletionOrder::Reverse,
        cancellation,
        ..SchedulerInput::new(graph)
    })
    .expect("scheduler run succeeds");

    assert_eq!(canonical.task_states, reverse.task_states);
    assert_eq!(canonical.results, reverse.results);
    assert_eq!(canonical.events, reverse.events);
    assert_eq!(canonical.diagnostics, reverse.diagnostics);
    assert_eq!(canonical.resource_telemetry, reverse.resource_telemetry);
}

#[test]
fn malformed_graph_without_root_returns_scheduler_diagnostics() {
    let mut graph = sample_graph();
    graph
        .tasks
        .retain(|task| task.kind != TaskKind::PackageResolve);
    graph
        .edges
        .retain(|edge| graph.tasks.iter().any(|task| task.id == edge.dependent));

    let diagnostics =
        run_scheduler(SchedulerInput::new(graph)).expect_err("missing root is invalid");
    assert!(
        diagnostics
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.kind == SchedulerDiagnosticKind::MissingRootTask })
    );
}

#[test]
fn scheduler_boundary_excludes_driver_ir_cache_and_publication_authority() {
    let manifest = include_str!("../../Cargo.toml");
    assert!(!manifest.contains("mizar-driver"));
    assert!(!manifest.contains("mizar-ir"));
    assert!(!manifest.contains("mizar-cache"));
    let source = include_str!("../scheduler.rs");
    let resource_source = include_str!("../resource.rs");
    let cancel_source = include_str!("../cancel.rs");
    let failure_source = include_str!("../failure_state.rs");
    let cache_seam_source = include_str!("../cache_seam.rs");
    for forbidden in [
        concat!("Cache", "Key"),
        concat!("Dependency", "Fingerprint"),
        concat!("Proof", "Reuse"),
        concat!("Publication", "Token"),
        concat!("Proof", "Authority"),
        concat!("Trusted", "Status"),
        concat!("std", "::process"),
        concat!("process", "::Command"),
        concat!("Command", "::new"),
        concat!("Driver", "Request"),
        concat!("Driver", "Session"),
        concat!("Ir", "Snapshot", "Handle"),
        concat!("Artifact", "Commit", "Token"),
        concat!("Proof", "Trust"),
        concat!("Cache", "Lookup"),
        concat!("mizar", "-", "diagnostics"),
        concat!("Diagnostic", "Registry"),
        concat!("Failure", "Snapshot", "Store"),
    ] {
        assert!(
            !source.contains(forbidden),
            "{forbidden} leaked into scheduler source"
        );
        assert!(
            !resource_source.contains(forbidden),
            "{forbidden} leaked into resource source"
        );
        assert!(
            !cancel_source.contains(forbidden),
            "{forbidden} leaked into cancel source"
        );
        assert!(
            !failure_source.contains(forbidden),
            "{forbidden} leaked into failure-state source"
        );
        assert!(
            !cache_seam_source.contains(forbidden),
            "{forbidden} leaked into cache seam source"
        );
    }
    for forbidden in [concat!("mizar", "_cache"), concat!("mizar", "-", "cache")] {
        assert!(
            !cache_seam_source.contains(forbidden),
            "{forbidden} leaked into cache seam source"
        );
    }

    let run = run_scheduler(SchedulerInput::new(sample_graph())).expect("scheduler run succeeds");
    let debug = format!("{run:#?}").to_lowercase();
    for forbidden in [
        "cachekey",
        "dependencyfingerprint",
        "proofreuse",
        "publicationtoken",
        "proofauthority",
        "trustedstatus",
    ] {
        assert!(
            !debug.contains(forbidden),
            "{forbidden} leaked into scheduler state"
        );
    }
}

fn sample_graph() -> TaskGraph {
    build_task_graph(TaskGraphInput {
        graph_version: crate::task_graph::TaskGraphVersion::current(),
        snapshot: snapshot(1),
        build_plan: build_plan(vec![workspace_package("app")]),
        module_index: module_index(vec![workspace_module("app", "main")]),
        dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
        vc_descriptors: vec![vc_descriptor(
            "vc-main",
            module_id("app", "main"),
            "001",
            vec!["vampire"],
            vec!["kernel-candidate"],
        )],
        profile: TaskGraphProfile {
            documentation: DocumentationProfile::Enabled,
            ..TaskGraphProfile::default()
        },
    })
    .expect("sample graph builds")
}

fn multi_module_graph() -> TaskGraph {
    build_task_graph(TaskGraphInput {
        graph_version: crate::task_graph::TaskGraphVersion::current(),
        snapshot: snapshot(3),
        build_plan: build_plan(vec![workspace_package("app")]),
        module_index: module_index(vec![
            workspace_module("app", "main"),
            workspace_module("app", "util"),
        ]),
        dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
        vc_descriptors: Vec::new(),
        profile: TaskGraphProfile::default(),
    })
    .expect("multi-module graph builds")
}

fn multi_backend_graph() -> TaskGraph {
    build_task_graph(TaskGraphInput {
        graph_version: crate::task_graph::TaskGraphVersion::current(),
        snapshot: snapshot(4),
        build_plan: build_plan(vec![workspace_package("app")]),
        module_index: module_index(vec![workspace_module("app", "main")]),
        dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
        vc_descriptors: vec![vc_descriptor(
            "vc-main",
            module_id("app", "main"),
            "001",
            vec!["vampire", "eprover"],
            vec!["kernel-a", "kernel-b"],
        )],
        profile: TaskGraphProfile::default(),
    })
    .expect("multi-backend graph builds")
}

fn graph_with_overlay(
    dependency_overlay: ModuleDependencyOverlay,
) -> Result<TaskGraph, crate::task_graph::TaskGraphDiagnostics> {
    build_task_graph(TaskGraphInput {
        graph_version: crate::task_graph::TaskGraphVersion::current(),
        snapshot: snapshot(2),
        build_plan: build_plan(vec![workspace_package("app")]),
        module_index: module_index(vec![workspace_module("app", "main")]),
        dependency_overlay,
        vc_descriptors: Vec::new(),
        profile: TaskGraphProfile::default(),
    })
}

fn diamond_failure_graph(left_coverage: DependencyCoverage) -> TaskGraph {
    let root = TaskId::new_for_test("root");
    let left = TaskId::new_for_test("left");
    let right = TaskId::new_for_test("right");
    let join = TaskId::new_for_test("join");
    TaskGraph {
        version: crate::task_graph::TaskGraphVersion::current(),
        snapshot: snapshot(5),
        // Join intentionally sorts before left so direct-block propagation
        // cannot rely on one forward scan before the no-schedulable fallback.
        tasks: vec![
            scheduler_test_task(
                root.clone(),
                TaskKind::PackageResolve,
                Vec::new(),
                DependencyCoverage::Complete,
                PriorityClass::Root,
            ),
            scheduler_test_task(
                join.clone(),
                TaskKind::ArtifactCommit,
                vec![left.clone(), right.clone()],
                DependencyCoverage::Complete,
                PriorityClass::Commit,
            ),
            scheduler_test_task(
                left.clone(),
                TaskKind::Frontend,
                vec![root.clone()],
                left_coverage,
                PriorityClass::Source,
            ),
            scheduler_test_task(
                right.clone(),
                TaskKind::ModuleResolve,
                vec![root.clone()],
                DependencyCoverage::Complete,
                PriorityClass::Source,
            ),
        ],
        edges: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn scheduler_test_task(
    id: TaskId,
    kind: TaskKind,
    dependencies: Vec<TaskId>,
    dependency_coverage: DependencyCoverage,
    priority_class: PriorityClass,
) -> BuildTask {
    BuildTask {
        id,
        kind,
        unit: WorkUnit::Workspace,
        phases: vec![pipeline_phase_for_kind(kind)],
        dependencies,
        dependency_coverage,
        resource_class: ResourceClass::CpuLocal,
        priority_class,
    }
}

fn pipeline_phase_for_kind(kind: TaskKind) -> PipelinePhase {
    match kind {
        TaskKind::PackageResolve => PipelinePhase::PackageResolve,
        TaskKind::SourceLoad => PipelinePhase::SourceLoad,
        TaskKind::Frontend => PipelinePhase::Frontend,
        TaskKind::ModuleResolve => PipelinePhase::ModuleResolve,
        TaskKind::CheckAndElaborate => PipelinePhase::TypeChecking,
        TaskKind::VcGenerate => PipelinePhase::VcGenerate,
        TaskKind::VcDischarge => PipelinePhase::VcDischarge,
        TaskKind::AtpSolve => PipelinePhase::AtpSolve,
        TaskKind::BackendRun => PipelinePhase::BackendRun,
        TaskKind::KernelCheck => PipelinePhase::KernelCheck,
        TaskKind::ArtifactCommit => PipelinePhase::ArtifactCommit,
        TaskKind::DocumentationExtract => PipelinePhase::DocumentationExtract,
    }
}

fn build_plan(packages: Vec<PackagePlan>) -> BuildPlan {
    BuildPlan {
        workspace_root: WorkspaceRoot::new("."),
        packages,
        dependency_graph: DependencyGraph { edges: Vec::new() },
        lockfile: Lockfile {
            schema_version: 1,
            packages: Vec::new(),
        },
        toolchain: ToolchainInfo::new("test"),
        verifier_config: WorkspaceVerifierConfig {
            packages: Vec::new(),
        },
        build_config: WorkspaceBuildConfig {
            packages: Vec::new(),
        },
    }
}

fn workspace_package(package_id: &str) -> PackagePlan {
    PackagePlan {
        package_id: PackageId::new(package_id),
        version: Version::new(1, 0, 0),
        source: PackagePlanSource::Workspace {
            root: package_id.to_owned(),
            source_root: format!("{package_id}/src"),
            manifest_path: format!("{package_id}/mizar.pkg"),
        },
        edition: Edition::new("2025"),
        dependencies: Vec::new(),
        verifier_config: VerifierConfig::default(),
        build_config: BuildConfig::default(),
    }
}

fn module_index(modules: Vec<ModuleIndexEntry>) -> ModuleIndex {
    let packages = modules
        .iter()
        .map(|entry| crate::module_index::PackageIndexEntry {
            package_id: entry.module.package.clone(),
            version: Version::new(1, 0, 0),
            edition: Edition::new("2025"),
            source: crate::module_index::PackageIndexSource::Workspace {
                package_root: entry.module.package.as_str().to_owned(),
                source_root: format!("{}/src", entry.module.package.as_str()),
                manifest_path: format!("{}/mizar.pkg", entry.module.package.as_str()),
            },
            dependencies: Vec::new(),
        })
        .collect();
    ModuleIndex {
        packages,
        namespace_bindings: Vec::new(),
        modules,
        dependency_summaries: Vec::new(),
    }
}

fn workspace_module(package_id: &str, module_path: &str) -> ModuleIndexEntry {
    let module = module_id(package_id, module_path);
    ModuleIndexEntry {
        module: module.clone(),
        package_id: module.package.clone(),
        module_path: module.path.clone(),
        location: ModuleIndexLocation::WorkspaceFile {
            source_root: format!("{package_id}/src"),
            normalized_path: format!("{package_id}/src/{module_path}.miz"),
            source_relative_path: format!("{module_path}.miz"),
        },
        edition: Edition::new("2025"),
    }
}

fn module_id(package_id: &str, module_path: &str) -> ModuleId {
    ModuleId::new(PackageId::new(package_id), ModulePath::new(module_path))
}

fn vc_descriptor(
    id: &str,
    module: ModuleId,
    order_key: &str,
    backend_profiles: Vec<&str>,
    evidence_candidates: Vec<&str>,
) -> VcTaskDescriptor {
    VcTaskDescriptor::new(
        VcTaskDescriptorId::new(id),
        module,
        VcOrderKey::new(order_key),
        backend_profiles
            .into_iter()
            .map(BackendProfileId::new)
            .collect(),
        evidence_candidates
            .into_iter()
            .map(crate::task_graph::EvidenceCandidateId::new)
            .collect(),
    )
}

fn snapshot(seed: u8) -> BuildSnapshotId {
    let hex = format!("{seed:02x}").repeat(Hash::BYTE_LEN);
    BuildSnapshotId::from_published_schema_str(&format!("mizar-session-build-snapshot-v1:{hex}"))
        .expect("valid snapshot id")
}

fn task_id_for_kind(graph: &TaskGraph, kind: TaskKind) -> TaskId {
    graph
        .tasks
        .iter()
        .find(|task| task.kind == kind)
        .expect("task exists")
        .id
        .clone()
}

fn task_ids_for_kind(graph: &TaskGraph, kind: TaskKind) -> Vec<TaskId> {
    graph
        .tasks
        .iter()
        .filter(|task| task.kind == kind)
        .map(|task| task.id.clone())
        .collect()
}

fn task_kind_for_id(graph: &TaskGraph, task_id: &TaskId) -> TaskKind {
    graph
        .tasks
        .iter()
        .find(|task| &task.id == task_id)
        .expect("task exists")
        .kind
}

fn task_id_for_module_kind(
    graph: &TaskGraph,
    kind: TaskKind,
    package_id: &str,
    module_path: &str,
) -> TaskId {
    let expected = module_id(package_id, module_path);
    graph
        .tasks
        .iter()
        .find(|task| {
            task.kind == kind
                && matches!(&task.unit, WorkUnit::Module { module } if module == &expected)
        })
        .expect("module task exists")
        .id
        .clone()
}

fn task_belongs_to_module(
    graph: &TaskGraph,
    task_id: &TaskId,
    package_id: &str,
    module_path: &str,
) -> bool {
    let expected = module_id(package_id, module_path);
    graph
        .tasks
        .iter()
        .find(|task| &task.id == task_id)
        .is_some_and(|task| match &task.unit {
            WorkUnit::Module { module }
            | WorkUnit::Vc { module, .. }
            | WorkUnit::BackendAttempt { module, .. }
            | WorkUnit::EvidenceCandidate { module, .. } => module == &expected,
            WorkUnit::Workspace | WorkUnit::Package { .. } => false,
        })
}

fn assert_backend_batches_are_serial(graph: &TaskGraph, batches: &[Vec<TaskId>]) {
    assert!(batches.iter().all(|batch| {
        batch
            .iter()
            .filter(|task_id| task_kind_for_id(graph, task_id) == TaskKind::BackendRun)
            .count()
            <= 1
    }));
}

fn telemetry_counts(
    run: &SchedulerRun,
    status: ResourceAdmissionStatus,
) -> BTreeMap<TaskId, usize> {
    let mut counts = BTreeMap::new();
    for telemetry in run
        .resource_telemetry
        .iter()
        .filter(|telemetry| telemetry.status == status)
    {
        *counts.entry(telemetry.task_id.clone()).or_default() += 1;
    }
    counts
}

fn assert_all_admitted_tasks_release_once(run: SchedulerRun) {
    let admitted = telemetry_counts(&run, ResourceAdmissionStatus::Admitted);
    let released = telemetry_counts(&run, ResourceAdmissionStatus::Released);

    assert_eq!(admitted, released);
    assert!(released.values().all(|count| *count == 1));
}

fn result_for<'a>(run: &'a SchedulerRun, task_id: &TaskId) -> &'a SchedulerResult {
    run.results
        .iter()
        .find(|result| &result.task_id == task_id)
        .expect("result exists")
}

fn blocked_record_for<'a>(run: &'a SchedulerRun, task_id: &TaskId) -> &'a BlockedTaskRecord {
    run.blocked_records
        .iter()
        .find(|record| &record.task_id == task_id)
        .expect("blocked record exists")
}

fn state_for(run: &SchedulerRun, task_id: &TaskId) -> TaskState {
    run.task_states
        .iter()
        .find(|record| &record.task_id == task_id)
        .expect("state exists")
        .state
}
