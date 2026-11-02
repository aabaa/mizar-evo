use super::{
    SOURCE_STATEMENT_B1_TEXT, SOURCE_STATEMENT_B2_TEXT, SOURCE_STATEMENT_B3_TEXT,
    SOURCE_STATEMENT_B3M1_TEXT, SOURCE_STATEMENT_B3M2A_TEXT, SOURCE_STATEMENT_B3M2B1_TEXT,
    SOURCE_STATEMENT_B3M2B2A_TEXT, SOURCE_STATEMENT_B3M2B2B1A_TEXT,
    SOURCE_STATEMENT_B3M2B2B1B1_TEXT, SOURCE_STATEMENT_B3M2B2B2A_TEXT,
    SOURCE_STATEMENT_B3M2B2B2B_TEXT, SOURCE_STATEMENT_B3M2B2B2C_TEXT,
    SOURCE_STATEMENT_B3M2B2B3A_TEXT, SOURCE_STATEMENT_B3M2B2B3B_TEXT,
    SOURCE_STATEMENT_B3M2B2B3C_TEXT, SOURCE_STATEMENT_B3M2B2B3D_TEXT,
    SOURCE_STATEMENT_B3M2B2B3E_TEXT, SOURCE_STATEMENT_B3N_TEXT, SOURCE_STATEMENT_B4A_TEXT,
    SOURCE_STATEMENT_B4B_TEXT, SOURCE_STATEMENT_B4C_TEXT, SOURCE_STATEMENT_B5A_TEXT,
    SOURCE_STATEMENT_B5B_TEXT, SOURCE_STATEMENT_TEXT, SourceStatementB1Extraction,
    SourceStatementB2Extraction, SourceStatementB3Extraction,
     SourceStatementB3M2AExtraction,
    SourceStatementB3M2ARouteInputs, SourceStatementB3M2B1Extraction,
    SourceStatementB3M2B2AExtraction,
    SourceStatementB3M2B2B1AExtraction,
    SourceStatementB3M2B2B1ARouteInputs, SourceStatementB3M2B2B1B1Extraction,
    SourceStatementB3M2B2B1B1RouteInputs, SourceStatementB3M2B2B2AExtraction,
    SourceStatementB3M2B2B2ARouteInputs, SourceStatementB3M2B2B2BExtraction,
    SourceStatementB3M2B2B2BRouteInputs, SourceStatementB3M2B2B2CExtraction,
    SourceStatementB3M2B2B2CRouteInputs, SourceStatementB3M2B2B3AExtraction,
    SourceStatementB3M2B2B3ARouteInputs, SourceStatementB3M2B2B3AStage,
    SourceStatementB3M2B2B3ASurfaceMutation, SourceStatementB3M2B2B3BExtraction,
    SourceStatementB3M2B2B3BSurfaceMutation,
    SourceStatementB3M2B2B3CExtraction, SourceStatementB3M2B2B3CSurfaceMutation, SourceStatementB3M2B2B3DExtraction,
    SourceStatementB3M2B2B3DLowerStage, SourceStatementB3M2B2B3DRouteInputs,
    SourceStatementB3M2B2B3DStage, SourceStatementB3M2B2B3DSurfaceMutation,
    SourceStatementB3M2B2B3EExtraction, SourceStatementB3M2B2B3ELowerStage,
    SourceStatementB3M2B2B3ERouteInputs, SourceStatementB3M2B2B3EStage,
    SourceStatementB3M2B2B3ESurfaceMutation, SourceStatementB3NExtraction,
    SourceStatementB4ASurfaceMutation,
    SourceStatementB4BSurfaceMutation, SourceStatementB4CSurfaceMutation,
    SourceStatementB5AExtraction, SourceStatementB5ASurfaceMutation, SourceStatementB5BExtraction,
    SourceStatementB5BSurfaceMutation, SourceStatementExtraction,
    SourceStatementRouteOutput, TASK258B3M2B2B3A_TASK256_FIELD_COUNT,
    TASK258B3M2B2B3A_TASK258_FIELD_COUNT, TASK258B3M2B2B3A_WITNESS_FIELD_COUNT,
    TASK258B3M2B2B3D_TASK48_FIELD_COUNT, TASK258B3M2B2B3D_TASK252_FIELD_COUNT,
    TASK258B3M2B2B3D_TASK255_FIELD_COUNT, TASK258B3M2B2B3D_TASK256_FIELD_COUNT,
    TASK258B3M2B2B3D_TASK258_FIELD_COUNT, TASK258B3M2B2B3D_WITNESS_FIELD_COUNT,
    TASK258B3M2B2B3E_TASK48_FIELD_COUNT, TASK258B3M2B2B3E_TASK252_FIELD_COUNT,
    TASK258B3M2B2B3E_TASK255_FIELD_COUNT, TASK258B3M2B2B3E_TASK256_FIELD_COUNT,
    TASK258B3M2B2B3E_TASK258_FIELD_COUNT, TASK258B3M2B2B3E_WITNESS_FIELD_COUNT,
    extract_ancestor_label_descendant_citation_source_statement,
    extract_application_witness_source_statement, extract_choice_witness_source_statement,
    extract_choice_witness_source_statement_with_surface_mutation,
    extract_comprehension_witness_source_statement,
    extract_comprehension_witness_source_statement_with_surface_mutation,
    extract_empty_set_enumeration_witness_source_statement,
    extract_empty_set_enumeration_witness_source_statement_with_surface_mutation,
    extract_imported_public_theorem_citation_source_statement,
     extract_named_witness_source_statement,
    extract_nested_parenthesized_witness_source_statement, extract_nested_source_statement,
    extract_numeral_witness_source_statement, extract_parenthesized_witness_source_statement,
    extract_qua_witness_source_statement,
    extract_qua_witness_source_statement_with_surface_mutation,
    extract_set_enumeration_witness_source_statement,
    extract_set_enumeration_witness_source_statement_with_surface_mutation,
    extract_single_assumption_source_statement, extract_single_witness_source_statement,
    extract_source_reserved_variable_theorem_statement,
    extract_structure_constructor_witness_source_statement,
    extract_structure_selector_witness_source_statement,
    extract_structure_update_witness_source_statement,
    extract_wrapped_application_witness_source_statement,


     source_statement_b3m2a_output_with_mutation,
    source_statement_b3m2a_output_with_resolver_mutation,
    source_statement_b3m2a_resolver_env_for_test, source_statement_b3m2b2b1a_output_with_mutation,
    source_statement_b3m2b2b1a_output_with_resolver_mutation,
    source_statement_b3m2b2b1a_resolver_env_for_test,
    source_statement_b3m2b2b1b1_output_with_mutation,
    source_statement_b3m2b2b1b1_output_with_resolver_mutation,
    source_statement_b3m2b2b1b1_resolver_env_for_test,
    source_statement_b3m2b2b2a_output_with_mutation,
    source_statement_b3m2b2b2a_output_with_resolver_mutation,
    source_statement_b3m2b2b2a_resolver_env_for_test,
    source_statement_b3m2b2b2b_output_with_mutation,
    source_statement_b3m2b2b2b_output_with_resolver_mutation,
    source_statement_b3m2b2b2b_resolver_env_for_test,
    source_statement_b3m2b2b2c_output_with_mutation,
    source_statement_b3m2b2b2c_output_with_resolver_mutation,
    source_statement_b3m2b2b2c_resolver_env_for_test,
    source_statement_b3m2b2b3a_output_with_mutation,
    source_statement_b3m2b2b3a_output_with_post_auth_family_mutation,
    source_statement_b3m2b2b3a_output_with_resolver_mutation,
    source_statement_b3m2b2b3a_output_with_stage_mutation,
    source_statement_b3m2b2b3a_resolver_env_for_test,
    source_statement_b3m2b2b3b_resolver_env_for_test,
    source_statement_b3m2b2b3c_output_with_resolver_mutation,
    source_statement_b3m2b2b3c_resolver_env_for_test,
    source_statement_b3m2b2b3d_output_with_lower_stage_mutation,
    source_statement_b3m2b2b3d_output_with_mutation,
    source_statement_b3m2b2b3d_output_with_post_auth_family_mutation,
    source_statement_b3m2b2b3d_output_with_resolver_mutation,
    source_statement_b3m2b2b3d_output_with_stage_mutation,
    source_statement_b3m2b2b3d_resolver_env_for_test,
    source_statement_b3m2b2b3e_output_with_lower_stage_mutation,
    source_statement_b3m2b2b3e_output_with_mutation,
    source_statement_b3m2b2b3e_output_with_post_auth_family_mutation,
    source_statement_b3m2b2b3e_output_with_resolver_mutation,
    source_statement_b3m2b2b3e_output_with_stage_mutation,
    source_statement_b3m2b2b3e_resolver_env_for_test,
    source_statement_b4a_output_with_mutation, source_statement_b4a_output_with_resolver_mutation,
    source_statement_b4a_resolver_env_for_test, source_statement_b4b_output_with_mutation,
    source_statement_b4b_output_with_resolver_mutation, source_statement_b4b_resolver_env_for_test,
    source_statement_b4c_output_with_mutation,
    source_statement_b4c_output_with_raw_resolver_mutation,
    source_statement_b4c_output_with_resolver_mutation, source_statement_b4c_resolver_env_for_test,
    source_statement_b5a_resolver_env_for_test,
    validate_task258b5a_raw_resolver_env, validate_task258b5a_enriched_resolver_env,
    source_statement_b5b_output_with_mutation,
    source_statement_b5b_output_with_raw_resolver_mutation,
    source_statement_b5b_output_with_resolver_mutation, source_statement_b5b_resolver_env_for_test,
    source_statement_output_with_source, source_statement_resolver_env_for_test,
    source_statement_transport_detail_keys,
    source_statement_transport_detail_keys_with_output_mutation_for_test,
    task258b4a_surface_profile_with_mutation_for_test,
    task258b4b_surface_profile_with_mutation_for_test,
    task258b4c_surface_profile_with_mutation_for_test,
    task258b5a_surface_profile_with_mutation_for_test,
    task258b5b_surface_profile_with_mutation_for_test,
};

fn sha256_text(text: &str) -> String {
    use std::{
        io::Write as _,
        process::{Command, Stdio},
    };

    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("sha256sum");
    child
        .stdin
        .as_mut()
        .expect("sha256 stdin")
        .write_all(text.as_bytes())
        .expect("hash input");
    let output = child.wait_with_output().expect("hash output");
    assert!(output.status.success());
    String::from_utf8(output.stdout)
        .expect("hash text")
        .split_whitespace()
        .next()
        .expect("hash")
        .to_owned()
}

fn task258b3m2a_primary_input(
    primary: &mizar_checker::source_term::SourcePrimaryTermHandoff,
) -> mizar_checker::source_term::SourcePrimaryTermHandoffInput {
    mizar_checker::source_term::SourcePrimaryTermHandoffInput {
        source_id: primary.source_id(),
        module_id: primary.module_id().clone(),
        terms: primary
            .terms()
            .iter()
            .map(
                |(_, term)| mizar_checker::source_term::SourcePrimaryTermInput {
                    site: term.site().clone(),
                    source_range: term.source_range(),
                    source_ordinal: term.source_ordinal(),
                    context: term.context(),
                    recovery: term.recovery(),
                    spelling: term.spelling().to_owned(),
                    kind: term.kind(),
                    role: term.role(),
                    parent: term.parent(),
                },
            )
            .collect(),
        references: primary
            .references()
            .iter()
            .map(
                |(_, reference)| mizar_checker::source_term::SourcePrimaryTermReferenceInput {
                    term: reference.term(),
                    binding: reference.binding(),
                    role: reference.role(),
                },
            )
            .collect(),
        numeric_type_requests: primary
            .numeric_type_requests()
            .iter()
            .map(
                |(_, request)| mizar_checker::source_term::SourceNumericTypeRequestInput {
                    term: request.term(),
                    owner: request.owner().clone(),
                    source_range: request.source_range(),
                    spelling: request.spelling().to_owned(),
                    request_ordinal: request.request_ordinal(),
                },
            )
            .collect(),
    }
}

fn task258b3m2b1_atomic_input(
    atomic: &mizar_checker::source_atomic_formula::SourceAtomicFormulaHandoff,
) -> mizar_checker::source_atomic_formula::SourceAtomicFormulaHandoffInput {
    assert!(atomic.wrappers().is_empty());
    assert!(atomic.predicate_segments().is_empty());
    assert!(atomic.predicate_heads().is_empty());
    assert!(atomic.candidates().is_empty());
    assert!(atomic.type_sites().is_empty());
    assert!(atomic.attributes().is_empty());
    mizar_checker::source_atomic_formula::SourceAtomicFormulaHandoffInput {
        source_id: atomic.source_id(),
        module_id: atomic.module_id().clone(),
        formulas: atomic
            .formulas()
            .iter()
            .map(
                |(_, formula)| mizar_checker::source_atomic_formula::SourceAtomicFormulaInput {
                    site: formula.site().clone(),
                    source_range: formula.source_range(),
                    source_ordinal: formula.source_ordinal(),
                    context: formula.context(),
                    recovery: formula.recovery(),
                    spelling: formula.spelling().to_owned(),
                    kind: formula.kind(),
                },
            )
            .collect(),
        wrappers: Vec::new(),
        predicate_segments: Vec::new(),
        predicate_heads: Vec::new(),
        candidates: Vec::new(),
        type_sites: Vec::new(),
        attributes: Vec::new(),
        edges: atomic
            .edges()
            .iter()
            .map(
                |(_, edge)| mizar_checker::source_atomic_formula::SourceAtomicEdgeInput {
                    formula: edge.formula(),
                    ordinal: edge.ordinal(),
                    role: edge.role(),
                    target: edge.target(),
                },
            )
            .collect(),
        requests: atomic
            .requests()
            .iter()
            .map(
                |(_, request)| mizar_checker::source_atomic_formula::SourceAtomicRequestInput {
                    formula: request.formula(),
                    ordinal: request.ordinal(),
                    kind: request.kind(),
                    edge: request.edge(),
                    candidate: request.candidate(),
                    type_site: request.type_site(),
                    attribute: request.attribute(),
                },
            )
            .collect(),
    }
}

