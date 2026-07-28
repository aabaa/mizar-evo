use std::collections::BTreeMap;

use mizar_checker::{
    binding_env::{BindingContextId, BindingEnv},
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_application::{
        SourceFunctorApplicationForm, SourceFunctorApplicationHandoff,
        SourceFunctorApplicationHandoffInput, SourceFunctorApplicationId,
        SourceFunctorApplicationInput, SourceFunctorApplicationKind,
        SourceFunctorApplicationProducer, SourceFunctorApplicationRecovery,
        SourceFunctorArgumentInput, SourceFunctorArgumentTarget, SourceFunctorCandidateId,
        SourceFunctorCandidateInput, SourceFunctorHeadSite, SourceFunctorTypeRequestInput,
        SourceFunctorTypeRequestKind, SourceFunctorWrapperInput,
    },
    source_context::{
        SourceBindingContextBuild, SourceBindingContextInput, SourceBindingContextOwner,
        SourceBindingContextProducer, SourceBindingSiteInput, SourceBindingSiteRole,
        SourceItemInput, SourceItemRecovery, SourceItemRole, SourceItemVisibility,
    },
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
        TypeFactTable, TypeTable, TypedAst, TypedAstParts, TypedSiteRef,
    },
};
use mizar_resolve::{
    declarations::{DeclarationShell, DeclarationShellKind, DeclarationShellSet},
    env::{SymbolEnv, SymbolKind},
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::{ModuleId, SymbolId},
};
use mizar_session::SourceRange;
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeId, SurfaceNodeKind};

use super::{
    checker_handoff::{assemble_empty_resolved_typed_ast, source_module_binding_env},
    source_ast::{
        direct_token_texts, exact_compilation_item_list, qualified_symbol_spelling,
        structural_child_ids, subtree_has_recovery, surface_nodes_with_kind, surface_site,
    },
    source_formula::{
        extract_source_imported_predicate_functor_formula,
        resolve_imported_fixture_term_formula_symbol,
    },
    source_reserve::extract_builtin_source_reserve_declarations_after_node_guard,
    source_term::{SourceTermParts, source_term_parts_for_roots},
};

#[cfg(test)]
use super::source_term::{
    source_term_parts_for_context_roots, synthetic_source_term_parts_for_roots,
};
#[cfg(test)]
use mizar_checker::typed_ast::NodeRecoveryState;
#[cfg(test)]
use mizar_resolve::env::SourceContributionId;

const INVALID_PAYLOAD_KEY: &str = "type_elaboration.checker.typed_ast_invalid";
const PAYLOAD_EXTRACTION_GAP_KEY: &str =
    "type_elaboration.external_dependency.ast_payload_extraction";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SourceApplicationRouteKind {
    ImportedInfix,
    LocalFunctional,
}

