use super::*;

#[test]
fn parses_valid_evidence_and_preserves_hashes() {
    let premise = atom_formula(1);
    let goal = Formula::Not(Box::new(atom_formula(2)));
    let bytes = evidence_bytes(
        vec![formula_item(1, 11, &premise)],
        Vec::new(),
        goal_item(20, &goal),
    );
    let parsed = parse_formula_evidence(&bytes, &context()).expect("valid evidence parses");

    assert_eq!(parsed.schema_version, 1);
    assert_eq!(parsed.encoding_version, 1);
    assert_eq!(parsed.target_vc, target());
    assert_eq!(parsed.formulas.len(), 1);
    assert_eq!(parsed.formulas[0].formula_id, 1);
    assert_eq!(
        parsed.formulas[0].formula.render(),
        atom_formula(1).render()
    );
    assert_eq!(parsed.final_goal.formula.render(), goal.render());
    assert_eq!(parsed.canonical_hash_input(), bytes);

    let entry_hash = parsed.formulas[0]
        .entry_hash_input()
        .expect("entry hash input is deterministic");
    assert_eq!(
        entry_hash,
        parsed.formulas[0]
            .entry_hash_input()
            .expect("entry hash input is stable")
    );
}

#[test]
fn parses_explicit_substitution_payload_without_instantiated_formula() {
    let formula = atom_formula(1);
    let substitution = substitution_item(7, 1, 12);
    let bytes = evidence_bytes(
        vec![formula_item(1, 11, &formula)],
        vec![substitution],
        goal_item(20, &Formula::Not(Box::new(formula.clone()))),
    );

    let parsed =
        parse_formula_evidence(&bytes, &context()).expect("valid substitution evidence parses");

    assert_eq!(parsed.substitutions.len(), 1);
    let substitution = &parsed.substitutions[0];
    assert_eq!(substitution.substitution_id, 7);
    assert_eq!(substitution.source_formula_id, 1);
    assert_eq!(substitution.payload.owner_substitution_id, 7);
    assert!(substitution.payload.replacements.is_empty());
}

#[test]
fn legacy_certificate_domain_is_unsupported() {
    let error = parse_formula_evidence(b"MIZAR_KERNEL_CERT\0", &context()).unwrap_err();

    assert_rejection(
        &error,
        RejectionCategory::CertificateRejection,
        RejectionDetail::UnsupportedCertificateFormat,
        Some("domain_separator"),
    );
}

#[test]
fn unknown_schema_version_rejects() {
    let formula = atom_formula(1);
    let mut bytes = evidence_bytes(
        vec![formula_item(1, 11, &formula)],
        Vec::new(),
        goal_item(20, &Formula::Not(Box::new(formula))),
    );
    let schema_offset = EVIDENCE_DOMAIN_SEPARATOR.len();
    bytes[schema_offset..schema_offset + 2].copy_from_slice(&2u16.to_be_bytes());

    let error = parse_formula_evidence(&bytes, &context()).unwrap_err();

    assert_rejection(
        &error,
        RejectionCategory::CertificateRejection,
        RejectionDetail::UnsupportedCertificateFormat,
        Some("schema_version"),
    );
}

#[test]
fn duplicate_formula_ids_reject_as_malformed_witness_data() {
    let formula = atom_formula(1);
    let bytes = evidence_bytes(
        vec![formula_item(1, 11, &formula), formula_item(1, 12, &formula)],
        Vec::new(),
        goal_item(20, &Formula::Not(Box::new(formula))),
    );

    let error = parse_formula_evidence(&bytes, &context()).unwrap_err();

    assert_rejection(
        &error,
        RejectionCategory::CertificateRejection,
        RejectionDetail::MalformedWitnessData,
        Some("formula.duplicate_id"),
    );
}

#[test]
fn malformed_formula_rejects_as_malformed_witness_data() {
    let bytes = evidence_bytes(
        vec![malformed_formula_item(1, 11)],
        Vec::new(),
        goal_item(20, &Formula::Not(Box::new(atom_formula(1)))),
    );

    let error = parse_formula_evidence(&bytes, &context()).unwrap_err();

    assert_rejection(
        &error,
        RejectionCategory::CertificateRejection,
        RejectionDetail::MalformedWitnessData,
        Some("formula"),
    );
}

