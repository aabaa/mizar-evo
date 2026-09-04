    use std::path::PathBuf;

    use mizar_checker::source_structure_semantics::{
        SourceStructureClaimInput, SourceStructureDefinitionInput, SourceStructureEqualityInput,
        SourceStructureMemberInput, SourceStructureMemberKind, SourceStructureProgramInput,
        SourceStructureSemanticsChecker, SourceStructureSemanticsOutput, SourceStructureTerm,
        SourceStructureType, SourceStructureVariableInput,
    };
    use mizar_core::elaborator::{
        SourceStructureCoreNormalizationError, SourceStructureCoreNormalizer,
    };
    use mizar_resolve::env::{DefinitionKind, DefinitionShell, SymbolIndex};

    #[test]
    fn task31_repository_vc_debug_is_deterministic() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("mizar-test crate should live below the workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("repository test plan should build");
        let (ordinal, case) = super::active_proof_verification_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_proof_verification_contradiction_formula_constant_001"
            })
            .expect("exact Task-180 proof-verification case");
        let first = super::proof_verification::generate_case_vc(&workspace_root, case, ordinal)
            .expect("first exact Task-180 VC generation");
        let second = super::proof_verification::generate_case_vc(&workspace_root, case, ordinal)
            .expect("second exact Task-180 VC generation");
        assert_eq!(first, second);
        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(
            first.debug_text(),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tests/snapshots/vc/pass_proof_verification_contradiction_formula_constant_001.vc_ir.snap"
            ))
        );
    }

    #[test]
    fn task31_admission_rejects_every_reserved_case_mismatch() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("repository plan");
        let exact = plan
            .cases
            .iter()
            .find(|case| {
                case.id.0 == "pass_proof_verification_contradiction_formula_constant_001"
            })
            .expect("exact Task-180 case")
            .clone();
        let type_sidecar = plan
            .cases
            .iter()
            .find(|case| {
                case.id.0 == "pass_type_elaboration_contradiction_formula_constant_001"
            })
            .expect("unchanged type sidecar");
        assert!(!super::proof_verification::is_active_proof_verification(
            type_sidecar
        ));

        let mut variants = Vec::new();
        let mut wrong_stage = exact.clone();
        wrong_stage.expectation.stage = crate::Stage::TypeElaboration;
        variants.push(wrong_stage);
        let mut missing_tag = exact.clone();
        missing_tag.expectation.tags.clear();
        variants.push(missing_tag);
        let mut duplicate_tag = exact.clone();
        duplicate_tag
            .expectation
            .tags
            .push("active_proof_verification".to_owned());
        variants.push(duplicate_tag);
        let mut wrong_tag = exact.clone();
        wrong_tag.expectation.tags = vec!["active_type_elaboration".to_owned()];
        variants.push(wrong_tag);
        let mut wrong_phase = exact.clone();
        wrong_phase.expectation.expected_phase = Some(crate::PipelinePhase::Verification);
        variants.push(wrong_phase);
        let mut wrong_outcome = exact.clone();
        wrong_outcome.expectation.expected_outcome = crate::ExpectedOutcome::Fail;
        variants.push(wrong_outcome);
        let mut absent_snapshot = exact;
        absent_snapshot.expectation.snapshots = None;
        variants.push(absent_snapshot);

        for variant in variants {
            let mut mutated = plan.clone();
            mutated.cases = vec![variant];
            let diagnostics =
                super::proof_verification::validate_active_proof_verification_tags(
                    &workspace_root,
                    &mutated,
                );
            assert_eq!(
                diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic.code.0 == "E-PROOF-VERIFICATION-ACTIVE-GATE"
                    })
                    .count(),
                1,
                "{:#?}",
                mutated.cases[0]
            );
            assert_eq!(super::active_proof_verification_cases(&mutated).count(), 0);
        }
    }

    #[test]
    fn step5c2_proof_admission_and_inventory_fail_closed() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let mut plan = build_test_plan(&config).expect("repository plan");
        let (id, source) = super::proof_verification::STEP5C2_PROOF_CASES[0];
        let exact = plan
            .cases
            .iter()
            .find(|case| case.id.0 == id)
            .expect("first Step 5C.2 proof case")
            .clone();
        assert!(super::proof_verification::is_active_proof_verification(
            &exact
        ));

        let mut duplicate_tag = exact.clone();
        duplicate_tag
            .expectation
            .tags
            .push("active_proof_verification".to_owned());
        assert!(!super::proof_verification::is_active_proof_verification(
            &duplicate_tag
        ));

        let mut unexpected_snapshot = exact.clone();
        unexpected_snapshot.expectation.snapshots = Some(PathBuf::from("unexpected.snap"));
        assert!(!super::proof_verification::is_active_proof_verification(
            &unexpected_snapshot
        ));

        let mut alias = exact.clone();
        alias.source_path = workspace_root.join(format!("alias/{source}"));
        assert!(alias.source_path.ends_with(source));
        plan.cases.push(alias);
        let diagnostics = super::proof_verification::validate_active_proof_verification_tags(
            &workspace_root,
            &plan,
        );
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code.0 == "E-PROOF-VERIFICATION-ACTIVE-GATE"
        }));

        let duplicate = plan
            .cases
            .iter()
            .find(|case| case.id.0 == super::proof_verification::STEP5C2_PROOF_CASES[1].0)
            .expect("second Step 5C.2 proof case")
            .clone();
        plan.cases.push(duplicate);
        let diagnostics = super::proof_verification::validate_active_proof_verification_tags(
            &workspace_root,
            &plan,
        );
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code.0 == "E-PROOF-VERIFICATION-STEP5C2-INVENTORY"
        }));

        let public_code_case = plan
            .cases
            .iter_mut()
            .find(|case| case.id.0 == id)
            .expect("Step 5C.2 proof case in mutable plan");
        public_code_case
            .expectation
            .diagnostic_codes
            .push("E-FORBIDDEN".to_owned());
        let diagnostics = super::proof_verification::validate_active_proof_verification_tags(
            &workspace_root,
            &plan,
        );
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code.0 == "E-PROOF-VERIFICATION-PUBLIC-DIAGNOSTIC-CODES"
        }));
    }

    #[test]
    fn task31_snapshot_failures_and_report_projection_fail_closed() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("repository plan");
        let (ordinal, case) = super::active_proof_verification_cases(&plan)
            .enumerate()
            .find(|(_, case)| {
                case.id.0 == "pass_proof_verification_contradiction_formula_constant_001"
            })
            .expect("exact Task-180 case");
        let vc = super::proof_verification::generate_case_vc(&workspace_root, case, ordinal)
            .expect("exact VC");
        let mut absent = case.clone();
        absent.expectation.snapshots = None;
        let absent_result = super::proof_verification::run_proof_verification_case(
            &workspace_root,
            &workspace_root.join("tests"),
            &absent,
            ordinal,
        );
        assert_eq!(
            absent_result.status,
            super::ProofVerificationCaseStatus::Failed
        );
        assert!(
            absent_result
                .failure
                .as_deref()
                .is_some_and(|failure| failure.contains("absent"))
        );

        static NEXT_VC31_TEMP_ID: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let temp = std::env::temp_dir().join(format!(
            "mizar-test-vc31-snapshot-{}-{}",
            std::process::id(),
            NEXT_VC31_TEMP_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&temp).expect("temp directory");
        let mut missing = case.clone();
        missing.expectation.snapshots = Some(PathBuf::from("missing.snap"));
        let missing_result = super::proof_verification::run_proof_verification_case(
            &workspace_root,
            &temp,
            &missing,
            ordinal,
        );
        assert_eq!(
            missing_result.status,
            super::ProofVerificationCaseStatus::Failed
        );
        assert!(
            missing_result
                .failure
                .as_deref()
                .is_some_and(|failure| failure.contains("could not read"))
        );
        std::fs::create_dir_all(temp.join("unreadable.snap")).expect("unreadable directory");
        let mut unreadable = case.clone();
        unreadable.expectation.snapshots = Some(PathBuf::from("unreadable.snap"));
        let unreadable_result = super::proof_verification::run_proof_verification_case(
            &workspace_root,
            &temp,
            &unreadable,
            ordinal,
        );
        assert_eq!(
            unreadable_result.status,
            super::ProofVerificationCaseStatus::Failed
        );
        std::fs::write(temp.join("mismatch.snap"), "wrong\n").expect("mismatch baseline");
        let mut mismatch_plan = plan.clone();
        let mismatch_case = mismatch_plan
            .cases
            .iter_mut()
            .find(|candidate| candidate.id == case.id)
            .expect("exact case in cloned plan");
        mismatch_case.expectation.snapshots = Some(PathBuf::from("mismatch.snap"));
        let mismatch_report = super::run_proof_verification_plan(
            &workspace_root,
            &temp,
            &mismatch_plan,
        );
        assert_eq!(mismatch_report.results.len(), 3);
        let mismatch_result = mismatch_report
            .results
            .iter()
            .find(|result| result.id == case.id)
            .expect("exact Task-180 result");
        assert_eq!(
            mismatch_result.status,
            super::ProofVerificationCaseStatus::Failed
        );
        assert!(
            mismatch_result
                .failure
                .as_deref()
                .is_some_and(|failure| failure.contains("differed"))
        );
        assert!(mismatch_report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code.0 == "E-PROOF-VERIFICATION-CASE"
                && diagnostic.detail_key
                    == "proof_verification.pass_proof_verification_contradiction_formula_constant_001"
        }));
        assert_eq!(vc.vcs().len(), 1);
        std::fs::remove_dir_all(temp).expect("remove temp directory");
    }

    #[test]
    fn repository_proof_verification_report_preserves_task180_and_adds_step5c2() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let report = super::run_proof_verification_corpus(&config).expect("proof report");
        assert_eq!(report.results.len(), 3);
        assert_eq!(report.passed_count(), 3);
        assert_eq!(report.failed_count(), 0);
        assert_eq!(report.error_count(), 0, "{:#?}", report.diagnostics);
    }

    #[test]
    fn step5c2_structure_receipts_are_deterministic_and_have_no_residual_vcs() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf();
        let config = DiscoveryConfig {
            workspace_root: workspace_root.clone(),
            tests_root: workspace_root.join("tests"),
            manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        };
        let plan = build_test_plan(&config).expect("repository plan");

        for (id, _) in super::proof_verification::STEP5C2_PROOF_CASES {
            let (ordinal, case) = super::active_proof_verification_cases(&plan)
                .enumerate()
                .find(|(_, case)| case.id.0 == id)
                .expect("exact Step 5C.2 proof case");
            let first = super::proof_verification::normalize_structure_case(
                &workspace_root,
                case,
                ordinal,
            )
            .expect("first structure normalization");
            let second = super::proof_verification::normalize_structure_case(
                &workspace_root,
                case,
                ordinal,
            )
            .expect("second structure normalization");
            assert_eq!(first, second);
            assert_eq!(first.equality_count(), 2);
            assert_eq!(first.residual_vc_count(), 0);
            assert!(first.has_zero_residual_vcs());
        }
    }

    #[test]
    fn step5c2_public_normalizer_rejects_all_frozen_error_classes() {
        let source = normalizer_source();
        let pair = normalizer_definition(
            source,
            "Pair",
            0,
            Some(("value", SourceStructureType::Set)),
        );
        let diagnostic = normalizer_output(
            source,
            vec![pair.clone()],
            &[pair],
            vec![],
            vec![SourceStructureTerm::Constructor {
                structure: normalizer_symbol("Pair"),
                type_arguments: vec![],
                arguments: vec![],
                source_range: normalizer_range(source, 60, 70),
                source_ordinal: 0,
                recovered: false,
            }],
            vec![],
        );
        assert_eq!(
            SourceStructureCoreNormalizer::normalize(&diagnostic),
            Err(SourceStructureCoreNormalizationError::DiagnosticBearingOutput)
        );

        let external = normalizer_definition(source, "External", 0, None);
        let boxed = normalizer_definition(
            source,
            "Box",
            1,
            Some((
                "value",
                SourceStructureType::Structure {
                    symbol: normalizer_symbol("External"),
                    arguments: vec![],
                },
            )),
        );
        let malformed = normalizer_output(
            source,
            vec![boxed.clone()],
            &[external, boxed],
            vec![normalizer_variable(source, "a", 0)],
            vec![],
            vec![normalizer_claim(source, "a", "a")],
        );
        assert_eq!(
            SourceStructureCoreNormalizer::normalize(&malformed),
            Err(SourceStructureCoreNormalizationError::MalformedOutput)
        );

        let marker = normalizer_definition(source, "Marker", 0, None);
        let unequal = normalizer_output(
            source,
            vec![marker.clone()],
            &[marker],
            vec![
                normalizer_variable(source, "a", 0),
                normalizer_variable(source, "b", 1),
            ],
            vec![],
            vec![normalizer_claim(source, "a", "b")],
        );
        assert_eq!(
            SourceStructureCoreNormalizer::normalize(&unequal),
            Err(SourceStructureCoreNormalizationError::UnequalEquality)
        );

        let unsupported = normalizer_output(source, vec![], &[], vec![], vec![], vec![]);
        assert_eq!(
            SourceStructureCoreNormalizer::normalize(&unsupported),
            Err(SourceStructureCoreNormalizationError::UnsupportedShape)
        );
    }

    fn normalizer_source() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "7a".repeat(32)
        ))
        .expect("snapshot");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source")
    }

    fn normalizer_module() -> ResolverModuleId {
        ResolverModuleId::new(PackageId::new("mizar-test"), ModulePath::new("normalizer"))
    }

    fn normalizer_symbol(name: &str) -> ResolverSymbolId {
        let module = normalizer_module();
        ResolverSymbolId::new(
            module.clone(),
            LocalSymbolId::new(name),
            FullyQualifiedName::new(format!("{}::{name}", module.path().as_str())),
        )
    }

    fn normalizer_variable_symbol(name: &str) -> ResolverSymbolId {
        let module = normalizer_module();
        ResolverSymbolId::new(
            module.clone(),
            LocalSymbolId::new(format!("step5c2/variable/{name}")),
            FullyQualifiedName::new(format!(
                "{}::step5c2::variable::{name}",
                module.path().as_str()
            )),
        )
    }

    const fn normalizer_range(source: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id: source,
            start,
            end,
        }
    }

    fn normalizer_definition(
        source: SourceId,
        name: &str,
        ordinal: usize,
        member: Option<(&str, SourceStructureType)>,
    ) -> SourceStructureDefinitionInput {
        let start = ordinal * 40;
        let members = member.into_iter().map(|(member, ty)| {
            SourceStructureMemberInput::new(
                normalizer_symbol(&format!("{name}/{member}")),
                member,
                member,
                SourceStructureMemberKind::Field,
                ty,
                normalizer_range(source, start + 10, start + 11),
                0,
                false,
            )
        });
        SourceStructureDefinitionInput::new(
            normalizer_symbol(name),
            name,
            vec![],
            members.collect(),
            normalizer_range(source, start, start + 30),
            ordinal,
            false,
        )
    }

    fn normalizer_env(
        source: SourceId,
        definitions: &[SourceStructureDefinitionInput],
    ) -> SymbolEnv {
        let module = normalizer_module();
        let mut symbols = SymbolIndex::new();
        let mut definitions_index = mizar_resolve::env::DefinitionIndex::new();
        let mut contributions = mizar_resolve::env::SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(normalizer_range(source, 0, 1)),
        );
        for definition in definitions {
            let origin = SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(definition.source_range()),
                vec![definition.source_ordinal() as u32],
            );
            symbols.insert(SymbolEntry::new(
                definition.symbol().clone(),
                SymbolKind::Structure,
                NamespacePath::new(module.path().as_str()),
                definition.spelling(),
                origin.clone(),
                contribution,
            ));
            definitions_index.insert(DefinitionShell::new(
                definition.symbol().clone(),
                DefinitionKind::Structure,
                origin,
                contribution,
            ));
            for member in definition.members() {
                let origin = SemanticOrigin::new(
                    source,
                    module.clone(),
                    SourceAnchor::Range(member.source_range()),
                    vec![
                        definition.source_ordinal() as u32,
                        member.source_ordinal() as u32,
                    ],
                );
                symbols.insert(SymbolEntry::new(
                    member.symbol().clone(),
                    SymbolKind::Selector,
                    NamespacePath::new(module.path().as_str()),
                    member.resolver_spelling(),
                    origin.clone(),
                    contribution,
                ));
                definitions_index.insert(DefinitionShell::new(
                    member.symbol().clone(),
                    DefinitionKind::Selector,
                    origin,
                    contribution,
                ));
            }
        }
        SymbolEnv::new(
            module,
            SymbolEnvIndexes {
                symbols,
                definitions: definitions_index,
                contributions,
                ..SymbolEnvIndexes::default()
            },
        )
    }

    fn normalizer_variable(
        source: SourceId,
        name: &str,
        ordinal: usize,
    ) -> SourceStructureVariableInput {
        SourceStructureVariableInput::new(
            normalizer_variable_symbol(name),
            name,
            SourceStructureType::Set,
            normalizer_range(source, 80 + ordinal, 81 + ordinal),
            ordinal,
            false,
        )
    }

    fn normalizer_term(
        source: SourceId,
        name: &str,
        start: usize,
        ordinal: usize,
    ) -> SourceStructureTerm {
        SourceStructureTerm::Variable {
            symbol: normalizer_variable_symbol(name),
            spelling: name.to_owned(),
            source_range: normalizer_range(source, start, start + 1),
            source_ordinal: ordinal,
            recovered: false,
        }
    }

    fn normalizer_claim(source: SourceId, left: &str, right: &str) -> SourceStructureClaimInput {
        let proposition = SourceStructureEqualityInput::new(
            normalizer_term(source, left, 110, 0),
            normalizer_term(source, right, 112, 1),
            normalizer_range(source, 108, 114),
            0,
            false,
        );
        let conclusion = SourceStructureEqualityInput::new(
            normalizer_term(source, left, 130, 2),
            normalizer_term(source, left, 132, 3),
            normalizer_range(source, 128, 134),
            1,
            false,
        );
        SourceStructureClaimInput::new(
            proposition,
            vec![conclusion],
            normalizer_range(source, 100, 140),
            0,
            false,
        )
    }

    fn normalizer_output(
        source: SourceId,
        definitions: Vec<SourceStructureDefinitionInput>,
        resolver_definitions: &[SourceStructureDefinitionInput],
        variables: Vec<SourceStructureVariableInput>,
        terms: Vec<SourceStructureTerm>,
        claims: Vec<SourceStructureClaimInput>,
    ) -> SourceStructureSemanticsOutput {
        SourceStructureSemanticsChecker::check(
            SourceStructureProgramInput::new(
                source,
                normalizer_module(),
                definitions,
                vec![],
                variables,
                terms,
                claims,
            ),
            &normalizer_env(source, resolver_definitions),
        )
        .expect("checker output")
    }
