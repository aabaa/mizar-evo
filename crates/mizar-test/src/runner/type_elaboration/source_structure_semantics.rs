use mizar_checker::source_structure_semantics::{
    SourceStructureClaimInput, SourceStructureDefinitionInput, SourceStructureEqualityInput,
    SourceStructureFieldArgument, SourceStructureInheritanceInput,
    SourceStructureInheritanceMappingInput, SourceStructureInheritanceParent,
    SourceStructureMemberInput, SourceStructureMemberKind, SourceStructureProgramInput,
    SourceStructureSemanticsChecker, SourceStructureSemanticsOutput, SourceStructureTerm,
    SourceStructureType, SourceStructureVariableInput,
};
use mizar_resolve::{
    env::{DefinitionKind, SymbolEnv, SymbolKind},
    resolved_ast::{FullyQualifiedName, LocalSymbolId, ModuleId, SymbolId},
};
use mizar_session::{SourceAnchor, SourceRange};
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeKind};

use super::source_ast::{
    direct_token_texts, qualified_symbol_spelling, structural_child_ids, subtree_has_recovery,
};

#[derive(Debug, Clone)]
struct ExtractedDefinition {
    base_spelling: String,
    symbol: SymbolId,
    members: Vec<ExtractedMember>,
}

#[derive(Debug, Clone)]
struct ExtractedMember {
    spelling: String,
    resolver_spelling: String,
    symbol: SymbolId,
    kind: SourceStructureMemberKind,
    ty: SourceStructureType,
}

#[derive(Debug, Clone)]
struct ExtractedVariable {
    spelling: String,
    symbol: SymbolId,
    ty: SourceStructureType,
}

pub(in crate::runner) fn source_structure_semantics_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Result<SourceStructureSemanticsOutput, String> {
    let input = extract_program(ast, module, symbols)?;
    SourceStructureSemanticsChecker::check(input, symbols)
        .map_err(|error| format!("Step 5C.2 checker rejected source payload: {error}"))
}

pub(in crate::runner) fn source_structure_semantics_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Vec<String> {
    match source_structure_semantics_output(ast, module, symbols) {
        Ok(output) => output
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.detail_key().to_owned())
            .collect(),
        Err(_) => {
            vec!["type_elaboration.checker.source_structure_semantics.invalid_payload".to_owned()]
        }
    }
}

