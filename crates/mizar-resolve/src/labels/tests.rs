use super::*;
use crate::env::{ContributionKind, SourceContributionIndex};
use crate::resolved_ast::{ResolvedArenaBuilder, ResolvedNode, SemanticOrigin};
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
    SourceAnchor, SourceId,
};
use mizar_syntax::{
    SurfaceAst, SurfaceAstBuilder, SurfaceBuilderNodeId, SurfaceFormulaConstant, SurfaceNodeKind,
    SurfaceTokenKind, SyntaxRecoveryKind,
};

#[test]
fn unqualified_citation_respects_proof_block_visibility_and_confinement() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let dep = module_id("dep", "logic");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let local_contribution = contribution(&mut contributions, current.clone(), source_id, 0);
    let imported_contribution = contribution(&mut contributions, dep.clone(), source_id, 100);
    let current_fixture = ProjectionFixture::new(
        source_id,
        current.clone(),
        namespace.clone(),
        local_contribution,
    );
    let dep_fixture = ProjectionFixture::new(
        source_id,
        dep.clone(),
        namespace.clone(),
        imported_contribution,
    );
    let outer_scope = LabelScopePath::new(vec![0]);
    let inner_scope = LabelScopePath::new(vec![0, 1]);

    let projections = vec![
        proof_step_projection(&current_fixture, "A", 10, 1, outer_scope.clone()),
        proof_step_projection(&current_fixture, "B", 20, 2, inner_scope.clone()),
        current_theorem_projection(&current_fixture, "T", 30, 3),
        imported_theorem_projection(&dep_fixture, "Lib", 40),
    ];
    let references = vec![
        unqualified_ref(
            source_id,
            current.clone(),
            2,
            80,
            "T",
            Some(inner_scope.clone()),
        ),
        unqualified_ref(
            source_id,
            current.clone(),
            4,
            60,
            "A",
            Some(inner_scope.clone()),
        ),
        unqualified_ref(source_id, current.clone(), 5, 70, "B", Some(outer_scope)),
        unqualified_ref(source_id, current.clone(), 6, 90, "Lib", Some(inner_scope)),
    ];

    let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

    assert_unresolved_label(&resolved, 0, LabelExpectation::ProofOrTheorem, "T");
    assert_resolved_label(&resolved, 1, "app::main::proof::A");
    assert_unresolved_label(&resolved, 2, LabelExpectation::ProofOrTheorem, "B");
    assert_resolved_label(&resolved, 3, "dep::logic::theorem::Lib");
    assert!(resolved.has_unresolved());
}

#[test]
fn duplicate_and_visible_nested_labels_are_internal_diagnostics() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
    let fixture =
        ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
    let outer_scope = LabelScopePath::new(vec![0]);
    let inner_scope = LabelScopePath::new(vec![0, 1]);
    let sibling_scope = LabelScopePath::new(vec![0, 2]);
    let outer = proof_step_projection(&fixture, "A", 10, 1, outer_scope.clone());
    let duplicate = proof_step_projection(&fixture, "A", 20, 2, outer_scope);
    let inner_conflict = proof_step_projection(&fixture, "A", 30, 3, inner_scope);
    let sibling_conflict = proof_step_projection(&fixture, "A", 40, 4, sibling_scope);
    let projections = vec![sibling_conflict, inner_conflict, duplicate, outer];

    let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &[]);

    let diagnostics = resolved.diagnostics();
    assert_eq!(diagnostics.len(), 3);
    assert_eq!(
        diagnostics
            .iter()
            .map(LabelDiagnostic::kind)
            .collect::<Vec<_>>(),
        vec![
            LabelDiagnosticKind::DuplicateLabel,
            LabelDiagnosticKind::ConflictingVisibleLabel,
            LabelDiagnosticKind::ConflictingVisibleLabel,
        ]
    );
    assert_eq!(
        diagnostics
            .iter()
            .map(LabelDiagnostic::primary_range)
            .collect::<Vec<_>>(),
        vec![
            range(source_id, 20, 21),
            range(source_id, 30, 31),
            range(source_id, 40, 41),
        ]
    );
    let related_ranges = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.related_ranges().to_vec())
        .collect::<Vec<_>>();
    assert_eq!(
        related_ranges,
        vec![
            vec![range(source_id, 10, 11)],
            vec![range(source_id, 10, 11), range(source_id, 20, 21)],
            vec![range(source_id, 10, 11), range(source_id, 20, 21)],
        ]
    );
}

#[test]
fn forward_references_to_later_theorem_labels_are_unresolved() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
    let fixture =
        ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
    let projections = vec![current_theorem_projection(&fixture, "Later", 20, 5)];
    let references = vec![
        unqualified_ref(source_id, current.clone(), 4, 10, "Later", None),
        unqualified_ref(source_id, current.clone(), 6, 30, "Later", None),
    ];

    let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

    assert_unresolved_label(&resolved, 0, LabelExpectation::ProofOrTheorem, "Later");
    assert_resolved_label(&resolved, 1, "app::main::theorem::Later");
}

#[test]
fn forward_references_to_later_proof_step_labels_are_unresolved() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
    let fixture =
        ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
    let scope = LabelScopePath::new(vec![0]);
    let projections = vec![proof_step_projection(
        &fixture,
        "LaterStep",
        20,
        5,
        scope.clone(),
    )];
    let references = vec![
        unqualified_ref(
            source_id,
            current.clone(),
            4,
            10,
            "LaterStep",
            Some(scope.clone()),
        ),
        unqualified_ref(source_id, current.clone(), 6, 40, "LaterStep", Some(scope)),
    ];

    let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

    assert_unresolved_label(&resolved, 0, LabelExpectation::ProofOrTheorem, "LaterStep");
    assert_resolved_label(&resolved, 1, "app::main::proof::LaterStep");
}

#[test]
fn qualified_and_lowered_grouped_item_citations_use_module_label_projections() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let dep = module_id("dep", "logic");
    let namespace = NamespacePath::new("logic");
    let current_namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let imported_contribution = contribution(&mut contributions, dep.clone(), source_id, 0);
    let dep_fixture = ProjectionFixture::new(
        source_id,
        dep.clone(),
        namespace.clone(),
        imported_contribution,
    );
    let projections = vec![
        imported_theorem_projection(&dep_fixture, "Th1", 10),
        imported_theorem_projection(&dep_fixture, "G1", 20),
        imported_theorem_projection(&dep_fixture, "G2", 30),
    ];
    let references = vec![
        qualified_ref(
            source_id,
            current.clone(),
            dep.clone(),
            namespace.clone(),
            2,
            50,
            "Th1",
        ),
        qualified_ref(
            source_id,
            current.clone(),
            dep.clone(),
            namespace.clone(),
            3,
            60,
            "G1",
        ),
        qualified_ref(
            source_id,
            current.clone(),
            dep.clone(),
            namespace,
            4,
            70,
            "G2",
        ),
    ];

    let resolved =
        LabelResolver::new(&projections).resolve(&current, &current_namespace, &references);

    assert_resolved_label(&resolved, 0, "dep::logic::theorem::Th1");
    assert_resolved_label(&resolved, 1, "dep::logic::theorem::G1");
    assert_resolved_label(&resolved, 2, "dep::logic::theorem::G2");
}

#[test]
fn imported_local_only_labels_are_not_visible_to_citations() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let dep = module_id("dep", "logic");
    let namespace = NamespacePath::new("logic");
    let current_namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let imported_contribution = contribution(&mut contributions, dep.clone(), source_id, 0);
    let dep_fixture = ProjectionFixture::new(
        source_id,
        dep.clone(),
        namespace.clone(),
        imported_contribution,
    );
    let projections = vec![
        imported_theorem_projection(&dep_fixture, "Hidden", 10)
            .with_export_status(ExportStatus::LocalOnly),
    ];
    let references = vec![
        unqualified_ref(source_id, current.clone(), 2, 30, "Hidden", None),
        qualified_ref(
            source_id,
            current.clone(),
            dep,
            namespace.clone(),
            3,
            40,
            "Hidden",
        ),
    ];

    let resolved =
        LabelResolver::new(&projections).resolve(&current, &current_namespace, &references);

    assert_unresolved_label(&resolved, 0, LabelExpectation::ProofOrTheorem, "Hidden");
    assert_unresolved_label(&resolved, 1, LabelExpectation::Theorem, "Hidden");
    assert!(
        resolved
            .index()
            .visible_candidates(&namespace, "Hidden")
            .is_empty()
    );
}

#[test]
fn recovered_empty_and_failed_namespace_references_are_unresolved() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let scope = LabelScopePath::new(vec![0]);
    let (recovered_site, recovered_origin) = reference_site(source_id, current.clone(), 10, "A", 1);
    let (empty_site, empty_origin) = reference_site(source_id, current.clone(), 20, "", 2);
    let (failed_site, failed_origin) = reference_site(source_id, current.clone(), 30, "B", 3);
    let references = vec![
        LabelReferenceCandidate::unqualified_citation(
            recovered_site,
            recovered_origin.recovered(),
            1,
            Some(scope),
        ),
        LabelReferenceCandidate::unqualified_citation(empty_site, empty_origin, 2, None),
        LabelReferenceCandidate::failed_namespace(
            failed_site,
            failed_origin,
            3,
            LabelExpectation::Theorem,
        ),
    ];

    let resolved = LabelResolver::new(&[]).resolve(&current, &namespace, &references);

    assert_unresolved_label(&resolved, 0, LabelExpectation::ProofOrTheorem, "A");
    assert_unresolved_label(&resolved, 1, LabelExpectation::ProofOrTheorem, "");
    assert_unresolved_label(&resolved, 2, LabelExpectation::Theorem, "B");
}

#[test]
fn recovered_label_projections_do_not_emit_conflict_diagnostics() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
    let fixture =
        ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
    let clean = current_theorem_projection(&fixture, "A", 10, 1);
    let mut recovered_data = fixture.data("A", LabelKind::Theorem, "theorem", 20, 2);
    recovered_data.origin = recovered_data.origin.recovered();
    let recovered = LabelProjection::current_module(recovered_data, 2)
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported);
    let projections = vec![clean, recovered];
    let references = vec![unqualified_ref(
        source_id,
        current.clone(),
        3,
        30,
        "A",
        None,
    )];

    let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

    assert!(resolved.diagnostics().is_empty());
    assert_resolved_label(&resolved, 0, "app::main::theorem::A");
}

#[test]
fn ambiguous_cross_family_citations_keep_sorted_candidates() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
    let fixture =
        ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
    let scope = LabelScopePath::new(vec![0]);
    let projections = vec![
        current_theorem_projection(&fixture, "A", 20, 1),
        proof_step_projection(&fixture, "A", 10, 2, scope.clone()),
    ];
    let references = vec![unqualified_ref(
        source_id,
        current.clone(),
        3,
        30,
        "A",
        Some(scope),
    )];

    let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

    let entry = resolved.table().get(resolved.ids()[0]).unwrap();
    let LabelResolution::Ambiguous(ambiguous) = entry.resolution() else {
        panic!("expected ambiguous label reference");
    };
    assert_eq!(
        ambiguous
            .candidates()
            .iter()
            .map(|candidate| candidate.origin().as_str())
            .collect::<Vec<_>>(),
        vec!["app::main::proof::A", "app::main::theorem::A"]
    );
}

#[test]
fn label_index_and_reference_table_order_are_deterministic() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
    let fixture =
        ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
    let projections = vec![
        current_theorem_projection(&fixture, "Z", 30, 3),
        current_theorem_projection(&fixture, "A", 10, 1),
        current_theorem_projection(&fixture, "M", 20, 2),
    ];
    let references = vec![
        unqualified_ref(source_id, current.clone(), 6, 60, "Z", None),
        unqualified_ref(source_id, current.clone(), 4, 40, "A", None),
        unqualified_ref(source_id, current.clone(), 5, 50, "M", None),
    ];

    let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

    assert_eq!(
        resolved
            .index()
            .iter()
            .map(|entry| entry.origin_path().as_str())
            .collect::<Vec<_>>(),
        vec![
            "app::main::theorem::A",
            "app::main::theorem::M",
            "app::main::theorem::Z",
        ]
    );
    assert_eq!(
        resolved
            .ids()
            .iter()
            .map(|id| resolved.table().get(*id).unwrap().site().spelling())
            .collect::<Vec<_>>(),
        vec!["A", "M", "Z"]
    );
}

