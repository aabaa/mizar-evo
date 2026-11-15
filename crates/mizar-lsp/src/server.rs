//! Connection lifecycle, full document buffers and bounded LSP stdio framing.

use std::{
    collections::BTreeMap,
    io::{self, BufRead, Write},
};

use mizar_ir::publisher::OutputAuthorization;
use serde_json::{Value, json};

#[path = "build_bridge.rs"]
mod build_bridge;
#[path = "completion.rs"]
mod completion;
#[path = "metadata.rs"]
mod metadata;
pub use build_bridge::BuildBridgeError;

#[cfg(test)]
#[path = "build_bridge_event_tests.rs"]
mod build_bridge_event_tests;
#[cfg(test)]
#[path = "build_bridge_timing_tests.rs"]
mod build_bridge_timing_tests;
#[cfg(test)]
#[path = "deferred_action_tests.rs"]
mod deferred_action_tests;
#[cfg(test)]
#[path = "explain_tests.rs"]
mod explain_tests;
#[cfg(test)]
#[path = "hover_tests.rs"]
mod hover_tests;
#[cfg(test)]
#[path = "semantic_token_tests.rs"]
mod semantic_token_tests;

/// One LSP connection and its exact client-owned document text.
#[derive(Default)]
pub struct Server {
    pub(crate) diagnostic_commands_enabled: bool,
    pub(crate) deferred_fixes: Vec<crate::diagnostics::RegisteredDeferredFix>,
    initialized: Option<Value>,
    shutdown: bool,
    exit_code: Option<i32>,
    documents: BTreeMap<String, (i32, String, std::sync::Arc<()>)>,
    native_open_buffers: Option<std::sync::Arc<NativeOpenBufferSnapshot>>,
    diagnostic_overlays: BTreeMap<(String, i32), std::sync::Arc<LspDiagnosticOverlay>>,
    lsp_snapshot: Option<std::sync::Arc<LspSnapshot>>,
    ranged_diagnostics: Option<std::sync::Arc<RangedDiagnosticPublication>>,
    build_bridge: Option<build_bridge::BuildBridge>,
}

/// A native capture tied to the exact Server open incarnations that supplied its text.
pub struct NativeOpenBufferCapture {
    session: Option<mizar_driver::request::BuildSession>,
    snapshot: std::sync::Arc<NativeOpenBufferSnapshot>,
    release_active: Option<Box<dyn FnOnce() + Send + Sync>>,
}

impl Drop for NativeOpenBufferCapture {
    fn drop(&mut self) {
        if let Some(release) = self.release_active.take() {
            release();
        }
    }
}

/// Immutable original native sources retained independently of current delivery.
pub struct NativeOpenBufferSnapshot {
    captured: mizar_driver::request::CapturedSnapshot,
    lifetimes: Vec<(String, std::sync::Weak<()>)>,
    release_overlay: Option<Box<dyn FnOnce() + Send + Sync>>,
}

impl NativeOpenBufferSnapshot {
    pub fn captured(&self) -> &mizar_driver::request::CapturedSnapshot {
        &self.captured
    }
}

impl Drop for NativeOpenBufferSnapshot {
    fn drop(&mut self) {
        self.release_overlay
            .take()
            .expect("native overlay cleanup missing")();
    }
}

/// Original Frontend records and source maps for one open document.
pub struct LspDiagnosticOverlay {
    index: std::sync::Arc<mizar_diagnostics::aggregator::BuildDiagnosticIndex>,
    association: std::sync::Arc<NativeOpenBufferSnapshot>,
    source_position: usize,
    session: mizar_session::BuildSessionId,
}

impl LspDiagnosticOverlay {
    pub fn document(&self) -> &str {
        let mizar_session::SourceOriginInput::OpenBuffer { uri, .. } = &self.source().2.origin
        else {
            unreachable!("diagnostic overlay requires an open buffer")
        };
        uri
    }

    pub fn version(&self) -> i32 {
        let mizar_session::SourceOriginInput::OpenBuffer { actual_version, .. } =
            &self.source().2.origin
        else {
            unreachable!("diagnostic overlay requires an open buffer")
        };
        i32::try_from(*actual_version).expect("eligible overlay version must fit i32")
    }

    pub fn diagnostics(
        &self,
    ) -> impl Iterator<Item = &mizar_diagnostics::failure_record::DiagnosticRecord> {
        self.index.records().iter().filter(|record| {
            matches!(record.primary_location(),
                mizar_diagnostics::failure_record::DiagnosticPrimaryLocation::Span(span)
                    if span.range().source_id == self.source().1.source_id)
        })
    }

    pub fn source(
        &self,
    ) -> &(
        mizar_session::DiskSourceLoader,
        mizar_session::LoadedSource,
        mizar_session::SourceInput,
    ) {
        &self
            .association
            .captured
            .native_sources()
            .expect("native overlay sources missing")[self.source_position]
    }
}

/// Editor freshness of a complete retained artifact read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspSnapshotFreshness {
    Current,
    StaleArtifact,
}

/// One original completed publication, its source association and independently pinned IR.
pub struct LspSnapshot {
    session: mizar_session::BuildSessionId,
    publication: std::sync::Arc<mizar_driver::events::SnapshotPublication>,
    native_open_buffers: std::sync::Arc<NativeOpenBufferSnapshot>,
    publisher: std::sync::Arc<mizar_ir::publisher::PhaseOutputPublisher>,
    outputs: Vec<mizar_ir::storage::AnyPhaseOutputRef>,
    retain_owner: mizar_ir::storage::RetainOwner,
    release_published: Option<Box<dyn FnOnce() + Send + Sync>>,
}

impl LspSnapshot {
    /// The producer's complete original payload, including its manifest-backed metadata.
    pub fn publication(&self) -> &std::sync::Arc<mizar_driver::events::SnapshotPublication> {
        &self.publication
    }

    pub fn native_open_buffers(&self) -> &std::sync::Arc<NativeOpenBufferSnapshot> {
        &self.native_open_buffers
    }

    pub fn outputs(&self) -> &[mizar_ir::storage::AnyPhaseOutputRef] {
        &self.outputs
    }

    pub(crate) fn publication_identity(&self) -> &str {
        self.retain_owner.as_str()
    }
}

/// One retained source-error publication; no manifest or semantic output authority.
pub(crate) struct RangedDiagnosticPublication {
    session: mizar_session::BuildSessionId,
    index: mizar_diagnostics::aggregator::BuildDiagnosticIndex,
    association: std::sync::Arc<NativeOpenBufferSnapshot>,
    identity: String,
    store: std::sync::Arc<mizar_diagnostics::explain::ExplanationStore>,
}

pub(crate) enum DiagnosticHostRead {
    Full(std::sync::Arc<LspSnapshot>),
    Ranged(std::sync::Arc<RangedDiagnosticPublication>),
}

impl DiagnosticHostRead {
    pub(crate) fn publication(&self) -> Option<&mizar_driver::events::SnapshotPublication> {
        match self {
            Self::Full(snapshot) => Some(snapshot.publication()),
            Self::Ranged(_) => None,
        }
    }

    pub(crate) fn diagnostics(&self) -> &mizar_diagnostics::aggregator::BuildDiagnosticIndex {
        match self {
            Self::Full(snapshot) => snapshot.publication().diagnostics(),
            Self::Ranged(snapshot) => &snapshot.index,
        }
    }

    pub(crate) fn native_open_buffers(&self) -> &std::sync::Arc<NativeOpenBufferSnapshot> {
        match self {
            Self::Full(snapshot) => snapshot.native_open_buffers(),
            Self::Ranged(snapshot) => &snapshot.association,
        }
    }

    pub(crate) fn publication_identity(&self) -> &str {
        match self {
            Self::Full(snapshot) => snapshot.publication_identity(),
            Self::Ranged(snapshot) => &snapshot.identity,
        }
    }

    pub(crate) fn same_publication(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Full(left), Self::Full(right)) => std::sync::Arc::ptr_eq(left, right),
            (Self::Ranged(left), Self::Ranged(right)) => std::sync::Arc::ptr_eq(left, right),
            _ => false,
        }
    }
}

impl Server {
    pub(crate) fn diagnostic_host<A: mizar_session::SessionIdAllocator>(
        &self,
        driver: &mizar_driver::driver::CompilerDriver,
        snapshots: &mizar_session::SnapshotRegistry<A>,
    ) -> Option<(DiagnosticHostRead, LspSnapshotFreshness)> {
        if let Some(selected) = &self.ranged_diagnostics {
            let current = self
                .native_open_buffer_snapshot()
                .is_some_and(|association| {
                    std::sync::Arc::ptr_eq(&association, &selected.association)
                })
                && driver
                    .validate_current_native_diagnostics(selected.session, snapshots)
                    .is_ok();
            return Some((
                DiagnosticHostRead::Ranged(selected.clone()),
                if current {
                    LspSnapshotFreshness::Current
                } else {
                    LspSnapshotFreshness::StaleArtifact
                },
            ));
        }
        self.lsp_snapshot(driver, snapshots)
            .map(|(snapshot, freshness)| (DiagnosticHostRead::Full(snapshot), freshness))
    }

    /// Retains only the ranged terminal diagnostics of this original native capture.
    /// This publishes no successful build, manifest, metadata or IR output.
    pub fn submit_originating_diagnostic_publication<
        A: mizar_session::SessionIdAllocator,
        L: mizar_build::module_index::SourceLayoutProvider,
    >(
        &mut self,
        driver: &mut mizar_driver::driver::CompilerDriver,
        mut capture: NativeOpenBufferCapture,
        allocator: &A,
        snapshots: &mizar_session::SnapshotRegistry<A>,
        input: mizar_driver::driver::DriverSubmitInput<L>,
    ) -> Result<
        (mizar_driver::driver::BuildSubmission, bool),
        mizar_driver::driver::DriverSubmitError,
    > {
        let association = capture.snapshot.clone();
        let eligible = self
            .native_open_buffer_snapshot()
            .is_some_and(|current| std::sync::Arc::ptr_eq(&current, &association));
        let session = capture
            .session
            .take()
            .expect("native capture session missing");
        capture.release_active = None;
        let (submission, notifications) =
            self.native_frontend_diagnostics(driver, session, allocator, snapshots, input)?;
        if !eligible || notifications.is_none() {
            return Ok((submission, false));
        }
        let Some(index) = submission.diagnostic_index.clone() else {
            return Ok((submission, false));
        };
        if crate::diagnostics::convert_index(&index, &association.captured).is_err() {
            return Ok((submission, false));
        }
        let sources = association
            .captured
            .native_sources()
            .expect("native sources missing");
        let mut store = mizar_diagnostics::explain::ExplanationStore::default();
        for record in index.records() {
            if let Some((_, loaded, _)) = sources.iter().find(|(_, loaded, _)| {
                matches!(record.primary_location(), mizar_diagnostics::failure_record::DiagnosticPrimaryLocation::Span(span)
                    if span.range().source_id == loaded.source_id)
            }) && let Some(payload) = mizar_driver::native_export_semicolon_context(record, loaded)
                && let Some(handle) = record.explanation()
            {
                store.insert_payload(handle, payload);
            }
        }
        static NEXT_DIAGNOSTIC_PUBLICATION: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(0);
        let Some(id) = NEXT_DIAGNOSTIC_PUBLICATION
            .try_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |id| id.checked_add(1),
            )
            .ok()
        else {
            return Ok((submission, false));
        };
        let candidate = std::sync::Arc::new(RangedDiagnosticPublication {
            session: submission.session.id(),
            index,
            association: association.clone(),
            identity: format!("lsp-diagnostic:{id}"),
            store: std::sync::Arc::new(store),
        });
        let mut registrations = Vec::new();
        for record in candidate.index.records() {
            let Some((_, loaded, _)) = sources.iter().find(|(_, loaded, _)| {
                matches!(record.primary_location(), mizar_diagnostics::failure_record::DiagnosticPrimaryLocation::Span(span)
                    if span.range().source_id == loaded.source_id)
            }) else { continue; };
            let Some(payload) = mizar_driver::native_export_semicolon_context(record, loaded)
            else {
                continue;
            };
            let Some(handle) = record.explanation() else {
                continue;
            };
            for fix in record
                .fixes()
                .iter()
                .filter(|fix| mizar_driver::native_export_semicolon_command(record, fix, loaded))
            {
                let bytes = payload.text().as_bytes().to_vec();
                let encoding = mizar_driver::NATIVE_EXPORT_SEMICOLON_ENCODING.to_owned();
                let registration = crate::diagnostics::DeferredFixRegistration {
                    text_hash_domain: None,
                    artifacts: Vec::new(),
                    explanation: Some(handle.clone()),
                    content: Some((encoding.clone(), bytes.clone())),
                    summary: Some((encoding, bytes)),
                    missing_facts: None,
                    command: fix.command().cloned(),
                    store: Some(candidate.store.clone()),
                    producer: std::sync::Arc::new((handle.clone(), payload.clone())),
                };
                if crate::diagnostics::validate_registration(
                    None,
                    &candidate.association.captured,
                    record,
                    fix,
                    &registration,
                ) {
                    registrations.push(crate::diagnostics::RegisteredDeferredFix {
                        publication: candidate.identity.clone(),
                        diagnostic: record.handle(),
                        fix: fix.clone(),
                        registration,
                    });
                }
            }
        }
        self.ranged_diagnostics = Some(candidate);
        self.deferred_fixes = registrations;
        Ok((submission, true))
    }

    /// Reads the selected producer-owned retained context without a caller-supplied store.
    pub fn handle_message_with_native_explanations<A: mizar_session::SessionIdAllocator>(
        &mut self,
        message: Value,
        driver: &mizar_driver::driver::CompilerDriver,
        snapshots: &mizar_session::SnapshotRegistry<A>,
        cancelled: &dyn Fn() -> bool,
    ) -> Option<Value> {
        let store = self
            .ranged_diagnostics
            .as_ref()
            .map(|selected| selected.store.clone())
            .unwrap_or_default();
        self.handle_message_with_explanations(message, driver, snapshots, &store, cancelled)
    }
}

impl Drop for LspSnapshot {
    fn drop(&mut self) {
        for output in &self.outputs {
            assert!(
                self.publisher
                    .storage()
                    .release(output.output(), &self.retain_owner)
                    .expect("LSP sealed output release failed"),
                "LSP sealed output retention missing"
            );
        }
        self.release_published
            .take()
            .expect("LSP published snapshot cleanup missing")();
    }
}

impl Server {
    /// Selects current Frontend diagnostics without granting semantic publication authority.
    pub fn diagnostic_overlay<A: mizar_session::SessionIdAllocator>(
        &self,
        uri: &str,
        version: i32,
        driver: &mizar_driver::driver::CompilerDriver,
        snapshots: &mizar_session::SnapshotRegistry<A>,
    ) -> Option<std::sync::Arc<LspDiagnosticOverlay>> {
        let overlay = self.diagnostic_overlays.get(&(uri.to_owned(), version))?;
        if self.initialized.is_none()
            || self.shutdown
            || self.exit_code.is_some()
            || !self.native_association_current(&overlay.association)
            || driver
                .validate_current_native_diagnostics(overlay.session, snapshots)
                .is_err()
        {
            return None;
        }
        Some(overlay.clone())
    }

    /// Stores only the original eligible index produced by the originating native capture.
    pub fn submit_originating_diagnostic_overlay<
        A: mizar_session::SessionIdAllocator,
        L: mizar_build::module_index::SourceLayoutProvider,
    >(
        &mut self,
        driver: &mut mizar_driver::driver::CompilerDriver,
        mut capture: NativeOpenBufferCapture,
        allocator: &A,
        snapshots: &mizar_session::SnapshotRegistry<A>,
        input: mizar_driver::driver::DriverSubmitInput<L>,
    ) -> Result<
        (mizar_driver::driver::BuildSubmission, bool),
        mizar_driver::driver::DriverSubmitError,
    > {
        let association = capture.snapshot.clone();
        let eligible = self
            .native_open_buffers
            .as_ref()
            .is_some_and(|current| std::sync::Arc::ptr_eq(current, &association))
            && self.native_association_current(&association);
        let session = capture
            .session
            .take()
            .expect("native capture session missing");
        capture.release_active = None;
        let (mut submission, notifications) =
            self.native_frontend_diagnostics(driver, session, allocator, snapshots, input)?;
        let index = submission.diagnostic_index.take();
        let installed = eligible && notifications.is_some();
        if installed {
            let index =
                std::sync::Arc::new(index.expect("eligible native diagnostic index missing"));
            for source_position in 0..association
                .captured
                .native_sources()
                .expect("native sources missing")
                .len()
            {
                let overlay = std::sync::Arc::new(LspDiagnosticOverlay {
                    index: index.clone(),
                    association: association.clone(),
                    source_position,
                    session: submission.session.id(),
                });
                self.diagnostic_overlays
                    .insert((overlay.document().to_owned(), overlay.version()), overlay);
            }
        }
        Ok((submission, installed))
    }

    /// Reads one whole retained publication and reports current editor/IR eligibility.
    pub fn lsp_snapshot<A: mizar_session::SessionIdAllocator>(
        &self,
        driver: &mizar_driver::driver::CompilerDriver,
        snapshots: &mizar_session::SnapshotRegistry<A>,
    ) -> Option<(std::sync::Arc<LspSnapshot>, LspSnapshotFreshness)> {
        let snapshot = self.lsp_snapshot.as_ref()?;
        let current = self.initialized.is_some()
            && !self.shutdown
            && self.exit_code.is_none()
            && driver
                .validate_current_native_diagnostics(snapshot.session, snapshots)
                .is_ok()
            && self.native_open_buffer_snapshot().is_some_and(|native| {
                std::sync::Arc::ptr_eq(&native, &snapshot.native_open_buffers)
            })
            && snapshot.outputs.iter().all(|output| {
                snapshot.publisher.storage().validate_handle(output).is_ok()
                    && snapshot
                        .publisher
                        .validate_output(
                            snapshot.publication.captured().snapshot().id,
                            output,
                            OutputAuthorization::Published,
                        )
                        .is_ok()
            });
        Some((
            snapshot.clone(),
            if current {
                LspSnapshotFreshness::Current
            } else {
                LspSnapshotFreshness::StaleArtifact
            },
        ))
    }

    /// Submits the original native capture and activates only its genuine complete publication.
    /// Supply the same publisher/storage used by this submission's configured phase providers;
    /// activation checks the original typed artifact objects against its actual output handles.
    /// LSP acquires its own Session lease and IR retain owner, without transferring Driver's owners.
    pub fn submit_originating_snapshot<
        A: mizar_session::SessionIdAllocator + Send + Sync + 'static,
        L: mizar_build::module_index::SourceLayoutProvider,
    >(
        &mut self,
        driver: &mut mizar_driver::driver::CompilerDriver,
        capture: NativeOpenBufferCapture,
        allocator: &A,
        snapshots: &std::sync::Arc<mizar_session::SnapshotRegistry<A>>,
        publisher: &std::sync::Arc<mizar_ir::publisher::PhaseOutputPublisher>,
        input: mizar_driver::driver::DriverSubmitInput<L>,
    ) -> Result<
        (mizar_driver::driver::BuildSubmission, bool),
        mizar_driver::driver::DriverSubmitError,
    > {
        let association = capture.snapshot.clone();
        let (submission, _) = self.submit_originating_native_frontend_diagnostics(
            driver, capture, allocator, snapshots, input,
        )?;
        let activated =
            self.activate_lsp_snapshot(driver, &submission, association, snapshots, publisher);
        Ok((submission, activated))
    }

