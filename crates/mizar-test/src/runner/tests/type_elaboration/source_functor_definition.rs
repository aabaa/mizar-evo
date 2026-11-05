use super::type_elaboration::{
    SOURCE_FUNCTOR_DEFINITION_TEXT, SourceFunctorDefinitionRouteMutation,
    source_functor_definition_output, source_functor_definition_output_with_mutation,
    step5c5_functor_duplicate_detail_keys, step5c5_functor_semantics_detail_keys,
};

const TASK260_CASE: &str = "pass_type_elaboration_functor_definition_payload_001";
const TASK260_FUNCTOR_MIXED_CASE: &str =
    "fail_type_elaboration_predicate_functor_definition_gap_001";
const TASK260_SPEC_REF: &str = "spec.en.checker.type_elaboration.source_functor_definition_payload";

#[test]
fn step5c5_functor_semantics_reject_malformed_or_unauthenticated_payloads() {
    let equals = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/miz/fail/functors/fail_type_elaboration_func_equals_result_type_mismatch_001.miz"
    ));
    let (ast, module, shells, symbols) = task253_ast_from_source_text(equals, 505_010);
    let mismatch = Some(vec!["functors.equals.result_type_mismatch".to_owned()]);
    assert_eq!(
        step5c5_functor_semantics_detail_keys(&ast, &module, &shells, &symbols),
        mismatch
    );
    let changed_kind = rebuild_surface_ast_replacing_kind(
        &ast,
        SurfaceNodeKind::TypeExpression,
        SurfaceNodeKind::TermExpression,
    );
    assert_ne!(
        step5c5_functor_semantics_detail_keys(&changed_kind, &module, &shells, &symbols),
        mismatch
    );
    let recovered = rebuild_surface_ast_recovering_first_token_in_kind(
        &ast,
        SurfaceNodeKind::FunctorDefinition,
    );
    assert_ne!(
        step5c5_functor_semantics_detail_keys(&recovered, &module, &shells, &symbols),
        mismatch
    );

    let bracket = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/miz/pass/functors/pass_type_elaboration_func_builtin_bracket_pair_001.miz"
    ));
    let (bracket_ast, bracket_module, bracket_shells, bracket_symbols) =
        task253_ast_from_source_text(bracket, 505_011);
    assert_eq!(
        step5c5_functor_semantics_detail_keys(
            &bracket_ast,
            &bracket_module,
            &bracket_shells,
            &bracket_symbols
        ),
        Some(Vec::new())
    );
    let changed_bracket = rebuild_surface_ast_replacing_kind(
        &bracket_ast,
        SurfaceNodeKind::ApplicationTerm,
        SurfaceNodeKind::TermList,
    );
    assert_ne!(
        step5c5_functor_semantics_detail_keys(
            &changed_bracket,
            &bracket_module,
            &bracket_shells,
            &bracket_symbols
        ),
        Some(Vec::new())
    );
    let recovered_theorem = rebuild_surface_ast_recovering_first_token_in_kind(
        &bracket_ast,
        SurfaceNodeKind::TheoremItem,
    );
    assert_ne!(
        step5c5_functor_semantics_detail_keys(
            &recovered_theorem,
            &bracket_module,
            &bracket_shells,
            &bracket_symbols
        ),
        Some(Vec::new())
    );
    let unbound = bracket.replacen("[x, y]", "[z, y]", 1);
    let (unbound_ast, unbound_module, unbound_shells, unbound_symbols) =
        task253_ast_from_source_text(&unbound, 505_012);
    assert_ne!(
        step5c5_functor_semantics_detail_keys(
            &unbound_ast,
            &unbound_module,
            &unbound_shells,
            &unbound_symbols
        ),
        Some(Vec::new())
    );
    assert_ne!(
        step5c5_functor_semantics_detail_keys(&ast, &module, &bracket_shells, &bracket_symbols),
        mismatch
    );
}

#[test]
fn step5c5_functor_missing_conditions_arity_and_duplicate_are_semantic() {
    let means = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/miz/fail/functors/fail_type_elaboration_func_means_missing_correctness_001.miz"
    ));
    let (ast, module, shells, symbols) = task253_ast_from_source_text(means, 505_020);
    let missing = Some(vec!["functors.means.missing_correctness".to_owned()]);
    assert_eq!(
        step5c5_functor_semantics_detail_keys(&ast, &module, &shells, &symbols),
        missing
    );
    let complete = means.replacen("end;", "  existence;\n  uniqueness;\nend;", 1);
    let (ast, module, shells, symbols) = task253_ast_from_source_text(&complete, 505_021);
    assert_ne!(
        step5c5_functor_semantics_detail_keys(&ast, &module, &shells, &symbols),
        missing
    );

    let property = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/miz/fail/functors/fail_type_elaboration_func_property_arity_mismatch_001.miz"
    ));
    let (ast, module, shells, symbols) = task253_ast_from_source_text(property, 505_022);
    let mismatch = Some(vec!["functors.property.arity_mismatch".to_owned()]);
    assert_eq!(
        step5c5_functor_semantics_detail_keys(&ast, &module, &shells, &symbols),
        mismatch
    );
    let widened = property
        .replacen("let X be set", "let X, Y be set", 1)
        .replacen("solobox X", "solobox(X,Y)", 1);
    let (ast, module, shells, symbols) = task253_ast_from_source_text(&widened, 505_023);
    assert_ne!(
        step5c5_functor_semantics_detail_keys(&ast, &module, &shells, &symbols),
        mismatch
    );

    let duplicate = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/miz/fail/functors/fail_type_elaboration_func_duplicate_same_signature_001.miz"
    ));
    let (ast, module, shells, symbols) = task253_ast_from_source_text(duplicate, 505_024);
    let resolver_key = vec![
        "declaration_symbol.signature.same_signature_definition_conflict".to_owned(),
    ];
    assert_eq!(
        step5c5_functor_duplicate_detail_keys(
            &ast,
            &module,
            &shells,
            &symbols,
            &resolver_key
        ),
        Some(vec!["functors.definition.duplicate_same_signature".to_owned()])
    );
    assert_eq!(
        step5c5_functor_duplicate_detail_keys(&ast, &module, &shells, &symbols, &[]),
        None
    );
}

#[test]
fn step5c5_unexpected_resolver_diagnostic_stays_lower_stage() {
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
    let plan = build_test_plan(&config).expect("Step 5C.5 test plan");
    let active = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_type_elaboration_func_duplicate_same_signature_001")
        .expect("active duplicate functor");
    let (ordinal, unexpected) = plan
        .cases
        .iter()
        .enumerate()
        .find(|(_, case)| case.id.0 == "fail_resolve_same_signature_return_conflict_001")
        .expect("different resolver conflict");
    let output = run_frontend(&workspace_root, unexpected, ordinal).expect("frontend output");
    assert!(output.diagnostics.is_empty());
    let mut snapshot = None;
    assert_eq!(
        type_elaboration_detail_keys(&workspace_root, active, output, &mut snapshot),
        vec![
            "type_elaboration.lower_stage.declaration_symbol.signature.same_signature_return_conflict"
                .to_owned()
        ]
    );
}

#[derive(Debug)]
enum Task260ExpectedSurfaceKind {
    Token(SurfaceTokenKind, &'static str),
    Structural(SurfaceNodeKind),
}

struct Task260ExpectedSurfaceRow {
    kind: Task260ExpectedSurfaceKind,
    start: usize,
    end: usize,
    children: &'static [usize],
}

macro_rules! task260_token_row {
    ($kind:ident, $text:literal, $start:literal, $end:literal) => {
        Task260ExpectedSurfaceRow {
            kind: Task260ExpectedSurfaceKind::Token(SurfaceTokenKind::$kind, $text),
            start: $start,
            end: $end,
            children: &[],
        }
    };
}

macro_rules! task260_structural_row {
    ($kind:ident, $start:literal, $end:literal, [$($child:literal),* $(,)?]) => {
        Task260ExpectedSurfaceRow {
            kind: Task260ExpectedSurfaceKind::Structural(SurfaceNodeKind::$kind),
            start: $start,
            end: $end,
            children: &[$($child),*],
        }
    };
}

const TASK260_EXPECTED_SURFACE_ROWS: &[Task260ExpectedSurfaceRow] = &[
    task260_token_row!(ReservedWord, "definition", 0, 10),
    task260_token_row!(ReservedWord, "let", 13, 16),
    task260_token_row!(Identifier, "x", 17, 18),
    task260_token_row!(ReservedWord, "be", 19, 21),
    task260_token_row!(ReservedWord, "set", 22, 25),
    task260_token_row!(ReservedSymbol, ";", 25, 26),
    task260_token_row!(ReservedWord, "let", 29, 32),
    task260_token_row!(Identifier, "y", 33, 34),
    task260_token_row!(ReservedWord, "be", 35, 37),
    task260_token_row!(ReservedWord, "set", 38, 41),
    task260_token_row!(ReservedSymbol, ";", 41, 42),
    task260_token_row!(ReservedWord, "assume", 45, 51),
    task260_token_row!(Identifier, "x", 52, 53),
    task260_token_row!(ReservedSymbol, "=", 54, 55),
    task260_token_row!(Identifier, "x", 56, 57),
    task260_token_row!(ReservedSymbol, ";", 57, 58),
    task260_token_row!(ReservedWord, "func", 61, 65),
    task260_token_row!(Identifier, "Task260EqualsDef", 66, 82),
    task260_token_row!(ReservedSymbol, ":", 82, 83),
    task260_token_row!(Identifier, "task260_equals", 84, 98),
    task260_token_row!(ReservedSymbol, "(", 98, 99),
    task260_token_row!(Identifier, "x", 99, 100),
    task260_token_row!(ReservedSymbol, ")", 100, 101),
    task260_token_row!(ReservedSymbol, "->", 102, 104),
    task260_token_row!(ReservedWord, "set", 105, 108),
    task260_token_row!(ReservedWord, "equals", 109, 115),
    task260_token_row!(Identifier, "x", 116, 117),
    task260_token_row!(ReservedSymbol, ";", 117, 118),
    task260_token_row!(ReservedWord, "func", 121, 125),
    task260_token_row!(Identifier, "Task260MeansDef", 126, 141),
    task260_token_row!(ReservedSymbol, ":", 141, 142),
    task260_token_row!(Identifier, "task260_means", 143, 156),
    task260_token_row!(ReservedSymbol, "(", 156, 157),
    task260_token_row!(Identifier, "y", 157, 158),
    task260_token_row!(ReservedSymbol, ")", 158, 159),
    task260_token_row!(ReservedSymbol, "->", 160, 162),
    task260_token_row!(ReservedWord, "set", 163, 166),
    task260_token_row!(ReservedWord, "means", 167, 172),
    task260_token_row!(Identifier, "x", 173, 174),
    task260_token_row!(ReservedSymbol, "=", 175, 176),
    task260_token_row!(Identifier, "y", 177, 178),
    task260_token_row!(ReservedSymbol, ";", 178, 179),
    task260_token_row!(ReservedWord, "existence", 182, 191),
    task260_token_row!(ReservedWord, "by", 192, 194),
    task260_token_row!(ReservedWord, "computation", 195, 206),
    task260_token_row!(ReservedSymbol, "(", 206, 207),
    task260_token_row!(Identifier, "steps", 207, 212),
    task260_token_row!(ReservedSymbol, ":", 212, 213),
    task260_token_row!(Numeral, "1", 214, 215),
    task260_token_row!(ReservedSymbol, ")", 215, 216),
    task260_token_row!(ReservedSymbol, ";", 216, 217),
    task260_token_row!(ReservedWord, "uniqueness", 220, 230),
    task260_token_row!(ReservedWord, "by", 231, 233),
    task260_token_row!(ReservedWord, "computation", 234, 245),
    task260_token_row!(ReservedSymbol, "(", 245, 246),
    task260_token_row!(Identifier, "steps", 246, 251),
    task260_token_row!(ReservedSymbol, ":", 251, 252),
    task260_token_row!(Numeral, "1", 253, 254),
    task260_token_row!(ReservedSymbol, ")", 254, 255),
    task260_token_row!(ReservedSymbol, ";", 255, 256),
    task260_token_row!(ReservedWord, "end", 257, 260),
    task260_token_row!(ReservedSymbol, ";", 260, 261),
    task260_structural_row!(TypeHead, 22, 25, [4]),
    task260_structural_row!(TypeExpression, 22, 25, [62]),
    task260_structural_row!(QualifiedVariableSegment, 17, 25, [2, 3, 63]),
    task260_structural_row!(DefinitionParameter, 13, 26, [1, 64, 5]),
    task260_structural_row!(TypeHead, 38, 41, [9]),
    task260_structural_row!(TypeExpression, 38, 41, [66]),
    task260_structural_row!(QualifiedVariableSegment, 33, 41, [7, 8, 67]),
    task260_structural_row!(DefinitionParameter, 29, 42, [6, 68, 10]),
    task260_structural_row!(TermReference, 52, 53, [12]),
    task260_structural_row!(TermExpression, 52, 53, [70]),
    task260_structural_row!(TermReference, 56, 57, [14]),
    task260_structural_row!(TermExpression, 56, 57, [72]),
    task260_structural_row!(BuiltinPredicateApplication, 52, 57, [71, 13, 73]),
    task260_structural_row!(FormulaExpression, 52, 57, [74]),
    task260_structural_row!(Proposition, 52, 57, [75]),
    task260_structural_row!(AssumptionStatement, 45, 58, [11, 76, 15]),
    task260_structural_row!(FunctorPattern, 84, 101, [19, 20, 21, 22]),
    task260_structural_row!(TypeHead, 105, 108, [24]),
    task260_structural_row!(TypeExpression, 105, 108, [79]),
    task260_structural_row!(TermReference, 116, 117, [26]),
    task260_structural_row!(TermExpression, 116, 117, [81]),
    task260_structural_row!(TermDefiniens, 116, 117, [82]),
    task260_structural_row!(
        FunctorDefinition,
        61,
        118,
        [16, 17, 18, 78, 23, 80, 25, 83, 27]
    ),
    task260_structural_row!(FunctorPattern, 143, 159, [31, 32, 33, 34]),
    task260_structural_row!(TypeHead, 163, 166, [36]),
    task260_structural_row!(TypeExpression, 163, 166, [86]),
    task260_structural_row!(TermReference, 173, 174, [38]),
    task260_structural_row!(TermExpression, 173, 174, [88]),
    task260_structural_row!(TermReference, 177, 178, [40]),
    task260_structural_row!(TermExpression, 177, 178, [90]),
    task260_structural_row!(BuiltinPredicateApplication, 173, 178, [89, 39, 91]),
    task260_structural_row!(FormulaExpression, 173, 178, [92]),
    task260_structural_row!(FormulaDefiniens, 173, 178, [93]),
    task260_structural_row!(
        FunctorDefinition,
        121,
        179,
        [28, 29, 30, 85, 35, 87, 37, 94, 41]
    ),
    task260_structural_row!(ComputationOption, 207, 215, [46, 47, 48]),
    task260_structural_row!(ComputationJustification, 195, 216, [44, 45, 96, 49]),
    task260_structural_row!(JustificationClause, 192, 216, [43, 97]),
    task260_structural_row!(CorrectnessCondition, 182, 217, [42, 98, 50]),
    task260_structural_row!(ComputationOption, 246, 254, [55, 56, 57]),
    task260_structural_row!(ComputationJustification, 234, 255, [53, 54, 100, 58]),
    task260_structural_row!(JustificationClause, 231, 255, [52, 101]),
    task260_structural_row!(CorrectnessCondition, 220, 256, [51, 102, 59]),
    task260_structural_row!(
        DefinitionBlockItem,
        0,
        261,
        [0, 65, 69, 77, 84, 95, 99, 103, 60, 61]
    ),
    task260_structural_row!(ItemList, 0, 261, [104]),
    task260_structural_row!(CompilationUnit, 0, 261, [105]),
    task260_structural_row!(
        Root,
        0,
        261,
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 106
        ]
    ),
];

