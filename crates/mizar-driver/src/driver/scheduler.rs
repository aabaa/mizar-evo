use std::collections::{BTreeMap, BTreeSet};

use mizar_build::{
    scheduler::{
        SchedulerDiagnosticRef, SchedulerDispatchOutcome, SchedulerDispatchTask, SchedulerRun,
        SchedulerTaskDispatcher, TaskState,
    },
    task_graph::{BuildTask, PipelinePhase, TaskGraph},
};

use crate::{
    driver::{DriverSchedulerRun, PhaseDispatchInputProvider},
    registry::{
        PhaseExecutionResources, PhaseInput, PhaseRegistry, PhaseRegistryError, PhaseStatus,
    },
    request::BuildSessionOutcome,
};

impl DriverSchedulerRun {
    pub(super) fn from_scheduler_run(run: SchedulerRun) -> Self {
        Self {
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
    phase_inputs: Option<&'a dyn PhaseDispatchInputProvider>,
}

impl<'a> RegistrySchedulerDispatcher<'a> {
    pub(super) const fn new(
        registry: &'a PhaseRegistry,
        phase_inputs: Option<&'a dyn PhaseDispatchInputProvider>,
    ) -> Self {
        Self {
            registry,
            phase_inputs,
        }
    }
}

impl SchedulerTaskDispatcher for RegistrySchedulerDispatcher<'_> {
    fn dispatch(&mut self, task: SchedulerDispatchTask<'_>) -> SchedulerDispatchOutcome {
        dispatch_registry_phase(self.registry, self.phase_inputs, task)
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
    phase_inputs: Option<&dyn PhaseDispatchInputProvider>,
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

    let Some(identities) =
        phase_inputs.and_then(|provider| provider.input_identities_for_task(task.task))
    else {
        return SchedulerDispatchOutcome::blocked(vec![dispatch_diagnostic(
            task.task,
            "missing_phase_input_identities",
            "owner-provided phase input identities are unavailable",
        )]);
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
        let input = PhaseInput::new(task.snapshot, task.task.unit.clone(), identities.clone());
        match registry.execute_phase_with_resources(
            *phase,
            input,
            PhaseExecutionResources {
                cancellation: task.cancellation.clone(),
                ..PhaseExecutionResources::default()
            },
        ) {
            Ok(result) => {
                let status = result.result.status;
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
