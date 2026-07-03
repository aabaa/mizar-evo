use mizar_session::Hash;
use mizar_test::toml_lite::TomlValue;
use mizar_test::{
    CoverageShape, DeclarationSymbolCaseStatus, ExpectedOutcome, HarnessError, ParallelismProfile,
    ParseOnlyCaseStatus, PipelinePhase, RequirementStatus, SnapshotBaselineError,
    SnapshotBaselineMismatch, SnapshotBaselineStatus, SnapshotError, SnapshotKind,
    SnapshotUpdateMode, SnapshotUpdateReason, Stage, TestKind, TestProfile, ToolchainInfo,
    TypeElaborationCaseStatus, ValidationMode, ValidationSeverity,
};

#[test]
fn public_enum_consumers_match_with_wildcards() {
    assert_eq!(validation_severity(ValidationSeverity::Error), "error");
    assert_eq!(test_kind(TestKind::Pass), "pass");
    assert_eq!(expected_outcome(ExpectedOutcome::Pass), "pass");
    assert_eq!(pipeline_phase(PipelinePhase::Parse), "parse");
    assert_eq!(test_profile(TestProfile::Fast), "fast");
    assert_eq!(validation_mode(ValidationMode::Metadata), "metadata");
    assert_eq!(
        harness_error(HarnessError::Infrastructure("io".to_owned())),
        "infrastructure"
    );
    assert_eq!(parse_status(ParseOnlyCaseStatus::Passed), "passed");
    assert_eq!(
        declaration_status(DeclarationSymbolCaseStatus::Failed),
        "failed"
    );
    assert_eq!(
        type_elaboration_status(TypeElaborationCaseStatus::Passed),
        "passed"
    );
    assert_eq!(snapshot_kind(SnapshotKind::SurfaceAst), "surface_ast");
    assert_eq!(
        parallelism(ParallelismProfile::Parallel { workers: 2 }),
        "parallel"
    );
    assert_eq!(
        snapshot_update_reason(SnapshotUpdateReason::SchemaChange),
        "schema"
    );
    assert_eq!(
        snapshot_update_mode(SnapshotUpdateMode::Update {
            reason: SnapshotUpdateReason::SemanticBehaviorChange,
        }),
        "update"
    );
    assert_eq!(
        snapshot_baseline_status(SnapshotBaselineStatus::Created),
        "created"
    );
    assert_eq!(
        snapshot_baseline_error(SnapshotBaselineError::MissingBaseline {
            path: "snapshots/missing.snap".into(),
        }),
        "missing"
    );
    assert_eq!(snapshot_error(SnapshotError::EmptyTestId), "empty-id");
    assert_eq!(stage(Stage::Lexical), "lexical");
    assert_eq!(toml_value(TomlValue::Boolean(true)), "bool");
    assert_eq!(requirement_status(RequirementStatus::Covered), "covered");
    assert_eq!(coverage_shape(CoverageShape::PassAndFail), "pass-and-fail");
}

fn validation_severity(value: ValidationSeverity) -> &'static str {
    match value {
        ValidationSeverity::Error => "error",
        ValidationSeverity::Warning => "warning",
        _ => "unknown",
    }
}

fn test_kind(value: TestKind) -> &'static str {
    match value {
        TestKind::Pass => "pass",
        TestKind::Fail => "fail",
        TestKind::Snapshot => "snapshot",
        TestKind::Generated => "generated",
        TestKind::FuzzSeed => "fuzz",
        TestKind::PropertySeed => "property",
        _ => "unknown",
    }
}

fn expected_outcome(value: ExpectedOutcome) -> &'static str {
    match value {
        ExpectedOutcome::Pass => "pass",
        ExpectedOutcome::Fail => "fail",
        ExpectedOutcome::Snapshot => "snapshot",
        ExpectedOutcome::MetadataOnly => "metadata",
        _ => "unknown",
    }
}