const TASK258A_SOURCE: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementReservedVariableEqualitySmoke: x = x;\n",
);
const TASK258A_LABEL: &str = "FormulaStatementReservedVariableEqualitySmoke";
const TASK258A_STATEMENT_SPELLING: &str =
    "theorem FormulaStatementReservedVariableEqualitySmoke : x = x ;";
const TASK258A_LOWER_SOURCE: &str = concat!(
    "import parser.type_fixtures;\n",
    "theorem FormulaPredicateChainPayloadBoundary: ",
    "1 divides 2 does not divides 3;\n",
);

fn task258_native_output(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
) -> (mizar_checker::typed_ast::TypedAst, mizar_checker::resolved_typed_ast::ResolvedTypedAst) {
    let (source, symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(ast, module)
        .expect("native source recollection")
        .expect("native theorem module");
    let (scope, labels, _) = mizar_resolve::symbols::source_proof_inputs(&source, &symbols)
        .expect("native source occurrences");
    let (_, _, base) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(
        &source, &symbols, &scope,
    )
    .expect("native equality base");
    let typed = base.clone()
        .with_source_statement_from_original(&source, &symbols, &scope)
        .expect("native statement installation");
    let statement = typed.source_statement().expect("installed native statement");
    let entries = typed.source_node_entries().expect("installed source table").collect::<Vec<_>>();
    let mut preinstalled = base.clone();
    preinstalled.install_source_nodes(&source).expect("independent canonical source table");
    assert_eq!(preinstalled.source_node_entries().unwrap().collect::<Vec<_>>(), entries);
    assert_eq!(preinstalled.with_source_statement_from_original(&source, &symbols, &scope),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
    assert_eq!(typed.clone().with_source_statement_from_original(&source, &symbols, &scope),
        Err(mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement));
    assert_eq!(base.with_source_statement_from_original(&source, &symbols, &scope).unwrap(), typed);
    let sites = entries.iter().filter(|entry| matches!(entry.kind(),
        mizar_syntax::SurfaceNodeKind::TheoremItem | mizar_syntax::SurfaceNodeKind::CompactStatement
        | mizar_syntax::SurfaceNodeKind::AssumptionStatement | mizar_syntax::SurfaceNodeKind::ConclusionStatement
        | mizar_syntax::SurfaceNodeKind::TakeStatement)).collect::<Vec<_>>();
    let mut statement_index = 0;
    for (source_ordinal, entry) in sites.iter().enumerate() {
        if matches!(entry.kind(), mizar_syntax::SurfaceNodeKind::TakeStatement) { continue; }
        let row = statement.statements().get(mizar_checker::source_statement::SourceStatementId::new(statement_index)).unwrap();
        assert_eq!((row.site().node().index(), row.source_range(), row.source_ordinal(), row.spelling()),
            (entry.site().index(), entry.range(), source_ordinal, entry.spelling()));
        assert_eq!(row.owner().index(), 0);
        statement_index += 1;
    }
    assert_eq!(statement_index, statement.statements().len());
    let owner = statement.owners().iter().next().unwrap().1;
    assert_eq!((owner.site().node().index(), owner.source_range()), (sites[0].site().index(), sites[0].range()));
    let owner_label = &ast.nodes()[ast.nodes()[sites[0].site().index()].children[1].index()];
    let mizar_syntax::SurfaceNodeKind::Token(label) = &owner_label.kind else { panic!("native theorem label") };
    assert_eq!(owner.spelling(), label.text.as_ref());
    assert_eq!(typed.source_term().expect("native primary terms").references().iter()
        .map(|(_, reference)| (reference.lexical_scope().unwrap(), reference.use_ordinal(), reference.binding().index()))
        .collect::<Vec<_>>(), scope.references().iter()
        .map(|reference| (reference.scope(), reference.ordinal(), reference.binding().index()))
        .collect::<Vec<_>>());
    let resolved = assemble_empty_resolved_typed_ast(&typed, Vec::new())
        .expect("native statement final assembly");
    assert_eq!(resolved.source_statement(), Some(statement));
    if ast.nodes().iter().any(|node| matches!(node.kind, mizar_syntax::ast::SurfaceNodeKind::CompactStatement)) {
        assert_eq!(labels.references().len(), 1);
        let references = typed.source_statement_references().expect("installed native references");
        assert_eq!(resolved.source_statement_references(), Some(references));
    } else {
        assert!(labels.references().is_empty());
        assert!(typed.source_statement_references().is_none());
        assert!(resolved.source_statement_references().is_none());
    }
    let takes = ast.nodes().iter().filter(|node| matches!(node.kind, mizar_syntax::ast::SurfaceNodeKind::TakeStatement)).count();
    let witnesses = ast.nodes().iter().filter(|node| matches!(node.kind, mizar_syntax::ast::SurfaceNodeKind::Witness)).count();
    assert_eq!(takes == 0, witnesses == 0, "native take/witness structure");
    let mut witness_count = 0;
    let mut named = 0;
    for (take_index, take) in ast.nodes().iter().enumerate().filter(|(_, node)| matches!(node.kind, mizar_syntax::ast::SurfaceNodeKind::TakeStatement)) {
        let take_witnesses = take.children.iter().filter(|child| matches!(ast.nodes()[child.index()].kind, mizar_syntax::ast::SurfaceNodeKind::Witness)).collect::<Vec<_>>();
        assert_eq!(take.children.len(), 2 * take_witnesses.len() + 1);
        for (ordinal, witness_id) in take_witnesses.into_iter().enumerate() {
        let witness = &ast.nodes()[witness_id.index()];
        assert!(matches!(witness.kind, mizar_syntax::ast::SurfaceNodeKind::Witness));
        let handoff = typed.source_statement_witnesses().expect("native witness handoff");
        let row = handoff.witnesses().get(mizar_checker::source_statement::SourceStatementWitnessId::new(witness_count)).expect("native witness row");
        assert_eq!(row.site().node().index(), witness_id.index());
        assert_eq!(row.take_site().node().index(), take_index);
        assert_eq!(row.take_range(), take.range);
        assert_eq!(row.ordinal(), ordinal);
        assert_eq!(row.source_range(), witness.range);
        let entry = entries.iter().find(|entry| entry.site().index() == witness_id.index()).unwrap();
        assert_eq!((entry.range(), entry.parent().map(|site| site.index()), entry.spelling()),
            (row.source_range(), Some(take_index), row.spelling()));
        assert_eq!(row.owner().index(), 0);
        assert_eq!(row.source_ordinal(), sites.iter().position(|entry| entry.site().index() == take_index).unwrap());
        match witness.children.as_slice() {
            [rhs] => {
                assert!(matches!(ast.nodes()[rhs.index()].kind, mizar_syntax::ast::SurfaceNodeKind::TermExpression));
                assert_eq!(row.kind(), mizar_checker::source_statement::SourceStatementWitnessKind::Unnamed);
                assert!(row.name().is_none());
            }
            [name_id, equals, rhs] => {
                let name_node = &ast.nodes()[name_id.index()];
                let mizar_syntax::SurfaceNodeKind::Token(token) = &name_node.kind else { panic!("native witness name token") };
                assert_eq!(token.kind, mizar_syntax::SurfaceTokenKind::Identifier);
                assert!(matches!(&ast.nodes()[equals.index()].kind, mizar_syntax::SurfaceNodeKind::Token(token) if token.text.as_ref() == "="));
                assert!(matches!(ast.nodes()[rhs.index()].kind, mizar_syntax::ast::SurfaceNodeKind::TermExpression));
                assert_eq!(row.kind(), mizar_checker::source_statement::SourceStatementWitnessKind::Named);
                let name_id = row.name().expect("native witness forward name link");
                assert_eq!(name_id.index(), named);
                let name = handoff.names().get(name_id).expect("native name row");
                assert_eq!(name.witness().index(), witness_count);
                assert_eq!(name.site().node().index(), witness.children[0].index());
                assert_eq!(name.source_range(), name_node.range);
                assert_eq!(name.source_range().source_id, ast.source_id);
                assert_eq!(name.spelling(), token.text.as_ref());
                let entry = entries.iter().find(|entry| entry.site().index() == name.site().node().index()).unwrap();
                assert_eq!((entry.range(), entry.parent().map(|site| site.index()), entry.spelling()),
                    (name.source_range(), Some(witness_id.index()), name.spelling()));
                assert!(!typed.source_term().unwrap().terms().iter().any(|(_, term)| term.site() == name.site()));
                assert_eq!(statement.binding_env().bindings().len(), 1);
                named += 1;
            }
            _ => panic!("native witness child shape"),
        }
            witness_count += 1;
        }
    }
    assert_eq!(witness_count, witnesses);
    assert_eq!(typed.source_statement_witnesses().is_some(), takes != 0);
    assert_eq!(typed.source_statement_witnesses(), resolved.source_statement_witnesses());
    if let Some(handoff) = typed.source_statement_witnesses() {
        assert_eq!(handoff.witnesses().len(), witnesses);
        assert_eq!(handoff.names().len(), named);
        assert!(typed.source_statement_references().is_none());
    }
    (typed, resolved)
}

#[test]
fn source_statement_native_finite_enumeration_original_transport() {
    use mizar_checker::{source_set_term::*, source_term::*, type_checker::*};
    use mizar_syntax::SurfaceNodeKind as K;
    // EN13.4.1/13.8.6: ordered finite term lists preserve occurrences, not set deduplication.
    for count in 0..=3 {
        for variant in 0..6 {
            let named = variant >= 3;
            let layout = variant % 3;
            let name = if layout == 0 { "x" } else { "parcel" };
            let elements = vec![name; count].join(if layout == 2 {
                ", :: next occurrence\n"
            } else {
                ", "
            });
            // EN15.4.4/15.11.5: the unused name is syntax; activation remains downstream.
            let prefix = if named { "item = " } else { "" };
            let text = format!(
                "reserve {name} for set;\ntheorem Finite{variant}: {name} = {name} proof\n take {prefix}{{{elements}}};\n thus {name} = {name}; end;"
            );
            let (ast, module, _, _, diagnostics) =
                task253_ast_from_source_text_with_diagnostic_count(
                    &text,
                    47_000 + count * 6 + variant,
                );
            assert_eq!(diagnostics, 0);
            let (source, symbols) =
                mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module)
                    .unwrap()
                    .unwrap();
            let (scope, _, _) =
                mizar_resolve::symbols::source_proof_inputs(&source, &symbols).unwrap();
            let before = (
                source.clone(),
                symbols.clone(),
                scope.clone(),
                ast.snapshot_text(),
            );
            let (typed, resolved) = task258_native_output(&ast, &module);
            let (_, inference, lower) =
                TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope)
                    .unwrap();
            assert_eq!(typed.source_term(), lower.source_term());
            assert_eq!(typed.source_set_term(), lower.source_set_term());
            assert_eq!(typed.source_atomic_formula(), lower.source_atomic_formula());
            let primary = typed.source_term().unwrap();
            let set = typed.source_set_term().unwrap();
            let (set_id, enumeration) = set.terms().iter().next().unwrap();
            let actual = ast
                .node_views()
                .find(|node| node.kind() == &K::SetEnumeration)
                .unwrap();
            assert_eq!(enumeration.source_range(), actual.range());
            assert_eq!(enumeration.kind(), SourceSetTermKind::Enumeration);
            assert_eq!(
                enumeration.spelling(),
                if count == 0 {
                    "{ }".to_owned()
                } else {
                    format!("{{ {} }}", vec![name; count].join(" , "))
                }
            );
            assert_eq!(
                (
                    set.terms().len(),
                    set.edges().len(),
                    set.requests().len(),
                    primary.terms().len(),
                    primary.references().len()
                ),
                (1, count, 1, 4 + count, 4 + count)
            );
            assert!(
                set.wrappers().is_empty()
                    && set.generators().is_empty()
                    && set.conditions().is_empty()
                    && set.type_sites().is_empty()
            );
            assert_eq!(set.primary_term_fingerprint(), primary.debug_text());
            let (_, request) = set.requests().iter().next().unwrap();
            assert_eq!(
                (
                    request.term(),
                    request.ordinal(),
                    request.kind(),
                    request.generator(),
                    request.type_site()
                ),
                (set_id, 0, SourceSetRequestKind::ResultType, None, None)
            );
            let expressions = actual
                .children()
                .iter()
                .filter(|id| ast.node(**id).unwrap().kind == K::TermExpression)
                .collect::<Vec<_>>();
            assert_eq!(expressions.len(), count);
            let mut sites = std::collections::BTreeSet::new();
            let mut references = std::collections::BTreeSet::new();
            for (ordinal, ((_, edge), expression)) in
                set.edges().iter().zip(expressions).enumerate()
            {
                let expression_site = *expression;
                let expression = ast.node(expression_site).unwrap();
                let reference_site = expression.children[0];
                let SourceSetTarget::Primary(id) = edge.target() else {
                    panic!("finite enumeration primary")
                };
                let child = primary.terms().get(id).unwrap();
                let (_, reference) = primary
                    .references()
                    .iter()
                    .find(|(_, row)| row.term() == id)
                    .unwrap();
                let actual_reference = scope
                    .references()
                    .iter()
                    .find(|row| {
                        source.resolved_node_for(row.node())
                            == source.resolved_node_for(reference_site)
                    })
                    .unwrap();
                assert!(sites.insert(child.site().node()));
                assert!(references.insert(reference.use_ordinal()));
                assert_eq!(
                    (edge.term(), edge.ordinal(), edge.role()),
                    (set_id, ordinal, SourceSetEdgeRole::EnumerationElement)
                );
                assert_eq!(
                    typed
                        .nodes()
                        .node(child.site().node())
                        .unwrap()
                        .resolved_node,
                    source.resolved_node_for(reference_site)
                );
                assert_eq!(
                    (
                        child.source_range(),
                        child.spelling(),
                        child.kind(),
                        child.role(),
                        child.context(),
                        child.source_ordinal()
                    ),
                    (
                        expression.range,
                        name,
                        SourcePrimaryTermKind::VariableReference,
                        SourcePrimaryTermRole::Value,
                        enumeration.context(),
                        actual_reference.ordinal()
                    )
                );
                assert_eq!(
                    (
                        reference.use_ordinal(),
                        reference.lexical_scope(),
                        reference.binding().index()
                    ),
                    (
                        actual_reference.ordinal(),
                        Some(actual_reference.scope()),
                        actual_reference.binding().index()
                    )
                );
                let entry = typed
                    .source_node_entries()
                    .unwrap()
                    .find(|row| Some(row.site()) == source.resolved_node_for(reference_site))
                    .unwrap();
                assert_eq!(entry.range(), ast.node(reference_site).unwrap().range);
                assert_eq!(entry.parent(), source.resolved_node_for(expression_site));
            }
            let statements = typed.source_statement().unwrap();
            assert_eq!(
                SourceVariableSemanticsChecker::source_theorem_binding_env(
                    &source, &symbols, &scope
                )
                .unwrap_err(),
                "theorems.equality.unsupported_source_types"
            );
            assert_eq!(
                (
                    statements.binding_env().bindings().len(),
                    statements.binding_env().contexts().len()
                ),
                (1, 2)
            );
            let witnesses = typed.source_statement_witnesses().unwrap();
            assert_eq!(witnesses.names().len(), usize::from(named));
            if named {
                let witness = ast
                    .node_views()
                    .find(|node| node.kind() == &K::Witness)
                    .unwrap();
                for child in witness.children() {
                    let entry = typed
                        .source_node_entries()
                        .unwrap()
                        .find(|entry| Some(entry.site()) == source.resolved_node_for(*child))
                        .unwrap();
                    assert_eq!(
                        (entry.parent(), entry.range()),
                        (
                            source.resolved_node_for(witness.id()),
                            ast.node(*child).unwrap().range
                        )
                    );
                }
            }
            assert!(
                statements
                    .binding_env()
                    .bindings()
                    .iter()
                    .all(|(_, row)| row.spelling != "item")
            );
            assert!(
                primary
                    .terms()
                    .iter()
                    .all(|(_, row)| row.spelling() != "item")
            );
            assert_eq!(
                TermFormulaChecker::check_source_theorem_equalities_with_typed_tables(
                    &source, &symbols, &scope
                )
                .unwrap_err(),
                "theorems.equality.unsupported_source_types"
            );
            for check in [
                TermFormulaChecker::check_source_theorem_equalities_with_expression_metadata,
                TermFormulaChecker::check_source_theorem_equalities_with_local_contexts,
                TermFormulaChecker::check_source_theorem_equalities_with_scoped_inference,
            ] {
                assert_eq!(
                    check(&source, &symbols, &scope).unwrap_err(),
                    "theorems.equality.unsupported_source_types"
                );
            }
            assert_eq!(statements.statements().len(), 2);
            assert_eq!(
                typed
                    .source_statement_witnesses()
                    .unwrap()
                    .witnesses()
                    .len(),
                1
            );
            assert_eq!(
                (
                    statements.input_facts().len(),
                    statements.candidate_facts().len()
                ),
                (2, 2)
            );
            assert!(statements.input_facts().iter().all(|(_, fact)| {
                fact.uses()
                    .iter()
                    .all(|reference| !references.contains(&reference.index()))
            }));
            assert!(statements.candidate_facts().iter().all(|(_, fact)| fact.kind() == mizar_checker::source_statement::SourceStatementCandidateFactKind::UnverifiedProposition));
            assert!(inference.facts().is_empty() && inference.diagnostics().is_empty());
            assert!(
                inference
                    .formulas()
                    .iter()
                    .all(|(_, row)| row.status == FormulaStatus::Checked && row.facts.is_empty())
            );
            assert!(typed.facts().is_empty() && typed.initial_obligations().is_empty());
            assert!(
                resolved.checked_formulas().is_empty()
                    && resolved.statement_semantics().is_empty()
                    && resolved.checked_proofs().is_empty()
            );
            assert_eq!(resolved.source_set_term(), typed.source_set_term());
            assert_eq!(
                assemble_empty_resolved_typed_ast(&typed, Vec::new()).unwrap(),
                resolved
            );
            assert_eq!((source, symbols, scope, ast.snapshot_text()), before);
        }
    }
    for (index, witness) in [
        "{x, 0, x}",
        "{x, (x), x}",
        "{x, {x}, x}",
        "item = {x, 0}",
        "item = {x, (x)}",
        "item = {x, {x}}",
        "item = ({x})",
        "item = {x}, x",
        "item = {x where x is set}",
        "item = the set",
        "x = {x, x}",
        "item = {item, x}",
        "{x, x}, {x}",
        "{x where x is set}",
    ]
    .into_iter()
    .enumerate()
    {
        let text = format!(
            "reserve x for set; theorem Refused: x = x proof take {witness}; thus x = x; end;"
        );
        let (ast, module, _, _, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&text, 48_000 + index);
        assert_eq!(diagnostics, 0);
        let (source, symbols) =
            mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module)
                .unwrap()
                .unwrap();
        let Ok((scope, _, _)) = mizar_resolve::symbols::source_proof_inputs(&source, &symbols) else {
            assert_eq!(witness, "item = {item, x}");
            continue;
        };
        assert_eq!(
            TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope)
                .unwrap_err(),
            "theorems.equality.unsupported_source_types",
            "{witness}"
        );
    }
    for (index, witness) in ["item = {x,}", "item = {x", "item ="]
        .into_iter()
        .enumerate()
    {
        let text = format!(
            "reserve x for set; theorem Malformed: x = x proof take {witness}; thus x = x; end;"
        );
        let (ast, module, _, _, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&text, 49_000 + index);
        assert!(diagnostics > 0);
        assert!(ast.node_views().any(|node| node.is_recovered()));
        if let Ok(Some((source, symbols))) =
            mizar_resolve::symbols::source_theorem_module_original_inputs(&ast, &module)
            && let Ok((scope, _, _)) =
                mizar_resolve::symbols::source_proof_inputs(&source, &symbols)
        {
            assert!(
                TermFormulaChecker::check_source_theorem_equalities(&source, &symbols, &scope)
                    .is_err()
            );
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Task258B2ResolverMutation {
    Imported,
    Missing,
    Duplicate,
    WrongPath,
    WrongKind,
    Private,
    LocalOnly,
    Recovered,
}

fn task258b2_mutate_resolver(symbols: SymbolEnv, mutation: Task258B2ResolverMutation) -> SymbolEnv {
    if matches!(mutation, Task258B2ResolverMutation::Imported) {
        return statement_env_with_imported_label(symbols);
    }
    let label = symbols
        .labels()
        .iter()
        .next()
        .expect("Task258B2 exact theorem label")
        .clone();
    let mut labels = mizar_resolve::env::LabelIndex::new();
    if !matches!(mutation, Task258B2ResolverMutation::Missing) {
        let origin_path = if matches!(mutation, Task258B2ResolverMutation::WrongPath) {
            mizar_resolve::resolved_ast::LabelOriginPath::new("task258b2::wrong::theorem::origin")
        } else {
            label.origin_path().clone()
        };
        let kind = if matches!(mutation, Task258B2ResolverMutation::WrongKind) {
            mizar_resolve::resolved_ast::LabelKind::ProofStep
        } else {
            label.kind()
        };
        let origin = if matches!(mutation, Task258B2ResolverMutation::Recovered) {
            label.origin().clone().recovered()
        } else {
            label.origin().clone()
        };
        let visibility = if matches!(mutation, Task258B2ResolverMutation::Private) {
            mizar_resolve::env::Visibility::Private
        } else {
            label.visibility()
        };
        let export_status = if matches!(mutation, Task258B2ResolverMutation::LocalOnly) {
            mizar_resolve::env::ExportStatus::LocalOnly
        } else {
            label.export_status()
        };
        labels.insert(
            mizar_resolve::env::LabelEntry::new(
                origin_path,
                kind,
                label.namespace().clone(),
                label.primary_spelling(),
                origin,
                label.contribution(),
            )
            .with_visibility(visibility)
            .with_export_status(export_status),
        );
        if matches!(mutation, Task258B2ResolverMutation::Duplicate) {
            labels.insert(
                mizar_resolve::env::LabelEntry::new(
                    mizar_resolve::resolved_ast::LabelOriginPath::new(
                        "task258b2::duplicate::theorem::origin",
                    ),
                    label.kind(),
                    label.namespace().clone(),
                    label.primary_spelling(),
                    label.origin().clone(),
                    label.contribution(),
                )
                .with_visibility(label.visibility())
                .with_export_status(label.export_status()),
            );
        }
    }
    SymbolEnv::new(
        symbols.module_id().clone(),
        mizar_resolve::env::SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: symbols.symbols().clone(),
            labels,
            definitions: symbols.definitions().clone(),
            overloads: symbols.overloads().clone(),
            registrations: symbols.registrations().clone(),
            lexical_summaries: symbols.lexical_summaries().clone(),
            namespace_graph: symbols.namespace_graph().clone(),
            declaration_dependencies: symbols.declaration_dependencies().clone(),
            contributions: symbols.contributions().clone(),
            module_summaries: symbols.module_summaries().clone(),
        },
    )
}

#[derive(Debug, Clone, Copy)]
enum Task258B3M2B2B3ALabelMutation {
    Namespace,
    PrimarySpelling,
    Contribution,
    OriginSource,
    OriginModule,
    OriginAnchor,
    OriginStructuralPath,
    ContributionEffectsMissing,
    ContributionEffectsExtra,
    ContributionEffectsWrongPath,
}

fn task258b3m2b2b3a_foreign_source_id(current: mizar_session::SourceId) -> mizar_session::SourceId {
    use mizar_session::SessionIdAllocator as _;

    let snapshot = mizar_session::BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "b3".repeat(mizar_session::Hash::BYTE_LEN)
    ))
    .expect("Task258B3M2B2B3A label-mutation snapshot");
    let allocator = mizar_session::InMemorySessionIdAllocator::new();
    loop {
        let candidate = allocator
            .next_source_id(snapshot)
            .expect("Task258B3M2B2B3A label-mutation source");
        if candidate != current {
            return candidate;
        }
    }
}

