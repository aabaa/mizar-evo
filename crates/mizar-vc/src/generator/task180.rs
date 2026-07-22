use super::{
    BuildCandidateInput, CoreGenerationCandidateSet, GeneratorError, VcNormalizationInput,
    build_candidate, source_shape_hash_marker,
};
use crate::vc_ir::{
    AnchorCompleteness, AnchorIngredient, GenerationSchemaVersion, SeedIntakeMapping,
    SeedIntakeTable, SeedOriginRef, SeedVcMapping, VcFormulaRef, VcId, VcKind, VcModuleRef,
    VcProvenance, VcProvenancePhase, VcSchemaVersion, VcSet, VcStatus, VcText,
};
use mizar_core::{
    control_flow::{
        ObligationHandoffId, ObligationHandoffOrigin, build_control_flow_ir,
        build_obligation_seed_handoff,
    },
    core_ir::{
        CoreFormulaId, CoreFormulaKind, CoreIr, CoreItemId, CoreItemKind, CoreItemStatus,
        CoreNodeRef, CoreProofId, CoreProofNodeId, CoreProofNodeKind, CoreProofStatus,
        CoreProvenancePhase, CoreSourceAnchor, ObligationSeedId, ObligationSeedKind,
        ObligationSeedStatus,
    },
};
use mizar_session::BuildSnapshotId;
use std::{error::Error, fmt};

/// Borrowed inputs for the exact source-derived Task-180 VC adapter.
#[derive(Debug, Clone, Copy)]
pub struct ExactTask180VcInput<'a> {
    pub core: &'a CoreIr,
    pub snapshot: BuildSnapshotId,
    pub generation_schema_version: &'a GenerationSchemaVersion,
    pub vc_schema_version: &'a VcSchemaVersion,
}

/// Atomic failure from the exact source-derived Task-180 VC adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExactTask180VcError {
    InvalidCore { reason: String },
    InvalidControlFlow { reason: String },
    InvalidHandoff { reason: String },
    InvalidIntake { reason: String },
    Generation { reason: String },
    InvalidOutput { reason: String },
}

impl fmt::Display for ExactTask180VcError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (stage, reason) = match self {
            Self::InvalidCore { reason } => ("core", reason),
            Self::InvalidControlFlow { reason } => ("control-flow", reason),
            Self::InvalidHandoff { reason } => ("handoff", reason),
            Self::InvalidIntake { reason } => ("intake", reason),
            Self::Generation { reason } => ("generation", reason),
            Self::InvalidOutput { reason } => ("output", reason),
        };
        write!(formatter, "invalid exact Task-180 {stage}: {reason}")
    }
}

impl Error for ExactTask180VcError {}

/// Validates and maps the exact Core Task-31 contradiction projection to one open VC.
///
/// The function constructs control flow, seed handoff, and seed intake internally and
/// returns no partial candidate set on failure. It classifies the terminal goal from
/// the authenticated Core proof graph rather than from provenance marker text.
pub fn generate_exact_task180_vc(
    input: ExactTask180VcInput<'_>,
) -> Result<VcSet, ExactTask180VcError> {
    validate_exact_core(input.core)?;

    let flow = build_control_flow_ir(input.core);
    if !flow.flows.is_empty() || !flow.flow_map.is_empty() {
        return Err(ExactTask180VcError::InvalidControlFlow {
            reason: "the exact theorem projection must produce an empty ControlFlowOutput"
                .to_owned(),
        });
    }

    let handoff = build_obligation_seed_handoff(input.core, &flow);
    validate_exact_handoff(input.core, &handoff)?;
    let intake = SeedIntakeTable::try_from_handoff(&handoff).map_err(|error| {
        ExactTask180VcError::InvalidIntake {
            reason: error.to_string(),
        }
    })?;
    validate_exact_intake(&intake)?;

    let module = canonical_module_ref(input.core);
    let row = &intake.rows()[0];
    let entry = handoff
        .entries
        .get(ObligationHandoffId::new(0))
        .ok_or_else(|| ExactTask180VcError::InvalidHandoff {
            reason: "missing handoff zero after validation".to_owned(),
        })?;
    let candidate = build_candidate(BuildCandidateInput {
        schema_version: input.generation_schema_version,
        module: &module,
        handoff: row.handoff,
        origin: row.origin.clone(),
        seed_status: row.seed_status,
        seed: &entry.seed,
        flow_id: None,
        flow_algorithm: None,
        flow_site: None,
        source: row.source.clone(),
        goal: CoreFormulaId::new(0),
        kind: VcKind::TerminalProofGoal,
    })
    .map_err(generation_error)?;
    let candidates = CoreGenerationCandidateSet {
        schema_version: input.generation_schema_version.clone(),
        module,
        candidates: vec![candidate],
        no_candidates: Vec::new(),
    };
    let output = CoreGenerationCandidateSet::try_normalize(VcNormalizationInput {
        schema_version: input.vc_schema_version,
        snapshot: input.snapshot,
        source: input.core.source_id(),
        candidates: &candidates,
    })
    .map_err(generation_error)?;
    validate_exact_output(input, &output)?;
    Ok(output)
}

fn generation_error(error: GeneratorError) -> ExactTask180VcError {
    ExactTask180VcError::Generation {
        reason: error.to_string(),
    }
}

fn canonical_module_ref(core: &CoreIr) -> VcModuleRef {
    let package = core.module_id().package().as_str();
    let path = core.module_id().path().as_str();
    VcModuleRef::new(format!(
        "package={}:{};module={}:{}",
        package.len(),
        package,
        path.len(),
        path
    ))
}

