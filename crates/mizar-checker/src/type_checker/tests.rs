use super::*;
use crate::binding_env::{
    BinderIdentity, BindingContextDraft, BindingContextOwner, BindingContextTable,
    BindingDiagnosticTable, BindingDraft, BindingEnvParts, BindingRecoveryState, BindingTable,
    BindingTypeSite, CapturedFreeVariables,
};
use crate::typed_ast::{TypeRole, TypedNodeId};
use mizar_resolve::{
    env::{
        ContributionKind, DeclarationConflictClass, DefinitionIndex, DefinitionShell, ExportStatus,
        LabelIndex, ModuleLexicalSummaryIndex, ModuleSummaryIndex, NamespaceGraph, NamespacePath,
        OverloadIndex, RegistrationIndex, ResolvedExportIndex, ResolvedImportIndex,
        SourceContributionIndex, SymbolEntry, SymbolEnvIndexes, SymbolIndex, Visibility,
    },
    names::{
        LocalTermScope, SourceVariableBindingKind, SourceVariableFormula, SourceVariableScopeInput,
        SourceVariableScopeResolver, SourceVariableStatement,
    },
    resolved_ast::{FullyQualifiedName, LocalSymbolId, ModuleId, SemanticOrigin},
};
use mizar_session::{
    BuildSnapshotId, GeneratedSpanAnchor, GeneratedSpanOrigin, InMemorySessionIdAllocator,
    ModulePath, PackageId, SessionIdAllocator, SourceAnchor, SourceId,
};
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/source_variable_semantics_unit.rs"
));

fn source_nodes_typed(
    source: &SurfaceResolvedArena,
    edit: impl FnOnce(&mut Vec<crate::typed_ast::TypedNode>),
) -> crate::typed_ast::TypedAst {
    use crate::typed_ast::{TypedArena, TypedAst, TypedAstParts, TypedNode};
    let mut nodes = source
        .arena()
        .iter()
        .map(|(id, node)| {
            TypedNode::new("semantic-kind", node.origin().anchor().clone())
                .with_resolved_node(id)
                .with_children(
                    node.children()
                        .iter()
                        .map(|id| TypedNodeId::new(id.index()))
                        .collect(),
                )
                .with_recovery(if node.origin().is_recovered() {
                    NodeRecoveryState::Recovered
                } else {
                    NodeRecoveryState::Normal
                })
        })
        .collect();
    edit(&mut nodes);
    TypedAst::try_new(TypedAstParts {
        source_id: source.source_id(),
        module_id: source.module().clone(),
        resolved_root: Some(source.arena().root()),
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: TypedArena::try_new(Some(TypedNodeId::new(source.arena().root().index())), nodes)
            .unwrap(),
        contexts: LocalTypeContextTable::default(),
        types: TypeTable::default(),
        facts: TypeFactTable::default(),
        coercions: CoercionTable::default(),
        initial_obligations: InitialObligationTable::default(),
        diagnostics: TypeDiagnosticTable::default(),
    })
    .unwrap()
}

#[test]
fn generic_source_nodes_transport_every_class_a_kind_without_semantic_admission() {
    use mizar_syntax as syntax;
    let kinds = vec![
        K::AnnotatedAlgorithmStatement,
        K::AnnotatedDefinitionContent,
        K::AnnotatedRegistrationContent,
        K::AnnotatedStatement,
        K::Annotation,
        K::AnnotationArgument,
        K::AnnotationArgumentList,
        K::AnnotationLabel,
        K::AnnotationLabelList,
        K::BulkReference,
        K::ConditionList,
        K::ExportItem,
        K::GroupedReference,
        K::GroupedReferenceItem,
        K::ImportAliasDecl,
        K::ImportItem,
        K::LibraryAnnotation,
        K::ModuleBranchImport,
        K::ModulePath,
        K::NamespacePath,
        K::ParameterPrefix,
        K::PlaceholderItem,
        K::PostfixExpression(syntax::ast::SurfacePostfixOperator {
            spelling: "!".into(),
            precedence: 1,
        }),
        K::ProofHintOption,
        K::ProofHintOptionList,
        K::QualifiedReference,
        K::RelativePrefix,
        K::StandaloneDiagnosticAnnotation,
        K::TermList,
        K::TermPlaceholder,
        K::TypedParameter,
    ];
    let source = source_id();
    let mut builder = SurfaceAstBuilder::new(source);
    let token = builder.add_token(SurfaceTokenKind::Identifier, "actual", range(source, 3, 9));
    let mut child = token;
    for kind in &kinds {
        child = builder.add_node(kind.clone(), range(source, 3, 9), vec![child]);
    }
    // The lexical root entry aliases the token owned by the structural subtree.
    let root = builder.add_node(K::Root, range(source, 3, 9), vec![token, child]);
    let ast = builder.finish(Some(root), None);
    let source = SurfaceResolvedArena::lower(&ast, &module_id()).unwrap();
    let mut typed = source_nodes_typed(&source, |_| {});
    let debug = typed.debug_text();
    typed.install_source_nodes(&source).unwrap();
    let rows = typed.source_nodes().unwrap();
    assert_eq!(rows.len(), kinds.len() + 2);
    assert_eq!(rows[0].site, source.arena().root());
    assert_eq!(rows[0].parent, None);
    assert_eq!(rows[1].child_ordinal, 1);
    for (ordinal, row) in rows.iter().enumerate() {
        assert_eq!(row.preorder_ordinal, ordinal);
        assert_eq!(row.range, range(source.source_id(), 3, 9));
        assert_eq!(row.spelling, "actual");
        assert_eq!(row.recovery, NodeRecoveryState::Normal);
        if ordinal > 0 {
            assert_eq!(row.parent, Some(rows[ordinal - 1].site));
        }
    }
    assert_eq!(
        rows[1..rows.len() - 1]
            .iter()
            .map(|row| &row.kind)
            .collect::<Vec<_>>(),
        kinds.iter().rev().collect::<Vec<_>>()
    );
    assert!(typed.source_statement().is_none());
    assert_eq!(typed.debug_text(), debug);
    let installed = typed.clone();
    assert!(typed.install_source_nodes(&source).is_err());
    assert_eq!(typed, installed);
}

#[test]
fn generic_source_nodes_keep_actual_root_order_token_spelling_and_recovery() {
    use mizar_syntax as syntax;
    let source = source_id();
    let mut builder = SurfaceAstBuilder::new(source);
    // Allocation and root order intentionally differ from lexical order.
    let late =
        builder.add_recovered_token(SurfaceTokenKind::Identifier, "late", range(source, 12, 16));
    let early = builder.add_token(SurfaceTokenKind::Identifier, "early", range(source, 2, 7));
    let structural = builder.add_node(K::Annotation, range(source, 2, 7), vec![early]);
    let recovery = builder.add_recovery(
        syntax::SyntaxRecoveryKind::MissingItem,
        range(source, 17, 17),
        vec![],
    );
    let root = builder.add_node(
        K::Root,
        range(source, 0, 18),
        vec![late, early, structural, recovery],
    );
    let ast = builder.finish(Some(root), None);
    let source = SurfaceResolvedArena::lower(&ast, &module_id()).unwrap();
    let mut typed = source_nodes_typed(&source, |_| {});
    typed.install_source_nodes(&source).unwrap();
    let rows = typed.source_nodes().unwrap();
    assert_eq!(
        rows.iter().map(|row| row.site.index()).collect::<Vec<_>>(),
        [4, 0, 2, 1, 3]
    );
    assert_eq!(rows[0].spelling, "early late");
    assert_eq!(rows[1].spelling, "late");
    assert_eq!(rows[1].parent, Some(rows[0].site));
    assert_eq!(rows[1].child_ordinal, 0);
    assert_eq!(rows[1].recovery, NodeRecoveryState::Recovered);
    assert_eq!(rows[2].child_ordinal, 2);
    assert_eq!(rows[3].parent, Some(rows[2].site));
    assert_eq!(rows[4].child_ordinal, 3);
    assert_eq!(rows[4].recovery, NodeRecoveryState::Recovered);
    assert_eq!(rows[4].spelling, "");
    assert!(typed.facts().is_empty());
    assert!(typed.initial_obligations().is_empty());
}

#[test]
fn generic_source_nodes_refuse_malformed_handoffs_atomically() {
    let source = source_id();
    for fault in ["reversed", "overlap", "outside", "duplicate", "unreachable"] {
        let mut builder = SurfaceAstBuilder::new(source);
        let first = builder.add_token(SurfaceTokenKind::Identifier, "first", range(source, 2, 7));
        let second = builder.add_token(
            SurfaceTokenKind::Identifier,
            "second",
            range(source, if fault == "overlap" { 6 } else { 9 }, 15),
        );
        let mut children = vec![first, second];
        if fault == "reversed" {
            children.reverse();
        }
        let structural = builder.add_node(
            K::TermList,
            range(source, 0, if fault == "outside" { 14 } else { 16 }),
            children,
        );
        let mut children = vec![structural];
        if fault == "duplicate" {
            children.push(structural);
        }
        if fault == "unreachable" {
            builder.add_node(K::Annotation, range(source, 1, 2), vec![]);
        }
        let root = builder.add_node(K::Root, range(source, 0, 16), children);
        let ast = builder.finish(Some(root), None);
        let source = SurfaceResolvedArena::lower(&ast, &module_id()).unwrap();
        let mut typed = source_nodes_typed(&source, |_| {});
        let before = typed.clone();
        assert!(typed.install_source_nodes(&source).is_err(), "{fault}");
        assert_eq!(typed, before, "{fault}");
        assert!(typed.source_nodes().is_none());
    }
}

#[test]
fn generic_source_nodes_refuse_foreign_typed_associations_atomically() {
    let source_id = source_id();
    let ast = source_variable_type_projection_ast(source_id);
    let source = SurfaceResolvedArena::lower(&ast, &module_id()).unwrap();
    for fault in ["resolved", "anchor", "children", "recovery", "count"] {
        let mut typed = source_nodes_typed(&source, |nodes| match fault {
            "resolved" => nodes[0].resolved_node = nodes[1].resolved_node,
            "anchor" => nodes[0].anchor = SourceAnchor::Range(range(source_id, 1, 2)),
            "children" => nodes.last_mut().unwrap().children.clear(),
            "recovery" => nodes[0].recovery = NodeRecoveryState::Recovered,
            "count" => nodes.push(nodes[0].clone()),
            _ => unreachable!(),
        });
        let before = typed.clone();
        assert!(typed.install_source_nodes(&source).is_err(), "{fault}");
        assert_eq!(typed, before, "{fault}");
    }
    let other_module = ModuleId::new(module_id().package().clone(), ModulePath::new("foreign"));
    let foreign = SurfaceResolvedArena::lower(&ast, &other_module).unwrap();
    let mut typed = source_nodes_typed(&source, |_| {});
    let before = typed.clone();
    assert!(typed.install_source_nodes(&foreign).is_err());
    assert_eq!(typed, before);
    let foreign_ast = source_variable_type_projection_ast(source_ids_pair().1);
    let foreign = SurfaceResolvedArena::lower(&foreign_ast, &module_id()).unwrap();
    assert!(typed.install_source_nodes(&foreign).is_err());
    assert_eq!(typed, before);
}

#[test]
fn generic_source_nodes_install_with_original_statement_after_canonical_comparison() {
    let source_id = source_id();
    let build = |recovered| {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let reserve = source_variable_token(
            &mut builder,
            source_id,
            SurfaceTokenKind::ReservedWord,
            "reserve",
            0,
        );
        let x = source_variable_token(
            &mut builder,
            source_id,
            SurfaceTokenKind::Identifier,
            "x",
            8,
        );
        let for_kw = source_variable_token(
            &mut builder,
            source_id,
            SurfaceTokenKind::ReservedWord,
            "for",
            10,
        );
        let set = source_variable_type(&mut builder, source_id, 14, "set");
        let segment = builder.add_node(
            K::ReserveSegment,
            range(source_id, 8, 17),
            vec![x, for_kw, set],
        );
        let semi = source_variable_token(
            &mut builder,
            source_id,
            SurfaceTokenKind::ReservedSymbol,
            ";",
            17,
        );
        let reserve = builder.add_node(
            K::ReserveItem,
            range(source_id, 0, 18),
            vec![reserve, segment, semi],
        );
        let theorem = source_variable_token(
            &mut builder,
            source_id,
            SurfaceTokenKind::ReservedWord,
            "theorem",
            19,
        );
        let label = if recovered {
            builder.add_recovered_token(SurfaceTokenKind::Identifier, "T", range(source_id, 27, 28))
        } else {
            source_variable_token(
                &mut builder,
                source_id,
                SurfaceTokenKind::Identifier,
                "T",
                27,
            )
        };
        let colon = source_variable_token(
            &mut builder,
            source_id,
            SurfaceTokenKind::ReservedSymbol,
            ":",
            28,
        );
        let equality = source_variable_equality(&mut builder, source_id, 30, "x", "x");
        let semi = source_variable_token(
            &mut builder,
            source_id,
            SurfaceTokenKind::ReservedSymbol,
            ";",
            35,
        );
        let theorem = builder.add_node(
            K::TheoremItem,
            range(source_id, 19, 36),
            vec![theorem, label, colon, equality, semi],
        );
        source_variable_finish(builder, source_id, 0, 37, vec![reserve, theorem])
    };
    let ast = build(false);
    let (source, symbols) =
        mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module_id())
            .unwrap()
            .unwrap();
    let (scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&source, &symbols).unwrap();
    let (_, _, base) =
        TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope).unwrap();
    assert!(base.source_nodes().is_none());
    let recovered = SurfaceResolvedArena::lower(&build(true), &module_id()).unwrap();
    assert_eq!(
        base.clone()
            .with_source_statement_from_original(&recovered, &symbols, &scope),
        Err(crate::typed_ast::TypedAstError::InvalidSourceStatement)
    );
    let installed = base
        .clone()
        .with_source_statement_from_original(&source, &symbols, &scope)
        .unwrap();
    assert_eq!(
        installed.source_nodes().unwrap(),
        source_nodes(&source, &base).unwrap()
    );
    assert!(installed.source_statement().is_some());
    let mut preinstalled = base.clone();
    preinstalled.install_source_nodes(&source).unwrap();
    assert_eq!(
        preinstalled.with_source_statement_from_original(&source, &symbols, &scope),
        Err(crate::typed_ast::TypedAstError::InvalidSourceStatement)
    );
    assert!(base.source_nodes().is_none());
}

#[test]
fn declarations_attach_types_and_context_snapshots_are_deterministic() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let binding_env = binding_env_for_declarations(
        source,
        vec![
            binding_spec("let_x", BindingKind::LetBinding, BindingStatus::Active),
            binding_spec(
                "reserved_y",
                BindingKind::ReservedVariable,
                BindingStatus::Reserved,
            ),
            binding_spec(
                "quant_z",
                BindingKind::QuantifierBinder,
                BindingStatus::Active,
            ),
            binding_spec(
                "given_w",
                BindingKind::QuantifierBinder,
                BindingStatus::Active,
            ),
            binding_spec(
                "set_a",
                BindingKind::LocalAbbreviation,
                BindingStatus::Active,
            ),
            binding_spec(
                "param_p",
                BindingKind::DefinitionParameter,
                BindingStatus::Active,
            ),
            binding_spec(
                "deffunc_arg",
                BindingKind::DefinitionParameter,
                BindingStatus::Active,
            ),
            binding_spec(
                "defpred_arg",
                BindingKind::DefinitionParameter,
                BindingStatus::Active,
            ),
            binding_spec(
                "reconsider_old",
                BindingKind::LetBinding,
                BindingStatus::Active,
            ),
            binding_spec(
                "reconsider_new",
                BindingKind::Generated,
                BindingStatus::Active,
            ),
            binding_spec(
                "consider_c",
                BindingKind::QuantifierBinder,
                BindingStatus::Active,
            ),
            binding_spec(
                "take_t",
                BindingKind::QuantifierBinder,
                BindingStatus::Active,
            ),
        ],
    );
    let contexts = vec![
        DeclarationContextInput::new(BindingContextId::new(0), site(200), range(source, 0, 1)),
        DeclarationContextInput::new(BindingContextId::new(1), site(201), range(source, 1, 2)),
    ];
    let declarations = vec![
        declaration_with_type(source, 0, DeclarationKind::Let, 10, 100),
        declaration_with_type_in_context(
            source,
            1,
            BindingContextId::new(0),
            DeclarationKind::ReservedVariable,
            11,
            101,
        )
        .with_reserved_default(ReservedDefaultPayload::new(site(101), false))
        .with_assumptions(vec![DeclarationAssumptionInput::new(
            TypePredicateRef::new("reserved_y_is_set"),
            range(source, 11, 12),
        )]),
        declaration_with_type(source, 2, DeclarationKind::QuantifiedVariable, 12, 102),
        declaration_with_type(source, 3, DeclarationKind::Given, 13, 103).with_assumptions(vec![
            DeclarationAssumptionInput::new(
                TypePredicateRef::new("given_w_is_set"),
                range(source, 13, 14),
            ),
        ]),
        declaration_with_type(source, 4, DeclarationKind::Set, 14, 104)
            .with_deferred(vec![DeclarationDeferredReason::MissingRightHandSidePayload]),
        declaration_with_type(source, 5, DeclarationKind::DefinitionParameter, 15, 105),
        declaration_with_type(source, 6, DeclarationKind::DefFuncFormal, 16, 106).with_deferred(
            vec![DeclarationDeferredReason::MissingDefinitionBodyPayload],
        ),
        declaration_with_type(source, 7, DeclarationKind::DefPredFormal, 17, 107).with_deferred(
            vec![DeclarationDeferredReason::MissingDefinitionBodyPayload],
        ),
        declaration_with_type(source, 8, DeclarationKind::ReconsiderExisting, 18, 108),
        declaration_with_type(source, 9, DeclarationKind::ReconsiderNew, 19, 109),
        declaration_with_type(source, 10, DeclarationKind::Consider, 20, 110),
        declaration_with_type(source, 11, DeclarationKind::Take, 21, 111),
    ];

    let first = DeclarationChecker::default().check(
        &symbols,
        &binding_env,
        contexts.clone(),
        declarations.clone(),
    );
    let second = DeclarationChecker::default().check(
        &symbols,
        &binding_env,
        contexts.into_iter().rev(),
        declarations.into_iter().rev(),
    );

    assert_eq!(first.debug_text(), second.debug_text());
    assert_eq!(first.declarations().len(), 12);
    assert_eq!(first.contexts().len(), 2);
    assert_eq!(first.facts().len(), 2);
    let declarations_by_binding = declarations_by_binding(&first);
    for binding_index in 0..12 {
        let declaration = declarations_by_binding
            .get(&BindingId::new(binding_index))
            .copied()
            .unwrap();
        assert_eq!(declaration.binding, BindingId::new(binding_index));
        assert_eq!(declaration.site, site(10 + binding_index));
        assert_eq!(declaration.type_site, Some(site(100 + binding_index)));
        let type_entry = first
            .type_entries()
            .get(declaration.type_entry.unwrap())
            .unwrap();
        assert_eq!(type_entry.owner, declaration.site);
        assert!(matches!(type_entry.actual, TypeEntryActual::Known(_)));
        if matches!(
            declaration.kind,
            DeclarationKind::Set | DeclarationKind::DefFuncFormal | DeclarationKind::DefPredFormal
        ) {
            assert_eq!(declaration.status, DeclarationStatus::Partial);
            assert_eq!(type_entry.status, TypeStatus::Unknown);
        } else {
            assert_eq!(declaration.status, DeclarationStatus::Checked);
            assert_eq!(type_entry.status, TypeStatus::Known);
        }
    }
    assert_eq!(
        declarations_by_binding
            .get(&BindingId::new(1))
            .unwrap()
            .facts
            .len(),
        1
    );
    assert_eq!(
        declarations_by_binding
            .get(&BindingId::new(3))
            .unwrap()
            .facts
            .len(),
        1
    );
    let module_context = first.contexts().get(LocalTypeContextId::new(0)).unwrap();
    let block_context = first.contexts().get(LocalTypeContextId::new(1)).unwrap();
    assert_eq!(module_context.parent, None);
    assert_eq!(block_context.parent, Some(LocalTypeContextId::new(0)));
    assert_eq!(
        module_context.bindings,
        vec![BindingTypeRef::Site(site(11))]
    );
    assert_eq!(
        module_context.introduced_assumptions,
        vec![TypeFactId::new(0)]
    );
    assert_eq!(module_context.visible_facts, vec![TypeFactId::new(0)]);
    assert_eq!(
        block_context.introduced_assumptions,
        vec![TypeFactId::new(1)]
    );
    assert_eq!(
        block_context.visible_facts,
        vec![TypeFactId::new(0), TypeFactId::new(1)]
    );
    assert_eq!(
        block_context.bindings,
        vec![
            BindingTypeRef::Site(site(10)),
            BindingTypeRef::Site(site(12)),
            BindingTypeRef::Site(site(13)),
            BindingTypeRef::Site(site(14)),
            BindingTypeRef::Site(site(15)),
            BindingTypeRef::Site(site(16)),
            BindingTypeRef::Site(site(17)),
            BindingTypeRef::Site(site(18)),
            BindingTypeRef::Site(site(19)),
            BindingTypeRef::Site(site(20)),
            BindingTypeRef::Site(site(21)),
        ]
    );
    let module_fact = first.facts().get(TypeFactId::new(0)).unwrap();
    assert_eq!(module_fact.subject, site(11));
    assert_eq!(module_fact.predicate.as_str(), "reserved_y_is_set");
    assert_eq!(module_fact.polarity, Polarity::Positive);
    assert_eq!(module_fact.status, FactStatus::Assumed);
    assert!(matches!(module_fact.provenance, FactProvenance::Assumed(_)));
    let block_fact = first.facts().get(TypeFactId::new(1)).unwrap();
    assert_eq!(block_fact.subject, site(13));
    assert_eq!(block_fact.predicate.as_str(), "given_w_is_set");
    assert_eq!(block_fact.polarity, Polarity::Positive);
    assert_eq!(block_fact.status, FactStatus::Assumed);
    assert!(matches!(block_fact.provenance, FactProvenance::Assumed(_)));
    assert!(first.debug_text().contains("kind=let"));
    assert!(first.debug_text().contains("kind=reserved_variable"));
    assert!(first.debug_text().contains("kind=quantified_variable"));
    assert!(first.debug_text().contains("kind=given"));
    assert!(first.debug_text().contains("kind=consider"));
    assert!(first.debug_text().contains("kind=take"));
    assert!(first.debug_text().contains("kind=set"));
    assert!(first.debug_text().contains("kind=definition_parameter"));
    assert!(first.debug_text().contains("kind=deffunc_formal"));
    assert!(first.debug_text().contains("kind=defpred_formal"));
    assert!(first.debug_text().contains("kind=reconsider_existing"));
    assert!(first.debug_text().contains("kind=reconsider_new"));
    assert!(
        first
            .debug_text()
            .contains("checker.declaration.deferred.rhs_payload")
    );
    assert!(
        first
            .debug_text()
            .contains("checker.declaration.deferred.definition_body_payload")
    );
    assert_eq!(
        diagnostic_ranges(&first, "checker.declaration.deferred.rhs_payload"),
        vec![(70, 75)]
    );
    assert_eq!(
        diagnostic_ranges(
            &first,
            "checker.declaration.deferred.definition_body_payload"
        ),
        vec![(80, 85), (85, 90)]
    );
    assert!(!first.debug_text().contains("obligation#"));
    assert!(!first.debug_text().contains("initial_obligation"));
    assert!(!first.debug_text().contains(concat!("V", "cId")));
    assert!(!first.debug_text().contains(concat!("Proof", "Witness")));
    assert!(
        !first
            .debug_text()
            .contains(concat!("active", "_refinement"))
    );
}

#[test]
fn source_reserve_declaration_bridge_builds_checker_owned_handoff() {
    let source = source_id();
    let module = module_id();
    let symbols = symbol_env(Vec::new());
    let bridge = SourceReserveDeclarationBridge::new(
        source,
        module,
        range(source, 0, 40),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 8, 9),
                range(source, 17, 20),
                "set",
                TypeHeadInput::BuiltinSet,
            ),
            SourceReserveBindingInput::new(
                "y",
                range(source, 11, 12),
                range(source, 17, 20),
                "set",
                TypeHeadInput::BuiltinSet,
            ),
            SourceReserveBindingInput::new(
                "z",
                range(source, 29, 30),
                range(source, 35, 41),
                "object",
                TypeHeadInput::BuiltinObject,
            ),
        ],
    )
    .expect("reserve bridge inputs should validate");

    let handoff = bridge
        .check(&symbols)
        .expect("checker-owned source reserve handoff should assemble");
    let binding_env = handoff.binding_env();
    let declarations = handoff.declarations();
    let module_context = binding_env
        .contexts()
        .get(bridge.module_context())
        .expect("module context should exist");

    assert_eq!(
        module_context.bindings,
        vec![BindingId::new(0), BindingId::new(1), BindingId::new(2)]
    );
    assert_eq!(module_context.visible_bindings, module_context.bindings);
    assert_eq!(binding_env.bindings().len(), 3);
    assert_eq!(declarations.declarations().len(), 3);
    assert_eq!(declarations.contexts().len(), 1);
    assert!(declarations.diagnostics().is_empty());
    assert_eq!(bridge.root_node(), TypedNodeId::new(6));
    assert_ne!(bridge.type_node(0), bridge.type_node(1));

    let checked = declarations_by_binding(declarations);
    for (index, source_binding) in bridge.bindings().iter().enumerate() {
        let binding_id = BindingId::new(index);
        let binding = binding_env
            .bindings()
            .get(binding_id)
            .expect("reserve binding should exist");
        assert_eq!(binding.spelling, source_binding.spelling);
        assert_eq!(binding.kind, BindingKind::ReservedVariable);
        assert_eq!(binding.owner_context, bridge.module_context());
        assert_eq!(
            binding.type_site,
            BindingTypeSite::Source(source_binding.type_range)
        );
        assert_eq!(binding.status, BindingStatus::Reserved);
        assert_eq!(binding.visible_after_ordinal, index);
        assert!(matches!(
            &binding.identity,
            BinderIdentity::ReservedVariable {
                spelling,
                declaration_range,
            } if spelling == &source_binding.spelling
                && *declaration_range == source_binding.binding_range
        ));

        let declaration = checked
            .get(&binding_id)
            .expect("checked declaration should exist");
        assert_eq!(declaration.kind, DeclarationKind::ReservedVariable);
        assert_eq!(declaration.status, DeclarationStatus::Checked);
        assert_eq!(
            declaration.site,
            TypedSiteRef::Node(bridge.declaration_node(index))
        );
        assert_eq!(
            declaration.type_site,
            Some(TypedSiteRef::Node(bridge.type_node(index)))
        );
        assert!(declaration.type_entry.is_some());
    }
}

