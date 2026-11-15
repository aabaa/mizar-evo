//! Shared scheduler phase and module work-unit labels.

/// A scheduler pipeline phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PipelinePhase {
    PackageResolve,
    SourceLoad,
    Frontend,
    ModuleResolve,
    SignatureCollection,
    TypeChecking,
    RegistrationResolution,
    OverloadResolution,
    Elaboration,
    AlgorithmPreparation,
    VcGenerate,
    VcDischarge,
    AtpSolve,
    BackendRun,
    KernelCheck,
    ArtifactCommit,
    DocumentationExtract,
}

impl PipelinePhase {
    /// Returns the phase identity label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PackageResolve => "PackageResolve",
            Self::SourceLoad => "SourceLoad",
            Self::Frontend => "Frontend",
            Self::ModuleResolve => "ModuleResolve",
            Self::SignatureCollection => "SignatureCollection",
            Self::TypeChecking => "TypeChecking",
            Self::RegistrationResolution => "RegistrationResolution",
            Self::OverloadResolution => "OverloadResolution",
            Self::Elaboration => "Elaboration",
            Self::AlgorithmPreparation => "AlgorithmPreparation",
            Self::VcGenerate => "VcGenerate",
            Self::VcDischarge => "VcDischarge",
            Self::AtpSolve => "AtpSolve",
            Self::BackendRun => "BackendRun",
            Self::KernelCheck => "KernelCheck",
            Self::ArtifactCommit => "ArtifactCommit",
            Self::DocumentationExtract => "DocumentationExtract",
        }
    }

    /// Returns the phase spelling used by CLI output.
    pub fn as_cli_str(self) -> &'static str {
        match self {
            Self::PackageResolve => "package_resolve",
            Self::SourceLoad => "source_load",
            Self::Frontend => "frontend",
            Self::ModuleResolve => "module_resolve",
            Self::SignatureCollection => "signature_collection",
            Self::TypeChecking => "type_checking",
            Self::RegistrationResolution => "registration_resolution",
            Self::OverloadResolution => "overload_resolution",
            Self::Elaboration => "elaboration",
            Self::AlgorithmPreparation => "algorithm_preparation",
            Self::VcGenerate => "vc_generate",
            Self::VcDischarge => "vc_discharge",
            Self::AtpSolve => "atp_solve",
            Self::BackendRun => "backend_run",
            Self::KernelCheck => "kernel_check",
            Self::ArtifactCommit => "artifact_commit",
            Self::DocumentationExtract => "documentation_extract",
        }
    }
}

/// Formats a module work-unit label using the canonical quoted strings.
pub fn module_work_unit_label(package: &str, module: &str) -> String {
    format!("{package:?}:{module:?}")
}