    fn activate_lsp_snapshot<A: mizar_session::SessionIdAllocator + Send + Sync + 'static>(
        &mut self,
        driver: &mizar_driver::driver::CompilerDriver,
        submission: &mizar_driver::driver::BuildSubmission,
        association: std::sync::Arc<NativeOpenBufferSnapshot>,
        snapshots: &std::sync::Arc<mizar_session::SnapshotRegistry<A>>,
        publisher: &std::sync::Arc<mizar_ir::publisher::PhaseOutputPublisher>,
    ) -> bool {
        use mizar_driver::{events::BuildEventKind, request::PublicationDecision};
        let candidate = (|| {
            if !self
                .native_open_buffer_snapshot()
                .is_some_and(|current| std::sync::Arc::ptr_eq(&current, &association))
                || submission.publication_decision != PublicationDecision::Current
                || driver
                    .validate_current_native_diagnostics(submission.session.id(), snapshots)
                    .is_err()
            {
                return None;
            }
            let events = driver.events(submission.session.id());
            let mut publications = events
                .events()
                .iter()
                .filter_map(|event| match &event.kind {
                    BuildEventKind::SnapshotPublished { snapshot }
                        if event.identity.session == submission.session.id()
                            && event.identity.snapshot
                                == Some(association.captured.snapshot().id)
                            && event.identity.publication == PublicationDecision::Current =>
                    {
                        Some(snapshot)
                    }
                    _ => None,
                });
            let publication = publications.next()?;
            if publications.next().is_some()
                || publication.captured() != &association.captured
                || submission.session.captured() != &association.captured
                || !std::ptr::eq(
                    publication.captured().native_sources()?,
                    association.captured.native_sources()?,
                )
            {
                return None;
            }
            let run = submission.scheduler_run.as_ref()?;
            let commits = run
                .phase_results
                .values()
                .flatten()
                .filter(|result| result.artifact_commit.is_some())
                .collect::<Vec<_>>();
            // Consume the producer's original manifest reads before resolving any typed output.
            if publication.manifests().is_empty()
                || publication.manifests().len() != commits.len()
                || publication.manifests().len() != publication.artifacts().len()
                || publication.manifests().iter().any(|manifest| {
                    manifest.manifest.modules.len() != 1
                        || commits
                            .iter()
                            .filter(|result| {
                                result.artifact_commit.as_ref().is_some_and(|commit| {
                                    commit.manifest() == &manifest.manifest
                                        && commit.write().artifact_hash == manifest.artifact_hash
                                })
                            })
                            .count()
                            != 1
                })
            {
                return None;
            }
            for (manifest, artifact) in publication.manifests().iter().zip(publication.artifacts())
            {
                let result = commits.iter().find(|result| {
                    result
                        .artifact_commit
                        .as_ref()
                        .is_some_and(|commit| commit.manifest() == &manifest.manifest)
                })?;
                let [output] = result.output_refs.as_slice() else {
                    return None;
                };
                let handle = publisher
                    .storage()
                    .typed_handle(
                        output,
                        &mizar_ir::identity::OutputKind::new("SourceTheoremArtifactData"),
                    )
                    .ok()?;
                let original = publisher.storage().get(&handle).ok()?;
                if !std::sync::Arc::ptr_eq(&original, artifact)
                    || manifest.manifest.modules[0].module != artifact.module
                {
                    return None;
                }
            }
            let mut outputs = Vec::new();
            for output in run
                .phase_results
                .values()
                .flatten()
                .flat_map(|result| &result.output_refs)
            {
                publisher.storage().validate_handle(output).ok()?;
                publisher
                    .validate_output(
                        association.captured.snapshot().id,
                        output,
                        OutputAuthorization::Published,
                    )
                    .ok()?;
                if !outputs.contains(output) {
                    outputs.push(output.clone());
                }
            }
            if outputs.is_empty() {
                return None;
            }
            static NEXT_PUBLICATION: std::sync::atomic::AtomicU64 =
                std::sync::atomic::AtomicU64::new(0);
            let publication_id = NEXT_PUBLICATION
                .try_update(
                    std::sync::atomic::Ordering::Relaxed,
                    std::sync::atomic::Ordering::Relaxed,
                    |id| id.checked_add(1),
                )
                .ok()?;
            let lease = snapshots
                .acquire_lease(
                    association.captured.snapshot().id,
                    mizar_session::RetentionReason::PublishedLspSnapshot,
                )
                .ok()?;
            let retain_owner = mizar_ir::storage::RetainOwner::new(format!("lsp:{publication_id}"));
            let registry = snapshots.clone();
            let mut candidate = LspSnapshot {
                session: submission.session.id(),
                publication: publication.clone(),
                native_open_buffers: association,
                publisher: publisher.clone(),
                outputs: Vec::new(),
                retain_owner,
                release_published: Some(Box::new(move || {
                    registry
                        .release_lease(lease.snapshot, lease.lease_id)
                        .expect("LSP PublishedLspSnapshot lease release failed");
                })),
            };
            for output in outputs {
                publisher
                    .storage()
                    .retain(output.output(), candidate.retain_owner.clone())
                    .ok()?;
                candidate.outputs.push(output);
            }
            driver
                .validate_current_native_diagnostics(submission.session.id(), snapshots)
                .ok()?;
            for output in &candidate.outputs {
                publisher
                    .validate_output(
                        candidate.publication.captured().snapshot().id,
                        output,
                        OutputAuthorization::Published,
                    )
                    .ok()?;
            }
            Some(std::sync::Arc::new(candidate))
        })();
        let Some(candidate) = candidate else {
            return false;
        };
        for (_, _, input) in candidate
            .native_open_buffers
            .captured
            .native_sources()
            .expect("native sources missing")
        {
            if let mizar_session::SourceOriginInput::OpenBuffer {
                uri,
                actual_version,
                ..
            } = &input.origin
                && let Ok(version) = i32::try_from(*actual_version)
            {
                self.diagnostic_overlays.remove(&(uri.clone(), version));
            }
        }
        self.deferred_fixes.clear();
        self.ranged_diagnostics = None;
        self.lsp_snapshot = Some(candidate);
        true
    }

    /// Captures current raw editor text and its original open incarnations together.
    pub fn capture_native_open_buffers<
        A: mizar_session::SessionIdAllocator + Send + Sync + 'static,
    >(
        &mut self,
        draft: mizar_driver::request::BuildRequestDraft,
        allocator: &A,
        snapshots: &std::sync::Arc<mizar_session::SnapshotRegistry<A>>,
        index: &mizar_build::module_index::ModuleIndex,
        sources: Vec<(
            mizar_session::DiskSourceLoader,
            mizar_session::PackageId,
            String,
        )>,
    ) -> Result<NativeOpenBufferCapture, String> {
        if self.initialized.is_none() || self.shutdown || self.exit_code.is_some() {
            return Err("native buffer capture requires a live initialized Server".into());
        }
        if sources.is_empty() {
            return Err("native buffer capture requires sources".into());
        }
        let mut seen = std::collections::HashSet::new();
        let mut inputs = Vec::new();
        let mut lifetimes = Vec::new();
        for (loader, package, uri) in sources {
            if !seen.insert(uri.clone()) {
                return Err("duplicate native buffer URI".into());
            }
            let (version, text, lifetime) = self
                .documents
                .get(&uri)
                .ok_or("native buffer is not open")?;
            if *version < 0 {
                return Err("native buffer version must be nonnegative".into());
            }
            let version = i64::from(*version);
            let input = index.native_open_buffer_input(
                &draft.workspace_root,
                &package,
                &loader,
                mizar_session::SourceOriginInput::OpenBuffer {
                    uri: uri.clone(),
                    expected_version: version,
                    actual_version: version,
                    text: text.as_str().into(),
                },
            )?;
            lifetimes.push((uri, std::sync::Arc::downgrade(lifetime)));
            inputs.push((loader, input));
        }
        let session = draft
            .capture_sources(allocator, snapshots, inputs)
            .map_err(|error| format!("{error:?}"))?;
        let active = session.captured().active_snapshot_lease().clone();
        let active_registry = snapshots.clone();
        let release_active = Box::new(move || {
            active_registry
                .release_lease(active.snapshot, active.lease_id)
                .expect("native unsubmitted ActiveBuild lease release failed");
        });
        let overlay = match snapshots.acquire_lease(
            session.captured().snapshot().id,
            mizar_session::RetentionReason::OpenBufferOverlay,
        ) {
            Ok(overlay) => overlay,
            Err(error) => {
                release_active();
                return Err(format!("{error:?}"));
            }
        };
        let overlay_registry = snapshots.clone();
        let snapshot = std::sync::Arc::new(NativeOpenBufferSnapshot {
            captured: session.captured().clone(),
            lifetimes,
            release_overlay: Some(Box::new(move || {
                overlay_registry
                    .release_lease(overlay.snapshot, overlay.lease_id)
                    .expect("native OpenBufferOverlay lease release failed");
            })),
        });
        self.native_open_buffers = Some(snapshot.clone());
        Ok(NativeOpenBufferCapture {
            session: Some(session),
            snapshot,
            release_active: Some(release_active),
        })
    }

    /// Selects one complete original source cohort while its editor association is current.
    pub fn native_open_buffer_snapshot(&self) -> Option<std::sync::Arc<NativeOpenBufferSnapshot>> {
        let snapshot = self.native_open_buffers.as_ref()?;
        self.native_association_current(snapshot)
            .then(|| snapshot.clone())
    }

    fn native_association_current(&self, snapshot: &NativeOpenBufferSnapshot) -> bool {
        snapshot.lifetimes.iter().all(|(uri, original)| {
            self.document_lifetime(uri)
                .is_some_and(|current| std::sync::Weak::ptr_eq(original, &current))
        }) && snapshot.captured.native_sources().is_some_and(|sources| {
            sources.iter().all(|(_, _, input)| match &input.origin {
                mizar_session::SourceOriginInput::OpenBuffer {
                    uri,
                    expected_version,
                    actual_version,
                    text,
                } => {
                    expected_version == actual_version
                        && self.document(uri).is_some_and(|(version, raw)| {
                            i64::from(version) == *actual_version && raw == text.as_ref()
                        })
                }
                _ => false,
            })
        })
    }

    /// Submits the original capture, withholding notifications after an incarnation changes.
    pub fn submit_originating_native_frontend_diagnostics<
        A: mizar_session::SessionIdAllocator,
        L: mizar_build::module_index::SourceLayoutProvider,
    >(
        &self,
        driver: &mut mizar_driver::driver::CompilerDriver,
        mut capture: NativeOpenBufferCapture,
        allocator: &A,
        snapshots: &mizar_session::SnapshotRegistry<A>,
        input: mizar_driver::driver::DriverSubmitInput<L>,
    ) -> Result<
        (mizar_driver::driver::BuildSubmission, Option<Vec<Value>>),
        mizar_driver::driver::DriverSubmitError,
    > {
        let selected = self.native_open_buffer_snapshot();
        let eligible = selected
            .as_ref()
            .is_some_and(|current| std::sync::Arc::ptr_eq(current, &capture.snapshot));
        let session = capture
            .session
            .take()
            .expect("native capture session missing");
        capture.release_active = None;
        let (submission, notifications) =
            self.submit_native_frontend_diagnostics(driver, session, allocator, snapshots, input)?;
        Ok((submission, notifications.filter(|_| eligible)))
    }

    /// Delivers through the existing guarded wire producer only for original incarnations.
    pub fn submit_originating_native_frontend_diagnostics_to_writer<
        A: mizar_session::SessionIdAllocator,
        L: mizar_build::module_index::SourceLayoutProvider,
    >(
        &self,
        driver: &mut mizar_driver::driver::CompilerDriver,
        mut capture: NativeOpenBufferCapture,
        allocator: &A,
        snapshots: &mizar_session::SnapshotRegistry<A>,
        input: mizar_driver::driver::DriverSubmitInput<L>,
        output: &mut impl Write,
    ) -> Result<
        (mizar_driver::driver::BuildSubmission, io::Result<usize>),
        mizar_driver::driver::DriverSubmitError,
    > {
        let selected = self.native_open_buffer_snapshot();
        let eligible = selected
            .as_ref()
            .is_some_and(|current| std::sync::Arc::ptr_eq(current, &capture.snapshot));
        let session = capture
            .session
            .take()
            .expect("native capture session missing");
        capture.release_active = None;
        if eligible {
            self.submit_native_frontend_diagnostics_to_writer(
                driver, session, allocator, snapshots, input, output,
            )
        } else {
            let (submission, _) = self
                .submit_native_frontend_diagnostics(driver, session, allocator, snapshots, input)?;
            Ok((submission, Ok(0)))
        }
    }

    /// Borrows exact raw params from the first successful initialize request.
    /// These client values are not effective settings or compiler context.
    pub fn initialization_params(&self) -> Option<&Value> {
        self.initialized.as_ref()
    }

    /// Acquires current native metadata from the explicit initialize root URI.
    pub fn native_workspace_inputs(
        &self,
    ) -> Result<
        Option<(
            mizar_session::WorkspaceRoot,
            Vec<mizar_build::planner::WorkspacePackage>,
            mizar_build::planner::Lockfile,
        )>,
        String,
    > {
        let Some(uri) = self
            .initialization_params()
            .and_then(|params| params.get("rootUri"))
            .and_then(Value::as_str)
        else {
            return Ok(None);
        };
        let path = mizar_session::source::file_path_from_document_uri(uri)
            .ok_or_else(|| "workspace root must be a supported file URI".to_owned())?
            .components()
            .collect::<std::path::PathBuf>();
        if !path.is_absolute() {
            return Err("workspace root must be absolute".into());
        }
        if !std::fs::symlink_metadata(&path)
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            return Err("workspace root must be an ordinary directory".into());
        }
        let path = std::fs::canonicalize(path).map_err(|error| error.to_string())?;
        let root =
            mizar_session::WorkspaceRoot::new(path.to_str().ok_or("workspace root must be UTF-8")?);
        let (packages, lockfile) = mizar_build::planner::read_native_workspace_inputs(&root)?;
        Ok(Some((root, packages, lockfile)))
    }

    /// Borrows the current full-sync document version and text.
    pub fn document(&self, uri: &str) -> Option<(i32, &str)> {
        self.documents
            .get(uri)
            .map(|(version, text, ..)| (*version, text.as_str()))
    }

    /// Returns this live connection's open incarnation, independent of version/compiler state.
    /// Compare with a fresh query using `Weak::ptr_eq`; upgrading does not prove currentness.
    pub fn document_lifetime(&self, uri: &str) -> Option<std::sync::Weak<()>> {
        if self.shutdown || self.exit_code.is_some() {
            return None;
        }
        self.documents
            .get(uri)
            .map(|(_, _, lifetime)| std::sync::Arc::downgrade(lifetime))
    }

    /// Discovers imports from the native capture before frontend diagnostic submission.
    /// Returns current all-OpenBuffer notifications through the existing eligibility checks.
    pub fn submit_native_frontend_diagnostics<
        A: mizar_session::SessionIdAllocator,
        L: mizar_build::module_index::SourceLayoutProvider,
    >(
        &self,
        driver: &mut mizar_driver::driver::CompilerDriver,
        session: mizar_driver::request::BuildSession,
        allocator: &A,
        snapshots: &mizar_session::SnapshotRegistry<A>,
        input: mizar_driver::driver::DriverSubmitInput<L>,
    ) -> Result<
        (mizar_driver::driver::BuildSubmission, Option<Vec<Value>>),
        mizar_driver::driver::DriverSubmitError,
    > {
        let (mut submission, notifications) =
            self.native_frontend_diagnostics(driver, session, allocator, snapshots, input)?;
        submission.diagnostic_index = None;
        Ok((submission, notifications))
    }

    fn native_frontend_diagnostics<
        A: mizar_session::SessionIdAllocator,
        L: mizar_build::module_index::SourceLayoutProvider,
    >(
        &self,
        driver: &mut mizar_driver::driver::CompilerDriver,
        session: mizar_driver::request::BuildSession,
        allocator: &A,
        snapshots: &mizar_session::SnapshotRegistry<A>,
        input: mizar_driver::driver::DriverSubmitInput<L>,
    ) -> Result<
        (mizar_driver::driver::BuildSubmission, Option<Vec<Value>>),
        mizar_driver::driver::DriverSubmitError,
    > {
        use mizar_build::task_graph::{PipelinePhase as BuildPhase, TaskKind, WorkUnit};
        use mizar_diagnostics::{
            failure_record::{DiagnosticFreshness, DiagnosticPrimaryLocation, PipelinePhase},
            registry::DiagnosticSeverity,
        };
        use mizar_driver::registry::PhaseStatus;
        let submission = driver.submit_captured(
            session,
            allocator,
            snapshots,
            input,
            mizar_driver::driver::SubmitOptions {
                discover_imports: true,
                diagnostic_index: Some(mizar_driver::driver::DiagnosticIndexMode::Native),
                ..Default::default()
            },
        )?;
        let notifications = (|| {
            if self.initialized.is_none() || self.shutdown || self.exit_code.is_some() {
                return None;
            }
            let index = submission.diagnostic_index.as_ref()?;
            let snapshot = submission.session.captured().snapshot().id;
            if !index.obsolete_drafts().is_empty() {
                return None;
            }
            driver
                .validate_current_native_diagnostics(submission.session.id(), snapshots)
                .ok()?;
            let sources = driver
                .session(submission.session.id())?
                .captured()
                .native_sources()?;
            if sources.is_empty() {
                return None;
            }
            let graph = submission.task_graph.as_ref()?;
            let run = submission.scheduler_run.as_ref()?;
            let mut by_source = std::collections::HashMap::new();
            let mut by_uri = BTreeMap::new();
            for (_, loaded, input) in sources {
                let mizar_session::SourceOriginInput::OpenBuffer {
                    uri,
                    actual_version,
                    text,
                    ..
                } = &input.origin
                else {
                    return None;
                };
                let version = i32::try_from(*actual_version).ok()?;
                if self.document(uri) != Some((version, text.as_ref()))
                    || by_source
                        .insert(loaded.source_id, (loaded, input, uri))
                        .is_some()
                    || by_uri.insert(uri, (version, Vec::new())).is_some()
                {
                    return None;
                }
                // Clean captures must also have valid original maps and version context.
                crate::lsp_range_from_open_buffer(
                    &loaded.line_map,
                    loaded.loading_map.as_ref(),
                    &input.origin,
                    mizar_session::SourceRange {
                        source_id: loaded.source_id,
                        start: 0,
                        end: 0,
                    },
                )
                .ok()?;
                let mut tasks = graph.tasks().iter().filter(|task| {
                    task.kind == TaskKind::Frontend
                        && matches!(&task.unit, WorkUnit::Module { module }
                            if module.package == input.package_id && module.path == input.module_path)
                });
                let task = tasks.next()?;
                if tasks.next().is_some() || task.phases != [BuildPhase::Frontend] {
                    return None;
                }
                let [result] = run.phase_results.get(&task.id)?.as_slice() else {
                    return None;
                };
                if !matches!(
                    result.status,
                    PhaseStatus::Complete | PhaseStatus::Recoverable | PhaseStatus::Fatal
                ) || (result.status != PhaseStatus::Complete
                    && !index.records().iter().any(|record| {
                        matches!(record.primary_location(), DiagnosticPrimaryLocation::Span(span)
                            if span.range().source_id == loaded.source_id)
                    }))
                {
                    return None;
                }
            }
            for record in index.records() {
                if record.phase() != PipelinePhase::Frontend
                    || record.freshness()
                        != &(DiagnosticFreshness::Current {
                            source_snapshot: snapshot,
                        })
                {
                    return None;
                }
                let DiagnosticPrimaryLocation::Span(span) = record.primary_location() else {
                    return None;
                };
                let (loaded, input, uri) = by_source.get(&span.range().source_id)?;
                let range = crate::lsp_range_from_open_buffer(
                    &loaded.line_map,
                    loaded.loading_map.as_ref(),
                    &input.origin,
                    span.range(),
                )
                .ok()?;
                let severity = match record.severity() {
                    DiagnosticSeverity::Error => 1,
                    DiagnosticSeverity::Warning => 2,
                    DiagnosticSeverity::Info => 3,
                    _ => return None,
                };
                by_uri.get_mut(uri)?.1.push(json!({
                    "range": {"start": {"line": range.start.line, "character": range.start.character},
                        "end": {"line": range.end.line, "character": range.end.character}},
                    "code": record.code().to_string(), "message": record.message(), "severity": severity
                }));
            }
            let notifications = by_uri
                .into_iter()
                .map(|(uri, (version, diagnostics))| {
                    json!({
                        "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                        "params": {"uri": uri, "version": version, "diagnostics": diagnostics}
                    })
                })
                .collect();
            driver
                .validate_current_native_diagnostics(submission.session.id(), snapshots)
                .ok()?;
            Some(notifications)
        })();
        Ok((submission, notifications))
    }

    /// Submits native frontend diagnostics and writes each currently eligible LSP frame.
    pub fn submit_native_frontend_diagnostics_to_writer<
        A: mizar_session::SessionIdAllocator,
        L: mizar_build::module_index::SourceLayoutProvider,
    >(
        &self,
        driver: &mut mizar_driver::driver::CompilerDriver,
        session: mizar_driver::request::BuildSession,
        allocator: &A,
        snapshots: &mizar_session::SnapshotRegistry<A>,
        input: mizar_driver::driver::DriverSubmitInput<L>,
        output: &mut impl Write,
    ) -> Result<
        (mizar_driver::driver::BuildSubmission, io::Result<usize>),
        mizar_driver::driver::DriverSubmitError,
    > {
        let (submission, notifications) =
            self.submit_native_frontend_diagnostics(driver, session, allocator, snapshots, input)?;
        let delivery = (|| {
            let mut count = 0;
            for notification in notifications.into_iter().flatten() {
                let body = serde_json::to_vec(&notification).map_err(io::Error::other)?;
                if driver
                    .validate_current_native_diagnostics(submission.session.id(), snapshots)
                    .is_err()
                {
                    break;
                }
                write!(output, "Content-Length: {}\r\n\r\n", body.len())?;
                output.write_all(&body)?;
                output.flush()?;
                count += 1;
            }
            Ok(count)
        })();
        Ok((submission, delivery))
    }

    /// Handles a JSON-RPC message without creating compiler snapshots.
    pub fn handle_message(&mut self, message: Value) -> Option<Value> {
        if self.exit_code.is_some() {
            return None;
        }
        let error = |id: Value, code, text| {
            Some(json!({"jsonrpc": "2.0", "id": id,
                "error": {"code": code, "message": text}}))
        };
        let valid_id =
            |id: &Value| id.is_string() || id.as_i64().is_some_and(|id| i32::try_from(id).is_ok());
        if !message.is_object() || message.get("jsonrpc") != Some(&json!("2.0")) {
            return error(Value::Null, -32600, "Invalid Request");
        }
        if message.get("method").is_none() {
            let result = message.get("result").is_some();
            let response_error = message.get("error");
            if message
                .get("id")
                .is_some_and(|id| id.is_null() || valid_id(id))
                && result != response_error.is_some()
                && response_error.is_none_or(|value| {
                    value.is_object()
                        && value.get("code").is_some_and(valid_id)
                        && value.get("code").is_some_and(Value::is_number)
                        && value.get("message").is_some_and(Value::is_string)
                })
            {
                return None;
            }
            return error(Value::Null, -32600, "Invalid Request");
        }
        let Some(method) = message.get("method").and_then(Value::as_str) else {
            return error(Value::Null, -32600, "Invalid Request");
        };
        if message.get("result").is_some() || message.get("error").is_some() {
            return error(Value::Null, -32600, "Invalid Request");
        }
        let params = message.get("params");
        if let Some(id) = message.get("id") {
            if !valid_id(id) {
                return error(Value::Null, -32600, "Invalid Request");
            }
            let result = if method == "initialize" {
                if self.initialized.is_some() {
                    return error(id.clone(), -32600, "Invalid Request");
                }
                let Some(params) = params.filter(|params| params.is_object()) else {
                    return error(id.clone(), -32602, "Invalid params");
                };
                if !params.get("processId").is_some_and(|id| {
                    id.is_null() || id.as_i64().is_some_and(|id| i32::try_from(id).is_ok())
                }) || !params
                    .get("rootUri")
                    .is_some_and(|uri| uri.is_null() || uri.is_string())
                    || !params.get("capabilities").is_some_and(Value::is_object)
                {
                    return error(id.clone(), -32602, "Invalid params");
                }
                self.initialized = Some(params.clone());
                let mut result = json!({"capabilities": {"positionEncoding": "utf-16", "hoverProvider": true, "completionProvider": {"triggerCharacters": ["."]}, "codeActionProvider": {"codeActionKinds": ["quickfix"], "resolveProvider": true},
                    "semanticTokensProvider": {"legend": {"tokenTypes": ["keyword", "operator", "number", "comment"], "tokenModifiers": []}, "full": true},
                    "textDocumentSync": {"openClose": true, "change": 1}}});
                if self.diagnostic_commands_enabled {
                    result["capabilities"]["executeCommandProvider"] =
                        json!({"commands":["mizar.executeDiagnosticCommand"]});
                }
                result
            } else {
                if self.initialized.is_none() {
                    return error(id.clone(), -32002, "Server not initialized");
                }
                if self.shutdown {
                    return error(id.clone(), -32600, "Invalid Request");
                }
                match method {
                    "workspace/executeCommand" => {
                        if !self.diagnostic_commands_enabled {
                            return error(id.clone(), -32601, "Method not found");
                        }
                        let Some(params) = params.filter(|params| params.is_object()) else {
                            return error(id.clone(), -32602, "Invalid params");
                        };
                        let Some(command) = params["command"].as_str() else {
                            return error(id.clone(), -32602, "Invalid params");
                        };
                        if command != "mizar.executeDiagnosticCommand" {
                            return error(id.clone(), -32601, "Method not found");
                        }
                        if !params["arguments"].as_array().is_some_and(|arguments| {
                            arguments.len() == 1
                                && arguments[0].is_object()
                                && ["publication", "snapshot", "fix", "uri"]
                                    .iter()
                                    .all(|field| {
                                        arguments[0][field]
                                            .as_str()
                                            .is_some_and(|value| !value.is_empty())
                                    })
                                && arguments[0]["id"].is_u64()
                                && arguments[0]["version"]
                                    .as_i64()
                                    .is_some_and(|version| i32::try_from(version).is_ok())
                        }) {
                            return error(id.clone(), -32602, "Invalid params");
                        }
                        return error(
                            id.clone(),
                            -32801,
                            "Diagnostic command is no longer current",
                        );
                    }
                    "mizar/explain" => {
                        if !params.is_some_and(crate::explain::valid_explanation_params) {
                            return error(id.clone(), -32602, "Invalid params");
                        }
                        crate::explain::unavailable("no_backing_host")
                    }
                    "textDocument/semanticTokens/full" => {
                        let Some(uri) = params
                            .and_then(|params| params.pointer("/textDocument/uri"))
                            .and_then(Value::as_str)
                            .filter(|uri| !uri.is_empty())
                        else {
                            return error(id.clone(), -32602, "Invalid params");
                        };
                        let Some((_, text, _)) = self.documents.get(uri) else {
                            return Some(json!({"jsonrpc": "2.0", "id": id, "result": null}));
                        };
                        // The lexer ends line comments at LF. This byte-preserving
                        // view also terminates them at raw CR; output uses raw text.
                        let source =
                            mizar_lexer::preprocess_source_for_lexing(&text.replace('\r', "\n"));
                        let mut spans = source
                            .comments
                            .iter()
                            .map(|comment| (comment.span, 3u32))
                            .collect::<Vec<_>>();
                        // Keep argument-string context across raw chunks: layout
                        // inside a literal must not expose its contents as code.
                        let (mut quote, mut escaped, mut previous) = (None, false, None);
                        for raw in mizar_lexer::scan_raw_recoverable(&source.lexical_text).tokens {
                            let mut string_content = quote.is_some();
                            for ch in raw.lexeme.chars() {
                                if let Some(delimiter) = quote {
                                    if matches!(ch, '\n' | '\r') {
                                        quote = None;
                                        escaped = false;
                                    } else if escaped {
                                        escaped = false;
                                    } else if ch == '\\' {
                                        escaped = true;
                                    } else if ch == delimiter {
                                        quote = None;
                                    }
                                } else if matches!(ch, '"' | '\'')
                                    && matches!(previous, Some('(' | ','))
                                {
                                    quote = Some(ch);
                                    string_content = true;
                                }
                                if !matches!(ch, ' ' | '\t' | '\r' | '\n') {
                                    previous = Some(ch);
                                }
                            }
                            if string_content
                                || matches!(
                                    raw.kind,
                                    mizar_lexer::RawTokenKind::Layout
                                        | mizar_lexer::RawTokenKind::Error
                                )
                            {
                                continue;
                            }
                            let Ok(tokens) = mizar_lexer::lex(&raw.lexeme) else {
                                continue;
                            };
                            for token in tokens {
                                let kind = match token.kind {
                                    mizar_lexer::TokenKind::ReservedWord => 0,
                                    mizar_lexer::TokenKind::ReservedSymbol => 1,
                                    mizar_lexer::TokenKind::Numeral => 2,
                                    _ => continue,
                                };
                                let span = mizar_lexer::SourceSpan::new(
                                    raw.span.start + token.span.start,
                                    raw.span.start + token.span.end,
                                );
                                let Some(ranges) =
                                    source.preprocess_map.source_ranges_for_lexical(span)
                                else {
                                    return error(id.clone(), -32603, "Invalid token source map");
                                };
                                spans.extend(ranges.into_iter().map(|span| (span, kind)));
                            }
                        }
                        spans.sort_by_key(|(span, kind)| (span.start, span.end, *kind));
                        // Lines belong to this exact raw buffer; CRLF is one break.
                        let mut lines = Vec::new();
                        let mut start = 0;
                        while start < text.len() {
                            let end = text[start..]
                                .find(['\r', '\n'])
                                .map_or(text.len(), |offset| start + offset);
                            lines.push((start, end));
                            start = end;
                            if text.as_bytes().get(start) == Some(&b'\r') {
                                start += 1;
                            }
                            if text.as_bytes().get(start) == Some(&b'\n') {
                                start += 1;
                            }
                        }
                        let mut data = Vec::new();
                        let (mut previous_line, mut previous_column) = (0, 0);
                        for (span, kind) in spans {
                            if text.get(span.start..span.end).is_none() {
                                return error(id.clone(), -32603, "Invalid token source range");
                            }
                            let first = lines.partition_point(|(_, end)| *end <= span.start);
                            for (line, &(line_start, line_end)) in lines
                                .iter()
                                .enumerate()
                                .skip(first)
                                .take_while(|(_, (line_start, _))| *line_start < span.end)
                            {
                                let start = span.start.max(line_start);
                                let end = span.end.min(line_end);
                                if start >= end {
                                    continue;
                                }
                                let column = text[line_start..start].encode_utf16().count();
                                let length = text[start..end].encode_utf16().count();
                                let coordinates = [line, column, length].map(|value| {
                                    u32::try_from(value)
                                        .ok()
                                        .filter(|value| *value <= i32::MAX as u32)
                                });
                                let [Some(line), Some(column), Some(length)] = coordinates else {
                                    return error(id.clone(), -32603, "Token position overflow");
                                };
                                let delta_line = line - previous_line;
                                let delta_column = if delta_line == 0 {
                                    column - previous_column
                                } else {
                                    column
                                };
                                data.extend([delta_line, delta_column, length, kind, 0]);
                                (previous_line, previous_column) = (line, column);
                            }
                        }
                        json!({"data": data})
                    }
                    "textDocument/codeAction" => {
                        let Some(params) = params.filter(|params| params.is_object()) else {
                            return error(id.clone(), -32602, "Invalid params");
                        };
                        let Some(uri) = params
                            .pointer("/textDocument/uri")
                            .and_then(Value::as_str)
                            .filter(|uri| !uri.is_empty())
                        else {
                            return error(id.clone(), -32602, "Invalid params");
                        };
                        let range = &params["range"];
                        let valid = ["start", "end"].iter().all(|endpoint| {
                            ["line", "character"].iter().all(|field| {
                                range[endpoint][field]
                                    .as_u64()
                                    .is_some_and(|value| value <= i32::MAX as u64)
                            })
                        }) && (
                            range["start"]["line"].as_u64(),
                            range["start"]["character"].as_u64(),
                        ) <= (
                            range["end"]["line"].as_u64(),
                            range["end"]["character"].as_u64(),
                        ) && params["context"]["diagnostics"].is_array()
                            && params["context"].get("only").is_none_or(|only| {
                                only.as_array()
                                    .is_some_and(|kinds| kinds.iter().all(Value::is_string))
                            })
                            && params["context"]
                                .get("triggerKind")
                                .is_none_or(|kind| matches!(kind.as_u64(), Some(1 | 2)))
                            && self.document(uri).is_none_or(|(_, text)| {
                                crate::diagnostics::code_action_range(text, range).is_some()
                            });
                        if !valid {
                            return error(id.clone(), -32602, "Invalid params");
                        }
                        json!([])
                    }
                    "codeAction/resolve" => {
                        let valid = params.is_some_and(|params| {
                            let data = &params["data"];
                            params["title"].is_string()
                                && ["publication", "snapshot", "fix", "uri"]
                                    .iter()
                                    .all(|field| {
                                        data[field].as_str().is_some_and(|value| !value.is_empty())
                                    })
                                && data["id"].as_u64().is_some()
                                && data["version"]
                                    .as_i64()
                                    .is_some_and(|version| i32::try_from(version).is_ok())
                        });
                        if !valid {
                            return error(id.clone(), -32602, "Invalid params");
                        }
                        return error(id.clone(), -32801, "Code action is no longer current");
                    }
                    "textDocument/completion" => {
                        let valid = params.is_some_and(|params| {
                            params
                                .pointer("/textDocument/uri")
                                .and_then(Value::as_str)
                                .is_some_and(|uri| !uri.is_empty())
                                && ["line", "character"].iter().all(|field| {
                                    params
                                        .get("position")
                                        .and_then(|position| position.get(field))
                                        .and_then(Value::as_u64)
                                        .is_some_and(|value| value <= i32::MAX as u64)
                                })
                                && params.get("context").is_none_or(|context| {
                                    context.is_object()
                                        && matches!(context["triggerKind"].as_u64(), Some(1..=3))
                                        && context
                                            .get("triggerCharacter")
                                            .is_none_or(Value::is_string)
                                        && (context["triggerKind"] != 2
                                            || context["triggerCharacter"]
                                                .as_str()
                                                .is_some_and(|value| value.chars().count() == 1))
                                })
                        });
                        if !valid {
                            return error(id.clone(), -32602, "Invalid params");
                        }
                        let params = params.expect("validated params");
                        json!(
                            self.completion(
                                params["textDocument"]["uri"]
                                    .as_str()
                                    .expect("validated URI"),
                                crate::LspPosition {
                                    line: params["position"]["line"]
                                        .as_u64()
                                        .expect("validated line")
                                        as u32,
                                    character: params["position"]["character"]
                                        .as_u64()
                                        .expect("validated character")
                                        as u32,
                                }
                            )
                        )
                    }
                    "textDocument/hover" => {
                        let valid = params.is_some_and(|params| {
                            params
                                .pointer("/textDocument/uri")
                                .and_then(Value::as_str)
                                .is_some_and(|uri| !uri.is_empty())
                                && ["line", "character"].iter().all(|field| {
                                    params
                                        .get("position")
                                        .and_then(|position| position.get(field))
                                        .and_then(Value::as_u64)
                                        .is_some_and(|value| value <= i32::MAX as u64)
                                })
                        });
                        if !valid {
                            return error(id.clone(), -32602, "Invalid params");
                        }
                        Value::Null
                    }
                    "shutdown" => {
                        if !params.is_none_or(Value::is_null) {
                            return error(id.clone(), -32602, "Invalid params");
                        }
                        self.shutdown = true;
                        self.deferred_fixes.clear();
                        self.native_open_buffers = None;
                        self.diagnostic_overlays.clear();
                        if let Some(bridge) = &mut self.build_bridge {
                            bridge.slot = None;
                        }
                        Value::Null
                    }
                    "exit"
                    | "initialized"
                    | "textDocument/didOpen"
                    | "textDocument/didChange"
                    | "textDocument/didClose" => {
                        return error(id.clone(), -32600, "Invalid Request");
                    }
                    _ => {
                        if !method.starts_with("$/")
                            && params
                                .is_some_and(|params| !params.is_object() && !params.is_array())
                        {
                            return error(id.clone(), -32602, "Invalid params");
                        }
                        return error(id.clone(), -32601, "Method not found");
                    }
                }
            };
            return Some(json!({"jsonrpc": "2.0", "id": id, "result": result}));
        }
        if method == "exit" {
            if params.is_none_or(Value::is_null) {
                self.exit_code = Some(if self.shutdown { 0 } else { 1 });
                self.native_open_buffers = None;
                self.diagnostic_overlays.clear();
                if let Some(bridge) = &mut self.build_bridge {
                    bridge.slot = None;
                }
            }
            return None;
        }
        if self.initialized.is_none() || self.shutdown {
            return None;
        }
        let params = params?;
        let document = params.get("textDocument")?;
        let uri = document.get("uri")?.as_str()?;
        if uri.is_empty() {
            return None;
        }
        match method {
            "textDocument/didOpen" => {
                let version = i32::try_from(document.get("version")?.as_i64()?).ok()?;
                document.get("languageId")?.as_str()?;
                let text = document.get("text")?.as_str()?;
                if !self.documents.contains_key(uri) {
                    self.documents.insert(
                        uri.to_owned(),
                        (version, text.to_owned(), std::sync::Arc::new(())),
                    );
                    self.admit_build();
                }
            }
            "textDocument/didChange" => {
                let version = i32::try_from(document.get("version")?.as_i64()?).ok()?;
                let changes = params.get("contentChanges")?.as_array()?;
                if changes.iter().any(|change| {
                    !change.get("text").is_some_and(Value::is_string)
                        || change.get("range").is_some()
                        || change.get("rangeLength").is_some()
                }) {
                    return None;
                }
                let buffer = self.documents.get_mut(uri)?;
                if version > buffer.0 {
                    if let Some(text) = changes
                        .last()
                        .and_then(|change| change.get("text"))
                        .and_then(Value::as_str)
                    {
                        buffer.1 = text.to_owned();
                    }
                    buffer.0 = version;
                    self.diagnostic_overlays.retain(|_, overlay| {
                        !overlay
                            .association
                            .lifetimes
                            .iter()
                            .any(|(source, _)| source == uri)
                    });
                    if self.native_open_buffers.as_ref().is_some_and(|snapshot| {
                        snapshot
                            .lifetimes
                            .iter()
                            .any(|(original, _)| original == uri)
                    }) {
                        self.native_open_buffers = None;
                    }
                    self.admit_build();
                }
            }
            "textDocument/didClose" => {
                let removed = self.documents.remove(uri).is_some();
                if self.native_open_buffers.as_ref().is_some_and(|snapshot| {
                    snapshot
                        .lifetimes
                        .iter()
                        .any(|(original, _)| original == uri)
                }) {
                    self.native_open_buffers = None;
                }
                if removed {
                    self.diagnostic_overlays.retain(|_, overlay| {
                        !overlay
                            .association
                            .lifetimes
                            .iter()
                            .any(|(source, _)| source == uri)
                    });
                    self.admit_build();
                }
            }
            _ => {}
        }
        None
    }

    /// Serves bounded Content-Length frames until an exit notification.
    pub fn serve(&mut self, input: &mut impl BufRead, output: &mut impl Write) -> io::Result<i32> {
        let invalid = || io::Error::new(io::ErrorKind::InvalidData, "Invalid LSP frame");
        loop {
            if let Some(code) = self.exit_code {
                return Ok(code);
            }
            let mut header = Vec::new();
            loop {
                let available = input.fill_buf()?;
                if available.is_empty() {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Missing LSP exit or truncated header",
                    ));
                }
                let count = available
                    .iter()
                    .position(|byte| *byte == b'\n')
                    .map_or(available.len(), |index| index + 1);
                if header.len() + count > 8 * 1024 {
                    return Err(invalid());
                }
                header.extend_from_slice(&available[..count]);
                input.consume(count);
                if header.last() == Some(&b'\n') && !header.ends_with(b"\r\n") {
                    return Err(invalid());
                }
                if header.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            if !header.is_ascii() {
                return Err(invalid());
            }
            let mut length = None;
            let mut content_type = None;
            let header = std::str::from_utf8(&header).map_err(|_| invalid())?;
            for line in header[..header.len() - 4].split("\r\n") {
                let (name, value) = line.split_once(':').ok_or_else(invalid)?;
                if name.is_empty()
                    || !name.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte)
                    })
                    || !value
                        .bytes()
                        .all(|byte| byte.is_ascii_graphic() || byte == b' ' || byte == b'\t')
                {
                    return Err(invalid());
                }
                let value = value.trim();
                if name.eq_ignore_ascii_case("Content-Length") {
                    if length.is_some()
                        || value.is_empty()
                        || !value.bytes().all(|byte| byte.is_ascii_digit())
                    {
                        return Err(invalid());
                    }
                    let value = value.parse::<usize>().map_err(|_| invalid())?;
                    if value > 16 * 1024 * 1024 {
                        return Err(invalid());
                    }
                    length = Some(value);
                } else if name.eq_ignore_ascii_case("Content-Type")
                    && content_type.replace(value).is_some()
                {
                    return Err(invalid());
                }
            }
            let mut body = vec![0; length.ok_or_else(invalid)?];
            input.read_exact(&mut body)?;
            let supported = content_type.is_none_or(|value| {
                let mut parts = value.split(';');
                parts.next().is_some_and(|kind| {
                    kind.trim()
                        .eq_ignore_ascii_case("application/vscode-jsonrpc")
                }) && parts.all(|part| {
                    part.trim().split_once('=').is_some_and(|(key, value)| {
                        !key.trim().eq_ignore_ascii_case("charset") || {
                            let value = value.trim();
                            let value = value
                                .strip_prefix('"')
                                .and_then(|value| value.strip_suffix('"'))
                                .unwrap_or(value);
                            value.eq_ignore_ascii_case("utf-8")
                                || value.eq_ignore_ascii_case("utf8")
                        }
                    })
                })
            });
            let response = match serde_json::from_slice::<Value>(&body) {
                Ok(message) if supported => self.handle_message(message),
                _ => Some(json!({"jsonrpc": "2.0", "id": null,
                    "error": {"code": -32700, "message": "Parse error"}})),
            };
            if let Some(response) = response {
                let body = serde_json::to_vec(&response).map_err(io::Error::other)?;
                write!(output, "Content-Length: {}\r\n\r\n", body.len())?;
                output.write_all(&body)?;
                output.flush()?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    fn request(method: &str, params: Value) -> Value {
        json!({"jsonrpc": "2.0", "id": 7, "method": method, "params": params})
    }

    fn notification(method: &str, params: Value) -> Value {
        json!({"jsonrpc": "2.0", "method": method, "params": params})
    }

    fn initialize(server: &mut Server) {
        let response = server
            .handle_message(request(
                "initialize",
                json!({"processId": null, "rootUri": null, "capabilities": {"unknown": true}}),
            ))
            .unwrap();
        assert_eq!(
            response,
            json!({"jsonrpc": "2.0", "id": 7, "result": {
            "capabilities": {"positionEncoding": "utf-16", "hoverProvider": true, "completionProvider": {"triggerCharacters": ["."]}, "codeActionProvider": {"codeActionKinds": ["quickfix"], "resolveProvider": true}, "semanticTokensProvider": {"legend": {"tokenTypes": ["keyword", "operator", "number", "comment"], "tokenModifiers": []}, "full": true}, "textDocumentSync": {"openClose": true, "change": 1}}}})
        );
    }

    fn assert_error(server: &mut Server, message: Value, code: i32) -> Value {
        let response = server.handle_message(message).unwrap();
        assert!(response.get("id").is_some());
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["error"]["code"], code);
        assert!(response["error"]["message"].is_string());
        assert!(response.get("result").is_none());
        response
    }

    fn frame(body: impl AsRef<[u8]>) -> Vec<u8> {
        let body = body.as_ref();
        let mut bytes = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
        bytes.extend_from_slice(body);
        bytes
    }

    fn responses(mut bytes: &[u8]) -> Vec<Value> {
        let mut result = Vec::new();
        while !bytes.is_empty() {
            let end = bytes
                .windows(4)
                .position(|part| part == b"\r\n\r\n")
                .unwrap();
            let header = std::str::from_utf8(&bytes[..end]).unwrap();
            let length: usize = header
                .strip_prefix("Content-Length: ")
                .unwrap()
                .parse()
                .unwrap();
            let body = &bytes[end + 4..end + 4 + length];
            result.push(serde_json::from_slice(body).unwrap());
            bytes = &bytes[end + 4 + length..];
        }
        result
    }

    #[test]
    fn lifecycle_and_message_kinds_preserve_request_ids() {
        let mut server = Server::default();
        assert_error(&mut server, request("unknown", Value::Null), -32002);
        for method in [
            "initialize",
            "shutdown",
            "textDocument/didOpen",
            "$/cancelRequest",
        ] {
            assert!(
                server
                    .handle_message(notification(
                        method,
                        if method == "initialize" {
                            json!({"processId": null, "rootUri": null, "capabilities": {}})
                        } else {
                            Value::Null
                        }
                    ))
                    .is_none()
            );
        }
        initialize(&mut server);
        assert!(
            server
                .handle_message(notification("shutdown", Value::Null))
                .is_none()
        );
        assert!(!server.shutdown);
        assert_error(&mut server, request("initialize", Value::Null), -32600);
        for id in [json!(i32::MIN), json!(i32::MAX), json!("リクエスト😀")] {
            let mut message = request("$/cancelRequest", Value::Null);
            message["id"] = id.clone();
            let response = server.handle_message(message).unwrap();
            assert_eq!(response["id"], id);
            assert_eq!(response["error"]["code"], -32601);
        }
        for method in [
            "exit",
            "initialized",
            "textDocument/didOpen",
            "textDocument/didChange",
            "textDocument/didClose",
        ] {
            assert_error(&mut server, request(method, Value::Null), -32600);
            assert!(
                server
                    .handle_message(notification(method, json!({})))
                    .is_none()
            );
        }
        assert_error(&mut server, request("shutdown", json!({})), -32602);
        for params in [Value::Null, json!(false), json!(1), json!("bad")] {
            assert_error(&mut server, request("unknown", params), -32602);
        }
        for params in [json!({}), json!([])] {
            assert_error(&mut server, request("unknown", params), -32601);
        }
        let mut unknown = request("unknown", Value::Null);
        unknown.as_object_mut().unwrap().remove("params");
        assert_error(&mut server, unknown, -32601);
        let mut shutdown = request("shutdown", Value::Null);
        shutdown.as_object_mut().unwrap().remove("params");
        assert_eq!(
            server.handle_message(shutdown).unwrap(),
            json!({"jsonrpc": "2.0", "id": 7, "result": null})
        );
        for method in ["shutdown", "initialize", "unknown"] {
            assert_error(&mut server, request(method, Value::Null), -32600);
        }
        assert!(
            server
                .handle_message(notification("exit", json!({})))
                .is_none()
        );
        assert_eq!(server.exit_code, None);
        assert!(
            server
                .handle_message(notification("exit", Value::Null))
                .is_none()
        );
        assert_eq!(
            server.serve(&mut Cursor::new([]), &mut Vec::new()).unwrap(),
            0
        );
    }

    #[test]
    fn initialization_validates_every_required_field_without_committing_failure() {
        for params in [
            Value::Null,
            json!([]),
            json!({}),
            json!({"processId": null, "capabilities": {}}),
            json!({"rootUri": null, "capabilities": {}}),
            json!({"processId": null, "rootUri": null}),
            json!({"processId": "1", "rootUri": null, "capabilities": {}}),
            json!({"processId": 2147483648_u64, "rootUri": null, "capabilities": {}}),
            json!({"processId": 1.5, "rootUri": null, "capabilities": {}}),
            json!({"processId": null, "rootUri": false, "capabilities": {}}),
            json!({"processId": null, "rootUri": null, "capabilities": []}),
        ] {
            let mut server = Server::default();
            assert_error(&mut server, request("initialize", params), -32602);
            initialize(&mut server);
        }
        for process in [i32::MIN, i32::MAX] {
            let mut server = Server::default();
            assert!(server.handle_message(request("initialize", json!({
                "processId": process, "rootUri": "file:///workspace", "capabilities": {}, "ignored": true
            }))).unwrap().get("result").is_some());
        }
    }

    #[test]
    fn invalid_envelopes_and_ids_cannot_invoke_methods() {
        let mut server = Server::default();
        initialize(&mut server);
        for message in [
            Value::Null,
            json!([]),
            json!([request("shutdown", Value::Null)]),
            json!({}),
            json!({"jsonrpc": "1.0", "method": "shutdown", "id": 1}),
            json!({"jsonrpc": "2.0", "method": 12, "id": 1}),
            json!({"jsonrpc": "2.0", "method": "shutdown", "id": 1, "result": null}),
            json!({"jsonrpc": "2.0", "method": "exit", "error": {"code": 1, "message": "error"}}),
        ] {
            let response = assert_error(&mut server, message, -32600);
            assert_eq!(response.get("id"), Some(&Value::Null));
        }
        for id in [
            Value::Null,
            json!(1.5),
            json!(2147483648_u64),
            json!(-2147483649_i64),
            json!(true),
            json!({}),
            json!([]),
        ] {
            let mut message = request("shutdown", Value::Null);
            message["id"] = id;
            assert_eq!(
                assert_error(&mut server, message, -32600).get("id"),
                Some(&Value::Null)
            );
        }
        assert!(!server.shutdown);
        assert_eq!(server.exit_code, None);
    }

    #[test]
    fn unmatched_responses_are_silent_but_malformed_responses_are_rejected() {
        let mut server = Server::default();
        for id in [Value::Null, json!(1), json!("unused")] {
            for payload in [
                json!({"result": null}),
                json!({"error": {"code": -32601, "message": "unused"}}),
            ] {
                let mut message = payload;
                message["jsonrpc"] = json!("2.0");
                message["id"] = id.clone();
                assert!(server.handle_message(message).is_none());
            }
        }
        for message in [
            json!({"jsonrpc": "2.0", "result": null}),
            json!({"jsonrpc": "2.0", "id": 1, "result": null, "error": null}),
            json!({"jsonrpc": "2.0", "id": 1, "error": null}),
            json!({"jsonrpc": "2.0", "id": 1, "error": {"code": "1", "message": "bad"}}),
            json!({"jsonrpc": "2.0", "id": 1, "error": {"code": 2147483648_u64, "message": "bad"}}),
            json!({"jsonrpc": "2.0", "id": 1, "error": {"code": 1, "message": null}}),
        ] {
            assert_error(&mut server, message, -32600);
        }
    }

    #[test]
    fn full_sync_keeps_exact_text_and_rejects_invalid_updates_atomically() {
        let mut server = Server::default();
        let original = "\u{feff}begin\r\n😀e\u{301}\r\nend;\n";
        let open = notification(
            "textDocument/didOpen",
            json!({"textDocument": {
            "uri": "file:///a.miz", "languageId": "mizar", "version": -1, "text": original}}),
        );
        assert!(server.handle_message(open.clone()).is_none());
        assert_eq!(server.document("file:///a.miz"), None);
        initialize(&mut server);
        assert!(server.handle_message(open.clone()).is_none());
        assert_eq!(server.document("file:///a.miz"), Some((-1, original)));
        let mut duplicate = open;
        duplicate["params"]["textDocument"]["text"] = json!("duplicate");
        server.handle_message(duplicate);
        for version in [json!(-2), json!(-1), json!(0.5), json!(2147483648_u64)] {
            server.handle_message(notification("textDocument/didChange", json!({
                "textDocument": {"uri": "file:///a.miz", "version": version}, "contentChanges": [{"text": "stale"}]})));
            assert_eq!(server.document("file:///a.miz"), Some((-1, original)));
        }
        for invalid in [
            Value::Null,
            json!("bad"),
            json!({}),
            json!({"text": null}),
            json!({"text": "partial", "range": null}),
            json!({"text": "partial", "rangeLength": 0}),
        ] {
            for invalid_last in [false, true] {
                let mut changes = [invalid.clone(), json!({"text": "must not be committed"})];
                if invalid_last {
                    changes.swap(0, 1);
                }
                assert!(
                    server
                        .handle_message(notification(
                            "textDocument/didChange",
                            json!({
                "textDocument": {"uri": "file:///a.miz", "version": 0},
                "contentChanges": changes})
                        ))
                        .is_none()
                );
                assert_eq!(server.document("file:///a.miz"), Some((-1, original)));
            }
        }
        server.handle_message(notification(
            "textDocument/didChange",
            json!({
            "textDocument": {"uri": "file:///a.miz", "version": 0}, "contentChanges": []}),
        ));
        assert_eq!(server.document("file:///a.miz"), Some((0, original)));
        let changed = "\u{feff}new😀\r\n";
        server.handle_message(notification(
            "textDocument/didChange",
            json!({
            "textDocument": {"uri": "file:///a.miz", "version": i32::MAX},
            "contentChanges": [{"text": "first"}, {"text": changed}]}),
        ));
        assert_eq!(server.document("file:///a.miz"), Some((i32::MAX, changed)));
        server.handle_message(notification("textDocument/didChange", json!({
            "textDocument": {"uri": "file:///unknown", "version": 0}, "contentChanges": [{"text": "unknown"}]})));
        server.handle_message(notification(
            "textDocument/didClose",
            json!({"textDocument": {"uri": "file:///unknown"}}),
        ));
        assert_eq!(server.document("file:///unknown"), None);
        server.handle_message(notification(
            "textDocument/didClose",
            json!({"textDocument": {"uri": "file:///a.miz"}}),
        ));
        assert_eq!(server.document("file:///a.miz"), None);
    }

    #[test]
    fn invalid_open_close_and_wrong_message_kind_leave_buffers_unchanged() {
        let mut server = Server::default();
        initialize(&mut server);
        let document =
            json!({"uri": "file:///a.miz", "version": 1, "languageId": "mizar", "text": "kept"});
        for (field, value) in [
            ("uri", json!("")),
            ("uri", Value::Null),
            ("version", json!(2147483648_u64)),
            ("version", json!(-2147483649_i64)),
            ("version", json!(1.5)),
            ("languageId", Value::Null),
            ("text", Value::Null),
        ] {
            let mut invalid = document.clone();
            invalid[field] = value;
            server.handle_message(notification(
                "textDocument/didOpen",
                json!({"textDocument": invalid}),
            ));
            assert_eq!(server.document("file:///a.miz"), None);
        }
        server.handle_message(notification(
            "textDocument/didOpen",
            json!({"textDocument": document}),
        ));
        for method in [
            "textDocument/didOpen",
            "textDocument/didChange",
            "textDocument/didClose",
        ] {
            assert_error(
                &mut server,
                request(
                    method,
                    json!({"textDocument": {
                "uri": "file:///a.miz", "version": 2, "languageId": "mizar", "text": "wrong kind"},
                "contentChanges": [{"text": "wrong kind"}]}),
                ),
                -32600,
            );
        }
        for params in [
            Value::Null,
            json!([]),
            json!({}),
            json!({"textDocument": {"uri": null}}),
        ] {
            for method in ["textDocument/didChange", "textDocument/didClose"] {
                assert!(
                    server
                        .handle_message(notification(method, params.clone()))
                        .is_none()
                );
            }
        }
        assert_eq!(server.document("file:///a.miz"), Some((1, "kept")));
        server.handle_message(request("shutdown", Value::Null));
        for exit in [false, true] {
            if exit {
                server.handle_message(notification("exit", Value::Null));
            }
            for method in [
                "textDocument/didOpen",
                "textDocument/didChange",
                "textDocument/didClose",
            ] {
                let uri = if method == "textDocument/didOpen" {
                    "file:///after.miz"
                } else {
                    "file:///a.miz"
                };
                assert!(
                    server
                        .handle_message(notification(
                            method,
                            json!({"textDocument": {
                    "uri": uri, "version": 2, "languageId": "mizar", "text": "terminal"},
                    "contentChanges": [{"text": "terminal"}]})
                        ))
                        .is_none()
                );
            }
            assert_eq!(server.document("file:///a.miz"), Some((1, "kept")));
            assert_eq!(server.document("file:///after.miz"), None);
        }
        assert!(
            server
                .handle_message(request("initialize", Value::Null))
                .is_none()
        );
    }

    #[test]
    fn exit_without_shutdown_returns_one_and_invalid_exit_does_not_terminate() {
        for initialized in [false, true] {
            let mut server = Server::default();
            if initialized {
                initialize(&mut server);
            }
            for params in [json!({}), json!([]), json!(1)] {
                server.handle_message(notification("exit", params));
                assert_eq!(server.exit_code, None);
            }
            server.handle_message(json!({"jsonrpc": "2.0", "method": "exit"}));
            assert_eq!(
                server.serve(&mut Cursor::new([]), &mut Vec::new()).unwrap(),
                1
            );
        }
    }

    #[test]
    fn fragmented_unicode_wire_traffic_uses_byte_lengths_and_silent_notifications() {
        let mut initialize = request(
            "initialize",
            json!({"processId": null, "rootUri": null, "capabilities": {}}),
        );
        initialize["id"] = json!("初期化😀");
        let messages = [
            initialize,
            notification(
                "textDocument/didOpen",
                json!({"textDocument": {"uri": "file:///あ.miz",
                "version": 1, "languageId": "mizar", "text": "\u{feff}😀\r\n"}}),
            ),
            notification(
                "textDocument/didChange",
                json!({"textDocument": {"uri": "file:///あ.miz", "version": 2},
                "contentChanges": [{"text": "é\r\n😀"}]}),
            ),
            notification("$/cancelRequest", json!({"id": "absent"})),
            request("shutdown", Value::Null),
            notification("exit", Value::Null),
        ];
        let bytes: Vec<_> = messages
            .iter()
            .flat_map(|message| frame(message.to_string()))
            .collect();
        for capacity in [1, 2, 3, 17] {
            let mut server = Server::default();
            let mut output = Vec::new();
            assert_eq!(
                server
                    .serve(
                        &mut BufReader::with_capacity(capacity, Cursor::new(&bytes)),
                        &mut output
                    )
                    .unwrap(),
                0
            );
            let replies = responses(&output);
            assert_eq!(replies.len(), 2);
            assert_eq!(replies[0]["id"], "初期化😀");
            assert_eq!(
                replies[1],
                json!({"jsonrpc": "2.0", "id": 7, "result": null})
            );
            assert_eq!(server.document("file:///あ.miz"), Some((2, "é\r\n😀")));
        }
    }

    #[test]
    fn complete_bad_frames_report_parse_errors_and_continue() {
        let mut bytes = frame(b"not JSON");
        bytes.extend(frame([0xff]));
        for content_type in [
            "application/json",
            "application/vscode-jsonrpc; charset=latin1",
            "application/vscode-jsonrpc; charset=",
        ] {
            bytes.extend_from_slice(
                format!("Content-Length: 2\r\nContent-Type: {content_type}\r\n\r\n{{}}").as_bytes(),
            );
        }
        bytes.extend(frame(
            request(
                "initialize",
                json!({"processId": null, "rootUri": null, "capabilities": {}}),
            )
            .to_string(),
        ));
        bytes.extend(frame(notification("exit", Value::Null).to_string()));
        let mut output = Vec::new();
        assert_eq!(
            Server::default()
                .serve(
                    &mut BufReader::with_capacity(2, Cursor::new(bytes)),
                    &mut output
                )
                .unwrap(),
            1
        );
        let replies = responses(&output);
        assert_eq!(replies.len(), 6);
        for reply in &replies[..5] {
            assert_eq!(reply.get("id"), Some(&Value::Null));
            assert_eq!(reply["error"]["code"], -32700);
        }
        assert!(replies[5].get("result").is_some());
    }

    #[test]
    fn case_insensitive_headers_unknown_headers_and_utf8_charsets_are_accepted() {
        for content_type in [
            "",
            "Content-Type: application/vscode-jsonrpc\r\n",
            "CONTENT-TYPE: Application/Vscode-Jsonrpc; Charset=UTF8\r\n",
            "content-type: application/vscode-jsonrpc; charset=\"utf-8\"\r\n",
        ] {
            let body = notification("exit", Value::Null).to_string();
            let bytes = format!(
                "cOnTeNt-LeNgTh: {}\r\nX-Unknown: ignored\r\nX9!#$%&'*+-.^_`|~: valid\r\n{content_type}\r\n{body}",
                body.len()
            );
            let mut output = Vec::new();
            assert_eq!(
                Server::default()
                    .serve(
                        &mut BufReader::with_capacity(1, Cursor::new(bytes)),
                        &mut output
                    )
                    .unwrap(),
                1
            );
            assert!(responses(&output).is_empty());
        }
    }

    #[test]
    fn invalid_framing_truncation_and_early_eof_are_io_errors() {
        for bytes in [b"".as_slice(), b"Content-Length: 2\r\n", b"Content-Length: 2\r\n\r\n{",
            b"Content-Length: 2\n\n{}", b"Content-Length: 2\r\ncontent-length: 2\r\n\r\n{}",
            b"Content-Length: -1\r\n\r\n", b"Content-Length: +1\r\n\r\n",
            b"Content-Length: nope\r\n\r\n", b"Content-Length:\r\n\r\n",
            b"Content-Length: 99999999999999999999999\r\n\r\n", b"Unknown: value\r\n\r\n",
            b"bad header\r\n\r\n", b"Content-Length: 0\r\nX: \xff\r\n\r\n",
            b"Content-Length: 0\r\nX: bad\rvalue\r\n\r\n",
            b"Content-Length: 33\r\nX@: ignored\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"exit\"}",
            b"Content-Length: 33\r\nX<: ignored\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"exit\"}",
            b"Content-Length: 0\r\nContent-Type: application/vscode-jsonrpc\r\ncontent-type: application/vscode-jsonrpc\r\n\r\n"] {
            let mut server = Server::default();
            let mut output = Vec::new();
            assert!(server.serve(&mut BufReader::with_capacity(1, Cursor::new(bytes)), &mut output).is_err(), "{bytes:?}");
            assert!(output.is_empty());
            assert_eq!(server.exit_code, None);
        }
        let mut server = Server::default();
        initialize(&mut server);
        let bytes = frame(request("shutdown", Value::Null).to_string());
        assert_eq!(
            server
                .serve(&mut Cursor::new(bytes), &mut Vec::new())
                .unwrap_err()
                .kind(),
            io::ErrorKind::UnexpectedEof
        );
    }

    #[test]
    fn header_and_body_bounds_are_checked_before_body_reads() {
        let body = notification("exit", Value::Null).to_string();
        let base = format!("Content-Length: {}\r\nX: ", body.len());
        let header = format!("{base}{}\r\n\r\n", "a".repeat(8192 - base.len() - 4));
        assert_eq!(header.len(), 8192);
        let bytes = format!("{header}{body}");
        assert_eq!(
            Server::default()
                .serve(&mut Cursor::new(bytes), &mut Vec::new())
                .unwrap(),
            1
        );
        for bytes in [
            header.replace("\r\n\r\n", "x\r\n\r\n"),
            "a".repeat(8193),
            "Content-Length: 16777217\r\n\r\n".to_owned(),
        ] {
            let mut input = Cursor::new(bytes);
            let error = Server::default()
                .serve(&mut input, &mut Vec::new())
                .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        }
        let mut body = vec![b' '; 16 * 1024 * 1024];
        body[..2].copy_from_slice(b"{}");
        let mut bytes = frame(body);
        bytes.extend(frame(notification("exit", Value::Null).to_string()));
        let mut output = Vec::new();
        assert_eq!(
            Server::default()
                .serve(&mut Cursor::new(bytes), &mut output)
                .unwrap(),
            1
        );
        assert_eq!(responses(&output)[0]["error"]["code"], -32600);
    }

    #[test]
    fn output_write_failures_propagate() {
        let bytes = frame(request("unknown", Value::Null).to_string());
        assert_eq!(
            Server::default()
                .serve(&mut Cursor::new(bytes), &mut [].as_mut_slice())
                .unwrap_err()
                .kind(),
            io::ErrorKind::WriteZero
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn input_read_and_output_flush_failures_propagate() {
        let directory = std::fs::File::open(std::env::temp_dir()).unwrap();
        assert!(
            Server::default()
                .serve(&mut BufReader::new(directory), &mut Vec::new())
                .is_err()
        );
        let sink = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap();
        let mut output = io::BufWriter::with_capacity(4096, sink);
        let bytes = frame(request("unknown", Value::Null).to_string());
        assert!(
            Server::default()
                .serve(&mut Cursor::new(bytes), &mut output)
                .is_err()
        );
    }

    #[test]
    fn raw_initialize_params_remain_exact_through_document_and_terminal_traffic() {
        for (root, options) in [
            (Value::Null, None),
            (json!("file:///未変換%20😀"), Some(Value::Null)),
            (Value::Null, Some(json!(false))),
            (
                json!("untouched:ルート"),
                Some(json!({"nested": [null, 3, "é😀"]})),
            ),
        ] {
            let mut server = Server::default();
            assert_eq!(server.initialization_params(), None);
            for invalid in [Value::Null, json!({"processId": null, "rootUri": null})] {
                assert_error(&mut server, request("initialize", invalid), -32602);
                assert_eq!(server.initialization_params(), None);
            }
            let mut expected = json!({
                "processId": -1, "rootUri": root,
                "capabilities": {"unknown": {"array": [true, null, "能力😀"]}},
                "unknown": [null, false, 1.5, {"raw": "\u{feff}\r\n"}]
            });
            if let Some(options) = options {
                expected["initializationOptions"] = options;
            }
            let mut original = expected.clone();
            assert_eq!(
                server.handle_message(request("initialize", original.clone())),
                Some(json!({
                    "jsonrpc": "2.0", "id": 7, "result": {"capabilities": {
                        "positionEncoding": "utf-16", "hoverProvider": true, "completionProvider": {"triggerCharacters": ["."]}, "codeActionProvider": {"codeActionKinds": ["quickfix"], "resolveProvider": true}, "semanticTokensProvider": {"legend": {"tokenTypes": ["keyword", "operator", "number", "comment"], "tokenModifiers": []}, "full": true}, "textDocumentSync": {"openClose": true, "change": 1}
                    }}
                }))
            );
            original["rootUri"] = json!("replaced");
            original["capabilities"]["unknown"] = Value::Null;
            assert_eq!(server.initialization_params(), Some(&expected));
            for params in [original.clone(), Value::Null] {
                assert_error(&mut server, request("initialize", params), -32600);
                assert_eq!(server.initialization_params(), Some(&expected));
            }
            let uri = "untouched:文書%20😀";
            let text = "\u{feff}begin\r\n😀\r\n";
            assert!(server.handle_message(notification("textDocument/didOpen", json!({
                "textDocument": {"uri": uri, "languageId": "mizar", "version": -1, "text": text}
            }))).is_none());
            assert_eq!(server.document(uri), Some((-1, text)));
            assert_eq!(server.initialization_params(), Some(&expected));
            let changed = "\u{feff}変更😀\r\n";
            assert!(server.handle_message(notification("textDocument/didChange", json!({
                "textDocument": {"uri": uri, "version": 0}, "contentChanges": [{"text": changed}]
            }))).is_none());
            assert_eq!(server.document(uri), Some((0, changed)));
            assert!(
                server
                    .handle_message(notification("$/cancelRequest", json!({"id": 7})))
                    .is_none()
            );
            assert_eq!(server.document(uri), Some((0, changed)));
            assert_eq!(server.initialization_params(), Some(&expected));
            assert!(
                server
                    .handle_message(notification(
                        "textDocument/didClose",
                        json!({
                            "textDocument": {"uri": uri}
                        })
                    ))
                    .is_none()
            );
            assert_eq!(server.document(uri), None);
            assert_eq!(server.initialization_params(), Some(&expected));
            assert_eq!(
                server.handle_message(request("shutdown", Value::Null)),
                Some(json!({"jsonrpc": "2.0", "id": 7, "result": null}))
            );
            assert_error(&mut server, request("initialize", original.clone()), -32600);
            assert_eq!(server.initialization_params(), Some(&expected));
            assert!(
                server
                    .handle_message(notification("exit", Value::Null))
                    .is_none()
            );
            assert!(
                server
                    .handle_message(request("initialize", original))
                    .is_none()
            );
            assert_eq!(server.initialization_params(), Some(&expected));
        }
    }

    #[test]
    fn raw_initialize_params_survive_framed_rejection_and_silent_notifications() {
        let expected = json!({
            "processId": null, "rootUri": "file:///ルート%20😀", "capabilities": {"future": [null, false]},
            "initializationOptions": {"raw": ["\u{feff}\r\n", 9]}, "unknown": {"nested": true}
        });
        let uri = "file:///文書%20😀.miz";
        let text = "\u{feff}編集😀\r\n";
        let mut bytes = Vec::new();
        for message in [
            request("initialize", Value::Null),
            request("initialize", expected.clone()),
            notification("initialized", json!({"unknown": true})),
            notification(
                "textDocument/didOpen",
                json!({"textDocument": {
                    "uri": uri, "languageId": "mizar", "version": -2, "text": "initial\r\n"
                }}),
            ),
            notification(
                "textDocument/didChange",
                json!({
                    "textDocument": {"uri": uri, "version": -1}, "contentChanges": [{"text": text}]
                }),
            ),
            notification("$/cancelRequest", json!({"id": 7})),
            request(
                "initialize",
                json!({"processId": 42, "rootUri": null, "capabilities": {}}),
            ),
            request("shutdown", Value::Null),
            notification(
                "textDocument/didClose",
                json!({"textDocument": {"uri": uri}}),
            ),
            request("initialize", Value::Null),
            notification("exit", Value::Null),
            request(
                "initialize",
                json!({"processId": 42, "rootUri": null, "capabilities": {}}),
            ),
        ] {
            bytes.extend(frame(message.to_string()));
        }
        let mut server = Server::default();
        let mut output = Vec::new();
        assert_eq!(
            server
                .serve(
                    &mut BufReader::with_capacity(2, Cursor::new(bytes)),
                    &mut output
                )
                .unwrap(),
            0
        );
        assert_eq!(
            responses(&output),
            vec![
                json!({"jsonrpc": "2.0", "id": 7, "error": {"code": -32602, "message": "Invalid params"}}),
                json!({"jsonrpc": "2.0", "id": 7, "result": {"capabilities": {
                    "positionEncoding": "utf-16", "hoverProvider": true, "completionProvider": {"triggerCharacters": ["."]}, "codeActionProvider": {"codeActionKinds": ["quickfix"], "resolveProvider": true}, "semanticTokensProvider": {"legend": {"tokenTypes": ["keyword", "operator", "number", "comment"], "tokenModifiers": []}, "full": true}, "textDocumentSync": {"openClose": true, "change": 1}
                }}}),
                json!({"jsonrpc": "2.0", "id": 7, "error": {"code": -32600, "message": "Invalid Request"}}),
                json!({"jsonrpc": "2.0", "id": 7, "result": null}),
                json!({"jsonrpc": "2.0", "id": 7, "error": {"code": -32600, "message": "Invalid Request"}}),
            ]
        );
        assert_eq!(server.initialization_params(), Some(&expected));
        assert_eq!(server.document(uri), Some((-1, text)));
    }

    #[test]
    fn native_workspace_inputs_follow_initialize_and_reacquire_real_metadata() {
        use std::{fs, time::SystemTime};

        let root = std::env::temp_dir().join(format!(
            "mizar lsp native inputs {} {}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let uri = format!(
            "file://{}{}",
            if cfg!(windows) { "/" } else { "" },
            root.to_str()
                .unwrap()
                .replace('\\', "/")
                .replace(' ', "%20")
        );
        let manifest = "[package]\nname = \"actual_package\"\nversion = \"1.2.3\"\n";
        let lock = "schema_version = 1\n[[package]]\nname = \"actual_package\"\nversion = \"1.2.3\"\nsource = { kind = \"workspace\", path = \".\" }\ndependencies = []\n";
        fs::write(root.join("mizar.pkg"), manifest).unwrap();
        fs::write(root.join("mizar.lock"), lock).unwrap();
        let params = json!({"processId": null, "rootUri": format!("{uri}/."),
            "capabilities": {}, "initializationOptions": {"unknown": [1, null]}});
        let mut server = Server::default();
        assert_eq!(
            server.handle_message(request("initialize", params.clone())),
            Some(
                json!({"jsonrpc": "2.0", "id": 7, "result": {"capabilities": {
                    "positionEncoding": "utf-16", "hoverProvider": true, "completionProvider": {"triggerCharacters": ["."]}, "codeActionProvider": {"codeActionKinds": ["quickfix"], "resolveProvider": true}, "semanticTokensProvider": {"legend": {"tokenTypes": ["keyword", "operator", "number", "comment"], "tokenModifiers": []}, "full": true}, "textDocumentSync": {"openClose": true, "change": 1}
                }}})
            )
        );
        let document_uri = format!("{uri}/unsaved.miz");
        let text = "\u{feff}editor😀\r\n";
        assert!(server.handle_message(notification("textDocument/didOpen", json!({
            "textDocument": {"uri": document_uri, "languageId": "mizar", "version": 3, "text": text}
        }))).is_none());
        let (canonical, packages, parsed_lock) = server.native_workspace_inputs().unwrap().unwrap();
        assert_eq!(
            canonical.as_str(),
            fs::canonicalize(&root).unwrap().to_str().unwrap()
        );
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].member_path, ".");
        assert_eq!(
            packages[0].manifest,
            mizar_build::planner::parse_package_manifest(manifest).unwrap()
        );
        assert_eq!(
            parsed_lock,
            mizar_build::planner::parse_lockfile(lock).unwrap()
        );

        fs::create_dir_all(root.join("members/nested")).unwrap();
        let changed_manifest = manifest.replace("1.2.3", "2.3.4");
        let changed_lock = lock
            .replace("1.2.3", "2.3.4")
            .replace("path = \".\"", "path = \"members/nested\"");
        fs::write(root.join("members/nested/mizar.pkg"), &changed_manifest).unwrap();
        fs::write(
            root.join("mizar.workspace"),
            "[workspace]\nmembers = [\"members/nested\"]\n",
        )
        .unwrap();
        fs::write(root.join("mizar.lock"), &changed_lock).unwrap();
        assert_error(
            &mut server,
            request(
                "initialize",
                json!({
                    "processId": null, "rootUri": null, "capabilities": {}
                }),
            ),
            -32600,
        );
        assert_eq!(
            server.handle_message(request("shutdown", Value::Null)),
            Some(json!({"jsonrpc": "2.0", "id": 7, "result": null}))
        );
        assert!(
            server
                .handle_message(notification("exit", Value::Null))
                .is_none()
        );
        let (current_root, current_packages, current_lock) =
            server.native_workspace_inputs().unwrap().unwrap();
        assert_eq!(current_root, canonical);
        assert_eq!(current_packages.len(), 1);
        assert_eq!(current_packages[0].member_path, "members/nested");
        assert_eq!(
            current_packages[0].manifest,
            mizar_build::planner::parse_package_manifest(&changed_manifest).unwrap()
        );
        assert_eq!(
            current_lock,
            mizar_build::planner::parse_lockfile(&changed_lock).unwrap()
        );
        fs::remove_file(root.join("mizar.lock")).unwrap();
        assert!(server.native_workspace_inputs().is_err());
        assert_eq!(server.initialization_params(), Some(&params));
        assert_eq!(server.document(&document_uri), Some((3, text)));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn native_workspace_inputs_refuse_absent_invalid_roots_and_reader_failures() {
        use std::{fs, time::SystemTime};

        let root = std::env::temp_dir().join(format!(
            "mizar-lsp-native-errors-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        fs::write(
            root.join("mizar.pkg"),
            "[package]\nname = \"actual_package\"\nversion = \"1.2.3\"\n",
        )
        .unwrap();
        fs::write(
            root.join("mizar.lock"),
            "schema_version = 1\npackage = []\n",
        )
        .unwrap();
        let uri = format!(
            "file://{}{}",
            if cfg!(windows) { "/" } else { "" },
            root.to_str()
                .unwrap()
                .replace('\\', "/")
                .replace(' ', "%20")
        );
        let mut server = Server::default();
        assert_eq!(server.native_workspace_inputs(), Ok(None));
        let fallback = json!({"processId": null, "rootUri": null, "capabilities": {},
            "rootPath": root, "workspaceFolders": [{"uri": uri, "name": "actual"}],
            "initializationOptions": {"rootUri": uri}, "settings": {"rootUri": uri}});
        assert!(
            server
                .handle_message(request("initialize", fallback.clone()))
                .unwrap()
                .get("result")
                .is_some()
        );
        assert_eq!(server.native_workspace_inputs(), Ok(None));
        assert_eq!(server.initialization_params(), Some(&fallback));
        for invalid in [
            "https://example.com/workspace".to_owned(),
            "file:///bad%GG".to_owned(),
            format!("{uri}/missing"),
            format!("{uri}/mizar.pkg"),
        ] {
            let mut server = Server::default();
            assert!(server.handle_message(request("initialize", json!({
                "processId": null, "rootUri": invalid, "capabilities": {}, "rootPath": root,
                "workspaceFolders": [{"uri": uri, "name": "actual"}]
            }))).unwrap().get("result").is_some());
            assert!(server.native_workspace_inputs().is_err(), "{invalid}");
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&root, root.join("linked-root")).unwrap();
            for suffix in ["/linked-root", "/linked-root/."] {
                let mut server = Server::default();
                server.handle_message(request("initialize", json!({
                    "processId": null, "rootUri": format!("{uri}{suffix}"), "capabilities": {}
                }))).unwrap();
                assert!(server.native_workspace_inputs().is_err(), "{suffix}");
            }
        }
        let mut server = Server::default();
        server
            .handle_message(request(
                "initialize",
                json!({
                    "processId": null, "rootUri": uri, "capabilities": {}
                }),
            ))
            .unwrap();
        assert!(server.native_workspace_inputs().unwrap().is_some());
        fs::write(
            root.join("mizar.lock"),
            "schema_version = 0\npackage = []\n",
        )
        .unwrap();
        let canonical =
            mizar_session::WorkspaceRoot::new(fs::canonicalize(&root).unwrap().to_str().unwrap());
        assert_eq!(
            server.native_workspace_inputs().unwrap_err(),
            mizar_build::planner::read_native_workspace_inputs(&canonical).unwrap_err()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn native_workspace_inputs_refuse_drive_relative_windows_roots() {
        use std::{
            fs,
            path::{Component, Prefix},
            time::SystemTime,
        };

        let current = std::env::current_dir().unwrap();
        let drive = match current.components().next() {
            Some(Component::Prefix(prefix)) => match prefix.kind() {
                Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => drive,
                _ => {
                    eprintln!("drive-relative workspace fixture requires a drive-prefixed cwd");
                    return;
                }
            },
            _ => unreachable!("Windows current directory has a path prefix"),
        };
        let name = format!(
            "mizar-lsp-drive-relative-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let root = current.join(&name);
        fs::create_dir(&root).unwrap();
        fs::write(
            root.join("mizar.pkg"),
            "[package]\nname = \"actual_package\"\nversion = \"1.2.3\"\n",
        )
        .unwrap();
        fs::write(
            root.join("mizar.lock"),
            "schema_version = 1\npackage = []\n",
        )
        .unwrap();
        let uri = format!("file:///{}:{name}", char::from(drive));
        let decoded = mizar_session::source::file_path_from_document_uri(&uri).unwrap();
        assert!(!decoded.is_absolute());
        assert!(fs::symlink_metadata(&decoded).unwrap().is_dir());
        let canonical = fs::canonicalize(&root).unwrap();
        assert_eq!(fs::canonicalize(&decoded).unwrap(), canonical);
        let workspace = mizar_session::WorkspaceRoot::new(canonical.to_str().unwrap());
        assert!(mizar_build::planner::read_native_workspace_inputs(&workspace).is_ok());
        let mut server = Server::default();
        assert!(
            server
                .handle_message(request(
                    "initialize",
                    json!({
                        "processId": null, "rootUri": uri, "capabilities": {}
                    })
                ))
                .unwrap()
                .get("result")
                .is_some()
        );
        assert_eq!(
            server.native_workspace_inputs().unwrap_err(),
            "workspace root must be absolute"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn native_frontend_notifications_preserve_real_submissions_and_original_editor_tuples() {
        use mizar_build::{
            module_index::{StaticSourceLayout, build_module_index},
            planner::{
                DependencySelection, PlanRequest, WorkspacePackage, parse_lockfile,
                parse_package_manifest, produce_build_plan,
            },
            task_graph::{ModuleDependencyOverlay, TaskKind},
        };
        use mizar_driver::{
            driver::{CompilerDriver, DriverSubmitError, DriverSubmitInput},
            registry::{PhaseRegistryBuilder, PhaseStatus},
            request::{
                BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
                BuildRequestGeneration, BuildRequestOrigin, BuildTargets, DependencyInputSet,
                SourceInputSet, VerifierConfigInput,
            },
        };
        use mizar_ir::{
            identity::{OutputKind, PipelinePhase as IrPhase, SnapshotHandleRegistry, WorkUnit},
            publisher::{AllowedWorkUnit, PhaseOutputPublisher},
            storage::IrStorageService,
        };
        use mizar_session::{
            DiskSourceLoader, Hash, InMemorySessionIdAllocator, PackageId, SnapshotRegistry,
            SourceOriginInput, ToolchainInfo, WorkspaceRoot,
        };
        use std::{fs, sync::Arc, time::SystemTime};

        for mode in [
            "fatal",
            "clean",
            "version",
            "raw",
            "closed",
            "alias",
            "disk",
            "mixed",
            "generated",
            "resolver",
            "uninitialized",
            "shutdown",
            "exit",
            "missing-publisher",
            "invalid-registry",
            "denied-source",
            "denied-clean-frontend",
        ] {
            let root = std::env::temp_dir().join(format!(
                "mizar-lsp-native-notifications-{}-{}-{mode}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(root.join("alpha/src")).unwrap();
            for name in ["main", "clean"] {
                fs::write(root.join(format!("alpha/src/{name}.miz")), [0xff]).unwrap();
            }
            let workspace = WorkspaceRoot::new(root.to_str().unwrap());
            let plan_request = PlanRequest {
                workspace_root: workspace.clone(),
                dependency_selection: DependencySelection::Normal,
                toolchain: ToolchainInfo::new("mizar-evo-test"),
            };
            let packages = vec![WorkspacePackage {
                member_path: "alpha".into(),
                manifest: parse_package_manifest(
                    "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n",
                )
                .unwrap(),
            }];
            let lock = parse_lockfile("schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n").unwrap();
            let plan =
                produce_build_plan(plan_request.clone(), packages.clone(), lock.clone()).unwrap();
            let layout = StaticSourceLayout::from_native_disk(&plan).unwrap();
            let index = build_module_index(&plan, &layout, &[]).unwrap();
            let loader = DiskSourceLoader::new(root.join("alpha"));
            let mut server = Server::default();
            initialize(&mut server);
            let mut descriptors = Vec::new();
            let mut uris = Vec::new();
            for name in ["main", "clean"] {
                let uri = format!(
                    "file://{}{}",
                    if cfg!(windows) { "/" } else { "" },
                    root.join(format!("alpha/src/{name}.miz"))
                        .to_str()
                        .unwrap()
                        .replace('\\', "/")
                );
                let raw = if name == "clean" || mode == "clean" {
                    "\u{feff}definition\r\nend;\r\n:: 🦀\r\n"
                } else if mode == "alias" {
                    "end\n"
                } else if mode == "resolver" {
                    "import mml.no_such;\ndefinition\nend;\n"
                } else {
                    "\u{feff}::=🦀=::end\r\n:: suffix\r"
                };
                server.handle_message(notification(
                    "textDocument/didOpen",
                    json!({"textDocument": {
                    "uri": uri, "languageId": "mizar", "version": 7, "text": raw}}),
                ));
                let (version, text) = server.document(&uri).unwrap();
                let mut descriptor = index
                    .native_open_buffer_input(
                        &workspace,
                        &PackageId::new("alpha"),
                        &loader,
                        SourceOriginInput::OpenBuffer {
                            uri: if mode == "alias" && name == "main" {
                                uri.replace("main.miz", "%6dain.miz")
                            } else {
                                uri.clone()
                            },
                            expected_version: i64::from(version),
                            actual_version: i64::from(version),
                            text: text.into(),
                        },
                    )
                    .unwrap();
                if mode == "disk" || (mode == "mixed" && name == "clean") {
                    fs::write(root.join(format!("alpha/src/{name}.miz")), raw).unwrap();
                    descriptor.origin = SourceOriginInput::Disk {
                        path: format!("src/{name}.miz").into(),
                    };
                } else if mode == "generated" {
                    descriptor.origin = SourceOriginInput::Generated {
                        generator: mizar_session::GeneratedSourceKind::new("lsp-native-test"),
                        text: raw.into(),
                        anchor: None,
                    };
                }
                descriptors.push((loader.clone(), descriptor));
                uris.push(uri);
            }
            let ids = InMemorySessionIdAllocator::new();
            let snapshots = SnapshotRegistry::new();
            let draft = BuildRequestDraft {
                lane: BuildLaneId::new(1),
                origin: BuildRequestOrigin::Batch(BatchRequest {
                    invocation: BatchInvocation::default(),
                }),
                generation: BuildRequestGeneration::new(0),
                workspace_root: workspace,
                profile: BuildProfile::new("check"),
                targets: BuildTargets::default(),
                source_inputs: SourceInputSet { versions: vec![] },
                dependency_inputs: DependencyInputSet::new(
                    vec![],
                    Hash::from_bytes([1; 32]),
                    ToolchainInfo::new("mizar-evo-test"),
                ),
                verifier_config: VerifierConfigInput::new(Hash::from_bytes([2; 32])),
            };
            let session = draft
                .capture_sources(&ids, &snapshots, descriptors)
                .unwrap();
            let snapshot = session.captured().snapshot().id;
            // SourceLoad and Frontend must use the protected capture, including Disk inputs.
            for name in ["main", "clean"] {
                fs::remove_file(root.join(format!("alpha/src/{name}.miz"))).unwrap();
            }
            match mode {
                "version" => {
                    server.handle_message(notification(
                        "textDocument/didChange",
                        json!({
                    "textDocument": {"uri": uris[1], "version": 8}, "contentChanges": []}),
                    ));
                }
                "raw" => {
                    server.handle_message(notification(
                        "textDocument/didClose",
                        json!({"textDocument": {"uri": uris[1]}}),
                    ));
                    server.handle_message(notification("textDocument/didOpen", json!({"textDocument": {
                        "uri": uris[1], "languageId": "mizar", "version": 7, "text": "definition\nend;\n:: 🦀\n"}})));
                    let (_, loaded, input) = session
                        .captured()
                        .native_sources()
                        .unwrap()
                        .iter()
                        .find(|(_, _, input)| input.module_path.as_str() == "clean")
                        .unwrap();
                    assert_eq!(server.document(&uris[1]), Some((7, loaded.text.as_ref())));
                    let SourceOriginInput::OpenBuffer { text, .. } = &input.origin else {
                        unreachable!()
                    };
                    assert_ne!(server.document(&uris[1]).unwrap().1, text.as_ref());
                }
                "closed" => {
                    server.handle_message(notification(
                        "textDocument/didClose",
                        json!({
                    "textDocument": {"uri": uris[1]}}),
                    ));
                }
                "uninitialized" => server = Server::default(),
                "shutdown" => {
                    server.handle_message(request("shutdown", Value::Null));
                }
                "exit" => {
                    server.handle_message(notification("exit", Value::Null));
                }
                _ => {}
            }
            let invalid_registry = SnapshotRegistry::new();
            let registry = if mode == "invalid-registry" {
                &invalid_registry
            } else {
                &snapshots
            };
            let mut ordinary = None;
            let mut original_records = Vec::new();
            for route in ["ordinary", "index", "notifications"] {
                let owner = Arc::new(PhaseOutputPublisher::new(
                    Arc::new(IrStorageService::new()),
                    Arc::new(SnapshotHandleRegistry::new()),
                ));
                owner.register_current_snapshot(snapshot);
                for name in ["main", "clean"] {
                    for (phase, kind) in
                        [("SourceLoad", "SourceUnit"), ("Frontend", "FrontendOutput")]
                    {
                        if (mode == "denied-source" && phase == "SourceLoad")
                            || (mode == "denied-clean-frontend"
                                && name == "clean"
                                && phase == "Frontend")
                        {
                            continue;
                        }
                        owner.allow_work_unit(AllowedWorkUnit::new(
                            IrPhase::new(phase),
                            OutputKind::new(kind),
                            WorkUnit::new(format!("{:?}:{:?}", "alpha", name)),
                        ));
                    }
                }
                let mut builder = PhaseRegistryBuilder::new();
                builder.register_source_load();
                builder.register_frontend(vec![]);
                let mut driver = CompilerDriver::new(builder.build().unwrap());
                if mode != "missing-publisher" {
                    driver = driver.with_output_publisher(owner);
                }
                let mut input = DriverSubmitInput::new(
                    plan_request.clone(),
                    packages.clone(),
                    lock.clone(),
                    layout.clone(),
                );
                input.dependency_overlay = ModuleDependencyOverlay::complete(vec![]);
                let result = match route {
                    "ordinary" => driver
                        .submit_captured(session.clone(), &ids, registry, input, Default::default())
                        .map(|s| (s, None)),
                    "index" => driver
                        .submit_captured(
                            session.clone(),
                            &ids,
                            registry,
                            input,
                            mizar_driver::driver::SubmitOptions {
                                diagnostic_index: Some(
                                    mizar_driver::driver::DiagnosticIndexMode::Native,
                                ),
                                ..Default::default()
                            },
                        )
                        .map(|mut submission| {
                            let index = submission.diagnostic_index.take();
                            (submission, index)
                        })
                        .map(|(s, index)| {
                            assert_eq!(
                                index.is_some(),
                                !matches!(mode, "generated" | "missing-publisher"),
                                "{mode}"
                            );
                            if let Some(index) = index {
                                original_records = index.records().to_vec();
                                if mode == "denied-source" {
                                    assert!(index.is_empty());
                                } else if mode == "denied-clean-frontend" {
                                    assert_eq!(
                                        original_records
                                            .iter()
                                            .map(|record| record.code().number())
                                            .collect::<Vec<_>>(),
                                        vec![33, 52]
                                    );
                                }
                                if mode == "resolver" {
                                    assert!(original_records.iter().any(|record| {
                                        record.phase() ==
                                        mizar_diagnostics::failure_record::PipelinePhase::Resolver
                                    }));
                                }
                            }
                            (s, None)
                        }),
                    _ => server.submit_native_frontend_diagnostics(
                        &mut driver,
                        session.clone(),
                        &ids,
                        registry,
                        input,
                    ),
                };
                if mode == "invalid-registry" {
                    assert!(
                        matches!(
                            &result,
                            Err(DriverSubmitError::InvalidCapturedSession { .. })
                        ),
                        "{mode}/{route}: {result:?}"
                    );
                } else {
                    assert!(result.is_ok(), "{mode}/{route}: {result:?}");
                }
                if route == "ordinary" {
                    ordinary = Some(result.map(|(submission, _)| submission));
                    continue;
                }
                let (submission, notifications) = match result {
                    Ok(value) => {
                        assert_eq!(
                            Ok(value.0.clone()),
                            *ordinary.as_ref().unwrap(),
                            "{mode}/{route}"
                        );
                        value
                    }
                    Err(error) => {
                        assert_eq!(Err(error), *ordinary.as_ref().unwrap(), "{mode}/{route}");
                        continue;
                    }
                };
                if !matches!(mode, "generated" | "missing-publisher") {
                    assert_eq!(
                        driver.validate_current_native_diagnostics(session.id(), registry),
                        Ok(()),
                        "{mode}"
                    );
                    let run = submission.scheduler_run.as_ref().unwrap();
                    if matches!(mode, "denied-source" | "denied-clean-frontend") {
                        for (_, _, input) in driver
                            .session(session.id())
                            .unwrap()
                            .captured()
                            .native_sources()
                            .unwrap()
                        {
                            let SourceOriginInput::OpenBuffer {
                                uri,
                                actual_version,
                                text,
                                ..
                            } = &input.origin
                            else {
                                unreachable!()
                            };
                            assert_eq!(
                                server.document(uri),
                                Some((i32::try_from(*actual_version).unwrap(), text.as_ref()))
                            );
                        }
                    }
                    for task in submission
                        .task_graph
                        .as_ref()
                        .unwrap()
                        .tasks()
                        .iter()
                        .filter(|task| task.kind == TaskKind::SourceLoad)
                    {
                        let source_results = &run.phase_results[&task.id];
                        assert_eq!(source_results.len(), 1);
                        assert_eq!(
                            source_results[0].status,
                            if mode == "denied-source" {
                                PhaseStatus::Blocking
                            } else {
                                PhaseStatus::Complete
                            }
                        );
                        assert!(source_results[0].diagnostics.is_empty());
                        if matches!(mode, "denied-source" | "denied-clean-frontend") {
                            let frontend = submission
                                .task_graph
                                .as_ref()
                                .unwrap()
                                .tasks()
                                .iter()
                                .find(|frontend| {
                                    frontend.kind == TaskKind::Frontend
                                        && frontend.unit == task.unit
                                })
                                .unwrap();
                            if mode == "denied-source" {
                                assert!(source_results[0].output_refs.is_empty());
                                assert!(!run.phase_results.contains_key(&frontend.id));
                            } else {
                                let frontend_results = &run.phase_results[&frontend.id];
                                assert_eq!(frontend_results.len(), 1);
                                let mizar_build::task_graph::WorkUnit::Module { module } =
                                    &task.unit
                                else {
                                    unreachable!()
                                };
                                assert_eq!(
                                    frontend_results[0].status,
                                    if module.path.as_str() == "clean" {
                                        PhaseStatus::Blocking
                                    } else {
                                        PhaseStatus::Fatal
                                    }
                                );
                                assert_eq!(
                                    frontend_results[0].diagnostics.is_empty(),
                                    module.path.as_str() == "clean"
                                );
                            }
                        }
                    }
                }
                if route == "notifications" {
                    if !matches!(mode, "fatal" | "clean") {
                        assert_eq!(notifications, None, "{mode}");
                        if mode == "alias" {
                            assert!(
                                driver
                                    .session(session.id())
                                    .unwrap()
                                    .captured()
                                    .native_sources()
                                    .unwrap()
                                    .iter()
                                    .find(|(_, _, input)| input.module_path.as_str() == "main")
                                    .unwrap()
                                    .1
                                    .loading_map
                                    .is_none()
                            );
                        }
                        continue;
                    }
                    let notifications = notifications.unwrap();
                    let expected_diagnostics = if mode == "clean" {
                        vec![]
                    } else {
                        vec![
                            json!({"range": {"start": {"line": 0, "character": 9},
                            "end": {"line": 0, "character": 12}}, "code": "E0033",
                            "message": "unmatched `end` in lexical scope skeleton", "severity": 1}),
                            json!({"range": {"start": {"line": 0, "character": 9},
                            "end": {"line": 0, "character": 12}}, "code": "E0052",
                            "message": "`end` has no matching block opener", "severity": 1}),
                        ]
                    };
                    assert_eq!(
                        notifications,
                        vec![
                            json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                            "params": {"uri": uris[1], "version": 7, "diagnostics": []}}),
                            json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                            "params": {"uri": uris[0], "version": 7, "diagnostics": expected_diagnostics}})
                        ],
                        "{mode}"
                    );
                    let diagnostics = notifications[1]["params"]["diagnostics"]
                        .as_array()
                        .unwrap();
                    assert_eq!(diagnostics.len(), original_records.len());
                    for (diagnostic, record) in diagnostics.iter().zip(&original_records) {
                        assert_eq!(diagnostic["code"], record.code().to_string());
                        assert_eq!(diagnostic["message"], record.message());
                        assert_eq!(
                            record.severity(),
                            mizar_diagnostics::registry::DiagnosticSeverity::Error
                        );
                    }
                }
            }
            fs::remove_dir_all(root).unwrap();
        }
    }

    fn native_wire_fixture(
        server: &mut Server,
        ids: &mizar_session::InMemorySessionIdAllocator,
        snapshots: &mizar_session::SnapshotRegistry,
    ) -> (
        mizar_driver::request::BuildSession,
        impl Fn() -> mizar_driver::driver::DriverSubmitInput<
            mizar_build::module_index::StaticSourceLayout,
        > + use<>,
        impl Fn() -> (
            mizar_driver::driver::CompilerDriver,
            std::sync::Arc<mizar_ir::publisher::PhaseOutputPublisher>,
        ) + use<>,
    ) {
        use mizar_build::{
            module_index::{StaticSourceLayout, build_module_index},
            planner::{
                DependencySelection, PlanRequest, WorkspacePackage, parse_lockfile,
                parse_package_manifest, produce_build_plan,
            },
            task_graph::ModuleDependencyOverlay,
        };
        use mizar_driver::{
            driver::{CompilerDriver, DriverSubmitInput},
            registry::PhaseRegistryBuilder,
            request::{
                BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
                BuildRequestGeneration, BuildRequestOrigin, BuildTargets, DependencyInputSet,
                SourceInputSet, VerifierConfigInput,
            },
        };
        use mizar_ir::{
            identity::{OutputKind, PipelinePhase, SnapshotHandleRegistry, WorkUnit},
            publisher::{AllowedWorkUnit, PhaseOutputPublisher},
            storage::IrStorageService,
        };
        use mizar_session::{
            DiskSourceLoader, Hash, PackageId, SourceOriginInput, ToolchainInfo, WorkspaceRoot,
        };
        use std::{fs, sync::Arc, time::SystemTime};

        let root = std::env::temp_dir().join(format!(
            "mizar-lsp-wire-🦀-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("alpha/src")).unwrap();
        for name in ["main", "clean"] {
            fs::write(root.join(format!("alpha/src/{name}.miz")), [0xff]).unwrap();
        }
        let workspace = WorkspaceRoot::new(root.to_str().unwrap());
        let plan_request = PlanRequest {
            workspace_root: workspace.clone(),
            dependency_selection: DependencySelection::Normal,
            toolchain: ToolchainInfo::new("mizar-evo-test"),
        };
        let packages = vec![WorkspacePackage {
            member_path: "alpha".into(),
            manifest: parse_package_manifest("[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n")
                .unwrap(),
        }];
        let lock = parse_lockfile("schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n").unwrap();
        let plan =
            produce_build_plan(plan_request.clone(), packages.clone(), lock.clone()).unwrap();
        let layout = StaticSourceLayout::from_native_disk(&plan).unwrap();
        let index = build_module_index(&plan, &layout, &[]).unwrap();
        let loader = DiskSourceLoader::new(root.join("alpha"));
        initialize(server);
        let mut descriptors = Vec::new();
        for (name, raw) in [
            ("main", "\u{feff}::=🦀=::end\r\n:: suffix\r"),
            ("clean", "\u{feff}definition\r\nend;\r\n:: 🦀\r\n"),
        ] {
            let uri = format!(
                "file://{}{}",
                if cfg!(windows) { "/" } else { "" },
                root.join(format!("alpha/src/{name}.miz"))
                    .to_str()
                    .unwrap()
                    .replace('\\', "/")
            );
            server.handle_message(notification(
                "textDocument/didOpen",
                json!({"textDocument": {
                "uri": uri, "languageId": "mizar", "version": 7, "text": raw}}),
            ));
            let descriptor = index
                .native_open_buffer_input(
                    &workspace,
                    &PackageId::new("alpha"),
                    &loader,
                    SourceOriginInput::OpenBuffer {
                        uri,
                        expected_version: 7,
                        actual_version: 7,
                        text: raw.into(),
                    },
                )
                .unwrap();
            descriptors.push((loader.clone(), descriptor));
        }
        let session = BuildRequestDraft {
            lane: BuildLaneId::new(1),
            origin: BuildRequestOrigin::Batch(BatchRequest {
                invocation: BatchInvocation::default(),
            }),
            generation: BuildRequestGeneration::new(0),
            workspace_root: workspace,
            profile: BuildProfile::new("check"),
            targets: BuildTargets::default(),
            source_inputs: SourceInputSet { versions: vec![] },
            dependency_inputs: DependencyInputSet::new(
                vec![],
                Hash::from_bytes([1; 32]),
                ToolchainInfo::new("mizar-evo-test"),
            ),
            verifier_config: VerifierConfigInput::new(Hash::from_bytes([2; 32])),
        }
        .capture_sources(ids, snapshots, descriptors)
        .unwrap();
        for name in ["main", "clean"] {
            fs::remove_file(root.join(format!("alpha/src/{name}.miz"))).unwrap();
        }
        let snapshot = session.captured().snapshot().id;
        let drivers = move || {
            let owner = Arc::new(PhaseOutputPublisher::new(
                Arc::new(IrStorageService::new()),
                Arc::new(SnapshotHandleRegistry::new()),
            ));
            owner.register_current_snapshot(snapshot);
            for name in ["main", "clean"] {
                for (phase, kind) in [("SourceLoad", "SourceUnit"), ("Frontend", "FrontendOutput")]
                {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        PipelinePhase::new(phase),
                        OutputKind::new(kind),
                        WorkUnit::new(format!("{:?}:{:?}", "alpha", name)),
                    ));
                }
            }
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            (
                CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone()),
                owner,
            )
        };
        let input = move || {
            let mut input = DriverSubmitInput::new(
                plan_request.clone(),
                packages.clone(),
                lock.clone(),
                layout.clone(),
            );
            input.dependency_overlay = ModuleDependencyOverlay::complete(vec![]);
            input
        };
        (session, input, drivers)
    }

    #[derive(Default)]
    struct NativeWireWriter {
        bytes: Vec<u8>,
        calls: Vec<&'static str>,
        body: bool,
        fail: Option<&'static str>,
        partial: bool,
        obsolete_on_flush: Option<(
            std::sync::Arc<mizar_ir::publisher::PhaseOutputPublisher>,
            mizar_session::BuildSnapshotId,
        )>,
    }

    impl Write for NativeWireWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let operation = if self.body { "body" } else { "header" };
            self.calls.push(operation);
            if self.fail == Some(operation) && self.partial {
                return Err(io::Error::other(operation));
            }
            let count = if self.fail == Some(operation) {
                self.partial = true;
                bytes.len().min(2)
            } else {
                bytes.len()
            };
            self.bytes.extend_from_slice(&bytes[..count]);
            self.body |= self.bytes.ends_with(b"\r\n\r\n");
            Ok(count)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.calls.push("flush");
            if self.fail == Some("flush") {
                return Err(io::Error::other("flush"));
            }
            if let Some((owner, snapshot)) = self.obsolete_on_flush.take() {
                owner.mark_obsolete(snapshot).unwrap();
            }
            self.body = false;
            Ok(())
        }
    }

    #[test]
    fn native_wire_frames_preserve_original_records_raw_ranges_and_submission() {
        let mut server = Server::default();
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        let snapshots = mizar_session::SnapshotRegistry::new();
        let (session, input, drivers) = native_wire_fixture(&mut server, &ids, &snapshots);
        let (mut ordinary_driver, _) = drivers();
        let ordinary = ordinary_driver
            .submit_captured(
                session.clone(),
                &ids,
                &snapshots,
                input(),
                Default::default(),
            )
            .unwrap();
        let (mut indexed_driver, _) = drivers();
        let (_, index) = indexed_driver
            .submit_captured(
                session.clone(),
                &ids,
                &snapshots,
                input(),
                mizar_driver::driver::SubmitOptions {
                    diagnostic_index: Some(mizar_driver::driver::DiagnosticIndexMode::Native),
                    ..Default::default()
                },
            )
            .map(|mut submission| {
                let index = submission.diagnostic_index.take();
                (submission, index)
            })
            .unwrap();
        let index = index.unwrap();
        let (mut driver, _) = drivers();
        let mut writer = NativeWireWriter::default();
        let (submission, delivery) = server
            .submit_native_frontend_diagnostics_to_writer(
                &mut driver,
                session,
                &ids,
                &snapshots,
                input(),
                &mut writer,
            )
            .unwrap();
        assert_eq!(submission, ordinary);
        assert_eq!(delivery.unwrap(), 2);
        assert_eq!(writer.calls.iter().filter(|&&op| op == "flush").count(), 2);
        let mut remaining = writer.bytes.as_slice();
        let mut messages = Vec::new();
        while !remaining.is_empty() {
            let header_end = remaining
                .windows(4)
                .position(|part| part == b"\r\n\r\n")
                .unwrap();
            let header = std::str::from_utf8(&remaining[..header_end]).unwrap();
            let length: usize = header
                .strip_prefix("Content-Length: ")
                .unwrap()
                .parse()
                .unwrap();
            remaining = &remaining[header_end + 4..];
            let body = &remaining[..length];
            assert!(body.len() > std::str::from_utf8(body).unwrap().chars().count());
            messages.push(serde_json::from_slice::<Value>(body).unwrap());
            remaining = &remaining[length..];
        }
        let uris = server.documents.keys().collect::<Vec<_>>();
        assert_eq!(
            messages[0],
            json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
            "params": {"uri": uris[0], "version": 7, "diagnostics": []}})
        );
        let diagnostics = vec![
            json!({"range": {"start": {"line": 0, "character": 9}, "end": {"line": 0, "character": 12}},
                "code": "E0033", "message": "unmatched `end` in lexical scope skeleton", "severity": 1}),
            json!({"range": {"start": {"line": 0, "character": 9}, "end": {"line": 0, "character": 12}},
                "code": "E0052", "message": "`end` has no matching block opener", "severity": 1}),
        ];
        assert_eq!(
            messages[1],
            json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
            "params": {"uri": uris[1], "version": 7, "diagnostics": diagnostics}})
        );
        assert_eq!(diagnostics.len(), index.records().len());
        for (diagnostic, record) in diagnostics.iter().zip(index.records()) {
            assert_eq!(diagnostic["code"], record.code().to_string());
            assert_eq!(diagnostic["message"], record.message());
            assert_eq!(
                record.severity(),
                mizar_diagnostics::registry::DiagnosticSeverity::Error
            );
        }
        std::fs::remove_dir_all(input().plan_request.workspace_root.as_str()).unwrap();
    }

    #[test]
    fn native_wire_withholds_ineligible_handoffs_and_preserves_driver_errors() {
        let mut server = Server::default();
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        let snapshots = mizar_session::SnapshotRegistry::new();
        let (session, input, drivers) = native_wire_fixture(&mut server, &ids, &snapshots);
        let (mut ordinary_driver, _) = drivers();
        let ordinary = ordinary_driver
            .submit_captured(
                session.clone(),
                &ids,
                &snapshots,
                input(),
                Default::default(),
            )
            .unwrap();
        let uri = server.documents.keys().next().unwrap().clone();
        let original = server.document(&uri).unwrap().1.to_owned();
        for mode in ["control", "version", "raw", "invalid-registry"] {
            let mut current = Server::default();
            initialize(&mut current);
            for (uri, (version, text, ..)) in &server.documents {
                current.handle_message(notification(
                    "textDocument/didOpen",
                    json!({"textDocument": {
                    "uri": uri, "languageId": "mizar", "version": version, "text": text}}),
                ));
            }
            if mode == "version" {
                current.handle_message(notification("textDocument/didChange", json!({
                    "textDocument": {"uri": uri, "version": 8}, "contentChanges": [{"text": original}]})));
                assert_eq!(current.document(&uri), Some((8, original.as_str())));
            } else if mode == "raw" {
                let normalized = original
                    .trim_start_matches('\u{feff}')
                    .replace("\r\n", "\n");
                current.handle_message(notification(
                    "textDocument/didClose",
                    json!({"textDocument": {"uri": uri}}),
                ));
                current.handle_message(notification(
                    "textDocument/didOpen",
                    json!({"textDocument": {
                    "uri": uri, "languageId": "mizar", "version": 7, "text": normalized}}),
                ));
                let loaded = session.captured().native_sources().unwrap().iter()
                    .find(|(_, _, input)| matches!(&input.origin,
                        mizar_session::SourceOriginInput::OpenBuffer { uri: captured, .. } if captured == &uri))
                    .unwrap();
                assert_eq!(loaded.1.text.as_ref(), normalized);
                assert_ne!(original, normalized);
            }
            let invalid = mizar_session::SnapshotRegistry::new();
            let registry = if mode == "invalid-registry" {
                &invalid
            } else {
                &snapshots
            };
            let (mut driver, _) = drivers();
            let mut writer = NativeWireWriter::default();
            let result = current.submit_native_frontend_diagnostics_to_writer(
                &mut driver,
                session.clone(),
                &ids,
                registry,
                input(),
                &mut writer,
            );
            if mode == "invalid-registry" {
                let (mut ordinary_driver, _) = drivers();
                let expected = ordinary_driver
                    .submit_captured(session.clone(), &ids, registry, input(), Default::default())
                    .unwrap_err();
                assert!(matches!(
                    expected,
                    mizar_driver::driver::DriverSubmitError::InvalidCapturedSession { .. }
                ));
                assert_eq!(result.unwrap_err(), expected);
            } else {
                let (submission, delivery) = result.unwrap();
                assert_eq!(submission, ordinary);
                assert_eq!(delivery.unwrap(), if mode == "control" { 2 } else { 0 });
                assert_eq!(
                    driver.validate_current_native_diagnostics(session.id(), registry),
                    Ok(())
                );
            }
            if mode != "control" {
                assert!(writer.bytes.is_empty());
                assert!(writer.calls.is_empty());
            }
        }
        std::fs::remove_dir_all(input().plan_request.workspace_root.as_str()).unwrap();
    }

    #[test]
    fn native_wire_stops_on_io_errors_and_rechecks_real_publisher_after_each_frame() {
        let mut server = Server::default();
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        let snapshots = mizar_session::SnapshotRegistry::new();
        let (session, input, drivers) = native_wire_fixture(&mut server, &ids, &snapshots);
        let (mut ordinary_driver, _) = drivers();
        let ordinary = ordinary_driver
            .submit_captured(
                session.clone(),
                &ids,
                &snapshots,
                input(),
                Default::default(),
            )
            .unwrap();
        let mut complete_prefix = None;
        for mode in ["header", "body", "flush", "obsolete"] {
            let (mut driver, owner) = drivers();
            let mut writer = NativeWireWriter {
                fail: (mode != "obsolete").then_some(mode),
                obsolete_on_flush: (mode == "obsolete")
                    .then_some((owner, session.captured().snapshot().id)),
                ..NativeWireWriter::default()
            };
            let (submission, delivery) = server
                .submit_native_frontend_diagnostics_to_writer(
                    &mut driver,
                    session.clone(),
                    &ids,
                    &snapshots,
                    input(),
                    &mut writer,
                )
                .unwrap();
            assert_eq!(submission, ordinary);
            if mode == "obsolete" {
                assert_eq!(delivery.unwrap(), 1);
                assert!(
                    driver
                        .validate_current_native_diagnostics(session.id(), &snapshots)
                        .is_err()
                );
                assert_eq!(writer.calls.iter().filter(|&&op| op == "flush").count(), 1);
                assert_eq!(&writer.bytes, complete_prefix.as_ref().unwrap());
            } else {
                let error = delivery.unwrap_err();
                assert_eq!(error.kind(), io::ErrorKind::Other);
                assert_eq!(error.to_string(), mode);
                assert_eq!(writer.calls.last(), Some(&mode));
                if mode != "flush" {
                    assert_eq!(writer.calls.iter().filter(|&&op| op == mode).count(), 2);
                }
                assert_eq!(
                    writer.calls.iter().filter(|&&op| op == "flush").count(),
                    usize::from(mode == "flush")
                );
                if mode == "header" {
                    assert_eq!(writer.bytes, b"Co");
                } else {
                    let header_end = writer
                        .bytes
                        .windows(4)
                        .position(|part| part == b"\r\n\r\n")
                        .unwrap();
                    let length: usize = std::str::from_utf8(&writer.bytes[..header_end])
                        .unwrap()
                        .strip_prefix("Content-Length: ")
                        .unwrap()
                        .parse()
                        .unwrap();
                    assert_eq!(
                        writer.bytes.len() - header_end - 4,
                        if mode == "body" { 2 } else { length }
                    );
                    if mode == "flush" {
                        let message: Value =
                            serde_json::from_slice(&writer.bytes[header_end + 4..]).unwrap();
                        assert_eq!(message["params"]["diagnostics"], json!([]));
                        complete_prefix = Some(writer.bytes);
                    }
                }
            }
        }
        std::fs::remove_dir_all(input().plan_request.workspace_root.as_str()).unwrap();
    }

    #[test]
    fn open_document_lifetime_retains_open_across_changes_and_separates_uris() {
        let mut server = Server::default();
        let uri = "file:///a.miz";
        assert!(server.document_lifetime(uri).is_none());
        initialize(&mut server);
        server.handle_message(notification(
            "textDocument/didOpen",
            json!({"textDocument": {
            "uri": uri, "languageId": "mizar", "version": 7, "text": "alpha"}}),
        ));
        let first = server.document_lifetime(uri).unwrap();
        assert!(first.ptr_eq(&server.document_lifetime(uri).unwrap()));
        for (version, changes) in [(8, json!([{"text": "\u{feff}beta\r\n"}])), (9, json!([]))] {
            let response = server.handle_message(notification(
                "textDocument/didChange",
                json!({
                "textDocument": {"uri": uri, "version": version}, "contentChanges": changes}),
            ));
            assert_eq!(response, None);
            assert_eq!(server.document(uri), Some((version, "\u{feff}beta\r\n")));
            assert!(first.ptr_eq(&server.document_lifetime(uri).unwrap()));
        }
        server.handle_message(notification("textDocument/didOpen", json!({"textDocument": {
            "uri": "file:///b.miz", "languageId": "mizar", "version": 9, "text": "\u{feff}beta\r\n"}})));
        assert!(!first.ptr_eq(&server.document_lifetime("file:///b.miz").unwrap()));
        assert!(first.ptr_eq(&server.document_lifetime(uri).unwrap()));
    }

    #[test]
    fn open_document_lifetime_reopen_and_other_server_have_distinct_handles() {
        let mut server = Server::default();
        initialize(&mut server);
        let uri = "file:///a.miz";
        let open = notification(
            "textDocument/didOpen",
            json!({"textDocument": {
            "uri": uri, "languageId": "mizar", "version": 7, "text": "alpha"}}),
        );
        server.handle_message(open.clone());
        let old = server.document_lifetime(uri).unwrap();
        let _retained_old_allocation = old.upgrade().unwrap();
        let response = server.handle_message(notification(
            "textDocument/didClose",
            json!({
            "textDocument": {"uri": uri}}),
        ));
        assert_eq!(response, None);
        assert!(server.document_lifetime(uri).is_none());
        assert_eq!(server.document(uri), None);
        server.handle_message(open.clone());
        let reopened = server.document_lifetime(uri).unwrap();
        assert!(!old.ptr_eq(&reopened));
        assert_eq!(server.document(uri), Some((7, "alpha")));
        let mut other = Server::default();
        initialize(&mut other);
        other.handle_message(open);
        let independent = other.document_lifetime(uri).unwrap();
        assert!(!old.ptr_eq(&independent));
        assert!(!reopened.ptr_eq(&independent));
        assert!(reopened.ptr_eq(&server.document_lifetime(uri).unwrap()));
    }

    #[test]
    fn open_document_lifetime_ignores_invalid_traffic_and_withholds_terminal_handles() {
        let mut server = Server::default();
        let uri = "file:///a.miz";
        let open = notification(
            "textDocument/didOpen",
            json!({"textDocument": {
            "uri": uri, "languageId": "mizar", "version": 7, "text": "alpha"}}),
        );
        assert_eq!(server.handle_message(open.clone()), None);
        assert!(server.document_lifetime(uri).is_none());
        assert_eq!(server.document(uri), None);
        initialize(&mut server);
        server.handle_message(notification(
            "textDocument/didOpen",
            json!({"textDocument": {
            "uri": uri, "version": 7, "text": "alpha"}}),
        ));
        assert!(server.document_lifetime(uri).is_none());
        server.handle_message(open.clone());
        let live = server.document_lifetime(uri).unwrap();
        for (method, params) in [
            (
                "textDocument/didOpen",
                json!({"textDocument": {
                "uri": uri, "languageId": "mizar", "version": 20, "text": "other"}}),
            ),
            (
                "textDocument/didChange",
                json!({"textDocument": {"uri": uri, "version": 8},
                "contentChanges": [{"text": "other", "range": {}}]}),
            ),
            (
                "textDocument/didChange",
                json!({"textDocument": {"uri": uri, "version": 7},
                "contentChanges": [{"text": "other"}]}),
            ),
            (
                "textDocument/didClose",
                json!({"textDocument": {"uri": "file:///b.miz"}}),
            ),
        ] {
            assert_eq!(server.handle_message(notification(method, params)), None);
            assert!(live.ptr_eq(&server.document_lifetime(uri).unwrap()));
            assert_eq!(server.document(uri), Some((7, "alpha")));
        }
        assert_eq!(
            server.handle_message(request("shutdown", Value::Null)),
            Some(json!({"jsonrpc": "2.0", "id": 7, "result": null}))
        );
        assert!(server.document_lifetime(uri).is_none());
        for message in [
            open.clone(),
            notification(
                "textDocument/didChange",
                json!({"textDocument": {"uri": uri, "version": 8},
                "contentChanges": [{"text": "other"}]}),
            ),
            notification(
                "textDocument/didClose",
                json!({"textDocument": {"uri": uri}}),
            ),
            notification("exit", Value::Null),
        ] {
            assert_eq!(server.handle_message(message), None);
            assert!(server.document_lifetime(uri).is_none());
            assert_eq!(server.document(uri), Some((7, "alpha")));
        }
        let mut abrupt = Server::default();
        initialize(&mut abrupt);
        abrupt.handle_message(open);
        assert!(abrupt.document_lifetime(uri).is_some());
        assert_eq!(
            abrupt.handle_message(notification("exit", Value::Null)),
            None
        );
        assert!(abrupt.document_lifetime(uri).is_none());
        assert_eq!(abrupt.document(uri), Some((7, "alpha")));
    }

    fn originating_native_inputs(
        server: &Server,
        template: &mizar_driver::request::BuildSession,
        input: mizar_driver::driver::DriverSubmitInput<
            mizar_build::module_index::StaticSourceLayout,
        >,
    ) -> (
        mizar_driver::request::BuildRequestDraft,
        mizar_build::module_index::ModuleIndex,
        Vec<(
            mizar_session::DiskSourceLoader,
            mizar_session::PackageId,
            String,
        )>,
    ) {
        use mizar_driver::request::{BuildRequestDraft, SourceInputSet};
        let root = std::path::Path::new(input.plan_request.workspace_root.as_str()).join("alpha");
        // The existing fixture removes these entries; association needs ordinary native files,
        // while invalid disk bytes ensure that capture must use the actual editor text.
        for name in ["main", "clean"] {
            std::fs::write(root.join(format!("src/{name}.miz")), [0xff]).unwrap();
        }
        let plan = mizar_build::planner::produce_build_plan(
            input.plan_request,
            input.workspace_packages,
            input.lockfile,
        )
        .unwrap();
        let index = mizar_build::module_index::build_module_index(&plan, &input.source_layout, &[])
            .unwrap();
        let request = template.request();
        let draft = BuildRequestDraft {
            lane: request.lane,
            origin: request.origin.clone(),
            generation: request.generation,
            workspace_root: request.workspace_root.clone(),
            profile: request.profile.clone(),
            targets: request.targets.clone(),
            source_inputs: SourceInputSet { versions: vec![] },
            dependency_inputs: request.dependency_inputs.clone(),
            verifier_config: request.verifier_config,
        };
        let sources = server
            .documents
            .keys()
            .map(|uri| {
                (
                    mizar_session::DiskSourceLoader::new(&root),
                    mizar_session::PackageId::new("alpha"),
                    uri.clone(),
                )
            })
            .collect();
        (draft, index, sources)
    }

    #[derive(Clone, Default)]
    struct ObservingAllocator {
        inner: std::sync::Arc<mizar_session::InMemorySessionIdAllocator>,
        leases: std::sync::Arc<std::sync::Mutex<Vec<mizar_session::SnapshotLeaseId>>>,
        fail_at: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl mizar_session::SessionIdAllocator for ObservingAllocator {
        fn next_session_id(&self) -> Result<mizar_session::BuildSessionId, mizar_session::IdError> {
            self.inner.next_session_id()
        }
        fn next_request_id(&self) -> Result<mizar_session::BuildRequestId, mizar_session::IdError> {
            self.inner.next_request_id()
        }
        fn next_source_id(
            &self,
            snapshot: mizar_session::BuildSnapshotId,
        ) -> Result<mizar_session::SourceId, mizar_session::IdError> {
            self.inner.next_source_id(snapshot)
        }
        fn next_source_map_id(
            &self,
            snapshot: mizar_session::BuildSnapshotId,
        ) -> Result<mizar_session::SourceMapId, mizar_session::IdError> {
            self.inner.next_source_map_id(snapshot)
        }
        fn next_lease_id(
            &self,
            snapshot: mizar_session::BuildSnapshotId,
        ) -> Result<mizar_session::SnapshotLeaseId, mizar_session::IdError> {
            let mut leases = self.leases.lock().unwrap();
            if self.fail_at.load(std::sync::atomic::Ordering::SeqCst) == leases.len() + 1 {
                return Err(mizar_session::IdError::AllocatorOverflow);
            }
            let lease = self.inner.next_lease_id(snapshot)?;
            leases.push(lease);
            Ok(lease)
        }
    }

    #[test]
    fn retained_native_sources_share_original_objects_across_equal_id_replacement() {
        let mut server = Server::default();
        let ids = ObservingAllocator::default();
        let (template, input, drivers) = native_wire_fixture(
            &mut server,
            &ids.inner,
            &mizar_session::SnapshotRegistry::new(),
        );
        let (draft, index, sources) = originating_native_inputs(&server, &template, input());
        let snapshots =
            std::sync::Arc::new(mizar_session::SnapshotRegistry::with_allocator(ids.clone()));
        let uri = sources[1].2.clone();
        let raw = server.document(&uri).unwrap().1.to_owned();
        let first = server
            .capture_native_open_buffers(draft.clone(), &ids, &snapshots, &index, sources.clone())
            .unwrap();
        let old = server.native_open_buffer_snapshot().unwrap();
        assert!(std::sync::Arc::ptr_eq(&old, &first.snapshot));
        let original = old.captured().native_sources().unwrap();
        assert!(std::ptr::eq(
            original,
            first
                .session
                .as_ref()
                .unwrap()
                .captured()
                .native_sources()
                .unwrap()
        ));
        let cloned = first.session.as_ref().unwrap().clone();
        assert!(std::ptr::eq(
            original,
            cloned.captured().native_sources().unwrap()
        ));
        for (version, changes) in [
            (8, json!([])),
            (
                9,
                json!([{"text": raw.trim_start_matches('\u{feff}').replace("\r\n", "\n")}]),
            ),
        ] {
            server.handle_message(notification("textDocument/didChange", json!({"textDocument": {"uri": uri, "version": version}, "contentChanges": changes})));
            assert!(server.native_open_buffer_snapshot().is_none());
            let next = server
                .capture_native_open_buffers(
                    draft.clone(),
                    &ids,
                    &snapshots,
                    &index,
                    sources.clone(),
                )
                .unwrap();
            let current = server.native_open_buffer_snapshot().unwrap();
            assert_eq!(
                current.captured().snapshot().id,
                old.captured().snapshot().id
            );
            assert!(!std::sync::Arc::ptr_eq(&old, &current));
            assert!(!std::ptr::eq(
                original,
                current.captured().native_sources().unwrap()
            ));
            let (_, loaded, descriptor) = current.captured().native_sources().unwrap().iter().find(|(_, _, input)| matches!(&input.origin, mizar_session::SourceOriginInput::OpenBuffer { uri: key, .. } if key == &uri)).unwrap();
            assert_eq!(
                loaded.text.as_ref(),
                raw.trim_start_matches('\u{feff}').replace("\r\n", "\n")
            );
            assert_eq!(
                descriptor.origin,
                mizar_session::SourceOriginInput::OpenBuffer {
                    uri: uri.clone(),
                    expected_version: version,
                    actual_version: version,
                    text: server.document(&uri).unwrap().1.into()
                }
            );
            drop(next);
        }
        let (_, loaded, descriptor) = &original[1];
        assert_eq!(
            descriptor.origin,
            mizar_session::SourceOriginInput::OpenBuffer {
                uri: uri.clone(),
                expected_version: 7,
                actual_version: 7,
                text: raw.into()
            }
        );
        assert!(loaded.loading_map.is_some());
        for wire in [false, true] {
            let earlier = server
                .capture_native_open_buffers(
                    draft.clone(),
                    &ids,
                    &snapshots,
                    &index,
                    sources.clone(),
                )
                .unwrap();
            let latest = server
                .capture_native_open_buffers(
                    draft.clone(),
                    &ids,
                    &snapshots,
                    &index,
                    sources.clone(),
                )
                .unwrap();
            let current = server.native_open_buffer_snapshot().unwrap();
            assert_eq!(
                earlier.snapshot.captured().snapshot().id,
                current.captured().snapshot().id
            );
            let session_id = earlier.session.as_ref().unwrap().id();
            let (mut driver, owner) = drivers();
            owner.register_current_snapshot(current.captured().snapshot().id);
            if wire {
                let (mut reference, owner) = drivers();
                owner.register_current_snapshot(current.captured().snapshot().id);
                let (expected, messages) = server
                    .submit_native_frontend_diagnostics(
                        &mut reference,
                        earlier.session.as_ref().unwrap().clone(),
                        &ids,
                        &snapshots,
                        input(),
                    )
                    .unwrap();
                assert_eq!(messages.unwrap().len(), 2);
                let mut writer = NativeWireWriter::default();
                let (submission, delivery) = server
                    .submit_originating_native_frontend_diagnostics_to_writer(
                        &mut driver,
                        earlier,
                        &ids,
                        &snapshots,
                        input(),
                        &mut writer,
                    )
                    .unwrap();
                assert_eq!(submission, expected);
                assert_eq!(delivery.unwrap(), 0);
                assert!(writer.calls.is_empty());
                assert!(writer.bytes.is_empty());
            } else {
                let (_, messages) = server
                    .submit_originating_native_frontend_diagnostics(
                        &mut driver,
                        earlier,
                        &ids,
                        &snapshots,
                        input(),
                    )
                    .unwrap();
                assert!(messages.is_none());
            }
            let stored = driver
                .session(session_id)
                .unwrap()
                .captured()
                .native_sources()
                .unwrap();
            assert!(!std::ptr::eq(
                stored,
                current.captured().native_sources().unwrap()
            ));
            let session_id = latest.session.as_ref().unwrap().id();
            let (mut driver, owner) = drivers();
            owner.register_current_snapshot(current.captured().snapshot().id);
            if wire {
                let mut writer = NativeWireWriter::default();
                let (_, delivery) = server
                    .submit_originating_native_frontend_diagnostics_to_writer(
                        &mut driver,
                        latest,
                        &ids,
                        &snapshots,
                        input(),
                        &mut writer,
                    )
                    .unwrap();
                assert_eq!(delivery.unwrap(), 2);
                assert!(!writer.calls.is_empty());
                assert!(!writer.bytes.is_empty());
            } else {
                let (_, messages) = server
                    .submit_originating_native_frontend_diagnostics(
                        &mut driver,
                        latest,
                        &ids,
                        &snapshots,
                        input(),
                    )
                    .unwrap();
                assert_eq!(messages.unwrap().len(), 2);
            }
            let stored = driver
                .session(session_id)
                .unwrap()
                .captured()
                .native_sources()
                .unwrap();
            assert!(std::ptr::eq(
                stored,
                current.captured().native_sources().unwrap()
            ));
            assert!(std::ptr::eq(
                &stored[1].1.line_map,
                &current.captured().native_sources().unwrap()[1].1.line_map
            ));
            assert!(stored[1].1.loading_map.is_none());
        }
        let session_id = first.session.as_ref().unwrap().id();
        let (mut driver, owner) = drivers();
        owner.register_current_snapshot(old.captured().snapshot().id);
        let (_, messages) = server
            .submit_originating_native_frontend_diagnostics(
                &mut driver,
                first,
                &ids,
                &snapshots,
                input(),
            )
            .unwrap();
        assert!(messages.is_none());
        let stored = driver
            .session(session_id)
            .unwrap()
            .captured()
            .native_sources()
            .unwrap();
        assert!(std::ptr::eq(original, stored));
        assert!(std::ptr::eq(
            original[1].1.loading_map.as_ref().unwrap(),
            stored[1].1.loading_map.as_ref().unwrap()
        ));
        std::fs::remove_dir_all(input().plan_request.workspace_root.as_str()).unwrap();
    }

    #[test]
    fn retained_native_selection_preserves_ignored_and_failed_capture_but_evicts_changes() {
        let mut server = Server::default();
        let ids = ObservingAllocator::default();
        let (template, input, _) = native_wire_fixture(
            &mut server,
            &ids.inner,
            &mizar_session::SnapshotRegistry::new(),
        );
        let (draft, index, sources) = originating_native_inputs(&server, &template, input());
        let snapshots =
            std::sync::Arc::new(mizar_session::SnapshotRegistry::with_allocator(ids.clone()));
        for mode in ["empty-change", "reopen", "shutdown", "exit"] {
            let mut current = Server::default();
            initialize(&mut current);
            for (uri, (version, text, ..)) in &server.documents {
                current.handle_message(notification("textDocument/didOpen", json!({"textDocument": {"uri": uri, "version": version, "languageId": "mizar", "text": text}})));
            }
            let server = &mut current;
            let capture = server
                .capture_native_open_buffers(
                    draft.clone(),
                    &ids,
                    &snapshots,
                    &index,
                    sources.clone(),
                )
                .unwrap();
            let reader = server.native_open_buffer_snapshot().unwrap();
            let uri = sources[0].2.clone();
            let (version, raw) = server.document(&uri).unwrap();
            let (version, raw) = (version, raw.to_owned());
            for message in [
                notification(
                    "textDocument/didChange",
                    json!({"textDocument": {"uri": uri, "version": version}, "contentChanges": []}),
                ),
                notification(
                    "textDocument/didChange",
                    json!({"textDocument": {"uri": uri, "version": version + 1}, "contentChanges": [{"text": "bad", "range": {}}]}),
                ),
                notification(
                    "textDocument/didOpen",
                    json!({"textDocument": {"uri": "untitled:other", "version": 1, "languageId": "mizar", "text": "other"}}),
                ),
                notification(
                    "textDocument/didChange",
                    json!({"textDocument": {"uri": "untitled:other", "version": 2}, "contentChanges": []}),
                ),
                notification(
                    "textDocument/didClose",
                    json!({"textDocument": {"uri": "untitled:other"}}),
                ),
            ] {
                server.handle_message(message);
                assert!(std::sync::Arc::ptr_eq(
                    &reader,
                    &server.native_open_buffer_snapshot().unwrap()
                ));
            }
            assert!(
                server
                    .capture_native_open_buffers(draft.clone(), &ids, &snapshots, &index, vec![])
                    .is_err()
            );
            let mut invalid = draft.clone();
            invalid.source_inputs = template.request().source_inputs.clone();
            assert!(
                server
                    .capture_native_open_buffers(invalid, &ids, &snapshots, &index, sources.clone())
                    .is_err()
            );
            ids.fail_at.store(
                ids.leases.lock().unwrap().len() + 2,
                std::sync::atomic::Ordering::SeqCst,
            );
            assert!(
                server
                    .capture_native_open_buffers(
                        draft.clone(),
                        &ids,
                        &snapshots,
                        &index,
                        sources.clone()
                    )
                    .is_err()
            );
            ids.fail_at.store(0, std::sync::atomic::Ordering::SeqCst);
            assert!(std::sync::Arc::ptr_eq(
                &reader,
                &server.native_open_buffer_snapshot().unwrap()
            ));
            match mode {
                "empty-change" => {
                    server.handle_message(notification("textDocument/didChange", json!({"textDocument": {"uri": uri, "version": version + 1}, "contentChanges": []})));
                }
                "reopen" => {
                    server.handle_message(notification(
                        "textDocument/didClose",
                        json!({"textDocument": {"uri": uri}}),
                    ));
                    assert!(server.native_open_buffer_snapshot().is_none());
                    server.handle_message(notification("textDocument/didOpen", json!({"textDocument": {"uri": uri, "version": version, "languageId": "mizar", "text": raw}})));
                }
                "shutdown" => {
                    server.handle_message(request("shutdown", Value::Null));
                }
                "exit" => {
                    server.handle_message(notification("exit", Value::Null));
                }
                _ => unreachable!(),
            }
            assert!(server.native_open_buffer_snapshot().is_none());
            assert_eq!(reader.captured().native_sources().unwrap().len(), 2);
            drop(capture);
        }
        std::fs::remove_dir_all(input().plan_request.workspace_root.as_str()).unwrap();
    }

    #[test]
    fn retained_native_leases_follow_real_ticket_reader_and_driver_ownership() {
        use mizar_session::SnapshotError;
        let mut original = Server::default();
        let ids = ObservingAllocator::default();
        let (template, input, drivers) = native_wire_fixture(
            &mut original,
            &ids.inner,
            &mizar_session::SnapshotRegistry::new(),
        );
        let (draft, index, sources) = originating_native_inputs(&original, &template, input());
        let snapshots =
            std::sync::Arc::new(mizar_session::SnapshotRegistry::with_allocator(ids.clone()));
        let subset = original
            .capture_native_open_buffers(
                draft.clone(),
                &ids,
                &snapshots,
                &index,
                sources[..1].to_vec(),
            )
            .unwrap();
        let other = subset.snapshot.captured().snapshot().id;
        let subset_overlay = ids.leases.lock().unwrap()[1];
        let full = original
            .capture_native_open_buffers(draft.clone(), &ids, &snapshots, &index, sources.clone())
            .unwrap();
        let full_id = full.snapshot.captured().snapshot().id;
        let full_overlay = ids.leases.lock().unwrap()[3];
        assert_ne!(full_id, other);
        // Only the old ticket retains the replaced subset association.
        assert!(
            matches!(snapshots.release_lease(full_id, subset_overlay), Err(SnapshotError::LeaseReleaseMismatch { actual_snapshot, .. }) if actual_snapshot == other)
        );
        drop(subset);
        assert_eq!(
            snapshots.release_lease(full_id, subset_overlay),
            Err(SnapshotError::UnknownSnapshotLease {
                lease_id: subset_overlay
            })
        );
        drop(full);
        // Only the Server retains this association.
        assert!(
            matches!(snapshots.release_lease(other, full_overlay), Err(SnapshotError::LeaseReleaseMismatch { actual_snapshot, .. }) if actual_snapshot == full_id)
        );
        for mode in [
            "abandon",
            "notification",
            "writer",
            "error",
            "writer-error",
            "overlay-error",
            "server-drop",
        ] {
            let mut server = Server::default();
            initialize(&mut server);
            for (uri, (version, text, ..)) in &original.documents {
                server.handle_message(notification("textDocument/didOpen", json!({"textDocument": {"uri": uri, "version": version, "languageId": "mizar", "text": text}})));
            }
            let start = ids.leases.lock().unwrap().len();
            if mode == "overlay-error" {
                ids.fail_at
                    .store(start + 2, std::sync::atomic::Ordering::SeqCst);
            }
            let result = server.capture_native_open_buffers(
                draft.clone(),
                &ids,
                &snapshots,
                &index,
                sources.clone(),
            );
            if mode == "overlay-error" {
                assert!(result.err().unwrap().contains("AllocatorOverflow"));
                let active = ids.leases.lock().unwrap()[start];
                assert_eq!(
                    snapshots.release_lease(other, active),
                    Err(SnapshotError::UnknownSnapshotLease { lease_id: active })
                );
                assert!(server.native_open_buffer_snapshot().is_none());
                ids.fail_at.store(0, std::sync::atomic::Ordering::SeqCst);
                continue;
            }
            let capture = result.unwrap();
            let reader = server.native_open_buffer_snapshot().unwrap();
            let active = capture
                .session
                .as_ref()
                .unwrap()
                .captured()
                .active_snapshot_lease()
                .clone();
            let overlay = ids.leases.lock().unwrap()[start + 1];
            assert_ne!(active.snapshot, other);
            assert_eq!(ids.leases.lock().unwrap()[start], active.lease_id);
            assert_ne!(active.lease_id, overlay);
            let present = |lease| matches!(snapshots.release_lease(other, lease), Err(SnapshotError::LeaseReleaseMismatch { actual_snapshot, .. }) if actual_snapshot == active.snapshot);
            assert!(present(active.lease_id));
            assert!(present(overlay));
            let (mut driver, owner) = drivers();
            owner.register_current_snapshot(active.snapshot);
            match mode {
                "notification" => {
                    let (_, messages) = server
                        .submit_originating_native_frontend_diagnostics(
                            &mut driver,
                            capture,
                            &ids,
                            &snapshots,
                            input(),
                        )
                        .unwrap();
                    assert_eq!(messages.unwrap().len(), 2);
                }
                "writer" => {
                    let mut writer = NativeWireWriter::default();
                    let (_, delivery) = server
                        .submit_originating_native_frontend_diagnostics_to_writer(
                            &mut driver,
                            capture,
                            &ids,
                            &snapshots,
                            input(),
                            &mut writer,
                        )
                        .unwrap();
                    assert_eq!(delivery.unwrap(), 2);
                    assert!(!writer.bytes.is_empty());
                }
                "error" | "writer-error" => {
                    let invalid = mizar_session::SnapshotRegistry::with_allocator(ids.clone());
                    if mode == "writer-error" {
                        let mut writer = NativeWireWriter::default();
                        assert!(
                            server
                                .submit_originating_native_frontend_diagnostics_to_writer(
                                    &mut driver,
                                    capture,
                                    &ids,
                                    &invalid,
                                    input(),
                                    &mut writer,
                                )
                                .is_err()
                        );
                        assert!(writer.calls.is_empty());
                        assert!(writer.bytes.is_empty());
                    } else {
                        assert!(
                            server
                                .submit_originating_native_frontend_diagnostics(
                                    &mut driver,
                                    capture,
                                    &ids,
                                    &invalid,
                                    input()
                                )
                                .is_err()
                        );
                    }
                }
                _ => drop(capture),
            }
            if matches!(mode, "notification" | "writer" | "error" | "writer-error") {
                assert!(present(active.lease_id));
            } else {
                assert_eq!(
                    snapshots.release_lease(other, active.lease_id),
                    Err(SnapshotError::UnknownSnapshotLease {
                        lease_id: active.lease_id
                    })
                );
            }
            assert!(present(overlay));
            if mode == "server-drop" {
                drop(server);
            } else {
                server.handle_message(notification("textDocument/didChange", json!({"textDocument": {"uri": sources[0].2, "version": 8}, "contentChanges": []})));
                assert!(server.native_open_buffer_snapshot().is_none());
            }
            assert!(present(overlay));
            assert_eq!(reader.captured().native_sources().unwrap().len(), 2);
            drop(reader);
            assert_eq!(
                snapshots.release_lease(other, overlay),
                Err(SnapshotError::UnknownSnapshotLease { lease_id: overlay })
            );
        }
        let capture = original
            .capture_native_open_buffers(draft, &ids, &snapshots, &index, sources)
            .unwrap();
        let reader = original.native_open_buffer_snapshot().unwrap();
        let registry_lifetime = std::sync::Arc::downgrade(&snapshots);
        drop(capture);
        drop(original);
        drop(snapshots);
        assert!(registry_lifetime.upgrade().is_some());
        assert_eq!(reader.captured().native_sources().unwrap().len(), 2);
        drop(reader);
        assert!(registry_lifetime.upgrade().is_none());
        std::fs::remove_dir_all(input().plan_request.workspace_root.as_str()).unwrap();
    }

    #[test]
    fn originating_native_capture_uses_current_raw_origins_and_original_producers() {
        let mut server = Server::default();
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        let snapshots = std::sync::Arc::new(mizar_session::SnapshotRegistry::new());
        let (template, input, drivers) = native_wire_fixture(&mut server, &ids, &snapshots);
        let (draft, index, sources) = originating_native_inputs(&server, &template, input());
        let uris = sources
            .iter()
            .map(|(_, _, uri)| uri.clone())
            .collect::<Vec<_>>();
        for wire in [false, true] {
            let capture = server
                .capture_native_open_buffers(
                    draft.clone(),
                    &ids,
                    &snapshots,
                    &index,
                    sources.clone(),
                )
                .unwrap();
            let session = capture.session.as_ref().unwrap().clone();
            assert_ne!(session.id(), template.id());
            for (name, raw) in [
                ("clean", "\u{feff}definition\r\nend;\r\n:: 🦀\r\n"),
                ("main", "\u{feff}::=🦀=::end\r\n:: suffix\r"),
            ] {
                let (_, loaded, descriptor) = session
                    .captured()
                    .native_sources()
                    .unwrap()
                    .iter()
                    .find(|(_, _, descriptor)| descriptor.module_path.as_str() == name)
                    .unwrap();
                let expected_uri = format!("{}/{name}.miz", uris[0].rsplit_once('/').unwrap().0);
                assert_eq!(
                    descriptor.origin,
                    mizar_session::SourceOriginInput::OpenBuffer {
                        uri: expected_uri.clone(),
                        expected_version: 7,
                        actual_version: 7,
                        text: raw.into(),
                    }
                );
                assert_eq!(
                    loaded.text.as_ref(),
                    raw.trim_start_matches('\u{feff}').replace("\r\n", "\n")
                );
                let (_, original) = capture
                    .snapshot
                    .lifetimes
                    .iter()
                    .find(|(uri, _)| uri == &expected_uri)
                    .unwrap();
                assert!(std::sync::Weak::ptr_eq(
                    original,
                    &server.document_lifetime(&expected_uri).unwrap()
                ));
            }
            let (mut ordinary_driver, owner) = drivers();
            owner.register_current_snapshot(session.captured().snapshot().id);
            let ordinary = ordinary_driver
                .submit_captured(
                    session.clone(),
                    &ids,
                    &snapshots,
                    input(),
                    Default::default(),
                )
                .unwrap();
            let diagnostics = vec![
                json!({"range": {"start": {"line": 0, "character": 9}, "end": {"line": 0, "character": 12}},
                    "code": "E0033", "message": "unmatched `end` in lexical scope skeleton", "severity": 1}),
                json!({"range": {"start": {"line": 0, "character": 9}, "end": {"line": 0, "character": 12}},
                    "code": "E0052", "message": "`end` has no matching block opener", "severity": 1}),
            ];
            let expected = vec![
                json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                    "params": {"uri": uris[0], "version": 7, "diagnostics": []}}),
                json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                    "params": {"uri": uris[1], "version": 7, "diagnostics": diagnostics}}),
            ];
            let (mut driver, owner) = drivers();
            owner.register_current_snapshot(session.captured().snapshot().id);
            if wire {
                let mut writer = NativeWireWriter::default();
                let (submission, delivery) = server
                    .submit_originating_native_frontend_diagnostics_to_writer(
                        &mut driver,
                        capture,
                        &ids,
                        &snapshots,
                        input(),
                        &mut writer,
                    )
                    .unwrap();
                assert_eq!(submission, ordinary);
                assert_eq!(delivery.unwrap(), 2);
                let mut bytes = Vec::new();
                for message in expected {
                    let body = serde_json::to_vec(&message).unwrap();
                    write!(bytes, "Content-Length: {}\r\n\r\n", body.len()).unwrap();
                    bytes.extend(body);
                }
                assert_eq!(writer.bytes, bytes);
                assert_eq!(writer.calls.iter().filter(|&&op| op == "flush").count(), 2);
            } else {
                let (submission, notifications) = server
                    .submit_originating_native_frontend_diagnostics(
                        &mut driver,
                        capture,
                        &ids,
                        &snapshots,
                        input(),
                    )
                    .unwrap();
                assert_eq!(submission, ordinary);
                assert_eq!(notifications, Some(expected));
            }
        }
        std::fs::remove_dir_all(input().plan_request.workspace_root.as_str()).unwrap();
    }

    #[test]
    fn originating_native_capture_discriminates_incarnations_and_preserves_raw_guards() {
        for mode in [
            "same",
            "reopen",
            "reopen-first",
            "other",
            "version",
            "raw",
            "closed",
            "shutdown",
            "exit",
        ] {
            for wire in [false, true] {
                let mut server = Server::default();
                let ids = mizar_session::InMemorySessionIdAllocator::new();
                let snapshots = std::sync::Arc::new(mizar_session::SnapshotRegistry::new());
                let (template, input, drivers) = native_wire_fixture(&mut server, &ids, &snapshots);
                let (draft, index, sources) =
                    originating_native_inputs(&server, &template, input());
                let capture = server
                    .capture_native_open_buffers(
                        draft.clone(),
                        &ids,
                        &snapshots,
                        &index,
                        sources.clone(),
                    )
                    .unwrap();
                let session = capture.session.as_ref().unwrap().clone();
                let uri = sources[if mode == "reopen-first" { 0 } else { 1 }]
                    .2
                    .clone();
                let (version, raw) = server.document(&uri).unwrap();
                let (version, raw) = (version, raw.to_owned());
                let retained = server.documents.get(&uri).unwrap().2.clone();
                let original = server.document_lifetime(&uri).unwrap();
                let mut other = Server::default();
                if mode == "other" {
                    initialize(&mut other);
                    for (uri, (version, text, ..)) in &server.documents {
                        other.handle_message(notification(
                            "textDocument/didOpen",
                            json!({"textDocument": {
                            "uri": uri, "languageId": "mizar", "version": version, "text": text}}),
                        ));
                    }
                } else if matches!(mode, "reopen" | "reopen-first" | "raw" | "closed") {
                    server.handle_message(notification(
                        "textDocument/didClose",
                        json!({"textDocument": {"uri": uri}}),
                    ));
                    if mode != "closed" {
                        let text = if mode == "raw" {
                            raw.trim_start_matches('\u{feff}').replace("\r\n", "\n")
                        } else {
                            raw.clone()
                        };
                        server.handle_message(notification(
                            "textDocument/didOpen",
                            json!({"textDocument": {
                            "uri": uri, "languageId": "mizar", "version": version, "text": text}}),
                        ));
                    }
                } else if mode == "version" {
                    server.handle_message(notification("textDocument/didChange", json!({
                        "textDocument": {"uri": uri, "version": 8}, "contentChanges": [{"text": raw}]})));
                } else if mode == "shutdown" {
                    server.handle_message(request("shutdown", Value::Null));
                } else if mode == "exit" {
                    server.handle_message(notification("exit", Value::Null));
                }
                let current = if mode == "other" { &other } else { &server };
                if mode == "reopen-first" {
                    assert!(uri.ends_with("/clean.miz"));
                    let (unchanged_uri, unchanged) = &capture.snapshot.lifetimes[1];
                    assert!(std::sync::Weak::ptr_eq(
                        unchanged,
                        &current.document_lifetime(unchanged_uri).unwrap(),
                    ));
                }
                assert!(original.upgrade().is_some());
                assert!(std::sync::Arc::ptr_eq(
                    &retained,
                    &original.upgrade().unwrap()
                ));
                if mode == "version" {
                    assert!(std::sync::Weak::ptr_eq(
                        &original,
                        &current.document_lifetime(&uri).unwrap()
                    ));
                }
                let (mut ordinary_driver, owner) = drivers();
                owner.register_current_snapshot(session.captured().snapshot().id);
                let ordinary = ordinary_driver
                    .submit_captured(
                        session.clone(),
                        &ids,
                        &snapshots,
                        input(),
                        Default::default(),
                    )
                    .unwrap();
                // Equal raw tuples remain viable through the old API, even on another Server.
                let (mut old_driver, owner) = drivers();
                owner.register_current_snapshot(session.captured().snapshot().id);
                let (old_submission, messages) = current
                    .submit_native_frontend_diagnostics(
                        &mut old_driver,
                        session.clone(),
                        &ids,
                        &snapshots,
                        input(),
                    )
                    .unwrap();
                assert_eq!(old_submission, ordinary);
                assert_eq!(
                    messages.is_some(),
                    matches!(mode, "same" | "reopen" | "reopen-first" | "other")
                );
                let (mut driver, owner) = drivers();
                owner.register_current_snapshot(session.captured().snapshot().id);
                if wire {
                    let mut writer = NativeWireWriter::default();
                    let (submission, delivery) = current
                        .submit_originating_native_frontend_diagnostics_to_writer(
                            &mut driver,
                            capture,
                            &ids,
                            &snapshots,
                            input(),
                            &mut writer,
                        )
                        .unwrap();
                    assert_eq!(submission, ordinary);
                    assert_eq!(delivery.unwrap(), if mode == "same" { 2 } else { 0 });
                    if mode != "same" {
                        assert!(writer.bytes.is_empty());
                        assert!(writer.calls.is_empty());
                    }
                } else {
                    let (submission, messages) = current
                        .submit_originating_native_frontend_diagnostics(
                            &mut driver,
                            capture,
                            &ids,
                            &snapshots,
                            input(),
                        )
                        .unwrap();
                    assert_eq!(submission, ordinary);
                    assert_eq!(messages.is_some(), mode == "same");
                }
                if matches!(mode, "reopen" | "version") {
                    let fresh = server
                        .capture_native_open_buffers(draft, &ids, &snapshots, &index, sources)
                        .unwrap();
                    let (mut driver, owner) = drivers();
                    owner.register_current_snapshot(
                        fresh.session.as_ref().unwrap().captured().snapshot().id,
                    );
                    let (_, messages) = server
                        .submit_originating_native_frontend_diagnostics(
                            &mut driver,
                            fresh,
                            &ids,
                            &snapshots,
                            input(),
                        )
                        .unwrap();
                    assert_eq!(messages.unwrap().len(), 2);
                }
                std::fs::remove_dir_all(input().plan_request.workspace_root.as_str()).unwrap();
            }
        }
    }

    #[test]
    fn originating_native_capture_rejects_invalid_acquisition_and_keeps_capture_errors() {
        use mizar_session::SessionIdAllocator;
        let mut server = Server::default();
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        let snapshots = std::sync::Arc::new(mizar_session::SnapshotRegistry::new());
        let (template, input, _) = native_wire_fixture(&mut server, &ids, &snapshots);
        let (draft, index, sources) = originating_native_inputs(&server, &template, input());
        for mode in [
            "uninitialized",
            "empty",
            "duplicate",
            "absent",
            "closed",
            "negative",
            "missing-index",
            "wrong-index",
            "wrong-later",
            "shutdown",
            "exit",
        ] {
            let mut current = Server::default();
            if mode != "uninitialized" {
                initialize(&mut current);
            }
            for (uri, (version, text, ..)) in &server.documents {
                current.handle_message(notification("textDocument/didOpen", json!({"textDocument": {
                    "uri": uri, "languageId": "mizar", "version": if mode == "negative" { -1 } else { *version }, "text": text}})));
            }
            let mut requested = sources.clone();
            let mut selected = index.clone();
            match mode {
                "empty" => requested.clear(),
                "duplicate" => requested.push(requested[0].clone()),
                "absent" => requested[0].2.push_str(".absent"),
                "closed" => {
                    current.handle_message(notification(
                        "textDocument/didClose",
                        json!({"textDocument": {"uri": requested[0].2}}),
                    ));
                }
                "missing-index" => selected.modules.clear(),
                "wrong-index" => requested[0].1 = mizar_session::PackageId::new("other"),
                "wrong-later" => requested[1].1 = mizar_session::PackageId::new("other"),
                "shutdown" => {
                    current.handle_message(request("shutdown", Value::Null));
                }
                "exit" => {
                    current.handle_message(notification("exit", Value::Null));
                }
                _ => {}
            }
            let acquisition_ids = mizar_session::InMemorySessionIdAllocator::new();
            let untouched_ids = mizar_session::InMemorySessionIdAllocator::new();
            let error = current
                .capture_native_open_buffers(
                    draft.clone(),
                    &acquisition_ids,
                    &snapshots,
                    &selected,
                    requested.clone(),
                )
                .err()
                .unwrap();
            assert_eq!(
                acquisition_ids.next_session_id(),
                untouched_ids.next_session_id()
            );
            assert_eq!(
                acquisition_ids.next_request_id(),
                untouched_ids.next_request_id()
            );
            if matches!(mode, "missing-index" | "wrong-index" | "wrong-later") {
                let (loader, package, uri) = &requested[if mode == "wrong-later" { 1 } else { 0 }];
                let (version, text) = current.document(uri).unwrap();
                let expected = selected
                    .native_open_buffer_input(
                        &draft.workspace_root,
                        package,
                        loader,
                        mizar_session::SourceOriginInput::OpenBuffer {
                            uri: uri.clone(),
                            expected_version: i64::from(version),
                            actual_version: i64::from(version),
                            text: text.into(),
                        },
                    )
                    .unwrap_err();
                assert_eq!(error, expected);
            }
            if mode == "negative" {
                assert_eq!(current.document(&sources[0].2).unwrap().0, -1);
            }
        }
        let mut invalid = draft.clone();
        invalid.source_inputs.versions = template.request().source_inputs.versions.clone();
        let expected = mizar_driver::driver::DriverSubmitError::NativeSourceVersionsProvided {
            request: Box::new(invalid.clone()),
        };
        let error = server
            .capture_native_open_buffers(invalid, &ids, &snapshots, &index, sources)
            .err()
            .unwrap();
        assert_eq!(error, format!("{expected:?}"));
        std::fs::remove_dir_all(input().plan_request.workspace_root.as_str()).unwrap();
    }

    #[test]
    fn originating_native_capture_preserves_driver_guards_errors_and_writer_failures() {
        let mut server = Server::default();
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        let snapshots = std::sync::Arc::new(mizar_session::SnapshotRegistry::new());
        let (template, input, drivers) = native_wire_fixture(&mut server, &ids, &snapshots);
        let (draft, index, sources) = originating_native_inputs(&server, &template, input());
        for mode in [
            "invalid-registry",
            "ineligible-error",
            "denied",
            "obsolete",
            "flush",
            "obsolete-on-flush",
        ] {
            let capture = server
                .capture_native_open_buffers(
                    draft.clone(),
                    &ids,
                    &snapshots,
                    &index,
                    sources.clone(),
                )
                .unwrap();
            let session = capture.session.as_ref().unwrap().clone();
            let snapshot = session.captured().snapshot().id;
            let invalid = mizar_session::SnapshotRegistry::new();
            let registry = if matches!(mode, "invalid-registry" | "ineligible-error") {
                &invalid
            } else {
                &snapshots
            };
            let make_driver = || {
                let (driver, owner) = if mode == "denied" {
                    // Real services with a publisher lacking work authorization deny SourceLoad.
                    let owner =
                        std::sync::Arc::new(mizar_ir::publisher::PhaseOutputPublisher::new(
                            std::sync::Arc::new(mizar_ir::storage::IrStorageService::new()),
                            std::sync::Arc::new(mizar_ir::identity::SnapshotHandleRegistry::new()),
                        ));
                    let mut builder = mizar_driver::registry::PhaseRegistryBuilder::new();
                    builder.register_source_load();
                    builder.register_frontend(vec![]);
                    (
                        mizar_driver::driver::CompilerDriver::new(builder.build().unwrap())
                            .with_output_publisher(owner.clone()),
                        owner,
                    )
                } else {
                    drivers()
                };
                owner.register_current_snapshot(snapshot);
                if mode == "obsolete" {
                    owner.mark_obsolete(snapshot).unwrap();
                }
                (driver, owner)
            };
            let (mut ordinary_driver, _) = make_driver();
            let ordinary = ordinary_driver.submit_captured(
                session.clone(),
                &ids,
                registry,
                input(),
                Default::default(),
            );
            let (mut driver, owner) = make_driver();
            let mut writer = NativeWireWriter {
                fail: (mode == "flush").then_some("flush"),
                obsolete_on_flush: (mode == "obsolete-on-flush").then_some((owner, snapshot)),
                ..NativeWireWriter::default()
            };
            let other = Server::default();
            let current = if mode == "ineligible-error" {
                &other
            } else {
                &server
            };
            let result = current.submit_originating_native_frontend_diagnostics_to_writer(
                &mut driver,
                capture,
                &ids,
                registry,
                input(),
                &mut writer,
            );
            if matches!(mode, "invalid-registry" | "ineligible-error") {
                assert_eq!(result.unwrap_err(), ordinary.unwrap_err());
                assert!(writer.calls.is_empty());
            } else {
                let (submission, delivery) = result.unwrap();
                assert_eq!(submission, ordinary.unwrap());
                match mode {
                    "flush" => {
                        let error = delivery.unwrap_err();
                        assert_eq!(error.kind(), io::ErrorKind::Other);
                        assert_eq!(error.to_string(), "flush");
                        assert_eq!(writer.calls.last(), Some(&"flush"));
                    }
                    "obsolete-on-flush" => {
                        assert_eq!(delivery.unwrap(), 1);
                        assert_eq!(writer.calls.iter().filter(|&&op| op == "flush").count(), 1);
                        assert!(
                            driver
                                .validate_current_native_diagnostics(session.id(), registry)
                                .is_err()
                        );
                    }
                    _ => {
                        assert_eq!(delivery.unwrap(), 0);
                        assert!(writer.calls.is_empty());
                        assert!(writer.bytes.is_empty());
                    }
                }
            }
        }
        std::fs::remove_dir_all(input().plan_request.workspace_root.as_str()).unwrap();
    }

    #[test]
    fn native_initialize_inputs_reach_originating_frontend_wire() {
        use mizar_build::{
            module_index::StaticSourceLayout,
            planner::{lockfile_content_hash, produce_build_plan, workspace_config_content_hash},
        };
        use std::fs;

        let ids = mizar_session::InMemorySessionIdAllocator::new();
        let snapshots = std::sync::Arc::new(mizar_session::SnapshotRegistry::new());
        // Reuse only fixture allocation, explicit request context and phase drivers.
        let (template, fixture_input, drivers) =
            native_wire_fixture(&mut Server::default(), &ids, &snapshots);
        let root = std::path::PathBuf::from(fixture_input().plan_request.workspace_root.as_str());
        fs::write(
            root.join("mizar.workspace"),
            "[workspace]\nmembers = [\"alpha\"]\n",
        )
        .unwrap();
        fs::write(
            root.join("alpha/mizar.pkg"),
            "[package]\nname = \"alpha\"\nversion = \"1.2.3\"\n",
        )
        .unwrap();
        fs::write(root.join("mizar.lock"), "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"1.2.3\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n").unwrap();
        // Native association needs files; successful Frontend work must use editor bytes.
        for name in ["main", "clean"] {
            fs::write(root.join(format!("alpha/src/{name}.miz")), [0xff]).unwrap();
        }
        let root_uri = format!(
            "file://{}{}",
            if cfg!(windows) { "/" } else { "" },
            root.to_str().unwrap().replace('\\', "/")
        );
        let mut server = Server::default();
        let params = json!({"processId": null, "rootUri": root_uri, "capabilities": {}});
        let response = server
            .handle_message(request("initialize", params.clone()))
            .unwrap();
        assert!(response["result"]["capabilities"].is_object());
        assert_eq!(server.initialization_params(), Some(&params));
        let (workspace, packages, lock) = server.native_workspace_inputs().unwrap().unwrap();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].member_path, "alpha");
        assert_eq!(packages[0].manifest.name, "alpha");
        assert_eq!(packages[0].manifest.version.to_string(), "1.2.3");
        assert_eq!(lock.packages.len(), 1);
        assert_eq!(lock.packages[0].version.to_string(), "1.2.3");
        // ToolchainInfo and remaining runtime operands are still explicit fixture context.
        let mut plan_request = fixture_input().plan_request;
        plan_request.workspace_root = workspace.clone();
        let plan =
            produce_build_plan(plan_request.clone(), packages.clone(), lock.clone()).unwrap();
        assert_eq!(plan.packages.len(), 1);
        assert_eq!(plan.packages[0].package_id.as_str(), "alpha");
        assert_eq!(plan.packages[0].version.to_string(), "1.2.3");
        let layout = StaticSourceLayout::from_native_disk(&plan).unwrap();
        let input = || {
            let mut input = fixture_input();
            input.plan_request = plan_request.clone();
            input.workspace_packages = packages.clone();
            input.lockfile = lock.clone();
            input.source_layout = layout.clone();
            input
        };
        for (name, raw) in [
            ("main", "\u{feff}::=🦀=::end\r\n:: suffix\r"),
            ("clean", "\u{feff}definition\r\nend;\r\n:: 🦀\r\n"),
        ] {
            let uri = format!("{root_uri}/alpha/src/{name}.miz");
            assert!(server
                .handle_message(notification(
                    "textDocument/didOpen",
                    json!({"textDocument": {"uri": uri, "languageId": "mizar", "version": 7, "text": raw}}),
                ))
                .is_none());
            assert_eq!(server.document(&uri), Some((7, raw)));
        }
        let (mut draft, index, sources) = originating_native_inputs(&server, &template, input());
        draft.workspace_root = workspace;
        draft.dependency_inputs.lockfile_hash = lockfile_content_hash(&lock);
        draft.verifier_config.hash =
            workspace_config_content_hash(&plan.verifier_config, &plan.build_config);
        let uris = sources
            .iter()
            .map(|(_, _, uri)| uri.clone())
            .collect::<Vec<_>>();
        let capture = server
            .capture_native_open_buffers(draft, &ids, &snapshots, &index, sources)
            .unwrap();
        let session = capture.session.as_ref().unwrap().clone();
        let (mut ordinary_driver, owner) = drivers();
        owner.register_current_snapshot(session.captured().snapshot().id);
        let ordinary = ordinary_driver
            .submit_captured(
                session.clone(),
                &ids,
                &snapshots,
                input(),
                Default::default(),
            )
            .unwrap();
        assert_eq!(ordinary.build_plan.as_ref(), Some(&plan));
        assert_eq!(ordinary.module_index.as_ref(), Some(&index));
        let (mut driver, owner) = drivers();
        owner.register_current_snapshot(session.captured().snapshot().id);
        let mut writer = NativeWireWriter::default();
        let (submission, delivery) = server
            .submit_originating_native_frontend_diagnostics_to_writer(
                &mut driver,
                capture,
                &ids,
                &snapshots,
                input(),
                &mut writer,
            )
            .unwrap();
        assert_eq!(submission, ordinary);
        assert_eq!(delivery.unwrap(), 2);
        let expected = [
            json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                "params": {"uri": uris[0], "version": 7, "diagnostics": []}}),
            json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
            "params": {"uri": uris[1], "version": 7, "diagnostics": [
                {"range": {"start": {"line": 0, "character": 9}, "end": {"line": 0, "character": 12}},
                    "code": "E0033", "message": "unmatched `end` in lexical scope skeleton", "severity": 1},
                {"range": {"start": {"line": 0, "character": 9}, "end": {"line": 0, "character": 12}},
                    "code": "E0052", "message": "`end` has no matching block opener", "severity": 1}
            ]}}),
        ];
        let mut bytes = Vec::new();
        for message in expected {
            let body = serde_json::to_vec(&message).unwrap();
            assert!(body.len() > std::str::from_utf8(&body).unwrap().chars().count());
            write!(bytes, "Content-Length: {}\r\n\r\n", body.len()).unwrap();
            bytes.extend(body);
        }
        assert_eq!(writer.bytes, bytes);
        assert_eq!(writer.calls.iter().filter(|&&op| op == "flush").count(), 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn discovery_notifications_preserve_captured_imports_original_records_and_wire() {
        use mizar_build::task_graph::{
            DependencyCoverage, ModuleDependencyOverlay, TaskKind, WorkUnit,
        };
        let mut server = Server::default();
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        let snapshots = std::sync::Arc::new(mizar_session::SnapshotRegistry::new());
        let (template, input, drivers) = native_wire_fixture(&mut server, &ids, &snapshots);
        let (draft, index, sources) = originating_native_inputs(&server, &template, input());
        let uris = sources
            .iter()
            .map(|(_, _, uri)| uri.clone())
            .collect::<Vec<_>>();
        for (uri, raw) in [
            (
                &uris[1],
                "\u{feff}import alpha.clean;\r\ntheorem T: a combine b = a;\r\n::=🦀=::end\r\n",
            ),
            (
                &uris[0],
                "\u{feff}definition\r\nlet x, y be set;\r\npublic func Infix: x combine y -> set equals x;\r\nend;\r\n",
            ),
        ] {
            server.handle_message(notification(
                "textDocument/didChange",
                json!({
                "textDocument": {"uri": uri, "version": 8}, "contentChanges": [{"text": raw}]}),
            ));
        }
        let discovered_input = || {
            let mut value = input();
            value.dependency_overlay = ModuleDependencyOverlay::unavailable();
            value
        };
        // The non-BMP comment prefix is eight UTF-16 units; the BOM is on line zero.
        let expected = vec![
            json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": {"uri": uris[0], "version": 8, "diagnostics": []}}),
            json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": {"uri": uris[1], "version": 8, "diagnostics": [
                {"range": {"start": {"line": 2, "character": 8}, "end": {"line": 2, "character": 11}}, "code": "E0033", "message": "unmatched `end` in lexical scope skeleton", "severity": 1},
                {"range": {"start": {"line": 2, "character": 8}, "end": {"line": 2, "character": 11}}, "code": "E0052", "message": "`end` has no matching block opener", "severity": 1}
            ]}}),
        ];
        let mut expected_bytes = Vec::new();
        for message in &expected {
            let body = serde_json::to_vec(message).unwrap();
            write!(expected_bytes, "Content-Length: {}\r\n\r\n", body.len()).unwrap();
            expected_bytes.extend(body);
        }
        for route in ["raw", "originating", "wire"] {
            for name in ["main", "clean"] {
                std::fs::write(
                    std::path::Path::new(input().plan_request.workspace_root.as_str())
                        .join(format!("alpha/src/{name}.miz")),
                    [0xff],
                )
                .unwrap();
            }
            let capture = server
                .capture_native_open_buffers(
                    draft.clone(),
                    &ids,
                    &snapshots,
                    &index,
                    sources.clone(),
                )
                .unwrap();
            for name in ["main", "clean"] {
                std::fs::remove_file(
                    std::path::Path::new(input().plan_request.workspace_root.as_str())
                        .join(format!("alpha/src/{name}.miz")),
                )
                .unwrap();
            }

            let session = capture.session.as_ref().unwrap().clone();
            let snapshot = session.captured().snapshot().id;
            let (mut reference_driver, reference_owner) = drivers();
            reference_owner.register_current_snapshot(snapshot);
            let (expected_submission, index) = reference_driver
                .submit_captured(
                    session.clone(),
                    &ids,
                    &snapshots,
                    discovered_input(),
                    mizar_driver::driver::SubmitOptions {
                        discover_imports: true,
                        diagnostic_index: Some(mizar_driver::driver::DiagnosticIndexMode::Native),
                        ..Default::default()
                    },
                )
                .map(|mut submission| {
                    let index = submission.diagnostic_index.take();
                    (submission, index)
                })
                .unwrap();
            let graph = expected_submission.task_graph.as_ref().unwrap();
            let frontend = |name| {
                graph.tasks().iter().find(|task| task.kind == TaskKind::Frontend && matches!(&task.unit, WorkUnit::Module { module } if module.path.as_str() == name)).unwrap()
            };
            assert!(
                frontend("main")
                    .dependencies
                    .contains(&frontend("clean").id)
            );
            assert!(
                graph
                    .tasks()
                    .iter()
                    .filter(|task| task.kind == TaskKind::ModuleResolve)
                    .all(|task| task.dependency_coverage
                        == DependencyCoverage::MissingModuleDependencyOverlay)
            );
            let index = index.unwrap();
            assert_eq!(index.records().len(), 2);
            for (record, value) in index
                .records()
                .iter()
                .zip(expected[1]["params"]["diagnostics"].as_array().unwrap())
            {
                assert_eq!(value["code"], record.code().to_string());
                assert_eq!(value["message"], record.message());
            }
            let (mut driver, owner) = drivers();
            owner.register_current_snapshot(snapshot);
            let actual = if route == "wire" {
                let mut writer = NativeWireWriter::default();
                let (submission, delivered) = server
                    .submit_originating_native_frontend_diagnostics_to_writer(
                        &mut driver,
                        capture,
                        &ids,
                        &snapshots,
                        discovered_input(),
                        &mut writer,
                    )
                    .unwrap();
                assert_eq!(delivered.unwrap(), 2);
                assert_eq!(writer.bytes, expected_bytes);
                assert_eq!(writer.calls.iter().filter(|&&op| op == "flush").count(), 2);
                submission
            } else {
                let (submission, notifications) = if route == "raw" {
                    server.submit_native_frontend_diagnostics(
                        &mut driver,
                        session,
                        &ids,
                        &snapshots,
                        discovered_input(),
                    )
                } else {
                    server.submit_originating_native_frontend_diagnostics(
                        &mut driver,
                        capture,
                        &ids,
                        &snapshots,
                        discovered_input(),
                    )
                }
                .unwrap();
                assert_eq!(notifications, Some(expected.clone()));
                submission
            };
            assert_eq!(actual, expected_submission);
        }
        std::fs::remove_dir_all(input().plan_request.workspace_root.as_str()).unwrap();
    }

    #[test]
    fn discovery_notifications_preserve_supplied_overlays_and_whole_failure() {
        use mizar_build::task_graph::{
            DependencyCoverage, ModuleDependencyCoverage, ModuleDependencyOverlay, TaskKind,
            WorkUnit,
        };
        let mut server = Server::default();
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        let snapshots = std::sync::Arc::new(mizar_session::SnapshotRegistry::new());
        let (template, input, drivers) = native_wire_fixture(&mut server, &ids, &snapshots);
        let (draft, index, sources) = originating_native_inputs(&server, &template, input());
        let main_uri = sources
            .iter()
            .find(|(_, _, uri)| uri.ends_with("/main.miz"))
            .unwrap()
            .2
            .clone();
        server.handle_message(notification("textDocument/didChange", json!({"textDocument": {"uri": main_uri, "version": 8}, "contentChanges": [{"text": "import alpha.clean;\ndefinition\nend;\n"}]})));
        for mode in ["missing-capture", "imports-only"] {
            let captured_sources = sources
                .iter()
                .filter(|(_, _, uri)| mode != "missing-capture" || uri.ends_with("/main.miz"))
                .cloned()
                .collect();
            let capture = server
                .capture_native_open_buffers(
                    draft.clone(),
                    &ids,
                    &snapshots,
                    &index,
                    captured_sources,
                )
                .unwrap();
            let session = capture.session.as_ref().unwrap().clone();
            let submit_input = || {
                let mut value = input();
                value.dependency_overlay = match mode {
                    "imports-only" => ModuleDependencyOverlay {
                        coverage: ModuleDependencyCoverage::ImportsOnly,
                        edges: vec![],
                    },
                    _ => ModuleDependencyOverlay::unavailable(),
                };
                value
            };
            let (mut reference, owner) = drivers();
            owner.register_current_snapshot(session.captured().snapshot().id);
            let expected = reference
                .submit_captured(
                    session.clone(),
                    &ids,
                    &snapshots,
                    submit_input(),
                    mizar_driver::driver::SubmitOptions {
                        discover_imports: true,
                        diagnostic_index: Some(mizar_driver::driver::DiagnosticIndexMode::Native),
                        ..Default::default()
                    },
                )
                .map(|mut submission| {
                    let index = submission.diagnostic_index.take();
                    (submission, index)
                });
            let (mut driver, owner) = drivers();
            owner.register_current_snapshot(session.captured().snapshot().id);
            let actual = server.submit_native_frontend_diagnostics(
                &mut driver,
                session,
                &ids,
                &snapshots,
                submit_input(),
            );
            match (actual, expected) {
                (Err(actual), Err(expected)) => {
                    assert_ne!(mode, "imports-only");
                    assert_eq!(actual, expected);
                    let mizar_driver::driver::DriverSubmitError::TaskGraph { diagnostics, .. } =
                        actual
                    else {
                        panic!("whole discovery/overlay failure must fail before scheduling")
                    };
                    assert!(diagnostics.diagnostics().iter().any(|diagnostic| diagnostic.kind == mizar_build::task_graph::TaskGraphDiagnosticKind::MissingModuleDependencyOverlay));
                }
                (Ok((actual, _)), Ok((expected, _))) => {
                    assert_eq!(mode, "imports-only");
                    assert_eq!(actual, expected);
                    let graph = actual.task_graph.as_ref().unwrap();
                    let frontend = |name| {
                        graph.tasks().iter().find(|task| task.kind == TaskKind::Frontend && matches!(&task.unit, WorkUnit::Module { module } if module.path.as_str() == name)).unwrap()
                    };
                    assert!(
                        !frontend("main")
                            .dependencies
                            .contains(&frontend("clean").id)
                    );
                    assert!(
                        graph
                            .tasks()
                            .iter()
                            .filter(|task| task.kind == TaskKind::ModuleResolve)
                            .all(|task| task.dependency_coverage
                                == DependencyCoverage::MissingModuleDependencyOverlay)
                    );
                }
                other => panic!("discovery reference mismatch: {other:?}"),
            }
        }
        std::fs::remove_dir_all(input().plan_request.workspace_root.as_str()).unwrap();
    }

    #[test]
    fn discovery_originating_lifetimes_and_wire_guards_preserve_real_submissions() {
        use mizar_build::task_graph::ModuleDependencyOverlay;
        let mut server = Server::default();
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        let snapshots = std::sync::Arc::new(mizar_session::SnapshotRegistry::new());
        let (template, input, drivers) = native_wire_fixture(&mut server, &ids, &snapshots);
        let (draft, index, sources) = originating_native_inputs(&server, &template, input());
        let main_uri = sources
            .iter()
            .find(|(_, _, uri)| uri.ends_with("/main.miz"))
            .unwrap()
            .2
            .clone();
        let raw = "\u{feff}import alpha.clean;\r\n::=🦀=::end\r\n";
        server.handle_message(notification("textDocument/didChange", json!({"textDocument": {"uri": main_uri, "version": 8}, "contentChanges": [{"text": raw}]})));
        let submit_input = || {
            let mut value = input();
            value.dependency_overlay = ModuleDependencyOverlay::unavailable();
            value
        };
        for mode in [
            "raw-reopened",
            "originating-reopened",
            "wire-reopened",
            "obsolete-on-flush",
        ] {
            let capture = server
                .capture_native_open_buffers(
                    draft.clone(),
                    &ids,
                    &snapshots,
                    &index,
                    sources.clone(),
                )
                .unwrap();
            let session = capture.session.as_ref().unwrap().clone();
            let snapshot = session.captured().snapshot().id;
            if mode.ends_with("reopened") {
                server.handle_message(notification(
                    "textDocument/didClose",
                    json!({"textDocument": {"uri": main_uri}}),
                ));
                server.handle_message(notification("textDocument/didOpen", json!({"textDocument": {"uri": main_uri, "languageId": "mizar", "version": 8, "text": raw}})));
                assert_eq!(server.document(&main_uri), Some((8, raw)));
            }
            let (mut reference, owner) = drivers();
            owner.register_current_snapshot(snapshot);
            let expected = reference
                .submit_captured(
                    session.clone(),
                    &ids,
                    &snapshots,
                    submit_input(),
                    mizar_driver::driver::SubmitOptions {
                        discover_imports: true,
                        diagnostic_index: Some(mizar_driver::driver::DiagnosticIndexMode::Native),
                        ..Default::default()
                    },
                )
                .map(|mut submission| {
                    let index = submission.diagnostic_index.take();
                    (submission, index)
                });
            let (mut driver, owner) = drivers();
            owner.register_current_snapshot(snapshot);
            if matches!(mode, "raw-reopened" | "originating-reopened") {
                let (actual, notifications) = if mode == "raw-reopened" {
                    server.submit_native_frontend_diagnostics(
                        &mut driver,
                        session,
                        &ids,
                        &snapshots,
                        submit_input(),
                    )
                } else {
                    server.submit_originating_native_frontend_diagnostics(
                        &mut driver,
                        capture,
                        &ids,
                        &snapshots,
                        submit_input(),
                    )
                }
                .unwrap();
                assert_eq!(actual, expected.unwrap().0);
                if mode == "raw-reopened" {
                    assert_eq!(notifications.unwrap().len(), 2);
                } else {
                    assert!(notifications.is_none());
                }
                continue;
            }
            let mut writer = NativeWireWriter {
                obsolete_on_flush: (mode == "obsolete-on-flush").then_some((owner, snapshot)),
                ..NativeWireWriter::default()
            };
            let result = server.submit_originating_native_frontend_diagnostics_to_writer(
                &mut driver,
                capture,
                &ids,
                &snapshots,
                submit_input(),
                &mut writer,
            );
            let (actual, delivery) = result.unwrap();
            assert_eq!(actual, expected.unwrap().0);
            match mode {
                "obsolete-on-flush" => {
                    assert_eq!(delivery.unwrap(), 1);
                    assert_eq!(writer.calls.iter().filter(|&&op| op == "flush").count(), 1);
                    assert!(
                        driver
                            .validate_current_native_diagnostics(session.id(), &snapshots)
                            .is_err()
                    );
                }
                "wire-reopened" => {
                    assert_eq!(delivery.unwrap(), 0);
                    assert!(writer.calls.is_empty());
                    assert!(writer.bytes.is_empty());
                }
                _ => unreachable!(),
            }
        }
        std::fs::remove_dir_all(input().plan_request.workspace_root.as_str()).unwrap();
    }
}

