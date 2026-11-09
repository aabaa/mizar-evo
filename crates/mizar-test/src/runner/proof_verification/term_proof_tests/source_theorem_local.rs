use super::*;

#[test]
fn source_theorem_local_handoff_exact_local_projection() {
    use mizar_core::core_ir::{CoreFormulaKind, CoreProofNodeKind, CoreTermKind};
    use mizar_kernel::{
        clause::{Atom, SymbolKey, SymbolKind, Term, VariableId},
        formula_evidence::Formula,
    };
    use mizar_vc::{kernel_evidence_handoff::*, vc_ir::VcFormulaRef};
    let guard_wire = [1, 4, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0];
    let goal_wire = [
        1, 3, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 2, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0,
    ];
    let guard_fingerprint = Formula::Atom(Atom::new(
        SymbolKey::new(SymbolKind::BuiltinRelation, 1),
        vec![Term::Variable(VariableId(0))],
    ))
    .canonical_hash_input()
    .unwrap();
    let goal_fingerprint = Formula::Atom(Atom::new(
        SymbolKey::new(SymbolKind::Equality, 0),
        vec![Term::Variable(VariableId(0)), Term::Variable(VariableId(0))],
    ))
    .canonical_hash_input()
    .unwrap();
    for source in [
        "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
        "theorem Renamed: for Bound being set holds Bound = Bound proof let Local be set; thus Local = Local; end;",
    ] {
        let core = step5c14_computation_core(source).unwrap();
        let vcs = generate_core_vcs(&core, snapshot_id(777)).unwrap();
        let vc = &vcs.vcs()[0];
        let proof = core.proofs().iter().next().unwrap().1;
        let CoreFormulaKind::Forall { binders, .. } =
            &core.formulas().get(proof.proposition).unwrap().kind
        else {
            panic!()
        };
        let CoreProofNodeKind::IntroduceBinder { binder, .. } =
            &core.proof_nodes().get(proof.root).unwrap().kind
        else {
            panic!()
        };
        assert_ne!(binders[0].var, binder.var);
        assert_eq!(binder.role.as_str(), "proof-let");
        let local_guard = binder.ty_guard.unwrap();
        let CoreFormulaKind::TypePred { subject, ty } =
            &core.formulas().get(local_guard).unwrap().kind
        else {
            panic!()
        };
        assert_eq!(ty.as_str(), "set");
        assert_eq!(
            core.terms().get(*subject).unwrap().kind,
            CoreTermKind::Var(binder.var)
        );
        let VcFormulaRef::Core(goal) = vc.goal else {
            panic!()
        };
        let CoreFormulaKind::Equals { left, right } = core.formulas().get(goal).unwrap().kind
        else {
            panic!()
        };
        for term in [left, right] {
            assert_eq!(
                core.terms().get(term).unwrap().kind,
                CoreTermKind::Var(binder.var)
            );
        }
        let handoff = build_source_theorem_kernel_handoff(&core, &vcs, vc.id).unwrap();
        let envelope = handoff.canonical_evidence();
        assert_eq!(
            envelope.symbol_manifest(),
            [
                KernelManifestEntry {
                    entry_id: 0,
                    payload: vec![3, 0, 0, 0, 0]
                },
                KernelManifestEntry {
                    entry_id: 1,
                    payload: vec![4, 0, 0, 0, 1]
                },
            ]
        );
        assert_eq!(
            envelope.variable_manifest(),
            [KernelManifestEntry {
                entry_id: 0,
                payload: vec![0, 0, 0, 0]
            }]
        );
        assert_eq!(envelope.formula_evidence().len(), 1);
        let guard = &envelope.formula_evidence()[0];
        assert_eq!(
            guard.source(),
            &KernelFormulaSource::LocalHypothesis {
                local_context_id: 1
            }
        );
        assert_eq!(guard.formula_bytes(), guard_wire);
        assert_eq!(
            guard.formula_fingerprint(),
            &KernelEvidenceFingerprint {
                algorithm_id: 2,
                digest: guard_fingerprint.clone()
            }
        );
        assert_eq!(
            guard.producer_formula_ref(),
            Some(VcFormulaRef::Core(local_guard))
        );
        assert_eq!(handoff.context_identity().entries().len(), 1);
        let context = &handoff.context_identity().entries()[0];
        assert_eq!(
            context.source(),
            KernelContextIdentitySource::LocalHypothesis {
                local_context_id: 1
            }
        );
        assert_eq!(
            context.producer_formula_ref(),
            VcFormulaRef::Core(local_guard)
        );
        assert_eq!(context.formula_fingerprint(), guard.formula_fingerprint());
        assert_eq!(envelope.final_goal().formula_bytes, goal_wire);
        assert_eq!(
            envelope.final_goal().formula_fingerprint,
            KernelEvidenceFingerprint {
                algorithm_id: 2,
                digest: goal_fingerprint.clone()
            }
        );
        assert_eq!(
            envelope.final_goal().polarity,
            KernelGoalPolarity::AssertFalseForRefutation
        );
        assert!(envelope.substitutions().is_empty());
        assert!(handoff.formula_context_requirements().is_none());
        assert!(handoff.context_identity_is_consistent());
        assert!(handoff.targets_vc(&vcs, vc.id).unwrap());
    }
}

