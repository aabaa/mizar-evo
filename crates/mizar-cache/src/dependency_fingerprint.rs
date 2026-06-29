//! Cache-side dependency footprint and fingerprint projection.
//!
//! The contract is specified in
//! [dependency_fingerprint.md](../../../doc/design/mizar-cache/en/dependency_fingerprint.md).
//! This module consumes producer-owned hashes and per-VC dependency-slice
//! fingerprints; it does not read cache records, schedule builds, publish
//! artifacts, or decide proof acceptance.

use std::{collections::BTreeMap, error::Error, fmt};

use mizar_artifact::{
    module_summary::{MODULE_SUMMARY_SCHEMA_FAMILY, ModuleSummary, ModuleSummaryIdentity},
    registration_summary::{REGISTRATION_SUMMARY_SCHEMA_FAMILY, RegistrationSummary},
};
use mizar_session::Hash;
use mizar_vc::dependency_slice::{
    DEPENDENCY_SLICE_SCHEMA_VERSION, DependencySlice, DependencySliceCompleteness,
};

use crate::cache_key::{
    CompatibilityField, DependencySliceHash, NamedHash, NamedSchemaVersion, PipelinePhase,
    SchemaVersion,
};

/// Current dependency-footprint schema version.
pub const DEPENDENCY_FINGERPRINT_SCHEMA_VERSION: &str =
    "mizar-cache/dependency-fingerprint-schema/v1";
/// Domain used for final dependency-footprint hashes.
pub const DEPENDENCY_FINGERPRINT_HASH_DOMAIN: &str = "mizar-cache/dependency-fingerprint/v1";

/// Stable owner for a dependency footprint.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FootprintOwner {
    /// Owner package id.
    pub package_id: String,
    /// Owner module path.
    pub module_path: String,
    /// Optional owner origin id for item-level footprints.
    pub origin_id: Option<String>,
    /// Optional language edition.
    pub language_edition: Option<String>,
    /// Optional lockfile identity.
    pub lockfile_identity: Option<String>,
}

/// Cache-side dependency target taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FingerprintTargetKind {
    /// Source content and language-edition dependency.
    Source,
    /// Lexical or parser dependency.
    LexicalParse,
    /// Published module interface dependency.
    ModuleInterface,
    /// Local module implementation dependency.
    ModuleImplementation,
    /// Published registration interface dependency.
    RegistrationInterface,
    /// Accepted registration or cluster trace dependency.
    ClusterTrace,
    /// Definition signature or unfolding-boundary dependency.
    Definition,
    /// Exported theorem-statement dependency.
    TheoremStatement,
    /// Local proof-body dependency.
    ProofBody,
    /// Per-VC dependency-slice dependency.
    VcSlice,
    /// Proof-reuse validation identity dependency.
    ProofReuseIdentity,
    /// Policy, backend profile, toolchain, or schema dependency.
    PolicyToolchain,
    /// Lockfile or manifest dependency.
    LockfileManifest,
}

/// Identity used to pair the same dependency target across builds.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FingerprintIdentity {
    /// Dependency package id.
    pub package_id: String,
    /// Dependency module path.
    pub module_path: String,
    /// Optional public origin id.
    pub origin_id: Option<String>,
    /// Stable target-local name.
    pub target_name: String,
    /// Producer-owned schema family.
    pub schema_family: String,
    /// Optional language edition.
    pub language_edition: Option<String>,
    /// Optional lockfile identity.
    pub lockfile_identity: Option<String>,
}

/// One cache-side dependency fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencyFingerprint {
    /// Target kind.
    pub target: FingerprintTargetKind,
    /// Target identity.
    pub identity: FingerprintIdentity,
    /// Domain of the producer-owned value hash.
    pub value_domain: String,
    /// Producer-owned value hash.
    pub value_hash: Hash,
    /// Producer or cache schema version that frames the value.
    pub schema_version: SchemaVersion,
    /// Whether the target participates in importer-visible semantics.
    pub importer_visible: bool,
}

/// Cache-side per-VC dependency-slice fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencySliceFingerprint {
    /// Slice class.
    pub slice_kind: String,
    /// Stable owner identity for this slice.
    pub owner: String,
    /// Slice-local stable name.
    pub name: String,
    /// Producer-owned hash domain.
    pub domain: String,
    /// Slice digest.
    pub digest: Hash,
    /// Completeness reported by the producer and projected fail-closed.
    pub completeness: DependencyFootprintCompleteness,
}

/// Unknown or incomplete dependency marker.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnknownDependencyMarker {
    /// Dependency family.
    pub family: String,
    /// Stable owner identity.
    pub owner: String,
    /// Fail-closed reason.
    pub reason: String,
}

/// Proof-reuse metadata consumed only as validation data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProofReuseValidationInput {
    /// Stable validation row name.
    pub name: String,
    /// Validation state produced by the proof owner.
    pub state: ProofReuseValidationState,
    /// Proof-reuse validation hash exported by proof metadata.
    pub validation_hash: Option<NamedHash>,
    /// Witness or deterministic-discharge hash used for validation.
    pub witness_or_discharge_hash: Option<NamedHash>,
    /// Proof metadata schema versions that affect interpretation.
    pub metadata_schema_versions: Vec<NamedSchemaVersion>,
}

/// State of proof-reuse validation data.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ProofReuseValidationState {
    /// All cache-side validation data matches.
    Complete,
    /// The current validation data differs from the cached candidate.
    Mismatched,
    /// Required validation data is absent.
    Missing,
    /// The only available data is external attestation.
    ExternalOnly,
    /// The evidence kind is unsupported by the cache-side validator.
    UnsupportedEvidenceKind(String),
}

/// Completeness state of a dependency footprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DependencyFootprintCompleteness {
    /// All required dependency information is available at the current granularity.
    Complete,
    /// Coverage is coarse but conservative.
    ConservativeComplete,
    /// The footprint is incomplete and must force a cache miss.
    IncompleteUncacheable,
}

/// Complete dependency footprint.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencyFootprint {
    /// Cache-side footprint schema version.
    pub schema_version: SchemaVersion,
    /// Footprint owner.
    pub owner: FootprintOwner,
    /// Pipeline phase.
    pub phase: PipelinePhase,
    /// Dependency target fingerprints.
    pub fingerprints: Vec<DependencyFingerprint>,
    /// Per-VC dependency-slice fingerprints.
    pub slices: Vec<DependencySliceFingerprint>,
    /// Compatibility fields affecting reuse eligibility.
    pub compatibility_fields: Vec<CompatibilityField>,
    /// Proof-reuse validation metadata.
    pub proof_reuse_validation: Vec<ProofReuseValidationInput>,
    /// Unknown markers that make the footprint miss.
    pub unknown_markers: Vec<UnknownDependencyMarker>,
    /// Footprint completeness.
    pub completeness: DependencyFootprintCompleteness,
    /// Explicit uncacheable marker.
    pub uncacheable: bool,
    /// Domain-separated final footprint hash.
    pub final_hash: Hash,
}

/// Request consumed by `DependencyFootprintBuilder`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyFootprintRequest {
    /// Cache-side footprint schema version.
    pub schema_version: SchemaVersion,
    /// Footprint owner.
    pub owner: FootprintOwner,
    /// Pipeline phase.
    pub phase: PipelinePhase,
    /// Dependency target fingerprints.
    pub fingerprints: Vec<DependencyFingerprint>,
    /// Per-VC dependency-slice fingerprints.
    pub slices: Vec<DependencySliceFingerprint>,
    /// Compatibility fields affecting reuse eligibility.
    pub compatibility_fields: Vec<CompatibilityField>,
    /// Proof-reuse validation metadata.
    pub proof_reuse_validation: Vec<ProofReuseValidationInput>,
    /// Unknown markers that make the footprint miss.
    pub unknown_markers: Vec<UnknownDependencyMarker>,
    /// Completeness requested by the caller before fail-closed projection.
    pub requested_completeness: DependencyFootprintCompleteness,
    /// Explicit uncacheable marker.
    pub uncacheable: bool,
}

/// Pure dependency-footprint builder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyFootprintBuilder {
    request: DependencyFootprintRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DuplicateConflict {
    collection: &'static str,
    key: String,
}

/// Result of dependency-footprint construction.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DependencyFootprintBuildOutcome {
    /// A reusable dependency footprint.
    Reusable(DependencyFootprint),
    /// A canonical footprint that must be treated as a miss.
    Uncacheable(DependencyFootprint),
    /// No canonical footprint can be produced.
    NoFootprint(DependencyFootprintBuildRejection),
}

/// Reasons footprint construction can reject a request.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DependencyFootprintBuildRejection {
    /// The request used an unsupported dependency-footprint schema.
    UnsupportedSchema {
        /// Actual schema.
        actual: SchemaVersion,
    },
    /// A canonical collection key appeared with conflicting payloads.
    ConflictingDuplicate {
        /// Collection name.
        collection: &'static str,
        /// Canonical key display.
        key: String,
    },
    /// A required identity field was empty.
    MissingRequiredIdentity {
        /// Field name.
        field: &'static str,
    },
    /// The request carried no dependency data.
    EmptyDependencyFootprint,
}

/// Rebuild trigger classification. Task 6 adds evaluator fixtures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RebuildTrigger {
    /// The cached output may remain a reuse candidate.
    ReuseAllowed,
    /// Rebuild the phase owning the changed footprint.
    RebuildPhase,
    /// Rebuild dependent phases that can see the changed target.
    RebuildDependents,
    /// Miss because the footprint is incomplete or incompatible.
    UncacheableMiss,
}

/// Fingerprint change class consumed by rebuild-trigger evaluation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FingerprintChangeKind {
    /// Comment-only source text changed outside semantic fingerprints.
    CommentOnly,
    /// Diagnostic wording or explanation text changed outside semantic output.
    DiagnosticWordingOnly,
    /// Backend runtime, task order, or cache timing changed.
    RuntimeObservationOnly,
    /// Source content changed token or AST shape.
    SourceTokenAst,
    /// Published module `interface_hash` changed.
    ModuleInterface,
    /// Local implementation or artifact hash changed without interface change.
    ModuleImplementationOnly,
    /// Published registration interface changed.
    RegistrationInterface,
    /// Accepted registration, cluster, reduction, or visible trace origin changed.
    ClusterReductionVisibleOrigin,
    /// Local proof body changed without theorem-statement or accepted-status change.
    ProofBodyOnly,
    /// Exported theorem, definition, mode, attribute, notation, cluster, or algorithm contract changed.
    ExportedSemantic,
    /// Verifier or proof policy changed.
    Policy,
    /// Toolchain compatibility changed.
    Toolchain,
    /// Schema version changed.
    SchemaVersion,
    /// Lockfile identity changed.
    Lockfile,
    /// Manifest identity changed.
    Manifest,
    /// Dependency footprint is incomplete.
    IncompleteFootprint,
    /// Schema compatibility is unknown.
    UnknownSchema,
    /// Toolchain compatibility is unknown.
    UnknownToolchain,
    /// The footprint carries an explicit uncacheable marker.
    UncacheableMarker,
    /// Proof-reuse validation data is missing.
    MissingProofReuseValidation,
}