#[cfg(test)]
pub(super) mod full_snapshot_registry_a {
    use super::*;
    use mizar_build::{
        module_index::{
            ModuleId, StaticSourceLayout, WorkspaceSourceFile, WorkspaceSourcePackage,
            build_module_index,
        },
        planner::{
            DependencySelection, PlanRequest, WorkspacePackage, lockfile_content_hash,
            parse_lockfile, parse_package_manifest, produce_build_plan,
            workspace_config_content_hash,
        },
        task_graph::{
            EvidenceCandidateId, ModuleDependencyOverlay, VcOrderKey, VcTaskDescriptor,
            VcTaskDescriptorId,
        },
    };
    use mizar_driver::{
        driver::{BuildSubmission, CompilerDriver, DriverSubmitInput},
        registry::PhaseRegistryBuilder,
        request::{
            BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
            BuildRequestGeneration, BuildRequestOrigin, BuildTargets, DependencyInputSet,
            SourceInputSet, VerifierConfigInput,
        },
    };
    use mizar_ir::publisher::OutputAuthorization;
    use mizar_ir::{
        identity::{OutputKind, PipelinePhase, SnapshotHandleRegistry, WorkUnit},
        publisher::{AllowedWorkUnit, PhaseOutputPublisher},
        storage::IrStorageService,
    };
    use mizar_session::{
        DiskSourceLoader, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
        SnapshotRegistry, ToolchainInfo, WorkspaceRoot,
    };
    use serde_json::{Value, json};
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::{
            Arc,
            atomic::{AtomicU64, Ordering},
        },
    };

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    #[derive(Clone, Default)]
    pub(super) struct RegistryObservingAllocator {
        inner: Arc<InMemorySessionIdAllocator>,
        pub(super) leases: Arc<std::sync::Mutex<Vec<mizar_session::SnapshotLeaseId>>>,
    }

    impl SessionIdAllocator for RegistryObservingAllocator {
        fn next_session_id(&self) -> Result<mizar_session::BuildSessionId, mizar_session::IdError> {
            self.inner.next_session_id()
        }
        fn next_request_id(&self) -> Result<mizar_session::BuildRequestId, mizar_session::IdError> {
            self.inner.next_request_id()
        }
        fn next_source_id(
            &self,
            snapshot: mizar_session::BuildSnapshotId,
        ) -> Result<mizar_session::SourceId, mizar_session::IdError> {
            self.inner.next_source_id(snapshot)
        }
        fn next_source_map_id(
            &self,
            snapshot: mizar_session::BuildSnapshotId,
        ) -> Result<mizar_session::SourceMapId, mizar_session::IdError> {
            self.inner.next_source_map_id(snapshot)
        }
        fn next_lease_id(
            &self,
            snapshot: mizar_session::BuildSnapshotId,
        ) -> Result<mizar_session::SnapshotLeaseId, mizar_session::IdError> {
            let id = self.inner.next_lease_id(snapshot)?;
            self.leases.lock().unwrap().push(id);
            Ok(id)
        }
    }

    pub(super) struct RegistryFixture {
        pub(super) server: Server,
        pub(super) ids: RegistryObservingAllocator,
        pub(super) snapshots: Arc<SnapshotRegistry<RegistryObservingAllocator>>,
        pub(super) driver: CompilerDriver,
        pub(super) publisher: Arc<PhaseOutputPublisher>,
        pub(super) draft: BuildRequestDraft,
        pub(super) index: mizar_build::module_index::ModuleIndex,
        pub(super) sources: Vec<(DiskSourceLoader, PackageId, String)>,
        pub(super) input_template: DriverSubmitInput<StaticSourceLayout>,
        root: PathBuf,
        raw: [String; 2],
        generation: u64,
    }

    impl RegistryFixture {
        pub(super) fn new() -> Self {
            let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "mizar-lsp-full-snapshot-{}-{sequence}",
                std::process::id()
            ));
            let package_root = root.join("alpha");
            fs::create_dir_all(package_root.join("src")).unwrap();
            fs::create_dir(package_root.join("artifacts")).unwrap();
            let raw = [
                "theorem Main: for X being set holds X = X proof let X be set; thus X = X; end;"
                    .to_owned(),
                "theorem Leaf: for X being set holds X = X proof let X be set; thus X = X; end;"
                    .to_owned(),
            ];
            for (module, text) in ["main", "leaf"].into_iter().zip(raw.iter()) {
                fs::write(package_root.join(format!("src/{module}.miz")), text).unwrap();
            }
            let workspace = WorkspaceRoot::new(root.to_string_lossy().into_owned());
            let plan_request = PlanRequest {
                workspace_root: workspace.clone(),
                dependency_selection: DependencySelection::Normal,
                toolchain: ToolchainInfo::new("mizar-evo-test"),
            };
            let packages = vec![WorkspacePackage {
                member_path: "alpha".into(),
                manifest: parse_package_manifest(
                    "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n",
                )
                .unwrap(),
            }];
            let lock = parse_lockfile("schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n").unwrap();
            let layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
                package_id: PackageId::new("alpha"),
                files: ["main", "leaf"]
                    .map(|module| {
                        WorkspaceSourceFile::new(
                            format!("src/{module}.miz"),
                            format!("{module}.miz"),
                        )
                    })
                    .to_vec(),
            }]);
            let plan =
                produce_build_plan(plan_request.clone(), packages.clone(), lock.clone()).unwrap();
            let index = build_module_index(&plan, &layout, &[]).unwrap();
            let dependency_overlay = ModuleDependencyOverlay::complete(Vec::new());
            let mut input_template = DriverSubmitInput::new(
                plan_request.clone(),
                packages.clone(),
                lock.clone(),
                layout.clone(),
            );
            input_template.dependency_overlay = dependency_overlay;
            input_template.vc_descriptors = ["main", "leaf"]
                .map(|module| {
                    VcTaskDescriptor::new(
                        VcTaskDescriptorId::new(format!("snapshot-{module}")),
                        ModuleId::new(PackageId::new("alpha"), ModulePath::new(module)),
                        VcOrderKey::new("00000000000000000000"),
                        vec![],
                        vec![EvidenceCandidateId::new("native-reflexivity")],
                    )
                })
                .to_vec();
            let draft = BuildRequestDraft {
                lane: BuildLaneId::new(1),
                origin: BuildRequestOrigin::Batch(BatchRequest {
                    invocation: BatchInvocation::default(),
                }),
                generation: BuildRequestGeneration::new(0),
                workspace_root: workspace,
                profile: BuildProfile::new("check"),
                targets: BuildTargets::default(),
                source_inputs: SourceInputSet::default(),
                dependency_inputs: DependencyInputSet::new(
                    vec![],
                    lockfile_content_hash(&plan.lockfile),
                    plan.toolchain.clone(),
                ),
                verifier_config: VerifierConfigInput::new(workspace_config_content_hash(
                    &plan.verifier_config,
                    &plan.build_config,
                )),
            };
            let ids = RegistryObservingAllocator::default();
            let snapshots = Arc::new(SnapshotRegistry::with_allocator(ids.clone()));
            let mut server = Server::default();
            initialize(&mut server);
            let sources = ["main", "leaf"].into_iter().map(|module| {
                let uri = file_uri(&package_root.join(format!("src/{module}.miz")));
                server.handle_message(notification("textDocument/didOpen", json!({"textDocument":{"uri":uri,"languageId":"mizar","version":1,"text":raw[usize::from(module == "leaf") ]}})));
                (DiskSourceLoader::new(&package_root), PackageId::new("alpha"), uri)
            }).collect::<Vec<_>>();
            let registry = mizar_driver::facade::assemble_registry(
                &mizar_driver::facade::StageConfig::source_theorem(vec![]),
            )
            .unwrap();
            let publisher = Arc::new(PhaseOutputPublisher::new(
                Arc::new(IrStorageService::new()),
                Arc::new(SnapshotHandleRegistry::new()),
            ));
            let driver = CompilerDriver::new(registry).with_output_publisher(publisher.clone());
            Self {
                server,
                ids,
                snapshots,
                driver,
                publisher,
                draft,
                index,
                sources,
                input_template,
                root,
                raw,
                generation: 1,
            }
        }

        pub(super) fn driver_input(&self) -> DriverSubmitInput<StaticSourceLayout> {
            let mut input = DriverSubmitInput::new(
                self.input_template.plan_request.clone(),
                self.input_template.workspace_packages.clone(),
                self.input_template.lockfile.clone(),
                self.input_template.source_layout.clone(),
            );
            input.dependency_overlay = self.input_template.dependency_overlay.clone();
            input.vc_descriptors = self.input_template.vc_descriptors.clone();
            input
        }
        pub(super) fn capture(&mut self) -> NativeOpenBufferCapture {
            let mut draft = self.draft.clone();
            draft.generation = BuildRequestGeneration::new(self.generation);
            self.generation += 1;
            let capture = self
                .server
                .capture_native_open_buffers(
                    draft,
                    &self.ids,
                    &self.snapshots,
                    &self.index,
                    self.sources.clone(),
                )
                .unwrap();
            // A publication generation owns its actual publisher, as in the Driver fixture.
            // Reusing equal-id output coordinates would attempt to store them twice.
            let publisher = Arc::new(PhaseOutputPublisher::new(
                Arc::new(IrStorageService::new()),
                Arc::new(SnapshotHandleRegistry::new()),
            ));
            publisher.register_current_snapshot(capture.snapshot.captured().snapshot().id);
            for module in ["main", "leaf"] {
                for (phase, kind) in [
                    ("SourceLoad", "SourceUnit"),
                    ("Frontend", "FrontendOutput"),
                    ("ModuleResolve", "SourceTheoremModuleCandidates"),
                    ("SignatureCollection", "SourceTheoremModuleSummary"),
                    ("TypeChecking", "SourceTheoremTypedAst"),
                    ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                    ("OverloadResolution", "SourceTheoremMetadata"),
                    ("Elaboration", "SourceTheoremCore"),
                    ("AlgorithmPreparation", "SourceTheoremPreparation"),
                    ("VcGenerate", "SourceTheoremVC"),
                    ("VcDischarge", "SourceTheoremDischarge"),
                    ("AtpSolve", "SourceTheoremCandidateInputs"),
                    ("KernelCheck", "SourceTheoremCandidateCheck"),
                    ("KernelCheck", "SourceTheoremCandidateWitnessFile"),
                    ("ArtifactCommit", "SourceTheoremArtifactData"),
                ] {
                    publisher.allow_work_unit(AllowedWorkUnit::new(
                        PipelinePhase::new(phase),
                        OutputKind::new(kind),
                        WorkUnit::new(format!("{:?}:{:?}", "alpha", module)),
                    ));
                }
            }
            self.publisher = publisher;
            let previous = std::mem::replace(
                &mut self.driver,
                CompilerDriver::new(PhaseRegistryBuilder::new().build().unwrap()),
            );
            self.driver = previous.with_output_publisher(self.publisher.clone());
            capture
        }
        pub(super) fn build(&mut self) -> (BuildSubmission, bool) {
            let capture = self.capture();
            let input = self.driver_input();
            self.server
                .submit_originating_snapshot(
                    &mut self.driver,
                    capture,
                    &self.ids,
                    &self.snapshots,
                    &self.publisher,
                    input,
                )
                .unwrap()
        }
        pub(super) fn submit_captured(
            &mut self,
            capture: NativeOpenBufferCapture,
        ) -> (BuildSubmission, Option<Vec<Value>>) {
            let input = self.driver_input();
            self.server
                .submit_originating_native_frontend_diagnostics(
                    &mut self.driver,
                    capture,
                    &self.ids,
                    &self.snapshots,
                    input,
                )
                .unwrap()
        }
        pub(super) fn frontend_only_driver(&self) -> CompilerDriver {
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load().register_frontend(vec![]);
            CompilerDriver::new(builder.build().unwrap())
                .with_output_publisher(self.publisher.clone())
        }
        pub(super) fn did_change_version_only(&mut self, uri: &str, version: i32) {
            self.server.handle_message(notification(
                "textDocument/didChange",
                json!({"textDocument":{"uri":uri,"version":version},"contentChanges":[]}),
            ));
        }
        pub(super) fn did_change_text(&mut self, uri: &str, version: i32, text: String) {
            self.server.handle_message(notification("textDocument/didChange", json!({"textDocument":{"uri":uri,"version":version},"contentChanges":[{"text":text}]})));
        }
        pub(super) fn did_close_and_reopen(&mut self, uri: &str, version: i32) {
            let text = self.server.document(uri).unwrap().1.to_owned();
            self.server.handle_message(notification(
                "textDocument/didClose",
                json!({"textDocument":{"uri":uri}}),
            ));
            self.server.handle_message(notification("textDocument/didOpen", json!({"textDocument":{"uri":uri,"languageId":"mizar","version":version,"text":text}})));
        }
    }

    impl Drop for RegistryFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn file_uri(path: &Path) -> String {
        format!("file://{}", path.to_string_lossy().replace('\\', "/"))
    }
    fn initialize(server: &mut Server) {
        server.handle_message(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}));
    }
    fn notification(method: &str, params: Value) -> Value {
        json!({"jsonrpc":"2.0","method":method,"params":params})
    }

    #[test]
    fn genuine_complete_payload_manifest_first_activates_one_coherent_snapshot() {
        let mut fixture = RegistryFixture::new();
        let (submission, activated) = fixture.build();
        assert!(activated);
        let (snapshot, freshness) = fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .expect("published snapshot");
        assert_eq!(freshness, LspSnapshotFreshness::Current);
        let publication = snapshot.publication();
        assert_eq!(publication.captured(), submission.session.captured());
        assert_eq!(publication.manifests().len(), 2);
        assert_eq!(publication.artifacts().len(), 2);
        assert_eq!(
            publication.diagnostics().snapshot(),
            submission.session.captured().snapshot().id
        );
        assert!(Arc::ptr_eq(
            snapshot.native_open_buffers(),
            &fixture.server.native_open_buffer_snapshot().unwrap()
        ));
        assert!(!snapshot.outputs().is_empty());
        for output in snapshot.outputs() {
            fixture.publisher.storage().validate_handle(output).unwrap();
        }
    }

    #[test]
    fn accepted_monotonic_edits_including_version_only_invalidate_currentness() {
        let mut fixture = RegistryFixture::new();
        fixture.build();
        let uri = fixture.sources[0].2.clone();
        let changed_text = format!("{}\n", fixture.server.document(&uri).unwrap().1);
        fixture.did_change_text(&uri, 2, changed_text);
        assert_eq!(fixture.server.document(&uri).unwrap().0, 2);
        assert_eq!(
            fixture
                .server
                .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                .unwrap()
                .1,
            LspSnapshotFreshness::StaleArtifact
        );
        fixture.did_change_version_only(&uri, 1);
        assert_eq!(fixture.server.document(&uri).unwrap().0, 2);
        fixture.did_change_version_only(&uri, 3);
        assert_eq!(fixture.server.document(&uri).unwrap().0, 3);
        assert_eq!(
            fixture
                .server
                .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                .unwrap()
                .1,
            LspSnapshotFreshness::StaleArtifact
        );
    }

    #[test]
    fn equal_id_edits_and_identical_tuple_reopen_do_not_revive_old_current_snapshot() {
        let mut fixture = RegistryFixture::new();
        fixture.build();
        let uri = fixture.sources[0].2.clone();
        let original = fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap()
            .0;
        let old_id = original.publication().captured().snapshot().id;
        fixture.did_change_version_only(&uri, 2);
        assert_eq!(original.publication().captured().snapshot().id, old_id);
        assert_eq!(
            fixture
                .server
                .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                .unwrap()
                .1,
            LspSnapshotFreshness::StaleArtifact
        );
        let (version_submission, activated) = fixture.build();
        assert!(activated);
        assert_eq!(version_submission.session.captured().snapshot().id, old_id);
        assert_eq!(
            fixture
                .server
                .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                .unwrap()
                .1,
            LspSnapshotFreshness::Current
        );
        let equivalent_raw = format!("\u{feff}{}", fixture.raw[0]);
        fixture.did_change_text(&uri, 3, equivalent_raw.clone());
        assert_eq!(original.publication().captured().snapshot().id, old_id);
        assert_eq!(
            fixture
                .server
                .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                .unwrap()
                .1,
            LspSnapshotFreshness::StaleArtifact
        );
        let (raw_submission, activated) = fixture.build();
        assert!(activated);
        assert_eq!(raw_submission.session.captured().snapshot().id, old_id);
        let (before_reopen, freshness) = fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap();
        assert_eq!(freshness, LspSnapshotFreshness::Current);
        assert!(original.publisher.validate_current_snapshot(old_id).is_ok());
        fixture.did_close_and_reopen(&uri, 3);
        assert_eq!(
            fixture.server.document(&uri),
            Some((3, equivalent_raw.as_str()))
        );
        assert!(std::sync::Arc::ptr_eq(
            &before_reopen,
            &fixture
                .server
                .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                .unwrap()
                .0
        ));
        assert_eq!(
            fixture
                .server
                .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                .unwrap()
                .1,
            LspSnapshotFreshness::StaleArtifact
        );
        assert!(fixture.server.native_open_buffer_snapshot().is_none());
    }

    #[test]
    fn retained_snapshot_after_edit_keeps_original_source_and_is_never_current() {
        let mut fixture = RegistryFixture::new();
        fixture.build();
        let (retained, freshness) = fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap();
        assert_eq!(freshness, LspSnapshotFreshness::Current);
        let source_before = retained
            .native_open_buffers()
            .captured()
            .native_sources()
            .unwrap();
        let original_source_text = Arc::clone(&source_before[0].1.text);
        let uri = fixture.sources[0].2.clone();
        fixture.did_change_version_only(&uri, 2);
        let (read_after, freshness) = fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap();
        assert_eq!(freshness, LspSnapshotFreshness::StaleArtifact);
        assert!(Arc::ptr_eq(&retained, &read_after));
        let source_after = read_after
            .native_open_buffers()
            .captured()
            .native_sources()
            .unwrap();
        assert_eq!(source_after.len(), source_before.len());
        assert!(Arc::ptr_eq(&source_after[0].1.text, &original_source_text));
        assert_eq!(source_after[0].2.origin, source_before[0].2.origin);
        let snapshot_id = read_after.publication().captured().snapshot().id;
        fixture.publisher.mark_obsolete(snapshot_id).unwrap();
        assert_eq!(
            fixture
                .server
                .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                .unwrap()
                .1,
            LspSnapshotFreshness::StaleArtifact
        );
        assert!(
            fixture
                .publisher
                .validate_output(
                    snapshot_id,
                    &read_after.outputs()[0],
                    OutputAuthorization::Published
                )
                .is_err()
        );
        assert_eq!(fixture.server.document(&uri).unwrap().0, 2);
    }
}

