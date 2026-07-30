use super::*;
use mizar_session::{
    BuildSnapshotId, GeneratedSpanAnchor, GeneratedSpanOrigin, Hash, InMemorySessionIdAllocator,
    SessionIdAllocator, SourceAnchor,
};
use mizar_syntax::ast::SurfaceNodeKind;
use mizar_syntax::{
    SurfaceAst, SurfaceAstBuilder, SurfaceFormulaConstant, SurfaceTokenKind, SyntaxRecoveryKind,
};

#[test]
fn module_and_symbol_ids_are_deterministic_and_alias_independent() {
    let module = module_id("pkg", "algebra.group");
    let same_module = module_id("pkg", "algebra.group");
    let alias_spelling_does_not_participate = module_id("pkg", "algebra.group");

    assert_eq!(module, same_module);
    assert_eq!(module, alias_spelling_does_not_participate);

    let symbol = symbol_id(module.clone(), "pred/0", "pkg::algebra.group::pred/0");
    let same_symbol = symbol_id(same_module, "pred/0", "pkg::algebra.group::pred/0");

    assert_eq!(symbol, same_symbol);
    assert_eq!(symbol.fqn().as_str(), "pkg::algebra.group::pred/0");
}

#[test]
fn arena_allocates_deterministic_ids_and_validates_children() {
    let source_id = source_id(1);
    let module = module_id("pkg", "main");
    let origin = origin(source_id, module);
    let mut builder = ResolvedArenaBuilder::new();

    let child = builder
        .push(ResolvedNode::new(
            SurfaceNodeKind::PlaceholderItem,
            Vec::new(),
            origin.clone(),
        ))
        .unwrap();
    let parent = builder
        .push(ResolvedNode::new(
            SurfaceNodeKind::ItemList,
            vec![child],
            origin.clone(),
        ))
        .unwrap();
    let arena = builder.finish(parent).unwrap();

    assert_eq!(child.index(), 0);
    assert_eq!(parent.index(), 1);
    assert_eq!(arena.node(parent).unwrap().children(), &[child]);

    let invalid = ResolvedNode::new(
        SurfaceNodeKind::CompilationUnit,
        vec![ResolvedNodeId::new(99)],
        origin,
    );
    assert!(matches!(
        ResolvedArena::try_new(ResolvedNodeId::new(0), vec![invalid]),
        Err(ResolvedArenaError::InvalidChild { .. })
    ));
}

#[test]
fn arena_rejects_cycles() {
    let source_id = source_id(2);
    let module = module_id("pkg", "main");
    let origin = origin(source_id, module);
    let first = ResolvedNode::new(
        SurfaceNodeKind::ItemList,
        vec![ResolvedNodeId::new(1)],
        origin.clone(),
    );
    let second = ResolvedNode::new(
        SurfaceNodeKind::PlaceholderItem,
        vec![ResolvedNodeId::new(0)],
        origin,
    );

    assert!(matches!(
        ResolvedArena::try_new(ResolvedNodeId::new(0), vec![first, second]),
        Err(ResolvedArenaError::Cycle { .. })
    ));
}

#[test]
fn surface_structural_arena_lowers_every_dense_node_same_index_and_deterministically() {
    let source_id = source_id(31);
    let module = module_id("pkg", "surface");
    let ast = surface_ast_fixture(source_id);
    let lowered = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let repeated = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let equivalent_ast = surface_ast_fixture(source_id);
    let equivalent_module = module_id("pkg", "surface");
    let equivalent = SurfaceResolvedArena::lower(&equivalent_ast, &equivalent_module).unwrap();
    let surface = ast.node_views().collect::<Vec<_>>();

    assert_eq!(lowered, repeated);
    assert_eq!(lowered, equivalent);
    assert_eq!(lowered.source_id(), source_id);
    assert_eq!(lowered.module(), &module);
    assert_eq!(lowered.arena().len(), surface.len());
    assert_eq!(lowered.arena().root().index(), 3);
    assert_eq!(surface.last().unwrap().id().index(), 4);

    for view in surface {
        let resolved_id = lowered.resolved_node_for(view.id()).unwrap();
        let resolved = lowered.arena().node(resolved_id).unwrap();
        assert_eq!(resolved_id.index(), view.id().index());
        assert_eq!(resolved.kind(), view.kind());
        assert_eq!(
            resolved
                .children()
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>(),
            view.children()
                .iter()
                .map(|child| child.index())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            resolved.origin().anchor(),
            &SourceAnchor::Range(view.range())
        );
        assert_eq!(
            resolved.origin().structural_path(),
            &[u32::try_from(view.id().index()).unwrap()]
        );
        assert_eq!(resolved.origin().source_id(), source_id);
        assert_eq!(resolved.origin().module_id(), &module);
        assert_eq!(resolved.origin().import_edge(), None);
        assert_eq!(
            resolved.recovery(),
            if view.is_recovered() {
                RecoveryState::Recovered
            } else {
                RecoveryState::Normal
            }
        );
        assert_eq!(resolved.origin().is_recovered(), view.is_recovered());
        assert_eq!(resolved.resolution(), NodeResolutionState::NotApplicable);
        assert_eq!(resolved.reference_key(), None);
    }

    let foreign = larger_surface_ast_fixture(source_id)
        .node_views()
        .last()
        .unwrap()
        .id();
    assert!(foreign.index() >= lowered.arena().len());
    assert_eq!(lowered.resolved_node_for(foreign), None);
}

#[test]
fn surface_structural_arena_rejects_missing_root() {
    let source_id = source_id(32);
    let module = module_id("pkg", "surface");
    let mut builder = SurfaceAstBuilder::new(source_id);
    builder.add_node(
        SurfaceNodeKind::PlaceholderItem,
        range(source_id, 0, 1),
        Vec::new(),
    );
    let ast = builder.finish(None, None);

    assert!(matches!(
        SurfaceResolvedArena::lower(&ast, &module),
        Err(SurfaceResolvedArenaError::MissingRoot)
    ));

    let rooted = surface_ast_fixture(source_id);
    let lowered = SurfaceResolvedArena::lower(&rooted, &module).unwrap();
    assert!(matches!(
        lowered.validate_against(&ast, &module),
        Err(SurfaceResolvedArenaError::NodeCountMismatch)
    ));

    let mut same_count_builder = SurfaceAstBuilder::new(source_id);
    for index in 0..rooted.node_views().len() {
        same_count_builder.add_node(
            SurfaceNodeKind::PlaceholderItem,
            range(source_id, index, index + 1),
            Vec::new(),
        );
    }
    let same_count_without_root = same_count_builder.finish(None, None);
    assert!(matches!(
        lowered.validate_against(&same_count_without_root, &module),
        Err(SurfaceResolvedArenaError::MissingRoot)
    ));
}

#[test]
fn surface_structural_arena_rejects_each_top_level_and_node_mismatch() {
    let (source_id, other_source_id) = source_id_pair(33);
    let module = module_id("pkg", "surface");
    let other_module = module_id("pkg", "other");
    let ast = surface_ast_fixture(source_id);
    let lowered = SurfaceResolvedArena::lower(&ast, &module).unwrap();

    let mut changed = lowered.clone();
    changed.source_id = other_source_id;
    assert!(matches!(
        changed.validate_against(&ast, &module),
        Err(SurfaceResolvedArenaError::SourceMismatch)
    ));

    let mut changed = lowered.clone();
    changed.module = other_module.clone();
    assert!(matches!(
        changed.validate_against(&ast, &module),
        Err(SurfaceResolvedArenaError::ModuleMismatch)
    ));

    let mut changed = lowered.clone();
    changed.arena.nodes.pop();
    assert!(matches!(
        changed.validate_against(&ast, &module),
        Err(SurfaceResolvedArenaError::NodeCountMismatch)
    ));

    let mut changed = lowered.clone();
    changed.arena.root = ResolvedNodeId::new(2);
    assert!(matches!(
        changed.validate_against(&ast, &module),
        Err(SurfaceResolvedArenaError::RootMismatch)
    ));

    let ids = ast.node_views().map(|node| node.id()).collect::<Vec<_>>();
    assert_surface_fault(
        &lowered,
        &ast,
        &module,
        ids[2],
        SurfaceArenaFault::Kind,
        "kind",
    );
    assert_surface_fault(
        &lowered,
        &ast,
        &module,
        ids[2],
        SurfaceArenaFault::Children,
        "children",
    );
    assert_surface_fault(
        &lowered,
        &ast,
        &module,
        ids[2],
        SurfaceArenaFault::Range,
        "range",
    );
    assert_surface_fault(
        &lowered,
        &ast,
        &module,
        ids[2],
        SurfaceArenaFault::Recovery,
        "recovery",
    );
    assert_surface_fault(
        &lowered,
        &ast,
        &module,
        ids[2],
        SurfaceArenaFault::Resolution,
        "resolution",
    );
    assert_surface_fault(
        &lowered,
        &ast,
        &module,
        ids[2],
        SurfaceArenaFault::ReferenceKey,
        "reference-key",
    );
    assert_surface_fault(
        &lowered,
        &ast,
        &module,
        ids[2],
        SurfaceArenaFault::OriginSource(other_source_id),
        "origin",
    );
    assert_surface_fault(
        &lowered,
        &ast,
        &module,
        ids[2],
        SurfaceArenaFault::StructuralPath,
        "structural-path",
    );

    for fault in [
        SurfaceArenaFault::OriginSource(other_source_id),
        SurfaceArenaFault::OriginModule(other_module),
        SurfaceArenaFault::OriginImport,
    ] {
        assert_surface_fault(&lowered, &ast, &module, ids[2], fault, "origin");
    }

    let mut changed = lowered.clone();
    changed.arena.nodes[1].origin.recovered = false;
    assert!(matches!(
        changed.validate_against(&ast, &module),
        Err(SurfaceResolvedArenaError::RecoveryMismatch { node })
            if node == ids[1]
    ));
}

