use mizar_ir::publisher::PhaseOutputPublisher;
use mizar_session::BuildSnapshotId;

use crate::{
    driver::{
        DriverCancelReason, DriverSubmitError, WatchModeGap, WatchModeGapOwner, WatchOwnerSeam,
        WatchSnapshotReplacement, WatchSnapshotReplacementStatus, WatchSubmitControl,
    },
    events::OwnerGapClassification,
    request::{BuildRequestOrigin, BuildSession, BuildSessionOutcome},
};

pub(super) fn replace_watch_snapshot(
    publisher: Option<&PhaseOutputPublisher>,
    old_snapshot: Option<BuildSnapshotId>,
    session: &BuildSession,
) -> WatchSnapshotReplacement {
    let new_snapshot = session.captured.snapshot.id;
    let Some(publisher) = publisher else {
        return WatchSnapshotReplacement {
            old_snapshot,
            new_snapshot,
            status: WatchSnapshotReplacementStatus::ExternalDependencyGap,
        };
    };
    let status = match old_snapshot {
        None => {
            publisher.register_current_snapshot(new_snapshot);
            WatchSnapshotReplacementStatus::RegisteredInitialSnapshot
        }
        Some(old_snapshot) if old_snapshot == new_snapshot => {
            WatchSnapshotReplacementStatus::SameSnapshot
        }
        Some(old_snapshot) => {
            match publisher.replace_current_snapshot(old_snapshot, new_snapshot) {
                Ok(()) => WatchSnapshotReplacementStatus::Replaced,
                Err(error) => WatchSnapshotReplacementStatus::Failed {
                    error: Box::new(error),
                },
            }
        }
    };
    WatchSnapshotReplacement {
        old_snapshot,
        new_snapshot,
        status,
    }
}

pub(super) fn watch_mode_gaps(control: &WatchSubmitControl<'_>) -> Vec<WatchModeGap> {
    let mut gaps = Vec::new();
    push_watch_seam_gap(
        &mut gaps,
        WatchModeGapOwner::FileWatcher,
        control.file_watcher,
    );
    push_watch_seam_gap(&mut gaps, WatchModeGapOwner::LspBridge, control.lsp_bridge);
    if control.output_publisher.is_none() {
        gaps.push(WatchModeGap {
            owner: WatchModeGapOwner::IrSnapshotReplacement,
            classification: OwnerGapClassification::ExternalDependencyGap,
        });
    }
    gaps
}

fn push_watch_seam_gap(
    gaps: &mut Vec<WatchModeGap>,
    owner: WatchModeGapOwner,
    seam: WatchOwnerSeam,
) {
    if let Some(classification) = watch_seam_classification(seam) {
        gaps.push(WatchModeGap {
            owner,
            classification,
        });
    }
}

fn watch_seam_classification(seam: WatchOwnerSeam) -> Option<OwnerGapClassification> {
    match seam {
        WatchOwnerSeam::OwnerProvided => None,
        WatchOwnerSeam::ExternalDependencyGap => {
            Some(OwnerGapClassification::ExternalDependencyGap)
        }
        WatchOwnerSeam::Deferred => Some(OwnerGapClassification::Deferred),
        WatchOwnerSeam::Unavailable => Some(OwnerGapClassification::Unavailable),
    }
}

pub(super) fn request_origin_name(origin: &BuildRequestOrigin) -> &'static str {
    match origin {
        BuildRequestOrigin::Batch(_) => "batch",
        BuildRequestOrigin::Watch(_) => "watch",
        BuildRequestOrigin::Lsp(_) => "lsp",
    }
}

pub(super) fn submit_error_session(error: &DriverSubmitError) -> Option<&BuildSession> {
    match error {
        DriverSubmitError::RequestIdAllocation { .. }
        | DriverSubmitError::SnapshotCapture { .. } => None,
        DriverSubmitError::Planning { session, .. }
        | DriverSubmitError::ModuleIndex { session, .. }
        | DriverSubmitError::TaskGraph { session, .. }
        | DriverSubmitError::PhaseRegistry { session, .. }
        | DriverSubmitError::Scheduler { session, .. } => Some(session.as_ref()),
    }
}

pub(super) fn cancellation_outcome(reason: DriverCancelReason) -> BuildSessionOutcome {
    match reason {
        DriverCancelReason::Superseded => BuildSessionOutcome::Superseded,
        DriverCancelReason::ExplicitRequest | DriverCancelReason::Shutdown => {
            BuildSessionOutcome::Cancelled
        }
    }
}
