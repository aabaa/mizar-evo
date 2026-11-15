use super::*;
use crate::{
    cluster_trace::ClusterFactTable,
    overload_resolution::{
        CandidateViabilityInput, CandidateViabilityOutput, OverloadCandidateInput,
        OverloadCollectionOutput, OverloadSelectionOutput, OverloadSiteInput,
        OverloadSiteResolutionInput, SpecificityComparisonInput, SpecificityGraphOutput,
        TemplateExpansionOutput,
    },
    resolved_typed_ast::{ResolvedTypedAst, ResolvedTypedAstError, ResolvedTypedAstInputs},
    source_type::{
        SourceTypeExpressionInput, SourceTypeStructureMemberHandoffInput,
        SourceTypeStructureMemberInput, SourceTypeStructureMemberProducer,
    },
    typed_ast::{
        CoercionTable, InitialObligationDraft, InitialObligationGoal, InitialObligationProvenance,
        InitialObligationStatus, LocalTypeContextTable, TypeDiagnosticTable, TypeFactTable,
        TypeTable, TypedAst, TypedAstError, TypedAstParts, TypedNode, TypingState,
    },
};
use mizar_resolve::{
    env::{DefinitionShell, NamespacePath, SignatureShell, SymbolEntry, SymbolEnvIndexes},
    resolved_ast::{FullyQualifiedName, LocalSymbolId},
};
use mizar_session::{
    BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
};
use mizar_syntax::SurfaceNodeKind;

#[derive(Clone)]
struct Fixture {
    source: SourceId,
    module: ModuleId,
    env: SymbolEnv,
    source_type: SourceTypeApplicationHandoff,
    arena: TypedArena,
    input: SourceStructureDefinitionHandoffInput,
}

impl Fixture {
    fn build_native(
        &self,
        baseline: &InitialObligationTable,
        source: &TypedAst,
    ) -> Result<SourceStructureDefinitionProjection, SourceStructureDefinitionError> {
        SourceStructureDefinitionProducer::build(
            self.input.clone(),
            &self.env,
            &self.source_type,
            baseline,
            &self.arena,
            source,
        )
    }

    fn typed(&self, baseline: InitialObligationTable) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: self.source,
            module_id: self.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: Some(self.source_type.clone()),
            source_attribute: None,
            nodes: self.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: baseline,
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("Task 263 typed baseline")
    }
}

fn assert_installation_error(
    fixture: &Fixture,
    handoff: &SourceStructureDefinitionHandoff,
    source_type: &SourceTypeApplicationHandoff,
    obligations: &InitialObligationTable,
    arena: &TypedArena,
    expected: SourceStructureDefinitionError,
) {
    assert_eq!(
        handoff.validate_installation(
            fixture.source,
            &fixture.module,
            source_type,
            obligations,
            arena,
        ),
        Err(expected),
    );
}