fn extract_program(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Result<SourceStructureProgramInput, String> {
    if symbols.module_id() != &module || ast.nodes().iter().any(|node| node.recovered) {
        return Err("source/module or recovery boundary mismatch".to_owned());
    }

    let (definitions, extracted_definitions) = extract_definitions(ast, symbols)?;
    let inheritances = extract_inheritances(ast, &extracted_definitions)?;
    let (variables, extracted_variables) = extract_variables(ast, &module, &extracted_definitions)?;
    let (terms, claims) = extract_claims(ast, &extracted_definitions, &extracted_variables)?;

    Ok(SourceStructureProgramInput::new(
        ast.source_id,
        module,
        definitions,
        inheritances,
        variables,
        terms,
        claims,
    ))
}

fn extract_definitions(
    ast: &SurfaceAst,
    symbols: &SymbolEnv,
) -> Result<
    (
        Vec<SourceStructureDefinitionInput>,
        Vec<ExtractedDefinition>,
    ),
    String,
> {
    let mut nodes = nodes_of_kind(ast, |kind| {
        matches!(kind, SurfaceNodeKind::StructureDefinition)
    });
    nodes.sort_by_key(|node| node.range.start);
    if nodes.is_empty() {
        return Err("structure program has no definition".to_owned());
    }

    let mut inputs = Vec::new();
    let mut extracted = Vec::new();
    for (source_ordinal, node) in nodes.into_iter().enumerate() {
        let children = structural_child_ids(ast, node);
        let pattern = children
            .iter()
            .filter_map(|id| ast.node(*id))
            .find(|child| matches!(child.kind, SurfaceNodeKind::StructurePattern))
            .ok_or_else(|| "structure definition has no pattern".to_owned())?;
        let pattern_tokens = direct_token_texts(ast, pattern);
        let base_spelling = pattern_tokens
            .first()
            .filter(|spelling| is_identifier_spelling(spelling))
            .cloned()
            .ok_or_else(|| "structure pattern has no base spelling".to_owned())?;
        let parameters = parse_bracket_parameters(&pattern_tokens)?;
        let (structure_symbol, resolver_spelling) =
            symbol_declared_at(symbols, node.range, SymbolKind::Structure)?;

        let mut member_inputs = Vec::new();
        let mut members = Vec::new();
        for child in children
            .iter()
            .filter_map(|id| ast.node(*id))
            .filter(|child| {
                matches!(
                    child.kind,
                    SurfaceNodeKind::StructureField | SurfaceNodeKind::StructureProperty
                )
            })
        {
            let member_tokens = direct_token_texts(ast, child);
            let spelling = member_tokens
                .get(1)
                .filter(|spelling| is_identifier_spelling(spelling))
                .cloned()
                .ok_or_else(|| "structure member has no spelling".to_owned())?;
            if member_tokens.iter().any(|token| token == ":=") {
                return Err("field defaults are outside Step 5C.2".to_owned());
            }
            let kind = match child.kind {
                SurfaceNodeKind::StructureField => SourceStructureMemberKind::Field,
                SurfaceNodeKind::StructureProperty => SourceStructureMemberKind::Property,
                _ => unreachable!(),
            };
            let type_node = one_structural_child_of_kind(ast, child, |kind| {
                matches!(kind, SurfaceNodeKind::TypeExpression)
            })?;
            let ty = extract_type(ast, type_node, &extracted)?;
            let (symbol, resolver_member_spelling) =
                symbol_declared_at(symbols, child.range, SymbolKind::Selector)?;
            let member = ExtractedMember {
                spelling: spelling.clone(),
                resolver_spelling: resolver_member_spelling.clone(),
                symbol: symbol.clone(),
                kind,
                ty: ty.clone(),
            };
            member_inputs.push(SourceStructureMemberInput::new(
                symbol,
                spelling,
                resolver_member_spelling,
                kind,
                ty,
                child.range,
                member_inputs.len(),
                subtree_has_recovery(ast, child),
            ));
            members.push(member);
        }
        if member_inputs.is_empty() {
            return Err("structure definition has no members".to_owned());
        }
        inputs.push(SourceStructureDefinitionInput::new(
            structure_symbol.clone(),
            resolver_spelling,
            parameters,
            member_inputs,
            node.range,
            source_ordinal,
            subtree_has_recovery(ast, node),
        ));
        extracted.push(ExtractedDefinition {
            base_spelling,
            symbol: structure_symbol,
            members,
        });
    }
    Ok((inputs, extracted))
}

fn extract_inheritances(
    ast: &SurfaceAst,
    definitions: &[ExtractedDefinition],
) -> Result<Vec<SourceStructureInheritanceInput>, String> {
    let mut nodes = nodes_of_kind(ast, |kind| {
        matches!(kind, SurfaceNodeKind::InheritanceDefinition)
    });
    nodes.sort_by_key(|node| node.range.start);
    let mut output = Vec::new();
    for (source_ordinal, node) in nodes.into_iter().enumerate() {
        let children = structural_child_ids(ast, node);
        let targets = children
            .iter()
            .filter_map(|id| ast.node(*id))
            .filter(|child| matches!(child.kind, SurfaceNodeKind::InheritanceTarget))
            .collect::<Vec<_>>();
        let [child_target, parent_target] = targets.as_slice() else {
            return Err("inheritance must have exactly two targets".to_owned());
        };
        let child_spelling = one_direct_spelling(ast, child_target)?;
        let child = definition_by_spelling(definitions, &child_spelling)?;
        let parent_spelling = one_direct_spelling(ast, parent_target)?;
        let parent_definition = if parent_spelling == "set" {
            None
        } else {
            Some(definition_by_spelling(definitions, &parent_spelling)?)
        };
        let parent = parent_definition.map_or(SourceStructureInheritanceParent::Set, |parent| {
            SourceStructureInheritanceParent::Structure {
                symbol: parent.symbol.clone(),
                arguments: Vec::new(),
            }
        });
        let explicit = direct_token_texts(ast, node)
            .iter()
            .any(|token| token == "where");
        let mut mappings = Vec::new();
        for mapping_node in children
            .iter()
            .filter_map(|id| ast.node(*id))
            .filter(|child| {
                matches!(
                    child.kind,
                    SurfaceNodeKind::FieldRedefinition | SurfaceNodeKind::PropertyRedefinition
                )
            })
        {
            let tokens = direct_token_texts(ast, mapping_node);
            if tokens.len() != 5
                || !matches!(tokens[0].as_str(), "field" | "property")
                || tokens[2] != "from"
                || tokens[4] != ";"
            {
                return Err("typed or malformed inheritance mapping is unsupported".to_owned());
            }
            let child_spelling = tokens[1].clone();
            let parent_spelling = tokens[3].clone();
            let kind = if tokens[0] == "field" {
                SourceStructureMemberKind::Field
            } else {
                SourceStructureMemberKind::Property
            };
            let child_member = unique_member_by_spelling(child, &child_spelling);
            let from_it = parent_spelling == "it";
            let parent_member = if from_it {
                None
            } else {
                parent_definition
                    .and_then(|parent| unique_member_by_spelling(parent, &parent_spelling))
            };
            mappings.push(SourceStructureInheritanceMappingInput::new(
                child_member.map(|member| member.symbol.clone()),
                child_spelling,
                child_member.map(|member| member.resolver_spelling.clone()),
                parent_member.map(|member| member.symbol.clone()),
                parent_spelling,
                parent_member.map(|member| member.resolver_spelling.clone()),
                from_it,
                kind,
                mapping_node.range,
                mappings.len(),
                subtree_has_recovery(ast, mapping_node),
            ));
        }
        if explicit == mappings.is_empty() {
            return Err("inheritance mapping block shape is unsupported".to_owned());
        }
        output.push(SourceStructureInheritanceInput::new(
            child.symbol.clone(),
            parent,
            explicit,
            mappings,
            false,
            node.range,
            source_ordinal,
            subtree_has_recovery(ast, node),
        ));
    }
    Ok(output)
}

fn extract_variables(
    ast: &SurfaceAst,
    module: &ModuleId,
    definitions: &[ExtractedDefinition],
) -> Result<(Vec<SourceStructureVariableInput>, Vec<ExtractedVariable>), String> {
    let mut segments = nodes_of_kind(ast, |kind| {
        matches!(
            kind,
            SurfaceNodeKind::ReserveSegment
                | SurfaceNodeKind::QuantifierVariableSegment
                | SurfaceNodeKind::QualifiedVariableSegment
        )
    });
    segments.sort_by_key(|node| node.range.start);
    let mut inputs = Vec::new();
    let mut extracted = Vec::new();
    for segment in segments {
        let tokens = direct_token_texts(ast, segment);
        let boundary = tokens
            .iter()
            .position(|token| matches!(token.as_str(), "for" | "be" | "being"));
        let names = tokens[..boundary.unwrap_or(tokens.len())]
            .iter()
            .filter(|token| token.as_str() != ",")
            .cloned()
            .collect::<Vec<_>>();
        if names.is_empty() || names.iter().any(|name| !is_identifier_spelling(name)) {
            return Err("variable segment has unsupported spelling".to_owned());
        }
        let type_node = structural_child_ids(ast, segment)
            .into_iter()
            .filter_map(|id| ast.node(id))
            .find(|node| matches!(node.kind, SurfaceNodeKind::TypeExpression));
        let Some(type_node) = type_node else {
            if names.iter().all(|name| {
                extracted
                    .iter()
                    .any(|variable: &ExtractedVariable| &variable.spelling == name)
            }) {
                continue;
            }
            return Err("untyped variable has no earlier exact declaration".to_owned());
        };
        let ty = extract_type(ast, type_node, definitions)?;
        for name in names {
            if extracted
                .iter()
                .any(|variable: &ExtractedVariable| variable.spelling == name)
            {
                continue;
            }
            let symbol = source_local_variable_symbol(module, &name);
            inputs.push(SourceStructureVariableInput::new(
                symbol.clone(),
                name.clone(),
                ty.clone(),
                segment.range,
                inputs.len(),
                subtree_has_recovery(ast, segment),
            ));
            extracted.push(ExtractedVariable {
                spelling: name,
                symbol,
                ty: ty.clone(),
            });
        }
    }
    Ok((inputs, extracted))
}

fn extract_claims(
    ast: &SurfaceAst,
    definitions: &[ExtractedDefinition],
    variables: &[ExtractedVariable],
) -> Result<(Vec<SourceStructureTerm>, Vec<SourceStructureClaimInput>), String> {
    let mut theorems = nodes_of_kind(ast, |kind| matches!(kind, SurfaceNodeKind::TheoremItem));
    theorems.sort_by_key(|node| node.range.start);
    let mut terms = Vec::new();
    let mut claims = Vec::new();
    for theorem in theorems {
        let formula = structural_child_ids(ast, theorem)
            .into_iter()
            .filter_map(|id| ast.node(id))
            .find(|node| matches!(node.kind, SurfaceNodeKind::FormulaExpression))
            .ok_or_else(|| "theorem has no proposition formula".to_owned())?;
        let proposition = extract_equality(ast, formula, definitions, variables)?;
        terms.push(proposition.left().clone());
        terms.push(proposition.right().clone());

        let mut conclusion_nodes = Vec::new();
        collect_descendants(ast, theorem, &mut |node| {
            if matches!(node.kind, SurfaceNodeKind::ConclusionStatement) {
                conclusion_nodes.push(node);
            }
        });
        conclusion_nodes.sort_by_key(|node| node.range.start);
        let mut conclusions = Vec::new();
        for conclusion in conclusion_nodes {
            let proposition_node = first_descendant(ast, conclusion, |kind| {
                matches!(kind, SurfaceNodeKind::Proposition)
            })
            .ok_or_else(|| "thus conclusion has no proposition".to_owned())?;
            let equality = extract_equality(ast, proposition_node, definitions, variables)?;
            terms.push(equality.left().clone());
            terms.push(equality.right().clone());
            conclusions.push(equality);
        }
        if conclusions.is_empty() {
            return Err("Step 5C.2 theorem has no thus conclusion".to_owned());
        }
        claims.push(SourceStructureClaimInput::new(
            proposition,
            conclusions,
            theorem.range,
            claims.len(),
            subtree_has_recovery(ast, theorem),
        ));
    }
    terms.sort_by_key(SourceStructureTerm::source_ordinal);
    if terms
        .windows(2)
        .any(|window| window[0].source_ordinal() >= window[1].source_ordinal())
    {
        return Err("top-level structure terms are not strictly source ordered".to_owned());
    }
    Ok((terms, claims))
}

fn extract_equality(
    ast: &SurfaceAst,
    root: &SurfaceNode,
    definitions: &[ExtractedDefinition],
    variables: &[ExtractedVariable],
) -> Result<SourceStructureEqualityInput, String> {
    let predicate = first_descendant(ast, root, |kind| {
        matches!(kind, SurfaceNodeKind::BuiltinPredicateApplication)
    })
    .ok_or_else(|| "claim is not a builtin equality".to_owned())?;
    if direct_token_texts(ast, predicate).as_slice() != ["="] {
        return Err("claim predicate is not exact equality".to_owned());
    }
    let children = structural_child_ids(ast, predicate);
    let [left_id, right_id] = children.as_slice() else {
        return Err("equality does not have two terms".to_owned());
    };
    let left_node = ast
        .node(*left_id)
        .ok_or_else(|| "equality left term disappeared".to_owned())?;
    let right_node = ast
        .node(*right_id)
        .ok_or_else(|| "equality right term disappeared".to_owned())?;
    let (left, _) = extract_term(ast, left_node, definitions, variables)?;
    let (right, _) = extract_term(ast, right_node, definitions, variables)?;
    Ok(SourceStructureEqualityInput::new(
        left,
        right,
        predicate.range,
        predicate.range.start,
        subtree_has_recovery(ast, predicate),
    ))
}

fn extract_term(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    definitions: &[ExtractedDefinition],
    variables: &[ExtractedVariable],
) -> Result<(SourceStructureTerm, Option<SourceStructureType>), String> {
    if subtree_has_recovery(ast, node) {
        return Err("recovered term is not admitted".to_owned());
    }
    match &node.kind {
        SurfaceNodeKind::TermExpression | SurfaceNodeKind::ParenthesizedTerm => {
            let children = structural_child_ids(ast, node);
            let [child] = children.as_slice() else {
                return Err("term shell does not have one structural child".to_owned());
            };
            extract_term(
                ast,
                ast.node(*child)
                    .ok_or_else(|| "term shell child disappeared".to_owned())?,
                definitions,
                variables,
            )
        }
        SurfaceNodeKind::TermReference => {
            let spelling = one_direct_spelling(ast, node)?;
            let variable = variables
                .iter()
                .find(|variable| variable.spelling == spelling)
                .ok_or_else(|| format!("unresolved structure variable `{spelling}`"))?;
            Ok((
                SourceStructureTerm::Variable {
                    symbol: variable.symbol.clone(),
                    spelling,
                    source_range: node.range,
                    source_ordinal: node.range.start,
                    recovered: false,
                },
                Some(variable.ty.clone()),
            ))
        }
        SurfaceNodeKind::StructureConstructor => {
            let children = structural_child_ids(ast, node);
            if children
                .iter()
                .filter_map(|id| ast.node(*id))
                .any(|child| matches!(child.kind, SurfaceNodeKind::TypeArguments))
            {
                return Err("constructor type arguments are outside Step 5C.2".to_owned());
            }
            let symbol_node = children
                .iter()
                .filter_map(|id| ast.node(*id))
                .find(|child| matches!(child.kind, SurfaceNodeKind::QualifiedSymbol))
                .ok_or_else(|| "constructor has no structure symbol".to_owned())?;
            let spelling = qualified_symbol_spelling(ast, symbol_node)
                .map_err(|()| "constructor structure spelling is malformed".to_owned())?;
            let definition = definition_by_spelling(definitions, &spelling)?;
            let mut arguments = Vec::new();
            for argument_node in children
                .iter()
                .filter_map(|id| ast.node(*id))
                .filter(|child| matches!(child.kind, SurfaceNodeKind::FieldArgument))
            {
                let argument_spelling = direct_token_texts(ast, argument_node)
                    .first()
                    .cloned()
                    .ok_or_else(|| "field argument has no spelling".to_owned())?;
                let value_node = one_structural_child_of_kind(ast, argument_node, |kind| {
                    matches!(kind, SurfaceNodeKind::TermExpression)
                })?;
                let (value, _) = extract_term(ast, value_node, definitions, variables)?;
                let member = member_by_spelling(definition, &argument_spelling)
                    .filter(|member| member.kind == SourceStructureMemberKind::Field)
                    .map(|member| member.symbol.clone());
                let resolver_spelling = member
                    .as_ref()
                    .and_then(|symbol| {
                        definition
                            .members
                            .iter()
                            .find(|candidate| &candidate.symbol == symbol)
                    })
                    .map(|member| member.resolver_spelling.clone());
                arguments.push(SourceStructureFieldArgument::new(
                    member,
                    argument_spelling,
                    resolver_spelling,
                    Box::new(value),
                    argument_node.range,
                    arguments.len(),
                    false,
                ));
            }
            let ty = SourceStructureType::Structure {
                symbol: definition.symbol.clone(),
                arguments: Vec::new(),
            };
            Ok((
                SourceStructureTerm::Constructor {
                    structure: definition.symbol.clone(),
                    type_arguments: Vec::new(),
                    arguments,
                    source_range: node.range,
                    source_ordinal: node.range.start,
                    recovered: false,
                },
                Some(ty),
            ))
        }
        SurfaceNodeKind::SelectorAccess => {
            let children = structural_child_ids(ast, node);
            let subject_node = children
                .first()
                .and_then(|id| ast.node(*id))
                .ok_or_else(|| "selector subject disappeared".to_owned())?;
            if children.len() != 1 {
                return Err("selector calls are outside Step 5C.2".to_owned());
            }
            let tokens = direct_token_texts(ast, node);
            let [dot, spelling] = tokens.as_slice() else {
                return Err("selector token shape is unsupported".to_owned());
            };
            if dot != "." || !is_identifier_spelling(spelling) {
                return Err("selector spelling is malformed".to_owned());
            }
            let (subject, subject_ty) = extract_term(ast, subject_node, definitions, variables)?;
            let definition = structure_definition_for_type(definitions, subject_ty.as_ref())?;
            let member = unique_member_by_spelling(definition, spelling);
            let selected_ty = member.map(|member| member.ty.clone());
            Ok((
                SourceStructureTerm::Select {
                    subject: Box::new(subject),
                    member: member.map(|member| member.symbol.clone()),
                    spelling: spelling.clone(),
                    resolver_spelling: member.map(|member| member.resolver_spelling.clone()),
                    source_range: node.range,
                    source_ordinal: node.range.start,
                    recovered: false,
                },
                selected_ty,
            ))
        }
        SurfaceNodeKind::StructureUpdate => {
            let children = structural_child_ids(ast, node);
            let subject_node = children
                .first()
                .and_then(|id| ast.node(*id))
                .ok_or_else(|| "update subject disappeared".to_owned())?;
            let updates = children
                .iter()
                .filter_map(|id| ast.node(*id))
                .filter(|child| matches!(child.kind, SurfaceNodeKind::FieldUpdate))
                .collect::<Vec<_>>();
            let [update] = updates.as_slice() else {
                return Err("Step 5C.2 requires exactly one field update".to_owned());
            };
            let (subject, subject_ty) = extract_term(ast, subject_node, definitions, variables)?;
            let definition = structure_definition_for_type(definitions, subject_ty.as_ref())?;
            let tokens = direct_token_texts(ast, update);
            let [spelling, assign] = tokens.as_slice() else {
                return Err("field update path is unsupported".to_owned());
            };
            if assign != ":=" || !is_identifier_spelling(spelling) {
                return Err("field update spelling is malformed".to_owned());
            }
            let value_node = one_structural_child_of_kind(ast, update, |kind| {
                matches!(kind, SurfaceNodeKind::TermExpression)
            })?;
            let (value, _) = extract_term(ast, value_node, definitions, variables)?;
            let member = unique_member_by_spelling(definition, spelling)
                .filter(|member| member.kind == SourceStructureMemberKind::Field)
                .ok_or_else(|| "update field did not resolve uniquely".to_owned())?;
            Ok((
                SourceStructureTerm::Update {
                    subject: Box::new(subject),
                    member: Some(member.symbol.clone()),
                    spelling: spelling.clone(),
                    resolver_spelling: Some(member.resolver_spelling.clone()),
                    value: Box::new(value),
                    source_range: node.range,
                    source_ordinal: node.range.start,
                    recovered: false,
                },
                subject_ty,
            ))
        }
        kind => Err(format!("unsupported Step 5C.2 term node {kind:?}")),
    }
}

fn extract_type(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    definitions: &[ExtractedDefinition],
) -> Result<SourceStructureType, String> {
    if subtree_has_recovery(ast, node) {
        return Err("recovered type is not admitted".to_owned());
    }
    let tokens = subtree_token_texts(ast, node);
    match tokens.as_slice() {
        [set] if set == "set" => Ok(SourceStructureType::Set),
        [spelling] => {
            let definition = definition_by_spelling(definitions, spelling)?;
            Ok(SourceStructureType::Structure {
                symbol: definition.symbol.clone(),
                arguments: Vec::new(),
            })
        }
        [spelling, open, argument, close]
            if open == "[" && close == "]" && is_identifier_spelling(argument) =>
        {
            let definition = definition_by_spelling(definitions, spelling)?;
            Ok(SourceStructureType::Structure {
                symbol: definition.symbol.clone(),
                arguments: vec![SourceStructureType::Set],
            })
        }
        _ => Err(format!("unsupported Step 5C.2 type shape {tokens:?}")),
    }
}

fn symbol_declared_at(
    symbols: &SymbolEnv,
    range: SourceRange,
    kind: SymbolKind,
) -> Result<(SymbolId, String), String> {
    let candidates = symbols
        .symbols()
        .iter()
        .filter(|entry| {
            entry.kind() == kind
                && entry.symbol().module() == symbols.module_id()
                && matches!(entry.origin().anchor(), SourceAnchor::Range(origin) if *origin == range)
        })
        .collect::<Vec<_>>();
    let [entry] = candidates.as_slice() else {
        return Err(format!(
            "expected one {kind:?} resolver row at {}..{}, found {}",
            range.start,
            range.end,
            candidates.len()
        ));
    };
    let definition = symbols
        .definitions()
        .by_symbol(entry.symbol())
        .ok_or_else(|| "resolver symbol has no definition row".to_owned())?;
    let expected_definition_kind = if kind == SymbolKind::Structure {
        DefinitionKind::Structure
    } else {
        DefinitionKind::Selector
    };
    if definition.kind() != expected_definition_kind
        || symbols
            .definitions()
            .iter()
            .filter(|candidate| candidate.symbol() == entry.symbol())
            .count()
            != 1
    {
        return Err("resolver definition identity is not exact".to_owned());
    }
    Ok((entry.symbol().clone(), entry.primary_spelling().to_owned()))
}

fn definition_by_spelling<'a>(
    definitions: &'a [ExtractedDefinition],
    spelling: &str,
) -> Result<&'a ExtractedDefinition, String> {
    let candidates = definitions
        .iter()
        .filter(|definition| definition.base_spelling == spelling)
        .collect::<Vec<_>>();
    let [definition] = candidates.as_slice() else {
        return Err(format!(
            "structure `{spelling}` did not resolve uniquely in source definitions"
        ));
    };
    Ok(definition)
}

