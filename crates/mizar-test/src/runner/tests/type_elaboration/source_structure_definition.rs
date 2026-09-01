use super::type_elaboration::{
    SOURCE_STRUCTURE_DEFINITION_TEXT, SourceStructureDefinitionRouteMutation,
    source_structure_definition_output, source_structure_definition_output_with_mutation,
};

const TASK263_CASE: &str = "pass_type_elaboration_structure_definition_payload_001";
const TASK263_SPEC_REF: &str =
    "spec.en.checker.type_elaboration.source_structure_definition_payload";

#[test]
fn task263_structure_definition_source_surface_and_resolver_are_exact() {
    assert_eq!(SOURCE_STRUCTURE_DEFINITION_TEXT.len(), 320);
    assert_eq!(SOURCE_STRUCTURE_DEFINITION_TEXT.lines().count(), 16);
    assert!(SOURCE_STRUCTURE_DEFINITION_TEXT.ends_with('\n'));
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_STRUCTURE_DEFINITION_TEXT, 263_000);
    assert_eq!(ast.nodes().len(), 75);
    assert_eq!(ast.root().map(|id| id.index()), Some(74));
    assert!(ast.expression_root().is_none());
    assert_eq!(shells.declarations().len(), 10);
    assert_eq!(symbols.symbols().len(), 8);
    assert_eq!(symbols.definitions().len(), 8);
    assert_eq!(symbols.contributions().len(), 1);
    let output = source_structure_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_STRUCTURE_DEFINITION_TEXT,
    )
    .expect("Task263 exact route")
    .expect("Task263 exact payload");
    let structure = output
        .typed_ast
        .source_structure_definition()
        .expect("Task263 typed handoff");
    assert_eq!(structure.definitions().len(), 2);
    assert_eq!(structure.members().len(), 4);
    assert_eq!(structure.inheritances().len(), 1);
    assert_eq!(structure.mappings().len(), 2);
    assert!(structure.coherence_requests().is_empty());
    assert_eq!(
        output.typed_ast.source_structure_definition(),
        output.resolved.source_structure_definition()
    );
}

#[test]
fn task263_structure_definition_lower_payload_and_subtree_corruption_fail_closed() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_STRUCTURE_DEFINITION_TEXT, 263_001);
    let mut mutations = Vec::new();
    mutations.extend(
        (0..75).map(SourceStructureDefinitionRouteMutation::WrongSurfaceRow),
    );
    mutations.extend([
        SourceStructureDefinitionRouteMutation::WrongSurfaceRange(57),
        SourceStructureDefinitionRouteMutation::WrongSurfaceRecovery(65),
        SourceStructureDefinitionRouteMutation::WrongSurfaceChildren(70),
        SourceStructureDefinitionRouteMutation::WrongSurfaceRoot,
    ]);
    mutations.extend((0..10).map(SourceStructureDefinitionRouteMutation::WrongShellRow));
    mutations.extend((0..8).map(SourceStructureDefinitionRouteMutation::WrongProjectionRow));
    mutations.extend([
        SourceStructureDefinitionRouteMutation::WrongResolverDefinition,
        SourceStructureDefinitionRouteMutation::WrongLowerMember,
        SourceStructureDefinitionRouteMutation::WrongDefinition,
        SourceStructureDefinitionRouteMutation::WrongMapping,
        SourceStructureDefinitionRouteMutation::WrongArena,
    ]);
    for mutation in mutations {
        assert!(
            source_structure_definition_output_with_mutation(
                &ast,
                module.clone(),
                &shells,
                &symbols,
                SOURCE_STRUCTURE_DEFINITION_TEXT,
                mutation,
            )
            .expect("Task263 exact source selection")
            .is_err(),
            "mutation {mutation:?} must fail closed"
        );
    }
    let without_final_lf = SOURCE_STRUCTURE_DEFINITION_TEXT.trim_end_matches('\n');
    let (wrong_ast, wrong_module, wrong_shells, wrong_symbols) =
        task253_ast_from_source_text(without_final_lf, 263_002);
    assert!(
        source_structure_definition_output(
            &wrong_ast,
            wrong_module,
            &wrong_shells,
            &wrong_symbols,
            without_final_lf,
        )
        .is_none()
    );
}

