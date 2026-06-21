use super::*;
use crate::recovery::suppress_dependent_diagnostic_for_recovered_origin;
use std::cmp::Ordering;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum DiagnosticRootKey {
    Name {
        range: (usize, usize),
        spelling: String,
        lookup: NameLookupClass,
    },
    AmbiguousName {
        range: (usize, usize),
        spelling: String,
        name_ref: usize,
    },
    Namespace {
        range: (usize, usize),
        spelling: String,
        class: NamespaceFailureClass,
        failed_segment: Vec<String>,
    },
    ImportAliasDependency {
        range: (usize, usize),
        alias: String,
        ordinal: usize,
        class: ImportPathFailureClass,
    },
}

#[derive(Debug, Clone)]
struct DiagnosticRootState {
    root: NameDiagnosticRootId,
    key: DiagnosticRootKey,
    kind: NameDiagnosticKind,
    range: SourceRange,
    attempted_spelling: String,
    normalized_namespace_prefix: Vec<String>,
    name_ref: Option<NameRefId>,
    secondary_ranges: Vec<SourceRange>,
    symbol_candidates: Vec<NameResolutionCandidate>,
    namespace_candidates: Vec<NameDiagnosticNamespaceCandidate>,
    dependent_ranges: Vec<SourceRange>,
}

#[derive(Debug, Clone)]
struct DiagnosticRootPayload {
    kind: NameDiagnosticKind,
    range: SourceRange,
    attempted_spelling: String,
    normalized_namespace_prefix: Vec<String>,
    name_ref: Option<NameRefId>,
    secondary_ranges: Vec<SourceRange>,
    symbol_candidates: Vec<NameResolutionCandidate>,
    namespace_candidates: Vec<NameDiagnosticNamespaceCandidate>,
}

#[derive(Debug, Clone)]
struct PendingDiagnosticRecord {
    root_key: DiagnosticRootKey,
    role: NameDiagnosticRole,
    kind: NameDiagnosticKind,
    range: SourceRange,
    attempted_spelling: String,
    normalized_namespace_prefix: Vec<String>,
    name_ref: Option<NameRefId>,
    secondary_ranges: Vec<SourceRange>,
    symbol_candidates: Vec<NameResolutionCandidate>,
    namespace_candidates: Vec<NameDiagnosticNamespaceCandidate>,
    dependent_ranges: Vec<SourceRange>,
    ordinal: usize,
}

impl PendingDiagnosticRecord {
    fn from_root(root: &DiagnosticRootState, ordinal: usize) -> Self {
        Self {
            root_key: root.key.clone(),
            role: NameDiagnosticRole::Primary,
            kind: root.kind,
            range: root.range,
            attempted_spelling: root.attempted_spelling.clone(),
            normalized_namespace_prefix: root.normalized_namespace_prefix.clone(),
            name_ref: root.name_ref,
            secondary_ranges: root.secondary_ranges.clone(),
            symbol_candidates: root.symbol_candidates.clone(),
            namespace_candidates: root.namespace_candidates.clone(),
            dependent_ranges: root.dependent_ranges.clone(),
            ordinal,
        }
    }
}