#[test]
fn source_reserve_declaration_bridge_preserves_attributed_builtin_evidence_gap() {
    let source = source_id();
    let module = module_id();
    let attr = symbol_id("empty/0", "pkg::main::empty/0");
    let symbols = symbol_env(vec![symbol_entry(attr.clone(), SymbolKind::Attribute)]);
    let bridge = SourceReserveDeclarationBridge::new(
        source,
        module,
        range(source, 0, 30),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 8, 9),
                range(source, 14, 27),
                "non empty set",
                TypeHeadInput::BuiltinSet,
            )
            .with_type_attributes(vec![AttributeInput::new(
                attr.clone(),
                AttributePolarity::Negative,
                range(source, 14, 23),
                "non empty",
            )]),
        ],
    )
    .expect("attributed builtin reserve payload should validate");

    let handoff = bridge
        .check(&symbols)
        .expect("checker-owned attributed reserve handoff should assemble");
    let declaration = declarations_by_binding(handoff.declarations())
        .remove(&BindingId::new(0))
        .expect("checked declaration should exist");
    assert_eq!(declaration.status, DeclarationStatus::Partial);
    assert_eq!(
        declaration.deferred,
        vec![DeclarationDeferredReason::MissingEvidenceQuery]
    );
    let type_entry = handoff
        .declarations()
        .type_entries()
        .get(declaration.type_entry.unwrap())
        .expect("attributed reserve type entry should exist");
    assert_eq!(type_entry.status, TypeStatus::Unknown);
    let TypeEntryActual::Known(normalized_id) = type_entry.actual else {
        panic!("attributed reserve should keep a normalized type");
    };
    let normalized = handoff
        .declarations()
        .normalized_types()
        .get(normalized_id)
        .expect("normalized attributed reserve type should exist");
    assert_eq!(normalized.status, NormalizedTypeStatus::Known);
    assert_eq!(normalized.attributes.negative().len(), 1);
    assert_eq!(normalized.attributes.negative()[0].symbol, attr);
    assert!(normalized.attributes.positive().is_empty());
    assert_eq!(
        diagnostic_ranges(
            handoff.declarations(),
            "checker.declaration.deferred.evidence_query"
        ),
        vec![(8, 9)]
    );
    assert!(handoff.declarations().facts().is_empty());
}