/// Dependency-slice precision available to the trigger evaluator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DependencySlicePrecision {
    /// Producer slices are precise enough to identify the dependent footprint.
    Exact,
    /// Producer slices are conservative and may over-trigger.
    ConservativeCoarse,
}

/// Input row for rebuild-trigger evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RebuildTriggerInput {
    /// Change classification.
    pub change_kind: FingerprintChangeKind,
    /// Fingerprint target that changed.
    pub target: FingerprintTargetKind,
    /// Dependent phase being evaluated.
    pub dependent_phase: PipelinePhase,
    /// Slice precision for this dependency edge.
    pub slice_precision: DependencySlicePrecision,
}

/// Rebuild-trigger decision.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RebuildTriggerDecision {
    /// Trigger result.
    pub trigger: RebuildTrigger,
    /// Dependent phase evaluated by this decision.
    pub dependent_phase: PipelinePhase,
    /// Whether the decision intentionally over-triggers because slices are coarse.
    pub conservative: bool,
    /// Stable diagnostic reason.
    pub reason: &'static str,
}

/// Combined rebuild-trigger result for a set of rows.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RebuildTriggerSummary {
    /// Strongest trigger after precedence is applied.
    pub trigger: RebuildTrigger,
    /// Whether any strongest row intentionally over-triggers because slices are coarse.
    pub conservative: bool,
    /// Number of rows considered.
    pub row_count: usize,
}

/// Pure rebuild-trigger evaluator.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct RebuildTriggerEvaluator;

impl RebuildTriggerEvaluator {
    /// Evaluates the trigger for one fingerprint change and dependent phase.
    pub fn evaluate(input: RebuildTriggerInput) -> RebuildTriggerDecision {
        let trigger = trigger_for_change(&input.change_kind);
        let conservative = input.slice_precision == DependencySlicePrecision::ConservativeCoarse
            && trigger == RebuildTrigger::RebuildDependents;
        RebuildTriggerDecision {
            reason: reason_for_change(&input.change_kind),
            dependent_phase: input.dependent_phase,
            trigger,
            conservative,
        }
    }

    /// Evaluates rows for one cache-dependency decision and applies trigger precedence.
    pub fn evaluate_all(
        inputs: impl IntoIterator<Item = RebuildTriggerInput>,
    ) -> RebuildTriggerSummary {
        let mut strongest = RebuildTrigger::ReuseAllowed;
        let mut conservative = false;
        let mut row_count = 0;

        for input in inputs {
            row_count += 1;
            let decision = Self::evaluate(input);
            let precedence = trigger_precedence(decision.trigger);
            let strongest_precedence = trigger_precedence(strongest);
            if precedence > strongest_precedence {
                strongest = decision.trigger;
                conservative = decision.conservative;
            } else if precedence == strongest_precedence {
                conservative |= decision.conservative;
            }
        }

        RebuildTriggerSummary {
            trigger: strongest,
            conservative,
            row_count,
        }
    }
}

impl FootprintOwner {
    /// Builds a module-level owner from an artifact module identity.
    pub fn from_module_identity(module: &ModuleSummaryIdentity) -> Self {
        Self {
            package_id: module.package_id.clone(),
            module_path: module.module_path.clone(),
            origin_id: None,
            language_edition: Some(module.language_edition.clone()),
            lockfile_identity: module.lockfile_identity.clone(),
        }
    }
}

impl FingerprintTargetKind {
    /// Returns the stable target-kind string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::LexicalParse => "lexical_parse",
            Self::ModuleInterface => "module_interface",
            Self::ModuleImplementation => "module_implementation",
            Self::RegistrationInterface => "registration_interface",
            Self::ClusterTrace => "cluster_trace",
            Self::Definition => "definition",
            Self::TheoremStatement => "theorem_statement",
            Self::ProofBody => "proof_body",
            Self::VcSlice => "vc_slice",
            Self::ProofReuseIdentity => "proof_reuse_identity",
            Self::PolicyToolchain => "policy_toolchain",
            Self::LockfileManifest => "lockfile_manifest",
        }
    }
}

impl FingerprintIdentity {
    /// Builds an identity from a published module identity.
    pub fn from_module(
        module: &ModuleSummaryIdentity,
        origin_id: Option<impl Into<String>>,
        target_name: impl Into<String>,
        schema_family: impl Into<String>,
    ) -> Self {
        Self {
            package_id: module.package_id.clone(),
            module_path: module.module_path.clone(),
            origin_id: origin_id.map(Into::into),
            target_name: target_name.into(),
            schema_family: schema_family.into(),
            language_edition: Some(module.language_edition.clone()),
            lockfile_identity: module.lockfile_identity.clone(),
        }
    }
}

impl DependencyFingerprint {
    /// Builds the importer-visible module-interface fingerprint from a summary.
    pub fn from_module_summary(summary: &ModuleSummary) -> Self {
        Self {
            target: FingerprintTargetKind::ModuleInterface,
            identity: FingerprintIdentity::from_module(
                &summary.module,
                None::<String>,
                "module-interface",
                MODULE_SUMMARY_SCHEMA_FAMILY,
            ),
            value_domain: MODULE_SUMMARY_SCHEMA_FAMILY.to_owned(),
            value_hash: summary.interface_hash,
            schema_version: SchemaVersion::new(summary.schema_version.to_string()),
            importer_visible: true,
        }
    }

    /// Builds the importer-visible registration-interface fingerprint.
    pub fn from_registration_summary(summary: &RegistrationSummary) -> Self {
        Self {
            target: FingerprintTargetKind::RegistrationInterface,
            identity: FingerprintIdentity::from_module(
                &summary.module,
                None::<String>,
                "registration-interface",
                REGISTRATION_SUMMARY_SCHEMA_FAMILY,
            ),
            value_domain: REGISTRATION_SUMMARY_SCHEMA_FAMILY.to_owned(),
            value_hash: summary.registration_interface_hash,
            schema_version: SchemaVersion::new(summary.schema_version.to_string()),
            importer_visible: true,
        }
    }

    /// Builds a local implementation fingerprint. This is not importer-visible.
    pub fn module_implementation(
        identity: FingerprintIdentity,
        value_domain: impl Into<String>,
        value_hash: Hash,
        schema_version: SchemaVersion,
    ) -> Self {
        Self {
            target: FingerprintTargetKind::ModuleImplementation,
            identity,
            value_domain: value_domain.into(),
            value_hash,
            schema_version,
            importer_visible: false,
        }
    }
}

impl DependencySliceFingerprint {
    /// Projects a `mizar-vc` dependency slice into a cache-side slice hash.
    ///
    /// The caller supplies stable semantic identity fields. Raw `VcId` values
    /// from the producer slice are intentionally not part of this API.
    pub fn from_vc_slice(
        slice_kind: impl Into<String>,
        owner: impl Into<String>,
        name: impl Into<String>,
        slice: &DependencySlice,
    ) -> Self {
        Self {
            slice_kind: slice_kind.into(),
            owner: owner.into(),
            name: name.into(),
            domain: DEPENDENCY_SLICE_SCHEMA_VERSION.to_owned(),
            digest: slice.fingerprint().hash(),
            completeness: match slice.completeness() {
                DependencySliceCompleteness::Complete => DependencyFootprintCompleteness::Complete,
                DependencySliceCompleteness::IncompleteUncacheable => {
                    DependencyFootprintCompleteness::IncompleteUncacheable
                }
                _ => DependencyFootprintCompleteness::IncompleteUncacheable,
            },
        }
    }

    /// Converts the slice fingerprint to the cache-key validation input shape.
    pub fn as_cache_key_slice(&self) -> DependencySliceHash {
        DependencySliceHash {
            slice_kind: self.slice_kind.clone(),
            owner: self.owner.clone(),
            name: self.name.clone(),
            domain: self.domain.clone(),
            digest: self.digest,
        }
    }
}

impl DependencyFootprintBuilder {
    /// Creates a dependency-footprint builder from a complete request.
    pub fn new(request: DependencyFootprintRequest) -> Self {
        Self { request }
    }

    /// Builds a dependency-footprint outcome.
    pub fn build(self) -> DependencyFootprintBuildOutcome {
        let mut request = self.request;
        if request.schema_version.as_str() != DEPENDENCY_FINGERPRINT_SCHEMA_VERSION {
            return DependencyFootprintBuildOutcome::NoFootprint(
                DependencyFootprintBuildRejection::UnsupportedSchema {
                    actual: request.schema_version,
                },
            );
        }

        if let Err(rejection) = validate_request(&request) {
            return DependencyFootprintBuildOutcome::NoFootprint(rejection);
        }

        append_fail_closed_markers_for_missing_families(&mut request);
        let conflicts = canonicalize_request(&mut request);
        if !conflicts.is_empty() {
            request
                .unknown_markers
                .extend(conflicts.into_iter().map(UnknownDependencyMarker::from));
            let _ = canonicalize_by_keys(
                &mut request.unknown_markers,
                "unknown_markers",
                UnknownDependencyMarker::duplicate_identity_key,
                UnknownDependencyMarker::canonical_sort_key,
            );
        }

        let completeness = projected_completeness(&request);
        let uncacheable = request.uncacheable
            || completeness == DependencyFootprintCompleteness::IncompleteUncacheable;
        let final_hash = final_hash_for_request(&request, completeness, uncacheable);
        let footprint = DependencyFootprint {
            schema_version: request.schema_version,
            owner: request.owner,
            phase: request.phase,
            fingerprints: request.fingerprints,
            slices: request.slices,
            compatibility_fields: request.compatibility_fields,
            proof_reuse_validation: request.proof_reuse_validation,
            unknown_markers: request.unknown_markers,
            completeness,
            uncacheable,
            final_hash,
        };

        if uncacheable {
            DependencyFootprintBuildOutcome::Uncacheable(footprint)
        } else {
            DependencyFootprintBuildOutcome::Reusable(footprint)
        }
    }
}

impl fmt::Display for DependencyFootprintBuildRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchema { actual } => {
                write!(
                    f,
                    "unsupported dependency footprint schema `{}`",
                    actual.as_str()
                )
            }
            Self::ConflictingDuplicate { collection, key } => {
                write!(f, "conflicting duplicate in {collection} for `{key}`")
            }
            Self::MissingRequiredIdentity { field } => {
                write!(
                    f,
                    "missing required dependency footprint identity field `{field}`"
                )
            }
            Self::EmptyDependencyFootprint => f.write_str("empty dependency footprint"),
        }
    }
}

impl Error for DependencyFootprintBuildRejection {}

