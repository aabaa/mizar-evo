use super::*;

#[test]
fn readable_constructor_heads_reach_genuine_frontend_patterns_and_later_uses() {
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind as T};
    use mizar_frontend::lexing::TokenKind as L;
    // EN2.5.2 and EN5.2/6.2/7.2: fresh declarations, with ordinary identifier controls.
    for (ordinal, (name, kind, text)) in [
        ("1-sorted", K::ModePattern, "definition mode ModeLabel: 1-sorted is set; end; reserve y for 1-sorted;"),
        ("R-module", K::StructurePattern, "definition struct R-module where field carrier -> set; end; end; reserve y for R-module;"),
        ("C-star-algebra", K::AttributePattern, "definition let X be set; attr AttrLabel: X is C-star-algebra means X = X; end; reserve y for C-star-algebra set;"),
    ].into_iter().enumerate() {
        for ordinary in [false, true] {
            let text = if ordinary { text.replace(name, "Ordinary") } else { text.to_owned() };
            let name = if ordinary { "Ordinary" } else { name };
            let frontend = task257c4c1_frontend_output(&text, 170 + ordinal);
            assert_eq!(frontend.cache_keys.tokens.version.as_ref(), "mizar-frontend/token-stream-cache-key/v6");
            let mut prior_tokens = frontend.cache_keys.tokens.clone();
            prior_tokens.version = std::sync::Arc::from("mizar-frontend/token-stream-cache-key/v5");
            assert_ne!(frontend.cache_keys.tokens.stable_hash(), prior_tokens.stable_hash());
            let current_ast_key = frontend.cache_keys.ast.as_ref().unwrap();
            assert_eq!(current_ast_key.token_stream_hash, frontend.cache_keys.tokens.stable_hash());
            assert_eq!(current_ast_key.parser_version.version.as_ref(), mizar_frontend::parsing::MIZAR_PARSER_CACHE_KEY_VERSION);
            let mut prior_ast = current_ast_key.clone();
            prior_ast.token_stream_hash = prior_tokens.stable_hash();
            assert_ne!(current_ast_key.stable_hash(), prior_ast.stable_hash());
            assert!(frontend.diagnostics.is_empty(), "{text}: {:?}", frontend.diagnostics);
            assert!(frontend.tokens.diagnostics.is_empty());
            let ast = frontend.ast.as_ref().unwrap();
            assert!(ast.node_views().all(|node| !node.is_recovered()));
            let patterns = ast.node_views().filter(|node| node.kind() == &kind).collect::<Vec<_>>();
            assert_eq!(patterns.len(), 1);
            let children = patterns[0].child_views().collect::<Vec<_>>();
            assert_eq!(children.len(), 1);
            let token = children[0].as_token().unwrap();
            assert_eq!((token.kind, token.text.as_ref()), (if ordinary { T::Identifier } else { T::UserSymbol }, name));
            let positions = text.match_indices(name).map(|(position, _)| position).collect::<Vec<_>>();
            assert_eq!(positions.len(), 2);
            assert_eq!(children[0].range(), range(ast.source_id, positions[0], positions[0] + name.len()));
            for (index, position) in positions.into_iter().enumerate() {
                let expected = range(ast.source_id, position, position + name.len());
                let token = frontend.tokens.tokens().iter().find(|token| token.span == expected).unwrap();
                assert_eq!(token.text.as_ref(), name);
                assert_eq!(token.kind, if ordinary && index == 0 { L::Identifier } else { L::UserSymbol });
                assert!(ast.token_views().any(|node| node.range() == expected && node.as_token().is_some_and(|token| token.text.as_ref() == name)));
            }
        }
    }
}

#[test]
fn repository_cases_match_fresh_discovery_and_keep_owned_mutations_local() {
    let fresh = build_test_plan(&step5c11_config()).unwrap();
    let selectors: [fn() -> crate::harness::TestCase; 2] =
        [step5c3_registration_case, step5c13_registration_case];
    for select in selectors {
        let mut case = select();
        let expected = fresh.cases.iter().find(|item| item.id == case.id).unwrap();
        assert_eq!(&case, expected);
        case.expectation.tags.push("local-mutation".to_owned());
        assert_eq!(&select(), expected);
    }
}

#[test]
fn source_direct_premise_candidate_preserves_real_source_handoff_and_fresh_kernel_check() {
    use mizar_atp::problem::{AtpFingerprint, AtpTargetBinding};
    use mizar_kernel::checker::KernelCheckStatus;
    use mizar_vc::{discharge::{DischargeInput, DischargePolicy, DischargeRule, DischargeEvidenceSource,
        DischargeEvidenceReplay, try_discharge}, kernel_evidence_handoff::{KernelFormulaSource,
        build_source_existential_discharge_candidate as build, build_source_existential_kernel_handoff}, vc_ir::VcStatus};
    for (multiple, case) in [(false, step5c3_registration_case()), (true, step5c13_registration_case())] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let renamed = original.replace("wmarked", "retagged").replace("oamarked", "leftmarked")
            .replace("obmarked", "rightmarked").replace("X", "Value");
        for (base, formal) in [(&original, "X"), (&renamed, "Value")] {
            let false_definitions = base.replace(&format!("means {formal} = {formal}"),
                &format!("means not {formal} = {formal}"));
            assert_ne!(&false_definitions, base);
            for text in [base, &false_definitions] {
                let (core, open, _) = source_candidate_core_and_vcs(multiple, text);
                let discharge = try_discharge(DischargeInput { vc_set: &open, policy: &DischargePolicy::default() }).unwrap();
                let mut contexts = std::collections::BTreeSet::new();
                for record in discharge.evidence_records() {
                    assert_eq!(record.source, DischargeEvidenceSource::NewlyProduced);
                    assert_eq!(record.rule, Some(DischargeRule::DirectLocalFact));
                    assert_eq!(record.replay, DischargeEvidenceReplay::ExplicitInputRefs);
                    let before = open.vc(record.vc).unwrap();
                    assert_eq!(before.status, VcStatus::Open);
                    let expected = build_source_existential_kernel_handoff(&core, &open, record.vc).unwrap();
                    let candidate = build(&core, &open, record.vc, &discharge).unwrap();
                    assert_eq!(candidate, expected);
                    assert!(candidate.targets_vc(&open, record.vc).unwrap());
                    assert!(!candidate.targets_vc(discharge.vc_set(), record.vc).unwrap());
                    assert!(contexts.insert(candidate.context_identity_hash().as_bytes().to_vec()));
                    let envelope = candidate.canonical_evidence();
                    assert!(envelope.substitutions().is_empty());
                    assert_eq!(envelope.formula_evidence().len(), 1);
                    assert!(matches!(envelope.formula_evidence()[0].source(),
                        KernelFormulaSource::GeneratedVcFact { .. }));
                    let fingerprint = envelope.target_vc();
                    let target = AtpTargetBinding::new(AtpFingerprint::new(fingerprint.algorithm_id,
                        fingerprint.digest.clone()).unwrap(), format!("{:?}", before.source)).unwrap();
                    let wire = candidate.encode_local_evidence(None).unwrap();
                    assert_eq!(source_candidate_check_bytes(&candidate, &wire, &target).unwrap(), KernelCheckStatus::Accepted);
                    assert!(build_source_existential_kernel_handoff(&core, discharge.vc_set(), record.vc).is_err());
                    assert!(build(&core, discharge.vc_set(), record.vc, &discharge).is_err());
                }
                assert_eq!(contexts.len(), if multiple { 3 } else { 1 });
                for vc in open.vcs().iter().filter(|vc| !discharge.evidence_records().iter().any(|record| record.vc == vc.id)) {
                    assert!(build(&core, &open, vc.id, &discharge).is_err(), "correctness leaf {:?}", vc.id);
                }
            }
        }
    }
}

#[test]
fn source_direct_premise_candidate_rejects_actual_mismatched_and_preserved_outputs() {
    use mizar_core::core_ir::{CoreDiagnosticId, CoreSourceAnchor};
    use mizar_session::{InMemorySessionIdAllocator, SessionIdAllocator};
    use mizar_vc::{discharge::{DischargeInput, DischargePolicy, DischargeRule, DischargeEvidenceSource,
        DischargeEvidenceReplay, try_discharge}, kernel_evidence_handoff::build_source_existential_discharge_candidate as build, vc_ir::*};
    for (multiple, case) in [(false, step5c3_registration_case()), (true, step5c13_registration_case())] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (core, open, _) = source_candidate_core_and_vcs(multiple, &text);
        let policy = DischargePolicy::default();
        let discharge = try_discharge(DischargeInput { vc_set: &open, policy: &policy }).unwrap();
        let selected = discharge.evidence_records().last().unwrap().vc;
        let unsupported = open.vcs().iter().find(|vc| !discharge.evidence_records().iter().any(|record| record.vc == vc.id)).unwrap().id;
        for target in [unsupported, VcId::new(open.vcs().len())] {
            assert!(build(&core, &open, target, &discharge).is_err());
        }
        let (foreign, foreign_open, _) = source_candidate_core_and_vcs(multiple, &text.replace("marked", "foreignmarked"));
        let foreign_output = try_discharge(DischargeInput { vc_set: &foreign_open, policy: &policy }).unwrap();
        assert!(build(&foreign, &open, selected, &discharge).is_err());
        assert!(build(&core, &foreign_open, selected, &discharge).is_err());
        assert!(build(&core, &open, selected, &foreign_output).is_err());
        let preserved = try_discharge(DischargeInput { vc_set: discharge.vc_set(), policy: &policy }).unwrap();
        assert!(preserved.evidence_records().iter().all(|record| record.source == DischargeEvidenceSource::PreExistingStatus
            && matches!(record.replay, DischargeEvidenceReplay::PreservedStatusOnly { .. })));
        assert!(build(&core, &open, selected, &preserved).is_err());
        for mutation in 0..19 {
            let mut parts = VcSetParts { schema_version: open.schema_version().clone(), snapshot: open.snapshot(), source: open.source(),
                module: open.module().clone(), generated_formulas: open.generated_formulas().to_vec(),
                vcs: open.vcs().to_vec(), seed_accounting: open.seed_accounting().to_vec() };
            match mutation {
                0 => parts.snapshot = super::super::shared::snapshot_id(1),
                1 => {
                    let ids = InMemorySessionIdAllocator::new();
                    ids.next_source_id(parts.snapshot).unwrap();
                    parts.source = ids.next_source_id(parts.snapshot).unwrap();
                    assert_ne!(parts.source, open.source());
                }
                2 => parts.module = VcModuleRef::new("foreign-module"),
                3 => { parts.vcs[unsupported.index()].provenance.pop(); }
                4 => { parts.vcs[selected.index()].premises.clear(); }
                5 => parts.vcs[selected.index()].goal = open.vc(unsupported).unwrap().goal,
                6 => parts.vcs[selected.index()].local_context = LocalContext::try_new(Vec::new(), vec![VerifierPolicyInput {
                    sort_key: CanonicalSortKey::new("different-policy"), key: PolicyKey::new("different-policy"), value: PolicyValue::new("true") }]).unwrap(),
                7 => parts.vcs[selected.index()].source.related.clear(),
                8 => {
                    let CoreSourceAnchor::SourceRange(range) = &mut parts.vcs[selected.index()].source.primary.anchor else { panic!("actual range") };
                    range.start += 1;
                }
                9 => {
                    parts.generated_formulas[0].shape = VcGeneratedFormulaShape::True;
                    parts.vcs[selected.index()].goal = VcFormulaRef::Generated(parts.generated_formulas[0].id);
                }
                10 => parts.vcs[selected.index()].status = discharge.vc_set().vc(selected).unwrap().status.clone(),
                11 => {
                    parts.vcs[selected.index()].status = discharge.vc_set().vc(selected).unwrap().status.clone();
                    let VcStatus::Discharged { evidence } = &mut parts.vcs[selected.index()].status else { unreachable!() };
                    evidence.rule = VcText::new("unsupported-preserved-rule");
                }
                12 => parts.vcs[selected.index()].status = VcStatus::NeedsAtp,
                13 => parts.vcs[selected.index()].status = VcStatus::PolicyOpen { policy: PolicyKey::new("test") },
                14 => parts.vcs[selected.index()].status = VcStatus::AssumedByPolicy { policy: PolicyKey::new("test"), marker: parts.vcs[selected.index()].premises[0].clone() },
                15 => parts.vcs[selected.index()].status = VcStatus::SkippedDueToInvalidInput { reason: VcText::new("test") },
                16 => parts.vcs[selected.index()].status = VcStatus::DeferredExternal { reason: VcText::new("test") },
                17 => parts.vcs[selected.index()].status = VcStatus::Error { diagnostic: CoreDiagnosticId::new(0) },
                _ => parts.generated_formulas[0].provenance.clear(),
            }
            let changed = VcSet::try_new(parts).unwrap_or_else(|error| panic!("mutation {mutation}: {error}"));
            let output = try_discharge(DischargeInput { vc_set: &changed, policy: &policy }).unwrap();
            if mutation != 12 { assert_ne!(output, discharge, "mutation {mutation}"); }
            if mutation == 0 {
                assert_eq!(output.evidence_records(), discharge.evidence_records());
                assert_eq!(output.explanations(), discharge.explanations());
            }
            if mutation == 9 {
                assert_eq!(output.evidence_records().iter().find(|record| record.vc == selected).unwrap().rule, Some(DischargeRule::GeneratedTautology));
            }
            if output != discharge {
                assert!(build(&core, &open, selected, &output).is_err(), "output mutation {mutation}");
            }
            assert!(build(&core, &changed, selected, &discharge).is_err(), "original mutation {mutation}");
            if mutation != 0 {
                assert!(build(&core, &changed, selected, &output).is_err(), "replayed original mutation {mutation}");
            }
        }
    }
}


#[test]
fn source_local_evidence_component_hashes_bind_actual_needs_atp_and_direct_premise_handoffs() {
    use mizar_session::hash_text;
    use mizar_vc::{
        discharge::{DischargeInput, DischargePolicy, DischargeRule, try_discharge},
        generator::generate_source_existential_registration,
        kernel_evidence_handoff::{
            build_source_existential_discharge_candidate, build_source_existential_kernel_handoff,
        },
        vc_ir::{GenerationSchemaVersion, VcSchemaVersion, VcStatus},
    };
    let profile_frame = concat!(
        "mizar-vc/local-evidence-component/v1\nprofile\n",
        "kernel-profile=KernelEvidenceProfile { profile_id: 1, clause_schema_version: 1, ",
        "clause_encoding_version: 1, clause_tautology_policy: Reject, ",
        "certificate_hash_input_algorithm: CanonicalEnvelopeV1 }\n",
    );
    let empty_substitutions =
        hash_text("mizar-vc/local-evidence-component/v1\nsubstitutions\n[substitutions]\n");
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
        let mut seen =
            std::collections::BTreeMap::<_, ([mizar_session::Hash; 4], mizar_session::Hash)>::new();
        for (base_index, (base, formal)) in [(&original, "X"), (&renamed, "Value")]
            .into_iter()
            .enumerate()
        {
            let false_definitions = base.replace(
                &format!("means {formal} = {formal}"),
                &format!("means not {formal} = {formal}"),
            );
            assert_ne!(&false_definitions, base);
            for (negative, text) in [(false, base), (true, &false_definitions)] {
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
                    let needs = discharge
                        .vc_set()
                        .vcs()
                        .iter()
                        .filter(|vc| vc.status == VcStatus::NeedsAtp)
                        .collect::<Vec<_>>();
                    assert_eq!(needs.len(), if multiple { 3 } else { 1 });
                    assert_eq!(discharge.evidence_records().len(), needs.len());
                    let mut handoffs = Vec::new();
                    for (registration, vc) in needs.iter().enumerate() {
                        let handoff = build_source_existential_kernel_handoff(
                            &core,
                            discharge.vc_set(),
                            vc.id,
                        )
                        .unwrap();
                        assert_eq!(
                            handoff.canonical_evidence().substitutions().len(),
                            if registration == 2 { 2 } else { 1 }
                        );
                        assert!(handoff.targets_vc(discharge.vc_set(), vc.id).unwrap());
                        let before =
                            build_source_existential_kernel_handoff(&core, &open, vc.id).unwrap();
                        let components = handoff.local_evidence_component_hashes().unwrap();
                        let prior = before.local_evidence_component_hashes().unwrap();
                        assert_eq!(&components[..3], &prior[..3]);
                        assert_ne!(components[3], prior[3]);
                        assert_ne!(
                            handoff.context_identity_hash(),
                            before.context_identity_hash()
                        );
                        handoffs.push((false, registration, handoff));
                    }
                    for (registration, record) in discharge.evidence_records().iter().enumerate() {
                        assert_eq!(record.rule, Some(DischargeRule::DirectLocalFact));
                        let handoff = build_source_existential_discharge_candidate(
                            &core, &open, record.vc, &discharge,
                        )
                        .unwrap();
                        assert_eq!(
                            handoff,
                            build_source_existential_kernel_handoff(&core, &open, record.vc)
                                .unwrap()
                        );
                        assert!(handoff.canonical_evidence().substitutions().is_empty());
                        assert!(handoff.targets_vc(&open, record.vc).unwrap());
                        handoffs.push((true, registration, handoff));
                    }
                    for (direct, registration, handoff) in handoffs {
                        let before = handoff.clone();
                        let wire = handoff.encode_local_evidence(None).unwrap();
                        let input = handoff.canonical_hash_input().to_vec();
                        let whole = handoff.canonical_hash();
                        let context = handoff.context_identity_hash();
                        let components = handoff.local_evidence_component_hashes().unwrap();
                        assert_eq!(
                            components,
                            handoff.local_evidence_component_hashes().unwrap()
                        );
                        assert_eq!(components[0], hash_text(profile_frame));
                        // Independent line-state oracle preserves every marker and LF.
                        let mut payloads = [String::new(), String::new(), String::new()];
                        let mut section = None;
                        for line in std::str::from_utf8(&input).unwrap().split_inclusive('\n') {
                            match line {
                                "[symbol-manifest]\n" | "[final-goal]\n" => section = Some(0),
                                "[substitutions]\n" => section = Some(1),
                                "[provenance]\n" => section = Some(2),
                                _ => {}
                            }
                            if let Some(section) = section {
                                payloads[section].push_str(line);
                            }
                        }
                        for (index, kind) in ["formula", "substitutions", "provenance"]
                            .iter()
                            .enumerate()
                        {
                            assert_eq!(
                                components[index + 1],
                                hash_text(&format!(
                                    "mizar-vc/local-evidence-component/v1\n{kind}\n{}",
                                    payloads[index]
                                ))
                            );
                        }
                        if direct {
                            assert_eq!(components[2], empty_substitutions);
                        } else {
                            assert_ne!(components[2], empty_substitutions);
                        }
                        assert!(payloads[0].contains("[variable-manifest]\n"));
                        assert!(payloads[0].contains("[formula-evidence]\n"));
                        assert!(
                            payloads[0]
                                .contains("[final-goal]\npolarity=AssertFalseForRefutation;")
                        );
                        if snapshot_index == 1 {
                            let (prior, prior_context) =
                                seen[&(base_index, negative, 0, direct, registration)];
                            assert_eq!(&components[..3], &prior[..3]);
                            assert_ne!(components[3], prior[3]);
                            assert_ne!(context, prior_context);
                        }
                        if negative && !direct {
                            let (positive, _) =
                                seen[&(base_index, false, snapshot_index, direct, registration)];
                            assert_ne!(components[1], positive[1]);
                            assert_ne!(components[3], positive[3]);
                        }
                        if base_index == 1 {
                            let (original, original_context) =
                                seen[&(0, negative, snapshot_index, direct, registration)];
                            assert_eq!(components[0], original[0]);
                            assert_ne!(components[3], original[3]);
                            assert_ne!(context, original_context);
                        }
                        seen.insert(
                            (base_index, negative, snapshot_index, direct, registration),
                            (components, context),
                        );
                        assert_eq!(handoff, before);
                        assert_eq!(handoff.canonical_hash_input(), input);
                        assert_eq!(handoff.canonical_hash(), whole);
                        assert_eq!(handoff.context_identity_hash(), context);
                        assert_eq!(handoff.encode_local_evidence(None).unwrap(), wire);
                    }
                }
            }
        }
    }
}

