//! Per-connection admission, native capture handoff and original Driver event replay.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use mizar_build::module_index::{ModuleIndex, SourceLayoutProvider};
use mizar_driver::{
    driver::{
        BuildSubmission, CompilerDriver, DiagnosticIndexMode, DriverCancelReason,
        DriverSubmitError, DriverSubmitInput, SubmitOptions,
    },
    events::{BuildEvent, BuildEventKind},
    request::{
        BuildLaneId, BuildRequestDraft, BuildRequestGeneration, BuildRequestOrigin, LspPriority,
        LspRequest, PublicationDecision,
    },
};
use mizar_session::{
    BuildSessionId, DiskSourceLoader, PackageId, SessionIdAllocator, SnapshotRegistry,
};
use serde_json::Value;

use super::{NativeOpenBufferSnapshot, Server};

/// Capture failures consume admission; submission failures retain Driver's original details.
#[derive(Debug)]
pub enum BuildBridgeError {
    Capture(String),
    Submit(DriverSubmitError),
}

pub(super) struct BuildBridge {
    pub(super) lane: BuildLaneId,
    clock: Box<dyn Fn() -> Instant + Send + Sync>,
    pub(super) slot: Option<(Instant, Instant)>,
    generation: u64,
    current: Option<BuildSessionId>,
    pub(super) nonterminal: bool,
    pub(super) submitted: Vec<(
        BuildSessionId,
        BuildRequestGeneration,
        Arc<NativeOpenBufferSnapshot>,
    )>,
}

impl BuildBridge {
    fn running(&self, driver: &CompilerDriver) -> bool {
        self.current
            .and_then(|id| driver.session(id))
            .is_some_and(|session| !session.is_terminal())
    }

    pub(super) fn deadline(&self, running: bool) -> Option<Instant> {
        let (first, last) = self.slot?;
        let quiet = last + Duration::from_millis(300);
        Some(if running {
            quiet
        } else {
            quiet.min(first + Duration::from_secs(2))
        })
    }
}

impl Server {
    /// Configures one connection's exclusive LSP lane with a monotonic clock.
    /// Use a dedicated Driver when phase providers have an output publisher: its publisher
    /// must be distinct from Watch's. The bridge never replaces a caller's publisher.
    pub fn with_build_bridge(
        mut self,
        lane: BuildLaneId,
        clock: impl Fn() -> Instant + Send + Sync + 'static,
    ) -> Self {
        assert!(
            self.build_bridge.is_none(),
            "build bridge already configured"
        );
        self.build_bridge = Some(BuildBridge {
            lane,
            clock: Box::new(clock),
            slot: None,
            generation: 0,
            current: None,
            nonterminal: false,
            submitted: Vec::new(),
        });
        self
    }

    /// Configures captured acceptance separately from host-driven execution progress.
    /// Call `progress_build_bridge` between protocol messages, also after Shutdown/exit.
    pub fn with_nonterminal_build_bridge(
        self,
        lane: BuildLaneId,
        clock: impl Fn() -> Instant + Send + Sync + 'static,
    ) -> Self {
        let mut server = self.with_build_bridge(lane, clock);
        server
            .build_bridge
            .as_mut()
            .expect("configured bridge missing")
            .nonterminal = true;
        server
    }

    pub(super) fn admit_build(&mut self) {
        if let Some(bridge) = &mut self.build_bridge {
            let now = (bridge.clock)();
            match &mut bridge.slot {
                Some((_, last)) => *last = now,
                slot => *slot = Some((now, now)),
            }
        }
    }

    /// The host polls again at this deadline, or drives immediately on incoming traffic.
    pub fn next_build_deadline(&self, driver: &CompilerDriver) -> Option<Instant> {
        let bridge = self.build_bridge.as_ref()?;
        if bridge.nonterminal
            && bridge.submitted.iter().any(|(id, _, _)| {
                driver
                    .session(*id)
                    .is_some_and(|session| !session.is_terminal())
            })
        {
            return Some((bridge.clock)());
        }
        bridge.deadline(bridge.running(driver))
    }