#[test]
fn missing_formula_provenance_rejects_fail_closed() {
    let formula = atom_formula(1);
    let bytes = evidence_bytes_without_formula_provenance(
        vec![formula_item(1, 99, &formula)],
        goal_item(20, &Formula::Not(Box::new(formula))),
    );

    let error = parse_formula_evidence(&bytes, &context()).unwrap_err();

    assert_rejection(
        &error,
        RejectionCategory::KernelRejection,
        RejectionDetail::MissingProvenance,
        Some("formula.provenance_id"),
    );
}

#[test]
fn imported_statement_fingerprint_mismatch_rejects_fail_closed() {
    let formula = atom_formula(1);
    let bytes = evidence_bytes(
        vec![imported_formula_item_with_wrong_statement(1, 11, &formula)],
        Vec::new(),
        goal_item(20, &Formula::Not(Box::new(formula))),
    );

    let error = parse_formula_evidence(&bytes, &context()).unwrap_err();

    assert_rejection(
        &error,
        RejectionCategory::KernelRejection,
        RejectionDetail::MissingProvenance,
        Some("formula.statement_fingerprint"),
    );
}

#[test]
fn final_goal_target_binding_mismatch_rejects_as_missing_provenance() {
    let formula = atom_formula(1);
    let goal = Formula::Not(Box::new(formula.clone()));
    let mut sections = base_sections(vec![formula_item(1, 11, &formula)], Vec::new());
    let goal_fingerprint = formula_fingerprint(&goal);
    sections[4].1.push(provenance_item_with_target(
        20,
        Fingerprint::new(9, b"other-target".to_vec()),
        &goal_fingerprint,
    ));
    sections[5].1.push(goal_item_with_provenance(20, &goal));
    let bytes = envelope(sections);

    let error = parse_formula_evidence(&bytes, &context()).unwrap_err();

    assert_rejection(
        &error,
        RejectionCategory::KernelRejection,
        RejectionDetail::MissingProvenance,
        Some("final_goal.provenance_id"),
    );
}

#[test]
fn substitution_source_formula_gap_duplicate_and_provenance_mismatch_reject() {
    let formula = atom_formula(1);
    let missing_source = evidence_bytes(
        vec![formula_item(1, 11, &formula)],
        vec![substitution_item(7, 99, 12)],
        goal_item(20, &Formula::Not(Box::new(formula.clone()))),
    );
    let error = parse_formula_evidence(&missing_source, &context()).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::KernelRejection,
        RejectionDetail::MissingProvenance,
        Some("substitution.source_formula_id"),
    );

    let duplicate = evidence_bytes(
        vec![formula_item(1, 11, &formula)],
        vec![substitution_item(7, 1, 12), substitution_item(7, 1, 13)],
        goal_item(20, &Formula::Not(Box::new(formula.clone()))),
    );
    let error = parse_formula_evidence(&duplicate, &context()).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::CertificateRejection,
        RejectionDetail::MalformedWitnessData,
        Some("substitution.duplicate_id"),
    );

    let other_formula = atom_formula(2);
    let provenance_mismatch = evidence_bytes(
        vec![formula_item(1, 11, &other_formula)],
        vec![substitution_item(7, 1, 12)],
        goal_item(20, &Formula::Not(Box::new(other_formula))),
    );
    let error = parse_formula_evidence(&provenance_mismatch, &context()).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::KernelRejection,
        RejectionDetail::MissingProvenance,
        Some("substitution.provenance_id"),
    );
}

#[test]
fn formula_tree_fingerprint_is_separate_from_source_identity() {
    let formula = atom_formula(1);
    let bytes = evidence_bytes(
        vec![
            formula_item(1, 11, &formula),
            generated_formula_item(2, 12, 9, &formula),
        ],
        Vec::new(),
        goal_item(20, &Formula::Not(Box::new(formula))),
    );

    let parsed =
        parse_formula_evidence(&bytes, &context()).expect("same tree can have two sources");

    assert_eq!(
        parsed.formulas[0].formula_fingerprint,
        parsed.formulas[1].formula_fingerprint
    );
    assert_ne!(
        parsed.formulas[0]
            .entry_hash_input()
            .expect("local entry hash input"),
        parsed.formulas[1]
            .entry_hash_input()
            .expect("generated entry hash input")
    );
}