#[test]
fn source_collector_reproduces_exact_inner_to_outer_b5c_profile() {
    let source_id = source_id();
    let ast = b5c_inner_ast(source_id);
    let module = module_id("pkg", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collection =
        ProofLabelSourceCollector::new(&ast, &module, namespace.clone(), contribution, &resolved)
            .unwrap()
            .collect()
            .unwrap();

    assert_eq!(collection.projections().len(), 1);
    assert_eq!(collection.references().len(), 1);
    let projection = &collection.projections()[0];
    assert_eq!(projection.primary_spelling(), "A");
    assert_eq!(projection.kind(), LabelKind::ProofStep);
    assert_eq!(projection.declaration_range(), range(source_id, 80, 81));
    assert_eq!(
        projection.origin_path().as_str(),
        "proof-step-v1|package=3:pkg|module=4:main|contribution=0|owner-kind=theorem|owner=33:ProofLabelInnerToOuterConfinement|owner-occurrence=0|proof-path=1:0|label=1:A|label-occurrence=0"
    );
    assert_eq!(projection.origin().source_id(), source_id);
    assert_eq!(projection.origin().module_id(), &module);
    assert_eq!(
        projection.origin().anchor(),
        &SourceAnchor::Range(range(source_id, 80, 81))
    );
    assert_eq!(projection.origin().structural_path(), &[57, 42, 8]);
    match projection.source() {
        LabelProjectionSource::CurrentModule {
            visible_after_ordinal,
            proof_scope: Some(scope),
        } => {
            assert_eq!(*visible_after_ordinal, 3);
            assert_eq!(scope.path(), &[0, 0]);
        }
        other => panic!("unexpected projection source: {other:?}"),
    }

    let reference = &collection.references()[0];
    assert_eq!(reference.site().node().index(), 52);
    assert_eq!(reference.site().range(), range(source_id, 165, 166));
    assert_eq!(reference.site().spelling(), "A");
    assert_eq!(reference.ordinal(), 5);
    assert_eq!(reference.origin().source_id(), source_id);
    assert_eq!(reference.origin().module_id(), &module);
    assert_eq!(
        reference.origin().anchor(),
        &SourceAnchor::Range(range(source_id, 165, 166))
    );
    assert_eq!(reference.origin().structural_path(), &[57, 55, 52]);
    match reference.scope() {
        LabelReferenceScope::Unqualified {
            proof_scope: Some(scope),
        } => assert_eq!(scope.path(), &[0]),
        other => panic!("unexpected reference scope: {other:?}"),
    }

    let result = LabelResolver::new(collection.projections()).resolve(
        &module,
        &namespace,
        collection.references(),
    );
    assert_eq!(result.index().len(), 1);
    assert_eq!(result.table().len(), 1);
    assert!(result.diagnostics().is_empty());
    assert!(result.has_unresolved());
    assert_unresolved_label(&result, 0, LabelExpectation::ProofOrTheorem, "A");
}

#[test]
fn source_collector_reproduces_exact_sibling_b5c_profile() {
    let source_id = source_id();
    let ast = b5c_sibling_ast(source_id);
    let module = module_id("pkg", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collection =
        ProofLabelSourceCollector::new(&ast, &module, namespace.clone(), contribution, &resolved)
            .unwrap()
            .collect()
            .unwrap();

    let [projection] = collection.projections() else {
        panic!("expected one projection");
    };
    assert_eq!(projection.declaration_range(), range(source_id, 75, 76));
    assert_eq!(projection.origin().source_id(), source_id);
    assert_eq!(projection.origin().module_id(), &module);
    assert_eq!(
        projection.origin().anchor(),
        &SourceAnchor::Range(range(source_id, 75, 76))
    );
    assert_eq!(projection.origin().structural_path(), &[67, 47, 8]);
    assert_eq!(
        projection.origin_path().as_str(),
        "proof-step-v1|package=3:pkg|module=4:main|contribution=0|owner-kind=theorem|owner=28:ProofLabelSiblingConfinement|owner-occurrence=0|proof-path=1:0|label=1:A|label-occurrence=0"
    );
    match projection.source() {
        LabelProjectionSource::CurrentModule {
            visible_after_ordinal,
            proof_scope: Some(scope),
        } => {
            assert_eq!(*visible_after_ordinal, 3);
            assert_eq!(scope.path(), &[0, 0]);
        }
        other => panic!("unexpected projection source: {other:?}"),
    }

    let [reference] = collection.references() else {
        panic!("expected one reference");
    };
    assert_eq!(reference.site().node().index(), 60);
    assert_eq!(reference.site().range(), range(source_id, 182, 183));
    assert_eq!(reference.site().spelling(), "A");
    assert_eq!(reference.ordinal(), 6);
    assert_eq!(reference.origin().source_id(), source_id);
    assert_eq!(reference.origin().module_id(), &module);
    assert_eq!(
        reference.origin().anchor(),
        &SourceAnchor::Range(range(source_id, 182, 183))
    );
    assert_eq!(reference.origin().structural_path(), &[67, 63, 60]);
    match reference.scope() {
        LabelReferenceScope::Unqualified {
            proof_scope: Some(scope),
        } => assert_eq!(scope.path(), &[0, 1]),
        other => panic!("unexpected reference scope: {other:?}"),
    }

    let result = LabelResolver::new(collection.projections()).resolve(
        &module,
        &namespace,
        collection.references(),
    );
    assert!(result.diagnostics().is_empty());
    assert_unresolved_label(&result, 0, LabelExpectation::ProofOrTheorem, "A");

    let inner = b5c_inner_ast(source_id);
    let inner_resolved = SurfaceResolvedArena::lower(&inner, &module).unwrap();
    let inner_collection =
        ProofLabelSourceCollector::new(&inner, &module, namespace, contribution, &inner_resolved)
            .unwrap()
            .collect()
            .unwrap();
    assert_ne!(
        inner_collection.projections()[0].origin_path(),
        projection.origin_path(),
        "the two B5C owners must retain distinct canonical label identities"
    );
}

#[test]
fn source_collector_defers_own_proof_visibility_and_allows_post_completion_use() {
    let source_id = source_id();
    let ast = own_proof_and_post_completion_ast(source_id);
    let module = module_id("pkg", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collection =
        ProofLabelSourceCollector::new(&ast, &module, namespace.clone(), contribution, &resolved)
            .unwrap()
            .collect()
            .unwrap();

    let [projection] = collection.projections() else {
        panic!("expected one projection");
    };
    assert_eq!(projection.visible_after_ordinal(), Some(2));
    assert_eq!(
        collection
            .references()
            .iter()
            .map(LabelReferenceCandidate::ordinal)
            .collect::<Vec<_>>(),
        vec![2, 3]
    );
    let result = LabelResolver::new(collection.projections()).resolve(
        &module,
        &namespace,
        collection.references(),
    );
    assert_unresolved_label(&result, 0, LabelExpectation::ProofOrTheorem, "A");
    assert_resolved_label(&result, 1, projection.origin_path().as_str());
}

#[test]
fn source_collector_allows_enclosing_label_in_child_and_skips_unlisted_proof_children() {
    let source_id = source_id();
    let ast = enclosing_to_child_ast(source_id);
    let module = module_id("pkg", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collection =
        ProofLabelSourceCollector::new(&ast, &module, namespace.clone(), contribution, &resolved)
            .unwrap()
            .collect()
            .unwrap();

    assert_eq!(collection.projections().len(), 1);
    assert_eq!(collection.references().len(), 1);
    assert_eq!(
        collection.projections()[0].proof_scope().unwrap().path(),
        &[0]
    );
    assert_eq!(collection.references()[0].ordinal(), 3);
    match collection.references()[0].scope() {
        LabelReferenceScope::Unqualified {
            proof_scope: Some(scope),
        } => assert_eq!(scope.path(), &[0, 0]),
        other => panic!("unexpected reference scope: {other:?}"),
    }
    let result = LabelResolver::new(collection.projections()).resolve(
        &module,
        &namespace,
        collection.references(),
    );
    assert_resolved_label(
        &result,
        0,
        collection.projections()[0].origin_path().as_str(),
    );
}

#[test]
fn source_collector_mixed_reference_list_is_default_deny_and_ordered() {
    let source_id = source_id();
    let ast = mixed_reference_ast(source_id);
    let module = module_id("pkg", "main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collection = ProofLabelSourceCollector::new(
        &ast,
        &module,
        NamespacePath::new("main"),
        contribution,
        &resolved,
    )
    .unwrap()
    .collect()
    .unwrap();

    assert!(collection.projections().is_empty());
    assert_eq!(
        collection
            .references()
            .iter()
            .map(|reference| (reference.site().spelling(), reference.ordinal()))
            .collect::<Vec<_>>(),
        vec![("A", 1), ("B", 1)]
    );
}

#[test]
fn source_collector_keeps_global_ordinals_and_theorem_roots_isolated() {
    let source_id = source_id();
    let ast = cross_theorem_ast(source_id);
    let module = module_id("pkg", "main");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collection =
        ProofLabelSourceCollector::new(&ast, &module, namespace.clone(), contribution, &resolved)
            .unwrap()
            .collect()
            .unwrap();

    assert_eq!(collection.projections().len(), 2);
    assert_eq!(
        collection
            .projections()
            .iter()
            .map(|projection| projection.proof_scope().unwrap().path())
            .collect::<Vec<_>>(),
        vec![&[0][..], &[1][..]]
    );
    assert!(
        collection.projections()[0]
            .origin_path()
            .as_str()
            .contains("|owner-occurrence=0|")
    );
    assert!(
        collection.projections()[1]
            .origin_path()
            .as_str()
            .contains("|owner-occurrence=1|")
    );
    assert_eq!(collection.projections()[0].visible_after_ordinal(), Some(1));
    assert!(
        collection.projections()[0]
            .origin_path()
            .as_str()
            .ends_with("|label=1:A|label-occurrence=0")
    );
    assert!(
        collection.projections()[1]
            .origin_path()
            .as_str()
            .ends_with("|label=1:A|label-occurrence=0"),
        "same-spelling label occurrence resets in the later theorem proof scope"
    );
    assert_eq!(collection.references()[0].ordinal(), 2);
    assert!(
        collection.projections()[0].visible_after_ordinal().unwrap()
            < collection.references()[0].ordinal(),
        "the earlier theorem projection must be ordinal-eligible and fail only confinement"
    );
    let result = LabelResolver::new(collection.projections()).resolve(
        &module,
        &namespace,
        collection.references(),
    );
    assert!(result.diagnostics().is_empty());
    assert_unresolved_label(&result, 0, LabelExpectation::ProofOrTheorem, "A");
}

#[test]
fn source_collector_rejects_malformed_upper_chains_and_recovered_boundary() {
    let source_id = source_id();
    let module = module_id("pkg", "main");
    for ast in [
        malformed_upper_ast(source_id, UpperShape::MissingCompilation),
        malformed_upper_ast(source_id, UpperShape::AdditionalCompilation),
        malformed_upper_ast(source_id, UpperShape::WrongCompilationChild),
        malformed_upper_ast(source_id, UpperShape::RootWrongStructuralChild),
        malformed_upper_ast(source_id, UpperShape::TheoremDirectlyUnderRoot),
        malformed_upper_ast(source_id, UpperShape::MissingItemList),
        malformed_upper_ast(source_id, UpperShape::AdditionalItemList),
        relocated_theorem_ast(source_id, true),
        relocated_theorem_ast(source_id, false),
        recovered_proof_boundary_ast(source_id),
    ] {
        let mut contributions = SourceContributionIndex::new();
        let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
        let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
        let collection = ProofLabelSourceCollector::new(
            &ast,
            &module,
            NamespacePath::new("main"),
            contribution,
            &resolved,
        )
        .unwrap()
        .collect()
        .unwrap();
        assert!(collection.projections().is_empty());
        assert!(collection.references().is_empty());
    }
}

#[test]
fn source_collector_rejects_wrong_module_and_reports_checked_overflow() {
    let source_id = source_id();
    let ast = b5c_inner_ast(source_id);
    let module = module_id("pkg", "main");
    let wrong_module = module_id("pkg", "other");
    let namespace = NamespacePath::new("main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();

    assert!(matches!(
        ProofLabelSourceCollector::new(&ast, &wrong_module, namespace, contribution, &resolved),
        Err(ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::ModuleMismatch
        ))
    ));
    let stale_ast = b5c_sibling_ast(source_id);
    assert!(matches!(
        ProofLabelSourceCollector::new(
            &stale_ast,
            &module,
            NamespacePath::new("main"),
            contribution,
            &resolved
        ),
        Err(ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::NodeCountMismatch
        ))
    ));
    #[cfg(target_pointer_width = "64")]
    {
        let node = ast.root().unwrap();
        let overflowing = usize::try_from(u64::from(u32::MAX) + 1).unwrap();
        assert!(matches!(
            checked_scope_component(node, overflowing),
            Err(ProofLabelSourceCollectionError::ScopeComponentOverflow {
                node: error_node
            }) if error_node == node
        ));
        assert!(matches!(
            checked_structural_component_value(node, overflowing),
            Err(
                ProofLabelSourceCollectionError::StructuralPathComponentOverflow {
                    node: error_node
                }
            ) if error_node == node
        ));
    }
}

#[test]
fn source_collector_is_deterministic_and_identity_is_length_framed() {
    let source_id = source_id();
    let ast = b5c_inner_ast(source_id);
    let module = module_id("på", "mód");
    let namespace = NamespacePath::new("mód");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collector =
        ProofLabelSourceCollector::new(&ast, &module, namespace, contribution, &resolved).unwrap();

    let first = collector.collect().unwrap();
    let second = collector.collect().unwrap();
    assert_eq!(first, second);
    assert_eq!(first.clone(), first);
    assert!(
        first.projections()[0]
            .origin_path()
            .as_str()
            .starts_with("proof-step-v1|package=3:på|module=4:mód|contribution=0|")
    );

    let empty_path = proof_label_origin_path(&module, contribution, "Ow|ner", 2, &[], "Å:|", 3);
    assert_eq!(
        empty_path.as_str(),
        "proof-step-v1|package=3:på|module=4:mód|contribution=0|owner-kind=theorem|owner=6:Ow|ner|owner-occurrence=2|proof-path=0:|label=4:Å:||label-occurrence=3"
    );
}

#[test]
fn proof_step_identity_mutates_each_framed_component_independently() {
    let source_id = source_id();
    let base_module = module_id("pkg", "main");
    let mut contributions = SourceContributionIndex::new();
    let base_contribution = contribution(&mut contributions, base_module.clone(), source_id, 0);
    let other_contribution = contribution(&mut contributions, base_module.clone(), source_id, 10);
    let base = proof_label_origin_path(
        &base_module,
        base_contribution,
        "Owner",
        0,
        &[0, 2],
        "Label",
        0,
    );
    assert_eq!(
        base.as_str(),
        "proof-step-v1|package=3:pkg|module=4:main|contribution=0|owner-kind=theorem|owner=5:Owner|owner-occurrence=0|proof-path=2:0,2|label=5:Label|label-occurrence=0"
    );

    let mutations = [
        proof_label_origin_path(
            &module_id("other", "main"),
            base_contribution,
            "Owner",
            0,
            &[0, 2],
            "Label",
            0,
        ),
        proof_label_origin_path(
            &module_id("pkg", "other"),
            base_contribution,
            "Owner",
            0,
            &[0, 2],
            "Label",
            0,
        ),
        proof_label_origin_path(
            &base_module,
            other_contribution,
            "Owner",
            0,
            &[0, 2],
            "Label",
            0,
        ),
        proof_label_origin_path(
            &base_module,
            base_contribution,
            "Other",
            0,
            &[0, 2],
            "Label",
            0,
        ),
        proof_label_origin_path(
            &base_module,
            base_contribution,
            "Owner",
            1,
            &[0, 2],
            "Label",
            0,
        ),
        proof_label_origin_path(
            &base_module,
            base_contribution,
            "Owner",
            0,
            &[0, 3],
            "Label",
            0,
        ),
        proof_label_origin_path(
            &base_module,
            base_contribution,
            "Owner",
            0,
            &[0, 2],
            "Other",
            0,
        ),
        proof_label_origin_path(
            &base_module,
            base_contribution,
            "Owner",
            0,
            &[0, 2],
            "Label",
            1,
        ),
    ];
    for mutation in mutations {
        assert_ne!(mutation, base);
    }

    let exact_bytes = proof_label_origin_path(
        &module_id("på", "mód"),
        base_contribution,
        "Öw|ner",
        2,
        &[],
        "Å:|",
        3,
    );
    assert_eq!(
        exact_bytes.as_str(),
        "proof-step-v1|package=3:på|module=4:mód|contribution=0|owner-kind=theorem|owner=7:Öw|ner|owner-occurrence=2|proof-path=0:|label=4:Å:||label-occurrence=3"
    );

    let composed =
        proof_label_origin_path(&base_module, base_contribution, "Owner", 0, &[], "Å", 0);
    let decomposed = proof_label_origin_path(
        &base_module,
        base_contribution,
        "Owner",
        0,
        &[],
        "A\u{030a}",
        0,
    );
    assert_eq!(
        composed.as_str(),
        "proof-step-v1|package=3:pkg|module=4:main|contribution=0|owner-kind=theorem|owner=5:Owner|owner-occurrence=0|proof-path=0:|label=2:Å|label-occurrence=0"
    );
    assert_eq!(
        decomposed.as_str(),
        "proof-step-v1|package=3:pkg|module=4:main|contribution=0|owner-kind=theorem|owner=5:Owner|owner-occurrence=0|proof-path=0:|label=3:A\u{030a}|label-occurrence=0"
    );
    assert_ne!(
        composed, decomposed,
        "identity must preserve exact parser bytes without Unicode normalization"
    );
}

#[test]
fn source_collector_accepts_root_to_compilation_with_direct_token_siblings() {
    let source_id = source_id();
    let mut builder = CollectorAstBuilder::new(source_id);
    let statement = builder.valid_statement("A", Some("A"), false);
    let theorem = builder.valid_theorem("Owner", vec![statement]);
    let ast = builder.finish_items(vec![theorem]);

    let root = ast.root_view().unwrap();
    assert!(
        root.child_views()
            .filter(|child| matches!(child.kind(), SurfaceNodeKind::Token(_)))
            .count()
            > 1
    );
    assert_eq!(
        root.child_views()
            .filter(|child| matches!(child.kind(), SurfaceNodeKind::CompilationUnit))
            .count(),
        1
    );
    let collection = collect_fixture(&ast);
    assert_eq!(collection.projections().len(), 1);
    assert_eq!(collection.references().len(), 1);
}

#[test]
fn source_collector_accepts_compilation_to_exact_item_list() {
    let source_id = source_id();
    let mut builder = CollectorAstBuilder::new(source_id);
    let statement = builder.valid_statement("A", None, false);
    let theorem = builder.valid_theorem("Owner", vec![statement]);
    let ast = builder.finish_items(vec![theorem]);

    let item_list = exact_proof_label_item_list(&ast).expect("exact upper chain");
    assert!(matches!(item_list.kind(), SurfaceNodeKind::ItemList));
    assert_eq!(collect_fixture(&ast).projections().len(), 1);
}

#[test]
fn source_collector_scans_only_direct_item_list_theorems() {
    let source_id = source_id();
    let mut builder = CollectorAstBuilder::new(source_id);
    let hidden_statement = builder.valid_statement("Hidden", Some("Hidden"), false);
    let hidden_theorem = builder.valid_theorem("HiddenOwner", vec![hidden_statement]);
    let wrapper = builder.node(SurfaceNodeKind::VisibleItem, vec![hidden_theorem]);
    let direct_statement = builder.valid_statement("Direct", Some("Direct"), false);
    let direct_theorem = builder.valid_theorem("DirectOwner", vec![direct_statement]);
    let ast = builder.finish_items(vec![wrapper, direct_theorem]);

    let collection = collect_fixture(&ast);
    assert_eq!(
        collection
            .projections()
            .iter()
            .map(LabelProjection::primary_spelling)
            .collect::<Vec<_>>(),
        vec!["Direct"]
    );
    assert_eq!(
        collection
            .references()
            .iter()
            .map(|reference| (reference.site().spelling(), reference.ordinal()))
            .collect::<Vec<_>>(),
        vec![("Direct", 1)]
    );
}

#[test]
fn source_collector_default_denies_every_representative_unsupported_item_owner() {
    let source_id = source_id();
    for kind in [
        SurfaceNodeKind::LemmaItem,
        SurfaceNodeKind::VisibleItem,
        SurfaceNodeKind::StatementItem,
        SurfaceNodeKind::DefinitionBlockItem,
        SurfaceNodeKind::RegistrationBlockItem,
        SurfaceNodeKind::ClaimBlockItem,
        SurfaceNodeKind::Annotation,
        SurfaceNodeKind::PlaceholderItem,
    ] {
        let collection = collect_fixture(&unsupported_item_wrapper_ast(source_id, kind.clone()));
        assert_eq!(
            collection
                .projections()
                .iter()
                .map(LabelProjection::primary_spelling)
                .collect::<Vec<_>>(),
            vec!["Sentinel"],
            "unsupported item kind {kind:?} must not be descended"
        );
        assert_eq!(
            collection.projections()[0].proof_scope().unwrap().path(),
            &[0],
            "unsupported item kind {kind:?} must not allocate a theorem root"
        );
        assert_eq!(
            collection
                .references()
                .iter()
                .map(|reference| (reference.site().spelling(), reference.ordinal()))
                .collect::<Vec<_>>(),
            vec![("Sentinel", 1)],
            "unsupported item kind {kind:?} must consume no ordinal"
        );
    }

    let recovered = collect_fixture(&recovered_item_wrapper_ast(source_id));
    assert_eq!(
        recovered
            .projections()
            .iter()
            .map(LabelProjection::primary_spelling)
            .collect::<Vec<_>>(),
        vec!["Sentinel"]
    );
    assert_eq!(recovered.references()[0].ordinal(), 1);
}

#[test]
fn source_collector_default_denies_every_representative_unlisted_proof_child() {
    let source_id = source_id();
    for kind in [
        SurfaceNodeKind::LetStatement,
        SurfaceNodeKind::AssumptionStatement,
        SurfaceNodeKind::GivenStatement,
        SurfaceNodeKind::TakeStatement,
        SurfaceNodeKind::SetStatement,
        SurfaceNodeKind::ConsiderStatement,
        SurfaceNodeKind::ReconsiderStatement,
        SurfaceNodeKind::CaseReasoningStatement,
        SurfaceNodeKind::CaseItem,
        SurfaceNodeKind::SupposeItem,
        SurfaceNodeKind::NowStatement,
        SurfaceNodeKind::HerebyStatement,
        SurfaceNodeKind::IterativeEqualityStatement,
    ] {
        let collection = collect_fixture(&unsupported_proof_child_ast(source_id, kind.clone()));
        assert_eq!(
            collection
                .projections()
                .iter()
                .map(LabelProjection::primary_spelling)
                .collect::<Vec<_>>(),
            vec!["Sentinel"],
            "unlisted proof child {kind:?} must not be descended"
        );
        assert_eq!(
            collection
                .references()
                .iter()
                .map(|reference| (reference.site().spelling(), reference.ordinal()))
                .collect::<Vec<_>>(),
            vec![("Sentinel", 1)],
            "unlisted proof child {kind:?} must consume no ordinal"
        );
    }
}

#[test]
fn source_collector_rejects_every_required_theorem_owner_shape_mutation() {
    let source_id = source_id();
    for mutation in [
        TheoremShapeMutation::WrongRole,
        TheoremShapeMutation::WrongOwnerKind,
        TheoremShapeMutation::WrongColon,
        TheoremShapeMutation::MissingProof,
        TheoremShapeMutation::AdditionalProof,
        TheoremShapeMutation::WrappedProof,
        TheoremShapeMutation::RecoveredRole,
        TheoremShapeMutation::RecoveredOwner,
        TheoremShapeMutation::RecoveredColon,
    ] {
        let collection = collect_fixture(&malformed_theorem_owner_ast(source_id, mutation));
        assert_eq!(
            collection
                .projections()
                .iter()
                .map(LabelProjection::primary_spelling)
                .collect::<Vec<_>>(),
            vec!["Sentinel"],
            "malformed theorem mutation {mutation:?} must not be descended"
        );
        assert_eq!(
            collection.projections()[0].proof_scope().unwrap().path(),
            &[0],
            "malformed theorem mutation {mutation:?} must not allocate a root"
        );
        assert_eq!(
            collection.references()[0].ordinal(),
            1,
            "malformed theorem mutation {mutation:?} must consume no ordinal"
        );
    }
}

#[test]
fn source_collector_rejects_every_malformed_or_recovered_proof_boundary() {
    let source_id = source_id();
    for mutation in [
        ProofBoundaryMutation::MissingEnd,
        ProofBoundaryMutation::WrongStart,
        ProofBoundaryMutation::WrongEnd,
        ProofBoundaryMutation::RecoveredStart,
        ProofBoundaryMutation::RecoveredEnd,
        ProofBoundaryMutation::RecoveredInterior,
    ] {
        let collection = collect_fixture(&malformed_proof_boundary_ast(source_id, mutation));
        assert_eq!(
            collection
                .projections()
                .iter()
                .map(LabelProjection::primary_spelling)
                .collect::<Vec<_>>(),
            vec!["Sentinel"],
            "malformed proof mutation {mutation:?} must not be descended"
        );
        assert_eq!(
            collection.projections()[0].proof_scope().unwrap().path(),
            &[0],
            "malformed proof mutation {mutation:?} must not allocate a root"
        );
        assert_eq!(
            collection.references()[0].ordinal(),
            1,
            "malformed proof mutation {mutation:?} must consume no ordinal"
        );
    }
}

#[test]
fn source_collector_rejects_every_malformed_compact_label_edge_but_keeps_statement_ordinal() {
    let source_id = source_id();
    for mutation in [
        CompactLabelMutation::MissingIdentifier,
        CompactLabelMutation::WrongIdentifierKind,
        CompactLabelMutation::MissingColon,
        CompactLabelMutation::WrongColon,
        CompactLabelMutation::RecoveredIdentifier,
        CompactLabelMutation::RecoveredColon,
        CompactLabelMutation::AdditionalProposition,
    ] {
        let collection = collect_fixture(&malformed_compact_label_ast(source_id, mutation));
        assert_eq!(
            collection
                .projections()
                .iter()
                .map(LabelProjection::primary_spelling)
                .collect::<Vec<_>>(),
            vec!["Sentinel"],
            "malformed compact label {mutation:?} must emit no projection"
        );
        assert_eq!(
            collection
                .references()
                .iter()
                .map(|reference| (reference.site().spelling(), reference.ordinal()))
                .collect::<Vec<_>>(),
            vec![("MutRef", 1), ("Sentinel", 2)],
            "malformed label {mutation:?} must not reject its supported statement"
        );
    }
}

#[test]
fn source_collector_rejects_every_malformed_simple_reference_edge() {
    let source_id = source_id();
    for mutation in [
        ReferenceMutation::MissingIdentifier,
        ReferenceMutation::WrongIdentifierKind,
        ReferenceMutation::AdditionalChild,
        ReferenceMutation::RecoveredIdentifier,
    ] {
        let collection = collect_fixture(&malformed_reference_ast(source_id, mutation));
        assert_eq!(
            collection
                .projections()
                .iter()
                .map(LabelProjection::primary_spelling)
                .collect::<Vec<_>>(),
            vec!["MutLabel", "Sentinel"]
        );
        assert_eq!(
            collection
                .references()
                .iter()
                .map(|reference| (reference.site().spelling(), reference.ordinal()))
                .collect::<Vec<_>>(),
            vec![("Sentinel", 2)],
            "malformed reference {mutation:?} must emit no row"
        );
    }
}

#[test]
fn source_collector_default_denies_relocated_wrapped_and_computation_edges() {
    let source_id = source_id();
    for mutation in [
        JustificationMutation::WrongFirstToken,
        JustificationMutation::RecoveredFirstToken,
        JustificationMutation::AdditionalReferenceList,
        JustificationMutation::ComputationInstead,
        JustificationMutation::WrappedJustification,
        JustificationMutation::WrappedReferenceList,
        JustificationMutation::RelocatedIntoProposition,
    ] {
        let collection = collect_fixture(&malformed_justification_edge_ast(source_id, mutation));
        assert_eq!(
            collection
                .projections()
                .iter()
                .map(LabelProjection::primary_spelling)
                .collect::<Vec<_>>(),
            vec!["MutLabel", "Sentinel"]
        );
        assert_eq!(
            collection
                .references()
                .iter()
                .map(|reference| (reference.site().spelling(), reference.ordinal()))
                .collect::<Vec<_>>(),
            vec![("Sentinel", 2)],
            "rejected edge {mutation:?} must emit no reference and consume no extra ordinal"
        );
    }

    let relocated_proof = collect_fixture(&malformed_justification_edge_ast(
        source_id,
        JustificationMutation::ProofRelocatedIntoProposition,
    ));
    assert_eq!(
        relocated_proof
            .projections()
            .iter()
            .map(LabelProjection::primary_spelling)
            .collect::<Vec<_>>(),
        vec!["MutLabel", "Sentinel"]
    );
    assert_eq!(
        relocated_proof.projections()[0].visible_after_ordinal(),
        Some(1),
        "a proof below Proposition must not be descended or delay visibility"
    );
    assert_eq!(
        relocated_proof
            .references()
            .iter()
            .map(|reference| (reference.site().spelling(), reference.ordinal()))
            .collect::<Vec<_>>(),
        vec![("MutRef", 1), ("Sentinel", 2)]
    );

    let conclusion = collect_fixture(&malformed_justification_edge_ast(
        source_id,
        JustificationMutation::ConclusionLabel,
    ));
    assert_eq!(
        conclusion
            .projections()
            .iter()
            .map(LabelProjection::primary_spelling)
            .collect::<Vec<_>>(),
        vec!["Sentinel"],
        "ConclusionStatement proposition labels are excluded"
    );
    assert_eq!(
        conclusion
            .references()
            .iter()
            .map(|reference| (reference.site().spelling(), reference.ordinal()))
            .collect::<Vec<_>>(),
        vec![("MutRef", 1), ("Sentinel", 2)]
    );
}

#[test]
fn source_collector_consumes_supported_unlabelled_statement_ordinals_only() {
    let source_id = source_id();
    let mut builder = CollectorAstBuilder::new(source_id);
    let compact_proposition = builder.proposition(None, None, true);
    let compact = builder.compact(compact_proposition, None);
    let conclusion_proposition = builder.proposition(None, None, true);
    let conclusion = builder.conclusion(conclusion_proposition, None, None);
    let sentinel = builder.valid_statement("Sentinel", Some("Sentinel"), false);
    let theorem = builder.valid_theorem("Owner", vec![compact, conclusion, sentinel]);
    let collection = collect_fixture(&builder.finish_items(vec![theorem]));

    assert_eq!(collection.projections().len(), 1);
    assert_eq!(collection.projections()[0].visible_after_ordinal(), Some(3));
    assert_eq!(
        collection
            .references()
            .iter()
            .map(|reference| (reference.site().spelling(), reference.ordinal()))
            .collect::<Vec<_>>(),
        vec![("Sentinel", 3)]
    );
}

#[test]
fn source_collector_accepts_two_child_label_and_never_descends_semantic_children() {
    let source_id = source_id();
    let mut builder = CollectorAstBuilder::new(source_id);
    let two_child = builder.proposition(Some("TwoChild"), Some(":"), false);
    let statement = builder.compact(two_child, None);
    let theorem = builder.valid_theorem("Owner", vec![statement]);
    let ast = builder.finish_items(vec![theorem]);
    let collection = collect_fixture(&ast);
    assert_eq!(collection.projections()[0].primary_spelling(), "TwoChild");
    assert!(collection.references().is_empty());

    let mut builder = CollectorAstBuilder::new(source_id);
    let hidden_reference = builder.reference("Hidden", false, false);
    let semantic = builder.node(SurfaceNodeKind::FormulaExpression, vec![hidden_reference]);
    let label = builder.token(SurfaceTokenKind::Identifier, "Visible");
    let colon = builder.token(SurfaceTokenKind::ReservedSymbol, ":");
    let proposition = builder.node(SurfaceNodeKind::Proposition, vec![label, colon, semantic]);
    let statement = builder.compact(proposition, None);
    let theorem = builder.valid_theorem("Owner", vec![statement]);
    let ast = builder.finish_items(vec![theorem]);
    let collection = collect_fixture(&ast);
    assert_eq!(collection.projections()[0].primary_spelling(), "Visible");
    assert!(collection.references().is_empty());
}

#[test]
fn source_collector_identity_is_stable_under_formatting_owner_and_order_mutations() {
    let source_id = source_id();
    let base = collect_fixture(&two_theorem_identity_ast(
        source_id,
        0,
        "FirstOwner",
        "StableOwner",
        false,
    ));
    let formatted = collect_fixture(&two_theorem_identity_ast(
        source_id,
        500,
        "FirstOwner",
        "StableOwner",
        false,
    ));
    assert_eq!(
        base.projections()
            .iter()
            .map(|projection| projection.origin_path().as_str())
            .collect::<Vec<_>>(),
        formatted
            .projections()
            .iter()
            .map(|projection| projection.origin_path().as_str())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        base.references()
            .iter()
            .map(|reference| reference.site().spelling())
            .collect::<Vec<_>>(),
        vec!["FirstRef", "SecondRef"]
    );

    let renamed = collect_fixture(&two_theorem_identity_ast(
        source_id,
        0,
        "RenamedFirstOwner",
        "StableOwner",
        false,
    ));
    let base_stable = base
        .projections()
        .iter()
        .find(|projection| projection.primary_spelling() == "SecondLabel")
        .unwrap();
    let renamed_stable = renamed
        .projections()
        .iter()
        .find(|projection| projection.primary_spelling() == "SecondLabel")
        .unwrap();
    assert_eq!(base_stable.origin_path(), renamed_stable.origin_path());

    let reordered = collect_fixture(&two_theorem_identity_ast(
        source_id,
        0,
        "FirstOwner",
        "StableOwner",
        true,
    ));
    assert_eq!(
        reordered
            .projections()
            .iter()
            .map(LabelProjection::primary_spelling)
            .collect::<Vec<_>>(),
        vec!["SecondLabel", "FirstLabel"]
    );
    assert_eq!(
        reordered
            .references()
            .iter()
            .map(|reference| reference.site().spelling())
            .collect::<Vec<_>>(),
        vec!["SecondRef", "FirstRef"]
    );
    for spelling in ["FirstLabel", "SecondLabel"] {
        let before = base
            .projections()
            .iter()
            .find(|projection| projection.primary_spelling() == spelling)
            .unwrap();
        let after = reordered
            .projections()
            .iter()
            .find(|projection| projection.primary_spelling() == spelling)
            .unwrap();
        assert_eq!(before.origin_path(), after.origin_path());
    }
}

#[test]
fn source_collector_identity_tracks_proof_topology_and_occurrence_only() {
    let source_id = source_id();
    let root = collect_fixture(&label_topology_ast(source_id, false));
    let nested = collect_fixture(&label_topology_ast(source_id, true));
    assert!(
        root.projections()[0]
            .origin_path()
            .as_str()
            .contains("|proof-path=0:|")
    );
    assert!(
        nested.projections()[0]
            .origin_path()
            .as_str()
            .contains("|proof-path=1:0|")
    );
    assert_ne!(
        root.projections()[0].origin_path(),
        nested.projections()[0].origin_path()
    );

    let repeated = collect_fixture(&same_scope_labels_ast(source_id));
    assert_eq!(repeated.projections().len(), 2);
    assert!(
        repeated.projections()[0]
            .origin_path()
            .as_str()
            .ends_with("|label-occurrence=0")
    );
    assert!(
        repeated.projections()[1]
            .origin_path()
            .as_str()
            .ends_with("|label-occurrence=1")
    );
    assert_ne!(
        repeated.projections()[0].origin_path(),
        repeated.projections()[1].origin_path()
    );
}

#[derive(Clone, Copy)]
enum ValidationAstMutation {
    None,
    Kind,
    Children,
    Range,
    Recovery,
    Root,
}

fn validation_ast(source_id: SourceId, mutation: ValidationAstMutation) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let token = if matches!(mutation, ValidationAstMutation::Recovery) {
        builder.add_recovered_token(
            SurfaceTokenKind::ReservedWord,
            "token",
            range(source_id, 0, 5),
        )
    } else {
        builder.add_token(
            SurfaceTokenKind::ReservedWord,
            "token",
            range(source_id, 0, 5),
        )
    };
    let item_kind = if matches!(mutation, ValidationAstMutation::Kind) {
        SurfaceNodeKind::PlaceholderItem
    } else {
        SurfaceNodeKind::ItemList
    };
    let item_children = if matches!(mutation, ValidationAstMutation::Children) {
        vec![token]
    } else {
        Vec::new()
    };
    let item_range = if matches!(mutation, ValidationAstMutation::Range) {
        range(source_id, 10, 11)
    } else {
        range(source_id, 0, 5)
    };
    let item_list = builder.add_node(item_kind, item_range, item_children);
    let compilation = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 0, 5),
        vec![item_list],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 5),
        vec![token, compilation],
    );
    let root = if matches!(mutation, ValidationAstMutation::Root) {
        compilation
    } else {
        root
    };
    builder.finish(Some(root), None)
}