#[test]
fn functor_term_arity_uses_genuine_complete_direct_patterns() {
    use mizar_resolve::{declarations::DeclarationShellCollector, env::{SignatureShell, Visibility, ExportStatus},
        symbols::{SignatureProjectionExtractor, validate_source_symbol_env}};
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind as T, SyntaxRecoveryKind};
    // EN10.1/10.8/10.13: ordinary and circumfix interpretations must agree.
    let cases = [("Zero", Some(0)), ("Prefix x", Some(1)), ("x Postfix", Some(1)),
        ("x %% y", Some(2)), ("( x , y ) %% ( x , y )", Some(4)),
        ("Call ( x , y )", Some(2)), ("|. x .|", Some(1)), ("|. x , y .|", Some(2)),
        ("|. ( x , y ) .|", Some(2)), ("x %% x", Some(2)), ("f x g", None),
        ("f ( x ) g", Some(1)), ("f [ x ] y", None), ("", None), ("f ( )", None),
        ("f x ,", None), ("f ( ( x ) )", None), ("f x y z", None)];
    let mut inputs = Vec::new();
    for (ordinal, (notation, arity)) in cases.into_iter().enumerate() {
        for visibility in ["public", "private"] {
            let text = format!("definition let x,y,unused be set; {visibility} func Decl: {notation} -> set equals x; end;");
            let (frontend, source, collected) = source_registration_owner_inputs(&text, "functor-arity", "main", 0);
            let malformed = ordinal >= 13;
            assert_eq!(frontend.diagnostics.len(), usize::from(malformed), "{notation}: {:?}", frontend.diagnostics);
            let ast = frontend.ast.as_ref().unwrap();
            let patterns = ast.node_views().filter(|node| node.kind() == &K::FunctorPattern).collect::<Vec<_>>();
            assert_eq!(patterns.len(), 1);
            let pattern = patterns[0];
            let tokens = pattern.child_views().filter_map(|child| child.as_token()).map(|token| (token.kind, token.text.as_ref())).collect::<Vec<_>>();
            if ordinal == 12 {
                assert_eq!(pattern.children().len(), 3);
                assert_eq!(tokens, [(T::Identifier, "f"), (T::Identifier, "y")]);
                assert_eq!(pattern.child_views().nth(1).unwrap().kind(), &K::TemplateLoci);
            } else {
                assert_eq!(tokens, notation.split_whitespace().map(|text| (match text {
                    "(" | ")" | "," => T::ReservedSymbol, "%%" | "|." | ".|" => T::UserSymbol, _ => T::Identifier,
                }, text)).collect::<Vec<_>>(), "{notation}");
                assert!(pattern.child_views().filter(|node| node.as_token().is_some()).all(|node| node.children().is_empty()));
            }
            assert_eq!(ast.node_views().filter_map(|node| node.as_recovery()).collect::<Vec<_>>(),
                if malformed { vec![SyntaxRecoveryKind::MissingTerm] } else { vec![] }, "{notation}");
            if malformed {
                assert!(frontend.diagnostics[0].message.contains("expected functor pattern"));
                assert_eq!(pattern.child_views().last().unwrap().as_recovery(), Some(SyntaxRecoveryKind::MissingTerm));
            } else { assert!(ast.node_views().all(|node| !node.is_recovered())); }
            assert!(ast.trivia().skipped_token_ranges().is_empty());
            let shells = DeclarationShellCollector::new(ast, source.module()).collect();
            let projections = SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new("main")).extract();
            inputs.push((ordinal, notation, arity, visibility, text, source, collected, projections));
        }
    }
    // Finish every genuine grammar/diagnostic/recovery row before the arity RED.
    for (ordinal, notation, arity, visibility, text, source, collected, projections) in inputs {
        if notation.is_empty() { assert!(projections.is_empty()); continue; }
        assert_eq!(projections.len(), 1);
        let projection = &projections[0];
        assert_eq!(projection.arity(), arity, "{notation}");
        if ordinal >= 13 { assert!(arity.is_none()); continue; }
        assert_eq!((projection.primary_spelling(), projection.notation_spelling()), (notation, Some(notation)));
        let symbol = collected.env().symbols().iter().next().unwrap();
        let definition = collected.env().definitions().by_symbol(symbol.symbol()).unwrap();
        assert_eq!((definition.arity(), definition.signature(), symbol.signature()), (arity, projection.signature(), projection.signature()));
        assert_eq!((symbol.visibility(), symbol.export_status()), if visibility == "public" {
            (Visibility::Public, ExportStatus::Exported)
        } else { (Visibility::Private, ExportStatus::LocalOnly) });
        let Some(SignatureShell::Opaque { schema, payload }) = projection.signature() else { panic!("parser signature"); };
        assert_eq!(schema, "parser-signature-v1");
        assert!(payload.contains(&format!(";arity={};", arity.map_or("_".into(), |value| value.to_string()))));
        let rows = collected.env().lexical_summaries().visible_candidates(&NamespacePath::new("main"), notation);
        if (notation.contains("%%") || notation.contains("|.")) && visibility == "public" {
            assert_eq!(rows.len(), 1);
            assert_eq!((rows[0].symbol(), rows[0].arity()), (symbol.symbol(), arity));
        } else { assert!(rows.is_empty()); }
        if ordinal < 13 { validate_source_symbol_env(&source, collected.env()).unwrap(); }
        if ordinal == 3 && visibility == "public" {
            for (edit, module, skip) in [(text.clone(), "main", 0), (format!(":: trivia\n{}", text.replace("func ", "  func ")), "main", 0),
                (text.replace("unused", "spare,extra"), "main", 0), (text.clone(), "elsewhere", 0), (text.clone(), "main", 7)] {
                let (parsed, arena, actual) = source_registration_owner_inputs(&edit, "functor-arity", module, skip);
                assert!(parsed.diagnostics.is_empty() && actual.diagnostics().is_empty());
                assert_eq!(actual.env().definitions().iter().next().unwrap().arity(), Some(2));
                let current = actual.env().symbols().iter().next().unwrap();
                if module != "main" { assert_ne!(current.symbol(), symbol.symbol()); }
                else if !edit.contains("spare") { assert_eq!(current.symbol(), symbol.symbol()); }
                if edit == text && skip == 0 && module == "main" { assert_eq!(actual, collected); }
                if skip != 0 { assert_ne!(arena.source_id(), source.source_id()); }
                validate_source_symbol_env(&arena, actual.env()).unwrap();
            }
        }
    }
}

#[test]
fn functor_term_arity_preserves_genuine_overloads_and_lexical_wire() {
    use mizar_artifact::module_summary::ModuleSummaryIdentity;
    use mizar_lexer::{ExportedSymbolShape, UserSymbolArity, UserSymbolKind};
    use mizar_resolve::{declarations::DeclarationShellCollector, env::{Visibility, ExportStatus},
        symbols::{SignatureProjectionExtractor, validate_source_symbol_env}};
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind as T};
    let text = concat!("definition let x,y be set; public func First: x %% y -> set equals x; ",
        "private func HiddenFirst: x ^^ y -> set equals x; end; ",
        "definition let x,y be object; public func Second: x %% y -> set equals x; ",
        "private func HiddenSecond: x ^^ y -> set equals x; end;");
    let frontend = task257c4c1_frontend_output(text, 167);
    assert!(frontend.diagnostics.is_empty(), "{:?}", frontend.diagnostics);
    let ast = frontend.ast.as_ref().unwrap();
    assert!(ast.node_views().all(|node| !node.is_recovered()));
    assert_eq!(ast.node_views().filter(|node| node.kind() == &K::FunctorPattern)
        .map(|node| node.child_views().nth(1).unwrap().as_token().unwrap().kind).collect::<Vec<_>>(),
        [T::UserSymbol, T::UserSymbol, T::UserSymbol, T::UserSymbol]);
    let module = ResolverModuleId::new(frontend.source.package_id.clone(), frontend.source.module_path.clone());
    let shells = DeclarationShellCollector::new(ast, &module).collect();
    let collected = SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new(module.path().as_str())).collect(&module);
    assert!(collected.diagnostics().is_empty(), "{:?}", collected.diagnostics());
    let arena = SurfaceResolvedArena::lower(ast, &module).unwrap();
    validate_source_symbol_env(&arena, collected.env()).unwrap();
    let groups = collected.env().overloads().iter().collect::<Vec<_>>();
    assert_eq!(groups.len(), 2);
    for (notation, visibility, status) in [("x %% y", Visibility::Public, ExportStatus::Exported),
        ("x ^^ y", Visibility::Private, ExportStatus::LocalOnly)] {
        let group = groups.iter().find(|group| group.key().spelling() == notation).unwrap();
        assert_eq!((group.key().arity(), group.candidates().len()), (Some(2), 2));
        for symbol in group.candidates() {
            let entry = collected.env().symbols().get(symbol).unwrap();
            assert_eq!((collected.env().definitions().by_symbol(symbol).unwrap().arity(), entry.visibility(), entry.export_status()), (Some(2), visibility, status));
        }
        let rows = collected.env().lexical_summaries().visible_candidates(&NamespacePath::new(module.path().as_str()), notation);
        if visibility == Visibility::Public {
            assert_eq!(rows.len(), 2);
            for symbol in group.candidates() {
                assert_eq!(rows.iter().find(|row| row.symbol() == symbol).unwrap().arity(), Some(2));
            }
        } else { assert!(rows.is_empty()); }
    }
    assert_eq!(frontend.tokens.local_declarations().user_symbols.len(), 4);
    for local in &frontend.tokens.local_declarations().user_symbols {
        assert_eq!((local.kind, local.arity), (UserSymbolKind::Functor, UserSymbolArity::exact(2)));
    }
    let pairs = collected.pair_frontend_lexical_declarations(&frontend).unwrap();
    assert_eq!(pairs.len(), 2);
    let identity = ModuleSummaryIdentity { package_id: module.package().as_str().into(), package_version: None,
        lockfile_identity: None, module_path: module.path().as_str().into(), language_edition: "2026".into() };
    let wire = collected.export_frontend_lexical_contributions(&frontend, &identity).unwrap();
    assert_eq!(wire.len(), 2);
    for (symbol, local) in pairs {
        assert_eq!((symbol.primary_spelling(), local.spelling.as_str()), ("x %% y", "%%"));
        let payload = wire.iter().find(|entry| entry.key == symbol.symbol().local().as_str()).unwrap();
        let shape = ExportedSymbolShape::from_canonical_bytes(payload.payload.as_bytes()).unwrap();
        assert_eq!((shape.spelling.as_str(), shape.kind, shape.arity), ("%%", UserSymbolKind::Functor, local.arity));
    }
}

#[test]
fn predicate_term_arity_uses_genuine_complete_direct_patterns() {
    use mizar_resolve::{declarations::DeclarationShellCollector, env::{SignatureShell, Visibility, ExportStatus},
        symbols::{SignatureProjectionExtractor, validate_source_symbol_env}};
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind as T};
    // EN9.1/9.7: literal written slots, including ambiguous head assignments and repeated loci.
    let cases = [("Zero", 0), ("Prefix x", 1), ("x Postfix", 1), ("x Infix y", 2),
        ("x , y Multi z", 3), ("( x , y ) Left z", 3), ("x Right ( y , z )", 3),
        ("( x , y ) Both ( z , x )", 4), ("x Repeat x", 2), ("x %% y", 2), ("x Pair", 1)];
    let mut inputs = Vec::new();
    for (ordinal, (notation, arity)) in cases.into_iter().enumerate() {
        for visibility in ["public", "private"] {
            let text = format!("definition let x,y,z,unused be set; {visibility} pred Decl: {notation} means thesis; end;");
            let (frontend, source, collected) = source_registration_owner_inputs(&text, "predicate-arity", "main", 0);
            assert!(frontend.diagnostics.is_empty(), "{notation}: {:?}", frontend.diagnostics);
            assert!(collected.diagnostics().is_empty(), "{notation}: {:?}", collected.diagnostics());
            let ast = frontend.ast.as_ref().unwrap();
            assert!(ast.node_views().all(|node| !node.is_recovered()));
            let patterns = ast.node_views().filter(|node| node.kind() == &K::PredicatePattern).collect::<Vec<_>>();
            assert_eq!(patterns.len(), 1);
            let expected = notation.split_whitespace().map(|text| (match text {
                "(" | ")" | "," => T::ReservedSymbol, "%%" => T::UserSymbol, _ => T::Identifier,
            }, text)).collect::<Vec<_>>();
            assert_eq!(patterns[0].child_views().map(|child| {
                assert!(child.children().is_empty());
                let token = child.as_token().unwrap(); (token.kind, token.text.as_ref())
            }).collect::<Vec<_>>(), expected);
            let shells = DeclarationShellCollector::new(ast, source.module()).collect();
            let projections = SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new("main")).extract();
            assert_eq!(projections.len(), 1);
            inputs.push((ordinal, notation, arity, visibility, text, source, collected, projections));
        }
    }
    // All genuine grammar rows above must pass before the specification-derived arity RED.
    assert_eq!(inputs.iter().map(|(_,_,_,_,_,_,_,rows)| rows[0].arity()).collect::<Vec<_>>(),
        inputs.iter().map(|(_,_,arity,_,_,_,_,_)| Some(*arity)).collect::<Vec<_>>());
    for (ordinal, notation, arity, visibility, text, source, collected, projections) in inputs {
            let projection = &projections[0];
            assert_eq!((projection.primary_spelling(), projection.notation_spelling()), (notation, Some(notation)));
            let symbol = collected.env().symbols().iter().next().unwrap();
            let definition = collected.env().definitions().by_symbol(symbol.symbol()).unwrap();
            assert_eq!((definition.arity(), definition.signature(), symbol.signature()),
                (Some(arity), projection.signature(), projection.signature()));
            assert_eq!((symbol.visibility(), symbol.export_status()), if visibility == "public" {
                (Visibility::Public, ExportStatus::Exported)
            } else { (Visibility::Private, ExportStatus::LocalOnly) });
            let rows = collected.env().lexical_summaries().visible_candidates(&NamespacePath::new("main"), notation);
            if notation.contains("%%") && visibility == "public" {
                assert_eq!(rows.len(), 1);
                assert_eq!((rows[0].symbol(), rows[0].arity()), (symbol.symbol(), Some(arity)));
            } else { assert!(rows.is_empty()); }
            let Some(SignatureShell::Opaque { schema, payload }) = projection.signature() else { panic!("parser signature"); };
            assert_eq!(schema, "parser-signature-v1");
            let operand = format!(";arity={arity};");
            assert!(payload.contains(&operand));
            validate_source_symbol_env(&source, collected.env()).unwrap();
            // Reuse one representative to check fresh provenance, replay, trivia, unused lets and module identity.
            if ordinal == 3 && visibility == "public" {
                for (edit, module, skip) in [(text.clone(), "main", 0), (format!(":: trivia\n{}", text.replace("pred ", "  pred ")), "main", 0),
                    (text.replace("unused", "spare,extra"), "main", 0), (text.clone(), "elsewhere", 0), (text.clone(), "main", 7)] {
                    let (parsed, arena, actual) = source_registration_owner_inputs(&edit, "predicate-arity", module, skip);
                    assert!(parsed.diagnostics.is_empty() && actual.diagnostics().is_empty());
                    assert_eq!(actual.env().definitions().iter().next().unwrap().arity(), Some(2));
                    let current = actual.env().symbols().iter().next().unwrap();
                    if module == "main" { assert_eq!(current.symbol(), symbol.symbol()); }
                    else { assert_ne!(current.symbol(), symbol.symbol()); }
                    if edit == text && skip == 0 && module == "main" { assert_eq!(actual, collected); }
                    if skip != 0 { assert_ne!(arena.source_id(), source.source_id()); }
                    validate_source_symbol_env(&arena, actual.env()).unwrap();
                }
            }
    }
}