pub(super) fn collect_name_diagnostics(
    name_refs: &NameRefTable,
    namespace_roots: &[UnresolvedNamespacePath],
) -> NameDiagnosticReport {
    let mut roots = BTreeMap::<DiagnosticRootKey, DiagnosticRootState>::new();
    let mut namespace_root_keys = BTreeMap::<(String, (usize, usize)), DiagnosticRootKey>::new();
    let mut pending = Vec::<PendingDiagnosticRecord>::new();

    let mut ordered_namespace_roots = namespace_roots.iter().collect::<Vec<_>>();
    ordered_namespace_roots.sort_by(|left, right| unresolved_namespace_path_cmp(left, right));
    for namespace in ordered_namespace_roots {
        if namespace.recovered() {
            continue;
        }
        collect_namespace_diagnostic_root(
            namespace,
            &mut roots,
            &mut namespace_root_keys,
            &mut pending,
        );
    }

    for (name_ref, entry) in name_refs.iter() {
        if suppress_dependent_diagnostic_for_recovered_origin(entry.origin()) {
            continue;
        }
        match entry.resolution() {
            NameResolution::Ambiguous(ambiguous) => {
                let key = ambiguous_name_root_key(name_ref, ambiguous);
                insert_root(
                    &mut roots,
                    key,
                    DiagnosticRootPayload {
                        kind: NameDiagnosticKind::AmbiguousName,
                        range: ambiguous.range(),
                        attempted_spelling: ambiguous.spelling().to_owned(),
                        normalized_namespace_prefix: Vec::new(),
                        name_ref: Some(name_ref),
                        secondary_ranges: ambiguous
                            .candidates()
                            .iter()
                            .map(|candidate| candidate.range())
                            .collect(),
                        symbol_candidates: ambiguous.candidates().to_vec(),
                        namespace_candidates: Vec::new(),
                    },
                );
            }
            NameResolution::Unresolved(unresolved) => {
                if unresolved.lookup() == NameLookupClass::Namespace {
                    let namespace_key = (
                        unresolved.spelling().to_owned(),
                        range_key(unresolved.range()),
                    );
                    if let Some(root_key) = namespace_root_keys.get(&namespace_key).cloned() {
                        add_dependent_range(&mut roots, &root_key, unresolved.range());
                        pending.push(PendingDiagnosticRecord {
                            root_key,
                            role: NameDiagnosticRole::Cascade,
                            kind: NameDiagnosticKind::UnresolvedName {
                                lookup: unresolved.lookup(),
                            },
                            range: unresolved.range(),
                            attempted_spelling: unresolved.spelling().to_owned(),
                            normalized_namespace_prefix: Vec::new(),
                            name_ref: Some(name_ref),
                            secondary_ranges: Vec::new(),
                            symbol_candidates: Vec::new(),
                            namespace_candidates: Vec::new(),
                            dependent_ranges: Vec::new(),
                            ordinal: name_ref.index(),
                        });
                        continue;
                    }
                }
                let key = unresolved_name_root_key(unresolved);
                let inserted = insert_root(
                    &mut roots,
                    key.clone(),
                    DiagnosticRootPayload {
                        kind: NameDiagnosticKind::UnresolvedName {
                            lookup: unresolved.lookup(),
                        },
                        range: unresolved.range(),
                        attempted_spelling: unresolved.spelling().to_owned(),
                        normalized_namespace_prefix: Vec::new(),
                        name_ref: Some(name_ref),
                        secondary_ranges: Vec::new(),
                        symbol_candidates: Vec::new(),
                        namespace_candidates: Vec::new(),
                    },
                );
                if !inserted {
                    add_dependent_range(&mut roots, &key, unresolved.range());
                    pending.push(PendingDiagnosticRecord {
                        root_key: key,
                        role: NameDiagnosticRole::Cascade,
                        kind: NameDiagnosticKind::UnresolvedName {
                            lookup: unresolved.lookup(),
                        },
                        range: unresolved.range(),
                        attempted_spelling: unresolved.spelling().to_owned(),
                        normalized_namespace_prefix: Vec::new(),
                        name_ref: Some(name_ref),
                        secondary_ranges: Vec::new(),
                        symbol_candidates: Vec::new(),
                        namespace_candidates: Vec::new(),
                        dependent_ranges: Vec::new(),
                        ordinal: name_ref.index(),
                    });
                }
            }
            NameResolution::Resolved(_)
            | NameResolution::ResolvedBuiltin(_)
            | NameResolution::DeferredSelector(_) => {}
        }
    }

    finish_name_diagnostics(roots, pending)
}

fn collect_namespace_diagnostic_root(
    namespace: &UnresolvedNamespacePath,
    roots: &mut BTreeMap<DiagnosticRootKey, DiagnosticRootState>,
    namespace_root_keys: &mut BTreeMap<(String, (usize, usize)), DiagnosticRootKey>,
    pending: &mut Vec<PendingDiagnosticRecord>,
) {
    let namespace_secondary_ranges = namespace_secondary_ranges(namespace);
    let namespace_candidates = diagnostic_namespace_candidates(namespace.candidate_targets());
    let namespace_prefix = namespace_normalized_prefix(namespace);
    let root_key = if let Some(import_root) = first_import_dependency_root_key(namespace) {
        for dependency in namespace.import_dependencies() {
            let dependency_key = import_dependency_root_key(dependency);
            let mut secondary_ranges = import_dependency_secondary_ranges(dependency);
            secondary_ranges.extend(namespace_secondary_ranges.clone());
            insert_root(
                roots,
                dependency_key.clone(),
                DiagnosticRootPayload {
                    kind: NameDiagnosticKind::UnresolvedImportAliasDependency {
                        class: dependency.class(),
                    },
                    range: dependency.range(),
                    attempted_spelling: dependency.alias().to_owned(),
                    normalized_namespace_prefix: namespace_prefix.clone(),
                    name_ref: None,
                    secondary_ranges,
                    symbol_candidates: Vec::new(),
                    namespace_candidates: namespace_candidates.clone(),
                },
            );
            add_dependent_range(roots, &dependency_key, namespace.range());
        }
        pending.push(namespace_cascade_record(namespace, import_root.clone()));
        import_root
    } else {
        let key = namespace_root_key(namespace);
        insert_root(
            roots,
            key.clone(),
            DiagnosticRootPayload {
                kind: namespace_diagnostic_kind(namespace),
                range: namespace.range(),
                attempted_spelling: namespace.spelling().to_owned(),
                normalized_namespace_prefix: namespace_prefix,
                name_ref: None,
                secondary_ranges: namespace_secondary_ranges,
                symbol_candidates: Vec::new(),
                namespace_candidates,
            },
        );
        key
    };
    namespace_root_keys.insert(
        (
            namespace.spelling().to_owned(),
            range_key(namespace.range()),
        ),
        root_key.clone(),
    );
    if let Some(failed_segment) = namespace.failed_segment() {
        namespace_root_keys.insert(
            (
                namespace.spelling().to_owned(),
                range_key(failed_segment.range()),
            ),
            root_key,
        );
    }
}