fn proof_label_surface_error_class(error: ProofLabelSourceCollectionError) -> &'static str {
    match error {
        ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::SourceMismatch,
        ) => "source",
        ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::ModuleMismatch,
        ) => "module",
        ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::NodeCountMismatch,
        ) => "count",
        ProofLabelSourceCollectionError::SurfaceArena(SurfaceResolvedArenaError::RootMismatch) => {
            "root"
        }
        ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::NodeKindMismatch { .. },
        ) => "kind",
        ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::ChildListMismatch { .. },
        ) => "children",
        ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::RangeMismatch { .. },
        ) => "range",
        ProofLabelSourceCollectionError::SurfaceArena(
            SurfaceResolvedArenaError::RecoveryMismatch { .. },
        ) => "recovery",
        _ => "other",
    }
}

fn assert_collector_rejects_stale_arena(
    ast: &SurfaceAst,
    module: &ModuleId,
    contribution: SourceContributionId,
    resolved: &SurfaceResolvedArena,
    expected: &str,
) {
    let constructor_error = ProofLabelSourceCollector::new(
        ast,
        module,
        NamespacePath::new("main"),
        contribution,
        resolved,
    )
    .err()
    .expect("constructor must reject stale arena");
    assert_eq!(proof_label_surface_error_class(constructor_error), expected);

    let collector = ProofLabelSourceCollector {
        ast,
        namespace: NamespacePath::new("main"),
        contribution,
        resolved,
    };
    let collect_error = collector
        .collect()
        .expect_err("collect must revalidate stale arena");
    assert_eq!(proof_label_surface_error_class(collect_error), expected);
}

