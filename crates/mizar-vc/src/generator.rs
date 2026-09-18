//! Verification-condition generation candidates.
//!
//! This module implements the task-6 and task-7 slices specified in
//! [generator.md](../../../doc/design/mizar-vc/en/generator.md): core
//! theorem/definition/generated candidates and the currently explicit
//! goal-bearing algorithm candidates from flow-derived obligation seeds.

use crate::vc_ir::{
    AnchorCompleteness, AnchorIngredient, AnchorLabel, AnchorLabelRole, AnchorOwner,
    CanonicalSortKey, CollectionLoopObligation, ContextEntry, ContextEntryId, ContextEntryKind,
    DefinitionOpacityOverride, DefinitionUnfoldRequest, GenerationSchemaVersion, LocalContext,
    LoopInvariantPhase, PolicyKey, PolicyValue, PremiseRef, ProofHint, RangeLoopObligation,
    RegistrationCorrectnessKind, SeedAccounting, SeedIntakeMapping, SeedIntakeTable,
    SeedNoVcReason, SeedOriginRef, SeedVcMapping, SeedVcRef, VcFormulaRef, VcId, VcIr, VcIrError,
    VcKind, VcModuleRef, VcProvenance, VcProvenancePhase, VcSchemaVersion, VcSet, VcSetParts,
    VcSourceRef, VcStatus, VcText, VerifierPolicyInput, canonical_goal_hash_marker,
    hash_marker_for_payload, local_context_hash_marker,
};
use mizar_core::{
    control_flow::{
        AssertionPlacement, ContractSiteKind, ContractSitePlacement, ControlFlowExitKind,
        ControlFlowId, ControlFlowIr, ControlFlowObligationSite, ControlFlowObligationSiteKind,
        ControlFlowOutput, LoopInvariantPlacement, ObligationHandoffId, ObligationHandoffOrigin,
        ObligationSeedHandoff,
    },
    core_ir::{
        CoreAlgorithmId, CoreFormulaId, CoreNodeRef, CoreSourceAnchor, CoreSourceRef,
        LocalProofOrProgramPath, NormalizedSemanticOrigin, ObligationSeed, ObligationSeedKind,
        ObligationSeedStatus,
    },
};
use mizar_session::{BuildSnapshotId, SourceId, SourceRange};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt::{self, Write as _},
};

mod task180;

pub use task180::{ExactTask180VcError, ExactTask180VcInput, generate_exact_task180_vc};

/// Generates one open symbolic request from the bounded source computation theorem.
pub fn generate_source_computation_request(
    core: &mizar_core::core_ir::CoreIr,
    snapshot: BuildSnapshotId,
    generation_schema: &GenerationSchemaVersion,
    vc_schema: &VcSchemaVersion,
) -> Result<VcSet, String> {
    use mizar_core::{
        control_flow::{ObligationHandoffEntry, ObligationHandoffTable},
        core_ir::{
            CoreFormulaKind, CoreItemKind, CoreItemStatus, CoreProofNodeKind, CoreProofStatus,
            CoreProvenance, CoreProvenancePhase, CoreTermKind,
        },
    };
    let invalid = || "computation.unsupported_core_request".to_owned();
    let range = |source: &CoreSourceRef| match source.anchor {
        CoreSourceAnchor::SourceRange(range)
            if range.source_id == core.source_id() && range.start < range.end =>
        {
            Ok(range)
        }
        _ => Err(invalid()),
    };
    if core.items().len() != 1
        || core.terms().len() != 2
        || core.formulas().len() != 1
        || core.proofs().len() != 1
        || core.proof_nodes().len() != 1
        || core.obligation_seeds().len() != 1
        || !core.algorithms().is_empty()
        || !core.algorithm_statements().is_empty()
        || !core.definitions().is_empty()
        || !core.generated().is_empty()
        || !core.diagnostics().is_empty()
    {
        return Err(invalid());
    }
    let (proof_id, proof) = core.proofs().iter().next().ok_or_else(invalid)?;
    let theorem = core.items().get(proof.item).ok_or_else(invalid)?;
    let terminal = core.proof_nodes().get(proof.root).ok_or_else(invalid)?;
    let CoreProofNodeKind::ComputationGoal { obligation, steps } = &terminal.kind else {
        return Err(invalid());
    };
    let seed = core
        .obligation_seeds()
        .get(*obligation)
        .ok_or_else(invalid)?;
    let formula = core.formulas().get(proof.proposition).ok_or_else(invalid)?;
    let CoreFormulaKind::Equals { left, right } = formula.kind else {
        return Err(invalid());
    };
    let left_term = core.terms().get(left).ok_or_else(invalid)?;
    let right_term = core.terms().get(right).ok_or_else(invalid)?;
    let mut refs = vec![
        CoreNodeRef::Item(proof.item),
        CoreNodeRef::Proof(proof_id),
        CoreNodeRef::ProofNode(proof.root),
        CoreNodeRef::Formula(proof.proposition),
    ];
    refs.sort();
    if theorem.kind != CoreItemKind::Theorem
        || theorem.status != CoreItemStatus::Valid
        || theorem.visibility.as_str() != "public"
        || theorem.symbol.module() != core.module_id()
        || !theorem.dependencies.is_empty()
        || !theorem.diagnostics.is_empty()
        || proof.status != CoreProofStatus::PendingAutomaticProof
        || !terminal.diagnostics.is_empty()
        || seed.owner != proof.item
        || seed.kind != ObligationSeedKind::TheoremProof
        || seed.status != ObligationSeedStatus::Active
        || seed.goal != Some(proof.proposition)
        || !seed.context.is_empty()
        || seed.label.is_some()
        || !seed.diagnostics.is_empty()
        || seed.core_refs != refs
        || left == right
        || left_term.kind != CoreTermKind::Numeral("0".into())
        || right_term.kind != CoreTermKind::Numeral("0".into())
        || steps.is_empty()
        || !steps.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(invalid());
    }
    let key = format!("checker/theorem/{}", theorem.symbol.fqn().as_str());
    let checker =
        |suffix: &str| CoreProvenance::new(CoreProvenancePhase::Checker, format!("{key}/{suffix}"));
    let mut owner_provenance = vec![
        checker("owner"),
        CoreProvenance::new(
            CoreProvenancePhase::Resolver,
            format!("resolver/theorem/{}", theorem.symbol.fqn().as_str()),
        ),
    ];
    owner_provenance.sort();
    let mut terminal_provenance = vec![
        checker("skeleton/terminal"),
        CoreProvenance::new(
            CoreProvenancePhase::ProofSkeleton,
            format!("{key}/skeleton"),
        ),
    ];
    terminal_provenance.sort();
    if theorem.source.provenance != owner_provenance
        || proof.source.provenance != [checker("skeleton/proof")]
        || formula.source.provenance != [checker("equality")]
        || left_term.source.provenance != [checker("term")]
        || right_term.source.provenance != [checker("term")]
        || terminal.source.provenance != terminal_provenance
        || seed.source != terminal.source
        || seed.provenance != [checker("skeleton/terminal")]
        || seed.local_path.as_str() != format!("proof/{}", theorem.symbol.fqn().as_str())
        || seed.semantic_origin.as_str() != format!("{}.proof", theorem.symbol.fqn().as_str())
    {
        return Err(invalid());
    }
    let owner_range = range(&theorem.source)?;
    let goal_range = range(&formula.source)?;
    let left_range = range(&left_term.source)?;
    let right_range = range(&right_term.source)?;
    let request_range = range(&terminal.source)?;
    if range(&proof.source)? != owner_range
        || goal_range.start <= owner_range.start
        || request_range.end >= owner_range.end
        || goal_range.end >= request_range.start
        || goal_range.start != left_range.start
        || goal_range.end != right_range.end
        || left_range.end >= right_range.start
        || left_range.end - left_range.start != 1
        || right_range.end - right_range.start != 1
    {
        return Err(invalid());
    }
    let mut entries = ObligationHandoffTable::new();
    let id = entries.insert(ObligationHandoffEntry {
        seed: seed.clone(),
        origin: ObligationHandoffOrigin::ExistingCore { seed: *obligation },
        flow_site: None,
    });
    let handoff = ObligationSeedHandoff {
        entries,
        source_map: BTreeMap::from([(id, seed.source.clone())]),
    };
    let intake = SeedIntakeTable::try_from_handoff(&handoff).map_err(|error| error.to_string())?;
    let package = core.module_id().package().as_str();
    let path = core.module_id().path().as_str();
    let module = VcModuleRef::new(format!(
        "package={}:{};module={}:{}",
        package.len(),
        package,
        path.len(),
        path
    ));
    let mut candidates = CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
        schema_version: generation_schema,
        module: &module,
        intake: &intake,
        handoff: &handoff,
        flow_output: None,
    })
    .map_err(|error| error.to_string())?;
    let [candidate] = candidates.candidates.as_mut_slice() else {
        return Err(invalid());
    };
    candidate.proof_hint = Some(ProofHint {
        citations: Vec::new(),
        unfold_requests: Vec::new(),
        premise_restrictions: Vec::new(),
        solver: None,
        max_axioms: None,
        timeout: None,
        computation: Some(crate::vc_ir::ComputationHint::SymbolicRequest(
            crate::vc_ir::ProofHintKey::new(format!("by-computation(steps:{steps})")),
        )),
        provenance: source_provenance(seed, &terminal.source),
    });
    candidate.source.related = vec![
        theorem.source.clone(),
        formula.source.clone(),
        left_term.source.clone(),
        right_term.source.clone(),
    ];
    CoreGenerationCandidateSet::try_normalize(VcNormalizationInput {
        schema_version: vc_schema,
        snapshot,
        source: core.source_id(),
        candidates: &candidates,
    })
    .map_err(|error| error.to_string())
}

