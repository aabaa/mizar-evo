use super::*;
use crate::resolved_ast::ModuleId;
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
    SourceId,
};
use mizar_syntax::{SurfaceAstBuilder, SurfaceBuilderNodeId, SurfaceTokenKind, SyntaxRecoveryKind};

#[test]
fn collector_records_represented_declaration_kinds_in_source_order() {
    let source_id = source_id();
    let mut builder = SurfaceAstBuilder::new(source_id);
    let export = export_item(&mut builder, source_id, 0, &["pub", "math"]);
    let import = node(
        &mut builder,
        SurfaceNodeKind::ImportItem,
        source_id,
        10,
        11,
        vec![],
    );
    let reserve = node(
        &mut builder,
        SurfaceNodeKind::ReserveItem,
        source_id,
        12,
        13,
        vec![],
    );
    let theorem = visible_item(
        &mut builder,
        source_id,
        14,
        "public",
        SurfaceNodeKind::TheoremItem,
        21,
        30,
    );
    let lemma = visible_item(
        &mut builder,
        source_id,
        31,
        "private",
        SurfaceNodeKind::LemmaItem,
        39,
        45,
    );
    let definition_block = block_with_children(
        &mut builder,
        source_id,
        SurfaceNodeKind::DefinitionBlockItem,
        46,
        100,
        &[
            SurfaceNodeKind::AttributeDefinition,
            SurfaceNodeKind::PredicateDefinition,
            SurfaceNodeKind::FunctorDefinition,
            SurfaceNodeKind::ModeDefinition,
            SurfaceNodeKind::StructureDefinition,
            SurfaceNodeKind::AlgorithmDefinition,
            SurfaceNodeKind::AttributeRedefinition,
            SurfaceNodeKind::PredicateRedefinition,
            SurfaceNodeKind::FunctorRedefinition,
            SurfaceNodeKind::NotationAlias,
            SurfaceNodeKind::PropertyClause,
            SurfaceNodeKind::StructureField,
            SurfaceNodeKind::StructureProperty,
            SurfaceNodeKind::InheritanceDefinition,
            SurfaceNodeKind::FieldRedefinition,
            SurfaceNodeKind::PropertyRedefinition,
        ],
    );
    let registration_block = block_with_children(
        &mut builder,
        source_id,
        SurfaceNodeKind::RegistrationBlockItem,
        101,
        130,
        &[
            SurfaceNodeKind::ExistentialRegistration,
            SurfaceNodeKind::ConditionalRegistration,
            SurfaceNodeKind::FunctorialRegistration,
            SurfaceNodeKind::ReductionRegistration,
        ],
    );
    let claim = node(
        &mut builder,
        SurfaceNodeKind::ClaimBlockItem,
        source_id,
        131,
        140,
        vec![],
    );
    let placeholder = node(
        &mut builder,
        SurfaceNodeKind::PlaceholderItem,
        source_id,
        141,
        145,
        vec![],
    );
    let root = finish_module(
        &mut builder,
        source_id,
        vec![
            export,
            import,
            reserve,
            theorem,
            lemma,
            definition_block,
            registration_block,
            claim,
            placeholder,
        ],
    );
    let ast = builder.finish(Some(root), None);

    let shells = DeclarationShellCollector::new(&ast, &module_id()).collect();

    assert_eq!(
        declaration_kinds(&shells),
        vec![
            DeclarationShellKind::Reserve,
            DeclarationShellKind::Theorem,
            DeclarationShellKind::Lemma,
            DeclarationShellKind::DefinitionBlock,
            DeclarationShellKind::AttributeDefinition,
            DeclarationShellKind::PredicateDefinition,
            DeclarationShellKind::FunctorDefinition,
            DeclarationShellKind::ModeDefinition,
            DeclarationShellKind::StructureDefinition,
            DeclarationShellKind::AlgorithmDefinition,
            DeclarationShellKind::AttributeRedefinition,
            DeclarationShellKind::PredicateRedefinition,
            DeclarationShellKind::FunctorRedefinition,
            DeclarationShellKind::NotationAlias,
            DeclarationShellKind::PropertyClause,
            DeclarationShellKind::StructureField,
            DeclarationShellKind::StructureProperty,
            DeclarationShellKind::InheritanceDefinition,
            DeclarationShellKind::FieldRedefinition,
            DeclarationShellKind::PropertyRedefinition,
            DeclarationShellKind::RegistrationBlock,
            DeclarationShellKind::ExistentialRegistration,
            DeclarationShellKind::ConditionalRegistration,
            DeclarationShellKind::FunctorialRegistration,
            DeclarationShellKind::ReductionRegistration,
            DeclarationShellKind::ClaimBlock,
            DeclarationShellKind::Placeholder,
        ]
    );
    assert_eq!(shells.exports().len(), 1);
    assert_eq!(shells.exports()[0].paths()[0].spelling(), "pub.math");
    assert_eq!(
        shells.declarations()[1].visibility().state(),
        DeclarationShellVisibilityState::Public
    );
    assert_eq!(
        shells.declarations()[1].visibility().spelling(),
        Some("public")
    );
    assert_eq!(
        shells.declarations()[2].visibility().state(),
        DeclarationShellVisibilityState::Private
    );
    let definition_id = shells.declarations()[3].id();
    assert!(
        shells.declarations()[4..20]
            .iter()
            .all(|shell| shell.parent() == Some(definition_id))
    );
    let registration_id = shells.declarations()[20].id();
    assert!(
        shells.declarations()[21..25]
            .iter()
            .all(|shell| shell.parent() == Some(registration_id))
    );
}