const fn task260_range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

#[test]
fn task260_real_source_surface_resolver_and_lower_bundle_is_exact() {
    assert_eq!(SOURCE_FUNCTOR_DEFINITION_TEXT.len(), 262);
    assert!(SOURCE_FUNCTOR_DEFINITION_TEXT.ends_with('\n'));
    assert!(!SOURCE_FUNCTOR_DEFINITION_TEXT.ends_with("\n\n"));
    assert_eq!(
        sha256_text(SOURCE_FUNCTOR_DEFINITION_TEXT),
        "9bbf50016c72faf8b86342a9a65f8d59bf7747b85b43b6c5bc3c624c7212416a"
    );

    let (ast, module, shells, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(SOURCE_FUNCTOR_DEFINITION_TEXT, 260_000);
    assert_eq!(diagnostics, 0);
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|id| id.index())),
        (108, Some(107))
    );
    assert!(ast.expression_root().is_none());
    assert_eq!(
        ast.root().and_then(|id| ast.node(id)).map(|node| (
            node.range.start,
            node.range.end,
            node.recovered
        )),
        Some((0, 261, false))
    );
    assert_eq!(TASK260_EXPECTED_SURFACE_ROWS.len(), 108);
    for (index, (node, expected)) in ast
        .nodes()
        .iter()
        .zip(TASK260_EXPECTED_SURFACE_ROWS)
        .enumerate()
    {
        assert_eq!(
            node.range,
            task260_range(ast.source_id, expected.start, expected.end),
            "surface row {index} range"
        );
        assert!(!node.recovered, "surface row {index} recovery");
        assert_eq!(
            node.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            expected.children,
            "surface row {index} ordered children"
        );
        match (&node.kind, &expected.kind) {
            (
                SurfaceNodeKind::Token(actual),
                Task260ExpectedSurfaceKind::Token(expected_kind, expected_text),
            ) => {
                assert_eq!(
                    actual.kind, *expected_kind,
                    "surface row {index} token kind"
                );
                assert_eq!(
                    actual.text.as_ref(),
                    *expected_text,
                    "surface row {index} token text"
                );
            }
            (actual, Task260ExpectedSurfaceKind::Structural(expected_kind)) => {
                assert_eq!(actual, expected_kind, "surface row {index} structural kind");
            }
            (actual, expected) => {
                panic!(
                    "surface row {index} kind mismatch: {actual:?} vs literal oracle {expected:?}"
                )
            }
        }
    }

    let [block, equals, means] = shells.declarations() else {
        panic!("Task260 must expose exactly three declaration shells");
    };
    assert_eq!(
        [
            (block.id().index(), block.ordinal(), block.node_id().index()),
            (
                equals.id().index(),
                equals.ordinal(),
                equals.node_id().index(),
            ),
            (means.id().index(), means.ordinal(), means.node_id().index(),),
        ],
        [(0, 0, 104), (1, 1, 84), (2, 2, 95)]
    );
    assert_eq!(equals.parent(), Some(block.id()));
    assert_eq!(means.parent(), Some(block.id()));
    assert!(shells.exports().is_empty());
    let projections = mizar_resolve::symbols::SignatureProjectionExtractor::new(
        &ast,
        &shells,
        mizar_resolve::env::NamespacePath::new(module.path().as_str()),
    )
    .extract();
    assert_eq!(projections.len(), 2);
    assert_eq!(
        projections
            .iter()
            .map(|projection| projection.primary_spelling())
            .collect::<Vec<_>>(),
        ["task260_equals ( x )", "task260_means ( y )"]
    );
    assert_eq!(
        projections
            .iter()
            .map(|projection| projection.arity())
            .collect::<Vec<_>>(),
        [Some(1), Some(1)]
    );
    assert_eq!(
        (
            symbols.symbols().len(),
            symbols.definitions().len(),
            symbols.contributions().len(),
        ),
        (2, 2, 1)
    );
    assert_eq!(
        symbols
            .definitions()
            .iter()
            .map(|definition| definition.arity())
            .collect::<Vec<_>>(),
        [Some(1), Some(1)]
    );

    let output = source_functor_definition_output(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        SOURCE_FUNCTOR_DEFINITION_TEXT,
    )
    .expect("Task260 exact selector")
    .unwrap_or_else(|error| panic!("Task260 exact route failed: {error}"));
    let context = output.typed_ast.source_context().expect("Task248");
    let source_type = output.typed_ast.source_type().expect("Task249+249R");
    let terms = output.typed_ast.source_term().expect("Task252");
    let formulas = output.typed_ast.source_atomic_formula().expect("Task256");
    let functor = output
        .typed_ast
        .source_functor_definition()
        .expect("Task260");
    assert_eq!(
        (
            context.items().len(),
            context.declarations().len(),
            context.binding_env().bindings().len(),
            context.binding_env().contexts().len(),
            context.local_contexts().len(),
            context.context_links().len(),
            context.binding_env().diagnostics().len(),
        ),
        (1, 2, 2, 2, 2, 2, 0)
    );
    assert_eq!(
        (
            source_type.applications().len(),
            source_type.expressions().len(),
            source_type.arguments().len(),
            source_type.definition_returns().len(),
            terms.terms().len(),
            terms.references().len(),
            terms.numeric_type_requests().len(),
        ),
        (2, 4, 0, 2, 5, 5, 0)
    );
    assert_eq!(
        (
            formulas.formulas().len(),
            formulas.wrappers().len(),
            formulas.predicate_segments().len(),
            formulas.predicate_heads().len(),
            formulas.candidates().len(),
            formulas.type_sites().len(),
            formulas.attributes().len(),
            formulas.edges().len(),
            formulas.requests().len(),
        ),
        (2, 0, 0, 0, 0, 0, 0, 4, 4)
    );
    assert_eq!(
        (
            functor.definitions().len(),
            functor.parameters().len(),
            functor.guards().len(),
            functor.definientia().len(),
            functor.correctness().len(),
        ),
        (2, 2, 1, 2, 2)
    );
    assert!(!functor.definitions().is_empty());
    assert!(!functor.parameters().is_empty());
    assert!(!functor.guards().is_empty());
    assert!(!functor.definientia().is_empty());
    assert!(!functor.correctness().is_empty());
    assert_eq!(functor.source_id(), ast.source_id);
    assert_eq!(functor.module_id(), &module);
    assert_eq!(functor.source_context_fingerprint(), context.debug_text());
    assert_eq!(functor.source_type_fingerprint(), source_type.debug_text());
    assert_eq!(functor.source_term_fingerprint(), terms.debug_text());
    assert_eq!(functor.application_fingerprint(), None);
    assert_eq!(functor.structure_fingerprint(), None);
    assert_eq!(functor.set_term_fingerprint(), None);
    assert_eq!(
        functor.atomic_formula_fingerprint(),
        Some(formulas.debug_text().as_str())
    );

    let definitions = functor.definitions().iter().collect::<Vec<_>>();
    let [(equals_id, equals_row), (means_id, means_row)] = definitions.as_slice() else {
        panic!("Task260 definition rows changed");
    };
    assert_eq!((equals_id.index(), means_id.index()), (0, 1));
    assert_eq!(functor.definitions().get(*equals_id), Some(*equals_row));
    assert_eq!(functor.definitions().get(*means_id), Some(*means_row));
    let resolver_definitions = symbols.definitions().iter().collect::<Vec<_>>();
    assert_eq!(resolver_definitions.len(), 2);
    for (index, (row, site, start, end, ordinal, style, spelling, path)) in [
        (
            *equals_row,
            84,
            61,
            118,
            0,
            mizar_checker::source_functor_definition::SourceFunctorDefinitionStyle::Equals,
            "func Task260EqualsDef: task260_equals(x) -> set equals x;",
            &[4, 0, 9, 0][..],
        ),
        (
            *means_row,
            95,
            121,
            179,
            1,
            mizar_checker::source_functor_definition::SourceFunctorDefinitionStyle::Means,
            "func Task260MeansDef: task260_means(y) -> set means x = y;",
            &[4, 0, 9, 1][..],
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let resolver_definition = resolver_definitions[index];
        assert_eq!(row.id().index(), index);
        assert_eq!(row.symbol(), resolver_definition.symbol());
        assert_eq!(row.definition(), resolver_definition.id());
        assert_eq!(row.contribution(), resolver_definition.contribution());
        assert_eq!(
            row.site(),
            &mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(site)
            )
        );
        assert_eq!(row.source_range(), task260_range(ast.source_id, start, end));
        assert_eq!(row.source_ordinal(), ordinal);
        assert_eq!(row.context().index(), 1);
        assert_eq!(
            row.recovery(),
            mizar_checker::source_functor_definition::SourceFunctorDefinitionRecovery::Normal
        );
        assert_eq!(row.spelling(), spelling);
        assert_eq!(row.style(), style);
        assert_eq!(row.return_type().index(), index);
        assert_eq!(row.definiens().index(), index);
        assert_eq!(row.origin().source_id(), ast.source_id);
        assert_eq!(row.origin().module_id(), &module);
        assert_eq!(
            row.origin().anchor(),
            &mizar_session::SourceAnchor::Range(task260_range(ast.source_id, start, end))
        );
        assert_eq!(row.origin().structural_path(), path);
        assert!(row.origin().import_edge().is_none());
        assert!(!row.origin().is_recovered());
    }

    let parameters = functor.parameters().iter().collect::<Vec<_>>();
    let [(x_id, x), (y_id, y)] = parameters.as_slice() else {
        panic!("Task260 parameter rows changed");
    };
    assert_eq!((x_id.index(), y_id.index()), (0, 1));
    assert_eq!(functor.parameters().get(*x_id), Some(*x));
    assert_eq!(functor.parameters().get(*y_id), Some(*y));
    for (index, (row, site, start, end, decl_start, decl_end, spelling)) in [
        (*x, 65, 13, 26, 17, 18, "let x be set;"),
        (*y, 69, 29, 42, 33, 34, "let y be set;"),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(row.id().index(), index);
        assert_eq!(row.ordinal(), index);
        assert_eq!(row.binding().index(), index);
        assert_eq!(row.written_type().index(), index);
        assert_eq!(
            row.site(),
            &mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(site)
            )
        );
        assert_eq!(row.source_range(), task260_range(ast.source_id, start, end));
        assert_eq!(
            row.declaration_range(),
            task260_range(ast.source_id, decl_start, decl_end)
        );
        assert_eq!(row.context().index(), 1);
        assert_eq!(
            row.recovery(),
            mizar_checker::source_functor_definition::SourceFunctorDefinitionRecovery::Normal
        );
        assert_eq!(row.spelling(), spelling);
    }

    let guards = functor.guards().iter().collect::<Vec<_>>();
    let [(guard_id, guard)] = guards.as_slice() else {
        panic!("Task260 guard rows changed");
    };
    assert_eq!(guard_id.index(), 0);
    assert_eq!(functor.guards().get(*guard_id), Some(*guard));
    assert_eq!(
        (guard.id().index(), guard.ordinal(), guard.formula().index()),
        (0, 0, 0)
    );
    assert_eq!(
        guard.site(),
        &mizar_checker::typed_ast::TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(
            77
        ))
    );
    assert_eq!(guard.source_range(), task260_range(ast.source_id, 45, 58));
    assert_eq!(guard.context().index(), 1);
    assert_eq!(
        guard.recovery(),
        mizar_checker::source_functor_definition::SourceFunctorDefinitionRecovery::Normal
    );
    assert_eq!(guard.spelling(), "assume x = x;");

    let definientia = functor.definientia().iter().collect::<Vec<_>>();
    let [
        (equals_definiens_id, equals_definiens),
        (means_definiens_id, means_definiens),
    ] = definientia.as_slice()
    else {
        panic!("Task260 definiens rows changed");
    };
    assert_eq!(
        (equals_definiens_id.index(), means_definiens_id.index()),
        (0, 1)
    );
    assert_eq!(
        functor.definientia().get(*equals_definiens_id),
        Some(*equals_definiens)
    );
    assert_eq!(
        functor.definientia().get(*means_definiens_id),
        Some(*means_definiens)
    );
    assert_eq!(
        (
            equals_definiens.id().index(),
            equals_definiens.owner().index(),
            equals_definiens.ordinal(),
            equals_definiens.target(),
            equals_definiens.context().index(),
            equals_definiens.spelling(),
        ),
        (
            0,
            0,
            0,
            mizar_checker::source_functor_definition::SourceFunctorDefiniensTarget::Primary(
                mizar_checker::source_term::SourcePrimaryTermId::new(2)
            ),
            1,
            "x",
        )
    );
    assert_eq!(
        equals_definiens.site(),
        &mizar_checker::typed_ast::TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(
            83
        ))
    );
    assert_eq!(
        equals_definiens.source_range(),
        task260_range(ast.source_id, 116, 117)
    );
    assert_eq!(
        (
            means_definiens.id().index(),
            means_definiens.owner().index(),
            means_definiens.ordinal(),
            means_definiens.target(),
            means_definiens.context().index(),
            means_definiens.spelling(),
        ),
        (
            1,
            1,
            1,
            mizar_checker::source_functor_definition::SourceFunctorDefiniensTarget::AtomicFormula(
                mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(1)
            ),
            1,
            "x = y",
        )
    );
    assert_eq!(
        means_definiens.site(),
        &mizar_checker::typed_ast::TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(
            94
        ))
    );
    assert_eq!(
        means_definiens.source_range(),
        task260_range(ast.source_id, 173, 178)
    );
    assert_eq!(
        (equals_definiens.recovery(), means_definiens.recovery()),
        (
            mizar_checker::source_functor_definition::SourceFunctorDefinitionRecovery::Normal,
            mizar_checker::source_functor_definition::SourceFunctorDefinitionRecovery::Normal,
        )
    );

    let correctness = functor.correctness().iter().collect::<Vec<_>>();
    let [(existence_id, existence), (uniqueness_id, uniqueness)] = correctness.as_slice() else {
        panic!("Task260 correctness rows changed");
    };
    assert_eq!((existence_id.index(), uniqueness_id.index()), (0, 1));
    assert_eq!(functor.correctness().get(*existence_id), Some(*existence));
    assert_eq!(functor.correctness().get(*uniqueness_id), Some(*uniqueness));
    for (index, (row, site, start, end, justification_start, justification_end, kind, spelling)) in
        [
            (
                *existence,
                99,
                182,
                217,
                192,
                216,
                mizar_checker::source_functor_definition::SourceFunctorCorrectnessKind::Existence,
                "existence by computation(steps: 1);",
            ),
            (
                *uniqueness,
                103,
                220,
                256,
                231,
                255,
                mizar_checker::source_functor_definition::SourceFunctorCorrectnessKind::Uniqueness,
                "uniqueness by computation(steps: 1);",
            ),
        ]
        .into_iter()
        .enumerate()
    {
        assert_eq!(row.id().index(), index);
        assert_eq!(row.owner().index(), 1);
        assert_eq!(row.ordinal(), index);
        assert_eq!(row.kind(), kind);
        assert_eq!(
            row.site(),
            &mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(site)
            )
        );
        assert_eq!(row.source_range(), task260_range(ast.source_id, start, end));
        assert_eq!(
            row.justification(),
            &mizar_session::SourceAnchor::Range(task260_range(
                ast.source_id,
                justification_start,
                justification_end,
            ))
        );
        assert_eq!(
            row.recovery(),
            mizar_checker::source_functor_definition::SourceFunctorDefinitionRecovery::Normal
        );
        assert_eq!(row.spelling(), spelling);
        assert_eq!(row.obligation().index(), index);
    }

    let obligations = output
        .typed_ast
        .initial_obligations()
        .iter()
        .collect::<Vec<_>>();
    let [
        (existence_obligation_id, existence_obligation),
        (uniqueness_obligation_id, uniqueness_obligation),
    ] = obligations.as_slice()
    else {
        panic!("Task260 pending-obligation rows changed");
    };
    for (index, (table_id, row, kind, owner, start, end, goal, provenance)) in [
        (
            *existence_obligation_id,
            *existence_obligation,
            mizar_checker::typed_ast::InitialObligationKind::FunctorExistence,
            99,
            182,
            217,
            "source.definition.functor.correctness:definition=1:existence",
            "source.definition.functor:definition=1:correctness=0",
        ),
        (
            *uniqueness_obligation_id,
            *uniqueness_obligation,
            mizar_checker::typed_ast::InitialObligationKind::FunctorUniqueness,
            103,
            220,
            256,
            "source.definition.functor.correctness:definition=1:uniqueness",
            "source.definition.functor:definition=1:correctness=1",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(table_id.index(), index);
        assert_eq!(row.id.index(), index);
        assert_eq!(row.kind, kind);
        assert_eq!(
            &row.owner,
            &mizar_checker::typed_ast::TypedSiteRef::Node(
                mizar_checker::typed_ast::TypedNodeId::new(owner)
            )
        );
        assert_eq!(row.source_range, task260_range(ast.source_id, start, end));
        assert!(row.assumptions.is_empty());
        assert_eq!(row.goal.as_str(), goal);
        assert_eq!(row.provenance.as_str(), provenance);
        assert_eq!(
            row.status,
            mizar_checker::typed_ast::InitialObligationStatus::Pending
        );
    }

    let handoff_debug = functor.debug_text();
    assert_eq!(
        handoff_debug
            .matches("source-functor-definition-debug-v1")
            .count(),
        1
    );
    for row_prefix in [
        "definition#0 ",
        "definition#1 ",
        "parameter#0 ",
        "parameter#1 ",
        "guard#0 ",
        "definiens#0 ",
        "definiens#1 ",
        "correctness#0 ",
        "correctness#1 ",
    ] {
        assert!(
            handoff_debug
                .lines()
                .any(|line| line.starts_with(row_prefix)),
            "missing debug grammar row {row_prefix}"
        );
    }
    assert!(handoff_debug.contains("application-fingerprint: none"));
    assert!(handoff_debug.contains("structure-fingerprint: none"));
    assert!(handoff_debug.contains("set-term-fingerprint: none"));
    assert!(handoff_debug.contains("atomic-formula-fingerprint: \""));
    assert_eq!(
        output
            .typed_ast
            .debug_text()
            .matches("source-functor-definition-debug-v1")
            .count(),
        1
    );
    assert_eq!(
        output
            .resolved
            .debug_text()
            .matches("source-functor-definition-debug-v1")
            .count(),
        1
    );
    assert_eq!(
        output.typed_ast.source_functor_definition(),
        output.resolved.source_functor_definition()
    );
}

