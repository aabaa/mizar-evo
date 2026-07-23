//! Syntax-free transport for source primary-term occurrences.

use crate::{
    binding_env::{BindingEnv, BindingId, BindingKind, BindingLookupResult, BindingLookupSite},
    typed_ast::{NodeRecoveryState, TypedArena, TypedSiteRef},
};
use mizar_lexer::is_identifier;
use mizar_resolve::{names::LocalTermScope, resolved_ast::ModuleId};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use std::{
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

dense_id!(SourcePrimaryTermId);
dense_id!(SourcePrimaryTermReferenceId);
dense_id!(SourceNumericTypeRequestId);

/// Complete input for one source/module primary-term transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrimaryTermHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub terms: Vec<SourcePrimaryTermInput>,
    pub references: Vec<SourcePrimaryTermReferenceInput>,
    pub numeric_type_requests: Vec<SourceNumericTypeRequestInput>,
}

/// One source primary-term occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrimaryTermInput {
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: crate::binding_env::BindingContextId,
    pub recovery: SourcePrimaryTermRecovery,
    pub spelling: String,
    pub kind: SourcePrimaryTermKind,
    pub role: SourcePrimaryTermRole,
    pub parent: Option<SourcePrimaryTermId>,
}

/// One binding reference attached to a primary term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrimaryTermReferenceInput {
    pub term: SourcePrimaryTermId,
    pub binding: BindingId,
    pub role: SourcePrimaryTermReferenceRole,
}

/// One unresolved numeric-type request attached to a numeral.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceNumericTypeRequestInput {
    pub term: SourcePrimaryTermId,
    pub owner: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub request_ordinal: usize,
}

/// Source primary-term shape admitted by Task 252.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourcePrimaryTermKind {
    VariableReference,
    ConstantReference,
    It,
    Numeral,
    Parenthesized,
}

/// Source role of a primary-term occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourcePrimaryTermRole {
    Value,
    CurrentDefinitionResult,
}

/// Binding role of a source reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourcePrimaryTermReferenceRole {
    Variable,
    LocalConstant,
}

/// Recovery state retained at the source-term boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourcePrimaryTermRecovery {
    Normal,
    Degraded,
}

/// Immutable validated primary-term handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrimaryTermHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    terms: SourcePrimaryTermTable,
    references: SourcePrimaryTermReferenceTable,
    numeric_type_requests: SourceNumericTypeRequestTable,
}

impl SourcePrimaryTermHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn terms(&self) -> &SourcePrimaryTermTable {
        &self.terms
    }

    pub const fn references(&self) -> &SourcePrimaryTermReferenceTable {
        &self.references
    }

    pub const fn numeric_type_requests(&self) -> &SourceNumericTypeRequestTable {
        &self.numeric_type_requests
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-primary-term-debug-v1\n");
        let _ = writeln!(output, "module: {}", self.module_id.path().as_str());
        for (id, term) in self.terms.iter() {
            let _ = write!(
                output,
                "term#{} ordinal={} kind={} role={} range={}..{} site={} context={} recovery={} spelling={:?} parent=",
                id.index(),
                term.source_ordinal,
                kind_key(term.kind),
                role_key(term.role),
                term.source_range.start,
                term.source_range.end,
                term.site.node().index(),
                term.context.index(),
                recovery_key(term.recovery),
                term.spelling
            );
            write_optional_term_id(&mut output, term.parent);
            output.push('\n');
        }
        for (id, reference) in self.references.iter() {
            let _ = write!(
                output,
                "reference#{} term={} binding={} role={} use_ordinal={} scope=",
                id.index(),
                reference.term.index(),
                reference.binding.index(),
                reference_role_key(reference.role),
                reference.use_ordinal
            );
            write_scope(&mut output, reference.lexical_scope.as_ref());
            output.push('\n');
        }
        for (id, request) in self.numeric_type_requests.iter() {
            let _ = writeln!(
                output,
                "numeric-request#{} term={} ordinal={} owner={} range={}..{} spelling={:?}",
                id.index(),
                request.term.index(),
                request.request_ordinal,
                request.owner.node().index(),
                request.source_range.start,
                request.source_range.end,
                request.spelling
            );
        }
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
    ) -> Result<(), SourcePrimaryTermError> {
        if self.source_id != source_id || &self.module_id != module_id {
            return Err(SourcePrimaryTermError::InvalidTransaction);
        }
        for (id, term) in self.terms.iter() {
            validate_term_node(term, arena)
                .map_err(|()| SourcePrimaryTermError::InvalidTerm { term: id })?;
        }
        Ok(())
    }
}

/// Dense immutable primary-term table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrimaryTermTable {
    rows: Vec<SourcePrimaryTerm>,
}

impl SourcePrimaryTermTable {
    pub fn get(&self, id: SourcePrimaryTermId) -> Option<&SourcePrimaryTerm> {
        self.rows.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourcePrimaryTermId, &SourcePrimaryTerm)> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourcePrimaryTermId::new(index), row))
    }

    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// One validated source primary-term row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrimaryTerm {
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    context: crate::binding_env::BindingContextId,
    recovery: SourcePrimaryTermRecovery,
    spelling: String,
    kind: SourcePrimaryTermKind,
    role: SourcePrimaryTermRole,
    parent: Option<SourcePrimaryTermId>,
}

impl SourcePrimaryTerm {
    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn context(&self) -> crate::binding_env::BindingContextId {
        self.context
    }

    pub const fn recovery(&self) -> SourcePrimaryTermRecovery {
        self.recovery
    }

    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    pub const fn kind(&self) -> SourcePrimaryTermKind {
        self.kind
    }

    pub const fn role(&self) -> SourcePrimaryTermRole {
        self.role
    }

    pub const fn parent(&self) -> Option<SourcePrimaryTermId> {
        self.parent
    }
}

/// Dense immutable source-reference table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrimaryTermReferenceTable {
    rows: Vec<SourcePrimaryTermReference>,
}

impl SourcePrimaryTermReferenceTable {
    pub fn get(&self, id: SourcePrimaryTermReferenceId) -> Option<&SourcePrimaryTermReference> {
        self.rows.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourcePrimaryTermReferenceId, &SourcePrimaryTermReference)> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourcePrimaryTermReferenceId::new(index), row))
    }

    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// One validated binding reference with producer-derived lookup coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePrimaryTermReference {
    term: SourcePrimaryTermId,
    binding: BindingId,
    role: SourcePrimaryTermReferenceRole,
    lexical_scope: Option<LocalTermScope>,
    use_ordinal: usize,
}

impl SourcePrimaryTermReference {
    pub const fn term(&self) -> SourcePrimaryTermId {
        self.term
    }

    pub const fn binding(&self) -> BindingId {
        self.binding
    }

    pub const fn role(&self) -> SourcePrimaryTermReferenceRole {
        self.role
    }

    pub const fn lexical_scope(&self) -> Option<&LocalTermScope> {
        self.lexical_scope.as_ref()
    }

    pub const fn use_ordinal(&self) -> usize {
        self.use_ordinal
    }
}

/// Dense immutable numeric-type-request table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceNumericTypeRequestTable {
    rows: Vec<SourceNumericTypeRequest>,
}

impl SourceNumericTypeRequestTable {
    pub fn get(&self, id: SourceNumericTypeRequestId) -> Option<&SourceNumericTypeRequest> {
        self.rows.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceNumericTypeRequestId, &SourceNumericTypeRequest)> {
        self.rows
            .iter()
            .enumerate()
            .map(|(index, row)| (SourceNumericTypeRequestId::new(index), row))
    }

    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// One validated unresolved numeric-type request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceNumericTypeRequest {
    term: SourcePrimaryTermId,
    owner: TypedSiteRef,
    source_range: SourceRange,
    spelling: String,
    request_ordinal: usize,
}

impl SourceNumericTypeRequest {
    pub const fn term(&self) -> SourcePrimaryTermId {
        self.term
    }

    pub const fn owner(&self) -> &TypedSiteRef {
        &self.owner
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    pub const fn request_ordinal(&self) -> usize {
        self.request_ordinal
    }
}

/// Atomic Task-252 producer failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePrimaryTermError {
    InvalidTransaction,
    InvalidTerm {
        term: SourcePrimaryTermId,
    },
    InvalidReference {
        reference: SourcePrimaryTermReferenceId,
    },
    InvalidNumericTypeRequest {
        request: SourceNumericTypeRequestId,
    },
    InvalidBindingEvent {
        event: usize,
    },
}