fn pipeline_phase(value: PipelinePhase) -> &'static str {
    match value {
        PipelinePhase::Lex => "lex",
        PipelinePhase::Parse => "parse",
        PipelinePhase::Resolve => "resolve",
        PipelinePhase::TypeCheck => "type",
        PipelinePhase::Elaboration => "elaboration",
        PipelinePhase::ClusterResolution => "cluster",
        PipelinePhase::OverloadResolution => "overload",
        PipelinePhase::StatementCheck => "statement",
        PipelinePhase::VcGeneration => "vc",
        PipelinePhase::Verification => "verification",
        PipelinePhase::CertificateCheck => "certificate",
        PipelinePhase::KernelCheck => "kernel",
        _ => "unknown",
    }
}

fn test_profile(value: TestProfile) -> &'static str {
    match value {
        TestProfile::Fast => "fast",
        TestProfile::Full => "full",
        TestProfile::Stress => "stress",
        TestProfile::FuzzRegression => "fuzz",
        TestProfile::SnapshotUpdate => "snapshot-update",
        _ => "unknown",
    }
}

fn validation_mode(value: ValidationMode) -> &'static str {
    match value {
        ValidationMode::Metadata => "metadata",
        ValidationMode::Development => "development",
        ValidationMode::Release => "release",
        _ => "unknown",
    }
}

fn harness_error(value: HarnessError) -> &'static str {
    match value {
        HarnessError::Infrastructure(_) => "infrastructure",
        _ => "unknown",
    }
}

fn parse_status(value: ParseOnlyCaseStatus) -> &'static str {
    match value {
        ParseOnlyCaseStatus::Passed => "passed",
        ParseOnlyCaseStatus::Failed => "failed",
        _ => "unknown",
    }
}

fn declaration_status(value: DeclarationSymbolCaseStatus) -> &'static str {
    match value {
        DeclarationSymbolCaseStatus::Passed => "passed",
        DeclarationSymbolCaseStatus::Failed => "failed",
        _ => "unknown",
    }
}

fn type_elaboration_status(value: TypeElaborationCaseStatus) -> &'static str {
    match value {
        TypeElaborationCaseStatus::Passed => "passed",
        TypeElaborationCaseStatus::Failed => "failed",
        _ => "unknown",
    }
}

fn snapshot_kind(value: SnapshotKind) -> &'static str {
    match value {
        SnapshotKind::SurfaceAst => "surface_ast",
        SnapshotKind::TypedAst => "typed_ast",
        SnapshotKind::CoreIr => "core_ir",
        SnapshotKind::VcIr => "vc_ir",
        SnapshotKind::SatClauses => "sat_clauses",
        SnapshotKind::ProofCertificate => "proof_certificate",
        SnapshotKind::VerifiedArtifact => "verified_artifact",
        SnapshotKind::DependencySlice => "dependency_slice",
        SnapshotKind::DependencyFingerprint => "dependency_fingerprint",
        SnapshotKind::FailureRecord => "failure_record",
        _ => "unknown",
    }
}

fn parallelism(value: ParallelismProfile) -> &'static str {
    match value {
        ParallelismProfile::Sequential => "sequential",
        ParallelismProfile::Parallel { .. } => "parallel",
        _ => "unknown",
    }
}

fn snapshot_update_reason(value: SnapshotUpdateReason) -> &'static str {
    match value {
        SnapshotUpdateReason::SchemaChange => "schema",
        SnapshotUpdateReason::DiagnosticContractChange => "diagnostic",
        SnapshotUpdateReason::SemanticBehaviorChange => "semantic",
        SnapshotUpdateReason::FuzzPropertyReproducer => "fuzz",
        _ => "unknown",
    }
}

fn snapshot_update_mode(value: SnapshotUpdateMode) -> &'static str {
    match value {
        SnapshotUpdateMode::VerifyOnly => "verify",
        SnapshotUpdateMode::Update { .. } => "update",
        _ => "unknown",
    }
}

fn snapshot_baseline_status(value: SnapshotBaselineStatus) -> &'static str {
    match value {
        SnapshotBaselineStatus::Matched => "matched",
        SnapshotBaselineStatus::Created => "created",
        SnapshotBaselineStatus::Updated => "updated",
        _ => "unknown",
    }
}

