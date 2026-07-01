use mizar_build::{
    scheduler::{SchedulerEvent, SchedulerEventKind, TaskStateRecord},
    task_graph::PipelinePhase,
};

use crate::{
    driver::DriverMissingPhaseService,
    events::{
        BuildEvent, BuildEventIdentity, BuildEventKind, BuildEventOrderKey, BuildEventStream,
        PlanningEventStatus, TaskEventRef,
    },
    request::{BuildSession, BuildSessionOutcome, BuildSessionState, PublicationDecision},
};

#[derive(Default)]
pub(super) struct DriverEventDetails<'a> {
    pub(super) planning: Option<PlanningEventStatus>,
    pub(super) missing_services: &'a [DriverMissingPhaseService],
    pub(super) dispatch_gap_phases: &'a [PipelinePhase],
    pub(super) scheduler_events: &'a [SchedulerEvent],
    pub(super) scheduler_task_states: &'a [TaskStateRecord],
}

pub(super) fn submission_events(
    session: &BuildSession,
    publication: PublicationDecision,
    details: DriverEventDetails<'_>,
) -> BuildEventStream {
    let identity = BuildEventIdentity {
        session: session.id,
        lane: session.request.lane,
        generation: session.request.generation,
        snapshot: Some(session.captured.snapshot.id),
        publication,
    };
    let mut events = vec![
        BuildEvent::new(
            identity.clone(),
            BuildEventOrderKey::new(0, 0),
            BuildEventKind::SessionAccepted,
        ),
        BuildEvent::new(
            identity.clone(),
            BuildEventOrderKey::new(10, 0),
            BuildEventKind::SnapshotCaptured,
        ),
    ];

    if let Some(status) = details.planning {
        events.push(BuildEvent::new(
            identity.clone(),
            BuildEventOrderKey::new(20, 0),
            BuildEventKind::PlanningReady { status },
        ));
    }

    for missing in details.missing_services {
        events.push(BuildEvent::new(
            identity.clone(),
            BuildEventOrderKey::new(30, 1).with_phase(missing.phase),
            BuildEventKind::PhaseServiceGap {
                phase: missing.phase,
                availability: missing.availability,
            },
        ));
    }

    if !details.dispatch_gap_phases.is_empty() {
        let first_phase = details.dispatch_gap_phases[0];
        events.push(BuildEvent::new(
            identity.clone(),
            BuildEventOrderKey::new(31, 1).with_phase(first_phase),
            BuildEventKind::DispatchGap {
                phases: details.dispatch_gap_phases.to_vec(),
            },
        ));
    }

    for event in details.scheduler_events {
        if event.kind == SchedulerEventKind::RunFinished {
            continue;
        }
        let Some(task_id_ref) = event.task_id.as_ref() else {
            continue;
        };
        let task_state = details
            .scheduler_task_states
            .iter()
            .find(|record| record.task_id == *task_id_ref)
            .map(|record| record.state);
        let task_id = task_id_ref.as_str().to_owned();
        events.push(BuildEvent::new(
            identity.clone(),
            BuildEventOrderKey::new(40, 1)
                .with_scheduler_order(event.order)
                .with_work_unit(task_id.clone()),
            BuildEventKind::TaskProgress {
                task: TaskEventRef::new(task_id, event.kind, task_state),
            },
        ));
    }

    if matches!(publication, PublicationDecision::Suppressed(_)) {
        events.push(BuildEvent::new(
            identity.clone(),
            BuildEventOrderKey::new(80, 1),
            BuildEventKind::PublicationSuppressed,
        ));
    }

    if let BuildSessionState::Finished(outcome) = session.state {
        events.push(BuildEvent::new(
            identity,
            BuildEventOrderKey::new(90, 1),
            BuildEventKind::SessionFinished { outcome },
        ));
    }

    BuildEventStream::from_events(session.id, true, events)
        .expect("driver-generated events reference the stored session")
}

pub(super) fn append_terminal_events(
    stream: &BuildEventStream,
    session: &BuildSession,
    publication: PublicationDecision,
) -> BuildEventStream {
    let identity = BuildEventIdentity {
        session: session.id,
        lane: session.request.lane,
        generation: session.request.generation,
        snapshot: Some(session.captured.snapshot.id),
        publication,
    };
    let mut events = stream.events().to_vec();
    if matches!(publication, PublicationDecision::Suppressed(_)) {
        events.push(BuildEvent::new(
            identity.clone(),
            BuildEventOrderKey::new(80, 2),
            BuildEventKind::PublicationSuppressed,
        ));
    }
    if let BuildSessionState::Finished(outcome) = session.state {
        events.push(BuildEvent::new(
            identity,
            BuildEventOrderKey::new(90, 2),
            BuildEventKind::SessionFinished { outcome },
        ));
    }
    BuildEventStream::from_events(session.id, true, events)
        .expect("driver-generated cancellation events reference the stored session")
}

pub(super) fn suppress_watch_replay_events(
    stream: &BuildEventStream,
    session: &BuildSession,
    publication: PublicationDecision,
) -> BuildEventStream {
    let identity = BuildEventIdentity {
        session: session.id,
        lane: session.request.lane,
        generation: session.request.generation,
        snapshot: Some(session.captured.snapshot.id),
        publication,
    };
    let mut events = Vec::new();
    for event in stream.events() {
        if matches!(
            event.kind,
            BuildEventKind::PublicationSuppressed
                | BuildEventKind::SessionFinished { .. }
                | BuildEventKind::PhaseReady { .. }
                | BuildEventKind::DiagnosticsReady { .. }
                | BuildEventKind::ArtifactBoundary { .. }
        ) {
            continue;
        }
        let mut suppressed = event.clone();
        suppressed.identity.publication = publication;
        events.push(suppressed);
    }
    if matches!(publication, PublicationDecision::Suppressed(_)) {
        events.push(BuildEvent::new(
            identity.clone(),
            BuildEventOrderKey::new(80, 3),
            BuildEventKind::PublicationSuppressed,
        ));
    }
    events.push(BuildEvent::new(
        identity,
        BuildEventOrderKey::new(90, 3),
        BuildEventKind::SessionFinished {
            outcome: BuildSessionOutcome::Superseded,
        },
    ));
    BuildEventStream::from_events(session.id, true, events)
        .expect("watch-suppressed replay events reference the stored session")
}
