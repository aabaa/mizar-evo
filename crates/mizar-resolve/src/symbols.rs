//! Opaque symbol/signature collection skeleton.
//!
//! This module implements the R-020 declaration-symbol skeleton over explicit
//! declaration projections. Parser-backed per-kind spelling and signature
//! extraction remains R-021 work.

use crate::declarations::{
    DeclarationShell, DeclarationShellId, DeclarationShellKind, DeclarationShellSet,
    DeclarationShellVisibilityState,
};
use crate::env::{
    ContributionKind, DeclarationConflictClass, DefinitionKind, DefinitionShell,
    DiagnosticAnchorId, ExportStatus, NamespacePath, OverloadKey, RegistrationKind, SignatureShell,
    SourceContributionId, SymbolEntry, SymbolEnv, SymbolEnvIndexes, SymbolKind, Visibility,
};
use crate::resolved_ast::{FullyQualifiedName, LocalSymbolId, ModuleId, SemanticOrigin, SymbolId};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// Duplicate and overload policy for an opaque declaration projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SymbolOverloadPolicy {
    /// Declarations with the same namespace, spelling, and kind conflict.
    NonOverloadable,
    /// Same-spelling declarations may form an overload candidate group.
    Overloadable,
    /// The projection participates in an illegal overload group diagnostic.
    IllegalGroup,
}

/// Explicit declaration projection consumed by the R-020 skeleton.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolDeclarationProjection {
    shell: DeclarationShellId,
    namespace: NamespacePath,
    primary_spelling: String,
    notation_spelling: Option<String>,
    symbol_kind: SymbolKind,
    definition_kind: Option<DefinitionKind>,
    registration_kind: Option<RegistrationKind>,
    arity: Option<u32>,
    overload_policy: SymbolOverloadPolicy,
    identity_slot: Option<String>,
    signature: Option<SignatureShell>,
}

impl SymbolDeclarationProjection {
    /// Creates a projection for a symbol-bearing declaration shell.
    #[must_use]
    pub fn new(
        shell: DeclarationShellId,
        namespace: NamespacePath,
        primary_spelling: impl Into<String>,
        symbol_kind: SymbolKind,
    ) -> Self {
        Self {
            shell,
            namespace,
            primary_spelling: primary_spelling.into(),
            notation_spelling: None,
            symbol_kind,
            definition_kind: None,
            registration_kind: None,
            arity: None,
            overload_policy: SymbolOverloadPolicy::NonOverloadable,
            identity_slot: None,
            signature: None,
        }
    }

    /// Sets notation spelling used for symbol and overload grouping.
    #[must_use]
    pub fn with_notation_spelling(mut self, notation_spelling: impl Into<String>) -> Self {
        self.notation_spelling = Some(notation_spelling.into());
        self
    }

    /// Sets the matching definition-kind shell.
    #[must_use]
    pub const fn with_definition_kind(mut self, definition_kind: DefinitionKind) -> Self {
        self.definition_kind = Some(definition_kind);
        self
    }

    /// Sets the matching registration-kind shell.
    #[must_use]
    pub const fn with_registration_kind(mut self, registration_kind: RegistrationKind) -> Self {
        self.registration_kind = Some(registration_kind);
        self.definition_kind = Some(DefinitionKind::Registration);
        self.symbol_kind = SymbolKind::Registration;
        self
    }

    /// Sets syntactic arity when available without type checking.
    #[must_use]
    pub const fn with_arity(mut self, arity: u32) -> Self {
        self.arity = Some(arity);
        self
    }

    /// Sets duplicate/overload grouping policy.
    #[must_use]
    pub const fn with_overload_policy(mut self, overload_policy: SymbolOverloadPolicy) -> Self {
        self.overload_policy = overload_policy;
        self
    }

    /// Sets an explicit relation/member/contract slot for stable identity.
    #[must_use]
    pub fn with_identity_slot(mut self, identity_slot: impl Into<String>) -> Self {
        self.identity_slot = Some(identity_slot.into());
        self
    }

    /// Sets an opaque signature shell.
    #[must_use]
    pub fn with_signature(mut self, signature: SignatureShell) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Returns the declaration shell id.
    #[must_use]
    pub const fn shell(&self) -> DeclarationShellId {
        self.shell
    }

    /// Returns the namespace path.
    #[must_use]
    pub const fn namespace(&self) -> &NamespacePath {
        &self.namespace
    }

    /// Returns the projected primary spelling.
    #[must_use]
    pub fn primary_spelling(&self) -> &str {
        &self.primary_spelling
    }

    /// Returns the projected symbol kind.
    #[must_use]
    pub const fn symbol_kind(&self) -> SymbolKind {
        self.symbol_kind
    }

    /// Returns the projected definition kind when this declaration contributes
    /// a definition shell.
    #[must_use]
    pub const fn definition_kind(&self) -> Option<DefinitionKind> {
        self.definition_kind
    }

    /// Returns the registration kind when this declaration contributes a
    /// registration entry.
    #[must_use]
    pub const fn registration_kind(&self) -> Option<RegistrationKind> {
        self.registration_kind
    }

    /// Returns the overload grouping policy.
    #[must_use]
    pub const fn overload_policy(&self) -> SymbolOverloadPolicy {
        self.overload_policy
    }
}

/// Crate-local/internal symbol collection diagnostic class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SymbolDiagnosticClass {
    /// A projection referenced a declaration shell that is not present.
    MissingShell,
    /// A context-only shell received a symbol projection.
    ContextOnlyShell,
    /// Duplicate declaration that cannot form an overload group.
    DuplicateDeclaration,
    /// Illegal overload group.
    IllegalOverloadGroup,
}

/// Crate-local/internal symbol collection diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolDiagnostic {
    id: DiagnosticAnchorId,
    class: SymbolDiagnosticClass,
    shell: Option<DeclarationShellId>,
    spelling: String,
    range: SourceRange,
    candidates: Vec<SymbolId>,
}

impl SymbolDiagnostic {
    /// Returns the diagnostic anchor id.
    #[must_use]
    pub const fn id(&self) -> DiagnosticAnchorId {
        self.id
    }

    /// Returns the diagnostic class.
    #[must_use]
    pub const fn class(&self) -> SymbolDiagnosticClass {
        self.class
    }

    /// Returns the source declaration shell when available.
    #[must_use]
    pub const fn shell(&self) -> Option<DeclarationShellId> {
        self.shell
    }

    /// Returns the primary spelling associated with the diagnostic.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the primary source range.
    #[must_use]
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns candidate symbols in deterministic order.
    #[must_use]
    pub fn candidates(&self) -> &[SymbolId] {
        &self.candidates
    }
}

impl Ord for SymbolDiagnostic {
    fn cmp(&self, other: &Self) -> Ordering {
        range_key(self.range)
            .cmp(&range_key(other.range))
            .then_with(|| self.class.cmp(&other.class))
            .then_with(|| self.spelling.cmp(&other.spelling))
            .then_with(|| self.candidates.cmp(&other.candidates))
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for SymbolDiagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Result of R-020 symbol/signature collection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolCollectionResult {
    env: SymbolEnv,
    diagnostics: Vec<SymbolDiagnostic>,
}

impl SymbolCollectionResult {
    /// Returns the collected symbol environment.
    #[must_use]
    pub const fn env(&self) -> &SymbolEnv {
        &self.env
    }

    /// Returns crate-local/internal diagnostics in deterministic order.
    #[must_use]
    pub fn diagnostics(&self) -> &[SymbolDiagnostic] {
        &self.diagnostics
    }

    /// Consumes this result and returns its symbol environment.
    #[must_use]
    pub fn into_env(self) -> SymbolEnv {
        self.env
    }
}

/// Opaque declaration symbol collector.
#[derive(Debug, Clone)]
pub struct SymbolCollector<'a> {
    source_id: SourceId,
    module: &'a ModuleId,
    shells: &'a DeclarationShellSet,
    projections: &'a [SymbolDeclarationProjection],
}

impl<'a> SymbolCollector<'a> {
    /// Creates a symbol collector over declaration shells and explicit
    /// symbol-bearing projections.
    #[must_use]
    pub const fn new(
        source_id: SourceId,
        module: &'a ModuleId,
        shells: &'a DeclarationShellSet,
        projections: &'a [SymbolDeclarationProjection],
    ) -> Self {
        Self {
            source_id,
            module,
            shells,
            projections,
        }
    }