#[derive(Debug, Clone, Copy)]
enum Task260SurfaceMutation {
    StructuralKind,
    StructuralRange,
    StructuralChildren,
    RootIdentity,
    RootChildOrder,
    DefinitionSiblingOrder,
    DefinitionChildRelocation,
    PatternKind,
    LabelToken,
    ReturnToken,
    CorrectnessKind,
    JustificationDescendant,
    ComputationDescendant,
    TokenRecovery,
    ExpressionRoot,
}

fn task260_mutated_surface_ast(ast: &SurfaceAst, mutation: Task260SurfaceMutation) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(ast.source_id);
    let mut rebuilt = Vec::<SurfaceBuilderNodeId>::with_capacity(ast.nodes().len());
    for (index, node) in ast.nodes().iter().enumerate() {
        let kind = match (index, mutation) {
            (63, Task260SurfaceMutation::StructuralKind) => SurfaceNodeKind::AttributeChain,
            (78, Task260SurfaceMutation::PatternKind) => SurfaceNodeKind::PredicatePattern,
            (99, Task260SurfaceMutation::CorrectnessKind) => SurfaceNodeKind::PropertyClause,
            _ => node.kind.clone(),
        };
        let range = match (index, mutation) {
            (63, Task260SurfaceMutation::StructuralRange) => SourceRange {
                source_id: ast.source_id,
                start: 22,
                end: 24,
            },
            (98, Task260SurfaceMutation::JustificationDescendant) => SourceRange {
                source_id: ast.source_id,
                start: 193,
                end: 216,
            },
            _ => node.range,
        };
        let mut children = node
            .children
            .iter()
            .map(|child| rebuilt[child.index()])
            .collect::<Vec<_>>();
        match (index, mutation) {
            (63, Task260SurfaceMutation::StructuralChildren) => children.clear(),
            (107, Task260SurfaceMutation::RootIdentity) => children.clear(),
            (107, Task260SurfaceMutation::RootChildOrder) => children.swap(0, 1),
            (104, Task260SurfaceMutation::DefinitionSiblingOrder) => children.swap(1, 2),
            (84, Task260SurfaceMutation::DefinitionChildRelocation) => {
                children.remove(3);
            }
            (104, Task260SurfaceMutation::DefinitionChildRelocation) => {
                children[4] = rebuilt[78];
            }
            _ => {}
        }
        let rebuilt_id = match kind {
            SurfaceNodeKind::Token(token) => {
                if index == 4 && matches!(mutation, Task260SurfaceMutation::TokenRecovery) {
                    builder.add_recovered_token(token.kind, token.text, range)
                } else {
                    let text = match (index, mutation) {
                        (17, Task260SurfaceMutation::LabelToken) => "Task260CorruptedDef",
                        (24, Task260SurfaceMutation::ReturnToken) => "object",
                        (48, Task260SurfaceMutation::ComputationDescendant) => "2",
                        _ => token.text.as_ref(),
                    };
                    builder.add_token(token.kind, text, range)
                }
            }
            structural => builder.add_node(structural, range, children),
        };
        rebuilt.push(rebuilt_id);
    }
    let expression_root = if matches!(mutation, Task260SurfaceMutation::ExpressionRoot) {
        Some(rebuilt[63])
    } else {
        ast.expression_root()
            .map(|expression_root| rebuilt[expression_root.index()])
    };
    let root = if matches!(mutation, Task260SurfaceMutation::RootIdentity) {
        Some(rebuilt[106])
    } else {
        ast.root().map(|root| rebuilt[root.index()])
    };
    builder.finish(root, expression_root)
}

