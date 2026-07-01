use mizar_build::{
    scheduler::{SchedulerEventKind, SchedulerOrderKey, TaskState},
    task_graph::PipelinePhase,
};
use mizar_session::{BuildSessionId, BuildSnapshotId};

use crate::{
    registry::{PhaseServiceAvailability, PhaseStatus},
    request::{BuildLaneId, BuildRequestGeneration, BuildSessionOutcome, PublicationDecision},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEventStream {
    pub session: BuildSessionId,
    pub known_session: bool,
    events: Vec<BuildEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEvent {
    pub identity: BuildEventIdentity,
    pub order: BuildEventOrderKey,
    pub kind: BuildEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEventIdentity {
    pub session: BuildSessionId,
    pub lane: BuildLaneId,
    pub generation: BuildRequestGeneration,
    pub snapshot: Option<BuildSnapshotId>,
    pub publication: PublicationDecision,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BuildEventOrderKey {
    pub lifecycle_rank: u16,
    pub scheduler_order: Option<SchedulerOrderKey>,
    pub phase: Option<PipelinePhase>,
    pub work_unit_identity: String,
    pub owner_order: String,
    pub kind_rank: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BuildEventKind {
    SessionAccepted,
    SnapshotCaptured,
    PlanningReady {
        status: PlanningEventStatus,
    },
    TaskProgress {
        task: TaskEventRef,
    },
    PhaseServiceGap {
        phase: PipelinePhase,
        availability: PhaseServiceAvailability,
    },
    DispatchGap {
        phases: Vec<PipelinePhase>,
    },
    OwnerReadinessGap {
        owner: EventOwner,
        classification: OwnerGapClassification,
    },
    PhaseReady {
        phase: PipelinePhase,
        status: PhaseStatus,
    },
    DiagnosticsReady {
        records: OwnerRecordRef,
    },
    ArtifactBoundary {
        committed: OwnerRecordRef,
    },
    PublicationSuppressed,
    SessionFinished {
        outcome: BuildSessionOutcome,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlanningEventStatus {
    Ready,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskEventRef {
    pub task_id: String,
    pub scheduler_event: SchedulerEventKind,
    pub state: Option<TaskState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum EventOwner {
    Diagnostics,
    Artifact,
    ProducerOutput,
    LspBridge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OwnerGapClassification {
    ExternalDependencyGap,
    Deferred,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerRecordRef {
    pub owner: EventOwner,
    pub identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BuildEventError {
    SessionMismatch {
        expected: BuildSessionId,
        actual: BuildSessionId,
    },
    UnknownSession {
        session: BuildSessionId,
    },
    MissingSnapshot {
        kind: String,
    },
    SuppressedCurrentPayload {
        kind: String,
    },
    InvalidPublicationSuppression {
        kind: String,
    },
    OwnerMismatch {
        kind: String,
        expected: EventOwner,
        actual: EventOwner,
    },
    IdentityMismatch {
        expected: BuildEventIdentityKey,
        actual: BuildEventIdentityKey,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEventLog {
    session: BuildSessionId,
    known_session: bool,
    identity: Option<BuildEventIdentityKey>,
    events: Vec<BuildEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildEventIdentityKey {
    pub lane: BuildLaneId,
    pub generation: BuildRequestGeneration,
    pub snapshot: Option<BuildSnapshotId>,
}

impl BuildEventStream {
    pub fn empty(session: BuildSessionId, known_session: bool) -> Self {
        Self {
            session,
            known_session,
            events: Vec::new(),
        }
    }

    pub fn from_events(
        session: BuildSessionId,
        known_session: bool,
        events: Vec<BuildEvent>,
    ) -> Result<Self, BuildEventError> {
        let mut log = BuildEventLog::new(session, known_session);
        for event in events {
            log.push(event)?;
        }
        Ok(log.into_stream())
    }

    pub fn events(&self) -> &[BuildEvent] {
        &self.events
    }

    pub fn replay(&self) -> Self {
        self.clone()
    }
}

impl BuildEvent {
    pub fn new(
        identity: BuildEventIdentity,
        order: BuildEventOrderKey,
        kind: BuildEventKind,
    ) -> Self {
        Self {
            identity,
            order,
            kind,
        }
    }
}

impl BuildEventOrderKey {
    pub fn new(lifecycle_rank: u16, kind_rank: u16) -> Self {
        Self {
            lifecycle_rank,
            scheduler_order: None,
            phase: None,
            work_unit_identity: String::new(),
            owner_order: String::new(),
            kind_rank,
        }
    }

    pub fn with_scheduler_order(mut self, order: SchedulerOrderKey) -> Self {
        self.scheduler_order = Some(order);
        self
    }

    pub fn with_phase(mut self, phase: PipelinePhase) -> Self {
        self.phase = Some(phase);
        self
    }

    pub fn with_work_unit(mut self, identity: impl Into<String>) -> Self {
        self.work_unit_identity = identity.into();
        self
    }

    pub fn with_owner_order(mut self, order: impl Into<String>) -> Self {
        self.owner_order = order.into();
        self
    }
}

impl TaskEventRef {
    pub fn new(
        task_id: impl Into<String>,
        scheduler_event: SchedulerEventKind,
        state: Option<TaskState>,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            scheduler_event,
            state,
        }
    }
}

impl OwnerRecordRef {
    pub fn new(owner: EventOwner, identity: impl Into<String>) -> Self {
        Self {
            owner,
            identity: identity.into(),
        }
    }
}

impl BuildEventLog {
    pub fn new(session: BuildSessionId, known_session: bool) -> Self {
        Self {
            session,
            known_session,
            identity: None,
            events: Vec::new(),
        }
    }

    pub fn push(&mut self, event: BuildEvent) -> Result<(), BuildEventError> {
        if event.identity.session != self.session {
            return Err(BuildEventError::SessionMismatch {
                expected: self.session,
                actual: event.identity.session,
            });
        }
        if !self.known_session {
            return Err(BuildEventError::UnknownSession {
                session: self.session,
            });
        }
        let identity_key = event.identity.key();
        let mut next_identity = self.identity;
        if let Some(expected) = self.identity {
            if expected.lane != identity_key.lane || expected.generation != identity_key.generation
            {
                return Err(BuildEventError::IdentityMismatch {
                    expected,
                    actual: identity_key,
                });
            }
            if event_requires_snapshot(&event.kind) {
                if event.identity.snapshot.is_none() {
                    return Err(BuildEventError::MissingSnapshot {
                        kind: event_kind_name(&event.kind).to_owned(),
                    });
                }
                if let Some(expected_snapshot) = expected.snapshot {
                    if Some(expected_snapshot) != identity_key.snapshot {
                        return Err(BuildEventError::IdentityMismatch {
                            expected,
                            actual: identity_key,
                        });
                    }
                } else {
                    next_identity = Some(identity_key);
                }
            }
        } else {
            if event_requires_snapshot(&event.kind) && event.identity.snapshot.is_none() {
                return Err(BuildEventError::MissingSnapshot {
                    kind: event_kind_name(&event.kind).to_owned(),
                });
            }
            next_identity = Some(identity_key);
        }
        if let PublicationDecision::Suppressed(obsolete) = event.identity.publication
            && obsolete.lane_current
            && obsolete.request_snapshot_current
        {
            return Err(BuildEventError::InvalidPublicationSuppression {
                kind: event_kind_name(&event.kind).to_owned(),
            });
        }
        if is_current_payload(&event.kind)
            && matches!(
                event.identity.publication,
                PublicationDecision::Suppressed(_)
            )
        {
            return Err(BuildEventError::SuppressedCurrentPayload {
                kind: event_kind_name(&event.kind).to_owned(),
            });
        }
        if matches!(event.kind, BuildEventKind::PublicationSuppressed)
            && matches!(event.identity.publication, PublicationDecision::Current)
        {
            return Err(BuildEventError::InvalidPublicationSuppression {
                kind: event_kind_name(&event.kind).to_owned(),
            });
        }
        validate_owner(&event.kind)?;
        self.identity = next_identity;
        self.events.push(event);
        Ok(())
    }

    pub fn extend<I>(&mut self, events: I) -> Result<(), BuildEventError>
    where
        I: IntoIterator<Item = BuildEvent>,
    {
        for event in events {
            self.push(event)?;
        }
        Ok(())
    }

    pub fn into_stream(mut self) -> BuildEventStream {
        self.events.sort_by(|left, right| {
            left.order
                .cmp(&right.order)
                .then_with(|| event_tie_key(&left.kind).cmp(&event_tie_key(&right.kind)))
                .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
        });
        BuildEventStream {
            session: self.session,
            known_session: self.known_session,
            events: self.events,
        }
    }
}

impl BuildEventIdentity {
    fn key(&self) -> BuildEventIdentityKey {
        BuildEventIdentityKey {
            lane: self.lane,
            generation: self.generation,
            snapshot: self.snapshot,
        }
    }
}

pub fn diagnostics_gap_event(
    identity: BuildEventIdentity,
    classification: OwnerGapClassification,
    order: BuildEventOrderKey,
) -> BuildEvent {
    BuildEvent::new(
        identity,
        order,
        BuildEventKind::OwnerReadinessGap {
            owner: EventOwner::Diagnostics,
            classification,
        },
    )
}

fn event_tie_key(kind: &BuildEventKind) -> String {
    match kind {
        BuildEventKind::SessionAccepted => "00-session-accepted".to_owned(),
        BuildEventKind::SnapshotCaptured => "01-snapshot-captured".to_owned(),
        BuildEventKind::PlanningReady { status } => format!("02-planning-{status:?}"),
        BuildEventKind::TaskProgress { task } => {
            format!(
                "03-task-{:?}-{:?}-{}",
                task.scheduler_event, task.state, task.task_id
            )
        }
        BuildEventKind::PhaseServiceGap {
            phase,
            availability,
        } => {
            format!("04-phase-service-gap-{phase:?}-{availability:?}")
        }
        BuildEventKind::DispatchGap { phases } => format!("05-dispatch-gap-{phases:?}"),
        BuildEventKind::OwnerReadinessGap {
            owner,
            classification,
        } => {
            format!("06-owner-gap-{owner:?}-{classification:?}")
        }
        BuildEventKind::PhaseReady { phase, status } => {
            format!("07-phase-ready-{phase:?}-{status:?}")
        }
        BuildEventKind::DiagnosticsReady { records } => {
            format!(
                "08-diagnostics-ready-{:?}-{}",
                records.owner, records.identity
            )
        }
        BuildEventKind::ArtifactBoundary { committed } => {
            format!(
                "09-artifact-boundary-{:?}-{}",
                committed.owner, committed.identity
            )
        }
        BuildEventKind::PublicationSuppressed => "10-publication-suppressed".to_owned(),
        BuildEventKind::SessionFinished { outcome } => format!("11-session-finished-{outcome:?}"),
    }
}

fn event_requires_snapshot(kind: &BuildEventKind) -> bool {
    !matches!(kind, BuildEventKind::SessionAccepted)
}

fn is_current_payload(kind: &BuildEventKind) -> bool {
    matches!(
        kind,
        BuildEventKind::PhaseReady { .. }
            | BuildEventKind::DiagnosticsReady { .. }
            | BuildEventKind::ArtifactBoundary { .. }
    )
}

fn event_kind_name(kind: &BuildEventKind) -> &'static str {
    match kind {
        BuildEventKind::SessionAccepted => "SessionAccepted",
        BuildEventKind::SnapshotCaptured => "SnapshotCaptured",
        BuildEventKind::PlanningReady { .. } => "PlanningReady",
        BuildEventKind::TaskProgress { .. } => "TaskProgress",
        BuildEventKind::PhaseServiceGap { .. } => "PhaseServiceGap",
        BuildEventKind::DispatchGap { .. } => "DispatchGap",
        BuildEventKind::OwnerReadinessGap { .. } => "OwnerReadinessGap",
        BuildEventKind::PhaseReady { .. } => "PhaseReady",
        BuildEventKind::DiagnosticsReady { .. } => "DiagnosticsReady",
        BuildEventKind::ArtifactBoundary { .. } => "ArtifactBoundary",
        BuildEventKind::PublicationSuppressed => "PublicationSuppressed",
        BuildEventKind::SessionFinished { .. } => "SessionFinished",
    }
}

fn validate_owner(kind: &BuildEventKind) -> Result<(), BuildEventError> {
    match kind {
        BuildEventKind::DiagnosticsReady { records }
            if records.owner != EventOwner::Diagnostics =>
        {
            Err(BuildEventError::OwnerMismatch {
                kind: event_kind_name(kind).to_owned(),
                expected: EventOwner::Diagnostics,
                actual: records.owner,
            })
        }
        BuildEventKind::ArtifactBoundary { committed }
            if committed.owner != EventOwner::Artifact =>
        {
            Err(BuildEventError::OwnerMismatch {
                kind: event_kind_name(kind).to_owned(),
                expected: EventOwner::Artifact,
                actual: committed.owner,
            })
        }
        _ => Ok(()),
    }
}