    /// Advances each owned execution once and returns newly drained submissions.
    /// Admission and protocol lifetime do not prevent cleanup of outstanding work.
    pub fn progress_build_bridge<A: SessionIdAllocator>(
        &mut self,
        driver: &mut CompilerDriver,
        allocator: &A,
        snapshots: &SnapshotRegistry<A>,
    ) -> Result<Vec<BuildSubmission>, BuildBridgeError> {
        let mut finished = Vec::new();
        if let Some(bridge) = &self.build_bridge {
            for (id, _, _) in &bridge.submitted {
                if driver
                    .session(*id)
                    .is_some_and(|session| !session.is_terminal())
                    && let Some(submission) = driver
                        .progress(*id, allocator, snapshots)
                        .map_err(BuildBridgeError::Submit)?
                {
                    finished.push(submission);
                }
            }
        }
        Ok(finished)
    }

    /// Handles optional traffic, then flushes due admission without activating or publishing.
    /// `prepare` supplies current workspace coordinates for every open URI, never editor text.
    pub fn drive_build_bridge<A, L, F>(
        &mut self,
        message: Option<Value>,
        driver: &mut CompilerDriver,
        allocator: &A,
        snapshots: &Arc<SnapshotRegistry<A>>,
        publisher: &mizar_ir::publisher::PhaseOutputPublisher,
        prepare: F,
    ) -> (
        Option<Value>,
        Result<Option<BuildSubmission>, BuildBridgeError>,
    )
    where
        A: SessionIdAllocator + Send + Sync + 'static,
        L: SourceLayoutProvider,
        F: FnOnce(
            &Server,
            BuildRequestGeneration,
        ) -> Result<
            (
                BuildRequestDraft,
                ModuleIndex,
                Vec<(DiskSourceLoader, PackageId, String)>,
                DriverSubmitInput<L>,
            ),
            String,
        >,
    {
        let response = message
            .and_then(|message| self.handle_message_with_metadata(message, driver, snapshots));
        let result = (|| {
            let Some(bridge) = &mut self.build_bridge else {
                return Ok(None);
            };
            if self.shutdown || self.exit_code.is_some() {
                bridge.slot = None;
                if bridge.running(driver)
                    && bridge
                        .current
                        .and_then(|id| driver.session(id))
                        .is_some_and(|session| {
                            session.state() != mizar_driver::request::BuildSessionState::Cancelling
                        })
                {
                    driver.cancel(
                        bridge.current.expect("running session missing"),
                        DriverCancelReason::Shutdown,
                        snapshots,
                    );
                }
                return Ok(None);
            }
            if !bridge
                .deadline(bridge.running(driver))
                .is_some_and(|deadline| (bridge.clock)() >= deadline)
            {
                return Ok(None);
            }
            bridge.slot = None;
            if self.documents.is_empty() {
                return Ok(None);
            }
            bridge.generation = bridge
                .generation
                .checked_add(1)
                .ok_or_else(|| BuildBridgeError::Capture("build generation exhausted".into()))?;
            let generation = BuildRequestGeneration::new(bridge.generation);
            let lane = bridge.lane;
            let (mut draft, index, sources, input) =
                prepare(self, generation).map_err(BuildBridgeError::Capture)?;
            if sources.len() != self.documents.len()
                || sources
                    .iter()
                    .any(|(_, _, uri)| !self.documents.contains_key(uri))
            {
                return Err(BuildBridgeError::Capture(
                    "build capture requires every open URI".into(),
                ));
            }
            draft.lane = lane;
            draft.generation = generation;
            draft.origin = BuildRequestOrigin::Lsp(LspRequest {
                focus: None,
                priority: LspPriority::Normal,
            });
            let mut capture = self
                .capture_native_open_buffers(draft, allocator, snapshots, &index, sources)
                .map_err(BuildBridgeError::Capture)?;
            let association = capture.snapshot.clone();
            let snapshot = association.captured().snapshot().id;
            let previous = self
                .build_bridge
                .as_ref()
                .and_then(|bridge| bridge.current)
                .and_then(|id| driver.session(id))
                .map(|session| session.captured().snapshot().id);
            match previous {
                Some(previous) if previous != snapshot => {
                    publisher.replace_current_snapshot(previous, snapshot)
                }
                Some(_) => publisher.validate_current_snapshot(snapshot),
                None => {
                    publisher.register_current_snapshot(snapshot);
                    publisher.validate_current_snapshot(snapshot)
                }
            }
            .map_err(|error| BuildBridgeError::Capture(format!("{error:?}")))?;
            let session = capture
                .session
                .take()
                .expect("native capture session missing");
            let id = session.id();
            capture.release_active = None;
            let nonterminal = self
                .build_bridge
                .as_ref()
                .is_some_and(|bridge| bridge.nonterminal);
            let options = SubmitOptions {
                discover_imports: true,
                diagnostic_index: Some(DiagnosticIndexMode::Native),
                ..Default::default()
            };
            let submit = if nonterminal {
                // Unit tests retain passive checkpoint receipts on the same captured route.
                #[cfg(test)]
                let start = CompilerDriver::start_captured_observed;
                #[cfg(not(test))]
                let start = CompilerDriver::start_captured;
                start
            } else {
                CompilerDriver::submit_captured
            };
            let result = submit(driver, session, allocator, snapshots, input, options);
            // Stored sessions can include a rejected generation; only original lane-current
            // acceptance permits superseding the previous session, including accepted failures.
            let accepted = driver.session(id).is_some_and(|session| {
                session.request().lane == lane && session.request().generation == generation
            }) && driver.events(id).events().iter().any(|event| {
                matches!(
                    event.identity.publication,
                    PublicationDecision::Current
                        | PublicationDecision::Suppressed(
                            mizar_driver::request::ObsoletePublication {
                                lane_current: true,
                                ..
                            }
                        )
                )
            });
            let bridge = self
                .build_bridge
                .as_mut()
                .expect("configured bridge missing");
            if driver.session(id).is_some() {
                bridge.submitted.push((id, generation, association));
            }
            if accepted
                && let Some(previous) = bridge.current.replace(id)
                && driver
                    .session(previous)
                    .is_some_and(|session| !session.is_terminal())
            {
                driver.cancel(previous, DriverCancelReason::Superseded, snapshots);
            }
            result.map(Some).map_err(BuildBridgeError::Submit)
        })();
        (response, result)
    }

