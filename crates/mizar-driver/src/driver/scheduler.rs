use std::collections::BTreeMap;

use mizar_build::{
    scheduler::{SchedulerRun, TaskState},
    task_graph::{PipelinePhase, TaskGraph},
};

use crate::{driver::DriverSchedulerRun, request::BuildSessionOutcome};

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

pub(super) fn dispatch_gap_phases(task_graph: &TaskGraph) -> Vec<PipelinePhase> {
    let mut phases = BTreeMap::new();
    for task in task_graph.tasks() {
        for phase in &task.phases {
            if *phase != PipelinePhase::PackageResolve {
                phases.entry(*phase).or_insert(());
            }
        }
    }
    phases.into_keys().collect()
}
