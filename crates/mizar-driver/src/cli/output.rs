use mizar_build::{
    planner::ManifestDiagnostics,
    scheduler::{SchedulerEventKind, TaskState},
    task_graph::PipelinePhase,
};

use crate::{
    cli::{CliExitCode, CliInvocation, CliMessageFormat, CliOutput, CliUsageError},
    driver::{BuildSubmission, CompilerDriver, DriverSubmissionStatus, DriverSubmitError},
    events::{BuildEvent, BuildEventKind, BuildEventStream, EventOwner, OwnerGapClassification},
    registry::{PhaseServiceAvailability, PhaseStatus},
    request::{BuildSession, BuildSessionOutcome, BuildSessionState, PublicationDecision},
};

pub(super) fn usage_output(error: CliUsageError, format: CliMessageFormat) -> CliOutput {
    match format {
        CliMessageFormat::Human => CliOutput {
            exit_code: CliExitCode::Usage,
            stdout: String::new(),
            stderr: format!("mizar build: usage_error reason={}\n", error.reason),
        },
        CliMessageFormat::Json => CliOutput {
            exit_code: CliExitCode::Usage,
            stdout: format!(
                "{{\"schema_version\":1,\"kind\":\"usage_error\",\"reason\":\"{}\"}}\n",
                json_escape(&error.reason)
            ),
            stderr: String::new(),
        },
    }
}

pub(super) fn owner_diagnostics_gap_output(
    invocation: &CliInvocation,
    exit_code: CliExitCode,
    owner: &'static str,
    diagnostics: ManifestDiagnostics,
) -> CliOutput {
    let count = diagnostics.diagnostics().len();
    gap_output_with_diagnostics(
        invocation,
        exit_code,
        owner,
        "external_dependency_gap",
        Some(count),
    )
}

pub(super) fn gap_output(
    invocation: &CliInvocation,
    exit_code: CliExitCode,
    owner: &str,
    classification: &str,
) -> CliOutput {
    gap_output_with_diagnostics(invocation, exit_code, owner, classification, None)
}

fn gap_output_with_diagnostics(
    invocation: &CliInvocation,
    exit_code: CliExitCode,
    owner: &str,
    classification: &str,
    diagnostics: Option<usize>,
) -> CliOutput {
    match invocation.message_format {
        CliMessageFormat::Human => CliOutput {
            exit_code,
            stdout: String::new(),
            stderr: match diagnostics {
                Some(count) => format!(
                    "mizar build: owner_gap owner={owner} classification={classification} diagnostics={count}\n"
                ),
                None => format!(
                    "mizar build: owner_gap owner={owner} classification={classification}\n"
                ),
            },
        },
        CliMessageFormat::Json => CliOutput {
            exit_code,
            stdout: match diagnostics {
                Some(count) => format!(
                    "{{\"schema_version\":1,\"kind\":\"owner_gap\",\"owner\":\"{}\",\"classification\":\"{}\",\"diagnostics\":{count}}}\n",
                    json_escape(owner),
                    json_escape(classification)
                ),
                None => format!(
                    "{{\"schema_version\":1,\"kind\":\"owner_gap\",\"owner\":\"{}\",\"classification\":\"{}\"}}\n",
                    json_escape(owner),
                    json_escape(classification)
                ),
            },
            stderr: String::new(),
        },
    }
}

pub(super) fn output_from_submission(
    invocation: &CliInvocation,
    driver: &CompilerDriver,
    submission: &BuildSubmission,
) -> CliOutput {
    let exit_code = exit_code_for_submission(submission);
    render_driver_output(
        invocation,
        exit_code,
        driver.events(submission.session.id),
        None,
    )
}