fn structure_definition_for_type<'a>(
    definitions: &'a [ExtractedDefinition],
    ty: Option<&SourceStructureType>,
) -> Result<&'a ExtractedDefinition, String> {
    let Some(SourceStructureType::Structure { symbol, .. }) = ty else {
        return Err("selector/update subject has no exact structure type".to_owned());
    };
    definitions
        .iter()
        .find(|definition| &definition.symbol == symbol)
        .ok_or_else(|| "subject structure identity is not source declared".to_owned())
}

fn member_by_spelling<'a>(
    definition: &'a ExtractedDefinition,
    spelling: &str,
) -> Option<&'a ExtractedMember> {
    definition
        .members
        .iter()
        .find(|member| member.spelling == spelling)
}

fn unique_member_by_spelling<'a>(
    definition: &'a ExtractedDefinition,
    spelling: &str,
) -> Option<&'a ExtractedMember> {
    let mut matches = definition
        .members
        .iter()
        .filter(|member| member.spelling == spelling);
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

fn parse_bracket_parameters(tokens: &[String]) -> Result<Vec<String>, String> {
    match tokens {
        [_] => Ok(Vec::new()),
        [_, open, parameter, close]
            if open == "[" && close == "]" && is_identifier_spelling(parameter) =>
        {
            Ok(vec![parameter.clone()])
        }
        _ => Err(format!("unsupported structure pattern {tokens:?}")),
    }
}