fn task258b3m2b2b3a_rebuild_contributions_with_label_effects(
    symbols: &SymbolEnv,
    target: mizar_resolve::env::SourceContributionId,
    label_effects: &[mizar_resolve::resolved_ast::LabelOriginPath],
) -> mizar_resolve::env::SourceContributionIndex {
    let mut rebuilt = mizar_resolve::env::SourceContributionIndex::new();
    for record in symbols.contributions().iter() {
        let id = rebuilt.insert(
            record.module().clone(),
            record.kind().clone(),
            record.anchor().clone(),
        );
        assert_eq!(id, record.id(), "contribution identity");
        let effects = record.effects();
        for symbol in effects.symbols() {
            rebuilt.add_symbol(id, symbol.clone());
        }
        for definition in effects.definitions() {
            rebuilt.add_definition(id, *definition);
        }
        for group in effects.overload_groups() {
            rebuilt.add_overload_group(id, *group);
        }
        for registration in effects.registrations() {
            rebuilt.add_registration(id, *registration);
        }
        for summary in effects.lexical_summaries() {
            rebuilt.add_lexical_summary(id, *summary);
        }
        let labels = if id == target {
            label_effects
        } else {
            effects.labels()
        };
        for label in labels {
            rebuilt.add_label(id, label.clone());
        }
        for edge in effects.namespace_edges() {
            rebuilt.add_namespace_edge(id, *edge);
        }
        for dependency in effects.declaration_dependencies() {
            rebuilt.add_declaration_dependency(id, *dependency);
        }
        for import in effects.imports() {
            rebuilt.add_import(id, *import);
        }
        for export in effects.exports() {
            rebuilt.add_export(id, *export);
        }
        for diagnostic in effects.diagnostics() {
            rebuilt.add_diagnostic(id, *diagnostic);
        }
    }
    rebuilt
}