#[cfg(test)]
pub(super) mod full_snapshot_registry_b {
    use super::full_snapshot_registry_a::RegistryFixture;
    use super::*;

    #[test]
    fn reader_spanning_replacement_observes_one_complete_immutable_set() {
        let mut fixture = RegistryFixture::new();
        let (old_submission, old_activated) = fixture.build();
        assert!(old_activated);
        let (old_reader, old_freshness) = fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .expect("first complete snapshot");
        assert_eq!(old_freshness, LspSnapshotFreshness::Current);
        let old_publication = old_reader.publication().clone();
        let old_association = old_reader.native_open_buffers().clone();
        assert_eq!(old_publication.manifests().len(), 2);
        assert_eq!(old_publication.artifacts().len(), 2);
        assert_eq!(old_publication.captured(), old_association.captured());
        assert_eq!(
            old_publication.captured(),
            old_submission.session.captured()
        );

        let uri = fixture.sources[0].2.clone();
        fixture.did_change_version_only(&uri, 2);
        assert_eq!(
            fixture
                .server
                .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                .unwrap()
                .1,
            LspSnapshotFreshness::StaleArtifact
        );
        let (new_submission, new_activated) = fixture.build();
        assert!(new_activated);
        let (new_reader, new_freshness) = fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .expect("replacement complete snapshot");
        assert_eq!(new_freshness, LspSnapshotFreshness::Current);
        assert!(!std::sync::Arc::ptr_eq(&old_reader, &new_reader));
        assert_eq!(new_reader.publication().manifests().len(), 2);
        assert_eq!(new_reader.publication().artifacts().len(), 2);
        assert_eq!(
            new_reader.publication().captured(),
            new_submission.session.captured()
        );
        assert_eq!(
            new_reader.publication().captured(),
            new_reader.native_open_buffers().captured()
        );

        // The in-flight reader still holds the entire old publication and association.
        assert!(std::sync::Arc::ptr_eq(
            old_reader.publication(),
            &old_publication
        ));
        assert!(std::sync::Arc::ptr_eq(
            old_reader.native_open_buffers(),
            &old_association
        ));
        assert_eq!(old_reader.publication().manifests().len(), 2);
        assert_eq!(old_reader.publication().artifacts().len(), 2);
        assert_eq!(
            old_reader.publication().captured(),
            old_submission.session.captured()
        );
        assert_eq!(
            fixture
                .server
                .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                .unwrap()
                .0
                .publication()
                .captured(),
            new_submission.session.captured()
        );
    }