fn validate_exact_core(core: &CoreIr) -> Result<(), ExactTask180VcError> {
    let invalid = |reason: &str| ExactTask180VcError::InvalidCore {
        reason: reason.to_owned(),
    };
    if core.items().len() != 1
        || !core.terms().is_empty()
        || core.formulas().len() != 1
        || !core.definitions().is_empty()
        || core.proofs().len() != 1
        || core.proof_nodes().len() != 1
        || !core.algorithms().is_empty()
        || !core.algorithm_statements().is_empty()
        || !core.generated().is_empty()
        || core.obligation_seeds().len() != 1
        || !core.diagnostics().is_empty()
    {
        return Err(invalid(
            "exact Core table allowlist or dense cardinality mismatch",
        ));
    }

    let item = core
        .items()
        .get(CoreItemId::new(0))
        .ok_or_else(|| invalid("missing theorem item zero"))?;
    let formula = core
        .formulas()
        .get(CoreFormulaId::new(0))
        .ok_or_else(|| invalid("missing false formula zero"))?;
    let proof = core
        .proofs()
        .get(CoreProofId::new(0))
        .ok_or_else(|| invalid("missing proof zero"))?;
    let proof_node = core
        .proof_nodes()
        .get(CoreProofNodeId::new(0))
        .ok_or_else(|| invalid("missing proof node zero"))?;
    let seed = core
        .obligation_seeds()
        .get(ObligationSeedId::new(0))
        .ok_or_else(|| invalid("missing obligation seed zero"))?;

    if item.kind != CoreItemKind::Theorem
        || item.visibility.as_str() != "public"
        || item.status != CoreItemStatus::Valid
        || !item.dependencies.is_empty()
        || !item.diagnostics.is_empty()
        || item.symbol.module() != core.module_id()
    {
        return Err(invalid("public structurally valid theorem owner mismatch"));
    }
    if formula.kind != CoreFormulaKind::False {
        return Err(invalid("exact theorem proposition must be False"));
    }
    if proof.item != CoreItemId::new(0)
        || proof.proposition != CoreFormulaId::new(0)
        || proof.root != CoreProofNodeId::new(0)
        || proof.status != CoreProofStatus::PendingAutomaticProof
    {
        return Err(invalid("pending proof backlink mismatch"));
    }
    if !matches!(
        &proof_node.kind,
        CoreProofNodeKind::TerminalGoal {
            obligation,
            citations
        } if *obligation == ObligationSeedId::new(0) && citations.is_empty()
    ) || !proof_node.diagnostics.is_empty()
    {
        return Err(invalid("direct terminal proof-node backlink mismatch"));
    }
    if seed.owner != CoreItemId::new(0)
        || seed.kind != ObligationSeedKind::TheoremProof
        || seed.goal != Some(CoreFormulaId::new(0))
        || !seed.context.is_empty()
        || seed.local_path.as_str() != "proof/0"
        || seed.label.is_some()
        || seed.semantic_origin.as_str() != item.symbol.fqn().as_str()
        || seed.status != ObligationSeedStatus::Active
        || !seed.diagnostics.is_empty()
        || seed.core_refs
            != [
                CoreNodeRef::Item(CoreItemId::new(0)),
                CoreNodeRef::Formula(CoreFormulaId::new(0)),
                CoreNodeRef::Proof(CoreProofId::new(0)),
                CoreNodeRef::ProofNode(CoreProofNodeId::new(0)),
            ]
    {
        return Err(invalid(
            "active theorem obligation identity or Core-reference mismatch",
        ));
    }

    let item_range = direct_range(&item.source, core, "item")?;
    let formula_range = direct_range(&formula.source, core, "formula")?;
    if direct_range(&proof.source, core, "proof")? != item_range
        || direct_range(&proof_node.source, core, "proof node")? != formula_range
        || direct_range(&seed.source, core, "obligation")? != formula_range
        || item_range.start > formula_range.start
        || formula_range.end > item_range.end
    {
        return Err(invalid("owner/formula source-range relation mismatch"));
    }

    let source_map = core.source_map();
    if source_map.item_sources.len() != 1
        || source_map.formula_sources.len() != 1
        || source_map.proof_sources.len() != 1
        || source_map.obligation_sources.len() != 1
        || !source_map.term_sources.is_empty()
        || !source_map.definition_sources.is_empty()
        || !source_map.algorithm_sources.is_empty()
        || !source_map.generated_sources.is_empty()
        || source_map.item_sources.get(&CoreItemId::new(0)) != Some(&item.source)
        || source_map.formula_sources.get(&CoreFormulaId::new(0)) != Some(&formula.source)
        || source_map.proof_sources.get(&CoreProofNodeId::new(0)) != Some(&proof_node.source)
        || source_map.obligation_sources.get(&ObligationSeedId::new(0)) != Some(&seed.source)
    {
        return Err(invalid("exact source-map domain or backlink mismatch"));
    }
    validate_exact_provenance(item, formula, proof, proof_node, seed)?;
    Ok(())
}

fn direct_range(
    source: &mizar_core::core_ir::CoreSourceRef,
    core: &CoreIr,
    role: &str,
) -> Result<mizar_session::SourceRange, ExactTask180VcError> {
    match source.anchor {
        CoreSourceAnchor::SourceRange(range) if range.source_id == core.source_id() => Ok(range),
        _ => Err(ExactTask180VcError::InvalidCore {
            reason: format!("{role} source is not a direct range in the Core source"),
        }),
    }
}