#[test]
fn source_reserve_declaration_bridge_validates_local_symbol_heads_and_mismatched_inputs() {
    let source = source_id();
    let module = module_id();
    let mode = symbol_id("Mode/0", "pkg::main::Mode/0");
    let mode_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 10),
        vec![SourceReserveBindingInput::new(
            "x",
            range(source, 0, 1),
            range(source, 4, 8),
            "Mode",
            TypeHeadInput::Symbol(mode),
        )],
    )
    .expect("local mode reserve payload should validate source shape");
    let mode_symbols = symbol_env(vec![symbol_entry(
        symbol_id("Mode/0", "pkg::main::Mode/0"),
        SymbolKind::Mode,
    )]);
    let mode_handoff = mode_bridge
        .check(&mode_symbols)
        .expect("local mode reserve payload should reach declaration checking");
    assert_eq!(
        diagnostic_ranges(
            mode_handoff.declarations(),
            "checker.type.external.mode_expansion_payload"
        ),
        vec![(4, 8)]
    );

    let mode_expansion_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 10),
        vec![SourceReserveBindingInput::new(
            "x",
            range(source, 0, 1),
            range(source, 4, 8),
            "Mode",
            TypeHeadInput::Symbol(symbol_id("Mode/0", "pkg::main::Mode/0")),
        )],
    )
    .expect("local mode reserve payload should validate source shape");
    let mode_expansion_handoff = mode_expansion_bridge
        .check_with_mode_expansions(
            &mode_symbols,
            [(
                symbol_id("Mode/0", "pkg::main::Mode/0"),
                ModeExpansion::new(
                    TypeExpressionInput::new(
                        site(777),
                        range(source, 20, 23),
                        "set",
                        TypeHeadInput::BuiltinSet,
                    ),
                    Vec::new(),
                ),
            )],
        )
        .expect("local mode expansion payload should reach declaration checking");
    assert!(
        diagnostic_ranges(
            &mode_expansion_handoff.declarations,
            "checker.type.external.mode_expansion_payload"
        )
        .is_empty()
    );
    assert!(
        diagnostic_ranges(
            &mode_expansion_handoff.declarations,
            "checker.declaration.deferred.evidence_query"
        )
        .is_empty()
    );
    let expanded_mode_declaration = declarations_by_binding(mode_expansion_handoff.declarations())
        .remove(&BindingId::new(0))
        .expect("checked expanded mode declaration should exist");
    assert_eq!(expanded_mode_declaration.status, DeclarationStatus::Checked);
    let expanded_mode_type_entry = mode_expansion_handoff
        .declarations()
        .type_entries()
        .get(
            expanded_mode_declaration
                .type_entry
                .expect("expanded mode declaration should keep a type entry"),
        )
        .expect("expanded mode type entry should exist");
    assert_eq!(expanded_mode_type_entry.status, TypeStatus::Known);

    let struct_radix = symbol_id("Struct/0", "pkg::main::Struct/0");
    let mode_to_structure_symbols = symbol_env(vec![
        symbol_entry(symbol_id("Mode/0", "pkg::main::Mode/0"), SymbolKind::Mode),
        symbol_entry(struct_radix.clone(), SymbolKind::Structure),
    ]);
    let mode_to_structure_handoff = mode_expansion_bridge
        .check_with_mode_expansions(
            &mode_to_structure_symbols,
            [(
                symbol_id("Mode/0", "pkg::main::Mode/0"),
                ModeExpansion::new(
                    TypeExpressionInput::new(
                        site(778),
                        range(source, 20, 26),
                        "Struct",
                        TypeHeadInput::Symbol(struct_radix),
                    ),
                    Vec::new(),
                ),
            )],
        )
        .expect("mode expansion to structure should reach declaration checking");
    assert!(
        diagnostic_ranges(
            &mode_to_structure_handoff.declarations,
            "checker.type.external.mode_expansion_payload"
        )
        .is_empty()
    );
    assert_eq!(
        diagnostic_ranges(
            &mode_to_structure_handoff.declarations,
            "checker.declaration.deferred.evidence_query"
        ),
        vec![(0, 1)]
    );
    let mode_to_structure_declaration =
        declarations_by_binding(mode_to_structure_handoff.declarations())
            .remove(&BindingId::new(0))
            .expect("checked mode-to-structure declaration should exist");
    assert_eq!(
        mode_to_structure_declaration.status,
        DeclarationStatus::Partial
    );
    let mode_to_structure_type_entry = mode_to_structure_handoff
        .declarations()
        .type_entries()
        .get(
            mode_to_structure_declaration
                .type_entry
                .expect("mode-to-structure declaration should keep a type entry"),
        )
        .expect("mode-to-structure type entry should exist");
    assert_eq!(mode_to_structure_type_entry.status, TypeStatus::Unknown);
    assert!(
        mode_to_structure_handoff.declarations().facts().is_empty(),
        "mode expansions with structure radixes must not seed facts without base-shape evidence"
    );

    let chained_struct_radix = symbol_id("Struct/0", "pkg::main::Struct/0");
    let chained_mode = symbol_id("Mode/0", "pkg::main::Mode/0");
    let chained_alias = symbol_id("Alias/0", "pkg::main::Alias/0");
    let chained_mode_to_structure_symbols = symbol_env(vec![
        symbol_entry(chained_mode.clone(), SymbolKind::Mode),
        symbol_entry(chained_alias.clone(), SymbolKind::Mode),
        symbol_entry(chained_struct_radix.clone(), SymbolKind::Structure),
    ]);
    let chained_mode_to_structure_handoff = mode_expansion_bridge
        .check_with_mode_expansions(
            &chained_mode_to_structure_symbols,
            [
                (
                    chained_mode,
                    ModeExpansion::new(
                        TypeExpressionInput::new(
                            site(779),
                            range(source, 30, 35),
                            "Alias",
                            TypeHeadInput::Symbol(chained_alias.clone()),
                        ),
                        Vec::new(),
                    ),
                ),
                (
                    chained_alias,
                    ModeExpansion::new(
                        TypeExpressionInput::new(
                            site(780),
                            range(source, 40, 46),
                            "Struct",
                            TypeHeadInput::Symbol(chained_struct_radix),
                        ),
                        Vec::new(),
                    ),
                ),
            ],
        )
        .expect("chained mode expansion to structure should reach declaration checking");
    assert!(
        diagnostic_ranges(
            &chained_mode_to_structure_handoff.declarations,
            "checker.type.external.mode_expansion_payload"
        )
        .is_empty()
    );
    assert_eq!(
        diagnostic_ranges(
            &chained_mode_to_structure_handoff.declarations,
            "checker.declaration.deferred.evidence_query"
        ),
        vec![(0, 1)]
    );
    assert!(
        chained_mode_to_structure_handoff
            .declarations()
            .facts()
            .is_empty(),
        "chained mode expansions ending in structure radixes must not seed facts"
    );

    let attributed_expansion_attr = symbol_id("marked/0", "pkg::main::marked/0");
    let attributed_expansion_symbols = symbol_env(vec![
        symbol_entry(symbol_id("Mode/0", "pkg::main::Mode/0"), SymbolKind::Mode),
        symbol_entry(attributed_expansion_attr.clone(), SymbolKind::Attribute),
    ]);
    let attributed_expansion_handoff = mode_expansion_bridge
        .check_with_mode_expansions(
            &attributed_expansion_symbols,
            [(
                symbol_id("Mode/0", "pkg::main::Mode/0"),
                ModeExpansion::new(
                    TypeExpressionInput::new(
                        site(781),
                        range(source, 50, 53),
                        "set",
                        TypeHeadInput::BuiltinSet,
                    ),
                    vec![AttributeInput::new(
                        attributed_expansion_attr.clone(),
                        AttributePolarity::Positive,
                        range(source, 45, 51),
                        "marked",
                    )],
                ),
            )],
        )
        .expect("attributed mode expansion should reach declaration checking");
    assert!(
        diagnostic_ranges(
            &attributed_expansion_handoff.declarations,
            "checker.type.external.mode_expansion_payload"
        )
        .is_empty()
    );
    assert_eq!(
        diagnostic_ranges(
            &attributed_expansion_handoff.declarations,
            "checker.declaration.deferred.evidence_query"
        ),
        vec![(0, 1)]
    );
    let attributed_expansion_declaration =
        declarations_by_binding(attributed_expansion_handoff.declarations())
            .remove(&BindingId::new(0))
            .expect("checked attributed-expansion declaration should exist");
    assert_eq!(
        attributed_expansion_declaration.status,
        DeclarationStatus::Partial
    );
    let attributed_expansion_type_entry = attributed_expansion_handoff
        .declarations()
        .type_entries()
        .get(
            attributed_expansion_declaration
                .type_entry
                .expect("attributed-expansion declaration should keep a type entry"),
        )
        .expect("attributed-expansion type entry should exist");
    assert_eq!(attributed_expansion_type_entry.status, TypeStatus::Unknown);
    let TypeEntryActual::Known(attributed_expansion_normalized_id) =
        attributed_expansion_type_entry.actual
    else {
        panic!("attributed-expansion reserve should keep a normalized type");
    };
    let attributed_expansion_normalized = attributed_expansion_handoff
        .declarations()
        .normalized_types()
        .get(attributed_expansion_normalized_id)
        .expect("normalized attributed-expansion type should exist");
    assert_eq!(
        attributed_expansion_normalized.attributes.positive().len(),
        1
    );
    assert_eq!(
        attributed_expansion_normalized.attributes.positive()[0].symbol,
        attributed_expansion_attr
    );
    assert!(
        attributed_expansion_handoff
            .declarations()
            .facts()
            .is_empty(),
        "attributed mode expansions must not seed facts without existential evidence"
    );

    let attributed_reserve_attributed_expansion_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 10),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 17),
                "non marked Mode",
                TypeHeadInput::Symbol(symbol_id("Mode/0", "pkg::main::Mode/0")),
            )
            .with_type_attributes(vec![AttributeInput::new(
                attributed_expansion_attr.clone(),
                AttributePolarity::Negative,
                range(source, 4, 14),
                "non marked",
            )]),
        ],
    )
    .expect("attributed local mode reserve payload should validate source shape");
    let attributed_reserve_attributed_expansion_handoff =
        attributed_reserve_attributed_expansion_bridge
            .check_with_mode_expansions(
                &attributed_expansion_symbols,
                [(
                    symbol_id("Mode/0", "pkg::main::Mode/0"),
                    ModeExpansion::new(
                        TypeExpressionInput::new(
                            site(784),
                            range(source, 80, 83),
                            "set",
                            TypeHeadInput::BuiltinSet,
                        ),
                        vec![AttributeInput::new(
                            attributed_expansion_attr.clone(),
                            AttributePolarity::Negative,
                            range(source, 70, 80),
                            "non marked",
                        )],
                    ),
                )],
            )
            .expect("attributed reserve with attributed expansion should reach checking");
    assert!(
        diagnostic_ranges(
            &attributed_reserve_attributed_expansion_handoff.declarations,
            "checker.type.external.mode_expansion_payload"
        )
        .is_empty()
    );
    assert_eq!(
        diagnostic_ranges(
            &attributed_reserve_attributed_expansion_handoff.declarations,
            "checker.declaration.deferred.evidence_query"
        ),
        vec![(0, 1)]
    );
    let attributed_reserve_attributed_expansion_declaration =
        declarations_by_binding(attributed_reserve_attributed_expansion_handoff.declarations())
            .remove(&BindingId::new(0))
            .expect("checked attributed reserve/expansion declaration should exist");
    assert_eq!(
        attributed_reserve_attributed_expansion_declaration.status,
        DeclarationStatus::Partial
    );
    let attributed_reserve_attributed_expansion_type_entry =
        attributed_reserve_attributed_expansion_handoff
            .declarations()
            .type_entries()
            .get(
                attributed_reserve_attributed_expansion_declaration
                    .type_entry
                    .expect("attributed reserve/expansion should keep a type entry"),
            )
            .expect("attributed reserve/expansion type entry should exist");
    assert_eq!(
        attributed_reserve_attributed_expansion_type_entry.status,
        TypeStatus::Unknown
    );
    let TypeEntryActual::Known(attributed_reserve_attributed_expansion_normalized_id) =
        attributed_reserve_attributed_expansion_type_entry.actual
    else {
        panic!("attributed reserve/expansion should keep a normalized type");
    };
    let attributed_reserve_attributed_expansion_normalized =
        attributed_reserve_attributed_expansion_handoff
            .declarations()
            .normalized_types()
            .get(attributed_reserve_attributed_expansion_normalized_id)
            .expect("normalized attributed reserve/expansion type should exist");
    assert_eq!(
        attributed_reserve_attributed_expansion_normalized
            .attributes
            .negative()
            .len(),
        1
    );
    assert!(
        attributed_reserve_attributed_expansion_handoff
            .declarations()
            .facts()
            .is_empty(),
        "attributed reserve heads with attributed expansions must not seed facts"
    );

    let chained_attributed_attr = symbol_id("marked/0", "pkg::main::marked/0");
    let chained_attributed_mode = symbol_id("Mode/0", "pkg::main::Mode/0");
    let chained_attributed_alias = symbol_id("Alias/0", "pkg::main::Alias/0");
    let chained_attributed_symbols = symbol_env(vec![
        symbol_entry(chained_attributed_mode.clone(), SymbolKind::Mode),
        symbol_entry(chained_attributed_alias.clone(), SymbolKind::Mode),
        symbol_entry(chained_attributed_attr.clone(), SymbolKind::Attribute),
    ]);
    let chained_attributed_handoff = mode_expansion_bridge
        .check_with_mode_expansions(
            &chained_attributed_symbols,
            [
                (
                    chained_attributed_mode,
                    ModeExpansion::new(
                        TypeExpressionInput::new(
                            site(782),
                            range(source, 60, 65),
                            "Alias",
                            TypeHeadInput::Symbol(chained_attributed_alias.clone()),
                        ),
                        Vec::new(),
                    ),
                ),
                (
                    chained_attributed_alias,
                    ModeExpansion::new(
                        TypeExpressionInput::new(
                            site(783),
                            range(source, 70, 73),
                            "set",
                            TypeHeadInput::BuiltinSet,
                        )
                        .with_attributes(vec![AttributeInput::new(
                            chained_attributed_attr,
                            AttributePolarity::Positive,
                            range(source, 64, 70),
                            "marked",
                        )]),
                        Vec::new(),
                    ),
                ),
            ],
        )
        .expect("chained attributed mode expansion should reach declaration checking");
    assert!(
        diagnostic_ranges(
            &chained_attributed_handoff.declarations,
            "checker.type.external.mode_expansion_payload"
        )
        .is_empty()
    );
    assert_eq!(
        diagnostic_ranges(
            &chained_attributed_handoff.declarations,
            "checker.declaration.deferred.evidence_query"
        ),
        vec![(0, 1)]
    );
    assert!(
        chained_attributed_handoff.declarations().facts().is_empty(),
        "chained attributed mode expansions must not seed facts"
    );

    let structure = symbol_id("Struct/0", "pkg::main::Struct/0");
    let structure_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 10),
        vec![SourceReserveBindingInput::new(
            "x",
            range(source, 0, 1),
            range(source, 4, 10),
            "Struct",
            TypeHeadInput::Symbol(structure.clone()),
        )],
    )
    .expect("symbol reserve payload shape should validate before SymbolEnv checks");
    let structure_symbols = symbol_env(vec![symbol_entry(structure, SymbolKind::Structure)]);
    let structure_handoff = structure_bridge
        .check(&structure_symbols)
        .expect("local structure reserve payload should reach declaration checking");
    assert_eq!(
        diagnostic_ranges(
            structure_handoff.declarations(),
            "checker.declaration.deferred.evidence_query"
        ),
        vec![(0, 1)]
    );
    assert!(
        structure_handoff.declarations().facts().is_empty(),
        "local structure reserve heads must not seed facts without base-shape evidence"
    );

    let attributed_structure = symbol_id("Struct/0", "pkg::main::Struct/0");
    let attributed_structure_attr = symbol_id("empty/0", "pkg::main::empty/0");
    let attributed_structure_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 25),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 25),
                "non empty Struct",
                TypeHeadInput::Symbol(attributed_structure.clone()),
            )
            .with_type_attributes(vec![AttributeInput::new(
                attributed_structure_attr.clone(),
                AttributePolarity::Negative,
                range(source, 4, 13),
                "non empty",
            )]),
        ],
    )
    .expect("attributed local structure reserve payload should validate source shape");
    let attributed_structure_symbols = symbol_env(vec![
        symbol_entry(attributed_structure, SymbolKind::Structure),
        symbol_entry(attributed_structure_attr.clone(), SymbolKind::Attribute),
    ]);
    let attributed_structure_handoff = attributed_structure_bridge
        .check(&attributed_structure_symbols)
        .expect("attributed local structure reserve payload should reach declaration checking");
    assert_eq!(
        diagnostic_ranges(
            attributed_structure_handoff.declarations(),
            "checker.declaration.deferred.evidence_query"
        ),
        vec![(0, 1)]
    );
    let attributed_structure_declaration =
        declarations_by_binding(attributed_structure_handoff.declarations())
            .remove(&BindingId::new(0))
            .expect("checked attributed structure declaration should exist");
    let attributed_structure_type_entry = attributed_structure_handoff
        .declarations()
        .type_entries()
        .get(
            attributed_structure_declaration
                .type_entry
                .expect("attributed structure declaration should keep a type entry"),
        )
        .expect("attributed structure type entry should exist");
    let TypeEntryActual::Known(attributed_structure_normalized_id) =
        attributed_structure_type_entry.actual
    else {
        panic!("attributed local structure should keep a normalized type");
    };
    let attributed_structure_normalized = attributed_structure_handoff
        .declarations()
        .normalized_types()
        .get(attributed_structure_normalized_id)
        .expect("normalized attributed local structure type should exist");
    assert_eq!(
        attributed_structure_normalized.status,
        NormalizedTypeStatus::Known
    );
    assert_eq!(
        attributed_structure_normalized.attributes.negative().len(),
        1
    );
    assert_eq!(
        attributed_structure_normalized.attributes.negative()[0].symbol,
        attributed_structure_attr
    );
    assert!(
        attributed_structure_normalized
            .attributes
            .positive()
            .is_empty()
    );
    assert!(
        attributed_structure_handoff
            .declarations()
            .facts()
            .is_empty(),
        "attributed local structure reserve heads must not seed facts without existential evidence"
    );

    let imported_attribute = SymbolId::new(
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("imported")),
        LocalSymbolId::new("empty/0"),
        FullyQualifiedName::new("pkg::imported::empty/0"),
    );
    let imported_attribute_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 25),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 25),
                "non empty Struct",
                TypeHeadInput::Symbol(symbol_id("Struct/0", "pkg::main::Struct/0")),
            )
            .with_type_attributes(vec![AttributeInput::new(
                imported_attribute.clone(),
                AttributePolarity::Negative,
                range(source, 4, 13),
                "non empty",
            )]),
        ],
    )
    .expect("imported attribute payload shape should validate before SymbolEnv checks");
    assert!(
        imported_attribute_bridge
            .check(&symbol_env_with_imported_attribute(
                symbol_id("Struct/0", "pkg::main::Struct/0"),
                imported_attribute.clone()
            ))
            .is_err(),
        "imported source attribute payloads must fail closed at the checker seam"
    );
    let generic_imported_set_attribute_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 17),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 17),
                "non empty set",
                TypeHeadInput::BuiltinSet,
            )
            .with_type_attributes(vec![AttributeInput::new(
                imported_attribute.clone(),
                AttributePolarity::Negative,
                range(source, 4, 13),
                "non empty",
            )]),
        ],
    )
    .expect("generic imported attribute over builtin set should validate shape");
    assert!(
        generic_imported_set_attribute_bridge
            .check(&symbol_env_with_imported_symbol(
                imported_attribute.clone(),
                SymbolKind::Attribute,
                "empty",
            ))
            .is_err(),
        "generic imported attributes must not be accepted by the parser fixture bridge"
    );

    let imported_fixture_attribute = SymbolId::new(
        ModuleId::new(
            PackageId::new("pkg"),
            ModulePath::new("parser.type_fixtures"),
        ),
        LocalSymbolId::new("TypeCaseAttr/0"),
        FullyQualifiedName::new("pkg::parser.type_fixtures::TypeCaseAttr/0"),
    );
    let imported_fixture_attribute_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 22),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 20),
                "TypeCaseAttr set",
                TypeHeadInput::BuiltinSet,
            )
            .with_type_attributes(vec![AttributeInput::new(
                imported_fixture_attribute.clone(),
                AttributePolarity::Positive,
                range(source, 4, 16),
                "TypeCaseAttr",
            )]),
        ],
    )
    .expect("imported fixture attribute payload shape should validate");
    let imported_fixture_attribute_handoff = imported_fixture_attribute_bridge
        .check(&symbol_env_with_imported_symbol(
            imported_fixture_attribute.clone(),
            SymbolKind::Attribute,
            "TypeCaseAttr",
        ))
        .expect("imported TypeCaseAttr should pass checker provenance validation");
    assert_eq!(
        diagnostic_ranges(
            imported_fixture_attribute_handoff.declarations(),
            "checker.declaration.deferred.evidence_query"
        ),
        vec![(0, 1)]
    );
    let imported_fixture_attribute_declaration =
        declarations_by_binding(imported_fixture_attribute_handoff.declarations())
            .remove(&BindingId::new(0))
            .expect("checked imported attribute declaration should exist");
    let imported_fixture_attribute_type_entry = imported_fixture_attribute_handoff
        .declarations()
        .type_entries()
        .get(
            imported_fixture_attribute_declaration
                .type_entry
                .expect("imported attribute declaration should keep a type entry"),
        )
        .expect("imported attribute type entry should exist");
    let TypeEntryActual::Known(imported_fixture_attribute_normalized_id) =
        imported_fixture_attribute_type_entry.actual
    else {
        panic!("imported fixture attribute should keep a normalized type");
    };
    let imported_fixture_attribute_normalized = imported_fixture_attribute_handoff
        .declarations()
        .normalized_types()
        .get(imported_fixture_attribute_normalized_id)
        .expect("normalized imported fixture attribute type should exist");
    assert_eq!(
        imported_fixture_attribute_normalized.status,
        NormalizedTypeStatus::Known
    );
    assert_eq!(
        imported_fixture_attribute_normalized
            .attributes
            .positive()
            .len(),
        1
    );
    assert_eq!(
        imported_fixture_attribute_normalized.attributes.positive()[0].symbol,
        imported_fixture_attribute
    );
    assert!(
        imported_fixture_attribute_handoff
            .declarations()
            .facts()
            .is_empty(),
        "imported attributed reserve heads must not seed facts without existential evidence"
    );
    let imported_fixture_attribute_object_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 25),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 25),
                "TypeCaseAttr object",
                TypeHeadInput::BuiltinObject,
            )
            .with_type_attributes(vec![AttributeInput::new(
                imported_fixture_attribute.clone(),
                AttributePolarity::Positive,
                range(source, 4, 16),
                "TypeCaseAttr",
            )]),
        ],
    )
    .expect("imported fixture attribute over object should validate shape before provenance");
    assert!(
        imported_fixture_attribute_object_bridge
            .check(&symbol_env_with_imported_symbol(
                imported_fixture_attribute.clone(),
                SymbolKind::Attribute,
                "TypeCaseAttr",
            ))
            .is_err(),
        "imported TypeCaseAttr is source-supported only over builtin set"
    );
    let imported_fixture_attribute_structure_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 25),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 23),
                "TypeCaseAttr Struct",
                TypeHeadInput::Symbol(symbol_id("Struct/0", "pkg::main::Struct/0")),
            )
            .with_type_attributes(vec![AttributeInput::new(
                imported_fixture_attribute.clone(),
                AttributePolarity::Positive,
                range(source, 4, 16),
                "TypeCaseAttr",
            )]),
        ],
    )
    .expect(
        "imported fixture attribute over a symbol head should validate shape before provenance",
    );
    assert!(
        imported_fixture_attribute_structure_bridge
            .check(&symbol_env_with_local_structure_and_imported_symbol(
                symbol_id("Struct/0", "pkg::main::Struct/0"),
                imported_fixture_attribute.clone(),
                SymbolKind::Attribute,
                "TypeCaseAttr",
            ))
            .is_err(),
        "imported TypeCaseAttr must not bridge attributed local symbol heads"
    );

    let imported_fixture_empty_attribute = SymbolId::new(
        ModuleId::new(
            PackageId::new("pkg"),
            ModulePath::new("parser.type_fixtures"),
        ),
        LocalSymbolId::new("empty/0"),
        FullyQualifiedName::new("pkg::parser.type_fixtures::empty/0"),
    );
    let imported_fixture_empty_attribute_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 22),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 17),
                "non empty set",
                TypeHeadInput::BuiltinSet,
            )
            .with_type_attributes(vec![AttributeInput::new(
                imported_fixture_empty_attribute.clone(),
                AttributePolarity::Negative,
                range(source, 4, 13),
                "non empty",
            )]),
        ],
    )
    .expect("imported fixture empty attribute payload shape should validate");
    let imported_fixture_empty_attribute_handoff = imported_fixture_empty_attribute_bridge
        .check(&symbol_env_with_imported_symbol(
            imported_fixture_empty_attribute.clone(),
            SymbolKind::Attribute,
            "empty",
        ))
        .expect("imported negative empty over builtin set should pass provenance validation");
    assert_eq!(
        diagnostic_ranges(
            imported_fixture_empty_attribute_handoff.declarations(),
            "checker.declaration.deferred.evidence_query"
        ),
        vec![(0, 1)]
    );
    let imported_fixture_empty_attribute_declaration =
        declarations_by_binding(imported_fixture_empty_attribute_handoff.declarations())
            .remove(&BindingId::new(0))
            .expect("checked imported empty declaration should exist");
    let imported_fixture_empty_attribute_type_entry = imported_fixture_empty_attribute_handoff
        .declarations()
        .type_entries()
        .get(
            imported_fixture_empty_attribute_declaration
                .type_entry
                .expect("imported empty declaration should keep a type entry"),
        )
        .expect("imported empty type entry should exist");
    let TypeEntryActual::Known(imported_fixture_empty_attribute_normalized_id) =
        imported_fixture_empty_attribute_type_entry.actual
    else {
        panic!("imported fixture empty should keep a normalized type");
    };
    let imported_fixture_empty_attribute_normalized = imported_fixture_empty_attribute_handoff
        .declarations()
        .normalized_types()
        .get(imported_fixture_empty_attribute_normalized_id)
        .expect("normalized imported fixture empty type should exist");
    assert_eq!(
        imported_fixture_empty_attribute_normalized.status,
        NormalizedTypeStatus::Known
    );
    assert_eq!(
        imported_fixture_empty_attribute_normalized
            .attributes
            .negative()
            .len(),
        1
    );
    assert_eq!(
        imported_fixture_empty_attribute_normalized
            .attributes
            .negative()[0]
            .symbol,
        imported_fixture_empty_attribute
    );
    assert!(
        imported_fixture_empty_attribute_handoff
            .declarations()
            .facts()
            .is_empty(),
        "imported negative empty reserve heads must not seed facts without evidence"
    );
    let imported_fixture_positive_empty_attribute_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 18),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 13),
                "empty set",
                TypeHeadInput::BuiltinSet,
            )
            .with_type_attributes(vec![AttributeInput::new(
                imported_fixture_empty_attribute.clone(),
                AttributePolarity::Positive,
                range(source, 4, 9),
                "empty",
            )]),
        ],
    )
    .expect("positive imported fixture empty payload shape should validate");
    let imported_fixture_positive_empty_attribute_handoff =
        imported_fixture_positive_empty_attribute_bridge
            .check(&symbol_env_with_imported_symbol(
                imported_fixture_empty_attribute.clone(),
                SymbolKind::Attribute,
                "empty",
            ))
            .expect("imported positive empty over builtin set should pass provenance validation");
    assert_eq!(
        diagnostic_ranges(
            imported_fixture_positive_empty_attribute_handoff.declarations(),
            "checker.declaration.deferred.evidence_query"
        ),
        vec![(0, 1)]
    );
    let imported_fixture_positive_empty_attribute_declaration =
        declarations_by_binding(imported_fixture_positive_empty_attribute_handoff.declarations())
            .remove(&BindingId::new(0))
            .expect("checked positive imported empty declaration should exist");
    let imported_fixture_positive_empty_attribute_type_entry =
        imported_fixture_positive_empty_attribute_handoff
            .declarations()
            .type_entries()
            .get(
                imported_fixture_positive_empty_attribute_declaration
                    .type_entry
                    .expect("positive imported empty declaration should keep a type entry"),
            )
            .expect("positive imported empty type entry should exist");
    let TypeEntryActual::Known(imported_fixture_positive_empty_attribute_normalized_id) =
        imported_fixture_positive_empty_attribute_type_entry.actual
    else {
        panic!("positive imported fixture empty should keep a normalized type");
    };
    let imported_fixture_positive_empty_attribute_normalized =
        imported_fixture_positive_empty_attribute_handoff
            .declarations()
            .normalized_types()
            .get(imported_fixture_positive_empty_attribute_normalized_id)
            .expect("normalized positive imported fixture empty type should exist");
    assert_eq!(
        imported_fixture_positive_empty_attribute_normalized.status,
        NormalizedTypeStatus::Known
    );
    assert_eq!(
        imported_fixture_positive_empty_attribute_normalized
            .attributes
            .positive()
            .len(),
        1
    );
    assert_eq!(
        imported_fixture_positive_empty_attribute_normalized
            .attributes
            .positive()[0]
            .symbol,
        imported_fixture_empty_attribute
    );
    assert!(
        imported_fixture_positive_empty_attribute_handoff
            .declarations()
            .facts()
            .is_empty(),
        "imported positive empty reserve heads must not seed facts without evidence"
    );
    let imported_fixture_empty_object_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 25),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 20),
                "non empty object",
                TypeHeadInput::BuiltinObject,
            )
            .with_type_attributes(vec![AttributeInput::new(
                imported_fixture_empty_attribute.clone(),
                AttributePolarity::Negative,
                range(source, 4, 13),
                "non empty",
            )]),
        ],
    )
    .expect("imported fixture empty over object should validate shape before provenance");
    let imported_fixture_empty_object_handoff = imported_fixture_empty_object_bridge
        .check(&symbol_env_with_imported_symbol(
            imported_fixture_empty_attribute.clone(),
            SymbolKind::Attribute,
            "empty",
        ))
        .expect("negative imported empty over builtin object should reach declaration checking");
    assert_eq!(
        diagnostic_ranges(
            imported_fixture_empty_object_handoff.declarations(),
            "checker.declaration.deferred.evidence_query"
        ),
        vec![(0, 1)]
    );
    let imported_fixture_empty_object_declaration =
        declarations_by_binding(imported_fixture_empty_object_handoff.declarations())
            .remove(&BindingId::new(0))
            .expect("checked negative imported empty object declaration should exist");
    let imported_fixture_empty_object_type_entry = imported_fixture_empty_object_handoff
        .declarations()
        .type_entries()
        .get(
            imported_fixture_empty_object_declaration
                .type_entry
                .expect("negative imported empty object declaration should keep a type entry"),
        )
        .expect("negative imported empty object type entry should exist");
    let TypeEntryActual::Known(imported_fixture_empty_object_normalized_id) =
        imported_fixture_empty_object_type_entry.actual
    else {
        panic!("negative imported empty object should keep a normalized type");
    };
    let imported_fixture_empty_object_normalized = imported_fixture_empty_object_handoff
        .declarations()
        .normalized_types()
        .get(imported_fixture_empty_object_normalized_id)
        .expect("normalized negative imported empty object type should exist");
    assert!(matches!(
        imported_fixture_empty_object_normalized.head,
        TypeHeadRef::BuiltinObject
    ));
    assert_eq!(
        imported_fixture_empty_object_normalized
            .attributes
            .negative()
            .len(),
        1
    );
    assert_eq!(
        imported_fixture_empty_object_normalized
            .attributes
            .negative()[0]
            .symbol,
        imported_fixture_empty_attribute
    );
    assert!(
        imported_fixture_empty_object_normalized
            .attributes
            .positive()
            .is_empty()
    );
    assert!(
        imported_fixture_empty_object_handoff
            .declarations()
            .facts()
            .is_empty(),
        "negative imported empty object reserve heads must not seed facts without evidence"
    );
    let imported_fixture_duplicate_empty_object_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 35),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 30),
                "non empty non empty object",
                TypeHeadInput::BuiltinObject,
            )
            .with_type_attributes(vec![
                AttributeInput::new(
                    imported_fixture_empty_attribute.clone(),
                    AttributePolarity::Negative,
                    range(source, 4, 13),
                    "non empty",
                ),
                AttributeInput::new(
                    imported_fixture_empty_attribute.clone(),
                    AttributePolarity::Negative,
                    range(source, 14, 23),
                    "non empty",
                ),
            ]),
        ],
    )
    .expect("duplicate imported empty object payload should validate shape");
    assert!(
        imported_fixture_duplicate_empty_object_bridge
            .check(&symbol_env_with_imported_symbol(
                imported_fixture_empty_attribute.clone(),
                SymbolKind::Attribute,
                "empty",
            ))
            .is_err(),
        "duplicate imported empty object attributes must remain unsupported"
    );
    let imported_fixture_positive_empty_object_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 18),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 16),
                "empty object",
                TypeHeadInput::BuiltinObject,
            )
            .with_type_attributes(vec![AttributeInput::new(
                imported_fixture_empty_attribute.clone(),
                AttributePolarity::Positive,
                range(source, 4, 9),
                "empty",
            )]),
        ],
    )
    .expect("positive imported fixture empty over object should validate shape");
    assert!(
        imported_fixture_positive_empty_object_bridge
            .check(&symbol_env_with_imported_symbol(
                imported_fixture_empty_attribute.clone(),
                SymbolKind::Attribute,
                "empty",
            ))
            .is_err(),
        "positive imported empty is source-supported only for builtin set"
    );
    let imported_fixture_empty_structure_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 25),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 20),
                "non empty Struct",
                TypeHeadInput::Symbol(symbol_id("Struct/0", "pkg::main::Struct/0")),
            )
            .with_type_attributes(vec![AttributeInput::new(
                imported_fixture_empty_attribute.clone(),
                AttributePolarity::Negative,
                range(source, 4, 13),
                "non empty",
            )]),
        ],
    )
    .expect("imported fixture empty over a symbol head should validate shape");
    assert!(
        imported_fixture_empty_structure_bridge
            .check(&symbol_env_with_local_structure_and_imported_symbol(
                symbol_id("Struct/0", "pkg::main::Struct/0"),
                imported_fixture_empty_attribute.clone(),
                SymbolKind::Attribute,
                "empty",
            ))
            .is_err(),
        "imported empty must not bridge attributed local symbol heads"
    );

    let wrong_kind_attribute = symbol_id("empty/0", "pkg::main::empty/0");
    let wrong_kind_attribute_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 25),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 25),
                "non empty Struct",
                TypeHeadInput::Symbol(symbol_id("Struct/0", "pkg::main::Struct/0")),
            )
            .with_type_attributes(vec![AttributeInput::new(
                wrong_kind_attribute.clone(),
                AttributePolarity::Negative,
                range(source, 4, 13),
                "non empty",
            )]),
        ],
    )
    .expect("wrong-kind attribute payload shape should validate before SymbolEnv checks");
    let wrong_kind_symbols = symbol_env(vec![
        symbol_entry(
            symbol_id("Struct/0", "pkg::main::Struct/0"),
            SymbolKind::Structure,
        ),
        symbol_entry(wrong_kind_attribute, SymbolKind::Mode),
    ]);
    assert!(
        wrong_kind_attribute_bridge
            .check(&wrong_kind_symbols)
            .is_err(),
        "non-attribute symbol payloads must fail closed at the checker seam"
    );

    let attribute_args = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 25),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 25),
                "empty(x) Struct",
                TypeHeadInput::Symbol(symbol_id("Struct/0", "pkg::main::Struct/0")),
            )
            .with_type_attributes(vec![
                AttributeInput::new(
                    symbol_id("empty/0", "pkg::main::empty/0"),
                    AttributePolarity::Positive,
                    range(source, 4, 12),
                    "empty(x)",
                )
                .with_args(vec![TypeExpressionInput::new(
                    TypedSiteRef::Node(TypedNodeId::new(999)),
                    range(source, 10, 11),
                    "set",
                    TypeHeadInput::BuiltinSet,
                )]),
            ]),
        ],
    );
    assert!(
        attribute_args.is_err(),
        "attribute arguments must stay on the extraction gap"
    );

    let attributed_mode = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 18),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 18),
                "non empty Mode",
                TypeHeadInput::Symbol(symbol_id("Mode/0", "pkg::main::Mode/0")),
            )
            .with_type_attributes(vec![AttributeInput::new(
                symbol_id("empty/0", "pkg::main::empty/0"),
                AttributePolarity::Negative,
                range(source, 4, 13),
                "non empty",
            )]),
        ],
    )
    .expect("attributed local mode payload should validate before SymbolEnv checks");
    let mode_attribute_symbols = symbol_env(vec![
        symbol_entry(symbol_id("Mode/0", "pkg::main::Mode/0"), SymbolKind::Mode),
        symbol_entry(
            symbol_id("empty/0", "pkg::main::empty/0"),
            SymbolKind::Attribute,
        ),
    ]);
    let attributed_mode_handoff = attributed_mode
        .check(&mode_attribute_symbols)
        .expect("attributed local mode reserve payload should reach type normalization");
    assert_eq!(
        diagnostic_ranges(
            attributed_mode_handoff.declarations(),
            "checker.type.external.mode_expansion_payload"
        ),
        vec![(4, 18)]
    );
    assert!(
        diagnostic_ranges(
            attributed_mode_handoff.declarations(),
            "checker.declaration.deferred.evidence_query"
        )
        .is_empty(),
        "attributed local modes must not receive an evidence-query diagnostic before real expansion payloads exist"
    );
    let attributed_mode_declaration =
        declarations_by_binding(attributed_mode_handoff.declarations())
            .remove(&BindingId::new(0))
            .expect("checked attributed mode declaration should exist");
    let attributed_mode_type_entry = attributed_mode_handoff
        .declarations()
        .type_entries()
        .get(
            attributed_mode_declaration
                .type_entry
                .expect("attributed mode declaration should keep a type entry"),
        )
        .expect("attributed mode type entry should exist");
    let TypeEntryActual::Known(attributed_mode_normalized_id) = attributed_mode_type_entry.actual
    else {
        panic!("attributed local mode should keep a normalized type");
    };
    let attributed_mode_normalized = attributed_mode_handoff
        .declarations()
        .normalized_types()
        .get(attributed_mode_normalized_id)
        .expect("normalized attributed local mode type should exist");
    assert_eq!(
        attributed_mode_normalized.status,
        NormalizedTypeStatus::Degraded
    );
    assert_eq!(attributed_mode_normalized.attributes.negative().len(), 1);
    assert_eq!(
        attributed_mode_normalized.attributes.negative()[0].symbol,
        symbol_id("empty/0", "pkg::main::empty/0")
    );
    assert!(attributed_mode_normalized.attributes.positive().is_empty());
    assert!(
        attributed_mode_handoff.declarations().facts().is_empty(),
        "attributed local mode reserve heads must not seed facts without mode expansion and existential evidence"
    );

    let attributed_mode_with_expansion = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 18),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 18),
                "non empty Mode",
                TypeHeadInput::Symbol(symbol_id("Mode/0", "pkg::main::Mode/0")),
            )
            .with_type_attributes(vec![AttributeInput::new(
                symbol_id("empty/0", "pkg::main::empty/0"),
                AttributePolarity::Negative,
                range(source, 4, 13),
                "non empty",
            )]),
        ],
    )
    .expect("attributed mode reserve payload should validate before SymbolEnv checks");
    let attributed_mode_with_expansion_handoff = attributed_mode_with_expansion
        .check_with_mode_expansions(
            &mode_attribute_symbols,
            [(
                symbol_id("Mode/0", "pkg::main::Mode/0"),
                ModeExpansion::new(
                    TypeExpressionInput::new(
                        site(778),
                        range(source, 20, 23),
                        "set",
                        TypeHeadInput::BuiltinSet,
                    ),
                    Vec::new(),
                ),
            )],
        )
        .expect("attributed mode with real expansion should fail closed in declaration checking");
    assert_eq!(
        diagnostic_ranges(
            attributed_mode_with_expansion_handoff.declarations(),
            "checker.declaration.deferred.evidence_query"
        ),
        vec![(0, 1)]
    );
    assert!(
        diagnostic_ranges(
            attributed_mode_with_expansion_handoff.declarations(),
            "checker.type.external.mode_expansion_payload"
        )
        .is_empty(),
        "real mode expansion should move attributed mode uses past the missing-expansion gap"
    );
    assert!(
        attributed_mode_with_expansion_handoff
            .declarations()
            .facts()
            .is_empty(),
        "real mode expansion must not make attributed mode uses accepted without existential evidence"
    );

    let structure_attribute_symbols = symbol_env(vec![
        symbol_entry(symbol_id("Mode/0", "pkg::main::Mode/0"), SymbolKind::Mode),
        symbol_entry(
            symbol_id("Struct/0", "pkg::main::Struct/0"),
            SymbolKind::Structure,
        ),
        symbol_entry(
            symbol_id("empty/0", "pkg::main::empty/0"),
            SymbolKind::Attribute,
        ),
    ]);
    let attributed_mode_with_structure_expansion = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 18),
        vec![
            SourceReserveBindingInput::new(
                "x",
                range(source, 0, 1),
                range(source, 4, 18),
                "non empty Mode",
                TypeHeadInput::Symbol(symbol_id("Mode/0", "pkg::main::Mode/0")),
            )
            .with_type_attributes(vec![AttributeInput::new(
                symbol_id("empty/0", "pkg::main::empty/0"),
                AttributePolarity::Negative,
                range(source, 4, 13),
                "non empty",
            )]),
        ],
    )
    .expect("attributed mode reserve payload should validate before SymbolEnv checks");
    let attributed_mode_with_structure_expansion_handoff = attributed_mode_with_structure_expansion
        .check_with_mode_expansions(
            &structure_attribute_symbols,
            [(
                symbol_id("Mode/0", "pkg::main::Mode/0"),
                ModeExpansion::new(
                    TypeExpressionInput::new(
                        site(779),
                        range(source, 20, 31),
                        "Struct",
                        TypeHeadInput::Symbol(symbol_id("Struct/0", "pkg::main::Struct/0")),
                    ),
                    Vec::new(),
                ),
            )],
        )
        .expect("attributed mode with real structure RHS expansion should fail closed");
    assert_eq!(
        diagnostic_ranges(
            attributed_mode_with_structure_expansion_handoff.declarations(),
            "checker.declaration.deferred.evidence_query"
        ),
        vec![(0, 1)]
    );
    assert!(
        diagnostic_ranges(
            attributed_mode_with_structure_expansion_handoff.declarations(),
            "checker.type.external.mode_expansion_payload"
        )
        .is_empty(),
        "real structure RHS expansion should move attributed mode uses past the missing-expansion gap"
    );
    assert!(
        attributed_mode_with_structure_expansion_handoff
            .declarations()
            .facts()
            .is_empty(),
        "structure RHS expansion must not make attributed mode uses accepted without evidence"
    );

    let unresolved = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 10),
        vec![SourceReserveBindingInput::new(
            "x",
            range(source, 0, 1),
            range(source, 4, 11),
            "Missing",
            TypeHeadInput::Unresolved("Missing".to_owned()),
        )],
    );
    assert!(
        unresolved.is_err(),
        "unresolved source reserve heads must stay on the extraction gap"
    );

    let ambiguous = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 10),
        vec![SourceReserveBindingInput::new(
            "x",
            range(source, 0, 1),
            range(source, 4, 8),
            "Mode",
            TypeHeadInput::Ambiguous(vec![
                symbol_id("Mode/0", "pkg::main::Mode/0"),
                symbol_id("Mode/1", "pkg::main::Mode/1"),
            ]),
        )],
    );
    assert!(
        ambiguous.is_err(),
        "ambiguous source reserve heads must stay on the extraction gap"
    );

    let other_source = source_ids_pair().1;
    let mismatched_range = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 10),
        vec![SourceReserveBindingInput::new(
            "x",
            range(other_source, 0, 1),
            range(source, 4, 7),
            "set",
            TypeHeadInput::BuiltinSet,
        )],
    );
    assert!(
        mismatched_range.is_err(),
        "source-derived payload ranges must belong to the bridge source"
    );

    let symbols = SymbolEnv::new(
        ModuleId::new(PackageId::new("test"), ModulePath::new("other")),
        SymbolEnvIndexes::default(),
    );
    let bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 10),
        vec![SourceReserveBindingInput::new(
            "x",
            range(source, 0, 1),
            range(source, 4, 7),
            "set",
            TypeHeadInput::BuiltinSet,
        )],
    )
    .expect("builtin payload should validate");
    assert!(
        bridge.check(&symbols).is_err(),
        "symbol environment module mismatches must fail closed"
    );

    let imported_mode = SymbolId::new(
        ModuleId::new(
            PackageId::new("pkg"),
            ModulePath::new("parser.type_fixtures"),
        ),
        LocalSymbolId::new("summary:TypeCaseMode/0"),
        FullyQualifiedName::new("pkg::parser.type_fixtures::TypeCaseMode/0"),
    );
    let imported_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 16),
        vec![SourceReserveBindingInput::new(
            "x",
            range(source, 0, 1),
            range(source, 4, 16),
            "TypeCaseMode",
            TypeHeadInput::Symbol(imported_mode.clone()),
        )],
    )
    .expect("symbol reserve payload shape should validate before local module checks");
    let imported_handoff = imported_bridge
        .check(&symbol_env_with_imported_symbol(
            imported_mode.clone(),
            SymbolKind::Mode,
            "TypeCaseMode",
        ))
        .expect("imported mode source provenance should reach type normalization");
    assert_eq!(
        diagnostic_ranges(
            imported_handoff.declarations(),
            "checker.type.external.mode_expansion_payload"
        ),
        vec![(4, 16)]
    );
    assert_eq!(
        diagnostic_ranges(imported_handoff.declarations(), "checker.type.recovery"),
        vec![(4, 16)]
    );
    assert!(
        imported_handoff.declarations().facts().is_empty(),
        "imported mode heads must not seed facts without real expansion payloads"
    );

    let generic_imported_mode = SymbolId::new(
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("imported")),
        LocalSymbolId::new("ImportedMode/0"),
        FullyQualifiedName::new("pkg::imported::ImportedMode/0"),
    );
    let generic_imported_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 15),
        vec![SourceReserveBindingInput::new(
            "x",
            range(source, 0, 1),
            range(source, 4, 15),
            "ImportedMode",
            TypeHeadInput::Symbol(generic_imported_mode.clone()),
        )],
    )
    .expect("generic imported mode payload shape should validate before seam checks");
    assert!(
        generic_imported_bridge
            .check(&symbol_env_with_imported_symbol(
                generic_imported_mode,
                SymbolKind::Mode,
                "ImportedMode",
            ))
            .is_err(),
        "imported mode heads outside the TypeCaseMode bridge must stay fail-closed"
    );

    let imported_fixture_structure = SymbolId::new(
        ModuleId::new(
            PackageId::new("pkg"),
            ModulePath::new("parser.type_fixtures"),
        ),
        LocalSymbolId::new("summary:R/0"),
        FullyQualifiedName::new("pkg::parser.type_fixtures::R/0"),
    );
    let imported_fixture_structure_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 5),
        vec![SourceReserveBindingInput::new(
            "x",
            range(source, 0, 1),
            range(source, 4, 5),
            "R",
            TypeHeadInput::Symbol(imported_fixture_structure.clone()),
        )],
    )
    .expect("imported fixture structure payload shape should validate");
    let imported_fixture_structure_handoff = imported_fixture_structure_bridge
        .check(&symbol_env_with_imported_symbol(
            imported_fixture_structure,
            SymbolKind::Structure,
            "R",
        ))
        .expect("imported structure provenance should reach declaration checking");
    assert_eq!(
        diagnostic_ranges(
            imported_fixture_structure_handoff.declarations(),
            "checker.declaration.deferred.evidence_query"
        ),
        vec![(0, 1)]
    );
    assert!(
        imported_fixture_structure_handoff
            .declarations()
            .facts()
            .is_empty(),
        "imported structure heads must not seed facts without base-shape evidence"
    );

    let imported_fixture_type_case_struct = SymbolId::new(
        ModuleId::new(
            PackageId::new("pkg"),
            ModulePath::new("parser.type_fixtures"),
        ),
        LocalSymbolId::new("summary:TypeCaseStruct/0"),
        FullyQualifiedName::new("pkg::parser.type_fixtures::TypeCaseStruct/0"),
    );
    let imported_fixture_type_case_struct_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 18),
        vec![SourceReserveBindingInput::new(
            "x",
            range(source, 0, 1),
            range(source, 4, 18),
            "TypeCaseStruct",
            TypeHeadInput::Symbol(imported_fixture_type_case_struct.clone()),
        )],
    )
    .expect("imported fixture structure payload shape should validate");
    let imported_fixture_type_case_struct_handoff = imported_fixture_type_case_struct_bridge
        .check(&symbol_env_with_imported_symbol(
            imported_fixture_type_case_struct,
            SymbolKind::Structure,
            "TypeCaseStruct",
        ))
        .expect("TypeCaseStruct imported structure provenance should reach declaration checking");
    assert_eq!(
        diagnostic_ranges(
            imported_fixture_type_case_struct_handoff.declarations(),
            "checker.declaration.deferred.evidence_query"
        ),
        vec![(0, 1)]
    );
    assert!(
        imported_fixture_type_case_struct_handoff
            .declarations()
            .facts()
            .is_empty(),
        "TypeCaseStruct heads must not seed facts without base-shape evidence"
    );

    let foreign_local_mode = SymbolId::new(
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("imported")),
        LocalSymbolId::new("ForeignLocalMode/0"),
        FullyQualifiedName::new("pkg::imported::ForeignLocalMode/0"),
    );
    let foreign_local_mode_bridge = SourceReserveDeclarationBridge::new(
        source,
        module.clone(),
        range(source, 0, 20),
        vec![SourceReserveBindingInput::new(
            "x",
            range(source, 0, 1),
            range(source, 4, 20),
            "ForeignLocalMode",
            TypeHeadInput::Symbol(foreign_local_mode.clone()),
        )],
    )
    .expect("foreign mode payload shape should validate before provenance checks");
    assert!(
        foreign_local_mode_bridge
            .check(&symbol_env_with_foreign_local_symbol(
                foreign_local_mode,
                SymbolKind::Mode,
                "ForeignLocalMode",
            ))
            .is_err(),
        "foreign mode heads must require ImportedSource provenance"
    );

    let imported_structure = SymbolId::new(
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("imported")),
        LocalSymbolId::new("ImportedStruct/0"),
        FullyQualifiedName::new("pkg::imported::ImportedStruct/0"),
    );
    let imported_structure_bridge = SourceReserveDeclarationBridge::new(
        source,
        module,
        range(source, 0, 17),
        vec![SourceReserveBindingInput::new(
            "x",
            range(source, 0, 1),
            range(source, 4, 17),
            "ImportedStruct",
            TypeHeadInput::Symbol(imported_structure.clone()),
        )],
    )
    .expect("imported structure payload shape should validate before SymbolEnv checks");
    assert!(
        imported_structure_bridge
            .check(&symbol_env_with_imported_symbol(
                imported_structure,
                SymbolKind::Structure,
                "ImportedStruct",
            ))
            .is_err(),
        "imported structure source heads must remain fail-closed at the checker seam"
    );
}