#[derive(Debug)]
pub(in crate::runner) struct SourceApplicationRouteOutput {
    pub(in crate::runner) kind: SourceApplicationRouteKind,
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
pub(in crate::runner) enum SyntheticSourceFunctorHead {
    Single(usize),
    Paired { left: usize, right: usize },
}

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
pub(in crate::runner) enum SyntheticSourceFunctorArgument {
    Primary(usize),
    Application(usize),
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub(in crate::runner) struct SyntheticSourceFunctorApplication {
    pub(in crate::runner) application: usize,
    pub(in crate::runner) form: SourceFunctorApplicationForm,
    pub(in crate::runner) kind: SourceFunctorApplicationKind,
    pub(in crate::runner) head: SyntheticSourceFunctorHead,
    pub(in crate::runner) wrappers: Vec<usize>,
    pub(in crate::runner) arguments: Vec<SyntheticSourceFunctorArgument>,
    pub(in crate::runner) candidates: Vec<(SymbolId, SourceContributionId)>,
    pub(in crate::runner) degraded: bool,
}

#[cfg(test)]
#[derive(Debug)]
pub(in crate::runner) struct SyntheticSourceApplicationOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[cfg(test)]
pub(in crate::runner) fn synthetic_functional_actual_count(
    ast: &SurfaceAst,
    application: usize,
) -> Result<usize, String> {
    let node = ast
        .nodes()
        .get(application)
        .ok_or_else(|| "synthetic functional application site disappeared".to_owned())?;
    if !matches!(node.kind, SurfaceNodeKind::ApplicationTerm) || node.recovered {
        return Err(
            "synthetic functional application must be one normal ApplicationTerm".to_owned(),
        );
    }
    let punctuation = direct_token_texts(ast, node);
    if punctuation.len() < 2 {
        return Err("synthetic functional application lost mandatory parentheses".to_owned());
    }
    let (Some(first), Some(last)) = (punctuation.first(), punctuation.last()) else {
        return Err("synthetic functional application lost mandatory parentheses".to_owned());
    };
    if first != "("
        || last != ")"
        || punctuation[1..punctuation.len() - 1]
            .iter()
            .any(|token| token != ",")
    {
        return Err("synthetic functional application punctuation is not canonical".to_owned());
    }
    let structural = structural_child_ids(ast, node);
    let Some(actual_count) = structural.len().checked_sub(1) else {
        return Err("synthetic functional application lost its generic head shape".to_owned());
    };
    let expected_punctuation = if actual_count == 0 {
        2
    } else {
        actual_count.saturating_add(1)
    };
    if punctuation.len() != expected_punctuation {
        return Err("synthetic functional application actual/comma cardinality drift".to_owned());
    }
    Ok(actual_count)
}

#[cfg(test)]
pub(in crate::runner) fn synthetic_source_application_output(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    symbols: &SymbolEnv,
    probes: &[SyntheticSourceFunctorApplication],
) -> Result<SyntheticSourceApplicationOutput, String> {
    if binding_env.source_id() != ast.source_id
        || binding_env.module_id() != &module
        || symbols.module_id() != &module
    {
        return Err("synthetic source-application identity mismatch".to_owned());
    }

    if probes
        .iter()
        .any(|probe| probe.application >= ast.nodes().len())
    {
        return Err("synthetic source-application site is out of range".to_owned());
    }
    if probes.iter().any(|probe| {
        probe.arguments.iter().any(|argument| {
            matches!(
                argument,
                SyntheticSourceFunctorArgument::Application(target) if *target >= probes.len()
            )
        })
    }) {
        return Err("synthetic source-application target is out of range".to_owned());
    }
    let mut excluded_indexes = probes
        .iter()
        .enumerate()
        .filter(|(_, probe)| synthetic_application_is_excluded(ast, probe.application))
        .map(|(index, _)| index)
        .collect::<std::collections::BTreeSet<_>>();
    loop {
        let before = excluded_indexes.len();
        for (index, probe) in probes.iter().enumerate() {
            for argument in &probe.arguments {
                let SyntheticSourceFunctorArgument::Application(target) = argument else {
                    continue;
                };
                if excluded_indexes.contains(&index) {
                    excluded_indexes.insert(*target);
                }
                if excluded_indexes.contains(target) {
                    excluded_indexes.insert(index);
                }
            }
        }
        if excluded_indexes.len() == before {
            break;
        }
    }
    let selected_indexes = (0..probes.len())
        .filter(|index| !excluded_indexes.contains(index))
        .collect::<Vec<_>>();
    let remap = selected_indexes
        .iter()
        .enumerate()
        .map(|(new, old)| (*old, new))
        .collect::<BTreeMap<_, _>>();
    let probes = selected_indexes
        .into_iter()
        .map(|index| {
            let mut probe = probes[index].clone();
            for argument in &mut probe.arguments {
                if let SyntheticSourceFunctorArgument::Application(target) = argument {
                    *target = remap[target];
                }
            }
            probe
        })
        .collect::<Vec<_>>();
    let mut owned_node_kinds = BTreeMap::new();
    let mut owned_node_recoveries = BTreeMap::new();
    let mut primary_roots = Vec::new();
    for (source_ordinal, probe) in probes.iter().enumerate() {
        let application = ast
            .nodes()
            .get(probe.application)
            .ok_or_else(|| format!("synthetic application {source_ordinal} disappeared"))?;
        let valid_kind = match probe.form {
            SourceFunctorApplicationForm::Bare => {
                matches!(application.kind, SurfaceNodeKind::TermExpression)
            }
            SourceFunctorApplicationForm::Prefix => {
                matches!(application.kind, SurfaceNodeKind::PrefixExpression(_))
            }
            SourceFunctorApplicationForm::Infix => {
                matches!(application.kind, SurfaceNodeKind::InfixExpression(_))
            }
            SourceFunctorApplicationForm::Postfix => {
                matches!(application.kind, SurfaceNodeKind::PostfixExpression(_))
            }
            SourceFunctorApplicationForm::Bracket | SourceFunctorApplicationForm::Functional => {
                matches!(application.kind, SurfaceNodeKind::ApplicationTerm)
            }
            _ => false,
        };
        if !valid_kind || subtree_has_recovery(ast, application) {
            return Err(format!(
                "synthetic application {source_ordinal} raw surface kind/recovery drift"
            ));
        }
        if probe.form == SourceFunctorApplicationForm::Functional
            && synthetic_functional_actual_count(ast, probe.application)? != probe.arguments.len()
        {
            return Err(format!(
                "synthetic application {source_ordinal} raw actual cardinality drift"
            ));
        }
        let application_kind = match probe.kind {
            SourceFunctorApplicationKind::Symbolic => "source.term.functor-application.symbolic",
            SourceFunctorApplicationKind::Inline => "source.term.functor-application.inline",
            _ => {
                return Err(format!(
                    "synthetic application {source_ordinal} has an unsupported kind"
                ));
            }
        };
        insert_owned_kind(&mut owned_node_kinds, probe.application, application_kind)?;
        if probe.degraded {
            owned_node_recoveries.insert(probe.application, NodeRecoveryState::Degraded);
        }
        match probe.head {
            SyntheticSourceFunctorHead::Single(head) => {
                if probe.form == SourceFunctorApplicationForm::Bracket {
                    return Err(format!(
                        "synthetic application {source_ordinal} bracket head is not paired"
                    ));
                }
                insert_owned_kind(
                    &mut owned_node_kinds,
                    head,
                    "source.term.functor-head.single",
                )?;
                if probe.degraded {
                    owned_node_recoveries.insert(head, NodeRecoveryState::Degraded);
                }
            }
            SyntheticSourceFunctorHead::Paired { left, right } => {
                if probe.form != SourceFunctorApplicationForm::Bracket {
                    return Err(format!(
                        "synthetic application {source_ordinal} non-bracket head is paired"
                    ));
                }
                for head in [left, right] {
                    insert_owned_kind(
                        &mut owned_node_kinds,
                        head,
                        "source.term.functor-head.bracket",
                    )?;
                    if probe.degraded {
                        owned_node_recoveries.insert(head, NodeRecoveryState::Degraded);
                    }
                }
            }
        }
        for wrapper in &probe.wrappers {
            let node = ast.nodes().get(*wrapper).ok_or_else(|| {
                format!("synthetic application {source_ordinal} wrapper disappeared")
            })?;
            if !matches!(node.kind, SurfaceNodeKind::ParenthesizedTerm)
                || direct_token_texts(ast, node).as_slice() != ["(", ")"]
            {
                return Err(format!(
                    "synthetic application {source_ordinal} wrapper shape drift"
                ));
            }
            insert_owned_kind(
                &mut owned_node_kinds,
                *wrapper,
                "source.term.functor-application.parenthesized",
            )?;
            if probe.degraded {
                owned_node_recoveries.insert(*wrapper, NodeRecoveryState::Degraded);
            }
        }
        primary_roots.extend(
            probe
                .arguments
                .iter()
                .filter_map(|argument| match argument {
                    SyntheticSourceFunctorArgument::Primary(root) => Some(*root),
                    SyntheticSourceFunctorArgument::Application(_) => None,
                }),
        );
    }

    let context = BindingContextId::new(0);
    let source_term = synthetic_source_term_parts_for_roots(
        ast,
        module.clone(),
        &binding_env,
        primary_roots,
        context,
        &owned_node_kinds,
        &owned_node_recoveries,
    )?;
    let primary_id = |root: usize| {
        let range = ast.nodes().get(root)?.range;
        source_term
            .handoff
            .terms()
            .iter()
            .find(|(_, term)| term.parent().is_none() && term.source_range() == range)
            .map(|(id, _)| id)
    };

    let mut applications = Vec::new();
    let mut wrappers = Vec::new();
    let mut candidates = Vec::new();
    let mut arguments = Vec::new();
    let mut type_requests = Vec::new();
    for (source_ordinal, probe) in probes.iter().enumerate() {
        let application_id = SourceFunctorApplicationId::new(source_ordinal);
        let application = &ast.nodes()[probe.application];
        let recovery = if probe.degraded {
            SourceFunctorApplicationRecovery::Degraded
        } else {
            SourceFunctorApplicationRecovery::Normal
        };
        let head = match probe.head {
            SyntheticSourceFunctorHead::Single(head) => {
                let node = ast
                    .nodes()
                    .get(head)
                    .ok_or_else(|| "synthetic single head disappeared".to_owned())?;
                SourceFunctorHeadSite::Single {
                    site: TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(head)),
                    source_range: node.range,
                    spelling: subtree_tokens(ast, node).join(" "),
                }
            }
            SyntheticSourceFunctorHead::Paired { left, right } => {
                let left_node = ast
                    .nodes()
                    .get(left)
                    .ok_or_else(|| "synthetic left bracket head disappeared".to_owned())?;
                let right_node = ast
                    .nodes()
                    .get(right)
                    .ok_or_else(|| "synthetic right bracket head disappeared".to_owned())?;
                SourceFunctorHeadSite::Paired {
                    left_site: TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(left)),
                    left_range: left_node.range,
                    left_spelling: subtree_tokens(ast, left_node).join(" "),
                    right_site: TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(
                        right,
                    )),
                    right_range: right_node.range,
                    right_spelling: subtree_tokens(ast, right_node).join(" "),
                }
            }
        };
        applications.push(SourceFunctorApplicationInput {
            site: TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(
                probe.application,
            )),
            source_range: application.range,
            source_ordinal,
            context,
            recovery,
            spelling: subtree_tokens(ast, application).join(" "),
            kind: probe.kind,
            form: probe.form,
            head_ordinal: match probe.form {
                SourceFunctorApplicationForm::Infix => 1,
                SourceFunctorApplicationForm::Postfix => probe.arguments.len(),
                _ => 0,
            },
            head,
        });
        for (ordinal, wrapper) in probe.wrappers.iter().copied().enumerate() {
            let node = &ast.nodes()[wrapper];
            wrappers.push(SourceFunctorWrapperInput {
                application: application_id,
                ordinal,
                site: TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(wrapper)),
                source_range: node.range,
                context,
                spelling: subtree_tokens(ast, node).join(" "),
                recovery,
            });
        }
        let candidate_start = candidates.len();
        for (ordinal, (symbol, contribution)) in probe.candidates.iter().enumerate() {
            candidates.push(SourceFunctorCandidateInput {
                application: application_id,
                ordinal,
                symbol: symbol.clone(),
                contribution: *contribution,
            });
        }
        for (ordinal, argument) in probe.arguments.iter().enumerate() {
            let target = match argument {
                SyntheticSourceFunctorArgument::Primary(root) => {
                    SourceFunctorArgumentTarget::Primary(primary_id(*root).ok_or_else(|| {
                        format!(
                            "synthetic application {source_ordinal} has an unowned primary argument"
                        )
                    })?)
                }
                SyntheticSourceFunctorArgument::Application(application) => {
                    SourceFunctorArgumentTarget::Application(SourceFunctorApplicationId::new(
                        *application,
                    ))
                }
            };
            arguments.push(SourceFunctorArgumentInput {
                application: application_id,
                ordinal,
                target,
            });
        }
        if probe.kind == SourceFunctorApplicationKind::Symbolic {
            for ordinal in 0..probe.candidates.len() {
                type_requests.push(SourceFunctorTypeRequestInput {
                    application: application_id,
                    candidate: Some(SourceFunctorCandidateId::new(candidate_start + ordinal)),
                    request_ordinal: ordinal,
                    kind: SourceFunctorTypeRequestKind::CandidateSignature,
                });
            }
            type_requests.push(SourceFunctorTypeRequestInput {
                application: application_id,
                candidate: None,
                request_ordinal: probe.candidates.len(),
                kind: SourceFunctorTypeRequestKind::ApplicationResultType,
            });
        }
    }

    let handoff = SourceFunctorApplicationProducer::build(
        SourceFunctorApplicationHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            applications,
            wrappers,
            candidates,
            arguments,
            type_requests,
        },
        symbols,
        &binding_env,
        &source_term.handoff,
        &source_term.arena,
    )
    .map_err(|error| error.to_string())?;
    let typed_ast = TypedAst::try_new(TypedAstParts {
        source_id: ast.source_id,
        module_id: module,
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: source_term.arena,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())?
    .with_source_term(source_term.handoff)
    .map_err(|error| error.to_string())?
    .with_source_application(handoff)
    .map_err(|error| error.to_string())?;
    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.term.surface"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)?;
    if resolved.source_application() != typed_ast.source_application()
        || resolved.source_term() != typed_ast.source_term()
    {
        return Err("synthetic source-application final preservation mismatch".to_owned());
    }
    Ok(SyntheticSourceApplicationOutput {
        typed_ast,
        resolved,
    })
}