fn finish_name_diagnostics(
    roots: BTreeMap<DiagnosticRootKey, DiagnosticRootState>,
    mut pending: Vec<PendingDiagnosticRecord>,
) -> NameDiagnosticReport {
    let mut ordered_roots = roots.into_values().collect::<Vec<_>>();
    ordered_roots.sort_by(diagnostic_root_cmp);
    let root_ids = ordered_roots
        .iter()
        .enumerate()
        .map(|(index, root)| (root.key.clone(), NameDiagnosticRootId::new(index)))
        .collect::<BTreeMap<_, _>>();
    for (index, root) in ordered_roots.iter_mut().enumerate() {
        root.root = NameDiagnosticRootId::new(index);
    }
    pending.extend(
        ordered_roots
            .iter()
            .enumerate()
            .map(|(index, root)| PendingDiagnosticRecord::from_root(root, index)),
    );

    let records = pending
        .into_iter()
        .filter_map(|record| {
            let root = *root_ids.get(&record.root_key)?;
            let root_state = ordered_roots.iter().find(|state| state.root == root)?;
            Some(NameDiagnostic {
                id: NameDiagnosticId::new(record.ordinal),
                root,
                role: record.role,
                kind: record.kind,
                root_range: root_state.range,
                range: record.range,
                attempted_spelling: record.attempted_spelling,
                normalized_namespace_prefix: record.normalized_namespace_prefix,
                name_ref: record.name_ref,
                secondary_ranges: sorted_ranges(record.secondary_ranges),
                symbol_candidates: sorted_symbol_candidates(record.symbol_candidates),
                namespace_candidates: sorted_namespace_candidates(record.namespace_candidates),
                dependent_ranges: sorted_ranges(record.dependent_ranges),
            })
        })
        .collect();
    NameDiagnosticReport::new(records)
}

fn insert_root(
    roots: &mut BTreeMap<DiagnosticRootKey, DiagnosticRootState>,
    key: DiagnosticRootKey,
    payload: DiagnosticRootPayload,
) -> bool {
    use std::collections::btree_map::Entry;
    match roots.entry(key.clone()) {
        Entry::Vacant(entry) => {
            entry.insert(DiagnosticRootState {
                root: NameDiagnosticRootId::new(0),
                key,
                kind: payload.kind,
                range: payload.range,
                attempted_spelling: payload.attempted_spelling,
                normalized_namespace_prefix: payload.normalized_namespace_prefix,
                name_ref: payload.name_ref,
                secondary_ranges: sorted_ranges(payload.secondary_ranges),
                symbol_candidates: sorted_symbol_candidates(payload.symbol_candidates),
                namespace_candidates: sorted_namespace_candidates(payload.namespace_candidates),
                dependent_ranges: Vec::new(),
            });
            true
        }
        Entry::Occupied(mut entry) => {
            let root = entry.get_mut();
            if root.normalized_namespace_prefix.is_empty() {
                root.normalized_namespace_prefix = payload.normalized_namespace_prefix;
            }
            root.secondary_ranges
                .extend(sorted_ranges(payload.secondary_ranges));
            root.secondary_ranges = sorted_ranges(root.secondary_ranges.clone());
            root.symbol_candidates
                .extend(sorted_symbol_candidates(payload.symbol_candidates));
            root.symbol_candidates = sorted_symbol_candidates(root.symbol_candidates.clone());
            root.namespace_candidates
                .extend(sorted_namespace_candidates(payload.namespace_candidates));
            root.namespace_candidates =
                sorted_namespace_candidates(root.namespace_candidates.clone());
            false
        }
    }
}