#[test]
fn functor_return_expression_layout_has_real_frontend_witnesses() {
    let (canonical, _, _, _, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(SOURCE_FUNCTOR_DEFINITION_TEXT, 260_300);
    assert_eq!(diagnostics, 0);
    assert_eq!(canonical.nodes().len(), 108);
    assert_eq!(canonical.root().map(|id| id.index()), Some(107));
    assert!(canonical.expression_root().is_none());
    assert!(canonical.nodes().iter().all(|node| {
        !node.recovered && node.range.source_id == canonical.source_id
    }));
    let first = SOURCE_FUNCTOR_DEFINITION_TEXT.replace("-> set equals", "->set  equals");
    let both = first.replace(
        "task260_means(y) -> set means",
        "task260_means(y)->set   means",
    );
    let first_owner = SOURCE_FUNCTOR_DEFINITION_TEXT
        .replace("  func Task260EqualsDef", "   func Task260EqualsDef")
        .replace("  func Task260MeansDef", " func Task260MeansDef");
    let both_owners = SOURCE_FUNCTOR_DEFINITION_TEXT
        .replace("  func", "   func")
        .replace("  existence", "existence");
    for (ordinal, (source, first_start, second_start, first_shift, second_shift)) in [
        (first, 104, 163, 0, 0),
        (both, 104, 161, 0, 0),
        (first_owner, 106, 163, 1, 0),
        (both_owners, 106, 165, 1, 2),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(source.len(), 262);
        let (ast, module, shells, symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&source, 260_301 + ordinal);
        assert_eq!(diagnostics, 0, "witness {ordinal}");
        assert_eq!(ast.nodes().len(), 108);
        assert_eq!(ast.root().map(|id| id.index()), Some(107));
        assert!(ast.expression_root().is_none());
        for (index, (actual, expected)) in ast.nodes().iter().zip(canonical.nodes()).enumerate() {
            assert!(!actual.recovered, "witness {ordinal} node {index}");
            assert_eq!(actual.range.source_id, ast.source_id);
            assert_eq!(actual.kind, expected.kind, "node {index} kind/text");
            assert_eq!(actual.children, expected.children, "node {index} children");
            let shift = match index {
                16..=27 | 78..=84 => first_shift,
                28..=41 | 85..=95 => second_shift,
                _ => 0,
            };
            // Owner witnesses move every inner range; geometry outside owners stays fixed.
            if ordinal >= 2
                || matches!(index, 0..=22 | 25..=34 | 37..=78 | 81..=85 | 88..=107)
            {
                assert_eq!(
                    (actual.range.start, actual.range.end),
                    (expected.range.start + shift, expected.range.end + shift),
                    "node {index} geometry"
                );
            }
        }
        for (owner, start, width) in [
            (84, 61 + first_shift, 57),
            (95, 121 + second_shift, 58),
        ] {
            assert_eq!(
                ast.nodes()[owner].range,
                task260_range(ast.source_id, start, start + width)
            );
            if ordinal >= 2 {
                let original = canonical.nodes()[owner].range;
                assert_eq!(
                    &source[start..start + width],
                    &SOURCE_FUNCTOR_DEFINITION_TEXT[original.start..original.end]
                );
            }
        }
        for (token, head, root, start) in [(24, 79, 80, first_start), (36, 86, 87, second_start)] {
            assert_eq!(ast.nodes()[token].token_text(), Some("set"));
            for index in [token, head, root] {
                assert_eq!(
                    ast.nodes()[index].range,
                    task260_range(ast.source_id, start, start + 3)
                );
            }
        }
        assert!(
            source_functor_definition_output(&ast, module, &shells, &symbols, &source).is_none()
        );
    }
}

#[test]
fn functor_prefix_geometry_has_real_frontend_witnesses() {
    let (canonical, canonical_module, canonical_shells, canonical_symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(SOURCE_FUNCTOR_DEFINITION_TEXT, 260_320);
    assert_eq!(diagnostics, 0);
    assert_eq!(canonical.nodes().len(), 108);
    let canonical_before = (canonical.clone(), canonical_shells.clone(), canonical_symbols.clone());
    let original = source_functor_definition_output(
        &canonical, canonical_module.clone(), &canonical_shells, &canonical_symbols,
        SOURCE_FUNCTOR_DEFINITION_TEXT,
    ).expect("original native selector").expect("original native public producer chain");
    let mut sources = metamorphic_edits::variants(&canonical, SOURCE_FUNCTOR_DEFINITION_TEXT)
        .into_iter()
        .filter(|(kind, _)| matches!(*kind, "leading_line" | "intertoken_layout"))
        .map(|(kind, edits)| {
            (kind, metamorphic_edits::apply(SOURCE_FUNCTOR_DEFINITION_TEXT, &edits))
        })
        .collect::<Vec<_>>();
    assert_eq!(sources.iter().map(|(kind, _)| *kind).collect::<Vec<_>>(),
        ["leading_line", "intertoken_layout"]);
    assert_eq!(sources[0].1, format!("\n{SOURCE_FUNCTOR_DEFINITION_TEXT}"));
    assert_eq!(sources[1].1, SOURCE_FUNCTOR_DEFINITION_TEXT.replacen(
        "definition", "definition :: metamorphic\n", 1));
    sources.push(("mixed_prefix_intertoken", format!(" \t\n{}", sources[1].1)));
    sources.push(("multiline_gap", SOURCE_FUNCTOR_DEFINITION_TEXT.replacen(
        "definition", "definition ::= gap\nbody =::", 1)));
    sources.push(("cutoff20", format!("{}{}", " ".repeat(20), SOURCE_FUNCTOR_DEFINITION_TEXT)));
    sources.push(("cutoff21", format!("{}{}", " ".repeat(21), SOURCE_FUNCTOR_DEFINITION_TEXT)));
    let mut outcomes = Vec::new();
    for (ordinal, ((case, source), (m, d))) in sources
        .into_iter()
        .zip([(1, 1), (0, 16), (3, 19), (0, 17), (20, 20), (21, 21)])
        .enumerate()
    {
        assert_eq!(source.len(), 262 + d, "{case}");
        if m != d {
            let lexical = mizar_lexer::preprocess_source_for_lexing(&source);
            assert!(lexical.diagnostics.is_empty(), "{case}");
            assert_eq!(lexical.comments.len(), 1, "{case}");
            assert_eq!(lexical.comments[0].kind, if case == "multiline_gap" {
                mizar_lexer::CommentKind::MultiLine
            } else { mizar_lexer::CommentKind::SingleLine });
            assert!(lexical.comments[0].span.start >= m + 10);
            assert!(lexical.comments[0].span.end <= d + 13);
        }
        assert_eq!(&source[d + 13..], &SOURCE_FUNCTOR_DEFINITION_TEXT[13..], "{case} body bytes");
        let (ast, module, shells, symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&source, 260_321 + ordinal);
        assert_eq!(diagnostics, 0, "{case} native Frontend diagnostics");
        assert_eq!((ast.nodes().len(), ast.root().map(|id| id.index())), (108, Some(107)));
        assert!(ast.expression_root().is_none());
        assert_eq!(ast.token_nodes().iter().map(|id| id.index()).collect::<Vec<_>>(), (0..62).collect::<Vec<_>>());
        for (index, (actual, expected)) in ast.nodes().iter().zip(canonical.nodes()).enumerate() {
            assert!(!actual.recovered, "{case} node {index}");
            assert_eq!(actual.kind, expected.kind, "{case} node {index} kind/text");
            assert_eq!(actual.children, expected.children, "{case} node {index} ordered children");
            let (start, end) = match index {
                0 => (m, m + 10),
                104..=107 => (m, d + 261),
                _ => (expected.range.start + d, expected.range.end + d),
            };
            assert_eq!(actual.range, task260_range(ast.source_id, start, end), "{case} node {index}");
            if let Some(text) = actual.token_text() {
                assert_eq!(&source[start..end], text, "{case} token {index}");
            }
        }
        for owner in [84, 95, 83, 94] {
            let original = canonical.nodes()[owner].range;
            let actual = ast.nodes()[owner].range;
            assert_eq!(&source[actual.start..actual.end],
                &SOURCE_FUNCTOR_DEFINITION_TEXT[original.start..original.end],
                "{case} definition/body {owner} bytes");
        }
        assert_eq!(shells.declarations().len(), 3);
        assert!(shells.exports().is_empty());
        for (index, (shell, original)) in shells.declarations().iter()
            .zip(canonical_shells.declarations()).enumerate()
        {
            assert_eq!(shell.id(), original.id());
            assert_eq!(shell.ordinal(), original.ordinal());
            assert_eq!(shell.kind(), original.kind());
            assert_eq!(shell.node_id(), original.node_id());
            assert_eq!(shell.parent(), original.parent());
            assert_eq!(shell.module(), &module);
            assert!(!shell.recovered());
            let start = if index == 0 { m } else { original.range().start + d };
            assert_eq!(shell.range(), task260_range(ast.source_id, start, original.range().end + d));
        }
        assert_eq!(symbols.module_id(), &module);
        assert_eq!((symbols.symbols().len(), symbols.definitions().len(), symbols.contributions().len()), (2, 2, 1));
        for (index, definition) in symbols.definitions().iter().enumerate() {
            let symbol = symbols.symbols().get(definition.symbol()).expect("native functor symbol");
            assert_eq!(definition.id().index(), index);
            assert_eq!(symbol.contribution(), definition.contribution());
            let original = canonical.nodes()[[84, 95][index]].range;
            let range = task260_range(ast.source_id, original.start + d, original.end + d);
            for origin in [definition.origin(), symbol.origin()] {
                assert_eq!(origin.source_id(), ast.source_id);
                assert_eq!(origin.module_id(), &module);
                assert_eq!(origin.anchor(), &mizar_session::SourceAnchor::Range(range));
                assert_eq!(origin.structural_path(), &[4, 0, 9, index as u32]);
                assert!(origin.import_edge().is_none());
                assert!(!origin.is_recovered());
            }
            let contribution = symbols.contributions().get(definition.contribution())
                .expect("native functor contribution");
            assert_eq!(contribution.module(), &module);
            assert!(matches!(contribution.kind(),
                mizar_resolve::env::ContributionKind::LocalSource { source_id }
                    if *source_id == ast.source_id));
            assert_eq!(contribution.anchor(), &mizar_session::SourceAnchor::Range(
                task260_range(ast.source_id, m, d + 261)));
            assert!(contribution.effects().symbols().contains(definition.symbol()));
            assert!(contribution.effects().definitions().contains(&definition.id()));
        }
        let before = (ast.clone(), shells.clone(), symbols.clone());
        let result = source_functor_definition_output(&ast, module.clone(), &shells, &symbols, &source);
        outcomes.push((case, source, m, d, ast, module, shells, symbols, before, result));
    }
    let failures = outcomes.iter().filter_map(|(case, _, _, d, _, _, _, _, _, result)| {
        match (*d <= 20, result) {
            (true, Some(Ok(_))) | (false, None) => None,
            (_, None) => Some(format!("{case}: None")),
            (_, Some(Err(error))) => Some(format!("{case}: SomeErr({error})")),
            (false, Some(Ok(_))) => Some(format!("{case}: unsupported cutoff selected")),
        }
    }).collect::<Vec<_>>();
    assert!(failures.is_empty(), "actual native prefix outcomes: {failures:?}");
    for (case, source, m, d, ast, module, shells, symbols, before, result) in outcomes {
        assert_eq!((&ast, &shells, &symbols), (&before.0, &before.1, &before.2), "{case} parsed inputs");
        if d > 20 { continue; }
        let output = result.unwrap().unwrap();
        let replay = source_functor_definition_output(&ast, module, &shells, &symbols, &source)
            .expect("fresh native selector").expect("fresh native producer replay");
        assert_eq!(output.typed_ast.debug_text(), replay.typed_ast.debug_text(), "{case}");
        assert_eq!(output.resolved.debug_text(), replay.resolved.debug_text(), "{case}");
        assert_eq!((&ast, &shells, &symbols), (&before.0, &before.1, &before.2), "{case} replay inputs");
        let typed = &output.typed_ast;
        let context = typed.source_context().expect("actual Context/BindingEnv");
        let types = typed.source_type().expect("actual generic Type/249R");
        let terms = typed.source_term().expect("actual Primary");
        let atomic = typed.source_atomic_formula().expect("actual Atomic");
        let functor = typed.source_functor_definition().expect("actual upper Functor");
        assert_eq!((context.items().len(), context.declarations().len(), context.binding_env().bindings().len()), (1, 2, 2));
        assert_eq!((types.applications().len(), types.expressions().len(), types.definition_returns().len(), terms.terms().len(), terms.references().len()), (2, 4, 2, 5, 5));
        assert_eq!((atomic.formulas().len(), atomic.edges().len(), atomic.requests().len()), (2, 4, 4));
        assert_eq!((functor.definitions().len(), functor.parameters().len(), functor.guards().len(), functor.definientia().len(), functor.correctness().len()), (2, 2, 1, 2, 2));
        assert_eq!(context.items().iter().next().unwrap().1.source_range, task260_range(ast.source_id, m, d + 261));
        assert_eq!(functor.source_context_fingerprint(), context.debug_text());
        assert_eq!(functor.source_type_fingerprint(), types.debug_text());
        assert_eq!(functor.source_term_fingerprint(), terms.debug_text());
        assert_eq!(functor.atomic_formula_fingerprint(), Some(atomic.debug_text().as_str()));
        assert_eq!(atomic.primary_term_fingerprint(), terms.debug_text());
        assert_eq!(output.resolved.source_context(), Some(context));
        assert_eq!(output.resolved.source_type(), Some(types));
        assert_eq!(output.resolved.source_term(), Some(terms));
        assert_eq!(output.resolved.source_atomic_formula(), Some(atomic));
        assert_eq!(output.resolved.source_functor_definition(), Some(functor));
        for (index, surface) in ast.nodes().iter().enumerate() {
            let expected = match index {
                65 => ast.nodes()[2].range,
                69 => ast.nodes()[7].range,
                _ => surface.range,
            };
            let node = typed.nodes().node(mizar_checker::typed_ast::TypedNodeId::new(index)).unwrap();
            assert_eq!(node.anchor, mizar_session::SourceAnchor::Range(expected), "{case} actual Arena row {index}");
            assert_eq!(node.kind, original.typed_ast.nodes().node(mizar_checker::typed_ast::TypedNodeId::new(index)).unwrap().kind);
            assert_eq!(node.children.iter().map(|id| id.index()).collect::<Vec<_>>(), surface.children.iter().map(|id| id.index()).collect::<Vec<_>>());
            let resolved = output.resolved.nodes().iter().find(|(_, row)| row.typed_node.index() == index).unwrap().1;
            assert_eq!(resolved.source_range, expected, "{case} final row {index}");
            assert!(resolved.final_type.is_none());
            assert!(resolved.metadata.is_none());
        }
        for (index, (_, row)) in functor.definitions().iter().enumerate() {
            let owner = types.definition_returns().get(row.return_type()).unwrap();
            let expression = types.expressions().get(owner.root()).unwrap();
            assert_eq!(row.source_range(), ast.nodes()[[84, 95][index]].range);
            assert_eq!(row.spelling(), &source[row.source_range().start..row.source_range().end]);
            assert_eq!(row.source_range(), owner.definition_range());
            assert_eq!(row.site(), owner.definition_site());
            assert_eq!(row.origin(), symbols.definitions().get(row.definition()).unwrap().origin());
            assert_eq!(expression.source_range(), ast.nodes()[[80, 87][index]].range);
            assert_eq!(expression.head_range(), ast.nodes()[[79, 86][index]].range);
        }
        for (index, (_, row)) in functor.parameters().iter().enumerate() {
            let declaration = context.declarations().iter().nth(index).unwrap().1;
            let binding = context.binding_env().bindings().get(row.binding()).unwrap();
            let application = types.applications().get(row.written_type()).unwrap();
            let expression = types.expressions().get(application.root()).unwrap();
            assert_eq!(row.source_range(), ast.nodes()[[65, 69][index]].range);
            assert_eq!(row.spelling(), &source[row.source_range().start..row.source_range().end]);
            assert_eq!(row.declaration_range(), ast.nodes()[[2, 7][index]].range);
            assert_eq!(row.binding(), declaration.binding);
            assert_eq!(binding.declaration_range, declaration.declaration_range);
            assert_eq!(row.declaration_range(), declaration.declaration_range);
            assert_eq!(application.binding(), row.binding());
            assert_eq!(expression.source_range(), declaration.written_type_range);
            assert_eq!(expression.source_range(), ast.nodes()[[63, 67][index]].range);
            assert_eq!(binding.type_site, mizar_checker::binding_env::BindingTypeSite::Source(expression.source_range()));
        }
        let guard = functor.guards().iter().next().unwrap().1;
        assert_eq!(atomic.formulas().get(guard.formula()).unwrap().source_range(), ast.nodes()[75].range);
        for (_, row) in functor.definientia().iter() {
            let target = match row.target() {
                mizar_checker::source_functor_definition::SourceFunctorDefiniensTarget::Primary(id) => terms.terms().get(id).unwrap().source_range(),
                mizar_checker::source_functor_definition::SourceFunctorDefiniensTarget::AtomicFormula(id) => atomic.formulas().get(id).unwrap().source_range(),
                _ => panic!("{case} unexpected definiens target"),
            };
            assert_eq!(row.source_range(), target);
            assert_eq!(functor.definitions().get(row.owner()).unwrap().definiens(), row.id());
        }
        assert_eq!(typed.initial_obligations().len(), 2);
        for (index, (_, row)) in functor.correctness().iter().enumerate() {
            let obligation = typed.initial_obligations().get(row.obligation()).unwrap();
            let old = original.typed_ast.initial_obligations().get(row.obligation()).unwrap();
            assert_eq!(obligation.source_range, ast.nodes()[[99, 103][index]].range);
            assert_eq!(obligation.status, mizar_checker::typed_ast::InitialObligationStatus::Pending);
            assert_eq!((&obligation.kind, &obligation.owner, &obligation.assumptions, &obligation.goal, &obligation.provenance), (&old.kind, &old.owner, &old.assumptions, &old.goal, &old.provenance));
            assert_eq!(row.justification(), &mizar_session::SourceAnchor::Range(ast.nodes()[[98, 102][index]].range));
        }
        assert!(typed.types().is_empty() && typed.facts().is_empty() && typed.coercions().is_empty() && typed.diagnostics().is_empty());
        assert!(output.resolved.checked_formulas().is_empty() && output.resolved.checked_proofs().is_empty() && output.resolved.checked_proof_nodes().is_empty() && output.resolved.checked_terminal_goals().is_empty() && output.resolved.statement_semantics().is_empty() && output.resolved.cluster_facts().is_empty() && output.resolved.expr_metadata().is_empty() && output.resolved.inserted_coercions().is_empty() && output.resolved.diagnostics().is_empty());
        let debug = format!("{}{}", typed.debug_text(), output.resolved.debug_text()).to_ascii_lowercase();
        for forbidden in ["accepted", "activated", "core-ir", "control-flow", "verification-condition"] {
            assert!(!debug.contains(forbidden), "{case} leaked {forbidden}");
        }
    }
    assert_eq!((canonical, canonical_shells, canonical_symbols), canonical_before, "original parsed artifacts");
}

#[test]
fn functor_raw_prefix_authentication_preserves_refusals() {
    let source = SOURCE_FUNCTOR_DEFINITION_TEXT.replacen("definition", "definition :: metamorphic\n", 1);
    let (ast, module, shells, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(&source, 260_340);
    assert_eq!(diagnostics, 0);
    source_functor_definition_output(&ast, module.clone(), &shells, &symbols, &source)
        .expect("authenticated intertoken family").expect("actual complete native baseline");
    let (canonical, canonical_module, canonical_shells, canonical_symbols) =
        task253_ast_from_source_text(SOURCE_FUNCTOR_DEFINITION_TEXT, 260_341);
    assert!(source_functor_definition_output(
        &ast, module.clone(), &shells, &symbols, SOURCE_FUNCTOR_DEFINITION_TEXT,
    ).is_none(), "current displaced AST cannot authenticate canonical raw text");
    assert!(source_functor_definition_output(
        &canonical, canonical_module, &canonical_shells, &canonical_symbols, &source,
    ).is_none(), "canonical AST cannot authenticate displaced raw text");
    for mutation in [Task260SurfaceMutation::StructuralKind, Task260SurfaceMutation::ComputationDescendant] {
        let malformed = task260_mutated_surface_ast(&ast, mutation);
        assert!(source_functor_definition_output(&malformed, module.clone(), &shells, &symbols, &source).is_none(),
            "owned/unowned Surface corruption {mutation:?}");
    }
    for (ordinal, (case, unsupported)) in [
        ("no_comment_changed_gap", SOURCE_FUNCTOR_DEFINITION_TEXT.replacen("definition\n  let", "definition \n let", 1)),
        ("documentation_gap", source.replacen(":: metamorphic", "::: metamorphic", 1)),
        ("prefix_comment", format!(":: prefix\n{SOURCE_FUNCTOR_DEFINITION_TEXT}")),
        ("parameter_comment", source.replacen("let x be", "let x :: parameter\nbe", 1)),
        ("body_comment", source.replacen("means x = y", "means x :: body\n= y", 1)),
        ("annotation", source.replacen("  func", "  @latex(\"P(x)\")\n  func", 1)),
        ("gap_cr", source.replacen("definition ", "definition\r", 1)),
        ("prefix_cr", format!("\r{source}")),
        ("suffix_comment", format!("{source}:: suffix\n")),
    ].into_iter().enumerate() {
        if case == "no_comment_changed_gap" { assert_eq!(unsupported.len(), SOURCE_FUNCTOR_DEFINITION_TEXT.len()); }
        if case == "documentation_gap" {
            assert_eq!(mizar_lexer::preprocess_source_for_lexing(&unsupported).comments[0].kind, mizar_lexer::CommentKind::Documentation);
        }
        let (candidate, candidate_module, candidate_shells, candidate_symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&unsupported, 260_342 + ordinal);
        if !matches!(case, "gap_cr" | "prefix_cr") { assert_eq!(diagnostics, 0, "{case} genuine Frontend witness"); }
        assert!(source_functor_definition_output(
            &candidate, candidate_module, &candidate_shells, &candidate_symbols, &unsupported,
        ).is_none(), "{case} remains outside the source family");
    }
    // Selected families keep owner-stage errors after source/Surface authentication.
    for (mutation, owner) in [
        (SourceFunctorDefinitionRouteMutation::WrongContextBindingSite(0), "Task248 source context:"),
        (SourceFunctorDefinitionRouteMutation::RemoveTypeExpression, "Task249 source type:"),
        (SourceFunctorDefinitionRouteMutation::WrongReturnRange, "Task249R return type:"),
        (SourceFunctorDefinitionRouteMutation::WrongTermBinding(4), "Task252 source term:"),
        (SourceFunctorDefinitionRouteMutation::WrongAtomicEdge(0), "Task256 atomic formula:"),
        (SourceFunctorDefinitionRouteMutation::RemoveFunctorGuard, "Task260 functor definition:"),
    ] {
        let error = source_functor_definition_output_with_mutation(&ast, module.clone(), &shells, &symbols, &source, mutation)
            .expect("actual intertoken source remains selected").expect_err("existing lower corruption must fail");
        assert!(error.starts_with(owner), "{mutation:?}: {error}");
    }
    // Rebind only geometry within the current bundle: source/module/IDs and all
    // signatures/effects stay current, while origins and contribution are canonical.
    let mut stale_symbols = mizar_resolve::env::SymbolIndex::new();
    let mut stale_definitions = mizar_resolve::env::DefinitionIndex::new();
    for (index, old) in symbols.definitions().iter().enumerate() {
        let symbol = symbols.symbols().get(old.symbol()).unwrap();
        let expected = &TASK260_EXPECTED_SURFACE_ROWS[[84, 95][index]];
        let origin = mizar_resolve::resolved_ast::SemanticOrigin::new(
            ast.source_id, module.clone(),
            mizar_session::SourceAnchor::Range(task260_range(ast.source_id, expected.start, expected.end)),
            old.origin().structural_path().to_vec(),
        );
        assert_eq!(origin.source_id(), old.origin().source_id());
        assert_eq!(origin.module_id(), old.origin().module_id());
        stale_symbols.insert(mizar_resolve::env::SymbolEntry::new(
            symbol.symbol().clone(), symbol.kind(), symbol.namespace().clone(), symbol.primary_spelling(), origin.clone(), symbol.contribution(),
        ).with_visibility(symbol.visibility()).with_export_status(symbol.export_status())
            .with_notation_spelling(symbol.notation_spelling().unwrap()).with_signature(symbol.signature().unwrap().clone())
            .with_relations(symbol.relations().to_vec()));
        assert!(old.doc_attachment().is_none() && old.conflict().is_none());
        assert_eq!(stale_definitions.insert(mizar_resolve::env::DefinitionShell::new(
            old.symbol().clone(), old.kind(), origin, old.contribution(),
        ).with_visibility(old.visibility()).with_parameters(old.parameters().to_vec()).with_binders(old.binders().to_vec())
            .with_dependencies(old.dependencies().to_vec()).with_arity(old.arity().unwrap())
            .with_notation_shape(old.notation_shape().unwrap()).with_signature(old.signature().unwrap().clone())), old.id());
    }
    let mut stale_contributions = mizar_resolve::env::SourceContributionIndex::new();
    for old in symbols.contributions().iter() {
        let id = stale_contributions.insert(module.clone(), old.kind().clone(),
            mizar_session::SourceAnchor::Range(task260_range(ast.source_id, 0, 261)));
        assert_eq!(id, old.id());
        for symbol in old.effects().symbols() { stale_contributions.add_symbol(id, symbol.clone()); }
        for definition in old.effects().definitions() { stale_contributions.add_definition(id, *definition); }
        assert_eq!(stale_contributions.get(id).unwrap().effects(), old.effects());
    }
    let stale = mizar_resolve::env::SymbolEnv::new(module.clone(), mizar_resolve::env::SymbolEnvIndexes {
        imports: symbols.imports().clone(), exports: symbols.exports().clone(), symbols: stale_symbols,
        labels: symbols.labels().clone(), definitions: stale_definitions, overloads: symbols.overloads().clone(),
        registrations: symbols.registrations().clone(), lexical_summaries: symbols.lexical_summaries().clone(),
        namespace_graph: symbols.namespace_graph().clone(), declaration_dependencies: symbols.declaration_dependencies().clone(),
        contributions: stale_contributions, module_summaries: symbols.module_summaries().clone(),
    });
    let error = source_functor_definition_output(&ast, module, &shells, &stale, &source)
        .expect("same-source stale resolver remains selected").expect_err("canonical resolver ranges are stale");
    assert_eq!(error, "Task260 resolver: functor origin is not exact");
}

#[test]
fn task260_source_ast_resolver_and_lower_mutations_fail_at_the_owner() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_FUNCTOR_DEFINITION_TEXT, 260_010);
    for (ordinal, near_source) in [
        SOURCE_FUNCTOR_DEFINITION_TEXT
            .trim_end_matches('\n')
            .to_owned(),
        format!("{SOURCE_FUNCTOR_DEFINITION_TEXT}\n"),
        SOURCE_FUNCTOR_DEFINITION_TEXT.replace("let y be set;", "let z be set;"),
        SOURCE_FUNCTOR_DEFINITION_TEXT.replace("assume x = x;", "assume y = y;"),
        SOURCE_FUNCTOR_DEFINITION_TEXT.replace("task260_equals", "task261_equals"),
        SOURCE_FUNCTOR_DEFINITION_TEXT.replace(
            "uniqueness by computation(steps: 1);",
            "uniqueness by computation(steps: 2);",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (near_ast, near_module, near_shells, near_symbols) =
            task253_ast_from_source_text(&near_source, 260_020 + ordinal);
        assert!(
            source_functor_definition_output(
                &near_ast,
                near_module,
                &near_shells,
                &near_symbols,
                &near_source,
            )
            .is_none(),
            "surface near miss {ordinal} selected"
        );
    }

    for mutation in [
        Task260SurfaceMutation::StructuralKind,
        Task260SurfaceMutation::StructuralRange,
        Task260SurfaceMutation::StructuralChildren,
        Task260SurfaceMutation::RootIdentity,
        Task260SurfaceMutation::RootChildOrder,
        Task260SurfaceMutation::DefinitionSiblingOrder,
        Task260SurfaceMutation::DefinitionChildRelocation,
        Task260SurfaceMutation::PatternKind,
        Task260SurfaceMutation::LabelToken,
        Task260SurfaceMutation::ReturnToken,
        Task260SurfaceMutation::CorrectnessKind,
        Task260SurfaceMutation::JustificationDescendant,
        Task260SurfaceMutation::ComputationDescendant,
        Task260SurfaceMutation::TokenRecovery,
        Task260SurfaceMutation::ExpressionRoot,
    ] {
        let malformed_ast = task260_mutated_surface_ast(&ast, mutation);
        assert!(
            source_functor_definition_output(
                &malformed_ast,
                module.clone(),
                &shells,
                &symbols,
                SOURCE_FUNCTOR_DEFINITION_TEXT,
            )
            .is_none(),
            "same-source malformed Surface AST {mutation:?} selected"
        );
    }

    let wrong_module = mizar_resolve::resolved_ast::ModuleId::new(
        mizar_session::PackageId::new("task260"),
        mizar_session::ModulePath::new("task260.wrong"),
    );
    let resolver_error = source_functor_definition_output(
        &ast,
        wrong_module,
        &shells,
        &symbols,
        SOURCE_FUNCTOR_DEFINITION_TEXT,
    )
    .expect("exact source remains selected")
    .expect_err("foreign module must fail");
    assert!(resolver_error.starts_with("Task260 resolver:"));

    for (mutation, owner) in [
        (
            SourceFunctorDefinitionRouteMutation::RemoveResolverShell,
            "Task260 resolver:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongResolverProjection,
            "Task260 resolver:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongResolverEntry,
            "Task260 resolver:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongResolverDefinitionEntry,
            "Task260 resolver:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongResolverContribution,
            "Task260 resolver:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongContextModuleSite,
            "Task248 source context:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongContextItemSite,
            "Task248 source context:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongContextBindingSite(0),
            "Task248 source context:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongContextBindingSite(1),
            "Task248 source context:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongContextBindingOwner(0),
            "Task248 source context:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongContextBindingOwner(1),
            "Task248 source context:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::RemoveTypeExpression,
            "Task249 source type:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongTypeApplicationBinding(0),
            "Task249 source type:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongTypeApplicationBinding(1),
            "Task249 source type:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongTypeApplicationRoot(0),
            "Task249 source type:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongTypeApplicationRoot(1),
            "Task249 source type:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongTypeExpressionSite(0),
            "Task249 source type:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongTypeExpressionSite(1),
            "Task249 source type:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongReturnType,
            "Task249R return type:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongReturnRange,
            "Task249R return type:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongReturnExpression(0),
            "Task249R return type:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongReturnExpression(1),
            "Task249R return type:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongTermBinding(0),
            "Task252 source term:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongTermBinding(1),
            "Task252 source term:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongTermBinding(2),
            "Task252 source term:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongTermBinding(3),
            "Task252 source term:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongTermBinding(4),
            "Task252 source term:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongTermSite(0),
            "Task252 source term:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongTermSite(1),
            "Task252 source term:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongTermSite(2),
            "Task252 source term:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongTermSite(3),
            "Task252 source term:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongTermSite(4),
            "Task252 source term:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::RemoveAtomicFormula,
            "Task256 atomic formula:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::RemoveAtomicEdge,
            "Task256 atomic formula:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongAtomicFormula(0),
            "Task256 atomic formula:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongAtomicFormula(1),
            "Task256 atomic formula:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongAtomicEdge(0),
            "Task256 atomic formula:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongAtomicEdge(1),
            "Task256 atomic formula:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongAtomicEdge(2),
            "Task256 atomic formula:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongAtomicEdge(3),
            "Task256 atomic formula:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongAtomicRequest(0),
            "Task256 atomic formula:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongAtomicRequest(1),
            "Task256 atomic formula:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongAtomicRequest(2),
            "Task256 atomic formula:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongAtomicRequest(3),
            "Task256 atomic formula:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::RemoveFunctorGuard,
            "Task260 functor definition:",
        ),
        (
            SourceFunctorDefinitionRouteMutation::WrongFunctorDefiniens,
            "Task260 functor definition:",
        ),
    ] {
        let error = source_functor_definition_output_with_mutation(
            &ast,
            module.clone(),
            &shells,
            &symbols,
            SOURCE_FUNCTOR_DEFINITION_TEXT,
            mutation,
        )
        .expect("exact source remains selected")
        .expect_err("lower mutation must fail");
        assert!(error.starts_with(owner), "{mutation:?}: {error}");
    }

    for definition_index in 0..2 {
        for arity in [None, Some(0), Some(2), Some(3)] {
            let mut definitions = mizar_resolve::env::DefinitionIndex::default();
            for original in symbols.definitions().iter() {
                let mut replacement = mizar_resolve::env::DefinitionShell::new(
                    original.symbol().clone(),
                    original.kind(),
                    original.origin().clone(),
                    original.contribution(),
                )
                .with_visibility(original.visibility())
                .with_parameters(original.parameters().to_vec())
                .with_binders(original.binders().to_vec())
                .with_dependencies(original.dependencies().to_vec());
                let replacement_arity = if original.id().index() == definition_index {
                    arity
                } else {
                    original.arity()
                };
                if let Some(replacement_arity) = replacement_arity {
                    replacement = replacement.with_arity(replacement_arity);
                }
                if let Some(notation_shape) = original.notation_shape() {
                    replacement = replacement.with_notation_shape(notation_shape);
                }
                if let Some(doc_attachment) = original.doc_attachment() {
                    replacement = replacement.with_doc_attachment(doc_attachment.clone());
                }
                if let Some(conflict) = original.conflict() {
                    replacement = replacement.with_conflict(conflict.clone());
                }
                if let Some(signature) = original.signature() {
                    replacement = replacement.with_signature(signature.clone());
                }
                assert_eq!(definitions.insert(replacement), original.id());
            }
            let changed = mizar_resolve::env::SymbolEnv::new(
                module.clone(),
                mizar_resolve::env::SymbolEnvIndexes {
                    imports: symbols.imports().clone(),
                    exports: symbols.exports().clone(),
                    symbols: symbols.symbols().clone(),
                    labels: symbols.labels().clone(),
                    definitions,
                    overloads: symbols.overloads().clone(),
                    registrations: symbols.registrations().clone(),
                    lexical_summaries: symbols.lexical_summaries().clone(),
                    namespace_graph: symbols.namespace_graph().clone(),
                    declaration_dependencies: symbols.declaration_dependencies().clone(),
                    contributions: symbols.contributions().clone(),
                    module_summaries: symbols.module_summaries().clone(),
                },
            );
            let error = source_functor_definition_output(
                &ast,
                module.clone(),
                &shells,
                &changed,
                SOURCE_FUNCTOR_DEFINITION_TEXT,
            )
            .expect("exact source remains selected")
            .expect_err("each Task260 functor requires exact resolver arity");
            assert_eq!(
                error,
                format!("Task260 resolver: functor {definition_index} is not exact"),
                "definition {definition_index} arity {arity:?}"
            );
        }
    }

    let first = source_functor_definition_output(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        SOURCE_FUNCTOR_DEFINITION_TEXT,
    )
    .expect("first selector")
    .expect("first route");
    let second = source_functor_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_FUNCTOR_DEFINITION_TEXT,
    )
    .expect("replay selector")
    .expect("replay route");
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
}

#[test]
fn task260_expectation_selection_and_predicate_route_stay_isolated() {
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
    let plan = build_test_plan(&config).expect("Task260 repository plan should build");
    let selected = active_type_elaboration_cases(&plan)
        .filter(|case| {
            std::fs::read_to_string(&case.source_path)
                .is_ok_and(|source| source == SOURCE_FUNCTOR_DEFINITION_TEXT)
        })
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].id.0, TASK260_CASE);
    assert!(selected[0].expectation_path.ends_with(Path::new(
        "tests/miz/pass/types/pass_type_elaboration_functor_definition_payload_001.expect.toml"
    )));

    let (task260_ordinal, task260_case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK260_CASE)
        .expect("Task260 active sidecar");
    assert_eq!(task260_case.expectation.schema_version, 1);
    assert_eq!(task260_case.expectation.id.0, TASK260_CASE);
    assert_eq!(
        task260_case.expectation.kind,
        crate::expectation::TestKind::Pass
    );
    assert_eq!(
        task260_case.expectation.stage,
        crate::staged_model::Stage::TypeElaboration
    );
    assert_eq!(task260_case.expectation.domain, "checker.type_elaboration");
    assert_eq!(
        task260_case.expectation.source,
        Path::new("pass_type_elaboration_functor_definition_payload_001.miz")
    );
    assert_eq!(
        task260_case.expectation.expected_outcome,
        crate::expectation::ExpectedOutcome::Pass
    );
    assert_eq!(
        task260_case.expectation.expected_phase,
        Some(crate::expectation::PipelinePhase::TypeCheck)
    );
    assert_eq!(
        task260_case
            .expectation
            .spec_refs
            .iter()
            .map(|reference| reference.0.as_str())
            .collect::<Vec<_>>(),
        [TASK260_SPEC_REF]
    );
    assert_eq!(task260_case.expectation.tags, ["active_type_elaboration"]);
    assert_eq!(task260_case.expectation.profiles, ["fast"]);
    assert!(task260_case.expectation.diagnostic_codes.is_empty());
    assert!(task260_case.expectation.diagnostic_payloads.is_empty());
    assert!(task260_case.expectation.failure_category.is_none());
    assert!(task260_case.expectation.rejection_reason.is_none());
    assert!(task260_case.expectation.stable_detail_key.is_none());
    assert!(task260_case.expectation.snapshots.is_none());
    assert!(task260_case.expectation.tokens.is_empty());
    let requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| requirement.id.0 == TASK260_SPEC_REF)
        .expect("Task260 trace row");
    assert_eq!(
        requirement.source,
        Path::new("doc/design/mizar-checker/en/source_functor_definition.md")
    );
    assert_eq!(requirement.section, "Dedicated Consumer And Trace Intent");
    assert_eq!(
        requirement.stage,
        crate::staged_model::Stage::TypeElaboration
    );
    assert_eq!(
        requirement.status,
        crate::traceability::RequirementStatus::Covered
    );
    assert_eq!(
        requirement.coverage,
        crate::traceability::CoverageShape::Pass
    );
    assert!(requirement.required);
    assert!(!requirement.built_in);
    assert!(requirement.depends_on.is_empty());
    assert!(requirement.deferred_reason.is_none());
    assert_eq!(
        requirement.tests,
        [Path::new(
            "tests/miz/pass/types/pass_type_elaboration_functor_definition_payload_001.expect.toml"
        )]
    );
    assert_eq!(
        plan.cases
            .iter()
            .filter(|case| case
                .expectation
                .spec_refs
                .iter()
                .any(|reference| reference.0 == TASK260_SPEC_REF))
            .map(|case| case.id.0.as_str())
            .collect::<Vec<_>>(),
        [TASK260_CASE]
    );
    let task260_result = run_type_elaboration_case(
        &workspace_root,
        &workspace_root.join("tests"),
        task260_case,
        task260_ordinal,
    );
    assert_eq!(task260_result.status, TypeElaborationCaseStatus::Passed);
    assert!(task260_result.actual_detail_keys.is_empty());
    let frontend = run_frontend(&workspace_root, task260_case, task260_ordinal).unwrap();
    assert!(frontend.diagnostics.is_empty());
    let mut variants = Vec::new();
    for (kind, edits) in metamorphic_edits::variants(frontend.ast.as_ref().unwrap(), SOURCE_FUNCTOR_DEFINITION_TEXT)
        .into_iter().filter(|(kind, _)| matches!(*kind, "leading_line" | "intertoken_layout"))
    {
        let source = metamorphic_edits::apply(SOURCE_FUNCTOR_DEFINITION_TEXT, &edits);
        let result = metamorphic_guard::with_source(task260_case, &source, || {
            run_type_elaboration_case(&workspace_root, &workspace_root.join("tests"), task260_case, task260_ordinal)
        });
        variants.push((kind, result));
    }
    assert_eq!(variants.iter().map(|(kind, _)| *kind).collect::<Vec<_>>(), ["leading_line", "intertoken_layout"]);
    for (kind, result) in variants {
        assert_eq!(result.status, TypeElaborationCaseStatus::Passed, "{kind}: {:?}", result.actual_detail_keys);
        assert!(result.actual_detail_keys.is_empty(), "{kind}");
    }

    let (functor_ast, functor_module, functor_shells, functor_symbols) =
        task253_ast_from_source_text(SOURCE_FUNCTOR_DEFINITION_TEXT, 260_090);
    assert!(
        super::type_elaboration::source_predicate_definition_output(
            &functor_ast,
            functor_module,
            &functor_shells,
            &functor_symbols,
            SOURCE_FUNCTOR_DEFINITION_TEXT,
        )
        .is_none()
    );
    let (predicate_ast, predicate_module, predicate_shells, predicate_symbols) =
        task253_ast_from_source_text(
            super::type_elaboration::SOURCE_PREDICATE_DEFINITION_TEXT,
            260_091,
        );
    assert!(
        source_functor_definition_output(
            &predicate_ast,
            predicate_module,
            &predicate_shells,
            &predicate_symbols,
            super::type_elaboration::SOURCE_PREDICATE_DEFINITION_TEXT,
        )
        .is_none()
    );

    let (mixed_ordinal, mixed_case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK260_FUNCTOR_MIXED_CASE)
        .expect("Task260 mixed boundary remains active");
    let mixed_source =
        std::fs::read_to_string(&mixed_case.source_path).expect("Task260 mixed-boundary source");
    let (mixed_ast, mixed_module, mixed_shells, mixed_symbols) =
        task253_ast_from_source_text(&mixed_source, 260_092);
    assert!(
        source_functor_definition_output(
            &mixed_ast,
            mixed_module,
            &mixed_shells,
            &mixed_symbols,
            &mixed_source,
        )
        .is_none(),
        "mixed predicate/functor source must not enter the Task260 selector"
    );
    let mixed_result = run_type_elaboration_case(
        &workspace_root,
        &workspace_root.join("tests"),
        mixed_case,
        mixed_ordinal,
    );
    assert_eq!(mixed_result.status, TypeElaborationCaseStatus::Passed);
    assert_eq!(
        mixed_result.actual_detail_keys,
        mixed_case.expectation.diagnostic_payloads
    );
}