/// Expands an authenticated local existential registration and its independent set gate.
pub fn generate_source_existential_registration(
    core: &mizar_core::core_ir::CoreIr,
    snapshot: BuildSnapshotId,
    generation_schema: &GenerationSchemaVersion,
    vc_schema: &VcSchemaVersion,
) -> Result<VcSet, String> {
    use crate::vc_ir::{
        ExpandedVcRef, ExpansionSchemaVersion, VcGeneratedFormula, VcGeneratedFormulaId,
        VcGeneratedFormulaKind, VcGeneratedFormulaShape,
    };
    use mizar_core::core_ir::*;
    let invalid = || "registration.vc.invalid_source_graph".to_owned();
    let range = |source: &CoreSourceRef| match source.anchor {
        CoreSourceAnchor::SourceRange(r) if r.source_id == core.source_id() && r.start < r.end => {
            Ok(r)
        }
        _ => Err(invalid()),
    };
    let contains = |outer: &CoreSourceRef, inner: &CoreSourceRef| -> Result<bool, String> {
        let (a, b) = (range(outer)?, range(inner)?);
        Ok(a.start <= b.start && b.end <= a.end)
    };
    let count = core.proofs().len();
    let definition_count = match count {
        1 => 1,
        3 => 2,
        _ => return Err(invalid()),
    };
    if core.items().len() != count + definition_count
        || core.definitions().len() != definition_count
        || core.generated().len() != count
        || core.proof_nodes().len() != 7 * count
        || core.terms().len() != definition_count + 4 * count
        || core.obligation_seeds().len() != 2 * count
        || !core.algorithms().is_empty()
        || !core.algorithm_statements().is_empty()
        || !core.diagnostics().is_empty()
    {
        return Err(invalid());
    }
    for (_, item) in core.items().iter() {
        if item.status != CoreItemStatus::Valid
            || item.symbol.module() != core.module_id()
            || item.visibility.as_str() != "public"
            || !item.dependencies.is_empty()
            || !item.diagnostics.is_empty()
        {
            return Err(invalid());
        }
    }
    let mut expected_formulas = BTreeSet::new();
    let mut expected_terms = BTreeSet::new();
    let mut expected_nodes = BTreeSet::new();
    let mut expected_seeds = BTreeSet::new();
    let mut patterns = BTreeSet::new();
    let mut binders_seen = BTreeSet::new();
    let mut generated_formulas = Vec::new();
    let mut vcs = Vec::new();
    let mut seed_accounting = Vec::new();
    let shared_formal = core
        .definitions()
        .iter()
        .next()
        .and_then(|(_, d)| d.params.first())
        .ok_or_else(invalid)?;
    binders_seen.insert(shared_formal.var);
    let mut previous_end = 0;
    for (proof_id, proof) in core.proofs().iter() {
        let owner = core.items().get(proof.item).ok_or_else(invalid)?;
        if range(&owner.source)?.start < previous_end {
            return Err(invalid());
        }
        previous_end = range(&owner.source)?.end;
        let origins = core
            .generated()
            .iter()
            .filter(|(_, origin)| origin.owner == proof.item)
            .collect::<Vec<_>>();
        let [(origin_id, origin)] = origins.as_slice() else {
            return Err(invalid());
        };
        let origin_id = *origin_id;
        let functor = origin.functor.as_ref().ok_or_else(invalid)?;
        if owner.kind != CoreItemKind::Registration
            || proof.status != CoreProofStatus::PendingAutomaticProof
            || origin.kind != GeneratedOriginKind::StableChoice
            || origin.key.as_str() != "choice:builtin.set"
            || !origin.params.is_empty()
            || origin.source != proof.source
            || origin.evidence.len() != 2
            || origin
                .evidence
                .iter()
                .any(|p| p.phase != CoreProvenancePhase::Checker)
            || functor.module() != core.module_id()
            || functor.local().as_str() != format!("{}::$choice_set", owner.symbol.local().as_str())
            || functor.fqn().as_str() != format!("{}::$choice_set", owner.symbol.fqn().as_str())
            || !contains(&owner.source, &proof.source)?
        {
            return Err(invalid());
        }
        let node = |id| core.proof_nodes().get(id).ok_or_else(invalid);
        let root = node(proof.root)?;
        let CoreProofNodeKind::CurrentGoal { thesis, child } = root.kind else {
            return Err(invalid());
        };
        let sequence = node(child)?;
        let CoreProofNodeKind::Sequence { children } = &sequence.kind else {
            return Err(invalid());
        };
        let [n_current, type_step, attr_step, terminal] = children.as_slice() else {
            return Err(invalid());
        };
        let nc = node(*n_current)?;
        let CoreProofNodeKind::CurrentGoal {
            thesis: n_goal,
            child: n_terminal,
        } = nc.kind
        else {
            return Err(invalid());
        };
        let nt = node(n_terminal)?;
        let CoreProofNodeKind::TerminalGoal {
            obligation: n_id,
            citations: n_citations,
        } = &nt.kind
        else {
            return Err(invalid());
        };
        for id in [
            proof.root, child, *n_current, n_terminal, *type_step, *attr_step, *terminal,
        ] {
            if !expected_nodes.insert(id) {
                return Err(invalid());
            }
        }
        let terminal = node(*terminal)?;
        let CoreProofNodeKind::TerminalGoal {
            obligation: parent_id,
            citations,
        } = &terminal.kind
        else {
            return Err(invalid());
        };
        let type_step = node(*type_step)?;
        let attr_step = node(*attr_step)?;
        let CoreProofNodeKind::Step {
            label: type_label,
            formula: witness_formula,
            justification: type_justification,
        } = &type_step.kind
        else {
            return Err(invalid());
        };
        let CoreProofNodeKind::Step {
            label: attr_label,
            formula: terminal_formula,
            justification: attr_justification,
        } = &attr_step.kind
        else {
            return Err(invalid());
        };
        let parent = core
            .obligation_seeds()
            .get(*parent_id)
            .ok_or_else(invalid)?;
        let nonempty = core.obligation_seeds().get(*n_id).ok_or_else(invalid)?;
        let required = parent
            .core_refs
            .iter()
            .filter_map(|reference| match reference {
                CoreNodeRef::Definition(id) => Some(*id),
                _ => None,
            })
            .collect::<Vec<_>>();
        if required.is_empty()
            || required.len() > definition_count
            || required.windows(2).any(|pair| pair[0] >= pair[1])
            || !expected_seeds.insert(*parent_id)
            || !expected_seeds.insert(*n_id)
        {
            return Err(invalid());
        }
        let definitions = required
            .iter()
            .map(|id| {
                let definition = core.definitions().get(*id).ok_or_else(invalid)?;
                let attribute = core
                    .items()
                    .get(definition.owner.item().ok_or_else(invalid)?)
                    .ok_or_else(invalid)?;
                let [formal] = definition.params.as_slice() else {
                    return Err(invalid());
                };
                let DefinitionBody::Formula(body) = definition.body else {
                    return Err(invalid());
                };
                if attribute.kind != CoreItemKind::Attribute
                    || definition.symbol != attribute.symbol
                    || definition.source != attribute.source
                    || definition.expansion != ExpansionPolicy::Transparent
                    || !definition.correctness.is_empty()
                    || !definition.generated_dependencies.is_empty()
                    || formal.role.as_str() != "definition-parameter"
                    || formal.source_name.is_some()
                    || !formal.source.provenance.is_empty()
                    || formal.var != shared_formal.var
                    || formal.source != shared_formal.source
                    || range(&attribute.source)?.end > range(&owner.source)?.start
                    || range(&formal.source)?.end > range(&attribute.source)?.start
                {
                    return Err(invalid());
                }
                Ok((*id, definition, attribute, formal, body))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let pattern = definitions
            .iter()
            .map(|(_, _, a, _, _)| a.symbol.clone())
            .collect::<BTreeSet<_>>();
        if pattern.len() != definitions.len() || !patterns.insert(pattern) {
            return Err(invalid());
        }
        let expected_citations = definitions
            .iter()
            .map(|(_, _, attribute, _, _)| {
                CoreCitation::Label(CoreLabelRef::new(format!(
                    "definition:{}",
                    attribute.symbol.fqn().as_str()
                )))
            })
            .collect::<Vec<_>>();
        if thesis != proof.proposition
            || parent.goal != Some(thesis)
            || nonempty.goal != Some(n_goal)
            || parent.kind != ObligationSeedKind::CheckerInitial
            || nonempty.kind != ObligationSeedKind::GeneratedNonEmptiness
            || parent.local_path.as_str() != "registration/existence"
            || nonempty.local_path.as_str() != "registration/choice-nonempty"
            || parent.semantic_origin != nonempty.semantic_origin
            || parent.semantic_origin.as_str().is_empty()
            || !n_citations.is_empty()
            || citations != &expected_citations
            || attr_justification.citations != *citations
            || type_justification.citations != [CoreCitation::Generated(origin_id)]
            || type_label.is_some()
            || attr_label.is_some()
            || type_justification.source != type_step.source
            || attr_justification.source != attr_step.source
            || terminal.source != attr_step.source
            || nonempty.source != proof.source
            || [&root.source, &sequence.source, &nc.source, &nt.source]
                .iter()
                .any(|s| **s != proof.source)
            || !contains(&parent.source, &proof.source)?
            || !contains(&owner.source, &parent.source)?
            || !contains(&proof.source, &type_step.source)?
            || !contains(&proof.source, &terminal.source)?
            || range(&type_step.source)?.end > range(&terminal.source)?.start
        {
            return Err(invalid());
        }
        for seed in [parent, nonempty] {
            if seed.owner != proof.item
                || seed.status != ObligationSeedStatus::Active
                || !seed.context.is_empty()
                || seed.label.is_some()
                || !seed.diagnostics.is_empty()
                || seed.provenance != seed.source.provenance
            {
                return Err(invalid());
            }
        }
        let formula = |id| core.formulas().get(id).ok_or_else(invalid);
        let term = |id| core.terms().get(id).ok_or_else(invalid);
        let CoreFormulaKind::Exists {
            binders,
            body: conjunction,
        } = &formula(thesis)?.kind
        else {
            return Err(invalid());
        };
        let [x] = binders.as_slice() else {
            return Err(invalid());
        };
        let CoreFormulaKind::Exists {
            binders,
            body: n_body,
        } = &formula(n_goal)?.kind
        else {
            return Err(invalid());
        };
        let [z] = binders.as_slice() else {
            return Err(invalid());
        };
        if !binders_seen.insert(x.var)
            || !binders_seen.insert(z.var)
            || x.var == z.var
            || x.ty_guard.is_some()
            || z.ty_guard.is_some()
            || x.source_name.is_some()
            || z.source_name.is_some()
            || x.role.as_str() != "registration-witness"
            || z.role.as_str() != "choice-nonempty"
            || x.source != owner.source
            || z.source != proof.source
        {
            return Err(invalid());
        }
        let CoreFormulaKind::And(conjuncts) = &formula(*conjunction)?.kind else {
            return Err(invalid());
        };
        let [parent_guard, parent_attributes @ ..] = conjuncts.as_slice() else {
            return Err(invalid());
        };
        if parent_attributes.len() != definitions.len() {
            return Err(invalid());
        }
        let type_subject = |id| match &formula(id)?.kind {
            CoreFormulaKind::TypePred { subject, ty } if ty.as_str() == "builtin.set" => {
                Ok(*subject)
            }
            _ => Err(invalid()),
        };
        let xt = type_subject(*parent_guard)?;
        let zt = type_subject(*n_body)?;
        let w = type_subject(*witness_formula)?;
        let terminal_attributes = match &formula(*terminal_formula)?.kind {
            CoreFormulaKind::Atom { .. } if definitions.len() == 1 => vec![*terminal_formula],
            CoreFormulaKind::And(attributes) if definitions.len() == 2 && attributes.len() == 2 => {
                attributes.clone()
            }
            _ => return Err(invalid()),
        };
        let CoreFormulaKind::Atom { args, .. } = &formula(terminal_attributes[0])?.kind else {
            return Err(invalid());
        };
        let [w1] = args.as_slice() else {
            return Err(invalid());
        };
        if xt == zt
            || w == *w1
            || term(xt)?.kind != CoreTermKind::Var(x.var)
            || term(zt)?.kind != CoreTermKind::Var(z.var)
            || term(xt)?.source != owner.source
            || term(zt)?.source != proof.source
            || term(w)?.kind
                != (CoreTermKind::Apply {
                    functor: functor.clone(),
                    args: Vec::new(),
                })
            || term(*w1)?.kind != term(w)?.kind
            || !contains(&type_step.source, &term(w)?.source)?
            || !contains(&terminal.source, &term(*w1)?.source)?
        {
            return Err(invalid());
        }
        expected_terms.extend([xt, zt, w, *w1]);
        for (witness, evidence) in [w, *w1].into_iter().zip(&origin.evidence) {
            let witness = term(witness)?;
            let [provenance] = witness.source.provenance.as_slice() else {
                return Err(invalid());
            };
            let node = provenance
                .key
                .as_str()
                .strip_prefix("registration/source-node#")
                .and_then(|node| node.parse::<usize>().ok())
                .ok_or_else(invalid)?;
            if evidence
                != &CoreProvenance::new(
                    CoreProvenancePhase::Checker,
                    format!(
                        "builtin-set-choice:Node(TypedNodeId({node})):{:?}",
                        range(&witness.source)?
                    ),
                )
            {
                return Err(invalid());
            }
        }
        let find = |kind: CoreFormulaKind| -> Result<CoreFormulaId, String> {
            let found = core
                .formulas()
                .iter()
                .filter(|(_, f)| f.kind == kind)
                .map(|(id, _)| id)
                .collect::<Vec<_>>();
            match found.as_slice() {
                [id] => Ok(*id),
                _ => Err(invalid()),
            }
        };
        let mut definition_formulas = Vec::new();
        let mut witness_attributes = Vec::new();
        for ((_, _, attribute, formal, body), (parent_attribute, terminal_attribute)) in definitions
            .iter()
            .zip(parent_attributes.iter().zip(&terminal_attributes))
        {
            let v = type_subject(formal.ty_guard.ok_or_else(invalid)?)?;
            if v == xt
                || v == zt
                || term(v)?.kind != CoreTermKind::Var(formal.var)
                || term(v)?.source != formal.source
                || formula(*parent_attribute)?.kind
                    != (CoreFormulaKind::Atom {
                        predicate: attribute.symbol.clone(),
                        args: vec![xt],
                    })
                || formula(*terminal_attribute)?.kind
                    != (CoreFormulaKind::Atom {
                        predicate: attribute.symbol.clone(),
                        args: vec![*w1],
                    })
            {
                return Err(invalid());
            }
            expected_terms.insert(v);
            let formal_attribute = find(CoreFormulaKind::Atom {
                predicate: attribute.symbol.clone(),
                args: vec![v],
            })?;
            let witness_attribute = find(CoreFormulaKind::Atom {
                predicate: attribute.symbol.clone(),
                args: vec![w],
            })?;
            let eq = find(CoreFormulaKind::Equals { left: v, right: v })?;
            if *body != eq && formula(*body)?.kind != CoreFormulaKind::Not(eq) {
                return Err(invalid());
            }
            expected_formulas.extend([
                formal.ty_guard.unwrap(),
                *body,
                eq,
                *parent_attribute,
                *terminal_attribute,
                formal_attribute,
                witness_attribute,
            ]);
            for (id, source) in [
                (formal.ty_guard.unwrap(), &attribute.source),
                (formal_attribute, &attribute.source),
                (*parent_attribute, &owner.source),
                (*terminal_attribute, &terminal.source),
                (witness_attribute, &type_step.source),
            ] {
                if &formula(id)?.source != source {
                    return Err(invalid());
                }
            }
            if !contains(&attribute.source, &formula(eq)?.source)?
                || !contains(&attribute.source, &formula(*body)?.source)?
            {
                return Err(invalid());
            }
            definition_formulas.push((formal.ty_guard.unwrap(), formal_attribute, *body));
            witness_attributes.push(witness_attribute);
        }
        expected_formulas.extend([
            *parent_guard,
            *conjunction,
            thesis,
            *n_body,
            n_goal,
            *witness_formula,
            *terminal_formula,
        ]);
        for (ids, source) in [
            (vec![*parent_guard, *conjunction, thesis], &owner.source),
            (vec![*n_body, n_goal], &proof.source),
            (vec![*witness_formula], &type_step.source),
            (vec![*terminal_formula], &terminal.source),
        ] {
            if ids
                .into_iter()
                .any(|id| !formula(id).is_ok_and(|f| &f.source == source))
            {
                return Err(invalid());
            }
        }
        let expected_refs = std::iter::once(CoreNodeRef::Item(proof.item))
            .chain(required.iter().copied().map(CoreNodeRef::Definition))
            .chain([
                CoreNodeRef::Generated(origin_id),
                CoreNodeRef::Term(w),
                CoreNodeRef::Term(*w1),
                CoreNodeRef::ObligationSeed(*n_id),
                CoreNodeRef::Formula(thesis),
                CoreNodeRef::Formula(*witness_formula),
                CoreNodeRef::Formula(*terminal_formula),
                CoreNodeRef::Proof(proof_id),
                CoreNodeRef::ProofNode(proof.root),
            ])
            .collect::<Vec<_>>();
        if parent.core_refs != expected_refs
            || nonempty.core_refs
                != [
                    CoreNodeRef::Item(proof.item),
                    CoreNodeRef::Generated(origin_id),
                    CoreNodeRef::Formula(n_goal),
                ]
        {
            return Err(invalid());
        }
        let mut provenance = generator_provenance(parent, "source-existential-registration");
        provenance.push(VcProvenance {
            phase: VcProvenancePhase::CoreHandoff,
            key: VcText::new(format!(
                "parent-refs={:?};nonempty-refs={:?}",
                parent.core_refs, nonempty.core_refs
            )),
            core: None,
        });
        let mut generated = |shape, kind| {
            let id = VcGeneratedFormulaId::new(generated_formulas.len());
            generated_formulas.push(VcGeneratedFormula {
                id,
                kind,
                shape,
                provenance: provenance.clone(),
            });
            VcFormulaRef::Generated(id)
        };
        // This fact is licensed only after replay of the independent bare-set inhabitation gate.
        let choice_type = generated(
            VcGeneratedFormulaShape::Ref(VcFormulaRef::Core(*witness_formula)),
            VcGeneratedFormulaKind::GeneratedTypeObligation,
        );
        let mut guarded_definitions = Vec::new();
        for (guard, formal_attribute, body) in definition_formulas {
            let not_guard = generated(
                VcGeneratedFormulaShape::Not(VcFormulaRef::Core(guard)),
                VcGeneratedFormulaKind::NegatedPremise,
            );
            let not_attribute = generated(
                VcGeneratedFormulaShape::Not(VcFormulaRef::Core(formal_attribute)),
                VcGeneratedFormulaKind::NegatedPremise,
            );
            let not_body = generated(
                VcGeneratedFormulaShape::Not(VcFormulaRef::Core(body)),
                VcGeneratedFormulaKind::NegatedPremise,
            );
            let forward = generated(
                VcGeneratedFormulaShape::Or(vec![not_attribute, VcFormulaRef::Core(body)]),
                VcGeneratedFormulaKind::Conjunction,
            );
            let backward = generated(
                VcGeneratedFormulaShape::Or(vec![not_body, VcFormulaRef::Core(formal_attribute)]),
                VcGeneratedFormulaKind::Conjunction,
            );
            let equivalent = generated(
                VcGeneratedFormulaShape::And(vec![forward, backward]),
                VcGeneratedFormulaKind::Conjunction,
            );
            let guarded_definition = generated(
                VcGeneratedFormulaShape::Or(vec![not_guard, equivalent]),
                VcGeneratedFormulaKind::Conjunction,
            );
            guarded_definitions.push(guarded_definition);
        }
        let attributed_goal = if witness_attributes.len() == 1 {
            VcFormulaRef::Core(witness_attributes[0])
        } else {
            generated(
                VcGeneratedFormulaShape::And(
                    witness_attributes
                        .iter()
                        .copied()
                        .map(VcFormulaRef::Core)
                        .collect(),
                ),
                VcGeneratedFormulaKind::Conjunction,
            )
        };
        let local_context =
            LocalContext::try_new(Vec::new(), Vec::new()).map_err(|e| e.to_string())?;
        let kind = VcKind::RegistrationStyleCorrectness {
            style: RegistrationCorrectnessKind::Registration,
        };
        let handoff = ObligationHandoffId::new(parent_id.index());
        let first_vc = vcs.len();
        let leaves = [VcFormulaRef::Core(*witness_formula), attributed_goal]
            .into_iter()
            .enumerate()
            .map(|(index, goal)| {
                let mut premises = vec![PremiseRef::GeneratedFact {
                    formula: choice_type,
                }];
                if index == 1 {
                    premises.extend(
                        guarded_definitions
                            .iter()
                            .copied()
                            .map(|formula| PremiseRef::GeneratedFact { formula }),
                    );
                }
                VcIr {
                    id: VcId::new(first_vc + index),
                    kind: kind.clone(),
                    source: VcSourceRef {
                        primary: parent.source.clone(),
                        related: vec![
                            proof.source.clone(),
                            nonempty.source.clone(),
                            type_step.source.clone(),
                            terminal.source.clone(),
                        ],
                    },
                    seed: SeedVcRef { handoff },
                    anchor: anchor_for_seed(AnchorForSeedInput {
                        schema_version: generation_schema,
                        seed: parent,
                        kind: &kind,
                        owner: AnchorOwner::Registration(proof.item),
                        label: None,
                        source: &parent.source,
                        goal,
                        local_context: &local_context,
                    }),
                    local_context: local_context.clone(),
                    premises,
                    goal,
                    proof_hint: None,
                    status: VcStatus::Open,
                    provenance: provenance.clone(),
                }
            })
            .collect::<Vec<_>>();
        vcs.extend(leaves);
        seed_accounting.extend([
            SeedAccounting {
                handoff,
                origin: SeedOriginRef::ExistingCore { seed: *parent_id },
                seed_status: parent.status,
                mapping: SeedVcMapping::Expanded {
                    vcs: (0..2)
                        .map(|i| ExpandedVcRef {
                            expansion_index: i,
                            vc: VcId::new(first_vc + i),
                        })
                        .collect(),
                    expansion_schema: ExpansionSchemaVersion::new(
                        "source-existential-registration-v1",
                    ),
                },
            },
            SeedAccounting {
                handoff: ObligationHandoffId::new(n_id.index()),
                origin: SeedOriginRef::ExistingCore { seed: *n_id },
                seed_status: nonempty.status,
                mapping: SeedVcMapping::NoConcreteVc {
                    reason: SeedNoVcReason::BuiltinSetInhabitation { origin: origin_id },
                },
            },
        ]);
    }
    let full = core
        .definitions()
        .iter()
        .map(|(_, definition)| definition.symbol.clone())
        .collect::<BTreeSet<_>>();
    let mut expected_patterns = full
        .iter()
        .map(|symbol| [symbol.clone()].into_iter().collect())
        .collect::<BTreeSet<_>>();
    expected_patterns.insert(full);
    if patterns != expected_patterns
        || expected_formulas.len() != core.formulas().len()
        || expected_terms.len() != core.terms().len()
        || expected_nodes.len() != core.proof_nodes().len()
        || expected_seeds.len() != core.obligation_seeds().len()
    {
        return Err(invalid());
    }
    let mut sources = CoreSourceMap::new();
    sources
        .item_sources
        .extend(core.items().iter().map(|(id, n)| (id, n.source.clone())));
    sources
        .term_sources
        .extend(core.terms().iter().map(|(id, n)| (id, n.source.clone())));
    sources
        .formula_sources
        .extend(core.formulas().iter().map(|(id, n)| (id, n.source.clone())));
    sources.definition_sources.extend(
        core.definitions()
            .iter()
            .map(|(id, definition)| (id, definition.source.clone())),
    );
    sources.generated_sources.extend(
        core.generated()
            .iter()
            .map(|(id, origin)| (id, origin.source.clone())),
    );
    sources.obligation_sources.extend(
        core.obligation_seeds()
            .iter()
            .map(|(id, n)| (id, n.source.clone())),
    );
    for (id, n) in core.proof_nodes().iter() {
        if !n.diagnostics.is_empty() {
            return Err(invalid());
        }
        sources.proof_sources.insert(id, n.source.clone());
    }
    if &sources != core.source_map() {
        return Err(invalid());
    }
    for source in sources
        .item_sources
        .values()
        .chain(sources.term_sources.values())
        .chain(sources.formula_sources.values())
        .chain(sources.proof_sources.values())
        .chain(sources.obligation_sources.values())
    {
        range(source)?;
        if source != &shared_formal.source {
            let [provenance] = source.provenance.as_slice() else {
                return Err(invalid());
            };
            if provenance.phase != CoreProvenancePhase::Checker
                || provenance
                    .key
                    .as_str()
                    .strip_prefix("registration/source-node#")
                    .and_then(|node| node.parse::<usize>().ok())
                    .is_none()
            {
                return Err(invalid());
            }
        }
    }
    seed_accounting.sort_by_key(|row| row.handoff);
    VcSet::try_new(VcSetParts {
        schema_version: vc_schema.clone(),
        snapshot,
        source: core.source_id(),
        module: VcModuleRef::new(format!(
            "package={}:{};module={}:{}",
            core.module_id().package().as_str().len(),
            core.module_id().package().as_str(),
            core.module_id().path().as_str().len(),
            core.module_id().path().as_str()
        )),
        generated_formulas,
        vcs,
        seed_accounting,
    })
    .map_err(|e| e.to_string())
}

/// Generates the open theorem obligation of an authenticated void-algorithm claim.
pub fn generate_source_void_claim(
    core: &mizar_core::core_ir::CoreIr,
    snapshot: BuildSnapshotId,
    generation_schema: &GenerationSchemaVersion,
    vc_schema: &VcSchemaVersion,
) -> Result<VcSet, String> {
    use mizar_core::{
        control_flow::{build_control_flow_ir, build_obligation_seed_handoff},
        core_ir::{
            CoreAlgorithmStmtKind, CoreContractSet, CoreFormulaKind, CoreItemKind, CoreItemStatus,
            CoreProofNodeKind, CoreProofStatus, CoreProvenance, CoreProvenancePhase, CoreTermKind,
        },
    };
    let invalid = || "algorithms.claim.unsupported_core".to_owned();
    let range = |source: &CoreSourceRef| match source.anchor {
        CoreSourceAnchor::SourceRange(range)
            if range.source_id == core.source_id() && range.start < range.end =>
        {
            Ok(range)
        }
        _ => Err(invalid()),
    };
    let contains = |outer: SourceRange, inner: SourceRange| {
        outer.start <= inner.start && inner.end <= outer.end
    };
    if core.items().len() != 2
        || core.algorithms().len() != 1
        || core.algorithm_statements().len() != 1
        || core.proofs().len() != 1
        || core.proof_nodes().len() != 2
        || core.obligation_seeds().len() != 1
        || core.terms().len() != 6
        || core.formulas().len() != 5
        || !core.definitions().is_empty()
        || !core.generated().is_empty()
        || !core.diagnostics().is_empty()
    {
        return Err(invalid());
    }
    let (algorithm_id, algorithm) = core.algorithms().iter().next().ok_or_else(invalid)?;
    let algorithm_item = core.items().get(algorithm.item).ok_or_else(invalid)?;
    let [statement_id] = algorithm.statements.as_slice() else {
        return Err(invalid());
    };
    let statement = core
        .algorithm_statements()
        .get(*statement_id)
        .ok_or_else(invalid)?;
    let (proof_id, proof) = core.proofs().iter().next().ok_or_else(invalid)?;
    let theorem = core.items().get(proof.item).ok_or_else(invalid)?;
    let root = core.proof_nodes().get(proof.root).ok_or_else(invalid)?;
    let CoreProofNodeKind::IntroduceBinder {
        binder: local,
        child,
    } = &root.kind
    else {
        return Err(invalid());
    };
    let terminal = core.proof_nodes().get(*child).ok_or_else(invalid)?;
    let CoreProofNodeKind::TerminalGoal {
        obligation,
        citations,
    } = &terminal.kind
    else {
        return Err(invalid());
    };
    let seed = core
        .obligation_seeds()
        .get(*obligation)
        .ok_or_else(invalid)?;
    let proposition = core.formulas().get(proof.proposition).ok_or_else(invalid)?;
    let CoreFormulaKind::Forall { binders, body } = &proposition.kind else {
        return Err(invalid());
    };
    let [quantifier] = binders.as_slice() else {
        return Err(invalid());
    };
    let guard = quantifier.ty_guard.ok_or_else(invalid)?;
    let local_guard = local.ty_guard.ok_or_else(invalid)?;
    let goal = seed.goal.ok_or_else(invalid)?;
    if algorithm_item.kind != CoreItemKind::Algorithm
        || algorithm_item.symbol != algorithm.symbol
        || algorithm.symbol.module() != core.module_id()
        || algorithm_item.status != CoreItemStatus::Valid
        || algorithm_item.visibility.as_str() != "public"
        || !algorithm_item.dependencies.is_empty()
        || !algorithm_item.diagnostics.is_empty()
        || !theorem.diagnostics.is_empty()
        || !algorithm.params.is_empty()
        || algorithm.result.is_some()
        || algorithm.contracts != CoreContractSet::default()
        || !algorithm.ghost_effects.is_empty()
        || !algorithm.diagnostics.is_empty()
        || statement.owner != algorithm_id
        || statement.kind != CoreAlgorithmStmtKind::Return(None)
        || !statement.diagnostics.is_empty()
        || theorem.kind != CoreItemKind::Theorem
        || theorem.status != CoreItemStatus::Valid
        || theorem.visibility.as_str() != "public"
        || theorem.symbol.module() != core.module_id()
        || theorem.dependencies != [algorithm.item]
        || proof.status != CoreProofStatus::PendingAutomaticProof
        || proof.root == *child
        || !root.diagnostics.is_empty()
        || !terminal.diagnostics.is_empty()
        || !citations.is_empty()
        || seed.owner != proof.item
        || seed.kind != ObligationSeedKind::TheoremProof
        || seed.status != ObligationSeedStatus::Active
        || seed.context != [local_guard]
        || seed.label.is_some()
        || !seed.diagnostics.is_empty()
        || quantifier.var == local.var
        || quantifier.role.as_str() != "quantifier"
        || local.role.as_str() != "proof-let"
        || quantifier.source_name.as_ref().is_none_or(String::is_empty)
        || local.source_name.as_ref().is_none_or(String::is_empty)
    {
        return Err(invalid());
    }
    let key = format!("checker/theorem/{}", theorem.symbol.fqn().as_str());
    let skeleton = format!("{key}/skeleton");
    let checker =
        |suffix: &str| CoreProvenance::new(CoreProvenancePhase::Checker, format!("{key}/{suffix}"));
    let terminal_provenance = checker("skeleton/terminal");
    let mut terminal_sources = vec![
        terminal_provenance.clone(),
        CoreProvenance::new(CoreProvenancePhase::ProofSkeleton, skeleton.clone()),
    ];
    terminal_sources.sort();
    let mut theorem_sources = vec![
        checker("owner"),
        CoreProvenance::new(
            CoreProvenancePhase::Resolver,
            format!("resolver/theorem/{}", theorem.symbol.fqn().as_str()),
        ),
    ];
    theorem_sources.sort();
    if theorem.source.provenance != theorem_sources
        || proof.source.provenance != [checker("skeleton/proof")]
        || proposition.source.provenance != [checker("proposition")]
        || quantifier.source.provenance != [checker("quantifier")]
        || !local.source.provenance.is_empty()
        || root.source.provenance != [checker("skeleton/let")]
        || terminal.source.provenance != terminal_sources
        || seed.source.provenance != terminal_sources
        || seed.provenance != [terminal_provenance]
        || !algorithm_item.source.provenance.is_empty()
        || seed.local_path.as_str() != format!("proof/{}", theorem.symbol.fqn().as_str())
        || seed.semantic_origin.as_str() != format!("{}.proof", theorem.symbol.fqn().as_str())
    {
        return Err(invalid());
    }
    for source in [&algorithm.source, &statement.source] {
        let [provenance] = source.provenance.as_slice() else {
            return Err(invalid());
        };
        if provenance.phase != CoreProvenancePhase::Checker
            || !provenance
                .key
                .as_str()
                .strip_prefix("algorithm/source-node#")
                .is_some_and(|id| id.parse::<usize>().is_ok())
        {
            return Err(invalid());
        }
    }
    let algorithm_range = range(&algorithm.source)?;
    let claim_range = range(&seed.source)?;
    let theorem_range = range(&theorem.source)?;
    let quantifier_range = range(&quantifier.source)?;
    let local_range = range(&local.source)?;
    let terminal_range = range(&terminal.source)?;
    let return_range = range(&statement.source)?;
    if range(&algorithm_item.source)? != algorithm_range
        || return_range.start <= algorithm_range.start
        || return_range.end >= algorithm_range.end
        || algorithm_range.end >= claim_range.start
        || claim_range.start >= theorem_range.start
        || theorem_range.end >= claim_range.end
        || range(&proof.source)? != theorem_range
        || range(&proposition.source)? != theorem_range
        || !contains(theorem_range, quantifier_range)
        || !contains(theorem_range, terminal_range)
        || quantifier_range.end >= local_range.start
        || local_range.end >= terminal_range.start
        || range(&root.source)? != local_range
    {
        return Err(invalid());
    }
    let mut used_terms = BTreeSet::new();
    let mut used_formulas = BTreeSet::from([proof.proposition]);
    for (binder, equality_id, guard_id, suffix, outer) in [
        (quantifier, *body, guard, "binder", theorem_range),
        (local, goal, local_guard, "proof-let", terminal_range),
    ] {
        let equality = core.formulas().get(equality_id).ok_or_else(invalid)?;
        let type_guard = core.formulas().get(guard_id).ok_or_else(invalid)?;
        let CoreFormulaKind::Equals { left, right } = equality.kind else {
            return Err(invalid());
        };
        let CoreFormulaKind::TypePred { subject, ref ty } = type_guard.kind else {
            return Err(invalid());
        };
        let equality_range = range(&equality.source)?;
        let binder_range = range(&binder.source)?;
        let guard_range = range(&type_guard.source)?;
        if !used_formulas.insert(equality_id)
            || !used_formulas.insert(guard_id)
            || ty.as_str() != "set"
            || equality.source.provenance != [checker("equality")]
            || type_guard.source.provenance != [checker(suffix)]
            || !contains(outer, equality_range)
            || !contains(binder_range, guard_range)
            || (suffix == "binder"
                && (equality_range.start <= quantifier_range.end
                    || equality_range.end >= local_range.start))
            || (suffix == "binder"
                && (guard_range.start != quantifier_range.start
                    || guard_range.end >= quantifier_range.end))
            || (suffix == "proof-let" && guard_range != local_range)
        {
            return Err(invalid());
        }
        for (id, term_suffix, term_outer) in [
            (left, "term", equality_range),
            (right, "term", equality_range),
            (subject, suffix, guard_range),
        ] {
            let term = core.terms().get(id).ok_or_else(invalid)?;
            if !used_terms.insert(id)
                || term.kind != CoreTermKind::Var(binder.var)
                || term.source.provenance != [checker(term_suffix)]
                || !contains(term_outer, range(&term.source)?)
                || (id == subject && range(&term.source)? != guard_range)
            {
                return Err(invalid());
            }
        }
        let left_range = range(&core.terms().get(left).ok_or_else(invalid)?.source)?;
        let right_range = range(&core.terms().get(right).ok_or_else(invalid)?.source)?;
        if equality_range.start != left_range.start
            || equality_range.end != right_range.end
            || (suffix == "proof-let"
                && (equality_range.start <= outer.start || equality_range.end >= outer.end))
            || left_range.end >= right_range.start
        {
            return Err(invalid());
        }
    }
    let mut references = vec![
        CoreNodeRef::Item(algorithm.item),
        CoreNodeRef::Item(proof.item),
        CoreNodeRef::Algorithm(algorithm_id),
        CoreNodeRef::Proof(proof_id),
        CoreNodeRef::ProofNode(*child),
        CoreNodeRef::Formula(goal),
        CoreNodeRef::Formula(local_guard),
    ];
    references.sort();
    if seed.core_refs != references
        || used_terms.len() != core.terms().len()
        || used_formulas.len() != core.formulas().len()
    {
        return Err(invalid());
    }
    let flow = build_control_flow_ir(core);
    let (flow_id, cfg) = flow.flows.iter().next().ok_or_else(invalid)?;
    if flow.flows.len() != 1
        || cfg.algorithm != algorithm_id
        || cfg.item != algorithm.item
        || !cfg.diagnostics.is_empty()
    {
        return Err(invalid());
    }
    let handoff = build_obligation_seed_handoff(core, &flow);
    if handoff.entries.len() != 2 {
        return Err(invalid());
    }
    for (_, entry) in handoff.entries.iter() {
        match entry.origin {
            ObligationHandoffOrigin::ExistingCore { seed: id }
                if id == *obligation && entry.seed == *seed && entry.flow_site.is_none() => {}
            ObligationHandoffOrigin::FlowDerived { flow, algorithm }
                if flow == flow_id
                    && algorithm == algorithm_id
                    && entry.seed.kind == ObligationSeedKind::AlgorithmTermination
                    && entry.seed.owner == cfg.item
                    && entry.seed.status == ObligationSeedStatus::Deferred
                    && entry.seed.goal.is_none()
                    && entry.seed.context.is_empty()
                    && entry.seed.diagnostics.is_empty()
                    && entry.flow_site.as_ref().is_some_and(|site| {
                        site.kind == ControlFlowObligationSiteKind::PartialTermination
                    }) => {}
            _ => return Err(invalid()),
        }
    }
    let intake = SeedIntakeTable::try_from_handoff(&handoff).map_err(|error| error.to_string())?;
    let package = core.module_id().package().as_str();
    let path = core.module_id().path().as_str();
    let module = VcModuleRef::new(format!(
        "package={}:{};module={}:{}",
        package.len(),
        package,
        path.len(),
        path
    ));
    let candidates = CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
        schema_version: generation_schema,
        module: &module,
        intake: &intake,
        handoff: &handoff,
        flow_output: Some(&flow),
    })
    .map_err(|error| error.to_string())?;
    let raw = CoreGenerationCandidateSet::try_normalize(VcNormalizationInput {
        schema_version: vc_schema,
        snapshot,
        source: core.source_id(),
        candidates: &candidates,
    })
    .map_err(|error| error.to_string())?;
    let [vc] = raw.vcs() else {
        return Err(invalid());
    };
    let mut vc = vc.clone();
    vc.source.related = vec![
        algorithm.source.clone(),
        theorem.source.clone(),
        terminal.source.clone(),
    ];
    VcSet::try_new(VcSetParts {
        schema_version: vc_schema.clone(),
        snapshot,
        source: core.source_id(),
        module,
        generated_formulas: Vec::new(),
        vcs: vec![vc],
        seed_accounting: raw.seed_accounting().to_vec(),
    })
    .map_err(|error| error.to_string())
}

