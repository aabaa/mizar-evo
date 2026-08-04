use mizar_checker::{
    binding_env::BindingContextId,
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_proof_local_declaration::{
        SourceProofLocalDeclarationHandoffInput, SourceProofLocalDeclarationInput,
        SourceProofLocalDeclarationKind, SourceProofLocalDeclarationProducer,
        SourceProofLocalDeclarationRecovery, SourceProofLocalGivenBindingHandoffInput,
        SourceProofLocalGivenBindingProducer, SourceProofLocalGivenBindingRecovery,
        SourceProofLocalGivenConditionBindingHandoffInput,
        SourceProofLocalGivenConditionBindingProducer, SourceProofLocalGivenUseBindingHandoff,
        SourceProofLocalGivenUseBindingHandoffInput, SourceProofLocalGivenUseBindingProducer,
        SourceProofLocalLetBindingHandoffInput, SourceProofLocalLetBindingProducer,
        SourceProofLocalLetBindingRecovery,
    },
    source_statement::{
        SourceStatementWitnessId, SourceStatementWitnessNameId, SourceStatementWitnessTermTarget,
    },
    source_term::{
        SourcePrimaryTermHandoffInput, SourcePrimaryTermId, SourcePrimaryTermInput,
        SourcePrimaryTermKind, SourcePrimaryTermRecovery, SourcePrimaryTermReferenceInput,
        SourcePrimaryTermReferenceRole, SourcePrimaryTermRole,
        SourceProofLocalGivenConditionUseTermProducer, SourceProofLocalGivenUseTermProducer,
    },
    source_type::{
        SourceProofLocalGivenConditionTypeProducer, SourceProofLocalGivenTypeProducer,
        SourceProofLocalGivenUseTypeProducer, SourceProofLocalLetTypeProducer,
        SourceTypeApplicationForm, SourceTypeApplicationInput, SourceTypeExpressionId,
        SourceTypeExpressionInput, SourceTypeHandoffInput, SourceTypeHead,
    },
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
        TypeFactTable, TypeRole, TypeTable, TypedArena, TypedArenaBuilder, TypedAst, TypedAstParts,
        TypedNode, TypedNodeId, TypedSiteRef,
    },
};
#[cfg(test)]
use mizar_checker::{
    source_term::SourcePrimaryTermProducer,
    typed_ast::{LocalTypeContextId, NodeRecoveryState, TypedNodeLinks, TypingState},
};
#[cfg(test)]
use mizar_resolve::resolved_ast::{ResolvedArenaBuilder, ResolvedNode, SemanticOrigin};
use mizar_resolve::{
    declarations::DeclarationShellSet,
    env::SymbolEnv,
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::ModuleId,
};
use mizar_session::{ModulePath, SourceAnchor, SourceRange};
use mizar_syntax::SurfaceAst;
#[cfg(test)]
use mizar_syntax::SurfaceNodeKind;

use super::{
    checker_handoff::{assemble_empty_resolved_typed_ast, source_module_binding_env},
    source_statement::{
        SOURCE_STATEMENT_B3M1_TEXT, SOURCE_STATEMENT_B3N_TEXT, SourceStatementRouteOutput,
        extract_multiple_witness_source_statement, extract_named_witness_source_statement,
        source_proof_local_given_condition_lower_output, source_proof_local_given_lower_output,
        source_proof_local_given_use_lower_output, source_proof_local_let_lower_output,
        source_statement_output_with_source,
    },
};

use super::source_reserve::extract_builtin_source_reserve_declarations_after_node_guard;

#[cfg(test)]
use super::source_statement::{
    SourceStatementB3RouteInputs, source_statement_b3m1_output_with_mutation,
    source_statement_b3n_output_with_mutation,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceProofLocalProfile {
    Task258B3N,
    Task258B3M1,
}

#[derive(Debug)]
#[allow(dead_code)] // Rationale: Tasks 269A/B are private dormant consumers until a later activation task.
pub(in crate::runner) struct SourceProofLocalDeclarationRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; remaining variants are private corruption seams.
pub(in crate::runner) enum SourceProofLocalDeclarationRouteMutation {
    None,
    WrongLocalSpelling,
    WrongLocalScope,
    WrongLocalRange,
    WrongLocalVisibleAfter,
    WrongWitness,
    WrongName,
    WrongRhs,
    WrongBindingContext,
    WrongSourceOrdinal,
    WrongLowerStatement,
    WrongLowerWitness,
    WrongLowerPrimary,
    WrongLowerArenaCardinality,
    WrongLowerArenaRoot,
    WrongLowerArenaKind,
    WrongLowerArenaAnchor,
    WrongLowerArenaChildren,
    WrongLowerArenaResolvedNode,
    WrongLowerArenaRecovery,
    WrongLowerArenaTyping,
    WrongLowerArenaLinks,
}

#[allow(dead_code)] // Rationale: Tasks 269A/B deliberately leave this exact private leaf dormant.
pub(in crate::runner) fn source_proof_local_declaration_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalDeclarationRouteOutput, String>> {
    source_proof_local_declaration_output_impl(
        ast,
        module,
        symbols,
        source_text,
        SourceProofLocalDeclarationRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_declaration_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalDeclarationRouteMutation,
) -> Option<Result<SourceProofLocalDeclarationRouteOutput, String>> {
    source_proof_local_declaration_output_impl(ast, module, symbols, source_text, mutation)
}

fn source_proof_local_declaration_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalDeclarationRouteMutation,
) -> Option<Result<SourceProofLocalDeclarationRouteOutput, String>> {
    let (profile, name_range) = if source_text == SOURCE_STATEMENT_B3N_TEXT {
        let extracted = extract_named_witness_source_statement(ast, source_text)?;
        if extracted.name_site.node().index() != 13
            || extracted.name_range.start != 81
            || extracted.name_range.end != 82
            || extracted.witness_site.node().index() != 36
            || extracted.witness_range.start != 81
            || extracted.witness_range.end != 86
            || extracted.take_site.node().index() != 37
            || extracted.take_range.start != 76
            || extracted.take_range.end != 87
            || extracted.term_sites[2].node().index() != 34
            || extracted.term_ranges[2].start != 85
            || extracted.term_ranges[2].end != 86
        {
            return None;
        }
        (SourceProofLocalProfile::Task258B3N, extracted.name_range)
    } else if source_text == SOURCE_STATEMENT_B3M1_TEXT {
        let extracted = extract_multiple_witness_source_statement(ast, source_text)?;
        if extracted.name_site.node().index() != 13
            || extracted.name_range.start != 84
            || extracted.name_range.end != 85
            || extracted.witness_sites[0].node().index() != 38
            || extracted.witness_ranges[0].start != 84
            || extracted.witness_ranges[0].end != 89
            || extracted.witness_sites[1].node().index() != 41
            || extracted.witness_ranges[1].start != 91
            || extracted.witness_ranges[1].end != 92
            || extracted.take_site.node().index() != 42
            || extracted.take_range.start != 79
            || extracted.take_range.end != 93
            || extracted.term_sites[2].node().index() != 36
            || extracted.term_ranges[2].start != 88
            || extracted.term_ranges[2].end != 89
            || extracted.term_sites[3].node().index() != 39
            || extracted.term_ranges[3].start != 91
            || extracted.term_ranges[3].end != 92
        {
            return None;
        }
        (SourceProofLocalProfile::Task258B3M1, extracted.name_range)
    } else {
        return None;
    };

    let lower = lower_output(ast, module, symbols, source_text, profile, mutation)?;
    Some(lower.and_then(|lower| {
        build_source_proof_local_declaration_output(ast, name_range, lower, mutation)
    }))
}

fn lower_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    profile: SourceProofLocalProfile,
    mutation: SourceProofLocalDeclarationRouteMutation,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    #[cfg(not(test))]
    let _ = (profile, mutation);
    #[cfg(test)]
    match mutation {
        SourceProofLocalDeclarationRouteMutation::WrongLowerStatement => {
            return lower_output_with_mutation(
                ast,
                module,
                symbols,
                source_text,
                profile,
                |input| input.statement.statements[1].source_ordinal = 3,
            );
        }
        SourceProofLocalDeclarationRouteMutation::WrongLowerWitness => {
            return lower_output_with_mutation(
                ast,
                module,
                symbols,
                source_text,
                profile,
                |input| input.witness.names[0].spelling = "z".to_owned(),
            );
        }
        SourceProofLocalDeclarationRouteMutation::WrongLowerPrimary => {
            return lower_output_with_mutation(
                ast,
                module,
                symbols,
                source_text,
                profile,
                |input| {
                    input.primary = SourcePrimaryTermProducer::build(
                        SourcePrimaryTermHandoffInput {
                            source_id: input.binding_env.source_id(),
                            module_id: input.binding_env.module_id().clone(),
                            terms: Vec::new(),
                            references: Vec::new(),
                            numeric_type_requests: Vec::new(),
                        },
                        &input.binding_env,
                        &input.arena,
                    )
                    .expect("Task269A empty primary corruption is internally valid");
                },
            );
        }
        SourceProofLocalDeclarationRouteMutation::WrongLowerArenaCardinality
        | SourceProofLocalDeclarationRouteMutation::WrongLowerArenaRoot
        | SourceProofLocalDeclarationRouteMutation::WrongLowerArenaKind
        | SourceProofLocalDeclarationRouteMutation::WrongLowerArenaAnchor
        | SourceProofLocalDeclarationRouteMutation::WrongLowerArenaChildren
        | SourceProofLocalDeclarationRouteMutation::WrongLowerArenaResolvedNode
        | SourceProofLocalDeclarationRouteMutation::WrongLowerArenaRecovery
        | SourceProofLocalDeclarationRouteMutation::WrongLowerArenaTyping
        | SourceProofLocalDeclarationRouteMutation::WrongLowerArenaLinks => {
            return lower_output_with_mutation(
                ast,
                module,
                symbols,
                source_text,
                profile,
                |input| {
                    let mut nodes = input
                        .arena
                        .iter()
                        .map(|(_, node)| node.clone())
                        .collect::<Vec<_>>();
                    let mut root = input.arena.root();
                    match mutation {
                        SourceProofLocalDeclarationRouteMutation::WrongLowerArenaCardinality => {
                            nodes.pop().expect("Task269 arena is non-empty");
                            root = Some(TypedNodeId::new(nodes.len() - 1));
                        }
                        SourceProofLocalDeclarationRouteMutation::WrongLowerArenaRoot => {
                            let prior = root.expect("Task269 arena root").index() - 1;
                            root = Some(TypedNodeId::new(prior));
                        }
                        SourceProofLocalDeclarationRouteMutation::WrongLowerArenaKind => {
                            nodes[13].kind = "source.task269.corrupt".into();
                        }
                        SourceProofLocalDeclarationRouteMutation::WrongLowerArenaAnchor => {
                            nodes[13].anchor = SourceAnchor::Range(SourceRange {
                                source_id: input.binding_env.source_id(),
                                start: 0,
                                end: 0,
                            });
                        }
                        SourceProofLocalDeclarationRouteMutation::WrongLowerArenaChildren => {
                            let index = root.expect("Task269 arena root").index();
                            nodes[index].children.clear();
                        }
                        SourceProofLocalDeclarationRouteMutation::WrongLowerArenaResolvedNode => {
                            let mut builder = ResolvedArenaBuilder::new();
                            let foreign = builder
                                .push(ResolvedNode::new(
                                    SurfaceNodeKind::Root,
                                    Vec::new(),
                                    SemanticOrigin::new(
                                        input.binding_env.source_id(),
                                        input.binding_env.module_id().clone(),
                                        SourceAnchor::Range(SourceRange {
                                            source_id: input.binding_env.source_id(),
                                            start: 0,
                                            end: 0,
                                        }),
                                        vec![269],
                                    ),
                                ))
                                .expect("Task269 foreign resolved-node id");
                            nodes[13].resolved_node = Some(foreign);
                        }
                        SourceProofLocalDeclarationRouteMutation::WrongLowerArenaRecovery => {
                            nodes[13].recovery = NodeRecoveryState::Recovered;
                        }
                        SourceProofLocalDeclarationRouteMutation::WrongLowerArenaTyping => {
                            nodes[13].typing = TypingState::Successful;
                        }
                        SourceProofLocalDeclarationRouteMutation::WrongLowerArenaLinks => {
                            nodes[13].links = TypedNodeLinks {
                                context: Some(LocalTypeContextId::new(0)),
                                ..TypedNodeLinks::default()
                            };
                        }
                        _ => unreachable!("Task269 lower arena mutation"),
                    }
                    input.arena = TypedArena::try_new(root, nodes)
                        .expect("Task269 arena corruption remains structurally valid");
                },
            );
        }
        _ => {}
    }

    source_statement_output_with_source(ast, module, symbols, source_text)
}

