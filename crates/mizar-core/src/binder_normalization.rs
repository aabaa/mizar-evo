//! Binder normalization, alpha-equivalence, and capture-avoiding substitution.
//!
//! Implements the task-5 and task-6 slices of
//! [binder_normalization.md](../../../../doc/design/mizar-core/en/binder_normalization.md).

use crate::core_ir::{
    CoreBinder, CoreDiagnosticId, CoreDiagnosticMessageKey, CoreFormulaId, CoreFormulaKind, CoreIr,
    CoreItemId, CoreNodeRef, CoreSourceRef, CoreTermId, CoreTermKind, CoreTypePredicate, CoreVarId,
    CoreVarRole, GeneratedOriginKey, GeneratedOriginKind,
};
use mizar_resolve::resolved_ast::SymbolId;
use mizar_session::SourceId;
use std::collections::{BTreeMap, BTreeSet};

pub type BinderResult<T> = Result<T, BinderDiagnostic>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundVar {
    pub index_from_innermost: u32,
}

impl BoundVar {
    pub const fn new(index_from_innermost: u32) -> Self {
        Self {
            index_from_innermost,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NormalizedVar {
    Bound(BoundVar),
    Free(CoreVarId),
    Schematic(CoreVarId),
    Generated(CoreVarId),
}

impl NormalizedVar {
    fn free_id(&self) -> Option<CoreVarId> {
        match self {
            Self::Bound(_) => None,
            Self::Free(id) | Self::Schematic(id) | Self::Generated(id) => Some(*id),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NormalizedVarClass {
    Free,
    Schematic,
    Generated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NormalizedVarSort {
    Term,
    Formula,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinderFrame {
    pub canonical_index: u32,
    pub original_var: CoreVarId,
    pub role: CoreVarRole,
    pub source_name: Option<String>,
    pub source: CoreSourceRef,
}

impl BinderFrame {
    pub fn new(
        canonical_index: u32,
        original_var: CoreVarId,
        role: impl Into<CoreVarRole>,
        source: CoreSourceRef,
    ) -> Self {
        Self {
            canonical_index,
            original_var,
            role: role.into(),
            source_name: None,
            source,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedBinderEntry {
    pub frame: BinderFrame,
    pub ty_guard: Option<Box<NormalizedFormula>>,
}

impl NormalizedBinderEntry {
    pub fn new(frame: BinderFrame) -> Self {
        Self {
            frame,
            ty_guard: None,
        }
    }

    pub fn with_guard(frame: BinderFrame, ty_guard: NormalizedFormula) -> Self {
        Self {
            frame,
            ty_guard: Some(Box::new(ty_guard)),
        }
    }
}

impl From<BinderFrame> for NormalizedBinderEntry {
    fn from(frame: BinderFrame) -> Self {
        Self::new(frame)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BinderContext {
    pub frames: Vec<BinderFrame>,
    pub free_variables: BTreeSet<CoreVarId>,
    pub variable_classes: BTreeMap<CoreVarId, NormalizedVarClass>,
    pub variable_roles: BTreeMap<CoreVarId, CoreVarRole>,
    pub variable_sorts: BTreeMap<CoreVarId, NormalizedVarSort>,
}

impl BinderContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn declare_variable(
        &mut self,
        var: CoreVarId,
        class: NormalizedVarClass,
        role: impl Into<CoreVarRole>,
        sort: NormalizedVarSort,
    ) {
        self.free_variables.insert(var);
        self.variable_classes.insert(var, class);
        self.variable_roles.insert(var, role.into());
        self.variable_sorts.insert(var, sort);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedTerm {
    pub kind: NormalizedTermKind,
    pub free_variables: BTreeSet<CoreVarId>,
}

impl NormalizedTerm {
    pub fn new(kind: NormalizedTermKind) -> Self {
        let free_variables = term_free_variables(&kind);
        Self {
            kind,
            free_variables,
        }
    }

    pub fn var(var: NormalizedVar) -> Self {
        Self::new(NormalizedTermKind::Var(var))
    }

    pub fn free(var: CoreVarId) -> Self {
        Self::var(NormalizedVar::Free(var))
    }

    pub fn bound(index_from_innermost: u32) -> Self {
        Self::var(NormalizedVar::Bound(BoundVar::new(index_from_innermost)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NormalizedTermKind {
    Var(NormalizedVar),
    Const(SymbolId),
    Apply {
        functor: SymbolId,
        args: Vec<NormalizedTerm>,
    },
    Select {
        selector: SymbolId,
        base: Box<NormalizedTerm>,
    },
    Tuple(Vec<NormalizedTerm>),
    SetEnum(Vec<NormalizedTerm>),
    Generated {
        origin: GeneratedOriginRecord,
        args: Vec<NormalizedTerm>,
    },
    Error(CoreDiagnosticId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedFormula {
    pub kind: NormalizedFormulaKind,
    pub free_variables: BTreeSet<CoreVarId>,
}

impl NormalizedFormula {
    pub fn new(kind: NormalizedFormulaKind) -> Self {
        let free_variables = formula_free_variables(&kind);
        Self {
            kind,
            free_variables,
        }
    }

    pub fn var(var: NormalizedVar) -> Self {
        Self::new(NormalizedFormulaKind::Var(var))
    }

    pub fn forall(binders: Vec<BinderFrame>, body: NormalizedFormula) -> Self {
        Self::forall_entries(binders.into_iter().map(Into::into).collect(), body)
    }

    pub fn forall_entries(binders: Vec<NormalizedBinderEntry>, body: NormalizedFormula) -> Self {
        Self::new(NormalizedFormulaKind::Forall {
            binders,
            body: Box::new(body),
        })
    }

    pub fn exists(binders: Vec<BinderFrame>, body: NormalizedFormula) -> Self {
        Self::exists_entries(binders.into_iter().map(Into::into).collect(), body)
    }

    pub fn exists_entries(binders: Vec<NormalizedBinderEntry>, body: NormalizedFormula) -> Self {
        Self::new(NormalizedFormulaKind::Exists {
            binders,
            body: Box::new(body),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NormalizedFormulaKind {
    Var(NormalizedVar),
    True,
    False,
    Atom {
        predicate: SymbolId,
        args: Vec<NormalizedTerm>,
    },
    Equals {
        left: NormalizedTerm,
        right: NormalizedTerm,
    },
    TypePred {
        subject: NormalizedTerm,
        ty: CoreTypePredicate,
    },
    Not(Box<NormalizedFormula>),
    And(Vec<NormalizedFormula>),
    Or(Vec<NormalizedFormula>),
    Implies {
        premise: Box<NormalizedFormula>,
        conclusion: Box<NormalizedFormula>,
    },
    Iff {
        left: Box<NormalizedFormula>,
        right: Box<NormalizedFormula>,
    },
    Forall {
        binders: Vec<NormalizedBinderEntry>,
        body: Box<NormalizedFormula>,
    },
    Exists {
        binders: Vec<NormalizedBinderEntry>,
        body: Box<NormalizedFormula>,
    },
    Error(CoreDiagnosticId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedOriginRecord {
    pub owner: CoreItemId,
    pub kind: GeneratedOriginKind,
    pub key: GeneratedOriginKey,
    pub params: Vec<NormalizedVar>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalTerm {
    kind: CanonicalTermKind,
}

impl CanonicalTerm {
    pub fn kind(&self) -> &CanonicalTermKind {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CanonicalTermKind {
    Var(CanonicalVar),
    Const(SymbolId),
    Apply {
        functor: SymbolId,
        args: Vec<CanonicalTerm>,
    },
    Select {
        selector: SymbolId,
        base: Box<CanonicalTerm>,
    },
    Tuple(Vec<CanonicalTerm>),
    SetEnum(Vec<CanonicalTerm>),
    Generated {
        origin: CanonicalGeneratedOrigin,
        args: Vec<CanonicalTerm>,
    },
    Error(CoreDiagnosticId),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalFormula {
    kind: CanonicalFormulaKind,
}

impl CanonicalFormula {
    pub fn kind(&self) -> &CanonicalFormulaKind {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CanonicalFormulaKind {
    Var(CanonicalVar),
    True,
    False,
    Atom {
        predicate: SymbolId,
        args: Vec<CanonicalTerm>,
    },
    Equals {
        left: CanonicalTerm,
        right: CanonicalTerm,
    },
    TypePred {
        subject: CanonicalTerm,
        ty: CoreTypePredicate,
    },
    Not(Box<CanonicalFormula>),
    And(Vec<CanonicalFormula>),
    Or(Vec<CanonicalFormula>),
    Implies {
        premise: Box<CanonicalFormula>,
        conclusion: Box<CanonicalFormula>,
    },
    Iff {
        left: Box<CanonicalFormula>,
        right: Box<CanonicalFormula>,
    },
    Forall {
        binders: Vec<CanonicalBinderEntry>,
        body: Box<CanonicalFormula>,
    },
    Exists {
        binders: Vec<CanonicalBinderEntry>,
        body: Box<CanonicalFormula>,
    },
    Error(CoreDiagnosticId),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CanonicalVar {
    Bound(u32),
    Free(CoreVarId),
    Schematic(CoreVarId),
    Generated(CoreVarId),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalBinderEntry {
    pub role: CoreVarRole,
    pub ty_guard: Option<Box<CanonicalFormula>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalGeneratedOrigin {
    pub owner: CoreItemId,
    pub kind: GeneratedOriginKind,
    pub key: GeneratedOriginKey,
    pub params: Vec<CanonicalVar>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NormalizedTermOrFormula {
    Term(NormalizedTerm),
    Formula(NormalizedFormula),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SubstitutionTarget {
    TermVar(CoreVarId),
    FormulaVar(CoreVarId),
}

impl SubstitutionTarget {
    fn var(&self) -> CoreVarId {
        match self {
            Self::TermVar(var) | Self::FormulaVar(var) => *var,
        }
    }

    const fn sort(&self) -> NormalizedVarSort {
        match self {
            Self::TermVar(_) => NormalizedVarSort::Term,
            Self::FormulaVar(_) => NormalizedVarSort::Formula,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SubstitutionReplacement {
    Term(NormalizedTerm),
    Formula(NormalizedFormula),
}

impl SubstitutionReplacement {
    fn sort(&self) -> NormalizedVarSort {
        match self {
            Self::Term(_) => NormalizedVarSort::Term,
            Self::Formula(_) => NormalizedVarSort::Formula,
        }
    }

    fn free_variables(&self) -> &BTreeSet<CoreVarId> {
        match self {
            Self::Term(term) => &term.free_variables,
            Self::Formula(formula) => &formula.free_variables,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CapturePolicy {
    Freshen,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstitutionSideConditions {
    pub forbidden_free_variables: BTreeSet<CoreVarId>,
    pub capture_policy: CapturePolicy,
    pub malformed_evidence: bool,
}

impl Default for SubstitutionSideConditions {
    fn default() -> Self {
        Self {
            forbidden_free_variables: BTreeSet::new(),
            capture_policy: CapturePolicy::Freshen,
            malformed_evidence: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshnessConfig {
    pub source_id: SourceId,
    pub owner: CoreItemId,
    pub binder_path: NormalizedTermOrFormulaPath,
    pub max_attempts: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NormalizedTermOrFormulaPath {
    segments: Vec<u32>,
}

impl NormalizedTermOrFormulaPath {
    pub fn new(segments: Vec<u32>) -> Self {
        Self { segments }
    }

    pub fn child(&self, segment: u32) -> Self {
        let mut segments = self.segments.clone();
        segments.push(segment);
        Self { segments }
    }

    pub fn segments(&self) -> &[u32] {
        &self.segments
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Substitution {
    pub target: SubstitutionTarget,
    pub replacement: SubstitutionReplacement,
    pub required_role: Option<CoreVarRole>,
    pub context: BinderContext,
    pub side_conditions: SubstitutionSideConditions,
    pub freshness: FreshnessConfig,
    pub diagnostic_source: CoreSourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SubstitutionResult<T> {
    Applied(SubstitutionOutput<T>),
    Rejected(BinderDiagnostic),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstitutionOutput<T> {
    pub value: T,
    pub freshness_witnesses: Vec<FreshnessWitness>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshnessWitness {
    pub source_id: SourceId,
    pub owner: CoreItemId,
    pub original: CoreVarId,
    pub fresh: CoreVarId,
    pub binder_path: NormalizedTermOrFormulaPath,
    pub role: CoreVarRole,
    pub counter: u32,
}

pub fn recompute_fresh_id(witness: &FreshnessWitness) -> CoreVarId {
    deterministic_fresh_id(
        witness.source_id,
        witness.owner,
        &witness.binder_path,
        &witness.role,
        witness.original,
        witness.counter,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinderDiagnostic {
    pub class: BinderDiagnosticClass,
    pub source: Box<CoreSourceRef>,
    pub owner: Option<Box<CoreNodeRef>>,
    pub message_key: CoreDiagnosticMessageKey,
}

impl BinderDiagnostic {
    pub fn new(
        class: BinderDiagnosticClass,
        source: CoreSourceRef,
        message_key: impl Into<CoreDiagnosticMessageKey>,
    ) -> Self {
        Self {
            class,
            source: Box::new(source),
            owner: None,
            message_key: message_key.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum BinderDiagnosticClass {
    ClassMismatch,
    SortMismatch,
    RoleMismatch,
    MissingVariableMetadata,
    SideConditionViolation,
    CaptureAvoidance,
    FreshnessExhausted,
    ClosureArityMismatch,
    InvalidBoundIndex,
    MalformedEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionClosure {
    pub formals: Vec<BinderFrame>,
    pub body: NormalizedTermOrFormula,
    pub captured_free_variables: BTreeSet<CoreVarId>,
    pub formal_type_guards: Vec<NormalizedFormula>,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionExpansion {
    pub body: NormalizedTermOrFormula,
    pub captured_free_variables: BTreeSet<CoreVarId>,
    pub guard_facts: Vec<NormalizedFormula>,
    pub freshness_witnesses: Vec<FreshnessWitness>,
}

pub fn shift_term(
    term: &NormalizedTerm,
    cutoff: u32,
    delta: i32,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedTerm> {
    shift_term_inner(term, cutoff, delta, diagnostic_source)
}

pub fn shift_formula(
    formula: &NormalizedFormula,
    cutoff: u32,
    delta: i32,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    shift_formula_inner(formula, cutoff, delta, diagnostic_source)
}

pub fn open_rec_term(
    term: &NormalizedTerm,
    depth: u32,
    replacement: &NormalizedTerm,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedTerm> {
    open_rec_term_inner(term, depth, replacement, diagnostic_source)
}

pub fn open_rec_formula_with_term(
    formula: &NormalizedFormula,
    depth: u32,
    replacement: &NormalizedTerm,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    open_rec_formula_with_term_inner(formula, depth, replacement, diagnostic_source)
}

pub fn open_rec_formula_with_formula(
    formula: &NormalizedFormula,
    depth: u32,
    replacement: &NormalizedFormula,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    open_rec_formula_with_formula_inner(formula, depth, replacement, diagnostic_source)
}

pub fn close_rec_term(
    term: &NormalizedTerm,
    depth: u32,
    variable: CoreVarId,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedTerm> {
    close_rec_term_inner(term, depth, variable, diagnostic_source)
}

pub fn close_rec_formula(
    formula: &NormalizedFormula,
    depth: u32,
    variable: CoreVarId,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    close_rec_formula_inner(formula, depth, variable, diagnostic_source)
}

pub fn subst_bound_term(
    term: &NormalizedTerm,
    depth: u32,
    replacement: &NormalizedTerm,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedTerm> {
    subst_bound_term_inner(term, depth, replacement, diagnostic_source)
}

pub fn subst_bound_formula(
    formula: &NormalizedFormula,
    depth: u32,
    replacement: &SubstitutionReplacement,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    subst_bound_formula_inner(formula, depth, replacement, diagnostic_source)
}

pub fn apply_substitution_to_term(
    term: &NormalizedTerm,
    substitution: &Substitution,
) -> SubstitutionResult<NormalizedTerm> {
    apply_substitution(
        term,
        substitution,
        validate_term_substitution_input,
        collect_term_binders,
        substitute_term_inner,
    )
}

pub fn apply_substitution_to_formula(
    formula: &NormalizedFormula,
    substitution: &Substitution,
) -> SubstitutionResult<NormalizedFormula> {
    apply_substitution(
        formula,
        substitution,
        validate_formula_substitution_input,
        collect_formula_binders,
        substitute_formula_inner,
    )
}

pub fn expand_definition_closure(
    closure: &DefinitionClosure,
    actuals: &[NormalizedTerm],
    context: &BinderContext,
    freshness: FreshnessConfig,
) -> SubstitutionResult<DefinitionExpansion> {
    if closure.formals.len() != actuals.len() {
        return SubstitutionResult::Rejected(BinderDiagnostic::new(
            BinderDiagnosticClass::ClosureArityMismatch,
            closure.source.clone(),
            "closure-arity-mismatch",
        ));
    }

    let base_depth = match context_depth(context, &closure.source) {
        Ok(depth) => depth,
        Err(diagnostic) => return SubstitutionResult::Rejected(diagnostic),
    };
    if let Err(diagnostic) = validate_context_frames(context, &closure.source) {
        return SubstitutionResult::Rejected(diagnostic);
    }
    for actual in actuals {
        if let Err(diagnostic) = validate_term_shape(actual, context, &closure.source, base_depth) {
            return SubstitutionResult::Rejected(diagnostic);
        }
    }
    let formal_depth = match u32::try_from(closure.formals.len()) {
        Ok(depth) => depth,
        Err(_) => {
            return SubstitutionResult::Rejected(BinderDiagnostic::new(
                BinderDiagnosticClass::InvalidBoundIndex,
                closure.source.clone(),
                "closure-formal-depth-overflow",
            ));
        }
    };
    if let Err(diagnostic) =
        validate_term_or_formula_shape(&closure.body, context, &closure.source, formal_depth)
    {
        return SubstitutionResult::Rejected(diagnostic);
    }
    for guard in &closure.formal_type_guards {
        if let Err(diagnostic) =
            validate_formula_shape(guard, context, &closure.source, formal_depth)
        {
            return SubstitutionResult::Rejected(diagnostic);
        }
    }

    let mut state = ClosureExpansionState::new(freshness, closure, actuals, context);
    let expanded_body = match instantiate_term_formals_capture_avoiding(
        closure.body.clone(),
        actuals,
        &mut state,
    ) {
        Ok(body) => body,
        Err(diagnostic) => return SubstitutionResult::Rejected(diagnostic),
    };
    let mut guard_facts = Vec::with_capacity(closure.formal_type_guards.len());
    for guard in &closure.formal_type_guards {
        let instantiated =
            instantiate_formula_term_formals_capture_avoiding(guard.clone(), actuals, &mut state);
        match instantiated {
            Ok(guard) => guard_facts.push(guard),
            Err(diagnostic) => return SubstitutionResult::Rejected(diagnostic),
        }
    }

    SubstitutionResult::Applied(SubstitutionOutput {
        value: DefinitionExpansion {
            body: expanded_body,
            captured_free_variables: closure.captured_free_variables.clone(),
            guard_facts,
            freshness_witnesses: state.freshness_witnesses.clone(),
        },
        freshness_witnesses: state.freshness_witnesses,
    })
}

pub fn normalize_core_term(
    core: &CoreIr,
    term_id: CoreTermId,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedTerm> {
    let mut state = RawNormalizationState::new(core, context, diagnostic_source);
    state.normalize_term(term_id)
}

pub fn normalize_core_formula(
    core: &CoreIr,
    formula_id: CoreFormulaId,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    let mut state = RawNormalizationState::new(core, context, diagnostic_source);
    state.normalize_formula(formula_id)
}

pub fn validate_normalized_term(
    term: &NormalizedTerm,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<()> {
    validate_context_frames(context, diagnostic_source)?;
    validate_term_shape(
        term,
        context,
        diagnostic_source,
        context_depth(context, diagnostic_source)?,
    )
}

pub fn validate_normalized_formula(
    formula: &NormalizedFormula,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<()> {
    validate_context_frames(context, diagnostic_source)?;
    validate_formula_shape(
        formula,
        context,
        diagnostic_source,
        context_depth(context, diagnostic_source)?,
    )
}

pub fn canonical_term(
    term: &NormalizedTerm,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<CanonicalTerm> {
    validate_normalized_term(term, context, diagnostic_source)?;
    Ok(canonicalize_term(term))
}

pub fn canonical_formula(
    formula: &NormalizedFormula,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<CanonicalFormula> {
    validate_normalized_formula(formula, context, diagnostic_source)?;
    Ok(canonicalize_formula(formula))
}

pub fn alpha_equivalent_terms(
    left: &NormalizedTerm,
    right: &NormalizedTerm,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<bool> {
    Ok(canonical_term(left, context, diagnostic_source)?
        == canonical_term(right, context, diagnostic_source)?)
}

pub fn alpha_equivalent_formulas(
    left: &NormalizedFormula,
    right: &NormalizedFormula,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<bool> {
    Ok(canonical_formula(left, context, diagnostic_source)?
        == canonical_formula(right, context, diagnostic_source)?)
}

struct RawNormalizationState<'a> {
    core: &'a CoreIr,
    context: &'a BinderContext,
    diagnostic_source: &'a CoreSourceRef,
    stack: Vec<BinderFrame>,
    next_canonical_index: u32,
}

impl<'a> RawNormalizationState<'a> {
    fn new(
        core: &'a CoreIr,
        context: &'a BinderContext,
        diagnostic_source: &'a CoreSourceRef,
    ) -> Self {
        Self {
            core,
            context,
            diagnostic_source,
            stack: context.frames.clone(),
            next_canonical_index: 0,
        }
    }

    fn normalize_term(&mut self, term_id: CoreTermId) -> BinderResult<NormalizedTerm> {
        let term = self
            .core
            .terms()
            .get(term_id)
            .ok_or_else(|| self.malformed("missing-core-term"))?;
        let kind = term.kind.clone();
        match kind {
            CoreTermKind::Var(var) => Ok(NormalizedTerm::var(self.normalize_var(var))),
            CoreTermKind::Const(symbol) => {
                Ok(NormalizedTerm::new(NormalizedTermKind::Const(symbol)))
            }
            CoreTermKind::Apply { functor, args } => {
                Ok(NormalizedTerm::new(NormalizedTermKind::Apply {
                    functor,
                    args: self.normalize_terms(&args)?,
                }))
            }
            CoreTermKind::Select { selector, base } => {
                Ok(NormalizedTerm::new(NormalizedTermKind::Select {
                    selector,
                    base: Box::new(self.normalize_term(base)?),
                }))
            }
            CoreTermKind::Tuple(items) => Ok(NormalizedTerm::new(NormalizedTermKind::Tuple(
                self.normalize_terms(&items)?,
            ))),
            CoreTermKind::SetEnum(items) => Ok(NormalizedTerm::new(NormalizedTermKind::SetEnum(
                self.normalize_terms(&items)?,
            ))),
            CoreTermKind::Generated { origin, args } => {
                let origin = self.normalize_generated_origin(origin)?;
                Ok(NormalizedTerm::new(NormalizedTermKind::Generated {
                    origin,
                    args: self.normalize_terms(&args)?,
                }))
            }
            CoreTermKind::Error(diagnostic) => {
                self.ensure_diagnostic(diagnostic, "missing-term-error-diagnostic")?;
                Ok(NormalizedTerm::new(NormalizedTermKind::Error(diagnostic)))
            }
        }
    }

    fn normalize_formula(&mut self, formula_id: CoreFormulaId) -> BinderResult<NormalizedFormula> {
        let formula = self
            .core
            .formulas()
            .get(formula_id)
            .ok_or_else(|| self.malformed("missing-core-formula"))?;
        let kind = formula.kind.clone();
        match kind {
            CoreFormulaKind::True => Ok(NormalizedFormula::new(NormalizedFormulaKind::True)),
            CoreFormulaKind::False => Ok(NormalizedFormula::new(NormalizedFormulaKind::False)),
            CoreFormulaKind::Atom { predicate, args } => {
                Ok(NormalizedFormula::new(NormalizedFormulaKind::Atom {
                    predicate,
                    args: self.normalize_terms(&args)?,
                }))
            }
            CoreFormulaKind::Equals { left, right } => {
                Ok(NormalizedFormula::new(NormalizedFormulaKind::Equals {
                    left: self.normalize_term(left)?,
                    right: self.normalize_term(right)?,
                }))
            }
            CoreFormulaKind::TypePred { subject, ty } => {
                Ok(NormalizedFormula::new(NormalizedFormulaKind::TypePred {
                    subject: self.normalize_term(subject)?,
                    ty,
                }))
            }
            CoreFormulaKind::Not(inner) => Ok(NormalizedFormula::new(NormalizedFormulaKind::Not(
                Box::new(self.normalize_formula(inner)?),
            ))),
            CoreFormulaKind::And(items) => Ok(NormalizedFormula::new(NormalizedFormulaKind::And(
                self.normalize_formulas(&items)?,
            ))),
            CoreFormulaKind::Or(items) => Ok(NormalizedFormula::new(NormalizedFormulaKind::Or(
                self.normalize_formulas(&items)?,
            ))),
            CoreFormulaKind::Implies {
                premise,
                conclusion,
            } => Ok(NormalizedFormula::new(NormalizedFormulaKind::Implies {
                premise: Box::new(self.normalize_formula(premise)?),
                conclusion: Box::new(self.normalize_formula(conclusion)?),
            })),
            CoreFormulaKind::Iff { left, right } => {
                Ok(NormalizedFormula::new(NormalizedFormulaKind::Iff {
                    left: Box::new(self.normalize_formula(left)?),
                    right: Box::new(self.normalize_formula(right)?),
                }))
            }
            CoreFormulaKind::Forall { binders, body } => {
                self.normalize_quantifier(NormalizedQuantifierKind::Forall, &binders, body)
            }
            CoreFormulaKind::Exists { binders, body } => {
                self.normalize_quantifier(NormalizedQuantifierKind::Exists, &binders, body)
            }
            CoreFormulaKind::Error(diagnostic) => {
                self.ensure_diagnostic(diagnostic, "missing-formula-error-diagnostic")?;
                Ok(NormalizedFormula::new(NormalizedFormulaKind::Error(
                    diagnostic,
                )))
            }
        }
    }

    fn normalize_terms(&mut self, terms: &[CoreTermId]) -> BinderResult<Vec<NormalizedTerm>> {
        terms
            .iter()
            .map(|term| self.normalize_term(*term))
            .collect()
    }

    fn normalize_formulas(
        &mut self,
        formulas: &[CoreFormulaId],
    ) -> BinderResult<Vec<NormalizedFormula>> {
        formulas
            .iter()
            .map(|formula| self.normalize_formula(*formula))
            .collect()
    }

    fn normalize_quantifier(
        &mut self,
        quantifier: NormalizedQuantifierKind,
        binders: &[CoreBinder],
        body: CoreFormulaId,
    ) -> BinderResult<NormalizedFormula> {
        let base_stack_len = self.stack.len();
        let base_next_index = self.next_canonical_index;
        let mut entries = Vec::with_capacity(binders.len());
        for (index, binder) in binders.iter().enumerate() {
            if let Some(guard) = binder.ty_guard {
                let later_vars = binders
                    .iter()
                    .skip(index + 1)
                    .map(|later| later.var)
                    .collect::<BTreeSet<_>>();
                if !later_vars.is_empty()
                    && core_formula_mentions_vars(self.core, guard, &later_vars, self)?
                {
                    self.stack.truncate(base_stack_len);
                    self.next_canonical_index = base_next_index;
                    return Err(self.malformed("binder-guard-references-later-binder"));
                }
            }
            let frame = self.frame_for_binder(binder)?;
            self.stack.push(frame.clone());
            let ty_guard = binder
                .ty_guard
                .map(|guard| self.normalize_formula(guard).map(Box::new))
                .transpose()?;
            entries.push(NormalizedBinderEntry { frame, ty_guard });
        }
        let body = self.normalize_formula(body);
        self.stack.truncate(base_stack_len);
        self.next_canonical_index = base_next_index;
        let body = body?;
        let kind = match quantifier {
            NormalizedQuantifierKind::Forall => NormalizedFormulaKind::Forall {
                binders: entries,
                body: Box::new(body),
            },
            NormalizedQuantifierKind::Exists => NormalizedFormulaKind::Exists {
                binders: entries,
                body: Box::new(body),
            },
        };
        Ok(NormalizedFormula::new(kind))
    }

    fn normalize_generated_origin(
        &self,
        origin_id: crate::core_ir::GeneratedOriginId,
    ) -> BinderResult<GeneratedOriginRecord> {
        let origin = self
            .core
            .generated()
            .get(origin_id)
            .ok_or_else(|| self.malformed("missing-generated-origin"))?;
        Ok(GeneratedOriginRecord {
            owner: origin.owner,
            kind: origin.kind,
            key: origin.key.clone(),
            params: origin
                .params
                .iter()
                .map(|param| self.normalize_var(*param))
                .collect(),
        })
    }

    fn frame_for_binder(&mut self, binder: &CoreBinder) -> BinderResult<BinderFrame> {
        let canonical_index = self.next_canonical_index;
        self.next_canonical_index = self
            .next_canonical_index
            .checked_add(1)
            .ok_or_else(|| self.malformed("binder-canonical-index-overflow"))?;
        Ok(BinderFrame {
            canonical_index,
            original_var: binder.var,
            role: binder.role.clone(),
            source_name: binder.source_name.clone(),
            source: binder.source.clone(),
        })
    }

    fn normalize_var(&self, var: CoreVarId) -> NormalizedVar {
        if let Some(index) = self
            .stack
            .iter()
            .rev()
            .position(|frame| frame.original_var == var)
        {
            return NormalizedVar::Bound(BoundVar::new(index as u32));
        }
        match self
            .context
            .variable_classes
            .get(&var)
            .copied()
            .unwrap_or(NormalizedVarClass::Free)
        {
            NormalizedVarClass::Free => NormalizedVar::Free(var),
            NormalizedVarClass::Schematic => NormalizedVar::Schematic(var),
            NormalizedVarClass::Generated => NormalizedVar::Generated(var),
        }
    }

    fn ensure_diagnostic(
        &self,
        diagnostic: CoreDiagnosticId,
        message_key: &'static str,
    ) -> BinderResult<()> {
        if self.core.diagnostics().get(diagnostic).is_some() {
            Ok(())
        } else {
            Err(self.malformed(message_key))
        }
    }

    fn malformed(&self, message_key: &'static str) -> BinderDiagnostic {
        BinderDiagnostic::new(
            BinderDiagnosticClass::MalformedEvidence,
            self.diagnostic_source.clone(),
            message_key,
        )
    }
}

fn canonicalize_term(term: &NormalizedTerm) -> CanonicalTerm {
    let kind = match &term.kind {
        NormalizedTermKind::Var(var) => CanonicalTermKind::Var(canonicalize_var(var)),
        NormalizedTermKind::Const(symbol) => CanonicalTermKind::Const(symbol.clone()),
        NormalizedTermKind::Apply { functor, args } => CanonicalTermKind::Apply {
            functor: functor.clone(),
            args: args.iter().map(canonicalize_term).collect(),
        },
        NormalizedTermKind::Select { selector, base } => CanonicalTermKind::Select {
            selector: selector.clone(),
            base: Box::new(canonicalize_term(base)),
        },
        NormalizedTermKind::Tuple(items) => {
            CanonicalTermKind::Tuple(items.iter().map(canonicalize_term).collect())
        }
        NormalizedTermKind::SetEnum(items) => {
            CanonicalTermKind::SetEnum(items.iter().map(canonicalize_term).collect())
        }
        NormalizedTermKind::Generated { origin, args } => CanonicalTermKind::Generated {
            origin: canonicalize_generated_origin(origin),
            args: args.iter().map(canonicalize_term).collect(),
        },
        NormalizedTermKind::Error(diagnostic) => CanonicalTermKind::Error(*diagnostic),
    };
    CanonicalTerm { kind }
}

fn canonicalize_formula(formula: &NormalizedFormula) -> CanonicalFormula {
    let kind = match &formula.kind {
        NormalizedFormulaKind::Var(var) => CanonicalFormulaKind::Var(canonicalize_var(var)),
        NormalizedFormulaKind::True => CanonicalFormulaKind::True,
        NormalizedFormulaKind::False => CanonicalFormulaKind::False,
        NormalizedFormulaKind::Atom { predicate, args } => CanonicalFormulaKind::Atom {
            predicate: predicate.clone(),
            args: args.iter().map(canonicalize_term).collect(),
        },
        NormalizedFormulaKind::Equals { left, right } => CanonicalFormulaKind::Equals {
            left: canonicalize_term(left),
            right: canonicalize_term(right),
        },
        NormalizedFormulaKind::TypePred { subject, ty } => CanonicalFormulaKind::TypePred {
            subject: canonicalize_term(subject),
            ty: ty.clone(),
        },
        NormalizedFormulaKind::Not(inner) => {
            CanonicalFormulaKind::Not(Box::new(canonicalize_formula(inner)))
        }
        NormalizedFormulaKind::And(items) => {
            CanonicalFormulaKind::And(items.iter().map(canonicalize_formula).collect())
        }
        NormalizedFormulaKind::Or(items) => {
            CanonicalFormulaKind::Or(items.iter().map(canonicalize_formula).collect())
        }
        NormalizedFormulaKind::Implies {
            premise,
            conclusion,
        } => CanonicalFormulaKind::Implies {
            premise: Box::new(canonicalize_formula(premise)),
            conclusion: Box::new(canonicalize_formula(conclusion)),
        },
        NormalizedFormulaKind::Iff { left, right } => CanonicalFormulaKind::Iff {
            left: Box::new(canonicalize_formula(left)),
            right: Box::new(canonicalize_formula(right)),
        },
        NormalizedFormulaKind::Forall { binders, body } => CanonicalFormulaKind::Forall {
            binders: binders.iter().map(canonicalize_binder_entry).collect(),
            body: Box::new(canonicalize_formula(body)),
        },
        NormalizedFormulaKind::Exists { binders, body } => CanonicalFormulaKind::Exists {
            binders: binders.iter().map(canonicalize_binder_entry).collect(),
            body: Box::new(canonicalize_formula(body)),
        },
        NormalizedFormulaKind::Error(diagnostic) => CanonicalFormulaKind::Error(*diagnostic),
    };
    CanonicalFormula { kind }
}

fn canonicalize_var(var: &NormalizedVar) -> CanonicalVar {
    match var {
        NormalizedVar::Bound(bound) => CanonicalVar::Bound(bound.index_from_innermost),
        NormalizedVar::Free(id) => CanonicalVar::Free(*id),
        NormalizedVar::Schematic(id) => CanonicalVar::Schematic(*id),
        NormalizedVar::Generated(id) => CanonicalVar::Generated(*id),
    }
}

fn canonicalize_binder_entry(entry: &NormalizedBinderEntry) -> CanonicalBinderEntry {
    CanonicalBinderEntry {
        role: entry.frame.role.clone(),
        ty_guard: entry
            .ty_guard
            .as_ref()
            .map(|guard| Box::new(canonicalize_formula(guard))),
    }
}

fn canonicalize_generated_origin(origin: &GeneratedOriginRecord) -> CanonicalGeneratedOrigin {
    CanonicalGeneratedOrigin {
        owner: origin.owner,
        kind: origin.kind,
        key: origin.key.clone(),
        params: origin.params.iter().map(canonicalize_var).collect(),
    }
}

fn core_formula_mentions_vars(
    core: &CoreIr,
    formula_id: CoreFormulaId,
    vars: &BTreeSet<CoreVarId>,
    state: &RawNormalizationState<'_>,
) -> BinderResult<bool> {
    let formula = core
        .formulas()
        .get(formula_id)
        .ok_or_else(|| state.malformed("missing-core-formula"))?;
    match &formula.kind {
        CoreFormulaKind::True | CoreFormulaKind::False | CoreFormulaKind::Error(_) => Ok(false),
        CoreFormulaKind::Atom { args, .. } => core_terms_mention_vars(core, args, vars, state),
        CoreFormulaKind::Equals { left, right } => {
            Ok(core_term_mentions_vars(core, *left, vars, state)?
                || core_term_mentions_vars(core, *right, vars, state)?)
        }
        CoreFormulaKind::TypePred { subject, .. } => {
            core_term_mentions_vars(core, *subject, vars, state)
        }
        CoreFormulaKind::Not(inner) => core_formula_mentions_vars(core, *inner, vars, state),
        CoreFormulaKind::And(items) | CoreFormulaKind::Or(items) => {
            for item in items {
                if core_formula_mentions_vars(core, *item, vars, state)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        CoreFormulaKind::Implies {
            premise,
            conclusion,
        } => Ok(core_formula_mentions_vars(core, *premise, vars, state)?
            || core_formula_mentions_vars(core, *conclusion, vars, state)?),
        CoreFormulaKind::Iff { left, right } => {
            Ok(core_formula_mentions_vars(core, *left, vars, state)?
                || core_formula_mentions_vars(core, *right, vars, state)?)
        }
        CoreFormulaKind::Forall { binders, body } | CoreFormulaKind::Exists { binders, body } => {
            for binder in binders {
                if vars.contains(&binder.var) {
                    return Ok(true);
                }
                if let Some(guard) = binder.ty_guard
                    && core_formula_mentions_vars(core, guard, vars, state)?
                {
                    return Ok(true);
                }
            }
            core_formula_mentions_vars(core, *body, vars, state)
        }
    }
}

fn core_terms_mention_vars(
    core: &CoreIr,
    terms: &[CoreTermId],
    vars: &BTreeSet<CoreVarId>,
    state: &RawNormalizationState<'_>,
) -> BinderResult<bool> {
    for term in terms {
        if core_term_mentions_vars(core, *term, vars, state)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn core_term_mentions_vars(
    core: &CoreIr,
    term_id: CoreTermId,
    vars: &BTreeSet<CoreVarId>,
    state: &RawNormalizationState<'_>,
) -> BinderResult<bool> {
    let term = core
        .terms()
        .get(term_id)
        .ok_or_else(|| state.malformed("missing-core-term"))?;
    match &term.kind {
        CoreTermKind::Var(var) => Ok(vars.contains(var)),
        CoreTermKind::Const(_) | CoreTermKind::Error(_) => Ok(false),
        CoreTermKind::Apply { args, .. }
        | CoreTermKind::Tuple(args)
        | CoreTermKind::SetEnum(args) => core_terms_mention_vars(core, args, vars, state),
        CoreTermKind::Generated { origin, args } => {
            let origin = core
                .generated()
                .get(*origin)
                .ok_or_else(|| state.malformed("missing-generated-origin"))?;
            if origin.params.iter().any(|param| vars.contains(param)) {
                return Ok(true);
            }
            core_terms_mention_vars(core, args, vars, state)
        }
        CoreTermKind::Select { base, .. } => core_term_mentions_vars(core, *base, vars, state),
    }
}

fn apply_substitution<Input, Output>(
    input: &Input,
    substitution: &Substitution,
    validate_input: fn(&Input, &Substitution) -> BinderResult<()>,
    collect_input_binders: fn(&Input, &mut BTreeSet<CoreVarId>),
    apply: fn(
        &Input,
        &mut SubstitutionState<'_>,
        u32,
        &NormalizedTermOrFormulaPath,
    ) -> BinderResult<Output>,
) -> SubstitutionResult<Output> {
    if let Err(diagnostic) = validate_substitution(substitution) {
        return SubstitutionResult::Rejected(diagnostic);
    }
    if let Err(diagnostic) = validate_input(input, substitution) {
        return SubstitutionResult::Rejected(diagnostic);
    }

    let mut state = SubstitutionState::new(substitution);
    collect_input_binders(input, &mut state.used_variables);
    match apply(input, &mut state, 0, &substitution.freshness.binder_path) {
        Ok(value) => SubstitutionResult::Applied(SubstitutionOutput {
            value,
            freshness_witnesses: state.freshness_witnesses,
        }),
        Err(diagnostic) => SubstitutionResult::Rejected(diagnostic),
    }
}

struct SubstitutionState<'a> {
    substitution: &'a Substitution,
    freshness_witnesses: Vec<FreshnessWitness>,
    used_variables: BTreeSet<CoreVarId>,
    next_counter: u32,
}

impl<'a> SubstitutionState<'a> {
    fn new(substitution: &'a Substitution) -> Self {
        let mut used_variables = substitution.context.free_variables.clone();
        used_variables.extend(substitution.replacement.free_variables().iter().copied());
        used_variables.extend(
            substitution
                .context
                .frames
                .iter()
                .map(|frame| frame.original_var),
        );
        Self {
            substitution,
            freshness_witnesses: Vec::new(),
            used_variables,
            next_counter: 0,
        }
    }

    fn target_var(&self) -> CoreVarId {
        self.substitution.target.var()
    }

    fn replacement_term(&self) -> Option<&NormalizedTerm> {
        match &self.substitution.replacement {
            SubstitutionReplacement::Term(term) => Some(term),
            SubstitutionReplacement::Formula(_) => None,
        }
    }

    fn replacement_formula(&self) -> Option<&NormalizedFormula> {
        match &self.substitution.replacement {
            SubstitutionReplacement::Term(_) => None,
            SubstitutionReplacement::Formula(formula) => Some(formula),
        }
    }

    fn diagnostic(
        &self,
        class: BinderDiagnosticClass,
        key: impl Into<CoreDiagnosticMessageKey>,
    ) -> BinderDiagnostic {
        BinderDiagnostic::new(
            class,
            self.substitution.diagnostic_source.clone(),
            key.into(),
        )
    }

    fn freshen_frame(
        &mut self,
        frame: &mut BinderFrame,
        path: &NormalizedTermOrFormulaPath,
    ) -> BinderResult<()> {
        if self.substitution.side_conditions.capture_policy == CapturePolicy::Reject {
            return Err(self.diagnostic(BinderDiagnosticClass::CaptureAvoidance, "capture"));
        }

        let max_attempts = self.substitution.freshness.max_attempts;
        for _ in 0..max_attempts {
            let counter = self.next_counter;
            self.next_counter = self.next_counter.saturating_add(1);
            let original = frame.original_var;
            let candidate = deterministic_fresh_id(
                self.substitution.freshness.source_id,
                self.substitution.freshness.owner,
                path,
                &frame.role,
                original,
                counter,
            );
            if self.used_variables.insert(candidate) {
                frame.original_var = candidate;
                self.freshness_witnesses.push(FreshnessWitness {
                    source_id: self.substitution.freshness.source_id,
                    owner: self.substitution.freshness.owner,
                    original,
                    fresh: candidate,
                    binder_path: path.clone(),
                    role: frame.role.clone(),
                    counter,
                });
                return Ok(());
            }
        }

        Err(self.diagnostic(
            BinderDiagnosticClass::FreshnessExhausted,
            "freshness-exhausted",
        ))
    }
}

struct ClosureExpansionState {
    freshness: FreshnessConfig,
    diagnostic_source: CoreSourceRef,
    freshness_witnesses: Vec<FreshnessWitness>,
    used_variables: BTreeSet<CoreVarId>,
    next_counter: u32,
}

impl ClosureExpansionState {
    fn new(
        freshness: FreshnessConfig,
        closure: &DefinitionClosure,
        actuals: &[NormalizedTerm],
        context: &BinderContext,
    ) -> Self {
        let mut used_variables = context.free_variables.clone();
        used_variables.extend(closure.captured_free_variables.iter().copied());
        used_variables.extend(closure.formals.iter().map(|frame| frame.original_var));
        collect_term_or_formula_binders(&closure.body, &mut used_variables);
        for guard in &closure.formal_type_guards {
            collect_formula_binders(guard, &mut used_variables);
        }
        for actual in actuals {
            used_variables.extend(actual.free_variables.iter().copied());
        }
        Self {
            freshness,
            diagnostic_source: closure.source.clone(),
            freshness_witnesses: Vec::new(),
            used_variables,
            next_counter: 0,
        }
    }

    fn diagnostic(
        &self,
        class: BinderDiagnosticClass,
        key: impl Into<CoreDiagnosticMessageKey>,
    ) -> BinderDiagnostic {
        BinderDiagnostic::new(class, self.diagnostic_source.clone(), key.into())
    }

    fn freshen_frame(
        &mut self,
        frame: &mut BinderFrame,
        path: &NormalizedTermOrFormulaPath,
    ) -> BinderResult<()> {
        let max_attempts = self.freshness.max_attempts;
        for _ in 0..max_attempts {
            let counter = self.next_counter;
            self.next_counter = self.next_counter.saturating_add(1);
            let original = frame.original_var;
            let candidate = deterministic_fresh_id(
                self.freshness.source_id,
                self.freshness.owner,
                path,
                &frame.role,
                original,
                counter,
            );
            if self.used_variables.insert(candidate) {
                frame.original_var = candidate;
                self.freshness_witnesses.push(FreshnessWitness {
                    source_id: self.freshness.source_id,
                    owner: self.freshness.owner,
                    original,
                    fresh: candidate,
                    binder_path: path.clone(),
                    role: frame.role.clone(),
                    counter,
                });
                return Ok(());
            }
        }

        Err(self.diagnostic(
            BinderDiagnosticClass::FreshnessExhausted,
            "freshness-exhausted",
        ))
    }
}

fn validate_substitution(substitution: &Substitution) -> BinderResult<()> {
    if substitution.side_conditions.malformed_evidence {
        return Err(BinderDiagnostic::new(
            BinderDiagnosticClass::MalformedEvidence,
            substitution.diagnostic_source.clone(),
            "malformed-substitution-evidence",
        ));
    }
    validate_context_frames(&substitution.context, &substitution.diagnostic_source)?;

    if substitution.target.sort() != substitution.replacement.sort() {
        return Err(BinderDiagnostic::new(
            BinderDiagnosticClass::SortMismatch,
            substitution.diagnostic_source.clone(),
            "substitution-sort-mismatch",
        ));
    }

    let target = substitution.target.var();
    if !substitution.context.variable_classes.contains_key(&target)
        || !substitution.context.variable_roles.contains_key(&target)
        || !substitution.context.variable_sorts.contains_key(&target)
    {
        return Err(BinderDiagnostic::new(
            BinderDiagnosticClass::MissingVariableMetadata,
            substitution.diagnostic_source.clone(),
            "missing-variable-metadata",
        ));
    }

    if substitution.context.variable_sorts.get(&target) != Some(&substitution.target.sort()) {
        return Err(BinderDiagnostic::new(
            BinderDiagnosticClass::SortMismatch,
            substitution.diagnostic_source.clone(),
            "target-sort-mismatch",
        ));
    }

    let base_depth = context_depth(&substitution.context, &substitution.diagnostic_source)?;
    match &substitution.replacement {
        SubstitutionReplacement::Term(term) => validate_term_shape(
            term,
            &substitution.context,
            &substitution.diagnostic_source,
            base_depth,
        )?,
        SubstitutionReplacement::Formula(formula) => validate_formula_shape(
            formula,
            &substitution.context,
            &substitution.diagnostic_source,
            base_depth,
        )?,
    }

    if let Some(required_role) = &substitution.required_role
        && substitution.context.variable_roles.get(&target) != Some(required_role)
    {
        return Err(BinderDiagnostic::new(
            BinderDiagnosticClass::RoleMismatch,
            substitution.diagnostic_source.clone(),
            "target-role-mismatch",
        ));
    }

    if substitution.replacement.free_variables().iter().any(|var| {
        substitution
            .side_conditions
            .forbidden_free_variables
            .contains(var)
    }) {
        return Err(BinderDiagnostic::new(
            BinderDiagnosticClass::SideConditionViolation,
            substitution.diagnostic_source.clone(),
            "free-variable-side-condition",
        ));
    }

    Ok(())
}

fn validate_term_substitution_input(
    term: &NormalizedTerm,
    substitution: &Substitution,
) -> BinderResult<()> {
    validate_term_shape(
        term,
        &substitution.context,
        &substitution.diagnostic_source,
        context_depth(&substitution.context, &substitution.diagnostic_source)?,
    )
}

fn validate_formula_substitution_input(
    formula: &NormalizedFormula,
    substitution: &Substitution,
) -> BinderResult<()> {
    validate_formula_shape(
        formula,
        &substitution.context,
        &substitution.diagnostic_source,
        context_depth(&substitution.context, &substitution.diagnostic_source)?,
    )
}

fn validate_term_or_formula_shape(
    node: &NormalizedTermOrFormula,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
    depth: u32,
) -> BinderResult<()> {
    match node {
        NormalizedTermOrFormula::Term(term) => {
            validate_term_shape(term, context, diagnostic_source, depth)
        }
        NormalizedTermOrFormula::Formula(formula) => {
            validate_formula_shape(formula, context, diagnostic_source, depth)
        }
    }
}

fn collect_term_binders(_term: &NormalizedTerm, _used: &mut BTreeSet<CoreVarId>) {}

fn collect_term_or_formula_binders(node: &NormalizedTermOrFormula, used: &mut BTreeSet<CoreVarId>) {
    match node {
        NormalizedTermOrFormula::Term(term) => collect_term_binders(term, used),
        NormalizedTermOrFormula::Formula(formula) => collect_formula_binders(formula, used),
    }
}

fn collect_formula_binders(formula: &NormalizedFormula, used: &mut BTreeSet<CoreVarId>) {
    match &formula.kind {
        NormalizedFormulaKind::Var(_)
        | NormalizedFormulaKind::True
        | NormalizedFormulaKind::False
        | NormalizedFormulaKind::Atom { .. }
        | NormalizedFormulaKind::Equals { .. }
        | NormalizedFormulaKind::TypePred { .. }
        | NormalizedFormulaKind::Error(_) => {}
        NormalizedFormulaKind::Not(inner) => collect_formula_binders(inner, used),
        NormalizedFormulaKind::And(items) | NormalizedFormulaKind::Or(items) => {
            for item in items {
                collect_formula_binders(item, used);
            }
        }
        NormalizedFormulaKind::Implies {
            premise,
            conclusion,
        } => {
            collect_formula_binders(premise, used);
            collect_formula_binders(conclusion, used);
        }
        NormalizedFormulaKind::Iff { left, right } => {
            collect_formula_binders(left, used);
            collect_formula_binders(right, used);
        }
        NormalizedFormulaKind::Forall { binders, body }
        | NormalizedFormulaKind::Exists { binders, body } => {
            used.extend(binders.iter().map(|entry| entry.frame.original_var));
            for entry in binders {
                if let Some(guard) = &entry.ty_guard {
                    collect_formula_binders(guard, used);
                }
            }
            collect_formula_binders(body, used);
        }
    }
}

fn validate_term_shape(
    term: &NormalizedTerm,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
    depth: u32,
) -> BinderResult<()> {
    match &term.kind {
        NormalizedTermKind::Var(var) => validate_var_shape(
            var,
            NormalizedVarSort::Term,
            context,
            diagnostic_source,
            depth,
        ),
        NormalizedTermKind::Const(_) | NormalizedTermKind::Error(_) => Ok(()),
        NormalizedTermKind::Select { base, .. } => {
            validate_term_shape(base, context, diagnostic_source, depth)
        }
        NormalizedTermKind::Apply { args, .. }
        | NormalizedTermKind::Tuple(args)
        | NormalizedTermKind::SetEnum(args) => {
            for arg in args {
                validate_term_shape(arg, context, diagnostic_source, depth)?;
            }
            Ok(())
        }
        NormalizedTermKind::Generated { origin, args } => {
            validate_generated_origin_shape(origin, context, diagnostic_source, depth)?;
            for arg in args {
                validate_term_shape(arg, context, diagnostic_source, depth)?;
            }
            Ok(())
        }
    }
    .and_then(|()| validate_term_free_cache(term, diagnostic_source))
}

fn validate_formula_shape(
    formula: &NormalizedFormula,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
    depth: u32,
) -> BinderResult<()> {
    match &formula.kind {
        NormalizedFormulaKind::Var(var) => validate_var_shape(
            var,
            NormalizedVarSort::Formula,
            context,
            diagnostic_source,
            depth,
        ),
        NormalizedFormulaKind::True
        | NormalizedFormulaKind::False
        | NormalizedFormulaKind::Error(_) => Ok(()),
        NormalizedFormulaKind::Atom { args, .. } => {
            for arg in args {
                validate_term_shape(arg, context, diagnostic_source, depth)?;
            }
            Ok(())
        }
        NormalizedFormulaKind::Equals { left, right } => {
            validate_term_shape(left, context, diagnostic_source, depth)?;
            validate_term_shape(right, context, diagnostic_source, depth)
        }
        NormalizedFormulaKind::TypePred { subject, .. } => {
            validate_term_shape(subject, context, diagnostic_source, depth)
        }
        NormalizedFormulaKind::Not(inner) => {
            validate_formula_shape(inner, context, diagnostic_source, depth)
        }
        NormalizedFormulaKind::And(items) | NormalizedFormulaKind::Or(items) => {
            for item in items {
                validate_formula_shape(item, context, diagnostic_source, depth)?;
            }
            Ok(())
        }
        NormalizedFormulaKind::Implies {
            premise,
            conclusion,
        } => {
            validate_formula_shape(premise, context, diagnostic_source, depth)?;
            validate_formula_shape(conclusion, context, diagnostic_source, depth)
        }
        NormalizedFormulaKind::Iff { left, right } => {
            validate_formula_shape(left, context, diagnostic_source, depth)?;
            validate_formula_shape(right, context, diagnostic_source, depth)
        }
        NormalizedFormulaKind::Forall { binders, body }
        | NormalizedFormulaKind::Exists { binders, body } => {
            for (index, entry) in binders.iter().enumerate() {
                validate_binder_frame_shape(&entry.frame, context, diagnostic_source)?;
                let guard_depth = close_nested_depth(depth, index + 1, diagnostic_source)?;
                if let Some(guard) = &entry.ty_guard {
                    validate_formula_shape(guard, context, diagnostic_source, guard_depth)?;
                    let in_scope_binder_mentions = binders
                        .iter()
                        .take(index + 1)
                        .any(|scoped| guard.free_variables.contains(&scoped.frame.original_var));
                    if in_scope_binder_mentions {
                        return Err(BinderDiagnostic::new(
                            BinderDiagnosticClass::MalformedEvidence,
                            diagnostic_source.clone(),
                            "binder-guard-uses-free-original-var",
                        ));
                    }
                    let later_binder_mentions = binders
                        .iter()
                        .skip(index + 1)
                        .any(|later| guard.free_variables.contains(&later.frame.original_var));
                    if later_binder_mentions {
                        return Err(BinderDiagnostic::new(
                            BinderDiagnosticClass::MalformedEvidence,
                            diagnostic_source.clone(),
                            "binder-guard-references-later-binder",
                        ));
                    }
                }
            }
            let nested_depth = close_nested_depth(depth, binders.len(), diagnostic_source)?;
            validate_formula_shape(body, context, diagnostic_source, nested_depth)?;
            if binders
                .iter()
                .any(|entry| body.free_variables.contains(&entry.frame.original_var))
            {
                return Err(BinderDiagnostic::new(
                    BinderDiagnosticClass::MalformedEvidence,
                    diagnostic_source.clone(),
                    "binder-body-uses-free-original-var",
                ));
            }
            Ok(())
        }
    }
    .and_then(|()| validate_formula_free_cache(formula, diagnostic_source))
}

fn validate_term_free_cache(
    term: &NormalizedTerm,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<()> {
    let expected = term_free_variables(&term.kind);
    if term.free_variables == expected {
        Ok(())
    } else {
        Err(BinderDiagnostic::new(
            BinderDiagnosticClass::MalformedEvidence,
            diagnostic_source.clone(),
            "term-free-variable-cache-mismatch",
        ))
    }
}

fn validate_formula_free_cache(
    formula: &NormalizedFormula,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<()> {
    let expected = formula_free_variables(&formula.kind);
    if formula.free_variables == expected {
        Ok(())
    } else {
        Err(BinderDiagnostic::new(
            BinderDiagnosticClass::MalformedEvidence,
            diagnostic_source.clone(),
            "formula-free-variable-cache-mismatch",
        ))
    }
}

fn validate_generated_origin_shape(
    origin: &GeneratedOriginRecord,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
    depth: u32,
) -> BinderResult<()> {
    for param in &origin.params {
        validate_var_shape(
            param,
            NormalizedVarSort::Term,
            context,
            diagnostic_source,
            depth,
        )?;
    }
    Ok(())
}

fn validate_context_frames(
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<()> {
    for frame in &context.frames {
        validate_binder_frame_shape(frame, context, diagnostic_source)?;
    }
    Ok(())
}

fn validate_binder_frame_shape(
    frame: &BinderFrame,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<()> {
    let class = context.variable_classes.get(&frame.original_var);
    let role = context.variable_roles.get(&frame.original_var);
    let sort = context.variable_sorts.get(&frame.original_var);
    if class.is_none() && role.is_none() && sort.is_none() {
        return Ok(());
    }
    let Some(role) = role else {
        return Err(BinderDiagnostic::new(
            BinderDiagnosticClass::MissingVariableMetadata,
            diagnostic_source.clone(),
            "missing-binder-role",
        ));
    };
    if role != &frame.role {
        return Err(BinderDiagnostic::new(
            BinderDiagnosticClass::RoleMismatch,
            diagnostic_source.clone(),
            "binder-role-mismatch",
        ));
    }
    if class.is_none() || sort.is_none() {
        return Err(BinderDiagnostic::new(
            BinderDiagnosticClass::MissingVariableMetadata,
            diagnostic_source.clone(),
            "missing-binder-metadata",
        ));
    }
    if sort != Some(&NormalizedVarSort::Term) {
        return Err(BinderDiagnostic::new(
            BinderDiagnosticClass::SortMismatch,
            diagnostic_source.clone(),
            "binder-sort-mismatch",
        ));
    }
    Ok(())
}

fn validate_var_shape(
    var: &NormalizedVar,
    expected_sort: NormalizedVarSort,
    context: &BinderContext,
    diagnostic_source: &CoreSourceRef,
    depth: u32,
) -> BinderResult<()> {
    let Some((id, observed_class)) = var_metadata(var) else {
        let NormalizedVar::Bound(bound) = var else {
            unreachable!("only bound variables do not have metadata");
        };
        return if bound.index_from_innermost < depth {
            Ok(())
        } else {
            Err(BinderDiagnostic::new(
                BinderDiagnosticClass::InvalidBoundIndex,
                diagnostic_source.clone(),
                "invalid-bound-index",
            ))
        };
    };

    let Some(actual_class) = context.variable_classes.get(&id) else {
        return Err(BinderDiagnostic::new(
            BinderDiagnosticClass::MissingVariableMetadata,
            diagnostic_source.clone(),
            "missing-variable-class",
        ));
    };
    if *actual_class != observed_class {
        return Err(BinderDiagnostic::new(
            BinderDiagnosticClass::ClassMismatch,
            diagnostic_source.clone(),
            "variable-class-mismatch",
        ));
    }
    if !context.variable_roles.contains_key(&id) {
        return Err(BinderDiagnostic::new(
            BinderDiagnosticClass::MissingVariableMetadata,
            diagnostic_source.clone(),
            "missing-variable-role",
        ));
    }
    match context.variable_sorts.get(&id) {
        Some(sort) if *sort == expected_sort => Ok(()),
        Some(_) => Err(BinderDiagnostic::new(
            BinderDiagnosticClass::SortMismatch,
            diagnostic_source.clone(),
            "variable-sort-mismatch",
        )),
        None => Err(BinderDiagnostic::new(
            BinderDiagnosticClass::MissingVariableMetadata,
            diagnostic_source.clone(),
            "missing-variable-sort",
        )),
    }
}

fn var_metadata(var: &NormalizedVar) -> Option<(CoreVarId, NormalizedVarClass)> {
    match var {
        NormalizedVar::Bound(_) => None,
        NormalizedVar::Free(id) => Some((*id, NormalizedVarClass::Free)),
        NormalizedVar::Schematic(id) => Some((*id, NormalizedVarClass::Schematic)),
        NormalizedVar::Generated(id) => Some((*id, NormalizedVarClass::Generated)),
    }
}

fn deterministic_fresh_id(
    source_id: SourceId,
    owner: CoreItemId,
    binder_path: &NormalizedTermOrFormulaPath,
    role: &CoreVarRole,
    original: CoreVarId,
    counter: u32,
) -> CoreVarId {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    fn feed_bytes(hash: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *hash ^= u64::from(*byte);
            *hash = hash.wrapping_mul(FNV_PRIME);
        }
    }

    fn feed_u64(hash: &mut u64, value: u64) {
        feed_bytes(hash, &value.to_le_bytes());
    }

    fn feed_u32(hash: &mut u64, value: u32) {
        feed_bytes(hash, &value.to_le_bytes());
    }

    let mut hash = FNV_OFFSET;
    feed_bytes(&mut hash, format!("{source_id:?}").as_bytes());
    feed_u64(&mut hash, owner.index() as u64);
    feed_u64(&mut hash, original.index() as u64);
    feed_bytes(&mut hash, role.as_str().as_bytes());
    for segment in binder_path.segments() {
        feed_u32(&mut hash, *segment);
    }
    feed_u32(&mut hash, counter);
    CoreVarId::new((hash & 0x7fff_ffff) as usize)
}

fn depth_delta(depth: u32, diagnostic_source: &CoreSourceRef) -> BinderResult<i32> {
    i32::try_from(depth).map_err(|_| {
        BinderDiagnostic::new(
            BinderDiagnosticClass::InvalidBoundIndex,
            diagnostic_source.clone(),
            "binder-depth-overflow",
        )
    })
}

fn substitute_term_inner(
    term: &NormalizedTerm,
    state: &mut SubstitutionState<'_>,
    depth: u32,
    _path: &NormalizedTermOrFormulaPath,
) -> BinderResult<NormalizedTerm> {
    match &term.kind {
        NormalizedTermKind::Var(var)
            if matches!(state.substitution.target, SubstitutionTarget::TermVar(_))
                && var.free_id() == Some(state.target_var()) =>
        {
            let replacement = state
                .replacement_term()
                .expect("validated term replacement for term target");
            shift_term_inner(
                replacement,
                0,
                depth_delta(depth, &state.substitution.diagnostic_source)?,
                &state.substitution.diagnostic_source,
            )
        }
        NormalizedTermKind::Var(_)
        | NormalizedTermKind::Const(_)
        | NormalizedTermKind::Error(_) => Ok(term.clone()),
        NormalizedTermKind::Select { selector, base } => {
            Ok(NormalizedTerm::new(NormalizedTermKind::Select {
                selector: selector.clone(),
                base: Box::new(substitute_term_inner(base, state, depth, _path)?),
            }))
        }
        NormalizedTermKind::Apply { functor, args } => {
            Ok(NormalizedTerm::new(NormalizedTermKind::Apply {
                functor: functor.clone(),
                args: substitute_terms(args, state, depth)?,
            }))
        }
        NormalizedTermKind::Tuple(items) => Ok(NormalizedTerm::new(NormalizedTermKind::Tuple(
            substitute_terms(items, state, depth)?,
        ))),
        NormalizedTermKind::SetEnum(items) => Ok(NormalizedTerm::new(NormalizedTermKind::SetEnum(
            substitute_terms(items, state, depth)?,
        ))),
        NormalizedTermKind::Generated { origin, args } => {
            let origin = substitute_generated_origin(origin, state, depth)?;
            Ok(NormalizedTerm::new(NormalizedTermKind::Generated {
                origin,
                args: substitute_terms(args, state, depth)?,
            }))
        }
    }
}

fn substitute_formula_inner(
    formula: &NormalizedFormula,
    state: &mut SubstitutionState<'_>,
    depth: u32,
    path: &NormalizedTermOrFormulaPath,
) -> BinderResult<NormalizedFormula> {
    match &formula.kind {
        NormalizedFormulaKind::Var(var)
            if matches!(state.substitution.target, SubstitutionTarget::FormulaVar(_))
                && var.free_id() == Some(state.target_var()) =>
        {
            let replacement = state
                .replacement_formula()
                .expect("validated formula replacement for formula target");
            shift_formula_inner(
                replacement,
                0,
                depth_delta(depth, &state.substitution.diagnostic_source)?,
                &state.substitution.diagnostic_source,
            )
        }
        NormalizedFormulaKind::Var(_)
        | NormalizedFormulaKind::True
        | NormalizedFormulaKind::False
        | NormalizedFormulaKind::Error(_) => Ok(formula.clone()),
        NormalizedFormulaKind::Atom { predicate, args } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Atom {
                predicate: predicate.clone(),
                args: substitute_terms(args, state, depth)?,
            }))
        }
        NormalizedFormulaKind::Equals { left, right } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Equals {
                left: substitute_term_inner(left, state, depth, path)?,
                right: substitute_term_inner(right, state, depth, path)?,
            }))
        }
        NormalizedFormulaKind::TypePred { subject, ty } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::TypePred {
                subject: substitute_term_inner(subject, state, depth, path)?,
                ty: ty.clone(),
            }))
        }
        NormalizedFormulaKind::Not(inner) => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Not(
                Box::new(substitute_formula_inner(inner, state, depth, path)?),
            )))
        }
        NormalizedFormulaKind::And(items) => Ok(NormalizedFormula::new(
            NormalizedFormulaKind::And(substitute_formulas(items, state, depth, path)?),
        )),
        NormalizedFormulaKind::Or(items) => Ok(NormalizedFormula::new(NormalizedFormulaKind::Or(
            substitute_formulas(items, state, depth, path)?,
        ))),
        NormalizedFormulaKind::Implies {
            premise,
            conclusion,
        } => Ok(NormalizedFormula::new(NormalizedFormulaKind::Implies {
            premise: Box::new(substitute_formula_inner(premise, state, depth, path)?),
            conclusion: Box::new(substitute_formula_inner(conclusion, state, depth, path)?),
        })),
        NormalizedFormulaKind::Iff { left, right } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Iff {
                left: Box::new(substitute_formula_inner(left, state, depth, path)?),
                right: Box::new(substitute_formula_inner(right, state, depth, path)?),
            }))
        }
        NormalizedFormulaKind::Forall { binders, body } => substitute_under_binders(
            NormalizedQuantifierKind::Forall,
            binders,
            body,
            state,
            depth,
            path,
        ),
        NormalizedFormulaKind::Exists { binders, body } => substitute_under_binders(
            NormalizedQuantifierKind::Exists,
            binders,
            body,
            state,
            depth,
            path,
        ),
    }
}

#[derive(Debug, Clone, Copy)]
enum NormalizedQuantifierKind {
    Forall,
    Exists,
}

fn substitute_under_binders(
    quantifier: NormalizedQuantifierKind,
    binders: &[NormalizedBinderEntry],
    body: &NormalizedFormula,
    state: &mut SubstitutionState<'_>,
    depth: u32,
    path: &NormalizedTermOrFormulaPath,
) -> BinderResult<NormalizedFormula> {
    let mut binders = binders.to_vec();
    state
        .used_variables
        .extend(body.free_variables.iter().copied());
    state
        .used_variables
        .extend(binders.iter().map(|entry| entry.frame.original_var));
    for entry in &binders {
        if let Some(guard) = &entry.ty_guard {
            state
                .used_variables
                .extend(guard.free_variables.iter().copied());
        }
    }

    let replacement_free = state.substitution.replacement.free_variables().clone();
    let mut target_shadowed = false;
    for (index, entry) in binders.iter_mut().enumerate() {
        let entry_shadows_target = entry.frame.original_var == state.target_var();
        if !target_shadowed
            && !entry_shadows_target
            && replacement_free.contains(&entry.frame.original_var)
        {
            let frame_path = path.child(entry.frame.canonical_index);
            state.freshen_frame(&mut entry.frame, &frame_path)?;
        }

        if let Some(guard) = &entry.ty_guard {
            let guard_depth = add_depth(depth, index + 1, state)?;
            let guard_path = path.child(entry.frame.canonical_index);
            if target_shadowed || entry_shadows_target {
                entry.ty_guard = Some(guard.clone());
            } else {
                entry.ty_guard = Some(Box::new(substitute_formula_inner(
                    guard,
                    state,
                    guard_depth,
                    &guard_path,
                )?));
            }
        }
        target_shadowed |= entry_shadows_target;
    }

    let nested_depth = add_depth(depth, binders.len(), state)?;
    let nested_path = binders.iter().fold(path.clone(), |path, entry| {
        path.child(entry.frame.canonical_index)
    });
    let body = if target_shadowed {
        body.clone()
    } else {
        substitute_formula_inner(body, state, nested_depth, &nested_path)?
    };
    let kind = match quantifier {
        NormalizedQuantifierKind::Forall => NormalizedFormulaKind::Forall {
            binders,
            body: Box::new(body),
        },
        NormalizedQuantifierKind::Exists => NormalizedFormulaKind::Exists {
            binders,
            body: Box::new(body),
        },
    };
    Ok(NormalizedFormula::new(kind))
}

fn substitute_terms(
    terms: &[NormalizedTerm],
    state: &mut SubstitutionState<'_>,
    depth: u32,
) -> BinderResult<Vec<NormalizedTerm>> {
    let path = state.substitution.freshness.binder_path.clone();
    terms
        .iter()
        .map(|term| substitute_term_inner(term, state, depth, &path))
        .collect()
}

fn substitute_formulas(
    formulas: &[NormalizedFormula],
    state: &mut SubstitutionState<'_>,
    depth: u32,
    path: &NormalizedTermOrFormulaPath,
) -> BinderResult<Vec<NormalizedFormula>> {
    formulas
        .iter()
        .map(|formula| substitute_formula_inner(formula, state, depth, path))
        .collect()
}

fn shift_term_inner(
    term: &NormalizedTerm,
    cutoff: u32,
    delta: i32,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedTerm> {
    match &term.kind {
        NormalizedTermKind::Var(var) => Ok(NormalizedTerm::var(shift_var(
            var,
            cutoff,
            delta,
            diagnostic_source,
        )?)),
        NormalizedTermKind::Const(_) | NormalizedTermKind::Error(_) => Ok(term.clone()),
        NormalizedTermKind::Select { selector, base } => {
            Ok(NormalizedTerm::new(NormalizedTermKind::Select {
                selector: selector.clone(),
                base: Box::new(shift_term_inner(base, cutoff, delta, diagnostic_source)?),
            }))
        }
        NormalizedTermKind::Apply { functor, args } => {
            let args = args
                .iter()
                .map(|arg| shift_term_inner(arg, cutoff, delta, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedTerm::new(NormalizedTermKind::Apply {
                functor: functor.clone(),
                args,
            }))
        }
        NormalizedTermKind::Tuple(items) => {
            let items = items
                .iter()
                .map(|item| shift_term_inner(item, cutoff, delta, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedTerm::new(NormalizedTermKind::Tuple(items)))
        }
        NormalizedTermKind::SetEnum(items) => {
            let items = items
                .iter()
                .map(|item| shift_term_inner(item, cutoff, delta, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedTerm::new(NormalizedTermKind::SetEnum(items)))
        }
        NormalizedTermKind::Generated { origin, args } => {
            let origin = shift_generated_origin(origin, cutoff, delta, diagnostic_source)?;
            let args = args
                .iter()
                .map(|arg| shift_term_inner(arg, cutoff, delta, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedTerm::new(NormalizedTermKind::Generated {
                origin,
                args,
            }))
        }
    }
}

fn shift_formula_inner(
    formula: &NormalizedFormula,
    cutoff: u32,
    delta: i32,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    match &formula.kind {
        NormalizedFormulaKind::Var(var) => Ok(NormalizedFormula::var(shift_var(
            var,
            cutoff,
            delta,
            diagnostic_source,
        )?)),
        NormalizedFormulaKind::True
        | NormalizedFormulaKind::False
        | NormalizedFormulaKind::Error(_) => Ok(formula.clone()),
        NormalizedFormulaKind::Atom { predicate, args } => {
            let args = args
                .iter()
                .map(|arg| shift_term_inner(arg, cutoff, delta, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Atom {
                predicate: predicate.clone(),
                args,
            }))
        }
        NormalizedFormulaKind::Equals { left, right } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Equals {
                left: shift_term_inner(left, cutoff, delta, diagnostic_source)?,
                right: shift_term_inner(right, cutoff, delta, diagnostic_source)?,
            }))
        }
        NormalizedFormulaKind::TypePred { subject, ty } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::TypePred {
                subject: shift_term_inner(subject, cutoff, delta, diagnostic_source)?,
                ty: ty.clone(),
            }))
        }
        NormalizedFormulaKind::Not(inner) => Ok(NormalizedFormula::new(
            NormalizedFormulaKind::Not(Box::new(shift_formula_inner(
                inner,
                cutoff,
                delta,
                diagnostic_source,
            )?)),
        )),
        NormalizedFormulaKind::And(items) => shift_formula_list(
            NormalizedFormulaListKind::And,
            items,
            cutoff,
            delta,
            diagnostic_source,
        ),
        NormalizedFormulaKind::Or(items) => shift_formula_list(
            NormalizedFormulaListKind::Or,
            items,
            cutoff,
            delta,
            diagnostic_source,
        ),
        NormalizedFormulaKind::Implies {
            premise,
            conclusion,
        } => Ok(NormalizedFormula::new(NormalizedFormulaKind::Implies {
            premise: Box::new(shift_formula_inner(
                premise,
                cutoff,
                delta,
                diagnostic_source,
            )?),
            conclusion: Box::new(shift_formula_inner(
                conclusion,
                cutoff,
                delta,
                diagnostic_source,
            )?),
        })),
        NormalizedFormulaKind::Iff { left, right } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Iff {
                left: Box::new(shift_formula_inner(left, cutoff, delta, diagnostic_source)?),
                right: Box::new(shift_formula_inner(
                    right,
                    cutoff,
                    delta,
                    diagnostic_source,
                )?),
            }))
        }
        NormalizedFormulaKind::Forall { binders, body } => shift_under_formula_binders(
            NormalizedQuantifierKind::Forall,
            binders,
            body,
            cutoff,
            delta,
            diagnostic_source,
        ),
        NormalizedFormulaKind::Exists { binders, body } => shift_under_formula_binders(
            NormalizedQuantifierKind::Exists,
            binders,
            body,
            cutoff,
            delta,
            diagnostic_source,
        ),
    }
}

#[derive(Debug, Clone, Copy)]
enum NormalizedFormulaListKind {
    And,
    Or,
}

fn shift_formula_list(
    kind: NormalizedFormulaListKind,
    items: &[NormalizedFormula],
    cutoff: u32,
    delta: i32,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    let items = items
        .iter()
        .map(|item| shift_formula_inner(item, cutoff, delta, diagnostic_source))
        .collect::<BinderResult<Vec<_>>>()?;
    let kind = match kind {
        NormalizedFormulaListKind::And => NormalizedFormulaKind::And(items),
        NormalizedFormulaListKind::Or => NormalizedFormulaKind::Or(items),
    };
    Ok(NormalizedFormula::new(kind))
}

fn shift_under_formula_binders(
    quantifier: NormalizedQuantifierKind,
    binders: &[NormalizedBinderEntry],
    body: &NormalizedFormula,
    cutoff: u32,
    delta: i32,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    let binders = binders
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let guard_cutoff = close_nested_depth(cutoff, index + 1, diagnostic_source)?;
            let ty_guard = entry
                .ty_guard
                .as_ref()
                .map(|guard| {
                    shift_formula_inner(guard, guard_cutoff, delta, diagnostic_source).map(Box::new)
                })
                .transpose()?;
            Ok(NormalizedBinderEntry {
                frame: entry.frame.clone(),
                ty_guard,
            })
        })
        .collect::<BinderResult<Vec<_>>>()?;
    let nested_cutoff = close_nested_depth(cutoff, binders.len(), diagnostic_source)?;
    let body = shift_formula_inner(body, nested_cutoff, delta, diagnostic_source)?;
    let kind = match quantifier {
        NormalizedQuantifierKind::Forall => NormalizedFormulaKind::Forall {
            binders,
            body: Box::new(body),
        },
        NormalizedQuantifierKind::Exists => NormalizedFormulaKind::Exists {
            binders,
            body: Box::new(body),
        },
    };
    Ok(NormalizedFormula::new(kind))
}

fn shift_var(
    var: &NormalizedVar,
    cutoff: u32,
    delta: i32,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedVar> {
    let NormalizedVar::Bound(bound) = var else {
        return Ok(var.clone());
    };
    let index = bound.index_from_innermost;
    if index < cutoff {
        return Ok(var.clone());
    }
    let shifted = i64::from(index) + i64::from(delta);
    if shifted < 0 || shifted > i64::from(u32::MAX) {
        return Err(BinderDiagnostic::new(
            BinderDiagnosticClass::InvalidBoundIndex,
            diagnostic_source.clone(),
            "invalid-bound-shift",
        ));
    }
    Ok(NormalizedVar::Bound(BoundVar::new(shifted as u32)))
}

fn substitute_generated_origin(
    origin: &GeneratedOriginRecord,
    state: &SubstitutionState<'_>,
    depth: u32,
) -> BinderResult<GeneratedOriginRecord> {
    let params = origin
        .params
        .iter()
        .map(|param| {
            if matches!(state.substitution.target, SubstitutionTarget::TermVar(_))
                && param.free_id() == Some(state.target_var())
            {
                let replacement = state
                    .replacement_term()
                    .expect("validated term replacement for term target");
                let shifted = shift_term_inner(
                    replacement,
                    0,
                    depth_delta(depth, &state.substitution.diagnostic_source)?,
                    &state.substitution.diagnostic_source,
                )?;
                replacement_term_as_var(&shifted, &state.substitution.diagnostic_source)
            } else {
                Ok(param.clone())
            }
        })
        .collect::<BinderResult<Vec<_>>>()?;
    Ok(GeneratedOriginRecord {
        owner: origin.owner,
        kind: origin.kind,
        key: origin.key.clone(),
        params,
    })
}

fn shift_generated_origin(
    origin: &GeneratedOriginRecord,
    cutoff: u32,
    delta: i32,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<GeneratedOriginRecord> {
    let params = origin
        .params
        .iter()
        .map(|param| shift_var(param, cutoff, delta, diagnostic_source))
        .collect::<BinderResult<Vec<_>>>()?;
    Ok(GeneratedOriginRecord {
        owner: origin.owner,
        kind: origin.kind,
        key: origin.key.clone(),
        params,
    })
}

fn open_generated_origin(
    origin: &GeneratedOriginRecord,
    depth: u32,
    replacement: &NormalizedTerm,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<GeneratedOriginRecord> {
    let params = origin
        .params
        .iter()
        .map(|param| open_var_with_term(param, depth, replacement, diagnostic_source))
        .collect::<BinderResult<Vec<_>>>()?;
    Ok(GeneratedOriginRecord {
        owner: origin.owner,
        kind: origin.kind,
        key: origin.key.clone(),
        params,
    })
}

fn close_generated_origin(
    origin: &GeneratedOriginRecord,
    depth: u32,
    variable: CoreVarId,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<GeneratedOriginRecord> {
    let params = origin
        .params
        .iter()
        .map(|param| close_var(param, depth, variable, diagnostic_source))
        .collect::<BinderResult<Vec<_>>>()?;
    Ok(GeneratedOriginRecord {
        owner: origin.owner,
        kind: origin.kind,
        key: origin.key.clone(),
        params,
    })
}

fn subst_bound_generated_origin(
    origin: &GeneratedOriginRecord,
    depth: u32,
    replacement: &NormalizedTerm,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<GeneratedOriginRecord> {
    let params = origin
        .params
        .iter()
        .map(|param| subst_bound_var(param, depth, replacement, diagnostic_source))
        .collect::<BinderResult<Vec<_>>>()?;
    Ok(GeneratedOriginRecord {
        owner: origin.owner,
        kind: origin.kind,
        key: origin.key.clone(),
        params,
    })
}

fn open_var_with_term(
    var: &NormalizedVar,
    depth: u32,
    replacement: &NormalizedTerm,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedVar> {
    match var {
        NormalizedVar::Bound(bound) if bound.index_from_innermost == depth => {
            let shifted = shift_term_inner(
                replacement,
                0,
                depth_delta(depth, diagnostic_source)?,
                diagnostic_source,
            )?;
            replacement_term_as_var(&shifted, diagnostic_source)
        }
        NormalizedVar::Bound(bound) if bound.index_from_innermost > depth => Ok(
            NormalizedVar::Bound(BoundVar::new(bound.index_from_innermost - 1)),
        ),
        _ => Ok(var.clone()),
    }
}

fn close_var(
    var: &NormalizedVar,
    depth: u32,
    variable: CoreVarId,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedVar> {
    match var {
        NormalizedVar::Free(id) if *id == variable => {
            Ok(NormalizedVar::Bound(BoundVar::new(depth)))
        }
        NormalizedVar::Bound(bound) if bound.index_from_innermost >= depth => {
            let shifted = bound.index_from_innermost.checked_add(1).ok_or_else(|| {
                BinderDiagnostic::new(
                    BinderDiagnosticClass::InvalidBoundIndex,
                    diagnostic_source.clone(),
                    "invalid-bound-close",
                )
            })?;
            Ok(NormalizedVar::Bound(BoundVar::new(shifted)))
        }
        _ => Ok(var.clone()),
    }
}

fn subst_bound_var(
    var: &NormalizedVar,
    depth: u32,
    replacement: &NormalizedTerm,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedVar> {
    match var {
        NormalizedVar::Bound(bound) if bound.index_from_innermost == depth => {
            let shifted = shift_term_inner(
                replacement,
                0,
                depth_delta(depth, diagnostic_source)?,
                diagnostic_source,
            )?;
            replacement_term_as_var(&shifted, diagnostic_source)
        }
        _ => Ok(var.clone()),
    }
}

fn replacement_term_as_var(
    term: &NormalizedTerm,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedVar> {
    let NormalizedTermKind::Var(var) = &term.kind else {
        return Err(BinderDiagnostic::new(
            BinderDiagnosticClass::MalformedEvidence,
            diagnostic_source.clone(),
            "generated-origin-param-non-var-replacement",
        ));
    };
    Ok(var.clone())
}

fn open_rec_term_inner(
    term: &NormalizedTerm,
    depth: u32,
    replacement: &NormalizedTerm,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedTerm> {
    match &term.kind {
        NormalizedTermKind::Var(NormalizedVar::Bound(bound))
            if bound.index_from_innermost == depth =>
        {
            shift_term_inner(
                replacement,
                0,
                depth_delta(depth, diagnostic_source)?,
                diagnostic_source,
            )
        }
        NormalizedTermKind::Var(NormalizedVar::Bound(bound))
            if bound.index_from_innermost > depth =>
        {
            Ok(NormalizedTerm::bound(bound.index_from_innermost - 1))
        }
        NormalizedTermKind::Var(_)
        | NormalizedTermKind::Const(_)
        | NormalizedTermKind::Error(_) => Ok(term.clone()),
        NormalizedTermKind::Select { selector, base } => {
            Ok(NormalizedTerm::new(NormalizedTermKind::Select {
                selector: selector.clone(),
                base: Box::new(open_rec_term_inner(
                    base,
                    depth,
                    replacement,
                    diagnostic_source,
                )?),
            }))
        }
        NormalizedTermKind::Apply { functor, args } => {
            let args = args
                .iter()
                .map(|arg| open_rec_term_inner(arg, depth, replacement, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedTerm::new(NormalizedTermKind::Apply {
                functor: functor.clone(),
                args,
            }))
        }
        NormalizedTermKind::Tuple(items) => {
            let items = items
                .iter()
                .map(|item| open_rec_term_inner(item, depth, replacement, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedTerm::new(NormalizedTermKind::Tuple(items)))
        }
        NormalizedTermKind::SetEnum(items) => {
            let items = items
                .iter()
                .map(|item| open_rec_term_inner(item, depth, replacement, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedTerm::new(NormalizedTermKind::SetEnum(items)))
        }
        NormalizedTermKind::Generated { origin, args } => {
            let origin = open_generated_origin(origin, depth, replacement, diagnostic_source)?;
            let args = args
                .iter()
                .map(|arg| open_rec_term_inner(arg, depth, replacement, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedTerm::new(NormalizedTermKind::Generated {
                origin,
                args,
            }))
        }
    }
}

fn open_rec_formula_with_term_inner(
    formula: &NormalizedFormula,
    depth: u32,
    replacement: &NormalizedTerm,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    match &formula.kind {
        NormalizedFormulaKind::Var(_)
        | NormalizedFormulaKind::True
        | NormalizedFormulaKind::False
        | NormalizedFormulaKind::Error(_) => Ok(formula.clone()),
        NormalizedFormulaKind::Atom { predicate, args } => {
            let args = args
                .iter()
                .map(|arg| open_rec_term_inner(arg, depth, replacement, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Atom {
                predicate: predicate.clone(),
                args,
            }))
        }
        NormalizedFormulaKind::Equals { left, right } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Equals {
                left: open_rec_term_inner(left, depth, replacement, diagnostic_source)?,
                right: open_rec_term_inner(right, depth, replacement, diagnostic_source)?,
            }))
        }
        NormalizedFormulaKind::TypePred { subject, ty } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::TypePred {
                subject: open_rec_term_inner(subject, depth, replacement, diagnostic_source)?,
                ty: ty.clone(),
            }))
        }
        NormalizedFormulaKind::Not(inner) => Ok(NormalizedFormula::new(
            NormalizedFormulaKind::Not(Box::new(open_rec_formula_with_term_inner(
                inner,
                depth,
                replacement,
                diagnostic_source,
            )?)),
        )),
        NormalizedFormulaKind::And(items) => open_formula_list_with_term(
            NormalizedFormulaListKind::And,
            items,
            depth,
            replacement,
            diagnostic_source,
        ),
        NormalizedFormulaKind::Or(items) => open_formula_list_with_term(
            NormalizedFormulaListKind::Or,
            items,
            depth,
            replacement,
            diagnostic_source,
        ),
        NormalizedFormulaKind::Implies {
            premise,
            conclusion,
        } => Ok(NormalizedFormula::new(NormalizedFormulaKind::Implies {
            premise: Box::new(open_rec_formula_with_term_inner(
                premise,
                depth,
                replacement,
                diagnostic_source,
            )?),
            conclusion: Box::new(open_rec_formula_with_term_inner(
                conclusion,
                depth,
                replacement,
                diagnostic_source,
            )?),
        })),
        NormalizedFormulaKind::Iff { left, right } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Iff {
                left: Box::new(open_rec_formula_with_term_inner(
                    left,
                    depth,
                    replacement,
                    diagnostic_source,
                )?),
                right: Box::new(open_rec_formula_with_term_inner(
                    right,
                    depth,
                    replacement,
                    diagnostic_source,
                )?),
            }))
        }
        NormalizedFormulaKind::Forall { binders, body } => open_under_formula_binders(
            NormalizedQuantifierKind::Forall,
            binders,
            body,
            depth,
            replacement,
            diagnostic_source,
        ),
        NormalizedFormulaKind::Exists { binders, body } => open_under_formula_binders(
            NormalizedQuantifierKind::Exists,
            binders,
            body,
            depth,
            replacement,
            diagnostic_source,
        ),
    }
}

fn open_formula_list_with_term(
    kind: NormalizedFormulaListKind,
    items: &[NormalizedFormula],
    depth: u32,
    replacement: &NormalizedTerm,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    let items = items
        .iter()
        .map(|item| open_rec_formula_with_term_inner(item, depth, replacement, diagnostic_source))
        .collect::<BinderResult<Vec<_>>>()?;
    let kind = match kind {
        NormalizedFormulaListKind::And => NormalizedFormulaKind::And(items),
        NormalizedFormulaListKind::Or => NormalizedFormulaKind::Or(items),
    };
    Ok(NormalizedFormula::new(kind))
}

fn open_under_formula_binders(
    quantifier: NormalizedQuantifierKind,
    binders: &[NormalizedBinderEntry],
    body: &NormalizedFormula,
    depth: u32,
    replacement: &NormalizedTerm,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    let binders = binders
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let guard_depth = close_nested_depth(depth, index + 1, diagnostic_source)?;
            let ty_guard = entry
                .ty_guard
                .as_ref()
                .map(|guard| {
                    open_rec_formula_with_term_inner(
                        guard,
                        guard_depth,
                        replacement,
                        diagnostic_source,
                    )
                    .map(Box::new)
                })
                .transpose()?;
            Ok(NormalizedBinderEntry {
                frame: entry.frame.clone(),
                ty_guard,
            })
        })
        .collect::<BinderResult<Vec<_>>>()?;
    let nested_depth = close_nested_depth(depth, binders.len(), diagnostic_source)?;
    let body =
        open_rec_formula_with_term_inner(body, nested_depth, replacement, diagnostic_source)?;
    let kind = match quantifier {
        NormalizedQuantifierKind::Forall => NormalizedFormulaKind::Forall {
            binders,
            body: Box::new(body),
        },
        NormalizedQuantifierKind::Exists => NormalizedFormulaKind::Exists {
            binders,
            body: Box::new(body),
        },
    };
    Ok(NormalizedFormula::new(kind))
}

fn open_rec_formula_with_term_capture_avoiding(
    formula: &NormalizedFormula,
    depth: u32,
    replacement: &NormalizedTerm,
    state: &mut ClosureExpansionState,
    path: &NormalizedTermOrFormulaPath,
) -> BinderResult<NormalizedFormula> {
    match &formula.kind {
        NormalizedFormulaKind::Var(_)
        | NormalizedFormulaKind::True
        | NormalizedFormulaKind::False
        | NormalizedFormulaKind::Error(_) => Ok(formula.clone()),
        NormalizedFormulaKind::Atom { predicate, args } => {
            let args = args
                .iter()
                .map(|arg| open_rec_term_inner(arg, depth, replacement, &state.diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Atom {
                predicate: predicate.clone(),
                args,
            }))
        }
        NormalizedFormulaKind::Equals { left, right } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Equals {
                left: open_rec_term_inner(left, depth, replacement, &state.diagnostic_source)?,
                right: open_rec_term_inner(right, depth, replacement, &state.diagnostic_source)?,
            }))
        }
        NormalizedFormulaKind::TypePred { subject, ty } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::TypePred {
                subject: open_rec_term_inner(
                    subject,
                    depth,
                    replacement,
                    &state.diagnostic_source,
                )?,
                ty: ty.clone(),
            }))
        }
        NormalizedFormulaKind::Not(inner) => Ok(NormalizedFormula::new(
            NormalizedFormulaKind::Not(Box::new(open_rec_formula_with_term_capture_avoiding(
                inner,
                depth,
                replacement,
                state,
                path,
            )?)),
        )),
        NormalizedFormulaKind::And(items) => open_formula_list_with_term_capture_avoiding(
            NormalizedFormulaListKind::And,
            items,
            depth,
            replacement,
            state,
            path,
        ),
        NormalizedFormulaKind::Or(items) => open_formula_list_with_term_capture_avoiding(
            NormalizedFormulaListKind::Or,
            items,
            depth,
            replacement,
            state,
            path,
        ),
        NormalizedFormulaKind::Implies {
            premise,
            conclusion,
        } => Ok(NormalizedFormula::new(NormalizedFormulaKind::Implies {
            premise: Box::new(open_rec_formula_with_term_capture_avoiding(
                premise,
                depth,
                replacement,
                state,
                path,
            )?),
            conclusion: Box::new(open_rec_formula_with_term_capture_avoiding(
                conclusion,
                depth,
                replacement,
                state,
                path,
            )?),
        })),
        NormalizedFormulaKind::Iff { left, right } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Iff {
                left: Box::new(open_rec_formula_with_term_capture_avoiding(
                    left,
                    depth,
                    replacement,
                    state,
                    path,
                )?),
                right: Box::new(open_rec_formula_with_term_capture_avoiding(
                    right,
                    depth,
                    replacement,
                    state,
                    path,
                )?),
            }))
        }
        NormalizedFormulaKind::Forall { binders, body } => {
            open_capture_avoiding_under_formula_binders(
                NormalizedQuantifierKind::Forall,
                binders,
                body,
                depth,
                replacement,
                state,
                path,
            )
        }
        NormalizedFormulaKind::Exists { binders, body } => {
            open_capture_avoiding_under_formula_binders(
                NormalizedQuantifierKind::Exists,
                binders,
                body,
                depth,
                replacement,
                state,
                path,
            )
        }
    }
}

fn open_formula_list_with_term_capture_avoiding(
    kind: NormalizedFormulaListKind,
    items: &[NormalizedFormula],
    depth: u32,
    replacement: &NormalizedTerm,
    state: &mut ClosureExpansionState,
    path: &NormalizedTermOrFormulaPath,
) -> BinderResult<NormalizedFormula> {
    let items = items
        .iter()
        .map(|item| {
            open_rec_formula_with_term_capture_avoiding(item, depth, replacement, state, path)
        })
        .collect::<BinderResult<Vec<_>>>()?;
    let kind = match kind {
        NormalizedFormulaListKind::And => NormalizedFormulaKind::And(items),
        NormalizedFormulaListKind::Or => NormalizedFormulaKind::Or(items),
    };
    Ok(NormalizedFormula::new(kind))
}

fn open_capture_avoiding_under_formula_binders(
    quantifier: NormalizedQuantifierKind,
    binders: &[NormalizedBinderEntry],
    body: &NormalizedFormula,
    depth: u32,
    replacement: &NormalizedTerm,
    state: &mut ClosureExpansionState,
    path: &NormalizedTermOrFormulaPath,
) -> BinderResult<NormalizedFormula> {
    let mut binders = binders.to_vec();
    state
        .used_variables
        .extend(body.free_variables.iter().copied());
    state
        .used_variables
        .extend(binders.iter().map(|entry| entry.frame.original_var));
    for entry in &binders {
        if let Some(guard) = &entry.ty_guard {
            state
                .used_variables
                .extend(guard.free_variables.iter().copied());
        }
    }
    for (index, entry) in binders.iter_mut().enumerate() {
        if replacement
            .free_variables
            .contains(&entry.frame.original_var)
        {
            let frame_path = path.child(entry.frame.canonical_index);
            state.freshen_frame(&mut entry.frame, &frame_path)?;
        }
        if let Some(guard) = &entry.ty_guard {
            let guard_depth = close_nested_depth_with_state(depth, index + 1, state)?;
            let guard_path = path.child(entry.frame.canonical_index);
            entry.ty_guard = Some(Box::new(open_rec_formula_with_term_capture_avoiding(
                guard,
                guard_depth,
                replacement,
                state,
                &guard_path,
            )?));
        }
    }
    let nested_depth = close_nested_depth_with_state(depth, binders.len(), state)?;
    let nested_path = binders.iter().fold(path.clone(), |path, entry| {
        path.child(entry.frame.canonical_index)
    });
    let body = open_rec_formula_with_term_capture_avoiding(
        body,
        nested_depth,
        replacement,
        state,
        &nested_path,
    )?;
    let kind = match quantifier {
        NormalizedQuantifierKind::Forall => NormalizedFormulaKind::Forall {
            binders,
            body: Box::new(body),
        },
        NormalizedQuantifierKind::Exists => NormalizedFormulaKind::Exists {
            binders,
            body: Box::new(body),
        },
    };
    Ok(NormalizedFormula::new(kind))
}

fn open_rec_formula_with_formula_inner(
    formula: &NormalizedFormula,
    depth: u32,
    replacement: &NormalizedFormula,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    match &formula.kind {
        NormalizedFormulaKind::Var(NormalizedVar::Bound(bound))
            if bound.index_from_innermost == depth =>
        {
            shift_formula_inner(
                replacement,
                0,
                depth_delta(depth, diagnostic_source)?,
                diagnostic_source,
            )
        }
        NormalizedFormulaKind::Var(NormalizedVar::Bound(bound))
            if bound.index_from_innermost > depth =>
        {
            Ok(NormalizedFormula::var(NormalizedVar::Bound(BoundVar::new(
                bound.index_from_innermost - 1,
            ))))
        }
        NormalizedFormulaKind::Var(_)
        | NormalizedFormulaKind::True
        | NormalizedFormulaKind::False
        | NormalizedFormulaKind::Error(_)
        | NormalizedFormulaKind::Atom { .. }
        | NormalizedFormulaKind::Equals { .. }
        | NormalizedFormulaKind::TypePred { .. } => Ok(formula.clone()),
        NormalizedFormulaKind::Not(inner) => Ok(NormalizedFormula::new(
            NormalizedFormulaKind::Not(Box::new(open_rec_formula_with_formula_inner(
                inner,
                depth,
                replacement,
                diagnostic_source,
            )?)),
        )),
        NormalizedFormulaKind::And(items) => open_formula_list_with_formula(
            NormalizedFormulaListKind::And,
            items,
            depth,
            replacement,
            diagnostic_source,
        ),
        NormalizedFormulaKind::Or(items) => open_formula_list_with_formula(
            NormalizedFormulaListKind::Or,
            items,
            depth,
            replacement,
            diagnostic_source,
        ),
        NormalizedFormulaKind::Implies {
            premise,
            conclusion,
        } => Ok(NormalizedFormula::new(NormalizedFormulaKind::Implies {
            premise: Box::new(open_rec_formula_with_formula_inner(
                premise,
                depth,
                replacement,
                diagnostic_source,
            )?),
            conclusion: Box::new(open_rec_formula_with_formula_inner(
                conclusion,
                depth,
                replacement,
                diagnostic_source,
            )?),
        })),
        NormalizedFormulaKind::Iff { left, right } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Iff {
                left: Box::new(open_rec_formula_with_formula_inner(
                    left,
                    depth,
                    replacement,
                    diagnostic_source,
                )?),
                right: Box::new(open_rec_formula_with_formula_inner(
                    right,
                    depth,
                    replacement,
                    diagnostic_source,
                )?),
            }))
        }
        NormalizedFormulaKind::Forall { binders, body } => open_formula_under_formula_binders(
            NormalizedQuantifierKind::Forall,
            binders,
            body,
            depth,
            replacement,
            diagnostic_source,
        ),
        NormalizedFormulaKind::Exists { binders, body } => open_formula_under_formula_binders(
            NormalizedQuantifierKind::Exists,
            binders,
            body,
            depth,
            replacement,
            diagnostic_source,
        ),
    }
}

fn open_formula_list_with_formula(
    kind: NormalizedFormulaListKind,
    items: &[NormalizedFormula],
    depth: u32,
    replacement: &NormalizedFormula,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    let items = items
        .iter()
        .map(|item| {
            open_rec_formula_with_formula_inner(item, depth, replacement, diagnostic_source)
        })
        .collect::<BinderResult<Vec<_>>>()?;
    let kind = match kind {
        NormalizedFormulaListKind::And => NormalizedFormulaKind::And(items),
        NormalizedFormulaListKind::Or => NormalizedFormulaKind::Or(items),
    };
    Ok(NormalizedFormula::new(kind))
}

fn open_formula_under_formula_binders(
    quantifier: NormalizedQuantifierKind,
    binders: &[NormalizedBinderEntry],
    body: &NormalizedFormula,
    depth: u32,
    replacement: &NormalizedFormula,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    let binders = binders
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let guard_depth = close_nested_depth(depth, index + 1, diagnostic_source)?;
            let ty_guard = entry
                .ty_guard
                .as_ref()
                .map(|guard| {
                    open_rec_formula_with_formula_inner(
                        guard,
                        guard_depth,
                        replacement,
                        diagnostic_source,
                    )
                    .map(Box::new)
                })
                .transpose()?;
            Ok(NormalizedBinderEntry {
                frame: entry.frame.clone(),
                ty_guard,
            })
        })
        .collect::<BinderResult<Vec<_>>>()?;
    let nested_depth = close_nested_depth(depth, binders.len(), diagnostic_source)?;
    let body =
        open_rec_formula_with_formula_inner(body, nested_depth, replacement, diagnostic_source)?;
    let kind = match quantifier {
        NormalizedQuantifierKind::Forall => NormalizedFormulaKind::Forall {
            binders,
            body: Box::new(body),
        },
        NormalizedQuantifierKind::Exists => NormalizedFormulaKind::Exists {
            binders,
            body: Box::new(body),
        },
    };
    Ok(NormalizedFormula::new(kind))
}

fn close_rec_term_inner(
    term: &NormalizedTerm,
    depth: u32,
    variable: CoreVarId,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedTerm> {
    match &term.kind {
        NormalizedTermKind::Var(NormalizedVar::Free(var)) if *var == variable => {
            Ok(NormalizedTerm::bound(depth))
        }
        NormalizedTermKind::Var(NormalizedVar::Bound(bound))
            if bound.index_from_innermost >= depth =>
        {
            let shifted = bound.index_from_innermost.checked_add(1).ok_or_else(|| {
                BinderDiagnostic::new(
                    BinderDiagnosticClass::InvalidBoundIndex,
                    diagnostic_source.clone(),
                    "invalid-bound-close",
                )
            })?;
            Ok(NormalizedTerm::bound(shifted))
        }
        NormalizedTermKind::Var(_)
        | NormalizedTermKind::Const(_)
        | NormalizedTermKind::Error(_) => Ok(term.clone()),
        NormalizedTermKind::Select { selector, base } => {
            Ok(NormalizedTerm::new(NormalizedTermKind::Select {
                selector: selector.clone(),
                base: Box::new(close_rec_term_inner(
                    base,
                    depth,
                    variable,
                    diagnostic_source,
                )?),
            }))
        }
        NormalizedTermKind::Apply { functor, args } => {
            let args = args
                .iter()
                .map(|arg| close_rec_term_inner(arg, depth, variable, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedTerm::new(NormalizedTermKind::Apply {
                functor: functor.clone(),
                args,
            }))
        }
        NormalizedTermKind::Tuple(items) => {
            let items = items
                .iter()
                .map(|item| close_rec_term_inner(item, depth, variable, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedTerm::new(NormalizedTermKind::Tuple(items)))
        }
        NormalizedTermKind::SetEnum(items) => {
            let items = items
                .iter()
                .map(|item| close_rec_term_inner(item, depth, variable, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedTerm::new(NormalizedTermKind::SetEnum(items)))
        }
        NormalizedTermKind::Generated { origin, args } => {
            let origin = close_generated_origin(origin, depth, variable, diagnostic_source)?;
            let args = args
                .iter()
                .map(|arg| close_rec_term_inner(arg, depth, variable, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedTerm::new(NormalizedTermKind::Generated {
                origin,
                args,
            }))
        }
    }
}

fn close_rec_formula_inner(
    formula: &NormalizedFormula,
    depth: u32,
    variable: CoreVarId,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    match &formula.kind {
        NormalizedFormulaKind::Var(NormalizedVar::Free(var)) if *var == variable => Ok(
            NormalizedFormula::var(NormalizedVar::Bound(BoundVar::new(depth))),
        ),
        NormalizedFormulaKind::Var(NormalizedVar::Bound(bound))
            if bound.index_from_innermost >= depth =>
        {
            let shifted = bound.index_from_innermost.checked_add(1).ok_or_else(|| {
                BinderDiagnostic::new(
                    BinderDiagnosticClass::InvalidBoundIndex,
                    diagnostic_source.clone(),
                    "invalid-bound-close",
                )
            })?;
            Ok(NormalizedFormula::var(NormalizedVar::Bound(BoundVar::new(
                shifted,
            ))))
        }
        NormalizedFormulaKind::Var(_)
        | NormalizedFormulaKind::True
        | NormalizedFormulaKind::False
        | NormalizedFormulaKind::Error(_) => Ok(formula.clone()),
        NormalizedFormulaKind::Atom { predicate, args } => {
            let args = args
                .iter()
                .map(|arg| close_rec_term_inner(arg, depth, variable, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Atom {
                predicate: predicate.clone(),
                args,
            }))
        }
        NormalizedFormulaKind::Equals { left, right } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Equals {
                left: close_rec_term_inner(left, depth, variable, diagnostic_source)?,
                right: close_rec_term_inner(right, depth, variable, diagnostic_source)?,
            }))
        }
        NormalizedFormulaKind::TypePred { subject, ty } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::TypePred {
                subject: close_rec_term_inner(subject, depth, variable, diagnostic_source)?,
                ty: ty.clone(),
            }))
        }
        NormalizedFormulaKind::Not(inner) => Ok(NormalizedFormula::new(
            NormalizedFormulaKind::Not(Box::new(close_rec_formula_inner(
                inner,
                depth,
                variable,
                diagnostic_source,
            )?)),
        )),
        NormalizedFormulaKind::And(items) => {
            let items = items
                .iter()
                .map(|item| close_rec_formula_inner(item, depth, variable, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedFormula::new(NormalizedFormulaKind::And(items)))
        }
        NormalizedFormulaKind::Or(items) => {
            let items = items
                .iter()
                .map(|item| close_rec_formula_inner(item, depth, variable, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Or(items)))
        }
        NormalizedFormulaKind::Implies {
            premise,
            conclusion,
        } => Ok(NormalizedFormula::new(NormalizedFormulaKind::Implies {
            premise: Box::new(close_rec_formula_inner(
                premise,
                depth,
                variable,
                diagnostic_source,
            )?),
            conclusion: Box::new(close_rec_formula_inner(
                conclusion,
                depth,
                variable,
                diagnostic_source,
            )?),
        })),
        NormalizedFormulaKind::Iff { left, right } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Iff {
                left: Box::new(close_rec_formula_inner(
                    left,
                    depth,
                    variable,
                    diagnostic_source,
                )?),
                right: Box::new(close_rec_formula_inner(
                    right,
                    depth,
                    variable,
                    diagnostic_source,
                )?),
            }))
        }
        NormalizedFormulaKind::Forall { binders, body } => {
            let binders = binders
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    let guard_depth = close_nested_depth(depth, index + 1, diagnostic_source)?;
                    let ty_guard = entry
                        .ty_guard
                        .as_ref()
                        .map(|guard| {
                            close_rec_formula_inner(guard, guard_depth, variable, diagnostic_source)
                                .map(Box::new)
                        })
                        .transpose()?;
                    Ok(NormalizedBinderEntry {
                        frame: entry.frame.clone(),
                        ty_guard,
                    })
                })
                .collect::<BinderResult<Vec<_>>>()?;
            let nested_depth = close_nested_depth(depth, binders.len(), diagnostic_source)?;
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Forall {
                binders,
                body: Box::new(close_rec_formula_inner(
                    body,
                    nested_depth,
                    variable,
                    diagnostic_source,
                )?),
            }))
        }
        NormalizedFormulaKind::Exists { binders, body } => {
            let binders = binders
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    let guard_depth = close_nested_depth(depth, index + 1, diagnostic_source)?;
                    let ty_guard = entry
                        .ty_guard
                        .as_ref()
                        .map(|guard| {
                            close_rec_formula_inner(guard, guard_depth, variable, diagnostic_source)
                                .map(Box::new)
                        })
                        .transpose()?;
                    Ok(NormalizedBinderEntry {
                        frame: entry.frame.clone(),
                        ty_guard,
                    })
                })
                .collect::<BinderResult<Vec<_>>>()?;
            let nested_depth = close_nested_depth(depth, binders.len(), diagnostic_source)?;
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Exists {
                binders,
                body: Box::new(close_rec_formula_inner(
                    body,
                    nested_depth,
                    variable,
                    diagnostic_source,
                )?),
            }))
        }
    }
}

fn close_nested_depth(
    depth: u32,
    binders: usize,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<u32> {
    depth
        .checked_add(u32::try_from(binders).map_err(|_| {
            BinderDiagnostic::new(
                BinderDiagnosticClass::InvalidBoundIndex,
                diagnostic_source.clone(),
                "binder-depth-overflow",
            )
        })?)
        .ok_or_else(|| {
            BinderDiagnostic::new(
                BinderDiagnosticClass::InvalidBoundIndex,
                diagnostic_source.clone(),
                "binder-depth-overflow",
            )
        })
}

fn context_depth(context: &BinderContext, diagnostic_source: &CoreSourceRef) -> BinderResult<u32> {
    u32::try_from(context.frames.len()).map_err(|_| {
        BinderDiagnostic::new(
            BinderDiagnosticClass::InvalidBoundIndex,
            diagnostic_source.clone(),
            "context-depth-overflow",
        )
    })
}

fn close_nested_depth_with_state(
    depth: u32,
    binders: usize,
    state: &ClosureExpansionState,
) -> BinderResult<u32> {
    depth
        .checked_add(u32::try_from(binders).map_err(|_| {
            state.diagnostic(
                BinderDiagnosticClass::InvalidBoundIndex,
                "binder-depth-overflow",
            )
        })?)
        .ok_or_else(|| {
            state.diagnostic(
                BinderDiagnosticClass::InvalidBoundIndex,
                "binder-depth-overflow",
            )
        })
}

fn subst_bound_term_inner(
    term: &NormalizedTerm,
    depth: u32,
    replacement: &NormalizedTerm,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedTerm> {
    match &term.kind {
        NormalizedTermKind::Var(NormalizedVar::Bound(bound))
            if bound.index_from_innermost == depth =>
        {
            shift_term_inner(
                replacement,
                0,
                depth_delta(depth, diagnostic_source)?,
                diagnostic_source,
            )
        }
        NormalizedTermKind::Var(_)
        | NormalizedTermKind::Const(_)
        | NormalizedTermKind::Error(_) => Ok(term.clone()),
        NormalizedTermKind::Select { selector, base } => {
            Ok(NormalizedTerm::new(NormalizedTermKind::Select {
                selector: selector.clone(),
                base: Box::new(subst_bound_term_inner(
                    base,
                    depth,
                    replacement,
                    diagnostic_source,
                )?),
            }))
        }
        NormalizedTermKind::Apply { functor, args } => {
            let args = args
                .iter()
                .map(|arg| subst_bound_term_inner(arg, depth, replacement, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedTerm::new(NormalizedTermKind::Apply {
                functor: functor.clone(),
                args,
            }))
        }
        NormalizedTermKind::Tuple(items) => {
            let items = items
                .iter()
                .map(|item| subst_bound_term_inner(item, depth, replacement, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedTerm::new(NormalizedTermKind::Tuple(items)))
        }
        NormalizedTermKind::SetEnum(items) => {
            let items = items
                .iter()
                .map(|item| subst_bound_term_inner(item, depth, replacement, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedTerm::new(NormalizedTermKind::SetEnum(items)))
        }
        NormalizedTermKind::Generated { origin, args } => {
            let origin =
                subst_bound_generated_origin(origin, depth, replacement, diagnostic_source)?;
            let args = args
                .iter()
                .map(|arg| subst_bound_term_inner(arg, depth, replacement, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedTerm::new(NormalizedTermKind::Generated {
                origin,
                args,
            }))
        }
    }
}

fn subst_bound_term_in_formula(
    term: &NormalizedTerm,
    depth: u32,
    replacement: &SubstitutionReplacement,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedTerm> {
    match replacement {
        SubstitutionReplacement::Term(replacement) => {
            subst_bound_term_inner(term, depth, replacement, diagnostic_source)
        }
        SubstitutionReplacement::Formula(_) => Ok(term.clone()),
    }
}

fn subst_bound_terms_in_formula(
    terms: &[NormalizedTerm],
    depth: u32,
    replacement: &SubstitutionReplacement,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<Vec<NormalizedTerm>> {
    terms
        .iter()
        .map(|term| subst_bound_term_in_formula(term, depth, replacement, diagnostic_source))
        .collect()
}

fn subst_bound_formula_inner(
    formula: &NormalizedFormula,
    depth: u32,
    replacement: &SubstitutionReplacement,
    diagnostic_source: &CoreSourceRef,
) -> BinderResult<NormalizedFormula> {
    match &formula.kind {
        NormalizedFormulaKind::Var(NormalizedVar::Bound(bound))
            if bound.index_from_innermost == depth =>
        {
            let SubstitutionReplacement::Formula(replacement) = replacement else {
                return Err(BinderDiagnostic::new(
                    BinderDiagnosticClass::SortMismatch,
                    diagnostic_source.clone(),
                    "formula-bound-sort-mismatch",
                ));
            };
            shift_formula_inner(
                replacement,
                0,
                depth_delta(depth, diagnostic_source)?,
                diagnostic_source,
            )
        }
        NormalizedFormulaKind::Var(_)
        | NormalizedFormulaKind::True
        | NormalizedFormulaKind::False
        | NormalizedFormulaKind::Error(_) => Ok(formula.clone()),
        NormalizedFormulaKind::Atom { predicate, args } => {
            let args = subst_bound_terms_in_formula(args, depth, replacement, diagnostic_source)?;
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Atom {
                predicate: predicate.clone(),
                args,
            }))
        }
        NormalizedFormulaKind::Equals { left, right } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Equals {
                left: subst_bound_term_in_formula(left, depth, replacement, diagnostic_source)?,
                right: subst_bound_term_in_formula(right, depth, replacement, diagnostic_source)?,
            }))
        }
        NormalizedFormulaKind::TypePred { subject, ty } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::TypePred {
                subject: subst_bound_term_in_formula(
                    subject,
                    depth,
                    replacement,
                    diagnostic_source,
                )?,
                ty: ty.clone(),
            }))
        }
        NormalizedFormulaKind::Not(inner) => Ok(NormalizedFormula::new(
            NormalizedFormulaKind::Not(Box::new(subst_bound_formula_inner(
                inner,
                depth,
                replacement,
                diagnostic_source,
            )?)),
        )),
        NormalizedFormulaKind::And(items) => {
            let items = items
                .iter()
                .map(|item| subst_bound_formula_inner(item, depth, replacement, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedFormula::new(NormalizedFormulaKind::And(items)))
        }
        NormalizedFormulaKind::Or(items) => {
            let items = items
                .iter()
                .map(|item| subst_bound_formula_inner(item, depth, replacement, diagnostic_source))
                .collect::<BinderResult<Vec<_>>>()?;
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Or(items)))
        }
        NormalizedFormulaKind::Implies {
            premise,
            conclusion,
        } => Ok(NormalizedFormula::new(NormalizedFormulaKind::Implies {
            premise: Box::new(subst_bound_formula_inner(
                premise,
                depth,
                replacement,
                diagnostic_source,
            )?),
            conclusion: Box::new(subst_bound_formula_inner(
                conclusion,
                depth,
                replacement,
                diagnostic_source,
            )?),
        })),
        NormalizedFormulaKind::Iff { left, right } => {
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Iff {
                left: Box::new(subst_bound_formula_inner(
                    left,
                    depth,
                    replacement,
                    diagnostic_source,
                )?),
                right: Box::new(subst_bound_formula_inner(
                    right,
                    depth,
                    replacement,
                    diagnostic_source,
                )?),
            }))
        }
        NormalizedFormulaKind::Forall { binders, body } => {
            let binders = binders
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    let guard_depth = close_nested_depth(depth, index + 1, diagnostic_source)?;
                    let ty_guard = entry
                        .ty_guard
                        .as_ref()
                        .map(|guard| {
                            subst_bound_formula_inner(
                                guard,
                                guard_depth,
                                replacement,
                                diagnostic_source,
                            )
                            .map(Box::new)
                        })
                        .transpose()?;
                    Ok(NormalizedBinderEntry {
                        frame: entry.frame.clone(),
                        ty_guard,
                    })
                })
                .collect::<BinderResult<Vec<_>>>()?;
            let nested_depth = close_nested_depth(depth, binders.len(), diagnostic_source)?;
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Forall {
                binders,
                body: Box::new(subst_bound_formula_inner(
                    body,
                    nested_depth,
                    replacement,
                    diagnostic_source,
                )?),
            }))
        }
        NormalizedFormulaKind::Exists { binders, body } => {
            let binders = binders
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    let guard_depth = close_nested_depth(depth, index + 1, diagnostic_source)?;
                    let ty_guard = entry
                        .ty_guard
                        .as_ref()
                        .map(|guard| {
                            subst_bound_formula_inner(
                                guard,
                                guard_depth,
                                replacement,
                                diagnostic_source,
                            )
                            .map(Box::new)
                        })
                        .transpose()?;
                    Ok(NormalizedBinderEntry {
                        frame: entry.frame.clone(),
                        ty_guard,
                    })
                })
                .collect::<BinderResult<Vec<_>>>()?;
            let nested_depth = close_nested_depth(depth, binders.len(), diagnostic_source)?;
            Ok(NormalizedFormula::new(NormalizedFormulaKind::Exists {
                binders,
                body: Box::new(subst_bound_formula_inner(
                    body,
                    nested_depth,
                    replacement,
                    diagnostic_source,
                )?),
            }))
        }
    }
}

fn instantiate_term_formals_capture_avoiding(
    mut body: NormalizedTermOrFormula,
    actuals: &[NormalizedTerm],
    state: &mut ClosureExpansionState,
) -> BinderResult<NormalizedTermOrFormula> {
    for actual in actuals.iter().rev() {
        body = match body {
            NormalizedTermOrFormula::Term(term) => NormalizedTermOrFormula::Term(
                open_rec_term_inner(&term, 0, actual, &state.diagnostic_source)?,
            ),
            NormalizedTermOrFormula::Formula(formula) => {
                NormalizedTermOrFormula::Formula(open_rec_formula_with_term_capture_avoiding(
                    &formula,
                    0,
                    actual,
                    state,
                    &state.freshness.binder_path.clone(),
                )?)
            }
        };
    }
    Ok(body)
}

fn instantiate_formula_term_formals_capture_avoiding(
    mut formula: NormalizedFormula,
    actuals: &[NormalizedTerm],
    state: &mut ClosureExpansionState,
) -> BinderResult<NormalizedFormula> {
    for actual in actuals.iter().rev() {
        formula = open_rec_formula_with_term_capture_avoiding(
            &formula,
            0,
            actual,
            state,
            &state.freshness.binder_path.clone(),
        )?;
    }
    Ok(formula)
}

fn term_free_variables(kind: &NormalizedTermKind) -> BTreeSet<CoreVarId> {
    let mut free = BTreeSet::new();
    match kind {
        NormalizedTermKind::Var(var) => {
            if let Some(id) = var.free_id() {
                free.insert(id);
            }
        }
        NormalizedTermKind::Const(_) | NormalizedTermKind::Error(_) => {}
        NormalizedTermKind::Select { base, .. } => {
            free.extend(base.free_variables.iter().copied());
        }
        NormalizedTermKind::Apply { args, .. }
        | NormalizedTermKind::Tuple(args)
        | NormalizedTermKind::SetEnum(args) => {
            for arg in args {
                free.extend(arg.free_variables.iter().copied());
            }
        }
        NormalizedTermKind::Generated { origin, args } => {
            for param in &origin.params {
                if let Some(id) = param.free_id() {
                    free.insert(id);
                }
            }
            for arg in args {
                free.extend(arg.free_variables.iter().copied());
            }
        }
    }
    free
}

fn formula_free_variables(kind: &NormalizedFormulaKind) -> BTreeSet<CoreVarId> {
    let mut free = BTreeSet::new();
    match kind {
        NormalizedFormulaKind::Var(var) => {
            if let Some(id) = var.free_id() {
                free.insert(id);
            }
        }
        NormalizedFormulaKind::True
        | NormalizedFormulaKind::False
        | NormalizedFormulaKind::Error(_) => {}
        NormalizedFormulaKind::Atom { args, .. } => {
            for arg in args {
                free.extend(arg.free_variables.iter().copied());
            }
        }
        NormalizedFormulaKind::Equals { left, right } => {
            free.extend(left.free_variables.iter().copied());
            free.extend(right.free_variables.iter().copied());
        }
        NormalizedFormulaKind::TypePred { subject, .. } => {
            free.extend(subject.free_variables.iter().copied());
        }
        NormalizedFormulaKind::Not(inner) => {
            free.extend(inner.free_variables.iter().copied());
        }
        NormalizedFormulaKind::And(items) | NormalizedFormulaKind::Or(items) => {
            for item in items {
                free.extend(item.free_variables.iter().copied());
            }
        }
        NormalizedFormulaKind::Implies {
            premise,
            conclusion,
        } => {
            free.extend(premise.free_variables.iter().copied());
            free.extend(conclusion.free_variables.iter().copied());
        }
        NormalizedFormulaKind::Iff { left, right } => {
            free.extend(left.free_variables.iter().copied());
            free.extend(right.free_variables.iter().copied());
        }
        NormalizedFormulaKind::Forall { binders, body }
        | NormalizedFormulaKind::Exists { binders, body } => {
            let mut scoped_free = BTreeSet::new();
            for (index, entry) in binders.iter().enumerate() {
                if let Some(guard) = &entry.ty_guard {
                    let mut guard_free = guard.free_variables.clone();
                    for scoped in binders.iter().take(index + 1) {
                        guard_free.remove(&scoped.frame.original_var);
                    }
                    scoped_free.extend(guard_free);
                }
            }
            let mut body_free = body.free_variables.clone();
            for binder in binders {
                body_free.remove(&binder.frame.original_var);
            }
            scoped_free.extend(body_free);
            free.extend(scoped_free);
        }
    }
    free
}

fn add_depth(depth: u32, binders: usize, state: &SubstitutionState<'_>) -> BinderResult<u32> {
    depth
        .checked_add(u32::try_from(binders).map_err(|_| {
            state.diagnostic(
                BinderDiagnosticClass::InvalidBoundIndex,
                "binder-depth-overflow",
            )
        })?)
        .ok_or_else(|| {
            state.diagnostic(
                BinderDiagnosticClass::InvalidBoundIndex,
                "binder-depth-overflow",
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core_ir::*;
    use mizar_resolve::resolved_ast::{FullyQualifiedName, LocalSymbolId, ModuleId};
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
        SourceRange,
    };

    fn source_id_for(hex_pair: &str) -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            hex_pair.repeat(32)
        ))
        .expect("valid snapshot id");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source id")
    }

    fn source_id() -> SourceId {
        source_id_for("11")
    }

    fn source() -> CoreSourceRef {
        let source_id = source_id();
        CoreSourceRef::direct(SourceRange {
            source_id,
            start: 0,
            end: 1,
        })
    }

    fn module_id() -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("main"))
    }

    fn symbol(name: &str) -> SymbolId {
        let module = module_id();
        SymbolId::new(
            module,
            LocalSymbolId::new(name),
            FullyQualifiedName::new(format!("pkg::main::{name}")),
        )
    }

    fn var(index: usize) -> CoreVarId {
        CoreVarId::new(index)
    }

    fn role(name: &str) -> CoreVarRole {
        CoreVarRole::new(name)
    }

    fn predicate_ty(name: &str) -> CoreTypePredicate {
        CoreTypePredicate::new(name)
    }

    fn frame(index: u32, original: CoreVarId, source_name: &str) -> BinderFrame {
        let mut frame = BinderFrame::new(index, original, role("term"), source());
        frame.source_name = Some(source_name.to_owned());
        frame
    }

    fn context_for(vars: &[(CoreVarId, NormalizedVarSort)]) -> BinderContext {
        let mut context = BinderContext::new();
        for (var, sort) in vars {
            context.declare_variable(*var, NormalizedVarClass::Free, role("term"), *sort);
        }
        context
    }

    fn context_for_classes(
        vars: &[(CoreVarId, NormalizedVarClass, NormalizedVarSort)],
    ) -> BinderContext {
        let mut context = BinderContext::new();
        for (var, class, sort) in vars {
            context.declare_variable(*var, *class, role("term"), *sort);
        }
        context
    }

    fn freshness(max_attempts: u32) -> FreshnessConfig {
        FreshnessConfig {
            source_id: source_id(),
            owner: CoreItemId::new(0),
            binder_path: NormalizedTermOrFormulaPath::new(vec![4]),
            max_attempts,
        }
    }

    fn core_parts() -> CoreIrParts {
        let source_id = source_id();
        let source = source();
        let mut items = CoreItemTable::new();
        let item = items.insert(CoreItem::new(
            symbol("Th1"),
            CoreItemKind::Theorem,
            "public",
            source.clone(),
        ));
        let mut source_map = CoreSourceMap::new();
        source_map.item_sources.insert(item, source);

        CoreIrParts {
            source_id,
            module_id: module_id(),
            items,
            terms: CoreTermTable::new(),
            formulas: CoreFormulaTable::new(),
            definitions: CoreDefinitionTable::new(),
            proofs: CoreProofTable::new(),
            proof_nodes: CoreProofNodeTable::new(),
            algorithms: CoreAlgorithmTable::new(),
            algorithm_statements: CoreAlgorithmStmtTable::new(),
            generated: GeneratedOriginTable::new(),
            obligation_seeds: ObligationSeedTable::new(),
            source_map,
            diagnostics: CoreDiagnosticTable::new(),
        }
    }

    fn push_term(parts: &mut CoreIrParts, kind: CoreTermKind) -> CoreTermId {
        let source = source();
        let id = parts.terms.insert(CoreTerm::new(kind, source.clone()));
        parts.source_map.term_sources.insert(id, source);
        id
    }

    fn push_formula(parts: &mut CoreIrParts, kind: CoreFormulaKind) -> CoreFormulaId {
        let source = source();
        let id = parts
            .formulas
            .insert(CoreFormula::new(kind, source.clone()));
        parts.source_map.formula_sources.insert(id, source);
        id
    }

    fn push_generated(
        parts: &mut CoreIrParts,
        kind: GeneratedOriginKind,
        key: &str,
        params: Vec<CoreVarId>,
    ) -> GeneratedOriginId {
        let source = source();
        let id = parts.generated.insert(GeneratedOrigin {
            owner: CoreItemId::new(0),
            kind,
            key: GeneratedOriginKey::new(key),
            functor: None,
            params,
            evidence: vec![CoreProvenance::new(CoreProvenancePhase::Generated, "test")],
            source: source.clone(),
        });
        parts.source_map.generated_sources.insert(id, source);
        id
    }

    fn core_binder(var: CoreVarId, guard: Option<CoreFormulaId>, source_name: &str) -> CoreBinder {
        CoreBinder {
            var,
            role: role("term"),
            ty_guard: guard,
            source_name: Some(source_name.to_owned()),
            source: source(),
        }
    }

    fn term_substitution(
        target: CoreVarId,
        replacement: NormalizedTerm,
        context: BinderContext,
    ) -> Substitution {
        Substitution {
            target: SubstitutionTarget::TermVar(target),
            replacement: SubstitutionReplacement::Term(replacement),
            required_role: Some(role("term")),
            context,
            side_conditions: SubstitutionSideConditions::default(),
            freshness: freshness(10),
            diagnostic_source: source(),
        }
    }

    fn expect_applied<T>(result: SubstitutionResult<T>, message: &str) -> SubstitutionOutput<T> {
        match result {
            SubstitutionResult::Applied(output) => output,
            SubstitutionResult::Rejected(diagnostic) => panic!("{message}: {diagnostic:?}"),
        }
    }

    fn equals(left: NormalizedTerm, right: NormalizedTerm) -> NormalizedFormula {
        NormalizedFormula::new(NormalizedFormulaKind::Equals { left, right })
    }

    #[test]
    fn substitution_freshens_binders_that_would_capture_replacement() {
        let target = var(0);
        let captured_name = var(2);
        let body = equals(NormalizedTerm::free(target), NormalizedTerm::bound(0));
        let formula = NormalizedFormula::forall(vec![frame(0, captured_name, "z")], body);
        let substitution = term_substitution(
            target,
            NormalizedTerm::free(captured_name),
            context_for(&[
                (target, NormalizedVarSort::Term),
                (captured_name, NormalizedVarSort::Term),
            ]),
        );

        let output = expect_applied(
            apply_substitution_to_formula(&formula, &substitution),
            "substitution",
        );

        let NormalizedFormulaKind::Forall { binders, body } = output.value.kind else {
            panic!("expected forall");
        };
        let expected_witness = FreshnessWitness {
            source_id: source_id(),
            owner: CoreItemId::new(0),
            original: captured_name,
            fresh: deterministic_fresh_id(
                source_id(),
                CoreItemId::new(0),
                &NormalizedTermOrFormulaPath::new(vec![4, 0]),
                &role("term"),
                captured_name,
                0,
            ),
            binder_path: NormalizedTermOrFormulaPath::new(vec![4, 0]),
            role: role("term"),
            counter: 0,
        };
        assert_eq!(binders[0].frame.original_var, expected_witness.fresh);
        assert_eq!(
            recompute_fresh_id(&expected_witness),
            expected_witness.fresh
        );
        assert_eq!(output.freshness_witnesses, vec![expected_witness]);
        assert_eq!(
            body.free_variables,
            BTreeSet::from([captured_name]),
            "replacement remains free after binder freshening"
        );
    }

    #[test]
    fn substitution_freshens_nested_binders_with_nested_witness_path() {
        let target = var(0);
        let outer = var(1);
        let inner_capture = var(2);
        let inner_body = equals(NormalizedTerm::free(target), NormalizedTerm::bound(0));
        let inner = NormalizedFormula::forall(vec![frame(1, inner_capture, "z")], inner_body);
        let formula = NormalizedFormula::forall(vec![frame(0, outer, "y")], inner);
        let substitution = term_substitution(
            target,
            NormalizedTerm::free(inner_capture),
            context_for(&[
                (target, NormalizedVarSort::Term),
                (outer, NormalizedVarSort::Term),
                (inner_capture, NormalizedVarSort::Term),
            ]),
        );

        let output = expect_applied(
            apply_substitution_to_formula(&formula, &substitution),
            "nested substitution",
        );
        let NormalizedFormulaKind::Forall {
            body: outer_body, ..
        } = output.value.kind
        else {
            panic!("expected outer forall");
        };
        let NormalizedFormulaKind::Forall { binders, body } = outer_body.kind else {
            panic!("expected inner forall");
        };
        let expected_path = NormalizedTermOrFormulaPath::new(vec![4, 0, 1]);
        let expected_fresh = deterministic_fresh_id(
            source_id(),
            CoreItemId::new(0),
            &expected_path,
            &role("term"),
            inner_capture,
            0,
        );

        assert_eq!(binders[0].frame.original_var, expected_fresh);
        assert_eq!(
            output.freshness_witnesses,
            vec![FreshnessWitness {
                source_id: source_id(),
                owner: CoreItemId::new(0),
                original: inner_capture,
                fresh: expected_fresh,
                binder_path: expected_path,
                role: role("term"),
                counter: 0,
            }]
        );
        assert_eq!(body.free_variables, BTreeSet::from([inner_capture]));
    }

    #[test]
    fn substitution_uses_stable_ids_not_shadowed_display_names() {
        let outer_x = var(0);
        let shadow_x = var(1);
        let replacement = var(3);
        let body = equals(NormalizedTerm::free(outer_x), NormalizedTerm::bound(0));
        let formula = NormalizedFormula::forall(vec![frame(0, shadow_x, "x")], body);
        let substitution = term_substitution(
            outer_x,
            NormalizedTerm::free(replacement),
            context_for(&[
                (outer_x, NormalizedVarSort::Term),
                (shadow_x, NormalizedVarSort::Term),
                (replacement, NormalizedVarSort::Term),
            ]),
        );

        let output = expect_applied(
            apply_substitution_to_formula(&formula, &substitution),
            "substitution",
        );
        let NormalizedFormulaKind::Forall { binders, body } = output.value.kind else {
            panic!("expected forall");
        };

        assert_eq!(binders[0].frame.original_var, shadow_x);
        assert_eq!(binders[0].frame.source_name.as_deref(), Some("x"));
        assert_eq!(body.free_variables, BTreeSet::from([replacement]));
        assert!(output.freshness_witnesses.is_empty());
    }

    #[test]
    fn independent_substitutions_compose_to_the_expected_result() {
        let x = var(0);
        let y = var(1);
        let a = var(10);
        let b = var(11);
        let source = NormalizedTerm::new(NormalizedTermKind::Tuple(vec![
            NormalizedTerm::free(x),
            NormalizedTerm::free(y),
        ]));
        let context = context_for(&[
            (x, NormalizedVarSort::Term),
            (y, NormalizedVarSort::Term),
            (a, NormalizedVarSort::Term),
            (b, NormalizedVarSort::Term),
        ]);

        let first = term_substitution(x, NormalizedTerm::free(a), context.clone());
        let after_first = expect_applied(apply_substitution_to_term(&source, &first), "first");
        let second = term_substitution(y, NormalizedTerm::free(b), context);
        let composed = expect_applied(
            apply_substitution_to_term(&after_first.value, &second),
            "second",
        );

        assert_eq!(
            composed.value,
            NormalizedTerm::new(NormalizedTermKind::Tuple(vec![
                NormalizedTerm::free(a),
                NormalizedTerm::free(b),
            ]))
        );
    }

    #[test]
    fn rejected_substitution_composition_reports_side_conditions() {
        let x = var(0);
        let y = var(1);
        let a = var(10);
        let b = var(11);
        let source = NormalizedTerm::new(NormalizedTermKind::Tuple(vec![
            NormalizedTerm::free(x),
            NormalizedTerm::free(y),
        ]));
        let context = context_for(&[
            (x, NormalizedVarSort::Term),
            (y, NormalizedVarSort::Term),
            (a, NormalizedVarSort::Term),
            (b, NormalizedVarSort::Term),
        ]);
        let first = term_substitution(x, NormalizedTerm::free(a), context.clone());
        let after_first = expect_applied(apply_substitution_to_term(&source, &first), "first");
        let mut second = term_substitution(y, NormalizedTerm::free(b), context);
        second.side_conditions.forbidden_free_variables.insert(b);

        assert!(matches!(
            apply_substitution_to_term(&after_first.value, &second),
            SubstitutionResult::Rejected(BinderDiagnostic {
                class: BinderDiagnosticClass::SideConditionViolation,
                ..
            })
        ));
    }

    #[test]
    fn substitution_rejects_side_condition_and_capture_failures() {
        let x = var(0);
        let z = var(2);
        let mut side_conditions = SubstitutionSideConditions::default();
        side_conditions.forbidden_free_variables.insert(z);
        let mut substitution = term_substitution(
            x,
            NormalizedTerm::free(z),
            context_for(&[(x, NormalizedVarSort::Term), (z, NormalizedVarSort::Term)]),
        );
        substitution.side_conditions = side_conditions;
        assert!(matches!(
            apply_substitution_to_term(&NormalizedTerm::free(x), &substitution),
            SubstitutionResult::Rejected(BinderDiagnostic {
                class: BinderDiagnosticClass::SideConditionViolation,
                ..
            })
        ));

        let body = equals(NormalizedTerm::free(x), NormalizedTerm::bound(0));
        let formula = NormalizedFormula::forall(vec![frame(0, z, "z")], body);
        let mut substitution = term_substitution(
            x,
            NormalizedTerm::free(z),
            context_for(&[(x, NormalizedVarSort::Term), (z, NormalizedVarSort::Term)]),
        );
        substitution.side_conditions.capture_policy = CapturePolicy::Reject;
        assert!(matches!(
            apply_substitution_to_formula(&formula, &substitution),
            SubstitutionResult::Rejected(BinderDiagnostic {
                class: BinderDiagnosticClass::CaptureAvoidance,
                ..
            })
        ));
    }

    #[test]
    fn malformed_substitutions_are_rejected_deterministically() {
        let x = var(0);
        let source_term = NormalizedTerm::free(x);
        let context = context_for(&[(x, NormalizedVarSort::Term)]);

        let sort_mismatch = Substitution {
            target: SubstitutionTarget::TermVar(x),
            replacement: SubstitutionReplacement::Formula(NormalizedFormula::new(
                NormalizedFormulaKind::True,
            )),
            required_role: Some(role("term")),
            context: context.clone(),
            side_conditions: SubstitutionSideConditions::default(),
            freshness: freshness(10),
            diagnostic_source: source(),
        };
        assert!(matches!(
            apply_substitution_to_term(&source_term, &sort_mismatch),
            SubstitutionResult::Rejected(BinderDiagnostic {
                class: BinderDiagnosticClass::SortMismatch,
                ..
            })
        ));

        let mut role_context = context.clone();
        role_context.declare_variable(
            var(9),
            NormalizedVarClass::Free,
            role("term"),
            NormalizedVarSort::Term,
        );
        let mut role_mismatch = term_substitution(x, NormalizedTerm::free(var(9)), role_context);
        role_mismatch.required_role = Some(role("scheme"));
        assert!(matches!(
            apply_substitution_to_term(&source_term, &role_mismatch),
            SubstitutionResult::Rejected(BinderDiagnostic {
                class: BinderDiagnosticClass::RoleMismatch,
                ..
            })
        ));

        let missing_metadata =
            term_substitution(x, NormalizedTerm::free(var(9)), BinderContext::new());
        assert!(matches!(
            apply_substitution_to_term(&source_term, &missing_metadata),
            SubstitutionResult::Rejected(BinderDiagnostic {
                class: BinderDiagnosticClass::MissingVariableMetadata,
                ..
            })
        ));

        let mut partial_metadata = BinderContext::new();
        partial_metadata
            .variable_classes
            .insert(x, NormalizedVarClass::Free);
        partial_metadata
            .variable_sorts
            .insert(x, NormalizedVarSort::Term);
        let partial = term_substitution(x, NormalizedTerm::free(x), partial_metadata);
        assert!(matches!(
            apply_substitution_to_term(&source_term, &partial),
            SubstitutionResult::Rejected(BinderDiagnostic {
                class: BinderDiagnosticClass::MissingVariableMetadata,
                ..
            })
        ));

        let schematic_source = NormalizedTerm::var(NormalizedVar::Schematic(x));
        let class_mismatch = term_substitution(
            x,
            NormalizedTerm::var(NormalizedVar::Generated(x)),
            context_for_classes(&[(x, NormalizedVarClass::Schematic, NormalizedVarSort::Term)]),
        );
        assert!(matches!(
            apply_substitution_to_term(&schematic_source, &class_mismatch),
            SubstitutionResult::Rejected(BinderDiagnostic {
                class: BinderDiagnosticClass::ClassMismatch,
                ..
            })
        ));

        let mut malformed = term_substitution(x, NormalizedTerm::free(var(9)), context.clone());
        malformed.side_conditions.malformed_evidence = true;
        assert!(matches!(
            apply_substitution_to_term(&source_term, &malformed),
            SubstitutionResult::Rejected(BinderDiagnostic {
                class: BinderDiagnosticClass::MalformedEvidence,
                ..
            })
        ));

        let z = var(2);
        let body = equals(NormalizedTerm::free(x), NormalizedTerm::bound(0));
        let formula = NormalizedFormula::forall(vec![frame(0, z, "z")], body);
        let colliding_fresh = deterministic_fresh_id(
            source_id(),
            CoreItemId::new(0),
            &NormalizedTermOrFormulaPath::new(vec![4, 0]),
            &role("term"),
            z,
            0,
        );
        let mut exhausted = term_substitution(
            x,
            NormalizedTerm::free(z),
            context_for(&[
                (x, NormalizedVarSort::Term),
                (z, NormalizedVarSort::Term),
                (colliding_fresh, NormalizedVarSort::Term),
            ]),
        );
        exhausted.freshness = freshness(1);
        assert!(matches!(
            apply_substitution_to_formula(&formula, &exhausted),
            SubstitutionResult::Rejected(BinderDiagnostic {
                class: BinderDiagnosticClass::FreshnessExhausted,
                ..
            })
        ));
    }

    #[test]
    fn definition_closure_expands_actuals_and_instantiates_guards() {
        let formal_n = var(0);
        let captured_m = var(2);
        let quantified_m = var(3);
        let formal = frame(0, formal_n, "n");
        let body = equals(NormalizedTerm::bound(0), NormalizedTerm::free(captured_m));
        let guard = NormalizedFormula::new(NormalizedFormulaKind::TypePred {
            subject: NormalizedTerm::bound(0),
            ty: predicate_ty("Nat"),
        });
        let closure = DefinitionClosure {
            formals: vec![formal],
            body: NormalizedTermOrFormula::Formula(body),
            captured_free_variables: BTreeSet::from([captured_m]),
            formal_type_guards: vec![guard],
            source: source(),
        };

        let expansion = expand_definition_closure(
            &closure,
            &[NormalizedTerm::free(quantified_m)],
            &context_for(&[
                (formal_n, NormalizedVarSort::Term),
                (captured_m, NormalizedVarSort::Term),
                (quantified_m, NormalizedVarSort::Term),
            ]),
            freshness(10),
        );
        let expansion = expect_applied(expansion, "closure expansion");

        assert_eq!(
            expansion.value.body,
            NormalizedTermOrFormula::Formula(equals(
                NormalizedTerm::free(quantified_m),
                NormalizedTerm::free(captured_m)
            ))
        );
        assert_eq!(
            expansion.value.guard_facts,
            vec![NormalizedFormula::new(NormalizedFormulaKind::TypePred {
                subject: NormalizedTerm::free(quantified_m),
                ty: predicate_ty("Nat"),
            })]
        );
        assert_eq!(
            expansion.value.captured_free_variables,
            BTreeSet::from([captured_m])
        );
    }

    #[test]
    fn definition_closure_expansion_freshens_body_binders_that_capture_actuals() {
        let formal_n = var(0);
        let capture_z = var(2);
        let formal = frame(0, formal_n, "n");
        let inner = NormalizedFormula::forall(
            vec![frame(0, capture_z, "z")],
            equals(NormalizedTerm::bound(1), NormalizedTerm::bound(0)),
        );
        let closure = DefinitionClosure {
            formals: vec![formal],
            body: NormalizedTermOrFormula::Formula(inner),
            captured_free_variables: BTreeSet::new(),
            formal_type_guards: Vec::new(),
            source: source(),
        };

        let expansion = expect_applied(
            expand_definition_closure(
                &closure,
                &[NormalizedTerm::free(capture_z)],
                &context_for(&[
                    (formal_n, NormalizedVarSort::Term),
                    (capture_z, NormalizedVarSort::Term),
                ]),
                freshness(10),
            ),
            "closure expansion",
        );
        let NormalizedTermOrFormula::Formula(expanded) = expansion.value.body else {
            panic!("expected formula body");
        };
        let NormalizedFormulaKind::Forall { binders, body } = expanded.kind else {
            panic!("expected forall");
        };
        assert_ne!(binders[0].frame.original_var, capture_z);
        assert_eq!(body.free_variables, BTreeSet::from([capture_z]));
        assert_eq!(expansion.value.freshness_witnesses.len(), 1);
        assert_eq!(
            recompute_fresh_id(&expansion.value.freshness_witnesses[0]),
            binders[0].frame.original_var
        );
    }

    #[test]
    fn definition_closure_shadowing_regression_keeps_use_site_m_distinct() {
        let formal_n = var(0);
        let captured_m = var(2);
        let quantified_m = var(3);
        let mut use_site_m = frame(0, quantified_m, "m");
        use_site_m.source_name = Some("m".to_owned());
        let closure = DefinitionClosure {
            formals: vec![frame(0, formal_n, "n")],
            body: NormalizedTermOrFormula::Formula(equals(
                NormalizedTerm::bound(0),
                NormalizedTerm::free(captured_m),
            )),
            captured_free_variables: BTreeSet::from([captured_m]),
            formal_type_guards: Vec::new(),
            source: source(),
        };

        let expansion = expect_applied(
            expand_definition_closure(
                &closure,
                &[NormalizedTerm::free(quantified_m)],
                &context_for(&[
                    (formal_n, NormalizedVarSort::Term),
                    (captured_m, NormalizedVarSort::Term),
                    (quantified_m, NormalizedVarSort::Term),
                ]),
                freshness(10),
            ),
            "closure expansion",
        );

        assert_eq!(use_site_m.source_name.as_deref(), Some("m"));
        assert_eq!(use_site_m.original_var, quantified_m);
        assert_eq!(
            expansion.value.body,
            NormalizedTermOrFormula::Formula(equals(
                NormalizedTerm::free(quantified_m),
                NormalizedTerm::free(captured_m)
            ))
        );
        assert_eq!(
            expansion.value.captured_free_variables,
            BTreeSet::from([captured_m])
        );
    }

    #[test]
    fn definition_closure_rejects_actual_formal_mismatch() {
        let closure = DefinitionClosure {
            formals: vec![frame(0, var(0), "n")],
            body: NormalizedTermOrFormula::Term(NormalizedTerm::bound(0)),
            captured_free_variables: BTreeSet::new(),
            formal_type_guards: Vec::new(),
            source: source(),
        };

        assert!(matches!(
            expand_definition_closure(&closure, &[], &BinderContext::new(), freshness(10)),
            SubstitutionResult::Rejected(BinderDiagnostic {
                class: BinderDiagnosticClass::ClosureArityMismatch,
                ..
            })
        ));
    }

    #[test]
    fn malformed_bound_indices_are_rejected() {
        let x = var(0);
        let substitution = term_substitution(
            x,
            NormalizedTerm::free(x),
            context_for(&[(x, NormalizedVarSort::Term)]),
        );
        assert!(matches!(
            apply_substitution_to_term(&NormalizedTerm::bound(0), &substitution),
            SubstitutionResult::Rejected(BinderDiagnostic {
                class: BinderDiagnosticClass::InvalidBoundIndex,
                ..
            })
        ));

        let closure = DefinitionClosure {
            formals: vec![frame(0, x, "x")],
            body: NormalizedTermOrFormula::Term(NormalizedTerm::bound(2)),
            captured_free_variables: BTreeSet::new(),
            formal_type_guards: Vec::new(),
            source: source(),
        };
        assert!(matches!(
            expand_definition_closure(
                &closure,
                &[NormalizedTerm::free(x)],
                &context_for(&[(x, NormalizedVarSort::Term)]),
                freshness(10),
            ),
            SubstitutionResult::Rejected(BinderDiagnostic {
                class: BinderDiagnosticClass::InvalidBoundIndex,
                ..
            })
        ));
    }

    #[test]
    fn closure_guard_formal_mismatch_is_rejected_at_same_arity() {
        let x = var(0);
        let closure = DefinitionClosure {
            formals: vec![frame(0, x, "x")],
            body: NormalizedTermOrFormula::Term(NormalizedTerm::bound(0)),
            captured_free_variables: BTreeSet::new(),
            formal_type_guards: vec![NormalizedFormula::new(NormalizedFormulaKind::TypePred {
                subject: NormalizedTerm::bound(1),
                ty: predicate_ty("Nat"),
            })],
            source: source(),
        };

        assert!(matches!(
            expand_definition_closure(
                &closure,
                &[NormalizedTerm::free(x)],
                &context_for(&[(x, NormalizedVarSort::Term)]),
                freshness(10),
            ),
            SubstitutionResult::Rejected(BinderDiagnostic {
                class: BinderDiagnosticClass::InvalidBoundIndex,
                ..
            })
        ));
    }

    #[test]
    fn explicit_bound_operations_reject_invalid_negative_shifts() {
        let result = shift_term(&NormalizedTerm::bound(0), 0, -1, &source());

        assert!(matches!(
            result,
            Err(BinderDiagnostic {
                class: BinderDiagnosticClass::InvalidBoundIndex,
                ..
            })
        ));
    }

    #[test]
    fn helper_constructors_support_formula_symbols() {
        let formula = NormalizedFormula::new(NormalizedFormulaKind::Atom {
            predicate: symbol("P"),
            args: vec![NormalizedTerm::free(var(0))],
        });

        assert_eq!(formula.free_variables, BTreeSet::from([var(0)]));
    }

    #[test]
    fn formula_de_bruijn_helpers_cover_formula_variables() {
        let phi = var(30);
        let source = source();
        let replacement = NormalizedFormula::new(NormalizedFormulaKind::True);
        let opened = open_rec_formula_with_formula(
            &NormalizedFormula::var(NormalizedVar::Bound(BoundVar::new(0))),
            0,
            &replacement,
            &source,
        )
        .expect("open formula");
        assert_eq!(opened, replacement);

        let closed = close_rec_formula(
            &NormalizedFormula::var(NormalizedVar::Free(phi)),
            0,
            phi,
            &source,
        )
        .expect("close formula");
        assert_eq!(
            closed,
            NormalizedFormula::var(NormalizedVar::Bound(BoundVar::new(0)))
        );

        let substituted = subst_bound_formula(
            &closed,
            0,
            &SubstitutionReplacement::Formula(replacement.clone()),
            &source,
        )
        .expect("subst formula bound");
        assert_eq!(substituted, replacement);

        let term_formula = equals(NormalizedTerm::free(phi), NormalizedTerm::bound(0));
        let closed_term_formula =
            close_rec_formula(&term_formula, 0, phi, &source).expect("close term in formula");
        assert_eq!(
            closed_term_formula,
            equals(NormalizedTerm::bound(0), NormalizedTerm::bound(1))
        );
        let substituted_term_formula = subst_bound_formula(
            &closed_term_formula,
            0,
            &SubstitutionReplacement::Term(NormalizedTerm::free(var(31))),
            &source,
        )
        .expect("subst term in formula");
        assert_eq!(
            substituted_term_formula,
            equals(NormalizedTerm::free(var(31)), NormalizedTerm::bound(1))
        );
    }

    #[test]
    fn alpha_equivalence_is_canonical_reflexive_symmetric_and_transitive() {
        let x = var(0);
        let y = var(1);
        let z = var(2);
        let formula_x = NormalizedFormula::forall(
            vec![frame(0, x, "x")],
            NormalizedFormula::new(NormalizedFormulaKind::Atom {
                predicate: symbol("P"),
                args: vec![NormalizedTerm::bound(0)],
            }),
        );
        let formula_y = NormalizedFormula::forall(
            vec![frame(9, y, "renamed")],
            NormalizedFormula::new(NormalizedFormulaKind::Atom {
                predicate: symbol("P"),
                args: vec![NormalizedTerm::bound(0)],
            }),
        );
        let formula_z = NormalizedFormula::forall(
            vec![frame(2, z, "z")],
            NormalizedFormula::new(NormalizedFormulaKind::Atom {
                predicate: symbol("P"),
                args: vec![NormalizedTerm::bound(0)],
            }),
        );
        let context = context_for(&[
            (x, NormalizedVarSort::Term),
            (y, NormalizedVarSort::Term),
            (z, NormalizedVarSort::Term),
        ]);

        assert!(alpha_equivalent_formulas(&formula_x, &formula_x, &context, &source()).unwrap());
        assert!(alpha_equivalent_formulas(&formula_x, &formula_y, &context, &source()).unwrap());
        assert!(alpha_equivalent_formulas(&formula_y, &formula_x, &context, &source()).unwrap());
        assert!(alpha_equivalent_formulas(&formula_y, &formula_z, &context, &source()).unwrap());
        assert!(alpha_equivalent_formulas(&formula_x, &formula_z, &context, &source()).unwrap());
        assert_eq!(
            canonical_formula(&formula_x, &context, &source()).unwrap(),
            canonical_formula(&formula_y, &context, &source()).unwrap()
        );
        assert_eq!(
            canonical_formula(&formula_x, &context, &source()).unwrap(),
            canonical_formula(&formula_x, &context, &source()).unwrap(),
            "canonical form is deterministic across repeated runs"
        );
    }

    #[test]
    fn canonical_forms_differ_exactly_when_alpha_equivalence_differs() {
        let x = var(0);
        let y = var(1);
        let context = context_for(&[(x, NormalizedVarSort::Term), (y, NormalizedVarSort::Term)]);
        let first = NormalizedTerm::new(NormalizedTermKind::Apply {
            functor: symbol("F"),
            args: vec![NormalizedTerm::free(x), NormalizedTerm::free(y)],
        });
        let same = NormalizedTerm::new(NormalizedTermKind::Apply {
            functor: symbol("F"),
            args: vec![NormalizedTerm::free(x), NormalizedTerm::free(y)],
        });
        let different_order = NormalizedTerm::new(NormalizedTermKind::Apply {
            functor: symbol("F"),
            args: vec![NormalizedTerm::free(y), NormalizedTerm::free(x)],
        });

        assert!(alpha_equivalent_terms(&first, &same, &context, &source()).unwrap());
        assert_eq!(
            canonical_term(&first, &context, &source()).unwrap(),
            canonical_term(&same, &context, &source()).unwrap()
        );
        assert!(!alpha_equivalent_terms(&first, &different_order, &context, &source()).unwrap());
        assert_ne!(
            canonical_term(&first, &context, &source()).unwrap(),
            canonical_term(&different_order, &context, &source()).unwrap()
        );
    }

    #[test]
    fn canonical_formula_iff_includes_binders_guards_and_iff_nodes() {
        let x = var(0);
        let y = var(1);
        let context = context_for(&[(x, NormalizedVarSort::Term), (y, NormalizedVarSort::Term)]);
        let guard_x = NormalizedFormula::new(NormalizedFormulaKind::TypePred {
            subject: NormalizedTerm::bound(0),
            ty: predicate_ty("Nat"),
        });
        let guard_y = NormalizedFormula::new(NormalizedFormulaKind::TypePred {
            subject: NormalizedTerm::bound(0),
            ty: predicate_ty("Nat"),
        });
        let left = NormalizedFormula::forall_entries(
            vec![NormalizedBinderEntry::with_guard(frame(0, x, "x"), guard_x)],
            NormalizedFormula::new(NormalizedFormulaKind::Iff {
                left: Box::new(NormalizedFormula::new(NormalizedFormulaKind::True)),
                right: Box::new(NormalizedFormula::new(NormalizedFormulaKind::False)),
            }),
        );
        let same = NormalizedFormula::forall_entries(
            vec![NormalizedBinderEntry::with_guard(
                frame(7, y, "renamed"),
                guard_y,
            )],
            NormalizedFormula::new(NormalizedFormulaKind::Iff {
                left: Box::new(NormalizedFormula::new(NormalizedFormulaKind::True)),
                right: Box::new(NormalizedFormula::new(NormalizedFormulaKind::False)),
            }),
        );
        let different_guard = NormalizedFormula::forall_entries(
            vec![NormalizedBinderEntry::with_guard(
                frame(0, x, "x"),
                NormalizedFormula::new(NormalizedFormulaKind::True),
            )],
            NormalizedFormula::new(NormalizedFormulaKind::Iff {
                left: Box::new(NormalizedFormula::new(NormalizedFormulaKind::True)),
                right: Box::new(NormalizedFormula::new(NormalizedFormulaKind::False)),
            }),
        );

        assert!(alpha_equivalent_formulas(&left, &same, &context, &source()).unwrap());
        assert_eq!(
            canonical_formula(&left, &context, &source()).unwrap(),
            canonical_formula(&same, &context, &source()).unwrap()
        );
        assert!(!alpha_equivalent_formulas(&left, &different_guard, &context, &source()).unwrap());
        assert_ne!(
            canonical_formula(&left, &context, &source()).unwrap(),
            canonical_formula(&different_guard, &context, &source()).unwrap()
        );
    }

    #[test]
    fn free_variables_remove_binders_from_guards_and_body_in_sorted_order() {
        let x = var(0);
        let y = var(1);
        let z = var(2);
        let guard = NormalizedFormula::new(NormalizedFormulaKind::And(vec![
            NormalizedFormula::new(NormalizedFormulaKind::TypePred {
                subject: NormalizedTerm::bound(0),
                ty: predicate_ty("Nat"),
            }),
            equals(NormalizedTerm::free(z), NormalizedTerm::bound(0)),
        ]));
        let entry = NormalizedBinderEntry::with_guard(frame(0, x, "x"), guard);
        let formula = NormalizedFormula::forall_entries(
            vec![entry],
            equals(NormalizedTerm::free(y), NormalizedTerm::bound(0)),
        );

        assert_eq!(
            formula.free_variables.iter().copied().collect::<Vec<_>>(),
            vec![y, z]
        );
    }

    #[test]
    fn hand_built_guard_later_binder_remains_free_and_is_rejected() {
        let x = var(0);
        let y = var(1);
        let guard_mentions_later = equals(NormalizedTerm::free(y), NormalizedTerm::bound(0));
        let formula = NormalizedFormula::forall_entries(
            vec![
                NormalizedBinderEntry::with_guard(frame(0, x, "x"), guard_mentions_later),
                NormalizedBinderEntry::new(frame(1, y, "y")),
            ],
            NormalizedFormula::new(NormalizedFormulaKind::True),
        );
        assert_eq!(formula.free_variables, BTreeSet::from([y]));
        assert!(matches!(
            validate_normalized_formula(
                &formula,
                &context_for(&[(x, NormalizedVarSort::Term), (y, NormalizedVarSort::Term)]),
                &source()
            ),
            Err(BinderDiagnostic {
                class: BinderDiagnosticClass::MalformedEvidence,
                ..
            })
        ));
    }

    #[test]
    fn validation_rejects_invalid_de_bruijn_and_malformed_binder_contexts() {
        assert!(matches!(
            validate_normalized_term(&NormalizedTerm::bound(0), &BinderContext::new(), &source()),
            Err(BinderDiagnostic {
                class: BinderDiagnosticClass::InvalidBoundIndex,
                ..
            })
        ));

        let x = var(0);
        let formula = NormalizedFormula::forall(
            vec![frame(0, x, "x")],
            NormalizedFormula::new(NormalizedFormulaKind::True),
        );
        let mut malformed_context = BinderContext::new();
        malformed_context
            .variable_roles
            .insert(x, role("unexpected"));
        assert!(matches!(
            validate_normalized_formula(&formula, &malformed_context, &source()),
            Err(BinderDiagnostic {
                class: BinderDiagnosticClass::RoleMismatch,
                ..
            })
        ));
    }

    #[test]
    fn validation_and_canonicalization_honor_ambient_context_frames() {
        let ambient = var(20);
        let mut context = context_for(&[(ambient, NormalizedVarSort::Term)]);
        context.frames.push(frame(0, ambient, "ambient"));
        let term = NormalizedTerm::bound(0);

        validate_normalized_term(&term, &context, &source()).expect("ambient bound validates");
        assert_eq!(
            canonical_term(&term, &context, &source()).unwrap().kind(),
            &CanonicalTermKind::Var(CanonicalVar::Bound(0))
        );

        let mut malformed_context = context.clone();
        malformed_context
            .variable_roles
            .insert(ambient, role("wrong"));
        assert!(matches!(
            validate_normalized_term(&term, &malformed_context, &source()),
            Err(BinderDiagnostic {
                class: BinderDiagnosticClass::RoleMismatch,
                ..
            })
        ));
    }

    #[test]
    fn substitution_under_ambient_context_keeps_replacement_depth_local() {
        let target = var(0);
        let ambient = var(20);
        let mut context = context_for(&[
            (target, NormalizedVarSort::Term),
            (ambient, NormalizedVarSort::Term),
        ]);
        context.frames.push(frame(0, ambient, "ambient"));
        let substitution = term_substitution(target, NormalizedTerm::bound(0), context.clone());

        let output = expect_applied(
            apply_substitution_to_term(&NormalizedTerm::free(target), &substitution),
            "ambient substitution",
        );
        assert_eq!(output.value, NormalizedTerm::bound(0));
        validate_normalized_term(&output.value, &context, &source()).unwrap();
    }

    #[test]
    fn closure_validation_does_not_borrow_use_site_ambient_depth() {
        let formal = var(0);
        let actual = var(1);
        let ambient = var(20);
        let closure = DefinitionClosure {
            formals: vec![frame(0, formal, "n")],
            body: NormalizedTermOrFormula::Term(NormalizedTerm::bound(1)),
            captured_free_variables: BTreeSet::new(),
            formal_type_guards: Vec::new(),
            source: source(),
        };
        let mut context = context_for(&[
            (formal, NormalizedVarSort::Term),
            (actual, NormalizedVarSort::Term),
            (ambient, NormalizedVarSort::Term),
        ]);
        context.frames.push(frame(0, ambient, "ambient"));

        assert!(matches!(
            expand_definition_closure(
                &closure,
                &[NormalizedTerm::free(actual)],
                &context,
                freshness(10),
            ),
            SubstitutionResult::Rejected(BinderDiagnostic {
                class: BinderDiagnosticClass::InvalidBoundIndex,
                ..
            })
        ));
    }

    #[test]
    fn validation_rejects_free_original_var_inside_binder_scope() {
        let x = var(0);
        let context = context_for(&[(x, NormalizedVarSort::Term)]);
        let body_bad = NormalizedFormula::forall(
            vec![frame(0, x, "x")],
            equals(NormalizedTerm::free(x), NormalizedTerm::bound(0)),
        );
        assert!(matches!(
            validate_normalized_formula(&body_bad, &context, &source()),
            Err(BinderDiagnostic {
                class: BinderDiagnosticClass::MalformedEvidence,
                ..
            })
        ));

        let guard_bad = NormalizedFormula::forall_entries(
            vec![NormalizedBinderEntry::with_guard(
                frame(0, x, "x"),
                equals(NormalizedTerm::free(x), NormalizedTerm::bound(0)),
            )],
            NormalizedFormula::new(NormalizedFormulaKind::True),
        );
        assert!(matches!(
            validate_normalized_formula(&guard_bad, &context, &source()),
            Err(BinderDiagnostic {
                class: BinderDiagnosticClass::MalformedEvidence,
                ..
            })
        ));
    }

    #[test]
    fn validation_rejects_free_variable_cache_tampering_before_capture() {
        let x = var(0);
        let z = var(2);
        let mut replacement = NormalizedTerm::free(z);
        replacement.free_variables.clear();
        let formula = NormalizedFormula::forall(
            vec![frame(0, z, "z")],
            equals(NormalizedTerm::free(x), NormalizedTerm::bound(0)),
        );
        let substitution = term_substitution(
            x,
            replacement,
            context_for(&[(x, NormalizedVarSort::Term), (z, NormalizedVarSort::Term)]),
        );

        assert!(matches!(
            apply_substitution_to_formula(&formula, &substitution),
            SubstitutionResult::Rejected(BinderDiagnostic {
                class: BinderDiagnosticClass::MalformedEvidence,
                ..
            })
        ));
    }

    #[test]
    fn freshened_substitution_outputs_revalidate_with_original_context() {
        let target = var(0);
        let captured_name = var(2);
        let body = equals(NormalizedTerm::free(target), NormalizedTerm::bound(0));
        let formula = NormalizedFormula::forall(vec![frame(0, captured_name, "z")], body);
        let context = context_for(&[
            (target, NormalizedVarSort::Term),
            (captured_name, NormalizedVarSort::Term),
        ]);
        let substitution =
            term_substitution(target, NormalizedTerm::free(captured_name), context.clone());
        let output = expect_applied(
            apply_substitution_to_formula(&formula, &substitution),
            "substitution",
        );

        validate_normalized_formula(&output.value, &context, &source())
            .expect("freshened binder frame carries enough metadata to revalidate");
    }

    #[test]
    fn nested_de_bruijn_helpers_distinguish_open_and_subst_at_nonzero_depth() {
        let replacement = NormalizedTerm::free(var(10));
        let tuple = NormalizedTerm::new(NormalizedTermKind::Tuple(vec![
            NormalizedTerm::bound(2),
            NormalizedTerm::bound(1),
            NormalizedTerm::bound(0),
        ]));

        let opened = open_rec_term(&tuple, 1, &replacement, &source()).expect("open");
        assert_eq!(
            opened,
            NormalizedTerm::new(NormalizedTermKind::Tuple(vec![
                NormalizedTerm::bound(1),
                NormalizedTerm::free(var(10)),
                NormalizedTerm::bound(0),
            ]))
        );

        let substituted =
            subst_bound_term(&tuple, 1, &replacement, &source()).expect("subst bound");
        assert_eq!(
            substituted,
            NormalizedTerm::new(NormalizedTermKind::Tuple(vec![
                NormalizedTerm::bound(2),
                NormalizedTerm::free(var(10)),
                NormalizedTerm::bound(0),
            ]))
        );
    }

    #[test]
    fn nested_de_bruijn_helpers_cover_close_shift_and_guards() {
        let x = var(0);
        let y = var(1);
        let guard = equals(NormalizedTerm::free(y), NormalizedTerm::bound(0));
        let formula = NormalizedFormula::forall_entries(
            vec![NormalizedBinderEntry::with_guard(frame(0, x, "x"), guard)],
            equals(NormalizedTerm::free(y), NormalizedTerm::bound(0)),
        );

        let closed = close_rec_formula(&formula, 1, y, &source()).expect("close nonzero");
        let NormalizedFormulaKind::Forall { binders, body } = closed.kind else {
            panic!("expected forall");
        };
        assert_eq!(
            binders[0].ty_guard.as_ref().unwrap().as_ref(),
            &equals(NormalizedTerm::bound(2), NormalizedTerm::bound(0))
        );
        assert_eq!(
            body.as_ref(),
            &equals(NormalizedTerm::bound(2), NormalizedTerm::bound(0))
        );

        let shifted = shift_formula(
            &NormalizedFormula::forall(
                vec![frame(0, x, "x")],
                equals(NormalizedTerm::bound(1), NormalizedTerm::bound(0)),
            ),
            0,
            1,
            &source(),
        )
        .expect("shift formula");
        let NormalizedFormulaKind::Forall { body, .. } = shifted.kind else {
            panic!("expected forall");
        };
        assert_eq!(
            body.as_ref(),
            &equals(NormalizedTerm::bound(2), NormalizedTerm::bound(0))
        );
    }

    #[test]
    fn generated_terms_compare_by_semantic_origin_record_not_dense_id_or_source() {
        let x = var(0);
        let y = var(1);
        let context = context_for(&[(x, NormalizedVarSort::Term), (y, NormalizedVarSort::Term)]);
        let origin = GeneratedOriginRecord {
            owner: CoreItemId::new(0),
            kind: GeneratedOriginKind::LocalAbbreviation,
            key: GeneratedOriginKey::new("abbr"),
            params: vec![NormalizedVar::Free(x)],
        };
        let same_record = GeneratedOriginRecord {
            owner: CoreItemId::new(0),
            kind: GeneratedOriginKind::LocalAbbreviation,
            key: GeneratedOriginKey::new("abbr"),
            params: vec![NormalizedVar::Free(x)],
        };
        let differing_payload = GeneratedOriginRecord {
            owner: CoreItemId::new(0),
            kind: GeneratedOriginKind::LocalAbbreviation,
            key: GeneratedOriginKey::new("abbr"),
            params: vec![NormalizedVar::Free(y)],
        };
        let differing_owner = GeneratedOriginRecord {
            owner: CoreItemId::new(1),
            kind: GeneratedOriginKind::LocalAbbreviation,
            key: GeneratedOriginKey::new("abbr"),
            params: vec![NormalizedVar::Free(x)],
        };
        let differing_kind = GeneratedOriginRecord {
            owner: CoreItemId::new(0),
            kind: GeneratedOriginKind::FraenkelComprehension,
            key: GeneratedOriginKey::new("abbr"),
            params: vec![NormalizedVar::Free(x)],
        };
        let differing_key = GeneratedOriginRecord {
            owner: CoreItemId::new(0),
            kind: GeneratedOriginKind::LocalAbbreviation,
            key: GeneratedOriginKey::new("other"),
            params: vec![NormalizedVar::Free(x)],
        };
        let first = NormalizedTerm::new(NormalizedTermKind::Generated {
            origin,
            args: vec![NormalizedTerm::free(x), NormalizedTerm::free(y)],
        });
        let same = NormalizedTerm::new(NormalizedTermKind::Generated {
            origin: same_record,
            args: vec![NormalizedTerm::free(x), NormalizedTerm::free(y)],
        });
        let different = NormalizedTerm::new(NormalizedTermKind::Generated {
            origin: differing_payload,
            args: vec![NormalizedTerm::free(x), NormalizedTerm::free(y)],
        });
        let different_order = NormalizedTerm::new(NormalizedTermKind::Generated {
            origin: GeneratedOriginRecord {
                owner: CoreItemId::new(0),
                kind: GeneratedOriginKind::LocalAbbreviation,
                key: GeneratedOriginKey::new("abbr"),
                params: vec![NormalizedVar::Free(x)],
            },
            args: vec![NormalizedTerm::free(y), NormalizedTerm::free(x)],
        });
        let different_owner = NormalizedTerm::new(NormalizedTermKind::Generated {
            origin: differing_owner,
            args: vec![NormalizedTerm::free(x), NormalizedTerm::free(y)],
        });
        let different_kind = NormalizedTerm::new(NormalizedTermKind::Generated {
            origin: differing_kind,
            args: vec![NormalizedTerm::free(x), NormalizedTerm::free(y)],
        });
        let different_key = NormalizedTerm::new(NormalizedTermKind::Generated {
            origin: differing_key,
            args: vec![NormalizedTerm::free(x), NormalizedTerm::free(y)],
        });

        assert!(alpha_equivalent_terms(&first, &same, &context, &source()).unwrap());
        assert!(!alpha_equivalent_terms(&first, &different, &context, &source()).unwrap());
        assert!(!alpha_equivalent_terms(&first, &different_order, &context, &source()).unwrap());
        assert!(!alpha_equivalent_terms(&first, &different_owner, &context, &source()).unwrap());
        assert!(!alpha_equivalent_terms(&first, &different_kind, &context, &source()).unwrap());
        assert!(!alpha_equivalent_terms(&first, &different_key, &context, &source()).unwrap());
    }

    #[test]
    fn generated_raw_terms_with_distinct_dense_ids_compare_by_semantic_record() {
        let x = var(0);
        let context = context_for(&[(x, NormalizedVarSort::Term)]);

        fn generated_fixture(x: CoreVarId, add_dummy: bool) -> (CoreIr, CoreTermId) {
            let mut parts = core_parts();
            if add_dummy {
                push_generated(
                    &mut parts,
                    GeneratedOriginKind::LocalAbbreviation,
                    "dummy",
                    Vec::new(),
                );
            }
            let origin = push_generated(
                &mut parts,
                GeneratedOriginKind::LocalAbbreviation,
                "same",
                vec![x],
            );
            let arg = push_term(&mut parts, CoreTermKind::Var(x));
            let term = push_term(
                &mut parts,
                CoreTermKind::Generated {
                    origin,
                    args: vec![arg],
                },
            );
            (CoreIr::try_new(parts).expect("core fixture"), term)
        }

        let (left_core, left_term) = generated_fixture(x, true);
        let (right_core, right_term) = generated_fixture(x, false);
        let left = normalize_core_term(&left_core, left_term, &context, &source()).unwrap();
        let right = normalize_core_term(&right_core, right_term, &context, &source()).unwrap();

        assert!(alpha_equivalent_terms(&left, &right, &context, &source()).unwrap());
    }

    #[test]
    fn error_nodes_are_canonical_by_diagnostic_id_only() {
        let first = NormalizedTerm::new(NormalizedTermKind::Error(CoreDiagnosticId::new(0)));
        let same = NormalizedTerm::new(NormalizedTermKind::Error(CoreDiagnosticId::new(0)));
        let different = NormalizedTerm::new(NormalizedTermKind::Error(CoreDiagnosticId::new(1)));
        let context = BinderContext::new();

        assert!(alpha_equivalent_terms(&first, &same, &context, &source()).unwrap());
        assert!(!alpha_equivalent_terms(&first, &different, &context, &source()).unwrap());
    }

    #[test]
    fn raw_core_normalization_covers_select_set_enum_generated_and_error_nodes() {
        let x = var(0);
        let mut parts = core_parts();
        let base = push_term(&mut parts, CoreTermKind::Const(symbol("base")));
        let selected = push_term(
            &mut parts,
            CoreTermKind::Select {
                selector: symbol("sel"),
                base,
            },
        );
        let var_x = push_term(&mut parts, CoreTermKind::Var(x));
        let set_enum = push_term(&mut parts, CoreTermKind::SetEnum(vec![selected, var_x]));
        let origin = push_generated(
            &mut parts,
            GeneratedOriginKind::LocalAbbreviation,
            "gen",
            vec![x],
        );
        let generated = push_term(
            &mut parts,
            CoreTermKind::Generated {
                origin,
                args: vec![set_enum],
            },
        );
        let diagnostic = parts.diagnostics.insert(CoreDiagnostic::error(
            CoreDiagnosticClass::UnsupportedLowering,
            "recovered",
            source(),
        ));
        let error = push_term(&mut parts, CoreTermKind::Error(diagnostic));
        let core = CoreIr::try_new(parts).expect("core fixture");
        let context = context_for(&[(x, NormalizedVarSort::Term)]);

        let normalized = normalize_core_term(&core, generated, &context, &source()).unwrap();
        let NormalizedTermKind::Generated { origin, args } = normalized.kind else {
            panic!("expected generated term");
        };
        assert_eq!(origin.params, vec![NormalizedVar::Free(x)]);
        assert!(matches!(args[0].kind, NormalizedTermKind::SetEnum(_)));
        let normalized_error = normalize_core_term(&core, error, &context, &source()).unwrap();
        assert_eq!(
            normalized_error,
            NormalizedTerm::new(NormalizedTermKind::Error(diagnostic))
        );
    }

    #[test]
    fn raw_core_normalization_is_idempotent_across_repeated_runs() {
        let x = var(0);
        let mut parts = core_parts();
        let term_x = push_term(&mut parts, CoreTermKind::Var(x));
        let true_formula = push_formula(&mut parts, CoreFormulaKind::True);
        let type_formula = push_formula(
            &mut parts,
            CoreFormulaKind::TypePred {
                subject: term_x,
                ty: predicate_ty("Nat"),
            },
        );
        let body = push_formula(
            &mut parts,
            CoreFormulaKind::Iff {
                left: true_formula,
                right: type_formula,
            },
        );
        let forall = push_formula(
            &mut parts,
            CoreFormulaKind::Forall {
                binders: vec![core_binder(x, None, "x")],
                body,
            },
        );
        let core = CoreIr::try_new(parts).expect("core fixture");
        let context = context_for(&[(x, NormalizedVarSort::Term)]);

        let first = normalize_core_formula(&core, forall, &context, &source()).unwrap();
        let second = normalize_core_formula(&core, forall, &context, &source()).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            canonical_formula(&first, &context, &source()).unwrap(),
            canonical_formula(&second, &context, &source()).unwrap()
        );
    }

    #[test]
    fn raw_core_normalization_reports_missing_root_rows_as_malformed_evidence() {
        let core = CoreIr::try_new(core_parts()).expect("core fixture");
        let context = BinderContext::new();

        assert!(matches!(
            normalize_core_term(&core, CoreTermId::new(99), &context, &source()),
            Err(BinderDiagnostic {
                class: BinderDiagnosticClass::MalformedEvidence,
                ..
            })
        ));
        assert!(matches!(
            normalize_core_formula(&core, CoreFormulaId::new(99), &context, &source()),
            Err(BinderDiagnostic {
                class: BinderDiagnosticClass::MalformedEvidence,
                ..
            })
        ));
    }

    #[test]
    fn raw_core_normalization_enforces_guard_scope_and_normalizes_guards() {
        let x = var(0);
        let y = var(1);
        let mut parts = core_parts();
        let term_x = push_term(&mut parts, CoreTermKind::Var(x));
        let term_y = push_term(&mut parts, CoreTermKind::Var(y));
        let guard_x = push_formula(
            &mut parts,
            CoreFormulaKind::TypePred {
                subject: term_x,
                ty: predicate_ty("Nat"),
            },
        );
        let guard_y = push_formula(
            &mut parts,
            CoreFormulaKind::Equals {
                left: term_x,
                right: term_y,
            },
        );
        let body = push_formula(
            &mut parts,
            CoreFormulaKind::Equals {
                left: term_x,
                right: term_y,
            },
        );
        let forall = push_formula(
            &mut parts,
            CoreFormulaKind::Forall {
                binders: vec![
                    core_binder(x, Some(guard_x), "display-x"),
                    core_binder(y, Some(guard_y), "display-y"),
                ],
                body,
            },
        );
        let core = CoreIr::try_new(parts).expect("core fixture");
        let context = context_for(&[(x, NormalizedVarSort::Term), (y, NormalizedVarSort::Term)]);
        let normalized = normalize_core_formula(&core, forall, &context, &source()).unwrap();
        validate_normalized_formula(&normalized, &context, &source()).unwrap();
        assert!(normalized.free_variables.is_empty());

        let NormalizedFormulaKind::Forall { binders, body } = normalized.kind else {
            panic!("expected forall");
        };
        assert_eq!(binders[0].frame.source_name.as_deref(), Some("display-x"));
        let guard_x = binders[0].ty_guard.as_ref().expect("guard x");
        assert!(matches!(
            &guard_x.kind,
            NormalizedFormulaKind::TypePred {
                subject,
                ..
            } if subject == &NormalizedTerm::bound(0)
        ));
        let guard_y = binders[1].ty_guard.as_ref().expect("guard y");
        assert_eq!(
            guard_y.as_ref(),
            &equals(NormalizedTerm::bound(1), NormalizedTerm::bound(0))
        );
        assert_eq!(
            body.as_ref(),
            &equals(NormalizedTerm::bound(1), NormalizedTerm::bound(0))
        );
    }

    #[test]
    fn raw_core_normalization_rejects_guard_references_to_later_binders() {
        let x = var(0);
        let y = var(1);
        let mut parts = core_parts();
        let term_y = push_term(&mut parts, CoreTermKind::Var(y));
        let guard_x = push_formula(
            &mut parts,
            CoreFormulaKind::TypePred {
                subject: term_y,
                ty: predicate_ty("Nat"),
            },
        );
        let body = push_formula(&mut parts, CoreFormulaKind::True);
        let forall = push_formula(
            &mut parts,
            CoreFormulaKind::Forall {
                binders: vec![
                    core_binder(x, Some(guard_x), "x"),
                    core_binder(y, None, "y"),
                ],
                body,
            },
        );
        let core = CoreIr::try_new(parts).expect("core fixture");
        let context = context_for(&[(x, NormalizedVarSort::Term), (y, NormalizedVarSort::Term)]);

        assert!(matches!(
            normalize_core_formula(&core, forall, &context, &source()),
            Err(BinderDiagnostic {
                class: BinderDiagnosticClass::MalformedEvidence,
                ..
            })
        ));
    }

    #[test]
    fn raw_core_normalization_rejects_later_binders_in_generated_origin_params() {
        let x = var(0);
        let y = var(1);
        let mut parts = core_parts();
        let origin = push_generated(
            &mut parts,
            GeneratedOriginKind::LocalAbbreviation,
            "leaky",
            vec![y],
        );
        let generated = push_term(
            &mut parts,
            CoreTermKind::Generated {
                origin,
                args: Vec::new(),
            },
        );
        let guard_x = push_formula(
            &mut parts,
            CoreFormulaKind::TypePred {
                subject: generated,
                ty: predicate_ty("Nat"),
            },
        );
        let body = push_formula(&mut parts, CoreFormulaKind::True);
        let forall = push_formula(
            &mut parts,
            CoreFormulaKind::Forall {
                binders: vec![
                    core_binder(x, Some(guard_x), "x"),
                    core_binder(y, None, "y"),
                ],
                body,
            },
        );
        let core = CoreIr::try_new(parts).expect("core fixture");
        let context = context_for(&[(x, NormalizedVarSort::Term), (y, NormalizedVarSort::Term)]);

        assert!(matches!(
            normalize_core_formula(&core, forall, &context, &source()),
            Err(BinderDiagnostic {
                class: BinderDiagnosticClass::MalformedEvidence,
                ..
            })
        ));
    }
}