impl fmt::Display for SourcePrimaryTermError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransaction => {
                formatter.write_str("source primary-term transaction is inconsistent")
            }
            Self::InvalidTerm { term } => {
                write!(
                    formatter,
                    "source primary term {} is inconsistent",
                    term.index()
                )
            }
            Self::InvalidReference { reference } => write!(
                formatter,
                "source primary-term reference {} is inconsistent",
                reference.index()
            ),
            Self::InvalidNumericTypeRequest { request } => write!(
                formatter,
                "source numeric-type request {} is inconsistent",
                request.index()
            ),
            Self::InvalidBindingEvent { event } => {
                write!(formatter, "source binding event {event} is inconsistent")
            }
        }
    }
}

impl Error for SourcePrimaryTermError {}

#[derive(Debug, Clone, Copy, Default)]
pub struct SourcePrimaryTermProducer;

impl SourcePrimaryTermProducer {
    pub fn build(
        input: SourcePrimaryTermHandoffInput,
        binding_env: &BindingEnv,
        arena: &TypedArena,
    ) -> Result<SourcePrimaryTermHandoff, SourcePrimaryTermError> {
        if input.source_id != binding_env.source_id() || &input.module_id != binding_env.module_id()
        {
            return Err(SourcePrimaryTermError::InvalidTransaction);
        }

        let terms = validate_terms(input.source_id, &input.terms, binding_env, arena)?;
        let derived_ordinals =
            derive_reference_ordinals(input.source_id, &input.references, &terms, binding_env)?;
        let references =
            validate_references(&input.references, &derived_ordinals, &terms, binding_env)?;
        validate_reference_cardinality(&terms, &references)?;
        let numeric_type_requests =
            validate_numeric_requests(&input.numeric_type_requests, &terms)?;

        Ok(SourcePrimaryTermHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            terms: SourcePrimaryTermTable { rows: terms },
            references: SourcePrimaryTermReferenceTable { rows: references },
            numeric_type_requests: SourceNumericTypeRequestTable {
                rows: numeric_type_requests,
            },
        })
    }
}

fn validate_terms(
    source_id: SourceId,
    inputs: &[SourcePrimaryTermInput],
    binding_env: &BindingEnv,
    arena: &TypedArena,
) -> Result<Vec<SourcePrimaryTerm>, SourcePrimaryTermError> {
    let mut rows: Vec<SourcePrimaryTerm> = Vec::with_capacity(inputs.len());
    let mut child_by_parent = vec![None; inputs.len()];
    let mut seen_sites = std::collections::BTreeSet::new();

    for (index, input) in inputs.iter().enumerate() {
        let id = SourcePrimaryTermId::new(index);
        if input.source_ordinal != index
            || !valid_range(source_id, input.source_range)
            || !seen_sites.insert(input.site.node())
            || binding_env.contexts().get(input.context).is_none()
            || !valid_term_role(input.kind, input.role)
            || !valid_leaf_spelling(input.kind, &input.spelling)
        {
            return Err(SourcePrimaryTermError::InvalidTerm { term: id });
        }

        let row = SourcePrimaryTerm {
            site: input.site.clone(),
            source_range: input.source_range,
            source_ordinal: input.source_ordinal,
            context: input.context,
            recovery: input.recovery,
            spelling: input.spelling.clone(),
            kind: input.kind,
            role: input.role,
            parent: input.parent,
        };
        validate_term_node(&row, arena)
            .map_err(|()| SourcePrimaryTermError::InvalidTerm { term: id })?;

        if let Some(parent) = input.parent {
            let Some(parent_row) = rows.get(parent.index()) else {
                return Err(SourcePrimaryTermError::InvalidTerm { term: id });
            };
            if parent.index() >= index
                || parent_row.kind != SourcePrimaryTermKind::Parenthesized
                || parent_row.context != input.context
                || !strictly_contains(parent_row.source_range, input.source_range)
                || child_by_parent[parent.index()].replace(index).is_some()
            {
                return Err(SourcePrimaryTermError::InvalidTerm { term: id });
            }
        }
        rows.push(row);
    }

    for (index, row) in rows.iter().enumerate() {
        match (row.kind, child_by_parent[index]) {
            (SourcePrimaryTermKind::Parenthesized, Some(child))
                if row.spelling != format!("( {} )", rows[child].spelling) =>
            {
                return Err(SourcePrimaryTermError::InvalidTerm {
                    term: SourcePrimaryTermId::new(index),
                });
            }
            (SourcePrimaryTermKind::Parenthesized, Some(_)) => {}
            (SourcePrimaryTermKind::Parenthesized, None) | (_, Some(_)) => {
                return Err(SourcePrimaryTermError::InvalidTerm {
                    term: SourcePrimaryTermId::new(index),
                });
            }
            _ => {}
        }
    }

    for left in 0..rows.len() {
        for right in left + 1..rows.len() {
            let left_range = rows[left].source_range;
            let right_range = rows[right].source_range;
            if left_range.end <= right_range.start {
                continue;
            }
            if strictly_contains(left_range, right_range)
                && is_ancestor(SourcePrimaryTermId::new(left), &rows[right], &rows)
            {
                continue;
            }
            return Err(SourcePrimaryTermError::InvalidTerm {
                term: SourcePrimaryTermId::new(right),
            });
        }
    }

    Ok(rows)
}

fn validate_references(
    inputs: &[SourcePrimaryTermReferenceInput],
    ordinals: &[usize],
    terms: &[SourcePrimaryTerm],
    binding_env: &BindingEnv,
) -> Result<Vec<SourcePrimaryTermReference>, SourcePrimaryTermError> {
    let mut rows = Vec::with_capacity(inputs.len());
    for (index, (input, use_ordinal)) in inputs.iter().zip(ordinals).enumerate() {
        let id = SourcePrimaryTermReferenceId::new(index);
        let Some(term) = terms.get(input.term.index()) else {
            return Err(SourcePrimaryTermError::InvalidReference { reference: id });
        };
        if !valid_reference_role(term.kind, input.role) {
            return Err(SourcePrimaryTermError::InvalidReference { reference: id });
        }
        let Some(context) = binding_env.contexts().get(term.context) else {
            return Err(SourcePrimaryTermError::InvalidReference { reference: id });
        };
        let lexical_scope = context.lexical_scope.clone();
        let lookup = BindingLookupSite::new(
            term.spelling.clone(),
            term.context,
            lexical_scope.clone(),
            *use_ordinal,
        );
        let Ok(BindingLookupResult::Local(binding)) = binding_env.lookup(&lookup) else {
            return Err(SourcePrimaryTermError::InvalidReference { reference: id });
        };
        let Some(binding_row) = binding_env.bindings().get(input.binding) else {
            return Err(SourcePrimaryTermError::InvalidReference { reference: id });
        };
        if binding != input.binding
            || binding_row.spelling != term.spelling
            || binding_row.declaration_range.end > term.source_range.start
            || !valid_binding_role(binding_row.kind, input.role)
        {
            return Err(SourcePrimaryTermError::InvalidReference { reference: id });
        }
        rows.push(SourcePrimaryTermReference {
            term: input.term,
            binding: input.binding,
            role: input.role,
            lexical_scope,
            use_ordinal: *use_ordinal,
        });
    }
    Ok(rows)
}

fn validate_reference_cardinality(
    terms: &[SourcePrimaryTerm],
    references: &[SourcePrimaryTermReference],
) -> Result<(), SourcePrimaryTermError> {
    let mut counts = vec![0usize; terms.len()];
    for (index, reference) in references.iter().enumerate() {
        let Some(count) = counts.get_mut(reference.term.index()) else {
            return Err(SourcePrimaryTermError::InvalidReference {
                reference: SourcePrimaryTermReferenceId::new(index),
            });
        };
        *count += 1;
        if *count > 1 {
            return Err(SourcePrimaryTermError::InvalidReference {
                reference: SourcePrimaryTermReferenceId::new(index),
            });
        }
    }
    for (index, term) in terms.iter().enumerate() {
        let expected = usize::from(matches!(
            term.kind,
            SourcePrimaryTermKind::VariableReference | SourcePrimaryTermKind::ConstantReference
        ));
        if counts[index] != expected {
            return Err(SourcePrimaryTermError::InvalidTerm {
                term: SourcePrimaryTermId::new(index),
            });
        }
    }
    Ok(())
}