#[test]
fn source_theorem_local_handoff_generic_equality_and_kernel_nonacceptance() {
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
    use mizar_vc::kernel_evidence_handoff::KernelContextIdentitySource as SourceContext;
    use mizar_vc::{kernel_evidence_handoff::*, vc_ir::VcFormulaRef};
    let source = "theorem T: for X being set holds X = X proof let Y be set; thus Y = Y; end;";
    let core = step5c14_computation_core(source).unwrap();
    let vcs = generate_core_vcs(&core, snapshot_id(777)).unwrap();
    let original = (core.clone(), vcs.clone());
    let vc = &vcs.vcs()[0];
    let (_, item) = core.items().iter().next().unwrap();
    let (_, proof) = core.proofs().iter().next().unwrap();
    let (_, seed) = core.obligation_seeds().iter().next().unwrap();
    let provenance = format!(
        "source-theorem-local-handoff-v1;source={:?};module={:?};item={:?};proof={:?};seed={:?};accounting={:?}",
        core.source_id(), core.module_id(), item, proof, seed, vcs.seed_accounting()
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
        vc_set: &vcs,
        vc: vc.id,
        goal_polarity: KernelGoalPolarity::AssertFalseForRefutation,
        kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
        symbol_manifest: &symbols,
        variable_manifest: &variables,
        formula_payloads: &formulas,
        imported_formula_payloads: &[],
        substitutions: &[],
        formula_context: None,
        discharge_output: None,
    })
    .unwrap();
    let handoff = build_source_theorem_kernel_handoff(&core, &vcs, vc.id).unwrap();
    assert_eq!(handoff, expected);
    assert_eq!(
        handoff,
        build_source_theorem_kernel_handoff(&core, &vcs, vc.id).unwrap()
    );
    for row in handoff.canonical_evidence().provenance() {
        assert_eq!(row.payload, provenance);
    }
    let fresh = generate_core_vcs(&core, snapshot_id(778)).unwrap();
    let fresh_handoff = build_source_theorem_kernel_handoff(&core, &fresh, vc.id).unwrap();
    assert_ne!(handoff.canonical_hash(), fresh_handoff.canonical_hash());
    assert!(!handoff.targets_vc(&fresh, vc.id).unwrap());
    let bytes = handoff.encode_local_evidence(None).unwrap();
    assert_eq!(&bytes[..26], b"MIZAR_KERNEL_EVIDENCE\0\0\x01\0\x01");
    let fingerprint = handoff.canonical_evidence().target_vc();
    let parsed = parse_formula_evidence(
        &bytes,
        &FormulaEvidenceParseContext::v1(
            Fingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone()),
            KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
        ),
    )
    .unwrap();
    assert!(parsed.equality_instances().is_empty());
    let target = TargetVcFingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone());
    let entries = handoff
        .context_identity()
        .entries()
        .iter()
        .map(|entry| {
            let VcFormulaRef::Core(formula) = entry.producer_formula_ref() else {
                panic!()
            };
            let SourceContext::LocalHypothesis { local_context_id } = entry.source() else {
                panic!()
            };
            KernelContextIdentityEntry::new(
                KernelContextIdentitySource::LocalHypothesis { local_context_id },
                entry.formula_id(),
                Fingerprint::new(
                    entry.formula_fingerprint().algorithm_id,
                    entry.formula_fingerprint().digest.clone(),
                ),
                KernelFormulaProducerRef::Core(formula),
            )
        })
        .collect();
    let context = FormulaEvidenceContext::with_context_identity(
        Some(handoff.context_identity_hash().as_bytes().to_vec()),
        vec![],
        vec![],
        Some(KernelContextIdentityPayload::new(
            target.clone(),
            handoff.canonical_hash(),
            handoff.context_identity_hash(),
            entries,
        )),
        ImportedFactContextLimits::default(),
    )
    .unwrap();
    let result = check_kernel_evidence(KernelEvidenceCheckInput {
        target_vc_fingerprint: &target,
        evidence: &parsed,
        formula_context: Some(&context),
        check_kind: KernelEvidenceCheckKind::ProofObligation,
        policy: KernelCheckPolicy::default(),
        limits: KernelEvidenceCheckLimits::default(),
    });
    assert_eq!(result.status(), KernelCheckStatus::Rejected);
    assert_eq!(result.rejections().len(), 1);
    let rejection = &result.rejections()[0];
    assert_eq!(rejection.detail(), RejectionDetail::InvalidSatRefutation);
    assert_eq!(
        rejection.location().field_path,
        Some("sat_checker.satisfiable")
    );
    assert!(!result.policy_taint());
    assert_eq!((core, vcs), original);
}