#[test]
fn recovered_subtrees_are_retained_and_marked_recovered() {
    let source_id = source_id();
    let mut builder = SurfaceAstBuilder::new(source_id);
    let recovery = builder.add_recovery(
        SyntaxRecoveryKind::MissingTerm,
        range(source_id, 10, 12),
        Vec::new(),
    );
    let recovered_predicate = node(
        &mut builder,
        SurfaceNodeKind::PredicateDefinition,
        source_id,
        8,
        20,
        vec![recovery],
    );
    let clean_functor = node(
        &mut builder,
        SurfaceNodeKind::FunctorDefinition,
        source_id,
        21,
        27,
        Vec::new(),
    );
    let definition = node(
        &mut builder,
        SurfaceNodeKind::DefinitionBlockItem,
        source_id,
        0,
        30,
        vec![recovered_predicate, clean_functor],
    );
    let dangling_visible = visible_item_without_target(&mut builder, source_id, 31, "public");
    let root = finish_module(&mut builder, source_id, vec![definition, dangling_visible]);
    let ast = builder.finish(Some(root), None);

    let shells = DeclarationShellCollector::new(&ast, &module_id()).collect();

    assert_eq!(
        declaration_kinds(&shells),
        vec![
            DeclarationShellKind::DefinitionBlock,
            DeclarationShellKind::PredicateDefinition,
            DeclarationShellKind::FunctorDefinition,
            DeclarationShellKind::VisibilityWrapper,
        ]
    );
    assert!(shells.declarations()[0].recovered());
    assert!(shells.declarations()[1].recovered());
    assert!(!shells.declarations()[2].recovered());
    assert!(shells.declarations()[3].recovered());
    assert_eq!(
        shells.declarations()[3].visibility().state(),
        DeclarationShellVisibilityState::Public
    );
}

#[test]
fn annotation_wrappers_are_transparent_for_shell_collection() {
    let source_id = source_id();
    let mut builder = SurfaceAstBuilder::new(source_id);
    let annotation = node(
        &mut builder,
        SurfaceNodeKind::LibraryAnnotation,
        source_id,
        1,
        4,
        Vec::new(),
    );
    let predicate = node(
        &mut builder,
        SurfaceNodeKind::PredicateDefinition,
        source_id,
        6,
        12,
        Vec::new(),
    );
    let recovery = builder.add_recovery(
        SyntaxRecoveryKind::SkippedToken,
        range(source_id, 13, 14),
        Vec::new(),
    );
    let annotated_definition = node(
        &mut builder,
        SurfaceNodeKind::AnnotatedDefinitionContent,
        source_id,
        1,
        14,
        vec![annotation, predicate, recovery],
    );
    let definition = node(
        &mut builder,
        SurfaceNodeKind::DefinitionBlockItem,
        source_id,
        0,
        20,
        vec![annotated_definition],
    );
    let registration = node(
        &mut builder,
        SurfaceNodeKind::ConditionalRegistration,
        source_id,
        31,
        38,
        Vec::new(),
    );
    let annotated_registration = node(
        &mut builder,
        SurfaceNodeKind::AnnotatedRegistrationContent,
        source_id,
        30,
        39,
        vec![registration],
    );
    let registration_block = node(
        &mut builder,
        SurfaceNodeKind::RegistrationBlockItem,
        source_id,
        29,
        45,
        vec![annotated_registration],
    );
    let inline_functor = node(
        &mut builder,
        SurfaceNodeKind::InlineFunctorDefinition,
        source_id,
        51,
        55,
        Vec::new(),
    );
    let annotated_statement = node(
        &mut builder,
        SurfaceNodeKind::AnnotatedStatement,
        source_id,
        50,
        56,
        vec![inline_functor],
    );
    let variable = node(
        &mut builder,
        SurfaceNodeKind::VariableDeclaration,
        source_id,
        57,
        60,
        Vec::new(),
    );
    let annotated_algorithm_statement = node(
        &mut builder,
        SurfaceNodeKind::AnnotatedAlgorithmStatement,
        source_id,
        56,
        61,
        vec![variable],
    );
    let standalone_annotation = node(
        &mut builder,
        SurfaceNodeKind::StandaloneDiagnosticAnnotation,
        source_id,
        62,
        66,
        Vec::new(),
    );
    let root = finish_module(
        &mut builder,
        source_id,
        vec![
            definition,
            registration_block,
            annotated_statement,
            annotated_algorithm_statement,
            standalone_annotation,
        ],
    );
    let ast = builder.finish(Some(root), None);

    let shells = DeclarationShellCollector::new(&ast, &module_id()).collect();

    assert_eq!(
        declaration_kinds(&shells),
        vec![
            DeclarationShellKind::DefinitionBlock,
            DeclarationShellKind::PredicateDefinition,
            DeclarationShellKind::RegistrationBlock,
            DeclarationShellKind::ConditionalRegistration,
        ]
    );
    assert_eq!(
        shells.declarations()[1].parent(),
        Some(shells.declarations()[0].id())
    );
    assert!(shells.declarations()[1].recovered());
    assert_eq!(
        shells.declarations()[3].parent(),
        Some(shells.declarations()[2].id())
    );
    assert!(!shells.declarations()[3].recovered());
}