#[cfg(test)]
fn synthetic_application_is_excluded(ast: &SurfaceAst, application: usize) -> bool {
    fn excluded_kind(kind: &SurfaceNodeKind) -> bool {
        matches!(
            kind,
            SurfaceNodeKind::TemplateParameter
                | SurfaceNodeKind::TemplateLoci
                | SurfaceNodeKind::TemplateLocus
                | SurfaceNodeKind::TemplateArguments
                | SurfaceNodeKind::TemplateArgument
                | SurfaceNodeKind::StructureConstructor
                | SurfaceNodeKind::FieldArgument
                | SurfaceNodeKind::SelectorAccess
                | SurfaceNodeKind::StructureUpdate
                | SurfaceNodeKind::FieldUpdate
                | SurfaceNodeKind::SetEnumeration
                | SurfaceNodeKind::SetComprehension
                | SurfaceNodeKind::ChoiceTerm
                | SurfaceNodeKind::QuaExpression
        )
    }
    fn excluded_descendant(ast: &SurfaceAst, node: usize) -> bool {
        ast.nodes()[node].children.iter().any(|child| {
            excluded_kind(&ast.nodes()[child.index()].kind)
                || excluded_descendant(ast, child.index())
        })
    }

    if excluded_descendant(ast, application) {
        return true;
    }
    let mut parents = vec![None; ast.nodes().len()];
    for (parent, node) in ast.nodes().iter().enumerate() {
        for child in &node.children {
            parents[child.index()] = Some(parent);
        }
    }
    let mut cursor = parents.get(application).copied().flatten();
    while let Some(parent) = cursor {
        if excluded_kind(&ast.nodes()[parent].kind) {
            return true;
        }
        cursor = parents[parent];
    }
    false
}

#[cfg(test)]
fn insert_owned_kind(
    kinds: &mut BTreeMap<usize, &'static str>,
    site: usize,
    kind: &'static str,
) -> Result<(), String> {
    if kinds.insert(site, kind).is_some() {
        return Err("synthetic source-application arena site is multiply owned".to_owned());
    }
    Ok(())
}

/// Runs Task 253 only for its two frozen real consumers. The imported route
/// preserves the legacy detail owner; the new local route reaches the
/// deliberately bounded definition-payload extraction gap.
pub(in crate::runner) fn source_application_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    match source_application_output(ast, module, shells, symbols) {
        None => None,
        Some(Ok(output))
            if output.typed_ast.source_application().is_some()
                && output.typed_ast.source_application()
                    == output.resolved.source_application()
                && output.typed_ast.source_term().is_some()
                && output.typed_ast.source_term() == output.resolved.source_term() =>
        {
            match output.kind {
                SourceApplicationRouteKind::ImportedInfix => None,
                SourceApplicationRouteKind::LocalFunctional => {
                    Some(vec![PAYLOAD_EXTRACTION_GAP_KEY.to_owned()])
                }
            }
        }
        Some(Ok(_)) | Some(Err(_)) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_application_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Result<SourceApplicationRouteOutput, String>> {
    source_application_output_with_mutation(ast, module, shells, symbols, |_| {})
}

#[cfg(not(test))]
fn source_application_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Result<SourceApplicationRouteOutput, String>> {
    source_application_output_with_mutation(ast, module, shells, symbols, |_| {})
}

#[cfg(test)]
pub(in crate::runner) fn source_application_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceFunctorApplicationHandoffInput),
) -> Option<Result<SourceApplicationRouteOutput, String>> {
    source_application_output_with_mutation_impl(ast, module, shells, symbols, mutate)
}