#[test]
fn surface_structural_arena_rejects_invalid_contained_arena_shapes() {
    let source_id = source_id(34);
    let module = module_id("pkg", "surface");
    let ast = surface_ast_fixture(source_id);
    let lowered = SurfaceResolvedArena::lower(&ast, &module).unwrap();

    let mut invalid_child = lowered.clone();
    invalid_child.arena.nodes[0].children = vec![ResolvedNodeId::new(99)];
    assert!(matches!(
        invalid_child.validate_against(&ast, &module),
        Err(SurfaceResolvedArenaError::InvalidArena(
            ResolvedArenaError::InvalidChild { node, child }
        )) if node.index() == 0 && child.index() == 99
    ));

    let mut invalid_root = lowered.clone();
    invalid_root.arena.root = ResolvedNodeId::new(99);
    assert!(matches!(
        invalid_root.validate_against(&ast, &module),
        Err(SurfaceResolvedArenaError::InvalidArena(
            ResolvedArenaError::InvalidRoot { root }
        )) if root.index() == 99
    ));

    let mut cycle = lowered;
    cycle.arena.nodes[0].children = vec![ResolvedNodeId::new(0)];
    assert!(matches!(
        cycle.validate_against(&ast, &module),
        Err(SurfaceResolvedArenaError::InvalidArena(
            ResolvedArenaError::Cycle { node }
        )) if node.index() == 0
    ));
}

#[test]
fn surface_structural_arena_validation_freezes_top_level_precedence() {
    let (source_id, other_source_id) = source_id_pair(35);
    let module = module_id("pkg", "surface");
    let other_module = module_id("pkg", "other");
    let ast = surface_ast_fixture(source_id);
    let lowered = SurfaceResolvedArena::lower(&ast, &module).unwrap();

    let mut changed = lowered.clone();
    changed.source_id = other_source_id;
    changed.module = other_module.clone();
    changed.arena.nodes[0].children = vec![ResolvedNodeId::new(99)];
    changed.arena.nodes.pop();
    changed.arena.root = ResolvedNodeId::new(2);
    assert_eq!(
        surface_error_class(changed.validate_against(&ast, &module).unwrap_err()),
        "source"
    );

    let mut changed = lowered.clone();
    changed.module = other_module;
    changed.arena.nodes[0].children = vec![ResolvedNodeId::new(99)];
    changed.arena.nodes.pop();
    changed.arena.root = ResolvedNodeId::new(2);
    assert_eq!(
        surface_error_class(changed.validate_against(&ast, &module).unwrap_err()),
        "module"
    );

    let mut changed = lowered.clone();
    changed.arena.nodes[0].children = vec![ResolvedNodeId::new(99)];
    changed.arena.nodes.pop();
    changed.arena.root = ResolvedNodeId::new(2);
    assert_eq!(
        surface_error_class(changed.validate_against(&ast, &module).unwrap_err()),
        "arena"
    );

    let mut changed = lowered.clone();
    changed.arena.nodes.pop();
    changed.arena.root = ResolvedNodeId::new(2);
    assert_eq!(
        surface_error_class(changed.validate_against(&ast, &module).unwrap_err()),
        "count"
    );

    let mut changed = lowered;
    changed.arena.root = ResolvedNodeId::new(2);
    changed.arena.nodes[0].kind = SurfaceNodeKind::ItemList;
    assert_eq!(
        surface_error_class(changed.validate_against(&ast, &module).unwrap_err()),
        "root"
    );
}

#[test]
fn surface_structural_arena_validation_freezes_per_node_precedence() {
    let (source_id, other_source_id) = source_id_pair(36);
    let module = module_id("pkg", "surface");
    let ast = surface_ast_fixture(source_id);
    let lowered = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let node = ast.node_views().nth(2).unwrap().id();
    let ordered_faults = [
        SurfaceArenaFault::Kind,
        SurfaceArenaFault::Children,
        SurfaceArenaFault::Range,
        SurfaceArenaFault::Recovery,
        SurfaceArenaFault::Resolution,
        SurfaceArenaFault::ReferenceKey,
        SurfaceArenaFault::OriginSource(other_source_id),
        SurfaceArenaFault::StructuralPath,
    ];
    let expected = [
        "kind",
        "children",
        "range",
        "recovery",
        "resolution",
        "reference-key",
        "origin",
    ];

    for (faults, expected) in ordered_faults.windows(2).zip(expected) {
        let mut changed = lowered.clone();
        apply_surface_fault(&mut changed, node, faults[1].clone());
        apply_surface_fault(&mut changed, node, faults[0].clone());
        assert_eq!(
            surface_error_class(changed.validate_against(&ast, &module).unwrap_err()),
            expected
        );
    }
}

#[test]
fn surface_structural_arena_reports_earlier_dense_node_before_later_faults() {
    let (source_id, other_source_id) = source_id_pair(37);
    let module = module_id("pkg", "surface");
    let ast = surface_ast_fixture(source_id);
    let lowered = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let ids = ast.node_views().map(|node| node.id()).collect::<Vec<_>>();

    for later_fault in [
        SurfaceArenaFault::Kind,
        SurfaceArenaFault::Children,
        SurfaceArenaFault::Range,
        SurfaceArenaFault::Recovery,
        SurfaceArenaFault::Resolution,
        SurfaceArenaFault::ReferenceKey,
        SurfaceArenaFault::OriginSource(other_source_id),
        SurfaceArenaFault::StructuralPath,
    ] {
        let mut changed = lowered.clone();
        apply_surface_fault(&mut changed, ids[4], later_fault);
        apply_surface_fault(&mut changed, ids[0], SurfaceArenaFault::Kind);
        assert!(matches!(
            changed.validate_against(&ast, &module),
            Err(SurfaceResolvedArenaError::NodeKindMismatch { node })
                if node == ids[0]
        ));
    }
}

#[test]
fn surface_structural_arena_checked_helpers_report_exact_payloads() {
    let source_id = source_id(38);
    let ast = surface_ast_fixture(source_id);
    let ids = ast.node_views().map(|node| node.id()).collect::<Vec<_>>();

    assert!(matches!(
        checked_surface_children(ids[0], &[ids[1]]),
        Err(SurfaceResolvedArenaError::InvalidChildOrder { node, child })
            if node == ids[0] && child == ids[1]
    ));

    #[cfg(target_pointer_width = "64")]
    {
        let overflowing_index = usize::try_from(u64::from(u32::MAX) + 1).unwrap();
        assert!(matches!(
            checked_surface_structural_path(ids[0], overflowing_index),
            Err(SurfaceResolvedArenaError::StructuralPathComponentOverflow { node })
                if node == ids[0]
        ));
    }
}

#[test]
fn name_ref_table_round_trips_all_current_result_kinds() {
    let source_id = source_id(3);
    let site_range = range(source_id, 1, 4);
    let node = ResolvedNodeId::new(0);
    let module = module_id("pkg", "main");
    let entry_origin = origin(source_id, module.clone());
    let symbol = symbol_id(module, "pred/0", "pkg::main::pred/0");
    let candidate_a = NameResolutionCandidate::new(symbol.clone(), range(source_id, 10, 12));
    let candidate_b = NameResolutionCandidate::new(
        symbol_id(module_id("pkg", "other"), "pred/0", "pkg::other::pred/0"),
        range(source_id, 8, 9),
    );
    let mut table = NameRefTable::new();

    let resolved = table.insert(NameRefEntry::new(
        ReferenceSite::new(node, site_range, "P"),
        NameResolution::Resolved(SymbolRef::new(symbol, site_range)),
        entry_origin.clone(),
    ));
    let builtin = table.insert(NameRefEntry::new(
        ReferenceSite::new(node, site_range, "true"),
        NameResolution::ResolvedBuiltin(BuiltinRef::new(
            BuiltinId::new("true"),
            site_range,
            "true",
        )),
        entry_origin.clone(),
    ));
    let deferred = table.insert(NameRefEntry::new(
        ReferenceSite::new(node, site_range, "x.y"),
        NameResolution::DeferredSelector(DeferredSelectorRef::new(node, "y", site_range)),
        entry_origin.clone(),
    ));
    let ambiguous = table.insert(NameRefEntry::new(
        ReferenceSite::new(node, site_range, "P"),
        NameResolution::Ambiguous(AmbiguousNameRef::new(
            "P",
            site_range,
            vec![candidate_a, candidate_b],
        )),
        entry_origin.clone(),
    ));
    let unresolved = table.insert(NameRefEntry::new(
        ReferenceSite::new(node, site_range, "Missing"),
        NameResolution::Unresolved(UnresolvedNameRef::new(
            "Missing",
            site_range,
            NameLookupClass::Symbol,
        )),
        entry_origin,
    ));

    assert!(matches!(
        table.get(resolved).unwrap().resolution(),
        NameResolution::Resolved(_)
    ));
    let NameResolution::Resolved(symbol_ref) = table.get(resolved).unwrap().resolution() else {
        panic!("expected resolved symbol ref");
    };
    assert_eq!(symbol_ref.range(), site_range);
    assert_eq!(symbol_ref.spelling(), None);
    assert!(matches!(
        table.get(builtin).unwrap().resolution(),
        NameResolution::ResolvedBuiltin(_)
    ));
    assert!(matches!(
        table.get(deferred).unwrap().resolution(),
        NameResolution::DeferredSelector(_)
    ));
    let NameResolution::Ambiguous(ambiguous_ref) = table.get(ambiguous).unwrap().resolution()
    else {
        panic!("expected ambiguous name ref");
    };
    assert_eq!(
        ambiguous_ref
            .candidates()
            .iter()
            .map(|candidate| candidate.symbol().fqn().as_str())
            .collect::<Vec<_>>(),
        vec!["pkg::main::pred/0", "pkg::other::pred/0"]
    );
    assert!(matches!(
        table.get(unresolved).unwrap().resolution(),
        NameResolution::Unresolved(_)
    ));
}