#[test]
fn excluded_context_body_statement_and_recovery_nodes_do_not_create_shells() {
    let source_id = source_id();
    let mut builder = SurfaceAstBuilder::new(source_id);
    let module_path = module_path(&mut builder, source_id, 1, &["pkg", "names"]);
    let parameter = node(
        &mut builder,
        SurfaceNodeKind::DefinitionParameter,
        source_id,
        12,
        15,
        Vec::new(),
    );
    let correctness = node(
        &mut builder,
        SurfaceNodeKind::CorrectnessCondition,
        source_id,
        16,
        21,
        Vec::new(),
    );
    let proof = node(
        &mut builder,
        SurfaceNodeKind::ProofBlock,
        source_id,
        22,
        28,
        Vec::new(),
    );
    let pattern = node(
        &mut builder,
        SurfaceNodeKind::PredicatePattern,
        source_id,
        29,
        33,
        Vec::new(),
    );
    let inline_functor = node(
        &mut builder,
        SurfaceNodeKind::InlineFunctorDefinition,
        source_id,
        34,
        42,
        Vec::new(),
    );
    let inline_predicate = node(
        &mut builder,
        SurfaceNodeKind::InlinePredicateDefinition,
        source_id,
        43,
        51,
        Vec::new(),
    );
    let raw_recovery = builder.add_recovery(
        SyntaxRecoveryKind::SkippedToken,
        range(source_id, 52, 53),
        Vec::new(),
    );
    let definition = node(
        &mut builder,
        SurfaceNodeKind::DefinitionBlockItem,
        source_id,
        10,
        54,
        vec![
            parameter,
            correctness,
            proof,
            pattern,
            inline_functor,
            inline_predicate,
            raw_recovery,
        ],
    );
    let variable = node(
        &mut builder,
        SurfaceNodeKind::VariableDeclaration,
        source_id,
        61,
        65,
        Vec::new(),
    );
    let algorithm_body = node(
        &mut builder,
        SurfaceNodeKind::AlgorithmBody,
        source_id,
        60,
        66,
        vec![variable],
    );
    let algorithm = node(
        &mut builder,
        SurfaceNodeKind::AlgorithmDefinition,
        source_id,
        56,
        67,
        vec![algorithm_body],
    );
    let root = finish_module(
        &mut builder,
        source_id,
        vec![module_path, definition, algorithm],
    );
    let ast = builder.finish(Some(root), None);

    let shells = DeclarationShellCollector::new(&ast, &module_id()).collect();

    assert_eq!(
        declaration_kinds(&shells),
        vec![
            DeclarationShellKind::DefinitionBlock,
            DeclarationShellKind::AlgorithmDefinition,
        ]
    );
    assert!(shells.declarations()[0].recovered());
    assert!(!shells.declarations()[1].recovered());
}

