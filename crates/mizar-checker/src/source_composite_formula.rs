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
    Negation,
    Contradiction,
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
    NegatedFormula,
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

        let binder = &input.binders[0];
        let type_site = &input.type_sites[0];
        let mut bindings = base_bindings.bindings().clone();
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
                binder: SourceQuantifierBinderId::new(0),
            });
        }

        let mut contexts = base_bindings.contexts().clone();
        let context = contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::SourceFormula {
                source_range: input.formulas[binder.formula.index()].source_range,
            },
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Expression,
            lexical_scope: Some(binder.local.scope().clone()),
            bindings: vec![binding],
            visible_bindings: vec![binding],
            recovery: BindingContextRecovery::Normal,
        });
        if context != binder.body_context {
            return Err(SourceCompositeFormulaError::InvalidBinder {
                binder: SourceQuantifierBinderId::new(0),
            });
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
    if input.formulas.len() != 5 {
        return Err(SourceCompositeFormulaError::InvalidTree);
    }
    let expected_formulas = [
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
    let mut sites = BTreeSet::new();
    for (index, (row, (kind, context, spelling))) in
        input.formulas.iter().zip(expected_formulas).enumerate()
    {
        if row.source_ordinal != index
            || row.kind != kind
            || row.context != context
            || row.recovery != SourceCompositeFormulaRecovery::Normal
            || row.spelling != spelling
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

    validate_wrappers(input, arena, &mut sites)?;
    validate_root(input)?;
    validate_binder(input, arena, &mut sites)?;
    validate_type_site(input, arena, &mut sites)?;
    validate_edges(input)?;
    validate_requests(input)?;
    Ok(())
}

fn validate_wrappers(
    input: &SourceCompositeFormulaHandoffInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceCompositeFormulaError> {
    let mut groups: [Vec<usize>; 5] = std::array::from_fn(|_| Vec::new());
    let mut expected_ordinal = [0; 5];
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
            || !wrapper_is_within_parent(input, formula, row.source_range)
            || wrapper_crosses_unrelated_formula(input, formula, row.source_range)
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
    owner: usize,
    wrapper_range: SourceRange,
) -> bool {
    let parent = match owner {
        0 => return true,
        1 | 2 => 0,
        3 => 2,
        4 => 3,
        _ => return false,
    };
    properly_contains(input.formulas[parent].source_range, wrapper_range)
}

fn wrapper_crosses_unrelated_formula(
    input: &SourceCompositeFormulaHandoffInput,
    owner: usize,
    wrapper_range: SourceRange,
) -> bool {
    input.formulas.iter().enumerate().any(|(other, formula)| {
        other != owner
            && !formula_is_ancestor(owner, other)
            && !formula_is_ancestor(other, owner)
            && ranges_overlap(wrapper_range, formula.source_range)
    })
}

fn formula_is_ancestor(ancestor: usize, descendant: usize) -> bool {
    matches!((ancestor, descendant), (0, 1..=4) | (2, 3 | 4) | (3, 4))
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
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceCompositeFormulaError> {
    let [binder] = input.binders.as_slice() else {
        return Err(SourceCompositeFormulaError::InvalidTree);
    };
    let universal = &input.formulas[2];
    if binder.formula != SourceCompositeFormulaId::new(2)
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

fn validate_type_site(
    input: &SourceCompositeFormulaHandoffInput,
    arena: &TypedArena,
    sites: &mut BTreeSet<TypedSiteRef>,
) -> Result<(), SourceCompositeFormulaError> {
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

fn validate_edges(
    input: &SourceCompositeFormulaHandoffInput,
) -> Result<(), SourceCompositeFormulaError> {
    let expected = [
        (0, 0, SourceFormulaEdgeRole::ImplicationLeft, 1),
        (0, 1, SourceFormulaEdgeRole::ImplicationRight, 2),
        (2, 0, SourceFormulaEdgeRole::UniversalBody, 3),
        (3, 0, SourceFormulaEdgeRole::NegatedFormula, 4),
    ];
    if input.edges.len() != expected.len() {
        return Err(SourceCompositeFormulaError::InvalidTree);
    }
    let mut incoming = [0_u8; 5];
    for (index, (row, (parent, ordinal, role, child))) in
        input.edges.iter().zip(expected).enumerate()
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
    if incoming != [0, 1, 1, 1, 1] {
        return Err(SourceCompositeFormulaError::InvalidTree);
    }
    Ok(())
}

fn validate_requests(
    input: &SourceCompositeFormulaHandoffInput,
) -> Result<(), SourceCompositeFormulaError> {
    let expected = [
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
    if input.requests.len() != expected.len() {
        return Err(SourceCompositeFormulaError::InvalidTree);
    }
    for (index, (row, (formula, ordinal, kind, binder, type_site))) in
        input.requests.iter().zip(expected).enumerate()
    {
        if row.formula != SourceCompositeFormulaId::new(formula)
            || row.ordinal != ordinal
            || row.kind != kind
            || row.binder != binder
            || row.type_site != type_site
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
    if root.owner != BindingContextOwner::Module
        || root.parent.is_some()
        || root.layer != BindingContextLayer::Module
        || root.lexical_scope.is_some()
        || !root.bindings.is_empty()
        || !root.visible_bindings.is_empty()
        || root.recovery != BindingContextRecovery::Normal
        || body.owner
            != (BindingContextOwner::SourceFormula {
                source_range: input.formulas[2].source_range,
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
        SourceCompositeFormulaKind::Negation => "source.formula.composite.negation",
        SourceCompositeFormulaKind::Contradiction => "source.formula.constant.contradiction",
    }
}

fn formula_kind_key(kind: SourceCompositeFormulaKind) -> &'static str {
    match kind {
        SourceCompositeFormulaKind::Implication => "implication",
        SourceCompositeFormulaKind::Universal => "universal",
        SourceCompositeFormulaKind::Negation => "negation",
        SourceCompositeFormulaKind::Contradiction => "contradiction",
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
        SourceFormulaEdgeRole::NegatedFormula => "negated-formula",
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
mod tests {
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

    fn build(
        fixture: &Fixture,
        input: SourceCompositeFormulaHandoffInput,
    ) -> Result<SourceCompositeFormulaHandoff, SourceCompositeFormulaError> {
        let extended =
            SourceCompositeFormulaProducer::extend_bindings(&input, &fixture.base, &fixture.arena)?;
        SourceCompositeFormulaProducer::build(input, &extended, &fixture.arena)
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
    fn transparent_nested_wrappers_are_bounded_and_ordered() {
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
        let handoff = build(&fixture, input.clone()).expect("nested wrappers");
        assert_eq!(handoff.wrappers().len(), 2);

        input.wrappers.swap(0, 1);
        input.wrappers[0].ordinal = 0;
        input.wrappers[1].ordinal = 1;
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
