//! Driver front door for build requests, sessions, phase-service wiring, and
//! entry-point orchestration.
//!
//! The crate is introduced by
//! [`00.crate_plan.md`](../../../doc/design/mizar-driver/en/00.crate_plan.md).
//! Task-scoped modules are added only after their paired EN/JA design specs
//! land. Until then, this scaffold owns no phase semantics, proof authority,
//! cache compatibility decisions, artifact serialization, or LSP protocol
//! conversion.

#![forbid(unsafe_code)]

pub mod cli;
pub mod driver;
pub mod events;
mod frontend_adapter;
pub mod registry;
pub mod request;

/// Consumer-facing entry points and the existing owner configuration types.
pub mod facade {
    pub use crate::driver::{
        BuildSubmission, CompilerDriver, DiagnosticIndexMode, DriverSubmitError, DriverSubmitInput,
        NativeCacheMode, NativeCacheOptions, SubmitOptions, WatchSubmission, WatchSubmitControl,
        WatchSubmitError, WatchSubmitFailure,
    };
    pub use crate::registry::{
        PhaseRegistry, PhaseRegistryBuilder, StageConfig, assemble_registry,
    };
    pub use crate::request::{
        BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequest, BuildRequestDraft,
        BuildRequestGeneration, BuildRequestOrigin, BuildSession, BuildTargets, CapturedSnapshot,
        DependencyInputSet, LspFocus, LspPriority, LspRequest, PendingBuildRequest,
        PublicationDecision, SourceInputSet, VerifierConfigInput, WatchRequest,
    };
    pub use mizar_atp::problem::{
        ConcreteFormat, EqualitySupport, LogicFragment, LogicProfile, NativePropertySupport,
        QuantifierPolicy, SoftTypeStrategy,
    };
    pub use mizar_build::task_graph::PipelinePhase;
    pub use mizar_cache::cache_store::CacheStoreRoot;
    pub use mizar_ir::{cache_adapter::IrCacheAdapter, publisher::PhaseOutputPublisher};
    pub use mizar_proof::policy::VerifierPolicy;
    pub use mizar_vc::{
        discharge::DischargePolicy,
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion},
    };
}

pub use frontend_adapter::native_algorithm_core_from_storage_bytes;
pub use frontend_adapter::native_algorithm_core_storage_bytes;
pub use frontend_adapter::native_algorithm_vc_from_storage_bytes;
pub use frontend_adapter::native_algorithm_vc_storage_bytes;
pub use frontend_adapter::native_theorem_candidate_check_from_storage_bytes;
pub use frontend_adapter::native_theorem_candidate_check_storage_bytes;
pub use frontend_adapter::native_theorem_candidate_from_storage_bytes;
pub use frontend_adapter::native_theorem_candidate_storage_bytes;
pub use frontend_adapter::native_theorem_core_from_storage_bytes;
pub use frontend_adapter::native_theorem_core_semantic_payload;
pub use frontend_adapter::native_theorem_core_storage_bytes;
pub use frontend_adapter::native_theorem_discharge_from_storage_bytes;
pub use frontend_adapter::native_theorem_discharge_storage_bytes;
pub use frontend_adapter::native_theorem_metadata_from_storage_bytes;
pub use frontend_adapter::native_theorem_metadata_storage_bytes;
pub use frontend_adapter::native_theorem_preparation_from_storage_bytes;
pub use frontend_adapter::native_theorem_preparation_storage_bytes;
pub use frontend_adapter::native_theorem_registration_closure_from_storage_bytes;
pub use frontend_adapter::native_theorem_registration_closure_storage_bytes;
pub use frontend_adapter::native_theorem_typed_ast_from_storage_bytes;
pub use frontend_adapter::native_theorem_typed_ast_storage_bytes;
pub use frontend_adapter::native_theorem_vc_from_storage_bytes;
pub use frontend_adapter::native_theorem_vc_storage_bytes;