#[test]
fn malformed_export_projection_is_retained_without_target_validation() {
    let source_id = source_id();
    let mut builder = SurfaceAstBuilder::new(source_id);
    let good_export = export_item(&mut builder, source_id, 0, &["pkg", "core"]);
    let recovery = builder.add_recovery(
        SyntaxRecoveryKind::SkippedToken,
        range(source_id, 20, 25),
        Vec::new(),
    );
    let bad_export = node(
        &mut builder,
        SurfaceNodeKind::ExportItem,
        source_id,
        18,
        26,
        vec![recovery],
    );
    let root = finish_module(&mut builder, source_id, vec![good_export, bad_export]);
    let ast = builder.finish(Some(root), None);

    let shells = DeclarationShellCollector::new(&ast, &module_id()).collect();

    assert!(shells.declarations().is_empty());
    assert_eq!(shells.exports().len(), 2);
    assert_eq!(shells.exports()[0].paths()[0].spelling(), "pkg.core");
    assert!(!shells.exports()[0].recovered());
    assert!(shells.exports()[1].paths().is_empty());
    assert!(shells.exports()[1].recovered());
}

fn declaration_kinds(shells: &DeclarationShellSet) -> Vec<DeclarationShellKind> {
    shells
        .declarations()
        .iter()
        .map(DeclarationShell::kind)
        .collect()
}

fn block_with_children(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    kind: SurfaceNodeKind,
    start: usize,
    end: usize,
    child_kinds: &[SurfaceNodeKind],
) -> SurfaceBuilderNodeId {
    let mut children = Vec::new();
    for (index, child_kind) in child_kinds.iter().enumerate() {
        let child_start = start + 1 + index * 2;
        children.push(node(
            builder,
            child_kind.clone(),
            source_id,
            child_start,
            child_start + 1,
            Vec::new(),
        ));
    }
    node(builder, kind, source_id, start, end, children)
}

fn visible_item(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    spelling: &str,
    target_kind: SurfaceNodeKind,
    target_start: usize,
    target_end: usize,
) -> SurfaceBuilderNodeId {
    let marker = visibility_marker(builder, source_id, start, spelling);
    let target = node(
        builder,
        target_kind,
        source_id,
        target_start,
        target_end,
        Vec::new(),
    );
    node(
        builder,
        SurfaceNodeKind::VisibleItem,
        source_id,
        start,
        target_end,
        vec![marker, target],
    )
}

fn visible_item_without_target(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    spelling: &str,
) -> SurfaceBuilderNodeId {
    let marker = visibility_marker(builder, source_id, start, spelling);
    node(
        builder,
        SurfaceNodeKind::VisibleItem,
        source_id,
        start,
        start + spelling.len(),
        vec![marker],
    )
}

fn visibility_marker(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    spelling: &str,
) -> SurfaceBuilderNodeId {
    let token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        spelling,
        range(source_id, start, start + spelling.len()),
    );
    node(
        builder,
        SurfaceNodeKind::VisibilityMarker,
        source_id,
        start,
        start + spelling.len(),
        vec![token],
    )
}

fn export_item(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    components: &[&str],
) -> SurfaceBuilderNodeId {
    let path = module_path(builder, source_id, start + 1, components);
    node(
        builder,
        SurfaceNodeKind::ExportItem,
        source_id,
        start,
        start + 1 + components.join(".").len(),
        vec![path],
    )
}

fn module_path(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    components: &[&str],
) -> SurfaceBuilderNodeId {
    let mut children = Vec::new();
    let mut cursor = start;
    for component in components {
        let token = builder.add_token(
            SurfaceTokenKind::Identifier,
            *component,
            range(source_id, cursor, cursor + component.len()),
        );
        children.push(node(
            builder,
            SurfaceNodeKind::PathSegment,
            source_id,
            cursor,
            cursor + component.len(),
            vec![token],
        ));
        cursor += component.len() + 1;
    }
    node(
        builder,
        SurfaceNodeKind::ModulePath,
        source_id,
        start,
        cursor.saturating_sub(1),
        children,
    )
}

fn finish_module(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    items: Vec<SurfaceBuilderNodeId>,
) -> SurfaceBuilderNodeId {
    let item_list = node(builder, SurfaceNodeKind::ItemList, source_id, 0, 200, items);
    let unit = node(
        builder,
        SurfaceNodeKind::CompilationUnit,
        source_id,
        0,
        200,
        vec![item_list],
    );
    node(
        builder,
        SurfaceNodeKind::Root,
        source_id,
        0,
        200,
        vec![unit],
    )
}

fn node(
    builder: &mut SurfaceAstBuilder,
    kind: SurfaceNodeKind,
    source_id: SourceId,
    start: usize,
    end: usize,
    children: Vec<SurfaceBuilderNodeId>,
) -> SurfaceBuilderNodeId {
    builder.add_node(kind, range(source_id, start, end), children)
}

fn module_id() -> ModuleId {
    ModuleId::new(PackageId::new("app"), ModulePath::new("main"))
}

fn source_id() -> SourceId {
    let snapshot_id = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "04".repeat(Hash::BYTE_LEN)
    ))
    .unwrap();
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot_id)
        .unwrap()
}

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}
