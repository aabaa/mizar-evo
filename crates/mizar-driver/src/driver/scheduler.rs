use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use mizar_build::{
    scheduler::{
        SchedulerDiagnosticRef, SchedulerDispatchOutcome, SchedulerDispatchTask, SchedulerRun,
        SchedulerTaskDispatcher, TaskState,
    },
    task_graph::{BuildTask, PipelinePhase, TaskGraph, TaskId},
};
use mizar_ir::{
    dispatch_input::{PhaseDispatchInputProvider, PhaseDispatchInputRequest},
    publisher::PhaseOutputPublisher,
};

use crate::{
    driver::DriverSchedulerRun,
    registry::{
        PhaseExecutionResources, PhaseInput, PhaseRegistry, PhaseRegistryError, PhaseResult,
        PhaseStatus, SourceLoadInputs,
    },
    request::BuildSessionOutcome,
};

impl DriverSchedulerRun {
    pub(super) fn from_scheduler_run(run: SchedulerRun) -> Self {
        Self {
            phase_results: BTreeMap::new(),
            task_states: run.task_states,
            events: run.events,
            diagnostics: run.diagnostics,
        }
    }
}

pub(super) fn scheduler_outcome(run: &SchedulerRun) -> BuildSessionOutcome {
    if run
        .task_states
        .iter()
        .any(|record| record.state == TaskState::Failed)
    {
        BuildSessionOutcome::Failed
    } else if run
        .task_states
        .iter()
        .any(|record| record.state == TaskState::Blocked)
    {
        BuildSessionOutcome::Blocked
    } else if run
        .task_states
        .iter()
        .any(|record| record.state == TaskState::Cancelled)
    {
        BuildSessionOutcome::Cancelled
    } else {
        BuildSessionOutcome::Succeeded
    }
}

pub(super) struct RegistrySchedulerDispatcher<'a> {
    registry: &'a PhaseRegistry,
    phase_inputs: Option<&'a dyn PhaseDispatchInputProvider<BuildTask>>,
    publisher: Option<&'a Arc<PhaseOutputPublisher>>,
    source_load: Option<SourceLoadInputs<'a>>,
    pub(super) phase_results: BTreeMap<TaskId, Vec<PhaseResult>>,
}

impl<'a> RegistrySchedulerDispatcher<'a> {
    pub(super) const fn new(
        registry: &'a PhaseRegistry,
        phase_inputs: Option<&'a dyn PhaseDispatchInputProvider<BuildTask>>,
        publisher: Option<&'a Arc<PhaseOutputPublisher>>,
        source_load: Option<SourceLoadInputs<'a>>,
    ) -> Self {
        Self {
            registry,
            phase_inputs,
            publisher,
            source_load,
            phase_results: BTreeMap::new(),
        }
    }
}

impl SchedulerTaskDispatcher for RegistrySchedulerDispatcher<'_> {
    fn dispatch(&mut self, task: SchedulerDispatchTask<'_>) -> SchedulerDispatchOutcome {
        dispatch_registry_phase(
            self.registry,
            self.phase_inputs,
            self.publisher,
            self.source_load,
            &mut self.phase_results,
            task,
        )
    }
}

pub(super) fn dispatch_gap_phases(
    task_graph: &TaskGraph,
    run: &SchedulerRun,
) -> Vec<PipelinePhase> {
    let dispatch_blocked_tasks = run
        .results
        .iter()
        .filter(|result| {
            result.state == TaskState::Blocked
                && result
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == "missing_phase_input_identities")
        })
        .map(|record| record.task_id.clone())
        .collect::<BTreeSet<_>>();
    let mut phases = BTreeMap::new();
    for task in task_graph.tasks() {
        if dispatch_blocked_tasks.contains(&task.id) {
            for phase in &task.phases {
                if *phase != PipelinePhase::PackageResolve {
                    phases.entry(*phase).or_insert(());
                }
            }
        }
    }
    phases.into_keys().collect()
}