#[test]
fn source_theorem_local_handoff_refuses_foreign_and_unsupported_graphs() {
    use mizar_core::core_ir::CoreItemKind;
    use mizar_vc::{
        discharge::{DischargeInput, DischargePolicy, try_discharge},
        kernel_evidence_handoff::build_source_theorem_kernel_handoff,
        vc_ir::*,
    };
    let source = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let core = step5c14_computation_core(source).unwrap();
    let vcs = generate_core_vcs(&core, snapshot_id(777)).unwrap();
    let selected = vcs.vcs()[0].id;
    let invalid = "theorem.handoff.invalid_projection";
    assert_eq!(
        build_source_theorem_kernel_handoff(&core, &vcs, VcId::new(1)).unwrap_err(),
        invalid
    );
    let foreign = step5c14_computation_core(&source.replace("T:", "Different:")).unwrap();
    assert!(build_source_theorem_kernel_handoff(&foreign, &vcs, selected).is_err());
    let lemma = step5c14_computation_core(&source.replacen("theorem", "lemma", 1)).unwrap();
    assert_eq!(lemma.items().len(), 1);
    assert_eq!(
        lemma.items().iter().next().unwrap().1.kind,
        CoreItemKind::Lemma
    );
    let lemma_vcs = generate_core_vcs(&lemma, snapshot_id(777)).unwrap();
    assert_eq!(lemma_vcs.vcs().len(), 1);
    assert_eq!(
        build_source_theorem_kernel_handoff(&lemma, &lemma_vcs, lemma_vcs.vcs()[0].id).unwrap_err(),
        invalid
    );
    for unsupported in [
        source.replace("set", "object"),
        format!("{source} {}", source.replace("T:", "U:")),
    ] {
        let actual = step5c14_computation_core(&unsupported).unwrap();
        let actual_vcs = generate_core_vcs(&actual, snapshot_id(777)).unwrap();
        assert!(!actual_vcs.vcs().is_empty(), "{unsupported}");
        for vc in actual_vcs.vcs() {
            assert_eq!(
                build_source_theorem_kernel_handoff(&actual, &actual_vcs, vc.id).unwrap_err(),
                invalid,
                "{unsupported}"
            );
        }
    }
    let computation =
        step5c14_computation_core("theorem C: 0 = 0 by computation(steps: 8);").unwrap();
    let computation_vcs = mizar_vc::generator::generate_source_computation_request(
        &computation,
        snapshot_id(777),
        &GenerationSchemaVersion::new("mizar-vc-generation-step5c14-computation-v1"),
        &VcSchemaVersion::new("mizar-vc-vcset-step5c14-computation-v1"),
    )
    .unwrap();
    assert_eq!(computation_vcs.vcs().len(), 1);
    assert_eq!(computation_vcs.vcs()[0].status, VcStatus::Open);
    assert_eq!(
        build_source_theorem_kernel_handoff(
            &computation,
            &computation_vcs,
            computation_vcs.vcs()[0].id
        )
        .unwrap_err(),
        invalid
    );
    let cited = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let cited_core = step5c14_computation_core(cited).unwrap();
    let cited_vcs = generate_core_vcs(&cited_core, snapshot_id(777)).unwrap();
    assert_eq!(cited_vcs.vcs().len(), 2);
    assert_eq!(
        check_theorem_source(cited, PipelinePhase::VcGeneration)
            .unwrap()
            .1
            .unwrap(),
        generate_core_vcs(&cited_core, snapshot_id(777)).unwrap()
    );
    assert!(cited_vcs.vcs()[1].local_context.entries().len() > 1);
    for vc in cited_vcs.vcs() {
        assert_eq!(
            build_source_theorem_kernel_handoff(&cited_core, &cited_vcs, vc.id).unwrap_err(),
            invalid
        );
    }
    let discharge = try_discharge(DischargeInput {
        vc_set: &vcs,
        policy: &DischargePolicy::default(),
    })
    .unwrap();
    assert_ne!(discharge.vc_set(), &vcs);
    assert_eq!(
        build_source_theorem_kernel_handoff(&core, discharge.vc_set(), selected).unwrap_err(),
        invalid
    );
    for mutation in 0..5 {
        let mut parts = VcSetParts {
            schema_version: vcs.schema_version().clone(),
            snapshot: vcs.snapshot(),
            source: vcs.source(),
            module: vcs.module().clone(),
            generated_formulas: vcs.generated_formulas().to_vec(),
            vcs: vcs.vcs().to_vec(),
            seed_accounting: vcs.seed_accounting().to_vec(),
        };
        match mutation {
            0 => parts.vcs[0].status = VcStatus::NeedsAtp,
            1 => parts.vcs[0].provenance.clear(),
            2 => {
                let primary = parts.vcs[0].source.primary.clone();
                parts.vcs[0].source.related.push(primary);
            }
            3 => parts.vcs[0].premises.clear(),
            4 => parts.module = VcModuleRef::new("foreign-module"),
            _ => unreachable!(),
        }
        let changed = VcSet::try_new(parts).unwrap();
        assert_ne!(
            changed, vcs,
            "mutation {mutation} must change the whole graph"
        );
        assert_eq!(
            build_source_theorem_kernel_handoff(&core, &changed, selected).unwrap_err(),
            invalid,
            "mutation {mutation}"
        );
    }
    assert!(build_source_theorem_kernel_handoff(&core, &vcs, selected).is_ok());
}

