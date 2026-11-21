use std::str::FromStr;

use mizar_diagnostics::failure_record::{
    DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput, DiagnosticFreshness,
    DiagnosticHandle, DiagnosticId, DiagnosticPrimaryLocation, DiagnosticRecord, DiagnosticSpan,
    FailureCategory, PipelinePhase,
};
use mizar_diagnostics::registry::{
    BUILTIN_DESCRIPTORS, DiagnosticCode, DiagnosticDescriptor, DiagnosticRegistry,
    DiagnosticSeverity, DiagnosticStatus, PhaseFamily, RegistryValidationError,
    validate_descriptors, validate_registry_compatibility,
};
use mizar_session::{BuildSnapshotId, InMemorySessionIdAllocator, SessionIdAllocator, SourceRange};

const EXPECTED_BUILTIN_CODES: &[&str] = &[
    "E0001", "E0002", "E0003", "E0010", "E0011", "E0012", "E0013", "E0014", "E0015", "E0016",
    "E0017", "E0018", "E0019", "E0020", "E0021", "E0022", "E0023", "E0024", "E0025", "E0026",
    "E0027", "E0028", "E0029", "E0030", "E0031", "E0032", "E0033", "E0034", "E0035", "E0036",
    "E0037", "E0038", "E0039", "E0040", "E0041", "E0042", "E0043", "E0044", "E0045", "E0046",
    "E0047", "E0048", "E0049", "E0050", "E0051", "E0052", "E0101", "E0102", "E0103", "E0104",
    "E0110", "E0111", "E0112", "E0120", "E0121", "E0122", "E0201", "E0202", "E0203", "E0204",
    "E0206", "E0220", "E0221", "E0222", "E0223", "E0224", "E0225", "E0301", "E0302", "E0303",
    "E0310", "E0320", "E0321", "E0332", "E0333", "E0350", "E0351", "E0352", "E0353", "E0401",
    "E0410", "E0411", "E0420", "E0421", "E0422", "E0423", "E0424", "E0425", "E0426", "E0430",
    "E0600", "E0601", "E0602", "E0603", "W0001", "W0002", "W0003", "W0010", "W0101", "W0102",
    "W0103", "W0201", "W0202", "W0210", "W0301", "W0302", "W0303", "W0304", "W0305", "W0400",
    "W0401", "W0402", "W0403", "W0404", "W0405", "W0406", "W0407", "W0408", "W0409", "W0410",
    "W0411", "W0412", "W0413", "W0414", "W0415",
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

    let out_of_range = DiagnosticCode::from_str("E0700").expect("well-formed code");
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
fn source_load_descriptors_lock_codes_metadata_and_family() {
    let registry = DiagnosticRegistry::builtin();
    for (code, name, summary) in [
        (
            "E0600",
            "source.load_failed",
            "Other source-loading failure",
        ),
        (
            "E0601",
            "source.invalid_utf8",
            "Source bytes are not valid UTF-8",
        ),
        (
            "E0602",
            "source.unreadable_file",
            "Source file cannot be read",
        ),
        (
            "E0603",
            "source.outside_package_root",
            "Source path escapes the package root",
        ),
    ] {
        let code = DiagnosticCode::from_str(code).expect("allocated source-load code");
        let descriptor = registry.lookup(code).expect("source-load descriptor");
        assert_eq!(descriptor.meaning_key, name);
        assert_eq!(descriptor.semantic_name, name);
        assert_eq!(descriptor.summary, summary);
        assert_eq!(descriptor.default_severity, DiagnosticSeverity::Error);
        assert_eq!(descriptor.phase_family, PhaseFamily::SourceLoad);
        assert_eq!(descriptor.since, "spec-22-source-load-v1");
    }
}

#[test]
fn documentation_descriptors_lock_the_five_attachment_meanings() {
    let registry = DiagnosticRegistry::builtin();
    let expected = [
        (
            "orphan",
            "No eligible module or adjacent named-item candidate for the documentation block.",
        ),
        (
            "unresolved_target",
            "Adjacent documentation target lacks supported verified identity, label or target boundary.",
        ),
        (
            "ambiguous",
            "Multiple identities or contradictory candidate/range associations remain for the documentation block.",
        ),
        (
            "unverified_target",
            "Documentation target is invalid or unverified.",
        ),
        (
            "unresolved_coordinates",
            "Required documentation association coordinates are absent or cannot be related to the verified target.",
        ),
    ];
    let documentation = registry
        .descriptors()
        .iter()
        .filter(|row| row.semantic_name.starts_with("documentation.attachment."))
        .collect::<Vec<_>>();
    assert_eq!(documentation.len(), expected.len());
    for (index, (suffix, summary)) in expected.into_iter().enumerate() {
        let code =
            DiagnosticCode::from_parts(DiagnosticSeverity::Warning, 400 + index as u16).unwrap();
        let row = registry
            .lookup(code)
            .expect("allocated Documentation warning");
        assert_eq!(
            row.semantic_name,
            format!("documentation.attachment.{suffix}")
        );
        assert_eq!(row.meaning_key, row.semantic_name);
        assert_eq!(row.summary, summary);
        assert_eq!(row.phase_family, PhaseFamily::Documentation);
        assert_eq!(row.default_severity, DiagnosticSeverity::Warning);
        assert_eq!(row.status, DiagnosticStatus::Active);
        assert_eq!(row.retired_since, None);
        assert!(row.aliases.is_empty());
        assert!(row.replacement_codes.is_empty());
        assert_eq!(row.since, "spec-22-documentation-v1");
        assert_eq!(
            row.doc_url,
            "doc/spec/en/22.error_handling_and_diagnostics.md#2268-documentation-attachment-warnings"
        );
    }
    for number in 400..=499 {
        let code = DiagnosticCode::from_parts(DiagnosticSeverity::Warning, number).unwrap();
        assert_eq!(code.phase_family(), Some(PhaseFamily::Documentation));
        assert_eq!(code.default_severity(), Some(DiagnosticSeverity::Warning));
        assert_eq!(registry.lookup(code).is_some(), number <= 415);
    }
    for text in ["W0500", "W0700"] {
        assert_eq!(DiagnosticCode::from_str(text).unwrap().phase_family(), None);
    }
    assert_eq!(
        DiagnosticCode::from_str("E0401").unwrap().phase_family(),
        Some(PhaseFamily::Logic)
    );
    assert_eq!(
        DiagnosticCode::from_str("W0399").unwrap().phase_family(),
        Some(PhaseFamily::CompatibilityWarning)
    );
}

#[test]
fn documentation_markdown_tag_descriptors_lock_approved_meanings() {
    let registry = DiagnosticRegistry::builtin();
    let expected = [
        (
            "documentation.tag.missing_slot",
            "A known tag lacks its required name, reference or version slot: `@param`, `@see`, `@since`, `@deprecated` respectively.",
        ),
        (
            "documentation.tag.missing_body",
            "A required explanatory body is empty for `@returns`, `@requires`, `@ensures`, or after `@deprecated version`.",
        ),
        (
            "documentation.tag.duplicate",
            "A later known-tag occurrence violates the approved same-target raw duplicate rule: same `@param` name, repeated singleton (`@returns`, `@since`, `@deprecated`), or identical raw tag/payload for repeatable `@requires`, `@ensures`, `@see`.",
        ),
        (
            "documentation.tag.metadata_mismatch",
            "Available authoritative metadata establishes that a named parameter is absent from a complete list, or that `@returns` targets an item with no return value.",
        ),
        (
            "documentation.markdown.unmatched_math_delimiter",
            "An eligible math opener has no valid matching closer under the approved protected-boundary rules; preserve its literal spelling/CommonMark parse.",
        ),
        (
            "documentation.markdown.empty_display_math",
            "A paired display-math candidate has only W in its interior; preserve its full literal spelling without a math node.",
        ),
        (
            "documentation.markdown.conversion_failure",
            "A real inability to derive or map the Markdown/tag presentation view requires raw fallback for the affected block.",
        ),
    ];
    for (index, (name, summary)) in expected.into_iter().enumerate() {
        let code =
            DiagnosticCode::from_parts(DiagnosticSeverity::Warning, 405 + index as u16).unwrap();
        let row = registry
            .lookup(code)
            .expect("allocated Markdown/tag warning");
        assert_eq!(row.semantic_name, name);
        assert_eq!(row.meaning_key, name);
        assert_eq!(row.summary, summary);
        assert_eq!(row.phase_family, PhaseFamily::Documentation);
        assert_eq!(row.default_severity, DiagnosticSeverity::Warning);
        assert_eq!(row.status, DiagnosticStatus::Active);
        assert_eq!(row.retired_since, None);
        assert!(row.aliases.is_empty());
        assert!(row.replacement_codes.is_empty());
        assert_eq!(row.since, "spec-22-documentation-markdown-tags-v1");
        assert_eq!(
            row.doc_url,
            "doc/spec/en/22.error_handling_and_diagnostics.md#2269-documentation-markdown-and-tag-warnings"
        );
    }
}

#[test]
fn documentation_link_descriptors_lock_adopted_meanings() {
    let registry = DiagnosticRegistry::builtin();
    let expected = [
        (
            "W0412",
            "documentation.link.unresolved",
            "Documentation reference is unresolved",
        ),
        (
            "W0413",
            "documentation.link.ambiguous",
            "Documentation reference is ambiguous",
        ),
        (
            "W0414",
            "documentation.link.incomplete_input",
            "Documentation link inputs are incomplete",
        ),
    ];
    assert_eq!(
        registry
            .descriptors()
            .iter()
            .filter(|row| row.semantic_name.starts_with("documentation.link."))
            .count(),
        expected.len()
    );
    for (code, name, summary) in expected {
        let row = registry
            .lookup(DiagnosticCode::from_str(code).unwrap())
            .unwrap();
        assert_eq!(row.meaning_key, name);
        assert_eq!(row.semantic_name, name);
        assert_eq!(row.summary, summary);
        assert_eq!(row.phase_family, PhaseFamily::Documentation);
        assert_eq!(row.default_severity, DiagnosticSeverity::Warning);
        assert_eq!(row.status, DiagnosticStatus::Active);
        assert!(row.aliases.is_empty());
        assert_eq!(row.retired_since, None);
        assert!(row.replacement_codes.is_empty());
        assert_eq!(row.since, "DOC10-SHARED-DIAGNOSTICS");
        assert_eq!(
            row.doc_url,
            "doc/spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks"
        );
    }
}

#[test]
fn documentation_url_descriptor_locks_adopted_meaning() {
    let registry = DiagnosticRegistry::builtin();
    let code = DiagnosticCode::from_str("W0415").unwrap();
    let row = registry.lookup(code).unwrap();
    assert_eq!(row.code, code);
    assert_eq!(row.meaning_key, "documentation.render.rejected_url");
    assert_eq!(row.semantic_name, row.meaning_key);
    assert_eq!(
        row.summary,
        "Documentation URL is rejected by output policy"
    );
    assert_eq!(row.phase_family, PhaseFamily::Documentation);
    assert_eq!(row.default_severity, DiagnosticSeverity::Warning);
    assert_eq!(row.status, DiagnosticStatus::Active);
    assert!(row.aliases.is_empty());
    assert_eq!(row.retired_since, None);
    assert!(row.replacement_codes.is_empty());
    assert_eq!(row.since, "DOC13-G4-URL-POLICY-DIAGNOSTIC-PREREQUISITES");
    assert_eq!(
        row.doc_url,
        "doc/design/mizar-doc/en/render.md#adopted-html-and-url-output-policy"
    );
}

#[test]
fn frontend_allocations_match_spec_and_construct_shared_records() {
    let spec = include_str!("../../../doc/spec/en/22.error_handling_and_diagnostics.md");
    let rows = spec
        .lines()
        .filter(|line| line.starts_with("| E00") && line.contains("| §22.2.3 |"))
        .map(|line| {
            let columns = line.split('|').map(str::trim).collect::<Vec<_>>();
            (columns[1], columns[2].trim_matches('`'), columns[4])
        })
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 40);
    let registry = DiagnosticRegistry::builtin();
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "27".repeat(32)
    ))
    .unwrap();
    let source_id = InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .unwrap();

    for (index, (code_text, name, summary)) in rows.into_iter().enumerate() {
        let code = DiagnosticCode::from_str(code_text).unwrap();
        assert_eq!(code.number(), index as u16 + 13);
        let descriptor = registry
            .lookup(code)
            .expect("frontend descriptor allocated");
        assert_eq!(descriptor.semantic_name, name);
        assert_eq!(descriptor.meaning_key, descriptor.semantic_name);
        assert_eq!(descriptor.summary, summary);
        assert_eq!(descriptor.default_severity, DiagnosticSeverity::Error);
        assert_eq!(descriptor.phase_family, PhaseFamily::Syntax);
        assert_eq!(descriptor.status, DiagnosticStatus::Active);
        assert_eq!(descriptor.since, "spec-22-frontend-v1");
        assert_eq!(
            descriptor.doc_url,
            "doc/spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference"
        );
        let draft = DiagnosticDraft::new(DiagnosticDraftInput {
            source_snapshot: snapshot,
            code,
            phase: PipelinePhase::Frontend,
            category: FailureCategory::ParseError,
            stable_detail_key: name.to_owned(),
            message: summary.to_owned(),
            primary_location: DiagnosticPrimaryLocation::Span(
                DiagnosticSpan::primary(
                    SourceRange {
                        source_id,
                        start: 0,
                        end: 1,
                    },
                    None,
                )
                .unwrap(),
            ),
            secondary_spans: vec![],
            notes: vec![],
            details: DiagnosticDetails::new(),
            fixes: vec![],
            explanation: None,
        })
        .expect("allocated frontend code creates draft");
        let record = DiagnosticRecord::from_draft(
            draft,
            DiagnosticHandle::new(snapshot, DiagnosticId::new(index as u64)),
            DiagnosticFreshness::Current {
                source_snapshot: snapshot,
            },
            vec![],
        )
        .expect("allocated frontend code creates record");
        assert_eq!(record.code(), code);
        assert_eq!(record.semantic_name(), name);
    }
}