#[test]
fn source_collector_forwards_r032a_source_node_shape_range_and_recovery_errors() {
    let (source_id, other_source_id) = distinct_source_ids();
    let module = module_id("pkg", "main");
    let base = validation_ast(source_id, ValidationAstMutation::None);
    let resolved = SurfaceResolvedArena::lower(&base, &module).unwrap();
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), source_id, 0);

    let cases = [
        (
            validation_ast(other_source_id, ValidationAstMutation::None),
            "source",
        ),
        (
            validation_ast(source_id, ValidationAstMutation::Kind),
            "kind",
        ),
        (
            validation_ast(source_id, ValidationAstMutation::Children),
            "children",
        ),
        (
            validation_ast(source_id, ValidationAstMutation::Range),
            "range",
        ),
        (
            validation_ast(source_id, ValidationAstMutation::Recovery),
            "recovery",
        ),
        (
            validation_ast(source_id, ValidationAstMutation::Root),
            "root",
        ),
    ];
    for (ast, expected) in &cases {
        assert_collector_rejects_stale_arena(ast, &module, contribution, &resolved, expected);
    }

    let wrong_module = module_id("pkg", "other");
    let error = ProofLabelSourceCollector::new(
        &base,
        &wrong_module,
        NamespacePath::new("main"),
        contribution,
        &resolved,
    )
    .err()
    .expect("wrong module must be rejected");
    assert!(
        error
            .to_string()
            .contains("invalid proof-label surface arena")
    );
    assert!(std::error::Error::source(&error).is_some());
    assert_eq!(proof_label_surface_error_class(error), "module");

    let larger = b5c_inner_ast(source_id);
    assert_collector_rejects_stale_arena(&larger, &module, contribution, &resolved, "count");
}

fn b5c_inner_ast(source_id: SourceId) -> SurfaceAst {
    let tokens = vec![
        (SurfaceTokenKind::ReservedWord, "theorem", 0, 7),
        (
            SurfaceTokenKind::Identifier,
            "ProofLabelInnerToOuterConfinement",
            8,
            41,
        ),
        (SurfaceTokenKind::ReservedSymbol, ":", 41, 42),
        (SurfaceTokenKind::ReservedWord, "thesis", 43, 49),
        (SurfaceTokenKind::ReservedWord, "proof", 50, 55),
        (SurfaceTokenKind::ReservedWord, "thus", 58, 62),
        (SurfaceTokenKind::ReservedWord, "thesis", 63, 69),
        (SurfaceTokenKind::ReservedWord, "proof", 70, 75),
        (SurfaceTokenKind::Identifier, "A", 80, 81),
        (SurfaceTokenKind::ReservedSymbol, ":", 81, 82),
        (SurfaceTokenKind::ReservedWord, "thesis", 83, 89),
        (SurfaceTokenKind::ReservedWord, "proof", 90, 95),
        (SurfaceTokenKind::ReservedWord, "thus", 102, 106),
        (SurfaceTokenKind::ReservedWord, "thesis", 107, 113),
        (SurfaceTokenKind::ReservedSymbol, ";", 113, 114),
        (SurfaceTokenKind::ReservedWord, "end", 119, 122),
        (SurfaceTokenKind::ReservedSymbol, ";", 122, 123),
        (SurfaceTokenKind::ReservedWord, "thus", 128, 132),
        (SurfaceTokenKind::ReservedWord, "thesis", 133, 139),
        (SurfaceTokenKind::ReservedSymbol, ";", 139, 140),
        (SurfaceTokenKind::ReservedWord, "end", 143, 146),
        (SurfaceTokenKind::ReservedSymbol, ";", 146, 147),
        (SurfaceTokenKind::ReservedWord, "thus", 150, 154),
        (SurfaceTokenKind::ReservedWord, "thesis", 155, 161),
        (SurfaceTokenKind::ReservedWord, "by", 162, 164),
        (SurfaceTokenKind::Identifier, "A", 165, 166),
        (SurfaceTokenKind::ReservedSymbol, ";", 166, 167),
        (SurfaceTokenKind::ReservedWord, "end", 168, 171),
        (SurfaceTokenKind::ReservedSymbol, ";", 171, 172),
    ];
    let nodes = vec![
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            43,
            49,
            &[3],
        ),
        node(SurfaceNodeKind::FormulaExpression, 43, 49, &[29]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            63,
            69,
            &[6],
        ),
        node(SurfaceNodeKind::FormulaExpression, 63, 69, &[31]),
        node(SurfaceNodeKind::Proposition, 63, 69, &[32]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            83,
            89,
            &[10],
        ),
        node(SurfaceNodeKind::FormulaExpression, 83, 89, &[34]),
        node(SurfaceNodeKind::Proposition, 80, 89, &[8, 9, 35]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            107,
            113,
            &[13],
        ),
        node(SurfaceNodeKind::FormulaExpression, 107, 113, &[37]),
        node(SurfaceNodeKind::Proposition, 107, 113, &[38]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            102,
            114,
            &[12, 39, 14],
        ),
        node(SurfaceNodeKind::ProofBlock, 90, 122, &[11, 40, 15]),
        node(SurfaceNodeKind::CompactStatement, 80, 123, &[36, 41, 16]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            133,
            139,
            &[18],
        ),
        node(SurfaceNodeKind::FormulaExpression, 133, 139, &[43]),
        node(SurfaceNodeKind::Proposition, 133, 139, &[44]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            128,
            140,
            &[17, 45, 19],
        ),
        node(SurfaceNodeKind::ProofBlock, 70, 146, &[7, 42, 46, 20]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            58,
            147,
            &[5, 33, 47, 21],
        ),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            155,
            161,
            &[23],
        ),
        node(SurfaceNodeKind::FormulaExpression, 155, 161, &[49]),
        node(SurfaceNodeKind::Proposition, 155, 161, &[50]),
        node(SurfaceNodeKind::Reference, 165, 166, &[25]),
        node(SurfaceNodeKind::ReferenceList, 165, 166, &[52]),
        node(SurfaceNodeKind::JustificationClause, 162, 166, &[24, 53]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            150,
            167,
            &[22, 51, 54, 26],
        ),
        node(SurfaceNodeKind::ProofBlock, 50, 171, &[4, 48, 55, 27]),
        node(SurfaceNodeKind::TheoremItem, 0, 172, &[0, 1, 2, 30, 56, 28]),
        node(SurfaceNodeKind::ItemList, 0, 172, &[57]),
        node(SurfaceNodeKind::CompilationUnit, 0, 172, &[58]),
        node(
            SurfaceNodeKind::Root,
            0,
            172,
            &(0..=28).chain(std::iter::once(59)).collect::<Vec<_>>(),
        ),
    ];
    build_surface_ast(source_id, tokens, nodes)
}