#[test]
fn reserved_shadowing_and_missing_payloads_are_partial_with_ranges() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let binding_env = binding_env_for_declarations(
        source,
        vec![
            binding_spec(
                "reserved_y",
                BindingKind::ReservedVariable,
                BindingStatus::Reserved,
            ),
            binding_spec(
                "reserved_z",
                BindingKind::ReservedVariable,
                BindingStatus::Reserved,
            ),
            binding_spec(
                "reserved_bad",
                BindingKind::ReservedVariable,
                BindingStatus::Reserved,
            ),
            binding_spec("let_x", BindingKind::LetBinding, BindingStatus::Active),
        ],
    );
    let contexts = vec![
        DeclarationContextInput::new(BindingContextId::new(0), site(199), range(source, 0, 1)),
        DeclarationContextInput::new(BindingContextId::new(1), site(200), range(source, 1, 2)),
    ];
    let shadowed = declaration_with_type_in_context(
        source,
        0,
        BindingContextId::new(0),
        DeclarationKind::ReservedVariable,
        10,
        100,
    )
    .with_reserved_default(ReservedDefaultPayload::new(site(100), true));
    let missing = DeclarationInput::new(
        BindingId::new(3),
        BindingContextId::new(1),
        site(11),
        range(source, 40, 45),
        DeclarationKind::Let,
    );
    let missing_reserved_default = DeclarationInput::new(
        BindingId::new(1),
        BindingContextId::new(0),
        site(12),
        range(source, 60, 65),
        DeclarationKind::ReservedVariable,
    );
    let mismatched_default = declaration_with_type_in_context(
        source,
        2,
        BindingContextId::new(0),
        DeclarationKind::ReservedVariable,
        13,
        103,
    )
    .with_reserved_default(ReservedDefaultPayload::new(site(999), false));

    let output = DeclarationChecker::default().check(
        &symbols,
        &binding_env,
        contexts,
        [
            shadowed,
            missing,
            missing_reserved_default,
            mismatched_default,
        ],
    );

    assert_eq!(output.normalized_types().len(), 1);
    assert_eq!(output.declarations().len(), 4);
    assert!(
        output
            .declarations()
            .iter()
            .all(|(_, declaration)| { declaration.status == DeclarationStatus::Partial })
    );
    let declarations = declarations_by_binding(&output);
    let shadowed_declaration = declarations.get(&BindingId::new(0)).copied().unwrap();
    assert_eq!(shadowed_declaration.type_site, None);
    let shadowed_entry = output
        .type_entries()
        .get(shadowed_declaration.type_entry.unwrap())
        .unwrap();
    assert_eq!(shadowed_entry.owner, site(10));
    assert_eq!(shadowed_entry.actual, TypeEntryActual::Absent);
    assert_eq!(shadowed_entry.status, TypeStatus::Unknown);
    assert_eq!(
        diagnostic_ranges(&output, "checker.declaration.reserved_default_shadowed"),
        vec![(50, 55)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.declaration.deferred.type_payload"),
        vec![(40, 45)]
    );
    assert_eq!(
        diagnostic_ranges(
            &output,
            "checker.declaration.deferred.reserved_default_payload"
        ),
        vec![(60, 65)]
    );
    assert_eq!(
        diagnostic_ranges(
            &output,
            "checker.declaration.reserved_default_type_site_mismatch"
        ),
        vec![(65, 70)]
    );
    let mismatch_declaration = declarations_by_binding(&output)
        .get(&BindingId::new(2))
        .copied()
        .unwrap();
    assert_eq!(mismatch_declaration.type_site, Some(site(103)));
    let mismatch_type_entry = output
        .type_entries()
        .get(mismatch_declaration.type_entry.unwrap())
        .unwrap();
    assert!(matches!(
        mismatch_type_entry.actual,
        TypeEntryActual::Known(_)
    ));
    assert_eq!(mismatch_type_entry.status, TypeStatus::Unknown);
}

#[test]
fn invalid_declarations_keep_partial_output_and_deterministic_diagnostics() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let binding_env = binding_env_for_declarations(
        source,
        vec![
            binding_spec("let_x", BindingKind::LetBinding, BindingStatus::Active),
            binding_spec(
                "set_a",
                BindingKind::LocalAbbreviation,
                BindingStatus::Omitted,
            ),
            binding_spec("let_y", BindingKind::LetBinding, BindingStatus::Active),
        ],
    );
    let contexts = vec![DeclarationContextInput::new(
        BindingContextId::new(1),
        site(200),
        range(source, 0, 1),
    )];
    let declarations = vec![
        declaration_with_type(source, 0, DeclarationKind::Set, 10, 100),
        declaration_with_type(source, 1, DeclarationKind::Set, 11, 101),
        declaration_with_type(source, 99, DeclarationKind::Let, 12, 102).with_assumptions(vec![
            DeclarationAssumptionInput::new(
                TypePredicateRef::new("bad_assumption"),
                range(source, 60, 65),
            ),
        ]),
        DeclarationInput::new(
            BindingId::new(0),
            BindingContextId::new(99),
            site(10),
            range(source, 60, 65),
            DeclarationKind::Let,
        ),
        declaration_with_type_in_context(
            source,
            2,
            BindingContextId::new(0),
            DeclarationKind::Let,
            13,
            103,
        ),
    ];

    let output =
        DeclarationChecker::default().check(&symbols, &binding_env, contexts, declarations);

    assert_eq!(output.declarations().len(), 5);
    assert!(output.type_entries().len() >= 4);
    let declarations = declarations_by_binding(&output);
    assert_eq!(
        declarations.get(&BindingId::new(0)).unwrap().status,
        DeclarationStatus::Partial
    );
    assert_eq!(
        declarations.get(&BindingId::new(1)).unwrap().status,
        DeclarationStatus::Partial
    );
    assert_eq!(
        declarations.get(&BindingId::new(99)).unwrap().status,
        DeclarationStatus::Error
    );
    for declaration in declarations.values().copied() {
        let type_entry = output
            .type_entries()
            .get(declaration.type_entry.unwrap())
            .unwrap();
        assert_eq!(type_entry.owner, declaration.site);
        if declaration.status == DeclarationStatus::Error {
            assert_eq!(type_entry.status, TypeStatus::Error);
        }
    }
    let unknown_context_declaration = output
        .declarations()
        .iter()
        .map(|(_, declaration)| declaration)
        .find(|declaration| {
            declaration.binding == BindingId::new(0)
                && declaration.context == BindingContextId::new(99)
                && declaration.site == site(10)
                && declaration.kind == DeclarationKind::Let
        })
        .unwrap();
    assert_eq!(unknown_context_declaration.status, DeclarationStatus::Error);
    let unknown_context_type_entry = output
        .type_entries()
        .get(unknown_context_declaration.type_entry.unwrap())
        .unwrap();
    assert_eq!(unknown_context_type_entry.actual, TypeEntryActual::Absent);
    assert_eq!(unknown_context_type_entry.status, TypeStatus::Error);
    let duplicate_set_declaration = output
        .declarations()
        .iter()
        .map(|(_, declaration)| declaration)
        .find(|declaration| {
            declaration.binding == BindingId::new(0)
                && declaration.context == BindingContextId::new(1)
                && declaration.site == site(10)
                && declaration.kind == DeclarationKind::Set
        })
        .unwrap();
    assert_eq!(duplicate_set_declaration.status, DeclarationStatus::Partial);
    let duplicate_set_type_entry = output
        .type_entries()
        .get(duplicate_set_declaration.type_entry.unwrap())
        .unwrap();
    assert!(matches!(
        duplicate_set_type_entry.actual,
        TypeEntryActual::Known(_)
    ));
    assert_eq!(duplicate_set_type_entry.status, TypeStatus::Unknown);
    let context_mismatch_declaration = output
        .declarations()
        .iter()
        .map(|(_, declaration)| declaration)
        .find(|declaration| {
            declaration.binding == BindingId::new(2)
                && declaration.context == BindingContextId::new(0)
                && declaration.site == site(13)
                && declaration.kind == DeclarationKind::Let
        })
        .unwrap();
    assert_eq!(
        context_mismatch_declaration.status,
        DeclarationStatus::Partial
    );
    let context_mismatch_type_entry = output
        .type_entries()
        .get(context_mismatch_declaration.type_entry.unwrap())
        .unwrap();
    assert!(matches!(
        context_mismatch_type_entry.actual,
        TypeEntryActual::Known(_)
    ));
    assert_eq!(context_mismatch_type_entry.status, TypeStatus::Unknown);
    assert!(output.facts().is_empty());
    for expected in [
        "checker.declaration.illegal_binding_kind",
        "checker.declaration.omitted_binding",
        "checker.declaration.unknown_binding",
        "checker.declaration.unknown_context",
        "checker.declaration.binding_context_mismatch",
        "checker.declaration.duplicate_site",
        "checker.declaration.duplicate_binding",
        "checker.declaration.assumption_dropped_after_recovery",
    ] {
        assert!(output.debug_text().contains(expected), "{expected}");
    }
    assert_eq!(
        diagnostic_ranges(&output, "checker.declaration.unknown_context"),
        vec![(60, 65)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.declaration.illegal_binding_kind"),
        vec![(0, 5)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.declaration.omitted_binding"),
        vec![(10, 15)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.declaration.unknown_binding"),
        vec![(60, 65)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.declaration.binding_context_mismatch"),
        vec![(60, 65), (65, 70)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.declaration.duplicate_site"),
        vec![(50, 55)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.declaration.duplicate_binding"),
        vec![(50, 55)]
    );
    assert_eq!(
        diagnostic_ranges(
            &output,
            "checker.declaration.assumption_dropped_after_recovery"
        ),
        vec![(60, 65)]
    );
    assert_eq!(output.contexts().len(), 1);
}

#[test]
fn constrained_declarations_defer_obligations_without_fabricating_facts() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let binding_env = binding_env_for_declarations(
        source,
        vec![binding_spec(
            "let_x",
            BindingKind::LetBinding,
            BindingStatus::Active,
        )],
    );
    let declaration = declaration_with_type(source, 0, DeclarationKind::Let, 10, 100)
        .with_deferred(vec![DeclarationDeferredReason::MissingEvidenceQuery])
        .with_assumptions(vec![DeclarationAssumptionInput::new(
            TypePredicateRef::new("partial_assumption"),
            range(source, 50, 55),
        )]);

    let output = DeclarationChecker::default().check(
        &symbols,
        &binding_env,
        [DeclarationContextInput::new(
            BindingContextId::new(1),
            site(200),
            range(source, 0, 1),
        )],
        [declaration],
    );

    assert!(output.facts().is_empty());
    assert!(
        output
            .debug_text()
            .contains("checker.declaration.deferred.evidence_query")
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.declaration.deferred.evidence_query"),
        vec![(50, 55)]
    );
    assert_eq!(
        diagnostic_ranges(
            &output,
            "checker.declaration.assumption_dropped_after_recovery"
        ),
        vec![(50, 55)]
    );
    assert!(output.debug_text().contains("status=partial"));
    assert!(!output.debug_text().contains("obligation#"));
    assert!(!output.debug_text().contains("registration"));
}

#[test]
fn set_declaration_missing_rhs_payload_is_partial_without_fabricated_type() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let binding_env = binding_env_for_declarations(
        source,
        vec![binding_spec(
            "set_a",
            BindingKind::LocalAbbreviation,
            BindingStatus::Active,
        )],
    );
    let declaration = DeclarationInput::new(
        BindingId::new(0),
        BindingContextId::new(1),
        site(10),
        range(source, 50, 55),
        DeclarationKind::Set,
    )
    .with_deferred(vec![DeclarationDeferredReason::MissingRightHandSidePayload]);

    let output = DeclarationChecker::default().check(
        &symbols,
        &binding_env,
        [DeclarationContextInput::new(
            BindingContextId::new(1),
            site(200),
            range(source, 0, 1),
        )],
        [declaration],
    );

    let declaration = output.declarations().iter().next().unwrap().1;
    assert_eq!(declaration.status, DeclarationStatus::Partial);
    assert!(
        declaration
            .deferred
            .contains(&DeclarationDeferredReason::MissingRightHandSidePayload)
    );
    assert_eq!(declaration.type_site, None);
    let type_entry = output
        .type_entries()
        .get(declaration.type_entry.unwrap())
        .unwrap();
    assert_eq!(type_entry.owner, site(10));
    assert_eq!(type_entry.actual, TypeEntryActual::Absent);
    assert_eq!(type_entry.status, TypeStatus::Unknown);
    assert!(output.facts().is_empty());
    assert_eq!(
        diagnostic_ranges(&output, "checker.declaration.deferred.rhs_payload"),
        vec![(50, 55)]
    );
    assert!(!output.debug_text().contains("obligation#"));
    assert!(!output.debug_text().contains("registration"));
}

#[test]
fn attributed_declaration_defers_evidence_without_losing_normalized_type() {
    let source = source_id();
    let attr = symbol_id("Attr/1", "pkg::main::Attr/1");
    let symbols = symbol_env(vec![symbol_entry(attr.clone(), SymbolKind::Attribute)]);
    let binding_env = binding_env_for_declarations(
        source,
        vec![binding_spec(
            "let_x",
            BindingKind::LetBinding,
            BindingStatus::Active,
        )],
    );
    let declaration = DeclarationInput::new(
        BindingId::new(0),
        BindingContextId::new(1),
        site(10),
        range(source, 50, 55),
        DeclarationKind::Let,
    )
    .with_type_expression(
        TypeExpressionInput::new(
            site(100),
            range(source, 60, 70),
            "set attr",
            TypeHeadInput::BuiltinSet,
        )
        .with_attributes(vec![AttributeInput::new(
            attr.clone(),
            AttributePolarity::Positive,
            range(source, 64, 68),
            "Attr",
        )]),
    )
    .with_deferred(vec![DeclarationDeferredReason::MissingEvidenceQuery]);

    let output = DeclarationChecker::default().check(
        &symbols,
        &binding_env,
        [DeclarationContextInput::new(
            BindingContextId::new(1),
            site(200),
            range(source, 0, 1),
        )],
        [declaration],
    );

    let declaration = output.declarations().iter().next().unwrap().1;
    assert_eq!(declaration.status, DeclarationStatus::Partial);
    let type_entry = output
        .type_entries()
        .get(declaration.type_entry.unwrap())
        .unwrap();
    assert_eq!(type_entry.owner, site(10));
    assert_eq!(type_entry.status, TypeStatus::Unknown);
    let TypeEntryActual::Known(normalized_id) = type_entry.actual else {
        panic!("attributed declaration should keep its normalized type");
    };
    let normalized = output.normalized_types().get(normalized_id).unwrap();
    assert_eq!(normalized.status, NormalizedTypeStatus::Known);
    assert_eq!(normalized.attributes.positive().len(), 1);
    assert_eq!(normalized.attributes.positive()[0].symbol, attr);
    assert!(normalized.attributes.negative().is_empty());
    assert!(output.facts().is_empty());
    assert_eq!(
        diagnostic_ranges(&output, "checker.declaration.deferred.evidence_query"),
        vec![(50, 55)]
    );
    assert!(!output.debug_text().contains("obligation#"));
    assert!(!output.debug_text().contains("registration"));
}

#[test]
fn reconsider_existing_allows_additional_type_view_for_same_binding() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let binding_env = binding_env_for_declarations(
        source,
        vec![binding_spec(
            "let_x",
            BindingKind::LetBinding,
            BindingStatus::Active,
        )],
    );
    let original = declaration_with_type(source, 0, DeclarationKind::Let, 10, 100);
    let reconsider = declaration_with_type(source, 0, DeclarationKind::ReconsiderExisting, 11, 101);

    let output = DeclarationChecker::default().check(
        &symbols,
        &binding_env,
        [DeclarationContextInput::new(
            BindingContextId::new(1),
            site(200),
            range(source, 0, 1),
        )],
        [original, reconsider],
    );

    assert_eq!(output.declarations().len(), 2);
    assert_eq!(
        diagnostic_ranges(&output, "checker.declaration.duplicate_binding"),
        Vec::<(usize, usize)>::new()
    );
    let reconsider = output
        .declarations()
        .iter()
        .map(|(_, declaration)| declaration)
        .find(|declaration| declaration.kind == DeclarationKind::ReconsiderExisting)
        .unwrap();
    assert_eq!(reconsider.binding, BindingId::new(0));
    assert_eq!(reconsider.status, DeclarationStatus::Checked);
    let type_entry = output
        .type_entries()
        .get(reconsider.type_entry.unwrap())
        .unwrap();
    assert_eq!(type_entry.owner, site(11));
    assert_eq!(type_entry.status, TypeStatus::Known);
    assert!(matches!(type_entry.actual, TypeEntryActual::Known(_)));
}

#[test]
fn reconsider_existing_accepts_visible_outer_binding_from_child_context() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let binding_env = binding_env_with_outer_visible_binding(source);
    let reconsider = declaration_with_type_in_context(
        source,
        0,
        BindingContextId::new(1),
        DeclarationKind::ReconsiderExisting,
        11,
        101,
    );

    let output = DeclarationChecker::default().check(
        &symbols,
        &binding_env,
        [
            DeclarationContextInput::new(BindingContextId::new(0), site(199), range(source, 0, 1)),
            DeclarationContextInput::new(BindingContextId::new(1), site(200), range(source, 1, 2)),
        ],
        [reconsider],
    );

    assert_eq!(
        diagnostic_ranges(&output, "checker.declaration.binding_context_mismatch"),
        Vec::<(usize, usize)>::new()
    );
    let declaration = output.declarations().iter().next().unwrap().1;
    assert_eq!(declaration.binding, BindingId::new(0));
    assert_eq!(declaration.context, BindingContextId::new(1));
    assert_eq!(declaration.status, DeclarationStatus::Checked);
    let type_entry = output
        .type_entries()
        .get(declaration.type_entry.unwrap())
        .unwrap();
    assert_eq!(type_entry.owner, site(11));
    assert_eq!(type_entry.status, TypeStatus::Known);
    assert!(matches!(type_entry.actual, TypeEntryActual::Known(_)));
}

#[test]
fn term_inference_covers_term_kinds_and_open_candidates_deterministically() {
    let source = source_id();
    let functor = symbol_id("F/1", "pkg::main::F/1");
    let selector = symbol_id("sel", "pkg::main::sel");
    let structure = symbol_id("Struct", "pkg::main::Struct");
    let symbols = symbol_env(vec![
        symbol_entry(functor.clone(), SymbolKind::Functor),
        symbol_entry(selector.clone(), SymbolKind::Selector),
        symbol_entry(structure.clone(), SymbolKind::Structure),
    ]);
    let binding_env = binding_env_for_declarations(
        source,
        vec![binding_spec(
            "let_x",
            BindingKind::LetBinding,
            BindingStatus::Active,
        )],
    );
    let terms = vec![
        term_with_type(source, 10, TermKind::Variable, 100)
            .with_reference(TermReference::Binding(BindingId::new(0))),
        term_with_type(source, 11, TermKind::Numeral, 101),
        term_with_type(source, 12, TermKind::It, 102),
        TermInput::new(
            site(13),
            BindingContextId::new(1),
            range(source, 65, 70),
            TermKind::FunctorApplication,
        )
        .with_candidates(vec![
            OpenCandidateInput::new(
                CandidateIdentity::Builtin("zeta".to_owned()),
                range(source, 80, 81),
            )
            .with_result_type(type_expression(source, 103, TypeHeadInput::BuiltinSet)),
            OpenCandidateInput::new(
                CandidateIdentity::Builtin("alpha".to_owned()),
                range(source, 70, 71),
            )
            .with_result_type(type_expression(
                source,
                104,
                TypeHeadInput::BuiltinObject,
            )),
        ]),
        TermInput::new(
            site(14),
            BindingContextId::new(1),
            range(source, 70, 75),
            TermKind::SelectorAccess,
        )
        .with_candidates(vec![
            OpenCandidateInput::new(CandidateIdentity::Symbol(selector), range(source, 75, 76))
                .with_result_type(type_expression(source, 105, TypeHeadInput::BuiltinSet)),
        ]),
        term_with_type(source, 15, TermKind::StructureConstructor, 106).with_result_type(
            type_expression(source, 106, TypeHeadInput::Symbol(structure)),
        ),
        term_with_type(source, 16, TermKind::SetEnumeration, 107)
            .with_deferred(vec![TermDeferredReason::SethoodRequirement]),
        term_with_type(source, 17, TermKind::SetComprehension, 108)
            .with_deferred(vec![TermDeferredReason::SethoodRequirement]),
        term_with_type(source, 18, TermKind::Choice, 109)
            .with_deferred(vec![TermDeferredReason::NonEmptinessRequirement]),
        term_with_type(source, 19, TermKind::SourceQua, 110)
            .with_deferred(vec![TermDeferredReason::SourceQuaRequirement]),
        term_with_type(source, 20, TermKind::Parenthesized, 111),
    ];

    let first =
        TermFormulaChecker::default().infer(&symbols, &binding_env, terms.clone(), Vec::new());
    let second = TermFormulaChecker::default().infer(
        &symbols,
        &binding_env,
        terms.into_iter().rev(),
        Vec::new(),
    );

    assert_eq!(first.debug_text(), second.debug_text());
    assert_eq!(first.terms().len(), 11);
    assert_eq!(first.candidate_sets().len(), 2);
    let variable = term_by_site(&first, site(10));
    assert_eq!(variable.status, TermStatus::Inferred);
    assert!(matches!(
        first
            .type_entries()
            .get(variable.type_entry)
            .unwrap()
            .actual,
        TypeEntryActual::Known(_)
    ));
    let functor = term_by_site(&first, site(13));
    assert_eq!(functor.status, TermStatus::Partial);
    assert!(matches!(
        first.type_entries().get(functor.type_entry).unwrap().actual,
        TypeEntryActual::CandidateSet(_)
    ));
    let set_enum = term_by_site(&first, site(16));
    assert_eq!(
        set_enum.deferred,
        vec![TermDeferredReason::SethoodRequirement]
    );
    let choice = term_by_site(&first, site(18));
    assert_eq!(
        choice.deferred,
        vec![TermDeferredReason::NonEmptinessRequirement]
    );
    let source_qua = term_by_site(&first, site(19));
    assert_eq!(
        source_qua.deferred,
        vec![TermDeferredReason::SourceQuaRequirement]
    );
    let debug = first.debug_text();
    assert!(debug.contains("kind=selector_access"));
    assert!(debug.contains("kind=structure_constructor"));
    assert!(debug.contains("kind=parenthesized"));
    assert!(debug.contains("candidate_set#"));
    assert!(debug.find("builtin=\"alpha\"") < debug.find("builtin=\"zeta\""));
    assert!(!debug.contains("obligation#"));
    assert!(!debug.contains("registration"));
}

#[test]
fn step5c7_type_detail_keys_require_inference_before_bounded_evidence() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let binding_env = binding_env_for_declarations(source, Vec::new());
    let object_type = |node| {
        TypeExpressionInput::new(
            site(node),
            range(source, node * 5, node * 5 + 3),
            "object",
            TypeHeadInput::BuiltinObject,
        )
    };
    let numeral_terms = vec![
        TermInput::new(
            site(10),
            BindingContextId::new(0),
            range(source, 10, 11),
            TermKind::Numeral,
        )
        .with_result_type(object_type(10)),
        TermInput::new(
            site(11),
            BindingContextId::new(0),
            range(source, 11, 12),
            TermKind::Numeral,
        )
        .with_result_type(object_type(11)),
    ];
    let equality = FormulaInput::new(
        site(12),
        BindingContextId::new(0),
        range(source, 12, 13),
        FormulaKind::Equality,
    )
    .with_terms(vec![site(10), site(11)]);
    assert!(
        TermFormulaChecker::default()
            .step5c7_type_detail_keys(&symbols, &binding_env, numeral_terms, [equality], None, [],)
            .is_empty()
    );

    let choice_type_left = TypeExpressionInput::new(
        site(20),
        range(source, 20, 21),
        "set",
        TypeHeadInput::BuiltinSet,
    );
    let choice_type_right = TypeExpressionInput::new(
        site(24),
        range(source, 24, 25),
        "set",
        TypeHeadInput::BuiltinSet,
    );
    let choice_terms = vec![
        TermInput::new(
            site(21),
            BindingContextId::new(0),
            range(source, 21, 22),
            TermKind::Choice,
        )
        .with_result_type(choice_type_left),
        TermInput::new(
            site(22),
            BindingContextId::new(0),
            range(source, 22, 23),
            TermKind::Choice,
        )
        .with_result_type(choice_type_right),
    ];
    let choice_equality = FormulaInput::new(
        site(23),
        BindingContextId::new(0),
        range(source, 23, 24),
        FormulaKind::Equality,
    )
    .with_terms(vec![site(21), site(22)]);
    assert_eq!(
        TermFormulaChecker::default().step5c7_type_detail_keys(
            &symbols,
            &binding_env,
            choice_terms,
            [choice_equality],
            Some(false),
            [],
        ),
        vec!["terms.choice.missing_inhabitation".to_owned()]
    );
}

#[test]
fn step5c7_qua_requires_normalized_corresponding_inputs_and_exact_rejection() {
    let source = source_id();
    let structure = symbol_id("Box", "pkg::main::Box");
    let symbols = symbol_env(vec![symbol_entry(structure.clone(), SymbolKind::Structure)]);
    let bindings = binding_env_for_declarations(
        source,
        vec![binding_spec(
            "X",
            BindingKind::LetBinding,
            BindingStatus::Active,
        )],
    );
    let from = TypeExpressionInput::new(
        site(10),
        range(source, 20, 21),
        "set",
        TypeHeadInput::BuiltinSet,
    );
    let to = TypeExpressionInput::new(
        site(12),
        range(source, 26, 29),
        "Box",
        TypeHeadInput::Symbol(structure.clone()),
    );
    let base = TermInput::new(
        site(10),
        BindingContextId::new(1),
        range(source, 20, 21),
        TermKind::Variable,
    )
    .with_reference(TermReference::Binding(BindingId::new(0)))
    .with_result_type(from.clone());
    let qua = TermInput::new(
        site(11),
        BindingContextId::new(1),
        range(source, 20, 29),
        TermKind::SourceQua,
    )
    .with_result_type(to.clone());
    let terms = vec![base, qua];
    let input = CoercionInput::new(
        site(11),
        range(source, 20, 29),
        CoercionRequestKind::SourceQua,
        to,
    )
    .with_from_type(from);
    let run = |terms: Vec<TermInput>, inputs: Vec<CoercionInput>| {
        TermFormulaChecker::default().step5c7_type_detail_keys(
            &symbols,
            &bindings,
            terms,
            [],
            None,
            inputs,
        )
    };
    assert_eq!(
        run(terms.clone(), vec![input.clone()]),
        ["terms.qua.invalid_narrowing"]
    );
    let inferred = TermFormulaChecker::default().infer(&symbols, &bindings, terms.clone(), []);
    let checked =
        CoercionObligationChecker::default().check(&symbols, inferred.facts(), [input.clone()], []);
    let row = checked.coercions().iter().next().unwrap().1;
    assert_eq!(row.site, input.site);
    assert_eq!(row.kind, CoercionKind::SourceQua);
    assert_eq!(row.status, CoercionStatus::Rejected);
    assert_eq!(
        checked
            .normalized_types()
            .get(row.from.unwrap())
            .unwrap()
            .head,
        TypeHeadRef::BuiltinSet
    );
    assert_eq!(
        checked.normalized_types().get(row.to).unwrap().head,
        TypeHeadRef::Structure(structure)
    );
    let CoercionProvenance::Recovery(id) = row.provenance else {
        panic!("missing recovery provenance")
    };
    let diagnostic = checked.diagnostics().get(id).unwrap();
    assert_eq!(diagnostic.owner.as_ref(), Some(&input.site));
    assert_eq!(diagnostic.source_range, input.source_range);
    assert_eq!(
        diagnostic.message_key,
        "checker.coercion.invalid_source_qua_target"
    );
    for mutation in 0..12 {
        let mut changed = input.clone();
        let mut changed_terms = terms.clone();
        match mutation {
            0 => changed.from_type.as_mut().unwrap().head = TypeHeadInput::BuiltinObject,
            1 => changed.to_type.head = TypeHeadInput::BuiltinObject,
            2 => changed.site = site(99),
            3 => changed.from_type.as_mut().unwrap().site = site(99),
            4 => changed.to_type.site = site(99),
            5 => changed.source_range = range(source, 21, 29),
            6 => changed.evidence = CoercionEvidence::StaticUpcast,
            7 => changed.evidence = CoercionEvidence::StructureInheritance,
            8 => changed_terms[0].reference = Some(TermReference::Binding(BindingId::new(99))),
            9 => changed_terms[0].source_range = range(source, 10, 11),
            10 => changed_terms[0].context = BindingContextId::new(0),
            11 => changed_terms.push(
                TermInput::new(
                    site(90),
                    BindingContextId::new(1),
                    range(source, 90, 91),
                    TermKind::Numeral,
                )
                .with_result_type(TypeExpressionInput::new(
                    site(91),
                    range(source, 90, 91),
                    "missing",
                    TypeHeadInput::Symbol(symbol_id("Missing", "pkg::main::Missing")),
                )),
            ),
            _ => unreachable!(),
        }
        assert_eq!(
            run(changed_terms, vec![changed]),
            ["type_elaboration.checker.typed_ast_invalid"],
            "mutation {mutation}"
        );
    }
    assert_eq!(
        run(terms.clone(), vec![]),
        ["type_elaboration.checker.typed_ast_invalid"]
    );
    assert_eq!(
        run(terms.clone(), vec![input.clone(), input.clone()]),
        ["type_elaboration.checker.typed_ast_invalid"]
    );
    for (from, to) in [
        (TypeHeadInput::BuiltinSet, TypeHeadInput::BuiltinSet),
        (TypeHeadInput::BuiltinSet, TypeHeadInput::BuiltinObject),
        (TypeHeadInput::BuiltinObject, TypeHeadInput::BuiltinObject),
        (TypeHeadInput::BuiltinObject, TypeHeadInput::BuiltinSet),
    ] {
        let mut changed_terms = terms.clone();
        let mut changed = input.clone();
        changed_terms[0].result_type.as_mut().unwrap().head = from.clone();
        changed_terms[1].result_type.as_mut().unwrap().head = to.clone();
        changed.from_type.as_mut().unwrap().head = from.clone();
        changed.to_type.head = to.clone();
        let keys = run(changed_terms, vec![changed]);
        if from == TypeHeadInput::BuiltinObject && to == TypeHeadInput::BuiltinSet {
            assert_eq!(keys, ["type_elaboration.checker.typed_ast_invalid"]);
        } else {
            assert!(keys.is_empty(), "{from:?} -> {to:?}: {keys:?}");
        }
    }
}