#[test]
fn predicate_term_arity_preserves_genuine_overloads_and_lexical_wire() {
    use mizar_artifact::module_summary::ModuleSummaryIdentity;
    use mizar_lexer::{ExportedSymbolShape, UserSymbolArity, UserSymbolKind};
    use mizar_resolve::{declarations::DeclarationShellCollector, env::{Visibility, ExportStatus},
        symbols::{SignatureProjectionExtractor, validate_source_symbol_env}};
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind as T};
    // Distinct parameter types preserve the existing predicate same-signature conflict policy.
    let text = concat!("definition let x,y be set; public pred First: x sharedrel y means thesis; ",
        "private pred HiddenFirst: x hiddenrel y means thesis; end; ",
        "definition let x,y be object; public pred Second: x sharedrel y means thesis; ",
        "private pred HiddenSecond: x hiddenrel y means thesis; end;");
    let frontend = task257c4c1_frontend_output(text, 166);
    assert!(frontend.diagnostics.is_empty(), "{:?}", frontend.diagnostics);
    let ast = frontend.ast.as_ref().unwrap();
    assert!(ast.node_views().all(|node| !node.is_recovered()));
    assert_eq!(ast.node_views().filter(|node| node.kind() == &K::PredicatePattern)
        .map(|node| node.child_views().nth(1).unwrap().as_token().unwrap().kind).collect::<Vec<_>>(),
        [T::Identifier, T::Identifier, T::UserSymbol, T::UserSymbol]);
    let module = ResolverModuleId::new(frontend.source.package_id.clone(), frontend.source.module_path.clone());
    let shells = DeclarationShellCollector::new(ast, &module).collect();
    let collected = SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new(module.path().as_str())).collect(&module);
    assert!(collected.diagnostics().is_empty(), "{:?}", collected.diagnostics());
    let arena = SurfaceResolvedArena::lower(ast, &module).unwrap();
    validate_source_symbol_env(&arena, collected.env()).unwrap();
    let groups = collected.env().overloads().iter().collect::<Vec<_>>();
    assert_eq!(groups.len(), 2);
    for (notation, visibility, status) in [("x sharedrel y", Visibility::Public, ExportStatus::Exported),
        ("x hiddenrel y", Visibility::Private, ExportStatus::LocalOnly)] {
        let group = groups.iter().find(|group| group.key().spelling() == notation).unwrap();
        assert_eq!((group.key().arity(), group.candidates().len()), (Some(2), 2));
        for symbol in group.candidates() {
            let entry = collected.env().symbols().get(symbol).unwrap();
            assert_eq!((collected.env().definitions().by_symbol(symbol).unwrap().arity(), entry.visibility(), entry.export_status()),
                (Some(2), visibility, status));
        }
        let rows = collected.env().lexical_summaries().visible_candidates(&NamespacePath::new(module.path().as_str()), notation);
        if visibility == Visibility::Public {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].arity(), Some(2));
            assert!(group.candidates().contains(rows[0].symbol()));
        } else { assert!(rows.is_empty()); }
    }
    assert_eq!(frontend.tokens.local_declarations().user_symbols.len(), 4);
    for local in &frontend.tokens.local_declarations().user_symbols {
        assert_eq!((local.kind, local.arity), (UserSymbolKind::Predicate, UserSymbolArity::exact(2)));
    }
    let pairs = collected.pair_frontend_lexical_declarations(&frontend).unwrap();
    assert_eq!(pairs.len(), 2);
    let identity = ModuleSummaryIdentity { package_id: module.package().as_str().into(), package_version: None,
        lockfile_identity: None, module_path: module.path().as_str().into(), language_edition: "2026".into() };
    let wire = collected.export_frontend_lexical_contributions(&frontend, &identity).unwrap();
    assert_eq!(wire.len(), 2);
    for (symbol, local) in pairs {
        assert_eq!((symbol.primary_spelling(), local.spelling.as_str()), ("x sharedrel y", "sharedrel"));
        let payload = wire.iter().find(|entry| entry.key == symbol.symbol().local().as_str()).unwrap();
        let shape = ExportedSymbolShape::from_canonical_bytes(payload.payload.as_bytes()).unwrap();
        assert_eq!((shape.spelling.as_str(), shape.kind, shape.arity), ("sharedrel", UserSymbolKind::Predicate, local.arity));
    }
}

#[test]
fn mode_parameter_arity_uses_genuine_direct_pattern_slots() {
    use mizar_resolve::{declarations::DeclarationShellCollector, env::SignatureShell,
        symbols::{SignatureProjectionExtractor, SymbolCollector, SymbolDeclarationProjection, validate_source_symbol_env}};
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind as T};
    // EN7.2/7.7: literal declared slots exclude the enclosing let inventory.
    let cases = [("Plain", 0), ("OfOne of x", 1), ("OfTwo of x , y", 2),
        ("OfThree of x , y , z", 3), ("OverOne over x", 1), ("OverTwo over x , y", 2),
        ("OverThree over x , y , z", 3), ("BracketOne [ x ]", 1),
        ("BracketTwo [ x , y ]", 2), ("BracketThree [ x , y , z ]", 3)];
    let text = format!("definition let x,y,z,unused be set; {} end;", cases.iter().enumerate()
        .map(|(index, (pattern, _))| format!("mode Decl{index}: {pattern} is set;"))
        .collect::<Vec<_>>().join(" "));
    let (frontend, source, collected) = source_registration_owner_inputs(&text, "mode-arity", "main", 0);
    assert!(frontend.diagnostics.is_empty(), "{:?}", frontend.diagnostics);
    assert!(collected.diagnostics().is_empty());
    let ast = frontend.ast.as_ref().unwrap();
    assert!(ast.node_views().all(|node| !node.is_recovered()));
    let patterns = ast.node_views().filter(|node| node.kind() == &K::ModePattern).collect::<Vec<_>>();
    assert_eq!(patterns.len(), cases.len());
    // Check genuine parser grammar before the arity oracle: every child is a direct token.
    for (pattern, (notation, _)) in patterns.iter().zip(cases) {
        let expected = notation.split_whitespace().map(|text| (match text {
            "of" | "over" => T::ReservedWord, "[" | "]" | "," => T::ReservedSymbol,
            _ => T::Identifier,
        }, text)).collect::<Vec<_>>();
        assert_eq!(pattern.child_views().map(|child| {
            assert!(child.children().is_empty());
            let token = child.as_token().unwrap(); (token.kind, token.text.as_ref())
        }).collect::<Vec<_>>(), expected);
    }
    let shells = DeclarationShellCollector::new(ast, source.module()).collect();
    let projections = SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new("main")).extract();
    assert_eq!(projections.iter().map(|row| row.arity()).collect::<Vec<_>>(),
        [Some(0), Some(1), Some(2), Some(3), Some(1), Some(2), Some(3), Some(1), Some(2), Some(3)]);
    let mut old_unknown = Vec::new();
    for (projection, (notation, arity)) in projections.iter().zip(cases) {
        assert_eq!((projection.primary_spelling(), projection.notation_spelling()), (notation, Some(notation)));
        let symbol = collected.env().symbols().iter().find(|row| row.primary_spelling() == notation).unwrap();
        let definition = collected.env().definitions().by_symbol(symbol.symbol()).unwrap();
        assert_eq!((definition.arity(), definition.signature(), symbol.signature()),
            (Some(arity), projection.signature(), projection.signature()));
        assert!(collected.env().lexical_summaries().visible_candidates(&NamespacePath::new("main"), notation).is_empty());
        let Some(SignatureShell::Opaque { schema, payload }) = projection.signature() else { panic!("parser signature"); };
        assert_eq!(schema, "parser-signature-v1");
        let operand = format!(";arity={arity};");
        assert!(payload.contains(&operand));
        // Identity-only reconstruction: omitted private lexical flags confer no source-env authority.
        old_unknown.push(SymbolDeclarationProjection::new(projection.shell(), projection.namespace().clone(), notation, projection.symbol_kind())
            .with_definition_kind(projection.definition_kind().unwrap()).with_overload_policy(projection.overload_policy())
            .with_notation_spelling(notation).with_signature(SignatureShell::Opaque { schema: schema.clone(), payload: payload.replace(&operand, ";arity=_;") }));
    }
    assert!(collected.env().overloads().is_empty());
    let old = SymbolCollector::new(ast.source_id, source.module(), &shells, &old_unknown).collect();
    for symbol in collected.env().symbols().iter() {
        let prior = old.env().symbols().iter().find(|row| row.primary_spelling() == symbol.primary_spelling()).unwrap();
        assert_ne!(prior.symbol(), symbol.symbol());
        assert_ne!(prior.symbol().fqn(), symbol.symbol().fqn());
        assert_eq!(old.env().definitions().by_symbol(prior.symbol()).unwrap().arity(), None);
    }
    validate_source_symbol_env(&source, collected.env()).unwrap();
    for (edit, module, skip) in [(text.clone(), "main", 0), (format!(":: trivia\n{}", text.replace("mode ", "  mode ")), "main", 0),
        (text.replace('x', "renamed"), "main", 0), (text.clone(), "elsewhere", 0), (text.clone(), "main", 7)] {
        let (replay, arena, actual) = source_registration_owner_inputs(&edit, "mode-arity", module, skip);
        assert!(replay.diagnostics.is_empty() && actual.diagnostics().is_empty());
        validate_source_symbol_env(&arena, actual.env()).unwrap();
        let actual_ast = replay.ast.as_ref().unwrap();
        let actual_shells = DeclarationShellCollector::new(actual_ast, arena.module()).collect();
        let actual_projections = SignatureProjectionExtractor::new(actual_ast, &actual_shells, NamespacePath::new(module)).extract();
        for ((original, projection), (_, arity)) in projections.iter().zip(actual_projections).zip(cases) {
            assert_eq!(projection.arity(), Some(arity));
            let symbol = actual.env().symbols().iter().find(|row| row.primary_spelling() == projection.primary_spelling()).unwrap();
            assert_eq!(actual.env().definitions().by_symbol(symbol.symbol()).unwrap().arity(), Some(arity));
            let prior = collected.env().symbols().iter().find(|row| row.primary_spelling() == original.primary_spelling()).unwrap();
            if module != "main" || projection.primary_spelling() != original.primary_spelling() { assert_ne!(symbol.symbol(), prior.symbol()); }
            else { assert_eq!(symbol.symbol(), prior.symbol()); }
        }
        if edit == text && skip == 0 && module == "main" { assert_eq!(actual, collected); }
        if skip != 0 { assert_ne!(arena.source_id(), source.source_id()); }
    }
}

#[test]
fn mode_parameter_arity_preserves_genuine_lexical_wire_and_private_exports() {
    use mizar_artifact::module_summary::ModuleSummaryIdentity;
    use mizar_lexer::{ExportedSymbolShape, UserSymbolArity, UserSymbolKind};
    use mizar_resolve::{declarations::DeclarationShellCollector, env::{ExportStatus, Visibility},
        symbols::{SignatureProjectionExtractor, validate_source_symbol_env}};
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind as T};
    let frontend = task257c4c1_frontend_output(concat!("definition let x,y,z be set; ",
        "public mode First: Shared [x,y] is set; public mode Second: Shared [x,y] is set; ",
        "private mode HiddenFirst: Hidden [x,y,z] is set; private mode HiddenSecond: Hidden [x,y,z] is set; end;"), 165);
    assert!(frontend.diagnostics.is_empty(), "{:?}", frontend.diagnostics);
    let ast = frontend.ast.as_ref().unwrap();
    let module = ResolverModuleId::new(frontend.source.package_id.clone(), frontend.source.module_path.clone());
    let shells = DeclarationShellCollector::new(ast, &module).collect();
    let collected = SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new(module.path().as_str())).collect(&module);
    assert!(collected.diagnostics().is_empty());
    assert_eq!(ast.node_views().filter(|node| node.kind() == &K::ModePattern)
        .map(|node| node.child_views().next().unwrap().as_token().unwrap().kind).collect::<Vec<_>>(),
        [T::Identifier, T::UserSymbol, T::Identifier, T::UserSymbol]);
    let arena = SurfaceResolvedArena::lower(ast, &module).unwrap();
    validate_source_symbol_env(&arena, collected.env()).unwrap();
    let groups = collected.env().overloads().iter().collect::<Vec<_>>();
    assert_eq!(groups.len(), 2);
    for (spelling, arity, visibility, status) in [("Shared [ x , y ]", 2, Visibility::Public, ExportStatus::Exported),
        ("Hidden [ x , y , z ]", 3, Visibility::Private, ExportStatus::LocalOnly)] {
        let group = groups.iter().find(|group| group.key().spelling() == spelling).unwrap();
        assert_eq!((group.key().arity(), group.candidates().len()), (Some(arity), 2));
        for symbol in group.candidates() {
            let definition = collected.env().definitions().by_symbol(symbol).unwrap();
            let entry = collected.env().symbols().get(symbol).unwrap();
            assert_eq!((definition.arity(), entry.visibility(), entry.export_status()), (Some(arity), visibility, status));
        }
        let rows = collected.env().lexical_summaries().visible_candidates(&NamespacePath::new(module.path().as_str()), spelling);
        if visibility == Visibility::Public {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].arity(), Some(2));
            assert!(group.candidates().contains(rows[0].symbol()));
        } else { assert!(rows.is_empty()); }
    }
    assert_eq!(frontend.tokens.local_declarations().user_symbols.len(), 4);
    for local in &frontend.tokens.local_declarations().user_symbols {
        assert_eq!((local.kind, local.arity), (UserSymbolKind::Mode, UserSymbolArity::exact(0)));
    }
    let pairs = collected.pair_frontend_lexical_declarations(&frontend).unwrap();
    assert_eq!(pairs.len(), 2);
    let identity = ModuleSummaryIdentity { package_id: module.package().as_str().into(), package_version: None,
        lockfile_identity: None, module_path: module.path().as_str().into(), language_edition: "2026".into() };
    let wire = collected.export_frontend_lexical_contributions(&frontend, &identity).unwrap();
    assert_eq!(wire.len(), 2);
    for (symbol, local) in pairs {
        assert_eq!((symbol.primary_spelling(), local.spelling.as_str()), ("Shared [ x , y ]", "Shared"));
        let payload = wire.iter().find(|entry| entry.key == symbol.symbol().local().as_str()).unwrap();
        let shape = ExportedSymbolShape::from_canonical_bytes(payload.payload.as_bytes()).unwrap();
        assert_eq!((shape.spelling.as_str(), shape.kind, shape.arity),
            ("Shared", UserSymbolKind::Mode, UserSymbolArity::exact(0)));
    }
}

#[test]
fn structure_parameter_arity_uses_genuine_direct_pattern_slots() {
    use mizar_resolve::{declarations::DeclarationShellCollector, env::{ExportStatus, SignatureShell, SymbolKind, Visibility},
        symbols::{SignatureProjectionExtractor, SymbolCollector, SymbolDeclarationProjection, validate_source_symbol_env}};
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind as T};
    // EN5.2/5.6 and A.5 count written slots, including repeats, independently of let bindings.
    let cases = [("Plain", 0), ("OfOne of x", 1), ("OfTwo of x , y", 2), ("OfRepeat of x , x , x", 3),
        ("OverOne over x", 1), ("OverTwo over x , y", 2), ("BracketOne [ x ]", 1),
        ("BracketTwo [ x , y ]", 2), ("BracketRepeat [ x , x , y ]", 3),
        ("Shared", 0), ("Shared of x", 1), ("Shared over x , y", 2)];
    let text = format!("definition let x,y,unused be set; {} end;", cases.iter().enumerate()
        .map(|(index, (pattern, _))| format!("{} struct {pattern} where field carrier{index} -> set; end;",
            if index == 11 { "private" } else { "public" })).collect::<Vec<_>>().join(" "));
    let (frontend, source, collected) = source_registration_owner_inputs(&text, "structure-arity", "main", 0);
    assert!(frontend.diagnostics.is_empty(), "{:?}", frontend.diagnostics);
    assert!(collected.diagnostics().is_empty(), "{:?}", collected.diagnostics());
    let ast = frontend.ast.as_ref().unwrap();
    assert!(ast.node_views().all(|node| !node.is_recovered()));
    let patterns = ast.node_views().filter(|node| node.kind() == &K::StructurePattern).collect::<Vec<_>>();
    assert_eq!(patterns.len(), cases.len());
    // Authenticate every genuine direct grammar row before the first declared-arity assertion.
    for (index, (pattern, (notation, _))) in patterns.iter().zip(cases).enumerate() {
        let expected = notation.split_whitespace().enumerate().map(|(slot, text)| (match text {
            "of" | "over" => T::ReservedWord, "[" | "]" | "," => T::ReservedSymbol,
            _ if slot == 0 && index >= 10 => T::UserSymbol, _ => T::Identifier,
        }, text)).collect::<Vec<_>>();
        assert_eq!(pattern.child_views().map(|child| {
            assert!(child.children().is_empty() && !child.is_recovered());
            let token = child.as_token().unwrap(); (token.kind, token.text.as_ref())
        }).collect::<Vec<_>>(), expected, "grammar row {index}");
    }
    let shells = DeclarationShellCollector::new(ast, source.module()).collect();
    let projections = SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new("main")).extract();
    let structures = projections.iter().filter(|row| row.symbol_kind() == SymbolKind::Structure).collect::<Vec<_>>();
    assert_eq!(structures.len(), cases.len());
    assert_eq!(structures.iter().map(|row| row.arity()).collect::<Vec<_>>(),
        cases.iter().map(|(_, arity)| Some(*arity)).collect::<Vec<_>>());
    let mut old_unknown = Vec::new();
    for (index, (projection, (notation, arity))) in structures.iter().zip(cases).enumerate() {
        assert_eq!((projection.primary_spelling(), projection.notation_spelling()), (notation, Some(notation)));
        let candidates = collected.env().symbols().visible_candidates(&NamespacePath::new("main"), notation);
        let [symbol] = candidates.as_slice() else { panic!("one structure"); };
        let definition = collected.env().definitions().by_symbol(symbol.symbol()).unwrap();
        assert_eq!((definition.arity(), definition.signature(), symbol.signature()),
            (Some(arity), projection.signature(), projection.signature()));
        assert_eq!((symbol.visibility(), symbol.export_status()), if index == 11 {
            (Visibility::Private, ExportStatus::LocalOnly) } else { (Visibility::Public, ExportStatus::Exported) });
        let rows = collected.env().lexical_summaries().visible_candidates(&NamespacePath::new("main"), notation);
        if index == 10 { assert_eq!(rows.len(), 1); assert_eq!((rows[0].symbol(), rows[0].arity()), (symbol.symbol(), Some(arity))); }
        else { assert!(rows.is_empty()); }
        let Some(SignatureShell::Opaque { schema, payload }) = projection.signature() else { panic!("parser signature"); };
        assert_eq!(schema, "parser-signature-v1");
        let operand = format!(";arity={arity};");
        assert!(payload.contains(&operand));
        old_unknown.push(SymbolDeclarationProjection::new(projection.shell(), projection.namespace().clone(), notation, projection.symbol_kind())
            .with_definition_kind(projection.definition_kind().unwrap()).with_overload_policy(projection.overload_policy())
            .with_notation_spelling(notation).with_signature(SignatureShell::Opaque { schema: schema.clone(), payload: payload.replace(&operand, ";arity=_;") }));
    }
    assert!(collected.env().overloads().is_empty());
    let old = SymbolCollector::new(ast.source_id, source.module(), &shells, &old_unknown).collect();
    for symbol in collected.env().symbols().iter().filter(|row| row.kind() == SymbolKind::Structure) {
        let prior = old.env().symbols().iter().find(|row| row.primary_spelling() == symbol.primary_spelling()).unwrap();
        assert_ne!(prior.symbol(), symbol.symbol()); assert_ne!(prior.symbol().fqn(), symbol.symbol().fqn());
        assert_ne!(prior.signature(), symbol.signature());
        assert!(collected.env().symbols().get(prior.symbol()).is_none());
        assert_eq!(old.env().definitions().by_symbol(prior.symbol()).unwrap().arity(), None);
    }
    validate_source_symbol_env(&source, collected.env()).unwrap();
    for (edit, module, skip) in [(text.clone(), "main", 0), (format!(":: trivia\n{}", text.replace("struct ", "  struct ")), "main", 0),
        (text.replace("unused", "unused,extra"), "main", 0), (text.clone(), "elsewhere", 0), (text.clone(), "main", 7)] {
        let (replay, arena, actual) = source_registration_owner_inputs(&edit, "structure-arity", module, skip);
        assert!(replay.diagnostics.is_empty() && actual.diagnostics().is_empty());
        validate_source_symbol_env(&arena, actual.env()).unwrap();
        for (notation, arity) in cases {
            let symbol = actual.env().symbols().iter().find(|row| row.primary_spelling() == notation).unwrap();
            let prior = collected.env().symbols().iter().find(|row| row.primary_spelling() == notation).unwrap();
            assert_eq!(actual.env().definitions().by_symbol(symbol.symbol()).unwrap().arity(), Some(arity));
            assert_eq!(symbol.signature(), prior.signature());
            if module == "main" { assert_eq!(symbol.symbol(), prior.symbol()); } else { assert_ne!(symbol.symbol(), prior.symbol()); }
        }
        if edit == text && skip == 0 && module == "main" { assert_eq!(actual, collected); }
        if skip != 0 { assert_ne!(arena.source_id(), source.source_id()); }
    }
}