#[cfg(test)]
fn lower_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    profile: SourceProofLocalProfile,
    mutate: impl FnOnce(&mut SourceStatementB3RouteInputs),
) -> Option<Result<SourceStatementRouteOutput, String>> {
    match profile {
        SourceProofLocalProfile::Task258B3N => {
            source_statement_b3n_output_with_mutation(ast, module, symbols, source_text, mutate)
        }
        SourceProofLocalProfile::Task258B3M1 => {
            source_statement_b3m1_output_with_mutation(ast, module, symbols, source_text, mutate)
        }
    }
}

fn build_source_proof_local_declaration_output(
    ast: &SurfaceAst,
    exact_name_range: SourceRange,
    lower: SourceStatementRouteOutput,
    mutation: SourceProofLocalDeclarationRouteMutation,
) -> Result<SourceProofLocalDeclarationRouteOutput, String> {
    let (spelling, scope, declaration_range, visible_after_ordinal) = match mutation {
        SourceProofLocalDeclarationRouteMutation::WrongLocalSpelling => {
            ("z", LocalTermScope::new(vec![0]), exact_name_range, 1)
        }
        SourceProofLocalDeclarationRouteMutation::WrongLocalScope => {
            ("y", LocalTermScope::new(vec![1]), exact_name_range, 1)
        }
        SourceProofLocalDeclarationRouteMutation::WrongLocalRange => (
            "y",
            LocalTermScope::new(vec![0]),
            SourceRange {
                source_id: ast.source_id,
                start: 80,
                end: 82,
            },
            1,
        ),
        SourceProofLocalDeclarationRouteMutation::WrongLocalVisibleAfter => {
            ("y", LocalTermScope::new(vec![0]), exact_name_range, 2)
        }
        _ => ("y", LocalTermScope::new(vec![0]), exact_name_range, 1),
    };
    let witness = if mutation == SourceProofLocalDeclarationRouteMutation::WrongWitness {
        SourceStatementWitnessId::new(1)
    } else {
        SourceStatementWitnessId::new(0)
    };
    let name = if mutation == SourceProofLocalDeclarationRouteMutation::WrongName {
        SourceStatementWitnessNameId::new(1)
    } else {
        SourceStatementWitnessNameId::new(0)
    };
    let rhs = if mutation == SourceProofLocalDeclarationRouteMutation::WrongRhs {
        SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(3))
    } else {
        SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId::new(2))
    };
    let binding_context =
        if mutation == SourceProofLocalDeclarationRouteMutation::WrongBindingContext {
            BindingContextId::new(0)
        } else {
            BindingContextId::new(1)
        };
    let source_ordinal = if mutation == SourceProofLocalDeclarationRouteMutation::WrongSourceOrdinal
    {
        2
    } else {
        1
    };

    let input = SourceProofLocalDeclarationHandoffInput {
        source_id: ast.source_id,
        module_id: lower.typed_ast.module_id().clone(),
        declarations: vec![SourceProofLocalDeclarationInput {
            witness,
            name,
            rhs,
            binding_context,
            source_ordinal,
            local: LocalTermBinding::new(spelling, scope, declaration_range, visible_after_ordinal),
            kind: SourceProofLocalDeclarationKind::NamedWitness,
            recovery: SourceProofLocalDeclarationRecovery::Normal,
        }],
    };
    let handoff = SourceProofLocalDeclarationProducer::build(
        input,
        lower
            .typed_ast
            .source_statement()
            .ok_or_else(|| "Task269A lower statement handoff is missing".to_owned())?,
        lower
            .typed_ast
            .source_statement_witnesses()
            .ok_or_else(|| "Task269A lower witness handoff is missing".to_owned())?,
        lower
            .typed_ast
            .source_term()
            .ok_or_else(|| "Task269A lower source-term handoff is missing".to_owned())?,
        lower.typed_ast.nodes(),
    )
    .map_err(|error| error.to_string())?;
    let typed_ast = lower
        .typed_ast
        .with_source_proof_local_declaration(handoff)
        .map_err(|error| error.to_string())?;
    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.statement.transport"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)?;
    Ok(SourceProofLocalDeclarationRouteOutput {
        typed_ast,
        resolved,
    })
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: Task 269C is a private dormant runner consumer until activation.
pub(in crate::runner) struct SourceProofLocalLetBindingRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; other variants are private corruption seams.
pub(in crate::runner) enum SourceProofLocalLetBindingRouteMutation {
    None,
    WrongLowerFingerprint,
    EmptyBase,
    WrongTheoremRange,
    WrongProofRange,
    WrongLetRange,
    WrongSegmentRange,
    WrongNameRange,
    WrongLocalSpelling,
    WrongLocalScope,
    WrongLocalRange,
    WrongLocalVisibleAfter,
    WrongSourceOrdinal,
}

#[allow(dead_code)] // Rationale: Task 269C deliberately leaves this exact private leaf dormant.
pub(in crate::runner) fn source_proof_local_let_binding_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalLetBindingRouteOutput, String>> {
    source_proof_local_let_binding_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceProofLocalLetBindingRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_let_binding_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalLetBindingRouteMutation,
) -> Option<Result<SourceProofLocalLetBindingRouteOutput, String>> {
    source_proof_local_let_binding_output_impl(ast, module, shells, symbols, source_text, mutation)
}