fn trigger_precedence(trigger: RebuildTrigger) -> u8 {
    match trigger {
        RebuildTrigger::ReuseAllowed => 0,
        RebuildTrigger::RebuildPhase => 1,
        RebuildTrigger::RebuildDependents => 2,
        RebuildTrigger::UncacheableMiss => 3,
    }
}

fn trigger_for_change(change: &FingerprintChangeKind) -> RebuildTrigger {
    match change {
        FingerprintChangeKind::CommentOnly
        | FingerprintChangeKind::DiagnosticWordingOnly
        | FingerprintChangeKind::RuntimeObservationOnly => RebuildTrigger::ReuseAllowed,
        FingerprintChangeKind::SourceTokenAst
        | FingerprintChangeKind::ModuleInterface
        | FingerprintChangeKind::RegistrationInterface
        | FingerprintChangeKind::ClusterReductionVisibleOrigin
        | FingerprintChangeKind::ExportedSemantic => RebuildTrigger::RebuildDependents,
        FingerprintChangeKind::ModuleImplementationOnly
        | FingerprintChangeKind::ProofBodyOnly
        | FingerprintChangeKind::Policy
        | FingerprintChangeKind::Toolchain
        | FingerprintChangeKind::SchemaVersion
        | FingerprintChangeKind::Lockfile
        | FingerprintChangeKind::Manifest => RebuildTrigger::RebuildPhase,
        FingerprintChangeKind::IncompleteFootprint
        | FingerprintChangeKind::UnknownSchema
        | FingerprintChangeKind::UnknownToolchain
        | FingerprintChangeKind::UncacheableMarker
        | FingerprintChangeKind::MissingProofReuseValidation => RebuildTrigger::UncacheableMiss,
    }
}

fn reason_for_change(change: &FingerprintChangeKind) -> &'static str {
    match change {
        FingerprintChangeKind::CommentOnly => "comment-only change excluded from fingerprints",
        FingerprintChangeKind::DiagnosticWordingOnly => {
            "diagnostic wording change excluded from semantic output"
        }
        FingerprintChangeKind::RuntimeObservationOnly => {
            "runtime, task-order, or cache timing observation changed"
        }
        FingerprintChangeKind::SourceTokenAst => "source token or AST shape changed",
        FingerprintChangeKind::ModuleInterface => "module interface hash changed",
        FingerprintChangeKind::ModuleImplementationOnly => {
            "local implementation changed without interface change"
        }
        FingerprintChangeKind::RegistrationInterface => "registration interface changed",
        FingerprintChangeKind::ClusterReductionVisibleOrigin => {
            "accepted registration, cluster, reduction, or trace origin changed"
        }
        FingerprintChangeKind::ProofBodyOnly => {
            "local proof body changed without exported proof boundary change"
        }
        FingerprintChangeKind::ExportedSemantic => "exported semantic dependency changed",
        FingerprintChangeKind::Policy => "verifier or proof policy changed",
        FingerprintChangeKind::Toolchain => "toolchain compatibility changed",
        FingerprintChangeKind::SchemaVersion => "schema version changed",
        FingerprintChangeKind::Lockfile => "lockfile identity changed",
        FingerprintChangeKind::Manifest => "manifest identity changed",
        FingerprintChangeKind::IncompleteFootprint => "dependency footprint is incomplete",
        FingerprintChangeKind::UnknownSchema => "schema compatibility is unknown",
        FingerprintChangeKind::UnknownToolchain => "toolchain compatibility is unknown",
        FingerprintChangeKind::UncacheableMarker => "uncacheable marker is present",
        FingerprintChangeKind::MissingProofReuseValidation => {
            "proof-reuse validation input is missing"
        }
    }
}

fn validate_request(
    request: &DependencyFootprintRequest,
) -> Result<(), DependencyFootprintBuildRejection> {
    reject_empty("owner.package_id", &request.owner.package_id)?;
    reject_empty("owner.module_path", &request.owner.module_path)?;
    reject_empty("phase", request.phase.as_str())?;

    if request.fingerprints.is_empty()
        && request.slices.is_empty()
        && request.unknown_markers.is_empty()
        && request.proof_reuse_validation.is_empty()
    {
        return Err(DependencyFootprintBuildRejection::EmptyDependencyFootprint);
    }

    for value in &request.fingerprints {
        reject_empty(
            "fingerprints.identity.package_id",
            &value.identity.package_id,
        )?;
        reject_empty(
            "fingerprints.identity.module_path",
            &value.identity.module_path,
        )?;
        reject_empty(
            "fingerprints.identity.target_name",
            &value.identity.target_name,
        )?;
        reject_empty(
            "fingerprints.identity.schema_family",
            &value.identity.schema_family,
        )?;
        reject_empty("fingerprints.value_domain", &value.value_domain)?;
        reject_empty("fingerprints.schema_version", value.schema_version.as_str())?;
    }
    for value in &request.slices {
        reject_empty("slices.slice_kind", &value.slice_kind)?;
        reject_empty("slices.owner", &value.owner)?;
        reject_empty("slices.name", &value.name)?;
        reject_empty("slices.domain", &value.domain)?;
    }
    for value in &request.compatibility_fields {
        reject_empty("compatibility_fields.family", &value.family)?;
        reject_empty("compatibility_fields.field_name", &value.field_name)?;
    }
    for value in &request.proof_reuse_validation {
        reject_empty("proof_reuse_validation.name", &value.name)?;
        if let Some(hash) = &value.validation_hash {
            reject_empty("proof_reuse_validation.validation_hash.name", &hash.name)?;
            reject_empty(
                "proof_reuse_validation.validation_hash.domain",
                &hash.domain,
            )?;
        }
        if let Some(hash) = &value.witness_or_discharge_hash {
            reject_empty(
                "proof_reuse_validation.witness_or_discharge_hash.name",
                &hash.name,
            )?;
            reject_empty(
                "proof_reuse_validation.witness_or_discharge_hash.domain",
                &hash.domain,
            )?;
        }
        for version in &value.metadata_schema_versions {
            reject_empty(
                "proof_reuse_validation.metadata_schema_versions.schema_family",
                &version.schema_family,
            )?;
            reject_empty(
                "proof_reuse_validation.metadata_schema_versions.name",
                &version.name,
            )?;
            reject_empty(
                "proof_reuse_validation.metadata_schema_versions.version",
                version.version.as_str(),
            )?;
        }
    }
    for value in &request.unknown_markers {
        reject_empty("unknown_markers.family", &value.family)?;
        reject_empty("unknown_markers.owner", &value.owner)?;
        reject_empty("unknown_markers.reason", &value.reason)?;
    }
    Ok(())
}

fn append_fail_closed_markers_for_missing_families(request: &mut DependencyFootprintRequest) {
    if request.compatibility_fields.is_empty() {
        request.unknown_markers.push(UnknownDependencyMarker {
            family: "compatibility".to_owned(),
            owner: owner_marker(&request.owner),
            reason: "missing toolchain or schema compatibility field".to_owned(),
        });
    }
    for field in &request.compatibility_fields {
        if compatibility_value_is_unknown(&field.value) {
            request.unknown_markers.push(UnknownDependencyMarker {
                family: format!("compatibility:{}", field.family),
                owner: owner_marker(&request.owner),
                reason: format!("unknown compatibility value for {}", field.field_name),
            });
        }
    }
    if is_vc_or_proof_phase(request.phase.as_str()) {
        if request.slices.is_empty() {
            request.unknown_markers.push(UnknownDependencyMarker {
                family: "vc_slice".to_owned(),
                owner: owner_marker(&request.owner),
                reason: "missing per-VC dependency slice fingerprint".to_owned(),
            });
        }
        if request.proof_reuse_validation.is_empty() {
            request.unknown_markers.push(UnknownDependencyMarker {
                family: "proof_reuse_identity".to_owned(),
                owner: owner_marker(&request.owner),
                reason: "missing proof-reuse validation metadata".to_owned(),
            });
        }
    }
}

fn compatibility_value_is_unknown(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "" | "unknown" | "unsupported" | "incompatible" | "missing" | "opaque"
    )
}

fn is_vc_or_proof_phase(phase: &str) -> bool {
    matches!(
        phase,
        "vc" | "proof" | "atp" | "proof_reuse" | "proof-reuse" | "kernel_handoff"
    )
}

fn owner_marker(owner: &FootprintOwner) -> String {
    owner.origin_id.as_ref().map_or_else(
        || owner.module_path.clone(),
        |origin| format!("{}#{origin}", owner.module_path),
    )
}

fn reject_empty(field: &'static str, value: &str) -> Result<(), DependencyFootprintBuildRejection> {
    if value.trim().is_empty() {
        return Err(DependencyFootprintBuildRejection::MissingRequiredIdentity { field });
    }
    Ok(())
}

fn canonicalize_request(request: &mut DependencyFootprintRequest) -> Vec<DuplicateConflict> {
    let mut conflicts = Vec::new();
    conflicts.extend(canonicalize_by_keys(
        &mut request.fingerprints,
        "fingerprints",
        DependencyFingerprint::duplicate_identity_key,
        DependencyFingerprint::canonical_sort_key,
    ));
    conflicts.extend(canonicalize_by_keys(
        &mut request.slices,
        "slices",
        DependencySliceFingerprint::duplicate_identity_key,
        DependencySliceFingerprint::canonical_sort_key,
    ));
    conflicts.extend(canonicalize_by_keys(
        &mut request.compatibility_fields,
        "compatibility_fields",
        compatibility_field_duplicate_identity_key,
        compatibility_field_canonical_sort_key,
    ));
    for value in &mut request.proof_reuse_validation {
        conflicts.extend(canonicalize_by_keys(
            &mut value.metadata_schema_versions,
            "proof_reuse_validation.metadata_schema_versions",
            named_schema_version_duplicate_identity_key,
            named_schema_version_canonical_sort_key,
        ));
    }
    conflicts.extend(canonicalize_by_keys(
        &mut request.proof_reuse_validation,
        "proof_reuse_validation",
        ProofReuseValidationInput::duplicate_identity_key,
        ProofReuseValidationInput::canonical_sort_key,
    ));
    conflicts.extend(canonicalize_by_keys(
        &mut request.unknown_markers,
        "unknown_markers",
        UnknownDependencyMarker::duplicate_identity_key,
        UnknownDependencyMarker::canonical_sort_key,
    ));
    conflicts
}

fn canonicalize_by_keys<T, I, S>(
    values: &mut Vec<T>,
    collection: &'static str,
    identity_key_fn: impl Fn(&T) -> I,
    sort_key_fn: impl Fn(&T) -> S,
) -> Vec<DuplicateConflict>
where
    T: Clone + Eq,
    I: Ord + fmt::Debug,
    S: Ord,
{
    let mut by_key = BTreeMap::<I, Vec<T>>::new();
    let mut conflicts = Vec::new();
    for value in values.drain(..) {
        let key = identity_key_fn(&value);
        match by_key.get_mut(&key) {
            Some(existing) if existing.iter().any(|existing| existing == &value) => {}
            Some(existing) => {
                conflicts.push(DuplicateConflict {
                    collection,
                    key: format!("{key:?}"),
                });
                existing.push(value);
            }
            None => {
                by_key.insert(key, vec![value]);
            }
        }
    }
    values.extend(by_key.into_values().flatten());
    values.sort_by_key(sort_key_fn);
    conflicts
}