fn fixture() -> Fixture {
    let source = source_id();
    let module = ModuleId::new(PackageId::new("pkg"), ModulePath::new("task263"));
    let arena = task263_arena(source);
    let source_type =
        SourceTypeStructureMemberProducer::build(source_type_input(source, module.clone()), &arena)
            .expect("Task249S exact lower handoff");
    let (env, identities) = resolver_env(source, module.clone());
    let definitions = vec![
        SourceStructureDefinitionInput { symbol: identities[0].0.clone(), definition: identities[0].1, contribution: identities[0].2, site: node_site(57), source_range: range(source, 13, 98), source_ordinal: 0, recovery: SourceStructureDefinitionRecovery::Normal, spelling: "struct Task263Base where\n    field carrier -> set;\n    property marker -> set;\n  end;".to_owned(), members: vec![SourceStructureMemberId::new(0), SourceStructureMemberId::new(1)], constructor_fields: vec![SourceStructureMemberId::new(0)] },
        SourceStructureDefinitionInput { symbol: identities[3].0.clone(), definition: identities[3].1, contribution: identities[3].2, site: node_site(65), source_range: range(source, 102, 190), source_ordinal: 1, recovery: SourceStructureDefinitionRecovery::Normal, spelling: "struct Task263Derived where\n    field carrier -> set;\n    property marker -> set;\n  end;".to_owned(), members: vec![SourceStructureMemberId::new(2), SourceStructureMemberId::new(3)], constructor_fields: vec![SourceStructureMemberId::new(2)] },
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
    let inheritances = vec![SourceStructureInheritanceInput { child: SourceStructureDefinitionId::new(1), parent: SourceStructureDefinitionId::new(0), site: node_site(70), source_range: range(source, 194, 314), source_ordinal: 0, recovery: SourceStructureDefinitionRecovery::Normal, spelling: "inherit Task263Derived extends Task263Base where\n    field carrier from carrier;\n    property marker from marker;\n  end;".to_owned(), mappings: vec![SourceStructureMappingId::new(0), SourceStructureMappingId::new(1)] }];
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
    Fixture {
        source,
        module: module.clone(),
        env,
        source_type,
        arena,
        input: SourceStructureDefinitionHandoffInput {
            source_id: source,
            module_id: module,
            definitions,
            members,
            inheritances,
            mappings,
        },
    }
}

type ResolverIdentity = (SymbolId, DefinitionId, SourceContributionId);

fn resolver_env(source: SourceId, module: ModuleId) -> (SymbolEnv, Vec<ResolverIdentity>) {
    resolver_env_with_effects(source, module, false, None)
}

fn resolver_env_with_effects(
    source: SourceId,
    module: ModuleId,
    stale_effects: bool,
    arity_override: Option<(usize, Option<u32>)>,
) -> (SymbolEnv, Vec<ResolverIdentity>) {
    let mut indexes = SymbolEnvIndexes::default();
    let contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 0, 319)),
    );
    let specs = [
        (
            SymbolKind::Structure,
            DefinitionKind::Structure,
            "Task263Base",
            Some("Task263Base"),
            13,
            98,
            &[4, 0, 11, 0][..],
        ),
        (
            SymbolKind::Selector,
            DefinitionKind::Selector,
            "carrier",
            None,
            42,
            63,
            &[4, 0, 11, 0, 18, 0][..],
        ),
        (
            SymbolKind::Selector,
            DefinitionKind::Selector,
            "marker",
            None,
            68,
            91,
            &[4, 0, 11, 0, 19, 1][..],
        ),
        (
            SymbolKind::Structure,
            DefinitionKind::Structure,
            "Task263Derived",
            Some("Task263Derived"),
            102,
            190,
            &[4, 0, 11, 1][..],
        ),
        (
            SymbolKind::Selector,
            DefinitionKind::Selector,
            "carrier",
            None,
            134,
            155,
            &[4, 0, 11, 1, 18, 0][..],
        ),
        (
            SymbolKind::Selector,
            DefinitionKind::Selector,
            "marker",
            None,
            160,
            183,
            &[4, 0, 11, 1, 19, 1][..],
        ),
        (
            SymbolKind::Redefinition,
            DefinitionKind::Redefinition,
            "carrier",
            Some("field carrier from carrier ;"),
            247,
            274,
            &[4, 0, 20, 2, 21, 0][..],
        ),
        (
            SymbolKind::Redefinition,
            DefinitionKind::Redefinition,
            "marker",
            Some("property marker from marker ;"),
            279,
            307,
            &[4, 0, 20, 2, 22, 1][..],
        ),
    ];
    let mut identities = Vec::new();
    for (index, (symbol_kind, definition_kind, spelling, notation, start, end, path)) in
        specs.into_iter().enumerate()
    {
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, start, end)),
            path.to_vec(),
        );
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new(format!("task263-{index}")),
            FullyQualifiedName::new(format!("pkg::task263::{index}:{spelling}")),
        );
        let signature = SignatureShell::Opaque {
            schema: "parser-signature-v1".to_owned(),
            payload: format!("task263:{index}"),
        };
        let mut entry = SymbolEntry::new(
            symbol.clone(),
            symbol_kind,
            NamespacePath::new(module.path().as_str()),
            spelling,
            origin.clone(),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported)
        .with_signature(signature.clone());
        if let Some(notation) = notation {
            entry = entry.with_notation_spelling(notation);
        }
        indexes.symbols.insert(entry);
        let mut shell = DefinitionShell::new(symbol.clone(), definition_kind, origin, contribution)
            .with_visibility(Visibility::Public)
            .with_signature(signature);
        let arity = match arity_override.filter(|(target, _)| *target == index) {
            Some((_, arity)) => arity,
            None if definition_kind == DefinitionKind::Structure => Some(0),
            None => None,
        };
        if let Some(arity) = arity {
            shell = shell.with_arity(arity);
        }
        if let Some(notation) = notation {
            shell = shell.with_notation_shape(notation);
        }
        let definition = indexes.definitions.insert(shell);
        if !stale_effects {
            indexes
                .contributions
                .add_symbol(contribution, symbol.clone());
        }
        indexes
            .contributions
            .add_definition(contribution, definition);
        identities.push((symbol, definition, contribution));
    }
    if stale_effects {
        for index in 0..8 {
            indexes.contributions.add_symbol(
                contribution,
                SymbolId::new(
                    module.clone(),
                    LocalSymbolId::new(format!("task263-stale-{index}")),
                    FullyQualifiedName::new(format!("pkg::task263::stale:{index}")),
                ),
            );
        }
    }
    (SymbolEnv::new(module, indexes), identities)
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

