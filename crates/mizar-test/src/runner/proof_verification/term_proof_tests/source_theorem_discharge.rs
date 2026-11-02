use super::*;

#[test]
fn source_theorem_discharge_handoff_matches_genuine_processed_target() {
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        checker::{
            FormulaEvidenceContext, ImportedFactContextLimits, KernelCheckPolicy,
            KernelCheckStatus, KernelContextIdentityEntry, KernelContextIdentityPayload,
            KernelContextIdentitySource, KernelEvidenceCheckInput, KernelEvidenceCheckKind,
            KernelEvidenceCheckLimits, KernelFormulaProducerRef, check_kernel_evidence,
        },
        clause::{Atom, SymbolKey, SymbolKind, Term, VariableId},
        formula_evidence::{Formula, FormulaEvidenceParseContext, parse_formula_evidence},
        rejection::{RejectionDetail, TargetVcFingerprint},
    };
    use mizar_vc::{
        discharge::{
            ComputationLimit, DischargeExplanationCategory, DischargeInput, DischargePolicy,
            try_discharge,
        },
        generator::generate_source_theorem_vcs,
        kernel_evidence_handoff::*,
        vc_ir::{
            GenerationSchemaVersion, PolicyKey, VcFormulaRef, VcProvenancePhase, VcSchemaVersion,
            VcStatus,
        },
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let generation = GenerationSchemaVersion::new("vc79-generation");
    let schema = VcSchemaVersion::new("vc79-schema");
    let alternate_generation = GenerationSchemaVersion::new("");
    let alternate_schema = VcSchemaVersion::new("λ\n\0");
    for (text, relocate, snapshot, label, max_steps) in [
        (text.to_owned(), false, 777, "limit", 64),
        (text.replace(" X", " Local"), false, 777, "limit", 64),
        (format!(":: 日本語 λ\n\n{text}"), false, 777, "limit", 64),
        (text.replace("T:", "Renamed:"), false, 777, "limit", 64),
        (text.to_owned(), true, 777, "limit", 64),
        (text.to_owned(), false, 778, "", 0),
        (text.to_owned(), false, 777, "λ\n\0", u32::MAX),
    ] {
        let (generation, schema) = if snapshot == 778 {
            (&alternate_generation, &alternate_schema)
        } else {
            (&generation, &schema)
        };
        let core = if relocate {
            let baseline = step5c14_computation_core(&text).unwrap();
            let module = baseline.module_id();
            let (frontend, source, collected) =
                super::super::super::tests::source_registration_owner_inputs(
                    &text,
                    module.package().as_str(),
                    module.path().as_str(),
                    1,
                );
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let ast = frontend.ast.as_ref().unwrap();
            assert_ne!(ast.source_id, baseline.source_id());
            assert_eq!(source.source_id(), ast.source_id);
            let core = with_source_theorem_check(ast, &source, collected.env(), |check| {
                mizar_core::elaborator::lower_source_theorem_skeletons(check, None)
            })
            .unwrap();
            assert_eq!(core.source_id(), ast.source_id);
            assert_eq!(core.module_id(), baseline.module_id());
            core
        } else {
            step5c14_computation_core(&text).unwrap()
        };
        let original =
            generate_source_theorem_vcs(&core, snapshot_id(snapshot), generation, schema).unwrap();
        let [vc] = original.vcs() else {
            panic!("one actual source VC")
        };
        let selected = vc.id;
        let policy = DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new(label),
                max_steps,
            },
        };
        let discharge = try_discharge(DischargeInput {
            vc_set: &original,
            policy: &policy,
        })
        .unwrap();
        let before = (
            core.clone(),
            original.clone(),
            discharge.clone(),
            policy.clone(),
        );
        let processed = discharge.vc_set();
        let [record] = processed.vcs() else {
            panic!("one processed VC")
        };
        assert_eq!(record.id, selected);
        assert_eq!(record.status, VcStatus::NeedsAtp);
        assert_eq!(
            record.provenance.last().unwrap().phase,
            VcProvenancePhase::Discharge
        );
        assert_eq!(&record.anchor.generation_schema_version, generation);
        assert_eq!(processed.schema_version(), schema);
        assert_eq!(processed.source(), core.source_id());
        assert_eq!(processed.module(), original.module());
        assert_eq!(processed.snapshot(), snapshot_id(snapshot));
        assert_eq!(discharge.explanations().len(), 1);
        assert_eq!(discharge.explanations()[0].vc, selected);
        assert_eq!(
            discharge.explanations()[0].category,
            DischargeExplanationCategory::UnsupportedRule
        );
        assert_eq!(discharge.explanations()[0].rule, None);
        assert!(discharge.evidence_records().is_empty());
        let (_, item) = core.items().iter().next().unwrap();
        let (_, proof) = core.proofs().iter().next().unwrap();
        let (_, seed) = core.obligation_seeds().iter().next().unwrap();
        let provenance = format!(
            "source-theorem-local-handoff-v1;source={:?};module={:?};item={:?};proof={:?};seed={:?};accounting={:?}",
            core.source_id(), core.module_id(), item, proof, seed, original.seed_accounting()
        ).into_bytes();
        let symbols = [
            KernelManifestEntry {
                entry_id: 0,
                payload: vec![3, 0, 0, 0, 0],
            },
            KernelManifestEntry {
                entry_id: 1,
                payload: vec![4, 0, 0, 0, 1],
            },
        ];
        let variables = [KernelManifestEntry {
            entry_id: 0,
            payload: vec![0, 0, 0, 0],
        }];
        let formulas = [
            (
                vc.local_context.entries()[0].formula.unwrap(),
                vec![1, 4, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0],
                Formula::Atom(Atom::new(
                    SymbolKey::new(SymbolKind::BuiltinRelation, 1),
                    vec![Term::Variable(VariableId(0))],
                )),
            ),
            (
                vc.goal,
                vec![
                    1, 3, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 2, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0,
                ],
                Formula::Atom(Atom::new(
                    SymbolKey::new(SymbolKind::Equality, 0),
                    vec![Term::Variable(VariableId(0)), Term::Variable(VariableId(0))],
                )),
            ),
        ]
        .into_iter()
        .map(
            |(formula_ref, formula_bytes, formula)| KernelFormulaPayload {
                formula_ref,
                projection: KernelFormulaProjection {
                    formula_fingerprint: KernelEvidenceFingerprint {
                        algorithm_id: 2,
                        digest: formula.canonical_hash_input().unwrap(),
                    },
                    formula_bytes,
                    provenance_payload: provenance.clone(),
                },
            },
        )
        .collect::<Vec<_>>();
        let expected = build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
            vc_set: processed,
            vc: selected,
            goal_polarity: KernelGoalPolarity::AssertFalseForRefutation,
            kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
            symbol_manifest: &symbols,
            variable_manifest: &variables,
            formula_payloads: &formulas,
            imported_formula_payloads: &[],
            substitutions: &[],
            formula_context: None,
            discharge_output: Some(&discharge),
        })
        .unwrap();
        let handoff = build_source_theorem_discharge_kernel_handoff(
            &core, &original, selected, &discharge, &policy,
        )
        .unwrap();
        assert_eq!(handoff, expected);
        assert_eq!(
            handoff,
            build_source_theorem_discharge_kernel_handoff(
                &core, &original, selected, &discharge, &policy
            )
            .unwrap()
        );
        let old = build_source_theorem_kernel_handoff(&core, &original, selected).unwrap();
        assert!(old.targets_vc(&original, selected).unwrap());
        assert!(!old.targets_vc(processed, selected).unwrap());
        assert!(handoff.targets_vc(processed, selected).unwrap());
        assert!(!handoff.targets_vc(&original, selected).unwrap());
        assert!(handoff.context_identity_is_consistent());
        assert_ne!(old.context_identity_hash(), handoff.context_identity_hash());
        for row in handoff.canonical_evidence().provenance() {
            assert_eq!(row.payload, provenance);
        }
        let same_output_policy = DischargePolicy {
            computation_limit: ComputationLimit {
                policy: PolicyKey::new("different-value-association"),
                max_steps: max_steps.wrapping_add(1),
            },
        };
        assert_eq!(
            discharge,
            try_discharge(DischargeInput {
                vc_set: &original,
                policy: &same_output_policy
            })
            .unwrap()
        );
        assert_eq!(
            handoff,
            build_source_theorem_discharge_kernel_handoff(
                &core,
                &original,
                selected,
                &discharge,
                &same_output_policy
            )
            .unwrap()
        );
        let fingerprint = handoff.canonical_evidence().target_vc();
        let target = TargetVcFingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone());
        let mut parse_context = FormulaEvidenceParseContext::v1(
            Fingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone()),
            KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
        );
        let parsed = parse_formula_evidence(
            &handoff.encode_local_evidence(None).unwrap(),
            &parse_context,
        )
        .unwrap();
        assert!(parsed.equality_instances().is_empty());
        let names = mizar_atp::backend::encode_local_variable_reflexivity_names(&parsed).unwrap();
        assert_eq!(names, [vec![0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0]]);
        parse_context.accepted_schema_versions.insert(2);
        let named = parse_formula_evidence(
            &handoff.encode_local_evidence(Some(&names)).unwrap(),
            &parse_context,
        )
        .unwrap();
        assert_eq!(named.equality_instances().len(), 1);
        let contexts = [&handoff, &old].map(|binding| {
            let entries = binding.context_identity().entries().iter().map(|entry| {
                let VcFormulaRef::Core(formula) = entry.producer_formula_ref() else { panic!() };
                let mizar_vc::kernel_evidence_handoff::KernelContextIdentitySource::LocalHypothesis { local_context_id } = entry.source() else { panic!() };
                KernelContextIdentityEntry::new(KernelContextIdentitySource::LocalHypothesis { local_context_id },
                    entry.formula_id(), Fingerprint::new(entry.formula_fingerprint().algorithm_id, entry.formula_fingerprint().digest.clone()),
                    KernelFormulaProducerRef::Core(formula))
            }).collect();
            let fingerprint = binding.canonical_evidence().target_vc();
            FormulaEvidenceContext::with_context_identity(
                Some(binding.context_identity_hash().as_bytes().to_vec()), vec![], vec![],
                Some(KernelContextIdentityPayload::new(
                    TargetVcFingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone()),
                    binding.canonical_hash(), binding.context_identity_hash(), entries)),
                ImportedFactContextLimits::default()).unwrap()
        });
        let old_fingerprint = old.canonical_evidence().target_vc();
        let old_target =
            TargetVcFingerprint::new(old_fingerprint.algorithm_id, old_fingerprint.digest.clone());
        for (evidence, target, context, accepted) in [
            (&named, &target, &contexts[0], true),
            (&parsed, &target, &contexts[0], false),
            (&named, &target, &contexts[1], false),
            (&named, &old_target, &contexts[0], false),
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
            assert!(!result.policy_taint());
            assert!(result.used_axioms().is_empty());
            if std::ptr::eq(evidence, &parsed) {
                assert_eq!(
                    result.rejections()[0].detail(),
                    RejectionDetail::InvalidSatRefutation
                );
            }
        }
        assert_eq!(vc.status, VcStatus::Open);
        assert!(core.proofs().iter().all(|(_, proof)| proof.status
            == mizar_core::core_ir::CoreProofStatus::PendingAutomaticProof));
        assert!(
            core.obligation_seeds()
                .iter()
                .all(|(_, seed)| seed.status == mizar_core::core_ir::ObligationSeedStatus::Active)
        );
        assert_eq!((core, original, discharge, policy), before);
    }
}