#[cfg(not(test))]
fn source_application_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceFunctorApplicationHandoffInput),
) -> Option<Result<SourceApplicationRouteOutput, String>> {
    source_application_output_with_mutation_impl(ast, module, shells, symbols, mutate)
}

fn source_application_output_with_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceFunctorApplicationHandoffInput),
) -> Option<Result<SourceApplicationRouteOutput, String>> {
    if let Some(extracted) = extract_imported(ast, &module, symbols) {
        let binding_env = match source_module_binding_env(ast, module.clone()) {
            Ok(binding_env) => binding_env,
            Err(error) => return Some(Err(error.to_string())),
        };
        return Some(build_output(
            ast,
            module,
            symbols,
            binding_env,
            extracted,
            mutate,
        ));
    }
    let extracted = extract_local(ast, &module, shells, symbols)?;
    Some(match extracted {
        Ok((binding_env, extracted)) => {
            build_output(ast, module, symbols, binding_env, extracted, mutate)
        }
        Err(error) => Err(error),
    })
}

struct ExtractedApplication {
    kind: SourceApplicationRouteKind,
    context: BindingContextId,
    application_id: SurfaceNodeId,
    application_range: SourceRange,
    application_spelling: String,
    form: SourceFunctorApplicationForm,
    head_id: SurfaceNodeId,
    head_range: SourceRange,
    head_spelling: String,
    wrapper: Option<(SurfaceNodeId, SourceRange)>,
    argument_roots: Vec<SurfaceNodeId>,
    candidate: SymbolId,
}

fn extract_imported(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
) -> Option<ExtractedApplication> {
    let payload = extract_source_imported_predicate_functor_formula(ast, module, symbols)?;
    let application_id = node_id(ast, payload.functor_site)?;
    let application = ast.node(application_id)?;
    if application.range != payload.functor_range
        || !matches!(
            &application.kind,
            SurfaceNodeKind::InfixExpression(operator) if operator.spelling.as_ref() == "++"
        )
        || direct_token_texts(ast, application).as_slice() != ["++"]
    {
        return None;
    }
    let head_ids = application
        .children
        .iter()
        .copied()
        .filter(|child| ast.node(*child).and_then(SurfaceNode::token_text) == Some("++"))
        .collect::<Vec<_>>();
    let [head_id] = head_ids.as_slice() else {
        return None;
    };
    let head = ast.node(*head_id)?;
    let wrapper = surface_nodes_with_kind(ast, SurfaceNodeKind::ParenthesizedTerm)
        .into_iter()
        .filter(|(_, node)| {
            !subtree_has_recovery(ast, node)
                && direct_token_texts(ast, node).as_slice() == ["(", ")"]
                && structural_descendant(ast, node, application_id)
        })
        .map(|(id, node)| (id, node.range))
        .collect::<Vec<_>>();
    let [(wrapper_id, wrapper_range)] = wrapper.as_slice() else {
        return None;
    };
    let left_id = node_id(ast, payload.functor_left_site)?;
    let right_id = node_id(ast, payload.functor_right_site)?;
    Some(ExtractedApplication {
        kind: SourceApplicationRouteKind::ImportedInfix,
        context: BindingContextId::new(0),
        application_id,
        application_range: application.range,
        application_spelling: "1 ++ 2".to_owned(),
        form: SourceFunctorApplicationForm::Infix,
        head_id: *head_id,
        head_range: head.range,
        head_spelling: "++".to_owned(),
        wrapper: Some((*wrapper_id, *wrapper_range)),
        argument_roots: vec![left_id, right_id],
        candidate: payload.functor_symbol,
    })
}

fn extract_unwrapped_imported(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    application_index: usize,
) -> Option<ExtractedApplication> {
    let application_id = node_id(
        ast,
        TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(
            application_index,
        )),
    )?;
    let application = ast.node(application_id)?;
    if application.recovered
        || !matches!(
            &application.kind,
            SurfaceNodeKind::InfixExpression(operator) if operator.spelling.as_ref() == "++"
        )
        || direct_token_texts(ast, application).as_slice() != ["++"]
        || subtree_tokens(ast, application) != ["1", "++", "2"]
    {
        return None;
    }
    let argument_roots = structural_child_ids(ast, application);
    let [left_id, right_id] = argument_roots.as_slice() else {
        return None;
    };
    let left = ast.node(*left_id)?;
    let right = ast.node(*right_id)?;
    if !matches!(left.kind, SurfaceNodeKind::NumeralTerm)
        || !matches!(right.kind, SurfaceNodeKind::NumeralTerm)
        || left.recovered
        || right.recovered
        || subtree_tokens(ast, left) != ["1"]
        || subtree_tokens(ast, right) != ["2"]
        || surface_nodes_with_kind(ast, SurfaceNodeKind::ParenthesizedTerm)
            .into_iter()
            .any(|(_, wrapper)| structural_descendant(ast, wrapper, application_id))
    {
        return None;
    }
    let head_ids = application
        .children
        .iter()
        .copied()
        .filter(|child| ast.node(*child).and_then(SurfaceNode::token_text) == Some("++"))
        .collect::<Vec<_>>();
    let [head_id] = head_ids.as_slice() else {
        return None;
    };
    let head = ast.node(*head_id)?;
    let candidate =
        resolve_imported_fixture_term_formula_symbol(symbols, module, "++", SymbolKind::Functor)
            .ok()?;
    Some(ExtractedApplication {
        kind: SourceApplicationRouteKind::ImportedInfix,
        context: BindingContextId::new(0),
        application_id,
        application_range: application.range,
        application_spelling: "1 ++ 2".to_owned(),
        form: SourceFunctorApplicationForm::Infix,
        head_id: *head_id,
        head_range: head.range,
        head_spelling: "++".to_owned(),
        wrapper: None,
        argument_roots: vec![*left_id, *right_id],
        candidate,
    })
}