fn task263_arena(source: SourceId) -> TypedArena {
    let nodes = (0..75)
        .map(|index| {
            let (kind, source_range) = match index {
                51 | 52 => (
                    if index == 51 {
                        "source.type.head"
                    } else {
                        "source.type.expression"
                    },
                    range(source, 59, 62),
                ),
                53 => ("source.definition.structure.member", range(source, 42, 63)),
                54 | 55 => (
                    if index == 54 {
                        "source.type.head"
                    } else {
                        "source.type.expression"
                    },
                    range(source, 87, 90),
                ),
                56 => ("source.definition.structure.member", range(source, 68, 91)),
                57 => ("source.definition.structure", range(source, 13, 98)),
                59 | 60 => (
                    if index == 59 {
                        "source.type.head"
                    } else {
                        "source.type.expression"
                    },
                    range(source, 151, 154),
                ),
                61 => (
                    "source.definition.structure.member",
                    range(source, 134, 155),
                ),
                62 | 63 => (
                    if index == 62 {
                        "source.type.head"
                    } else {
                        "source.type.expression"
                    },
                    range(source, 179, 182),
                ),
                64 => (
                    "source.definition.structure.member",
                    range(source, 160, 183),
                ),
                65 => ("source.definition.structure", range(source, 102, 190)),
                68 | 69 => (
                    "source.definition.structure.mapping",
                    if index == 68 {
                        range(source, 247, 274)
                    } else {
                        range(source, 279, 307)
                    },
                ),
                70 => (
                    "source.definition.structure.inheritance",
                    range(source, 194, 314),
                ),
                74 => ("source.module", range(source, 0, 319)),
                _ => ("source.unowned", range(source, 0, 1)),
            };
            TypedNode::new(kind, SourceAnchor::Range(source_range))
                .with_typing(TypingState::Unknown)
                .with_recovery(NodeRecoveryState::Normal)
        })
        .collect();
    TypedArena::try_new(Some(TypedNodeId::new(74)), nodes).unwrap()
}

fn unrelated_baseline(source: SourceId, key: &str) -> InitialObligationTable {
    let mut table = InitialObligationTable::new();
    table.insert(obligation(source, key));
    table
}

fn obligation(source: SourceId, key: &str) -> InitialObligationDraft {
    let domain = if key.starts_with("source.") {
        key.to_owned()
    } else {
        format!("unrelated.{key}")
    };
    InitialObligationDraft {
        kind: InitialObligationKind::RegistrationCorrectness,
        owner: node_site(74),
        source_range: range(source, 0, 319),
        assumptions: Vec::new(),
        goal: InitialObligationGoal::new(domain.clone()),
        provenance: InitialObligationProvenance::new(domain),
        status: InitialObligationStatus::Pending,
    }
}

fn assemble_empty(typed_ast: &TypedAst) -> Result<ResolvedTypedAst, ResolvedTypedAstError> {
    let cluster_facts = ClusterFactTable::new();
    let collection = OverloadCollectionOutput::collect(
        Vec::<OverloadSiteInput>::new(),
        Vec::<OverloadCandidateInput>::new(),
    );
    let expansion = TemplateExpansionOutput::expand(&collection);
    let viability =
        CandidateViabilityOutput::filter(&expansion, Vec::<CandidateViabilityInput>::new());
    let specificity =
        SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
    let selection =
        OverloadSelectionOutput::resolve(&specificity, Vec::<OverloadSiteResolutionInput>::new());
    ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
        typed_ast,
        cluster_facts: &cluster_facts,
        overload_collection: &collection,
        template_expansion: &expansion,
        viability: &viability,
        specificity: &specificity,
        overload_selection: &selection,
        expressions: Vec::new(),
        node_hints: Vec::new(),
        statement_semantics: None,
        statement_proofs: None,
    })
}

fn source_id() -> SourceId {
    source_id_for("e2")
}
fn other_source_id() -> SourceId {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "e2".repeat(32)
    ))
    .unwrap();
    let allocator = InMemorySessionIdAllocator::new();
    allocator.next_source_id(snapshot).unwrap();
    allocator.next_source_id(snapshot).unwrap()
}
fn source_id_for(byte: &str) -> SourceId {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        byte.repeat(32)
    ))
    .unwrap();
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .unwrap()
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

enum NativeInheritanceSourceMutation {
    None,
    ParentKind,
    FrameKind,
    KeywordKind,
    ParentKeyword,
    Identifier,
    DeclarationName,
    FieldKeyword,
    PropertyFrom,
    LocalFieldName,
    ParentPropertyName,
    Duplicate,
}