fn task258b3m2b2b3a_mutate_label_resolver(
    symbols: SymbolEnv,
    mutation: Task258B3M2B2B3ALabelMutation,
) -> SymbolEnv {
    let label = symbols
        .labels()
        .iter()
        .next()
        .expect("Task258B3M2B2B3A exact theorem label")
        .clone();
    let module = symbols.module_id().clone();
    let mut contributions = symbols.contributions().clone();
    let namespace = if matches!(mutation, Task258B3M2B2B3ALabelMutation::Namespace) {
        mizar_resolve::env::NamespacePath::new("statement.foreign-label-namespace")
    } else {
        label.namespace().clone()
    };
    let primary_spelling = if matches!(mutation, Task258B3M2B2B3ALabelMutation::PrimarySpelling) {
        "FormulaStatementSetEnumerationWitnessNearMiss"
    } else {
        label.primary_spelling()
    };
    let contribution = if matches!(mutation, Task258B3M2B2B3ALabelMutation::Contribution) {
        let owner = contributions
            .get(label.contribution())
            .expect("owner contribution")
            .clone();
        contributions.insert(
            owner.module().clone(),
            owner.kind().clone(),
            owner.anchor().clone(),
        )
    } else {
        label.contribution()
    };
    let origin_source = if matches!(mutation, Task258B3M2B2B3ALabelMutation::OriginSource) {
        task258b3m2b2b3a_foreign_source_id(label.origin().source_id())
    } else {
        label.origin().source_id()
    };
    let origin_module = if matches!(mutation, Task258B3M2B2B3ALabelMutation::OriginModule) {
        mizar_resolve::resolved_ast::ModuleId::new(
            module.package().clone(),
            mizar_session::ModulePath::new("statement.foreign-label-origin"),
        )
    } else {
        label.origin().module_id().clone()
    };
    let origin_anchor = if matches!(mutation, Task258B3M2B2B3ALabelMutation::OriginAnchor) {
        mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: label.origin().source_id(),
            start: 20,
            end: 116,
        })
    } else {
        label.origin().anchor().clone()
    };
    let origin_path = if matches!(
        mutation,
        Task258B3M2B2B3ALabelMutation::OriginStructuralPath
    ) {
        vec![2, 9]
    } else {
        label.origin().structural_path().to_vec()
    };
    let origin = mizar_resolve::resolved_ast::SemanticOrigin::new(
        origin_source,
        origin_module,
        origin_anchor,
        origin_path,
    );
    let mut labels = mizar_resolve::env::LabelIndex::new();
    labels.insert(
        mizar_resolve::env::LabelEntry::new(
            label.origin_path().clone(),
            label.kind(),
            namespace,
            primary_spelling,
            origin,
            contribution,
        )
        .with_visibility(label.visibility())
        .with_export_status(label.export_status()),
    );
    let expected_path = label.origin_path().clone();
    contributions = match mutation {
        Task258B3M2B2B3ALabelMutation::ContributionEffectsMissing => {
            task258b3m2b2b3a_rebuild_contributions_with_label_effects(
                &symbols,
                label.contribution(),
                &[],
            )
        }
        Task258B3M2B2B3ALabelMutation::ContributionEffectsExtra => {
            task258b3m2b2b3a_rebuild_contributions_with_label_effects(
                &symbols,
                label.contribution(),
                &[
                    expected_path,
                    mizar_resolve::resolved_ast::LabelOriginPath::new(
                        "statement::extra::theorem::label",
                    ),
                ],
            )
        }
        Task258B3M2B2B3ALabelMutation::ContributionEffectsWrongPath => {
            task258b3m2b2b3a_rebuild_contributions_with_label_effects(
                &symbols,
                label.contribution(),
                &[mizar_resolve::resolved_ast::LabelOriginPath::new(
                    "statement::wrong::theorem::label",
                )],
            )
        }
        _ => contributions,
    };
    SymbolEnv::new(
        module,
        mizar_resolve::env::SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: symbols.symbols().clone(),
            labels,
            definitions: symbols.definitions().clone(),
            overloads: symbols.overloads().clone(),
            registrations: symbols.registrations().clone(),
            lexical_summaries: symbols.lexical_summaries().clone(),
            namespace_graph: symbols.namespace_graph().clone(),
            declaration_dependencies: symbols.declaration_dependencies().clone(),
            contributions,
            module_summaries: symbols.module_summaries().clone(),
        },
    )
}

#[derive(Debug, Clone, Copy)]
enum Task258B4CRawResolverMutation {
    OwnerOriginSource,
    OwnerOriginModule,
    OwnerOriginRange,
    OwnerOriginPath,
    OwnerKind,
    OwnerVisibility,
    OwnerExport,
    OwnerSpelling,
    ContributionModule,
    ContributionKind,
    ContributionSource,
    ContributionAnchor,
    ContributionSymbolEffects,
    ContributionDefinitionEffects,
}

fn task258b4c_mutate_raw_resolver(
    symbols: SymbolEnv,
    mutation: Task258B4CRawResolverMutation,
) -> SymbolEnv {
    let module = symbols.module_id().clone();
    let owner = symbols
        .symbols()
        .iter()
        .next()
        .expect("Task258B4C raw owner")
        .clone();
    let source = owner.origin().source_id();
    let foreign_source = task258b3m2b2b3a_foreign_source_id(source);
    let origin_source = if matches!(mutation, Task258B4CRawResolverMutation::OwnerOriginSource) {
        foreign_source
    } else {
        source
    };
    let origin_module = if matches!(mutation, Task258B4CRawResolverMutation::OwnerOriginModule) {
        mizar_resolve::resolved_ast::ModuleId::new(
            module.package().clone(),
            mizar_session::ModulePath::new("statement.b4c.raw-wrong"),
        )
    } else {
        owner.origin().module_id().clone()
    };
    let origin_anchor = if matches!(mutation, Task258B4CRawResolverMutation::OwnerOriginRange) {
        mizar_session::SourceAnchor::Range(range(source, 19, 136))
    } else {
        owner.origin().anchor().clone()
    };
    let origin_path = if matches!(mutation, Task258B4CRawResolverMutation::OwnerOriginPath) {
        vec![2, 9]
    } else {
        owner.origin().structural_path().to_vec()
    };
    let kind = if matches!(mutation, Task258B4CRawResolverMutation::OwnerKind) {
        mizar_resolve::env::SymbolKind::Builtin
    } else {
        owner.kind()
    };
    let visibility = if matches!(mutation, Task258B4CRawResolverMutation::OwnerVisibility) {
        mizar_resolve::env::Visibility::Private
    } else {
        owner.visibility()
    };
    let export_status = if matches!(mutation, Task258B4CRawResolverMutation::OwnerExport) {
        mizar_resolve::env::ExportStatus::LocalOnly
    } else {
        owner.export_status()
    };
    let spelling = if matches!(mutation, Task258B4CRawResolverMutation::OwnerSpelling) {
        "FormulaNestedQuantifierPayloadNearMiss"
    } else {
        owner.primary_spelling()
    };
    let mut replacement = mizar_resolve::env::SymbolEntry::new(
        owner.symbol().clone(),
        kind,
        owner.namespace().clone(),
        spelling,
        mizar_resolve::resolved_ast::SemanticOrigin::new(
            origin_source,
            origin_module,
            origin_anchor,
            origin_path,
        ),
        owner.contribution(),
    )
    .with_visibility(visibility)
    .with_export_status(export_status)
    .with_relations(owner.relations().to_vec());
    if let Some(notation) = owner.notation_spelling() {
        replacement = replacement.with_notation_spelling(notation);
    }
    if let Some(signature) = owner.signature().cloned() {
        replacement = replacement.with_signature(signature);
    }
    let mut owner_index = mizar_resolve::env::SymbolIndex::new();
    owner_index.insert(replacement);

    let record = symbols
        .contributions()
        .iter()
        .next()
        .expect("Task258B4C raw contribution");
    let contribution_module =
        if matches!(mutation, Task258B4CRawResolverMutation::ContributionModule) {
            mizar_resolve::resolved_ast::ModuleId::new(
                module.package().clone(),
                mizar_session::ModulePath::new("statement.b4c.contribution-wrong"),
            )
        } else {
            record.module().clone()
        };
    let contribution_kind = if matches!(mutation, Task258B4CRawResolverMutation::ContributionKind) {
        mizar_resolve::env::ContributionKind::ImportedSource { source_id: source }
    } else if matches!(mutation, Task258B4CRawResolverMutation::ContributionSource) {
        mizar_resolve::env::ContributionKind::LocalSource {
            source_id: foreign_source,
        }
    } else {
        record.kind().clone()
    };
    let contribution_anchor =
        if matches!(mutation, Task258B4CRawResolverMutation::ContributionAnchor) {
            mizar_session::SourceAnchor::Range(range(source, 0, 17))
        } else {
            record.anchor().clone()
        };
    let mut contributions = mizar_resolve::env::SourceContributionIndex::new();
    let contribution =
        contributions.insert(contribution_module, contribution_kind, contribution_anchor);
    assert_eq!(contribution, record.id(), "Task258B4C contribution id");
    let effects = record.effects();
    if !matches!(
        mutation,
        Task258B4CRawResolverMutation::ContributionSymbolEffects
    ) {
        for symbol in effects.symbols() {
            contributions.add_symbol(contribution, symbol.clone());
        }
    }
    if !matches!(
        mutation,
        Task258B4CRawResolverMutation::ContributionDefinitionEffects
    ) {
        for definition in effects.definitions() {
            contributions.add_definition(contribution, *definition);
        }
    }
    for group in effects.overload_groups() {
        contributions.add_overload_group(contribution, *group);
    }
    for registration in effects.registrations() {
        contributions.add_registration(contribution, *registration);
    }
    for summary in effects.lexical_summaries() {
        contributions.add_lexical_summary(contribution, *summary);
    }
    for label in effects.labels() {
        contributions.add_label(contribution, label.clone());
    }
    for edge in effects.namespace_edges() {
        contributions.add_namespace_edge(contribution, *edge);
    }
    for dependency in effects.declaration_dependencies() {
        contributions.add_declaration_dependency(contribution, *dependency);
    }
    for import in effects.imports() {
        contributions.add_import(contribution, *import);
    }
    for export in effects.exports() {
        contributions.add_export(contribution, *export);
    }
    for diagnostic in effects.diagnostics() {
        contributions.add_diagnostic(contribution, *diagnostic);
    }

    SymbolEnv::new(
        module,
        mizar_resolve::env::SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: owner_index,
            labels: symbols.labels().clone(),
            definitions: symbols.definitions().clone(),
            overloads: symbols.overloads().clone(),
            registrations: symbols.registrations().clone(),
            lexical_summaries: symbols.lexical_summaries().clone(),
            namespace_graph: symbols.namespace_graph().clone(),
            declaration_dependencies: symbols.declaration_dependencies().clone(),
            contributions,
            module_summaries: symbols.module_summaries().clone(),
        },
    )
}


fn statement_env_with_imported_label(symbols: SymbolEnv) -> SymbolEnv {
    let label = symbols
        .labels()
        .iter()
        .next()
        .expect("Task258A exact label")
        .clone();
    let mut nodes = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
    let owner = nodes
        .push(mizar_resolve::resolved_ast::ResolvedNode::new(
            mizar_syntax::SurfaceNodeKind::ImportAliasDecl,
            Vec::new(),
            label.origin().clone(),
        ))
        .expect("Task258A imported-label owner");
    let source_range = mizar_session::SourceRange {
        source_id: label.origin().source_id(),
        start: 0,
        end: 1,
    };
    let mut imports = mizar_resolve::resolved_ast::ResolvedImports::new();
    let import = imports.push_import(mizar_resolve::resolved_ast::ResolvedImport::new(
        owner,
        source_range,
        "import statement.fixture;",
        None,
        mizar_resolve::resolved_ast::ImportResolution::Resolved(symbols.module_id().clone()),
        label.origin().clone(),
    ));
    let mut labels = mizar_resolve::env::LabelIndex::new();
    labels.insert(
        mizar_resolve::env::LabelEntry::new(
            label.origin_path().clone(),
            label.kind(),
            label.namespace().clone(),
            label.primary_spelling(),
            label.origin().clone().with_import_edge(import),
            label.contribution(),
        )
        .with_visibility(label.visibility())
        .with_export_status(label.export_status()),
    );
    SymbolEnv::new(
        symbols.module_id().clone(),
        mizar_resolve::env::SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: symbols.symbols().clone(),
            labels,
            definitions: symbols.definitions().clone(),
            overloads: symbols.overloads().clone(),
            registrations: symbols.registrations().clone(),
            lexical_summaries: symbols.lexical_summaries().clone(),
            namespace_graph: symbols.namespace_graph().clone(),
            declaration_dependencies: symbols.declaration_dependencies().clone(),
            contributions: symbols.contributions().clone(),
            module_summaries: symbols.module_summaries().clone(),
        },
    )
}