    /// Collects opaque symbol entries and duplicate/overload metadata.
    #[must_use]
    pub fn collect(self) -> SymbolCollectionResult {
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            self.module.clone(),
            ContributionKind::LocalSource {
                source_id: self.source_id,
            },
            self.contribution_anchor(),
        );

        let mut collected = Vec::new();
        let mut diagnostic_drafts = Vec::new();

        for projection in sorted_projections(self.projections) {
            let Some(shell) = self.shells.declaration(projection.shell()) else {
                diagnostic_drafts.push(DiagnosticDraft::missing_shell(projection, self.source_id));
                continue;
            };
            if !symbol_bearing_shell(shell.kind()) {
                diagnostic_drafts.push(DiagnosticDraft::context_only(shell, projection));
                continue;
            }
            collected.push(CollectedProjection::new(
                self.source_id,
                self.module,
                self.shells,
                shell,
                projection,
                contribution,
            ));
        }

        let conflicts = classify_conflicts(&collected, &mut diagnostic_drafts);
        let (diagnostics, diagnostic_by_group) = finalize_diagnostics(diagnostic_drafts);
        for diagnostic in &diagnostics {
            indexes
                .contributions
                .add_diagnostic(contribution, diagnostic.id());
        }

        for item in &collected {
            self.insert_symbol(&mut indexes, item, conflicts.get(&item.symbol));
        }

        insert_overload_groups(&mut indexes, &collected, contribution, &diagnostic_by_group);

        SymbolCollectionResult {
            env: SymbolEnv::new(self.module.clone(), indexes),
            diagnostics,
        }
    }

    fn contribution_anchor(&self) -> SourceAnchor {
        self.shells
            .declarations()
            .first()
            .map(|shell| SourceAnchor::Range(shell.range()))
            .unwrap_or(SourceAnchor::Point {
                source_id: self.source_id,
                offset: 0,
            })
    }

    fn insert_symbol(
        &self,
        indexes: &mut SymbolEnvIndexes,
        item: &CollectedProjection,
        conflict: Option<&DeclarationConflictClass>,
    ) {
        let signature = item.signature.clone();
        let mut symbol_entry = SymbolEntry::new(
            item.symbol.clone(),
            item.projection.symbol_kind(),
            item.projection.namespace().clone(),
            item.projection.primary_spelling(),
            item.origin.clone(),
            item.contribution,
        )
        .with_visibility(item.visibility)
        .with_export_status(item.export_status)
        .with_signature(signature.clone());
        if let Some(notation) = &item.projection.notation_spelling {
            symbol_entry = symbol_entry.with_notation_spelling(notation.clone());
        }
        indexes.symbols.insert(symbol_entry);
        indexes
            .contributions
            .add_symbol(item.contribution, item.symbol.clone());

        if let Some(definition_kind) = item.projection.definition_kind() {
            let mut definition = DefinitionShell::new(
                item.symbol.clone(),
                definition_kind,
                item.origin.clone(),
                item.contribution,
            )
            .with_visibility(item.visibility)
            .with_signature(signature.clone());
            if let Some(arity) = item.projection.arity {
                definition = definition.with_arity(arity);
            }
            if let Some(notation) = &item.projection.notation_spelling {
                definition = definition.with_notation_shape(notation.clone());
            }
            if item.recovered {
                definition = definition.with_conflict(DeclarationConflictClass::RecoveredShell);
            } else if let Some(conflict) = conflict {
                definition = definition.with_conflict(conflict.clone());
            }
            let definition_id = indexes.definitions.insert(definition);
            indexes
                .contributions
                .add_definition(item.contribution, definition_id);
        }

        if let Some(registration_kind) = item.projection.registration_kind() {
            let registration_id = indexes.registrations.insert(
                Some(item.symbol.clone()),
                registration_kind,
                signature,
                item.origin.clone(),
                item.contribution,
            );
            if let Some(registration) = indexes.registrations.get_mut(registration_id) {
                registration
                    .set_visibility(item.visibility)
                    .set_export_status(item.export_status);
            }
            indexes
                .contributions
                .add_registration(item.contribution, registration_id);
        }
    }
}

#[derive(Debug, Clone)]
struct CollectedProjection<'a> {
    shell: &'a DeclarationShell,
    projection: &'a SymbolDeclarationProjection,
    symbol: SymbolId,
    origin: SemanticOrigin,
    signature: SignatureShell,
    visibility: Visibility,
    export_status: ExportStatus,
    recovered: bool,
    contribution: SourceContributionId,
}

impl<'a> CollectedProjection<'a> {
    fn new(
        source_id: SourceId,
        module: &ModuleId,
        shells: &DeclarationShellSet,
        shell: &'a DeclarationShell,
        projection: &'a SymbolDeclarationProjection,
        contribution: SourceContributionId,
    ) -> Self {
        let context = shell_context(shells, shell);
        let origin = shell_origin(source_id, module, shell, &context);
        let visibility = context.visibility;
        let export_status = shell_export_status(visibility, context.recovered);
        let symbol = symbol_id(module, shell, projection, contribution, &context);
        let signature = projection
            .signature
            .clone()
            .unwrap_or_else(|| default_signature(shell, projection, &context));
        Self {
            shell,
            projection,
            symbol,
            origin,
            signature,
            visibility,
            export_status,
            recovered: context.recovered,
            contribution,
        }
    }

    fn conflict_key(&self) -> ConflictKey {
        ConflictKey {
            namespace: self.projection.namespace().clone(),
            spelling: self.projection.primary_spelling().to_owned(),
            kind: self.projection.symbol_kind(),
        }
    }