fn projected_completeness(request: &DependencyFootprintRequest) -> DependencyFootprintCompleteness {
    if request.requested_completeness == DependencyFootprintCompleteness::IncompleteUncacheable
        || request.uncacheable
        || !request.unknown_markers.is_empty()
        || request.slices.iter().any(|slice| {
            slice.completeness == DependencyFootprintCompleteness::IncompleteUncacheable
        })
        || request
            .proof_reuse_validation
            .iter()
            .any(proof_reuse_requires_miss)
    {
        DependencyFootprintCompleteness::IncompleteUncacheable
    } else {
        request.requested_completeness
    }
}

fn proof_reuse_requires_miss(value: &ProofReuseValidationInput) -> bool {
    !matches!(value.state, ProofReuseValidationState::Complete)
        || value.validation_hash.is_none()
        || value.witness_or_discharge_hash.is_none()
        || value.metadata_schema_versions.is_empty()
}

fn final_hash_for_request(
    request: &DependencyFootprintRequest,
    completeness: DependencyFootprintCompleteness,
    uncacheable: bool,
) -> Hash {
    let mut hasher = stable_hasher("final");
    write_schema_version_field(&mut hasher, "schema_version", &request.schema_version);
    write_owner(&mut hasher, &request.owner);
    write_str_field(&mut hasher, "phase", request.phase.as_str());
    write_fingerprints(&mut hasher, "fingerprints", &request.fingerprints);
    write_slices(&mut hasher, "slices", &request.slices);
    write_compatibility_fields(
        &mut hasher,
        "compatibility_fields",
        &request.compatibility_fields,
    );
    write_proof_reuse_validation(
        &mut hasher,
        "proof_reuse_validation",
        &request.proof_reuse_validation,
    );
    write_unknown_markers(&mut hasher, "unknown_markers", &request.unknown_markers);
    write_completeness(&mut hasher, "completeness", completeness);
    write_field_tag(&mut hasher, "uncacheable");
    hasher.update(&[u8::from(uncacheable)]);
    finish_hash(hasher)
}

fn stable_hasher(label: &str) -> blake3::Hasher {
    let mut hasher = blake3::Hasher::new();
    write_str(&mut hasher, DEPENDENCY_FINGERPRINT_HASH_DOMAIN);
    write_str(&mut hasher, label);
    hasher
}

fn write_owner(hasher: &mut blake3::Hasher, value: &FootprintOwner) {
    write_field_tag(hasher, "FootprintOwner");
    write_str_field(hasher, "package_id", &value.package_id);
    write_str_field(hasher, "module_path", &value.module_path);
    write_optional_str(hasher, "origin_id", value.origin_id.as_deref());
    write_optional_str(
        hasher,
        "language_edition",
        value.language_edition.as_deref(),
    );
    write_optional_str(
        hasher,
        "lockfile_identity",
        value.lockfile_identity.as_deref(),
    );
}

fn write_fingerprints(hasher: &mut blake3::Hasher, field: &str, values: &[DependencyFingerprint]) {
    write_field_tag(hasher, field);
    write_len(hasher, values.len());
    for value in values {
        write_field_tag(hasher, "DependencyFingerprint");
        write_str_field(hasher, "target", value.target.as_str());
        write_identity(hasher, &value.identity);
        write_str_field(hasher, "value_domain", &value.value_domain);
        write_hash_field(hasher, "value_hash", &value.value_domain, value.value_hash);
        write_schema_version_field(hasher, "schema_version", &value.schema_version);
        write_field_tag(hasher, "importer_visible");
        hasher.update(&[u8::from(value.importer_visible)]);
    }
}

fn write_identity(hasher: &mut blake3::Hasher, value: &FingerprintIdentity) {
    write_field_tag(hasher, "FingerprintIdentity");
    write_str_field(hasher, "package_id", &value.package_id);
    write_str_field(hasher, "module_path", &value.module_path);
    write_optional_str(hasher, "origin_id", value.origin_id.as_deref());
    write_str_field(hasher, "target_name", &value.target_name);
    write_str_field(hasher, "schema_family", &value.schema_family);
    write_optional_str(
        hasher,
        "language_edition",
        value.language_edition.as_deref(),
    );
    write_optional_str(
        hasher,
        "lockfile_identity",
        value.lockfile_identity.as_deref(),
    );
}

fn write_slices(hasher: &mut blake3::Hasher, field: &str, values: &[DependencySliceFingerprint]) {
    write_field_tag(hasher, field);
    write_len(hasher, values.len());
    for value in values {
        write_field_tag(hasher, "DependencySliceFingerprint");
        write_str_field(hasher, "slice_kind", &value.slice_kind);
        write_str_field(hasher, "owner", &value.owner);
        write_str_field(hasher, "name", &value.name);
        write_str_field(hasher, "domain", &value.domain);
        write_hash_field(hasher, "digest", &value.domain, value.digest);
        write_completeness(hasher, "completeness", value.completeness);
    }
}

fn write_compatibility_fields(
    hasher: &mut blake3::Hasher,
    field: &str,
    values: &[CompatibilityField],
) {
    write_field_tag(hasher, field);
    write_len(hasher, values.len());
    for value in values {
        write_field_tag(hasher, "CompatibilityField");
        write_str_field(hasher, "family", &value.family);
        write_str_field(hasher, "field_name", &value.field_name);
        write_str_field(hasher, "value", &value.value);
    }
}

fn write_proof_reuse_validation(
    hasher: &mut blake3::Hasher,
    field: &str,
    values: &[ProofReuseValidationInput],
) {
    write_field_tag(hasher, field);
    write_len(hasher, values.len());
    for value in values {
        write_field_tag(hasher, "ProofReuseValidationInput");
        write_str_field(hasher, "name", &value.name);
        write_proof_reuse_state(hasher, &value.state);
        write_optional_named_hash(hasher, "validation_hash", value.validation_hash.as_ref());
        write_optional_named_hash(
            hasher,
            "witness_or_discharge_hash",
            value.witness_or_discharge_hash.as_ref(),
        );
        write_schema_versions(
            hasher,
            "metadata_schema_versions",
            &value.metadata_schema_versions,
        );
    }
}

fn write_proof_reuse_state(hasher: &mut blake3::Hasher, value: &ProofReuseValidationState) {
    write_field_tag(hasher, "ProofReuseValidationState");
    match value {
        ProofReuseValidationState::Complete => write_str(hasher, "complete"),
        ProofReuseValidationState::Mismatched => write_str(hasher, "mismatched"),
        ProofReuseValidationState::Missing => write_str(hasher, "missing"),
        ProofReuseValidationState::ExternalOnly => write_str(hasher, "external_only"),
        ProofReuseValidationState::UnsupportedEvidenceKind(kind) => {
            write_str(hasher, "unsupported_evidence_kind");
            write_str(hasher, kind);
        }
    }
}

fn write_unknown_markers(
    hasher: &mut blake3::Hasher,
    field: &str,
    values: &[UnknownDependencyMarker],
) {
    write_field_tag(hasher, field);
    write_len(hasher, values.len());
    for value in values {
        write_field_tag(hasher, "UnknownDependencyMarker");
        write_str_field(hasher, "family", &value.family);
        write_str_field(hasher, "owner", &value.owner);
        write_str_field(hasher, "reason", &value.reason);
    }
}

fn write_completeness(
    hasher: &mut blake3::Hasher,
    field: &str,
    value: DependencyFootprintCompleteness,
) {
    write_str_field(
        hasher,
        field,
        match value {
            DependencyFootprintCompleteness::Complete => "complete",
            DependencyFootprintCompleteness::ConservativeComplete => "conservative_complete",
            DependencyFootprintCompleteness::IncompleteUncacheable => "incomplete_uncacheable",
        },
    );
}

fn write_optional_named_hash(hasher: &mut blake3::Hasher, field: &str, value: Option<&NamedHash>) {
    write_field_tag(hasher, field);
    match value {
        Some(value) => {
            hasher.update(&[1]);
            write_named_hash(hasher, value);
        }
        None => {
            hasher.update(&[0]);
        }
    }
}

fn write_named_hash(hasher: &mut blake3::Hasher, value: &NamedHash) {
    write_field_tag(hasher, "NamedHash");
    write_str_field(hasher, "name", &value.name);
    write_str_field(hasher, "domain", &value.domain);
    write_hash_field(hasher, "digest", &value.domain, value.digest);
}

fn write_schema_versions(hasher: &mut blake3::Hasher, field: &str, values: &[NamedSchemaVersion]) {
    write_field_tag(hasher, field);
    write_len(hasher, values.len());
    for value in values {
        write_field_tag(hasher, "NamedSchemaVersion");
        write_str_field(hasher, "schema_family", &value.schema_family);
        write_str_field(hasher, "name", &value.name);
        write_schema_version_field(hasher, "version", &value.version);
    }
}

fn write_schema_version_field(hasher: &mut blake3::Hasher, field: &str, value: &SchemaVersion) {
    write_str_field(hasher, field, value.as_str());
}

fn write_optional_str(hasher: &mut blake3::Hasher, field: &str, value: Option<&str>) {
    write_field_tag(hasher, field);
    match value {
        Some(value) => {
            hasher.update(&[1]);
            write_str(hasher, value);
        }
        None => {
            hasher.update(&[0]);
        }
    }
}

fn write_str_field(hasher: &mut blake3::Hasher, field: &str, value: &str) {
    write_field_tag(hasher, field);
    write_str(hasher, value);
}

fn write_hash_field(hasher: &mut blake3::Hasher, field: &str, domain: &str, value: Hash) {
    write_field_tag(hasher, field);
    write_hash(hasher, domain, value);
}

fn write_field_tag(hasher: &mut blake3::Hasher, field: &str) {
    write_str(hasher, field);
}

fn write_str(hasher: &mut blake3::Hasher, value: &str) {
    write_len(hasher, value.len());
    hasher.update(value.as_bytes());
}

fn write_hash(hasher: &mut blake3::Hasher, domain: &str, value: Hash) {
    write_str(hasher, domain);
    hasher.update(value.as_bytes());
}

fn write_len(hasher: &mut blake3::Hasher, value: usize) {
    hasher.update(&(value as u64).to_le_bytes());
}

fn finish_hash(hasher: blake3::Hasher) -> Hash {
    Hash::from_bytes(*hasher.finalize().as_bytes())
}