fn task258b3m2b2b1a_fixture(
    source_ordinal: usize,
) -> (
    mizar_syntax::ast::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
    mizar_resolve::env::SymbolEnv,
) {
    let (ast, module, _, symbols, diagnostics) = task253_ast_from_source_text_with_diagnostic_count(
        SOURCE_STATEMENT_B3M2B2B1A_TEXT,
        source_ordinal,
    );
    assert_eq!(diagnostics, 0);
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, symbols);
    (ast, module, symbols)
}

fn task258b3m2b2b1b1_fixture(
    source_ordinal: usize,
) -> (
    mizar_syntax::ast::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
    mizar_resolve::env::SymbolEnv,
) {
    let (ast, module, _, symbols, diagnostics) = task253_ast_from_source_text_with_diagnostic_count(
        SOURCE_STATEMENT_B3M2B2B1B1_TEXT,
        source_ordinal,
    );
    assert_eq!(diagnostics, 0);
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, symbols);
    (ast, module, symbols)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Task258B3M2B2B1B1ResolverMutation {
    LocalAndFqn,
    StructuralPath,
    Signature,
    ExportStatus,
    Contribution,
}

fn task258b3m2b2b1b1_substituted_symbol_env(
    symbols: &SymbolEnv,
    mutation: Task258B3M2B2B1B1ResolverMutation,
) -> SymbolEnv {
    let selected = symbols
        .symbols()
        .iter()
        .find(|entry| {
            entry.symbol().module().path().as_str() == "parser.type_fixtures"
                && entry.kind() == mizar_resolve::env::SymbolKind::Functor
                && entry.primary_spelling() == "++"
        })
        .expect("Task258B3M2B2B1B1 imported ++ entry")
        .clone();
    let mut indexes = super::import_fixtures::clone_symbol_env_indexes(symbols);
    let contribution = if mutation == Task258B3M2B2B1B1ResolverMutation::Contribution {
        indexes.contributions.insert(
            selected.symbol().module().clone(),
            mizar_resolve::env::ContributionKind::ImportedSource {
                source_id: selected.origin().source_id(),
            },
            selected.origin().anchor().clone(),
        )
    } else {
        selected.contribution()
    };
    let symbol = if mutation == Task258B3M2B2B1B1ResolverMutation::LocalAndFqn {
        mizar_resolve::resolved_ast::SymbolId::new(
            selected.symbol().module().clone(),
            mizar_resolve::resolved_ast::LocalSymbolId::new(
                "summary:parser.type_fixtures#parse-only#++:substitute",
            ),
            mizar_resolve::resolved_ast::FullyQualifiedName::new(
                "parser.type_fixtures::++#substitute",
            ),
        )
    } else {
        selected.symbol().clone()
    };
    let structural_path = if mutation == Task258B3M2B2B1B1ResolverMutation::StructuralPath {
        vec![13]
    } else {
        selected.origin().structural_path().to_vec()
    };
    let origin = mizar_resolve::resolved_ast::SemanticOrigin::new(
        selected.origin().source_id(),
        selected.origin().module_id().clone(),
        selected.origin().anchor().clone(),
        structural_path,
    );
    let export_status = if mutation == Task258B3M2B2B1B1ResolverMutation::ExportStatus {
        mizar_resolve::env::ExportStatus::ReExported
    } else {
        selected.export_status()
    };
    let mut replacement = mizar_resolve::env::SymbolEntry::new(
        symbol.clone(),
        selected.kind(),
        selected.namespace().clone(),
        selected.primary_spelling(),
        origin,
        contribution,
    )
    .with_visibility(selected.visibility())
    .with_export_status(export_status)
    .with_relations(selected.relations().to_vec());
    if let Some(notation) = selected.notation_spelling() {
        replacement = replacement.with_notation_spelling(notation);
    }
    if mutation == Task258B3M2B2B1B1ResolverMutation::Signature {
        replacement = replacement.with_signature(mizar_resolve::env::SignatureShell::Pending);
    } else if let Some(signature) = selected.signature().cloned() {
        replacement = replacement.with_signature(signature);
    }
    indexes.symbols = Default::default();
    for entry in symbols.symbols().iter() {
        indexes
            .symbols
            .insert(if entry.symbol() == selected.symbol() {
                replacement.clone()
            } else {
                entry.clone()
            });
    }
    indexes.contributions.add_symbol(contribution, symbol);
    SymbolEnv::new(symbols.module_id().clone(), indexes)
}

fn task258b3m2b2b2a_fixture(
    source_ordinal: usize,
) -> (
    mizar_syntax::ast::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
    mizar_resolve::env::SymbolEnv,
) {
    let (ast, module, _, symbols, diagnostics) = task253_ast_from_source_text_with_diagnostic_count(
        SOURCE_STATEMENT_B3M2B2B2A_TEXT,
        source_ordinal,
    );
    assert_eq!(diagnostics, 0);
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, symbols);
    (ast, module, symbols)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Task258B3M2B2B2AResolverMutation {
    LocalAndFqn,
    StructuralPath,
    Signature,
    ExportStatus,
    Contribution,
}

fn task258b3m2b2b2a_substituted_structure_env(
    symbols: &SymbolEnv,
    mutation: Task258B3M2B2B2AResolverMutation,
) -> SymbolEnv {
    let selected = symbols
        .symbols()
        .iter()
        .find(|entry| {
            entry.symbol().module().path().as_str() == "parser.type_fixtures"
                && entry.kind() == mizar_resolve::env::SymbolKind::Structure
                && entry.primary_spelling() == "TypeCaseStruct"
        })
        .expect("Task258B3M2B2B2A imported TypeCaseStruct")
        .clone();
    let mut indexes = super::import_fixtures::clone_symbol_env_indexes(symbols);
    let contribution = if mutation == Task258B3M2B2B2AResolverMutation::Contribution {
        indexes.contributions.insert(
            selected.symbol().module().clone(),
            mizar_resolve::env::ContributionKind::ImportedSource {
                source_id: selected.origin().source_id(),
            },
            selected.origin().anchor().clone(),
        )
    } else {
        selected.contribution()
    };
    let symbol = if mutation == Task258B3M2B2B2AResolverMutation::LocalAndFqn {
        mizar_resolve::resolved_ast::SymbolId::new(
            selected.symbol().module().clone(),
            mizar_resolve::resolved_ast::LocalSymbolId::new(
                "summary:parser.type_fixtures#parse-only#TypeCaseStruct:substitute",
            ),
            mizar_resolve::resolved_ast::FullyQualifiedName::new(
                "parser.type_fixtures::TypeCaseStruct#substitute",
            ),
        )
    } else {
        selected.symbol().clone()
    };
    let structural_path = if mutation == Task258B3M2B2B2AResolverMutation::StructuralPath {
        vec![6]
    } else {
        selected.origin().structural_path().to_vec()
    };
    let origin = mizar_resolve::resolved_ast::SemanticOrigin::new(
        selected.origin().source_id(),
        selected.origin().module_id().clone(),
        selected.origin().anchor().clone(),
        structural_path,
    );
    let export_status = if mutation == Task258B3M2B2B2AResolverMutation::ExportStatus {
        mizar_resolve::env::ExportStatus::ReExported
    } else {
        selected.export_status()
    };
    let mut replacement = mizar_resolve::env::SymbolEntry::new(
        symbol.clone(),
        selected.kind(),
        selected.namespace().clone(),
        selected.primary_spelling(),
        origin,
        contribution,
    )
    .with_visibility(selected.visibility())
    .with_export_status(export_status)
    .with_relations(selected.relations().to_vec());
    if mutation == Task258B3M2B2B2AResolverMutation::Signature {
        replacement = replacement.with_signature(mizar_resolve::env::SignatureShell::Pending);
    } else if let Some(signature) = selected.signature().cloned() {
        replacement = replacement.with_signature(signature);
    }
    indexes.symbols = Default::default();
    for entry in symbols.symbols().iter() {
        indexes
            .symbols
            .insert(if entry.symbol() == selected.symbol() {
                replacement.clone()
            } else {
                entry.clone()
            });
    }
    indexes.contributions.add_symbol(contribution, symbol);
    SymbolEnv::new(symbols.module_id().clone(), indexes)
}

fn task258b3m2b2b2a_roots() -> [(usize, mizar_checker::binding_env::BindingContextId); 6] {
    let module = mizar_checker::binding_env::BindingContextId::new(0);
    let proof = mizar_checker::binding_env::BindingContextId::new(1);
    [
        (45, module),
        (47, module),
        (54, proof),
        (57, proof),
        (63, proof),
        (65, proof),
    ]
}

fn task258b3m2b2b2b_fixture(
    source_ordinal: usize,
) -> (
    mizar_syntax::ast::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
    mizar_resolve::env::SymbolEnv,
) {
    let (ast, module, _, symbols, diagnostics) = task253_ast_from_source_text_with_diagnostic_count(
        SOURCE_STATEMENT_B3M2B2B2B_TEXT,
        source_ordinal,
    );
    assert_eq!(diagnostics, 0);
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, symbols);
    (ast, module, symbols)
}

fn task258b3m2b2b2b_roots() -> [(usize, mizar_checker::binding_env::BindingContextId); 6] {
    let module = mizar_checker::binding_env::BindingContextId::new(0);
    let proof = mizar_checker::binding_env::BindingContextId::new(1);
    [
        (47, module),
        (49, module),
        (56, proof),
        (59, proof),
        (66, proof),
        (68, proof),
    ]
}

fn task258b3m2b2b2c_fixture(
    source_ordinal: usize,
) -> (
    mizar_syntax::ast::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
    mizar_resolve::env::SymbolEnv,
) {
    let (ast, module, _, symbols, diagnostics) = task253_ast_from_source_text_with_diagnostic_count(
        SOURCE_STATEMENT_B3M2B2B2C_TEXT,
        source_ordinal,
    );
    assert_eq!(diagnostics, 0);
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, symbols);
    (ast, module, symbols)
}

fn task258b3m2b2b2c_roots() -> [(usize, mizar_checker::binding_env::BindingContextId); 7] {
    let module = mizar_checker::binding_env::BindingContextId::new(0);
    let proof = mizar_checker::binding_env::BindingContextId::new(1);
    [
        (51, module),
        (53, module),
        (60, proof),
        (63, proof),
        (67, proof),
        (73, proof),
        (75, proof),
    ]
}

fn task258b3m2b2b3a_fixture(
    source_ordinal: usize,
) -> (
    mizar_syntax::ast::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
    mizar_resolve::env::SymbolEnv,
) {
    let (ast, module, _, symbols, diagnostics) = task253_ast_from_source_text_with_diagnostic_count(
        SOURCE_STATEMENT_B3M2B2B3A_TEXT,
        source_ordinal,
    );
    assert_eq!(diagnostics, 0);
    (ast, module, symbols)
}

fn task258b3m2b2b3a_output(
    source_ordinal: usize,
) -> (
    mizar_syntax::ast::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
    mizar_resolve::env::SymbolEnv,
    SourceStatementRouteOutput,
) {
    let (ast, module, symbols) = task258b3m2b2b3a_fixture(source_ordinal);
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B3A_TEXT,
    )
    .expect("Task258B3M2B2B3A selector")
    .unwrap_or_else(|error| panic!("Task258B3M2B2B3A route failed: {error}"));
    (ast, module, symbols, output)
}

fn task258b3m2b2b3b_fixture(
    source_ordinal: usize,
) -> (
    mizar_syntax::ast::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
    mizar_resolve::env::SymbolEnv,
) {
    let (ast, module, _, symbols, diagnostics) = task253_ast_from_source_text_with_diagnostic_count(
        SOURCE_STATEMENT_B3M2B2B3B_TEXT,
        source_ordinal,
    );
    assert_eq!(diagnostics, 0);
    (ast, module, symbols)
}

fn task258b3m2b2b3c_fixture(
    source_ordinal: usize,
) -> (
    mizar_syntax::ast::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
    mizar_resolve::env::SymbolEnv,
) {
    let (ast, module, _, symbols, diagnostics) = task253_ast_from_source_text_with_diagnostic_count(
        SOURCE_STATEMENT_B3M2B2B3C_TEXT,
        source_ordinal,
    );
    assert_eq!(diagnostics, 0);
    (ast, module, symbols)
}

