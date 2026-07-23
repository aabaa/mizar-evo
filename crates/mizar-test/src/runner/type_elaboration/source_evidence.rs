#[cfg(test)]
use mizar_checker::resolved_typed_ast::ResolvedTypedAst;
use mizar_checker::{
    resolved_typed_ast::{ResolvedNodeKindHint, ResolvedNodeKindHintKind, SourceNodeRole},
    source_attribute::SourceAttributeHandoff,
    source_evidence::{
        SourceEvidenceDependencyCatalog, SourceEvidenceDependencyRecord,
        SourceEvidenceHandoffInput, SourceEvidenceInputState, SourceEvidenceProducer,
        SourceEvidenceRecovery, SourceEvidenceRequestInput, SourceEvidenceRequestKind,
        SourceEvidenceRequestOrigin,
    },
    source_type::{SourceTypeApplicationHandoff, SourceTypeHead},
    typed_ast::{NodeRecoveryState, TypedAst},
};
use mizar_resolve::{
    env::{SymbolEnv, SymbolKind},
    resolved_ast::ModuleId,
};
use mizar_syntax::SurfaceAst;

use super::{
    checker_handoff::assemble_empty_resolved_typed_ast,
    source_attribute::{SourceAttributeRouteOutput, source_attribute_evidence_output},
    source_type::{SourceTypeRouteOutput, source_type_application_output},
};

const MISSING_KEY: &str = "type_elaboration.checker.source_evidence.dependency_input_missing";
const INVALID_PAYLOAD_KEY: &str = "type_elaboration.checker.source_evidence.invalid_payload";
const EVIDENCE_QUERY_KEY: &str =
    "type_elaboration.checker.checker.declaration.deferred.evidence_query";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SourceEvidenceRouteKind {
    SourceType,
    AttributedType,
}

#[derive(Debug)]
pub(in crate::runner) struct SourceEvidenceRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    #[cfg(test)]
    pub(in crate::runner) resolved: ResolvedTypedAst,
    pub(in crate::runner) kind: SourceEvidenceRouteKind,
}

pub(in crate::runner) fn source_evidence_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    source_evidence_output(ast, module, symbols).map(|result| match result {
        Ok(output) => match output.kind {
            SourceEvidenceRouteKind::SourceType => vec![MISSING_KEY.to_owned()],
            SourceEvidenceRouteKind::AttributedType => {
                let mut keys = output
                    .typed_ast
                    .diagnostics()
                    .canonical_iter()
                    .map(|(_, diagnostic)| {
                        format!("type_elaboration.checker.{}", diagnostic.message_key)
                    })
                    .collect::<Vec<_>>();
                keys.sort();
                keys.dedup();
                if keys == [EVIDENCE_QUERY_KEY] {
                    keys
                } else {
                    vec![INVALID_PAYLOAD_KEY.to_owned()]
                }
            }
        },
        Err(_) => vec![INVALID_PAYLOAD_KEY.to_owned()],
    })
}

pub(in crate::runner) fn source_evidence_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Result<SourceEvidenceRouteOutput, String>> {
    source_evidence_output_with_mutation(ast, module, symbols, |_, _| {})
}

#[cfg(test)]
pub(in crate::runner) fn source_evidence_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceEvidenceHandoffInput, &mut Vec<SourceEvidenceDependencyRecord>),
) -> Option<Result<SourceEvidenceRouteOutput, String>> {
    source_evidence_output_with_mutation_impl(ast, module, symbols, mutate)
}

#[cfg(not(test))]
fn source_evidence_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceEvidenceHandoffInput, &mut Vec<SourceEvidenceDependencyRecord>),
) -> Option<Result<SourceEvidenceRouteOutput, String>> {
    source_evidence_output_with_mutation_impl(ast, module, symbols, mutate)
}

fn source_evidence_output_with_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceEvidenceHandoffInput, &mut Vec<SourceEvidenceDependencyRecord>),
) -> Option<Result<SourceEvidenceRouteOutput, String>> {
    let base = if let Some(output) = source_type_application_output(ast, module.clone(), symbols) {
        output.map(|output| BaseRouteOutput::SourceType(Box::new(output)))
    } else {
        source_attribute_evidence_output(ast, module, symbols)?
            .map(|output| BaseRouteOutput::AttributedType(Box::new(output)))
    };
    Some(base.and_then(|base| build_output(base, symbols, mutate)))
}

#[derive(Debug)]
enum BaseRouteOutput {
    SourceType(Box<SourceTypeRouteOutput>),
    AttributedType(Box<SourceAttributeRouteOutput>),
}

impl BaseRouteOutput {
    const fn kind(&self) -> SourceEvidenceRouteKind {
        match self {
            Self::SourceType(_) => SourceEvidenceRouteKind::SourceType,
            Self::AttributedType(_) => SourceEvidenceRouteKind::AttributedType,
        }
    }

