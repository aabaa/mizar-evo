use mizar_syntax::{
    SurfaceAst, SurfaceAstBuilder, SurfaceNodeKind, SurfaceQuantifierKind, SurfaceTokenKind,
};

    #[test]
    fn source_variable_checker_projects_explicit_and_implicit_reserve_types() {
        let source = source_id();
        let module = module_id();
        let ast = source_variable_type_projection_ast(source);
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let scope = SourceVariableScopeResolver::resolve(SourceVariableScopeInput::new(
            &ast, &module, &symbols,
        ))
        .expect("source-variable type projection should resolve");

        let output =
            SourceVariableSemanticsChecker::check(SourceVariableSemanticsInput::new(&scope));
        assert!(output.diagnostics().is_empty());
        assert_eq!(output.binding_types().len(), 4);

        let binding_types = scope
            .bindings()
            .iter()
            .map(|binding| (binding.spelling(), binding.id(), binding.kind()))
            .collect::<Vec<_>>();
        let reserve_x = binding_types
            .iter()
            .find(|(spelling, _, kind)| {
                *spelling == "x" && *kind == SourceVariableBindingKind::Reserve
            })
            .map(|(_, id, _)| *id)
            .expect("reserve x binding should exist");
        let reserve_y = binding_types
            .iter()
            .find(|(spelling, _, kind)| {
                *spelling == "y" && *kind == SourceVariableBindingKind::Reserve
            })
            .map(|(_, id, _)| *id)
            .expect("reserve y binding should exist");
        let explicit_x = binding_types
            .iter()
            .find(|(spelling, _, kind)| *spelling == "x" && *kind == SourceVariableBindingKind::Let)
            .map(|(_, id, _)| *id)
            .expect("explicit x binding should exist");
        let implicit_y = binding_types
            .iter()
            .find(|(spelling, _, kind)| *spelling == "y" && *kind == SourceVariableBindingKind::Let)
            .map(|(_, id, _)| *id)
            .expect("implicit y binding should exist");

        assert_eq!(
            output.binding_types()[&reserve_x].radix(),
            SourceVariableTypeRadix::Set
        );
        assert_eq!(
            output.binding_types()[&explicit_x].radix(),
            SourceVariableTypeRadix::Object,
            "an explicit local type must override the reservation"
        );
        assert_eq!(
            output.binding_types()[&reserve_y].radix(),
            SourceVariableTypeRadix::Object
        );
        assert_eq!(
            output.binding_types()[&implicit_y].radix(),
            SourceVariableTypeRadix::Object,
            "an implicit local type must inherit its prior reservation"
        );
    }

    #[test]
    fn source_variable_checker_tracks_aliases_assumptions_and_existential_take() {
        let source = source_id();
        let module = module_id();
        let ast = source_variable_alias_take_ast(source);
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let scope = SourceVariableScopeResolver::resolve(SourceVariableScopeInput::new(
            &ast, &module, &symbols,
        ))
        .expect("source-variable alias/take fixture should resolve");

        let output =
            SourceVariableSemanticsChecker::check(SourceVariableSemanticsInput::new(&scope));
        assert!(output.diagnostics().is_empty());

        let set_binding = scope
            .bindings()
            .iter()
            .find(|binding| {
                binding.kind() == SourceVariableBindingKind::Set && binding.spelling() == "a"
            })
            .expect("set alias binding should exist")
            .id();
        let reconsider_binding = scope
            .bindings()
            .iter()
            .find(|binding| {
                binding.kind() == SourceVariableBindingKind::Reconsider
                    && binding.spelling() == "b"
            })
            .expect("reconsidered binding should exist")
            .id();
        assert_eq!(
            output.binding_types()[&set_binding].radix(),
            SourceVariableTypeRadix::Object
        );
        assert_eq!(
            output.binding_types()[&reconsider_binding].radix(),
            SourceVariableTypeRadix::Set,
            "the fact about x must discharge narrowing of its alias a"
        );
        assert_eq!(output.assumptions().len(), 2);
        assert!(matches!(
            output.assumptions()[0],
            SourceVariableFormula::Equality { .. }
        ));
        assert!(matches!(
            output.assumptions()[1],
            SourceVariableFormula::TypeAssertion { .. }
        ));
        assert!(matches!(
            output.thesis(),
            Some(SourceVariableFormula::Equality { .. })
        ));

        let take = scope
            .statements()
            .iter()
            .find(|statement| matches!(statement, SourceVariableStatement::Take { .. }));
        assert!(
            take.is_some(),
            "fixture should include the existential take"
        );
    }

    #[test]
    fn source_variable_checker_keeps_inline_capture_and_shadowing_identity() {
        let source = source_id();
        let module = module_id();
        let ast = source_variable_capture_shadow_ast(source, false);
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let scope = SourceVariableScopeResolver::resolve(SourceVariableScopeInput::new(
            &ast, &module, &symbols,
        ))
        .expect("captured inline functor fixture should resolve");

        let functor = scope
            .bindings()
            .iter()
            .find(|binding| binding.kind() == SourceVariableBindingKind::InlineFunctor)
            .expect("inline functor binding should exist");
        let reserve = scope
            .bindings()
            .iter()
            .find(|binding| {
                binding.kind() == SourceVariableBindingKind::Reserve && binding.spelling() == "x"
            })
            .expect("outer reserve binding should exist");
        let local_x = scope
            .bindings()
            .iter()
            .find(|binding| {
                binding.kind() == SourceVariableBindingKind::Let && binding.spelling() == "x"
            })
            .expect("shadowing local binding should exist");
        assert_eq!(functor.captures(), &[reserve.id()]);
        assert_ne!(reserve.id(), local_x.id());

        let output =
            SourceVariableSemanticsChecker::check(SourceVariableSemanticsInput::new(&scope));
        assert!(
            output.diagnostics().is_empty(),
            "F(local x) and the pre-shadow alias must both reduce to captured outer x"
        );
        assert!(
            output.assumptions().len() == 1,
            "the type assertion is retained before the identity mismatch"
        );
    }

    #[test]
    fn source_variable_checker_cycle_guard_fails_closed_for_recursive_functor() {
        let source = source_id();
        let module = module_id();
        let ast = source_variable_capture_shadow_ast(source, true);
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let scope = SourceVariableScopeResolver::resolve(SourceVariableScopeInput::new(
            &ast, &module, &symbols,
        ))
        .expect("recursive inline functor fixture should resolve");

        let output =
            SourceVariableSemanticsChecker::check(SourceVariableSemanticsInput::new(&scope));
        assert_eq!(
            output
                .diagnostics()
                .first()
                .map(SourceVariableSemanticsDiagnostic::detail_key),
            Some("variables.reconsider.unjustified_narrowing")
        );
        assert!(
            output
                .binding_types()
                .values()
                .all(|ty| ty.radix() == SourceVariableTypeRadix::Object)
        );
    }

    #[test]
    fn source_variable_checker_stops_state_mutation_after_first_diagnostic() {
        let source = source_id();
        let module = module_id();
        let ast = source_variable_diagnostic_freeze_ast(source);
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let scope = SourceVariableScopeResolver::resolve(SourceVariableScopeInput::new(
            &ast, &module, &symbols,
        ))
        .expect("diagnostic freeze fixture should resolve");

        let output =
            SourceVariableSemanticsChecker::check(SourceVariableSemanticsInput::new(&scope));
        assert_eq!(output.diagnostics().len(), 1);
        assert_eq!(
            output.diagnostics()[0].detail_key(),
            "variables.reconsider.unjustified_narrowing"
        );
        assert_eq!(output.binding_types().len(), 1);
        assert!(output.assumptions().is_empty());
        assert!(matches!(
            output.thesis(),
            Some(SourceVariableFormula::Equality { .. })
        ));
        assert!(scope.bindings().iter().any(|binding| {
            binding.kind() == SourceVariableBindingKind::Let && binding.spelling() == "later"
        }));
        assert!(
            scope
                .statements()
                .iter()
                .any(|statement| { matches!(statement, SourceVariableStatement::Set { .. }) })
        );
    }

    fn source_variable_type_projection_ast(source: SourceId) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source);
        let reserve_x = source_variable_reserve_item(&mut builder, source, 0, "x", "set");
        let reserve_y = source_variable_reserve_item(&mut builder, source, 21, "y", "object");
        let explicit_x_type = source_variable_type(&mut builder, source, 49, "object");
        let explicit_x =
            source_variable_let(&mut builder, source, 45, "x", Some(explicit_x_type), None);
        let implicit_y = source_variable_let(&mut builder, source, 65, "y", None, None);
        source_variable_finish(
            builder,
            source,
            0,
            70,
            vec![reserve_x, reserve_y, explicit_x, implicit_y],
        )
    }

    fn source_variable_alias_take_ast(source: SourceId) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source);
        let reserve_x = source_variable_reserve_item(&mut builder, source, 0, "x", "object");
        let thesis_body = source_variable_equality(&mut builder, source, 41, "y", "x");
        let existential = source_variable_existential(
            &mut builder,
            source,
            20,
            "y",
            thesis_body,
        );
        let let_condition = source_variable_equality(&mut builder, source, 115, "l", "l");
        let local_let_type = source_variable_type(&mut builder, source, 105, "object");
        let local_let = source_variable_let(
            &mut builder,
            source,
            100,
            "l",
            Some(local_let_type),
            Some(let_condition),
        );
        let set_alias = source_variable_set(&mut builder, source, 125, "a", "x");
        let alias_fact = source_variable_type_assertion(&mut builder, source, 145, "x", "set");
        let alias_fact = source_variable_assumption(&mut builder, source, alias_fact);
        let reconsider = source_variable_reconsider(&mut builder, source, 165, "b", "a", "set");
        let take = source_variable_take(&mut builder, source, 190, "a");
        let proof = source_variable_proof(
            &mut builder,
            source,
            90,
            vec![local_let, set_alias, alias_fact, reconsider, take],
        );
        let theorem = builder.add_node(
            SurfaceNodeKind::TheoremItem,
            range(source, 20, 220),
            vec![existential, proof],
        );
        source_variable_finish(builder, source, 0, 220, vec![reserve_x, theorem])
    }

    fn source_variable_capture_shadow_ast(source: SourceId, recursive: bool) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source);
        let reserve_x = source_variable_reserve_item(&mut builder, source, 0, "x", "object");
        let theorem_formula = source_variable_equality(&mut builder, source, 34, "x", "x");
        let body = if recursive {
            source_variable_application(&mut builder, source, 135, "F", "y", 138)
        } else {
            source_variable_term(&mut builder, source, "x", 135)
        };
        let functor = source_variable_functor_definition(&mut builder, source, 100, body);
        let outer_alias = source_variable_set(&mut builder, source, 145, "a", "x");
        let local_let_type = source_variable_type(&mut builder, source, 154, "object");
        let local_let =
            source_variable_let(&mut builder, source, 150, "x", Some(local_let_type), None);
        let fact = source_variable_type_assertion_application(
            &mut builder,
            source,
            170,
            "F",
            "x",
            173,
            "set",
        );
        let fact = source_variable_assumption(&mut builder, source, fact);
        let reconsider = source_variable_reconsider(&mut builder, source, 200, "z", "a", "set");
        let proof = source_variable_proof(
            &mut builder,
            source,
            90,
            vec![functor, outer_alias, local_let, fact, reconsider],
        );
        let theorem = builder.add_node(
            SurfaceNodeKind::TheoremItem,
            range(source, 20, 250),
            vec![theorem_formula, proof],
        );
        source_variable_finish(builder, source, 0, 250, vec![reserve_x, theorem])
    }

    fn source_variable_diagnostic_freeze_ast(source: SourceId) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source);
        let reserve_x = source_variable_reserve_item(&mut builder, source, 0, "x", "object");
        let theorem_formula = source_variable_equality(&mut builder, source, 25, "x", "x");
        let first_reconsider =
            source_variable_reconsider(&mut builder, source, 90, "bad", "x", "set");
        let later_condition = source_variable_equality(&mut builder, source, 130, "later", "later");
        let later_let_type = source_variable_type(&mut builder, source, 132, "object");
        let later_let = source_variable_let(
            &mut builder,
            source,
            125,
            "later",
            Some(later_let_type),
            Some(later_condition),
        );
        let later_set = source_variable_set(&mut builder, source, 160, "alias", "x");
        let later_take = source_variable_take(&mut builder, source, 185, "x");
        let proof = source_variable_proof(
            &mut builder,
            source,
            80,
            vec![first_reconsider, later_let, later_set, later_take],
        );
        let theorem = builder.add_node(
            SurfaceNodeKind::TheoremItem,
            range(source, 20, 215),
            vec![theorem_formula, proof],
        );
        source_variable_finish(builder, source, 0, 215, vec![reserve_x, theorem])
    }

    fn source_variable_token(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        kind: SurfaceTokenKind,
        text: &str,
        start: usize,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        builder.add_token(kind, text, range(source, start, start + text.len()))
    }

    fn source_variable_type(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        start: usize,
        radix: &str,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        let head_token = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedWord,
            radix,
            start,
        );
        let head = builder.add_node(
            SurfaceNodeKind::TypeHead,
            range(source, start, start + radix.len()),
            vec![head_token],
        );
        builder.add_node(
            SurfaceNodeKind::TypeExpression,
            range(source, start, start + radix.len()),
            vec![head],
        )
    }

    fn source_variable_reference(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        spelling: &str,
        start: usize,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        let token = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::Identifier,
            spelling,
            start,
        );
        builder.add_node(
            SurfaceNodeKind::TermReference,
            range(source, start, start + spelling.len()),
            vec![token],
        )
    }

    fn source_variable_term(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        spelling: &str,
        start: usize,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        let reference = source_variable_reference(builder, source, spelling, start);
        builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source, start, start + spelling.len()),
            vec![reference],
        )
    }

    fn source_variable_equality(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        start: usize,
        left: &str,
        right: &str,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        let left_term = source_variable_term(builder, source, left, start);
        let equals_start = start + left.len() + 1;
        let equals = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedSymbol,
            "=",
            equals_start,
        );
        let right_start = equals_start + 2;
        let right_term = source_variable_term(builder, source, right, right_start);
        let end = right_start + right.len();
        let equality = builder.add_node(
            SurfaceNodeKind::BuiltinPredicateApplication,
            range(source, start, end),
            vec![left_term, equals, right_term],
        );
        builder.add_node(
            SurfaceNodeKind::FormulaExpression,
            range(source, start, end),
            vec![equality],
        )
    }

    fn source_variable_type_assertion(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        start: usize,
        spelling: &str,
        radix: &str,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        let term = source_variable_term(builder, source, spelling, start);
        let is_start = start + spelling.len() + 1;
        let is_token = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedWord,
            "is",
            is_start,
        );
        let type_start = is_start + 3;
        let type_expression = source_variable_type(builder, source, type_start, radix);
        let end = type_start + radix.len();
        let assertion = builder.add_node(
            SurfaceNodeKind::IsAssertion,
            range(source, start, end),
            vec![term, is_token, type_expression],
        );
        builder.add_node(
            SurfaceNodeKind::FormulaExpression,
            range(source, start, end),
            vec![assertion],
        )
    }

    fn source_variable_application(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        start: usize,
        callee: &str,
        argument: &str,
        argument_start: usize,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        let open = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedSymbol,
            "(",
            start,
        );
        let callee_reference = source_variable_reference(builder, source, callee, start + 1);
        let argument_term = source_variable_term(builder, source, argument, argument_start);
        let close_start = argument_start + argument.len();
        let close = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedSymbol,
            ")",
            close_start,
        );
        let application_end = close_start + 1;
        let application = builder.add_node(
            SurfaceNodeKind::ApplicationTerm,
            range(source, start, application_end),
            vec![open, callee_reference, argument_term, close],
        );
        builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source, start, application_end),
            vec![application],
        )
    }

    fn source_variable_type_assertion_application(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        start: usize,
        callee: &str,
        argument: &str,
        argument_start: usize,
        radix: &str,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        let term =
            source_variable_application(builder, source, start, callee, argument, argument_start);
        let end = builder
            .node_range(term)
            .expect("application term range should exist")
            .end;
        let is_start = end + 1;
        let is_token = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedWord,
            "is",
            is_start,
        );
        let type_expression = source_variable_type(builder, source, is_start + 3, radix);
        let assertion_end = is_start + 3 + radix.len();
        let assertion = builder.add_node(
            SurfaceNodeKind::IsAssertion,
            range(source, start, assertion_end),
            vec![term, is_token, type_expression],
        );
        builder.add_node(
            SurfaceNodeKind::FormulaExpression,
            range(source, start, assertion_end),
            vec![assertion],
        )
    }

    fn source_variable_assumption(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        formula: mizar_syntax::SurfaceBuilderNodeId,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        let formula_range = builder
            .node_range(formula)
            .expect("assumption formula range should exist");
        builder.add_node(
            SurfaceNodeKind::AssumptionStatement,
            range(source, formula_range.start, formula_range.end),
            vec![formula],
        )
    }

    fn source_variable_reserve_item(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        start: usize,
        spelling: &str,
        radix: &str,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        let keyword = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedWord,
            "reserve",
            start,
        );
        let binder_start = start + 8;
        let binder = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::Identifier,
            spelling,
            binder_start,
        );
        let be = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedWord,
            "be",
            binder_start + spelling.len() + 1,
        );
        let type_start = binder_start + spelling.len() + 4;
        let type_expression = source_variable_type(builder, source, type_start, radix);
        let segment_end = type_start + radix.len();
        let segment = builder.add_node(
            SurfaceNodeKind::ReserveSegment,
            range(source, binder_start, segment_end),
            vec![binder, be, type_expression],
        );
        builder.add_node(
            SurfaceNodeKind::ReserveItem,
            range(source, start, segment_end),
            vec![keyword, segment],
        )
    }

    fn source_variable_let(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        start: usize,
        spelling: &str,
        type_expression: Option<mizar_syntax::SurfaceBuilderNodeId>,
        condition: Option<mizar_syntax::SurfaceBuilderNodeId>,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        let binder = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::Identifier,
            spelling,
            start,
        );
        let mut children = vec![binder];
        let end = if let Some(type_expression) = type_expression {
            let being = source_variable_token(
                builder,
                source,
                SurfaceTokenKind::ReservedWord,
                "being",
                start + spelling.len() + 1,
            );
            children.push(being);
            children.push(type_expression);
            builder
                .node_range(type_expression)
                .expect("local type range should exist")
                .end
        } else {
            start + spelling.len()
        };
        if let Some(condition) = condition {
            children.push(condition);
        }
        let condition_end = children
            .last()
            .and_then(|child| builder.node_range(*child))
            .map_or(end, |range| range.end);
        let segment = builder.add_node(
            SurfaceNodeKind::QualifiedVariableSegment,
            range(source, start, end),
            children[..if type_expression.is_some() { 3 } else { 1 }].to_vec(),
        );
        let mut statement_children = vec![segment];
        if let Some(condition) = condition {
            statement_children.push(condition);
        }
        builder.add_node(
            SurfaceNodeKind::LetStatement,
            range(source, start, condition_end),
            statement_children,
        )
    }

    fn source_variable_set(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        start: usize,
        spelling: &str,
        value: &str,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        let binder = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::Identifier,
            spelling,
            start,
        );
        let equals = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedSymbol,
            "=",
            start + spelling.len() + 1,
        );
        let value_start = start + spelling.len() + 3;
        let value_term = source_variable_term(builder, source, value, value_start);
        let end = value_start + value.len();
        let equating = builder.add_node(
            SurfaceNodeKind::Equating,
            range(source, start, end),
            vec![binder, equals, value_term],
        );
        builder.add_node(
            SurfaceNodeKind::SetStatement,
            range(source, start, end),
            vec![equating],
        )
    }

    fn source_variable_reconsider(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        start: usize,
        spelling: &str,
        value: &str,
        radix: &str,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        let target = source_variable_type(builder, source, start, radix);
        let item_start = start + radix.len() + 2;
        let binder = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::Identifier,
            spelling,
            item_start,
        );
        let equals = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedSymbol,
            "=",
            item_start + spelling.len() + 1,
        );
        let value_start = item_start + spelling.len() + 3;
        let value_term = source_variable_term(builder, source, value, value_start);
        let end = value_start + value.len();
        let item = builder.add_node(
            SurfaceNodeKind::ReconsiderItem,
            range(source, item_start, end),
            vec![binder, equals, value_term],
        );
        builder.add_node(
            SurfaceNodeKind::ReconsiderStatement,
            range(source, start, end),
            vec![target, item],
        )
    }

    fn source_variable_take(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        start: usize,
        spelling: &str,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        let term = source_variable_term(builder, source, spelling, start);
        let witness = builder.add_node(
            SurfaceNodeKind::Witness,
            range(source, start, start + spelling.len()),
            vec![term],
        );
        builder.add_node(
            SurfaceNodeKind::TakeStatement,
            range(source, start, start + spelling.len()),
            vec![witness],
        )
    }

    fn source_variable_functor_definition(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        start: usize,
        body: mizar_syntax::SurfaceBuilderNodeId,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        let keyword = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedWord,
            "deffunc",
            start,
        );
        let name = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::Identifier,
            "F",
            start + 8,
        );
        let open = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedSymbol,
            "(",
            start + 9,
        );
        let formal_name = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::Identifier,
            "y",
            start + 11,
        );
        let formal_being = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedWord,
            "being",
            start + 13,
        );
        let formal_type = source_variable_type(builder, source, start + 19, "object");
        let formal = builder.add_node(
            SurfaceNodeKind::TypedParameter,
            range(source, start + 11, start + 25),
            vec![formal_name, formal_being, formal_type],
        );
        let result_type = source_variable_type(builder, source, start + 27, "object");
        let body_range = builder
            .node_range(body)
            .expect("functor body range should exist");
        let definiens = builder.add_node(SurfaceNodeKind::TermDefiniens, body_range, vec![body]);
        builder.add_node(
            SurfaceNodeKind::InlineFunctorDefinition,
            range(source, start, body_range.end),
            vec![keyword, name, open, formal, result_type, definiens],
        )
    }

    fn source_variable_proof(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        start: usize,
        statements: Vec<mizar_syntax::SurfaceBuilderNodeId>,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        let proof = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedWord,
            "proof",
            start,
        );
        let end_start = statements
            .iter()
            .filter_map(|statement| builder.node_range(*statement))
            .map(|range| range.end)
            .max()
            .unwrap_or(start + 5)
            + 1;
        let end = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedWord,
            "end",
            end_start,
        );
        let proof_end = end_start + 3;
        let mut children = vec![proof];
        children.extend(statements);
        children.push(end);
        builder.add_node(
            SurfaceNodeKind::ProofBlock,
            range(source, start, proof_end),
            children,
        )
    }

    fn source_variable_existential(
        builder: &mut SurfaceAstBuilder,
        source: SourceId,
        start: usize,
        spelling: &str,
        body: mizar_syntax::SurfaceBuilderNodeId,
    ) -> mizar_syntax::SurfaceBuilderNodeId {
        let keyword = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedWord,
            "ex",
            start,
        );
        let binder_start = start + 3;
        let binder = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::Identifier,
            spelling,
            binder_start,
        );
        let being = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedWord,
            "being",
            binder_start + spelling.len() + 1,
        );
        let type_start = binder_start + spelling.len() + 7;
        let type_expression = source_variable_type(builder, source, type_start, "object");
        let segment_end = type_start + 6;
        let segment = builder.add_node(
            SurfaceNodeKind::QuantifierVariableSegment,
            range(source, binder_start, segment_end),
            vec![binder, being, type_expression],
        );
        let tail_start = segment_end + 1;
        let tail = source_variable_token(
            builder,
            source,
            SurfaceTokenKind::ReservedWord,
            "st",
            tail_start,
        );
        let body_range = builder
            .node_range(body)
            .expect("quantified body range should exist");
        builder.add_node(
            SurfaceNodeKind::QuantifiedFormula(SurfaceQuantifierKind::Existential),
            range(source, start, body_range.end),
            vec![keyword, segment, tail, body],
        )
    }

    fn source_variable_finish(
        mut builder: SurfaceAstBuilder,
        source: SourceId,
        start: usize,
        end: usize,
        items: Vec<mizar_syntax::SurfaceBuilderNodeId>,
    ) -> SurfaceAst {
        let item_list =
            builder.add_node(SurfaceNodeKind::ItemList, range(source, start, end), items);
        let unit = builder.add_node(
            SurfaceNodeKind::CompilationUnit,
            range(source, start, end),
            vec![item_list],
        );
        let root = builder.add_node(SurfaceNodeKind::Root, range(source, start, end), vec![unit]);
        builder.finish(Some(root), None)
    }