fn task258b3m2b2b3_family_debug(family: usize, ordinal: usize) -> String {
    let source = match family {
        0 => SOURCE_STATEMENT_B3M2B2B3A_TEXT,
        1 => SOURCE_STATEMENT_B3M2B2B3B_TEXT,
        2 => SOURCE_STATEMENT_B3M2B2B3C_TEXT,
        3 => SOURCE_STATEMENT_B3M2B2B3D_TEXT,
        4 => SOURCE_STATEMENT_B3M2B2B3E_TEXT,
        _ => unreachable!(),
    };
    let (ast, module, _, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(source, ordinal);
    assert_eq!(diagnostics, 0);
    if family == 1 || family == 2 {
        let before = (ast.snapshot_text(), module.clone(), symbols.clone());
        let (typed, resolved) = task258_native_output(&ast, &module);
        if family == 2 {
            use mizar_checker::source_set_term::{
                SourceSetRequestKind, SourceSetTermId, SourceSetTermKind, SourceSetTypeHead,
                SourceSetTypeOwner, SourceSetTypeRole, SourceSetTypeSiteId,
            };
            let (source, native_symbols) = mizar_resolve::symbols::source_theorem_module_original_inputs(
                &ast, &module,
            ).expect("native choice recollection").expect("native choice module");
            let (scope, _, _) = mizar_resolve::symbols::source_proof_inputs(&source, &native_symbols)
                .expect("native choice occurrences");
            let native_before = (source.clone(), native_symbols.clone(), scope.clone());
            let (_, _, lower) = mizar_checker::type_checker::TermFormulaChecker::check_source_theorem_equalities(
                &source, &native_symbols, &scope,
            ).expect("native choice lower");
            assert_eq!(typed.source_term(), lower.source_term());
            assert_eq!(typed.source_set_term(), lower.source_set_term());
            assert_eq!(typed.source_atomic_formula(), lower.source_atomic_formula());
            let set = typed.source_set_term().expect("native choice set");
            let term = set.terms().get(SourceSetTermId::new(0)).expect("native choice term");
            assert_eq!(term.kind(), SourceSetTermKind::Choice);
            assert_eq!((set.terms().len(), set.generators().len(), set.type_sites().len(), set.requests().len()), (1, 0, 1, 2));
            let target = set.type_sites().get(SourceSetTypeSiteId::new(0)).expect("native choice target");
            assert_eq!(target.head(), SourceSetTypeHead::BuiltinSet);
            assert_eq!(target.owner(), SourceSetTypeOwner::Term {
                term: SourceSetTermId::new(0), role: SourceSetTypeRole::ChoiceTarget,
            });
            assert_eq!(set.requests().iter().map(|(_, request)| (
                request.term().index(), request.ordinal(), request.kind(), request.generator(), request.type_site(),
            )).collect::<Vec<_>>(), [
                (0, 0, SourceSetRequestKind::ChoiceNonempty, None, Some(SourceSetTypeSiteId::new(0))),
                (0, 1, SourceSetRequestKind::ResultType, None, None),
            ]);
            assert_eq!((source, native_symbols, scope), native_before);
        }
        assert_eq!(typed.source_term(), resolved.source_term());
        assert!(typed.source_set_term().is_some());
        assert_eq!(typed.source_set_term(), resolved.source_set_term());
        assert_eq!(typed.source_atomic_formula(), resolved.source_atomic_formula());
        assert_eq!(typed.source_statement(), resolved.source_statement());
        assert_eq!(typed.source_statement_witnesses(), resolved.source_statement_witnesses());
        let witness = typed.source_statement_witnesses().expect("native set witness")
            .witnesses().iter().next().expect("native witness row").1;
        assert_eq!(
            witness.term(),
            mizar_checker::source_statement::SourceStatementWitnessTermTarget::SetTerm(
                mizar_checker::source_set_term::SourceSetTermId::new(0),
            ),
        );
        assert_eq!((ast.snapshot_text(), module.clone(), symbols.clone()), before);
        return typed.debug_text().replace(&ordinal.to_string(), "<ordinal>");
    }
    source_statement_output_with_source(&ast, module, &symbols, source)
        .expect("family selector")
        .expect("family route")
        .typed_ast
        .debug_text()
        .replace(&ordinal.to_string(), "<ordinal>")
}

fn task258b3m2b2b3d_fixture(
    source_ordinal: usize,
) -> (
    mizar_syntax::ast::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
    mizar_resolve::env::SymbolEnv,
) {
    let (ast, module, _, symbols, diagnostics) = task253_ast_from_source_text_with_diagnostic_count(
        SOURCE_STATEMENT_B3M2B2B3D_TEXT,
        source_ordinal,
    );
    assert_eq!(diagnostics, 0);
    (ast, module, symbols)
}

fn task258b3m2b2b3d_output(
    source_ordinal: usize,
) -> (
    mizar_syntax::ast::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
    mizar_resolve::env::SymbolEnv,
    SourceStatementRouteOutput,
) {
    let (ast, module, symbols) = task258b3m2b2b3d_fixture(source_ordinal);
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B3D_TEXT,
    )
    .expect("Task258B3M2B2B3D selector")
    .unwrap_or_else(|error| panic!("Task258B3M2B2B3D route failed: {error}"));
    (ast, module, symbols, output)
}

fn task258b3m2b2b3e_fixture(
    source_ordinal: usize,
) -> (
    mizar_syntax::ast::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
    mizar_resolve::env::SymbolEnv,
) {
    let (ast, module, _, symbols, diagnostics) = task253_ast_from_source_text_with_diagnostic_count(
        SOURCE_STATEMENT_B3M2B2B3E_TEXT,
        source_ordinal,
    );
    assert_eq!(diagnostics, 0);
    (ast, module, symbols)
}

fn task258b3m2b2b3e_output(
    source_ordinal: usize,
) -> (
    mizar_syntax::ast::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
    mizar_resolve::env::SymbolEnv,
    SourceStatementRouteOutput,
) {
    let (ast, module, symbols) = task258b3m2b2b3e_fixture(source_ordinal);
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B3M2B2B3E_TEXT,
    )
    .expect("Task258B3M2B2B3E selector")
    .unwrap_or_else(|error| panic!("Task258B3M2B2B3E route failed: {error}"));
    (ast, module, symbols, output)
}

fn task258b4a_output(
    seed: usize,
) -> (
    SurfaceAst,
    ResolverModuleId,
    SymbolEnv,
    SourceStatementRouteOutput,
) {
    let (ast, module, _, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(SOURCE_STATEMENT_B4A_TEXT, seed);
    assert_eq!(diagnostics, 0);
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, symbols);
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B4A_TEXT,
    )
    .expect("Task258B4A selector")
    .expect("Task258B4A output");
    (ast, module, symbols, output)
}

fn task258b4b_output(
    seed: usize,
) -> (
    SurfaceAst,
    ResolverModuleId,
    SymbolEnv,
    SourceStatementRouteOutput,
) {
    let (ast, module, _, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(SOURCE_STATEMENT_B4B_TEXT, seed);
    assert_eq!(diagnostics, 0);
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, symbols);
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B4B_TEXT,
    )
    .expect("Task258B4B selector")
    .expect("Task258B4B output");
    (ast, module, symbols, output)
}

fn task258b4c_output(
    seed: usize,
) -> (
    SurfaceAst,
    ResolverModuleId,
    SymbolEnv,
    SourceStatementRouteOutput,
) {
    let (ast, module, _, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(SOURCE_STATEMENT_B4C_TEXT, seed);
    assert_eq!(diagnostics, 0);
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, symbols);
    let output = source_statement_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        SOURCE_STATEMENT_B4C_TEXT,
    )
    .expect("Task258B4C selector")
    .expect("Task258B4C output");
    (ast, module, symbols, output)
}

fn task258_reference_syntax_base(
    typed: &mizar_checker::typed_ast::TypedAst,
) -> mizar_checker::typed_ast::TypedAst {
    mizar_checker::typed_ast::TypedAst::try_new(mizar_checker::typed_ast::TypedAstParts {
        source_id: typed.source_id(),
        module_id: typed.module_id().clone(),
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: typed.nodes().clone(),
        contexts: mizar_checker::typed_ast::LocalTypeContextTable::new(),
        types: mizar_checker::typed_ast::TypeTable::new(),
        facts: mizar_checker::typed_ast::TypeFactTable::new(),
        coercions: mizar_checker::typed_ast::CoercionTable::new(),
        initial_obligations: mizar_checker::typed_ast::InitialObligationTable::new(),
        diagnostics: mizar_checker::typed_ast::TypeDiagnosticTable::new(),
    })
    .expect("syntax-only typed base")
    .with_source_term(
        typed
            .source_term()
            .expect("Task252 handoff")
            .clone(),
    )
    .expect("Task252 install")
    .with_source_atomic_formula(
        typed
            .source_atomic_formula()
            .expect("Task256 handoff")
            .clone(),
    )
    .expect("Task256 install")
}



fn task258b5a_binding_env_with_context_recovery(
    binding_env: &mizar_checker::binding_env::BindingEnv,
    target: usize,
) -> mizar_checker::binding_env::BindingEnv {
    let mut contexts = mizar_checker::binding_env::BindingContextTable::new();
    for (id, context) in binding_env.contexts().iter() {
        let inserted = contexts.insert(mizar_checker::binding_env::BindingContextDraft {
            owner: context.owner.clone(),
            parent: context.parent,
            layer: context.layer,
            lexical_scope: context.lexical_scope.clone(),
            bindings: context.bindings.clone(),
            visible_bindings: context.visible_bindings.clone(),
            recovery: if id.index() == target {
                mizar_checker::binding_env::BindingContextRecovery::Degraded
            } else {
                context.recovery
            },
        });
        assert_eq!(inserted, id);
    }
    mizar_checker::binding_env::BindingEnv::try_new(mizar_checker::binding_env::BindingEnvParts {
        source_id: binding_env.source_id(),
        module_id: binding_env.module_id().clone(),
        contexts,
        bindings: binding_env.bindings().clone(),
        diagnostics: binding_env.diagnostics().clone(),
    })
    .expect("coherent B5A context-recovery mutation")
}

fn task258b5a_binding_env_with_binding_recovery(
    binding_env: &mizar_checker::binding_env::BindingEnv,
) -> mizar_checker::binding_env::BindingEnv {
    let mut bindings = mizar_checker::binding_env::BindingTable::new();
    for (id, binding) in binding_env.bindings().iter() {
        let inserted = bindings.insert(mizar_checker::binding_env::BindingDraft {
            spelling: binding.spelling.clone(),
            kind: binding.kind,
            identity: binding.identity.clone(),
            owner_context: binding.owner_context,
            declaration_range: binding.declaration_range,
            visible_after_ordinal: binding.visible_after_ordinal,
            type_site: binding.type_site.clone(),
            status: binding.status,
            captured: binding.captured.clone(),
            diagnostics: binding.diagnostics.clone(),
            recovery: mizar_checker::binding_env::BindingRecoveryState::Degraded,
        });
        assert_eq!(inserted, id);
    }
    mizar_checker::binding_env::BindingEnv::try_new(mizar_checker::binding_env::BindingEnvParts {
        source_id: binding_env.source_id(),
        module_id: binding_env.module_id().clone(),
        contexts: binding_env.contexts().clone(),
        bindings,
        diagnostics: binding_env.diagnostics().clone(),
    })
    .expect("coherent B5A binding-recovery mutation")
}

#[derive(Debug, Clone, Copy)]
enum Task258B5ARawResolverMutation {
    OwnerOriginSource,
    OwnerOriginModule,
    OwnerOriginRange,
    OwnerOriginPath,
    OwnerKind,
    OwnerVisibility,
    OwnerExport,
    OwnerSpelling,
    ContributionModule,
    ContributionKind,
    ContributionSource,
    ContributionAnchor,
    ContributionSymbolEffects,
    ContributionDefinitionEffects,
}

fn task258b5a_mutate_raw_resolver(
    symbols: SymbolEnv,
    mutation: Task258B5ARawResolverMutation,
) -> SymbolEnv {
    let mapped = match mutation {
        Task258B5ARawResolverMutation::OwnerOriginSource => {
            Task258B4CRawResolverMutation::OwnerOriginSource
        }
        Task258B5ARawResolverMutation::OwnerOriginModule => {
            Task258B4CRawResolverMutation::OwnerOriginModule
        }
        Task258B5ARawResolverMutation::OwnerOriginRange => {
            Task258B4CRawResolverMutation::OwnerOriginRange
        }
        Task258B5ARawResolverMutation::OwnerOriginPath => {
            Task258B4CRawResolverMutation::OwnerOriginPath
        }
        Task258B5ARawResolverMutation::OwnerKind => Task258B4CRawResolverMutation::OwnerKind,
        Task258B5ARawResolverMutation::OwnerVisibility => {
            Task258B4CRawResolverMutation::OwnerVisibility
        }
        Task258B5ARawResolverMutation::OwnerExport => Task258B4CRawResolverMutation::OwnerExport,
        Task258B5ARawResolverMutation::OwnerSpelling => {
            Task258B4CRawResolverMutation::OwnerSpelling
        }
        Task258B5ARawResolverMutation::ContributionModule => {
            Task258B4CRawResolverMutation::ContributionModule
        }
        Task258B5ARawResolverMutation::ContributionKind => {
            Task258B4CRawResolverMutation::ContributionKind
        }
        Task258B5ARawResolverMutation::ContributionSource => {
            Task258B4CRawResolverMutation::ContributionSource
        }
        Task258B5ARawResolverMutation::ContributionAnchor => {
            Task258B4CRawResolverMutation::ContributionAnchor
        }
        Task258B5ARawResolverMutation::ContributionSymbolEffects => {
            Task258B4CRawResolverMutation::ContributionSymbolEffects
        }
        Task258B5ARawResolverMutation::ContributionDefinitionEffects => {
            Task258B4CRawResolverMutation::ContributionDefinitionEffects
        }
    };
    task258b4c_mutate_raw_resolver(symbols, mapped)
}



const SOURCE_STATEMENT_B5B_LOWER_TEXT: &str = concat!(
    "import parser.type_fixtures;\n",
    "reserve x for set;\n",
    "theorem FormulaStatementImportedPublicTheoremCitationSmoke: x = x proof\n",
    "  thus x = x by Ref;\n",
    "end;\n",
);