#[test]
fn formula_fingerprint_algorithm_and_digest_mismatches_reject() {
    let formula = atom_formula(1);
    let mut bad_algorithm = formula_item(1, 11, &formula);
    bad_algorithm[5] = 99;
    let bytes = evidence_bytes(
        vec![bad_algorithm],
        Vec::new(),
        goal_item(20, &Formula::Not(Box::new(formula.clone()))),
    );
    let error = parse_formula_evidence(&bytes, &context()).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::KernelRejection,
        RejectionDetail::MissingProvenance,
        Some("formula.fingerprint"),
    );

    let mut bad_digest = formula_item(1, 11, &formula);
    let digest_offset = 10;
    bad_digest[digest_offset] ^= 0x55;
    let bytes = evidence_bytes(
        vec![bad_digest],
        Vec::new(),
        goal_item(20, &Formula::Not(Box::new(formula))),
    );
    let error = parse_formula_evidence(&bytes, &context()).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::KernelRejection,
        RejectionDetail::MissingProvenance,
        Some("formula.fingerprint"),
    );
}

#[test]
fn imported_identity_shape_and_status_are_validated() {
    let formula = atom_formula(1);
    let bytes = evidence_bytes(
        vec![imported_formula_item_with_empty_package(1, 11, &formula)],
        Vec::new(),
        goal_item(20, &Formula::Not(Box::new(formula.clone()))),
    );
    let error = parse_formula_evidence(&bytes, &context()).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::KernelRejection,
        RejectionDetail::MissingProvenance,
        Some("formula.imported_source"),
    );

    let bytes = evidence_bytes(
        vec![imported_formula_item_with_bad_required_status(
            1, 11, &formula,
        )],
        Vec::new(),
        goal_item(20, &Formula::Not(Box::new(formula))),
    );
    let error = parse_formula_evidence(&bytes, &context()).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::CertificateRejection,
        RejectionDetail::MalformedWitnessData,
        Some("formula.required_proof_status"),
    );
}

#[test]
fn envelope_encoding_profile_target_directory_and_trailing_bytes_reject() {
    let formula = atom_formula(1);
    let mut unknown_encoding = evidence_bytes(
        vec![formula_item(1, 11, &formula)],
        Vec::new(),
        goal_item(20, &Formula::Not(Box::new(formula.clone()))),
    );
    let encoding_offset = EVIDENCE_DOMAIN_SEPARATOR.len() + 2;
    unknown_encoding[encoding_offset..encoding_offset + 2].copy_from_slice(&2u16.to_be_bytes());
    let error = parse_formula_evidence(&unknown_encoding, &context()).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::CertificateRejection,
        RejectionDetail::UnsupportedCertificateFormat,
        Some("encoding_version"),
    );

    let mut profile_mismatch = evidence_bytes(
        vec![formula_item(1, 11, &formula)],
        Vec::new(),
        goal_item(20, &Formula::Not(Box::new(formula.clone()))),
    );
    let profile_offset = EVIDENCE_DOMAIN_SEPARATOR.len() + 4;
    profile_mismatch[profile_offset..profile_offset + 2].copy_from_slice(&8u16.to_be_bytes());
    let error = parse_formula_evidence(&profile_mismatch, &context()).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::CertificateRejection,
        RejectionDetail::UnsupportedCertificateFormat,
        Some("kernel_profile"),
    );

    let target_mismatch = evidence_bytes(
        vec![formula_item(1, 11, &formula)],
        Vec::new(),
        goal_item(20, &Formula::Not(Box::new(formula.clone()))),
    );
    let other_context =
        FormulaEvidenceParseContext::v1(Fingerprint::new(9, b"other-target".to_vec()), profile());
    let error = parse_formula_evidence(&target_mismatch, &other_context).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::CertificateRejection,
        RejectionDetail::ContextMismatch,
        Some("target_vc"),
    );

    let mut section_count = evidence_bytes(
        vec![formula_item(1, 11, &formula)],
        Vec::new(),
        goal_item(20, &Formula::Not(Box::new(formula.clone()))),
    );
    let directory_count_offset = directory_count_offset();
    section_count[directory_count_offset..directory_count_offset + 4]
        .copy_from_slice(&5u32.to_be_bytes());
    let error = parse_formula_evidence(&section_count, &context()).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::CertificateRejection,
        RejectionDetail::MalformedWitnessData,
        Some("directory_entry_count"),
    );

    let mut trailing = evidence_bytes(
        vec![formula_item(1, 11, &formula)],
        Vec::new(),
        goal_item(20, &Formula::Not(Box::new(formula))),
    );
    trailing.push(0);
    let error = parse_formula_evidence(&trailing, &context()).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::CertificateRejection,
        RejectionDetail::MalformedWitnessData,
        Some("section_payloads.trailing_bytes"),
    );
}

