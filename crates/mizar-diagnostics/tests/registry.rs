use std::str::FromStr;

use mizar_diagnostics::registry::{
    BUILTIN_DESCRIPTORS, DiagnosticCode, DiagnosticDescriptor, DiagnosticRegistry,
    DiagnosticSeverity, DiagnosticStatus, PhaseFamily, RegistryValidationError,
    validate_descriptors, validate_registry_compatibility,
};

const EXPECTED_BUILTIN_CODES: &[&str] = &[
    "E0001", "E0002", "E0003", "E0010", "E0011", "E0012", "E0101", "E0102", "E0103", "E0110",
    "E0120", "E0121", "E0122", "E0201", "E0202", "E0203", "E0204", "E0301", "E0302", "E0303",
    "E0310", "E0320", "E0321", "E0350", "E0351", "E0352", "E0353", "E0401", "E0410", "E0411",
    "E0420", "E0421", "E0422", "E0423", "E0424", "E0425", "E0426", "E0430", "W0001", "W0002",
    "W0003", "W0010", "W0101", "W0102", "W0103", "W0201", "W0202", "W0210", "W0301", "W0302",
    "W0303", "W0304", "W0305",
];

const E0002: DiagnosticCode = match DiagnosticCode::from_parts(DiagnosticSeverity::Error, 2) {
    Ok(code) => code,
    Err(_) => panic!("E0002 is valid"),
};
const REPLACEMENT_CODES: &[DiagnosticCode] = &[E0002];

#[test]
fn diagnostic_code_parsing_and_ranges_are_stable() {
    let code = DiagnosticCode::from_str("E0201").expect("valid code");
    assert_eq!(code.to_string(), "E0201");
    assert_eq!(code.severity(), DiagnosticSeverity::Error);
    assert_eq!(code.number(), 201);
    assert_eq!(code.phase_family(), Some(PhaseFamily::Resolution));
    assert_eq!(code.default_severity(), Some(DiagnosticSeverity::Error));

    let info = DiagnosticCode::from_str("I0001").expect("reserved info code shape");
    assert_eq!(info.phase_family(), Some(PhaseFamily::Info));
    assert_eq!(info.default_severity(), Some(DiagnosticSeverity::Info));

    let out_of_range = DiagnosticCode::from_str("E0600").expect("well-formed code");
    assert_eq!(out_of_range.phase_family(), None);
    assert!(DiagnosticCode::from_str("E01").is_err());
    assert!(DiagnosticCode::from_str("X0101").is_err());
    assert!(DiagnosticCode::from_str("E01A1").is_err());
    assert!(DiagnosticCode::from_parts(DiagnosticSeverity::Error, 10_000).is_err());
}

#[test]
fn builtin_registry_locks_allocated_codes() {
    let registry = DiagnosticRegistry::builtin();
    let codes = registry
        .descriptors()
        .iter()
        .map(|descriptor| descriptor.code.to_string())
        .collect::<Vec<_>>();

    assert_eq!(codes, EXPECTED_BUILTIN_CODES);
    assert_eq!(BUILTIN_DESCRIPTORS.len(), EXPECTED_BUILTIN_CODES.len());
    assert!(
        registry
            .lookup(DiagnosticCode::from_str("I0001").expect("valid info code"))
            .is_none()
    );
}

#[test]
fn registry_constructor_locks_builtin_compatibility() {
    assert!(matches!(
        DiagnosticRegistry::new(&[]),
        Err(RegistryValidationError::MissingCode { code })
            if code == BUILTIN_DESCRIPTORS[0].code
    ));

    let mut changed = BUILTIN_DESCRIPTORS.to_vec();
    changed[0] = DiagnosticDescriptor {
        meaning_key: "syntax.reused_for_other_meaning",
        semantic_name: "syntax.reused_for_other_meaning",
        ..changed[0]
    };
    assert!(matches!(
        DiagnosticRegistry::new(&changed),
        Err(RegistryValidationError::MeaningKeyChanged { .. })
    ));
}

