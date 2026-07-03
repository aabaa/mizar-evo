//! Symbol/signature projection and collection.
//!
//! This module implements the R-020 declaration-symbol skeleton over explicit
//! declaration projections. R-021 adds parser-backed per-kind spelling and
//! opaque signature extraction for represented declaration shells.

use crate::declarations::{
    DeclarationShell, DeclarationShellId, DeclarationShellKind, DeclarationShellSet,
    DeclarationShellVisibilityState,
};
use crate::env::{
    ContributionKind, DeclarationConflictClass, DefinitionKind, DefinitionShell,
    DiagnosticAnchorId, ExportStatus, LexicalSummaryKind, NamespacePath, OverloadKey,
    RegistrationKind, SignatureShell, SourceContributionId, SymbolEntry, SymbolEnv,
    SymbolEnvIndexes, SymbolKind, Visibility,
};
use crate::recovery::suppress_dependent_diagnostic_for_recovered_shell;
use crate::resolved_ast::{FullyQualifiedName, LocalSymbolId, ModuleId, SemanticOrigin, SymbolId};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use mizar_syntax::{SurfaceAst, SurfaceNodeKind, SurfaceNodeView, SurfaceTokenKind};
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
    lexical_summary_kind: Option<LexicalSummaryKind>,
    functor_signature_key: Option<FunctorSignatureKey>,
    signature: Option<SignatureShell>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct FunctorSignatureKey {
    argument_context: String,
    pattern: String,
    arity: Option<u32>,
    return_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct FunctorArgumentSignatureKey {
    namespace: NamespacePath,
    argument_context: String,
    pattern: String,
    arity: Option<u32>,
}

impl FunctorSignatureKey {
    fn argument_key(&self, namespace: NamespacePath) -> FunctorArgumentSignatureKey {
        FunctorArgumentSignatureKey {
            namespace,
            argument_context: self.argument_context.clone(),
            pattern: self.pattern.clone(),
            arity: self.arity,
        }
    }
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
            lexical_summary_kind: None,
            functor_signature_key: None,
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

    fn with_parser_backed_lexical_summary_kind(mut self, kind: LexicalSummaryKind) -> Self {
        self.lexical_summary_kind = Some(kind);
        self
    }

    fn with_functor_signature_key(mut self, key: FunctorSignatureKey) -> Self {
        self.functor_signature_key = Some(key);
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

    /// Returns notation spelling when present.
    #[must_use]
    pub fn notation_spelling(&self) -> Option<&str> {
        self.notation_spelling.as_deref()
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

    /// Returns syntactic arity when available.
    #[must_use]
    pub const fn arity(&self) -> Option<u32> {
        self.arity
    }

    /// Returns the overload grouping policy.
    #[must_use]
    pub const fn overload_policy(&self) -> SymbolOverloadPolicy {
        self.overload_policy
    }

    /// Returns the opaque signature shell when populated by extraction.
    #[must_use]
    pub const fn signature(&self) -> Option<&SignatureShell> {
        self.signature.as_ref()
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
    /// Same argument-signature functor declarations with incompatible return
    /// signatures.
    SameSignatureReturnConflict,
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

/// Parser-backed per-kind signature projection extractor.
#[derive(Debug, Clone)]
pub struct SignatureProjectionExtractor<'a> {
    ast: &'a SurfaceAst,
    shells: &'a DeclarationShellSet,
    namespace: NamespacePath,
}

impl<'a> SignatureProjectionExtractor<'a> {
    /// Creates a parser-backed projection extractor.
    #[must_use]
    pub fn new(
        ast: &'a SurfaceAst,
        shells: &'a DeclarationShellSet,
        namespace: NamespacePath,
    ) -> Self {
        Self {
            ast,
            shells,
            namespace,
        }
    }

    /// Extracts concrete parser-backed projections for represented shell kinds.
    #[must_use]
    pub fn extract(&self) -> Vec<SymbolDeclarationProjection> {
        let mut projections = self
            .shells
            .declarations()
            .iter()
            .filter_map(|shell| self.extract_shell(shell))
            .collect::<Vec<_>>();
        projections.sort_by(|left, right| {
            left.shell()
                .cmp(&right.shell())
                .then_with(|| left.primary_spelling().cmp(right.primary_spelling()))
                .then_with(|| left.symbol_kind().cmp(&right.symbol_kind()))
        });
        projections
    }

    fn extract_shell(&self, shell: &DeclarationShell) -> Option<SymbolDeclarationProjection> {
        let view = self.ast.node_view(shell.node_id())?;
        match shell.kind() {
            DeclarationShellKind::Theorem => {
                self.label_projection(shell, view, SymbolKind::Theorem, DefinitionKind::Theorem)
            }
            DeclarationShellKind::Lemma => {
                self.label_projection(shell, view, SymbolKind::Lemma, DefinitionKind::Lemma)
            }
            DeclarationShellKind::AttributeDefinition => self.pattern_projection(
                shell,
                view,
                is_attribute_pattern,
                SymbolKind::Attribute,
                DefinitionKind::Attribute,
                SymbolOverloadPolicy::Overloadable,
            ),
            DeclarationShellKind::PredicateDefinition => self.pattern_projection(
                shell,
                view,
                is_predicate_pattern,
                SymbolKind::Predicate,
                DefinitionKind::Predicate,
                SymbolOverloadPolicy::Overloadable,
            ),
            DeclarationShellKind::FunctorDefinition => self.pattern_projection(
                shell,
                view,
                is_functor_pattern,
                SymbolKind::Functor,
                DefinitionKind::Functor,
                SymbolOverloadPolicy::Overloadable,
            ),
            DeclarationShellKind::ModeDefinition => self.pattern_projection(
                shell,
                view,
                is_mode_pattern,
                SymbolKind::Mode,
                DefinitionKind::Mode,
                SymbolOverloadPolicy::Overloadable,
            ),
            DeclarationShellKind::StructureDefinition => self.pattern_projection(
                shell,
                view,
                is_structure_pattern,
                SymbolKind::Structure,
                DefinitionKind::Structure,
                SymbolOverloadPolicy::NonOverloadable,
            ),
            DeclarationShellKind::AlgorithmDefinition => self.algorithm_projection(shell, view),
            DeclarationShellKind::AttributeRedefinition
            | DeclarationShellKind::PredicateRedefinition
            | DeclarationShellKind::FunctorRedefinition
            | DeclarationShellKind::FieldRedefinition
            | DeclarationShellKind::PropertyRedefinition => {
                self.redefinition_projection(shell, view)
            }
            DeclarationShellKind::NotationAlias => self.notation_alias_projection(shell, view),
            DeclarationShellKind::PropertyClause => self.property_projection(shell, view),
            DeclarationShellKind::StructureField | DeclarationShellKind::StructureProperty => {
                self.selector_projection(shell, view)
            }
            DeclarationShellKind::ExistentialRegistration
            | DeclarationShellKind::ConditionalRegistration
            | DeclarationShellKind::FunctorialRegistration
            | DeclarationShellKind::ReductionRegistration => {
                self.registration_projection(shell, view)
            }
            DeclarationShellKind::Placeholder
            | DeclarationShellKind::Reserve
            | DeclarationShellKind::DefinitionBlock
            | DeclarationShellKind::RegistrationBlock
            | DeclarationShellKind::ClaimBlock
            | DeclarationShellKind::InheritanceDefinition
            | DeclarationShellKind::VisibilityWrapper => None,
        }
    }

    fn label_projection(
        &self,
        shell: &DeclarationShell,
        view: SurfaceNodeView<'_>,
        symbol_kind: SymbolKind,
        definition_kind: DefinitionKind,
    ) -> Option<SymbolDeclarationProjection> {
        let spelling = label_before_colon(view)?;
        Some(
            SymbolDeclarationProjection::new(
                shell.id(),
                self.namespace.clone(),
                spelling,
                symbol_kind,
            )
            .with_definition_kind(definition_kind)
            .with_signature(parser_signature(
                view,
                symbol_kind,
                definition_kind,
                None,
                None,
            )),
        )
    }

    fn pattern_projection(
        &self,
        shell: &DeclarationShell,
        view: SurfaceNodeView<'_>,
        is_pattern: fn(&SurfaceNodeKind) -> bool,
        symbol_kind: SymbolKind,
        definition_kind: DefinitionKind,
        overload_policy: SymbolOverloadPolicy,
    ) -> Option<SymbolDeclarationProjection> {
        let pattern = first_child_matching(view, is_pattern).unwrap_or(view);
        let spelling = normalized_token_shape(pattern);
        if spelling.is_empty() {
            return None;
        }
        let notation = normalized_token_shape(pattern);
        let functor_signature_key = (symbol_kind == SymbolKind::Functor
            && definition_kind == DefinitionKind::Functor)
            .then(|| self.functor_signature_key(shell, view, &notation, None))
            .flatten();
        let mut projection = SymbolDeclarationProjection::new(
            shell.id(),
            self.namespace.clone(),
            spelling,
            symbol_kind,
        )
        .with_definition_kind(definition_kind)
        .with_overload_policy(overload_policy)
        .with_signature(parser_signature(
            view,
            symbol_kind,
            definition_kind,
            Some(&notation),
            None,
        ));
        if let Some(key) = functor_signature_key {
            projection = projection.with_functor_signature_key(key);
        }
        if !notation.is_empty() {
            projection = projection.with_notation_spelling(notation);
        }
        if lexer_visible_pattern(pattern) {
            projection =
                projection.with_parser_backed_lexical_summary_kind(LexicalSummaryKind::Notation);
        }
        Some(projection)
    }

    fn functor_signature_key(
        &self,
        shell: &DeclarationShell,
        view: SurfaceNodeView<'_>,
        pattern: &str,
        arity: Option<u32>,
    ) -> Option<FunctorSignatureKey> {
        let return_type = first_child_matching(view, is_type_expression)?;
        Some(FunctorSignatureKey {
            argument_context: self.definition_context_key(shell),
            pattern: pattern.to_owned(),
            arity,
            return_type: normalized_token_shape(return_type),
        })
    }

    fn definition_context_key(&self, shell: &DeclarationShell) -> String {
        let Some(block) = self.definition_context_block(shell) else {
            return "_".to_owned();
        };
        let Some(block_view) = self.ast.node_view(block.node_id()) else {
            return "_".to_owned();
        };
        let excluded = self
            .shells
            .declarations()
            .iter()
            .filter(|candidate| candidate.id() != block.id())
            .filter(|candidate| source_range_contains(block.range(), candidate.range()))
            .filter(|candidate| excludes_from_definition_context(candidate.kind()))
            .map(DeclarationShell::range)
            .collect::<Vec<_>>();
        let context = normalized_token_shape_excluding_ranges(block_view, &excluded);
        if context.is_empty() {
            "_".to_owned()
        } else {
            context
        }
    }

    fn definition_context_block(&self, shell: &DeclarationShell) -> Option<&DeclarationShell> {
        let mut current = shell.parent();
        while let Some(id) = current {
            let context = self.shells.declaration(id)?;
            if context.kind() == DeclarationShellKind::DefinitionBlock {
                return Some(context);
            }
            current = context.parent();
        }
        None
    }

    fn algorithm_projection(
        &self,
        shell: &DeclarationShell,
        view: SurfaceNodeView<'_>,
    ) -> Option<SymbolDeclarationProjection> {
        let spelling = identifier_after_keyword(view, "algorithm")?;
        Some(
            SymbolDeclarationProjection::new(
                shell.id(),
                self.namespace.clone(),
                spelling,
                SymbolKind::Algorithm,
            )
            .with_definition_kind(DefinitionKind::Algorithm)
            .with_signature(parser_signature(
                view,
                SymbolKind::Algorithm,
                DefinitionKind::Algorithm,
                None,
                None,
            )),
        )
    }

    fn redefinition_projection(
        &self,
        shell: &DeclarationShell,
        view: SurfaceNodeView<'_>,
    ) -> Option<SymbolDeclarationProjection> {
        let spelling = preferred_symbol_spelling(view)?;
        let notation = normalized_token_shape(view);
        let mut projection = SymbolDeclarationProjection::new(
            shell.id(),
            self.namespace.clone(),
            spelling,
            SymbolKind::Redefinition,
        )
        .with_definition_kind(DefinitionKind::Redefinition)
        .with_identity_slot(format!(
            "redefinition:{}",
            declaration_shell_kind_key(shell.kind())
        ))
        .with_signature(parser_signature(
            view,
            SymbolKind::Redefinition,
            DefinitionKind::Redefinition,
            Some(&notation),
            None,
        ));
        if !notation.is_empty() {
            projection = projection.with_notation_spelling(notation);
        }
        Some(projection)
    }

    fn notation_alias_projection(
        &self,
        shell: &DeclarationShell,
        view: SurfaceNodeView<'_>,
    ) -> Option<SymbolDeclarationProjection> {
        let alias_kind = if first_reserved_word(view).as_deref() == Some("antonym") {
            (SymbolKind::Antonym, DefinitionKind::Antonym)
        } else {
            (SymbolKind::Synonym, DefinitionKind::Synonym)
        };
        let pattern = first_child_matching(view, is_notation_pattern).unwrap_or(view);
        let spelling = preferred_symbol_spelling(pattern)?;
        let notation = normalized_token_shape(pattern);
        let mut projection = SymbolDeclarationProjection::new(
            shell.id(),
            self.namespace.clone(),
            spelling,
            alias_kind.0,
        )
        .with_definition_kind(alias_kind.1)
        .with_identity_slot(format!("relation:{}", symbol_kind_key(alias_kind.0)))
        .with_signature(parser_signature(
            view,
            alias_kind.0,
            alias_kind.1,
            Some(&notation),
            None,
        ));
        if !notation.is_empty() {
            projection = projection.with_notation_spelling(notation);
        }
        if lexer_visible_pattern(pattern) {
            projection =
                projection.with_parser_backed_lexical_summary_kind(LexicalSummaryKind::Notation);
        }
        Some(projection)
    }

    fn property_projection(
        &self,
        shell: &DeclarationShell,
        view: SurfaceNodeView<'_>,
    ) -> Option<SymbolDeclarationProjection> {
        let spelling = first_reserved_word(view).or_else(|| preferred_symbol_spelling(view))?;
        Some(
            SymbolDeclarationProjection::new(
                shell.id(),
                self.namespace.clone(),
                spelling,
                SymbolKind::Attribute,
            )
            .with_definition_kind(DefinitionKind::Attribute)
            .with_identity_slot("property-clause")
            .with_signature(parser_signature(
                view,
                SymbolKind::Attribute,
                DefinitionKind::Attribute,
                None,
                None,
            )),
        )
    }

    fn selector_projection(
        &self,
        shell: &DeclarationShell,
        view: SurfaceNodeView<'_>,
    ) -> Option<SymbolDeclarationProjection> {
        let spelling = preferred_symbol_spelling(view)?;
        Some(
            SymbolDeclarationProjection::new(
                shell.id(),
                self.namespace.clone(),
                spelling,
                SymbolKind::Selector,
            )
            .with_definition_kind(DefinitionKind::Selector)
            .with_identity_slot(format!(
                "selector:{}",
                declaration_shell_kind_key(shell.kind())
            ))
            .with_signature(parser_signature(
                view,
                SymbolKind::Selector,
                DefinitionKind::Selector,
                None,
                None,
            )),
        )
    }

    fn registration_projection(
        &self,
        shell: &DeclarationShell,
        view: SurfaceNodeView<'_>,
    ) -> Option<SymbolDeclarationProjection> {
        let spelling = label_before_colon(view)?;
        let registration_kind = match shell.kind() {
            DeclarationShellKind::ReductionRegistration => RegistrationKind::Reduction,
            DeclarationShellKind::ExistentialRegistration
            | DeclarationShellKind::ConditionalRegistration
            | DeclarationShellKind::FunctorialRegistration => RegistrationKind::Cluster,
            _ => return None,
        };
        Some(
            SymbolDeclarationProjection::new(
                shell.id(),
                self.namespace.clone(),
                spelling,
                SymbolKind::Registration,
            )
            .with_registration_kind(registration_kind)
            .with_signature(parser_signature(
                view,
                SymbolKind::Registration,
                DefinitionKind::Registration,
                None,
                None,
            )),
        )
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
            let context_recovered = shell_context(self.shells, shell).recovered;
            if !symbol_bearing_shell(shell.kind()) {
                if !suppress_dependent_diagnostic_for_recovered_shell(context_recovered) {
                    diagnostic_drafts.push(DiagnosticDraft::context_only(shell, projection));
                }
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
        if let Some((summary_kind, spelling)) = lexical_summary_projection(item) {
            let summary = indexes.lexical_summaries.insert(
                item.symbol.clone(),
                item.projection.namespace().clone(),
                spelling,
                summary_kind,
                item.projection.arity,
                item.contribution,
            );
            indexes
                .contributions
                .add_lexical_summary(item.contribution, summary);
        }

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
        let signature = if context.recovered {
            default_signature(shell, projection, &context)
        } else {
            projection
                .signature
                .clone()
                .unwrap_or_else(|| default_signature(shell, projection, &context))
        };
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

    fn illegal_overload(
        candidates: &[&CollectedProjection<'_>],
        class: SymbolDiagnosticClass,
    ) -> Self {
        let first = candidates[0];
        Self {
            class,
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
    conflicts.extend(classify_same_signature_return_conflicts(
        collected,
        diagnostic_drafts,
    ));
    let mut groups: BTreeMap<ConflictKey, Vec<&CollectedProjection<'_>>> = BTreeMap::new();
    for item in collected {
        if suppress_dependent_diagnostic_for_recovered_shell(item.recovered) {
            continue;
        }
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

fn classify_same_signature_return_conflicts(
    collected: &[CollectedProjection<'_>],
    diagnostic_drafts: &mut Vec<DiagnosticDraft>,
) -> BTreeMap<SymbolId, DeclarationConflictClass> {
    let mut groups: BTreeMap<FunctorArgumentSignatureKey, Vec<&CollectedProjection<'_>>> =
        BTreeMap::new();
    for item in collected {
        if suppress_dependent_diagnostic_for_recovered_shell(item.recovered) {
            continue;
        }
        if item.projection.overload_policy() != SymbolOverloadPolicy::Overloadable {
            continue;
        }
        let Some(key) = &item.projection.functor_signature_key else {
            continue;
        };
        groups
            .entry(key.argument_key(item.projection.namespace().clone()))
            .or_default()
            .push(item);
    }

    let mut conflicts = BTreeMap::new();
    for mut candidates in groups.into_values() {
        candidates.sort_by(collected_projection_cmp);
        if candidates.len() < 2 {
            continue;
        }
        let mut return_keys = candidates
            .iter()
            .filter_map(|candidate| candidate.projection.functor_signature_key.as_ref())
            .map(|key| key.return_type.as_str())
            .collect::<Vec<_>>();
        return_keys.sort_unstable();
        return_keys.dedup();
        if return_keys.len() < 2 {
            continue;
        }

        diagnostic_drafts.push(DiagnosticDraft::illegal_overload(
            &candidates,
            SymbolDiagnosticClass::SameSignatureReturnConflict,
        ));
        for candidate in candidates {
            conflicts.insert(
                candidate.symbol.clone(),
                DeclarationConflictClass::SameSignatureReturnConflict,
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
        if suppress_dependent_diagnostic_for_recovered_shell(item.recovered) {
            continue;
        }
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
        diagnostic_drafts.push(DiagnosticDraft::illegal_overload(
            &candidates,
            SymbolDiagnosticClass::IllegalOverloadGroup,
        ));
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
            .then_with(|| left.lexical_summary_kind.cmp(&right.lexical_summary_kind))
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

fn parser_signature(
    view: SurfaceNodeView<'_>,
    symbol_kind: SymbolKind,
    definition_kind: DefinitionKind,
    notation: Option<&str>,
    arity: Option<u32>,
) -> SignatureShell {
    SignatureShell::Opaque {
        schema: "parser-signature-v1".to_owned(),
        payload: format!(
            "node={};symbol={};definition={};primary_tokens={};notation={};arity={};roles={}",
            surface_node_kind_key(view.kind()),
            symbol_kind_key(symbol_kind),
            definition_kind_key(definition_kind),
            normalized_token_shape(view),
            notation.unwrap_or("_"),
            arity
                .map(|arity| arity.to_string())
                .unwrap_or_else(|| "_".to_owned()),
            direct_structural_roles(view)
        ),
    }
}

fn lexical_summary_projection(
    item: &CollectedProjection<'_>,
) -> Option<(LexicalSummaryKind, String)> {
    if item.export_status == ExportStatus::LocalOnly
        || suppress_dependent_diagnostic_for_recovered_shell(item.recovered)
    {
        return None;
    }
    let kind = item.projection.lexical_summary_kind?;
    Some((
        kind,
        item.projection
            .notation_spelling()
            .unwrap_or_else(|| item.projection.primary_spelling())
            .to_owned(),
    ))
}

fn first_child_matching<'a>(
    view: SurfaceNodeView<'a>,
    matches_kind: fn(&SurfaceNodeKind) -> bool,
) -> Option<SurfaceNodeView<'a>> {
    view.child_views().find(|child| matches_kind(child.kind()))
}

fn label_before_colon(view: SurfaceNodeView<'_>) -> Option<String> {
    let tokens = flattened_tokens(view);
    tokens.windows(2).find_map(|window| {
        let (kind, text) = window[0];
        let (_, next) = window[1];
        if matches!(
            kind,
            SurfaceTokenKind::Identifier | SurfaceTokenKind::UserSymbol
        ) && next == ":"
        {
            Some(text.to_owned())
        } else {
            None
        }
    })
}

fn identifier_after_keyword(view: SurfaceNodeView<'_>, keyword: &str) -> Option<String> {
    let tokens = flattened_tokens(view);
    tokens.windows(2).find_map(|window| {
        let (_, text) = window[0];
        let (next_kind, next_text) = window[1];
        if text == keyword && next_kind == SurfaceTokenKind::Identifier {
            Some(next_text.to_owned())
        } else {
            None
        }
    })
}

fn preferred_symbol_spelling(view: SurfaceNodeView<'_>) -> Option<String> {
    first_token_text(view, SurfaceTokenKind::UserSymbol)
        .or_else(|| first_token_text(view, SurfaceTokenKind::LexemeRun))
        .or_else(|| first_token_text(view, SurfaceTokenKind::Identifier))
}

fn first_reserved_word(view: SurfaceNodeView<'_>) -> Option<String> {
    first_token_text(view, SurfaceTokenKind::ReservedWord)
}

fn first_token_text(view: SurfaceNodeView<'_>, kind: SurfaceTokenKind) -> Option<String> {
    flattened_tokens(view)
        .into_iter()
        .find_map(|(token_kind, text)| (token_kind == kind).then(|| text.to_owned()))
}

fn normalized_token_shape(view: SurfaceNodeView<'_>) -> String {
    flattened_tokens(view)
        .into_iter()
        .filter(|(kind, _)| {
            matches!(
                *kind,
                SurfaceTokenKind::Identifier
                    | SurfaceTokenKind::ReservedWord
                    | SurfaceTokenKind::ReservedSymbol
                    | SurfaceTokenKind::UserSymbol
                    | SurfaceTokenKind::LexemeRun
                    | SurfaceTokenKind::Numeral
            )
        })
        .map(|(_, text)| text)
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalized_token_shape_excluding_ranges(
    view: SurfaceNodeView<'_>,
    excluded_ranges: &[SourceRange],
) -> String {
    flattened_tokens_excluding_ranges(view, excluded_ranges)
        .into_iter()
        .filter(|(kind, _)| {
            matches!(
                *kind,
                SurfaceTokenKind::Identifier
                    | SurfaceTokenKind::ReservedWord
                    | SurfaceTokenKind::ReservedSymbol
                    | SurfaceTokenKind::UserSymbol
                    | SurfaceTokenKind::LexemeRun
                    | SurfaceTokenKind::Numeral
            )
        })
        .map(|(_, text)| text)
        .collect::<Vec<_>>()
        .join(" ")
}

fn lexer_visible_pattern(view: SurfaceNodeView<'_>) -> bool {
    flattened_tokens(view).into_iter().any(|(kind, _)| {
        matches!(
            kind,
            SurfaceTokenKind::UserSymbol | SurfaceTokenKind::LexemeRun
        )
    })
}

fn flattened_tokens(view: SurfaceNodeView<'_>) -> Vec<(SurfaceTokenKind, &str)> {
    let mut tokens = Vec::new();
    collect_tokens(view, &mut tokens);
    tokens
}

fn flattened_tokens_excluding_ranges<'a>(
    view: SurfaceNodeView<'a>,
    excluded_ranges: &[SourceRange],
) -> Vec<(SurfaceTokenKind, &'a str)> {
    let mut tokens = Vec::new();
    collect_tokens_excluding_ranges(view, excluded_ranges, &mut tokens);
    tokens
}

fn collect_tokens<'a>(view: SurfaceNodeView<'a>, tokens: &mut Vec<(SurfaceTokenKind, &'a str)>) {
    if let Some(token) = view.as_token() {
        tokens.push((token.kind, token.text.as_ref()));
        return;
    }
    for child in view.child_views() {
        collect_tokens(child, tokens);
    }
}

fn collect_tokens_excluding_ranges<'a>(
    view: SurfaceNodeView<'a>,
    excluded_ranges: &[SourceRange],
    tokens: &mut Vec<(SurfaceTokenKind, &'a str)>,
) {
    if excludes_from_definition_context_node(view.kind()) {
        return;
    }
    if excluded_ranges
        .iter()
        .any(|range| source_range_contains(*range, view.range()))
    {
        return;
    }
    if let Some(token) = view.as_token() {
        tokens.push((token.kind, token.text.as_ref()));
        return;
    }
    for child in view.child_views() {
        collect_tokens_excluding_ranges(child, excluded_ranges, tokens);
    }
}

fn source_range_contains(parent: SourceRange, child: SourceRange) -> bool {
    parent.source_id == child.source_id && parent.start <= child.start && child.end <= parent.end
}

fn direct_structural_roles(view: SurfaceNodeView<'_>) -> String {
    view.child_views()
        .filter(|child| child.kind().is_structural())
        .map(|child| surface_node_kind_key(child.kind()))
        .collect::<Vec<_>>()
        .join(",")
}

fn is_attribute_pattern(kind: &SurfaceNodeKind) -> bool {
    matches!(kind, SurfaceNodeKind::AttributePattern)
}

fn is_predicate_pattern(kind: &SurfaceNodeKind) -> bool {
    matches!(kind, SurfaceNodeKind::PredicatePattern)
}

fn is_functor_pattern(kind: &SurfaceNodeKind) -> bool {
    matches!(kind, SurfaceNodeKind::FunctorPattern)
}

fn is_type_expression(kind: &SurfaceNodeKind) -> bool {
    matches!(kind, SurfaceNodeKind::TypeExpression)
}

fn is_mode_pattern(kind: &SurfaceNodeKind) -> bool {
    matches!(kind, SurfaceNodeKind::ModePattern)
}

fn is_structure_pattern(kind: &SurfaceNodeKind) -> bool {
    matches!(kind, SurfaceNodeKind::StructurePattern)
}

fn is_notation_pattern(kind: &SurfaceNodeKind) -> bool {
    matches!(kind, SurfaceNodeKind::NotationPattern)
}

fn excludes_from_definition_context(kind: DeclarationShellKind) -> bool {
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
            | DeclarationShellKind::ExistentialRegistration
            | DeclarationShellKind::ConditionalRegistration
            | DeclarationShellKind::FunctorialRegistration
            | DeclarationShellKind::ReductionRegistration
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
    )
}

fn excludes_from_definition_context_node(kind: &SurfaceNodeKind) -> bool {
    matches!(kind, SurfaceNodeKind::VisibilityMarker)
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

fn surface_node_kind_key(kind: &SurfaceNodeKind) -> String {
    format!("{:?}", kind.syntax_kind())
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
mod tests;