fn dispatch_registry_phase(
    registry: &PhaseRegistry,
    phase_inputs: Option<&dyn PhaseDispatchInputProvider<BuildTask>>,
    publisher: Option<&Arc<PhaseOutputPublisher>>,
    source_load: Option<SourceLoadInputs<'_>>,
    phase_results: &mut BTreeMap<TaskId, Vec<PhaseResult>>,
    task: SchedulerDispatchTask<'_>,
) -> SchedulerDispatchOutcome {
    if task.task.phases.is_empty() {
        return SchedulerDispatchOutcome::blocked(vec![dispatch_diagnostic(
            task.task,
            "missing_phase",
            "scheduler-selected task has no registry phase",
        )]);
    }
    if task
        .task
        .phases
        .iter()
        .all(|phase| *phase == PipelinePhase::PackageResolve)
    {
        return SchedulerDispatchOutcome::complete();
    }

    let dispatch_input = match phase_inputs {
        Some(provider) => {
            match provider
                .dispatch_input_for_task(PhaseDispatchInputRequest::new(task.task, task.snapshot))
            {
                Ok(Some(bundle)) => {
                    if let Err(error) = bundle.validate_snapshot(task.snapshot) {
                        return SchedulerDispatchOutcome::failed(vec![dispatch_diagnostic(
                            task.task,
                            "invalid_phase_dispatch_input",
                            error.to_string(),
                        )]);
                    }
                    bundle
                }
                Ok(None) => {
                    return SchedulerDispatchOutcome::blocked(vec![dispatch_diagnostic(
                        task.task,
                        "missing_phase_input_identities",
                        "owner-provided phase input bundle is unavailable",
                    )]);
                }
                Err(error) => {
                    return SchedulerDispatchOutcome::failed(vec![dispatch_diagnostic(
                        task.task,
                        "invalid_phase_dispatch_input",
                        error.to_string(),
                    )]);
                }
            }
        }
        None => {
            if task.task.phases.iter().any(|phase| {
                matches!(
                    registry.descriptor_for_phase(*phase),
                    Err(PhaseRegistryError::MissingPhaseService { .. })
                )
            }) {
                return SchedulerDispatchOutcome::blocked(vec![dispatch_diagnostic(
                    task.task,
                    "missing_phase_service",
                    "phase service is unavailable at dispatch time",
                )]);
            }
            let bundle = (|| {
                use crate::{
                    frontend_adapter::{FrontendService, SourceLoadService},
                    registry::PhaseService,
                };
                use mizar_ir::{
                    dispatch_input::{PhaseDispatchInputBundle, SealedParentOutputHandle},
                    identity::{OutputKind, PipelinePhase as IrPhase, WorkUnit as IrWorkUnit},
                };
                let expected = match task.task.phases.as_slice() {
                    [PipelinePhase::SourceLoad] => SourceLoadService.phase(),
                    [PipelinePhase::Frontend] => FrontendService {
                        artifact_roots: Vec::new(),
                    }
                    .phase(),
                    _ => return None,
                };
                if registry.descriptor_for_phase(task.task.phases[0]).ok()? != &expected {
                    return None;
                }
                let publisher = publisher?;
                publisher.validate_current_snapshot(task.snapshot).ok()?;
                let inputs = source_load?;
                if inputs.snapshot.id != task.snapshot {
                    return None;
                }
                let mizar_build::task_graph::WorkUnit::Module { module } = &task.task.unit else {
                    return None;
                };
                let mut versions = inputs.snapshot.source_versions.iter().filter(|version| {
                    version.package_id == module.package && version.module_path == module.path
                });
                let version = versions.next()?;
                if versions.next().is_some() {
                    return None;
                }
                let key = crate::frontend_adapter::source_input_hash(version);
                if task.task.phases == [PipelinePhase::SourceLoad] {
                    return Some(PhaseDispatchInputBundle::without_parent_outputs(
                        task.snapshot,
                        key,
                        Vec::new(),
                    ));
                }
                let unit = IrWorkUnit::new(format!(
                    "{:?}:{:?}",
                    module.package.as_str(),
                    module.path.as_str()
                ));
                let parents = task
                    .task
                    .dependencies
                    .iter()
                    .flat_map(|id| phase_results.get(id).into_iter().flatten())
                    .filter(|result| result.status == PhaseStatus::Complete)
                    .flat_map(|result| result.output_refs.iter())
                    .filter(|parent| {
                        (parent.phase() == &IrPhase::new("SourceLoad")
                            && parent.work_unit() == &unit
                            && parent.output_kind() == &OutputKind::new("SourceUnit"))
                            || (parent.phase() == &IrPhase::new("Frontend")
                                && parent.output_kind() == &OutputKind::new("FrontendOutput"))
                    })
                    .collect::<Vec<_>>();
                if parents
                    .iter()
                    .filter(|parent| parent.phase() == &IrPhase::new("SourceLoad"))
                    .count()
                    != 1
                {
                    return None;
                }
                let parents = parents
                    .into_iter()
                    .map(|parent| {
                        SealedParentOutputHandle::from_current_output(
                            publisher,
                            task.snapshot,
                            parent.clone(),
                        )
                        .ok()
                    })
                    .collect::<Option<Vec<_>>>()?;
                let mut dependencies = inputs
                    .module_index
                    .dependency_summaries
                    .iter()
                    .map(|summary| summary.content_hash)
                    .collect::<Vec<_>>();
                dependencies.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
                PhaseDispatchInputBundle::new(task.snapshot, key, dependencies, parents).ok()
            })();
            let Some(bundle) = bundle else {
                return SchedulerDispatchOutcome::blocked(vec![dispatch_diagnostic(
                    task.task,
                    "missing_phase_input_identities",
                    "owner-provided phase input bundle is unavailable",
                )]);
            };
            bundle
        }
    };

    let mut dispatched_descriptors = BTreeSet::new();
    for phase in &task.task.phases {
        if *phase == PipelinePhase::PackageResolve {
            continue;
        }
        let descriptor = match registry.descriptor_for_phase(*phase) {
            Ok(descriptor) => descriptor,
            Err(PhaseRegistryError::MissingPhaseService { .. }) => {
                return SchedulerDispatchOutcome::blocked(vec![dispatch_diagnostic(
                    task.task,
                    "missing_phase_service",
                    "phase service is unavailable at dispatch time",
                )]);
            }
            Err(error) => {
                return SchedulerDispatchOutcome::failed(vec![dispatch_diagnostic(
                    task.task,
                    "phase_registry_error",
                    error.to_string(),
                )]);
            }
        };
        let descriptor_key = (
            descriptor.service_name.clone(),
            descriptor.schema_version.clone(),
            descriptor.output_kind.clone(),
            descriptor.phases.clone(),
        );
        if !dispatched_descriptors.insert(descriptor_key) {
            continue;
        }
        let input = PhaseInput::new(task.task.unit.clone(), dispatch_input.clone());
        match registry.execute_phase_with_resources(
            *phase,
            input,
            PhaseExecutionResources {
                cancellation: task.cancellation.clone(),
                output_publisher: publisher.cloned(),
                source_load,
                diagnostics: matches!(phase, PipelinePhase::SourceLoad | PipelinePhase::Frontend)
                    .then(|| {
                        let (diagnostic_phase, producer) = if *phase == PipelinePhase::SourceLoad {
                            (
                                mizar_diagnostics::failure_record::PipelinePhase::SourceLoad,
                                "SourceLoad",
                            )
                        } else {
                            (
                                mizar_diagnostics::failure_record::PipelinePhase::Frontend,
                                "Frontend",
                            )
                        };
                        mizar_diagnostics::sink::DiagnosticSink::new(
                            mizar_diagnostics::sink::DiagnosticProducerScope::new(
                                diagnostic_phase,
                                task.snapshot,
                                producer,
                            ),
                        )
                    }),
            },
        ) {
            Ok(result) => {
                let mut result = result.result;
                let status = result.status;
                let valid_outputs = result.output_refs.iter().all(|output| {
                    publisher.is_some_and(|publisher| {
                        publisher
                            .validate_current_output(task.snapshot, output)
                            .is_ok()
                            && publisher.storage().validate_handle(output).is_ok()
                    })
                });
                let valid_snapshot = publisher.is_none_or(|publisher| {
                    publisher.validate_current_snapshot(task.snapshot).is_ok()
                });
                if !valid_outputs
                    || !valid_snapshot
                    || result
                        .diagnostics
                        .iter()
                        .any(|batch| batch.scope().source_snapshot() != task.snapshot)
                {
                    return SchedulerDispatchOutcome::failed(vec![dispatch_diagnostic(
                        task.task,
                        "invalid_phase_result",
                        "phase result failed snapshot or sealed-output validation",
                    )]);
                }
                if status != PhaseStatus::Complete {
                    result.output_refs.clear();
                }
                phase_results
                    .entry(task.task.id.clone())
                    .or_default()
                    .push(result);
                let diagnostic = phase_status_diagnostic(task.task, status);
                match status {
                    PhaseStatus::Complete => {}
                    PhaseStatus::Recoverable | PhaseStatus::Fatal => {
                        return SchedulerDispatchOutcome::failed(diagnostic.into_iter().collect());
                    }
                    PhaseStatus::Blocking => {
                        return SchedulerDispatchOutcome::blocked(diagnostic.into_iter().collect());
                    }
                    PhaseStatus::Cancelled => return SchedulerDispatchOutcome::cancelled(),
                }
            }
            Err(PhaseRegistryError::MissingPhaseService { .. }) => {
                return SchedulerDispatchOutcome::blocked(vec![dispatch_diagnostic(
                    task.task,
                    "missing_phase_service",
                    "phase service is unavailable at dispatch time",
                )]);
            }
            Err(error) => {
                return SchedulerDispatchOutcome::failed(vec![dispatch_diagnostic(
                    task.task,
                    "phase_registry_error",
                    error.to_string(),
                )]);
            }
        }
    }
    SchedulerDispatchOutcome::complete()
}

fn phase_status_diagnostic(
    task: &BuildTask,
    status: PhaseStatus,
) -> Option<SchedulerDiagnosticRef> {
    match status {
        PhaseStatus::Complete | PhaseStatus::Cancelled => None,
        PhaseStatus::Recoverable => Some(dispatch_diagnostic(
            task,
            "phase_recoverable",
            "phase completed with recoverable diagnostics",
        )),
        PhaseStatus::Blocking => Some(dispatch_diagnostic(
            task,
            "phase_blocking",
            "phase reported a blocking outcome",
        )),
        PhaseStatus::Fatal => Some(dispatch_diagnostic(
            task,
            "phase_fatal",
            "phase reported a fatal outcome",
        )),
    }
}

fn dispatch_diagnostic(
    task: &BuildTask,
    code: impl Into<String>,
    message: impl Into<String>,
) -> SchedulerDiagnosticRef {
    SchedulerDiagnosticRef::new(task.id.as_str(), code, message)
}
