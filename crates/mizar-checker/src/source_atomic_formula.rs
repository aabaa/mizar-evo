//! Syntax-free transport for source atomic-formula occurrences.

use crate::{
    binding_env::{BindingContextId, BindingEnv},
    source_application::{
        SourceFunctorApplicationHandoff, SourceFunctorApplicationId, SourceFunctorArgumentTarget,
    },
    source_set_term::{
        SourceSetTarget, SourceSetTermHandoff, SourceSetTermId, SourceSetTermKind,
        SourceSetTermRecovery,
    },
    source_structure::{SourceStructureHandoff, SourceStructureTarget, SourceStructureTermId},
    source_term::{SourcePrimaryTermHandoff, SourcePrimaryTermId},
    typed_ast::{NodeRecoveryState, TypedArena, TypedSiteRef},
};
use mizar_resolve::{
    env::{
        ContributionKind, DefinitionKind, ExportStatus, SignatureShell, SourceContributionId,
        SymbolEntry, SymbolEnv, SymbolKind, Visibility,
    },
    resolved_ast::{ModuleId, SymbolId},
};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use std::{
    collections::BTreeSet,
    error::Error,
    fmt::{self, Write as _},
};

macro_rules! dense_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(usize);

        impl $name {
            pub const fn new(index: usize) -> Self {
                Self(index)
            }

            pub const fn index(self) -> usize {
                self.0
            }
        }
    };
}

dense_id!(SourceAtomicFormulaId);
dense_id!(SourceAtomicWrapperId);
dense_id!(SourcePredicateSegmentId);
dense_id!(SourcePredicateHeadId);
dense_id!(SourcePredicateCandidateId);
dense_id!(SourceAssertionTypeSiteId);
dense_id!(SourceAssertionAttributeId);
dense_id!(SourceAtomicEdgeId);
dense_id!(SourceAtomicRequestId);

/// Complete syntax-free input for one source/module atomic-formula transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAtomicFormulaHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub formulas: Vec<SourceAtomicFormulaInput>,
    pub wrappers: Vec<SourceAtomicWrapperInput>,
    pub predicate_segments: Vec<SourcePredicateSegmentInput>,
    pub predicate_heads: Vec<SourcePredicateHeadInput>,
    pub candidates: Vec<SourcePredicateCandidateInput>,
    pub type_sites: Vec<SourceAssertionTypeSiteInput>,
    pub attributes: Vec<SourceAssertionAttributeInput>,
    pub edges: Vec<SourceAtomicEdgeInput>,
    pub requests: Vec<SourceAtomicRequestInput>,
}

/// One atomic source formula occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAtomicFormulaInput {
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourceAtomicFormulaRecovery,
    pub spelling: String,
    pub kind: SourceAtomicFormulaKind,
}

/// One transparent parenthesized formula wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAtomicWrapperInput {
    pub formula: SourceAtomicFormulaId,
    pub ordinal: usize,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceAtomicFormulaRecovery,
    pub spelling: String,
}

/// One source-written segment of an ordinary predicate chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateSegmentInput {
    pub formula: SourceAtomicFormulaId,
    pub ordinal: usize,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceAtomicFormulaRecovery,
    pub spelling: String,
    pub head: SourcePredicateHeadId,
    pub polarity: SourcePredicateSegmentPolarityInput,
    pub left_edge: SourceAtomicEdgeId,
    pub right_edge: SourceAtomicEdgeId,
}

/// One ordinary predicate-segment head.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateHeadInput {
    pub formula: SourceAtomicFormulaId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceAtomicFormulaRecovery,
    pub spelling: String,
    pub left_arity: usize,
    pub right_arity: usize,
}

/// One individually resolver-authenticated predicate reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateCandidateInput {
    pub head: SourcePredicateHeadId,
    pub ordinal: usize,
    pub symbol: SymbolId,
    pub contribution: SourceContributionId,
}

/// One formula-owned bare asserted-type occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAssertionTypeSiteInput {
    pub formula: SourceAtomicFormulaId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub head_site: TypedSiteRef,
    pub head_range: SourceRange,
    pub head_spelling: String,
    pub context: BindingContextId,
    pub recovery: SourceAtomicFormulaRecovery,
    pub head: SourceAssertionTypeHead,
}

/// One formula-owned simple attribute occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAssertionAttributeInput {
    pub formula: SourceAtomicFormulaId,
    pub ordinal: usize,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub target_site: TypedSiteRef,
    pub target_range: SourceRange,
    pub target_spelling: String,
    pub context: BindingContextId,
    pub recovery: SourceAtomicFormulaRecovery,
    pub symbol: SymbolId,
    pub contribution: SourceContributionId,
    pub polarity: SourceAssertionAttributePolarityInput,
}

/// One ordered direct formula-to-term association.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAtomicEdgeInput {
    pub formula: SourceAtomicFormulaId,
    pub ordinal: usize,
    pub role: SourceAtomicEdgeRole,
    pub target: SourceAtomicTermTarget,
}

/// One unresolved atomic-formula dependency request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAtomicRequestInput {
    pub formula: SourceAtomicFormulaId,
    pub ordinal: usize,
    pub kind: SourceAtomicRequestKind,
    pub edge: Option<SourceAtomicEdgeId>,
    pub candidate: Option<SourcePredicateCandidateId>,
    pub type_site: Option<SourceAssertionTypeSiteId>,
    pub attribute: Option<SourceAssertionAttributeId>,
}

/// Atomic formula family admitted by Task 256.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceAtomicFormulaKind {
    PredicateApplication,
    Equality,
    Inequality,
    Membership,
    TypeAssertion,
    AttributeAssertion,
}

/// Recovery state retained at the source atomic-formula boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceAtomicFormulaRecovery {
    Normal,
    Degraded,
}

/// Bare builtin asserted-type head admitted by Task 256.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceAssertionTypeHead {
    BuiltinSet,
    BuiltinObject,
}

/// Polarity of one source-written assertion attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceAssertionAttributePolarityInput {
    Positive,
    Negative {
        non_site: TypedSiteRef,
        non_range: SourceRange,
        non_spelling: String,
        non_recovery: SourceAtomicFormulaRecovery,
    },
}

/// Source-written polarity tokens for one ordinary predicate segment.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePredicateSegmentPolarityInput {
    Positive,
    Negative {
        verb_site: TypedSiteRef,
        verb_range: SourceRange,
        verb_spelling: String,
        verb_recovery: SourceAtomicFormulaRecovery,
        not_site: TypedSiteRef,
        not_range: SourceRange,
        not_spelling: String,
        not_recovery: SourceAtomicFormulaRecovery,
    },
}

/// Source role of one direct atomic-formula term.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceAtomicEdgeRole {
    PredicateLeftArgument,
    PredicateChainBoundary,
    PredicateRightArgument,
    BuiltinLeftOperand,
    BuiltinRightOperand,
    AssertionSubject,
}

/// Cross-family target owned by one direct formula term slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceAtomicTermTarget {
    Primary(SourcePrimaryTermId),
    Application(SourceFunctorApplicationId),
    Structure(SourceStructureTermId),
    SetTerm(SourceSetTermId),
}

/// Unresolved atomic-formula dependency request kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceAtomicRequestKind {
    OperandExpectedType,
    PredicateCandidateSignature,
    TypeAssertionReachability,
    AttributeAdmissibility,
}

/// Immutable validated source atomic-formula handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAtomicFormulaHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    primary_term_fingerprint: String,
    application_fingerprint: Option<String>,
    structure_fingerprint: Option<String>,
    set_term_fingerprint: Option<String>,
    formulas: SourceAtomicFormulaTable,
    wrappers: SourceAtomicWrapperTable,
    predicate_segments: SourcePredicateSegmentTable,
    predicate_heads: SourcePredicateHeadTable,
    candidates: SourcePredicateCandidateTable,
    type_sites: SourceAssertionTypeSiteTable,
    attributes: SourceAssertionAttributeTable,
    edges: SourceAtomicEdgeTable,
    requests: SourceAtomicRequestTable,
}

impl SourceAtomicFormulaHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn primary_term_fingerprint(&self) -> &str {
        &self.primary_term_fingerprint
    }

    pub fn application_fingerprint(&self) -> Option<&str> {
        self.application_fingerprint.as_deref()
    }

    pub fn structure_fingerprint(&self) -> Option<&str> {
        self.structure_fingerprint.as_deref()
    }

    pub fn set_term_fingerprint(&self) -> Option<&str> {
        self.set_term_fingerprint.as_deref()
    }

    pub const fn formulas(&self) -> &SourceAtomicFormulaTable {
        &self.formulas
    }

    pub const fn wrappers(&self) -> &SourceAtomicWrapperTable {
        &self.wrappers
    }

    pub const fn predicate_segments(&self) -> &SourcePredicateSegmentTable {
        &self.predicate_segments
    }

    pub const fn predicate_heads(&self) -> &SourcePredicateHeadTable {
        &self.predicate_heads
    }

    pub const fn candidates(&self) -> &SourcePredicateCandidateTable {
        &self.candidates
    }

    pub const fn type_sites(&self) -> &SourceAssertionTypeSiteTable {
        &self.type_sites
    }

    pub const fn attributes(&self) -> &SourceAssertionAttributeTable {
        &self.attributes
    }

    pub const fn edges(&self) -> &SourceAtomicEdgeTable {
        &self.edges
    }

    pub const fn requests(&self) -> &SourceAtomicRequestTable {
        &self.requests
    }

    /// Stable, source-ordered representation used as a dependency fingerprint.
    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-atomic-formula-debug-v1\n");
        let _ = writeln!(output, "module: {}", self.module_id.path().as_str());
        let _ = writeln!(
            output,
            "primary-term-fingerprint: {:?}",
            self.primary_term_fingerprint
        );
        let _ = writeln!(
            output,
            "application-fingerprint: {:?}",
            self.application_fingerprint
        );
        let _ = writeln!(
            output,
            "structure-fingerprint: {:?}",
            self.structure_fingerprint
        );
        let _ = writeln!(
            output,
            "set-term-fingerprint: {:?}",
            self.set_term_fingerprint
        );
        for (id, row) in self.formulas.iter() {
            let _ = writeln!(
                output,
                "formula#{} ordinal={} kind={} range={}..{} site={} context={} recovery={} spelling={:?}",
                id.index(),
                row.source_ordinal,
                formula_kind_key(row.kind),
                row.source_range.start,
                row.source_range.end,
                row.site.node().index(),
                row.context.index(),
                recovery_key(row.recovery),
                row.spelling,
            );
        }
        for (id, row) in self.wrappers.iter() {
            let _ = writeln!(
                output,
                "wrapper#{} formula={} ordinal={} range={}..{} site={} context={} recovery={} spelling={:?}",
                id.index(),
                row.formula.index(),
                row.ordinal,
                row.source_range.start,
                row.source_range.end,
                row.site.node().index(),
                row.context.index(),
                recovery_key(row.recovery),
                row.spelling,
            );
        }
        for (id, row) in self.predicate_segments.iter() {
            let _ = write!(
                output,
                "predicate-segment#{} formula={} ordinal={} range={}..{} site={} context={} recovery={} spelling={:?} head={} polarity=",
                id.index(),
                row.formula.index(),
                row.ordinal,
                row.source_range.start,
                row.source_range.end,
                row.site.node().index(),
                row.context.index(),
                recovery_key(row.recovery),
                row.spelling,
                row.head.index(),
            );
            write_predicate_segment_polarity(&mut output, &row.polarity);
            let _ = writeln!(
                output,
                " left_edge={} right_edge={}",
                row.left_edge.index(),
                row.right_edge.index(),
            );
        }
        for (id, row) in self.predicate_heads.iter() {
            let _ = writeln!(
                output,
                "predicate-head#{} formula={} range={}..{} site={} context={} recovery={} spelling={:?} left_arity={} right_arity={}",
                id.index(),
                row.formula.index(),
                row.source_range.start,
                row.source_range.end,
                row.site.node().index(),
                row.context.index(),
                recovery_key(row.recovery),
                row.spelling,
                row.left_arity,
                row.right_arity,
            );
        }
        for (id, row) in self.candidates.iter() {
            let _ = writeln!(
                output,
                "candidate#{} head={} ordinal={} symbol={:?} contribution={}",
                id.index(),
                row.head.index(),
                row.ordinal,
                row.symbol,
                row.contribution.index(),
            );
        }
        for (id, row) in self.type_sites.iter() {
            let _ = writeln!(
                output,
                "type-site#{} formula={} range={}..{} site={} spelling={:?} head_range={}..{} head_site={} head_spelling={:?} context={} recovery={} head={}",
                id.index(),
                row.formula.index(),
                row.source_range.start,
                row.source_range.end,
                row.site.node().index(),
                row.spelling,
                row.head_range.start,
                row.head_range.end,
                row.head_site.node().index(),
                row.head_spelling,
                row.context.index(),
                recovery_key(row.recovery),
                type_head_key(row.head),
            );
        }
        for (id, row) in self.attributes.iter() {
            let _ = write!(
                output,
                "attribute#{} formula={} ordinal={} range={}..{} site={} spelling={:?} target_range={}..{} target_site={} target_spelling={:?} context={} recovery={} symbol={:?} contribution={} polarity=",
                id.index(),
                row.formula.index(),
                row.ordinal,
                row.source_range.start,
                row.source_range.end,
                row.site.node().index(),
                row.spelling,
                row.target_range.start,
                row.target_range.end,
                row.target_site.node().index(),
                row.target_spelling,
                row.context.index(),
                recovery_key(row.recovery),
                row.symbol,
                row.contribution.index(),
            );
            write_polarity(&mut output, &row.polarity);
            output.push('\n');
        }
        for (id, row) in self.edges.iter() {
            let _ = write!(
                output,
                "edge#{} formula={} ordinal={} role={} target=",
                id.index(),
                row.formula.index(),
                row.ordinal,
                edge_role_key(row.role),
            );
            write_target(&mut output, row.target);
            output.push('\n');
        }
        for (id, row) in self.requests.iter() {
            let _ = write!(
                output,
                "request#{} formula={} ordinal={} kind={} edge=",
                id.index(),
                row.formula.index(),
                row.ordinal,
                request_kind_key(row.kind),
            );
            write_optional_id(&mut output, row.edge.map(SourceAtomicEdgeId::index));
            output.push_str(" candidate=");
            write_optional_id(
                &mut output,
                row.candidate.map(SourcePredicateCandidateId::index),
            );
            output.push_str(" type_site=");
            write_optional_id(
                &mut output,
                row.type_site.map(SourceAssertionTypeSiteId::index),
            );
            output.push_str(" attribute=");
            write_optional_id(
                &mut output,
                row.attribute.map(SourceAssertionAttributeId::index),
            );
            output.push('\n');
        }
        output
    }

    #[cfg(test)]
    pub(crate) fn set_primary_term_fingerprint_for_test(&mut self, fingerprint: String) {
        self.primary_term_fingerprint = fingerprint;
    }

    #[allow(clippy::too_many_arguments)] // Rationale: installation must reauthenticate every frozen lower-family dependency explicitly.
    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        primary_terms: &SourcePrimaryTermHandoff,
        applications: Option<&SourceFunctorApplicationHandoff>,
        structures: Option<&SourceStructureHandoff>,
        set_terms: Option<&SourceSetTermHandoff>,
        arena: &TypedArena,
    ) -> Result<(), SourceAtomicFormulaError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.primary_term_fingerprint != primary_terms.debug_text()
        {
            return Err(SourceAtomicFormulaError::PrimaryDependencyMismatch);
        }
        validate_dependencies(
            source_id,
            module_id,
            primary_terms,
            applications,
            structures,
            set_terms,
            arena,
        )?;
        let input = self.to_input();
        validate_payload(
            &input,
            None,
            None,
            primary_terms,
            applications,
            structures,
            set_terms,
            arena,
        )?;
        if self.application_fingerprint
            != dependency_fingerprint(
                input
                    .edges
                    .iter()
                    .any(|row| matches!(row.target, SourceAtomicTermTarget::Application(_))),
                applications.map(SourceFunctorApplicationHandoff::debug_text),
            )
        {
            return Err(SourceAtomicFormulaError::ApplicationDependencyMismatch);
        }
        if self.structure_fingerprint
            != dependency_fingerprint(
                input
                    .edges
                    .iter()
                    .any(|row| matches!(row.target, SourceAtomicTermTarget::Structure(_))),
                structures.map(SourceStructureHandoff::debug_text),
            )
        {
            return Err(SourceAtomicFormulaError::StructureDependencyMismatch);
        }
        if self.set_term_fingerprint
            != dependency_fingerprint(
                input
                    .edges
                    .iter()
                    .any(|row| matches!(row.target, SourceAtomicTermTarget::SetTerm(_))),
                set_terms.map(SourceSetTermHandoff::debug_text),
            )
        {
            return Err(SourceAtomicFormulaError::SetTermDependencyMismatch);
        }
        Ok(())
    }

    fn to_input(&self) -> SourceAtomicFormulaHandoffInput {
        SourceAtomicFormulaHandoffInput {
            source_id: self.source_id,
            module_id: self.module_id.clone(),
            formulas: self
                .formulas
                .iter()
                .map(|(_, row)| SourceAtomicFormulaInput {
                    site: row.site.clone(),
                    source_range: row.source_range,
                    source_ordinal: row.source_ordinal,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling.clone(),
                    kind: row.kind,
                })
                .collect(),
            wrappers: self
                .wrappers
                .iter()
                .map(|(_, row)| SourceAtomicWrapperInput {
                    formula: row.formula,
                    ordinal: row.ordinal,
                    site: row.site.clone(),
                    source_range: row.source_range,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling.clone(),
                })
                .collect(),
            predicate_segments: self
                .predicate_segments
                .iter()
                .map(|(_, row)| SourcePredicateSegmentInput {
                    formula: row.formula,
                    ordinal: row.ordinal,
                    site: row.site.clone(),
                    source_range: row.source_range,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling.clone(),
                    head: row.head,
                    polarity: row.polarity.clone(),
                    left_edge: row.left_edge,
                    right_edge: row.right_edge,
                })
                .collect(),
            predicate_heads: self
                .predicate_heads
                .iter()
                .map(|(_, row)| SourcePredicateHeadInput {
                    formula: row.formula,
                    site: row.site.clone(),
                    source_range: row.source_range,
                    context: row.context,
                    recovery: row.recovery,
                    spelling: row.spelling.clone(),
                    left_arity: row.left_arity,
                    right_arity: row.right_arity,
                })
                .collect(),
            candidates: self
                .candidates
                .iter()
                .map(|(_, row)| SourcePredicateCandidateInput {
                    head: row.head,
                    ordinal: row.ordinal,
                    symbol: row.symbol.clone(),
                    contribution: row.contribution,
                })
                .collect(),
            type_sites: self
                .type_sites
                .iter()
                .map(|(_, row)| SourceAssertionTypeSiteInput {
                    formula: row.formula,
                    site: row.site.clone(),
                    source_range: row.source_range,
                    spelling: row.spelling.clone(),
                    head_site: row.head_site.clone(),
                    head_range: row.head_range,
                    head_spelling: row.head_spelling.clone(),
                    context: row.context,
                    recovery: row.recovery,
                    head: row.head,
                })
                .collect(),
            attributes: self
                .attributes
                .iter()
                .map(|(_, row)| SourceAssertionAttributeInput {
                    formula: row.formula,
                    ordinal: row.ordinal,
                    site: row.site.clone(),
                    source_range: row.source_range,
                    spelling: row.spelling.clone(),
                    target_site: row.target_site.clone(),
                    target_range: row.target_range,
                    target_spelling: row.target_spelling.clone(),
                    context: row.context,
                    recovery: row.recovery,
                    symbol: row.symbol.clone(),
                    contribution: row.contribution,
                    polarity: row.polarity.clone(),
                })
                .collect(),
            edges: self
                .edges
                .iter()
                .map(|(_, row)| SourceAtomicEdgeInput {
                    formula: row.formula,
                    ordinal: row.ordinal,
                    role: row.role,
                    target: row.target,
                })
                .collect(),
            requests: self
                .requests
                .iter()
                .map(|(_, row)| SourceAtomicRequestInput {
                    formula: row.formula,
                    ordinal: row.ordinal,
                    kind: row.kind,
                    edge: row.edge,
                    candidate: row.candidate,
                    type_site: row.type_site,
                    attribute: row.attribute,
                })
                .collect(),
        }
    }
}

macro_rules! table {
    ($table:ident, $row:ident, $id:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $table {
            rows: Vec<$row>,
        }

        impl $table {
            pub fn get(&self, id: $id) -> Option<&$row> {
                self.rows.get(id.index())
            }

            pub fn iter(&self) -> impl Iterator<Item = ($id, &$row)> {
                self.rows
                    .iter()
                    .enumerate()
                    .map(|(index, row)| ($id::new(index), row))
            }

            pub const fn len(&self) -> usize {
                self.rows.len()
            }

            pub const fn is_empty(&self) -> bool {
                self.rows.is_empty()
            }
        }
    };
}

table!(
    SourceAtomicFormulaTable,
    SourceAtomicFormula,
    SourceAtomicFormulaId
);
table!(
    SourceAtomicWrapperTable,
    SourceAtomicWrapper,
    SourceAtomicWrapperId
);
table!(
    SourcePredicateSegmentTable,
    SourcePredicateSegment,
    SourcePredicateSegmentId
);
table!(
    SourcePredicateHeadTable,
    SourcePredicateHead,
    SourcePredicateHeadId
);
table!(
    SourcePredicateCandidateTable,
    SourcePredicateCandidate,
    SourcePredicateCandidateId
);
table!(
    SourceAssertionTypeSiteTable,
    SourceAssertionTypeSite,
    SourceAssertionTypeSiteId
);
table!(
    SourceAssertionAttributeTable,
    SourceAssertionAttribute,
    SourceAssertionAttributeId
);
table!(SourceAtomicEdgeTable, SourceAtomicEdge, SourceAtomicEdgeId);
table!(
    SourceAtomicRequestTable,
    SourceAtomicRequest,
    SourceAtomicRequestId
);

/// One validated atomic formula occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAtomicFormula {
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    context: BindingContextId,
    recovery: SourceAtomicFormulaRecovery,
    spelling: String,
    kind: SourceAtomicFormulaKind,
}

impl SourceAtomicFormula {
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }
    pub const fn context(&self) -> BindingContextId {
        self.context
    }
    pub const fn recovery(&self) -> SourceAtomicFormulaRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn kind(&self) -> SourceAtomicFormulaKind {
        self.kind
    }
}

/// One validated transparent formula wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAtomicWrapper {
    formula: SourceAtomicFormulaId,
    ordinal: usize,
    site: TypedSiteRef,
    source_range: SourceRange,
    context: BindingContextId,
    recovery: SourceAtomicFormulaRecovery,
    spelling: String,
}

impl SourceAtomicWrapper {
    pub const fn formula(&self) -> SourceAtomicFormulaId {
        self.formula
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub const fn context(&self) -> BindingContextId {
        self.context
    }
    pub const fn recovery(&self) -> SourceAtomicFormulaRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

/// One validated source-written predicate-chain segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateSegment {
    formula: SourceAtomicFormulaId,
    ordinal: usize,
    site: TypedSiteRef,
    source_range: SourceRange,
    context: BindingContextId,
    recovery: SourceAtomicFormulaRecovery,
    spelling: String,
    head: SourcePredicateHeadId,
    polarity: SourcePredicateSegmentPolarityInput,
    left_edge: SourceAtomicEdgeId,
    right_edge: SourceAtomicEdgeId,
}

impl SourcePredicateSegment {
    pub const fn formula(&self) -> SourceAtomicFormulaId {
        self.formula
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub const fn context(&self) -> BindingContextId {
        self.context
    }
    pub const fn recovery(&self) -> SourceAtomicFormulaRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn head(&self) -> SourcePredicateHeadId {
        self.head
    }
    pub const fn polarity(&self) -> &SourcePredicateSegmentPolarityInput {
        &self.polarity
    }
    pub const fn left_edge(&self) -> SourceAtomicEdgeId {
        self.left_edge
    }
    pub const fn right_edge(&self) -> SourceAtomicEdgeId {
        self.right_edge
    }
}

/// One validated ordinary predicate head.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateHead {
    formula: SourceAtomicFormulaId,
    site: TypedSiteRef,
    source_range: SourceRange,
    context: BindingContextId,
    recovery: SourceAtomicFormulaRecovery,
    spelling: String,
    left_arity: usize,
    right_arity: usize,
}

impl SourcePredicateHead {
    pub const fn formula(&self) -> SourceAtomicFormulaId {
        self.formula
    }
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub const fn context(&self) -> BindingContextId {
        self.context
    }
    pub const fn recovery(&self) -> SourceAtomicFormulaRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn left_arity(&self) -> usize {
        self.left_arity
    }
    pub const fn right_arity(&self) -> usize {
        self.right_arity
    }
}

/// One validated predicate candidate reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePredicateCandidate {
    head: SourcePredicateHeadId,
    ordinal: usize,
    symbol: SymbolId,
    contribution: SourceContributionId,
}

impl SourcePredicateCandidate {
    pub const fn head(&self) -> SourcePredicateHeadId {
        self.head
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }
}

/// One validated formula-owned asserted type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAssertionTypeSite {
    formula: SourceAtomicFormulaId,
    site: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    head_site: TypedSiteRef,
    head_range: SourceRange,
    head_spelling: String,
    context: BindingContextId,
    recovery: SourceAtomicFormulaRecovery,
    head: SourceAssertionTypeHead,
}

impl SourceAssertionTypeSite {
    pub const fn formula(&self) -> SourceAtomicFormulaId {
        self.formula
    }
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn head_site(&self) -> &TypedSiteRef {
        &self.head_site
    }
    pub const fn head_range(&self) -> SourceRange {
        self.head_range
    }
    pub fn head_spelling(&self) -> &str {
        &self.head_spelling
    }
    pub const fn context(&self) -> BindingContextId {
        self.context
    }
    pub const fn recovery(&self) -> SourceAtomicFormulaRecovery {
        self.recovery
    }
    pub const fn head(&self) -> SourceAssertionTypeHead {
        self.head
    }
}

/// One validated formula-owned assertion attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAssertionAttribute {
    formula: SourceAtomicFormulaId,
    ordinal: usize,
    site: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    target_site: TypedSiteRef,
    target_range: SourceRange,
    target_spelling: String,
    context: BindingContextId,
    recovery: SourceAtomicFormulaRecovery,
    symbol: SymbolId,
    contribution: SourceContributionId,
    polarity: SourceAssertionAttributePolarityInput,
}

impl SourceAssertionAttribute {
    pub const fn formula(&self) -> SourceAtomicFormulaId {
        self.formula
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn target_site(&self) -> &TypedSiteRef {
        &self.target_site
    }
    pub const fn target_range(&self) -> SourceRange {
        self.target_range
    }
    pub fn target_spelling(&self) -> &str {
        &self.target_spelling
    }
    pub const fn context(&self) -> BindingContextId {
        self.context
    }
    pub const fn recovery(&self) -> SourceAtomicFormulaRecovery {
        self.recovery
    }
    pub const fn symbol(&self) -> &SymbolId {
        &self.symbol
    }
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }
    pub const fn polarity(&self) -> &SourceAssertionAttributePolarityInput {
        &self.polarity
    }
}

/// One validated direct formula-to-term association.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAtomicEdge {
    formula: SourceAtomicFormulaId,
    ordinal: usize,
    role: SourceAtomicEdgeRole,
    target: SourceAtomicTermTarget,
}

impl SourceAtomicEdge {
    pub const fn formula(&self) -> SourceAtomicFormulaId {
        self.formula
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn role(&self) -> SourceAtomicEdgeRole {
        self.role
    }
    pub const fn target(&self) -> SourceAtomicTermTarget {
        self.target
    }
}

/// One validated unresolved atomic-formula request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAtomicRequest {
    formula: SourceAtomicFormulaId,
    ordinal: usize,
    kind: SourceAtomicRequestKind,
    edge: Option<SourceAtomicEdgeId>,
    candidate: Option<SourcePredicateCandidateId>,
    type_site: Option<SourceAssertionTypeSiteId>,
    attribute: Option<SourceAssertionAttributeId>,
}

impl SourceAtomicRequest {
    pub const fn formula(&self) -> SourceAtomicFormulaId {
        self.formula
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn kind(&self) -> SourceAtomicRequestKind {
        self.kind
    }
    pub const fn edge(&self) -> Option<SourceAtomicEdgeId> {
        self.edge
    }
    pub const fn candidate(&self) -> Option<SourcePredicateCandidateId> {
        self.candidate
    }
    pub const fn type_site(&self) -> Option<SourceAssertionTypeSiteId> {
        self.type_site
    }
    pub const fn attribute(&self) -> Option<SourceAssertionAttributeId> {
        self.attribute
    }
}

/// Atomically validates and constructs source atomic-formula handoffs.
pub struct SourceAtomicFormulaProducer;

impl SourceAtomicFormulaProducer {
    #[allow(clippy::too_many_arguments)] // Rationale: the public transaction makes every frozen dependency explicit.
    pub fn build(
        input: SourceAtomicFormulaHandoffInput,
        bindings: &BindingEnv,
        symbols: &SymbolEnv,
        primary_terms: &SourcePrimaryTermHandoff,
        applications: Option<&SourceFunctorApplicationHandoff>,
        structures: Option<&SourceStructureHandoff>,
        set_terms: Option<&SourceSetTermHandoff>,
        arena: &TypedArena,
    ) -> Result<SourceAtomicFormulaHandoff, SourceAtomicFormulaError> {
        validate_input(
            &input,
            bindings,
            symbols,
            primary_terms,
            applications,
            structures,
            set_terms,
            arena,
        )?;

        let application_fingerprint = dependency_fingerprint(
            input
                .edges
                .iter()
                .any(|row| matches!(row.target, SourceAtomicTermTarget::Application(_))),
            applications.map(SourceFunctorApplicationHandoff::debug_text),
        );
        let structure_fingerprint = dependency_fingerprint(
            input
                .edges
                .iter()
                .any(|row| matches!(row.target, SourceAtomicTermTarget::Structure(_))),
            structures.map(SourceStructureHandoff::debug_text),
        );
        let set_term_fingerprint = dependency_fingerprint(
            input
                .edges
                .iter()
                .any(|row| matches!(row.target, SourceAtomicTermTarget::SetTerm(_))),
            set_terms.map(SourceSetTermHandoff::debug_text),
        );

        Ok(SourceAtomicFormulaHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            primary_term_fingerprint: primary_terms.debug_text(),
            application_fingerprint,
            structure_fingerprint,
            set_term_fingerprint,
            formulas: SourceAtomicFormulaTable {
                rows: input
                    .formulas
                    .into_iter()
                    .map(|row| SourceAtomicFormula {
                        site: row.site,
                        source_range: row.source_range,
                        source_ordinal: row.source_ordinal,
                        context: row.context,
                        recovery: row.recovery,
                        spelling: row.spelling,
                        kind: row.kind,
                    })
                    .collect(),
            },
            wrappers: SourceAtomicWrapperTable {
                rows: input
                    .wrappers
                    .into_iter()
                    .map(|row| SourceAtomicWrapper {
                        formula: row.formula,
                        ordinal: row.ordinal,
                        site: row.site,
                        source_range: row.source_range,
                        context: row.context,
                        recovery: row.recovery,
                        spelling: row.spelling,
                    })
                    .collect(),
            },
            predicate_segments: SourcePredicateSegmentTable {
                rows: input
                    .predicate_segments
                    .into_iter()
                    .map(|row| SourcePredicateSegment {
                        formula: row.formula,
                        ordinal: row.ordinal,
                        site: row.site,
                        source_range: row.source_range,
                        context: row.context,
                        recovery: row.recovery,
                        spelling: row.spelling,
                        head: row.head,
                        polarity: row.polarity,
                        left_edge: row.left_edge,
                        right_edge: row.right_edge,
                    })
                    .collect(),
            },
            predicate_heads: SourcePredicateHeadTable {
                rows: input
                    .predicate_heads
                    .into_iter()
                    .map(|row| SourcePredicateHead {
                        formula: row.formula,
                        site: row.site,
                        source_range: row.source_range,
                        context: row.context,
                        recovery: row.recovery,
                        spelling: row.spelling,
                        left_arity: row.left_arity,
                        right_arity: row.right_arity,
                    })
                    .collect(),
            },
            candidates: SourcePredicateCandidateTable {
                rows: input
                    .candidates
                    .into_iter()
                    .map(|row| SourcePredicateCandidate {
                        head: row.head,
                        ordinal: row.ordinal,
                        symbol: row.symbol,
                        contribution: row.contribution,
                    })
                    .collect(),
            },
            type_sites: SourceAssertionTypeSiteTable {
                rows: input
                    .type_sites
                    .into_iter()
                    .map(|row| SourceAssertionTypeSite {
                        formula: row.formula,
                        site: row.site,
                        source_range: row.source_range,
                        spelling: row.spelling,
                        head_site: row.head_site,
                        head_range: row.head_range,
                        head_spelling: row.head_spelling,
                        context: row.context,
                        recovery: row.recovery,
                        head: row.head,
                    })
                    .collect(),
            },
            attributes: SourceAssertionAttributeTable {
                rows: input
                    .attributes
                    .into_iter()
                    .map(|row| SourceAssertionAttribute {
                        formula: row.formula,
                        ordinal: row.ordinal,
                        site: row.site,
                        source_range: row.source_range,
                        spelling: row.spelling,
                        target_site: row.target_site,
                        target_range: row.target_range,
                        target_spelling: row.target_spelling,
                        context: row.context,
                        recovery: row.recovery,
                        symbol: row.symbol,
                        contribution: row.contribution,
                        polarity: row.polarity,
                    })
                    .collect(),
            },
            edges: SourceAtomicEdgeTable {
                rows: input
                    .edges
                    .into_iter()
                    .map(|row| SourceAtomicEdge {
                        formula: row.formula,
                        ordinal: row.ordinal,
                        role: row.role,
                        target: row.target,
                    })
                    .collect(),
            },
            requests: SourceAtomicRequestTable {
                rows: input
                    .requests
                    .into_iter()
                    .map(|row| SourceAtomicRequest {
                        formula: row.formula,
                        ordinal: row.ordinal,
                        kind: row.kind,
                        edge: row.edge,
                        candidate: row.candidate,
                        type_site: row.type_site,
                        attribute: row.attribute,
                    })
                    .collect(),
            },
        })
    }
}