    #[test]
    fn session_and_ir_retention_release_once_after_last_association_and_reader() {
        let mut fixture = RegistryFixture::new();
        let lease_start = fixture.ids.leases.lock().unwrap().len();
        let (submission, activated) = fixture.build();
        assert!(activated);
        let snapshot_id = submission.session.captured().snapshot().id;
        let published_lease = *fixture.ids.leases.lock().unwrap().last().unwrap();
        assert!(fixture.ids.leases.lock().unwrap().len() > lease_start);
        let (reader, freshness) = fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .expect("published snapshot");
        assert_eq!(freshness, LspSnapshotFreshness::Current);
        let outputs = reader.outputs().to_vec();
        assert!(!outputs.is_empty());
        let storage = fixture.publisher.storage().clone();
        for output in &outputs {
            storage.validate_handle(output).unwrap();
        }

        let uri = fixture.sources[0].2.clone();
        let changed_text = format!("{}\n", fixture.server.document(&uri).unwrap().1);
        fixture.did_change_text(&uri, 2, changed_text);
        let other_capture = fixture.capture();
        let other_id = other_capture.snapshot.captured().snapshot().id;
        assert_ne!(other_id, snapshot_id);
        // A wrong-snapshot release checks this exact live lease without consuming it.
        assert!(matches!(
            fixture.snapshots.release_lease(other_id, published_lease),
            Err(mizar_session::SnapshotError::LeaseReleaseMismatch { lease_id, actual_snapshot, .. })
                if lease_id == published_lease && actual_snapshot == snapshot_id
        ));
        drop(other_capture);
        assert_eq!(
            fixture
                .server
                .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                .unwrap()
                .1,
            LspSnapshotFreshness::StaleArtifact
        );
        let held = storage.collect(mizar_ir::storage::CollectInput {
            snapshot: snapshot_id,
            protected_outputs: Vec::new(),
        });
        assert!(held.retained_outputs >= outputs.len());
        for output in &outputs {
            storage.validate_handle(output).unwrap();
        }

        // The server association is released first; the reader remains the last owner.
        drop(fixture.server.lsp_snapshot.take());
        assert!(matches!(
            fixture.snapshots.release_lease(other_id, published_lease),
            Err(mizar_session::SnapshotError::LeaseReleaseMismatch { lease_id, actual_snapshot, .. })
                if lease_id == published_lease && actual_snapshot == snapshot_id
        ));
        let still_held = storage.collect(mizar_ir::storage::CollectInput {
            snapshot: snapshot_id,
            protected_outputs: Vec::new(),
        });
        assert!(still_held.retained_outputs >= outputs.len());
        for output in &outputs {
            storage.validate_handle(output).unwrap();
        }
        drop(reader);
        assert!(matches!(
            fixture.snapshots.release_lease(snapshot_id, published_lease),
            Err(mizar_session::SnapshotError::UnknownSnapshotLease { lease_id })
                if lease_id == published_lease
        ));

        let released = storage.collect(mizar_ir::storage::CollectInput {
            snapshot: snapshot_id,
            protected_outputs: Vec::new(),
        });
        assert!(released.outputs_dropped >= outputs.len());
        for output in &outputs {
            assert!(storage.validate_handle(output).is_err());
        }
        let collected_again = storage.collect(mizar_ir::storage::CollectInput {
            snapshot: snapshot_id,
            protected_outputs: Vec::new(),
        });
        assert_eq!(collected_again.outputs_dropped, 0);
        assert!(matches!(
            fixture.snapshots.release_lease(snapshot_id, published_lease),
            Err(mizar_session::SnapshotError::UnknownSnapshotLease { lease_id })
                if lease_id == published_lease
        ));
    }