fn source_proof_local_let_binding_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalLetBindingRouteMutation,
) -> Option<Result<SourceProofLocalLetBindingRouteOutput, String>> {
    let lower =
        source_proof_local_let_lower_output(ast, module.clone(), shells, symbols, source_text)?;
    Some(lower.and_then(|lower| {
        let extraction = extract_builtin_source_reserve_declarations_after_node_guard(
            ast,
            module.clone(),
            symbols,
        )
        .map_err(|()| "Task269C exact reserve base extraction failed".to_owned())?;
        let exact_base = extraction
            .bridge
            .prepare_binding_env(symbols)
            .map_err(|error| format!("Task269C exact reserve base failed: {error}"))?;
        let base = if mutation == SourceProofLocalLetBindingRouteMutation::EmptyBase {
            source_module_binding_env(ast, module.clone()).map_err(|error| error.to_string())?
        } else {
            exact_base
        };

        let mut lower_fingerprint = lower.debug_text();
        if mutation == SourceProofLocalLetBindingRouteMutation::WrongLowerFingerprint {
            lower_fingerprint.push_str("corrupt");
        }
        let theorem_range =
            if mutation == SourceProofLocalLetBindingRouteMutation::WrongTheoremRange {
                SourceRange {
                    end: lower.theorem_range().end + 1,
                    ..lower.theorem_range()
                }
            } else {
                lower.theorem_range()
            };
        let proof_range = mutated_task269c_range(
            lower.proof_range(),
            mutation == SourceProofLocalLetBindingRouteMutation::WrongProofRange,
        );
        let let_range = mutated_task269c_range(
            lower.let_range(),
            mutation == SourceProofLocalLetBindingRouteMutation::WrongLetRange,
        );
        let segment_range = mutated_task269c_range(
            lower.segment_range(),
            mutation == SourceProofLocalLetBindingRouteMutation::WrongSegmentRange,
        );
        let name_range = mutated_task269c_range(
            lower.name_range(),
            mutation == SourceProofLocalLetBindingRouteMutation::WrongNameRange,
        );
        let local = match mutation {
            SourceProofLocalLetBindingRouteMutation::WrongLocalSpelling => LocalTermBinding::new(
                "z",
                lower.local().scope().clone(),
                lower.local().declaration_range(),
                lower.local().visible_after_ordinal(),
            ),
            SourceProofLocalLetBindingRouteMutation::WrongLocalScope => LocalTermBinding::new(
                lower.local().spelling(),
                LocalTermScope::new(vec![1]),
                lower.local().declaration_range(),
                lower.local().visible_after_ordinal(),
            ),
            SourceProofLocalLetBindingRouteMutation::WrongLocalRange => LocalTermBinding::new(
                lower.local().spelling(),
                lower.local().scope().clone(),
                SourceRange {
                    start: lower.local().declaration_range().start - 1,
                    ..lower.local().declaration_range()
                },
                lower.local().visible_after_ordinal(),
            ),
            SourceProofLocalLetBindingRouteMutation::WrongLocalVisibleAfter => {
                LocalTermBinding::new(
                    lower.local().spelling(),
                    lower.local().scope().clone(),
                    lower.local().declaration_range(),
                    lower.local().visible_after_ordinal() + 1,
                )
            }
            _ => lower.local().clone(),
        };
        let source_ordinal =
            if mutation == SourceProofLocalLetBindingRouteMutation::WrongSourceOrdinal {
                lower.source_ordinal() + 1
            } else {
                lower.source_ordinal()
            };
        let handoff = SourceProofLocalLetBindingProducer::build(
            SourceProofLocalLetBindingHandoffInput {
                source_id: lower.source_id(),
                module_id: lower.module_id().clone(),
                lower_fingerprint,
                theorem_symbol: lower.theorem_symbol().clone(),
                theorem_definition: lower.theorem_definition(),
                contribution: lower.contribution(),
                theorem_range,
                proof_range,
                let_range,
                segment_range,
                name_range,
                source_ordinal,
                local,
                recovery: SourceProofLocalLetBindingRecovery::Normal,
            },
            &base,
        )
        .map_err(|error| error.to_string())?;
        let typed_ast = empty_task269c_typed_ast(ast.source_id, module.clone())?
            .with_source_proof_local_let_binding(handoff)
            .map_err(|error| error.to_string())?;
        let resolved = assemble_empty_resolved_typed_ast(&typed_ast, Vec::new())?;
        Ok(SourceProofLocalLetBindingRouteOutput {
            typed_ast,
            resolved,
        })
    }))
}

fn mutated_task269c_range(mut source_range: SourceRange, mutate: bool) -> SourceRange {
    if mutate {
        source_range.end += 1;
    }
    source_range
}