fn add_dependent_range(
    roots: &mut BTreeMap<DiagnosticRootKey, DiagnosticRootState>,
    key: &DiagnosticRootKey,
    range: SourceRange,
) {
    if let Some(root) = roots.get_mut(key) {
        root.dependent_ranges.push(range);
        root.dependent_ranges = sorted_ranges(root.dependent_ranges.clone());
    }
}

fn ambiguous_name_root_key(name_ref: NameRefId, ambiguous: &AmbiguousNameRef) -> DiagnosticRootKey {
    DiagnosticRootKey::AmbiguousName {
        range: range_key(ambiguous.range()),
        spelling: ambiguous.spelling().to_owned(),
        name_ref: name_ref.index(),
    }
}

fn unresolved_name_root_key(
    unresolved: &crate::resolved_ast::UnresolvedNameRef,
) -> DiagnosticRootKey {
    DiagnosticRootKey::Name {
        range: range_key(unresolved.range()),
        spelling: unresolved.spelling().to_owned(),
        lookup: unresolved.lookup(),
    }
}

fn namespace_root_key(namespace: &UnresolvedNamespacePath) -> DiagnosticRootKey {
    DiagnosticRootKey::Namespace {
        range: range_key(namespace.range()),
        spelling: namespace.spelling().to_owned(),
        class: namespace.class(),
        failed_segment: failed_segment_key(namespace),
    }
}

fn first_import_dependency_root_key(
    namespace: &UnresolvedNamespacePath,
) -> Option<DiagnosticRootKey> {
    namespace
        .import_dependencies()
        .iter()
        .min_by(|left, right| namespace_import_dependency_cmp(left, right))
        .map(import_dependency_root_key)
}

fn import_dependency_root_key(dependency: &NamespaceImportDependency) -> DiagnosticRootKey {
    DiagnosticRootKey::ImportAliasDependency {
        range: range_key(dependency.range()),
        alias: dependency.alias().to_owned(),
        ordinal: dependency.ordinal(),
        class: dependency.class(),
    }
}

fn namespace_cascade_record(
    namespace: &UnresolvedNamespacePath,
    root_key: DiagnosticRootKey,
) -> PendingDiagnosticRecord {
    PendingDiagnosticRecord {
        root_key,
        role: NameDiagnosticRole::Cascade,
        kind: namespace_diagnostic_kind(namespace),
        range: namespace.range(),
        attempted_spelling: namespace.spelling().to_owned(),
        normalized_namespace_prefix: namespace_normalized_prefix(namespace),
        name_ref: None,
        secondary_ranges: namespace_secondary_ranges(namespace),
        symbol_candidates: Vec::new(),
        namespace_candidates: diagnostic_namespace_candidates(namespace.candidate_targets()),
        dependent_ranges: Vec::new(),
        ordinal: namespace.ordinal(),
    }
}

fn namespace_diagnostic_kind(namespace: &UnresolvedNamespacePath) -> NameDiagnosticKind {
    if namespace_is_ambiguous(namespace) {
        NameDiagnosticKind::AmbiguousNamespace {
            class: namespace.class(),
        }
    } else {
        NameDiagnosticKind::UnresolvedNamespace {
            class: namespace.class(),
        }
    }
}

fn namespace_is_ambiguous(namespace: &UnresolvedNamespacePath) -> bool {
    namespace.class() == NamespaceFailureClass::AmbiguousImportAlias
        || !namespace.candidate_targets().is_empty()
}

fn namespace_secondary_ranges(namespace: &UnresolvedNamespacePath) -> Vec<SourceRange> {
    let mut ranges = Vec::new();
    if let Some(segment) = namespace.failed_segment() {
        ranges.push(segment.range());
    }
    ranges.extend(
        namespace
            .import_dependencies()
            .iter()
            .flat_map(import_dependency_secondary_ranges),
    );
    ranges.extend(
        namespace
            .candidate_targets()
            .iter()
            .flat_map(namespace_candidate_secondary_ranges),
    );
    sorted_ranges(ranges)
}