impl DependencyFingerprint {
    fn duplicate_identity_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.target.as_str().as_bytes(),
            self.identity.package_id.as_bytes(),
            self.identity.module_path.as_bytes(),
            optional_str_bytes(self.identity.origin_id.as_deref()).as_slice(),
            self.identity.target_name.as_bytes(),
            self.identity.schema_family.as_bytes(),
        ])
    }

    fn canonical_sort_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.target.as_str().as_bytes(),
            self.identity.package_id.as_bytes(),
            self.identity.module_path.as_bytes(),
            optional_str_bytes(self.identity.origin_id.as_deref()).as_slice(),
            self.identity.target_name.as_bytes(),
            self.identity.schema_family.as_bytes(),
            optional_str_bytes(self.identity.language_edition.as_deref()).as_slice(),
            optional_str_bytes(self.identity.lockfile_identity.as_deref()).as_slice(),
            self.value_domain.as_bytes(),
            self.value_hash.as_bytes(),
            self.schema_version.as_str().as_bytes(),
            bool_sort_bytes(self.importer_visible),
        ])
    }
}

impl DependencySliceFingerprint {
    fn duplicate_identity_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.slice_kind.as_bytes(),
            self.owner.as_bytes(),
            self.name.as_bytes(),
            self.domain.as_bytes(),
        ])
    }

    fn canonical_sort_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.slice_kind.as_bytes(),
            self.owner.as_bytes(),
            self.name.as_bytes(),
            self.domain.as_bytes(),
            self.digest.as_bytes(),
            completeness_sort_bytes(self.completeness),
        ])
    }
}

impl ProofReuseValidationInput {
    fn duplicate_identity_key(&self) -> Vec<u8> {
        canonical_key_parts([self.name.as_bytes()])
    }

    fn canonical_sort_key(&self) -> Vec<u8> {
        let mut key = canonical_key_parts([self.name.as_bytes()]);
        append_key_part(&mut key, b"state");
        append_key_part(&mut key, &state_sort_key(&self.state));
        append_optional_named_hash_sort_key(&mut key, b"validation_hash", &self.validation_hash);
        append_optional_named_hash_sort_key(
            &mut key,
            b"witness_or_discharge_hash",
            &self.witness_or_discharge_hash,
        );
        for version in &self.metadata_schema_versions {
            append_key_part(&mut key, b"metadata_schema_version");
            append_key_part(&mut key, version.schema_family.as_bytes());
            append_key_part(&mut key, version.name.as_bytes());
            append_key_part(&mut key, version.version.as_str().as_bytes());
        }
        key
    }
}

impl UnknownDependencyMarker {
    fn duplicate_identity_key(&self) -> Vec<u8> {
        self.canonical_sort_key()
    }

    fn canonical_sort_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.family.as_bytes(),
            self.reason.as_bytes(),
            self.owner.as_bytes(),
        ])
    }
}

impl From<DuplicateConflict> for UnknownDependencyMarker {
    fn from(value: DuplicateConflict) -> Self {
        Self {
            family: "conflicting_duplicate".to_owned(),
            owner: value.collection.to_owned(),
            reason: format!("conflicting duplicate identity {}", value.key),
        }
    }
}

fn optional_str_bytes(value: Option<&str>) -> Vec<u8> {
    let mut key = Vec::new();
    match value {
        Some(value) => {
            key.push(1);
            key.extend_from_slice(value.as_bytes());
        }
        None => key.push(0),
    }
    key
}

fn bool_sort_bytes(value: bool) -> &'static [u8] {
    if value { b"true" } else { b"false" }
}

fn completeness_sort_bytes(value: DependencyFootprintCompleteness) -> &'static [u8] {
    match value {
        DependencyFootprintCompleteness::Complete => b"complete",
        DependencyFootprintCompleteness::ConservativeComplete => b"conservative_complete",
        DependencyFootprintCompleteness::IncompleteUncacheable => b"incomplete_uncacheable",
    }
}

fn state_sort_key(value: &ProofReuseValidationState) -> Vec<u8> {
    let mut key = Vec::new();
    match value {
        ProofReuseValidationState::Complete => key.extend(b"complete"),
        ProofReuseValidationState::Mismatched => key.extend(b"mismatched"),
        ProofReuseValidationState::Missing => key.extend(b"missing"),
        ProofReuseValidationState::ExternalOnly => key.extend(b"external_only"),
        ProofReuseValidationState::UnsupportedEvidenceKind(kind) => {
            key.extend(b"unsupported_evidence_kind");
            key.extend(kind.as_bytes());
        }
    }
    key
}

fn compatibility_field_duplicate_identity_key(value: &CompatibilityField) -> Vec<u8> {
    canonical_key_parts([value.family.as_bytes(), value.field_name.as_bytes()])
}

fn compatibility_field_canonical_sort_key(value: &CompatibilityField) -> Vec<u8> {
    canonical_key_parts([
        value.family.as_bytes(),
        value.field_name.as_bytes(),
        value.value.as_bytes(),
    ])
}

fn named_schema_version_duplicate_identity_key(value: &NamedSchemaVersion) -> Vec<u8> {
    canonical_key_parts([value.schema_family.as_bytes(), value.name.as_bytes()])
}

fn named_schema_version_canonical_sort_key(value: &NamedSchemaVersion) -> Vec<u8> {
    canonical_key_parts([
        value.schema_family.as_bytes(),
        value.name.as_bytes(),
        value.version.as_str().as_bytes(),
    ])
}

fn append_optional_named_hash_sort_key(key: &mut Vec<u8>, field: &[u8], value: &Option<NamedHash>) {
    append_key_part(key, field);
    match value {
        Some(value) => {
            append_key_part(key, b"some");
            append_key_part(key, value.name.as_bytes());
            append_key_part(key, value.domain.as_bytes());
            append_key_part(key, value.digest.as_bytes());
        }
        None => append_key_part(key, b"none"),
    }
}

fn canonical_key_parts<const N: usize>(parts: [&[u8]; N]) -> Vec<u8> {
    let mut key = Vec::new();
    for part in parts {
        append_key_part(&mut key, part);
    }
    key
}

