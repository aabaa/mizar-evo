use mizar_checker::{
    binding_env::BindingContextId,
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_proof_local_declaration::{
        SourceProofLocalDeclarationHandoffInput, SourceProofLocalDeclarationInput,
        SourceProofLocalDeclarationKind, SourceProofLocalDeclarationProducer,
        SourceProofLocalDeclarationRecovery,
    },
    source_statement::{
        SourceStatementWitnessId, SourceStatementWitnessNameId, SourceStatementWitnessTermTarget,
    },
    source_term::SourcePrimaryTermId,
    typed_ast::TypedAst,
};
#[cfg(test)]
use mizar_checker::{
    source_term::{SourcePrimaryTermHandoffInput, SourcePrimaryTermProducer},
    typed_ast::TypedArena,
};
use mizar_resolve::{
    env::SymbolEnv,
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::ModuleId,
};
use mizar_session::SourceRange;
use mizar_syntax::SurfaceAst;

use super::{
    checker_handoff::assemble_empty_resolved_typed_ast,
    source_statement::{
        SOURCE_STATEMENT_B3N_TEXT, SourceStatementRouteOutput,
        extract_named_witness_source_statement, source_statement_output_with_source,
    },
};

#[cfg(test)]
use super::source_statement::source_statement_b3n_output_with_mutation;

#[derive(Debug)]
#[allow(dead_code)] // Rationale: Task 269A is a private dormant consumer until a later activation task.
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
    WrongLowerArena,
}

#[allow(dead_code)] // Rationale: Task 269A deliberately leaves this exact private leaf dormant.
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
    let extracted = extract_named_witness_source_statement(ast, source_text)?;
    if source_text != SOURCE_STATEMENT_B3N_TEXT
        || extracted.name_site.node().index() != 13
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

    let lower = lower_output(ast, module, symbols, source_text, mutation)?;
    Some(lower.and_then(|lower| {
        build_source_proof_local_declaration_output(ast, extracted.name_range, lower, mutation)
    }))
}

fn lower_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalDeclarationRouteMutation,
) -> Option<Result<SourceStatementRouteOutput, String>> {
    #[cfg(not(test))]
    let _ = mutation;
    #[cfg(test)]
    match mutation {
        SourceProofLocalDeclarationRouteMutation::WrongLowerStatement => {
            return source_statement_b3n_output_with_mutation(
                ast,
                module,
                symbols,
                source_text,
                |input| input.statement.statements[1].source_ordinal = 3,
            );
        }
        SourceProofLocalDeclarationRouteMutation::WrongLowerWitness => {
            return source_statement_b3n_output_with_mutation(
                ast,
                module,
                symbols,
                source_text,
                |input| input.witness.names[0].spelling = "z".to_owned(),
            );
        }
        SourceProofLocalDeclarationRouteMutation::WrongLowerPrimary => {
            return source_statement_b3n_output_with_mutation(
                ast,
                module,
                symbols,
                source_text,
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
        SourceProofLocalDeclarationRouteMutation::WrongLowerArena => {
            return source_statement_b3n_output_with_mutation(
                ast,
                module,
                symbols,
                source_text,
                |input| {
                    let mut nodes = input
                        .arena
                        .iter()
                        .map(|(_, node)| node.clone())
                        .collect::<Vec<_>>();
                    nodes[13].kind = "source.task269a.corrupt".into();
                    input.arena = TypedArena::try_new(input.arena.root(), nodes)
                        .expect("Task269A arena corruption remains structurally valid");
                },
            );
        }
        _ => {}
    }

    source_statement_output_with_source(ast, module, symbols, source_text)
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