/// Generates open return and assertion obligations for authenticated flat object-state algorithms.
pub fn generate_source_algorithm_postconditions(
    core: &mizar_core::core_ir::CoreIr,
    snapshot: BuildSnapshotId,
    generation_schema: &GenerationSchemaVersion,
    vc_schema: &VcSchemaVersion,
) -> Result<VcSet, String> {
    use crate::vc_ir::{
        VcGeneratedFormula, VcGeneratedFormulaId, VcGeneratedFormulaKind, VcGeneratedFormulaShape,
        VcProgramValue,
    };
    use mizar_core::{
        control_flow::{
            AssignmentEffectTarget, ControlFlowStatementPlacement, ControlFlowTerminator,
            LocalKind, LocalMutability, Reachability, build_control_flow_ir,
            build_obligation_seed_handoff,
        },
        core_ir::{
            CoreAlgorithmStmtKind, CoreFormulaKind, CoreItemKind, CoreItemStatus,
            CoreProvenancePhase, CoreTermKind,
        },
    };
    let invalid = || "algorithms.postcondition.unsupported_core".to_owned();
    let range = |source: &CoreSourceRef| match source.anchor {
        CoreSourceAnchor::SourceRange(range)
            if range.source_id == core.source_id() && range.start < range.end =>
        {
            Ok(range)
        }
        _ => Err(invalid()),
    };
    let checked_source = |source: &CoreSourceRef| -> Result<(), String> {
        range(source)?;
        let [provenance] = source.provenance.as_slice() else {
            return Err(invalid());
        };
        if provenance.phase != CoreProvenancePhase::Checker
            || !provenance
                .key
                .as_str()
                .strip_prefix("algorithm/source-node#")
                .is_some_and(|id| id.parse::<usize>().is_ok())
        {
            return Err(invalid());
        }
        Ok(())
    };
    if core.items().len() != 1
        || core.algorithms().len() != 1
        || !core.definitions().is_empty()
        || !core.proofs().is_empty()
        || !core.proof_nodes().is_empty()
        || !core.generated().is_empty()
        || !core.obligation_seeds().is_empty()
        || !core.diagnostics().is_empty()
    {
        return Err(invalid());
    }
    let (algorithm_id, algorithm) = core.algorithms().iter().next().ok_or_else(invalid)?;
    let item = core.items().get(algorithm.item).ok_or_else(invalid)?;
    let [parameter] = algorithm.params.as_slice() else {
        return Err(invalid());
    };
    let result = algorithm.result.as_ref().ok_or_else(invalid)?;
    let statement_id = algorithm.statements.last().ok_or_else(invalid)?;
    let stateful = algorithm.statements.len() > 1;
    let mut statement_order = algorithm.statements.clone();
    let mut source_loop = None;
    if algorithm.statements.iter().any(|id| {
        core.algorithm_statements()
            .get(*id)
            .is_some_and(|row| matches!(row.kind, CoreAlgorithmStmtKind::While { .. }))
    }) {
        let [initial, loop_id, returned] = algorithm.statements.as_slice() else {
            return Err(invalid());
        };
        let CoreAlgorithmStmtKind::Let {
            binder,
            value: Some(_),
            ghost: false,
        } = &core
            .algorithm_statements()
            .get(*initial)
            .ok_or_else(invalid)?
            .kind
        else {
            return Err(invalid());
        };
        let loop_row = core
            .algorithm_statements()
            .get(*loop_id)
            .ok_or_else(invalid)?;
        let CoreAlgorithmStmtKind::While {
            condition,
            invariants,
            decreasing,
            body,
        } = &loop_row.kind
        else {
            return Err(invalid());
        };
        let [invariant] = invariants.as_slice() else {
            return Err(invalid());
        };
        let [assignment] = body.as_slice() else {
            return Err(invalid());
        };
        let assignment_row = core
            .algorithm_statements()
            .get(*assignment)
            .ok_or_else(invalid)?;
        if binder.role.as_str() != "local:var"
            || !decreasing.is_empty()
            || algorithm.contracts.ensures.len() != 1
            || !matches!(assignment_row.kind, CoreAlgorithmStmtKind::AssignLocal { target, .. } if target == binder.var)
            || range(&assignment_row.source)?.end >= range(&loop_row.source)?.end
            || range(
                &core
                    .algorithm_statements()
                    .get(*returned)
                    .ok_or_else(invalid)?
                    .source,
            )?
            .start
                <= range(&loop_row.source)?.end
        {
            return Err(invalid());
        }
        source_loop = Some((*loop_id, *condition, *invariant, *assignment));
        statement_order = vec![*initial, *loop_id, *assignment, *returned];
    }
    if core.algorithm_statements().len() != statement_order.len()
        || statement_order.iter().collect::<BTreeSet<_>>().len() != statement_order.len()
    {
        return Err(invalid());
    }
    let statement = core
        .algorithm_statements()
        .get(*statement_id)
        .ok_or_else(invalid)?;
    let CoreAlgorithmStmtKind::Return(Some(returned)) = statement.kind else {
        return Err(invalid());
    };
    let contracts = &algorithm.contracts;
    if item.kind != CoreItemKind::Algorithm
        || item.status != CoreItemStatus::Valid
        || item.visibility.as_str() != "public"
        || item.symbol != algorithm.symbol
        || item.symbol.module() != core.module_id()
        || item.source.anchor != algorithm.source.anchor
        || !item.source.provenance.is_empty()
        || !item.dependencies.is_empty()
        || !algorithm.ghost_effects.is_empty()
        || parameter.var == result.var
        || parameter.role.as_str() != "parameter"
        || result.role.as_str() != "result"
        || result.source_name.is_some()
        || !parameter.source.provenance.is_empty()
        || !contracts.requires.is_empty()
        || !contracts.assertions.is_empty()
        || !contracts.invariants.is_empty()
        || !contracts.decreasing.is_empty()
        || contracts.ensures.len() > 1
    {
        return Err(invalid());
    }
    checked_source(&algorithm.source)?;
    checked_source(&statement.source)?;
    checked_source(&result.source)?;
    let owner_range = range(&algorithm.source)?;
    let parameter_range = range(&parameter.source)?;
    let result_range = range(&result.source)?;
    let return_range = range(&statement.source)?;
    if parameter_range.end > owner_range.start
        || owner_range.start >= result_range.start
        || result_range.end >= return_range.start
        || return_range.end > owner_range.end
    {
        return Err(invalid());
    }
    for (_, term) in core.terms().iter() {
        checked_source(&term.source)?;
    }
    for (_, formula) in core.formulas().iter() {
        checked_source(&formula.source)?;
    }
    let mut binders = vec![parameter, result];
    for id in &algorithm.statements {
        let statement = core.algorithm_statements().get(*id).ok_or_else(invalid)?;
        if let CoreAlgorithmStmtKind::Let {
            binder,
            value: Some(_),
            ghost: _,
        } = &statement.kind
        {
            if !matches!(binder.role.as_str(), "local:var" | "local:const")
                || !binder.source.provenance.is_empty()
            {
                return Err(invalid());
            }
            binders.push(binder);
        }
    }
    if binders
        .iter()
        .map(|binder| binder.var)
        .collect::<BTreeSet<_>>()
        .len()
        != binders.len()
    {
        return Err(invalid());
    }
    let mut used_terms = BTreeSet::new();
    let mut used_formulas = BTreeSet::new();
    let parameter_guard = parameter.ty_guard.ok_or_else(invalid)?;
    for binder in &binders {
        let guard = binder.ty_guard.ok_or_else(invalid)?;
        let formula = core.formulas().get(guard).ok_or_else(invalid)?;
        let CoreFormulaKind::TypePred { subject, ty } = &formula.kind else {
            return Err(invalid());
        };
        let term = core.terms().get(*subject).ok_or_else(invalid)?;
        if ty.as_str() != "object"
            || term.kind != CoreTermKind::Var(binder.var)
            || term.source != formula.source
            || term.source.anchor != binder.source.anchor
            || (binder.var == result.var && term.source != binder.source)
            || !used_terms.insert(*subject)
            || !used_formulas.insert(guard)
        {
            return Err(invalid());
        }
    }
    let mut available = BTreeSet::from([parameter.var]);
    let mut assertions = BTreeMap::new();
    let mut visible = vec![parameter];
    let mut snapshot_names = BTreeSet::new();
    let mut previous_end = result_range.end;
    for id in &statement_order {
        let current = core.algorithm_statements().get(*id).ok_or_else(invalid)?;
        let current_range = range(&current.source)?;
        if current_range.start < previous_end || current_range.end > owner_range.end {
            return Err(invalid());
        }
        previous_end = current_range.end;
        if matches!(current.kind, CoreAlgorithmStmtKind::AssignLocal { .. }) {
            if current.source.provenance.len() != 2
                || current.source.provenance[0] == current.source.provenance[1]
            {
                return Err(invalid());
            }
            for provenance in &current.source.provenance {
                let mut single = current.source.clone();
                single.provenance = vec![provenance.clone()];
                checked_source(&single)?;
            }
        } else {
            checked_source(&current.source)?;
        }
        let terms = match &current.kind {
            CoreAlgorithmStmtKind::Let {
                binder,
                value: Some(value),
                ghost: _,
            } => {
                let declaration = range(&binder.source)?;
                if declaration.start <= current_range.start
                    || declaration.end >= current_range.end
                    || declaration.end
                        > range(&core.terms().get(*value).ok_or_else(invalid)?.source)?.start
                {
                    return Err(invalid());
                }
                vec![*value]
            }
            CoreAlgorithmStmtKind::AssignLocal { target, value }
                if binders
                    .iter()
                    .any(|binder| binder.var == *target && binder.role.as_str() == "local:var")
                    && available.contains(target) =>
            {
                vec![*value]
            }
            CoreAlgorithmStmtKind::Snapshot { name, captures } => {
                let expected = visible
                    .iter()
                    .enumerate()
                    .filter_map(|(index, binder)| {
                        (!visible[index + 1..]
                            .iter()
                            .any(|later| later.source_name == binder.source_name))
                        .then_some(binder.var)
                    })
                    .collect::<Vec<_>>();
                if name.is_empty() || !snapshot_names.insert(name) || *captures != expected {
                    return Err(invalid());
                }
                Vec::new()
            }
            CoreAlgorithmStmtKind::While {
                condition,
                invariants,
                ..
            } => {
                let outer = core.formulas().get(*condition).ok_or_else(invalid)?;
                let CoreFormulaKind::Not(inner) = outer.kind else {
                    return Err(invalid());
                };
                let invariant = invariants[0];
                let condition_range = range(&outer.source)?;
                let invariant_range =
                    range(&core.formulas().get(invariant).ok_or_else(invalid)?.source)?;
                if source_loop.is_none_or(|(loop_id, _, _, _)| loop_id != *id)
                    || condition_range.start <= current_range.start
                    || condition_range.end >= invariant_range.start
                    || invariant_range.end >= current_range.end
                    || !used_formulas.insert(*condition)
                {
                    return Err(invalid());
                }
                let mut operands = Vec::new();
                for formula in [inner, invariant] {
                    let formula_row = core.formulas().get(formula).ok_or_else(invalid)?;
                    let CoreFormulaKind::Equals { left, right } = formula_row.kind else {
                        return Err(invalid());
                    };
                    let span = range(&formula_row.source)?;
                    if !used_formulas.insert(formula)
                        || range(&core.terms().get(left).ok_or_else(invalid)?.source)?.start
                            != span.start
                        || range(&core.terms().get(right).ok_or_else(invalid)?.source)?.end
                            != span.end
                        || (formula == inner
                            && (span.start <= condition_range.start
                                || span.end != condition_range.end))
                    {
                        return Err(invalid());
                    }
                    operands.extend([left, right]);
                }
                previous_end = invariant_range.end;
                operands
            }
            CoreAlgorithmStmtKind::Assert { formula } => {
                let outer = core.formulas().get(*formula).ok_or_else(invalid)?;
                let outer_span = range(&outer.source)?;
                let entry = if let CoreFormulaKind::Not(child) = outer.kind {
                    let inner = core.formulas().get(child).ok_or_else(invalid)?;
                    let inner_span = range(&inner.source)?;
                    if !used_formulas.insert(child)
                        || inner_span.start <= outer_span.start
                        || inner_span.end != outer_span.end
                    {
                        return Err(invalid());
                    }
                    inner
                } else {
                    outer
                };
                let CoreFormulaKind::Equals { left, right } = entry.kind else {
                    return Err(invalid());
                };
                let span = range(&entry.source)?;
                if outer_span.start <= current_range.start
                    || outer_span.end >= current_range.end
                    || !used_formulas.insert(*formula)
                    || range(&core.terms().get(left).ok_or_else(invalid)?.source)?.start
                        != span.start
                    || range(&core.terms().get(right).ok_or_else(invalid)?.source)?.end != span.end
                {
                    return Err(invalid());
                }
                assertions.insert(*id, *formula);
                vec![left, right]
            }
            CoreAlgorithmStmtKind::Return(Some(value)) if id == statement_id => vec![*value],
            _ => return Err(invalid()),
        };
        let mut operand_end = current_range.start;
        for term_id in terms {
            let term = core.terms().get(term_id).ok_or_else(invalid)?;
            let span = range(&term.source)?;
            if !matches!(term.kind, CoreTermKind::Var(var) if available.contains(&var))
                || span.start < operand_end
                || span.end > current_range.end
                || !used_terms.insert(term_id)
            {
                return Err(invalid());
            }
            operand_end = span.end;
        }
        if let CoreAlgorithmStmtKind::Let { binder, .. } = &current.kind {
            available.insert(binder.var);
            visible.push(binder);
        }
    }
    let mut substituted = None;
    if let [ensures] = contracts.ensures.as_slice() {
        let formula = core.formulas().get(*ensures).ok_or_else(invalid)?;
        let CoreFormulaKind::Equals { left, right } = formula.kind else {
            return Err(invalid());
        };
        let ensures_range = range(&formula.source)?;
        let mut operands = Vec::new();
        let mut previous_end = ensures_range.start;
        for term_id in [left, right] {
            let term = core.terms().get(term_id).ok_or_else(invalid)?;
            let term_range = range(&term.source)?;
            if !used_terms.insert(term_id)
                || term_range.start < previous_end
                || term_range.end > ensures_range.end
            {
                return Err(invalid());
            }
            previous_end = term_range.end;
            operands.push(match term.kind {
                CoreTermKind::Var(var) if var == result.var => returned,
                CoreTermKind::Var(var) if var == parameter.var => term_id,
                _ => return Err(invalid()),
            });
        }
        if ensures_range.start != range(&core.terms().get(left).ok_or_else(invalid)?.source)?.start
            || ensures_range.end != range(&core.terms().get(right).ok_or_else(invalid)?.source)?.end
            || ensures_range.start <= result_range.end
            || ensures_range.end
                >= range(
                    &core
                        .algorithm_statements()
                        .get(algorithm.statements[0])
                        .ok_or_else(invalid)?
                        .source,
                )?
                .start
            || !used_formulas.insert(*ensures)
        {
            return Err(invalid());
        }
        substituted = Some((operands[0], operands[1]));
    }
    if used_formulas.len() != core.formulas().len() {
        return Err(invalid());
    }
    let header_terms = core
        .terms()
        .iter()
        .filter(|(id, _)| !used_terms.contains(id))
        .collect::<Vec<_>>();
    let [(_, header)] = header_terms.as_slice() else {
        return Err(invalid());
    };
    let header_range = range(&header.source)?;
    if header.kind != CoreTermKind::Var(parameter.var)
        || header_range.start <= owner_range.start
        || header_range.end >= result_range.start
    {
        return Err(invalid());
    }
    let flow_output = build_control_flow_ir(core);
    let (flow_id, flow) = flow_output.flows.iter().next().ok_or_else(invalid)?;
    let block = flow.blocks.get(flow.entry).ok_or_else(invalid)?;
    let (exit_id, exit) = flow.exits.iter().next().ok_or_else(invalid)?;
    let context = flow.contexts.get(block.context_in).ok_or_else(invalid)?;
    let parameter_local = flow
        .locals
        .iter()
        .find(|(_, local)| local.kind == LocalKind::Parameter)
        .ok_or_else(invalid)?;
    let return_block = if let Some((loop_statement, condition, invariant, assignment)) = source_loop
    {
        let Some(ControlFlowStatementPlacement::LoopHeader { loop_id, header }) =
            flow.source_map.statement_placements.get(&loop_statement)
        else {
            return Err(invalid());
        };
        let loop_row = flow.loops.get(*loop_id).ok_or_else(invalid)?;
        let header_block = flow.blocks.get(*header).ok_or_else(invalid)?;
        let body_block = flow.blocks.get(loop_row.body).ok_or_else(invalid)?;
        if flow.loops.len() != 1
            || flow.blocks.len() != 4
            || loop_row.header != *header
            || loop_row.algorithm != algorithm_id
            || loop_row.condition != condition
            || loop_row.invariants != [invariant]
            || !loop_row.decreasing.is_empty()
            || block.terminator != ControlFlowTerminator::Goto(*header)
            || header_block.terminator
                != (ControlFlowTerminator::Branch {
                    condition,
                    then_block: loop_row.body,
                    else_block: loop_row.exit,
                })
            || body_block.statements != [assignment]
            || body_block.terminator != ControlFlowTerminator::Goto(*header)
            || flow.contracts.loop_invariants.len() != 2
            || flow.termination.partial_sites.len() != 2
        {
            return Err(invalid());
        }
        loop_row.exit
    } else {
        flow.entry
    };
    let return_block_row = flow.blocks.get(return_block).ok_or_else(invalid)?;
    if flow_output.flows.len() != 1
        || flow.item != algorithm.item
        || flow.algorithm != algorithm_id
        || flow.symbol != algorithm.symbol
        || (source_loop.is_none() && flow.blocks.len() != 1)
        || flow.exits.len() != 1
        || flow.locals.len() != binders.len()
        || (!stateful && flow.contexts.len() != 1)
        || !flow.diagnostics.is_empty()
        || block.reachable != Reachability::Reachable
        || return_block_row.terminator != ControlFlowTerminator::Return(Some(returned))
        || exit.kind != ControlFlowExitKind::Return
        || exit.statement != Some(*statement_id)
        || exit.from != return_block
        || parameter_local.1.binder != *parameter
        || parameter_local.1.ghost
        || parameter_local.1.mutability != LocalMutability::Immutable
        || context.definitely_initialized != [parameter_local.0]
        || !context.available_facts.is_empty()
        || !context.path_conditions.is_empty()
        || (!stateful && !flow.assignment_effects.is_empty())
        || !flow.call_sites.is_empty()
        || (source_loop.is_none() && !flow.loops.is_empty())
        || (!stateful && !flow.context_facts.is_empty())
    {
        return Err(invalid());
    }
    let handoff = build_obligation_seed_handoff(core, &flow_output);
    let ghost_effects = &flow.ghost_effects.ghost_assignment_effects;
    if handoff.entries.len()
        != 1 + contracts.ensures.len()
            + assertions.len()
            + ghost_effects.len()
            + if source_loop.is_some() { 3 } else { 0 }
    {
        return Err(invalid());
    }
    for (_, entry) in handoff.entries.iter() {
        let site = entry.flow_site.as_ref().ok_or_else(invalid)?;
        if entry.origin
            != (ObligationHandoffOrigin::FlowDerived {
                flow: flow_id,
                algorithm: algorithm_id,
            })
            || entry.seed.owner != algorithm.item
            || entry.seed.status != ObligationSeedStatus::Deferred
            || !entry.seed.context.is_empty()
            || !entry.seed.diagnostics.is_empty()
        {
            return Err(invalid());
        }
        match site.kind {
            ControlFlowObligationSiteKind::Ensures
                if contracts.ensures.first().copied() == entry.seed.goal
                    && entry.seed.kind == ObligationSeedKind::AlgorithmContract
                    && site.statement == Some(*statement_id)
                    && site.block == Some(return_block)
                    && site.exit == Some(exit_id) => {}
            ControlFlowObligationSiteKind::StatementAssertion
                if site.statement.and_then(|id| assertions.get(&id)).copied()
                    == entry.seed.goal
                    && entry.seed.goal.is_some()
                    && entry.seed.kind == ObligationSeedKind::AlgorithmContract
                    && site.block == Some(flow.entry)
                    && site.exit.is_none() => {}
            ControlFlowObligationSiteKind::GhostAssignment
                if entry.seed.kind == ObligationSeedKind::GhostErasure
                    && entry.seed.goal.is_none()
                    && site.exit.is_none()
                    && site.block == Some(flow.entry)
                    && ghost_effects.get(site.ordinal).copied() == site.assignment_effect
                    && site.assignment_effect.and_then(|id| flow.assignment_effects.get(id)).is_some_and(|effect| {
                        let mut provenance = effect.source.provenance.clone();
                        provenance.push(mizar_core::core_ir::CoreProvenance::new(
                            CoreProvenancePhase::Generated,
                            format!("flow-handoff:ghost-assignment:{}", site.ordinal),
                        ));
                        Some(effect.statement) == site.statement
                            && entry.seed.source == effect.source.clone().with_provenance(provenance)
                            && matches!(effect.target, AssignmentEffectTarget::Local(local) if Some(local) == site.local && flow.locals.get(local).is_some_and(|local| local.ghost && local.algorithm == algorithm_id))
                    }) => {}
            ControlFlowObligationSiteKind::LoopInvariant
                if source_loop.is_some_and(|(_, _, invariant, _)| entry.seed.goal == Some(invariant))
                    && entry.seed.kind == ObligationSeedKind::AlgorithmContract
                    && matches!(flow_invariant_site(&entry.seed, site, flow), Some(LoopInvariantPhase::Entry | LoopInvariantPhase::Preservation)) => {}
            ControlFlowObligationSiteKind::PartialTermination
                if entry.seed.kind == ObligationSeedKind::AlgorithmTermination
                    && entry.seed.goal.is_none()
                    && site.statement.is_none()
                    && site.exit.is_none() => {}
            _ => return Err(invalid()),
        }
    }
    let intake = SeedIntakeTable::try_from_handoff(&handoff).map_err(|error| error.to_string())?;
    let package = core.module_id().package().as_str();
    let path = core.module_id().path().as_str();
    let module = VcModuleRef::new(format!(
        "package={}:{};module={}:{}",
        package.len(),
        package,
        path.len(),
        path
    ));
    let candidates = CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
        schema_version: generation_schema,
        module: &module,
        intake: &intake,
        handoff: &handoff,
        flow_output: Some(&flow_output),
    })
    .map_err(|error| error.to_string())?;
    let raw = CoreGenerationCandidateSet::try_normalize(VcNormalizationInput {
        schema_version: vc_schema,
        snapshot,
        source: core.source_id(),
        candidates: &candidates,
    })
    .map_err(|error| error.to_string())?;
    if raw.vcs().len()
        != contracts.ensures.len() + assertions.len() + if source_loop.is_some() { 2 } else { 0 }
        || raw.seed_accounting().len() != handoff.entries.len()
        || candidates
            .no_candidates()
            .iter()
            .map(|row| row.handoff)
            .collect::<BTreeSet<_>>()
            != handoff
                .entries
                .iter()
                .filter_map(|(id, entry)| {
                    matches!(
                        entry.seed.kind,
                        ObligationSeedKind::AlgorithmTermination | ObligationSeedKind::GhostErasure
                    )
                    .then_some(id)
                })
                .collect::<BTreeSet<_>>()
    {
        return Err(invalid());
    }
    if stateful {
        let mut generated = Vec::new();
        let mut values = BTreeMap::from([(
            parameter.var,
            VcProgramValue {
                var: parameter.var,
                definition: None,
            },
        )]);
        let mut entries = vec![ContextEntry {
            id: ContextEntryId::new(0),
            sort_key: "algorithm-state-00000000".into(),
            kind: ContextEntryKind::CheckerFact,
            formula: Some(VcFormulaRef::Core(parameter_guard)),
            provenance: Vec::new(),
        }];
        let mut projected = BTreeMap::new();
        let mut goals = BTreeMap::new();
        let mut preceding = BTreeSet::new();
        for id in &algorithm.statements {
            let current = core.algorithm_statements().get(*id).ok_or_else(invalid)?;
            let provenance = current
                .source
                .provenance
                .iter()
                .cloned()
                .map(|core| VcProvenance {
                    phase: VcProvenancePhase::CoreHandoff,
                    key: "algorithm-state".into(),
                    core: Some(core),
                })
                .collect::<Vec<_>>();
            let mut new_fact = None;
            match &current.kind {
                CoreAlgorithmStmtKind::Let {
                    binder,
                    value: Some(value),
                    ghost: _,
                } => {
                    let rhs =
                        source_program_value(core, *value, &mut values, Some((binder.var, *id)))?;
                    let lhs = values[&binder.var];
                    new_fact = Some((lhs, rhs, binder.ty_guard.ok_or_else(invalid)?));
                }
                CoreAlgorithmStmtKind::AssignLocal { target, value } => {
                    let rhs =
                        source_program_value(core, *value, &mut values, Some((*target, *id)))?;
                    let lhs = values[target];
                    let guard = binders
                        .iter()
                        .find(|binder| binder.var == *target)
                        .and_then(|binder| binder.ty_guard)
                        .ok_or_else(invalid)?;
                    new_fact = Some((lhs, rhs, guard));
                }
                CoreAlgorithmStmtKind::Snapshot { captures, .. } => {
                    let Some(ControlFlowStatementPlacement::Snapshot {
                        block,
                        context: saved,
                        captures: locals,
                    }) = flow.source_map.statement_placements.get(id)
                    else {
                        return Err(invalid());
                    };
                    let saved = flow.contexts.get(*saved).ok_or_else(invalid)?;
                    let expected_locals = captures
                        .iter()
                        .map(|var| {
                            flow.locals
                                .iter()
                                .find(|(_, local)| {
                                    local.binder.var == *var && local.algorithm == algorithm_id
                                })
                                .map(|(id, _)| id)
                                .ok_or_else(invalid)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let effects = flow
                        .assignment_effects
                        .iter()
                        .filter_map(|(id, effect)| {
                            preceding.contains(&effect.statement).then_some(id)
                        })
                        .collect::<Vec<_>>();
                    let initialized = flow
                        .locals
                        .iter()
                        .filter_map(|(id, local)| {
                            values.contains_key(&local.binder.var).then_some(id)
                        })
                        .collect::<Vec<_>>();
                    let facts = flow
                        .context_facts
                        .iter()
                        .filter_map(|(id, fact)| {
                            preceding
                                .iter()
                                .any(|statement| assertions.get(statement) == Some(&fact.formula))
                                .then_some(id)
                        })
                        .collect::<Vec<_>>();
                    if *block != flow.entry
                        || *locals != expected_locals
                        || captures.iter().any(|var| !values.contains_key(var))
                        || saved.assignment_effects != effects
                        || saved.definitely_initialized != initialized
                        || saved.available_facts != facts
                        || !saved.path_conditions.is_empty()
                        || !saved.maybe_assigned.is_empty()
                        || !saved.call_effects.is_empty()
                        || !saved.active_invariants.is_empty()
                        || !saved.loop_stack.is_empty()
                    {
                        return Err(invalid());
                    }
                }
                CoreAlgorithmStmtKind::While {
                    condition,
                    invariants,
                    body,
                    ..
                } => {
                    let invariant = invariants[0];
                    let assignment = body[0];
                    let CoreAlgorithmStmtKind::AssignLocal { target, value } = core
                        .algorithm_statements()
                        .get(assignment)
                        .ok_or_else(invalid)?
                        .kind
                    else {
                        return Err(invalid());
                    };
                    // The actual body is the complete MayWrite set for this bounded profile.
                    let target_guard = binders
                        .iter()
                        .find(|binder| binder.var == target)
                        .and_then(|binder| binder.ty_guard)
                        .ok_or_else(invalid)?;
                    let CoreFormulaKind::TypePred { ty, .. } =
                        &core.formulas().get(target_guard).ok_or_else(invalid)?.kind
                    else {
                        return Err(invalid());
                    };
                    let invariant_sites = handoff
                        .entries
                        .iter()
                        .filter_map(|(id, entry)| {
                            let site = entry.flow_site.as_ref()?;
                            (site.kind == ControlFlowObligationSiteKind::LoopInvariant)
                                .then(|| {
                                    flow_invariant_site(&entry.seed, site, flow)
                                        .map(|phase| (phase, id))
                                })
                                .flatten()
                        })
                        .collect::<BTreeMap<_, _>>();
                    if invariant_sites.len() != 2 {
                        return Err(invalid());
                    }
                    let entry_handoff = *invariant_sites
                        .get(&LoopInvariantPhase::Entry)
                        .ok_or_else(invalid)?;
                    let preservation_handoff = *invariant_sites
                        .get(&LoopInvariantPhase::Preservation)
                        .ok_or_else(invalid)?;
                    let emit = |generated: &mut Vec<VcGeneratedFormula>, shape, kind| {
                        let id = VcGeneratedFormulaId::new(generated.len());
                        generated.push(VcGeneratedFormula {
                            id,
                            kind,
                            shape,
                            provenance: provenance.clone(),
                        });
                        VcFormulaRef::Generated(id)
                    };
                    let instantiate = |formula,
                                       values: &BTreeMap<_, VcProgramValue>,
                                       generated: &mut Vec<VcGeneratedFormula>|
                     -> Result<VcFormulaRef, String> {
                        let outer = &core.formulas().get(formula).ok_or_else(invalid)?.kind;
                        let inner = if let CoreFormulaKind::Not(child) = outer {
                            &core.formulas().get(*child).ok_or_else(invalid)?.kind
                        } else {
                            outer
                        };
                        let CoreFormulaKind::Equals { left, right } = *inner else {
                            return Err(invalid());
                        };
                        let operand = |id| -> Result<VcProgramValue, String> {
                            let CoreTermKind::Var(var) =
                                core.terms().get(id).ok_or_else(invalid)?.kind
                            else {
                                return Err(invalid());
                            };
                            values.get(&var).copied().ok_or_else(invalid)
                        };
                        let formula = emit(
                            generated,
                            VcGeneratedFormulaShape::ProgramEquals {
                                left: operand(left)?,
                                right: operand(right)?,
                            },
                            VcGeneratedFormulaKind::AlgorithmStateFact,
                        );
                        Ok(if matches!(outer, CoreFormulaKind::Not(_)) {
                            emit(
                                generated,
                                VcGeneratedFormulaShape::Not(formula),
                                VcGeneratedFormulaKind::AlgorithmPathCondition,
                            )
                        } else {
                            formula
                        })
                    };
                    goals.insert(
                        entry_handoff,
                        instantiate(invariant, &values, &mut generated)?,
                    );
                    projected.insert(entry_handoff, entries.clone());
                    // Retain only the immutable parameter context across the cutpoint.
                    // The local's new value has no equality with its entry value.
                    entries.truncate(1);
                    values.insert(
                        target,
                        VcProgramValue {
                            var: target,
                            definition: Some(*id),
                        },
                    );
                    let head_values = values.clone();
                    let head_type = emit(
                        &mut generated,
                        VcGeneratedFormulaShape::ProgramTypePredicate {
                            subject: values[&target],
                            ty: ty.clone(),
                        },
                        VcGeneratedFormulaKind::AlgorithmStateFact,
                    );
                    let head_invariant = instantiate(invariant, &values, &mut generated)?;
                    let guard = instantiate(*condition, &values, &mut generated)?;
                    let context = |rows: &[(ContextEntryKind, VcFormulaRef)]| {
                        let mut context = entries.clone();
                        for (kind, formula) in rows {
                            let index = context.len();
                            context.push(ContextEntry {
                                id: ContextEntryId::new(index),
                                sort_key: format!("algorithm-state-{index:08}").into(),
                                kind: kind.clone(),
                                formula: Some(*formula),
                                provenance: provenance.clone(),
                            });
                        }
                        context
                    };
                    let rhs =
                        source_program_value(core, value, &mut values, Some((target, assignment)))?;
                    let assignment_fact = emit(
                        &mut generated,
                        VcGeneratedFormulaShape::ProgramEquals {
                            left: values[&target],
                            right: rhs,
                        },
                        VcGeneratedFormulaKind::AlgorithmStateFact,
                    );
                    let assignment_type = emit(
                        &mut generated,
                        VcGeneratedFormulaShape::ProgramTypePredicate {
                            subject: values[&target],
                            ty: ty.clone(),
                        },
                        VcGeneratedFormulaKind::AlgorithmStateFact,
                    );
                    projected.insert(
                        preservation_handoff,
                        context(&[
                            (ContextEntryKind::PostHavocFact, head_type),
                            (ContextEntryKind::LoopInvariantAvailable, head_invariant),
                            (ContextEntryKind::AlgorithmPathCondition, guard),
                            (ContextEntryKind::GeneratedFact, assignment_fact),
                            (ContextEntryKind::GeneratedFact, assignment_type),
                        ]),
                    );
                    goals.insert(
                        preservation_handoff,
                        instantiate(invariant, &values, &mut generated)?,
                    );
                    let exit_guard = emit(
                        &mut generated,
                        VcGeneratedFormulaShape::Not(guard),
                        VcGeneratedFormulaKind::AlgorithmPathCondition,
                    );
                    let exit_context = context(&[
                        (ContextEntryKind::PostHavocFact, head_type),
                        (ContextEntryKind::LoopInvariantAvailable, head_invariant),
                        (ContextEntryKind::AlgorithmPathCondition, exit_guard),
                    ]);
                    entries = exit_context;
                    values = head_values;
                }
                CoreAlgorithmStmtKind::Assert { formula } => {
                    let outer = &core.formulas().get(*formula).ok_or_else(invalid)?.kind;
                    let equality = if let CoreFormulaKind::Not(child) = outer {
                        &core.formulas().get(*child).ok_or_else(invalid)?.kind
                    } else {
                        outer
                    };
                    let CoreFormulaKind::Equals { left, right } = *equality else {
                        return Err(invalid());
                    };
                    let mut goal =
                        VcFormulaRef::Generated(VcGeneratedFormulaId::new(generated.len()));
                    generated.push(VcGeneratedFormula {
                        id: VcGeneratedFormulaId::new(generated.len()),
                        kind: VcGeneratedFormulaKind::AlgorithmAssertion,
                        shape: VcGeneratedFormulaShape::ProgramEquals {
                            left: source_program_value(core, left, &mut values, None)?,
                            right: source_program_value(core, right, &mut values, None)?,
                        },
                        provenance: provenance.clone(),
                    });
                    if matches!(outer, CoreFormulaKind::Not(_)) {
                        let id = VcGeneratedFormulaId::new(generated.len());
                        generated.push(VcGeneratedFormula {
                            id,
                            kind: VcGeneratedFormulaKind::AlgorithmAssertion,
                            shape: VcGeneratedFormulaShape::Not(goal),
                            provenance: provenance.clone(),
                        });
                        goal = VcFormulaRef::Generated(id);
                    }
                    let (handoff_id, _) = handoff
                        .entries
                        .iter()
                        .find(|(_, entry)| {
                            entry.flow_site.as_ref().is_some_and(|site| {
                                site.kind == ControlFlowObligationSiteKind::StatementAssertion
                                    && site.statement == Some(*id)
                            })
                        })
                        .ok_or_else(invalid)?;
                    projected.insert(handoff_id, entries.clone());
                    goals.insert(handoff_id, goal);
                    let index = entries.len();
                    entries.push(ContextEntry {
                        id: ContextEntryId::new(index),
                        sort_key: format!("algorithm-state-{index:08}").into(),
                        kind: ContextEntryKind::PendingAlgorithmAssertion {
                            handoff: handoff_id,
                        },
                        formula: Some(goal),
                        provenance: provenance.clone(),
                    });
                }
                CoreAlgorithmStmtKind::Return(Some(_)) => {
                    if let Some((left, right)) = substituted {
                        let goal =
                            VcFormulaRef::Generated(VcGeneratedFormulaId::new(generated.len()));
                        generated.push(VcGeneratedFormula {
                            id: VcGeneratedFormulaId::new(generated.len()),
                            kind: VcGeneratedFormulaKind::AlgorithmPostcondition,
                            shape: VcGeneratedFormulaShape::ProgramEquals {
                                left: source_program_value(core, left, &mut values, None)?,
                                right: source_program_value(core, right, &mut values, None)?,
                            },
                            provenance: provenance.clone(),
                        });
                        let (handoff_id, _) = handoff
                            .entries
                            .iter()
                            .find(|(_, entry)| {
                                entry.flow_site.as_ref().is_some_and(|site| {
                                    site.kind == ControlFlowObligationSiteKind::Ensures
                                        && site.statement == Some(*id)
                                })
                            })
                            .ok_or_else(invalid)?;
                        projected.insert(handoff_id, entries.clone());
                        goals.insert(handoff_id, goal);
                    }
                }
                _ => return Err(invalid()),
            }
            preceding.insert(*id);
            if let Some((left, right, guard)) = new_fact {
                let CoreFormulaKind::TypePred { ty, .. } =
                    &core.formulas().get(guard).ok_or_else(invalid)?.kind
                else {
                    return Err(invalid());
                };
                for shape in [
                    VcGeneratedFormulaShape::ProgramEquals { left, right },
                    VcGeneratedFormulaShape::ProgramTypePredicate {
                        subject: left,
                        ty: ty.clone(),
                    },
                ] {
                    let formula =
                        VcFormulaRef::Generated(VcGeneratedFormulaId::new(generated.len()));
                    generated.push(VcGeneratedFormula {
                        id: VcGeneratedFormulaId::new(generated.len()),
                        kind: VcGeneratedFormulaKind::AlgorithmStateFact,
                        shape,
                        provenance: provenance.clone(),
                    });
                    let index = entries.len();
                    entries.push(ContextEntry {
                        id: ContextEntryId::new(index),
                        sort_key: format!("algorithm-state-{index:08}").into(),
                        kind: ContextEntryKind::GeneratedFact,
                        formula: Some(formula),
                        provenance: provenance.clone(),
                    });
                }
            }
        }
        if raw.vcs().is_empty() {
            return Ok(raw);
        }
        let mut vcs = raw.vcs().to_vec();
        for vc in &mut vcs {
            let candidate = candidates
                .candidate_for_handoff(vc.seed.handoff)
                .ok_or_else(invalid)?;
            let entry = handoff.entries.get(vc.seed.handoff).ok_or_else(invalid)?;
            let site = entry.flow_site.as_ref().ok_or_else(invalid)?;
            let consuming_statement = site
                .statement
                .or_else(|| {
                    let (loop_statement, _, _, assignment) = source_loop?;
                    match flow_invariant_site(&entry.seed, site, flow)? {
                        LoopInvariantPhase::Entry => Some(loop_statement),
                        LoopInvariantPhase::Preservation => Some(assignment),
                        _ => None,
                    }
                })
                .ok_or_else(invalid)?;
            let statement = core
                .algorithm_statements()
                .get(consuming_statement)
                .ok_or_else(invalid)?;
            if vc.status != VcStatus::Open
                || vc.goal != VcFormulaRef::Core(entry.seed.goal.ok_or_else(invalid)?)
            {
                return Err(invalid());
            }
            vc.local_context = LocalContext::try_new(
                projected.remove(&vc.seed.handoff).ok_or_else(invalid)?,
                vc.local_context.policy_inputs().to_vec(),
            )
            .map_err(|error| error.to_string())?;
            vc.premises = vc
                .local_context
                .entries()
                .iter()
                .map(|entry| PremiseRef::LocalContext(entry.id))
                .collect();
            if source_loop.is_some() {
                vc.premises.push(PremiseRef::ConservativeUnknown {
                    reason: "unresolved source loop invariant obligations".into(),
                });
            }
            vc.goal = goals.remove(&vc.seed.handoff).ok_or_else(invalid)?;
            // Preserve the exact consuming program point separately from the contract formula's source.
            vc.source.related.push(statement.source.clone());
            vc.anchor = anchor_for_seed(AnchorForSeedInput {
                schema_version: generation_schema,
                seed: &entry.seed,
                kind: &vc.kind,
                owner: candidate.owner.clone(),
                label: candidate.label.clone(),
                source: &entry.seed.source,
                goal: vc.goal,
                local_context: &vc.local_context,
            });
        }
        if !projected.is_empty() || !goals.is_empty() {
            return Err(invalid());
        }
        return VcSet::try_new(VcSetParts {
            schema_version: vc_schema.clone(),
            snapshot,
            source: core.source_id(),
            module,
            generated_formulas: generated,
            vcs,
            seed_accounting: raw.seed_accounting().to_vec(),
        })
        .map_err(|error| error.to_string());
    }
    let Some((left, right)) = substituted else {
        return Ok(raw);
    };
    let [original] = raw.vcs() else {
        return Err(invalid());
    };
    let mut vc = original.clone();
    let candidate = candidates
        .candidate_for_handoff(vc.seed.handoff)
        .ok_or_else(invalid)?;
    let seed = &handoff
        .entries
        .get(vc.seed.handoff)
        .ok_or_else(invalid)?
        .seed;
    if vc.kind != VcKind::AlgorithmPostcondition
        || vc.status != VcStatus::Open
        || vc.goal != VcFormulaRef::Core(contracts.ensures[0])
    {
        return Err(invalid());
    }
    let provenance = core
        .formulas()
        .get(parameter_guard)
        .ok_or_else(invalid)?
        .source
        .provenance
        .iter()
        .cloned()
        .map(|core| VcProvenance {
            phase: VcProvenancePhase::CoreHandoff,
            key: "algorithm-parameter-type".into(),
            core: Some(core),
        })
        .collect();
    vc.local_context = LocalContext::try_new(
        vec![ContextEntry {
            id: ContextEntryId::new(0),
            sort_key: "algorithm-parameter-type-0000".into(),
            kind: ContextEntryKind::CheckerFact,
            formula: Some(VcFormulaRef::Core(parameter_guard)),
            provenance,
        }],
        vc.local_context.policy_inputs().to_vec(),
    )
    .map_err(|error| error.to_string())?;
    vc.premises = vec![PremiseRef::LocalContext(ContextEntryId::new(0))];
    vc.goal = VcFormulaRef::Generated(VcGeneratedFormulaId::new(0));
    vc.provenance.push(VcProvenance {
        phase: VcProvenancePhase::Generator,
        key: format!(
            "source-algorithm-postcondition-v1:{}",
            generation_schema.as_str()
        )
        .into(),
        core: None,
    });
    vc.anchor = anchor_for_seed(AnchorForSeedInput {
        schema_version: generation_schema,
        seed,
        kind: &vc.kind,
        owner: candidate.owner.clone(),
        label: candidate.label.clone(),
        source: &seed.source,
        goal: vc.goal,
        local_context: &vc.local_context,
    });
    let generated = VcGeneratedFormula {
        id: VcGeneratedFormulaId::new(0),
        kind: VcGeneratedFormulaKind::AlgorithmPostcondition,
        shape: VcGeneratedFormulaShape::Equals { left, right },
        provenance: vc.provenance.clone(),
    };
    VcSet::try_new(VcSetParts {
        schema_version: vc_schema.clone(),
        snapshot,
        source: core.source_id(),
        module,
        generated_formulas: vec![generated],
        vcs: vec![vc],
        seed_accounting: raw.seed_accounting().to_vec(),
    })
    .map_err(|error| error.to_string())
}

fn source_program_value(
    core: &mizar_core::core_ir::CoreIr,
    term: mizar_core::core_ir::CoreTermId,
    values: &mut BTreeMap<mizar_core::core_ir::CoreVarId, crate::vc_ir::VcProgramValue>,
    write: Option<(
        mizar_core::core_ir::CoreVarId,
        mizar_core::core_ir::CoreAlgorithmStmtId,
    )>,
) -> Result<crate::vc_ir::VcProgramValue, String> {
    let invalid = || "algorithms.postcondition.unsupported_core".to_owned();
    let mizar_core::core_ir::CoreTermKind::Var(var) =
        core.terms().get(term).ok_or_else(invalid)?.kind
    else {
        return Err(invalid());
    };
    let value = values.get(&var).copied().ok_or_else(invalid)?;
    if let Some((target, definition)) = write {
        values.insert(
            target,
            crate::vc_ir::VcProgramValue {
                var: target,
                definition: Some(definition),
            },
        );
    }
    Ok(value)
}

#[derive(Debug, Clone, Copy)]
pub struct CoreGenerationInput<'a> {
    pub schema_version: &'a GenerationSchemaVersion,
    pub module: &'a VcModuleRef,
    pub intake: &'a SeedIntakeTable,
    pub handoff: &'a ObligationSeedHandoff,
    pub flow_output: Option<&'a ControlFlowOutput>,
}

