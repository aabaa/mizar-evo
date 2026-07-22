pub mod diagnostic;
pub mod expectation;
pub mod harness;
pub mod layout;
pub mod path_rules;
pub mod runner;
pub mod snapshot;
pub mod staged_model;
pub mod toml_lite;
pub mod traceability;

pub use diagnostic::{DiagnosticCode, ValidationDiagnostic, ValidationSeverity};
pub use expectation::{
    Architecture22Gate, Architecture22Metadata, Architecture22ScenarioSpec, Expectation,
    ExpectedOutcome, OriginMetadata, PipelinePhase, TestCaseId, TestKind,
    architecture22_scenario_spec, architecture22_scenario_specs, parse_expectation_file,
};
pub use harness::{
    DiscoveryConfig, HarnessError, TestCase, TestPlan, TestProfile, ValidationMode, build_test_plan,
};
pub use runner::{
    DeclarationSymbolCaseResult, DeclarationSymbolCaseStatus, DeclarationSymbolRunReport,
    ParseOnlyCaseResult, ParseOnlyCaseStatus, ParseOnlyRunReport, ProofVerificationCaseResult,
    ProofVerificationCaseStatus, ProofVerificationRunReport, TypeElaborationCaseResult,
    TypeElaborationCaseStatus, TypeElaborationRunReport, active_declaration_symbol_cases,
    active_parse_only_cases, active_proof_verification_cases, active_type_elaboration_cases,
    run_declaration_symbol_corpus, run_parse_only_corpus, run_proof_verification_corpus,
    run_type_elaboration_corpus,
};
pub use snapshot::{
    ParallelismProfile, SchemaVersion, SnapshotBaselineError, SnapshotBaselineMismatch,
    SnapshotBaselineReport, SnapshotBaselineStatus, SnapshotBody, SnapshotDeterminismFailure,
    SnapshotError, SnapshotKind, SnapshotMismatch, SnapshotProfile, SnapshotRecord,
    SnapshotTextDiff, SnapshotUpdateMode, SnapshotUpdateReason, ToolchainInfo,
    compare_snapshot_records, verify_or_update_snapshot_baseline, verify_snapshot_determinism,
    verify_snapshot_parallel_equivalence,
};
pub use staged_model::Stage;
pub use traceability::{
    Architecture22MatrixReport, Architecture22ScenarioReport, CoverageEvidence,
    CoverageEvidenceSummary, CoverageReport, CoverageShape, PassFailMix, RequirementCoverage,
    RequirementStatus, SpecRequirement, SpecRequirementId, StageCoverage, TraceManifest,
    parse_trace_manifest,
};