fn b5c_sibling_ast(source_id: SourceId) -> SurfaceAst {
    let tokens = vec![
        (SurfaceTokenKind::ReservedWord, "theorem", 0, 7),
        (
            SurfaceTokenKind::Identifier,
            "ProofLabelSiblingConfinement",
            8,
            36,
        ),
        (SurfaceTokenKind::ReservedSymbol, ":", 36, 37),
        (SurfaceTokenKind::ReservedWord, "thesis", 38, 44),
        (SurfaceTokenKind::ReservedWord, "proof", 45, 50),
        (SurfaceTokenKind::ReservedWord, "thus", 53, 57),
        (SurfaceTokenKind::ReservedWord, "thesis", 58, 64),
        (SurfaceTokenKind::ReservedWord, "proof", 65, 70),
        (SurfaceTokenKind::Identifier, "A", 75, 76),
        (SurfaceTokenKind::ReservedSymbol, ":", 76, 77),
        (SurfaceTokenKind::ReservedWord, "thesis", 78, 84),
        (SurfaceTokenKind::ReservedWord, "proof", 85, 90),
        (SurfaceTokenKind::ReservedWord, "thus", 97, 101),
        (SurfaceTokenKind::ReservedWord, "thesis", 102, 108),
        (SurfaceTokenKind::ReservedSymbol, ";", 108, 109),
        (SurfaceTokenKind::ReservedWord, "end", 114, 117),
        (SurfaceTokenKind::ReservedSymbol, ";", 117, 118),
        (SurfaceTokenKind::ReservedWord, "thus", 123, 127),
        (SurfaceTokenKind::ReservedWord, "thesis", 128, 134),
        (SurfaceTokenKind::ReservedSymbol, ";", 134, 135),
        (SurfaceTokenKind::ReservedWord, "end", 138, 141),
        (SurfaceTokenKind::ReservedSymbol, ";", 141, 142),
        (SurfaceTokenKind::ReservedWord, "thus", 145, 149),
        (SurfaceTokenKind::ReservedWord, "thesis", 150, 156),
        (SurfaceTokenKind::ReservedWord, "proof", 157, 162),
        (SurfaceTokenKind::ReservedWord, "thus", 167, 171),
        (SurfaceTokenKind::ReservedWord, "thesis", 172, 178),
        (SurfaceTokenKind::ReservedWord, "by", 179, 181),
        (SurfaceTokenKind::Identifier, "A", 182, 183),
        (SurfaceTokenKind::ReservedSymbol, ";", 183, 184),
        (SurfaceTokenKind::ReservedWord, "end", 187, 190),
        (SurfaceTokenKind::ReservedSymbol, ";", 190, 191),
        (SurfaceTokenKind::ReservedWord, "end", 192, 195),
        (SurfaceTokenKind::ReservedSymbol, ";", 195, 196),
    ];
    let nodes = vec![
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            38,
            44,
            &[3],
        ),
        node(SurfaceNodeKind::FormulaExpression, 38, 44, &[34]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            58,
            64,
            &[6],
        ),
        node(SurfaceNodeKind::FormulaExpression, 58, 64, &[36]),
        node(SurfaceNodeKind::Proposition, 58, 64, &[37]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            78,
            84,
            &[10],
        ),
        node(SurfaceNodeKind::FormulaExpression, 78, 84, &[39]),
        node(SurfaceNodeKind::Proposition, 75, 84, &[8, 9, 40]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            102,
            108,
            &[13],
        ),
        node(SurfaceNodeKind::FormulaExpression, 102, 108, &[42]),
        node(SurfaceNodeKind::Proposition, 102, 108, &[43]),
        node(SurfaceNodeKind::ConclusionStatement, 97, 109, &[12, 44, 14]),
        node(SurfaceNodeKind::ProofBlock, 85, 117, &[11, 45, 15]),
        node(SurfaceNodeKind::CompactStatement, 75, 118, &[41, 46, 16]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            128,
            134,
            &[18],
        ),
        node(SurfaceNodeKind::FormulaExpression, 128, 134, &[48]),
        node(SurfaceNodeKind::Proposition, 128, 134, &[49]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            123,
            135,
            &[17, 50, 19],
        ),
        node(SurfaceNodeKind::ProofBlock, 65, 141, &[7, 47, 51, 20]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            53,
            142,
            &[5, 38, 52, 21],
        ),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            150,
            156,
            &[23],
        ),
        node(SurfaceNodeKind::FormulaExpression, 150, 156, &[54]),
        node(SurfaceNodeKind::Proposition, 150, 156, &[55]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            172,
            178,
            &[26],
        ),
        node(SurfaceNodeKind::FormulaExpression, 172, 178, &[57]),
        node(SurfaceNodeKind::Proposition, 172, 178, &[58]),
        node(SurfaceNodeKind::Reference, 182, 183, &[28]),
        node(SurfaceNodeKind::ReferenceList, 182, 183, &[60]),
        node(SurfaceNodeKind::JustificationClause, 179, 183, &[27, 61]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            167,
            184,
            &[25, 59, 62, 29],
        ),
        node(SurfaceNodeKind::ProofBlock, 157, 190, &[24, 63, 30]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            145,
            191,
            &[22, 56, 64, 31],
        ),
        node(SurfaceNodeKind::ProofBlock, 45, 195, &[4, 53, 65, 32]),
        node(SurfaceNodeKind::TheoremItem, 0, 196, &[0, 1, 2, 35, 66, 33]),
        node(SurfaceNodeKind::ItemList, 0, 196, &[67]),
        node(SurfaceNodeKind::CompilationUnit, 0, 196, &[68]),
        node(
            SurfaceNodeKind::Root,
            0,
            196,
            &(0..=33).chain(std::iter::once(69)).collect::<Vec<_>>(),
        ),
    ];
    build_surface_ast(source_id, tokens, nodes)
}

fn own_proof_and_post_completion_ast(source_id: SourceId) -> SurfaceAst {
    let tokens = vec![
        (SurfaceTokenKind::ReservedWord, "theorem", 0, 7),
        (SurfaceTokenKind::Identifier, "Owner", 8, 13),
        (SurfaceTokenKind::ReservedSymbol, ":", 13, 14),
        (SurfaceTokenKind::ReservedWord, "thesis", 15, 21),
        (SurfaceTokenKind::ReservedWord, "proof", 22, 27),
        (SurfaceTokenKind::Identifier, "A", 28, 29),
        (SurfaceTokenKind::ReservedSymbol, ":", 29, 30),
        (SurfaceTokenKind::ReservedWord, "thesis", 31, 37),
        (SurfaceTokenKind::ReservedWord, "proof", 38, 43),
        (SurfaceTokenKind::ReservedWord, "thus", 44, 48),
        (SurfaceTokenKind::ReservedWord, "thesis", 49, 55),
        (SurfaceTokenKind::ReservedWord, "by", 56, 58),
        (SurfaceTokenKind::Identifier, "A", 59, 60),
        (SurfaceTokenKind::ReservedSymbol, ";", 60, 61),
        (SurfaceTokenKind::ReservedWord, "end", 62, 65),
        (SurfaceTokenKind::ReservedSymbol, ";", 65, 66),
        (SurfaceTokenKind::ReservedWord, "thus", 67, 71),
        (SurfaceTokenKind::ReservedWord, "thesis", 72, 78),
        (SurfaceTokenKind::ReservedWord, "by", 79, 81),
        (SurfaceTokenKind::Identifier, "A", 82, 83),
        (SurfaceTokenKind::ReservedSymbol, ";", 83, 84),
        (SurfaceTokenKind::ReservedWord, "end", 85, 88),
        (SurfaceTokenKind::ReservedSymbol, ";", 88, 89),
    ];
    let nodes = vec![
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            15,
            21,
            &[3],
        ),
        node(SurfaceNodeKind::FormulaExpression, 15, 21, &[23]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            31,
            37,
            &[7],
        ),
        node(SurfaceNodeKind::FormulaExpression, 31, 37, &[25]),
        node(SurfaceNodeKind::Proposition, 28, 37, &[5, 6, 26]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            49,
            55,
            &[10],
        ),
        node(SurfaceNodeKind::FormulaExpression, 49, 55, &[28]),
        node(SurfaceNodeKind::Proposition, 49, 55, &[29]),
        node(SurfaceNodeKind::Reference, 59, 60, &[12]),
        node(SurfaceNodeKind::ReferenceList, 59, 60, &[31]),
        node(SurfaceNodeKind::JustificationClause, 56, 60, &[11, 32]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            44,
            61,
            &[9, 30, 33, 13],
        ),
        node(SurfaceNodeKind::ProofBlock, 38, 65, &[8, 34, 14]),
        node(SurfaceNodeKind::CompactStatement, 28, 66, &[27, 35, 15]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            72,
            78,
            &[17],
        ),
        node(SurfaceNodeKind::FormulaExpression, 72, 78, &[37]),
        node(SurfaceNodeKind::Proposition, 72, 78, &[38]),
        node(SurfaceNodeKind::Reference, 82, 83, &[19]),
        node(SurfaceNodeKind::ReferenceList, 82, 83, &[40]),
        node(SurfaceNodeKind::JustificationClause, 79, 83, &[18, 41]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            67,
            84,
            &[16, 39, 42, 20],
        ),
        node(SurfaceNodeKind::ProofBlock, 22, 88, &[4, 36, 43, 21]),
        node(SurfaceNodeKind::TheoremItem, 0, 89, &[0, 1, 2, 24, 44, 22]),
        node(SurfaceNodeKind::ItemList, 0, 89, &[45]),
        node(SurfaceNodeKind::CompilationUnit, 0, 89, &[46]),
        node(
            SurfaceNodeKind::Root,
            0,
            89,
            &(0..=22).chain(std::iter::once(47)).collect::<Vec<_>>(),
        ),
    ];
    build_surface_ast(source_id, tokens, nodes)
}

fn enclosing_to_child_ast(source_id: SourceId) -> SurfaceAst {
    let tokens = vec![
        (SurfaceTokenKind::ReservedWord, "theorem", 0, 7),
        (SurfaceTokenKind::Identifier, "Owner", 8, 13),
        (SurfaceTokenKind::ReservedSymbol, ":", 13, 14),
        (SurfaceTokenKind::ReservedWord, "thesis", 15, 21),
        (SurfaceTokenKind::ReservedWord, "proof", 22, 27),
        (SurfaceTokenKind::Identifier, "A", 28, 29),
        (SurfaceTokenKind::ReservedSymbol, ":", 29, 30),
        (SurfaceTokenKind::ReservedWord, "thesis", 31, 37),
        (SurfaceTokenKind::ReservedSymbol, ";", 37, 38),
        (SurfaceTokenKind::ReservedWord, "thus", 39, 43),
        (SurfaceTokenKind::ReservedWord, "thesis", 44, 50),
        (SurfaceTokenKind::ReservedWord, "proof", 51, 56),
        (SurfaceTokenKind::ReservedWord, "thus", 57, 61),
        (SurfaceTokenKind::ReservedWord, "thesis", 62, 68),
        (SurfaceTokenKind::ReservedWord, "by", 69, 71),
        (SurfaceTokenKind::Identifier, "A", 72, 73),
        (SurfaceTokenKind::ReservedSymbol, ";", 73, 74),
        (SurfaceTokenKind::ReservedWord, "end", 75, 78),
        (SurfaceTokenKind::ReservedSymbol, ";", 78, 79),
        (SurfaceTokenKind::ReservedWord, "noise", 80, 85),
        (SurfaceTokenKind::ReservedWord, "end", 86, 89),
        (SurfaceTokenKind::ReservedSymbol, ";", 89, 90),
    ];
    let nodes = vec![
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            15,
            21,
            &[3],
        ),
        node(SurfaceNodeKind::FormulaExpression, 15, 21, &[22]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            31,
            37,
            &[7],
        ),
        node(SurfaceNodeKind::FormulaExpression, 31, 37, &[24]),
        node(SurfaceNodeKind::Proposition, 28, 37, &[5, 6, 25]),
        node(SurfaceNodeKind::CompactStatement, 28, 38, &[26, 8]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            44,
            50,
            &[10],
        ),
        node(SurfaceNodeKind::FormulaExpression, 44, 50, &[28]),
        node(SurfaceNodeKind::Proposition, 44, 50, &[29]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            62,
            68,
            &[13],
        ),
        node(SurfaceNodeKind::FormulaExpression, 62, 68, &[31]),
        node(SurfaceNodeKind::Proposition, 62, 68, &[32]),
        node(SurfaceNodeKind::Reference, 72, 73, &[15]),
        node(SurfaceNodeKind::ReferenceList, 72, 73, &[34]),
        node(SurfaceNodeKind::JustificationClause, 69, 73, &[14, 35]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            57,
            74,
            &[12, 33, 36, 16],
        ),
        node(SurfaceNodeKind::ProofBlock, 51, 78, &[11, 37, 17]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            39,
            79,
            &[9, 30, 38, 18],
        ),
        node(SurfaceNodeKind::ProofBlock, 22, 89, &[4, 27, 19, 39, 20]),
        node(SurfaceNodeKind::TheoremItem, 0, 90, &[0, 1, 2, 23, 40, 21]),
        node(SurfaceNodeKind::ItemList, 0, 90, &[41]),
        node(SurfaceNodeKind::CompilationUnit, 0, 90, &[42]),
        node(
            SurfaceNodeKind::Root,
            0,
            90,
            &(0..=21).chain(std::iter::once(43)).collect::<Vec<_>>(),
        ),
    ];
    build_surface_ast(source_id, tokens, nodes)
}