#[test]
fn structure_parameter_arity_preserves_genuine_frontend_lexical_wire() {
    use mizar_artifact::module_summary::ModuleSummaryIdentity;
    use mizar_lexer::{ExportedSymbolShape, UserSymbolArity, UserSymbolKind};
    use mizar_resolve::{declarations::DeclarationShellCollector, symbols::SignatureProjectionExtractor};
    let frontend = task257c4c1_frontend_output(concat!("definition let x,y be set; ",
        "public struct Shared where field carrier -> set; end; public struct Shared [x,y] where field value -> set; end; ",
        "private struct Shared over x where field hidden -> set; end; end;"), 166);
    assert!(frontend.diagnostics.is_empty(), "{:?}", frontend.diagnostics);
    let ast = frontend.ast.as_ref().unwrap();
    let module = ResolverModuleId::new(frontend.source.package_id.clone(), frontend.source.module_path.clone());
    let shells = DeclarationShellCollector::new(ast, &module).collect();
    let collected = SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new(module.path().as_str())).collect(&module);
    assert!(collected.diagnostics().is_empty());
    for local in &frontend.tokens.local_declarations().user_symbols {
        assert_eq!((local.kind, local.arity), (UserSymbolKind::Structure, UserSymbolArity::exact(0)));
    }
    let pairs = collected.pair_frontend_lexical_declarations(&frontend).unwrap();
    assert!(pairs.iter().all(|(symbol, _)| symbol.primary_spelling() != "Shared over x"));
    assert!(pairs.iter().any(|(symbol, _)| symbol.primary_spelling() == "Shared [ x , y ]"));
    let identity = ModuleSummaryIdentity { package_id: module.package().as_str().into(), package_version: None,
        lockfile_identity: None, module_path: module.path().as_str().into(), language_edition: "2026".into() };
    let wire = collected.export_frontend_lexical_contributions(&frontend, &identity).unwrap();
    assert_eq!(wire.len(), pairs.len());
    for (symbol, local) in pairs {
        let payload = wire.iter().find(|entry| entry.key == symbol.symbol().local().as_str()).unwrap();
        let shape = ExportedSymbolShape::from_canonical_bytes(payload.payload.as_bytes()).unwrap();
        assert_eq!((shape.spelling.as_str(), shape.kind, shape.arity),
            (local.spelling.as_str(), UserSymbolKind::Structure, UserSymbolArity::exact(0)));
        assert!(symbol.signature().is_some());
    }
}

#[test]
fn arena_registration_owner_exact_current_keys_match_legacy_single_aggregate_and_empty() {
    use mizar_resolve::{env::SymbolKind, symbols::{source_registration_owner_payloads as raw,
        source_registration_owner_payloads_from_arena as owners, validate_source_symbol_env}};
    for (multiple, case) in [(false, step5c3_registration_case()), (true, step5c13_registration_case())] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (frontend, source, collected) = source_registration_owner_inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let before = (source.clone(), collected.clone());
        validate_source_symbol_env(&source, collected.env()).unwrap();
        let actual = owners(&source, collected.env()).unwrap();
        assert_eq!(actual, raw(frontend.ast.as_ref().unwrap(), &source, collected.env()).unwrap());
        let expected: Vec<(u8, &str, &[u8])> = if multiple {
            vec![(1, "oamarked", b"\0\0\0\x06OAADef\0\0\0\x08oamarked"),
                (1, "obmarked", b"\0\0\0\x06OBBDef\0\0\0\x08obmarked"),
                (2, "OAExists", b"\0\0\0\x08OAExists"),
                (2, "OBExists", b"\0\0\0\x08OBExists"),
                (2, "OABExists", b"\0\0\0\x09OABExists")]
        } else {
            vec![(1, "wmarked", b"\0\0\0\x05WMDef\0\0\0\x07wmarked"),
                (2, "WMarkedExists", b"\0\0\0\x0dWMarkedExists")]
        };
        assert_eq!(actual.len(), expected.len());
        for (tag, spelling, tail) in expected {
            let kind = if tag == 1 { SymbolKind::Attribute } else { SymbolKind::Registration };
            let entries = collected.env().symbols().iter().filter(|entry|
                entry.kind() == kind && entry.primary_spelling() == spelling).collect::<Vec<_>>();
            assert_eq!(entries.len(), 1);
            let bytes = [b"mizar-resolve/source-registration-owner/v1\0".as_slice(), &[tag],
                b"\0\0\0\x0aowner-test\0\0\0\x06owners", tail].concat();
            assert_eq!(actual.get(entries[0].symbol()), Some(&bytes));
        }
        assert_eq!((source, collected), before);
    }
    for text in ["", "definition\n let X be set;\n func Box: box X -> set equals X;\n coherence;\nend;\n"] {
        let (frontend, source, collected) = source_registration_owner_inputs(text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        validate_source_symbol_env(&source, collected.env()).unwrap();
        let actual = owners(&source, collected.env()).unwrap();
        assert!(actual.is_empty());
        assert_eq!(actual, raw(frontend.ast.as_ref().unwrap(), &source, collected.env()).unwrap());
    }
}

#[test]
fn arena_registration_owner_tracks_real_edits_reordering_relocation_and_participants() {
    use mizar_resolve::{env::SymbolKind, symbols::{source_registration_owner_payloads as raw,
        source_registration_owner_payloads_from_arena as owners, validate_source_symbol_env}};
    let text = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let (_, source, collected) = source_registration_owner_inputs(&text, "owner-test", "owners", 0);
    let baseline = owners(&source, collected.env()).unwrap();
    let functor = "definition\n  let X be set;\n  func WBoxDef: wbox X -> set equals X;\n  coherence;\nend;\n";
    let reordered = format!("{functor}\n{}", text.replace(functor, ""));
    assert_ne!(reordered, text);
    for (index, edited, module, skip) in [
        (0, format!("\n:: shifted ranges\n{}", text.replace("  ", "    ")), "owners", 0),
        (1, text.replace('X', "Subject"), "owners", 0),
        (2, reordered, "owners", 0),
        (3, text.clone(), "owners", 7),
        (4, text.replace("WMDef", "RenamedDef"), "owners", 0),
        (5, text.replace("wmarked", "renamedmarked"), "owners", 0),
        (6, text.replace("WMarkedExists", "RenamedExists"), "owners", 0),
        (7, text.clone(), "other_module", 0),
    ] {
        let (frontend, current, collected) = source_registration_owner_inputs(&edited, "owner-test", module, skip);
        assert!(frontend.diagnostics.is_empty(), "edit {index}");
        assert!(collected.diagnostics().is_empty(), "edit {index}");
        validate_source_symbol_env(&current, collected.env()).unwrap();
        let actual = owners(&current, collected.env()).unwrap();
        assert_eq!(actual, raw(frontend.ast.as_ref().unwrap(), &current, collected.env()).unwrap());
        assert_eq!(actual.len(), 2);
        for (kind, spelling, tag, tail) in [
            (SymbolKind::Attribute, if index == 5 { "renamedmarked" } else { "wmarked" }, 1,
                match index {
                    4 => b"\0\0\0\x0aRenamedDef\0\0\0\x07wmarked".as_slice(),
                    5 => b"\0\0\0\x05WMDef\0\0\0\x0drenamedmarked",
                    _ => b"\0\0\0\x05WMDef\0\0\0\x07wmarked",
                }),
            (SymbolKind::Registration, if index == 6 { "RenamedExists" } else { "WMarkedExists" }, 2,
                if index == 6 { b"\0\0\0\x0dRenamedExists".as_slice() } else { b"\0\0\0\x0dWMarkedExists" }),
        ] {
            let entry = collected.env().symbols().iter().find(|entry|
                entry.kind() == kind && entry.primary_spelling() == spelling).unwrap();
            let module_frame = if index == 7 { b"\0\0\0\x0cother_module".as_slice() } else { b"\0\0\0\x06owners" };
            let bytes = [b"mizar-resolve/source-registration-owner/v1\0".as_slice(), &[tag],
                b"\0\0\0\x0aowner-test", module_frame, tail].concat();
            assert_eq!(actual.get(entry.symbol()), Some(&bytes), "edit {index}: {spelling}");
        }
        if index == 2 { assert_ne!(actual.keys().collect::<Vec<_>>(), baseline.keys().collect::<Vec<_>>()); }
        if skip != 0 { assert_ne!(current.source_id(), source.source_id()); }
    }
}

#[test]
fn arena_registration_owner_preserves_foreign_augmented_recovered_and_profile_errors() {
    use mizar_resolve::symbols::{source_registration_owner_payloads as raw,
        source_registration_owner_payloads_from_arena as owners, validate_source_symbol_env};
    let text = std::fs::read_to_string(step5c3_registration_case().source_path).unwrap();
    let (frontend, source, collected) = source_registration_owner_inputs(&text, "owner-test", "owners", 0);
    let before = (frontend.ast.clone(), source.clone(), collected.clone());
    for edited in [text.replace("wmarked", "foreignmarked"),
        format!("{text}\ndefinition\n let X be set;\n func ExtraDef: extra X -> set equals X;\n coherence;\nend;\n")] {
        let (foreign_frontend, foreign_source, foreign) = source_registration_owner_inputs(&edited, "owner-test", "owners", 0);
        assert!(foreign_frontend.diagnostics.is_empty());
        assert!(foreign.diagnostics().is_empty());
        validate_source_symbol_env(&foreign_source, foreign.env()).unwrap();
        let foreign_before = (foreign_frontend.ast.clone(), foreign_source.clone(), foreign.clone());
        let error = validate_source_symbol_env(&source, foreign.env()).unwrap_err();
        assert_eq!(owners(&source, foreign.env()).unwrap_err(), error);
        assert_eq!(raw(frontend.ast.as_ref().unwrap(), &source, foreign.env()).unwrap_err(), error);
        let mismatch = source.validate_against(foreign_frontend.ast.as_ref().unwrap(), source.module()).unwrap_err();
        assert_eq!(raw(foreign_frontend.ast.as_ref().unwrap(), &source, foreign.env()).unwrap_err(), mismatch.to_string());
        let reverse = validate_source_symbol_env(&foreign_source, collected.env()).unwrap_err();
        assert_eq!(owners(&foreign_source, collected.env()).unwrap_err(), reverse);
        assert_eq!((foreign_frontend.ast, foreign_source, foreign), foreign_before);
    }
    for (edited, unsupported) in [
        (text.replace("means X = X", "means X ="), false),
        (format!("{text}\nregistration\n cluster Conditional: wmarked -> wmarked for set;\n coherence;\nend;\n"), true),
    ] {
        let (bad_frontend, bad_source, bad) = source_registration_owner_inputs(&edited, "owner-test", "owners", 0);
        let bad_before = (bad_frontend.ast.clone(), bad_source.clone(), bad.clone());
        let legacy = validate_source_symbol_env(&bad_source, bad.env());
        let error = raw(bad_frontend.ast.as_ref().unwrap(), &bad_source, bad.env()).unwrap_err();
        if unsupported {
            assert!(bad_frontend.diagnostics.is_empty());
            assert!(bad.diagnostics().is_empty());
            legacy.unwrap();
            assert_eq!(error, "unsupported source registration owner payload");
        } else {
            assert!(!bad_frontend.diagnostics.is_empty());
            assert_eq!(error, legacy.unwrap_err());
        }
        assert_eq!(owners(&bad_source, bad.env()).unwrap_err(), error);
        assert_eq!((bad_frontend.ast, bad_source, bad), bad_before);
    }
    assert_eq!((frontend.ast, source, collected), before);
}

#[test]
fn normal_statement_profile_rejects_genuine_builtin_reference() {
    use mizar_artifact::{
        proof_witness::{ProofStatus, ProofWitnessError, builtin_proof_witness_ref_json},
        store::{CanonicalJson, SchemaVersion},
        verified_artifact::*,
    };
    let (mut mixed, _) = builtin_enclosing_fixture(false);
    let builtin = mixed
        .proof_witnesses
        .iter()
        .find(|reference| reference.proof_status == ProofStatus::DischargedBuiltin)
        .unwrap()
        .clone();
    mixed.schema_version = SchemaVersion::new(1, 1);
    let mut normal = mixed.clone();
    normal
        .proof_witnesses
        .retain(|reference| reference.proof_status == ProofStatus::KernelVerified);
    assert!(!normal.proof_witnesses.is_empty());
    normal.obligations.retain(|obligation| {
        normal
            .proof_witnesses
            .iter()
            .any(|reference| reference.obligation_id == obligation.obligation_id)
    });
    normal.refresh_hashes().unwrap();
    let json = verified_artifact_json(&normal).unwrap();
    assert_eq!(
        read_verified_artifact(&json, Default::default()).unwrap(),
        normal
    );
    assert!(write_verified_artifact(&normal).is_ok());
    for result in [
        verified_artifact_json(&mixed),
        implementation_hash_input_json(&mixed),
    ] {
        assert!(matches!(
            result,
            Err(VerifiedArtifactError::ProofWitness {
                error: ProofWitnessError::SchemaVersion(_),
                ..
            })
        ));
    }
    let mut invalid = json;
    let CanonicalJson::Object(fields) = &mut invalid else {
        unreachable!()
    };
    fields.insert(
        "proof_witnesses".into(),
        CanonicalJson::array([builtin_proof_witness_ref_json(&builtin).unwrap()]),
    );
    assert!(
        matches!(read_verified_artifact(&invalid, Default::default()),
        Err(VerifiedArtifactError::ProofWitness { path, error: ProofWitnessError::SchemaVersion(_) })
            if path == "$.proof_witnesses[0]")
    );
}

#[test]
fn builtin_enclosing_roundtrip_mixed_and_normal_compatibility() {
    use mizar_artifact::{proof_witness::{ProofStatus, builtin_proof_witness_ref_json, proof_witness_ref_json},
        store::{CanonicalJson, SchemaVersion, canonical_json_bytes}, verified_artifact::*};
    assert_eq!(current_schema_version(), SchemaVersion::new(1, 1));
    assert!(schema_version_support().check(Some("2.0")).is_err());
    for multiple in [false, true] {
        let (artifact, _) = builtin_enclosing_fixture(multiple);
        for builtin_only in [false, true] {
            let mut artifact = artifact.clone();
            if builtin_only {
                artifact.proof_witnesses.retain(|w| w.proof_status == ProofStatus::DischargedBuiltin);
                artifact.obligations.retain(|o| artifact.proof_witnesses.iter().any(|w| w.obligation_id == o.obligation_id));
                artifact.refresh_hashes().unwrap();
            }
            let json = builtin_verified_artifact_json(&artifact).unwrap();
            if builtin_only {
                let mut equivalent = json.clone();
                let CanonicalJson::Object(fields) = &mut equivalent else { unreachable!() };
                let CanonicalJson::Array(references) = fields.get_mut("proof_witnesses").unwrap() else { unreachable!() };
                for reference in references {
                    let CanonicalJson::Object(fields) = reference else { unreachable!() };
                    fields.insert("schema_version".into(), CanonicalJson::string("03.00"));
                }
                let decoded = read_builtin_verified_artifact(&equivalent, Default::default()).unwrap();
                assert!(decoded.proof_witnesses.iter().all(|w| w.schema_version == SchemaVersion::new(3, 0)));
                assert_eq!(builtin_verified_artifact_json(&decoded).unwrap(), json);
            }
            let read = read_builtin_verified_artifact(&json, Default::default()).unwrap();
            assert_eq!(read.proof_witnesses.len(), artifact.proof_witnesses.len());
            for reference in &read.proof_witnesses {
                assert!(artifact.proof_witnesses.contains(reference));
                let encoded = match reference.proof_status {
                    ProofStatus::DischargedBuiltin => builtin_proof_witness_ref_json(reference).unwrap(),
                    ProofStatus::KernelVerified => proof_witness_ref_json(reference).unwrap(),
                    _ => panic!("unexpected source proof status"),
                };
                let CanonicalJson::Object(fields) = &json else { unreachable!() };
                let CanonicalJson::Array(witnesses) = &fields["proof_witnesses"] else { unreachable!() };
                assert!(witnesses.contains(&encoded));
            }
            assert_eq!(canonical_json_bytes(&json), canonical_json_bytes(&builtin_verified_artifact_json(&read).unwrap()));
            assert!(read_verified_artifact(&json, Default::default()).is_err());
            assert!(verified_artifact_json(&artifact).is_err());
            assert!(write_verified_artifact(&artifact).is_err());
            artifact.schema_version = SchemaVersion::new(1, 0);
            assert!(artifact.refresh_hashes().is_err());
        }
        let mut normal = artifact.clone();
        normal.schema_version = SchemaVersion::new(1, 0);
        normal.proof_witnesses.retain(|w| w.proof_status == ProofStatus::KernelVerified);
        normal.obligations.retain(|o| normal.proof_witnesses.iter().any(|w| w.obligation_id == o.obligation_id));
        normal.refresh_hashes().unwrap();
        let json = verified_artifact_json(&normal).unwrap();
        let read = read_verified_artifact(&json, Default::default()).unwrap();
        assert_eq!(json, verified_artifact_json(&read).unwrap());
        assert_eq!(write_verified_artifact(&normal).unwrap(), canonical_json_bytes(&json));
        assert!(read_builtin_verified_artifact(&json, Default::default()).is_err());
        assert!(builtin_verified_artifact_json(&normal).is_err());
    }
}