#[test]
fn import_allocations_match_spec_and_construct_shared_records() {
    let spec = include_str!("../../../doc/spec/en/22.error_handling_and_diagnostics.md");
    let rows = spec
        .lines()
        .filter(|line| line.starts_with("| E022") && line.contains("| §22.3.5 |"))
        .map(|line| {
            let columns = line.split('|').map(str::trim).collect::<Vec<_>>();
            (columns[1], columns[2].trim_matches('`'), columns[4])
        })
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 6);
    let registry = DiagnosticRegistry::builtin();
    assert!(
        registry
            .lookup(DiagnosticCode::from_str("E0205").unwrap())
            .is_none()
    );
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "40".repeat(32)
    ))
    .unwrap();
    let source_id = InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .unwrap();
    for (index, (code_text, name, summary)) in rows.into_iter().enumerate() {
        let code = DiagnosticCode::from_str(code_text).unwrap();
        assert_eq!(code.number(), 220 + index as u16);
        let descriptor = registry.lookup(code).expect("allocated import descriptor");
        assert_eq!(descriptor.semantic_name, name);
        assert_eq!(descriptor.meaning_key, name);
        assert_eq!(descriptor.summary, summary);
        assert_eq!(descriptor.default_severity, DiagnosticSeverity::Error);
        assert_eq!(descriptor.phase_family, PhaseFamily::Resolution);
        assert_eq!(descriptor.status, DiagnosticStatus::Active);
        assert_eq!(descriptor.since, "spec-22-import-v1");
        assert_eq!(
            descriptor.doc_url,
            "doc/spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference"
        );
        assert!(descriptor.aliases.is_empty());
        assert!(descriptor.replacement_codes.is_empty());
        assert_eq!(descriptor.retired_since, None);
        assert_eq!(
            registry.lookup_semantic_name(PhaseFamily::Resolution, name),
            Some(descriptor)
        );
        let draft = DiagnosticDraft::new(DiagnosticDraftInput {
            source_snapshot: snapshot,
            code,
            phase: PipelinePhase::Resolver,
            category: FailureCategory::ResolveError,
            stable_detail_key: name.to_owned(),
            message: summary.to_owned(),
            primary_location: DiagnosticPrimaryLocation::Span(
                DiagnosticSpan::primary(
                    SourceRange {
                        source_id,
                        start: 0,
                        end: 1,
                    },
                    None,
                )
                .unwrap(),
            ),
            secondary_spans: vec![],
            notes: vec![],
            details: DiagnosticDetails::new(),
            fixes: vec![],
            explanation: None,
        })
        .expect("allocated import code creates draft");
        let record = DiagnosticRecord::from_draft(
            draft,
            DiagnosticHandle::new(snapshot, DiagnosticId::new(index as u64)),
            DiagnosticFreshness::Current {
                source_snapshot: snapshot,
            },
            vec![],
        )
        .expect("allocated import code creates record");
        assert_eq!(record.code(), code);
        assert_eq!(record.semantic_name(), name);
    }
}