#[test]
fn task260_route_publishes_no_proof_fact_acceptance_or_vc() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate should live below the workspace root")
        .to_path_buf();
    let plan = build_test_plan(&DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    })
    .expect("Task260 repository count plan");
    let has_active_type_tag = |case: &&crate::harness::TestCase| {
        case.expectation
            .tags
            .iter()
            .any(|tag| tag == "active_type_elaboration")
    };
    let active_type_consumer_counts = [
        active_type_elaboration_cases(&plan).count(),
        plan.cases.iter().filter(has_active_type_tag).count(),
        plan.cases
            .iter()
            .filter(has_active_type_tag)
            .filter(|case| case.expectation.stage == crate::staged_model::Stage::TypeElaboration)
            .count(),
        plan.cases
            .iter()
            .filter(has_active_type_tag)
            .filter(|case| {
                case.expectation.expected_phase
                    == Some(crate::expectation::PipelinePhase::TypeCheck)
            })
            .count(),
        plan.cases
            .iter()
            .filter(has_active_type_tag)
            .filter(|case| {
                matches!(
                    case.expectation.expected_outcome,
                    crate::expectation::ExpectedOutcome::Pass
                        | crate::expectation::ExpectedOutcome::Fail
                )
            })
            .count(),
        plan.cases
            .iter()
            .filter(has_active_type_tag)
            .filter(|case| case.source_path.extension().is_some_and(|ext| ext == "miz"))
            .count(),
    ];
    assert_eq!(active_type_consumer_counts, [266, 266, 266, 253, 266, 266]);
    assert_eq!(
        (plan.cases.len(), plan.manifest.requirements.len()),
        (620, 541)
    );
    assert_eq!(
        plan.cases.iter().fold((0, 0), |(pass, fail), case| {
            match case.expectation.expected_outcome {
                crate::expectation::ExpectedOutcome::Pass => (pass + 1, fail),
                crate::expectation::ExpectedOutcome::Fail => (pass, fail + 1),
                _ => (pass, fail),
            }
        }),
        (346, 274)
    );
    assert_eq!(
        (
            crate::active_parse_only_cases(&plan).count(),
            crate::active_declaration_symbol_cases(&plan).count(),
            active_type_elaboration_cases(&plan).count(),
            crate::active_proof_verification_cases(&plan).count(),
        ),
        (172, 13, 266, 23)
    );
    let type_stage = plan
        .coverage_report
        .stages
        .iter()
        .find(|stage| stage.stage == crate::staged_model::Stage::TypeElaboration)
        .expect("Task260 type-elaboration coverage stage");
    assert_eq!((type_stage.requirements, type_stage.covered), (307, 295));
    assert_eq!((plan.warning_count(), plan.error_count()), (23, 0));

    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_FUNCTOR_DEFINITION_TEXT, 260_100);
    let symbols_before = symbols.clone();
    let first = source_functor_definition_output(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        SOURCE_FUNCTOR_DEFINITION_TEXT,
    )
    .expect("Task260 selector")
    .expect("Task260 route");
    let second = source_functor_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_FUNCTOR_DEFINITION_TEXT,
    )
    .expect("Task260 replay selector")
    .expect("Task260 replay route");
    let obligations = first
        .typed_ast
        .initial_obligations()
        .iter()
        .collect::<Vec<_>>();
    let [(existence_id, existence), (uniqueness_id, uniqueness)] = obligations.as_slice() else {
        panic!("Task260 must publish exactly two pending obligations");
    };
    assert_eq!((existence_id.index(), uniqueness_id.index()), (0, 1));
    assert_eq!((existence.id.index(), uniqueness.id.index()), (0, 1));
    assert_eq!(
        (
            existence.kind,
            uniqueness.kind,
            existence.status,
            uniqueness.status,
        ),
        (
            mizar_checker::typed_ast::InitialObligationKind::FunctorExistence,
            mizar_checker::typed_ast::InitialObligationKind::FunctorUniqueness,
            mizar_checker::typed_ast::InitialObligationStatus::Pending,
            mizar_checker::typed_ast::InitialObligationStatus::Pending,
        )
    );
    assert!(existence.assumptions.is_empty());
    assert!(uniqueness.assumptions.is_empty());
    assert_eq!(
        &existence.owner,
        &mizar_checker::typed_ast::TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(
            99
        ))
    );
    assert_eq!(
        &uniqueness.owner,
        &mizar_checker::typed_ast::TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(
            103
        ))
    );
    assert_eq!(
        (existence.source_range, uniqueness.source_range),
        (
            task260_range(ast.source_id, 182, 217),
            task260_range(ast.source_id, 220, 256),
        )
    );
    assert_eq!(
        existence.goal.as_str(),
        "source.definition.functor.correctness:definition=1:existence"
    );
    assert_eq!(
        uniqueness.goal.as_str(),
        "source.definition.functor.correctness:definition=1:uniqueness"
    );
    assert_eq!(
        existence.provenance.as_str(),
        "source.definition.functor:definition=1:correctness=0"
    );
    assert_eq!(
        uniqueness.provenance.as_str(),
        "source.definition.functor:definition=1:correctness=1"
    );
    for (index, expected_children) in [
        (96, &[46, 47, 48][..]),
        (97, &[44, 45, 96, 49][..]),
        (98, &[43, 97][..]),
        (99, &[42, 98, 50][..]),
        (100, &[55, 56, 57][..]),
        (101, &[53, 54, 100, 58][..]),
        (102, &[52, 101][..]),
        (103, &[51, 102, 59][..]),
    ] {
        let surface = &ast.nodes()[index];
        assert_eq!(
            surface
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            expected_children,
            "surface computation/correctness subtree row {index}"
        );
        let typed = first
            .typed_ast
            .nodes()
            .node(mizar_checker::typed_ast::TypedNodeId::new(index))
            .expect("Task260 preserved typed subtree row");
        assert_eq!(
            typed
                .children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            expected_children,
            "typed computation/correctness subtree row {index}"
        );
        assert_eq!(
            typed.anchor,
            mizar_session::SourceAnchor::Range(surface.range)
        );
        assert_eq!(typed.typing, mizar_checker::typed_ast::TypingState::Unknown);
        assert_eq!(
            typed.recovery,
            mizar_checker::typed_ast::NodeRecoveryState::Normal
        );
        assert!(typed.links.facts.is_empty());
        assert!(typed.links.coercions.is_empty());
        assert!(typed.links.initial_obligations.is_empty());
        assert!(typed.links.diagnostics.is_empty());

        let (resolved_id, resolved) = first
            .resolved
            .nodes()
            .iter()
            .find(|(_, row)| row.typed_node.index() == index)
            .expect("Task260 preserved resolved subtree row");
        assert_eq!(resolved_id.index(), index);
        assert_eq!(resolved.source_range, surface.range);
        assert!(matches!(
            &resolved.kind,
            mizar_checker::resolved_typed_ast::ResolvedTypedNodeKind::Degraded {
                reason: mizar_checker::resolved_typed_ast::ResolvedNodeRecoveryReason::TypingState(
                    mizar_checker::typed_ast::TypingState::Unknown
                )
            }
        ));
        assert!(resolved.final_type.is_none());
        assert!(resolved.metadata.is_none());
        assert!(resolved.diagnostics.is_empty());
        assert_eq!(
            resolved.recovery,
            mizar_checker::resolved_typed_ast::ResolvedNodeRecovery::Normal
        );
    }
    assert_eq!(
        symbols, symbols_before,
        "Task260 must not activate resolver data"
    );
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.coercions().is_empty());
    assert!(first.typed_ast.diagnostics().is_empty());
    assert!(first.typed_ast.source_attribute().is_none());
    assert!(first.typed_ast.source_evidence().is_none());
    assert!(first.typed_ast.source_application().is_none());
    assert!(first.typed_ast.source_structure().is_none());
    assert!(first.typed_ast.source_set_term().is_none());
    assert!(first.typed_ast.source_predicate_definition().is_none());
    assert!(first.typed_ast.source_composite_formula().is_none());
    assert!(first.typed_ast.source_formula_composition().is_none());
    assert!(
        first
            .typed_ast
            .source_condition_formula_composition()
            .is_none()
    );
    assert!(
        first
            .typed_ast
            .source_predicate_chain_composition()
            .is_none()
    );
    assert!(first.typed_ast.source_statement().is_none());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.expr_metadata().is_empty());
    assert!(first.resolved.collection_candidates().is_empty());
    assert!(first.resolved.expanded_candidates().is_empty());
    assert!(first.resolved.template_expansions().is_empty());
    assert!(first.resolved.viable_candidates().is_empty());
    assert!(first.resolved.viability_decisions().is_empty());
    assert!(first.resolved.specificity_graphs().is_empty());
    assert!(first.resolved.resolved_overloads().is_empty());
    assert!(first.resolved.inserted_coercions().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
    assert!(first.resolved.checked_proofs().is_empty());
    assert!(first.resolved.checked_proof_nodes().is_empty());
    assert!(first.resolved.checked_terminal_goals().is_empty());
    assert!(first.resolved.cluster_facts().is_empty());
    assert!(first.resolved.diagnostics().is_empty());
    assert!(first.resolved.source_predicate_definition().is_none());
    assert!(first.resolved.source_composite_formula().is_none());
    assert!(first.resolved.source_formula_composition().is_none());
    assert!(
        first
            .resolved
            .source_condition_formula_composition()
            .is_none()
    );
    assert!(
        first
            .resolved
            .source_predicate_chain_composition()
            .is_none()
    );
    assert!(first.resolved.source_statement().is_none());
    assert_eq!(
        first.typed_ast.source_functor_definition(),
        first.resolved.source_functor_definition()
    );
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    let debug = format!(
        "{}{}",
        first.typed_ast.debug_text(),
        first.resolved.debug_text()
    );
    for forbidden in [
        "accepted",
        "activated",
        "core-ir",
        "control-flow",
        "verification-condition",
    ] {
        assert!(
            !debug.to_ascii_lowercase().contains(forbidden),
            "Task260 leaked deferred semantic marker {forbidden}"
        );
    }
}