fn empty_task269c_typed_ast(
    source_id: mizar_session::SourceId,
    module_id: ModuleId,
) -> Result<TypedAst, String> {
    TypedAst::try_new(TypedAstParts {
        source_id,
        module_id,
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: TypedArena::try_new(None, Vec::new()).map_err(|error| error.to_string())?,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: Task 269G is a private dormant runner consumer until activation.
pub(in crate::runner) struct SourceProofLocalGivenBindingRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; other variants are private corruption seams.
pub(in crate::runner) enum SourceProofLocalGivenBindingRouteMutation {
    None,
    WrongLowerFingerprint,
    EmptyBase,
    WrongTheoremRange,
    WrongProofRange,
    WrongGivenRange,
    WrongSegmentRange,
    WrongNameRange,
    WrongLocalSpelling,
    WrongLocalScope,
    WrongLocalRange,
    WrongLocalVisibleAfter,
    WrongSourceOrdinal,
}

#[allow(dead_code)] // Rationale: Task 269G deliberately leaves this exact private leaf dormant.
pub(in crate::runner) fn source_proof_local_given_binding_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalGivenBindingRouteOutput, String>> {
    source_proof_local_given_binding_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceProofLocalGivenBindingRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_binding_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenBindingRouteMutation,
) -> Option<Result<SourceProofLocalGivenBindingRouteOutput, String>> {
    source_proof_local_given_binding_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        mutation,
    )
}

fn source_proof_local_given_binding_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenBindingRouteMutation,
) -> Option<Result<SourceProofLocalGivenBindingRouteOutput, String>> {
    let lower =
        source_proof_local_given_lower_output(ast, module.clone(), shells, symbols, source_text)?;
    Some(lower.and_then(|lower| {
        let extraction = extract_builtin_source_reserve_declarations_after_node_guard(
            ast,
            module.clone(),
            symbols,
        )
        .map_err(|()| "Task269G exact reserve base extraction failed".to_owned())?;
        let exact_base = extraction
            .bridge
            .prepare_binding_env(symbols)
            .map_err(|error| format!("Task269G exact reserve base failed: {error}"))?;
        let base = if mutation == SourceProofLocalGivenBindingRouteMutation::EmptyBase {
            source_module_binding_env(ast, module.clone()).map_err(|error| error.to_string())?
        } else {
            exact_base
        };

        let mut lower_fingerprint = lower.debug_text();
        if mutation == SourceProofLocalGivenBindingRouteMutation::WrongLowerFingerprint {
            lower_fingerprint.push_str("corrupt");
        }
        let theorem_range = mutated_task269g_range(
            lower.theorem_range(),
            mutation == SourceProofLocalGivenBindingRouteMutation::WrongTheoremRange,
        );
        let proof_range = mutated_task269g_range(
            lower.proof_range(),
            mutation == SourceProofLocalGivenBindingRouteMutation::WrongProofRange,
        );
        let given_range = mutated_task269g_range(
            lower.given_range(),
            mutation == SourceProofLocalGivenBindingRouteMutation::WrongGivenRange,
        );
        let segment_range = mutated_task269g_range(
            lower.segment_range(),
            mutation == SourceProofLocalGivenBindingRouteMutation::WrongSegmentRange,
        );
        let name_range = mutated_task269g_range(
            lower.name_range(),
            mutation == SourceProofLocalGivenBindingRouteMutation::WrongNameRange,
        );
        let exact_local = LocalTermBinding::new(
            lower.name_spelling(),
            LocalTermScope::new(vec![0]),
            lower.name_range(),
            1,
        );
        let local = match mutation {
            SourceProofLocalGivenBindingRouteMutation::WrongLocalSpelling => LocalTermBinding::new(
                "z",
                exact_local.scope().clone(),
                exact_local.declaration_range(),
                exact_local.visible_after_ordinal(),
            ),
            SourceProofLocalGivenBindingRouteMutation::WrongLocalScope => LocalTermBinding::new(
                exact_local.spelling(),
                LocalTermScope::new(vec![1]),
                exact_local.declaration_range(),
                exact_local.visible_after_ordinal(),
            ),
            SourceProofLocalGivenBindingRouteMutation::WrongLocalRange => LocalTermBinding::new(
                exact_local.spelling(),
                exact_local.scope().clone(),
                SourceRange {
                    start: exact_local.declaration_range().start - 1,
                    ..exact_local.declaration_range()
                },
                exact_local.visible_after_ordinal(),
            ),
            SourceProofLocalGivenBindingRouteMutation::WrongLocalVisibleAfter => {
                LocalTermBinding::new(
                    exact_local.spelling(),
                    exact_local.scope().clone(),
                    exact_local.declaration_range(),
                    exact_local.visible_after_ordinal() + 1,
                )
            }
            _ => exact_local,
        };
        let source_ordinal =
            if mutation == SourceProofLocalGivenBindingRouteMutation::WrongSourceOrdinal {
                lower.source_ordinal() + 1
            } else {
                lower.source_ordinal()
            };
        let handoff = SourceProofLocalGivenBindingProducer::build(
            SourceProofLocalGivenBindingHandoffInput {
                source_id: lower.source_id(),
                module_id: lower.module_id().clone(),
                lower_fingerprint,
                theorem_symbol: lower.theorem_symbol().clone(),
                theorem_definition: lower.theorem_definition(),
                contribution: lower.contribution(),
                theorem_range,
                proof_range,
                given_range,
                segment_range,
                name_range,
                source_ordinal,
                local,
                recovery: SourceProofLocalGivenBindingRecovery::Normal,
            },
            &base,
        )
        .map_err(|error| error.to_string())?;
        let typed_ast = empty_task269c_typed_ast(ast.source_id, module.clone())?
            .with_source_proof_local_given_binding(handoff)
            .map_err(|error| error.to_string())?;
        let resolved = assemble_empty_resolved_typed_ast(&typed_ast, Vec::new())?;
        Ok(SourceProofLocalGivenBindingRouteOutput {
            typed_ast,
            resolved,
        })
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; other variants are private corruption seams.
pub(in crate::runner) enum SourceProofLocalGivenUseBindingRouteMutation {
    None,
    WrongLowerFingerprint,
    EmptyBase,
    WrongTheoremRange,
    WrongProofRange,
    WrongGivenRange,
    WrongSegmentRange,
    WrongNameRange,
    WrongLocalSpelling,
    WrongLocalScope,
    WrongLocalRange,
    WrongLocalVisibleAfter,
    WrongSourceOrdinal,
}

#[allow(dead_code)] // Rationale: Task 269GUP leaves this exact handoff seam dormant for GUPT.
pub(in crate::runner) fn source_proof_local_given_use_binding_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalGivenUseBindingHandoff, String>> {
    source_proof_local_given_use_binding_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceProofLocalGivenUseBindingRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_use_binding_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenUseBindingRouteMutation,
) -> Option<Result<SourceProofLocalGivenUseBindingHandoff, String>> {
    source_proof_local_given_use_binding_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        mutation,
    )
}

fn source_proof_local_given_use_binding_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenUseBindingRouteMutation,
) -> Option<Result<SourceProofLocalGivenUseBindingHandoff, String>> {
    let lower = source_proof_local_given_use_lower_output(
        ast,
        module.clone(),
        shells,
        symbols,
        source_text,
    )?;
    Some(lower.and_then(|lower| {
        let extraction = extract_builtin_source_reserve_declarations_after_node_guard(
            ast,
            module.clone(),
            symbols,
        )
        .map_err(|()| "Task269GUP exact reserve base extraction failed".to_owned())?;
        let exact_base = extraction
            .bridge
            .prepare_binding_env(symbols)
            .map_err(|error| format!("Task269GUP exact reserve base failed: {error}"))?;
        let base = if mutation == SourceProofLocalGivenUseBindingRouteMutation::EmptyBase {
            source_module_binding_env(ast, module).map_err(|error| error.to_string())?
        } else {
            exact_base
        };

        let mut lower_fingerprint = lower.debug_text();
        if mutation == SourceProofLocalGivenUseBindingRouteMutation::WrongLowerFingerprint {
            lower_fingerprint.push_str("corrupt");
        }
        let theorem_range = mutated_task269g_range(
            lower.theorem_range(),
            mutation == SourceProofLocalGivenUseBindingRouteMutation::WrongTheoremRange,
        );
        let proof_range = mutated_task269g_range(
            lower.proof_range(),
            mutation == SourceProofLocalGivenUseBindingRouteMutation::WrongProofRange,
        );
        let given_range = mutated_task269g_range(
            lower.given_range(),
            mutation == SourceProofLocalGivenUseBindingRouteMutation::WrongGivenRange,
        );
        let segment_range = mutated_task269g_range(
            lower.segment_range(),
            mutation == SourceProofLocalGivenUseBindingRouteMutation::WrongSegmentRange,
        );
        let name_range = mutated_task269g_range(
            lower.name_range(),
            mutation == SourceProofLocalGivenUseBindingRouteMutation::WrongNameRange,
        );
        let exact_local = LocalTermBinding::new(
            lower.name_spelling(),
            LocalTermScope::new(vec![0]),
            lower.name_range(),
            1,
        );
        let local = match mutation {
            SourceProofLocalGivenUseBindingRouteMutation::WrongLocalSpelling => {
                LocalTermBinding::new(
                    "z",
                    exact_local.scope().clone(),
                    exact_local.declaration_range(),
                    exact_local.visible_after_ordinal(),
                )
            }
            SourceProofLocalGivenUseBindingRouteMutation::WrongLocalScope => LocalTermBinding::new(
                exact_local.spelling(),
                LocalTermScope::new(vec![1]),
                exact_local.declaration_range(),
                exact_local.visible_after_ordinal(),
            ),
            SourceProofLocalGivenUseBindingRouteMutation::WrongLocalRange => LocalTermBinding::new(
                exact_local.spelling(),
                exact_local.scope().clone(),
                SourceRange {
                    start: exact_local.declaration_range().start - 1,
                    ..exact_local.declaration_range()
                },
                exact_local.visible_after_ordinal(),
            ),
            SourceProofLocalGivenUseBindingRouteMutation::WrongLocalVisibleAfter => {
                LocalTermBinding::new(
                    exact_local.spelling(),
                    exact_local.scope().clone(),
                    exact_local.declaration_range(),
                    exact_local.visible_after_ordinal() + 1,
                )
            }
            _ => exact_local,
        };
        let source_ordinal =
            if mutation == SourceProofLocalGivenUseBindingRouteMutation::WrongSourceOrdinal {
                lower.source_ordinal() + 1
            } else {
                lower.source_ordinal()
            };
        SourceProofLocalGivenUseBindingProducer::build(
            SourceProofLocalGivenUseBindingHandoffInput {
                source_id: lower.source_id(),
                module_id: lower.module_id().clone(),
                lower_fingerprint,
                theorem_symbol: lower.theorem_symbol().clone(),
                theorem_definition: lower.theorem_definition(),
                contribution: lower.contribution(),
                theorem_range,
                proof_range,
                given_range,
                segment_range,
                name_range,
                source_ordinal,
                local,
                recovery: SourceProofLocalGivenBindingRecovery::Normal,
            },
            &base,
        )
        .map_err(|error| error.to_string())
    }))
}

fn mutated_task269g_range(mut source_range: SourceRange, mutate: bool) -> SourceRange {
    if mutate {
        source_range.end += 1;
    }
    source_range
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: Task 269GC is a private dormant runner consumer until activation.
pub(in crate::runner) struct SourceProofLocalGivenConditionBindingRouteOutput {
    typed_ast: TypedAst,
    resolved: ResolvedTypedAst,
}

impl SourceProofLocalGivenConditionBindingRouteOutput {
    pub(in crate::runner) const fn typed_ast(&self) -> &TypedAst {
        &self.typed_ast
    }

    pub(in crate::runner) const fn resolved(&self) -> &ResolvedTypedAst {
        &self.resolved
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; other variants are private corruption seams.
pub(in crate::runner) enum SourceProofLocalGivenConditionBindingRouteMutation {
    None,
    WrongLowerFingerprint,
    EmptyBase,
    WrongTheoremRange,
    WrongProofRange,
    WrongGivenRange,
    WrongSegmentRange,
    WrongNameRange,
    WrongLocalSpelling,
    WrongLocalScope,
    WrongLocalRange,
    WrongLocalVisibleAfter,
    WrongSourceOrdinal,
}

#[allow(dead_code)] // Rationale: Task 269GC deliberately leaves this exact private leaf dormant.
pub(in crate::runner) fn source_proof_local_given_condition_binding_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalGivenConditionBindingRouteOutput, String>> {
    source_proof_local_given_condition_binding_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceProofLocalGivenConditionBindingRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_condition_binding_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenConditionBindingRouteMutation,
) -> Option<Result<SourceProofLocalGivenConditionBindingRouteOutput, String>> {
    source_proof_local_given_condition_binding_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        mutation,
    )
}

fn source_proof_local_given_condition_binding_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenConditionBindingRouteMutation,
) -> Option<Result<SourceProofLocalGivenConditionBindingRouteOutput, String>> {
    let lower = source_proof_local_given_condition_lower_output(
        ast,
        module.clone(),
        shells,
        symbols,
        source_text,
    )?;
    Some(lower.and_then(|lower| {
        let extraction = extract_builtin_source_reserve_declarations_after_node_guard(
            ast,
            module.clone(),
            symbols,
        )
        .map_err(|()| "Task269GC exact reserve base extraction failed".to_owned())?;
        let exact_base = extraction
            .bridge
            .prepare_binding_env(symbols)
            .map_err(|error| format!("Task269GC exact reserve base failed: {error}"))?;
        let base = if mutation == SourceProofLocalGivenConditionBindingRouteMutation::EmptyBase {
            source_module_binding_env(ast, module.clone()).map_err(|error| error.to_string())?
        } else {
            exact_base
        };

        let mut lower_fingerprint = lower.debug_text();
        if mutation == SourceProofLocalGivenConditionBindingRouteMutation::WrongLowerFingerprint {
            lower_fingerprint.push_str("corrupt");
        }
        let theorem_range = mutated_task269g_range(
            lower.theorem_range(),
            mutation == SourceProofLocalGivenConditionBindingRouteMutation::WrongTheoremRange,
        );
        let proof_range = mutated_task269g_range(
            lower.proof_range(),
            mutation == SourceProofLocalGivenConditionBindingRouteMutation::WrongProofRange,
        );
        let given_range = mutated_task269g_range(
            lower.given_range(),
            mutation == SourceProofLocalGivenConditionBindingRouteMutation::WrongGivenRange,
        );
        let segment_range = mutated_task269g_range(
            lower.segment_range(),
            mutation == SourceProofLocalGivenConditionBindingRouteMutation::WrongSegmentRange,
        );
        let name_range = mutated_task269g_range(
            lower.name_range(),
            mutation == SourceProofLocalGivenConditionBindingRouteMutation::WrongNameRange,
        );
        let exact_local = LocalTermBinding::new(
            lower.name_spelling(),
            LocalTermScope::new(vec![0]),
            lower.name_range(),
            1,
        );
        let local = match mutation {
            SourceProofLocalGivenConditionBindingRouteMutation::WrongLocalSpelling => {
                LocalTermBinding::new(
                    "z",
                    exact_local.scope().clone(),
                    exact_local.declaration_range(),
                    exact_local.visible_after_ordinal(),
                )
            }
            SourceProofLocalGivenConditionBindingRouteMutation::WrongLocalScope => {
                LocalTermBinding::new(
                    exact_local.spelling(),
                    LocalTermScope::new(vec![1]),
                    exact_local.declaration_range(),
                    exact_local.visible_after_ordinal(),
                )
            }
            SourceProofLocalGivenConditionBindingRouteMutation::WrongLocalRange => {
                LocalTermBinding::new(
                    exact_local.spelling(),
                    exact_local.scope().clone(),
                    SourceRange {
                        start: exact_local.declaration_range().start - 1,
                        ..exact_local.declaration_range()
                    },
                    exact_local.visible_after_ordinal(),
                )
            }
            SourceProofLocalGivenConditionBindingRouteMutation::WrongLocalVisibleAfter => {
                LocalTermBinding::new(
                    exact_local.spelling(),
                    exact_local.scope().clone(),
                    exact_local.declaration_range(),
                    exact_local.visible_after_ordinal() + 1,
                )
            }
            _ => exact_local,
        };
        let source_ordinal =
            if mutation == SourceProofLocalGivenConditionBindingRouteMutation::WrongSourceOrdinal {
                lower.source_ordinal() + 1
            } else {
                lower.source_ordinal()
            };
        let handoff = SourceProofLocalGivenConditionBindingProducer::build(
            SourceProofLocalGivenConditionBindingHandoffInput {
                source_id: lower.source_id(),
                module_id: lower.module_id().clone(),
                lower_fingerprint,
                theorem_symbol: lower.theorem_symbol().clone(),
                theorem_definition: lower.theorem_definition(),
                contribution: lower.contribution(),
                theorem_range,
                proof_range,
                given_range,
                segment_range,
                name_range,
                source_ordinal,
                local,
                recovery: SourceProofLocalGivenBindingRecovery::Normal,
            },
            &base,
        )
        .map_err(|error| error.to_string())?;
        let typed_ast = empty_task269c_typed_ast(ast.source_id, module.clone())?
            .with_source_proof_local_given_condition_binding(handoff)
            .map_err(|error| error.to_string())?;
        let resolved = assemble_empty_resolved_typed_ast(&typed_ast, Vec::new())?;
        let output = SourceProofLocalGivenConditionBindingRouteOutput {
            typed_ast,
            resolved,
        };
        let _ = (output.typed_ast(), output.resolved());
        Ok(output)
    }))
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: Task 269GCT is a private dormant runner consumer until activation.
pub(in crate::runner) struct SourceProofLocalGivenConditionTypeRouteOutput {
    typed_ast: TypedAst,
    resolved: ResolvedTypedAst,
}

impl SourceProofLocalGivenConditionTypeRouteOutput {
    pub(in crate::runner) const fn typed_ast(&self) -> &TypedAst {
        &self.typed_ast
    }

    pub(in crate::runner) const fn resolved(&self) -> &ResolvedTypedAst {
        &self.resolved
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; other variants are private Task-269GCT corruption seams.
pub(in crate::runner) enum SourceProofLocalGivenConditionTypeRouteMutation {
    None,
    WrongDependencyModule,
    WrongTypeRange,
    WrongArenaRoot,
    WrongArenaKind,
}

#[allow(dead_code)] // Rationale: Task 269GCT deliberately leaves this exact private leaf dormant.
pub(in crate::runner) fn source_proof_local_given_condition_type_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalGivenConditionTypeRouteOutput, String>> {
    source_proof_local_given_condition_type_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceProofLocalGivenConditionTypeRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_condition_type_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenConditionTypeRouteMutation,
) -> Option<Result<SourceProofLocalGivenConditionTypeRouteOutput, String>> {
    source_proof_local_given_condition_type_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        mutation,
    )
}

fn source_proof_local_given_condition_type_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenConditionTypeRouteMutation,
) -> Option<Result<SourceProofLocalGivenConditionTypeRouteOutput, String>> {
    let binding = source_proof_local_given_condition_binding_output(
        ast,
        module.clone(),
        shells,
        symbols,
        source_text,
    )?;
    let lower = source_proof_local_given_condition_lower_output(
        ast,
        module.clone(),
        shells,
        symbols,
        source_text,
    )?;
    Some(binding.and_then(|binding| {
        lower.and_then(|lower| {
            let dependency = binding
                .typed_ast()
                .source_proof_local_given_condition_binding()
                .cloned()
                .ok_or_else(|| "Task269GCT Task269GC dependency is missing".to_owned())?;
            let reserve_range = match dependency
                .binding_env()
                .bindings()
                .get(mizar_checker::binding_env::BindingId::new(0))
                .map(|binding| &binding.type_site)
            {
                Some(mizar_checker::binding_env::BindingTypeSite::Source(source_range)) => {
                    *source_range
                }
                _ => return Err("Task269GCT reserve type range is missing".to_owned()),
            };
            let local_range = lower.type_range();
            let local_head_range = lower.type_head_range();
            let local_spelling = lower.type_spelling();
            let arena = task269gct_arena(ast.source_id, mutation)?;
            let input = task269gct_type_input(
                ast.source_id,
                module.clone(),
                reserve_range,
                local_range,
                local_head_range,
                local_spelling,
            );
            #[cfg(test)]
            let mut input = input;
            #[cfg(test)]
            {
                if mutation
                    == SourceProofLocalGivenConditionTypeRouteMutation::WrongDependencyModule
                {
                    input.module_id = ModuleId::new(
                        module.package().clone(),
                        ModulePath::new("task269gct.wrong"),
                    );
                    for expression in &mut input.expressions {
                        expression.module_id = input.module_id.clone();
                    }
                }
                if mutation == SourceProofLocalGivenConditionTypeRouteMutation::WrongTypeRange {
                    input.expressions[1].source_range.end += 1;
                }
            }
            #[cfg(not(test))]
            let _ = mutation;
            let handoff = SourceProofLocalGivenConditionTypeProducer::build(
                dependency, input, symbols, &arena,
            )
            .map_err(|error| error.to_string())?;
            let typed_ast = empty_task269gct_typed_ast(ast.source_id, module, arena)?
                .with_source_proof_local_given_condition_type(handoff)
                .map_err(|error| error.to_string())?;
            let resolved = assemble_empty_resolved_typed_ast(&typed_ast, Vec::new())?;
            let output = SourceProofLocalGivenConditionTypeRouteOutput {
                typed_ast,
                resolved,
            };
            let _ = (output.typed_ast(), output.resolved());
            Ok(output)
        })
    }))
}

fn task269gct_type_input(
    source_id: mizar_session::SourceId,
    module_id: ModuleId,
    reserve_range: SourceRange,
    local_range: SourceRange,
    local_head_range: SourceRange,
    local_spelling: &str,
) -> SourceTypeHandoffInput {
    let rows = [
        (reserve_range, reserve_range, "set"),
        (local_range, local_head_range, local_spelling),
    ];
    SourceTypeHandoffInput {
        source_id,
        module_id: module_id.clone(),
        applications: rows
            .iter()
            .enumerate()
            .map(|(index, _)| SourceTypeApplicationInput {
                binding: mizar_checker::binding_env::BindingId::new(index),
                source_ordinal: index,
                root: SourceTypeExpressionId::new(index),
            })
            .collect(),
        expressions: rows
            .into_iter()
            .enumerate()
            .map(
                |(index, (source_range, head_range, spelling))| SourceTypeExpressionInput {
                    source_id,
                    module_id: module_id.clone(),
                    site: TypedSiteRef::Role {
                        node: TypedNodeId::new(index),
                        role: TypeRole::new("source.type.expression"),
                    },
                    source_range,
                    spelling: spelling.to_owned(),
                    head_site: TypedSiteRef::Role {
                        node: TypedNodeId::new(index),
                        role: TypeRole::new("source.type.head"),
                    },
                    head_range,
                    head_spelling: spelling.to_owned(),
                    form: SourceTypeApplicationForm::Bare,
                    head: SourceTypeHead::BuiltinSet,
                    recovery: mizar_checker::typed_ast::NodeRecoveryState::Normal,
                },
            )
            .collect(),
        arguments: Vec::new(),
    }
}

fn task269gct_arena(
    source_id: mizar_session::SourceId,
    mutation: SourceProofLocalGivenConditionTypeRouteMutation,
) -> Result<TypedArena, String> {
    #[cfg(not(test))]
    let _ = mutation;
    #[cfg(test)]
    let wrong_kind = mutation == SourceProofLocalGivenConditionTypeRouteMutation::WrongArenaKind;
    #[cfg(not(test))]
    let wrong_kind = false;
    #[cfg(test)]
    let wrong_root = mutation == SourceProofLocalGivenConditionTypeRouteMutation::WrongArenaRoot;
    #[cfg(not(test))]
    let wrong_root = false;
    let mut builder = TypedArenaBuilder::new();
    let reserve = builder
        .push(TypedNode::new(
            "source.proof-local.given-condition.reserve-type",
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 14,
                end: 17,
            }),
        ))
        .map_err(|error| error.to_string())?;
    let local = builder
        .push(TypedNode::new(
            if wrong_kind {
                "source.proof-local.given-condition.type.wrong"
            } else {
                "source.proof-local.given-condition.type"
            },
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 90,
                end: 93,
            }),
        ))
        .map_err(|error| error.to_string())?;
    let root = builder
        .push(
            TypedNode::new(
                "source.proof-local.given-condition.type-root",
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 0,
                    end: 133,
                }),
            )
            .with_children(vec![reserve, local]),
        )
        .map_err(|error| error.to_string())?;
    builder
        .finish(Some(if wrong_root { local } else { root }))
        .map_err(|error| error.to_string())
}