#[test]
fn ambiguous_name_candidates_tie_break_by_range_before_local_symbol_id() {
    let source_id = source_id(10);
    let module = module_id("pkg", "main");
    let late_range = range(source_id, 20, 22);
    let early_range = range(source_id, 10, 12);
    let ambiguous = AmbiguousNameRef::new(
        "P",
        range(source_id, 0, 1),
        vec![
            NameResolutionCandidate::new(
                symbol_id(module.clone(), "a-local", "pkg::main::P"),
                late_range,
            ),
            NameResolutionCandidate::new(
                symbol_id(module.clone(), "z-local", "pkg::main::P"),
                early_range,
            ),
        ],
    );

    assert_eq!(
        ambiguous
            .candidates()
            .iter()
            .map(|candidate| (candidate.symbol().local().as_str(), candidate.range().start))
            .collect::<Vec<_>>(),
        vec![("z-local", 10), ("a-local", 20)]
    );

    let local_tie = AmbiguousNameRef::new(
        "P",
        range(source_id, 0, 1),
        vec![
            NameResolutionCandidate::new(
                symbol_id(module.clone(), "z-local", "pkg::main::P"),
                range(source_id, 0, 1),
            ),
            NameResolutionCandidate::new(
                symbol_id(module.clone(), "a-local", "pkg::main::P"),
                range(source_id, 0, 1),
            ),
        ],
    );
    assert_eq!(
        local_tie
            .candidates()
            .iter()
            .map(|candidate| candidate.symbol().local().as_str())
            .collect::<Vec<_>>(),
        vec!["a-local", "z-local"]
    );

    let module_tie = AmbiguousNameRef::new(
        "P",
        range(source_id, 0, 1),
        vec![
            NameResolutionCandidate::new(
                symbol_id(module_id("pkg", "zeta"), "same-local", "pkg::P"),
                range(source_id, 0, 1),
            ),
            NameResolutionCandidate::new(
                symbol_id(module_id("pkg", "alpha"), "same-local", "pkg::P"),
                range(source_id, 0, 1),
            ),
        ],
    );
    assert_eq!(
        module_tie
            .candidates()
            .iter()
            .map(|candidate| candidate.symbol().module().path().as_str())
            .collect::<Vec<_>>(),
        vec!["alpha", "zeta"]
    );
}

#[test]
fn label_ref_table_round_trips_all_current_result_kinds() {
    let source_id = source_id(4);
    let range = range(source_id, 0, 2);
    let node = ResolvedNodeId::new(0);
    let entry_origin = origin(source_id, module_id("pkg", "main"));
    let mut table = LabelRefTable::new();

    let resolved = table.insert(LabelRefEntry::new(
        ReferenceSite::new(node, range, "A1"),
        LabelResolution::Resolved(LabelRef::new(
            LabelOriginPath::new("pkg::main::A1"),
            LabelKind::Theorem,
            range,
        )),
        entry_origin.clone(),
    ));
    let ambiguous = table.insert(LabelRefEntry::new(
        ReferenceSite::new(node, range, "A1"),
        LabelResolution::Ambiguous(AmbiguousLabelRef::new(
            "A1",
            range,
            vec![
                LabelCandidate::new(
                    LabelOriginPath::new("pkg::main::B1"),
                    LabelKind::ProofStep,
                    range,
                ),
                LabelCandidate::new(
                    LabelOriginPath::new("pkg::main::A1"),
                    LabelKind::Theorem,
                    range,
                ),
            ],
        )),
        entry_origin.clone(),
    ));
    let unresolved = table.insert(LabelRefEntry::new(
        ReferenceSite::new(node, range, "A2"),
        LabelResolution::Unresolved(UnresolvedLabelRef::new(
            "A2",
            range,
            LabelExpectation::Theorem,
        )),
        entry_origin,
    ));

    assert!(matches!(
        table.get(resolved).unwrap().resolution(),
        LabelResolution::Resolved(_)
    ));
    let LabelResolution::Ambiguous(ambiguous_ref) = table.get(ambiguous).unwrap().resolution()
    else {
        panic!("expected ambiguous label ref");
    };
    assert_eq!(
        ambiguous_ref
            .candidates()
            .iter()
            .map(|candidate| candidate.origin.as_str())
            .collect::<Vec<_>>(),
        vec!["pkg::main::A1", "pkg::main::B1"]
    );
    assert!(matches!(
        table.get(unresolved).unwrap().resolution(),
        LabelResolution::Unresolved(_)
    ));
}

#[test]
fn resolved_imports_round_trip_and_project_canonical_modules() {
    let source_id = source_id(5);
    let first = module_id("pkg", "zeta");
    let second = module_id("pkg", "alpha");
    let import_origin = origin(source_id, module_id("pkg", "main"));
    let mut imports = ResolvedImports::new();

    let first_id = imports.push_import(ResolvedImport::new(
        ResolvedNodeId::new(0),
        range(source_id, 0, 10),
        "import zeta;",
        Some("z".to_owned()),
        ImportResolution::Resolved(first.clone()),
        import_origin.clone(),
    ));
    let second_id = imports.push_import(ResolvedImport::new(
        ResolvedNodeId::new(1),
        range(source_id, 11, 20),
        "import alpha;",
        None,
        ImportResolution::Resolved(second.clone()),
        import_origin.clone(),
    ));
    let unresolved_id = imports.push_import(ResolvedImport::new(
        ResolvedNodeId::new(2),
        range(source_id, 21, 30),
        "import missing;",
        None,
        ImportResolution::Unresolved(UnresolvedImport::new(
            "missing",
            range(source_id, 21, 30),
            ImportFailureClass::ModuleNotFound,
        )),
        import_origin.clone(),
    ));
    let ambiguous_id = imports.push_import(ResolvedImport::new(
        ResolvedNodeId::new(3),
        range(source_id, 31, 40),
        "import ambiguous;",
        None,
        ImportResolution::Ambiguous(AmbiguousImport::new(vec![
            module_id("pkg", "omega"),
            module_id("pkg", "beta"),
        ])),
        import_origin.clone(),
    ));
    let export_id = imports.push_export(ResolvedExport::new(
        ResolvedNodeId::new(4),
        range(source_id, 41, 50),
        "export zeta;",
        ExportTarget::Module(first.clone()),
        import_origin,
    ));

    assert!(matches!(
        imports.import(first_id).unwrap().resolution(),
        ImportResolution::Resolved(module) if module == &first
    ));
    assert!(matches!(
        imports.import(second_id).unwrap().resolution(),
        ImportResolution::Resolved(module) if module == &second
    ));
    assert!(matches!(
        imports.import(unresolved_id).unwrap().resolution(),
        ImportResolution::Unresolved(_)
    ));
    let ImportResolution::Ambiguous(ambiguous) = imports.import(ambiguous_id).unwrap().resolution()
    else {
        panic!("expected ambiguous import");
    };
    assert_eq!(
        ambiguous
            .candidates()
            .iter()
            .map(|module| module.path().as_str())
            .collect::<Vec<_>>(),
        vec!["beta", "omega"]
    );
    assert!(matches!(
        imports.export(export_id).unwrap().target(),
        ExportTarget::Module(module) if module == &first
    ));
    assert_eq!(
        imports
            .canonical_import_modules()
            .iter()
            .map(|module| module.path().as_str())
            .collect::<Vec<_>>(),
        vec!["alpha", "zeta"]
    );
}

#[test]
fn node_resolution_state_and_reference_key_are_preserved() {
    let source_id = source_id(6);
    let module = module_id("pkg", "main");
    let origin = origin(source_id, module).recovered();
    let node = ResolvedNode::new(SurfaceNodeKind::TermReference, Vec::new(), origin)
        .with_resolution(NodeResolutionState::Unresolved)
        .with_reference_key(NodeReferenceKey::Name(NameRefId::new(3)));

    assert_eq!(node.recovery(), RecoveryState::Recovered);
    assert_eq!(node.resolution(), NodeResolutionState::Unresolved);
    assert_eq!(
        node.reference_key(),
        Some(NodeReferenceKey::Name(NameRefId::new(3)))
    );
    assert!(node.origin().is_recovered());
}