/// Atomic Task-256 producer failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceAtomicFormulaError {
    EnvironmentMismatch,
    PrimaryDependencyMismatch,
    ApplicationDependencyMismatch,
    StructureDependencyMismatch,
    SetTermDependencyMismatch,
    InvalidFormula {
        formula: SourceAtomicFormulaId,
    },
    InvalidWrapper {
        wrapper: SourceAtomicWrapperId,
    },
    InvalidPredicateSegment {
        segment: SourcePredicateSegmentId,
    },
    InvalidPredicateHead {
        head: SourcePredicateHeadId,
    },
    InvalidCandidate {
        candidate: SourcePredicateCandidateId,
    },
    InvalidTypeSite {
        type_site: SourceAssertionTypeSiteId,
    },
    InvalidAttribute {
        attribute: SourceAssertionAttributeId,
    },
    InvalidEdge {
        edge: SourceAtomicEdgeId,
    },
    InvalidRequest {
        request: SourceAtomicRequestId,
    },
    DuplicateSite,
    ReorderedFormula {
        formula: SourceAtomicFormulaId,
    },
    ReorderedWrapper {
        wrapper: SourceAtomicWrapperId,
    },
    ReorderedPredicateSegment {
        segment: SourcePredicateSegmentId,
    },
    ReorderedPredicateHead {
        head: SourcePredicateHeadId,
    },
    ReorderedCandidate {
        candidate: SourcePredicateCandidateId,
    },
    ReorderedTypeSite {
        type_site: SourceAssertionTypeSiteId,
    },
    ReorderedAttribute {
        attribute: SourceAssertionAttributeId,
    },
    ReorderedEdge {
        edge: SourceAtomicEdgeId,
    },
    ReorderedRequest {
        request: SourceAtomicRequestId,
    },
    DuplicateTarget {
        edge: SourceAtomicEdgeId,
    },
    OverlappingTerms {
        formula: SourceAtomicFormulaId,
    },
}

impl fmt::Display for SourceAtomicFormulaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("source atomic-formula environment identity mismatch")
            }
            Self::PrimaryDependencyMismatch => {
                formatter.write_str("source atomic-formula primary dependency mismatch")
            }
            Self::ApplicationDependencyMismatch => {
                formatter.write_str("source atomic-formula application dependency mismatch")
            }
            Self::StructureDependencyMismatch => {
                formatter.write_str("source atomic-formula structure dependency mismatch")
            }
            Self::SetTermDependencyMismatch => {
                formatter.write_str("source atomic-formula set-term dependency mismatch")
            }
            Self::InvalidFormula { formula } => {
                write!(
                    formatter,
                    "source atomic formula {} is invalid",
                    formula.index()
                )
            }
            Self::InvalidWrapper { wrapper } => write!(
                formatter,
                "source atomic formula wrapper {} is invalid",
                wrapper.index()
            ),
            Self::InvalidPredicateSegment { segment } => write!(
                formatter,
                "source predicate segment {} is invalid",
                segment.index()
            ),
            Self::InvalidPredicateHead { head } => write!(
                formatter,
                "source predicate head {} is invalid",
                head.index()
            ),
            Self::InvalidCandidate { candidate } => write!(
                formatter,
                "source predicate candidate {} is invalid",
                candidate.index()
            ),
            Self::InvalidTypeSite { type_site } => write!(
                formatter,
                "source assertion type site {} is invalid",
                type_site.index()
            ),
            Self::InvalidAttribute { attribute } => write!(
                formatter,
                "source assertion attribute {} is invalid",
                attribute.index()
            ),
            Self::InvalidEdge { edge } => {
                write!(formatter, "source atomic edge {} is invalid", edge.index())
            }
            Self::InvalidRequest { request } => write!(
                formatter,
                "source atomic request {} is invalid",
                request.index()
            ),
            Self::DuplicateSite => {
                formatter.write_str("source atomic formula repeats a typed site")
            }
            Self::ReorderedFormula { formula } => write!(
                formatter,
                "source atomic formula {} is out of source order",
                formula.index()
            ),
            Self::ReorderedWrapper { wrapper } => write!(
                formatter,
                "source atomic wrapper {} is out of order",
                wrapper.index()
            ),
            Self::ReorderedPredicateSegment { segment } => write!(
                formatter,
                "source predicate segment {} is out of order",
                segment.index()
            ),
            Self::ReorderedPredicateHead { head } => {
                write!(
                    formatter,
                    "source predicate head {} is out of order",
                    head.index()
                )
            }
            Self::ReorderedCandidate { candidate } => write!(
                formatter,
                "source predicate candidate {} is out of order",
                candidate.index()
            ),
            Self::ReorderedTypeSite { type_site } => write!(
                formatter,
                "source assertion type site {} is out of order",
                type_site.index()
            ),
            Self::ReorderedAttribute { attribute } => write!(
                formatter,
                "source assertion attribute {} is out of order",
                attribute.index()
            ),
            Self::ReorderedEdge { edge } => {
                write!(
                    formatter,
                    "source atomic edge {} is out of order",
                    edge.index()
                )
            }
            Self::ReorderedRequest { request } => write!(
                formatter,
                "source atomic request {} is out of order",
                request.index()
            ),
            Self::DuplicateTarget { edge } => write!(
                formatter,
                "source atomic edge {} repeats a direct term target",
                edge.index()
            ),
            Self::OverlappingTerms { formula } => write!(
                formatter,
                "source atomic formula {} has overlapping direct terms",
                formula.index()
            ),
        }
    }
}

impl Error for SourceAtomicFormulaError {}