fn extract_local(
    ast: &SurfaceAst,
    module: &ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Result<(BindingEnv, ExtractedApplication), String>> {
    let item_list = exact_compilation_item_list(ast)?;
    let item_ids = structural_child_ids(ast, item_list);
    let [reserve_id, definition_id] = item_ids.as_slice() else {
        return None;
    };
    let reserve = ast.node(*reserve_id)?;
    let definition = ast.node(*definition_id)?;
    if !matches!(reserve.kind, SurfaceNodeKind::ReserveItem)
        || !matches!(definition.kind, SurfaceNodeKind::DefinitionBlockItem)
        || subtree_has_recovery(ast, reserve)
        || subtree_has_recovery(ast, definition)
        || subtree_tokens(ast, reserve) != ["reserve", "x", "for", "set", ";"]
        || subtree_tokens(ast, definition)
            != [
                "definition",
                "let",
                "x",
                "be",
                "set",
                ";",
                "func",
                "Task253LocalSourceDef",
                ":",
                "task253_local_source",
                "(",
                "x",
                ")",
                "->",
                "set",
                "equals",
                "x",
                ";",
                "func",
                "Task253LocalConsumerDef",
                ":",
                "task253_local_consumer",
                "(",
                "x",
                ")",
                "->",
                "set",
                "equals",
                "task253_local_source",
                "(",
                "x",
                ")",
                ";",
                "end",
                ";",
            ]
    {
        return None;
    }

    let definition_children = structural_child_ids(ast, definition);
    let [parameter_id, first_functor_id, second_functor_id] = definition_children.as_slice() else {
        return None;
    };
    let parameter = ast.node(*parameter_id)?;
    let first_functor = ast.node(*first_functor_id)?;
    let second_functor = ast.node(*second_functor_id)?;
    if !matches!(parameter.kind, SurfaceNodeKind::DefinitionParameter)
        || !matches!(first_functor.kind, SurfaceNodeKind::FunctorDefinition)
        || !matches!(second_functor.kind, SurfaceNodeKind::FunctorDefinition)
        || subtree_has_recovery(ast, parameter)
        || subtree_has_recovery(ast, first_functor)
        || subtree_has_recovery(ast, second_functor)
    {
        return None;
    }

    let applications = surface_nodes_with_kind(ast, SurfaceNodeKind::ApplicationTerm);
    let [(application_id, application)] = applications.as_slice() else {
        return None;
    };
    if !structural_descendant(ast, second_functor, *application_id)
        || direct_token_texts(ast, application).as_slice() != ["(", ")"]
        || subtree_tokens(ast, application) != ["task253_local_source", "(", "x", ")"]
    {
        return None;
    }
    let application_children = structural_child_ids(ast, application);
    let [head_id, argument_expression_id] = application_children.as_slice() else {
        return None;
    };
    let head = ast.node(*head_id)?;
    let argument_expression = ast.node(*argument_expression_id)?;
    if !matches!(head.kind, SurfaceNodeKind::TermReference)
        || !matches!(argument_expression.kind, SurfaceNodeKind::TermExpression)
        || !direct_token_texts(ast, head).is_empty()
        || !direct_token_texts(ast, argument_expression).is_empty()
    {
        return None;
    }
    let head_children = structural_child_ids(ast, head);
    let [qualified_id] = head_children.as_slice() else {
        return None;
    };
    let qualified = ast.node(*qualified_id)?;
    if qualified_symbol_spelling(ast, qualified).ok()?.as_str() != "task253_local_source" {
        return None;
    }
    let argument_children = structural_child_ids(ast, argument_expression);
    let [actual_id] = argument_children.as_slice() else {
        return None;
    };
    let actual = ast.node(*actual_id)?;
    if !matches!(actual.kind, SurfaceNodeKind::TermReference)
        || direct_token_texts(ast, actual).as_slice() != ["x"]
        || !structural_child_ids(ast, actual).is_empty()
    {
        return None;
    }
    let candidates = symbols
        .symbols()
        .iter()
        .filter(|entry| {
            entry.kind() == SymbolKind::Functor
                && entry.symbol().module() == module
                && entry.primary_spelling() == "task253_local_source ( x )"
                && entry.origin().anchor()
                    == &mizar_session::SourceAnchor::Range(first_functor.range)
        })
        .collect::<Vec<_>>();
    let [candidate] = candidates.as_slice() else {
        return None;
    };
    if first_functor.range.end > application.range.start {
        return None;
    }

    Some(
        build_local_binding_env(
            ast,
            module.clone(),
            shells,
            symbols,
            *reserve_id,
            reserve,
            *definition_id,
            definition,
            *parameter_id,
            parameter,
        )
        .map(|binding_env| {
            (
                binding_env,
                ExtractedApplication {
                    kind: SourceApplicationRouteKind::LocalFunctional,
                    context: BindingContextId::new(1),
                    application_id: *application_id,
                    application_range: application.range,
                    application_spelling: "task253_local_source ( x )".to_owned(),
                    form: SourceFunctorApplicationForm::Functional,
                    head_id: *head_id,
                    head_range: head.range,
                    head_spelling: "task253_local_source".to_owned(),
                    wrapper: None,
                    argument_roots: vec![*argument_expression_id],
                    candidate: candidate.symbol().clone(),
                },
            )
        }),
    )
}

#[allow(clippy::too_many_arguments)] // Rationale: keep each authenticated local binding owner explicit at this private source boundary.
fn build_local_binding_env(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    reserve_id: SurfaceNodeId,
    reserve: &SurfaceNode,
    definition_id: SurfaceNodeId,
    definition: &SurfaceNode,
    parameter_id: SurfaceNodeId,
    parameter: &SurfaceNode,
) -> Result<BindingEnv, String> {
    let reserve_shell = unique_shell(shells, reserve_id, DeclarationShellKind::Reserve)?;
    let definition_shell =
        unique_shell(shells, definition_id, DeclarationShellKind::DefinitionBlock)?;
    let reserve_payload =
        extract_builtin_source_reserve_declarations_after_node_guard(ast, module.clone(), symbols)
            .map_err(|()| "Task253 local reserve extraction failed".to_owned())?;
    let [reserve_binding] = reserve_payload.bridge.bindings() else {
        return Err("Task253 local route requires one reserve binding".to_owned());
    };
    let reserve_children = structural_child_ids(ast, reserve);
    let [reserve_segment_id] = reserve_children.as_slice() else {
        return Err("Task253 local reserve requires one segment".to_owned());
    };
    let parameter_children = structural_child_ids(ast, parameter);
    let [segment_id] = parameter_children.as_slice() else {
        return Err("Task253 local parameter requires one segment".to_owned());
    };
    let segment = ast
        .node(*segment_id)
        .ok_or_else(|| "Task253 local parameter segment disappeared".to_owned())?;
    let segment_children = structural_child_ids(ast, segment);
    let [type_id] = segment_children.as_slice() else {
        return Err("Task253 local parameter requires one written type".to_owned());
    };
    let type_node = ast
        .node(*type_id)
        .ok_or_else(|| "Task253 local parameter type disappeared".to_owned())?;
    if !matches!(segment.kind, SurfaceNodeKind::QualifiedVariableSegment)
        || direct_token_texts(ast, segment).as_slice() != ["x", "be"]
        || !matches!(type_node.kind, SurfaceNodeKind::TypeExpression)
        || subtree_tokens(ast, type_node) != ["set"]
    {
        return Err("Task253 local parameter shape is not exact".to_owned());
    }
    let declaration_range = unique_direct_token_range(ast, segment, "x")?;
    let root_id = ast
        .root()
        .ok_or_else(|| "Task253 local root disappeared".to_owned())?;
    let local_scope = LocalTermScope::new(vec![definition_shell.id().index() as u32]);
    let build = SourceBindingContextProducer::build(SourceBindingContextInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        module_site: surface_site(root_id),
        items: vec![
            SourceItemInput {
                shell: reserve_shell.id(),
                shell_ordinal: reserve_shell.ordinal(),
                role: SourceItemRole::Reserve,
                module_id: module.clone(),
                source_range: reserve.range,
                parent: None,
                visibility: SourceItemVisibility::Unspecified,
                site: surface_site(reserve_id),
                local_scope: None,
                recovery: SourceItemRecovery::Normal,
            },
            SourceItemInput {
                shell: definition_shell.id(),
                shell_ordinal: definition_shell.ordinal(),
                role: SourceItemRole::DefinitionBlock,
                module_id: module.clone(),
                source_range: definition.range,
                parent: None,
                visibility: SourceItemVisibility::Unspecified,
                site: surface_site(definition_id),
                local_scope: Some(local_scope.clone()),
                recovery: SourceItemRecovery::Normal,
            },
        ],
        bindings: vec![
            SourceBindingSiteInput {
                shell: reserve_shell.id(),
                context_owner: SourceBindingContextOwner::Module,
                source_ordinal: 0,
                spelling: "x".to_owned(),
                declaration_range: reserve_binding.binding_range,
                written_type_range: reserve_binding.type_range,
                site: surface_site(*reserve_segment_id),
                role: SourceBindingSiteRole::ReserveDefault,
                recovery: mizar_checker::binding_env::BindingRecoveryState::Normal,
            },
            SourceBindingSiteInput {
                shell: definition_shell.id(),
                context_owner: SourceBindingContextOwner::Shell(definition_shell.id()),
                source_ordinal: 1,
                spelling: "x".to_owned(),
                declaration_range,
                written_type_range: type_node.range,
                site: surface_site(parameter_id),
                role: SourceBindingSiteRole::DefinitionParameter {
                    local: LocalTermBinding::new("x", local_scope, declaration_range, 1),
                },
                recovery: mizar_checker::binding_env::BindingRecoveryState::Normal,
            },
        ],
    })
    .map_err(|error| error.to_string())?;
    match build {
        SourceBindingContextBuild::Complete(projection) => {
            let handoff = projection.into_handoff();
            if handoff.binding_env().bindings().len() != 2
                || handoff.binding_env().contexts().len() != 2
            {
                return Err("Task253 local binding projection cardinality mismatch".to_owned());
            }
            Ok(handoff.binding_env().clone())
        }
        SourceBindingContextBuild::Incomplete(_) => {
            Err("Task253 local binding projection was incomplete".to_owned())
        }
        _ => Err("Task253 local binding projection returned an unsupported state".to_owned()),
    }
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    binding_env: BindingEnv,
    extracted: ExtractedApplication,
    mutate: impl FnOnce(&mut SourceFunctorApplicationHandoffInput),
) -> Result<SourceApplicationRouteOutput, String> {
    let mut owned_node_kinds = BTreeMap::from([
        (
            extracted.application_id.index(),
            "source.term.functor-application.symbolic",
        ),
        (extracted.head_id.index(), "source.term.functor-head.single"),
    ]);
    if let Some((wrapper, _)) = extracted.wrapper {
        owned_node_kinds.insert(
            wrapper.index(),
            "source.term.functor-application.parenthesized",
        );
    }
    let source_term = source_term_parts_for_roots(
        ast,
        module.clone(),
        &binding_env,
        extracted.argument_roots.iter().map(|root| root.index()),
        extracted.context,
        &owned_node_kinds,
    )?;
    build_output_with_source_term(
        ast,
        module,
        symbols,
        binding_env,
        extracted,
        source_term,
        mutate,
    )
}