fn validate_numeric_requests(
    inputs: &[SourceNumericTypeRequestInput],
    terms: &[SourcePrimaryTerm],
) -> Result<Vec<SourceNumericTypeRequest>, SourcePrimaryTermError> {
    let mut rows = Vec::with_capacity(inputs.len());
    let mut counts = vec![0usize; terms.len()];
    let mut previous_term = None;
    for (index, input) in inputs.iter().enumerate() {
        let id = SourceNumericTypeRequestId::new(index);
        let Some(term) = terms.get(input.term.index()) else {
            return Err(SourcePrimaryTermError::InvalidNumericTypeRequest { request: id });
        };
        if term.kind != SourcePrimaryTermKind::Numeral
            || input.request_ordinal != index
            || input.owner != term.site
            || input.source_range != term.source_range
            || input.spelling != term.spelling
            || previous_term.is_some_and(|previous| previous >= input.term.index())
        {
            return Err(SourcePrimaryTermError::InvalidNumericTypeRequest { request: id });
        }
        previous_term = Some(input.term.index());
        counts[input.term.index()] += 1;
        if counts[input.term.index()] > 1 {
            return Err(SourcePrimaryTermError::InvalidNumericTypeRequest { request: id });
        }
        rows.push(SourceNumericTypeRequest {
            term: input.term,
            owner: input.owner.clone(),
            source_range: input.source_range,
            spelling: input.spelling.clone(),
            request_ordinal: input.request_ordinal,
        });
    }
    for (index, term) in terms.iter().enumerate() {
        let expected = usize::from(term.kind == SourcePrimaryTermKind::Numeral);
        if counts[index] != expected {
            return Err(SourcePrimaryTermError::InvalidTerm {
                term: SourcePrimaryTermId::new(index),
            });
        }
    }
    Ok(rows)
}

fn derive_reference_ordinals(
    source_id: SourceId,
    references: &[SourcePrimaryTermReferenceInput],
    terms: &[SourcePrimaryTerm],
    binding_env: &BindingEnv,
) -> Result<Vec<usize>, SourcePrimaryTermError> {
    let bindings = binding_env.bindings().iter().collect::<Vec<_>>();
    let mut group_start = 0;
    while group_start < bindings.len() {
        let (_, first) = bindings[group_start];
        if !valid_range(source_id, first.declaration_range)
            || group_start > 0
                && bindings[group_start - 1].1.declaration_range.end > first.declaration_range.start
        {
            return Err(SourcePrimaryTermError::InvalidBindingEvent { event: group_start });
        }

        let mut group_end = group_start + 1;
        while group_end < bindings.len()
            && bindings[group_end].1.declaration_range == first.declaration_range
        {
            group_end += 1;
        }
        let final_index = group_end - 1;
        if group_end - group_start == 1 {
            if first.visible_after_ordinal != group_start {
                return Err(SourcePrimaryTermError::InvalidBindingEvent { event: group_start });
            }
        } else {
            for (index, (_, binding)) in bindings[group_start..group_end].iter().enumerate() {
                let event = group_start + index;
                if binding.spelling != first.spelling
                    || binding.kind != first.kind
                    || binding.owner_context != first.owner_context
                    || binding.identity != first.identity
                    || binding.visible_after_ordinal != final_index
                {
                    return Err(SourcePrimaryTermError::InvalidBindingEvent { event });
                }
            }
        }
        group_start = group_end;
    }

    let mut reference_ranges: Vec<SourceRange> = Vec::with_capacity(references.len());
    let mut previous_term = None;
    for (index, reference) in references.iter().enumerate() {
        let Some(term) = terms.get(reference.term.index()) else {
            return Err(SourcePrimaryTermError::InvalidReference {
                reference: SourcePrimaryTermReferenceId::new(index),
            });
        };
        if previous_term.is_some_and(|previous| previous >= reference.term.index())
            || index > 0 && reference_ranges[index - 1].end > term.source_range.start
        {
            return Err(SourcePrimaryTermError::InvalidReference {
                reference: SourcePrimaryTermReferenceId::new(index),
            });
        }
        previous_term = Some(reference.term.index());
        reference_ranges.push(term.source_range);
    }

    let mut binding_index = 0;
    let mut ordinals = Vec::with_capacity(reference_ranges.len());
    for reference_range in reference_ranges {
        while let Some((_, binding)) = bindings.get(binding_index) {
            if binding.declaration_range.end <= reference_range.start {
                binding_index += 1;
            } else {
                break;
            }
        }
        if bindings
            .get(binding_index)
            .is_some_and(|(_, binding)| binding.declaration_range.start < reference_range.end)
        {
            return Err(SourcePrimaryTermError::InvalidBindingEvent {
                event: binding_index,
            });
        }
        ordinals.push(binding_index);
    }
    Ok(ordinals)
}

fn validate_term_node(term: &SourcePrimaryTerm, arena: &TypedArena) -> Result<(), ()> {
    let TypedSiteRef::Node(node_id) = &term.site else {
        return Err(());
    };
    let Some(node) = arena.node(*node_id) else {
        return Err(());
    };
    if node.anchor != SourceAnchor::Range(term.source_range)
        || node.kind.as_str() != typed_kind_key(term.kind)
        || !recovery_matches(term.recovery, node.recovery)
    {
        return Err(());
    }
    Ok(())
}

fn valid_term_role(kind: SourcePrimaryTermKind, role: SourcePrimaryTermRole) -> bool {
    match kind {
        SourcePrimaryTermKind::It => role == SourcePrimaryTermRole::CurrentDefinitionResult,
        _ => role == SourcePrimaryTermRole::Value,
    }
}

fn valid_reference_role(kind: SourcePrimaryTermKind, role: SourcePrimaryTermReferenceRole) -> bool {
    matches!(
        (kind, role),
        (
            SourcePrimaryTermKind::VariableReference,
            SourcePrimaryTermReferenceRole::Variable
        ) | (
            SourcePrimaryTermKind::ConstantReference,
            SourcePrimaryTermReferenceRole::LocalConstant
        )
    )
}

fn valid_binding_role(kind: BindingKind, role: SourcePrimaryTermReferenceRole) -> bool {
    match role {
        SourcePrimaryTermReferenceRole::Variable => matches!(
            kind,
            BindingKind::ReservedVariable
                | BindingKind::LetBinding
                | BindingKind::QuantifierBinder
                | BindingKind::DefinitionParameter
        ),
        SourcePrimaryTermReferenceRole::LocalConstant => kind == BindingKind::LocalAbbreviation,
    }
}

fn valid_leaf_spelling(kind: SourcePrimaryTermKind, spelling: &str) -> bool {
    match kind {
        SourcePrimaryTermKind::VariableReference | SourcePrimaryTermKind::ConstantReference => {
            is_identifier(spelling)
        }
        SourcePrimaryTermKind::It => spelling == "it",
        SourcePrimaryTermKind::Numeral => {
            !spelling.is_empty() && spelling.bytes().all(|byte| byte.is_ascii_digit())
        }
        SourcePrimaryTermKind::Parenthesized => !spelling.is_empty(),
    }
}

fn recovery_matches(recovery: SourcePrimaryTermRecovery, node_recovery: NodeRecoveryState) -> bool {
    match recovery {
        SourcePrimaryTermRecovery::Normal => node_recovery == NodeRecoveryState::Normal,
        SourcePrimaryTermRecovery::Degraded => matches!(
            node_recovery,
            NodeRecoveryState::Recovered | NodeRecoveryState::Degraded
        ),
    }
}

fn valid_range(source_id: SourceId, range: SourceRange) -> bool {
    range.source_id == source_id && range.start < range.end
}

fn strictly_contains(parent: SourceRange, child: SourceRange) -> bool {
    parent.source_id == child.source_id && parent.start < child.start && parent.end > child.end
}