#[test]
fn lookup_metadata_round_trips_without_message_text_identity() {
    let registry = DiagnosticRegistry::builtin();
    let code = DiagnosticCode::from_str("W0303").expect("valid code");
    let descriptor = registry.lookup(code).expect("allocated descriptor");

    assert_eq!(descriptor.code, code);
    assert_eq!(descriptor.meaning_key, "compat.overload_resolution_shift");
    assert_eq!(descriptor.semantic_name, "compat.overload_resolution_shift");
    assert_eq!(descriptor.default_severity, DiagnosticSeverity::Warning);
    assert_eq!(descriptor.phase_family, PhaseFamily::CompatibilityWarning);
    assert_eq!(
        descriptor.summary,
        "Registration, redefinition, or conditional-cluster change may shift \
         overload/refinement resolution (heuristic MAJOR)"
    );
    assert_eq!(
        descriptor.doc_url,
        "doc/spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference"
    );
    assert_eq!(descriptor.status, DiagnosticStatus::Active);
    assert_eq!(descriptor.since, "spec-22.7-v1");
    assert!(descriptor.aliases.is_empty());
    assert!(descriptor.replacement_codes.is_empty());

    assert_eq!(
        registry
            .lookup_semantic_name(
                PhaseFamily::CompatibilityWarning,
                "compat.overload_resolution_shift"
            )
            .map(|descriptor| descriptor.code),
        Some(code)
    );
    assert!(
        registry
            .lookup_semantic_name(PhaseFamily::CompatibilityWarning, "message text changed")
            .is_none()
    );
}

#[test]
fn descriptor_validation_rejects_range_severity_and_alias_gaps() {
    assert_eq!(
        validate_descriptors(&[descriptor(
            "E0600",
            "unknown.future",
            PhaseFamily::Algorithm,
            DiagnosticSeverity::Error,
        )]),
        Err(RegistryValidationError::CodeOutsideDefinedRange {
            code: DiagnosticCode::from_str("E0600").expect("valid code"),
        })
    );
    assert_eq!(
        validate_descriptors(&[descriptor(
            "I0001",
            "info.thesis",
            PhaseFamily::Info,
            DiagnosticSeverity::Info,
        )]),
        Err(RegistryValidationError::ReservedInfoCodeAllocated {
            code: DiagnosticCode::from_str("I0001").expect("valid code"),
        })
    );

    assert!(matches!(
        validate_descriptors(&[descriptor(
            "E0101",
            "type.mismatch",
            PhaseFamily::Syntax,
            DiagnosticSeverity::Error,
        )]),
        Err(RegistryValidationError::PhaseFamilyMismatch { .. })
    ));

    assert!(matches!(
        validate_descriptors(&[descriptor(
            "W0301",
            "compat.breaking_change",
            PhaseFamily::CompatibilityWarning,
            DiagnosticSeverity::Error,
        )]),
        Err(RegistryValidationError::SeverityMismatch { .. })
    ));

    let first = descriptor_with_aliases("E0001", "syntax.new_name", &["syntax.old_name"]);
    let second = descriptor(
        "E0002",
        "syntax.old_name",
        PhaseFamily::Syntax,
        DiagnosticSeverity::Error,
    );
    assert!(matches!(
        validate_descriptors(&[first, second]),
        Err(RegistryValidationError::NameCollision { .. })
    ));

    let first = descriptor_with_aliases("E0001", "syntax.first", &["syntax.old_name"]);
    let second = descriptor_with_aliases("E0002", "syntax.second", &["syntax.old_name"]);
    assert!(matches!(
        validate_descriptors(&[first, second]),
        Err(RegistryValidationError::NameCollision { .. })
    ));

    let duplicate = descriptor(
        "E0001",
        "syntax.other",
        PhaseFamily::Syntax,
        DiagnosticSeverity::Error,
    );
    assert!(matches!(
        validate_descriptors(&[first, duplicate]),
        Err(RegistryValidationError::DuplicateCode { .. })
    ));
}