fn empty_task269gct_typed_ast(
    source_id: mizar_session::SourceId,
    module_id: ModuleId,
    nodes: TypedArena,
) -> Result<TypedAst, String> {
    TypedAst::try_new(TypedAstParts {
        source_id,
        module_id,
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: Task 269GCU is a private dormant runner consumer until activation.
pub(in crate::runner) struct SourceProofLocalGivenConditionUseTermRouteOutput {
    typed_ast: TypedAst,
    resolved: ResolvedTypedAst,
}

impl SourceProofLocalGivenConditionUseTermRouteOutput {
    pub(in crate::runner) const fn typed_ast(&self) -> &TypedAst {
        &self.typed_ast
    }

    pub(in crate::runner) const fn resolved(&self) -> &ResolvedTypedAst {
        &self.resolved
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; other variants are private Task-269GCU corruption seams.
pub(in crate::runner) enum SourceProofLocalGivenConditionUseTermRouteMutation {
    None,
    WrongDependencyModule,
    WrongTermRange,
    WrongReferenceBinding,
    WrongArenaRoot,
    WrongArenaKind,
}

#[allow(dead_code)] // Rationale: Task 269GCU deliberately leaves this exact private leaf dormant.
pub(in crate::runner) fn source_proof_local_given_condition_use_term_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalGivenConditionUseTermRouteOutput, String>> {
    source_proof_local_given_condition_use_term_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceProofLocalGivenConditionUseTermRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_condition_use_term_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenConditionUseTermRouteMutation,
) -> Option<Result<SourceProofLocalGivenConditionUseTermRouteOutput, String>> {
    source_proof_local_given_condition_use_term_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        mutation,
    )
}

fn source_proof_local_given_condition_use_term_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenConditionUseTermRouteMutation,
) -> Option<Result<SourceProofLocalGivenConditionUseTermRouteOutput, String>> {
    let dependency = source_proof_local_given_condition_type_output(
        ast,
        module.clone(),
        shells,
        symbols,
        source_text,
    )?;
    Some(dependency.and_then(|dependency| {
        let dependency = dependency
            .typed_ast()
            .source_proof_local_given_condition_type()
            .cloned()
            .ok_or_else(|| "Task269GCU GCT dependency is missing".to_owned())?;
        let arena = task269gcu_arena(ast.source_id, mutation)?;
        let mut input = task269gcu_term_input(ast.source_id, module.clone());
        match mutation {
            SourceProofLocalGivenConditionUseTermRouteMutation::WrongDependencyModule => {
                input.module_id = ModuleId::new(
                    module.package().clone(),
                    ModulePath::new("task269gcu.wrong"),
                );
            }
            SourceProofLocalGivenConditionUseTermRouteMutation::WrongTermRange => {
                input.terms[0].source_range.end += 1;
            }
            SourceProofLocalGivenConditionUseTermRouteMutation::WrongReferenceBinding => {
                input.references[0].binding = mizar_checker::binding_env::BindingId::new(0);
            }
            SourceProofLocalGivenConditionUseTermRouteMutation::None
            | SourceProofLocalGivenConditionUseTermRouteMutation::WrongArenaRoot
            | SourceProofLocalGivenConditionUseTermRouteMutation::WrongArenaKind => {}
        }
        let handoff =
            SourceProofLocalGivenConditionUseTermProducer::build(dependency, input, &arena)
                .map_err(|error| error.to_string())?;
        let typed_ast = empty_task269gcu_typed_ast(ast.source_id, module, arena)?
            .with_source_proof_local_given_condition_use_term(handoff)
            .map_err(|error| error.to_string())?;
        let resolved = assemble_empty_resolved_typed_ast(&typed_ast, Vec::new())?;
        let output = SourceProofLocalGivenConditionUseTermRouteOutput {
            typed_ast,
            resolved,
        };
        let _ = (output.typed_ast(), output.resolved());
        Ok(output)
    }))
}

fn task269gcu_term_input(
    source_id: mizar_session::SourceId,
    module_id: ModuleId,
) -> SourcePrimaryTermHandoffInput {
    SourcePrimaryTermHandoffInput {
        source_id,
        module_id,
        terms: [(3, 107, 108), (4, 111, 112)]
            .into_iter()
            .enumerate()
            .map(
                |(source_ordinal, (node, start, end))| SourcePrimaryTermInput {
                    site: TypedSiteRef::Node(TypedNodeId::new(node)),
                    source_range: SourceRange {
                        source_id,
                        start,
                        end,
                    },
                    source_ordinal,
                    context: BindingContextId::new(1),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: "y".to_owned(),
                    kind: SourcePrimaryTermKind::VariableReference,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                },
            )
            .collect(),
        references: (0..2)
            .map(|index| SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(index),
                binding: mizar_checker::binding_env::BindingId::new(1),
                role: SourcePrimaryTermReferenceRole::Variable,
            })
            .collect(),
        numeric_type_requests: Vec::new(),
    }
}

fn task269gcu_arena(
    source_id: mizar_session::SourceId,
    mutation: SourceProofLocalGivenConditionUseTermRouteMutation,
) -> Result<TypedArena, String> {
    let mut builder = TypedArenaBuilder::new();
    let reserve = builder
        .push(TypedNode::new(
            "source.proof-local.given-condition.reserve-type",
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 14,
                end: 17,
            }),
        ))
        .map_err(|error| error.to_string())?;
    let local = builder
        .push(TypedNode::new(
            "source.proof-local.given-condition.type",
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 90,
                end: 93,
            }),
        ))
        .map_err(|error| error.to_string())?;
    let type_root = builder
        .push(
            TypedNode::new(
                "source.proof-local.given-condition.type-root",
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 0,
                    end: 133,
                }),
            )
            .with_children(vec![reserve, local]),
        )
        .map_err(|error| error.to_string())?;
    let first = builder
        .push(TypedNode::new(
            "source.term.variable-reference",
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 107,
                end: 108,
            }),
        ))
        .map_err(|error| error.to_string())?;
    let second = builder
        .push(TypedNode::new(
            "source.term.variable-reference",
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 111,
                end: 112,
            }),
        ))
        .map_err(|error| error.to_string())?;
    let root = builder
        .push(
            TypedNode::new(
                if mutation == SourceProofLocalGivenConditionUseTermRouteMutation::WrongArenaKind {
                    "source.proof-local.given-condition.term-root.wrong"
                } else {
                    "source.proof-local.given-condition.term-root"
                },
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 0,
                    end: 133,
                }),
            )
            .with_children(vec![type_root, first, second]),
        )
        .map_err(|error| error.to_string())?;
    builder
        .finish(Some(
            if mutation == SourceProofLocalGivenConditionUseTermRouteMutation::WrongArenaRoot {
                second
            } else {
                root
            },
        ))
        .map_err(|error| error.to_string())
}