#[test]
fn source_theorem_local_type_contexts_retain_real_sites_and_consumers() -> Result<(), String> {
    use mizar_checker::{
        binding_env::{BinderIdentity, BindingContextLayer, BindingContextOwner},
        type_checker::{
            SourceVariableSemanticsChecker as Semantics, TermFormulaChecker as Checker,
        },
        typed_ast::{BindingTypeRef, ContextRecoveryState, TypeContextLayer, TypedSiteRef},
    };
    use mizar_core::{
        core_ir::{CoreProofStatus, ObligationSeedStatus},
        elaborator::lower_source_theorem_skeletons,
    };
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::NamespacePath,
        labels::{LabelResolver, ProofLabelSourceCollector},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::{
            SignatureProjectionExtractor, source_proof_scope_maps,
            validate_source_proof_occurrences,
        },
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceInput,
        SourceOrigin, SourceOriginInput, hash_text,
    };
    use mizar_syntax::SurfaceNodeKind as K;
    use std::collections::{BTreeMap, BTreeSet};
    let fixture = include_str!(
        "../../../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
    );
    let sources = [
        fixture.to_owned(),
        fixture.replace("Lem1", "Seed").replace("let X be set;", "let Local be set;").replace("thus X = X", "thus Local = Local"),
        "theorem Pair: for A, Spare, B being set holds A = B proof let L, Unused, R be set; thus L = R; end;".into(),
        "theorem Nested: for A being set for Unused being set holds for B being set holds A = B proof let L be set; let Idle be set; let R be set; thus L = R; end;".into(),
        "theorem Groups: for A being set, Unused, B being set holds A = B proof let L be set, Idle, R be set; thus L = R; end;".into(),
        "theorem Shadow: for A being set holds for A being set holds A = A proof let A be set; thus A = A; end;".into(),
        "theorem Unused: for A being set holds A = A proof end;".into(),
    ];
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/local_type_contexts.miz");
    let ids = InMemorySessionIdAllocator::new();
    for (case, text) in sources.iter().enumerate() {
        let mut disk_capture = None;
        for buffer in [false, true] {
            fs::write(&path, if buffer { "stale disk content" } else { text }).unwrap();
            let frontend = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
                super::super::super::ParseOnlyImportProvider,
                MizarParserSeam,
            )
            .run(
                SourceUnitRequest {
                    snapshot: snapshot_id(777),
                    input: SourceInput {
                        package_id: PackageId::new("local-type-context-test"),
                        module_path: ModulePath::new("owners"),
                        normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                        edition: Edition::new("2026"),
                        origin: if buffer {
                            SourceOriginInput::OpenBuffer {
                                uri: format!("file://{}", path.display()),
                                expected_version: 3,
                                actual_version: 3,
                                text: text.clone().into(),
                            }
                        } else {
                            SourceOriginInput::Disk { path: path.clone() }
                        },
                    },
                },
                &ids,
            )
            .unwrap();
            assert!(
                frontend.diagnostics.is_empty(),
                "{:?}",
                frontend.diagnostics
            );
            assert_eq!(frontend.source.source_text.as_ref(), text);
            assert_eq!(frontend.source.source_hash, hash_text(text));
            assert_eq!(
                matches!(
                    frontend.source.origin,
                    SourceOrigin::OpenBuffer { version: 3 }
                ),
                buffer
            );
            let ast = frontend.ast.as_ref().unwrap();
            let module = ModuleId::new(
                frontend.source.package_id.clone(),
                frontend.source.module_path.clone(),
            );
            let source = SurfaceResolvedArena::lower(ast, &module).unwrap();
            let shells = DeclarationShellCollector::new(ast, &module).collect();
            let collected = SignatureProjectionExtractor::new(
                ast,
                &shells,
                NamespacePath::new(module.path().as_str()),
            )
            .collect(&module);
            assert!(collected.diagnostics().is_empty());
            let symbols = collected.env();
            let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                SourceVariableScopeInput::new(ast, &module, symbols),
            )
            .unwrap();
            let before = (
                source.clone(),
                symbols.clone(),
                scope.clone(),
                ast.snapshot_text(),
                frontend.source.clone(),
            );
            let baseline = Checker::check_source_theorem_equalities_with_expression_metadata(
                &source, symbols, &scope,
            )?;
            validate_source_proof_occurrences(&source, symbols, &scope)?;
            let maps = source_proof_scope_maps(&source, symbols, &scope)?;
            let lexical = Semantics::source_theorem_binding_env(&source, symbols, &scope)?;
            let output = Checker::check_source_theorem_equalities_with_local_contexts(
                &source, symbols, &scope,
            )?;
            let (bindings, inference, typed, expressions) = &output;
            let base = &baseline.2;
            assert_eq!(bindings, &baseline.0);
            assert_eq!(inference, &baseline.1);
            assert_eq!(typed.source_id(), source.source_id());
            assert_eq!(typed.module_id(), base.module_id());
            assert_eq!(typed.resolved_root(), base.resolved_root());
            assert_eq!(typed.nodes().root(), base.nodes().root());
            assert_eq!(typed.nodes().len(), base.nodes().len());
            assert_eq!(typed.types(), base.types());
            assert_eq!(typed.facts(), base.facts());
            assert_eq!(typed.coercions(), base.coercions());
            assert_eq!(typed.initial_obligations(), base.initial_obligations());
            assert_eq!(typed.diagnostics(), base.diagnostics());
            assert_eq!(typed.source_context(), base.source_context());
            assert_eq!(typed.source_type(), base.source_type());
            assert_eq!(typed.source_attribute(), base.source_attribute());
            assert_eq!(typed.source_term(), base.source_term());
            assert_eq!(typed.source_atomic_formula(), base.source_atomic_formula());
            assert!(typed.source_context().is_none());
            assert!(base.contexts().is_empty());
            let typed_for = |surface| {
                let resolved = source.resolved_node_for(surface).unwrap();
                let mut found = base
                    .nodes()
                    .iter()
                    .filter(|(_, node)| node.resolved_node == Some(resolved));
                let id = found.next().unwrap().0;
                assert!(found.next().is_none());
                id
            };
            let root_node = ast.root().unwrap();
            assert_eq!(source.resolved_node_for(root_node), base.resolved_root());
            assert_eq!(typed_for(root_node), typed.nodes().root().unwrap());
            let root_scope = &maps.0[&root_node];
            let mut boundaries = vec![(
                root_node,
                root_scope.clone(),
                None,
                TypeContextLayer::Module,
            )];
            for view in ast.node_views() {
                let Some(parent) = maps.1.get(&view.id()) else {
                    continue;
                };
                if maps.0[&view.id()] == maps.0[parent] {
                    continue;
                }
                let layer = match view.kind() {
                    K::QuantifiedFormula(_) => TypeContextLayer::Expression,
                    K::ProofBlock => TypeContextLayer::Proof,
                    _ => panic!("unexpected admitted lexical boundary"),
                };
                boundaries.push((
                    view.id(),
                    maps.0[&view.id()].clone(),
                    Some(maps.0[parent].clone()),
                    layer,
                ));
            }
            boundaries.sort_by(|left, right| {
                (left.1.path().len(), left.1.path()).cmp(&(right.1.path().len(), right.1.path()))
            });
            assert_eq!(typed.contexts().len(), boundaries.len());
            assert_eq!(typed.contexts().len(), lexical.contexts().len());
            let mut local_ids = BTreeMap::new();
            let mut binding_ids = BTreeMap::new();
            let mut owners = BTreeSet::new();
            for ((node, local, parent, layer), (id, row)) in
                boundaries.iter().zip(typed.contexts().iter())
            {
                let owner = TypedSiteRef::Node(typed_for(*node));
                assert_eq!(row.id, id);
                assert_eq!(row.owner, owner);
                assert!(owners.insert(owner));
                assert_eq!(&row.layer, layer);
                assert_eq!(row.parent, parent.as_ref().map(|outer| local_ids[outer]));
                assert_eq!(row.recovery, ContextRecoveryState::Normal);
                assert!(row.introduced_assumptions.is_empty());
                assert!(row.visible_facts.is_empty());
                assert!(local_ids.insert(local.clone(), id).is_none());
                let mut contexts = lexical.contexts().iter().filter(|(_, context)| {
                    context.lexical_scope.as_ref().unwrap_or(root_scope) == local
                });
                let (binding_id, context) = contexts.next().unwrap();
                assert!(contexts.next().is_none());
                assert!(binding_ids.insert(binding_id, id).is_none());
                let (expected_owner, expected_layer) = if *node == root_node {
                    (BindingContextOwner::Module, BindingContextLayer::Module)
                } else {
                    let range = ast.node_view(*node).unwrap().range();
                    match layer {
                        TypeContextLayer::Expression => (
                            BindingContextOwner::SourceFormula {
                                source_range: range,
                            },
                            BindingContextLayer::Expression,
                        ),
                        TypeContextLayer::Proof => (
                            BindingContextOwner::SourceStatement {
                                source_range: range,
                            },
                            BindingContextLayer::Proof,
                        ),
                        _ => unreachable!(),
                    }
                };
                assert_eq!(context.owner, expected_owner);
                assert_eq!(context.layer, expected_layer);
                assert_eq!(
                    row.parent,
                    context.parent.map(|parent| binding_ids[&parent])
                );
                let declarations = scope
                    .bindings()
                    .iter()
                    .filter(|binding| binding.scope() == local)
                    .collect::<Vec<_>>();
                assert_eq!(context.bindings.len(), declarations.len());
                let mut own = Vec::new();
                for (declaration, binding) in declarations.iter().zip(&context.bindings) {
                    let binding = lexical.bindings().get(*binding).unwrap();
                    assert_eq!(binding.owner_context, binding_id);
                    assert_eq!(
                        binding.identity,
                        BinderIdentity::ResolverLocal {
                            scope: declaration.scope().clone(),
                            ordinal: declaration.ordinal(),
                            declaration_range: declaration.range(),
                        }
                    );
                    assert_eq!(maps.0.get(&declaration.node()), Some(declaration.scope()));
                    own.push(BindingTypeRef::Site(TypedSiteRef::Node(typed_for(
                        declaration.node(),
                    ))));
                }
                assert_eq!(row.bindings, own);
            }
            assert_eq!(local_ids.len(), typed.contexts().len());
            assert_eq!(binding_ids.len(), lexical.contexts().len());
            let mut covered = BTreeSet::new();
            for (surface, local) in &maps.0 {
                let id = typed_for(*surface);
                assert!(covered.insert(id));
                let old = base.nodes().node(id).unwrap();
                let node = typed.nodes().node(id).unwrap();
                assert_eq!(node.links.context, Some(local_ids[local]));
                let mut expected = old.clone();
                assert!(expected.links.context.is_none());
                expected.links.context = node.links.context;
                assert_eq!(node, &expected);
                let actual = source.arena().node(node.resolved_node.unwrap()).unwrap();
                assert_eq!(actual.origin().anchor(), &node.anchor);
                assert_eq!(actual.origin().source_id(), typed.source_id());
                assert_eq!(actual.origin().module_id(), typed.module_id());
            }
            assert_eq!(covered.len(), typed.nodes().len());
            assert_eq!(covered.len(), maps.0.len());
            assert_eq!(expressions.len(), baseline.3.len());
            for (row, old) in expressions.iter().zip(&baseline.3) {
                let node = typed.nodes().node(row.typed_site.node()).unwrap();
                let mut expected = old.clone();
                assert!(expected.local_context.is_none());
                expected.local_context = node.links.context;
                assert_eq!(row, &expected);
                assert!(row.local_context.is_some());
            }
            if (2..=4).contains(&case) {
                assert!(scope.bindings().iter().any(|binding| {
                    !scope
                        .references()
                        .iter()
                        .any(|reference| reference.binding() == binding.id())
                }));
                assert!(inference.formulas().iter().any(|(_, formula)| {
                    let references = formula
                        .terms
                        .iter()
                        .map(|site| {
                            inference
                                .terms()
                                .iter()
                                .find(|(_, term)| &term.site == site)
                                .unwrap()
                                .1
                                .reference
                                .clone()
                        })
                        .collect::<Vec<_>>();
                    references[0] != references[1]
                }));
            }
            if case == 5 {
                let inner = scope
                    .references()
                    .iter()
                    .find(|reference| reference.scope().path().len() > 1)
                    .unwrap();
                assert_eq!(
                    scope.bindings()[inner.binding().index()].scope(),
                    inner.scope()
                );
            }
            if case == 6 {
                assert!(
                    typed
                        .contexts()
                        .iter()
                        .any(|(_, context)| context.layer == TypeContextLayer::Proof
                            && context.bindings.is_empty())
                );
            }
            let owner = symbols.symbols().iter().next().unwrap();
            let namespace = NamespacePath::new(module.path().as_str());
            let labels = ProofLabelSourceCollector::new(
                ast,
                &module,
                namespace.clone(),
                owner.contribution(),
                &source,
            )
            .and_then(|collector| collector.collect_with_theorem_owners(symbols))
            .unwrap();
            let resolved = LabelResolver::new(labels.projections()).resolve(
                &module,
                &namespace,
                labels.references(),
            );
            let label_before = (labels.clone(), resolved.clone());
            let old_check = Semantics::check_theorem_skeletons(
                &source,
                &baseline.2,
                &scope,
                symbols,
                &labels,
                &resolved,
                None,
            );
            let new_check = Semantics::check_theorem_skeletons(
                &source, typed, &scope, symbols, &labels, &resolved, None,
            );
            if case < 2 {
                let old_check = old_check?;
                let new_check = new_check?;
                assert_eq!(new_check.owners(), old_check.owners());
                assert_eq!(new_check.scope(), old_check.scope());
                assert_eq!(new_check.labels(), old_check.labels());
                assert_eq!(new_check.resolved_labels(), old_check.resolved_labels());
                assert_eq!(old_check.typed_ast(), base);
                assert_eq!(new_check.typed_ast(), typed);
                let expected = lower_source_theorem_skeletons(&old_check, None)?;
                let core = lower_source_theorem_skeletons(&new_check, None)?;
                assert_eq!(core, expected);
                assert!(
                    core.proofs()
                        .iter()
                        .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
                );
                assert_eq!(core.obligation_seeds().len(), 2);
                assert!(
                    core.obligation_seeds()
                        .iter()
                        .all(|(_, seed)| seed.status == ObligationSeedStatus::Active
                            && seed.goal.is_some())
                );
                let vcs = generate_core_vcs(&core, snapshot_id(777))?;
                assert_eq!(vcs, generate_core_vcs(&expected, snapshot_id(777))?);
                assert_eq!(vcs.vcs().len(), 2);
                assert!(
                    vcs.vcs()
                        .iter()
                        .all(|vc| vc.status == mizar_vc::vc_ir::VcStatus::Open)
                );
                assert_ne!(vcs.vcs()[0].local_context, vcs.vcs()[1].local_context);
            } else {
                assert_eq!(new_check.unwrap_err(), old_check.unwrap_err());
            }
            assert_eq!(label_before, (labels, resolved));
            assert_eq!(
                Checker::check_source_theorem_equalities_with_expression_metadata(
                    &source, symbols, &scope
                )?,
                baseline
            );
            assert_eq!(
                Checker::check_source_theorem_equalities_with_local_contexts(
                    &source, symbols, &scope
                )?,
                output
            );
            assert_eq!(source_proof_scope_maps(&source, symbols, &scope)?, maps);
            assert_eq!(
                Semantics::source_theorem_binding_env(&source, symbols, &scope)?,
                lexical
            );
            let signature = (
                typed
                    .contexts()
                    .iter()
                    .map(|(_, row)| {
                        (
                            base.nodes().node(row.owner.node()).unwrap().resolved_node,
                            row.parent.map(|parent| {
                                base.nodes()
                                    .node(typed.contexts().get(parent).unwrap().owner.node())
                                    .unwrap()
                                    .resolved_node
                            }),
                            row.layer,
                            row.bindings.clone(),
                        )
                    })
                    .collect::<Vec<_>>(),
                typed
                    .nodes()
                    .iter()
                    .map(|(id, node)| (id, node.resolved_node, node.links.context))
                    .collect::<Vec<_>>(),
                expressions.clone(),
            );
            if let Some((source_id, previous)) = &disk_capture {
                assert_ne!(*source_id, typed.source_id());
                assert_eq!(previous, &signature);
            } else {
                disk_capture = Some((typed.source_id(), signature));
            }
            assert_eq!(
                before,
                (
                    source,
                    symbols.clone(),
                    scope,
                    ast.snapshot_text(),
                    frontend.source.clone()
                )
            );
        }
    }
    fs::remove_dir_all(root).unwrap();
    Ok(())
}