    #[test]
    fn partial_index_and_artifact_boundary_inputs_do_not_activate_or_clear() {
        use mizar_driver::events::BuildEventKind;

        let mut fixture = RegistryFixture::new();
        let (full_submission, activated) = fixture.build();
        assert!(activated);
        let (retained, _) = fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap();
        let original_publication = retained.publication().clone();
        let original_diagnostic_count = original_publication.diagnostics().records().len();

        // Genuine frontend/index-only submission. It returns a diagnostic index but no
        // coherent SnapshotPublished payload and cannot replace the prior publication.
        let capture = fixture.capture();
        let association = fixture.server.native_open_buffer_snapshot().unwrap();
        let frontend_driver = fixture.frontend_only_driver();
        let complete_driver = std::mem::replace(&mut fixture.driver, frontend_driver);
        let (partial, notifications) = fixture.submit_captured(capture);
        // Native notifications are derived from the genuine returned BuildDiagnosticIndex.
        assert_eq!(
            notifications
                .expect("genuine native diagnostic index")
                .len(),
            2
        );
        let partial_events = fixture.driver.events(partial.session.id());
        assert!(
            !partial_events
                .events()
                .iter()
                .any(|event| { matches!(&event.kind, BuildEventKind::SnapshotPublished { .. }) })
        );
        let run = partial.scheduler_run.as_ref().expect("frontend run");
        assert!(
            run.phase_results
                .values()
                .flatten()
                .all(|result| { result.artifact_commit.is_none() })
        );
        assert!(!fixture.server.activate_lsp_snapshot(
            &fixture.driver,
            &partial,
            association,
            &fixture.snapshots,
            &fixture.publisher,
        ));
        fixture.driver = complete_driver;

        // A genuine full-pipeline submission with a malformed leaf yields an artifact boundary
        // for the successful module but no complete cohort publication.
        let leaf_uri = fixture.sources[1].2.clone();
        fixture.did_change_text(&leaf_uri, 2, "theorem Leaf: invalid Mizar.".to_owned());
        let (artifact_partial, artifact_activated) = fixture.build();
        assert!(!artifact_activated);
        let artifact_events = fixture.driver.events(artifact_partial.session.id());
        assert!(
            artifact_events
                .events()
                .iter()
                .any(|event| { matches!(&event.kind, BuildEventKind::ArtifactBoundary { .. }) })
        );
        assert!(
            !artifact_events
                .events()
                .iter()
                .any(|event| { matches!(&event.kind, BuildEventKind::SnapshotPublished { .. }) })
        );
        assert!(!fixture.server.activate_lsp_snapshot(
            &fixture.driver,
            &artifact_partial,
            fixture.server.native_open_buffer_snapshot().unwrap(),
            &fixture.snapshots,
            &fixture.publisher,
        ));

        let (still_selected, freshness) = fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .expect("previous complete publication remains retained");
        assert!(std::sync::Arc::ptr_eq(&retained, &still_selected));
        assert_eq!(freshness, LspSnapshotFreshness::StaleArtifact);
        assert_eq!(
            still_selected.publication().diagnostics().records().len(),
            original_diagnostic_count
        );
        assert_eq!(
            still_selected.publication().captured(),
            full_submission.session.captured()
        );
        assert_eq!(still_selected.publication().manifests().len(), 2);
        assert_eq!(still_selected.publication().artifacts().len(), 2);
        assert!(std::sync::Arc::ptr_eq(
            still_selected.publication(),
            &original_publication
        ));
    }