fn empty_task269gcu_typed_ast(
    source_id: mizar_session::SourceId,
    module_id: ModuleId,
    nodes: TypedArena,
) -> Result<TypedAst, String> {
    TypedAst::try_new(TypedAstParts {
        source_id,
        module_id,
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: Task 269CT is a private dormant runner consumer until activation.
pub(in crate::runner) struct SourceProofLocalLetTypeRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; other variants are private corruption seams.
pub(in crate::runner) enum SourceProofLocalLetTypeRouteMutation {
    None,
    WrongDependencyModule,
    WrongTypeRange,
    WrongArenaRoot,
    WrongArenaKind,
}

#[allow(dead_code)] // Rationale: Task 269CT deliberately leaves this exact private leaf dormant.
pub(in crate::runner) fn source_proof_local_let_type_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalLetTypeRouteOutput, String>> {
    source_proof_local_let_type_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceProofLocalLetTypeRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_let_type_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalLetTypeRouteMutation,
) -> Option<Result<SourceProofLocalLetTypeRouteOutput, String>> {
    source_proof_local_let_type_output_impl(ast, module, shells, symbols, source_text, mutation)
}

fn source_proof_local_let_type_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalLetTypeRouteMutation,
) -> Option<Result<SourceProofLocalLetTypeRouteOutput, String>> {
    let binding =
        source_proof_local_let_binding_output(ast, module.clone(), shells, symbols, source_text)?;
    let lower =
        source_proof_local_let_lower_output(ast, module.clone(), shells, symbols, source_text)?;
    Some(binding.and_then(|binding| {
        lower.and_then(|lower| {
            let dependency = binding
                .typed_ast
                .source_proof_local_let_binding()
                .cloned()
                .ok_or_else(|| "Task269CT Task269C dependency is missing".to_owned())?;
            let reserve_range = match dependency
                .binding_env()
                .bindings()
                .get(mizar_checker::binding_env::BindingId::new(0))
                .map(|binding| &binding.type_site)
            {
                Some(mizar_checker::binding_env::BindingTypeSite::Source(source_range)) => {
                    *source_range
                }
                _ => return Err("Task269CT reserve type range is missing".to_owned()),
            };
            let local_range = lower.type_range();
            let arena = task269ct_arena(ast.source_id, mutation)?;
            let mut input =
                task269ct_type_input(ast.source_id, module.clone(), reserve_range, local_range);
            if mutation == SourceProofLocalLetTypeRouteMutation::WrongDependencyModule {
                input.module_id =
                    ModuleId::new(module.package().clone(), ModulePath::new("task269ct.wrong"));
                for expression in &mut input.expressions {
                    expression.module_id = input.module_id.clone();
                }
            }
            if mutation == SourceProofLocalLetTypeRouteMutation::WrongTypeRange {
                input.expressions[1].source_range.end += 1;
            }
            let handoff =
                SourceProofLocalLetTypeProducer::build(dependency, input, symbols, &arena)
                    .map_err(|error| error.to_string())?;
            let typed_ast = empty_task269ct_typed_ast(ast.source_id, module, arena)?
                .with_source_proof_local_let_type(handoff)
                .map_err(|error| error.to_string())?;
            let resolved = assemble_empty_resolved_typed_ast(&typed_ast, Vec::new())?;
            Ok(SourceProofLocalLetTypeRouteOutput {
                typed_ast,
                resolved,
            })
        })
    }))
}

fn task269ct_type_input(
    source_id: mizar_session::SourceId,
    module_id: ModuleId,
    reserve_range: SourceRange,
    local_range: SourceRange,
) -> SourceTypeHandoffInput {
    let ranges = [reserve_range, local_range];
    SourceTypeHandoffInput {
        source_id,
        module_id: module_id.clone(),
        applications: ranges
            .iter()
            .enumerate()
            .map(|(index, _)| SourceTypeApplicationInput {
                binding: mizar_checker::binding_env::BindingId::new(index),
                source_ordinal: index,
                root: SourceTypeExpressionId::new(index),
            })
            .collect(),
        expressions: ranges
            .into_iter()
            .enumerate()
            .map(|(index, source_range)| SourceTypeExpressionInput {
                source_id,
                module_id: module_id.clone(),
                site: TypedSiteRef::Role {
                    node: TypedNodeId::new(index),
                    role: TypeRole::new("source.type.expression"),
                },
                source_range,
                spelling: "set".to_owned(),
                head_site: TypedSiteRef::Role {
                    node: TypedNodeId::new(index),
                    role: TypeRole::new("source.type.head"),
                },
                head_range: source_range,
                head_spelling: "set".to_owned(),
                form: SourceTypeApplicationForm::Bare,
                head: SourceTypeHead::BuiltinSet,
                recovery: mizar_checker::typed_ast::NodeRecoveryState::Normal,
            })
            .collect(),
        arguments: Vec::new(),
    }
}

fn task269ct_arena(
    source_id: mizar_session::SourceId,
    mutation: SourceProofLocalLetTypeRouteMutation,
) -> Result<TypedArena, String> {
    let mut builder = TypedArenaBuilder::new();
    let reserve = builder
        .push(TypedNode::new(
            "source.proof-local.let.reserve-type",
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 14,
                end: 17,
            }),
        ))
        .map_err(|error| error.to_string())?;
    let local = builder
        .push(TypedNode::new(
            if mutation == SourceProofLocalLetTypeRouteMutation::WrongArenaKind {
                "source.proof-local.let.type.wrong"
            } else {
                "source.proof-local.let.type"
            },
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 76,
                end: 79,
            }),
        ))
        .map_err(|error| error.to_string())?;
    let root = builder
        .push(
            TypedNode::new(
                "source.proof-local.let.type-root",
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 0,
                    end: 99,
                }),
            )
            .with_children(vec![reserve, local]),
        )
        .map_err(|error| error.to_string())?;
    builder
        .finish(Some(
            if mutation == SourceProofLocalLetTypeRouteMutation::WrongArenaRoot {
                local
            } else {
                root
            },
        ))
        .map_err(|error| error.to_string())
}