#[derive(Debug, Clone, Copy)]
pub struct VcNormalizationInput<'a> {
    pub schema_version: &'a VcSchemaVersion,
    pub snapshot: BuildSnapshotId,
    pub source: SourceId,
    pub candidates: &'a CoreGenerationCandidateSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreGenerationCandidateSet {
    schema_version: GenerationSchemaVersion,
    module: VcModuleRef,
    candidates: Vec<CoreGenerationCandidate>,
    no_candidates: Vec<CoreGenerationNoCandidate>,
}

impl CoreGenerationCandidateSet {
    pub fn try_from_seed_intake(input: CoreGenerationInput<'_>) -> Result<Self, GeneratorError> {
        let mut candidates = Vec::new();
        let mut no_candidates = Vec::new();
        let mut sort_keys = BTreeSet::new();
        let fresh_intake =
            SeedIntakeTable::try_from_handoff(input.handoff).map_err(GeneratorError::SeedIntake)?;

        if fresh_intake.rows() != input.intake.rows() {
            return Err(GeneratorError::IntakeHandoffMismatch {
                handoff: first_mismatched_handoff(input.intake.rows(), fresh_intake.rows()),
            });
        }

        for row in input.intake.rows() {
            let entry = input.handoff.entries.get(row.handoff).ok_or(
                GeneratorError::MissingHandoffEntry {
                    handoff: row.handoff,
                },
            )?;
            let seed = &entry.seed;
            let flow_id = flow_id_from_origin(&entry.origin);
            let flow_algorithm = flow_algorithm_from_origin(&entry.origin);
            let flow = flow_from_origin(input.flow_output, &entry.origin);

            match &row.mapping {
                SeedIntakeMapping::EligibleOneVc { goal } => {
                    if let Some(kind) = generation_kind(seed, entry.flow_site.as_ref(), flow) {
                        let candidate = build_candidate(BuildCandidateInput {
                            schema_version: input.schema_version,
                            module: input.module,
                            handoff: row.handoff,
                            origin: row.origin.clone(),
                            seed_status: row.seed_status,
                            seed,
                            flow_id,
                            flow_algorithm,
                            flow_site: entry.flow_site.as_ref(),
                            source: row.source.clone(),
                            goal: *goal,
                            kind,
                        })?;
                        if !sort_keys.insert(candidate.sort_key.clone()) {
                            return Err(GeneratorError::DuplicateCandidateSortKey {
                                sort_key: candidate.sort_key,
                            });
                        }
                        candidates.push(candidate);
                    } else {
                        no_candidates.push(no_candidate(
                            row.handoff,
                            row.origin.clone(),
                            row.seed_status,
                            no_candidate_reason_for_seed(
                                seed,
                                &entry.origin,
                                entry.flow_site.as_ref(),
                                input.flow_output,
                                None,
                            ),
                        ));
                    }
                }
                SeedIntakeMapping::NoConcreteVc { reason } => {
                    no_candidates.push(no_candidate(
                        row.handoff,
                        row.origin.clone(),
                        row.seed_status,
                        no_candidate_reason_for_seed(
                            seed,
                            &entry.origin,
                            entry.flow_site.as_ref(),
                            input.flow_output,
                            Some(reason),
                        ),
                    ));
                }
            }
        }

        Ok(Self {
            schema_version: input.schema_version.clone(),
            module: input.module.clone(),
            candidates,
            no_candidates,
        })
    }

    pub const fn schema_version(&self) -> &GenerationSchemaVersion {
        &self.schema_version
    }

    pub const fn module(&self) -> &VcModuleRef {
        &self.module
    }

    pub fn candidates(&self) -> &[CoreGenerationCandidate] {
        &self.candidates
    }

    pub fn no_candidates(&self) -> &[CoreGenerationNoCandidate] {
        &self.no_candidates
    }

    pub fn candidate_for_handoff(
        &self,
        handoff: ObligationHandoffId,
    ) -> Option<&CoreGenerationCandidate> {
        self.candidates
            .iter()
            .find(|candidate| candidate.handoff == handoff)
    }