fn is_ancestor(
    expected: SourcePrimaryTermId,
    row: &SourcePrimaryTerm,
    rows: &[SourcePrimaryTerm],
) -> bool {
    let mut cursor = row.parent;
    while let Some(parent) = cursor {
        if parent == expected {
            return true;
        }
        cursor = rows.get(parent.index()).and_then(|term| term.parent);
    }
    false
}

fn typed_kind_key(kind: SourcePrimaryTermKind) -> &'static str {
    match kind {
        SourcePrimaryTermKind::VariableReference => "source.term.variable-reference",
        SourcePrimaryTermKind::ConstantReference => "source.term.constant-reference",
        SourcePrimaryTermKind::It => "source.term.it",
        SourcePrimaryTermKind::Numeral => "source.term.numeral",
        SourcePrimaryTermKind::Parenthesized => "source.term.parenthesized",
    }
}

fn kind_key(kind: SourcePrimaryTermKind) -> &'static str {
    match kind {
        SourcePrimaryTermKind::VariableReference => "variable-reference",
        SourcePrimaryTermKind::ConstantReference => "constant-reference",
        SourcePrimaryTermKind::It => "it",
        SourcePrimaryTermKind::Numeral => "numeral",
        SourcePrimaryTermKind::Parenthesized => "parenthesized",
    }
}

fn role_key(role: SourcePrimaryTermRole) -> &'static str {
    match role {
        SourcePrimaryTermRole::Value => "value",
        SourcePrimaryTermRole::CurrentDefinitionResult => "current-definition-result",
    }
}

fn reference_role_key(role: SourcePrimaryTermReferenceRole) -> &'static str {
    match role {
        SourcePrimaryTermReferenceRole::Variable => "variable",
        SourcePrimaryTermReferenceRole::LocalConstant => "local-constant",
    }
}

fn recovery_key(recovery: SourcePrimaryTermRecovery) -> &'static str {
    match recovery {
        SourcePrimaryTermRecovery::Normal => "normal",
        SourcePrimaryTermRecovery::Degraded => "degraded",
    }
}

fn write_optional_term_id(output: &mut String, term: Option<SourcePrimaryTermId>) {
    if let Some(term) = term {
        let _ = write!(output, "{}", term.index());
    } else {
        output.push('-');
    }
}

