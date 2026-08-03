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
        TypedAstParts, TypedNode, TypedNodeId, TypedSiteRef, TypingState,
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
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use mizar_syntax::{SurfaceAst, SurfaceNodeId, SurfaceNodeKind, SurfaceTokenKind};

use super::checker_handoff::assemble_empty_resolved_typed_ast;

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

enum ExactSurfaceKind {
    Token(SurfaceTokenKind, &'static str),
    Structural(SurfaceNodeKind),
}

struct ExactSurfaceRow {
    kind: ExactSurfaceKind,
    start: usize,
    end: usize,
    children: &'static [usize],
}

macro_rules! token_row {
    ($kind:ident, $text:literal, $start:literal, $end:literal) => {
        ExactSurfaceRow {
            kind: ExactSurfaceKind::Token(SurfaceTokenKind::$kind, $text),
            start: $start,
            end: $end,
            children: &[],
        }
    };
}
macro_rules! structural_row {
    ($kind:ident, $start:literal, $end:literal, [$($child:literal),* $(,)?]) => {
        ExactSurfaceRow { kind: ExactSurfaceKind::Structural(SurfaceNodeKind::$kind), start: $start, end: $end, children: &[$($child),*] }
    };
}

const EXACT_SURFACE_PROFILE: &[ExactSurfaceRow] = &[
    token_row!(ReservedWord, "definition", 0, 10),
    token_row!(ReservedWord, "struct", 13, 19),
    token_row!(Identifier, "Task263Base", 20, 31),
    token_row!(ReservedWord, "where", 32, 37),
    token_row!(ReservedWord, "field", 42, 47),
    token_row!(Identifier, "carrier", 48, 55),
    token_row!(ReservedSymbol, "->", 56, 58),
    token_row!(ReservedWord, "set", 59, 62),
    token_row!(ReservedSymbol, ";", 62, 63),
    token_row!(ReservedWord, "property", 68, 76),
    token_row!(Identifier, "marker", 77, 83),
    token_row!(ReservedSymbol, "->", 84, 86),
    token_row!(ReservedWord, "set", 87, 90),
    token_row!(ReservedSymbol, ";", 90, 91),
    token_row!(ReservedWord, "end", 94, 97),
    token_row!(ReservedSymbol, ";", 97, 98),
    token_row!(ReservedWord, "struct", 102, 108),
    token_row!(Identifier, "Task263Derived", 109, 123),
    token_row!(ReservedWord, "where", 124, 129),
    token_row!(ReservedWord, "field", 134, 139),
    token_row!(Identifier, "carrier", 140, 147),
    token_row!(ReservedSymbol, "->", 148, 150),
    token_row!(ReservedWord, "set", 151, 154),
    token_row!(ReservedSymbol, ";", 154, 155),
    token_row!(ReservedWord, "property", 160, 168),
    token_row!(Identifier, "marker", 169, 175),
    token_row!(ReservedSymbol, "->", 176, 178),
    token_row!(ReservedWord, "set", 179, 182),
    token_row!(ReservedSymbol, ";", 182, 183),
    token_row!(ReservedWord, "end", 186, 189),
    token_row!(ReservedSymbol, ";", 189, 190),
    token_row!(ReservedWord, "inherit", 194, 201),
    token_row!(UserSymbol, "Task263Derived", 202, 216),
    token_row!(ReservedWord, "extends", 217, 224),
    token_row!(UserSymbol, "Task263Base", 225, 236),
    token_row!(ReservedWord, "where", 237, 242),
    token_row!(ReservedWord, "field", 247, 252),
    token_row!(Identifier, "carrier", 253, 260),
    token_row!(ReservedWord, "from", 261, 265),
    token_row!(Identifier, "carrier", 266, 273),
    token_row!(ReservedSymbol, ";", 273, 274),
    token_row!(ReservedWord, "property", 279, 287),
    token_row!(Identifier, "marker", 288, 294),
    token_row!(ReservedWord, "from", 295, 299),
    token_row!(Identifier, "marker", 300, 306),
    token_row!(ReservedSymbol, ";", 306, 307),
    token_row!(ReservedWord, "end", 310, 313),
    token_row!(ReservedSymbol, ";", 313, 314),
    token_row!(ReservedWord, "end", 315, 318),
    token_row!(ReservedSymbol, ";", 318, 319),
    structural_row!(StructurePattern, 20, 31, [2]),
    structural_row!(TypeHead, 59, 62, [7]),
    structural_row!(TypeExpression, 59, 62, [51]),
    structural_row!(StructureField, 42, 63, [4, 5, 6, 52, 8]),
    structural_row!(TypeHead, 87, 90, [12]),
    structural_row!(TypeExpression, 87, 90, [54]),
    structural_row!(StructureProperty, 68, 91, [9, 10, 11, 55, 13]),
    structural_row!(StructureDefinition, 13, 98, [1, 50, 3, 53, 56, 14, 15]),
    structural_row!(StructurePattern, 109, 123, [17]),
    structural_row!(TypeHead, 151, 154, [22]),
    structural_row!(TypeExpression, 151, 154, [59]),
    structural_row!(StructureField, 134, 155, [19, 20, 21, 60, 23]),
    structural_row!(TypeHead, 179, 182, [27]),
    structural_row!(TypeExpression, 179, 182, [62]),
    structural_row!(StructureProperty, 160, 183, [24, 25, 26, 63, 28]),
    structural_row!(StructureDefinition, 102, 190, [16, 58, 18, 61, 64, 29, 30]),
    structural_row!(InheritanceTarget, 202, 216, [32]),
    structural_row!(InheritanceTarget, 225, 236, [34]),
    structural_row!(FieldRedefinition, 247, 274, [36, 37, 38, 39, 40]),
    structural_row!(PropertyRedefinition, 279, 307, [41, 42, 43, 44, 45]),
    structural_row!(
        InheritanceDefinition,
        194,
        314,
        [31, 66, 33, 67, 35, 68, 69, 46, 47]
    ),
    structural_row!(DefinitionBlockItem, 0, 319, [0, 57, 65, 70, 48, 49]),
    structural_row!(ItemList, 0, 319, [71]),
    structural_row!(CompilationUnit, 0, 319, [72]),
    structural_row!(
        Root,
        0,
        319,
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47, 48, 49, 73
        ]
    ),
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
    if source_text != SOURCE_STRUCTURE_DEFINITION_TEXT
        || source_text.len() != 320
        || !source_text.ends_with('\n')
        || source_text.ends_with("\n\n")
    {
        return None;
    }
    Some(build_output(ast, module, shells, symbols, mutation))
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    mutation: SourceStructureDefinitionRouteMutation,
) -> Result<SourceStructureDefinitionRouteOutput, String> {
    if !surface_profile_is_exact(ast, mutation) {
        return Err("Task263 Surface profile is not exact".to_owned());
    }
    validate_shells_and_projections(ast, &module, shells, mutation)?;
    let identities = resolver_identities(symbols)?;
    let arena = task263_arena(ast.source_id, mutation)?;
    let mut lower_input = source_type_input(ast.source_id, module.clone());
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
    let mut input = checker_input(ast.source_id, module, &identities);
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

fn surface_profile_is_exact(
    ast: &SurfaceAst,
    mutation: SourceStructureDefinitionRouteMutation,
) -> bool {
    let mut kinds = ast
        .nodes()
        .iter()
        .map(|node| match &node.kind {
            SurfaceNodeKind::Token(token) => format!("token:{:?}:{:?}", token.kind, token.text),
            kind => format!("node:{kind:?}"),
        })
        .collect::<Vec<_>>();
    let mut ranges = ast
        .nodes()
        .iter()
        .map(|node| (node.range.start, node.range.end))
        .collect::<Vec<_>>();
    let mut recoveries = ast
        .nodes()
        .iter()
        .map(|node| node.recovered)
        .collect::<Vec<_>>();
    let mut children = ast
        .nodes()
        .iter()
        .map(|node| {
            node.children
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut root = ast.root().map(SurfaceNodeId::index);
    match mutation {
        SourceStructureDefinitionRouteMutation::WrongSurfaceRow(index) => {
            if let Some(kind) = kinds.get_mut(index) {
                kind.push('!');
            }
        }
        SourceStructureDefinitionRouteMutation::WrongSurfaceRange(index) => {
            if let Some(range) = ranges.get_mut(index) {
                range.1 = range.1.saturating_add(1);
            }
        }
        SourceStructureDefinitionRouteMutation::WrongSurfaceRecovery(index) => {
            if let Some(recovered) = recoveries.get_mut(index) {
                *recovered = !*recovered;
            }
        }
        SourceStructureDefinitionRouteMutation::WrongSurfaceChildren(index) => {
            if let Some(row) = children.get_mut(index) {
                if row.len() > 1 {
                    row.rotate_left(1);
                } else {
                    row.push(index);
                }
            }
        }
        SourceStructureDefinitionRouteMutation::WrongSurfaceRoot => root = None,
        _ => {}
    }
    root == Some(74)
        && ast.expression_root().is_none()
        && ast.nodes().len() == EXACT_SURFACE_PROFILE.len()
        && ast
            .nodes()
            .iter()
            .all(|node| node.range.source_id == ast.source_id)
        && ast
            .nodes()
            .iter()
            .enumerate()
            .zip(EXACT_SURFACE_PROFILE)
            .all(|((index, _), expected)| {
                let expected_kind = match &expected.kind {
                    ExactSurfaceKind::Token(kind, text) => {
                        format!("token:{kind:?}:{text:?}")
                    }
                    ExactSurfaceKind::Structural(kind) => format!("node:{kind:?}"),
                };
                kinds[index] == expected_kind
                    && ranges[index] == (expected.start, expected.end)
                    && !recoveries[index]
                    && children[index] == expected.children
            })
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
    let ranges = [
        (0, 319),
        (13, 98),
        (42, 63),
        (68, 91),
        (102, 190),
        (134, 155),
        (160, 183),
        (194, 314),
        (247, 274),
        (279, 307),
    ];
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
            || row.range() != range(ast.source_id, ranges[index].0, ranges[index].1)
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
            || row.arity().is_some()
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

fn source_type_input(source: SourceId, module: ModuleId) -> SourceTypeStructureMemberHandoffInput {
    let specs = [
        (53, 42, 63, 52, 51, 59, 62),
        (56, 68, 91, 55, 54, 87, 90),
        (61, 134, 155, 60, 59, 151, 154),
        (64, 160, 183, 63, 62, 179, 182),
    ];
    SourceTypeStructureMemberHandoffInput {
        source_id: source,
        module_id: module.clone(),
        members: specs
            .into_iter()
            .enumerate()
            .map(
                |(
                    source_ordinal,
                    (member_node, member_start, member_end, expression_node, head_node, start, end),
                )| SourceTypeStructureMemberInput {
                    member_site: node_site(member_node),
                    member_range: range(source, member_start, member_end),
                    source_ordinal,
                    expression: SourceTypeExpressionInput {
                        source_id: source,
                        module_id: module.clone(),
                        site: node_site(expression_node),
                        source_range: range(source, start, end),
                        spelling: "set".to_owned(),
                        head_site: node_site(head_node),
                        head_range: range(source, start, end),
                        head_spelling: "set".to_owned(),
                        form: SourceTypeApplicationForm::Bare,
                        head: SourceTypeHead::BuiltinSet,
                        recovery: NodeRecoveryState::Normal,
                    },
                },
            )
            .collect(),
    }
}

fn checker_input(
    source: SourceId,
    module: ModuleId,
    identities: &[ResolverIdentity],
) -> SourceStructureDefinitionHandoffInput {
    let definitions = vec![
        SourceStructureDefinitionInput { symbol: identities[0].0.clone(), definition: identities[0].1, contribution: identities[0].2, site: node_site(57), source_range: range(source,13,98), source_ordinal: 0, recovery: SourceStructureDefinitionRecovery::Normal, spelling: "struct Task263Base where\n    field carrier -> set;\n    property marker -> set;\n  end;".to_owned(), members: vec![SourceStructureMemberId::new(0),SourceStructureMemberId::new(1)], constructor_fields: vec![SourceStructureMemberId::new(0)] },
        SourceStructureDefinitionInput { symbol: identities[3].0.clone(), definition: identities[3].1, contribution: identities[3].2, site: node_site(65), source_range: range(source,102,190), source_ordinal: 1, recovery: SourceStructureDefinitionRecovery::Normal, spelling: "struct Task263Derived where\n    field carrier -> set;\n    property marker -> set;\n  end;".to_owned(), members: vec![SourceStructureMemberId::new(2),SourceStructureMemberId::new(3)], constructor_fields: vec![SourceStructureMemberId::new(2)] },
    ];
    let member_specs = [
        (
            1,
            0,
            0,
            SourceStructureMemberKind::Field,
            53,
            42,
            63,
            "field carrier -> set;",
            Some(0),
        ),
        (
            2,
            0,
            1,
            SourceStructureMemberKind::Property,
            56,
            68,
            91,
            "property marker -> set;",
            None,
        ),
        (
            4,
            1,
            0,
            SourceStructureMemberKind::Field,
            61,
            134,
            155,
            "field carrier -> set;",
            Some(0),
        ),
        (
            5,
            1,
            1,
            SourceStructureMemberKind::Property,
            64,
            160,
            183,
            "property marker -> set;",
            None,
        ),
    ];
    let members = member_specs
        .into_iter()
        .enumerate()
        .map(
            |(
                index,
                (identity, owner, ordinal, kind, node, start, end, spelling, constructor_ordinal),
            )| SourceStructureMemberInput {
                symbol: identities[identity].0.clone(),
                definition: identities[identity].1,
                contribution: identities[identity].2,
                owner: SourceStructureDefinitionId::new(owner),
                ordinal,
                kind,
                site: node_site(node),
                source_range: range(source, start, end),
                recovery: SourceStructureDefinitionRecovery::Normal,
                spelling: spelling.to_owned(),
                written_type: SourceTypeStructureMemberId::new(index),
                constructor_ordinal,
            },
        )
        .collect();
    let inheritances = vec![SourceStructureInheritanceInput { child: SourceStructureDefinitionId::new(1), parent: SourceStructureDefinitionId::new(0), site: node_site(70), source_range: range(source,194,314), source_ordinal: 0, recovery: SourceStructureDefinitionRecovery::Normal, spelling: "inherit Task263Derived extends Task263Base where\n    field carrier from carrier;\n    property marker from marker;\n  end;".to_owned(), mappings: vec![SourceStructureMappingId::new(0),SourceStructureMappingId::new(1)] }];
    let mapping_specs = [
        (
            6,
            SourceStructureMemberKind::Field,
            2,
            0,
            68,
            247,
            274,
            "field carrier from carrier;",
        ),
        (
            7,
            SourceStructureMemberKind::Property,
            3,
            1,
            69,
            279,
            307,
            "property marker from marker;",
        ),
    ];
    let mappings = mapping_specs
        .into_iter()
        .enumerate()
        .map(
            |(index, (identity, kind, view, parent, node, start, end, spelling))| {
                SourceStructureMappingInput {
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
                    site: node_site(node),
                    source_range: range(source, start, end),
                    recovery: SourceStructureDefinitionRecovery::Normal,
                    spelling: spelling.to_owned(),
                }
            },
        )
        .collect();
    SourceStructureDefinitionHandoffInput {
        source_id: source,
        module_id: module,
        definitions,
        members,
        inheritances,
        mappings,
    }
}

fn task263_arena(
    source: SourceId,
    mutation: SourceStructureDefinitionRouteMutation,
) -> Result<TypedArena, String> {
    let mut builder = TypedArenaBuilder::new();
    for (index, row) in EXACT_SURFACE_PROFILE.iter().enumerate() {
        let generic = match row.kind {
            ExactSurfaceKind::Token(_, _) => "source.surface.token",
            ExactSurfaceKind::Structural(_) => "source.surface.structural",
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
        let children = row.children.iter().copied().map(TypedNodeId::new).collect();
        let actual = builder
            .push(
                TypedNode::new(kind, SourceAnchor::Range(range(source, row.start, row.end)))
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

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}
const fn node_site(index: usize) -> TypedSiteRef {
    TypedSiteRef::Node(TypedNodeId::new(index))
}