fn task258b5b_lower_resolver_profile(symbols: &mizar_resolve::env::SymbolEnv) -> [usize; 5] {
    [
        symbols.symbols().len(),
        symbols.labels().len(),
        symbols.definitions().len(),
        symbols.contributions().len(),
        symbols.imports().len(),
    ]
}

fn task258b5b_imported_label_contract_is_exact(
    ast: &mizar_syntax::SurfaceAst,
    module: &mizar_resolve::resolved_ast::ModuleId,
    symbols: &mizar_resolve::env::SymbolEnv,
) -> bool {
    if task258b5b_lower_resolver_profile(symbols) != [8, 1, 1, 3, 1]
        || !symbols.module_summaries().is_empty()
    {
        return false;
    }
    let mut labels = symbols.labels().iter();
    let Some(label) = labels.next() else {
        return false;
    };
    if labels.next().is_some() {
        return false;
    }
    let imported_module = mizar_resolve::resolved_ast::ModuleId::new(
        module.package().clone(),
        mizar_session::ModulePath::new("parser.type_fixtures"),
    );
    let expected_range = mizar_session::SourceRange {
        source_id: ast.source_id,
        start: 7,
        end: 27,
    };
    let expected_origin_path = "summary:parser.type_fixtures::Ref:label:Ref";
    if label.origin_path().as_str() != expected_origin_path
        || label.kind() != mizar_resolve::resolved_ast::LabelKind::Theorem
        || label.visibility() != mizar_resolve::env::Visibility::Public
        || label.export_status() != mizar_resolve::env::ExportStatus::Exported
        || label.namespace().as_str() != module.path().as_str()
        || label.primary_spelling() != "Ref"
        || label.contribution().index() != 2
        || label.recovery() != mizar_resolve::resolved_ast::RecoveryState::Normal
        || label.origin().source_id() != ast.source_id
        || label.origin().module_id() != &imported_module
        || label.origin().anchor() != &mizar_session::SourceAnchor::Range(expected_range)
        || label.origin().structural_path() != [1, 0]
        || label.origin().import_edge().is_some()
        || label.origin().is_recovered()
    {
        return false;
    }
    let Some(contribution) = symbols.contributions().get(label.contribution()) else {
        return false;
    };
    contribution.module() == &imported_module
        && matches!(
            contribution.kind(),
            mizar_resolve::env::ContributionKind::ImportedSource { source_id }
                if *source_id == ast.source_id
        )
        && contribution.anchor() == &mizar_session::SourceAnchor::Range(expected_range)
        && contribution.effects().labels().len() == 1
        && contribution.effects().labels()[0].as_str() == expected_origin_path
}

#[derive(Debug, Clone, Copy)]
enum Task258B5BImportedLabelMutation {
    Missing,
    Duplicate,
    Private,
    LocalOnly,
    ReExported,
    WrongKind,
    WrongSpelling,
    WrongModule,
    WrongNamespace,
    WrongContribution,
    Recovered,
    WrongOriginPath,
    WrongSource,
    WrongRange,
    WrongStructuralPath,
    MissingEffect,
    ExtraEffect,
    WrongEffect,
}

fn task258b5b_mutate_imported_label(
    symbols: &mizar_resolve::env::SymbolEnv,
    current_module: &mizar_resolve::resolved_ast::ModuleId,
    foreign_source: mizar_session::SourceId,
    mutation: Task258B5BImportedLabelMutation,
) -> mizar_resolve::env::SymbolEnv {
    let label = symbols
        .labels()
        .iter()
        .next()
        .expect("Task258B5B exact imported label")
        .clone();
    let origin_path = if matches!(mutation, Task258B5BImportedLabelMutation::WrongOriginPath) {
        mizar_resolve::resolved_ast::LabelOriginPath::new(
            "summary:parser.type_fixtures::Other:label:Other",
        )
    } else {
        label.origin_path().clone()
    };
    let kind = if matches!(mutation, Task258B5BImportedLabelMutation::WrongKind) {
        mizar_resolve::resolved_ast::LabelKind::ProofStep
    } else {
        label.kind()
    };
    let namespace = if matches!(mutation, Task258B5BImportedLabelMutation::WrongNamespace) {
        mizar_resolve::env::NamespacePath::new("parser.type_fixtures")
    } else {
        label.namespace().clone()
    };
    let spelling = if matches!(mutation, Task258B5BImportedLabelMutation::WrongSpelling) {
        "Other"
    } else {
        label.primary_spelling()
    };
    let contribution = if matches!(mutation, Task258B5BImportedLabelMutation::WrongContribution) {
        symbols
            .contributions()
            .iter()
            .find(|record| record.id() != label.contribution())
            .expect("Task258B5B foreign contribution")
            .id()
    } else {
        label.contribution()
    };
    let origin_source = if matches!(mutation, Task258B5BImportedLabelMutation::WrongSource) {
        foreign_source
    } else {
        label.origin().source_id()
    };
    let origin_module = if matches!(mutation, Task258B5BImportedLabelMutation::WrongModule) {
        current_module.clone()
    } else {
        label.origin().module_id().clone()
    };
    let origin_anchor = if matches!(mutation, Task258B5BImportedLabelMutation::WrongRange) {
        mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: label.origin().source_id(),
            start: 8,
            end: 27,
        })
    } else {
        label.origin().anchor().clone()
    };
    let structural_path = if matches!(
        mutation,
        Task258B5BImportedLabelMutation::WrongStructuralPath
    ) {
        vec![1, 1]
    } else {
        label.origin().structural_path().to_vec()
    };
    let mut origin = mizar_resolve::resolved_ast::SemanticOrigin::new(
        origin_source,
        origin_module,
        origin_anchor,
        structural_path,
    );
    if matches!(mutation, Task258B5BImportedLabelMutation::Recovered) {
        origin = origin.recovered();
    }
    let visibility = if matches!(mutation, Task258B5BImportedLabelMutation::Private) {
        mizar_resolve::env::Visibility::Private
    } else {
        label.visibility()
    };
    let export_status = match mutation {
        Task258B5BImportedLabelMutation::LocalOnly => mizar_resolve::env::ExportStatus::LocalOnly,
        Task258B5BImportedLabelMutation::ReExported => mizar_resolve::env::ExportStatus::ReExported,
        _ => label.export_status(),
    };
    let mut labels = mizar_resolve::env::LabelIndex::new();
    if !matches!(mutation, Task258B5BImportedLabelMutation::Missing) {
        labels.insert(
            mizar_resolve::env::LabelEntry::new(
                origin_path,
                kind,
                namespace,
                spelling,
                origin,
                contribution,
            )
            .with_visibility(visibility)
            .with_export_status(export_status),
        );
    }
    if matches!(mutation, Task258B5BImportedLabelMutation::Duplicate) {
        labels.insert(
            mizar_resolve::env::LabelEntry::new(
                mizar_resolve::resolved_ast::LabelOriginPath::new(
                    "summary:parser.type_fixtures::Ref:label:Ref:duplicate",
                ),
                label.kind(),
                label.namespace().clone(),
                label.primary_spelling(),
                label.origin().clone(),
                label.contribution(),
            )
            .with_visibility(label.visibility())
            .with_export_status(label.export_status()),
        );
    }
    let contributions = match mutation {
        Task258B5BImportedLabelMutation::MissingEffect => {
            task258b3m2b2b3a_rebuild_contributions_with_label_effects(
                symbols,
                label.contribution(),
                &[],
            )
        }
        Task258B5BImportedLabelMutation::ExtraEffect => {
            task258b3m2b2b3a_rebuild_contributions_with_label_effects(
                symbols,
                label.contribution(),
                &[
                    label.origin_path().clone(),
                    mizar_resolve::resolved_ast::LabelOriginPath::new(
                        "summary:parser.type_fixtures::Other:label:Other",
                    ),
                ],
            )
        }
        Task258B5BImportedLabelMutation::WrongEffect => {
            task258b3m2b2b3a_rebuild_contributions_with_label_effects(
                symbols,
                label.contribution(),
                &[mizar_resolve::resolved_ast::LabelOriginPath::new(
                    "summary:parser.type_fixtures::Other:label:Other",
                )],
            )
        }
        _ => symbols.contributions().clone(),
    };
    let mut indexes = super::import_fixtures::clone_symbol_env_indexes(symbols);
    indexes.labels = labels;
    indexes.contributions = contributions;
    mizar_resolve::env::SymbolEnv::new(symbols.module_id().clone(), indexes)
}

fn task258b5b_exact_env(
    source_id_seed: usize,
) -> (
    mizar_syntax::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
    SymbolEnv,
    SymbolEnv,
) {
    let (ast, module, _, raw) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B5B_TEXT, source_id_seed);
    let enriched = super::import_fixtures::
        augment_type_elaboration_import_summaries_with_imported_public_theorem_label(
            &ast,
            &module,
            raw.clone(),
        );
    (ast, module, raw, enriched)
}

fn task258b5b_expected_reference_debug(
    module: &mizar_resolve::resolved_ast::ModuleId,
    statement_fingerprint: &str,
) -> String {
    format!(
        "source-statement-reference-debug-v1\nmodule: {}::{}\nstatement-fingerprint: {:?}\nresolver-ast root=56 nodes=57 name_refs=0 label_refs=1 imports=1 exports=0 label_node=absent reference_node=48 reference_state=resolved reference_key=label#0\nresolver-projection source=imported origin=summary:parser.type_fixtures::Ref:label:Ref module=parser.type_fixtures namespace={} range=7..27 contribution=2 path=[1,0] kind=theorem visibility=public export=exported spelling=\"Ref\"\nresolver-reference node=48 range=136..139 source_ordinal=1 scope=[0] expectation=proof-or-theorem spelling=\"Ref\"\nresolver-result index=1 references=1 ids=[0] diagnostics=0\ncitation#0 statement=1 context=1 target=imported label_ref=0 scope=[0] range=136..139 ordinal=0 kind=simple-imported recovery=normal\n",
        module.package().as_str(),
        module.path().as_str(),
        statement_fingerprint,
        module.path().as_str(),
    )
}

#[derive(Debug, Clone, Copy)]
enum Task258B5BImportMutation {
    Owner,
    Range,
    Spelling,
    Alias,
    Resolution,
    OriginSource,
    OriginModule,
    OriginAnchor,
    OriginPath,
    ImportEdge,
    Recovered,
}

const TASK258B5B_IMPORT_MUTATIONS: &[Task258B5BImportMutation] = &[
    Task258B5BImportMutation::Owner,
    Task258B5BImportMutation::Range,
    Task258B5BImportMutation::Spelling,
    Task258B5BImportMutation::Alias,
    Task258B5BImportMutation::Resolution,
    Task258B5BImportMutation::OriginSource,
    Task258B5BImportMutation::OriginModule,
    Task258B5BImportMutation::OriginAnchor,
    Task258B5BImportMutation::OriginPath,
    Task258B5BImportMutation::ImportEdge,
    Task258B5BImportMutation::Recovered,
];

fn task258b5b_resolver_ast_with_import_mutation(
    resolver_ast: &mizar_resolve::resolved_ast::ResolvedAst,
    module: &mizar_resolve::resolved_ast::ModuleId,
    mutation: Task258B5BImportMutation,
) -> Result<mizar_resolve::resolved_ast::ResolvedAst, mizar_resolve::resolved_ast::ResolvedAstError>
{
    let (import_id, original) = resolver_ast.imports().imports().next().expect("B5B import");
    let mut owner = original.owner();
    let mut import_range = original.range();
    let mut spelling = original.source_spelling().to_owned();
    let mut alias = original.alias().map(str::to_owned);
    let mut resolution = original.resolution().clone();
    let mut origin_source = original.origin().source_id();
    let mut origin_module = original.origin().module_id().clone();
    let mut anchor = original.origin().anchor().clone();
    let mut path = original.origin().structural_path().to_vec();
    let mut import_edge = original.origin().import_edge();
    let mut recovered = original.origin().is_recovered();
    match mutation {
        Task258B5BImportMutation::Owner => {
            owner = resolver_ast
                .nodes()
                .iter()
                .find_map(|(id, _)| (id.index() == 30).then_some(id))
                .expect("B5B import-item node")
        }
        Task258B5BImportMutation::Range => import_range.start -= 1,
        Task258B5BImportMutation::Spelling => spelling.push('!'),
        Task258B5BImportMutation::Alias => alias = Some("ImportedAlias".to_owned()),
        Task258B5BImportMutation::Resolution => {
            resolution = mizar_resolve::resolved_ast::ImportResolution::Resolved(
                mizar_resolve::resolved_ast::ModuleId::new(
                    module.package().clone(),
                    mizar_session::ModulePath::new("parser.wrong_fixture"),
                ),
            )
        }
        Task258B5BImportMutation::OriginSource => {
            origin_source = task258b3m2b2b3a_foreign_source_id(origin_source);
            anchor = mizar_session::SourceAnchor::Range(range(origin_source, 7, 27));
        }
        Task258B5BImportMutation::OriginModule => {
            origin_module = mizar_resolve::resolved_ast::ModuleId::new(
                module.package().clone(),
                mizar_session::ModulePath::new("statement.b5b.import-wrong"),
            )
        }
        Task258B5BImportMutation::OriginAnchor => {
            anchor = mizar_session::SourceAnchor::Range(range(origin_source, 6, 27))
        }
        Task258B5BImportMutation::OriginPath => path = vec![99],
        Task258B5BImportMutation::ImportEdge => import_edge = Some(import_id),
        Task258B5BImportMutation::Recovered => recovered = true,
    }
    let mut origin = mizar_resolve::resolved_ast::SemanticOrigin::new(
        origin_source,
        origin_module,
        anchor,
        path,
    );
    if let Some(edge) = import_edge {
        origin = origin.with_import_edge(edge);
    }
    if recovered {
        origin = origin.recovered();
    }
    let mut imports = mizar_resolve::resolved_ast::ResolvedImports::new();
    imports.push_import(mizar_resolve::resolved_ast::ResolvedImport::new(
        owner,
        import_range,
        spelling,
        alias,
        resolution,
        origin,
    ));
    mizar_resolve::resolved_ast::ResolvedAst::try_new(
        resolver_ast.source_id(),
        resolver_ast.module_id().clone(),
        resolver_ast.nodes().clone(),
        resolver_ast.name_refs().clone(),
        resolver_ast.label_refs().clone(),
        imports,
    )
}