#[test]
fn source_theorem_discharge_handoff_preserves_refusals() {
    use mizar_vc::{
        discharge::{ComputationLimit, DischargeInput, DischargePolicy, try_discharge},
        generator::generate_source_computation_request,
        kernel_evidence_handoff::{
            build_source_theorem_discharge_kernel_handoff as configured,
            build_source_theorem_kernel_handoff as original_handoff,
        },
        vc_ir::*,
    };
    let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let core = step5c14_computation_core(text).unwrap();
    let original = generate_core_vcs(&core, snapshot_id(777)).unwrap();
    let selected = original.vcs()[0].id;
    let policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("explicit-refusals"),
            max_steps: 64,
        },
    };
    let discharge = try_discharge(DischargeInput {
        vc_set: &original,
        policy: &policy,
    })
    .unwrap();
    let foreign = step5c14_computation_core(&text.replace("T:", "Foreign:")).unwrap();
    let foreign_vcs = generate_core_vcs(&foreign, snapshot_id(777)).unwrap();
    let foreign_discharge = try_discharge(DischargeInput {
        vc_set: &foreign_vcs,
        policy: &policy,
    })
    .unwrap();
    let later = generate_core_vcs(&core, snapshot_id(778)).unwrap();
    let later_discharge = try_discharge(DischargeInput {
        vc_set: &later,
        policy: &policy,
    })
    .unwrap();
    let before = (
        core.clone(),
        original.clone(),
        discharge.clone(),
        policy.clone(),
        foreign.clone(),
        foreign_vcs.clone(),
    );
    let invalid = "theorem.discharge_handoff.invalid_projection";
    for wrong in [&foreign_discharge, &later_discharge] {
        assert_ne!(wrong, &discharge);
        assert_eq!(
            configured(&core, &original, selected, wrong, &policy).unwrap_err(),
            invalid
        );
    }
    for (actual_core, actual_vcs, id) in [
        (&foreign, &original, selected),
        (&core, &foreign_vcs, selected),
        (&core, &original, VcId::new(usize::MAX)),
        (&core, discharge.vc_set(), selected),
    ] {
        let expected = original_handoff(actual_core, actual_vcs, id).unwrap_err();
        assert_eq!(
            configured(actual_core, actual_vcs, id, &foreign_discharge, &policy).unwrap_err(),
            expected
        );
    }
    let cited = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    for source in [
        text.replacen("theorem", "lemma", 1),
        text.replace("set", "object"),
        format!("{text} {}", text.replace("T:", "U:")),
        cited.to_owned(),
    ] {
        let unsupported = step5c14_computation_core(&source).unwrap();
        let vcs = generate_core_vcs(&unsupported, snapshot_id(777)).unwrap();
        assert!(!vcs.vcs().is_empty(), "{source}");
        let actual = try_discharge(DischargeInput {
            vc_set: &vcs,
            policy: &policy,
        })
        .unwrap();
        for record in vcs.vcs() {
            let expected = original_handoff(&unsupported, &vcs, record.id).unwrap_err();
            for output in [&actual, &foreign_discharge] {
                assert_eq!(
                    configured(&unsupported, &vcs, record.id, output, &policy).unwrap_err(),
                    expected
                );
            }
        }
    }
    let computation =
        step5c14_computation_core("theorem C: 0 = 0 by computation(steps: 8);").unwrap();
    let computation_vcs = generate_source_computation_request(
        &computation,
        snapshot_id(777),
        &GenerationSchemaVersion::new("mizar-vc-generation-step5c14-computation-v1"),
        &VcSchemaVersion::new("mizar-vc-vcset-step5c14-computation-v1"),
    )
    .unwrap();
    assert_eq!(computation_vcs.vcs().len(), 1);
    let id = computation_vcs.vcs()[0].id;
    assert_eq!(
        configured(&computation, &computation_vcs, id, &discharge, &policy).unwrap_err(),
        original_handoff(&computation, &computation_vcs, id).unwrap_err()
    );
    for mutation in 0..3 {
        let mut parts = VcSetParts {
            schema_version: original.schema_version().clone(),
            snapshot: original.snapshot(),
            source: original.source(),
            module: original.module().clone(),
            generated_formulas: original.generated_formulas().to_vec(),
            vcs: original.vcs().to_vec(),
            seed_accounting: original.seed_accounting().to_vec(),
        };
        match mutation {
            0 => parts.vcs[0].status = VcStatus::NeedsAtp,
            1 => parts.vcs[0].provenance.clear(),
            2 => parts.module = VcModuleRef::new("foreign-module"),
            _ => unreachable!(),
        }
        let changed = VcSet::try_new(parts).unwrap();
        assert_ne!(changed, original);
        let actual = try_discharge(DischargeInput {
            vc_set: &changed,
            policy: &policy,
        })
        .unwrap();
        assert_ne!(actual, discharge);
        let expected = original_handoff(&core, &changed, selected).unwrap_err();
        assert_eq!(
            configured(&core, &changed, selected, &actual, &policy).unwrap_err(),
            expected
        );
        assert_eq!(
            configured(&core, &original, selected, &actual, &policy).unwrap_err(),
            invalid
        );
    }
    assert!(configured(&core, &original, selected, &discharge, &policy).is_ok());
    assert_eq!(
        (core, original, discharge, policy, foreign, foreign_vcs),
        before
    );
}