#[test]
fn builtin_enclosing_hash_participants_order_and_exclusions() {
    use mizar_artifact::{proof_witness::ProofStatus, store::{CanonicalJson, SchemaVersion, artifact_hash_domain}, verified_artifact::*};
    let (artifact, _) = builtin_enclosing_fixture(true);
    let interface = artifact.compute_interface_hash().unwrap();
    let implementation = artifact.compute_implementation_hash().unwrap();
    let json = builtin_verified_artifact_json(&artifact).unwrap();
    let domain = artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version);
    let store_hash = domain.hash(&json, &artifact_hash_excluded_paths());
    // Frozen participant sets are independent of the new profile dispatch.
    for (projection, keys) in [
        (interface_hash_input_json(&artifact).unwrap(), vec!["dependency_artifact_hashes", "exports", "module", "schema_version"]),
        (implementation_hash_input_json(&artifact).unwrap(), vec!["diagnostics", "exports", "expressions", "module", "obligations", "proof_witnesses", "provenance", "schema_version", "source_file", "source_hash"]),
    ] {
        let CanonicalJson::Object(fields) = projection else { unreachable!() };
        assert_eq!(fields.keys().map(String::as_str).collect::<Vec<_>>(), keys);
    }
    let mut reordered = artifact.clone();
    reordered.proof_witnesses.reverse();
    reordered.obligations.reverse();
    assert_eq!(builtin_verified_artifact_json(&reordered).unwrap(), json);
    for index in 0..4 {
        let mut changed = artifact.clone();
        let reference = &mut changed.proof_witnesses[0];
        match index {
            0 => reference.witness_path = "proof-witnesses/relocated.bin".into(),
            1 => reference.witness_artifact_hash.digest = mizar_session::hash_text("changed payload"),
            2 => reference.kernel_acceptance.accepted_result_hash.digest = mizar_session::hash_text("changed acceptance metadata"),
            _ => reference.kernel_acceptance.formula_context_hash = Some(reference.obligation_fingerprint.clone()),
        }
        changed.refresh_hashes().unwrap();
        assert_eq!(changed.interface_hash, interface);
        assert_ne!(changed.implementation_hash, implementation);
        assert_ne!(domain.hash(&builtin_verified_artifact_json(&changed).unwrap(), &artifact_hash_excluded_paths()), store_hash);
    }
    let mut local = artifact.clone();
    local.verified_at = Some("2026-10-16T01:02:03Z".into());
    local.provenance.cache_key = Some("caller-local-cache-key".into());
    local.refresh_hashes().unwrap();
    assert_eq!((local.interface_hash, local.implementation_hash), (interface, implementation));
    assert_ne!(builtin_verified_artifact_json(&local).unwrap(), json);
    assert_eq!(domain.hash(&builtin_verified_artifact_json(&local).unwrap(), &artifact_hash_excluded_paths()), store_hash);
    // Use the same genuine normal references in both enclosing versions.
    let mut normal = artifact.clone();
    normal.proof_witnesses.retain(|w| w.proof_status == ProofStatus::KernelVerified);
    normal.obligations.retain(|o| normal.proof_witnesses.iter().any(|w| w.obligation_id == o.obligation_id));
    normal.refresh_hashes().unwrap();
    let explicit_hashes = (normal.interface_hash, normal.implementation_hash);
    normal.schema_version = SchemaVersion::new(1, 0);
    normal.refresh_hashes().unwrap();
    assert_ne!(normal.interface_hash, explicit_hashes.0);
    assert_ne!(normal.implementation_hash, explicit_hashes.1);
}

#[test]
fn builtin_enclosing_rejects_invalid_envelopes_and_manifest_bindings() {
    use mizar_artifact::{manifest::*, proof_witness::{EvidenceKind, ProofWitnessError},
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{CanonicalJson, SchemaVersion, artifact_hash_domain, canonical_json_bytes}, verified_artifact::*};
    let (artifact, payloads) = builtin_enclosing_fixture(false);
    let json = builtin_verified_artifact_json(&artifact).unwrap();
    for version in [None, Some(CanonicalJson::string("2.1")), Some(CanonicalJson::string("3.0")), Some(CanonicalJson::string("bad")), Some(CanonicalJson::null())] {
        let mut bad = json.clone();
        let CanonicalJson::Object(fields) = &mut bad else { unreachable!() };
        fields.insert("unexpected".into(), CanonicalJson::null());
        match version { Some(v) => { fields.insert("schema_version".into(), v); } None => { fields.remove("schema_version"); } }
        assert!(matches!(read_builtin_verified_artifact(&bad, Default::default()), Err(VerifiedArtifactError::SchemaVersion(_) | VerifiedArtifactError::UnexpectedType { .. })));
    }
    for index in 0..7 {
        let mut bad = artifact.clone();
        match index {
            0 => bad.obligations[0].obligation_fingerprint.digest = mizar_session::hash_text("wrong fingerprint"),
            1 => bad.obligations[0].verifier_policy_fingerprint.digest = mizar_session::hash_text("wrong policy"),
            2 => bad.proof_witnesses[0].obligation_id.push_str("-orphan"),
            3 => bad.proof_witnesses.push(bad.proof_witnesses[0].clone()),
            4 => bad.obligations[0].accepted_witness_obligation_id = None,
            5 => bad.proof_witnesses[0].evidence_kind = EvidenceKind::AtpCertificate,
            _ => bad.proof_witnesses[0].schema_version = SchemaVersion::new(3, 1),
        }
        let error = builtin_verified_artifact_json(&bad).unwrap_err();
        assert!(match (index, &error) {
            (0, VerifiedArtifactError::WitnessReferenceMismatch { path, .. }) => path == "$.obligations[0].obligation_fingerprint",
            (1, VerifiedArtifactError::WitnessReferenceMismatch { path, .. }) => path == "$.obligations[0].verifier_policy_fingerprint",
            (2 | 4, VerifiedArtifactError::WitnessReferenceMismatch { path, .. }) => path == "$.obligations[0].accepted_witness_obligation_id",
            (3, VerifiedArtifactError::DuplicateEntry { path, .. }) => path == "$.proof_witnesses",
            (5, VerifiedArtifactError::ProofWitness { path, error: ProofWitnessError::InvalidStatusEvidence { .. } }) => path == "$.proof_witnesses[0]",
            (6, VerifiedArtifactError::ProofWitness { path, error: ProofWitnessError::SchemaVersion(_) }) => path == "$.proof_witnesses[0]",
            _ => false,
        }, "case {index}: {error:?}");
    }
    for index in 0..5 {
        let mut bad = json.clone();
        let CanonicalJson::Object(fields) = &mut bad else { unreachable!() };
        let CanonicalJson::Array(references) = fields.get_mut("proof_witnesses").unwrap() else { unreachable!() };
        let CanonicalJson::Object(reference) = &mut references[0] else { unreachable!() };
        match index {
            0 => { reference.remove("schema_version"); }
            1 => { reference.insert("schema_version".into(), CanonicalJson::string("4.0")); }
            2 => { reference.insert("legacy_certificate".into(), CanonicalJson::null()); }
            3 => { reference.insert("proof_status".into(), CanonicalJson::string("externally_attested")); }
            _ => { reference.insert("evidence_kind".into(), CanonicalJson::string("backend_certificate")); }
        }
        let error = read_builtin_verified_artifact(&bad, Default::default()).unwrap_err();
        assert!(matches!(&error, VerifiedArtifactError::ProofWitness { path, error: nested }
            if path == "$.proof_witnesses[0]" && match (index, nested) {
                (0 | 1, ProofWitnessError::SchemaVersion(_)) => true,
                (2, ProofWitnessError::UnknownField { path, field }) => path == "$" && field == "legacy_certificate",
                (3, ProofWitnessError::InvalidField { path, .. }) => path == "$.proof_status",
                (4, ProofWitnessError::InvalidField { path, .. }) => path == "$.evidence_kind",
                _ => false,
            }), "nested case {index}: {error:?}");
    }
    let mut foreign_module = artifact.module.clone();
    foreign_module.module_path = "foreign".into();
    for options in [
        VerifiedArtifactReadOptions { expected_module: Some(&foreign_module), ..Default::default() },
        VerifiedArtifactReadOptions { expected_interface_hash: Some(mizar_session::hash_text("wrong interface")), ..Default::default() },
        VerifiedArtifactReadOptions { expected_implementation_hash: Some(mizar_session::hash_text("wrong implementation")), ..Default::default() },
    ] { assert!(read_builtin_verified_artifact(&json, options).is_err()); }
    let temp = std::process::Command::new("mktemp").arg("-d").output().unwrap();
    assert!(temp.status.success());
    let root = PathBuf::from(String::from_utf8(temp.stdout).unwrap().trim());
    std::fs::create_dir_all(root.join("modules")).unwrap();
    std::fs::write(root.join("modules/source.json"), canonical_json_bytes(&json)).unwrap();
    let domain = artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version);
    let entry = ModuleArtifactEntry {
        module: artifact.module.clone(), source_file: artifact.source_file.clone(), source_hash: artifact.source_hash,
        artifact_file: "modules/source.json".into(), artifact_hash: ArtifactHashRef::new(ArtifactHashClass::Artifact,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version, domain.hash(&json, &artifact_hash_excluded_paths())),
        interface_hash: ArtifactHashRef::new(ArtifactHashClass::Interface, VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version, artifact.interface_hash),
        implementation_hash: ArtifactHashRef::new(ArtifactHashClass::Implementation, VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version, artifact.implementation_hash),
        module_summary_file: None, module_summary_hash: None, module_summary_interface_hash: None,
        registration_summary_file: None, registration_summary_hash: None, registration_interface_hash: None,
        proof_witnesses: artifact.proof_witnesses.iter().map(|w| ManifestProofWitnessEntry {
            obligation_id: w.obligation_id.clone(), obligation_fingerprint: w.obligation_fingerprint.clone(),
            witness_path: w.witness_path.clone(), witness_artifact_hash: w.witness_artifact_hash.clone(),
        }).collect(), diagnostics_hash: None,
    };
    let manifest = ArtifactManifest { schema_version: SchemaVersion::new(1, 0),
        package: PackageIdentity { package_id: artifact.module.package_id.clone(), package_version: None, lockfile_identity: None },
        artifact_root: "artifacts".into(), lockfile_hash: artifact.provenance.lockfile_hash.clone(),
        toolchain: artifact.provenance.toolchain.clone(), language_edition: "2026".into(),
        verifier_config_hash: artifact.provenance.verifier_config_hash.clone(), modules: vec![entry], development_artifacts: Vec::new(),
        provenance: ManifestProvenance { generated_by: "structural-test".into(), manifest_policy: "test".into(), transaction_format: "test".into() },
    };
    let options = ManifestFileReadOptions { validate_references: true, ..Default::default() };
    write_manifest_file(&root, &manifest).unwrap();
    assert_eq!(read_manifest_file(&root, options).unwrap().manifest.schema_version, SchemaVersion::new(1, 0));
    assert!(read_manifest_file(&root, ManifestFileReadOptions { validate_witness_files: true, ..options }).is_err());
    for (reference, bytes) in artifact.proof_witnesses.iter().zip(payloads) {
        let path = root.join(&reference.witness_path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }
    assert!(read_manifest_file(&root, ManifestFileReadOptions { validate_witness_files: true, ..options }).is_ok());
    for index in 0..7 {
        let mut bad = manifest.clone();
        let entry = &mut bad.modules[0];
        match index {
            0 => entry.artifact_hash.digest = mizar_session::hash_text("stale stored hash"),
            1 => { entry.proof_witnesses.pop(); }
            2 => entry.proof_witnesses[0].witness_path = "proof-witnesses/wrong.bin".into(),
            3 => entry.source_hash = mizar_session::hash_text("other source"),
            _ => {
                let version = match index { 4 => SchemaVersion::new(1, 0), 5 => SchemaVersion::new(2, 1), _ => SchemaVersion::new(9, 0) };
                entry.artifact_hash.schema_version = version;
                entry.artifact_hash.digest = artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, version).hash(&json, &artifact_hash_excluded_paths());
                entry.interface_hash.schema_version = version;
                entry.implementation_hash.schema_version = version;
            }
        }
        write_manifest_file(&root, &bad).unwrap();
        assert!(read_manifest_file(&root, options).is_err(), "manifest case {index}");
    }
    write_manifest_file(&root, &manifest).unwrap();
    let mut corrupt = json.clone();
    let CanonicalJson::Object(fields) = &mut corrupt else { unreachable!() };
    fields.insert("source_file".into(), CanonicalJson::string("src/corrupt.miz"));
    std::fs::write(root.join("modules/source.json"), canonical_json_bytes(&corrupt)).unwrap();
    assert!(matches!(read_manifest_file(&root, options), Err(ManifestError::StoreIo(_))));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_dependency_exact_original_single_and_aggregate_leaves() {
    use mizar_proof::status::source_registration_dependency_slice as slice;
    use mizar_vc::{dependency_slice::{DependencySliceCompleteness, DependencySliceInput,
        DependencyUnknownFamily, try_compute_dependency_slices}, vc_ir::VcStatus};
    let snapshot = super::super::shared::snapshot_id(0);
    for (count, case) in [(2, step5c3_registration_case()), (6, step5c13_registration_case())] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (source, nodes, collected, vcs) = source_dependency_inputs(&text, "vc_fingerprints", 0, snapshot);
        let before = (source.clone(), nodes.clone(), collected.env().clone());
        let expected = try_compute_dependency_slices(DependencySliceInput { vc_set: &vcs, discharge_output: None }).unwrap();
        assert_eq!(expected.slices().len(), count);
        for vc in vcs.vcs() {
            let actual = slice(&source, &nodes, collected.env(), snapshot, vc.id).unwrap();
            assert_eq!(&actual, expected.slice_for(vc.id).unwrap());
            assert_eq!(actual.status(), &VcStatus::Open);
            assert_eq!(actual.completeness(), DependencySliceCompleteness::IncompleteUncacheable);
            assert!(actual.requires_cache_miss());
            assert!(actual.unknowns().iter().any(|unknown| unknown.family() == DependencyUnknownFamily::UpstreamPayload));
            assert!(actual.unknowns().iter().any(|unknown| unknown.family() == DependencyUnknownFamily::Anchor));
        }
        assert_eq!((source, nodes, collected.env().clone()), before);
    }
}

#[test]
fn source_dependency_edits_snapshots_and_current_numeric_handles() {
    use mizar_proof::status::source_registration_dependency_slice as slice;
    use mizar_vc::dependency_slice::{DependencySliceInput, try_compute_dependency_slices};
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let ra = text.find("  cluster OAExists:").unwrap();
    let rb = text.find("  cluster OBExists:").unwrap();
    let rab = text.find("  cluster OABExists:").unwrap();
    let end = text[rab..].find("end;\nend;").unwrap() + rab + "end;\n".len();
    let reordered = format!("{}{}{}{}{}", &text[..ra], &text[rab..end], &text[ra..rb], &text[rb..rab], &text[end..]);
    assert_ne!(reordered, text);
    let mut baseline = None;
    for (edited, module, skip) in [
        (text.clone(), "vc_fingerprints", 0),
        (format!("\n:: shifted source\n{text}"), "vc_fingerprints", 0),
        (text.replace('X', "Subject"), "vc_fingerprints", 7),
        (text.replacen("means X = X", "means not X = X", 1), "vc_fingerprints", 0),
        (text.replace("OAExists", "RenamedExists"), "vc_fingerprints", 0),
        (text.clone(), "other_module", 0),
        (reordered, "vc_fingerprints", 0),
    ] {
        for snapshot in [super::super::shared::snapshot_id(0), super::super::shared::snapshot_id(41)] {
            let (source, nodes, collected, vcs) = source_dependency_inputs(&edited, module, skip, snapshot);
            let before = (source.clone(), nodes.clone(), collected.env().clone());
            let expected = try_compute_dependency_slices(DependencySliceInput { vc_set: &vcs, discharge_output: None }).unwrap();
            if baseline.is_none() { baseline = Some((vcs.clone(), expected.clone())); }
            let (old_vcs, old_slices) = baseline.as_ref().unwrap();
            assert_eq!(vcs.vcs().len(), old_vcs.vcs().len());
            for old_vc in old_vcs.vcs() {
                // Equal numeric handles select this call's graph, including across edits/snapshots.
                let actual = slice(&source, &nodes, collected.env(), snapshot, old_vc.id).unwrap();
                assert_eq!(&actual, expected.slice_for(old_vc.id).unwrap());
                assert_eq!(actual.status(), old_slices.slice_for(old_vc.id).unwrap().status());
                assert_eq!(actual.completeness(), old_slices.slice_for(old_vc.id).unwrap().completeness());
                assert!(actual.requires_cache_miss());
                assert!(!actual.unknowns().is_empty());
            }
            assert_eq!((source, nodes, collected.env().clone()), before);
        }
    }
}