    /// Replays owned events with their original capture. Labels grant no renewed authority;
    /// only an actual SnapshotPublished payload is a coherent publication for downstream owners.
    pub fn build_bridge_events(
        &self,
        driver: &CompilerDriver,
    ) -> Vec<(BuildEvent, Arc<NativeOpenBufferSnapshot>)> {
        let Some(bridge) = &self.build_bridge else {
            return Vec::new();
        };
        let mut events = Vec::new();
        for (id, generation, association) in &bridge.submitted {
            let same_capture = |captured: &mizar_driver::request::CapturedSnapshot| {
                captured == association.captured()
                    && captured
                        .native_sources()
                        .zip(association.captured().native_sources())
                        .is_some_and(|(actual, original)| std::ptr::eq(actual, original))
            };
            let Some(session) = driver.session(*id) else {
                continue;
            };
            let stream = driver.events(*id);
            if !stream.known_session
                || stream.session != *id
                || session.request().lane != bridge.lane
                || session.request().generation != *generation
                || !same_capture(session.captured())
            {
                continue;
            }
            for event in stream.events() {
                if event.identity.session != *id
                    || event.identity.lane != bridge.lane
                    || event.identity.generation != *generation
                    || event
                        .identity
                        .snapshot
                        .is_some_and(|snapshot| snapshot != association.captured().snapshot().id)
                {
                    continue;
                }
                match &event.kind {
                    BuildEventKind::SnapshotPublished { snapshot }
                        if !same_capture(snapshot.captured()) =>
                    {
                        continue;
                    }
                    _ => {}
                }
                events.push((event.clone(), association.clone()));
            }
        }
        events
    }
}
