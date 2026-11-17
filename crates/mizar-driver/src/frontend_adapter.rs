//! Real disk source publication, native cache request preparation and dependency lexical input.
//! See [the source service contract](../../../../doc/design/mizar-driver/en/frontend_adapter.md).
use std::{path::PathBuf, sync::Arc};

use mizar_build::{
    module_index::ModuleIndexLocation,
    planner::PackagePlanSource,
    task_graph::{PipelinePhase, WorkUnit},
};
use mizar_diagnostics::{
    failure_record::{
        DiagnosticDetailValue, DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput,
        DiagnosticNote, DiagnosticNoteKind, DiagnosticPrimaryLocation, DiagnosticSpan,
        DiagnosticSpanRole, FailureCategory, PipelinePhase as DiagnosticPhase, SpanFreshness,
    },
    registry::{DiagnosticCode, DiagnosticSeverity},
};
pub(crate) use mizar_frontend::cache_key::SOURCE_UNIT_CACHE_KEY_VERSION;
use mizar_frontend::{
    cache_key::{
        ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION, SourceUnitCacheKey,
        TOKEN_STREAM_CACHE_KEY_VERSION,
    },
    lexical_env::{
        ExportedSymbolShape, LexicalEnvironmentDiagnosticCode as EnvironmentCode,
        ModuleLexicalSummary,
    },
    lexing::{LexDiagnosticCode, LexingDiagnosticKind, ScopeSkeletonDiagnosticCode},
    orchestration::{
        DiagnosticClass, DiagnosticCode as FrontendCode, DiagnosticLocation, Frontend,
        FrontendDiagnostic, FrontendOutput,
    },
    parsing::{MizarParserSeam, ParserSeam},
    preprocess::{
        ImportPrescanDiagnosticCode, PreprocessDiagnosticKind, SourcePreprocessDiagnosticCode,
    },
    source::{
        FrontendSourceLoader, SourceUnit, SourceUnitLoader, SourceUnitRequest,
        diagnostic_file_path, source_unit_from_loaded,
    },
};
use mizar_ir::{
    identity::{
        NamedInputHash, OutputKind, PipelinePhase as IrPipelinePhase, WorkUnit as IrWorkUnit,
    },
    publisher::{OutputAuthorization, OutputOrigin, PublicationTarget, PublishOutputInput},
    storage::{BlobDecodeError, BlobDecoder, IrSideTables, SchemaVersion, SideTableRecord},
};
use mizar_session::{
    DiskSourceLoader, Hash, PackageId, SourceInput, SourceLoadError, SourceOrigin,
    SourceOriginInput, module_work_unit_label,
};

use crate::registry::{
    PhaseCacheContext, PhaseCacheIntent, PhaseDescriptor, PhaseExecutionContext, PhaseInput,
    PhaseOwner, PhaseResult, PhaseService, PhaseStatus,
};

fn source_publication_route(origin: &SourceOrigin) -> (OutputOrigin, PublicationTarget) {
    match origin {
        SourceOrigin::OpenBuffer { .. } => {
            (OutputOrigin::OpenBuffer, PublicationTarget::InternalOnly)
        }
        _ => (
            OutputOrigin::PackageSource,
            PublicationTarget::CurrentPackage,
        ),
    }
}

pub(crate) fn source_input_hash(version: &mizar_session::SourceVersion) -> Hash {
    SourceUnitCacheKey {
        version: Arc::from(SOURCE_UNIT_CACHE_KEY_VERSION),
        package_id: version.package_id.clone(),
        module_path: version.module_path.clone(),
        normalized_path: version.normalized_path.clone(),
        source_hash: version.source_hash,
        edition: version.edition.clone(),
    }
    .stable_hash()
}

fn unique<T>(mut items: impl Iterator<Item = T>) -> Option<T> {
    let item = items.next()?;
    items.next().is_none().then_some(item)
}

pub(crate) fn safe_relative(path: &str) -> bool {
    path == "."
        || (!path.is_empty()
            && !path.contains(['\\', ':'])
            && path
                .split('/')
                .all(|part| !part.is_empty() && part != "." && part != ".."))
}

fn blocking() -> PhaseResult {
    PhaseResult {
        status: PhaseStatus::Blocking,
        ..PhaseResult::complete()
    }
}

mod checking;
mod frontend;
mod resolver;
mod source;
pub use checking::{
    native_theorem_metadata_from_storage_bytes, native_theorem_metadata_storage_bytes,
    native_theorem_registration_closure_from_storage_bytes,
    native_theorem_registration_closure_storage_bytes, native_theorem_typed_ast_from_storage_bytes,
    native_theorem_typed_ast_storage_bytes,
};
#[cfg(test)]
pub(crate) use source::imported_native_frontend_cache_tests::ImportedFixture;
mod core;
pub use core::{
    native_algorithm_core_from_storage_bytes, native_algorithm_core_storage_bytes,
    native_theorem_core_from_storage_bytes, native_theorem_core_semantic_payload,
    native_theorem_core_storage_bytes, native_theorem_preparation_from_storage_bytes,
    native_theorem_preparation_storage_bytes,
};
mod vc;
pub use vc::{
    native_algorithm_vc_from_storage_bytes, native_algorithm_vc_storage_bytes,
    native_theorem_discharge_from_storage_bytes, native_theorem_discharge_storage_bytes,
    native_theorem_vc_from_storage_bytes, native_theorem_vc_storage_bytes,
};
mod candidate;
pub use candidate::{
    native_theorem_candidate_check_from_storage_bytes,
    native_theorem_candidate_check_storage_bytes, native_theorem_candidate_from_storage_bytes,
    native_theorem_candidate_storage_bytes,
};
mod artifact;

pub(crate) use artifact::remove_native_artifact_staging;
pub use artifact::{PublishedSourceMetadata, SnapshotPublication};
pub(crate) use artifact::{
    SourceTheoremArtifactCommitService, SourceTheoremReflexivityArtifactCommitService,
};
pub(crate) use candidate::{SourceTheoremAtpSolveService, SourceTheoremCandidateCheckService};
pub(crate) use checking::{
    SourceTheoremOverloadResolutionService, SourceTheoremRegistrationResolutionService,
    SourceTheoremSignatureCollectionService, SourceTheoremTypeCheckingService,
};
pub(crate) use core::{SourceTheoremAlgorithmPreparationService, SourceTheoremElaborationService};
pub(crate) use frontend::FrontendService;
pub use frontend::{
    NATIVE_EXPORT_SEMICOLON_ENCODING, native_export_semicolon_command,
    native_export_semicolon_context,
};
pub(crate) use resolver::SourceTheoremModuleResolverService;
use source::DependencyLexicalProvider;
pub(crate) use source::SourceLoadService;
pub(crate) use vc::{
    SourceTheoremReflexivityKernelCheckService, SourceTheoremVcDischargeService,
    SourceTheoremVcGenerationService,
};

mod framing;
mod publication;
use publication::publish_native_output;