#[test]
fn source_dependency_real_upstream_errors_precede_absent_handle() {
    use mizar_checker::registration_resolution::check_source_existential_registration_proof as check;
    use mizar_proof::status::source_registration_dependency_slice as slice;
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion};
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let snapshot = super::super::shared::snapshot_id(0);
    let (source, nodes, collected, vcs) = source_dependency_inputs(&text, "vc_fingerprints", 0, snapshot);
    let before = (source.clone(), nodes.clone(), collected.env().clone());
    let selected = vcs.vcs()[0].id;
    let absent = VcId::new(vcs.vcs().len());
    let invalid = "registration.proof.invalid_source_dependency_slice";
    assert_eq!(slice(&source, &nodes, collected.env(), snapshot, absent).unwrap_err(), invalid);
    let (foreign_source, foreign_nodes, foreign_env, _) = source_registration_formula_inputs(&text.replace("oamarked", "foreignmarked"), "vc-fingerprint-test", "vc_fingerprints", 0);
    let foreign_before = (foreign_source.clone(), foreign_nodes.clone(), foreign_env.env().clone());
    for (s, n, e) in [(&foreign_source, &nodes, collected.env()), (&source, &foreign_nodes, collected.env()), (&source, &nodes, foreign_env.env())] {
        let upstream = check(s, n, e).unwrap_err();
        assert_ne!(upstream, invalid);
        for id in [selected, absent] { assert_eq!(slice(s, n, e, snapshot, id).unwrap_err(), upstream); }
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
        let inputs_before = (s.clone(), n.clone(), e.env().clone());
        let upstream = if index == 0 { check(&s, &n, e.env()).unwrap_err() } else {
            let checked = check(&s, &n, e.env()).unwrap();
            let core = mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
            mizar_vc::generator::generate_source_existential_registration(&core, snapshot,
                &GenerationSchemaVersion::new("source-existential-registration-v1"), &VcSchemaVersion::new("vc-v1")).unwrap_err()
        };
        assert_ne!(upstream, invalid);
        for id in [selected, absent] { assert_eq!(slice(&s, &n, e.env(), snapshot, id).unwrap_err(), upstream); }
        assert_eq!((s, n, e.env().clone()), inputs_before);
    }
    assert_eq!((source, nodes, collected.env().clone()), before);
}

#[test]
fn source_ground_term_correspondence_preserves_exact_original_pairs_and_inputs() {
    use mizar_atp::{
        backend::source_registration_ground_term_correspondence as correspondence,
        problem::ConcreteFormat,
        translator::translate_source_existential_registration as translate,
    };
    use mizar_kernel::{
        certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
        clause::{SymbolKind, Term},
        formula_evidence::{Formula, FormulaEvidenceParseContext, parse_formula_evidence},
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
        for base in [&original, &renamed] {
            let formal = if base == &original { "X" } else { "Value" };
            let negative = base.replace(
                &format!("means {formal} = {formal}"),
                &format!("means not {formal} = {formal}"),
            );
            assert_ne!(&negative, base);
            for text in [base, &negative] {
                let (core, _, vcs) = source_candidate_core_and_vcs(multiple, text);
                let before = (core.clone(), vcs.clone());
                let selected = vcs
                    .vcs()
                    .iter()
                    .filter(|vc| vc.status == VcStatus::NeedsAtp)
                    .collect::<Vec<_>>();
                assert_eq!(selected.len(), if multiple { 3 } else { 1 });
                for vc in selected {
                    let handoff =
                        build_source_existential_kernel_handoff(&core, &vcs, vc.id).unwrap();
                    let envelope = handoff.canonical_evidence();
                    let target = envelope.target_vc();
                    let context = FormulaEvidenceParseContext::v1(
                        Fingerprint::new(target.algorithm_id, target.digest.clone()),
                        KernelProfileRecord::v1(
                            envelope.kernel_profile().profile_id,
                            ClauseTautologyPolicy::Reject,
                        ),
                    );
                    let parsed = parse_formula_evidence(
                        &handoff.encode_local_evidence(None).unwrap(),
                        &context,
                    )
                    .unwrap();
                    // Independent value oracle: final goal occurrences, never the set-premise join.
                    let goals = match &parsed.final_goal().formula {
                        Formula::Atom(_) => std::slice::from_ref(&parsed.final_goal().formula),
                        Formula::And(children) => children.as_slice(),
                        _ => panic!("actual source goal"),
                    };
                    assert!(!goals.is_empty());
                    let terms = goals.iter().map(|goal| {
                        let Formula::Atom(atom) = goal else { panic!("actual conjunct") };
                        assert_eq!(atom.symbol.kind, SymbolKind::Predicate);
                        let [term] = atom.arguments.as_slice() else { panic!("actual argument") };
                        assert!(matches!(term, Term::Application { symbol, arguments }
                            if symbol.kind == SymbolKind::FunctorPredicate && arguments.is_empty()));
                        term
                    }).collect::<Vec<_>>();
                    let expected_term = terms[0];
                    assert!(terms.iter().all(|term| *term == expected_term));
                    for substitution in parsed.substitutions() {
                        let [replacement] = substitution.payload.replacements.as_slice() else {
                            panic!("actual replacement")
                        };
                        assert_eq!(&replacement.actual_term, expected_term);
                    }
                    for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
                        let profile =
                            source_candidate_profile("source-ground-term-correspondence", format);
                        let problem = translate(&core, &vcs, vc.id, profile.clone()).unwrap();
                        let observed = problem.observed_ground_terms().unwrap();
                        assert_eq!(observed.len(), 1);
                        let expected = observed
                            .into_iter()
                            .map(|term| (term.clone(), expected_term.clone()))
                            .collect::<std::collections::BTreeMap<_, _>>();
                        let actual = correspondence(&core, &vcs, vc.id, profile.clone()).unwrap();
                        assert_eq!(actual, expected);
                        assert_eq!(correspondence(&core, &vcs, vc.id, profile).unwrap(), actual);
                    }
                }
                assert_eq!((core, vcs), before);
            }
        }
    }
}

#[test]
fn source_ground_term_correspondence_preserves_existing_whole_input_refusals() {
    use mizar_atp::{
        backend::source_registration_ground_term_correspondence as correspondence, problem::*,
        translator::translate_source_existential_registration as translate,
    };
    use mizar_core::core_ir::{CoreIr, CoreIrParts};
    use mizar_vc::vc_ir::*;
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (core, open, vcs) = source_candidate_core_and_vcs(multiple, &text);
        let (foreign, _, foreign_vcs) =
            source_candidate_core_and_vcs(multiple, &text.replace("marked", "foreignmarked"));
        let selected = vcs
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
        let absent = VcId::new(vcs.vcs().len());
        let mut parts = VcSetParts {
            schema_version: vcs.schema_version().clone(),
            snapshot: vcs.snapshot(),
            source: vcs.source(),
            module: vcs.module().clone(),
            generated_formulas: vcs.generated_formulas().to_vec(),
            vcs: vcs.vcs().to_vec(),
            seed_accounting: vcs.seed_accounting().to_vec(),
        };
        // A sibling outside the selected leaf still belongs to the producer's whole input.
        parts.vcs[discharged.index()].provenance.pop();
        let changed_vcs = VcSet::try_new(parts).unwrap();
        let mut parts = CoreIrParts {
            source_id: core.source_id(),
            module_id: core.module_id().clone(),
            items: core.items().clone(),
            terms: core.terms().clone(),
            formulas: core.formulas().clone(),
            definitions: core.definitions().clone(),
            proofs: core.proofs().clone(),
            proof_nodes: core.proof_nodes().clone(),
            algorithms: core.algorithms().clone(),
            algorithm_statements: core.algorithm_statements().clone(),
            generated: core.generated().clone(),
            obligation_seeds: core.obligation_seeds().clone(),
            source_map: core.source_map().clone(),
            diagnostics: core.diagnostics().clone(),
        };
        let id = core.generated().iter().next().unwrap().0;
        parts.generated.get_mut(id).unwrap().evidence.clear();
        let changed_core = CoreIr::try_new(parts).unwrap();
        let unsupported = LogicProfile::try_new(
            "unsupported",
            LogicFragment::TffLike,
            EqualitySupport::Supported,
            QuantifierPolicy::FirstOrder,
            SoftTypeStrategy::GuardPredicates,
            NativePropertySupport::Unsupported,
            [ConcreteFormat::Tptp].into_iter().collect(),
        )
        .unwrap();
        for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
            let profile = source_candidate_profile("source-ground-term-refusals", format);
            for (c, v, id, p) in [
                (&core, &open, selected, profile.clone()),
                (&core, &vcs, discharged, profile.clone()),
                (&core, &vcs, absent, profile.clone()),
                (&foreign, &vcs, selected, profile.clone()),
                (&core, &foreign_vcs, selected, profile.clone()),
                (&changed_core, &vcs, selected, profile.clone()),
                (&core, &changed_vcs, selected, profile.clone()),
                (&core, &changed_vcs, absent, profile.clone()),
                (&core, &vcs, selected, unsupported.clone()),
            ] {
                let before = (c.clone(), v.clone());
                let old_error = translate(c, v, id, p.clone()).unwrap_err();
                assert_eq!(correspondence(c, v, id, p).unwrap_err(), old_error);
                assert_eq!((c.clone(), v.clone()), before);
            }
        }
    }
}

#[test]
fn structured_source_candidates_preserve_legacy_container_and_original_context() {
    use mizar_atp::{
        backend::{
            BackendCandidatePayload, EncodedBackendProblem,
            build_source_registration_candidate_inputs as build,
        },
        problem::ConcreteFormat,
        translator::translate_source_existential_registration,
    };
    use mizar_kernel::checker::KernelCheckStatus;
    use mizar_vc::{
        kernel_evidence_handoff::build_source_existential_kernel_handoff, vc_ir::VcStatus,
    };
    for (multiple, case) in [
        (false, step5c3_registration_case()),
        (true, step5c13_registration_case()),
    ] {
        let original = std::fs::read_to_string(case.source_path).unwrap();
        let text = if multiple {
            original
                .replace("marked", "retagged")
                .replace("X", "Value")
                .replacen("means Value = Value", "means not Value = Value", 1)
        } else {
            original
        };
        let (core, _, vcs) = source_candidate_core_and_vcs(multiple, &text);
        let before = (core.clone(), vcs.clone());
        for vc in vcs
            .vcs()
            .iter()
            .filter(|vc| vc.status == VcStatus::NeedsAtp)
        {
            let handoff = build_source_existential_kernel_handoff(&core, &vcs, vc.id).unwrap();
            assert!(handoff.context_identity_is_consistent());
            let original_v2_bytes = source_candidate_wire(&handoff);
            for format in [ConcreteFormat::Tptp, ConcreteFormat::SmtLib] {
                let profile = source_candidate_profile("structured-source-compatibility", format);
                let problem =
                    translate_source_existential_registration(&core, &vcs, vc.id, profile.clone())
                        .unwrap();
                let expected_encoded = EncodedBackendProblem::encode(&problem, format).unwrap();
                let (encoded, candidate) = build(&core, &vcs, vc.id, profile, format).unwrap();
                assert_eq!(encoded, expected_encoded);
                assert_eq!(candidate.target_binding(), encoded.target_binding());
                assert_eq!(candidate.encoded_problem_hash(), encoded.input_hash());
                assert_eq!(candidate.provenance_hash(), encoded.provenance_hash());
                assert_eq!(candidate.formula_label_refs(), encoded.formula_labels());
                assert_eq!(candidate.symbol_binding_refs(), encoded.symbol_bindings());
                let BackendCandidatePayload::FormulaSubstitutionBytes(bytes) = candidate.payload()
                else {
                    panic!("actual source wire")
                };
                assert_eq!(bytes, &original_v2_bytes);
                assert_eq!(
                    source_candidate_check_bytes(&handoff, bytes, candidate.target_binding())
                        .unwrap(),
                    if multiple && vc.id.index() != 3 {
                        KernelCheckStatus::Rejected
                    } else {
                        KernelCheckStatus::Accepted
                    }
                );
            }
        }
        assert_eq!((core, vcs), before);
    }
}

#[test]
fn source_theorem_syntax_payloads_exact_current_keys_and_empty() {
    use mizar_resolve::{
        env::SymbolKind,
        symbols::{
            source_theorem_syntax_payloads_from_arena as syntax, validate_source_symbol_env,
        },
    };
    let t = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let u = t.replace("theorem T:", "theorem U:");
    // Complete literal frozen independently from the old parser and written grammar.
    let expected_t = concat!(
        "6d697a61722d7265736f6c76652f736f757263652d7468656f72656d2d73796e7461782f7631000060000000",
        "0600020065000000077468656f72656d000000000002006400000001540000000000020066000000013a0000",
        "0000002c000000010037000000040002006500000003666f7200000000003800000003000200640000000158",
        "0000000000020065000000056265696e67000000000015000000010019000000010002006500000003736574",
        "000000000002006500000005686f6c647300000000002d00000003001c00000001001d000000010002006400",
        "000001580000000000020066000000013d00000000001c00000001001d000000010002006400000001580000",
        "0000006200000004000200650000000570726f6f6600000000003d0000000300020065000000036c65740000",
        "0000003e00000003000200640000000158000000000002006500000002626500000000001500000001001900",
        "00000100020065000000037365740000000000020066000000013b0000000000540000000300020065000000",
        "047468757300000000004000000001002c00000001002d00000003001c00000001001d000000010002006400",
        "000001580000000000020066000000013d00000000001c00000001001d000000010002006400000001580000",
        "000000020066000000013b000000000002006500000003656e640000000000020066000000013b00000000",
    )
    .as_bytes()
    .as_chunks::<2>().0.iter()
    .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
    .collect::<Vec<_>>();
    assert_eq!(expected_t.len(), 527);
    let mut expected_u = expected_t.clone();
    // The unique complete Token/Identifier/length/T/child-count record at offset 64.
    assert_eq!(
        &expected_t[64..77],
        b"\x00\x02\x00\x64\x00\x00\x00\x01T\x00\x00\x00\x00"
    );
    expected_u[64..77].copy_from_slice(b"\x00\x02\x00\x64\x00\x00\x00\x01U\x00\x00\x00\x00");
    for (text, count) in [(t.to_owned(), 1), (format!("{t}\n{u}"), 2)] {
        let (frontend, source, collected) =
            source_registration_owner_inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let before = (source.clone(), collected.env().clone());
        let payloads = syntax(&source, collected.env()).unwrap();
        assert_eq!(payloads.len(), count);
        assert_eq!(collected.env().symbols().len(), count);
        for entry in collected.env().symbols().iter() {
            assert_eq!(entry.kind(), SymbolKind::Theorem);
            let expected = match entry.primary_spelling() {
                "T" => &expected_t,
                "U" => &expected_u,
                _ => unreachable!(),
            };
            assert_eq!(payloads.get(entry.symbol()), Some(expected));
        }
        assert_eq!(syntax(&source, collected.env()).unwrap(), payloads);
        assert_eq!((source, collected.env().clone()), before);
    }
    for text in [String::new(), t.replace("theorem T:", "lemma L:")] {
        let (frontend, source, collected) =
            source_registration_owner_inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let before = (source.clone(), collected.env().clone());
        validate_source_symbol_env(&source, collected.env()).unwrap();
        assert!(syntax(&source, collected.env()).unwrap().is_empty());
        assert_eq!((source, collected.env().clone()), before);
    }
}

#[test]
fn source_theorem_syntax_payloads_neutral_relocation_order_and_local_edits() {
    use mizar_resolve::{
        env::{SymbolKind, Visibility},
        symbols::source_theorem_syntax_payloads_from_arena as syntax,
    };
    let t = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let u = t.replace("theorem T:", "theorem U:");
    let text = format!("{t}\n{u}");
    let (_, source, collected) = source_registration_owner_inputs(&text, "owner-test", "owners", 0);
    let prior = syntax(&source, collected.env()).unwrap();
    let before = (source.clone(), collected.env().clone(), prior.clone());
    let baseline = collected
        .env()
        .symbols()
        .iter()
        .filter(|e| e.kind() == SymbolKind::Theorem)
        .map(|entry| (entry.primary_spelling(), &prior[entry.symbol()]))
        .collect::<std::collections::BTreeMap<_, _>>();
    let trivia = format!(":: λ 日本語\n{}", text.replace(' ', "  "));
    let leading = format!("{}\n{text}", t.replace("theorem T:", "lemma L:"));
    let renamed = format!("{}\n{u}", t.replace("theorem T:", "theorem R:"));
    let binder = format!("{}\n{u}", t.replace('X', "Y"));
    let local = format!(
        "{}\n{u}",
        t.replace("let X be set; thus X = X", "let Y be set; thus Y = Y")
    );
    for (index, edited, package, module, skip) in [
        (0, trivia, "owner-test", "owners", 0),
        (1, text.clone(), "owner-test", "owners", 7),
        (2, format!("{u}\n{t}"), "owner-test", "owners", 0),
        (3, leading, "owner-test", "owners", 0),
        (4, text.clone(), "other-package", "owners", 0),
        (5, text.clone(), "owner-test", "other_module", 0),
        (6, format!("private {t}\n{u}"), "owner-test", "owners", 0),
        (7, renamed, "owner-test", "owners", 0),
        (8, binder, "owner-test", "owners", 0),
        (9, local, "owner-test", "owners", 0),
    ] {
        let (frontend, current, entries) =
            source_registration_owner_inputs(&edited, package, module, skip);
        assert!(frontend.diagnostics.is_empty(), "edit {index}");
        assert!(entries.diagnostics().is_empty(), "edit {index}");
        let input_before = (current.clone(), entries.env().clone());
        let payloads = syntax(&current, entries.env()).unwrap();
        assert_eq!(payloads.len(), 2);
        for entry in entries
            .env()
            .symbols()
            .iter()
            .filter(|e| e.kind() == SymbolKind::Theorem)
        {
            let name = if entry.primary_spelling() == "R" {
                "T"
            } else {
                entry.primary_spelling()
            };
            let actual = &payloads[entry.symbol()];
            if index >= 7 && name == "T" {
                assert_ne!(actual, baseline[name], "edit {index}");
            } else {
                assert_eq!(actual, baseline[name], "edit {index}");
            }
            assert_eq!(entry.symbol().module(), current.module());
            assert_eq!(entry.namespace().as_str(), current.module().path().as_str());
            assert_eq!(
                entry.visibility(),
                if index == 6 && name == "T" {
                    Visibility::Private
                } else {
                    Visibility::Public
                }
            );
        }
        if index == 1 {
            assert_ne!(current.source_id(), source.source_id());
        }
        if index == 3 {
            assert!(current.arena().len() > source.arena().len());
        }
        if matches!(index, 2 | 4 | 5) {
            assert_ne!(
                payloads.keys().collect::<Vec<_>>(),
                prior.keys().collect::<Vec<_>>()
            );
        }
        assert_eq!((current, entries.env().clone()), input_before);
    }
    assert_eq!((source, collected.env().clone(), prior), before);
}

