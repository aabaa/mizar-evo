use mizar_checker::{
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_structure_definition::{
        SourceStructureDefinitionHandoffInput, SourceStructureDefinitionId,
        SourceStructureDefinitionInput, SourceStructureDefinitionProducer,
        SourceStructureDefinitionRecovery, SourceStructureInheritanceId,
        SourceStructureInheritanceInput, SourceStructureMappingId, SourceStructureMappingInput,
        SourceStructureMemberId, SourceStructureMemberInput, SourceStructureMemberKind,
    },
    source_type::{
        SourceTypeApplicationForm, SourceTypeExpressionInput, SourceTypeHead,
        SourceTypeStructureMemberHandoffInput, SourceTypeStructureMemberId,
        SourceTypeStructureMemberInput, SourceTypeStructureMemberProducer,
    },
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, NodeRecoveryState,
        TypeDiagnosticTable, TypeFactTable, TypeTable, TypedArena, TypedArenaBuilder, TypedAst,
        TypedAstError, TypedAstParts, TypedNode, TypedNodeId, TypedSiteRef, TypingState,
    },
};
use mizar_resolve::{
    declarations::{DeclarationShellKind, DeclarationShellSet},
    env::{
        DefinitionId, DefinitionKind, NamespacePath, SourceContributionId, SymbolEnv, SymbolKind,
    },
    resolved_ast::{ModuleId, SymbolId},
    symbols::{SignatureProjectionExtractor, SymbolOverloadPolicy},
};
use mizar_session::SourceAnchor;
use mizar_syntax::{SurfaceAst, SurfaceNodeKind, SurfaceTokenKind};

use super::checker_handoff::{assemble_empty_resolved_typed_ast, source_syntax_receiver};

#[cfg(test)]
pub(in crate::runner) const SOURCE_STRUCTURE_DEFINITION_TEXT: &str = concat!(
    "definition\n",
    "  struct Task263Base where\n",
    "    field carrier -> set;\n",
    "    property marker -> set;\n",
    "  end;\n",
    "\n",
    "  struct Task263Derived where\n",
    "    field carrier -> set;\n",
    "    property marker -> set;\n",
    "  end;\n",
    "\n",
    "  inherit Task263Derived extends Task263Base where\n",
    "    field carrier from carrier;\n",
    "    property marker from marker;\n",
    "  end;\n",
    "end;\n",
);

const INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.source_structure_definition.invalid_payload";

#[derive(Debug)]
pub(in crate::runner) struct SourceStructureDefinitionRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; variants are private corruption seams.
pub(in crate::runner) enum SourceStructureDefinitionRouteMutation {
    None,
    WrongSurfaceRow(usize),
    WrongSurfaceRange(usize),
    WrongSurfaceRecovery(usize),
    WrongSurfaceChildren(usize),
    WrongSurfaceRoot,
    WrongShellRow(usize),
    WrongProjectionRow(usize),
    WrongResolverDefinition,
    WrongLowerMember,
    WrongDefinition,
    WrongMapping,
    WrongArena,
}

macro_rules! token_row {
    ($kind:ident, $text:literal) => {
        (SurfaceTokenKind::$kind, $text)
    };
}

const STRUCTURE_TERMINALS: &[(SurfaceTokenKind, &str)] = &[
    token_row!(ReservedWord, "definition"),
    token_row!(ReservedWord, "struct"),
    token_row!(Identifier, "Task263Base"),
    token_row!(ReservedWord, "where"),
    token_row!(ReservedWord, "field"),
    token_row!(Identifier, "carrier"),
    token_row!(ReservedSymbol, "->"),
    token_row!(ReservedWord, "set"),
    token_row!(ReservedSymbol, ";"),
    token_row!(ReservedWord, "property"),
    token_row!(Identifier, "marker"),
    token_row!(ReservedSymbol, "->"),
    token_row!(ReservedWord, "set"),
    token_row!(ReservedSymbol, ";"),
    token_row!(ReservedWord, "end"),
    token_row!(ReservedSymbol, ";"),
    token_row!(ReservedWord, "struct"),
    token_row!(Identifier, "Task263Derived"),
    token_row!(ReservedWord, "where"),
    token_row!(ReservedWord, "field"),
    token_row!(Identifier, "carrier"),
    token_row!(ReservedSymbol, "->"),
    token_row!(ReservedWord, "set"),
    token_row!(ReservedSymbol, ";"),
    token_row!(ReservedWord, "property"),
    token_row!(Identifier, "marker"),
    token_row!(ReservedSymbol, "->"),
    token_row!(ReservedWord, "set"),
    token_row!(ReservedSymbol, ";"),
    token_row!(ReservedWord, "end"),
    token_row!(ReservedSymbol, ";"),
    token_row!(ReservedWord, "inherit"),
    token_row!(UserSymbol, "Task263Derived"),
    token_row!(ReservedWord, "extends"),
    token_row!(UserSymbol, "Task263Base"),
    token_row!(ReservedWord, "where"),
    token_row!(ReservedWord, "field"),
    token_row!(Identifier, "carrier"),
    token_row!(ReservedWord, "from"),
    token_row!(Identifier, "carrier"),
    token_row!(ReservedSymbol, ";"),
    token_row!(ReservedWord, "property"),
    token_row!(Identifier, "marker"),
    token_row!(ReservedWord, "from"),
    token_row!(Identifier, "marker"),
    token_row!(ReservedSymbol, ";"),
    token_row!(ReservedWord, "end"),
    token_row!(ReservedSymbol, ";"),
    token_row!(ReservedWord, "end"),
    token_row!(ReservedSymbol, ";"),
];