#[test]
fn compatibility_validation_rejects_code_reuse_and_disappearance() {
    let baseline = [descriptor(
        "E0001",
        "syntax.unexpected_token",
        PhaseFamily::Syntax,
        DiagnosticSeverity::Error,
    )];
    let reused = [DiagnosticDescriptor {
        meaning_key: "syntax.different_meaning",
        semantic_name: "syntax.different_meaning",
        ..baseline[0]
    }];

    assert!(matches!(
        validate_registry_compatibility(&baseline, &reused),
        Err(RegistryValidationError::MeaningKeyChanged { .. })
    ));
    let summary_changed = [DiagnosticDescriptor {
        summary: "summary text may improve without changing identity",
        ..baseline[0]
    }];
    assert!(validate_registry_compatibility(&baseline, &summary_changed).is_ok());
    assert_eq!(
        validate_registry_compatibility(&baseline, &[]),
        Err(RegistryValidationError::MissingCode {
            code: baseline[0].code,
        })
    );
}

#[test]
fn retirement_is_final_and_requires_metadata() {
    let active = descriptor(
        "E0001",
        "syntax.unexpected_token",
        PhaseFamily::Syntax,
        DiagnosticSeverity::Error,
    );
    let retired = DiagnosticDescriptor {
        status: DiagnosticStatus::Retired,
        retired_since: Some("task-test"),
        ..active
    };
    assert!(validate_registry_compatibility(&[active], &[retired]).is_ok());

    assert_eq!(
        validate_descriptors(&[DiagnosticDescriptor {
            status: DiagnosticStatus::Retired,
            retired_since: None,
            ..active
        }]),
        Err(RegistryValidationError::RetiredWithoutRetiredSince { code: active.code })
    );
    assert_eq!(
        validate_descriptors(&[DiagnosticDescriptor {
            retired_since: Some("task-test"),
            ..active
        }]),
        Err(RegistryValidationError::ActiveWithRetiredSince { code: active.code })
    );
    assert_eq!(
        validate_descriptors(&[DiagnosticDescriptor {
            replacement_codes: REPLACEMENT_CODES,
            ..active
        }]),
        Err(RegistryValidationError::ActiveWithReplacementCodes { code: active.code })
    );
    assert_eq!(
        validate_registry_compatibility(&[retired], &[active]),
        Err(RegistryValidationError::RetiredCodeReactivated { code: active.code })
    );
}

#[test]
fn semantic_renames_require_aliases_and_preserve_lookup_determinism() {
    let baseline = [descriptor(
        "E0001",
        "syntax.unexpected_token",
        PhaseFamily::Syntax,
        DiagnosticSeverity::Error,
    )];
    let renamed_without_alias = [DiagnosticDescriptor {
        semantic_name: "syntax.token_unexpected",
        ..baseline[0]
    }];
    let renamed_with_alias = [DiagnosticDescriptor {
        semantic_name: "syntax.token_unexpected",
        aliases: &["syntax.unexpected_token"],
        ..baseline[0]
    }];

    assert!(matches!(
        validate_registry_compatibility(&baseline, &renamed_without_alias),
        Err(RegistryValidationError::SemanticRenameWithoutAlias { .. })
    ));
    assert!(validate_registry_compatibility(&baseline, &renamed_with_alias).is_ok());

    let second_generation = [DiagnosticDescriptor {
        aliases: &[],
        ..renamed_with_alias[0]
    }];
    assert!(matches!(
        validate_registry_compatibility(&renamed_with_alias, &second_generation),
        Err(RegistryValidationError::SemanticRenameWithoutAlias { .. })
    ));

    assert!(
        renamed_with_alias[0]
            .aliases
            .contains(&"syntax.unexpected_token")
    );
}

fn descriptor(
    code: &str,
    semantic_name: &'static str,
    phase_family: PhaseFamily,
    severity: DiagnosticSeverity,
) -> DiagnosticDescriptor {
    DiagnosticDescriptor {
        code: DiagnosticCode::from_str(code).expect("test code is well-formed"),
        meaning_key: semantic_name,
        semantic_name,
        default_severity: severity,
        phase_family,
        summary: "test summary",
        doc_url: "doc/spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference",
        status: DiagnosticStatus::Active,
        since: "test",
        retired_since: None,
        replacement_codes: &[],
        aliases: &[],
    }
}

fn descriptor_with_aliases(
    code: &str,
    semantic_name: &'static str,
    aliases: &'static [&'static str],
) -> DiagnosticDescriptor {
    DiagnosticDescriptor {
        aliases,
        ..descriptor(
            code,
            semantic_name,
            PhaseFamily::Syntax,
            DiagnosticSeverity::Error,
        )
    }
}