#[test]
fn provenance_payload_duplicates_and_final_goal_fingerprint_reject() {
    let formula = atom_formula(1);
    let mut sections = base_sections(vec![formula_item(1, 11, &formula)], Vec::new());
    sections[4].1.push(provenance_item_empty_payload(
        20,
        &formula_fingerprint(&Formula::Not(Box::new(formula.clone()))),
    ));
    sections[5].1.push(goal_item_with_provenance(
        20,
        &Formula::Not(Box::new(formula.clone())),
    ));
    let error = parse_formula_evidence(&envelope(sections), &context()).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::KernelRejection,
        RejectionDetail::MissingProvenance,
        Some("provenance.payload"),
    );

    let mut sections = base_sections(vec![formula_item(1, 11, &formula)], Vec::new());
    sections[4]
        .1
        .push(provenance_item(11, &formula_fingerprint(&formula)));
    sections[4].1.push(provenance_item(
        20,
        &formula_fingerprint(&Formula::Not(Box::new(formula.clone()))),
    ));
    sections[5].1.push(goal_item_with_provenance(
        20,
        &Formula::Not(Box::new(formula.clone())),
    ));
    let error = parse_formula_evidence(&envelope(sections), &context()).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::CertificateRejection,
        RejectionDetail::MalformedWitnessData,
        Some("provenance.duplicate_id"),
    );

    let bytes = evidence_bytes(
        vec![formula_item(1, 11, &formula)],
        Vec::new(),
        goal_item_with_wrong_fingerprint(20, &Formula::Not(Box::new(formula))),
    );
    let error = parse_formula_evidence(&bytes, &context()).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::KernelRejection,
        RejectionDetail::MissingProvenance,
        Some("final_goal.fingerprint"),
    );
}

#[test]
fn oversized_counts_reject_before_unbounded_allocation() {
    let formula = atom_formula(1);
    let bytes = evidence_bytes(
        vec![formula_item_with_huge_child_count(1, 11)],
        Vec::new(),
        goal_item(20, &Formula::Not(Box::new(formula.clone()))),
    );
    let error = parse_formula_evidence(&bytes, &context()).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::CertificateRejection,
        RejectionDetail::ResourceExhaustion,
        Some("formula"),
    );

    let bytes = evidence_bytes(
        vec![formula_item(1, 11, &formula)],
        vec![substitution_item_with_huge_replacement_count(7, 1, 12)],
        goal_item(20, &Formula::Not(Box::new(atom_formula(1)))),
    );
    let error = parse_formula_evidence(&bytes, &context()).unwrap_err();
    assert_rejection(
        &error,
        RejectionCategory::CertificateRejection,
        RejectionDetail::ResourceExhaustion,
        Some("substitution.payload.replacement_count"),
    );
}

fn assert_rejection(
    record: &RejectionRecord,
    category: RejectionCategory,
    detail: RejectionDetail,
    field_path: Option<&'static str>,
) {
    assert_eq!(record.category(), category);
    assert_eq!(record.detail(), detail);
    assert_eq!(record.location().field_path, field_path);
}

fn context() -> FormulaEvidenceParseContext {
    FormulaEvidenceParseContext::v1(target(), profile())
}

fn profile() -> KernelProfileRecord {
    KernelProfileRecord::v1(7, ClauseTautologyPolicy::Reject)
}