#[test]
fn formula_inference_records_expected_constraints_facts_and_open_candidates() {
    let source = source_id();
    let predicate = symbol_id("P/1", "pkg::main::P/1");
    let attr = symbol_id("Attr/1", "pkg::main::Attr/1");
    let symbols = symbol_env(vec![
        symbol_entry(predicate.clone(), SymbolKind::Predicate),
        symbol_entry(attr.clone(), SymbolKind::Attribute),
    ]);
    let binding_env = binding_env_for_declarations(
        source,
        vec![binding_spec(
            "let_x",
            BindingKind::LetBinding,
            BindingStatus::Active,
        )],
    );
    let terms = vec![
        term_with_type(source, 10, TermKind::Variable, 100)
            .with_reference(TermReference::Binding(BindingId::new(0))),
        term_with_type(source, 11, TermKind::Numeral, 101),
    ];
    let formulas = vec![
        FormulaInput::new(
            site(30),
            BindingContextId::new(1),
            range(source, 150, 155),
            FormulaKind::Equality,
        )
        .with_terms(vec![site(10), site(11)])
        .with_expected_types(vec![
            ExpectedTypeInput::new(
                site(10),
                type_expression(source, 200, TypeHeadInput::BuiltinSet),
                range(source, 150, 151),
            ),
            ExpectedTypeInput::new(
                site(11),
                type_expression(source, 201, TypeHeadInput::BuiltinSet),
                range(source, 152, 153),
            ),
        ]),
        FormulaInput::new(
            site(31),
            BindingContextId::new(1),
            range(source, 155, 160),
            FormulaKind::Inequality,
        )
        .with_terms(vec![site(10), site(11)])
        .with_expected_types(vec![
            ExpectedTypeInput::new(
                site(10),
                type_expression(source, 205, TypeHeadInput::BuiltinSet),
                range(source, 155, 156),
            ),
            ExpectedTypeInput::new(
                site(11),
                type_expression(source, 206, TypeHeadInput::BuiltinSet),
                range(source, 157, 158),
            ),
        ]),
        FormulaInput::new(
            site(32),
            BindingContextId::new(1),
            range(source, 160, 165),
            FormulaKind::Membership,
        )
        .with_terms(vec![site(10), site(11)])
        .with_expected_types(vec![ExpectedTypeInput::new(
            site(11),
            type_expression(source, 202, TypeHeadInput::BuiltinSet),
            range(source, 162, 163),
        )]),
        FormulaInput::new(
            site(33),
            BindingContextId::new(1),
            range(source, 165, 170),
            FormulaKind::PredicateApplication,
        )
        .with_terms(vec![site(10)])
        .with_candidates(vec![
            OpenCandidateInput::new(
                CandidateIdentity::Builtin("z-pred".to_owned()),
                range(source, 166, 167),
            ),
            OpenCandidateInput::new(
                CandidateIdentity::Symbol(predicate),
                range(source, 165, 166),
            ),
        ]),
        FormulaInput::new(
            site(42),
            BindingContextId::new(1),
            range(source, 190, 195),
            FormulaKind::PredicateApplication,
        )
        .with_terms(vec![site(10)]),
        FormulaInput::new(
            site(34),
            BindingContextId::new(1),
            range(source, 170, 175),
            FormulaKind::TypeAssertion,
        )
        .with_terms(vec![site(10)])
        .with_asserted_type(type_expression(source, 203, TypeHeadInput::BuiltinSet))
        .with_facts(vec![FormulaFactInput::new(
            site(10),
            TypePredicateRef::new("type_asserted_set"),
            Polarity::Positive,
            range(source, 170, 175),
        )]),
        FormulaInput::new(
            site(35),
            BindingContextId::new(1),
            range(source, 175, 180),
            FormulaKind::AttributeAssertion,
        )
        .with_terms(vec![site(10)])
        .with_asserted_type(
            type_expression(source, 204, TypeHeadInput::BuiltinSet).with_attributes(vec![
                AttributeInput::new(
                    attr,
                    AttributePolarity::Positive,
                    range(source, 176, 177),
                    "Attr",
                ),
            ]),
        )
        .with_facts(vec![FormulaFactInput::new(
            site(10),
            TypePredicateRef::new("attr_asserted"),
            Polarity::Positive,
            range(source, 175, 180),
        )]),
        FormulaInput::new(
            site(43),
            BindingContextId::new(1),
            range(source, 195, 196),
            FormulaKind::Thesis,
        )
        .with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload]),
        FormulaInput::new(
            site(44),
            BindingContextId::new(1),
            range(source, 196, 197),
            FormulaKind::Contradiction,
        )
        .with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload]),
        FormulaInput::new(
            site(36),
            BindingContextId::new(1),
            range(source, 180, 181),
            FormulaKind::Negation,
        ),
        FormulaInput::new(
            site(37),
            BindingContextId::new(1),
            range(source, 181, 182),
            FormulaKind::Conjunction,
        ),
        FormulaInput::new(
            site(38),
            BindingContextId::new(1),
            range(source, 182, 183),
            FormulaKind::Disjunction,
        ),
        FormulaInput::new(
            site(39),
            BindingContextId::new(1),
            range(source, 183, 184),
            FormulaKind::Implication,
        ),
        FormulaInput::new(
            site(40),
            BindingContextId::new(1),
            range(source, 184, 185),
            FormulaKind::Biconditional,
        ),
        FormulaInput::new(
            site(41),
            BindingContextId::new(1),
            range(source, 185, 190),
            FormulaKind::Quantified,
        )
        .with_deferred(vec![FormulaDeferredReason::MissingQuantifierPayload]),
    ];

    let first = TermFormulaChecker::default().infer(
        &symbols,
        &binding_env,
        terms.clone(),
        formulas.clone(),
    );
    let second = TermFormulaChecker::default().infer(
        &symbols,
        &binding_env,
        terms.into_iter().rev(),
        formulas.into_iter().rev(),
    );
    assert_eq!(first.debug_text(), second.debug_text());
    let output = first;

    assert_eq!(output.formulas().len(), 15);
    assert_eq!(output.facts().len(), 2);
    let equality = formula_by_site(&output, site(30));
    assert_eq!(equality.expected_types.len(), 2);
    assert_eq!(equality.status, FormulaStatus::Checked);
    let inequality = formula_by_site(&output, site(31));
    assert_eq!(inequality.expected_types.len(), 2);
    assert_eq!(inequality.status, FormulaStatus::Checked);
    let membership = formula_by_site(&output, site(32));
    assert_eq!(membership.expected_types.len(), 1);
    assert_eq!(membership.expected_types[0].term, site(11));
    assert_eq!(membership.status, FormulaStatus::Checked);
    let predicate = formula_by_site(&output, site(33));
    assert_eq!(predicate.status, FormulaStatus::Partial);
    assert!(predicate.candidate_set.is_some());
    let missing_predicate_payload = formula_by_site(&output, site(42));
    assert_eq!(missing_predicate_payload.status, FormulaStatus::Partial);
    assert_eq!(
        diagnostic_ranges(
            &output,
            "checker.formula.external.predicate_signature_payload"
        ),
        vec![(190, 195)]
    );
    let type_assertion = formula_by_site(&output, site(34));
    assert_eq!(type_assertion.facts.len(), 1);
    let fact = output.facts().get(type_assertion.facts[0]).unwrap();
    assert_eq!(fact.status, FactStatus::Known);
    assert!(matches!(fact.provenance, FactProvenance::Inferred(_)));
    let quantified = formula_by_site(&output, site(41));
    assert_eq!(quantified.status, FormulaStatus::Partial);
    assert_eq!(
        diagnostic_ranges(&output, "checker.formula.external.quantifier_payload"),
        vec![(185, 190)]
    );
    for (formula, kind, range) in [
        (
            formula_by_site(&output, site(43)),
            FormulaKind::Thesis,
            (195, 196),
        ),
        (
            formula_by_site(&output, site(44)),
            FormulaKind::Contradiction,
            (196, 197),
        ),
    ] {
        assert_eq!(formula.kind, kind);
        assert_eq!(formula.status, FormulaStatus::Partial);
        assert_eq!(
            diagnostic_ranges(&output, "checker.formula.external.formula_payload")
                .into_iter()
                .filter(|diagnostic_range| *diagnostic_range == range)
                .collect::<Vec<_>>(),
            vec![range]
        );
    }
    let debug = output.debug_text();
    for expected in [
        "kind=thesis",
        "kind=contradiction",
        "kind=equality",
        "kind=inequality",
        "kind=membership",
        "kind=predicate_application",
        "kind=type_assertion",
        "kind=attribute_assertion",
        "kind=negation",
        "kind=conjunction",
        "kind=disjunction",
        "kind=implication",
        "kind=biconditional",
        "kind=quantified",
    ] {
        assert!(debug.contains(expected), "{expected}");
    }
    assert!(!debug.contains("obligation#"));
    assert!(!debug.contains("registration"));
}

#[test]
fn type_assertion_requires_reflexive_or_external_reachability_payload() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let binding_env = binding_env_for_declarations(
        source,
        vec![binding_spec(
            "let_x",
            BindingKind::LetBinding,
            BindingStatus::Active,
        )],
    );
    let subject = term_with_type(source, 10, TermKind::Variable, 100)
        .with_reference(TermReference::Binding(BindingId::new(0)));
    let compatible = FormulaInput::new(
        site(30),
        BindingContextId::new(1),
        range(source, 150, 155),
        FormulaKind::TypeAssertion,
    )
    .with_terms(vec![site(10)])
    .with_asserted_type(type_expression(source, 200, TypeHeadInput::BuiltinSet));
    let incompatible = FormulaInput::new(
        site(31),
        BindingContextId::new(1),
        range(source, 155, 160),
        FormulaKind::TypeAssertion,
    )
    .with_terms(vec![site(10)])
    .with_asserted_type(TypeExpressionInput::new(
        site(201),
        range(source, 160, 163),
        "object",
        TypeHeadInput::BuiltinObject,
    ));
    let missing_asserted = FormulaInput::new(
        site(32),
        BindingContextId::new(1),
        range(source, 165, 170),
        FormulaKind::TypeAssertion,
    )
    .with_terms(vec![site(10)]);
    let missing_subject = FormulaInput::new(
        site(33),
        BindingContextId::new(1),
        range(source, 170, 175),
        FormulaKind::TypeAssertion,
    )
    .with_asserted_type(type_expression(source, 202, TypeHeadInput::BuiltinSet));
    let multiple_subjects = FormulaInput::new(
        site(34),
        BindingContextId::new(1),
        range(source, 175, 180),
        FormulaKind::TypeAssertion,
    )
    .with_terms(vec![site(10), site(10)])
    .with_asserted_type(type_expression(source, 203, TypeHeadInput::BuiltinSet));
    let incompatible_with_other_deferred = FormulaInput::new(
        site(35),
        BindingContextId::new(1),
        range(source, 180, 185),
        FormulaKind::TypeAssertion,
    )
    .with_terms(vec![site(10)])
    .with_asserted_type(TypeExpressionInput::new(
        site(204),
        range(source, 185, 188),
        "object",
        TypeHeadInput::BuiltinObject,
    ))
    .with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload]);

    let output = TermFormulaChecker::default().infer(
        &symbols,
        &binding_env,
        [subject],
        [
            compatible,
            incompatible,
            missing_asserted,
            missing_subject,
            multiple_subjects,
            incompatible_with_other_deferred,
        ],
    );

    let compatible = formula_by_site(&output, site(30));
    assert_eq!(compatible.status, FormulaStatus::Checked);
    assert!(compatible.deferred.is_empty());
    assert!(compatible.facts.is_empty());

    let incompatible = formula_by_site(&output, site(31));
    assert_eq!(incompatible.status, FormulaStatus::Partial);
    assert_eq!(
        incompatible.deferred,
        vec![FormulaDeferredReason::MissingTypeAssertionReachabilityPayload]
    );
    assert_eq!(
        diagnostic_ranges(
            &output,
            "checker.formula.external.type_assertion_reachability_payload"
        ),
        vec![(155, 160), (180, 185)]
    );

    assert_eq!(
        formula_by_site(&output, site(32)).status,
        FormulaStatus::Partial
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.formula.missing_asserted_type"),
        vec![(165, 170)]
    );
    for formula_site in [site(33), site(34)] {
        assert_eq!(
            formula_by_site(&output, formula_site).status,
            FormulaStatus::Error
        );
    }
    assert_eq!(
        diagnostic_ranges(&output, "checker.formula.type_assertion.subject_arity"),
        vec![(170, 175), (175, 180)]
    );
    assert_eq!(
        formula_by_site(&output, site(35)).deferred,
        vec![
            FormulaDeferredReason::MissingTypeAssertionReachabilityPayload,
            FormulaDeferredReason::MissingFormulaPayload,
        ]
    );
    assert!(
        output
            .debug_text()
            .contains("missing_type_assertion_reachability_payload")
    );
}