#[test]
fn reflection_allocations_match_spec_and_construct_shared_records() {
    let spec = include_str!("../../../doc/spec/en/22.error_handling_and_diagnostics.md");
    let rows = spec
        .lines()
        .filter(|line| line.starts_with("| E") && line.contains("| §22.4.8 |"))
        .map(|line| {
            let columns = line.split('|').map(str::trim).collect::<Vec<_>>();
            (columns[1], columns[2].trim_matches('`'), columns[4])
        })
        .collect::<Vec<_>>();
    let expected = [
        ("E0104", "type.ghost_runtime_flow", PhaseFamily::Type),
        ("E0111", "type.binder_formation", PhaseFamily::Type),
        ("E0112", "type.valuation_formation", PhaseFamily::Type),
        ("E0206", "resolve.ast_scope", PhaseFamily::Resolution),
        (
            "E0332",
            "proof.computation.invalid_import",
            PhaseFamily::Proof,
        ),
        ("E0333", "proof.computation.incomplete", PhaseFamily::Proof),
    ];
    assert_eq!(rows.len(), expected.len());
    let registry = DiagnosticRegistry::builtin();
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "48".repeat(32)
    ))
    .unwrap();
    let source_id = InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .unwrap();
    for (index, ((code_text, name, summary), (expected_code, expected_name, family))) in
        rows.into_iter().zip(expected).enumerate()
    {
        assert_eq!((code_text, name), (expected_code, expected_name));
        assert_eq!(
            summary,
            "Reflection diagnostic; specified, registry/emission deferred."
        );
        let code = DiagnosticCode::from_str(code_text).unwrap();
        let descriptor = registry
            .lookup(code)
            .expect("allocated reflection descriptor");
        assert_eq!(descriptor.code, code);
        assert_eq!(descriptor.semantic_name, name);
        assert_eq!(descriptor.meaning_key, name);
        assert_eq!(descriptor.summary, summary);
        assert_eq!(descriptor.default_severity, DiagnosticSeverity::Error);
        assert_eq!(descriptor.phase_family, family);
        assert_eq!(descriptor.status, DiagnosticStatus::Active);
        assert_eq!(descriptor.since, "spec-22.7-v1");
        assert_eq!(
            descriptor.doc_url,
            "doc/spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference"
        );
        assert!(descriptor.aliases.is_empty());
        assert_eq!(descriptor.retired_since, None);
        assert!(descriptor.replacement_codes.is_empty());
        assert_eq!(
            registry.lookup_semantic_name(family, name),
            Some(descriptor)
        );
        // Synthetic registry construction establishes no owning-stage emission coverage.
        let draft = DiagnosticDraft::new(DiagnosticDraftInput {
            source_snapshot: snapshot,
            code,
            phase: PipelinePhase::Internal,
            category: FailureCategory::InternalInvariant,
            stable_detail_key: name.to_owned(),
            message: summary.to_owned(),
            primary_location: DiagnosticPrimaryLocation::Span(
                DiagnosticSpan::primary(
                    SourceRange {
                        source_id,
                        start: 0,
                        end: 1,
                    },
                    None,
                )
                .unwrap(),
            ),
            secondary_spans: vec![],
            notes: vec![],
            details: DiagnosticDetails::new(),
            fixes: vec![],
            explanation: None,
        })
        .expect("allocated reflection code creates draft");
        let record = DiagnosticRecord::from_draft(
            draft,
            DiagnosticHandle::new(snapshot, DiagnosticId::new(index as u64)),
            DiagnosticFreshness::Current {
                source_snapshot: snapshot,
            },
            vec![],
        )
        .expect("allocated reflection code creates record");
        assert_eq!(record.code(), descriptor.code);
        assert_eq!(record.semantic_name(), descriptor.semantic_name);
        assert_eq!(record.severity(), descriptor.default_severity);
    }
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
            "E0700",
            "unknown.future",
            PhaseFamily::Algorithm,
            DiagnosticSeverity::Error,
        )]),
        Err(RegistryValidationError::CodeOutsideDefinedRange {
            code: DiagnosticCode::from_str("E0700").expect("valid code"),
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
    let original = descriptor(
        "E0001",
        "syntax.unexpected_token",
        PhaseFamily::Syntax,
        DiagnosticSeverity::Error,
    );
    for row in std::iter::once(original).chain(
        BUILTIN_DESCRIPTORS
            .iter()
            .copied()
            .filter(|row| row.phase_family == PhaseFamily::Documentation),
    ) {
        let baseline = [row];
        let reused = [DiagnosticDescriptor {
            meaning_key: "syntax.different_meaning",
            semantic_name: "syntax.different_meaning",
            ..baseline[0]
        }];

        assert!(matches!(
            validate_registry_compatibility(&baseline, &reused),
            Err(RegistryValidationError::MeaningKeyChanged { .. })
        ));
        for mutation in [
            DiagnosticDescriptor {
                phase_family: PhaseFamily::Info,
                ..row
            },
            DiagnosticDescriptor {
                default_severity: DiagnosticSeverity::Info,
                ..row
            },
            DiagnosticDescriptor {
                semantic_name: "documentation.renamed",
                ..row
            },
        ] {
            assert!(validate_registry_compatibility(&baseline, &[mutation]).is_err());
        }
        let retired = DiagnosticDescriptor {
            status: DiagnosticStatus::Retired,
            retired_since: Some("test-retirement"),
            ..row
        };
        assert!(matches!(
            validate_registry_compatibility(&[retired], &baseline),
            Err(RegistryValidationError::RetiredCodeReactivated { .. })
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