fn empty_task269ct_typed_ast(
    source_id: mizar_session::SourceId,
    module_id: ModuleId,
    nodes: TypedArena,
) -> Result<TypedAst, String> {
    TypedAst::try_new(TypedAstParts {
        source_id,
        module_id,
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: Task 269GT is a private dormant runner consumer until activation.
pub(in crate::runner) struct SourceProofLocalGivenTypeRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; other variants are private corruption seams.
pub(in crate::runner) enum SourceProofLocalGivenTypeRouteMutation {
    None,
    WrongDependencyModule,
    WrongTypeRange,
    WrongArenaRoot,
    WrongArenaKind,
}

#[allow(dead_code)] // Rationale: Task 269GT deliberately leaves this exact private leaf dormant.
pub(in crate::runner) fn source_proof_local_given_type_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalGivenTypeRouteOutput, String>> {
    source_proof_local_given_type_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceProofLocalGivenTypeRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_type_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenTypeRouteMutation,
) -> Option<Result<SourceProofLocalGivenTypeRouteOutput, String>> {
    source_proof_local_given_type_output_impl(ast, module, shells, symbols, source_text, mutation)
}

fn source_proof_local_given_type_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenTypeRouteMutation,
) -> Option<Result<SourceProofLocalGivenTypeRouteOutput, String>> {
    let binding =
        source_proof_local_given_binding_output(ast, module.clone(), shells, symbols, source_text)?;
    let lower =
        source_proof_local_given_lower_output(ast, module.clone(), shells, symbols, source_text)?;
    Some(binding.and_then(|binding| {
        lower.and_then(|lower| {
            let dependency = binding
                .typed_ast
                .source_proof_local_given_binding()
                .cloned()
                .ok_or_else(|| "Task269GT Task269G dependency is missing".to_owned())?;
            let reserve_range = match dependency
                .binding_env()
                .bindings()
                .get(mizar_checker::binding_env::BindingId::new(0))
                .map(|binding| &binding.type_site)
            {
                Some(mizar_checker::binding_env::BindingTypeSite::Source(source_range)) => {
                    *source_range
                }
                _ => return Err("Task269GT reserve type range is missing".to_owned()),
            };
            let local_range = lower.type_range();
            let arena = task269gt_arena(ast.source_id, mutation)?;
            let mut input =
                task269gt_type_input(ast.source_id, module.clone(), reserve_range, local_range);
            if mutation == SourceProofLocalGivenTypeRouteMutation::WrongDependencyModule {
                input.module_id =
                    ModuleId::new(module.package().clone(), ModulePath::new("task269gt.wrong"));
                for expression in &mut input.expressions {
                    expression.module_id = input.module_id.clone();
                }
            }
            if mutation == SourceProofLocalGivenTypeRouteMutation::WrongTypeRange {
                input.expressions[1].source_range.end += 1;
            }
            let handoff =
                SourceProofLocalGivenTypeProducer::build(dependency, input, symbols, &arena)
                    .map_err(|error| error.to_string())?;
            let typed_ast = empty_task269gt_typed_ast(ast.source_id, module, arena)?
                .with_source_proof_local_given_type(handoff)
                .map_err(|error| error.to_string())?;
            let resolved = assemble_empty_resolved_typed_ast(&typed_ast, Vec::new())?;
            Ok(SourceProofLocalGivenTypeRouteOutput {
                typed_ast,
                resolved,
            })
        })
    }))
}

fn task269gt_type_input(
    source_id: mizar_session::SourceId,
    module_id: ModuleId,
    reserve_range: SourceRange,
    local_range: SourceRange,
) -> SourceTypeHandoffInput {
    let ranges = [reserve_range, local_range];
    SourceTypeHandoffInput {
        source_id,
        module_id: module_id.clone(),
        applications: ranges
            .iter()
            .enumerate()
            .map(|(index, _)| SourceTypeApplicationInput {
                binding: mizar_checker::binding_env::BindingId::new(index),
                source_ordinal: index,
                root: SourceTypeExpressionId::new(index),
            })
            .collect(),
        expressions: ranges
            .into_iter()
            .enumerate()
            .map(|(index, source_range)| SourceTypeExpressionInput {
                source_id,
                module_id: module_id.clone(),
                site: TypedSiteRef::Role {
                    node: TypedNodeId::new(index),
                    role: TypeRole::new("source.type.expression"),
                },
                source_range,
                spelling: "set".to_owned(),
                head_site: TypedSiteRef::Role {
                    node: TypedNodeId::new(index),
                    role: TypeRole::new("source.type.head"),
                },
                head_range: source_range,
                head_spelling: "set".to_owned(),
                form: SourceTypeApplicationForm::Bare,
                head: SourceTypeHead::BuiltinSet,
                recovery: mizar_checker::typed_ast::NodeRecoveryState::Normal,
            })
            .collect(),
        arguments: Vec::new(),
    }
}

fn task269gt_arena(
    source_id: mizar_session::SourceId,
    mutation: SourceProofLocalGivenTypeRouteMutation,
) -> Result<TypedArena, String> {
    let mut builder = TypedArenaBuilder::new();
    let reserve = builder
        .push(TypedNode::new(
            "source.proof-local.given.reserve-type",
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 14,
                end: 17,
            }),
        ))
        .map_err(|error| error.to_string())?;
    let local = builder
        .push(TypedNode::new(
            if mutation == SourceProofLocalGivenTypeRouteMutation::WrongArenaKind {
                "source.proof-local.given.type.wrong"
            } else {
                "source.proof-local.given.type"
            },
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 84,
                end: 87,
            }),
        ))
        .map_err(|error| error.to_string())?;
    let root = builder
        .push(
            TypedNode::new(
                "source.proof-local.given.type-root",
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 0,
                    end: 128,
                }),
            )
            .with_children(vec![reserve, local]),
        )
        .map_err(|error| error.to_string())?;
    builder
        .finish(Some(
            if mutation == SourceProofLocalGivenTypeRouteMutation::WrongArenaRoot {
                local
            } else {
                root
            },
        ))
        .map_err(|error| error.to_string())
}