pub(super) fn imported_source_application_output_with_source_term(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    binding_env: BindingEnv,
    source_term: SourceTermParts,
) -> Option<Result<SourceApplicationRouteOutput, String>> {
    let extracted = extract_imported(ast, &module, symbols)?;
    Some(build_output_with_source_term(
        ast,
        module,
        symbols,
        binding_env,
        extracted,
        source_term,
        |_| {},
    ))
}

pub(super) fn imported_source_application_owned_node_kinds(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
) -> Option<BTreeMap<usize, &'static str>> {
    let extracted = extract_imported(ast, module, symbols)?;
    let mut kinds = BTreeMap::from([
        (
            extracted.application_id.index(),
            "source.term.functor-application.symbolic",
        ),
        (extracted.head_id.index(), "source.term.functor-head.single"),
    ]);
    if let Some((wrapper, _)) = extracted.wrapper {
        kinds.insert(
            wrapper.index(),
            "source.term.functor-application.parenthesized",
        );
    }
    Some(kinds)
}

pub(super) fn unwrapped_imported_source_application_owned_node_kinds(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    application: usize,
) -> Option<BTreeMap<usize, &'static str>> {
    let extracted = extract_unwrapped_imported(ast, module, symbols, application)?;
    Some(BTreeMap::from([
        (
            extracted.application_id.index(),
            "source.term.functor-application.symbolic",
        ),
        (extracted.head_id.index(), "source.term.functor-head.single"),
    ]))
}

pub(super) fn unwrapped_imported_source_application_handoff(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    binding_env: &BindingEnv,
    source_term: &SourceTermParts,
    application: usize,
) -> Option<Result<SourceFunctorApplicationHandoff, String>> {
    unwrapped_imported_source_application_handoff_in_context(
        ast,
        module,
        symbols,
        binding_env,
        source_term,
        application,
        BindingContextId::new(0),
    )
}

pub(super) fn unwrapped_imported_source_application_handoff_in_context(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    binding_env: &BindingEnv,
    source_term: &SourceTermParts,
    application: usize,
    context: BindingContextId,
) -> Option<Result<SourceFunctorApplicationHandoff, String>> {
    let mut extracted = extract_unwrapped_imported(ast, module, symbols, application)?;
    extracted.context = context;
    Some(build_handoff_with_source_term(
        ast,
        module,
        symbols,
        binding_env,
        &extracted,
        source_term,
        |_| {},
    ))
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum UnwrappedImportedApplicationTestMutation {
    None,
    ApplicationRange,
    HeadRange,
    ArgumentTarget,
    Form,
    CandidateSymbol,
    CandidateContribution,
    StalePrimaryReplay,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy)]
pub(in crate::runner) struct UnwrappedImportedApplicationTestOptions {
    pub(in crate::runner) application: usize,
    pub(in crate::runner) context: BindingContextId,
    pub(in crate::runner) legacy_context_zero: bool,
    pub(in crate::runner) mutation: UnwrappedImportedApplicationTestMutation,
}