pub(super) fn output_from_submit_error(
    invocation: &CliInvocation,
    driver: &CompilerDriver,
    error: DriverSubmitError,
) -> CliOutput {
    match error {
        DriverSubmitError::RequestIdAllocation { .. } => {
            internal_error_output(invocation, "request_id_allocation")
        }
        DriverSubmitError::SnapshotCapture { .. } => gap_output(
            invocation,
            CliExitCode::BuildFailed,
            "snapshot_capture",
            "external_dependency_gap",
        ),
        DriverSubmitError::Planning {
            session,
            diagnostics,
        } => render_driver_error_output(
            invocation,
            driver,
            &session,
            CliExitCode::BuildFailed,
            "planning",
            diagnostics.diagnostics().len(),
        ),
        DriverSubmitError::ModuleIndex {
            session,
            diagnostics,
        } => render_driver_error_output(
            invocation,
            driver,
            &session,
            CliExitCode::BuildFailed,
            "module_index",
            diagnostics.diagnostics().len(),
        ),
        DriverSubmitError::TaskGraph {
            session,
            diagnostics,
        } => render_driver_error_output(
            invocation,
            driver,
            &session,
            CliExitCode::BuildFailed,
            "task_graph",
            diagnostics.diagnostics().len(),
        ),
        DriverSubmitError::PhaseRegistry { session, .. } => render_driver_error_output(
            invocation,
            driver,
            &session,
            CliExitCode::InternalError,
            "phase_registry",
            1,
        ),
        DriverSubmitError::Scheduler {
            session,
            diagnostics,
        } => render_driver_error_output(
            invocation,
            driver,
            &session,
            CliExitCode::BuildFailed,
            "scheduler",
            diagnostics.diagnostics().len(),
        ),
    }
}

fn render_driver_error_output(
    invocation: &CliInvocation,
    driver: &CompilerDriver,
    session: &BuildSession,
    exit_code: CliExitCode,
    owner: &'static str,
    diagnostics_count: usize,
) -> CliOutput {
    render_driver_output(
        invocation,
        exit_code,
        driver.events(session.id),
        Some((owner, diagnostics_count)),
    )
}

fn internal_error_output(invocation: &CliInvocation, kind: &'static str) -> CliOutput {
    match invocation.message_format {
        CliMessageFormat::Human => CliOutput {
            exit_code: CliExitCode::InternalError,
            stdout: String::new(),
            stderr: format!("mizar build: internal_error kind={kind}\n"),
        },
        CliMessageFormat::Json => CliOutput {
            exit_code: CliExitCode::InternalError,
            stdout: format!(
                "{{\"schema_version\":1,\"kind\":\"internal_error\",\"error\":\"{kind}\"}}\n"
            ),
            stderr: String::new(),
        },
    }
}