#[test]
fn resolved_ast_validates_node_keys_and_preserves_traversal_states() {
    let source_id = source_id(7);
    let module = module_id("pkg", "main");
    let normal_origin = origin(source_id, module.clone());
    let recovered_origin = origin(source_id, module.clone()).recovered();
    let mut name_refs = NameRefTable::new();
    let mut label_refs = LabelRefTable::new();
    let mut imports = ResolvedImports::new();
    let name_id = name_refs.insert(NameRefEntry::new(
        ReferenceSite::new(ResolvedNodeId::new(0), range(source_id, 0, 1), "Missing"),
        NameResolution::Unresolved(UnresolvedNameRef::new(
            "Missing",
            range(source_id, 0, 1),
            NameLookupClass::Symbol,
        )),
        recovered_origin
            .clone()
            .with_import_edge(ResolvedImportId::new(0)),
    ));
    let label_id = label_refs.insert(LabelRefEntry::new(
        ReferenceSite::new(ResolvedNodeId::new(1), range(source_id, 1, 2), "A1"),
        LabelResolution::Unresolved(UnresolvedLabelRef::new(
            "A1",
            range(source_id, 1, 2),
            LabelExpectation::Theorem,
        )),
        normal_origin.clone(),
    ));
    let import_id = imports.push_import(ResolvedImport::new(
        ResolvedNodeId::new(2),
        range(source_id, 2, 3),
        "import dep;",
        None,
        ImportResolution::Resolved(module_id("pkg", "dep")),
        normal_origin.clone(),
    ));
    let imported_name_id = name_refs.insert(NameRefEntry::new(
        ReferenceSite::new(ResolvedNodeId::new(0), range(source_id, 4, 5), "Imported"),
        NameResolution::Resolved(
            SymbolRef::new(
                symbol_id(module_id("pkg", "dep"), "pred/0", "pkg::dep::pred/0"),
                range(source_id, 4, 5),
            )
            .with_import(import_id),
        ),
        normal_origin.clone().with_import_edge(import_id),
    ));
    let export_id = imports.push_export(ResolvedExport::new(
        ResolvedNodeId::new(3),
        range(source_id, 3, 4),
        "export Missing;",
        ExportTarget::Unresolved(UnresolvedExport::new(
            "Missing",
            range(source_id, 3, 4),
            ExportFailureClass::TargetNotFound,
        )),
        recovered_origin.clone(),
    ));

    let mut builder = ResolvedArenaBuilder::new();
    let unresolved = builder
        .push(
            ResolvedNode::new(SurfaceNodeKind::TermReference, Vec::new(), recovered_origin)
                .with_recovery(RecoveryState::Recovered)
                .with_resolution(NodeResolutionState::Unresolved)
                .with_reference_key(NodeReferenceKey::Name(name_id)),
        )
        .unwrap();
    let ambiguous = builder
        .push(
            ResolvedNode::new(
                SurfaceNodeKind::Reference,
                Vec::new(),
                normal_origin.clone(),
            )
            .with_resolution(NodeResolutionState::Ambiguous)
            .with_reference_key(NodeReferenceKey::Label(label_id)),
        )
        .unwrap();
    let deferred = builder
        .push(
            ResolvedNode::new(
                SurfaceNodeKind::ImportItem,
                Vec::new(),
                normal_origin.clone(),
            )
            .with_resolution(NodeResolutionState::Deferred)
            .with_reference_key(NodeReferenceKey::Import(import_id)),
        )
        .unwrap();
    let resolved = builder
        .push(
            ResolvedNode::new(
                SurfaceNodeKind::ExportItem,
                Vec::new(),
                normal_origin.clone(),
            )
            .with_resolution(NodeResolutionState::Resolved)
            .with_reference_key(NodeReferenceKey::Export(export_id)),
        )
        .unwrap();
    let root = builder
        .push(ResolvedNode::new(
            SurfaceNodeKind::CompilationUnit,
            vec![unresolved, ambiguous, deferred, resolved],
            normal_origin,
        ))
        .unwrap();
    let arena = builder.finish(root).unwrap();
    let ast =
        ResolvedAst::try_new(source_id, module, arena, name_refs, label_refs, imports).unwrap();

    assert_eq!(ast.nodes().root(), root);
    assert_eq!(
        ast.nodes()
            .iter()
            .take(4)
            .map(|(_, node)| node.resolution())
            .collect::<Vec<_>>(),
        vec![
            NodeResolutionState::Unresolved,
            NodeResolutionState::Ambiguous,
            NodeResolutionState::Deferred,
            NodeResolutionState::Resolved,
        ]
    );
    assert_eq!(
        ast.nodes()
            .iter()
            .take(4)
            .map(|(_, node)| node.reference_key())
            .collect::<Vec<_>>(),
        vec![
            Some(NodeReferenceKey::Name(name_id)),
            Some(NodeReferenceKey::Label(label_id)),
            Some(NodeReferenceKey::Import(import_id)),
            Some(NodeReferenceKey::Export(export_id)),
        ]
    );
    assert_eq!(
        ast.nodes().node(root).unwrap().origin().anchor(),
        &SourceAnchor::Range(range(source_id, 0, 1))
    );
    assert_eq!(
        ast.nodes().node(root).unwrap().origin().structural_path(),
        &[0]
    );
    let name_entry = ast.name_refs().get(name_id).unwrap();
    assert_eq!(name_entry.recovery(), RecoveryState::Recovered);
    assert_eq!(
        name_entry.origin().anchor(),
        &SourceAnchor::Range(range(source_id, 0, 1))
    );
    assert_eq!(name_entry.origin().structural_path(), &[0]);
    assert_eq!(
        name_entry.origin().import_edge(),
        Some(ResolvedImportId::new(0))
    );
    assert_eq!(
        ast.imports().import(import_id).unwrap().origin().anchor(),
        &SourceAnchor::Range(range(source_id, 0, 1))
    );
    let imported_name = ast.name_refs().get(imported_name_id).unwrap();
    assert!(matches!(
        imported_name.resolution(),
        NameResolution::Resolved(symbol) if symbol.import() == Some(import_id)
    ));
    assert_eq!(imported_name.origin().import_edge(), Some(import_id));
    let export = ast.imports().export(export_id).unwrap();
    assert_eq!(export.recovery(), RecoveryState::Recovered);
    assert_eq!(
        export.origin().anchor(),
        &SourceAnchor::Range(range(source_id, 0, 1))
    );
    assert_eq!(export.origin().structural_path(), &[0]);
    assert!(matches!(
        export.target(),
        ExportTarget::Unresolved(unresolved)
            if unresolved.class() == ExportFailureClass::TargetNotFound
                && unresolved.spelling() == "Missing"
    ));
}