    #[test]
    fn superseded_equal_id_driver_submission_makes_old_payload_stale_and_unactivatable() {
        let mut fixture = RegistryFixture::new();
        let (old_submission, activated) = fixture.build();
        assert!(activated);
        let association = fixture.server.native_open_buffer_snapshot().unwrap();
        let (old_reader, freshness) = fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap();
        assert_eq!(freshness, LspSnapshotFreshness::Current);

        // Capture the same accepted source inputs directly through Session and submit a newer
        // request on the same lane. The Server's accepted-document association is untouched.
        let source_inputs = old_submission
            .session
            .captured()
            .native_sources()
            .unwrap()
            .iter()
            .zip(&fixture.sources)
            .map(|((_, _, input), (loader, _, _))| (loader.clone(), input.clone()))
            .collect();
        let mut newer_draft = fixture.draft.clone();
        newer_draft.generation = mizar_driver::request::BuildRequestGeneration::new(2);
        let newer_session = newer_draft
            .capture_sources(&fixture.ids, &fixture.snapshots, source_inputs)
            .unwrap();
        assert_eq!(
            newer_session.captured().snapshot().id,
            old_submission.session.captured().snapshot().id
        );
        assert_eq!(
            newer_session.request().lane,
            old_submission.session.request().lane
        );
        let newer_submission = fixture
            .driver
            .submit_captured(
                newer_session,
                &fixture.ids,
                &fixture.snapshots,
                fixture.driver_input(),
                mizar_driver::driver::SubmitOptions {
                    discover_imports: true,
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(
            newer_submission.session.request().lane,
            old_submission.session.request().lane
        );
        assert_eq!(
            newer_submission.session.captured().snapshot().id,
            old_submission.session.captured().snapshot().id
        );
        assert!(std::sync::Arc::ptr_eq(
            &association,
            &fixture.server.native_open_buffer_snapshot().unwrap()
        ));
        let (selected, freshness) = fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap();
        assert!(std::sync::Arc::ptr_eq(&selected, &old_reader));
        assert_eq!(freshness, LspSnapshotFreshness::StaleArtifact);
        assert!(!fixture.server.activate_lsp_snapshot(
            &fixture.driver,
            &old_submission,
            association,
            &fixture.snapshots,
            &fixture.publisher,
        ));
        assert!(std::sync::Arc::ptr_eq(
            &fixture
                .server
                .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                .unwrap()
                .0,
            &old_reader
        ));
    }
}

#[cfg(test)]
mod task9_publication_api_tests {
    use super::full_snapshot_registry_a::RegistryFixture;
    use super::*;
    use std::io::{self, Write};

    #[derive(Default)]
    struct RecordingWriter {
        bytes: Vec<u8>,
        fail_at: Option<&'static str>,
        in_body: bool,
        obsolete_on_flush: Option<(
            std::sync::Arc<mizar_ir::publisher::PhaseOutputPublisher>,
            mizar_session::BuildSnapshotId,
        )>,
        flushes: usize,
    }

    impl Write for RecordingWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let operation = if self.in_body { "body" } else { "header" };
            if self.fail_at == Some(operation) {
                return Err(io::Error::other(operation));
            }
            self.bytes.extend_from_slice(bytes);
            self.in_body |= self.bytes.ends_with(b"\r\n\r\n");
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            if self.fail_at == Some("flush") {
                return Err(io::Error::other("flush"));
            }
            if let Some((publisher, snapshot)) = self.obsolete_on_flush.take() {
                publisher.mark_obsolete(snapshot).unwrap();
            }
            self.in_body = false;
            Ok(())
        }
    }

    fn publish(fixture: &RegistryFixture) -> Option<Vec<Value>> {
        fixture
            .server
            .snapshot_diagnostic_notifications(&fixture.driver, &fixture.snapshots)
            .unwrap()
    }

    #[test]
    fn coherent_snapshot_publishes_each_exact_version_and_clean_empty_diagnostics() {
        let mut fixture = RegistryFixture::new();
        assert!(publish(&fixture).is_none());
        let mut unavailable_writer = RecordingWriter::default();
        assert_eq!(
            fixture
                .server
                .publish_snapshot_diagnostics_to_writer(
                    &fixture.driver,
                    &fixture.snapshots,
                    &mut unavailable_writer,
                )
                .unwrap(),
            0
        );
        assert!(unavailable_writer.bytes.is_empty());
        assert_eq!(unavailable_writer.flushes, 0);
        fixture.build();

        let notifications = publish(&fixture).expect("current coherent publication");
        assert_eq!(notifications.len(), 2);
        let expected_uris = fixture
            .sources
            .iter()
            .map(|(_, _, uri)| uri.clone())
            .collect::<std::collections::BTreeSet<_>>();
        let actual_uris = notifications
            .iter()
            .map(|notification| notification["params"]["uri"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(actual_uris, expected_uris.into_iter().collect::<Vec<_>>());

        for notification in &notifications {
            assert_eq!(notification["jsonrpc"], "2.0");
            assert_eq!(notification["method"], "textDocument/publishDiagnostics");
            let params = &notification["params"];
            let uri = params["uri"].as_str().unwrap();
            assert_eq!(params["version"], 1);
            assert_eq!(params["diagnostics"], json!([]));
            assert!(
                fixture
                    .sources
                    .iter()
                    .any(|(_, _, source_uri)| source_uri == uri)
            );
        }

        // A fresh coherent generation produces explicit empty arrays to clear prior diagnostics.
        let changed_uri = fixture.sources[0].2.clone();
        fixture.did_change_version_only(&changed_uri, 2);
        assert!(publish(&fixture).is_none());
        fixture.build();
        let clearing = publish(&fixture).expect("replacement current publication");
        assert_eq!(clearing.len(), 2);
        assert!(
            clearing
                .iter()
                .all(|n| n["params"]["diagnostics"] == json!([]))
        );
        assert!(clearing.iter().any(|n| n["params"]["version"] == 2));
    }

    #[test]
    fn stale_or_unavailable_publication_is_suppressed_across_editor_and_terminal_lifecycle() {
        for edit in ["version", "normalized-raw", "identical-reopen", "recapture"] {
            let mut fixture = RegistryFixture::new();
            fixture.build();
            let uri = fixture.sources[0].2.clone();
            let retained = fixture
                .server
                .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                .unwrap()
                .0;
            match edit {
                "version" => fixture.did_change_version_only(&uri, 2),
                "normalized-raw" => fixture.did_change_text(
                    &uri,
                    2,
                    format!("\u{feff}{}", fixture.server.document(&uri).unwrap().1),
                ),
                "identical-reopen" => fixture.did_close_and_reopen(&uri, 1),
                "recapture" => {
                    fixture.capture();
                }
                _ => unreachable!(),
            }
            assert!(publish(&fixture).is_none(), "edit case: {edit}");
            let mut writer = RecordingWriter::default();
            assert_eq!(
                fixture
                    .server
                    .publish_snapshot_diagnostics_to_writer(
                        &fixture.driver,
                        &fixture.snapshots,
                        &mut writer,
                    )
                    .unwrap(),
                0,
                "stale edit case: {edit}"
            );
            assert!(writer.bytes.is_empty(), "stale edit case: {edit}");
            assert_eq!(writer.flushes, 0, "stale edit case: {edit}");
            assert!(std::sync::Arc::ptr_eq(
                &retained,
                &fixture
                    .server
                    .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                    .unwrap()
                    .0
            ));
            assert_eq!(retained.publication().manifests().len(), 2);
        }

        for terminal in ["obsolete", "shutdown", "exit"] {
            let mut fixture = RegistryFixture::new();
            let (submission, activated) = fixture.build();
            assert!(activated);
            if terminal == "obsolete" {
                fixture
                    .publisher
                    .mark_obsolete(submission.session.captured().snapshot().id)
                    .unwrap();
            } else if terminal == "shutdown" {
                let _ = fixture.server.handle_message(json!({
                    "jsonrpc":"2.0", "id":9, "method":"shutdown", "params":null
                }));
            } else {
                let _ = fixture.server.handle_message(json!({
                    "jsonrpc":"2.0", "method":"exit", "params":null
                }));
            }
            assert!(publish(&fixture).is_none(), "terminal case: {terminal}");
            let mut writer = RecordingWriter::default();
            assert_eq!(
                fixture
                    .server
                    .publish_snapshot_diagnostics_to_writer(
                        &fixture.driver,
                        &fixture.snapshots,
                        &mut writer,
                    )
                    .unwrap(),
                0,
                "terminal case: {terminal}"
            );
            assert!(writer.bytes.is_empty(), "terminal case: {terminal}");
            assert_eq!(writer.flushes, 0, "terminal case: {terminal}");
        }
    }

    #[test]
    fn writer_emits_exact_framed_current_notifications_and_stops_after_obsolescence() {
        let mut fixture = RegistryFixture::new();
        fixture.build();
        let notifications = publish(&fixture).unwrap();
        let mut expected = Vec::new();
        for notification in &notifications {
            let body = serde_json::to_vec(notification).unwrap();
            expected
                .extend_from_slice(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes());
            expected.extend_from_slice(&body);
        }
        let first_body = serde_json::to_vec(&notifications[0]).unwrap();
        let first_header = format!("Content-Length: {}\r\n\r\n", first_body.len());
        let mut first_frame = first_header.as_bytes().to_vec();
        first_frame.extend_from_slice(&first_body);
        let mut writer = RecordingWriter::default();
        assert_eq!(
            fixture
                .server
                .publish_snapshot_diagnostics_to_writer(
                    &fixture.driver,
                    &fixture.snapshots,
                    &mut writer,
                )
                .unwrap(),
            2
        );
        assert_eq!(writer.bytes, expected);
        assert_eq!(writer.flushes, 2);

        let snapshot_id = fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap()
            .0
            .publication()
            .captured()
            .snapshot()
            .id;
        let mut obsolete_writer = RecordingWriter {
            obsolete_on_flush: Some((fixture.publisher.clone(), snapshot_id)),
            ..Default::default()
        };
        assert_eq!(
            fixture
                .server
                .publish_snapshot_diagnostics_to_writer(
                    &fixture.driver,
                    &fixture.snapshots,
                    &mut obsolete_writer,
                )
                .unwrap(),
            1
        );
        assert_eq!(obsolete_writer.flushes, 1);
        assert_eq!(obsolete_writer.bytes, first_frame);
    }

    #[test]
    fn writer_propagates_header_body_and_flush_failures_without_claiming_delivery() {
        let mut fixture = RegistryFixture::new();
        fixture.build();
        let notifications = publish(&fixture).unwrap();
        let first_body = serde_json::to_vec(&notifications[0]).unwrap();
        let header = format!("Content-Length: {}\r\n\r\n", first_body.len());
        let mut complete_first_frame = header.as_bytes().to_vec();
        complete_first_frame.extend_from_slice(&first_body);
        for fail_at in ["header", "body", "flush"] {
            let mut writer = RecordingWriter {
                fail_at: Some(fail_at),
                ..Default::default()
            };
            let error = fixture
                .server
                .publish_snapshot_diagnostics_to_writer(
                    &fixture.driver,
                    &fixture.snapshots,
                    &mut writer,
                )
                .unwrap_err();
            assert_eq!(error.to_string(), fail_at);
            assert_eq!(writer.flushes, usize::from(fail_at == "flush"));
            if fail_at == "header" {
                assert!(writer.bytes.is_empty());
            } else if fail_at == "body" {
                assert_eq!(writer.bytes.as_slice(), header.as_bytes());
            } else {
                assert_eq!(writer.bytes, complete_first_frame);
            }
        }
    }
}

#[cfg(test)]
mod determinism_freshness_matrix_tests {
    use super::full_snapshot_registry_a::RegistryFixture;
    use super::*;
    use crate::LspPosition;
    use std::sync::Arc;

    fn request(method: &str, params: Value) -> Value {
        json!({"jsonrpc":"2.0", "id":77, "method":method, "params":params})
    }

    fn feature_replies(
        fixture: &mut RegistryFixture,
        uri: &str,
        position: LspPosition,
        reverse: bool,
    ) -> [Value; 3] {
        let messages = [
            request(
                "textDocument/hover",
                json!({"textDocument":{"uri":uri},
                "position":{"line":position.line,"character":position.character}}),
            ),
            request(
                "textDocument/semanticTokens/full",
                json!({"textDocument":{"uri":uri}}),
            ),
            request(
                "textDocument/codeAction",
                json!({"textDocument":{"uri":uri},
                "range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},
                "context":{"diagnostics":[]}}),
            ),
        ];
        let mut replies = [Value::Null, Value::Null, Value::Null];
        for index in if reverse { [2, 1, 0] } else { [0, 1, 2] } {
            replies[index] = fixture
                .server
                .handle_message_with_metadata(
                    messages[index].clone(),
                    &fixture.driver,
                    &fixture.snapshots,
                )
                .unwrap();
        }
        replies
    }

    fn notifications(fixture: &RegistryFixture) -> Option<Vec<Value>> {
        fixture
            .server
            .snapshot_diagnostic_notifications(&fixture.driver, &fixture.snapshots)
            .unwrap()
    }

    fn frames(fixture: &RegistryFixture) -> (usize, Vec<u8>) {
        let mut bytes = Vec::new();
        let count = fixture
            .server
            .publish_snapshot_diagnostics_to_writer(&fixture.driver, &fixture.snapshots, &mut bytes)
            .unwrap();
        (count, bytes)
    }

    #[test]
    fn genuine_replay_is_invariant_across_six_layouts_and_query_order() {
        let base = "theorem Main: for X being set holds X = X proof let X be set; thus X = X; end;";
        for newline in ["\n", "\r\n"] {
            for spaces in ["", " ", "  "] {
                let mut fixture = RegistryFixture::new();
                for (_, _, uri) in fixture.sources.clone() {
                    fixture.did_change_text(&uri, 2, format!(":: café{newline}{spaces}{base}"));
                }
                assert!(fixture.build().1);
                let (baseline, freshness) = fixture
                    .server
                    .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                    .unwrap();
                assert_eq!(freshness, LspSnapshotFreshness::Current);
                let baseline_id = baseline.publication().captured().snapshot().id;
                let baseline_diagnostics = notifications(&fixture).unwrap();
                assert_eq!(baseline_diagnostics.len(), 2);
                assert!(baseline_diagnostics.iter().all(
                    |n| n["params"]["version"] == 2 && n["params"]["diagnostics"] == json!([])
                ));
                let baseline_frames = frames(&fixture);
                assert_eq!(baseline_frames.0, 2);
                assert!(!baseline_frames.1.is_empty());
                let uris = fixture
                    .sources
                    .iter()
                    .map(|source| source.2.clone())
                    .collect::<Vec<_>>();
                let position = LspPosition {
                    line: 1,
                    character: u32::try_from(spaces.len() + base.find("thus X").unwrap() + 5)
                        .unwrap(),
                };
                let baseline_features = uris
                    .iter()
                    .map(|uri| feature_replies(&mut fixture, uri, position, false))
                    .collect::<Vec<_>>();
                assert!(baseline_features.iter().all(|[hover, tokens, actions]| {
                    hover["result"]["contents"]["value"]
                        .as_str()
                        .is_some_and(|s| !s.is_empty())
                        && hover["result"]["range"].is_object()
                        && !tokens["result"]["data"].as_array().unwrap().is_empty()
                        && actions["result"] == json!([])
                }));
                fixture.sources.reverse();
                fixture.input_template.vc_descriptors.reverse();
                assert!(fixture.build().1);
                let (replay, freshness) = fixture
                    .server
                    .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                    .unwrap();
                assert_eq!(freshness, LspSnapshotFreshness::Current);
                assert_eq!(replay.publication().captured().snapshot().id, baseline_id);
                assert_eq!(notifications(&fixture).unwrap(), baseline_diagnostics);
                assert_eq!(frames(&fixture), baseline_frames);
                for (index, uri) in uris.iter().enumerate().rev() {
                    assert_eq!(
                        feature_replies(&mut fixture, uri, position, true),
                        baseline_features[index]
                    );
                }
            }
        }
    }

    #[test]
    fn invalidation_orders_keep_original_publication_readable_and_suppress_diagnostics() {
        let orders: [&[u8]; 8] = [
            &[0, 1, 2],
            &[0, 2, 1],
            &[1, 0, 2],
            &[1, 2, 0],
            &[2, 0, 1],
            &[2, 1, 0],
            &[3],
            &[4],
        ];
        for order in orders {
            let mut fixture = RegistryFixture::new();
            assert!(fixture.build().1);
            let uri = fixture.sources[0].2.clone();
            let (retained, freshness) = fixture
                .server
                .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                .unwrap();
            assert_eq!(freshness, LspSnapshotFreshness::Current);
            let initial_text = fixture.server.document(&uri).unwrap().1.to_owned();
            // The original fixture is one ASCII line; stale lookup keeps these original coordinates.
            let position = LspPosition {
                line: 0,
                character: u32::try_from(initial_text.find("thus X").unwrap() + 5).unwrap(),
            };
            let original = feature_replies(&mut fixture, &uri, position, false);
            let original_hover = original[0]["result"]["contents"]["value"].as_str().unwrap();
            assert!(original[0]["result"]["range"].is_object());
            for step in order {
                let version = fixture.server.document(&uri).unwrap().0;
                match step {
                    0 => {
                        fixture.did_change_version_only(&uri, version + 1);
                        assert_eq!(fixture.server.document(&uri).unwrap().0, version + 1);
                    }
                    1 => {
                        fixture.did_change_text(
                            &uri,
                            version + 1,
                            format!("\u{feff}{initial_text}"),
                        );
                        assert_eq!(fixture.server.document(&uri).unwrap().0, version + 1);
                        assert_eq!(
                            fixture.server.document(&uri).unwrap().1,
                            format!("\u{feff}{initial_text}")
                        );
                    }
                    2 => fixture.did_close_and_reopen(&uri, version),
                    3 => {
                        fixture.capture();
                    }
                    4 => {
                        fixture
                            .publisher
                            .mark_obsolete(retained.publication().captured().snapshot().id)
                            .unwrap();
                    }
                    _ => unreachable!(),
                }
                let (selected, freshness) = fixture
                    .server
                    .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                    .unwrap();
                assert_eq!(freshness, LspSnapshotFreshness::StaleArtifact);
                assert!(Arc::ptr_eq(&retained, &selected));
                assert_eq!(notifications(&fixture), None);
                assert_eq!(frames(&fixture), (0, Vec::new()));
                let replies = feature_replies(&mut fixture, &uri, position, true);
                let stale = &replies[0]["result"];
                let text = stale["contents"]["value"].as_str().unwrap();
                assert!(
                    text.contains("stale_artifact")
                        && text.contains(&uri)
                        && text.contains("version 1")
                );
                assert!(text.contains(original_hover));
                assert_eq!(stale.get("range"), None);
                assert_eq!(
                    feature_replies(&mut fixture, &uri, position, false)[1],
                    replies[1]
                );
                // A separate syntax-only connection is the oracle for live buffer input.
                let mut live = Server::default();
                live.handle_message(request(
                    "initialize",
                    json!({"processId":null,"rootUri":null,"capabilities":{}}),
                ));
                live.handle_message(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                    "textDocument":{"uri":uri,"languageId":"mizar","version":1,"text":fixture.server.document(&uri).unwrap().1}}}));
                assert_eq!(
                    live.handle_message(request(
                        "textDocument/semanticTokens/full",
                        json!({"textDocument":{"uri":uri}})
                    ))
                    .unwrap(),
                    replies[1]
                );
            }
            if !order.contains(&4) {
                assert!(fixture.build().1);
                let (rebuilt, freshness) = fixture
                    .server
                    .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                    .unwrap();
                assert_eq!(freshness, LspSnapshotFreshness::Current);
                assert_eq!(
                    rebuilt.publication().captured().snapshot().id,
                    retained.publication().captured().snapshot().id
                );
                assert_eq!(notifications(&fixture).unwrap().len(), 2);
                assert_eq!(frames(&fixture).0, 2);
                let position = LspPosition {
                    character: position.character
                        + u32::from(
                            fixture
                                .server
                                .document(&uri)
                                .unwrap()
                                .1
                                .starts_with('\u{feff}'),
                        ),
                    ..position
                };
                let fresh = feature_replies(&mut fixture, &uri, position, false);
                assert!(fresh[0]["result"]["range"].is_object());
                assert!(
                    !fresh[0]["result"]["contents"]["value"]
                        .as_str()
                        .unwrap()
                        .contains("stale_artifact")
                );
                assert_eq!(feature_replies(&mut fixture, &uri, position, true), fresh);
            }
        }
    }
}

#[cfg(test)]
mod diagnostic_overlay_tests {
    use super::full_snapshot_registry_a::RegistryFixture;
    use super::*;
    use mizar_diagnostics::failure_record::DiagnosticPrimaryLocation;
    use std::sync::Arc;

    fn submit_overlay(
        fixture: &mut RegistryFixture,
        capture: NativeOpenBufferCapture,
    ) -> Result<
        (mizar_driver::driver::BuildSubmission, bool),
        mizar_driver::driver::DriverSubmitError,
    > {
        let input = fixture.driver_input();
        fixture.server.submit_originating_diagnostic_overlay(
            &mut fixture.driver,
            capture,
            &fixture.ids,
            &fixture.snapshots,
            input,
        )
    }

    fn overlay(
        fixture: &RegistryFixture,
        uri: &str,
        version: i32,
    ) -> Option<Arc<LspDiagnosticOverlay>> {
        fixture
            .server
            .diagnostic_overlay(uri, version, &fixture.driver, &fixture.snapshots)
    }

    #[test]
    fn error_and_clean_cohort_reads_preserve_original_source_records_and_order() {
        let mut fixture = RegistryFixture::new();
        let error_uri = fixture.sources[0].2.clone();
        fixture.did_change_text(&error_uri, 2, "theorem Main: invalid Mizar.".to_owned());
        let capture = fixture.capture();
        let (submission, installed) =
            submit_overlay(&mut fixture, capture).expect("native frontend diagnostic submission");
        assert!(installed);
        assert!(submission.diagnostic_index.is_none());
        let error = overlay(&fixture, &error_uri, 2).expect("exact error key");
        assert_eq!(error.document(), error_uri);
        assert_eq!(error.version(), 2);
        let error_source_id = error.source().1.source_id;
        let expected = error
            .index
            .handles_for_source(error_source_id)
            .unwrap_or_default()
            .iter()
            .map(|handle| error.index.record_by_id(handle.id).unwrap())
            .collect::<Vec<_>>();
        let actual = error.diagnostics().collect::<Vec<_>>();
        assert!(!actual.is_empty());
        assert_eq!(actual, expected);
        assert!(actual.iter().all(|record| {
            matches!(record.primary_location(), DiagnosticPrimaryLocation::Span(span)
                if span.range().source_id == error_source_id)
        }));
        assert!(overlay(&fixture, &error_uri, 1).is_none());
        assert!(overlay(&fixture, "file:///not-open.miz", 2).is_none());
        let clean_uri = fixture.sources[1].2.clone();
        let clean = overlay(&fixture, &clean_uri, 1).expect("clean cohort member");
        assert_eq!(clean.document(), clean_uri);
        assert_eq!(clean.version(), 1);
        assert_eq!(clean.diagnostics().count(), 0);
        assert!(Arc::ptr_eq(&error.index, &clean.index));
        let original_sources = fixture
            .driver
            .session(submission.session.id())
            .unwrap()
            .captured()
            .native_sources()
            .unwrap();
        assert!(std::ptr::eq(
            error.source(),
            &original_sources[error.source_position]
        ));
        assert!(std::ptr::eq(
            clean.source(),
            &original_sources[clean.source_position]
        ));
        assert!(overlay(&fixture, &clean_uri, 2).is_none());
    }

    #[test]
    fn replacement_clean_clearing_and_failed_work_preserve_old_readers() {
        let mut fixture = RegistryFixture::new();
        let uri = fixture.sources[0].2.clone();
        fixture.did_change_text(&uri, 2, "theorem Main: invalid Mizar.".to_owned());
        let capture = fixture.capture();
        let (_, installed) =
            submit_overlay(&mut fixture, capture).expect("first eligible submission");
        assert!(installed);
        let old = overlay(&fixture, &uri, 2).expect("first reader");
        let old_records = old.diagnostics().cloned().collect::<Vec<_>>();
        assert!(!old_records.is_empty());

        let capture = fixture.capture();
        let (_, replaced) =
            submit_overlay(&mut fixture, capture).expect("same version eligible replacement");
        assert!(replaced);
        let same_version = overlay(&fixture, &uri, 2).expect("same-version replacement");
        assert!(!Arc::ptr_eq(&old, &same_version));
        assert_eq!(old.diagnostics().cloned().collect::<Vec<_>>(), old_records);

        fixture.did_change_text(
            &uri,
            3,
            "theorem Main: for X being set holds X = X proof let X be set; thus X = X; end;"
                .to_owned(),
        );
        let capture = fixture.capture();
        let (_, cleared) =
            submit_overlay(&mut fixture, capture).expect("clean eligible replacement");
        assert!(cleared);
        let clean = overlay(&fixture, &uri, 3).expect("clean empty replacement");
        assert_eq!(clean.diagnostics().count(), 0);
        assert!(overlay(&fixture, &uri, 2).is_none());
        assert_eq!(old.diagnostics().cloned().collect::<Vec<_>>(), old_records);

        let stale = fixture.capture();
        let _newer_capture = fixture.capture();
        let (_, ineligible) = submit_overlay(&mut fixture, stale)
            .expect("old native capture remains a real submission");
        assert!(!ineligible);
        assert!(Arc::ptr_eq(
            &clean,
            fixture
                .server
                .diagnostic_overlays
                .get(&(uri.clone(), 3))
                .unwrap()
        ));

        let capture = fixture.capture();
        let mut unavailable = mizar_driver::driver::CompilerDriver::new(
            mizar_driver::registry::PhaseRegistryBuilder::new()
                .build()
                .unwrap(),
        );
        let input = fixture.driver_input();
        let invalid_registry = mizar_session::SnapshotRegistry::with_allocator(fixture.ids.clone());
        assert!(
            fixture
                .server
                .submit_originating_diagnostic_overlay(
                    &mut unavailable,
                    capture,
                    &fixture.ids,
                    &invalid_registry,
                    input,
                )
                .is_err()
        );
        assert!(Arc::ptr_eq(
            &clean,
            fixture
                .server
                .diagnostic_overlays
                .get(&(uri.clone(), 3))
                .unwrap()
        ));
    }

    #[test]
    fn currentness_tracks_versions_incarnations_and_server_lifecycle() {
        for action in [
            "version",
            "normalized",
            "close-reopen",
            "invalid",
            "unrelated",
        ] {
            let mut fixture = RegistryFixture::new();
            let uri = fixture.sources[0].2.clone();
            let capture = fixture.capture();
            assert!(submit_overlay(&mut fixture, capture).unwrap().1);
            let retained = overlay(&fixture, &uri, 1).unwrap();
            let old_snapshot_id = retained.association.captured.snapshot().id;
            match action {
                "version" => fixture.did_change_version_only(&uri, 2),
                "normalized" => fixture.did_change_text(
                    &uri,
                    2,
                    format!("\u{feff}{}", fixture.server.document(&uri).unwrap().1),
                ),
                "close-reopen" => fixture.did_close_and_reopen(&uri, 1),
                "invalid" => {
                    let _ = fixture.server.handle_message(json!({
                        "jsonrpc":"2.0", "method":"textDocument/didChange",
                        "params":{"textDocument":{"uri":uri,"version":0},"contentChanges":[{"text":"ignored"}]}
                    }));
                }
                "unrelated" => {
                    let _ = fixture.server.handle_message(json!({
                        "jsonrpc":"2.0", "method":"textDocument/didChange",
                        "params":{"textDocument":{"uri":"file:///unrelated.miz","version":2},"contentChanges":[{"text":"ignored"}]}
                    }));
                }
                _ => unreachable!(),
            }
            if matches!(action, "version" | "normalized") {
                assert!(
                    overlay(&fixture, &uri, 1).is_none(),
                    "before recapture: {action}"
                );
                let next = fixture.capture();
                assert_eq!(
                    old_snapshot_id,
                    next.snapshot.captured().snapshot().id,
                    "version-only and BOM-normalized text retain the original build id"
                );
            }
            if matches!(action, "invalid" | "unrelated") {
                assert!(Arc::ptr_eq(&retained, &overlay(&fixture, &uri, 1).unwrap()));
            } else {
                assert!(overlay(&fixture, &uri, 1).is_none(), "{action}");
            }
            assert_eq!(retained.document(), uri);
            match &retained.source().2.origin {
                mizar_session::SourceOriginInput::OpenBuffer {
                    actual_version,
                    text,
                    ..
                } => {
                    assert_eq!(*actual_version, 1);
                    assert!(text.starts_with("theorem Main:"));
                    assert!(!text.starts_with('\u{feff}'));
                }
                _ => panic!("retained overlay must keep its original open source"),
            }
        }

        let mut fixture = RegistryFixture::new();
        let uri = fixture.sources[0].2.clone();
        let capture = fixture.capture();
        assert!(submit_overlay(&mut fixture, capture).unwrap().1);
        let retained = overlay(&fixture, &uri, 1).unwrap();
        let mut other = Server::default();
        other.handle_message(json!({"jsonrpc":"2.0","id":1,"method":"initialize",
            "params":{"processId":null,"rootUri":null,"capabilities":{}}}));
        for (_, _, source_uri) in &fixture.sources {
            let (version, text) = fixture.server.document(source_uri).unwrap();
            other.handle_message(json!({"jsonrpc":"2.0","method":"textDocument/didOpen",
                "params":{"textDocument":{"uri":source_uri,"languageId":"mizar",
                    "version":version,"text":text}}}));
        }
        assert!(
            other
                .diagnostic_overlay(&uri, 1, &fixture.driver, &fixture.snapshots)
                .is_none()
        );
        // Fresh authorized outputs for a genuine producer on the second connection.
        drop(fixture.capture());
        let mut other_driver = fixture.frontend_only_driver();
        let capture = other
            .capture_native_open_buffers(
                fixture.draft.clone(),
                &fixture.ids,
                &fixture.snapshots,
                &fixture.index,
                fixture.sources.clone(),
            )
            .unwrap();
        let (_, installed) = other
            .submit_originating_diagnostic_overlay(
                &mut other_driver,
                capture,
                &fixture.ids,
                &fixture.snapshots,
                fixture.driver_input(),
            )
            .unwrap();
        assert!(installed);
        let other_reader = other
            .diagnostic_overlay(&uri, 1, &other_driver, &fixture.snapshots)
            .unwrap();
        assert!(!Arc::ptr_eq(&retained, &other_reader));
        assert!(Arc::ptr_eq(&retained, &overlay(&fixture, &uri, 1).unwrap()));
        let _ = fixture.server.handle_message(json!({
            "jsonrpc":"2.0", "id":9, "method":"shutdown", "params":null
        }));
        assert!(overlay(&fixture, &uri, 1).is_none());
        assert_eq!(retained.document(), uri);
        assert!(Arc::ptr_eq(
            &other_reader,
            &other
                .diagnostic_overlay(&uri, 1, &other_driver, &fixture.snapshots)
                .unwrap()
        ));

        let mut exiting = RegistryFixture::new();
        let uri = exiting.sources[0].2.clone();
        let capture = exiting.capture();
        assert!(submit_overlay(&mut exiting, capture).unwrap().1);
        let retained = overlay(&exiting, &uri, 1).unwrap();
        let _ = exiting
            .server
            .handle_message(json!({"jsonrpc":"2.0","method":"exit","params":null}));
        assert!(overlay(&exiting, &uri, 1).is_none());
        assert_eq!(retained.document(), uri);
    }