fn target() -> Fingerprint {
    Fingerprint::new(9, b"target-vc".to_vec())
}

fn atom_formula(symbol_id: u32) -> Formula {
    Formula::Atom(Atom::with_arity(
        SymbolKey::new(SymbolKind::Predicate, symbol_id),
        0,
        Vec::new(),
    ))
}

fn formula_fingerprint(formula: &Formula) -> Fingerprint {
    Fingerprint::new(
        SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID,
        formula
            .canonical_hash_input()
            .expect("test formulas have canonical bytes"),
    )
}

fn evidence_bytes(formulas: Vec<Vec<u8>>, substitutions: Vec<Vec<u8>>, goal: Vec<u8>) -> Vec<u8> {
    let mut sections = base_sections(formulas, substitutions);
    sections[4].1.push(provenance_item(
        goal_provenance_id(&goal),
        &goal_fingerprint_from_item(&goal),
    ));
    sections[5].1.push(goal);
    envelope(sections)
}

fn evidence_bytes_without_formula_provenance(formulas: Vec<Vec<u8>>, goal: Vec<u8>) -> Vec<u8> {
    let mut sections = vec![
        (EvidenceSectionTag::SymbolManifest, symbol_items()),
        (EvidenceSectionTag::VariableManifest, Vec::new()),
        (EvidenceSectionTag::Formulas, formulas),
        (EvidenceSectionTag::Substitutions, Vec::new()),
        (EvidenceSectionTag::Provenance, Vec::new()),
        (EvidenceSectionTag::FinalGoal, vec![goal]),
    ];
    let goal_formula = Formula::Not(Box::new(atom_formula(1)));
    sections[4]
        .1
        .push(provenance_item(20, &formula_fingerprint(&goal_formula)));
    envelope(sections)
}

fn base_sections(
    formulas: Vec<Vec<u8>>,
    substitutions: Vec<Vec<u8>>,
) -> Vec<(EvidenceSectionTag, Vec<Vec<u8>>)> {
    let mut provenance = Vec::new();
    for formula in &formulas {
        let provenance_id = formula_provenance_id(formula);
        let fingerprint = formula_fingerprint_from_item(formula);
        provenance.push(provenance_item(provenance_id, &fingerprint));
    }
    for substitution in &substitutions {
        let provenance_id = substitution_provenance_id(substitution);
        provenance.push(provenance_item(
            provenance_id,
            &formula_fingerprint(&atom_formula(1)),
        ));
    }
    vec![
        (EvidenceSectionTag::SymbolManifest, symbol_items()),
        (EvidenceSectionTag::VariableManifest, Vec::new()),
        (EvidenceSectionTag::Formulas, formulas),
        (EvidenceSectionTag::Substitutions, substitutions),
        (EvidenceSectionTag::Provenance, provenance),
        (EvidenceSectionTag::FinalGoal, Vec::new()),
    ]
}

fn envelope(sections: Vec<(EvidenceSectionTag, Vec<Vec<u8>>)>) -> Vec<u8> {
    let mut payloads = Vec::new();
    let mut directory = Vec::new();
    let mut offset = 0u32;
    for (section, items) in &sections {
        let mut section_payload = Vec::new();
        for item in items {
            section_payload.push(section.byte());
            section_payload.push(1);
            put_len(item.len(), &mut section_payload);
            section_payload.extend_from_slice(item);
        }
        let length = u32::try_from(section_payload.len()).expect("test section length fits");
        directory.push((*section, items.len() as u32, offset, length));
        offset = offset
            .checked_add(length)
            .expect("test payload offset fits");
        payloads.push(section_payload);
    }

    let mut bytes = Vec::from(EVIDENCE_DOMAIN_SEPARATOR);
    put_u16(1, &mut bytes);
    put_u16(1, &mut bytes);
    put_profile(&mut bytes);
    put_fingerprint(&target(), &mut bytes);
    put_u32(sections.len() as u32, &mut bytes);
    for (section, count, payload_offset, payload_length) in directory {
        bytes.push(section.byte());
        put_u32(count, &mut bytes);
        put_u32(payload_offset, &mut bytes);
        put_u32(payload_length, &mut bytes);
    }
    for payload in payloads {
        bytes.extend(payload);
    }
    bytes
}