#[test]
fn task263_structure_definition_selection_trace_and_family_isolation_are_exact() {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap()
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task263 repository plan");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK263_CASE)
        .expect("Task263 active case");
    assert_eq!(case.expectation.spec_refs.len(), 1);
    assert_eq!(case.expectation.spec_refs[0].0, TASK263_SPEC_REF);
    assert!(case.expectation.diagnostic_codes.is_empty());
    assert!(case.expectation.diagnostic_payloads.is_empty());
    let requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|row| row.id.0 == TASK263_SPEC_REF)
        .expect("Task263 trace row");
    assert!(requirement.required);
    assert_eq!(
        requirement.status,
        crate::traceability::RequirementStatus::Covered
    );
    assert_eq!(
        requirement.coverage,
        crate::traceability::CoverageShape::Pass
    );
    assert_eq!(requirement.tests.len(), 1);
    let result = run_type_elaboration_case(
        &workspace_root,
        &workspace_root.join("tests"),
        case,
        ordinal,
    );
    assert_eq!(result.status, TypeElaborationCaseStatus::Passed);
    assert!(result.actual_detail_keys.is_empty());

    for (source, source_ordinal) in [
        (super::type_elaboration::SOURCE_PREDICATE_DEFINITION_TEXT, 263_010),
        (super::type_elaboration::SOURCE_FUNCTOR_DEFINITION_TEXT, 263_011),
        (super::type_elaboration::SOURCE_ATTRIBUTE_DEFINITION_TEXT, 263_012),
        (super::type_elaboration::SOURCE_MODE_DEFINITION_TEXT, 263_013),
    ] {
        let (ast, module, shells, symbols) =
            task253_ast_from_source_text(source, source_ordinal);
        assert!(
            source_structure_definition_output(&ast, module, &shells, &symbols, source).is_none()
        );
    }

    let gap = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_type_elaboration_mode_structure_definition_gap_001")
        .expect("mixed mode/structure gap remains");
    let gap_source = std::fs::read_to_string(&gap.source_path).unwrap();
    let (gap_ast, gap_module, gap_shells, gap_symbols) =
        task253_ast_from_source_text(&gap_source, 263_014);
    assert!(
        source_structure_definition_output(
            &gap_ast,
            gap_module,
            &gap_shells,
            &gap_symbols,
            &gap_source,
        )
        .is_none()
    );
}

#[test]
fn task263_structure_definition_semantic_deferrals_are_not_published() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_STRUCTURE_DEFINITION_TEXT, 263_020);
    let output = source_structure_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_STRUCTURE_DEFINITION_TEXT,
    )
    .unwrap()
    .unwrap();
    let typed = &output.typed_ast;
    let resolved = &output.resolved;
    assert!(typed.source_context().is_none());
    assert!(typed.initial_obligations().is_empty());
    assert!(typed.facts().is_empty());
    assert!(typed.coercions().is_empty());
    assert!(typed.diagnostics().is_empty());
    assert!(typed.source_predicate_definition().is_none());
    assert!(typed.source_functor_definition().is_none());
    assert!(typed.source_attribute_definition().is_none());
    assert!(typed.source_mode_definition().is_none());
    assert!(resolved.checked_formulas().is_empty());
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.cluster_facts().is_empty());
    assert!(resolved.diagnostics().is_empty());
    let debug = typed
        .source_structure_definition()
        .unwrap()
        .debug_text();
    for prohibited in ["goal=", "guard=", "proof=", "accepted=", "fact=", "axiom=", "core", "control-flow", "vc="] {
        assert!(!debug.contains(prohibited), "published prohibited semantic field {prohibited}");
    }
}