fn write_scope(output: &mut String, scope: Option<&LocalTermScope>) {
    let Some(scope) = scope else {
        output.push('-');
        return;
    };
    output.push('[');
    for (index, segment) in scope.path().iter().enumerate() {
        if index > 0 {
            output.push('.');
        }
        let _ = write!(output, "{segment}");
    }
    output.push(']');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binding_env::{
            BinderIdentity, BindingContextDraft, BindingContextId, BindingContextLayer,
            BindingContextOwner, BindingContextRecovery, BindingContextTable,
            BindingDiagnosticTable, BindingDraft, BindingEnvParts, BindingRecoveryState,
            BindingStatus, BindingTable, BindingTypeSite, CapturedFreeVariables,
        },
        typed_ast::{
            CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
            TypeFactTable, TypeRole, TypeTable, TypedAst, TypedAstError, TypedAstParts, TypedNode,
            TypedNodeId,
        },
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator as _,
    };

    #[derive(Clone)]
    struct Fixture {
        source: SourceId,
        module: ModuleId,
        input: SourcePrimaryTermHandoffInput,
        bindings: BindingEnv,
        arena: TypedArena,
    }

    fn source_id() -> SourceId {
        source_id_for("b2")
    }

    fn other_source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "b2".repeat(32)
        ))
        .expect("snapshot");
        let allocator = InMemorySessionIdAllocator::new();
        allocator.next_source_id(snapshot).expect("first source");
        allocator.next_source_id(snapshot).expect("second source")
    }

    fn source_id_for(byte: &str) -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            byte.repeat(32)
        ))
        .expect("snapshot");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source")
    }

    fn module(path: &str) -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new(path))
    }

    fn range(source: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id: source,
            start,
            end,
        }
    }

    fn node(index: usize) -> TypedSiteRef {
        TypedSiteRef::Node(TypedNodeId::new(index))
    }

    fn context(bindings: Vec<BindingId>, scope: Option<Vec<u32>>) -> BindingContextDraft {
        BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: scope.map(LocalTermScope::new),
            bindings: bindings.clone(),
            visible_bindings: bindings,
            recovery: BindingContextRecovery::Normal,
        }
    }

    fn reserved_binding(
        source: SourceId,
        spelling: &str,
        start: usize,
        visible_after_ordinal: usize,
    ) -> BindingDraft {
        let declaration_range = range(source, start, start + 1);
        BindingDraft {
            spelling: spelling.to_owned(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: spelling.to_owned(),
                declaration_range,
            },
            owner_context: BindingContextId::new(0),
            declaration_range,
            visible_after_ordinal,
            type_site: BindingTypeSite::Missing,
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        }
    }

    fn local_binding(
        source: SourceId,
        spelling: &str,
        start: usize,
        visible_after_ordinal: usize,
        kind: BindingKind,
    ) -> BindingDraft {
        let declaration_range = range(source, start, start + 1);
        BindingDraft {
            spelling: spelling.to_owned(),
            kind,
            identity: BinderIdentity::ResolverLocal {
                scope: LocalTermScope::new(vec![0]),
                ordinal: visible_after_ordinal,
                declaration_range,
            },
            owner_context: BindingContextId::new(0),
            declaration_range,
            visible_after_ordinal,
            type_site: BindingTypeSite::Missing,
            status: BindingStatus::Active,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        }
    }

    fn binding_env(
        source: SourceId,
        module: &ModuleId,
        drafts: Vec<BindingDraft>,
        scope: Option<Vec<u32>>,
    ) -> BindingEnv {
        let binding_ids = (0..drafts.len()).map(BindingId::new).collect::<Vec<_>>();
        let mut contexts = BindingContextTable::new();
        contexts.insert(context(binding_ids, scope));
        let mut bindings = BindingTable::new();
        for draft in drafts {
            bindings.insert(draft);
        }
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("binding environment")
    }

    fn binding_env_with_child_context(
        source: SourceId,
        module: &ModuleId,
        drafts: Vec<BindingDraft>,
    ) -> BindingEnv {
        let binding_ids = (0..drafts.len()).map(BindingId::new).collect::<Vec<_>>();
        let mut contexts = BindingContextTable::new();
        contexts.insert(context(binding_ids.clone(), Some(vec![0])));
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("child".to_owned()),
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Expression,
            lexical_scope: Some(LocalTermScope::new(vec![0, 0])),
            bindings: Vec::new(),
            visible_bindings: binding_ids,
            recovery: BindingContextRecovery::Normal,
        });
        let mut bindings = BindingTable::new();
        for draft in drafts {
            bindings.insert(draft);
        }
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("binding environment with child context")
    }

    fn binding_env_with_split_owners(
        source: SourceId,
        module: &ModuleId,
        mut drafts: Vec<BindingDraft>,
    ) -> BindingEnv {
        assert_eq!(drafts.len(), 2);
        drafts[0].owner_context = BindingContextId::new(0);
        drafts[1].owner_context = BindingContextId::new(1);
        let mut contexts = BindingContextTable::new();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: Some(LocalTermScope::new(vec![0])),
            bindings: vec![BindingId::new(0)],
            visible_bindings: vec![BindingId::new(0)],
            recovery: BindingContextRecovery::Normal,
        });
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Generated("child".to_owned()),
            parent: Some(BindingContextId::new(0)),
            layer: BindingContextLayer::Expression,
            lexical_scope: Some(LocalTermScope::new(vec![0, 0])),
            bindings: vec![BindingId::new(1)],
            visible_bindings: vec![BindingId::new(0), BindingId::new(1)],
            recovery: BindingContextRecovery::Normal,
        });
        let mut bindings = BindingTable::new();
        for draft in drafts {
            bindings.insert(draft);
        }
        BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("binding environment with split owners")
    }

    fn term(
        source: SourceId,
        index: usize,
        start: usize,
        end: usize,
        spelling: &str,
        kind: SourcePrimaryTermKind,
        parent: Option<usize>,
    ) -> SourcePrimaryTermInput {
        let role = if kind == SourcePrimaryTermKind::It {
            SourcePrimaryTermRole::CurrentDefinitionResult
        } else {
            SourcePrimaryTermRole::Value
        };
        SourcePrimaryTermInput {
            site: node(index),
            source_range: range(source, start, end),
            source_ordinal: index,
            context: BindingContextId::new(0),
            recovery: SourcePrimaryTermRecovery::Normal,
            spelling: spelling.to_owned(),
            kind,
            role,
            parent: parent.map(SourcePrimaryTermId::new),
        }
    }

    fn arena_nodes_for(terms: &[SourcePrimaryTermInput]) -> Vec<TypedNode> {
        let mut nodes = vec![None; terms.len()];
        for term in terms {
            nodes[term.site.node().index()] = Some(
                TypedNode::new(
                    typed_kind_key(term.kind),
                    SourceAnchor::Range(term.source_range),
                )
                .with_recovery(match term.recovery {
                    SourcePrimaryTermRecovery::Normal => NodeRecoveryState::Normal,
                    SourcePrimaryTermRecovery::Degraded => NodeRecoveryState::Degraded,
                }),
            );
        }
        nodes
            .into_iter()
            .map(|node| node.expect("dense fixture nodes"))
            .collect()
    }

    fn arena_for(terms: &[SourcePrimaryTermInput]) -> TypedArena {
        TypedArena::try_new(None, arena_nodes_for(terms)).expect("typed arena")
    }

    fn fixture() -> Fixture {
        let source = source_id();
        let module = module("source.term");
        let terms = vec![
            term(
                source,
                0,
                10,
                15,
                "( x )",
                SourcePrimaryTermKind::Parenthesized,
                None,
            ),
            term(
                source,
                1,
                11,
                12,
                "x",
                SourcePrimaryTermKind::VariableReference,
                Some(0),
            ),
            term(
                source,
                2,
                20,
                21,
                "x",
                SourcePrimaryTermKind::VariableReference,
                None,
            ),
            term(source, 3, 30, 31, "1", SourcePrimaryTermKind::Numeral, None),
            term(source, 4, 40, 42, "it", SourcePrimaryTermKind::It, None),
            term(
                source,
                5,
                50,
                51,
                "c",
                SourcePrimaryTermKind::ConstantReference,
                None,
            ),
        ];
        let input = SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module.clone(),
            terms,
            references: vec![
                SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(1),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                },
                SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(2),
                    binding: BindingId::new(0),
                    role: SourcePrimaryTermReferenceRole::Variable,
                },
                SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(5),
                    binding: BindingId::new(1),
                    role: SourcePrimaryTermReferenceRole::LocalConstant,
                },
            ],
            numeric_type_requests: vec![SourceNumericTypeRequestInput {
                term: SourcePrimaryTermId::new(3),
                owner: node(3),
                source_range: range(source, 30, 31),
                spelling: "1".to_owned(),
                request_ordinal: 0,
            }],
        };
        let bindings = binding_env(
            source,
            &module,
            vec![
                reserved_binding(source, "x", 0, 0),
                local_binding(source, "c", 2, 1, BindingKind::LocalAbbreviation),
            ],
            Some(vec![0]),
        );
        let arena = arena_for(&input.terms);
        Fixture {
            source,
            module,
            input,
            bindings,
            arena,
        }
    }

    fn build(fixture: &Fixture) -> Result<SourcePrimaryTermHandoff, SourcePrimaryTermError> {
        SourcePrimaryTermProducer::build(fixture.input.clone(), &fixture.bindings, &fixture.arena)
    }

    fn replace_bindings(fixture: &mut Fixture, drafts: Vec<BindingDraft>, scope: Option<Vec<u32>>) {
        fixture.bindings = binding_env(fixture.source, &fixture.module, drafts, scope);
    }

    fn empty_typed_parts(fixture: &Fixture) -> TypedAstParts {
        TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: None,
            source_attribute: None,
            nodes: fixture.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: TypeFactTable::new(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        }
    }

    #[test]
    fn dense_rows_accessors_and_debug_cover_every_task252_kind() {
        let fixture = fixture();
        let handoff = build(&fixture).expect("valid handoff");
        assert_eq!(handoff.source_id(), fixture.source);
        assert_eq!(handoff.module_id(), &fixture.module);
        assert_eq!(handoff.terms().len(), 6);
        assert_eq!(handoff.references().len(), 3);
        assert_eq!(handoff.numeric_type_requests().len(), 1);
        assert!(!handoff.terms().is_empty());

        let kinds = handoff
            .terms()
            .iter()
            .map(|(id, row)| {
                assert_eq!(id.index(), row.source_ordinal());
                assert_eq!(row.context(), BindingContextId::new(0));
                assert_eq!(row.recovery(), SourcePrimaryTermRecovery::Normal);
                assert_eq!(row.site(), &node(id.index()));
                row.kind()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            [
                SourcePrimaryTermKind::Parenthesized,
                SourcePrimaryTermKind::VariableReference,
                SourcePrimaryTermKind::VariableReference,
                SourcePrimaryTermKind::Numeral,
                SourcePrimaryTermKind::It,
                SourcePrimaryTermKind::ConstantReference,
            ]
        );
        assert_eq!(
            handoff
                .terms()
                .get(SourcePrimaryTermId::new(1))
                .expect("child")
                .parent(),
            Some(SourcePrimaryTermId::new(0))
        );
        assert_eq!(
            handoff
                .terms()
                .get(SourcePrimaryTermId::new(5))
                .expect("constant")
                .spelling(),
            "c"
        );
        for (id, reference) in handoff.references().iter() {
            assert_eq!(
                id.index(),
                [
                    SourcePrimaryTermId::new(1),
                    SourcePrimaryTermId::new(2),
                    SourcePrimaryTermId::new(5)
                ]
                .iter()
                .position(|term| *term == reference.term())
                .expect("known reference term")
            );
            assert_eq!(reference.use_ordinal(), 2);
            assert_eq!(
                reference.lexical_scope().map(LocalTermScope::path),
                Some(&[0][..])
            );
        }
        let request = handoff
            .numeric_type_requests()
            .get(SourceNumericTypeRequestId::new(0))
            .expect("numeric request");
        assert_eq!(request.term(), SourcePrimaryTermId::new(3));
        assert_eq!(request.owner(), &node(3));
        assert_eq!(request.source_range(), range(fixture.source, 30, 31));
        assert_eq!(request.spelling(), "1");
        assert_eq!(request.request_ordinal(), 0);

        let debug = handoff.debug_text();
        assert_eq!(debug, build(&fixture).expect("replay").debug_text());
        assert!(debug.starts_with("source-primary-term-debug-v1\n"));
        assert!(debug.contains("kind=parenthesized"));
        assert!(debug.contains("role=current-definition-result"));
        assert!(debug.contains("role=local-constant"));
        assert!(debug.contains("numeric-request#0 term=3 ordinal=0"));
    }

    #[test]
    fn environment_site_range_kind_recovery_context_and_spelling_corruption_fail_closed() {
        let fixture = fixture();

        let mut cases = Vec::new();
        let mut changed = fixture.clone();
        changed.input.source_id = other_source_id();
        cases.push(changed);
        let mut changed = fixture.clone();
        changed.input.module_id = module("wrong");
        cases.push(changed);
        let mut changed = fixture.clone();
        changed.input.terms[2].source_ordinal = 9;
        cases.push(changed);
        let mut changed = fixture.clone();
        changed.input.terms[2].source_range.end = changed.input.terms[2].source_range.start;
        cases.push(changed);
        let mut changed = fixture.clone();
        changed.input.terms[2].site = TypedSiteRef::Role {
            node: TypedNodeId::new(2),
            role: TypeRole::new("term"),
        };
        cases.push(changed);
        let mut changed = fixture.clone();
        changed.input.terms[2].context = BindingContextId::new(9);
        cases.push(changed);
        let mut changed = fixture.clone();
        changed.input.terms[4].spelling = "IT".to_owned();
        cases.push(changed);
        let mut changed = fixture.clone();
        changed.input.terms[3].spelling = "1a".to_owned();
        cases.push(changed);
        let mut changed = fixture.clone();
        changed.input.terms[4].role = SourcePrimaryTermRole::Value;
        cases.push(changed);

        for changed in cases {
            assert!(build(&changed).is_err());
        }

        let mut wrong_kind = fixture.clone();
        let mut nodes = arena_nodes_for(&wrong_kind.input.terms);
        nodes[2] = TypedNode::new(
            "source.term.constant-reference",
            SourceAnchor::Range(range(fixture.source, 20, 21)),
        );
        wrong_kind.arena = TypedArena::try_new(None, nodes).expect("wrong-kind arena");
        assert!(build(&wrong_kind).is_err());

        let mut wrong_recovery = fixture.clone();
        wrong_recovery.input.terms[2].recovery = SourcePrimaryTermRecovery::Degraded;
        assert!(build(&wrong_recovery).is_err());

        for malformed in ["1", "x y", "'x", "x-", "it", "theorem"] {
            let mut malformed_reference = fixture.clone();
            malformed_reference.input.terms[1].spelling = malformed.to_owned();
            malformed_reference.input.terms[2].spelling = malformed.to_owned();
            replace_bindings(
                &mut malformed_reference,
                vec![
                    reserved_binding(fixture.source, malformed, 0, 0),
                    local_binding(fixture.source, "c", 2, 1, BindingKind::LocalAbbreviation),
                ],
                Some(vec![0]),
            );
            assert!(matches!(
                build(&malformed_reference),
                Err(SourcePrimaryTermError::InvalidTerm { term })
                    if term == SourcePrimaryTermId::new(1)
            ));
        }

        let mut degraded = fixture.clone();
        degraded.input.terms[2].recovery = SourcePrimaryTermRecovery::Degraded;
        let mut nodes = arena_nodes_for(&degraded.input.terms);
        nodes[2].recovery = NodeRecoveryState::Recovered;
        degraded.arena = TypedArena::try_new(None, nodes).expect("recovered arena");
        assert!(build(&degraded).is_ok());
    }

    #[test]
    fn reference_cardinality_roles_binding_kinds_and_winners_are_authenticated() {
        let fixture = fixture();
        let mut missing = fixture.clone();
        missing.input.references.remove(0);
        assert!(build(&missing).is_err());

        let mut duplicate = fixture.clone();
        duplicate
            .input
            .references
            .insert(1, duplicate.input.references[0].clone());
        assert!(build(&duplicate).is_err());

        let mut wrong_role = fixture.clone();
        wrong_role.input.references[0].role = SourcePrimaryTermReferenceRole::LocalConstant;
        assert!(build(&wrong_role).is_err());

        let mut wrong_binding = fixture.clone();
        wrong_binding.input.references[0].binding = BindingId::new(1);
        assert!(build(&wrong_binding).is_err());

        let mut wrong_kind = fixture.clone();
        replace_bindings(
            &mut wrong_kind,
            vec![
                local_binding(fixture.source, "x", 0, 0, BindingKind::LocalAbbreviation),
                local_binding(fixture.source, "c", 2, 1, BindingKind::LocalAbbreviation),
            ],
            Some(vec![0]),
        );
        assert!(build(&wrong_kind).is_err());

        let mut different_winner = fixture.clone();
        replace_bindings(
            &mut different_winner,
            vec![
                reserved_binding(fixture.source, "x", 0, 0),
                reserved_binding(fixture.source, "x", 2, 1),
            ],
            Some(vec![0]),
        );
        different_winner.input.references.truncate(2);
        different_winner.input.terms.truncate(5);
        different_winner.input.numeric_type_requests.truncate(1);
        different_winner.arena = arena_for(&different_winner.input.terms);
        assert!(build(&different_winner).is_err());
    }

    #[test]
    fn numeric_request_cardinality_association_and_order_are_atomic() {
        let fixture = fixture();
        let mut missing = fixture.clone();
        missing.input.numeric_type_requests.clear();
        assert!(build(&missing).is_err());

        let mut duplicate = fixture.clone();
        duplicate
            .input
            .numeric_type_requests
            .push(duplicate.input.numeric_type_requests[0].clone());
        assert!(build(&duplicate).is_err());

        let mut wrong_term = fixture.clone();
        wrong_term.input.numeric_type_requests[0].term = SourcePrimaryTermId::new(2);
        assert!(build(&wrong_term).is_err());

        let mut wrong_owner = fixture.clone();
        wrong_owner.input.numeric_type_requests[0].owner = node(2);
        assert!(build(&wrong_owner).is_err());

        let mut wrong_range = fixture.clone();
        wrong_range.input.numeric_type_requests[0].source_range.end += 1;
        assert!(build(&wrong_range).is_err());

        let mut wrong_spelling = fixture.clone();
        wrong_spelling.input.numeric_type_requests[0].spelling = "01".to_owned();
        assert!(build(&wrong_spelling).is_err());

        let mut wrong_ordinal = fixture.clone();
        wrong_ordinal.input.numeric_type_requests[0].request_ordinal = 1;
        assert!(build(&wrong_ordinal).is_err());

        let source = source_id_for("d4");
        let module = module("source.term.numeric-order");
        let terms = vec![
            term(source, 0, 10, 11, "1", SourcePrimaryTermKind::Numeral, None),
            term(source, 1, 20, 21, "2", SourcePrimaryTermKind::Numeral, None),
        ];
        let mut ordered = Fixture {
            source,
            module: module.clone(),
            input: SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: terms.clone(),
                references: Vec::new(),
                numeric_type_requests: vec![
                    SourceNumericTypeRequestInput {
                        term: SourcePrimaryTermId::new(0),
                        owner: node(0),
                        source_range: range(source, 10, 11),
                        spelling: "1".to_owned(),
                        request_ordinal: 0,
                    },
                    SourceNumericTypeRequestInput {
                        term: SourcePrimaryTermId::new(1),
                        owner: node(1),
                        source_range: range(source, 20, 21),
                        spelling: "2".to_owned(),
                        request_ordinal: 1,
                    },
                ],
            },
            bindings: binding_env(source, &module, Vec::new(), None),
            arena: arena_for(&terms),
        };
        let handoff = build(&ordered).expect("two numeric requests in source order");
        assert_eq!(
            handoff
                .numeric_type_requests()
                .iter()
                .map(|(_, request)| (request.term().index(), request.request_ordinal()))
                .collect::<Vec<_>>(),
            [(0, 0), (1, 1)]
        );
        ordered.input.numeric_type_requests.swap(0, 1);
        for (request_ordinal, request) in ordered.input.numeric_type_requests.iter_mut().enumerate()
        {
            request.request_ordinal = request_ordinal;
        }
        assert!(matches!(
            build(&ordered),
            Err(SourcePrimaryTermError::InvalidNumericTypeRequest { request })
                if request == SourceNumericTypeRequestId::new(1)
        ));
    }

    #[test]
    fn parent_graph_preorder_containment_context_and_ownership_are_enforced() {
        let fixture = fixture();
        let mut dangling = fixture.clone();
        dangling.input.terms[1].parent = Some(SourcePrimaryTermId::new(9));
        assert!(build(&dangling).is_err());

        let mut forward = fixture.clone();
        forward.input.terms[0].parent = Some(SourcePrimaryTermId::new(1));
        assert!(build(&forward).is_err());

        let mut non_parent = fixture.clone();
        non_parent.input.terms[2].parent = Some(SourcePrimaryTermId::new(1));
        assert!(build(&non_parent).is_err());

        let mut second_child = fixture.clone();
        second_child.input.terms[2].parent = Some(SourcePrimaryTermId::new(0));
        second_child.input.terms[2].source_range = range(fixture.source, 12, 13);
        let mut nodes = arena_nodes_for(&second_child.input.terms);
        nodes[2].anchor = SourceAnchor::Range(range(fixture.source, 12, 13));
        second_child.arena = TypedArena::try_new(None, nodes).expect("second-child arena");
        assert!(build(&second_child).is_err());

        let mut containment = fixture.clone();
        containment.input.terms[1].source_range = range(fixture.source, 15, 16);
        let mut nodes = arena_nodes_for(&containment.input.terms);
        nodes[1].anchor = SourceAnchor::Range(range(fixture.source, 15, 16));
        containment.arena = TypedArena::try_new(None, nodes).expect("containment arena");
        assert!(build(&containment).is_err());

        let mut spelling = fixture.clone();
        spelling.input.terms[0].spelling = "(  x  )".to_owned();
        assert!(build(&spelling).is_err());

        let mut missing_child = fixture.clone();
        missing_child.input.terms[1].parent = None;
        assert!(build(&missing_child).is_err());

        let mut crossed_context = fixture.clone();
        crossed_context.bindings = binding_env_with_child_context(
            fixture.source,
            &fixture.module,
            vec![
                reserved_binding(fixture.source, "x", 0, 0),
                local_binding(fixture.source, "c", 2, 1, BindingKind::LocalAbbreviation),
            ],
        );
        crossed_context.input.terms[1].context = BindingContextId::new(1);
        assert!(matches!(
            build(&crossed_context),
            Err(SourcePrimaryTermError::InvalidTerm { term })
                if term == SourcePrimaryTermId::new(1)
        ));
    }

    #[test]
    fn roots_siblings_references_and_overlaps_must_stay_in_source_order() {
        let fixture = fixture();
        let mut reordered_root = fixture.clone();
        reordered_root.input.terms[2].source_range = range(fixture.source, 5, 6);
        let mut nodes = arena_nodes_for(&reordered_root.input.terms);
        nodes[2].anchor = SourceAnchor::Range(range(fixture.source, 5, 6));
        reordered_root.arena = TypedArena::try_new(None, nodes).expect("reordered arena");
        assert!(build(&reordered_root).is_err());

        let mut overlapping_roots = fixture.clone();
        overlapping_roots.input.terms[2].source_range = range(fixture.source, 14, 16);
        let mut nodes = arena_nodes_for(&overlapping_roots.input.terms);
        nodes[2].anchor = SourceAnchor::Range(range(fixture.source, 14, 16));
        overlapping_roots.arena = TypedArena::try_new(None, nodes).expect("overlap arena");
        assert!(build(&overlapping_roots).is_err());

        let mut reordered_references = fixture.clone();
        reordered_references.input.references.swap(0, 1);
        assert!(build(&reordered_references).is_err());
    }

    #[test]
    fn binding_ordinals_count_only_preceding_declarations_across_interleaved_uses() {
        let source = source_id();
        let module = module("source.term.interleaved");
        let terms = vec![
            term(
                source,
                0,
                10,
                11,
                "x",
                SourcePrimaryTermKind::VariableReference,
                None,
            ),
            term(
                source,
                1,
                20,
                21,
                "x",
                SourcePrimaryTermKind::VariableReference,
                None,
            ),
            term(
                source,
                2,
                40,
                41,
                "y",
                SourcePrimaryTermKind::VariableReference,
                None,
            ),
        ];
        let fixture = Fixture {
            source,
            module: module.clone(),
            input: SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: terms.clone(),
                references: vec![
                    SourcePrimaryTermReferenceInput {
                        term: SourcePrimaryTermId::new(0),
                        binding: BindingId::new(0),
                        role: SourcePrimaryTermReferenceRole::Variable,
                    },
                    SourcePrimaryTermReferenceInput {
                        term: SourcePrimaryTermId::new(1),
                        binding: BindingId::new(0),
                        role: SourcePrimaryTermReferenceRole::Variable,
                    },
                    SourcePrimaryTermReferenceInput {
                        term: SourcePrimaryTermId::new(2),
                        binding: BindingId::new(1),
                        role: SourcePrimaryTermReferenceRole::Variable,
                    },
                ],
                numeric_type_requests: Vec::new(),
            },
            bindings: binding_env(
                source,
                &module,
                vec![
                    reserved_binding(source, "x", 0, 0),
                    reserved_binding(source, "y", 30, 1),
                ],
                None,
            ),
            arena: arena_for(&terms),
        };
        let handoff = build(&fixture).expect("interleaved binding/use transaction");
        assert_eq!(
            handoff
                .references()
                .iter()
                .map(|(_, row)| row.use_ordinal())
                .collect::<Vec<_>>(),
            [1, 1, 2]
        );

        let mut forward = fixture.clone();
        forward.input.terms[1].spelling = "y".to_owned();
        forward.input.references[1].binding = BindingId::new(1);
        assert!(build(&forward).is_err());

        let mut stale_singleton = fixture.clone();
        replace_bindings(
            &mut stale_singleton,
            vec![
                reserved_binding(source, "x", 0, 1),
                reserved_binding(source, "y", 30, 1),
            ],
            None,
        );
        assert!(matches!(
            build(&stale_singleton),
            Err(SourcePrimaryTermError::InvalidBindingEvent { event: 0 })
        ));

        let mut reordered_bindings = fixture.clone();
        replace_bindings(
            &mut reordered_bindings,
            vec![
                reserved_binding(source, "y", 30, 0),
                reserved_binding(source, "x", 0, 1),
            ],
            None,
        );
        assert!(matches!(
            build(&reordered_bindings),
            Err(SourcePrimaryTermError::InvalidBindingEvent { event: 1 })
        ));

        let mut overlapping_bindings = fixture.clone();
        let mut first = reserved_binding(source, "x", 0, 0);
        first.declaration_range = range(source, 0, 31);
        first.identity = BinderIdentity::ReservedVariable {
            spelling: "x".to_owned(),
            declaration_range: first.declaration_range,
        };
        replace_bindings(
            &mut overlapping_bindings,
            vec![first, reserved_binding(source, "y", 30, 1)],
            None,
        );
        assert!(matches!(
            build(&overlapping_bindings),
            Err(SourcePrimaryTermError::InvalidBindingEvent { event: 1 })
        ));
        assert!(!valid_range(source, range(other_source_id(), 0, 1)));
        assert!(!valid_range(source, range(source, 1, 1)));
    }

    #[test]
    fn duplicate_priority_groups_reach_ambiguous_and_group_drift_is_rejected() {
        let mut fixture = fixture();
        let mut first = reserved_binding(fixture.source, "x", 0, 1);
        let mut second = first.clone();
        first.visible_after_ordinal = 1;
        second.visible_after_ordinal = 1;
        replace_bindings(&mut fixture, vec![first.clone(), second.clone()], None);
        fixture.input.terms = vec![term(
            fixture.source,
            0,
            10,
            11,
            "x",
            SourcePrimaryTermKind::VariableReference,
            None,
        )];
        fixture.input.references = vec![SourcePrimaryTermReferenceInput {
            term: SourcePrimaryTermId::new(0),
            binding: BindingId::new(0),
            role: SourcePrimaryTermReferenceRole::Variable,
        }];
        fixture.input.numeric_type_requests.clear();
        fixture.arena = arena_for(&fixture.input.terms);
        assert!(matches!(
            build(&fixture),
            Err(SourcePrimaryTermError::InvalidReference { .. })
        ));

        let mut stale_ordinal = fixture.clone();
        let first = reserved_binding(fixture.source, "x", 0, 0);
        let second = first.clone();
        replace_bindings(&mut stale_ordinal, vec![first, second], None);
        assert!(matches!(
            build(&stale_ordinal),
            Err(SourcePrimaryTermError::InvalidBindingEvent { .. })
        ));

        let mut different_spelling = fixture.clone();
        let first = reserved_binding(fixture.source, "x", 0, 1);
        let second = reserved_binding(fixture.source, "y", 0, 1);
        replace_bindings(&mut different_spelling, vec![first, second], None);
        assert!(matches!(
            build(&different_spelling),
            Err(SourcePrimaryTermError::InvalidBindingEvent { .. })
        ));

        let mut different_kind = fixture.clone();
        let first = local_binding(fixture.source, "x", 0, 1, BindingKind::LetBinding);
        let mut second = first.clone();
        second.kind = BindingKind::QuantifierBinder;
        replace_bindings(&mut different_kind, vec![first, second], Some(vec![0]));
        assert!(matches!(
            build(&different_kind),
            Err(SourcePrimaryTermError::InvalidBindingEvent { event: 1 })
        ));

        let mut different_identity = fixture.clone();
        let first = local_binding(fixture.source, "x", 0, 1, BindingKind::LetBinding);
        let mut second = first.clone();
        second.identity = BinderIdentity::ResolverLocal {
            scope: LocalTermScope::new(vec![1]),
            ordinal: 1,
            declaration_range: second.declaration_range,
        };
        replace_bindings(&mut different_identity, vec![first, second], Some(vec![0]));
        assert!(matches!(
            build(&different_identity),
            Err(SourcePrimaryTermError::InvalidBindingEvent { event: 1 })
        ));

        let mut different_owner = fixture.clone();
        let mut first = local_binding(fixture.source, "x", 0, 1, BindingKind::LetBinding);
        let mut second = first.clone();
        first.identity = BinderIdentity::ResolverLocal {
            scope: LocalTermScope::new(vec![0, 0]),
            ordinal: 1,
            declaration_range: first.declaration_range,
        };
        second.identity = first.identity.clone();
        different_owner.bindings =
            binding_env_with_split_owners(fixture.source, &fixture.module, vec![first, second]);
        different_owner.input.terms = vec![term(
            fixture.source,
            0,
            10,
            11,
            "x",
            SourcePrimaryTermKind::VariableReference,
            None,
        )];
        different_owner.input.terms[0].context = BindingContextId::new(1);
        different_owner.input.references = vec![SourcePrimaryTermReferenceInput {
            term: SourcePrimaryTermId::new(0),
            binding: BindingId::new(0),
            role: SourcePrimaryTermReferenceRole::Variable,
        }];
        different_owner.input.numeric_type_requests.clear();
        different_owner.arena = arena_for(&different_owner.input.terms);
        assert!(matches!(
            build(&different_owner),
            Err(SourcePrimaryTermError::InvalidBindingEvent { event: 1 })
        ));
    }

    #[test]
    fn every_frozen_variable_binding_kind_is_accepted_with_authenticated_scope() {
        let source = source_id_for("e5");
        let module = module("source.term.variable-kinds");
        for kind in [
            BindingKind::LetBinding,
            BindingKind::QuantifierBinder,
            BindingKind::DefinitionParameter,
        ] {
            let terms = vec![term(
                source,
                0,
                10,
                11,
                "x",
                SourcePrimaryTermKind::VariableReference,
                None,
            )];
            let fixture = Fixture {
                source,
                module: module.clone(),
                input: SourcePrimaryTermHandoffInput {
                    source_id: source,
                    module_id: module.clone(),
                    terms: terms.clone(),
                    references: vec![SourcePrimaryTermReferenceInput {
                        term: SourcePrimaryTermId::new(0),
                        binding: BindingId::new(0),
                        role: SourcePrimaryTermReferenceRole::Variable,
                    }],
                    numeric_type_requests: Vec::new(),
                },
                bindings: binding_env(
                    source,
                    &module,
                    vec![local_binding(source, "x", 0, 0, kind)],
                    Some(vec![0]),
                ),
                arena: arena_for(&terms),
            };
            let handoff = build(&fixture)
                .unwrap_or_else(|error| panic!("{kind:?} should be accepted: {error}"));
            let reference = handoff
                .references()
                .get(SourcePrimaryTermReferenceId::new(0))
                .expect("variable reference");
            assert_eq!(reference.role(), SourcePrimaryTermReferenceRole::Variable);
            assert_eq!(reference.binding(), BindingId::new(0));
            assert_eq!(reference.use_ordinal(), 1);
            assert_eq!(
                reference.lexical_scope().map(LocalTermScope::path),
                Some(&[0][..])
            );
        }
    }

    #[test]
    fn forward_missing_scope_unresolved_and_different_local_winners_are_rejected() {
        let mut forward = fixture();
        forward.input.terms = vec![term(
            forward.source,
            0,
            1,
            2,
            "x",
            SourcePrimaryTermKind::VariableReference,
            None,
        )];
        forward.input.references = vec![SourcePrimaryTermReferenceInput {
            term: SourcePrimaryTermId::new(0),
            binding: BindingId::new(0),
            role: SourcePrimaryTermReferenceRole::Variable,
        }];
        forward.input.numeric_type_requests.clear();
        let forward_source = forward.source;
        replace_bindings(
            &mut forward,
            vec![reserved_binding(forward_source, "x", 2, 0)],
            None,
        );
        forward.arena = arena_for(&forward.input.terms);
        assert!(build(&forward).is_err());

        let mut missing_scope = fixture();
        let missing_scope_source = missing_scope.source;
        replace_bindings(
            &mut missing_scope,
            vec![
                local_binding(
                    missing_scope_source,
                    "x",
                    0,
                    0,
                    BindingKind::QuantifierBinder,
                ),
                local_binding(
                    missing_scope_source,
                    "c",
                    2,
                    1,
                    BindingKind::LocalAbbreviation,
                ),
            ],
            None,
        );
        assert!(build(&missing_scope).is_err());

        let mut unresolved = fixture();
        unresolved.input.terms[1].spelling = "missing".to_owned();
        assert!(build(&unresolved).is_err());
    }

    #[test]
    fn eligible_deep_parentheses_form_a_dense_iterative_preorder_tree() {
        let source = source_id_for("c3");
        let module = module("source.term.deep");
        let depth = 64;
        let mut terms = Vec::with_capacity(depth + 1);
        let mut spelling = "7".to_owned();
        let mut spellings = vec![String::new(); depth];
        for index in (0..depth).rev() {
            spelling = format!("( {spelling} )");
            spellings[index] = spelling.clone();
        }
        for (index, spelling) in spellings.into_iter().enumerate() {
            terms.push(term(
                source,
                index,
                index,
                depth * 2 + 1 - index,
                &spelling,
                SourcePrimaryTermKind::Parenthesized,
                if index > 0 { Some(index - 1) } else { None },
            ));
        }
        terms.push(term(
            source,
            depth,
            depth,
            depth + 1,
            "7",
            SourcePrimaryTermKind::Numeral,
            Some(depth - 1),
        ));
        let fixture = Fixture {
            source,
            module: module.clone(),
            input: SourcePrimaryTermHandoffInput {
                source_id: source,
                module_id: module.clone(),
                terms: terms.clone(),
                references: Vec::new(),
                numeric_type_requests: vec![SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(depth),
                    owner: node(depth),
                    source_range: range(source, depth, depth + 1),
                    spelling: "7".to_owned(),
                    request_ordinal: 0,
                }],
            },
            bindings: binding_env(source, &module, Vec::new(), None),
            arena: arena_for(&terms),
        };
        let handoff = build(&fixture).expect("deep eligible tree");
        assert_eq!(handoff.terms().len(), depth + 1);
        assert_eq!(
            handoff
                .terms()
                .get(SourcePrimaryTermId::new(depth))
                .expect("leaf")
                .parent(),
            Some(SourcePrimaryTermId::new(depth - 1))
        );
    }

    #[test]
    fn typed_ast_installation_revalidates_arena_and_rejects_replacement() {
        let fixture = fixture();
        let handoff = build(&fixture).expect("handoff");
        let typed = TypedAst::try_new(empty_typed_parts(&fixture))
            .expect("typed AST")
            .with_source_term(handoff.clone())
            .expect("source term installation");
        assert_eq!(typed.source_term(), Some(&handoff));
        assert!(
            typed
                .debug_text()
                .contains("source-primary-term-debug-v1\n")
        );
        assert!(matches!(
            typed.clone().with_source_term(handoff.clone()),
            Err(TypedAstError::InvalidSourceTerm)
        ));

        let mut wrong_arena = fixture.clone();
        let mut nodes = arena_nodes_for(&wrong_arena.input.terms);
        nodes[0].anchor = SourceAnchor::Range(range(fixture.source, 9, 15));
        wrong_arena.arena = TypedArena::try_new(None, nodes).expect("wrong arena");
        let wrong_parts = empty_typed_parts(&wrong_arena);
        assert!(matches!(
            TypedAst::try_new(wrong_parts)
                .expect("legacy typed AST")
                .with_source_term(handoff),
            Err(TypedAstError::InvalidSourceTerm)
        ));
    }

    #[test]
    fn production_boundary_stays_syntax_free_and_has_no_semantic_result_payloads() {
        let source = include_str!("source_term.rs");
        let production = &source[..source.find("#[cfg(test)]").expect("test module")];
        for forbidden in [
            "mizar_syntax",
            "SurfaceAst",
            "SurfaceNodeId",
            "SyntaxKind",
            "NormalizedType",
            "TypeFact",
            "CheckedFormula",
            "Fol",
            "Axiom",
            "numeric_type_result",
        ] {
            assert!(
                !production.contains(forbidden),
                "production source term transport must not contain {forbidden}"
            );
        }
    }
}