fn import_dependency_secondary_ranges(dependency: &NamespaceImportDependency) -> Vec<SourceRange> {
    let mut ranges = vec![dependency.range()];
    if let Some(alias_range) = dependency.alias_range() {
        ranges.push(alias_range);
    }
    sorted_ranges(ranges)
}

fn namespace_candidate_secondary_ranges(candidate: &NamespaceCandidateTarget) -> Vec<SourceRange> {
    let mut ranges = vec![candidate.range()];
    if let Some(alias_range) = candidate.alias_range() {
        ranges.push(alias_range);
    }
    sorted_ranges(ranges)
}

fn sorted_ranges(mut ranges: Vec<SourceRange>) -> Vec<SourceRange> {
    ranges.sort_by(|left, right| source_range_cmp(*left, *right));
    ranges.dedup_by(|left, right| *left == *right);
    ranges
}

fn sorted_symbol_candidates(
    mut candidates: Vec<NameResolutionCandidate>,
) -> Vec<NameResolutionCandidate> {
    candidates.sort();
    candidates.dedup();
    candidates
}

fn sorted_namespace_candidates(
    mut candidates: Vec<NameDiagnosticNamespaceCandidate>,
) -> Vec<NameDiagnosticNamespaceCandidate> {
    candidates.sort_by(diagnostic_namespace_candidate_cmp);
    candidates.dedup();
    candidates
}

fn diagnostic_namespace_candidates(
    candidates: &[NamespaceCandidateTarget],
) -> Vec<NameDiagnosticNamespaceCandidate> {
    sorted_namespace_candidates(
        candidates
            .iter()
            .map(NameDiagnosticNamespaceCandidate::from_namespace_target)
            .collect(),
    )
}

fn namespace_normalized_prefix(namespace: &UnresolvedNamespacePath) -> Vec<String> {
    let Some(partial) = namespace.partial() else {
        return Vec::new();
    };
    let mut prefix = Vec::new();
    if partial.origin() == NamespacePartialOrigin::ReservedRoot
        && let Some(root) = namespace.segments().first()
    {
        prefix.push(root.spelling().to_owned());
    }
    prefix.extend(partial.matched_prefix().iter().cloned());
    prefix
}

pub(super) fn name_diagnostic_cmp(left: &NameDiagnostic, right: &NameDiagnostic) -> Ordering {
    source_range_cmp(left.root_range(), right.root_range())
        .then_with(|| left.role().cmp(&right.role()))
        .then_with(|| {
            name_diagnostic_kind_name(left.kind()).cmp(name_diagnostic_kind_name(right.kind()))
        })
        .then_with(|| left.attempted_spelling().cmp(right.attempted_spelling()))
        .then_with(|| {
            name_diagnostic_candidate_key(left).cmp(&name_diagnostic_candidate_key(right))
        })
        .then_with(|| source_range_cmp(left.range(), right.range()))
        .then_with(|| {
            left.name_ref()
                .map(NameRefId::index)
                .cmp(&right.name_ref().map(NameRefId::index))
        })
        .then_with(|| left.id().cmp(&right.id()))
}

fn diagnostic_root_cmp(left: &DiagnosticRootState, right: &DiagnosticRootState) -> Ordering {
    source_range_cmp(left.range, right.range)
        .then_with(|| {
            name_diagnostic_kind_name(left.kind).cmp(name_diagnostic_kind_name(right.kind))
        })
        .then_with(|| left.attempted_spelling.cmp(&right.attempted_spelling))
        .then_with(|| {
            diagnostic_root_candidate_key(left).cmp(&diagnostic_root_candidate_key(right))
        })
        .then_with(|| left.key.cmp(&right.key))
}

fn name_diagnostic_candidate_key(diagnostic: &NameDiagnostic) -> Vec<String> {
    let mut keys = diagnostic
        .symbol_candidates()
        .iter()
        .map(symbol_candidate_key)
        .chain(
            diagnostic
                .namespace_candidates()
                .iter()
                .map(namespace_candidate_key),
        )
        .collect::<Vec<_>>();
    keys.sort();
    keys
}

fn diagnostic_root_candidate_key(root: &DiagnosticRootState) -> Vec<String> {
    let mut keys = root
        .symbol_candidates
        .iter()
        .map(symbol_candidate_key)
        .chain(
            root.namespace_candidates
                .iter()
                .map(namespace_candidate_key),
        )
        .collect::<Vec<_>>();
    keys.sort();
    keys
}