fn symbol_items() -> Vec<Vec<u8>> {
    [1u32, 2u32]
        .into_iter()
        .map(|id| {
            let mut item = Vec::new();
            item.push(symbol_kind_tag(SymbolKind::Predicate));
            put_u32(id, &mut item);
            item
        })
        .collect()
}

fn formula_item(formula_id: u32, provenance_id: u32, formula: &Formula) -> Vec<u8> {
    let fingerprint = formula_fingerprint(formula);
    let mut item = Vec::new();
    put_u32(formula_id, &mut item);
    item.push(FormulaSourceClass::LocalHypothesis.tag());
    put_fingerprint(&fingerprint, &mut item);
    put_u32(provenance_id, &mut item);
    put_u32(1, &mut item);
    put_formula(formula, &mut item);
    item
}

fn generated_formula_item(
    formula_id: u32,
    provenance_id: u32,
    vc_fact_id: u32,
    formula: &Formula,
) -> Vec<u8> {
    let fingerprint = formula_fingerprint(formula);
    let mut item = Vec::new();
    put_u32(formula_id, &mut item);
    item.push(FormulaSourceClass::GeneratedVcFact.tag());
    put_fingerprint(&fingerprint, &mut item);
    put_u32(provenance_id, &mut item);
    put_u32(vc_fact_id, &mut item);
    put_formula(formula, &mut item);
    item
}

fn malformed_formula_item(formula_id: u32, provenance_id: u32) -> Vec<u8> {
    let fingerprint = formula_fingerprint(&atom_formula(1));
    let mut item = Vec::new();
    put_u32(formula_id, &mut item);
    item.push(FormulaSourceClass::LocalHypothesis.tag());
    put_fingerprint(&fingerprint, &mut item);
    put_u32(provenance_id, &mut item);
    put_u32(1, &mut item);
    item.push(99);
    item
}

fn formula_item_with_huge_child_count(formula_id: u32, provenance_id: u32) -> Vec<u8> {
    let fingerprint = formula_fingerprint(&atom_formula(1));
    let mut item = Vec::new();
    put_u32(formula_id, &mut item);
    item.push(FormulaSourceClass::LocalHypothesis.tag());
    put_fingerprint(&fingerprint, &mut item);
    put_u32(provenance_id, &mut item);
    put_u32(1, &mut item);
    item.push(3);
    put_u32(u32::MAX, &mut item);
    item
}

fn imported_formula_item_with_empty_package(
    formula_id: u32,
    provenance_id: u32,
    formula: &Formula,
) -> Vec<u8> {
    let fingerprint = formula_fingerprint(formula);
    let mut item = Vec::new();
    put_u32(formula_id, &mut item);
    item.push(FormulaSourceClass::AcceptedImportedAxiom.tag());
    put_fingerprint(&fingerprint, &mut item);
    put_u32(provenance_id, &mut item);
    put_bytes(b"", &mut item);
    put_bytes(b"module", &mut item);
    put_bytes(b"ITEM", &mut item);
    put_fingerprint(&fingerprint, &mut item);
    item.push(required_status_tag(RequiredProofStatus::KernelVerified));
    put_formula(formula, &mut item);
    item
}

fn imported_formula_item_with_bad_required_status(
    formula_id: u32,
    provenance_id: u32,
    formula: &Formula,
) -> Vec<u8> {
    let fingerprint = formula_fingerprint(formula);
    let mut item = Vec::new();
    put_u32(formula_id, &mut item);
    item.push(FormulaSourceClass::AcceptedImportedAxiom.tag());
    put_fingerprint(&fingerprint, &mut item);
    put_u32(provenance_id, &mut item);
    put_bytes(b"pkg", &mut item);
    put_bytes(b"module", &mut item);
    put_bytes(b"ITEM", &mut item);
    put_fingerprint(&fingerprint, &mut item);
    item.push(99);
    put_formula(formula, &mut item);
    item
}

