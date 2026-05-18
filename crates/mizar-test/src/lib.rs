pub mod diagnostic;
pub mod expectation;
pub mod harness;
pub mod layout;
pub mod path_rules;
pub mod staged_model;
pub mod toml_lite;
pub mod traceability;

pub use diagnostic::{DiagnosticCode, ValidationDiagnostic, ValidationSeverity};
pub use expectation::{
    Expectation, ExpectedOutcome, PipelinePhase, TestCaseId, TestKind, parse_expectation_file,
};
pub use harness::{
    DiscoveryConfig, HarnessError, TestCase, TestPlan, TestProfile, ValidationMode, build_test_plan,
};
pub use staged_model::Stage;
pub use traceability::{SpecRequirement, SpecRequirementId, TraceManifest, parse_trace_manifest};