#[test]
fn resolved_ast_rejects_stale_keys_and_mismatched_modules() {
    let (primary_source_id, other_source_id) = source_id_pair(8);
    let module = module_id("pkg", "main");
    let other_module = module_id("pkg", "other");
    let module_origin = origin(primary_source_id, module.clone());
    let mut builder = ResolvedArenaBuilder::new();
    let root = builder
        .push(
            ResolvedNode::new(SurfaceNodeKind::CompilationUnit, Vec::new(), module_origin)
                .with_reference_key(NodeReferenceKey::Name(NameRefId::new(99))),
        )
        .unwrap();
    let arena = builder.finish(root).unwrap();

    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module.clone(),
            arena,
            NameRefTable::new(),
            LabelRefTable::new(),
            ResolvedImports::new(),
        ),
        Err(ResolvedAstError::InvalidNodeReferenceKey { .. })
    ));

    let mismatched_arena = ResolvedArena::try_new(
        root,
        vec![ResolvedNode::new(
            SurfaceNodeKind::CompilationUnit,
            Vec::new(),
            origin(primary_source_id, other_module),
        )],
    )
    .unwrap();
    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module,
            mismatched_arena,
            NameRefTable::new(),
            LabelRefTable::new(),
            ResolvedImports::new(),
        ),
        Err(ResolvedAstError::NodeModuleMismatch { .. })
    ));

    let wrong_source_arena = ResolvedArena::try_new(
        root,
        vec![ResolvedNode::new(
            SurfaceNodeKind::CompilationUnit,
            Vec::new(),
            origin(other_source_id, module_id("pkg", "main")),
        )],
    )
    .unwrap();
    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module_id("pkg", "main"),
            wrong_source_arena,
            NameRefTable::new(),
            LabelRefTable::new(),
            ResolvedImports::new(),
        ),
        Err(ResolvedAstError::OriginSourceMismatch)
    ));

    let mut stale_import_edge_refs = NameRefTable::new();
    stale_import_edge_refs.insert(NameRefEntry::new(
        ReferenceSite::new(root, range(primary_source_id, 0, 1), "VisibleByImport"),
        NameResolution::Unresolved(UnresolvedNameRef::new(
            "VisibleByImport",
            range(primary_source_id, 0, 1),
            NameLookupClass::Symbol,
        )),
        origin(primary_source_id, module_id("pkg", "main"))
            .with_import_edge(ResolvedImportId::new(99)),
    ));
    let stale_import_edge_arena = ResolvedArena::try_new(
        root,
        vec![ResolvedNode::new(
            SurfaceNodeKind::CompilationUnit,
            Vec::new(),
            origin(primary_source_id, module_id("pkg", "main")),
        )],
    )
    .unwrap();
    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module_id("pkg", "main"),
            stale_import_edge_arena,
            stale_import_edge_refs,
            LabelRefTable::new(),
            ResolvedImports::new(),
        ),
        Err(ResolvedAstError::InvalidImportEdge { .. })
    ));

    let wrong_anchor_arena = ResolvedArena::try_new(
        root,
        vec![ResolvedNode::new(
            SurfaceNodeKind::CompilationUnit,
            Vec::new(),
            SemanticOrigin::new(
                primary_source_id,
                module_id("pkg", "main"),
                SourceAnchor::Range(range(other_source_id, 0, 1)),
                vec![0],
            ),
        )],
    )
    .unwrap();
    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module_id("pkg", "main"),
            wrong_anchor_arena,
            NameRefTable::new(),
            LabelRefTable::new(),
            ResolvedImports::new(),
        ),
        Err(ResolvedAstError::PayloadSourceMismatch)
    ));

    let mut wrong_site_refs = NameRefTable::new();
    wrong_site_refs.insert(NameRefEntry::new(
        ReferenceSite::new(root, range(other_source_id, 0, 1), "WrongSource"),
        NameResolution::Unresolved(UnresolvedNameRef::new(
            "WrongSource",
            range(primary_source_id, 0, 1),
            NameLookupClass::Symbol,
        )),
        origin(primary_source_id, module_id("pkg", "main")),
    ));
    let wrong_site_arena = ResolvedArena::try_new(
        root,
        vec![ResolvedNode::new(
            SurfaceNodeKind::CompilationUnit,
            Vec::new(),
            origin(primary_source_id, module_id("pkg", "main")),
        )],
    )
    .unwrap();
    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module_id("pkg", "main"),
            wrong_site_arena,
            wrong_site_refs,
            LabelRefTable::new(),
            ResolvedImports::new(),
        ),
        Err(ResolvedAstError::PayloadSourceMismatch)
    ));

    let mut stale_symbol_import_refs = NameRefTable::new();
    stale_symbol_import_refs.insert(NameRefEntry::new(
        ReferenceSite::new(root, range(primary_source_id, 0, 1), "P"),
        NameResolution::Resolved(
            SymbolRef::new(
                symbol_id(module_id("pkg", "main"), "pred/0", "pkg::main::pred/0"),
                range(primary_source_id, 0, 1),
            )
            .with_import(ResolvedImportId::new(99)),
        ),
        origin(primary_source_id, module_id("pkg", "main")),
    ));
    let stale_symbol_import_arena = ResolvedArena::try_new(
        root,
        vec![ResolvedNode::new(
            SurfaceNodeKind::CompilationUnit,
            Vec::new(),
            origin(primary_source_id, module_id("pkg", "main")),
        )],
    )
    .unwrap();
    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module_id("pkg", "main"),
            stale_symbol_import_arena,
            stale_symbol_import_refs,
            LabelRefTable::new(),
            ResolvedImports::new(),
        ),
        Err(ResolvedAstError::InvalidImportEdge { .. })
    ));

    let mut missing_base_refs = NameRefTable::new();
    missing_base_refs.insert(NameRefEntry::new(
        ReferenceSite::new(root, range(primary_source_id, 0, 1), "x.y"),
        NameResolution::DeferredSelector(DeferredSelectorRef::new(
            ResolvedNodeId::new(99),
            "y",
            range(primary_source_id, 0, 1),
        )),
        origin(primary_source_id, module_id("pkg", "main")),
    ));
    let missing_base_arena = ResolvedArena::try_new(
        root,
        vec![ResolvedNode::new(
            SurfaceNodeKind::CompilationUnit,
            Vec::new(),
            origin(primary_source_id, module_id("pkg", "main")),
        )],
    )
    .unwrap();
    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module_id("pkg", "main"),
            missing_base_arena,
            missing_base_refs,
            LabelRefTable::new(),
            ResolvedImports::new(),
        ),
        Err(ResolvedAstError::InvalidDeferredSelectorBase { .. })
    ));

    let mut mismatched_site_refs = NameRefTable::new();
    let mismatched_name_id = mismatched_site_refs.insert(NameRefEntry::new(
        ReferenceSite::new(
            ResolvedNodeId::new(0),
            range(primary_source_id, 0, 1),
            "OwnedByChild",
        ),
        NameResolution::Unresolved(UnresolvedNameRef::new(
            "OwnedByChild",
            range(primary_source_id, 0, 1),
            NameLookupClass::Symbol,
        )),
        origin(primary_source_id, module_id("pkg", "main")),
    ));
    let child = ResolvedNode::new(
        SurfaceNodeKind::TermReference,
        Vec::new(),
        origin(primary_source_id, module_id("pkg", "main")),
    );
    let keyed_parent = ResolvedNode::new(
        SurfaceNodeKind::CompilationUnit,
        vec![ResolvedNodeId::new(0)],
        origin(primary_source_id, module_id("pkg", "main")),
    )
    .with_reference_key(NodeReferenceKey::Name(mismatched_name_id));
    let mismatched_site_arena =
        ResolvedArena::try_new(ResolvedNodeId::new(1), vec![child, keyed_parent]).unwrap();
    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module_id("pkg", "main"),
            mismatched_site_arena,
            mismatched_site_refs,
            LabelRefTable::new(),
            ResolvedImports::new(),
        ),
        Err(ResolvedAstError::NodeReferenceSiteMismatch { .. })
    ));

    let mut mismatched_label_refs = LabelRefTable::new();
    let mismatched_label_id = mismatched_label_refs.insert(LabelRefEntry::new(
        ReferenceSite::new(
            ResolvedNodeId::new(0),
            range(primary_source_id, 0, 1),
            "OwnedByChildLabel",
        ),
        LabelResolution::Unresolved(UnresolvedLabelRef::new(
            "OwnedByChildLabel",
            range(primary_source_id, 0, 1),
            LabelExpectation::Theorem,
        )),
        origin(primary_source_id, module_id("pkg", "main")),
    ));
    let label_child = ResolvedNode::new(
        SurfaceNodeKind::AnnotationLabel,
        Vec::new(),
        origin(primary_source_id, module_id("pkg", "main")),
    );
    let keyed_label_parent = ResolvedNode::new(
        SurfaceNodeKind::CompilationUnit,
        vec![ResolvedNodeId::new(0)],
        origin(primary_source_id, module_id("pkg", "main")),
    )
    .with_reference_key(NodeReferenceKey::Label(mismatched_label_id));
    let mismatched_label_site_arena = ResolvedArena::try_new(
        ResolvedNodeId::new(1),
        vec![label_child, keyed_label_parent],
    )
    .unwrap();
    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module_id("pkg", "main"),
            mismatched_label_site_arena,
            NameRefTable::new(),
            mismatched_label_refs,
            ResolvedImports::new(),
        ),
        Err(ResolvedAstError::NodeReferenceSiteMismatch { .. })
    ));

    let mut import_owner_imports = ResolvedImports::new();
    let owner_mismatched_import_id = import_owner_imports.push_import(ResolvedImport::new(
        ResolvedNodeId::new(0),
        range(primary_source_id, 0, 1),
        "import dep;",
        None,
        ImportResolution::Resolved(module_id("pkg", "dep")),
        origin(primary_source_id, module_id("pkg", "main")),
    ));
    let import_owner_child = ResolvedNode::new(
        SurfaceNodeKind::ImportItem,
        Vec::new(),
        origin(primary_source_id, module_id("pkg", "main")),
    );
    let import_keyed_parent = ResolvedNode::new(
        SurfaceNodeKind::CompilationUnit,
        vec![ResolvedNodeId::new(0)],
        origin(primary_source_id, module_id("pkg", "main")),
    )
    .with_reference_key(NodeReferenceKey::Import(owner_mismatched_import_id));
    let import_owner_arena = ResolvedArena::try_new(
        ResolvedNodeId::new(1),
        vec![import_owner_child, import_keyed_parent],
    )
    .unwrap();
    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module_id("pkg", "main"),
            import_owner_arena,
            NameRefTable::new(),
            LabelRefTable::new(),
            import_owner_imports,
        ),
        Err(ResolvedAstError::NodeReferenceSiteMismatch { .. })
    ));

    let mut export_owner_imports = ResolvedImports::new();
    let owner_mismatched_export_id = export_owner_imports.push_export(ResolvedExport::new(
        ResolvedNodeId::new(0),
        range(primary_source_id, 0, 1),
        "export dep;",
        ExportTarget::Module(module_id("pkg", "dep")),
        origin(primary_source_id, module_id("pkg", "main")),
    ));
    let export_owner_child = ResolvedNode::new(
        SurfaceNodeKind::ExportItem,
        Vec::new(),
        origin(primary_source_id, module_id("pkg", "main")),
    );
    let export_keyed_parent = ResolvedNode::new(
        SurfaceNodeKind::CompilationUnit,
        vec![ResolvedNodeId::new(0)],
        origin(primary_source_id, module_id("pkg", "main")),
    )
    .with_reference_key(NodeReferenceKey::Export(owner_mismatched_export_id));
    let export_owner_arena = ResolvedArena::try_new(
        ResolvedNodeId::new(1),
        vec![export_owner_child, export_keyed_parent],
    )
    .unwrap();
    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module_id("pkg", "main"),
            export_owner_arena,
            NameRefTable::new(),
            LabelRefTable::new(),
            export_owner_imports,
        ),
        Err(ResolvedAstError::NodeReferenceSiteMismatch { .. })
    ));

    let mut provenance_imports = ResolvedImports::new();
    let first_import = provenance_imports.push_import(ResolvedImport::new(
        ResolvedNodeId::new(1),
        range(primary_source_id, 1, 2),
        "import one;",
        None,
        ImportResolution::Resolved(module_id("pkg", "one")),
        origin(primary_source_id, module_id("pkg", "main")),
    ));
    let second_import = provenance_imports.push_import(ResolvedImport::new(
        ResolvedNodeId::new(2),
        range(primary_source_id, 2, 3),
        "import two;",
        None,
        ImportResolution::Resolved(module_id("pkg", "two")),
        origin(primary_source_id, module_id("pkg", "main")),
    ));
    let mut provenance_refs = NameRefTable::new();
    let provenance_name_id = provenance_refs.insert(NameRefEntry::new(
        ReferenceSite::new(ResolvedNodeId::new(0), range(primary_source_id, 0, 1), "P"),
        NameResolution::Resolved(
            SymbolRef::new(
                symbol_id(module_id("pkg", "dep"), "pred/0", "pkg::dep::pred/0"),
                range(primary_source_id, 0, 1),
            )
            .with_import(second_import),
        ),
        origin(primary_source_id, module_id("pkg", "main")).with_import_edge(first_import),
    ));
    let provenance_arena = ResolvedArena::try_new(
        ResolvedNodeId::new(3),
        vec![
            ResolvedNode::new(
                SurfaceNodeKind::Reference,
                Vec::new(),
                origin(primary_source_id, module_id("pkg", "main")),
            )
            .with_reference_key(NodeReferenceKey::Name(provenance_name_id)),
            ResolvedNode::new(
                SurfaceNodeKind::ImportItem,
                Vec::new(),
                origin(primary_source_id, module_id("pkg", "main")),
            ),
            ResolvedNode::new(
                SurfaceNodeKind::ImportItem,
                Vec::new(),
                origin(primary_source_id, module_id("pkg", "main")),
            ),
            ResolvedNode::new(
                SurfaceNodeKind::CompilationUnit,
                vec![
                    ResolvedNodeId::new(0),
                    ResolvedNodeId::new(1),
                    ResolvedNodeId::new(2),
                ],
                origin(primary_source_id, module_id("pkg", "main")),
            ),
        ],
    )
    .unwrap();
    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module_id("pkg", "main"),
            provenance_arena.clone(),
            provenance_refs,
            LabelRefTable::new(),
            provenance_imports.clone(),
        ),
        Err(ResolvedAstError::ImportProvenanceMismatch { .. })
    ));

    let mut symbol_only_provenance_refs = NameRefTable::new();
    symbol_only_provenance_refs.insert(NameRefEntry::new(
        ReferenceSite::new(ResolvedNodeId::new(0), range(primary_source_id, 0, 1), "P"),
        NameResolution::Resolved(
            SymbolRef::new(
                symbol_id(module_id("pkg", "dep"), "pred/0", "pkg::dep::pred/0"),
                range(primary_source_id, 0, 1),
            )
            .with_import(first_import),
        ),
        origin(primary_source_id, module_id("pkg", "main")),
    ));
    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module_id("pkg", "main"),
            provenance_arena.clone(),
            symbol_only_provenance_refs,
            LabelRefTable::new(),
            provenance_imports.clone(),
        ),
        Err(ResolvedAstError::ImportProvenanceMismatch { .. })
    ));

    let mut origin_only_provenance_refs = NameRefTable::new();
    origin_only_provenance_refs.insert(NameRefEntry::new(
        ReferenceSite::new(ResolvedNodeId::new(0), range(primary_source_id, 0, 1), "P"),
        NameResolution::Resolved(SymbolRef::new(
            symbol_id(module_id("pkg", "dep"), "pred/0", "pkg::dep::pred/0"),
            range(primary_source_id, 0, 1),
        )),
        origin(primary_source_id, module_id("pkg", "main")).with_import_edge(first_import),
    ));
    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module_id("pkg", "main"),
            provenance_arena,
            origin_only_provenance_refs,
            LabelRefTable::new(),
            provenance_imports,
        ),
        Err(ResolvedAstError::ImportProvenanceMismatch { .. })
    ));

    let stale_owner_arena = ResolvedArena::try_new(
        root,
        vec![ResolvedNode::new(
            SurfaceNodeKind::CompilationUnit,
            Vec::new(),
            origin(primary_source_id, module_id("pkg", "main")),
        )],
    )
    .unwrap();
    let mut stale_import_owner_imports = ResolvedImports::new();
    stale_import_owner_imports.push_import(ResolvedImport::new(
        ResolvedNodeId::new(99),
        range(primary_source_id, 0, 1),
        "import stale;",
        None,
        ImportResolution::Resolved(module_id("pkg", "stale")),
        origin(primary_source_id, module_id("pkg", "main")),
    ));
    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module_id("pkg", "main"),
            stale_owner_arena.clone(),
            NameRefTable::new(),
            LabelRefTable::new(),
            stale_import_owner_imports,
        ),
        Err(ResolvedAstError::InvalidDirectiveOwner { .. })
    ));

    let mut stale_export_owner_imports = ResolvedImports::new();
    stale_export_owner_imports.push_export(ResolvedExport::new(
        ResolvedNodeId::new(99),
        range(primary_source_id, 0, 1),
        "export stale;",
        ExportTarget::Module(module_id("pkg", "stale")),
        origin(primary_source_id, module_id("pkg", "main")),
    ));
    assert!(matches!(
        ResolvedAst::try_new(
            primary_source_id,
            module_id("pkg", "main"),
            stale_owner_arena,
            NameRefTable::new(),
            LabelRefTable::new(),
            stale_export_owner_imports,
        ),
        Err(ResolvedAstError::InvalidDirectiveOwner { .. })
    ));
}