fn render_driver_output(
    invocation: &CliInvocation,
    exit_code: CliExitCode,
    stream: BuildEventStream,
    diagnostics_gap: Option<(&'static str, usize)>,
) -> CliOutput {
    let mut stdout = String::new();
    let mut stderr = String::new();
    match invocation.message_format {
        CliMessageFormat::Human => {
            render_human_events(invocation, &stream, &mut stderr);
            if let Some((owner, count)) = diagnostics_gap {
                stderr.push_str(&format!(
                    "mizar build: owner_gap owner={owner} classification=external_dependency_gap diagnostics={count}\n"
                ));
            }
        }
        CliMessageFormat::Json => {
            render_json_events(invocation, &stream, &mut stdout);
            if let Some((owner, count)) = diagnostics_gap {
                stdout.push_str(&format!(
                    "{{\"schema_version\":1,\"kind\":\"owner_gap\",\"owner\":\"{owner}\",\"classification\":\"external_dependency_gap\",\"diagnostics\":{count}}}\n"
                ));
            }
        }
    }
    CliOutput {
        exit_code,
        stdout,
        stderr,
    }
}

fn render_human_events(invocation: &CliInvocation, stream: &BuildEventStream, output: &mut String) {
    for event in stream.events() {
        if invocation.quiet && !matches!(event.kind, BuildEventKind::SessionFinished { .. }) {
            continue;
        }
        output.push_str("mizar build: ");
        output.push_str(&human_event(event));
        output.push('\n');
    }
}

fn render_json_events(invocation: &CliInvocation, stream: &BuildEventStream, output: &mut String) {
    for event in stream.events() {
        if invocation.quiet && !matches!(event.kind, BuildEventKind::SessionFinished { .. }) {
            continue;
        }
        output.push_str(&json_event(event));
        output.push('\n');
    }
}

fn exit_code_for_submission(submission: &BuildSubmission) -> CliExitCode {
    match submission.status {
        DriverSubmissionStatus::BlockedByMissingPhaseServices
        | DriverSubmissionStatus::BlockedByPhaseDispatchGap => CliExitCode::UnavailableOwner,
        DriverSubmissionStatus::SupersededBeforeSubmission => CliExitCode::Cancelled,
        DriverSubmissionStatus::SchedulerValidated => match submission.session.state {
            BuildSessionState::Finished(BuildSessionOutcome::Succeeded) => CliExitCode::Success,
            BuildSessionState::Finished(
                BuildSessionOutcome::Cancelled | BuildSessionOutcome::Superseded,
            ) => CliExitCode::Cancelled,
            BuildSessionState::Finished(
                BuildSessionOutcome::Failed | BuildSessionOutcome::Blocked,
            ) => CliExitCode::BuildFailed,
            BuildSessionState::SnapshotCaptured
            | BuildSessionState::Submitted
            | BuildSessionState::Running
            | BuildSessionState::Cancelling => CliExitCode::InternalError,
        },
    }
}

fn human_event(event: &BuildEvent) -> String {
    match &event.kind {
        BuildEventKind::SessionAccepted => "session_accepted".to_owned(),
        BuildEventKind::SnapshotCaptured => "snapshot_captured".to_owned(),
        BuildEventKind::PlanningReady { status } => {
            format!("planning_ready status={}", planning_status_name(*status))
        }
        BuildEventKind::TaskProgress { task } => {
            let state = task.state.map(task_state_name).unwrap_or("none");
            format!(
                "task_progress task={} scheduler_event={} state={state}",
                task.task_id,
                scheduler_event_name(task.scheduler_event)
            )
        }
        BuildEventKind::PhaseServiceGap {
            phase,
            availability,
        } => format!(
            "phase_service_gap phase={} availability={}",
            phase_name(*phase),
            availability_name(*availability)
        ),
        BuildEventKind::DispatchGap { phases } => {
            format!("dispatch_gap phases={}", phase_list(phases))
        }
        BuildEventKind::OwnerReadinessGap {
            owner,
            classification,
        } => format!(
            "owner_gap owner={} classification={}",
            owner_name(*owner),
            owner_gap_name(*classification)
        ),
        BuildEventKind::PhaseReady { phase, status } => format!(
            "phase_ready phase={} status={}",
            phase_name(*phase),
            phase_status_name(*status)
        ),
        BuildEventKind::DiagnosticsReady { records } => format!(
            "diagnostics_ready owner={} identity={}",
            owner_name(records.owner),
            records.identity
        ),
        BuildEventKind::ArtifactBoundary { committed } => format!(
            "artifact_boundary owner={} identity={}",
            owner_name(committed.owner),
            committed.identity
        ),
        BuildEventKind::PublicationSuppressed => "publication_suppressed".to_owned(),
        BuildEventKind::SessionFinished { outcome } => {
            format!("session_finished outcome={}", outcome_name(*outcome))
        }
    }
}

fn json_event(event: &BuildEvent) -> String {
    let publication = publication_name(event.identity.publication);
    match &event.kind {
        BuildEventKind::SessionAccepted => {
            json_base_event("session_accepted", publication, String::new())
        }
        BuildEventKind::SnapshotCaptured => {
            json_base_event("snapshot_captured", publication, String::new())
        }
        BuildEventKind::PlanningReady { status } => json_base_event(
            "planning_ready",
            publication,
            format!(",\"status\":\"{}\"", planning_status_name(*status)),
        ),
        BuildEventKind::TaskProgress { task } => {
            let state = task.state.map(task_state_name).unwrap_or("none");
            json_base_event(
                "task_progress",
                publication,
                format!(
                    ",\"task\":\"{}\",\"scheduler_event\":\"{}\",\"state\":\"{}\"",
                    json_escape(&task.task_id),
                    scheduler_event_name(task.scheduler_event),
                    state
                ),
            )
        }
        BuildEventKind::PhaseServiceGap {
            phase,
            availability,
        } => json_base_event(
            "phase_service_gap",
            publication,
            format!(
                ",\"phase\":\"{}\",\"availability\":\"{}\"",
                phase_name(*phase),
                availability_name(*availability)
            ),
        ),
        BuildEventKind::DispatchGap { phases } => json_base_event(
            "dispatch_gap",
            publication,
            format!(",\"phases\":[{}]", json_phase_array(phases)),
        ),
        BuildEventKind::OwnerReadinessGap {
            owner,
            classification,
        } => json_base_event(
            "owner_gap",
            publication,
            format!(
                ",\"owner\":\"{}\",\"classification\":\"{}\"",
                owner_name(*owner),
                owner_gap_name(*classification)
            ),
        ),
        BuildEventKind::PhaseReady { phase, status } => json_base_event(
            "phase_ready",
            publication,
            format!(
                ",\"phase\":\"{}\",\"status\":\"{}\"",
                phase_name(*phase),
                phase_status_name(*status)
            ),
        ),
        BuildEventKind::DiagnosticsReady { records } => json_base_event(
            "diagnostics_ready",
            publication,
            format!(
                ",\"owner\":\"{}\",\"identity\":\"{}\"",
                owner_name(records.owner),
                json_escape(&records.identity)
            ),
        ),
        BuildEventKind::ArtifactBoundary { committed } => json_base_event(
            "artifact_boundary",
            publication,
            format!(
                ",\"owner\":\"{}\",\"identity\":\"{}\"",
                owner_name(committed.owner),
                json_escape(&committed.identity)
            ),
        ),
        BuildEventKind::PublicationSuppressed => {
            json_base_event("publication_suppressed", publication, String::new())
        }
        BuildEventKind::SessionFinished { outcome } => json_base_event(
            "session_finished",
            publication,
            format!(",\"outcome\":\"{}\"", outcome_name(*outcome)),
        ),
    }
}

fn json_base_event(kind: &'static str, publication: &'static str, fields: String) -> String {
    format!(
        "{{\"schema_version\":1,\"kind\":\"{kind}\",\"publication\":\"{publication}\"{fields}}}"
    )
}

fn phase_list(phases: &[PipelinePhase]) -> String {
    phases
        .iter()
        .copied()
        .map(phase_name)
        .collect::<Vec<_>>()
        .join(",")
}

fn json_phase_array(phases: &[PipelinePhase]) -> String {
    phases
        .iter()
        .copied()
        .map(|phase| format!("\"{}\"", phase_name(phase)))
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            other if other <= '\u{1f}' => push_json_control_escape(&mut escaped, other),
            other => escaped.push(other),
        }
    }
    escaped
}