fn snapshot_baseline_error(value: SnapshotBaselineError) -> &'static str {
    match value {
        SnapshotBaselineError::Snapshot(_) => "snapshot",
        SnapshotBaselineError::InvalidBaselinePath { .. } => "invalid-path",
        SnapshotBaselineError::Io { .. } => "io",
        SnapshotBaselineError::MissingBaseline { .. } => "missing",
        SnapshotBaselineError::Mismatch { .. } => "mismatch",
        _ => "unknown",
    }
}

fn snapshot_error(value: SnapshotError) -> &'static str {
    match value {
        SnapshotError::EmptyTestId => "empty-id",
        SnapshotError::EmptyToolchainName => "empty-toolchain",
        SnapshotError::EmptyToolchainVersion => "empty-version",
        SnapshotError::EmptyMetadataKey => "empty-metadata",
        SnapshotError::ParallelWorkerCountZero => "zero-workers",
        SnapshotError::LocalPath { .. } => "local-path",
        SnapshotError::StaleContentHash { .. } => "stale-hash",
        _ => "unknown",
    }
}

fn stage(value: Stage) -> &'static str {
    match value {
        Stage::Lexical => "lexical",
        Stage::ParseOnly => "parse",
        Stage::DeclarationSymbol => "resolve",
        Stage::TypeElaboration => "type",
        Stage::FormulaStatement => "formula",
        Stage::ProofVerification => "proof",
        Stage::AdvancedSemantics => "advanced",
        _ => "unknown",
    }
}

fn toml_value(value: TomlValue) -> &'static str {
    match value {
        TomlValue::String(_) => "string",
        TomlValue::Integer(_) => "integer",
        TomlValue::Boolean(_) => "bool",
        TomlValue::Array(_) => "array",
        _ => "unknown",
    }
}

fn requirement_status(value: RequirementStatus) -> &'static str {
    match value {
        RequirementStatus::Planned => "planned",
        RequirementStatus::Covered => "covered",
        RequirementStatus::Partial => "partial",
        RequirementStatus::Deferred => "deferred",
        RequirementStatus::Obsolete => "obsolete",
        _ => "unknown",
    }
}

fn coverage_shape(value: CoverageShape) -> &'static str {
    match value {
        CoverageShape::None => "none",
        CoverageShape::Pass => "pass",
        CoverageShape::Fail => "fail",
        CoverageShape::PassAndFail => "pass-and-fail",
        CoverageShape::Diagnostic => "diagnostic",
        CoverageShape::Snapshot => "snapshot",
        CoverageShape::Property => "property",
        CoverageShape::ManualReview => "manual",
        _ => "unknown",
    }
}

#[test]
fn public_enum_payload_variants_are_downstream_constructible() {
    let _ = SnapshotBaselineError::Snapshot(SnapshotError::EmptyTestId);
    let _ = SnapshotBaselineError::InvalidBaselinePath {
        path: "snapshots/invalid.snap".into(),
    };
    let _ = SnapshotBaselineError::Io {
        path: "snapshots/io.snap".into(),
        message: "io".to_owned(),
    };
    let _ = SnapshotBaselineError::Mismatch {
        path: "snapshots/mismatch.snap".into(),
        mismatch: Box::new(SnapshotBaselineMismatch {
            expected_hash: Some(Hash::from_bytes([1; Hash::BYTE_LEN])),
            actual_hash: Hash::from_bytes([2; Hash::BYTE_LEN]),
            first_difference: None,
        }),
    };
    let _ = SnapshotError::LocalPath {
        token: "/tmp/local".to_owned(),
    };
    let _ = SnapshotError::StaleContentHash {
        stored: Hash::from_bytes([3; Hash::BYTE_LEN]),
        recomputed: Hash::from_bytes([4; Hash::BYTE_LEN]),
    };
    let _ = ToolchainInfo::new("mizar-test", "task12");
}