fn mixed_reference_ast(source_id: SourceId) -> SurfaceAst {
    let tokens = vec![
        (SurfaceTokenKind::ReservedWord, "theorem", 0, 7),
        (SurfaceTokenKind::Identifier, "Owner", 8, 13),
        (SurfaceTokenKind::ReservedSymbol, ":", 13, 14),
        (SurfaceTokenKind::ReservedWord, "thesis", 15, 21),
        (SurfaceTokenKind::ReservedWord, "proof", 22, 27),
        (SurfaceTokenKind::ReservedWord, "thus", 28, 32),
        (SurfaceTokenKind::ReservedWord, "thesis", 33, 39),
        (SurfaceTokenKind::ReservedWord, "by", 40, 42),
        (SurfaceTokenKind::Identifier, "A", 43, 44),
        (SurfaceTokenKind::ReservedSymbol, ",", 44, 45),
        (SurfaceTokenKind::Identifier, "Q", 46, 47),
        (SurfaceTokenKind::ReservedSymbol, ",", 47, 48),
        (SurfaceTokenKind::Identifier, "G", 49, 50),
        (SurfaceTokenKind::ReservedSymbol, ",", 50, 51),
        (SurfaceTokenKind::Identifier, "Bulk", 52, 56),
        (SurfaceTokenKind::ReservedSymbol, ",", 56, 57),
        (SurfaceTokenKind::Identifier, "Templated", 58, 67),
        (SurfaceTokenKind::Identifier, "Arg", 68, 71),
        (SurfaceTokenKind::ReservedSymbol, ",", 71, 72),
        (SurfaceTokenKind::Identifier, "B", 73, 74),
        (SurfaceTokenKind::ReservedSymbol, ";", 74, 75),
        (SurfaceTokenKind::ReservedWord, "noise", 76, 81),
        (SurfaceTokenKind::ReservedWord, "end", 82, 85),
        (SurfaceTokenKind::ReservedSymbol, ";", 85, 86),
    ];
    let nodes = vec![
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            15,
            21,
            &[3],
        ),
        node(SurfaceNodeKind::FormulaExpression, 15, 21, &[24]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            33,
            39,
            &[6],
        ),
        node(SurfaceNodeKind::FormulaExpression, 33, 39, &[26]),
        node(SurfaceNodeKind::Proposition, 33, 39, &[27]),
        node(SurfaceNodeKind::Reference, 43, 44, &[8]),
        node(SurfaceNodeKind::QualifiedReference, 46, 47, &[10]),
        node(SurfaceNodeKind::GroupedReference, 49, 50, &[12]),
        node(SurfaceNodeKind::BulkReference, 52, 56, &[14]),
        node(SurfaceNodeKind::TemplateArgument, 68, 71, &[17]),
        node(SurfaceNodeKind::TemplateArguments, 68, 71, &[33]),
        node(SurfaceNodeKind::Reference, 58, 71, &[16, 34]),
        node(SurfaceNodeKind::Reference, 73, 74, &[19]),
        node(
            SurfaceNodeKind::ReferenceList,
            43,
            74,
            &[29, 9, 30, 11, 31, 13, 32, 15, 35, 18, 36],
        ),
        node(SurfaceNodeKind::JustificationClause, 40, 74, &[7, 37]),
        node(SurfaceNodeKind::CompactStatement, 28, 75, &[5, 28, 38, 20]),
        node(SurfaceNodeKind::NowStatement, 76, 81, &[]),
        node(SurfaceNodeKind::ProofBlock, 22, 85, &[4, 39, 21, 40, 22]),
        node(SurfaceNodeKind::TheoremItem, 0, 86, &[0, 1, 2, 25, 41, 23]),
        node(SurfaceNodeKind::ItemList, 0, 86, &[42]),
        node(SurfaceNodeKind::CompilationUnit, 0, 86, &[43]),
        node(
            SurfaceNodeKind::Root,
            0,
            86,
            &(0..=23).chain(std::iter::once(44)).collect::<Vec<_>>(),
        ),
    ];
    build_surface_ast(source_id, tokens, nodes)
}

fn cross_theorem_ast(source_id: SourceId) -> SurfaceAst {
    let tokens = vec![
        (SurfaceTokenKind::ReservedWord, "theorem", 0, 7),
        (SurfaceTokenKind::Identifier, "Same", 8, 12),
        (SurfaceTokenKind::ReservedSymbol, ":", 12, 13),
        (SurfaceTokenKind::ReservedWord, "thesis", 14, 20),
        (SurfaceTokenKind::ReservedWord, "proof", 21, 26),
        (SurfaceTokenKind::Identifier, "A", 27, 28),
        (SurfaceTokenKind::ReservedSymbol, ":", 28, 29),
        (SurfaceTokenKind::ReservedWord, "thesis", 30, 36),
        (SurfaceTokenKind::ReservedSymbol, ";", 36, 37),
        (SurfaceTokenKind::ReservedWord, "end", 38, 41),
        (SurfaceTokenKind::ReservedSymbol, ";", 41, 42),
        (SurfaceTokenKind::ReservedWord, "theorem", 43, 50),
        (SurfaceTokenKind::Identifier, "Same", 51, 55),
        (SurfaceTokenKind::ReservedSymbol, ":", 55, 56),
        (SurfaceTokenKind::ReservedWord, "thesis", 57, 63),
        (SurfaceTokenKind::ReservedWord, "proof", 64, 69),
        (SurfaceTokenKind::ReservedWord, "thus", 70, 74),
        (SurfaceTokenKind::ReservedWord, "thesis", 75, 81),
        (SurfaceTokenKind::ReservedWord, "by", 82, 84),
        (SurfaceTokenKind::Identifier, "A", 85, 86),
        (SurfaceTokenKind::ReservedSymbol, ";", 86, 87),
        (SurfaceTokenKind::Identifier, "A", 88, 89),
        (SurfaceTokenKind::ReservedSymbol, ":", 89, 90),
        (SurfaceTokenKind::ReservedWord, "thesis", 91, 97),
        (SurfaceTokenKind::ReservedSymbol, ";", 97, 98),
        (SurfaceTokenKind::ReservedWord, "end", 99, 102),
        (SurfaceTokenKind::ReservedSymbol, ";", 102, 103),
    ];
    let nodes = vec![
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            14,
            20,
            &[3],
        ),
        node(SurfaceNodeKind::FormulaExpression, 14, 20, &[27]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            30,
            36,
            &[7],
        ),
        node(SurfaceNodeKind::FormulaExpression, 30, 36, &[29]),
        node(SurfaceNodeKind::Proposition, 27, 36, &[5, 6, 30]),
        node(SurfaceNodeKind::CompactStatement, 27, 37, &[31, 8]),
        node(SurfaceNodeKind::ProofBlock, 21, 41, &[4, 32, 9]),
        node(SurfaceNodeKind::TheoremItem, 0, 42, &[0, 1, 2, 28, 33, 10]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            57,
            63,
            &[14],
        ),
        node(SurfaceNodeKind::FormulaExpression, 57, 63, &[35]),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            75,
            81,
            &[17],
        ),
        node(SurfaceNodeKind::FormulaExpression, 75, 81, &[37]),
        node(SurfaceNodeKind::Proposition, 75, 81, &[38]),
        node(SurfaceNodeKind::Reference, 85, 86, &[19]),
        node(SurfaceNodeKind::ReferenceList, 85, 86, &[40]),
        node(SurfaceNodeKind::JustificationClause, 82, 86, &[18, 41]),
        node(
            SurfaceNodeKind::ConclusionStatement,
            70,
            87,
            &[16, 39, 42, 20],
        ),
        node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            91,
            97,
            &[23],
        ),
        node(SurfaceNodeKind::FormulaExpression, 91, 97, &[44]),
        node(SurfaceNodeKind::Proposition, 88, 97, &[21, 22, 45]),
        node(SurfaceNodeKind::CompactStatement, 88, 98, &[46, 24]),
        node(SurfaceNodeKind::ProofBlock, 64, 102, &[15, 43, 47, 25]),
        node(
            SurfaceNodeKind::TheoremItem,
            43,
            103,
            &[11, 12, 13, 36, 48, 26],
        ),
        node(SurfaceNodeKind::ItemList, 0, 103, &[34, 49]),
        node(SurfaceNodeKind::CompilationUnit, 0, 103, &[50]),
        node(
            SurfaceNodeKind::Root,
            0,
            103,
            &(0..=26).chain(std::iter::once(51)).collect::<Vec<_>>(),
        ),
    ];
    build_surface_ast(source_id, tokens, nodes)
}

#[derive(Clone, Copy)]
enum UpperShape {
    MissingCompilation,
    AdditionalCompilation,
    WrongCompilationChild,
    RootWrongStructuralChild,
    TheoremDirectlyUnderRoot,
    MissingItemList,
    AdditionalItemList,
}

fn malformed_upper_ast(source_id: SourceId, shape: UpperShape) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "theorem",
        range(source_id, 0, 7),
    );
    let root = match shape {
        UpperShape::MissingCompilation => {
            builder.add_node(SurfaceNodeKind::Root, range(source_id, 0, 7), vec![token])
        }
        UpperShape::AdditionalCompilation => {
            let first_list =
                builder.add_node(SurfaceNodeKind::ItemList, range(source_id, 0, 7), vec![]);
            let first = builder.add_node(
                SurfaceNodeKind::CompilationUnit,
                range(source_id, 0, 7),
                vec![first_list],
            );
            let second_list =
                builder.add_node(SurfaceNodeKind::ItemList, range(source_id, 0, 7), vec![]);
            let second = builder.add_node(
                SurfaceNodeKind::CompilationUnit,
                range(source_id, 0, 7),
                vec![second_list],
            );
            builder.add_node(
                SurfaceNodeKind::Root,
                range(source_id, 0, 7),
                vec![token, first, second],
            )
        }
        UpperShape::WrongCompilationChild => {
            let theorem =
                builder.add_node(SurfaceNodeKind::TheoremItem, range(source_id, 0, 7), vec![]);
            let compilation = builder.add_node(
                SurfaceNodeKind::CompilationUnit,
                range(source_id, 0, 7),
                vec![theorem],
            );
            builder.add_node(
                SurfaceNodeKind::Root,
                range(source_id, 0, 7),
                vec![token, compilation],
            )
        }
        UpperShape::RootWrongStructuralChild => {
            let item_list =
                builder.add_node(SurfaceNodeKind::ItemList, range(source_id, 0, 7), vec![]);
            let compilation = builder.add_node(
                SurfaceNodeKind::CompilationUnit,
                range(source_id, 0, 7),
                vec![item_list],
            );
            let wrong = builder.add_node(
                SurfaceNodeKind::PlaceholderItem,
                range(source_id, 0, 7),
                vec![],
            );
            builder.add_node(
                SurfaceNodeKind::Root,
                range(source_id, 0, 7),
                vec![token, compilation, wrong],
            )
        }
        UpperShape::TheoremDirectlyUnderRoot => {
            let theorem =
                builder.add_node(SurfaceNodeKind::TheoremItem, range(source_id, 0, 7), vec![]);
            builder.add_node(
                SurfaceNodeKind::Root,
                range(source_id, 0, 7),
                vec![token, theorem],
            )
        }
        UpperShape::MissingItemList => {
            let compilation = builder.add_node(
                SurfaceNodeKind::CompilationUnit,
                range(source_id, 0, 7),
                vec![],
            );
            builder.add_node(
                SurfaceNodeKind::Root,
                range(source_id, 0, 7),
                vec![token, compilation],
            )
        }
        UpperShape::AdditionalItemList => {
            let first = builder.add_node(SurfaceNodeKind::ItemList, range(source_id, 0, 7), vec![]);
            let second =
                builder.add_node(SurfaceNodeKind::ItemList, range(source_id, 0, 7), vec![]);
            let compilation = builder.add_node(
                SurfaceNodeKind::CompilationUnit,
                range(source_id, 0, 7),
                vec![first, second],
            );
            builder.add_node(
                SurfaceNodeKind::Root,
                range(source_id, 0, 7),
                vec![token, compilation],
            )
        }
    };
    builder.finish(Some(root), None)
}

fn recovered_proof_boundary_ast(source_id: SourceId) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let theorem = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "theorem",
        range(source_id, 0, 7),
    );
    let owner = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Owner",
        range(source_id, 8, 13),
    );
    let colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 13, 14),
    );
    let thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 15, 21),
    );
    let proof = builder.add_recovered_token(
        SurfaceTokenKind::ReservedWord,
        "proof",
        range(source_id, 22, 27),
    );
    let label = builder.add_token(SurfaceTokenKind::Identifier, "A", range(source_id, 28, 29));
    let label_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 29, 30),
    );
    let statement_thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 31, 37),
    );
    let statement_end = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 37, 38),
    );
    let end = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "end",
        range(source_id, 39, 42),
    );
    let theorem_end = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 42, 43),
    );
    let theorem_constant = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, 15, 21),
        vec![thesis],
    );
    let theorem_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 15, 21),
        vec![theorem_constant],
    );
    let statement_constant = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, 31, 37),
        vec![statement_thesis],
    );
    let statement_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 31, 37),
        vec![statement_constant],
    );
    let proposition = builder.add_node(
        SurfaceNodeKind::Proposition,
        range(source_id, 28, 37),
        vec![label, label_colon, statement_formula],
    );
    let statement = builder.add_node(
        SurfaceNodeKind::CompactStatement,
        range(source_id, 28, 38),
        vec![proposition, statement_end],
    );
    let proof_block = builder.add_node(
        SurfaceNodeKind::ProofBlock,
        range(source_id, 22, 42),
        vec![proof, statement, end],
    );
    let theorem_item = builder.add_node(
        SurfaceNodeKind::TheoremItem,
        range(source_id, 0, 43),
        vec![
            theorem,
            owner,
            colon,
            theorem_formula,
            proof_block,
            theorem_end,
        ],
    );
    let item_list = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source_id, 0, 43),
        vec![theorem_item],
    );
    let compilation = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 0, 43),
        vec![item_list],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 43),
        vec![
            theorem,
            owner,
            colon,
            thesis,
            proof,
            label,
            label_colon,
            statement_thesis,
            statement_end,
            end,
            theorem_end,
            compilation,
        ],
    );
    builder.finish(Some(root), None)
}

struct TestNode {
    kind: SurfaceNodeKind,
    start: usize,
    end: usize,
    children: Vec<usize>,
}

fn node(kind: SurfaceNodeKind, start: usize, end: usize, children: &[usize]) -> TestNode {
    TestNode {
        kind,
        start,
        end,
        children: children.to_vec(),
    }
}

fn build_surface_ast(
    source_id: SourceId,
    tokens: Vec<(SurfaceTokenKind, &'static str, usize, usize)>,
    nodes: Vec<TestNode>,
) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let mut ids = Vec::new();
    for (kind, text, start, end) in tokens {
        ids.push(builder.add_token(kind, text, range(source_id, start, end)));
    }
    for node in nodes {
        let children = node.children.iter().map(|index| ids[*index]).collect();
        ids.push(builder.add_node(node.kind, range(source_id, node.start, node.end), children));
    }
    builder.finish(ids.last().copied(), None)
}

struct CollectorAstBuilder {
    source_id: SourceId,
    builder: SurfaceAstBuilder,
    base: usize,
    cursor: usize,
}

impl CollectorAstBuilder {
    fn new(source_id: SourceId) -> Self {
        Self::with_offset(source_id, 0)
    }

    fn with_offset(source_id: SourceId, offset: usize) -> Self {
        Self {
            source_id,
            builder: SurfaceAstBuilder::new(source_id),
            base: offset,
            cursor: offset,
        }
    }

    fn token(&mut self, kind: SurfaceTokenKind, text: &str) -> SurfaceBuilderNodeId {
        let start = self.cursor;
        let width = text.len().max(1);
        self.cursor += width + 1;
        self.builder
            .add_token(kind, text, range(self.source_id, start, start + width))
    }