#[test]
fn term_formula_recovery_keeps_partial_entries_without_successful_types() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let binding_env = binding_env_for_declarations(
        source,
        vec![binding_spec(
            "let_x",
            BindingKind::LetBinding,
            BindingStatus::Active,
        )],
    );
    let terms = vec![
        TermInput::new(
            site(50),
            BindingContextId::new(1),
            range(source, 250, 255),
            TermKind::It,
        ),
        TermInput::new(
            site(51),
            BindingContextId::new(1),
            range(source, 255, 260),
            TermKind::Unsupported,
        ),
        TermInput::new(
            site(52),
            BindingContextId::new(1),
            range(source, 260, 265),
            TermKind::Variable,
        )
        .with_reference(TermReference::Binding(BindingId::new(99))),
        TermInput::new(
            site(53),
            BindingContextId::new(1),
            range(source, 265, 270),
            TermKind::Numeral,
        ),
        TermInput::new(
            site(54),
            BindingContextId::new(1),
            range(source, 270, 275),
            TermKind::Variable,
        )
        .with_reference(TermReference::Binding(BindingId::new(0)))
        .with_result_type(type_expression(
            source,
            120,
            TypeHeadInput::Unresolved("MissingMode".to_owned()),
        )),
    ];
    let formulas = vec![
        FormulaInput::new(
            site(60),
            BindingContextId::new(1),
            range(source, 300, 305),
            FormulaKind::Unsupported,
        ),
        FormulaInput::new(
            site(61),
            BindingContextId::new(1),
            range(source, 305, 310),
            FormulaKind::Equality,
        )
        .with_terms(vec![site(50), site(99)]),
        FormulaInput::new(
            site(62),
            BindingContextId::new(1),
            range(source, 310, 315),
            FormulaKind::Membership,
        )
        .with_terms(vec![site(51)]),
        FormulaInput::new(
            site(63),
            BindingContextId::new(1),
            range(source, 315, 320),
            FormulaKind::Inequality,
        )
        .with_terms(vec![site(53)]),
        FormulaInput::new(
            site(64),
            BindingContextId::new(1),
            range(source, 320, 325),
            FormulaKind::TypeAssertion,
        )
        .with_terms(vec![site(54)])
        .with_facts(vec![FormulaFactInput::new(
            site(54),
            TypePredicateRef::new("degraded_term_fact"),
            Polarity::Positive,
            range(source, 320, 325),
        )]),
    ];

    let output = TermFormulaChecker::default().infer(&symbols, &binding_env, terms, formulas);

    let it = term_by_site(&output, site(50));
    assert_eq!(it.status, TermStatus::Error);
    let it_entry = output.type_entries().get(it.type_entry).unwrap();
    assert_eq!(it_entry.actual, TypeEntryActual::Absent);
    assert_eq!(it_entry.status, TypeStatus::Error);
    let unsupported = term_by_site(&output, site(51));
    assert_eq!(unsupported.status, TermStatus::Skipped);
    assert_eq!(
        output
            .type_entries()
            .get(unsupported.type_entry)
            .unwrap()
            .status,
        TypeStatus::Skipped
    );
    let unknown_binding = term_by_site(&output, site(52));
    assert_eq!(unknown_binding.status, TermStatus::Error);
    let numeral = term_by_site(&output, site(53));
    assert_eq!(numeral.status, TermStatus::Partial);
    assert_eq!(
        output
            .type_entries()
            .get(numeral.type_entry)
            .unwrap()
            .actual,
        TypeEntryActual::Absent
    );
    let degraded_type_term = term_by_site(&output, site(54));
    assert_eq!(degraded_type_term.status, TermStatus::Inferred);
    assert_eq!(
        output
            .type_entries()
            .get(degraded_type_term.type_entry)
            .unwrap()
            .status,
        TypeStatus::Unknown
    );
    assert_eq!(
        formula_by_site(&output, site(60)).status,
        FormulaStatus::Skipped
    );
    assert_eq!(
        formula_by_site(&output, site(61)).status,
        FormulaStatus::Error
    );
    assert_eq!(
        formula_by_site(&output, site(62)).status,
        FormulaStatus::Error
    );
    assert_eq!(
        formula_by_site(&output, site(63)).status,
        FormulaStatus::Partial
    );
    let degraded_fact_formula = formula_by_site(&output, site(64));
    assert_eq!(degraded_fact_formula.status, FormulaStatus::Partial);
    assert_eq!(degraded_fact_formula.facts.len(), 1);
    assert_eq!(
        output
            .facts()
            .get(degraded_fact_formula.facts[0])
            .unwrap()
            .status,
        FactStatus::Degraded
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.formula.term.not_well_formed"),
        vec![(305, 310), (310, 315)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.formula.term.missing"),
        vec![(305, 310)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.formula.term.partial"),
        vec![(315, 320), (320, 325)]
    );
    for key in [
        "checker.term.it.missing_current_result_type",
        "checker.term.unsupported_payload",
        "checker.term.reference.unknown_binding",
        "checker.term.external.numeric_type_payload",
        "checker.formula.unsupported_payload",
        "checker.formula.term.not_well_formed",
        "checker.formula.term.missing",
        "checker.formula.term.partial",
    ] {
        assert!(output.debug_text().contains(key), "{key}");
    }
    assert_eq!(output.facts().len(), 1);
    assert!(
        !output
            .debug_text()
            .contains("status=Known actual=normalized_type")
    );
}

#[test]
fn coercion_checker_records_candidates_and_initial_obligations_deterministically() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let mut input_facts = TypeFactTable::new();
    let known_fact = input_facts.insert(TypeFactDraft {
        subject: site(90),
        predicate: TypePredicateRef::new("known_widening_support"),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Inferred(TypeRuleId::new("fixture.known")),
        status: FactStatus::Known,
    });
    let coercions = vec![
        CoercionInput::new(
            site(12),
            range(source, 60, 65),
            CoercionRequestKind::Narrowing,
            type_expression(source, 112, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(source, 212, TypeHeadInput::BuiltinObject))
        .with_supporting_facts(vec![known_fact])
        .with_obligation(
            InitialObligationInput::new(
                site(12),
                range(source, 60, 65),
                InitialRequirementKind::Narrowing,
                type_expression(source, 312, TypeHeadInput::BuiltinSet),
            )
            .with_assumptions(vec![known_fact])
            .with_goal(InitialObligationGoal::new("reconsider.existing"))
            .with_provenance(InitialObligationProvenance::new("reconsider-existing")),
        ),
        CoercionInput::new(
            site(13),
            range(source, 65, 70),
            CoercionRequestKind::Narrowing,
            type_expression(source, 113, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(source, 213, TypeHeadInput::BuiltinObject))
        .with_obligation(
            InitialObligationInput::new(
                site(13),
                range(source, 65, 70),
                InitialRequirementKind::Narrowing,
                type_expression(source, 313, TypeHeadInput::BuiltinSet),
            )
            .with_goal(InitialObligationGoal::new("reconsider.new"))
            .with_provenance(InitialObligationProvenance::new("reconsider-new")),
        ),
        CoercionInput::new(
            site(10),
            range(source, 50, 55),
            CoercionRequestKind::SourceQua,
            type_expression(source, 110, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(source, 210, TypeHeadInput::BuiltinObject))
        .with_evidence(CoercionEvidence::StaticUpcast),
        CoercionInput::new(
            site(14),
            range(source, 70, 75),
            CoercionRequestKind::SourceQua,
            type_expression(source, 114, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(source, 214, TypeHeadInput::BuiltinObject))
        .with_evidence(CoercionEvidence::CompatibleView),
        CoercionInput::new(
            site(11),
            range(source, 55, 60),
            CoercionRequestKind::Widening,
            type_expression(source, 111, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(source, 211, TypeHeadInput::BuiltinObject))
        .with_evidence(CoercionEvidence::KnownFacts)
        .with_supporting_facts(vec![known_fact]),
        CoercionInput::new(
            site(15),
            range(source, 75, 80),
            CoercionRequestKind::Widening,
            type_expression(source, 115, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(source, 215, TypeHeadInput::BuiltinObject))
        .with_evidence(CoercionEvidence::BuiltinRadix),
        CoercionInput::new(
            site(16),
            range(source, 80, 85),
            CoercionRequestKind::Widening,
            type_expression(source, 116, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(source, 216, TypeHeadInput::BuiltinObject))
        .with_evidence(CoercionEvidence::StructureInheritance)
        .with_supporting_facts(vec![known_fact]),
        CoercionInput::new(
            site(17),
            range(source, 85, 90),
            CoercionRequestKind::Widening,
            type_expression(source, 117, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(source, 217, TypeHeadInput::BuiltinObject))
        .with_evidence(CoercionEvidence::ActivatedSummary)
        .with_supporting_facts(vec![known_fact]),
        CoercionInput::new(
            site(18),
            range(source, 100, 105),
            CoercionRequestKind::Narrowing,
            type_expression(source, 118, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(source, 218, TypeHeadInput::BuiltinObject))
        .with_evidence(CoercionEvidence::KnownFacts)
        .with_supporting_facts(vec![known_fact]),
    ];
    let obligations = vec![
        InitialObligationInput::new(
            site(21),
            range(source, 95, 100),
            InitialRequirementKind::NonEmptiness,
            type_expression(source, 121, TypeHeadInput::BuiltinSet),
        )
        .with_goal(InitialObligationGoal::new("choice.non_empty"))
        .with_provenance(InitialObligationProvenance::new("choice-term")),
        InitialObligationInput::new(
            site(20),
            range(source, 90, 95),
            InitialRequirementKind::Sethood,
            type_expression(source, 120, TypeHeadInput::BuiltinSet),
        )
        .with_assumptions(vec![known_fact]),
    ];

    let first = CoercionObligationChecker::default().check(
        &symbols,
        &input_facts,
        coercions.clone(),
        obligations.clone(),
    );
    let second = CoercionObligationChecker::default().check(
        &symbols,
        &input_facts,
        coercions.into_iter().rev(),
        obligations.into_iter().rev(),
    );
    assert_eq!(first.debug_text(), second.debug_text());

    assert_eq!(first.coercions().len(), 9);
    assert_eq!(first.initial_obligations().len(), 4);
    assert_eq!(first.facts().len(), 6);
    assert_eq!(
        first.facts().get(known_fact).unwrap().status,
        FactStatus::Known
    );
    let coercion_at = |site_ref: TypedSiteRef| {
        first
            .coercions()
            .iter()
            .map(|(_, coercion)| coercion)
            .find(|coercion| coercion.site == site_ref)
            .unwrap()
    };
    let widening = coercion_at(site(11));
    assert_eq!(widening.kind, CoercionKind::Widening);
    assert_eq!(widening.status, CoercionStatus::Candidate);
    assert_eq!(widening.supporting_facts, vec![known_fact]);
    let builtin_widening = coercion_at(site(15));
    assert_eq!(builtin_widening.kind, CoercionKind::Widening);
    assert_eq!(builtin_widening.status, CoercionStatus::Candidate);
    assert_eq!(builtin_widening.supporting_facts.len(), 1);
    let builtin_fact = first
        .facts()
        .get(builtin_widening.supporting_facts[0])
        .unwrap();
    assert_eq!(builtin_fact.status, FactStatus::Known);
    assert!(matches!(
        builtin_fact.provenance,
        FactProvenance::Builtin(_)
    ));
    for widening_site in [site(16), site(17)] {
        let widening = coercion_at(widening_site);
        assert_eq!(widening.kind, CoercionKind::Widening);
        assert_eq!(widening.status, CoercionStatus::Candidate);
        assert_eq!(widening.supporting_facts, vec![known_fact]);
    }
    let source_qua = coercion_at(site(10));
    assert_eq!(source_qua.kind, CoercionKind::SourceQua);
    assert_eq!(source_qua.status, CoercionStatus::Candidate);
    let compatible_qua = coercion_at(site(14));
    assert_eq!(compatible_qua.kind, CoercionKind::SourceQua);
    assert_eq!(compatible_qua.status, CoercionStatus::Candidate);

    let obligation_at = |site_ref: TypedSiteRef, kind: InitialObligationKind| {
        first
            .initial_obligations()
            .iter()
            .map(|(_, obligation)| obligation)
            .find(|obligation| obligation.owner == site_ref && obligation.kind == kind)
            .unwrap()
    };
    let existing_narrowing = coercion_at(site(12));
    assert_eq!(existing_narrowing.kind, CoercionKind::Narrowing);
    assert_eq!(
        existing_narrowing.status,
        CoercionStatus::RequiresObligation
    );
    let existing_obligation_id = existing_narrowing.obligation.unwrap();
    let existing_obligation = first
        .initial_obligations()
        .get(existing_obligation_id)
        .unwrap();
    assert_eq!(existing_obligation.kind, InitialObligationKind::Narrowing);
    assert_eq!(existing_obligation.source_range, range(source, 60, 65));
    assert_eq!(existing_obligation.assumptions, vec![known_fact]);
    assert_eq!(existing_obligation.goal.as_str(), "reconsider.existing");
    assert_eq!(
        existing_obligation.provenance.as_str(),
        "reconsider-existing"
    );
    assert_eq!(existing_obligation.status, InitialObligationStatus::Pending);
    let new_narrowing = coercion_at(site(13));
    assert_eq!(new_narrowing.kind, CoercionKind::Narrowing);
    assert_eq!(new_narrowing.status, CoercionStatus::RequiresObligation);
    let new_obligation_id = new_narrowing.obligation.unwrap();
    let new_obligation = first.initial_obligations().get(new_obligation_id).unwrap();
    assert_eq!(new_obligation.kind, InitialObligationKind::Narrowing);
    assert_eq!(new_obligation.source_range, range(source, 65, 70));
    assert!(new_obligation.assumptions.is_empty());
    assert_eq!(new_obligation.goal.as_str(), "reconsider.new");
    assert_eq!(new_obligation.provenance.as_str(), "reconsider-new");
    assert_eq!(new_obligation.status, InitialObligationStatus::Pending);
    let supported_narrowing = coercion_at(site(18));
    assert_eq!(supported_narrowing.kind, CoercionKind::Narrowing);
    assert_eq!(supported_narrowing.status, CoercionStatus::Candidate);
    assert_eq!(supported_narrowing.supporting_facts, vec![known_fact]);
    assert_eq!(supported_narrowing.obligation, None);

    let sethood = obligation_at(site(20), InitialObligationKind::Sethood);
    assert_eq!(
        sethood.id,
        first.initial_obligations().get(sethood.id).unwrap().id
    );
    assert_eq!(sethood.source_range, range(source, 90, 95));
    assert_eq!(sethood.assumptions, vec![known_fact]);
    let non_empty = obligation_at(site(21), InitialObligationKind::NonEmptiness);
    assert_eq!(
        non_empty.id,
        first.initial_obligations().get(non_empty.id).unwrap().id
    );
    assert_eq!(non_empty.source_range, range(source, 95, 100));
    assert!(non_empty.assumptions.is_empty());
    for (_, obligation) in first.initial_obligations().iter() {
        let fact = first
            .facts()
            .iter()
            .map(|(_, fact)| fact)
            .find(|fact| {
                matches!(
                    &fact.provenance,
                    FactProvenance::Obligation(obligation_id)
                        if *obligation_id == obligation.id
                )
            })
            .unwrap();
        assert_eq!(fact.subject, obligation.owner);
        assert_eq!(fact.status, FactStatus::PendingObligation);
    }
    assert_eq!(
        first
            .initial_obligations()
            .iter()
            .map(|(_, obligation)| obligation.kind)
            .collect::<Vec<_>>(),
        vec![
            InitialObligationKind::Narrowing,
            InitialObligationKind::Narrowing,
            InitialObligationKind::Sethood,
            InitialObligationKind::NonEmptiness,
        ]
    );
    let debug = first.debug_text();
    assert!(debug.contains("coercion-checking-debug-v1"));
    assert!(debug.contains("kind=widening status=candidate"));
    assert!(debug.contains("kind=source_qua status=candidate"));
    assert!(debug.contains("kind=narrowing status=requires_obligation"));
    assert!(debug.contains("kind=sethood"));
    assert!(debug.contains("kind=non_emptiness"));
    assert!(debug.contains("widening_rule=\"coercion-widening-builtin_radix\""));
    assert!(debug.contains("widening_rule=\"coercion-widening-structure_inheritance\""));
    assert!(debug.contains("widening_rule=\"coercion-widening-activated_summary\""));
    assert!(debug.contains("source_qua:70..75"));
    assert!(debug.contains("goal=\"reconsider.existing\""));
    assert!(debug.contains("goal=\"reconsider.new\""));
    assert!(debug.contains("obligation#"));
    assert!(!debug.contains(concat!("V", "cId")));
    assert!(!debug.contains(concat!("Proof", "Witness")));
    assert!(!debug.contains("ObligationAnchor"));
}

#[test]
fn omitted_reconsider_accepts_only_proof_free_consumable_evidence() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let mut input_facts = TypeFactTable::new();
    let known_fact = input_facts.insert(TypeFactDraft {
        subject: site(90),
        predicate: TypePredicateRef::new("proof_free_reconsider_support"),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Inferred(TypeRuleId::new("fixture.known")),
        status: FactStatus::Known,
    });
    let degraded_fact = input_facts.insert(TypeFactDraft {
        subject: site(91),
        predicate: TypePredicateRef::new("not_consumable_reconsider_support"),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Inferred(TypeRuleId::new("fixture.degraded")),
        status: FactStatus::Degraded,
    });

    let accepted_evidence = [
        (30, CoercionEvidence::KnownFacts),
        (31, CoercionEvidence::BuiltinRadix),
        (32, CoercionEvidence::StructureInheritance),
        (33, CoercionEvidence::ActivatedSummary),
        (34, CoercionEvidence::StaticUpcast),
        (35, CoercionEvidence::CompatibleView),
    ]
    .into_iter()
    .map(|(offset, evidence)| {
        omitted_reconsider_input(source, offset, evidence).with_supporting_facts(vec![known_fact])
    });
    let rejected_evidence = [
        omitted_reconsider_input(source, 40, CoercionEvidence::ActivatedSummary),
        omitted_reconsider_input(source, 41, CoercionEvidence::BuiltinRadix),
        omitted_reconsider_input(source, 42, CoercionEvidence::StaticUpcast),
        omitted_reconsider_input(source, 43, CoercionEvidence::CompatibleView)
            .with_supporting_facts(vec![degraded_fact]),
        omitted_reconsider_input(source, 44, CoercionEvidence::Missing)
            .with_supporting_facts(vec![known_fact]),
    ];
    let output = CoercionObligationChecker::default().check(
        &symbols,
        &input_facts,
        accepted_evidence
            .chain(rejected_evidence)
            .collect::<Vec<_>>(),
        Vec::new(),
    );

    assert!(output.initial_obligations().is_empty());
    let coercion_at = |site_ref: TypedSiteRef| {
        output
            .coercions()
            .iter()
            .map(|(_, coercion)| coercion)
            .find(|coercion| coercion.site == site_ref)
            .unwrap()
    };
    for offset in 30..=35 {
        let coercion = coercion_at(site(offset));
        assert_eq!(coercion.kind, CoercionKind::Narrowing);
        assert_eq!(coercion.status, CoercionStatus::Candidate);
        assert_eq!(coercion.supporting_facts, vec![known_fact]);
        assert_eq!(coercion.obligation, None);
    }
    for offset in 40..=44 {
        let coercion = coercion_at(site(offset));
        assert_eq!(coercion.kind, CoercionKind::Narrowing);
        assert_eq!(coercion.status, CoercionStatus::Rejected);
        assert_eq!(coercion.obligation, None);
    }
    assert_eq!(
        diagnostic_ranges(&output, "type.narrowing_requires_proof"),
        vec![(200, 205), (205, 210), (210, 215), (215, 220), (220, 225)]
    );
    let debug = output.debug_text();
    assert!(debug.contains("type.narrowing_requires_proof"));
    assert!(debug.contains("checker.coercion.supporting_fact_not_consumable"));
    assert!(!debug.contains("kind=narrowing status=requires_obligation"));
}

#[test]
fn explicit_narrowing_preserves_task_ten_obligation_path() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let mut input_facts = TypeFactTable::new();
    let known_fact = input_facts.insert(TypeFactDraft {
        subject: site(92),
        predicate: TypePredicateRef::new("explicit_narrowing_support"),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Inferred(TypeRuleId::new("fixture.explicit")),
        status: FactStatus::Known,
    });
    let explicit_markers = [
        (50, CoercionEvidence::BuiltinRadix),
        (51, CoercionEvidence::StructureInheritance),
        (52, CoercionEvidence::ActivatedSummary),
        (53, CoercionEvidence::StaticUpcast),
        (54, CoercionEvidence::CompatibleView),
    ]
    .into_iter()
    .map(|(offset, evidence)| {
        CoercionInput::new(
            site(offset),
            range(source, offset * 5, offset * 5 + 5),
            CoercionRequestKind::Narrowing,
            type_expression(source, offset + 100, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(
            source,
            offset + 200,
            TypeHeadInput::BuiltinObject,
        ))
        .with_evidence(evidence)
        .with_supporting_facts(vec![known_fact])
    })
    .collect::<Vec<_>>();

    let output = CoercionObligationChecker::default().check(
        &symbols,
        &input_facts,
        explicit_markers,
        Vec::new(),
    );

    assert_eq!(output.initial_obligations().len(), 5);
    for offset in 50..=54 {
        let coercion = output
            .coercions()
            .iter()
            .map(|(_, coercion)| coercion)
            .find(|coercion| coercion.site == site(offset))
            .unwrap();
        assert_eq!(coercion.kind, CoercionKind::Narrowing);
        assert_eq!(coercion.status, CoercionStatus::RequiresObligation);
        let obligation = output
            .initial_obligations()
            .get(coercion.obligation.unwrap())
            .unwrap();
        assert_eq!(obligation.kind, InitialObligationKind::Narrowing);
        assert_eq!(obligation.assumptions, vec![known_fact]);
    }
    assert!(diagnostic_ranges(&output, "type.narrowing_requires_proof").is_empty());
}

#[test]
fn coercion_checker_blocks_missing_evidence_and_invalid_qua_without_fabrication() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let mut input_facts = TypeFactTable::new();
    let degraded_fact = input_facts.insert(TypeFactDraft {
        subject: site(91),
        predicate: TypePredicateRef::new("degraded_support"),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Inferred(TypeRuleId::new("fixture.degraded")),
        status: FactStatus::Degraded,
    });
    let known_fact = input_facts.insert(TypeFactDraft {
        subject: site(92),
        predicate: TypePredicateRef::new("known_qua_is_not_qua_evidence"),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Inferred(TypeRuleId::new("fixture.known_qua")),
        status: FactStatus::Known,
    });
    input_facts.insert(TypeFactDraft {
        subject: site(34),
        predicate: TypePredicateRef::new("coercion.widening.builtin_radix"),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Builtin(BuiltinRuleId::new("coercion-widening-builtin-radix")),
        status: FactStatus::Degraded,
    });
    let coercions = vec![
        CoercionInput::new(
            site(30),
            range(source, 150, 155),
            CoercionRequestKind::Widening,
            type_expression(source, 130, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(source, 230, TypeHeadInput::BuiltinObject))
        .with_deferred(vec![CoercionDeferredReason::MissingWideningSupportPayload]),
        CoercionInput::new(
            site(31),
            range(source, 155, 160),
            CoercionRequestKind::SourceQua,
            type_expression(source, 131, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(source, 231, TypeHeadInput::BuiltinObject)),
        CoercionInput::new(
            site(32),
            range(source, 160, 165),
            CoercionRequestKind::Widening,
            type_expression(source, 132, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(source, 232, TypeHeadInput::BuiltinObject))
        .with_evidence(CoercionEvidence::KnownFacts)
        .with_supporting_facts(vec![degraded_fact]),
        CoercionInput::new(
            site(33),
            range(source, 165, 170),
            CoercionRequestKind::Widening,
            type_expression(source, 133, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(source, 233, TypeHeadInput::BuiltinObject))
        .with_evidence(CoercionEvidence::StructureInheritance),
        CoercionInput::new(
            site(34),
            range(source, 170, 175),
            CoercionRequestKind::Widening,
            type_expression(source, 134, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(source, 234, TypeHeadInput::BuiltinObject))
        .with_evidence(CoercionEvidence::BuiltinRadix),
        CoercionInput::new(
            site(35),
            range(source, 175, 180),
            CoercionRequestKind::SourceQua,
            type_expression(source, 135, TypeHeadInput::BuiltinSet),
        )
        .with_from_type(type_expression(source, 235, TypeHeadInput::BuiltinObject))
        .with_evidence(CoercionEvidence::KnownFacts)
        .with_supporting_facts(vec![known_fact]),
    ];
    let obligations = vec![
        InitialObligationInput::new(
            site(40),
            range(source, 200, 205),
            InitialRequirementKind::Sethood,
            type_expression(source, 140, TypeHeadInput::BuiltinSet),
        )
        .with_assumptions(vec![degraded_fact]),
        InitialObligationInput::new(
            site(41),
            range(source, 205, 210),
            InitialRequirementKind::NonEmptiness,
            type_expression(source, 141, TypeHeadInput::BuiltinSet),
        )
        .with_assumptions(vec![TypeFactId::new(99)]),
    ];

    let output = CoercionObligationChecker::default().check(
        &symbols,
        &input_facts,
        coercions.clone(),
        obligations.clone(),
    );
    let reversed = CoercionObligationChecker::default().check(
        &symbols,
        &input_facts,
        coercions.into_iter().rev(),
        obligations.into_iter().rev(),
    );
    assert_eq!(output.debug_text(), reversed.debug_text());

    assert_eq!(
        output
            .coercions()
            .iter()
            .map(|(_, coercion)| (coercion.site.clone(), coercion.status))
            .collect::<Vec<_>>(),
        vec![
            (site(30), CoercionStatus::Blocked),
            (site(31), CoercionStatus::Rejected),
            (site(32), CoercionStatus::Blocked),
            (site(33), CoercionStatus::Blocked),
            (site(34), CoercionStatus::Blocked),
            (site(35), CoercionStatus::Rejected),
        ]
    );
    assert!(
        output
            .initial_obligations()
            .iter()
            .all(|(_, obligation)| obligation.status == InitialObligationStatus::Blocked)
    );
    assert_eq!(
        diagnostic_ranges(
            &output,
            "checker.coercion.external.widening_support_payload"
        ),
        vec![(150, 155), (160, 165), (165, 170)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.coercion.invalid_source_qua_target"),
        vec![(155, 160), (175, 180)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.coercion.supporting_fact_not_consumable"),
        vec![(160, 165)]
    );
    assert_eq!(
        diagnostic_ranges(
            &output,
            "checker.coercion.builtin_widening_fact_not_consumable"
        ),
        vec![(170, 175)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.obligation.assumption_not_consumable"),
        vec![(200, 205)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.obligation.unknown_assumption"),
        vec![(205, 210)]
    );
    let debug = output.debug_text();
    assert!(debug.contains("kind=widening status=blocked"));
    assert!(debug.contains("kind=source_qua status=rejected"));
    assert!(debug.contains("kind=sethood"));
    assert!(debug.contains("kind=non_emptiness"));
    assert!(debug.contains("status=blocked"));
    assert!(!debug.contains("accepted_verifier_status"));
    assert!(!debug.contains(concat!("inserted", "_qua")));
}

#[test]
fn coercion_checker_preserves_alternate_same_site_kind_candidates() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let mut input_facts = TypeFactTable::new();
    let support = input_facts.insert(TypeFactDraft {
        subject: site(92),
        predicate: TypePredicateRef::new("alternate_widening_support"),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Inferred(TypeRuleId::new("fixture.alternate")),
        status: FactStatus::Known,
    });
    let first_input = CoercionInput::new(
        site(50),
        range(source, 250, 255),
        CoercionRequestKind::Widening,
        type_expression(source, 150, TypeHeadInput::BuiltinSet),
    )
    .with_from_type(type_expression(source, 250, TypeHeadInput::BuiltinObject))
    .with_evidence(CoercionEvidence::KnownFacts)
    .with_supporting_facts(vec![support]);
    let second_input = CoercionInput::new(
        site(50),
        range(source, 250, 255),
        CoercionRequestKind::Widening,
        TypeExpressionInput::new(
            site(151),
            range(source, 252, 255),
            "object",
            TypeHeadInput::BuiltinObject,
        ),
    )
    .with_from_type(type_expression(source, 251, TypeHeadInput::BuiltinObject))
    .with_evidence(CoercionEvidence::BuiltinRadix);

    let first = CoercionObligationChecker::default().check(
        &symbols,
        &input_facts,
        [first_input.clone(), second_input.clone()],
        Vec::new(),
    );
    let second = CoercionObligationChecker::default().check(
        &symbols,
        &input_facts,
        [second_input, first_input],
        Vec::new(),
    );

    assert_eq!(first.debug_text(), second.debug_text());
    assert_eq!(first.coercions().len(), 2);
    assert!(
        first
            .coercions()
            .iter()
            .all(|(_, coercion)| coercion.site == site(50)
                && coercion.kind == CoercionKind::Widening
                && coercion.status == CoercionStatus::Candidate)
    );
    assert_eq!(
        diagnostic_ranges(&first, "checker.coercion.duplicate_site"),
        Vec::<(usize, usize)>::new()
    );
}

#[test]
fn type_fact_queries_are_deterministic_and_ignore_provenance_for_matching() {
    let source = source_id();
    let subject = site(70);
    let predicate = TypePredicateRef::new("is_set_like");
    let mut first_facts = TypeFactTable::new();
    let inferred = first_facts.insert(TypeFactDraft {
        subject: subject.clone(),
        predicate: predicate.clone(),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Inferred(TypeRuleId::new("z-rule")),
        status: FactStatus::Known,
    });
    let builtin = first_facts.insert(TypeFactDraft {
        subject: subject.clone(),
        predicate: predicate.clone(),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Builtin(BuiltinRuleId::new("a-builtin")),
        status: FactStatus::Known,
    });
    let duplicate = first_facts.insert(TypeFactDraft {
        subject: subject.clone(),
        predicate: predicate.clone(),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Builtin(BuiltinRuleId::new("a-builtin")),
        status: FactStatus::Known,
    });
    assert_eq!(duplicate, builtin);
    assert_eq!(first_facts.len(), 2);

    let mut second_facts = TypeFactTable::new();
    second_facts.insert(TypeFactDraft {
        subject: subject.clone(),
        predicate: predicate.clone(),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Builtin(BuiltinRuleId::new("a-builtin")),
        status: FactStatus::Known,
    });
    second_facts.insert(TypeFactDraft {
        subject: subject.clone(),
        predicate: predicate.clone(),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Inferred(TypeRuleId::new("z-rule")),
        status: FactStatus::Known,
    });

    let query = TypeFactQuery::new(
        subject.clone(),
        predicate.clone(),
        Polarity::Positive,
        range(source, 350, 355),
    );
    let first = TypeFactQueryEngine::new(&first_facts).query(query.clone());
    let second = TypeFactQueryEngine::new(&second_facts).query(query);

    assert_eq!(first.status(), TypeFactQueryStatus::Satisfied);
    assert_eq!(first.matched_facts(), &[inferred, builtin]);
    assert_eq!(second.status(), TypeFactQueryStatus::Satisfied);
    assert_eq!(
        second.matched_facts(),
        &[TypeFactId::new(1), TypeFactId::new(0)]
    );
    assert!(first.debug_text().contains("status=satisfied"));
    assert!(first.debug_text().contains("matched=[fact#0, fact#1]"));
    assert!(first.diagnostics().is_empty());
    assert!(!first.debug_text().contains("registration"));
    assert!(!first.debug_text().contains(concat!("Proof", "Witness")));
}

#[test]
fn type_fact_queries_respect_assumption_visibility_and_context_absence() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let binding_env = binding_env_for_declarations(
        source,
        vec![
            binding_spec(
                "reserved_y",
                BindingKind::ReservedVariable,
                BindingStatus::Active,
            ),
            binding_spec(
                "given_w",
                BindingKind::QuantifierBinder,
                BindingStatus::Active,
            ),
        ],
    );
    let output = DeclarationChecker::default().check(
        &symbols,
        &binding_env,
        vec![
            DeclarationContextInput::new(BindingContextId::new(0), site(300), range(source, 0, 1)),
            DeclarationContextInput::new(BindingContextId::new(1), site(301), range(source, 1, 2)),
        ],
        vec![
            declaration_with_type_in_context(
                source,
                0,
                BindingContextId::new(0),
                DeclarationKind::ReservedVariable,
                80,
                180,
            )
            .with_reserved_default(ReservedDefaultPayload::new(site(180), false))
            .with_assumptions(vec![DeclarationAssumptionInput::new(
                TypePredicateRef::new("module_assumption"),
                range(source, 350, 351),
            )]),
            declaration_with_type_in_context(
                source,
                1,
                BindingContextId::new(1),
                DeclarationKind::Given,
                81,
                181,
            )
            .with_assumptions(vec![DeclarationAssumptionInput::new(
                TypePredicateRef::new("block_assumption"),
                range(source, 351, 352),
            )]),
        ],
    );
    let module_query = TypeFactQuery::new(
        site(80),
        TypePredicateRef::new("module_assumption"),
        Polarity::Positive,
        range(source, 360, 361),
    );
    let block_query = TypeFactQuery::new(
        site(81),
        TypePredicateRef::new("block_assumption"),
        Polarity::Positive,
        range(source, 361, 362),
    );
    let contextless = TypeFactQueryEngine::new(output.facts()).query(module_query.clone());
    let no_query_context = TypeFactQueryEngine::with_contexts(output.facts(), output.contexts())
        .query(module_query.clone());
    let module_context = TypeFactQueryEngine::with_contexts(output.facts(), output.contexts())
        .query(
            module_query
                .clone()
                .with_context(LocalTypeContextId::new(0)),
        );
    let child_context = TypeFactQueryEngine::with_contexts(output.facts(), output.contexts())
        .query(module_query.with_context(LocalTypeContextId::new(1)));
    let block_from_parent = TypeFactQueryEngine::with_contexts(output.facts(), output.contexts())
        .query(block_query.clone().with_context(LocalTypeContextId::new(0)));
    let block_from_child = TypeFactQueryEngine::with_contexts(output.facts(), output.contexts())
        .query(block_query.with_context(LocalTypeContextId::new(1)));

    assert_eq!(contextless.status(), TypeFactQueryStatus::Missing);
    assert_eq!(no_query_context.status(), TypeFactQueryStatus::Missing);
    assert_eq!(module_context.status(), TypeFactQueryStatus::Satisfied);
    assert_eq!(child_context.status(), TypeFactQueryStatus::Satisfied);
    assert_eq!(block_from_parent.status(), TypeFactQueryStatus::Missing);
    assert_eq!(block_from_child.status(), TypeFactQueryStatus::Satisfied);
    assert_eq!(
        TypeFactQueryEngine::with_contexts(output.facts(), output.contexts())
            .active_facts(Some(LocalTypeContextId::new(1))),
        vec![TypeFactId::new(0), TypeFactId::new(1)]
    );
    assert_eq!(
        TypeFactQueryEngine::with_contexts(output.facts(), output.contexts()).active_facts(None),
        Vec::<TypeFactId>::new()
    );
    assert_eq!(
        TypeFactQueryEngine::new(output.facts()).active_facts(Some(LocalTypeContextId::new(1))),
        Vec::<TypeFactId>::new()
    );
    assert_eq!(
        TypeFactQueryEngine::with_contexts(output.facts(), output.contexts())
            .active_facts(Some(LocalTypeContextId::new(99))),
        Vec::<TypeFactId>::new()
    );

    let mut hidden_facts = TypeFactTable::new();
    let hidden_fact = hidden_facts.insert(TypeFactDraft {
        subject: site(82),
        predicate: TypePredicateRef::new("hidden_assumption"),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Inferred(TypeRuleId::new("hidden")),
        status: FactStatus::Assumed,
    });
    let mut hidden_contexts = LocalTypeContextTable::new();
    let parent = hidden_contexts.insert(LocalTypeContextDraft {
        owner: site(302),
        parent: None,
        layer: TypeContextLayer::Block,
        bindings: Vec::new(),
        introduced_assumptions: vec![hidden_fact],
        visible_facts: vec![hidden_fact],
        recovery: ContextRecoveryState::Normal,
    });
    let hidden_child = hidden_contexts.insert(LocalTypeContextDraft {
        owner: site(303),
        parent: Some(parent),
        layer: TypeContextLayer::Block,
        bindings: Vec::new(),
        introduced_assumptions: Vec::new(),
        visible_facts: Vec::new(),
        recovery: ContextRecoveryState::Normal,
    });
    let hidden_query = TypeFactQuery::new(
        site(82),
        TypePredicateRef::new("hidden_assumption"),
        Polarity::Positive,
        range(source, 362, 363),
    );
    let hidden_engine = TypeFactQueryEngine::with_contexts(&hidden_facts, &hidden_contexts);

    assert_eq!(
        hidden_engine
            .query(hidden_query.clone().with_context(parent))
            .status(),
        TypeFactQueryStatus::Satisfied
    );
    assert_eq!(
        hidden_engine
            .query(hidden_query.with_context(hidden_child))
            .status(),
        TypeFactQueryStatus::Missing
    );
    assert_eq!(hidden_engine.active_facts(Some(parent)), vec![hidden_fact]);
    assert_eq!(
        hidden_engine.active_facts(Some(hidden_child)),
        Vec::<TypeFactId>::new()
    );

    let mut filtered_facts = TypeFactTable::new();
    let visible_positive = filtered_facts.insert(TypeFactDraft {
        subject: site(83),
        predicate: TypePredicateRef::new("visibility_filtered_opposite"),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Inferred(TypeRuleId::new("visible")),
        status: FactStatus::Known,
    });
    let hidden_negative = filtered_facts.insert(TypeFactDraft {
        subject: site(83),
        predicate: TypePredicateRef::new("visibility_filtered_opposite"),
        polarity: Polarity::Negative,
        provenance: FactProvenance::Inferred(TypeRuleId::new("hidden-opposite")),
        status: FactStatus::Assumed,
    });
    let mut filtered_contexts = LocalTypeContextTable::new();
    let contradiction_parent = filtered_contexts.insert(LocalTypeContextDraft {
        owner: site(304),
        parent: None,
        layer: TypeContextLayer::Block,
        bindings: Vec::new(),
        introduced_assumptions: vec![hidden_negative],
        visible_facts: vec![hidden_negative],
        recovery: ContextRecoveryState::Normal,
    });
    let filtered_child = filtered_contexts.insert(LocalTypeContextDraft {
        owner: site(305),
        parent: Some(contradiction_parent),
        layer: TypeContextLayer::Block,
        bindings: Vec::new(),
        introduced_assumptions: Vec::new(),
        visible_facts: Vec::new(),
        recovery: ContextRecoveryState::Normal,
    });
    let filtered_query = TypeFactQuery::new(
        site(83),
        TypePredicateRef::new("visibility_filtered_opposite"),
        Polarity::Positive,
        range(source, 363, 364),
    );
    let filtered_engine = TypeFactQueryEngine::with_contexts(&filtered_facts, &filtered_contexts);

    assert_eq!(
        filtered_engine
            .query(filtered_query.clone().with_context(contradiction_parent))
            .status(),
        TypeFactQueryStatus::Contradicted
    );
    assert_eq!(
        filtered_engine
            .query(filtered_query.with_context(filtered_child))
            .status(),
        TypeFactQueryStatus::Satisfied
    );
    assert_eq!(
        filtered_engine.active_facts(Some(filtered_child)),
        vec![visible_positive]
    );
}

#[test]
fn type_fact_queries_report_contradictions_without_mutating_facts() {
    let source = source_id();
    let subject = site(90);
    let predicate = TypePredicateRef::new("contradictory");
    let mut facts = TypeFactTable::new();
    let negative = facts.insert(TypeFactDraft {
        subject: subject.clone(),
        predicate: predicate.clone(),
        polarity: Polarity::Negative,
        provenance: FactProvenance::Builtin(BuiltinRuleId::new("negative")),
        status: FactStatus::Known,
    });
    let positive = facts.insert(TypeFactDraft {
        subject: subject.clone(),
        predicate: predicate.clone(),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Inferred(TypeRuleId::new("positive")),
        status: FactStatus::Known,
    });
    facts.insert(TypeFactDraft {
        subject: subject.clone(),
        predicate: predicate.clone(),
        polarity: Polarity::Negative,
        provenance: FactProvenance::Inferred(TypeRuleId::new("degraded-negative")),
        status: FactStatus::Degraded,
    });
    facts.insert(TypeFactDraft {
        subject: subject.clone(),
        predicate: predicate.clone(),
        polarity: Polarity::Positive,
        provenance: FactProvenance::Obligation(InitialObligationId::new(7)),
        status: FactStatus::PendingObligation,
    });
    facts.insert(TypeFactDraft {
        subject: subject.clone(),
        predicate: predicate.clone(),
        polarity: Polarity::Negative,
        provenance: FactProvenance::Inferred(TypeRuleId::new("rejected-negative")),
        status: FactStatus::Rejected,
    });
    let query = TypeFactQuery::new(
        subject.clone(),
        predicate.clone(),
        Polarity::Positive,
        range(source, 400, 405),
    );

    let output = TypeFactQueryEngine::new(&facts).query(query);
    let negative_output = TypeFactQueryEngine::new(&facts).query(TypeFactQuery::new(
        subject.clone(),
        predicate.clone(),
        Polarity::Negative,
        range(source, 405, 410),
    ));

    assert_eq!(output.status(), TypeFactQueryStatus::Contradicted);
    assert_eq!(output.matched_facts(), &[positive, negative]);
    assert_eq!(negative_output.status(), TypeFactQueryStatus::Contradicted);
    assert_eq!(negative_output.matched_facts(), &[positive, negative]);
    assert_eq!(facts.len(), 5);
    assert_eq!(
        TypeFactQueryEngine::new(&facts).active_facts(None),
        vec![positive, negative]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.fact.contradiction"),
        vec![(400, 405)]
    );
    assert!(output.debug_text().contains("status=contradicted"));
    assert!(output.debug_text().contains("class=TypeFact"));
}

#[test]
fn attributes_are_sorted_deduplicated_and_contradictions_are_diagnosed() {
    let source = source_id();
    let attr_a = symbol_id("AttrA/1", "pkg::main::AttrA/1");
    let attr_b = symbol_id("AttrB/1", "pkg::main::AttrB/1");
    let symbols = symbol_env(vec![
        symbol_entry(attr_b.clone(), SymbolKind::Attribute),
        symbol_entry(attr_a.clone(), SymbolKind::Attribute),
    ]);
    let input = TypeExpressionInput::new(
        site(0),
        range(source, 0, 10),
        "set attr",
        TypeHeadInput::BuiltinSet,
    )
    .with_attributes(vec![
        AttributeInput::new(
            attr_b.clone(),
            AttributePolarity::Positive,
            range(source, 8, 9),
            "B",
        ),
        AttributeInput::new(
            attr_a.clone(),
            AttributePolarity::Negative,
            range(source, 4, 5),
            "non A",
        ),
        AttributeInput::new(
            attr_a.clone(),
            AttributePolarity::Positive,
            range(source, 2, 3),
            "A",
        ),
        AttributeInput::new(
            attr_b,
            AttributePolarity::Positive,
            range(source, 6, 7),
            "B",
        ),
    ]);

    let output = TypeNormalizer::default().normalize(&symbols, [input]);
    let normalized = output
        .normalized_types()
        .get(NormalizedTypeId::new(0))
        .unwrap();

    assert_eq!(normalized.status, NormalizedTypeStatus::Degraded);
    assert_eq!(normalized.attributes.positive().len(), 2);
    assert_eq!(normalized.attributes.negative().len(), 1);
    assert_eq!(
        normalized
            .attributes
            .positive()
            .iter()
            .map(|attribute| attribute.symbol.local().as_str())
            .collect::<Vec<_>>(),
        vec!["AttrA/1", "AttrB/1"]
    );
    assert_eq!(output.diagnostics().len(), 2);
    assert!(
        output
            .debug_text()
            .contains("checker.type.contradictory_attribute")
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.type.contradictory_attribute"),
        vec![(0, 10)]
    );
}

#[test]
fn attribute_arguments_and_duplicate_ranges_are_canonicalized() {
    let source = source_id();
    let attr = symbol_id("Attr/1", "pkg::main::Attr/1");
    let symbols = symbol_env(vec![symbol_entry(attr.clone(), SymbolKind::Attribute)]);
    let input = TypeExpressionInput::new(
        site(0),
        range(source, 0, 50),
        "attributed set",
        TypeHeadInput::BuiltinSet,
    )
    .with_attributes(vec![
        AttributeInput::new(
            attr.clone(),
            AttributePolarity::Positive,
            range(source, 30, 35),
            "Attr set late",
        )
        .with_args(vec![TypeExpressionInput::new(
            site(1),
            range(source, 36, 39),
            "set",
            TypeHeadInput::BuiltinSet,
        )]),
        AttributeInput::new(
            attr.clone(),
            AttributePolarity::Positive,
            range(source, 20, 25),
            "Attr object",
        )
        .with_args(vec![TypeExpressionInput::new(
            site(2),
            range(source, 26, 32),
            "object",
            TypeHeadInput::BuiltinObject,
        )]),
        AttributeInput::new(
            attr.clone(),
            AttributePolarity::Positive,
            range(source, 10, 15),
            "Attr set early",
        )
        .with_args(vec![TypeExpressionInput::new(
            site(3),
            range(source, 16, 19),
            "set",
            TypeHeadInput::BuiltinSet,
        )]),
    ]);

    let output = TypeNormalizer::default().normalize(&symbols, [input]);
    let normalized = output
        .type_entries()
        .iter()
        .find_map(|(_, entry)| match entry.actual {
            TypeEntryActual::Known(id) if entry.owner == site(0) => {
                output.normalized_types().get(id)
            }
            _ => None,
        })
        .unwrap();

    assert_eq!(normalized.status, NormalizedTypeStatus::Known);
    assert_eq!(normalized.attributes.positive().len(), 2);
    assert_eq!(
        normalized
            .attributes
            .positive()
            .iter()
            .map(|attribute| attribute.args.clone())
            .collect::<Vec<_>>(),
        vec![
            vec![NormalizedTypeId::new(0)],
            vec![NormalizedTypeId::new(1)]
        ]
    );
    assert_eq!(normalized.attributes.positive()[1].source_range.start, 10);
    assert_eq!(normalized.attributes.positive()[1].source_range.end, 15);
}

#[test]
fn equivalent_inputs_have_order_independent_debug_rendering() {
    let (source, alternate_source) = source_ids_pair();
    let symbols = symbol_env(Vec::new());
    let known_later = TypeExpressionInput::new(
        site(2),
        range(source, 20, 23),
        "set shared",
        TypeHeadInput::BuiltinSet,
    );
    let known_earlier = TypeExpressionInput::new(
        site(1),
        range(alternate_source, 20, 23),
        "set shared",
        TypeHeadInput::BuiltinSet,
    );
    let bad_later = TypeExpressionInput::new(
        site(4),
        range(source, 40, 47),
        "missing later",
        TypeHeadInput::Unresolved("missing".to_owned()),
    );
    let bad_earlier = TypeExpressionInput::new(
        site(3),
        range(source, 30, 37),
        "missing earlier",
        TypeHeadInput::Unresolved("missing".to_owned()),
    );

    let first = TypeNormalizer::default().normalize(
        &symbols,
        [
            known_later.clone(),
            bad_later.clone(),
            known_earlier.clone(),
            bad_earlier.clone(),
        ],
    );
    let second = TypeNormalizer::default().normalize(
        &symbols,
        [bad_earlier, known_earlier, bad_later, known_later],
    );

    assert_eq!(first.debug_text(), second.debug_text());
    assert_eq!(first.normalized_types().len(), 2);
    assert!(first.debug_text().contains("source=\"set shared\""));
    assert!(first.debug_text().contains("error=unknown"));
    assert!(!first.debug_text().contains("error=diagnostic#"));
    assert_eq!(
        first
            .normalized_types()
            .get(NormalizedTypeId::new(0))
            .unwrap()
            .source
            .range
            .source_id,
        second
            .normalized_types()
            .get(NormalizedTypeId::new(0))
            .unwrap()
            .source
            .range
            .source_id
    );
}

#[test]
fn builtins_structures_and_recursive_arguments_have_deterministic_ids() {
    let source = source_id();
    let structure = symbol_id("Struct/0", "pkg::main::Struct/0");
    let mode = symbol_id("Mode/1", "pkg::main::Mode/1");
    let symbols = symbol_env(vec![
        symbol_entry(structure.clone(), SymbolKind::Structure),
        symbol_entry(mode.clone(), SymbolKind::Mode),
    ]);
    let arg = TypeExpressionInput::new(
        site(1),
        range(source, 2, 5),
        "set",
        TypeHeadInput::BuiltinSet,
    );
    let input = TypeExpressionInput::new(
        site(0),
        range(source, 0, 6),
        "Mode of set",
        TypeHeadInput::Symbol(mode.clone()),
    )
    .with_args(vec![arg]);
    let structure_input = TypeExpressionInput::new(
        site(2),
        range(source, 7, 13),
        "Struct",
        TypeHeadInput::Symbol(structure),
    );
    let object_input = TypeExpressionInput::new(
        site(3),
        range(source, 14, 20),
        "object",
        TypeHeadInput::BuiltinObject,
    );
    let mode_with_object_arg = TypeExpressionInput::new(
        site(4),
        range(source, 21, 35),
        "Mode of object",
        TypeHeadInput::Symbol(mode.clone()),
    )
    .with_args(vec![TypeExpressionInput::new(
        site(5),
        range(source, 29, 35),
        "object",
        TypeHeadInput::BuiltinObject,
    )]);

    let first = TypeNormalizer::default().normalize(
        &symbols,
        [
            input.clone(),
            structure_input.clone(),
            object_input.clone(),
            mode_with_object_arg.clone(),
        ],
    );
    let second = TypeNormalizer::default().normalize(
        &symbols,
        [input, structure_input, object_input, mode_with_object_arg],
    );

    assert_eq!(first.debug_text(), second.debug_text());
    assert_eq!(first.normalized_types().len(), 5);
    assert!(matches!(
        first
            .normalized_types()
            .get(NormalizedTypeId::new(0))
            .unwrap()
            .head,
        TypeHeadRef::BuiltinObject
    ));
    assert!(matches!(
        first
            .normalized_types()
            .get(NormalizedTypeId::new(1))
            .unwrap()
            .head,
        TypeHeadRef::BuiltinSet
    ));
    assert!(matches!(
        first.normalized_types().get(NormalizedTypeId::new(2)).unwrap().head,
        TypeHeadRef::Mode(ref actual) if actual == &mode
    ));
    assert!(
        first
            .normalized_types()
            .iter()
            .any(|(_, normalized)| matches!(normalized.head, TypeHeadRef::Structure(_)))
    );
    let mode_arg_sets = first
        .normalized_types()
        .iter()
        .filter_map(|(id, normalized)| match normalized.head {
            TypeHeadRef::Mode(ref actual) if actual == &mode => Some((id, &normalized.args)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(mode_arg_sets.len(), 2);
    assert_ne!(mode_arg_sets[0].0, mode_arg_sets[1].0);
    assert_ne!(mode_arg_sets[0].1, mode_arg_sets[1].1);
    assert!(
        mode_arg_sets
            .iter()
            .any(|(_, args)| args.as_slice() == [NormalizedTypeId::new(0)])
    );
    assert!(
        mode_arg_sets
            .iter()
            .any(|(_, args)| args.as_slice() == [NormalizedTypeId::new(1)])
    );
    assert_eq!(
        first
            .type_entries()
            .canonical_iter()
            .map(|(id, _)| id.index())
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert!(
        first
            .debug_text()
            .starts_with("type-normalization-debug-v1\n")
    );
    assert!(!first.debug_text().contains("SourceId"));
    assert!(!first.debug_text().contains(concat!("V", "cId")));
    assert!(!first.debug_text().contains(concat!("Proof", "Witness")));
    assert!(!first.debug_text().contains("accepted_verifier_status"));
    assert!(!first.debug_text().contains(concat!("inserted", "_qua")));
    assert!(
        !first
            .debug_text()
            .contains(concat!("active", "_refinement"))
    );
    assert!(!first.debug_text().contains(concat!("overload", "_root")));
}

#[test]
fn mode_expansion_provider_unfolds_radix_and_attributes_idempotently() {
    let source = source_id();
    let mode = symbol_id("Mode/0", "pkg::main::Mode/0");
    let attr = symbol_id("Expandable/0", "pkg::main::Expandable/0");
    let symbols = symbol_env(vec![
        symbol_entry(mode.clone(), SymbolKind::Mode),
        symbol_entry(attr.clone(), SymbolKind::Attribute),
    ]);
    let expansion = ModeExpansion::new(
        TypeExpressionInput::new(
            site(10),
            range(source, 20, 23),
            "set",
            TypeHeadInput::BuiltinSet,
        ),
        vec![AttributeInput::new(
            attr.clone(),
            AttributePolarity::Positive,
            range(source, 24, 25),
            "expandable",
        )],
    );
    let input = TypeExpressionInput::new(
        site(0),
        range(source, 0, 4),
        "Mode",
        TypeHeadInput::Symbol(mode.clone()),
    );
    let repeated = TypeExpressionInput::new(
        site(1),
        range(source, 5, 9),
        "Mode",
        TypeHeadInput::Symbol(mode.clone()),
    );

    let first = TypeNormalizer::new([(mode.clone(), expansion.clone())])
        .normalize(&symbols, [input.clone(), repeated.clone()]);
    let second = TypeNormalizer::new([(mode, expansion)]).normalize(&symbols, [input, repeated]);
    assert_eq!(first.debug_text(), second.debug_text());
    assert_eq!(first.normalized_types().len(), 2);
    assert_eq!(
        first
            .type_entries()
            .canonical_iter()
            .map(|(_, entry)| entry.actual)
            .collect::<Vec<_>>(),
        vec![
            TypeEntryActual::Known(NormalizedTypeId::new(1)),
            TypeEntryActual::Known(NormalizedTypeId::new(1)),
        ]
    );
    let expanded = first
        .normalized_types()
        .canonical_iter()
        .map(|(_, normalized)| normalized)
        .find(|normalized| !normalized.attributes.positive().is_empty())
        .unwrap();

    assert!(matches!(expanded.head, TypeHeadRef::BuiltinSet));
    assert_eq!(expanded.attributes.positive()[0].symbol, attr);
    assert_eq!(first.diagnostics().len(), 0);
}

#[test]
fn mode_expansion_preserves_radix_arguments_and_attributes() {
    let source = source_id();
    let outer = symbol_id("Outer/0", "pkg::main::Outer/0");
    let inner = symbol_id("Inner/1", "pkg::main::Inner/1");
    let base_attr = symbol_id("Base/0", "pkg::main::Base/0");
    let extra_attr = symbol_id("Extra/0", "pkg::main::Extra/0");
    let symbols = symbol_env(vec![
        symbol_entry(outer.clone(), SymbolKind::Mode),
        symbol_entry(inner.clone(), SymbolKind::Mode),
        symbol_entry(base_attr.clone(), SymbolKind::Attribute),
        symbol_entry(extra_attr.clone(), SymbolKind::Attribute),
    ]);
    let radix = TypeExpressionInput::new(
        site(10),
        range(source, 20, 30),
        "Inner of object",
        TypeHeadInput::Symbol(inner.clone()),
    )
    .with_args(vec![TypeExpressionInput::new(
        site(11),
        range(source, 29, 35),
        "object",
        TypeHeadInput::BuiltinObject,
    )])
    .with_attributes(vec![AttributeInput::new(
        base_attr.clone(),
        AttributePolarity::Positive,
        range(source, 36, 37),
        "base",
    )]);
    let expansion = ModeExpansion::new(
        radix,
        vec![AttributeInput::new(
            extra_attr.clone(),
            AttributePolarity::Positive,
            range(source, 38, 39),
            "extra",
        )],
    );
    let input = TypeExpressionInput::new(
        site(0),
        range(source, 0, 5),
        "Outer",
        TypeHeadInput::Symbol(outer.clone()),
    );

    let output = TypeNormalizer::new([(outer, expansion)]).normalize(&symbols, [input]);
    let expanded = output
        .type_entries()
        .iter()
        .find_map(|(_, entry)| match entry.actual {
            TypeEntryActual::Known(id) if entry.owner == site(0) => {
                output.normalized_types().get(id)
            }
            _ => None,
        })
        .unwrap();

    assert!(matches!(expanded.head, TypeHeadRef::Mode(ref actual) if actual == &inner));
    assert_eq!(expanded.args.len(), 1);
    assert_eq!(
        expanded
            .attributes
            .positive()
            .iter()
            .map(|attribute| attribute.symbol.local().as_str())
            .collect::<Vec<_>>(),
        vec!["Base/0", "Extra/0"]
    );
}

#[test]
fn mode_expansion_with_arguments_reports_wrong_mode_arity() {
    let source = source_id();
    let mode = symbol_id("Mode/0", "pkg::main::Mode/0");
    let symbols = symbol_env(vec![symbol_entry(mode.clone(), SymbolKind::Mode)]);
    let expansion = ModeExpansion::new(
        TypeExpressionInput::new(
            site(10),
            range(source, 20, 23),
            "set",
            TypeHeadInput::BuiltinSet,
        ),
        Vec::new(),
    );
    let input = TypeExpressionInput::new(
        site(0),
        range(source, 0, 12),
        "Mode of object",
        TypeHeadInput::Symbol(mode.clone()),
    )
    .with_args(vec![TypeExpressionInput::new(
        site(1),
        range(source, 8, 14),
        "object",
        TypeHeadInput::BuiltinObject,
    )]);

    let output = TypeNormalizer::new([(mode, expansion)]).normalize(&symbols, [input]);
    let normalized = output
        .type_entries()
        .iter()
        .find_map(|(_, entry)| match entry.actual {
            TypeEntryActual::Known(id) if entry.owner == site(0) => {
                output.normalized_types().get(id)
            }
            _ => None,
        })
        .unwrap();

    assert!(matches!(normalized.head, TypeHeadRef::BuiltinSet));
    assert_eq!(normalized.status, NormalizedTypeStatus::Degraded);
    assert_eq!(
        diagnostic_ranges(&output, "checker.type.wrong_mode_arity"),
        vec![(0, 12)]
    );
}

#[test]
fn missing_mode_payload_degrades_without_cluster_repair() {
    let source = source_id();
    let mode = symbol_id("Mode/0", "pkg::main::Mode/0");
    let symbols = symbol_env(vec![symbol_entry(mode.clone(), SymbolKind::Mode)]);
    let input = TypeExpressionInput::new(
        site(0),
        range(source, 0, 4),
        "Mode",
        TypeHeadInput::Symbol(mode.clone()),
    );

    let output = TypeNormalizer::default().normalize(&symbols, [input]);
    let normalized = output
        .normalized_types()
        .get(NormalizedTypeId::new(0))
        .unwrap();

    assert!(matches!(normalized.head, TypeHeadRef::Mode(ref actual) if actual == &mode));
    assert_eq!(normalized.status, NormalizedTypeStatus::Degraded);
    assert!(
        output
            .debug_text()
            .contains("checker.type.external.mode_expansion_payload")
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.type.external.mode_expansion_payload"),
        vec![(0, 4)]
    );
    assert!(!output.debug_text().contains("cluster"));
    assert!(!output.debug_text().contains("registration"));
}

#[test]
fn unknown_wrong_kind_ambiguous_and_unsupported_heads_are_partial_entries() {
    let source = source_id();
    let attr = symbol_id("Attr/0", "pkg::main::Attr/0");
    let candidate_a = symbol_id("A/0", "pkg::main::A/0");
    let candidate_b = symbol_id("B/0", "pkg::main::B/0");
    let symbols = symbol_env(vec![symbol_entry(attr.clone(), SymbolKind::Attribute)]);
    let inputs = [
        TypeExpressionInput::new(
            site(0),
            range(source, 0, 1),
            "missing",
            TypeHeadInput::Unresolved("missing".to_owned()),
        ),
        TypeExpressionInput::new(
            site(1),
            range(source, 2, 3),
            "Attr",
            TypeHeadInput::Symbol(attr),
        ),
        TypeExpressionInput::new(
            site(2),
            range(source, 4, 5),
            "ambiguous",
            TypeHeadInput::Ambiguous(vec![candidate_b, candidate_a]),
        ),
        TypeExpressionInput::new(
            site(3),
            range(source, 6, 7),
            "unsupported",
            TypeHeadInput::Unsupported("raw".to_owned()),
        ),
    ];

    let output = TypeNormalizer::default().normalize(&symbols, inputs);

    assert_eq!(output.type_entries().len(), 4);
    assert!(
        output
            .type_entries()
            .iter()
            .all(|(_, entry)| entry.status == TypeStatus::Unknown)
    );
    for expected in [
        "checker.type.unknown_head",
        "checker.type.wrong_head_kind",
        "checker.type.ambiguous_head",
        "checker.type.unsupported_payload",
        "checker.type.recovery",
    ] {
        assert!(output.debug_text().contains(expected), "{expected}");
    }
    assert_eq!(
        diagnostic_ranges(&output, "checker.type.unknown_head"),
        vec![(0, 1)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.type.wrong_head_kind"),
        vec![(2, 3)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.type.ambiguous_head"),
        vec![(4, 5)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.type.unsupported_payload"),
        vec![(6, 7)]
    );
}

#[test]
fn structure_arity_and_wrong_attribute_kind_emit_degraded_diagnostics() {
    let source = source_id();
    let structure = symbol_id("Struct/0", "pkg::main::Struct/0");
    let mode = symbol_id("Mode/0", "pkg::main::Mode/0");
    let wrong_attribute = symbol_id("Wrong/0", "pkg::main::Wrong/0");
    let symbols = symbol_env(vec![
        symbol_entry(structure, SymbolKind::Structure),
        symbol_entry(mode.clone(), SymbolKind::Mode),
        symbol_entry(wrong_attribute.clone(), SymbolKind::Mode),
    ]);
    let input = TypeExpressionInput::new(
        site(0),
        range(source, 0, 6),
        "Struct of Mode",
        TypeHeadInput::Symbol(symbol_id("Struct/0", "pkg::main::Struct/0")),
    )
    .with_args(vec![TypeExpressionInput::new(
        site(1),
        range(source, 7, 11),
        "Mode",
        TypeHeadInput::Symbol(mode),
    )])
    .with_attributes(vec![AttributeInput::new(
        wrong_attribute,
        AttributePolarity::Positive,
        range(source, 12, 13),
        "Wrong",
    )]);
    let builtin_with_arg = TypeExpressionInput::new(
        site(2),
        range(source, 14, 20),
        "set of Mode",
        TypeHeadInput::BuiltinSet,
    )
    .with_args(vec![TypeExpressionInput::new(
        site(3),
        range(source, 21, 25),
        "Mode",
        TypeHeadInput::Symbol(symbol_id("Mode/0", "pkg::main::Mode/0")),
    )]);

    let output = TypeNormalizer::default().normalize(&symbols, [input, builtin_with_arg]);

    assert!(
        output
            .debug_text()
            .contains("checker.type.wrong_structure_arity")
    );
    assert!(
        output
            .debug_text()
            .contains("checker.type.wrong_attribute_kind")
    );
    assert!(
        output
            .debug_text()
            .contains("checker.type.wrong_builtin_arity")
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.type.wrong_structure_arity"),
        vec![(0, 6)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.type.wrong_attribute_kind"),
        vec![(12, 13)]
    );
    assert_eq!(
        diagnostic_ranges(&output, "checker.type.wrong_builtin_arity"),
        vec![(14, 20)]
    );
}

#[test]
fn duplicate_type_expression_sites_are_diagnosed_without_duplicate_entries() {
    let source = source_id();
    let symbols = symbol_env(Vec::new());
    let first = TypeExpressionInput::new(
        site(0),
        range(source, 0, 3),
        "set",
        TypeHeadInput::BuiltinSet,
    );
    let duplicate = TypeExpressionInput::new(
        site(0),
        range(source, 4, 7),
        "object",
        TypeHeadInput::BuiltinObject,
    );

    let output = TypeNormalizer::default().normalize(&symbols, [first, duplicate]);

    assert_eq!(output.type_entries().len(), 1);
    assert_eq!(
        diagnostic_ranges(&output, "checker.type.duplicate_site"),
        vec![(4, 7)]
    );
}

trait HasDiagnostics {
    fn diagnostics(&self) -> &TypeDiagnosticTable;
}

impl HasDiagnostics for TypeNormalizationOutput {
    fn diagnostics(&self) -> &TypeDiagnosticTable {
        self.diagnostics()
    }
}

impl HasDiagnostics for DeclarationCheckingOutput {
    fn diagnostics(&self) -> &TypeDiagnosticTable {
        self.diagnostics()
    }
}

impl HasDiagnostics for TermFormulaInferenceOutput {
    fn diagnostics(&self) -> &TypeDiagnosticTable {
        self.diagnostics()
    }
}

impl HasDiagnostics for CoercionCheckingOutput {
    fn diagnostics(&self) -> &TypeDiagnosticTable {
        self.diagnostics()
    }
}

impl HasDiagnostics for TypeFactQueryOutput {
    fn diagnostics(&self) -> &TypeDiagnosticTable {
        self.diagnostics()
    }
}

fn declarations_by_binding(
    output: &DeclarationCheckingOutput,
) -> BTreeMap<BindingId, &CheckedDeclaration> {
    output
        .declarations()
        .iter()
        .map(|(_, declaration)| (declaration.binding, declaration))
        .collect()
}

fn term_by_site(output: &TermFormulaInferenceOutput, site: TypedSiteRef) -> &CheckedTerm {
    output
        .terms()
        .iter()
        .map(|(_, term)| term)
        .find(|term| term.site == site)
        .unwrap()
}

fn formula_by_site(output: &TermFormulaInferenceOutput, site: TypedSiteRef) -> &CheckedFormula {
    output
        .formulas()
        .iter()
        .map(|(_, formula)| formula)
        .find(|formula| formula.site == site)
        .unwrap()
}

fn diagnostic_ranges(output: &impl HasDiagnostics, message_key: &str) -> Vec<(usize, usize)> {
    output
        .diagnostics()
        .iter()
        .filter_map(|(_, diagnostic)| {
            (diagnostic.message_key == message_key)
                .then_some((diagnostic.source_range.start, diagnostic.source_range.end))
        })
        .collect()
}

#[derive(Debug, Clone, Copy)]
struct BindingSpec {
    spelling: &'static str,
    kind: BindingKind,
    status: BindingStatus,
}

fn binding_spec(spelling: &'static str, kind: BindingKind, status: BindingStatus) -> BindingSpec {
    BindingSpec {
        spelling,
        kind,
        status,
    }
}

fn binding_env_for_declarations(source: SourceId, specs: Vec<BindingSpec>) -> BindingEnv {
    let reserved = specs
        .iter()
        .enumerate()
        .filter_map(|(index, spec)| {
            (spec.kind == BindingKind::ReservedVariable).then_some(BindingId::new(index))
        })
        .collect::<Vec<_>>();
    let local = specs
        .iter()
        .enumerate()
        .filter_map(|(index, spec)| {
            (spec.kind != BindingKind::ReservedVariable).then_some(BindingId::new(index))
        })
        .collect::<Vec<_>>();
    let mut visible = reserved.clone();
    visible.extend(local.iter().copied());

    let mut contexts = BindingContextTable::new();
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: reserved.clone(),
        visible_bindings: reserved,
        recovery: BindingContextRecovery::Normal,
    });
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Generated("block".to_owned()),
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Block,
        lexical_scope: Some(LocalTermScope::new(vec![1])),
        bindings: local,
        visible_bindings: visible,
        recovery: BindingContextRecovery::Normal,
    });

    let mut bindings = BindingTable::new();
    for (index, spec) in specs.into_iter().enumerate() {
        let owner_context = if spec.kind == BindingKind::ReservedVariable {
            BindingContextId::new(0)
        } else {
            BindingContextId::new(1)
        };
        bindings.insert(binding_draft_for_spec(source, index, owner_context, spec));
    }

    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module_id(),
        contexts,
        bindings,
        diagnostics: BindingDiagnosticTable::new(),
    })
    .unwrap()
}

fn binding_env_with_outer_visible_binding(source: SourceId) -> BindingEnv {
    let mut contexts = BindingContextTable::new();
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: vec![BindingId::new(0)],
        visible_bindings: vec![BindingId::new(0)],
        recovery: BindingContextRecovery::Normal,
    });
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Generated("block".to_owned()),
        parent: Some(BindingContextId::new(0)),
        layer: BindingContextLayer::Block,
        lexical_scope: Some(LocalTermScope::new(vec![1])),
        bindings: Vec::new(),
        visible_bindings: vec![BindingId::new(0)],
        recovery: BindingContextRecovery::Normal,
    });

    let mut bindings = BindingTable::new();
    bindings.insert(binding_draft_for_spec(
        source,
        0,
        BindingContextId::new(0),
        binding_spec("let_x", BindingKind::LetBinding, BindingStatus::Active),
    ));

    BindingEnv::try_new(BindingEnvParts {
        source_id: source,
        module_id: module_id(),
        contexts,
        bindings,
        diagnostics: BindingDiagnosticTable::new(),
    })
    .unwrap()
}

fn binding_draft_for_spec(
    source: SourceId,
    index: usize,
    owner_context: BindingContextId,
    spec: BindingSpec,
) -> BindingDraft {
    let declaration_range = range(source, index * 10, index * 10 + spec.spelling.len());
    let identity = match spec.kind {
        BindingKind::ReservedVariable => BinderIdentity::ReservedVariable {
            spelling: spec.spelling.to_owned(),
            declaration_range,
        },
        BindingKind::Generated => BinderIdentity::Generated {
            context: owner_context,
            counter: index as u32,
        },
        _ => BinderIdentity::ResolverLocal {
            scope: LocalTermScope::new(vec![1]),
            ordinal: index,
            declaration_range,
        },
    };
    BindingDraft {
        spelling: spec.spelling.to_owned(),
        kind: spec.kind,
        identity,
        owner_context,
        declaration_range,
        visible_after_ordinal: index,
        type_site: BindingTypeSite::Source(declaration_range),
        status: spec.status,
        captured: CapturedFreeVariables::default(),
        diagnostics: Vec::new(),
        recovery: BindingRecoveryState::Normal,
    }
}

fn declaration_with_type(
    source: SourceId,
    binding_index: usize,
    kind: DeclarationKind,
    declaration_node: usize,
    type_node: usize,
) -> DeclarationInput {
    declaration_with_type_in_context(
        source,
        binding_index,
        BindingContextId::new(1),
        kind,
        declaration_node,
        type_node,
    )
}

fn declaration_with_type_in_context(
    source: SourceId,
    binding_index: usize,
    context: BindingContextId,
    kind: DeclarationKind,
    declaration_node: usize,
    type_node: usize,
) -> DeclarationInput {
    DeclarationInput::new(
        BindingId::new(binding_index),
        context,
        site(declaration_node),
        range(source, declaration_node * 5, declaration_node * 5 + 5),
        kind,
    )
    .with_type_expression(TypeExpressionInput::new(
        site(type_node),
        range(source, type_node * 5, type_node * 5 + 3),
        "set",
        TypeHeadInput::BuiltinSet,
    ))
}

fn term_with_type(
    source: SourceId,
    term_node: usize,
    kind: TermKind,
    type_node: usize,
) -> TermInput {
    TermInput::new(
        site(term_node),
        BindingContextId::new(1),
        range(source, term_node * 5, term_node * 5 + 5),
        kind,
    )
    .with_result_type(type_expression(
        source,
        type_node,
        TypeHeadInput::BuiltinSet,
    ))
}

fn type_expression(source: SourceId, type_node: usize, head: TypeHeadInput) -> TypeExpressionInput {
    TypeExpressionInput::new(
        site(type_node),
        range(source, type_node * 5, type_node * 5 + 3),
        "set",
        head,
    )
}

fn omitted_reconsider_input(
    source: SourceId,
    offset: usize,
    evidence: CoercionEvidence,
) -> CoercionInput {
    CoercionInput::new(
        site(offset),
        range(source, offset * 5, offset * 5 + 5),
        CoercionRequestKind::Narrowing,
        type_expression(source, offset + 100, TypeHeadInput::BuiltinSet),
    )
    .with_from_type(type_expression(
        source,
        offset + 200,
        TypeHeadInput::BuiltinObject,
    ))
    .with_justification(CoercionJustification::Omitted)
    .with_evidence(evidence)
}

#[test]
fn statement_owner_and_formula_handoff_are_exact_and_fail_closed() {
    let source = source_id();
    let bindings = binding_env_for_declarations(source, Vec::new());
    let formula_range = range(source, 20, 30);
    let formula_input = FormulaInput::new(
        TypedSiteRef::Node(TypedNodeId::new(17)),
        BindingContextId::new(0),
        formula_range,
        FormulaKind::Contradiction,
    );
    assert_eq!(formula_input.recovery, NodeRecoveryState::Normal);
    let formula_output =
        TermFormulaChecker::default().infer_without_symbols_for_test(&bindings, [formula_input]);
    let checked_formula = formula_output
        .formulas()
        .get(CheckedFormulaId::new(0))
        .expect("default-normal formula");
    assert_eq!(checked_formula.source_range, formula_range);
    assert_eq!(checked_formula.recovery, NodeRecoveryState::Normal);

    let other_module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("other"));
    let mismatched_symbols = SymbolEnv::new(other_module.clone(), SymbolEnvIndexes::default());
    let mismatch = TermFormulaChecker::default()
        .try_infer(
            &mismatched_symbols,
            &bindings,
            Vec::<TermInput>::new(),
            Vec::<FormulaInput>::new(),
        )
        .expect_err("term/formula inference must reject mismatched modules");
    assert!(matches!(
        mismatch,
        TermFormulaInferenceError::ModuleMismatch { .. }
    ));

    let module = module_id();
    let owner = symbol_id("Task180", "pkg::main::theorem::Task180");
    let owner_range = range(source, 10, 90);
    let origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(owner_range),
        vec![0],
    );
    let base = StatementOwnerEnvSpec {
        include_owner: true,
        owner_kind: SymbolKind::Theorem,
        owner_visibility: Visibility::Public,
        owner_export_status: ExportStatus::Exported,
        owner_origin: origin.clone(),
        contribution_module: module.clone(),
        contribution_kind: Some(ContributionKind::LocalSource { source_id: source }),
        definition_kind: Some(DefinitionKind::Theorem),
        definition_visibility: Visibility::Public,
        definition_origin: origin.clone(),
        definition_conflict: false,
        definition_contribution_mismatch: false,
        duplicate_owner: false,
    };

    let valid_env = statement_owner_env(&module, &owner, owner_range, base.clone());
    let checked = CheckedStatementOwner::validate_exact_local_theorem(
        &valid_env,
        owner.clone(),
        source,
        &module,
    )
    .expect("exact local theorem owner");
    assert_eq!(checked.symbol(), &owner);
    assert_eq!(checked.source_range(), owner_range);
    assert_eq!(checked.origin(), &origin);
    assert_eq!(checked.visibility(), Visibility::Public);
    assert_eq!(checked.export_status(), ExportStatus::Exported);

    let local_contribution = ContributionKind::LocalSource { source_id: source };
    for state in [
        StatementOwnerProvenanceState {
            anchor: origin.anchor(),
            owner_source: source,
            owner_module: &module,
            has_import_edge: true,
            owner_recovered: false,
            definition_recovered: false,
            contribution_module: &module,
            contribution_kind: &local_contribution,
        },
        StatementOwnerProvenanceState {
            anchor: origin.anchor(),
            owner_source: source,
            owner_module: &module,
            has_import_edge: false,
            owner_recovered: false,
            definition_recovered: true,
            contribution_module: &module,
            contribution_kind: &local_contribution,
        },
    ] {
        assert_eq!(
            exact_statement_owner_source_range(state, source, &module),
            None,
            "imported or recovered-definition provenance must fail closed"
        );
    }

    assert!(matches!(
        CheckedStatementOwner::validate_exact_local_theorem(
            &valid_env,
            owner.clone(),
            source,
            &other_module,
        ),
        Err(StatementOwnerError::EnvironmentMismatch)
    ));
    let requested_other = symbol_id("Other", "pkg::main::theorem::Other");
    assert!(matches!(
        CheckedStatementOwner::validate_exact_local_theorem(
            &valid_env,
            requested_other,
            source,
            &module,
        ),
        Err(StatementOwnerError::InvalidOwner { .. })
    ));

    let mut owner_cases = Vec::new();
    let mut missing = base.clone();
    missing.include_owner = false;
    owner_cases.push(missing);
    let mut duplicate = base.clone();
    duplicate.duplicate_owner = true;
    owner_cases.push(duplicate);
    let mut wrong_kind = base.clone();
    wrong_kind.owner_kind = SymbolKind::Mode;
    owner_cases.push(wrong_kind);
    let mut private_owner = base.clone();
    private_owner.owner_visibility = Visibility::Private;
    owner_cases.push(private_owner);
    let mut local_only_owner = base.clone();
    local_only_owner.owner_export_status = ExportStatus::LocalOnly;
    owner_cases.push(local_only_owner);
    let mut wrong_source = base.clone();
    wrong_source.owner_origin = SemanticOrigin::new(
        source_ids_pair().1,
        module.clone(),
        SourceAnchor::Range(owner_range),
        vec![0],
    );
    wrong_source.definition_origin = wrong_source.owner_origin.clone();
    owner_cases.push(wrong_source);
    for spec in owner_cases {
        let env = statement_owner_env(&module, &owner, owner_range, spec);
        assert!(matches!(
            CheckedStatementOwner::validate_exact_local_theorem(
                &env,
                owner.clone(),
                source,
                &module,
            ),
            Err(StatementOwnerError::InvalidOwner { .. })
        ));
    }

    let mut definition_cases = Vec::new();
    let mut missing_definition = base.clone();
    missing_definition.definition_kind = None;
    definition_cases.push(missing_definition);
    let mut wrong_definition = base.clone();
    wrong_definition.definition_kind = Some(DefinitionKind::Mode);
    definition_cases.push(wrong_definition);
    let mut private_definition = base.clone();
    private_definition.definition_visibility = Visibility::Private;
    definition_cases.push(private_definition);
    let mut conflicted = base.clone();
    conflicted.definition_conflict = true;
    definition_cases.push(conflicted);
    let mut origin_mismatch = base.clone();
    origin_mismatch.definition_origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Range(range(source, 11, 90)),
        vec![0],
    );
    definition_cases.push(origin_mismatch);
    let mut definition_only_recovery = base.clone();
    definition_only_recovery.definition_origin = origin.clone().recovered();
    definition_cases.push(definition_only_recovery);
    let mut contribution_mismatch = base.clone();
    contribution_mismatch.definition_contribution_mismatch = true;
    definition_cases.push(contribution_mismatch);
    for spec in definition_cases {
        let env = statement_owner_env(&module, &owner, owner_range, spec);
        assert!(matches!(
            CheckedStatementOwner::validate_exact_local_theorem(
                &env,
                owner.clone(),
                source,
                &module,
            ),
            Err(StatementOwnerError::InvalidDefinition { .. })
        ));
    }

    let mut provenance_cases = Vec::new();
    let mut missing_contribution = base.clone();
    missing_contribution.contribution_kind = None;
    provenance_cases.push(missing_contribution);
    let mut point_anchor = base.clone();
    point_anchor.owner_origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Point {
            source_id: source,
            offset: owner_range.start,
        },
        vec![0],
    );
    point_anchor.definition_origin = point_anchor.owner_origin.clone();
    provenance_cases.push(point_anchor);
    let generated_anchor = GeneratedSpanOrigin::new(
        GeneratedSpanAnchor::Range(owner_range),
        "statement-owner-generated-anchor-test",
    )
    .expect("generated statement-owner test anchor");
    let mut generated = base.clone();
    generated.owner_origin = SemanticOrigin::new(
        source,
        module.clone(),
        SourceAnchor::Generated(generated_anchor),
        vec![0],
    );
    generated.definition_origin = generated.owner_origin.clone();
    provenance_cases.push(generated);
    let mut owner_module_mismatch = base.clone();
    owner_module_mismatch.owner_origin = SemanticOrigin::new(
        source,
        other_module.clone(),
        SourceAnchor::Range(owner_range),
        vec![0],
    );
    owner_module_mismatch.definition_origin = owner_module_mismatch.owner_origin.clone();
    provenance_cases.push(owner_module_mismatch);
    let mut recovered = base.clone();
    recovered.owner_origin = origin.clone().recovered();
    recovered.definition_origin = recovered.owner_origin.clone();
    provenance_cases.push(recovered);
    let mut imported_contribution = base.clone();
    imported_contribution.contribution_kind =
        Some(ContributionKind::ImportedSource { source_id: source });
    provenance_cases.push(imported_contribution);
    let mut contribution_source_mismatch = base.clone();
    contribution_source_mismatch.contribution_kind = Some(ContributionKind::LocalSource {
        source_id: source_ids_pair().1,
    });
    provenance_cases.push(contribution_source_mismatch);
    let mut contribution_module_mismatch = base;
    contribution_module_mismatch.contribution_module = other_module;
    provenance_cases.push(contribution_module_mismatch);
    for spec in provenance_cases {
        let env = statement_owner_env(&module, &owner, owner_range, spec);
        assert!(matches!(
            CheckedStatementOwner::validate_exact_local_theorem(
                &env,
                owner.clone(),
                source,
                &module,
            ),
            Err(StatementOwnerError::InvalidProvenance { .. })
        ));
    }
}

#[derive(Clone)]
struct StatementOwnerEnvSpec {
    include_owner: bool,
    owner_kind: SymbolKind,
    owner_visibility: Visibility,
    owner_export_status: ExportStatus,
    owner_origin: SemanticOrigin,
    contribution_module: ModuleId,
    contribution_kind: Option<ContributionKind>,
    definition_kind: Option<DefinitionKind>,
    definition_visibility: Visibility,
    definition_origin: SemanticOrigin,
    definition_conflict: bool,
    definition_contribution_mismatch: bool,
    duplicate_owner: bool,
}

fn statement_owner_env(
    module: &ModuleId,
    owner: &SymbolId,
    owner_range: SourceRange,
    spec: StatementOwnerEnvSpec,
) -> SymbolEnv {
    let mut contributions = SourceContributionIndex::new();
    let contribution = spec.contribution_kind.map_or_else(
        || {
            let mut detached = SourceContributionIndex::new();
            detached.insert(
                module.clone(),
                ContributionKind::LocalSource {
                    source_id: spec.owner_origin.source_id(),
                },
                SourceAnchor::Range(owner_range),
            )
        },
        |kind| {
            contributions.insert(
                spec.contribution_module,
                kind,
                SourceAnchor::Range(owner_range),
            )
        },
    );
    let mut symbols = SymbolIndex::new();
    if spec.include_owner {
        symbols.insert(
            SymbolEntry::new(
                owner.clone(),
                spec.owner_kind,
                NamespacePath::new("main"),
                "Task180",
                spec.owner_origin.clone(),
                contribution,
            )
            .with_visibility(spec.owner_visibility)
            .with_export_status(spec.owner_export_status),
        );
    }
    if spec.duplicate_owner {
        symbols.insert(SymbolEntry::new(
            SymbolId::new(
                module.clone(),
                LocalSymbolId::new("Task180Duplicate"),
                FullyQualifiedName::new("pkg::main::theorem::Task180Duplicate"),
            ),
            SymbolKind::Theorem,
            NamespacePath::new("main"),
            "Task180Duplicate",
            spec.owner_origin.clone(),
            contribution,
        ));
    }
    let mut definitions = DefinitionIndex::new();
    if let Some(kind) = spec.definition_kind {
        let definition_contribution = if spec.definition_contribution_mismatch {
            contributions.insert(
                module.clone(),
                ContributionKind::LocalSource {
                    source_id: spec.definition_origin.source_id(),
                },
                SourceAnchor::Range(owner_range),
            )
        } else {
            contribution
        };
        let mut definition = DefinitionShell::new(
            owner.clone(),
            kind,
            spec.definition_origin,
            definition_contribution,
        )
        .with_visibility(spec.definition_visibility);
        if spec.definition_conflict {
            definition = definition.with_conflict(DeclarationConflictClass::DuplicateSpelling);
        }
        definitions.insert(definition);
    }
    SymbolEnv::new(
        module.clone(),
        SymbolEnvIndexes {
            symbols,
            definitions,
            contributions,
            ..SymbolEnvIndexes::default()
        },
    )
}

fn symbol_env(entries: Vec<SymbolEntry>) -> SymbolEnv {
    let mut symbols = SymbolIndex::new();
    for entry in entries {
        symbols.insert(entry);
    }
    let mut contributions = SourceContributionIndex::new();
    let source = source_id();
    contributions.insert(
        module_id(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 1)),
    );
    SymbolEnv::new(
        module_id(),
        SymbolEnvIndexes {
            imports: ResolvedImportIndex::new(),
            exports: ResolvedExportIndex::new(),
            symbols,
            labels: LabelIndex::new(),
            definitions: DefinitionIndex::new(),
            overloads: OverloadIndex::new(),
            registrations: RegistrationIndex::new(),
            lexical_summaries: ModuleLexicalSummaryIndex::new(),
            namespace_graph: NamespaceGraph::new(),
            declaration_dependencies: Default::default(),
            contributions,
            module_summaries: ModuleSummaryIndex::new(),
        },
    )
}

fn symbol_env_with_imported_attribute(
    structure: SymbolId,
    imported_attribute: SymbolId,
) -> SymbolEnv {
    let source = source_id();
    let imported_module = imported_attribute.module().clone();
    let mut contributions = SourceContributionIndex::new();
    let local_contribution = contributions.insert(
        module_id(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 1)),
    );
    let imported_contribution = contributions.insert(
        imported_module.clone(),
        ContributionKind::ImportedSource { source_id: source },
        SourceAnchor::Range(range(source, 1, 2)),
    );
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            structure,
            SymbolKind::Structure,
            NamespacePath::new("main"),
            "Struct",
            SemanticOrigin::new(
                source,
                module_id(),
                SourceAnchor::Range(range(source, 0, 1)),
                Vec::new(),
            ),
            local_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    symbols.insert(
        SymbolEntry::new(
            imported_attribute,
            SymbolKind::Attribute,
            NamespacePath::new("main"),
            "empty",
            SemanticOrigin::new(
                source,
                imported_module,
                SourceAnchor::Range(range(source, 1, 2)),
                Vec::new(),
            ),
            imported_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    SymbolEnv::new(
        module_id(),
        SymbolEnvIndexes {
            imports: ResolvedImportIndex::new(),
            exports: ResolvedExportIndex::new(),
            symbols,
            labels: LabelIndex::new(),
            definitions: DefinitionIndex::new(),
            overloads: OverloadIndex::new(),
            registrations: RegistrationIndex::new(),
            lexical_summaries: ModuleLexicalSummaryIndex::new(),
            namespace_graph: NamespaceGraph::new(),
            declaration_dependencies: Default::default(),
            contributions,
            module_summaries: ModuleSummaryIndex::new(),
        },
    )
}

fn symbol_env_with_imported_symbol(
    imported_symbol: SymbolId,
    kind: SymbolKind,
    spelling: &'static str,
) -> SymbolEnv {
    let source = source_id();
    let imported_module = imported_symbol.module().clone();
    let mut contributions = SourceContributionIndex::new();
    let imported_contribution = contributions.insert(
        imported_module.clone(),
        ContributionKind::ImportedSource { source_id: source },
        SourceAnchor::Range(range(source, 1, 2)),
    );
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            imported_symbol,
            kind,
            NamespacePath::new("main"),
            spelling,
            SemanticOrigin::new(
                source,
                imported_module,
                SourceAnchor::Range(range(source, 1, 2)),
                Vec::new(),
            ),
            imported_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    SymbolEnv::new(
        module_id(),
        SymbolEnvIndexes {
            imports: ResolvedImportIndex::new(),
            exports: ResolvedExportIndex::new(),
            symbols,
            labels: LabelIndex::new(),
            definitions: DefinitionIndex::new(),
            overloads: OverloadIndex::new(),
            registrations: RegistrationIndex::new(),
            lexical_summaries: ModuleLexicalSummaryIndex::new(),
            namespace_graph: NamespaceGraph::new(),
            declaration_dependencies: Default::default(),
            contributions,
            module_summaries: ModuleSummaryIndex::new(),
        },
    )
}

fn symbol_env_with_local_structure_and_imported_symbol(
    structure: SymbolId,
    imported_symbol: SymbolId,
    kind: SymbolKind,
    spelling: &'static str,
) -> SymbolEnv {
    let source = source_id();
    let imported_module = imported_symbol.module().clone();
    let mut contributions = SourceContributionIndex::new();
    let local_contribution = contributions.insert(
        module_id(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 1)),
    );
    let imported_contribution = contributions.insert(
        imported_module.clone(),
        ContributionKind::ImportedSource { source_id: source },
        SourceAnchor::Range(range(source, 1, 2)),
    );
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            structure,
            SymbolKind::Structure,
            NamespacePath::new("main"),
            "Struct",
            SemanticOrigin::new(
                source,
                module_id(),
                SourceAnchor::Range(range(source, 0, 1)),
                Vec::new(),
            ),
            local_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    symbols.insert(
        SymbolEntry::new(
            imported_symbol,
            kind,
            NamespacePath::new("main"),
            spelling,
            SemanticOrigin::new(
                source,
                imported_module,
                SourceAnchor::Range(range(source, 1, 2)),
                Vec::new(),
            ),
            imported_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    SymbolEnv::new(
        module_id(),
        SymbolEnvIndexes {
            imports: ResolvedImportIndex::new(),
            exports: ResolvedExportIndex::new(),
            symbols,
            labels: LabelIndex::new(),
            definitions: DefinitionIndex::new(),
            overloads: OverloadIndex::new(),
            registrations: RegistrationIndex::new(),
            lexical_summaries: ModuleLexicalSummaryIndex::new(),
            namespace_graph: NamespaceGraph::new(),
            declaration_dependencies: Default::default(),
            contributions,
            module_summaries: ModuleSummaryIndex::new(),
        },
    )
}

fn symbol_env_with_foreign_local_symbol(
    foreign_symbol: SymbolId,
    kind: SymbolKind,
    spelling: &'static str,
) -> SymbolEnv {
    let source = source_id();
    let foreign_module = foreign_symbol.module().clone();
    let mut contributions = SourceContributionIndex::new();
    let foreign_local_contribution = contributions.insert(
        foreign_module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 1, 2)),
    );
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            foreign_symbol,
            kind,
            NamespacePath::new("main"),
            spelling,
            SemanticOrigin::new(
                source,
                foreign_module,
                SourceAnchor::Range(range(source, 1, 2)),
                Vec::new(),
            ),
            foreign_local_contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    SymbolEnv::new(
        module_id(),
        SymbolEnvIndexes {
            imports: ResolvedImportIndex::new(),
            exports: ResolvedExportIndex::new(),
            symbols,
            labels: LabelIndex::new(),
            definitions: DefinitionIndex::new(),
            overloads: OverloadIndex::new(),
            registrations: RegistrationIndex::new(),
            lexical_summaries: ModuleLexicalSummaryIndex::new(),
            namespace_graph: NamespaceGraph::new(),
            declaration_dependencies: Default::default(),
            contributions,
            module_summaries: ModuleSummaryIndex::new(),
        },
    )
}

fn symbol_entry(symbol: SymbolId, kind: SymbolKind) -> SymbolEntry {
    let source = source_id();
    let mut contributions = SourceContributionIndex::new();
    let contribution = contributions.insert(
        module_id(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 1)),
    );
    SymbolEntry::new(
        symbol,
        kind,
        NamespacePath::new("main"),
        "symbol",
        SemanticOrigin::new(
            source,
            module_id(),
            SourceAnchor::Range(range(source, 0, 1)),
            Vec::new(),
        ),
        contribution,
    )
    .with_visibility(Visibility::Public)
    .with_export_status(ExportStatus::Exported)
}

fn site(index: usize) -> TypedSiteRef {
    TypedSiteRef::Role {
        node: TypedNodeId::new(index),
        role: TypeRole::new("type"),
    }
}

fn source_id() -> SourceId {
    source_ids_pair().0
}

fn source_ids_pair() -> (SourceId, SourceId) {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "17".repeat(32)
    ))
    .unwrap();
    let allocator = InMemorySessionIdAllocator::new();
    (
        allocator.next_source_id(snapshot).unwrap(),
        allocator.next_source_id(snapshot).unwrap(),
    )
}

fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn module_id() -> ModuleId {
    ModuleId::new(PackageId::new("pkg"), ModulePath::new("main"))
}

fn symbol_id(local: &str, fqn: &str) -> SymbolId {
    SymbolId::new(
        module_id(),
        LocalSymbolId::new(local),
        FullyQualifiedName::new(fqn),
    )
}