fn symbol_candidate_key(candidate: &NameResolutionCandidate) -> String {
    format!(
        "symbol:{}:{}:{}:{}",
        candidate.symbol().fqn().as_str(),
        candidate.symbol().module().package().as_str(),
        candidate.symbol().module().path().as_str(),
        range_key_string(candidate.range())
    )
}

fn namespace_candidate_key(candidate: &NameDiagnosticNamespaceCandidate) -> String {
    format!(
        "namespace:{}:{}:{}:{}",
        candidate.stable_variant(),
        candidate.target().package().as_str(),
        candidate.target().path().as_str(),
        range_key_string(candidate.range())
    )
}

fn name_diagnostic_kind_name(kind: NameDiagnosticKind) -> &'static str {
    match kind {
        NameDiagnosticKind::UnresolvedName { lookup } => match lookup {
            NameLookupClass::Module => "unresolved-name.module",
            NameLookupClass::Namespace => "unresolved-name.namespace",
            NameLookupClass::Symbol => "unresolved-name.symbol",
            NameLookupClass::Builtin => "unresolved-name.builtin",
            NameLookupClass::Selector => "unresolved-name.selector",
        },
        NameDiagnosticKind::AmbiguousName => "ambiguous-name",
        NameDiagnosticKind::UnresolvedNamespace { class } => match class {
            NamespaceFailureClass::EmptyPath => "unresolved-namespace.empty-path",
            NamespaceFailureClass::RecoveredSyntax => "unresolved-namespace.recovered-syntax",
            NamespaceFailureClass::UnknownNamespaceSegment => {
                "unresolved-namespace.unknown-segment"
            }
            NamespaceFailureClass::UnknownModule => "unresolved-namespace.unknown-module",
            NamespaceFailureClass::AmbiguousImportAlias => {
                "unresolved-namespace.ambiguous-import-alias"
            }
            NamespaceFailureClass::UnresolvedImportAlias => {
                "unresolved-namespace.unresolved-import-alias"
            }
            NamespaceFailureClass::ProviderError => "unresolved-namespace.provider-error",
            NamespaceFailureClass::IllegalCandidateState => {
                "unresolved-namespace.illegal-candidate-state"
            }
        },
        NameDiagnosticKind::AmbiguousNamespace { class } => match class {
            NamespaceFailureClass::EmptyPath => "ambiguous-namespace.empty-path",
            NamespaceFailureClass::RecoveredSyntax => "ambiguous-namespace.recovered-syntax",
            NamespaceFailureClass::UnknownNamespaceSegment => "ambiguous-namespace.unknown-segment",
            NamespaceFailureClass::UnknownModule => "ambiguous-namespace.unknown-module",
            NamespaceFailureClass::AmbiguousImportAlias => {
                "ambiguous-namespace.ambiguous-import-alias"
            }
            NamespaceFailureClass::UnresolvedImportAlias => {
                "ambiguous-namespace.unresolved-import-alias"
            }
            NamespaceFailureClass::ProviderError => "ambiguous-namespace.provider-error",
            NamespaceFailureClass::IllegalCandidateState => {
                "ambiguous-namespace.illegal-candidate-state"
            }
        },
        NameDiagnosticKind::UnresolvedImportAliasDependency { class } => match class {
            ImportPathFailureClass::EmptyPath => "unresolved-import-alias.empty-path",
            ImportPathFailureClass::UnknownNamespaceOrPackage => {
                "unresolved-import-alias.unknown-namespace-or-package"
            }
            ImportPathFailureClass::UnknownModule => "unresolved-import-alias.unknown-module",
            ImportPathFailureClass::RelativePathEscapesPackage => {
                "unresolved-import-alias.relative-path-escapes-package"
            }
            ImportPathFailureClass::RecoveredSyntax => "unresolved-import-alias.recovered-syntax",
            ImportPathFailureClass::DuplicateAlias => "unresolved-import-alias.duplicate-alias",
            ImportPathFailureClass::AliasRootConflict => {
                "unresolved-import-alias.alias-root-conflict"
            }
            ImportPathFailureClass::IllegalCandidateState => {
                "unresolved-import-alias.illegal-candidate-state"
            }
        },
    }
}

fn diagnostic_namespace_candidate_cmp(
    left: &NameDiagnosticNamespaceCandidate,
    right: &NameDiagnosticNamespaceCandidate,
) -> Ordering {
    left.stable_variant()
        .cmp(right.stable_variant())
        .then_with(|| left.target().cmp(right.target()))
        .then_with(|| left.ordinal().cmp(&right.ordinal()))
        .then_with(|| range_key(left.range()).cmp(&range_key(right.range())))
}