    #[test]
    fn retained_open_buffer_lease_releases_after_last_overlay_reader() {
        let mut fixture = RegistryFixture::new();
        let uri = fixture.sources[0].2.clone();
        fixture.did_change_text(&uri, 2, "theorem Main: invalid Mizar.".to_owned());
        let capture = fixture.capture();
        let (_, installed) =
            submit_overlay(&mut fixture, capture).expect("eligible syntax overlay");
        assert!(installed);
        let reader = overlay(&fixture, &uri, 2).unwrap();
        let snapshot_id = reader.association.captured().snapshot().id;
        let lease = *fixture.ids.leases.lock().unwrap().last().unwrap();

        fixture.did_change_text(
            &uri,
            3,
            "theorem Main: for X being set holds X = X proof let X be set; thus X = X; end;"
                .to_owned(),
        );
        let capture = fixture.capture();
        let other_snapshot = capture.snapshot.captured().snapshot().id;
        let _ = submit_overlay(&mut fixture, capture).unwrap();
        assert!(overlay(&fixture, &uri, 2).is_none());
        assert!(reader.source().1.text.contains("invalid Mizar"));
        assert!(matches!(
            fixture.snapshots.release_lease(other_snapshot, lease),
            Err(mizar_session::SnapshotError::LeaseReleaseMismatch { lease_id, actual_snapshot, .. })
                if lease_id == lease && actual_snapshot == snapshot_id
        ));
        drop(reader);
        assert!(matches!(
            fixture.snapshots.release_lease(snapshot_id, lease),
            Err(mizar_session::SnapshotError::UnknownSnapshotLease { lease_id }) if lease_id == lease
        ));
    }

    #[test]
    fn full_publication_clears_matching_keys_and_partial_work_does_not_clear() {
        let mut fixture = RegistryFixture::new();
        let main_uri = fixture.sources[0].2.clone();
        let leaf_uri = fixture.sources[1].2.clone();
        let capture = fixture.capture();
        let (_, installed) =
            submit_overlay(&mut fixture, capture).expect("eligible clean overlays");
        assert!(installed);
        let leaf = overlay(&fixture, &leaf_uri, 1).unwrap();
        let original_id = leaf.association.captured().snapshot().id;
        let (full, activated) = fixture.build();
        assert!(activated);
        assert_eq!(full.session.captured().snapshot().id, original_id);
        assert!(fixture.server.diagnostic_overlays.is_empty());
        assert_eq!(leaf.document(), leaf_uri);
        assert_eq!(leaf.diagnostics().count(), 0);

        let capture = fixture.capture();
        assert!(submit_overlay(&mut fixture, capture).unwrap().1);
        fixture.input_template.source_layout =
            mizar_build::module_index::StaticSourceLayout::new(vec![
                mizar_build::module_index::WorkspaceSourcePackage {
                    package_id: mizar_session::PackageId::new("alpha"),
                    files: vec![mizar_build::module_index::WorkspaceSourceFile::new(
                        "src/main.miz",
                        "main.miz",
                    )],
                },
            ]);
        fixture.input_template.vc_descriptors.truncate(1);
        fixture.sources.truncate(1);
        let (_, activated) = fixture.build();
        assert!(activated);
        assert!(
            !fixture
                .server
                .diagnostic_overlays
                .contains_key(&(main_uri, 1))
        );
        assert!(
            fixture
                .server
                .diagnostic_overlays
                .contains_key(&(leaf_uri.clone(), 1))
        );
        assert_eq!(leaf.document(), leaf_uri);

        let mut partial_fixture = RegistryFixture::new();
        let partial_main_uri = partial_fixture.sources[0].2.clone();
        let partial_leaf_uri = partial_fixture.sources[1].2.clone();
        partial_fixture.did_change_text(
            &partial_leaf_uri,
            2,
            "theorem Leaf: invalid Mizar.".to_owned(),
        );
        let capture = partial_fixture.capture();
        assert!(submit_overlay(&mut partial_fixture, capture).unwrap().1);
        let current = overlay(&partial_fixture, &partial_main_uri, 1).unwrap();
        let (partial, activated) = partial_fixture.build();
        assert!(!activated);
        assert!(
            partial_fixture
                .server
                .diagnostic_overlays
                .contains_key(&(partial_main_uri, 1))
        );
        assert!(
            partial_fixture
                .server
                .diagnostic_overlays
                .contains_key(&(partial_leaf_uri, 2))
        );
        assert_eq!(current.version(), 1);
        assert!(
            partial_fixture
                .driver
                .events(partial.session.id())
                .events()
                .iter()
                .all(|event| !matches!(
                    &event.kind,
                    mizar_driver::events::BuildEventKind::SnapshotPublished { .. }
                ))
        );
    }

    #[test]
    fn syntax_overlay_does_not_change_semantic_publication_cli_or_artifact_bytes() {
        use mizar_artifact::verified_artifact::write_verified_artifact;
        use mizar_build::module_index::WorkspaceSourceFile;
        use mizar_driver::cli::{CliBatchInput, CliSnapshotInputs, run_batch};

        let mut fixture = RegistryFixture::new();
        let (submission, activated) = fixture.build();
        assert!(activated);
        let (semantic_reader, _) = fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .expect("genuine semantic publication");
        let semantic_index = semantic_reader.publication().diagnostics().clone();
        assert!(semantic_index.is_empty());
        let artifact_bytes = semantic_reader
            .publication()
            .artifacts()
            .iter()
            .map(|artifact| write_verified_artifact(artifact).unwrap())
            .collect::<Vec<_>>();
        assert!(
            semantic_reader
                .publication()
                .artifacts()
                .iter()
                .all(|artifact| artifact.diagnostics.is_empty())
        );
        let request = submission.session.request();
        let cli_input = CliBatchInput::new(
            "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n",
            "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n",
            "alpha",
            request
                .source_inputs
                .versions
                .iter()
                .map(|version| {
                    WorkspaceSourceFile::new(
                        version.normalized_path.as_str(),
                        format!("{}.miz", version.module_path.as_str()),
                    )
                })
                .collect(),
            CliSnapshotInputs::new(
                request.source_inputs.clone(),
                request.dependency_inputs.clone(),
                request.verifier_config,
            ),
        );
        let workspace = fixture.draft.workspace_root.as_str().to_owned();
        let cli_args = [
            "mizar",
            "build",
            "--workspace",
            workspace.as_str(),
            "--message-format",
            "json",
        ];
        let cli_before = run_batch(cli_args, cli_input.clone());
        assert_ne!(cli_before.exit_code, mizar_driver::cli::CliExitCode::Usage);

        let uri = fixture.sources[0].2.clone();
        fixture.did_change_text(&uri, 2, "theorem Main: invalid Mizar.".to_owned());
        let capture = fixture.capture();
        fixture.driver = fixture.frontend_only_driver();
        let (overlay_submission, installed) =
            submit_overlay(&mut fixture, capture).expect("syntax-only frontend run");
        assert!(installed);
        assert!(overlay_submission.diagnostic_index.is_none());
        let syntax = overlay(&fixture, &uri, 2).unwrap();
        assert!(syntax.diagnostics().next().is_some());
        assert!(
            !fixture
                .driver
                .events(overlay_submission.session.id())
                .events()
                .iter()
                .any(|event| matches!(
                    event.kind,
                    mizar_driver::events::BuildEventKind::SnapshotPublished { .. }
                ))
        );
        let (still_selected, freshness) = fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap();
        assert!(Arc::ptr_eq(&semantic_reader, &still_selected));
        assert_eq!(freshness, LspSnapshotFreshness::StaleArtifact);
        assert_eq!(semantic_reader.publication().diagnostics(), &semantic_index);
        assert_eq!(
            semantic_reader
                .publication()
                .artifacts()
                .iter()
                .map(|artifact| write_verified_artifact(artifact).unwrap())
                .collect::<Vec<_>>(),
            artifact_bytes
        );
        assert_eq!(run_batch(cli_args, cli_input), cli_before);
    }
}

#[cfg(test)]
mod native_payload_producer_tests {
    use super::full_snapshot_registry_a::RegistryFixture;
    use super::*;
    use serde_json::{Value, json};

    const RAW_SOURCE: &str =
        "\u{feff}:: 前置 café 😀\r\n:: export profile\r\nexport Std theorem T;";

    fn enable_native_action_capabilities(fixture: &mut RegistryFixture) {
        let documents = fixture
            .sources
            .iter()
            .map(|(_, _, uri)| {
                let (version, text) = fixture.server.document(uri).unwrap();
                (uri.clone(), version, text.to_owned())
            })
            .collect::<Vec<_>>();
        let mut server = Server::default();
        assert!(server.enable_diagnostic_commands());
        server
            .handle_message(json!({
                "jsonrpc":"2.0", "id":1, "method":"initialize",
                "params":{"processId":null,"rootUri":null,"capabilities":{
                    "textDocument":{"codeAction":{"dataSupport":true,
                        "resolveSupport":{"properties":["edit","command"]}}}
                }}
            }))
            .unwrap();
        for (uri, version, text) in documents {
            server.handle_message(json!({"jsonrpc":"2.0", "method":"textDocument/didOpen",
                "params":{"textDocument":{"uri":uri,"languageId":"mizar",
                    "version":version,"text":text}}}));
        }
        fixture.server = server;
    }

    fn publish_export_error(
        fixture: &mut RegistryFixture,
    ) -> (mizar_driver::driver::BuildSubmission, String) {
        enable_native_action_capabilities(fixture);
        let uri = fixture.sources[0].2.clone();
        fixture.did_change_text(&uri, 2, RAW_SOURCE.to_owned());
        fixture.driver = fixture.frontend_only_driver();
        let capture = fixture.capture();
        let input = fixture.driver_input();
        let (submission, selected) = fixture
            .server
            .submit_originating_diagnostic_publication(
                &mut fixture.driver,
                capture,
                &fixture.ids,
                &fixture.snapshots,
                input,
            )
            .expect("native source-error publication");
        assert!(
            selected,
            "genuine current terminal capture must be selected"
        );
        assert!(
            fixture
                .server
                .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                .is_none()
        );
        (submission, uri)
    }

    fn notifications(fixture: &RegistryFixture) -> Vec<Value> {
        fixture
            .server
            .snapshot_diagnostic_notifications(&fixture.driver, &fixture.snapshots)
            .expect("ranged diagnostic conversion")
            .expect("selected current diagnostic publication")
    }

    fn export_diagnostic(fixture: &RegistryFixture, uri: &str) -> Value {
        notifications(fixture)
            .into_iter()
            .find(|notification| notification["params"]["uri"] == uri)
            .expect("main diagnostic notification")["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .find(|diagnostic| diagnostic["code"] == "E0041")
            .expect("native missing-export-semicolon diagnostic")
            .clone()
    }

    fn explain(fixture: &mut RegistryFixture, diagnostic: &Value, id: u64) -> Value {
        let request = json!({"jsonrpc":"2.0", "id":id, "method":"mizar/explain",
            "params":{"snapshot":diagnostic["data"]["snapshot"],
                "diagnosticId":diagnostic["data"]["id"]}});
        fixture
            .server
            .handle_message_with_native_explanations(
                request,
                &fixture.driver,
                &fixture.snapshots,
                &|| false,
            )
            .expect("explanation response")
    }

    fn code_actions(fixture: &mut RegistryFixture, uri: &str, id: u64) -> Value {
        let (_, text) = fixture.server.document(uri).unwrap();
        let last_line = text.split('\n').count().saturating_sub(1);
        let last_line_text = text
            .split('\n')
            .next_back()
            .unwrap_or_default()
            .trim_end_matches('\r');
        let end_character = last_line_text.encode_utf16().count();
        let request = json!({"jsonrpc":"2.0", "id":id, "method":"textDocument/codeAction",
            "params":{"textDocument":{"uri":uri},
                "range":{"start":{"line":0,"character":0},
                    "end":{"line":last_line,"character":end_character}},
                "context":{"diagnostics":[]}}});
        fixture
            .server
            .handle_message_with_metadata(request, &fixture.driver, &fixture.snapshots)
            .expect("code action response")["result"]
            .clone()
    }

    fn resolve(fixture: &mut RegistryFixture, action: &Value, id: u64) -> Value {
        fixture
            .server
            .handle_message_with_metadata(
                json!({"jsonrpc":"2.0", "id":id, "method":"codeAction/resolve",
                    "params":action}),
                &fixture.driver,
                &fixture.snapshots,
            )
            .expect("resolve response")["result"]
            .clone()
    }

    fn native_actions(fixture: &mut RegistryFixture, uri: &str) -> (Value, Value, Value) {
        let diagnostic = export_diagnostic(fixture, uri);
        let actions = code_actions(fixture, uri, 20);
        let local = actions
            .as_array()
            .unwrap()
            .iter()
            .find(|action| {
                action["data"]["fix"]
                    .as_str()
                    .is_some_and(|fix| fix.ends_with(".insert"))
            })
            .expect("advisory insertion action");
        let command = actions
            .as_array()
            .unwrap()
            .iter()
            .find(|action| {
                action["data"]["fix"]
                    .as_str()
                    .is_some_and(|fix| fix.ends_with(".explain"))
            })
            .expect("bound explanation command action");
        (diagnostic, local.clone(), command.clone())
    }

    #[test]
    fn genuine_export_error_delivers_raw_mapped_edit_bounded_context_and_command() {
        let mut fixture = RegistryFixture::new();
        let (submission, uri) = publish_export_error(&mut fixture);
        assert!(submission.diagnostic_index.is_some());
        let frames = notifications(&fixture);
        let mut wire = Vec::new();
        let written = fixture
            .server
            .publish_snapshot_diagnostics_to_writer(&fixture.driver, &fixture.snapshots, &mut wire)
            .expect("ranged diagnostic frames");
        assert_eq!(written, frames.len());
        assert!(wire.starts_with(b"Content-Length: "));
        assert!(
            wire.windows(uri.len())
                .any(|window| window == uri.as_bytes())
        );
        let (diagnostic, local, command) = native_actions(&mut fixture, &uri);
        assert_eq!(
            diagnostic["range"]["start"],
            json!({"line":2,"character":11})
        );
        assert_eq!(diagnostic["range"]["end"], json!({"line":2,"character":18}));

        let resolved_edit = resolve(&mut fixture, &local, 21);
        let edit = &resolved_edit["edit"]["documentChanges"][0];
        assert_eq!(edit["textDocument"]["uri"], uri);
        assert_eq!(edit["textDocument"]["version"], 2);
        assert_eq!(
            edit["edits"][0]["range"]["start"],
            json!({"line":2,"character":11})
        );
        assert_eq!(
            edit["edits"][0]["range"]["end"],
            json!({"line":2,"character":11})
        );
        assert_eq!(edit["edits"][0]["newText"], ";");

        let explanation = explain(&mut fixture, &diagnostic, 22);
        assert_eq!(explanation["result"]["status"], "available");
        assert_eq!(
            explanation["result"]["payload"]["format"],
            "structured_text"
        );
        let payload_text = explanation["result"]["payload"]["text"].as_str().unwrap();
        assert!(payload_text.len() <= 256, "native context remains bounded");
        let payload: Value = serde_json::from_str(payload_text).unwrap();
        let raw_insertion = RAW_SOURCE.find("theorem").unwrap();
        let expected_loaded_offset = raw_insertion - '\u{feff}'.len_utf8() - 2;
        let loaded = fixture
            .server
            .ranged_diagnostics
            .as_ref()
            .unwrap()
            .association
            .captured
            .native_sources()
            .unwrap()
            .iter()
            .find(|(_, _, input)| {
                matches!(
                    &input.origin,
                    mizar_session::SourceOriginInput::OpenBuffer { uri: source_uri, .. }
                        if source_uri == &uri
                )
            })
            .map(|(_, loaded, _)| loaded)
            .unwrap();
        assert_eq!(payload["category"], "syntax.missing_semicolon");
        assert_eq!(payload["site"], "export-before-item");
        assert_eq!(payload["expected"], ";");
        assert_eq!(payload["insertion_offset"], expected_loaded_offset);
        assert_eq!(loaded.text.find("theorem"), Some(expected_loaded_offset));
        assert_eq!(
            payload_text,
            format!("{}\n", payload),
            "context is canonical JSON"
        );
        assert_eq!(explanation["result"]["locations"][0]["uri"], uri);
        assert_eq!(explanation["result"]["locations"][0]["version"], 2);
        assert_eq!(
            explanation["result"]["locations"][0]["range"]["start"],
            json!({"line":2,"character":11})
        );
        let cancelled = fixture
            .server
            .handle_message_with_native_explanations(
                json!({"jsonrpc":"2.0", "id":25, "method":"mizar/explain",
                    "params":{"snapshot":diagnostic["data"]["snapshot"],
                        "diagnosticId":diagnostic["data"]["id"]}}),
                &fixture.driver,
                &fixture.snapshots,
                &|| true,
            )
            .unwrap();
        assert_eq!(cancelled["error"]["code"], -32800);

        let resolved_command = resolve(&mut fixture, &command, 23);
        let command = &resolved_command["command"];
        assert_eq!(command["command"], "mizar.executeDiagnosticCommand");
        let executed = fixture
            .server
            .handle_message_with_diagnostic_commands(
                json!({"jsonrpc":"2.0", "id":24, "method":"workspace/executeCommand",
                    "params":{"command":command["command"],"arguments":command["arguments"]}}),
                &fixture.driver,
                &fixture.snapshots,
                &|| false,
            )
            .unwrap();
        assert_eq!(executed["result"]["status"], "available");
        assert_eq!(executed["result"]["payload"]["text"], payload_text);
        assert!(
            fixture
                .server
                .lsp_snapshot(&fixture.driver, &fixture.snapshots)
                .is_none()
        );
    }

    #[test]
    fn stale_genuine_payload_refuses_actions_but_retains_original_explanation() {
        for transition in [
            "version-only",
            "normalized-equal",
            "preceding-line",
            "close-reopen",
        ] {
            let mut fixture = RegistryFixture::new();
            let (_, uri) = publish_export_error(&mut fixture);
            let (diagnostic, local, command) = native_actions(&mut fixture, &uri);
            let original = explain(&mut fixture, &diagnostic, 30);
            let retained_text = original["result"]["payload"]["text"].clone();
            match transition {
                "version-only" => fixture.did_change_version_only(&uri, 3),
                "normalized-equal" => fixture.did_change_text(
                    &uri,
                    3,
                    RAW_SOURCE
                        .trim_start_matches('\u{feff}')
                        .replace("\r\n", "\n"),
                ),
                "preceding-line" => fixture.did_change_text(
                    &uri,
                    3,
                    format!(
                        "\u{feff}:: inserted preceding line\r\n{}",
                        RAW_SOURCE.trim_start_matches('\u{feff}')
                    ),
                ),
                "close-reopen" => fixture.did_close_and_reopen(&uri, 2),
                _ => unreachable!(),
            }
            let after = explain(&mut fixture, &diagnostic, 31);
            assert_eq!(after["result"]["status"], "available", "{transition}");
            assert_eq!(
                after["result"]["freshness"], "stale_artifact",
                "{transition}"
            );
            assert_eq!(
                after["result"]["payload"]["text"], retained_text,
                "{transition}"
            );
            assert_eq!(after["result"]["locations"][0]["uri"], uri, "{transition}");
            assert_eq!(
                after["result"]["locations"][0]["version"], 2,
                "{transition}"
            );
            assert_eq!(
                after["result"]["locations"][0]["range"]["start"],
                json!({"line":2,"character":11}),
                "{transition}"
            );
            assert_eq!(
                after["result"]["locations"][0]["range"]["end"],
                json!({"line":2,"character":18}),
                "{transition}"
            );
            assert_eq!(
                code_actions(&mut fixture, &uri, 32),
                json!([]),
                "{transition}"
            );
            assert!(
                resolve(&mut fixture, &local, 33).is_null(),
                "stale edit locator: {transition}"
            );
            assert!(
                resolve(&mut fixture, &command, 34).is_null(),
                "stale command locator: {transition}"
            );
            let execute = fixture.server.handle_message_with_diagnostic_commands(
                json!({"jsonrpc":"2.0", "id":35, "method":"workspace/executeCommand",
                    "params":{"command":"mizar.executeDiagnosticCommand",
                        "arguments":[command["data"]]}}),
                &fixture.driver,
                &fixture.snapshots,
                &|| false,
            );
            assert_eq!(execute.unwrap()["error"]["code"], -32801, "{transition}");
        }
    }

    #[test]
    fn equal_id_replacement_cannot_reuse_old_locator_or_backing() {
        let mut fixture = RegistryFixture::new();
        let (_, uri) = publish_export_error(&mut fixture);
        let (old_diagnostic, _local, old_command) = native_actions(&mut fixture, &uri);
        let old_snapshot = old_diagnostic["data"]["snapshot"].clone();
        let old_id = old_diagnostic["data"]["id"].as_u64().unwrap();
        let old_identity = fixture
            .server
            .ranged_diagnostics
            .as_ref()
            .unwrap()
            .identity
            .clone();
        let old_snapshot_id = fixture
            .server
            .ranged_diagnostics
            .as_ref()
            .unwrap()
            .index
            .snapshot();
        let old_publication =
            std::sync::Arc::clone(fixture.server.ranged_diagnostics.as_ref().unwrap());
        let old_store = old_publication.store.clone();
        let old_handle = old_publication
            .index
            .records()
            .iter()
            .find(|record| record.handle().id.get() == old_id)
            .and_then(|record| record.explanation())
            .expect("original canonical explanation handle")
            .clone();
        let prior_native = fixture.server.native_open_buffer_snapshot().unwrap();
        let capture = fixture.capture();
        assert_eq!(
            capture.snapshot.captured().snapshot().id,
            prior_native.captured().snapshot().id
        );
        let input = fixture.driver_input();
        let (submission, installed) = fixture
            .server
            .submit_originating_diagnostic_publication(
                &mut fixture.driver,
                capture,
                &fixture.ids,
                &fixture.snapshots,
                input,
            )
            .expect("equal-id replacement submission");
        assert!(installed);
        assert_eq!(
            submission.session.captured().snapshot().id,
            prior_native.captured().snapshot().id
        );
        assert_ne!(
            fixture.server.ranged_diagnostics.as_ref().unwrap().identity,
            old_identity
        );
        let selected = fixture.server.ranged_diagnostics.as_ref().unwrap().clone();
        assert!(!std::sync::Arc::ptr_eq(&selected.store, &old_store));
        let new_handle = selected
            .index
            .records()
            .iter()
            .find(|record| record.code().to_string() == "E0041")
            .and_then(|record| record.explanation())
            .expect("replacement canonical explanation handle");
        assert_ne!(old_handle.source(), new_handle.source());
        assert!(!matches!(
            selected
                .store
                .resolve(&old_handle, Some(selected.index.snapshot())),
            mizar_diagnostics::explain::ExplanationResolution::Available(_)
        ));
        assert!(matches!(
            old_store.resolve(&old_handle, Some(old_snapshot_id)),
            mizar_diagnostics::explain::ExplanationResolution::Available(_)
        ));
        let original_sources = old_publication
            .association
            .captured()
            .native_sources()
            .unwrap();
        let (_, old_loaded, old_input) = original_sources
            .iter()
            .find(|(_, _, input)| {
                matches!(
                    &input.origin,
                    mizar_session::SourceOriginInput::OpenBuffer { uri: source_uri, .. }
                        if source_uri == &uri
                )
            })
            .expect("retained original OpenBuffer source");
        assert!(old_loaded.text.starts_with(":: 前置 café 😀\n"));
        assert_eq!(old_loaded.line_map.source(), old_loaded.text.as_ref());
        assert!(matches!(
            &old_input.origin,
            mizar_session::SourceOriginInput::OpenBuffer {
                uri: old_uri,
                actual_version: 2,
                text,
                ..
            } if old_uri == &uri && text.as_ref() == RAW_SOURCE
        ));

        let old_read = fixture
            .server
            .handle_message_with_native_explanations(
                json!({"jsonrpc":"2.0", "id":40, "method":"mizar/explain",
                "params":{"snapshot":old_snapshot,"diagnosticId":old_id}}),
                &fixture.driver,
                &fixture.snapshots,
                &|| false,
            )
            .unwrap();
        assert_eq!(old_read["result"]["status"], "available");
        assert_eq!(old_read["result"]["freshness"], "current");
        assert_eq!(old_read["result"]["locations"][0]["uri"], uri);
        let mizar_diagnostics::explain::ExplanationResolution::Available(new_payload) = selected
            .store
            .resolve(new_handle, Some(selected.index.snapshot()))
        else {
            panic!("replacement publication retains its own backing")
        };
        assert_eq!(old_read["result"]["payload"]["text"], new_payload.text());
        let old_action = fixture
            .server
            .handle_message_with_metadata(
                json!({"jsonrpc":"2.0", "id":41, "method":"codeAction/resolve",
                "params":old_command}),
                &fixture.driver,
                &fixture.snapshots,
            )
            .unwrap();
        assert!(old_action["result"].is_null());
        let old_execute = fixture
            .server
            .handle_message_with_diagnostic_commands(
                json!({"jsonrpc":"2.0", "id":42, "method":"workspace/executeCommand",
                "params":{"command":"mizar.executeDiagnosticCommand",
                    "arguments":[old_command["data"]]}}),
                &fixture.driver,
                &fixture.snapshots,
                &|| false,
            )
            .unwrap();
        assert_eq!(old_execute["error"]["code"], -32801);

        let changed_source = RAW_SOURCE.replace("前置", "更新");
        fixture.did_change_text(&uri, 3, changed_source);
        let changed_capture = fixture.capture();
        let changed_input = fixture.driver_input();
        let (_, changed_selected) = fixture
            .server
            .submit_originating_diagnostic_publication(
                &mut fixture.driver,
                changed_capture,
                &fixture.ids,
                &fixture.snapshots,
                changed_input,
            )
            .expect("changed-source native replacement");
        assert!(changed_selected);
        let obsolete_request = fixture
            .server
            .handle_message_with_native_explanations(
                json!({"jsonrpc":"2.0", "id":43, "method":"mizar/explain",
                    "params":{"snapshot":old_snapshot,"diagnosticId":old_id}}),
                &fixture.driver,
                &fixture.snapshots,
                &|| false,
            )
            .unwrap();
        assert_eq!(obsolete_request["result"]["status"], "stale_handle");
    }

    #[test]
    fn incomplete_command_registration_keeps_diagnostic_edit_and_explanation_independent() {
        use mizar_diagnostics::explain::{
            ExplanationPayload, ExplanationPreviewFormat, ExplanationStore,
        };

        for (case, id) in [
            ("content-encoding", 50),
            ("content-bytes", 60),
            ("summary-encoding", 70),
            ("summary-bytes", 80),
            ("store-format", 90),
            ("store-truncated", 100),
            ("store-hash", 110),
        ] {
            let mut fixture = RegistryFixture::new();
            let (_, uri) = publish_export_error(&mut fixture);
            let (diagnostic, _local, _command) = native_actions(&mut fixture, &uri);
            let selected = fixture.server.ranged_diagnostics.as_ref().unwrap().clone();
            let registration_handle = fixture
                .server
                .deferred_fixes
                .iter()
                .find(|registered| registered.registration.command.is_some())
                .and_then(|registered| registered.registration.explanation.as_ref())
                .expect("native command registration has its exact handle")
                .clone();
            let original_payload = match selected
                .store
                .resolve(&registration_handle, Some(selected.index.snapshot()))
            {
                mizar_diagnostics::explain::ExplanationResolution::Available(payload) => payload,
                _ => panic!("native publication store contains the registered payload"),
            };
            let registration = &mut fixture
                .server
                .deferred_fixes
                .iter_mut()
                .find(|registered| registered.registration.command.is_some())
                .expect("native command registration")
                .registration;
            match case {
                "content-encoding" => {
                    registration.content.as_mut().unwrap().0.push_str(".wrong");
                }
                "content-bytes" => {
                    registration.content.as_mut().unwrap().1.push(b'!');
                }
                "summary-encoding" => {
                    registration.summary.as_mut().unwrap().0.push_str(".wrong");
                }
                "summary-bytes" => {
                    registration.summary.as_mut().unwrap().1.push(b'!');
                }
                "store-format" | "store-truncated" | "store-hash" => {
                    let (format, text, hash, max_bytes) = match case {
                        "store-format" => (
                            ExplanationPreviewFormat::PlainText,
                            original_payload.text().to_owned(),
                            original_payload.summary_hash(),
                            1024,
                        ),
                        "store-truncated" => (
                            ExplanationPreviewFormat::StructuredText,
                            original_payload.text().to_owned(),
                            original_payload.summary_hash(),
                            mizar_diagnostics::explain::TRUNCATION_MARKER.len(),
                        ),
                        "store-hash" => (
                            ExplanationPreviewFormat::StructuredText,
                            original_payload.text().to_owned(),
                            Some(mizar_session::hash_text("wrong summary hash")),
                            1024,
                        ),
                        _ => unreachable!(),
                    };
                    let payload = ExplanationPayload::with_bounds(format, text, hash, max_bytes, 8)
                        .expect("bounded altered payload");
                    let mut store = ExplanationStore::new();
                    store.insert_payload(&registration_handle, payload);
                    registration.store = Some(std::sync::Arc::new(store));
                }
                _ => unreachable!(),
            }

            let actions = code_actions(&mut fixture, &uri, id);
            assert_eq!(
                actions.as_array().unwrap().len(),
                1,
                "{case}: unsupported command payload is withheld independently"
            );
            let local = resolve(&mut fixture, &actions[0], id + 1);
            assert_eq!(
                local["edit"]["documentChanges"][0]["edits"][0]["newText"], ";",
                "{case}: independent local edit survives"
            );
            let explanation = explain(&mut fixture, &diagnostic, id + 2);
            assert_eq!(explanation["result"]["status"], "available", "{case}");
            assert_eq!(
                explanation["result"]["payload"]["text"],
                original_payload.text(),
                "{case}"
            );
            assert_eq!(export_diagnostic(&fixture, &uri)["code"], "E0041", "{case}");
        }
    }

    #[test]
    fn obsolete_capture_and_missing_host_do_not_admit_native_payloads() {
        let mut fixture = RegistryFixture::new();
        enable_native_action_capabilities(&mut fixture);
        let uri = fixture.sources[0].2.clone();
        fixture.did_change_text(&uri, 2, RAW_SOURCE.to_owned());
        fixture.driver = fixture.frontend_only_driver();
        let obsolete = fixture.capture();
        let newer = fixture.capture();
        let snapshot = newer
            .snapshot
            .captured()
            .snapshot()
            .id
            .to_published_schema_string()
            .unwrap();
        let input = fixture.driver_input();
        let (_, selected) = fixture
            .server
            .submit_originating_diagnostic_publication(
                &mut fixture.driver,
                obsolete,
                &fixture.ids,
                &fixture.snapshots,
                input,
            )
            .expect("obsolete capture remains a real but ineligible submission");
        assert!(!selected);
        assert!(
            fixture
                .server
                .snapshot_diagnostic_notifications(&fixture.driver, &fixture.snapshots)
                .unwrap()
                .is_none()
        );
        assert!(fixture.server.ranged_diagnostics.is_none());

        let mut no_host = Server::default();
        no_host.handle_message(json!({"jsonrpc":"2.0","id":1,"method":"initialize",
            "params":{"processId":null,"rootUri":null,"capabilities":{}}}));
        let unavailable = no_host
            .handle_message_with_native_explanations(
                json!({"jsonrpc":"2.0","id":2,"method":"mizar/explain",
                "params":{"snapshot":snapshot,"diagnosticId":0}}),
                &fixture.driver,
                &fixture.snapshots,
                &|| false,
            )
            .unwrap();
        assert_eq!(unavailable["result"]["reason"], "no_snapshot");
    }

    #[test]
    fn foreign_capture_refusal_preserves_selected_native_publication_and_source() {
        let mut selected_fixture = RegistryFixture::new();
        let (_, uri) = publish_export_error(&mut selected_fixture);
        let selected =
            std::sync::Arc::clone(selected_fixture.server.ranged_diagnostics.as_ref().unwrap());
        let selected_identity = selected.identity.clone();
        let selected_store = std::sync::Arc::clone(&selected.store);
        let selected_loaded = selected
            .association
            .captured()
            .native_sources()
            .unwrap()
            .iter()
            .find(|(_, _, input)| {
                matches!(
                    &input.origin,
                    mizar_session::SourceOriginInput::OpenBuffer { uri: source_uri, .. }
                        if source_uri == &uri
                )
            })
            .map(|(_, loaded, _)| loaded.text.clone())
            .unwrap();

        let mut foreign = RegistryFixture::new();
        let foreign_uri = foreign.sources[0].2.clone();
        foreign.did_change_text(&foreign_uri, 2, RAW_SOURCE.to_owned());
        foreign.driver = foreign.frontend_only_driver();
        let foreign_capture = foreign.capture();
        let foreign_input = foreign.driver_input();
        let (_, installed) = selected_fixture
            .server
            .submit_originating_diagnostic_publication(
                &mut foreign.driver,
                foreign_capture,
                &foreign.ids,
                &foreign.snapshots,
                foreign_input,
            )
            .expect("genuine foreign capture submission");
        assert!(
            !installed,
            "foreign association cannot select the publication"
        );
        let still_selected = selected_fixture.server.ranged_diagnostics.as_ref().unwrap();
        assert_eq!(still_selected.identity, selected_identity);
        assert!(std::sync::Arc::ptr_eq(still_selected, &selected));
        assert!(std::sync::Arc::ptr_eq(
            &still_selected.store,
            &selected_store
        ));
        let retained_source = still_selected
            .association
            .captured()
            .native_sources()
            .unwrap()
            .iter()
            .find(|(_, _, input)| {
                matches!(
                    &input.origin,
                    mizar_session::SourceOriginInput::OpenBuffer { uri: source_uri, .. }
                        if source_uri == &uri
                )
            })
            .map(|(_, loaded, _)| loaded.text.clone())
            .unwrap();
        assert_eq!(retained_source, selected_loaded);

        selected_fixture.did_change_text(
            &uri,
            3,
            "theorem Main: for X being set holds X = X proof let X be set; thus X = X; end;"
                .to_owned(),
        );
        let clean_capture = selected_fixture.capture();
        let clean_input = selected_fixture.driver_input();
        let (_, clean_selected) = selected_fixture
            .server
            .submit_originating_diagnostic_publication(
                &mut selected_fixture.driver,
                clean_capture,
                &selected_fixture.ids,
                &selected_fixture.snapshots,
                clean_input,
            )
            .expect("genuine clean native replacement");
        assert!(clean_selected, "eligible clean capture atomically selects");
        let clean = selected_fixture.server.ranged_diagnostics.as_ref().unwrap();
        assert_ne!(clean.identity, selected_identity);
        assert!(clean.index.is_empty());
        assert!(clean.store.is_empty());
        assert!(selected_fixture.server.deferred_fixes.is_empty());
        assert!(!std::sync::Arc::ptr_eq(&selected, clean));
        assert_eq!(
            selected
                .association
                .captured()
                .native_sources()
                .unwrap()
                .iter()
                .find(|(_, _, input)| {
                    matches!(
                        &input.origin,
                        mizar_session::SourceOriginInput::OpenBuffer { uri: source_uri, .. }
                            if source_uri == &uri
                    )
                })
                .map(|(_, loaded, _)| loaded.text.clone())
                .unwrap(),
            selected_loaded,
            "replaced publication keeps original source readable while retained"
        );
    }
}