fn validate_exact_provenance(
    item: &mizar_core::core_ir::CoreItem,
    formula: &mizar_core::core_ir::CoreFormula,
    proof: &mizar_core::core_ir::CoreProof,
    proof_node: &mizar_core::core_ir::CoreProofNode,
    seed: &mizar_core::core_ir::ObligationSeed,
) -> Result<(), ExactTask180VcError> {
    let invalid = || ExactTask180VcError::InvalidCore {
        reason: "exact Core Task-31 provenance contract mismatch".to_owned(),
    };
    let [item_resolver, item_statement] = item.source.provenance.as_slice() else {
        return Err(invalid());
    };
    let [formula_statement] = formula.source.provenance.as_slice() else {
        return Err(invalid());
    };
    let [proof_resolver, proof_policy] = proof.source.provenance.as_slice() else {
        return Err(invalid());
    };
    let [terminal, skeleton] = proof_node.source.provenance.as_slice() else {
        return Err(invalid());
    };
    if item_resolver.phase != CoreProvenancePhase::Resolver
        || item_statement.phase != CoreProvenancePhase::Checker
        || formula_statement.phase != CoreProvenancePhase::Checker
        || proof_resolver.phase != CoreProvenancePhase::Resolver
        || proof_policy.phase != CoreProvenancePhase::Checker
        || terminal.phase != CoreProvenancePhase::Checker
        || skeleton.phase != CoreProvenancePhase::ProofSkeleton
        || item_resolver != proof_resolver
        || item_statement != formula_statement
        || seed.provenance != proof_node.source.provenance
        || seed.source.provenance != seed.provenance
    {
        return Err(invalid());
    }

    if !valid_resolver_key(item_resolver.key.as_str(), item.symbol.fqn().as_str())
        || proof_policy.key.as_str()
            != "task267/v1;proof=0;statement=0;policy=unmodified;justification=omitted;status=pending-automatic-proof"
        || skeleton.key.as_str() != "task267/v1;local-path=7:proof/0"
    {
        return Err(invalid());
    }
    let statement_ids = parse_statement_key(item_statement.key.as_str()).ok_or_else(invalid)?;
    let terminal_ids = parse_terminal_key(terminal.key.as_str()).ok_or_else(invalid)?;
    if statement_ids.1 != terminal_ids.0 || statement_ids.2 != terminal_ids.1 {
        return Err(invalid());
    }
    Ok(())
}

fn valid_resolver_key(key: &str, expected_fqn: &str) -> bool {
    let Some(rest) = key.strip_prefix("task267/v1;owner-fqn=") else {
        return false;
    };
    let Some((length, rest)) = parse_decimal_field(rest, ":") else {
        return false;
    };
    let Ok(length) = usize::try_from(length) else {
        return false;
    };
    if rest.len() < length || !rest.is_char_boundary(length) {
        return false;
    }
    let (fqn, rest) = rest.split_at(length);
    let Some(rest) = rest.strip_prefix(";origin-path=") else {
        return false;
    };
    let Some((count, path)) = parse_decimal_field(rest, ":") else {
        return false;
    };
    let Ok(count) = usize::try_from(count) else {
        return false;
    };
    let path_is_canonical = if count == 0 {
        path.is_empty()
    } else {
        path.split(',').count() == count
            && path.split(',').all(|part| {
                parse_canonical_decimal(part).is_some_and(|value| u32::try_from(value).is_ok())
            })
    };
    fqn == expected_fqn && length == expected_fqn.len() && path_is_canonical
}

fn parse_statement_key(key: &str) -> Option<(usize, usize, usize)> {
    let rest = key.strip_prefix("task267/v1;statement=0;owner-node=")?;
    let (owner_node, rest) = parse_decimal_field(rest, ";formula=0;formula-site-node=")?;
    let (formula_site_node, formula_node) = parse_decimal_field(rest, ";formula-node=")?;
    Some((
        usize::try_from(owner_node).ok()?,
        usize::try_from(formula_site_node).ok()?,
        usize::try_from(parse_canonical_decimal(formula_node)?).ok()?,
    ))
}

fn parse_terminal_key(key: &str) -> Option<(usize, usize)> {
    let rest =
        key.strip_prefix("task267/v1;proof-node=0;terminal-goal=0;formula=0;formula-site-node=")?;
    let (formula_site_node, formula_node) = parse_decimal_field(rest, ";formula-node=")?;
    Some((
        usize::try_from(formula_site_node).ok()?,
        usize::try_from(parse_canonical_decimal(formula_node)?).ok()?,
    ))
}

fn parse_decimal_field<'a>(input: &'a str, delimiter: &str) -> Option<(u64, &'a str)> {
    let (number, rest) = input.split_once(delimiter)?;
    Some((parse_canonical_decimal(number)?, rest))
}

fn parse_canonical_decimal(input: &str) -> Option<u64> {
    if input.is_empty()
        || (input.len() > 1 && input.starts_with('0'))
        || !input.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    input.parse().ok()
}

fn validate_exact_handoff(
    core: &CoreIr,
    handoff: &mizar_core::control_flow::ObligationSeedHandoff,
) -> Result<(), ExactTask180VcError> {
    let invalid = |reason: &str| ExactTask180VcError::InvalidHandoff {
        reason: reason.to_owned(),
    };
    let entry = handoff
        .entries
        .get(ObligationHandoffId::new(0))
        .ok_or_else(|| invalid("missing handoff zero"))?;
    let seed = core
        .obligation_seeds()
        .get(ObligationSeedId::new(0))
        .ok_or_else(|| invalid("missing Core seed zero"))?;
    if handoff.entries.len() != 1
        || handoff.source_map.len() != 1
        || !matches!(
            entry.origin,
            ObligationHandoffOrigin::ExistingCore { seed }
                if seed == ObligationSeedId::new(0)
        )
        || entry.flow_site.is_some()
        || &entry.seed != seed
        || handoff.source_map.get(&ObligationHandoffId::new(0)) != Some(&seed.source)
    {
        return Err(invalid(
            "singleton ExistingCore handoff/source-map mismatch",
        ));
    }
    Ok(())
}

fn validate_exact_intake(intake: &SeedIntakeTable) -> Result<(), ExactTask180VcError> {
    let row = intake
        .rows()
        .first()
        .ok_or_else(|| ExactTask180VcError::InvalidIntake {
            reason: "missing intake row zero".to_owned(),
        })?;
    if intake.rows().len() != 1
        || row.handoff != ObligationHandoffId::new(0)
        || row.origin
            != (SeedOriginRef::ExistingCore {
                seed: ObligationSeedId::new(0),
            })
        || row.seed_status != ObligationSeedStatus::Active
        || !matches!(
            row.mapping,
            SeedIntakeMapping::EligibleOneVc { goal }
                if goal == CoreFormulaId::new(0)
        )
    {
        return Err(ExactTask180VcError::InvalidIntake {
            reason: "fresh singleton EligibleOneVc intake mismatch".to_owned(),
        });
    }
    Ok(())
}