    fn overload_key(&self) -> OverloadKey {
        OverloadKey::new(
            self.projection.namespace().clone(),
            overload_spelling(self.projection),
            self.projection.symbol_kind(),
            self.projection.arity,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ConflictKey {
    namespace: NamespacePath,
    spelling: String,
    kind: SymbolKind,
}

#[derive(Debug, Clone)]
struct DiagnosticDraft {
    class: SymbolDiagnosticClass,
    shell: Option<DeclarationShellId>,
    spelling: String,
    range: SourceRange,
    candidates: Vec<SymbolId>,
    overload_key: Option<OverloadKey>,
}

impl DiagnosticDraft {
    fn missing_shell(projection: &SymbolDeclarationProjection, source_id: SourceId) -> Self {
        Self {
            class: SymbolDiagnosticClass::MissingShell,
            shell: Some(projection.shell()),
            spelling: projection.primary_spelling().to_owned(),
            range: SourceRange {
                source_id,
                start: 0,
                end: 0,
            },
            candidates: Vec::new(),
            overload_key: None,
        }
    }

    fn context_only(shell: &DeclarationShell, projection: &SymbolDeclarationProjection) -> Self {
        Self {
            class: SymbolDiagnosticClass::ContextOnlyShell,
            shell: Some(shell.id()),
            spelling: projection.primary_spelling().to_owned(),
            range: shell.range(),
            candidates: Vec::new(),
            overload_key: None,
        }
    }

    fn duplicate(candidates: &[&CollectedProjection<'_>]) -> Self {
        let first = candidates[0];
        Self {
            class: SymbolDiagnosticClass::DuplicateDeclaration,
            shell: Some(first.shell.id()),
            spelling: first.projection.primary_spelling().to_owned(),
            range: first.shell.range(),
            candidates: sorted_symbols(candidates),
            overload_key: None,
        }
    }

    fn illegal_overload(candidates: &[&CollectedProjection<'_>]) -> Self {
        let first = candidates[0];
        Self {
            class: SymbolDiagnosticClass::IllegalOverloadGroup,
            shell: Some(first.shell.id()),
            spelling: overload_spelling(first.projection),
            range: first.shell.range(),
            candidates: sorted_symbols(candidates),
            overload_key: Some(first.overload_key()),
        }
    }
}

fn classify_conflicts(
    collected: &[CollectedProjection<'_>],
    diagnostic_drafts: &mut Vec<DiagnosticDraft>,
) -> BTreeMap<SymbolId, DeclarationConflictClass> {
    let mut conflicts = classify_illegal_overload_groups(collected, diagnostic_drafts);
    let mut groups: BTreeMap<ConflictKey, Vec<&CollectedProjection<'_>>> = BTreeMap::new();
    for item in collected {
        if item.projection.overload_policy() == SymbolOverloadPolicy::Overloadable {
            continue;
        }
        groups.entry(item.conflict_key()).or_default().push(item);
    }

    for mut candidates in groups.into_values() {
        candidates.sort_by(collected_projection_cmp);
        if candidates.len() < 2 {
            continue;
        }
        if candidates
            .iter()
            .any(|candidate| conflicts.contains_key(&candidate.symbol))
        {
            continue;
        }

        diagnostic_drafts.push(DiagnosticDraft::duplicate(&candidates));
        for candidate in candidates {
            conflicts.insert(
                candidate.symbol.clone(),
                DeclarationConflictClass::DuplicateSpelling,
            );
        }
    }
    conflicts
}

fn classify_illegal_overload_groups(
    collected: &[CollectedProjection<'_>],
    diagnostic_drafts: &mut Vec<DiagnosticDraft>,
) -> BTreeMap<SymbolId, DeclarationConflictClass> {
    let mut groups: BTreeMap<OverloadKey, Vec<&CollectedProjection<'_>>> = BTreeMap::new();
    for item in collected {
        if item.projection.overload_policy() == SymbolOverloadPolicy::IllegalGroup {
            groups.entry(item.overload_key()).or_default().push(item);
        }
    }

    let mut conflicts = BTreeMap::new();
    for mut candidates in groups.into_values() {
        candidates.sort_by(collected_projection_cmp);
        if candidates.len() < 2 {
            continue;
        }
        diagnostic_drafts.push(DiagnosticDraft::illegal_overload(&candidates));
        for candidate in candidates {
            conflicts.insert(
                candidate.symbol.clone(),
                DeclarationConflictClass::IllegalOverloadGroup,
            );
        }
    }
    conflicts
}

fn insert_overload_groups(
    indexes: &mut SymbolEnvIndexes,
    collected: &[CollectedProjection<'_>],
    contribution: SourceContributionId,
    diagnostic_by_group: &BTreeMap<OverloadKey, DiagnosticAnchorId>,
) {
    let mut groups: BTreeMap<OverloadKey, Vec<SymbolId>> = BTreeMap::new();
    for item in collected {
        if matches!(
            item.projection.overload_policy(),
            SymbolOverloadPolicy::Overloadable | SymbolOverloadPolicy::IllegalGroup
        ) {
            groups
                .entry(item.overload_key())
                .or_default()
                .push(item.symbol.clone());
        }
    }

    for (key, mut candidates) in groups {
        candidates.sort();
        candidates.dedup();
        if candidates.len() < 2 {
            continue;
        }
        let diagnostic = diagnostic_by_group.get(&key).copied();
        let group_id = indexes.overloads.insert(key, candidates, contribution);
        if let Some(diagnostic) = diagnostic {
            indexes.overloads.add_diagnostic(group_id, diagnostic);
            indexes
                .contributions
                .add_diagnostic(contribution, diagnostic);
        }
        indexes
            .contributions
            .add_overload_group(contribution, group_id);
    }
}

fn finalize_diagnostics(
    mut drafts: Vec<DiagnosticDraft>,
) -> (
    Vec<SymbolDiagnostic>,
    BTreeMap<OverloadKey, DiagnosticAnchorId>,
) {
    for draft in &mut drafts {
        draft.candidates.sort();
        draft.candidates.dedup();
    }
    drafts.sort_by(diagnostic_draft_cmp);
    let mut diagnostic_by_group = BTreeMap::new();
    let diagnostics = drafts
        .into_iter()
        .enumerate()
        .map(|(index, draft)| {
            let id = DiagnosticAnchorId::new(index);
            if let Some(overload_key) = draft.overload_key {
                diagnostic_by_group.insert(overload_key, id);
            }
            SymbolDiagnostic {
                id,
                class: draft.class,
                shell: draft.shell,
                spelling: draft.spelling,
                range: draft.range,
                candidates: draft.candidates,
            }
        })
        .collect();
    (diagnostics, diagnostic_by_group)
}

fn sorted_projections(
    projections: &[SymbolDeclarationProjection],
) -> Vec<&SymbolDeclarationProjection> {
    let mut sorted = projections.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| {
        left.shell()
            .cmp(&right.shell())
            .then_with(|| left.namespace().cmp(right.namespace()))
            .then_with(|| left.primary_spelling().cmp(right.primary_spelling()))
            .then_with(|| left.symbol_kind().cmp(&right.symbol_kind()))
            .then_with(|| left.notation_spelling.cmp(&right.notation_spelling))
            .then_with(|| left.definition_kind.cmp(&right.definition_kind))
            .then_with(|| left.registration_kind.cmp(&right.registration_kind))
            .then_with(|| left.arity.cmp(&right.arity))
            .then_with(|| left.overload_policy.cmp(&right.overload_policy))
            .then_with(|| left.identity_slot.cmp(&right.identity_slot))
            .then_with(|| left.signature.cmp(&right.signature))
    });
    sorted
}

fn symbol_bearing_shell(kind: DeclarationShellKind) -> bool {
    matches!(
        kind,
        DeclarationShellKind::Theorem
            | DeclarationShellKind::Lemma
            | DeclarationShellKind::AttributeDefinition
            | DeclarationShellKind::PredicateDefinition
            | DeclarationShellKind::FunctorDefinition
            | DeclarationShellKind::ModeDefinition
            | DeclarationShellKind::StructureDefinition
            | DeclarationShellKind::AlgorithmDefinition
            | DeclarationShellKind::AttributeRedefinition
            | DeclarationShellKind::PredicateRedefinition
            | DeclarationShellKind::FunctorRedefinition
            | DeclarationShellKind::NotationAlias
            | DeclarationShellKind::PropertyClause
            | DeclarationShellKind::StructureField
            | DeclarationShellKind::StructureProperty
            | DeclarationShellKind::InheritanceDefinition
            | DeclarationShellKind::FieldRedefinition
            | DeclarationShellKind::PropertyRedefinition
            | DeclarationShellKind::ExistentialRegistration
            | DeclarationShellKind::ConditionalRegistration
            | DeclarationShellKind::FunctorialRegistration
            | DeclarationShellKind::ReductionRegistration
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShellContext {
    visibility: Visibility,
    recovered: bool,
    structural_path: Vec<StructuralPathStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StructuralPathStep {
    kind: DeclarationShellKind,
    sibling_ordinal: usize,
}

fn shell_context(shells: &DeclarationShellSet, shell: &DeclarationShell) -> ShellContext {
    let mut visibility = Visibility::Public;
    let mut recovered = false;
    let mut current = Some(shell.id());
    while let Some(id) = current {
        let Some(context_shell) = shells.declaration(id) else {
            break;
        };
        recovered = recovered
            || context_shell.recovered()
            || matches!(
                context_shell.visibility().state(),
                DeclarationShellVisibilityState::Recovered
            );
        if matches!(
            context_shell.visibility().state(),
            DeclarationShellVisibilityState::Private | DeclarationShellVisibilityState::Recovered
        ) {
            visibility = Visibility::Private;
        }
        current = context_shell.parent();
    }

    ShellContext {
        visibility,
        recovered,
        structural_path: structural_path(shells, shell),
    }
}

const fn shell_export_status(visibility: Visibility, recovered: bool) -> ExportStatus {
    if recovered {
        return ExportStatus::LocalOnly;
    }
    match visibility {
        Visibility::Public => ExportStatus::Exported,
        Visibility::Private => ExportStatus::LocalOnly,
    }
}

fn shell_origin(
    source_id: SourceId,
    module: &ModuleId,
    shell: &DeclarationShell,
    context: &ShellContext,
) -> SemanticOrigin {
    let origin = SemanticOrigin::new(
        source_id,
        module.clone(),
        SourceAnchor::Range(shell.range()),
        origin_structural_path(&context.structural_path),
    );
    if context.recovered {
        origin.recovered()
    } else {
        origin
    }
}

fn symbol_id(
    module: &ModuleId,
    shell: &DeclarationShell,
    projection: &SymbolDeclarationProjection,
    contribution: SourceContributionId,
    context: &ShellContext,
) -> SymbolId {
    let kind = symbol_kind_key(projection.symbol_kind());
    let namespace = escape_component(projection.namespace().as_str());
    let primary = escape_component(projection.primary_spelling());
    let notation = projection
        .notation_spelling
        .as_deref()
        .map(escape_component)
        .unwrap_or_else(|| "_".to_owned());
    let slot = identity_slot(projection);
    let owner = structural_path_key(&context.structural_path);
    let local = format!(
        concat!(
            "contribution={}:namespace={}:owner={}:shell={}:kind={}:",
            "name={}:notation={}:arity={}:definition={}:registration={}:policy={}:slot={}"
        ),
        contribution.index(),
        namespace,
        owner,
        declaration_shell_kind_key(shell.kind()),
        kind,
        primary,
        notation,
        projection
            .arity
            .map(|arity| arity.to_string())
            .unwrap_or_else(|| "_".to_owned()),
        projection
            .definition_kind
            .map(definition_kind_key)
            .unwrap_or("_"),
        projection
            .registration_kind
            .map(registration_kind_key)
            .unwrap_or("_"),
        overload_policy_key(projection.overload_policy),
        slot
    );
    let fqn = format!(
        "{}::{}::{}",
        module.package().as_str(),
        module.path().as_str(),
        local
    );
    SymbolId::new(
        module.clone(),
        LocalSymbolId::new(local),
        FullyQualifiedName::new(fqn),
    )
}

fn default_signature(
    shell: &DeclarationShell,
    projection: &SymbolDeclarationProjection,
    context: &ShellContext,
) -> SignatureShell {
    if context.recovered {
        SignatureShell::Malformed {
            class: "recovered-shell".to_owned(),
        }
    } else {
        SignatureShell::Opaque {
            schema: "symbol-collection-v1".to_owned(),
            payload: format!(
                "{}:{}:{}:{}",
                declaration_shell_kind_key(shell.kind()),
                projection.primary_spelling(),
                structural_path_key(&context.structural_path),
                identity_slot(projection)
            ),
        }
    }
}

fn structural_path(
    shells: &DeclarationShellSet,
    shell: &DeclarationShell,
) -> Vec<StructuralPathStep> {
    let mut ids = Vec::new();
    let mut current = Some(shell.id());
    while let Some(id) = current {
        let Some(context_shell) = shells.declaration(id) else {
            break;
        };
        ids.push(id);
        current = context_shell.parent();
    }
    ids.reverse();
    ids.into_iter()
        .filter_map(|id| shells.declaration(id))
        .map(|context_shell| StructuralPathStep {
            kind: context_shell.kind(),
            sibling_ordinal: sibling_ordinal(shells, context_shell),
        })
        .collect()
}

fn sibling_ordinal(shells: &DeclarationShellSet, shell: &DeclarationShell) -> usize {
    shells
        .declarations()
        .iter()
        .filter(|candidate| candidate.parent() == shell.parent())
        .filter(|candidate| candidate.ordinal() <= shell.ordinal())
        .count()
        .saturating_sub(1)
}

fn origin_structural_path(path: &[StructuralPathStep]) -> Vec<u32> {
    path.iter()
        .flat_map(|step| {
            [
                declaration_shell_kind_code(step.kind),
                usize_to_u32(step.sibling_ordinal),
            ]
        })
        .collect()
}

fn structural_path_key(path: &[StructuralPathStep]) -> String {
    path.iter()
        .map(|step| {
            format!(
                "{}#{}",
                declaration_shell_kind_key(step.kind),
                step.sibling_ordinal
            )
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn identity_slot(projection: &SymbolDeclarationProjection) -> String {
    projection
        .identity_slot
        .as_deref()
        .map(escape_component)
        .unwrap_or_else(|| {
            format!(
                "{}:{}:{}:{}",
                overload_policy_key(projection.overload_policy),
                projection
                    .arity
                    .map(|arity| arity.to_string())
                    .unwrap_or_else(|| "_".to_owned()),
                projection
                    .definition_kind
                    .map(definition_kind_key)
                    .unwrap_or("_"),
                projection
                    .registration_kind
                    .map(registration_kind_key)
                    .unwrap_or("_")
            )
        })
}

fn escape_component(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(':', "\\c")
        .replace('|', "\\p")
        .replace('/', "\\s")
}

fn overload_spelling(projection: &SymbolDeclarationProjection) -> String {
    projection
        .notation_spelling
        .clone()
        .unwrap_or_else(|| projection.primary_spelling().to_owned())
}

fn sorted_symbols(candidates: &[&CollectedProjection<'_>]) -> Vec<SymbolId> {
    let mut symbols = candidates
        .iter()
        .map(|candidate| candidate.symbol.clone())
        .collect::<Vec<_>>();
    symbols.sort();
    symbols
}

fn collected_projection_cmp(
    left: &&CollectedProjection<'_>,
    right: &&CollectedProjection<'_>,
) -> Ordering {
    left.shell
        .ordinal()
        .cmp(&right.shell.ordinal())
        .then_with(|| left.symbol.cmp(&right.symbol))
}

fn diagnostic_draft_cmp(left: &DiagnosticDraft, right: &DiagnosticDraft) -> Ordering {
    range_key(left.range)
        .cmp(&range_key(right.range))
        .then_with(|| left.class.cmp(&right.class))
        .then_with(|| left.spelling.cmp(&right.spelling))
        .then_with(|| left.candidates.cmp(&right.candidates))
        .then_with(|| left.shell.cmp(&right.shell))
}

const fn range_key(range: SourceRange) -> (usize, usize) {
    (range.start, range.end)
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn symbol_kind_key(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Predicate => "predicate",
        SymbolKind::Functor => "functor",
        SymbolKind::Mode => "mode",
        SymbolKind::Attribute => "attribute",
        SymbolKind::Structure => "structure",
        SymbolKind::Selector => "selector",
        SymbolKind::Registration => "registration",
        SymbolKind::Theorem => "theorem",
        SymbolKind::Lemma => "lemma",
        SymbolKind::Algorithm => "algorithm",
        SymbolKind::Scheme => "scheme",
        SymbolKind::Template => "template",
        SymbolKind::Synonym => "synonym",
        SymbolKind::Antonym => "antonym",
        SymbolKind::Redefinition => "redefinition",
        SymbolKind::Builtin => "builtin",
    }
}

fn definition_kind_key(kind: DefinitionKind) -> &'static str {
    match kind {
        DefinitionKind::Predicate => "predicate",
        DefinitionKind::Functor => "functor",
        DefinitionKind::Mode => "mode",
        DefinitionKind::Attribute => "attribute",
        DefinitionKind::Structure => "structure",
        DefinitionKind::Registration => "registration",
        DefinitionKind::Theorem => "theorem",
        DefinitionKind::Lemma => "lemma",
        DefinitionKind::Algorithm => "algorithm",
        DefinitionKind::Scheme => "scheme",
        DefinitionKind::Template => "template",
        DefinitionKind::Synonym => "synonym",
        DefinitionKind::Antonym => "antonym",
        DefinitionKind::Redefinition => "redefinition",
        DefinitionKind::Selector => "selector",
    }
}

fn registration_kind_key(kind: RegistrationKind) -> &'static str {
    match kind {
        RegistrationKind::Cluster => "cluster",
        RegistrationKind::Identify => "identify",
        RegistrationKind::Reduction => "reduction",
        RegistrationKind::Property => "property",
    }
}

fn overload_policy_key(policy: SymbolOverloadPolicy) -> &'static str {
    match policy {
        SymbolOverloadPolicy::NonOverloadable => "non-overloadable",
        SymbolOverloadPolicy::Overloadable => "overloadable",
        SymbolOverloadPolicy::IllegalGroup => "illegal-group",
    }
}

const fn declaration_shell_kind_code(kind: DeclarationShellKind) -> u32 {
    match kind {
        DeclarationShellKind::Placeholder => 0,
        DeclarationShellKind::Reserve => 1,
        DeclarationShellKind::Theorem => 2,
        DeclarationShellKind::Lemma => 3,
        DeclarationShellKind::DefinitionBlock => 4,
        DeclarationShellKind::RegistrationBlock => 5,
        DeclarationShellKind::ClaimBlock => 6,
        DeclarationShellKind::AttributeDefinition => 7,
        DeclarationShellKind::PredicateDefinition => 8,
        DeclarationShellKind::FunctorDefinition => 9,
        DeclarationShellKind::ModeDefinition => 10,
        DeclarationShellKind::StructureDefinition => 11,
        DeclarationShellKind::AlgorithmDefinition => 12,
        DeclarationShellKind::AttributeRedefinition => 13,
        DeclarationShellKind::PredicateRedefinition => 14,
        DeclarationShellKind::FunctorRedefinition => 15,
        DeclarationShellKind::NotationAlias => 16,
        DeclarationShellKind::PropertyClause => 17,
        DeclarationShellKind::StructureField => 18,
        DeclarationShellKind::StructureProperty => 19,
        DeclarationShellKind::InheritanceDefinition => 20,
        DeclarationShellKind::FieldRedefinition => 21,
        DeclarationShellKind::PropertyRedefinition => 22,
        DeclarationShellKind::ExistentialRegistration => 23,
        DeclarationShellKind::ConditionalRegistration => 24,
        DeclarationShellKind::FunctorialRegistration => 25,
        DeclarationShellKind::ReductionRegistration => 26,
        DeclarationShellKind::VisibilityWrapper => 27,
    }
}

fn declaration_shell_kind_key(kind: DeclarationShellKind) -> &'static str {
    match kind {
        DeclarationShellKind::Placeholder => "placeholder",
        DeclarationShellKind::Reserve => "reserve",
        DeclarationShellKind::Theorem => "theorem",
        DeclarationShellKind::Lemma => "lemma",
        DeclarationShellKind::DefinitionBlock => "definition-block",
        DeclarationShellKind::RegistrationBlock => "registration-block",
        DeclarationShellKind::ClaimBlock => "claim-block",
        DeclarationShellKind::AttributeDefinition => "attribute-definition",
        DeclarationShellKind::PredicateDefinition => "predicate-definition",
        DeclarationShellKind::FunctorDefinition => "functor-definition",
        DeclarationShellKind::ModeDefinition => "mode-definition",
        DeclarationShellKind::StructureDefinition => "structure-definition",
        DeclarationShellKind::AlgorithmDefinition => "algorithm-definition",
        DeclarationShellKind::AttributeRedefinition => "attribute-redefinition",
        DeclarationShellKind::PredicateRedefinition => "predicate-redefinition",
        DeclarationShellKind::FunctorRedefinition => "functor-redefinition",
        DeclarationShellKind::NotationAlias => "notation-alias",
        DeclarationShellKind::PropertyClause => "property-clause",
        DeclarationShellKind::StructureField => "structure-field",
        DeclarationShellKind::StructureProperty => "structure-property",
        DeclarationShellKind::InheritanceDefinition => "inheritance-definition",
        DeclarationShellKind::FieldRedefinition => "field-redefinition",
        DeclarationShellKind::PropertyRedefinition => "property-redefinition",
        DeclarationShellKind::ExistentialRegistration => "existential-registration",
        DeclarationShellKind::ConditionalRegistration => "conditional-registration",
        DeclarationShellKind::FunctorialRegistration => "functorial-registration",
        DeclarationShellKind::ReductionRegistration => "reduction-registration",
        DeclarationShellKind::VisibilityWrapper => "visibility-wrapper",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarations::DeclarationShellCollector;
    use mizar_session::{
        BuildSnapshotId, Hash, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator,
    };
    use mizar_syntax::{
        SurfaceAstBuilder, SurfaceBuilderNodeId, SurfaceNodeKind, SurfaceTokenKind,
        SyntaxRecoveryKind,
    };

    #[test]
    fn registers_opaque_symbols_definitions_and_contribution_effects() {
        let source_id = source_id();
        let shells = shells_for(
            source_id,
            vec![
                test_item(0, SurfaceNodeKind::PredicateDefinition),
                test_item(10, SurfaceNodeKind::FunctorDefinition),
                visible_test_item(20, "private", SurfaceNodeKind::ModeDefinition),
            ],
        );
        let namespace = NamespacePath::new("main");
        let projections = vec![
            projection(
                shells.declarations()[0].id(),
                namespace.clone(),
                "P",
                SymbolKind::Predicate,
                DefinitionKind::Predicate,
            ),
            projection(
                shells.declarations()[1].id(),
                namespace.clone(),
                "F",
                SymbolKind::Functor,
                DefinitionKind::Functor,
            )
            .with_notation_spelling("_ + _")
            .with_arity(2),
            projection(
                shells.declarations()[2].id(),
                namespace.clone(),
                "M",
                SymbolKind::Mode,
                DefinitionKind::Mode,
            ),
        ];

        let result = collect(source_id, &shells, &projections);
        let env = result.env();

        assert!(result.diagnostics().is_empty());
        assert_eq!(env.symbols().len(), 3);
        assert_eq!(env.definitions().len(), 3);
        assert_eq!(env.symbols().visible_candidates(&namespace, "P").len(), 1);
        assert_eq!(
            env.symbols().visible_candidates(&namespace, "F")[0].notation_spelling(),
            Some("_ + _")
        );
        assert_eq!(
            env.symbols().visible_candidates(&namespace, "M")[0].visibility(),
            Visibility::Private
        );
        assert_eq!(
            env.symbols().visible_candidates(&namespace, "M")[0].export_status(),
            ExportStatus::LocalOnly
        );
        let effects = env.contributions().iter().next().unwrap().effects();
        assert_eq!(effects.symbols().len(), 3);
        assert_eq!(effects.definitions().len(), 3);
    }

    #[test]
    fn duplicate_detection_marks_represented_kind_families_in_order() {
        let source_id = source_id();
        let cases = duplicate_cases();
        let shells = shells_for(source_id, duplicate_case_items(&cases));
        let namespace = NamespacePath::new("main");
        let mut projections = Vec::new();
        for (index, case) in cases.iter().enumerate() {
            let first = shells.declarations()[index * 2].id();
            let second = shells.declarations()[index * 2 + 1].id();
            projections.push(case.projection(second, namespace.clone()));
            projections.push(case.projection(first, namespace.clone()));
        }

        let result = collect(source_id, &shells, &projections);

        assert_eq!(result.diagnostics().len(), cases.len());
        for (diagnostic, case) in result.diagnostics().iter().zip(cases.iter()) {
            assert_eq!(
                diagnostic.class(),
                SymbolDiagnosticClass::DuplicateDeclaration
            );
            assert_eq!(diagnostic.spelling(), case.spelling);
            assert_eq!(diagnostic.candidates().len(), 2);
        }
        let conflicts = result
            .env()
            .definitions()
            .iter()
            .filter_map(|entry| entry.conflict())
            .collect::<Vec<_>>();
        assert_eq!(conflicts.len(), cases.len() * 2);
        assert!(
            conflicts
                .iter()
                .all(|conflict| **conflict == DeclarationConflictClass::DuplicateSpelling)
        );
        assert_eq!(
            result
                .env()
                .contributions()
                .iter()
                .next()
                .unwrap()
                .effects()
                .diagnostics()
                .len(),
            cases.len()
        );
    }

    #[test]
    fn overloadable_candidates_form_groups_and_illegal_groups_get_diagnostics() {
        let source_id = source_id();
        let shells = shells_for(
            source_id,
            vec![
                test_item(0, SurfaceNodeKind::FunctorDefinition),
                test_item(10, SurfaceNodeKind::FunctorDefinition),
                test_item(20, SurfaceNodeKind::PredicateDefinition),
                test_item(30, SurfaceNodeKind::PredicateDefinition),
            ],
        );
        let namespace = NamespacePath::new("main");
        let projections = vec![
            projection(
                shells.declarations()[0].id(),
                namespace.clone(),
                "F",
                SymbolKind::Functor,
                DefinitionKind::Functor,
            )
            .with_overload_policy(SymbolOverloadPolicy::Overloadable)
            .with_arity(1),
            projection(
                shells.declarations()[1].id(),
                namespace.clone(),
                "F",
                SymbolKind::Functor,
                DefinitionKind::Functor,
            )
            .with_overload_policy(SymbolOverloadPolicy::Overloadable)
            .with_arity(1),
            projection(
                shells.declarations()[2].id(),
                namespace.clone(),
                "BadLeft",
                SymbolKind::Predicate,
                DefinitionKind::Predicate,
            )
            .with_notation_spelling("_ bad _")
            .with_overload_policy(SymbolOverloadPolicy::IllegalGroup),
            projection(
                shells.declarations()[3].id(),
                namespace,
                "BadRight",
                SymbolKind::Predicate,
                DefinitionKind::Predicate,
            )
            .with_notation_spelling("_ bad _")
            .with_overload_policy(SymbolOverloadPolicy::IllegalGroup),
        ];

        let result = collect(source_id, &shells, &projections);

        assert_eq!(result.env().overloads().len(), 2);
        assert_eq!(
            result.diagnostics()[0].class(),
            SymbolDiagnosticClass::IllegalOverloadGroup
        );
        let illegal = result
            .env()
            .overloads()
            .iter()
            .find(|group| group.key().spelling() == "_ bad _")
            .unwrap();
        assert_eq!(illegal.diagnostics(), &[result.diagnostics()[0].id()]);
        let legal = result
            .env()
            .overloads()
            .iter()
            .find(|group| group.key().spelling() == "F")
            .unwrap();
        assert_eq!(legal.candidates().len(), 2);
        assert!(legal.diagnostics().is_empty());
    }

    #[test]
    fn diagnostics_are_sorted_by_range_class_spelling_and_stable_ids() {
        let source_id = source_id();
        let shells = shells_for(
            source_id,
            vec![
                test_item(0, SurfaceNodeKind::DefinitionBlockItem),
                test_item(0, SurfaceNodeKind::PredicateDefinition),
                test_item(0, SurfaceNodeKind::PredicateDefinition),
                test_item(0, SurfaceNodeKind::AttributeDefinition),
                test_item(0, SurfaceNodeKind::AttributeDefinition),
                test_item(70, SurfaceNodeKind::FunctorDefinition),
                test_item(80, SurfaceNodeKind::FunctorDefinition),
            ],
        );
        let namespace = NamespacePath::new("main");
        let projections = vec![
            projection(
                shells.declarations()[6].id(),
                namespace.clone(),
                "IllegalRight",
                SymbolKind::Functor,
                DefinitionKind::Functor,
            )
            .with_notation_spelling("_ ? _")
            .with_overload_policy(SymbolOverloadPolicy::IllegalGroup),
            projection(
                shells.declarations()[2].id(),
                namespace.clone(),
                "BDup",
                SymbolKind::Predicate,
                DefinitionKind::Predicate,
            ),
            projection(
                shells.declarations()[4].id(),
                namespace.clone(),
                "ADup",
                SymbolKind::Attribute,
                DefinitionKind::Attribute,
            ),
            projection(
                shells.declarations()[0].id(),
                namespace.clone(),
                "Context",
                SymbolKind::Structure,
                DefinitionKind::Structure,
            ),
            projection(
                shells.declarations()[5].id(),
                namespace.clone(),
                "IllegalLeft",
                SymbolKind::Functor,
                DefinitionKind::Functor,
            )
            .with_notation_spelling("_ ? _")
            .with_overload_policy(SymbolOverloadPolicy::IllegalGroup),
            projection(
                shells.declarations()[1].id(),
                namespace.clone(),
                "BDup",
                SymbolKind::Predicate,
                DefinitionKind::Predicate,
            ),
            projection(
                shells.declarations()[3].id(),
                namespace,
                "ADup",
                SymbolKind::Attribute,
                DefinitionKind::Attribute,
            ),
        ];

        let result = collect(source_id, &shells, &projections);
        let diagnostics = result.diagnostics();

        assert_eq!(diagnostics.len(), 4);
        assert_eq!(
            diagnostics
                .iter()
                .map(SymbolDiagnostic::class)
                .collect::<Vec<_>>(),
            vec![
                SymbolDiagnosticClass::ContextOnlyShell,
                SymbolDiagnosticClass::DuplicateDeclaration,
                SymbolDiagnosticClass::DuplicateDeclaration,
                SymbolDiagnosticClass::IllegalOverloadGroup,
            ]
        );
        assert_eq!(
            diagnostics
                .iter()
                .map(SymbolDiagnostic::spelling)
                .collect::<Vec<_>>(),
            vec!["Context", "ADup", "BDup", "_ ? _"]
        );
        assert_eq!(
            diagnostics
                .iter()
                .map(|diagnostic| diagnostic.id().index())
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert_eq!(
            diagnostics
                .iter()
                .map(|diagnostic| diagnostic.range().start)
                .collect::<Vec<_>>(),
            vec![0, 0, 0, 70]
        );
    }

    #[test]
    fn symbol_identity_includes_namespace_notation_arity_and_explicit_slot() {
        let source_id = source_id();
        let shells = shells_for(
            source_id,
            vec![
                test_item(0, SurfaceNodeKind::FunctorDefinition),
                test_item(10, SurfaceNodeKind::FunctorDefinition),
            ],
        );
        let projections = vec![
            projection(
                shells.declarations()[0].id(),
                NamespacePath::new("left"),
                "Op",
                SymbolKind::Functor,
                DefinitionKind::Functor,
            )
            .with_notation_spelling("_ + _")
            .with_arity(2)
            .with_identity_slot("member:0"),
            projection(
                shells.declarations()[1].id(),
                NamespacePath::new("right"),
                "Op",
                SymbolKind::Functor,
                DefinitionKind::Functor,
            )
            .with_notation_spelling("_ * _")
            .with_arity(2)
            .with_identity_slot("member:1"),
        ];

        let result = collect(source_id, &shells, &projections);
        let locals = result
            .env()
            .symbols()
            .iter()
            .map(|entry| entry.symbol().local().as_str())
            .collect::<Vec<_>>();

        assert_eq!(locals.len(), 2);
        assert_ne!(locals[0], locals[1]);
        assert!(locals.iter().any(|local| local.contains("namespace=left")));
        assert!(locals.iter().any(|local| local.contains("namespace=right")));
        assert!(locals.iter().any(|local| local.contains("notation=_ + _")));
        assert!(locals.iter().any(|local| local.contains("notation=_ * _")));
        assert!(locals.iter().all(|local| local.contains("arity=2")));
        assert!(locals.iter().any(|local| local.contains("slot=member\\c0")));
        assert!(locals.iter().any(|local| local.contains("slot=member\\c1")));
    }

    #[test]
    fn registration_projection_populates_symbol_definition_and_registration_indexes() {
        let source_id = source_id();
        let shells = shells_for(
            source_id,
            vec![test_item(0, SurfaceNodeKind::ConditionalRegistration)],
        );
        let projection = SymbolDeclarationProjection::new(
            shells.declarations()[0].id(),
            NamespacePath::new("main"),
            "Reg",
            SymbolKind::Registration,
        )
        .with_registration_kind(RegistrationKind::Cluster);

        let result = collect(source_id, &shells, &[projection]);
        let env = result.env();

        assert_eq!(env.symbols().len(), 1);
        assert_eq!(env.definitions().len(), 1);
        assert_eq!(env.registrations().len(), 1);
        let symbol = env.symbols().iter().next().unwrap();
        let definition = env.definitions().iter().next().unwrap();
        let registration = env.registrations().iter().next().unwrap();
        assert_eq!(definition.symbol(), symbol.symbol());
        assert_eq!(definition.kind(), DefinitionKind::Registration);
        assert_eq!(registration.symbol(), Some(symbol.symbol()));
        assert_eq!(registration.kind(), RegistrationKind::Cluster);
        let effects = env.contributions().iter().next().unwrap().effects();
        assert_eq!(effects.symbols(), &[symbol.symbol().clone()]);
        assert_eq!(effects.definitions(), &[definition.id()]);
        assert_eq!(effects.registrations().len(), 1);
        assert_eq!(effects.registrations(), &[registration.id()]);
        assert_eq!(symbol.contribution(), definition.contribution());
        assert_eq!(symbol.contribution(), registration.contribution());
    }

    #[test]
    fn recovered_shells_stay_local_and_malformed_without_panicking() {
        let source_id = source_id();
        let mut builder = SurfaceAstBuilder::new(source_id);
        let recovery = builder.add_recovery(
            SyntaxRecoveryKind::SkippedToken,
            range(source_id, 1, 2),
            Vec::new(),
        );
        let predicate = node(
            &mut builder,
            SurfaceNodeKind::PredicateDefinition,
            source_id,
            0,
            5,
            vec![recovery],
        );
        let root = finish_module(&mut builder, source_id, vec![predicate]);
        let ast = builder.finish(Some(root), None);
        let module = module_id();
        let shells = DeclarationShellCollector::new(&ast, &module).collect();
        let projection = projection(
            shells.declarations()[0].id(),
            NamespacePath::new("main"),
            "Recovered",
            SymbolKind::Predicate,
            DefinitionKind::Predicate,
        );

        let result = collect(source_id, &shells, &[projection]);
        let symbol = result
            .env()
            .symbols()
            .visible_candidates(&NamespacePath::new("main"), "Recovered")[0];

        assert_eq!(symbol.export_status(), ExportStatus::LocalOnly);
        assert!(matches!(
            symbol.signature(),
            Some(SignatureShell::Malformed { class }) if class == "recovered-shell"
        ));
        assert_eq!(
            result.env().definitions().iter().next().unwrap().conflict(),
            Some(&DeclarationConflictClass::RecoveredShell)
        );
    }

    #[test]
    fn context_parent_visibility_and_recovery_propagate_to_child_symbols() {
        let source_id = source_id();
        let mut builder = SurfaceAstBuilder::new(source_id);
        let private_marker = visibility_marker(&mut builder, source_id, 0, "private");
        let recovery = builder.add_recovery(
            SyntaxRecoveryKind::SkippedToken,
            range(source_id, 12, 13),
            Vec::new(),
        );
        let predicate = node(
            &mut builder,
            SurfaceNodeKind::PredicateDefinition,
            source_id,
            20,
            25,
            Vec::new(),
        );
        let definition_block = node(
            &mut builder,
            SurfaceNodeKind::DefinitionBlockItem,
            source_id,
            10,
            30,
            vec![recovery, predicate],
        );
        let visible_block = node(
            &mut builder,
            SurfaceNodeKind::VisibleItem,
            source_id,
            0,
            30,
            vec![private_marker, definition_block],
        );
        let root = finish_module(&mut builder, source_id, vec![visible_block]);
        let ast = builder.finish(Some(root), None);
        let module = module_id();
        let shells = DeclarationShellCollector::new(&ast, &module).collect();
        let child = shells
            .declarations()
            .iter()
            .find(|shell| shell.kind() == DeclarationShellKind::PredicateDefinition)
            .unwrap();
        let projection = projection(
            child.id(),
            NamespacePath::new("main"),
            "InheritedContext",
            SymbolKind::Predicate,
            DefinitionKind::Predicate,
        );

        let result = collect(source_id, &shells, &[projection]);
        let symbol = result
            .env()
            .symbols()
            .visible_candidates(&NamespacePath::new("main"), "InheritedContext")[0];

        assert_eq!(symbol.visibility(), Visibility::Private);
        assert_eq!(symbol.export_status(), ExportStatus::LocalOnly);
        assert!(symbol.origin().is_recovered());
        assert!(matches!(
            symbol.signature(),
            Some(SignatureShell::Malformed { class }) if class == "recovered-shell"
        ));
        assert_eq!(
            result.env().definitions().iter().next().unwrap().conflict(),
            Some(&DeclarationConflictClass::RecoveredShell)
        );
    }

    #[test]
    fn context_only_shells_do_not_fabricate_symbol_identities() {
        let source_id = source_id();
        let shells = shells_for(
            source_id,
            vec![
                test_item(0, SurfaceNodeKind::DefinitionBlockItem),
                visible_test_item(10, "public", SurfaceNodeKind::FunctorDefinition),
            ],
        );
        let projections = vec![
            projection(
                shells.declarations()[0].id(),
                NamespacePath::new("main"),
                "Block",
                SymbolKind::Structure,
                DefinitionKind::Structure,
            ),
            projection(
                shells.declarations()[1].id(),
                NamespacePath::new("main"),
                "VisibleFunctor",
                SymbolKind::Functor,
                DefinitionKind::Functor,
            ),
        ];

        let result = collect(source_id, &shells, &projections);

        assert_eq!(result.env().symbols().len(), 1);
        assert_eq!(result.diagnostics().len(), 1);
        assert_eq!(
            result.diagnostics()[0].class(),
            SymbolDiagnosticClass::ContextOnlyShell
        );
        assert_eq!(
            result
                .env()
                .symbols()
                .iter()
                .next()
                .unwrap()
                .export_status(),
            ExportStatus::Exported
        );
    }

    #[derive(Debug, Clone)]
    struct DuplicateCase {
        item_kind: SurfaceNodeKind,
        spelling: &'static str,
        symbol_kind: SymbolKind,
        definition_kind: Option<DefinitionKind>,
        registration_kind: Option<RegistrationKind>,
    }

    impl DuplicateCase {
        fn projection(
            &self,
            shell: DeclarationShellId,
            namespace: NamespacePath,
        ) -> SymbolDeclarationProjection {
            let projection =
                SymbolDeclarationProjection::new(shell, namespace, self.spelling, self.symbol_kind);
            if let Some(registration_kind) = self.registration_kind {
                projection.with_registration_kind(registration_kind)
            } else {
                projection.with_definition_kind(
                    self.definition_kind
                        .expect("non-registration duplicate case has a definition kind"),
                )
            }
        }
    }

    fn duplicate_cases() -> Vec<DuplicateCase> {
        vec![
            duplicate_case(
                SurfaceNodeKind::PredicateDefinition,
                "DupPredicate",
                SymbolKind::Predicate,
                DefinitionKind::Predicate,
            ),
            duplicate_case(
                SurfaceNodeKind::FunctorDefinition,
                "DupFunctor",
                SymbolKind::Functor,
                DefinitionKind::Functor,
            ),
            duplicate_case(
                SurfaceNodeKind::ModeDefinition,
                "DupMode",
                SymbolKind::Mode,
                DefinitionKind::Mode,
            ),
            duplicate_case(
                SurfaceNodeKind::AttributeDefinition,
                "DupAttribute",
                SymbolKind::Attribute,
                DefinitionKind::Attribute,
            ),
            duplicate_case(
                SurfaceNodeKind::StructureDefinition,
                "DupStructure",
                SymbolKind::Structure,
                DefinitionKind::Structure,
            ),
            duplicate_case(
                SurfaceNodeKind::TheoremItem,
                "DupTheorem",
                SymbolKind::Theorem,
                DefinitionKind::Theorem,
            ),
            duplicate_case(
                SurfaceNodeKind::LemmaItem,
                "DupLemma",
                SymbolKind::Lemma,
                DefinitionKind::Lemma,
            ),
            duplicate_case(
                SurfaceNodeKind::AlgorithmDefinition,
                "DupAlgorithm",
                SymbolKind::Algorithm,
                DefinitionKind::Algorithm,
            ),
            duplicate_case(
                SurfaceNodeKind::NotationAlias,
                "DupSynonym",
                SymbolKind::Synonym,
                DefinitionKind::Synonym,
            ),
            duplicate_case(
                SurfaceNodeKind::NotationAlias,
                "DupAntonym",
                SymbolKind::Antonym,
                DefinitionKind::Antonym,
            ),
            duplicate_case(
                SurfaceNodeKind::PredicateRedefinition,
                "DupRedefinition",
                SymbolKind::Redefinition,
                DefinitionKind::Redefinition,
            ),
            duplicate_case(
                SurfaceNodeKind::StructureField,
                "DupSelector",
                SymbolKind::Selector,
                DefinitionKind::Selector,
            ),
            DuplicateCase {
                item_kind: SurfaceNodeKind::ConditionalRegistration,
                spelling: "DupRegistration",
                symbol_kind: SymbolKind::Registration,
                definition_kind: None,
                registration_kind: Some(RegistrationKind::Cluster),
            },
        ]
    }

    const fn duplicate_case(
        item_kind: SurfaceNodeKind,
        spelling: &'static str,
        symbol_kind: SymbolKind,
        definition_kind: DefinitionKind,
    ) -> DuplicateCase {
        DuplicateCase {
            item_kind,
            spelling,
            symbol_kind,
            definition_kind: Some(definition_kind),
            registration_kind: None,
        }
    }

    fn duplicate_case_items(cases: &[DuplicateCase]) -> Vec<TestItem> {
        cases
            .iter()
            .enumerate()
            .flat_map(|(index, case)| {
                let start = index * 20;
                [
                    test_item(start, case.item_kind.clone()),
                    test_item(start + 10, case.item_kind.clone()),
                ]
            })
            .collect()
    }

    fn collect(
        source_id: SourceId,
        shells: &DeclarationShellSet,
        projections: &[SymbolDeclarationProjection],
    ) -> SymbolCollectionResult {
        let module = module_id();
        SymbolCollector::new(source_id, &module, shells, projections).collect()
    }

    fn projection(
        shell: DeclarationShellId,
        namespace: NamespacePath,
        spelling: &str,
        symbol_kind: SymbolKind,
        definition_kind: DefinitionKind,
    ) -> SymbolDeclarationProjection {
        SymbolDeclarationProjection::new(shell, namespace, spelling, symbol_kind)
            .with_definition_kind(definition_kind)
    }

    fn shells_for(source_id: SourceId, items: Vec<TestItem>) -> DeclarationShellSet {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let item_nodes = items
            .into_iter()
            .map(|item| item.build(&mut builder, source_id))
            .collect();
        let root = finish_module(&mut builder, source_id, item_nodes);
        let ast = builder.finish(Some(root), None);
        DeclarationShellCollector::new(&ast, &module_id()).collect()
    }

    enum TestItem {
        Node {
            start: usize,
            kind: SurfaceNodeKind,
        },
        Visible {
            start: usize,
            spelling: &'static str,
            target_kind: SurfaceNodeKind,
        },
    }

    impl TestItem {
        fn build(
            self,
            builder: &mut SurfaceAstBuilder,
            source_id: SourceId,
        ) -> SurfaceBuilderNodeId {
            match self {
                Self::Node { start, kind } => {
                    node(builder, kind, source_id, start, start + 5, Vec::new())
                }
                Self::Visible {
                    start,
                    spelling,
                    target_kind,
                } => {
                    let marker = visibility_marker(builder, source_id, start, spelling);
                    let target_start = start + spelling.len() + 1;
                    let target = node(
                        builder,
                        target_kind,
                        source_id,
                        target_start,
                        target_start + 5,
                        Vec::new(),
                    );
                    node(
                        builder,
                        SurfaceNodeKind::VisibleItem,
                        source_id,
                        start,
                        target_start + 5,
                        vec![marker, target],
                    )
                }
            }
        }
    }

    const fn test_item(start: usize, kind: SurfaceNodeKind) -> TestItem {
        TestItem::Node { start, kind }
    }

    const fn visible_test_item(
        start: usize,
        spelling: &'static str,
        target_kind: SurfaceNodeKind,
    ) -> TestItem {
        TestItem::Visible {
            start,
            spelling,
            target_kind,
        }
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
            "05".repeat(Hash::BYTE_LEN)
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
}