    fn into_typed_ast(self) -> TypedAst {
        match self {
            Self::SourceType(output) => output.typed_ast,
            Self::AttributedType(output) => output.typed_ast,
        }
    }
}

fn build_output(
    base: BaseRouteOutput,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceEvidenceHandoffInput, &mut Vec<SourceEvidenceDependencyRecord>),
) -> Result<SourceEvidenceRouteOutput, String> {
    let kind = base.kind();
    let typed_ast = base.into_typed_ast();
    let source_type = typed_ast
        .source_type()
        .ok_or_else(|| "source-evidence requires Task 249".to_owned())?;
    let mut input = source_evidence_input(source_type, typed_ast.source_attribute(), symbols)?;
    let mut records = Vec::new();
    mutate(&mut input, &mut records);
    let catalog = SourceEvidenceDependencyCatalog::new(records);
    let source_evidence = SourceEvidenceProducer::build(
        input,
        source_type,
        typed_ast.source_attribute(),
        symbols,
        typed_ast.facts(),
        &catalog,
    )
    .map_err(|error| error.to_string())?;
    let typed_ast = typed_ast
        .with_source_evidence(source_evidence)
        .map_err(|error| error.to_string())?;
    let role = match kind {
        SourceEvidenceRouteKind::SourceType => "source.type.surface",
        SourceEvidenceRouteKind::AttributedType => "source.attribute.surface",
    };
    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new(role),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)?;
    if typed_ast.source_evidence().is_none()
        || resolved.source_evidence() != typed_ast.source_evidence()
    {
        return Err("source-evidence immutable final handoff mismatch".to_owned());
    }
    Ok(SourceEvidenceRouteOutput {
        typed_ast,
        #[cfg(test)]
        resolved,
        kind,
    })
}

fn source_evidence_input(
    source_type: &SourceTypeApplicationHandoff,
    source_attribute: Option<&SourceAttributeHandoff>,
    symbols: &SymbolEnv,
) -> Result<SourceEvidenceHandoffInput, String> {
    let mut requests = Vec::new();
    for (application_id, application) in source_type.applications().iter() {
        let expression = source_type
            .expressions()
            .get(application.root())
            .ok_or_else(|| "source-evidence application root disappeared".to_owned())?;
        let chain = source_attribute.and_then(|handoff| {
            handoff.chains().iter().find_map(|(id, chain)| {
                (chain.expression() == expression.id()).then_some((id, chain))
            })
        });
        let (site, source_range, recovery, kind, attribute_chain) = if let Some((chain_id, chain)) =
            chain
        {
            (
                chain.site().clone(),
                chain.source_range(),
                source_evidence_recovery(chain.recovery()),
                SourceEvidenceRequestKind::AttributedTypeInhabitation,
                Some(chain_id),
            )
        } else {
            let kind = match expression.head() {
                SourceTypeHead::BuiltinSet | SourceTypeHead::BuiltinObject => continue,
                SourceTypeHead::Symbol { symbol, .. } => match symbols_kind(symbols, symbol)? {
                    SymbolKind::Mode => SourceEvidenceRequestKind::ModeExpansion,
                    SymbolKind::Structure => SourceEvidenceRequestKind::StructureInhabitation,
                    _ => {
                        return Err("source-evidence head is not a mode or structure".to_owned());
                    }
                },
                _ => return Err("source-evidence head is outside Task 251".to_owned()),
            };
            (
                expression.head_site().clone(),
                expression.source_range(),
                source_evidence_recovery(expression.recovery()),
                kind,
                None,
            )
        };
        requests.push(SourceEvidenceRequestInput {
            owner: expression.site().clone(),
            site,
            source_range,
            source_ordinal: application.source_ordinal(),
            recovery,
            kind,
            state: SourceEvidenceInputState::Missing,
            origin: SourceEvidenceRequestOrigin::SourceTypeApplication {
                application: application_id,
                expression: expression.id(),
                attribute_chain,
            },
        });
    }
    Ok(SourceEvidenceHandoffInput {
        source_id: source_type.source_id(),
        module_id: source_type.module_id().clone(),
        requests,
        responses: Vec::new(),
    })
}

fn symbols_kind(
    symbols: &SymbolEnv,
    symbol: &mizar_resolve::resolved_ast::SymbolId,
) -> Result<SymbolKind, String> {
    symbols
        .symbols()
        .get(symbol)
        .map(|entry| entry.kind())
        .ok_or_else(|| "source-evidence symbol disappeared".to_owned())
}

const fn source_evidence_recovery(recovery: NodeRecoveryState) -> SourceEvidenceRecovery {
    match recovery {
        NodeRecoveryState::Normal => SourceEvidenceRecovery::Normal,
        NodeRecoveryState::Recovered | NodeRecoveryState::Degraded => {
            SourceEvidenceRecovery::Degraded
        }
        _ => SourceEvidenceRecovery::Degraded,
    }
}