fn source_local_variable_symbol(module: &ModuleId, spelling: &str) -> SymbolId {
    SymbolId::new(
        module.clone(),
        LocalSymbolId::new(format!("step5c2/variable/{spelling}")),
        FullyQualifiedName::new(format!(
            "{}::step5c2::variable::{spelling}",
            module.path().as_str()
        )),
    )
}

fn one_direct_spelling(ast: &SurfaceAst, node: &SurfaceNode) -> Result<String, String> {
    let tokens = direct_token_texts(ast, node);
    let [spelling] = tokens.as_slice() else {
        return Err(format!("expected one source spelling, got {tokens:?}"));
    };
    if is_identifier_spelling(spelling) || spelling == "set" {
        Ok(spelling.clone())
    } else {
        Err(format!("invalid source spelling `{spelling}`"))
    }
}

fn one_structural_child_of_kind<'a>(
    ast: &'a SurfaceAst,
    node: &SurfaceNode,
    predicate: impl Fn(&SurfaceNodeKind) -> bool,
) -> Result<&'a SurfaceNode, String> {
    let children = structural_child_ids(ast, node)
        .into_iter()
        .filter_map(|id| ast.node(id))
        .filter(|child| predicate(&child.kind))
        .collect::<Vec<_>>();
    let [child] = children.as_slice() else {
        return Err("expected exactly one structural child".to_owned());
    };
    Ok(child)
}