#[test]
fn node_reference_keys_are_stable_for_equivalent_builds() {
    let first = reference_key_snapshot(source_id(11), module_id("pkg", "main"));
    let second = reference_key_snapshot(source_id(11), module_id("pkg", "main"));

    assert_eq!(first, second);
    assert_eq!(
        first,
        vec![
            Some(NodeReferenceKey::Name(NameRefId::new(0))),
            Some(NodeReferenceKey::Label(LabelRefId::new(0))),
            Some(NodeReferenceKey::Import(ResolvedImportId::new(0))),
            Some(NodeReferenceKey::Export(ResolvedExportId::new(0))),
        ]
    );
}

#[test]
fn table_and_import_export_iteration_is_stable() {
    let source_id = source_id(9);
    let module = module_id("pkg", "main");
    let origin = origin(source_id, module);
    let node = ResolvedNodeId::new(0);
    let mut name_refs = NameRefTable::new();
    let first_name = name_refs.insert(NameRefEntry::new(
        ReferenceSite::new(node, range(source_id, 0, 1), "A"),
        NameResolution::Unresolved(UnresolvedNameRef::new(
            "A",
            range(source_id, 0, 1),
            NameLookupClass::Symbol,
        )),
        origin.clone(),
    ));
    let second_name = name_refs.insert(NameRefEntry::new(
        ReferenceSite::new(node, range(source_id, 1, 2), "B"),
        NameResolution::Unresolved(UnresolvedNameRef::new(
            "B",
            range(source_id, 1, 2),
            NameLookupClass::Symbol,
        )),
        origin.clone(),
    ));

    let mut label_refs = LabelRefTable::new();
    let first_label = label_refs.insert(LabelRefEntry::new(
        ReferenceSite::new(node, range(source_id, 2, 3), "L1"),
        LabelResolution::Unresolved(UnresolvedLabelRef::new(
            "L1",
            range(source_id, 2, 3),
            LabelExpectation::Theorem,
        )),
        origin.clone(),
    ));
    let second_label = label_refs.insert(LabelRefEntry::new(
        ReferenceSite::new(node, range(source_id, 3, 4), "L2"),
        LabelResolution::Unresolved(UnresolvedLabelRef::new(
            "L2",
            range(source_id, 3, 4),
            LabelExpectation::Theorem,
        )),
        origin.clone(),
    ));

    let mut imports = ResolvedImports::new();
    let first_import = imports.push_import(ResolvedImport::new(
        ResolvedNodeId::new(0),
        range(source_id, 4, 5),
        "import a;",
        None,
        ImportResolution::Resolved(module_id("pkg", "a")),
        origin.clone(),
    ));
    let second_import = imports.push_import(ResolvedImport::new(
        ResolvedNodeId::new(1),
        range(source_id, 5, 6),
        "import b;",
        None,
        ImportResolution::Resolved(module_id("pkg", "b")),
        origin.clone(),
    ));
    let first_export = imports.push_export(ResolvedExport::new(
        ResolvedNodeId::new(2),
        range(source_id, 6, 7),
        "export a;",
        ExportTarget::Module(module_id("pkg", "a")),
        origin.clone(),
    ));
    let second_export = imports.push_export(ResolvedExport::new(
        ResolvedNodeId::new(3),
        range(source_id, 7, 8),
        "export b;",
        ExportTarget::Module(module_id("pkg", "b")),
        origin,
    ));

    assert_eq!(
        name_refs.iter().map(|(id, _)| id).collect::<Vec<_>>(),
        vec![first_name, second_name]
    );
    assert_eq!(
        label_refs.iter().map(|(id, _)| id).collect::<Vec<_>>(),
        vec![first_label, second_label]
    );
    assert_eq!(
        imports.imports().map(|(id, _)| id).collect::<Vec<_>>(),
        vec![first_import, second_import]
    );
    assert_eq!(
        imports.exports().map(|(id, _)| id).collect::<Vec<_>>(),
        vec![first_export, second_export]
    );
}

#[test]
fn resolved_ast_snapshot_text_is_stable_and_covers_tables() {
    let first = debug_snapshot_ast_fixture(source_id(13)).snapshot_text();
    let second = debug_snapshot_ast_fixture(source_id(13)).snapshot_text();

    assert_eq!(first, second);
    assert!(first.starts_with("resolved-ast-debug-v1\nmodule: pkg::main\n"));
    assert!(!first.contains("SourceId"));
    assert!(!first.contains('\r'));
    assert_ordered_fragments(
        &first,
        &[
            "module: pkg::main\n",
            "root:",
            "nodes:\n",
            "name_refs:\n",
            "label_refs:\n",
            "imports:\n",
            "exports:\n",
            "canonical_import_modules:\n",
        ],
    );
    for expected in [
        "root: node#15",
        "node#0 kind=TermReference children=[] recovery=normal resolution=resolved ref=name#0",
        "node#1 kind=TermReference children=[] recovery=normal resolution=resolved ref=name#1",
        "node#2 kind=SelectorAccess children=[] recovery=normal resolution=deferred ref=name#2",
        "node#3 kind=Reference children=[] recovery=normal resolution=ambiguous ref=name#3",
        "node#4 kind=Reference children=[] recovery=recovered resolution=unresolved ref=name#4",
        "node#5 kind=AnnotationLabel children=[] recovery=normal resolution=resolved ref=label#0",
        "node#8 kind=ImportItem children=[] recovery=normal resolution=resolved ref=import#0",
        "node#14 kind=ExportItem children=[] recovery=recovered resolution=unresolved ref=export#3",
        "node#15 kind=CompilationUnit children=[node#0, node#1, node#2, node#3, node#4, node#5, node#6, node#7, node#8, node#9, node#10, node#11, node#12, node#13, node#14] recovery=normal resolution=not_applicable ref=<none>",
        "resolution=resolved symbol={fqn=\"pkg::main::pred/0\" module=pkg::main local=\"pred/0\"}",
        "resolution=builtin builtin=\"builtin::equals\"",
        "resolution=deferred_selector base=node#0 member=\"field\"",
        "resolution=ambiguous spelling=\"P\" range=6..7 candidates=[",
        "resolution=unresolved spelling=\"Missing\" lookup=symbol",
        "resolution=resolved origin=\"pkg::main::T1\" kind=theorem",
        "resolution=ambiguous spelling=\"A1\" range=11..12 candidates=[",
        "resolution=unresolved spelling=\"A2\" expectation=proof_step",
        "resolution=resolved module=pkg::dep",
        "resolution=ambiguous candidates=[pkg::alpha, pkg::zeta]",
        "resolution=unresolved spelling=\"missing\" class=module_not_found",
        "target=module=pkg::dep",
        "target=import_alias alias=\"D\" module=pkg::dep",
        "target=symbol={fqn=\"pkg::main::pred/0\" module=pkg::main local=\"pred/0\"}",
        "target=unresolved spelling=\"Missing\" class=target_not_found",
        "canonical_import_modules:\n  module=pkg::dep\n",
    ] {
        assert!(
            first.contains(expected),
            "snapshot should contain fixture fragment: {expected}\n{first}"
        );
    }
}

#[test]
fn resolved_ast_snapshot_text_covers_payload_escaping_and_non_range_anchors() {
    let source_id = source_id(14);
    let module = module_id("pkg", "main");
    let generated = GeneratedSpanOrigin::new(
        GeneratedSpanAnchor::Point {
            source_id,
            offset: 9,
        },
        "/tmp/private/generated-source",
    )
    .unwrap();
    let ast = tiny_snapshot_ast(
        source_id,
        module.clone(),
        SurfaceNodeKind::Token(mizar_syntax::SurfaceToken::new(
            SurfaceTokenKind::StringLiteral,
            "line\n\"\\value",
        )),
        SemanticOrigin::new(
            source_id,
            module.clone(),
            SourceAnchor::Generated(generated),
            vec![7],
        ),
    );
    let snapshot = ast.snapshot_text();
    assert!(snapshot.contains("Token kind=StringLiteral text=\"line\\n\\\"\\\\value\""));
    assert!(snapshot.contains("anchor=generated(point(9), reason=present)"));
    assert!(!snapshot.contains("/tmp/private"));
    assert!(!snapshot.contains("SourceId"));

    let point_ast = tiny_snapshot_ast(
        source_id,
        module.clone(),
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction),
        SemanticOrigin::new(
            source_id,
            module.clone(),
            SourceAnchor::Point {
                source_id,
                offset: 11,
            },
            vec![8],
        ),
    );
    assert!(
        point_ast
            .snapshot_text()
            .contains("FormulaConstant constant=Contradiction")
    );
    assert!(point_ast.snapshot_text().contains("anchor=point(11)"));

    let recovery_ast = tiny_snapshot_ast(
        source_id,
        module,
        SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm),
        origin(source_id, module_id("pkg", "main")).recovered(),
    );
    assert!(
        recovery_ast
            .snapshot_text()
            .contains("ErrorRecovery kind=MissingTerm")
    );
}