fn push_json_control_escape(output: &mut String, ch: char) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let value = ch as usize;
    output.push_str("\\u00");
    output.push(HEX[(value >> 4) & 0x0f] as char);
    output.push(HEX[value & 0x0f] as char);
}

fn planning_status_name(status: crate::events::PlanningEventStatus) -> &'static str {
    match status {
        crate::events::PlanningEventStatus::Ready => "ready",
        crate::events::PlanningEventStatus::Failed => "failed",
    }
}

fn task_state_name(state: TaskState) -> &'static str {
    match state {
        TaskState::Pending => "pending",
        TaskState::Ready => "ready",
        TaskState::Running => "running",
        TaskState::Completed => "completed",
        TaskState::CacheHit => "cache_hit",
        TaskState::Skipped => "skipped",
        TaskState::Failed => "failed",
        TaskState::Blocked => "blocked",
        TaskState::Cancelled => "cancelled",
        _ => "unknown",
    }
}

fn scheduler_event_name(kind: SchedulerEventKind) -> &'static str {
    match kind {
        SchedulerEventKind::TaskBecameReady => "task_became_ready",
        SchedulerEventKind::TaskStarted => "task_started",
        SchedulerEventKind::TaskSkipped => "task_skipped",
        SchedulerEventKind::TaskBlocked => "task_blocked",
        SchedulerEventKind::TaskFinished => "task_finished",
        SchedulerEventKind::RunFinished => "run_finished",
        _ => "unknown",
    }
}