    pub fn try_normalize(input: VcNormalizationInput<'_>) -> Result<VcSet, GeneratorError> {
        let mut candidates = input.candidates.candidates.iter().collect::<Vec<_>>();
        let mut seen_sort_keys = BTreeSet::new();

        for candidate in &candidates {
            if !seen_sort_keys.insert(candidate.sort_key.clone()) {
                return Err(GeneratorError::DuplicateCandidateSortKey {
                    sort_key: candidate.sort_key.clone(),
                });
            }
        }

        candidates.sort_by_key(|candidate| {
            (
                kind_classification_rank(&candidate.kind),
                candidate.sort_key.clone(),
                candidate.handoff,
            )
        });

        let mut seed_accounting = BTreeMap::new();
        let mut vcs = Vec::with_capacity(candidates.len());

        for (index, candidate) in candidates.into_iter().enumerate() {
            let id = VcId::new(index);
            insert_seed_accounting(
                &mut seed_accounting,
                SeedAccounting {
                    handoff: candidate.handoff,
                    origin: candidate.origin.clone(),
                    seed_status: candidate.seed_status,
                    mapping: SeedVcMapping::One { vc: id },
                },
            )?;
            vcs.push(VcIr {
                id,
                kind: candidate.kind.clone(),
                source: candidate.source.clone(),
                seed: SeedVcRef {
                    handoff: candidate.handoff,
                },
                anchor: candidate.anchor.clone(),
                local_context: candidate.local_context.clone(),
                premises: candidate.premises.clone(),
                goal: candidate.goal,
                proof_hint: candidate.proof_hint.clone(),
                status: candidate.status.clone(),
                provenance: normalized_provenance(candidate.provenance.clone()),
            });
        }

        let mut no_candidates = input.candidates.no_candidates.iter().collect::<Vec<_>>();
        no_candidates.sort_by_key(|no_candidate| no_candidate.handoff);
        for no_candidate in no_candidates {
            insert_seed_accounting(
                &mut seed_accounting,
                SeedAccounting {
                    handoff: no_candidate.handoff,
                    origin: no_candidate.origin.clone(),
                    seed_status: no_candidate.seed_status,
                    mapping: SeedVcMapping::NoConcreteVc {
                        reason: no_candidate.reason.clone(),
                    },
                },
            )?;
        }

        VcSet::try_new(VcSetParts {
            schema_version: input.schema_version.clone(),
            snapshot: input.snapshot,
            source: input.source,
            module: input.candidates.module.clone(),
            generated_formulas: Vec::new(),
            vcs,
            seed_accounting: seed_accounting.into_values().collect(),
        })
        .map_err(GeneratorError::VcSet)
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("core-generation-candidates-debug-v1\n");
        writeln!(&mut output, "schema-version: {:?}", self.schema_version).expect("write string");
        writeln!(&mut output, "module: {:?}", self.module).expect("write string");
        for candidate in &self.candidates {
            writeln!(
                &mut output,
                "candidate {:?}: sort-key={:?}; kind={:?}; goal={:?}; status={:?}; source={:?}",
                candidate.handoff,
                candidate.sort_key,
                candidate.kind,
                candidate.goal,
                candidate.status,
                candidate.source,
            )
            .expect("write string");
        }
        for no_candidate in &self.no_candidates {
            writeln!(
                &mut output,
                "no-candidate {:?}: origin={:?}; status={:?}; reason={:?}",
                no_candidate.handoff,
                no_candidate.origin,
                no_candidate.seed_status,
                no_candidate.reason,
            )
            .expect("write string");
        }
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreGenerationCandidate {
    pub handoff: ObligationHandoffId,
    pub origin: SeedOriginRef,
    pub seed_status: ObligationSeedStatus,
    pub sort_key: CanonicalSortKey,
    pub kind: VcKind,
    pub source: VcSourceRef,
    pub owner: AnchorOwner,
    pub local_path: LocalProofOrProgramPath,
    pub label: Option<AnchorLabel>,
    pub semantic_origin: NormalizedSemanticOrigin,
    pub local_context: LocalContext,
    pub premises: Vec<PremiseRef>,
    pub goal: VcFormulaRef,
    pub proof_hint: Option<ProofHint>,
    pub status: VcStatus,
    pub provenance: Vec<VcProvenance>,
    pub anchor: crate::vc_ir::ObligationAnchor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreGenerationNoCandidate {
    pub handoff: ObligationHandoffId,
    pub origin: SeedOriginRef,
    pub seed_status: ObligationSeedStatus,
    pub reason: SeedNoVcReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum GeneratorError {
    MissingHandoffEntry { handoff: ObligationHandoffId },
    IntakeHandoffMismatch { handoff: ObligationHandoffId },
    DuplicateCandidateSortKey { sort_key: CanonicalSortKey },
    DuplicateSeedOutput { handoff: ObligationHandoffId },
    LocalContext(VcIrError),
    SeedIntake(VcIrError),
    VcSet(VcIrError),
}

impl fmt::Display for GeneratorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHandoffEntry { handoff } => {
                write!(formatter, "seed intake references missing {handoff:?}")
            }
            Self::IntakeHandoffMismatch { handoff } => write!(
                formatter,
                "seed intake table no longer matches handoff entry {handoff:?}"
            ),
            Self::DuplicateCandidateSortKey { sort_key } => {
                write!(
                    formatter,
                    "duplicate generation candidate sort key {sort_key:?}"
                )
            }
            Self::DuplicateSeedOutput { handoff } => {
                write!(formatter, "duplicate generation output for {handoff:?}")
            }
            Self::LocalContext(error) => write!(formatter, "invalid generated context: {error}"),
            Self::SeedIntake(error) => write!(formatter, "invalid seed intake: {error}"),
            Self::VcSet(error) => write!(formatter, "invalid normalized VC set: {error}"),
        }
    }
}

impl Error for GeneratorError {}

fn first_mismatched_handoff(
    expected: &[crate::vc_ir::SeedIntakeRow],
    actual: &[crate::vc_ir::SeedIntakeRow],
) -> ObligationHandoffId {
    expected
        .iter()
        .zip(actual)
        .find_map(|(left, right)| (left != right).then_some(left.handoff))
        .or_else(|| expected.get(actual.len()).map(|row| row.handoff))
        .or_else(|| actual.get(expected.len()).map(|row| row.handoff))
        .unwrap_or_else(|| ObligationHandoffId::new(0))
}

struct BuildCandidateInput<'a> {
    schema_version: &'a GenerationSchemaVersion,
    module: &'a VcModuleRef,
    handoff: ObligationHandoffId,
    origin: SeedOriginRef,
    seed_status: ObligationSeedStatus,
    seed: &'a ObligationSeed,
    flow_id: Option<ControlFlowId>,
    flow_algorithm: Option<CoreAlgorithmId>,
    flow_site: Option<&'a ControlFlowObligationSite>,
    source: CoreSourceRef,
    goal: CoreFormulaId,
    kind: VcKind,
}

fn build_candidate(
    input: BuildCandidateInput<'_>,
) -> Result<CoreGenerationCandidate, GeneratorError> {
    let BuildCandidateInput {
        schema_version,
        module,
        handoff,
        origin,
        seed_status,
        seed,
        flow_id,
        flow_algorithm,
        flow_site,
        source,
        goal,
        kind,
    } = input;
    let local_context = local_context_from_seed(seed, flow_id, flow_site)?;
    let mut premises = local_context
        .entries()
        .iter()
        .map(|entry| PremiseRef::LocalContext(entry.id))
        .collect::<Vec<_>>();
    if let Some(label) = &seed.label {
        premises.push(PremiseRef::LocalLabel {
            label: label.clone(),
        });
    }
    let proof_hint = proof_hint_from_seed(seed);
    let provenance = generator_provenance(seed, generator_stage_key(&kind));
    let owner = owner_for_kind(&kind, seed, flow_algorithm);
    let label = seed.label.as_ref().map(|label| AnchorLabel {
        role: AnchorLabelRole::UserLabel,
        hint: Some(label.clone()),
    });
    let source_ref = VcSourceRef {
        primary: source.clone(),
        related: related_sources(&source, &seed.source),
    };
    let goal = VcFormulaRef::Core(goal);
    let anchor = anchor_for_seed(AnchorForSeedInput {
        schema_version,
        seed,
        kind: &kind,
        owner: owner.clone(),
        label: label.clone(),
        source: &source,
        goal,
        local_context: &local_context,
    });

    Ok(CoreGenerationCandidate {
        handoff,
        origin,
        seed_status,
        sort_key: candidate_sort_key(schema_version, module, handoff, seed, &kind),
        kind,
        source: source_ref,
        owner,
        local_path: seed.local_path.clone(),
        label,
        semantic_origin: seed.semantic_origin.clone(),
        local_context,
        premises,
        goal,
        proof_hint,
        status: VcStatus::Open,
        provenance,
        anchor,
    })
}

fn no_candidate(
    handoff: ObligationHandoffId,
    origin: SeedOriginRef,
    seed_status: ObligationSeedStatus,
    reason: SeedNoVcReason,
) -> CoreGenerationNoCandidate {
    CoreGenerationNoCandidate {
        handoff,
        origin,
        seed_status,
        reason,
    }
}

fn insert_seed_accounting(
    rows: &mut BTreeMap<ObligationHandoffId, SeedAccounting>,
    row: SeedAccounting,
) -> Result<(), GeneratorError> {
    let handoff = row.handoff;
    if rows.insert(handoff, row).is_some() {
        return Err(GeneratorError::DuplicateSeedOutput { handoff });
    }
    Ok(())
}

fn normalized_provenance(mut provenance: Vec<VcProvenance>) -> Vec<VcProvenance> {
    provenance.push(VcProvenance {
        phase: VcProvenancePhase::Normalization,
        key: VcText::new("task-8-normalized-vc"),
        core: None,
    });
    provenance
}

fn kind_classification_rank(kind: &VcKind) -> (u8, u8) {
    match kind {
        VcKind::TheoremProofStep => (0, 0),
        VcKind::TerminalProofGoal => (1, 0),
        VcKind::DefinitionCorrectness => (2, 0),
        VcKind::RegistrationStyleCorrectness { style } => {
            (3, registration_correctness_rank(*style))
        }
        VcKind::CheckerInitial => (4, 0),
        VcKind::GeneratedNonEmptiness => (5, 0),
        VcKind::GeneratedSethood => (6, 0),
        VcKind::FraenkelMembershipAxiom => (7, 0),
        VcKind::AlgorithmPrecondition => (8, 0),
        VcKind::AlgorithmPostcondition => (9, 0),
        VcKind::CallPrecondition => (10, 0),
        VcKind::AlgorithmAssertion => (11, 0),
        VcKind::LoopInvariant { phase } => (12, loop_invariant_rank(*phase)),
        VcKind::RangeLoop { obligation } => (13, range_loop_rank(*obligation)),
        VcKind::CollectionLoop { obligation } => (14, collection_loop_rank(*obligation)),
        VcKind::Termination => (15, 0),
        VcKind::PartialTermination => (16, 0),
        VcKind::GhostErasureSafety => (17, 0),
        VcKind::PolicyDeferredTraceability => (18, 0),
    }
}

fn registration_correctness_rank(style: RegistrationCorrectnessKind) -> u8 {
    match style {
        RegistrationCorrectnessKind::Registration => 0,
        RegistrationCorrectnessKind::Redefinition => 1,
        RegistrationCorrectnessKind::Reduction => 2,
        RegistrationCorrectnessKind::ExplicitCoreSeed => 3,
    }
}

fn loop_invariant_rank(phase: LoopInvariantPhase) -> u8 {
    match phase {
        LoopInvariantPhase::Entry => 0,
        LoopInvariantPhase::Preservation => 1,
        LoopInvariantPhase::Break => 2,
        LoopInvariantPhase::Continue => 3,
        LoopInvariantPhase::Exit => 4,
    }
}

fn range_loop_rank(obligation: RangeLoopObligation) -> u8 {
    match obligation {
        RangeLoopObligation::PositiveStep => 0,
        RangeLoopObligation::RangeBound => 1,
        RangeLoopObligation::HiddenIndex => 2,
    }
}

fn collection_loop_rank(obligation: CollectionLoopObligation) -> u8 {
    match obligation {
        CollectionLoopObligation::Finiteness => 0,
        CollectionLoopObligation::OrderIndependence => 1,
    }
}

fn generation_kind(
    seed: &ObligationSeed,
    flow_site: Option<&ControlFlowObligationSite>,
    flow: Option<&ControlFlowIr>,
) -> Option<VcKind> {
    task_six_kind(seed).or_else(|| task_seven_algorithm_kind(seed, flow_site?, flow?))
}

fn task_six_kind(seed: &ObligationSeed) -> Option<VcKind> {
    match seed.kind {
        ObligationSeedKind::TheoremProof => Some(
            if explicit_marker_values(seed, "vc-proof-goal").any(|value| value == "terminal") {
                VcKind::TerminalProofGoal
            } else {
                VcKind::TheoremProofStep
            },
        ),
        ObligationSeedKind::DefinitionCorrectness => registration_style_kind(seed)
            .map_or(Some(VcKind::DefinitionCorrectness), |style| {
                Some(VcKind::RegistrationStyleCorrectness { style })
            }),
        ObligationSeedKind::CheckerInitial => registration_style_kind(seed)
            .map_or(Some(VcKind::CheckerInitial), |style| {
                Some(VcKind::RegistrationStyleCorrectness { style })
            }),
        ObligationSeedKind::GeneratedNonEmptiness => Some(VcKind::GeneratedNonEmptiness),
        ObligationSeedKind::GeneratedSethood => Some(VcKind::GeneratedSethood),
        ObligationSeedKind::FraenkelMembershipAxiom => Some(VcKind::FraenkelMembershipAxiom),
        ObligationSeedKind::AlgorithmContract
        | ObligationSeedKind::AlgorithmTermination
        | ObligationSeedKind::GhostErasure => None,
        _ => None,
    }
}

fn task_seven_algorithm_kind(
    seed: &ObligationSeed,
    site: &ControlFlowObligationSite,
    flow: &ControlFlowIr,
) -> Option<VcKind> {
    if !matches!(seed.kind, ObligationSeedKind::AlgorithmContract) {
        return None;
    }

    match site.kind {
        ControlFlowObligationSiteKind::Requires => {
            flow_requires_site(seed, site, flow).then_some(VcKind::AlgorithmPrecondition)
        }
        ControlFlowObligationSiteKind::Ensures => {
            flow_ensures_site(seed, site, flow).then_some(VcKind::AlgorithmPostcondition)
        }
        ControlFlowObligationSiteKind::AlgorithmAssertion
        | ControlFlowObligationSiteKind::StatementAssertion => {
            flow_assertion_site(seed, site, flow).then_some(VcKind::AlgorithmAssertion)
        }
        ControlFlowObligationSiteKind::AlgorithmInvariant => flow_invariant_site(seed, site, flow)
            .is_some_and(|phase| phase == LoopInvariantPhase::Entry)
            .then_some(VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Entry,
            }),
        ControlFlowObligationSiteKind::LoopInvariant => Some(VcKind::LoopInvariant {
            phase: flow_invariant_site(seed, site, flow)?,
        }),
        ControlFlowObligationSiteKind::TerminationMeasure
        | ControlFlowObligationSiteKind::PartialTermination
        | ControlFlowObligationSiteKind::GhostPick
        | ControlFlowObligationSiteKind::GhostAssignment => None,
        _ => None,
    }
}

fn flow_requires_site(
    seed: &ObligationSeed,
    site: &ControlFlowObligationSite,
    flow: &ControlFlowIr,
) -> bool {
    let Some(contract) = flow.contracts.requires.get(site.ordinal) else {
        return false;
    };
    if contract.kind != ContractSiteKind::Requires || Some(contract.formula) != seed.goal {
        return false;
    }
    match contract.placement {
        ContractSitePlacement::Entry { block, .. } => {
            site.block.is_none_or(|site_block| site_block == block)
                && site.exit.is_none()
                && site.statement.is_none()
        }
        ContractSitePlacement::Return { block, exit } => {
            site.block.is_none_or(|site_block| site_block == block)
                && site.exit.is_none_or(|site_exit| site_exit == exit)
        }
        _ => false,
    }
}

fn flow_ensures_site(
    seed: &ObligationSeed,
    site: &ControlFlowObligationSite,
    flow: &ControlFlowIr,
) -> bool {
    let Some(contract) = flow.contracts.ensures.get(site.ordinal) else {
        return false;
    };
    if contract.kind != ContractSiteKind::Ensures || Some(contract.formula) != seed.goal {
        return false;
    }
    match contract.placement {
        ContractSitePlacement::Entry { block, .. } => {
            site.block.is_none_or(|site_block| site_block == block)
                && site.exit.is_none()
                && site.statement.is_none()
        }
        ContractSitePlacement::Return { block, exit } => {
            site.block.is_none_or(|site_block| site_block == block)
                && site.exit.is_none_or(|site_exit| site_exit == exit)
        }
        _ => false,
    }
}

fn flow_assertion_site(
    seed: &ObligationSeed,
    site: &ControlFlowObligationSite,
    flow: &ControlFlowIr,
) -> bool {
    let Some(assertion) = flow.contracts.assertions.get(site.ordinal) else {
        return false;
    };
    if Some(assertion.formula) != seed.goal {
        return false;
    }
    match assertion.placement {
        AssertionPlacement::AlgorithmContract { block, .. } => {
            site.kind == ControlFlowObligationSiteKind::AlgorithmAssertion
                && site.block.is_none_or(|site_block| site_block == block)
                && site.statement.is_none()
        }
        AssertionPlacement::Statement {
            statement, block, ..
        } => {
            site.kind == ControlFlowObligationSiteKind::StatementAssertion
                && site
                    .statement
                    .is_none_or(|site_statement| site_statement == statement)
                && site.block.is_none_or(|site_block| site_block == block)
        }
        _ => false,
    }
}

fn flow_invariant_site(
    seed: &ObligationSeed,
    site: &ControlFlowObligationSite,
    flow: &ControlFlowIr,
) -> Option<LoopInvariantPhase> {
    let invariant = flow.contracts.loop_invariants.get(site.ordinal)?;
    if Some(invariant.formula) != seed.goal {
        return None;
    }
    match invariant.placement {
        LoopInvariantPlacement::AlgorithmContract { block, .. } => (site.kind
            == ControlFlowObligationSiteKind::AlgorithmInvariant
            && site.block.is_none_or(|site_block| site_block == block)
            && site.loop_id.is_none()
            && site.exit.is_none())
        .then_some(LoopInvariantPhase::Entry),
        LoopInvariantPlacement::Header { loop_id, block } => (site.kind
            == ControlFlowObligationSiteKind::LoopInvariant
            && site.loop_id == Some(loop_id)
            && site.block == Some(block)
            && site.exit.is_none())
        .then_some(LoopInvariantPhase::Entry),
        LoopInvariantPlacement::NormalBackedge { loop_id, from, .. } => (site.kind
            == ControlFlowObligationSiteKind::LoopInvariant
            && site.loop_id == Some(loop_id)
            && site.block == Some(from)
            && site.exit.is_none())
        .then_some(LoopInvariantPhase::Preservation),
        LoopInvariantPlacement::BreakExit { loop_id, exit } => {
            let flow_exit = flow.exits.get(exit)?;
            (site.kind == ControlFlowObligationSiteKind::LoopInvariant
                && site.loop_id == Some(loop_id)
                && site.exit == Some(exit)
                && matches!(
                    &flow_exit.kind,
                    ControlFlowExitKind::Break { loop_id: exit_loop } if *exit_loop == loop_id
                ))
            .then_some(LoopInvariantPhase::Break)
        }
        LoopInvariantPlacement::ContinueExit { loop_id, exit } => {
            let flow_exit = flow.exits.get(exit)?;
            (site.kind == ControlFlowObligationSiteKind::LoopInvariant
                && site.loop_id == Some(loop_id)
                && site.exit == Some(exit)
                && matches!(
                    &flow_exit.kind,
                    ControlFlowExitKind::Continue { loop_id: exit_loop } if *exit_loop == loop_id
                ))
            .then_some(LoopInvariantPhase::Continue)
        }
        _ => None,
    }
}

fn registration_style_kind(seed: &ObligationSeed) -> Option<RegistrationCorrectnessKind> {
    explicit_marker_values(seed, "vc-registration-style").find_map(|value| match value {
        "registration" => Some(RegistrationCorrectnessKind::Registration),
        "redefinition" => Some(RegistrationCorrectnessKind::Redefinition),
        "reduction" | "reducibility" => Some(RegistrationCorrectnessKind::Reduction),
        "explicit-core-seed" | "explicit" => Some(RegistrationCorrectnessKind::ExplicitCoreSeed),
        _ => None,
    })
}

fn explicit_marker_values<'a>(
    seed: &'a ObligationSeed,
    marker: &'static str,
) -> impl Iterator<Item = &'a str> {
    seed.provenance.iter().filter_map(move |provenance| {
        provenance
            .key
            .as_str()
            .strip_prefix(marker)?
            .strip_prefix(':')
    })
}

fn deferred_task_six_kind(seed: &ObligationSeed) -> SeedNoVcReason {
    SeedNoVcReason::DeferredExternal(VcText::new(format!(
        "task 6 does not generate {:?} seeds",
        seed.kind
    )))
}

fn flow_id_from_origin(origin: &ObligationHandoffOrigin) -> Option<ControlFlowId> {
    match origin {
        ObligationHandoffOrigin::FlowDerived { flow, .. } => Some(*flow),
        _ => None,
    }
}

fn flow_algorithm_from_origin(origin: &ObligationHandoffOrigin) -> Option<CoreAlgorithmId> {
    match origin {
        ObligationHandoffOrigin::FlowDerived { algorithm, .. } => Some(*algorithm),
        _ => None,
    }
}

fn flow_from_origin<'a>(
    flow_output: Option<&'a ControlFlowOutput>,
    origin: &ObligationHandoffOrigin,
) -> Option<&'a ControlFlowIr> {
    let flow = flow_id_from_origin(origin)?;
    let algorithm = flow_algorithm_from_origin(origin)?;
    flow_output?
        .flows
        .get(flow)
        .filter(|flow_ir| flow_ir.algorithm == algorithm)
}

fn no_candidate_reason_for_seed(
    seed: &ObligationSeed,
    origin: &ObligationHandoffOrigin,
    flow_site: Option<&ControlFlowObligationSite>,
    flow_output: Option<&ControlFlowOutput>,
    intake_reason: Option<&SeedNoVcReason>,
) -> SeedNoVcReason {
    if matches!(
        seed.status,
        ObligationSeedStatus::Skipped | ObligationSeedStatus::Error
    ) {
        return intake_reason
            .cloned()
            .unwrap_or_else(|| deferred_task_six_kind(seed));
    }

    if matches!(
        seed.kind,
        ObligationSeedKind::AlgorithmContract
            | ObligationSeedKind::AlgorithmTermination
            | ObligationSeedKind::GhostErasure
    ) {
        return task_seven_no_candidate_reason(seed, origin, flow_site, flow_output);
    }

    intake_reason
        .cloned()
        .unwrap_or_else(|| deferred_task_six_kind(seed))
}

fn task_seven_no_candidate_reason(
    seed: &ObligationSeed,
    origin: &ObligationHandoffOrigin,
    flow_site: Option<&ControlFlowObligationSite>,
    flow_output: Option<&ControlFlowOutput>,
) -> SeedNoVcReason {
    let Some(flow) = flow_id_from_origin(origin) else {
        return SeedNoVcReason::DeferredExternal(VcText::new(format!(
            "task 7 requires FlowDerived origin for {:?} seed",
            seed.kind
        )));
    };
    let Some(site) = flow_site else {
        return SeedNoVcReason::DeferredExternal(VcText::new(format!(
            "task 7 requires explicit ControlFlowObligationSite for {:?} seed",
            seed.kind
        )));
    };
    if flow_from_origin(flow_output, origin).is_none() {
        return SeedNoVcReason::DeferredExternal(VcText::new(format!(
            "task 7 requires matching ControlFlowOutput for flow {}",
            flow.index()
        )));
    }
    if seed.goal.is_none() {
        return SeedNoVcReason::DeferredExternal(VcText::new(format!(
            "task 7 requires explicit goal formula for {:?} site {:?}",
            seed.kind, site.kind
        )));
    }
    SeedNoVcReason::DeferredExternal(VcText::new(format!(
        "task 7 does not generate {:?} site {:?}",
        seed.kind, site.kind
    )))
}

fn local_context_from_seed(
    seed: &ObligationSeed,
    flow_id: Option<ControlFlowId>,
    flow_site: Option<&ControlFlowObligationSite>,
) -> Result<LocalContext, GeneratorError> {
    let mut formulas = seed.context.iter().copied().enumerate().collect::<Vec<_>>();
    formulas.sort_by_key(|(source_index, formula)| (formula.index(), *source_index));

    let entries = formulas
        .into_iter()
        .enumerate()
        .map(|(entry_index, (source_index, formula))| ContextEntry {
            id: ContextEntryId::new(entry_index),
            sort_key: CanonicalSortKey::new(format!(
                "core-context-formula-{:08}-source-ordinal-{:08}",
                formula.index(),
                source_index
            )),
            kind: context_kind_for_seed(seed),
            formula: Some(VcFormulaRef::Core(formula)),
            provenance: context_provenance(seed, formula),
        })
        .collect();

    LocalContext::try_new(entries, policy_inputs_from_seed(seed, flow_id, flow_site))
        .map_err(GeneratorError::LocalContext)
}

fn policy_inputs_from_seed(
    seed: &ObligationSeed,
    flow_id: Option<ControlFlowId>,
    flow_site: Option<&ControlFlowObligationSite>,
) -> Vec<VerifierPolicyInput> {
    let mut inputs = theorem_status_policy_inputs(seed);
    inputs.extend(algorithm_site_policy_inputs(flow_id, flow_site));
    inputs.sort_by(|left, right| left.sort_key.cmp(&right.sort_key));
    inputs
}

fn context_kind_for_seed(seed: &ObligationSeed) -> ContextEntryKind {
    match seed.kind {
        ObligationSeedKind::TheoremProof => ContextEntryKind::ProofAssumption,
        ObligationSeedKind::DefinitionCorrectness | ObligationSeedKind::CheckerInitial => {
            ContextEntryKind::CheckerFact
        }
        ObligationSeedKind::GeneratedNonEmptiness
        | ObligationSeedKind::GeneratedSethood
        | ObligationSeedKind::FraenkelMembershipAxiom => ContextEntryKind::GeneratedFact,
        _ => ContextEntryKind::CheckerFact,
    }
}

