use super::*;

#[test]
fn source_theorem_variable_reflexivity_names_reach_normal_kernel() {
    use mizar_atp::backend::{
        encode_closed_choice_reflexivity_names, encode_local_variable_reflexivity_names,
    };
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::{
            FormulaEvidenceContext, ImportedFactContextLimits, KernelCheckPolicy,
            KernelCheckStatus, KernelContextIdentityEntry, KernelContextIdentityPayload,
            KernelContextIdentitySource, KernelEvidenceCheckInput, KernelEvidenceCheckKind,
            KernelEvidenceCheckLimits, KernelFormulaProducerRef, check_kernel_evidence,
        },
        formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
        rejection::{RejectionDetail, TargetVcFingerprint},
    };
    use mizar_vc::{kernel_evidence_handoff::build_source_theorem_kernel_handoff, vc_ir::*};
    for source in [
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
        "theorem Renamed: for Bound being set holds Bound = Bound proof let Local be set; thus Local = Local; end;",
    ] {
        let core = step5c14_computation_core(source).unwrap();
        let vcs = generate_core_vcs(&core, snapshot_id(777)).unwrap();
        let original = (core.clone(), vcs.clone());
        let vc = &vcs.vcs()[0];
        assert_eq!(vc.status, VcStatus::Open);
        let handoff = build_source_theorem_kernel_handoff(&core, &vcs, vc.id).unwrap();
        let before = handoff.clone();
        let fingerprint = handoff.canonical_evidence().target_vc();
        let target = TargetVcFingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone());
        let mut parse_context = FormulaEvidenceParseContext::v1(
            Fingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone()),
            KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
        );
        let v1 = handoff.encode_local_evidence(None).unwrap();
        let parsed = parse_formula_evidence(&v1, &parse_context).unwrap();
        let original_parsed = parsed.clone();
        let names = encode_local_variable_reflexivity_names(&parsed).unwrap();
        assert_eq!(names, [vec![0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0]]);
        assert_eq!(
            names,
            encode_local_variable_reflexivity_names(&parsed).unwrap()
        );
        assert_eq!(parsed, original_parsed);
        assert!(
            encode_closed_choice_reflexivity_names(&parsed)
                .unwrap()
                .is_empty()
        );
        let v2 = handoff.encode_local_evidence(Some(&names)).unwrap();
        parse_context.accepted_schema_versions.insert(2);
        let named = parse_formula_evidence(&v2, &parse_context).unwrap();
        assert_eq!(named.equality_instances().len(), 1);
        let foreign_vcs = generate_core_vcs(&core, snapshot_id(778)).unwrap();
        let foreign = build_source_theorem_kernel_handoff(&core, &foreign_vcs, vc.id).unwrap();
        let contexts = [&handoff, &foreign].map(|binding| {
            let entries = binding.context_identity().entries().iter().map(|entry| {
                let VcFormulaRef::Core(formula) = entry.producer_formula_ref() else {
                    panic!()
                };
                let mizar_vc::kernel_evidence_handoff::KernelContextIdentitySource::LocalHypothesis { local_context_id } = entry.source() else {
                    panic!()
                };
                assert_eq!(local_context_id, 1);
                KernelContextIdentityEntry::new(
                    KernelContextIdentitySource::LocalHypothesis { local_context_id },
                    entry.formula_id(),
                    Fingerprint::new(entry.formula_fingerprint().algorithm_id, entry.formula_fingerprint().digest.clone()),
                    KernelFormulaProducerRef::Core(formula),
                )
            }).collect();
            let fingerprint = binding.canonical_evidence().target_vc();
            FormulaEvidenceContext::with_context_identity(
                Some(binding.context_identity_hash().as_bytes().to_vec()),
                vec![], vec![],
                Some(KernelContextIdentityPayload::new(
                    TargetVcFingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone()),
                    binding.canonical_hash(), binding.context_identity_hash(), entries,
                )),
                ImportedFactContextLimits::default(),
            ).unwrap()
        });
        let foreign_fingerprint = foreign.canonical_evidence().target_vc();
        let foreign_target = TargetVcFingerprint::new(
            foreign_fingerprint.algorithm_id,
            foreign_fingerprint.digest.clone(),
        );
        assert_ne!(target, foreign_target);
        for (evidence, target, context, accepted) in [
            (&named, &target, &contexts[0], true),
            (&parsed, &target, &contexts[0], false),
            (&named, &target, &contexts[1], false),
            (&named, &foreign_target, &contexts[0], false),
        ] {
            let result = check_kernel_evidence(KernelEvidenceCheckInput {
                target_vc_fingerprint: target,
                evidence,
                formula_context: Some(context),
                check_kind: KernelEvidenceCheckKind::ProofObligation,
                policy: KernelCheckPolicy::default(),
                limits: KernelEvidenceCheckLimits::default(),
            });
            assert_eq!(
                result.status(),
                if accepted {
                    KernelCheckStatus::Accepted
                } else {
                    KernelCheckStatus::Rejected
                }
            );
            assert_eq!(
                result.evidence_check_kind(),
                Some(KernelEvidenceCheckKind::ProofObligation)
            );
            assert!(!result.policy_taint());
            assert!(result.used_axioms().is_empty());
            if std::ptr::eq(evidence, &parsed) {
                assert_eq!(
                    result.rejections()[0].detail(),
                    RejectionDetail::InvalidSatRefutation
                );
            }
        }
        assert_eq!(handoff, before);
        assert_eq!(handoff.encode_local_evidence(None).unwrap(), v1);
        assert_eq!((core, vcs), original);
    }
}