fn tiny_snapshot_ast(
    source_id: SourceId,
    module: ModuleId,
    kind: SurfaceNodeKind,
    origin: SemanticOrigin,
) -> ResolvedAst {
    let arena = ResolvedArena::try_new(
        ResolvedNodeId::new(0),
        vec![ResolvedNode::new(kind, Vec::new(), origin)],
    )
    .unwrap();
    ResolvedAst::try_new(
        source_id,
        module,
        arena,
        NameRefTable::new(),
        LabelRefTable::new(),
        ResolvedImports::new(),
    )
    .unwrap()
}

fn assert_ordered_fragments(snapshot: &str, fragments: &[&str]) {
    let mut cursor = 0;
    for fragment in fragments {
        let Some(offset) = snapshot[cursor..].find(fragment) else {
            panic!("missing ordered fragment: {fragment}\n{snapshot}");
        };
        cursor += offset + fragment.len();
    }
}

fn debug_snapshot_ast_fixture(source_id: SourceId) -> ResolvedAst {
    let module = module_id("pkg", "main");
    let normal_origin = origin(source_id, module.clone());
    let recovered_origin = origin(source_id, module.clone()).recovered();
    let symbol = symbol_id(module.clone(), "pred/0", "pkg::main::pred/0");

    let mut name_refs = NameRefTable::new();
    let resolved_name = name_refs.insert(NameRefEntry::new(
        ReferenceSite::new(ResolvedNodeId::new(0), range(source_id, 0, 1), "P"),
        NameResolution::Resolved(
            SymbolRef::new(symbol.clone(), range(source_id, 0, 1)).with_spelling("P"),
        ),
        normal_origin.clone(),
    ));
    let builtin_name = name_refs.insert(NameRefEntry::new(
        ReferenceSite::new(ResolvedNodeId::new(1), range(source_id, 2, 3), "="),
        NameResolution::ResolvedBuiltin(BuiltinRef::new(
            BuiltinId::new("builtin::equals"),
            range(source_id, 2, 3),
            "=",
        )),
        normal_origin.clone(),
    ));
    let deferred_name = name_refs.insert(NameRefEntry::new(
        ReferenceSite::new(ResolvedNodeId::new(2), range(source_id, 4, 5), "x.field"),
        NameResolution::DeferredSelector(DeferredSelectorRef::new(
            ResolvedNodeId::new(0),
            "field",
            range(source_id, 4, 5),
        )),
        normal_origin.clone(),
    ));
    let ambiguous_name = name_refs.insert(NameRefEntry::new(
        ReferenceSite::new(ResolvedNodeId::new(3), range(source_id, 6, 7), "P"),
        NameResolution::Ambiguous(AmbiguousNameRef::new(
            "P",
            range(source_id, 6, 7),
            vec![
                NameResolutionCandidate::new(
                    symbol_id(module_id("pkg", "other"), "pred/0", "pkg::other::pred/0"),
                    range(source_id, 20, 21),
                ),
                NameResolutionCandidate::new(symbol.clone(), range(source_id, 10, 11)),
            ],
        )),
        normal_origin.clone(),
    ));
    let unresolved_name = name_refs.insert(NameRefEntry::new(
        ReferenceSite::new(ResolvedNodeId::new(4), range(source_id, 8, 9), "Missing"),
        NameResolution::Unresolved(UnresolvedNameRef::new(
            "Missing",
            range(source_id, 8, 9),
            NameLookupClass::Symbol,
        )),
        recovered_origin.clone(),
    ));

    let mut label_refs = LabelRefTable::new();
    let resolved_label = label_refs.insert(LabelRefEntry::new(
        ReferenceSite::new(ResolvedNodeId::new(5), range(source_id, 10, 11), "T1"),
        LabelResolution::Resolved(LabelRef::new(
            LabelOriginPath::new("pkg::main::T1"),
            LabelKind::Theorem,
            range(source_id, 10, 11),
        )),
        normal_origin.clone(),
    ));
    let ambiguous_label = label_refs.insert(LabelRefEntry::new(
        ReferenceSite::new(ResolvedNodeId::new(6), range(source_id, 11, 12), "A1"),
        LabelResolution::Ambiguous(AmbiguousLabelRef::new(
            "A1",
            range(source_id, 11, 12),
            vec![
                LabelCandidate::new(
                    LabelOriginPath::new("pkg::main::B1"),
                    LabelKind::ProofStep,
                    range(source_id, 13, 14),
                ),
                LabelCandidate::new(
                    LabelOriginPath::new("pkg::main::A1"),
                    LabelKind::Definition,
                    range(source_id, 12, 13),
                ),
            ],
        )),
        normal_origin.clone(),
    ));
    let unresolved_label = label_refs.insert(LabelRefEntry::new(
        ReferenceSite::new(ResolvedNodeId::new(7), range(source_id, 14, 15), "A2"),
        LabelResolution::Unresolved(UnresolvedLabelRef::new(
            "A2",
            range(source_id, 14, 15),
            LabelExpectation::ProofStep,
        )),
        recovered_origin.clone(),
    ));

    let mut imports = ResolvedImports::new();
    let resolved_import = imports.push_import(ResolvedImport::new(
        ResolvedNodeId::new(8),
        range(source_id, 16, 17),
        "import dep;",
        Some("D".to_owned()),
        ImportResolution::Resolved(module_id("pkg", "dep")),
        normal_origin.clone(),
    ));
    let ambiguous_import = imports.push_import(ResolvedImport::new(
        ResolvedNodeId::new(9),
        range(source_id, 18, 19),
        "import ambiguous;",
        None,
        ImportResolution::Ambiguous(AmbiguousImport::new(vec![
            module_id("pkg", "zeta"),
            module_id("pkg", "alpha"),
        ])),
        normal_origin.clone(),
    ));
    let unresolved_import = imports.push_import(ResolvedImport::new(
        ResolvedNodeId::new(10),
        range(source_id, 20, 21),
        "import missing;",
        None,
        ImportResolution::Unresolved(UnresolvedImport::new(
            "missing",
            range(source_id, 20, 21),
            ImportFailureClass::ModuleNotFound,
        )),
        recovered_origin.clone(),
    ));
    let module_export = imports.push_export(ResolvedExport::new(
        ResolvedNodeId::new(11),
        range(source_id, 22, 23),
        "export dep;",
        ExportTarget::Module(module_id("pkg", "dep")),
        normal_origin.clone(),
    ));
    let alias_export = imports.push_export(ResolvedExport::new(
        ResolvedNodeId::new(12),
        range(source_id, 24, 25),
        "export D;",
        ExportTarget::ImportAlias {
            alias: "D".to_owned(),
            module: module_id("pkg", "dep"),
        },
        normal_origin.clone(),
    ));
    let symbol_export = imports.push_export(ResolvedExport::new(
        ResolvedNodeId::new(13),
        range(source_id, 26, 27),
        "export P;",
        ExportTarget::Symbol(symbol.clone()),
        normal_origin.clone(),
    ));
    let unresolved_export = imports.push_export(ResolvedExport::new(
        ResolvedNodeId::new(14),
        range(source_id, 28, 29),
        "export Missing;",
        ExportTarget::Unresolved(UnresolvedExport::new(
            "Missing",
            range(source_id, 28, 29),
            ExportFailureClass::TargetNotFound,
        )),
        recovered_origin.clone(),
    ));

    let mut builder = ResolvedArenaBuilder::new();
    for (kind, resolution, key, origin) in [
        (
            SurfaceNodeKind::TermReference,
            NodeResolutionState::Resolved,
            NodeReferenceKey::Name(resolved_name),
            normal_origin.clone(),
        ),
        (
            SurfaceNodeKind::TermReference,
            NodeResolutionState::Resolved,
            NodeReferenceKey::Name(builtin_name),
            normal_origin.clone(),
        ),
        (
            SurfaceNodeKind::SelectorAccess,
            NodeResolutionState::Deferred,
            NodeReferenceKey::Name(deferred_name),
            normal_origin.clone(),
        ),
        (
            SurfaceNodeKind::Reference,
            NodeResolutionState::Ambiguous,
            NodeReferenceKey::Name(ambiguous_name),
            normal_origin.clone(),
        ),
        (
            SurfaceNodeKind::Reference,
            NodeResolutionState::Unresolved,
            NodeReferenceKey::Name(unresolved_name),
            recovered_origin.clone(),
        ),
        (
            SurfaceNodeKind::AnnotationLabel,
            NodeResolutionState::Resolved,
            NodeReferenceKey::Label(resolved_label),
            normal_origin.clone(),
        ),
        (
            SurfaceNodeKind::AnnotationLabel,
            NodeResolutionState::Ambiguous,
            NodeReferenceKey::Label(ambiguous_label),
            normal_origin.clone(),
        ),
        (
            SurfaceNodeKind::AnnotationLabel,
            NodeResolutionState::Unresolved,
            NodeReferenceKey::Label(unresolved_label),
            recovered_origin.clone(),
        ),
        (
            SurfaceNodeKind::ImportItem,
            NodeResolutionState::Resolved,
            NodeReferenceKey::Import(resolved_import),
            normal_origin.clone(),
        ),
        (
            SurfaceNodeKind::ImportItem,
            NodeResolutionState::Ambiguous,
            NodeReferenceKey::Import(ambiguous_import),
            normal_origin.clone(),
        ),
        (
            SurfaceNodeKind::ImportItem,
            NodeResolutionState::Unresolved,
            NodeReferenceKey::Import(unresolved_import),
            recovered_origin.clone(),
        ),
        (
            SurfaceNodeKind::ExportItem,
            NodeResolutionState::Resolved,
            NodeReferenceKey::Export(module_export),
            normal_origin.clone(),
        ),
        (
            SurfaceNodeKind::ExportItem,
            NodeResolutionState::Resolved,
            NodeReferenceKey::Export(alias_export),
            normal_origin.clone(),
        ),
        (
            SurfaceNodeKind::ExportItem,
            NodeResolutionState::Resolved,
            NodeReferenceKey::Export(symbol_export),
            normal_origin.clone(),
        ),
        (
            SurfaceNodeKind::ExportItem,
            NodeResolutionState::Unresolved,
            NodeReferenceKey::Export(unresolved_export),
            recovered_origin.clone(),
        ),
    ] {
        builder
            .push(
                ResolvedNode::new(kind, Vec::new(), origin)
                    .with_resolution(resolution)
                    .with_reference_key(key),
            )
            .unwrap();
    }
    let root = builder
        .push(ResolvedNode::new(
            SurfaceNodeKind::CompilationUnit,
            (0..15).map(ResolvedNodeId::new).collect(),
            normal_origin,
        ))
        .unwrap();
    let arena = builder.finish(root).unwrap();
    ResolvedAst::try_new(source_id, module, arena, name_refs, label_refs, imports).unwrap()
}