fn phase_name(phase: PipelinePhase) -> &'static str {
    match phase {
        PipelinePhase::PackageResolve => "package_resolve",
        PipelinePhase::SourceLoad => "source_load",
        PipelinePhase::Frontend => "frontend",
        PipelinePhase::ModuleResolve => "module_resolve",
        PipelinePhase::SignatureCollection => "signature_collection",
        PipelinePhase::TypeChecking => "type_checking",
        PipelinePhase::RegistrationResolution => "registration_resolution",
        PipelinePhase::OverloadResolution => "overload_resolution",
        PipelinePhase::Elaboration => "elaboration",
        PipelinePhase::AlgorithmPreparation => "algorithm_preparation",
        PipelinePhase::VcGenerate => "vc_generate",
        PipelinePhase::VcDischarge => "vc_discharge",
        PipelinePhase::AtpSolve => "atp_solve",
        PipelinePhase::BackendRun => "backend_run",
        PipelinePhase::KernelCheck => "kernel_check",
        PipelinePhase::ArtifactCommit => "artifact_commit",
        PipelinePhase::DocumentationExtract => "documentation_extract",
        _ => "unknown",
    }
}

fn availability_name(availability: PhaseServiceAvailability) -> &'static str {
    match availability {
        PhaseServiceAvailability::AvailableOwner => "available_owner",
        PhaseServiceAvailability::ExternalDependencyGap => "external_dependency_gap",
        PhaseServiceAvailability::Deferred => "deferred",
    }
}

fn phase_status_name(status: PhaseStatus) -> &'static str {
    match status {
        PhaseStatus::Complete => "complete",
        PhaseStatus::Recoverable => "recoverable",
        PhaseStatus::Blocking => "blocking",
        PhaseStatus::Fatal => "fatal",
        PhaseStatus::Cancelled => "cancelled",
    }
}

fn owner_name(owner: EventOwner) -> &'static str {
    match owner {
        EventOwner::Diagnostics => "diagnostics",
        EventOwner::Artifact => "artifact",
        EventOwner::ProducerOutput => "producer_output",
        EventOwner::LspBridge => "lsp_bridge",
    }
}

fn owner_gap_name(classification: OwnerGapClassification) -> &'static str {
    match classification {
        OwnerGapClassification::ExternalDependencyGap => "external_dependency_gap",
        OwnerGapClassification::Deferred => "deferred",
        OwnerGapClassification::Unavailable => "unavailable",
    }
}

fn outcome_name(outcome: BuildSessionOutcome) -> &'static str {
    match outcome {
        BuildSessionOutcome::Succeeded => "succeeded",
        BuildSessionOutcome::Failed => "failed",
        BuildSessionOutcome::Blocked => "blocked",
        BuildSessionOutcome::Cancelled => "cancelled",
        BuildSessionOutcome::Superseded => "superseded",
    }
}

fn publication_name(publication: PublicationDecision) -> &'static str {
    match publication {
        PublicationDecision::Current => "current",
        PublicationDecision::Suppressed(_) => "suppressed",
    }
}