#[cfg(test)]
#[derive(Debug)]
pub(in crate::runner) struct UnwrappedImportedApplicationTestOutput {
    pub(in crate::runner) handoff: SourceFunctorApplicationHandoff,
    pub(in crate::runner) primary_counts: (usize, usize, usize),
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[cfg(test)]
pub(in crate::runner) fn unwrapped_imported_source_application_handoff_for_test(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    binding_env: &BindingEnv,
    roots: &[(usize, BindingContextId)],
    options: UnwrappedImportedApplicationTestOptions,
) -> Option<Result<UnwrappedImportedApplicationTestOutput, String>> {
    let UnwrappedImportedApplicationTestOptions {
        application,
        context,
        legacy_context_zero,
        mutation,
    } = options;
    let owned_node_kinds =
        unwrapped_imported_source_application_owned_node_kinds(ast, module, symbols, application)?;
    let source_term = match source_term_parts_for_context_roots(
        ast,
        module.clone(),
        binding_env,
        roots.iter().copied(),
        &owned_node_kinds,
    ) {
        Ok(source_term) => source_term,
        Err(error) => return Some(Err(error)),
    };
    let handoff = if legacy_context_zero {
        if context != BindingContextId::new(0) {
            return Some(Err(
                "legacy unwrapped Task253 test route requires context 0".to_owned(),
            ));
        }
        if mutation != UnwrappedImportedApplicationTestMutation::None {
            return Some(Err(
                "legacy unwrapped Task253 test route cannot mutate input".to_owned(),
            ));
        }
        match unwrapped_imported_source_application_handoff(
            ast,
            module,
            symbols,
            binding_env,
            &source_term,
            application,
        )? {
            Ok(handoff) => handoff,
            Err(error) => return Some(Err(error)),
        }
    } else if matches!(
        mutation,
        UnwrappedImportedApplicationTestMutation::None
            | UnwrappedImportedApplicationTestMutation::StalePrimaryReplay
    ) {
        match unwrapped_imported_source_application_handoff_in_context(
            ast,
            module,
            symbols,
            binding_env,
            &source_term,
            application,
            context,
        )? {
            Ok(handoff) => handoff,
            Err(error) => return Some(Err(error)),
        }
    } else {
        let mut extracted = extract_unwrapped_imported(ast, module, symbols, application)?;
        extracted.context = context;
        let selected_contribution = symbols
            .symbols()
            .get(&extracted.candidate)
            .expect("B1P extracted candidate remains in the symbol environment")
            .contribution();
        let substitute_contribution = symbols
            .symbols()
            .iter()
            .map(|entry| entry.contribution())
            .find(|candidate| *candidate != selected_contribution)
            .unwrap_or(selected_contribution);
        match build_handoff_with_source_term(
            ast,
            module,
            symbols,
            binding_env,
            &extracted,
            &source_term,
            |input| match mutation {
                UnwrappedImportedApplicationTestMutation::None
                | UnwrappedImportedApplicationTestMutation::StalePrimaryReplay => {
                    unreachable!("unmutated B1P test routes use the production helper")
                }
                UnwrappedImportedApplicationTestMutation::ApplicationRange => {
                    input.applications[0].source_range.start += 1;
                }
                UnwrappedImportedApplicationTestMutation::HeadRange => {
                    if let SourceFunctorHeadSite::Single { source_range, .. } =
                        &mut input.applications[0].head
                    {
                        source_range.start += 1;
                    }
                }
                UnwrappedImportedApplicationTestMutation::ArgumentTarget => {
                    input.arguments[0].target = SourceFunctorArgumentTarget::Primary(
                        mizar_checker::source_term::SourcePrimaryTermId::new(4),
                    );
                }
                UnwrappedImportedApplicationTestMutation::Form => {
                    input.applications[0].form = SourceFunctorApplicationForm::Prefix;
                }
                UnwrappedImportedApplicationTestMutation::CandidateSymbol => {
                    input.candidates[0].symbol = SymbolId::new(
                        module.clone(),
                        mizar_resolve::resolved_ast::LocalSymbolId::new("B1P/substitute"),
                        mizar_resolve::resolved_ast::FullyQualifiedName::new("b1p::substitute::++"),
                    );
                }
                UnwrappedImportedApplicationTestMutation::CandidateContribution => {
                    input.candidates[0].contribution = substitute_contribution;
                }
            },
        ) {
            Ok(handoff) => handoff,
            Err(error) => return Some(Err(error)),
        }
    };

    if mutation == UnwrappedImportedApplicationTestMutation::StalePrimaryReplay {
        let stale_source_term = match source_term_parts_for_context_roots(
            ast,
            module.clone(),
            binding_env,
            [(44, context), (45, context)],
            &owned_node_kinds,
        ) {
            Ok(source_term) => source_term,
            Err(error) => return Some(Err(error)),
        };
        let stale_typed = match task258b3m2b2b1p_typed_ast(ast, module, stale_source_term) {
            Ok(typed) => typed,
            Err(error) => return Some(Err(error)),
        };
        return Some(match stale_typed.with_source_application(handoff) {
            Ok(_) => Err("accepted stale Task252 fingerprint during replay".to_owned()),
            Err(error) => Err(format!(
                "rejected stale Task252 fingerprint during replay: {error}"
            )),
        });
    }

    let primary_counts = (
        source_term.handoff.terms().len(),
        source_term.handoff.references().len(),
        source_term.handoff.numeric_type_requests().len(),
    );
    let typed_ast = match task258b3m2b2b1p_typed_ast(ast, module, source_term) {
        Ok(typed) => match typed.with_source_application(handoff.clone()) {
            Ok(typed) => typed,
            Err(error) => return Some(Err(error.to_string())),
        },
        Err(error) => return Some(Err(error)),
    };
    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.term.surface"),
            },
        })
        .collect();
    let resolved = match assemble_empty_resolved_typed_ast(&typed_ast, node_hints) {
        Ok(resolved) => resolved,
        Err(error) => return Some(Err(error)),
    };
    Some(Ok(UnwrappedImportedApplicationTestOutput {
        handoff,
        primary_counts,
        typed_ast,
        resolved,
    }))
}