#[test]
fn source_theorem_variable_reflexivity_untrusted_parsed_profiles() {
    use mizar_atp::backend::encode_local_variable_reflexivity_names;
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        clause::{Atom, SymbolKey, SymbolKind, Term, VariableId},
        formula_evidence::{
            Formula, FormulaEvidenceParseContext, GoalPolarity, parse_formula_evidence,
        },
    };
    use mizar_vc::kernel_evidence_handoff::*;
    let core = step5c14_computation_core(
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
    )
    .unwrap();
    let vcs = generate_core_vcs(&core, snapshot_id(777)).unwrap();
    let vc = &vcs.vcs()[0];
    let equality = SymbolKey::new(SymbolKind::Equality, 7);
    let variable = Term::Variable(VariableId(11));
    let goal = Formula::Atom(Atom::new(
        equality,
        vec![variable.clone(), variable.clone()],
    ));
    let goal_wire = vec![
        1, 3, 0, 0, 0, 7, 0, 0, 0, 2, 0, 0, 0, 2, 1, 0, 0, 0, 11, 1, 0, 0, 0, 11,
    ];
    // These raw projections exercise only the untrusted encoder, never source acceptance.
    for case in 0..9 {
        let mut formula = goal.clone();
        let mut wire = goal_wire.clone();
        match case {
            1 => {
                formula = Formula::Atom(Atom::new(
                    SymbolKey::new(SymbolKind::Predicate, 8),
                    vec![variable.clone(), variable.clone()],
                ));
                wire[1] = 1;
                wire[5] = 8;
            }
            2 => {
                formula = Formula::Not(Box::new(formula));
                wire.insert(0, 2);
            }
            3 => {
                formula = Formula::Atom(Atom::new(
                    equality,
                    vec![variable.clone(), Term::Variable(VariableId(12))],
                ));
                wire[23] = 12;
            }
            4 => {
                let ground = Term::Application {
                    symbol: SymbolKey::new(SymbolKind::FunctorPredicate, 9),
                    arguments: vec![],
                };
                formula = Formula::Atom(Atom::new(equality, vec![ground.clone(), ground]));
                wire.truncate(14);
                wire.extend([2, 2, 0, 0, 0, 9, 0, 0, 0, 0, 2, 2, 0, 0, 0, 9, 0, 0, 0, 0]);
            }
            8 => {
                formula = Formula::Atom(Atom::new(equality, vec![variable.clone(); 3]));
                wire[9] = 3;
                wire[13] = 3;
                wire.extend([1, 0, 0, 0, 11]);
            }
            _ => {}
        }
        let formulas =
            [vc.local_context.entries()[0].formula.unwrap(), vc.goal].map(|formula_ref| {
                KernelFormulaPayload {
                    formula_ref,
                    projection: KernelFormulaProjection {
                        formula_fingerprint: KernelEvidenceFingerprint {
                            algorithm_id: 2,
                            digest: formula.canonical_hash_input().unwrap(),
                        },
                        formula_bytes: wire.clone(),
                        provenance_payload: b"untrusted-encoder-input".to_vec(),
                    },
                }
            });
        let substitutions = if case == 5 {
            vec![KernelSubstitutionPayload {
                substitution_id: 1,
                source_formula: formulas[0].formula_ref,
                binder_context_encoding: vec![
                    0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 11, 0, 0, 0, 0,
                ],
                payload: vec![
                    0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 11, 1, 0, 0, 0, 11, 1,
                ],
                freshness_witnesses: vec![],
                free_variable_constraints: vec![],
                provenance_payload: b"untrusted-substitution".to_vec(),
            }]
        } else {
            vec![]
        };
        let symbols = [(8, 1), (9, 2), (7, 3)].map(|(id, kind)| KernelManifestEntry {
            entry_id: kind as u32,
            payload: vec![kind, 0, 0, 0, id],
        });
        let variables = [11, 12].map(|id| KernelManifestEntry {
            entry_id: id,
            payload: id.to_be_bytes().to_vec(),
        });
        let handoff = build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
            vc_set: &vcs,
            vc: vc.id,
            goal_polarity: KernelGoalPolarity::AssertFalseForRefutation,
            kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
            symbol_manifest: &symbols,
            variable_manifest: &variables,
            formula_payloads: &formulas,
            imported_formula_payloads: &[],
            substitutions: &substitutions,
            formula_context: None,
            discharge_output: None,
        })
        .unwrap();
        let mut bytes = handoff
            .encode_local_evidence(if case == 6 { Some(&[]) } else { None })
            .unwrap();
        if case == 7 {
            // The ordinary writer's final-goal frame is last; change only its polarity byte.
            let goal_size =
                1 + 1 + 4 + formula.canonical_hash_input().unwrap().len() + 4 + wire.len();
            let polarity_offset = bytes.len() - goal_size;
            assert_eq!(bytes[polarity_offset], 1);
            bytes[polarity_offset] = 2;
        }
        let target = handoff.canonical_evidence().target_vc();
        let mut context = FormulaEvidenceParseContext::v1(
            Fingerprint::new(target.algorithm_id, target.digest.clone()),
            KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
        );
        context.accepted_schema_versions.insert(2);
        let parsed = parse_formula_evidence(&bytes, &context).unwrap();
        if case == 8 {
            let Formula::Atom(atom) = &parsed.final_goal().formula else {
                panic!()
            };
            assert_eq!(atom.arguments.len(), 3);
            assert_eq!(atom.arguments, vec![variable.clone(); 3]);
        }
        if case == 0 {
            assert_eq!(
                encode_local_variable_reflexivity_names(&parsed).unwrap(),
                [vec![0, 0, 0, 1, 1, 0, 0, 0, 7, 1, 0, 0, 0, 11]]
            );
        } else {
            assert_eq!(
                encode_local_variable_reflexivity_names(&parsed).unwrap_err(),
                "unsupported local-variable reflexivity input",
                "case {case}"
            );
        }
        if case == 5 {
            assert_eq!(parsed.substitutions().len(), 1);
        }
        if case == 6 {
            assert_eq!(parsed.schema_version(), 2);
        }
        if case == 7 {
            assert_eq!(
                parsed.final_goal().polarity,
                GoalPolarity::AssertTrueForConsistency
            );
        }
    }
}