fn validate_exact_output(
    input: ExactTask180VcInput<'_>,
    output: &VcSet,
) -> Result<(), ExactTask180VcError> {
    let invalid = |reason: &str| ExactTask180VcError::InvalidOutput {
        reason: reason.to_owned(),
    };
    let seed = input
        .core
        .obligation_seeds()
        .get(ObligationSeedId::new(0))
        .ok_or_else(|| invalid("missing validated Core seed"))?;
    let vc = output
        .vc(VcId::new(0))
        .ok_or_else(|| invalid("missing dense VC zero"))?;
    let expected_range = match seed.source.anchor {
        CoreSourceAnchor::SourceRange(range) => Some(range),
        _ => None,
    };
    let expected_anchor_provenance = seed
        .provenance
        .iter()
        .chain(seed.source.provenance.iter())
        .cloned()
        .map(|core| VcProvenance {
            phase: VcProvenancePhase::CoreHandoff,
            key: VcText::new("source-provenance"),
            core: Some(core),
        })
        .collect::<Vec<_>>();
    let mut expected_vc_provenance = seed
        .provenance
        .iter()
        .cloned()
        .map(|core| VcProvenance {
            phase: VcProvenancePhase::CoreHandoff,
            key: VcText::new("TheoremProof"),
            core: Some(core),
        })
        .collect::<Vec<_>>();
    expected_vc_provenance.extend([
        VcProvenance {
            phase: VcProvenancePhase::Generator,
            key: VcText::new("task-6-core-candidate"),
            core: None,
        },
        VcProvenance {
            phase: VcProvenancePhase::Normalization,
            key: VcText::new("task-8-normalized-vc"),
            core: None,
        },
    ]);
    let expected_source_shape = source_shape_hash_marker(
        &vc.anchor.owner,
        &VcKind::TerminalProofGoal,
        seed,
        None,
        &expected_anchor_provenance,
    );
    let expected_context_hash = crate::vc_ir::local_context_hash_marker(&vc.local_context);
    if output.schema_version() != input.vc_schema_version
        || output.snapshot() != input.snapshot
        || output.source() != input.core.source_id()
        || output.module() != &canonical_module_ref(input.core)
        || !output.generated_formulas().is_empty()
        || output.vcs().len() != 1
        || output.seed_accounting().len() != 1
        || vc.kind != VcKind::TerminalProofGoal
        || vc.status != VcStatus::Open
        || vc.goal != VcFormulaRef::Core(CoreFormulaId::new(0))
        || vc.seed.handoff != ObligationHandoffId::new(0)
        || vc.source.primary != seed.source
        || !vc.source.related.is_empty()
        || !vc.local_context.entries().is_empty()
        || !vc.local_context.policy_inputs().is_empty()
        || !vc.premises.is_empty()
        || vc.proof_hint.is_some()
        || vc.anchor.owner != crate::vc_ir::AnchorOwner::Theorem(CoreItemId::new(0))
        || vc.anchor.kind != VcKind::TerminalProofGoal
        || vc.anchor.local_path != seed.local_path
        || vc.anchor.label.is_some()
        || vc.anchor.semantic_origin != seed.semantic_origin
        || vc.anchor.source_range != expected_range
        || vc.anchor.provenance != expected_anchor_provenance
        || vc.anchor.source_shape_hash != expected_source_shape
        || vc.anchor.canonical_context_hash != expected_context_hash
        || vc.anchor.canonical_goal_hash.is_available()
        || vc.anchor.generation_schema_version != *input.generation_schema_version
        || vc.anchor.completeness
            != (AnchorCompleteness::Incomplete {
                missing: vec![AnchorIngredient::CanonicalGoalHash],
            })
        || output.seed_accounting()[0].handoff != ObligationHandoffId::new(0)
        || output.seed_accounting()[0].origin
            != (SeedOriginRef::ExistingCore {
                seed: ObligationSeedId::new(0),
            })
        || output.seed_accounting()[0].seed_status != ObligationSeedStatus::Active
        || output.seed_accounting()[0].mapping != (SeedVcMapping::One { vc: VcId::new(0) })
        || vc.provenance != expected_vc_provenance
    {
        return Err(invalid("full Task-180 VcSet contract mismatch"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mizar_core::control_flow::{ControlFlowObligationSite, ControlFlowObligationSiteKind};
    use mizar_core::core_ir::{
        CoreAlgorithm, CoreAlgorithmStmt, CoreAlgorithmStmtKind, CoreAlgorithmStmtTable,
        CoreAlgorithmTable, CoreCitation, CoreContractSet, CoreDefinition, CoreDefinitionTable,
        CoreDiagnostic, CoreDiagnosticClass, CoreDiagnosticTable, CoreFormula, CoreFormulaTable,
        CoreIrParts, CoreItem, CoreItemTable, CoreLabelRef, CoreProof, CoreProofNode,
        CoreProofNodeTable, CoreProofTable, CoreProvenance, CoreProvenanceKey, CoreSourceMap,
        CoreSourceRef, CoreTerm, CoreTermKind, CoreTermTable, DefinitionBody, ExpansionPolicy,
        GeneratedOrigin, GeneratedOriginKey, GeneratedOriginKind, GeneratedOriginTable,
        LocalProofOrProgramPath, NormalizedSemanticOrigin, ObligationSeed, ObligationSeedTable,
    };
    use mizar_resolve::resolved_ast::{FullyQualifiedName, LocalSymbolId, ModuleId, SymbolId};
    use mizar_session::snapshot::{ModulePath, PackageId};
    use mizar_session::{InMemorySessionIdAllocator, SessionIdAllocator, SourceRange};

    type CoreCorruption = (&'static str, Box<dyn Fn(&mut CoreIrParts)>);

    #[test]
    fn exact_task180_adapter_uses_structural_terminal_relation_without_marker() {
        let core = exact_core(exact_parts()).expect("exact CoreIr");
        let first = generate(&core).expect("exact Task-180 VC");
        let second = generate(&core).expect("deterministic exact Task-180 VC");

        assert_eq!(first, second);
        assert_eq!(first.debug_text(), second.debug_text());
        assert!(!first.debug_text().contains("vc-proof-goal:terminal"));
        assert_eq!(first.vcs().len(), 1);
        assert_eq!(first.vcs()[0].kind, VcKind::TerminalProofGoal);
        assert_eq!(first.vcs()[0].status, VcStatus::Open);
        assert_eq!(
            first.vcs()[0].anchor.completeness,
            AnchorCompleteness::Incomplete {
                missing: vec![AnchorIngredient::CanonicalGoalHash]
            }
        );
    }

    #[test]
    fn exact_task180_core_corruption_matrix_fails_closed_atomically() {
        let mut corruptions: Vec<CoreCorruption> = Vec::new();
        corruptions.push((
            "extra-table-row",
            Box::new(|parts| {
                let source = parts.items.get(CoreItemId::new(0)).unwrap().source.clone();
                let symbol = symbol(&parts.module_id, "Extra");
                let id = parts.items.insert(CoreItem::new(
                    symbol,
                    CoreItemKind::Theorem,
                    "public",
                    source.clone(),
                ));
                parts.source_map.item_sources.insert(id, source);
            }),
        ));
        corruptions.push((
            "wrong-owner-kind",
            Box::new(|parts| {
                parts.items.get_mut(CoreItemId::new(0)).unwrap().kind = CoreItemKind::Lemma;
            }),
        ));
        corruptions.push((
            "wrong-owner-visibility",
            Box::new(|parts| {
                parts.items.get_mut(CoreItemId::new(0)).unwrap().visibility = "private".into();
            }),
        ));
        corruptions.push((
            "wrong-owner-status",
            Box::new(|parts| {
                parts.items.get_mut(CoreItemId::new(0)).unwrap().status = CoreItemStatus::Partial;
            }),
        ));
        corruptions.push((
            "unexpected-owner-dependency",
            Box::new(|parts| {
                parts
                    .items
                    .get_mut(CoreItemId::new(0))
                    .unwrap()
                    .dependencies = vec![CoreItemId::new(0)];
            }),
        ));
        corruptions.push((
            "wrong-formula",
            Box::new(|parts| {
                parts.formulas.get_mut(CoreFormulaId::new(0)).unwrap().kind = CoreFormulaKind::True;
            }),
        ));
        corruptions.push((
            "wrong-proof-status",
            Box::new(|parts| {
                parts.proofs.get_mut(CoreProofId::new(0)).unwrap().status = CoreProofStatus::Open;
            }),
        ));
        corruptions.push((
            "wrong-proof-source",
            Box::new(|parts| {
                let proof = parts.proofs.get_mut(CoreProofId::new(0)).unwrap();
                let CoreSourceAnchor::SourceRange(mut range) = proof.source.anchor else {
                    panic!("direct source")
                };
                range.start += 1;
                proof.source.anchor = CoreSourceAnchor::SourceRange(range);
            }),
        ));
        corruptions.push((
            "wrong-proof-link",
            Box::new(|parts| {
                parts
                    .proof_nodes
                    .get_mut(CoreProofNodeId::new(0))
                    .unwrap()
                    .kind = CoreProofNodeKind::Sequence {
                    children: Vec::new(),
                };
            }),
        ));
        corruptions.push((
            "wrong-seed-kind",
            Box::new(|parts| {
                parts
                    .obligation_seeds
                    .get_mut(ObligationSeedId::new(0))
                    .unwrap()
                    .kind = ObligationSeedKind::DefinitionCorrectness;
            }),
        ));
        corruptions.push((
            "unexpected-terminal-citation",
            Box::new(|parts| {
                let owner = parts.items.get(CoreItemId::new(0)).unwrap().symbol.clone();
                let CoreProofNodeKind::TerminalGoal { citations, .. } = &mut parts
                    .proof_nodes
                    .get_mut(CoreProofNodeId::new(0))
                    .unwrap()
                    .kind
                else {
                    panic!("terminal goal")
                };
                citations.push(CoreCitation::Symbol(owner));
            }),
        ));
        corruptions.push((
            "unexpected-context",
            Box::new(|parts| {
                parts
                    .obligation_seeds
                    .get_mut(ObligationSeedId::new(0))
                    .unwrap()
                    .context = vec![CoreFormulaId::new(0)];
            }),
        ));
        corruptions.push((
            "wrong-path",
            Box::new(|parts| {
                parts
                    .obligation_seeds
                    .get_mut(ObligationSeedId::new(0))
                    .unwrap()
                    .local_path = LocalProofOrProgramPath::new("proof/1");
            }),
        ));
        corruptions.push((
            "unexpected-seed-label",
            Box::new(|parts| {
                parts
                    .obligation_seeds
                    .get_mut(ObligationSeedId::new(0))
                    .unwrap()
                    .label = Some(CoreLabelRef::new("A1"));
            }),
        ));
        corruptions.push((
            "wrong-origin",
            Box::new(|parts| {
                parts
                    .obligation_seeds
                    .get_mut(ObligationSeedId::new(0))
                    .unwrap()
                    .semantic_origin = NormalizedSemanticOrigin::new("stale::owner");
            }),
        ));
        corruptions.push((
            "wrong-seed-status",
            Box::new(|parts| {
                parts
                    .obligation_seeds
                    .get_mut(ObligationSeedId::new(0))
                    .unwrap()
                    .status = ObligationSeedStatus::Deferred;
            }),
        ));
        corruptions.push((
            "wrong-core-refs",
            Box::new(|parts| {
                parts
                    .obligation_seeds
                    .get_mut(ObligationSeedId::new(0))
                    .unwrap()
                    .core_refs = vec![CoreNodeRef::Formula(CoreFormulaId::new(0))];
            }),
        ));
        corruptions.push((
            "stale-source",
            Box::new(|parts| {
                let mut source = parts
                    .obligation_seeds
                    .get(ObligationSeedId::new(0))
                    .unwrap()
                    .source
                    .clone();
                let CoreSourceAnchor::SourceRange(mut range) = source.anchor else {
                    panic!("direct source")
                };
                range.start += 1;
                source.anchor = CoreSourceAnchor::SourceRange(range);
                parts
                    .obligation_seeds
                    .get_mut(ObligationSeedId::new(0))
                    .unwrap()
                    .source = source.clone();
                parts
                    .source_map
                    .obligation_sources
                    .insert(ObligationSeedId::new(0), source);
            }),
        ));
        corruptions.push((
            "same-prefix-resolver-provenance",
            Box::new(|parts| {
                replace_resolver_key(
                    parts,
                    "task267/v1;owner-fqn=20:pkg::main::Task180;origin-path=01:0",
                );
            }),
        ));
        corruptions.push((
            "same-prefix-statement-terminal-cross-row",
            Box::new(|parts| {
                replace_statement_key(
                    parts,
                    "task267/v1;statement=0;owner-node=0;formula=0;formula-site-node=6;formula-node=0",
                );
            }),
        ));
        corruptions.push((
            "same-prefix-proof-policy-provenance",
            Box::new(|parts| {
                parts
                    .proofs
                    .get_mut(CoreProofId::new(0))
                    .unwrap()
                    .source
                    .provenance[1]
                    .key = CoreProvenanceKey::new(
                    "task267/v1;proof=0;statement=0;policy=modified;justification=omitted;status=pending-automatic-proof",
                );
            }),
        ));
        corruptions.push((
            "same-prefix-terminal-provenance",
            Box::new(|parts| {
                replace_terminal_key(
                    parts,
                    "task267/v1;proof-node=0;terminal-goal=0;formula=0;formula-site-node=6;formula-node=0",
                );
            }),
        ));
        corruptions.push((
            "same-prefix-skeleton-provenance",
            Box::new(|parts| {
                replace_skeleton_key(parts, "task267/v1;local-path=8:proof/00");
            }),
        ));
        corruptions.push((
            "extra-term-table-row",
            Box::new(|parts| {
                let source = parts
                    .formulas
                    .get(CoreFormulaId::new(0))
                    .unwrap()
                    .source
                    .clone();
                let id = parts.terms.insert(CoreTerm::new(
                    CoreTermKind::Tuple(Vec::new()),
                    source.clone(),
                ));
                parts.source_map.term_sources.insert(id, source);
            }),
        ));
        corruptions.push((
            "extra-formula-table-row",
            Box::new(|parts| {
                let source = parts
                    .formulas
                    .get(CoreFormulaId::new(0))
                    .unwrap()
                    .source
                    .clone();
                let id = parts
                    .formulas
                    .insert(CoreFormula::new(CoreFormulaKind::True, source.clone()));
                parts.source_map.formula_sources.insert(id, source);
            }),
        ));
        corruptions.push((
            "extra-proof-table-row",
            Box::new(|parts| {
                let proof = parts.proofs.get(CoreProofId::new(0)).unwrap().clone();
                parts.proofs.insert(proof);
            }),
        ));
        corruptions.push((
            "extra-proof-node-table-row",
            Box::new(|parts| {
                let source = parts
                    .proof_nodes
                    .get(CoreProofNodeId::new(0))
                    .unwrap()
                    .source
                    .clone();
                let id = parts.proof_nodes.insert(CoreProofNode {
                    kind: CoreProofNodeKind::Sequence {
                        children: Vec::new(),
                    },
                    source: source.clone(),
                    diagnostics: Vec::new(),
                });
                parts.source_map.proof_sources.insert(id, source);
            }),
        ));
        corruptions.push((
            "extra-diagnostic-table-row",
            Box::new(|parts| {
                let source = parts
                    .formulas
                    .get(CoreFormulaId::new(0))
                    .unwrap()
                    .source
                    .clone();
                parts.diagnostics.insert(CoreDiagnostic::error(
                    CoreDiagnosticClass::MalformedProofSkeleton,
                    "task180.unexpected",
                    source,
                ));
            }),
        ));
        corruptions.push((
            "extra-definition-table-row",
            Box::new(|parts| {
                let source = parts.items.get(CoreItemId::new(0)).unwrap().source.clone();
                let id = parts.definitions.insert(CoreDefinition {
                    item: CoreItemId::new(0),
                    symbol: symbol(&parts.module_id, "ExtraDefinition"),
                    params: Vec::new(),
                    body: DefinitionBody::Formula(CoreFormulaId::new(0)),
                    expansion: ExpansionPolicy::Opaque,
                    correctness: Vec::new(),
                    generated_dependencies: Vec::new(),
                    source: source.clone(),
                });
                parts.source_map.definition_sources.insert(id, source);
            }),
        ));
        corruptions.push((
            "extra-algorithm-table-row",
            Box::new(|parts| {
                let source = parts.items.get(CoreItemId::new(0)).unwrap().source.clone();
                parts.algorithms.insert(CoreAlgorithm {
                    item: CoreItemId::new(0),
                    symbol: symbol(&parts.module_id, "ExtraAlgorithm"),
                    params: Vec::new(),
                    result: None,
                    contracts: CoreContractSet::default(),
                    statements: Vec::new(),
                    ghost_effects: Vec::new(),
                    source,
                    diagnostics: Vec::new(),
                });
            }),
        ));
        corruptions.push((
            "extra-algorithm-statement-requires-owner-row",
            Box::new(|parts| {
                let source = parts.items.get(CoreItemId::new(0)).unwrap().source.clone();
                let algorithm = parts.algorithms.insert(CoreAlgorithm {
                    item: CoreItemId::new(0),
                    symbol: symbol(&parts.module_id, "ExtraAlgorithm"),
                    params: Vec::new(),
                    result: None,
                    contracts: CoreContractSet::default(),
                    statements: vec![mizar_core::core_ir::CoreAlgorithmStmtId::new(0)],
                    ghost_effects: Vec::new(),
                    source: source.clone(),
                    diagnostics: Vec::new(),
                });
                let statement = parts.algorithm_statements.insert(CoreAlgorithmStmt {
                    owner: algorithm,
                    kind: CoreAlgorithmStmtKind::Return(None),
                    source: source.clone(),
                    diagnostics: Vec::new(),
                });
                parts.source_map.algorithm_sources.insert(statement, source);
            }),
        ));
        corruptions.push((
            "extra-generated-origin-table-row",
            Box::new(|parts| {
                let source = parts
                    .formulas
                    .get(CoreFormulaId::new(0))
                    .unwrap()
                    .source
                    .clone();
                let id = parts.generated.insert(GeneratedOrigin {
                    owner: CoreItemId::new(0),
                    kind: GeneratedOriginKind::LocalAbbreviation,
                    key: GeneratedOriginKey::new("task180-extra-generated"),
                    functor: None,
                    params: Vec::new(),
                    evidence: Vec::new(),
                    source: source.clone(),
                });
                parts.source_map.generated_sources.insert(id, source);
            }),
        ));
        corruptions.push((
            "extra-obligation-seed-table-row",
            Box::new(|parts| {
                let source = parts
                    .obligation_seeds
                    .get(ObligationSeedId::new(0))
                    .unwrap()
                    .source
                    .clone();
                let id = parts.obligation_seeds.insert(ObligationSeed {
                    owner: CoreItemId::new(0),
                    kind: ObligationSeedKind::TheoremProof,
                    goal: Some(CoreFormulaId::new(0)),
                    context: Vec::new(),
                    local_path: LocalProofOrProgramPath::new("proof/1"),
                    label: None,
                    semantic_origin: NormalizedSemanticOrigin::new("pkg::main::Extra"),
                    provenance: source.provenance.clone(),
                    source: source.clone(),
                    core_refs: vec![CoreNodeRef::Item(CoreItemId::new(0))],
                    status: ObligationSeedStatus::Active,
                    diagnostics: Vec::new(),
                });
                parts.source_map.obligation_sources.insert(id, source);
            }),
        ));

        for (name, mutate) in corruptions {
            let mut parts = exact_parts();
            mutate(&mut parts);
            let core = CoreIr::try_new(parts)
                .unwrap_or_else(|error| panic!("{name} must remain generic CoreIr: {error}"));
            assert!(generate(&core).is_err(), "{name} reached a VC");
        }

        let empty = CoreIr::try_new(empty_parts()).expect("empty CoreIr is structurally valid");
        assert!(generate(&empty).is_err(), "missing rows reached a VC");
    }

    #[test]
    fn exact_task180_handoff_and_flow_boundaries_reject_corruption() {
        let core = exact_core(exact_parts()).expect("exact CoreIr");
        let flow = build_control_flow_ir(&core);
        assert!(flow.flows.is_empty());
        assert!(flow.flow_map.is_empty());
        let handoff = build_obligation_seed_handoff(&core, &flow);
        validate_exact_handoff(&core, &handoff).expect("exact handoff");
        let intake = SeedIntakeTable::try_from_handoff(&handoff).expect("fresh intake");
        validate_exact_intake(&intake).expect("exact intake");

        let mut missing_source = handoff.clone();
        missing_source.source_map.clear();
        assert!(validate_exact_handoff(&core, &missing_source).is_err());

        let mut duplicate = handoff.clone();
        let entry = duplicate
            .entries
            .get(ObligationHandoffId::new(0))
            .unwrap()
            .clone();
        duplicate.entries.insert(entry.clone());
        duplicate
            .source_map
            .insert(ObligationHandoffId::new(1), entry.seed.source.clone());
        assert!(validate_exact_handoff(&core, &duplicate).is_err());

        let mut wrong_origin = handoff.clone();
        wrong_origin
            .entries
            .get_mut(ObligationHandoffId::new(0))
            .unwrap()
            .origin = ObligationHandoffOrigin::FlowDerived {
            flow: mizar_core::control_flow::ControlFlowId::new(0),
            algorithm: mizar_core::core_ir::CoreAlgorithmId::new(0),
        };
        assert!(validate_exact_handoff(&core, &wrong_origin).is_err());

        let mut flow_site = handoff;
        flow_site
            .entries
            .get_mut(ObligationHandoffId::new(0))
            .unwrap()
            .flow_site = Some(ControlFlowObligationSite {
            kind: ControlFlowObligationSiteKind::Requires,
            ordinal: 0,
            statement: None,
            block: None,
            loop_id: None,
            exit: None,
            local: None,
            assignment_effect: None,
        });
        assert!(validate_exact_handoff(&core, &flow_site).is_err());
    }

    fn generate(core: &CoreIr) -> Result<VcSet, ExactTask180VcError> {
        generate_exact_task180_vc(ExactTask180VcInput {
            core,
            snapshot: snapshot_id(),
            generation_schema_version: &GenerationSchemaVersion::new("task31-generation-v1"),
            vc_schema_version: &VcSchemaVersion::new("task31-vc-v1"),
        })
    }

    fn exact_core(parts: CoreIrParts) -> Result<CoreIr, mizar_core::core_ir::CoreIrError> {
        CoreIr::try_new(parts)
    }

    fn exact_parts() -> CoreIrParts {
        let source_id = source_id();
        let module = module_id();
        let owner_range = SourceRange {
            source_id,
            start: 0,
            end: 66,
        };
        let formula_range = SourceRange {
            source_id,
            start: 52,
            end: 65,
        };
        let owner_symbol = symbol(&module, "Task180");
        let fqn = owner_symbol.fqn().as_str();
        let resolver = CoreProvenance::new(
            CoreProvenancePhase::Resolver,
            format!("task267/v1;owner-fqn={}:{};origin-path=0:", fqn.len(), fqn),
        );
        let statement = CoreProvenance::new(
            CoreProvenancePhase::Checker,
            "task267/v1;statement=0;owner-node=0;formula=0;formula-site-node=5;formula-node=0",
        );
        let proof_key = CoreProvenance::new(
            CoreProvenancePhase::Checker,
            "task267/v1;proof=0;statement=0;policy=unmodified;justification=omitted;status=pending-automatic-proof",
        );
        let terminal = CoreProvenance::new(
            CoreProvenancePhase::Checker,
            "task267/v1;proof-node=0;terminal-goal=0;formula=0;formula-site-node=5;formula-node=0",
        );
        let skeleton = CoreProvenance::new(
            CoreProvenancePhase::ProofSkeleton,
            "task267/v1;local-path=7:proof/0",
        );
        let item_source = CoreSourceRef::direct(owner_range)
            .with_provenance(vec![resolver.clone(), statement.clone()]);
        let formula_source = CoreSourceRef::direct(formula_range).with_provenance(vec![statement]);
        let proof_source =
            CoreSourceRef::direct(owner_range).with_provenance(vec![resolver, proof_key]);
        let terminal_source =
            CoreSourceRef::direct(formula_range).with_provenance(vec![terminal, skeleton]);
        let mut items = CoreItemTable::new();
        let item = items.insert(CoreItem::new(
            owner_symbol.clone(),
            CoreItemKind::Theorem,
            "public",
            item_source.clone(),
        ));
        let mut formulas = CoreFormulaTable::new();
        let formula = formulas.insert(CoreFormula::new(
            CoreFormulaKind::False,
            formula_source.clone(),
        ));
        let mut seeds = ObligationSeedTable::new();
        let seed = seeds.insert(ObligationSeed {
            owner: item,
            kind: ObligationSeedKind::TheoremProof,
            goal: Some(formula),
            context: Vec::new(),
            local_path: LocalProofOrProgramPath::new("proof/0"),
            label: None,
            semantic_origin: NormalizedSemanticOrigin::new(owner_symbol.fqn().as_str()),
            provenance: terminal_source.provenance.clone(),
            source: terminal_source.clone(),
            core_refs: vec![
                CoreNodeRef::Item(item),
                CoreNodeRef::Formula(formula),
                CoreNodeRef::Proof(CoreProofId::new(0)),
                CoreNodeRef::ProofNode(CoreProofNodeId::new(0)),
            ],
            status: ObligationSeedStatus::Active,
            diagnostics: Vec::new(),
        });
        let mut proof_nodes = CoreProofNodeTable::new();
        let proof_node = proof_nodes.insert(CoreProofNode {
            kind: CoreProofNodeKind::TerminalGoal {
                obligation: seed,
                citations: Vec::new(),
            },
            source: terminal_source.clone(),
            diagnostics: Vec::new(),
        });
        let mut proofs = CoreProofTable::new();
        proofs.insert(CoreProof {
            item,
            proposition: formula,
            root: proof_node,
            status: CoreProofStatus::PendingAutomaticProof,
            source: proof_source,
        });
        let mut source_map = CoreSourceMap::new();
        source_map.item_sources.insert(item, item_source);
        source_map.formula_sources.insert(formula, formula_source);
        source_map
            .proof_sources
            .insert(proof_node, terminal_source.clone());
        source_map.obligation_sources.insert(seed, terminal_source);

        CoreIrParts {
            source_id,
            module_id: module,
            items,
            terms: CoreTermTable::new(),
            formulas,
            definitions: CoreDefinitionTable::new(),
            proofs,
            proof_nodes,
            algorithms: CoreAlgorithmTable::new(),
            algorithm_statements: CoreAlgorithmStmtTable::new(),
            generated: GeneratedOriginTable::new(),
            obligation_seeds: seeds,
            source_map,
            diagnostics: CoreDiagnosticTable::new(),
        }
    }

    fn replace_resolver_key(parts: &mut CoreIrParts, key: &str) {
        let key = CoreProvenanceKey::new(key);
        let item = parts.items.get_mut(CoreItemId::new(0)).unwrap();
        item.source.provenance[0].key = key.clone();
        parts
            .source_map
            .item_sources
            .insert(CoreItemId::new(0), item.source.clone());
        parts
            .proofs
            .get_mut(CoreProofId::new(0))
            .unwrap()
            .source
            .provenance[0]
            .key = key;
    }

    fn replace_statement_key(parts: &mut CoreIrParts, key: &str) {
        let key = CoreProvenanceKey::new(key);
        let item = parts.items.get_mut(CoreItemId::new(0)).unwrap();
        item.source.provenance[1].key = key.clone();
        parts
            .source_map
            .item_sources
            .insert(CoreItemId::new(0), item.source.clone());
        let formula = parts.formulas.get_mut(CoreFormulaId::new(0)).unwrap();
        formula.source.provenance[0].key = key;
        parts
            .source_map
            .formula_sources
            .insert(CoreFormulaId::new(0), formula.source.clone());
    }

    fn replace_terminal_key(parts: &mut CoreIrParts, key: &str) {
        replace_terminal_provenance_entry(parts, 0, key);
    }

    fn replace_skeleton_key(parts: &mut CoreIrParts, key: &str) {
        replace_terminal_provenance_entry(parts, 1, key);
    }

    fn replace_terminal_provenance_entry(parts: &mut CoreIrParts, index: usize, key: &str) {
        let key = CoreProvenanceKey::new(key);
        let node = parts.proof_nodes.get_mut(CoreProofNodeId::new(0)).unwrap();
        node.source.provenance[index].key = key.clone();
        parts
            .source_map
            .proof_sources
            .insert(CoreProofNodeId::new(0), node.source.clone());
        let seed = parts
            .obligation_seeds
            .get_mut(ObligationSeedId::new(0))
            .unwrap();
        seed.provenance[index].key = key.clone();
        seed.source.provenance[index].key = key;
        parts
            .source_map
            .obligation_sources
            .insert(ObligationSeedId::new(0), seed.source.clone());
    }

    fn empty_parts() -> CoreIrParts {
        let mut parts = exact_parts();
        parts.items = CoreItemTable::new();
        parts.formulas = CoreFormulaTable::new();
        parts.proofs = CoreProofTable::new();
        parts.proof_nodes = CoreProofNodeTable::new();
        parts.obligation_seeds = ObligationSeedTable::new();
        parts.source_map = CoreSourceMap::new();
        parts
    }

    fn module_id() -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("main"))
    }

    fn symbol(module: &ModuleId, name: &str) -> SymbolId {
        SymbolId::new(
            module.clone(),
            LocalSymbolId::new(name),
            FullyQualifiedName::new(format!("pkg::main::{name}")),
        )
    }

    fn snapshot_id() -> BuildSnapshotId {
        BuildSnapshotId::from_published_schema_str(
            "mizar-session-build-snapshot-v1:\
             3131313131313131313131313131313131313131313131313131313131313131",
        )
        .expect("snapshot id")
    }

    fn source_id() -> mizar_session::SourceId {
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id())
            .expect("source id")
    }
}