#[test]
fn source_theorem_syntax_payloads_preserve_upstream_and_refuse_unsupported_subtrees() {
    use mizar_resolve::symbols::{
        source_theorem_syntax_payloads_from_arena as syntax, validate_source_symbol_env,
    };
    use mizar_syntax::{SurfaceNodeKind, SurfaceQuantifierKind};
    let t = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let (_, source, collected) = source_registration_owner_inputs(t, "owner-test", "owners", 0);
    let prior = syntax(&source, collected.env()).unwrap();
    let before = (source.clone(), collected.env().clone(), prior.clone());
    for edited in [
        t.replace("theorem T:", "theorem Foreign:"),
        format!("{t}\n{}", t.replace("theorem T:", "theorem U:")),
    ] {
        let (frontend, current, entries) =
            source_registration_owner_inputs(&edited, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(entries.diagnostics().is_empty());
        let input_before = (current.clone(), entries.env().clone());
        let error = validate_source_symbol_env(&source, entries.env()).unwrap_err();
        assert_eq!(syntax(&source, entries.env()).unwrap_err(), error);
        let error = validate_source_symbol_env(&current, collected.env()).unwrap_err();
        assert_eq!(syntax(&current, collected.env()).unwrap_err(), error);
        assert_eq!((current, entries.env().clone()), input_before);
    }
    let (frontend, recovered, entries) = source_registration_owner_inputs(
        "theorem T: for X being set holds X = ;",
        "owner-test",
        "owners",
        0,
    );
    assert!(!frontend.diagnostics.is_empty());
    let recovered_before = (recovered.clone(), entries.env().clone());
    let error = validate_source_symbol_env(&recovered, entries.env()).unwrap_err();
    assert_eq!(syntax(&recovered, entries.env()).unwrap_err(), error);
    assert_eq!((recovered, entries.env().clone()), recovered_before);
    let existential = "theorem E: ex X being set st X = X;";
    for edited in [existential.to_owned(), format!("{t}\n{existential}")] {
        let (frontend, current, entries) =
            source_registration_owner_inputs(&edited, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(entries.diagnostics().is_empty());
        assert!(
            frontend
                .ast
                .as_ref()
                .unwrap()
                .node_views()
                .any(|node| matches!(
                    node.kind(),
                    SurfaceNodeKind::QuantifiedFormula(SurfaceQuantifierKind::Existential)
                ))
        );
        let input_before = (current.clone(), entries.env().clone());
        validate_source_symbol_env(&current, entries.env()).unwrap();
        assert_eq!(
            syntax(&current, entries.env()).unwrap_err(),
            "unsupported source theorem syntax payload"
        );
        assert_eq!((current, entries.env().clone()), input_before);
    }
    assert_eq!(syntax(&source, collected.env()).unwrap(), prior);
    assert_eq!((source, collected.env().clone(), prior), before);
}

#[test]
fn expr_type_source_projection_preserves_recursive_heads_and_actual_routes() {
    use mizar_syntax::SurfaceNodeKind as K;
    let text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/miz/pass/parser/pass_parser_expr_types_001.miz"
    ));
    let output = super::super::formula_statement::step5c8_test_frontend(text);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    let mut heads = ast
        .nodes()
        .iter()
        .filter(|n| n.kind == K::TypeHead && text[n.range.start..].starts_with("expr"))
        .collect::<Vec<_>>();
    heads.sort_by_key(|n| n.range.start);
    let spellings = heads.iter().map(|n| &text[n.range.start..n.range.end]);
    assert_eq!(
        spellings.collect::<Vec<_>>(),
        [
            "expr of object",
            "expr of expr of non empty T",
            "expr of non empty T",
            "expr of object",
            "expr of set",
            "expr of object",
            "expr of T",
            "expr of object",
            "expr of object",
        ]
    );
    for head in heads {
        assert!(!head.recovered);
        assert_eq!(head.children.len(), 3);
        for (id, word) in head.children[..2].iter().zip(["expr", "of"]) {
            let child = ast.node_view(*id).unwrap();
            let token = child.as_token().unwrap();
            assert_eq!(token.text.as_ref(), word);
            assert_eq!(token.kind, SurfaceTokenKind::ReservedWord);
        }
        let operand = ast.node(head.children[2]).unwrap();
        assert_eq!(operand.kind, K::TypeExpression);
        assert_eq!(
            operand.range,
            range(ast.source_id, head.range.start + 8, head.range.end)
        );
    }
    let attributed = ast
        .node_views()
        .find(|v| {
            v.as_type_expression().is_some()
                && &text[v.range().start..v.range().end] == "non empty T"
        })
        .unwrap();
    let kinds = attributed
        .children()
        .iter()
        .map(|id| ast.node(*id).unwrap().kind.clone())
        .collect::<Vec<_>>();
    assert_eq!(kinds, [K::AttributeChain, K::TypeHead]);
    let mut actuals = ast
        .node_views()
        .filter_map(|v| v.as_type_arguments())
        .collect::<Vec<_>>();
    actuals.sort_by_key(|v| v.range().start);
    assert_eq!(actuals.len(), 3);
    for (arguments, (spelling, kinds)) in actuals.iter().zip([
        ("of expr of object", vec![K::TypeExpression]),
        (
            "over expr of set, a",
            vec![K::TypeExpression, K::TermExpression],
        ),
        (
            "[expr of object, expr of T]",
            vec![K::TypeExpression, K::TypeExpression],
        ),
    ]) {
        assert_eq!(
            &text[arguments.range().start..arguments.range().end],
            spelling
        );
        let children = arguments
            .children()
            .iter()
            .map(|id| ast.node_view(*id).unwrap());
        let structural = children.filter(|v| v.as_token().is_none());
        assert_eq!(
            structural.map(|v| v.kind().clone()).collect::<Vec<_>>(),
            kinds
        );
    }
    let value = ast
        .node_view(*actuals[1].children().last().unwrap())
        .unwrap();
    assert_eq!(&text[value.range().start..value.range().end], "a");
    for (kind, spelling) in [
        (K::DefinitionParameter, "let input be expr of object;"),
        (
            K::AlgorithmDefinition,
            "algorithm echo(input) -> expr of object do\n    return input;\n  end;",
        ),
    ] {
        let host = ast.node_views().find(|v| v.kind() == &kind).unwrap();
        let start = text.find(spelling).unwrap();
        assert_eq!(
            host.range(),
            range(ast.source_id, start, start + spelling.len())
        );
        assert!(ast.node_views().any(|v| v.as_type_expression().is_some()
            && v.range().start > host.range().start
            && v.range().end < host.range().end));
    }
    for (source, valid) in [
        (
            "import parser.type_fixtures; reserve x for T[V qua expr of non empty object];",
            true,
        ),
        (
            "import parser.type_fixtures; reserve x for T[V qua non empty expr of object];",
            true,
        ),
        (
            "definition let U be type; pred H: h[U] means for x being expr of U holds x = x; end;",
            true,
        ),
        (
            "definition let U be type; pred H: h[U] means x is expr of U; end;",
            false,
        ),
    ] {
        let output = super::super::formula_statement::step5c8_test_frontend(source);
        assert_eq!(
            output.diagnostics.is_empty(),
            valid,
            "{source}: {:?}",
            output.diagnostics
        );
        if source.contains("qua non empty expr of object") {
            let ast = output.ast.unwrap();
            let qua = ast.node_views().find(|v| v.kind() == &K::QuaExpression).unwrap();
            let target = ast.node_view(qua.children()[2]).unwrap();
            assert_eq!(target.kind(), &K::TypeExpression);
            assert_eq!(&source[target.range().start..target.range().end], "non empty expr of object");
            assert_eq!(target.children().len(), 2);
            assert_eq!(ast.node_view(target.children()[0]).unwrap().kind(), &K::AttributeChain);
            let head = ast.node_view(target.children()[1]).unwrap();
            assert_eq!(head.kind(), &K::TypeHead);
            assert_eq!(&source[head.range().start..head.range().end], "expr of object");
            let operand = ast.node_view(head.children()[2]).unwrap();
            assert_eq!(operand.kind(), &K::TypeExpression);
            assert_eq!(&source[operand.range().start..operand.range().end], "object");
        }
    }
}

#[test]
fn expr_type_recovery_cache_storage_and_source_provenance() {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation, FrontendOutput};
    let plan = step5c11_plan();
    let cases = [
        ("pass_parser_expr_types_001", vec![]),
        ("fail_parser_expr_missing_of_001", vec![("object;", 6)]),
        (
            "fail_parser_expr_missing_target_001",
            vec![(";\nreserve nested", 1), ("];", 1)],
        ),
    ];
    for (ordinal, (id, boundaries)) in cases.iter().enumerate() {
        let case = plan.cases.iter().find(|case| case.id.0 == *id).unwrap();
        let text = std::fs::read_to_string(&case.source_path).unwrap();
        let checked = super::super::formula_statement::step5c8_test_frontend(&text);
        assert_eq!(
            checked.diagnostics.len(),
            boundaries.len(),
            "{:?}",
            checked.diagnostics
        );
        let ast = checked.ast.unwrap();
        let mut expected_missing = Vec::new();
        for (diagnostic, (boundary, width)) in checked.diagnostics.iter().zip(boundaries) {
            let code = DiagnosticCode::Syntax("malformed_type_expression".into());
            assert_eq!(diagnostic.code, code);
            let start = text.find(*boundary).unwrap();
            let location = range(ast.source_id, start, start + width);
            assert_eq!(
                diagnostic.location,
                DiagnosticLocation::SourceRange(location)
            );
            assert!(diagnostic.secondary.is_empty());
            if ordinal == 2 {
                expected_missing.push(range(ast.source_id, start, start));
                assert!(ast.token_views().any(|v| v.range() == location));
            }
        }
        let mut recovered = ast
            .node_views()
            .filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingTypeExpression))
            .map(|v| v.range())
            .collect::<Vec<_>>();
        recovered.sort_by_key(|r| r.start);
        assert_eq!(recovered, expected_missing);
        if ordinal == 1 {
            let operand = ast
                .node_views()
                .filter_map(|v| v.as_type_expression())
                .find(|v| &text[v.range().start..v.range().end] == "object")
                .unwrap();
            assert!(!operand.is_recovered());
        }
        if ordinal > 0 {
            let start = text.find("sentinel for set").unwrap();
            let sentinel = ast
                .node_views()
                .filter_map(|v| v.as_reserve_segment())
                .find(|v| v.range().start == start)
                .unwrap();
            assert_eq!(sentinel.range(), range(ast.source_id, start, start + 16));
            assert!(!sentinel.is_recovered());
        }
        assert!(ast.trivia().skipped_token_ranges().is_empty());
        let output = task257c4c1_frontend_output(&text, 90 + ordinal);
        let replay = task257c4c1_frontend_output(&text, 90 + ordinal);
        assert_eq!(output.cache_keys, replay.cache_keys);
        assert_eq!(output.ast, replay.ast);
        assert_eq!(output.diagnostics, replay.diagnostics);
        let source = &output.source;
        let input = mizar_session::SourceInput {
            package_id: source.package_id.clone(),
            module_path: source.module_path.clone(),
            normalized_path: source.normalized_path.clone(),
            edition: source.edition.clone(),
            origin: mizar_session::SourceOriginInput::Disk {
                path: source.file_path.clone(),
            },
        };
        let bytes = output.canonical_disk_bytes().unwrap();
        let same =
            FrontendOutput::from_canonical_disk_bytes(&bytes, source.source_id, &input).unwrap();
        assert_eq!(same, output);
        assert_eq!(same.ast.as_ref().unwrap().source_id, source.source_id);
        let key = output.cache_keys.ast.as_ref().unwrap();
        assert_eq!(
            key.parser_version.version.as_ref(),
            "mizar-parser/surface-ast-v30"
        );
    }
}

#[test]
fn exhaustive_retirement_preserves_match_closers_and_identifier_uses() {
    use mizar_syntax::SurfaceNodeKind as K;
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    let plan = step5c11_plan();
    for (id, diagnostics) in [
        ("pass_parser_exhaustive_identifier_001", 0),
        ("fail_parser_match_exhaustive_001", 1),
        ("fail_parser_match_exhaustive_justification_001", 1),
    ] {
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let text = std::fs::read_to_string(&case.source_path).unwrap();
        let checked = super::super::formula_statement::step5c8_test_frontend(&text);
        assert_eq!(checked.diagnostics.len(), diagnostics, "{id}: {:?}", checked.diagnostics);
        let ast = checked.ast.unwrap();
        assert_eq!(ast.node_views().filter(|v| v.kind() == &K::MatchStatement).count(), 1);
        assert!(ast.node_views().all(|v| v.kind() != &K::MatchEnding));
        if diagnostics == 0 {
            assert!(ast.node_views().all(|v| !v.is_recovered()));
            let words = ast.token_views().filter(|v| v.as_token().unwrap().text.as_ref() == "exhaustive").collect::<Vec<_>>();
            assert_eq!(words.len(), text.matches("exhaustive").count());
            assert!(words.iter().all(|v| v.as_token().unwrap().kind == SurfaceTokenKind::Identifier));
            let start = text.find("case captured").unwrap();
            let assignment = start + text[start..].find("exhaustive := seed;").unwrap();
            assert!(ast.node_views().any(|v| v.kind() == &K::AssignmentStatement && v.range().start == assignment));
        } else {
            let start = text.find("exhaustive").unwrap();
            let semicolon = start + text[start..].find(';').unwrap();
            let diagnostic = &checked.diagnostics[0];
            assert_eq!(diagnostic.code, DiagnosticCode::Syntax("malformed_formula_expression".into()));
            assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, start, start + 10)));
            assert!(diagnostic.secondary.is_empty());
            let skipped = ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken)).collect::<Vec<_>>();
            assert_eq!(skipped.len(), 1);
            assert_eq!(skipped[0].range(), range(ast.source_id, start, semicolon));
            assert!(ast.trivia().skipped_token_ranges().iter().any(|v| v.range == skipped[0].range()));
            let retained = skipped[0].children().iter().map(|child| ast.node_view(*child).unwrap().as_token().unwrap().text.as_ref()).collect::<String>();
            assert_eq!(retained, text[start..semicolon].chars().filter(|c| !c.is_whitespace()).collect::<String>());
            let matched = ast.node_views().find(|v| v.kind() == &K::MatchStatement).unwrap();
            let children = matched.children();
            assert_eq!(ast.node_view(children[children.len()-2]).unwrap().as_token().unwrap().text.as_ref(), "end");
            assert_eq!(ast.node_view(*children.last().unwrap()).unwrap().as_token().unwrap().text.as_ref(), ";");
            let closer = text.find("end;\n    return t;").unwrap();
            assert_eq!(matched.range().end, closer + 4);
            let next_return = text.rfind("return t;").unwrap();
            assert!(ast.node_views().any(|v| v.kind() == &K::ReturnStatement && v.range().start == next_return && !v.is_recovered()));
        }
    }
    let case = plan.cases.iter().find(|c| c.id.0 == "fail_parser_match_exhaustive_001").unwrap();
    let original = std::fs::read_to_string(&case.source_path).unwrap();
    let head = &original[..original.find("      exhaustive;").unwrap()];
    for (source, successor) in [
        (original.replace("exhaustive;", "another_tail;"), "another_tail"),
        (format!("{head}    return t;\n  end;\nend;\n"), "return t;"),
        (format!("{head}theorem Survivor: thesis;\n"), "theorem Survivor"),
        (head.to_owned(), ""),
    ] {
        let checked = super::super::formula_statement::step5c8_test_frontend(&source);
        let ast = checked.ast.unwrap();
        assert!(ast.node_views().all(|v| v.kind() != &K::MatchEnding));
        let syntax = checked.diagnostics.iter().filter_map(|d| match &d.code {
            DiagnosticCode::Syntax(code) => Some((code.as_ref(), d)),
            _ => None,
        }).collect::<Vec<_>>();
        if successor == "another_tail" {
            let start = source.find(successor).unwrap();
            assert_eq!(syntax.len(), 1, "{:?}", checked.diagnostics);
            assert_eq!(syntax[0].0, "malformed_formula_expression");
            assert_eq!(syntax[0].1.location, DiagnosticLocation::SourceRange(range(ast.source_id, start, start + successor.len())));
        } else {
            let boundary = if successor.is_empty() { source.trim_end().len() } else { source.rfind(successor).unwrap() };
            let opener = source.find("match t do").unwrap();
            assert!(syntax.iter().any(|(code, diagnostic)| *code == "missing_end"
                && matches!(diagnostic.location, DiagnosticLocation::SourceRange(r) if r.start == boundary)
                && diagnostic.secondary.iter().any(|anchor| matches!(anchor, mizar_session::SourceAnchor::Range(r) if r.start == opener))),
                "missing match-owned boundary diagnostic: {:?}", checked.diagnostics);
            // The prepass may also report outer blocks left unmatched by the same input.
            let item_errors = syntax.iter().filter(|(code,_)| *code == "malformed_formula_expression").collect::<Vec<_>>();
            assert_eq!(item_errors.len(), usize::from(successor == "theorem Survivor"));
            for (_, diagnostic) in item_errors {
                assert_eq!(diagnostic.message.as_ref(), "top-level items are not algorithm statements");
                assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, boundary, boundary + 7)));
            }
            assert!(syntax.iter().all(|(code,_)| matches!(*code, "missing_end" | "missing_semicolon")
                || (successor == "theorem Survivor" && *code == "malformed_formula_expression")), "{:?}", checked.diagnostics);
            if successor == "return t;" {
                assert!(ast.node_views().any(|v| v.kind() == &K::ReturnStatement && v.range().start == boundary));
            } else if successor == "theorem Survivor" {
                let matched = ast.node_views().find(|v| v.kind() == &K::MatchStatement).unwrap();
                assert!(matched.range().end <= boundary);
                let item = ast.node_views().find(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken) && v.range().start == boundary).unwrap();
                assert_eq!(item.range().end, source.trim_end().len() - 1);
                assert_eq!(item.children().iter().map(|id| ast.node_view(*id).unwrap().as_token().unwrap().text.as_ref()).collect::<String>(), "theoremSurvivor:thesis");
            }
        }
    }

    for opener in ["(", "[", "{"] {
        let unclosed = original.replace("exhaustive;", &format!("another_tail {opener};"));
        let checked = super::super::formula_statement::step5c8_test_frontend(&unclosed);
        let ast = checked.ast.unwrap();
        let next_return = unclosed.rfind("return t;").unwrap();
        assert!(ast.node_views().any(|v| v.kind() == &K::ReturnStatement && v.range().start == next_return),
            "an unclosed delimiter in invalid post-case material must not consume the successor");
        let start = unclosed.find("another_tail").unwrap();
        let semicolon = start + unclosed[start..].find(';').unwrap();
        assert!(ast.node_views().any(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken)
            && v.range() == range(ast.source_id, start, semicolon)));
    }

    for (source, boundary_kind) in [
        (original.replace("exhaustive;", "another_tail ("), "end"),
        (format!("{head}    another_tail (\n    return t;\n  end;\nend;\n"), "statement"),
        (format!("{head}    another_tail (\ntheorem Survivor: thesis;\n"), "item"),
        (format!("{head}    another_tail ("), "eof"),
    ] {
        let checked = super::super::formula_statement::step5c8_test_frontend(&source);
        let ast = checked.ast.unwrap();
        let start = source.find("another_tail").unwrap();
        let tail_end = start + "another_tail (".len();
        assert!(ast.node_views().any(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken)
            && v.range() == range(ast.source_id, start, tail_end)));
        let boundary = match boundary_kind {
            "end" => source.find("end;\n    return t;").unwrap(),
            "statement" => source.rfind("return t;").unwrap(),
            "item" => source.find("theorem Survivor").unwrap(),
            _ => source.trim_end().len(),
        };
        let matched = ast.node_views().find(|v| v.kind() == &K::MatchStatement).unwrap();
        assert!(ast.node_views().all(|v| v.kind() != &K::MatchEnding));
        assert!(checked.diagnostics.iter().any(|d| d.message.as_ref() == "unexpected token after match cases"
            && d.location == DiagnosticLocation::SourceRange(range(ast.source_id, start, start + "another_tail".len()))));
        if boundary_kind == "end" {
            assert_eq!(matched.range().end, boundary + 4);
        } else {
            let opener = source.find("match t do").unwrap();
            assert!(matched.range().end <= boundary);
            assert!(checked.diagnostics.iter().any(|d| matches!(&d.code, DiagnosticCode::Syntax(code) if code.as_ref() == "missing_end")
                && matches!(d.location, DiagnosticLocation::SourceRange(r) if r.start == boundary)
                && d.secondary.iter().any(|a| matches!(a, mizar_session::SourceAnchor::Range(r) if r.start == opener))));
        }
        if matches!(boundary_kind, "end" | "statement") {
            let successor = source.rfind("return t;").unwrap();
            assert!(ast.node_views().any(|v| v.kind() == &K::ReturnStatement && v.range().start == successor));
        } else if boundary_kind == "item" {
            assert!(ast.node_views().any(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken) && v.range().start == boundary));
        }
    }

}

