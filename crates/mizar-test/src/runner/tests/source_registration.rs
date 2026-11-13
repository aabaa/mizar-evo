use super::*;

#[test]
fn source_registration_discharge_handoff_replays_actual_default_phase_and_rejects_corruption() {
    use mizar_core::core_ir::{CoreDiagnosticId, CoreSourceAnchor};
    use mizar_vc::{
        discharge::{DischargeInput, DischargePolicy, try_discharge},
        kernel_evidence_handoff::build_source_existential_kernel_handoff,
        vc_ir::*,
    };
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original
            .replace("wmarked", "retagged")
            .replace("oamarked", "leftmarked")
            .replace("obmarked", "rightmarked")
            .replace("X", "Value");
        for text in [&original, &renamed] {
            let (source, nodes, symbols) = if multiple {
                step5c13_registration_inputs(text)
            } else {
                step5c3_registration_inputs(text)
            }
            .unwrap();
            let checked = mizar_checker::registration_resolution::check_source_existential_registration_proof(&source, &nodes, &symbols).unwrap();
            let core =
                mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
            let open = step5c3_registration_vcs(&core).unwrap();
            let registrations = if multiple { 3 } else { 1 };
            assert_eq!(open.vcs().len(), registrations * 2);
            let discharge = try_discharge(DischargeInput {
                vc_set: &open,
                policy: &DischargePolicy::default(),
            })
            .unwrap();
            let changed = discharge.vc_set();
            assert_eq!(
                changed
                    .vcs()
                    .iter()
                    .filter(|vc| vc.status == VcStatus::NeedsAtp)
                    .count(),
                registrations
            );
            assert_eq!(
                changed
                    .vcs()
                    .iter()
                    .filter(|vc| matches!(vc.status, VcStatus::Discharged { .. }))
                    .count(),
                registrations
            );
            assert!(open.vcs().iter().all(|vc| vc.status == VcStatus::Open));
            assert!(checked.database().activated().is_empty());
            for before in open.vcs() {
                let old = build_source_existential_kernel_handoff(&core, &open, before.id).unwrap();
                assert!(old.targets_vc(&open, before.id).unwrap());
                assert!(!old.targets_vc(changed, before.id).unwrap());
                let after = changed.vc(before.id).unwrap();
                assert_eq!(after.goal, before.goal);
                assert_eq!(after.premises, before.premises);
                assert_eq!(after.source, before.source);
                assert!(after.provenance.starts_with(&before.provenance));
                if after.status != VcStatus::NeedsAtp {
                    assert!(
                        build_source_existential_kernel_handoff(&core, changed, after.id).is_err()
                    );
                    continue;
                }
                let handoff =
                    build_source_existential_kernel_handoff(&core, changed, after.id).unwrap();
                assert!(handoff.targets_vc(changed, after.id).unwrap());
                assert!(!handoff.targets_vc(&open, after.id).unwrap());
                assert_ne!(handoff.context_identity_hash(), old.context_identity_hash());
                assert_eq!(
                    handoff.canonical_evidence().symbol_manifest(),
                    old.canonical_evidence().symbol_manifest()
                );
                assert_eq!(
                    handoff.canonical_evidence().variable_manifest(),
                    old.canonical_evidence().variable_manifest()
                );
                assert_eq!(
                    handoff.canonical_evidence().formula_evidence(),
                    old.canonical_evidence().formula_evidence()
                );
                assert_eq!(
                    handoff.canonical_evidence().substitutions(),
                    old.canonical_evidence().substitutions()
                );
                assert_eq!(
                    handoff.canonical_evidence().provenance().len(),
                    old.canonical_evidence().provenance().len()
                );
                for (actual, original) in handoff
                    .canonical_evidence()
                    .provenance()
                    .iter()
                    .zip(old.canonical_evidence().provenance())
                {
                    assert_eq!(actual.provenance_id, original.provenance_id);
                    assert_eq!(actual.formula_fingerprint, original.formula_fingerprint);
                    assert_eq!(actual.payload, original.payload);
                    assert_eq!(&actual.target_vc, handoff.canonical_evidence().target_vc());
                }
                assert_eq!(
                    handoff.canonical_evidence().final_goal(),
                    old.canonical_evidence().final_goal()
                );
            }
            let selected = changed
                .vcs()
                .iter()
                .find(|vc| vc.status == VcStatus::NeedsAtp)
                .unwrap()
                .id;
            let sibling = changed
                .vcs()
                .iter()
                .find(|vc| matches!(vc.status, VcStatus::Discharged { .. }))
                .unwrap()
                .id;
            for mutation in 0..15 {
                let mut parts = VcSetParts {
                    schema_version: changed.schema_version().clone(),
                    snapshot: changed.snapshot(),
                    source: changed.source(),
                    module: changed.module().clone(),
                    generated_formulas: changed.generated_formulas().to_vec(),
                    vcs: changed.vcs().to_vec(),
                    seed_accounting: changed.seed_accounting().to_vec(),
                };
                match mutation {
                    0 => parts.vcs[selected.index()].status = VcStatus::Open,
                    1 => {
                        parts.vcs[selected.index()].status =
                            changed.vc(sibling).unwrap().status.clone()
                    }
                    2 => {
                        parts.vcs[selected.index()].status = VcStatus::PolicyOpen {
                            policy: PolicyKey::new("test"),
                        }
                    }
                    3 => {
                        parts.vcs[selected.index()].status = VcStatus::AssumedByPolicy {
                            policy: PolicyKey::new("test"),
                            marker: changed.vc(selected).unwrap().premises[0].clone(),
                        }
                    }
                    4 => {
                        parts.vcs[selected.index()].status = VcStatus::SkippedDueToInvalidInput {
                            reason: VcText::new("test"),
                        }
                    }
                    5 => {
                        parts.vcs[selected.index()].status = VcStatus::DeferredExternal {
                            reason: VcText::new("test"),
                        }
                    }
                    6 => {
                        parts.vcs[selected.index()].status = VcStatus::Error {
                            diagnostic: CoreDiagnosticId::new(0),
                        }
                    }
                    7 => {
                        parts.vcs[selected.index()].provenance.pop();
                    }
                    8 => {
                        parts.vcs[sibling.index()].provenance.pop();
                    }
                    9 => parts.vcs[sibling.index()].status = VcStatus::Open,
                    10 => parts.vcs[sibling.index()].source.related.clear(),
                    11 => {
                        let CoreSourceAnchor::SourceRange(range) =
                            &mut parts.vcs[selected.index()].source.primary.anchor
                        else {
                            panic!("actual source range");
                        };
                        range.start += 1;
                    }
                    12 => parts.generated_formulas[0].provenance.clear(),
                    13 => parts.generated_formulas[0].shape = VcGeneratedFormulaShape::True,
                    _ => parts.module = VcModuleRef::new("foreign-module"),
                }
                let altered = VcSet::try_new(parts)
                    .unwrap_or_else(|error| panic!("coherent mutation {mutation}: {error}"));
                assert!(
                    build_source_existential_kernel_handoff(&core, &altered, selected).is_err(),
                    "accepted mutation {mutation}, multiple={multiple}"
                );
            }
        }
    }
}

#[test]
fn source_registration_atp_projections_preserve_structured_input_and_reject_corruption() {
    use mizar_atp::{
        problem::*,
        smtlib_encoder::{SmtLibDialect, encode_smtlib},
        tptp_encoder::{TptpDialect, encode_tptp},
        translator::translate_source_existential_registration as translate,
    };
    use mizar_core::core_ir::{CoreSourceAnchor, CoreTermKind};
    use mizar_vc::{
        discharge::{DischargeInput, DischargePolicy, try_discharge},
        kernel_evidence_handoff::build_source_existential_kernel_handoff,
        vc_ir::*,
    };
    let profile = |fragment, equality, quantifiers, soft_types, format| {
        LogicProfile::try_new(
            "source-registration", fragment, equality, quantifiers, soft_types,
            NativePropertySupport::Unsupported, [format].into_iter().collect(),
        ).unwrap()
    };
    let fof = profile(LogicFragment::Fof, EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder, SoftTypeStrategy::GuardPredicates, ConcreteFormat::Tptp);
    let smt = profile(LogicFragment::SmtLibUninterpreted, EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder, SoftTypeStrategy::GuardPredicates, ConcreteFormat::SmtLib);
    let hex = |bytes: &[u8]| bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    for (multiple, case) in [(false, step5c3_registration_case()), (true, step5c13_registration_case())] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original.replace("wmarked", "retagged")
            .replace("oamarked", "leftmarked").replace("obmarked", "rightmarked")
            .replace("X", "Value");
        let negative_sources = (0..if multiple { 2 } else { 1 }).map(|index| {
            let changed = if multiple {
                let attribute = ["oamarked", "obmarked"][index];
                original.replace(&format!("X is {attribute} means X = X"), &format!("X is {attribute} means not X = X"))
            } else { original.replacen("means X = X", "means not X = X", 1) };
            assert_ne!(changed, original);
            changed
        }).collect::<Vec<_>>();
        let mut variants = vec![(&original, None), (&renamed, None)];
        variants.extend(negative_sources.iter().enumerate().map(|(index, text)| (text, Some(index))));
        for (text, negative_definition) in variants {
            let (source, nodes, symbols) = if multiple {
                step5c13_registration_inputs(text)
            } else {
                step5c3_registration_inputs(text)
            }.unwrap();
            let checked = mizar_checker::registration_resolution::check_source_existential_registration_proof(
                &source, &nodes, &symbols).unwrap();
            let core = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
            let open = step5c3_registration_vcs(&core).unwrap();
            let discharge = try_discharge(DischargeInput {
                vc_set: &open, policy: &DischargePolicy::default(),
            }).unwrap();
            let vcs = discharge.vc_set();
            for vc in open.vcs() {
                assert_eq!(vc.status, VcStatus::Open);
                assert!(translate(&core, &open, vc.id, fof.clone()).is_err());
            }
            let definitions = core.definitions().iter().map(|(_, row)| row).collect::<Vec<_>>();
            let formal = &definitions[0].params[0];
            assert!(definitions.iter().all(|row| row.params[0].var == formal.var));
            let variable = AtpSymbolName::new(format!("definition-formal:{:?}:{:?}", definitions[0].symbol, formal.var));
            let formal_term = AtpTerm::Variable(variable.clone());
            let attribute = |index: usize, term| AtpFormulaTree::Atom(AtpAtom::new(
                format!("attribute:{:?}", definitions[index].symbol), vec![term]));
            let equality = AtpFormulaTree::Equality { left: formal_term.clone(), right: formal_term.clone() };
            let mut expected_declarations = Vec::new();
            for row in &definitions {
                expected_declarations.push((AtpSymbolName::new(format!("attribute:{:?}", row.symbol)),
                    AtpDeclarationKind::Predicate, 1,
                    AtpSymbolSource::MizarSymbol(AtpSourceBinding::new(format!("attribute:{:?}", row.symbol))),
                    AtpSourceRef::CheckerOwnedFact(AtpSourceBinding::new(format!("{:?}", row.symbol))),
                    format!("{:?}", row.source)));
            }
            let origins = core.generated().iter().map(|(_, row)| row).collect::<Vec<_>>();
            for origin in &origins {
                let functor = origin.functor.as_ref().unwrap();
                expected_declarations.push((AtpSymbolName::new(format!("choice:{functor:?}")),
                    AtpDeclarationKind::Function, 0,
                    AtpSymbolSource::MizarSymbol(AtpSourceBinding::new(format!("choice:{functor:?}"))),
                    AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new(format!("{functor:?}"))), format!("{origin:?}")));
            }
            expected_declarations.push((variable.clone(), AtpDeclarationKind::GeneratedBinder, 0,
                AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new(variable.as_str())),
                AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new(format!("{:?}", formal.var))), format!("{formal:?}")));
            expected_declarations.push((AtpSymbolName::new("builtin:set"), AtpDeclarationKind::Predicate, 1,
                AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("builtin:set")),
                AtpSourceRef::TypeFact(AtpSourceBinding::new("builtin.set")),
                format!("{:?}", core.formulas().get(formal.ty_guard.unwrap()).unwrap().source)));
            expected_declarations.sort_by(|left, right| left.0.cmp(&right.0));
            let selected = vcs.vcs().iter().filter(|vc| vc.status == VcStatus::NeedsAtp).collect::<Vec<_>>();
            assert_eq!(selected.len(), origins.len());
            for (registration, vc) in selected.iter().enumerate() {
                let functor = origins[registration].functor.as_ref().unwrap();
                let choice = AtpTerm::Function { function: AtpSymbolName::new(format!("choice:{functor:?}")), arguments: vec![] };
                assert!(core.terms().iter().any(|(_, term)| matches!(&term.kind,
                    CoreTermKind::Apply { functor: actual, args } if actual == functor && args.is_empty())));
                let indices = if registration == 2 { vec![0, 1] } else { vec![registration] };
                let mut expected_axioms = vec![AtpFormulaTree::Atom(AtpAtom::new("builtin:set", vec![choice.clone()]))];
                // Universally close set(x) -> (attribute(x) <-> the actual equality or negation).
                expected_axioms.extend(indices.iter().map(|&index| {
                    let body = if negative_definition == Some(index) {
                        AtpFormulaTree::Not(Box::new(equality.clone()))
                    } else { equality.clone() };
                    AtpFormulaTree::Forall {
                        binders: vec![AtpBinder::new(variable.clone(), None)],
                        body: Box::new(AtpFormulaTree::Or(vec![
                            AtpFormulaTree::Not(Box::new(AtpFormulaTree::Atom(AtpAtom::new("builtin:set", vec![formal_term.clone()])))),
                            AtpFormulaTree::And(vec![
                                AtpFormulaTree::Or(vec![AtpFormulaTree::Not(Box::new(attribute(index, formal_term.clone()))), body.clone()]),
                                AtpFormulaTree::Or(vec![AtpFormulaTree::Not(Box::new(body)), attribute(index, formal_term.clone())]),
                            ]),
                        ])),
                    }
                }));
                let goal = if registration == 2 {
                    AtpFormulaTree::And(indices.iter().map(|index| attribute(*index, choice.clone())).collect())
                } else { attribute(indices[0], choice.clone()) };
                let handoff = build_source_existential_kernel_handoff(&core, vcs, vc.id).unwrap();
                assert!(handoff.targets_vc(vcs, vc.id).unwrap());
                for caller in [&fof, &smt] {
                    let problem = translate(&core, vcs, vc.id, caller.clone()).unwrap();
                    assert_eq!(problem, translate(&core, vcs, vc.id, caller.clone()).unwrap());
                    assert_eq!(problem.logic_profile(), caller);
                    let additional_format = LogicProfile::try_new(caller.name().clone(), caller.fragment(),
                        caller.equality(), caller.quantifiers(), caller.soft_types(), caller.native_properties(),
                        [ConcreteFormat::Tptp, ConcreteFormat::SmtLib].into_iter().collect()).unwrap();
                    let additional_problem = translate(&core, vcs, vc.id, additional_format.clone()).unwrap();
                    assert_eq!(additional_problem.logic_profile(), &additional_format);
                    assert_eq!(additional_problem.axioms(), problem.axioms());
                    assert_eq!(additional_problem.conjecture(), problem.conjecture());
                    assert_eq!(problem.vc_id(), vc.id);
                    assert_eq!(problem.expected_result(), ExpectedBackendResult::Unsat);
                    assert!(problem.properties().is_empty() && problem.diagnostics().is_empty());
                    assert_eq!(problem.axioms().iter().map(|row| row.formula().unwrap()).collect::<Vec<_>>(), expected_axioms.iter().collect::<Vec<_>>());
                    assert_eq!(problem.conjecture().formula(), Some(&goal));
                    assert_eq!(problem.type_context().guards().len(), 1);
                    let guard = &problem.type_context().guards()[0];
                    assert_eq!(guard.id(), AtpTypeGuardId::new(0));
                    assert_eq!(guard.formula(), &expected_axioms[0]);
                    assert_eq!(problem.declarations().len(), expected_declarations.len());
                    assert_eq!(problem.symbol_map().len(), expected_declarations.len());
                    for (index, (symbol, kind, arity, source, provenance_source, payload)) in expected_declarations.iter().enumerate() {
                        let declaration = &problem.declarations()[index];
                        assert_eq!(declaration.id(), AtpDeclarationId::new(index as u32));
                        assert_eq!(declaration.symbol(), symbol);
                        assert_eq!(declaration.kind(), *kind);
                        assert_eq!(declaration.arity(), *arity);
                        let binding = &problem.symbol_map()[index];
                        assert_eq!(binding.backend_symbol(), symbol);
                        assert_eq!(binding.source(), source);
                        let provenance = &problem.provenance()[declaration.provenance().index() as usize];
                        assert_eq!(provenance.source(), provenance_source);
                        assert_eq!(provenance.payload().as_str(), payload);
                    }
                    let target = handoff.canonical_evidence().target_vc();
                    assert_eq!(problem.target_binding().fingerprint(), &AtpFingerprint::new(target.algorithm_id, target.digest.clone()).unwrap());
                    assert_eq!(problem.target_binding().producer_binding().as_str(), format!("mizar-vc-kernel-evidence-handoff:{}", hex(handoff.canonical_hash().as_bytes())));
                    let mut premises = vc.premises.clone();
                    premises.sort();
                    assert_eq!(problem.axioms().len(), premises.len());
                    for (index, (axiom, premise)) in problem.axioms().iter().zip(&premises).enumerate() {
                        let PremiseRef::GeneratedFact { formula: VcFormulaRef::Generated(id) } = premise else { panic!("actual generated premise") };
                        let entry = handoff.canonical_evidence().formula_evidence().iter().find(|entry|
                            entry.producer_formula_ref() == Some(VcFormulaRef::Generated(*id))).unwrap();
                        let anchor = handoff.canonical_evidence().provenance().iter().find(|row| row.provenance_id == entry.provenance_id()).unwrap();
                        let provenance = &problem.provenance()[axiom.provenance().index() as usize];
                        assert_eq!(axiom.id(), AtpFormulaId::new(index as u32));
                        assert_eq!(provenance.source(), &AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new(format!("generated:{}", id.index() + 1))));
                        assert_eq!(provenance.payload().as_str(), format!("mizar-vc-handoff-provenance:{}", hex(&anchor.payload)));
                        if index == 0 {
                            let guard_provenance = &problem.provenance()[guard.provenance().index() as usize];
                            assert_eq!(guard_provenance.source(), provenance.source());
                            assert_eq!(guard_provenance.payload(), provenance.payload());
                        }
                    }
                    let goal_anchor = handoff.canonical_evidence().provenance().iter().find(|row|
                        row.provenance_id == handoff.canonical_evidence().final_goal().provenance_id).unwrap();
                    let goal_provenance = &problem.provenance()[problem.conjecture().provenance().index() as usize];
                    assert_eq!(goal_provenance.source(), &AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("goal:1")));
                    assert_eq!(goal_provenance.payload().as_str(), format!("mizar-vc-handoff-provenance:{}", hex(&goal_anchor.payload)));
                    assert_eq!(problem.conjecture().id(), AtpFormulaId::new(expected_axioms.len() as u32));
                    assert_eq!(problem.provenance().len(), expected_declarations.len() + expected_axioms.len() + 2);
                    if caller.fragment() == LogicFragment::Fof {
                        let encoded = encode_tptp(&problem, TptpDialect::Fof).unwrap();
                        assert_eq!(encoded, encode_tptp(&problem, TptpDialect::Fof).unwrap());
                        assert_eq!(encoded.formula_labels().len(), expected_axioms.len() + 2);
                    } else {
                        let encoded = encode_smtlib(&problem, SmtLibDialect::Uninterpreted).unwrap();
                        assert_eq!(encoded, encode_smtlib(&problem, SmtLibDialect::Uninterpreted).unwrap());
                        assert_eq!(encoded.assertion_labels().len(), expected_axioms.len() + 2);
                        assert!(encoded.assertion_labels().last().unwrap().is_negated());
                    }
                }
            }
            for vc in vcs.vcs().iter().filter(|vc| vc.status != VcStatus::NeedsAtp) {
                assert!(matches!(vc.status, VcStatus::Discharged { .. }));
                assert!(translate(&core, vcs, vc.id, fof.clone()).is_err());
            }
            assert!(translate(&core, vcs, VcId::new(vcs.vcs().len()), fof.clone()).is_err());
            let chosen = selected[0].id;
            let sibling = vcs.vcs().iter().find(|vc| matches!(vc.status, VcStatus::Discharged { .. })).unwrap().id;
            let foreign_text = text.replace("marked", "foreignmarked").replace("retagged", "foreigntagged");
            let (foreign_source, foreign_nodes, foreign_symbols) = if multiple {
                step5c13_registration_inputs(&foreign_text)
            } else {
                step5c3_registration_inputs(&foreign_text)
            }.unwrap();
            let foreign_checked = mizar_checker::registration_resolution::check_source_existential_registration_proof(
                &foreign_source, &foreign_nodes, &foreign_symbols).unwrap();
            let foreign = mizar_core::elaborator::lower_source_existential_registration(&foreign_checked).unwrap();
            assert!(translate(&foreign, vcs, chosen, fof.clone()).is_err());
            for mutation in 0..14 {
                let mut parts = VcSetParts {
                    schema_version: vcs.schema_version().clone(), snapshot: vcs.snapshot(), source: vcs.source(),
                    module: vcs.module().clone(), generated_formulas: vcs.generated_formulas().to_vec(),
                    vcs: vcs.vcs().to_vec(), seed_accounting: vcs.seed_accounting().to_vec(),
                };
                match mutation {
                    0 => parts.vcs[chosen.index()].status = VcStatus::Open,
                    1 => parts.vcs[chosen.index()].status = vcs.vc(sibling).unwrap().status.clone(),
                    2 => { parts.vcs[chosen.index()].provenance.pop(); }
                    3 => { parts.vcs[sibling.index()].provenance.pop(); }
                    4 => parts.vcs[sibling.index()].status = VcStatus::Open,
                    5 => parts.vcs[sibling.index()].source.related.clear(),
                    6 => {
                        let CoreSourceAnchor::SourceRange(range) = &mut parts.vcs[chosen.index()].source.primary.anchor else { panic!("source range") };
                        range.start += 1;
                    }
                    7 => parts.generated_formulas[0].provenance.clear(),
                    8 => parts.generated_formulas[0].shape = VcGeneratedFormulaShape::True,
                    9 => parts.module = VcModuleRef::new("foreign-module"),
                    10 => { parts.vcs[chosen.index()].premises.pop(); }
                    11 => parts.vcs[chosen.index()].goal = vcs.vc(sibling).unwrap().goal,
                    12 => {
                        let SeedVcMapping::Expanded { expansion_schema, .. } = &mut parts.seed_accounting[1].mapping else { panic!("expanded source seed") };
                        *expansion_schema = ExpansionSchemaVersion::new("foreign-schema");
                    }
                    _ => parts.vcs[sibling.index()].goal = vcs.vc(chosen).unwrap().goal,
                }
                let altered = VcSet::try_new(parts).unwrap_or_else(|error| panic!("coherent mutation {mutation}: {error}"));
                assert!(translate(&core, &altered, chosen, fof.clone()).is_err(), "accepted mutation {mutation}, multiple={multiple}");
            }
            for unsupported in [
                profile(LogicFragment::Fof, EqualitySupport::Unsupported, QuantifierPolicy::FirstOrder, SoftTypeStrategy::GuardPredicates, ConcreteFormat::Tptp),
                profile(LogicFragment::Fof, EqualitySupport::Supported, QuantifierPolicy::PropositionalOnly, SoftTypeStrategy::GuardPredicates, ConcreteFormat::Tptp),
                profile(LogicFragment::TffLike, EqualitySupport::Supported, QuantifierPolicy::FirstOrder, SoftTypeStrategy::GuardPredicates, ConcreteFormat::Tptp),
                profile(LogicFragment::Fof, EqualitySupport::Supported, QuantifierPolicy::FirstOrder, SoftTypeStrategy::BackendSorts, ConcreteFormat::Tptp),
                profile(LogicFragment::Fof, EqualitySupport::Supported, QuantifierPolicy::FirstOrder, SoftTypeStrategy::SortsAndGuards, ConcreteFormat::Tptp),
                profile(LogicFragment::Fof, EqualitySupport::Supported, QuantifierPolicy::FirstOrder, SoftTypeStrategy::GuardPredicates, ConcreteFormat::SmtLib),
                profile(LogicFragment::SmtLibUninterpreted, EqualitySupport::Supported, QuantifierPolicy::FirstOrder, SoftTypeStrategy::GuardPredicates, ConcreteFormat::Tptp),
            ] {
                assert!(translate(&core, vcs, chosen, unsupported).is_err());
            }
        }
    }
}

#[test]
fn source_registration_local_wire_round_trips_and_keeps_fresh_kernel_outcomes() {
    use std::collections::BTreeSet;
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::{FormulaEvidenceContext, ImportedFactContextLimits, KernelCheckPolicy,
            KernelCheckStatus, KernelContextIdentityEntry, KernelContextIdentityPayload,
            KernelContextIdentitySource, KernelEvidenceCheckInput, KernelEvidenceCheckKind,
            KernelEvidenceCheckLimits, KernelFormulaProducerRef, KernelVcGeneratedFormulaId,
            check_kernel_evidence},
        clause::{SymbolKind, Term},
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
        rejection::TargetVcFingerprint,
    };
    use mizar_vc::{
        discharge::{DischargeInput, DischargePolicy, try_discharge},
        kernel_evidence_handoff::{KernelContextIdentitySource as VcSource,
            build_source_existential_kernel_handoff},
        vc_ir::{VcFormulaRef, VcStatus},
    };
    for (multiple, case) in [(false, step5c3_registration_case()), (true, step5c13_registration_case())] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original.replace("wmarked", "retagged").replace("oamarked", "leftmarked")
            .replace("obmarked", "rightmarked").replace("X", "Value");
        for (renamed_names, base) in [(false, &original), (true, &renamed)] {
            for negative in std::iter::once(None).chain((0..if multiple { 2 } else { 1 }).map(Some)) {
                let text = if let Some(index) = negative {
                    let attribute = if multiple { ["oamarked", "obmarked"][index] } else { "wmarked" };
                    let attribute = if renamed_names {
                        attribute.replace("wmarked", "retagged").replace("oamarked", "leftmarked").replace("obmarked", "rightmarked")
                    } else { attribute.to_owned() };
                    let formal = if renamed_names { "Value" } else { "X" };
                    let changed = base.replace(&format!("{formal} is {attribute} means {formal} = {formal}"),
                        &format!("{formal} is {attribute} means not {formal} = {formal}"));
                    assert_ne!(&changed, base);
                    changed
                } else { base.clone() };
                let (source, nodes, symbols) = if multiple {
                    step5c13_registration_inputs(&text)
                } else { step5c3_registration_inputs(&text) }.unwrap();
                let checked = mizar_checker::registration_resolution::check_source_existential_registration_proof(&source, &nodes, &symbols).unwrap();
                let core = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
                let open = step5c3_registration_vcs(&core).unwrap();
                let discharge = try_discharge(DischargeInput { vc_set: &open, policy: &DischargePolicy::default() }).unwrap();
                for (after_discharge, vcs) in [(false, &open), (true, discharge.vc_set())] {
                    for vc in vcs.vcs() {
                        if after_discharge && vc.status != VcStatus::NeedsAtp {
                            assert!(matches!(vc.status, VcStatus::Discharged { .. }));
                            assert!(build_source_existential_kernel_handoff(&core, vcs, vc.id).is_err());
                            continue;
                        }
                        assert_eq!(vc.status, if after_discharge { VcStatus::NeedsAtp } else { VcStatus::Open });
                        let handoff = build_source_existential_kernel_handoff(&core, vcs, vc.id).unwrap();
                        let envelope = handoff.canonical_evidence();
                        let target = envelope.target_vc();
                        let fingerprint = Fingerprint::new(target.algorithm_id, target.digest.clone());
                        let mut parse_context = FormulaEvidenceParseContext::v1(fingerprint.clone(),
                            KernelProfileRecord::v1(envelope.kernel_profile().profile_id, ClauseTautologyPolicy::Reject));
                        let v1 = handoff.encode_local_evidence(None).unwrap();
                        let parsed = parse_formula_evidence(&v1, &parse_context).unwrap();
                        assert_eq!(parsed.schema_version(), 1);
                        assert_eq!(parsed.canonical_hash_input(), v1);
                        assert!(parsed.equality_instances().is_empty());
                        let equalities = parsed.symbol_manifest().iter().filter(|row| row.symbol.kind == SymbolKind::Equality).collect::<Vec<_>>();
                        assert_eq!(equalities.len(), 1);
                        let mut choices = BTreeSet::new();
                        for substitution in parsed.substitutions() {
                            assert_eq!(substitution.payload.payload_kind, 1);
                            assert!(substitution.payload.rewrite_path.segments.is_empty());
                            let [replacement] = substitution.payload.replacements.as_slice() else { panic!("one actual replacement") };
                            let Term::Application { symbol, arguments } = &replacement.actual_term else { panic!("actual closed choice") };
                            assert_eq!(symbol.kind, SymbolKind::FunctorPredicate);
                            assert!(arguments.is_empty());
                            assert_eq!(replacement.replacement_role, 1);
                            choices.insert(*symbol);
                        }
                        let mut names = Vec::new();
                        for (index, choice) in choices.into_iter().enumerate() {
                            let mut item = u32::try_from(index + 1).unwrap().to_be_bytes().to_vec();
                            item.push(1);
                            item.extend(equalities[0].symbol.id.0.to_be_bytes());
                            item.extend([2, 2]);
                            item.extend(choice.id.0.to_be_bytes());
                            item.extend(0u32.to_be_bytes());
                            names.push(item);
                        }
                        let v2 = handoff.encode_local_evidence(Some(&names)).unwrap();
                        assert_eq!(v2, handoff.encode_local_evidence(Some(&names)).unwrap());
                        parse_context.accepted_schema_versions.insert(2);
                        let parsed_v2 = parse_formula_evidence(&v2, &parse_context).unwrap();
                        assert_eq!(parsed_v2.schema_version(), 2);
                        assert_eq!(parsed_v2.canonical_hash_input(), v2);
                        assert_eq!(parsed_v2.equality_instances().len(), names.len());
                        assert_eq!(parsed_v2.symbol_manifest(), parsed.symbol_manifest());
                        assert_eq!(parsed_v2.variable_manifest(), parsed.variable_manifest());
                        assert_eq!(parsed_v2.formulas(), parsed.formulas());
                        assert_eq!(parsed_v2.substitutions(), parsed.substitutions());
                        assert_eq!(parsed_v2.provenance(), parsed.provenance());
                        assert_eq!(parsed_v2.final_goal(), parsed.final_goal());
                        assert_eq!(parsed_v2.target_vc(), &fingerprint);
                        let target = TargetVcFingerprint::new(target.algorithm_id, target.digest.clone());
                        let entries = handoff.context_identity().entries().iter().map(|entry| {
                            let source = match entry.source() {
                                VcSource::LocalHypothesis { local_context_id } => KernelContextIdentitySource::LocalHypothesis { local_context_id },
                                VcSource::CitedPremise { local_context_id } => KernelContextIdentitySource::CitedPremise { local_context_id },
                                VcSource::GeneratedVcFact { vc_fact_id } => KernelContextIdentitySource::GeneratedVcFact { vc_fact_id },
                                _ => panic!("actual local source"),
                            };
                            let producer = match entry.producer_formula_ref() {
                                VcFormulaRef::Core(id) => KernelFormulaProducerRef::Core(id),
                                VcFormulaRef::Generated(id) => KernelFormulaProducerRef::Generated(KernelVcGeneratedFormulaId::new(id.index())),
                                _ => panic!("actual local formula"),
                            };
                            KernelContextIdentityEntry::new(source, entry.formula_id(),
                                Fingerprint::new(entry.formula_fingerprint().algorithm_id, entry.formula_fingerprint().digest.clone()), producer)
                        }).collect();
                        let context = FormulaEvidenceContext::with_context_identity(
                            Some(handoff.context_identity_hash().as_bytes().to_vec()), Vec::new(), Vec::new(),
                            Some(KernelContextIdentityPayload::new(target.clone(), handoff.canonical_hash(), handoff.context_identity_hash(), entries)),
                            ImportedFactContextLimits::default()).unwrap();
                        let result = check_kernel_evidence(KernelEvidenceCheckInput {
                            target_vc_fingerprint: &target, evidence: &parsed_v2, formula_context: Some(&context),
                            check_kind: KernelEvidenceCheckKind::ProofObligation,
                            policy: KernelCheckPolicy::default(), limits: KernelEvidenceCheckLimits::default(),
                        });
                        let registration = vc.id.index() / 2;
                        let expected_accepted = vc.id.index() % 2 == 0
                            || negative.is_none_or(|index| registration != index && registration != 2);
                        assert_eq!(result.status() == KernelCheckStatus::Accepted, expected_accepted,
                            "multiple={multiple}, renamed={renamed_names}, negative={negative:?}, after={after_discharge}, vc={:?}", vc.id);
                        assert_eq!(result.status() == KernelCheckStatus::Accepted,
                            step5c3_check_registration_handoff(&handoff, "clean").is_ok());
                        assert!(!result.policy_taint());
                    }
                }
                let proof = mizar_proof::status::prove_source_existential_registration(&source, &nodes, &symbols,
                    super::super::shared::snapshot_id(0), &mizar_proof::policy::VerifierPolicy::release());
                if negative.is_none() {
                    assert_eq!(proof.unwrap().activated().len(), if multiple { 3 } else { 1 });
                } else { assert!(proof.is_err()); }
            }
        }
    }
}

#[test]
fn source_registration_encoded_producer_binds_real_problem_and_concrete_output() {
    use mizar_atp::{
        backend::EncodedBackendProblem,
        problem::{AtpSymbolSource, ConcreteFormat, EqualitySupport, ExpectedBackendResult,
            LogicFragment, LogicProfile, NativePropertySupport, QuantifierPolicy, SoftTypeStrategy},
        smtlib_encoder::{SmtLibDialect, encode_smtlib},
        tptp_encoder::{TptpDialect, encode_tptp},
        translator::translate_source_existential_registration,
    };
    use mizar_vc::{discharge::{DischargeInput, DischargePolicy, try_discharge}, vc_ir::VcStatus};
    let hex = |bytes: &[u8]| bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    for (multiple, case) in [(false, step5c3_registration_case()), (true, step5c13_registration_case())] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original.replace("wmarked", "retagged").replace("oamarked", "leftmarked")
            .replace("obmarked", "rightmarked").replace("X", "Value");
        for (renamed_names, base) in [(false, &original), (true, &renamed)] {
            for negative in std::iter::once(None).chain((0..if multiple { 2 } else { 1 }).map(Some)) {
                let text = if let Some(index) = negative {
                    let attribute = if multiple { ["oamarked", "obmarked"][index] } else { "wmarked" };
                    let attribute = if renamed_names {
                        attribute.replace("wmarked", "retagged").replace("oamarked", "leftmarked").replace("obmarked", "rightmarked")
                    } else { attribute.to_owned() };
                    let formal = if renamed_names { "Value" } else { "X" };
                    let changed = base.replace(&format!("{formal} is {attribute} means {formal} = {formal}"),
                        &format!("{formal} is {attribute} means not {formal} = {formal}"));
                    assert_ne!(&changed, base);
                    changed
                } else { base.clone() };
                let (source, nodes, symbols) = if multiple {
                    step5c13_registration_inputs(&text)
                } else { step5c3_registration_inputs(&text) }.unwrap();
                let checked = mizar_checker::registration_resolution::check_source_existential_registration_proof(&source, &nodes, &symbols).unwrap();
                let core = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
                let open = step5c3_registration_vcs(&core).unwrap();
                let discharge = try_discharge(DischargeInput { vc_set: &open, policy: &DischargePolicy::default() }).unwrap();
                let vcs = discharge.vc_set();
                for vc in vcs.vcs().iter().filter(|vc| vc.status == VcStatus::NeedsAtp) {
                    for (fragment, format, tag) in [
                        (LogicFragment::Fof, ConcreteFormat::Tptp, "fof"),
                        (LogicFragment::SmtLibUninterpreted, ConcreteFormat::SmtLib, "smtlib-uninterpreted"),
                    ] {
                        let profile = LogicProfile::try_new(format!("actual-source-{tag}"), fragment,
                            EqualitySupport::Supported, QuantifierPolicy::FirstOrder, SoftTypeStrategy::GuardPredicates,
                            NativePropertySupport::Unsupported, [format].into_iter().collect()).unwrap();
                        let problem = translate_source_existential_registration(&core, vcs, vc.id, profile.clone()).unwrap();
                        let encoded = EncodedBackendProblem::encode(&problem, format).unwrap();
                        assert_eq!(encoded, EncodedBackendProblem::encode(&problem, format).unwrap());
                        assert_eq!(encoded.problem_id(), problem.problem_id());
                        assert_eq!(encoded.target_binding(), problem.target_binding());
                        assert_eq!(encoded.expected_result(), ExpectedBackendResult::Unsat);
                        assert_eq!(encoded.concrete_format(), format);
                        assert_eq!(encoded.logic_profile_name(), profile.name().as_str());
                        assert_eq!(encoded.logic_fragment(), tag);
                        let (bytes, labels, rows) = match format {
                            ConcreteFormat::Tptp => {
                                let actual = encode_tptp(&problem, TptpDialect::Fof).unwrap();
                                (actual.text().as_bytes().to_vec(),
                                    actual.formula_labels().iter().map(|row| row.label().to_owned()).collect::<Vec<_>>(),
                                    actual.symbol_bindings().iter().map(|row| (row.atp_symbol().as_str().to_owned(),
                                        row.tptp_name().to_owned(), row.source().clone())).collect::<Vec<_>>())
                            }
                            ConcreteFormat::SmtLib => {
                                let actual = encode_smtlib(&problem, SmtLibDialect::Uninterpreted).unwrap();
                                (actual.text().as_bytes().to_vec(),
                                    actual.assertion_labels().iter().map(|row| row.label().to_owned()).collect::<Vec<_>>(),
                                    actual.symbol_bindings().iter().map(|row| (row.atp_symbol().as_str().to_owned(),
                                        row.smtlib_symbol().to_owned(), row.source().clone())).collect::<Vec<_>>())
                            }
                            _ => panic!("requested concrete format"),
                        };
                        let mut labels = labels;
                        labels.sort();
                        assert_eq!(encoded.input_text(), bytes);
                        assert_eq!(encoded.formula_labels(), labels);
                        let mut mappings = rows.iter().map(|(atp, backend, source)| {
                            let source = match source {
                                AtpSymbolSource::MizarSymbol(binding) => format!("mizar={}:{}", binding.as_str().len(), hex(binding.as_str().as_bytes())),
                                AtpSymbolSource::GeneratedBinder(binding) => format!("binder={}:{}", binding.as_str().len(), hex(binding.as_str().as_bytes())),
                                AtpSymbolSource::TypeGuard(id) => format!("guard={}", id.index()),
                                _ => panic!("actual typed source"),
                            };
                            format!("a={}:{};b={}:{};s={source}", atp.len(), hex(atp.as_bytes()), backend.len(), hex(backend.as_bytes()))
                        }).collect::<Vec<_>>();
                        mappings.sort();
                        assert_eq!(encoded.symbol_bindings(), mappings);
                    }
                }
            }
        }
    }
}

#[test]
fn source_registration_atp_reflexivity_names_preserve_actual_open_and_needs_atp_wire() {
    use mizar_atp::backend::encode_closed_choice_reflexivity_names;
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        clause::{SymbolKind, Term},
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_vc::{
        discharge::{DischargeInput, DischargePolicy, try_discharge},
        kernel_evidence_handoff::build_source_existential_kernel_handoff,
        vc_ir::VcStatus,
    };
    for (multiple, case) in [(false, step5c3_registration_case()), (true, step5c13_registration_case())] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original.replace("wmarked", "retagged").replace("oamarked", "leftmarked")
            .replace("obmarked", "rightmarked").replace("X", "Value");
        for (renamed_names, base) in [(false, &original), (true, &renamed)] {
            for negative in std::iter::once(None).chain((0..if multiple { 2 } else { 1 }).map(Some)) {
                let text = if let Some(index) = negative {
                    let attribute = if multiple { ["oamarked", "obmarked"][index] } else { "wmarked" };
                    let attribute = if renamed_names {
                        attribute.replace("wmarked", "retagged").replace("oamarked", "leftmarked").replace("obmarked", "rightmarked")
                    } else { attribute.to_owned() };
                    let formal = if renamed_names { "Value" } else { "X" };
                    let changed = base.replace(&format!("{formal} is {attribute} means {formal} = {formal}"),
                        &format!("{formal} is {attribute} means not {formal} = {formal}"));
                    assert_ne!(&changed, base);
                    changed
                } else { base.clone() };
                let (source, nodes, symbols) = if multiple {
                    step5c13_registration_inputs(&text)
                } else { step5c3_registration_inputs(&text) }.unwrap();
                let checked = mizar_checker::registration_resolution::check_source_existential_registration_proof(&source, &nodes, &symbols).unwrap();
                let core = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
                let open = step5c3_registration_vcs(&core).unwrap();
                let discharge = try_discharge(DischargeInput { vc_set: &open, policy: &DischargePolicy::default() }).unwrap();
                for (after_discharge, vcs) in [(false, &open), (true, discharge.vc_set())] {
                    for vc in vcs.vcs() {
                        if after_discharge && vc.status != VcStatus::NeedsAtp { continue; }
                        assert_eq!(vc.status, if after_discharge { VcStatus::NeedsAtp } else { VcStatus::Open });
                        let handoff = build_source_existential_kernel_handoff(&core, vcs, vc.id).unwrap();
                        let envelope = handoff.canonical_evidence();
                        let target = envelope.target_vc();
                        let mut context = FormulaEvidenceParseContext::v1(
                            Fingerprint::new(target.algorithm_id, target.digest.clone()),
                            KernelProfileRecord::v1(envelope.kernel_profile().profile_id, ClauseTautologyPolicy::Reject));
                        let v1 = handoff.encode_local_evidence(None).unwrap();
                        let parsed = parse_formula_evidence(&v1, &context).unwrap();
                        let names = encode_closed_choice_reflexivity_names(&parsed).unwrap();
                        assert_eq!(names, encode_closed_choice_reflexivity_names(&parsed).unwrap());
                        assert_eq!(parsed.canonical_hash_input(), v1);
                        let equalities = parsed.symbol_manifest().iter().filter(|row| row.symbol.kind == SymbolKind::Equality).collect::<Vec<_>>();
                        assert_eq!(equalities.len(), 1);
                        let equality = equalities[0].symbol.id.0;
                        let mut choices = parsed.substitutions().iter().map(|row| {
                            let [replacement] = row.payload.replacements.as_slice() else { panic!("actual replacement") };
                            let Term::Application { symbol, arguments } = &replacement.actual_term else { panic!("actual choice") };
                            assert!(arguments.is_empty());
                            assert_eq!(symbol.kind, SymbolKind::FunctorPredicate);
                            symbol.id.0
                        }).collect::<Vec<_>>();
                        choices.sort_unstable();
                        choices.dedup();
                        let expected = choices.iter().enumerate().map(|(index, choice)| format!(
                            "{:08x}01{equality:08x}0202{choice:08x}00000000", index + 1)).collect::<Vec<_>>();
                        let actual = names.iter().map(|item| item.iter().map(|byte| format!("{byte:02x}")).collect::<String>()).collect::<Vec<_>>();
                        assert_eq!(actual, expected);
                        if vc.id.index() % 2 == 0 {
                            assert!(names.is_empty());
                        } else {
                            assert_eq!(names.len(), 1);
                            assert_eq!(parsed.substitutions().len(), if multiple && vc.id.index() == 5 { 2 } else { 1 });
                            if !renamed_names && negative.is_none() {
                                let (equality_id, choice_id, bytes) = if multiple {
                                    match vc.id.index() {
                                        1 => (0xe11c7f8d, 0x28b19da4, "0000000101e11c7f8d020228b19da400000000"),
                                        3 => (0xe11c7f8d, 0x86321e50, "0000000101e11c7f8d020286321e5000000000"),
                                        5 => (0xe11c7f8d, 0x9f34d69e, "0000000101e11c7f8d02029f34d69e00000000"),
                                        _ => panic!("actual attribute leaf"),
                                    }
                                } else { (0x0b38941b, 0x5c51e676, "00000001010b38941b02025c51e67600000000") };
                                assert_eq!(equality, equality_id);
                                assert_eq!(choices, [choice_id]);
                                assert_eq!(actual, [bytes]);
                            }
                        }
                        let v2 = handoff.encode_local_evidence(Some(&names)).unwrap();
                        context.accepted_schema_versions.insert(2);
                        let parsed_v2 = parse_formula_evidence(&v2, &context).unwrap();
                        assert_eq!(parsed_v2.equality_instances().len(), names.len());
                        assert_eq!(parsed_v2.substitutions(), parsed.substitutions());
                        assert!(encode_closed_choice_reflexivity_names(&parsed_v2).is_err());
                        let registration = vc.id.index() / 2;
                        let accepted = vc.id.index() % 2 == 0 || negative.is_none_or(|index| registration != index && registration != 2);
                        assert_eq!(step5c3_check_registration_handoff(&handoff, "clean").is_ok(), accepted);
                    }
                }
                let proof = mizar_proof::status::prove_source_existential_registration(&source, &nodes, &symbols,
                    super::super::shared::snapshot_id(0), &mizar_proof::policy::VerifierPolicy::release());
                if negative.is_none() { assert_eq!(proof.unwrap().activated().len(), if multiple { 3 } else { 1 }); }
                else { assert!(proof.is_err()); }
            }
        }
    }
}

#[test]
fn source_registration_atp_reflexivity_names_reject_parsed_shapes_and_sort_actual_keys() {
    use mizar_atp::backend::encode_closed_choice_reflexivity_names;
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_vc::{
        discharge::{DischargeInput, DischargePolicy, try_discharge},
        kernel_evidence_handoff::build_source_existential_kernel_handoff,
        vc_ir::VcStatus,
    };
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let (source, nodes, symbols) = step5c13_registration_inputs(&text).unwrap();
    let checked = mizar_checker::registration_resolution::check_source_existential_registration_proof(&source, &nodes, &symbols).unwrap();
    let core = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
    let open = step5c3_registration_vcs(&core).unwrap();
    let discharge = try_discharge(DischargeInput { vc_set: &open, policy: &DischargePolicy::default() }).unwrap();
    let vcs = discharge.vc_set();
    let attribute = vcs.vcs().iter().find(|vc| vc.id.index() == 5).unwrap();
    assert_eq!(attribute.status, VcStatus::NeedsAtp);
    // Mutations start from actual authenticated handoffs; only the in-memory wire changes.
    for (vcs, vc) in [(vcs, attribute), (&open, &open.vcs()[0])] {
        let handoff = build_source_existential_kernel_handoff(&core, vcs, vc.id).unwrap();
        let envelope = handoff.canonical_evidence();
        let target = envelope.target_vc();
        let context = FormulaEvidenceParseContext::v1(
            Fingerprint::new(target.algorithm_id, target.digest.clone()),
            KernelProfileRecord::v1(envelope.kernel_profile().profile_id, ClauseTautologyPolicy::Reject));
        let wire = handoff.encode_local_evidence(None).unwrap();
        let read_u32 = |bytes: &[u8]| u32::from_be_bytes(bytes.try_into().unwrap()) as usize;
        let directory = b"MIZAR_KERNEL_EVIDENCE\0".len() + 12 + 5 + target.digest.len() + 4;
        let payload = directory + 6 * 13;
        let sections = (0..6).map(|index| {
            let entry = directory + index * 13;
            let mut cursor = payload + read_u32(&wire[entry + 5..entry + 9]);
            (0..read_u32(&wire[entry + 1..entry + 5])).map(|_| {
                let length = read_u32(&wire[cursor + 2..cursor + 6]);
                let row = wire[cursor + 6..cursor + 6 + length].to_vec();
                cursor += 6 + length;
                row
            }).collect::<Vec<_>>()
        }).collect::<Vec<_>>();
        let encode = |sections: &[Vec<Vec<u8>>]| {
            let mut changed = wire[..directory].to_vec();
            let mut payload = Vec::new();
            for (index, rows) in sections.iter().enumerate() {
                let tag = u8::try_from(index + 1).unwrap();
                let start = payload.len();
                for row in rows {
                    payload.extend([tag, 1]);
                    payload.extend(u32::try_from(row.len()).unwrap().to_be_bytes());
                    payload.extend(row);
                }
                changed.push(tag);
                for value in [rows.len(), start, payload.len() - start] {
                    changed.extend(u32::try_from(value).unwrap().to_be_bytes());
                }
            }
            changed.extend(payload);
            changed
        };
        assert_eq!(encode(&sections), wire);
        if vc.id.index() == 0 {
            let mut missing = sections.clone();
            missing[0].retain(|row| row[0] != 3);
            let parsed = parse_formula_evidence(&encode(&missing), &context).unwrap();
            assert!(encode_closed_choice_reflexivity_names(&parsed).is_err());
            continue;
        }
        assert_eq!(sections[3].len(), 2);
        let mut ambiguous = sections.clone();
        let mut equality = sections[0].iter().find(|row| row[0] == 1).unwrap().clone();
        equality[0] = 3;
        ambiguous[0].push(equality);
        ambiguous[0].sort();
        let parsed = parse_formula_evidence(&encode(&ambiguous), &context).unwrap();
        assert!(encode_closed_choice_reflexivity_names(&parsed).is_err());
        for index in 0..2 {
            for mutation in ["kind", "rewrite", "empty", "two", "role", "variable", "binder", "symbol", "arguments"] {
                let mut changed = sections.clone();
                let row = &mut changed[3][index];
                let offset = 16 + read_u32(&row[12..16]);
                let mut replacement = row[offset..offset + 28].to_vec();
                match mutation {
                    "kind" => replacement[4] = 2,
                    "rewrite" => {
                        replacement[5..9].copy_from_slice(&1u32.to_be_bytes());
                        replacement.splice(9..9, [1, 0, 0, 0, 0]);
                    }
                    "empty" => {
                        replacement[9..13].copy_from_slice(&0u32.to_be_bytes());
                        replacement.drain(13..28);
                    }
                    "two" => {
                        replacement[9..13].copy_from_slice(&2u32.to_be_bytes());
                        let mut second = replacement[13..28].to_vec();
                        second[..4].copy_from_slice(&sections[3][1][..4]);
                        replacement.extend(second);
                    }
                    "role" => replacement[27] = 2,
                    "variable" => { replacement.splice(17..27, std::iter::once(1).chain(sections[1][0].iter().copied())); }
                    "binder" => {
                        let mut term = vec![3];
                        term.extend(&sections[3][index][..4]);
                        term.extend(&replacement[17..27]);
                        replacement.splice(17..27, term);
                    }
                    "symbol" => { replacement[18..23].copy_from_slice(&sections[0][0]); }
                    "arguments" => {
                        replacement[23..27].copy_from_slice(&1u32.to_be_bytes());
                        replacement.splice(27..27, std::iter::once(1).chain(sections[1][0].iter().copied()));
                    }
                    _ => unreachable!(),
                }
                row.splice(offset..offset + 28, replacement);
                let parsed = parse_formula_evidence(&encode(&changed), &context).unwrap_or_else(|error| panic!("{mutation}: {error:?}"));
                assert!(encode_closed_choice_reflexivity_names(&parsed).is_err(), "substitution {index}: {mutation}");
            }
        }
        let choices = sections[0].iter().filter(|row| row[0] == 2).collect::<Vec<_>>();
        assert_eq!(choices.len(), 3);
        for ordering in [[2, 0], [0, 2], [2, 2]] {
            let mut changed = sections.clone();
            for (row, choice) in changed[3].iter_mut().zip(ordering) {
                let offset = 16 + read_u32(&row[12..16]);
                row[offset + 18..offset + 23].copy_from_slice(choices[choice]);
            }
            let parsed = parse_formula_evidence(&encode(&changed), &context).unwrap();
            let names = encode_closed_choice_reflexivity_names(&parsed).unwrap();
            let actual = names.iter().map(|name| name.iter().map(|byte| format!("{byte:02x}")).collect::<String>()).collect::<Vec<_>>();
            let expected = if ordering == [2, 2] {
                vec!["0000000101e11c7f8d02029f34d69e00000000"]
            } else { vec!["0000000101e11c7f8d020228b19da400000000", "0000000201e11c7f8d02029f34d69e00000000"] };
            assert_eq!(actual, expected);
        }
    }
}


#[test]
fn source_registration_candidate_inputs_bind_encoding_metadata_identity_and_replay() {
    use mizar_atp::{backend::{BackendCandidatePayload, EncodedBackendProblem, build_source_registration_candidate_inputs as build},
        problem::ConcreteFormat, translator::translate_source_existential_registration};
    use mizar_vc::{kernel_evidence_handoff::build_source_existential_kernel_handoff, vc_ir::VcStatus};
    let original = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let renamed = original.replace("wmarked", "retagged").replace("X", "Value");
    let mut identities = std::collections::BTreeSet::new();
    for (changed_source, text) in [(false, &original), (true, &renamed)] {
        let (core, _, vcs) = source_candidate_core_and_vcs(false, text);
        let vc = vcs.vcs().iter().find(|vc| vc.status == VcStatus::NeedsAtp).unwrap();
        let handoff = build_source_existential_kernel_handoff(&core, &vcs, vc.id).unwrap();
        let wire = source_candidate_wire(&handoff);
        for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
            for name in ["source-candidate", "source-candidate-renamed-profile"] {
                let profile = source_candidate_profile(name, format);
                let problem = translate_source_existential_registration(&core, &vcs, vc.id, profile.clone()).unwrap();
                let expected = EncodedBackendProblem::encode(&problem, format).unwrap();
                let concrete = match format {
                    ConcreteFormat::Tptp => mizar_atp::tptp_encoder::encode_tptp(&problem, mizar_atp::tptp_encoder::TptpDialect::Fof).unwrap().text().as_bytes().to_vec(),
                    ConcreteFormat::SmtLib => mizar_atp::smtlib_encoder::encode_smtlib(&problem, mizar_atp::smtlib_encoder::SmtLibDialect::Uninterpreted).unwrap().text().as_bytes().to_vec(),
                    _ => panic!("requested concrete format"),
                };
                let pair = build(&core, &vcs, vc.id, profile.clone(), format).unwrap();
                let (encoded, candidate) = &pair;
                assert_eq!(encoded, &expected);
                assert_eq!(encoded.input_text(), concrete);
                assert_eq!(candidate.payload(), &BackendCandidatePayload::FormulaSubstitutionBytes(wire.clone()));
                assert_eq!(candidate.target_binding(), expected.target_binding());
                assert_eq!(candidate.encoded_problem_hash(), expected.input_hash());
                assert_eq!(candidate.provenance_hash(), expected.provenance_hash());
                assert_eq!(candidate.formula_label_refs(), expected.formula_labels());
                assert_eq!(candidate.symbol_binding_refs(), expected.symbol_bindings());
                assert_eq!(pair, build(&core, &vcs, vc.id, profile, format).unwrap());
                if !changed_source && format == ConcreteFormat::Tptp && name == "source-candidate" {
                    // Independently computed from the genuine encoded inputs and normal wire.
                    assert_eq!(candidate.candidate_id(), "source-registration-v1:9e611ca6bdc8d1d9c77f7f719bd972233c36cbb72dd6d00d93b275ab4b8ba245");
                }
                assert!(identities.insert(candidate.candidate_id().to_owned()));
            }
        }
    }
}

#[test]
fn source_registration_candidate_inputs_supply_independent_wire_and_fresh_kernel_outcomes() {
    use mizar_atp::{backend::{BackendCandidatePayload, build_source_registration_candidate_inputs as build}, problem::ConcreteFormat};
    use mizar_kernel::checker::KernelCheckStatus;
    use mizar_vc::{kernel_evidence_handoff::build_source_existential_kernel_handoff, vc_ir::VcStatus};
    for (multiple, case) in [(false, step5c3_registration_case()), (true, step5c13_registration_case())] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        for renamed in [false, true] {
            let base = if renamed { original.replace("marked", "retagged").replace("X", "Value") } else { original.clone() };
            for negative in [false, true] {
                let formal = if renamed { "Value" } else { "X" };
                let text = if negative { base.replacen(&format!("means {formal} = {formal}"), &format!("means not {formal} = {formal}"), 1) } else { base.clone() };
                let (core, _, vcs) = source_candidate_core_and_vcs(multiple, &text);
                for vc in vcs.vcs().iter().filter(|vc| vc.status == VcStatus::NeedsAtp) {
                    let handoff = build_source_existential_kernel_handoff(&core, &vcs, vc.id).unwrap();
                    let (_, candidate) = build(&core, &vcs, vc.id, source_candidate_profile("source-candidate", ConcreteFormat::Tptp), ConcreteFormat::Tptp).unwrap();
                    let BackendCandidatePayload::FormulaSubstitutionBytes(bytes) = candidate.payload() else { panic!("actual bytes") };
                    assert_eq!(bytes, &source_candidate_wire(&handoff));
                    let expected = if negative && vc.id.index() != 3 { KernelCheckStatus::Rejected } else { KernelCheckStatus::Accepted };
                    assert_eq!(source_candidate_check_bytes(&handoff, bytes, candidate.target_binding()).unwrap(), expected,
                        "multiple={multiple}, renamed={renamed}, negative={negative}, vc={:?}", vc.id);
                }
            }
        }
    }
}

#[test]
fn source_registration_candidate_inputs_reject_whole_bindings_and_tampered_transport() {
    use mizar_atp::{backend::{BackendCandidatePayload, build_source_registration_candidate_inputs as build}, problem::*};
    use mizar_vc::{kernel_evidence_handoff::build_source_existential_kernel_handoff, vc_ir::*};
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let (core, open, vcs) = source_candidate_core_and_vcs(true, &text);
    let profile = source_candidate_profile("source-candidate", ConcreteFormat::Tptp);
    let selected = vcs.vcs().iter().find(|vc| vc.status == VcStatus::NeedsAtp).unwrap().id;
    let sibling = vcs.vcs().iter().find(|vc| matches!(vc.status, VcStatus::Discharged { .. })).unwrap().id;
    assert!(build(&core, &open, selected, profile.clone(), ConcreteFormat::Tptp).is_err());
    assert!(build(&core, &vcs, sibling, profile.clone(), ConcreteFormat::Tptp).is_err());
    assert!(build(&core, &vcs, VcId::new(vcs.vcs().len()), profile.clone(), ConcreteFormat::Tptp).is_err());
    let (foreign, _, _) = source_candidate_core_and_vcs(true, &text.replace("marked", "retagged"));
    assert!(build(&foreign, &vcs, selected, profile.clone(), ConcreteFormat::Tptp).is_err());
    for mutation in 0..8 {
        let mut parts = VcSetParts { schema_version: vcs.schema_version().clone(), snapshot: vcs.snapshot(), source: vcs.source(),
            module: vcs.module().clone(), generated_formulas: vcs.generated_formulas().to_vec(),
            vcs: vcs.vcs().to_vec(), seed_accounting: vcs.seed_accounting().to_vec() };
        match mutation {
            0 => parts.vcs[selected.index()].status = VcStatus::Open,
            1 => parts.vcs[selected.index()].status = vcs.vc(sibling).unwrap().status.clone(),
            2 => { parts.vcs[sibling.index()].provenance.pop(); }
            3 => parts.vcs[sibling.index()].status = VcStatus::Open,
            4 => parts.vcs[selected.index()].source.related.clear(),
            5 => { parts.vcs[selected.index()].premises.pop(); }
            6 => parts.vcs[selected.index()].goal = vcs.vc(sibling).unwrap().goal,
            _ => parts.generated_formulas[0].shape = VcGeneratedFormulaShape::True,
        }
        let changed = VcSet::try_new(parts).unwrap();
        assert!(build(&core, &changed, selected, profile.clone(), ConcreteFormat::Tptp).is_err(), "mutation={mutation}");
    }
    assert!(build(&core, &vcs, selected, profile.clone(), ConcreteFormat::SmtLib).is_err());
    let unsupported = LogicProfile::try_new("unsupported", LogicFragment::TffLike, EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder, SoftTypeStrategy::GuardPredicates, NativePropertySupport::Unsupported,
        [ConcreteFormat::Tptp].into_iter().collect()).unwrap();
    assert!(build(&core, &vcs, selected, unsupported, ConcreteFormat::Tptp).is_err());
    let handoff = build_source_existential_kernel_handoff(&core, &vcs, selected).unwrap();
    let (_, candidate) = build(&core, &vcs, selected, profile, ConcreteFormat::Tptp).unwrap();
    let BackendCandidatePayload::FormulaSubstitutionBytes(bytes) = candidate.payload() else { panic!("actual bytes") };
    assert_eq!(bytes, &source_candidate_wire(&handoff));
    let mut corrupted = bytes.clone(); corrupted[0] ^= 1;
    assert!(source_candidate_check_bytes(&handoff, &corrupted, candidate.target_binding()).is_err());
    let sibling_handoff = build_source_existential_kernel_handoff(&core, &vcs, VcId::new(3)).unwrap();
    assert_eq!(source_candidate_check_bytes(&sibling_handoff, bytes, candidate.target_binding()).unwrap(),
        mizar_kernel::checker::KernelCheckStatus::Rejected);
    let mut digest = candidate.target_binding().fingerprint().digest().to_vec(); digest[0] ^= 1;
    let foreign_target = AtpTargetBinding::new(AtpFingerprint::new(candidate.target_binding().fingerprint().algorithm_id(), digest).unwrap(), candidate.target_binding().producer_binding().clone()).unwrap();
    assert!(source_candidate_check_bytes(&handoff, bytes, &foreign_target).is_err());
}

#[test]
fn source_registration_candidate_check_single_uses_fresh_normal_checks_for_both_encodings() {
    use mizar_atp::problem::ConcreteFormat;
    use mizar_kernel::{checker::{KernelCheckStatus, KernelEvidenceCheckKind}, rejection::TargetVcFingerprint};
    use mizar_proof::{policy::{AcceptedGoalPolarity, CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput, VerifierPolicy},
        status::check_source_registration_candidate as check};
    use mizar_vc::{kernel_evidence_handoff::build_source_existential_kernel_handoff, vc_ir::VcStatus};
    let original = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let renamed = original.replace("wmarked", "retagged").replace("X", "Value");
    let mut targets = std::collections::BTreeSet::new();
    for text in [&original, &renamed] {
        let (source, nodes, symbols) = step5c3_registration_inputs(text).unwrap();
        let (core, _, vcs) = source_candidate_core_and_vcs(false, text);
        let vc = vcs.vcs().iter().find(|vc| vc.status == VcStatus::NeedsAtp).unwrap();
        let handoff = build_source_existential_kernel_handoff(&core, &vcs, vc.id).unwrap();
        let target = handoff.canonical_evidence().target_vc();
        let expected = TargetVcFingerprint::new(target.algorithm_id, target.digest.clone());
        assert!(targets.insert(expected.clone()));
        let mut prior = None;
        for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
            let output = check(&source, &nodes, &symbols, super::super::shared::snapshot_id(0), vc.id,
                (source_candidate_profile("source-check", format), format), &VerifierPolicy::release()).unwrap();
            let (result, decision) = &output;
            assert_eq!(result.target_vc_fingerprint(), &expected);
            assert_eq!(result.status(), KernelCheckStatus::Accepted);
            assert_eq!(result.evidence_check_kind(), Some(KernelEvidenceCheckKind::ProofObligation));
            assert!(!result.policy_taint());
            assert!(result.rejections().is_empty());
            assert!(result.sat_check_report().is_some());
            let input = KernelPolicyInput::from_kernel_result(result, KernelEvidenceOrigin::AtpFormulaSubstitution);
            assert_eq!(input.accepted_goal_polarity(), Some(AcceptedGoalPolarity::AssertFalseForRefutation));
            assert_eq!(decision.class, CandidatePolicyClass::KernelVerified);
            assert_eq!(decision.kernel_evidence_check_kind, result.evidence_check_kind());
            assert!(decision.external_admission.is_none());
            if let Some(previous) = prior { assert_eq!(output, previous); }
            prior = Some(output);
        }
    }
}

#[test]
fn source_registration_candidate_check_aggregate_preserves_false_goal_rejection_in_all_modes() {
    use mizar_atp::problem::ConcreteFormat;
    use mizar_kernel::{checker::{KernelCheckStatus, KernelEvidenceCheckKind}, rejection::{RejectionDetail, TargetVcFingerprint}};
    use mizar_proof::{policy::{CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput, VerifierPolicy},
        status::check_source_registration_candidate as check};
    use mizar_vc::{kernel_evidence_handoff::build_source_existential_kernel_handoff, vc_ir::VcStatus};
    let original = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    for renamed in [false, true] {
        let base = if renamed { original.replace("marked", "retagged").replace("X", "Value") } else { original.clone() };
        for negative in [false, true] {
            let formal = if renamed { "Value" } else { "X" };
            let text = if negative { base.replacen(&format!("means {formal} = {formal}"), &format!("means not {formal} = {formal}"), 1) } else { base.clone() };
            let (source, nodes, symbols) = step5c13_registration_inputs(&text).unwrap();
            let (core, _, vcs) = source_candidate_core_and_vcs(true, &text);
            let mut rejections = 0;
            for vc in vcs.vcs().iter().filter(|vc| vc.status == VcStatus::NeedsAtp) {
                let handoff = build_source_existential_kernel_handoff(&core, &vcs, vc.id).unwrap();
                let target = handoff.canonical_evidence().target_vc();
                let expected_target = TargetVcFingerprint::new(target.algorithm_id, target.digest.clone());
                // The second registration cites the unchanged second attribute; the first and conjunction cite the false definition.
                let rejected = negative && vc.id.index() != 3;
                for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
                    for policy in [VerifierPolicy::release(), VerifierPolicy::development(), VerifierPolicy::interactive()] {
                        let (result, decision) = check(&source, &nodes, &symbols, super::super::shared::snapshot_id(0), vc.id,
                            (source_candidate_profile("source-check", format), format), &policy).unwrap();
                        assert_eq!(result.target_vc_fingerprint(), &expected_target);
                        assert_eq!(result.evidence_check_kind(), Some(KernelEvidenceCheckKind::ProofObligation));
                        assert!(!result.policy_taint());
                        assert_eq!(decision.kernel_evidence_check_kind, result.evidence_check_kind());
                        assert_eq!(decision.kernel_rejections, result.rejections());
                        assert!(decision.external_admission.is_none());
                        if rejected {
                            assert_eq!(result.status(), KernelCheckStatus::Rejected);
                            assert_eq!(decision.class, CandidatePolicyClass::KernelRejected);
                            assert!(result.rejections().iter().any(|record| record.detail() == RejectionDetail::InvalidSatRefutation));
                            assert!(KernelPolicyInput::from_kernel_result(&result, KernelEvidenceOrigin::AtpFormulaSubstitution).accepted_goal_polarity().is_none());
                            rejections += 1;
                        } else {
                            assert_eq!(result.status(), KernelCheckStatus::Accepted);
                            assert_eq!(decision.class, CandidatePolicyClass::KernelVerified);
                        }
                    }
                }
            }
            assert_eq!(rejections, if negative { 12 } else { 0 });
        }
    }
}

#[test]
fn source_registration_candidate_check_rejects_foreign_inputs_and_distinguishes_format_permissions() {
    use mizar_atp::problem::*;
    use mizar_proof::{policy::{CandidatePolicyClass, KernelEvidenceFormat, VerifierPolicy},
        status::{check_source_registration_candidate as check, prove_source_existential_registration as builtin}};
    use mizar_vc::vc_ir::{VcId, VcStatus};
    let text = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let (source, nodes, symbols) = step5c3_registration_inputs(&text).unwrap();
    let (_, _, vcs) = source_candidate_core_and_vcs(false, &text);
    let selected = vcs.vcs().iter().find(|vc| vc.status == VcStatus::NeedsAtp).unwrap().id;
    let discharged = vcs.vcs().iter().find(|vc| matches!(vc.status, VcStatus::Discharged { .. })).unwrap().id;
    let format = ConcreteFormat::Tptp;
    let profile = source_candidate_profile("source-check", format);
    let policy = VerifierPolicy::release();
    let snapshot = super::super::shared::snapshot_id(0);
    let foreign_text = text.replace("wmarked", "foreignmarked");
    let (foreign_source, foreign_nodes, foreign_symbols) = step5c3_registration_inputs(&foreign_text).unwrap();
    assert!(check(&foreign_source, &nodes, &symbols, snapshot, selected, (profile.clone(), format), &policy).is_err());
    assert!(check(&source, &foreign_nodes, &symbols, snapshot, selected, (profile.clone(), format), &policy).is_err());
    assert!(check(&source, &nodes, &foreign_symbols, snapshot, selected, (profile.clone(), format), &policy).is_err());
    for target in [discharged, VcId::new(vcs.vcs().len())] {
        assert!(check(&source, &nodes, &symbols, snapshot, target, (profile.clone(), format), &policy).is_err());
    }
    let unsupported_text = text.replacen("take the set;", "take the wmarked set;", 1);
    assert_ne!(unsupported_text, text);
    let (unsupported_source, unsupported_nodes, unsupported_symbols) = step5c3_registration_inputs(&unsupported_text).unwrap();
    assert!(check(&unsupported_source, &unsupported_nodes, &unsupported_symbols, snapshot, selected, (profile.clone(), format), &policy).is_err());
    assert!(check(&source, &nodes, &symbols, snapshot, selected, (profile.clone(), ConcreteFormat::SmtLib), &policy).is_err());
    let unsupported = LogicProfile::try_new("unsupported", LogicFragment::TffLike, EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder, SoftTypeStrategy::GuardPredicates, NativePropertySupport::Unsupported,
        [format].into_iter().collect()).unwrap();
    assert!(check(&source, &nodes, &symbols, snapshot, selected, (unsupported, format), &policy).is_err());
    let builtin_only = policy.clone().with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]);
    assert!(check(&source, &nodes, &symbols, snapshot, selected, (profile.clone(), format), &builtin_only).is_err());
    assert_eq!(builtin(&source, &nodes, &symbols, snapshot, &builtin_only).unwrap().activated().len(), 1);
    let formula_only = policy.with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]);
    let (_, decision) = check(&source, &nodes, &symbols, snapshot, selected, (profile, format), &formula_only).unwrap();
    assert_eq!(decision.class, CandidatePolicyClass::KernelVerified);
    assert!(builtin(&source, &nodes, &symbols, snapshot, &formula_only).is_err());
}

#[test]
fn source_registration_candidate_selection_single_matches_actual_producers_and_selector() {
    use mizar_atp::{backend::build_source_registration_candidate_inputs as build, problem::ConcreteFormat};
    use mizar_proof::{policy::{AcceptedGoalPolarity, CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput, VerifierPolicy},
        selection::{CandidateSourceId, ProofEvidenceCandidate, ProofEvidenceSet, ProofWinnerClass, ProofWitnessPublication, select_winner},
        status::{check_source_registration_candidate as check, select_source_registration_candidate as select}};
    use mizar_vc::vc_ir::VcStatus;
    let original = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let renamed = original.replace("wmarked", "retagged").replace("X", "Value");
    let policy = VerifierPolicy::release();
    let mut identities = std::collections::BTreeSet::new();
    let mut targets = std::collections::BTreeSet::new();
    for text in [&original, &renamed] {
        let (source, nodes, symbols) = step5c3_registration_inputs(text).unwrap();
        let (core, _, vcs) = source_candidate_core_and_vcs(false, text);
        let vc = vcs.vcs().iter().find(|vc| vc.status == VcStatus::NeedsAtp).unwrap();
        for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
            let profile = source_candidate_profile("source-selection", format);
            let (encoded, actual) = build(&core, &vcs, vc.id, profile.clone(), format).unwrap();
            let (result, decision) = check(&source, &nodes, &symbols, super::super::shared::snapshot_id(0), vc.id,
                (profile.clone(), format), &policy).unwrap();
            assert_eq!(decision.class, CandidatePolicyClass::KernelVerified);
            let target = result.target_vc_fingerprint();
            assert_eq!(target.algorithm_id, encoded.target_binding().fingerprint().algorithm_id());
            assert_eq!(target.digest, encoded.target_binding().fingerprint().digest());
            let input = KernelPolicyInput::from_kernel_result(&result, KernelEvidenceOrigin::AtpFormulaSubstitution);
            let candidate = ProofEvidenceCandidate::from_trusted_kernel_input(
                CandidateSourceId::new(actual.candidate_id()).unwrap(), &input).unwrap()
                .with_provenance_hash(actual.provenance_hash());
            let expected = select_winner(&ProofEvidenceSet::new(target.sort_bytes(), encoded.input_hash(), policy.clone())
                .with_candidates([candidate]));
            let selected = select(&source, &nodes, &symbols, super::super::shared::snapshot_id(0), vc.id,
                (profile, format), &policy).unwrap();
            assert_eq!(selected, expected);
            assert_eq!(selected.selected_candidate_id().unwrap().as_str(), actual.candidate_id());
            assert_eq!(selected.selected_class(), ProofWinnerClass::KernelVerified);
            assert!(selected.trusted_used_axioms_available());
            let metadata = selected.reuse_metadata();
            assert_eq!(metadata.encoded_problem_hash(), Some(encoded.input_hash()));
            assert_eq!(metadata.policy_fingerprint(), policy.policy_fingerprint());
            assert_eq!(metadata.selected_candidate_provenance_hash(), Some(actual.provenance_hash()));
            assert_eq!(metadata.selected_evidence_hash(), input.accepted_evidence_hash());
            assert!(metadata.selected_evidence_hash().is_some());
            assert_eq!(metadata.accepted_goal_polarity(), Some(AcceptedGoalPolarity::AssertFalseForRefutation));
            assert_eq!(metadata.selected_proof_witness_hash(), None);
            assert_eq!(metadata.deterministic_discharge_hash(), None);
            assert_eq!(metadata.proof_witness_publication(), ProofWitnessPublication::NotApplicable);
            assert!(identities.insert(actual.candidate_id().to_owned()));
            targets.insert(target.clone());
        }
    }
    assert_eq!(identities.len(), 4);
    assert_eq!(targets.len(), 2);
}

#[test]
fn source_registration_candidate_selection_aggregate_keeps_rejected_leaf_identity() {
    use mizar_atp::{backend::build_source_registration_candidate_inputs as build, problem::ConcreteFormat};
    use mizar_kernel::checker::KernelCheckStatus;
    use mizar_proof::{policy::{CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput, VerifierPolicy},
        selection::{CandidateSourceId, ProofEvidenceCandidate, ProofEvidenceSet, ProofWinnerClass, select_winner},
        status::{check_source_registration_candidate as check, select_source_registration_candidate as select}};
    use mizar_vc::{kernel_evidence_handoff::build_source_existential_kernel_handoff, vc_ir::VcStatus};
    let original = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    for renamed in [false, true] {
        let base = if renamed { original.replace("marked", "retagged").replace("X", "Value") } else { original.clone() };
        for negative in [false, true] {
            let formal = if renamed { "Value" } else { "X" };
            let text = if negative { base.replacen(&format!("means {formal} = {formal}"), &format!("means not {formal} = {formal}"), 1) } else { base.clone() };
            let (source, nodes, symbols) = step5c13_registration_inputs(&text).unwrap();
            let (core, _, vcs) = source_candidate_core_and_vcs(true, &text);
            let mut targets = std::collections::BTreeSet::new();
            let mut accepted = 0;
            let mut rejected = 0;
            for vc in vcs.vcs().iter().filter(|vc| vc.status == VcStatus::NeedsAtp) {
                let handoff = build_source_existential_kernel_handoff(&core, &vcs, vc.id).unwrap();
                let target = handoff.canonical_evidence().target_vc();
                assert!(targets.insert((target.algorithm_id, target.digest.clone())));
                // Only the second registration is independent of the first definition.
                let false_goal = negative && vc.id.index() != 3;
                for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
                    let profile = source_candidate_profile("source-selection", format);
                    let (encoded, actual) = build(&core, &vcs, vc.id, profile.clone(), format).unwrap();
                    for policy in [VerifierPolicy::release(), VerifierPolicy::development(), VerifierPolicy::interactive()] {
                        let (result, decision) = check(&source, &nodes, &symbols, super::super::shared::snapshot_id(0), vc.id,
                            (profile.clone(), format), &policy).unwrap();
                        assert_eq!(result.target_vc_fingerprint().algorithm_id, target.algorithm_id);
                        assert_eq!(result.target_vc_fingerprint().digest, target.digest);
                        let input = KernelPolicyInput::from_kernel_result(&result, KernelEvidenceOrigin::AtpFormulaSubstitution);
                        let id = CandidateSourceId::new(actual.candidate_id()).unwrap();
                        let candidate = if decision.class == CandidatePolicyClass::KernelVerified {
                            ProofEvidenceCandidate::from_trusted_kernel_input(id, &input).unwrap()
                        } else { ProofEvidenceCandidate::new(id, decision.clone()) }
                            .with_provenance_hash(actual.provenance_hash());
                        assert_eq!(candidate.trusted_kernel_evidence().is_some(), !false_goal);
                        let expected = select_winner(&ProofEvidenceSet::new(result.target_vc_fingerprint().sort_bytes(),
                            encoded.input_hash(), policy.clone()).with_candidates([candidate]));
                        let selected = select(&source, &nodes, &symbols, super::super::shared::snapshot_id(0), vc.id,
                            (profile.clone(), format), &policy).unwrap();
                        assert_eq!(selected, expected);
                        assert_eq!(selected.selected_candidate_id().unwrap().as_str(), actual.candidate_id());
                        assert_eq!(selected.reuse_metadata().encoded_problem_hash(), Some(encoded.input_hash()));
                        assert_eq!(selected.reuse_metadata().selected_candidate_provenance_hash(), Some(actual.provenance_hash()));
                        assert_eq!(selected.reuse_metadata().selected_proof_witness_hash(), None);
                        if false_goal {
                            assert_eq!(result.status(), KernelCheckStatus::Rejected);
                            assert_eq!(decision.class, CandidatePolicyClass::KernelRejected);
                            assert_eq!(selected.selected_class(), ProofWinnerClass::Rejected);
                            assert!(!selected.trusted_used_axioms_available());
                            assert_eq!(selected.reuse_metadata().selected_evidence_hash(), None);
                            assert_eq!(selected.reuse_metadata().accepted_goal_polarity(), None);
                            rejected += 1;
                        } else {
                            assert_eq!(result.status(), KernelCheckStatus::Accepted);
                            assert_eq!(selected.selected_class(), ProofWinnerClass::KernelVerified);
                            assert_eq!(selected.reuse_metadata().selected_evidence_hash(), input.accepted_evidence_hash());
                            assert!(selected.trusted_used_axioms_available());
                            accepted += 1;
                        }
                    }
                }
            }
            assert_eq!(targets.len(), 3);
            assert_eq!(accepted, if negative { 6 } else { 18 });
            assert_eq!(rejected, if negative { 12 } else { 0 });
        }
    }
}

#[test]
fn source_registration_candidate_selection_rejects_mixed_inputs_and_current_policy_denials() {
    use mizar_atp::problem::*;
    use mizar_proof::{policy::{KernelEvidenceFormat, VerifierPolicy}, selection::ProofWinnerClass,
        status::{prove_source_existential_registration as builtin, select_source_registration_candidate as select}};
    use mizar_vc::vc_ir::{VcId, VcStatus};
    let text = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let (source, nodes, symbols) = step5c3_registration_inputs(&text).unwrap();
    let (_, _, vcs) = source_candidate_core_and_vcs(false, &text);
    let selected = vcs.vcs().iter().find(|vc| vc.status == VcStatus::NeedsAtp).unwrap().id;
    let discharged = vcs.vcs().iter().find(|vc| matches!(vc.status, VcStatus::Discharged { .. })).unwrap().id;
    let snapshot = super::super::shared::snapshot_id(0);
    let policy = VerifierPolicy::release();
    let format = ConcreteFormat::Tptp;
    let profile = source_candidate_profile("source-selection", format);
    let renamed = text.replace("wmarked", "foreignmarked");
    let (foreign_source, foreign_nodes, foreign_symbols) = step5c3_registration_inputs(&renamed).unwrap();
    assert!(select(&foreign_source, &nodes, &symbols, snapshot, selected, (profile.clone(), format), &policy).is_err());
    assert!(select(&source, &foreign_nodes, &symbols, snapshot, selected, (profile.clone(), format), &policy).is_err());
    assert!(select(&source, &nodes, &foreign_symbols, snapshot, selected, (profile.clone(), format), &policy).is_err());
    for target in [discharged, VcId::new(vcs.vcs().len())] {
        assert!(select(&source, &nodes, &symbols, snapshot, target, (profile.clone(), format), &policy).is_err());
    }
    let unsupported_text = text.replacen("take the set;", "take the wmarked set;", 1);
    assert_ne!(unsupported_text, text);
    let (unsupported_source, unsupported_nodes, unsupported_symbols) = step5c3_registration_inputs(&unsupported_text).unwrap();
    assert!(select(&unsupported_source, &unsupported_nodes, &unsupported_symbols, snapshot, selected, (profile.clone(), format), &policy).is_err());
    assert!(select(&source, &nodes, &symbols, snapshot, selected, (profile.clone(), ConcreteFormat::SmtLib), &policy).is_err());
    let unsupported = LogicProfile::try_new("unsupported", LogicFragment::TffLike, EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder, SoftTypeStrategy::GuardPredicates, NativePropertySupport::Unsupported,
        [format].into_iter().collect()).unwrap();
    assert!(select(&source, &nodes, &symbols, snapshot, selected, (unsupported, format), &policy).is_err());
    let builtin_only = policy.clone().with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]);
    assert!(select(&source, &nodes, &symbols, snapshot, selected, (profile.clone(), format), &builtin_only).is_err());
    assert_eq!(builtin(&source, &nodes, &symbols, snapshot, &builtin_only).unwrap().activated().len(), 1);
    let denied = policy.clone().with_kernel_evidence_formats([]);
    assert!(select(&source, &nodes, &symbols, snapshot, selected, (profile.clone(), format), &denied).is_err());
    let formula_only = policy.with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]);
    let selection = select(&source, &nodes, &symbols, snapshot, selected, (profile, format), &formula_only).unwrap();
    assert_eq!(selection.selected_class(), ProofWinnerClass::KernelVerified);
    assert_eq!(selection.reuse_metadata().policy_fingerprint(), formula_only.policy_fingerprint());
    assert!(builtin(&source, &nodes, &symbols, snapshot, &formula_only).is_err());
}

#[test]
fn source_registration_discharge_check_preserves_original_targets_and_actual_builtin_classification() {
    use mizar_atp::problem::{AtpFingerprint, AtpTargetBinding};
    use mizar_kernel::{checker::{KernelCheckStatus, KernelEvidenceCheckKind}, rejection::TargetVcFingerprint};
    use mizar_proof::{policy::{AcceptedGoalPolarity, CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput,
        PolicyCandidate, ProofPolicyEvaluator, VerifierPolicy}, status::{check_source_registration_discharge_candidate as check,
        prove_source_existential_registration as activate}};
    use mizar_vc::{discharge::{DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_existential_registration,
        kernel_evidence_handoff::build_source_existential_discharge_candidate as build,
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion}};
    for (multiple, case) in [(false, step5c3_registration_case()), (true, step5c13_registration_case())] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original.replace("wmarked", "retagged").replace("oamarked", "leftmarked")
            .replace("obmarked", "rightmarked").replace("X", "Value");
        for (base, formal) in [(&original, "X"), (&renamed, "Value")] {
            let false_definitions = base.replace(&format!("means {formal} = {formal}"), &format!("means not {formal} = {formal}"));
            assert_ne!(&false_definitions, base);
            for (text, negative) in [(base, false), (&false_definitions, true)] {
                let (source, nodes, symbols) = if multiple { step5c13_registration_inputs(text) } else { step5c3_registration_inputs(text) }.unwrap();
                let (core, _, _) = source_candidate_core_and_vcs(multiple, text);
                let mut targets = std::collections::BTreeMap::new();
                for snapshot in [super::super::shared::snapshot_id(0), super::super::shared::snapshot_id(1)] {
                    let open = generate_source_existential_registration(&core, snapshot,
                        &GenerationSchemaVersion::new("source-existential-registration-v1"), &VcSchemaVersion::new("vc-v1")).unwrap();
                    let discharge = try_discharge(DischargeInput { vc_set: &open, policy: &DischargePolicy::default() }).unwrap();
                    assert_eq!(discharge.evidence_records().len(), if multiple { 3 } else { 1 });
                    for record in discharge.evidence_records() {
                        let handoff = build(&core, &open, record.vc, &discharge).unwrap();
                        let fingerprint = handoff.canonical_evidence().target_vc();
                        let expected = TargetVcFingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone());
                        if let Some(zero_target) = targets.insert(record.vc.index(), expected.clone()) { assert_ne!(expected, zero_target); }
                        let target = AtpTargetBinding::new(AtpFingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone()).unwrap(),
                            format!("{:?}", open.vc(record.vc).unwrap().source)).unwrap();
                        let actual_status = source_candidate_check_bytes(&handoff, &handoff.encode_local_evidence(None).unwrap(), &target).unwrap();
                        for policy in [VerifierPolicy::release(), VerifierPolicy::development(), VerifierPolicy::interactive()] {
                            let (result, decision) = check(&source, &nodes, &symbols, snapshot, record.vc, &policy).unwrap();
                            assert_eq!(result.target_vc_fingerprint(), &expected);
                            assert_eq!(result.status(), actual_status);
                            assert_eq!(result.status(), KernelCheckStatus::Accepted);
                            assert_eq!(result.evidence_check_kind(), Some(KernelEvidenceCheckKind::ProofObligation));
                            assert!(!result.policy_taint());
                            assert!(result.used_axioms().is_empty());
                            assert!(result.rejections().is_empty());
                            assert!(result.sat_check_report().is_some());
                            let input = KernelPolicyInput::from_kernel_result(&result, KernelEvidenceOrigin::BuiltinDischarge);
                            assert_eq!(input.accepted_goal_polarity(), Some(AcceptedGoalPolarity::AssertFalseForRefutation));
                            assert_eq!(decision, ProofPolicyEvaluator::new(policy.clone()).evaluate_candidate(&PolicyCandidate::KernelResult(input)));
                            assert_eq!(decision.class, CandidatePolicyClass::DischargedBuiltin);
                            assert_eq!(decision.kernel_evidence_check_kind, result.evidence_check_kind());
                            assert_eq!(decision.kernel_rejections, result.rejections());
                            assert!(decision.external_admission.is_none());
                        }
                    }
                }
                if negative { assert!(activate(&source, &nodes, &symbols, super::super::shared::snapshot_id(0), &VerifierPolicy::release()).is_err()); }
            }
        }
    }
}

#[test]
fn source_registration_discharge_check_rejects_foreign_targets_and_distinguishes_format_permissions() {
    use mizar_atp::problem::ConcreteFormat;
    use mizar_proof::{policy::{CandidatePolicyClass, KernelEvidenceFormat, VerifierPolicy},
        status::{check_source_registration_discharge_candidate as check, check_source_registration_candidate as atp}};
    use mizar_vc::vc_ir::{VcId, VcStatus};
    for (multiple, case) in [(false, step5c3_registration_case()), (true, step5c13_registration_case())] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (source, nodes, symbols) = if multiple { step5c13_registration_inputs(&text) } else { step5c3_registration_inputs(&text) }.unwrap();
        let (_, open, discharged) = source_candidate_core_and_vcs(multiple, &text);
        let selected = discharged.vcs().iter().find(|vc| matches!(vc.status, VcStatus::Discharged { .. })).unwrap().id;
        let correctness = discharged.vcs().iter().find(|vc| vc.status == VcStatus::NeedsAtp).unwrap().id;
        let foreign_text = text.replace("marked", "foreignmarked");
        let (foreign_source, foreign_nodes, foreign_symbols) = if multiple { step5c13_registration_inputs(&foreign_text) }
            else { step5c3_registration_inputs(&foreign_text) }.unwrap();
        let snapshot = super::super::shared::snapshot_id(0);
        let policy = VerifierPolicy::release();
        for (s, n, e) in [(&foreign_source, &nodes, &symbols), (&source, &foreign_nodes, &symbols),
            (&source, &nodes, &foreign_symbols), (&foreign_source, &foreign_nodes, &symbols), (&foreign_source, &nodes, &foreign_symbols)] {
            assert!(check(s, n, e, snapshot, selected, &policy).is_err());
        }
        for target in [correctness, VcId::new(open.vcs().len())] {
            assert!(check(&source, &nodes, &symbols, snapshot, target, &policy).is_err());
        }
        let unsupported_text = text.replacen("take the set;", "take the wmarked set;", 1);
        if !multiple {
            assert_ne!(unsupported_text, text);
            let (s, n, e) = step5c3_registration_inputs(&unsupported_text).unwrap();
            assert!(check(&s, &n, &e, snapshot, selected, &policy).is_err());
        }
        let format = ConcreteFormat::Tptp;
        let profile = source_candidate_profile("discharge-check-permissions", format);
        let builtin_only = policy.clone().with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]);
        assert_eq!(check(&source, &nodes, &symbols, snapshot, selected, &builtin_only).unwrap().1.class, CandidatePolicyClass::DischargedBuiltin);
        assert!(atp(&source, &nodes, &symbols, snapshot, correctness, (profile.clone(), format), &builtin_only).is_err());
        let formula_only = policy.clone().with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]);
        assert!(check(&source, &nodes, &symbols, snapshot, selected, &formula_only).is_err());
        assert_eq!(atp(&source, &nodes, &symbols, snapshot, correctness, (profile.clone(), format), &formula_only).unwrap().1.class, CandidatePolicyClass::KernelVerified);
        let neither = policy.with_kernel_evidence_formats([]);
        assert!(check(&source, &nodes, &symbols, snapshot, selected, &neither).is_err());
        assert!(atp(&source, &nodes, &symbols, snapshot, correctness, (profile, format), &neither).is_err());
    }
}

#[test]
fn source_registration_discharge_results_supply_public_trusted_consumers() {
    use mizar_atp::problem::ConcreteFormat;
    use mizar_kernel::checker::KernelCheckStatus;
    use mizar_proof::{policy::{AcceptedGoalPolarity, CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput, VerifierPolicy},
        selection::{ProofWinnerClass, TrustedKernelEvidence}, status::{TrustedUsedAxiomsRef,
        check_source_registration_discharge_candidate as check, check_source_registration_candidate as atp}};
    use mizar_vc::vc_ir::VcStatus;
    let policy = VerifierPolicy::release();
    for (multiple, case) in [(false, step5c3_registration_case()), (true, step5c13_registration_case())] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original.replace("marked", "retagged").replace("X", "Value");
        let false_definitions = original.replace("means X = X", "means not X = X");
        assert_ne!(false_definitions, original);
        for (text, negative) in [(&original, false), (&renamed, false), (&false_definitions, true)] {
            let (source, nodes, symbols) = if multiple { step5c13_registration_inputs(text) }
                else { step5c3_registration_inputs(text) }.unwrap();
            let (_, _, vcs) = source_candidate_core_and_vcs(multiple, text);
            let direct = vcs.vcs().iter().filter(|vc| matches!(vc.status, VcStatus::Discharged { .. })).collect::<Vec<_>>();
            assert_eq!(direct.len(), if multiple { 3 } else { 1 });
            for vc in direct {
                let mut outputs = Vec::new();
                for snapshot in [super::super::shared::snapshot_id(0), super::super::shared::snapshot_id(1)] {
                    let (result, decision) = check(&source, &nodes, &symbols, snapshot, vc.id, &policy).unwrap();
                    assert_eq!(result.status(), KernelCheckStatus::Accepted);
                    let input = KernelPolicyInput::from_kernel_result(&result, KernelEvidenceOrigin::BuiltinDischarge);
                    let trusted = TrustedKernelEvidence::from_policy_input(&input).unwrap();
                    let used = TrustedUsedAxiomsRef::from_kernel_result(&result, KernelEvidenceOrigin::BuiltinDischarge).unwrap();
                    assert_eq!(input.origin(), KernelEvidenceOrigin::BuiltinDischarge);
                    assert_eq!(decision.class, CandidatePolicyClass::DischargedBuiltin);
                    assert_eq!(trusted.selected_class(), ProofWinnerClass::DischargedBuiltin);
                    assert_eq!(input.accepted_evidence_hash(), Some(trusted.accepted_evidence_hash()));
                    assert_eq!(input.accepted_evidence_hash(), Some(used.accepted_evidence_hash()));
                    assert_eq!(input.accepted_goal_polarity(), Some(AcceptedGoalPolarity::AssertFalseForRefutation));
                    assert_eq!(trusted.accepted_goal_polarity(), AcceptedGoalPolarity::AssertFalseForRefutation);
                    assert!(trusted.trusted_used_axioms_available());
                    assert_eq!(used.used_axiom_count(), result.used_axioms().len());
                    assert!(result.used_axioms().is_empty());
                    outputs.push((result, used));
                }
                let [first, second] = outputs.as_slice() else { panic!("two actual snapshots") };
                assert_ne!(first.0.target_vc_fingerprint(), second.0.target_vc_fingerprint());
                assert_ne!(first.1.accepted_evidence_hash(), second.1.accepted_evidence_hash());
                assert_eq!(first.0.used_axioms(), second.0.used_axioms());
                assert_eq!(first.1.used_axioms_hash(), second.1.used_axioms_hash());
            }
            if negative {
                let correctness = vcs.vcs().iter().find(|vc| vc.status == VcStatus::NeedsAtp).unwrap().id;
                let format = ConcreteFormat::Tptp;
                let (result, decision) = atp(&source, &nodes, &symbols, super::super::shared::snapshot_id(0), correctness,
                    (source_candidate_profile("source-trusted-consumers", format), format), &policy).unwrap();
                assert_eq!(result.status(), KernelCheckStatus::Rejected);
                assert_eq!(decision.class, CandidatePolicyClass::KernelRejected);
                let input = KernelPolicyInput::from_kernel_result(&result, KernelEvidenceOrigin::AtpFormulaSubstitution);
                assert_eq!(input.origin(), KernelEvidenceOrigin::AtpFormulaSubstitution);
                assert!(TrustedKernelEvidence::from_policy_input(&input).is_none());
                assert!(TrustedUsedAxiomsRef::from_kernel_result(&result, KernelEvidenceOrigin::AtpFormulaSubstitution).is_err());
            }
        }
    }
}

#[test]
fn source_registration_local_ranking_preserves_actual_input_and_encodings() {
    use mizar_atp::{
        problem::*,
        smtlib_encoder::{SmtLibDialect, encode_smtlib},
        tptp_encoder::{TptpDialect, encode_tptp},
        translator::translate_source_existential_registration as translate,
    };
    use mizar_vc::{kernel_evidence_handoff::build_source_existential_kernel_handoff, vc_ir::*};
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original.replace("marked", "retagged").replace("X", "Value");
        let false_definitions = original.replace("means X = X", "means not X = X");
        assert_ne!(false_definitions, original);
        for text in [&original, &renamed, &false_definitions] {
            let (core, _, vcs) = source_candidate_core_and_vcs(multiple, text);
            let selected = vcs
                .vcs()
                .iter()
                .filter(|vc| vc.status == VcStatus::NeedsAtp)
                .collect::<Vec<_>>();
            assert_eq!(selected.len(), if multiple { 3 } else { 1 });
            for (registration, vc) in selected.iter().enumerate() {
                let handoff = build_source_existential_kernel_handoff(&core, &vcs, vc.id).unwrap();
                for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
                    let profile = source_candidate_profile("source-local-ranking", format);
                    let problem = translate(&core, &vcs, vc.id, profile.clone()).unwrap();
                    let before = problem.clone();
                    let encoded_before = if format == ConcreteFormat::Tptp {
                        encode_tptp(&problem, TptpDialect::Fof)
                            .unwrap()
                            .text()
                            .to_owned()
                    } else {
                        encode_smtlib(&problem, SmtLibDialect::Uninterpreted)
                            .unwrap()
                            .text()
                            .to_owned()
                    };
                    let (configuration, ranked) = problem.rank_axioms().unwrap();
                    assert_eq!(configuration, "symbol-overlap-v1");
                    let expected = if registration == 2 {
                        vec![0, 1, 2]
                    } else {
                        vec![0, 1]
                    };
                    assert_eq!(
                        ranked
                            .iter()
                            .map(|row| row.id().index())
                            .collect::<Vec<_>>(),
                        expected
                    );
                    assert_eq!(
                        (configuration, ranked.clone()),
                        problem.rank_axioms().unwrap()
                    );
                    assert_eq!(ranked.len(), problem.axioms().len());
                    let mut premises = vc.premises.clone();
                    premises.sort();
                    assert_eq!(ranked.len(), premises.len());
                    for (row, premise) in ranked.iter().zip(premises) {
                        let original = problem
                            .axioms()
                            .iter()
                            .find(|original| original.id() == row.id())
                            .unwrap();
                        assert!(std::ptr::eq(*row, original));
                        let PremiseRef::GeneratedFact {
                            formula: VcFormulaRef::Generated(id),
                        } = premise
                        else {
                            panic!("actual generated premise")
                        };
                        let entry = handoff
                            .canonical_evidence()
                            .formula_evidence()
                            .iter()
                            .find(|entry| {
                                entry.producer_formula_ref() == Some(VcFormulaRef::Generated(id))
                            })
                            .unwrap();
                        let anchor = handoff
                            .canonical_evidence()
                            .provenance()
                            .iter()
                            .find(|anchor| anchor.provenance_id == entry.provenance_id())
                            .unwrap();
                        let provenance = &problem.provenance()[row.provenance().index() as usize];
                        assert_eq!(
                            provenance.source(),
                            &AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new(format!(
                                "generated:{}",
                                id.index() + 1
                            )))
                        );
                        let payload = anchor
                            .payload
                            .iter()
                            .map(|byte| format!("{byte:02x}"))
                            .collect::<String>();
                        assert_eq!(
                            provenance.payload().as_str(),
                            format!("mizar-vc-handoff-provenance:{payload}")
                        );
                    }
                    assert_eq!(problem.vc_id(), vc.id);
                    assert_eq!(problem.logic_profile(), &profile);
                    let target = handoff.canonical_evidence().target_vc();
                    assert_eq!(
                        problem.target_binding().fingerprint(),
                        &AtpFingerprint::new(target.algorithm_id, target.digest.clone()).unwrap()
                    );
                    assert_eq!(problem, before);
                    let encoded_after = if format == ConcreteFormat::Tptp {
                        encode_tptp(&problem, TptpDialect::Fof)
                            .unwrap()
                            .text()
                            .to_owned()
                    } else {
                        encode_smtlib(&problem, SmtLibDialect::Uninterpreted)
                            .unwrap()
                            .text()
                            .to_owned()
                    };
                    assert_eq!(encoded_before, encoded_after);
                }
            }
        }
    }
}

#[test]
fn source_registration_discharge_metadata_binds_same_call_original_handoff_and_policy() {
    use mizar_artifact::{registration_summary::ArtifactHashClass, store::SchemaVersion};
    use mizar_atp::problem::{AtpFingerprint, AtpTargetBinding};
    use mizar_kernel::{
        checker::{KernelCheckStatus, KernelEvidenceCheckKind},
        rejection::TargetVcFingerprint,
    };
    use mizar_proof::{
        policy::{
            AcceptedGoalPolarity, CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput,
            PolicyCandidate, ProofPolicyEvaluator, VerifierPolicy,
        },
        status::{
            check_source_registration_discharge_candidate as old,
            check_source_registration_discharge_metadata as check,
        },
    };
    use mizar_vc::{
        discharge::{DischargeInput, DischargePolicy, DischargeRule, try_discharge},
        generator::generate_source_existential_registration,
        kernel_evidence_handoff::{
            KernelFormulaSource, build_source_existential_discharge_candidate,
            build_source_existential_kernel_handoff,
        },
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion},
    };
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original
            .replace("wmarked", "retagged")
            .replace("oamarked", "leftmarked")
            .replace("obmarked", "rightmarked")
            .replace("X", "Value");
        let false_definitions = original.replace("means X = X", "means not X = X");
        assert_ne!(false_definitions, original);
        let mut seen = std::collections::BTreeMap::new();
        for (variant, text) in [&original, &renamed, &false_definitions]
            .into_iter()
            .enumerate()
        {
            let (source, nodes, symbols) = if multiple {
                step5c13_registration_inputs(text)
            } else {
                step5c3_registration_inputs(text)
            }
            .unwrap();
            let before = (source.clone(), nodes.clone(), symbols.clone());
            let (core, _, _) = source_candidate_core_and_vcs(multiple, text);
            for snapshot_index in 0..2 {
                let snapshot = super::super::shared::snapshot_id(snapshot_index);
                let open = generate_source_existential_registration(
                    &core,
                    snapshot,
                    &GenerationSchemaVersion::new("source-existential-registration-v1"),
                    &VcSchemaVersion::new("vc-v1"),
                )
                .unwrap();
                let discharge = try_discharge(DischargeInput {
                    vc_set: &open,
                    policy: &DischargePolicy::default(),
                })
                .unwrap();
                assert_eq!(
                    discharge.evidence_records().len(),
                    if multiple { 3 } else { 1 }
                );
                for record in discharge.evidence_records() {
                    assert_eq!(record.rule, Some(DischargeRule::DirectLocalFact));
                    let handoff =
                        build_source_existential_kernel_handoff(&core, &open, record.vc).unwrap();
                    assert_eq!(
                        handoff,
                        build_source_existential_discharge_candidate(
                            &core, &open, record.vc, &discharge
                        )
                        .unwrap()
                    );
                    let envelope = handoff.canonical_evidence();
                    assert!(envelope.substitutions().is_empty());
                    assert_eq!(envelope.formula_evidence().len(), 1);
                    assert!(envelope.formula_evidence().iter().all(|formula| matches!(
                        formula.source(),
                        KernelFormulaSource::GeneratedVcFact { .. }
                    )));
                    let target = envelope.target_vc();
                    let binding = AtpTargetBinding::new(
                        AtpFingerprint::new(target.algorithm_id, target.digest.clone()).unwrap(),
                        format!("{:?}", open.vc(record.vc).unwrap().source),
                    )
                    .unwrap();
                    let wire = handoff.encode_local_evidence(None).unwrap();
                    assert_eq!(
                        source_candidate_check_bytes(&handoff, &wire, &binding).unwrap(),
                        KernelCheckStatus::Accepted
                    );
                    let components = handoff.local_evidence_component_hashes().unwrap();
                    for (policy_index, policy) in [
                        VerifierPolicy::release(),
                        VerifierPolicy::development(),
                        VerifierPolicy::interactive(),
                        VerifierPolicy::release().with_schema_version(2),
                        VerifierPolicy::release().with_checker_schema_version(7),
                        VerifierPolicy::release().with_profile_id("configured-metadata-profile"),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let (result, decision, metadata) =
                            check(&source, &nodes, &symbols, snapshot, record.vc, &policy).unwrap();
                        assert_eq!(
                            old(&source, &nodes, &symbols, snapshot, record.vc, &policy).unwrap(),
                            (result.clone(), decision.clone())
                        );
                        assert_eq!(
                            check(&source, &nodes, &symbols, snapshot, record.vc, &policy).unwrap(),
                            (result.clone(), decision.clone(), metadata.clone())
                        );
                        assert_eq!(result.status(), KernelCheckStatus::Accepted);
                        assert_eq!(
                            result.target_vc_fingerprint(),
                            &TargetVcFingerprint::new(target.algorithm_id, target.digest.clone())
                        );
                        assert_eq!(
                            result.evidence_check_kind(),
                            Some(KernelEvidenceCheckKind::ProofObligation)
                        );
                        assert!(!result.policy_taint());
                        assert!(result.used_axioms().is_empty());
                        assert!(result.checked_imports().is_empty());
                        assert!(result.checked_substitutions().is_empty());
                        let input = KernelPolicyInput::from_kernel_result(
                            &result,
                            KernelEvidenceOrigin::BuiltinDischarge,
                        );
                        assert_eq!(
                            input.accepted_goal_polarity(),
                            Some(AcceptedGoalPolarity::AssertFalseForRefutation)
                        );
                        assert_eq!(
                            decision,
                            ProofPolicyEvaluator::new(policy.clone())
                                .evaluate_candidate(&PolicyCandidate::KernelResult(input.clone()))
                        );
                        assert_eq!(decision.class, CandidatePolicyClass::DischargedBuiltin);
                        let metadata = metadata.unwrap();
                        assert_eq!(
                            metadata.checker_schema_version,
                            SchemaVersion::new(policy.checker_schema_version(), 0)
                        );
                        assert_eq!(
                            metadata.evidence_schema_version,
                            SchemaVersion::new(envelope.schema_version(), 0)
                        );
                        assert!(metadata.formula_context_hash.is_none());
                        for (reference, family, version, digest) in [
                            (
                                &metadata.kernel_profile_fingerprint,
                                "mizar-vc/local-evidence-component/profile",
                                SchemaVersion::new(1, 0),
                                components[0],
                            ),
                            (
                                &metadata.formula_evidence_hash,
                                "mizar-vc/local-evidence-component/formula",
                                SchemaVersion::new(1, 0),
                                components[1],
                            ),
                            (
                                &metadata.substitution_evidence_hash,
                                "mizar-vc/local-evidence-component/substitutions",
                                SchemaVersion::new(1, 0),
                                components[2],
                            ),
                            (
                                &metadata.provenance_hash,
                                "mizar-vc/local-evidence-component/provenance",
                                SchemaVersion::new(1, 0),
                                components[3],
                            ),
                            (
                                &metadata.target_binding_hash,
                                "mizar-vc/kernel-evidence-handoff-canonical",
                                SchemaVersion::new(1, 0),
                                handoff.canonical_hash(),
                            ),
                            (
                                &metadata.verifier_policy_fingerprint,
                                "mizar-proof/verifier-policy",
                                SchemaVersion::new(policy.schema_version(), 0),
                                policy.policy_fingerprint().hash(),
                            ),
                            (
                                &metadata.accepted_result_hash,
                                "mizar-proof/kernel-accepted-evidence",
                                SchemaVersion::new(1, 0),
                                input.accepted_evidence_hash().unwrap(),
                            ),
                        ] {
                            assert_eq!(reference.class, ArtifactHashClass::Interface);
                            assert_eq!(reference.schema_family, family);
                            assert_eq!(reference.schema_version, version);
                            assert_eq!(reference.digest, digest);
                        }
                        if snapshot_index == 1 {
                            let prior: &mizar_artifact::proof_witness::KernelAcceptanceMetadata =
                                &seen[&(variant, 0, record.vc.index(), policy_index)];
                            assert_eq!(
                                metadata.kernel_profile_fingerprint,
                                prior.kernel_profile_fingerprint
                            );
                            assert_eq!(metadata.formula_evidence_hash, prior.formula_evidence_hash);
                            assert_eq!(
                                metadata.substitution_evidence_hash,
                                prior.substitution_evidence_hash
                            );
                            assert_ne!(metadata.provenance_hash, prior.provenance_hash);
                            assert_ne!(metadata.target_binding_hash, prior.target_binding_hash);
                            assert_ne!(metadata.accepted_result_hash, prior.accepted_result_hash);
                        }
                        if variant == 1 {
                            let prior =
                                &seen[&(0, snapshot_index, record.vc.index(), policy_index)];
                            assert_ne!(metadata.provenance_hash, prior.provenance_hash);
                            assert_ne!(metadata.target_binding_hash, prior.target_binding_hash);
                            assert_ne!(metadata.accepted_result_hash, prior.accepted_result_hash);
                        }
                        if policy_index > 0 {
                            let prior = &seen[&(variant, snapshot_index, record.vc.index(), 0)];
                            assert_ne!(
                                metadata.verifier_policy_fingerprint,
                                prior.verifier_policy_fingerprint
                            );
                            assert_eq!(metadata.target_binding_hash, prior.target_binding_hash);
                            assert_eq!(metadata.accepted_result_hash, prior.accepted_result_hash);
                        }
                        seen.insert(
                            (variant, snapshot_index, record.vc.index(), policy_index),
                            metadata,
                        );
                    }
                }
            }
            assert_eq!((source, nodes, symbols), before);
        }
    }
}

#[cfg(test)]
mod premise_ranking_suggestions {
    use super::*;
    use mizar_atp::{
        backend::{
            BackendCancellationToken, BackendCandidateEvidence, BackendCandidatePayload,
            BackendCommand, BackendIoMode, BackendKind, BackendObservation, BackendObservedResult,
            BackendProfile, BackendProfileId, BackendResourceLimits, BackendRunId, BackendRunInput,
            BackendRunStatus, EncodedBackendProblem, classify_backend_observation, run_backend,
        },
        problem::{
            AtpFormula, AtpPayload, AtpProblem, AtpProblemParts, AtpProvenance, ConcreteFormat,
        },
        translator::translate_source_existential_registration,
    };
    use mizar_vc::vc_ir::VcStatus;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static RUN_ID: AtomicUsize = AtomicUsize::new(0);

    fn registered_problem(multiple: bool, format: ConcreteFormat) -> AtpProblem {
        let case = if multiple {
            step5c13_registration_case()
        } else {
            step5c3_registration_case()
        };
        let text = std::fs::read_to_string(case.source_path).unwrap();
        registered_problem_from_text(multiple, &text, format)
    }

    fn registered_problem_from_text(
        multiple: bool,
        text: &str,
        format: ConcreteFormat,
    ) -> AtpProblem {
        let (core, _, vcs) = source_candidate_core_and_vcs(multiple, text);
        let vc = vcs
            .vcs()
            .iter()
            .find(|vc| vc.status == VcStatus::NeedsAtp)
            .unwrap();
        translate_source_existential_registration(
            &core,
            &vcs,
            vc.id,
            source_candidate_profile("premise-ranking-suggestions", format),
        )
        .unwrap()
    }

    fn attempted(original: &AtpProblem, rows: Vec<AtpFormula>) -> AtpProblem {
        AtpProblem::try_new(AtpProblemParts {
            vc_id: original.vc_id(),
            target_binding: original.target_binding().clone(),
            logic_profile: original.logic_profile().clone(),
            expected_result: original.expected_result(),
            declarations: original.declarations().to_vec(),
            axioms: rows,
            conjecture: original.conjecture().clone(),
            type_context: original.type_context().clone(),
            properties: original.properties().to_vec(),
            symbol_map: original.symbol_map().to_vec(),
            provenance: original.provenance().to_vec(),
            diagnostics: original.diagnostics().to_vec(),
        })
        .unwrap()
    }

    fn retry_problem(original: &AtpProblem, subset: &AtpProblem) -> AtpProblem {
        let (_, ranked) = original.rank_axioms().unwrap();
        let next = ranked
            .into_iter()
            .find(|row| !subset.axioms().contains(row))
            .unwrap();
        let mut axioms = subset.axioms().to_vec();
        axioms.push(next.clone());
        attempted(original, axioms)
    }

    fn backend_result(
        problem: &AtpProblem,
        format: ConcreteFormat,
        failed: bool,
    ) -> mizar_atp::backend::BackendRunResult {
        let encoded = EncodedBackendProblem::encode(problem, format).unwrap();
        let sequence = RUN_ID.fetch_add(1, Ordering::Relaxed);
        let command = BackendCommand::new(
            PathBuf::from("/bin/sh"),
            vec![
                "-c".to_owned(),
                if failed {
                    "cat >/dev/null; exit 1"
                } else {
                    "cat >/dev/null; exit 0"
                }
                .to_owned(),
            ],
        )
        .unwrap();
        let input = BackendRunInput::new(
            BackendRunId::new(format!("premise-ranking-{sequence}")).unwrap(),
            encoded,
            BackendProfile::new(
                BackendProfileId::new("premise-ranking-test").unwrap(),
                BackendKind::new("mock-shell").unwrap(),
                format,
            ),
            command,
            BackendResourceLimits::new(),
            BackendIoMode::Stdin,
            BackendCancellationToken::new(),
        );
        let result = run_backend(input);
        assert_eq!(
            result.status(),
            if failed {
                BackendRunStatus::Error
            } else {
                BackendRunStatus::Unknown
            }
        );
        if failed {
            return result;
        }
        let encoded = result.encoded_problem();
        let candidate = BackendCandidateEvidence::new(
            "ranking-test-candidate",
            BackendCandidatePayload::FormulaSubstitutionBytes(b"source-candidate".to_vec()),
            encoded.target_binding().clone(),
            encoded.input_hash(),
            encoded.provenance_hash(),
            encoded.formula_labels().to_vec(),
            encoded.symbol_bindings().to_vec(),
        )
        .unwrap();
        classify_backend_observation(
            result,
            BackendObservation::new(BackendObservedResult::Unsat)
                .with_candidate_evidence(candidate),
        )
    }

    #[test]
    fn suggestions_without_retry_and_failed_retry_are_ranked_for_both_encodings() {
        for multiple in [false, true] {
            for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
                let original = registered_problem(multiple, format);
                let (_, ranking) = original.rank_axioms().unwrap();
                let attempted = attempted(&original, vec![ranking[0].clone()]);
                let (configuration, suggested, resolving) = original
                    .suggest_axioms(&attempted, "symbol-overlap-v1", None, None)
                    .unwrap();
                assert_eq!(configuration, "symbol-overlap-v1");
                assert_eq!(suggested, ranking[1..].to_vec());
                assert!(resolving.is_none());

                let retried = retry_problem(&original, &attempted);
                let failed = backend_result(&retried, format, true);
                let (_, still_suggested, no_resolution) = original
                    .suggest_axioms(&attempted, "symbol-overlap-v1", None, Some(&failed))
                    .unwrap();
                assert_eq!(still_suggested, suggested);
                assert!(no_resolution.is_none());
            }
        }
    }

    #[test]
    fn successful_retry_classifies_first_ranked_absent_candidate_without_changing_source_problem() {
        for multiple in [false, true] {
            for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
                let original = registered_problem(multiple, format);
                let (_, ranking) = original.rank_axioms().unwrap();
                let attempted = attempted(&original, vec![ranking[0].clone()]);
                let retried = retry_problem(&original, &attempted);
                let proved = backend_result(&retried, format, false);
                assert_eq!(proved.status(), BackendRunStatus::Proved);
                let (configuration, suggested, resolving) = original
                    .suggest_axioms(&attempted, "symbol-overlap-v1", None, Some(&proved))
                    .unwrap();
                assert_eq!(configuration, "symbol-overlap-v1");
                assert_eq!(suggested, ranking[1..].to_vec());
                assert_eq!(resolving, Some(ranking[1]));
                // Reclassification replaces observations even when the Proved status remains.
                let inconsistent = classify_backend_observation(
                    proved,
                    BackendObservation::new(BackendObservedResult::Sat),
                );
                assert_eq!(inconsistent.status(), BackendRunStatus::Proved);
                assert!(original
                    .suggest_axioms(&attempted, "symbol-overlap-v1", None, Some(&inconsistent))
                    .is_err());
            }
        }
    }

    #[test]
    fn altered_candidate_identity_provenance_or_configuration_fails_closed() {
        let original = registered_problem(true, ConcreteFormat::Tptp);
        let (_, ranked) = original.rank_axioms().unwrap();
        let base = attempted(&original, vec![ranked[0].clone()]);

        let mut changed_id = base.axioms().to_vec();
        let row = &changed_id[0];
        changed_id[0] = AtpFormula::new(
            mizar_atp::problem::AtpFormulaId::new(row.id().index() + 1000),
            row.formula().unwrap().clone(),
            row.provenance(),
        );
        let bad_identity = attempted(&original, changed_id);
        assert!(
            original
                .suggest_axioms(&bad_identity, "symbol-overlap-v1", None, None)
                .is_err()
        );

        let mut provenance = original.provenance().to_vec();
        let first = provenance[0].clone();
        provenance[0] = AtpProvenance::new(
            first.id(),
            first.source().clone(),
            AtpPayload::new("tampered-payload"),
        );
        let bad_provenance = AtpProblem::try_new(AtpProblemParts {
            vc_id: base.vc_id(),
            target_binding: base.target_binding().clone(),
            logic_profile: base.logic_profile().clone(),
            expected_result: base.expected_result(),
            declarations: base.declarations().to_vec(),
            axioms: base.axioms().to_vec(),
            conjecture: base.conjecture().clone(),
            type_context: base.type_context().clone(),
            properties: base.properties().to_vec(),
            symbol_map: base.symbol_map().to_vec(),
            provenance,
            diagnostics: base.diagnostics().to_vec(),
        })
        .unwrap();
        assert!(
            original
                .suggest_axioms(&bad_provenance, "symbol-overlap-v1", None, None)
                .is_err()
        );
        assert!(
            original
                .suggest_axioms(&base, "unknown-ranking", None, None)
                .is_err()
        );
    }

    #[test]
    fn suggestions_do_not_change_source_proof_acceptance_inputs() {
        use mizar_checker::registration_resolution::{
            check_source_existential_registration_proof, check_source_registration_intake,
        };
        use mizar_session::SourceAnchor;

        let source = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
        let registration_end = source
            .find("\ndefinition\n")
            .expect("following theorem definition");
        let cited = &source[..registration_end];
        let uncited = cited.replace(" by WMDef", "         ");
        let (cited_source, cited_nodes, cited_symbols) =
            step5c3_registration_inputs(cited).unwrap();
        let (source, nodes, symbols) = step5c3_registration_inputs(&uncited).unwrap();
        let cited_intake =
            check_source_registration_intake(&cited_source, &cited_nodes, &cited_symbols).unwrap();
        let intake = check_source_registration_intake(&source, &nodes, &symbols).unwrap();
        let [cited_registration] = cited_intake.validations() else {
            panic!("one authenticated registration")
        };
        let [registration] = intake.validations() else {
            panic!("one authenticated registration")
        };
        assert_eq!(registration.source_range(), cited_registration.source_range());
        assert_eq!(registration.pattern(), cited_registration.pattern());
        assert_eq!(registration.parameters(), cited_registration.parameters());
        assert_eq!(registration.assumptions(), cited_registration.assumptions());
        assert_eq!(
            registration.referenced_symbols(),
            cited_registration.referenced_symbols()
        );
        // Preserve the checked definitions, registration target and conclusion. Token
        // removal shifts composite arena IDs, so compare anchored nodes and child edges.
        // Uncited proofs stop before the normalized ATP producer.
        assert_eq!(source.module(), cited_source.module());
        let citation_start = cited.find(" by WMDef").expect("source citation");
        let before_citation = |arena: &mizar_resolve::resolved_ast::SurfaceResolvedArena| {
            arena
                .arena()
                .iter()
                .filter(|(_, node)| {
                    matches!(node.origin().anchor(), SourceAnchor::Range(range)
                        if range.end <= citation_start)
                })
                .map(|(_, node)| {
                    (
                        node.kind().clone(),
                        node.origin().anchor().clone(),
                        node.recovery(),
                        node.resolution(),
                        node.reference_key(),
                        node.children()
                            .iter()
                            .map(|id| {
                                let child = arena.arena().node(*id).unwrap();
                                (child.kind().clone(), child.origin().anchor().clone())
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>()
        };
        let checked_prefix = before_citation(&cited_source);
        assert!(!checked_prefix.is_empty());
        assert_eq!(before_citation(&source), checked_prefix);
        assert_eq!(
            check_source_existential_registration_proof(&source, &nodes, &symbols)
                .unwrap_err(),
            "registration.source_existential_proof_invalid"
        );

        let original = registered_problem_from_text(false, cited, ConcreteFormat::Tptp);
        let attempted = attempted(&original, Vec::new());
        let (_, suggestions, _) = original
            .suggest_axioms(&attempted, "symbol-overlap-v1", None, None)
            .unwrap();
        let suggested = *suggestions.first().expect("a ranked absent candidate");
        let retried = retry_problem(&original, &attempted);
        let proved_retry = backend_result(&retried, ConcreteFormat::Tptp, false);
        let (_, suggestions_after, resolving) = original
            .suggest_axioms(&attempted, "symbol-overlap-v1", None, Some(&proved_retry))
            .unwrap();
        assert_eq!(suggestions_after, suggestions);
        assert_eq!(resolving, Some(suggested));
        assert_eq!(
            check_source_existential_registration_proof(&source, &nodes, &symbols)
                .unwrap_err(),
            "registration.source_existential_proof_invalid",
            "retry classification does not admit the unchanged uncited source proof"
        );
    }
}

#[test]
fn source_registration_discharge_metadata_keeps_real_negative_errors_and_no_association() {
    use mizar_atp::problem::ConcreteFormat;
    use mizar_kernel::checker::KernelCheckStatus;
    use mizar_proof::{
        policy::{KernelEvidenceFormat, VerifierPolicy},
        status::{
            check_source_registration_candidate as atp,
            check_source_registration_discharge_candidate as old,
            check_source_registration_discharge_metadata as check,
        },
    };
    use mizar_vc::vc_ir::{VcId, VcStatus};
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (source, nodes, symbols) = if multiple {
            step5c13_registration_inputs(&text)
        } else {
            step5c3_registration_inputs(&text)
        }
        .unwrap();
        let (_, open, discharged) = source_candidate_core_and_vcs(multiple, &text);
        let direct = discharged
            .vcs()
            .iter()
            .find(|vc| matches!(vc.status, VcStatus::Discharged { .. }))
            .unwrap()
            .id;
        let correctness = discharged
            .vcs()
            .iter()
            .find(|vc| vc.status == VcStatus::NeedsAtp)
            .unwrap()
            .id;
        let foreign_text = text.replace("marked", "foreignmarked");
        let (foreign_source, foreign_nodes, foreign_symbols) = if multiple {
            step5c13_registration_inputs(&foreign_text)
        } else {
            step5c3_registration_inputs(&foreign_text)
        }
        .unwrap();
        let policy = VerifierPolicy::release();
        let snapshot = super::super::shared::snapshot_id(0);
        for (s, n, e) in [
            (&foreign_source, &nodes, &symbols),
            (&source, &foreign_nodes, &symbols),
            (&source, &nodes, &foreign_symbols),
            (&foreign_source, &foreign_nodes, &symbols),
            (&foreign_source, &nodes, &foreign_symbols),
        ] {
            assert_eq!(
                check(s, n, e, snapshot, direct, &policy).unwrap_err(),
                old(s, n, e, snapshot, direct, &policy).unwrap_err()
            );
        }
        for target in [correctness, VcId::new(open.vcs().len())] {
            assert_eq!(
                check(&source, &nodes, &symbols, snapshot, target, &policy).unwrap_err(),
                old(&source, &nodes, &symbols, snapshot, target, &policy).unwrap_err()
            );
        }
        if !multiple {
            let unsupported = text.replacen("take the set;", "take the wmarked set;", 1);
            assert_ne!(unsupported, text);
            let (s, n, e) = step5c3_registration_inputs(&unsupported).unwrap();
            assert_eq!(
                check(&s, &n, &e, snapshot, direct, &policy).unwrap_err(),
                old(&s, &n, &e, snapshot, direct, &policy).unwrap_err()
            );
        }
        for denied in [
            policy
                .clone()
                .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
            policy.clone().with_kernel_evidence_formats([]),
        ] {
            assert_eq!(
                check(&source, &nodes, &symbols, snapshot, direct, &denied).unwrap_err(),
                "registration.proof.kernel_policy_rejected"
            );
            assert_eq!(
                check(&source, &nodes, &symbols, snapshot, direct, &denied).unwrap_err(),
                old(&source, &nodes, &symbols, snapshot, direct, &denied).unwrap_err()
            );
        }
        let false_definitions = text.replace("means X = X", "means not X = X");
        let (s, n, e) = if multiple {
            step5c13_registration_inputs(&false_definitions)
        } else {
            step5c3_registration_inputs(&false_definitions)
        }
        .unwrap();
        let (result, _) = atp(
            &s,
            &n,
            &e,
            snapshot,
            correctness,
            (
                source_candidate_profile("metadata-negative", ConcreteFormat::Tptp),
                ConcreteFormat::Tptp,
            ),
            &policy,
        )
        .unwrap();
        assert_eq!(result.status(), KernelCheckStatus::Rejected);
        assert_eq!(
            check(&s, &n, &e, snapshot, correctness, &policy).unwrap_err(),
            old(&s, &n, &e, snapshot, correctness, &policy).unwrap_err()
        );
    }
}

#[test]
fn source_registration_discharge_selection_matches_fresh_actual_evidence() {
    use mizar_kernel::checker::KernelCheckStatus;
    use mizar_proof::{
        policy::{
            AcceptedGoalPolarity, CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput,
            VerifierPolicy,
        },
        selection::{
            CandidateSourceId, ProofEvidenceCandidate, ProofEvidenceSet, ProofWinnerClass,
            ProofWitnessPublication, select_winner,
        },
        status::{
            check_source_registration_discharge_metadata as check,
            select_source_registration_discharge_candidate as select,
        },
    };
    use mizar_vc::vc_ir::VcStatus;
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original.replace("marked", "retagged").replace("X", "Value");
        let mut targets = std::collections::BTreeSet::new();
        for text in [&original, &renamed] {
            let (source, nodes, symbols) = if multiple {
                step5c13_registration_inputs(text)
            } else {
                step5c3_registration_inputs(text)
            }
            .unwrap();
            let before = (source.clone(), nodes.clone(), symbols.clone());
            let (_, _, vcs) = source_candidate_core_and_vcs(multiple, text);
            let direct = vcs
                .vcs()
                .iter()
                .filter(|vc| matches!(vc.status, VcStatus::Discharged { .. }))
                .collect::<Vec<_>>();
            assert_eq!(direct.len(), if multiple { 3 } else { 1 });
            for vc in direct {
                for snapshot_index in 0..2 {
                    let snapshot = super::super::shared::snapshot_id(snapshot_index);
                    let mut prior = None;
                    for policy in [
                        VerifierPolicy::release(),
                        VerifierPolicy::development(),
                        VerifierPolicy::interactive(),
                        VerifierPolicy::release().with_schema_version(2),
                        VerifierPolicy::release().with_checker_schema_version(7),
                        VerifierPolicy::release().with_profile_id("source-selection-profile"),
                    ] {
                        let (result, decision, raw) =
                            check(&source, &nodes, &symbols, snapshot, vc.id, &policy).unwrap();
                        let raw = raw.unwrap();
                        assert_eq!(result.status(), KernelCheckStatus::Accepted);
                        assert_eq!(decision.class, CandidatePolicyClass::DischargedBuiltin);
                        assert!(result.used_axioms().is_empty());
                        let input = KernelPolicyInput::from_kernel_result(
                            &result,
                            KernelEvidenceOrigin::BuiltinDischarge,
                        );
                        let target = result.target_vc_fingerprint().sort_bytes();
                        let id = format!(
                            "source-registration-discharge-v1:{}",
                            target
                                .iter()
                                .map(|byte| format!("{byte:02x}"))
                                .collect::<String>()
                        );
                        let candidate = ProofEvidenceCandidate::from_trusted_kernel_input(
                            CandidateSourceId::new(id.clone()).unwrap(),
                            &input,
                        )
                        .unwrap()
                        .with_provenance_hash(raw.provenance_hash.digest);
                        let expected = select_winner(
                            &ProofEvidenceSet::new(target.clone(), None, policy.clone())
                                .with_candidates([candidate]),
                        );
                        let actual =
                            select(&source, &nodes, &symbols, snapshot, vc.id, &policy).unwrap();
                        assert_eq!(actual, expected);
                        assert_eq!(
                            actual,
                            select(&source, &nodes, &symbols, snapshot, vc.id, &policy).unwrap()
                        );
                        assert_eq!(actual.selected_candidate_id().unwrap().as_str(), id);
                        assert_eq!(actual.selected_class(), ProofWinnerClass::DischargedBuiltin);
                        assert!(actual.trusted_used_axioms_available());
                        let reuse = actual.reuse_metadata();
                        assert_eq!(reuse.encoded_problem_hash(), None);
                        assert_eq!(
                            reuse.selected_evidence_hash(),
                            input.accepted_evidence_hash()
                        );
                        assert_eq!(
                            reuse.selected_evidence_hash(),
                            Some(raw.accepted_result_hash.digest)
                        );
                        assert_eq!(
                            reuse.selected_candidate_provenance_hash(),
                            Some(raw.provenance_hash.digest)
                        );
                        assert_eq!(reuse.policy_fingerprint(), policy.policy_fingerprint());
                        assert_eq!(
                            reuse.accepted_goal_polarity(),
                            Some(AcceptedGoalPolarity::AssertFalseForRefutation)
                        );
                        assert_eq!(reuse.deterministic_discharge_hash(), None);
                        assert_eq!(reuse.selected_proof_witness_hash(), None);
                        assert_eq!(
                            reuse.proof_witness_publication(),
                            ProofWitnessPublication::ExternalDependencyGap
                        );
                        assert!(actual.ordered_diagnostic_refs().is_empty());
                        if let Some((previous_target, previous_raw)) = &prior {
                            assert_eq!(&target, previous_target);
                            let previous_raw: &mizar_artifact::proof_witness::KernelAcceptanceMetadata = previous_raw;
                            assert_eq!(raw.accepted_result_hash, previous_raw.accepted_result_hash);
                            assert_eq!(raw.provenance_hash, previous_raw.provenance_hash);
                            assert_ne!(
                                raw.verifier_policy_fingerprint,
                                previous_raw.verifier_policy_fingerprint
                            );
                        } else {
                            assert!(targets.insert(target.clone()));
                            prior = Some((target, raw));
                        }
                    }
                }
            }
            assert_eq!((source, nodes, symbols), before);
        }
    }
}

#[test]
fn source_registration_discharge_selection_preserves_upstream_failures() {
    use mizar_proof::{
        policy::{KernelEvidenceFormat, VerifierPolicy},
        status::{
            check_source_registration_discharge_metadata as check,
            select_source_registration_discharge_candidate as select,
        },
    };
    use mizar_vc::vc_ir::{VcId, VcStatus};
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (source, nodes, symbols) = if multiple {
            step5c13_registration_inputs(&text)
        } else {
            step5c3_registration_inputs(&text)
        }
        .unwrap();
        let (_, open, vcs) = source_candidate_core_and_vcs(multiple, &text);
        let direct = vcs
            .vcs()
            .iter()
            .find(|vc| matches!(vc.status, VcStatus::Discharged { .. }))
            .unwrap()
            .id;
        let correctness = vcs
            .vcs()
            .iter()
            .find(|vc| vc.status == VcStatus::NeedsAtp)
            .unwrap()
            .id;
        let foreign = text.replace("marked", "foreignmarked");
        let (foreign_source, foreign_nodes, foreign_symbols) = if multiple {
            step5c13_registration_inputs(&foreign)
        } else {
            step5c3_registration_inputs(&foreign)
        }
        .unwrap();
        let snapshot = super::super::shared::snapshot_id(0);
        let policy = VerifierPolicy::release();
        for (s, n, e, target) in [
            (&foreign_source, &nodes, &symbols, direct),
            (&source, &foreign_nodes, &symbols, direct),
            (&source, &nodes, &foreign_symbols, direct),
            (&source, &nodes, &symbols, correctness),
            (&source, &nodes, &symbols, VcId::new(open.vcs().len())),
        ] {
            assert_eq!(
                select(s, n, e, snapshot, target, &policy).unwrap_err(),
                check(s, n, e, snapshot, target, &policy).unwrap_err()
            );
        }
        for denied in [
            policy
                .clone()
                .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
            policy.clone().with_kernel_evidence_formats([]),
        ] {
            assert_eq!(
                select(&source, &nodes, &symbols, snapshot, direct, &denied).unwrap_err(),
                check(&source, &nodes, &symbols, snapshot, direct, &denied).unwrap_err()
            );
        }
        let builtin_only = policy
            .clone()
            .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]);
        assert_eq!(
            select(&source, &nodes, &symbols, snapshot, direct, &builtin_only)
                .unwrap()
                .selected_class(),
            mizar_proof::selection::ProofWinnerClass::DischargedBuiltin
        );
        if !multiple {
            let unsupported = text.replacen("take the set;", "take the wmarked set;", 1);
            assert_ne!(text, unsupported);
            let (s, n, e) = step5c3_registration_inputs(&unsupported).unwrap();
            assert_eq!(
                select(&s, &n, &e, snapshot, direct, &policy).unwrap_err(),
                check(&s, &n, &e, snapshot, direct, &policy).unwrap_err()
            );
        }
    }
}

#[test]
fn source_registration_discharge_selection_keeps_false_correctness_rejection_and_unknown_identity()
{
    use mizar_atp::problem::ConcreteFormat;
    use mizar_kernel::checker::KernelCheckStatus;
    use mizar_proof::{
        policy::{CandidatePolicyClass, VerifierPolicy},
        selection::ProofWinnerClass,
        status::{
            check_source_registration_candidate as atp,
            check_source_registration_discharge_metadata as check,
            select_source_registration_discharge_candidate as select,
        },
    };
    use mizar_vc::{
        dependency_slice::{
            DependencySliceCompleteness, DependencySliceInput, try_compute_dependency_slices,
        },
        vc_ir::{HashMarker, VcStatus},
    };
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let false_definitions = original.replace("means X = X", "means not X = X");
        assert_ne!(false_definitions, original);
        let (source, nodes, symbols) = if multiple {
            step5c13_registration_inputs(&false_definitions)
        } else {
            step5c3_registration_inputs(&false_definitions)
        }
        .unwrap();
        let (_, open, discharged) = source_candidate_core_and_vcs(multiple, &false_definitions);
        let snapshot = super::super::shared::snapshot_id(0);
        let policy = VerifierPolicy::release();
        for vc in discharged
            .vcs()
            .iter()
            .filter(|vc| matches!(vc.status, VcStatus::Discharged { .. }))
        {
            let selection = select(&source, &nodes, &symbols, snapshot, vc.id, &policy).unwrap();
            assert_eq!(
                selection.selected_class(),
                ProofWinnerClass::DischargedBuiltin
            );
            assert_eq!(selection.reuse_metadata().encoded_problem_hash(), None);
            assert_eq!(
                selection.reuse_metadata().deterministic_discharge_hash(),
                None
            );
            assert!(open.canonical_vc_fingerprint(vc.id).is_none());
            let VcStatus::Discharged { evidence, .. } = &vc.status else {
                unreachable!()
            };
            assert!(matches!(
                evidence.evidence_hash,
                HashMarker::ConservativeUnknown { .. }
            ));
        }
        let slices = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &open,
            discharge_output: None,
        })
        .unwrap();
        assert!(slices.slices().iter().all(
            |slice| slice.completeness() == DependencySliceCompleteness::IncompleteUncacheable
        ));
        let correctness = discharged
            .vcs()
            .iter()
            .find(|vc| vc.status == VcStatus::NeedsAtp)
            .unwrap()
            .id;
        let (result, decision) = atp(
            &source,
            &nodes,
            &symbols,
            snapshot,
            correctness,
            (
                source_candidate_profile("source-selection-negative", ConcreteFormat::Tptp),
                ConcreteFormat::Tptp,
            ),
            &policy,
        )
        .unwrap();
        assert_eq!(result.status(), KernelCheckStatus::Rejected);
        assert_eq!(decision.class, CandidatePolicyClass::KernelRejected);
        assert!(!result.rejections().is_empty());
        assert_eq!(
            select(&source, &nodes, &symbols, snapshot, correctness, &policy).unwrap_err(),
            check(&source, &nodes, &symbols, snapshot, correctness, &policy).unwrap_err()
        );
    }
}
#[test]
fn source_registration_discharge_witness_projects_exact_checked_payload_and_reference() {
    use mizar_artifact::{
        proof_witness::{
            EvidenceKind, ProofStatus, ProofWitnessReadOptions, builtin_proof_witness_ref_json,
            proof_witness_ref_json, read_builtin_proof_witness_ref, read_proof_witness_ref,
        },
        registration_summary::ArtifactHashClass,
        store::{
            CanonicalHashDomain, CanonicalJson, HashClass, SchemaVersion, canonical_json_bytes,
        },
    };
    use mizar_proof::{
        policy::{AcceptedGoalPolarity, VerifierPolicy},
        selection::{ProofWinnerClass, ProofWitnessPublication},
        status::{
            check_source_registration_discharge_metadata as check,
            project_source_registration_discharge_witness as project,
            select_source_registration_discharge_candidate as select,
        },
    };
    use mizar_vc::{
        dependency_slice::{
            DependencySliceCompleteness, DependencySliceInput, try_compute_dependency_slices,
        },
        discharge::{DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_existential_registration,
        kernel_evidence_handoff::build_source_existential_discharge_candidate,
        vc_ir::{GenerationSchemaVersion, HashMarker, VcSchemaVersion, VcStatus},
    };
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original.replace("marked", "retagged").replace("X", "Value");
        let mut contents = std::collections::BTreeSet::new();
        for text in [&original, &renamed] {
            let (source, nodes, symbols) = if multiple {
                step5c13_registration_inputs(text)
            } else {
                step5c3_registration_inputs(text)
            }
            .unwrap();
            let before = (source.clone(), nodes.clone(), symbols.clone());
            let (core, _, _) = source_candidate_core_and_vcs(multiple, text);
            for snapshot_index in 0..2 {
                let snapshot = super::super::shared::snapshot_id(snapshot_index);
                let open = generate_source_existential_registration(
                    &core,
                    snapshot,
                    &GenerationSchemaVersion::new("source-existential-registration-v1"),
                    &VcSchemaVersion::new("vc-v1"),
                )
                .unwrap();
                let discharged = try_discharge(DischargeInput {
                    vc_set: &open,
                    policy: &DischargePolicy::default(),
                })
                .unwrap();
                let slices = try_compute_dependency_slices(DependencySliceInput {
                    vc_set: &open,
                    discharge_output: None,
                })
                .unwrap();
                assert!(slices.slices().iter().all(|slice| slice.completeness()
                    == DependencySliceCompleteness::IncompleteUncacheable));
                assert_eq!(
                    discharged.evidence_records().len(),
                    if multiple { 3 } else { 1 }
                );
                for record in discharged.evidence_records() {
                    let handoff = build_source_existential_discharge_candidate(
                        &core,
                        &open,
                        record.vc,
                        &discharged,
                    )
                    .unwrap();
                    let wire = handoff.encode_local_evidence(None).unwrap();
                    assert!(open.canonical_vc_fingerprint(record.vc).is_none());
                    let VcStatus::Discharged { evidence, .. } =
                        &discharged.vc_set().vc(record.vc).unwrap().status
                    else {
                        unreachable!()
                    };
                    assert!(matches!(
                        evidence.evidence_hash,
                        HashMarker::ConservativeUnknown { .. }
                    ));
                    let mut prior = None;
                    for policy in [
                        VerifierPolicy::release(),
                        VerifierPolicy::development(),
                        VerifierPolicy::interactive(),
                        VerifierPolicy::release().with_schema_version(2),
                        VerifierPolicy::release().with_checker_schema_version(7),
                        VerifierPolicy::release().with_profile_id("source-witness-profile"),
                    ] {
                        let actual =
                            project(&source, &nodes, &symbols, snapshot, record.vc, &policy)
                                .unwrap();
                        assert_eq!(
                            actual,
                            project(&source, &nodes, &symbols, snapshot, record.vc, &policy)
                                .unwrap()
                        );
                        let (selection, reference, bytes) = actual;
                        assert_eq!(
                            selection,
                            select(&source, &nodes, &symbols, snapshot, record.vc, &policy)
                                .unwrap()
                        );
                        let (result, _, raw) =
                            check(&source, &nodes, &symbols, snapshot, record.vc, &policy).unwrap();
                        assert_eq!(reference.kernel_acceptance, raw.unwrap());
                        assert_eq!(bytes, wire);
                        let target = handoff.canonical_evidence().target_vc();
                        assert_eq!(
                            result.target_vc_fingerprint().algorithm_id,
                            target.algorithm_id
                        );
                        assert_eq!(result.target_vc_fingerprint().digest, target.digest);
                        assert_eq!(
                            selection.selected_class(),
                            ProofWinnerClass::DischargedBuiltin
                        );
                        let reuse = selection.reuse_metadata();
                        assert_eq!(
                            reuse.accepted_goal_polarity(),
                            Some(AcceptedGoalPolarity::AssertFalseForRefutation)
                        );
                        assert_eq!(reuse.encoded_problem_hash(), None);
                        assert_eq!(reuse.selected_proof_witness_hash(), None);
                        assert_eq!(reuse.deterministic_discharge_hash(), None);
                        assert_eq!(
                            reuse.proof_witness_publication(),
                            ProofWitnessPublication::ExternalDependencyGap
                        );
                        assert_eq!(reference.schema_version, SchemaVersion::new(3, 0));
                        assert_eq!(reference.proof_status, ProofStatus::DischargedBuiltin);
                        assert_eq!(
                            reference.evidence_kind,
                            EvidenceKind::FormulaSubstitutionKernelEvidence
                        );
                        assert_eq!(
                            reference.obligation_id,
                            selection.selected_candidate_id().unwrap().as_str()
                        );
                        let payload_hex = bytes
                            .iter()
                            .map(|byte| format!("{byte:02x}"))
                            .collect::<String>();
                        let payload_domain = CanonicalHashDomain::new(
                            HashClass::Artifact,
                            "mizar-proof/source-registration-discharge-witness-payload",
                            SchemaVersion::new(1, 0),
                        );
                        assert_eq!(
                            reference.witness_artifact_hash.class,
                            ArtifactHashClass::Artifact
                        );
                        assert_eq!(
                            reference.witness_artifact_hash.schema_family,
                            "mizar-proof/source-registration-discharge-witness-payload"
                        );
                        assert_eq!(
                            reference.witness_artifact_hash.schema_version,
                            SchemaVersion::new(1, 0)
                        );
                        assert_eq!(
                            reference.witness_artifact_hash.digest,
                            payload_domain.hash(&CanonicalJson::string(&payload_hex), &[])
                        );
                        let m = &reference.kernel_acceptance;
                        let obligation = CanonicalJson::object([
                            (
                                "original_target_bytes",
                                CanonicalJson::string(
                                    result
                                        .target_vc_fingerprint()
                                        .sort_bytes()
                                        .iter()
                                        .map(|b| format!("{b:02x}"))
                                        .collect::<String>(),
                                ),
                            ),
                            (
                                "kernel_profile_fingerprint",
                                CanonicalJson::string(
                                    m.kernel_profile_fingerprint.to_artifact_hash_string(),
                                ),
                            ),
                            (
                                "verifier_policy_fingerprint",
                                CanonicalJson::string(
                                    m.verifier_policy_fingerprint.to_artifact_hash_string(),
                                ),
                            ),
                            (
                                "checker_schema_version",
                                CanonicalJson::string(m.checker_schema_version.to_string()),
                            ),
                            (
                                "evidence_schema_version",
                                CanonicalJson::string(m.evidence_schema_version.to_string()),
                            ),
                            (
                                "target_binding_hash",
                                CanonicalJson::string(
                                    m.target_binding_hash.to_artifact_hash_string(),
                                ),
                            ),
                            (
                                "formula_evidence_hash",
                                CanonicalJson::string(
                                    m.formula_evidence_hash.to_artifact_hash_string(),
                                ),
                            ),
                            (
                                "substitution_evidence_hash",
                                CanonicalJson::string(
                                    m.substitution_evidence_hash.to_artifact_hash_string(),
                                ),
                            ),
                            (
                                "provenance_hash",
                                CanonicalJson::string(m.provenance_hash.to_artifact_hash_string()),
                            ),
                            ("formula_context_hash", CanonicalJson::null()),
                            (
                                "accepted_result_hash",
                                CanonicalJson::string(
                                    m.accepted_result_hash.to_artifact_hash_string(),
                                ),
                            ),
                            (
                                "accepted_goal_polarity",
                                CanonicalJson::string(
                                    AcceptedGoalPolarity::AssertFalseForRefutation.as_str(),
                                ),
                            ),
                            (
                                "witness_artifact_hash",
                                CanonicalJson::string(
                                    reference.witness_artifact_hash.to_artifact_hash_string(),
                                ),
                            ),
                        ])
                        .unwrap();
                        assert!(m.formula_context_hash.is_none());
                        let domain = CanonicalHashDomain::new(
                            HashClass::Interface,
                            "mizar-proof/source-registration-discharge-obligation",
                            SchemaVersion::new(1, 0),
                        );
                        assert_eq!(
                            reference.obligation_fingerprint.class,
                            ArtifactHashClass::Interface
                        );
                        assert_eq!(
                            reference.obligation_fingerprint.schema_family,
                            "mizar-proof/source-registration-discharge-obligation"
                        );
                        assert_eq!(
                            reference.obligation_fingerprint.schema_version,
                            SchemaVersion::new(1, 0)
                        );
                        assert_eq!(
                            reference.obligation_fingerprint.digest,
                            domain.hash(&obligation, &[])
                        );
                        assert_eq!(
                            reference.witness_path,
                            format!(
                                "proof-witnesses/source-registration-discharge-v1/{}.bin",
                                reference
                                    .witness_artifact_hash
                                    .digest
                                    .as_bytes()
                                    .iter()
                                    .map(|b| format!("{b:02x}"))
                                    .collect::<String>()
                            )
                        );
                        let json = builtin_proof_witness_ref_json(&reference).unwrap();
                        assert_eq!(
                            read_builtin_proof_witness_ref(
                                &json,
                                ProofWitnessReadOptions {
                                    expected_witness_artifact_hash: Some(
                                        &reference.witness_artifact_hash
                                    ),
                                    ..Default::default()
                                }
                            )
                            .unwrap(),
                            reference
                        );
                        assert_eq!(
                            canonical_json_bytes(&json),
                            canonical_json_bytes(
                                &builtin_proof_witness_ref_json(&reference).unwrap()
                            )
                        );
                        assert!(proof_witness_ref_json(&reference).is_err());
                        assert!(read_proof_witness_ref(&json, Default::default()).is_err());
                        let mut corrupt = bytes.clone();
                        corrupt[0] ^= 1;
                        let corrupt_hex = corrupt
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>();
                        assert_ne!(
                            payload_domain.hash(&CanonicalJson::string(corrupt_hex), &[]),
                            reference.witness_artifact_hash.digest
                        );
                        if let Some((previous_payload, previous_obligation)) = &prior {
                            assert_eq!(&reference.witness_artifact_hash, previous_payload);
                            assert_ne!(&reference.obligation_fingerprint, previous_obligation);
                        } else {
                            assert!(
                                contents.insert(
                                    reference.witness_artifact_hash.to_artifact_hash_string()
                                )
                            );
                            prior = Some((
                                reference.witness_artifact_hash,
                                reference.obligation_fingerprint,
                            ));
                        }
                    }
                }
            }
            assert_eq!((source, nodes, symbols), before);
        }
    }
}

#[test]
fn source_registration_discharge_witness_preserves_failures_and_false_correctness() {
    use mizar_proof::{
        policy::{CandidatePolicyClass, KernelEvidenceFormat, VerifierPolicy},
        status::{
            check_source_registration_candidate as atp,
            check_source_registration_discharge_metadata as check,
            project_source_registration_discharge_witness as project,
        },
    };
    use mizar_vc::vc_ir::{VcId, VcStatus};
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let text = original.replace("means X = X", "means not X = X");
        assert_ne!(text, original);
        let (source, nodes, symbols) = if multiple {
            step5c13_registration_inputs(&text)
        } else {
            step5c3_registration_inputs(&text)
        }
        .unwrap();
        let (_, open, vcs) = source_candidate_core_and_vcs(multiple, &text);
        let direct = vcs
            .vcs()
            .iter()
            .find(|vc| matches!(vc.status, VcStatus::Discharged { .. }))
            .unwrap()
            .id;
        let correctness = vcs
            .vcs()
            .iter()
            .find(|vc| vc.status == VcStatus::NeedsAtp)
            .unwrap()
            .id;
        let snapshot = super::super::shared::snapshot_id(0);
        let policy = VerifierPolicy::release();
        assert!(project(&source, &nodes, &symbols, snapshot, direct, &policy).is_ok());
        let foreign = text.replace("marked", "foreignmarked");
        let (fs, fnodes, fenv) = if multiple {
            step5c13_registration_inputs(&foreign)
        } else {
            step5c3_registration_inputs(&foreign)
        }
        .unwrap();
        for (s, n, e, target) in [
            (&fs, &nodes, &symbols, direct),
            (&source, &fnodes, &symbols, direct),
            (&source, &nodes, &fenv, direct),
            (&source, &nodes, &symbols, correctness),
            (&source, &nodes, &symbols, VcId::new(open.vcs().len())),
        ] {
            assert_eq!(
                project(s, n, e, snapshot, target, &policy).unwrap_err(),
                check(s, n, e, snapshot, target, &policy).unwrap_err()
            );
        }
        let denied = policy
            .clone()
            .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]);
        assert_eq!(
            project(&source, &nodes, &symbols, snapshot, direct, &denied).unwrap_err(),
            check(&source, &nodes, &symbols, snapshot, direct, &denied).unwrap_err()
        );
        if !multiple {
            let unsupported = text.replacen("take the set;", "take the wmarked set;", 1);
            let (s, n, e) = step5c3_registration_inputs(&unsupported).unwrap();
            assert_eq!(
                project(&s, &n, &e, snapshot, direct, &policy).unwrap_err(),
                check(&s, &n, &e, snapshot, direct, &policy).unwrap_err()
            );
        }
        let format = mizar_atp::problem::ConcreteFormat::Tptp;
        let (result, decision) = atp(
            &source,
            &nodes,
            &symbols,
            snapshot,
            correctness,
            (
                source_candidate_profile("source-witness-negative", format),
                format,
            ),
            &policy,
        )
        .unwrap();
        assert_eq!(
            result.status(),
            mizar_kernel::checker::KernelCheckStatus::Rejected
        );
        assert_eq!(decision.class, CandidatePolicyClass::KernelRejected);
        assert!(!result.rejections().is_empty());
    }
}

#[test]
fn source_registration_candidate_metadata_single_matches_independent_wire_and_kernel() {
    use mizar_artifact::{registration_summary::ArtifactHashClass, store::SchemaVersion};
    use mizar_atp::{
        backend::{BackendCandidatePayload, build_source_registration_candidate_inputs as build},
        problem::ConcreteFormat,
    };
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::*,
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
        rejection::TargetVcFingerprint,
    };
    use mizar_proof::{
        policy::{
            AcceptedGoalPolarity, CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput,
            PolicyCandidate, ProofPolicyEvaluator, VerifierPolicy,
        },
        status::{
            check_source_registration_candidate as old,
            check_source_registration_candidate_metadata as check,
        },
    };
    use mizar_vc::{
        kernel_evidence_handoff::{
            KernelContextIdentitySource as VcSource, build_source_existential_kernel_handoff,
        },
        vc_ir::{VcFormulaRef, VcStatus},
    };
    let original = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let renamed = original
        .replace("wmarked", "retagged")
        .replace("X", "Value");
    let policy = VerifierPolicy::release();
    let mut identities = std::collections::BTreeSet::new();
    for text in [&original, &renamed] {
        let (source, nodes, symbols) = step5c3_registration_inputs(text).unwrap();
        let before = (source.clone(), nodes.clone(), symbols.clone());
        let (core, open, vcs) = source_candidate_core_and_vcs(false, text);
        let vc = vcs
            .vcs()
            .iter()
            .find(|vc| vc.status == VcStatus::NeedsAtp)
            .unwrap();
        let handoff = build_source_existential_kernel_handoff(&core, &vcs, vc.id).unwrap();
        assert_ne!(
            handoff,
            build_source_existential_kernel_handoff(&core, &open, vc.id).unwrap()
        );
        let envelope = handoff.canonical_evidence();
        assert_eq!(envelope.schema_version(), 1);
        let target = envelope.target_vc();
        let target = TargetVcFingerprint::new(target.algorithm_id, target.digest.clone());
        let mut parse_context = FormulaEvidenceParseContext::v1(
            Fingerprint::new(target.algorithm_id, target.digest.clone()),
            KernelProfileRecord::v1(
                envelope.kernel_profile().profile_id,
                ClauseTautologyPolicy::Reject,
            ),
        );
        parse_context.accepted_schema_versions.insert(2);
        let wire = source_candidate_wire(&handoff);
        let parsed = parse_formula_evidence(&wire, &parse_context).unwrap();
        assert_eq!(parsed.schema_version(), 2);
        assert!(!parsed.equality_instances().is_empty());
        let v1 = parse_formula_evidence(
            &handoff.encode_local_evidence(None).unwrap(),
            &parse_context,
        )
        .unwrap();
        assert!(v1.equality_instances().is_empty());
        assert_eq!(parsed.formulas(), v1.formulas());
        assert_eq!(parsed.substitutions(), v1.substitutions());
        assert_eq!(parsed.provenance(), v1.provenance());
        // The four canonical-v1 components exclude the actual v2 equality names.
        let canonical = std::str::from_utf8(handoff.canonical_hash_input()).unwrap();
        let profile_start = canonical.find("kernel-profile=").unwrap();
        let symbols_start = canonical.find("[symbol-manifest]\n").unwrap();
        let substitutions_start = canonical.find("[substitutions]\n").unwrap();
        let provenance_start = canonical.find("[provenance]\n").unwrap();
        let goal_start = canonical.find("[final-goal]\n").unwrap();
        let components = [
            (
                "profile",
                canonical[profile_start..symbols_start].to_owned(),
            ),
            (
                "formula",
                format!(
                    "{}{}",
                    &canonical[symbols_start..substitutions_start],
                    &canonical[goal_start..]
                ),
            ),
            (
                "substitutions",
                canonical[substitutions_start..provenance_start].to_owned(),
            ),
            (
                "provenance",
                canonical[provenance_start..goal_start].to_owned(),
            ),
        ]
        .map(|(kind, payload)| {
            mizar_session::hash_text(&format!(
                "mizar-vc/local-evidence-component/v1\n{kind}\n{payload}"
            ))
        });
        assert_eq!(
            components,
            handoff.local_evidence_component_hashes().unwrap()
        );
        let entries = handoff
            .context_identity()
            .entries()
            .iter()
            .map(|entry| {
                let source = match entry.source() {
                    VcSource::LocalHypothesis { local_context_id } => {
                        KernelContextIdentitySource::LocalHypothesis { local_context_id }
                    }
                    VcSource::CitedPremise { local_context_id } => {
                        KernelContextIdentitySource::CitedPremise { local_context_id }
                    }
                    VcSource::GeneratedVcFact { vc_fact_id } => {
                        KernelContextIdentitySource::GeneratedVcFact { vc_fact_id }
                    }
                    _ => panic!("actual source context"),
                };
                let producer = match entry.producer_formula_ref() {
                    VcFormulaRef::Core(id) => KernelFormulaProducerRef::Core(id),
                    VcFormulaRef::Generated(id) => KernelFormulaProducerRef::Generated(
                        KernelVcGeneratedFormulaId::new(id.index()),
                    ),
                    _ => panic!("actual producer"),
                };
                KernelContextIdentityEntry::new(
                    source,
                    entry.formula_id(),
                    Fingerprint::new(
                        entry.formula_fingerprint().algorithm_id,
                        entry.formula_fingerprint().digest.clone(),
                    ),
                    producer,
                )
            })
            .collect();
        let context = FormulaEvidenceContext::with_context_identity(
            Some(handoff.context_identity_hash().as_bytes().to_vec()),
            Vec::new(),
            Vec::new(),
            Some(KernelContextIdentityPayload::new(
                target.clone(),
                handoff.canonical_hash(),
                handoff.context_identity_hash(),
                entries,
            )),
            ImportedFactContextLimits::default(),
        )
        .unwrap();
        let independent = check_kernel_evidence(KernelEvidenceCheckInput {
            target_vc_fingerprint: &target,
            evidence: &parsed,
            formula_context: Some(&context),
            check_kind: KernelEvidenceCheckKind::ProofObligation,
            policy: KernelCheckPolicy::default(),
            limits: KernelEvidenceCheckLimits::default(),
        });
        assert_eq!(independent.status(), KernelCheckStatus::Accepted);
        let input = KernelPolicyInput::from_kernel_result(
            &independent,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
        );
        assert_eq!(
            input.accepted_goal_polarity(),
            Some(AcceptedGoalPolarity::AssertFalseForRefutation)
        );
        for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
            let profile = source_candidate_profile("source-metadata", format);
            let (encoded, actual) = build(&core, &vcs, vc.id, profile.clone(), format).unwrap();
            let output = check(
                &source,
                &nodes,
                &symbols,
                super::super::shared::snapshot_id(0),
                vc.id,
                (profile.clone(), format),
                &policy,
            )
            .unwrap();
            let (result, decision, metadata, candidate) = &output;
            assert_eq!(result, &independent);
            assert_eq!(
                *decision,
                ProofPolicyEvaluator::new(policy.clone())
                    .evaluate_candidate(&PolicyCandidate::KernelResult(input.clone()))
            );
            assert_eq!(decision.class, CandidatePolicyClass::KernelVerified);
            assert_eq!(candidate, &actual);
            assert_eq!(
                candidate.payload(),
                &BackendCandidatePayload::FormulaSubstitutionBytes(wire.clone())
            );
            assert_eq!(candidate.target_binding(), encoded.target_binding());
            assert_eq!(candidate.encoded_problem_hash(), encoded.input_hash());
            assert_eq!(candidate.provenance_hash(), encoded.provenance_hash());
            assert_eq!(candidate.formula_label_refs(), encoded.formula_labels());
            assert_eq!(candidate.symbol_binding_refs(), encoded.symbol_bindings());
            assert!(identities.insert(candidate.candidate_id().to_owned()));
            assert_eq!(
                source_candidate_check_bytes(&handoff, &wire, encoded.target_binding()).unwrap(),
                result.status()
            );
            assert_eq!(
                old(
                    &source,
                    &nodes,
                    &symbols,
                    super::super::shared::snapshot_id(0),
                    vc.id,
                    (profile.clone(), format),
                    &policy
                )
                .unwrap(),
                (result.clone(), decision.clone())
            );
            assert_eq!(
                check(
                    &source,
                    &nodes,
                    &symbols,
                    super::super::shared::snapshot_id(0),
                    vc.id,
                    (profile, format),
                    &policy
                )
                .unwrap(),
                output
            );
            let metadata = metadata.as_ref().unwrap();
            assert_eq!(
                metadata.checker_schema_version,
                SchemaVersion::new(policy.checker_schema_version(), 0)
            );
            assert_eq!(
                metadata.evidence_schema_version,
                SchemaVersion::new(parsed.schema_version(), 0)
            );
            assert_eq!(metadata.formula_context_hash, None);
            for (reference, family, version, digest) in [
                (
                    &metadata.kernel_profile_fingerprint,
                    "mizar-vc/local-evidence-component/profile",
                    SchemaVersion::new(1, 0),
                    components[0],
                ),
                (
                    &metadata.formula_evidence_hash,
                    "mizar-vc/local-evidence-component/formula",
                    SchemaVersion::new(1, 0),
                    components[1],
                ),
                (
                    &metadata.substitution_evidence_hash,
                    "mizar-vc/local-evidence-component/substitutions",
                    SchemaVersion::new(1, 0),
                    components[2],
                ),
                (
                    &metadata.provenance_hash,
                    "mizar-vc/local-evidence-component/provenance",
                    SchemaVersion::new(1, 0),
                    components[3],
                ),
                (
                    &metadata.target_binding_hash,
                    "mizar-vc/kernel-evidence-handoff-canonical",
                    SchemaVersion::new(1, 0),
                    handoff.canonical_hash(),
                ),
                (
                    &metadata.verifier_policy_fingerprint,
                    "mizar-proof/verifier-policy",
                    SchemaVersion::new(policy.schema_version(), 0),
                    policy.policy_fingerprint().hash(),
                ),
                (
                    &metadata.accepted_result_hash,
                    "mizar-proof/kernel-accepted-evidence",
                    SchemaVersion::new(1, 0),
                    input.accepted_evidence_hash().unwrap(),
                ),
            ] {
                assert_eq!(reference.class, ArtifactHashClass::Interface);
                assert_eq!(reference.schema_family, family);
                assert_eq!(reference.schema_version, version);
                assert_eq!(reference.digest, digest);
            }
        }
        assert_eq!((source, nodes, symbols), before);
    }
}

#[test]
fn source_registration_candidate_metadata_aggregate_preserves_rejection_snapshot_and_policy_identity()
 {
    use mizar_artifact::store::SchemaVersion;
    use mizar_atp::{
        backend::{BackendCandidatePayload, build_source_registration_candidate_inputs as build},
        problem::ConcreteFormat,
    };
    use mizar_kernel::{
        checker::{KernelCheckStatus, KernelEvidenceCheckKind},
        rejection::RejectionDetail,
    };
    use mizar_proof::{
        policy::{CandidatePolicyClass, KernelEvidenceOrigin, KernelPolicyInput, VerifierPolicy},
        status::check_source_registration_candidate_metadata as check,
    };
    use mizar_vc::{
        discharge::{DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_existential_registration,
        kernel_evidence_handoff::build_source_existential_kernel_handoff,
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion, VcStatus},
    };
    let original = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let renamed = original.replace("marked", "retagged").replace("X", "Value");
    let mut seen = std::collections::BTreeMap::new();
    for (variant, base) in [&original, &renamed].into_iter().enumerate() {
        let formal = if variant == 0 { "X" } else { "Value" };
        let negative = base.replacen(
            &format!("means {formal} = {formal}"),
            &format!("means not {formal} = {formal}"),
            1,
        );
        assert_ne!(&negative, base);
        for (is_negative, text) in [(false, base), (true, &negative)] {
            let (source, nodes, symbols) = step5c13_registration_inputs(text).unwrap();
            let before = (source.clone(), nodes.clone(), symbols.clone());
            let (core, _, _) = source_candidate_core_and_vcs(true, text);
            for snapshot_index in 0..2 {
                let snapshot = super::super::shared::snapshot_id(snapshot_index);
                let open = generate_source_existential_registration(
                    &core,
                    snapshot,
                    &GenerationSchemaVersion::new("source-existential-registration-v1"),
                    &VcSchemaVersion::new("vc-v1"),
                )
                .unwrap();
                let discharge = try_discharge(DischargeInput {
                    vc_set: &open,
                    policy: &DischargePolicy::default(),
                })
                .unwrap();
                let vcs = discharge.vc_set();
                let mut targets = std::collections::BTreeSet::new();
                let mut counts = [0, 0];
                for vc in vcs
                    .vcs()
                    .iter()
                    .filter(|vc| vc.status == VcStatus::NeedsAtp)
                {
                    let handoff =
                        build_source_existential_kernel_handoff(&core, vcs, vc.id).unwrap();
                    assert!(targets.insert(handoff.canonical_hash().as_bytes().to_vec()));
                    let wire = source_candidate_wire(&handoff);
                    let components = handoff.local_evidence_component_hashes().unwrap();
                    let false_goal = is_negative && vc.id.index() != 3;
                    for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
                        for (profile_index, name) in ["source-metadata", "source-metadata-profile"]
                            .into_iter()
                            .enumerate()
                        {
                            let profile = source_candidate_profile(name, format);
                            let (encoded, actual) =
                                build(&core, vcs, vc.id, profile.clone(), format).unwrap();
                            let independent_status = source_candidate_check_bytes(
                                &handoff,
                                &wire,
                                encoded.target_binding(),
                            )
                            .unwrap();
                            for (policy_index, policy) in [
                                VerifierPolicy::release(),
                                VerifierPolicy::development(),
                                VerifierPolicy::interactive(),
                                VerifierPolicy::release().with_schema_version(2),
                                VerifierPolicy::release().with_checker_schema_version(7),
                                VerifierPolicy::release()
                                    .with_profile_id("configured-source-metadata"),
                            ]
                            .into_iter()
                            .enumerate()
                            {
                                let (result, decision, metadata, candidate) = check(
                                    &source,
                                    &nodes,
                                    &symbols,
                                    snapshot,
                                    vc.id,
                                    (profile.clone(), format),
                                    &policy,
                                )
                                .unwrap();
                                assert_eq!(candidate, actual);
                                assert_eq!(
                                    candidate.payload(),
                                    &BackendCandidatePayload::FormulaSubstitutionBytes(
                                        wire.clone()
                                    )
                                );
                                assert_eq!(result.status(), independent_status);
                                assert_eq!(
                                    result.target_vc_fingerprint().algorithm_id,
                                    encoded.target_binding().fingerprint().algorithm_id()
                                );
                                assert_eq!(
                                    result.target_vc_fingerprint().digest,
                                    encoded.target_binding().fingerprint().digest()
                                );
                                assert_eq!(
                                    result.evidence_check_kind(),
                                    Some(KernelEvidenceCheckKind::ProofObligation)
                                );
                                assert!(!result.policy_taint());
                                assert_eq!(decision.kernel_rejections, result.rejections());
                                let input = KernelPolicyInput::from_kernel_result(
                                    &result,
                                    KernelEvidenceOrigin::AtpFormulaSubstitution,
                                );
                                if false_goal {
                                    assert_eq!(result.status(), KernelCheckStatus::Rejected);
                                    assert_eq!(
                                        decision.class,
                                        CandidatePolicyClass::KernelRejected
                                    );
                                    assert!(
                                        result.rejections().iter().any(|record| record.detail()
                                            == RejectionDetail::InvalidSatRefutation)
                                    );
                                    assert!(metadata.is_none());
                                    assert!(input.accepted_evidence_hash().is_none());
                                    counts[1] += 1;
                                } else {
                                    assert_eq!(result.status(), KernelCheckStatus::Accepted);
                                    assert_eq!(
                                        decision.class,
                                        CandidatePolicyClass::KernelVerified
                                    );
                                    let metadata = metadata.unwrap();
                                    assert_eq!(
                                        metadata.kernel_profile_fingerprint.digest,
                                        components[0]
                                    );
                                    assert_eq!(
                                        metadata.formula_evidence_hash.digest,
                                        components[1]
                                    );
                                    assert_eq!(
                                        metadata.substitution_evidence_hash.digest,
                                        components[2]
                                    );
                                    assert_eq!(metadata.provenance_hash.digest, components[3]);
                                    assert_eq!(
                                        metadata.target_binding_hash.digest,
                                        handoff.canonical_hash()
                                    );
                                    assert_eq!(
                                        metadata.accepted_result_hash.digest,
                                        input.accepted_evidence_hash().unwrap()
                                    );
                                    assert_eq!(
                                        metadata.verifier_policy_fingerprint.digest,
                                        policy.policy_fingerprint().hash()
                                    );
                                    assert_eq!(
                                        metadata.verifier_policy_fingerprint.schema_version,
                                        SchemaVersion::new(policy.schema_version(), 0)
                                    );
                                    assert_eq!(
                                        metadata.checker_schema_version,
                                        SchemaVersion::new(policy.checker_schema_version(), 0)
                                    );
                                    assert_eq!(
                                        metadata.evidence_schema_version,
                                        SchemaVersion::new(2, 0)
                                    );
                                    assert!(metadata.formula_context_hash.is_none());
                                    if policy_index > 0 {
                                        let prior: &mizar_artifact::proof_witness::KernelAcceptanceMetadata = &seen[&(variant, is_negative, snapshot_index, vc.id.index(), format, profile_index, 0)];
                                        assert_ne!(
                                            metadata.verifier_policy_fingerprint,
                                            prior.verifier_policy_fingerprint
                                        );
                                        assert_eq!(
                                            metadata.accepted_result_hash,
                                            prior.accepted_result_hash
                                        );
                                    }
                                    if snapshot_index > 0 {
                                        let prior = &seen[&(
                                            variant,
                                            is_negative,
                                            0,
                                            vc.id.index(),
                                            format,
                                            profile_index,
                                            policy_index,
                                        )];
                                        assert_ne!(
                                            metadata.target_binding_hash,
                                            prior.target_binding_hash
                                        );
                                        assert_ne!(metadata.provenance_hash, prior.provenance_hash);
                                        assert_ne!(
                                            metadata.accepted_result_hash,
                                            prior.accepted_result_hash
                                        );
                                    }
                                    seen.insert(
                                        (
                                            variant,
                                            is_negative,
                                            snapshot_index,
                                            vc.id.index(),
                                            format,
                                            profile_index,
                                            policy_index,
                                        ),
                                        metadata,
                                    );
                                    counts[0] += 1;
                                }
                            }
                        }
                    }
                }
                assert_eq!(targets.len(), 3);
                assert_eq!(counts, if is_negative { [24, 48] } else { [72, 0] });
            }
            assert_eq!((source, nodes, symbols), before);
        }
    }
}

#[test]
fn source_registration_candidate_metadata_rejects_mixed_unsupported_and_denied_inputs() {
    use mizar_atp::problem::*;
    use mizar_proof::{
        policy::{CandidatePolicyClass, KernelEvidenceFormat, VerifierPolicy},
        selection::ProofWinnerClass,
        status::{
            check_source_registration_candidate as old,
            check_source_registration_candidate_metadata as check,
            prove_source_existential_registration as builtin,
            select_source_registration_candidate as select,
        },
    };
    use mizar_vc::vc_ir::{VcId, VcStatus};
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let foreign = text.replace("marked", "foreignmarked");
        let (source, nodes, symbols) = if multiple {
            step5c13_registration_inputs(&text)
        } else {
            step5c3_registration_inputs(&text)
        }
        .unwrap();
        let (fs, fnodes, fenv) = if multiple {
            step5c13_registration_inputs(&foreign)
        } else {
            step5c3_registration_inputs(&foreign)
        }
        .unwrap();
        let (_, _, vcs) = source_candidate_core_and_vcs(multiple, &text);
        let target = vcs
            .vcs()
            .iter()
            .find(|vc| vc.status == VcStatus::NeedsAtp)
            .unwrap()
            .id;
        let discharged = vcs
            .vcs()
            .iter()
            .find(|vc| matches!(vc.status, VcStatus::Discharged { .. }))
            .unwrap()
            .id;
        let snapshot = super::super::shared::snapshot_id(0);
        let policy = VerifierPolicy::release();
        let format = ConcreteFormat::Tptp;
        let profile = source_candidate_profile("source-metadata-negative", format);
        for (s, n, e, selected) in [
            (&fs, &nodes, &symbols, target),
            (&source, &fnodes, &symbols, target),
            (&source, &nodes, &fenv, target),
            (&source, &nodes, &symbols, discharged),
            (&source, &nodes, &symbols, VcId::new(vcs.vcs().len())),
        ] {
            let error = check(
                s,
                n,
                e,
                snapshot,
                selected,
                (profile.clone(), format),
                &policy,
            )
            .unwrap_err();
            assert_eq!(
                error,
                old(
                    s,
                    n,
                    e,
                    snapshot,
                    selected,
                    (profile.clone(), format),
                    &policy
                )
                .unwrap_err()
            );
            assert_eq!(
                error,
                select(
                    s,
                    n,
                    e,
                    snapshot,
                    selected,
                    (profile.clone(), format),
                    &policy
                )
                .unwrap_err()
            );
        }
        let unsupported = LogicProfile::try_new(
            "unsupported",
            LogicFragment::TffLike,
            EqualitySupport::Supported,
            QuantifierPolicy::FirstOrder,
            SoftTypeStrategy::GuardPredicates,
            NativePropertySupport::Unsupported,
            [format].into_iter().collect(),
        )
        .unwrap();
        for encoding in [
            (unsupported, format),
            (profile.clone(), ConcreteFormat::SmtLib),
        ] {
            assert!(
                check(
                    &source,
                    &nodes,
                    &symbols,
                    snapshot,
                    target,
                    encoding.clone(),
                    &policy
                )
                .is_err()
            );
            assert!(
                old(
                    &source, &nodes, &symbols, snapshot, target, encoding, &policy
                )
                .is_err()
            );
        }
        for denied in [
            policy.clone().with_kernel_evidence_formats([]),
            policy
                .clone()
                .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]),
        ] {
            assert_eq!(
                check(
                    &source,
                    &nodes,
                    &symbols,
                    snapshot,
                    target,
                    (profile.clone(), format),
                    &denied
                )
                .unwrap_err(),
                "registration.proof.kernel_policy_rejected"
            );
            assert!(
                select(
                    &source,
                    &nodes,
                    &symbols,
                    snapshot,
                    target,
                    (profile.clone(), format),
                    &denied
                )
                .is_err()
            );
        }
        let builtin_only = policy
            .clone()
            .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]);
        assert_eq!(
            builtin(&source, &nodes, &symbols, snapshot, &builtin_only)
                .unwrap()
                .activated()
                .len(),
            if multiple { 3 } else { 1 }
        );
        let formula_only =
            policy.with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]);
        let (_, decision, metadata, _) = check(
            &source,
            &nodes,
            &symbols,
            snapshot,
            target,
            (profile.clone(), format),
            &formula_only,
        )
        .unwrap();
        assert_eq!(decision.class, CandidatePolicyClass::KernelVerified);
        let selected = select(
            &source,
            &nodes,
            &symbols,
            snapshot,
            target,
            (profile, format),
            &formula_only,
        )
        .unwrap();
        assert_eq!(selected.selected_class(), ProofWinnerClass::KernelVerified);
        assert_eq!(
            selected.reuse_metadata().selected_evidence_hash(),
            Some(metadata.unwrap().accepted_result_hash.digest)
        );
        assert!(builtin(&source, &nodes, &symbols, snapshot, &formula_only).is_err());
        if !multiple {
            let unsupported = text.replacen("take the set;", "take the wmarked set;", 1);
            assert_ne!(unsupported, text);
            let (s, n, e) = step5c3_registration_inputs(&unsupported).unwrap();
            let encoding = (
                source_candidate_profile("source-metadata-negative", format),
                format,
            );
            assert!(
                check(
                    &s,
                    &n,
                    &e,
                    snapshot,
                    target,
                    encoding.clone(),
                    &VerifierPolicy::release()
                )
                .is_err()
            );
            assert!(
                old(
                    &s,
                    &n,
                    &e,
                    snapshot,
                    target,
                    encoding,
                    &VerifierPolicy::release()
                )
                .is_err()
            );
        }
    }
}
#[test]
fn source_registration_candidate_witness_projects_genuine_payload_selection_and_normal_reference() {
    use mizar_artifact::{
        proof_witness::{
            EvidenceKind, ProofStatus, ProofWitnessReadOptions, builtin_proof_witness_ref_json,
            proof_witness_ref_json, read_builtin_proof_witness_ref, read_proof_witness_ref,
        },
        store::{SchemaVersion, canonical_json_bytes},
    };
    use mizar_atp::{
        backend::{BackendCandidatePayload, build_source_registration_candidate_inputs as build},
        problem::ConcreteFormat,
    };
    use mizar_kernel::checker::KernelCheckStatus;
    use mizar_proof::{
        policy::{AcceptedGoalPolarity, VerifierPolicy},
        selection::{ProofWinnerClass, ProofWitnessPublication},
        status::{
            check_source_registration_candidate_metadata as check,
            project_source_registration_candidate_witness as project,
            select_source_registration_candidate as select,
        },
    };
    use mizar_vc::{
        discharge::{DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_existential_registration,
        kernel_evidence_handoff::build_source_existential_kernel_handoff,
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion, VcStatus},
    };
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original.replace("marked", "retagged").replace("X", "Value");
        let mut identities = std::collections::BTreeSet::new();
        for text in [&original, &renamed] {
            let (source, nodes, symbols) = if multiple {
                step5c13_registration_inputs(text)
            } else {
                step5c3_registration_inputs(text)
            }
            .unwrap();
            let before = (source.clone(), nodes.clone(), symbols.clone());
            let (core, _, _) = source_candidate_core_and_vcs(multiple, text);
            for snapshot_index in 0..2 {
                let snapshot = super::super::shared::snapshot_id(snapshot_index);
                let open = generate_source_existential_registration(
                    &core,
                    snapshot,
                    &GenerationSchemaVersion::new("source-existential-registration-v1"),
                    &VcSchemaVersion::new("vc-v1"),
                )
                .unwrap();
                let discharged = try_discharge(DischargeInput {
                    vc_set: &open,
                    policy: &DischargePolicy::default(),
                })
                .unwrap();
                let vcs = discharged.vc_set();
                for vc in vcs
                    .vcs()
                    .iter()
                    .filter(|vc| vc.status == VcStatus::NeedsAtp)
                {
                    let handoff =
                        build_source_existential_kernel_handoff(&core, vcs, vc.id).unwrap();
                    let wire = source_candidate_wire(&handoff);
                    for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
                        let profile = source_candidate_profile("source-witness", format);
                        let policy = VerifierPolicy::release();
                        let (encoded, actual) =
                            build(&core, vcs, vc.id, profile.clone(), format).unwrap();
                        assert_eq!(
                            source_candidate_check_bytes(&handoff, &wire, encoded.target_binding())
                                .unwrap(),
                            KernelCheckStatus::Accepted
                        );
                        let (_, _, metadata, candidate) = check(
                            &source,
                            &nodes,
                            &symbols,
                            snapshot,
                            vc.id,
                            (profile.clone(), format),
                            &policy,
                        )
                        .unwrap();
                        assert_eq!(candidate, actual);
                        let output = project(
                            &source,
                            &nodes,
                            &symbols,
                            snapshot,
                            vc.id,
                            (profile.clone(), format),
                            &policy,
                        )
                        .unwrap();
                        let (selection, reference, bytes) = &output;
                        assert_eq!(bytes, &wire);
                        assert_eq!(
                            actual.payload(),
                            &BackendCandidatePayload::FormulaSubstitutionBytes(bytes.clone())
                        );
                        assert_eq!(reference.kernel_acceptance, metadata.unwrap());
                        assert_eq!(
                            selection,
                            &select(
                                &source,
                                &nodes,
                                &symbols,
                                snapshot,
                                vc.id,
                                (profile.clone(), format),
                                &policy
                            )
                            .unwrap()
                        );
                        assert_eq!(selection.selected_class(), ProofWinnerClass::KernelVerified);
                        assert_eq!(
                            selection.selected_candidate_id().unwrap().as_str(),
                            actual.candidate_id()
                        );
                        assert_eq!(reference.obligation_id, actual.candidate_id());
                        assert!(identities.insert(reference.obligation_id.clone()));
                        let reuse = selection.reuse_metadata();
                        assert_eq!(reuse.encoded_problem_hash(), Some(encoded.input_hash()));
                        assert_eq!(
                            reuse.selected_candidate_provenance_hash(),
                            Some(actual.provenance_hash())
                        );
                        assert_eq!(
                            reuse.selected_evidence_hash(),
                            Some(reference.kernel_acceptance.accepted_result_hash.digest)
                        );
                        assert_eq!(
                            reuse.accepted_goal_polarity(),
                            Some(AcceptedGoalPolarity::AssertFalseForRefutation)
                        );
                        assert_eq!(reuse.selected_proof_witness_hash(), None);
                        assert_eq!(reuse.deterministic_discharge_hash(), None);
                        assert_eq!(
                            reuse.proof_witness_publication(),
                            ProofWitnessPublication::NotApplicable
                        );
                        assert_eq!(reference.schema_version, SchemaVersion::new(2, 0));
                        assert_eq!(
                            reference.kernel_acceptance.evidence_schema_version,
                            SchemaVersion::new(2, 0)
                        );
                        assert_eq!(reference.proof_status, ProofStatus::KernelVerified);
                        assert_eq!(
                            reference.evidence_kind,
                            EvidenceKind::FormulaSubstitutionKernelEvidence
                        );
                        let json = proof_witness_ref_json(reference).unwrap();
                        let decoded = read_proof_witness_ref(
                            &json,
                            ProofWitnessReadOptions {
                                expected_witness_artifact_hash: Some(
                                    &reference.witness_artifact_hash,
                                ),
                                ..Default::default()
                            },
                        )
                        .unwrap();
                        assert_eq!(&decoded, reference);
                        assert_eq!(
                            canonical_json_bytes(&json),
                            canonical_json_bytes(&proof_witness_ref_json(&decoded).unwrap())
                        );
                        assert!(builtin_proof_witness_ref_json(reference).is_err());
                        assert!(read_builtin_proof_witness_ref(&json, Default::default()).is_err());
                        assert_eq!(
                            project(
                                &source,
                                &nodes,
                                &symbols,
                                snapshot,
                                vc.id,
                                (profile, format),
                                &policy
                            )
                            .unwrap(),
                            output
                        );
                    }
                }
            }
            assert_eq!((source, nodes, symbols), before);
        }
        assert_eq!(identities.len(), if multiple { 24 } else { 8 });
    }
}

#[test]
fn source_registration_candidate_witness_binds_all_seventeen_participants_and_full_name_bytes() {
    use mizar_artifact::{
        registration_summary::ArtifactHashClass,
        store::{CanonicalHashDomain, CanonicalJson, HashClass, SchemaVersion},
    };
    use mizar_atp::{
        backend::build_source_registration_candidate_inputs as build, problem::ConcreteFormat,
    };
    use mizar_proof::{
        policy::{AcceptedGoalPolarity, VerifierPolicy},
        status::{
            check_source_registration_candidate_metadata as check,
            project_source_registration_candidate_witness as project,
        },
    };
    use mizar_vc::{
        discharge::{DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_existential_registration,
        kernel_evidence_handoff::build_source_existential_kernel_handoff,
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion, VcStatus},
    };
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original.replace("marked", "retagged").replace("X", "Value");
        let mut seen = std::collections::BTreeMap::new();
        for (source_index, text) in [&original, &renamed].into_iter().enumerate() {
            let (source, nodes, symbols) = if multiple {
                step5c13_registration_inputs(text)
            } else {
                step5c3_registration_inputs(text)
            }
            .unwrap();
            let (core, _, _) = source_candidate_core_and_vcs(multiple, text);
            for snapshot_index in 0..2 {
                let snapshot = super::super::shared::snapshot_id(snapshot_index);
                let open = generate_source_existential_registration(
                    &core,
                    snapshot,
                    &GenerationSchemaVersion::new("source-existential-registration-v1"),
                    &VcSchemaVersion::new("vc-v1"),
                )
                .unwrap();
                let discharged = try_discharge(DischargeInput {
                    vc_set: &open,
                    policy: &DischargePolicy::default(),
                })
                .unwrap();
                let vcs = discharged.vc_set();
                for vc in vcs
                    .vcs()
                    .iter()
                    .filter(|vc| vc.status == VcStatus::NeedsAtp)
                {
                    let handoff =
                        build_source_existential_kernel_handoff(&core, vcs, vc.id).unwrap();
                    let wire = source_candidate_wire(&handoff);
                    let without_names = handoff.encode_local_evidence(None).unwrap();
                    assert_ne!(wire, without_names);
                    assert_ne!(handoff.context_identity_hash(), handoff.canonical_hash());
                    for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
                        for (profile_index, name) in ["source-witness", "source-witness-profile"]
                            .into_iter()
                            .enumerate()
                        {
                            let profile = source_candidate_profile(name, format);
                            let (encoded, actual) =
                                build(&core, vcs, vc.id, profile.clone(), format).unwrap();
                            for (policy_index, policy) in [
                                VerifierPolicy::release(),
                                VerifierPolicy::development(),
                                VerifierPolicy::release().with_schema_version(2),
                                VerifierPolicy::release().with_checker_schema_version(7),
                                VerifierPolicy::release()
                                    .with_profile_id("configured-source-witness"),
                            ]
                            .into_iter()
                            .enumerate()
                            {
                                let (result, _, m, _) = check(
                                    &source,
                                    &nodes,
                                    &symbols,
                                    snapshot,
                                    vc.id,
                                    (profile.clone(), format),
                                    &policy,
                                )
                                .unwrap();
                                let m = m.unwrap();
                                let (_, reference, bytes) = project(
                                    &source,
                                    &nodes,
                                    &symbols,
                                    snapshot,
                                    vc.id,
                                    (profile.clone(), format),
                                    &policy,
                                )
                                .unwrap();
                                assert_eq!(bytes, wire);
                                assert_eq!(reference.kernel_acceptance, m);
                                assert!(m.formula_context_hash.is_none());
                                let hex =
                                    bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
                                let payload_domain = CanonicalHashDomain::new(
                                    HashClass::Artifact,
                                    "mizar-proof/source-registration-candidate-witness-payload",
                                    SchemaVersion::new(1, 0),
                                );
                                let payload = &reference.witness_artifact_hash;
                                assert_eq!(payload.class, ArtifactHashClass::Artifact);
                                assert_eq!(
                                    payload.schema_family,
                                    "mizar-proof/source-registration-candidate-witness-payload"
                                );
                                assert_eq!(payload.schema_version, SchemaVersion::new(1, 0));
                                assert_eq!(
                                    payload.digest,
                                    payload_domain.hash(&CanonicalJson::string(hex), &[])
                                );
                                let fields = [
                                    (
                                        "checked_target_bytes",
                                        CanonicalJson::string(
                                            result
                                                .target_vc_fingerprint()
                                                .sort_bytes()
                                                .iter()
                                                .map(|b| format!("{b:02x}"))
                                                .collect::<String>(),
                                        ),
                                    ),
                                    (
                                        "local_context_identity_hash",
                                        CanonicalJson::string(
                                            handoff
                                                .context_identity_hash()
                                                .as_bytes()
                                                .iter()
                                                .map(|b| format!("{b:02x}"))
                                                .collect::<String>(),
                                        ),
                                    ),
                                    ("candidate_id", CanonicalJson::string(actual.candidate_id())),
                                    (
                                        "encoded_problem_hash",
                                        CanonicalJson::string(
                                            encoded
                                                .input_hash()
                                                .as_bytes()
                                                .iter()
                                                .map(|b| format!("{b:02x}"))
                                                .collect::<String>(),
                                        ),
                                    ),
                                    (
                                        "atp_provenance_hash",
                                        CanonicalJson::string(
                                            encoded
                                                .provenance_hash()
                                                .as_bytes()
                                                .iter()
                                                .map(|b| format!("{b:02x}"))
                                                .collect::<String>(),
                                        ),
                                    ),
                                    (
                                        "kernel_profile_fingerprint",
                                        CanonicalJson::string(
                                            m.kernel_profile_fingerprint.to_artifact_hash_string(),
                                        ),
                                    ),
                                    (
                                        "verifier_policy_fingerprint",
                                        CanonicalJson::string(
                                            m.verifier_policy_fingerprint.to_artifact_hash_string(),
                                        ),
                                    ),
                                    (
                                        "checker_schema_version",
                                        CanonicalJson::string(m.checker_schema_version.to_string()),
                                    ),
                                    (
                                        "evidence_schema_version",
                                        CanonicalJson::string(
                                            m.evidence_schema_version.to_string(),
                                        ),
                                    ),
                                    (
                                        "target_binding_hash",
                                        CanonicalJson::string(
                                            m.target_binding_hash.to_artifact_hash_string(),
                                        ),
                                    ),
                                    (
                                        "formula_evidence_hash",
                                        CanonicalJson::string(
                                            m.formula_evidence_hash.to_artifact_hash_string(),
                                        ),
                                    ),
                                    (
                                        "substitution_evidence_hash",
                                        CanonicalJson::string(
                                            m.substitution_evidence_hash.to_artifact_hash_string(),
                                        ),
                                    ),
                                    (
                                        "provenance_hash",
                                        CanonicalJson::string(
                                            m.provenance_hash.to_artifact_hash_string(),
                                        ),
                                    ),
                                    ("formula_context_hash", CanonicalJson::null()),
                                    (
                                        "accepted_result_hash",
                                        CanonicalJson::string(
                                            m.accepted_result_hash.to_artifact_hash_string(),
                                        ),
                                    ),
                                    (
                                        "accepted_goal_polarity",
                                        CanonicalJson::string(
                                            AcceptedGoalPolarity::AssertFalseForRefutation.as_str(),
                                        ),
                                    ),
                                    (
                                        "witness_artifact_hash",
                                        CanonicalJson::string(payload.to_artifact_hash_string()),
                                    ),
                                ];
                                let obligation = CanonicalJson::object(fields).unwrap();
                                let domain = CanonicalHashDomain::new(
                                    HashClass::Interface,
                                    "mizar-proof/source-registration-candidate-obligation",
                                    SchemaVersion::new(1, 0),
                                );
                                assert_eq!(
                                    reference.obligation_fingerprint.class,
                                    ArtifactHashClass::Interface
                                );
                                assert_eq!(
                                    reference.obligation_fingerprint.schema_family,
                                    "mizar-proof/source-registration-candidate-obligation"
                                );
                                assert_eq!(
                                    reference.obligation_fingerprint.schema_version,
                                    SchemaVersion::new(1, 0)
                                );
                                assert_eq!(
                                    reference.obligation_fingerprint.digest,
                                    domain.hash(&obligation, &[])
                                );
                                assert_eq!(
                                    reference.witness_path,
                                    format!(
                                        "proof-witnesses/source-registration-candidate-v1/{}.bin",
                                        payload
                                            .digest
                                            .as_bytes()
                                            .iter()
                                            .map(|b| format!("{b:02x}"))
                                            .collect::<String>()
                                    )
                                );
                                if policy_index > 0 {
                                    let prior: &mizar_artifact::proof_witness::ProofWitnessRef =
                                        &seen[&(
                                            source_index,
                                            snapshot_index,
                                            vc.id.index(),
                                            format,
                                            profile_index,
                                            0,
                                        )];
                                    assert_eq!(payload, &prior.witness_artifact_hash);
                                    assert_ne!(
                                        reference.obligation_fingerprint,
                                        prior.obligation_fingerprint
                                    );
                                }
                                if profile_index > 0 {
                                    let prior = &seen[&(
                                        source_index,
                                        snapshot_index,
                                        vc.id.index(),
                                        format,
                                        0,
                                        policy_index,
                                    )];
                                    assert_eq!(payload, &prior.witness_artifact_hash);
                                    assert_ne!(reference.obligation_id, prior.obligation_id);
                                    assert_ne!(
                                        reference.obligation_fingerprint,
                                        prior.obligation_fingerprint
                                    );
                                }
                                if snapshot_index > 0 {
                                    let prior = &seen[&(
                                        source_index,
                                        0,
                                        vc.id.index(),
                                        format,
                                        profile_index,
                                        policy_index,
                                    )];
                                    assert_ne!(payload, &prior.witness_artifact_hash);
                                    assert_ne!(
                                        reference.obligation_fingerprint,
                                        prior.obligation_fingerprint
                                    );
                                }
                                if source_index > 0 {
                                    let prior = &seen[&(
                                        0,
                                        snapshot_index,
                                        vc.id.index(),
                                        format,
                                        profile_index,
                                        policy_index,
                                    )];
                                    assert_ne!(payload, &prior.witness_artifact_hash);
                                    assert_ne!(
                                        reference.obligation_fingerprint,
                                        prior.obligation_fingerprint
                                    );
                                }
                                seen.insert(
                                    (
                                        source_index,
                                        snapshot_index,
                                        vc.id.index(),
                                        format,
                                        profile_index,
                                        policy_index,
                                    ),
                                    reference,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn source_registration_candidate_witness_preserves_upstream_failures_and_real_rejected_leaves() {
    use mizar_atp::problem::*;
    use mizar_kernel::checker::KernelCheckStatus;
    use mizar_proof::{
        policy::{CandidatePolicyClass, KernelEvidenceFormat, VerifierPolicy},
        status::{
            check_source_registration_candidate_metadata as check,
            project_source_registration_candidate_witness as project,
            prove_source_existential_registration as builtin,
        },
    };
    use mizar_vc::vc_ir::{VcId, VcStatus};
    let snapshot = super::super::shared::snapshot_id(0);
    let policy = VerifierPolicy::release();
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let foreign = text.replace("marked", "foreignmarked");
        let (source, nodes, symbols) = if multiple {
            step5c13_registration_inputs(&text)
        } else {
            step5c3_registration_inputs(&text)
        }
        .unwrap();
        let (fs, fnodes, fenv) = if multiple {
            step5c13_registration_inputs(&foreign)
        } else {
            step5c3_registration_inputs(&foreign)
        }
        .unwrap();
        let (_, _, vcs) = source_candidate_core_and_vcs(multiple, &text);
        let target = vcs
            .vcs()
            .iter()
            .find(|vc| vc.status == VcStatus::NeedsAtp)
            .unwrap()
            .id;
        let discharged = vcs
            .vcs()
            .iter()
            .find(|vc| matches!(vc.status, VcStatus::Discharged { .. }))
            .unwrap()
            .id;
        let format = ConcreteFormat::Tptp;
        let profile = source_candidate_profile("source-witness-negative", format);
        for (s, n, e, id) in [
            (&fs, &nodes, &symbols, target),
            (&source, &fnodes, &symbols, target),
            (&source, &nodes, &fenv, target),
            (&source, &nodes, &symbols, discharged),
            (&source, &nodes, &symbols, VcId::new(vcs.vcs().len())),
        ] {
            assert_eq!(
                project(s, n, e, snapshot, id, (profile.clone(), format), &policy).unwrap_err(),
                check(s, n, e, snapshot, id, (profile.clone(), format), &policy).unwrap_err()
            );
        }
        let unsupported = LogicProfile::try_new(
            "unsupported",
            LogicFragment::TffLike,
            EqualitySupport::Supported,
            QuantifierPolicy::FirstOrder,
            SoftTypeStrategy::GuardPredicates,
            NativePropertySupport::Unsupported,
            [format].into_iter().collect(),
        )
        .unwrap();
        for encoding in [
            (unsupported, format),
            (profile.clone(), ConcreteFormat::SmtLib),
        ] {
            assert_eq!(
                project(
                    &source,
                    &nodes,
                    &symbols,
                    snapshot,
                    target,
                    encoding.clone(),
                    &policy
                )
                .unwrap_err(),
                check(
                    &source, &nodes, &symbols, snapshot, target, encoding, &policy
                )
                .unwrap_err()
            );
        }
        for denied in [
            policy.clone().with_kernel_evidence_formats([]),
            policy
                .clone()
                .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]),
        ] {
            let error = project(
                &source,
                &nodes,
                &symbols,
                snapshot,
                target,
                (profile.clone(), format),
                &denied,
            )
            .unwrap_err();
            assert_eq!(
                error,
                check(
                    &source,
                    &nodes,
                    &symbols,
                    snapshot,
                    target,
                    (profile.clone(), format),
                    &denied
                )
                .unwrap_err()
            );
            assert_eq!(error, "registration.proof.kernel_policy_rejected");
        }
        let builtin_only = policy
            .clone()
            .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]);
        assert_eq!(
            builtin(&source, &nodes, &symbols, snapshot, &builtin_only)
                .unwrap()
                .activated()
                .len(),
            if multiple { 3 } else { 1 }
        );
        let formula_only = policy
            .clone()
            .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]);
        assert!(
            project(
                &source,
                &nodes,
                &symbols,
                snapshot,
                target,
                (profile.clone(), format),
                &formula_only
            )
            .is_ok()
        );
        assert!(builtin(&source, &nodes, &symbols, snapshot, &formula_only).is_err());
        if !multiple {
            let unsupported = text.replacen("take the set;", "take the wmarked set;", 1);
            assert_ne!(unsupported, text);
            let (s, n, e) = step5c3_registration_inputs(&unsupported).unwrap();
            assert_eq!(
                project(
                    &s,
                    &n,
                    &e,
                    snapshot,
                    target,
                    (profile.clone(), format),
                    &policy
                )
                .unwrap_err(),
                check(
                    &s,
                    &n,
                    &e,
                    snapshot,
                    target,
                    (profile.clone(), format),
                    &policy
                )
                .unwrap_err()
            );
        } else {
            let negative = text.replacen("means X = X", "means not X = X", 1);
            assert_ne!(negative, text);
            let (s, n, e) = step5c13_registration_inputs(&negative).unwrap();
            let (_, _, vcs) = source_candidate_core_and_vcs(true, &negative);
            let mut counts = [0, 0];
            let mut accepted = None;
            let mut rejected = std::collections::BTreeSet::new();
            for vc in vcs
                .vcs()
                .iter()
                .filter(|vc| vc.status == VcStatus::NeedsAtp)
            {
                let (result, decision, metadata, actual) = check(
                    &s,
                    &n,
                    &e,
                    snapshot,
                    vc.id,
                    (profile.clone(), format),
                    &policy,
                )
                .unwrap();
                let projected = project(
                    &s,
                    &n,
                    &e,
                    snapshot,
                    vc.id,
                    (profile.clone(), format),
                    &policy,
                );
                if vc.id.index() == 3 {
                    assert_eq!(result.status(), KernelCheckStatus::Accepted);
                    assert_eq!(decision.class, CandidatePolicyClass::KernelVerified);
                    let (selection, reference, _) = projected.unwrap();
                    assert_eq!(reference.kernel_acceptance, metadata.unwrap());
                    assert_eq!(reference.obligation_id, actual.candidate_id());
                    assert_eq!(
                        selection.selected_candidate_id().unwrap().as_str(),
                        actual.candidate_id()
                    );
                    accepted = Some(reference.obligation_id);
                    counts[0] += 1;
                } else {
                    assert_eq!(result.status(), KernelCheckStatus::Rejected);
                    assert_eq!(decision.class, CandidatePolicyClass::KernelRejected);
                    assert!(metadata.is_none());
                    assert_eq!(
                        projected.unwrap_err(),
                        "registration.proof.invalid_source_candidate"
                    );
                    rejected.insert(actual.candidate_id().to_owned());
                    counts[1] += 1;
                }
            }
            assert_eq!(counts, [1, 2]);
            assert!(!rejected.contains(&accepted.unwrap()));
        }
    }
}

#[test]
fn source_registration_owner_exact_frames_bind_actual_single_and_aggregate_symbols() {
    use mizar_resolve::{env::SymbolKind, symbols::{source_registration_owner_payloads as owners,
        validate_source_symbol_env}};
    for (multiple, case) in [(false, step5c3_registration_case()), (true, step5c13_registration_case())] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (frontend, source, collected) = source_registration_owner_inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        let before = collected.clone();
        validate_source_symbol_env(&source, collected.env()).unwrap();
        let payloads = owners(ast, &source, collected.env()).unwrap();
        let expected: Vec<(u8, &str, &[u8])> = if multiple {
            vec![
                (1, "oamarked", b"\0\0\0\x06OAADef\0\0\0\x08oamarked"),
                (1, "obmarked", b"\0\0\0\x06OBBDef\0\0\0\x08obmarked"),
                (2, "OAExists", b"\0\0\0\x08OAExists"),
                (2, "OBExists", b"\0\0\0\x08OBExists"),
                (2, "OABExists", b"\0\0\0\x09OABExists"),
            ]
        } else {
            vec![
                (1, "wmarked", b"\0\0\0\x05WMDef\0\0\0\x07wmarked"),
                (2, "WMarkedExists", b"\0\0\0\x0dWMarkedExists"),
            ]
        };
        assert_eq!(payloads.len(), expected.len());
        for (kind, spelling, tail) in expected {
            let entry = collected.env().symbols().iter().find(|entry|
                entry.kind() == if kind == 1 { SymbolKind::Attribute } else { SymbolKind::Registration }
                && entry.primary_spelling() == spelling).unwrap();
            let mut bytes = b"mizar-resolve/source-registration-owner/v1\0".to_vec();
            bytes.push(kind);
            bytes.extend_from_slice(b"\0\0\0\x0aowner-test\0\0\0\x06owners");
            bytes.extend_from_slice(tail);
            assert_eq!(payloads.get(entry.symbol()), Some(&bytes));
        }
        assert!(collected.env().symbols().iter().any(|entry| entry.kind() == SymbolKind::Functor));
        assert_eq!(collected, before);
        validate_source_symbol_env(&source, collected.env()).unwrap();
    }
    for text in ["", "definition\n let X be set;\n func Box: box X -> set equals X;\n coherence;\nend;\n"] {
        let (frontend, source, collected) = source_registration_owner_inputs(text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        assert!(owners(frontend.ast.as_ref().unwrap(), &source, collected.env()).unwrap().is_empty());
    }
}

#[test]
fn source_registration_owner_payloads_survive_real_relocation_and_owner_neutral_edits() {
    use mizar_resolve::{env::SymbolKind, symbols::source_registration_owner_payloads as owners};
    let attribute_a = "  attr OAADef: X is oamarked means X = X;\n";
    let attribute_b = "  attr OBBDef: X is obmarked means X = X;\n";
    let registration_a = "  cluster OAExists: oamarked set;\n  existence proof take the set; thus the set is oamarked by OAADef; end;\n";
    let registration_b = "  cluster OBExists: obmarked set;\n  existence proof take the set; thus the set is obmarked by OBBDef; end;\n";
    let definitions = format!("definition\n  let X be set;\n{attribute_a}{attribute_b}end;\n");
    let registrations = format!("registration\n{registration_a}{registration_b}end;\n");
    let text = format!("{definitions}\n{registrations}");
    let (frontend, source, collected) = source_registration_owner_inputs(&text, "owner-test", "owners", 0);
    assert!(frontend.diagnostics.is_empty());
    assert!(collected.diagnostics().is_empty());
    let baseline = owners(frontend.ast.as_ref().unwrap(), &source, collected.env()).unwrap();
    assert_eq!(baseline.len(), 4);
    let expected: [(SymbolKind, &str, u8, &[u8]); 4] = [
        (SymbolKind::Attribute, "oamarked", 1, b"\0\0\0\x06OAADef\0\0\0\x08oamarked"),
        (SymbolKind::Attribute, "obmarked", 1, b"\0\0\0\x06OBBDef\0\0\0\x08obmarked"),
        (SymbolKind::Registration, "OAExists", 2, b"\0\0\0\x08OAExists"),
        (SymbolKind::Registration, "OBExists", 2, b"\0\0\0\x08OBExists"),
    ];
    let reordered = format!("definition\n  let X be set;\n{attribute_b}{attribute_a}end;\n\nregistration\n{registration_b}{registration_a}end;\n");
    let separate_blocks = format!("definition\n let X be set;\n{attribute_b}end;\ndefinition\n let X be set;\n{attribute_a}end;\nregistration\n{registration_b}end;\nregistration\n{registration_a}end;\n");
    for (index, edited) in [
        format!("\n:: leading trivia shifts every range\n{}", text.replace("  ", "    ")),
        text.replace('X', "RenamedBinder"),
        reordered,
        format!("definition\n let X be set;\n{attribute_a}end;\ndefinition\n let X be set;\n{attribute_b}end;\nregistration\n{registration_a}end;\nregistration\n{registration_b}end;\n"),
        separate_blocks,
        text.clone(),
    ].into_iter().enumerate() {
        let skip = if index == 5 { 7 } else { 0 };
        let (edited_frontend, edited_source, edited_collected) = source_registration_owner_inputs(&edited, "owner-test", "owners", skip);
        assert!(edited_frontend.diagnostics.is_empty(), "edit {index}");
        assert!(edited_collected.diagnostics().is_empty(), "edit {index}");
        let changed = owners(edited_frontend.ast.as_ref().unwrap(), &edited_source, edited_collected.env()).unwrap();
        assert_eq!(changed.len(), expected.len(), "edit {index}");
        for (kind, spelling, tag, tail) in expected {
            let entries = edited_collected.env().symbols().iter().filter(|entry|
                entry.kind() == kind && entry.primary_spelling() == spelling).collect::<Vec<_>>();
            assert_eq!(entries.len(), 1, "edit {index}: {spelling}");
            let bytes = [b"mizar-resolve/source-registration-owner/v1\0".as_slice(), &[tag],
                b"\0\0\0\x0aowner-test\0\0\0\x06owners", tail].concat();
            assert_eq!(changed.get(entries[0].symbol()), Some(&bytes), "edit {index}: {spelling}");
        }
        if index == 2 || index == 4 {
            assert_ne!(changed.keys().collect::<Vec<_>>(), baseline.keys().collect::<Vec<_>>());
        }
        if skip != 0 {
            assert_ne!(edited_source.source_id(), source.source_id());
        }
    }
    for (index, (edited, package, module, package_frame, module_frame)) in [
        (text.replace("OAADef", "RenamedLabel"), "owner-test", "owners", b"\0\0\0\x0aowner-test".as_slice(), b"\0\0\0\x06owners".as_slice()),
        (text.replace("oamarked", "renamedconstructor"), "owner-test", "owners", b"\0\0\0\x0aowner-test", b"\0\0\0\x06owners"),
        (text.clone(), "owner-test", "other_module", b"\0\0\0\x0aowner-test", b"\0\0\0\x0cother_module"),
        (text, "other-package", "owners", b"\0\0\0\x0dother-package", b"\0\0\0\x06owners"),
    ].into_iter().enumerate() {
        let (frontend, source, collected) = source_registration_owner_inputs(&edited, package, module, 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let changed = owners(frontend.ast.as_ref().unwrap(), &source, collected.env()).unwrap();
        assert_eq!(changed.len(), expected.len());
        for (kind, spelling, tag, tail) in expected {
            let (spelling, tail) = match (index, spelling) {
                (0, "oamarked") => (spelling, b"\0\0\0\x0cRenamedLabel\0\0\0\x08oamarked".as_slice()),
                (1, "oamarked") => ("renamedconstructor", b"\0\0\0\x06OAADef\0\0\0\x12renamedconstructor".as_slice()),
                _ => (spelling, tail),
            };
            let entries = collected.env().symbols().iter().filter(|entry|
                entry.kind() == kind && entry.primary_spelling() == spelling).collect::<Vec<_>>();
            assert_eq!(entries.len(), 1, "control {index}: {spelling}");
            let bytes = [b"mizar-resolve/source-registration-owner/v1\0".as_slice(), &[tag],
                package_frame, module_frame, tail].concat();
            assert_eq!(changed.get(entries[0].symbol()), Some(&bytes), "control {index}: {spelling}");
        }
    }
    for binder in ["n", "renamed"] {
        let text = format!("definition\n let {binder} be set;\n let X be set;\n attr ParamDef: X is {binder}-dimensional means X = X;\nend;\n");
        let (frontend, source, collected) = source_registration_owner_inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let payloads = owners(frontend.ast.as_ref().unwrap(), &source, collected.env()).unwrap();
        assert_eq!(payloads.len(), 1);
        let entries = collected.env().symbols().iter().filter(|entry|
            entry.kind() == SymbolKind::Attribute && entry.primary_spelling() == "dimensional").collect::<Vec<_>>();
        assert_eq!(entries.len(), 1);
        let payload = payloads.get(entries[0].symbol()).unwrap();
        assert_eq!(payload, b"mizar-resolve/source-registration-owner/v1\0\x01\0\0\0\x0aowner-test\0\0\0\x06owners\0\0\0\x08ParamDef\0\0\0\x0bdimensional");
    }
}

#[test]
fn source_registration_owner_refuses_foreign_missing_recovered_duplicate_and_unsupported_inputs() {
    use mizar_resolve::symbols::{source_registration_owner_payloads as owners, validate_source_symbol_env};
    let text = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let (frontend, source, collected) = source_registration_owner_inputs(&text, "owner-test", "owners", 0);
    let ast = frontend.ast.as_ref().unwrap();
    assert!(owners(ast, &source, collected.env()).is_ok());
    for (edited, package, skip) in [
        (text.replace("wmarked", "foreignmarked"), "owner-test", 0),
        (text.clone(), "foreign-package", 0),
        (text.clone(), "owner-test", 1),
    ] {
        let (foreign_frontend, foreign_source, foreign_collected) = source_registration_owner_inputs(&edited, package, "owners", skip);
        assert!(foreign_frontend.diagnostics.is_empty());
        assert!(foreign_collected.diagnostics().is_empty());
        let foreign_ast = foreign_frontend.ast.as_ref().unwrap();
        if let Err(error) = source.validate_against(foreign_ast, source.module()) {
            assert_eq!(owners(foreign_ast, &source, collected.env()).unwrap_err(), error.to_string());
        }
        if let Err(error) = foreign_source.validate_against(ast, foreign_source.module()) {
            assert_eq!(owners(ast, &foreign_source, collected.env()).unwrap_err(), error.to_string());
        } else {
            let error = validate_source_symbol_env(&foreign_source, collected.env()).unwrap_err();
            assert_eq!(owners(ast, &foreign_source, collected.env()).unwrap_err(), error);
        }
        let error = validate_source_symbol_env(&source, foreign_collected.env()).unwrap_err();
        assert_eq!(owners(ast, &source, foreign_collected.env()).unwrap_err(), error);
    }
    let unsupported = "unsupported source registration owner payload";
    for (edited, expected_optional_refusal) in [
        (text.replace("attr WMDef:", "attr :"), false),
        (text.replace("cluster WMarkedExists:", "cluster :"), false),
        (text.replace("means X = X", "means X ="), false),
        (text.replace("end;\n\nregistration", "  attr WMDef: X is othermarked means X = X;\nend;\n\nregistration"), true),
        (format!("{text}\nregistration\n cluster WMarkedExists: wmarked set;\n existence proof take the set; thus the set is wmarked by WMDef; end;\nend;\n"), false),
        (format!("{text}\nregistration\n cluster Conditional: wmarked -> wmarked for set;\n coherence;\nend;\n"), true),
    ] {
        let (bad_frontend, bad_source, bad_collected) = source_registration_owner_inputs(&edited, "owner-test", "owners", 0);
        let before = bad_collected.clone();
        let result = owners(bad_frontend.ast.as_ref().unwrap(), &bad_source, bad_collected.env());
        if expected_optional_refusal {
            assert!(bad_frontend.diagnostics.is_empty(), "{edited}");
            assert!(bad_collected.diagnostics().is_empty(), "{edited}");
            validate_source_symbol_env(&bad_source, bad_collected.env()).unwrap();
            assert_eq!(result.unwrap_err(), unsupported, "{edited}");
        } else {
            let error = validate_source_symbol_env(&bad_source, bad_collected.env()).unwrap_err();
            assert_eq!(result.unwrap_err(), error, "{edited}");
        }
        assert_eq!(bad_collected, before);
    }
}

#[test]
fn source_registration_formula_exact_current_roots_cover_six_shapes_and_real_choices() {
    use mizar_core::{core_ir::{CoreFormulaKind as F, CoreTermKind as T, DefinitionBody},
        elaborator::{lower_source_existential_registration as lower, source_registration_formula_payloads as payloads}};
    let frame = |value: &[u8]| [u32::try_from(value.len()).unwrap().to_be_bytes().as_slice(), value].concat();
    let free = b"\x02\0\0\0\0".as_slice();
    let bound = b"\x01\0\0\0\0".as_slice();
    let type_prefix = [b"\x03".as_slice(), &frame(b"builtin.set")].concat();
    let equality = [b"\x02".as_slice(), free, free].concat();
    for case in [step5c3_registration_case(), step5c13_registration_case()] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        for text in [text.clone(), text.replacen("means X = X", "means not X = X", 1)] {
            let (source, nodes, collected, owners) = source_registration_formula_inputs(&text, "formula-test", "formulas", 0);
            let check = mizar_checker::registration_resolution::check_source_existential_registration_proof(&source, &nodes, collected.env()).unwrap_or_else(|error| panic!("{error}: {text}"));
            let core = lower(&check).unwrap();
            let before = core.clone();
            let formal = core.definitions().iter().next().unwrap().1.params[0].var;
            let mut header = b"mizar-core/source-registration-formula/v1\0".to_vec();
            header.extend(frame(b"formula-test"));
            header.extend(frame(b"formulas"));
            header.extend(frame(b"definition-parameter"));
            let mut choices = std::collections::BTreeMap::new();
            for (_, origin) in core.generated().iter() {
                assert_eq!(origin.kind, mizar_core::core_ir::GeneratedOriginKind::StableChoice);
                assert!(origin.params.is_empty());
                assert_eq!(origin.key.as_str(), "choice:builtin.set");
                let bytes = [b"\x03".as_slice(), &frame(&owners[&core.items().get(origin.owner).unwrap().symbol]), &frame(b"choice:builtin.set")].concat();
                assert!(choices.insert(origin.functor.as_ref().unwrap(), bytes).is_none());
            }
            let term = |id| match &core.terms().get(id).unwrap().kind {
                T::Var(var) if *var == formal => Some(free.to_vec()),
                T::Apply { functor, args } => { assert!(args.is_empty()); Some(choices[functor].clone()) },
                T::Var(_) => None,
                other => panic!("unexpected genuine term {other:?}"),
            };
            let mut expected = std::collections::BTreeMap::new();
            let mut shapes = std::collections::BTreeSet::new();
            for (id, row) in core.formulas().iter() {
                let bytes = match &row.kind {
                    F::Equals { left, right } => {
                        assert_eq!(term(*left).unwrap(), free); assert_eq!(term(*right).unwrap(), free);
                        Some(equality.clone())
                    }
                    F::Not(child) => { assert_eq!(expected[child], equality); Some([b"\x04".as_slice(), &equality].concat()) },
                    F::TypePred { subject, ty } => {
                        assert_eq!(ty.as_str(), "builtin.set");
                        term(*subject).map(|subject| [type_prefix.as_slice(), &subject].concat())
                    }
                    F::Atom { predicate, args } => {
                        assert_eq!(args.len(), 1);
                        term(args[0]).map(|arg| [b"\x01".as_slice(), &frame(&owners[predicate]), b"\0\0\0\x01", &arg].concat())
                    }
                    F::And(children) => {
                        if children.iter().all(|child| expected.contains_key(child)) {
                            let mut conjunction = vec![5];
                            conjunction.extend(u32::try_from(children.len()).unwrap().to_be_bytes());
                            for child in children { conjunction.extend(&expected[child]); }
                            Some(conjunction)
                        } else { None }
                    }
                    F::Exists { binders, body } => {
                        assert_eq!(binders.len(), 1); assert!(binders[0].ty_guard.is_none());
                        let mut existential = b"\x06\0\0\0\x01".to_vec();
                        let children = match &core.formulas().get(*body).unwrap().kind {
                            F::TypePred { .. } => {
                                assert_eq!(binders[0].role.as_str(), "choice-nonempty");
                                existential.extend(frame(b"choice-nonempty")); existential.push(0);
                                vec![*body]
                            }
                            F::And(children) => {
                                assert_eq!(binders[0].role.as_str(), "registration-witness");
                                existential.extend(frame(b"registration-witness")); existential.push(0);
                                shapes.insert(5); existential.push(5); existential.extend(u32::try_from(children.len()).unwrap().to_be_bytes());
                                children.clone()
                            }
                            other => panic!("unexpected quantified body {other:?}"),
                        };
                        for child in children {
                            match &core.formulas().get(child).unwrap().kind {
                                F::TypePred { subject, ty } => {
                                    assert_eq!(ty.as_str(), "builtin.set");
                                    assert_eq!(core.terms().get(*subject).unwrap().kind, T::Var(binders[0].var));
                                    existential.extend(&type_prefix); existential.extend(bound);
                                }
                                F::Atom { predicate, args } => {
                                    assert_eq!(args.len(), 1);
                                    assert_eq!(core.terms().get(args[0]).unwrap().kind, T::Var(binders[0].var));
                                    existential.push(1); existential.extend(frame(&owners[predicate]));
                                    existential.extend(b"\0\0\0\x01"); existential.extend(bound);
                                }
                                other => panic!("unexpected bound child {other:?}"),
                            }
                        }
                        Some(existential)
                    }
                    other => panic!("unexpected genuine formula {other:?}"),
                };
                if let Some(bytes) = bytes { shapes.insert(bytes[0]); expected.insert(id, bytes); }
            }
            assert_eq!(shapes, if text.contains("means not") { [1,2,3,4,5,6].into_iter().collect() } else { [1,2,3,5,6].into_iter().collect() });
            let roots = expected.keys().copied().rev().collect::<Vec<_>>();
            let actual = payloads(&check, &core, &owners, &roots).unwrap();
            assert_eq!(actual.len(), roots.len());
            for (id, bytes) in &expected { assert_eq!(actual[id], [header.as_slice(), bytes].concat(), "current root {id:?}"); }
            assert_eq!(choices.values().collect::<std::collections::BTreeSet<_>>().len(), check.validations().len());
            for (_, definition) in core.definitions().iter() {
                let DefinitionBody::Formula(body) = definition.body else { unreachable!() };
                assert!(actual.contains_key(&body));
            }
            assert_eq!(core, before);
            assert_eq!(lower(&check).unwrap(), before);
        }
    }
}

#[test]
fn source_registration_formula_relocation_neutral_edits_and_participating_content() {
    use mizar_core::{core_ir::{CoreProofNodeKind, DefinitionBody, ObligationSeedKind},
        elaborator::{lower_source_existential_registration as lower, source_registration_formula_payloads as payloads}};
    let snapshot = |text: &str, package: &str, module: &str, skip| {
        let (source, nodes, collected, owners) = source_registration_formula_inputs(text, package, module, skip);
        let check = mizar_checker::registration_resolution::check_source_existential_registration_proof(&source, &nodes, collected.env()).unwrap_or_else(|error| panic!("{error}: {text}"));
        let core = lower(&check).unwrap();
        let spelling = |symbol: &mizar_resolve::resolved_ast::SymbolId| collected.env().symbols().iter().find(|entry| entry.symbol() == symbol).unwrap().primary_spelling().replace("renamedmarked", "oamarked").replace("RenamedExists", "OAExists");
        let mut roots = std::collections::BTreeMap::new();
        for (_, definition) in core.definitions().iter() {
            let DefinitionBody::Formula(body) = definition.body else { unreachable!() };
            let name = spelling(&definition.symbol);
            roots.insert(format!("{name}:body"), body);
            roots.insert(format!("{name}:guard"), definition.params[0].ty_guard.unwrap());
        }
        for (_, seed) in core.obligation_seeds().iter() {
            let name = spelling(&core.items().get(seed.owner).unwrap().symbol);
            let role = if seed.kind == ObligationSeedKind::CheckerInitial { "parent" } else { "nonempty" };
            roots.insert(format!("{name}:{role}"), seed.goal.unwrap());
        }
        for (_, proof) in core.proofs().iter() {
            let name = spelling(&core.items().get(proof.item).unwrap().symbol);
            let CoreProofNodeKind::CurrentGoal { child, .. } = core.proof_nodes().get(proof.root).unwrap().kind else { unreachable!() };
            let CoreProofNodeKind::Sequence { children } = &core.proof_nodes().get(child).unwrap().kind else { unreachable!() };
            for (index, child) in children.iter().enumerate() {
                if let CoreProofNodeKind::Step { formula, .. } = core.proof_nodes().get(*child).unwrap().kind {
                    roots.insert(format!("{name}:step{index}"), formula);
                }
            }
        }
        let ids = roots.values().copied().collect::<Vec<_>>();
        let actual = payloads(&check, &core, &owners, &ids).unwrap();
        let keyed = roots.into_iter().map(|(name, id)| { assert!(core.formulas().get(id).is_some()); (name, (id, actual[&id].clone())) }).collect::<std::collections::BTreeMap<_, _>>();
        (core.source_id(), keyed)
    };
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let baseline = snapshot(&text, "formula-test", "formulas", 0);
    let a = "  attr OAADef: X is oamarked means X = X;\n";
    let b = "  attr OBBDef: X is obmarked means X = X;\n";
    let registration_a = "  cluster OAExists: oamarked set;\n  existence proof\n    take the set;\n    thus the set is oamarked by OAADef;\n  end;\n";
    let registration_b = "  cluster OBExists: obmarked set;\n  existence proof\n    take the set;\n    thus the set is obmarked by OBBDef;\n  end;\n";
    let reordered = text.replace(&format!("{a}{b}"), &format!("{b}{a}"))
        .replace(&format!("{registration_a}{registration_b}"), &format!("{registration_b}{registration_a}"));
    let consumer_a = "definition\n  let X be oamarked set;\n  func Ov3Def: ovpick X -> set equals X;\n  coherence;\nend;\n";
    let consumer_b = "definition\n  let X be obmarked set;\n  func Ov4Def: ovpick X -> set equals X;\n  coherence;\nend;\n";
    let reordered_blocks = text.replace(&format!("{consumer_a}\n{consumer_b}"), &format!("{consumer_b}\n{consumer_a}"));
    assert_ne!(reordered_blocks, text);
    for (index, edited) in [format!("\n:: shifted ranges\n{}", text.replace("  ", "    ")), text.replace('X', "Subject"), reordered, reordered_blocks, text.clone()].into_iter().enumerate() {
        let edited = snapshot(&edited, "formula-test", "formulas", if index == 4 { 7 } else { 0 });
        assert_eq!(edited.1.keys().collect::<Vec<_>>(), baseline.1.keys().collect::<Vec<_>>());
        for (name, (_, bytes)) in &baseline.1 { assert_eq!(edited.1[name].1, *bytes, "edit {index}: {name}"); }
        if index == 2 { assert!(baseline.1.iter().any(|(name, (id, _))| edited.1[name].0 != *id)); }
        if index == 4 { assert_ne!(edited.0, baseline.0); }
    }
    for (index, edited, package, module) in [
        (0, text.replace("OAADef", "RenamedDef"), "formula-test", "formulas"),
        (1, text.replace("oamarked", "renamedmarked"), "formula-test", "formulas"),
        (2, text.replace("OAExists", "RenamedExists"), "formula-test", "formulas"),
        (3, text.replacen("means X = X", "means not X = X", 1), "formula-test", "formulas"),
        (4, text.clone(), "formula-test", "other_module"),
        (5, text.clone(), "other-package", "formulas"),
    ] {
        let edited = snapshot(&edited, package, module, 0);
        assert_eq!(edited.1.len(), baseline.1.len());
        for (name, (_, bytes)) in &baseline.1 {
            let changed = match index {
                0 | 1 => (name.starts_with("OAExists:") || name.starts_with("OABExists:")) && (name.ends_with(":parent") || name.ends_with(":step2")),
                2 => name.starts_with("OAExists:step"),
                3 => name == "oamarked:body",
                _ => true,
            };
            assert_eq!(edited.1[name].1 != *bytes, changed, "control {index}: {name}");
        }
    }
}

#[test]
fn source_registration_formula_refuses_foreign_incomplete_duplicate_and_unbound_inputs() {
    use mizar_core::{binder_normalization::{canonical_formula, normalize_core_formula, BinderContext,
        CanonicalFormulaKind, CanonicalTermKind, CanonicalVar, NormalizedVarClass, NormalizedVarSort},
        core_ir::{CoreFormulaId, CoreFormulaKind, DefinitionBody},
        elaborator::{lower_source_existential_registration as lower, source_registration_formula_payloads as payloads}};
    let text = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let (source, nodes, collected, owners) = source_registration_formula_inputs(&text, "formula-test", "formulas", 0);
    let check = mizar_checker::registration_resolution::check_source_existential_registration_proof(&source, &nodes, collected.env()).unwrap_or_else(|error| panic!("{error}: {text}"));
    let core = lower(&check).unwrap();
    let before = core.clone();
    let definition = core.definitions().iter().next().unwrap().1;
    let DefinitionBody::Formula(root) = definition.body else { unreachable!() };
    let unsupported = "unsupported source registration formula payload";
    assert!(payloads(&check, &core, &owners, &[]).unwrap().is_empty());
    for edited in [text.replace("wmarked", "foreignmarked"), text.replacen("means X = X", "means not X = X", 1), text.clone()] {
        let skip = usize::from(edited == text);
        let (s, n, e, o) = source_registration_formula_inputs(&edited, "formula-test", "formulas", skip);
        let foreign = mizar_checker::registration_resolution::check_source_existential_registration_proof(&s, &n, e.env()).unwrap();
        let foreign_core = lower(&foreign).unwrap();
        assert_ne!(foreign_core, core);
        assert_eq!(payloads(&check, &foreign_core, &owners, &[root]).unwrap_err(), unsupported);
        assert_eq!(payloads(&foreign, &core, &o, &[root]).unwrap_err(), unsupported);
    }
    let first = owners.keys().next().unwrap();
    let second = owners.keys().nth(1).unwrap();
    for index in 0..4 {
        let mut bad = owners.clone();
        match index {
            0 => { bad.remove(first); }
            1 => { let extra = collected.env().symbols().iter().find(|entry| !owners.contains_key(entry.symbol())).unwrap(); bad.insert(extra.symbol().clone(), b"extra".to_vec()); }
            2 => { bad.insert(first.clone(), Vec::new()); }
            _ => { bad.insert(first.clone(), owners[second].clone()); }
        }
        assert_eq!(payloads(&check, &core, &bad, &[root]).unwrap_err(), unsupported);
        assert_eq!(payloads(&check, &core, &bad, &[]).unwrap_err(), unsupported);
    }
    let unbound = core.formulas().iter().find_map(|(_, row)| match row.kind { CoreFormulaKind::Exists { body, .. } => Some(body), _ => None }).unwrap();
    for roots in [vec![root, root], vec![root, CoreFormulaId::new(usize::MAX)], vec![root, unbound]] {
        assert_eq!(payloads(&check, &core, &owners, &roots).unwrap_err(), unsupported);
    }
    let mut context = BinderContext::new();
    let formal = &definition.params[0];
    context.declare_variable(formal.var, NormalizedVarClass::Free, formal.role.clone(), NormalizedVarSort::Term);
    let row = core.formulas().get(root).unwrap();
    let normalized = normalize_core_formula(&core, root, &context, &row.source).unwrap();
    let canonical = canonical_formula(&normalized, &context, &row.source).unwrap();
    let CanonicalFormulaKind::Equals { left, right } = canonical.kind() else { unreachable!() };
    assert_eq!(left.kind(), &CanonicalTermKind::Var(CanonicalVar::Free(formal.var)));
    assert_eq!(left, right);
    assert!(canonical_formula(&normalized, &BinderContext::new(), &row.source).is_err());
    assert_eq!(core, before);
    assert_eq!(lower(&check).unwrap(), before);
}

#[test]
fn source_registration_vc_fingerprint_exact_current_owners_and_default_discharge() {
    use mizar_vc::{discharge::{try_discharge, DischargeInput, DischargePolicy},
        vc_ir::{VcGeneratedFormulaKind as K, VcGeneratedFormulaShape as S}};
    // Spec 6.2/17.2-17.3; independent semantic-literal frozen-format oracles.
    for (multiple, case) in [(false, step5c3_registration_case()), (true, step5c13_registration_case())] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (core, vcs, formulas, owners) = source_registration_vc_fingerprint_inputs(&text, "vc_fingerprints", 0);
        let before = (core.clone(), vcs.clone(), formulas.clone());
        let expected = if multiple { vec![
            ("OAExists:0", "e57355f39c54373d7f570560b13bc34cb66bb5e20c64407784c2b9cadbfbdb76"),
            ("OAExists:1", "e61d1df754e81f993807869ad9f1dabc604531797e75013b531fe4325a7ee63c"),
            ("OBExists:0", "e57355f39c54373d7f570560b13bc34cb66bb5e20c644077f69ff60695462458"),
            ("OBExists:1", "958f0230e4775dbc398c9b44b18b58eb21aa589938602ff532a0bc0b34a3e970"),
            ("OABExists:0", "29f8bc55904087b67c6f415370ca447dd25d45048f2afc39f4baef84c142f365"),
            ("OABExists:1", "846cd428c1d02683d533f7f322f3b85cff086d3460df586f24a0ea97e7bccc77"),
        ] } else { vec![
            ("WMarkedExists:0", "c1a1b732458066ba3c57c63552f444880bb9ee4b8e40a414670bd5363fdb31c1"),
            ("WMarkedExists:1", "753ab31d072228bd91586ae1000c9b1e9b662e6df3e9689f7d68f654db091eb7"),
        ] };
        let actual = vcs.source_registration_canonical_fingerprints(&core, &formulas).unwrap();
        assert_eq!(actual.len(), expected.len());
        assert_eq!(actual.keys().copied().collect::<Vec<_>>(), vcs.vcs().iter().map(|vc| vc.id).collect::<Vec<_>>());
        for (id, fingerprint) in &actual {
            let owner = &owners[id];
            let expected = expected.iter().find(|(name, _)| name == owner).unwrap().1;
            let hex = fingerprint.hash().as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
            assert_eq!(hex, expected, "{owner}");
        }
        assert!(formulas.values().all(|bytes| !bytes.is_empty()));
        if multiple { assert!(formulas.iter().any(|(a, bytes)| formulas.iter().any(|(b, other)| a != b && bytes == other))); }
        let kinds = vcs.generated_formulas().iter().map(|row| match row.kind {
            K::GeneratedTypeObligation => 1, K::NegatedPremise => 2, K::Conjunction => 3,
            ref other => panic!("unexpected genuine kind {other:?}"),
        }).collect::<std::collections::BTreeSet<_>>();
        let shapes = vcs.generated_formulas().iter().map(|row| match row.shape {
            S::Ref(_) => 1, S::Not(_) => 2, S::And(_) => 3, S::Or(_) => 4,
            ref other => panic!("unexpected genuine shape {other:?}"),
        }).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(kinds, [1, 2, 3].into_iter().collect());
        assert_eq!(shapes, [1, 2, 3, 4].into_iter().collect());
        let discharged = try_discharge(DischargeInput { vc_set: &vcs, policy: &DischargePolicy::default() }).unwrap();
        assert_ne!(discharged.vc_set(), &vcs);
        assert_eq!(discharged.vc_set().source_registration_canonical_fingerprints(&core, &formulas).unwrap(), actual);
        for vc in vcs.vcs() {
            assert!(vcs.canonical_vc_fingerprint(vc.id).is_none());
            assert!(!vc.anchor.is_complete());
        }
        assert_eq!((core, vcs, formulas), before);
    }
}

#[test]
fn source_registration_vc_fingerprint_relocation_order_and_content_participants() {
    use mizar_core::core_ir::CoreNodeRef;
    use mizar_vc::{generator::generate_source_existential_registration as generate,
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion}};
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let (core, vcs, formulas, owners) = source_registration_vc_fingerprint_inputs(&text, "vc_fingerprints", 0);
    let before = (core.clone(), vcs.clone(), formulas.clone());
    let fingerprints = vcs.source_registration_canonical_fingerprints(&core, &formulas).unwrap();
    let a = "  attr OAADef: X is oamarked means X = X;\n";
    let b = "  attr OBBDef: X is obmarked means X = X;\n";
    let registration_a = "  cluster OAExists: oamarked set;\n  existence proof\n    take the set;\n    thus the set is oamarked by OAADef;\n  end;\n";
    let registration_b = "  cluster OBExists: obmarked set;\n  existence proof\n    take the set;\n    thus the set is obmarked by OBBDef;\n  end;\n";
    let registration_ab = "  cluster OABExists: oamarked obmarked set;\n  existence proof\n    take the set;\n    thus the set is oamarked obmarked by OAADef, OBBDef;\n  end;\n";
    let reordered = text.replace(&format!("{a}{b}"), &format!("{b}{a}"))
        .replace(&format!("{registration_a}{registration_b}{registration_ab}"), &format!("{registration_ab}{registration_a}{registration_b}"));
    assert_ne!(reordered, text);
    for (index, edited, skip) in [
        (0, format!("\n:: shifted ranges\n{}", text.replace("  ", "    ")), 0),
        (1, text.replace('X', "Subject"), 0),
        (2, reordered, 0),
        (3, text.clone(), 7),
    ] {
        let (other_core, other_vcs, other_formulas, other_owners) = source_registration_vc_fingerprint_inputs(&edited, "vc_fingerprints", skip);
        let other_before = (other_core.clone(), other_vcs.clone(), other_formulas.clone());
        let actual = other_vcs.source_registration_canonical_fingerprints(&other_core, &other_formulas).unwrap();
        assert_eq!(actual.len(), fingerprints.len());
        for (id, owner) in &owners {
            let current = other_owners.iter().find(|(_, candidate)| *candidate == owner).unwrap().0;
            assert_eq!(actual[current], fingerprints[id], "neutral edit {index}: {owner:?}");
        }
        if index == 2 {
            let current = other_owners.iter().find(|(_, owner)| *owner == "OABExists:1").unwrap().0;
            let old = owners.iter().find(|(_, owner)| *owner == "OABExists:1").unwrap().0;
            assert_ne!(current, old);
            assert_ne!(other_vcs.vc(*current).unwrap().goal, vcs.vc(*old).unwrap().goal);
            assert_ne!(other_vcs.generated_formulas(), vcs.generated_formulas());
            assert_ne!(other_formulas.keys().collect::<Vec<_>>(), formulas.keys().collect::<Vec<_>>());
        }
        if skip != 0 { assert_ne!(other_core.source_id(), core.source_id()); }
        assert_eq!((other_core, other_vcs, other_formulas), other_before);
    }
    // Genuine A/B definition references keep that order; resolved owner bytes sort B/A.
    let (changed_core, changed_vcs, changed_formulas, changed_owners) = source_registration_vc_fingerprint_inputs(&text.replace("OAADef", "ZZZDef"), "vc_fingerprints", 0);
    let current = *changed_owners.iter().find(|(_, owner)| *owner == "OABExists:1").unwrap().0;
    let changed_before = (changed_core.clone(), changed_vcs.clone(), changed_formulas.clone());
    let vc = changed_vcs.vc(current).unwrap();
    let seed = changed_core.obligation_seeds().iter().find(|(id, _)| id.index() == vc.seed.handoff.index()).unwrap().1;
    let definitions = seed.core_refs.iter().filter_map(|reference| match reference { CoreNodeRef::Definition(id) => Some(*id), _ => None }).collect::<Vec<_>>();
    assert_eq!(definitions.len(), 2);
    assert!(definitions[0] < definitions[1]);
    let a_symbol = &changed_core.definitions().get(definitions[0]).unwrap().symbol;
    let b_symbol = &changed_core.definitions().get(definitions[1]).unwrap().symbol;
    assert_ne!(a_symbol, b_symbol);
    assert!(vc.premises[1] < vc.premises[2]);
    let mut resolved_attributes = Vec::new();
    for (premise, symbol) in vc.premises[1..].iter().zip([a_symbol, b_symbol]) {
        use mizar_vc::vc_ir::{PremiseRef, VcFormulaRef, VcGeneratedFormulaShape as S};
        let PremiseRef::GeneratedFact { formula: VcFormulaRef::Generated(root) } = premise else { unreachable!() };
        let S::Or(guarded) = &changed_vcs.generated_formula(*root).unwrap().shape else { unreachable!() };
        let VcFormulaRef::Generated(equivalent) = guarded[1] else { unreachable!() };
        let S::And(directions) = &changed_vcs.generated_formula(equivalent).unwrap().shape else { unreachable!() };
        let VcFormulaRef::Generated(forward) = directions[0] else { unreachable!() };
        let S::Or(forward) = &changed_vcs.generated_formula(forward).unwrap().shape else { unreachable!() };
        let VcFormulaRef::Generated(negated) = forward[0] else { unreachable!() };
        let S::Not(VcFormulaRef::Core(attribute)) = changed_vcs.generated_formula(negated).unwrap().shape else { unreachable!() };
        let mizar_core::core_ir::CoreFormulaKind::Atom { predicate, .. } = &changed_core.formulas().get(attribute).unwrap().kind else { unreachable!() };
        assert_eq!(predicate, symbol);
        resolved_attributes.push(&changed_formulas[&attribute]);
    }
    assert!(resolved_attributes[0].windows(6).any(|part| part == b"ZZZDef"));
    assert!(resolved_attributes[1].windows(6).any(|part| part == b"OBBDef"));
    assert!(resolved_attributes[1] < resolved_attributes[0]);
    let changed = changed_vcs.source_registration_canonical_fingerprints(&changed_core, &changed_formulas).unwrap();
    let hex = changed[&current].hash().as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    assert_eq!(hex, "bd7f2c8c1380b2698de870772a109ee5384770663e238283b72c624f4e4974d5");
    assert_eq!((changed_core, changed_vcs, changed_formulas), changed_before);
    for (index, edited, module) in [
        (0, text.replacen("means X = X", "means not X = X", 1), "vc_fingerprints"),
        (1, text.replace("OAExists", "RenamedExists"), "vc_fingerprints"),
        (2, text.clone(), "other_module"),
    ] {
        let (other_core, other_vcs, other_formulas, other_owners) = source_registration_vc_fingerprint_inputs(&edited, module, 0);
        let other_before = (other_core.clone(), other_vcs.clone(), other_formulas.clone());
        let actual = other_vcs.source_registration_canonical_fingerprints(&other_core, &other_formulas).unwrap();
        for (id, owner) in &owners {
            let name = if index == 1 { owner.replace("OAExists:", "RenamedExists:") } else { owner.clone() };
            let current = other_owners.iter().find(|(_, candidate)| **candidate == name).unwrap().0;
            let changed = match index { 0 => owner.ends_with(":1") && owner != "OBExists:1", 1 => owner.starts_with("OAExists:"), _ => true };
            assert_eq!(actual[current] != fingerprints[id], changed, "control {index}: {owner:?}");
            if index == 0 && owner == "OABExists:1" {
                let hex = actual[current].hash().as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
                assert_eq!(hex, "85c5f68eaebb5e1e8ea009284c5c072d65e0266469fdf5fcd0553465115fa3f1");
            }
        }
        assert_eq!((other_core, other_vcs, other_formulas), other_before);
    }
    for (generation, schema) in [("other-generation", "source-vc-fingerprint-v1"), ("source-vc-fingerprint-test", "other-vc-schema")] {
        let other_vcs = generate(&core, vcs.snapshot(), &GenerationSchemaVersion::new(generation), &VcSchemaVersion::new(schema)).unwrap();
        let actual = other_vcs.source_registration_canonical_fingerprints(&core, &formulas).unwrap();
        for (id, fingerprint) in &fingerprints { assert_ne!(actual[id], *fingerprint); }
    }
    assert_eq!((core, vcs, formulas), before);
}

#[test]
fn source_registration_vc_fingerprint_refuses_foreign_incomplete_and_altered_sets_atomically() {
    use mizar_vc::vc_ir::{LocalContext, PolicyKey, PolicyValue, VcSet, VcSetParts,
        VcStatus, VcText, VerifierPolicyInput, CanonicalSortKey};
    let text = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let (core, vcs, formulas, _) = source_registration_vc_fingerprint_inputs(&text, "vc_fingerprints", 0);
    let before = (core.clone(), vcs.clone(), formulas.clone());
    let unsupported = "unsupported source registration VC fingerprint";
    for (edited, skip) in [(text.replace("wmarked", "foreignmarked"), 0),
        (text.replacen("means X = X", "means not X = X", 1), 0), (text.clone(), 7)] {
        let (foreign_core, foreign_vcs, foreign_formulas, _) = source_registration_vc_fingerprint_inputs(&edited, "vc_fingerprints", skip);
        let foreign_before = (foreign_core.clone(), foreign_vcs.clone(), foreign_formulas.clone());
        assert_ne!(foreign_core, core);
        assert_eq!(vcs.source_registration_canonical_fingerprints(&foreign_core, &formulas).unwrap_err(), unsupported);
        assert_eq!(foreign_vcs.source_registration_canonical_fingerprints(&core, &foreign_formulas).unwrap_err(), unsupported);
        assert_eq!((foreign_core, foreign_vcs, foreign_formulas), foreign_before);
    }
    // Genuine Core55 supports this order; the source VC generator rejects its OAB refs.
    {
        let aggregate = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
        let a = aggregate.find("  cluster OAExists:").unwrap();
        let b = aggregate.find("  cluster OBExists:").unwrap();
        let ab = aggregate.find("  cluster OABExists:").unwrap();
        let swapped = format!("{}{}{}{}", &aggregate[..a], &aggregate[b..ab], &aggregate[a..b], &aggregate[ab..]);
        assert_ne!(swapped, aggregate);
        let (source, nodes, collected, owners) = source_registration_formula_inputs(&swapped, "vc-fingerprint-test", "vc_fingerprints", 0);
        let check = mizar_checker::registration_resolution::check_source_existential_registration_proof(&source, &nodes, collected.env()).unwrap();
        let unsupported_core = mizar_core::elaborator::lower_source_existential_registration(&check).unwrap();
        let empty = mizar_core::elaborator::source_registration_formula_payloads(&check, &unsupported_core, &owners, &[]).unwrap();
        assert!(empty.is_empty());
        let upstream = mizar_vc::generator::generate_source_existential_registration(
            &unsupported_core, vcs.snapshot(), &vcs.vcs()[0].anchor.generation_schema_version, vcs.schema_version(),
        ).unwrap_err();
        assert_ne!(upstream, unsupported);
        let core_and_map_before = (unsupported_core.clone(), empty.clone());
        assert_eq!(vcs.source_registration_canonical_fingerprints(&unsupported_core, &empty).unwrap_err(), upstream);
        assert_eq!((unsupported_core, empty), core_and_map_before);
    }
    let root = *formulas.keys().next().unwrap();
    let extra = core.formulas().iter().find(|(id, _)| !formulas.contains_key(id)).unwrap().0;
    for index in 0..3 {
        let mut bad = formulas.clone();
        match index { 0 => { bad.remove(&root); }, 1 => { bad.insert(root, Vec::new()); },
            _ => { bad.insert(extra, formulas[&root].clone()); } }
        let before = bad.clone();
        assert_eq!(vcs.source_registration_canonical_fingerprints(&core, &bad).unwrap_err(), unsupported);
        assert_eq!(bad, before);
    }
    for index in 0..9 {
        let mut parts = VcSetParts { schema_version: vcs.schema_version().clone(), snapshot: vcs.snapshot(),
            source: vcs.source(), module: vcs.module().clone(), generated_formulas: vcs.generated_formulas().to_vec(),
            vcs: vcs.vcs().to_vec(), seed_accounting: vcs.seed_accounting().to_vec() };
        match index {
            0 => parts.vcs[0].goal = parts.vcs[1].goal,
            1 => parts.vcs[1].premises.pop().map(|_| ()).unwrap(),
            2 => parts.vcs[0].provenance[0].key = VcText::new("altered"),
            3 => parts.module = "foreign-module".into(),
            4 => parts.vcs[0].status = VcStatus::NeedsAtp,
            5 => parts.vcs[0].local_context = LocalContext::try_new(Vec::new(), vec![VerifierPolicyInput {
                sort_key: CanonicalSortKey::new("policy"), key: PolicyKey::new("policy"), value: PolicyValue::new("altered"),
            }]).unwrap(),
            6 => parts.generated_formulas[0].provenance[0].key = VcText::new("altered"),
            7 => parts.vcs[0].anchor.generation_schema_version = "other-generation".into(),
            _ => { parts.vcs.clear(); parts.generated_formulas.clear(); parts.seed_accounting.clear(); }
        }
        let bad = VcSet::try_new(parts).unwrap();
        let before = bad.clone();
        assert_eq!(bad.source_registration_canonical_fingerprints(&core, &formulas).unwrap_err(), unsupported, "corruption {index}");
        assert_eq!(bad, before);
    }
    assert_eq!((core, vcs, formulas), before);
}

#[test]
fn source_registration_vc_fingerprint_goal_context_exact_and_default_discharge() {
    use mizar_vc::discharge::{try_discharge, DischargeInput, DischargePolicy};
    // Spec 6.2/17.2-17.3; independent semantic-literal component expectations.
    for (multiple, case) in [(false, step5c3_registration_case()), (true, step5c13_registration_case())] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (core, vcs, formulas, owners) = source_registration_vc_fingerprint_inputs(&text, "vc_fingerprints", 0);
        let before = (core.clone(), vcs.clone(), formulas.clone());
        let expected = if multiple { vec![
            ("OAExists:0", "4b4a60f718622f1c4634fb5bd1c387511846dba0c963586349b8a2403b95474e", "47bcff63208e4f2bb751aa7f0e7de82e2e937c366e03d8e20d1a51bc0ccf404a"),
            ("OAExists:1", "84d94214fdef120907bc1f27bb53c0f243dabf8b169a57ef064ed257e42188e5", "0ce52dd269455ebfa02edfca43f8e9fbdb8a8c5eda2dcbf89b91fff1aa66893d"),
            ("OBExists:0", "4b4a60f718622f1c4634fb5bd1c38751231ead4bebb4003949b8a2403b95474e", "47bcff63208e4f2bb751aa7f0e7de82ec4cd8d3b34755d0f0d1a51bc0ccf404a"),
            ("OBExists:1", "33c378c34ceba23e07bc1f27bb53c0f26352c612321e14de4e307a1c7cac4644", "9ecce3fea430af414f3523d20bc9f3394f1f6a3103e5fa6ff3db1dc458fac0fa"),
            ("OABExists:0", "e380b37c2b98e63cfe64c047e89d1394b370fbd5d1153c57d96e4b8476c3fb61", "0f160ddcfcc2f9106d7ffc5ce2d257a15c5b8bafac06294bdc8ccce6152d3200"),
            ("OABExists:1", "d7d777d0d2fe0151e97721647d9190ad3c8580767fe76d7d98f90153d81264a5", "1bf3aefadacb21dda37ccec89075065345dfaec53d77eeaa0d6e35699144a0b7"),
        ] } else { vec![
            ("WMarkedExists:0", "71433ad25a6e8569d236f5ab777d8a50873e4d313d68cc861a8be02fd85bb00a", "f443b2554f5faa7beb6010089818bc5c54aa46205a82deba74ec749506bdffb7"),
            ("WMarkedExists:1", "17b53b73858f307f1649fb6bb0d909889bee8acf6c4f5e63376856076054b7e2", "e53a60bb620c4c579fd3d1693a5882cbacd4a2d9acb894097b1f9e8783d0ebf7"),
        ] };
        assert_eq!(owners.len(), expected.len());
        let discharged = try_discharge(DischargeInput { vc_set: &vcs, policy: &DischargePolicy::default() }).unwrap();
        assert_ne!(discharged.vc_set(), &vcs);
        let discharged_before = discharged.vc_set().clone();
        for (id, owner) in &owners {
            let pair = vcs.source_registration_canonical_goal_context(*id, &core, &formulas).unwrap();
            let actual = [pair.0, pair.1].map(|hash| hash.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>());
            let (_, goal, context) = expected.iter().find(|(name, _, _)| *name == owner).unwrap();
            assert_eq!(actual, [*goal, *context], "{owner}");
            assert_eq!(discharged.vc_set().source_registration_canonical_goal_context(*id, &core, &formulas).unwrap(), pair);
        }
        assert_eq!(discharged.vc_set(), &discharged_before);
        assert_eq!((core, vcs, formulas), before);
    }
}

#[test]
fn source_registration_vc_fingerprint_goal_context_edits_order_and_separation() {
    use mizar_vc::{generator::generate_source_existential_registration as generate,
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion}};
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let (core, vcs, formulas, owners) = source_registration_vc_fingerprint_inputs(&text, "vc_fingerprints", 0);
    let before = (core.clone(), vcs.clone(), formulas.clone());
    let baseline = owners.iter().map(|(id, owner)| (owner.clone(), vcs.source_registration_canonical_goal_context(*id, &core, &formulas).unwrap())).collect::<std::collections::BTreeMap<_, _>>();
    let a = "  attr OAADef: X is oamarked means X = X;\n";
    let b = "  attr OBBDef: X is obmarked means X = X;\n";
    let registration_a = "  cluster OAExists: oamarked set;\n  existence proof\n    take the set;\n    thus the set is oamarked by OAADef;\n  end;\n";
    let registration_b = "  cluster OBExists: obmarked set;\n  existence proof\n    take the set;\n    thus the set is obmarked by OBBDef;\n  end;\n";
    let registration_ab = "  cluster OABExists: oamarked obmarked set;\n  existence proof\n    take the set;\n    thus the set is oamarked obmarked by OAADef, OBBDef;\n  end;\n";
    let reordered = text.replace(&format!("{a}{b}"), &format!("{b}{a}"))
        .replace(&format!("{registration_a}{registration_b}{registration_ab}"), &format!("{registration_ab}{registration_a}{registration_b}"));
    assert_ne!(reordered, text);
    for (index, edited, skip) in [
        (0, format!("\n:: shifted ranges\n{}", text.replace("  ", "    ")), 0),
        (1, text.replace('X', "Subject"), 0),
        (2, reordered, 0),
        (3, text.clone(), 7),
    ] {
        let (other_core, other_vcs, other_formulas, other_owners) = source_registration_vc_fingerprint_inputs(&edited, "vc_fingerprints", skip);
        let other_before = (other_core.clone(), other_vcs.clone(), other_formulas.clone());
        for (id, owner) in &other_owners {
            assert_eq!(other_vcs.source_registration_canonical_goal_context(*id, &other_core, &other_formulas).unwrap(), baseline[owner], "neutral edit {index}: {owner}");
        }
        if index == 2 {
            let current = *other_owners.iter().find(|(_, owner)| *owner == "OABExists:1").unwrap().0;
            let old = *owners.iter().find(|(_, owner)| *owner == "OABExists:1").unwrap().0;
            assert_ne!(current, old);
            assert_ne!(other_vcs.vc(current).unwrap().goal, vcs.vc(old).unwrap().goal);
            assert_ne!(other_formulas.keys().collect::<Vec<_>>(), formulas.keys().collect::<Vec<_>>());
        }
        if skip != 0 { assert_ne!(other_core.source_id(), core.source_id()); }
        assert_eq!((other_core, other_vcs, other_formulas), other_before);
    }
    // Existing VC65 coverage traces this genuine A/B-reference control to B/A content order.
    let (changed_core, changed_vcs, changed_formulas, changed_owners) = source_registration_vc_fingerprint_inputs(&text.replace("OAADef", "ZZZDef"), "vc_fingerprints", 0);
    let current = *changed_owners.iter().find(|(_, owner)| *owner == "OABExists:1").unwrap().0;
    let changed_before = (changed_core.clone(), changed_vcs.clone(), changed_formulas.clone());
    let pair = changed_vcs.source_registration_canonical_goal_context(current, &changed_core, &changed_formulas).unwrap();
    let actual = [pair.0, pair.1].map(|hash| hash.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>());
    assert_eq!(actual, ["ab613a0887575f91e97721647d9190ad09e7790e6143deb841a193015c41303b", "ea87af78ac61e0060bc626c2207e6bcf252c4e198d07d52b033789e02936206e"]);
    assert_eq!((changed_core, changed_vcs, changed_formulas), changed_before);
    for (index, edited, module) in [
        (0, text.replacen("means X = X", "means not X = X", 1), "vc_fingerprints"),
        (1, text.replace("OAExists", "RenamedExists"), "vc_fingerprints"),
        (2, text.clone(), "other_module"),
    ] {
        let (other_core, other_vcs, other_formulas, other_owners) = source_registration_vc_fingerprint_inputs(&edited, module, 0);
        let other_before = (other_core.clone(), other_vcs.clone(), other_formulas.clone());
        for (id, owner) in &other_owners {
            let name = if index == 1 { owner.replace("RenamedExists:", "OAExists:") } else { owner.clone() };
            let pair = other_vcs.source_registration_canonical_goal_context(*id, &other_core, &other_formulas).unwrap();
            let expected = baseline[&name];
            let context_changed = match index { 0 => owner == "OAExists:1" || owner == "OABExists:1", 1 => name.starts_with("OAExists:"), _ => true };
            let goal_changed = match index { 0 => false, 1 => name.starts_with("OAExists:"), _ => true };
            assert_eq!(pair.0 != expected.0, goal_changed, "goal control {index}: {owner}");
            assert_eq!(pair.1 != expected.1, context_changed, "context control {index}: {owner}");
            if index == 0 && owner == "OABExists:1" {
                let hex = pair.1.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
                assert_eq!(hex, "4505d4e4b00dda13a34b976f8a27518052ee5cdf74095f8c7a09949fe5534184");
            }
        }
        assert_eq!((other_core, other_vcs, other_formulas), other_before);
    }
    for (generation, schema) in [("other-generation", "source-vc-fingerprint-v1"), ("source-vc-fingerprint-test", "other-vc-schema")] {
        let other_vcs = generate(&core, vcs.snapshot(), &GenerationSchemaVersion::new(generation), &VcSchemaVersion::new(schema)).unwrap();
        let other_before = other_vcs.clone();
        for (id, owner) in &owners {
            let pair = other_vcs.source_registration_canonical_goal_context(*id, &core, &formulas).unwrap();
            assert_ne!(pair.0, baseline[owner].0);
            assert_ne!(pair.1, baseline[owner].1);
        }
        assert_eq!(other_vcs, other_before);
    }
    assert_eq!((core, vcs, formulas), before);
}

#[test]
fn source_registration_vc_fingerprint_goal_context_refuses_whole_inputs_before_selection() {
    use mizar_vc::vc_ir::{VcFormulaRef, VcId, VcSet, VcSetParts, VcText};
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let (core, vcs, formulas, owners) = source_registration_vc_fingerprint_inputs(&text, "vc_fingerprints", 0);
    let before = (core.clone(), vcs.clone(), formulas.clone());
    let selected = *owners.iter().find(|(_, owner)| *owner == "OAExists:0").unwrap().0;
    let absent = VcId::new(vcs.vcs().len());
    let unsupported = "unsupported source registration VC fingerprint";
    assert_eq!(vcs.source_registration_canonical_goal_context(absent, &core, &formulas).unwrap_err(), unsupported);
    // The selected type leaf reaches just its one actual Core root.
    let VcFormulaRef::Core(selected_root) = vcs.vc(selected).unwrap().goal else { unreachable!() };
    let outside = *formulas.keys().find(|id| **id != selected_root).unwrap();
    let extra = core.formulas().iter().find(|(id, _)| !formulas.contains_key(id)).unwrap().0;
    for index in 0..3 {
        let mut bad = formulas.clone();
        match index { 0 => { bad.remove(&outside); }, 1 => { bad.insert(outside, Vec::new()); },
            _ => { bad.insert(extra, formulas[&outside].clone()); } }
        let map_before = bad.clone();
        assert_eq!(vcs.source_registration_canonical_goal_context(selected, &core, &bad).unwrap_err(), unsupported);
        assert_eq!(bad, map_before);
    }
    let mut parts = VcSetParts { schema_version: vcs.schema_version().clone(), snapshot: vcs.snapshot(),
        source: vcs.source(), module: vcs.module().clone(), generated_formulas: vcs.generated_formulas().to_vec(),
        vcs: vcs.vcs().to_vec(), seed_accounting: vcs.seed_accounting().to_vec() };
    let unselected = parts.vcs.iter_mut().find(|vc| vc.id != selected).unwrap();
    unselected.provenance[0].key = VcText::new("altered unselected leaf");
    let bad = VcSet::try_new(parts).unwrap();
    let bad_before = bad.clone();
    assert_eq!(bad.source_registration_canonical_goal_context(selected, &core, &formulas).unwrap_err(), unsupported);
    assert_eq!(bad, bad_before);
    let (foreign_core, foreign_vcs, foreign_formulas, _) = source_registration_vc_fingerprint_inputs(&text.replace("oamarked", "foreignmarked"), "vc_fingerprints", 0);
    let foreign_before = (foreign_core.clone(), foreign_vcs.clone(), foreign_formulas.clone());
    assert_ne!(foreign_core, core);
    assert_eq!(vcs.source_registration_canonical_goal_context(selected, &foreign_core, &formulas).unwrap_err(), unsupported);
    assert_eq!(foreign_vcs.source_registration_canonical_goal_context(selected, &core, &foreign_formulas).unwrap_err(), unsupported);
    assert_eq!((foreign_core, foreign_vcs, foreign_formulas), foreign_before);
    // Core55 supports OA/OB swapping, while genuine source VC regeneration rejects it.
    let a = text.find("  cluster OAExists:").unwrap();
    let b = text.find("  cluster OBExists:").unwrap();
    let ab = text.find("  cluster OABExists:").unwrap();
    let swapped = format!("{}{}{}{}", &text[..a], &text[b..ab], &text[a..b], &text[ab..]);
    assert_ne!(swapped, text);
    let (source, nodes, collected, owner_payloads) = source_registration_formula_inputs(&swapped, "vc-fingerprint-test", "vc_fingerprints", 0);
    let check = mizar_checker::registration_resolution::check_source_existential_registration_proof(&source, &nodes, collected.env()).unwrap();
    let unsupported_core = mizar_core::elaborator::lower_source_existential_registration(&check).unwrap();
    let empty = mizar_core::elaborator::source_registration_formula_payloads(&check, &unsupported_core, &owner_payloads, &[]).unwrap();
    assert!(empty.is_empty());
    let upstream = mizar_vc::generator::generate_source_existential_registration(
        &unsupported_core, vcs.snapshot(), &vcs.vcs()[0].anchor.generation_schema_version, vcs.schema_version(),
    ).unwrap_err();
    assert_ne!(upstream, unsupported);
    let upstream_before = (unsupported_core.clone(), empty.clone());
    assert_eq!(vcs.source_registration_canonical_goal_context(absent, &unsupported_core, &empty).unwrap_err(), upstream);
    assert_eq!((unsupported_core, empty), upstream_before);
    assert_eq!((core, vcs, formulas), before);
}

#[test]
fn source_registration_canonical_components_exact_current_leaves_and_producer_equality() {
    use mizar_proof::status::source_registration_canonical_components as components;
    use mizar_vc::{generator::generate_source_existential_registration as generate,
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion}};
    // Spec 6.2/17.2-17.3; independent semantic literals with existing Proof schemas.
    let expected = [
            ("WMarkedExists:0", ["41511505b528f198d3eaa04b46cf8db24c65dae53fdb2ab19dd6d4da683dc983", "c7cb7dea9c41ea1ceabf196a535723219b3df51f307942f6256265f39873498d", "0ab48d5682ae1b48fff910f004655a485e8dad151d7e573040e33a6b344a2b9b"]),
            ("WMarkedExists:1", ["af00f4f76a9b95ee07503426b32b958d52b6badb091a8f01a2ee8dbb92a8d47b", "da98780a50d5c8ece525ebd1155dbe3855450f76b18ce0d80626c0cb378254ec", "48bc5627acf00e1c9c73e9c5193de72c98df05fe16db7c923544cabbf0352283"]),
            ("OAExists:0", ["90b4afcfba9e663b7dfb02b029a64787e7290811e8e6d4ccf35d5cd0aad43604", "2e217ac8e5780d68d8d90cb1684ffab8b8b8b167e2d30e4dd09c531c7323d46e", "ef79f6d1ae040194674547055df012d76375069d97ec8cfecc91f512dfe67edb"]),
            ("OAExists:1", ["7a0ba9e438f251286b86f5e947f18abd7baa1a092ac1a9e8556aa0eac1ab471a", "a9560e0e4eb5b1673bccef0f3d503fccbae72b1255df67c7bbd9bed99316b10e", "19c7ca5c8b33f06205363a444fbd61ef0b82c3f6e543e8eb3afdc70ac7ebd7f5"]),
            ("OBExists:0", ["fa6ed869a5372bf37dfb02b029a647878a12db1e3c4030f9f35d5cd0aad43604", "2e217ac8e5780d68d8d90cb1684ffab8b8b8b167e2d30e4dd1e263caa9600da5", "ef79f6d1ae040194674547055df012d76375069d97ec8cfefbe1f1878eecb493"]),
            ("OBExists:1", ["823953e2287436c2cb922958af274eb5dcb854a516ccfaebd7ff26330b5947bc", "90b3eb40750e7a8f43ecc0d3bd872f7048158c0f883a93dbe7a0de2e9e249f5c", "efcc953a42287158e83a16ade07057e9a68c9588e0b6e7deed45089e58fffd2a"]),
            ("OABExists:0", ["86ebaecbde4404129e1b76a02d80436d94f5614528616d2406727c218d4130ee", "1858c583981b9e80d8b06ab5adf11b0cc4196cec1c94154f98cf38eb26903e91", "119e79588d41d5c07508fa1d46879b36c4a8248bdf60f16aafd93a676df8311f"]),
            ("OABExists:1", ["ab7f0c1db0ed819a1fb434e5c4b7c44347a3de5a1a6cd45fae853d523cc937fc", "93fb0ea3ef9be446687613ae6b349994d6e160e7f03310cddc051bc498602255", "0aca806622a46550fa0ed46926004662fc55d52d8999b0afb4c36201b66acb52"]),
    ];
    let mut checked = 0;
    for case in [step5c3_registration_case(), step5c13_registration_case()] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (source, nodes, collected, _) = source_registration_formula_inputs(&text, "vc-fingerprint-test", "vc_fingerprints", 0);
        let before = (source.clone(), nodes.clone(), collected.env().clone());
        let (core, _, formulas, owners) = source_registration_vc_fingerprint_inputs(&text, "vc_fingerprints", 0);
        for snapshot in [super::super::shared::snapshot_id(0), super::super::shared::snapshot_id(41)] {
            let vcs = generate(&core, snapshot, &GenerationSchemaVersion::new("source-existential-registration-v1"), &VcSchemaVersion::new("vc-v1")).unwrap();
            let full = vcs.source_registration_canonical_fingerprints(&core, &formulas).unwrap();
            assert_eq!(vcs.vcs().len(), owners.len());
            for (id, owner) in &owners {
                let triple = components(&source, &nodes, collected.env(), snapshot, *id).unwrap();
                let pair = vcs.source_registration_canonical_goal_context(*id, &core, &formulas).unwrap();
                assert_eq!(triple, (full[id], pair.0, pair.1), "{owner}");
                let hex = [triple.0.hash(), triple.1, triple.2].map(|hash| hash.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>());
                assert_eq!(hex, expected.iter().find(|(name, _)| name == owner).unwrap().1, "{owner}");
                if snapshot == super::super::shared::snapshot_id(0) { checked += 1; }
            }
        }
        assert_eq!((source, nodes, collected.env().clone()), before);
    }
    assert_eq!(checked, 8);
}

#[test]
fn source_registration_canonical_components_neutral_relocation_and_participating_edits() {
    use mizar_proof::status::source_registration_canonical_components as components;
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let snapshot = super::super::shared::snapshot_id(0);
    let (source, nodes, collected, _) = source_registration_formula_inputs(&text, "vc-fingerprint-test", "vc_fingerprints", 0);
    let before = (source.clone(), nodes.clone(), collected.env().clone());
    let (core, vcs, formulas, owners) = source_registration_vc_fingerprint_inputs(&text, "vc_fingerprints", 0);
    let mut baseline = std::collections::BTreeMap::new();
    for (id, owner) in &owners {
        baseline.insert(owner.clone(), components(&source, &nodes, collected.env(), snapshot, *id).unwrap());
    }
    let a = "  attr OAADef: X is oamarked means X = X;\n";
    let b = "  attr OBBDef: X is obmarked means X = X;\n";
    let registration_a = "  cluster OAExists: oamarked set;\n  existence proof\n    take the set;\n    thus the set is oamarked by OAADef;\n  end;\n";
    let registration_b = "  cluster OBExists: obmarked set;\n  existence proof\n    take the set;\n    thus the set is obmarked by OBBDef;\n  end;\n";
    let registration_ab = "  cluster OABExists: oamarked obmarked set;\n  existence proof\n    take the set;\n    thus the set is oamarked obmarked by OAADef, OBBDef;\n  end;\n";
    let reordered = text.replace(&format!("{a}{b}"), &format!("{b}{a}"))
        .replace(&format!("{registration_a}{registration_b}{registration_ab}"), &format!("{registration_ab}{registration_a}{registration_b}"));
    assert_ne!(reordered, text);
    for (index, edited, skip) in [
        (0, format!("\n:: shifted ranges\n{}", text.replace("  ", "    ")), 0),
        (1, text.replace('X', "Subject"), 0), (2, reordered, 0), (3, text.clone(), 7),
    ] {
        let (s, n, e, _) = source_registration_formula_inputs(&edited, "vc-fingerprint-test", "vc_fingerprints", skip);
        let inputs_before = (s.clone(), n.clone(), e.env().clone());
        let (other_core, other_vcs, other_formulas, other_owners) = source_registration_vc_fingerprint_inputs(&edited, "vc_fingerprints", skip);
        assert_eq!(other_owners.len(), baseline.len());
        for (id, owner) in &other_owners {
            assert_eq!(components(&s, &n, e.env(), snapshot, *id).unwrap(), baseline[owner], "neutral edit {index}: {owner}");
        }
        if index == 2 {
            let current = *other_owners.iter().find(|(_, owner)| *owner == "OABExists:1").unwrap().0;
            let old = *owners.iter().find(|(_, owner)| *owner == "OABExists:1").unwrap().0;
            assert_ne!(current, old);
            assert_ne!(other_vcs.vc(current).unwrap().goal, vcs.vc(old).unwrap().goal);
            assert_ne!(other_formulas.keys().collect::<Vec<_>>(), formulas.keys().collect::<Vec<_>>());
        }
        if skip != 0 { assert_ne!(other_core.source_id(), core.source_id()); }
        assert_eq!((s, n, e.env().clone()), inputs_before);
    }
    for (index, edited, module) in [
        (0, text.replacen("means X = X", "means not X = X", 1), "vc_fingerprints"),
        (1, text.replace("OAExists", "RenamedExists"), "vc_fingerprints"),
        (2, text.clone(), "other_module"),
    ] {
        let (s, n, e, _) = source_registration_formula_inputs(&edited, "vc-fingerprint-test", module, 0);
        let inputs_before = (s.clone(), n.clone(), e.env().clone());
        let (_, _, _, current_owners) = source_registration_vc_fingerprint_inputs(&edited, module, 0);
        assert_eq!(current_owners.len(), baseline.len());
        for (id, owner) in &current_owners {
            let name = if index == 1 { owner.replace("RenamedExists:", "OAExists:") } else { owner.clone() };
            let triple = components(&s, &n, e.env(), snapshot, *id).unwrap();
            let original = &baseline[&name];
            let changed = match index { 0 => owner == "OAExists:1" || owner == "OABExists:1", 1 => name.starts_with("OAExists:"), _ => true };
            assert_eq!(triple.0 != original.0, changed, "VC control {index}: {owner}");
            assert_eq!(triple.1 != original.1, index != 0 && changed, "goal control {index}: {owner}");
            assert_eq!(triple.2 != original.2, changed, "context control {index}: {owner}");
            if index == 0 && owner == "OABExists:1" {
                let hex = [triple.0.hash(), triple.1, triple.2].map(|hash| hash.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>());
                assert_eq!(hex, ["095bdfff03efe1ddc81f05f7af856040afec2930beeb84da0f95b16d9e0a4bcc", "93fb0ea3ef9be446687613ae6b349994d6e160e7f03310cddc051bc498602255", "042d35a9d0248233bada41d2e6413c4c8d23722fac79e8eeee6a16be4ef8f4b3"]);
            }
        }
        assert_eq!((s, n, e.env().clone()), inputs_before);
    }
    assert_eq!((source, nodes, collected.env().clone()), before);
}

#[test]
fn source_registration_canonical_components_rejects_absent_foreign_and_upstream_errors() {
    use mizar_checker::registration_resolution::check_source_existential_registration_proof as check;
    use mizar_proof::status::source_registration_canonical_components as components;
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion};
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let snapshot = super::super::shared::snapshot_id(0);
    let (source, nodes, collected, _) = source_registration_formula_inputs(&text, "vc-fingerprint-test", "vc_fingerprints", 0);
    let before = (source.clone(), nodes.clone(), collected.env().clone());
    let (_, vcs, _, owners) = source_registration_vc_fingerprint_inputs(&text, "vc_fingerprints", 0);
    let selected = *owners.keys().next().unwrap();
    let absent = VcId::new(vcs.vcs().len());
    let invalid = "registration.proof.invalid_source_components";
    assert_eq!(components(&source, &nodes, collected.env(), snapshot, absent).unwrap_err(), invalid);
    let (foreign_source, foreign_nodes, foreign_env, _) = source_registration_formula_inputs(&text.replace("oamarked", "foreignmarked"), "vc-fingerprint-test", "vc_fingerprints", 0);
    let foreign_before = (foreign_source.clone(), foreign_nodes.clone(), foreign_env.env().clone());
    for (s, n, e) in [(&foreign_source, &nodes, collected.env()), (&source, &foreign_nodes, collected.env()), (&source, &nodes, foreign_env.env())] {
        let upstream = check(s, n, e).unwrap_err();
        assert_ne!(upstream, invalid);
        for id in [selected, absent] {
            assert_eq!(components(s, n, e, snapshot, id).unwrap_err(), upstream);
        }
    }
    assert_eq!((foreign_source, foreign_nodes, foreign_env.env().clone()), foreign_before);
    // A syntactically valid proof with its required witness step missing is refused upstream.
    let malformed = text.replacen("    take the set;\n", "", 1);
    assert_ne!(malformed, text);
    let (s, n, e, _) = source_registration_formula_inputs(&malformed, "vc-fingerprint-test", "vc_fingerprints", 0);
    let inputs_before = (s.clone(), n.clone(), e.env().clone());
    let upstream = check(&s, &n, e.env()).unwrap_err();
    assert_ne!(upstream, invalid);
    for id in [selected, absent] {
        assert_eq!(components(&s, &n, e.env(), snapshot, id).unwrap_err(), upstream);
    }
    assert_eq!((s, n, e.env().clone()), inputs_before);
    // Genuine Core supports OA/OB swapping, while source VC generation refuses it.
    let a = text.find("  cluster OAExists:").unwrap();
    let b = text.find("  cluster OBExists:").unwrap();
    let ab = text.find("  cluster OABExists:").unwrap();
    let swapped = format!("{}{}{}{}", &text[..a], &text[b..ab], &text[a..b], &text[ab..]);
    assert_ne!(swapped, text);
    let (s, n, e, _) = source_registration_formula_inputs(&swapped, "vc-fingerprint-test", "vc_fingerprints", 0);
    let inputs_before = (s.clone(), n.clone(), e.env().clone());
    let checked = check(&s, &n, e.env()).unwrap();
    let core = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
    let upstream = mizar_vc::generator::generate_source_existential_registration(&core, snapshot,
        &GenerationSchemaVersion::new("source-existential-registration-v1"), &VcSchemaVersion::new("vc-v1")).unwrap_err();
    assert_ne!(upstream, invalid);
    for id in [selected, absent] {
        assert_eq!(components(&s, &n, e.env(), snapshot, id).unwrap_err(), upstream);
    }
    assert_eq!((s, n, e.env().clone()), inputs_before);
    assert_eq!((source, nodes, collected.env().clone()), before);
}
#[test]
fn source_registration_syntax_exact_single_aggregate_current_keys_and_empty() {
    use mizar_resolve::{
        env::SymbolKind, symbols::source_registration_syntax_payloads_from_arena as syntax,
    };
    let expected = [
        (
            "WMarkedExists",
            source_registration_syntax_expected("WMarkedExists", &["wmarked"], &["WMDef"]),
        ),
        (
            "OAExists",
            source_registration_syntax_expected("OAExists", &["oamarked"], &["OAADef"]),
        ),
        (
            "OBExists",
            source_registration_syntax_expected("OBExists", &["obmarked"], &["OBBDef"]),
        ),
        (
            "OABExists",
            source_registration_syntax_expected(
                "OABExists",
                &["oamarked", "obmarked"],
                &["OAADef", "OBBDef"],
            ),
        ),
    ];
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (frontend, source, collected) =
            source_registration_owner_inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let before = (frontend.ast.clone(), source.clone(), collected.clone());
        let payloads = syntax(&source, collected.env()).unwrap();
        let entries = collected
            .env()
            .symbols()
            .iter()
            .filter(|entry| entry.kind() == SymbolKind::Registration)
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), if multiple { 3 } else { 1 });
        assert_eq!(payloads.len(), entries.len());
        for entry in entries {
            let bytes = &expected
                .iter()
                .find(|(label, _)| *label == entry.primary_spelling())
                .unwrap()
                .1;
            assert_eq!(payloads.get(entry.symbol()), Some(bytes));
        }
        assert_eq!(syntax(&source, collected.env()).unwrap(), payloads);
        assert_eq!((frontend.ast, source, collected), before);
    }
    for text in [
        "",
        "definition\n let X be set;\n attr OnlyDef: X is onlymarked means X = X;\nend;\n",
    ] {
        let (frontend, source, collected) =
            source_registration_owner_inputs(text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let before = (frontend.ast.clone(), source.clone(), collected.clone());
        assert!(syntax(&source, collected.env()).unwrap().is_empty());
        assert_eq!((frontend.ast, source, collected), before);
    }
}

#[test]
fn source_registration_syntax_neutral_relocation_order_and_participating_edits() {
    use mizar_resolve::{
        env::SymbolKind, symbols::source_registration_syntax_payloads_from_arena as syntax,
    };
    use mizar_syntax::SurfaceNodeKind;
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let (_, source, collected) = source_registration_owner_inputs(&text, "owner-test", "owners", 0);
    let baseline = syntax(&source, collected.env()).unwrap();
    let baseline_ids = source
        .arena()
        .iter()
        .filter_map(|(id, node)| {
            matches!(node.kind(), SurfaceNodeKind::ExistentialRegistration).then_some(id)
        })
        .collect::<Vec<_>>();
    let attribute_a = "  attr OAADef: X is oamarked means X = X;\n";
    let attribute_b = "  attr OBBDef: X is obmarked means X = X;\n";
    let registration_a = "  cluster OAExists: oamarked set;\n  existence proof\n    take the set;\n    thus the set is oamarked by OAADef;\n  end;\n";
    let registration_b = "  cluster OBExists: obmarked set;\n  existence proof\n    take the set;\n    thus the set is obmarked by OBBDef;\n  end;\n";
    let registration_ab = "  cluster OABExists: oamarked obmarked set;\n  existence proof\n    take the set;\n    thus the set is oamarked obmarked by OAADef, OBBDef;\n  end;\n";
    let reordered = text
        .replace(
            &format!("{attribute_a}{attribute_b}"),
            &format!("{attribute_b}{attribute_a}"),
        )
        .replace(
            &format!("{registration_a}{registration_b}{registration_ab}"),
            &format!("{registration_ab}{registration_b}{registration_a}"),
        );
    assert_ne!(reordered, text);
    let split = format!("definition\n let X be set;\n{attribute_b}end;\ndefinition\n let X be set;\n{attribute_a}end;\nregistration\n{registration_ab}end;\nregistration\n{registration_b}end;\nregistration\n{registration_a}end;\n");
    let leading =
        "definition\n let X be set;\n func ExtraDef: extra X -> set equals X;\n coherence;\nend;\n";
    let before = (source.clone(), collected.clone(), baseline.clone());
    for (index, edited, skip) in [
        (
            0,
            format!("\n:: shifted trivia\n{}", text.replace("  ", "    ")),
            0,
        ),
        (1, text.replace('X', "OuterBinder"), 0),
        (2, text.replacen("means X = X", "means not X = X", 1), 0),
        (3, text.clone(), 7),
        (4, format!("{leading}{text}"), 0),
        (5, reordered, 0),
        (6, split, 0),
    ] {
        let (frontend, current, entries) =
            source_registration_owner_inputs(&edited, "owner-test", "owners", skip);
        assert!(frontend.diagnostics.is_empty(), "edit {index}");
        assert!(entries.diagnostics().is_empty(), "edit {index}");
        let current_before = (frontend.ast.clone(), current.clone(), entries.clone());
        let actual = syntax(&current, entries.env()).unwrap();
        assert_eq!(actual.len(), 3, "edit {index}");
        for entry in entries
            .env()
            .symbols()
            .iter()
            .filter(|entry| entry.kind() == SymbolKind::Registration)
        {
            let old = collected
                .env()
                .symbols()
                .iter()
                .find(|old| {
                    old.kind() == SymbolKind::Registration
                        && old.primary_spelling() == entry.primary_spelling()
                })
                .unwrap();
            assert_eq!(
                actual.get(entry.symbol()),
                baseline.get(old.symbol()),
                "edit {index}: {}",
                entry.primary_spelling()
            );
        }
        if index == 3 {
            assert_ne!(current.source_id(), source.source_id());
        }
        if index == 4 {
            let ids = current
                .arena()
                .iter()
                .filter_map(|(id, node)| {
                    matches!(node.kind(), SurfaceNodeKind::ExistentialRegistration).then_some(id)
                })
                .collect::<Vec<_>>();
            assert_ne!(ids, baseline_ids);
        }
        if index >= 5 {
            assert_ne!(
                actual.keys().collect::<Vec<_>>(),
                baseline.keys().collect::<Vec<_>>()
            );
        }
        assert_eq!((frontend.ast, current, entries), current_before);
    }
    for (index, edited, package, module) in [
        (
            0,
            text.replace("OAExists", "RenamedExists"),
            "owner-test",
            "owners",
        ),
        (
            1,
            text.replacen(
                "thus the set is oamarked by OAADef",
                "thus the set is obmarked by OAADef",
                1,
            ),
            "owner-test",
            "owners",
        ),
        (2, text.clone(), "other-package", "owners"),
        (3, text.clone(), "owner-test", "other_module"),
    ] {
        let (frontend, current, entries) =
            source_registration_owner_inputs(&edited, package, module, 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(entries.diagnostics().is_empty());
        let actual = syntax(&current, entries.env()).unwrap();
        assert_eq!(actual.len(), 3);
        for entry in entries
            .env()
            .symbols()
            .iter()
            .filter(|entry| entry.kind() == SymbolKind::Registration)
        {
            let name = if entry.primary_spelling() == "RenamedExists" {
                "OAExists"
            } else {
                entry.primary_spelling()
            };
            let old = collected
                .env()
                .symbols()
                .iter()
                .find(|old| {
                    old.kind() == SymbolKind::Registration && old.primary_spelling() == name
                })
                .unwrap();
            if index >= 2 || name == "OAExists" {
                assert_ne!(actual.get(entry.symbol()), baseline.get(old.symbol()));
            } else {
                assert_eq!(actual.get(entry.symbol()), baseline.get(old.symbol()));
            }
        }
    }
    assert_eq!((source, collected, baseline), before);
}

#[test]
fn source_registration_syntax_preserves_upstream_errors_and_refuses_real_prefix_payload() {
    use mizar_resolve::symbols::{
        source_registration_owner_payloads_from_arena as owners,
        source_registration_syntax_payloads_from_arena as syntax, validate_source_symbol_env,
    };
    use mizar_syntax::{SurfaceFormulaPrefixOperator, SurfaceNodeKind};
    let text = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let (frontend, source, collected) =
        source_registration_owner_inputs(&text, "owner-test", "owners", 0);
    let prior = syntax(&source, collected.env()).unwrap();
    let before = (
        frontend.ast.clone(),
        source.clone(),
        collected.clone(),
        prior.clone(),
    );
    for edited in [text.replace("wmarked", "foreignmarked"),
        format!("{text}\ndefinition\n let X be set;\n func ExtraDef: extra X -> set equals X;\n coherence;\nend;\n")] {
        let (other_frontend, other_source, other) = source_registration_owner_inputs(&edited, "owner-test", "owners", 0);
        assert!(other_frontend.diagnostics.is_empty());
        assert!(other.diagnostics().is_empty());
        let other_before = (other_frontend.ast.clone(), other_source.clone(), other.clone());
        let error = validate_source_symbol_env(&source, other.env()).unwrap_err();
        assert_eq!(syntax(&source, other.env()).unwrap_err(), error);
        let error = validate_source_symbol_env(&other_source, collected.env()).unwrap_err();
        assert_eq!(syntax(&other_source, collected.env()).unwrap_err(), error);
        assert_eq!((other_frontend.ast, other_source, other), other_before);
    }
    for (edited, owner_refusal) in [
        (text.replace("means X = X", "means X ="), false),
        (format!("{text}\nregistration\n cluster WMarkedExists: wmarked set;\n existence proof take the set; thus the set is wmarked by WMDef; end;\nend;\n"), false),
        (text.replace("end;\n\nregistration", "  attr WMDef: X is othermarked means X = X;\nend;\n\nregistration"), true),
        (format!("{text}\nregistration\n cluster Conditional: wmarked -> wmarked for set;\n coherence;\nend;\n"), true),
    ] {
        let (bad_frontend, bad_source, bad) = source_registration_owner_inputs(&edited, "owner-test", "owners", 0);
        let bad_before = (bad_frontend.ast.clone(), bad_source.clone(), bad.clone());
        let upstream = owners(&bad_source, bad.env()).unwrap_err();
        if owner_refusal {
            assert!(bad_frontend.diagnostics.is_empty());
            assert!(bad.diagnostics().is_empty());
            validate_source_symbol_env(&bad_source, bad.env()).unwrap();
            assert_eq!(upstream, "unsupported source registration owner payload");
        } else {
            assert_eq!(upstream, validate_source_symbol_env(&bad_source, bad.env()).unwrap_err());
        }
        assert_eq!(syntax(&bad_source, bad.env()).unwrap_err(), upstream);
        assert_eq!((bad_frontend.ast, bad_source, bad), bad_before);
    }
    let edited = text.replace("thus the set is wmarked", "thus not the set is wmarked");
    let (unsupported_frontend, unsupported_source, unsupported) =
        source_registration_owner_inputs(&edited, "owner-test", "owners", 0);
    assert!(unsupported_frontend.diagnostics.is_empty());
    assert!(unsupported.diagnostics().is_empty());
    validate_source_symbol_env(&unsupported_source, unsupported.env()).unwrap();
    owners(&unsupported_source, unsupported.env()).unwrap();
    assert!(unsupported_frontend
        .ast
        .as_ref()
        .unwrap()
        .node_views()
        .any(|node| matches!(
            node.kind(),
            SurfaceNodeKind::PrefixFormula(SurfaceFormulaPrefixOperator::Not)
        )));
    let unsupported_before = (
        unsupported_frontend.ast.clone(),
        unsupported_source.clone(),
        unsupported.clone(),
    );
    assert_eq!(
        syntax(&unsupported_source, unsupported.env()).unwrap_err(),
        "unsupported source registration syntax payload"
    );
    assert_eq!(
        (unsupported_frontend.ast, unsupported_source, unsupported),
        unsupported_before
    );
    assert_eq!(syntax(&source, collected.env()).unwrap(), prior);
    assert_eq!((frontend.ast, source, collected, prior), before);
}

#[test]
fn source_registration_source_shape_exact_current_leaves_and_default_discharge() {
    use mizar_vc::discharge::{DischargeInput, DischargePolicy, try_discharge};
    // Spec 6.2/17.2-17.3; independent pre-Rust grammar/framing/hash literals.
    let expected = [
        ("WMarkedExists:0", "093d0dadf9f2aa9230440a7f7bf470a6ebb250c4f54331b25183329cd478a4c1"),
        ("WMarkedExists:1", "093d0dadf9f2aa92ac93248f33789946ebb250c4f54331b2b45d3b8d4944b75b"),
        ("OAExists:0", "a03b1cace69c8010524b10ab52e1a4ac28192d4024c8ddfc74cd191d362b6833"),
        ("OAExists:1", "a03b1cace69c80105180874e8f354fc928192d4024c8ddfc0631a2a7ab47c84a"),
        ("OBExists:0", "8fee99cbb2eaec3ad96796514ee9ebc041e01cca2dc6a22de4125a9c3fc6f956"),
        ("OBExists:1", "8fee99cbb2eaec3a19a90b8cf13025a841e01cca2dc6a22d37a9ed67060be14f"),
        ("OABExists:0", "7a2e0049775c610ccd178dc7f5ebd30a610d8884dfbdeb09e69240073f75c5cc"),
        ("OABExists:1", "7a2e0049775c610c4e61ea6952aa4283610d8884dfbdeb092a6794ef0d44f5a2"),
    ];
    for (count, case) in [(2, step5c3_registration_case()), (6, step5c13_registration_case())] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (core, vcs, formulas, owners, syntax) = source_registration_source_shape_inputs(&text, "vc_fingerprints", 0);
        let before = (core.clone(), vcs.clone(), formulas.clone(), syntax.clone());
        let hashes = vcs.source_registration_source_shape_hashes(&core, &formulas, &syntax).unwrap();
        assert_eq!(hashes.len(), count);
        assert_eq!(hashes.keys().collect::<Vec<_>>(), owners.keys().collect::<Vec<_>>());
        for (id, owner) in &owners {
            let hex = hashes[id].as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
            assert_eq!(hex, expected.iter().find(|(name, _)| name == owner).unwrap().1);
            if let Some(label) = owner.strip_suffix(":0") {
                let other = *owners.iter().find(|(_, name)| **name == format!("{label}:1")).unwrap().0;
                assert_ne!(hashes[id], hashes[&other]);
                assert_eq!(vcs.vc(*id).unwrap().anchor.owner, vcs.vc(other).unwrap().anchor.owner);
            }
        }
        let discharged = try_discharge(DischargeInput { vc_set: &vcs, policy: &DischargePolicy::default() }).unwrap();
        let discharged_before = discharged.vc_set().clone();
        assert_eq!(discharged.vc_set().source_registration_source_shape_hashes(&core, &formulas, &syntax).unwrap(), hashes);
        assert_eq!(discharged.vc_set(), &discharged_before);
        assert_eq!((core, vcs, formulas, syntax), before);
    }
}

#[test]
fn source_registration_source_shape_neutral_relocation_and_participating_edits() {
    use mizar_vc::{
        generator::generate_source_existential_registration as generate,
        vc_ir::{AnchorOwner, GenerationSchemaVersion, VcSchemaVersion},
    };
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let (core, vcs, formulas, owners, syntax) = source_registration_source_shape_inputs(&text, "vc_fingerprints", 0);
    let before = (core.clone(), vcs.clone(), formulas.clone(), syntax.clone());
    let hashes = vcs.source_registration_source_shape_hashes(&core, &formulas, &syntax).unwrap();
    let a = "  attr OAADef: X is oamarked means X = X;\n";
    let b = "  attr OBBDef: X is obmarked means X = X;\n";
    let ra = text.find("  cluster OAExists:").unwrap();
    let rb = text.find("  cluster OBExists:").unwrap();
    let rab = text.find("  cluster OABExists:").unwrap();
    let end = text[rab..].find("end;\nend;").unwrap() + rab + "end;\n".len();
    let reordered =
        format!("{}{}{}{}{}", &text[..ra], &text[rab..end], &text[ra..rb], &text[rb..rab], &text[end..]).replace(&format!("{a}{b}"), &format!("{b}{a}"));
    assert_ne!(reordered, text);
    for (index, edited, skip) in [
        (0, format!("\n:: neutral trivia\n{}", text.replace("  ", "    ")), 0),
        (1, text.replace('X', "OuterBinder"), 0),
        (2, text.replacen("means X = X", "means not X = X", 1), 0),
        (3, text.clone(), 7),
        (4, reordered, 0),
    ] {
        let (other_core, other_vcs, other_formulas, other_owners, other_syntax) = source_registration_source_shape_inputs(&edited, "vc_fingerprints", skip);
        let other_before = (other_core.clone(), other_vcs.clone(), other_formulas.clone(), other_syntax.clone());
        let actual = other_vcs.source_registration_source_shape_hashes(&other_core, &other_formulas, &other_syntax).unwrap();
        for (id, name) in &owners {
            let current = *other_owners.iter().find(|(_, other)| *other == name).unwrap().0;
            assert_eq!(actual[&current], hashes[id], "neutral {index}: {name}");
            if index == 4 && name == "OABExists:1" {
                assert_ne!(current, *id);
                let AnchorOwner::Registration(old_item) = vcs.vc(*id).unwrap().anchor.owner else { unreachable!() };
                let AnchorOwner::Registration(item) = other_vcs.vc(current).unwrap().anchor.owner else { unreachable!() };
                assert_ne!(item, old_item);
                assert_ne!(other_core.items().get(item).unwrap().symbol, core.items().get(old_item).unwrap().symbol);
                assert_ne!(other_vcs.vc(current).unwrap().goal, vcs.vc(*id).unwrap().goal);
            }
        }
        if index == 4 {
            let (_, old_source, _) = source_registration_owner_inputs(&text, "vc-fingerprint-test", "vc_fingerprints", 0);
            let (_, relocated, _) = source_registration_owner_inputs(&edited, "vc-fingerprint-test", "vc_fingerprints", 0);
            let old_nodes = old_source.arena().iter().filter_map(|(id, node)|
                matches!(node.kind(), mizar_syntax::SurfaceNodeKind::ExistentialRegistration).then_some(id)).collect::<Vec<_>>();
            let current_nodes = relocated.arena().iter().filter_map(|(id, node)|
                matches!(node.kind(), mizar_syntax::SurfaceNodeKind::ExistentialRegistration).then_some(id)).collect::<Vec<_>>();
            assert_ne!(current_nodes, old_nodes);
        }
        if skip != 0 {
            assert_ne!(other_core.source_id(), core.source_id());
        }
        assert_eq!((other_core, other_vcs, other_formulas, other_syntax), other_before);
    }
    for (index, edited, module) in [
        (0, text.replace("OAExists", "RenamedExists"), "vc_fingerprints"),
        (1, text.replace("oamarked", "othermarked"), "vc_fingerprints"),
        (2, text.clone(), "other_module"),
    ] {
        let (other_core, other_vcs, other_formulas, other_owners, other_syntax) = source_registration_source_shape_inputs(&edited, module, 0);
        let other_before = (other_core.clone(), other_vcs.clone(), other_formulas.clone(), other_syntax.clone());
        let actual = other_vcs.source_registration_source_shape_hashes(&other_core, &other_formulas, &other_syntax).unwrap();
        for (id, name) in &owners {
            let current_name = if index == 0 { name.replace("OAExists:", "RenamedExists:") } else { name.clone() };
            let current = other_owners.iter().find(|(_, other)| **other == current_name).unwrap().0;
            let changed = match index {
                0 => name.starts_with("OAExists:"),
                1 => !name.starts_with("OBExists:"),
                _ => true,
            };
            assert_eq!(actual[current] != hashes[id], changed, "participating {index}: {name}");
        }
        assert_eq!((other_core, other_vcs, other_formulas, other_syntax), other_before);
    }
    for (snapshot, generation, schema, changed) in [
        (41, "source-vc-fingerprint-test", "source-vc-fingerprint-v1", false),
        (0, "other-generation", "source-vc-fingerprint-v1", true),
        (0, "source-vc-fingerprint-test", "other-vc-schema", true),
    ] {
        let other = generate(&core, super::super::shared::snapshot_id(snapshot), &GenerationSchemaVersion::new(generation), &VcSchemaVersion::new(schema)).unwrap();
        let other_before = other.clone();
        let actual = other.source_registration_source_shape_hashes(&core, &formulas, &syntax).unwrap();
        for id in owners.keys() {
            assert_eq!(actual[id] != hashes[id], changed);
        }
        assert_eq!(other, other_before);
    }
    assert_eq!((core, vcs, formulas, syntax), before);
}

#[test]
fn source_registration_source_shape_refuses_whole_inputs_and_invalid_syntax_maps() {
    use mizar_core::core_ir::CoreItemKind;
    use mizar_vc::vc_ir::{VcSet, VcSetParts, VcText};
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let (core, vcs, formulas, _, syntax) = source_registration_source_shape_inputs(&text, "vc_fingerprints", 0);
    let before = (core.clone(), vcs.clone(), formulas.clone(), syntax.clone());
    let valid = vcs.source_registration_source_shape_hashes(&core, &formulas, &syntax).unwrap();
    let first = *syntax.keys().next().unwrap();
    let second = *syntax.keys().nth(1).unwrap();
    let extra = core.items().iter().find(|(_, item)| item.kind != CoreItemKind::Registration).unwrap().0;
    for index in 0..4 {
        let mut bad = syntax.clone();
        match index {
            0 => {
                bad.remove(&first);
            }
            1 => {
                bad.insert(extra, b"extra".to_vec());
            }
            2 => {
                bad.insert(first, Vec::new());
            }
            _ => {
                bad.insert(first, syntax[&second].clone());
            }
        }
        let bad_before = bad.clone();
        assert_eq!(vcs.source_registration_source_shape_hashes(&core, &formulas, &bad).unwrap_err(), "unsupported source registration source shape");
        assert_eq!(bad, bad_before);
        assert_eq!(vcs.source_registration_source_shape_hashes(&core, &formulas, &syntax).unwrap(), valid);
    }
    let empty = std::collections::BTreeMap::new();
    let empty_syntax = std::collections::BTreeMap::new();
    let upstream = vcs.source_registration_canonical_fingerprints(&core, &empty).unwrap_err();
    assert_eq!(vcs.source_registration_source_shape_hashes(&core, &empty, &empty_syntax).unwrap_err(), upstream);
    let (foreign_core, foreign_vcs, foreign_formulas, _, foreign_syntax) =
        source_registration_source_shape_inputs(&text.replace("oamarked", "foreignmarked"), "vc_fingerprints", 0);
    let foreign_before = (foreign_core.clone(), foreign_vcs.clone(), foreign_formulas.clone(), foreign_syntax.clone());
    for (set, graph, roots) in [(&vcs, &foreign_core, &formulas), (&foreign_vcs, &core, &foreign_formulas)] {
        let upstream = set.source_registration_canonical_fingerprints(graph, roots).unwrap_err();
        assert_eq!(set.source_registration_source_shape_hashes(graph, roots, &empty_syntax).unwrap_err(), upstream);
    }
    assert_eq!((foreign_core, foreign_vcs, foreign_formulas, foreign_syntax), foreign_before);
    let mut parts = VcSetParts {
        schema_version: vcs.schema_version().clone(),
        snapshot: vcs.snapshot(),
        source: vcs.source(),
        module: vcs.module().clone(),
        generated_formulas: vcs.generated_formulas().to_vec(),
        vcs: vcs.vcs().to_vec(),
        seed_accounting: vcs.seed_accounting().to_vec(),
    };
    let empty_set = VcSet::try_new(VcSetParts { vcs: Vec::new(), generated_formulas: Vec::new(), seed_accounting: Vec::new(), ..parts.clone() }).unwrap();
    let upstream = empty_set.source_registration_canonical_fingerprints(&core, &formulas).unwrap_err();
    assert_eq!(empty_set.source_registration_source_shape_hashes(&core, &formulas, &empty_syntax).unwrap_err(), upstream);
    parts.vcs.last_mut().unwrap().provenance[0].key = VcText::new("altered last leaf");
    let bad = VcSet::try_new(parts).unwrap();
    let bad_before = bad.clone();
    let upstream = bad.source_registration_canonical_fingerprints(&core, &formulas).unwrap_err();
    assert_eq!(bad.source_registration_source_shape_hashes(&core, &formulas, &syntax).unwrap_err(), upstream);
    assert_eq!(bad, bad_before);
    let ra = text.find("  cluster OAExists:").unwrap();
    let rb = text.find("  cluster OBExists:").unwrap();
    let rab = text.find("  cluster OABExists:").unwrap();
    let swapped = format!("{}{}{}{}", &text[..ra], &text[rb..rab], &text[ra..rb], &text[rab..]);
    let (source, nodes, collected, _) = source_registration_formula_inputs(&swapped, "vc-fingerprint-test", "vc_fingerprints", 0);
    let check = mizar_checker::registration_resolution::check_source_existential_registration_proof(&source, &nodes, collected.env()).unwrap();
    let unsupported = mizar_core::elaborator::lower_source_existential_registration(&check).unwrap();
    let unsupported_before = unsupported.clone();
    let upstream = vcs.source_registration_canonical_fingerprints(&unsupported, &formulas).unwrap_err();
    assert_ne!(upstream, "unsupported source registration VC fingerprint");
    assert_eq!(vcs.source_registration_source_shape_hashes(&unsupported, &formulas, &empty_syntax).unwrap_err(), upstream);
    assert_eq!(unsupported, unsupported_before);
    assert_eq!((core, vcs, formulas, syntax), before);
}

#[test]
fn source_registration_candidate_anchor_exact_current_leaves_and_producer_equality() {
    use mizar_proof::status::{source_registration_candidate_anchor as anchor, source_registration_canonical_components as components};
    use mizar_resolve::symbols::source_registration_owner_payloads_from_arena;
    use mizar_vc::{discharge::{DischargeInput, DischargePolicy, try_discharge}, generator::generate_source_existential_registration as generate, vc_ir::{GenerationSchemaVersion, VcSchemaVersion}};
    // Spec 6.2/17.2–17.3; independently frozen before Rust, with existing Proof schemas.
    let expected = [
        ("WMarkedExists:0", "source-registration-candidate-anchor-v1:a61a931461b783629037ec7da1d22cc11f4e20d9a38cb82c33e8caba96443b97"),
        ("WMarkedExists:1", "source-registration-candidate-anchor-v1:e0fbdfe1b0777c0ca9f8db22079af89634455500f11bcec129d65be7aaa4a0d3"),
        ("OAExists:0", "source-registration-candidate-anchor-v1:2acda8e7312b142d64afd3ceb739ffa3b408be2347daa17ee00621ddd40cdd9f"),
        ("OAExists:1", "source-registration-candidate-anchor-v1:f90669c4bb9505f540e656f5abc6eb0aad1a4779723572aabc1333a039a3348f"),
        ("OBExists:0", "source-registration-candidate-anchor-v1:2c00d66e6ac6f4de5ae8d2f0b316f853e9d14e4ed9870d116f57601dd79926aa"),
        ("OBExists:1", "source-registration-candidate-anchor-v1:20c58bed6ea0c7f9e088509c4d387e845dda890f0498d35f959ca047203b6ac9"),
        ("OABExists:0", "source-registration-candidate-anchor-v1:719a0eb5e27775666a1621434e5ab91709db050488c16d070f62e8608afde693"),
        ("OABExists:1", "source-registration-candidate-anchor-v1:27aa46f5336c191e20ee6bb8197f8134b97d04fe2b20bbe87c0458dabb8dd2ef"),
    ];
    for (count, case) in [(2, step5c3_registration_case()), (6, step5c13_registration_case())] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (core, _, formulas, leaves, syntax) = source_registration_source_shape_inputs(&text, "vc_fingerprints", 0);
        let (source, nodes, collected, _) = source_registration_formula_inputs(&text, "vc-fingerprint-test", "vc_fingerprints", 0);
        let actual_owners = source_registration_owner_payloads_from_arena(&source, collected.env()).unwrap();
        let owners: std::collections::BTreeMap<_, _> = core.items().iter().filter(|(_, item)| item.kind == mizar_core::core_ir::CoreItemKind::Registration)
            .map(|(id, item)| (id, actual_owners[&item.symbol].clone())).collect();
        let before = (source.clone(), nodes.clone(), collected.env().clone(), core.clone(), formulas.clone(), syntax.clone(), owners.clone());
        for snapshot in [super::super::shared::snapshot_id(0), super::super::shared::snapshot_id(41)] {
            let vcs = generate(&core, snapshot, &GenerationSchemaVersion::new("source-existential-registration-v1"), &VcSchemaVersion::new("vc-v1")).unwrap();
            let original = vcs.clone();
            let hashes = vcs.source_registration_candidate_anchor_hashes(&core, &formulas, &syntax, &owners).unwrap();
            let full = vcs.source_registration_canonical_fingerprints(&core, &formulas).unwrap();
            assert_eq!(hashes.len(), count);
            assert_eq!(hashes.keys().collect::<Vec<_>>(), leaves.keys().collect::<Vec<_>>());
            assert_eq!(hashes.values().map(|hash| hash.as_bytes()).collect::<std::collections::BTreeSet<_>>().len(), count);
            for (id, name) in &leaves {
                let result = anchor(&source, &nodes, collected.env(), snapshot, *id).unwrap();
                assert_eq!(result.0.as_str(), expected.iter().find(|(owner, _)| owner == name).unwrap().1);
                let hex = hashes[id].as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
                assert_eq!(result.0.as_str(), format!("source-registration-candidate-anchor-v1:{hex}"));
                let pair = vcs.source_registration_canonical_goal_context(*id, &core, &formulas).unwrap();
                assert_eq!((result.1, result.2, result.3), (full[id], pair.0, pair.1));
                assert_eq!((result.1, result.2, result.3), components(&source, &nodes, collected.env(), snapshot, *id).unwrap());
                assert!(vcs.vc(*id).unwrap().anchor.label.is_none());
            }
            let discharged = try_discharge(DischargeInput { vc_set: &vcs, policy: &DischargePolicy::default() }).unwrap();
            let discharged_before = discharged.vc_set().clone();
            assert_eq!(discharged.vc_set().source_registration_candidate_anchor_hashes(&core, &formulas, &syntax, &owners).unwrap(), hashes);
            assert_eq!(discharged.vc_set(), &discharged_before);
            assert_eq!(vcs, original);
        }
        assert_eq!((source, nodes, collected.env().clone(), core, formulas, syntax, owners), before);
    }
}

#[test]
fn source_registration_candidate_anchor_neutral_relocation_and_participating_edits() {
    use mizar_proof::status::{source_registration_candidate_anchor as anchor, source_registration_canonical_components as components};
    use mizar_vc::{generator::generate_source_existential_registration as generate, vc_ir::{AnchorOwner, GenerationSchemaVersion, VcSchemaVersion}};
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let a = "  attr OAADef: X is oamarked means X = X;\n";
    let b = "  attr OBBDef: X is obmarked means X = X;\n";
    let ra = text.find("  cluster OAExists:").unwrap();
    let rb = text.find("  cluster OBExists:").unwrap();
    let rab = text.find("  cluster OABExists:").unwrap();
    let end = text[rab..].find("end;\nend;").unwrap() + rab + "end;\n".len();
    let reordered = format!("{}{}{}{}{}", &text[..ra], &text[rab..end], &text[ra..rb], &text[rb..rab], &text[end..]).replace(&format!("{a}{b}"), &format!("{b}{a}"));
    assert_ne!(reordered, text);
    let mut baseline = std::collections::BTreeMap::new();
    for (index, edited, module, skip) in [
        (0, text.clone(), "vc_fingerprints", 0),
        (1, format!("\n:: neutral trivia\n{}", text.replace("  ", "    ")), "vc_fingerprints", 0),
        (2, text.replace('X', "OuterBinder"), "vc_fingerprints", 0),
        (3, text.clone(), "vc_fingerprints", 7),
        (4, reordered, "vc_fingerprints", 0),
        (5, text.replace("OAExists", "RenamedExists"), "vc_fingerprints", 0),
        (6, text.clone(), "other_module", 0),
        (7, text.replace("oamarked", "othermarked"), "vc_fingerprints", 0),
        (8, text.replacen("means X = X", "means not X = X", 1), "vc_fingerprints", 0),
    ] {
        let (core, _, formulas, leaves, syntax) = source_registration_source_shape_inputs(&edited, module, skip);
        let (source, nodes, collected, _) = source_registration_formula_inputs(&edited, "vc-fingerprint-test", module, skip);
        let actual_owners = mizar_resolve::symbols::source_registration_owner_payloads_from_arena(&source, collected.env()).unwrap();
        let owners: std::collections::BTreeMap<_, _> = core.items().iter().filter(|(_, item)| item.kind == mizar_core::core_ir::CoreItemKind::Registration)
            .map(|(id, item)| (id, actual_owners[&item.symbol].clone())).collect();
        let snapshot = super::super::shared::snapshot_id(41);
        let vcs = generate(&core, snapshot, &GenerationSchemaVersion::new("source-existential-registration-v1"), &VcSchemaVersion::new("vc-v1")).unwrap();
        let before = (source.clone(), nodes.clone(), collected.env().clone(), core.clone(), vcs.clone(), formulas.clone(), syntax.clone(), owners.clone());
        let hashes = vcs.source_registration_candidate_anchor_hashes(&core, &formulas, &syntax, &owners).unwrap();
        let shapes = vcs.source_registration_source_shape_hashes(&core, &formulas, &syntax).unwrap();
        for (id, name) in &leaves {
            let result = anchor(&source, &nodes, collected.env(), snapshot, *id).unwrap();
            let triple = components(&source, &nodes, collected.env(), snapshot, *id).unwrap();
            assert_eq!((result.1, result.2, result.3), triple);
            let hex = hashes[id].as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
            assert_eq!(result.0.as_str(), format!("source-registration-candidate-anchor-v1:{hex}"));
            let AnchorOwner::Registration(item) = vcs.vc(*id).unwrap().anchor.owner else { unreachable!() };
            let current = (result, hashes[id], shapes[id], *id, item, vcs.vc(*id).unwrap().goal, core.source_id());
            let name = if index == 5 { name.replace("RenamedExists:", "OAExists:") } else { name.clone() };
            if index == 0 { baseline.insert(name, current); continue; }
            let old = &baseline[&name];
            let changed = match index { 5 => name.starts_with("OAExists:"), 6 => true, 7 => !name.starts_with("OBExists:"), 8 => name == "OAExists:1" || name == "OABExists:1", _ => false };
            assert_eq!(current.0.0 != old.0.0, changed, "anchor {index}: {name}");
            assert_eq!(current.1 != old.1, changed, "lower {index}: {name}");
            if index < 5 { assert_eq!(current.0, old.0, "neutral {index}: {name}"); }
            if index == 3 { assert_ne!(current.6, old.6); }
            if index == 4 && name == "OABExists:1" {
                assert_ne!(current.3, old.3);
                assert_ne!(current.4, old.4);
                assert_ne!(current.5, old.5);
            }
            if index == 8 {
                assert_eq!(current.2, old.2, "external body leaves local shape: {name}");
                assert_eq!(current.0.2, old.0.2, "external body leaves goal: {name}");
                assert_eq!(current.0.3 != old.0.3, changed, "external context: {name}");
            }
        }
        assert_eq!((source, nodes, collected.env().clone(), core, vcs, formulas, syntax, owners), before);
    }
}

#[test]
fn source_registration_candidate_anchor_refuses_whole_inputs_before_selection() {
    use mizar_checker::registration_resolution::check_source_existential_registration_proof as check;
    use mizar_proof::status::source_registration_candidate_anchor as anchor;
    use mizar_vc::{generator::generate_source_existential_registration as generate, vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion}};
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let (core, _, formulas, leaves, syntax) = source_registration_source_shape_inputs(&text, "vc_fingerprints", 0);
    let (source, nodes, collected, _) = source_registration_formula_inputs(&text, "vc-fingerprint-test", "vc_fingerprints", 0);
    let actual_owners = mizar_resolve::symbols::source_registration_owner_payloads_from_arena(&source, collected.env()).unwrap();
    let owners: std::collections::BTreeMap<_, _> = core.items().iter().filter(|(_, item)| item.kind == mizar_core::core_ir::CoreItemKind::Registration)
        .map(|(id, item)| (id, actual_owners[&item.symbol].clone())).collect();
    let snapshot = super::super::shared::snapshot_id(0);
    let vcs = generate(&core, snapshot, &GenerationSchemaVersion::new("source-existential-registration-v1"), &VcSchemaVersion::new("vc-v1")).unwrap();
    let before = (source.clone(), nodes.clone(), collected.env().clone(), core.clone(), vcs.clone(), formulas.clone(), syntax.clone(), owners.clone());
    let valid = vcs.source_registration_candidate_anchor_hashes(&core, &formulas, &syntax, &owners).unwrap();
    let selected = *leaves.keys().next().unwrap();
    let absent = VcId::new(vcs.vcs().len());
    assert_eq!(anchor(&source, &nodes, collected.env(), snapshot, absent).unwrap_err(), "registration.proof.invalid_source_anchor");
    let first = *owners.keys().next().unwrap();
    let second = *owners.keys().nth(1).unwrap();
    let extra = core.items().iter().find(|(_, item)| item.kind != mizar_core::core_ir::CoreItemKind::Registration).unwrap().0;
    for index in 0..4 {
        let mut bad = owners.clone();
        match index {
            0 => { bad.remove(&first); }
            1 => { bad.insert(extra, b"extra".to_vec()); }
            2 => { bad.insert(first, Vec::new()); }
            _ => { bad.insert(first, owners[&second].clone()); }
        }
        let bad_before = bad.clone();
        assert_eq!(vcs.source_registration_candidate_anchor_hashes(&core, &formulas, &syntax, &bad).unwrap_err(), "unsupported source registration candidate anchor");
        assert_eq!(bad, bad_before);
        assert_eq!(vcs.source_registration_candidate_anchor_hashes(&core, &formulas, &syntax, &owners).unwrap(), valid);
    }
    let empty = std::collections::BTreeMap::new();
    let upstream = vcs.source_registration_source_shape_hashes(&core, &formulas, &empty).unwrap_err();
    assert_eq!(vcs.source_registration_candidate_anchor_hashes(&core, &formulas, &empty, &empty).unwrap_err(), upstream);
    let empty_formulas = std::collections::BTreeMap::new();
    let upstream = vcs.source_registration_source_shape_hashes(&core, &empty_formulas, &syntax).unwrap_err();
    assert_eq!(vcs.source_registration_candidate_anchor_hashes(&core, &empty_formulas, &syntax, &empty).unwrap_err(), upstream);
    let (foreign_source, foreign_nodes, foreign_env, _) = source_registration_formula_inputs(&text.replace("oamarked", "foreignmarked"), "vc-fingerprint-test", "vc_fingerprints", 0);
    let foreign_before = (foreign_source.clone(), foreign_nodes.clone(), foreign_env.env().clone());
    for (s, n, e) in [(&foreign_source, &nodes, collected.env()), (&source, &foreign_nodes, collected.env()), (&source, &nodes, foreign_env.env())] {
        let upstream = check(s, n, e).unwrap_err();
        assert_ne!(upstream, "registration.proof.invalid_source_anchor");
        for id in [selected, absent] { assert_eq!(anchor(s, n, e, snapshot, id).unwrap_err(), upstream); }
    }
    assert_eq!((foreign_source, foreign_nodes, foreign_env.env().clone()), foreign_before);
    let ra = text.find("  cluster OAExists:").unwrap();
    let rb = text.find("  cluster OBExists:").unwrap();
    let rab = text.find("  cluster OABExists:").unwrap();
    let swapped = format!("{}{}{}{}", &text[..ra], &text[rb..rab], &text[ra..rb], &text[rab..]);
    let malformed = text.replacen("    take the set;\n", "", 1);
    for (index, edited) in [malformed, swapped].into_iter().enumerate() {
        assert_ne!(edited, text);
        let (s, n, e, _) = source_registration_formula_inputs(&edited, "vc-fingerprint-test", "vc_fingerprints", 0);
        let input_before = (s.clone(), n.clone(), e.env().clone());
        let upstream = if index == 0 { check(&s, &n, e.env()).unwrap_err() } else {
            let checked = check(&s, &n, e.env()).unwrap();
            let unsupported = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
            let unsupported_before = unsupported.clone();
            let upstream = vcs.source_registration_source_shape_hashes(&unsupported, &formulas, &syntax).unwrap_err();
            assert_eq!(vcs.source_registration_candidate_anchor_hashes(&unsupported, &formulas, &syntax, &empty).unwrap_err(), upstream);
            let failure = generate(&unsupported, snapshot, &GenerationSchemaVersion::new("source-existential-registration-v1"), &VcSchemaVersion::new("vc-v1")).unwrap_err();
            assert_eq!(failure, upstream);
            assert_eq!(unsupported, unsupported_before);
            failure
        };
        assert_ne!(upstream, "registration.proof.invalid_source_anchor");
        for id in [selected, absent] { assert_eq!(anchor(&s, &n, e.env(), snapshot, id).unwrap_err(), upstream); }
        assert_eq!((s, n, e.env().clone()), input_before);
    }
    assert_eq!((source, nodes, collected.env().clone(), core, vcs, formulas, syntax, owners), before);
}

#[test]
fn source_registration_observed_ground_terms_preserve_actual_choices_inputs_and_encodings() {
    use mizar_atp::{
        problem::*,
        smtlib_encoder::{SmtLibDialect, encode_smtlib},
        tptp_encoder::{TptpDialect, encode_tptp},
        translator::translate_source_existential_registration as translate,
    };
    use mizar_core::core_ir::CoreTermKind;
    use mizar_vc::vc_ir::VcStatus;
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original.replace("marked", "retagged").replace("X", "Value");
        let negative = original.replace("means X = X", "means not X = X");
        assert_ne!(renamed, original);
        assert_ne!(negative, original);
        for text in [&original, &renamed, &negative] {
            let (core, _, vcs) = source_candidate_core_and_vcs(multiple, text);
            let before_core = core.clone();
            let before_vcs = vcs.clone();
            let origins = core
                .generated()
                .iter()
                .map(|(_, row)| row)
                .collect::<Vec<_>>();
            let selected = vcs
                .vcs()
                .iter()
                .filter(|vc| vc.status == VcStatus::NeedsAtp)
                .collect::<Vec<_>>();
            assert_eq!(selected.len(), if multiple { 3 } else { 1 });
            assert_eq!(selected.len(), origins.len());
            for (registration, vc) in selected.iter().enumerate() {
                let functor = origins[registration].functor.as_ref().unwrap();
                let expected = AtpTerm::Function {
                    function: AtpSymbolName::new(format!("choice:{functor:?}")),
                    arguments: vec![],
                };
                assert!(core.terms().iter().any(|(_, term)| matches!(
                    &term.kind,
                    CoreTermKind::Apply { functor: actual, args }
                        if actual == functor && args.is_empty()
                )));
                for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
                    let problem = translate(
                        &core,
                        &vcs,
                        vc.id,
                        source_candidate_profile("source-observed-ground-terms", format),
                    )
                    .unwrap();
                    let before = problem.clone();
                    let encoded_before = if format == ConcreteFormat::Tptp {
                        encode_tptp(&problem, TptpDialect::Fof)
                            .unwrap()
                            .text()
                            .to_owned()
                    } else {
                        encode_smtlib(&problem, SmtLibDialect::Uninterpreted)
                            .unwrap()
                            .text()
                            .to_owned()
                    };
                    assert_eq!(
                        problem.observed_ground_terms().unwrap(),
                        std::collections::BTreeSet::from([&expected])
                    );
                    assert_eq!(problem, before);
                    let encoded_after = if format == ConcreteFormat::Tptp {
                        encode_tptp(&problem, TptpDialect::Fof)
                            .unwrap()
                            .text()
                            .to_owned()
                    } else {
                        encode_smtlib(&problem, SmtLibDialect::Uninterpreted)
                            .unwrap()
                            .text()
                            .to_owned()
                    };
                    assert_eq!(encoded_after.as_bytes(), encoded_before.as_bytes());
                }
            }
            assert_eq!(core, before_core);
            assert_eq!(vcs, before_vcs);
        }
    }
}

#[test]
fn source_registration_instance_search_limits_preserve_fresh_check_results() {
    use mizar_atp::problem::ConcreteFormat;
    use mizar_kernel::checker::KernelCheckStatus;
    use mizar_proof::{
        policy::{CandidatePolicyClass, VerifierPolicy},
        status::check_source_registration_candidate as check,
    };
    use mizar_vc::vc_ir::VcStatus;
    let original = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let snapshot = super::super::shared::snapshot_id(0);
    for negative in [false, true] {
        let text = if negative {
            original.replacen("means X = X", "means not X = X", 1)
        } else {
            original.clone()
        };
        let (source, nodes, symbols) = step5c13_registration_inputs(&text).unwrap();
        let (_, _, vcs) = source_candidate_core_and_vcs(true, &text);
        let before = (source.clone(), nodes.clone(), symbols.clone());
        let mut leaves = 0;
        for vc in vcs
            .vcs()
            .iter()
            .filter(|vc| vc.status == VcStatus::NeedsAtp)
        {
            leaves += 1;
            for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
                for policy in [
                    VerifierPolicy::release(),
                    VerifierPolicy::development(),
                    VerifierPolicy::interactive(),
                ] {
                    let profile = source_candidate_profile("instance-search-limits", format);
                    let baseline = check(
                        &source,
                        &nodes,
                        &symbols,
                        snapshot,
                        vc.id,
                        (profile.clone(), format),
                        &policy,
                    )
                    .unwrap();
                    let rejected = negative && vc.id.index() != 3;
                    assert_eq!(
                        baseline.0.status(),
                        if rejected {
                            KernelCheckStatus::Rejected
                        } else {
                            KernelCheckStatus::Accepted
                        }
                    );
                    assert_eq!(
                        baseline.1.class,
                        if rejected {
                            CandidatePolicyClass::KernelRejected
                        } else {
                            CandidatePolicyClass::KernelVerified
                        }
                    );
                    for limits in [[0; 4], [3, 5, 7, 11]] {
                        let configured = policy.clone().with_instance_search_limits(Some(limits));
                        let actual = check(
                            &source,
                            &nodes,
                            &symbols,
                            snapshot,
                            vc.id,
                            (profile.clone(), format),
                            &configured,
                        )
                        .unwrap();
                        assert_eq!(actual, baseline);
                    }
                }
            }
        }
        assert_eq!(leaves, 3);
        assert_eq!((source, nodes, symbols), before);
    }
}

#[test]
fn source_registration_screened_attempt_admission_preserves_legacy_checks() {
    use mizar_atp::{
        problem::ConcreteFormat, translator::translate_source_existential_registration,
    };
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
    };
    use mizar_proof::{
        policy::VerifierPolicy,
        status::{
            check_source_registration_candidate as check,
            try_check_source_registration_candidate as attempt,
        },
    };
    use mizar_vc::{
        kernel_evidence_handoff::build_source_existential_kernel_handoff, vc_ir::VcStatus,
    };
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (source, nodes, symbols) = if multiple {
            step5c13_registration_inputs(&text)
        } else {
            step5c3_registration_inputs(&text)
        }
        .unwrap();
        let (core, _, vcs) = source_candidate_core_and_vcs(multiple, &text);
        let before = (source.clone(), nodes.clone(), symbols.clone());
        let vc = vcs
            .vcs()
            .iter()
            .rev()
            .find(|vc| vc.status == VcStatus::NeedsAtp)
            .unwrap();
        let handoff = build_source_existential_kernel_handoff(&core, &vcs, vc.id).unwrap();
        let envelope = handoff.canonical_evidence();
        let target = envelope.target_vc();
        let mut context = FormulaEvidenceParseContext::v1(
            Fingerprint::new(target.algorithm_id, target.digest.clone()),
            KernelProfileRecord::v1(
                envelope.kernel_profile().profile_id,
                ClauseTautologyPolicy::Reject,
            ),
        );
        context.accepted_schema_versions.insert(2);
        let parsed = parse_formula_evidence(&source_candidate_wire(&handoff), &context).unwrap();
        for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
            let profile = source_candidate_profile("proof-screened-counts", format);
            let problem =
                translate_source_existential_registration(&core, &vcs, vc.id, profile.clone())
                    .unwrap();
            let exact = [
                u64::try_from(problem.observed_ground_terms().unwrap().len()).unwrap(),
                u64::try_from(parsed.substitutions().len()).unwrap(),
                u64::try_from(parsed.equality_instances().len()).unwrap(),
                1,
            ];
            assert!(exact.into_iter().all(|count| count > 0));
            let mut below = exact;
            below[1] -= 1;
            let policy = VerifierPolicy::release();
            let snapshot = super::super::shared::snapshot_id(0);
            let baseline = check(
                &source,
                &nodes,
                &symbols,
                snapshot,
                vc.id,
                (profile.clone(), format),
                &policy,
            )
            .unwrap();
            for (limits, admitted) in [
                (None, false),
                (Some([0; 4]), false),
                (Some(below), false),
                (Some(exact), true),
                (Some([u64::MAX; 4]), true),
            ] {
                let configured = policy.clone().with_instance_search_limits(limits);
                let policy_before = configured.clone();
                assert_eq!(
                    attempt(
                        &source,
                        &nodes,
                        &symbols,
                        snapshot,
                        vc.id,
                        (profile.clone(), format),
                        &configured
                    )
                    .unwrap(),
                    admitted.then(|| baseline.clone())
                );
                assert_eq!(
                    check(
                        &source,
                        &nodes,
                        &symbols,
                        snapshot,
                        vc.id,
                        (profile.clone(), format),
                        &configured
                    )
                    .unwrap(),
                    baseline
                );
                assert_eq!(configured, policy_before);
            }
        }
        assert_eq!((source, nodes, symbols), before);
    }
}

#[test]
fn source_registration_screened_attempt_matches_fresh_kernel_and_policy() {
    use mizar_atp::{
        backend::{BackendCandidatePayload, build_source_registration_candidate_inputs as build},
        problem::ConcreteFormat,
    };
    use mizar_kernel::checker::KernelCheckStatus;
    use mizar_proof::{
        policy::{CandidatePolicyClass, VerifierPolicy},
        status::{
            check_source_registration_candidate as check,
            try_check_source_registration_candidate as attempt,
        },
    };
    use mizar_vc::{
        kernel_evidence_handoff::build_source_existential_kernel_handoff, vc_ir::VcStatus,
    };
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original.replace("marked", "retagged").replace("X", "Value");
        for (formal, base) in [("X", &original), ("Value", &renamed)] {
            let negative = base.replacen(
                &format!("means {formal} = {formal}"),
                &format!("means not {formal} = {formal}"),
                1,
            );
            assert_ne!(&negative, base);
            for (false_source, text) in [(false, base), (true, &negative)] {
                let (source, nodes, symbols) = if multiple {
                    step5c13_registration_inputs(text)
                } else {
                    step5c3_registration_inputs(text)
                }
                .unwrap();
                let (core, _, vcs) = source_candidate_core_and_vcs(multiple, text);
                let before = (source.clone(), nodes.clone(), symbols.clone());
                let mut outcomes = [0; 2];
                for vc in vcs
                    .vcs()
                    .iter()
                    .filter(|vc| vc.status == VcStatus::NeedsAtp)
                {
                    let handoff =
                        build_source_existential_kernel_handoff(&core, &vcs, vc.id).unwrap();
                    for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
                        let profile = source_candidate_profile("proof-screened-results", format);
                        let policy = VerifierPolicy::release()
                            .with_instance_search_limits(Some([u64::MAX; 4]));
                        let policy_before = policy.clone();
                        let old = check(
                            &source,
                            &nodes,
                            &symbols,
                            super::super::shared::snapshot_id(0),
                            vc.id,
                            (profile.clone(), format),
                            &policy,
                        )
                        .unwrap();
                        let (_, candidate) =
                            build(&core, &vcs, vc.id, profile.clone(), format).unwrap();
                        let BackendCandidatePayload::FormulaSubstitutionBytes(bytes) =
                            candidate.payload()
                        else {
                            panic!("actual candidate bytes")
                        };
                        let normal = source_candidate_check_bytes(
                            &handoff,
                            bytes,
                            candidate.target_binding(),
                        )
                        .unwrap();
                        assert_eq!(old.0.status(), normal);
                        let actual = attempt(
                            &source,
                            &nodes,
                            &symbols,
                            super::super::shared::snapshot_id(0),
                            vc.id,
                            (profile.clone(), format),
                            &policy,
                        )
                        .unwrap();
                        if normal == KernelCheckStatus::Accepted {
                            assert_eq!(old.1.class, CandidatePolicyClass::KernelVerified);
                            assert_eq!(actual, Some(old));
                            outcomes[0] += 1;
                        } else {
                            assert_eq!(old.1.class, CandidatePolicyClass::KernelRejected);
                            assert_eq!(actual, None);
                            outcomes[1] += 1;
                        }
                        assert_eq!(policy, policy_before);
                    }
                }
                let leaves = if multiple { 3 } else { 1 };
                let rejected = if false_source {
                    if multiple { 2 } else { 1 }
                } else {
                    0
                };
                assert_eq!(outcomes, [(leaves - rejected) * 2, rejected * 2]);
                assert_eq!((source, nodes, symbols), before);
            }
        }
    }
}

#[test]
fn source_registration_screened_attempt_preserves_lower_errors_and_absence_before_permission() {
    use mizar_atp::problem::*;
    use mizar_proof::{
        policy::{KernelEvidenceFormat, VerifierPolicy},
        status::{
            check_source_registration_candidate as check,
            try_check_source_registration_candidate as attempt,
        },
    };
    use mizar_vc::vc_ir::{VcId, VcStatus};
    let text = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let (source, nodes, symbols) = step5c3_registration_inputs(&text).unwrap();
    let (foreign_source, foreign_nodes, foreign_symbols) =
        step5c3_registration_inputs(&text.replace("wmarked", "foreignmarked")).unwrap();
    let unsupported_text = text.replacen("take the set;", "take the wmarked set;", 1);
    assert_ne!(unsupported_text, text);
    let (bad_source, bad_nodes, bad_symbols) =
        step5c3_registration_inputs(&unsupported_text).unwrap();
    let before = (
        source.clone(),
        nodes.clone(),
        symbols.clone(),
        foreign_source.clone(),
        foreign_nodes.clone(),
        foreign_symbols.clone(),
        bad_source.clone(),
        bad_nodes.clone(),
        bad_symbols.clone(),
    );
    let (_, _, vcs) = source_candidate_core_and_vcs(false, &text);
    let target = vcs
        .vcs()
        .iter()
        .find(|vc| vc.status == VcStatus::NeedsAtp)
        .unwrap()
        .id;
    let discharged = vcs
        .vcs()
        .iter()
        .find(|vc| matches!(vc.status, VcStatus::Discharged { .. }))
        .unwrap()
        .id;
    let snapshot = super::super::shared::snapshot_id(0);
    let policy = VerifierPolicy::release();
    let format = ConcreteFormat::Tptp;
    let profile = source_candidate_profile("proof-screened-order", format);
    let unsupported = LogicProfile::try_new(
        "unsupported",
        LogicFragment::TffLike,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::GuardPredicates,
        NativePropertySupport::Unsupported,
        [format].into_iter().collect(),
    )
    .unwrap();
    for denied in [
        policy.clone().with_kernel_evidence_formats([]),
        policy
            .clone()
            .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]),
    ] {
        for limits in [None, Some([0; 4]), Some([u64::MAX; 4])] {
            let configured = denied.clone().with_instance_search_limits(limits);
            let policy_before = configured.clone();
            let result = attempt(
                &source,
                &nodes,
                &symbols,
                snapshot,
                target,
                (profile.clone(), format),
                &configured,
            );
            if limits == Some([u64::MAX; 4]) {
                assert_eq!(
                    result.unwrap_err(),
                    "registration.proof.kernel_policy_rejected"
                );
            } else {
                assert_eq!(result.unwrap(), None);
            }
            assert_eq!(
                check(
                    &source,
                    &nodes,
                    &symbols,
                    snapshot,
                    target,
                    (profile.clone(), format),
                    &configured
                )
                .unwrap_err(),
                "registration.proof.kernel_policy_rejected"
            );
            for (s, n, e, id, encoding) in [
                (
                    &foreign_source,
                    &nodes,
                    &symbols,
                    target,
                    (profile.clone(), format),
                ),
                (
                    &source,
                    &foreign_nodes,
                    &symbols,
                    target,
                    (profile.clone(), format),
                ),
                (
                    &source,
                    &nodes,
                    &foreign_symbols,
                    target,
                    (profile.clone(), format),
                ),
                (
                    &bad_source,
                    &bad_nodes,
                    &bad_symbols,
                    target,
                    (profile.clone(), format),
                ),
                (
                    &source,
                    &nodes,
                    &symbols,
                    discharged,
                    (profile.clone(), format),
                ),
                (
                    &source,
                    &nodes,
                    &symbols,
                    VcId::new(vcs.vcs().len()),
                    (profile.clone(), format),
                ),
                (
                    &source,
                    &nodes,
                    &symbols,
                    target,
                    (unsupported.clone(), format),
                ),
                (
                    &source,
                    &nodes,
                    &symbols,
                    target,
                    (profile.clone(), ConcreteFormat::SmtLib),
                ),
            ] {
                let lower_error =
                    check(s, n, e, snapshot, id, encoding.clone(), &policy).unwrap_err();
                assert_ne!(lower_error, "registration.proof.kernel_policy_rejected");
                assert_eq!(
                    attempt(s, n, e, snapshot, id, encoding, &configured).unwrap_err(),
                    lower_error
                );
            }
            assert_eq!(configured, policy_before);
        }
    }
    assert_eq!(
        (
            source,
            nodes,
            symbols,
            foreign_source,
            foreign_nodes,
            foreign_symbols,
            bad_source,
            bad_nodes,
            bad_symbols
        ),
        before
    );
}