#[cfg(test)]
fn task258b3m2b2b1p_typed_ast(
    ast: &SurfaceAst,
    module: &ModuleId,
    source_term: SourceTermParts,
) -> Result<TypedAst, String> {
    TypedAst::try_new(TypedAstParts {
        source_id: ast.source_id,
        module_id: module.clone(),
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: source_term.arena,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())?
    .with_source_term(source_term.handoff)
    .map_err(|error| error.to_string())
}

fn build_output_with_source_term(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    binding_env: BindingEnv,
    extracted: ExtractedApplication,
    source_term: SourceTermParts,
    mutate: impl FnOnce(&mut SourceFunctorApplicationHandoffInput),
) -> Result<SourceApplicationRouteOutput, String> {
    let kind = extracted.kind;
    let handoff = build_handoff_with_source_term(
        ast,
        &module,
        symbols,
        &binding_env,
        &extracted,
        &source_term,
        mutate,
    )?;
    let typed_ast = TypedAst::try_new(TypedAstParts {
        source_id: ast.source_id,
        module_id: module,
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: source_term.arena,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())?
    .with_source_term(source_term.handoff)
    .map_err(|error| error.to_string())?
    .with_source_application(handoff)
    .map_err(|error| error.to_string())?;
    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.term.surface"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)?;
    if typed_ast.source_application().is_none()
        || resolved.source_application() != typed_ast.source_application()
        || resolved.source_term() != typed_ast.source_term()
        || !typed_ast.types().is_empty()
        || !typed_ast.facts().is_empty()
        || !typed_ast.coercions().is_empty()
        || !typed_ast.initial_obligations().is_empty()
        || !typed_ast.diagnostics().is_empty()
        || !resolved.expr_metadata().is_empty()
        || !resolved.cluster_facts().is_empty()
        || !resolved.diagnostics().is_empty()
    {
        return Err("source functor-application immutable final handoff mismatch".to_owned());
    }
    Ok(SourceApplicationRouteOutput {
        kind,
        typed_ast,
        resolved,
    })
}

fn build_handoff_with_source_term(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    binding_env: &BindingEnv,
    extracted: &ExtractedApplication,
    source_term: &SourceTermParts,
    mutate: impl FnOnce(&mut SourceFunctorApplicationHandoffInput),
) -> Result<SourceFunctorApplicationHandoff, String> {
    let candidate = symbols
        .symbols()
        .get(&extracted.candidate)
        .ok_or_else(|| "Task253 candidate disappeared".to_owned())?;
    if candidate.kind() != SymbolKind::Functor {
        return Err("Task253 real selector resolved a non-functor".to_owned());
    }
    let application = SourceFunctorApplicationId::new(0);
    let candidate_id = SourceFunctorCandidateId::new(0);
    let mut input = SourceFunctorApplicationHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        applications: vec![SourceFunctorApplicationInput {
            site: surface_site(extracted.application_id),
            source_range: extracted.application_range,
            source_ordinal: 0,
            context: extracted.context,
            recovery: SourceFunctorApplicationRecovery::Normal,
            spelling: extracted.application_spelling.clone(),
            kind: SourceFunctorApplicationKind::Symbolic,
            form: extracted.form,
            head_ordinal: usize::from(extracted.form == SourceFunctorApplicationForm::Infix),
            head: SourceFunctorHeadSite::Single {
                site: surface_site(extracted.head_id),
                source_range: extracted.head_range,
                spelling: extracted.head_spelling.clone(),
            },
        }],
        wrappers: extracted
            .wrapper
            .as_ref()
            .map(|(wrapper, range)| SourceFunctorWrapperInput {
                application,
                ordinal: 0,
                site: surface_site(*wrapper),
                source_range: *range,
                context: extracted.context,
                spelling: "( 1 ++ 2 )".to_owned(),
                recovery: SourceFunctorApplicationRecovery::Normal,
            })
            .into_iter()
            .collect(),
        candidates: vec![SourceFunctorCandidateInput {
            application,
            ordinal: 0,
            symbol: candidate.symbol().clone(),
            contribution: candidate.contribution(),
        }],
        arguments: extracted
            .argument_roots
            .iter()
            .enumerate()
            .map(|(ordinal, root)| {
                let range = ast
                    .node(*root)
                    .ok_or_else(|| "Task253 argument root disappeared".to_owned())?
                    .range;
                let target = source_term
                    .handoff
                    .terms()
                    .iter()
                    .find(|(_, term)| term.parent().is_none() && term.source_range() == range)
                    .map(|(id, _)| id)
                    .ok_or_else(|| {
                        "Task253 argument is not a root of the complete Task252 handoff".to_owned()
                    })?;
                Ok(SourceFunctorArgumentInput {
                    application,
                    ordinal,
                    target: SourceFunctorArgumentTarget::Primary(target),
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        type_requests: vec![
            SourceFunctorTypeRequestInput {
                application,
                candidate: Some(candidate_id),
                request_ordinal: 0,
                kind: SourceFunctorTypeRequestKind::CandidateSignature,
            },
            SourceFunctorTypeRequestInput {
                application,
                candidate: None,
                request_ordinal: 1,
                kind: SourceFunctorTypeRequestKind::ApplicationResultType,
            },
        ],
    };
    mutate(&mut input);
    SourceFunctorApplicationProducer::build(
        input,
        symbols,
        binding_env,
        &source_term.handoff,
        &source_term.arena,
    )
    .map_err(|error| error.to_string())
}

fn unique_shell(
    shells: &DeclarationShellSet,
    node: SurfaceNodeId,
    kind: DeclarationShellKind,
) -> Result<&DeclarationShell, String> {
    let matches = shells
        .declarations()
        .iter()
        .filter(|shell| shell.node_id() == node && shell.kind() == kind && !shell.recovered())
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [shell] => Ok(*shell),
        _ => Err(format!(
            "Task253 local route requires one {kind:?} resolver shell"
        )),
    }
}

fn node_id(ast: &SurfaceAst, site: TypedSiteRef) -> Option<SurfaceNodeId> {
    fn find(ast: &SurfaceAst, id: SurfaceNodeId, index: usize) -> Option<SurfaceNodeId> {
        if id.index() == index {
            return Some(id);
        }
        ast.node(id)?
            .children
            .iter()
            .find_map(|child| find(ast, *child, index))
    }
    find(ast, ast.root()?, site.node().index())
}

fn structural_descendant(ast: &SurfaceAst, root: &SurfaceNode, target: SurfaceNodeId) -> bool {
    root.children.iter().any(|child| {
        *child == target
            || ast
                .node(*child)
                .is_some_and(|node| structural_descendant(ast, node, target))
    })
}

fn subtree_tokens<'a>(ast: &'a SurfaceAst, node: &'a SurfaceNode) -> Vec<&'a str> {
    fn collect<'a>(ast: &'a SurfaceAst, node: &'a SurfaceNode, output: &mut Vec<&'a str>) {
        if let Some(token) = node.token_text() {
            output.push(token);
        } else {
            for child in &node.children {
                if let Some(child) = ast.node(*child) {
                    collect(ast, child, output);
                }
            }
        }
    }
    let mut output = Vec::new();
    collect(ast, node, &mut output);
    output
}

fn unique_direct_token_range(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    spelling: &str,
) -> Result<SourceRange, String> {
    let matches = node
        .children
        .iter()
        .filter_map(|child| ast.node(*child))
        .filter(|child| child.token_text() == Some(spelling))
        .map(|child| child.range)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [range] => Ok(*range),
        _ => Err(format!(
            "Task253 local route requires one `{spelling}` token"
        )),
    }
}