fn proof_hint_from_seed(seed: &ObligationSeed) -> Option<ProofHint> {
    let citations = seed
        .label
        .iter()
        .cloned()
        .map(|label| PremiseRef::LocalLabel { label })
        .collect::<Vec<_>>();
    let unfold_requests = if explicit_marker_values(seed, "vc-unfold")
        .any(|value| matches!(value, "transparent" | "local" | "request" | "permitted"))
    {
        seed.core_refs
            .iter()
            .filter_map(|reference| match reference {
                CoreNodeRef::Definition(definition) => Some(DefinitionUnfoldRequest {
                    definition: *definition,
                    opacity_override: Some(DefinitionOpacityOverride::Transparent),
                }),
                _ => None,
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    if citations.is_empty() && unfold_requests.is_empty() {
        return None;
    }

    Some(ProofHint {
        citations,
        unfold_requests,
        premise_restrictions: Vec::new(),
        solver: None,
        max_axioms: None,
        timeout: None,
        computation: None,
        provenance: generator_provenance(seed, "task-6-core-candidate"),
    })
}

fn theorem_status_policy_inputs(seed: &ObligationSeed) -> Vec<VerifierPolicyInput> {
    if !matches!(seed.kind, ObligationSeedKind::TheoremProof) {
        return Vec::new();
    }

    let explicit = explicit_marker_values(seed, "vc-theorem-status").collect::<Vec<_>>();
    let statuses = ["non-clean", "clean", "open", "assumed", "conditional"]
        .into_iter()
        .filter(|status| explicit.contains(status))
        .collect::<Vec<_>>();

    statuses
        .into_iter()
        .enumerate()
        .map(|(index, status)| VerifierPolicyInput {
            sort_key: CanonicalSortKey::new(format!("{index:04}-theorem-status-{status}")),
            key: PolicyKey::new("theorem-status-dependency"),
            value: PolicyValue::new(status),
        })
        .collect()
}

fn algorithm_site_policy_inputs(
    flow_id: Option<ControlFlowId>,
    flow_site: Option<&ControlFlowObligationSite>,
) -> Vec<VerifierPolicyInput> {
    let Some(site) = flow_site else {
        return Vec::new();
    };
    let mut values = Vec::new();
    if let Some(flow) = flow_id {
        values.push(("flow", flow.index().to_string()));
    }
    values.push(("site-kind", format!("{:?}", site.kind)));
    values.push(("ordinal", site.ordinal.to_string()));
    if let Some(statement) = site.statement {
        values.push(("statement", statement.index().to_string()));
    }
    if let Some(block) = site.block {
        values.push(("block", block.index().to_string()));
    }
    if let Some(loop_id) = site.loop_id {
        values.push(("loop", loop_id.index().to_string()));
    }
    if let Some(exit) = site.exit {
        values.push(("exit", exit.index().to_string()));
    }
    if let Some(local) = site.local {
        values.push(("local", local.index().to_string()));
    }
    if let Some(effect) = site.assignment_effect {
        values.push(("assignment-effect", effect.index().to_string()));
    }

    values
        .into_iter()
        .enumerate()
        .map(|(index, (key, value))| VerifierPolicyInput {
            sort_key: CanonicalSortKey::new(format!("{index:04}-algorithm-site-{key}")),
            key: PolicyKey::new(format!("algorithm-site-{key}")),
            value: PolicyValue::new(value),
        })
        .collect()
}

fn context_provenance(seed: &ObligationSeed, formula: CoreFormulaId) -> Vec<VcProvenance> {
    seed.provenance
        .iter()
        .cloned()
        .map(|core| VcProvenance {
            phase: VcProvenancePhase::CoreHandoff,
            key: VcText::new(format!("context-formula-{}", formula.index())),
            core: Some(core),
        })
        .collect()
}

fn generator_provenance(seed: &ObligationSeed, stage_key: &'static str) -> Vec<VcProvenance> {
    let mut provenance = seed
        .provenance
        .iter()
        .cloned()
        .map(|core| VcProvenance {
            phase: VcProvenancePhase::CoreHandoff,
            key: VcText::new(format!("{:?}", seed.kind)),
            core: Some(core),
        })
        .collect::<Vec<_>>();
    provenance.push(VcProvenance {
        phase: VcProvenancePhase::Generator,
        key: VcText::new(stage_key),
        core: None,
    });
    provenance
}

fn generator_stage_key(kind: &VcKind) -> &'static str {
    if matches!(
        kind,
        VcKind::AlgorithmPrecondition
            | VcKind::AlgorithmPostcondition
            | VcKind::CallPrecondition
            | VcKind::AlgorithmAssertion
            | VcKind::LoopInvariant { .. }
            | VcKind::RangeLoop { .. }
            | VcKind::CollectionLoop { .. }
            | VcKind::Termination
            | VcKind::PartialTermination
            | VcKind::GhostErasureSafety
    ) {
        "task-7-algorithm-candidate"
    } else {
        "task-6-core-candidate"
    }
}

struct AnchorForSeedInput<'a> {
    schema_version: &'a GenerationSchemaVersion,
    seed: &'a ObligationSeed,
    kind: &'a VcKind,
    owner: AnchorOwner,
    label: Option<AnchorLabel>,
    source: &'a CoreSourceRef,
    goal: VcFormulaRef,
    local_context: &'a LocalContext,
}

fn anchor_for_seed(input: AnchorForSeedInput<'_>) -> crate::vc_ir::ObligationAnchor {
    let AnchorForSeedInput {
        schema_version,
        seed,
        kind,
        owner,
        label,
        source,
        goal,
        local_context,
    } = input;
    let anchor_provenance = source_provenance(seed, source);
    let source_shape_hash =
        source_shape_hash_marker(&owner, kind, seed, label.as_ref(), &anchor_provenance);
    let canonical_goal_hash = canonical_goal_hash_marker(goal);
    let canonical_context_hash = local_context_hash_marker(local_context);
    let mut missing = Vec::new();
    if anchor_provenance.is_empty() {
        missing.push(AnchorIngredient::SourceProvenance);
    }
    for (ingredient, marker) in [
        (AnchorIngredient::SourceShapeHash, &source_shape_hash),
        (AnchorIngredient::CanonicalGoalHash, &canonical_goal_hash),
        (
            AnchorIngredient::CanonicalContextHash,
            &canonical_context_hash,
        ),
    ] {
        if !marker.is_available() {
            missing.push(ingredient);
        }
    }
    missing.sort();

    let completeness = if missing.is_empty() {
        AnchorCompleteness::Complete
    } else {
        AnchorCompleteness::Incomplete { missing }
    };

    crate::vc_ir::ObligationAnchor {
        owner,
        kind: kind.clone(),
        local_path: seed.local_path.clone(),
        label,
        semantic_origin: seed.semantic_origin.clone(),
        source_range: source_range(source),
        provenance: anchor_provenance,
        source_shape_hash,
        canonical_goal_hash,
        canonical_context_hash,
        generation_schema_version: schema_version.clone(),
        completeness,
    }
}

fn source_provenance(seed: &ObligationSeed, source: &CoreSourceRef) -> Vec<VcProvenance> {
    seed.provenance
        .iter()
        .chain(source.provenance.iter())
        .cloned()
        .map(|core| VcProvenance {
            phase: VcProvenancePhase::CoreHandoff,
            key: VcText::new("source-provenance"),
            core: Some(core),
        })
        .collect()
}

fn source_shape_hash_marker(
    owner: &AnchorOwner,
    kind: &VcKind,
    seed: &ObligationSeed,
    label: Option<&AnchorLabel>,
    provenance: &[VcProvenance],
) -> crate::vc_ir::HashMarker {
    let mut payload = String::from("source-shape-hash-v1\n");
    writeln!(
        &mut payload,
        "owner: {}",
        stable_anchor_owner_payload(owner)
    )
    .expect("write string");
    writeln!(&mut payload, "kind: {kind:?}").expect("write string");
    writeln!(&mut payload, "local-path: {:?}", seed.local_path).expect("write string");
    writeln!(&mut payload, "label: {label:?}").expect("write string");
    writeln!(&mut payload, "semantic-origin: {:?}", seed.semantic_origin).expect("write string");
    writeln!(&mut payload, "provenance: {provenance:?}").expect("write string");
    hash_marker_for_payload("mizar-vc-source-shape", &payload)
}

fn stable_anchor_owner_payload(owner: &AnchorOwner) -> String {
    match owner {
        AnchorOwner::Theorem(_) => "theorem".to_owned(),
        AnchorOwner::Definition(_) => "definition".to_owned(),
        AnchorOwner::Registration(_) => "registration".to_owned(),
        AnchorOwner::GeneratedSymbol(_) => "generated-symbol".to_owned(),
        AnchorOwner::Algorithm(_) => "algorithm".to_owned(),
        AnchorOwner::ProofBlock(_) => "proof-block".to_owned(),
        AnchorOwner::CheckerOrigin(origin) => format!("checker-origin:{}", origin.as_str()),
    }
}

fn source_range(source: &CoreSourceRef) -> Option<SourceRange> {
    match &source.anchor {
        CoreSourceAnchor::SourceRange(range) => Some(*range),
        CoreSourceAnchor::GeneratedFrom(_) => None,
        _ => None,
    }
}

fn related_sources(row_source: &CoreSourceRef, seed_source: &CoreSourceRef) -> Vec<CoreSourceRef> {
    if row_source == seed_source {
        Vec::new()
    } else {
        vec![seed_source.clone()]
    }
}

fn owner_for_kind(
    kind: &VcKind,
    seed: &ObligationSeed,
    flow_algorithm: Option<CoreAlgorithmId>,
) -> AnchorOwner {
    match kind {
        VcKind::TheoremProofStep | VcKind::TerminalProofGoal => AnchorOwner::Theorem(seed.owner),
        VcKind::DefinitionCorrectness => AnchorOwner::Definition(seed.owner),
        VcKind::RegistrationStyleCorrectness { .. } => AnchorOwner::Registration(seed.owner),
        VcKind::AlgorithmPrecondition
        | VcKind::AlgorithmPostcondition
        | VcKind::CallPrecondition
        | VcKind::AlgorithmAssertion
        | VcKind::LoopInvariant { .. }
        | VcKind::RangeLoop { .. }
        | VcKind::CollectionLoop { .. }
        | VcKind::Termination
        | VcKind::PartialTermination
        | VcKind::GhostErasureSafety => flow_algorithm.map_or_else(
            || AnchorOwner::CheckerOrigin(VcText::new(format!("{:?}", seed.kind))),
            AnchorOwner::Algorithm,
        ),
        VcKind::CheckerInitial => AnchorOwner::CheckerOrigin(VcText::new(format!(
            "{}:{:?}",
            seed.semantic_origin.as_str(),
            seed.kind
        ))),
        VcKind::GeneratedNonEmptiness
        | VcKind::GeneratedSethood
        | VcKind::FraenkelMembershipAxiom => AnchorOwner::GeneratedSymbol(seed.owner),
        _ => AnchorOwner::CheckerOrigin(VcText::new(format!("{:?}", seed.kind))),
    }
}

fn candidate_sort_key(
    schema_version: &GenerationSchemaVersion,
    module: &VcModuleRef,
    handoff: ObligationHandoffId,
    seed: &ObligationSeed,
    kind: &VcKind,
) -> CanonicalSortKey {
    CanonicalSortKey::new(format!(
        "module={};schema={};owner={};seed-key={:?};source={:?};core-provenance={:?};expansion=000000;handoff={:08};kind={:?}",
        module.as_str(),
        schema_version.as_str(),
        seed.owner.index(),
        seed.canonical_key(),
        seed.source,
        seed.provenance,
        handoff.index(),
        kind
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vc_ir::{ExpandedVcRef, ExpansionSchemaVersion};
    use mizar_core::control_flow::{
        AssertionPlacement, AssertionSite, BasicBlockId, CallSiteTable, ContextFactTable,
        ContractSite, ContractSiteKind, ContractSitePlacement, ControlFlowBlockTable,
        ControlFlowContractSet, ControlFlowDiagnosticTable, ControlFlowExit, ControlFlowExitId,
        ControlFlowExitKind, ControlFlowExitTable, ControlFlowGhostTable, ControlFlowId,
        ControlFlowIr, ControlFlowLocalTable, ControlFlowLoop, ControlFlowLoopTable,
        ControlFlowObligationSite, ControlFlowObligationSiteKind, ControlFlowOutput,
        ControlFlowSourceMap, ControlFlowTable, LocalId, LoopId, LoopInvariantPlacement,
        LoopInvariantSite, ObligationHandoffEntry, ObligationHandoffOrigin, ObligationHandoffTable,
        ProgramContextId, ProgramContextTable,
    };
    use mizar_core::core_ir::{
        CoreAlgorithmId, CoreDefinitionId, CoreDiagnosticId, CoreItemId, CoreLabelRef,
        CoreProvenance, CoreProvenanceKey, CoreProvenancePhase, GeneratedFrom, GeneratedOriginKey,
        GeneratedOriginKind, ObligationSeedId,
    };
    use mizar_resolve::resolved_ast::{FullyQualifiedName, LocalSymbolId, ModuleId, SymbolId};
    use mizar_session::snapshot::{ModulePath, PackageId};
    use mizar_session::{BuildSnapshotId, InMemorySessionIdAllocator, SessionIdAllocator};

    #[test]
    fn state_value_projection_preserves_copies_old_values_and_self_assignment() {
        use crate::vc_ir::VcProgramValue;
        use mizar_core::control_flow::{ControlFlowStatementPlacement, build_control_flow_ir};
        use mizar_core::core_ir::*;
        let source_id = sample_source_id();
        let module = ModuleId::new(PackageId::new("state"), ModulePath::new("values"));
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new("f"),
            FullyQualifiedName::new("state::values::f"),
        );
        let source = CoreSourceRef::direct(SourceRange {
            source_id,
            start: 0,
            end: 100,
        });
        let binder = |var, role: &str| CoreBinder {
            var: CoreVarId::new(var),
            role: role.into(),
            ty_guard: None,
            source_name: None,
            source: source.clone(),
        };
        let mut items = CoreItemTable::new();
        let item = items.insert(CoreItem::new(
            symbol.clone(),
            CoreItemKind::Algorithm,
            "public",
            source.clone(),
        ));
        let mut terms = CoreTermTable::new();
        let [a, b, x, saved, y] = std::array::from_fn(|var| {
            terms.insert(CoreTerm::new(
                CoreTermKind::Var(CoreVarId::new(var)),
                source.clone(),
            ))
        });
        let mut formulas = CoreFormulaTable::new();
        let first = formulas.insert(CoreFormula::new(
            CoreFormulaKind::Equals { left: x, right: b },
            source.clone(),
        ));
        let second = formulas.insert(CoreFormula::new(
            CoreFormulaKind::Equals {
                left: saved,
                right: a,
            },
            source.clone(),
        ));
        let mut statements = CoreAlgorithmStmtTable::new();
        for kind in [
            CoreAlgorithmStmtKind::Let {
                binder: binder(2, "local:var"),
                value: Some(a),
                ghost: false,
            },
            CoreAlgorithmStmtKind::Let {
                binder: binder(3, "local:const"),
                value: Some(x),
                ghost: false,
            },
            CoreAlgorithmStmtKind::Assert { formula: first },
            CoreAlgorithmStmtKind::AssignLocal {
                target: CoreVarId::new(2),
                value: b,
            },
            CoreAlgorithmStmtKind::Snapshot {
                name: "after_b".into(),
                captures: (0..4).map(CoreVarId::new).collect(),
            },
            CoreAlgorithmStmtKind::Let {
                binder: binder(4, "local:var"),
                value: Some(x),
                ghost: false,
            },
            CoreAlgorithmStmtKind::AssignLocal {
                target: CoreVarId::new(2),
                value: x,
            },
            CoreAlgorithmStmtKind::Assert { formula: second },
            CoreAlgorithmStmtKind::AssignLocal {
                target: CoreVarId::new(2),
                value: saved,
            },
            CoreAlgorithmStmtKind::Snapshot {
                name: "after_saved".into(),
                captures: (0..5).map(CoreVarId::new).collect(),
            },
            CoreAlgorithmStmtKind::Return(Some(x)),
        ] {
            statements.insert(CoreAlgorithmStmt {
                owner: CoreAlgorithmId::new(0),
                kind,
                source: source.clone(),
                diagnostics: vec![],
            });
        }
        let mut algorithms = CoreAlgorithmTable::new();
        algorithms.insert(CoreAlgorithm {
            item,
            symbol,
            params: vec![binder(0, "parameter"), binder(1, "parameter")],
            result: Some(binder(5, "result")),
            contracts: CoreContractSet::default(),
            statements: statements.iter().map(|(id, _)| id).collect(),
            ghost_effects: vec![],
            source: source.clone(),
            diagnostics: vec![],
        });
        let mut source_map = CoreSourceMap::new();
        source_map.item_sources.insert(item, source.clone());
        for (id, term) in terms.iter() {
            source_map.term_sources.insert(id, term.source.clone());
        }
        for (id, formula) in formulas.iter() {
            source_map
                .formula_sources
                .insert(id, formula.source.clone());
        }
        for (id, statement) in statements.iter() {
            source_map
                .algorithm_sources
                .insert(id, statement.source.clone());
        }
        let core = CoreIr::try_new(CoreIrParts {
            source_id,
            module_id: module,
            items,
            terms,
            formulas,
            algorithms,
            algorithm_statements: statements,
            source_map,
            definitions: Default::default(),
            proofs: Default::default(),
            proof_nodes: Default::default(),
            generated: Default::default(),
            obligation_seeds: Default::default(),
            diagnostics: Default::default(),
        })
        .unwrap();
        let v = |var: usize, definition: Option<usize>| VcProgramValue {
            var: CoreVarId::new(var),
            definition: definition.map(CoreAlgorithmStmtId::new),
        };
        let mut values = BTreeMap::from([
            (CoreVarId::new(0), v(0, None)),
            (CoreVarId::new(1), v(1, None)),
        ]);
        let output = build_control_flow_ir(&core);
        let (_, flow) = output.flows.iter().next().unwrap();
        assert!(flow.diagnostics.is_empty());
        let mut snapshots = Vec::new();
        let mut writes = Vec::new();
        let mut assertions = Vec::new();
        for (id, statement) in core.algorithm_statements().iter() {
            match &statement.kind {
                CoreAlgorithmStmtKind::Let {
                    binder,
                    value: Some(value),
                    ..
                } => {
                    let rhs =
                        source_program_value(&core, *value, &mut values, Some((binder.var, id)))
                            .unwrap();
                    writes.push((values[&binder.var], rhs));
                }
                CoreAlgorithmStmtKind::AssignLocal { target, value } => {
                    let rhs = source_program_value(&core, *value, &mut values, Some((*target, id)))
                        .unwrap();
                    writes.push((values[target], rhs));
                }
                CoreAlgorithmStmtKind::Assert { formula } => {
                    let CoreFormulaKind::Equals { left, right } =
                        core.formulas().get(*formula).unwrap().kind
                    else {
                        panic!()
                    };
                    assertions.push((
                        source_program_value(&core, left, &mut values, None).unwrap(),
                        source_program_value(&core, right, &mut values, None).unwrap(),
                    ));
                }
                CoreAlgorithmStmtKind::Snapshot { captures, .. } => {
                    let ControlFlowStatementPlacement::Snapshot {
                        context,
                        captures: locals,
                        ..
                    } = &flow.source_map.statement_placements[&id]
                    else {
                        panic!()
                    };
                    let context = flow.contexts.get(*context).unwrap();
                    assert_eq!(
                        *captures,
                        locals
                            .iter()
                            .map(|local| flow.locals.get(*local).unwrap().binder.var)
                            .collect::<Vec<_>>()
                    );
                    assert!(
                        locals
                            .iter()
                            .all(|local| context.definitely_initialized.contains(local))
                    );
                    assert_eq!(
                        context
                            .assignment_effects
                            .iter()
                            .map(|effect| flow.assignment_effects.get(*effect).unwrap().statement)
                            .collect::<Vec<_>>(),
                        writes
                            .iter()
                            .map(|(lhs, _)| lhs.definition.unwrap())
                            .collect::<Vec<_>>()
                    );
                    snapshots.push(captures.iter().map(|var| values[var]).collect::<Vec<_>>());
                }
                CoreAlgorithmStmtKind::Return(Some(term)) => assert_eq!(
                    source_program_value(&core, *term, &mut values, None).unwrap(),
                    v(2, Some(8))
                ),
                _ => panic!(),
            }
        }
        assert_eq!(
            writes,
            vec![
                (v(2, Some(0)), v(0, None)),
                (v(3, Some(1)), v(2, Some(0))),
                (v(2, Some(3)), v(1, None)),
                (v(4, Some(5)), v(2, Some(3))),
                (v(2, Some(6)), v(2, Some(3))),
                (v(2, Some(8)), v(3, Some(1)))
            ]
        );
        assert_eq!(
            assertions,
            vec![(v(2, Some(0)), v(1, None)), (v(3, Some(1)), v(0, None))]
        );
        assert_eq!(
            source_program_value(&core, y, &mut values, None).unwrap(),
            v(4, Some(5))
        );
        assert_eq!(
            snapshots,
            vec![
                vec![v(0, None), v(1, None), v(2, Some(3)), v(3, Some(1))],
                vec![
                    v(0, None),
                    v(1, None),
                    v(2, Some(8)),
                    v(3, Some(1)),
                    v(4, Some(5))
                ],
            ]
        );
        assert_ne!(snapshots[0][2], values[&CoreVarId::new(2)]);
        values.remove(&CoreVarId::new(4));
        assert!(source_program_value(&core, y, &mut values, None).is_err());
    }

    #[test]
    fn generates_task_six_candidates_for_core_seed_families() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(10)),
                    "proof/step/0",
                    "theorem:sample:proof-step:0",
                )
                .with_context(vec![CoreFormulaId::new(1)])
                .with_label("A1")
                .with_core_ref(CoreNodeRef::Definition(CoreDefinitionId::new(0)))
                .with_provenance_key("vc-unfold:transparent")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(15)),
                    "proof/terminal/0",
                    "theorem:sample:terminal",
                )
                .with_provenance_key("vc-proof-goal:terminal")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(5),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::DefinitionCorrectness,
                    Some(CoreFormulaId::new(11)),
                    "definition/existence",
                    "definition:sample:existence",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GeneratedNonEmptiness,
                    Some(CoreFormulaId::new(12)),
                    "generated/non-emptiness/0",
                    "generated:non-emptiness:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(2),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GeneratedSethood,
                    Some(CoreFormulaId::new(13)),
                    "generated/sethood/0",
                    "generated:sethood:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(3),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::FraenkelMembershipAxiom,
                    Some(CoreFormulaId::new(14)),
                    "generated/fraenkel/0",
                    "generated:fraenkel-membership:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(4),
                },
            ),
        ]);

        let set = generate(&handoff);

        assert_eq!(set.no_candidates(), []);
        assert_eq!(
            set.candidates()
                .iter()
                .map(|candidate| &candidate.kind)
                .collect::<Vec<_>>(),
            vec![
                &VcKind::TheoremProofStep,
                &VcKind::TerminalProofGoal,
                &VcKind::DefinitionCorrectness,
                &VcKind::GeneratedNonEmptiness,
                &VcKind::GeneratedSethood,
                &VcKind::FraenkelMembershipAxiom,
            ]
        );
        for candidate in set.candidates() {
            assert!(candidate.anchor.source_shape_hash.is_available());
            assert!(!candidate.anchor.canonical_goal_hash.is_available());
            let AnchorCompleteness::Incomplete { missing } = &candidate.anchor.completeness else {
                panic!("core goal candidates must fail closed until canonical payloads exist");
            };
            assert!(missing.contains(&AnchorIngredient::CanonicalGoalHash));
        }
        let theorem = &set.candidates()[0];
        assert_eq!(
            theorem.local_context.entries()[0].formula,
            Some(VcFormulaRef::Core(CoreFormulaId::new(1)))
        );
        assert!(
            theorem
                .premises
                .contains(&PremiseRef::LocalContext(ContextEntryId::new(0)))
        );
        assert!(theorem.premises.contains(&PremiseRef::LocalLabel {
            label: CoreLabelRef::new("A1")
        }));
        let hint = theorem.proof_hint.as_ref().expect("symbolic proof hint");
        assert_eq!(hint.unfold_requests[0].definition, CoreDefinitionId::new(0));
        assert!(matches!(
            theorem.anchor.completeness,
            AnchorCompleteness::Incomplete { .. }
        ));
        assert!(theorem.anchor.source_shape_hash.is_available());
        assert!(!theorem.anchor.canonical_goal_hash.is_available());
        assert!(!theorem.anchor.canonical_context_hash.is_available());
    }

    #[test]
    fn classifies_explicit_registration_style_payloads() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::DefinitionCorrectness,
                    Some(CoreFormulaId::new(0)),
                    "definition/existence/0",
                    "definition:existence",
                )
                .with_provenance_key("vc-registration-style:registration")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::DefinitionCorrectness,
                    Some(CoreFormulaId::new(1)),
                    "definition/compatibility",
                    "definition:compatibility",
                )
                .with_provenance_key("vc-registration-style:redefinition")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::CheckerInitial,
                    Some(CoreFormulaId::new(2)),
                    "checker/reducibility",
                    "checker:reducibility",
                )
                .with_provenance_key("vc-registration-style:reduction")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(2),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::CheckerInitial,
                    Some(CoreFormulaId::new(3)),
                    "checker/carried/0",
                    "checker:carried",
                )
                .with_provenance_key("vc-registration-style:explicit-core-seed")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(3),
                },
            ),
        ]);

        let set = generate(&handoff);

        assert!(matches!(
            set.candidates()[0].kind,
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::Registration
            }
        ));
        assert!(matches!(
            set.candidates()[1].kind,
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::Redefinition
            }
        ));
        assert!(matches!(
            set.candidates()[2].kind,
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::Reduction
            }
        ));
        assert!(matches!(
            set.candidates()[3].kind,
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::ExplicitCoreSeed
            }
        ));
    }

    #[test]
    fn definition_correctness_families_stay_ordinary_definition_candidates() {
        let source = sample_source_id();
        let families = [
            "existence",
            "uniqueness",
            "coherence",
            "compatibility",
            "consistency",
            "reducibility",
            "sethood",
            "non-emptiness",
        ];
        let handoff = seed_handoff(
            families
                .iter()
                .enumerate()
                .map(|(index, family)| {
                    (
                        obligation_seed(
                            source,
                            ObligationSeedKind::DefinitionCorrectness,
                            Some(CoreFormulaId::new(index)),
                            &format!("definition/{family}"),
                            &format!("definition:sample:{family}"),
                        )
                        .into(),
                        ObligationHandoffOrigin::ExistingCore {
                            seed: ObligationSeedId::new(index),
                        },
                    )
                })
                .collect(),
        );

        let set = generate(&handoff);

        assert_eq!(set.candidates().len(), families.len());
        assert_eq!(set.no_candidates(), []);
        for (candidate, family) in set.candidates().iter().zip(families) {
            assert_eq!(candidate.kind, VcKind::DefinitionCorrectness);
            assert!(candidate.local_path.as_str().contains(family));
            assert!(candidate.semantic_origin.as_str().contains(family));
        }
    }

    #[test]
    fn preserves_explicit_theorem_status_dependency_markers_without_invention() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(0)),
                    "proof/non-clean/open/conditional",
                    "theorem:non-clean:open:conditional",
                )
                .with_provenance_key("vc-theorem-status:non-clean")
                .with_provenance_key("vc-theorem-status:open")
                .with_provenance_key("vc-theorem-status:conditional")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(1)),
                    "proof/clean/assumed",
                    "theorem:clean:assumed",
                )
                .with_provenance_key("vc-theorem-status:clean")
                .with_provenance_key("vc-theorem-status:assumed")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(2)),
                    "proof/plain",
                    "theorem:plain",
                )
                .with_label("registration")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(2),
                },
            ),
        ]);

        let set = generate(&handoff);
        let policy_inputs = set.candidates()[0].local_context.policy_inputs();

        assert_eq!(policy_inputs.len(), 3);
        assert_eq!(policy_inputs[0].value, PolicyValue::new("non-clean"));
        assert_eq!(policy_inputs[1].value, PolicyValue::new("open"));
        assert_eq!(policy_inputs[2].value, PolicyValue::new("conditional"));
        let clean_assumed = set.candidates()[1].local_context.policy_inputs();
        assert_eq!(clean_assumed.len(), 2);
        assert_eq!(clean_assumed[0].value, PolicyValue::new("clean"));
        assert_eq!(clean_assumed[1].value, PolicyValue::new("assumed"));
        assert_eq!(set.candidates()[2].local_context.policy_inputs(), []);
        assert_eq!(set.candidates()[2].kind, VcKind::TheoremProofStep);
    }

    #[test]
    fn local_context_entries_are_canonicalized_before_dense_ids() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(9)),
                "proof/context-order",
                "theorem:context-order",
            )
            .with_context(vec![CoreFormulaId::new(2), CoreFormulaId::new(1)])
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);

        let set = generate(&handoff);
        let entries = set.candidates()[0].local_context.entries();

        assert_eq!(entries[0].id, ContextEntryId::new(0));
        assert_eq!(
            entries[0].formula,
            Some(VcFormulaRef::Core(CoreFormulaId::new(1)))
        );
        assert_eq!(entries[1].id, ContextEntryId::new(1));
        assert_eq!(
            entries[1].formula,
            Some(VcFormulaRef::Core(CoreFormulaId::new(2)))
        );
        assert!(entries[0].sort_key < entries[1].sort_key);
    }

    #[test]
    fn registration_style_labels_alone_do_not_classify_and_deferred_payloads_stay_visible() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::DefinitionCorrectness,
                    Some(CoreFormulaId::new(0)),
                    "definition/coherence",
                    "definition:coherence",
                )
                .with_label("registration")
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::DefinitionCorrectness,
                    Some(CoreFormulaId::new(1)),
                    "registration/coherence/deferred",
                    "registration:coherence:deferred",
                )
                .with_provenance_key("vc-registration-style:registration")
                .with_status(ObligationSeedStatus::Deferred)
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
        ]);

        let set = generate(&handoff);

        assert_eq!(set.candidates().len(), 1);
        assert_eq!(set.candidates()[0].kind, VcKind::DefinitionCorrectness);
        assert_eq!(set.no_candidates().len(), 1);
        assert!(matches!(
            &set.no_candidates()[0].reason,
            SeedNoVcReason::DeferredExternal(reason)
                if reason.as_str().contains("seed status is deferred")
        ));
    }

    #[test]
    fn records_deferred_rows_for_no_vc_and_later_task_seed_kinds() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmContract,
                    Some(CoreFormulaId::new(0)),
                    "algorithm/requires/0",
                    "algorithm:requires:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmTermination,
                    Some(CoreFormulaId::new(1)),
                    "algorithm/termination/0",
                    "algorithm:termination:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GhostErasure,
                    Some(CoreFormulaId::new(2)),
                    "algorithm/ghost-erasure/0",
                    "algorithm:ghost-erasure:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(2),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    None,
                    "proof/missing-goal",
                    "theorem:missing-goal",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(3),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(3)),
                    "proof/deferred",
                    "theorem:deferred",
                )
                .with_status(ObligationSeedStatus::Deferred)
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(4),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(4)),
                    "proof/skipped",
                    "theorem:skipped",
                )
                .with_status(ObligationSeedStatus::Skipped)
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(5),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(5)),
                    "proof/error",
                    "theorem:error",
                )
                .with_status(ObligationSeedStatus::Error)
                .with_diagnostics(vec![CoreDiagnosticId::new(7)])
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(6),
                },
            ),
        ]);

        let set = generate(&handoff);

        assert_eq!(set.candidates(), []);
        assert_eq!(set.no_candidates().len(), 7);
        assert!(matches!(
            &set.no_candidates()[0].reason,
            SeedNoVcReason::DeferredExternal(reason)
                if reason.as_str().contains("AlgorithmContract")
        ));
        assert!(matches!(
            &set.no_candidates()[1].reason,
            SeedNoVcReason::DeferredExternal(reason)
                if reason.as_str().contains("AlgorithmTermination")
        ));
        assert!(matches!(
            &set.no_candidates()[2].reason,
            SeedNoVcReason::DeferredExternal(reason)
                if reason.as_str().contains("GhostErasure")
        ));
        assert!(matches!(
            &set.no_candidates()[3].reason,
            SeedNoVcReason::MissingGoal(reason)
                if reason.as_str().contains("active seed has no goal")
        ));
        assert!(matches!(
            &set.no_candidates()[4].reason,
            SeedNoVcReason::DeferredExternal(reason)
                if reason.as_str().contains("seed status is deferred")
        ));
        assert!(matches!(
            set.no_candidates()[5].reason,
            SeedNoVcReason::SkippedInvalidInput
        ));
        assert!(matches!(
            set.no_candidates()[6].reason,
            SeedNoVcReason::Error(diagnostic)
                if diagnostic == CoreDiagnosticId::new(7)
        ));
    }

    #[test]
    fn generates_goal_bearing_algorithm_candidates_from_flow_sites() {
        let source = sample_source_id();
        let flow_output = sample_flow_output(source);
        let handoff = seed_handoff_with_sites(vec![
            flow_seed(
                source,
                CoreFormulaId::new(10),
                "program/0/contract/requires/0",
                "flow:0:requires:0",
                flow_site(ControlFlowObligationSiteKind::Requires, 0),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(11),
                "program/0/contract/ensures/0",
                "flow:0:ensures:0",
                flow_site(ControlFlowObligationSiteKind::Ensures, 0),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(12),
                "program/0/assertion/algorithm/0",
                "flow:0:assertion:algorithm:0",
                flow_site(ControlFlowObligationSiteKind::AlgorithmAssertion, 0).with_block(0),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(13),
                "program/0/assertion/0",
                "flow:0:assertion:0",
                flow_site(ControlFlowObligationSiteKind::StatementAssertion, 1)
                    .with_statement(0)
                    .with_block(1),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(14),
                "program/0/invariant/algorithm/0",
                "flow:0:invariant:algorithm:0",
                flow_site(ControlFlowObligationSiteKind::AlgorithmInvariant, 0).with_block(0),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(15),
                "program/0/invariant/header/0",
                "flow:0:invariant:header:0",
                flow_site(ControlFlowObligationSiteKind::LoopInvariant, 1)
                    .with_loop(0)
                    .with_block(0),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(16),
                "program/0/invariant/backedge/0",
                "flow:0:invariant:backedge:0",
                flow_site(ControlFlowObligationSiteKind::LoopInvariant, 2)
                    .with_loop(0)
                    .with_block(1),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(17),
                "program/0/invariant/break/0",
                "flow:0:invariant:break:0",
                flow_site(ControlFlowObligationSiteKind::LoopInvariant, 3)
                    .with_loop(0)
                    .with_exit(0),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(18),
                "program/0/invariant/continue/0",
                "flow:0:invariant:continue:0",
                flow_site(ControlFlowObligationSiteKind::LoopInvariant, 4)
                    .with_loop(0)
                    .with_exit(1),
            ),
        ]);

        let set = generate_with_flows(&handoff, &flow_output);

        assert_eq!(set.no_candidates(), []);
        assert_eq!(
            set.candidates()
                .iter()
                .map(|candidate| &candidate.kind)
                .collect::<Vec<_>>(),
            vec![
                &VcKind::AlgorithmPrecondition,
                &VcKind::AlgorithmPostcondition,
                &VcKind::AlgorithmAssertion,
                &VcKind::AlgorithmAssertion,
                &VcKind::LoopInvariant {
                    phase: LoopInvariantPhase::Entry
                },
                &VcKind::LoopInvariant {
                    phase: LoopInvariantPhase::Entry
                },
                &VcKind::LoopInvariant {
                    phase: LoopInvariantPhase::Preservation
                },
                &VcKind::LoopInvariant {
                    phase: LoopInvariantPhase::Break
                },
                &VcKind::LoopInvariant {
                    phase: LoopInvariantPhase::Continue
                },
            ]
        );
        for candidate in set.candidates() {
            assert!(candidate.anchor.source_shape_hash.is_available());
            assert!(!candidate.anchor.canonical_goal_hash.is_available());
            let AnchorCompleteness::Incomplete { missing } = &candidate.anchor.completeness else {
                panic!(
                    "algorithm core-goal candidates must fail closed until canonical payloads exist"
                );
            };
            assert!(missing.contains(&AnchorIngredient::CanonicalGoalHash));
        }
        let requires = &set.candidates()[0];
        assert_eq!(requires.seed_status, ObligationSeedStatus::Deferred);
        assert_eq!(requires.status, VcStatus::Open);
        assert_eq!(
            requires.owner,
            AnchorOwner::Algorithm(CoreAlgorithmId::new(0))
        );
        assert!(requires.local_context.policy_inputs().iter().any(|input| {
            input.key == PolicyKey::new("algorithm-site-site-kind")
                && input.value == PolicyValue::new("Requires")
        }));
        let statement_assertion = &set.candidates()[3];
        assert_policy_input(statement_assertion, "algorithm-site-statement", "0");
        assert_policy_input(statement_assertion, "algorithm-site-block", "1");
        let break_invariant = &set.candidates()[7];
        assert_policy_input(break_invariant, "algorithm-site-loop", "0");
        assert_policy_input(break_invariant, "algorithm-site-exit", "0");
        assert!(requires.provenance.iter().any(|provenance| {
            provenance.phase == VcProvenancePhase::Generator
                && provenance.key == VcText::new("task-7-algorithm-candidate")
        }));
    }

    #[test]
    fn algorithm_candidate_debug_rendering_is_deterministic() {
        let source = sample_source_id();
        let flow_output = sample_flow_output(source);
        let handoff = seed_handoff_with_sites(vec![
            flow_seed(
                source,
                CoreFormulaId::new(13),
                "program/0/assertion/0",
                "flow:0:assertion:0",
                flow_site(ControlFlowObligationSiteKind::StatementAssertion, 1)
                    .with_statement(0)
                    .with_block(1),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(10),
                "program/0/contract/requires/0",
                "flow:0:requires:0",
                flow_site(ControlFlowObligationSiteKind::Requires, 0),
            ),
        ]);

        let first = generate_with_flows(&handoff, &flow_output);
        let second = generate_with_flows(&handoff, &flow_output);

        assert_eq!(first.debug_text(), second.debug_text());
        assert!(first.debug_text().contains("AlgorithmPrecondition"));
        assert!(first.debug_text().contains("AlgorithmAssertion"));
    }

    #[test]
    fn records_no_candidates_for_unavailable_algorithm_payloads() {
        let source = sample_source_id();
        let flow_output = sample_flow_output(source);
        let handoff = seed_handoff_with_sites(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmContract,
                    Some(CoreFormulaId::new(0)),
                    "program/0/contract/requires/missing-site",
                    "flow:0:requires:missing-site",
                )
                .with_status(ObligationSeedStatus::Deferred)
                .into(),
                flow_origin(),
                None,
            ),
            flow_seed(
                source,
                CoreFormulaId::new(10),
                "program/0/contract/requires/valid",
                "flow:0:requires:valid",
                flow_site(ControlFlowObligationSiteKind::Requires, 0),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(1),
                "program/0/contract/requires/missing-flow",
                "flow:0:requires:missing-flow",
                flow_site(ControlFlowObligationSiteKind::Requires, 1),
            ),
            flow_seed(
                source,
                CoreFormulaId::new(3),
                "program/0/invariant/incomplete-phase",
                "flow:0:invariant:incomplete-phase",
                flow_site(ControlFlowObligationSiteKind::LoopInvariant, 5).with_loop(0),
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmTermination,
                    None,
                    "program/0/termination/measure/0",
                    "flow:0:termination:measure:0",
                )
                .with_status(ObligationSeedStatus::Deferred)
                .into(),
                flow_origin(),
                Some(flow_site(ControlFlowObligationSiteKind::TerminationMeasure, 2).into()),
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GhostErasure,
                    None,
                    "program/0/ghost/pick/0",
                    "flow:0:ghost:pick:0",
                )
                .with_status(ObligationSeedStatus::Deferred)
                .into(),
                flow_origin(),
                Some(
                    flow_site(ControlFlowObligationSiteKind::GhostPick, 3)
                        .with_local(0)
                        .into(),
                ),
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmTermination,
                    None,
                    "program/0/termination/partial/0",
                    "flow:0:termination:partial:0",
                )
                .with_status(ObligationSeedStatus::Deferred)
                .into(),
                flow_origin(),
                Some(flow_site(ControlFlowObligationSiteKind::PartialTermination, 4).into()),
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GhostErasure,
                    None,
                    "program/0/ghost/assignment/0",
                    "flow:0:ghost:assignment:0",
                )
                .with_status(ObligationSeedStatus::Deferred)
                .into(),
                flow_origin(),
                Some(
                    flow_site(ControlFlowObligationSiteKind::GhostAssignment, 5)
                        .with_assignment_effect(0)
                        .into(),
                ),
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmContract,
                    Some(CoreFormulaId::new(2)),
                    "program/0/contract/requires/existing-core",
                    "flow:0:requires:existing-core",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(9),
                },
                Some(flow_site(ControlFlowObligationSiteKind::Requires, 4).into()),
            ),
        ]);
        let empty_flow_output = ControlFlowOutput {
            flows: ControlFlowTable::new(),
            flow_map: std::collections::BTreeMap::new(),
        };

        let missing_flow_set = generate_with_flows(&handoff, &empty_flow_output);
        assert_eq!(missing_flow_set.candidates(), []);
        assert!(missing_flow_set.no_candidates().iter().any(|no_candidate| {
            matches!(
                &no_candidate.reason,
                SeedNoVcReason::DeferredExternal(reason)
                    if reason.as_str().contains("matching ControlFlowOutput")
            )
        }));

        let set = generate_with_flows(&handoff, &flow_output);
        assert_eq!(set.candidates().len(), 1);
        assert_eq!(set.candidates()[0].kind, VcKind::AlgorithmPrecondition);
        assert!(set.no_candidates().iter().any(|no_candidate| {
            matches!(
                &no_candidate.reason,
                SeedNoVcReason::DeferredExternal(reason)
                    if reason.as_str().contains("ControlFlowObligationSite")
            )
        }));
        assert!(set.no_candidates().iter().any(|no_candidate| {
            matches!(
                &no_candidate.reason,
                SeedNoVcReason::DeferredExternal(reason)
                    if reason.as_str().contains("explicit goal formula")
                        && reason.as_str().contains("TerminationMeasure")
            )
        }));
        assert!(set.no_candidates().iter().any(|no_candidate| {
            matches!(
                &no_candidate.reason,
                SeedNoVcReason::DeferredExternal(reason)
                    if reason.as_str().contains("explicit goal formula")
                        && reason.as_str().contains("GhostPick")
            )
        }));
        assert!(set.no_candidates().iter().any(|no_candidate| {
            matches!(
                &no_candidate.reason,
                SeedNoVcReason::DeferredExternal(reason)
                    if reason.as_str().contains("explicit goal formula")
                        && reason.as_str().contains("PartialTermination")
            )
        }));
        assert!(set.no_candidates().iter().any(|no_candidate| {
            matches!(
                &no_candidate.reason,
                SeedNoVcReason::DeferredExternal(reason)
                    if reason.as_str().contains("explicit goal formula")
                        && reason.as_str().contains("GhostAssignment")
            )
        }));
        assert!(set.no_candidates().iter().any(|no_candidate| {
            matches!(
                &no_candidate.reason,
                SeedNoVcReason::DeferredExternal(reason)
                    if reason.as_str().contains("LoopInvariant")
            )
        }));
        assert!(set.no_candidates().iter().any(|no_candidate| {
            matches!(
                &no_candidate.reason,
                SeedNoVcReason::DeferredExternal(reason)
                    if reason.as_str().contains("FlowDerived origin")
            )
        }));
    }

    #[test]
    fn seed_intake_marks_goal_bearing_deferred_flow_rows_eligible() {
        let source = sample_source_id();
        let handoff = seed_handoff_with_sites(vec![
            flow_seed(
                source,
                CoreFormulaId::new(0),
                "program/0/contract/requires/0",
                "flow:0:requires:0",
                flow_site(ControlFlowObligationSiteKind::Requires, 0),
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmTermination,
                    None,
                    "program/0/termination/measure/0",
                    "flow:0:termination:measure:0",
                )
                .with_status(ObligationSeedStatus::Deferred)
                .into(),
                flow_origin(),
                Some(flow_site(ControlFlowObligationSiteKind::TerminationMeasure, 1).into()),
            ),
        ]);

        let intake = SeedIntakeTable::try_from_handoff(&handoff).expect("intake");

        assert!(matches!(
            intake.rows()[0].mapping,
            SeedIntakeMapping::EligibleOneVc { goal }
                if goal == CoreFormulaId::new(0)
        ));
        assert!(matches!(
            intake.rows()[1].mapping,
            SeedIntakeMapping::NoConcreteVc { .. }
        ));
    }

    #[test]
    fn rejects_flow_output_algorithm_mismatch() {
        let source = sample_source_id();
        let mismatched_flow_output =
            sample_flow_output_for_algorithm(source, CoreAlgorithmId::new(1));
        let handoff = seed_handoff_with_sites(vec![flow_seed(
            source,
            CoreFormulaId::new(10),
            "program/0/contract/requires/0",
            "flow:0:requires:0",
            flow_site(ControlFlowObligationSiteKind::Requires, 0),
        )]);

        let set = generate_with_flows(&handoff, &mismatched_flow_output);

        assert_eq!(set.candidates(), []);
        assert!(matches!(
            &set.no_candidates()[0].reason,
            SeedNoVcReason::DeferredExternal(reason)
                if reason.as_str().contains("matching ControlFlowOutput")
        ));
    }

    #[test]
    fn unavailable_algorithm_families_remain_documented_deferred() {
        let generator_doc = include_str!("../../../doc/design/mizar-vc/en/generator.md");
        let todo_doc = include_str!("../../../doc/design/mizar-vc/en/todo.md");

        for family in [
            "call-precondition",
            "branch",
            "match",
            "range-loop",
            "collection-loop",
            "Pick non-emptiness",
            "ghost-erasure",
        ] {
            assert!(
                generator_doc.contains(family),
                "generator.md must classify unavailable {family} payloads"
            );
            assert!(
                todo_doc.contains(family),
                "todo.md must keep unavailable {family} payloads deferred"
            );
        }
        assert!(generator_doc.contains("visible no-candidate/deferred"));
        assert!(todo_doc.contains("deferred/no-candidate"));
    }

    #[test]
    fn rejects_stale_intake_handoff_mismatch() {
        let source = sample_source_id();
        let original = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(0)),
                "proof/original",
                "theorem:original",
            )
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);
        let changed = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(0)),
                "proof/original",
                "theorem:original",
            )
            .with_status(ObligationSeedStatus::Deferred)
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);
        let intake = SeedIntakeTable::try_from_handoff(&original).expect("intake");

        assert!(matches!(
            CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
                schema_version: &GenerationSchemaVersion::new("generator-task-6-test"),
                module: &VcModuleRef::new("sample"),
                intake: &intake,
                handoff: &changed,
                flow_output: None,
            }),
            Err(GeneratorError::IntakeHandoffMismatch { handoff })
                if handoff == ObligationHandoffId::new(0)
        ));
    }

    #[test]
    fn rejects_partial_intake_when_handoff_adds_obligations() {
        let source = sample_source_id();
        let original = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(0)),
                "proof/original",
                "theorem:original",
            )
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);
        let expanded = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(0)),
                    "proof/original",
                    "theorem:original",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GeneratedNonEmptiness,
                    Some(CoreFormulaId::new(1)),
                    "generated/non-emptiness/0",
                    "generated:non-emptiness:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
        ]);
        let stale_intake = SeedIntakeTable::try_from_handoff(&original).expect("stale intake");

        assert!(matches!(
            CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
                schema_version: &GenerationSchemaVersion::new("generator-task-6-test"),
                module: &VcModuleRef::new("sample"),
                intake: &stale_intake,
                handoff: &expanded,
                flow_output: None,
            }),
            Err(GeneratorError::IntakeHandoffMismatch { handoff })
                if handoff == ObligationHandoffId::new(1)
        ));
    }

    #[test]
    fn debug_rendering_is_deterministic() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GeneratedSethood,
                    Some(CoreFormulaId::new(0)),
                    "generated/sethood/0",
                    "generated:sethood:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::AlgorithmContract,
                    Some(CoreFormulaId::new(1)),
                    "algorithm/requires/0",
                    "algorithm:requires:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::DefinitionCorrectness,
                    Some(CoreFormulaId::new(2)),
                    "definition/existence",
                    "definition:existence",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(2),
                },
            ),
        ]);

        let first = generate(&handoff);
        let second = generate(&handoff);

        assert_eq!(first.debug_text(), second.debug_text());
        assert!(
            first
                .debug_text()
                .contains("core-generation-candidates-debug-v1")
        );
        assert!(first.debug_text().contains("GeneratedSethood"));
        assert!(
            first
                .debug_text()
                .contains("no-candidate ObligationHandoffId(1)")
        );
    }

    #[test]
    fn normalizes_candidates_to_dense_vc_set_and_seed_accounting() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(10)),
                    "proof/step/0",
                    "theorem:sample:proof-step:0",
                )
                .with_context(vec![CoreFormulaId::new(1)])
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GeneratedSethood,
                    Some(CoreFormulaId::new(11)),
                    "generated/sethood/0",
                    "generated:sethood:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::DefinitionCorrectness,
                    None,
                    "definition/missing-goal",
                    "definition:missing-goal",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(2),
                },
            ),
        ]);
        let candidates = generate(&handoff);

        let vc_set = normalize(&candidates);

        assert_eq!(vc_set.generated_formulas(), []);
        assert_eq!(
            vc_set.vcs().iter().map(|vc| vc.id).collect::<Vec<_>>(),
            vec![VcId::new(0), VcId::new(1)]
        );
        assert_eq!(
            vc_set.vcs().iter().map(|vc| &vc.kind).collect::<Vec<_>>(),
            vec![&VcKind::TheoremProofStep, &VcKind::GeneratedSethood]
        );
        assert!(
            vc_set.vcs()[0]
                .provenance
                .iter()
                .any(|provenance| provenance.phase == VcProvenancePhase::Normalization)
        );
        assert_eq!(
            vc_set
                .seed_accounting()
                .iter()
                .map(|row| row.handoff)
                .collect::<Vec<_>>(),
            vec![
                ObligationHandoffId::new(0),
                ObligationHandoffId::new(1),
                ObligationHandoffId::new(2),
            ]
        );
        assert!(matches!(
            vc_set.seed_accounting_for(ObligationHandoffId::new(0)),
            Some(SeedAccounting {
                mapping: SeedVcMapping::One { vc },
                ..
            }) if *vc == VcId::new(0)
        ));
        assert!(matches!(
            vc_set.seed_accounting_for(ObligationHandoffId::new(1)),
            Some(SeedAccounting {
                mapping: SeedVcMapping::One { vc },
                ..
            }) if *vc == VcId::new(1)
        ));
        assert!(matches!(
            vc_set.seed_accounting_for(ObligationHandoffId::new(2)),
            Some(SeedAccounting {
                mapping: SeedVcMapping::NoConcreteVc {
                    reason: SeedNoVcReason::MissingGoal(reason),
                },
                ..
            }) if reason.as_str().contains("active seed has no goal")
        ));
    }

    #[test]
    fn normalization_uses_documented_kind_rank_before_candidate_sort_key() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GeneratedSethood,
                    Some(CoreFormulaId::new(0)),
                    "generated/sethood/0",
                    "generated:sethood:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(1)),
                    "proof/step/0",
                    "theorem:sample:proof-step:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
        ]);
        let mut candidates = generate(&handoff);
        candidates
            .candidates
            .iter_mut()
            .find(|candidate| candidate.kind == VcKind::TheoremProofStep)
            .expect("theorem candidate")
            .sort_key = CanonicalSortKey::new("zzz-theorem-tiebreak");
        candidates
            .candidates
            .iter_mut()
            .find(|candidate| candidate.kind == VcKind::GeneratedSethood)
            .expect("sethood candidate")
            .sort_key = CanonicalSortKey::new("aaa-sethood-tiebreak");

        let vc_set = normalize(&candidates);

        assert_eq!(vc_set.vcs()[0].kind, VcKind::TheoremProofStep);
        assert_eq!(vc_set.vcs()[1].kind, VcKind::GeneratedSethood);
    }

    #[test]
    fn documented_kind_rank_covers_all_task_eight_variants() {
        let kinds = vec![
            VcKind::TheoremProofStep,
            VcKind::TerminalProofGoal,
            VcKind::DefinitionCorrectness,
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::Registration,
            },
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::Redefinition,
            },
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::Reduction,
            },
            VcKind::RegistrationStyleCorrectness {
                style: RegistrationCorrectnessKind::ExplicitCoreSeed,
            },
            VcKind::CheckerInitial,
            VcKind::GeneratedNonEmptiness,
            VcKind::GeneratedSethood,
            VcKind::FraenkelMembershipAxiom,
            VcKind::AlgorithmPrecondition,
            VcKind::AlgorithmPostcondition,
            VcKind::CallPrecondition,
            VcKind::AlgorithmAssertion,
            VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Entry,
            },
            VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Preservation,
            },
            VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Break,
            },
            VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Continue,
            },
            VcKind::LoopInvariant {
                phase: LoopInvariantPhase::Exit,
            },
            VcKind::RangeLoop {
                obligation: RangeLoopObligation::PositiveStep,
            },
            VcKind::RangeLoop {
                obligation: RangeLoopObligation::RangeBound,
            },
            VcKind::RangeLoop {
                obligation: RangeLoopObligation::HiddenIndex,
            },
            VcKind::CollectionLoop {
                obligation: CollectionLoopObligation::Finiteness,
            },
            VcKind::CollectionLoop {
                obligation: CollectionLoopObligation::OrderIndependence,
            },
            VcKind::Termination,
            VcKind::PartialTermination,
            VcKind::GhostErasureSafety,
            VcKind::PolicyDeferredTraceability,
        ];

        for pair in kinds.windows(2) {
            assert!(
                kind_classification_rank(&pair[0]) < kind_classification_rank(&pair[1]),
                "{:?} must rank before {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn normalization_preserves_deferred_flow_status_accounting() {
        let source = sample_source_id();
        let flow_output = sample_flow_output(source);
        let handoff = seed_handoff_with_sites(vec![flow_seed(
            source,
            CoreFormulaId::new(10),
            "program/0/contract/requires/0",
            "flow:0:requires:0",
            flow_site(ControlFlowObligationSiteKind::Requires, 0),
        )]);
        let candidates = generate_with_flows(&handoff, &flow_output);

        let vc_set = normalize(&candidates);

        assert_eq!(vc_set.vcs().len(), 1);
        assert_eq!(vc_set.vcs()[0].kind, VcKind::AlgorithmPrecondition);
        assert_eq!(vc_set.vcs()[0].status, VcStatus::Open);
        assert!(matches!(
            vc_set.seed_accounting_for(ObligationHandoffId::new(0)),
            Some(SeedAccounting {
                seed_status: ObligationSeedStatus::Deferred,
                mapping: SeedVcMapping::One { vc },
                ..
            }) if *vc == VcId::new(0)
        ));
    }

    #[test]
    fn normalization_preserves_non_open_status_without_later_phase_outputs() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(0)),
                "proof/step/0",
                "theorem:sample:proof-step:0",
            )
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);
        let mut candidates = generate(&handoff);
        candidates.candidates[0].status = VcStatus::PolicyOpen {
            policy: PolicyKey::new("task-8-preserve-status"),
        };

        let vc_set = normalize(&candidates);
        let vc = &vc_set.vcs()[0];

        assert_eq!(
            vc.status,
            VcStatus::PolicyOpen {
                policy: PolicyKey::new("task-8-preserve-status")
            }
        );
        assert!(
            vc.provenance
                .iter()
                .any(|provenance| provenance.phase == VcProvenancePhase::Normalization)
        );
        assert!(!vc.provenance.iter().any(|provenance| matches!(
            provenance.phase,
            VcProvenancePhase::StatusPolicy
                | VcProvenancePhase::Discharge
                | VcProvenancePhase::DependencySlice
        )));
        assert_eq!(vc_set.generated_formulas(), []);
    }

    #[test]
    fn normalization_debug_rendering_is_deterministic_and_fails_closed_for_core_goal_anchors() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(0)),
                "proof/step/0",
                "theorem:sample:proof-step:0",
            )
            .with_context(vec![CoreFormulaId::new(1)])
            .with_label("A1")
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);

        let first = normalize(&generate(&handoff));
        let second = normalize(&generate(&handoff));

        assert_eq!(first.debug_text(), second.debug_text());
        assert!(first.debug_text().contains("vc-set-debug-v1"));
        assert_eq!(first.vcs()[0].local_context.entries().len(), 1);
        let anchor = &first.vcs()[0].anchor;
        assert!(matches!(
            anchor.completeness,
            AnchorCompleteness::Incomplete { .. }
        ));
        assert!(anchor.source_shape_hash.is_available());
        assert!(!anchor.canonical_goal_hash.is_available());
        assert!(!anchor.canonical_context_hash.is_available());
    }

    #[test]
    fn normalization_rejects_duplicate_candidate_sort_key() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::TheoremProof,
                    Some(CoreFormulaId::new(0)),
                    "proof/step/0",
                    "theorem:sample:proof-step:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(0),
                },
            ),
            (
                obligation_seed(
                    source,
                    ObligationSeedKind::GeneratedSethood,
                    Some(CoreFormulaId::new(1)),
                    "generated/sethood/0",
                    "generated:sethood:0",
                )
                .into(),
                ObligationHandoffOrigin::ExistingCore {
                    seed: ObligationSeedId::new(1),
                },
            ),
        ]);
        let mut candidates = generate(&handoff);
        candidates.candidates[1].sort_key = candidates.candidates[0].sort_key.clone();

        assert!(matches!(
            try_normalize(&candidates),
            Err(GeneratorError::DuplicateCandidateSortKey { .. })
        ));
    }

    #[test]
    fn normalization_rejects_duplicate_seed_output() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(0)),
                "proof/step/0",
                "theorem:sample:proof-step:0",
            )
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);
        let mut candidates = generate(&handoff);
        let candidate = candidates.candidates()[0].clone();
        candidates.no_candidates.push(CoreGenerationNoCandidate {
            handoff: candidate.handoff,
            origin: candidate.origin,
            seed_status: candidate.seed_status,
            reason: SeedNoVcReason::DeferredExternal(VcText::new("duplicate test row")),
        });

        assert!(matches!(
            try_normalize(&candidates),
            Err(GeneratorError::DuplicateSeedOutput { handoff })
                if handoff == ObligationHandoffId::new(0)
        ));
    }

    #[test]
    fn normalization_keeps_existing_expanded_mapping_contract_validated() {
        let source = sample_source_id();
        let handoff = seed_handoff(vec![(
            obligation_seed(
                source,
                ObligationSeedKind::TheoremProof,
                Some(CoreFormulaId::new(0)),
                "proof/step/0",
                "theorem:sample:proof-step:0",
            )
            .into(),
            ObligationHandoffOrigin::ExistingCore {
                seed: ObligationSeedId::new(0),
            },
        )]);
        let vc_set = normalize(&generate(&handoff));
        let mut parts = VcSetParts {
            schema_version: vc_set.schema_version().clone(),
            snapshot: vc_set.snapshot(),
            source: vc_set.source(),
            module: vc_set.module().clone(),
            generated_formulas: vc_set.generated_formulas().to_vec(),
            vcs: vc_set.vcs().to_vec(),
            seed_accounting: vc_set.seed_accounting().to_vec(),
        };
        parts.seed_accounting[0].mapping = SeedVcMapping::Expanded {
            vcs: vec![ExpandedVcRef {
                expansion_index: 0,
                vc: VcId::new(0),
            }],
            expansion_schema: ExpansionSchemaVersion::new("task-8-validation-test"),
        };

        VcSet::try_new(parts).expect("expanded mapping remains validated by VcSet");
    }

    fn generate(handoff: &ObligationSeedHandoff) -> CoreGenerationCandidateSet {
        let intake = SeedIntakeTable::try_from_handoff(handoff).expect("intake");
        CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
            schema_version: &GenerationSchemaVersion::new("generator-task-6-test"),
            module: &VcModuleRef::new("sample"),
            intake: &intake,
            handoff,
            flow_output: None,
        })
        .expect("generation candidates")
    }

    fn normalize(candidates: &CoreGenerationCandidateSet) -> VcSet {
        try_normalize(candidates).expect("normalized vc set")
    }

    fn try_normalize(candidates: &CoreGenerationCandidateSet) -> Result<VcSet, GeneratorError> {
        let source = sample_source_id();
        CoreGenerationCandidateSet::try_normalize(VcNormalizationInput {
            schema_version: &VcSchemaVersion::new("vc-task-8-test"),
            snapshot: sample_snapshot_id(),
            source,
            candidates,
        })
    }

    fn generate_with_flows(
        handoff: &ObligationSeedHandoff,
        flow_output: &ControlFlowOutput,
    ) -> CoreGenerationCandidateSet {
        let intake = SeedIntakeTable::try_from_handoff(handoff).expect("intake");
        CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
            schema_version: &GenerationSchemaVersion::new("generator-task-7-test"),
            module: &VcModuleRef::new("sample"),
            intake: &intake,
            handoff,
            flow_output: Some(flow_output),
        })
        .expect("generation candidates")
    }

    fn assert_policy_input(candidate: &CoreGenerationCandidate, key: &str, value: &str) {
        assert!(
            candidate
                .local_context
                .policy_inputs()
                .iter()
                .any(|input| input.key == PolicyKey::new(key)
                    && input.value == PolicyValue::new(value)),
            "missing policy input {key}={value} in {:?}",
            candidate.local_context.policy_inputs()
        );
    }

    fn seed_handoff(
        entries: Vec<(ObligationSeed, ObligationHandoffOrigin)>,
    ) -> ObligationSeedHandoff {
        seed_handoff_with_sites(
            entries
                .into_iter()
                .map(|(seed, origin)| (seed, origin, None))
                .collect(),
        )
    }

    fn seed_handoff_with_sites(
        entries: Vec<(
            ObligationSeed,
            ObligationHandoffOrigin,
            Option<ControlFlowObligationSite>,
        )>,
    ) -> ObligationSeedHandoff {
        let mut table = ObligationHandoffTable::new();
        let mut source_map = std::collections::BTreeMap::new();

        for (seed, origin, flow_site) in entries {
            let source = seed.source.clone();
            let id = table.insert(ObligationHandoffEntry {
                seed,
                origin,
                flow_site,
            });
            source_map.insert(id, source);
        }

        ObligationSeedHandoff {
            entries: table,
            source_map,
        }
    }

    fn flow_seed(
        source: mizar_session::SourceId,
        goal: CoreFormulaId,
        local_path: &str,
        semantic_origin: &str,
        site: impl Into<ControlFlowObligationSite>,
    ) -> (
        ObligationSeed,
        ObligationHandoffOrigin,
        Option<ControlFlowObligationSite>,
    ) {
        (
            obligation_seed(
                source,
                ObligationSeedKind::AlgorithmContract,
                Some(goal),
                local_path,
                semantic_origin,
            )
            .with_status(ObligationSeedStatus::Deferred)
            .into(),
            flow_origin(),
            Some(site.into()),
        )
    }

    fn flow_origin() -> ObligationHandoffOrigin {
        ObligationHandoffOrigin::FlowDerived {
            flow: ControlFlowId::new(0),
            algorithm: CoreAlgorithmId::new(0),
        }
    }

    fn flow_site(kind: ControlFlowObligationSiteKind, ordinal: usize) -> FlowSiteBuilder {
        FlowSiteBuilder {
            site: ControlFlowObligationSite {
                kind,
                ordinal,
                statement: None,
                block: None,
                loop_id: None,
                exit: None,
                local: None,
                assignment_effect: None,
            },
        }
    }

    struct FlowSiteBuilder {
        site: ControlFlowObligationSite,
    }

    impl FlowSiteBuilder {
        fn with_statement(mut self, statement: usize) -> Self {
            self.site.statement = Some(mizar_core::core_ir::CoreAlgorithmStmtId::new(statement));
            self
        }

        fn with_block(mut self, block: usize) -> Self {
            self.site.block = Some(BasicBlockId::new(block));
            self
        }

        fn with_loop(mut self, loop_id: usize) -> Self {
            self.site.loop_id = Some(LoopId::new(loop_id));
            self
        }

        fn with_exit(mut self, exit: usize) -> Self {
            self.site.exit = Some(ControlFlowExitId::new(exit));
            self
        }

        fn with_local(mut self, local: usize) -> Self {
            self.site.local = Some(LocalId::new(local));
            self
        }

        fn with_assignment_effect(mut self, effect: usize) -> Self {
            self.site.assignment_effect =
                Some(mizar_core::control_flow::AssignmentEffectId::new(effect));
            self
        }
    }

    impl From<FlowSiteBuilder> for ControlFlowObligationSite {
        fn from(builder: FlowSiteBuilder) -> Self {
            builder.site
        }
    }

    fn sample_flow_output(source: mizar_session::SourceId) -> ControlFlowOutput {
        sample_flow_output_for_algorithm(source, CoreAlgorithmId::new(0))
    }

    fn sample_flow_output_for_algorithm(
        source: mizar_session::SourceId,
        algorithm: CoreAlgorithmId,
    ) -> ControlFlowOutput {
        let mut loops = ControlFlowLoopTable::new();
        let loop_id = loops.insert(ControlFlowLoop {
            algorithm,
            header: BasicBlockId::new(0),
            body: BasicBlockId::new(1),
            exit: BasicBlockId::new(2),
            condition: CoreFormulaId::new(90),
            invariants: vec![CoreFormulaId::new(91)],
            decreasing: Vec::new(),
            source: source_ref(source),
        });
        assert_eq!(loop_id, LoopId::new(0));

        let mut exits = ControlFlowExitTable::new();
        let break_exit = exits.insert(ControlFlowExit {
            algorithm,
            statement: None,
            from: BasicBlockId::new(3),
            target: Some(BasicBlockId::new(2)),
            kind: ControlFlowExitKind::Break { loop_id },
            source: source_ref(source),
        });
        let continue_exit = exits.insert(ControlFlowExit {
            algorithm,
            statement: None,
            from: BasicBlockId::new(4),
            target: Some(BasicBlockId::new(0)),
            kind: ControlFlowExitKind::Continue { loop_id },
            source: source_ref(source),
        });
        assert_eq!(break_exit, ControlFlowExitId::new(0));
        assert_eq!(continue_exit, ControlFlowExitId::new(1));

        let mut flows = ControlFlowTable::new();
        let flow_id = flows.insert(ControlFlowIr {
            algorithm,
            item: CoreItemId::new(0),
            symbol: sample_symbol("Algorithm"),
            entry: BasicBlockId::new(0),
            blocks: ControlFlowBlockTable::new(),
            locals: ControlFlowLocalTable::new(),
            contexts: ProgramContextTable::new(),
            context_facts: ContextFactTable::new(),
            assignment_effects: mizar_core::control_flow::AssignmentEffectTable::new(),
            call_sites: CallSiteTable::new(),
            contracts: ControlFlowContractSet {
                requires: vec![ContractSite {
                    kind: ContractSiteKind::Requires,
                    formula: CoreFormulaId::new(10),
                    placement: ContractSitePlacement::Entry {
                        block: BasicBlockId::new(0),
                        context: ProgramContextId::new(0),
                    },
                    source: source_ref(source),
                }],
                ensures: vec![ContractSite {
                    kind: ContractSiteKind::Ensures,
                    formula: CoreFormulaId::new(11),
                    placement: ContractSitePlacement::Return {
                        block: BasicBlockId::new(2),
                        exit: ControlFlowExitId::new(0),
                    },
                    source: source_ref(source),
                }],
                calls: Vec::new(),
                assertions: vec![
                    AssertionSite {
                        formula: CoreFormulaId::new(12),
                        placement: AssertionPlacement::AlgorithmContract {
                            block: BasicBlockId::new(0),
                            context: ProgramContextId::new(0),
                        },
                        source: source_ref(source),
                    },
                    AssertionSite {
                        formula: CoreFormulaId::new(13),
                        placement: AssertionPlacement::Statement {
                            statement: mizar_core::core_ir::CoreAlgorithmStmtId::new(0),
                            block: BasicBlockId::new(1),
                            successor_context: ProgramContextId::new(0),
                        },
                        source: source_ref(source),
                    },
                ],
                loop_invariants: vec![
                    LoopInvariantSite {
                        formula: CoreFormulaId::new(14),
                        placement: LoopInvariantPlacement::AlgorithmContract {
                            block: BasicBlockId::new(0),
                            context: ProgramContextId::new(0),
                        },
                        source: source_ref(source),
                    },
                    LoopInvariantSite {
                        formula: CoreFormulaId::new(15),
                        placement: LoopInvariantPlacement::Header {
                            loop_id: LoopId::new(0),
                            block: BasicBlockId::new(0),
                        },
                        source: source_ref(source),
                    },
                    LoopInvariantSite {
                        formula: CoreFormulaId::new(16),
                        placement: LoopInvariantPlacement::NormalBackedge {
                            loop_id: LoopId::new(0),
                            from: BasicBlockId::new(1),
                            to: BasicBlockId::new(0),
                        },
                        source: source_ref(source),
                    },
                    LoopInvariantSite {
                        formula: CoreFormulaId::new(17),
                        placement: LoopInvariantPlacement::BreakExit {
                            loop_id: LoopId::new(0),
                            exit: ControlFlowExitId::new(0),
                        },
                        source: source_ref(source),
                    },
                    LoopInvariantSite {
                        formula: CoreFormulaId::new(18),
                        placement: LoopInvariantPlacement::ContinueExit {
                            loop_id: LoopId::new(0),
                            exit: ControlFlowExitId::new(1),
                        },
                        source: source_ref(source),
                    },
                ],
                decreasing: Vec::new(),
            },
            loops,
            exits,
            ghost_effects: ControlFlowGhostTable::default(),
            termination: mizar_core::control_flow::ControlFlowTerminationPlan::default(),
            source_map: ControlFlowSourceMap::default(),
            diagnostics: ControlFlowDiagnosticTable::new(),
        });
        assert_eq!(flow_id, ControlFlowId::new(0));

        ControlFlowOutput {
            flows,
            flow_map: std::collections::BTreeMap::from([(algorithm, flow_id)]),
        }
    }

    fn sample_symbol(name: &str) -> SymbolId {
        let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("vc_fixture"));
        SymbolId::new(
            module,
            LocalSymbolId::new(name),
            FullyQualifiedName::new(format!("pkg::vc_fixture::{name}")),
        )
    }

    fn obligation_seed(
        source: mizar_session::SourceId,
        kind: ObligationSeedKind,
        goal: Option<CoreFormulaId>,
        local_path: &str,
        semantic_origin: &str,
    ) -> ObligationSeedBuilder {
        ObligationSeedBuilder {
            seed: ObligationSeed {
                owner: CoreItemId::new(0),
                kind,
                goal,
                context: Vec::new(),
                local_path: LocalProofOrProgramPath::new(local_path),
                label: None,
                semantic_origin: NormalizedSemanticOrigin::new(semantic_origin),
                provenance: vec![CoreProvenance::new(
                    CoreProvenancePhase::ProofSkeleton,
                    CoreProvenanceKey::new(local_path),
                )],
                source: source_ref(source),
                core_refs: goal.map(CoreNodeRef::Formula).into_iter().collect(),
                status: ObligationSeedStatus::Active,
                diagnostics: Vec::new(),
            },
        }
    }

    struct ObligationSeedBuilder {
        seed: ObligationSeed,
    }

    impl ObligationSeedBuilder {
        fn with_context(mut self, context: Vec<CoreFormulaId>) -> Self {
            self.seed.context = context;
            self
        }

        fn with_label(mut self, label: &str) -> Self {
            self.seed.label = Some(CoreLabelRef::new(label));
            self
        }

        fn with_core_ref(mut self, reference: CoreNodeRef) -> Self {
            self.seed.core_refs.push(reference);
            self
        }

        fn with_provenance_key(mut self, key: &str) -> Self {
            self.seed.provenance.push(CoreProvenance::new(
                CoreProvenancePhase::Generated,
                CoreProvenanceKey::new(key),
            ));
            self
        }

        fn with_status(mut self, status: ObligationSeedStatus) -> Self {
            self.seed.status = status;
            self
        }

        fn with_diagnostics(mut self, diagnostics: Vec<CoreDiagnosticId>) -> Self {
            self.seed.diagnostics = diagnostics;
            self
        }
    }

    impl From<ObligationSeedBuilder> for ObligationSeed {
        fn from(builder: ObligationSeedBuilder) -> Self {
            builder.seed
        }
    }

    fn source_ref(source: mizar_session::SourceId) -> CoreSourceRef {
        CoreSourceRef::direct(SourceRange {
            source_id: source,
            start: 0,
            end: 10,
        })
        .with_provenance(vec![CoreProvenance::new(
            CoreProvenancePhase::Generated,
            CoreProvenanceKey::new("source"),
        )])
    }

    fn generated_source_ref() -> CoreSourceRef {
        CoreSourceRef::generated(GeneratedFrom {
            owner: CoreNodeRef::Item(CoreItemId::new(0)),
            kind: GeneratedOriginKind::TypePredicate,
            key: GeneratedOriginKey::new("generated-source"),
            reason: CoreProvenanceKey::new("generated"),
        })
    }

    fn sample_source_id() -> mizar_session::SourceId {
        InMemorySessionIdAllocator::new()
            .next_source_id(sample_snapshot_id())
            .expect("source id")
    }

    fn sample_snapshot_id() -> BuildSnapshotId {
        BuildSnapshotId::from_published_schema_str(
            "mizar-session-build-snapshot-v1:\
             2222222222222222222222222222222222222222222222222222222222222222",
        )
        .expect("snapshot id")
    }

    #[test]
    fn generated_source_refs_remain_unranged() {
        let source = generated_source_ref();
        assert_eq!(source_range(&source), None);
    }
}