fn nodes_of_kind(
    ast: &SurfaceAst,
    predicate: impl Fn(&SurfaceNodeKind) -> bool,
) -> Vec<&SurfaceNode> {
    ast.nodes()
        .iter()
        .filter(|node| predicate(&node.kind))
        .collect()
}

fn first_descendant<'a>(
    ast: &'a SurfaceAst,
    node: &'a SurfaceNode,
    predicate: impl Fn(&SurfaceNodeKind) -> bool + Copy,
) -> Option<&'a SurfaceNode> {
    if predicate(&node.kind) {
        return Some(node);
    }
    node.children
        .iter()
        .filter_map(|id| ast.node(*id))
        .find_map(|child| first_descendant(ast, child, predicate))
}

fn collect_descendants<'a>(
    ast: &'a SurfaceAst,
    node: &'a SurfaceNode,
    visitor: &mut impl FnMut(&'a SurfaceNode),
) {
    for child in node.children.iter().filter_map(|id| ast.node(*id)) {
        visitor(child);
        collect_descendants(ast, child, visitor);
    }
}

fn subtree_token_texts(ast: &SurfaceAst, node: &SurfaceNode) -> Vec<String> {
    let mut output = Vec::new();
    collect_subtree_tokens(ast, node, &mut output);
    output
}

fn collect_subtree_tokens(ast: &SurfaceAst, node: &SurfaceNode, output: &mut Vec<String>) {
    if let Some(text) = node.token_text() {
        output.push(text.to_owned());
        return;
    }
    for child in node.children.iter().filter_map(|id| ast.node(*id)) {
        collect_subtree_tokens(ast, child, output);
    }
}

fn is_identifier_spelling(spelling: &str) -> bool {
    let mut chars = spelling.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}