    fn recovered_token(&mut self, kind: SurfaceTokenKind, text: &str) -> SurfaceBuilderNodeId {
        let start = self.cursor;
        let width = text.len().max(1);
        self.cursor += width + 1;
        self.builder
            .add_recovered_token(kind, text, range(self.source_id, start, start + width))
    }

    fn node(
        &mut self,
        kind: SurfaceNodeKind,
        children: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        self.builder.add_node(
            kind,
            range(self.source_id, self.base, self.cursor.max(self.base + 1)),
            children,
        )
    }

    fn formula(&mut self) -> SurfaceBuilderNodeId {
        let thesis = self.token(SurfaceTokenKind::ReservedWord, "thesis");
        let constant = self.node(
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
            vec![thesis],
        );
        self.node(SurfaceNodeKind::FormulaExpression, vec![constant])
    }

    fn proposition(
        &mut self,
        label: Option<&str>,
        colon: Option<&str>,
        include_semantic_child: bool,
    ) -> SurfaceBuilderNodeId {
        let mut children = Vec::new();
        if let Some(label) = label {
            children.push(self.token(SurfaceTokenKind::Identifier, label));
        }
        if let Some(colon) = colon {
            children.push(self.token(SurfaceTokenKind::ReservedSymbol, colon));
        }
        if include_semantic_child {
            let formula = self.formula();
            children.push(formula);
        }
        self.node(SurfaceNodeKind::Proposition, children)
    }

    fn reference(
        &mut self,
        spelling: &str,
        additional_child: bool,
        recovered_identifier: bool,
    ) -> SurfaceBuilderNodeId {
        let identifier = if recovered_identifier {
            self.recovered_token(SurfaceTokenKind::Identifier, spelling)
        } else {
            self.token(SurfaceTokenKind::Identifier, spelling)
        };
        let mut children = vec![identifier];
        if additional_child {
            let arguments = self.node(SurfaceNodeKind::TemplateArguments, Vec::new());
            children.push(arguments);
        }
        self.node(SurfaceNodeKind::Reference, children)
    }

    fn justification(
        &mut self,
        first: (&str, SurfaceTokenKind),
        reference_children: Vec<SurfaceBuilderNodeId>,
        additional_list: bool,
    ) -> SurfaceBuilderNodeId {
        let first = self.token(first.1, first.0);
        let list = self.node(SurfaceNodeKind::ReferenceList, reference_children);
        let mut children = vec![first, list];
        if additional_list {
            let second = self.node(SurfaceNodeKind::ReferenceList, Vec::new());
            children.push(second);
        }
        self.node(SurfaceNodeKind::JustificationClause, children)
    }

    fn compact(
        &mut self,
        proposition: SurfaceBuilderNodeId,
        justification: Option<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        let mut children = vec![proposition];
        if let Some(justification) = justification {
            children.push(justification);
        }
        let semicolon = self.token(SurfaceTokenKind::ReservedSymbol, ";");
        children.push(semicolon);
        self.node(SurfaceNodeKind::CompactStatement, children)
    }

    fn conclusion(
        &mut self,
        proposition: SurfaceBuilderNodeId,
        justification: Option<SurfaceBuilderNodeId>,
        proof: Option<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        let thus = self.token(SurfaceTokenKind::ReservedWord, "thus");
        let mut children = vec![thus, proposition];
        if let Some(justification) = justification {
            children.push(justification);
        }
        if let Some(proof) = proof {
            children.push(proof);
        }
        let semicolon = self.token(SurfaceTokenKind::ReservedSymbol, ";");
        children.push(semicolon);
        self.node(SurfaceNodeKind::ConclusionStatement, children)
    }

    fn proof(
        &mut self,
        statements: Vec<SurfaceBuilderNodeId>,
        first: (&str, bool),
        last: (&str, bool),
    ) -> SurfaceBuilderNodeId {
        let first = if first.1 {
            self.recovered_token(SurfaceTokenKind::ReservedWord, first.0)
        } else {
            self.token(SurfaceTokenKind::ReservedWord, first.0)
        };
        let last = if last.1 {
            self.recovered_token(SurfaceTokenKind::ReservedWord, last.0)
        } else {
            self.token(SurfaceTokenKind::ReservedWord, last.0)
        };
        let mut children = vec![first];
        children.extend(statements);
        children.push(last);
        self.node(SurfaceNodeKind::ProofBlock, children)
    }

    fn theorem(
        &mut self,
        owner: &str,
        proof: Option<SurfaceBuilderNodeId>,
        role: &str,
        colon: &str,
        additional_proof: Option<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        let role = self.token(SurfaceTokenKind::ReservedWord, role);
        let owner = self.token(SurfaceTokenKind::Identifier, owner);
        let colon = self.token(SurfaceTokenKind::ReservedSymbol, colon);
        let formula = self.formula();
        let mut children = vec![role, owner, colon, formula];
        if let Some(proof) = proof {
            children.push(proof);
        }
        if let Some(proof) = additional_proof {
            children.push(proof);
        }
        let semicolon = self.token(SurfaceTokenKind::ReservedSymbol, ";");
        children.push(semicolon);
        self.node(SurfaceNodeKind::TheoremItem, children)
    }

    fn valid_statement(
        &mut self,
        label: &str,
        reference: Option<&str>,
        include_semantic_child: bool,
    ) -> SurfaceBuilderNodeId {
        let proposition = self.proposition(Some(label), Some(":"), include_semantic_child);
        let justification = reference.map(|reference| {
            let reference = self.reference(reference, false, false);
            self.justification(
                ("by", SurfaceTokenKind::ReservedWord),
                vec![reference],
                false,
            )
        });
        self.compact(proposition, justification)
    }

    fn valid_theorem(
        &mut self,
        owner: &str,
        statements: Vec<SurfaceBuilderNodeId>,
    ) -> SurfaceBuilderNodeId {
        let proof = self.proof(statements, ("proof", false), ("end", false));
        self.theorem(owner, Some(proof), "theorem", ":", None)
    }

    fn finish_items(mut self, items: Vec<SurfaceBuilderNodeId>) -> SurfaceAst {
        let item_list = self.node(SurfaceNodeKind::ItemList, items);
        let compilation = self.node(SurfaceNodeKind::CompilationUnit, vec![item_list]);
        let mut root_children = self.builder.token_node_ids().to_vec();
        root_children.push(compilation);
        let root = self.node(SurfaceNodeKind::Root, root_children);
        self.builder.finish(Some(root), None)
    }
}

fn collect_fixture(ast: &SurfaceAst) -> ProofLabelSourceCollection {
    let module = module_id("pkg", "main");
    let mut contributions = SourceContributionIndex::new();
    let contribution = contribution(&mut contributions, module.clone(), ast.source_id, 0);
    let resolved = SurfaceResolvedArena::lower(ast, &module).unwrap();
    ProofLabelSourceCollector::new(
        ast,
        &module,
        NamespacePath::new("main"),
        contribution,
        &resolved,
    )
    .unwrap()
    .collect()
    .unwrap()
}

fn two_theorem_identity_ast(
    source_id: SourceId,
    offset: usize,
    first_owner: &str,
    second_owner: &str,
    reverse_items: bool,
) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::with_offset(source_id, offset);
    let first_statement = builder.valid_statement("FirstLabel", Some("FirstRef"), false);
    let first = builder.valid_theorem(first_owner, vec![first_statement]);
    let second_statement = builder.valid_statement("SecondLabel", Some("SecondRef"), false);
    let second = builder.valid_theorem(second_owner, vec![second_statement]);
    let items = if reverse_items {
        vec![second, first]
    } else {
        vec![first, second]
    };
    builder.finish_items(items)
}

fn label_topology_ast(source_id: SourceId, nested: bool) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let labelled = builder.valid_statement("Topology", None, false);
    let statements = if nested {
        let child_proof = builder.proof(vec![labelled], ("proof", false), ("end", false));
        let proposition = builder.proposition(None, None, true);
        let owner = builder.conclusion(proposition, None, Some(child_proof));
        vec![owner]
    } else {
        vec![labelled]
    };
    let theorem = builder.valid_theorem("Owner", statements);
    builder.finish_items(vec![theorem])
}

fn same_scope_labels_ast(source_id: SourceId) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let first = builder.valid_statement("Repeated", None, false);
    let second = builder.valid_statement("Repeated", None, false);
    let theorem = builder.valid_theorem("Owner", vec![first, second]);
    builder.finish_items(vec![theorem])
}

fn relocated_theorem_ast(source_id: SourceId, directly_under_root: bool) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let statement = builder.valid_statement("Hidden", Some("Hidden"), false);
    let theorem = builder.valid_theorem("HiddenOwner", vec![statement]);
    let mut root_children = builder.builder.token_node_ids().to_vec();
    if directly_under_root {
        root_children.push(theorem);
    } else {
        let compilation = builder.node(SurfaceNodeKind::CompilationUnit, vec![theorem]);
        root_children.push(compilation);
    }
    let root = builder.node(SurfaceNodeKind::Root, root_children);
    builder.builder.finish(Some(root), None)
}

fn unsupported_item_wrapper_ast(source_id: SourceId, kind: SurfaceNodeKind) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let hidden_statement = builder.valid_statement("Hidden", Some("Hidden"), false);
    let hidden_theorem = builder.valid_theorem("HiddenOwner", vec![hidden_statement]);
    let unsupported = builder.node(kind, vec![hidden_theorem]);
    let sentinel_statement = builder.valid_statement("Sentinel", Some("Sentinel"), false);
    let sentinel_theorem = builder.valid_theorem("SentinelOwner", vec![sentinel_statement]);
    builder.finish_items(vec![unsupported, sentinel_theorem])
}

fn recovered_item_wrapper_ast(source_id: SourceId) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let hidden_statement = builder.valid_statement("Hidden", Some("Hidden"), false);
    let hidden_theorem = builder.valid_theorem("HiddenOwner", vec![hidden_statement]);
    let recovered = builder.builder.add_recovery(
        SyntaxRecoveryKind::MissingItem,
        range(source_id, 0, 1),
        vec![hidden_theorem],
    );
    let sentinel_statement = builder.valid_statement("Sentinel", Some("Sentinel"), false);
    let sentinel_theorem = builder.valid_theorem("SentinelOwner", vec![sentinel_statement]);
    builder.finish_items(vec![recovered, sentinel_theorem])
}

fn unsupported_proof_child_ast(source_id: SourceId, kind: SurfaceNodeKind) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let hidden_statement = builder.valid_statement("Hidden", Some("Hidden"), false);
    let hidden_proof = builder.proof(vec![hidden_statement], ("proof", false), ("end", false));
    let unsupported = builder.node(kind, vec![hidden_proof]);
    let sentinel = builder.valid_statement("Sentinel", Some("Sentinel"), false);
    let theorem = builder.valid_theorem("Owner", vec![unsupported, sentinel]);
    builder.finish_items(vec![theorem])
}

#[derive(Clone, Copy, Debug)]
enum TheoremShapeMutation {
    WrongRole,
    WrongOwnerKind,
    WrongColon,
    MissingProof,
    AdditionalProof,
    WrappedProof,
    RecoveredRole,
    RecoveredOwner,
    RecoveredColon,
}

fn malformed_theorem_owner_ast(source_id: SourceId, mutation: TheoremShapeMutation) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let role = if matches!(mutation, TheoremShapeMutation::RecoveredRole) {
        builder.recovered_token(SurfaceTokenKind::ReservedWord, "theorem")
    } else {
        builder.token(
            SurfaceTokenKind::ReservedWord,
            if matches!(mutation, TheoremShapeMutation::WrongRole) {
                "lemma"
            } else {
                "theorem"
            },
        )
    };
    let owner = if matches!(mutation, TheoremShapeMutation::RecoveredOwner) {
        builder.recovered_token(SurfaceTokenKind::Identifier, "BadOwner")
    } else {
        builder.token(
            if matches!(mutation, TheoremShapeMutation::WrongOwnerKind) {
                SurfaceTokenKind::ReservedWord
            } else {
                SurfaceTokenKind::Identifier
            },
            "BadOwner",
        )
    };
    let colon = if matches!(mutation, TheoremShapeMutation::RecoveredColon) {
        builder.recovered_token(SurfaceTokenKind::ReservedSymbol, ":")
    } else {
        builder.token(
            SurfaceTokenKind::ReservedSymbol,
            if matches!(mutation, TheoremShapeMutation::WrongColon) {
                ";"
            } else {
                ":"
            },
        )
    };
    let formula = builder.formula();
    let hidden = builder.valid_statement("Hidden", Some("Hidden"), false);
    let proof = builder.proof(vec![hidden], ("proof", false), ("end", false));
    let mut children = vec![role, owner, colon, formula];
    match mutation {
        TheoremShapeMutation::MissingProof => {}
        TheoremShapeMutation::AdditionalProof => {
            children.push(proof);
            let second = builder.proof(Vec::new(), ("proof", false), ("end", false));
            children.push(second);
        }
        TheoremShapeMutation::WrappedProof => {
            let wrapper = builder.node(SurfaceNodeKind::VisibleItem, vec![proof]);
            children.push(wrapper);
        }
        _ => children.push(proof),
    }
    let end = builder.token(SurfaceTokenKind::ReservedSymbol, ";");
    children.push(end);
    let malformed = builder.node(SurfaceNodeKind::TheoremItem, children);

    let sentinel_statement = builder.valid_statement("Sentinel", Some("Sentinel"), false);
    let sentinel = builder.valid_theorem("SentinelOwner", vec![sentinel_statement]);
    builder.finish_items(vec![malformed, sentinel])
}

#[derive(Clone, Copy, Debug)]
enum ProofBoundaryMutation {
    MissingEnd,
    WrongStart,
    WrongEnd,
    RecoveredStart,
    RecoveredEnd,
    RecoveredInterior,
}