#[test]
fn task260_core_item_context_association_is_exact_and_deterministic() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_FUNCTOR_DEFINITION_TEXT, 260_200);
    let output = source_functor_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_FUNCTOR_DEFINITION_TEXT,
    )
    .expect("Task260 selector")
    .expect("Task260 route");
    let source_context = output
        .typed_ast
        .source_context()
        .expect("Task248 source context")
        .clone();
    let checker_owner = output
        .typed_ast
        .source_functor_definition()
        .expect("Task260 checker owner")
        .clone();
    let source_bindings = task260_source_binding_core_handoff(&source_context, &checker_owner);
    let expected_source_bindings = source_bindings.clone();
    let first = mizar_core::elaborator::SourceFunctorCoreContextProducer::build(
        source_bindings.clone(),
        source_context.clone(),
        checker_owner.clone(),
    )
    .expect("Task260 Core item context");
    let second = mizar_core::elaborator::SourceFunctorCoreContextProducer::build(
        source_bindings,
        source_context.clone(),
        checker_owner.clone(),
    )
    .expect("Task260 deterministic replay");
    assert_eq!(first, second);
    assert_eq!(first.source_id(), source_context.source_id());
    assert_eq!(first.module_id(), source_context.module_id());
    assert_eq!(first.source_bindings(), &expected_source_bindings);
    assert_eq!(
        first.source_bindings().binding_env(),
        source_context.binding_env()
    );
    assert_eq!(first.source_context(), &source_context);
    assert_eq!(first.checker_owner(), &checker_owner);
    assert_eq!(first.items().len(), 2);
    assert!(!first.items().is_empty());

    let source_item = mizar_checker::source_context::SourceItemId::new(0);
    let definitions = checker_owner.definitions().iter().collect::<Vec<_>>();
    let associations = first
        .items()
        .iter()
        .map(|(id, row)| (id, row.source_item(), row.symbol().clone(), row.core_item()))
        .collect::<Vec<_>>();
    assert_eq!(associations.len(), 2);
    assert_ne!(associations[0].3, associations[1].3);
    for (index, (definition_id, definition)) in definitions.iter().enumerate() {
        let association = first
            .items()
            .get(*definition_id)
            .expect("Task260 association");
        assert_eq!(association.definition(), *definition_id);
        assert_eq!(association.source_item(), source_item);
        assert_eq!(association.symbol(), definition.symbol());
        let core_item = first
            .context()
            .item_registry()
            .id_for_symbol(definition.symbol())
            .expect("Core functor item");
        assert_eq!(association.core_item(), core_item);
        assert_eq!(associations[index].0, *definition_id);
        assert_eq!(associations[index].1, source_item);
        assert_eq!(associations[index].2, *definition.symbol());
        assert_eq!(associations[index].3, core_item);
    }
    assert_eq!(first.context().item_registry().items().len(), 2);
    assert!(first.context().dependency_summaries().is_empty());
    assert!(first.context().generated_origins().table().is_empty());
    assert!(first.context().diagnostics().is_empty());

    for (index, (definition_id, definition)) in definitions.iter().enumerate() {
        let core_item = first
            .items()
            .get(*definition_id)
            .expect("Task260 association")
            .core_item();
        let item = first
            .context()
            .item_registry()
            .items()
            .get(core_item)
            .expect("Core functor row");
        assert_eq!(item.symbol, *definition.symbol());
        assert_eq!(item.kind, mizar_core::core_ir::CoreItemKind::Functor);
        assert_eq!(item.visibility.as_str(), "public");
        assert_eq!(item.status, mizar_core::core_ir::CoreItemStatus::Valid);
        assert!(item.dependencies.is_empty());
        assert!(item.diagnostics.is_empty());
        assert_eq!(
            item.source.anchor,
            mizar_core::core_ir::CoreSourceAnchor::SourceRange(definition.source_range())
        );
        assert_eq!(
            item.source.provenance,
            vec![mizar_core::core_ir::CoreProvenance::new(
                mizar_core::core_ir::CoreProvenancePhase::Checker,
                format!("source-functor-core-item-v1.definition.{index}"),
            )]
        );
        let boundary = first
            .context()
            .definition_boundaries()
            .get_by_item(core_item)
            .expect("pending definition boundary");
        assert_eq!(
            boundary.kind,
            mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem
        );
        assert_eq!(
            boundary.status,
            mizar_core::elaborator::DefinitionBoundaryStatus::PendingBody
        );
        assert_eq!(boundary.item, core_item);
        assert_eq!(boundary.symbol, *definition.symbol());
        assert_eq!(boundary.source, item.source);
        assert_eq!(
            boundary.provenance.as_slice(),
            item.source.provenance.as_slice()
        );
    }

    let source_map = first.context().source_map();
    assert_eq!(source_map.item_sources.len(), 2);
    for (definition_id, definition) in &definitions {
        let core_item = first
            .items()
            .get(*definition_id)
            .expect("Task260 association")
            .core_item();
        let item = first
            .context()
            .item_registry()
            .items()
            .get(core_item)
            .expect("Core functor row");
        assert_eq!(source_map.item_sources.get(&core_item), Some(&item.source));
        assert_eq!(
            item.source.anchor,
            mizar_core::core_ir::CoreSourceAnchor::SourceRange(definition.source_range())
        );
    }
    assert!(source_map.term_sources.is_empty());
    assert!(source_map.formula_sources.is_empty());
    assert!(source_map.definition_sources.is_empty());
    assert!(source_map.proof_sources.is_empty());
    assert!(source_map.algorithm_sources.is_empty());
    assert!(source_map.generated_sources.is_empty());
    assert!(source_map.obligation_sources.is_empty());
    let expected_worklist = definitions
        .iter()
        .map(|(definition_id, definition)| {
            let core_item = first
                .items()
                .get(*definition_id)
                .expect("Task260 association")
                .core_item();
            let source = mizar_core::core_ir::CoreSourceRef::direct(definition.source_range())
                .with_provenance(vec![mizar_core::core_ir::CoreProvenance::new(
                    mizar_core::core_ir::CoreProvenancePhase::Checker,
                    format!(
                        "source-functor-core-item-v1.definition.{}",
                        definition_id.index()
                    ),
                )]);
            mizar_core::elaborator::ElaborationWorkItem {
                kind: mizar_core::elaborator::ElaborationWorkItemKind::Item(core_item),
                status: mizar_core::elaborator::ElaborationWorkStatus::Pending,
                source,
                diagnostics: Vec::new(),
                checker_diagnostics: Vec::new(),
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(first.context().worklist().entries(), expected_worklist);
}

#[test]
fn task260_core_item_context_transports_actual_prefix_geometry() {
    let intertoken = SOURCE_FUNCTOR_DEFINITION_TEXT
        .replacen("definition", "definition :: metamorphic\n", 1);
    let cases = [
        ("leading_line", format!("\n{SOURCE_FUNCTOR_DEFINITION_TEXT}"), 1, 1),
        ("intertoken", intertoken.clone(), 0, 16),
        ("mixed_prefix_intertoken", format!(" \t\n{intertoken}"), 3, 19),
    ];
    let mut outcomes = Vec::new();
    for (ordinal, (case, source, m, d)) in cases.into_iter().enumerate() {
        assert_eq!(source.len(), 262 + d, "{case}");
        let (ast, module, shells, symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&source, 260_220 + ordinal);
        assert_eq!(diagnostics, 0, "{case} native Frontend diagnostics");
        assert_eq!(ast.nodes().len(), 108, "{case} genuine source layout");
        let output = source_functor_definition_output(&ast, module, &shells, &symbols, &source)
            .expect("fresh Task260 selector")
            .expect("fresh public checker producer chain");
        let source_context = output
            .typed_ast
            .source_context()
            .expect("actual Task248 Context")
            .clone();
        let checker_owner = output
            .typed_ast
            .source_functor_definition()
            .expect("actual Task260 upper owner")
            .clone();
        let source_bindings = task260_source_binding_core_handoff(&source_context, &checker_owner);
        let result = mizar_core::elaborator::SourceFunctorCoreContextProducer::build(
            source_bindings.clone(),
            source_context.clone(),
            checker_owner.clone(),
        );
        outcomes.push((case, m, d, source_context, checker_owner, source_bindings, result));
    }

    let failures = outcomes
        .iter()
        .filter_map(|(case, _, _, _, _, _, result)| {
            result.as_ref().err().map(|error| format!("{case}: {error:?}"))
        })
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "actual public Core results: {failures:?}");

    for (case, m, d, source_context, checker_owner, source_bindings, result) in outcomes {
        let core = result.expect("collected public Core success");
        assert_eq!((core.source_context(), core.checker_owner(), core.source_bindings()),
            (&source_context, &checker_owner, &source_bindings), "{case} retained inputs");
        assert_eq!(source_context.items().get(mizar_checker::source_context::SourceItemId::new(0)).unwrap().source_range,
            task260_range(source_context.source_id(), m, d + 261), "{case} outer item range");
        let definitions = checker_owner.definitions().iter().collect::<Vec<_>>();
        assert_eq!(definitions.len(), 2, "{case}");
        for (index, (_, definition)) in definitions.iter().enumerate() {
            let (start, end) = [(61 + d, 118 + d), (121 + d, 179 + d)][index];
            assert_eq!(
                definition.source_range(),
                task260_range(source_context.source_id(), start, end),
                "{case} inner definition {index}"
            );
        }

        let replay = mizar_core::elaborator::SourceFunctorCoreContextProducer::build(
            source_bindings.clone(),
            source_context.clone(),
            checker_owner.clone(),
        )
        .expect("fresh public Core replay");
        assert_eq!(core, replay, "{case} deterministic replay");

        let associations = core
            .items()
            .iter()
            .map(|(_, row)| (row.symbol(), row.core_item()))
            .collect::<Vec<_>>();
        assert_eq!(associations.len(), 2, "{case}");
        assert_ne!(associations[0].0, associations[1].0, "{case} whole SymbolIds");
        assert_ne!(associations[0].1, associations[1].1, "{case} Core item IDs");
        for (index, (definition_id, definition)) in definitions.iter().enumerate() {
            let association = core.items().get(*definition_id).expect("whole-symbol association");
            assert_eq!(association.source_item(), mizar_checker::source_context::SourceItemId::new(0), "{case}");
            assert_eq!(association.symbol(), definition.symbol(), "{case} full SymbolId join");
            let item_id = core
                .context()
                .item_registry()
                .id_for_symbol(definition.symbol())
                .expect("joined Core item");
            assert_eq!(association.core_item(), item_id, "{case}");
            let item = core.context().item_registry().items().get(item_id).expect("Core item");
            let expected_range = task260_range(source_context.source_id(), definition.source_range().start, definition.source_range().end);
            assert_eq!(item.symbol, *definition.symbol(), "{case}");
            assert_eq!(item.source.anchor, mizar_core::core_ir::CoreSourceAnchor::SourceRange(expected_range), "{case}");
            let provenance = vec![mizar_core::core_ir::CoreProvenance::new(
                mizar_core::core_ir::CoreProvenancePhase::Checker,
                format!("source-functor-core-item-v1.definition.{index}"),
            )];
            assert_eq!(item.source.provenance, provenance, "{case}");
            let boundary = core.context().definition_boundaries().get_by_item(item_id).expect("pending boundary");
            assert_eq!(boundary.kind, mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem, "{case}");
            assert_eq!(boundary.status, mizar_core::elaborator::DefinitionBoundaryStatus::PendingBody, "{case}");
            assert_eq!(boundary.source, item.source, "{case}");
            assert_eq!(boundary.provenance.as_slice(), provenance.as_slice(), "{case}");
        }

        let context = core.context();
        assert_eq!(context.item_registry().items().len(), 2, "{case}");
        assert!(context.dependency_summaries().is_empty() && context.generated_origins().table().is_empty()
            && context.diagnostics().is_empty(), "{case} empty Core semantics");
        let source_map = context.source_map();
        assert_eq!(source_map.item_sources.len(), 2, "{case}");
        for (definition_id, _) in &definitions {
            let item_id = core.items().get(*definition_id).unwrap().core_item();
            let item = context.item_registry().items().get(item_id).unwrap();
            assert_eq!(source_map.item_sources.get(&item_id), Some(&item.source), "{case}");
        }
        assert!(source_map.term_sources.is_empty() && source_map.formula_sources.is_empty()
            && source_map.definition_sources.is_empty() && source_map.proof_sources.is_empty()
            && source_map.algorithm_sources.is_empty() && source_map.generated_sources.is_empty()
            && source_map.obligation_sources.is_empty(), "{case} empty semantic maps and obligations");
        let expected_worklist = definitions.iter().map(|(definition_id, definition)| {
            mizar_core::elaborator::ElaborationWorkItem {
                kind: mizar_core::elaborator::ElaborationWorkItemKind::Item(core.items().get(*definition_id).unwrap().core_item()),
                status: mizar_core::elaborator::ElaborationWorkStatus::Pending,
                source: mizar_core::core_ir::CoreSourceRef::direct(definition.source_range()).with_provenance(vec![
                    mizar_core::core_ir::CoreProvenance::new(mizar_core::core_ir::CoreProvenancePhase::Checker,
                        format!("source-functor-core-item-v1.definition.{}", definition_id.index()))
                ]),
                diagnostics: Vec::new(), checker_diagnostics: Vec::new(),
            }
        }).collect::<Vec<_>>();
        assert_eq!(context.worklist().entries(), expected_worklist, "{case}");

        if d == 19 {
            for mutation in [Task260CoreContextMutation::WrongSource(0), Task260CoreContextMutation::WrongProvenance(1)] {
                let stale_bindings = task260_source_binding_core_handoff_with_mutation(
                    &source_context, &checker_owner, mutation,
                );
                assert_eq!(
                    mizar_core::elaborator::SourceFunctorCoreContextProducer::build(
                        stale_bindings, source_context.clone(), checker_owner.clone(),
                    ),
                    Err(mizar_core::elaborator::SourceFunctorCoreContextError::InvalidCoreContext),
                    "{case} shifted stale Core handoff {mutation:?}"
                );
            }

            let stale_context_input = mizar_checker::source_context::SourceBindingContextInput {
                source_id: source_context.source_id(),
                module_id: source_context.module_id().clone(),
                module_site: source_context.local_contexts().get(mizar_checker::typed_ast::LocalTypeContextId::new(0))
                    .expect("module local context").owner.clone(),
                items: source_context.items().iter().map(|(_, item)| mizar_checker::source_context::SourceItemInput {
                        shell: item.shell,
                        shell_ordinal: item.shell_ordinal,
                        role: item.role,
                        module_id: source_context.module_id().clone(),
                        source_range: task260_range(source_context.source_id(), 0, 261),
                        parent: item.parent,
                        visibility: item.visibility,
                        site: item.site.clone(),
                        local_scope: item.local_scope.clone(),
                        recovery: item.recovery,
                    })
                    .collect(),
                bindings: source_context
                    .declarations()
                    .iter()
                    .map(|(_, declaration)| {
                        let item = source_context.items().get(declaration.item).expect("binding item");
                        let binding = source_context.binding_env().bindings().get(declaration.binding).expect("binding row");
                        mizar_checker::source_context::SourceBindingSiteInput {
                            shell: item.shell,
                            context_owner: mizar_checker::source_context::SourceBindingContextOwner::Shell(item.shell),
                            source_ordinal: declaration.source_ordinal,
                            spelling: declaration.spelling.clone(),
                            declaration_range: declaration.declaration_range,
                            written_type_range: declaration.written_type_range,
                            site: declaration.site.clone(),
                            role: declaration.role.clone(),
                            recovery: binding.recovery,
                        }
                    })
                    .collect(),
            };
            let stale_context = mizar_checker::source_context::SourceBindingContextProducer::build(stale_context_input)
                .expect("fresh public stale-context producer")
                .into_complete()
                .expect("complete stale context")
                .into_handoff();
            assert_eq!((stale_context.source_id(), stale_context.module_id(), stale_context.binding_env()),
                (source_context.source_id(), source_context.module_id(), source_context.binding_env()),
                "{case} same-source/module shifted BindingEnv");
            assert_ne!(stale_context, source_context, "{case} stale Context geometry");
            let stale_context_bindings = task260_source_binding_core_handoff(&stale_context, &checker_owner);
            assert_eq!(
                mizar_core::elaborator::SourceFunctorCoreContextProducer::build(
                    stale_context_bindings, stale_context, checker_owner.clone(),
                ),
                Err(mizar_core::elaborator::SourceFunctorCoreContextError::InvalidCheckerOwner),
                "{case} same-source stale Context/owner geometry"
            );
        }
    }
}

#[test]
fn task260_core_item_context_default_deny_mutations_and_foreign_environment() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_FUNCTOR_DEFINITION_TEXT, 260_210);
    let output = source_functor_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_FUNCTOR_DEFINITION_TEXT,
    )
    .expect("Task260 selector")
    .expect("Task260 route");
    let source_context = output
        .typed_ast
        .source_context()
        .expect("Task248 source context")
        .clone();
    let checker_owner = output
        .typed_ast
        .source_functor_definition()
        .expect("Task260 checker owner")
        .clone();
    for mutation in [
        Task260CoreContextMutation::MissingItem,
        Task260CoreContextMutation::ExtraItem,
        Task260CoreContextMutation::WrongKind(0),
        Task260CoreContextMutation::WrongVisibility(1),
        Task260CoreContextMutation::WrongSource(0),
        Task260CoreContextMutation::WrongProvenance(1),
        Task260CoreContextMutation::MissingBoundary(0),
        Task260CoreContextMutation::WrongBoundary(1),
        Task260CoreContextMutation::UnexpectedDependency(0),
        Task260CoreContextMutation::InvalidStatus(1),
    ] {
        let source_bindings = task260_source_binding_core_handoff_with_mutation(
            &source_context,
            &checker_owner,
            mutation,
        );
        let error = mizar_core::elaborator::SourceFunctorCoreContextProducer::build(
            source_bindings,
            source_context.clone(),
            checker_owner.clone(),
        )
        .expect_err("Core mutation must fail closed");
        assert_eq!(
            error,
            mizar_core::elaborator::SourceFunctorCoreContextError::InvalidCoreContext,
            "{mutation:?}"
        );
    }

    let (foreign_ast, foreign_module, foreign_shells, foreign_symbols) =
        task253_ast_from_source_text(SOURCE_FUNCTOR_DEFINITION_TEXT, 260_211);
    let foreign_output = source_functor_definition_output(
        &foreign_ast,
        foreign_module,
        &foreign_shells,
        &foreign_symbols,
        SOURCE_FUNCTOR_DEFINITION_TEXT,
    )
    .expect("foreign Task260 selector")
    .expect("foreign Task260 route");
    let foreign_context = foreign_output
        .typed_ast
        .source_context()
        .expect("foreign source context")
        .clone();
    let foreign_owner = foreign_output
        .typed_ast
        .source_functor_definition()
        .expect("foreign checker owner")
        .clone();
    let base_bindings = task260_source_binding_core_handoff(&source_context, &checker_owner);
    let foreign_bindings = task260_source_binding_core_handoff(&foreign_context, &foreign_owner);
    for (bindings, context, owner) in [
        (
            base_bindings.clone(),
            foreign_context.clone(),
            foreign_owner.clone(),
        ),
        (
            base_bindings.clone(),
            foreign_context.clone(),
            checker_owner.clone(),
        ),
        (
            base_bindings.clone(),
            source_context.clone(),
            foreign_owner.clone(),
        ),
        (foreign_bindings, source_context, checker_owner),
    ] {
        let error = mizar_core::elaborator::SourceFunctorCoreContextProducer::build(
            bindings, context, owner,
        )
        .expect_err("foreign environment must fail closed");
        assert_eq!(
            error,
            mizar_core::elaborator::SourceFunctorCoreContextError::EnvironmentMismatch
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Task260CoreContextMutation {
    Baseline,
    MissingItem,
    ExtraItem,
    WrongKind(usize),
    WrongVisibility(usize),
    WrongSource(usize),
    WrongProvenance(usize),
    MissingBoundary(usize),
    WrongBoundary(usize),
    UnexpectedDependency(usize),
    InvalidStatus(usize),
}

fn task260_source_binding_core_handoff(
    source_context: &mizar_checker::source_context::SourceBindingContextHandoff,
    checker_owner: &mizar_checker::source_functor_definition::SourceFunctorDefinitionHandoff,
) -> mizar_core::elaborator::SourceBindingCoreContextHandoff {
    task260_source_binding_core_handoff_with_mutation(
        source_context,
        checker_owner,
        Task260CoreContextMutation::Baseline,
    )
}

fn task260_source_binding_core_handoff_with_mutation(
    source_context: &mizar_checker::source_context::SourceBindingContextHandoff,
    checker_owner: &mizar_checker::source_functor_definition::SourceFunctorDefinitionHandoff,
    mutation: Task260CoreContextMutation,
) -> mizar_core::elaborator::SourceBindingCoreContextHandoff {
    let definitions = checker_owner.definitions().iter().collect::<Vec<_>>();
    let mut input = mizar_core::elaborator::CoreContextInput::new(
        mizar_core::elaborator::ResolvedTypedAstSummary::new(
            source_context.source_id(),
            source_context.module_id().clone(),
        ),
    );
    for (index, (_, definition)) in definitions.iter().enumerate() {
        if matches!(mutation, Task260CoreContextMutation::MissingItem) && index == 0 {
            continue;
        }
        let source_range = if matches!(mutation, Task260CoreContextMutation::WrongSource(i) if i == index)
        {
            mizar_session::SourceRange {
                source_id: source_context.source_id(),
                start: definition.source_range().start + 1,
                end: definition.source_range().end,
            }
        } else {
            definition.source_range()
        };
        let provenance_key = if matches!(mutation, Task260CoreContextMutation::WrongProvenance(i) if i == index)
        {
            format!("wrong-task260-provenance-{index}")
        } else {
            format!("source-functor-core-item-v1.definition.{index}")
        };
        let kind = if matches!(mutation, Task260CoreContextMutation::WrongKind(i) if i == index) {
            mizar_core::core_ir::CoreItemKind::Predicate
        } else {
            mizar_core::core_ir::CoreItemKind::Functor
        };
        let visibility = if matches!(mutation, Task260CoreContextMutation::WrongVisibility(i) if i == index)
        {
            "private"
        } else {
            "public"
        };
        let seed = mizar_core::elaborator::CoreItemSeed::new(
            definition.symbol().clone(),
            kind,
            visibility,
            mizar_core::core_ir::CoreSourceRef::direct(source_range).with_provenance(vec![
                mizar_core::core_ir::CoreProvenance::new(
                    mizar_core::core_ir::CoreProvenancePhase::Checker,
                    provenance_key.clone(),
                ),
            ]),
            mizar_core::elaborator::CheckerOwnedProvenance::checker(provenance_key),
        );
        let seed = if matches!(mutation, Task260CoreContextMutation::MissingBoundary(i) if i == index)
        {
            seed
        } else if matches!(mutation, Task260CoreContextMutation::WrongBoundary(i) if i == index) {
            seed.with_definition_boundary(mizar_core::elaborator::DefinitionBoundaryKind::Theorem)
        } else {
            seed.with_definition_boundary(
                mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem,
            )
        };
        let seed = if matches!(mutation, Task260CoreContextMutation::UnexpectedDependency(i) if i == index)
        {
            seed.with_dependencies(vec![definition.symbol().clone()])
        } else if matches!(mutation, Task260CoreContextMutation::InvalidStatus(i) if i == index) {
            seed.with_dependencies(vec![mizar_resolve::resolved_ast::SymbolId::new(
                definition.symbol().module().clone(),
                mizar_resolve::resolved_ast::LocalSymbolId::new("task260-missing"),
                mizar_resolve::resolved_ast::FullyQualifiedName::new(format!(
                    "{}.task260-missing",
                    definition.symbol().fqn().as_str()
                )),
            )])
        } else {
            seed
        };
        input.item_seeds.push(seed);
    }
    if mutation == Task260CoreContextMutation::ExtraItem {
        let definition = definitions[0].1;
        let extra_symbol = mizar_resolve::resolved_ast::SymbolId::new(
            definition.symbol().module().clone(),
            mizar_resolve::resolved_ast::LocalSymbolId::new("task260-extra"),
            mizar_resolve::resolved_ast::FullyQualifiedName::new(format!(
                "{}.task260-extra",
                definition.symbol().fqn().as_str()
            )),
        );
        input.item_seeds.push(
            mizar_core::elaborator::CoreItemSeed::new(
                extra_symbol,
                mizar_core::core_ir::CoreItemKind::Functor,
                "public",
                mizar_core::core_ir::CoreSourceRef::direct(definition.source_range())
                    .with_provenance(vec![mizar_core::core_ir::CoreProvenance::new(
                        mizar_core::core_ir::CoreProvenancePhase::Checker,
                        "source-functor-core-item-v1.definition.extra",
                    )]),
                mizar_core::elaborator::CheckerOwnedProvenance::checker(
                    "source-functor-core-item-v1.definition.extra",
                ),
            )
            .with_definition_boundary(
                mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem,
            ),
        );
    }
    let context = mizar_core::elaborator::prepare_core_context(input)
        .expect("Task260 Core context seed should prepare");
    mizar_core::elaborator::SourceBindingCoreContextProducer::build(
        context,
        source_context.binding_env().clone(),
    )
    .expect("Task260 33LB handoff should build")
}