#[test]
fn task263_structure_definition_core_item_context_association_and_local_dependency_are_exact() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_STRUCTURE_DEFINITION_TEXT, 263_200);
    let output = source_structure_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_STRUCTURE_DEFINITION_TEXT,
    )
    .expect("Task263 selector")
    .expect("Task263 route");
    let checker_owner = output
        .typed_ast
        .source_structure_definition()
        .expect("Task263 checker owner")
        .clone();
    let context = task263_core_context(&checker_owner);
    let expected_context = context.clone();
    let first = mizar_core::elaborator::SourceStructureCoreContextProducer::build(
        context.clone(),
        checker_owner.clone(),
    )
    .expect("Task263 Core item context");
    let second = mizar_core::elaborator::SourceStructureCoreContextProducer::build(
        context,
        checker_owner.clone(),
    )
    .expect("Task263 deterministic replay");
    assert_eq!(first, second);
    assert_eq!(first.source_id(), checker_owner.source_id());
    assert_eq!(first.module_id(), checker_owner.module_id());
    assert_eq!(first.context(), &expected_context);
    assert_eq!(first.checker_owner(), &checker_owner);
    assert_eq!(first.items().len(), 2);
    assert!(!first.items().is_empty());
    assert_eq!(first.context().item_registry().items().len(), 2);
    assert!(first.context().dependency_summaries().is_empty());
    assert!(first.context().generated_origins().table().is_empty());
    assert!(first.context().diagnostics().is_empty());

    let definitions = [
        checker_owner
            .definitions()
            .get(mizar_checker::source_structure_definition::SourceStructureDefinitionId::new(0))
            .expect("Task263 base definition"),
        checker_owner
            .definitions()
            .get(mizar_checker::source_structure_definition::SourceStructureDefinitionId::new(1))
            .expect("Task263 derived definition"),
    ];
    let core_items = definitions.map(|definition| {
        first
            .context()
            .item_registry()
            .id_for_symbol(definition.symbol())
            .expect("Task263 Core structure item")
    });
    let inheritance = checker_owner
        .inheritances()
        .get(
            mizar_checker::source_structure_definition::SourceStructureInheritanceId::new(0),
        )
        .expect("Task263 direct inheritance");
    assert_eq!(inheritance.child(), definitions[1].id());
    assert_eq!(inheritance.parent(), definitions[0].id());
    assert_ne!(core_items[0], core_items[1]);
    assert_eq!(
        first
            .items()
            .iter()
            .map(|(id, row)| (id, row.symbol().clone(), row.core_item()))
            .collect::<Vec<_>>(),
        vec![
            (
                definitions[0].id(),
                definitions[0].symbol().clone(),
                core_items[0],
            ),
            (
                definitions[1].id(),
                definitions[1].symbol().clone(),
                core_items[1],
            ),
        ]
    );

    for index in 0..2 {
        let association = first
            .items()
            .get(definitions[index].id())
            .expect("Task263 association");
        assert_eq!(association.definition(), definitions[index].id());
        assert_eq!(association.symbol(), definitions[index].symbol());
        assert_eq!(association.core_item(), core_items[index]);
        let item = first
            .context()
            .item_registry()
            .items()
            .get(core_items[index])
            .expect("Task263 Core item row");
        assert_eq!(item.symbol, *definitions[index].symbol());
        assert_eq!(item.kind, mizar_core::core_ir::CoreItemKind::Structure);
        assert_eq!(item.visibility.as_str(), "public");
        assert_eq!(item.status, mizar_core::core_ir::CoreItemStatus::Valid);
        assert!(item.diagnostics.is_empty());
        assert_eq!(
            item.source.anchor,
            mizar_core::core_ir::CoreSourceAnchor::SourceRange(
                definitions[index].source_range()
            )
        );
        assert_eq!(
            item.source.provenance,
            vec![mizar_core::core_ir::CoreProvenance::new(
                mizar_core::core_ir::CoreProvenancePhase::Checker,
                format!("source-structure-core-item-v1.definition.{index}"),
            )]
        );
        let expected_local: &[mizar_core::core_ir::CoreItemId] = if index == 0 {
            &[]
        } else {
            &core_items[..1]
        };
        assert_eq!(item.dependencies, expected_local);
        let dependency = first
            .context()
            .item_registry()
            .dependencies(core_items[index])
            .expect("Task263 dependency resolution");
        assert_eq!(dependency.local, expected_local);
        assert!(dependency.external.is_empty());
        assert!(dependency.missing.is_empty());
        let boundary = first
            .context()
            .definition_boundaries()
            .get_by_item(core_items[index])
            .expect("Task263 pending boundary");
        assert_eq!(boundary.item, core_items[index]);
        assert_eq!(boundary.symbol, *definitions[index].symbol());
        assert_eq!(
            boundary.kind,
            mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem
        );
        assert_eq!(
            boundary.status,
            mizar_core::elaborator::DefinitionBoundaryStatus::PendingBody
        );
        assert_eq!(boundary.source, item.source);
        assert_eq!(
            boundary.provenance.as_slice(),
            &[mizar_core::core_ir::CoreProvenance::new(
                mizar_core::core_ir::CoreProvenancePhase::Checker,
                format!("source-structure-core-item-v1.definition.{index}"),
            )]
        );
        assert_eq!(
            first
                .context()
                .source_map()
                .item_sources
                .get(&core_items[index]),
            Some(&item.source)
        );
    }
    assert_eq!(first.context().source_map().item_sources.len(), 2);
    assert!(first.context().source_map().term_sources.is_empty());
    assert!(first.context().source_map().formula_sources.is_empty());
    assert!(first.context().source_map().definition_sources.is_empty());
    assert!(first.context().source_map().proof_sources.is_empty());
    assert!(first.context().source_map().algorithm_sources.is_empty());
    assert!(first.context().source_map().generated_sources.is_empty());
    assert!(first.context().source_map().obligation_sources.is_empty());
    assert_eq!(
        first.context().worklist().entries(),
        core_items
            .iter()
            .map(|core_item| {
                let item = first
                    .context()
                    .item_registry()
                    .items()
                    .get(*core_item)
                    .expect("Task263 worklist item source");
                mizar_core::elaborator::ElaborationWorkItem {
                    kind: mizar_core::elaborator::ElaborationWorkItemKind::Item(*core_item),
                    status: mizar_core::elaborator::ElaborationWorkStatus::Pending,
                    source: item.source.clone(),
                    diagnostics: Vec::new(),
                    checker_diagnostics: Vec::new(),
                }
            })
            .collect::<Vec<_>>()
    );
}