fn imported_formula_item_with_wrong_statement(
    formula_id: u32,
    provenance_id: u32,
    formula: &Formula,
) -> Vec<u8> {
    let fingerprint = formula_fingerprint(formula);
    let mut item = Vec::new();
    put_u32(formula_id, &mut item);
    item.push(FormulaSourceClass::AcceptedImportedAxiom.tag());
    put_fingerprint(&fingerprint, &mut item);
    put_u32(provenance_id, &mut item);
    put_bytes(b"pkg", &mut item);
    put_bytes(b"module", &mut item);
    put_bytes(b"ITEM", &mut item);
    put_fingerprint(&Fingerprint::new(2, b"wrong".to_vec()), &mut item);
    item.push(required_status_tag(RequiredProofStatus::KernelVerified));
    put_formula(formula, &mut item);
    item
}

fn substitution_item(substitution_id: u32, source_formula_id: u32, provenance_id: u32) -> Vec<u8> {
    let mut item = Vec::new();
    put_u32(substitution_id, &mut item);
    put_u32(source_formula_id, &mut item);
    put_u32(provenance_id, &mut item);
    put_bytes(b"binder-context", &mut item);
    put_u32(substitution_id, &mut item);
    item.push(PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP);
    put_u32(0, &mut item);
    put_u32(0, &mut item);
    put_u32(0, &mut item);
    put_u32(0, &mut item);
    item
}

fn substitution_item_with_huge_replacement_count(
    substitution_id: u32,
    source_formula_id: u32,
    provenance_id: u32,
) -> Vec<u8> {
    let mut item = Vec::new();
    put_u32(substitution_id, &mut item);
    put_u32(source_formula_id, &mut item);
    put_u32(provenance_id, &mut item);
    put_bytes(b"binder-context", &mut item);
    put_u32(substitution_id, &mut item);
    item.push(PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP);
    put_u32(0, &mut item);
    put_u32(u32::MAX, &mut item);
    item
}

fn goal_item(provenance_id: u32, formula: &Formula) -> Vec<u8> {
    goal_item_with_provenance(provenance_id, formula)
}

fn goal_item_with_provenance(provenance_id: u32, formula: &Formula) -> Vec<u8> {
    let fingerprint = formula_fingerprint(formula);
    let mut item = Vec::new();
    item.push(GoalPolarity::AssertFalseForRefutation.tag());
    put_fingerprint(&fingerprint, &mut item);
    put_u32(provenance_id, &mut item);
    put_formula(formula, &mut item);
    item
}

fn goal_item_with_wrong_fingerprint(provenance_id: u32, formula: &Formula) -> Vec<u8> {
    let mut fingerprint = formula_fingerprint(formula);
    fingerprint.digest[0] ^= 0x33;
    let mut item = Vec::new();
    item.push(GoalPolarity::AssertFalseForRefutation.tag());
    put_fingerprint(&fingerprint, &mut item);
    put_u32(provenance_id, &mut item);
    put_formula(formula, &mut item);
    item
}

fn provenance_item(provenance_id: u32, fingerprint: &Fingerprint) -> Vec<u8> {
    provenance_item_with_target(provenance_id, target(), fingerprint)
}

fn provenance_item_with_target(
    provenance_id: u32,
    target_vc: Fingerprint,
    fingerprint: &Fingerprint,
) -> Vec<u8> {
    let mut item = Vec::new();
    put_u32(provenance_id, &mut item);
    put_fingerprint(&target_vc, &mut item);
    put_fingerprint(fingerprint, &mut item);
    put_bytes(b"producer-payload", &mut item);
    item
}

fn provenance_item_empty_payload(provenance_id: u32, fingerprint: &Fingerprint) -> Vec<u8> {
    let mut item = Vec::new();
    put_u32(provenance_id, &mut item);
    put_fingerprint(&target(), &mut item);
    put_fingerprint(fingerprint, &mut item);
    put_bytes(b"", &mut item);
    item
}

fn formula_provenance_id(item: &[u8]) -> u32 {
    u32::from_be_bytes([
        item[10 + fingerprint_len(item)],
        item[11 + fingerprint_len(item)],
        item[12 + fingerprint_len(item)],
        item[13 + fingerprint_len(item)],
    ])
}

fn substitution_provenance_id(item: &[u8]) -> u32 {
    u32::from_be_bytes([item[8], item[9], item[10], item[11]])
}