fn append_key_part(key: &mut Vec<u8>, part: &[u8]) {
    for byte in part {
        key.push(1);
        key.push(*byte);
    }
    key.push(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use mizar_artifact::{
        module_summary::{
            DependencyInterfaceRef, ExportedLabelSummary, ExportedSymbolSummary,
            LexicalContributionSummary, ModuleLexicalSummary, ModuleReexportSummary,
            ProofStatusSummary, SourceRangeSummary, current_schema_version,
        },
        registration_summary::{
            ActivatedRegistrationSummary, ArtifactHashClass, ArtifactHashRef,
            RegistrationAcceptedStatus, RegistrationContributionKind,
            RegistrationContributionSummary, RegistrationKind, RegistrationPatternSummary,
            RegistrationSummary, RegistrationVisibility,
        },
    };
    use mizar_core::{
        control_flow::ObligationHandoffId,
        core_ir::{
            CoreFormulaId, CoreItemId, CoreLabelRef, CoreProvenance, CoreProvenancePhase,
            CoreSourceRef, LocalProofOrProgramPath, NormalizedSemanticOrigin, ObligationSeedId,
            ObligationSeedStatus,
        },
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, SessionIdAllocator, SourceRange,
    };
    use mizar_vc::{
        dependency_slice::{DependencySliceInput, try_compute_dependency_slices},
        vc_ir::{
            AnchorCompleteness, AnchorLabel, AnchorLabelRole, AnchorOwner, CanonicalSortKey,
            ContextEntry, ContextEntryId, ContextEntryKind, GenerationSchemaVersion, HashMarker,
            LocalContext, PremiseRef, SeedAccounting, SeedOriginRef, SeedVcMapping, SeedVcRef,
            VcFormulaRef, VcId, VcIr, VcKind, VcModuleRef, VcProvenance, VcProvenancePhase,
            VcSchemaVersion, VcSet, VcSetParts, VcSourceRef, VcStatus, VcText,
        },
    };

    #[test]
    fn footprint_is_deterministic_and_canonicalizes_vectors() {
        let mut first = request();
        first.fingerprints.push(first.fingerprints[0].clone());
        first.slices.push(first.slices[0].clone());
        first
            .compatibility_fields
            .push(first.compatibility_fields[0].clone());
        first
            .proof_reuse_validation
            .push(first.proof_reuse_validation[0].clone());
        first.unknown_markers.reverse();
        first.fingerprints.reverse();
        first.slices.reverse();
        first.compatibility_fields.reverse();
        first.proof_reuse_validation.reverse();

        let first = reusable(first);
        let second = reusable(request());

        assert_eq!(first.final_hash, second.final_hash);
        assert_eq!(first.fingerprints, second.fingerprints);
        assert_eq!(first.slices, second.slices);
        assert_eq!(first.compatibility_fields, second.compatibility_fields);
        assert_eq!(first.proof_reuse_validation, second.proof_reuse_validation);
    }

    #[test]
    fn unknown_markers_are_sorted_and_deduplicated() {
        let mut first = request();
        first.unknown_markers = vec![
            unknown_marker("trace", "pkg::A", "unknown trace coverage"),
            unknown_marker("import", "pkg::A", "unknown imported interface"),
            unknown_marker("trace", "pkg::A", "unknown trace coverage"),
        ];
        first.unknown_markers.reverse();
        let mut second = request();
        second.unknown_markers = vec![
            unknown_marker("import", "pkg::A", "unknown imported interface"),
            unknown_marker("trace", "pkg::A", "unknown trace coverage"),
        ];

        let first = uncacheable(first);
        let second = uncacheable(second);

        assert_eq!(first.final_hash, second.final_hash);
        assert_eq!(first.unknown_markers, second.unknown_markers);
        assert_eq!(
            first.unknown_markers,
            [
                unknown_marker("import", "pkg::A", "unknown imported interface"),
                unknown_marker("trace", "pkg::A", "unknown trace coverage"),
            ]
        );
    }

    #[test]
    fn duplicate_identity_with_different_payload_forces_uncacheable_diagnostic() {
        assert_conflicting_duplicate("fingerprints", |request| {
            let mut conflicting = request.fingerprints[0].clone();
            conflicting.value_hash = hash(99);
            request.fingerprints.push(conflicting);
        });
        assert_conflicting_duplicate("slices", |request| {
            let mut conflicting = request.slices[0].clone();
            conflicting.digest = hash(98);
            request.slices.push(conflicting);
        });
        assert_conflicting_duplicate("compatibility_fields", |request| {
            let mut conflicting = request.compatibility_fields[0].clone();
            conflicting.value = "nightly".to_owned();
            request.compatibility_fields.push(conflicting);
        });
        assert_conflicting_duplicate("proof_reuse_validation", |request| {
            let mut conflicting = request.proof_reuse_validation[0].clone();
            conflicting
                .validation_hash
                .as_mut()
                .expect("validation hash")
                .digest = hash(97);
            request.proof_reuse_validation.push(conflicting);
        });
        assert_conflicting_duplicate(
            "proof_reuse_validation.metadata_schema_versions",
            |request| {
                request.proof_reuse_validation[0]
                    .metadata_schema_versions
                    .push(NamedSchemaVersion {
                        schema_family: "mizar-proof/reuse-metadata".to_owned(),
                        name: "proof-reuse".to_owned(),
                        version: SchemaVersion::new("mizar-proof/reuse-metadata/v2"),
                    });
            },
        );
    }

    #[test]
    fn conflicting_duplicate_diagnostic_hash_is_order_independent() {
        let mut slice_first = request();
        let mut slice_conflict = slice_first.slices[0].clone();
        slice_conflict.completeness = DependencyFootprintCompleteness::IncompleteUncacheable;
        let mut slice_second = slice_first.clone();
        slice_first.slices.push(slice_conflict.clone());
        slice_second.slices.insert(0, slice_conflict);
        assert_eq!(
            uncacheable(slice_first).final_hash,
            uncacheable(slice_second).final_hash
        );

        let mut proof_first = request();
        let mut validation_only = proof_first.proof_reuse_validation[0].clone();
        validation_only.witness_or_discharge_hash = None;
        let mut witness_only = proof_first.proof_reuse_validation[0].clone();
        witness_only.validation_hash = None;
        proof_first.proof_reuse_validation = vec![validation_only.clone(), witness_only.clone()];
        let mut proof_second = request();
        proof_second.proof_reuse_validation = vec![witness_only, validation_only];
        assert_eq!(
            uncacheable(proof_first).final_hash,
            uncacheable(proof_second).final_hash
        );

        let mut fingerprint_first = request();
        let mut hidden_payload_conflict = fingerprint_first.fingerprints[0].clone();
        hidden_payload_conflict.importer_visible = false;
        let mut fingerprint_second = fingerprint_first.clone();
        fingerprint_first
            .fingerprints
            .push(hidden_payload_conflict.clone());
        fingerprint_second
            .fingerprints
            .insert(0, hidden_payload_conflict);
        assert_eq!(
            uncacheable(fingerprint_first).final_hash,
            uncacheable(fingerprint_second).final_hash
        );
    }

    #[test]
    fn semantic_fingerprint_fields_change_final_hash() {
        let base = reusable(request()).final_hash;
        let cases = vec![
            mutate(|request| request.owner.package_id = "other".to_owned()),
            mutate(|request| request.phase = PipelinePhase::new("proof")),
            mutate(|request| request.fingerprints[0].target = FingerprintTargetKind::Definition),
            mutate(|request| request.fingerprints[0].identity.target_name = "other".to_owned()),
            mutate(|request| request.fingerprints[0].value_domain = "other-domain".to_owned()),
            mutate(|request| request.fingerprints[0].value_hash = hash(42)),
            mutate(|request| request.slices[0].name = "other-vc".to_owned()),
            mutate(|request| request.slices[0].digest = hash(43)),
            mutate(|request| request.compatibility_fields[0].value = "other".to_owned()),
            mutate(|request| {
                request.proof_reuse_validation[0]
                    .validation_hash
                    .as_mut()
                    .expect("validation hash")
                    .digest = hash(44);
            }),
        ];

        for case in cases {
            assert_ne!(base, reusable(case).final_hash);
        }
    }

    #[test]
    fn non_interface_summary_metadata_is_excluded_from_importer_visible_fingerprint() {
        let base_module = module_summary(hash(11));
        let mut changed_module = module_summary(hash(11));
        changed_module.source_hash = hash(90);
        changed_module.exported_symbols[0].source_range = SourceRangeSummary {
            start_byte: 100,
            end_byte: 120,
        };
        changed_module.exported_labels[0].source_range = SourceRangeSummary {
            start_byte: 200,
            end_byte: 220,
        };
        assert_eq!(
            DependencyFingerprint::from_module_summary(&base_module),
            DependencyFingerprint::from_module_summary(&changed_module)
        );

        let base_registration = registration_summary(hash(12));
        let mut changed_registration = registration_summary(hash(12));
        changed_registration.source_hash = hash(91);
        changed_registration.activated_registrations[0].source_range = Some(SourceRangeSummary {
            start_byte: 300,
            end_byte: 320,
        });
        assert_eq!(
            DependencyFingerprint::from_registration_summary(&base_registration),
            DependencyFingerprint::from_registration_summary(&changed_registration)
        );
    }

    #[test]
    fn interface_change_invalidates_importer_visible_fingerprint() {
        let mut changed = request();
        changed.fingerprints[0].value_hash = hash(50);

        assert_ne!(reusable(request()).final_hash, reusable(changed).final_hash);
    }

    #[test]
    fn implementation_only_change_does_not_change_importer_visible_subset() {
        let mut base = request();
        base.fingerprints.push(implementation_fingerprint(hash(70)));
        let mut changed = request();
        changed
            .fingerprints
            .push(implementation_fingerprint(hash(71)));

        let base_importer = importer_visible_fingerprint_hashes(&reusable(base));
        let changed_importer = importer_visible_fingerprint_hashes(&reusable(changed));

        assert_eq!(base_importer, changed_importer);
    }

    #[test]
    fn slice_change_flips_only_dependent_slice_hash_at_current_granularity() {
        let mut base_request = request();
        base_request
            .slices
            .push(slice("vc-slice", "pkg::A", "obligation-2", hash(32)));
        let mut changed_request = base_request.clone();
        changed_request.slices[0].digest = hash(33);

        let base = reusable(base_request);
        let changed = reusable(changed_request);

        let base_named = slices_by_name(&base);
        let changed_named = slices_by_name(&changed);
        assert_ne!(
            base_named.get("obligation-1"),
            changed_named.get("obligation-1")
        );
        assert_eq!(
            base_named.get("obligation-2"),
            changed_named.get("obligation-2")
        );
        assert_ne!(base.final_hash, changed.final_hash);
    }

    #[test]
    fn vc_dependency_slice_projection_consumes_producer_slice_fingerprint() {
        let vc_set = vc_fixture_set();
        let slice_set = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &vc_set,
            discharge_output: None,
        })
        .expect("dependency slices");
        let producer_slice = slice_set.slice_for(VcId::new(0)).expect("vc slice");
        let projected = DependencySliceFingerprint::from_vc_slice(
            "vc-slice",
            "pkg::A",
            "obligation-1",
            producer_slice,
        );

        assert_eq!(projected.digest, producer_slice.fingerprint().hash());
        assert_eq!(projected.domain, DEPENDENCY_SLICE_SCHEMA_VERSION);
        assert_eq!(
            projected.completeness,
            match producer_slice.completeness() {
                DependencySliceCompleteness::Complete => DependencyFootprintCompleteness::Complete,
                DependencySliceCompleteness::IncompleteUncacheable => {
                    DependencyFootprintCompleteness::IncompleteUncacheable
                }
                _ => DependencyFootprintCompleteness::IncompleteUncacheable,
            }
        );
    }

    #[test]
    fn missing_unknown_and_uncacheable_inputs_force_miss() {
        for case in [
            mutate(|request| {
                request.requested_completeness =
                    DependencyFootprintCompleteness::IncompleteUncacheable
            }),
            mutate(|request| {
                request.unknown_markers.push(UnknownDependencyMarker {
                    family: "import".to_owned(),
                    owner: "pkg::A".to_owned(),
                    reason: "producer reported unknown coverage".to_owned(),
                });
            }),
            mutate(|request| {
                request.slices[0].completeness =
                    DependencyFootprintCompleteness::IncompleteUncacheable;
            }),
            mutate(|request| request.uncacheable = true),
        ] {
            let footprint = uncacheable(case);
            assert_eq!(
                footprint.completeness,
                DependencyFootprintCompleteness::IncompleteUncacheable
            );
            assert!(footprint.uncacheable);
        }

        let mut missing_required_families = request();
        missing_required_families.requested_completeness =
            DependencyFootprintCompleteness::Complete;
        missing_required_families.slices.clear();
        missing_required_families.compatibility_fields.clear();
        missing_required_families.proof_reuse_validation.clear();
        let footprint = uncacheable(missing_required_families);
        assert_marker_family(&footprint, "compatibility");
        assert_marker_family(&footprint, "vc_slice");
        assert_marker_family(&footprint, "proof_reuse_identity");

        let mut unknown_toolchain = request();
        unknown_toolchain.compatibility_fields[0].value = "unknown".to_owned();
        assert!(uncacheable(unknown_toolchain).uncacheable);

        let mut empty_toolchain = request();
        empty_toolchain.compatibility_fields[0].value.clear();
        assert!(uncacheable(empty_toolchain).uncacheable);
    }

    #[test]
    fn proof_reuse_validation_failures_force_miss_without_granting_trust() {
        for state in [
            ProofReuseValidationState::Mismatched,
            ProofReuseValidationState::Missing,
            ProofReuseValidationState::ExternalOnly,
            ProofReuseValidationState::UnsupportedEvidenceKind("backend-log".to_owned()),
        ] {
            let mut request = request();
            request.proof_reuse_validation[0].state = state;
            let footprint = uncacheable(request);
            assert!(footprint.uncacheable);
        }

        let mut missing_hash = request();
        missing_hash.proof_reuse_validation[0].validation_hash = None;
        assert!(uncacheable(missing_hash).uncacheable);

        let mut missing_witness = request();
        missing_witness.proof_reuse_validation[0].witness_or_discharge_hash = None;
        assert!(uncacheable(missing_witness).uncacheable);

        let mut missing_schema = request();
        missing_schema.proof_reuse_validation[0]
            .metadata_schema_versions
            .clear();
        assert!(uncacheable(missing_schema).uncacheable);
    }

    #[test]
    fn unsupported_footprint_schema_produces_no_footprint() {
        let mut request = request();
        request.schema_version = SchemaVersion::new("mizar-cache/dependency-fingerprint-schema/v2");

        let rejection = match DependencyFootprintBuilder::new(request).build() {
            DependencyFootprintBuildOutcome::NoFootprint(rejection) => rejection,
            outcome => panic!("expected no footprint, got {outcome:?}"),
        };

        assert!(matches!(
            rejection,
            DependencyFootprintBuildRejection::UnsupportedSchema { .. }
        ));
    }

    #[test]
    fn trigger_evaluator_reuses_for_non_semantic_changes() {
        for change_kind in [
            FingerprintChangeKind::CommentOnly,
            FingerprintChangeKind::DiagnosticWordingOnly,
            FingerprintChangeKind::RuntimeObservationOnly,
        ] {
            let decision = trigger(change_kind, FingerprintTargetKind::Source);
            assert_eq!(decision.trigger, RebuildTrigger::ReuseAllowed);
            assert!(!decision.conservative);
        }
    }

    #[test]
    fn trigger_evaluator_rebuilds_dependents_for_visible_semantic_changes() {
        for (change_kind, target) in [
            (
                FingerprintChangeKind::SourceTokenAst,
                FingerprintTargetKind::Source,
            ),
            (
                FingerprintChangeKind::ModuleInterface,
                FingerprintTargetKind::ModuleInterface,
            ),
            (
                FingerprintChangeKind::RegistrationInterface,
                FingerprintTargetKind::RegistrationInterface,
            ),
            (
                FingerprintChangeKind::ClusterReductionVisibleOrigin,
                FingerprintTargetKind::ClusterTrace,
            ),
            (
                FingerprintChangeKind::ExportedSemantic,
                FingerprintTargetKind::TheoremStatement,
            ),
        ] {
            let decision = trigger(change_kind, target);
            assert_eq!(decision.trigger, RebuildTrigger::RebuildDependents);
            assert!(!decision.conservative);
        }
    }

    #[test]
    fn trigger_evaluator_refreshes_local_or_affected_phase_for_local_and_config_changes() {
        for (change_kind, target) in [
            (
                FingerprintChangeKind::ModuleImplementationOnly,
                FingerprintTargetKind::ModuleImplementation,
            ),
            (
                FingerprintChangeKind::ProofBodyOnly,
                FingerprintTargetKind::ProofBody,
            ),
            (
                FingerprintChangeKind::Policy,
                FingerprintTargetKind::PolicyToolchain,
            ),
            (
                FingerprintChangeKind::Toolchain,
                FingerprintTargetKind::PolicyToolchain,
            ),
            (
                FingerprintChangeKind::SchemaVersion,
                FingerprintTargetKind::PolicyToolchain,
            ),
            (
                FingerprintChangeKind::Lockfile,
                FingerprintTargetKind::LockfileManifest,
            ),
            (
                FingerprintChangeKind::Manifest,
                FingerprintTargetKind::LockfileManifest,
            ),
        ] {
            let decision = trigger(change_kind, target);
            assert_eq!(decision.trigger, RebuildTrigger::RebuildPhase);
            assert!(!decision.conservative);
        }
    }

    #[test]
    fn trigger_evaluator_misses_for_incomplete_unknown_or_uncacheable_inputs() {
        for change_kind in [
            FingerprintChangeKind::IncompleteFootprint,
            FingerprintChangeKind::UnknownSchema,
            FingerprintChangeKind::UnknownToolchain,
            FingerprintChangeKind::UncacheableMarker,
            FingerprintChangeKind::MissingProofReuseValidation,
        ] {
            let decision = trigger(change_kind, FingerprintTargetKind::ProofReuseIdentity);
            assert_eq!(decision.trigger, RebuildTrigger::UncacheableMiss);
            assert!(!decision.conservative);
        }
    }

    #[test]
    fn conservative_coarse_slices_overtrigger_without_false_negative_reuse() {
        let semantic = RebuildTriggerEvaluator::evaluate(RebuildTriggerInput {
            change_kind: FingerprintChangeKind::ExportedSemantic,
            target: FingerprintTargetKind::Definition,
            dependent_phase: PipelinePhase::new("proof"),
            slice_precision: DependencySlicePrecision::ConservativeCoarse,
        });

        assert_eq!(semantic.trigger, RebuildTrigger::RebuildDependents);
        assert!(semantic.conservative);

        let diagnostic = RebuildTriggerEvaluator::evaluate(RebuildTriggerInput {
            change_kind: FingerprintChangeKind::DiagnosticWordingOnly,
            target: FingerprintTargetKind::VcSlice,
            dependent_phase: PipelinePhase::new("proof"),
            slice_precision: DependencySlicePrecision::ConservativeCoarse,
        });

        assert_eq!(diagnostic.trigger, RebuildTrigger::ReuseAllowed);
        assert!(!diagnostic.conservative);
    }

    #[test]
    fn trigger_evaluator_combines_rows_by_documented_precedence() {
        let rows = vec![
            trigger_input(
                FingerprintChangeKind::CommentOnly,
                FingerprintTargetKind::Source,
                DependencySlicePrecision::Exact,
            ),
            trigger_input(
                FingerprintChangeKind::ModuleImplementationOnly,
                FingerprintTargetKind::ModuleImplementation,
                DependencySlicePrecision::Exact,
            ),
            trigger_input(
                FingerprintChangeKind::ModuleInterface,
                FingerprintTargetKind::ModuleInterface,
                DependencySlicePrecision::Exact,
            ),
            trigger_input(
                FingerprintChangeKind::UnknownSchema,
                FingerprintTargetKind::PolicyToolchain,
                DependencySlicePrecision::Exact,
            ),
        ];
        let mut reversed = rows.clone();
        reversed.reverse();

        let first = RebuildTriggerEvaluator::evaluate_all(rows);
        let second = RebuildTriggerEvaluator::evaluate_all(reversed);

        assert_eq!(first, second);
        assert_eq!(first.trigger, RebuildTrigger::UncacheableMiss);
        assert!(!first.conservative);
        assert_eq!(first.row_count, 4);
    }

    #[test]
    fn trigger_evaluator_combined_conservative_flag_tracks_strongest_coarse_rows() {
        let summary = RebuildTriggerEvaluator::evaluate_all([
            trigger_input(
                FingerprintChangeKind::ModuleInterface,
                FingerprintTargetKind::ModuleInterface,
                DependencySlicePrecision::ConservativeCoarse,
            ),
            trigger_input(
                FingerprintChangeKind::ProofBodyOnly,
                FingerprintTargetKind::ProofBody,
                DependencySlicePrecision::ConservativeCoarse,
            ),
        ]);

        assert_eq!(summary.trigger, RebuildTrigger::RebuildDependents);
        assert!(summary.conservative);
        assert_eq!(summary.row_count, 2);
    }

    #[test]
    fn trigger_evaluator_combined_conservative_flag_ignores_lower_precedence_rows() {
        let mixed_equal_precedence = RebuildTriggerEvaluator::evaluate_all([
            trigger_input(
                FingerprintChangeKind::ModuleInterface,
                FingerprintTargetKind::ModuleInterface,
                DependencySlicePrecision::Exact,
            ),
            trigger_input(
                FingerprintChangeKind::ExportedSemantic,
                FingerprintTargetKind::TheoremStatement,
                DependencySlicePrecision::ConservativeCoarse,
            ),
        ]);
        assert_eq!(
            mixed_equal_precedence.trigger,
            RebuildTrigger::RebuildDependents
        );
        assert!(mixed_equal_precedence.conservative);

        let stronger_uncacheable = RebuildTriggerEvaluator::evaluate_all([
            trigger_input(
                FingerprintChangeKind::ExportedSemantic,
                FingerprintTargetKind::TheoremStatement,
                DependencySlicePrecision::ConservativeCoarse,
            ),
            trigger_input(
                FingerprintChangeKind::UncacheableMarker,
                FingerprintTargetKind::PolicyToolchain,
                DependencySlicePrecision::Exact,
            ),
        ]);
        assert_eq!(
            stronger_uncacheable.trigger,
            RebuildTrigger::UncacheableMiss
        );
        assert!(!stronger_uncacheable.conservative);
    }

    #[test]
    fn summary_constructors_project_producer_hashes() {
        let module_summary = module_summary(hash(11));
        let module_fingerprint = DependencyFingerprint::from_module_summary(&module_summary);
        assert_eq!(
            module_fingerprint.target,
            FingerprintTargetKind::ModuleInterface
        );
        assert_eq!(module_fingerprint.value_hash, module_summary.interface_hash);
        assert!(module_fingerprint.importer_visible);

        let registration_summary = registration_summary(hash(12));
        let registration_fingerprint =
            DependencyFingerprint::from_registration_summary(&registration_summary);
        assert_eq!(
            registration_fingerprint.target,
            FingerprintTargetKind::RegistrationInterface
        );
        assert_eq!(
            registration_fingerprint.value_hash,
            registration_summary.registration_interface_hash
        );
        assert!(registration_fingerprint.importer_visible);
    }

    fn request() -> DependencyFootprintRequest {
        DependencyFootprintRequest {
            schema_version: SchemaVersion::new(DEPENDENCY_FINGERPRINT_SCHEMA_VERSION),
            owner: FootprintOwner {
                package_id: "pkg".to_owned(),
                module_path: "pkg::A".to_owned(),
                origin_id: None,
                language_edition: Some("miz-2025".to_owned()),
                lockfile_identity: Some("lock-v1".to_owned()),
            },
            phase: PipelinePhase::new("vc"),
            fingerprints: vec![
                DependencyFingerprint::from_module_summary(&module_summary(hash(10))),
                DependencyFingerprint::from_registration_summary(&registration_summary(hash(20))),
            ],
            slices: vec![slice("vc-slice", "pkg::A", "obligation-1", hash(30))],
            compatibility_fields: vec![CompatibilityField {
                family: "toolchain".to_owned(),
                field_name: "rustc".to_owned(),
                value: "stable".to_owned(),
            }],
            proof_reuse_validation: vec![proof_reuse_validation()],
            unknown_markers: Vec::new(),
            requested_completeness: DependencyFootprintCompleteness::ConservativeComplete,
            uncacheable: false,
        }
    }

    fn reusable(request: DependencyFootprintRequest) -> DependencyFootprint {
        match DependencyFootprintBuilder::new(request).build() {
            DependencyFootprintBuildOutcome::Reusable(footprint) => footprint,
            outcome => panic!("expected reusable footprint, got {outcome:?}"),
        }
    }

    fn uncacheable(request: DependencyFootprintRequest) -> DependencyFootprint {
        match DependencyFootprintBuilder::new(request).build() {
            DependencyFootprintBuildOutcome::Uncacheable(footprint) => footprint,
            outcome => panic!("expected uncacheable footprint, got {outcome:?}"),
        }
    }

    fn assert_conflicting_duplicate(
        collection: &'static str,
        mutate: impl FnOnce(&mut DependencyFootprintRequest),
    ) {
        let mut request = request();
        mutate(&mut request);
        let footprint = uncacheable(request);

        assert_eq!(
            footprint.completeness,
            DependencyFootprintCompleteness::IncompleteUncacheable
        );
        assert!(footprint.uncacheable);
        assert!(
            footprint.unknown_markers.iter().any(|marker| {
                marker.family == "conflicting_duplicate" && marker.owner == collection
            }),
            "expected conflicting duplicate marker for {collection}, got {:?}",
            footprint.unknown_markers
        );
    }

    fn assert_marker_family(footprint: &DependencyFootprint, family: &str) {
        assert!(
            footprint
                .unknown_markers
                .iter()
                .any(|marker| marker.family == family),
            "expected unknown marker family {family}, got {:?}",
            footprint.unknown_markers
        );
    }

    fn trigger(
        change_kind: FingerprintChangeKind,
        target: FingerprintTargetKind,
    ) -> RebuildTriggerDecision {
        RebuildTriggerEvaluator::evaluate(trigger_input(
            change_kind,
            target,
            DependencySlicePrecision::Exact,
        ))
    }

    fn trigger_input(
        change_kind: FingerprintChangeKind,
        target: FingerprintTargetKind,
        slice_precision: DependencySlicePrecision,
    ) -> RebuildTriggerInput {
        RebuildTriggerInput {
            change_kind,
            target,
            dependent_phase: PipelinePhase::new("proof"),
            slice_precision,
        }
    }

    fn mutate(mut f: impl FnMut(&mut DependencyFootprintRequest)) -> DependencyFootprintRequest {
        let mut request = request();
        f(&mut request);
        request
    }

    fn importer_visible_fingerprint_hashes(footprint: &DependencyFootprint) -> Vec<Hash> {
        footprint
            .fingerprints
            .iter()
            .filter(|fingerprint| fingerprint.importer_visible)
            .map(|fingerprint| fingerprint.value_hash)
            .collect()
    }

    fn slices_by_name(footprint: &DependencyFootprint) -> BTreeMap<String, Hash> {
        footprint
            .slices
            .iter()
            .map(|slice| (slice.name.clone(), slice.digest))
            .collect()
    }

    fn slice(
        slice_kind: impl Into<String>,
        owner: impl Into<String>,
        name: impl Into<String>,
        digest: Hash,
    ) -> DependencySliceFingerprint {
        DependencySliceFingerprint {
            slice_kind: slice_kind.into(),
            owner: owner.into(),
            name: name.into(),
            domain: DEPENDENCY_SLICE_SCHEMA_VERSION.to_owned(),
            digest,
            completeness: DependencyFootprintCompleteness::Complete,
        }
    }

    fn proof_reuse_validation() -> ProofReuseValidationInput {
        ProofReuseValidationInput {
            name: "proof-reuse:obligation-1".to_owned(),
            state: ProofReuseValidationState::Complete,
            validation_hash: Some(NamedHash {
                name: "validation".to_owned(),
                domain: "mizar-proof/reuse-validation/v1".to_owned(),
                digest: hash(40),
            }),
            witness_or_discharge_hash: Some(NamedHash {
                name: "witness".to_owned(),
                domain: "mizar-proof/witness/v1".to_owned(),
                digest: hash(41),
            }),
            metadata_schema_versions: vec![NamedSchemaVersion {
                schema_family: "mizar-proof/reuse-metadata".to_owned(),
                name: "proof-reuse".to_owned(),
                version: SchemaVersion::new("mizar-proof/reuse-metadata/v1"),
            }],
        }
    }

    fn unknown_marker(
        family: impl Into<String>,
        owner: impl Into<String>,
        reason: impl Into<String>,
    ) -> UnknownDependencyMarker {
        UnknownDependencyMarker {
            family: family.into(),
            owner: owner.into(),
            reason: reason.into(),
        }
    }

    fn implementation_fingerprint(value_hash: Hash) -> DependencyFingerprint {
        DependencyFingerprint::module_implementation(
            FingerprintIdentity {
                package_id: "pkg".to_owned(),
                module_path: "pkg::A".to_owned(),
                origin_id: None,
                target_name: "module-implementation".to_owned(),
                schema_family: "mizar-artifact/module-implementation".to_owned(),
                language_edition: Some("miz-2025".to_owned()),
                lockfile_identity: Some("lock-v1".to_owned()),
            },
            "mizar-artifact/module-implementation",
            value_hash,
            SchemaVersion::new("mizar-artifact/module-implementation/v1"),
        )
    }

    fn module_summary(interface_hash: Hash) -> ModuleSummary {
        ModuleSummary {
            schema_version: current_schema_version(),
            module: module_identity(),
            source_hash: hash(1),
            interface_hash,
            exported_symbols: vec![ExportedSymbolSummary {
                origin_id: "th:1".to_owned(),
                fully_qualified_name: "pkg::A::T".to_owned(),
                namespace_path: vec!["pkg".to_owned(), "A".to_owned()],
                visibility: "public".to_owned(),
                declaration_kind: "theorem".to_owned(),
                source_range: source_range(),
                rendered_signature: "T: thesis".to_owned(),
                interface_fingerprint: hash(2),
                proof_status: Some(ProofStatusSummary::Accepted),
            }],
            exported_labels: vec![ExportedLabelSummary {
                origin_id: "label:1".to_owned(),
                label: "L1".to_owned(),
                owner_fully_qualified_name: "pkg::A::T".to_owned(),
                visibility: "public".to_owned(),
                source_range: source_range(),
                target_kind: "theorem".to_owned(),
            }],
            lexical_summary: ModuleLexicalSummary {
                schema_version: "lexical-v1".to_owned(),
                fingerprint: Some(hash(3)),
                contributions: vec![LexicalContributionSummary {
                    kind: "notation".to_owned(),
                    key: "foo".to_owned(),
                    payload: "bar".to_owned(),
                }],
            },
            reexports: vec![ModuleReexportSummary {
                target_module: module_identity(),
                target_item_origin_id: Some("th:1".to_owned()),
                exported_name: Some("T".to_owned()),
                provenance_origin_id: Some("reexport:1".to_owned()),
            }],
            dependency_interfaces: vec![DependencyInterfaceRef {
                module: module_identity(),
                interface_hash: hash(4),
            }],
        }
    }

    fn registration_summary(registration_interface_hash: Hash) -> RegistrationSummary {
        RegistrationSummary {
            schema_version: mizar_artifact::registration_summary::current_schema_version(),
            module: module_identity(),
            source_hash: hash(5),
            registration_interface_hash,
            activated_registrations: vec![ActivatedRegistrationSummary {
                origin_id: "reg:1".to_owned(),
                label: Some("R1".to_owned()),
                registration_kind: RegistrationKind::Conditional,
                visibility: RegistrationVisibility::Public,
                namespace_path: vec!["pkg".to_owned(), "A".to_owned()],
                source_module: module_identity(),
                trigger_key: "cluster:foo".to_owned(),
                normalized_pattern: RegistrationPatternSummary {
                    fingerprint: artifact_hash(hash(6)),
                    type_head: Some("set".to_owned()),
                    attribute: None,
                    functor: None,
                    term_head: None,
                    parameters: vec!["x".to_owned()],
                    guards: vec![artifact_hash(hash(7))],
                },
                generated_contribution: RegistrationContributionSummary {
                    kind: RegistrationContributionKind::AttributeFact,
                    summary: "cluster foo".to_owned(),
                    fingerprint: artifact_hash(hash(8)),
                },
                accepted_status: RegistrationAcceptedStatus::Accepted,
                verifier_policy_fingerprint: artifact_hash(hash(9)),
                trace_ids: Vec::new(),
                source_range: Some(source_range()),
            }],
            trace_artifacts: Vec::new(),
            dependency_registrations: Vec::new(),
        }
    }

    fn artifact_hash(digest: Hash) -> ArtifactHashRef {
        ArtifactHashRef::new(
            ArtifactHashClass::Interface,
            "test-artifact",
            mizar_artifact::registration_summary::current_schema_version(),
            digest,
        )
    }

    fn module_identity() -> ModuleSummaryIdentity {
        ModuleSummaryIdentity {
            package_id: "pkg".to_owned(),
            package_version: Some("0.1.0".to_owned()),
            lockfile_identity: Some("lock-v1".to_owned()),
            module_path: "pkg::A".to_owned(),
            language_edition: "miz-2025".to_owned(),
        }
    }

    fn source_range() -> SourceRangeSummary {
        SourceRangeSummary {
            start_byte: 0,
            end_byte: 1,
        }
    }

    fn vc_fixture_set() -> VcSet {
        let snapshot = BuildSnapshotId::from_published_schema_str(
            "mizar-session-build-snapshot-v1:\
             2222222222222222222222222222222222222222222222222222222222222222",
        )
        .expect("snapshot id");
        let source = InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source id");
        let handoff = ObligationHandoffId::new(0);
        let source_ref = core_source_ref(source);

        VcSet::try_new(VcSetParts {
            schema_version: VcSchemaVersion::new("mizar-cache-task-5-test-v1"),
            snapshot,
            source,
            module: VcModuleRef::new("pkg::A"),
            generated_formulas: Vec::new(),
            vcs: vec![VcIr {
                id: VcId::new(0),
                kind: VcKind::TheoremProofStep,
                source: VcSourceRef {
                    primary: source_ref.clone(),
                    related: Vec::new(),
                },
                seed: SeedVcRef { handoff },
                anchor: complete_anchor(source),
                local_context: LocalContext::try_new(
                    vec![ContextEntry {
                        id: ContextEntryId::new(0),
                        sort_key: CanonicalSortKey::new("000-assumption"),
                        kind: ContextEntryKind::ProofAssumption,
                        formula: Some(VcFormulaRef::Core(CoreFormulaId::new(1))),
                        provenance: vec![vc_provenance("context")],
                    }],
                    Vec::new(),
                )
                .expect("local context"),
                premises: vec![PremiseRef::LocalContext(ContextEntryId::new(0))],
                goal: VcFormulaRef::Core(CoreFormulaId::new(0)),
                proof_hint: None,
                status: VcStatus::NeedsAtp,
                provenance: vec![vc_provenance("vc")],
            }],
            seed_accounting: vec![SeedAccounting {
                handoff,
                origin: SeedOriginRef::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
                seed_status: ObligationSeedStatus::Active,
                mapping: SeedVcMapping::One { vc: VcId::new(0) },
            }],
        })
        .expect("valid VC fixture")
    }

    fn complete_anchor(source: mizar_session::SourceId) -> mizar_vc::vc_ir::ObligationAnchor {
        mizar_vc::vc_ir::ObligationAnchor {
            owner: AnchorOwner::Theorem(CoreItemId::new(0)),
            kind: VcKind::TheoremProofStep,
            local_path: LocalProofOrProgramPath::new("proof/step/0"),
            label: Some(AnchorLabel {
                role: AnchorLabelRole::UserLabel,
                hint: Some(CoreLabelRef::new("A1")),
            }),
            semantic_origin: NormalizedSemanticOrigin::new("theorem:sample:proof-step:0"),
            source_range: Some(SourceRange {
                source_id: source,
                start: 0,
                end: 10,
            }),
            provenance: vec![vc_provenance("anchor")],
            source_shape_hash: HashMarker::Available(hash(81)),
            canonical_goal_hash: HashMarker::Available(hash(82)),
            canonical_context_hash: HashMarker::Available(hash(83)),
            generation_schema_version: GenerationSchemaVersion::new("mizar-cache-task-5-test"),
            completeness: AnchorCompleteness::Complete,
        }
    }

    fn core_source_ref(source: mizar_session::SourceId) -> CoreSourceRef {
        CoreSourceRef::direct(SourceRange {
            source_id: source,
            start: 0,
            end: 10,
        })
        .with_provenance(vec![CoreProvenance::new(
            CoreProvenancePhase::ProofSkeleton,
            "direct-source",
        )])
    }

    fn vc_provenance(key: &str) -> VcProvenance {
        VcProvenance {
            phase: VcProvenancePhase::Generator,
            key: VcText::new(key),
            core: None,
        }
    }

    fn hash(seed: u8) -> Hash {
        Hash::from_bytes([seed; Hash::BYTE_LEN])
    }
}