#[test]
fn task263_structure_definition_core_item_context_mutations_and_foreign_environment_fail_closed() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_STRUCTURE_DEFINITION_TEXT, 263_210);
    let output = source_structure_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_STRUCTURE_DEFINITION_TEXT,
    )
    .expect("Task263 selector")
    .expect("Task263 route");
    let checker_owner = output
        .typed_ast
        .source_structure_definition()
        .expect("Task263 checker owner")
        .clone();
    for mutation in [
        Task263CoreContextMutation::MissingItem,
        Task263CoreContextMutation::ExtraItem,
        Task263CoreContextMutation::WrongKind,
        Task263CoreContextMutation::WrongVisibility,
        Task263CoreContextMutation::WrongSymbol,
        Task263CoreContextMutation::WrongSource,
        Task263CoreContextMutation::WrongProvenance,
        Task263CoreContextMutation::WrongBoundaryProvenance,
        Task263CoreContextMutation::MissingBoundary,
        Task263CoreContextMutation::WrongBoundary,
        Task263CoreContextMutation::MissingDerivedDependency,
        Task263CoreContextMutation::WrongDerivedDependency,
        Task263CoreContextMutation::UnexpectedBaseDependency,
        Task263CoreContextMutation::InvalidStatus,
        Task263CoreContextMutation::UnexpectedGeneratedOrigin,
        Task263CoreContextMutation::UnexpectedCheckerSite,
        Task263CoreContextMutation::UnexpectedBinder,
    ] {
        let context = task263_core_context_with_mutation(&checker_owner, mutation);
        let error = mizar_core::elaborator::SourceStructureCoreContextProducer::build(
            context,
            checker_owner.clone(),
        )
        .expect_err("Task263 Core mutation must fail closed");
        assert_eq!(
            error,
            mizar_core::elaborator::SourceStructureCoreContextError::InvalidCoreContext,
            "{mutation:?}"
        );
    }

    let (foreign_ast, foreign_module, foreign_shells, foreign_symbols) =
        task253_ast_from_source_text(SOURCE_STRUCTURE_DEFINITION_TEXT, 263_211);
    let foreign_output = source_structure_definition_output(
        &foreign_ast,
        foreign_module,
        &foreign_shells,
        &foreign_symbols,
        SOURCE_STRUCTURE_DEFINITION_TEXT,
    )
    .expect("foreign Task263 selector")
    .expect("foreign Task263 route");
    let foreign_owner = foreign_output
        .typed_ast
        .source_structure_definition()
        .expect("foreign Task263 checker owner")
        .clone();
    for (context, owner) in [
        (task263_core_context(&checker_owner), foreign_owner.clone()),
        (task263_core_context(&foreign_owner), checker_owner.clone()),
    ] {
        assert_eq!(
            mizar_core::elaborator::SourceStructureCoreContextProducer::build(context, owner)
                .expect_err("foreign Task263 environment must fail closed"),
            mizar_core::elaborator::SourceStructureCoreContextError::EnvironmentMismatch
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Task263CoreContextMutation {
    Baseline,
    MissingItem,
    ExtraItem,
    WrongKind,
    WrongVisibility,
    WrongSymbol,
    WrongSource,
    WrongProvenance,
    WrongBoundaryProvenance,
    MissingBoundary,
    WrongBoundary,
    MissingDerivedDependency,
    WrongDerivedDependency,
    UnexpectedBaseDependency,
    InvalidStatus,
    UnexpectedGeneratedOrigin,
    UnexpectedCheckerSite,
    UnexpectedBinder,
}

fn task263_core_context(
    checker_owner: &mizar_checker::source_structure_definition::SourceStructureDefinitionHandoff,
) -> mizar_core::elaborator::CoreContext {
    task263_core_context_with_mutation(checker_owner, Task263CoreContextMutation::Baseline)
}

fn task263_core_context_with_mutation(
    checker_owner: &mizar_checker::source_structure_definition::SourceStructureDefinitionHandoff,
    mutation: Task263CoreContextMutation,
) -> mizar_core::elaborator::CoreContext {
    let definitions = [
        checker_owner
            .definitions()
            .get(mizar_checker::source_structure_definition::SourceStructureDefinitionId::new(0))
            .expect("Task263 base definition"),
        checker_owner
            .definitions()
            .get(mizar_checker::source_structure_definition::SourceStructureDefinitionId::new(1))
            .expect("Task263 derived definition"),
    ];
    let inheritance = checker_owner
        .inheritances()
        .get(
            mizar_checker::source_structure_definition::SourceStructureInheritanceId::new(0),
        )
        .expect("Task263 direct inheritance");
    assert_eq!(inheritance.child(), definitions[1].id());
    assert_eq!(inheritance.parent(), definitions[0].id());
    let child_index = inheritance.child().index();
    let parent_index = inheritance.parent().index();
    let summary = mizar_core::elaborator::ResolvedTypedAstSummary::new(
        checker_owner.source_id(),
        checker_owner.module_id().clone(),
    );
    let summary = if mutation == Task263CoreContextMutation::UnexpectedCheckerSite {
        summary.with_checker_sites(vec![
            mizar_core::elaborator::CheckerSiteSummary::failed_overload(
                mizar_checker::resolved_typed_ast::OverloadResolutionId::new(0),
                mizar_core::core_ir::CoreSourceRef::direct(definitions[0].source_range()),
            ),
        ])
    } else {
        summary
    };
    let mut input = mizar_core::elaborator::CoreContextInput::new(
        summary,
    );
    for index in 0..2 {
        if mutation == Task263CoreContextMutation::MissingItem && index == 1 {
            continue;
        }
        let definition = definitions[index];
        let expected_provenance_key =
            format!("source-structure-core-item-v1.definition.{index}");
        let source_provenance_key = if mutation == Task263CoreContextMutation::WrongProvenance
            && index == parent_index
        {
            "wrong-task263-provenance".to_owned()
        } else {
            expected_provenance_key.clone()
        };
        let boundary_provenance_key =
            if mutation == Task263CoreContextMutation::WrongBoundaryProvenance
                && index == parent_index
            {
                "wrong-task263-boundary-provenance".to_owned()
            } else {
                expected_provenance_key
            };
        let source_range = if mutation == Task263CoreContextMutation::WrongSource && index == 0 {
            mizar_session::SourceRange {
                source_id: checker_owner.source_id(),
                start: definition.source_range().start + 1,
                end: definition.source_range().end,
            }
        } else {
            definition.source_range()
        };
        let source = mizar_core::core_ir::CoreSourceRef::direct(source_range).with_provenance(vec![
            mizar_core::core_ir::CoreProvenance::new(
                mizar_core::core_ir::CoreProvenancePhase::Checker,
                source_provenance_key,
            ),
        ]);
        let kind = if mutation == Task263CoreContextMutation::WrongKind && index == 0 {
            mizar_core::core_ir::CoreItemKind::Mode
        } else {
            mizar_core::core_ir::CoreItemKind::Structure
        };
        let visibility = if mutation == Task263CoreContextMutation::WrongVisibility && index == 0 {
            "private"
        } else {
            "public"
        };
        let seed_symbol = if mutation == Task263CoreContextMutation::WrongSymbol
            && index == parent_index
        {
            task263_extra_symbol(checker_owner, "task263-wrong")
        } else {
            definition.symbol().clone()
        };
        let mut seed = mizar_core::elaborator::CoreItemSeed::new(
            seed_symbol,
            kind,
            visibility,
            source,
            mizar_core::elaborator::CheckerOwnedProvenance::checker(
                boundary_provenance_key,
            ),
        );
        if !(mutation == Task263CoreContextMutation::MissingBoundary && index == 0) {
            seed = seed.with_definition_boundary(
                if mutation == Task263CoreContextMutation::WrongBoundary && index == 0 {
                    mizar_core::elaborator::DefinitionBoundaryKind::Theorem
                } else {
                    mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem
                },
            );
        }
        let dependencies = if index == parent_index {
            if mutation == Task263CoreContextMutation::UnexpectedBaseDependency {
                vec![definitions[child_index].symbol().clone()]
            } else {
                Vec::new()
            }
        } else if mutation == Task263CoreContextMutation::MissingDerivedDependency {
            Vec::new()
        } else if mutation == Task263CoreContextMutation::WrongDerivedDependency {
            vec![definitions[child_index].symbol().clone()]
        } else if mutation == Task263CoreContextMutation::InvalidStatus {
            vec![task263_extra_symbol(checker_owner, "task263-missing")]
        } else {
            vec![definitions[parent_index].symbol().clone()]
        };
        input.item_seeds.push(seed.with_dependencies(dependencies));
    }
    if mutation == Task263CoreContextMutation::ExtraItem {
        let symbol = task263_extra_symbol(checker_owner, "task263-extra");
        input.item_seeds.push(
            mizar_core::elaborator::CoreItemSeed::new(
                symbol,
                mizar_core::core_ir::CoreItemKind::Structure,
                "public",
                mizar_core::core_ir::CoreSourceRef::direct(definitions[1].source_range())
                    .with_provenance(vec![mizar_core::core_ir::CoreProvenance::new(
                        mizar_core::core_ir::CoreProvenancePhase::Checker,
                        "source-structure-core-item-v1.definition.extra",
                    )]),
                mizar_core::elaborator::CheckerOwnedProvenance::checker(
                    "source-structure-core-item-v1.definition.extra",
                ),
            )
            .with_definition_boundary(
                mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem,
            ),
        );
    }
    if mutation == Task263CoreContextMutation::UnexpectedGeneratedOrigin {
        input.generated_origin_seeds.push(
            mizar_core::elaborator::GeneratedOriginSeed::new(
                definitions[parent_index].symbol().clone(),
                mizar_core::core_ir::GeneratedOriginKind::StableChoice,
                "task263-unexpected-generated-origin",
                mizar_core::core_ir::CoreSourceRef::direct(
                    definitions[parent_index].source_range(),
                ),
                mizar_core::elaborator::CheckerOwnedProvenance::checker(
                    "task263-unexpected-generated-origin",
                ),
            ),
        );
    }
    if mutation == Task263CoreContextMutation::UnexpectedBinder {
        let var = mizar_core::core_ir::CoreVarId::new(0);
        input
            .variable_seeds
            .push(mizar_core::elaborator::CoreVariableSeed::new(
                var,
                mizar_core::binder_normalization::NormalizedVarClass::Free,
                "task263-unexpected-binder",
                mizar_core::binder_normalization::NormalizedVarSort::Term,
                mizar_core::elaborator::CheckerOwnedProvenance::checker(
                    "task263-unexpected-binder",
                ),
            ));
        input
            .binder_seeds
            .push(mizar_core::elaborator::CoreBinderSeed::new(
                var,
                mizar_core::core_ir::CoreSourceRef::direct(
                    definitions[parent_index].source_range(),
                ),
                mizar_core::elaborator::CheckerOwnedProvenance::checker(
                    "task263-unexpected-binder",
                ),
            ));
    }
    mizar_core::elaborator::prepare_core_context(input)
        .expect("Task263 Core context seed should prepare")
}

fn task263_extra_symbol(
    checker_owner: &mizar_checker::source_structure_definition::SourceStructureDefinitionHandoff,
    local: &str,
) -> mizar_resolve::resolved_ast::SymbolId {
    mizar_resolve::resolved_ast::SymbolId::new(
        checker_owner.module_id().clone(),
        mizar_resolve::resolved_ast::LocalSymbolId::new(local),
        mizar_resolve::resolved_ast::FullyQualifiedName::new(format!(
            "{}.{}",
            checker_owner.module_id().path().as_str(),
            local,
        )),
    )
}