fn empty_task269gt_typed_ast(
    source_id: mizar_session::SourceId,
    module_id: ModuleId,
    nodes: TypedArena,
) -> Result<TypedAst, String> {
    TypedAst::try_new(TypedAstParts {
        source_id,
        module_id,
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: Task 269GUPT is a private dormant runner consumer until activation.
pub(in crate::runner) struct SourceProofLocalGivenUseTypeRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; other variants are private corruption seams.
pub(in crate::runner) enum SourceProofLocalGivenUseTypeRouteMutation {
    None,
    WrongDependencyModule,
    WrongTypeRange,
    WrongArenaRoot,
    WrongArenaKind,
}

#[allow(dead_code)] // Rationale: Task 269GUPT deliberately leaves this exact private leaf dormant.
pub(in crate::runner) fn source_proof_local_given_use_type_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalGivenUseTypeRouteOutput, String>> {
    source_proof_local_given_use_type_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceProofLocalGivenUseTypeRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_use_type_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenUseTypeRouteMutation,
) -> Option<Result<SourceProofLocalGivenUseTypeRouteOutput, String>> {
    source_proof_local_given_use_type_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        mutation,
    )
}

fn source_proof_local_given_use_type_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenUseTypeRouteMutation,
) -> Option<Result<SourceProofLocalGivenUseTypeRouteOutput, String>> {
    let binding = source_proof_local_given_use_binding_output(
        ast,
        module.clone(),
        shells,
        symbols,
        source_text,
    )?;
    let lower = source_proof_local_given_use_lower_output(
        ast,
        module.clone(),
        shells,
        symbols,
        source_text,
    )?;
    Some(binding.and_then(|dependency| {
        lower.and_then(|lower| {
            let reserve_range = match dependency
                .binding_env()
                .bindings()
                .get(mizar_checker::binding_env::BindingId::new(0))
                .map(|binding| &binding.type_site)
            {
                Some(mizar_checker::binding_env::BindingTypeSite::Source(source_range)) => {
                    *source_range
                }
                _ => return Err("Task269GUPT reserve type range is missing".to_owned()),
            };
            let local_range = lower.type_range();
            let arena = task269gupt_arena(ast.source_id, mutation)?;
            let mut input =
                task269gupt_type_input(ast.source_id, module.clone(), reserve_range, local_range);
            if mutation == SourceProofLocalGivenUseTypeRouteMutation::WrongDependencyModule {
                input.module_id = ModuleId::new(
                    module.package().clone(),
                    ModulePath::new("task269gupt.wrong"),
                );
                for expression in &mut input.expressions {
                    expression.module_id = input.module_id.clone();
                }
            }
            if mutation == SourceProofLocalGivenUseTypeRouteMutation::WrongTypeRange {
                input.expressions[1].source_range.end += 1;
            }
            let handoff =
                SourceProofLocalGivenUseTypeProducer::build(dependency, input, symbols, &arena)
                    .map_err(|error| error.to_string())?;
            let typed_ast = empty_task269gupt_typed_ast(ast.source_id, module, arena)?
                .with_source_proof_local_given_use_type(handoff)
                .map_err(|error| error.to_string())?;
            let resolved = assemble_empty_resolved_typed_ast(&typed_ast, Vec::new())?;
            Ok(SourceProofLocalGivenUseTypeRouteOutput {
                typed_ast,
                resolved,
            })
        })
    }))
}

fn task269gupt_type_input(
    source_id: mizar_session::SourceId,
    module_id: ModuleId,
    reserve_range: SourceRange,
    local_range: SourceRange,
) -> SourceTypeHandoffInput {
    let ranges = [reserve_range, local_range];
    SourceTypeHandoffInput {
        source_id,
        module_id: module_id.clone(),
        applications: ranges
            .iter()
            .enumerate()
            .map(|(index, _)| SourceTypeApplicationInput {
                binding: mizar_checker::binding_env::BindingId::new(index),
                source_ordinal: index,
                root: SourceTypeExpressionId::new(index),
            })
            .collect(),
        expressions: ranges
            .into_iter()
            .enumerate()
            .map(|(index, source_range)| SourceTypeExpressionInput {
                source_id,
                module_id: module_id.clone(),
                site: TypedSiteRef::Role {
                    node: TypedNodeId::new(index),
                    role: TypeRole::new("source.type.expression"),
                },
                source_range,
                spelling: "set".to_owned(),
                head_site: TypedSiteRef::Role {
                    node: TypedNodeId::new(index),
                    role: TypeRole::new("source.type.head"),
                },
                head_range: source_range,
                head_spelling: "set".to_owned(),
                form: SourceTypeApplicationForm::Bare,
                head: SourceTypeHead::BuiltinSet,
                recovery: mizar_checker::typed_ast::NodeRecoveryState::Normal,
            })
            .collect(),
        arguments: Vec::new(),
    }
}

fn task269gupt_arena(
    source_id: mizar_session::SourceId,
    mutation: SourceProofLocalGivenUseTypeRouteMutation,
) -> Result<TypedArena, String> {
    let mut builder = TypedArenaBuilder::new();
    let reserve = builder
        .push(TypedNode::new(
            "source.proof-local.given-use.reserve-type",
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 14,
                end: 17,
            }),
        ))
        .map_err(|error| error.to_string())?;
    let local = builder
        .push(TypedNode::new(
            if mutation == SourceProofLocalGivenUseTypeRouteMutation::WrongArenaKind {
                "source.proof-local.given-use.type.wrong"
            } else {
                "source.proof-local.given-use.type"
            },
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 84,
                end: 87,
            }),
        ))
        .map_err(|error| error.to_string())?;
    let root = builder
        .push(
            TypedNode::new(
                "source.proof-local.given-use.type-root",
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 0,
                    end: 127,
                }),
            )
            .with_children(vec![reserve, local]),
        )
        .map_err(|error| error.to_string())?;
    builder
        .finish(Some(
            if mutation == SourceProofLocalGivenUseTypeRouteMutation::WrongArenaRoot {
                local
            } else {
                root
            },
        ))
        .map_err(|error| error.to_string())
}

fn empty_task269gupt_typed_ast(
    source_id: mizar_session::SourceId,
    module_id: ModuleId,
    nodes: TypedArena,
) -> Result<TypedAst, String> {
    TypedAst::try_new(TypedAstParts {
        source_id,
        module_id,
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: Task 269GU is a private dormant runner consumer until activation.
pub(in crate::runner) struct SourceProofLocalGivenUseTermRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; other variants are private corruption seams.
pub(in crate::runner) enum SourceProofLocalGivenUseTermRouteMutation {
    None,
    WrongDependencyModule,
    WrongTermRange,
    WrongReferenceBinding,
    WrongArenaRoot,
    WrongArenaKind,
}

#[allow(dead_code)] // Rationale: Task 269GU deliberately leaves this exact private leaf dormant.
pub(in crate::runner) fn source_proof_local_given_use_term_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalGivenUseTermRouteOutput, String>> {
    source_proof_local_given_use_term_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceProofLocalGivenUseTermRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_use_term_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenUseTermRouteMutation,
) -> Option<Result<SourceProofLocalGivenUseTermRouteOutput, String>> {
    source_proof_local_given_use_term_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        mutation,
    )
}

fn source_proof_local_given_use_term_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenUseTermRouteMutation,
) -> Option<Result<SourceProofLocalGivenUseTermRouteOutput, String>> {
    let dependency = source_proof_local_given_use_type_output(
        ast,
        module.clone(),
        shells,
        symbols,
        source_text,
    )?;
    Some(dependency.and_then(|dependency| {
        let dependency = dependency
            .typed_ast
            .source_proof_local_given_use_type()
            .cloned()
            .ok_or_else(|| "Task269GU GUPT dependency is missing".to_owned())?;
        let arena = task269gu_arena(ast.source_id, mutation)?;
        let mut input = task269gu_term_input(ast.source_id, module.clone());
        match mutation {
            SourceProofLocalGivenUseTermRouteMutation::WrongDependencyModule => {
                input.module_id =
                    ModuleId::new(module.package().clone(), ModulePath::new("task269gu.wrong"));
            }
            SourceProofLocalGivenUseTermRouteMutation::WrongTermRange => {
                input.terms[0].source_range.end += 1;
            }
            SourceProofLocalGivenUseTermRouteMutation::WrongReferenceBinding => {
                input.references[0].binding = mizar_checker::binding_env::BindingId::new(0);
            }
            SourceProofLocalGivenUseTermRouteMutation::None
            | SourceProofLocalGivenUseTermRouteMutation::WrongArenaRoot
            | SourceProofLocalGivenUseTermRouteMutation::WrongArenaKind => {}
        }
        let handoff = SourceProofLocalGivenUseTermProducer::build(dependency, input, &arena)
            .map_err(|error| error.to_string())?;
        let typed_ast = empty_task269gu_typed_ast(ast.source_id, module, arena)?
            .with_source_proof_local_given_use_term(handoff)
            .map_err(|error| error.to_string())?;
        let resolved = assemble_empty_resolved_typed_ast(&typed_ast, Vec::new())?;
        Ok(SourceProofLocalGivenUseTermRouteOutput {
            typed_ast,
            resolved,
        })
    }))
}

fn task269gu_term_input(
    source_id: mizar_session::SourceId,
    module_id: ModuleId,
) -> SourcePrimaryTermHandoffInput {
    SourcePrimaryTermHandoffInput {
        source_id,
        module_id,
        terms: [(3, 116, 117), (4, 120, 121)]
            .into_iter()
            .enumerate()
            .map(
                |(source_ordinal, (node, start, end))| SourcePrimaryTermInput {
                    site: TypedSiteRef::Node(TypedNodeId::new(node)),
                    source_range: SourceRange {
                        source_id,
                        start,
                        end,
                    },
                    source_ordinal,
                    context: BindingContextId::new(1),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: "y".to_owned(),
                    kind: SourcePrimaryTermKind::VariableReference,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                },
            )
            .collect(),
        references: (0..2)
            .map(|index| SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(index),
                binding: mizar_checker::binding_env::BindingId::new(1),
                role: SourcePrimaryTermReferenceRole::Variable,
            })
            .collect(),
        numeric_type_requests: Vec::new(),
    }
}

fn task269gu_arena(
    source_id: mizar_session::SourceId,
    mutation: SourceProofLocalGivenUseTermRouteMutation,
) -> Result<TypedArena, String> {
    let mut builder = TypedArenaBuilder::new();
    let reserve = builder
        .push(TypedNode::new(
            "source.proof-local.given-use.reserve-type",
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 14,
                end: 17,
            }),
        ))
        .map_err(|error| error.to_string())?;
    let local = builder
        .push(TypedNode::new(
            "source.proof-local.given-use.type",
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 84,
                end: 87,
            }),
        ))
        .map_err(|error| error.to_string())?;
    let type_root = builder
        .push(
            TypedNode::new(
                "source.proof-local.given-use.type-root",
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 0,
                    end: 127,
                }),
            )
            .with_children(vec![reserve, local]),
        )
        .map_err(|error| error.to_string())?;
    let first = builder
        .push(TypedNode::new(
            "source.term.variable-reference",
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 116,
                end: 117,
            }),
        ))
        .map_err(|error| error.to_string())?;
    let second = builder
        .push(TypedNode::new(
            "source.term.variable-reference",
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 120,
                end: 121,
            }),
        ))
        .map_err(|error| error.to_string())?;
    let root = builder
        .push(
            TypedNode::new(
                if mutation == SourceProofLocalGivenUseTermRouteMutation::WrongArenaKind {
                    "source.proof-local.given-use.term-root.wrong"
                } else {
                    "source.proof-local.given-use.term-root"
                },
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 0,
                    end: 127,
                }),
            )
            .with_children(vec![type_root, first, second]),
        )
        .map_err(|error| error.to_string())?;
    builder
        .finish(Some(
            if mutation == SourceProofLocalGivenUseTermRouteMutation::WrongArenaRoot {
                second
            } else {
                root
            },
        ))
        .map_err(|error| error.to_string())
}

fn empty_task269gu_typed_ast(
    source_id: mizar_session::SourceId,
    module_id: ModuleId,
    nodes: TypedArena,
) -> Result<TypedAst, String> {
    TypedAst::try_new(TypedAstParts {
        source_id,
        module_id,
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())
}