fn goal_provenance_id(item: &[u8]) -> u32 {
    let start = 1 + fingerprint_item_len(&item[1..]);
    u32::from_be_bytes([
        item[start],
        item[start + 1],
        item[start + 2],
        item[start + 3],
    ])
}

fn goal_fingerprint_from_item(item: &[u8]) -> Fingerprint {
    fingerprint_from_slice(&item[1..])
}

fn formula_fingerprint_from_item(item: &[u8]) -> Fingerprint {
    fingerprint_from_slice(&item[5..])
}

fn fingerprint_len(item: &[u8]) -> usize {
    u32::from_be_bytes([item[6], item[7], item[8], item[9]]) as usize
}

fn fingerprint_from_slice(bytes: &[u8]) -> Fingerprint {
    let algorithm_id = bytes[0];
    let len = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;
    Fingerprint::new(algorithm_id, bytes[5..5 + len].to_vec())
}

fn fingerprint_item_len(bytes: &[u8]) -> usize {
    5 + u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize
}

fn directory_count_offset() -> usize {
    EVIDENCE_DOMAIN_SEPARATOR.len() + 4 + PROFILE_LEN + 1 + 4 + target().digest.len()
}

fn put_formula(formula: &Formula, bytes: &mut Vec<u8>) {
    match formula {
        Formula::Atom(atom) => {
            bytes.push(1);
            put_atom(atom, bytes);
        }
        Formula::Not(child) => {
            bytes.push(2);
            put_formula(child, bytes);
        }
        Formula::And(children) => {
            bytes.push(3);
            put_u32(children.len() as u32, bytes);
            for child in children {
                put_formula(child, bytes);
            }
        }
        Formula::Or(children) => {
            bytes.push(4);
            put_u32(children.len() as u32, bytes);
            for child in children {
                put_formula(child, bytes);
            }
        }
    }
}

fn put_atom(atom: &Atom, bytes: &mut Vec<u8>) {
    bytes.push(symbol_kind_tag(atom.symbol.kind));
    put_u32(atom.symbol.id.0, bytes);
    put_u32(atom.arity, bytes);
    put_u32(atom.arguments.len() as u32, bytes);
    for argument in &atom.arguments {
        put_term(argument, bytes);
    }
}

fn put_term(term: &Term, bytes: &mut Vec<u8>) {
    match term {
        Term::Variable(variable) => {
            bytes.push(1);
            put_u32(variable.0, bytes);
        }
        Term::Application { symbol, arguments } => {
            bytes.push(2);
            bytes.push(symbol_kind_tag(symbol.kind));
            put_u32(symbol.id.0, bytes);
            put_u32(arguments.len() as u32, bytes);
            for argument in arguments {
                put_term(argument, bytes);
            }
        }
        Term::BinderNormalized { binder_id, body } => {
            bytes.push(3);
            put_u32(*binder_id, bytes);
            put_term(body, bytes);
        }
        Term::Malformed => bytes.push(255),
    }
}

fn put_profile(bytes: &mut Vec<u8>) {
    let profile = profile();
    put_u16(profile.profile_id, bytes);
    put_u16(profile.clause_schema_version, bytes);
    put_u16(profile.clause_encoding_version, bytes);
    bytes.push(profile.clause_tautology_policy.tag());
    bytes.push(profile.certificate_hash_input_algorithm.tag());
}

fn put_fingerprint(fingerprint: &Fingerprint, bytes: &mut Vec<u8>) {
    bytes.push(fingerprint.algorithm_id);
    put_bytes(&fingerprint.digest, bytes);
}

fn put_bytes(payload: &[u8], bytes: &mut Vec<u8>) {
    put_len(payload.len(), bytes);
    bytes.extend_from_slice(payload);
}

fn put_len(len: usize, bytes: &mut Vec<u8>) {
    put_u32(u32::try_from(len).expect("test length fits"), bytes);
}

fn put_u16(value: u16, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn put_u32(value: u32, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn symbol_kind_tag(kind: SymbolKind) -> u8 {
    match kind {
        SymbolKind::Predicate => 1,
        SymbolKind::FunctorPredicate => 2,
        SymbolKind::Equality => 3,
        SymbolKind::BuiltinRelation => 4,
    }
}