#[allow(clippy::too_many_arguments)] // Rationale: input validation owns the complete atomic transaction boundary.
fn validate_input(
    input: &SourceAtomicFormulaHandoffInput,
    bindings: &BindingEnv,
    symbols: &SymbolEnv,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    set_terms: Option<&SourceSetTermHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceAtomicFormulaError> {
    if bindings.source_id() != input.source_id
        || bindings.module_id() != &input.module_id
        || symbols.module_id() != &input.module_id
        || primary_terms.source_id() != input.source_id
        || primary_terms.module_id() != &input.module_id
    {
        return Err(SourceAtomicFormulaError::EnvironmentMismatch);
    }
    validate_dependencies(
        input.source_id,
        &input.module_id,
        primary_terms,
        applications,
        structures,
        set_terms,
        arena,
    )?;
    validate_payload(
        input,
        Some(bindings),
        Some(symbols),
        primary_terms,
        applications,
        structures,
        set_terms,
        arena,
    )
}

#[allow(clippy::too_many_arguments)] // Rationale: dependency identity is checked without hiding optional family inputs.
fn validate_dependencies(
    source_id: SourceId,
    module_id: &ModuleId,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    set_terms: Option<&SourceSetTermHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceAtomicFormulaError> {
    primary_terms
        .validate_installation(source_id, module_id, arena)
        .map_err(|_| SourceAtomicFormulaError::PrimaryDependencyMismatch)?;
    if let Some(applications) = applications {
        applications
            .validate_installation(source_id, module_id, primary_terms)
            .map_err(|_| SourceAtomicFormulaError::ApplicationDependencyMismatch)?;
    }
    if let Some(structures) = structures {
        structures
            .validate_installation(source_id, module_id, primary_terms, applications, arena)
            .map_err(|_| SourceAtomicFormulaError::StructureDependencyMismatch)?;
    }
    if let Some(set_terms) = set_terms {
        set_terms
            .validate_installation(
                source_id,
                module_id,
                primary_terms,
                applications,
                structures,
                arena,
            )
            .map_err(|_| SourceAtomicFormulaError::SetTermDependencyMismatch)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)] // Rationale: payload validation keeps all frozen tables and dependencies in one atomic check.
fn validate_payload(
    input: &SourceAtomicFormulaHandoffInput,
    bindings: Option<&BindingEnv>,
    symbols: Option<&SymbolEnv>,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    set_terms: Option<&SourceSetTermHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceAtomicFormulaError> {
    if input.formulas.is_empty()
        && (!input.wrappers.is_empty()
            || !input.predicate_segments.is_empty()
            || !input.predicate_heads.is_empty()
            || !input.candidates.is_empty()
            || !input.type_sites.is_empty()
            || !input.attributes.is_empty()
            || !input.edges.is_empty()
            || !input.requests.is_empty())
    {
        return Err(SourceAtomicFormulaError::EnvironmentMismatch);
    }
    let uses_applications = input
        .edges
        .iter()
        .any(|row| matches!(row.target, SourceAtomicTermTarget::Application(_)));
    let uses_structures = input
        .edges
        .iter()
        .any(|row| matches!(row.target, SourceAtomicTermTarget::Structure(_)));
    let uses_set_terms = input
        .edges
        .iter()
        .any(|row| matches!(row.target, SourceAtomicTermTarget::SetTerm(_)));
    if uses_applications && applications.is_none() {
        return Err(SourceAtomicFormulaError::ApplicationDependencyMismatch);
    }
    if uses_structures && structures.is_none() {
        return Err(SourceAtomicFormulaError::StructureDependencyMismatch);
    }
    if uses_set_terms && set_terms.is_none() {
        return Err(SourceAtomicFormulaError::SetTermDependencyMismatch);
    }

    let mut sites = BTreeSet::new();
    validate_formulas(input, bindings, arena, &mut sites)?;
    let wrapper_groups = validate_wrappers(input, arena, &mut sites)?;
    let effective = effective_formula_occurrences(input, &wrapper_groups);
    validate_formula_order(input, &effective)?;
    validate_cross_family_ranges(
        input,
        &effective,
        primary_terms,
        applications,
        structures,
        set_terms,
        arena,
    )?;
    let segment_groups = validate_predicate_segments(input, arena, &mut sites)?;
    let head_groups = validate_predicate_heads(input, arena, &mut sites, &segment_groups)?;
    let candidate_groups = validate_candidates(input, symbols, &head_groups)?;
    for (formula_index, segments) in segment_groups.iter().enumerate() {
        if !segments.is_empty()
            && head_groups[formula_index]
                .iter()
                .any(|head| candidate_groups[*head].len() != 1)
        {
            return Err(SourceAtomicFormulaError::InvalidFormula {
                formula: SourceAtomicFormulaId::new(formula_index),
            });
        }
    }
    let type_by_formula = validate_type_sites(input, arena, &mut sites)?;
    let attribute_groups = validate_attributes(input, symbols, arena, &mut sites)?;
    let edge_groups = validate_edges(input, primary_terms, applications, structures, set_terms)?;
    validate_shapes(
        input,
        primary_terms,
        applications,
        structures,
        set_terms,
        &segment_groups,
        &head_groups,
        &type_by_formula,
        &attribute_groups,
        &edge_groups,
    )?;
    validate_requests(
        input,
        &head_groups,
        &candidate_groups,
        &type_by_formula,
        &attribute_groups,
        &edge_groups,
    )
}

fn validate_cross_family_ranges(
    input: &SourceAtomicFormulaHandoffInput,
    effective: &[EffectiveOccurrence],
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    set_terms: Option<&SourceSetTermHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceAtomicFormulaError> {
    for (formula_index, formula) in input.formulas.iter().enumerate() {
        let outer = effective[formula_index].range;
        for (_, row) in primary_terms.terms().iter() {
            if ranges_overlap(outer, row.source_range())
                && !properly_contains(formula.source_range, row.source_range())
            {
                return Err(SourceAtomicFormulaError::PrimaryDependencyMismatch);
            }
        }
        if let Some(applications) = applications {
            for (id, _) in applications.applications().iter() {
                let occurrence = application_occurrence(applications, id)
                    .ok_or(SourceAtomicFormulaError::ApplicationDependencyMismatch)?;
                if ranges_overlap(outer, occurrence.range)
                    && !properly_contains(formula.source_range, occurrence.range)
                {
                    return Err(SourceAtomicFormulaError::ApplicationDependencyMismatch);
                }
            }
        }
        if let Some(structures) = structures {
            for (id, _) in structures.terms().iter() {
                let occurrence = structure_occurrence(structures, id)
                    .ok_or(SourceAtomicFormulaError::StructureDependencyMismatch)?;
                if ranges_overlap(outer, occurrence.range)
                    && !properly_contains(formula.source_range, occurrence.range)
                {
                    return Err(SourceAtomicFormulaError::StructureDependencyMismatch);
                }
            }
        }
        if let Some(set_terms) = set_terms {
            for (id, _) in set_terms.terms().iter() {
                let occurrence = set_term_occurrence(set_terms, id)
                    .ok_or(SourceAtomicFormulaError::SetTermDependencyMismatch)?;
                if ranges_overlap(outer, occurrence.range)
                    && !properly_contains(formula.source_range, occurrence.range)
                    && !is_authenticated_condition_container(
                        formula,
                        outer,
                        set_terms,
                        id,
                        occurrence.range,
                        arena,
                    )
                {
                    return Err(SourceAtomicFormulaError::SetTermDependencyMismatch);
                }
            }
        }
    }
    Ok(())
}

fn is_authenticated_condition_container(
    formula: &SourceAtomicFormulaInput,
    effective_formula_range: SourceRange,
    set_terms: &SourceSetTermHandoff,
    set_term_id: SourceSetTermId,
    set_term_range: SourceRange,
    arena: &TypedArena,
) -> bool {
    let Some(set_term) = set_terms.terms().get(set_term_id) else {
        return false;
    };
    if set_term.kind() != SourceSetTermKind::Comprehension
        || formula.kind != SourceAtomicFormulaKind::Equality
        || formula.recovery != SourceAtomicFormulaRecovery::Normal
        || formula.context != set_term.context()
        || formula.source_range != effective_formula_range
        || !properly_contains(set_term_range, formula.source_range)
    {
        return false;
    }
    set_terms.conditions().iter().any(|(_, condition)| {
        condition.term() == set_term_id
            && condition.recovery() == SourceSetTermRecovery::Normal
            && condition.source_range() == formula.source_range
            && condition.spelling() == formula.spelling
            && condition.condition_site() != &formula.site
            && arena
                .node(condition.condition_site().node())
                .is_some_and(|node| node.children.contains(&formula.site.node()))
    })
}

fn validate_formulas(
    input: &SourceAtomicFormulaHandoffInput,
    bindings: Option<&BindingEnv>,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceAtomicFormulaError> {
    for (index, formula) in input.formulas.iter().enumerate() {
        let id = SourceAtomicFormulaId::new(index);
        if formula.source_ordinal != index
            || bindings.is_some_and(|env| env.contexts().get(formula.context).is_none())
            || !valid_range(input.source_id, formula.source_range)
            || !canonical_spelling(&formula.spelling)
        {
            return Err(SourceAtomicFormulaError::InvalidFormula { formula: id });
        }
        validate_arena_site(
            &formula.site,
            formula.source_range,
            formula_node_key(formula.kind),
            formula.recovery,
            arena,
        )
        .map_err(|()| SourceAtomicFormulaError::InvalidFormula { formula: id })?;
        if !sites.insert(formula.site.clone()) {
            return Err(SourceAtomicFormulaError::DuplicateSite);
        }
    }
    Ok(())
}

fn validate_wrappers(
    input: &SourceAtomicFormulaHandoffInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<Vec<Vec<usize>>, SourceAtomicFormulaError> {
    let groups = grouped_rows(
        input.formulas.len(),
        &input.wrappers,
        |row| row.formula.index(),
        |row| row.ordinal,
        |index| SourceAtomicFormulaError::ReorderedWrapper {
            wrapper: SourceAtomicWrapperId::new(index),
        },
    )?;
    for (formula_index, group) in groups.iter().enumerate() {
        let formula_id = SourceAtomicFormulaId::new(formula_index);
        let formula = &input.formulas[formula_index];
        let mut inner_range = formula.source_range;
        let mut inner_spelling = formula.spelling.as_str();
        for wrapper_index in group.iter().rev().copied() {
            let id = SourceAtomicWrapperId::new(wrapper_index);
            let wrapper = &input.wrappers[wrapper_index];
            if wrapper.formula != formula_id
                || wrapper.context != formula.context
                || !valid_range(input.source_id, wrapper.source_range)
                || !strictly_contains(wrapper.source_range, inner_range)
                || wrapper.spelling != format!("( {inner_spelling} )")
            {
                return Err(SourceAtomicFormulaError::InvalidWrapper { wrapper: id });
            }
            validate_arena_site(
                &wrapper.site,
                wrapper.source_range,
                "source.formula.atomic.parenthesized",
                wrapper.recovery,
                arena,
            )
            .map_err(|()| SourceAtomicFormulaError::InvalidWrapper { wrapper: id })?;
            if !sites.insert(wrapper.site.clone()) {
                return Err(SourceAtomicFormulaError::DuplicateSite);
            }
            inner_range = wrapper.source_range;
            inner_spelling = &wrapper.spelling;
        }
    }
    Ok(groups)
}

#[derive(Debug, Clone)]
struct EffectiveOccurrence {
    range: SourceRange,
}

fn effective_formula_occurrences(
    input: &SourceAtomicFormulaHandoffInput,
    wrapper_groups: &[Vec<usize>],
) -> Vec<EffectiveOccurrence> {
    input
        .formulas
        .iter()
        .enumerate()
        .map(|(index, formula)| {
            wrapper_groups[index].first().map_or_else(
                || EffectiveOccurrence {
                    range: formula.source_range,
                },
                |wrapper| {
                    let wrapper = &input.wrappers[*wrapper];
                    EffectiveOccurrence {
                        range: wrapper.source_range,
                    }
                },
            )
        })
        .collect()
}

fn validate_formula_order(
    input: &SourceAtomicFormulaHandoffInput,
    effective: &[EffectiveOccurrence],
) -> Result<(), SourceAtomicFormulaError> {
    for index in 1..effective.len() {
        let previous = &effective[index - 1];
        let current = &effective[index];
        if current.range.start < previous.range.start
            || ranges_overlap(previous.range, current.range)
        {
            return Err(SourceAtomicFormulaError::ReorderedFormula {
                formula: SourceAtomicFormulaId::new(index),
            });
        }
    }
    debug_assert_eq!(effective.len(), input.formulas.len());
    Ok(())
}

fn validate_predicate_segments(
    input: &SourceAtomicFormulaHandoffInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<Vec<Vec<usize>>, SourceAtomicFormulaError> {
    let groups = grouped_rows(
        input.formulas.len(),
        &input.predicate_segments,
        |row| row.formula.index(),
        |row| row.ordinal,
        |index| SourceAtomicFormulaError::ReorderedPredicateSegment {
            segment: SourcePredicateSegmentId::new(index),
        },
    )?;
    for (formula_index, group) in groups.iter().enumerate() {
        let formula_id = SourceAtomicFormulaId::new(formula_index);
        let formula = &input.formulas[formula_index];
        if formula.kind != SourceAtomicFormulaKind::PredicateApplication {
            if !group.is_empty() {
                return Err(SourceAtomicFormulaError::InvalidFormula {
                    formula: formula_id,
                });
            }
            continue;
        }
        if !group.is_empty() && group.len() < 2 {
            return Err(SourceAtomicFormulaError::InvalidPredicateSegment {
                segment: SourcePredicateSegmentId::new(group[0]),
            });
        }
        let mut previous_range = None;
        for segment_index in group {
            let id = SourcePredicateSegmentId::new(*segment_index);
            let segment = &input.predicate_segments[*segment_index];
            if segment.context != formula.context
                || segment.recovery != formula.recovery
                || !valid_range(input.source_id, segment.source_range)
                || !properly_contains(formula.source_range, segment.source_range)
                || !canonical_spelling(&segment.spelling)
                || previous_range
                    .is_some_and(|previous: SourceRange| previous.end > segment.source_range.start)
            {
                return Err(SourceAtomicFormulaError::InvalidPredicateSegment { segment: id });
            }
            validate_arena_site(
                &segment.site,
                segment.source_range,
                "source.formula.atomic.predicate-segment",
                segment.recovery,
                arena,
            )
            .map_err(|()| SourceAtomicFormulaError::InvalidPredicateSegment { segment: id })?;
            if !sites.insert(segment.site.clone()) {
                return Err(SourceAtomicFormulaError::DuplicateSite);
            }
            validate_predicate_segment_polarity(input, id, segment, arena, sites)?;
            previous_range = Some(segment.source_range);
        }
    }
    Ok(groups)
}

fn validate_predicate_segment_polarity(
    input: &SourceAtomicFormulaHandoffInput,
    id: SourcePredicateSegmentId,
    segment: &SourcePredicateSegmentInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceAtomicFormulaError> {
    let invalid = || SourceAtomicFormulaError::InvalidPredicateSegment { segment: id };
    let SourcePredicateSegmentPolarityInput::Negative {
        verb_site,
        verb_range,
        verb_spelling,
        verb_recovery,
        not_site,
        not_range,
        not_spelling,
        not_recovery,
    } = &segment.polarity
    else {
        return Ok(());
    };
    if !matches!(verb_spelling.as_str(), "does" | "do")
        || not_spelling != "not"
        || *verb_recovery != segment.recovery
        || *not_recovery != segment.recovery
        || !valid_range(input.source_id, *verb_range)
        || !valid_range(input.source_id, *not_range)
        || !range_contains(segment.source_range, *verb_range)
        || !range_contains(segment.source_range, *not_range)
        || verb_range.end > not_range.start
        || verb_site == &segment.site
        || not_site == &segment.site
        || verb_site == not_site
    {
        return Err(invalid());
    }
    validate_arena_site(
        verb_site,
        *verb_range,
        "source.formula.atomic.predicate-negation-verb",
        *verb_recovery,
        arena,
    )
    .map_err(|()| invalid())?;
    validate_arena_site(
        not_site,
        *not_range,
        "source.formula.atomic.predicate-negation-not",
        *not_recovery,
        arena,
    )
    .map_err(|()| invalid())?;
    if !sites.insert(verb_site.clone()) || !sites.insert(not_site.clone()) {
        return Err(SourceAtomicFormulaError::DuplicateSite);
    }
    Ok(())
}

fn validate_predicate_heads(
    input: &SourceAtomicFormulaHandoffInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
    segment_groups: &[Vec<usize>],
) -> Result<Vec<Vec<usize>>, SourceAtomicFormulaError> {
    let mut groups = vec![Vec::new(); input.formulas.len()];
    let mut previous_formula = None;
    let mut previous_range = None;
    for (index, head) in input.predicate_heads.iter().enumerate() {
        let id = SourcePredicateHeadId::new(index);
        let Some(formula) = input.formulas.get(head.formula.index()) else {
            return Err(SourceAtomicFormulaError::InvalidPredicateHead { head: id });
        };
        let formula_changed = previous_formula.is_some_and(|previous| previous != head.formula);
        if previous_formula.is_some_and(|previous| previous > head.formula)
            || formula.kind != SourceAtomicFormulaKind::PredicateApplication
            || head.context != formula.context
            || !valid_range(input.source_id, head.source_range)
            || !properly_contains(formula.source_range, head.source_range)
            || !canonical_spelling(&head.spelling)
            || head.left_arity + head.right_arity == 0
            || (!formula_changed
                && previous_range
                    .is_some_and(|previous: SourceRange| previous.end > head.source_range.start))
        {
            return Err(SourceAtomicFormulaError::ReorderedPredicateHead { head: id });
        }
        validate_arena_site(
            &head.site,
            head.source_range,
            "source.formula.atomic.predicate-head",
            head.recovery,
            arena,
        )
        .map_err(|()| SourceAtomicFormulaError::InvalidPredicateHead { head: id })?;
        if !sites.insert(head.site.clone()) {
            return Err(SourceAtomicFormulaError::DuplicateSite);
        }
        groups[head.formula.index()].push(index);
        previous_formula = Some(head.formula);
        previous_range = Some(head.source_range);
    }
    for (index, formula) in input.formulas.iter().enumerate() {
        let heads = &groups[index];
        let segments = &segment_groups[index];
        let valid = if formula.kind != SourceAtomicFormulaKind::PredicateApplication {
            heads.is_empty() && segments.is_empty()
        } else if segments.is_empty() {
            heads.len() == 1
        } else {
            heads.len() == segments.len()
                && segments.iter().enumerate().all(|(ordinal, segment_index)| {
                    let segment = &input.predicate_segments[*segment_index];
                    let head = &input.predicate_heads[heads[ordinal]];
                    segment.head == SourcePredicateHeadId::new(heads[ordinal])
                        && head.recovery == segment.recovery
                        && properly_contains(segment.source_range, head.source_range)
                })
        };
        if !valid {
            return Err(SourceAtomicFormulaError::InvalidFormula {
                formula: SourceAtomicFormulaId::new(index),
            });
        }
    }
    Ok(groups)
}

fn validate_candidates(
    input: &SourceAtomicFormulaHandoffInput,
    symbols: Option<&SymbolEnv>,
    head_groups: &[Vec<usize>],
) -> Result<Vec<Vec<usize>>, SourceAtomicFormulaError> {
    let groups = grouped_rows(
        input.predicate_heads.len(),
        &input.candidates,
        |row| row.head.index(),
        |row| row.ordinal,
        |index| SourceAtomicFormulaError::ReorderedCandidate {
            candidate: SourcePredicateCandidateId::new(index),
        },
    )?;
    let mut seen = BTreeSet::new();
    for (head_index, group) in groups.iter().enumerate() {
        let head_id = SourcePredicateHeadId::new(head_index);
        let head = &input.predicate_heads[head_index];
        if group.is_empty() {
            return Err(SourceAtomicFormulaError::InvalidPredicateHead { head: head_id });
        }
        let formula = head.formula.index();
        if head_groups
            .get(formula)
            .is_none_or(|heads| !heads.contains(&head_index))
        {
            return Err(SourceAtomicFormulaError::InvalidPredicateHead { head: head_id });
        }
        for candidate_index in group {
            let id = SourcePredicateCandidateId::new(*candidate_index);
            let candidate = &input.candidates[*candidate_index];
            if !seen.insert((
                candidate.head,
                candidate.symbol.clone(),
                candidate.contribution,
            )) {
                return Err(SourceAtomicFormulaError::InvalidCandidate { candidate: id });
            }
            if let Some(symbols) = symbols {
                validate_symbol(
                    input,
                    symbols,
                    &candidate.symbol,
                    candidate.contribution,
                    SymbolKind::Predicate,
                    DefinitionKind::Predicate,
                    &head.spelling,
                    input.formulas[formula].source_range,
                )
                .map_err(|()| SourceAtomicFormulaError::InvalidCandidate { candidate: id })?;
            }
        }
    }
    Ok(groups)
}

fn validate_type_sites(
    input: &SourceAtomicFormulaHandoffInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<Vec<Option<usize>>, SourceAtomicFormulaError> {
    let mut by_formula = vec![None; input.formulas.len()];
    let mut previous_range = None;
    for (index, type_site) in input.type_sites.iter().enumerate() {
        let id = SourceAssertionTypeSiteId::new(index);
        let Some(formula) = input.formulas.get(type_site.formula.index()) else {
            return Err(SourceAtomicFormulaError::InvalidTypeSite { type_site: id });
        };
        let expected = type_head_spelling(type_site.head);
        if formula.kind != SourceAtomicFormulaKind::TypeAssertion
            || by_formula[type_site.formula.index()]
                .replace(index)
                .is_some()
            || type_site.context != formula.context
            || !valid_range(input.source_id, type_site.source_range)
            || type_site.source_range != type_site.head_range
            || !properly_contains(formula.source_range, type_site.source_range)
            || type_site.spelling != expected
            || type_site.head_spelling != expected
        {
            return Err(SourceAtomicFormulaError::InvalidTypeSite { type_site: id });
        }
        if previous_range
            .is_some_and(|previous: SourceRange| previous.end > type_site.source_range.start)
        {
            return Err(SourceAtomicFormulaError::ReorderedTypeSite { type_site: id });
        }
        validate_arena_site(
            &type_site.site,
            type_site.source_range,
            "source.formula.atomic.asserted-type",
            type_site.recovery,
            arena,
        )
        .map_err(|()| SourceAtomicFormulaError::InvalidTypeSite { type_site: id })?;
        validate_arena_site(
            &type_site.head_site,
            type_site.head_range,
            "source.formula.atomic.asserted-type-head",
            type_site.recovery,
            arena,
        )
        .map_err(|()| SourceAtomicFormulaError::InvalidTypeSite { type_site: id })?;
        if !sites.insert(type_site.site.clone()) || !sites.insert(type_site.head_site.clone()) {
            return Err(SourceAtomicFormulaError::DuplicateSite);
        }
        previous_range = Some(type_site.source_range);
    }
    for (index, formula) in input.formulas.iter().enumerate() {
        if (formula.kind == SourceAtomicFormulaKind::TypeAssertion) != by_formula[index].is_some() {
            return Err(SourceAtomicFormulaError::InvalidFormula {
                formula: SourceAtomicFormulaId::new(index),
            });
        }
    }
    Ok(by_formula)
}

fn validate_attributes(
    input: &SourceAtomicFormulaHandoffInput,
    symbols: Option<&SymbolEnv>,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<Vec<Vec<usize>>, SourceAtomicFormulaError> {
    let groups = grouped_rows(
        input.formulas.len(),
        &input.attributes,
        |row| row.formula.index(),
        |row| row.ordinal,
        |index| SourceAtomicFormulaError::ReorderedAttribute {
            attribute: SourceAssertionAttributeId::new(index),
        },
    )?;
    for (formula_index, group) in groups.iter().enumerate() {
        let formula_id = SourceAtomicFormulaId::new(formula_index);
        let formula = &input.formulas[formula_index];
        if (formula.kind == SourceAtomicFormulaKind::AttributeAssertion) == group.is_empty() {
            return Err(SourceAtomicFormulaError::InvalidFormula {
                formula: formula_id,
            });
        }
        let mut previous_range = None;
        for attribute_index in group {
            let id = SourceAssertionAttributeId::new(*attribute_index);
            let attribute = &input.attributes[*attribute_index];
            if attribute.context != formula.context
                || !valid_range(input.source_id, attribute.source_range)
                || !valid_range(input.source_id, attribute.target_range)
                || !properly_contains(formula.source_range, attribute.source_range)
                || !range_contains(attribute.source_range, attribute.target_range)
                || attribute.site == attribute.target_site
                || !identifier_spelling(&attribute.target_spelling)
                || previous_range.is_some_and(|previous: SourceRange| {
                    previous.end > attribute.source_range.start
                })
            {
                return Err(SourceAtomicFormulaError::InvalidAttribute { attribute: id });
            }
            validate_arena_site(
                &attribute.site,
                attribute.source_range,
                "source.formula.atomic.attribute",
                attribute.recovery,
                arena,
            )
            .map_err(|()| SourceAtomicFormulaError::InvalidAttribute { attribute: id })?;
            validate_arena_site(
                &attribute.target_site,
                attribute.target_range,
                "source.formula.atomic.attribute-target",
                attribute.recovery,
                arena,
            )
            .map_err(|()| SourceAtomicFormulaError::InvalidAttribute { attribute: id })?;
            if !sites.insert(attribute.site.clone()) || !sites.insert(attribute.target_site.clone())
            {
                return Err(SourceAtomicFormulaError::DuplicateSite);
            }
            validate_attribute_polarity(input, id, attribute, arena, sites)?;
            if let Some(symbols) = symbols {
                validate_symbol(
                    input,
                    symbols,
                    &attribute.symbol,
                    attribute.contribution,
                    SymbolKind::Attribute,
                    DefinitionKind::Attribute,
                    &attribute.target_spelling,
                    attribute.source_range,
                )
                .map_err(|()| SourceAtomicFormulaError::InvalidAttribute { attribute: id })?;
            }
            previous_range = Some(attribute.source_range);
        }
    }
    Ok(groups)
}

fn validate_attribute_polarity(
    input: &SourceAtomicFormulaHandoffInput,
    id: SourceAssertionAttributeId,
    attribute: &SourceAssertionAttributeInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceAtomicFormulaError> {
    let invalid = || SourceAtomicFormulaError::InvalidAttribute { attribute: id };
    match &attribute.polarity {
        SourceAssertionAttributePolarityInput::Positive => {
            if attribute.spelling != attribute.target_spelling
                || attribute.source_range != attribute.target_range
            {
                return Err(invalid());
            }
        }
        SourceAssertionAttributePolarityInput::Negative {
            non_site,
            non_range,
            non_spelling,
            non_recovery,
        } => {
            if attribute.spelling != format!("non {}", attribute.target_spelling)
                || non_spelling != "non"
                || *non_recovery != attribute.recovery
                || !valid_range(input.source_id, *non_range)
                || !range_contains(attribute.source_range, *non_range)
                || non_range.end > attribute.target_range.start
                || non_site == &attribute.site
                || non_site == &attribute.target_site
            {
                return Err(invalid());
            }
            validate_arena_site(
                non_site,
                *non_range,
                "source.formula.atomic.attribute-non",
                *non_recovery,
                arena,
            )
            .map_err(|()| invalid())?;
            if !sites.insert(non_site.clone()) {
                return Err(SourceAtomicFormulaError::DuplicateSite);
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct TargetOccurrence {
    target: SourceAtomicTermTarget,
    range: SourceRange,
    spelling: String,
    context: BindingContextId,
}

#[allow(clippy::too_many_arguments)] // Rationale: edge validation compares all four nearest-owner families explicitly.
fn validate_edges(
    input: &SourceAtomicFormulaHandoffInput,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    set_terms: Option<&SourceSetTermHandoff>,
) -> Result<Vec<Vec<usize>>, SourceAtomicFormulaError> {
    let groups = grouped_rows(
        input.formulas.len(),
        &input.edges,
        |row| row.formula.index(),
        |row| row.ordinal,
        |index| SourceAtomicFormulaError::ReorderedEdge {
            edge: SourceAtomicEdgeId::new(index),
        },
    )?;
    let application_roots = applications.map(application_root_ids);
    let structure_roots = structures.map(structure_root_ids);
    let set_roots = set_terms.map(set_term_root_ids);
    let application_owned_primary = applications
        .map(application_primary_ids)
        .unwrap_or_default();
    let structure_owned_primary = structures.map(structure_primary_ids).unwrap_or_default();
    let structure_owned_application = structures
        .map(structure_application_ids)
        .unwrap_or_default();
    let set_owned_primary = set_terms.map(set_primary_ids).unwrap_or_default();
    let set_owned_application = set_terms.map(set_application_ids).unwrap_or_default();
    let set_owned_structure = set_terms.map(set_structure_ids).unwrap_or_default();
    let mut owned_targets = BTreeSet::new();

    for (formula_index, group) in groups.iter().enumerate() {
        let formula_id = SourceAtomicFormulaId::new(formula_index);
        let formula = &input.formulas[formula_index];
        let expected = direct_targets(formula, primary_terms, applications, structures, set_terms)?;
        if group.len() != expected.len() {
            return Err(SourceAtomicFormulaError::InvalidFormula {
                formula: formula_id,
            });
        }
        let mut previous = None;
        for (ordinal, edge_index) in group.iter().copied().enumerate() {
            let id = SourceAtomicEdgeId::new(edge_index);
            let edge = &input.edges[edge_index];
            let occurrence = target_occurrence(
                edge.target,
                primary_terms,
                applications,
                structures,
                set_terms,
            )
            .ok_or(SourceAtomicFormulaError::InvalidEdge { edge: id })?;
            let root_is_valid = match edge.target {
                SourceAtomicTermTarget::Primary(primary) => {
                    !application_owned_primary.contains(&primary)
                        && !structure_owned_primary.contains(&primary)
                        && !set_owned_primary.contains(&primary)
                        && primary_terms
                            .terms()
                            .get(primary)
                            .is_some_and(|row| row.parent().is_none())
                }
                SourceAtomicTermTarget::Application(application) => {
                    application_roots
                        .as_ref()
                        .is_some_and(|roots| roots.contains(&application))
                        && !structure_owned_application.contains(&application)
                        && !set_owned_application.contains(&application)
                }
                SourceAtomicTermTarget::Structure(structure) => {
                    structure_roots
                        .as_ref()
                        .is_some_and(|roots| roots.contains(&structure))
                        && !set_owned_structure.contains(&structure)
                }
                SourceAtomicTermTarget::SetTerm(term) => set_roots
                    .as_ref()
                    .is_some_and(|roots| roots.contains(&term)),
            };
            if !root_is_valid
                || occurrence.context != formula.context
                || !properly_contains(formula.source_range, occurrence.range)
                || expected
                    .get(ordinal)
                    .is_none_or(|row| row.target != edge.target)
            {
                return Err(SourceAtomicFormulaError::InvalidEdge { edge: id });
            }
            if !owned_targets.insert(edge.target) {
                return Err(SourceAtomicFormulaError::DuplicateTarget { edge: id });
            }
            if previous.is_some_and(|range: SourceRange| range.end > occurrence.range.start) {
                return Err(SourceAtomicFormulaError::OverlappingTerms {
                    formula: formula_id,
                });
            }
            previous = Some(occurrence.range);
        }
    }
    Ok(groups)
}

#[allow(clippy::too_many_arguments)] // Rationale: direct-target partitioning must inspect every lower-family graph.
fn direct_targets(
    formula: &SourceAtomicFormulaInput,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    set_terms: Option<&SourceSetTermHandoff>,
) -> Result<Vec<TargetOccurrence>, SourceAtomicFormulaError> {
    let application_owned_primary = applications
        .map(application_primary_ids)
        .unwrap_or_default();
    let structure_owned_primary = structures.map(structure_primary_ids).unwrap_or_default();
    let structure_owned_application = structures
        .map(structure_application_ids)
        .unwrap_or_default();
    let set_owned_primary = set_terms.map(set_primary_ids).unwrap_or_default();
    let set_owned_application = set_terms.map(set_application_ids).unwrap_or_default();
    let set_owned_structure = set_terms.map(set_structure_ids).unwrap_or_default();
    let mut candidates = Vec::new();

    for (id, row) in primary_terms.terms().iter() {
        if row.parent().is_some()
            || application_owned_primary.contains(&id)
            || structure_owned_primary.contains(&id)
            || set_owned_primary.contains(&id)
            || !properly_contains(formula.source_range, row.source_range())
        {
            continue;
        }
        if row.context() != formula.context {
            return Err(SourceAtomicFormulaError::PrimaryDependencyMismatch);
        }
        candidates.push(TargetOccurrence {
            target: SourceAtomicTermTarget::Primary(id),
            range: row.source_range(),
            spelling: row.spelling().to_owned(),
            context: row.context(),
        });
    }
    if let Some(applications) = applications {
        for id in application_root_ids(applications) {
            if structure_owned_application.contains(&id) || set_owned_application.contains(&id) {
                continue;
            }
            let occurrence = application_occurrence(applications, id)
                .ok_or(SourceAtomicFormulaError::ApplicationDependencyMismatch)?;
            if !properly_contains(formula.source_range, occurrence.range) {
                continue;
            }
            if occurrence.context != formula.context {
                return Err(SourceAtomicFormulaError::ApplicationDependencyMismatch);
            }
            candidates.push(TargetOccurrence {
                target: SourceAtomicTermTarget::Application(id),
                ..occurrence
            });
        }
    }
    if let Some(structures) = structures {
        for id in structure_root_ids(structures) {
            if set_owned_structure.contains(&id) {
                continue;
            }
            let occurrence = structure_occurrence(structures, id)
                .ok_or(SourceAtomicFormulaError::StructureDependencyMismatch)?;
            if !properly_contains(formula.source_range, occurrence.range) {
                continue;
            }
            if occurrence.context != formula.context {
                return Err(SourceAtomicFormulaError::StructureDependencyMismatch);
            }
            candidates.push(TargetOccurrence {
                target: SourceAtomicTermTarget::Structure(id),
                ..occurrence
            });
        }
    }
    if let Some(set_terms) = set_terms {
        for id in set_term_root_ids(set_terms) {
            let occurrence = set_term_occurrence(set_terms, id)
                .ok_or(SourceAtomicFormulaError::SetTermDependencyMismatch)?;
            if !properly_contains(formula.source_range, occurrence.range) {
                continue;
            }
            if occurrence.context != formula.context {
                return Err(SourceAtomicFormulaError::SetTermDependencyMismatch);
            }
            candidates.push(TargetOccurrence {
                target: SourceAtomicTermTarget::SetTerm(id),
                ..occurrence
            });
        }
    }

    let all = candidates.clone();
    candidates.retain(|candidate| {
        !all.iter().any(|container| {
            container.target != candidate.target
                && properly_contains(container.range, candidate.range)
        })
    });
    candidates.sort_by_key(|row| (row.range.start, row.range.end, row.target));
    if candidates
        .windows(2)
        .any(|pair| ranges_overlap(pair[0].range, pair[1].range))
    {
        return Err(SourceAtomicFormulaError::OverlappingTerms {
            formula: SourceAtomicFormulaId::new(formula.source_ordinal),
        });
    }
    Ok(candidates)
}

#[allow(clippy::too_many_arguments)] // Rationale: shape validation checks each cross-family target against the same transaction.
fn validate_shapes(
    input: &SourceAtomicFormulaHandoffInput,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    set_terms: Option<&SourceSetTermHandoff>,
    segment_groups: &[Vec<usize>],
    head_groups: &[Vec<usize>],
    type_by_formula: &[Option<usize>],
    attribute_groups: &[Vec<usize>],
    edge_groups: &[Vec<usize>],
) -> Result<(), SourceAtomicFormulaError> {
    for (formula_index, formula) in input.formulas.iter().enumerate() {
        let formula_id = SourceAtomicFormulaId::new(formula_index);
        let edge_indexes = &edge_groups[formula_index];
        let edges = edge_indexes
            .iter()
            .map(|index| &input.edges[*index])
            .collect::<Vec<_>>();
        let occurrences = edges
            .iter()
            .enumerate()
            .map(|(ordinal, edge)| {
                target_occurrence(
                    edge.target,
                    primary_terms,
                    applications,
                    structures,
                    set_terms,
                )
                .ok_or(SourceAtomicFormulaError::InvalidEdge {
                    edge: SourceAtomicEdgeId::new(edge_indexes[ordinal]),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let (expected_spelling, positions_valid) =
            match formula.kind {
                SourceAtomicFormulaKind::PredicateApplication => {
                    let segments = &segment_groups[formula_index];
                    let heads = &head_groups[formula_index];
                    if segments.is_empty() {
                        let Some(head_index) = heads.first().copied() else {
                            return Err(SourceAtomicFormulaError::InvalidFormula {
                                formula: formula_id,
                            });
                        };
                        let head = &input.predicate_heads[head_index];
                        if edges.len() != head.left_arity + head.right_arity
                            || edges.iter().take(head.left_arity).any(|edge| {
                                edge.role != SourceAtomicEdgeRole::PredicateLeftArgument
                            })
                            || edges.iter().skip(head.left_arity).any(|edge| {
                                edge.role != SourceAtomicEdgeRole::PredicateRightArgument
                            })
                        {
                            return Err(SourceAtomicFormulaError::InvalidFormula {
                                formula: formula_id,
                            });
                        }
                        let left = occurrences
                            .iter()
                            .take(head.left_arity)
                            .map(|row| row.spelling.as_str())
                            .collect::<Vec<_>>()
                            .join(" , ");
                        let right = occurrences
                            .iter()
                            .skip(head.left_arity)
                            .map(|row| row.spelling.as_str())
                            .collect::<Vec<_>>()
                            .join(" , ");
                        let spelling = [left.as_str(), head.spelling.as_str(), right.as_str()]
                            .into_iter()
                            .filter(|part| !part.is_empty())
                            .collect::<Vec<_>>()
                            .join(" ");
                        let left_valid = occurrences
                            .get(head.left_arity.wrapping_sub(1))
                            .is_none_or(|row| row.range.end <= head.source_range.start);
                        let right_valid = occurrences
                            .get(head.left_arity)
                            .is_none_or(|row| head.source_range.end <= row.range.start);
                        (spelling, left_valid && right_valid)
                    } else {
                        if edges.len() != segments.len() + 1
                            || heads.len() != segments.len()
                            || edges.iter().enumerate().any(|(ordinal, edge)| {
                                edge.role
                                    != if ordinal == 0 {
                                        SourceAtomicEdgeRole::PredicateLeftArgument
                                    } else if ordinal + 1 == edges.len() {
                                        SourceAtomicEdgeRole::PredicateRightArgument
                                    } else {
                                        SourceAtomicEdgeRole::PredicateChainBoundary
                                    }
                            })
                        {
                            return Err(SourceAtomicFormulaError::InvalidFormula {
                                formula: formula_id,
                            });
                        }
                        let mut spellings = Vec::with_capacity(segments.len());
                        for (ordinal, segment_index) in segments.iter().copied().enumerate() {
                            let segment_id = SourcePredicateSegmentId::new(segment_index);
                            let segment = &input.predicate_segments[segment_index];
                            let head_index = heads[ordinal];
                            let head = &input.predicate_heads[head_index];
                            let left_edge = SourceAtomicEdgeId::new(edge_indexes[ordinal]);
                            let right_edge = SourceAtomicEdgeId::new(edge_indexes[ordinal + 1]);
                            let left = &occurrences[ordinal];
                            let right = &occurrences[ordinal + 1];
                            if segment.head != SourcePredicateHeadId::new(head_index)
                                || segment.left_edge != left_edge
                                || segment.right_edge != right_edge
                                || head.left_arity != 1
                                || head.right_arity != 1
                                || !properly_contains(segment.source_range, head.source_range)
                                || !range_contains(segment.source_range, right.range)
                                || head.source_range.end > right.range.start
                                || (ordinal == 0
                                    && (!range_contains(segment.source_range, left.range)
                                        || left.range.end > head.source_range.start))
                                || (ordinal > 0
                                    && (range_contains(segment.source_range, left.range)
                                        || left.range.end > segment.source_range.start))
                            {
                                return Err(SourceAtomicFormulaError::InvalidPredicateSegment {
                                    segment: segment_id,
                                });
                            }
                            let polarity = match &segment.polarity {
                                SourcePredicateSegmentPolarityInput::Positive => String::new(),
                                SourcePredicateSegmentPolarityInput::Negative {
                                    verb_range,
                                    verb_spelling,
                                    not_range,
                                    not_spelling,
                                    ..
                                } => {
                                    if verb_range.start < segment.source_range.start
                                        || verb_range.end > not_range.start
                                        || not_range.end > head.source_range.start
                                    {
                                        return Err(
                                            SourceAtomicFormulaError::InvalidPredicateSegment {
                                                segment: segment_id,
                                            },
                                        );
                                    }
                                    format!("{verb_spelling} {not_spelling} ")
                                }
                            };
                            let expected = if ordinal == 0 {
                                format!(
                                    "{} {polarity}{} {}",
                                    left.spelling, head.spelling, right.spelling
                                )
                            } else {
                                format!("{polarity}{} {}", head.spelling, right.spelling)
                            };
                            if segment.spelling != expected {
                                return Err(SourceAtomicFormulaError::InvalidPredicateSegment {
                                    segment: segment_id,
                                });
                            }
                            spellings.push(expected);
                        }
                        (spellings.join(" "), true)
                    }
                }
                SourceAtomicFormulaKind::Equality
                | SourceAtomicFormulaKind::Inequality
                | SourceAtomicFormulaKind::Membership => {
                    if edges.len() != 2
                        || edges[0].role != SourceAtomicEdgeRole::BuiltinLeftOperand
                        || edges[1].role != SourceAtomicEdgeRole::BuiltinRightOperand
                    {
                        return Err(SourceAtomicFormulaError::InvalidFormula {
                            formula: formula_id,
                        });
                    }
                    let operator = match formula.kind {
                        SourceAtomicFormulaKind::Equality => "=",
                        SourceAtomicFormulaKind::Inequality => "<>",
                        SourceAtomicFormulaKind::Membership => "in",
                        _ => unreachable!(),
                    };
                    (
                        format!(
                            "{} {operator} {}",
                            occurrences[0].spelling, occurrences[1].spelling
                        ),
                        occurrences[0].range.end <= occurrences[1].range.start,
                    )
                }
                SourceAtomicFormulaKind::TypeAssertion => {
                    if edges.len() != 1
                        || edges[0].role != SourceAtomicEdgeRole::AssertionSubject
                        || !attribute_groups[formula_index].is_empty()
                    {
                        return Err(SourceAtomicFormulaError::InvalidFormula {
                            formula: formula_id,
                        });
                    }
                    let type_site = type_by_formula[formula_index]
                        .and_then(|index| input.type_sites.get(index))
                        .ok_or(SourceAtomicFormulaError::InvalidFormula {
                            formula: formula_id,
                        })?;
                    (
                        format!("{} is {}", occurrences[0].spelling, type_site.spelling),
                        occurrences[0].range.end <= type_site.source_range.start,
                    )
                }
                SourceAtomicFormulaKind::AttributeAssertion => {
                    if edges.len() != 1
                        || edges[0].role != SourceAtomicEdgeRole::AssertionSubject
                        || type_by_formula[formula_index].is_some()
                    {
                        return Err(SourceAtomicFormulaError::InvalidFormula {
                            formula: formula_id,
                        });
                    }
                    let attributes = attribute_groups[formula_index]
                        .iter()
                        .map(|index| input.attributes[*index].spelling.as_str())
                        .collect::<Vec<_>>()
                        .join(" ");
                    let first = &input.attributes[attribute_groups[formula_index][0]];
                    (
                        format!("{} is {attributes}", occurrences[0].spelling),
                        occurrences[0].range.end <= first.source_range.start,
                    )
                }
            };
        if formula.spelling != expected_spelling || !positions_valid {
            return Err(SourceAtomicFormulaError::InvalidFormula {
                formula: formula_id,
            });
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)] // Rationale: request validation receives each precomputed association group explicitly.
fn validate_requests(
    input: &SourceAtomicFormulaHandoffInput,
    head_groups: &[Vec<usize>],
    candidate_groups: &[Vec<usize>],
    type_by_formula: &[Option<usize>],
    attribute_groups: &[Vec<usize>],
    edge_groups: &[Vec<usize>],
) -> Result<(), SourceAtomicFormulaError> {
    let groups = grouped_rows(
        input.formulas.len(),
        &input.requests,
        |row| row.formula.index(),
        |row| row.ordinal,
        |index| SourceAtomicFormulaError::ReorderedRequest {
            request: SourceAtomicRequestId::new(index),
        },
    )?;
    for (formula_index, group) in groups.iter().enumerate() {
        let formula = &input.formulas[formula_index];
        let expected = match formula.kind {
            SourceAtomicFormulaKind::PredicateApplication => head_groups[formula_index]
                .iter()
                .flat_map(|head| candidate_groups[*head].iter())
                .map(|candidate| {
                    (
                        SourceAtomicRequestKind::PredicateCandidateSignature,
                        None,
                        Some(SourcePredicateCandidateId::new(*candidate)),
                        None,
                        None,
                    )
                })
                .collect::<Vec<_>>(),
            SourceAtomicFormulaKind::Equality | SourceAtomicFormulaKind::Inequality => edge_groups
                [formula_index]
                .iter()
                .map(|edge| {
                    (
                        SourceAtomicRequestKind::OperandExpectedType,
                        Some(SourceAtomicEdgeId::new(*edge)),
                        None,
                        None,
                        None,
                    )
                })
                .collect(),
            SourceAtomicFormulaKind::Membership => vec![(
                SourceAtomicRequestKind::OperandExpectedType,
                Some(SourceAtomicEdgeId::new(edge_groups[formula_index][1])),
                None,
                None,
                None,
            )],
            SourceAtomicFormulaKind::TypeAssertion => vec![(
                SourceAtomicRequestKind::TypeAssertionReachability,
                None,
                None,
                type_by_formula[formula_index].map(SourceAssertionTypeSiteId::new),
                None,
            )],
            SourceAtomicFormulaKind::AttributeAssertion => attribute_groups[formula_index]
                .iter()
                .map(|attribute| {
                    (
                        SourceAtomicRequestKind::AttributeAdmissibility,
                        None,
                        None,
                        None,
                        Some(SourceAssertionAttributeId::new(*attribute)),
                    )
                })
                .collect(),
        };
        if group.len() != expected.len() {
            return Err(SourceAtomicFormulaError::InvalidRequest {
                request: SourceAtomicRequestId::new(
                    group.first().copied().unwrap_or(input.requests.len()),
                ),
            });
        }
        for (request_index, expected) in group.iter().copied().zip(expected) {
            let request = &input.requests[request_index];
            if (
                request.kind,
                request.edge,
                request.candidate,
                request.type_site,
                request.attribute,
            ) != expected
            {
                return Err(SourceAtomicFormulaError::InvalidRequest {
                    request: SourceAtomicRequestId::new(request_index),
                });
            }
        }
    }
    Ok(())
}

fn target_occurrence(
    target: SourceAtomicTermTarget,
    primary_terms: &SourcePrimaryTermHandoff,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    set_terms: Option<&SourceSetTermHandoff>,
) -> Option<TargetOccurrence> {
    match target {
        SourceAtomicTermTarget::Primary(id) => {
            let row = primary_terms.terms().get(id)?;
            Some(TargetOccurrence {
                target,
                range: row.source_range(),
                spelling: row.spelling().to_owned(),
                context: row.context(),
            })
        }
        SourceAtomicTermTarget::Application(id) => application_occurrence(applications?, id),
        SourceAtomicTermTarget::Structure(id) => structure_occurrence(structures?, id),
        SourceAtomicTermTarget::SetTerm(id) => set_term_occurrence(set_terms?, id),
    }
}

fn application_occurrence(
    handoff: &SourceFunctorApplicationHandoff,
    id: SourceFunctorApplicationId,
) -> Option<TargetOccurrence> {
    let row = handoff.applications().get(id)?;
    Some(
        handoff
            .wrappers()
            .iter()
            .find(|(_, wrapper)| wrapper.application() == id && wrapper.ordinal() == 0)
            .map_or_else(
                || TargetOccurrence {
                    target: SourceAtomicTermTarget::Application(id),
                    range: row.source_range(),
                    spelling: row.spelling().to_owned(),
                    context: row.context(),
                },
                |(_, wrapper)| TargetOccurrence {
                    target: SourceAtomicTermTarget::Application(id),
                    range: wrapper.source_range(),
                    spelling: wrapper.spelling().to_owned(),
                    context: wrapper.context(),
                },
            ),
    )
}

fn structure_occurrence(
    handoff: &SourceStructureHandoff,
    id: SourceStructureTermId,
) -> Option<TargetOccurrence> {
    let row = handoff.terms().get(id)?;
    Some(
        handoff
            .wrappers()
            .iter()
            .find(|(_, wrapper)| wrapper.term() == id && wrapper.ordinal() == 0)
            .map_or_else(
                || TargetOccurrence {
                    target: SourceAtomicTermTarget::Structure(id),
                    range: row.source_range(),
                    spelling: row.spelling().to_owned(),
                    context: row.context(),
                },
                |(_, wrapper)| TargetOccurrence {
                    target: SourceAtomicTermTarget::Structure(id),
                    range: wrapper.source_range(),
                    spelling: wrapper.spelling().to_owned(),
                    context: wrapper.context(),
                },
            ),
    )
}

fn set_term_occurrence(
    handoff: &SourceSetTermHandoff,
    id: SourceSetTermId,
) -> Option<TargetOccurrence> {
    let row = handoff.terms().get(id)?;
    Some(
        handoff
            .wrappers()
            .iter()
            .find(|(_, wrapper)| wrapper.term() == id && wrapper.ordinal() == 0)
            .map_or_else(
                || TargetOccurrence {
                    target: SourceAtomicTermTarget::SetTerm(id),
                    range: row.source_range(),
                    spelling: row.spelling().to_owned(),
                    context: row.context(),
                },
                |(_, wrapper)| TargetOccurrence {
                    target: SourceAtomicTermTarget::SetTerm(id),
                    range: wrapper.source_range(),
                    spelling: wrapper.spelling().to_owned(),
                    context: wrapper.context(),
                },
            ),
    )
}

fn application_root_ids(
    handoff: &SourceFunctorApplicationHandoff,
) -> BTreeSet<SourceFunctorApplicationId> {
    let nested = handoff
        .arguments()
        .iter()
        .filter_map(|(_, row)| match row.target() {
            SourceFunctorArgumentTarget::Application(id) => Some(id),
            SourceFunctorArgumentTarget::Primary(_) => None,
        })
        .collect::<BTreeSet<_>>();
    handoff
        .applications()
        .iter()
        .map(|(id, _)| id)
        .filter(|id| !nested.contains(id))
        .collect()
}

fn application_primary_ids(
    handoff: &SourceFunctorApplicationHandoff,
) -> BTreeSet<SourcePrimaryTermId> {
    handoff
        .arguments()
        .iter()
        .filter_map(|(_, row)| match row.target() {
            SourceFunctorArgumentTarget::Primary(id) => Some(id),
            SourceFunctorArgumentTarget::Application(_) => None,
        })
        .collect()
}

fn structure_root_ids(handoff: &SourceStructureHandoff) -> BTreeSet<SourceStructureTermId> {
    let nested = handoff
        .edges()
        .iter()
        .filter_map(|(_, row)| match row.target() {
            SourceStructureTarget::Structure(id) => Some(id),
            SourceStructureTarget::Primary(_) | SourceStructureTarget::Application(_) => None,
        })
        .collect::<BTreeSet<_>>();
    handoff
        .terms()
        .iter()
        .map(|(id, _)| id)
        .filter(|id| !nested.contains(id))
        .collect()
}

fn structure_primary_ids(handoff: &SourceStructureHandoff) -> BTreeSet<SourcePrimaryTermId> {
    handoff
        .edges()
        .iter()
        .filter_map(|(_, row)| match row.target() {
            SourceStructureTarget::Primary(id) => Some(id),
            SourceStructureTarget::Application(_) | SourceStructureTarget::Structure(_) => None,
        })
        .collect()
}

fn structure_application_ids(
    handoff: &SourceStructureHandoff,
) -> BTreeSet<SourceFunctorApplicationId> {
    handoff
        .edges()
        .iter()
        .filter_map(|(_, row)| match row.target() {
            SourceStructureTarget::Application(id) => Some(id),
            SourceStructureTarget::Primary(_) | SourceStructureTarget::Structure(_) => None,
        })
        .collect()
}

fn set_term_root_ids(handoff: &SourceSetTermHandoff) -> BTreeSet<SourceSetTermId> {
    let nested = handoff
        .edges()
        .iter()
        .filter_map(|(_, row)| match row.target() {
            SourceSetTarget::SetTerm(id) => Some(id),
            SourceSetTarget::Primary(_)
            | SourceSetTarget::Application(_)
            | SourceSetTarget::Structure(_) => None,
        })
        .collect::<BTreeSet<_>>();
    handoff
        .terms()
        .iter()
        .map(|(id, _)| id)
        .filter(|id| !nested.contains(id))
        .collect()
}

fn set_primary_ids(handoff: &SourceSetTermHandoff) -> BTreeSet<SourcePrimaryTermId> {
    handoff
        .edges()
        .iter()
        .filter_map(|(_, row)| match row.target() {
            SourceSetTarget::Primary(id) => Some(id),
            SourceSetTarget::Application(_)
            | SourceSetTarget::Structure(_)
            | SourceSetTarget::SetTerm(_) => None,
        })
        .collect()
}

fn set_application_ids(handoff: &SourceSetTermHandoff) -> BTreeSet<SourceFunctorApplicationId> {
    handoff
        .edges()
        .iter()
        .filter_map(|(_, row)| match row.target() {
            SourceSetTarget::Application(id) => Some(id),
            SourceSetTarget::Primary(_)
            | SourceSetTarget::Structure(_)
            | SourceSetTarget::SetTerm(_) => None,
        })
        .collect()
}

fn set_structure_ids(handoff: &SourceSetTermHandoff) -> BTreeSet<SourceStructureTermId> {
    handoff
        .edges()
        .iter()
        .filter_map(|(_, row)| match row.target() {
            SourceSetTarget::Structure(id) => Some(id),
            SourceSetTarget::Primary(_)
            | SourceSetTarget::Application(_)
            | SourceSetTarget::SetTerm(_) => None,
        })
        .collect()
}

#[allow(clippy::too_many_arguments)] // Rationale: provenance validation authenticates the complete frozen symbol identity.
fn validate_symbol(
    input: &SourceAtomicFormulaHandoffInput,
    symbols: &SymbolEnv,
    symbol: &SymbolId,
    contribution_id: SourceContributionId,
    kind: SymbolKind,
    definition_kind: DefinitionKind,
    spelling: &str,
    use_range: SourceRange,
) -> Result<(), ()> {
    let entry = symbols.symbols().get(symbol).ok_or(())?;
    let contribution = symbols.contributions().get(contribution_id).ok_or(())?;
    if entry.contribution() != contribution_id
        || entry.kind() != kind
        || entry.primary_spelling() != spelling
        || entry.namespace().as_str() != input.module_id.path().as_str()
        || contribution.module() != symbol.module()
        || !contribution.effects().symbols().contains(symbol)
        || entry.origin().is_recovered()
        || matches!(entry.signature(), Some(SignatureShell::Malformed { .. }))
    {
        return Err(());
    }
    match contribution.kind() {
        ContributionKind::LocalSource { source_id } => {
            let definition = symbols.definitions().by_symbol(symbol).ok_or(())?;
            let origin_range = source_range(entry.origin().anchor()).ok_or(())?;
            if *source_id != input.source_id
                || contribution.module() != &input.module_id
                || symbol.module() != &input.module_id
                || entry.origin().source_id() != input.source_id
                || entry.origin().module_id() != &input.module_id
                || entry.origin().import_edge().is_some()
                || !valid_range(input.source_id, origin_range)
                || origin_range.end > use_range.start
                || definition.kind() != definition_kind
                || definition.symbol() != symbol
                || definition.contribution() != contribution_id
                || definition.origin() != entry.origin()
                || definition.visibility() != entry.visibility()
                || definition.signature() != entry.signature()
                || definition.conflict().is_some()
                || !contribution
                    .effects()
                    .definitions()
                    .contains(&definition.id())
            {
                return Err(());
            }
        }
        ContributionKind::ImportedSource { source_id } => {
            let contribution_range = source_range(contribution.anchor()).ok_or(())?;
            let authenticated_import = contribution.effects().imports().iter().any(|import| {
                symbols.imports().get(*import).and_then(|row| row.module()) == Some(symbol.module())
            });
            if *source_id != input.source_id
                || !valid_imported_provenance(
                    entry,
                    symbol,
                    input.source_id,
                    use_range,
                    contribution_range,
                    authenticated_import,
                )
            {
                return Err(());
            }
        }
        ContributionKind::Summary { .. } | ContributionKind::Builtin { .. } | _ => {
            return Err(());
        }
    }
    Ok(())
}

fn valid_imported_provenance(
    entry: &SymbolEntry,
    symbol: &SymbolId,
    source_id: SourceId,
    use_range: SourceRange,
    contribution_range: SourceRange,
    authenticated_import: bool,
) -> bool {
    entry.visibility() == Visibility::Public
        && matches!(
            entry.export_status(),
            ExportStatus::Exported | ExportStatus::ReExported
        )
        && valid_range(source_id, contribution_range)
        && contribution_range.end <= use_range.start
        && entry.origin().source_id() == source_id
        && entry.origin().module_id() == symbol.module()
        && authenticated_import
}

fn grouped_rows<T, FOwner, FOrdinal, FError>(
    owner_count: usize,
    rows: &[T],
    owner: FOwner,
    ordinal: FOrdinal,
    error: FError,
) -> Result<Vec<Vec<usize>>, SourceAtomicFormulaError>
where
    FOwner: Fn(&T) -> usize,
    FOrdinal: Fn(&T) -> usize,
    FError: Fn(usize) -> SourceAtomicFormulaError,
{
    let mut groups = vec![Vec::new(); owner_count];
    let mut previous_owner = 0;
    for (index, row) in rows.iter().enumerate() {
        let owner_index = owner(row);
        let Some(group) = groups.get_mut(owner_index) else {
            return Err(error(index));
        };
        if (index > 0 && owner_index < previous_owner) || ordinal(row) != group.len() {
            return Err(error(index));
        }
        group.push(index);
        previous_owner = owner_index;
    }
    Ok(groups)
}

fn dependency_fingerprint(used: bool, fingerprint: Option<String>) -> Option<String> {
    used.then(|| fingerprint.expect("validated dependency"))
}

fn validate_arena_site(
    site: &TypedSiteRef,
    range: SourceRange,
    kind: &str,
    recovery: SourceAtomicFormulaRecovery,
    arena: &TypedArena,
) -> Result<(), ()> {
    let TypedSiteRef::Node(node) = site else {
        return Err(());
    };
    let row = arena.node(*node).ok_or(())?;
    if row.anchor != SourceAnchor::Range(range)
        || row.kind.as_str() != kind
        || !recovery_matches(recovery, row.recovery)
    {
        return Err(());
    }
    Ok(())
}

fn recovery_matches(
    recovery: SourceAtomicFormulaRecovery,
    node_recovery: NodeRecoveryState,
) -> bool {
    match recovery {
        SourceAtomicFormulaRecovery::Normal => node_recovery == NodeRecoveryState::Normal,
        SourceAtomicFormulaRecovery::Degraded => matches!(
            node_recovery,
            NodeRecoveryState::Recovered | NodeRecoveryState::Degraded
        ),
    }
}

fn valid_range(source_id: SourceId, range: SourceRange) -> bool {
    range.source_id == source_id && range.start < range.end
}

fn range_contains(parent: SourceRange, child: SourceRange) -> bool {
    parent.source_id == child.source_id && parent.start <= child.start && child.end <= parent.end
}

fn strictly_contains(parent: SourceRange, child: SourceRange) -> bool {
    parent.source_id == child.source_id && parent.start < child.start && child.end < parent.end
}

fn properly_contains(parent: SourceRange, child: SourceRange) -> bool {
    range_contains(parent, child) && parent != child
}

fn ranges_overlap(left: SourceRange, right: SourceRange) -> bool {
    left.source_id == right.source_id && left.start < right.end && right.start < left.end
}

fn source_range(anchor: &SourceAnchor) -> Option<SourceRange> {
    match anchor {
        SourceAnchor::Range(range) => Some(*range),
        SourceAnchor::Point { .. } | SourceAnchor::Generated(_) | _ => None,
    }
}

fn canonical_spelling(spelling: &str) -> bool {
    !spelling.is_empty()
        && spelling.trim() == spelling
        && !spelling.contains("  ")
        && !spelling
            .chars()
            .any(|character| character.is_whitespace() && character != ' ')
}

fn identifier_spelling(spelling: &str) -> bool {
    let mut chars = spelling.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn formula_node_key(kind: SourceAtomicFormulaKind) -> &'static str {
    match kind {
        SourceAtomicFormulaKind::PredicateApplication => "source.formula.atomic.predicate",
        SourceAtomicFormulaKind::Equality => "source.formula.atomic.equality",
        SourceAtomicFormulaKind::Inequality => "source.formula.atomic.inequality",
        SourceAtomicFormulaKind::Membership => "source.formula.atomic.membership",
        SourceAtomicFormulaKind::TypeAssertion => "source.formula.atomic.type-assertion",
        SourceAtomicFormulaKind::AttributeAssertion => "source.formula.atomic.attribute-assertion",
    }
}

fn formula_kind_key(kind: SourceAtomicFormulaKind) -> &'static str {
    match kind {
        SourceAtomicFormulaKind::PredicateApplication => "predicate",
        SourceAtomicFormulaKind::Equality => "equality",
        SourceAtomicFormulaKind::Inequality => "inequality",
        SourceAtomicFormulaKind::Membership => "membership",
        SourceAtomicFormulaKind::TypeAssertion => "type-assertion",
        SourceAtomicFormulaKind::AttributeAssertion => "attribute-assertion",
    }
}

fn recovery_key(recovery: SourceAtomicFormulaRecovery) -> &'static str {
    match recovery {
        SourceAtomicFormulaRecovery::Normal => "normal",
        SourceAtomicFormulaRecovery::Degraded => "degraded",
    }
}

fn type_head_spelling(head: SourceAssertionTypeHead) -> &'static str {
    match head {
        SourceAssertionTypeHead::BuiltinSet => "set",
        SourceAssertionTypeHead::BuiltinObject => "object",
    }
}

fn type_head_key(head: SourceAssertionTypeHead) -> &'static str {
    type_head_spelling(head)
}

fn edge_role_key(role: SourceAtomicEdgeRole) -> &'static str {
    match role {
        SourceAtomicEdgeRole::PredicateLeftArgument => "predicate-left",
        SourceAtomicEdgeRole::PredicateChainBoundary => "predicate-chain-boundary",
        SourceAtomicEdgeRole::PredicateRightArgument => "predicate-right",
        SourceAtomicEdgeRole::BuiltinLeftOperand => "builtin-left",
        SourceAtomicEdgeRole::BuiltinRightOperand => "builtin-right",
        SourceAtomicEdgeRole::AssertionSubject => "assertion-subject",
    }
}

fn request_kind_key(kind: SourceAtomicRequestKind) -> &'static str {
    match kind {
        SourceAtomicRequestKind::OperandExpectedType => "operand-expected-type",
        SourceAtomicRequestKind::PredicateCandidateSignature => "predicate-candidate-signature",
        SourceAtomicRequestKind::TypeAssertionReachability => "type-assertion-reachability",
        SourceAtomicRequestKind::AttributeAdmissibility => "attribute-admissibility",
    }
}

fn write_target(output: &mut String, target: SourceAtomicTermTarget) {
    match target {
        SourceAtomicTermTarget::Primary(id) => {
            let _ = write!(output, "primary:{}", id.index());
        }
        SourceAtomicTermTarget::Application(id) => {
            let _ = write!(output, "application:{}", id.index());
        }
        SourceAtomicTermTarget::Structure(id) => {
            let _ = write!(output, "structure:{}", id.index());
        }
        SourceAtomicTermTarget::SetTerm(id) => {
            let _ = write!(output, "set-term:{}", id.index());
        }
    }
}

fn write_polarity(output: &mut String, polarity: &SourceAssertionAttributePolarityInput) {
    match polarity {
        SourceAssertionAttributePolarityInput::Positive => output.push_str("positive"),
        SourceAssertionAttributePolarityInput::Negative {
            non_site,
            non_range,
            non_spelling,
            non_recovery,
        } => {
            let _ = write!(
                output,
                "negative(site={} range={}..{} spelling={:?} recovery={})",
                non_site.node().index(),
                non_range.start,
                non_range.end,
                non_spelling,
                recovery_key(*non_recovery),
            );
        }
    }
}

fn write_predicate_segment_polarity(
    output: &mut String,
    polarity: &SourcePredicateSegmentPolarityInput,
) {
    match polarity {
        SourcePredicateSegmentPolarityInput::Positive => output.push_str("positive"),
        SourcePredicateSegmentPolarityInput::Negative {
            verb_site,
            verb_range,
            verb_spelling,
            verb_recovery,
            not_site,
            not_range,
            not_spelling,
            not_recovery,
        } => {
            let _ = write!(
                output,
                "negative(verb_site={} verb_range={}..{} verb_spelling={:?} verb_recovery={} not_site={} not_range={}..{} not_spelling={:?} not_recovery={})",
                verb_site.node().index(),
                verb_range.start,
                verb_range.end,
                verb_spelling,
                recovery_key(*verb_recovery),
                not_site.node().index(),
                not_range.start,
                not_range.end,
                not_spelling,
                recovery_key(*not_recovery),
            );
        }
    }
}

fn write_optional_id(output: &mut String, id: Option<usize>) {
    if let Some(id) = id {
        let _ = write!(output, "{id}");
    } else {
        output.push('-');
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        binding_env::{
            BindingContextDraft, BindingContextLayer, BindingContextOwner, BindingContextRecovery,
            BindingContextTable, BindingDiagnosticTable, BindingEnvParts, BindingTable,
        },
        source_application::{
            SourceFunctorApplicationForm, SourceFunctorApplicationHandoffInput,
            SourceFunctorApplicationId, SourceFunctorApplicationInput,
            SourceFunctorApplicationKind, SourceFunctorApplicationProducer,
            SourceFunctorApplicationRecovery, SourceFunctorArgumentInput,
            SourceFunctorArgumentTarget, SourceFunctorCandidateId, SourceFunctorCandidateInput,
            SourceFunctorHeadSite, SourceFunctorTypeRequestInput, SourceFunctorTypeRequestKind,
        },
        source_set_term::{
            SourceSetConditionInput, SourceSetEdgeInput, SourceSetEdgeRole, SourceSetGeneratorId,
            SourceSetGeneratorInput, SourceSetRequestInput, SourceSetRequestKind, SourceSetTarget,
            SourceSetTermError, SourceSetTermHandoffInput, SourceSetTermId, SourceSetTermInput,
            SourceSetTermKind, SourceSetTermProducer, SourceSetTermRecovery, SourceSetTypeHead,
            SourceSetTypeOwner, SourceSetTypeSiteId, SourceSetTypeSiteInput,
        },
        source_structure::{
            SourceStructureEdgeInput, SourceStructureEdgeRole, SourceStructureHandoffInput,
            SourceStructureMemberId, SourceStructureMemberInput, SourceStructureMemberRole,
            SourceStructureProducer, SourceStructureRecovery, SourceStructureRequestInput,
            SourceStructureRequestKind, SourceStructureTarget, SourceStructureTermId,
            SourceStructureTermInput, SourceStructureTermKind,
        },
        source_term::{
            SourceNumericTypeRequestInput, SourcePrimaryTermHandoffInput, SourcePrimaryTermInput,
            SourcePrimaryTermKind, SourcePrimaryTermProducer, SourcePrimaryTermRecovery,
            SourcePrimaryTermRole,
        },
        typed_ast::{
            CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
            TypeFactTable, TypeTable, TypedAst, TypedAstError, TypedAstParts, TypedNode,
            TypedNodeId,
        },
    };
    use mizar_resolve::{
        env::{DefinitionShell, NamespacePath, SymbolEntry, SymbolEnvIndexes},
        resolved_ast::{FullyQualifiedName, LocalSymbolId, SemanticOrigin},
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator as _,
    };

    pub(crate) struct Fixture {
        pub(crate) source: SourceId,
        pub(crate) module: ModuleId,
        pub(crate) bindings: BindingEnv,
        pub(crate) symbols: SymbolEnv,
        pub(crate) primary: SourcePrimaryTermHandoff,
        pub(crate) arena: TypedArena,
        pub(crate) input: SourceAtomicFormulaHandoffInput,
    }

    fn source_id() -> SourceId {
        source_id_with_snapshot_byte("d6", 1)
    }

    fn source_id_with_snapshot_byte(snapshot_byte: &str, source_ordinal: usize) -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            snapshot_byte.repeat(32)
        ))
        .expect("snapshot");
        let allocator = InMemorySessionIdAllocator::new();
        (0..source_ordinal)
            .map(|_| allocator.next_source_id(snapshot).expect("source"))
            .last()
            .expect("positive source ordinal")
    }

    fn module() -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("atomic.fixture"))
    }

    fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn node(index: usize) -> TypedSiteRef {
        TypedSiteRef::Node(TypedNodeId::new(index))
    }

    fn bindings(source: SourceId, module: &ModuleId) -> BindingEnv {
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: Vec::new(),
            visible_bindings: Vec::new(),
            recovery: BindingContextRecovery::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings: BindingTable::new(),
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("bindings")
    }

    fn bindings_with_two_contexts(source: SourceId, module: &ModuleId) -> BindingEnv {
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: Vec::new(),
            visible_bindings: Vec::new(),
            recovery: BindingContextRecovery::Normal,
        });
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("atomic-cross-context".to_owned()),
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Expression,
            lexical_scope: None,
            bindings: Vec::new(),
            visible_bindings: Vec::new(),
            recovery: BindingContextRecovery::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings: BindingTable::new(),
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("bindings with two contexts")
    }

    fn primary_handoff(
        source: SourceId,
        module: &ModuleId,
        bindings: &BindingEnv,
        arena: &TypedArena,
        occurrences: &[(usize, usize, usize, &str)],
    ) -> SourcePrimaryTermHandoff {
        SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: occurrences
                    .iter()
                    .enumerate()
                    .map(
                        |(ordinal, (site, start, end, spelling))| SourcePrimaryTermInput {
                            site: node(*site),
                            source_range: range(source, *start, *end),
                            source_ordinal: ordinal,
                            context: BindingContextId::new(0),
                            recovery: SourcePrimaryTermRecovery::Normal,
                            spelling: (*spelling).to_owned(),
                            kind: SourcePrimaryTermKind::Numeral,
                            role: SourcePrimaryTermRole::Value,
                            parent: None,
                        },
                    )
                    .collect(),
                references: Vec::new(),
                numeric_type_requests: occurrences
                    .iter()
                    .enumerate()
                    .map(
                        |(ordinal, (site, start, end, spelling))| SourceNumericTypeRequestInput {
                            term: SourcePrimaryTermId::new(ordinal),
                            owner: node(*site),
                            source_range: range(source, *start, *end),
                            spelling: (*spelling).to_owned(),
                            request_ordinal: ordinal,
                        },
                    )
                    .collect(),
            },
            bindings,
            arena,
        )
        .expect("primary handoff")
    }

    #[allow(clippy::too_many_arguments)] // Rationale: the synthetic type-assertion fixture exposes each frozen source anchor independently.
    fn type_assertion_input(
        source: SourceId,
        module: &ModuleId,
        formula_site: usize,
        formula_range: (usize, usize),
        type_site: usize,
        type_head_site: usize,
        type_range: (usize, usize),
        subject_spelling: &str,
        target: SourceAtomicTermTarget,
    ) -> SourceAtomicFormulaHandoffInput {
        SourceAtomicFormulaHandoffInput {
            source_id: source,
            module_id: module.clone(),
            formulas: vec![SourceAtomicFormulaInput {
                site: node(formula_site),
                source_range: range(source, formula_range.0, formula_range.1),
                source_ordinal: 0,
                context: BindingContextId::new(0),
                recovery: SourceAtomicFormulaRecovery::Normal,
                spelling: format!("{subject_spelling} is set"),
                kind: SourceAtomicFormulaKind::TypeAssertion,
            }],
            wrappers: Vec::new(),
            predicate_segments: Vec::new(),
            predicate_heads: Vec::new(),
            candidates: Vec::new(),
            type_sites: vec![SourceAssertionTypeSiteInput {
                formula: SourceAtomicFormulaId::new(0),
                site: node(type_site),
                source_range: range(source, type_range.0, type_range.1),
                spelling: "set".to_owned(),
                head_site: node(type_head_site),
                head_range: range(source, type_range.0, type_range.1),
                head_spelling: "set".to_owned(),
                context: BindingContextId::new(0),
                recovery: SourceAtomicFormulaRecovery::Normal,
                head: SourceAssertionTypeHead::BuiltinSet,
            }],
            attributes: Vec::new(),
            edges: vec![SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                role: SourceAtomicEdgeRole::AssertionSubject,
                target,
            }],
            requests: vec![SourceAtomicRequestInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                kind: SourceAtomicRequestKind::TypeAssertionReachability,
                edge: None,
                candidate: None,
                type_site: Some(SourceAssertionTypeSiteId::new(0)),
                attribute: None,
            }],
        }
    }

    pub(crate) fn make_fixture(kind: SourceAtomicFormulaKind) -> Fixture {
        let source = source_id();
        let module = module();
        let bindings = bindings(source, &module);
        let term_count = usize::from(matches!(
            kind,
            SourceAtomicFormulaKind::TypeAssertion | SourceAtomicFormulaKind::AttributeAssertion
        )) + usize::from(!matches!(
            kind,
            SourceAtomicFormulaKind::TypeAssertion | SourceAtomicFormulaKind::AttributeAssertion
        )) * 2;
        let term_ranges = [(10, 11, "1"), (20, 21, "2")];
        let mut nodes = term_ranges
            .iter()
            .map(|(start, end, _)| {
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, *start, *end)),
                )
            })
            .collect::<Vec<_>>();
        nodes.push(TypedNode::new(
            formula_node_key(kind),
            SourceAnchor::Range(range(source, 5, 25)),
        ));
        nodes.push(TypedNode::new(
            "source.formula.atomic.asserted-type",
            SourceAnchor::Range(range(source, 20, 23)),
        ));
        nodes.push(TypedNode::new(
            "source.formula.atomic.asserted-type-head",
            SourceAnchor::Range(range(source, 20, 23)),
        ));
        nodes.push(TypedNode::new(
            "source.formula.atomic.parenthesized",
            SourceAnchor::Range(range(source, 3, 27)),
        ));
        let arena = TypedArena::try_new(None, nodes).expect("arena");
        let primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: term_ranges[..term_count]
                    .iter()
                    .enumerate()
                    .map(|(ordinal, (start, end, spelling))| SourcePrimaryTermInput {
                        site: node(ordinal),
                        source_range: range(source, *start, *end),
                        source_ordinal: ordinal,
                        context: BindingContextId::new(0),
                        recovery: SourcePrimaryTermRecovery::Normal,
                        spelling: (*spelling).to_owned(),
                        kind: SourcePrimaryTermKind::Numeral,
                        role: SourcePrimaryTermRole::Value,
                        parent: None,
                    })
                    .collect(),
                references: Vec::new(),
                numeric_type_requests: term_ranges[..term_count]
                    .iter()
                    .enumerate()
                    .map(
                        |(ordinal, (start, end, spelling))| SourceNumericTypeRequestInput {
                            term: SourcePrimaryTermId::new(ordinal),
                            owner: node(ordinal),
                            source_range: range(source, *start, *end),
                            spelling: (*spelling).to_owned(),
                            request_ordinal: ordinal,
                        },
                    )
                    .collect(),
            },
            &bindings,
            &arena,
        )
        .expect("primary");
        let (spelling, roles, requests) = match kind {
            SourceAtomicFormulaKind::Equality => (
                "1 = 2",
                vec![
                    SourceAtomicEdgeRole::BuiltinLeftOperand,
                    SourceAtomicEdgeRole::BuiltinRightOperand,
                ],
                vec![0, 1],
            ),
            SourceAtomicFormulaKind::Inequality => (
                "1 <> 2",
                vec![
                    SourceAtomicEdgeRole::BuiltinLeftOperand,
                    SourceAtomicEdgeRole::BuiltinRightOperand,
                ],
                vec![0, 1],
            ),
            SourceAtomicFormulaKind::Membership => (
                "1 in 2",
                vec![
                    SourceAtomicEdgeRole::BuiltinLeftOperand,
                    SourceAtomicEdgeRole::BuiltinRightOperand,
                ],
                vec![1],
            ),
            SourceAtomicFormulaKind::TypeAssertion => (
                "1 is set",
                vec![SourceAtomicEdgeRole::AssertionSubject],
                Vec::new(),
            ),
            SourceAtomicFormulaKind::PredicateApplication
            | SourceAtomicFormulaKind::AttributeAssertion => {
                unreachable!("specialized internal fixtures")
            }
        };
        let edges = roles
            .into_iter()
            .enumerate()
            .map(|(ordinal, role)| SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal,
                role,
                target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(ordinal)),
            })
            .collect::<Vec<_>>();
        let type_sites = (kind == SourceAtomicFormulaKind::TypeAssertion)
            .then(|| SourceAssertionTypeSiteInput {
                formula: SourceAtomicFormulaId::new(0),
                site: node(3),
                source_range: range(source, 20, 23),
                spelling: "set".to_owned(),
                head_site: node(4),
                head_range: range(source, 20, 23),
                head_spelling: "set".to_owned(),
                context: BindingContextId::new(0),
                recovery: SourceAtomicFormulaRecovery::Normal,
                head: SourceAssertionTypeHead::BuiltinSet,
            })
            .into_iter()
            .collect::<Vec<_>>();
        let mut request_rows = requests
            .into_iter()
            .enumerate()
            .map(|(ordinal, edge)| SourceAtomicRequestInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal,
                kind: SourceAtomicRequestKind::OperandExpectedType,
                edge: Some(SourceAtomicEdgeId::new(edge)),
                candidate: None,
                type_site: None,
                attribute: None,
            })
            .collect::<Vec<_>>();
        if kind == SourceAtomicFormulaKind::TypeAssertion {
            request_rows.push(SourceAtomicRequestInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                kind: SourceAtomicRequestKind::TypeAssertionReachability,
                edge: None,
                candidate: None,
                type_site: Some(SourceAssertionTypeSiteId::new(0)),
                attribute: None,
            });
        }
        Fixture {
            source,
            module: module.clone(),
            bindings,
            symbols: SymbolEnv::new(module.clone(), SymbolEnvIndexes::default()),
            primary,
            arena,
            input: SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module,
                formulas: vec![SourceAtomicFormulaInput {
                    site: node(2),
                    source_range: range(source, 5, 25),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceAtomicFormulaRecovery::Normal,
                    spelling: spelling.to_owned(),
                    kind,
                }],
                wrappers: Vec::new(),
                predicate_segments: Vec::new(),
                predicate_heads: Vec::new(),
                candidates: Vec::new(),
                type_sites,
                attributes: Vec::new(),
                edges,
                requests: request_rows,
            },
        }
    }

    fn build(fixture: &Fixture) -> Result<SourceAtomicFormulaHandoff, SourceAtomicFormulaError> {
        SourceAtomicFormulaProducer::build(
            fixture.input.clone(),
            &fixture.bindings,
            &fixture.symbols,
            &fixture.primary,
            None,
            None,
            None,
            &fixture.arena,
        )
    }

    fn assert_build_rejects(fixture: &Fixture) {
        assert!(build(fixture).is_err(), "corruption unexpectedly published");
    }

    fn typed_ast(fixture: &Fixture) -> TypedAst {
        typed_ast_with_primary(
            fixture.source,
            &fixture.module,
            &fixture.arena,
            fixture.primary.clone(),
        )
    }

    fn typed_ast_with_primary(
        source: SourceId,
        module: &ModuleId,
        arena: &TypedArena,
        primary: SourcePrimaryTermHandoff,
    ) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: source,
            module_id: module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("typed AST")
        .with_source_term(primary)
        .expect("primary install")
    }

    pub(crate) struct ConditionContainerFixture {
        pub(crate) source: SourceId,
        pub(crate) module: ModuleId,
        pub(crate) bindings: BindingEnv,
        pub(crate) symbols: SymbolEnv,
        pub(crate) primary: SourcePrimaryTermHandoff,
        pub(crate) application: SourceFunctorApplicationHandoff,
        pub(crate) arena: TypedArena,
        pub(crate) set_input: SourceSetTermHandoffInput,
        pub(crate) atomic_input: SourceAtomicFormulaHandoffInput,
    }

    impl ConditionContainerFixture {
        pub(crate) fn build_set(
            &self,
            input: SourceSetTermHandoffInput,
        ) -> Result<SourceSetTermHandoff, SourceSetTermError> {
            SourceSetTermProducer::build(
                input,
                &self.bindings,
                &self.primary,
                Some(&self.application),
                None,
                &self.arena,
            )
        }

        pub(crate) fn build_atomic(
            &self,
            input: SourceAtomicFormulaHandoffInput,
            set_terms: Option<&SourceSetTermHandoff>,
        ) -> Result<SourceAtomicFormulaHandoff, SourceAtomicFormulaError> {
            SourceAtomicFormulaProducer::build(
                input,
                &self.bindings,
                &self.symbols,
                &self.primary,
                Some(&self.application),
                None,
                set_terms,
                &self.arena,
            )
        }

        pub(crate) fn typed_ast(&self) -> TypedAst {
            typed_ast_with_primary(self.source, &self.module, &self.arena, self.primary.clone())
                .with_source_application(self.application.clone())
                .expect("condition mapper application install")
        }
    }

    #[derive(Clone, Copy)]
    struct ConditionContainerOptions {
        formula_kind: SourceAtomicFormulaKind,
        formula_range: (usize, usize),
        formula_spelling: &'static str,
        formula_context: BindingContextId,
        formula_recovery: SourceAtomicFormulaRecovery,
        operand_ranges: [(usize, usize); 2],
        direct_condition_child: bool,
        snapshot_byte: &'static str,
        source_ordinal: usize,
    }

    impl ConditionContainerOptions {
        const fn exact() -> Self {
            Self {
                formula_kind: SourceAtomicFormulaKind::Equality,
                formula_range: (177, 182),
                formula_spelling: "3 = 4",
                formula_context: BindingContextId::new(0),
                formula_recovery: SourceAtomicFormulaRecovery::Normal,
                operand_ranges: [(177, 178), (181, 182)],
                direct_condition_child: true,
                snapshot_byte: "d6",
                source_ordinal: 1,
            }
        }
    }

    fn condition_container_fixture(
        options: ConditionContainerOptions,
    ) -> ConditionContainerFixture {
        let source = source_id_with_snapshot_byte(options.snapshot_byte, options.source_ordinal);
        let module = module();
        let bindings = bindings_with_two_contexts(source, &module);
        let formula_recovery = match options.formula_recovery {
            SourceAtomicFormulaRecovery::Normal => NodeRecoveryState::Normal,
            SourceAtomicFormulaRecovery::Degraded => NodeRecoveryState::Degraded,
        };
        let condition_children = options
            .direct_condition_child
            .then(|| node(10).node())
            .into_iter()
            .collect();
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 141, 142)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 146, 147)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(
                        source,
                        options.operand_ranges[0].0,
                        options.operand_ranges[0].1,
                    )),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(
                        source,
                        options.operand_ranges[1].0,
                        options.operand_ranges[1].1,
                    )),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(source, 143, 145)),
                ),
                TypedNode::new(
                    "source.term.functor-application.symbolic",
                    SourceAnchor::Range(range(source, 141, 147)),
                )
                .with_children(vec![
                    node(0).node(),
                    node(4).node(),
                    node(1).node(),
                ]),
                TypedNode::new(
                    "source.term.set.comprehension-generator",
                    SourceAnchor::Range(range(source, 154, 167)),
                ),
                TypedNode::new(
                    "source.term.set.target-type",
                    SourceAnchor::Range(range(source, 171, 174)),
                ),
                TypedNode::new(
                    "source.term.set.target-type-head",
                    SourceAnchor::Range(range(source, 171, 174)),
                ),
                TypedNode::new(
                    "source.term.set.comprehension-condition-colon",
                    SourceAnchor::Range(range(source, 175, 176)),
                ),
                TypedNode::new(
                    formula_node_key(options.formula_kind),
                    SourceAnchor::Range(range(
                        source,
                        options.formula_range.0,
                        options.formula_range.1,
                    )),
                )
                .with_children(vec![node(2).node(), node(3).node()])
                .with_recovery(formula_recovery),
                TypedNode::new(
                    "source.term.set.comprehension-condition",
                    SourceAnchor::Range(range(source, 177, 182)),
                )
                .with_children(condition_children),
                TypedNode::new(
                    "source.term.set.comprehension",
                    SourceAnchor::Range(range(source, 139, 184)),
                )
                .with_children(vec![
                    node(5).node(),
                    node(6).node(),
                    node(9).node(),
                    node(11).node(),
                ]),
            ],
        )
        .expect("condition-container arena");
        let primary_occurrences = [
            (0, 141, 142, "1", BindingContextId::new(0)),
            (1, 146, 147, "2", BindingContextId::new(0)),
            (
                2,
                options.operand_ranges[0].0,
                options.operand_ranges[0].1,
                "3",
                options.formula_context,
            ),
            (
                3,
                options.operand_ranges[1].0,
                options.operand_ranges[1].1,
                "4",
                options.formula_context,
            ),
        ];
        let primary = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: primary_occurrences
                    .iter()
                    .enumerate()
                    .map(|(ordinal, (site, start, end, spelling, context))| {
                        SourcePrimaryTermInput {
                            site: node(*site),
                            source_range: range(source, *start, *end),
                            source_ordinal: ordinal,
                            context: *context,
                            recovery: SourcePrimaryTermRecovery::Normal,
                            spelling: (*spelling).to_owned(),
                            kind: SourcePrimaryTermKind::Numeral,
                            role: SourcePrimaryTermRole::Value,
                            parent: None,
                        }
                    })
                    .collect(),
                references: Vec::new(),
                numeric_type_requests: primary_occurrences
                    .iter()
                    .enumerate()
                    .map(|(ordinal, (site, start, end, spelling, _))| {
                        SourceNumericTypeRequestInput {
                            term: SourcePrimaryTermId::new(ordinal),
                            owner: node(*site),
                            source_range: range(source, *start, *end),
                            spelling: (*spelling).to_owned(),
                            request_ordinal: ordinal,
                        }
                    })
                    .collect(),
            },
            &bindings,
            &arena,
        )
        .expect("condition-container primary handoff");

        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 100, 110)),
        );
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new("condition-plus"),
            FullyQualifiedName::new(format!(
                "{}::functor/condition-plus",
                module.path().as_str()
            )),
        );
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 101, 103)),
            vec![0],
        );
        indexes.symbols.insert(SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Functor,
            NamespacePath::new(module.path().as_str()),
            "++",
            origin.clone(),
            contribution,
        ));
        indexes
            .contributions
            .add_symbol(contribution, symbol.clone());
        let definition = indexes.definitions.insert(DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Functor,
            origin,
            contribution,
        ));
        indexes
            .contributions
            .add_definition(contribution, definition);
        let symbols = SymbolEnv::new(module.clone(), indexes);

        let application = SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: source,
                module_id: module.clone(),
                applications: vec![SourceFunctorApplicationInput {
                    site: node(5),
                    source_range: range(source, 141, 147),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceFunctorApplicationRecovery::Normal,
                    spelling: "1 ++ 2".to_owned(),
                    kind: SourceFunctorApplicationKind::Symbolic,
                    form: SourceFunctorApplicationForm::Infix,
                    head_ordinal: 1,
                    head: SourceFunctorHeadSite::Single {
                        site: node(4),
                        source_range: range(source, 143, 145),
                        spelling: "++".to_owned(),
                    },
                }],
                wrappers: Vec::new(),
                candidates: vec![SourceFunctorCandidateInput {
                    application: SourceFunctorApplicationId::new(0),
                    ordinal: 0,
                    symbol,
                    contribution,
                }],
                arguments: vec![
                    SourceFunctorArgumentInput {
                        application: SourceFunctorApplicationId::new(0),
                        ordinal: 0,
                        target: SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(0)),
                    },
                    SourceFunctorArgumentInput {
                        application: SourceFunctorApplicationId::new(0),
                        ordinal: 1,
                        target: SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(1)),
                    },
                ],
                type_requests: vec![
                    SourceFunctorTypeRequestInput {
                        application: SourceFunctorApplicationId::new(0),
                        candidate: Some(SourceFunctorCandidateId::new(0)),
                        request_ordinal: 0,
                        kind: SourceFunctorTypeRequestKind::CandidateSignature,
                    },
                    SourceFunctorTypeRequestInput {
                        application: SourceFunctorApplicationId::new(0),
                        candidate: None,
                        request_ordinal: 1,
                        kind: SourceFunctorTypeRequestKind::ApplicationResultType,
                    },
                ],
            },
            &symbols,
            &bindings,
            &primary,
            &arena,
        )
        .expect("condition mapper application");

        let set_input = SourceSetTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms: vec![SourceSetTermInput {
                site: node(12),
                source_range: range(source, 139, 184),
                source_ordinal: 0,
                context: BindingContextId::new(0),
                recovery: SourceSetTermRecovery::Normal,
                spelling: "{ 1 ++ 2 where candidate255c is set : 3 = 4 }".to_owned(),
                kind: SourceSetTermKind::Comprehension,
            }],
            wrappers: Vec::new(),
            generators: vec![SourceSetGeneratorInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                site: node(6),
                source_range: range(source, 154, 167),
                spelling: "candidate255c".to_owned(),
                context: BindingContextId::new(0),
                recovery: SourceSetTermRecovery::Normal,
                type_site: SourceSetTypeSiteId::new(0),
            }],
            type_sites: vec![SourceSetTypeSiteInput {
                owner: SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(0)),
                site: node(7),
                source_range: range(source, 171, 174),
                spelling: "set".to_owned(),
                head_site: node(8),
                head_range: range(source, 171, 174),
                head_spelling: "set".to_owned(),
                context: BindingContextId::new(0),
                recovery: SourceSetTermRecovery::Normal,
                head: SourceSetTypeHead::BuiltinSet,
            }],
            conditions: vec![SourceSetConditionInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                colon_site: node(9),
                colon_range: range(source, 175, 176),
                colon_spelling: ":".to_owned(),
                condition_site: node(11),
                source_range: range(source, 177, 182),
                spelling: "3 = 4".to_owned(),
                recovery: SourceSetTermRecovery::Normal,
            }],
            edges: vec![SourceSetEdgeInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                role: SourceSetEdgeRole::ComprehensionMapper,
                target: SourceSetTarget::Application(SourceFunctorApplicationId::new(0)),
            }],
            requests: vec![
                SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    kind: SourceSetRequestKind::GeneratorSethood,
                    generator: Some(SourceSetGeneratorId::new(0)),
                    type_site: Some(SourceSetTypeSiteId::new(0)),
                },
                SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 1,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                },
            ],
        };
        let atomic_input = SourceAtomicFormulaHandoffInput {
            source_id: source,
            module_id: module.clone(),
            formulas: vec![SourceAtomicFormulaInput {
                site: node(10),
                source_range: range(source, options.formula_range.0, options.formula_range.1),
                source_ordinal: 0,
                context: options.formula_context,
                recovery: options.formula_recovery,
                spelling: options.formula_spelling.to_owned(),
                kind: options.formula_kind,
            }],
            wrappers: Vec::new(),
            predicate_segments: Vec::new(),
            predicate_heads: Vec::new(),
            candidates: Vec::new(),
            type_sites: Vec::new(),
            attributes: Vec::new(),
            edges: vec![
                SourceAtomicEdgeInput {
                    formula: SourceAtomicFormulaId::new(0),
                    ordinal: 0,
                    role: SourceAtomicEdgeRole::BuiltinLeftOperand,
                    target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(2)),
                },
                SourceAtomicEdgeInput {
                    formula: SourceAtomicFormulaId::new(0),
                    ordinal: 1,
                    role: SourceAtomicEdgeRole::BuiltinRightOperand,
                    target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(3)),
                },
            ],
            requests: vec![
                SourceAtomicRequestInput {
                    formula: SourceAtomicFormulaId::new(0),
                    ordinal: 0,
                    kind: SourceAtomicRequestKind::OperandExpectedType,
                    edge: Some(SourceAtomicEdgeId::new(0)),
                    candidate: None,
                    type_site: None,
                    attribute: None,
                },
                SourceAtomicRequestInput {
                    formula: SourceAtomicFormulaId::new(0),
                    ordinal: 1,
                    kind: SourceAtomicRequestKind::OperandExpectedType,
                    edge: Some(SourceAtomicEdgeId::new(1)),
                    candidate: None,
                    type_site: None,
                    attribute: None,
                },
            ]
            .into_iter()
            .enumerate()
            .filter_map(|(index, mut request)| {
                if options.formula_kind == SourceAtomicFormulaKind::Membership && index == 0 {
                    None
                } else {
                    if options.formula_kind == SourceAtomicFormulaKind::Membership {
                        request.ordinal = 0;
                    }
                    Some(request)
                }
            })
            .collect(),
        };
        ConditionContainerFixture {
            source,
            module,
            bindings,
            symbols,
            primary,
            application,
            arena,
            set_input,
            atomic_input,
        }
    }

    pub(crate) fn exact_condition_container_fixture() -> ConditionContainerFixture {
        condition_container_fixture(ConditionContainerOptions::exact())
    }

    fn predicate_fixture(
        head_range: (usize, usize),
        left_arity: usize,
        right_arity: usize,
        spelling: &str,
    ) -> Fixture {
        let mut fixture = make_fixture(SourceAtomicFormulaKind::Equality);
        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        nodes[2].kind = "source.formula.atomic.predicate".into();
        nodes.push(TypedNode::new(
            "source.formula.atomic.predicate-head",
            SourceAnchor::Range(range(fixture.source, head_range.0, head_range.1)),
        ));
        fixture.arena = TypedArena::try_new(None, nodes).expect("predicate arena");
        fixture.input.formulas[0].kind = SourceAtomicFormulaKind::PredicateApplication;
        fixture.input.formulas[0].spelling = spelling.to_owned();
        fixture.input.predicate_heads = vec![SourcePredicateHeadInput {
            formula: SourceAtomicFormulaId::new(0),
            site: node(6),
            source_range: range(fixture.source, head_range.0, head_range.1),
            context: BindingContextId::new(0),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "divides".to_owned(),
            left_arity,
            right_arity,
        }];
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            fixture.module.clone(),
            ContributionKind::LocalSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 1, 2)),
        );
        fixture.input.candidates = vec![SourcePredicateCandidateInput {
            head: SourcePredicateHeadId::new(0),
            ordinal: 0,
            symbol: SymbolId::new(
                fixture.module.clone(),
                LocalSymbolId::new("divides/test"),
                FullyQualifiedName::new("atomic.fixture::divides"),
            ),
            contribution,
        }];
        fixture.input.edges = (0..left_arity)
            .map(|ordinal| SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal,
                role: SourceAtomicEdgeRole::PredicateLeftArgument,
                target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(ordinal)),
            })
            .chain((0..right_arity).map(|right| SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: left_arity + right,
                role: SourceAtomicEdgeRole::PredicateRightArgument,
                target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                    left_arity + right,
                )),
            }))
            .collect();
        fixture.input.requests = vec![SourceAtomicRequestInput {
            formula: SourceAtomicFormulaId::new(0),
            ordinal: 0,
            kind: SourceAtomicRequestKind::PredicateCandidateSignature,
            edge: None,
            candidate: Some(SourcePredicateCandidateId::new(0)),
            type_site: None,
            attribute: None,
        }];
        fixture
    }

    pub(crate) fn predicate_chain_fixture() -> Fixture {
        let source = source_id();
        let module = module();
        let bindings = bindings(source, &module);
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 75, 76)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 85, 86)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 104, 105)),
                ),
                TypedNode::new(
                    "source.formula.atomic.predicate",
                    SourceAnchor::Range(range(source, 75, 105)),
                ),
                TypedNode::new(
                    "source.formula.atomic.predicate-segment",
                    SourceAnchor::Range(range(source, 75, 86)),
                ),
                TypedNode::new(
                    "source.formula.atomic.predicate-segment",
                    SourceAnchor::Range(range(source, 87, 105)),
                ),
                TypedNode::new(
                    "source.formula.atomic.predicate-head",
                    SourceAnchor::Range(range(source, 77, 84)),
                ),
                TypedNode::new(
                    "source.formula.atomic.predicate-head",
                    SourceAnchor::Range(range(source, 96, 103)),
                ),
                TypedNode::new(
                    "source.formula.atomic.predicate-negation-verb",
                    SourceAnchor::Range(range(source, 87, 91)),
                ),
                TypedNode::new(
                    "source.formula.atomic.predicate-negation-not",
                    SourceAnchor::Range(range(source, 92, 95)),
                ),
            ],
        )
        .expect("predicate-chain arena");
        let primary = primary_handoff(
            source,
            &module,
            &bindings,
            &arena,
            &[(0, 75, 76, "1"), (1, 85, 86, "2"), (2, 104, 105, "3")],
        );
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 1, 2)),
        );
        let symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new("divides/chain"),
            FullyQualifiedName::new("atomic.fixture::divides"),
        );
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 3, 4)),
            vec![0],
        );
        indexes.symbols.insert(SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Predicate,
            NamespacePath::new(module.path().as_str()),
            "divides",
            origin.clone(),
            contribution,
        ));
        indexes
            .contributions
            .add_symbol(contribution, symbol.clone());
        let definition = indexes.definitions.insert(DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Predicate,
            origin,
            contribution,
        ));
        indexes
            .contributions
            .add_definition(contribution, definition);
        let formula = SourceAtomicFormulaId::new(0);
        Fixture {
            source,
            module: module.clone(),
            bindings,
            symbols: SymbolEnv::new(module.clone(), indexes),
            primary,
            arena,
            input: SourceAtomicFormulaHandoffInput {
                source_id: source,
                module_id: module,
                formulas: vec![SourceAtomicFormulaInput {
                    site: node(3),
                    source_range: range(source, 75, 105),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceAtomicFormulaRecovery::Normal,
                    spelling: "1 divides 2 does not divides 3".to_owned(),
                    kind: SourceAtomicFormulaKind::PredicateApplication,
                }],
                wrappers: Vec::new(),
                predicate_segments: vec![
                    SourcePredicateSegmentInput {
                        formula,
                        ordinal: 0,
                        site: node(4),
                        source_range: range(source, 75, 86),
                        context: BindingContextId::new(0),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "1 divides 2".to_owned(),
                        head: SourcePredicateHeadId::new(0),
                        polarity: SourcePredicateSegmentPolarityInput::Positive,
                        left_edge: SourceAtomicEdgeId::new(0),
                        right_edge: SourceAtomicEdgeId::new(1),
                    },
                    SourcePredicateSegmentInput {
                        formula,
                        ordinal: 1,
                        site: node(5),
                        source_range: range(source, 87, 105),
                        context: BindingContextId::new(0),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "does not divides 3".to_owned(),
                        head: SourcePredicateHeadId::new(1),
                        polarity: SourcePredicateSegmentPolarityInput::Negative {
                            verb_site: node(8),
                            verb_range: range(source, 87, 91),
                            verb_spelling: "does".to_owned(),
                            verb_recovery: SourceAtomicFormulaRecovery::Normal,
                            not_site: node(9),
                            not_range: range(source, 92, 95),
                            not_spelling: "not".to_owned(),
                            not_recovery: SourceAtomicFormulaRecovery::Normal,
                        },
                        left_edge: SourceAtomicEdgeId::new(1),
                        right_edge: SourceAtomicEdgeId::new(2),
                    },
                ],
                predicate_heads: vec![
                    SourcePredicateHeadInput {
                        formula,
                        site: node(6),
                        source_range: range(source, 77, 84),
                        context: BindingContextId::new(0),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "divides".to_owned(),
                        left_arity: 1,
                        right_arity: 1,
                    },
                    SourcePredicateHeadInput {
                        formula,
                        site: node(7),
                        source_range: range(source, 96, 103),
                        context: BindingContextId::new(0),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "divides".to_owned(),
                        left_arity: 1,
                        right_arity: 1,
                    },
                ],
                candidates: vec![
                    SourcePredicateCandidateInput {
                        head: SourcePredicateHeadId::new(0),
                        ordinal: 0,
                        symbol: symbol.clone(),
                        contribution,
                    },
                    SourcePredicateCandidateInput {
                        head: SourcePredicateHeadId::new(1),
                        ordinal: 0,
                        symbol,
                        contribution,
                    },
                ],
                type_sites: Vec::new(),
                attributes: Vec::new(),
                edges: vec![
                    SourceAtomicEdgeInput {
                        formula,
                        ordinal: 0,
                        role: SourceAtomicEdgeRole::PredicateLeftArgument,
                        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0)),
                    },
                    SourceAtomicEdgeInput {
                        formula,
                        ordinal: 1,
                        role: SourceAtomicEdgeRole::PredicateChainBoundary,
                        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(1)),
                    },
                    SourceAtomicEdgeInput {
                        formula,
                        ordinal: 2,
                        role: SourceAtomicEdgeRole::PredicateRightArgument,
                        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(2)),
                    },
                ],
                requests: vec![
                    SourceAtomicRequestInput {
                        formula,
                        ordinal: 0,
                        kind: SourceAtomicRequestKind::PredicateCandidateSignature,
                        edge: None,
                        candidate: Some(SourcePredicateCandidateId::new(0)),
                        type_site: None,
                        attribute: None,
                    },
                    SourceAtomicRequestInput {
                        formula,
                        ordinal: 1,
                        kind: SourceAtomicRequestKind::PredicateCandidateSignature,
                        edge: None,
                        candidate: Some(SourcePredicateCandidateId::new(1)),
                        type_site: None,
                        attribute: None,
                    },
                ],
            },
        }
    }

    pub(crate) fn single_predicate_on_predicate_chain_arena_fixture() -> Fixture {
        let mut fixture = predicate_chain_fixture();
        fixture.primary = primary_handoff(
            fixture.source,
            &fixture.module,
            &fixture.bindings,
            &fixture.arena,
            &[(0, 75, 76, "1"), (1, 85, 86, "2")],
        );
        fixture.input = SourceAtomicFormulaHandoffInput {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            formulas: vec![SourceAtomicFormulaInput {
                site: node(3),
                source_range: range(fixture.source, 75, 105),
                source_ordinal: 0,
                context: BindingContextId::new(0),
                recovery: SourceAtomicFormulaRecovery::Normal,
                spelling: "1 divides 2".to_owned(),
                kind: SourceAtomicFormulaKind::PredicateApplication,
            }],
            wrappers: Vec::new(),
            predicate_segments: Vec::new(),
            predicate_heads: fixture.input.predicate_heads[..1].to_vec(),
            candidates: fixture.input.candidates[..1].to_vec(),
            type_sites: Vec::new(),
            attributes: Vec::new(),
            edges: vec![
                SourceAtomicEdgeInput {
                    formula: SourceAtomicFormulaId::new(0),
                    ordinal: 0,
                    role: SourceAtomicEdgeRole::PredicateLeftArgument,
                    target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0)),
                },
                SourceAtomicEdgeInput {
                    formula: SourceAtomicFormulaId::new(0),
                    ordinal: 1,
                    role: SourceAtomicEdgeRole::PredicateRightArgument,
                    target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(1)),
                },
            ],
            requests: fixture.input.requests[..1].to_vec(),
        };
        fixture
    }

    fn install_local_symbols(
        fixture: &mut Fixture,
        spelling: &str,
        kind: SymbolKind,
        definition_kind: DefinitionKind,
        count: usize,
    ) {
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            fixture.module.clone(),
            ContributionKind::LocalSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 0, 4)),
        );
        let mut candidates = Vec::new();
        for ordinal in 0..count {
            let symbol = SymbolId::new(
                fixture.module.clone(),
                LocalSymbolId::new(format!("{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/{ordinal}",
                    fixture.module.path().as_str()
                )),
            );
            let origin = SemanticOrigin::new(
                fixture.source,
                fixture.module.clone(),
                SourceAnchor::Range(range(fixture.source, 1, 2)),
                vec![ordinal as u32],
            );
            indexes.symbols.insert(SymbolEntry::new(
                symbol.clone(),
                kind,
                NamespacePath::new(fixture.module.path().as_str()),
                spelling,
                origin.clone(),
                contribution,
            ));
            indexes
                .contributions
                .add_symbol(contribution, symbol.clone());
            let definition = indexes.definitions.insert(DefinitionShell::new(
                symbol.clone(),
                definition_kind,
                origin,
                contribution,
            ));
            indexes
                .contributions
                .add_definition(contribution, definition);
            candidates.push(SourcePredicateCandidateInput {
                head: SourcePredicateHeadId::new(0),
                ordinal,
                symbol,
                contribution,
            });
        }
        fixture.symbols = SymbolEnv::new(fixture.module.clone(), indexes);
        fixture.input.candidates = candidates;
        fixture.input.requests = (0..count)
            .map(|ordinal| SourceAtomicRequestInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal,
                kind: SourceAtomicRequestKind::PredicateCandidateSignature,
                edge: None,
                candidate: Some(SourcePredicateCandidateId::new(ordinal)),
                type_site: None,
                attribute: None,
            })
            .collect();
    }

    fn negative_attribute_fixture() -> Fixture {
        let mut fixture = make_fixture(SourceAtomicFormulaKind::TypeAssertion);
        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        nodes[2].kind = "source.formula.atomic.attribute-assertion".into();
        nodes.push(TypedNode::new(
            "source.formula.atomic.attribute",
            SourceAnchor::Range(range(fixture.source, 13, 22)),
        ));
        nodes.push(TypedNode::new(
            "source.formula.atomic.attribute-target",
            SourceAnchor::Range(range(fixture.source, 18, 22)),
        ));
        nodes.push(TypedNode::new(
            "source.formula.atomic.attribute-non",
            SourceAnchor::Range(range(fixture.source, 13, 16)),
        ));
        fixture.arena = TypedArena::try_new(None, nodes).expect("attribute arena");
        fixture.input.formulas[0].kind = SourceAtomicFormulaKind::AttributeAssertion;
        fixture.input.formulas[0].spelling = "1 is non empty".to_owned();
        fixture.input.type_sites.clear();
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            fixture.module.clone(),
            ContributionKind::LocalSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 1, 2)),
        );
        let symbol = SymbolId::new(
            fixture.module.clone(),
            LocalSymbolId::new("empty/test"),
            FullyQualifiedName::new("atomic.fixture::empty"),
        );
        let origin = SemanticOrigin::new(
            fixture.source,
            fixture.module.clone(),
            SourceAnchor::Range(range(fixture.source, 1, 2)),
            vec![0],
        );
        indexes.symbols.insert(SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Attribute,
            NamespacePath::new(fixture.module.path().as_str()),
            "empty",
            origin.clone(),
            contribution,
        ));
        indexes
            .contributions
            .add_symbol(contribution, symbol.clone());
        let definition = indexes.definitions.insert(DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Attribute,
            origin,
            contribution,
        ));
        indexes
            .contributions
            .add_definition(contribution, definition);
        fixture.symbols = SymbolEnv::new(fixture.module.clone(), indexes);
        fixture.input.attributes = vec![SourceAssertionAttributeInput {
            formula: SourceAtomicFormulaId::new(0),
            ordinal: 0,
            site: node(6),
            source_range: range(fixture.source, 13, 22),
            spelling: "non empty".to_owned(),
            target_site: node(7),
            target_range: range(fixture.source, 18, 22),
            target_spelling: "empty".to_owned(),
            context: BindingContextId::new(0),
            recovery: SourceAtomicFormulaRecovery::Normal,
            symbol,
            contribution,
            polarity: SourceAssertionAttributePolarityInput::Negative {
                non_site: node(8),
                non_range: range(fixture.source, 13, 16),
                non_spelling: "non".to_owned(),
                non_recovery: SourceAtomicFormulaRecovery::Normal,
            },
        }];
        fixture.input.requests = vec![SourceAtomicRequestInput {
            formula: SourceAtomicFormulaId::new(0),
            ordinal: 0,
            kind: SourceAtomicRequestKind::AttributeAdmissibility,
            edge: None,
            candidate: None,
            type_site: None,
            attribute: Some(SourceAssertionAttributeId::new(0)),
        }];
        fixture
    }

    #[test]
    fn builtin_formula_kinds_build_dense_deterministic_transactions() {
        for kind in [
            SourceAtomicFormulaKind::Equality,
            SourceAtomicFormulaKind::Inequality,
            SourceAtomicFormulaKind::Membership,
            SourceAtomicFormulaKind::TypeAssertion,
        ] {
            let fixture = make_fixture(kind);
            let first = build(&fixture).expect("valid handoff");
            let second = build(&fixture).expect("deterministic handoff");
            assert_eq!(first, second);
            assert_eq!(first.formulas().len(), 1);
            assert_eq!(first.edges().len(), fixture.input.edges.len());
            assert_eq!(first.requests().len(), fixture.input.requests.len());
            assert_eq!(
                first.primary_term_fingerprint(),
                fixture.primary.debug_text()
            );
            assert_eq!(first.application_fingerprint(), None);
            assert_eq!(first.structure_fingerprint(), None);
            assert_eq!(first.set_term_fingerprint(), None);
            first
                .validate_installation(
                    fixture.source,
                    &fixture.module,
                    &fixture.primary,
                    None,
                    None,
                    None,
                    &fixture.arena,
                )
                .expect("installable");
            assert_eq!(first.debug_text(), second.debug_text());
        }
    }

    #[test]
    fn every_direct_target_family_is_accepted_and_subtrees_are_excluded() {
        let primary_fixture = make_fixture(SourceAtomicFormulaKind::TypeAssertion);
        let primary_formula = build(&primary_fixture).expect("primary target");
        assert_eq!(
            primary_formula
                .edges()
                .get(SourceAtomicEdgeId::new(0))
                .unwrap()
                .target(),
            SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0))
        );

        let source = source_id();
        let application_module = module();
        let application_bindings = bindings(source, &application_module);
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 14, 15)),
                ),
                TypedNode::new(
                    "source.term.functor-application.inline",
                    SourceAnchor::Range(range(source, 10, 18)),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(source, 10, 11)),
                ),
                TypedNode::new(
                    "source.formula.atomic.type-assertion",
                    SourceAnchor::Range(range(source, 5, 25)),
                ),
                TypedNode::new(
                    "source.formula.atomic.asserted-type",
                    SourceAnchor::Range(range(source, 20, 23)),
                ),
                TypedNode::new(
                    "source.formula.atomic.asserted-type-head",
                    SourceAnchor::Range(range(source, 20, 23)),
                ),
            ],
        )
        .expect("application target arena");
        let primary = primary_handoff(
            source,
            &application_module,
            &application_bindings,
            &arena,
            &[(0, 14, 15, "1")],
        );
        let symbols = SymbolEnv::new(application_module.clone(), SymbolEnvIndexes::default());
        let applications = SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: source,
                module_id: application_module.clone(),
                applications: vec![SourceFunctorApplicationInput {
                    site: node(1),
                    source_range: range(source, 10, 18),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceFunctorApplicationRecovery::Normal,
                    spelling: "f ( 1 )".to_owned(),
                    kind: SourceFunctorApplicationKind::Inline,
                    form: SourceFunctorApplicationForm::Functional,
                    head_ordinal: 0,
                    head: SourceFunctorHeadSite::Single {
                        site: node(2),
                        source_range: range(source, 10, 11),
                        spelling: "f".to_owned(),
                    },
                }],
                wrappers: Vec::new(),
                candidates: Vec::new(),
                arguments: vec![SourceFunctorArgumentInput {
                    application: SourceFunctorApplicationId::new(0),
                    ordinal: 0,
                    target: SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(0)),
                }],
                type_requests: Vec::new(),
            },
            &symbols,
            &application_bindings,
            &primary,
            &arena,
        )
        .expect("application handoff");
        let application_input = type_assertion_input(
            source,
            &application_module,
            3,
            (5, 25),
            4,
            5,
            (20, 23),
            "f ( 1 )",
            SourceAtomicTermTarget::Application(SourceFunctorApplicationId::new(0)),
        );
        let application_formula = SourceAtomicFormulaProducer::build(
            application_input.clone(),
            &application_bindings,
            &symbols,
            &primary,
            Some(&applications),
            None,
            None,
            &arena,
        )
        .expect("application target");
        assert_eq!(
            application_formula.edges().len(),
            1,
            "the application-owned primary subtree must be excluded"
        );
        assert_eq!(
            application_formula
                .edges()
                .get(SourceAtomicEdgeId::new(0))
                .unwrap()
                .target(),
            SourceAtomicTermTarget::Application(SourceFunctorApplicationId::new(0))
        );
        assert!(matches!(
            SourceAtomicFormulaProducer::build(
                application_input.clone(),
                &application_bindings,
                &symbols,
                &primary,
                None,
                None,
                None,
                &arena,
            ),
            Err(SourceAtomicFormulaError::ApplicationDependencyMismatch)
        ));
        let mut application_subtree_smuggling = application_input;
        application_subtree_smuggling
            .edges
            .push(SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 1,
                role: SourceAtomicEdgeRole::AssertionSubject,
                target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0)),
            });
        assert!(
            SourceAtomicFormulaProducer::build(
                application_subtree_smuggling,
                &application_bindings,
                &symbols,
                &primary,
                Some(&applications),
                None,
                None,
                &arena,
            )
            .is_err()
        );
        assert!(application_formula.application_fingerprint().is_some());
        let application_ast =
            typed_ast_with_primary(source, &application_module, &arena, primary.clone());
        assert_eq!(
            application_ast
                .clone()
                .with_source_atomic_formula(application_formula.clone())
                .expect_err("targeted application must be installed first"),
            TypedAstError::InvalidSourceAtomicFormula
        );
        let substituted_applications = SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: source,
                module_id: application_module.clone(),
                applications: Vec::new(),
                wrappers: Vec::new(),
                candidates: Vec::new(),
                arguments: Vec::new(),
                type_requests: Vec::new(),
            },
            &symbols,
            &application_bindings,
            &primary,
            &arena,
        )
        .expect("empty substitute application handoff");
        assert_eq!(
            application_ast
                .clone()
                .with_source_application(substituted_applications)
                .expect("substitute application installs")
                .with_source_atomic_formula(application_formula.clone())
                .expect_err("recorded application fingerprint must reject substitution"),
            TypedAstError::InvalidSourceAtomicFormula
        );
        application_ast
            .with_source_application(applications.clone())
            .expect("application first")
            .with_source_atomic_formula(application_formula)
            .expect("atomic after targeted application");

        let source = source_id();
        let structure_module = module();
        let structure_bindings = bindings(source, &structure_module);
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 11, 12)),
                ),
                TypedNode::new(
                    "source.term.structure.selector",
                    SourceAnchor::Range(range(source, 10, 30)),
                ),
                TypedNode::new(
                    "source.term.structure.member.selector",
                    SourceAnchor::Range(range(source, 13, 20)),
                ),
                TypedNode::new(
                    "source.formula.atomic.type-assertion",
                    SourceAnchor::Range(range(source, 5, 40)),
                ),
                TypedNode::new(
                    "source.formula.atomic.asserted-type",
                    SourceAnchor::Range(range(source, 32, 35)),
                ),
                TypedNode::new(
                    "source.formula.atomic.asserted-type-head",
                    SourceAnchor::Range(range(source, 32, 35)),
                ),
            ],
        )
        .expect("structure target arena");
        let primary = primary_handoff(
            source,
            &structure_module,
            &structure_bindings,
            &arena,
            &[(0, 11, 12, "1")],
        );
        let symbols = SymbolEnv::new(structure_module.clone(), SymbolEnvIndexes::default());
        let structures = SourceStructureProducer::build(
            SourceStructureHandoffInput {
                source_id: source,
                module_id: structure_module.clone(),
                terms: vec![SourceStructureTermInput {
                    site: node(1),
                    source_range: range(source, 10, 30),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceStructureRecovery::Normal,
                    spelling: "1 . carrier".to_owned(),
                    kind: SourceStructureTermKind::SelectorAccess,
                }],
                wrappers: Vec::new(),
                roots: Vec::new(),
                members: vec![SourceStructureMemberInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    site: node(2),
                    source_range: range(source, 13, 20),
                    spelling: "carrier".to_owned(),
                    role: SourceStructureMemberRole::Selector,
                    parent: None,
                }],
                field_updates: Vec::new(),
                edges: vec![SourceStructureEdgeInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    role: SourceStructureEdgeRole::SelectorBase,
                    member: None,
                    target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(0)),
                }],
                requests: vec![
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 0,
                        kind: SourceStructureRequestKind::MemberIdentity,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 1,
                        kind: SourceStructureRequestKind::InheritancePath,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: None,
                        request_ordinal: 2,
                        kind: SourceStructureRequestKind::ResultType,
                    },
                ],
            },
            &symbols,
            &structure_bindings,
            &primary,
            None,
            &arena,
        )
        .expect("structure handoff");
        let structure_input = type_assertion_input(
            source,
            &structure_module,
            3,
            (5, 40),
            4,
            5,
            (32, 35),
            "1 . carrier",
            SourceAtomicTermTarget::Structure(SourceStructureTermId::new(0)),
        );
        let structure_formula = SourceAtomicFormulaProducer::build(
            structure_input.clone(),
            &structure_bindings,
            &symbols,
            &primary,
            None,
            Some(&structures),
            None,
            &arena,
        )
        .expect("structure target");
        assert_eq!(
            structure_formula.edges().len(),
            1,
            "the structure-owned primary subtree must be excluded"
        );
        assert_eq!(
            structure_formula
                .edges()
                .get(SourceAtomicEdgeId::new(0))
                .unwrap()
                .target(),
            SourceAtomicTermTarget::Structure(SourceStructureTermId::new(0))
        );
        assert!(matches!(
            SourceAtomicFormulaProducer::build(
                structure_input.clone(),
                &structure_bindings,
                &symbols,
                &primary,
                None,
                None,
                None,
                &arena,
            ),
            Err(SourceAtomicFormulaError::StructureDependencyMismatch)
        ));
        let mut structure_subtree_smuggling = structure_input;
        structure_subtree_smuggling
            .edges
            .push(SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 1,
                role: SourceAtomicEdgeRole::AssertionSubject,
                target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0)),
            });
        assert!(
            SourceAtomicFormulaProducer::build(
                structure_subtree_smuggling,
                &structure_bindings,
                &symbols,
                &primary,
                None,
                Some(&structures),
                None,
                &arena,
            )
            .is_err()
        );
        assert!(structure_formula.structure_fingerprint().is_some());
        let structure_ast =
            typed_ast_with_primary(source, &structure_module, &arena, primary.clone());
        assert_eq!(
            structure_ast
                .clone()
                .with_source_atomic_formula(structure_formula.clone())
                .expect_err("targeted structure must be installed first"),
            TypedAstError::InvalidSourceAtomicFormula
        );
        let substituted_structures = SourceStructureProducer::build(
            SourceStructureHandoffInput {
                source_id: source,
                module_id: structure_module.clone(),
                terms: Vec::new(),
                wrappers: Vec::new(),
                roots: Vec::new(),
                members: Vec::new(),
                field_updates: Vec::new(),
                edges: Vec::new(),
                requests: Vec::new(),
            },
            &symbols,
            &structure_bindings,
            &primary,
            None,
            &arena,
        )
        .expect("empty substitute structure handoff");
        assert_eq!(
            structure_ast
                .clone()
                .with_source_structure(substituted_structures)
                .expect("substitute structure installs")
                .with_source_atomic_formula(structure_formula.clone())
                .expect_err("recorded structure fingerprint must reject substitution"),
            TypedAstError::InvalidSourceAtomicFormula
        );
        structure_ast
            .with_source_structure(structures.clone())
            .expect("structure first")
            .with_source_atomic_formula(structure_formula)
            .expect("atomic after targeted structure");

        let source = source_id();
        let set_module = module();
        let set_bindings = bindings(source, &set_module);
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 12, 13)),
                ),
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 10, 16)),
                ),
                TypedNode::new(
                    "source.formula.atomic.type-assertion",
                    SourceAnchor::Range(range(source, 5, 25)),
                ),
                TypedNode::new(
                    "source.formula.atomic.asserted-type",
                    SourceAnchor::Range(range(source, 20, 23)),
                ),
                TypedNode::new(
                    "source.formula.atomic.asserted-type-head",
                    SourceAnchor::Range(range(source, 20, 23)),
                ),
            ],
        )
        .expect("set target arena");
        let primary = primary_handoff(
            source,
            &set_module,
            &set_bindings,
            &arena,
            &[(0, 12, 13, "1")],
        );
        let set_terms = SourceSetTermProducer::build(
            SourceSetTermHandoffInput {
                source_id: source,
                module_id: set_module.clone(),
                terms: vec![SourceSetTermInput {
                    site: node(1),
                    source_range: range(source, 10, 16),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    spelling: "{ 1 }".to_owned(),
                    kind: SourceSetTermKind::Enumeration,
                }],
                wrappers: Vec::new(),
                generators: Vec::new(),
                type_sites: Vec::new(),
                conditions: Vec::new(),
                edges: vec![SourceSetEdgeInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    role: SourceSetEdgeRole::EnumerationElement,
                    target: SourceSetTarget::Primary(SourcePrimaryTermId::new(0)),
                }],
                requests: vec![SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                }],
            },
            &set_bindings,
            &primary,
            None,
            None,
            &arena,
        )
        .expect("set handoff");
        let symbols = SymbolEnv::new(set_module.clone(), SymbolEnvIndexes::default());
        let set_input = type_assertion_input(
            source,
            &set_module,
            2,
            (5, 25),
            3,
            4,
            (20, 23),
            "{ 1 }",
            SourceAtomicTermTarget::SetTerm(SourceSetTermId::new(0)),
        );
        let set_formula = SourceAtomicFormulaProducer::build(
            set_input.clone(),
            &set_bindings,
            &symbols,
            &primary,
            None,
            None,
            Some(&set_terms),
            &arena,
        )
        .expect("set target");
        assert_eq!(
            set_formula.edges().len(),
            1,
            "the set-owned primary subtree must be excluded"
        );
        assert_eq!(
            set_formula
                .edges()
                .get(SourceAtomicEdgeId::new(0))
                .unwrap()
                .target(),
            SourceAtomicTermTarget::SetTerm(SourceSetTermId::new(0))
        );
        assert!(matches!(
            SourceAtomicFormulaProducer::build(
                set_input.clone(),
                &set_bindings,
                &symbols,
                &primary,
                None,
                None,
                None,
                &arena,
            ),
            Err(SourceAtomicFormulaError::SetTermDependencyMismatch)
        ));
        let mut set_subtree_smuggling = set_input;
        set_subtree_smuggling.edges.push(SourceAtomicEdgeInput {
            formula: SourceAtomicFormulaId::new(0),
            ordinal: 1,
            role: SourceAtomicEdgeRole::AssertionSubject,
            target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0)),
        });
        assert!(
            SourceAtomicFormulaProducer::build(
                set_subtree_smuggling,
                &set_bindings,
                &symbols,
                &primary,
                None,
                None,
                Some(&set_terms),
                &arena,
            )
            .is_err()
        );
        assert!(set_formula.set_term_fingerprint().is_some());
        let set_ast = typed_ast_with_primary(source, &set_module, &arena, primary.clone());
        assert_eq!(
            set_ast
                .clone()
                .with_source_atomic_formula(set_formula.clone())
                .expect_err("targeted set term must be installed first"),
            TypedAstError::InvalidSourceAtomicFormula
        );
        let substituted_set_terms = SourceSetTermProducer::build(
            SourceSetTermHandoffInput {
                source_id: source,
                module_id: set_module.clone(),
                terms: Vec::new(),
                wrappers: Vec::new(),
                generators: Vec::new(),
                type_sites: Vec::new(),
                conditions: Vec::new(),
                edges: Vec::new(),
                requests: Vec::new(),
            },
            &set_bindings,
            &primary,
            None,
            None,
            &arena,
        )
        .expect("empty substitute set handoff");
        assert_eq!(
            set_ast
                .clone()
                .with_source_set_term(substituted_set_terms)
                .expect("substitute set installs")
                .with_source_atomic_formula(set_formula.clone())
                .expect_err("recorded set fingerprint must reject substitution"),
            TypedAstError::InvalidSourceAtomicFormula
        );
        set_ast
            .with_source_set_term(set_terms.clone())
            .expect("set first")
            .with_source_atomic_formula(set_formula)
            .expect("atomic after targeted set term");
    }

    #[test]
    fn non_root_task253_254_255_rows_cannot_be_direct_formula_targets() {
        let source = source_id();
        let module = module();
        let bindings = bindings(source, &module);
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 34, 35)),
                ),
                TypedNode::new(
                    "source.term.functor-application.inline",
                    SourceAnchor::Range(range(source, 30, 38)),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(source, 30, 31)),
                ),
                TypedNode::new(
                    "source.term.structure.selector",
                    SourceAnchor::Range(range(source, 25, 48)),
                ),
                TypedNode::new(
                    "source.term.structure.member.selector",
                    SourceAnchor::Range(range(source, 42, 43)),
                ),
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 20, 52)),
                ),
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 10, 60)),
                ),
                TypedNode::new(
                    "source.formula.atomic.type-assertion",
                    SourceAnchor::Range(range(source, 5, 70)),
                ),
                TypedNode::new(
                    "source.formula.atomic.asserted-type",
                    SourceAnchor::Range(range(source, 65, 68)),
                ),
                TypedNode::new(
                    "source.formula.atomic.asserted-type-head",
                    SourceAnchor::Range(range(source, 65, 68)),
                ),
            ],
        )
        .expect("nested lower-family arena");
        let primary = primary_handoff(source, &module, &bindings, &arena, &[(0, 34, 35, "1")]);
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let applications = SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: source,
                module_id: module.clone(),
                applications: vec![SourceFunctorApplicationInput {
                    site: node(1),
                    source_range: range(source, 30, 38),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceFunctorApplicationRecovery::Normal,
                    spelling: "f ( 1 )".to_owned(),
                    kind: SourceFunctorApplicationKind::Inline,
                    form: SourceFunctorApplicationForm::Functional,
                    head_ordinal: 0,
                    head: SourceFunctorHeadSite::Single {
                        site: node(2),
                        source_range: range(source, 30, 31),
                        spelling: "f".to_owned(),
                    },
                }],
                wrappers: Vec::new(),
                candidates: Vec::new(),
                arguments: vec![SourceFunctorArgumentInput {
                    application: SourceFunctorApplicationId::new(0),
                    ordinal: 0,
                    target: SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(0)),
                }],
                type_requests: Vec::new(),
            },
            &symbols,
            &bindings,
            &primary,
            &arena,
        )
        .expect("nested application");
        let structures = SourceStructureProducer::build(
            SourceStructureHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![SourceStructureTermInput {
                    site: node(3),
                    source_range: range(source, 25, 48),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceStructureRecovery::Normal,
                    spelling: "f ( 1 ) . x".to_owned(),
                    kind: SourceStructureTermKind::SelectorAccess,
                }],
                wrappers: Vec::new(),
                roots: Vec::new(),
                members: vec![SourceStructureMemberInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    site: node(4),
                    source_range: range(source, 42, 43),
                    spelling: "x".to_owned(),
                    role: SourceStructureMemberRole::Selector,
                    parent: None,
                }],
                field_updates: Vec::new(),
                edges: vec![SourceStructureEdgeInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    role: SourceStructureEdgeRole::SelectorBase,
                    member: None,
                    target: SourceStructureTarget::Application(SourceFunctorApplicationId::new(0)),
                }],
                requests: vec![
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 0,
                        kind: SourceStructureRequestKind::MemberIdentity,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 1,
                        kind: SourceStructureRequestKind::InheritancePath,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: None,
                        request_ordinal: 2,
                        kind: SourceStructureRequestKind::ResultType,
                    },
                ],
            },
            &symbols,
            &bindings,
            &primary,
            Some(&applications),
            &arena,
        )
        .expect("nested structure");
        let set_terms = SourceSetTermProducer::build(
            SourceSetTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![
                    SourceSetTermInput {
                        site: node(6),
                        source_range: range(source, 10, 60),
                        source_ordinal: 0,
                        context: BindingContextId::new(0),
                        recovery: SourceSetTermRecovery::Normal,
                        spelling: "{ { f ( 1 ) . x } }".to_owned(),
                        kind: SourceSetTermKind::Enumeration,
                    },
                    SourceSetTermInput {
                        site: node(5),
                        source_range: range(source, 20, 52),
                        source_ordinal: 1,
                        context: BindingContextId::new(0),
                        recovery: SourceSetTermRecovery::Normal,
                        spelling: "{ f ( 1 ) . x }".to_owned(),
                        kind: SourceSetTermKind::Enumeration,
                    },
                ],
                wrappers: Vec::new(),
                generators: Vec::new(),
                type_sites: Vec::new(),
                conditions: Vec::new(),
                edges: vec![
                    SourceSetEdgeInput {
                        term: SourceSetTermId::new(0),
                        ordinal: 0,
                        role: SourceSetEdgeRole::EnumerationElement,
                        target: SourceSetTarget::SetTerm(SourceSetTermId::new(1)),
                    },
                    SourceSetEdgeInput {
                        term: SourceSetTermId::new(1),
                        ordinal: 0,
                        role: SourceSetEdgeRole::EnumerationElement,
                        target: SourceSetTarget::Structure(SourceStructureTermId::new(0)),
                    },
                ],
                requests: vec![
                    SourceSetRequestInput {
                        term: SourceSetTermId::new(0),
                        ordinal: 0,
                        kind: SourceSetRequestKind::ResultType,
                        generator: None,
                        type_site: None,
                    },
                    SourceSetRequestInput {
                        term: SourceSetTermId::new(1),
                        ordinal: 0,
                        kind: SourceSetRequestKind::ResultType,
                        generator: None,
                        type_site: None,
                    },
                ],
            },
            &bindings,
            &primary,
            Some(&applications),
            Some(&structures),
            &arena,
        )
        .expect("nested set terms");
        let input = type_assertion_input(
            source,
            &module,
            7,
            (5, 70),
            8,
            9,
            (65, 68),
            "{ { f ( 1 ) . x } }",
            SourceAtomicTermTarget::SetTerm(SourceSetTermId::new(0)),
        );
        SourceAtomicFormulaProducer::build(
            input.clone(),
            &bindings,
            &symbols,
            &primary,
            Some(&applications),
            Some(&structures),
            Some(&set_terms),
            &arena,
        )
        .expect("outer set root is the sole direct target");
        for target in [
            SourceAtomicTermTarget::Application(SourceFunctorApplicationId::new(0)),
            SourceAtomicTermTarget::Structure(SourceStructureTermId::new(0)),
            SourceAtomicTermTarget::SetTerm(SourceSetTermId::new(1)),
        ] {
            let mut non_root = input.clone();
            non_root.edges[0].target = target;
            assert!(
                SourceAtomicFormulaProducer::build(
                    non_root,
                    &bindings,
                    &symbols,
                    &primary,
                    Some(&applications),
                    Some(&structures),
                    Some(&set_terms),
                    &arena,
                )
                .is_err()
            );
        }
    }

    #[test]
    fn disjoint_lower_families_install_before_or_after_atomic_without_fingerprints() {
        let source = source_id();
        let module = module();
        let bindings = bindings(source, &module);
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.functor-application.inline",
                    SourceAnchor::Range(range(source, 10, 13)),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(source, 10, 11)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 31, 32)),
                ),
                TypedNode::new(
                    "source.term.structure.selector",
                    SourceAnchor::Range(range(source, 30, 50)),
                ),
                TypedNode::new(
                    "source.term.structure.member.selector",
                    SourceAnchor::Range(range(source, 35, 42)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 61, 62)),
                ),
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 60, 66)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 90, 91)),
                ),
                TypedNode::new(
                    "source.formula.atomic.type-assertion",
                    SourceAnchor::Range(range(source, 85, 100)),
                ),
                TypedNode::new(
                    "source.formula.atomic.asserted-type",
                    SourceAnchor::Range(range(source, 95, 98)),
                ),
                TypedNode::new(
                    "source.formula.atomic.asserted-type-head",
                    SourceAnchor::Range(range(source, 95, 98)),
                ),
                TypedNode::new(
                    "source.term.functor-application.inline",
                    SourceAnchor::Range(range(source, 88, 94)),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(source, 88, 89)),
                ),
                TypedNode::new(
                    "source.term.structure.selector",
                    SourceAnchor::Range(range(source, 88, 94)),
                ),
                TypedNode::new(
                    "source.term.structure.member.selector",
                    SourceAnchor::Range(range(source, 92, 94)),
                ),
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 88, 94)),
                ),
            ],
        )
        .expect("disjoint-family arena");
        let primary = primary_handoff(
            source,
            &module,
            &bindings,
            &arena,
            &[(2, 31, 32, "1"), (5, 61, 62, "2"), (7, 90, 91, "3")],
        );
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let applications = SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: source,
                module_id: module.clone(),
                applications: vec![SourceFunctorApplicationInput {
                    site: node(0),
                    source_range: range(source, 10, 13),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceFunctorApplicationRecovery::Normal,
                    spelling: "f ( )".to_owned(),
                    kind: SourceFunctorApplicationKind::Inline,
                    form: SourceFunctorApplicationForm::Functional,
                    head_ordinal: 0,
                    head: SourceFunctorHeadSite::Single {
                        site: node(1),
                        source_range: range(source, 10, 11),
                        spelling: "f".to_owned(),
                    },
                }],
                wrappers: Vec::new(),
                candidates: Vec::new(),
                arguments: Vec::new(),
                type_requests: Vec::new(),
            },
            &symbols,
            &bindings,
            &primary,
            &arena,
        )
        .expect("disjoint application");
        let structures = SourceStructureProducer::build(
            SourceStructureHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![SourceStructureTermInput {
                    site: node(3),
                    source_range: range(source, 30, 50),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceStructureRecovery::Normal,
                    spelling: "1 . carrier".to_owned(),
                    kind: SourceStructureTermKind::SelectorAccess,
                }],
                wrappers: Vec::new(),
                roots: Vec::new(),
                members: vec![SourceStructureMemberInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    site: node(4),
                    source_range: range(source, 35, 42),
                    spelling: "carrier".to_owned(),
                    role: SourceStructureMemberRole::Selector,
                    parent: None,
                }],
                field_updates: Vec::new(),
                edges: vec![SourceStructureEdgeInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    role: SourceStructureEdgeRole::SelectorBase,
                    member: None,
                    target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(0)),
                }],
                requests: vec![
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 0,
                        kind: SourceStructureRequestKind::MemberIdentity,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 1,
                        kind: SourceStructureRequestKind::InheritancePath,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: None,
                        request_ordinal: 2,
                        kind: SourceStructureRequestKind::ResultType,
                    },
                ],
            },
            &symbols,
            &bindings,
            &primary,
            None,
            &arena,
        )
        .expect("disjoint structure");
        let set_terms = SourceSetTermProducer::build(
            SourceSetTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![SourceSetTermInput {
                    site: node(6),
                    source_range: range(source, 60, 66),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    spelling: "{ 2 }".to_owned(),
                    kind: SourceSetTermKind::Enumeration,
                }],
                wrappers: Vec::new(),
                generators: Vec::new(),
                type_sites: Vec::new(),
                conditions: Vec::new(),
                edges: vec![SourceSetEdgeInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    role: SourceSetEdgeRole::EnumerationElement,
                    target: SourceSetTarget::Primary(SourcePrimaryTermId::new(1)),
                }],
                requests: vec![SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                }],
            },
            &bindings,
            &primary,
            None,
            None,
            &arena,
        )
        .expect("disjoint set");
        let input = type_assertion_input(
            source,
            &module,
            8,
            (85, 100),
            9,
            10,
            (95, 98),
            "3",
            SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(2)),
        );
        let without_optional = SourceAtomicFormulaProducer::build(
            input.clone(),
            &bindings,
            &symbols,
            &primary,
            None,
            None,
            None,
            &arena,
        )
        .expect("atomic without disjoint lower families");
        let with_optional = SourceAtomicFormulaProducer::build(
            input,
            &bindings,
            &symbols,
            &primary,
            Some(&applications),
            Some(&structures),
            Some(&set_terms),
            &arena,
        )
        .expect("atomic with disjoint lower families");
        assert_eq!(with_optional, without_optional);
        assert_eq!(with_optional.application_fingerprint(), None);
        assert_eq!(with_optional.structure_fingerprint(), None);
        assert_eq!(with_optional.set_term_fingerprint(), None);

        let base = typed_ast_with_primary(source, &module, &arena, primary.clone());
        let application_after = base
            .clone()
            .with_source_atomic_formula(with_optional.clone())
            .expect("atomic first")
            .with_source_application(applications.clone())
            .expect("later application revalidates atomic");
        assert_eq!(
            application_after.source_atomic_formula(),
            Some(&with_optional)
        );
        let structure_after = base
            .clone()
            .with_source_atomic_formula(with_optional.clone())
            .expect("atomic first")
            .with_source_structure(structures.clone())
            .expect("later structure revalidates atomic");
        assert_eq!(
            structure_after.source_atomic_formula(),
            Some(&with_optional)
        );
        let set_after = base
            .clone()
            .with_source_atomic_formula(with_optional.clone())
            .expect("atomic first")
            .with_source_set_term(set_terms.clone())
            .expect("later set revalidates atomic");
        assert_eq!(set_after.source_atomic_formula(), Some(&with_optional));

        let all_after = base
            .clone()
            .with_source_atomic_formula(with_optional.clone())
            .expect("atomic first")
            .with_source_application(applications.clone())
            .expect("application after atomic")
            .with_source_structure(structures.clone())
            .expect("structure after atomic")
            .with_source_set_term(set_terms.clone())
            .expect("set after atomic");
        assert_eq!(all_after.source_atomic_formula(), Some(&with_optional));
        let all_before = base
            .with_source_application(applications)
            .expect("application before atomic")
            .with_source_structure(structures)
            .expect("structure before atomic")
            .with_source_set_term(set_terms)
            .expect("set before atomic")
            .with_source_atomic_formula(with_optional.clone())
            .expect("atomic after lower families");
        assert_eq!(all_before.source_atomic_formula(), Some(&with_optional));

        let overlapping_application = SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: source,
                module_id: module.clone(),
                applications: vec![SourceFunctorApplicationInput {
                    site: node(11),
                    source_range: range(source, 88, 94),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceFunctorApplicationRecovery::Normal,
                    spelling: "f ( 3 )".to_owned(),
                    kind: SourceFunctorApplicationKind::Inline,
                    form: SourceFunctorApplicationForm::Functional,
                    head_ordinal: 0,
                    head: SourceFunctorHeadSite::Single {
                        site: node(12),
                        source_range: range(source, 88, 89),
                        spelling: "f".to_owned(),
                    },
                }],
                wrappers: Vec::new(),
                candidates: Vec::new(),
                arguments: vec![SourceFunctorArgumentInput {
                    application: SourceFunctorApplicationId::new(0),
                    ordinal: 0,
                    target: SourceFunctorArgumentTarget::Primary(SourcePrimaryTermId::new(2)),
                }],
                type_requests: Vec::new(),
            },
            &symbols,
            &bindings,
            &primary,
            &arena,
        )
        .expect("overlapping application");
        let overlapping_structure = SourceStructureProducer::build(
            SourceStructureHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![SourceStructureTermInput {
                    site: node(13),
                    source_range: range(source, 88, 94),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceStructureRecovery::Normal,
                    spelling: "3 . x".to_owned(),
                    kind: SourceStructureTermKind::SelectorAccess,
                }],
                wrappers: Vec::new(),
                roots: Vec::new(),
                members: vec![SourceStructureMemberInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    site: node(14),
                    source_range: range(source, 92, 94),
                    spelling: "x".to_owned(),
                    role: SourceStructureMemberRole::Selector,
                    parent: None,
                }],
                field_updates: Vec::new(),
                edges: vec![SourceStructureEdgeInput {
                    term: SourceStructureTermId::new(0),
                    ordinal: 0,
                    role: SourceStructureEdgeRole::SelectorBase,
                    member: None,
                    target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(2)),
                }],
                requests: vec![
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 0,
                        kind: SourceStructureRequestKind::MemberIdentity,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: Some(SourceStructureMemberId::new(0)),
                        request_ordinal: 1,
                        kind: SourceStructureRequestKind::InheritancePath,
                    },
                    SourceStructureRequestInput {
                        term: SourceStructureTermId::new(0),
                        member: None,
                        request_ordinal: 2,
                        kind: SourceStructureRequestKind::ResultType,
                    },
                ],
            },
            &symbols,
            &bindings,
            &primary,
            None,
            &arena,
        )
        .expect("overlapping structure");
        let overlapping_set = SourceSetTermProducer::build(
            SourceSetTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![SourceSetTermInput {
                    site: node(15),
                    source_range: range(source, 88, 94),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    spelling: "{ 3 }".to_owned(),
                    kind: SourceSetTermKind::Enumeration,
                }],
                wrappers: Vec::new(),
                generators: Vec::new(),
                type_sites: Vec::new(),
                conditions: Vec::new(),
                edges: vec![SourceSetEdgeInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    role: SourceSetEdgeRole::EnumerationElement,
                    target: SourceSetTarget::Primary(SourcePrimaryTermId::new(2)),
                }],
                requests: vec![SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                }],
            },
            &bindings,
            &primary,
            None,
            None,
            &arena,
        )
        .expect("overlapping set");
        let atomic_first = typed_ast_with_primary(source, &module, &arena, primary)
            .with_source_atomic_formula(with_optional.clone())
            .expect("primary-only atomic install");
        assert_eq!(
            atomic_first
                .clone()
                .with_source_application(overlapping_application)
                .expect_err("overlapping later application must reject"),
            TypedAstError::InvalidSourceApplication
        );
        assert_eq!(atomic_first.source_atomic_formula(), Some(&with_optional));
        assert_eq!(
            atomic_first
                .clone()
                .with_source_structure(overlapping_structure)
                .expect_err("overlapping later structure must reject"),
            TypedAstError::InvalidSourceStructure
        );
        assert_eq!(atomic_first.source_atomic_formula(), Some(&with_optional));
        assert_eq!(
            atomic_first
                .clone()
                .with_source_set_term(overlapping_set)
                .expect_err("overlapping later set must reject"),
            TypedAstError::InvalidSourceSetTerm
        );
        assert_eq!(atomic_first.source_atomic_formula(), Some(&with_optional));
    }

    #[test]
    fn wrapper_is_canonical_and_changes_effective_formula_only() {
        let mut fixture = make_fixture(SourceAtomicFormulaKind::Equality);
        fixture.input.wrappers.push(SourceAtomicWrapperInput {
            formula: SourceAtomicFormulaId::new(0),
            ordinal: 0,
            site: node(5),
            source_range: range(fixture.source, 3, 27),
            context: BindingContextId::new(0),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "( 1 = 2 )".to_owned(),
        });
        let handoff = build(&fixture).expect("wrapped formula");
        assert_eq!(handoff.wrappers().len(), 1);
        fixture.input.wrappers[0].spelling = "(1 = 2)".to_owned();
        assert!(matches!(
            build(&fixture),
            Err(SourceAtomicFormulaError::InvalidWrapper { .. })
        ));
    }

    #[test]
    fn spelling_edge_and_request_corruption_reject_atomically() {
        let mut fixture = make_fixture(SourceAtomicFormulaKind::Equality);
        fixture.input.formulas[0].spelling = "1 = 1".to_owned();
        assert!(matches!(
            build(&fixture),
            Err(SourceAtomicFormulaError::InvalidFormula { .. })
        ));

        let mut fixture = make_fixture(SourceAtomicFormulaKind::Equality);
        fixture.input.edges.swap(0, 1);
        assert!(matches!(
            build(&fixture),
            Err(SourceAtomicFormulaError::ReorderedEdge { .. })
                | Err(SourceAtomicFormulaError::InvalidEdge { .. })
        ));

        let mut fixture = make_fixture(SourceAtomicFormulaKind::Membership);
        fixture.input.requests[0].edge = Some(SourceAtomicEdgeId::new(0));
        assert!(matches!(
            build(&fixture),
            Err(SourceAtomicFormulaError::InvalidRequest { .. })
        ));
    }

    #[test]
    fn partial_cross_context_duplicate_parent_and_overlapping_maxima_reject() {
        let mut partial = make_fixture(SourceAtomicFormulaKind::Equality);
        partial.input.formulas[0].source_range = range(partial.source, 5, 21);
        let mut nodes = partial
            .arena
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        nodes[1].anchor = SourceAnchor::Range(range(partial.source, 19, 23));
        nodes[2].anchor = SourceAnchor::Range(range(partial.source, 5, 21));
        partial.arena = TypedArena::try_new(None, nodes).expect("partial-overlap arena");
        partial.primary = primary_handoff(
            partial.source,
            &partial.module,
            &partial.bindings,
            &partial.arena,
            &[(0, 10, 11, "1"), (1, 19, 23, "2")],
        );
        assert_build_rejects(&partial);

        let mut cross_context = make_fixture(SourceAtomicFormulaKind::Equality);
        cross_context.bindings =
            bindings_with_two_contexts(cross_context.source, &cross_context.module);
        cross_context.input.formulas[0].context = BindingContextId::new(1);
        assert!(matches!(
            build(&cross_context),
            Err(SourceAtomicFormulaError::PrimaryDependencyMismatch)
        ));

        let mut multiple_parent = make_fixture(SourceAtomicFormulaKind::Equality);
        let mut nodes = multiple_parent
            .arena
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        nodes.push(TypedNode::new(
            "source.formula.atomic.equality",
            SourceAnchor::Range(range(multiple_parent.source, 5, 25)),
        ));
        multiple_parent.arena = TypedArena::try_new(None, nodes).expect("multiple-parent arena");
        multiple_parent
            .input
            .formulas
            .push(SourceAtomicFormulaInput {
                site: node(6),
                source_range: range(multiple_parent.source, 5, 25),
                source_ordinal: 1,
                context: BindingContextId::new(0),
                recovery: SourceAtomicFormulaRecovery::Normal,
                spelling: "1 = 2".to_owned(),
                kind: SourceAtomicFormulaKind::Equality,
            });
        multiple_parent.input.edges.extend([
            SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(1),
                ordinal: 0,
                role: SourceAtomicEdgeRole::BuiltinLeftOperand,
                target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0)),
            },
            SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(1),
                ordinal: 1,
                role: SourceAtomicEdgeRole::BuiltinRightOperand,
                target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(1)),
            },
        ]);
        multiple_parent.input.requests.extend([
            SourceAtomicRequestInput {
                formula: SourceAtomicFormulaId::new(1),
                ordinal: 0,
                kind: SourceAtomicRequestKind::OperandExpectedType,
                edge: Some(SourceAtomicEdgeId::new(2)),
                candidate: None,
                type_site: None,
                attribute: None,
            },
            SourceAtomicRequestInput {
                formula: SourceAtomicFormulaId::new(1),
                ordinal: 1,
                kind: SourceAtomicRequestKind::OperandExpectedType,
                edge: Some(SourceAtomicEdgeId::new(3)),
                candidate: None,
                type_site: None,
                attribute: None,
            },
        ]);
        let multiple_parent_error =
            build(&multiple_parent).expect_err("duplicate formula parents must reject");
        assert!(
            matches!(
                multiple_parent_error,
                SourceAtomicFormulaError::DuplicateTarget { .. }
                    | SourceAtomicFormulaError::ReorderedFormula { .. }
            ),
            "unexpected duplicate-parent error: {multiple_parent_error:?}"
        );

        let source = source_id();
        let module = module();
        let bindings = bindings(source, &module);
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.functor-application.inline",
                    SourceAnchor::Range(range(source, 10, 16)),
                ),
                TypedNode::new(
                    "source.term.functor-head.single",
                    SourceAnchor::Range(range(source, 10, 11)),
                ),
                TypedNode::new(
                    "source.term.set.enumeration",
                    SourceAnchor::Range(range(source, 12, 18)),
                ),
                TypedNode::new(
                    "source.formula.atomic.type-assertion",
                    SourceAnchor::Range(range(source, 5, 25)),
                ),
                TypedNode::new(
                    "source.formula.atomic.asserted-type",
                    SourceAnchor::Range(range(source, 20, 23)),
                ),
                TypedNode::new(
                    "source.formula.atomic.asserted-type-head",
                    SourceAnchor::Range(range(source, 20, 23)),
                ),
            ],
        )
        .expect("overlapping maxima arena");
        let primary = primary_handoff(source, &module, &bindings, &arena, &[]);
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let applications = SourceFunctorApplicationProducer::build(
            SourceFunctorApplicationHandoffInput {
                source_id: source,
                module_id: module.clone(),
                applications: vec![SourceFunctorApplicationInput {
                    site: node(0),
                    source_range: range(source, 10, 16),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceFunctorApplicationRecovery::Normal,
                    spelling: "f ( )".to_owned(),
                    kind: SourceFunctorApplicationKind::Inline,
                    form: SourceFunctorApplicationForm::Functional,
                    head_ordinal: 0,
                    head: SourceFunctorHeadSite::Single {
                        site: node(1),
                        source_range: range(source, 10, 11),
                        spelling: "f".to_owned(),
                    },
                }],
                wrappers: Vec::new(),
                candidates: Vec::new(),
                arguments: Vec::new(),
                type_requests: Vec::new(),
            },
            &symbols,
            &bindings,
            &primary,
            &arena,
        )
        .expect("overlapping maximum application");
        let set_terms = SourceSetTermProducer::build(
            SourceSetTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: vec![SourceSetTermInput {
                    site: node(2),
                    source_range: range(source, 12, 18),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceSetTermRecovery::Normal,
                    spelling: "{ }".to_owned(),
                    kind: SourceSetTermKind::Enumeration,
                }],
                wrappers: Vec::new(),
                generators: Vec::new(),
                type_sites: Vec::new(),
                conditions: Vec::new(),
                edges: Vec::new(),
                requests: vec![SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                }],
            },
            &bindings,
            &primary,
            None,
            None,
            &arena,
        )
        .expect("overlapping maximum set");
        let overlapping_maxima_error = SourceAtomicFormulaProducer::build(
            type_assertion_input(
                source,
                &module,
                3,
                (5, 25),
                4,
                5,
                (20, 23),
                "f ( )",
                SourceAtomicTermTarget::Application(SourceFunctorApplicationId::new(0)),
            ),
            &bindings,
            &symbols,
            &primary,
            Some(&applications),
            None,
            Some(&set_terms),
            &arena,
        )
        .expect_err("overlapping lower-family maxima must reject");
        assert!(
            matches!(
                overlapping_maxima_error,
                SourceAtomicFormulaError::OverlappingTerms { .. }
                    | SourceAtomicFormulaError::SetTermDependencyMismatch
            ),
            "unexpected overlapping-maxima error: {overlapping_maxima_error:?}"
        );
    }

    #[test]
    fn arena_key_context_and_dependency_identity_are_authenticated() {
        let mut fixture = make_fixture(SourceAtomicFormulaKind::Equality);
        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        nodes[2].kind = "source.formula.atomic.membership".into();
        fixture.arena = TypedArena::try_new(None, nodes).expect("corrupt arena");
        assert!(matches!(
            build(&fixture),
            Err(SourceAtomicFormulaError::InvalidFormula { .. })
        ));

        let mut fixture = make_fixture(SourceAtomicFormulaKind::Equality);
        fixture.input.formulas[0].context = BindingContextId::new(1);
        assert!(matches!(
            build(&fixture),
            Err(SourceAtomicFormulaError::InvalidFormula { .. })
        ));
    }

    #[test]
    fn empty_transaction_is_valid_but_orphan_rows_are_not() {
        let mut fixture = make_fixture(SourceAtomicFormulaKind::Equality);
        fixture.input.formulas.clear();
        fixture.input.edges.clear();
        fixture.input.requests.clear();
        let handoff = build(&fixture).expect("empty transaction");
        assert!(handoff.formulas().is_empty());

        fixture.input.edges.push(SourceAtomicEdgeInput {
            formula: SourceAtomicFormulaId::new(0),
            ordinal: 0,
            role: SourceAtomicEdgeRole::AssertionSubject,
            target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0)),
        });
        assert_eq!(
            build(&fixture),
            Err(SourceAtomicFormulaError::EnvironmentMismatch)
        );
    }

    #[test]
    fn many_predicate_candidates_are_dense_and_provenance_substitution_rejects() {
        let mut fixture = predicate_fixture((13, 19), 1, 1, "1 divides 2");
        install_local_symbols(
            &mut fixture,
            "divides",
            SymbolKind::Predicate,
            DefinitionKind::Predicate,
            2,
        );
        let handoff = build(&fixture).expect("two individually authenticated candidates");
        assert_eq!(handoff.candidates().len(), 2);
        assert_eq!(handoff.requests().len(), 2);

        install_local_symbols(
            &mut fixture,
            "divides",
            SymbolKind::Functor,
            DefinitionKind::Functor,
            2,
        );
        assert!(matches!(
            build(&fixture),
            Err(SourceAtomicFormulaError::InvalidCandidate { .. })
        ));

        let mut fixture = predicate_fixture((13, 19), 1, 1, "1 divides 2");
        install_local_symbols(
            &mut fixture,
            "other",
            SymbolKind::Predicate,
            DefinitionKind::Predicate,
            1,
        );
        assert!(matches!(
            build(&fixture),
            Err(SourceAtomicFormulaError::InvalidCandidate { .. })
        ));

        let mut fixture = predicate_fixture((13, 19), 1, 1, "1 divides 2");
        let mut indexes = SymbolEnvIndexes::default();
        let valid_contribution = indexes.contributions.insert(
            fixture.module.clone(),
            ContributionKind::LocalSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 0, 3)),
        );
        let substituted_contribution = indexes.contributions.insert(
            fixture.module.clone(),
            ContributionKind::LocalSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 3, 4)),
        );
        let symbol = SymbolId::new(
            fixture.module.clone(),
            LocalSymbolId::new("divides/drift"),
            FullyQualifiedName::new("atomic.fixture::divides/drift"),
        );
        let origin = SemanticOrigin::new(
            fixture.source,
            fixture.module.clone(),
            SourceAnchor::Range(range(fixture.source, 1, 2)),
            vec![0],
        );
        indexes.symbols.insert(SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Predicate,
            NamespacePath::new(fixture.module.path().as_str()),
            "divides",
            origin.clone(),
            valid_contribution,
        ));
        indexes
            .contributions
            .add_symbol(valid_contribution, symbol.clone());
        let definition = indexes.definitions.insert(DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Predicate,
            origin,
            valid_contribution,
        ));
        indexes
            .contributions
            .add_definition(valid_contribution, definition);
        fixture.symbols = SymbolEnv::new(fixture.module.clone(), indexes);
        fixture.input.candidates = vec![SourcePredicateCandidateInput {
            head: SourcePredicateHeadId::new(0),
            ordinal: 0,
            symbol,
            contribution: substituted_contribution,
        }];
        assert!(matches!(
            build(&fixture),
            Err(SourceAtomicFormulaError::InvalidCandidate { .. })
        ));
    }

    #[test]
    fn legacy_predicate_head_recovery_remains_independent_of_formula_recovery() {
        let mut fixture = predicate_fixture((13, 19), 1, 1, "1 divides 2");
        install_local_symbols(
            &mut fixture,
            "divides",
            SymbolKind::Predicate,
            DefinitionKind::Predicate,
            1,
        );
        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        nodes[6] = nodes[6].clone().with_recovery(NodeRecoveryState::Degraded);
        fixture.arena = TypedArena::try_new(None, nodes).expect("degraded predicate-head arena");
        fixture.input.predicate_heads[0].recovery = SourceAtomicFormulaRecovery::Degraded;

        let handoff = build(&fixture).expect("legacy predicate head recovery stays independent");
        assert_eq!(
            handoff
                .formulas()
                .get(SourceAtomicFormulaId::new(0))
                .expect("legacy formula")
                .recovery(),
            SourceAtomicFormulaRecovery::Normal
        );
        assert_eq!(
            handoff
                .predicate_heads()
                .get(SourcePredicateHeadId::new(0))
                .expect("legacy head")
                .recovery(),
            SourceAtomicFormulaRecovery::Degraded
        );
    }

    #[test]
    fn predicate_chain_segments_share_one_boundary_and_clone_preserve() {
        let fixture = predicate_chain_fixture();
        let handoff = build(&fixture).expect("predicate-chain handoff");
        assert_eq!(handoff.formulas().len(), 1);
        assert_eq!(handoff.wrappers().len(), 0);
        assert_eq!(handoff.predicate_segments().len(), 2);
        assert_eq!(handoff.predicate_heads().len(), 2);
        assert_eq!(handoff.candidates().len(), 2);
        assert_eq!(handoff.type_sites().len(), 0);
        assert_eq!(handoff.attributes().len(), 0);
        assert_eq!(handoff.edges().len(), 3);
        assert_eq!(handoff.requests().len(), 2);

        let first = handoff
            .predicate_segments()
            .get(SourcePredicateSegmentId::new(0))
            .expect("first segment");
        let second = handoff
            .predicate_segments()
            .get(SourcePredicateSegmentId::new(1))
            .expect("second segment");
        assert_eq!(
            handoff
                .predicate_segments()
                .iter()
                .map(|(id, row)| (id.index(), row.ordinal()))
                .collect::<Vec<_>>(),
            [(0, 0), (1, 1)]
        );
        assert_eq!(first.formula(), SourceAtomicFormulaId::new(0));
        assert_eq!(first.ordinal(), 0);
        assert_eq!(first.site(), &node(4));
        assert_eq!(first.source_range(), range(fixture.source, 75, 86));
        assert_eq!(first.context(), BindingContextId::new(0));
        assert_eq!(first.recovery(), SourceAtomicFormulaRecovery::Normal);
        assert_eq!(first.spelling(), "1 divides 2");
        assert_eq!(first.head(), SourcePredicateHeadId::new(0));
        assert_eq!(first.left_edge(), SourceAtomicEdgeId::new(0));
        assert_eq!(first.right_edge(), SourceAtomicEdgeId::new(1));
        assert_eq!(
            first.polarity(),
            &SourcePredicateSegmentPolarityInput::Positive
        );
        assert_eq!(second.formula(), SourceAtomicFormulaId::new(0));
        assert_eq!(second.ordinal(), 1);
        assert_eq!(second.site(), &node(5));
        assert_eq!(second.source_range(), range(fixture.source, 87, 105));
        assert_eq!(second.context(), BindingContextId::new(0));
        assert_eq!(second.recovery(), SourceAtomicFormulaRecovery::Normal);
        assert_eq!(second.spelling(), "does not divides 3");
        assert_eq!(second.head(), SourcePredicateHeadId::new(1));
        assert_eq!(second.left_edge(), SourceAtomicEdgeId::new(1));
        assert_eq!(second.right_edge(), SourceAtomicEdgeId::new(2));
        assert_eq!(
            second.polarity(),
            &SourcePredicateSegmentPolarityInput::Negative {
                verb_site: node(8),
                verb_range: range(fixture.source, 87, 91),
                verb_spelling: "does".to_owned(),
                verb_recovery: SourceAtomicFormulaRecovery::Normal,
                not_site: node(9),
                not_range: range(fixture.source, 92, 95),
                not_spelling: "not".to_owned(),
                not_recovery: SourceAtomicFormulaRecovery::Normal,
            }
        );
        assert_eq!(
            handoff
                .edges()
                .get(SourceAtomicEdgeId::new(1))
                .map(|edge| (edge.role(), edge.target())),
            Some((
                SourceAtomicEdgeRole::PredicateChainBoundary,
                SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(1)),
            ))
        );
        assert_eq!(
            handoff
                .candidates()
                .get(SourcePredicateCandidateId::new(0))
                .map(SourcePredicateCandidate::symbol),
            handoff
                .candidates()
                .get(SourcePredicateCandidateId::new(1))
                .map(SourcePredicateCandidate::symbol)
        );

        let debug = handoff.debug_text();
        assert_eq!(
            debug,
            r#"source-atomic-formula-debug-v1
module: atomic.fixture
primary-term-fingerprint: "source-primary-term-debug-v1\nmodule: atomic.fixture\nterm#0 ordinal=0 kind=numeral role=value range=75..76 site=0 context=0 recovery=normal spelling=\"1\" parent=-\nterm#1 ordinal=1 kind=numeral role=value range=85..86 site=1 context=0 recovery=normal spelling=\"2\" parent=-\nterm#2 ordinal=2 kind=numeral role=value range=104..105 site=2 context=0 recovery=normal spelling=\"3\" parent=-\nnumeric-request#0 term=0 ordinal=0 owner=0 range=75..76 spelling=\"1\"\nnumeric-request#1 term=1 ordinal=1 owner=1 range=85..86 spelling=\"2\"\nnumeric-request#2 term=2 ordinal=2 owner=2 range=104..105 spelling=\"3\"\n"
application-fingerprint: None
structure-fingerprint: None
set-term-fingerprint: None
formula#0 ordinal=0 kind=predicate range=75..105 site=3 context=0 recovery=normal spelling="1 divides 2 does not divides 3"
predicate-segment#0 formula=0 ordinal=0 range=75..86 site=4 context=0 recovery=normal spelling="1 divides 2" head=0 polarity=positive left_edge=0 right_edge=1
predicate-segment#1 formula=0 ordinal=1 range=87..105 site=5 context=0 recovery=normal spelling="does not divides 3" head=1 polarity=negative(verb_site=8 verb_range=87..91 verb_spelling="does" verb_recovery=normal not_site=9 not_range=92..95 not_spelling="not" not_recovery=normal) left_edge=1 right_edge=2
predicate-head#0 formula=0 range=77..84 site=6 context=0 recovery=normal spelling="divides" left_arity=1 right_arity=1
predicate-head#1 formula=0 range=96..103 site=7 context=0 recovery=normal spelling="divides" left_arity=1 right_arity=1
candidate#0 head=0 ordinal=0 symbol=SymbolId { module: ModuleId { package: PackageId("pkg"), path: ModulePath("atomic.fixture") }, local: LocalSymbolId("divides/chain"), fqn: FullyQualifiedName("atomic.fixture::divides") } contribution=0
candidate#1 head=1 ordinal=0 symbol=SymbolId { module: ModuleId { package: PackageId("pkg"), path: ModulePath("atomic.fixture") }, local: LocalSymbolId("divides/chain"), fqn: FullyQualifiedName("atomic.fixture::divides") } contribution=0
edge#0 formula=0 ordinal=0 role=predicate-left target=primary:0
edge#1 formula=0 ordinal=1 role=predicate-chain-boundary target=primary:1
edge#2 formula=0 ordinal=2 role=predicate-right target=primary:2
request#0 formula=0 ordinal=0 kind=predicate-candidate-signature edge=- candidate=0 type_site=- attribute=-
request#1 formula=0 ordinal=1 kind=predicate-candidate-signature edge=- candidate=1 type_site=- attribute=-
"#
        );

        let typed = typed_ast(&fixture)
            .with_source_atomic_formula(handoff.clone())
            .expect("install predicate chain");
        assert_eq!(typed.source_atomic_formula(), Some(&handoff));
        handoff
            .validate_installation(
                fixture.source,
                &fixture.module,
                &fixture.primary,
                None,
                None,
                None,
                &fixture.arena,
            )
            .expect("clone-preserved revalidation");

        let legacy =
            build(&make_fixture(SourceAtomicFormulaKind::Equality)).expect("legacy atomic formula");
        assert_eq!(
            legacy.debug_text(),
            r#"source-atomic-formula-debug-v1
module: atomic.fixture
primary-term-fingerprint: "source-primary-term-debug-v1\nmodule: atomic.fixture\nterm#0 ordinal=0 kind=numeral role=value range=10..11 site=0 context=0 recovery=normal spelling=\"1\" parent=-\nterm#1 ordinal=1 kind=numeral role=value range=20..21 site=1 context=0 recovery=normal spelling=\"2\" parent=-\nnumeric-request#0 term=0 ordinal=0 owner=0 range=10..11 spelling=\"1\"\nnumeric-request#1 term=1 ordinal=1 owner=1 range=20..21 spelling=\"2\"\n"
application-fingerprint: None
structure-fingerprint: None
set-term-fingerprint: None
formula#0 ordinal=0 kind=equality range=5..25 site=2 context=0 recovery=normal spelling="1 = 2"
edge#0 formula=0 ordinal=0 role=builtin-left target=primary:0
edge#1 formula=0 ordinal=1 role=builtin-right target=primary:1
request#0 formula=0 ordinal=0 kind=operand-expected-type edge=0 candidate=- type_site=- attribute=-
request#1 formula=0 ordinal=1 kind=operand-expected-type edge=1 candidate=- type_site=- attribute=-
"#
        );
        assert!(legacy.predicate_segments().is_empty());
    }

    #[test]
    fn predicate_chain_segment_corruption_fails_closed() {
        type Mutation = fn(&mut Fixture);
        let mutations: &[Mutation] = &[
            |fixture| {
                fixture.input.predicate_segments.pop();
            },
            |fixture| fixture.input.predicate_segments[1].ordinal = 0,
            |fixture| {
                fixture.input.predicate_segments[1].formula = SourceAtomicFormulaId::new(1);
            },
            |fixture| fixture.input.predicate_segments[1].site = node(4),
            |fixture| {
                fixture.input.predicate_segments[1].source_range = range(fixture.source, 84, 105);
            },
            |fixture| {
                fixture.input.predicate_segments[1].context = BindingContextId::new(1);
            },
            |fixture| {
                fixture.input.predicate_segments[1].recovery =
                    SourceAtomicFormulaRecovery::Degraded;
            },
            |fixture| fixture.input.predicate_segments[1].spelling = "divides 3".to_owned(),
            |fixture| fixture.input.predicate_segments[1].head = SourcePredicateHeadId::new(0),
            |fixture| fixture.input.predicate_segments[1].left_edge = SourceAtomicEdgeId::new(0),
            |fixture| fixture.input.predicate_segments[1].right_edge = SourceAtomicEdgeId::new(1),
            |fixture| {
                let SourcePredicateSegmentPolarityInput::Negative { verb_spelling, .. } =
                    &mut fixture.input.predicate_segments[1].polarity
                else {
                    unreachable!()
                };
                *verb_spelling = "is".to_owned();
            },
            |fixture| {
                let SourcePredicateSegmentPolarityInput::Negative { verb_site, .. } =
                    &mut fixture.input.predicate_segments[1].polarity
                else {
                    unreachable!()
                };
                *verb_site = node(9);
            },
            |fixture| {
                let SourcePredicateSegmentPolarityInput::Negative { verb_range, .. } =
                    &mut fixture.input.predicate_segments[1].polarity
                else {
                    unreachable!()
                };
                *verb_range = range(fixture.source, 86, 91);
            },
            |fixture| {
                let SourcePredicateSegmentPolarityInput::Negative { verb_recovery, .. } =
                    &mut fixture.input.predicate_segments[1].polarity
                else {
                    unreachable!()
                };
                *verb_recovery = SourceAtomicFormulaRecovery::Degraded;
            },
            |fixture| {
                let SourcePredicateSegmentPolarityInput::Negative { not_site, .. } =
                    &mut fixture.input.predicate_segments[1].polarity
                else {
                    unreachable!()
                };
                *not_site = node(8);
            },
            |fixture| {
                let SourcePredicateSegmentPolarityInput::Negative { not_range, .. } =
                    &mut fixture.input.predicate_segments[1].polarity
                else {
                    unreachable!()
                };
                *not_range = range(fixture.source, 91, 95);
            },
            |fixture| {
                let SourcePredicateSegmentPolarityInput::Negative { not_spelling, .. } =
                    &mut fixture.input.predicate_segments[1].polarity
                else {
                    unreachable!()
                };
                *not_spelling = "non".to_owned();
            },
            |fixture| {
                let SourcePredicateSegmentPolarityInput::Negative { not_recovery, .. } =
                    &mut fixture.input.predicate_segments[1].polarity
                else {
                    unreachable!()
                };
                *not_recovery = SourceAtomicFormulaRecovery::Degraded;
            },
            |fixture| {
                fixture.input.predicate_segments[1].polarity =
                    SourcePredicateSegmentPolarityInput::Positive;
            },
            |fixture| {
                let mut nodes = fixture
                    .arena
                    .iter()
                    .map(|(_, row)| row.clone())
                    .collect::<Vec<_>>();
                nodes[7] = nodes[7].clone().with_recovery(NodeRecoveryState::Degraded);
                fixture.arena =
                    TypedArena::try_new(None, nodes).expect("degraded chain-head arena");
                fixture.input.predicate_heads[1].recovery = SourceAtomicFormulaRecovery::Degraded;
            },
            |fixture| fixture.input.predicate_heads[1].left_arity = 2,
            |fixture| fixture.input.candidates[1].head = SourcePredicateHeadId::new(0),
            |fixture| fixture.input.edges[1].role = SourceAtomicEdgeRole::PredicateRightArgument,
            |fixture| {
                fixture.input.edges[1].target =
                    SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0));
            },
            |fixture| {
                fixture.input.requests[1].candidate = Some(SourcePredicateCandidateId::new(0));
            },
        ];
        for mutate in mutations {
            let mut fixture = predicate_chain_fixture();
            mutate(&mut fixture);
            assert_build_rejects(&fixture);
        }

        let mut one_segment = predicate_chain_fixture();
        one_segment.input.predicate_segments.truncate(1);
        one_segment.input.predicate_heads.truncate(1);
        one_segment.input.candidates.truncate(1);
        one_segment.input.edges.truncate(2);
        one_segment.input.edges[1].role = SourceAtomicEdgeRole::PredicateRightArgument;
        one_segment.input.requests.truncate(1);
        assert_build_rejects(&one_segment);
    }

    #[test]
    fn prefix_infix_and_postfix_predicate_arities_preserve_written_order() {
        for (head_range, left, right, spelling) in [
            ((6, 9), 0, 2, "divides 1 , 2"),
            ((13, 19), 1, 1, "1 divides 2"),
            ((22, 24), 2, 0, "1 , 2 divides"),
        ] {
            let fixture = predicate_fixture(head_range, left, right, spelling);
            validate_payload(
                &fixture.input,
                Some(&fixture.bindings),
                None,
                &fixture.primary,
                None,
                None,
                None,
                &fixture.arena,
            )
            .expect("valid predicate arity");
        }
    }

    #[test]
    fn nested_wrappers_and_independent_degraded_rows_are_preserved() {
        let mut fixture = make_fixture(SourceAtomicFormulaKind::Equality);
        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        nodes[5] = nodes[5].clone().with_recovery(NodeRecoveryState::Degraded);
        nodes.push(
            TypedNode::new(
                "source.formula.atomic.parenthesized",
                SourceAnchor::Range(range(fixture.source, 1, 29)),
            )
            .with_recovery(NodeRecoveryState::Recovered),
        );
        fixture.arena = TypedArena::try_new(None, nodes).expect("nested wrapper arena");
        fixture.input.wrappers = vec![
            SourceAtomicWrapperInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                site: node(6),
                source_range: range(fixture.source, 1, 29),
                context: BindingContextId::new(0),
                recovery: SourceAtomicFormulaRecovery::Degraded,
                spelling: "( ( 1 = 2 ) )".to_owned(),
            },
            SourceAtomicWrapperInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 1,
                site: node(5),
                source_range: range(fixture.source, 3, 27),
                context: BindingContextId::new(0),
                recovery: SourceAtomicFormulaRecovery::Degraded,
                spelling: "( 1 = 2 )".to_owned(),
            },
        ];
        let handoff = build(&fixture).expect("nested degraded wrappers");
        assert_eq!(handoff.wrappers().len(), 2);

        fixture.input.wrappers.swap(0, 1);
        assert!(matches!(
            build(&fixture),
            Err(SourceAtomicFormulaError::ReorderedWrapper { .. })
        ));
    }

    #[test]
    fn duplicate_crossing_same_range_and_cross_formula_wrappers_reject() {
        let wrapper = |fixture: &Fixture, ordinal, site, start, end, spelling: &str| {
            SourceAtomicWrapperInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal,
                site: node(site),
                source_range: range(fixture.source, start, end),
                context: BindingContextId::new(0),
                recovery: SourceAtomicFormulaRecovery::Normal,
                spelling: spelling.to_owned(),
            }
        };

        let mut duplicate = make_fixture(SourceAtomicFormulaKind::Equality);
        duplicate.input.wrappers = vec![
            wrapper(&duplicate, 0, 5, 3, 27, "( 1 = 2 )"),
            wrapper(&duplicate, 1, 5, 3, 27, "( 1 = 2 )"),
        ];
        assert_build_rejects(&duplicate);

        let mut same_range = make_fixture(SourceAtomicFormulaKind::Equality);
        let mut nodes = same_range
            .arena
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        nodes.push(TypedNode::new(
            "source.formula.atomic.parenthesized",
            SourceAnchor::Range(range(same_range.source, 3, 27)),
        ));
        same_range.arena = TypedArena::try_new(None, nodes).expect("same-range wrapper arena");
        same_range.input.wrappers = vec![
            wrapper(&same_range, 0, 5, 3, 27, "( 1 = 2 )"),
            wrapper(&same_range, 1, 6, 3, 27, "( 1 = 2 )"),
        ];
        assert_build_rejects(&same_range);

        let mut crossing = make_fixture(SourceAtomicFormulaKind::Equality);
        let mut nodes = crossing
            .arena
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        nodes.push(TypedNode::new(
            "source.formula.atomic.parenthesized",
            SourceAnchor::Range(range(crossing.source, 1, 26)),
        ));
        crossing.arena = TypedArena::try_new(None, nodes).expect("crossing wrapper arena");
        crossing.input.wrappers = vec![
            wrapper(&crossing, 0, 6, 1, 26, "( ( 1 = 2 )"),
            wrapper(&crossing, 1, 5, 3, 27, "( 1 = 2 )"),
        ];
        assert_build_rejects(&crossing);

        let source = source_id();
        let module = module();
        let bindings = bindings(source, &module);
        let arena = TypedArena::try_new(
            None,
            vec![
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 10, 11)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 20, 21)),
                ),
                TypedNode::new(
                    "source.formula.atomic.equality",
                    SourceAnchor::Range(range(source, 5, 25)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 40, 41)),
                ),
                TypedNode::new(
                    "source.term.numeral",
                    SourceAnchor::Range(range(source, 50, 51)),
                ),
                TypedNode::new(
                    "source.formula.atomic.equality",
                    SourceAnchor::Range(range(source, 35, 55)),
                ),
                TypedNode::new(
                    "source.formula.atomic.parenthesized",
                    SourceAnchor::Range(range(source, 33, 57)),
                ),
            ],
        )
        .expect("cross-formula wrapper arena");
        let primary = primary_handoff(
            source,
            &module,
            &bindings,
            &arena,
            &[
                (0, 10, 11, "1"),
                (1, 20, 21, "2"),
                (3, 40, 41, "3"),
                (4, 50, 51, "4"),
            ],
        );
        let mut edges = Vec::new();
        let mut requests = Vec::new();
        for formula in 0..2 {
            for ordinal in 0..2 {
                let edge = edges.len();
                edges.push(SourceAtomicEdgeInput {
                    formula: SourceAtomicFormulaId::new(formula),
                    ordinal,
                    role: if ordinal == 0 {
                        SourceAtomicEdgeRole::BuiltinLeftOperand
                    } else {
                        SourceAtomicEdgeRole::BuiltinRightOperand
                    },
                    target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(
                        formula * 2 + ordinal,
                    )),
                });
                requests.push(SourceAtomicRequestInput {
                    formula: SourceAtomicFormulaId::new(formula),
                    ordinal,
                    kind: SourceAtomicRequestKind::OperandExpectedType,
                    edge: Some(SourceAtomicEdgeId::new(edge)),
                    candidate: None,
                    type_site: None,
                    attribute: None,
                });
            }
        }
        let input = SourceAtomicFormulaHandoffInput {
            source_id: source,
            module_id: module.clone(),
            formulas: vec![
                SourceAtomicFormulaInput {
                    site: node(2),
                    source_range: range(source, 5, 25),
                    source_ordinal: 0,
                    context: BindingContextId::new(0),
                    recovery: SourceAtomicFormulaRecovery::Normal,
                    spelling: "1 = 2".to_owned(),
                    kind: SourceAtomicFormulaKind::Equality,
                },
                SourceAtomicFormulaInput {
                    site: node(5),
                    source_range: range(source, 35, 55),
                    source_ordinal: 1,
                    context: BindingContextId::new(0),
                    recovery: SourceAtomicFormulaRecovery::Normal,
                    spelling: "3 = 4".to_owned(),
                    kind: SourceAtomicFormulaKind::Equality,
                },
            ],
            wrappers: vec![SourceAtomicWrapperInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                site: node(6),
                source_range: range(source, 33, 57),
                context: BindingContextId::new(0),
                recovery: SourceAtomicFormulaRecovery::Normal,
                spelling: "( 3 = 4 )".to_owned(),
            }],
            predicate_segments: Vec::new(),
            predicate_heads: Vec::new(),
            candidates: Vec::new(),
            type_sites: Vec::new(),
            attributes: Vec::new(),
            edges,
            requests,
        };
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        assert!(
            SourceAtomicFormulaProducer::build(
                input, &bindings, &symbols, &primary, None, None, None, &arena,
            )
            .is_err()
        );
    }

    #[test]
    fn atomic_handoff_is_one_shot_and_clone_preserved_by_typed_ast() {
        let fixture = make_fixture(SourceAtomicFormulaKind::Equality);
        let handoff = build(&fixture).expect("handoff");
        let typed = typed_ast(&fixture)
            .with_source_atomic_formula(handoff.clone())
            .expect("first atomic install");
        assert_eq!(typed.source_atomic_formula(), Some(&handoff));
        assert_eq!(
            typed
                .with_source_atomic_formula(handoff)
                .expect_err("replacement must reject"),
            TypedAstError::InvalidSourceAtomicFormula
        );
    }

    #[test]
    fn every_flat_table_rejects_identity_order_and_association_corruption() {
        for mutate in [
            |fixture: &mut Fixture| fixture.input.formulas[0].site = node(99),
            |fixture: &mut Fixture| {
                fixture.input.formulas[0].source_range = range(fixture.source, 25, 5);
            },
            |fixture: &mut Fixture| fixture.input.formulas[0].source_ordinal = 1,
            |fixture: &mut Fixture| {
                fixture.input.formulas[0].context = BindingContextId::new(1);
            },
            |fixture: &mut Fixture| {
                fixture.input.formulas[0].recovery = SourceAtomicFormulaRecovery::Degraded;
            },
            |fixture: &mut Fixture| fixture.input.formulas[0].spelling = "1  = 2".to_owned(),
        ] {
            let mut fixture = make_fixture(SourceAtomicFormulaKind::Equality);
            mutate(&mut fixture);
            assert_build_rejects(&fixture);
        }

        for mutate in [
            |fixture: &mut Fixture| fixture.input.wrappers[0].site = node(2),
            |fixture: &mut Fixture| {
                fixture.input.wrappers[0].source_range = range(fixture.source, 6, 24);
            },
            |fixture: &mut Fixture| fixture.input.wrappers[0].ordinal = 1,
            |fixture: &mut Fixture| {
                fixture.input.wrappers[0].context = BindingContextId::new(1);
            },
            |fixture: &mut Fixture| {
                fixture.input.wrappers[0].recovery = SourceAtomicFormulaRecovery::Degraded;
            },
            |fixture: &mut Fixture| fixture.input.wrappers[0].spelling = "(1 = 2)".to_owned(),
        ] {
            let mut fixture = make_fixture(SourceAtomicFormulaKind::Equality);
            fixture.input.wrappers = vec![SourceAtomicWrapperInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                site: node(5),
                source_range: range(fixture.source, 3, 27),
                context: BindingContextId::new(0),
                recovery: SourceAtomicFormulaRecovery::Normal,
                spelling: "( 1 = 2 )".to_owned(),
            }];
            mutate(&mut fixture);
            assert_build_rejects(&fixture);
        }

        for mutate in [
            |fixture: &mut Fixture| fixture.input.type_sites[0].site = node(4),
            |fixture: &mut Fixture| {
                fixture.input.type_sites[0].source_range = range(fixture.source, 19, 23);
            },
            |fixture: &mut Fixture| {
                fixture.input.type_sites[0].context = BindingContextId::new(1);
            },
            |fixture: &mut Fixture| {
                fixture.input.type_sites[0].recovery = SourceAtomicFormulaRecovery::Degraded;
            },
            |fixture: &mut Fixture| fixture.input.type_sites[0].spelling = "object".to_owned(),
        ] {
            let mut fixture = make_fixture(SourceAtomicFormulaKind::TypeAssertion);
            mutate(&mut fixture);
            assert_build_rejects(&fixture);
        }

        for mutate in [
            |fixture: &mut Fixture| fixture.input.edges[0].ordinal = 1,
            |fixture: &mut Fixture| {
                fixture.input.edges[0].role = SourceAtomicEdgeRole::AssertionSubject;
            },
            |fixture: &mut Fixture| {
                fixture.input.edges[1].target =
                    SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0));
            },
            |fixture: &mut Fixture| {
                fixture.input.edges[0].target =
                    SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(99));
            },
        ] {
            let mut fixture = make_fixture(SourceAtomicFormulaKind::Equality);
            mutate(&mut fixture);
            assert_build_rejects(&fixture);
        }

        for mutate in [
            |fixture: &mut Fixture| fixture.input.requests[0].ordinal = 1,
            |fixture: &mut Fixture| {
                fixture.input.requests[0].edge = Some(SourceAtomicEdgeId::new(1));
            },
            |fixture: &mut Fixture| {
                fixture.input.requests[0].candidate = Some(SourcePredicateCandidateId::new(0));
            },
            |fixture: &mut Fixture| {
                fixture.input.requests[0].kind = SourceAtomicRequestKind::AttributeAdmissibility;
            },
        ] {
            let mut fixture = make_fixture(SourceAtomicFormulaKind::Equality);
            mutate(&mut fixture);
            assert_build_rejects(&fixture);
        }

        for mutate in [
            |fixture: &mut Fixture| {
                fixture.input.predicate_heads[0].formula = SourceAtomicFormulaId::new(1);
            },
            |fixture: &mut Fixture| fixture.input.predicate_heads[0].site = node(2),
            |fixture: &mut Fixture| {
                fixture.input.predicate_heads[0].source_range = range(fixture.source, 4, 19);
            },
            |fixture: &mut Fixture| {
                fixture.input.predicate_heads[0].context = BindingContextId::new(1);
            },
            |fixture: &mut Fixture| {
                fixture.input.predicate_heads[0].recovery = SourceAtomicFormulaRecovery::Degraded;
            },
            |fixture: &mut Fixture| {
                fixture.input.predicate_heads[0].spelling = "other".to_owned();
            },
            |fixture: &mut Fixture| fixture.input.predicate_heads[0].left_arity = 2,
            |fixture: &mut Fixture| fixture.input.predicate_heads[0].right_arity = 2,
        ] {
            let mut fixture = predicate_fixture((13, 19), 1, 1, "1 divides 2");
            mutate(&mut fixture);
            assert!(
                validate_payload(
                    &fixture.input,
                    Some(&fixture.bindings),
                    None,
                    &fixture.primary,
                    None,
                    None,
                    None,
                    &fixture.arena,
                )
                .is_err()
            );
        }

        for mutate in [
            |fixture: &mut Fixture| fixture.input.candidates[0].ordinal = 1,
            |fixture: &mut Fixture| {
                fixture.input.candidates[0].head = SourcePredicateHeadId::new(1);
            },
            |fixture: &mut Fixture| {
                fixture
                    .input
                    .candidates
                    .push(fixture.input.candidates[0].clone());
            },
        ] {
            let mut fixture = predicate_fixture((13, 19), 1, 1, "1 divides 2");
            mutate(&mut fixture);
            assert!(
                validate_payload(
                    &fixture.input,
                    Some(&fixture.bindings),
                    None,
                    &fixture.primary,
                    None,
                    None,
                    None,
                    &fixture.arena,
                )
                .is_err()
            );
        }

        for mutate in [
            |fixture: &mut Fixture| {
                fixture.input.attributes[0].formula = SourceAtomicFormulaId::new(1);
            },
            |fixture: &mut Fixture| fixture.input.attributes[0].ordinal = 1,
            |fixture: &mut Fixture| fixture.input.attributes[0].site = node(7),
            |fixture: &mut Fixture| {
                fixture.input.attributes[0].source_range = range(fixture.source, 12, 22);
            },
            |fixture: &mut Fixture| {
                fixture.input.attributes[0].spelling = "non  empty".to_owned();
            },
            |fixture: &mut Fixture| fixture.input.attributes[0].target_site = node(6),
            |fixture: &mut Fixture| {
                fixture.input.attributes[0].target_range = range(fixture.source, 17, 22);
            },
            |fixture: &mut Fixture| {
                fixture.input.attributes[0].target_spelling = "finite".to_owned();
            },
            |fixture: &mut Fixture| {
                fixture.input.attributes[0].context = BindingContextId::new(1);
            },
            |fixture: &mut Fixture| {
                fixture.input.attributes[0].recovery = SourceAtomicFormulaRecovery::Degraded;
            },
            |fixture: &mut Fixture| {
                fixture.input.attributes[0].symbol = SymbolId::new(
                    fixture.module.clone(),
                    LocalSymbolId::new("other/test"),
                    FullyQualifiedName::new("atomic.fixture::other"),
                );
            },
            |fixture: &mut Fixture| {
                fixture.input.attributes[0].polarity =
                    SourceAssertionAttributePolarityInput::Positive;
            },
        ] {
            let mut fixture = negative_attribute_fixture();
            build(&fixture).expect("valid authenticated negative attribute");
            mutate(&mut fixture);
            assert_build_rejects(&fixture);
        }

        let mut contribution_drift = negative_attribute_fixture();
        let mut indexes = SymbolEnvIndexes::default();
        let valid_contribution = indexes.contributions.insert(
            contribution_drift.module.clone(),
            ContributionKind::LocalSource {
                source_id: contribution_drift.source,
            },
            SourceAnchor::Range(range(contribution_drift.source, 1, 2)),
        );
        let wrong_contribution = indexes.contributions.insert(
            contribution_drift.module.clone(),
            ContributionKind::LocalSource {
                source_id: contribution_drift.source,
            },
            SourceAnchor::Range(range(contribution_drift.source, 2, 3)),
        );
        let symbol = contribution_drift.input.attributes[0].symbol.clone();
        let origin = SemanticOrigin::new(
            contribution_drift.source,
            contribution_drift.module.clone(),
            SourceAnchor::Range(range(contribution_drift.source, 1, 2)),
            vec![0],
        );
        indexes.symbols.insert(SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Attribute,
            NamespacePath::new(contribution_drift.module.path().as_str()),
            "empty",
            origin.clone(),
            valid_contribution,
        ));
        indexes
            .contributions
            .add_symbol(valid_contribution, symbol.clone());
        let definition = indexes.definitions.insert(DefinitionShell::new(
            symbol,
            DefinitionKind::Attribute,
            origin,
            valid_contribution,
        ));
        indexes
            .contributions
            .add_definition(valid_contribution, definition);
        contribution_drift.symbols = SymbolEnv::new(contribution_drift.module.clone(), indexes);
        contribution_drift.input.attributes[0].contribution = wrong_contribution;
        assert_build_rejects(&contribution_drift);

        for mutate in [
            |fixture: &mut Fixture| {
                let SourceAssertionAttributePolarityInput::Negative { non_site, .. } =
                    &mut fixture.input.attributes[0].polarity
                else {
                    unreachable!()
                };
                *non_site = node(7);
            },
            |fixture: &mut Fixture| {
                let SourceAssertionAttributePolarityInput::Negative { non_range, .. } =
                    &mut fixture.input.attributes[0].polarity
                else {
                    unreachable!()
                };
                *non_range = range(fixture.source, 12, 16);
            },
            |fixture: &mut Fixture| {
                let SourceAssertionAttributePolarityInput::Negative { non_spelling, .. } =
                    &mut fixture.input.attributes[0].polarity
                else {
                    unreachable!()
                };
                *non_spelling = "not".to_owned();
            },
            |fixture: &mut Fixture| {
                let SourceAssertionAttributePolarityInput::Negative { non_recovery, .. } =
                    &mut fixture.input.attributes[0].polarity
                else {
                    unreachable!()
                };
                *non_recovery = SourceAtomicFormulaRecovery::Degraded;
            },
        ] {
            let mut fixture = negative_attribute_fixture();
            mutate(&mut fixture);
            assert_build_rejects(&fixture);
        }

        for mutate in [
            |fixture: &mut Fixture| {
                fixture.input.type_sites[0].formula = SourceAtomicFormulaId::new(1);
            },
            |fixture: &mut Fixture| fixture.input.type_sites[0].head_site = node(3),
            |fixture: &mut Fixture| {
                fixture.input.type_sites[0].head_range = range(fixture.source, 19, 23);
            },
            |fixture: &mut Fixture| {
                fixture.input.type_sites[0].head_spelling = "object".to_owned();
            },
            |fixture: &mut Fixture| {
                fixture.input.type_sites[0].head = SourceAssertionTypeHead::BuiltinObject;
            },
        ] {
            let mut fixture = make_fixture(SourceAtomicFormulaKind::TypeAssertion);
            mutate(&mut fixture);
            assert_build_rejects(&fixture);
        }

        for mutate in [
            |fixture: &mut Fixture| {
                fixture.input.wrappers[0].formula = SourceAtomicFormulaId::new(1);
            },
            |fixture: &mut Fixture| {
                fixture.input.edges[0].formula = SourceAtomicFormulaId::new(1);
            },
            |fixture: &mut Fixture| {
                fixture.input.requests[0].formula = SourceAtomicFormulaId::new(1);
            },
        ] {
            let mut fixture = make_fixture(SourceAtomicFormulaKind::Equality);
            fixture.input.wrappers = vec![SourceAtomicWrapperInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                site: node(5),
                source_range: range(fixture.source, 3, 27),
                context: BindingContextId::new(0),
                recovery: SourceAtomicFormulaRecovery::Normal,
                spelling: "( 1 = 2 )".to_owned(),
            }];
            mutate(&mut fixture);
            assert_build_rejects(&fixture);
        }

        for mutate in [
            |fixture: &mut Fixture| fixture.input.requests[0].edge = None,
            |fixture: &mut Fixture| {
                fixture.input.requests[0].type_site = Some(SourceAssertionTypeSiteId::new(0));
            },
            |fixture: &mut Fixture| {
                fixture.input.requests[0].attribute = Some(SourceAssertionAttributeId::new(0));
            },
            |fixture: &mut Fixture| {
                fixture
                    .input
                    .requests
                    .push(fixture.input.requests[0].clone());
            },
            |fixture: &mut Fixture| {
                fixture.input.requests.remove(0);
            },
            |fixture: &mut Fixture| {
                fixture.input.requests.swap(0, 1);
            },
        ] {
            let mut fixture = make_fixture(SourceAtomicFormulaKind::Equality);
            mutate(&mut fixture);
            assert_build_rejects(&fixture);
        }

        for mutate in [
            |fixture: &mut Fixture| fixture.input.requests[0].attribute = None,
            |fixture: &mut Fixture| {
                fixture.input.requests[0].attribute = Some(SourceAssertionAttributeId::new(1));
            },
            |fixture: &mut Fixture| {
                fixture.input.requests[0].edge = Some(SourceAtomicEdgeId::new(0));
            },
            |fixture: &mut Fixture| {
                fixture.input.requests[0].candidate = Some(SourcePredicateCandidateId::new(0));
            },
            |fixture: &mut Fixture| {
                fixture.input.requests[0].type_site = Some(SourceAssertionTypeSiteId::new(0));
            },
        ] {
            let mut fixture = negative_attribute_fixture();
            mutate(&mut fixture);
            assert_build_rejects(&fixture);
        }

        let mut wrong_type_site = make_fixture(SourceAtomicFormulaKind::TypeAssertion);
        wrong_type_site.input.requests[0].type_site = Some(SourceAssertionTypeSiteId::new(1));
        assert_build_rejects(&wrong_type_site);
    }

    #[test]
    fn predicate_candidate_cardinality_roles_and_request_are_exact() {
        let mut fixture = make_fixture(SourceAtomicFormulaKind::Equality);
        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        nodes[2].kind = "source.formula.atomic.predicate".into();
        nodes.push(TypedNode::new(
            "source.formula.atomic.predicate-head",
            SourceAnchor::Range(range(fixture.source, 13, 19)),
        ));
        fixture.arena = TypedArena::try_new(None, nodes).expect("predicate arena");
        fixture.input.formulas[0].kind = SourceAtomicFormulaKind::PredicateApplication;
        fixture.input.formulas[0].spelling = "1 divides 2".to_owned();
        fixture
            .input
            .predicate_heads
            .push(SourcePredicateHeadInput {
                formula: SourceAtomicFormulaId::new(0),
                site: node(6),
                source_range: range(fixture.source, 13, 19),
                context: BindingContextId::new(0),
                recovery: SourceAtomicFormulaRecovery::Normal,
                spelling: "divides".to_owned(),
                left_arity: 1,
                right_arity: 1,
            });
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            fixture.module.clone(),
            ContributionKind::LocalSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 1, 2)),
        );
        fixture
            .input
            .candidates
            .push(SourcePredicateCandidateInput {
                head: SourcePredicateHeadId::new(0),
                ordinal: 0,
                symbol: SymbolId::new(
                    fixture.module.clone(),
                    LocalSymbolId::new("divides/test"),
                    FullyQualifiedName::new("atomic.fixture::divides"),
                ),
                contribution,
            });
        fixture.input.edges[0].role = SourceAtomicEdgeRole::PredicateLeftArgument;
        fixture.input.edges[1].role = SourceAtomicEdgeRole::PredicateRightArgument;
        fixture.input.requests = vec![SourceAtomicRequestInput {
            formula: SourceAtomicFormulaId::new(0),
            ordinal: 0,
            kind: SourceAtomicRequestKind::PredicateCandidateSignature,
            edge: None,
            candidate: Some(SourcePredicateCandidateId::new(0)),
            type_site: None,
            attribute: None,
        }];
        validate_payload(
            &fixture.input,
            Some(&fixture.bindings),
            None,
            &fixture.primary,
            None,
            None,
            None,
            &fixture.arena,
        )
        .expect("synthetic predicate shape");

        fixture.input.candidates.clear();
        assert!(matches!(
            validate_payload(
                &fixture.input,
                Some(&fixture.bindings),
                None,
                &fixture.primary,
                None,
                None,
                None,
                &fixture.arena,
            ),
            Err(SourceAtomicFormulaError::InvalidPredicateHead { .. })
        ));
    }

    #[test]
    fn negative_attribute_owns_distinct_target_and_non_anchors() {
        let mut fixture = negative_attribute_fixture();
        let contribution = fixture.input.attributes[0].contribution;
        validate_payload(
            &fixture.input,
            Some(&fixture.bindings),
            None,
            &fixture.primary,
            None,
            None,
            None,
            &fixture.arena,
        )
        .expect("synthetic negative attribute");

        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        nodes.push(TypedNode::new(
            "source.formula.atomic.attribute",
            SourceAnchor::Range(range(fixture.source, 22, 24)),
        ));
        nodes.push(TypedNode::new(
            "source.formula.atomic.attribute-target",
            SourceAnchor::Range(range(fixture.source, 22, 24)),
        ));
        fixture.arena = TypedArena::try_new(None, nodes).expect("multiple attribute arena");
        fixture.input.formulas[0].spelling = "1 is non empty finite".to_owned();
        fixture
            .input
            .attributes
            .push(SourceAssertionAttributeInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 1,
                site: node(9),
                source_range: range(fixture.source, 22, 24),
                spelling: "finite".to_owned(),
                target_site: node(10),
                target_range: range(fixture.source, 22, 24),
                target_spelling: "finite".to_owned(),
                context: BindingContextId::new(0),
                recovery: SourceAtomicFormulaRecovery::Normal,
                symbol: SymbolId::new(
                    fixture.module.clone(),
                    LocalSymbolId::new("finite/test"),
                    FullyQualifiedName::new("atomic.fixture::finite"),
                ),
                contribution,
                polarity: SourceAssertionAttributePolarityInput::Positive,
            });
        fixture.input.requests.push(SourceAtomicRequestInput {
            formula: SourceAtomicFormulaId::new(0),
            ordinal: 1,
            kind: SourceAtomicRequestKind::AttributeAdmissibility,
            edge: None,
            candidate: None,
            type_site: None,
            attribute: Some(SourceAssertionAttributeId::new(1)),
        });
        validate_payload(
            &fixture.input,
            Some(&fixture.bindings),
            None,
            &fixture.primary,
            None,
            None,
            None,
            &fixture.arena,
        )
        .expect("multiple simple attributes");

        let SourceAssertionAttributePolarityInput::Negative { non_spelling, .. } =
            &mut fixture.input.attributes[0].polarity
        else {
            unreachable!()
        };
        *non_spelling = "not".to_owned();
        assert!(matches!(
            validate_payload(
                &fixture.input,
                Some(&fixture.bindings),
                None,
                &fixture.primary,
                None,
                None,
                None,
                &fixture.arena,
            ),
            Err(SourceAtomicFormulaError::InvalidAttribute { .. })
        ));
    }

    #[test]
    fn authenticated_condition_container_is_validation_only_and_debug_stable() {
        let fixture = exact_condition_container_fixture();
        let set_terms = fixture
            .build_set(fixture.set_input.clone())
            .expect("exact conditioned comprehension");
        let without_set = fixture
            .build_atomic(fixture.atomic_input.clone(), None)
            .expect("family-local exact equality");
        let with_set = fixture
            .build_atomic(fixture.atomic_input.clone(), Some(&set_terms))
            .expect("authenticated condition container");

        assert_eq!(without_set, with_set);
        assert_eq!(without_set.debug_text(), with_set.debug_text());
        assert_eq!(with_set.set_term_fingerprint(), None);
        assert_eq!(
            (
                fixture.primary.terms().len(),
                fixture.primary.references().len(),
                fixture.primary.numeric_type_requests().len(),
            ),
            (4, 0, 4)
        );
        assert_eq!(
            (
                fixture.application.applications().len(),
                fixture.application.wrappers().len(),
                fixture.application.candidates().len(),
                fixture.application.arguments().len(),
                fixture.application.type_requests().len(),
            ),
            (1, 0, 1, 2, 2)
        );
        assert_eq!(
            (
                set_terms.terms().len(),
                set_terms.wrappers().len(),
                set_terms.generators().len(),
                set_terms.type_sites().len(),
                set_terms.conditions().len(),
                set_terms.edges().len(),
                set_terms.requests().len(),
            ),
            (1, 0, 1, 1, 1, 1, 2)
        );
        assert_eq!(
            (
                with_set.formulas().len(),
                with_set.wrappers().len(),
                with_set.predicate_segments().len(),
                with_set.predicate_heads().len(),
                with_set.candidates().len(),
                with_set.type_sites().len(),
                with_set.attributes().len(),
                with_set.edges().len(),
                with_set.requests().len(),
            ),
            (1, 0, 0, 0, 0, 0, 0, 2, 2)
        );
    }

    #[test]
    fn authenticated_condition_container_installs_in_both_orders_atomically() {
        let fixture = exact_condition_container_fixture();
        let set_terms = fixture
            .build_set(fixture.set_input.clone())
            .expect("exact conditioned comprehension");
        let atomic = fixture
            .build_atomic(fixture.atomic_input.clone(), None)
            .expect("family-local exact equality");

        let set_then_atomic = fixture
            .typed_ast()
            .with_source_set_term(set_terms.clone())
            .expect("set-first install")
            .with_source_atomic_formula(atomic.clone())
            .expect("atomic-after-set install");
        let atomic_then_set = fixture
            .typed_ast()
            .with_source_atomic_formula(atomic.clone())
            .expect("atomic-first install")
            .with_source_set_term(set_terms.clone())
            .expect("set-after-atomic install");
        assert_eq!(set_then_atomic, atomic_then_set);
        assert_eq!(
            set_then_atomic.debug_text(),
            atomic_then_set.debug_text(),
            "installation order must not affect immutable output"
        );
        assert_eq!(set_then_atomic.source_atomic_formula(), Some(&atomic));
        assert_eq!(set_then_atomic.source_set_term(), Some(&set_terms));

        let mut substituted_input = fixture.set_input.clone();
        substituted_input.terms[0].spelling =
            "{ 1 ++ 2 where candidate255c is set : 4 = 3 }".to_owned();
        substituted_input.conditions[0].spelling = "4 = 3".to_owned();
        let substituted = fixture
            .build_set(substituted_input)
            .expect("family-local substituted validation context");
        assert_eq!(
            fixture
                .build_atomic(fixture.atomic_input.clone(), Some(&substituted))
                .expect_err("substituted optional validation context"),
            SourceAtomicFormulaError::SetTermDependencyMismatch
        );
        let substituted_first = fixture
            .typed_ast()
            .with_source_set_term(substituted.clone())
            .expect("substituted set installs independently");
        assert_eq!(
            substituted_first
                .clone()
                .with_source_atomic_formula(atomic.clone())
                .expect_err("substituted optional set context must reject"),
            TypedAstError::InvalidSourceAtomicFormula
        );
        assert_eq!(substituted_first.source_atomic_formula(), None);
        let valid_after_atomic_failure = fixture
            .typed_ast()
            .with_source_set_term(set_terms.clone())
            .expect("valid set replay")
            .with_source_atomic_formula(atomic.clone())
            .expect("valid atomic replay");

        let atomic_first = fixture
            .typed_ast()
            .with_source_atomic_formula(atomic.clone())
            .expect("family-local atomic install");
        assert_eq!(atomic_first.source_set_term(), None);
        assert_eq!(
            atomic_first
                .clone()
                .with_source_set_term(substituted)
                .expect_err("substituted set must fail revalidation"),
            TypedAstError::InvalidSourceSetTerm
        );
        assert_eq!(atomic_first.source_set_term(), None);
        let valid_after_set_failure = atomic_first
            .with_source_set_term(set_terms)
            .expect("valid set replay after rollback");
        assert_eq!(valid_after_atomic_failure, valid_after_set_failure);
        assert_eq!(atomic.set_term_fingerprint(), None);
    }

    #[test]
    fn condition_container_corruption_and_preservation_matrix_is_fail_closed() {
        let mut wrong_kind = ConditionContainerOptions::exact();
        wrong_kind.formula_kind = SourceAtomicFormulaKind::Inequality;
        wrong_kind.formula_spelling = "3 <> 4";
        let mut wrong_range = ConditionContainerOptions::exact();
        wrong_range.formula_range = (178, 182);
        wrong_range.operand_ranges[0] = (178, 179);
        let mut wrong_recovery = ConditionContainerOptions::exact();
        wrong_recovery.formula_recovery = SourceAtomicFormulaRecovery::Degraded;
        let mut non_direct = ConditionContainerOptions::exact();
        non_direct.direct_condition_child = false;
        let mut partial_crossing = ConditionContainerOptions::exact();
        partial_crossing.formula_range = (183, 187);
        partial_crossing.operand_ranges = [(184, 185), (186, 187)];
        partial_crossing.direct_condition_child = false;

        for (label, options) in [
            ("wrong kind", wrong_kind),
            ("wrong range", wrong_range),
            ("wrong recovery", wrong_recovery),
            ("non-direct relation", non_direct),
            ("partial crossing overlap", partial_crossing),
        ] {
            let fixture = condition_container_fixture(options);
            let set_terms = fixture
                .build_set(fixture.set_input.clone())
                .unwrap_or_else(|error| {
                    panic!("{label}: Task-255 family-local failure: {error:?}")
                });
            fixture
                .build_atomic(fixture.atomic_input.clone(), None)
                .unwrap_or_else(|error| {
                    panic!("{label}: Task-256 family-local failure: {error:?}")
                });
            assert_eq!(
                fixture
                    .build_atomic(fixture.atomic_input.clone(), Some(&set_terms))
                    .expect_err(label),
                SourceAtomicFormulaError::SetTermDependencyMismatch,
                "{label}: the independently valid pair must fail only at C1"
            );
        }

        let fixture = exact_condition_container_fixture();
        let exact_set = fixture
            .build_set(fixture.set_input.clone())
            .expect("exact set family");
        let exact_atomic = fixture
            .build_atomic(fixture.atomic_input.clone(), None)
            .expect("exact atomic family");
        let mut wrong_context_options = ConditionContainerOptions::exact();
        wrong_context_options.formula_context = BindingContextId::new(1);
        let wrong_context = condition_container_fixture(wrong_context_options);
        wrong_context
            .build_atomic(wrong_context.atomic_input.clone(), None)
            .expect("wrong-context Task-256 row is independently valid");
        assert_eq!(
            wrong_context
                .build_set(wrong_context.set_input.clone())
                .expect_err("Task-255 rejects condition children outside its owner context"),
            SourceSetTermError::PrimaryDependencyMismatch,
            "wrong context is not an applicable two-family-local near miss"
        );
        let mut wrong_context_input = fixture.atomic_input.clone();
        wrong_context_input.formulas[0].context = BindingContextId::new(1);
        assert_eq!(
            fixture
                .build_atomic(wrong_context_input, Some(&exact_set))
                .expect_err("wrong owner/formula context"),
            SourceAtomicFormulaError::SetTermDependencyMismatch
        );

        let mut wrong_spelling_input = fixture.set_input.clone();
        wrong_spelling_input.terms[0].spelling =
            "{ 1 ++ 2 where candidate255c is set : 4 = 3 }".to_owned();
        wrong_spelling_input.conditions[0].spelling = "4 = 3".to_owned();
        let wrong_spelling = fixture
            .build_set(wrong_spelling_input)
            .expect("wrong-spelling set remains family-local valid");
        assert_eq!(
            fixture
                .build_atomic(fixture.atomic_input.clone(), Some(&wrong_spelling))
                .expect_err("wrong spelling"),
            SourceAtomicFormulaError::SetTermDependencyMismatch
        );

        let mut copied_options = ConditionContainerOptions::exact();
        copied_options.source_ordinal = 2;
        let copied_fixture = condition_container_fixture(copied_options);
        let copied_set = copied_fixture
            .build_set(copied_fixture.set_input.clone())
            .expect("cross-source copied set is locally valid");
        assert_eq!(
            fixture
                .build_atomic(fixture.atomic_input.clone(), Some(&copied_set))
                .expect_err("cross-source copied set"),
            SourceAtomicFormulaError::SetTermDependencyMismatch
        );

        let mut stale = exact_condition_container_fixture();
        let stale_set = stale
            .build_set(stale.set_input.clone())
            .expect("pre-mutation set handoff");
        let mut stale_nodes = stale
            .arena
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        stale_nodes[11].children.clear();
        stale.arena = TypedArena::try_new(None, stale_nodes).expect("stale direct-edge arena");
        stale_set
            .validate_installation(
                stale.source,
                &stale.module,
                &stale.primary,
                Some(&stale.application),
                None,
                &stale.arena,
            )
            .expect("Task-255 handoff remains family-local valid in the stale arena");
        stale
            .build_atomic(stale.atomic_input.clone(), None)
            .expect("Task-256 handoff remains family-local valid in the stale arena");
        assert_eq!(
            stale
                .build_atomic(stale.atomic_input.clone(), Some(&stale_set))
                .expect_err("stale direct-child relationship"),
            SourceAtomicFormulaError::SetTermDependencyMismatch
        );

        let mut wrapped = exact_condition_container_fixture();
        let mut wrapped_nodes = wrapped
            .arena
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        wrapped_nodes.push(TypedNode::new(
            "source.formula.atomic.parenthesized",
            SourceAnchor::Range(range(wrapped.source, 176, 183)),
        ));
        wrapped.arena = TypedArena::try_new(None, wrapped_nodes).expect("wrapped condition arena");
        wrapped.atomic_input.wrappers = vec![SourceAtomicWrapperInput {
            formula: SourceAtomicFormulaId::new(0),
            ordinal: 0,
            site: node(13),
            source_range: range(wrapped.source, 176, 183),
            context: BindingContextId::new(0),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "( 3 = 4 )".to_owned(),
        }];
        let wrapped_set = wrapped
            .build_set(wrapped.set_input.clone())
            .expect("wrapped near-miss set family");
        wrapped
            .build_atomic(wrapped.atomic_input.clone(), None)
            .expect("wrapped near-miss atomic family");
        assert_eq!(
            wrapped
                .build_atomic(wrapped.atomic_input.clone(), Some(&wrapped_set))
                .expect_err("effective wrapper must not widen the exact relation"),
            SourceAtomicFormulaError::SetTermDependencyMismatch
        );

        let mut wrong_owner = exact_condition_container_fixture();
        let mut wrong_owner_nodes = wrong_owner
            .arena
            .iter()
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        wrong_owner_nodes.push(
            TypedNode::new(
                "source.term.set.comprehension",
                SourceAnchor::Range(range(wrong_owner.source, 130, 220)),
            )
            .with_children(vec![node(12).node(), node(14).node()]),
        );
        wrong_owner_nodes.push(TypedNode::new(
            "source.term.set.comprehension-generator",
            SourceAnchor::Range(range(wrong_owner.source, 190, 204)),
        ));
        wrong_owner_nodes.push(TypedNode::new(
            "source.term.set.target-type",
            SourceAnchor::Range(range(wrong_owner.source, 208, 211)),
        ));
        wrong_owner_nodes.push(TypedNode::new(
            "source.term.set.target-type-head",
            SourceAnchor::Range(range(wrong_owner.source, 208, 211)),
        ));
        wrong_owner.arena =
            TypedArena::try_new(None, wrong_owner_nodes).expect("wrong-owner nested arena");
        wrong_owner.set_input.terms[0].source_ordinal = 1;
        wrong_owner.set_input.terms.insert(
            0,
            SourceSetTermInput {
                site: node(13),
                source_range: range(wrong_owner.source, 130, 220),
                source_ordinal: 0,
                context: BindingContextId::new(0),
                recovery: SourceSetTermRecovery::Normal,
                spelling:
                    "{ { 1 ++ 2 where candidate255c is set : 3 = 4 } where outercandidate is set }"
                        .to_owned(),
                kind: SourceSetTermKind::Comprehension,
            },
        );
        wrong_owner.set_input.generators[0].term = SourceSetTermId::new(1);
        wrong_owner.set_input.generators.insert(
            0,
            SourceSetGeneratorInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                site: node(14),
                source_range: range(wrong_owner.source, 190, 204),
                spelling: "outercandidate".to_owned(),
                context: BindingContextId::new(0),
                recovery: SourceSetTermRecovery::Normal,
                type_site: SourceSetTypeSiteId::new(1),
            },
        );
        wrong_owner.set_input.type_sites[0].owner =
            SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(1));
        wrong_owner
            .set_input
            .type_sites
            .push(SourceSetTypeSiteInput {
                owner: SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(0)),
                site: node(15),
                source_range: range(wrong_owner.source, 208, 211),
                spelling: "set".to_owned(),
                head_site: node(16),
                head_range: range(wrong_owner.source, 208, 211),
                head_spelling: "set".to_owned(),
                context: BindingContextId::new(0),
                recovery: SourceSetTermRecovery::Normal,
                head: SourceSetTypeHead::BuiltinSet,
            });
        wrong_owner.set_input.conditions[0].term = SourceSetTermId::new(1);
        wrong_owner.set_input.edges[0].term = SourceSetTermId::new(1);
        wrong_owner.set_input.edges.insert(
            0,
            SourceSetEdgeInput {
                term: SourceSetTermId::new(0),
                ordinal: 0,
                role: SourceSetEdgeRole::ComprehensionMapper,
                target: SourceSetTarget::SetTerm(SourceSetTermId::new(1)),
            },
        );
        wrong_owner.set_input.requests[0].generator = Some(SourceSetGeneratorId::new(1));
        for request in &mut wrong_owner.set_input.requests {
            request.term = SourceSetTermId::new(1);
        }
        wrong_owner.set_input.requests.splice(
            0..0,
            [
                SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 0,
                    kind: SourceSetRequestKind::GeneratorSethood,
                    generator: Some(SourceSetGeneratorId::new(0)),
                    type_site: Some(SourceSetTypeSiteId::new(1)),
                },
                SourceSetRequestInput {
                    term: SourceSetTermId::new(0),
                    ordinal: 1,
                    kind: SourceSetRequestKind::ResultType,
                    generator: None,
                    type_site: None,
                },
            ],
        );
        wrong_owner.set_input.requests[2].type_site = Some(SourceSetTypeSiteId::new(0));
        let wrong_owner_set = wrong_owner
            .build_set(wrong_owner.set_input.clone())
            .expect("nested wrong-owner set family");
        wrong_owner
            .build_atomic(wrong_owner.atomic_input.clone(), None)
            .expect("nested wrong-owner atomic family");
        assert_eq!(
            wrong_owner
                .build_atomic(wrong_owner.atomic_input.clone(), Some(&wrong_owner_set),)
                .expect_err("overlapping outer term does not own the matching condition"),
            SourceAtomicFormulaError::SetTermDependencyMismatch
        );

        let mut disjoint_options = ConditionContainerOptions::exact();
        disjoint_options.formula_range = (190, 195);
        disjoint_options.operand_ranges = [(190, 191), (194, 195)];
        disjoint_options.direct_condition_child = false;
        let disjoint = condition_container_fixture(disjoint_options);
        let disjoint_set = disjoint
            .build_set(disjoint.set_input.clone())
            .expect("disjoint set family");
        disjoint
            .build_atomic(disjoint.atomic_input.clone(), None)
            .expect("disjoint atomic family");
        disjoint
            .build_atomic(disjoint.atomic_input.clone(), Some(&disjoint_set))
            .expect("disjoint optional set remains dependency-neutral");

        let mut containing_input = fixture.atomic_input.clone();
        containing_input.formulas[0].source_range = range(fixture.source, 130, 190);
        containing_input.formulas[0].spelling =
            "{ 1 ++ 2 where candidate255c is set : 3 = 4 } = 4".to_owned();
        let containing_effective = vec![EffectiveOccurrence {
            range: containing_input.formulas[0].source_range,
        }];
        validate_cross_family_ranges(
            &containing_input,
            &containing_effective,
            &fixture.primary,
            Some(&fixture.application),
            None,
            Some(&exact_set),
            &fixture.arena,
        )
        .expect("the pre-existing formula-contains-set range rule is preserved");

        let mut arbitrary_options = ConditionContainerOptions::exact();
        arbitrary_options.formula_range = (139, 184);
        arbitrary_options.direct_condition_child = false;
        let arbitrary = condition_container_fixture(arbitrary_options);
        let arbitrary_set = arbitrary
            .build_set(arbitrary.set_input.clone())
            .expect("arbitrary equal-range set family");
        assert_eq!(
            arbitrary
                .build_atomic(arbitrary.atomic_input.clone(), Some(&arbitrary_set))
                .expect_err("arbitrary equal-range overlap"),
            SourceAtomicFormulaError::SetTermDependencyMismatch
        );
        assert!(
            arbitrary
                .build_atomic(arbitrary.atomic_input.clone(), None)
                .is_err(),
            "equal-range formula is not an applicable family-local near miss"
        );
        assert_eq!(exact_atomic.set_term_fingerprint(), None);
    }
}