#[derive(Debug, Clone, Copy)]
enum Task258B5BResolverNodeMutation {
    Resolution,
    ReferenceKey,
    Linkage,
}

const TASK258B5B_RESOLVER_NODE_MUTATIONS: &[Task258B5BResolverNodeMutation] = &[
    Task258B5BResolverNodeMutation::Resolution,
    Task258B5BResolverNodeMutation::ReferenceKey,
    Task258B5BResolverNodeMutation::Linkage,
];

fn task258b5b_resolver_ast_with_node_mutation(
    resolver_ast: &mizar_resolve::resolved_ast::ResolvedAst,
    mutation: Task258B5BResolverNodeMutation,
) -> Result<mizar_resolve::resolved_ast::ResolvedAst, mizar_resolve::resolved_ast::ResolvedAstError>
{
    let reference_key = resolver_ast
        .nodes()
        .iter()
        .find_map(|(id, node)| (id.index() == 48).then_some(node.reference_key()))
        .flatten()
        .expect("B5B node 48 label key");
    let import_id = resolver_ast
        .imports()
        .imports()
        .next()
        .expect("B5B import")
        .0;
    let mut builder = mizar_resolve::resolved_ast::ResolvedArenaBuilder::new();
    let mut ids = Vec::with_capacity(resolver_ast.nodes().len());
    for (id, node) in resolver_ast.nodes().iter() {
        let mut resolution = node.resolution();
        let mut key = node.reference_key();
        if id.index() == 48 {
            match mutation {
                Task258B5BResolverNodeMutation::Resolution => {
                    resolution = mizar_resolve::resolved_ast::NodeResolutionState::Deferred;
                }
                Task258B5BResolverNodeMutation::ReferenceKey => {
                    key = Some(mizar_resolve::resolved_ast::NodeReferenceKey::Import(
                        import_id,
                    ));
                }
                Task258B5BResolverNodeMutation::Linkage => {
                    resolution = mizar_resolve::resolved_ast::NodeResolutionState::NotApplicable;
                    key = None;
                }
            }
        } else if id.index() == 47 && matches!(mutation, Task258B5BResolverNodeMutation::Linkage) {
            resolution = mizar_resolve::resolved_ast::NodeResolutionState::Resolved;
            key = Some(reference_key);
        }
        let mut rebuilt = mizar_resolve::resolved_ast::ResolvedNode::new(
            node.kind().clone(),
            node.children().to_vec(),
            node.origin().clone(),
        )
        .with_recovery(node.recovery())
        .with_resolution(resolution);
        if let Some(key) = key {
            rebuilt = rebuilt.with_reference_key(key);
        }
        let inserted = builder.push(rebuilt).expect("coherent B5B resolver node");
        assert_eq!(inserted.index(), id.index());
        ids.push(inserted);
    }
    let arena = builder
        .finish(ids[resolver_ast.nodes().root().index()])
        .expect("coherent B5B resolver arena mutation");
    mizar_resolve::resolved_ast::ResolvedAst::try_new(
        resolver_ast.source_id(),
        resolver_ast.module_id().clone(),
        arena,
        resolver_ast.name_refs().clone(),
        resolver_ast.label_refs().clone(),
        resolver_ast.imports().clone(),
    )
}

#[derive(Debug, Clone, Copy)]
enum Task258B5BProjectionMutation {
    OriginPath,
    Module,
    Namespace,
    Spelling,
    Kind,
    Visibility,
    Export,
    Range,
    Contribution,
    OriginSource,
    OriginModule,
    OriginAnchor,
    OriginPathShape,
    ImportEdge,
    Recovered,
}

const TASK258B5B_PROJECTION_MUTATIONS: &[Task258B5BProjectionMutation] = &[
    Task258B5BProjectionMutation::OriginPath,
    Task258B5BProjectionMutation::Module,
    Task258B5BProjectionMutation::Namespace,
    Task258B5BProjectionMutation::Spelling,
    Task258B5BProjectionMutation::Kind,
    Task258B5BProjectionMutation::Visibility,
    Task258B5BProjectionMutation::Export,
    Task258B5BProjectionMutation::Range,
    Task258B5BProjectionMutation::Contribution,
    Task258B5BProjectionMutation::OriginSource,
    Task258B5BProjectionMutation::OriginModule,
    Task258B5BProjectionMutation::OriginAnchor,
    Task258B5BProjectionMutation::OriginPathShape,
    Task258B5BProjectionMutation::ImportEdge,
    Task258B5BProjectionMutation::Recovered,
];

fn task258b5b_projection_with_mutation(
    projection: &mizar_resolve::labels::LabelProjection,
    current_module: &mizar_resolve::resolved_ast::ModuleId,
    wrong_contribution: mizar_resolve::env::SourceContributionId,
    import_id: mizar_resolve::resolved_ast::ResolvedImportId,
    mutation: Task258B5BProjectionMutation,
) -> mizar_resolve::labels::LabelProjection {
    let mut origin_path = projection.origin_path().clone();
    let mut module = projection.module().clone();
    let mut namespace = projection.namespace().clone();
    let mut spelling = projection.primary_spelling().to_owned();
    let mut kind = projection.kind();
    let mut visibility = projection.visibility();
    let mut export = projection.export_status();
    let mut declaration_range = projection.declaration_range();
    let mut contribution = projection.contribution();
    let mut origin_source = projection.origin().source_id();
    let mut origin_module = projection.origin().module_id().clone();
    let mut anchor = projection.origin().anchor().clone();
    let mut path = projection.origin().structural_path().to_vec();
    let mut import_edge = projection.origin().import_edge();
    let mut recovered = projection.origin().is_recovered();
    match mutation {
        Task258B5BProjectionMutation::OriginPath => {
            origin_path = mizar_resolve::resolved_ast::LabelOriginPath::new("summary:wrong::Ref")
        }
        Task258B5BProjectionMutation::Module => {
            module = mizar_resolve::resolved_ast::ModuleId::new(
                current_module.package().clone(),
                mizar_session::ModulePath::new("parser.wrong_fixture"),
            )
        }
        Task258B5BProjectionMutation::Namespace => {
            namespace = mizar_resolve::env::NamespacePath::new("statement.b5b.wrong")
        }
        Task258B5BProjectionMutation::Spelling => spelling.push('!'),
        Task258B5BProjectionMutation::Kind => {
            kind = mizar_resolve::resolved_ast::LabelKind::ProofStep
        }
        Task258B5BProjectionMutation::Visibility => {
            visibility = mizar_resolve::env::Visibility::Private
        }
        Task258B5BProjectionMutation::Export => {
            export = mizar_resolve::env::ExportStatus::ReExported
        }
        Task258B5BProjectionMutation::Range => declaration_range.start -= 1,
        Task258B5BProjectionMutation::Contribution => contribution = wrong_contribution,
        Task258B5BProjectionMutation::OriginSource => {
            origin_source = task258b3m2b2b3a_foreign_source_id(origin_source);
            anchor = mizar_session::SourceAnchor::Range(range(origin_source, 7, 27));
        }
        Task258B5BProjectionMutation::OriginModule => origin_module = current_module.clone(),
        Task258B5BProjectionMutation::OriginAnchor => {
            anchor = mizar_session::SourceAnchor::Range(range(origin_source, 6, 27))
        }
        Task258B5BProjectionMutation::OriginPathShape => path = vec![1, 1],
        Task258B5BProjectionMutation::ImportEdge => import_edge = Some(import_id),
        Task258B5BProjectionMutation::Recovered => recovered = true,
    }
    let mut origin = mizar_resolve::resolved_ast::SemanticOrigin::new(
        origin_source,
        origin_module,
        anchor,
        path,
    );
    if let Some(edge) = import_edge {
        origin = origin.with_import_edge(edge);
    }
    if recovered {
        origin = origin.recovered();
    }
    mizar_resolve::labels::LabelProjection::imported(mizar_resolve::labels::LabelProjectionData {
        origin_path,
        module,
        namespace,
        primary_spelling: spelling,
        kind,
        declaration_range,
        origin,
        contribution,
    })
    .with_visibility(visibility)
    .with_export_status(export)
}

#[derive(Debug, Clone, Copy)]
enum Task258B5BReferenceMutation {
    Node,
    Range,
    Spelling,
    SourceOrdinal,
    Scope,
    Expectation,
    OriginSource,
    OriginModule,
    OriginAnchor,
    OriginPath,
    ImportEdge,
    Recovered,
}

const TASK258B5B_REFERENCE_MUTATIONS: &[Task258B5BReferenceMutation] = &[
    Task258B5BReferenceMutation::Node,
    Task258B5BReferenceMutation::Range,
    Task258B5BReferenceMutation::Spelling,
    Task258B5BReferenceMutation::SourceOrdinal,
    Task258B5BReferenceMutation::Scope,
    Task258B5BReferenceMutation::Expectation,
    Task258B5BReferenceMutation::OriginSource,
    Task258B5BReferenceMutation::OriginModule,
    Task258B5BReferenceMutation::OriginAnchor,
    Task258B5BReferenceMutation::OriginPath,
    Task258B5BReferenceMutation::ImportEdge,
    Task258B5BReferenceMutation::Recovered,
];

fn task258b5b_reference_with_mutation(
    reference: &mizar_resolve::labels::LabelReferenceCandidate,
    module: &mizar_resolve::resolved_ast::ModuleId,
    wrong_node: mizar_resolve::resolved_ast::ResolvedNodeId,
    import_id: mizar_resolve::resolved_ast::ResolvedImportId,
    mutation: Task258B5BReferenceMutation,
) -> mizar_resolve::labels::LabelReferenceCandidate {
    let mut node = reference.site().node();
    let mut reference_range = reference.site().range();
    let mut spelling = reference.site().spelling().to_owned();
    let mut source_ordinal = reference.ordinal();
    let mut scope = Some(mizar_resolve::labels::LabelScopePath::new(vec![0]));
    let mut expectation = reference.expectation();
    let mut origin_source = reference.origin().source_id();
    let mut origin_module = reference.origin().module_id().clone();
    let mut anchor = reference.origin().anchor().clone();
    let mut path = reference.origin().structural_path().to_vec();
    let mut import_edge = reference.origin().import_edge();
    let mut recovered = reference.origin().is_recovered();
    match mutation {
        Task258B5BReferenceMutation::Node => node = wrong_node,
        Task258B5BReferenceMutation::Range => reference_range.start -= 1,
        Task258B5BReferenceMutation::Spelling => spelling.push('!'),
        Task258B5BReferenceMutation::SourceOrdinal => source_ordinal = 0,
        Task258B5BReferenceMutation::Scope => {
            scope = Some(mizar_resolve::labels::LabelScopePath::new(vec![0, 1]))
        }
        Task258B5BReferenceMutation::Expectation => {
            expectation = mizar_resolve::resolved_ast::LabelExpectation::Theorem
        }
        Task258B5BReferenceMutation::OriginSource => {
            origin_source = task258b3m2b2b3a_foreign_source_id(origin_source);
            anchor = mizar_session::SourceAnchor::Range(range(origin_source, 136, 139));
        }
        Task258B5BReferenceMutation::OriginModule => {
            origin_module = mizar_resolve::resolved_ast::ModuleId::new(
                module.package().clone(),
                mizar_session::ModulePath::new("statement.b5b.reference-wrong"),
            )
        }
        Task258B5BReferenceMutation::OriginAnchor => {
            anchor = mizar_session::SourceAnchor::Range(range(origin_source, 135, 139))
        }
        Task258B5BReferenceMutation::OriginPath => path = vec![47],
        Task258B5BReferenceMutation::ImportEdge => import_edge = Some(import_id),
        Task258B5BReferenceMutation::Recovered => recovered = true,
    }
    let mut origin = mizar_resolve::resolved_ast::SemanticOrigin::new(
        origin_source,
        origin_module,
        anchor,
        path,
    );
    if let Some(edge) = import_edge {
        origin = origin.with_import_edge(edge);
    }
    if recovered {
        origin = origin.recovered();
    }
    mizar_resolve::labels::LabelReferenceCandidate::unqualified_citation(
        mizar_resolve::resolved_ast::ReferenceSite::new(node, reference_range, spelling),
        origin,
        source_ordinal,
        scope,
    )
    .with_expectation(expectation)
}