fn malformed_proof_boundary_ast(
    source_id: SourceId,
    mutation: ProofBoundaryMutation,
) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let hidden = builder.valid_statement("Hidden", Some("Hidden"), false);
    let proof = match mutation {
        ProofBoundaryMutation::MissingEnd => {
            let start = builder.token(SurfaceTokenKind::ReservedWord, "proof");
            builder.node(SurfaceNodeKind::ProofBlock, vec![start, hidden])
        }
        ProofBoundaryMutation::WrongStart => {
            builder.proof(vec![hidden], ("begin", false), ("end", false))
        }
        ProofBoundaryMutation::WrongEnd => {
            builder.proof(vec![hidden], ("proof", false), ("stop", false))
        }
        ProofBoundaryMutation::RecoveredStart => {
            builder.proof(vec![hidden], ("proof", true), ("end", false))
        }
        ProofBoundaryMutation::RecoveredEnd => {
            builder.proof(vec![hidden], ("proof", false), ("end", true))
        }
        ProofBoundaryMutation::RecoveredInterior => {
            let start = builder.token(SurfaceTokenKind::ReservedWord, "proof");
            let recovery = builder.recovered_token(SurfaceTokenKind::ErrorRecovery, "?");
            let end = builder.token(SurfaceTokenKind::ReservedWord, "end");
            builder.node(
                SurfaceNodeKind::ProofBlock,
                vec![start, hidden, recovery, end],
            )
        }
    };
    let malformed = builder.theorem("BadOwner", Some(proof), "theorem", ":", None);
    let sentinel_statement = builder.valid_statement("Sentinel", Some("Sentinel"), false);
    let sentinel = builder.valid_theorem("SentinelOwner", vec![sentinel_statement]);
    builder.finish_items(vec![malformed, sentinel])
}

fn finish_statement_mutation(
    mut builder: CollectorAstBuilder,
    mutated: SurfaceBuilderNodeId,
) -> SurfaceAst {
    let sentinel = builder.valid_statement("Sentinel", Some("Sentinel"), false);
    let theorem = builder.valid_theorem("Owner", vec![mutated, sentinel]);
    builder.finish_items(vec![theorem])
}

#[derive(Clone, Copy, Debug)]
enum CompactLabelMutation {
    MissingIdentifier,
    WrongIdentifierKind,
    MissingColon,
    WrongColon,
    RecoveredIdentifier,
    RecoveredColon,
    AdditionalProposition,
}

fn malformed_compact_label_ast(source_id: SourceId, mutation: CompactLabelMutation) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let identifier = match mutation {
        CompactLabelMutation::MissingIdentifier => None,
        CompactLabelMutation::WrongIdentifierKind => {
            Some(builder.token(SurfaceTokenKind::ReservedWord, "MutLabel"))
        }
        CompactLabelMutation::RecoveredIdentifier => {
            Some(builder.recovered_token(SurfaceTokenKind::Identifier, "MutLabel"))
        }
        _ => Some(builder.token(SurfaceTokenKind::Identifier, "MutLabel")),
    };
    let colon = match mutation {
        CompactLabelMutation::MissingColon => None,
        CompactLabelMutation::WrongColon => {
            Some(builder.token(SurfaceTokenKind::ReservedSymbol, ";"))
        }
        CompactLabelMutation::RecoveredColon => {
            Some(builder.recovered_token(SurfaceTokenKind::ReservedSymbol, ":"))
        }
        _ => Some(builder.token(SurfaceTokenKind::ReservedSymbol, ":")),
    };
    let formula = builder.formula();
    let mut proposition_children = Vec::new();
    if let Some(identifier) = identifier {
        proposition_children.push(identifier);
    }
    if let Some(colon) = colon {
        proposition_children.push(colon);
    }
    proposition_children.push(formula);
    let proposition = builder.node(SurfaceNodeKind::Proposition, proposition_children);
    let reference = builder.reference("MutRef", false, false);
    let justification = builder.justification(
        ("by", SurfaceTokenKind::ReservedWord),
        vec![reference],
        false,
    );
    let mutated = if matches!(mutation, CompactLabelMutation::AdditionalProposition) {
        let additional = builder.proposition(Some("Other"), Some(":"), false);
        let semicolon = builder.token(SurfaceTokenKind::ReservedSymbol, ";");
        builder.node(
            SurfaceNodeKind::CompactStatement,
            vec![proposition, additional, justification, semicolon],
        )
    } else {
        builder.compact(proposition, Some(justification))
    };
    finish_statement_mutation(builder, mutated)
}

#[derive(Clone, Copy, Debug)]
enum ReferenceMutation {
    MissingIdentifier,
    WrongIdentifierKind,
    AdditionalChild,
    RecoveredIdentifier,
}

fn malformed_reference_ast(source_id: SourceId, mutation: ReferenceMutation) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let proposition = builder.proposition(Some("MutLabel"), Some(":"), false);
    let reference = match mutation {
        ReferenceMutation::MissingIdentifier => {
            builder.node(SurfaceNodeKind::Reference, Vec::new())
        }
        ReferenceMutation::WrongIdentifierKind => {
            let child = builder.token(SurfaceTokenKind::ReservedWord, "MutRef");
            builder.node(SurfaceNodeKind::Reference, vec![child])
        }
        ReferenceMutation::AdditionalChild => builder.reference("MutRef", true, false),
        ReferenceMutation::RecoveredIdentifier => builder.reference("MutRef", false, true),
    };
    let justification = builder.justification(
        ("by", SurfaceTokenKind::ReservedWord),
        vec![reference],
        false,
    );
    let mutated = builder.compact(proposition, Some(justification));
    finish_statement_mutation(builder, mutated)
}

#[derive(Clone, Copy, Debug)]
enum JustificationMutation {
    WrongFirstToken,
    RecoveredFirstToken,
    AdditionalReferenceList,
    ComputationInstead,
    WrappedJustification,
    WrappedReferenceList,
    RelocatedIntoProposition,
    ProofRelocatedIntoProposition,
    ConclusionLabel,
}

fn malformed_justification_edge_ast(
    source_id: SourceId,
    mutation: JustificationMutation,
) -> SurfaceAst {
    let mut builder = CollectorAstBuilder::new(source_id);
    let reference = builder.reference("MutRef", false, false);
    let justification = match mutation {
        JustificationMutation::WrongFirstToken => builder.justification(
            ("from", SurfaceTokenKind::ReservedWord),
            vec![reference],
            false,
        ),
        JustificationMutation::RecoveredFirstToken => {
            let by = builder.recovered_token(SurfaceTokenKind::ReservedWord, "by");
            let list = builder.node(SurfaceNodeKind::ReferenceList, vec![reference]);
            builder.node(SurfaceNodeKind::JustificationClause, vec![by, list])
        }
        JustificationMutation::AdditionalReferenceList => builder.justification(
            ("by", SurfaceTokenKind::ReservedWord),
            vec![reference],
            true,
        ),
        JustificationMutation::ComputationInstead => {
            let by = builder.token(SurfaceTokenKind::ReservedWord, "by");
            let list = builder.node(SurfaceNodeKind::ReferenceList, vec![reference]);
            builder.node(SurfaceNodeKind::ComputationJustification, vec![by, list])
        }
        JustificationMutation::WrappedReferenceList => {
            let by = builder.token(SurfaceTokenKind::ReservedWord, "by");
            let list = builder.node(SurfaceNodeKind::ReferenceList, vec![reference]);
            let wrapper = builder.node(SurfaceNodeKind::VisibleItem, vec![list]);
            builder.node(SurfaceNodeKind::JustificationClause, vec![by, wrapper])
        }
        _ => builder.justification(
            ("by", SurfaceTokenKind::ReservedWord),
            vec![reference],
            false,
        ),
    };

    let mutated = match mutation {
        JustificationMutation::WrappedJustification => {
            let proposition = builder.proposition(Some("MutLabel"), Some(":"), false);
            let wrapper = builder.node(SurfaceNodeKind::VisibleItem, vec![justification]);
            let semicolon = builder.token(SurfaceTokenKind::ReservedSymbol, ";");
            builder.node(
                SurfaceNodeKind::CompactStatement,
                vec![proposition, wrapper, semicolon],
            )
        }
        JustificationMutation::RelocatedIntoProposition => {
            let label = builder.token(SurfaceTokenKind::Identifier, "MutLabel");
            let colon = builder.token(SurfaceTokenKind::ReservedSymbol, ":");
            let proposition = builder.node(
                SurfaceNodeKind::Proposition,
                vec![label, colon, justification],
            );
            builder.compact(proposition, None)
        }
        JustificationMutation::ProofRelocatedIntoProposition => {
            let hidden = builder.valid_statement("Hidden", Some("Hidden"), false);
            let hidden_proof = builder.proof(vec![hidden], ("proof", false), ("end", false));
            let label = builder.token(SurfaceTokenKind::Identifier, "MutLabel");
            let colon = builder.token(SurfaceTokenKind::ReservedSymbol, ":");
            let proposition = builder.node(
                SurfaceNodeKind::Proposition,
                vec![label, colon, hidden_proof],
            );
            builder.compact(proposition, Some(justification))
        }
        JustificationMutation::ConclusionLabel => {
            let proposition = builder.proposition(Some("MutLabel"), Some(":"), false);
            builder.conclusion(proposition, Some(justification), None)
        }
        _ => {
            let proposition = builder.proposition(Some("MutLabel"), Some(":"), false);
            builder.compact(proposition, Some(justification))
        }
    };
    finish_statement_mutation(builder, mutated)
}

fn current_theorem_projection(
    fixture: &ProjectionFixture,
    spelling: &str,
    start: usize,
    visible_after: usize,
) -> LabelProjection {
    LabelProjection::current_module(
        fixture.data(
            spelling,
            LabelKind::Theorem,
            "theorem",
            start,
            visible_after,
        ),
        visible_after,
    )
    .with_visibility(Visibility::Public)
    .with_export_status(ExportStatus::Exported)
}

fn imported_theorem_projection(
    fixture: &ProjectionFixture,
    spelling: &str,
    start: usize,
) -> LabelProjection {
    LabelProjection::imported(fixture.data(spelling, LabelKind::Theorem, "theorem", start, start))
}

fn proof_step_projection(
    fixture: &ProjectionFixture,
    spelling: &str,
    start: usize,
    visible_after: usize,
    scope: LabelScopePath,
) -> LabelProjection {
    LabelProjection::proof_step(
        fixture.data(
            spelling,
            LabelKind::ProofStep,
            "proof",
            start,
            visible_after,
        ),
        visible_after,
        scope,
    )
}

#[derive(Clone)]
struct ProjectionFixture {
    source_id: SourceId,
    module: ModuleId,
    namespace: NamespacePath,
    contribution: SourceContributionId,
}

impl ProjectionFixture {
    fn new(
        source_id: SourceId,
        module: ModuleId,
        namespace: NamespacePath,
        contribution: SourceContributionId,
    ) -> Self {
        Self {
            source_id,
            module,
            namespace,
            contribution,
        }
    }

    fn data(
        &self,
        spelling: &str,
        kind: LabelKind,
        origin_role: &str,
        start: usize,
        ordinal: usize,
    ) -> LabelProjectionData {
        let range = range(self.source_id, start, start + spelling.len());
        LabelProjectionData {
            origin_path: LabelOriginPath::new(format!(
                "{}::{}::{origin_role}::{spelling}",
                self.module.package().as_str(),
                self.module.path().as_str()
            )),
            module: self.module.clone(),
            namespace: self.namespace.clone(),
            primary_spelling: spelling.to_owned(),
            kind,
            declaration_range: range,
            origin: origin(self.source_id, self.module.clone(), range, ordinal),
            contribution: self.contribution,
        }
    }
}

fn unqualified_ref(
    source_id: SourceId,
    module: ModuleId,
    ordinal: usize,
    start: usize,
    spelling: &str,
    scope: Option<LabelScopePath>,
) -> LabelReferenceCandidate {
    let (site, origin) = reference_site(source_id, module, start, spelling, ordinal);
    LabelReferenceCandidate::unqualified_citation(site, origin, ordinal, scope)
}

fn qualified_ref(
    source_id: SourceId,
    current: ModuleId,
    target: ModuleId,
    namespace: NamespacePath,
    ordinal: usize,
    start: usize,
    spelling: &str,
) -> LabelReferenceCandidate {
    let (site, origin) = reference_site(source_id, current, start, spelling, ordinal);
    LabelReferenceCandidate::qualified_citation(site, origin, ordinal, target, namespace)
}

fn reference_site(
    source_id: SourceId,
    module: ModuleId,
    start: usize,
    spelling: &str,
    ordinal: usize,
) -> (ReferenceSite, SemanticOrigin) {
    let range = range(source_id, start, start + spelling.len());
    let origin = origin(source_id, module, range, ordinal);
    let mut arena = ResolvedArenaBuilder::new();
    let node = arena
        .push(ResolvedNode::new(
            SurfaceNodeKind::Reference,
            Vec::new(),
            origin.clone(),
        ))
        .unwrap();
    (ReferenceSite::new(node, range, spelling), origin)
}

fn origin(
    source_id: SourceId,
    module: ModuleId,
    range: SourceRange,
    ordinal: usize,
) -> SemanticOrigin {
    SemanticOrigin::new(
        source_id,
        module,
        SourceAnchor::Range(range),
        vec![ordinal as u32],
    )
}

fn contribution(
    index: &mut SourceContributionIndex,
    module: ModuleId,
    source_id: SourceId,
    start: usize,
) -> SourceContributionId {
    index.insert(
        module,
        ContributionKind::LocalSource { source_id },
        SourceAnchor::Range(range(source_id, start, start + 1)),
    )
}

fn assert_resolved_label(resolution: &LabelResolutionResult, index: usize, expected_origin: &str) {
    let entry = resolution.table().get(resolution.ids()[index]).unwrap();
    let LabelResolution::Resolved(label) = entry.resolution() else {
        panic!("expected resolved label at index {index}");
    };
    assert_eq!(label.origin().as_str(), expected_origin);
}

fn assert_unresolved_label(
    resolution: &LabelResolutionResult,
    index: usize,
    expected_expectation: LabelExpectation,
    expected_spelling: &str,
) {
    let entry = resolution.table().get(resolution.ids()[index]).unwrap();
    let LabelResolution::Unresolved(unresolved) = entry.resolution() else {
        panic!("expected unresolved label at index {index}");
    };
    assert_eq!(unresolved.expectation(), expected_expectation);
    assert_eq!(unresolved.spelling(), expected_spelling);
}

fn module_id(package: &str, path: &str) -> ModuleId {
    ModuleId::new(PackageId::new(package), ModulePath::new(path))
}

fn source_id() -> SourceId {
    let snapshot_id = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "34".repeat(Hash::BYTE_LEN)
    ))
    .unwrap();
    let allocator = InMemorySessionIdAllocator::new();
    allocator.next_source_id(snapshot_id).unwrap()
}

fn distinct_source_ids() -> (SourceId, SourceId) {
    let snapshot_id = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "35".repeat(Hash::BYTE_LEN)
    ))
    .unwrap();
    let allocator = InMemorySessionIdAllocator::new();
    (
        allocator.next_source_id(snapshot_id).unwrap(),
        allocator.next_source_id(snapshot_id).unwrap(),
    )
}

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}
