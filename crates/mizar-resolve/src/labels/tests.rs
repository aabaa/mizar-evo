use super::*;
use crate::env::{ContributionKind, SourceContributionIndex};
use crate::resolved_ast::{ResolvedArenaBuilder, ResolvedNode, SemanticOrigin};
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
    SourceAnchor, SourceId,
};
use mizar_syntax::ast::SurfaceNodeKind;

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

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}