#[test]
fn source_theorem_local_type_contexts_preserve_upstream_errors() {
    use mizar_checker::type_checker::TermFormulaChecker as Checker;
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::{NamespacePath, SymbolEnv, SymbolEnvIndexes},
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        resolved_ast::{ModuleId, SurfaceResolvedArena},
        symbols::{SignatureProjectionExtractor, validate_source_proof_occurrences},
    };
    use mizar_session::{ModulePath, PackageId};
    use mizar_syntax::{SurfaceAstBuilder, SurfaceNodeKind as K};
    let text = "theorem T: for X being set, Unused being set holds X = X proof let X be set; thus X = X; end;";
    let ast = parse(text);
    let module = ModuleId::new(
        PackageId::new("local-type-context-test"),
        ModulePath::new("owners"),
    );
    let source = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let collected = SignatureProjectionExtractor::new(
        &ast,
        &shells,
        NamespacePath::new(module.path().as_str()),
    )
    .collect(&module);
    let symbols = collected.env();
    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&ast, &module, symbols),
    )
    .unwrap();
    let empty = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
    let foreign_module = ModuleId::new(PackageId::new("foreign"), ModulePath::new("other"));
    let mut controls = vec![
        (
            "foreign environment",
            source.clone(),
            empty.clone(),
            scope.clone(),
        ),
        (
            "foreign source",
            SurfaceResolvedArena::lower(&ast, &foreign_module).unwrap(),
            symbols.clone(),
            scope.clone(),
        ),
    ];
    for control in ["foreign scope", "recovered source"] {
        let mut builder = SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = Vec::new();
        for view in ast.node_views() {
            let children = view
                .children()
                .iter()
                .map(|id| rebuilt[id.index()])
                .collect();
            rebuilt.push(match view.kind() {
                K::Token(token)
                    if control == "foreign scope" && token.text.as_ref() == "Unused" =>
                {
                    builder.add_token(token.kind, "Hidden", view.range())
                }
                K::Token(token) if control == "recovered source" && token.text.as_ref() == "=" => {
                    builder.add_recovered_token(token.kind, token.text.clone(), view.range())
                }
                K::Token(token) => builder.add_token(token.kind, token.text.clone(), view.range()),
                kind => builder.add_node(kind.clone(), view.range(), children),
            });
        }
        let changed = builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None);
        if control == "foreign scope" {
            let foreign = SourceVariableScopeResolver::resolve_proof_occurrences(
                SourceVariableScopeInput::new(&changed, &module, symbols),
            )
            .unwrap();
            assert_eq!(foreign.source_id(), scope.source_id());
            assert_eq!(foreign.references(), scope.references());
            assert_ne!(foreign.bindings(), scope.bindings());
            controls.push((control, source.clone(), symbols.clone(), foreign));
        } else {
            controls.push((
                control,
                SurfaceResolvedArena::lower(&changed, &module).unwrap(),
                symbols.clone(),
                scope.clone(),
            ));
        }
    }
    let unsupported = parse(&format!("reserve spare for object; {text}"));
    let unsupported_source = SurfaceResolvedArena::lower(&unsupported, &module).unwrap();
    let shells = DeclarationShellCollector::new(&unsupported, &module).collect();
    let collected = SignatureProjectionExtractor::new(
        &unsupported,
        &shells,
        NamespacePath::new(module.path().as_str()),
    )
    .collect(&module);
    let unsupported_scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&unsupported, &module, collected.env()),
    )
    .unwrap();
    controls.push((
        "unsupported type",
        unsupported_source.clone(),
        collected.env().clone(),
        unsupported_scope.clone(),
    ));
    controls.push(("combined precedence", unsupported_source, empty, scope));
    for (control, source, symbols, scope) in controls {
        let before = (source.clone(), symbols.clone(), scope.clone());
        let expected = Checker::check_source_theorem_equalities_with_expression_metadata(
            &source, &symbols, &scope,
        )
        .unwrap_err();
        if control == "combined precedence" {
            assert_eq!(
                expected,
                validate_source_proof_occurrences(&source, &symbols, &scope).unwrap_err()
            );
        }
        assert_eq!(
            Checker::check_source_theorem_equalities_with_local_contexts(&source, &symbols, &scope)
                .unwrap_err(),
            expected,
            "{control}"
        );
        let expected_replay = Checker::check_source_theorem_equalities_with_expression_metadata(
            &source, &symbols, &scope,
        )
        .unwrap_err();
        assert_eq!(
            Checker::check_source_theorem_equalities_with_local_contexts(&source, &symbols, &scope)
                .unwrap_err(),
            expected_replay,
            "{control}"
        );
        assert_eq!(before, (source, symbols, scope));
    }
}