fn native_inheritance_source(f: &Fixture, mutation: NativeInheritanceSourceMutation) -> TypedAst {
    use mizar_resolve::resolved_ast::SurfaceResolvedArena;
    use mizar_syntax::{SurfaceAstBuilder, SurfaceNodeKind as K, SurfaceTokenKind as T};
    let canonical = include_str!(
        "../../../../tests/miz/pass/types/pass_type_elaboration_structure_definition_payload_001.miz"
    );
    let extra = format!("  {}\n", f.input.inheritances[0].spelling);
    let text = match &mutation {
        NativeInheritanceSourceMutation::Identifier => {
            canonical.replace("inherit Task263Derived", "inherit Task263DeriveX")
        }
        NativeInheritanceSourceMutation::DeclarationName => {
            canonical.replace("struct Task263Derived where", "struct Task263DeriveX where")
        }
        NativeInheritanceSourceMutation::LocalFieldName => canonical.replace(
            "struct Task263Derived where\n    field carrier",
            "struct Task263Derived where\n    field carriex",
        ),
        NativeInheritanceSourceMutation::ParentPropertyName => {
            canonical.replacen("property marker", "property markex", 1)
        }
        NativeInheritanceSourceMutation::Duplicate => {
            canonical.replace("\nend;\n", &format!("\n{extra}end;\n"))
        }
        _ => canonical.to_owned(),
    };
    let words = "definition struct Task263Base where field carrier -> set ; property marker -> set ; end ; struct Task263Derived where field carrier -> set ; property marker -> set ; end ; inherit Task263Derived extends Task263Base where field carrier from carrier ; property marker from marker ; end ; end ;".split_ascii_whitespace().collect::<Vec<_>>();
    let mut builder = SurfaceAstBuilder::new(f.source);
    let mut sites = Vec::new();
    let mut spans = Vec::new();
    let mut cursor = 0;
    for (index, word) in words.iter().enumerate() {
        if index == 48 && matches!(&mutation, NativeInheritanceSourceMutation::Duplicate) {
            cursor += extra.len();
        }
        let word = if (index == 32
            && matches!(&mutation, NativeInheritanceSourceMutation::Identifier))
            || (index == 17
                && matches!(&mutation, NativeInheritanceSourceMutation::DeclarationName))
        {
            "Task263DeriveX"
        } else if index == 20
            && matches!(&mutation, NativeInheritanceSourceMutation::LocalFieldName)
        {
            "carriex"
        } else if index == 10
            && matches!(
                &mutation,
                NativeInheritanceSourceMutation::ParentPropertyName
            )
        {
            "markex"
        } else {
            word
        };
        let start = cursor + text[cursor..].find(word).unwrap();
        cursor = start + word.len();
        let mut kind = match word {
            "Task263Base" | "Task263Derived" | "Task263DeriveX" if index == 32 || index == 34 => {
                T::UserSymbol
            }
            "Task263Base" | "Task263Derived" | "Task263DeriveX" | "carrier" | "carriex"
            | "marker" | "markex" => T::Identifier,
            ";" | "->" => T::ReservedSymbol,
            _ => T::ReservedWord,
        };
        if (index == 31 && matches!(&mutation, NativeInheritanceSourceMutation::KeywordKind))
            || (index == 0 && matches!(&mutation, NativeInheritanceSourceMutation::ParentKeyword))
            || (index == 36 && matches!(&mutation, NativeInheritanceSourceMutation::FieldKeyword))
            || (index == 43 && matches!(&mutation, NativeInheritanceSourceMutation::PropertyFrom))
        {
            kind = T::Identifier;
        }
        sites.push(builder.add_token(kind, word, range(f.source, start, cursor)));
        spans.push((start, cursor));
    }
    let mut duplicate = None;
    let mut duplicate_tokens = Vec::new();
    for (mut kind, children) in [
        (K::StructurePattern, vec![2]),
        (K::TypeHead, vec![7]),
        (K::TypeExpression, vec![51]),
        (K::StructureField, vec![4, 5, 6, 52, 8]),
        (K::TypeHead, vec![12]),
        (K::TypeExpression, vec![54]),
        (K::StructureProperty, vec![9, 10, 11, 55, 13]),
        (K::StructureDefinition, vec![1, 50, 3, 53, 56, 14, 15]),
        (K::StructurePattern, vec![17]),
        (K::TypeHead, vec![22]),
        (K::TypeExpression, vec![59]),
        (K::StructureField, vec![19, 20, 21, 60, 23]),
        (K::TypeHead, vec![27]),
        (K::TypeExpression, vec![62]),
        (K::StructureProperty, vec![24, 25, 26, 63, 28]),
        (K::StructureDefinition, vec![16, 58, 18, 61, 64, 29, 30]),
        (K::InheritanceTarget, vec![32]),
        (K::InheritanceTarget, vec![34]),
        (K::FieldRedefinition, vec![36, 37, 38, 39, 40]),
        (K::PropertyRedefinition, vec![41, 42, 43, 44, 45]),
        (
            K::InheritanceDefinition,
            vec![31, 66, 33, 67, 35, 68, 69, 46, 47],
        ),
        (K::DefinitionBlockItem, vec![0, 57, 65, 70, 48, 49]),
        (K::ItemList, vec![71]),
        (K::CompilationUnit, vec![72]),
        (K::Root, (0..50).chain([73]).collect()),
    ] {
        if kind == K::DefinitionBlockItem
            && matches!(&mutation, NativeInheritanceSourceMutation::Duplicate)
        {
            let mut cursor = 315;
            let mut extra_spans = Vec::new();
            for (index, word) in words[31..48].iter().enumerate() {
                let start = cursor + text[cursor..].find(word).unwrap();
                cursor = start + word.len();
                let token_kind = if index == 1 || index == 3 {
                    T::UserSymbol
                } else if matches!(*word, "carrier" | "marker") {
                    T::Identifier
                } else if *word == ";" {
                    T::ReservedSymbol
                } else {
                    T::ReservedWord
                };
                duplicate_tokens.push(builder.add_token(
                    token_kind,
                    *word,
                    range(f.source, start, cursor),
                ));
                extra_spans.push((start, cursor));
            }
            let target = builder.add_node(
                K::InheritanceTarget,
                range(f.source, extra_spans[1].0, extra_spans[1].1),
                vec![duplicate_tokens[1]],
            );
            let parent = builder.add_node(
                K::InheritanceTarget,
                range(f.source, extra_spans[3].0, extra_spans[3].1),
                vec![duplicate_tokens[3]],
            );
            let field = builder.add_node(
                K::FieldRedefinition,
                range(f.source, extra_spans[5].0, extra_spans[9].1),
                duplicate_tokens[5..10].to_vec(),
            );
            let property = builder.add_node(
                K::PropertyRedefinition,
                range(f.source, extra_spans[10].0, extra_spans[14].1),
                duplicate_tokens[10..15].to_vec(),
            );
            duplicate = Some(builder.add_node(
                K::InheritanceDefinition,
                range(f.source, extra_spans[0].0, extra_spans[16].1),
                vec![
                    duplicate_tokens[0],
                    target,
                    duplicate_tokens[2],
                    parent,
                    duplicate_tokens[4],
                    field,
                    property,
                    duplicate_tokens[15],
                    duplicate_tokens[16],
                ],
            ));
        }
        let start = spans[children[0]].0;
        let end = spans[*children.last().unwrap()].1;
        let mut actual = children
            .iter()
            .map(|index| sites[*index])
            .collect::<Vec<_>>();
        if kind == K::DefinitionBlockItem
            && let Some(owner) = duplicate
        {
            actual.insert(4, owner);
        }
        if kind == K::Root && duplicate.is_some() {
            actual = sites[..48]
                .iter()
                .chain(duplicate_tokens.iter())
                .chain(sites[48..50].iter())
                .copied()
                .chain([sites[73]])
                .collect();
        }
        if (kind == K::DefinitionBlockItem
            && matches!(&mutation, NativeInheritanceSourceMutation::ParentKind))
            || (kind == K::FieldRedefinition
                && matches!(&mutation, NativeInheritanceSourceMutation::FrameKind))
        {
            kind = K::TermList;
        }
        sites.push(builder.add_node(kind, range(f.source, start, end), actual));
        spans.push((start, end));
    }
    let ast = builder.finish(sites.last().copied(), None);
    let source = SurfaceResolvedArena::lower(&ast, &f.module).unwrap();
    let nodes = TypedArena::try_new(
        Some(TypedNodeId::new(source.arena().root().index())),
        source
            .arena()
            .iter()
            .map(|(id, node)| {
                TypedNode::new("syntax", node.origin().anchor().clone())
                    .with_resolved_node(id)
                    .with_children(
                        node.children()
                            .iter()
                            .map(|id| TypedNodeId::new(id.index()))
                            .collect(),
                    )
            })
            .collect(),
    )
    .unwrap();
    let mut syntax = TypedAst::try_new(TypedAstParts {
        source_id: f.source,
        module_id: f.module.clone(),
        resolved_root: Some(source.arena().root()),
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
    .unwrap();
    syntax.install_source_nodes(&source).unwrap();
    syntax
}

#[test]
fn inheritance_native_source_roles_and_compound_priorities() {
    let f = fixture();
    let source = native_inheritance_source(&f, NativeInheritanceSourceMutation::None);
    let before = source.clone();
    let baseline = unrelated_baseline(f.source, "native");
    let projection = f.build_native(&baseline, &source).unwrap();
    assert!(Arc::ptr_eq(
        source.source_nodes().unwrap(),
        projection.handoff.original_source_nodes.as_ref().unwrap()
    ));
    assert_eq!(source, before);
    let missing = f.typed(baseline.clone());
    let mut foreign = f.clone();
    foreign.module = ModuleId::new(PackageId::new("foreign"), ModulePath::new("inheritance"));
    let foreign = native_inheritance_source(&foreign, NativeInheritanceSourceMutation::None);
    for (authority, error) in [
        (
            &missing,
            SourceStructureDefinitionError::InvalidInheritance { index: 0 },
        ),
        (
            &foreign,
            SourceStructureDefinitionError::SourceIdentityMismatch,
        ),
    ] {
        assert_eq!(f.build_native(&baseline, authority), Err(error.clone()));
        for stage in 0..3 {
            let mut earlier = f.clone();
            let expected = match stage {
                0 => {
                    earlier.input.members[3].definition = earlier.input.members[2].definition;
                    SourceStructureDefinitionError::InvalidResolverDefinition { index: 5 }
                }
                1 => {
                    earlier.input.definitions[1].source_ordinal = 9;
                    SourceStructureDefinitionError::InvalidDefinition { index: 1 }
                }
                _ => {
                    earlier.input.members[3].ordinal = 9;
                    SourceStructureDefinitionError::InvalidMember { index: 3 }
                }
            };
            assert_eq!(earlier.build_native(&baseline, authority), Err(expected));
        }
        let mut later = f.clone();
        later.input.mappings[0].path.clear();
        assert_eq!(
            later.build_native(&baseline, &source),
            Err(SourceStructureDefinitionError::InvalidMapping { index: 0 })
        );
        assert_eq!(later.build_native(&baseline, authority), Err(error.clone()));
        let bad_baseline = unrelated_baseline(f.source, "source.definition.functor.bad");
        assert_eq!(
            f.build_native(&bad_baseline, &source),
            Err(SourceStructureDefinitionError::InvalidObligation)
        );
        assert_eq!(f.build_native(&bad_baseline, authority), Err(error.clone()));
        let mut later = f.clone();
        let mut lower = source_type_input(f.source, f.module.clone());
        lower.members[0].member_range.end += 1;
        let owner = lower.members[0].member_range;
        later.arena = TypedArena::try_new(
            later.arena.root(),
            later
                .arena
                .iter()
                .map(|(id, node)| {
                    let mut node = node.clone();
                    if id.index() == 53 {
                        node.anchor = SourceAnchor::Range(owner);
                    }
                    node
                })
                .collect(),
        )
        .unwrap();
        later.source_type = SourceTypeStructureMemberProducer::build(lower, &later.arena).unwrap();
        assert_eq!(
            later.build_native(&baseline, &source),
            Err(SourceStructureDefinitionError::InvalidMember { index: 0 })
        );
        assert_eq!(later.build_native(&baseline, authority), Err(error));
    }
    let mut foreign_id = f.clone();
    foreign_id.source = other_source_id();
    let foreign_id = native_inheritance_source(&foreign_id, NativeInheritanceSourceMutation::None);
    assert_eq!(
        f.build_native(&baseline, &foreign_id),
        Err(SourceStructureDefinitionError::SourceIdentityMismatch)
    );
    for mutation in [
        NativeInheritanceSourceMutation::ParentKind,
        NativeInheritanceSourceMutation::FrameKind,
        NativeInheritanceSourceMutation::KeywordKind,
        NativeInheritanceSourceMutation::ParentKeyword,
        NativeInheritanceSourceMutation::Identifier,
        NativeInheritanceSourceMutation::Duplicate,
    ] {
        let malformed = native_inheritance_source(&f, mutation);
        assert_eq!(
            f.build_native(&baseline, &malformed),
            Err(SourceStructureDefinitionError::InvalidInheritance { index: 0 })
        );
    }
}

#[test]
fn inheritance_native_original_rows_refuse_coherent_forgery_and_replay() {
    let f = fixture();
    let source = native_inheritance_source(&f, NativeInheritanceSourceMutation::None);
    let baseline = unrelated_baseline(f.source, "native-replay");
    let projection = f.build_native(&baseline, &source).unwrap();
    let typed = f
        .typed(baseline.clone())
        .with_source_structure_definition(projection.clone())
        .unwrap();
    assert!(typed.source_nodes().is_none());
    assert!(assemble_empty(&typed).is_ok());
    let mut counterfeit = f.clone();
    let forged_range = range(f.source, 193, 315);
    counterfeit.input.inheritances[0].source_range = forged_range;
    counterfeit.arena = TypedArena::try_new(
        f.arena.root(),
        f.arena
            .iter()
            .map(|(id, node)| {
                let mut node = node.clone();
                if id.index() == 70 {
                    node.anchor = SourceAnchor::Range(forged_range);
                }
                node
            })
            .collect(),
    )
    .unwrap();
    assert_eq!(
        counterfeit.build_native(&baseline, &source),
        Err(SourceStructureDefinitionError::InvalidInheritance { index: 0 })
    );
    let mut forged = projection.clone();
    forged.handoff.inheritances.rows[0].source_range = forged_range;
    assert_eq!(
        counterfeit
            .typed(baseline.clone())
            .with_source_structure_definition(forged.clone()),
        Err(TypedAstError::InvalidSourceStructureDefinition)
    );
    let mut injected = counterfeit.typed(baseline.clone());
    injected.replace_initial_obligations_for_test(forged.initial_obligations);
    injected.inject_source_structure_definition_for_test(forged.handoff);
    assert_eq!(
        assemble_empty(&injected),
        Err(ResolvedTypedAstError::InvalidSourceStructureDefinition)
    );
    let duplicate = native_inheritance_source(&f, NativeInheritanceSourceMutation::Duplicate);
    let mut recovered_duplicate = projection.clone();
    recovered_duplicate.handoff.original_source_nodes = duplicate.source_nodes().cloned();
    let rows = Arc::make_mut(
        recovered_duplicate
            .handoff
            .original_source_nodes
            .as_mut()
            .unwrap(),
    );
    rows.iter_mut()
        .rfind(|row| row.kind == SurfaceNodeKind::InheritanceDefinition)
        .unwrap()
        .recovery = NodeRecoveryState::Recovered;
    assert_eq!(
        f.typed(baseline.clone())
            .with_source_structure_definition(recovered_duplicate),
        Err(TypedAstError::InvalidSourceStructureDefinition)
    );
    let mut foreign = projection.clone();
    Arc::make_mut(foreign.handoff.original_source_nodes.as_mut().unwrap())[0]
        .range
        .source_id = other_source_id();
    assert_installation_error(
        &f,
        &foreign.handoff,
        &f.source_type,
        &baseline,
        &f.arena,
        SourceStructureDefinitionError::SourceIdentityMismatch,
    );
    let mut injected = typed.clone();
    injected.inject_source_structure_definition_for_test(foreign.handoff);
    assert_eq!(
        assemble_empty(&injected),
        Err(ResolvedTypedAstError::InvalidSourceStructureDefinition)
    );
    assert!(Arc::ptr_eq(
        source.source_nodes().unwrap(),
        projection.handoff.original_source_nodes.as_ref().unwrap()
    ));
    assert!(assemble_empty(&typed).is_ok());
}

#[test]
fn inheritance_source_target_and_mapping_rules_have_independent_phase_controls() {
    let f = fixture();
    let baseline = unrelated_baseline(f.source, "role-bundle");
    for (mutation, expected) in [
        (
            NativeInheritanceSourceMutation::DeclarationName,
            SourceStructureDefinitionError::InvalidInheritance { index: 0 },
        ),
        (
            NativeInheritanceSourceMutation::FieldKeyword,
            SourceStructureDefinitionError::InvalidMapping { index: 0 },
        ),
        (
            NativeInheritanceSourceMutation::PropertyFrom,
            SourceStructureDefinitionError::InvalidMapping { index: 1 },
        ),
        (
            NativeInheritanceSourceMutation::LocalFieldName,
            SourceStructureDefinitionError::InvalidMapping { index: 0 },
        ),
        (
            NativeInheritanceSourceMutation::ParentPropertyName,
            SourceStructureDefinitionError::InvalidMapping { index: 1 },
        ),
    ] {
        let source = native_inheritance_source(&f, mutation);
        let before = source.clone();
        let owner = inheritance_definition_rule(source.source_nodes().unwrap()).unwrap();
        assert_eq!(owner.range(), f.input.inheritances[0].source_range);
        assert_eq!(
            owner.spelling(),
            "inherit Task263Derived extends Task263Base where field carrier from carrier ; property marker from marker ; end ;"
        );
        assert_eq!(f.build_native(&baseline, &source), Err(expected));
        assert_eq!(source, before);
    }
}

#[test]
fn inheritance_source_mapping_roles_replay_against_original_members() {
    let f = fixture();
    let baseline = unrelated_baseline(f.source, "role-replay");
    let source = native_inheritance_source(&f, NativeInheritanceSourceMutation::None);
    let projection = f.build_native(&baseline, &source).unwrap();
    let typed = f
        .typed(baseline.clone())
        .with_source_structure_definition(projection.clone())
        .unwrap();
    for (index, mapping_node) in [(0, 68), (1, 69)] {
        let mut counterfeit = f.clone();
        let mut forged_range = counterfeit.input.mappings[index].source_range;
        forged_range.start -= 1;
        forged_range.end += 1;
        counterfeit.input.mappings[index].source_range = forged_range;
        let mut env = SymbolEnvIndexes {
            imports: f.env.imports().clone(),
            exports: f.env.exports().clone(),
            symbols: Default::default(),
            definitions: Default::default(),
            contributions: f.env.contributions().clone(),
            ..SymbolEnvIndexes::default()
        };
        for old in f.env.definitions().iter() {
            let symbol = f.env.symbols().get(old.symbol()).unwrap();
            let origin = if old.id() == f.input.mappings[index].definition {
                SemanticOrigin::new(
                    f.source,
                    f.module.clone(),
                    SourceAnchor::Range(forged_range),
                    old.origin().structural_path().to_vec(),
                )
            } else {
                old.origin().clone()
            };
            let mut entry = SymbolEntry::new(
                old.symbol().clone(),
                symbol.kind(),
                symbol.namespace().clone(),
                symbol.primary_spelling(),
                origin.clone(),
                old.contribution(),
            )
            .with_visibility(symbol.visibility())
            .with_export_status(symbol.export_status());
            if let Some(notation) = symbol.notation_spelling() {
                entry = entry.with_notation_spelling(notation);
            }
            if let Some(signature) = symbol.signature() {
                entry = entry.with_signature(signature.clone());
            }
            env.symbols.insert(entry);
            let mut definition =
                DefinitionShell::new(old.symbol().clone(), old.kind(), origin, old.contribution())
                    .with_visibility(old.visibility());
            if let Some(arity) = old.arity() {
                definition = definition.with_arity(arity);
            }
            if let Some(notation) = old.notation_shape() {
                definition = definition.with_notation_shape(notation);
            }
            if let Some(signature) = old.signature() {
                definition = definition.with_signature(signature.clone());
            }
            assert_eq!(env.definitions.insert(definition), old.id());
        }
        counterfeit.env = SymbolEnv::new(f.module.clone(), env);
        counterfeit.arena = TypedArena::try_new(
            f.arena.root(),
            f.arena
                .iter()
                .map(|(id, old)| {
                    let mut node = old.clone();
                    if id.index() == mapping_node {
                        node.anchor = SourceAnchor::Range(forged_range);
                    }
                    node
                })
                .collect(),
        )
        .unwrap();
        assert_eq!(
            counterfeit.build_native(&baseline, &source),
            Err(SourceStructureDefinitionError::InvalidMapping { index })
        );
        let mut forged = projection.clone();
        forged.handoff.mappings.rows[index].source_range = forged_range;
        forged.handoff.mappings.rows[index].origin = counterfeit
            .env
            .definitions()
            .get(f.input.mappings[index].definition)
            .unwrap()
            .origin()
            .clone();
        assert_installation_error(
            &counterfeit,
            &forged.handoff,
            &counterfeit.source_type,
            &baseline,
            &counterfeit.arena,
            SourceStructureDefinitionError::InvalidMapping { index },
        );
        assert_eq!(
            counterfeit
                .typed(baseline.clone())
                .with_source_structure_definition(forged.clone()),
            Err(TypedAstError::InvalidSourceStructureDefinition)
        );
        let mut injected = counterfeit.typed(baseline.clone());
        injected.inject_source_structure_definition_for_test(forged.handoff);
        assert_eq!(
            assemble_empty(&injected),
            Err(ResolvedTypedAstError::InvalidSourceStructureDefinition)
        );
    }
    for (mutation, error) in [
        (
            NativeInheritanceSourceMutation::DeclarationName,
            SourceStructureDefinitionError::InvalidInheritance { index: 0 },
        ),
        (
            NativeInheritanceSourceMutation::LocalFieldName,
            SourceStructureDefinitionError::InvalidMapping { index: 0 },
        ),
        (
            NativeInheritanceSourceMutation::ParentPropertyName,
            SourceStructureDefinitionError::InvalidMapping { index: 1 },
        ),
    ] {
        let wrong_original = native_inheritance_source(&f, mutation);
        let mut forged = projection.clone();
        forged.handoff.original_source_nodes = wrong_original.source_nodes().cloned();
        assert_installation_error(
            &f,
            &forged.handoff,
            &f.source_type,
            &baseline,
            &f.arena,
            error,
        );
        assert_eq!(
            f.typed(baseline.clone())
                .with_source_structure_definition(forged.clone()),
            Err(TypedAstError::InvalidSourceStructureDefinition)
        );
        let mut injected = typed.clone();
        injected.inject_source_structure_definition_for_test(forged.handoff);
        assert_eq!(
            assemble_empty(&injected),
            Err(ResolvedTypedAstError::InvalidSourceStructureDefinition)
        );
    }
    assert!(Arc::ptr_eq(
        source.source_nodes().unwrap(),
        projection.handoff.original_source_nodes.as_ref().unwrap()
    ));
    assert!(assemble_empty(&typed).is_ok());
}
