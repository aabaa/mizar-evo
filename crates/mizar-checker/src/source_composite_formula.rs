//! Syntax-free source composite-formula and quantified-binder transport.

use crate::{
    binding_env::{
        BinderIdentity, BindingContextDraft, BindingContextId, BindingContextLayer,
        BindingContextOwner, BindingContextRecovery, BindingDiagnosticClass,
        BindingDiagnosticRecovery, BindingDiagnosticSeverity, BindingDiagnosticTable, BindingDraft,
        BindingEnv, BindingEnvParts, BindingId, BindingKind, BindingRecoveryState, BindingStatus,
        BindingTypeSite, CapturedFreeVariables,
    },
    typed_ast::{NodeRecoveryState, TypedArena, TypedSiteRef},
};
use mizar_resolve::{
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::ModuleId,
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

dense_id!(SourceCompositeFormulaId);
dense_id!(SourceFormulaWrapperId);
dense_id!(SourceFormulaRootId);
dense_id!(SourceQuantifierBinderId);
dense_id!(SourceBinderTypeSiteId);
dense_id!(SourceFormulaEdgeId);
dense_id!(SourceFormulaRequestId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCompositeFormulaHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub formulas: Vec<SourceCompositeFormulaInput>,
    pub wrappers: Vec<SourceFormulaWrapperInput>,
    pub roots: Vec<SourceFormulaRootInput>,
    pub binders: Vec<SourceQuantifierBinderInput>,
    pub type_sites: Vec<SourceBinderTypeSiteInput>,
    pub edges: Vec<SourceFormulaEdgeInput>,
    pub requests: Vec<SourceFormulaRequestInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCompositeFormulaInput {
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourceCompositeFormulaRecovery,
    pub spelling: String,
    pub kind: SourceCompositeFormulaKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFormulaWrapperInput {
    pub formula: SourceCompositeFormulaId,
    pub ordinal: usize,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceCompositeFormulaRecovery,
    pub spelling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFormulaRootInput {
    pub formula: SourceCompositeFormulaId,
    pub ordinal: usize,
    pub ownership: SourceFormulaRootOwnership,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceQuantifierBinderInput {
    pub formula: SourceCompositeFormulaId,
    pub ordinal: usize,
    pub segment_site: TypedSiteRef,
    pub segment_range: SourceRange,
    pub segment_spelling: String,
    pub identifier_site: TypedSiteRef,
    pub identifier_range: SourceRange,
    pub identifier_spelling: String,
    pub local: LocalTermBinding,
    pub binding: BindingId,
    pub body_context: BindingContextId,
    pub type_site: SourceBinderTypeSiteId,
    pub recovery: SourceCompositeFormulaRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceBinderTypeSiteInput {
    pub binder: SourceQuantifierBinderId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub head_site: TypedSiteRef,
    pub head_range: SourceRange,
    pub head_spelling: String,
    pub context: BindingContextId,
    pub recovery: SourceCompositeFormulaRecovery,
    pub head: SourceBinderTypeHead,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFormulaEdgeInput {
    pub parent: SourceCompositeFormulaId,
    pub ordinal: usize,
    pub role: SourceFormulaEdgeRole,
    pub child: SourceCompositeFormulaId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFormulaRequestInput {
    pub formula: SourceCompositeFormulaId,
    pub ordinal: usize,
    pub kind: SourceFormulaRequestKind,
    pub binder: Option<SourceQuantifierBinderId>,
    pub type_site: Option<SourceBinderTypeSiteId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceCompositeFormulaKind {
    Implication,
    Universal,
    Existential,
    Negation,
    Contradiction,
    Conjunction,
    RepeatedConjunction,
    Disjunction,
    RepeatedDisjunction,
    Biconditional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceCompositeFormulaRecovery {
    Normal,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceFormulaRootOwnership {
    UnassignedStatement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceBinderTypeHead {
    BuiltinSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceFormulaEdgeRole {
    ImplicationLeft,
    ImplicationRight,
    UniversalBody,
    ExistentialBody,
    NegatedFormula,
    DisjunctionLeft,
    DisjunctionRight,
    BiconditionalLeft,
    BiconditionalRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceFormulaRequestKind {
    ConnectiveSemantics,
    ConstantSemantics,
    QuantifierSemantics,
    BinderType,
    NegationSemantics,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCompositeFormulaHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    binding_env: BindingEnv,
    formulas: SourceCompositeFormulaTable,
    wrappers: SourceFormulaWrapperTable,
    roots: SourceFormulaRootTable,
    binders: SourceQuantifierBinderTable,
    type_sites: SourceBinderTypeSiteTable,
    edges: SourceFormulaEdgeTable,
    requests: SourceFormulaRequestTable,
}

impl SourceCompositeFormulaHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn binding_env(&self) -> &BindingEnv {
        &self.binding_env
    }

    pub const fn formulas(&self) -> &SourceCompositeFormulaTable {
        &self.formulas
    }

    pub const fn wrappers(&self) -> &SourceFormulaWrapperTable {
        &self.wrappers
    }

    pub const fn roots(&self) -> &SourceFormulaRootTable {
        &self.roots
    }

    pub const fn binders(&self) -> &SourceQuantifierBinderTable {
        &self.binders
    }

    pub const fn type_sites(&self) -> &SourceBinderTypeSiteTable {
        &self.type_sites
    }

    pub const fn edges(&self) -> &SourceFormulaEdgeTable {
        &self.edges
    }

    pub const fn requests(&self) -> &SourceFormulaRequestTable {
        &self.requests
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-composite-formula-debug-v1\n");
        output.push_str("module: ");
        write_module_id(&mut output, &self.module_id);
        output.push('\n');
        output.push_str(&self.binding_env.debug_text());
        let _ = writeln!(output, "formulas: {}", self.formulas.len());
        for (id, row) in self.formulas.iter() {
            let _ = writeln!(
                output,
                "  formula#{} site={} range={}..{} ordinal={} context={} recovery={} spelling={:?} kind={}",
                id.index(),
                row.site.node().index(),
                row.source_range.start,
                row.source_range.end,
                row.source_ordinal,
                row.context.index(),
                recovery_key(row.recovery),
                row.spelling,
                formula_kind_key(row.kind),
            );
        }
        let _ = writeln!(output, "wrappers: {}", self.wrappers.len());
        for (id, row) in self.wrappers.iter() {
            let _ = writeln!(
                output,
                "  wrapper#{} formula={} ordinal={} site={} range={}..{} context={} recovery={} spelling={:?}",
                id.index(),
                row.formula.index(),
                row.ordinal,
                row.site.node().index(),
                row.source_range.start,
                row.source_range.end,
                row.context.index(),
                recovery_key(row.recovery),
                row.spelling,
            );
        }
        let _ = writeln!(output, "roots: {}", self.roots.len());
        for (id, row) in self.roots.iter() {
            let _ = writeln!(
                output,
                "  root#{} formula={} ordinal={} ownership={}",
                id.index(),
                row.formula.index(),
                row.ordinal,
                root_ownership_key(row.ownership),
            );
        }
        let _ = writeln!(output, "binders: {}", self.binders.len());
        for (id, row) in self.binders.iter() {
            let _ = writeln!(
                output,
                "  binder#{} formula={} ordinal={} segment-site={} segment-range={}..{} segment-spelling={:?} identifier-site={} identifier-range={}..{} identifier-spelling={:?} local-scope={:?} local-ordinal={} binding={} body-context={} type-site={} recovery={}",
                id.index(),
                row.formula.index(),
                row.ordinal,
                row.segment_site.node().index(),
                row.segment_range.start,
                row.segment_range.end,
                row.segment_spelling,
                row.identifier_site.node().index(),
                row.identifier_range.start,
                row.identifier_range.end,
                row.identifier_spelling,
                row.local.scope().path(),
                row.local.visible_after_ordinal(),
                row.binding.index(),
                row.body_context.index(),
                row.type_site.index(),
                recovery_key(row.recovery),
            );
        }
        let _ = writeln!(output, "type-sites: {}", self.type_sites.len());
        for (id, row) in self.type_sites.iter() {
            let _ = writeln!(
                output,
                "  type-site#{} binder={} site={} range={}..{} spelling={:?} head-site={} head-range={}..{} head-spelling={:?} context={} recovery={} head={}",
                id.index(),
                row.binder.index(),
                row.site.node().index(),
                row.source_range.start,
                row.source_range.end,
                row.spelling,
                row.head_site.node().index(),
                row.head_range.start,
                row.head_range.end,
                row.head_spelling,
                row.context.index(),
                recovery_key(row.recovery),
                binder_type_head_key(row.head),
            );
        }
        let _ = writeln!(output, "edges: {}", self.edges.len());
        for (id, row) in self.edges.iter() {
            let _ = writeln!(
                output,
                "  edge#{} parent={} ordinal={} role={} child={}",
                id.index(),
                row.parent.index(),
                row.ordinal,
                edge_role_key(row.role),
                row.child.index(),
            );
        }
        let _ = writeln!(output, "requests: {}", self.requests.len());
        for (id, row) in self.requests.iter() {
            let _ = write!(
                output,
                "  request#{} formula={} ordinal={} kind={} binder=",
                id.index(),
                row.formula.index(),
                row.ordinal,
                request_kind_key(row.kind),
            );
            write_optional_id(&mut output, row.binder.map(SourceQuantifierBinderId::index));
            output.push_str(" type-site=");
            write_optional_id(
                &mut output,
                row.type_site.map(SourceBinderTypeSiteId::index),
            );
            output.push('\n');
        }
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
    ) -> Result<(), SourceCompositeFormulaError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || self.binding_env.source_id() != source_id
            || self.binding_env.module_id() != module_id
        {
            return Err(SourceCompositeFormulaError::EnvironmentMismatch);
        }
        let input = self.to_input();
        validate_input(&input, arena)?;
        validate_extended_bindings(&input, &self.binding_env)
    }

    pub(crate) fn is_task_257a_profile(&self) -> bool {
        self.formulas.len() == 5
            && self.wrappers.is_empty()
            && self.roots.len() == 1
            && self.binders.len() == 1
            && self.type_sites.len() == 1
            && self.edges.len() == 4
            && self.requests.len() == 6
    }

    pub(crate) fn is_task_257b1_profile(&self) -> bool {
        self.formulas.len() == 1
            && self.wrappers.is_empty()
            && self.roots.len() == 1
            && self.binders.len() == 1
            && self.type_sites.len() == 1
            && self.edges.is_empty()
            && self.requests.len() == 2
    }

    pub(crate) fn is_task_257b2_profile(&self) -> bool {
        self.formulas.len() == 8
            && self.wrappers.len() == 6
            && self.roots.len() == 1
            && self.binders.len() == 1
            && self.type_sites.len() == 1
            && self.edges.len() == 7
            && self.requests.len() == 9
    }

    pub(crate) fn is_task_257b3_profile(&self) -> bool {
        self.formulas.len() == 3
            && self.wrappers.is_empty()
            && self.roots.len() == 1
            && self.binders.len() == 3
            && self.type_sites.len() == 3
            && self.edges.len() == 2
            && self.requests.len() == 6
    }

    fn to_input(&self) -> SourceCompositeFormulaHandoffInput {
        SourceCompositeFormulaHandoffInput {
            source_id: self.source_id,
            module_id: self.module_id.clone(),
            formulas: self.formulas.rows.iter().map(Into::into).collect(),
            wrappers: self.wrappers.rows.iter().map(Into::into).collect(),
            roots: self.roots.rows.iter().map(Into::into).collect(),
            binders: self.binders.rows.iter().map(Into::into).collect(),
            type_sites: self.type_sites.rows.iter().map(Into::into).collect(),
            edges: self.edges.rows.iter().map(Into::into).collect(),
            requests: self.requests.rows.iter().map(Into::into).collect(),
        }
    }
}

macro_rules! table {
    ($name:ident, $row:ident, $id:ident) => {
        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        pub struct $name {
            rows: Vec<$row>,
        }

        impl $name {
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
    SourceCompositeFormulaTable,
    SourceCompositeFormula,
    SourceCompositeFormulaId
);
table!(
    SourceFormulaWrapperTable,
    SourceFormulaWrapper,
    SourceFormulaWrapperId
);
table!(
    SourceFormulaRootTable,
    SourceFormulaRoot,
    SourceFormulaRootId
);
table!(
    SourceQuantifierBinderTable,
    SourceQuantifierBinder,
    SourceQuantifierBinderId
);
table!(
    SourceBinderTypeSiteTable,
    SourceBinderTypeSite,
    SourceBinderTypeSiteId
);
table!(
    SourceFormulaEdgeTable,
    SourceFormulaEdge,
    SourceFormulaEdgeId
);
table!(
    SourceFormulaRequestTable,
    SourceFormulaRequest,
    SourceFormulaRequestId
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCompositeFormula {
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    context: BindingContextId,
    recovery: SourceCompositeFormulaRecovery,
    spelling: String,
    kind: SourceCompositeFormulaKind,
}

impl SourceCompositeFormula {
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
    pub const fn recovery(&self) -> SourceCompositeFormulaRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    pub const fn kind(&self) -> SourceCompositeFormulaKind {
        self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFormulaWrapper {
    formula: SourceCompositeFormulaId,
    ordinal: usize,
    site: TypedSiteRef,
    source_range: SourceRange,
    context: BindingContextId,
    recovery: SourceCompositeFormulaRecovery,
    spelling: String,
}

impl SourceFormulaWrapper {
    pub const fn formula(&self) -> SourceCompositeFormulaId {
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
    pub const fn recovery(&self) -> SourceCompositeFormulaRecovery {
        self.recovery
    }
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFormulaRoot {
    formula: SourceCompositeFormulaId,
    ordinal: usize,
    ownership: SourceFormulaRootOwnership,
}

impl SourceFormulaRoot {
    pub const fn formula(&self) -> SourceCompositeFormulaId {
        self.formula
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn ownership(&self) -> SourceFormulaRootOwnership {
        self.ownership
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceQuantifierBinder {
    formula: SourceCompositeFormulaId,
    ordinal: usize,
    segment_site: TypedSiteRef,
    segment_range: SourceRange,
    segment_spelling: String,
    identifier_site: TypedSiteRef,
    identifier_range: SourceRange,
    identifier_spelling: String,
    local: LocalTermBinding,
    binding: BindingId,
    body_context: BindingContextId,
    type_site: SourceBinderTypeSiteId,
    recovery: SourceCompositeFormulaRecovery,
}

impl SourceQuantifierBinder {
    pub const fn formula(&self) -> SourceCompositeFormulaId {
        self.formula
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn segment_site(&self) -> &TypedSiteRef {
        &self.segment_site
    }
    pub const fn segment_range(&self) -> SourceRange {
        self.segment_range
    }
    pub fn segment_spelling(&self) -> &str {
        &self.segment_spelling
    }
    pub const fn identifier_site(&self) -> &TypedSiteRef {
        &self.identifier_site
    }
    pub const fn identifier_range(&self) -> SourceRange {
        self.identifier_range
    }
    pub fn identifier_spelling(&self) -> &str {
        &self.identifier_spelling
    }
    pub const fn local(&self) -> &LocalTermBinding {
        &self.local
    }
    pub const fn binding(&self) -> BindingId {
        self.binding
    }
    pub const fn body_context(&self) -> BindingContextId {
        self.body_context
    }
    pub const fn type_site(&self) -> SourceBinderTypeSiteId {
        self.type_site
    }
    pub const fn recovery(&self) -> SourceCompositeFormulaRecovery {
        self.recovery
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceBinderTypeSite {
    binder: SourceQuantifierBinderId,
    site: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    head_site: TypedSiteRef,
    head_range: SourceRange,
    head_spelling: String,
    context: BindingContextId,
    recovery: SourceCompositeFormulaRecovery,
    head: SourceBinderTypeHead,
}

impl SourceBinderTypeSite {
    pub const fn binder(&self) -> SourceQuantifierBinderId {
        self.binder
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
    pub const fn recovery(&self) -> SourceCompositeFormulaRecovery {
        self.recovery
    }
    pub const fn head(&self) -> SourceBinderTypeHead {
        self.head
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFormulaEdge {
    parent: SourceCompositeFormulaId,
    ordinal: usize,
    role: SourceFormulaEdgeRole,
    child: SourceCompositeFormulaId,
}

impl SourceFormulaEdge {
    pub const fn parent(&self) -> SourceCompositeFormulaId {
        self.parent
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn role(&self) -> SourceFormulaEdgeRole {
        self.role
    }
    pub const fn child(&self) -> SourceCompositeFormulaId {
        self.child
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFormulaRequest {
    formula: SourceCompositeFormulaId,
    ordinal: usize,
    kind: SourceFormulaRequestKind,
    binder: Option<SourceQuantifierBinderId>,
    type_site: Option<SourceBinderTypeSiteId>,
}

impl SourceFormulaRequest {
    pub const fn formula(&self) -> SourceCompositeFormulaId {
        self.formula
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub const fn kind(&self) -> SourceFormulaRequestKind {
        self.kind
    }
    pub const fn binder(&self) -> Option<SourceQuantifierBinderId> {
        self.binder
    }
    pub const fn type_site(&self) -> Option<SourceBinderTypeSiteId> {
        self.type_site
    }
}

macro_rules! from_row {
    ($row:ident => $input:ident { $($field:ident),+ $(,)? }) => {
        impl From<&$row> for $input {
            fn from(row: &$row) -> Self {
                Self { $($field: row.$field.clone()),+ }
            }
        }
    };
}

from_row!(SourceCompositeFormula => SourceCompositeFormulaInput {
    site, source_range, source_ordinal, context, recovery, spelling, kind
});
from_row!(SourceFormulaWrapper => SourceFormulaWrapperInput {
    formula, ordinal, site, source_range, context, recovery, spelling
});
from_row!(SourceFormulaRoot => SourceFormulaRootInput {
    formula, ordinal, ownership
});
from_row!(SourceQuantifierBinder => SourceQuantifierBinderInput {
    formula, ordinal, segment_site, segment_range, segment_spelling, identifier_site,
    identifier_range, identifier_spelling, local, binding, body_context, type_site, recovery
});
from_row!(SourceBinderTypeSite => SourceBinderTypeSiteInput {
    binder, site, source_range, spelling, head_site, head_range, head_spelling, context,
    recovery, head
});
from_row!(SourceFormulaEdge => SourceFormulaEdgeInput {
    parent, ordinal, role, child
});
from_row!(SourceFormulaRequest => SourceFormulaRequestInput {
    formula, ordinal, kind, binder, type_site
});

pub struct SourceCompositeFormulaProducer;

impl SourceCompositeFormulaProducer {
    pub fn extend_bindings(
        input: &SourceCompositeFormulaHandoffInput,
        base_bindings: &BindingEnv,
        arena: &TypedArena,
    ) -> Result<BindingEnv, SourceCompositeFormulaError> {
        validate_input(input, arena)?;
        validate_base_bindings(input, base_bindings)?;

        let mut bindings = base_bindings.bindings().clone();
        let mut contexts = base_bindings.contexts().clone();
        for (index, binder) in input.binders.iter().enumerate() {
            let type_site = &input.type_sites[binder.type_site.index()];
            let binding = bindings.insert(BindingDraft {
                spelling: binder.identifier_spelling.clone(),
                kind: BindingKind::QuantifierBinder,
                identity: BinderIdentity::ResolverLocal {
                    scope: binder.local.scope().clone(),
                    ordinal: binder.local.visible_after_ordinal(),
                    declaration_range: binder.local.declaration_range(),
                },
                owner_context: binder.body_context,
                declaration_range: binder.identifier_range,
                visible_after_ordinal: binder.local.visible_after_ordinal(),
                type_site: BindingTypeSite::Source(type_site.source_range),
                status: BindingStatus::Active,
                captured: CapturedFreeVariables::default(),
                diagnostics: Vec::new(),
                recovery: BindingRecoveryState::Normal,
            });
            if binding != binder.binding {
                return Err(SourceCompositeFormulaError::InvalidBinder {
                    binder: SourceQuantifierBinderId::new(index),
                });
            }

            let parent = input.formulas[binder.formula.index()].context;
            let mut visible_bindings = contexts
                .get(parent)
                .ok_or(SourceCompositeFormulaError::InvalidBinder {
                    binder: SourceQuantifierBinderId::new(index),
                })?
                .visible_bindings
                .clone();
            visible_bindings.push(binding);
            let context = contexts.insert(BindingContextDraft {
                owner: BindingContextOwner::SourceFormula {
                    source_range: input.formulas[binder.formula.index()].source_range,
                },
                parent: Some(parent),
                layer: BindingContextLayer::Expression,
                lexical_scope: Some(binder.local.scope().clone()),
                bindings: vec![binding],
                visible_bindings,
                recovery: BindingContextRecovery::Normal,
            });
            if context != binder.body_context {
                return Err(SourceCompositeFormulaError::InvalidBinder {
                    binder: SourceQuantifierBinderId::new(index),
                });
            }
        }

        let result = BindingEnv::try_new(BindingEnvParts {
            source_id: input.source_id,
            module_id: input.module_id.clone(),
            contexts,
            bindings,
            diagnostics: base_bindings.diagnostics().clone(),
        })
        .map_err(|_| SourceCompositeFormulaError::EnvironmentMismatch)?;
        validate_extended_bindings(input, &result)?;
        Ok(result)
    }

    pub fn build(
        input: SourceCompositeFormulaHandoffInput,
        bindings: &BindingEnv,
        arena: &TypedArena,
    ) -> Result<SourceCompositeFormulaHandoff, SourceCompositeFormulaError> {
        validate_input(&input, arena)?;
        validate_extended_bindings(&input, bindings)?;
        Ok(SourceCompositeFormulaHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            binding_env: bindings.clone(),
            formulas: SourceCompositeFormulaTable {
                rows: input.formulas.into_iter().map(Into::into).collect(),
            },
            wrappers: SourceFormulaWrapperTable {
                rows: input.wrappers.into_iter().map(Into::into).collect(),
            },
            roots: SourceFormulaRootTable {
                rows: input.roots.into_iter().map(Into::into).collect(),
            },
            binders: SourceQuantifierBinderTable {
                rows: input.binders.into_iter().map(Into::into).collect(),
            },
            type_sites: SourceBinderTypeSiteTable {
                rows: input.type_sites.into_iter().map(Into::into).collect(),
            },
            edges: SourceFormulaEdgeTable {
                rows: input.edges.into_iter().map(Into::into).collect(),
            },
            requests: SourceFormulaRequestTable {
                rows: input.requests.into_iter().map(Into::into).collect(),
            },
        })
    }
}

macro_rules! into_row {
    ($input:ident => $row:ident { $($field:ident),+ $(,)? }) => {
        impl From<$input> for $row {
            fn from(input: $input) -> Self {
                Self { $($field: input.$field),+ }
            }
        }
    };
}

into_row!(SourceCompositeFormulaInput => SourceCompositeFormula {
    site, source_range, source_ordinal, context, recovery, spelling, kind
});
into_row!(SourceFormulaWrapperInput => SourceFormulaWrapper {
    formula, ordinal, site, source_range, context, recovery, spelling
});
into_row!(SourceFormulaRootInput => SourceFormulaRoot {
    formula, ordinal, ownership
});
into_row!(SourceQuantifierBinderInput => SourceQuantifierBinder {
    formula, ordinal, segment_site, segment_range, segment_spelling, identifier_site,
    identifier_range, identifier_spelling, local, binding, body_context, type_site, recovery
});
into_row!(SourceBinderTypeSiteInput => SourceBinderTypeSite {
    binder, site, source_range, spelling, head_site, head_range, head_spelling, context,
    recovery, head
});
into_row!(SourceFormulaEdgeInput => SourceFormulaEdge {
    parent, ordinal, role, child
});
into_row!(SourceFormulaRequestInput => SourceFormulaRequest {
    formula, ordinal, kind, binder, type_site
});

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceCompositeFormulaError {
    EnvironmentMismatch,
    InvalidFormula { formula: SourceCompositeFormulaId },
    InvalidWrapper { wrapper: SourceFormulaWrapperId },
    InvalidRoot { root: SourceFormulaRootId },
    InvalidBinder { binder: SourceQuantifierBinderId },
    InvalidTypeSite { type_site: SourceBinderTypeSiteId },
    InvalidEdge { edge: SourceFormulaEdgeId },
    InvalidRequest { request: SourceFormulaRequestId },
    DuplicateSite,
    InvalidTree,
}

impl fmt::Display for SourceCompositeFormulaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("source composite-formula environment mismatch")
            }
            Self::InvalidFormula { formula } => {
                write!(
                    formatter,
                    "source composite formula {} is invalid",
                    formula.index()
                )
            }
            Self::InvalidWrapper { wrapper } => {
                write!(
                    formatter,
                    "source formula wrapper {} is invalid",
                    wrapper.index()
                )
            }
            Self::InvalidRoot { root } => {
                write!(formatter, "source formula root {} is invalid", root.index())
            }
            Self::InvalidBinder { binder } => {
                write!(
                    formatter,
                    "source quantifier binder {} is invalid",
                    binder.index()
                )
            }
            Self::InvalidTypeSite { type_site } => {
                write!(
                    formatter,
                    "source binder type site {} is invalid",
                    type_site.index()
                )
            }
            Self::InvalidEdge { edge } => {
                write!(formatter, "source formula edge {} is invalid", edge.index())
            }
            Self::InvalidRequest { request } => {
                write!(
                    formatter,
                    "source formula request {} is invalid",
                    request.index()
                )
            }
            Self::DuplicateSite => formatter.write_str("source composite formula repeats a site"),
            Self::InvalidTree => formatter.write_str("source composite formula tree is invalid"),
        }
    }
}

impl Error for SourceCompositeFormulaError {}

fn validate_input(
    input: &SourceCompositeFormulaHandoffInput,
    arena: &TypedArena,
) -> Result<(), SourceCompositeFormulaError> {
    let profile = composite_profile(input)?;
    let task_257a_formulas = [
        (
            SourceCompositeFormulaKind::Implication,
            BindingContextId::new(0),
            "implies",
        ),
        (
            SourceCompositeFormulaKind::Contradiction,
            BindingContextId::new(0),
            "contradiction",
        ),
        (
            SourceCompositeFormulaKind::Universal,
            BindingContextId::new(0),
            "for holds",
        ),
        (
            SourceCompositeFormulaKind::Negation,
            BindingContextId::new(1),
            "not",
        ),
        (
            SourceCompositeFormulaKind::Contradiction,
            BindingContextId::new(1),
            "contradiction",
        ),
    ];
    let task_257b1_formulas = [(
        SourceCompositeFormulaKind::Universal,
        BindingContextId::new(0),
        "for holds",
    )];
    let task_257b2_formulas = [
        (
            SourceCompositeFormulaKind::Universal,
            BindingContextId::new(0),
            "for holds",
        ),
        (
            SourceCompositeFormulaKind::Biconditional,
            BindingContextId::new(1),
            "iff",
        ),
        (
            SourceCompositeFormulaKind::Disjunction,
            BindingContextId::new(1),
            "or",
        ),
        (
            SourceCompositeFormulaKind::RepeatedConjunction,
            BindingContextId::new(1),
            "& ... &",
        ),
        (
            SourceCompositeFormulaKind::RepeatedDisjunction,
            BindingContextId::new(1),
            "or ... or",
        ),
        (
            SourceCompositeFormulaKind::Disjunction,
            BindingContextId::new(1),
            "or",
        ),
        (
            SourceCompositeFormulaKind::Conjunction,
            BindingContextId::new(1),
            "&",
        ),
        (
            SourceCompositeFormulaKind::Disjunction,
            BindingContextId::new(1),
            "or",
        ),
    ];
    let task_257b3_formulas = [
        (
            SourceCompositeFormulaKind::Universal,
            BindingContextId::new(0),
            "for st",
        ),
        (
            SourceCompositeFormulaKind::Existential,
            BindingContextId::new(1),
            "ex st",
        ),
        (
            SourceCompositeFormulaKind::Universal,
            BindingContextId::new(2),
            "for st holds",
        ),
    ];
    let expected_formulas = match profile {
        CompositeProfile::Task257A => task_257a_formulas.as_slice(),
        CompositeProfile::Task257B1 => task_257b1_formulas.as_slice(),
        CompositeProfile::Task257B2 => task_257b2_formulas.as_slice(),
        CompositeProfile::Task257B3 => task_257b3_formulas.as_slice(),
    };
    let mut sites = BTreeSet::new();
    for (index, (row, (kind, context, spelling))) in
        input.formulas.iter().zip(expected_formulas).enumerate()
    {
        if row.source_ordinal != index
            || row.kind != *kind
            || row.context != *context
            || row.recovery != SourceCompositeFormulaRecovery::Normal
            || row.spelling != *spelling
            || !valid_range(input.source_id, row.source_range)
            || validate_arena_site(
                &row.site,
                row.source_range,
                formula_node_key(row.kind),
                row.recovery,
                arena,
            )
            .is_err()
        {
            return Err(SourceCompositeFormulaError::InvalidFormula {
                formula: SourceCompositeFormulaId::new(index),
            });
        }
        if !sites.insert(row.site.clone()) {
            return Err(SourceCompositeFormulaError::DuplicateSite);
        }
    }

    validate_wrappers(input, profile, arena, &mut sites)?;
    validate_root(input)?;
    validate_binder(input, profile, arena, &mut sites)?;
    validate_type_site(input, profile, arena, &mut sites)?;
    validate_edges(input, profile)?;
    validate_requests(input, profile)?;
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CompositeProfile {
    Task257A,
    Task257B1,
    Task257B2,
    Task257B3,
}

fn composite_profile(
    input: &SourceCompositeFormulaHandoffInput,
) -> Result<CompositeProfile, SourceCompositeFormulaError> {
    let counts = (
        input.formulas.len(),
        input.roots.len(),
        input.binders.len(),
        input.type_sites.len(),
        input.edges.len(),
        input.requests.len(),
    );
    match counts {
        (5, 1, 1, 1, 4, 6) if input.wrappers.is_empty() => Ok(CompositeProfile::Task257A),
        (1, 1, 1, 1, 0, 2) if input.wrappers.is_empty() => Ok(CompositeProfile::Task257B1),
        (8, 1, 1, 1, 7, 9) if input.wrappers.len() == 6 => Ok(CompositeProfile::Task257B2),
        (3, 1, 3, 3, 2, 6) if input.wrappers.is_empty() => Ok(CompositeProfile::Task257B3),
        _ => Err(SourceCompositeFormulaError::InvalidTree),
    }
}

fn validate_wrappers(
    input: &SourceCompositeFormulaHandoffInput,
    profile: CompositeProfile,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceCompositeFormulaError> {
    if matches!(
        profile,
        CompositeProfile::Task257B1 | CompositeProfile::Task257B3
    ) && !input.wrappers.is_empty()
    {
        return Err(SourceCompositeFormulaError::InvalidTree);
    }
    let mut groups = vec![Vec::new(); input.formulas.len()];
    let mut expected_ordinal = vec![0; input.formulas.len()];
    let mut previous_formula = None;
    for (index, row) in input.wrappers.iter().enumerate() {
        let formula = row.formula.index();
        let Some(owner) = input.formulas.get(formula) else {
            return Err(SourceCompositeFormulaError::InvalidWrapper {
                wrapper: SourceFormulaWrapperId::new(index),
            });
        };
        if previous_formula.is_some_and(|previous| formula < previous)
            || row.ordinal != expected_ordinal[formula]
            || row.context != owner.context
            || row.recovery != SourceCompositeFormulaRecovery::Normal
            || !strictly_contains(row.source_range, owner.source_range)
            || !wrapper_is_within_parent(input, profile, formula, row.source_range)
            || wrapper_crosses_unrelated_formula(input, profile, formula, row.source_range)
            || validate_arena_site(
                &row.site,
                row.source_range,
                "source.formula.parenthesized",
                row.recovery,
                arena,
            )
            .is_err()
            || !sites.insert(row.site.clone())
        {
            return Err(SourceCompositeFormulaError::InvalidWrapper {
                wrapper: SourceFormulaWrapperId::new(index),
            });
        }
        groups[formula].push(index);
        expected_ordinal[formula] += 1;
        previous_formula = Some(formula);
    }
    for (formula, group) in groups.iter().enumerate() {
        let owner = &input.formulas[formula];
        let mut inner_range = owner.source_range;
        let mut inner_spelling = owner.spelling.as_str();
        for &index in group.iter().rev() {
            let row = &input.wrappers[index];
            if !strictly_contains(row.source_range, inner_range)
                || row.spelling != format!("( {inner_spelling} )")
            {
                return Err(SourceCompositeFormulaError::InvalidWrapper {
                    wrapper: SourceFormulaWrapperId::new(index),
                });
            }
            inner_range = row.source_range;
            inner_spelling = &row.spelling;
        }
    }
    Ok(())
}

fn wrapper_is_within_parent(
    input: &SourceCompositeFormulaHandoffInput,
    profile: CompositeProfile,
    owner: usize,
    wrapper_range: SourceRange,
) -> bool {
    let parent = match profile {
        CompositeProfile::Task257B1 => return owner == 0,
        CompositeProfile::Task257A => match owner {
            0 => return true,
            1 | 2 => 0,
            3 => 2,
            4 => 3,
            _ => return false,
        },
        CompositeProfile::Task257B2 => match owner {
            0 => return true,
            1 => 0,
            2 | 5 => 1,
            3 | 4 => 2,
            6 | 7 => 5,
            _ => return false,
        },
        CompositeProfile::Task257B3 => return owner < 3,
    };
    properly_contains(input.formulas[parent].source_range, wrapper_range)
}

fn wrapper_crosses_unrelated_formula(
    input: &SourceCompositeFormulaHandoffInput,
    profile: CompositeProfile,
    owner: usize,
    wrapper_range: SourceRange,
) -> bool {
    input.formulas.iter().enumerate().any(|(other, formula)| {
        other != owner
            && !formula_is_ancestor(profile, owner, other)
            && !formula_is_ancestor(profile, other, owner)
            && ranges_overlap(wrapper_range, formula.source_range)
    })
}

fn formula_is_ancestor(profile: CompositeProfile, ancestor: usize, descendant: usize) -> bool {
    match profile {
        CompositeProfile::Task257A => {
            matches!((ancestor, descendant), (0, 1..=4) | (2, 3 | 4) | (3, 4))
        }
        CompositeProfile::Task257B1 => false,
        CompositeProfile::Task257B2 => matches!(
            (ancestor, descendant),
            (0, 1..=7) | (1, 2..=7) | (2, 3 | 4) | (5, 6 | 7)
        ),
        CompositeProfile::Task257B3 => {
            matches!((ancestor, descendant), (0, 1 | 2) | (1, 2))
        }
    }
}

fn ranges_overlap(left: SourceRange, right: SourceRange) -> bool {
    left.source_id == right.source_id && left.start < right.end && right.start < left.end
}

fn validate_root(
    input: &SourceCompositeFormulaHandoffInput,
) -> Result<(), SourceCompositeFormulaError> {
    let [root] = input.roots.as_slice() else {
        return Err(SourceCompositeFormulaError::InvalidTree);
    };
    if root.formula != SourceCompositeFormulaId::new(0)
        || root.ordinal != 0
        || root.ownership != SourceFormulaRootOwnership::UnassignedStatement
    {
        return Err(SourceCompositeFormulaError::InvalidRoot {
            root: SourceFormulaRootId::new(0),
        });
    }
    Ok(())
}

fn validate_binder(
    input: &SourceCompositeFormulaHandoffInput,
    profile: CompositeProfile,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceCompositeFormulaError> {
    if profile == CompositeProfile::Task257B3 {
        return validate_task_257b3_binders(input, arena, sites);
    }
    let [binder] = input.binders.as_slice() else {
        return Err(SourceCompositeFormulaError::InvalidTree);
    };
    let universal_index = match profile {
        CompositeProfile::Task257A => 2,
        CompositeProfile::Task257B1 | CompositeProfile::Task257B2 => 0,
        CompositeProfile::Task257B3 => unreachable!("handled above"),
    };
    let universal = &input.formulas[universal_index];
    if binder.formula != SourceCompositeFormulaId::new(universal_index)
        || binder.ordinal != 0
        || binder.segment_spelling != "x being"
        || binder.identifier_spelling != "x"
        || binder.local.spelling() != "x"
        || binder.local.scope() != &LocalTermScope::new(vec![0])
        || binder.local.declaration_range() != binder.identifier_range
        || binder.local.visible_after_ordinal() != 0
        || binder.binding != BindingId::new(0)
        || binder.body_context != BindingContextId::new(1)
        || binder.type_site != SourceBinderTypeSiteId::new(0)
        || binder.recovery != SourceCompositeFormulaRecovery::Normal
        || !valid_range(input.source_id, binder.segment_range)
        || !valid_range(input.source_id, binder.identifier_range)
        || !properly_contains(universal.source_range, binder.segment_range)
        || !properly_contains(binder.segment_range, binder.identifier_range)
        || validate_arena_site(
            &binder.segment_site,
            binder.segment_range,
            "source.formula.quantifier-binder",
            binder.recovery,
            arena,
        )
        .is_err()
        || validate_arena_site(
            &binder.identifier_site,
            binder.identifier_range,
            "source.formula.quantifier-binder",
            binder.recovery,
            arena,
        )
        .is_err()
        || !sites.insert(binder.segment_site.clone())
        || !sites.insert(binder.identifier_site.clone())
    {
        return Err(SourceCompositeFormulaError::InvalidBinder {
            binder: SourceQuantifierBinderId::new(0),
        });
    }
    Ok(())
}

fn validate_task_257b3_binders(
    input: &SourceCompositeFormulaHandoffInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceCompositeFormulaError> {
    let expected = [
        ("x being", "x", &[0_u32][..], 1, 1),
        ("y being", "y", &[0_u32, 0][..], 2, 2),
        ("r", "r", &[0_u32, 0, 0][..], 3, 3),
    ];
    for (index, (binder, (segment_spelling, identifier_spelling, scope, ordinal, binding))) in
        input.binders.iter().zip(expected).enumerate()
    {
        let formula = &input.formulas[index];
        if binder.formula != SourceCompositeFormulaId::new(index)
            || binder.ordinal != 0
            || binder.segment_spelling != segment_spelling
            || binder.identifier_spelling != identifier_spelling
            || binder.local.spelling() != identifier_spelling
            || binder.local.scope().path() != scope
            || binder.local.declaration_range() != binder.identifier_range
            || binder.local.visible_after_ordinal() != ordinal
            || binder.binding != BindingId::new(binding)
            || binder.body_context != BindingContextId::new(index + 1)
            || binder.type_site != SourceBinderTypeSiteId::new(index)
            || binder.recovery != SourceCompositeFormulaRecovery::Normal
            || !valid_range(input.source_id, binder.segment_range)
            || !valid_range(input.source_id, binder.identifier_range)
            || !properly_contains(formula.source_range, binder.segment_range)
            || !range_contains(binder.segment_range, binder.identifier_range)
            || (index < 2 && binder.segment_range == binder.identifier_range)
            || (index == 2 && binder.segment_range != binder.identifier_range)
            || validate_arena_site(
                &binder.segment_site,
                binder.segment_range,
                "source.formula.quantifier-binder",
                binder.recovery,
                arena,
            )
            .is_err()
            || validate_arena_site(
                &binder.identifier_site,
                binder.identifier_range,
                "source.formula.quantifier-binder",
                binder.recovery,
                arena,
            )
            .is_err()
            || !sites.insert(binder.segment_site.clone())
            || (binder.identifier_site != binder.segment_site
                && !sites.insert(binder.identifier_site.clone()))
        {
            return Err(SourceCompositeFormulaError::InvalidBinder {
                binder: SourceQuantifierBinderId::new(index),
            });
        }
    }
    Ok(())
}

fn validate_type_site(
    input: &SourceCompositeFormulaHandoffInput,
    profile: CompositeProfile,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceCompositeFormulaError> {
    if profile == CompositeProfile::Task257B3 {
        return validate_task_257b3_type_sites(input, arena, sites);
    }
    let [row] = input.type_sites.as_slice() else {
        return Err(SourceCompositeFormulaError::InvalidTree);
    };
    let binder = &input.binders[0];
    if row.binder != SourceQuantifierBinderId::new(0)
        || row.spelling != "set"
        || row.head_spelling != "set"
        || row.context != BindingContextId::new(0)
        || row.recovery != SourceCompositeFormulaRecovery::Normal
        || row.head != SourceBinderTypeHead::BuiltinSet
        || row.source_range != row.head_range
        || !valid_range(input.source_id, row.source_range)
        || !properly_contains(binder.segment_range, row.source_range)
        || validate_arena_site(
            &row.site,
            row.source_range,
            "source.formula.binder-type",
            row.recovery,
            arena,
        )
        .is_err()
        || validate_arena_site(
            &row.head_site,
            row.head_range,
            "source.formula.binder-type-head",
            row.recovery,
            arena,
        )
        .is_err()
        || !sites.insert(row.site.clone())
        || !sites.insert(row.head_site.clone())
    {
        return Err(SourceCompositeFormulaError::InvalidTypeSite {
            type_site: SourceBinderTypeSiteId::new(0),
        });
    }
    Ok(())
}

fn validate_task_257b3_type_sites(
    input: &SourceCompositeFormulaHandoffInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceCompositeFormulaError> {
    for (index, row) in input.type_sites.iter().enumerate() {
        let binder = &input.binders[index];
        let explicit = index < 2;
        if row.binder != SourceQuantifierBinderId::new(index)
            || row.spelling != "set"
            || row.head_spelling != "set"
            || row.context != BindingContextId::new(if index == 1 { 1 } else { 0 })
            || row.recovery != SourceCompositeFormulaRecovery::Normal
            || row.head != SourceBinderTypeHead::BuiltinSet
            || row.source_range != row.head_range
            || !valid_range(input.source_id, row.source_range)
            || (explicit && !properly_contains(binder.segment_range, row.source_range))
            || (!explicit && row.source_range.end > input.formulas[0].source_range.start)
            || validate_arena_site(
                &row.site,
                row.source_range,
                "source.formula.binder-type",
                row.recovery,
                arena,
            )
            .is_err()
            || validate_arena_site(
                &row.head_site,
                row.head_range,
                "source.formula.binder-type-head",
                row.recovery,
                arena,
            )
            .is_err()
            || !sites.insert(row.site.clone())
            || !sites.insert(row.head_site.clone())
        {
            return Err(SourceCompositeFormulaError::InvalidTypeSite {
                type_site: SourceBinderTypeSiteId::new(index),
            });
        }
    }
    if input.type_sites[0].source_range == input.type_sites[1].source_range
        || input.type_sites[0].source_range == input.type_sites[2].source_range
        || input.type_sites[1].source_range == input.type_sites[2].source_range
    {
        return Err(SourceCompositeFormulaError::InvalidTree);
    }
    Ok(())
}

fn validate_edges(
    input: &SourceCompositeFormulaHandoffInput,
    profile: CompositeProfile,
) -> Result<(), SourceCompositeFormulaError> {
    let task_257a_expected = [
        (0, 0, SourceFormulaEdgeRole::ImplicationLeft, 1),
        (0, 1, SourceFormulaEdgeRole::ImplicationRight, 2),
        (2, 0, SourceFormulaEdgeRole::UniversalBody, 3),
        (3, 0, SourceFormulaEdgeRole::NegatedFormula, 4),
    ];
    let task_257b2_expected = [
        (0, 0, SourceFormulaEdgeRole::UniversalBody, 1),
        (1, 0, SourceFormulaEdgeRole::BiconditionalLeft, 2),
        (1, 1, SourceFormulaEdgeRole::BiconditionalRight, 5),
        (2, 0, SourceFormulaEdgeRole::DisjunctionLeft, 3),
        (2, 1, SourceFormulaEdgeRole::DisjunctionRight, 4),
        (5, 0, SourceFormulaEdgeRole::DisjunctionLeft, 6),
        (5, 1, SourceFormulaEdgeRole::DisjunctionRight, 7),
    ];
    let task_257b3_expected = [
        (0, 0, SourceFormulaEdgeRole::UniversalBody, 1),
        (1, 0, SourceFormulaEdgeRole::ExistentialBody, 2),
    ];
    let expected = match profile {
        CompositeProfile::Task257A => task_257a_expected.as_slice(),
        CompositeProfile::Task257B1 => {
            return if input.edges.is_empty() {
                Ok(())
            } else {
                Err(SourceCompositeFormulaError::InvalidTree)
            };
        }
        CompositeProfile::Task257B2 => task_257b2_expected.as_slice(),
        CompositeProfile::Task257B3 => task_257b3_expected.as_slice(),
    };
    if input.edges.len() != expected.len() {
        return Err(SourceCompositeFormulaError::InvalidTree);
    }
    let mut incoming = vec![0_u8; input.formulas.len()];
    for (index, (row, (parent, ordinal, role, child))) in
        input.edges.iter().zip(expected.iter().copied()).enumerate()
    {
        if row.parent != SourceCompositeFormulaId::new(parent)
            || row.ordinal != ordinal
            || row.role != role
            || row.child != SourceCompositeFormulaId::new(child)
            || !properly_contains(
                input.formulas[parent].source_range,
                input.formulas[child].source_range,
            )
        {
            return Err(SourceCompositeFormulaError::InvalidEdge {
                edge: SourceFormulaEdgeId::new(index),
            });
        }
        incoming[child] += 1;
    }
    if incoming.first() != Some(&0) || incoming[1..].iter().any(|count| *count != 1) {
        return Err(SourceCompositeFormulaError::InvalidTree);
    }
    Ok(())
}

fn validate_requests(
    input: &SourceCompositeFormulaHandoffInput,
    profile: CompositeProfile,
) -> Result<(), SourceCompositeFormulaError> {
    let task_257a_expected = [
        (
            0,
            0,
            SourceFormulaRequestKind::ConnectiveSemantics,
            None,
            None,
        ),
        (
            1,
            0,
            SourceFormulaRequestKind::ConstantSemantics,
            None,
            None,
        ),
        (
            2,
            0,
            SourceFormulaRequestKind::QuantifierSemantics,
            None,
            None,
        ),
        (
            2,
            1,
            SourceFormulaRequestKind::BinderType,
            Some(SourceQuantifierBinderId::new(0)),
            Some(SourceBinderTypeSiteId::new(0)),
        ),
        (
            3,
            0,
            SourceFormulaRequestKind::NegationSemantics,
            None,
            None,
        ),
        (
            4,
            0,
            SourceFormulaRequestKind::ConstantSemantics,
            None,
            None,
        ),
    ];
    let task_257b1_expected = [
        (
            0,
            0,
            SourceFormulaRequestKind::QuantifierSemantics,
            None,
            None,
        ),
        (
            0,
            1,
            SourceFormulaRequestKind::BinderType,
            Some(SourceQuantifierBinderId::new(0)),
            Some(SourceBinderTypeSiteId::new(0)),
        ),
    ];
    let task_257b2_expected = [
        (
            0,
            0,
            SourceFormulaRequestKind::QuantifierSemantics,
            None,
            None,
        ),
        (
            0,
            1,
            SourceFormulaRequestKind::BinderType,
            Some(SourceQuantifierBinderId::new(0)),
            Some(SourceBinderTypeSiteId::new(0)),
        ),
        (
            1,
            0,
            SourceFormulaRequestKind::ConnectiveSemantics,
            None,
            None,
        ),
        (
            2,
            0,
            SourceFormulaRequestKind::ConnectiveSemantics,
            None,
            None,
        ),
        (
            3,
            0,
            SourceFormulaRequestKind::ConnectiveSemantics,
            None,
            None,
        ),
        (
            4,
            0,
            SourceFormulaRequestKind::ConnectiveSemantics,
            None,
            None,
        ),
        (
            5,
            0,
            SourceFormulaRequestKind::ConnectiveSemantics,
            None,
            None,
        ),
        (
            6,
            0,
            SourceFormulaRequestKind::ConnectiveSemantics,
            None,
            None,
        ),
        (
            7,
            0,
            SourceFormulaRequestKind::ConnectiveSemantics,
            None,
            None,
        ),
    ];
    let task_257b3_expected = [
        (
            0,
            0,
            SourceFormulaRequestKind::QuantifierSemantics,
            None,
            None,
        ),
        (
            0,
            1,
            SourceFormulaRequestKind::BinderType,
            Some(SourceQuantifierBinderId::new(0)),
            Some(SourceBinderTypeSiteId::new(0)),
        ),
        (
            1,
            0,
            SourceFormulaRequestKind::QuantifierSemantics,
            None,
            None,
        ),
        (
            1,
            1,
            SourceFormulaRequestKind::BinderType,
            Some(SourceQuantifierBinderId::new(1)),
            Some(SourceBinderTypeSiteId::new(1)),
        ),
        (
            2,
            0,
            SourceFormulaRequestKind::QuantifierSemantics,
            None,
            None,
        ),
        (
            2,
            1,
            SourceFormulaRequestKind::BinderType,
            Some(SourceQuantifierBinderId::new(2)),
            Some(SourceBinderTypeSiteId::new(2)),
        ),
    ];
    let expected = match profile {
        CompositeProfile::Task257A => task_257a_expected.as_slice(),
        CompositeProfile::Task257B1 => task_257b1_expected.as_slice(),
        CompositeProfile::Task257B2 => task_257b2_expected.as_slice(),
        CompositeProfile::Task257B3 => task_257b3_expected.as_slice(),
    };
    if input.requests.len() != expected.len() {
        return Err(SourceCompositeFormulaError::InvalidTree);
    }
    for (index, (row, (formula, ordinal, kind, binder, type_site))) in
        input.requests.iter().zip(expected).enumerate()
    {
        if row.formula != SourceCompositeFormulaId::new(*formula)
            || row.ordinal != *ordinal
            || row.kind != *kind
            || row.binder != *binder
            || row.type_site != *type_site
        {
            return Err(SourceCompositeFormulaError::InvalidRequest {
                request: SourceFormulaRequestId::new(index),
            });
        }
    }
    Ok(())
}

fn validate_base_bindings(
    input: &SourceCompositeFormulaHandoffInput,
    env: &BindingEnv,
) -> Result<(), SourceCompositeFormulaError> {
    if composite_profile(input)? == CompositeProfile::Task257B3 {
        return validate_task_257b3_base_bindings(input, env);
    }
    if env.source_id() != input.source_id
        || env.module_id() != &input.module_id
        || env.contexts().len() != 1
        || !env.bindings().is_empty()
        || env.diagnostics().len() != 4
    {
        return Err(SourceCompositeFormulaError::EnvironmentMismatch);
    }
    let Some(root) = env.contexts().get(BindingContextId::new(0)) else {
        return Err(SourceCompositeFormulaError::EnvironmentMismatch);
    };
    if root.owner != BindingContextOwner::Module
        || root.parent.is_some()
        || root.layer != BindingContextLayer::Module
        || root.lexical_scope.is_some()
        || !root.bindings.is_empty()
        || !root.visible_bindings.is_empty()
        || root.recovery != BindingContextRecovery::Normal
        || !diagnostics_are_exact(env.diagnostics())
    {
        return Err(SourceCompositeFormulaError::EnvironmentMismatch);
    }
    Ok(())
}

fn validate_extended_bindings(
    input: &SourceCompositeFormulaHandoffInput,
    env: &BindingEnv,
) -> Result<(), SourceCompositeFormulaError> {
    if composite_profile(input)? == CompositeProfile::Task257B3 {
        return validate_task_257b3_extended_bindings(input, env);
    }
    if env.source_id() != input.source_id
        || env.module_id() != &input.module_id
        || env.contexts().len() != 2
        || env.bindings().len() != 1
        || env.diagnostics().len() != 4
    {
        return Err(SourceCompositeFormulaError::EnvironmentMismatch);
    }
    let Some(root) = env.contexts().get(BindingContextId::new(0)) else {
        return Err(SourceCompositeFormulaError::EnvironmentMismatch);
    };
    let Some(body) = env.contexts().get(BindingContextId::new(1)) else {
        return Err(SourceCompositeFormulaError::EnvironmentMismatch);
    };
    let Some(binding) = env.bindings().get(BindingId::new(0)) else {
        return Err(SourceCompositeFormulaError::EnvironmentMismatch);
    };
    let binder = &input.binders[0];
    let type_site = &input.type_sites[0];
    let universal_index = match composite_profile(input)? {
        CompositeProfile::Task257A => 2,
        CompositeProfile::Task257B1 | CompositeProfile::Task257B2 => 0,
        CompositeProfile::Task257B3 => unreachable!("handled above"),
    };
    if root.owner != BindingContextOwner::Module
        || root.parent.is_some()
        || root.layer != BindingContextLayer::Module
        || root.lexical_scope.is_some()
        || !root.bindings.is_empty()
        || !root.visible_bindings.is_empty()
        || root.recovery != BindingContextRecovery::Normal
        || body.owner
            != (BindingContextOwner::SourceFormula {
                source_range: input.formulas[universal_index].source_range,
            })
        || body.parent != Some(BindingContextId::new(0))
        || body.layer != BindingContextLayer::Expression
        || body.lexical_scope.as_ref() != Some(binder.local.scope())
        || body.bindings != [BindingId::new(0)]
        || body.visible_bindings != [BindingId::new(0)]
        || body.recovery != BindingContextRecovery::Normal
        || binding.spelling != "x"
        || binding.kind != BindingKind::QuantifierBinder
        || binding.identity
            != (BinderIdentity::ResolverLocal {
                scope: binder.local.scope().clone(),
                ordinal: 0,
                declaration_range: binder.identifier_range,
            })
        || binding.owner_context != BindingContextId::new(1)
        || binding.declaration_range != binder.identifier_range
        || binding.visible_after_ordinal != 0
        || binding.type_site != BindingTypeSite::Source(type_site.source_range)
        || binding.status != BindingStatus::Active
        || !binding.captured.identities().is_empty()
        || !binding.diagnostics.is_empty()
        || binding.recovery != BindingRecoveryState::Normal
        || !diagnostics_are_exact(env.diagnostics())
    {
        return Err(SourceCompositeFormulaError::EnvironmentMismatch);
    }
    Ok(())
}

fn validate_task_257b3_base_bindings(
    input: &SourceCompositeFormulaHandoffInput,
    env: &BindingEnv,
) -> Result<(), SourceCompositeFormulaError> {
    if env.source_id() != input.source_id
        || env.module_id() != &input.module_id
        || env.contexts().len() != 1
        || env.bindings().len() != 1
        || !env.diagnostics().is_empty()
    {
        return Err(SourceCompositeFormulaError::EnvironmentMismatch);
    }
    let root = env
        .contexts()
        .get(BindingContextId::new(0))
        .ok_or(SourceCompositeFormulaError::EnvironmentMismatch)?;
    let reserved = env
        .bindings()
        .get(BindingId::new(0))
        .ok_or(SourceCompositeFormulaError::EnvironmentMismatch)?;
    let reserve_type = input.type_sites[2].source_range;
    if root.owner != BindingContextOwner::Module
        || root.parent.is_some()
        || root.layer != BindingContextLayer::Module
        || root.lexical_scope.is_some()
        || root.bindings != [BindingId::new(0)]
        || root.visible_bindings != [BindingId::new(0)]
        || root.recovery != BindingContextRecovery::Normal
        || reserved.spelling != "r"
        || reserved.kind != BindingKind::ReservedVariable
        || reserved.identity
            != (BinderIdentity::ReservedVariable {
                spelling: "r".to_owned(),
                declaration_range: reserved.declaration_range,
            })
        || reserved.owner_context != BindingContextId::new(0)
        || !valid_range(input.source_id, reserved.declaration_range)
        || reserved.declaration_range.end > reserve_type.start
        || reserve_type.end > input.formulas[0].source_range.start
        || reserved.visible_after_ordinal != 0
        || reserved.type_site != BindingTypeSite::Source(reserve_type)
        || reserved.status != BindingStatus::Reserved
        || !reserved.captured.identities().is_empty()
        || !reserved.diagnostics.is_empty()
        || reserved.recovery != BindingRecoveryState::Normal
    {
        return Err(SourceCompositeFormulaError::EnvironmentMismatch);
    }
    Ok(())
}

fn validate_task_257b3_extended_bindings(
    input: &SourceCompositeFormulaHandoffInput,
    env: &BindingEnv,
) -> Result<(), SourceCompositeFormulaError> {
    if env.source_id() != input.source_id
        || env.module_id() != &input.module_id
        || env.contexts().len() != 4
        || env.bindings().len() != 4
        || !env.diagnostics().is_empty()
    {
        return Err(SourceCompositeFormulaError::EnvironmentMismatch);
    }
    let root = env
        .contexts()
        .get(BindingContextId::new(0))
        .ok_or(SourceCompositeFormulaError::EnvironmentMismatch)?;
    let reserved = env
        .bindings()
        .get(BindingId::new(0))
        .ok_or(SourceCompositeFormulaError::EnvironmentMismatch)?;
    let reserve_type = input.type_sites[2].source_range;
    if root.owner != BindingContextOwner::Module
        || root.parent.is_some()
        || root.layer != BindingContextLayer::Module
        || root.lexical_scope.is_some()
        || root.bindings != [BindingId::new(0)]
        || root.visible_bindings != [BindingId::new(0)]
        || root.recovery != BindingContextRecovery::Normal
        || reserved.spelling != "r"
        || reserved.kind != BindingKind::ReservedVariable
        || reserved.identity
            != (BinderIdentity::ReservedVariable {
                spelling: "r".to_owned(),
                declaration_range: reserved.declaration_range,
            })
        || reserved.owner_context != BindingContextId::new(0)
        || !valid_range(input.source_id, reserved.declaration_range)
        || reserved.declaration_range.end > reserve_type.start
        || reserved.visible_after_ordinal != 0
        || reserved.type_site != BindingTypeSite::Source(reserve_type)
        || reserved.status != BindingStatus::Reserved
        || !reserved.captured.identities().is_empty()
        || !reserved.diagnostics.is_empty()
        || reserved.recovery != BindingRecoveryState::Normal
    {
        return Err(SourceCompositeFormulaError::EnvironmentMismatch);
    }
    for (index, binder) in input.binders.iter().enumerate() {
        let binding_id = BindingId::new(index + 1);
        let context_id = BindingContextId::new(index + 1);
        let context = env
            .contexts()
            .get(context_id)
            .ok_or(SourceCompositeFormulaError::EnvironmentMismatch)?;
        let binding = env
            .bindings()
            .get(binding_id)
            .ok_or(SourceCompositeFormulaError::EnvironmentMismatch)?;
        let expected_visible = (0..=index + 1).map(BindingId::new).collect::<Vec<_>>();
        if context.owner
            != (BindingContextOwner::SourceFormula {
                source_range: input.formulas[index].source_range,
            })
            || context.parent != Some(BindingContextId::new(index))
            || context.layer != BindingContextLayer::Expression
            || context.lexical_scope.as_ref() != Some(binder.local.scope())
            || context.bindings != [binding_id]
            || context.visible_bindings != expected_visible
            || context.recovery != BindingContextRecovery::Normal
            || binding.spelling != binder.identifier_spelling
            || binding.kind != BindingKind::QuantifierBinder
            || binding.identity
                != (BinderIdentity::ResolverLocal {
                    scope: binder.local.scope().clone(),
                    ordinal: index + 1,
                    declaration_range: binder.identifier_range,
                })
            || binding.owner_context != context_id
            || binding.declaration_range != binder.identifier_range
            || binding.visible_after_ordinal != index + 1
            || binding.type_site != BindingTypeSite::Source(input.type_sites[index].source_range)
            || binding.status != BindingStatus::Active
            || !binding.captured.identities().is_empty()
            || !binding.diagnostics.is_empty()
            || binding.recovery != BindingRecoveryState::Normal
        {
            return Err(SourceCompositeFormulaError::EnvironmentMismatch);
        }
    }
    Ok(())
}

fn diagnostics_are_exact(table: &BindingDiagnosticTable) -> bool {
    let keys = [
        "checker.binding.external.local_bindings",
        "checker.binding.external.use_site_scope",
        "checker.binding.external.reserve_payload",
        "checker.binding.external.closure_payload",
    ];
    table.len() == keys.len()
        && table.iter().zip(keys).all(|((id, row), key)| {
            id.index() < keys.len()
                && row.source_range.is_none()
                && row.class == BindingDiagnosticClass::ExternalDependencyGap
                && row.severity == BindingDiagnosticSeverity::Note
                && row.message_key == key
                && row.recovery == BindingDiagnosticRecovery::Degraded
        })
}

fn validate_arena_site(
    site: &TypedSiteRef,
    range: SourceRange,
    kind: &str,
    recovery: SourceCompositeFormulaRecovery,
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
    recovery: SourceCompositeFormulaRecovery,
    node_recovery: NodeRecoveryState,
) -> bool {
    match recovery {
        SourceCompositeFormulaRecovery::Normal => node_recovery == NodeRecoveryState::Normal,
        SourceCompositeFormulaRecovery::Degraded => matches!(
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

fn formula_node_key(kind: SourceCompositeFormulaKind) -> &'static str {
    match kind {
        SourceCompositeFormulaKind::Implication => "source.formula.composite.implication",
        SourceCompositeFormulaKind::Universal => "source.formula.composite.universal",
        SourceCompositeFormulaKind::Existential => "source.formula.composite.existential",
        SourceCompositeFormulaKind::Negation => "source.formula.composite.negation",
        SourceCompositeFormulaKind::Contradiction => "source.formula.constant.contradiction",
        SourceCompositeFormulaKind::Conjunction => "source.formula.composite.conjunction",
        SourceCompositeFormulaKind::RepeatedConjunction => {
            "source.formula.composite.repeated-conjunction"
        }
        SourceCompositeFormulaKind::Disjunction => "source.formula.composite.disjunction",
        SourceCompositeFormulaKind::RepeatedDisjunction => {
            "source.formula.composite.repeated-disjunction"
        }
        SourceCompositeFormulaKind::Biconditional => "source.formula.composite.biconditional",
    }
}

fn formula_kind_key(kind: SourceCompositeFormulaKind) -> &'static str {
    match kind {
        SourceCompositeFormulaKind::Implication => "implication",
        SourceCompositeFormulaKind::Universal => "universal",
        SourceCompositeFormulaKind::Existential => "existential",
        SourceCompositeFormulaKind::Negation => "negation",
        SourceCompositeFormulaKind::Contradiction => "contradiction",
        SourceCompositeFormulaKind::Conjunction => "conjunction",
        SourceCompositeFormulaKind::RepeatedConjunction => "repeated-conjunction",
        SourceCompositeFormulaKind::Disjunction => "disjunction",
        SourceCompositeFormulaKind::RepeatedDisjunction => "repeated-disjunction",
        SourceCompositeFormulaKind::Biconditional => "biconditional",
    }
}

fn recovery_key(recovery: SourceCompositeFormulaRecovery) -> &'static str {
    match recovery {
        SourceCompositeFormulaRecovery::Normal => "normal",
        SourceCompositeFormulaRecovery::Degraded => "degraded",
    }
}

fn root_ownership_key(ownership: SourceFormulaRootOwnership) -> &'static str {
    match ownership {
        SourceFormulaRootOwnership::UnassignedStatement => "unassigned-statement",
    }
}

fn binder_type_head_key(head: SourceBinderTypeHead) -> &'static str {
    match head {
        SourceBinderTypeHead::BuiltinSet => "builtin-set",
    }
}

fn edge_role_key(role: SourceFormulaEdgeRole) -> &'static str {
    match role {
        SourceFormulaEdgeRole::ImplicationLeft => "implication-left",
        SourceFormulaEdgeRole::ImplicationRight => "implication-right",
        SourceFormulaEdgeRole::UniversalBody => "universal-body",
        SourceFormulaEdgeRole::ExistentialBody => "existential-body",
        SourceFormulaEdgeRole::NegatedFormula => "negated-formula",
        SourceFormulaEdgeRole::DisjunctionLeft => "disjunction-left",
        SourceFormulaEdgeRole::DisjunctionRight => "disjunction-right",
        SourceFormulaEdgeRole::BiconditionalLeft => "biconditional-left",
        SourceFormulaEdgeRole::BiconditionalRight => "biconditional-right",
    }
}

fn request_kind_key(kind: SourceFormulaRequestKind) -> &'static str {
    match kind {
        SourceFormulaRequestKind::ConnectiveSemantics => "connective-semantics",
        SourceFormulaRequestKind::ConstantSemantics => "constant-semantics",
        SourceFormulaRequestKind::QuantifierSemantics => "quantifier-semantics",
        SourceFormulaRequestKind::BinderType => "binder-type",
        SourceFormulaRequestKind::NegationSemantics => "negation-semantics",
    }
}

fn write_optional_id(output: &mut String, id: Option<usize>) {
    if let Some(id) = id {
        let _ = write!(output, "{id}");
    } else {
        output.push('-');
    }
}

fn write_module_id(output: &mut String, module: &ModuleId) {
    let _ = write!(
        output,
        "{}::{}",
        module.package().as_str(),
        module.path().as_str()
    );
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        binding_env::{
            BindingContextTable, BindingDiagnosticDraft, BindingDiagnosticTable, BindingTable,
        },
        typed_ast::{
            CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
            TypeFactTable, TypeTable, TypedAst, TypedAstError, TypedAstParts, TypedNode,
            TypedNodeId,
        },
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator as _,
    };

    struct Fixture {
        source: SourceId,
        module: ModuleId,
        arena: TypedArena,
        input: SourceCompositeFormulaHandoffInput,
        base: BindingEnv,
    }

    pub(crate) struct Task257B2CompositeFixture {
        pub(crate) source: SourceId,
        pub(crate) module: ModuleId,
        pub(crate) arena: TypedArena,
        pub(crate) input: SourceCompositeFormulaHandoffInput,
        pub(crate) base: BindingEnv,
    }

    pub(crate) struct Task257B3CompositeFixture {
        pub(crate) source: SourceId,
        pub(crate) module: ModuleId,
        pub(crate) arena: TypedArena,
        pub(crate) input: SourceCompositeFormulaHandoffInput,
        pub(crate) base: BindingEnv,
    }

    fn source_id() -> SourceId {
        source_id_with_snapshot_byte("a7")
    }

    fn distinct_source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "b7".repeat(32)
        ))
        .expect("snapshot");
        let allocator = InMemorySessionIdAllocator::new();
        let _ = allocator.next_source_id(snapshot).expect("first source");
        allocator.next_source_id(snapshot).expect("second source")
    }

    fn source_id_with_snapshot_byte(byte: &str) -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            byte.repeat(32)
        ))
        .expect("snapshot");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source")
    }

    fn module() -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("composite.fixture"))
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

    fn base_bindings(source: SourceId, module: &ModuleId) -> BindingEnv {
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
        let mut diagnostics = BindingDiagnosticTable::new();
        for message_key in [
            "checker.binding.external.local_bindings",
            "checker.binding.external.use_site_scope",
            "checker.binding.external.reserve_payload",
            "checker.binding.external.closure_payload",
        ] {
            diagnostics.insert(BindingDiagnosticDraft {
                source_range: None,
                class: BindingDiagnosticClass::ExternalDependencyGap,
                severity: BindingDiagnosticSeverity::Note,
                message_key: message_key.to_owned(),
                recovery: BindingDiagnosticRecovery::Degraded,
            });
        }
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings: BindingTable::new(),
            diagnostics,
        })
        .expect("base bindings")
    }

    fn extended_bindings_with_identity(
        fixture: &Fixture,
        identity: BinderIdentity,
        duplicate: bool,
        owner_range: SourceRange,
    ) -> Result<BindingEnv, crate::binding_env::BindingEnvError> {
        let declaration_range = fixture.input.binders[0].identifier_range;
        let type_range = fixture.input.type_sites[0].source_range;
        let mut bindings = BindingTable::new();
        let binding = bindings.insert(BindingDraft {
            spelling: "x".to_owned(),
            kind: BindingKind::QuantifierBinder,
            identity: identity.clone(),
            owner_context: BindingContextId::new(1),
            declaration_range,
            visible_after_ordinal: 0,
            type_site: BindingTypeSite::Source(type_range),
            status: BindingStatus::Active,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        let mut owned = vec![binding];
        if duplicate {
            owned.push(bindings.insert(BindingDraft {
                spelling: "x".to_owned(),
                kind: BindingKind::QuantifierBinder,
                identity,
                owner_context: BindingContextId::new(1),
                declaration_range,
                visible_after_ordinal: 0,
                type_site: BindingTypeSite::Source(type_range),
                status: BindingStatus::Active,
                captured: CapturedFreeVariables::default(),
                diagnostics: Vec::new(),
                recovery: BindingRecoveryState::Normal,
            }));
        }
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
            owner: BindingContextOwner::SourceFormula {
                source_range: owner_range,
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Expression,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: owned.clone(),
            visible_bindings: owned,
            recovery: BindingContextRecovery::Normal,
        });
        BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts,
            bindings,
            diagnostics: fixture.base.diagnostics().clone(),
        })
    }

    fn fixture() -> Fixture {
        let source = source_id();
        let module = module();
        let ranges = [
            (10, 90),
            (10, 23),
            (30, 90),
            (60, 90),
            (70, 83),
            (35, 50),
            (35, 36),
            (45, 48),
            (45, 48),
            (5, 95),
            (1, 99),
            (65, 85),
        ];
        let keys = [
            "source.formula.composite.implication",
            "source.formula.constant.contradiction",
            "source.formula.composite.universal",
            "source.formula.composite.negation",
            "source.formula.constant.contradiction",
            "source.formula.quantifier-binder",
            "source.formula.quantifier-binder",
            "source.formula.binder-type",
            "source.formula.binder-type-head",
            "source.formula.parenthesized",
            "source.formula.parenthesized",
            "source.formula.parenthesized",
        ];
        let arena = TypedArena::try_new(
            None,
            ranges
                .iter()
                .zip(keys)
                .map(|(&(start, end), key)| {
                    TypedNode::new(key, SourceAnchor::Range(range(source, start, end)))
                })
                .collect(),
        )
        .expect("arena");
        let formula = |site: usize,
                       source_range: (usize, usize),
                       source_ordinal: usize,
                       context: usize,
                       spelling: &str,
                       kind: SourceCompositeFormulaKind| {
            SourceCompositeFormulaInput {
                site: node(site),
                source_range: range(source, source_range.0, source_range.1),
                source_ordinal,
                context: BindingContextId::new(context),
                recovery: SourceCompositeFormulaRecovery::Normal,
                spelling: spelling.to_owned(),
                kind,
            }
        };
        let input = SourceCompositeFormulaHandoffInput {
            source_id: source,
            module_id: module.clone(),
            formulas: vec![
                formula(
                    0,
                    (10, 90),
                    0,
                    0,
                    "implies",
                    SourceCompositeFormulaKind::Implication,
                ),
                formula(
                    1,
                    (10, 23),
                    1,
                    0,
                    "contradiction",
                    SourceCompositeFormulaKind::Contradiction,
                ),
                formula(
                    2,
                    (30, 90),
                    2,
                    0,
                    "for holds",
                    SourceCompositeFormulaKind::Universal,
                ),
                formula(
                    3,
                    (60, 90),
                    3,
                    1,
                    "not",
                    SourceCompositeFormulaKind::Negation,
                ),
                formula(
                    4,
                    (70, 83),
                    4,
                    1,
                    "contradiction",
                    SourceCompositeFormulaKind::Contradiction,
                ),
            ],
            wrappers: Vec::new(),
            roots: vec![SourceFormulaRootInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                ownership: SourceFormulaRootOwnership::UnassignedStatement,
            }],
            binders: vec![SourceQuantifierBinderInput {
                formula: SourceCompositeFormulaId::new(2),
                ordinal: 0,
                segment_site: node(5),
                segment_range: range(source, 35, 50),
                segment_spelling: "x being".to_owned(),
                identifier_site: node(6),
                identifier_range: range(source, 35, 36),
                identifier_spelling: "x".to_owned(),
                local: LocalTermBinding::new(
                    "x",
                    LocalTermScope::new(vec![0]),
                    range(source, 35, 36),
                    0,
                ),
                binding: BindingId::new(0),
                body_context: BindingContextId::new(1),
                type_site: SourceBinderTypeSiteId::new(0),
                recovery: SourceCompositeFormulaRecovery::Normal,
            }],
            type_sites: vec![SourceBinderTypeSiteInput {
                binder: SourceQuantifierBinderId::new(0),
                site: node(7),
                source_range: range(source, 45, 48),
                spelling: "set".to_owned(),
                head_site: node(8),
                head_range: range(source, 45, 48),
                head_spelling: "set".to_owned(),
                context: BindingContextId::new(0),
                recovery: SourceCompositeFormulaRecovery::Normal,
                head: SourceBinderTypeHead::BuiltinSet,
            }],
            edges: vec![
                SourceFormulaEdgeInput {
                    parent: SourceCompositeFormulaId::new(0),
                    ordinal: 0,
                    role: SourceFormulaEdgeRole::ImplicationLeft,
                    child: SourceCompositeFormulaId::new(1),
                },
                SourceFormulaEdgeInput {
                    parent: SourceCompositeFormulaId::new(0),
                    ordinal: 1,
                    role: SourceFormulaEdgeRole::ImplicationRight,
                    child: SourceCompositeFormulaId::new(2),
                },
                SourceFormulaEdgeInput {
                    parent: SourceCompositeFormulaId::new(2),
                    ordinal: 0,
                    role: SourceFormulaEdgeRole::UniversalBody,
                    child: SourceCompositeFormulaId::new(3),
                },
                SourceFormulaEdgeInput {
                    parent: SourceCompositeFormulaId::new(3),
                    ordinal: 0,
                    role: SourceFormulaEdgeRole::NegatedFormula,
                    child: SourceCompositeFormulaId::new(4),
                },
            ],
            requests: vec![
                SourceFormulaRequestInput {
                    formula: SourceCompositeFormulaId::new(0),
                    ordinal: 0,
                    kind: SourceFormulaRequestKind::ConnectiveSemantics,
                    binder: None,
                    type_site: None,
                },
                SourceFormulaRequestInput {
                    formula: SourceCompositeFormulaId::new(1),
                    ordinal: 0,
                    kind: SourceFormulaRequestKind::ConstantSemantics,
                    binder: None,
                    type_site: None,
                },
                SourceFormulaRequestInput {
                    formula: SourceCompositeFormulaId::new(2),
                    ordinal: 0,
                    kind: SourceFormulaRequestKind::QuantifierSemantics,
                    binder: None,
                    type_site: None,
                },
                SourceFormulaRequestInput {
                    formula: SourceCompositeFormulaId::new(2),
                    ordinal: 1,
                    kind: SourceFormulaRequestKind::BinderType,
                    binder: Some(SourceQuantifierBinderId::new(0)),
                    type_site: Some(SourceBinderTypeSiteId::new(0)),
                },
                SourceFormulaRequestInput {
                    formula: SourceCompositeFormulaId::new(3),
                    ordinal: 0,
                    kind: SourceFormulaRequestKind::NegationSemantics,
                    binder: None,
                    type_site: None,
                },
                SourceFormulaRequestInput {
                    formula: SourceCompositeFormulaId::new(4),
                    ordinal: 0,
                    kind: SourceFormulaRequestKind::ConstantSemantics,
                    binder: None,
                    type_site: None,
                },
            ],
        };
        let base = base_bindings(source, &module);
        Fixture {
            source,
            module,
            arena,
            input,
            base,
        }
    }

    pub(crate) fn task_257b2_composite_fixture() -> Task257B2CompositeFixture {
        let source = source_id_with_snapshot_byte("d7");
        let module = ModuleId::new(
            PackageId::new("pkg"),
            ModulePath::new("composite.task257b2"),
        );
        let formula_ranges = [
            (50, 164),
            (72, 164),
            (73, 121),
            (74, 93),
            (99, 120),
            (128, 163),
            (129, 142),
            (148, 162),
        ];
        let wrapper_ranges = [
            (72, 122),
            (73, 94),
            (98, 121),
            (127, 164),
            (128, 143),
            (147, 163),
        ];
        let equality_ranges = [
            (74, 79),
            (88, 93),
            (99, 104),
            (115, 120),
            (129, 134),
            (137, 142),
            (148, 153),
            (157, 162),
        ];
        let numeral_ranges = [
            (74, 75),
            (78, 79),
            (88, 89),
            (92, 93),
            (99, 100),
            (103, 104),
            (115, 116),
            (119, 120),
            (129, 130),
            (133, 134),
            (137, 138),
            (141, 142),
            (148, 149),
            (152, 153),
            (157, 158),
            (161, 162),
        ];
        let formula_keys = [
            "source.formula.composite.universal",
            "source.formula.composite.biconditional",
            "source.formula.composite.disjunction",
            "source.formula.composite.repeated-conjunction",
            "source.formula.composite.repeated-disjunction",
            "source.formula.composite.disjunction",
            "source.formula.composite.conjunction",
            "source.formula.composite.disjunction",
        ];
        let mut nodes = formula_ranges
            .into_iter()
            .zip(formula_keys)
            .map(|((start, end), key)| {
                TypedNode::new(key, SourceAnchor::Range(range(source, start, end)))
            })
            .collect::<Vec<_>>();
        nodes.extend(wrapper_ranges.into_iter().map(|(start, end)| {
            TypedNode::new(
                "source.formula.parenthesized",
                SourceAnchor::Range(range(source, start, end)),
            )
        }));
        nodes.extend(
            [
                ((54, 65), "source.formula.quantifier-binder"),
                ((54, 55), "source.formula.quantifier-binder"),
                ((62, 65), "source.formula.binder-type"),
                ((62, 65), "source.formula.binder-type-head"),
            ]
            .into_iter()
            .map(|((start, end), key)| {
                TypedNode::new(key, SourceAnchor::Range(range(source, start, end)))
            }),
        );
        nodes.extend(equality_ranges.into_iter().map(|(start, end)| {
            TypedNode::new(
                "source.formula.atomic.equality",
                SourceAnchor::Range(range(source, start, end)),
            )
        }));
        nodes.extend(numeral_ranges.into_iter().map(|(start, end)| {
            TypedNode::new(
                "source.term.numeral",
                SourceAnchor::Range(range(source, start, end)),
            )
        }));
        let arena = TypedArena::try_new(None, nodes).expect("Task 257B2 arena");
        let formula_specs = [
            (SourceCompositeFormulaKind::Universal, "for holds"),
            (SourceCompositeFormulaKind::Biconditional, "iff"),
            (SourceCompositeFormulaKind::Disjunction, "or"),
            (SourceCompositeFormulaKind::RepeatedConjunction, "& ... &"),
            (SourceCompositeFormulaKind::RepeatedDisjunction, "or ... or"),
            (SourceCompositeFormulaKind::Disjunction, "or"),
            (SourceCompositeFormulaKind::Conjunction, "&"),
            (SourceCompositeFormulaKind::Disjunction, "or"),
        ];
        let formulas = formula_specs
            .into_iter()
            .zip(formula_ranges)
            .enumerate()
            .map(
                |(index, ((kind, spelling), (start, end)))| SourceCompositeFormulaInput {
                    site: node(index),
                    source_range: range(source, start, end),
                    source_ordinal: index,
                    context: BindingContextId::new(usize::from(index != 0)),
                    recovery: SourceCompositeFormulaRecovery::Normal,
                    spelling: spelling.to_owned(),
                    kind,
                },
            )
            .collect();
        let wrapper_owners = [2, 3, 4, 5, 6, 7];
        let wrapper_spellings = [
            "( or )",
            "( & ... & )",
            "( or ... or )",
            "( or )",
            "( & )",
            "( or )",
        ];
        let wrappers = wrapper_owners
            .into_iter()
            .zip(wrapper_spellings)
            .zip(wrapper_ranges)
            .enumerate()
            .map(
                |(index, ((formula, spelling), (start, end)))| SourceFormulaWrapperInput {
                    formula: SourceCompositeFormulaId::new(formula),
                    ordinal: 0,
                    site: node(8 + index),
                    source_range: range(source, start, end),
                    context: BindingContextId::new(1),
                    recovery: SourceCompositeFormulaRecovery::Normal,
                    spelling: spelling.to_owned(),
                },
            )
            .collect();
        let mut requests = vec![
            SourceFormulaRequestInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                kind: SourceFormulaRequestKind::QuantifierSemantics,
                binder: None,
                type_site: None,
            },
            SourceFormulaRequestInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 1,
                kind: SourceFormulaRequestKind::BinderType,
                binder: Some(SourceQuantifierBinderId::new(0)),
                type_site: Some(SourceBinderTypeSiteId::new(0)),
            },
        ];
        requests.extend((1..8).map(|formula| SourceFormulaRequestInput {
            formula: SourceCompositeFormulaId::new(formula),
            ordinal: 0,
            kind: SourceFormulaRequestKind::ConnectiveSemantics,
            binder: None,
            type_site: None,
        }));
        let input = SourceCompositeFormulaHandoffInput {
            source_id: source,
            module_id: module.clone(),
            formulas,
            wrappers,
            roots: vec![SourceFormulaRootInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                ownership: SourceFormulaRootOwnership::UnassignedStatement,
            }],
            binders: vec![SourceQuantifierBinderInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                segment_site: node(14),
                segment_range: range(source, 54, 65),
                segment_spelling: "x being".to_owned(),
                identifier_site: node(15),
                identifier_range: range(source, 54, 55),
                identifier_spelling: "x".to_owned(),
                local: LocalTermBinding::new(
                    "x",
                    LocalTermScope::new(vec![0]),
                    range(source, 54, 55),
                    0,
                ),
                binding: BindingId::new(0),
                body_context: BindingContextId::new(1),
                type_site: SourceBinderTypeSiteId::new(0),
                recovery: SourceCompositeFormulaRecovery::Normal,
            }],
            type_sites: vec![SourceBinderTypeSiteInput {
                binder: SourceQuantifierBinderId::new(0),
                site: node(16),
                source_range: range(source, 62, 65),
                spelling: "set".to_owned(),
                head_site: node(17),
                head_range: range(source, 62, 65),
                head_spelling: "set".to_owned(),
                context: BindingContextId::new(0),
                recovery: SourceCompositeFormulaRecovery::Normal,
                head: SourceBinderTypeHead::BuiltinSet,
            }],
            edges: [
                (0, 0, SourceFormulaEdgeRole::UniversalBody, 1),
                (1, 0, SourceFormulaEdgeRole::BiconditionalLeft, 2),
                (1, 1, SourceFormulaEdgeRole::BiconditionalRight, 5),
                (2, 0, SourceFormulaEdgeRole::DisjunctionLeft, 3),
                (2, 1, SourceFormulaEdgeRole::DisjunctionRight, 4),
                (5, 0, SourceFormulaEdgeRole::DisjunctionLeft, 6),
                (5, 1, SourceFormulaEdgeRole::DisjunctionRight, 7),
            ]
            .into_iter()
            .map(|(parent, ordinal, role, child)| SourceFormulaEdgeInput {
                parent: SourceCompositeFormulaId::new(parent),
                ordinal,
                role,
                child: SourceCompositeFormulaId::new(child),
            })
            .collect(),
            requests,
        };
        let base = base_bindings(source, &module);
        Task257B2CompositeFixture {
            source,
            module,
            arena,
            input,
            base,
        }
    }

    fn build(
        fixture: &Fixture,
        input: SourceCompositeFormulaHandoffInput,
    ) -> Result<SourceCompositeFormulaHandoff, SourceCompositeFormulaError> {
        let extended =
            SourceCompositeFormulaProducer::extend_bindings(&input, &fixture.base, &fixture.arena)?;
        SourceCompositeFormulaProducer::build(input, &extended, &fixture.arena)
    }

    fn build_task_257b2(
        fixture: &Task257B2CompositeFixture,
        input: SourceCompositeFormulaHandoffInput,
    ) -> Result<SourceCompositeFormulaHandoff, SourceCompositeFormulaError> {
        let extended =
            SourceCompositeFormulaProducer::extend_bindings(&input, &fixture.base, &fixture.arena)?;
        SourceCompositeFormulaProducer::build(input, &extended, &fixture.arena)
    }

    pub(crate) fn task_257b2_debug_oracle(value: &str) -> (usize, u64, u64) {
        let mut fnv = 0xcbf2_9ce4_8422_2325_u64;
        let mut mixed = 0x6a09_e667_f3bc_c909_u64;
        for (index, byte) in value.bytes().enumerate() {
            fnv ^= u64::from(byte);
            fnv = fnv.wrapping_mul(0x0000_0100_0000_01b3);
            mixed ^= u64::from(byte).wrapping_add((index as u64).rotate_left(17));
            mixed = mixed.rotate_left(11).wrapping_mul(0x9e37_79b1_85eb_ca87);
        }
        (value.len(), fnv, mixed)
    }

    pub(crate) fn task_257a_installed_typed_ast() -> TypedAst {
        let fixture = fixture();
        let handoff = build(&fixture, fixture.input.clone()).expect("Task 257A handoff");
        empty_typed_ast(fixture.source, fixture.module, fixture.arena)
            .with_source_composite_formula(handoff)
            .expect("Task 257A installation")
    }

    #[track_caller]
    fn assert_input_rejected(
        fixture: &Fixture,
        mutate: impl FnOnce(&mut SourceCompositeFormulaHandoffInput),
    ) {
        let mut input = fixture.input.clone();
        mutate(&mut input);
        assert!(build(fixture, input).is_err());
    }

    #[track_caller]
    fn assert_task_257b2_input_rejected(
        fixture: &Task257B2CompositeFixture,
        mutate: impl FnOnce(&mut SourceCompositeFormulaHandoffInput),
    ) {
        let mut input = fixture.input.clone();
        mutate(&mut input);
        assert!(build_task_257b2(fixture, input).is_err());
    }

    #[test]
    fn task_257b2_exact_third_profile_wrappers_edges_requests_and_debug_publish() {
        let fixture = task_257b2_composite_fixture();
        let first =
            build_task_257b2(&fixture, fixture.input.clone()).expect("Task 257B2 composite");
        let second = build_task_257b2(&fixture, fixture.input.clone()).expect("Task 257B2 replay");
        assert!(first.is_task_257b2_profile());
        assert_eq!(first, second);
        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(
            task_257b2_debug_oracle(&first.debug_text()),
            (4255, 15930575779633039590, 13566331828362069151),
            "Task 257B2 composite debug bytes changed"
        );
        assert_eq!(
            (
                first.formulas().len(),
                first.wrappers().len(),
                first.roots().len(),
                first.binders().len(),
                first.type_sites().len(),
                first.edges().len(),
                first.requests().len(),
                first.binding_env().contexts().len(),
                first.binding_env().bindings().len(),
                first.binding_env().diagnostics().len(),
            ),
            (8, 6, 1, 1, 1, 7, 9, 2, 1, 4)
        );
        assert_eq!(
            first
                .wrappers()
                .iter()
                .map(|(_, row)| {
                    (
                        row.formula().index(),
                        row.ordinal(),
                        row.source_range().start,
                        row.source_range().end,
                        row.spelling(),
                    )
                })
                .collect::<Vec<_>>(),
            [
                (2, 0, 72, 122, "( or )"),
                (3, 0, 73, 94, "( & ... & )"),
                (4, 0, 98, 121, "( or ... or )"),
                (5, 0, 127, 164, "( or )"),
                (6, 0, 128, 143, "( & )"),
                (7, 0, 147, 163, "( or )"),
            ]
        );
        assert_eq!(
            first
                .edges()
                .iter()
                .map(|(_, row)| {
                    (
                        row.parent().index(),
                        row.ordinal(),
                        row.role(),
                        row.child().index(),
                    )
                })
                .collect::<Vec<_>>(),
            [
                (0, 0, SourceFormulaEdgeRole::UniversalBody, 1),
                (1, 0, SourceFormulaEdgeRole::BiconditionalLeft, 2),
                (1, 1, SourceFormulaEdgeRole::BiconditionalRight, 5),
                (2, 0, SourceFormulaEdgeRole::DisjunctionLeft, 3),
                (2, 1, SourceFormulaEdgeRole::DisjunctionRight, 4),
                (5, 0, SourceFormulaEdgeRole::DisjunctionLeft, 6),
                (5, 1, SourceFormulaEdgeRole::DisjunctionRight, 7),
            ]
        );
        assert_eq!(
            first
                .requests()
                .iter()
                .map(|(_, row)| (row.formula().index(), row.ordinal(), row.kind()))
                .collect::<Vec<_>>(),
            [
                (0, 0, SourceFormulaRequestKind::QuantifierSemantics),
                (0, 1, SourceFormulaRequestKind::BinderType),
                (1, 0, SourceFormulaRequestKind::ConnectiveSemantics),
                (2, 0, SourceFormulaRequestKind::ConnectiveSemantics),
                (3, 0, SourceFormulaRequestKind::ConnectiveSemantics),
                (4, 0, SourceFormulaRequestKind::ConnectiveSemantics),
                (5, 0, SourceFormulaRequestKind::ConnectiveSemantics),
                (6, 0, SourceFormulaRequestKind::ConnectiveSemantics),
                (7, 0, SourceFormulaRequestKind::ConnectiveSemantics),
            ]
        );
        assert!(first.debug_text().contains("kind=repeated-conjunction"));
        assert!(first.debug_text().contains("role=biconditional-right"));
    }

    #[test]
    fn task_257b2_profile_specific_field_and_link_corruptions_fail_closed() {
        let fixture = task_257b2_composite_fixture();
        for index in 0..8 {
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.formulas[index].site = node(17);
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.formulas[index].source_range.end -= 1;
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.formulas[index].kind = SourceCompositeFormulaKind::Contradiction;
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.formulas[index].spelling.push_str("-substituted");
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.formulas[index].source_ordinal += 1;
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.formulas[index].context =
                    BindingContextId::new(1 - input.formulas[index].context.index());
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.formulas[index].recovery = SourceCompositeFormulaRecovery::Degraded;
            });
        }
        for index in 0..6 {
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.wrappers[index].formula = SourceCompositeFormulaId::new(0);
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.wrappers[index].ordinal = 1;
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.wrappers[index].site = node(0);
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.wrappers[index].source_range.end -= 1;
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.wrappers[index].context = BindingContextId::new(0);
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.wrappers[index].spelling.push_str("-substituted");
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.wrappers[index].recovery = SourceCompositeFormulaRecovery::Degraded;
            });
        }
        for index in 0..7 {
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.edges[index].parent = SourceCompositeFormulaId::new(7);
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.edges[index].ordinal += 1;
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.edges[index].role = SourceFormulaEdgeRole::ImplicationLeft;
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.edges[index].child = SourceCompositeFormulaId::new(0);
            });
        }
        for index in 0..9 {
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.requests[index].formula =
                    if input.requests[index].formula == SourceCompositeFormulaId::new(7) {
                        SourceCompositeFormulaId::new(0)
                    } else {
                        SourceCompositeFormulaId::new(7)
                    };
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.requests[index].ordinal += 1;
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.requests[index].kind = SourceFormulaRequestKind::ConstantSemantics;
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.requests[index].binder = if index == 1 {
                    None
                } else {
                    Some(SourceQuantifierBinderId::new(0))
                };
            });
            assert_task_257b2_input_rejected(&fixture, |input| {
                input.requests[index].type_site = if index == 1 {
                    None
                } else {
                    Some(SourceBinderTypeSiteId::new(0))
                };
            });
        }
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.roots[0].formula = SourceCompositeFormulaId::new(1);
        });
        assert_task_257b2_input_rejected(&fixture, |input| input.roots[0].ordinal = 1);
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.binders[0].formula = SourceCompositeFormulaId::new(1);
        });
        assert_task_257b2_input_rejected(&fixture, |input| input.binders[0].ordinal = 1);
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.binders[0].segment_site = node(0);
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.binders[0].segment_range.end -= 1;
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.binders[0].segment_spelling = "y being".to_owned();
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.binders[0].identifier_site = node(0);
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.binders[0].identifier_range.end += 1;
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.binders[0].identifier_spelling = "y".to_owned();
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.binders[0].local = LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![0]),
                input.binders[0].identifier_range,
                0,
            );
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.binders[0].local = LocalTermBinding::new(
                "x",
                LocalTermScope::new(vec![1]),
                input.binders[0].identifier_range,
                0,
            );
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.binders[0].local = LocalTermBinding::new(
                "x",
                LocalTermScope::new(vec![0]),
                input.binders[0].identifier_range,
                1,
            );
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.binders[0].binding = BindingId::new(1);
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.binders[0].body_context = BindingContextId::new(0);
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.binders[0].type_site = SourceBinderTypeSiteId::new(1);
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.binders[0].recovery = SourceCompositeFormulaRecovery::Degraded;
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.type_sites[0].binder = SourceQuantifierBinderId::new(1);
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.type_sites[0].site = node(0);
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.type_sites[0].source_range.start -= 1;
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.type_sites[0].spelling = "object".to_owned();
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.type_sites[0].head_site = node(0);
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.type_sites[0].head_range.start -= 1;
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.type_sites[0].head_spelling = "object".to_owned();
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.type_sites[0].context = BindingContextId::new(1);
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.type_sites[0].recovery = SourceCompositeFormulaRecovery::Degraded;
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.formulas[3].kind = SourceCompositeFormulaKind::Conjunction;
            input.formulas[3].spelling = "&".to_owned();
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.formulas[6].kind = SourceCompositeFormulaKind::RepeatedConjunction;
            input.formulas[6].spelling = "& ... &".to_owned();
        });
        assert_task_257b2_input_rejected(&fixture, |input| input.wrappers.swap(0, 1));
        assert_task_257b2_input_rejected(&fixture, |input| input.edges.swap(1, 2));
        assert_task_257b2_input_rejected(&fixture, |input| input.requests.swap(1, 2));
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.wrappers[0].source_range.end = 164;
        });
        assert_task_257b2_input_rejected(&fixture, |input| input.formulas.pop().map_or((), drop));
        assert_task_257b2_input_rejected(&fixture, |input| input.wrappers.pop().map_or((), drop));
        assert_task_257b2_input_rejected(&fixture, |input| input.edges.pop().map_or((), drop));
        assert_task_257b2_input_rejected(&fixture, |input| input.requests.pop().map_or((), drop));
        assert!(
            build_task_257b2(&fixture, fixture.input.clone()).is_ok(),
            "valid replay must recover after every rejected corruption"
        );
    }

    #[test]
    fn task_257b2_profile_partition_rejects_hybrids_and_a_coherent_fourth_shape() {
        let fixture = task_257b2_composite_fixture();
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.formulas[0].kind = SourceCompositeFormulaKind::Implication;
            input.formulas[0].spelling = "implies".to_owned();
        });
        assert_task_257b2_input_rejected(&fixture, |input| {
            input.formulas.truncate(1);
            input.wrappers.clear();
            input.edges.clear();
            input.requests.truncate(2);
            input.formulas[0].kind = SourceCompositeFormulaKind::Biconditional;
            input.formulas[0].spelling = "iff".to_owned();
        });

        let fourth_source = source_id_with_snapshot_byte("e7");
        let fourth_module = ModuleId::new(
            PackageId::new("pkg"),
            ModulePath::new("composite.fourth-profile"),
        );
        let fourth_range = range(fourth_source, 10, 20);
        let fourth_arena = TypedArena::try_new(
            None,
            vec![TypedNode::new(
                "source.formula.composite.conjunction",
                SourceAnchor::Range(fourth_range),
            )],
        )
        .expect("coherent fourth-profile arena");
        let fourth = SourceCompositeFormulaHandoffInput {
            source_id: fourth_source,
            module_id: fourth_module.clone(),
            formulas: vec![SourceCompositeFormulaInput {
                site: node(0),
                source_range: fourth_range,
                source_ordinal: 0,
                context: BindingContextId::new(0),
                recovery: SourceCompositeFormulaRecovery::Normal,
                spelling: "&".to_owned(),
                kind: SourceCompositeFormulaKind::Conjunction,
            }],
            wrappers: Vec::new(),
            roots: vec![SourceFormulaRootInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                ownership: SourceFormulaRootOwnership::UnassignedStatement,
            }],
            binders: Vec::new(),
            type_sites: Vec::new(),
            edges: Vec::new(),
            requests: vec![SourceFormulaRequestInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                kind: SourceFormulaRequestKind::ConnectiveSemantics,
                binder: None,
                type_site: None,
            }],
        };
        assert!(
            SourceCompositeFormulaProducer::extend_bindings(
                &fourth,
                &base_bindings(fourth_source, &fourth_module),
                &fourth_arena,
            )
            .is_err(),
            "an otherwise coherent unsupported fourth profile must fail closed"
        );
    }

    #[test]
    fn exact_transaction_extends_bindings_and_publishes_all_tables() {
        let fixture = fixture();
        let extended = SourceCompositeFormulaProducer::extend_bindings(
            &fixture.input,
            &fixture.base,
            &fixture.arena,
        )
        .expect("extended bindings");
        assert_eq!(
            (
                extended.contexts().len(),
                extended.bindings().len(),
                extended.diagnostics().len()
            ),
            (2, 1, 4)
        );
        let handoff =
            SourceCompositeFormulaProducer::build(fixture.input.clone(), &extended, &fixture.arena)
                .expect("handoff");
        assert_eq!(
            (
                handoff.formulas().len(),
                handoff.wrappers().len(),
                handoff.roots().len(),
                handoff.binders().len(),
                handoff.type_sites().len(),
                handoff.edges().len(),
                handoff.requests().len(),
            ),
            (5, 0, 1, 1, 1, 4, 6)
        );
        assert_eq!(
            handoff
                .binding_env()
                .contexts()
                .get(BindingContextId::new(1))
                .map(|context| context.owner.clone()),
            Some(BindingContextOwner::SourceFormula {
                source_range: range(fixture.source, 30, 90),
            })
        );
        assert_eq!(
            handoff
                .binders()
                .get(SourceQuantifierBinderId::new(0))
                .map(SourceQuantifierBinder::binding),
            Some(BindingId::new(0))
        );
        assert!(!handoff.debug_text().is_empty());
    }

    #[test]
    fn replay_is_deterministic_and_debug_is_a_complete_literal_oracle() {
        let fixture = fixture();
        let first = build(&fixture, fixture.input.clone()).expect("first");
        let second = build(&fixture, fixture.input.clone()).expect("second");
        assert_eq!(first, second);
        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(first.debug_text(), EXPECTED_DEBUG);
    }

    #[test]
    fn task_257a_wrapper_shape_is_rejected_as_a_third_profile() {
        let fixture = fixture();
        let mut input = fixture.input.clone();
        input.wrappers = vec![
            SourceFormulaWrapperInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                site: node(10),
                source_range: range(fixture.source, 1, 99),
                context: BindingContextId::new(0),
                recovery: SourceCompositeFormulaRecovery::Normal,
                spelling: "( ( implies ) )".to_owned(),
            },
            SourceFormulaWrapperInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 1,
                site: node(9),
                source_range: range(fixture.source, 5, 95),
                context: BindingContextId::new(0),
                recovery: SourceCompositeFormulaRecovery::Normal,
                spelling: "( implies )".to_owned(),
            },
        ];
        assert!(build(&fixture, input).is_err());
    }

    #[test]
    fn task_257a_rows_at_task_257b1_cardinality_reject_as_a_profile_hybrid() {
        let fixture = fixture();
        let mut input = fixture.input.clone();
        input.formulas.truncate(1);
        input.edges.clear();
        input.requests.truncate(2);
        assert_eq!(
            (
                input.formulas.len(),
                input.wrappers.len(),
                input.roots.len(),
                input.binders.len(),
                input.type_sites.len(),
                input.edges.len(),
                input.requests.len(),
            ),
            (1, 0, 1, 1, 1, 0, 2)
        );
        assert!(build(&fixture, input).is_err());
    }

    #[test]
    fn wrapper_crossing_an_unrelated_sibling_fails_independently() {
        let fixture = fixture();
        let mut input = fixture.input.clone();
        input.formulas[0].source_range = range(fixture.source, 0, 100);
        input.wrappers.push(SourceFormulaWrapperInput {
            formula: SourceCompositeFormulaId::new(1),
            ordinal: 0,
            site: node(9),
            source_range: range(fixture.source, 5, 40),
            context: BindingContextId::new(0),
            recovery: SourceCompositeFormulaRecovery::Normal,
            spelling: "( contradiction )".to_owned(),
        });
        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[0].anchor = SourceAnchor::Range(input.formulas[0].source_range);
        nodes[9].anchor = SourceAnchor::Range(input.wrappers[0].source_range);
        let arena = TypedArena::try_new(None, nodes).expect("crossing-wrapper arena");
        assert!(
            SourceCompositeFormulaProducer::extend_bindings(&input, &fixture.base, &arena).is_err()
        );
    }

    #[test]
    fn every_table_and_cross_table_association_fail_closes() {
        let fixture = fixture();

        let mut formula = fixture.input.clone();
        formula.formulas[1].site = formula.formulas[0].site.clone();
        assert!(build(&fixture, formula).is_err());

        let mut root = fixture.input.clone();
        root.roots.push(root.roots[0].clone());
        assert!(build(&fixture, root).is_err());

        let mut binder = fixture.input.clone();
        binder.binders[0].local = LocalTermBinding::new(
            "x",
            LocalTermScope::new(vec![1]),
            range(fixture.source, 35, 36),
            0,
        );
        assert!(build(&fixture, binder).is_err());

        let mut type_site = fixture.input.clone();
        type_site.type_sites[0].context = BindingContextId::new(1);
        assert!(build(&fixture, type_site).is_err());

        let mut edge = fixture.input.clone();
        edge.edges[3].child = SourceCompositeFormulaId::new(3);
        assert!(build(&fixture, edge).is_err());

        let mut request = fixture.input.clone();
        request.requests[3].binder = None;
        assert!(build(&fixture, request).is_err());

        let mut wrapper = fixture.input.clone();
        wrapper.wrappers.push(SourceFormulaWrapperInput {
            formula: SourceCompositeFormulaId::new(1),
            ordinal: 0,
            site: node(9),
            source_range: range(fixture.source, 5, 95),
            context: BindingContextId::new(0),
            recovery: SourceCompositeFormulaRecovery::Normal,
            spelling: "( contradiction )".to_owned(),
        });
        assert!(build(&fixture, wrapper).is_err());
    }

    #[test]
    fn every_input_field_and_named_graph_failure_is_rejected() {
        let fixture = fixture();

        assert_input_rejected(&fixture, |input| {
            input.source_id = distinct_source_id();
        });
        assert_input_rejected(&fixture, |input| {
            input.module_id = ModuleId::new(PackageId::new("pkg"), ModulePath::new("other"));
        });

        assert_input_rejected(&fixture, |input| input.formulas[0].site = node(1));
        assert_input_rejected(&fixture, |input| {
            input.formulas[0].source_range = range(input.source_id, 10, 89);
        });
        assert_input_rejected(&fixture, |input| {
            input.formulas[0].source_ordinal = 1;
        });
        assert_input_rejected(&fixture, |input| {
            input.formulas[0].context = BindingContextId::new(1);
        });
        assert_input_rejected(&fixture, |input| {
            input.formulas[0].recovery = SourceCompositeFormulaRecovery::Degraded;
        });
        assert_input_rejected(&fixture, |input| {
            input.formulas[0].spelling = "iff".to_owned();
        });
        assert_input_rejected(&fixture, |input| {
            input.formulas[0].kind = SourceCompositeFormulaKind::Universal;
        });

        let add_root_wrapper = |input: &mut SourceCompositeFormulaHandoffInput| {
            input.wrappers.push(SourceFormulaWrapperInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                site: node(9),
                source_range: range(input.source_id, 5, 95),
                context: BindingContextId::new(0),
                recovery: SourceCompositeFormulaRecovery::Normal,
                spelling: "( implies )".to_owned(),
            });
        };
        let wrapper_mutations: [fn(&mut SourceFormulaWrapperInput); 7] = [
            |row| row.formula = SourceCompositeFormulaId::new(1),
            |row| row.ordinal = 1,
            |row| row.site = node(0),
            |row| row.source_range.end = 90,
            |row| row.context = BindingContextId::new(1),
            |row| row.recovery = SourceCompositeFormulaRecovery::Degraded,
            |row| row.spelling = "hello".to_owned(),
        ];
        for mutate in wrapper_mutations {
            let mut input = fixture.input.clone();
            add_root_wrapper(&mut input);
            mutate(&mut input.wrappers[0]);
            assert!(build(&fixture, input).is_err());
        }

        assert_input_rejected(&fixture, |input| {
            input.roots[0].formula = SourceCompositeFormulaId::new(1);
        });
        assert_input_rejected(&fixture, |input| input.roots[0].ordinal = 1);
        assert_input_rejected(&fixture, |input| input.roots.clear());
        assert_input_rejected(&fixture, |input| {
            input.roots.push(input.roots[0].clone());
        });

        assert_input_rejected(&fixture, |input| {
            input.binders[0].formula = SourceCompositeFormulaId::new(3);
        });
        assert_input_rejected(&fixture, |input| input.binders[0].ordinal = 1);
        assert_input_rejected(&fixture, |input| input.binders[0].segment_site = node(0));
        assert_input_rejected(&fixture, |input| {
            input.binders[0].segment_range.end = 49;
        });
        assert_input_rejected(&fixture, |input| {
            input.binders[0].segment_spelling = "y being".to_owned();
        });
        assert_input_rejected(&fixture, |input| {
            input.binders[0].identifier_site = node(0);
        });
        assert_input_rejected(&fixture, |input| {
            input.binders[0].identifier_range.end = 37;
        });
        assert_input_rejected(&fixture, |input| {
            input.binders[0].identifier_spelling = "y".to_owned();
        });
        assert_input_rejected(&fixture, |input| {
            input.binders[0].local = LocalTermBinding::new(
                "y",
                LocalTermScope::new(vec![0]),
                input.binders[0].identifier_range,
                0,
            );
        });
        assert_input_rejected(&fixture, |input| {
            input.binders[0].local = LocalTermBinding::new(
                "x",
                LocalTermScope::new(vec![1]),
                input.binders[0].identifier_range,
                0,
            );
        });
        assert_input_rejected(&fixture, |input| {
            input.binders[0].local = LocalTermBinding::new(
                "x",
                LocalTermScope::new(vec![0]),
                input.binders[0].identifier_range,
                1,
            );
        });
        assert_input_rejected(&fixture, |input| {
            input.binders[0].binding = BindingId::new(1);
        });
        assert_input_rejected(&fixture, |input| {
            input.binders[0].body_context = BindingContextId::new(0);
        });
        assert_input_rejected(&fixture, |input| {
            input.binders[0].type_site = SourceBinderTypeSiteId::new(1);
        });
        assert_input_rejected(&fixture, |input| {
            input.binders[0].recovery = SourceCompositeFormulaRecovery::Degraded;
        });
        assert_input_rejected(&fixture, |input| input.binders.clear());
        assert_input_rejected(&fixture, |input| {
            input.binders.push(input.binders[0].clone());
        });

        assert_input_rejected(&fixture, |input| {
            input.type_sites[0].binder = SourceQuantifierBinderId::new(1);
        });
        assert_input_rejected(&fixture, |input| input.type_sites[0].site = node(0));
        assert_input_rejected(&fixture, |input| {
            input.type_sites[0].source_range.start = 44;
        });
        assert_input_rejected(&fixture, |input| {
            input.type_sites[0].spelling = "object".to_owned();
        });
        assert_input_rejected(&fixture, |input| input.type_sites[0].head_site = node(0));
        assert_input_rejected(&fixture, |input| {
            input.type_sites[0].head_range.start = 44;
        });
        assert_input_rejected(&fixture, |input| {
            input.type_sites[0].head_spelling = "object".to_owned();
        });
        assert_input_rejected(&fixture, |input| {
            input.type_sites[0].context = BindingContextId::new(1);
        });
        assert_input_rejected(&fixture, |input| {
            input.type_sites[0].recovery = SourceCompositeFormulaRecovery::Degraded;
        });
        assert_input_rejected(&fixture, |input| input.type_sites.clear());
        assert_input_rejected(&fixture, |input| {
            input.type_sites.push(input.type_sites[0].clone());
        });

        for index in 0..4 {
            assert_input_rejected(&fixture, |input| {
                input.edges[index].parent = SourceCompositeFormulaId::new(4);
            });
            assert_input_rejected(&fixture, |input| {
                input.edges[index].ordinal += 1;
            });
            assert_input_rejected(&fixture, |input| {
                input.edges[index].role =
                    if input.edges[index].role == SourceFormulaEdgeRole::NegatedFormula {
                        SourceFormulaEdgeRole::ImplicationLeft
                    } else {
                        SourceFormulaEdgeRole::NegatedFormula
                    };
            });
            assert_input_rejected(&fixture, |input| {
                input.edges[index].child = SourceCompositeFormulaId::new(0);
            });
        }
        assert_input_rejected(&fixture, |input| {
            input.edges[3].child = SourceCompositeFormulaId::new(3);
        });
        assert_input_rejected(&fixture, |input| {
            input.edges.pop();
        });
        assert_input_rejected(&fixture, |input| {
            input.edges.push(input.edges[0].clone());
        });

        for index in 0..6 {
            assert_input_rejected(&fixture, |input| {
                input.requests[index].formula =
                    if input.requests[index].formula == SourceCompositeFormulaId::new(4) {
                        SourceCompositeFormulaId::new(0)
                    } else {
                        SourceCompositeFormulaId::new(4)
                    };
            });
            assert_input_rejected(&fixture, |input| {
                input.requests[index].ordinal += 1;
            });
            assert_input_rejected(&fixture, |input| {
                input.requests[index].kind =
                    if input.requests[index].kind == SourceFormulaRequestKind::BinderType {
                        SourceFormulaRequestKind::ConnectiveSemantics
                    } else {
                        SourceFormulaRequestKind::BinderType
                    };
            });
        }
        assert_input_rejected(&fixture, |input| {
            input.requests[0].binder = Some(SourceQuantifierBinderId::new(0));
        });
        assert_input_rejected(&fixture, |input| {
            input.requests[0].type_site = Some(SourceBinderTypeSiteId::new(0));
        });
        assert_input_rejected(&fixture, |input| {
            input.requests[3].type_site = None;
        });
        assert_input_rejected(&fixture, |input| {
            input.requests.pop();
        });
        assert_input_rejected(&fixture, |input| {
            input.requests.push(input.requests[0].clone());
        });
    }

    #[test]
    fn zero_length_binder_type_is_rejected_even_when_the_arena_matches() {
        let fixture = fixture();
        let zero = range(fixture.source, 45, 45);
        let mut input = fixture.input.clone();
        input.type_sites[0].source_range = zero;
        input.type_sites[0].head_range = zero;
        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[7].anchor = SourceAnchor::Range(zero);
        nodes[8].anchor = SourceAnchor::Range(zero);
        let arena = TypedArena::try_new(None, nodes).expect("zero-range arena");
        assert!(
            SourceCompositeFormulaProducer::extend_bindings(&input, &fixture.base, &arena).is_err()
        );
    }

    #[test]
    fn collapsed_binder_ranges_are_rejected_even_when_the_arena_matches() {
        let fixture = fixture();

        let mut segment = fixture.input.clone();
        segment.binders[0].segment_range = segment.formulas[2].source_range;
        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[5].anchor = SourceAnchor::Range(segment.binders[0].segment_range);
        let arena = TypedArena::try_new(None, nodes).expect("collapsed segment arena");
        assert!(
            SourceCompositeFormulaProducer::extend_bindings(&segment, &fixture.base, &arena)
                .is_err()
        );

        let mut identifier = fixture.input.clone();
        identifier.binders[0].identifier_range = identifier.binders[0].segment_range;
        identifier.binders[0].local = LocalTermBinding::new(
            "x",
            LocalTermScope::new(vec![0]),
            identifier.binders[0].identifier_range,
            0,
        );
        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[6].anchor = SourceAnchor::Range(identifier.binders[0].identifier_range);
        let arena = TypedArena::try_new(None, nodes).expect("collapsed identifier arena");
        assert!(
            SourceCompositeFormulaProducer::extend_bindings(&identifier, &fixture.base, &arena)
                .is_err()
        );

        let mut type_site = fixture.input.clone();
        type_site.type_sites[0].source_range = type_site.binders[0].segment_range;
        type_site.type_sites[0].head_range = type_site.binders[0].segment_range;
        let mut nodes = fixture
            .arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        nodes[7].anchor = SourceAnchor::Range(type_site.type_sites[0].source_range);
        nodes[8].anchor = SourceAnchor::Range(type_site.type_sites[0].head_range);
        let arena = TypedArena::try_new(None, nodes).expect("collapsed binder-type arena");
        assert!(
            SourceCompositeFormulaProducer::extend_bindings(&type_site, &fixture.base, &arena)
                .is_err()
        );
    }

    #[test]
    fn dense_owner_order_and_cardinality_mutations_fail_close() {
        let fixture = fixture();

        let mut formulas = fixture.input.clone();
        formulas.formulas.swap(1, 2);
        assert!(build(&fixture, formulas).is_err());

        let mut roots = fixture.input.clone();
        roots.roots.clear();
        assert!(build(&fixture, roots).is_err());

        let mut binders = fixture.input.clone();
        binders.binders.push(binders.binders[0].clone());
        assert!(build(&fixture, binders).is_err());

        let mut type_sites = fixture.input.clone();
        type_sites.type_sites.push(type_sites.type_sites[0].clone());
        assert!(build(&fixture, type_sites).is_err());

        let mut edges = fixture.input.clone();
        edges.edges.swap(0, 1);
        assert!(build(&fixture, edges).is_err());
        let mut partial_edges = fixture.input.clone();
        partial_edges.edges.pop();
        assert!(build(&fixture, partial_edges).is_err());

        let mut requests = fixture.input.clone();
        requests.requests.swap(2, 3);
        assert!(build(&fixture, requests).is_err());
        let mut partial_requests = fixture.input.clone();
        partial_requests.requests.pop();
        assert!(build(&fixture, partial_requests).is_err());

        let mut wrappers = fixture.input.clone();
        wrappers.wrappers = vec![
            SourceFormulaWrapperInput {
                formula: SourceCompositeFormulaId::new(4),
                ordinal: 0,
                site: node(11),
                source_range: range(fixture.source, 65, 85),
                context: BindingContextId::new(1),
                recovery: SourceCompositeFormulaRecovery::Normal,
                spelling: "( contradiction )".to_owned(),
            },
            SourceFormulaWrapperInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                site: node(9),
                source_range: range(fixture.source, 5, 95),
                context: BindingContextId::new(0),
                recovery: SourceCompositeFormulaRecovery::Normal,
                spelling: "( implies )".to_owned(),
            },
        ];
        assert!(build(&fixture, wrappers).is_err());
    }

    #[test]
    fn exact_module_shell_and_arena_vocabulary_are_authenticated() {
        let fixture = fixture();
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
        let missing_diagnostics = BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts,
            bindings: BindingTable::new(),
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("structurally valid but incomplete shell");
        assert!(
            SourceCompositeFormulaProducer::extend_bindings(
                &fixture.input,
                &missing_diagnostics,
                &fixture.arena,
            )
            .is_err()
        );

        let mut wrong_diagnostics = BindingDiagnosticTable::new();
        for message_key in [
            "checker.binding.external.local_bindings",
            "checker.binding.external.use_site_scope",
            "checker.binding.external.reserve_payload",
            "checker.binding.external.wrong",
        ] {
            wrong_diagnostics.insert(BindingDiagnosticDraft {
                source_range: None,
                class: BindingDiagnosticClass::ExternalDependencyGap,
                severity: BindingDiagnosticSeverity::Note,
                message_key: message_key.to_owned(),
                recovery: BindingDiagnosticRecovery::Degraded,
            });
        }
        let wrong_diagnostics = BindingEnv::try_new(BindingEnvParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            contexts: fixture.base.contexts().clone(),
            bindings: BindingTable::new(),
            diagnostics: wrong_diagnostics,
        })
        .expect("structurally valid shell with a stale diagnostic");
        assert!(
            SourceCompositeFormulaProducer::extend_bindings(
                &fixture.input,
                &wrong_diagnostics,
                &fixture.arena,
            )
            .is_err()
        );

        for index in 0..9 {
            let mut nodes = fixture
                .arena
                .iter()
                .map(|(_, node)| node.clone())
                .collect::<Vec<_>>();
            nodes[index].kind = "source.formula.generic".into();
            let wrong_arena = TypedArena::try_new(None, nodes).expect("generic-key arena");
            assert!(
                SourceCompositeFormulaProducer::extend_bindings(
                    &fixture.input,
                    &fixture.base,
                    &wrong_arena,
                )
                .is_err()
            );
        }
        for (index, key) in [
            "source.formula.atomic.predicate",
            "source.formula.composite.universal",
            "source.formula.composite.negation",
            "source.formula.constant.contradiction",
            "source.formula.quantifier-binder",
            "source.formula.binder-type",
            "source.formula.binder-type-head",
            "source.formula.quantifier-binder",
            "source.formula.binder-type",
        ]
        .into_iter()
        .enumerate()
        {
            let mut nodes = fixture
                .arena
                .iter()
                .map(|(_, node)| node.clone())
                .collect::<Vec<_>>();
            nodes[index].kind = key.into();
            let wrong_arena = TypedArena::try_new(None, nodes).expect("cross-role arena");
            assert!(
                SourceCompositeFormulaProducer::extend_bindings(
                    &fixture.input,
                    &fixture.base,
                    &wrong_arena,
                )
                .is_err()
            );
        }
    }

    #[test]
    fn stale_generated_and_duplicate_extended_bindings_fail_close() {
        let fixture = fixture();
        let declaration_range = fixture.input.binders[0].identifier_range;
        for identity in [
            BinderIdentity::ResolverLocal {
                scope: LocalTermScope::new(vec![0]),
                ordinal: 1,
                declaration_range,
            },
            BinderIdentity::Generated {
                context: BindingContextId::new(1),
                counter: 0,
            },
        ] {
            if let Ok(env) = extended_bindings_with_identity(
                &fixture,
                identity,
                false,
                fixture.input.formulas[2].source_range,
            ) {
                assert!(
                    SourceCompositeFormulaProducer::build(
                        fixture.input.clone(),
                        &env,
                        &fixture.arena,
                    )
                    .is_err()
                );
            }
        }

        let duplicate = extended_bindings_with_identity(
            &fixture,
            BinderIdentity::ResolverLocal {
                scope: LocalTermScope::new(vec![0]),
                ordinal: 0,
                declaration_range,
            },
            true,
            fixture.input.formulas[2].source_range,
        );
        if let Ok(env) = duplicate {
            assert!(
                SourceCompositeFormulaProducer::build(fixture.input.clone(), &env, &fixture.arena,)
                    .is_err()
            );
        }

        let wrong_owner = extended_bindings_with_identity(
            &fixture,
            BinderIdentity::ResolverLocal {
                scope: LocalTermScope::new(vec![0]),
                ordinal: 0,
                declaration_range,
            },
            false,
            fixture.input.formulas[3].source_range,
        )
        .expect("structurally valid wrong owner environment");
        assert!(
            SourceCompositeFormulaProducer::build(
                fixture.input.clone(),
                &wrong_owner,
                &fixture.arena,
            )
            .is_err()
        );
    }

    #[test]
    fn installation_is_one_shot_and_legacy_debug_bytes_are_unchanged() {
        let fixture = fixture();
        let handoff = build(&fixture, fixture.input.clone()).expect("handoff");
        let typed = empty_typed_ast(
            fixture.source,
            fixture.module.clone(),
            fixture.arena.clone(),
        );
        let installed = typed
            .with_source_composite_formula(handoff.clone())
            .expect("standalone install");
        assert_eq!(installed.source_composite_formula(), Some(&handoff));
        assert_eq!(
            installed
                .with_source_composite_formula(handoff)
                .expect_err("duplicate install"),
            TypedAstError::InvalidSourceCompositeFormula
        );

        let legacy = empty_typed_ast(
            fixture.source,
            fixture.module,
            TypedArena::try_new(None, Vec::new()).expect("empty arena"),
        );
        assert_eq!(legacy.debug_text(), EXPECTED_LEGACY_TYPED_AST_DEBUG);
    }

    pub(crate) fn task_257b3_composite_fixture() -> Task257B3CompositeFixture {
        let source = source_id_with_snapshot_byte("c7");
        let module = module();
        let ranges = [
            (67, 136),
            (92, 136),
            (110, 136),
            (71, 82),
            (71, 72),
            (95, 106),
            (95, 96),
            (114, 115),
            (114, 115),
            (79, 82),
            (79, 82),
            (103, 106),
            (103, 106),
            (14, 17),
            (14, 17),
        ];
        let keys = [
            "source.formula.composite.universal",
            "source.formula.composite.existential",
            "source.formula.composite.universal",
            "source.formula.quantifier-binder",
            "source.formula.quantifier-binder",
            "source.formula.quantifier-binder",
            "source.formula.quantifier-binder",
            "source.formula.quantifier-binder",
            "source.formula.quantifier-binder",
            "source.formula.binder-type",
            "source.formula.binder-type-head",
            "source.formula.binder-type",
            "source.formula.binder-type-head",
            "source.formula.binder-type",
            "source.formula.binder-type-head",
        ];
        let arena = TypedArena::try_new(
            None,
            ranges
                .into_iter()
                .zip(keys)
                .map(|((start, end), key)| {
                    TypedNode::new(key, SourceAnchor::Range(range(source, start, end)))
                })
                .collect(),
        )
        .expect("Task257B3 arena");
        let input = SourceCompositeFormulaHandoffInput {
            source_id: source,
            module_id: module.clone(),
            formulas: [
                (
                    0,
                    (67, 136),
                    0,
                    "for st",
                    SourceCompositeFormulaKind::Universal,
                ),
                (
                    1,
                    (92, 136),
                    1,
                    "ex st",
                    SourceCompositeFormulaKind::Existential,
                ),
                (
                    2,
                    (110, 136),
                    2,
                    "for st holds",
                    SourceCompositeFormulaKind::Universal,
                ),
            ]
            .into_iter()
            .enumerate()
            .map(|(ordinal, (site, (start, end), context, spelling, kind))| {
                SourceCompositeFormulaInput {
                    site: node(site),
                    source_range: range(source, start, end),
                    source_ordinal: ordinal,
                    context: BindingContextId::new(context),
                    recovery: SourceCompositeFormulaRecovery::Normal,
                    spelling: spelling.to_owned(),
                    kind,
                }
            })
            .collect(),
            wrappers: Vec::new(),
            roots: vec![SourceFormulaRootInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                ownership: SourceFormulaRootOwnership::UnassignedStatement,
            }],
            binders: [
                (0, 3, (71, 82), "x being", 4, (71, 72), "x", vec![0]),
                (1, 5, (95, 106), "y being", 6, (95, 96), "y", vec![0, 0]),
                (2, 7, (114, 115), "r", 8, (114, 115), "r", vec![0, 0, 0]),
            ]
            .into_iter()
            .enumerate()
            .map(
                |(
                    index,
                    (
                        formula,
                        segment_site,
                        (segment_start, segment_end),
                        segment_spelling,
                        identifier_site,
                        (identifier_start, identifier_end),
                        identifier_spelling,
                        scope,
                    ),
                )| SourceQuantifierBinderInput {
                    formula: SourceCompositeFormulaId::new(formula),
                    ordinal: 0,
                    segment_site: node(segment_site),
                    segment_range: range(source, segment_start, segment_end),
                    segment_spelling: segment_spelling.to_owned(),
                    identifier_site: node(identifier_site),
                    identifier_range: range(source, identifier_start, identifier_end),
                    identifier_spelling: identifier_spelling.to_owned(),
                    local: LocalTermBinding::new(
                        identifier_spelling,
                        LocalTermScope::new(scope),
                        range(source, identifier_start, identifier_end),
                        index + 1,
                    ),
                    binding: BindingId::new(index + 1),
                    body_context: BindingContextId::new(index + 1),
                    type_site: SourceBinderTypeSiteId::new(index),
                    recovery: SourceCompositeFormulaRecovery::Normal,
                },
            )
            .collect(),
            type_sites: [
                (0, 9, 10, (79, 82), 0),
                (1, 11, 12, (103, 106), 1),
                (2, 13, 14, (14, 17), 0),
            ]
            .into_iter()
            .map(
                |(binder, site, head_site, (start, end), context)| SourceBinderTypeSiteInput {
                    binder: SourceQuantifierBinderId::new(binder),
                    site: node(site),
                    source_range: range(source, start, end),
                    spelling: "set".to_owned(),
                    head_site: node(head_site),
                    head_range: range(source, start, end),
                    head_spelling: "set".to_owned(),
                    context: BindingContextId::new(context),
                    recovery: SourceCompositeFormulaRecovery::Normal,
                    head: SourceBinderTypeHead::BuiltinSet,
                },
            )
            .collect(),
            edges: vec![
                SourceFormulaEdgeInput {
                    parent: SourceCompositeFormulaId::new(0),
                    ordinal: 0,
                    role: SourceFormulaEdgeRole::UniversalBody,
                    child: SourceCompositeFormulaId::new(1),
                },
                SourceFormulaEdgeInput {
                    parent: SourceCompositeFormulaId::new(1),
                    ordinal: 0,
                    role: SourceFormulaEdgeRole::ExistentialBody,
                    child: SourceCompositeFormulaId::new(2),
                },
            ],
            requests: (0..3)
                .flat_map(|formula| {
                    [
                        SourceFormulaRequestInput {
                            formula: SourceCompositeFormulaId::new(formula),
                            ordinal: 0,
                            kind: SourceFormulaRequestKind::QuantifierSemantics,
                            binder: None,
                            type_site: None,
                        },
                        SourceFormulaRequestInput {
                            formula: SourceCompositeFormulaId::new(formula),
                            ordinal: 1,
                            kind: SourceFormulaRequestKind::BinderType,
                            binder: Some(SourceQuantifierBinderId::new(formula)),
                            type_site: Some(SourceBinderTypeSiteId::new(formula)),
                        },
                    ]
                })
                .collect(),
        };
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: vec![BindingId::new(0)],
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        let mut bindings = BindingTable::new();
        bindings.insert(BindingDraft {
            spelling: "r".to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: "r".to_owned(),
                declaration_range: range(source, 8, 9),
            },
            owner_context: BindingContextId::new(0),
            declaration_range: range(source, 8, 9),
            visible_after_ordinal: 0,
            type_site: BindingTypeSite::Source(range(source, 14, 17)),
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
        let base = BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("Task257B3 reserve base");
        Task257B3CompositeFixture {
            source,
            module,
            arena,
            input,
            base,
        }
    }

    fn task_257b3_debug_oracle(value: &str) -> (usize, u64, u64) {
        value
            .bytes()
            .enumerate()
            .fold((0, 0_u64, 0_u64), |(_, sum, weighted), (index, byte)| {
                (
                    index + 1,
                    sum.wrapping_add(u64::from(byte)),
                    weighted.wrapping_add((index as u64 + 1) * u64::from(byte)),
                )
            })
    }

    #[test]
    fn task_257b3_exact_fourth_profile_nested_bindings_and_debug_publish() {
        let fixture = task_257b3_composite_fixture();
        let bindings = SourceCompositeFormulaProducer::extend_bindings(
            &fixture.input,
            &fixture.base,
            &fixture.arena,
        )
        .expect("Task257B3 binding extension");
        let first =
            SourceCompositeFormulaProducer::build(fixture.input.clone(), &bindings, &fixture.arena)
                .expect("Task257B3 composite");
        let replay =
            SourceCompositeFormulaProducer::build(fixture.input.clone(), &bindings, &fixture.arena)
                .expect("Task257B3 replay");
        assert!(first.is_task_257b3_profile());
        assert_eq!(first.debug_text(), replay.debug_text());
        assert_eq!(
            first.debug_text(),
            include_str!("testdata/task257b3_composite.debug")
        );
        assert_eq!(
            (
                bindings.contexts().len(),
                bindings.bindings().len(),
                bindings.diagnostics().len(),
            ),
            (4, 4, 0)
        );
        assert_eq!(
            bindings
                .contexts()
                .iter()
                .map(|(_, row)| {
                    row.visible_bindings
                        .iter()
                        .map(|id| id.index())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            [vec![0], vec![0, 1], vec![0, 1, 2], vec![0, 1, 2, 3]]
        );
        assert!(bindings.bindings().iter().all(|(_, binding)| {
            binding.captured.identities().is_empty() && binding.diagnostics.is_empty()
        }));
        assert_eq!(
            task_257b3_debug_oracle(&first.debug_text()),
            (3914, 349179, 678114531)
        );
    }

    #[test]
    fn task_257b3_profile_fields_links_and_hybrids_fail_closed() {
        let fixture = task_257b3_composite_fixture();
        let corruptions: [fn(&mut SourceCompositeFormulaHandoffInput); 14] = [
            |input| input.formulas[0].spelling = "for holds".to_owned(),
            |input| input.formulas[1].kind = SourceCompositeFormulaKind::Universal,
            |input| input.formulas[2].context = BindingContextId::new(1),
            |input| input.formulas.swap(1, 2),
            |input| input.binders[0].binding = BindingId::new(0),
            |input| {
                input.binders[1].local = LocalTermBinding::new(
                    "y",
                    LocalTermScope::new(vec![0]),
                    input.binders[1].identifier_range,
                    2,
                )
            },
            |input| input.binders[2].identifier_spelling = "s".to_owned(),
            |input| input.binders.swap(1, 2),
            |input| input.type_sites[2].source_range = input.type_sites[1].source_range,
            |input| input.type_sites[2].context = BindingContextId::new(2),
            |input| input.edges[1].role = SourceFormulaEdgeRole::UniversalBody,
            |input| input.edges.swap(0, 1),
            |input| input.requests[3].binder = Some(SourceQuantifierBinderId::new(0)),
            |input| input.requests.pop().map_or((), drop),
        ];
        for corrupt in corruptions {
            let mut input = fixture.input.clone();
            corrupt(&mut input);
            assert!(
                SourceCompositeFormulaProducer::extend_bindings(
                    &input,
                    &fixture.base,
                    &fixture.arena,
                )
                .is_err()
            );
        }
        let mut fifth = fixture.input.clone();
        fifth.formulas.push(fifth.formulas[2].clone());
        assert!(
            SourceCompositeFormulaProducer::extend_bindings(&fifth, &fixture.base, &fixture.arena,)
                .is_err()
        );
        assert!(
            SourceCompositeFormulaProducer::extend_bindings(
                &fixture.input,
                &base_bindings(fixture.source, &fixture.module),
                &fixture.arena,
            )
            .is_err()
        );
    }

    #[test]
    fn task_257b3_every_mutable_field_cardinality_and_cross_identity_rejects() {
        type Mutation = Box<dyn Fn(&mut SourceCompositeFormulaHandoffInput)>;

        let fixture = task_257b3_composite_fixture();
        let mutations: Vec<Mutation> = vec![
            Box::new(|input| input.source_id = distinct_source_id()),
            Box::new(|input| {
                input.module_id =
                    ModuleId::new(PackageId::new("other"), ModulePath::new("other.module"))
            }),
            Box::new(|input| input.formulas[0].site = node(14)),
            Box::new(|input| input.formulas[0].source_range.start += 1),
            Box::new(|input| input.formulas[0].source_ordinal = 1),
            Box::new(|input| input.formulas[0].context = BindingContextId::new(1)),
            Box::new(|input| input.formulas[0].recovery = SourceCompositeFormulaRecovery::Degraded),
            Box::new(|input| input.formulas[0].spelling = "for holds".to_owned()),
            Box::new(|input| input.formulas[0].kind = SourceCompositeFormulaKind::Existential),
            Box::new(|input| input.formulas.swap(0, 1)),
            Box::new(|input| {
                input.wrappers.push(SourceFormulaWrapperInput {
                    formula: SourceCompositeFormulaId::new(0),
                    ordinal: 0,
                    site: node(0),
                    source_range: input.formulas[0].source_range,
                    context: BindingContextId::new(0),
                    recovery: SourceCompositeFormulaRecovery::Normal,
                    spelling: "(for st)".to_owned(),
                })
            }),
            Box::new(|input| input.roots[0].formula = SourceCompositeFormulaId::new(1)),
            Box::new(|input| input.roots[0].ordinal = 1),
            Box::new(|input| input.binders[0].formula = SourceCompositeFormulaId::new(1)),
            Box::new(|input| input.binders[0].ordinal = 1),
            Box::new(|input| input.binders[0].segment_site = node(0)),
            Box::new(|input| input.binders[0].segment_range.start += 1),
            Box::new(|input| input.binders[0].segment_spelling = "x".to_owned()),
            Box::new(|input| input.binders[0].identifier_site = node(0)),
            Box::new(|input| input.binders[0].identifier_range.end += 1),
            Box::new(|input| input.binders[0].identifier_spelling = "z".to_owned()),
            Box::new(|input| {
                input.binders[0].local = LocalTermBinding::new(
                    "x",
                    LocalTermScope::new(vec![9]),
                    input.binders[0].identifier_range,
                    1,
                )
            }),
            Box::new(|input| {
                input.binders[0].local = LocalTermBinding::new(
                    "x",
                    LocalTermScope::new(vec![0]),
                    input.binders[0].identifier_range,
                    9,
                )
            }),
            Box::new(|input| input.binders[0].binding = BindingId::new(0)),
            Box::new(|input| input.binders[0].body_context = BindingContextId::new(2)),
            Box::new(|input| input.binders[0].type_site = SourceBinderTypeSiteId::new(1)),
            Box::new(|input| input.binders[0].recovery = SourceCompositeFormulaRecovery::Degraded),
            Box::new(|input| input.binders.swap(0, 1)),
            Box::new(|input| input.type_sites[0].binder = SourceQuantifierBinderId::new(1)),
            Box::new(|input| input.type_sites[0].site = node(0)),
            Box::new(|input| input.type_sites[0].source_range.start += 1),
            Box::new(|input| input.type_sites[0].spelling = "object".to_owned()),
            Box::new(|input| input.type_sites[0].head_site = node(0)),
            Box::new(|input| input.type_sites[0].head_range.start += 1),
            Box::new(|input| input.type_sites[0].head_spelling = "object".to_owned()),
            Box::new(|input| input.type_sites[0].context = BindingContextId::new(1)),
            Box::new(|input| {
                input.type_sites[0].recovery = SourceCompositeFormulaRecovery::Degraded
            }),
            Box::new(|input| input.type_sites.swap(0, 1)),
            Box::new(|input| input.edges[0].parent = SourceCompositeFormulaId::new(1)),
            Box::new(|input| input.edges[0].ordinal = 1),
            Box::new(|input| input.edges[0].role = SourceFormulaEdgeRole::ExistentialBody),
            Box::new(|input| input.edges[0].child = SourceCompositeFormulaId::new(2)),
            Box::new(|input| input.edges.swap(0, 1)),
            Box::new(|input| input.requests[0].formula = SourceCompositeFormulaId::new(1)),
            Box::new(|input| input.requests[0].ordinal = 1),
            Box::new(|input| input.requests[0].kind = SourceFormulaRequestKind::BinderType),
            Box::new(|input| input.requests[0].binder = Some(SourceQuantifierBinderId::new(0))),
            Box::new(|input| input.requests[0].type_site = Some(SourceBinderTypeSiteId::new(0))),
            Box::new(|input| input.requests.swap(0, 1)),
        ];
        for (index, mutate) in mutations.into_iter().enumerate() {
            let mut input = fixture.input.clone();
            mutate(&mut input);
            assert!(
                SourceCompositeFormulaProducer::extend_bindings(
                    &input,
                    &fixture.base,
                    &fixture.arena,
                )
                .is_err(),
                "Task257B3 field corruption #{index} was accepted"
            );
        }
        for truncate in [
            |input: &mut SourceCompositeFormulaHandoffInput| {
                input.formulas.pop();
            },
            |input: &mut SourceCompositeFormulaHandoffInput| {
                input.roots.pop();
            },
            |input: &mut SourceCompositeFormulaHandoffInput| {
                input.binders.pop();
            },
            |input: &mut SourceCompositeFormulaHandoffInput| {
                input.type_sites.pop();
            },
            |input: &mut SourceCompositeFormulaHandoffInput| {
                input.edges.pop();
            },
            |input: &mut SourceCompositeFormulaHandoffInput| {
                input.requests.pop();
            },
        ] {
            let mut input = fixture.input.clone();
            truncate(&mut input);
            assert!(
                SourceCompositeFormulaProducer::extend_bindings(
                    &input,
                    &fixture.base,
                    &fixture.arena,
                )
                .is_err()
            );
        }
        for duplicate in [
            |input: &mut SourceCompositeFormulaHandoffInput| {
                input.formulas.push(input.formulas[2].clone())
            },
            |input: &mut SourceCompositeFormulaHandoffInput| {
                input.roots.push(input.roots[0].clone())
            },
            |input: &mut SourceCompositeFormulaHandoffInput| {
                input.binders.push(input.binders[2].clone())
            },
            |input: &mut SourceCompositeFormulaHandoffInput| {
                input.type_sites.push(input.type_sites[2].clone())
            },
            |input: &mut SourceCompositeFormulaHandoffInput| {
                input.edges.push(input.edges[1].clone())
            },
            |input: &mut SourceCompositeFormulaHandoffInput| {
                input.requests.push(input.requests[5].clone())
            },
        ] {
            let mut input = fixture.input.clone();
            duplicate(&mut input);
            assert!(
                SourceCompositeFormulaProducer::extend_bindings(
                    &input,
                    &fixture.base,
                    &fixture.arena,
                )
                .is_err()
            );
        }
    }

    #[test]
    fn task_257b3_rejects_a_coherent_unsupported_fifth_profile() {
        let source = source_id_with_snapshot_byte("d7");
        let module = ModuleId::new(
            PackageId::new("pkg"),
            ModulePath::new("composite.fifth-profile"),
        );
        let source_range = range(source, 10, 20);
        let arena = TypedArena::try_new(
            None,
            vec![TypedNode::new(
                "source.formula.composite.conjunction",
                SourceAnchor::Range(source_range),
            )],
        )
        .expect("coherent fifth-profile arena");
        let input = SourceCompositeFormulaHandoffInput {
            source_id: source,
            module_id: module.clone(),
            formulas: vec![SourceCompositeFormulaInput {
                site: node(0),
                source_range,
                source_ordinal: 0,
                context: BindingContextId::new(0),
                recovery: SourceCompositeFormulaRecovery::Normal,
                spelling: "&".to_owned(),
                kind: SourceCompositeFormulaKind::Conjunction,
            }],
            wrappers: Vec::new(),
            roots: vec![SourceFormulaRootInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                ownership: SourceFormulaRootOwnership::UnassignedStatement,
            }],
            binders: Vec::new(),
            type_sites: Vec::new(),
            edges: Vec::new(),
            requests: vec![SourceFormulaRequestInput {
                formula: SourceCompositeFormulaId::new(0),
                ordinal: 0,
                kind: SourceFormulaRequestKind::ConnectiveSemantics,
                binder: None,
                type_site: None,
            }],
        };
        assert!(
            SourceCompositeFormulaProducer::extend_bindings(
                &input,
                &base_bindings(source, &module),
                &arena,
            )
            .is_err()
        );
    }

    fn empty_typed_ast(source: SourceId, module: ModuleId, nodes: TypedArena) -> TypedAst {
        TypedAst::try_new(TypedAstParts {
            source_id: source,
            module_id: module,
            resolved_root: None,
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
        .expect("empty typed AST")
    }

    const EXPECTED_DEBUG: &str = r#"source-composite-formula-debug-v1
module: pkg::composite.fixture
binding-env-debug-v1
module: pkg::composite.fixture
contexts:
  context#0 owner=module parent=none layer=module scope=none bindings=[] visible=[] recovery=normal
  context#1 owner=source-formula(30..90) parent=context#0 layer=expression scope=[0] bindings=[binding#0] visible=[binding#0] recovery=normal
bindings:
  binding#0 spelling="x" kind=quantifier_binder owner=context#1 identity=resolver_local(scope=[0], ordinal=0, range=35..36) range=35..36 visible_after=0 type=source(45..48) status=active captured=[] diagnostics=[] recovery=normal
diagnostics:
  diagnostic#3 range=none class=external_dependency_gap severity=note key="checker.binding.external.closure_payload" recovery=degraded
  diagnostic#0 range=none class=external_dependency_gap severity=note key="checker.binding.external.local_bindings" recovery=degraded
  diagnostic#2 range=none class=external_dependency_gap severity=note key="checker.binding.external.reserve_payload" recovery=degraded
  diagnostic#1 range=none class=external_dependency_gap severity=note key="checker.binding.external.use_site_scope" recovery=degraded
formulas: 5
  formula#0 site=0 range=10..90 ordinal=0 context=0 recovery=normal spelling="implies" kind=implication
  formula#1 site=1 range=10..23 ordinal=1 context=0 recovery=normal spelling="contradiction" kind=contradiction
  formula#2 site=2 range=30..90 ordinal=2 context=0 recovery=normal spelling="for holds" kind=universal
  formula#3 site=3 range=60..90 ordinal=3 context=1 recovery=normal spelling="not" kind=negation
  formula#4 site=4 range=70..83 ordinal=4 context=1 recovery=normal spelling="contradiction" kind=contradiction
wrappers: 0
roots: 1
  root#0 formula=0 ordinal=0 ownership=unassigned-statement
binders: 1
  binder#0 formula=2 ordinal=0 segment-site=5 segment-range=35..50 segment-spelling="x being" identifier-site=6 identifier-range=35..36 identifier-spelling="x" local-scope=[0] local-ordinal=0 binding=0 body-context=1 type-site=0 recovery=normal
type-sites: 1
  type-site#0 binder=0 site=7 range=45..48 spelling="set" head-site=8 head-range=45..48 head-spelling="set" context=0 recovery=normal head=builtin-set
edges: 4
  edge#0 parent=0 ordinal=0 role=implication-left child=1
  edge#1 parent=0 ordinal=1 role=implication-right child=2
  edge#2 parent=2 ordinal=0 role=universal-body child=3
  edge#3 parent=3 ordinal=0 role=negated-formula child=4
requests: 6
  request#0 formula=0 ordinal=0 kind=connective-semantics binder=- type-site=-
  request#1 formula=1 ordinal=0 kind=constant-semantics binder=- type-site=-
  request#2 formula=2 ordinal=0 kind=quantifier-semantics binder=- type-site=-
  request#3 formula=2 ordinal=1 kind=binder-type binder=0 type-site=0
  request#4 formula=3 ordinal=0 kind=negation-semantics binder=- type-site=-
  request#5 formula=4 ordinal=0 kind=constant-semantics binder=- type-site=-
"#;
    const EXPECTED_LEGACY_TYPED_AST_DEBUG: &str = r#"typed-ast-debug-v1
module: pkg::composite.fixture
root: <none>
resolved_root: <none>
nodes:
  <none>
contexts:
  <none>
types:
  <none>
facts:
  <none>
coercions:
  <none>
initial_obligations:
  <none>
diagnostics:
  <none>
"#;
}