fn reference_key_snapshot(source_id: SourceId, module: ModuleId) -> Vec<Option<NodeReferenceKey>> {
    let origin = origin(source_id, module.clone());
    let mut name_refs = NameRefTable::new();
    let mut label_refs = LabelRefTable::new();
    let mut imports = ResolvedImports::new();
    let name_id = name_refs.insert(NameRefEntry::new(
        ReferenceSite::new(ResolvedNodeId::new(0), range(source_id, 0, 1), "N"),
        NameResolution::Unresolved(UnresolvedNameRef::new(
            "N",
            range(source_id, 0, 1),
            NameLookupClass::Symbol,
        )),
        origin.clone(),
    ));
    let label_id = label_refs.insert(LabelRefEntry::new(
        ReferenceSite::new(ResolvedNodeId::new(1), range(source_id, 1, 2), "L"),
        LabelResolution::Unresolved(UnresolvedLabelRef::new(
            "L",
            range(source_id, 1, 2),
            LabelExpectation::Theorem,
        )),
        origin.clone(),
    ));
    let import_id = imports.push_import(ResolvedImport::new(
        ResolvedNodeId::new(2),
        range(source_id, 2, 3),
        "import dep;",
        None,
        ImportResolution::Resolved(module_id("pkg", "dep")),
        origin.clone(),
    ));
    let export_id = imports.push_export(ResolvedExport::new(
        ResolvedNodeId::new(3),
        range(source_id, 3, 4),
        "export dep;",
        ExportTarget::Module(module_id("pkg", "dep")),
        origin.clone(),
    ));

    let mut builder = ResolvedArenaBuilder::new();
    let name_node = builder
        .push(
            ResolvedNode::new(SurfaceNodeKind::Reference, Vec::new(), origin.clone())
                .with_reference_key(NodeReferenceKey::Name(name_id)),
        )
        .unwrap();
    let label_node = builder
        .push(
            ResolvedNode::new(SurfaceNodeKind::AnnotationLabel, Vec::new(), origin.clone())
                .with_reference_key(NodeReferenceKey::Label(label_id)),
        )
        .unwrap();
    let import_node = builder
        .push(
            ResolvedNode::new(SurfaceNodeKind::ImportItem, Vec::new(), origin.clone())
                .with_reference_key(NodeReferenceKey::Import(import_id)),
        )
        .unwrap();
    let export_node = builder
        .push(
            ResolvedNode::new(SurfaceNodeKind::ExportItem, Vec::new(), origin.clone())
                .with_reference_key(NodeReferenceKey::Export(export_id)),
        )
        .unwrap();
    let root = builder
        .push(ResolvedNode::new(
            SurfaceNodeKind::CompilationUnit,
            vec![name_node, label_node, import_node, export_node],
            origin,
        ))
        .unwrap();
    let arena = builder.finish(root).unwrap();
    ResolvedAst::try_new(source_id, module, arena, name_refs, label_refs, imports)
        .unwrap()
        .nodes()
        .iter()
        .take(4)
        .map(|(_, node)| node.reference_key())
        .collect()
}

#[derive(Clone)]
enum SurfaceArenaFault {
    Kind,
    Children,
    Range,
    Recovery,
    Resolution,
    ReferenceKey,
    OriginSource(SourceId),
    OriginModule(ModuleId),
    OriginImport,
    StructuralPath,
}

fn apply_surface_fault(
    lowered: &mut SurfaceResolvedArena,
    source: SurfaceNodeId,
    fault: SurfaceArenaFault,
) {
    let resolved = &mut lowered.arena.nodes[source.index()];
    match fault {
        SurfaceArenaFault::Kind => {
            resolved.kind = if resolved.kind == SurfaceNodeKind::ItemList {
                SurfaceNodeKind::PlaceholderItem
            } else {
                SurfaceNodeKind::ItemList
            };
        }
        SurfaceArenaFault::Children => {
            if resolved.children.is_empty() {
                assert!(source.index() > 0);
                resolved.children.push(ResolvedNodeId::new(0));
            } else {
                resolved.children.clear();
            }
        }
        SurfaceArenaFault::Range => {
            let origin_source = resolved.origin.source_id;
            resolved.origin.anchor = SourceAnchor::Range(range(origin_source, 900, 901));
        }
        SurfaceArenaFault::Recovery => {
            resolved.recovery = match resolved.recovery {
                RecoveryState::Normal => RecoveryState::Recovered,
                RecoveryState::Recovered => RecoveryState::Normal,
            };
        }
        SurfaceArenaFault::Resolution => {
            resolved.resolution = NodeResolutionState::Resolved;
        }
        SurfaceArenaFault::ReferenceKey => {
            resolved.reference_key = Some(NodeReferenceKey::Name(NameRefId::new(0)));
        }
        SurfaceArenaFault::OriginSource(source_id) => {
            resolved.origin.source_id = source_id;
        }
        SurfaceArenaFault::OriginModule(module) => {
            resolved.origin.module_id = module;
        }
        SurfaceArenaFault::OriginImport => {
            resolved.origin.import_edge = Some(ResolvedImportId::new(0));
        }
        SurfaceArenaFault::StructuralPath => {
            resolved.origin.structural_path = vec![999];
        }
    }
}

fn assert_surface_fault(
    lowered: &SurfaceResolvedArena,
    ast: &SurfaceAst,
    module: &ModuleId,
    node: SurfaceNodeId,
    fault: SurfaceArenaFault,
    expected: &str,
) {
    let mut changed = lowered.clone();
    apply_surface_fault(&mut changed, node, fault);
    let error = changed.validate_against(ast, module).unwrap_err();
    assert_eq!(surface_error_class(error), expected);
}

fn surface_error_class(error: SurfaceResolvedArenaError) -> &'static str {
    match error {
        SurfaceResolvedArenaError::MissingRoot => "missing-root",
        SurfaceResolvedArenaError::StructuralPathComponentOverflow { .. } => "overflow",
        SurfaceResolvedArenaError::InvalidChildOrder { .. } => "child-order",
        SurfaceResolvedArenaError::InvalidArena(_) => "arena",
        SurfaceResolvedArenaError::SourceMismatch => "source",
        SurfaceResolvedArenaError::ModuleMismatch => "module",
        SurfaceResolvedArenaError::NodeCountMismatch => "count",
        SurfaceResolvedArenaError::RootMismatch => "root",
        SurfaceResolvedArenaError::NodeKindMismatch { .. } => "kind",
        SurfaceResolvedArenaError::ChildListMismatch { .. } => "children",
        SurfaceResolvedArenaError::RangeMismatch { .. } => "range",
        SurfaceResolvedArenaError::RecoveryMismatch { .. } => "recovery",
        SurfaceResolvedArenaError::ResolutionStateMismatch { .. } => "resolution",
        SurfaceResolvedArenaError::ReferenceKeyMismatch { .. } => "reference-key",
        SurfaceResolvedArenaError::OriginMismatch { .. } => "origin",
        SurfaceResolvedArenaError::StructuralPathMismatch { .. } => "structural-path",
    }
}

fn surface_ast_fixture(source_id: SourceId) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let constructor_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Example",
        range(source_id, 0, 7),
    );
    let recovered_argument =
        builder.add_recovered_token(SurfaceTokenKind::ErrorRecovery, "?", range(source_id, 8, 9));
    let constructor = builder.add_node(
        SurfaceNodeKind::StructureConstructor,
        range(source_id, 0, 9),
        vec![constructor_name, recovered_argument],
    );
    let root = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 0, 9),
        vec![constructor],
    );
    builder.add_node(
        SurfaceNodeKind::SelectorAccess,
        range(source_id, 20, 24),
        Vec::new(),
    );
    builder.finish(Some(root), None)
}

fn larger_surface_ast_fixture(source_id: SourceId) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    for index in 0..6 {
        builder.add_node(
            SurfaceNodeKind::PlaceholderItem,
            range(source_id, index, index + 1),
            Vec::new(),
        );
    }
    let root = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 0, 6),
        Vec::new(),
    );
    builder.finish(Some(root), None)
}

fn source_id(seed: u8) -> SourceId {
    let snapshot_id = snapshot_id(seed);
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot_id)
        .unwrap()
}

fn source_id_pair(seed: u8) -> (SourceId, SourceId) {
    let snapshot_id = snapshot_id(seed);
    let allocator = InMemorySessionIdAllocator::new();
    (
        allocator.next_source_id(snapshot_id).unwrap(),
        allocator.next_source_id(snapshot_id).unwrap(),
    )
}

fn snapshot_id(seed: u8) -> BuildSnapshotId {
    let hex = format!("{seed:02x}").repeat(Hash::BYTE_LEN);
    BuildSnapshotId::from_published_schema_str(&format!("mizar-session-build-snapshot-v1:{hex}"))
        .unwrap()
}

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn origin(source_id: SourceId, module_id: ModuleId) -> SemanticOrigin {
    SemanticOrigin::new(
        source_id,
        module_id,
        SourceAnchor::Range(range(source_id, 0, 1)),
        vec![0],
    )
}

fn module_id(package: &str, path: &str) -> ModuleId {
    ModuleId::new(PackageId::new(package), ModulePath::new(path))
}

fn symbol_id(module: ModuleId, local: &str, fqn: &str) -> SymbolId {
    SymbolId::new(
        module,
        LocalSymbolId::new(local),
        FullyQualifiedName::new(fqn),
    )
}