#[test]
fn qua_constructor_fields_retain_outer_ownership_through_targets_and_recovery() {
    use mizar_syntax::SurfaceNodeKind as K;
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    let plan = step5c11_plan();
    for (id, diagnostics, constructors) in [
        ("pass_parser_qua_constructor_fields_001", 0, 5),
        ("fail_parser_qua_constructor_fields_recovery_001", 3, 3),
    ] {
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let text = std::fs::read_to_string(&case.source_path).unwrap();
        let output = super::super::formula_statement::step5c8_test_frontend(&text);
        assert_eq!(output.diagnostics.len(), diagnostics, "{id}: {:?}", output.diagnostics);
        let ast = output.ast.unwrap();
        let values = ast.node_views().filter(|v| v.kind() == &K::StructureConstructor).collect::<Vec<_>>();
        assert_eq!(values.len(), constructors, "{id}");
        for value in values {
            let start = value.range().start;
            assert!(text[start..].starts_with("ValueBox of"));
            let opener = start + text[start..].find("(value:").unwrap();
            let end = start + text[start..].find(';').unwrap();
            assert_eq!(value.range().end, end);
            let children = value.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
            let fields = children.iter().filter(|v| v.kind() == &K::FieldArgument).collect::<Vec<_>>();
            assert_eq!(fields.len(), 2);
            assert_eq!(fields.iter().map(|field| {
                ast.node_view(field.children()[0]).unwrap().as_token().unwrap().text.as_ref().to_owned()
            }).collect::<Vec<_>>(), ["value", "other"]);
            assert!(fields.iter().all(|field| !field.is_recovered()));
            assert!(children.iter().any(|v| v.as_token().is_some_and(|t| t.text.as_ref() == "(") && v.range().start == opener));
            assert!(children.iter().any(|v| v.as_token().is_some_and(|t| t.text.as_ref() == ")") && v.range().end == end));
            let arguments = children.iter().find(|v| v.kind() == &K::TypeArguments).unwrap();
            assert!(arguments.range().end <= opener);
        }
        let after = text.find("set after_qua_fields").unwrap();
        assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range().start == after && !v.is_recovered()));
        if diagnostics == 0 {
            assert!(ast.node_views().all(|v| !v.is_recovered()));
            assert!(ast.trivia().skipped_token_ranges().is_empty());
            assert_eq!(ast.node_views().filter(|v| v.kind() == &K::TermReference && &text[v.range().start..v.range().end] == "atom").count(), 5);
            let frontend = task257c4c1_frontend_output(&text, 123);
            let current = frontend.cache_keys.ast.as_ref().unwrap();
            assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
            let mut retained = current.clone();
            retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v14");
            assert_ne!(current.stable_hash(), retained.stable_hash());
        } else {
            let expected = [
                text.find("qua (value:").unwrap() + "qua ".len(),
                text.find("qua 7").unwrap() + "qua ".len(),
                text.find("expr of (value:").unwrap() + "expr of ".len(),
            ];
            let mut missing = ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::MissingTypeExpression)).collect::<Vec<_>>();
            missing.sort_by_key(|v| v.range().start);
            assert_eq!(missing.len(), expected.len());
            for ((recovery, start), diagnostic) in missing.iter().zip(expected).zip(&output.diagnostics) {
                assert_eq!(recovery.range(), range(ast.source_id, start, start));
                assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, start, start + 1)));
                assert!(matches!(&diagnostic.code, DiagnosticCode::Syntax(code) if code.as_ref() == "malformed_type_expression"));
            }
            let skipped = ast.node_views().filter(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken)).collect::<Vec<_>>();
            assert_eq!(skipped.len(), 1);
            assert_eq!(skipped[0].range(), range(ast.source_id, expected[1], expected[1] + 1));
            assert_eq!(ast.trivia().skipped_token_ranges().len(), 1);
            assert_eq!(ast.trivia().skipped_token_ranges()[0].range, skipped[0].range());
        }
    }
}

#[test]
fn bracket_actuals_preserve_complete_payloads_and_type_preference() {
    use mizar_syntax::{SurfaceNodeKind as K, SyntaxKind as S};
    let plan = step5c11_plan();
    for (id, names, generic, is_type) in [
        ("pass_parser_bracket_value_actuals_001", &["roots", "tails", "collections", "binders", "operators", "symbol_tails", "constructors", "qualifications", "nested_arguments"][..], "pick", false),
        ("pass_parser_bracket_type_controls_001", &["builtins", "attributes", "expressions", "nesting"][..], "typed", true),
    ] {
        let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
        let text = std::fs::read_to_string(&case.source_path).unwrap();
        let output = super::super::formula_statement::step5c8_test_frontend(&text);
        assert!(output.diagnostics.is_empty(), "{id}: {:?}", output.diagnostics);
        let ast = output.ast.unwrap();
        assert!(ast.node_views().all(|v| !v.is_recovered()));
        assert!(ast.trivia().skipped_token_ranges().is_empty());
        for name in names {
            let general_start = text.find(&format!("reserve {name} for Family[")).unwrap() + format!("reserve {name} for Family").len();
            let generic_start = text.find(&format!("set generic_{name} = {generic}[")).unwrap() + format!("set generic_{name} = {generic}").len();
            let mut payloads = Vec::new();
            for (kind, start) in [(K::TypeArguments, general_start), (K::TemplateArguments, generic_start)] {
                let list = ast.node_views().find(|v| v.kind() == &kind && v.range().start == start).unwrap();
                let children = list.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
                assert_eq!(children.first().unwrap().as_token().unwrap().text.as_ref(), "[");
                assert_eq!(children.last().unwrap().as_token().unwrap().text.as_ref(), "]");
                let actuals = children.into_iter().filter(|v| v.as_token().is_none()).map(|v| {
                    if kind == K::TemplateArguments {
                        assert_eq!(v.kind(), &K::TemplateArgument);
                        assert_eq!(v.children().len(), 1);
                        ast.node_view(v.children()[0]).unwrap()
                    } else { v }
                }).collect::<Vec<_>>();
                assert_eq!(actuals.len(), 4, "{name}: {kind:?}");
                for actual in &actuals {
                    let source = &text[actual.range().start..actual.range().end];
                    let expected = if is_type && source != "x" { K::TypeExpression } else { K::TermExpression };
                    assert_eq!(actual.kind(), &expected, "{source}");
                }
                payloads.push(actuals);
            }
            assert_eq!(payloads[0].iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), payloads[1].iter().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>());
            let expected: &[S] = match *name {
                "roots" => &[S::TermReference, S::NumeralTerm, S::ItTerm, S::ApplicationTerm],
                "tails" => &[S::SelectorAccess, S::SelectorAccess, S::StructureUpdate, S::SetEnumeration],
                "collections" => &[S::ApplicationTerm, S::ChoiceTerm, S::SetComprehension, S::BinderExpression],
                "binders" => &[S::BinderExpression, S::BinderExpression, S::QuaExpression, S::QuaExpression],
                "operators" => &[S::InfixExpression, S::PrefixExpression, S::PostfixExpression, S::InfixExpression],
                "symbol_tails" => &[S::PostfixExpression, S::SelectorAccess, S::StructureUpdate, S::QuaExpression],
                "constructors" => &[S::StructureConstructor, S::StructureConstructor, S::StructureConstructor, S::ApplicationTerm],
                "qualifications" => &[S::QuaExpression, S::QuaExpression, S::QuaExpression, S::TermReference],
                "nested_arguments" => &[S::ApplicationTerm, S::ApplicationTerm, S::ApplicationTerm, S::ApplicationTerm],
                _ => &[],
            };
            for actuals in payloads {
                for (actual, kind) in actuals.iter().zip(expected) {
                    assert_eq!(&ast.node_view(actual.children()[0]).unwrap().syntax_kind(), kind);
                }
            }
        }
        if !is_type {
            let start = text.find("ValueBox[Inner[image(x)](value: x)](value: sentinel)").unwrap();
            let outer = ast.node_views().find(|v| v.kind() == &K::StructureConstructor && v.range().start == start).unwrap();
            assert_eq!(&text[outer.range().start..outer.range().end], "ValueBox[Inner[image(x)](value: x)](value: sentinel)");
            let outer_children = outer.children().iter().map(|id| ast.node_view(*id).unwrap()).collect::<Vec<_>>();
            let outer_field = outer_children.iter().find(|v| v.kind() == &K::FieldArgument).unwrap();
            assert_eq!(&text[outer_field.range().start..outer_field.range().end], "value: sentinel");
            let args = outer_children.iter().find(|v| v.kind() == &K::TypeArguments).unwrap();
            assert_eq!(&text[args.range().start..args.range().end], "[Inner[image(x)](value: x)]");
            let term = ast.node_view(args.children()[1]).unwrap();
            let inner = ast.node_view(term.children()[0]).unwrap();
            assert_eq!(inner.kind(), &K::StructureConstructor);
            assert_eq!(&text[inner.range().start..inner.range().end], "Inner[image(x)](value: x)");
            let inner_field = inner.children().iter().map(|id| ast.node_view(*id).unwrap()).find(|v| v.kind() == &K::FieldArgument).unwrap();
            assert_eq!(&text[inner_field.range().start..inner_field.range().end], "value: x");
            assert!(inner.range().end < outer_field.range().start);
        }
        let after = text.find(if is_type { "set after_bracket_types" } else { "set after_bracket_values" }).unwrap();
        assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range().start == after && !v.is_recovered()));
        let frontend = task257c4c1_frontend_output(&text, 126);
        let current = frontend.cache_keys.ast.as_ref().unwrap();
        assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
        let mut retained = current.clone();
        retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v17");
        assert_ne!(current.stable_hash(), retained.stable_hash());
    }
}

#[test]
fn bracket_actual_recovery_retains_actuals_closers_and_outer_fields() {
    use mizar_syntax::SurfaceNodeKind as K;
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    let plan = step5c11_plan();
    let case = plan.cases.iter().find(|case| case.id.0 == "fail_parser_bracket_value_recovery_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert_eq!(output.diagnostics.len(), 6, "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    let starts = ["x |->,", "x |->,", "x ++,", "x ++,", "x qua,", "x qua,"];
    let mut cursor = 0;
    for ((prefix, diagnostic), code) in starts.into_iter().zip(&output.diagnostics).zip(["malformed_term_expression", "malformed_term_expression", "dangling_operator", "dangling_operator", "malformed_type_expression", "malformed_type_expression"]) {
        let start = cursor + text[cursor..].find(prefix).unwrap();
        cursor = start + prefix.len();
        let delimiter = cursor - 1;
        assert!(matches!(&diagnostic.code, DiagnosticCode::Syntax(actual) if actual.as_ref() == code));
        let location = if code == "dangling_operator" { range(ast.source_id, start + 2, delimiter) } else { range(ast.source_id, delimiter, delimiter + 1) };
        assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(location));
        if code != "dangling_operator" {
            let missing_kind = if code == "malformed_type_expression" { SyntaxRecoveryKind::MissingTypeExpression } else { SyntaxRecoveryKind::MissingTerm };
            assert!(ast.node_views().any(|v| v.as_recovery() == Some(missing_kind) && v.range() == range(ast.source_id, delimiter, delimiter)));
        }
    }
    for list in ast.node_views().filter(|v| matches!(v.kind(), K::TypeArguments | K::TemplateArguments)) {
        let actuals = list.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| v.as_token().is_none()).collect::<Vec<_>>();
        assert_eq!(actuals.len(), 2);
        let sentinel = actuals.last().unwrap();
        assert_eq!(&text[sentinel.range().start..sentinel.range().end], "sentinel");
        assert!(!sentinel.is_recovered());
        assert_eq!(ast.node_view(*list.children().last().unwrap()).unwrap().as_token().unwrap().text.as_ref(), "]");
    }
    assert_eq!(ast.node_views().filter(|v| v.kind() == &K::FieldArgument && !v.is_recovered()).count(), 1);
    assert_eq!(ast.node_views().filter(|v| v.kind() == &K::TermReference && &text[v.range().start..v.range().end] == "a" && !v.is_recovered()).count(), 3);
    let after = text.find("set after_bracket_recovery").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range().start == after && !v.is_recovered()));
}

#[test]
fn source_variable_type_projection_rejects_unrepresented_attribute_payloads() {
    use mizar_resolve::names::{
        SourceVariableScopeError, SourceVariableScopeInput as Input,
        SourceVariableScopeResolver as Resolver,
    };
    let declaration = "definition let n, element be set; attr TaggedDef: element is n-tagged means thesis; end;";
    for (tail, declarations) in [
        ("reserve a for tagged(0) set;", true),
        ("reserve a for tagged(0,1) set;", true),
        ("reserve a for tagged({0}) set;", true),
        ("reserve a for non tagged set;", true),
        ("reserve a for non tagged(0) set;", true),
        ("reserve a for 0-tagged set;", true),
        ("theorem Test: for a being tagged(0) set holds a = a;", true),
        ("theorem Test: for a being set holds a = a proof let a be tagged(0) set; end;", true),
        ("deffunc F(a be tagged(0) set) -> set equals a;", true),
        ("deffunc F(a be set) -> tagged(1) set equals a;", true),
        ("reserve a for set; theorem Test: a is tagged(1) set;", false),
        ("reserve a for set; theorem Test: a = a proof reconsider b = a as tagged(1) set; end;", true),
    ] {
        let text = format!("{declaration} {tail}");
        let (frontend, source, collection) =
            source_registration_owner_inputs(&text, "attribute-boundary", "main", 0);
        assert!(frontend.diagnostics.is_empty(), "{tail}: {:?}", frontend.diagnostics);
        assert!(collection.diagnostics().is_empty(), "{tail}: {:?}", collection.diagnostics());
        let ast = frontend.ast.as_ref().unwrap();
        let before = ast.snapshot_text();
        let mut results = vec![Resolver::resolve(Input::new(ast, source.module(), collection.env()))];
        if declarations {
            results.extend([
                Resolver::resolve_occurrences(Input::new(ast, source.module(), collection.env())),
                Resolver::resolve_proof_occurrences(Input::new(ast, source.module(), collection.env())),
            ]);
        }
        for result in results {
            assert_eq!(result.unwrap_err(), SourceVariableScopeError::InvalidShape, "{tail}");
        }
        assert_eq!(ast.snapshot_text(), before, "{tail}");
    }
}

#[test]
fn source_variable_type_projection_preserves_builtin_and_plain_attribute_checking() {
    use mizar_checker::type_checker::{SourceVariableSemanticsChecker, SourceVariableSemanticsInput};
    use mizar_resolve::names::{SourceVariableScopeInput as Input, SourceVariableScopeResolver as Resolver};
    let narrowing = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/miz/fail/variables/fail_type_elaboration_reconsider_unjustified_narrowing_001.miz"
    ));
    for (text, expected) in [
        ("reserve a for set; reserve b for object;", None),
        ("theorem Test: for a being set holds a = a proof let a be set; reconsider b = a as object; end;", None),
        ("deffunc F(a be set) -> set equals a;", None),
        (narrowing, Some("variables.reconsider.unjustified_narrowing")),
    ] {
        let (frontend, source, collection) =
            source_registration_owner_inputs(text, "attribute-boundary", "main", 0);
        assert!(frontend.diagnostics.is_empty(), "{text}: {:?}", frontend.diagnostics);
        assert!(collection.diagnostics().is_empty(), "{text}: {:?}", collection.diagnostics());
        let ast = frontend.ast.as_ref().unwrap();
        let before = ast.snapshot_text();
        let scope = Resolver::resolve(Input::new(ast, source.module(), collection.env())).unwrap();
        if expected.is_some() {
            assert!(scope.bindings().iter().any(|binding| binding.declared_type()
                .is_some_and(|ty| ty.attributes() == ["marked1"])));
        }
        let output = SourceVariableSemanticsChecker::check(SourceVariableSemanticsInput::new(&scope));
        assert_eq!(output.diagnostics().iter().map(|diagnostic| diagnostic.detail_key())
            .collect::<Vec<_>>(), expected.into_iter().collect::<Vec<_>>(), "{text}");
        assert!(!output.binding_types().is_empty(), "{text}");
        assert_eq!(ast.snapshot_text(), before, "{text}");
    }
}