pub(in crate::runner) fn source_structure_definition_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Vec<String>> {
    match source_structure_definition_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceStructureDefinitionRouteMutation::None,
    ) {
        None => None,
        Some(Ok(output)) if route_output_is_exact(&output) => Some(Vec::new()),
        Some(Ok(_)) | Some(Err(_)) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_structure_definition_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceStructureDefinitionRouteOutput, String>> {
    source_structure_definition_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceStructureDefinitionRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_structure_definition_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceStructureDefinitionRouteMutation,
) -> Option<Result<SourceStructureDefinitionRouteOutput, String>> {
    source_structure_definition_output_impl(ast, module, shells, symbols, source_text, mutation)
}

fn source_structure_definition_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceStructureDefinitionRouteMutation,
) -> Option<Result<SourceStructureDefinitionRouteOutput, String>> {
    let preprocessed = mizar_lexer::preprocess_source_for_lexing(source_text);
    if !preprocessed.diagnostics.is_empty() {
        return None;
    }
    let raw = mizar_lexer::scan_raw(&preprocessed.lexical_text).ok()?;
    let lexical_env = mizar_lexer::build_lexical_environment(&[], &[]).ok()?;
    let locals = mizar_lexer::collect_local_lexical_declarations(
        &raw,
        mizar_lexer::ModuleId::new(module.path().as_str()),
    );
    let scopes = mizar_lexer::build_scope_skeleton(&raw);
    let tokens = mizar_lexer::disambiguate_with_local_declarations(
        &raw,
        &lexical_env,
        &locals,
        &mizar_lexer::ParserLexContext::general(),
        &scopes,
    );
    if !scopes.diagnostics.is_empty()
        || !tokens.diagnostics().is_empty()
        || tokens.tokens().len() != 50
        || !tokens
            .tokens()
            .iter()
            .zip(STRUCTURE_TERMINALS)
            .all(|(actual, (kind, text))| {
                let kind = match kind {
                    SurfaceTokenKind::ReservedWord => mizar_lexer::TokenKind::ReservedWord,
                    SurfaceTokenKind::ReservedSymbol => mizar_lexer::TokenKind::ReservedSymbol,
                    SurfaceTokenKind::Identifier => mizar_lexer::TokenKind::Identifier,
                    SurfaceTokenKind::UserSymbol => mizar_lexer::TokenKind::UserSymbol,
                    _ => return false,
                };
                actual.kind() == kind && actual.lexeme() == *text
            })
    {
        return None;
    }
    let source_matches = || -> Option<()> {
        if ast.canonical_bytes().is_none() || ast.token_nodes().len() != tokens.tokens().len() {
            return None;
        }
        for (actual, supplied) in tokens.tokens().iter().zip(ast.token_views()) {
            let mapped = preprocessed
                .preprocess_map
                .source_ranges_for_lexical(actual.span())?;
            let [range] = mapped.as_slice() else {
                return None;
            };
            let token = supplied.as_token()?;
            if token.text.as_ref() != actual.lexeme()
                || supplied.range().start != range.start
                || supplied.range().end != range.end
                || source_text.get(range.start..range.end) != Some(actual.lexeme())
            {
                return None;
            }
        }
        Some(())
    };
    if source_matches().is_none() {
        return Some(Err("Task263 source association is invalid".to_owned()));
    }
    let terminal_range = |ordinal: usize, range: mizar_session::SourceRange| {
        preprocessed
            .preprocess_map
            .source_ranges_for_lexical(tokens.tokens()[ordinal].span())
            .is_some_and(|mapped| {
                mapped.len() == 1 && mapped[0].start == range.start && mapped[0].end == range.end
            })
    };
    Some(build_output(
        ast,
        module,
        shells,
        symbols,
        mutation,
        &terminal_range,
    ))
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    mutation: SourceStructureDefinitionRouteMutation,
    terminal_range: &dyn Fn(usize, mizar_session::SourceRange) -> bool,
) -> Result<SourceStructureDefinitionRouteOutput, String> {
    let syntax = source_syntax_receiver(ast, &module)?;
    let replay_shells =
        mizar_resolve::declarations::DeclarationShellCollector::new(ast, &module).collect();
    if structure_roles_are_exact(ast, &syntax, &replay_shells, mutation, terminal_range).is_none() {
        return Err("Task263 Surface profile is not exact".to_owned());
    }
    validate_shells_and_projections(ast, &module, shells, mutation)?;
    let identities = resolver_identities(symbols)?;
    let replay = SignatureProjectionExtractor::new(
        ast,
        &replay_shells,
        NamespacePath::new(module.path().as_str()),
    )
    .collect(&module);
    if &replay_shells != shells || !replay.diagnostics().is_empty() || &replay.into_env() != symbols
    {
        return Err("Task263 source association is invalid".to_owned());
    }
    let arena = task263_arena(&syntax, mutation)?;
    let mut lower_input = source_type_input(&syntax, module.clone())?;
    if mutation == SourceStructureDefinitionRouteMutation::WrongLowerMember {
        lower_input.members[0].member_site = node_site(56);
    }
    let source_type = SourceTypeStructureMemberProducer::build(lower_input, &arena)
        .map_err(|error| format!("Task249S source type: {error}"))?;
    let mut typed_ast = TypedAst::try_new(TypedAstParts {
        source_id: ast.source_id,
        module_id: module.clone(),
        resolved_root: None,
        source_context: None,
        source_type: Some(source_type),
        source_attribute: None,
        nodes: arena,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| format!("Task249S typed base: {error}"))?;
    let mut input = checker_input(&syntax, module, &identities)?;
    match mutation {
        SourceStructureDefinitionRouteMutation::WrongResolverDefinition => {
            input.members[3].definition = input.members[2].definition
        }
        SourceStructureDefinitionRouteMutation::WrongDefinition => {
            input.definitions[0].source_ordinal = 1
        }
        SourceStructureDefinitionRouteMutation::WrongMapping => input.mappings[0].path.clear(),
        _ => {}
    }
    let projection = SourceStructureDefinitionProducer::build(
        input,
        symbols,
        typed_ast
            .source_type()
            .ok_or_else(|| "Task249S handoff disappeared".to_owned())?,
        typed_ast.initial_obligations(),
        typed_ast.nodes(),
    )
    .map_err(|error| format!("Task263 structure definition: {error}"))?;
    typed_ast = typed_ast
        .with_source_structure_definition(projection)
        .map_err(|error| format!("Task263 typed installation: {error}"))?;
    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.definition.structure"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)
        .map_err(|error| format!("Task263 final assembly: {error}"))?;
    Ok(SourceStructureDefinitionRouteOutput {
        typed_ast,
        resolved,
    })
}

fn structure_roles_are_exact(
    ast: &SurfaceAst,
    syntax: &TypedAst,
    shells: &DeclarationShellSet,
    mutation: SourceStructureDefinitionRouteMutation,
    terminal_range: &dyn Fn(usize, mizar_session::SourceRange) -> bool,
) -> Option<()> {
    use SurfaceNodeKind as K;
    let root = if mutation == SourceStructureDefinitionRouteMutation::WrongSurfaceRoot {
        None
    } else {
        syntax.nodes().root()
    }?;
    if ast.expression_root().is_some() {
        return None;
    }
    let mut seen = std::collections::BTreeSet::new();
    let obs = |id: TypedNodeId, expected: K, seen: &mut std::collections::BTreeSet<usize>| {
        let row = syntax
            .source_node_entries()?
            .find(|row| row.site().index() == id.index())?;
        let node = syntax.nodes().node(id)?;
        let changed_kind = (mutation
            == SourceStructureDefinitionRouteMutation::WrongSurfaceRow(id.index()))
        .then(|| match row.kind() {
            SurfaceNodeKind::Token(actual) => {
                let mut token = actual.clone();
                token.kind = if token.kind == SurfaceTokenKind::Identifier {
                    SurfaceTokenKind::ReservedWord
                } else {
                    SurfaceTokenKind::Identifier
                };
                SurfaceNodeKind::Token(token)
            }
            SurfaceNodeKind::Root => SurfaceNodeKind::CompilationUnit,
            _ => SurfaceNodeKind::Root,
        });
        let kind = changed_kind.as_ref().unwrap_or(row.kind());
        let mut range = row.range();
        let mut recovery = row.recovery();
        let mut children = std::borrow::Cow::Borrowed(node.children.as_slice());
        if mutation == SourceStructureDefinitionRouteMutation::WrongSurfaceRange(id.index()) {
            range.end = range.end.saturating_add(1);
        }
        if mutation == SourceStructureDefinitionRouteMutation::WrongSurfaceRecovery(id.index()) {
            recovery = NodeRecoveryState::Recovered;
        }
        if mutation == SourceStructureDefinitionRouteMutation::WrongSurfaceChildren(id.index()) {
            if children.len() > 1 {
                children.to_mut().rotate_left(1);
            } else {
                children.to_mut().push(id);
            }
        }
        if kind != &expected
            || range.source_id != syntax.source_id()
            || range.start >= range.end
            || (matches!(kind, K::Token(_))
                && !terminal_range(
                    ast.token_nodes()
                        .iter()
                        .position(|token| token.index() == id.index())?,
                    range,
                ))
            || recovery != NodeRecoveryState::Normal
            || !seen.insert(id.index())
        {
            return None;
        }
        if let (Some(first), Some(last)) = (children.first(), children.last()) {
            let child_row = |id: &TypedNodeId| {
                syntax
                    .source_node_entries()?
                    .find(|row| row.site().index() == id.index())
            };
            if range.start != child_row(first)?.range().start
                || range.end != child_row(last)?.range().end
            {
                return None;
            }
            let mut previous_end = range.start;
            for (ordinal, child) in children.iter().enumerate() {
                let child = child_row(child)?;
                let alias = *kind == SurfaceNodeKind::Root
                    && matches!(child.kind(), SurfaceNodeKind::Token(_));
                if child.range().source_id != range.source_id
                    || child.range().start < range.start
                    || child.range().end > range.end
                    || (!alias
                        && (child.parent() != Some(row.site()) || child.child_ordinal() != ordinal))
                    || (*kind != SurfaceNodeKind::Root && previous_end > child.range().start)
                {
                    return None;
                }
                previous_end = child.range().end;
            }
        }
        Some(children)
    };
    let mut next = 0;
    let terms =
        |ids: &[TypedNodeId], next: &mut usize, seen: &mut std::collections::BTreeSet<usize>| {
            for id in ids {
                let (kind, text) = STRUCTURE_TERMINALS.get(*next)?;
                let children = obs(
                    *id,
                    K::Token(mizar_syntax::ast::SurfaceToken::new(*kind, *text)),
                    seen,
                )?;
                if !children.is_empty() {
                    return None;
                }
                *next += 1;
            }
            Some(())
        };
    let mut next_role = STRUCTURE_TERMINALS.len();
    let mut finish = |id: TypedNodeId| {
        if id.index() != next_role {
            return None;
        }
        next_role += 1;
        Some(())
    };
    let root_children = obs(root, K::Root, &mut seen)?;
    let (unit, aliases) = root_children.split_last()?;
    if !aliases
        .iter()
        .map(|id| id.index())
        .eq(ast.token_nodes().iter().map(|id| id.index()))
    {
        return None;
    }
    let unit_children = obs(*unit, K::CompilationUnit, &mut seen)?;
    let [list] = unit_children.as_ref() else {
        return None;
    };
    let list_children = obs(*list, K::ItemList, &mut seen)?;
    let [block] = list_children.as_ref() else {
        return None;
    };
    let is_shell = |id: TypedNodeId, kind| {
        shells
            .declarations()
            .iter()
            .any(|shell| shell.node_id().index() == id.index() && shell.kind() == kind)
    };
    if !is_shell(*block, DeclarationShellKind::DefinitionBlock) {
        return None;
    }
    let block_children = obs(*block, K::DefinitionBlockItem, &mut seen)?;
    let [open, base, derived, inheritance, end, semi] = block_children.as_ref() else {
        return None;
    };
    terms(&[*open], &mut next, &mut seen)?;
    for definition in [base, derived] {
        if !is_shell(*definition, DeclarationShellKind::StructureDefinition) {
            return None;
        }
        let children = obs(*definition, K::StructureDefinition, &mut seen)?;
        let [keyword, pattern, where_token, field, property, end, semi] = children.as_ref() else {
            return None;
        };
        terms(&[*keyword], &mut next, &mut seen)?;
        let name = obs(*pattern, K::StructurePattern, &mut seen)?;
        if name.len() != 1 {
            return None;
        }
        terms(&name, &mut next, &mut seen)?;
        finish(*pattern)?;
        terms(&[*where_token], &mut next, &mut seen)?;
        for (member, kind) in [(field, K::StructureField), (property, K::StructureProperty)] {
            let member_children = obs(*member, kind, &mut seen)?;
            let [keyword, name, arrow, expression, semi] = member_children.as_ref() else {
                return None;
            };
            terms(&[*keyword, *name, *arrow], &mut next, &mut seen)?;
            let expression_children = obs(*expression, K::TypeExpression, &mut seen)?;
            let [head] = expression_children.as_ref() else {
                return None;
            };
            let head_children = obs(*head, K::TypeHead, &mut seen)?;
            if head_children.len() != 1 {
                return None;
            }
            terms(&head_children, &mut next, &mut seen)?;
            finish(*head)?;
            finish(*expression)?;
            terms(&[*semi], &mut next, &mut seen)?;
            finish(*member)?;
        }
        terms(&[*end, *semi], &mut next, &mut seen)?;
        finish(*definition)?;
    }
    if !is_shell(*inheritance, DeclarationShellKind::InheritanceDefinition) {
        return None;
    }
    let inheritance_children = obs(*inheritance, K::InheritanceDefinition, &mut seen)?;
    let [
        keyword,
        child,
        extends,
        parent,
        where_token,
        field,
        property,
        close,
        terminator,
    ] = inheritance_children.as_ref()
    else {
        return None;
    };
    for (prefix, target) in [(keyword, child), (extends, parent)] {
        terms(&[*prefix], &mut next, &mut seen)?;
        let target_children = obs(*target, K::InheritanceTarget, &mut seen)?;
        if target_children.len() != 1 {
            return None;
        }
        terms(&target_children, &mut next, &mut seen)?;
        finish(*target)?;
    }
    terms(&[*where_token], &mut next, &mut seen)?;
    for (mapping, kind) in [
        (field, K::FieldRedefinition),
        (property, K::PropertyRedefinition),
    ] {
        let mapping_children = obs(*mapping, kind, &mut seen)?;
        if mapping_children.len() != 5 {
            return None;
        }
        terms(&mapping_children, &mut next, &mut seen)?;
        finish(*mapping)?;
    }
    terms(&[*close, *terminator], &mut next, &mut seen)?;
    finish(*inheritance)?;
    terms(&[*end, *semi], &mut next, &mut seen)?;
    finish(*block)?;
    finish(*list)?;
    finish(*unit)?;
    finish(root)?;
    (next == STRUCTURE_TERMINALS.len() && seen.len() == syntax.nodes().len()).then_some(())
}

fn validate_shells_and_projections(
    ast: &SurfaceAst,
    module: &ModuleId,
    shells: &DeclarationShellSet,
    mutation: SourceStructureDefinitionRouteMutation,
) -> Result<(), String> {
    let declarations = shells.declarations();
    let kinds = [
        DeclarationShellKind::DefinitionBlock,
        DeclarationShellKind::StructureDefinition,
        DeclarationShellKind::StructureField,
        DeclarationShellKind::StructureProperty,
        DeclarationShellKind::StructureDefinition,
        DeclarationShellKind::StructureField,
        DeclarationShellKind::StructureProperty,
        DeclarationShellKind::InheritanceDefinition,
        DeclarationShellKind::FieldRedefinition,
        DeclarationShellKind::PropertyRedefinition,
    ];
    let nodes = [71, 57, 53, 56, 65, 61, 64, 70, 68, 69];
    let parents = [
        None,
        Some(0),
        Some(1),
        Some(1),
        Some(0),
        Some(4),
        Some(4),
        Some(0),
        Some(7),
        Some(7),
    ];
    if declarations.len() != 10 || !shells.exports().is_empty() {
        return Err("Task263 resolver shell cardinality changed".to_owned());
    }
    for index in 0..10 {
        let row = &declarations[index];
        let mut ordinal = row.ordinal();
        if mutation == SourceStructureDefinitionRouteMutation::WrongShellRow(index) {
            ordinal = ordinal.saturating_add(1);
        }
        if row.id().index() != index
            || ordinal != index
            || row.kind() != kinds[index]
            || row.node_id().index() != nodes[index]
            || row.parent().map(|id| id.index()) != parents[index]
            || row.range() != ast.nodes()[nodes[index]].range
            || row.module() != module
            || row.recovered()
        {
            return Err(format!("Task263 resolver shell {index} changed"));
        }
    }
    let projections =
        SignatureProjectionExtractor::new(ast, shells, NamespacePath::new(module.path().as_str()))
            .extract();
    let expected = [
        (
            SymbolKind::Structure,
            DefinitionKind::Structure,
            "Task263Base",
        ),
        (SymbolKind::Selector, DefinitionKind::Selector, "carrier"),
        (SymbolKind::Selector, DefinitionKind::Selector, "marker"),
        (
            SymbolKind::Structure,
            DefinitionKind::Structure,
            "Task263Derived",
        ),
        (SymbolKind::Selector, DefinitionKind::Selector, "carrier"),
        (SymbolKind::Selector, DefinitionKind::Selector, "marker"),
        (
            SymbolKind::Redefinition,
            DefinitionKind::Redefinition,
            "carrier",
        ),
        (
            SymbolKind::Redefinition,
            DefinitionKind::Redefinition,
            "marker",
        ),
    ];
    if projections.len() != expected.len() {
        return Err("Task263 resolver projection cardinality changed".to_owned());
    }
    for (index, row) in projections.iter().enumerate() {
        let mut primary_spelling = row.primary_spelling().to_owned();
        if mutation == SourceStructureDefinitionRouteMutation::WrongProjectionRow(index) {
            primary_spelling.push('!');
        }
        if row.symbol_kind() != expected[index].0
            || row.definition_kind() != Some(expected[index].1)
            || primary_spelling != expected[index].2
            || row.overload_policy() != SymbolOverloadPolicy::NonOverloadable
            || row.arity() != (expected[index].1 == DefinitionKind::Structure).then_some(0)
            || row.signature().is_none()
        {
            return Err(format!("Task263 resolver projection {index} changed"));
        }
    }
    Ok(())
}
type ResolverIdentity = (SymbolId, DefinitionId, SourceContributionId);

fn resolver_identities(symbols: &SymbolEnv) -> Result<Vec<ResolverIdentity>, String> {
    if symbols.symbols().len() != 8
        || symbols.definitions().len() != 8
        || symbols.contributions().len() != 1
    {
        return Err("Task263 resolver env cardinality changed".to_owned());
    }
    (0..8)
        .map(|index| {
            let row = symbols
                .definitions()
                .iter()
                .find(|row| row.id().index() == index)
                .ok_or_else(|| format!("Task263 resolver definition {index} disappeared"))?;
            Ok((row.symbol().clone(), row.id(), row.contribution()))
        })
        .collect()
}

fn source_type_input(
    syntax: &TypedAst,
    module: ModuleId,
) -> Result<SourceTypeStructureMemberHandoffInput, String> {
    let source = syntax.source_id();
    let row = |index| {
        syntax
            .source_node_entries()
            .and_then(|mut rows| rows.find(|row| row.site().index() == index))
            .ok_or_else(|| TypedAstError::InvalidSourceStatement.to_string())
    };
    let specs = [(53, 52, 51), (56, 55, 54), (61, 60, 59), (64, 63, 62)];
    Ok(SourceTypeStructureMemberHandoffInput {
        source_id: source,
        module_id: module.clone(),
        members: specs
            .into_iter()
            .enumerate()
            .map(
                |(source_ordinal, (member_node, expression_node, head_node))| {
                    let member = row(member_node)?;
                    let expression = row(expression_node)?;
                    let head = row(head_node)?;
                    Ok(SourceTypeStructureMemberInput {
                        member_site: node_site(member.site().index()),
                        member_range: member.range(),
                        source_ordinal,
                        expression: SourceTypeExpressionInput {
                            source_id: source,
                            module_id: module.clone(),
                            site: node_site(expression.site().index()),
                            source_range: expression.range(),
                            spelling: "set".to_owned(),
                            head_site: node_site(head.site().index()),
                            head_range: head.range(),
                            head_spelling: "set".to_owned(),
                            form: SourceTypeApplicationForm::Bare,
                            head: SourceTypeHead::BuiltinSet,
                            recovery: NodeRecoveryState::Normal,
                        },
                    })
                },
            )
            .collect::<Result<_, String>>()?,
    })
}

fn checker_input(
    syntax: &TypedAst,
    module: ModuleId,
    identities: &[ResolverIdentity],
) -> Result<SourceStructureDefinitionHandoffInput, String> {
    let source = syntax.source_id();
    let row = |index| {
        syntax
            .source_node_entries()
            .and_then(|mut rows| rows.find(|row| row.site().index() == index))
            .ok_or_else(|| TypedAstError::InvalidSourceStatement.to_string())
    };
    let base = row(57)?;
    let derived = row(65)?;
    let inheritance = row(70)?;
    let definitions = vec![
        SourceStructureDefinitionInput { symbol: identities[0].0.clone(), definition: identities[0].1, contribution: identities[0].2, site: node_site(base.site().index()), source_range: base.range(), source_ordinal: 0, recovery: SourceStructureDefinitionRecovery::Normal, spelling: "struct Task263Base where\n    field carrier -> set;\n    property marker -> set;\n  end;".to_owned(), members: vec![SourceStructureMemberId::new(0),SourceStructureMemberId::new(1)], constructor_fields: vec![SourceStructureMemberId::new(0)] },
        SourceStructureDefinitionInput { symbol: identities[3].0.clone(), definition: identities[3].1, contribution: identities[3].2, site: node_site(derived.site().index()), source_range: derived.range(), source_ordinal: 1, recovery: SourceStructureDefinitionRecovery::Normal, spelling: "struct Task263Derived where\n    field carrier -> set;\n    property marker -> set;\n  end;".to_owned(), members: vec![SourceStructureMemberId::new(2),SourceStructureMemberId::new(3)], constructor_fields: vec![SourceStructureMemberId::new(2)] },
    ];
    let member_specs = [
        (
            1,
            0,
            0,
            SourceStructureMemberKind::Field,
            53,
            "field carrier -> set;",
            Some(0),
        ),
        (
            2,
            0,
            1,
            SourceStructureMemberKind::Property,
            56,
            "property marker -> set;",
            None,
        ),
        (
            4,
            1,
            0,
            SourceStructureMemberKind::Field,
            61,
            "field carrier -> set;",
            Some(0),
        ),
        (
            5,
            1,
            1,
            SourceStructureMemberKind::Property,
            64,
            "property marker -> set;",
            None,
        ),
    ];
    let members = member_specs
        .into_iter()
        .enumerate()
        .map(
            |(index, (identity, owner, ordinal, kind, node, spelling, constructor_ordinal))| {
                let member = row(node)?;
                Ok(SourceStructureMemberInput {
                    symbol: identities[identity].0.clone(),
                    definition: identities[identity].1,
                    contribution: identities[identity].2,
                    owner: SourceStructureDefinitionId::new(owner),
                    ordinal,
                    kind,
                    site: node_site(member.site().index()),
                    source_range: member.range(),
                    recovery: SourceStructureDefinitionRecovery::Normal,
                    spelling: spelling.to_owned(),
                    written_type: SourceTypeStructureMemberId::new(index),
                    constructor_ordinal,
                })
            },
        )
        .collect::<Result<_, String>>()?;
    let inheritances = vec![SourceStructureInheritanceInput { child: SourceStructureDefinitionId::new(1), parent: SourceStructureDefinitionId::new(0), site: node_site(inheritance.site().index()), source_range: inheritance.range(), source_ordinal: 0, recovery: SourceStructureDefinitionRecovery::Normal, spelling: "inherit Task263Derived extends Task263Base where\n    field carrier from carrier;\n    property marker from marker;\n  end;".to_owned(), mappings: vec![SourceStructureMappingId::new(0),SourceStructureMappingId::new(1)] }];
    let mapping_specs = [
        (
            6,
            SourceStructureMemberKind::Field,
            2,
            0,
            68,
            "field carrier from carrier;",
        ),
        (
            7,
            SourceStructureMemberKind::Property,
            3,
            1,
            69,
            "property marker from marker;",
        ),
    ];
    let mappings = mapping_specs
        .into_iter()
        .enumerate()
        .map(|(index, (identity, kind, view, parent, node, spelling))| {
            let mapping = row(node)?;
            Ok(SourceStructureMappingInput {
                symbol: identities[identity].0.clone(),
                definition: identities[identity].1,
                contribution: identities[identity].2,
                inheritance: SourceStructureInheritanceId::new(0),
                ordinal: index,
                kind,
                view_member: SourceStructureMemberId::new(view),
                parent_member: SourceStructureMemberId::new(parent),
                root_member: SourceStructureMemberId::new(parent),
                path: vec![SourceStructureInheritanceId::new(0)],
                site: node_site(mapping.site().index()),
                source_range: mapping.range(),
                recovery: SourceStructureDefinitionRecovery::Normal,
                spelling: spelling.to_owned(),
            })
        })
        .collect::<Result<_, String>>()?;
    Ok(SourceStructureDefinitionHandoffInput {
        source_id: source,
        module_id: module,
        definitions,
        members,
        inheritances,
        mappings,
    })
}

fn task263_arena(
    syntax: &TypedAst,
    mutation: SourceStructureDefinitionRouteMutation,
) -> Result<TypedArena, String> {
    let mut builder = TypedArenaBuilder::new();
    for (id, node) in syntax.nodes().iter() {
        let index = id.index();
        let row = syntax
            .source_node_entries()
            .and_then(|mut rows| rows.find(|row| row.site().index() == index))
            .ok_or_else(|| TypedAstError::InvalidSourceStatement.to_string())?;
        let generic = match row.kind() {
            SurfaceNodeKind::Token(_) => "source.surface.token",
            _ => "source.surface.structural",
        };
        let kind = match index {
            51 | 54 | 59 | 62 => "source.type.head",
            52 | 55 | 60 | 63 => "source.type.expression",
            53 | 56 | 61 | 64 => "source.definition.structure.member",
            57 | 65 => "source.definition.structure",
            68 | 69 => "source.definition.structure.mapping",
            70 => "source.definition.structure.inheritance",
            74 => "source.module",
            _ => generic,
        };
        let kind = if mutation == SourceStructureDefinitionRouteMutation::WrongArena && index == 68
        {
            "source.definition.structure.member"
        } else {
            kind
        };
        let children = node
            .children
            .iter()
            .map(|id| TypedNodeId::new(id.index()))
            .collect();
        let actual = builder
            .push(
                TypedNode::new(kind, SourceAnchor::Range(row.range()))
                    .with_children(children)
                    .with_typing(TypingState::Unknown)
                    .with_recovery(NodeRecoveryState::Normal),
            )
            .map_err(|error| error.to_string())?;
        if actual.index() != index {
            return Err("Task263 typed arena order changed".to_owned());
        }
    }
    builder
        .finish(Some(TypedNodeId::new(74)))
        .map_err(|error| error.to_string())
}

fn route_output_is_exact(output: &SourceStructureDefinitionRouteOutput) -> bool {
    let typed = &output.typed_ast;
    let resolved = &output.resolved;
    let Some(source_type) = typed.source_type() else {
        return false;
    };
    let Some(structure) = typed.source_structure_definition() else {
        return false;
    };
    source_type.applications().is_empty()
        && source_type.expressions().len() == 4
        && source_type.arguments().is_empty()
        && source_type.definition_returns().is_empty()
        && source_type.mode_rhs().is_empty()
        && source_type.structure_members().len() == 4
        && structure.definitions().len() == 2
        && structure.members().len() == 4
        && structure.inheritances().len() == 1
        && structure.mappings().len() == 2
        && structure.coherence_requests().is_empty()
        && typed.source_structure_definition() == resolved.source_structure_definition()
        && typed.source_type() == resolved.source_type()
        && typed.initial_obligations().is_empty()
        && typed.source_context().is_none()
        && typed.source_predicate_definition().is_none()
        && typed.source_functor_definition().is_none()
        && typed.source_attribute_definition().is_none()
        && typed.source_mode_definition().is_none()
        && typed.types().is_empty()
        && typed.facts().is_empty()
        && typed.coercions().is_empty()
        && typed.diagnostics().is_empty()
        && resolved.expr_metadata().is_empty()
        && resolved.cluster_facts().is_empty()
        && resolved.diagnostics().is_empty()
}

const fn node_site(index: usize) -> TypedSiteRef {
    TypedSiteRef::Node(TypedNodeId::new(index))
}
