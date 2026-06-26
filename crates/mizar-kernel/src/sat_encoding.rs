use std::collections::{BTreeMap, BTreeSet};

use crate::{
    certificate_parser::{ClauseTautologyPolicy, Fingerprint},
    clause::{
        Atom, Clause, ClauseError, ClauseProfile, ClauseValidationContext, Literal, Polarity, Term,
        VariableId,
    },
    formula_evidence::{
        FinalGoalEvidence, Formula, FormulaEvidenceEntry, FormulaEvidenceError,
        FormulaSubstitutionEvidence, GoalPolarity, ParsedKernelEvidence,
        SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID,
    },
    rejection::{
        RejectionCategory, RejectionDetail, RejectionLocation, RejectionRecord, TargetVcFingerprint,
    },
    substitution_checker::Replacement,
};

pub const SAT_PROBLEM_SCHEMA_VERSION: u16 = 1;
pub const SAT_PROBLEM_ENCODING_VERSION: u16 = 1;
pub const ASSERTION_KIND_PREMISE: u8 = 1;
pub const ASSERTION_KIND_SUBSTITUTION_INSTANCE: u8 = 2;
pub const ASSERTION_KIND_FINAL_GOAL: u8 = 3;

const SAT_PROBLEM_DOMAIN_SEPARATOR: &[u8] = b"MIZAR_KERNEL_SAT_PROBLEM\0";
const PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP: u8 = 1;
const REPLACEMENT_ROLE_TERM_ARGUMENT: u8 = 1;
const REPLACEMENT_ROLE_PREDICATE_ARGUMENT: u8 = 2;
const BINDER_CONTEXT_SCHEMA_VERSION: u16 = 1;
const BINDER_FRAME_ENCODED_BYTES: usize = 13;
const BINDER_VARIABLE_ENCODED_BYTES: usize = 4;
const DEFAULT_MAX_ASSERTIONS: usize = 65_536;
const DEFAULT_MAX_SUBSTITUTION_INSTANCES: usize = 32_768;
const DEFAULT_MAX_FORMULA_NODES: usize = 1_000_000;
const DEFAULT_MAX_ATOMS: usize = 262_144;
const DEFAULT_MAX_AUX_VARIABLES: usize = 1_000_000;
const DEFAULT_MAX_CLAUSES: usize = 4_000_000;
const DEFAULT_MAX_LITERALS: usize = 16_000_000;
const DEFAULT_MAX_LITERALS_PER_CLAUSE: usize = 65_536;
const DEFAULT_MAX_CANONICAL_BYTES: usize = 64 * 1024 * 1024;
const DEFAULT_MAX_BINDER_CONTEXT_BYTES: usize = 1_048_576;
const DEFAULT_MAX_BINDER_FRAMES: usize = 65_536;
const DEFAULT_MAX_TERM_ENCODING_BYTES: usize = 1_048_576;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SatEncodingContext {
    pub limits: SatEncodingLimits,
}

impl SatEncodingContext {
    #[must_use]
    pub fn v1() -> Self {
        Self {
            limits: SatEncodingLimits::default(),
        }
    }

    #[must_use]
    pub const fn with_limits(mut self, limits: SatEncodingLimits) -> Self {
        self.limits = limits;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SatEncodingLimits {
    pub max_assertions: usize,
    pub max_substitution_instances: usize,
    pub max_formula_nodes: usize,
    pub max_atoms: usize,
    pub max_aux_variables: usize,
    pub max_clauses: usize,
    pub max_literals: usize,
    pub max_literals_per_clause: usize,
    pub max_canonical_bytes: usize,
    pub max_binder_context_bytes: usize,
    pub max_binder_frames: usize,
    pub max_term_encoding_bytes: usize,
    pub max_term_recursion_depth: usize,
}

impl Default for SatEncodingLimits {
    fn default() -> Self {
        Self {
            max_assertions: DEFAULT_MAX_ASSERTIONS,
            max_substitution_instances: DEFAULT_MAX_SUBSTITUTION_INSTANCES,
            max_formula_nodes: DEFAULT_MAX_FORMULA_NODES,
            max_atoms: DEFAULT_MAX_ATOMS,
            max_aux_variables: DEFAULT_MAX_AUX_VARIABLES,
            max_clauses: DEFAULT_MAX_CLAUSES,
            max_literals: DEFAULT_MAX_LITERALS,
            max_literals_per_clause: DEFAULT_MAX_LITERALS_PER_CLAUSE,
            max_canonical_bytes: DEFAULT_MAX_CANONICAL_BYTES,
            max_binder_context_bytes: DEFAULT_MAX_BINDER_CONTEXT_BYTES,
            max_binder_frames: DEFAULT_MAX_BINDER_FRAMES,
            max_term_encoding_bytes: DEFAULT_MAX_TERM_ENCODING_BYTES,
            max_term_recursion_depth: 64,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SatVariable(pub u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SatLiteral {
    pub variable: SatVariable,
    pub positive: bool,
}

impl SatLiteral {
    #[must_use]
    pub const fn positive(variable: SatVariable) -> Self {
        Self {
            variable,
            positive: true,
        }
    }

    #[must_use]
    pub const fn negative(variable: SatVariable) -> Self {
        Self {
            variable,
            positive: false,
        }
    }

    #[must_use]
    pub const fn negated(self) -> Self {
        Self {
            variable: self.variable,
            positive: !self.positive,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SatClause {
    pub literals: Vec<SatLiteral>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SatAtomVariable {
    pub variable: SatVariable,
    pub atom: Atom,
    pub atom_canonical_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedFormulaAssertion {
    pub assertion_kind: u8,
    pub source_formula_id: Option<u32>,
    pub substitution_id: Option<u32>,
    pub asserted_true: bool,
    pub formula: Formula,
    pub formula_fingerprint: Fingerprint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedSatProblem {
    schema_version: u16,
    encoding_version: u16,
    target_vc: Fingerprint,
    atom_variables: Vec<SatAtomVariable>,
    assertions: Vec<EncodedFormulaAssertion>,
    clauses: Vec<SatClause>,
    canonical_bytes: Vec<u8>,
}

impl EncodedSatProblem {
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    #[must_use]
    pub const fn encoding_version(&self) -> u16 {
        self.encoding_version
    }

    #[must_use]
    pub const fn target_vc(&self) -> &Fingerprint {
        &self.target_vc
    }

    #[must_use]
    pub fn atom_variables(&self) -> &[SatAtomVariable] {
        &self.atom_variables
    }

    #[must_use]
    pub fn assertions(&self) -> &[EncodedFormulaAssertion] {
        &self.assertions
    }

    #[must_use]
    pub fn clauses(&self) -> &[SatClause] {
        &self.clauses
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    #[cfg(test)]
    pub(crate) fn from_test_parts(
        target_vc: Fingerprint,
        atom_variables: Vec<SatAtomVariable>,
        clauses: Vec<SatClause>,
        canonical_bytes: Vec<u8>,
    ) -> Self {
        Self {
            schema_version: SAT_PROBLEM_SCHEMA_VERSION,
            encoding_version: SAT_PROBLEM_ENCODING_VERSION,
            target_vc,
            atom_variables,
            assertions: Vec::new(),
            clauses,
            canonical_bytes,
        }
    }
}

pub type SatEncodingResult<T> = Result<T, Box<RejectionRecord>>;

pub fn encode_formula_evidence(
    evidence: &ParsedKernelEvidence,
    context: &SatEncodingContext,
) -> SatEncodingResult<EncodedSatProblem> {
    let target = TargetVcFingerprint::from_certificate_fingerprint(evidence.target_vc());
    let validation_context = formula_validation_context(evidence, context);

    let mut assertions = premise_assertions(&target, evidence, context)?;
    assertions.extend(instantiate_substitutions(
        &target,
        evidence,
        &validation_context,
        context,
    )?);
    assertions.push(final_goal_assertion(evidence.final_goal()));
    if assertions.len() > context.limits.max_assertions {
        return Err(resource_rejection(
            &target,
            RejectionLocation::new().with_field_path("sat_encoding.assertions"),
        ));
    }
    sort_assertions(&mut assertions, &target)?;

    let atom_variables = assign_atom_variables(&target, &assertions, context)?;
    let atom_lookup = atom_variables
        .iter()
        .map(|entry| (entry.atom_canonical_bytes.clone(), entry.variable))
        .collect::<BTreeMap<_, _>>();
    let atom_count = u32::try_from(atom_variables.len()).map_err(|_| {
        resource_rejection(
            &target,
            RejectionLocation::new().with_field_path("sat_encoding.atom_variables"),
        )
    })?;

    let mut builder = EncodingBuilder::new(&target, context, atom_count, atom_lookup);
    for assertion in &assertions {
        let literal = builder.encode_formula(&assertion.formula, 0)?;
        let asserted_literal = if assertion.asserted_true {
            literal
        } else {
            literal.negated()
        };
        builder.push_clause(
            vec![asserted_literal],
            RejectionLocation::new().with_field_path("sat_encoding.assertion"),
        )?;
    }
    let clauses = builder.finish();

    let canonical_bytes =
        canonical_problem_bytes(evidence.target_vc(), &atom_variables, &assertions, &clauses)
            .map_err(|detail| {
                rejection(
                    &target,
                    RejectionCategory::KernelRejection,
                    detail,
                    RejectionLocation::new().with_field_path("sat_encoding.canonical_bytes"),
                )
            })?;
    if canonical_bytes.len() > context.limits.max_canonical_bytes {
        return Err(resource_rejection(
            &target,
            RejectionLocation::new().with_field_path("sat_encoding.canonical_bytes"),
        ));
    }

    Ok(EncodedSatProblem {
        schema_version: SAT_PROBLEM_SCHEMA_VERSION,
        encoding_version: SAT_PROBLEM_ENCODING_VERSION,
        target_vc: evidence.target_vc().clone(),
        atom_variables,
        assertions,
        clauses,
        canonical_bytes,
    })
}

fn premise_assertions(
    target: &TargetVcFingerprint,
    evidence: &ParsedKernelEvidence,
    context: &SatEncodingContext,
) -> SatEncodingResult<Vec<EncodedFormulaAssertion>> {
    if evidence.formulas().len() > context.limits.max_assertions {
        return Err(resource_rejection(
            target,
            RejectionLocation::new().with_field_path("formula_evidence"),
        ));
    }
    evidence
        .formulas()
        .iter()
        .map(|entry| {
            Ok(EncodedFormulaAssertion {
                assertion_kind: ASSERTION_KIND_PREMISE,
                source_formula_id: Some(entry.formula_id),
                substitution_id: None,
                asserted_true: true,
                formula: entry.formula.clone(),
                formula_fingerprint: entry.formula_fingerprint.clone(),
            })
        })
        .collect()
}

fn instantiate_substitutions(
    target: &TargetVcFingerprint,
    evidence: &ParsedKernelEvidence,
    validation_context: &ClauseValidationContext,
    context: &SatEncodingContext,
) -> SatEncodingResult<Vec<EncodedFormulaAssertion>> {
    if evidence.substitutions().len() > context.limits.max_substitution_instances {
        return Err(resource_rejection(
            target,
            RejectionLocation::new().with_field_path("substitutions"),
        ));
    }
    let formulas = evidence
        .formulas()
        .iter()
        .map(|entry| (entry.formula_id, entry))
        .collect::<BTreeMap<_, _>>();
    let mut assertions = Vec::with_capacity(evidence.substitutions().len());
    for substitution in evidence.substitutions() {
        let Some(source) = formulas.get(&substitution.source_formula_id).copied() else {
            return Err(invalid_substitution(
                target,
                substitution.substitution_id,
                "substitution.source_formula_id",
            ));
        };
        let formula =
            instantiate_formula(target, source, substitution, validation_context, context)?;
        let formula_fingerprint = formula_fingerprint(&formula).map_err(|detail| {
            rejection(
                target,
                RejectionCategory::KernelRejection,
                detail,
                RejectionLocation::new()
                    .with_substitution_id(substitution.substitution_id)
                    .with_field_path("substitution.instantiated_formula"),
            )
        })?;
        assertions.push(EncodedFormulaAssertion {
            assertion_kind: ASSERTION_KIND_SUBSTITUTION_INSTANCE,
            source_formula_id: Some(source.formula_id),
            substitution_id: Some(substitution.substitution_id),
            asserted_true: true,
            formula,
            formula_fingerprint,
        });
    }
    Ok(assertions)
}

fn instantiate_formula(
    target: &TargetVcFingerprint,
    source: &FormulaEvidenceEntry,
    substitution: &FormulaSubstitutionEvidence,
    validation_context: &ClauseValidationContext,
    context: &SatEncodingContext,
) -> SatEncodingResult<Formula> {
    validate_supported_payload_shape(target, substitution)?;
    let binder_context = BinderContext::decode(
        target,
        substitution,
        validation_context.canonical_variable_ids.clone(),
        context,
    )?;
    let replacements = replacement_map(target, substitution)?;
    validate_binder_context_usage(target, source, substitution, &binder_context, &replacements)?;
    let mut replay = FormulaReplay {
        target,
        substitution_id: substitution.substitution_id,
        validation_context,
        binder_context,
        replacements,
        replacement_actual_variables: BTreeMap::new(),
        applied_formals: BTreeSet::new(),
        active_bound_variables: Vec::new(),
    };
    replay.cache_actual_variables()?;
    let formula = replay.instantiate_formula(&source.formula, 0)?;
    for formal in replay.replacements.keys() {
        if !replay.applied_formals.contains(formal) {
            return Err(invalid_substitution(
                target,
                substitution.substitution_id,
                "substitution.payload.formal_variable_id",
            ));
        }
    }
    validate_formula_atoms(
        target,
        substitution.substitution_id,
        &formula,
        validation_context,
        context,
        0,
        &mut 0,
    )?;
    Ok(formula)
}

fn validate_supported_payload_shape(
    target: &TargetVcFingerprint,
    substitution: &FormulaSubstitutionEvidence,
) -> SatEncodingResult<()> {
    let payload = &substitution.payload;
    if payload.owner_substitution_id != substitution.substitution_id {
        return Err(invalid_substitution(
            target,
            substitution.substitution_id,
            "substitution.payload.owner_substitution_id",
        ));
    }
    if payload.payload_kind != PAYLOAD_KIND_FORMAL_TO_ACTUAL_MAP {
        return Err(invalid_substitution(
            target,
            substitution.substitution_id,
            "substitution.payload.payload_kind",
        ));
    }
    if !payload.rewrite_path.segments.is_empty() {
        return Err(invalid_substitution(
            target,
            substitution.substitution_id,
            "substitution.payload.rewrite_path",
        ));
    }
    if !substitution.freshness_witnesses.is_empty() {
        return Err(invalid_substitution(
            target,
            substitution.substitution_id,
            "substitution.freshness_witnesses",
        ));
    }
    if !substitution.free_variable_constraints.is_empty() {
        return Err(invalid_substitution(
            target,
            substitution.substitution_id,
            "substitution.free_variable_constraints",
        ));
    }
    for replacement in &payload.replacements {
        if !matches!(
            replacement.replacement_role,
            REPLACEMENT_ROLE_TERM_ARGUMENT | REPLACEMENT_ROLE_PREDICATE_ARGUMENT
        ) {
            return Err(invalid_substitution(
                target,
                substitution.substitution_id,
                "substitution.payload.replacement_role",
            ));
        }
    }
    Ok(())
}

fn validate_binder_context_usage(
    target: &TargetVcFingerprint,
    source: &FormulaEvidenceEntry,
    substitution: &FormulaSubstitutionEvidence,
    binder_context: &BinderContext,
    replacements: &BTreeMap<VariableId, Term>,
) -> SatEncodingResult<()> {
    let mut used_binders = BTreeSet::new();
    collect_formula_binder_ids(
        target,
        substitution,
        &source.formula,
        binder_context,
        &mut used_binders,
    )?;
    for actual in replacements.values() {
        let mut seen_binders = BTreeSet::new();
        let mut previous_canonical_index = None;
        validate_term_binders(
            target,
            substitution,
            actual,
            binder_context,
            "substitution.payload.actual_term",
            &mut seen_binders,
            &mut previous_canonical_index,
        )?;
        used_binders.extend(seen_binders);
    }
    if used_binders != binder_context.binder_ids() {
        return Err(invalid_substitution(
            target,
            substitution.substitution_id,
            "substitution.binder_context",
        ));
    }
    Ok(())
}

fn replacement_map(
    target: &TargetVcFingerprint,
    substitution: &FormulaSubstitutionEvidence,
) -> SatEncodingResult<BTreeMap<VariableId, Term>> {
    let mut replacements = BTreeMap::new();
    for Replacement {
        formal_variable_id,
        actual_term,
        ..
    } in &substitution.payload.replacements
    {
        if replacements
            .insert(*formal_variable_id, actual_term.clone())
            .is_some()
        {
            return Err(invalid_substitution(
                target,
                substitution.substitution_id,
                "substitution.payload.formal_variable_id",
            ));
        }
    }
    Ok(replacements)
}

fn final_goal_assertion(goal: &FinalGoalEvidence) -> EncodedFormulaAssertion {
    EncodedFormulaAssertion {
        assertion_kind: ASSERTION_KIND_FINAL_GOAL,
        source_formula_id: None,
        substitution_id: None,
        asserted_true: matches!(goal.polarity, GoalPolarity::AssertTrueForConsistency),
        formula: goal.formula.clone(),
        formula_fingerprint: goal.formula_fingerprint.clone(),
    }
}

fn sort_assertions(
    assertions: &mut [EncodedFormulaAssertion],
    target: &TargetVcFingerprint,
) -> SatEncodingResult<()> {
    let mut keyed = Vec::with_capacity(assertions.len());
    for assertion in assertions.iter() {
        keyed.push((assertion_sort_key(assertion, target)?, assertion.clone()));
    }
    keyed.sort_by(|left, right| left.0.cmp(&right.0));
    for (slot, (_, assertion)) in assertions.iter_mut().zip(keyed) {
        *slot = assertion;
    }
    Ok(())
}

fn assertion_sort_key(
    assertion: &EncodedFormulaAssertion,
    target: &TargetVcFingerprint,
) -> SatEncodingResult<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.push(assertion.assertion_kind);
    bytes.push(u8::from(assertion.asserted_true));
    write_option_u32(assertion.source_formula_id, &mut bytes);
    write_option_u32(assertion.substitution_id, &mut bytes);
    bytes.extend(
        fingerprint_bytes(&assertion.formula_fingerprint).map_err(|detail| {
            rejection(
                target,
                RejectionCategory::KernelRejection,
                detail,
                RejectionLocation::new().with_field_path("sat_encoding.assertion.fingerprint"),
            )
        })?,
    );
    bytes.extend(assertion.formula.canonical_hash_input().map_err(|error| {
        formula_error_rejection(
            target,
            error,
            RejectionLocation::new().with_field_path("sat_encoding.assertion.formula"),
        )
    })?);
    Ok(bytes)
}

fn assign_atom_variables(
    target: &TargetVcFingerprint,
    assertions: &[EncodedFormulaAssertion],
    context: &SatEncodingContext,
) -> SatEncodingResult<Vec<SatAtomVariable>> {
    let mut atoms = BTreeMap::new();
    let mut node_count = 0usize;
    for assertion in assertions {
        collect_atoms(
            target,
            &assertion.formula,
            context,
            &mut node_count,
            &mut atoms,
        )?;
    }
    if atoms.len() > context.limits.max_atoms {
        return Err(resource_rejection(
            target,
            RejectionLocation::new().with_field_path("sat_encoding.atom_variables"),
        ));
    }
    let mut variables = Vec::with_capacity(atoms.len());
    for (index, (atom_bytes, atom)) in atoms.into_iter().enumerate() {
        let variable = SatVariable(u32::try_from(index + 1).map_err(|_| {
            resource_rejection(
                target,
                RejectionLocation::new().with_field_path("sat_encoding.atom_variables"),
            )
        })?);
        variables.push(SatAtomVariable {
            variable,
            atom,
            atom_canonical_bytes: atom_bytes,
        });
    }
    Ok(variables)
}

fn collect_atoms(
    target: &TargetVcFingerprint,
    formula: &Formula,
    context: &SatEncodingContext,
    node_count: &mut usize,
    atoms: &mut BTreeMap<Vec<u8>, Atom>,
) -> SatEncodingResult<()> {
    *node_count = node_count.checked_add(1).ok_or_else(|| {
        resource_rejection(
            target,
            RejectionLocation::new().with_field_path("sat_encoding.formula_nodes"),
        )
    })?;
    if *node_count > context.limits.max_formula_nodes {
        return Err(resource_rejection(
            target,
            RejectionLocation::new().with_field_path("sat_encoding.formula_nodes"),
        ));
    }
    match formula {
        Formula::Atom(atom) => {
            let bytes = atom.canonical_bytes().map_err(|error| {
                clause_error_rejection(
                    target,
                    error,
                    RejectionLocation::new().with_field_path("sat_encoding.atom"),
                )
            })?;
            atoms.entry(bytes).or_insert_with(|| atom.clone());
        }
        Formula::Not(child) => collect_atoms(target, child, context, node_count, atoms)?,
        Formula::And(children) | Formula::Or(children) => {
            for child in children {
                collect_atoms(target, child, context, node_count, atoms)?;
            }
        }
    }
    Ok(())
}

struct EncodingBuilder<'a> {
    target: &'a TargetVcFingerprint,
    context: &'a SatEncodingContext,
    next_aux_variable: u32,
    atom_lookup: BTreeMap<Vec<u8>, SatVariable>,
    clauses: Vec<SatClause>,
    total_literals: usize,
}

impl<'a> EncodingBuilder<'a> {
    fn new(
        target: &'a TargetVcFingerprint,
        context: &'a SatEncodingContext,
        atom_count: u32,
        atom_lookup: BTreeMap<Vec<u8>, SatVariable>,
    ) -> Self {
        Self {
            target,
            context,
            next_aux_variable: atom_count.saturating_add(1),
            atom_lookup,
            clauses: Vec::new(),
            total_literals: 0,
        }
    }

    fn encode_formula(&mut self, formula: &Formula, depth: usize) -> SatEncodingResult<SatLiteral> {
        match formula {
            Formula::Atom(atom) => {
                let bytes = atom.canonical_bytes().map_err(|error| {
                    clause_error_rejection(
                        self.target,
                        error,
                        RejectionLocation::new().with_field_path("sat_encoding.atom"),
                    )
                })?;
                let Some(variable) = self.atom_lookup.get(&bytes).copied() else {
                    return Err(rejection(
                        self.target,
                        RejectionCategory::KernelRejection,
                        RejectionDetail::InvalidSatRefutation,
                        RejectionLocation::new().with_field_path("sat_encoding.atom_variable"),
                    ));
                };
                Ok(SatLiteral::positive(variable))
            }
            Formula::Not(child) => Ok(self
                .encode_formula(
                    child,
                    next_depth(self.target, depth, "sat_encoding.formula_depth")?,
                )?
                .negated()),
            Formula::And(children) => self.encode_and(children, depth),
            Formula::Or(children) => self.encode_or(children, depth),
        }
    }

    fn encode_and(&mut self, children: &[Formula], depth: usize) -> SatEncodingResult<SatLiteral> {
        let output = SatLiteral::positive(self.allocate_aux()?);
        let mut child_literals = Vec::with_capacity(children.len());
        for child in children {
            let child_literal = self.encode_formula(
                child,
                next_depth(self.target, depth, "sat_encoding.formula_depth")?,
            )?;
            self.push_clause(
                vec![output.negated(), child_literal],
                RejectionLocation::new().with_field_path("sat_encoding.and.forward"),
            )?;
            child_literals.push(child_literal);
        }
        let mut reverse = Vec::with_capacity(child_literals.len() + 1);
        reverse.push(output);
        reverse.extend(child_literals.into_iter().map(SatLiteral::negated));
        self.push_clause(
            reverse,
            RejectionLocation::new().with_field_path("sat_encoding.and.reverse"),
        )?;
        Ok(output)
    }

    fn encode_or(&mut self, children: &[Formula], depth: usize) -> SatEncodingResult<SatLiteral> {
        let output = SatLiteral::positive(self.allocate_aux()?);
        let mut child_literals = Vec::with_capacity(children.len());
        for child in children {
            let child_literal = self.encode_formula(
                child,
                next_depth(self.target, depth, "sat_encoding.formula_depth")?,
            )?;
            self.push_clause(
                vec![output, child_literal.negated()],
                RejectionLocation::new().with_field_path("sat_encoding.or.forward"),
            )?;
            child_literals.push(child_literal);
        }
        let mut reverse = Vec::with_capacity(child_literals.len() + 1);
        reverse.push(output.negated());
        reverse.extend(child_literals);
        self.push_clause(
            reverse,
            RejectionLocation::new().with_field_path("sat_encoding.or.reverse"),
        )?;
        Ok(output)
    }

    fn allocate_aux(&mut self) -> SatEncodingResult<SatVariable> {
        let atom_count = u32::try_from(self.atom_lookup.len()).map_err(|_| {
            resource_rejection(
                self.target,
                RejectionLocation::new().with_field_path("sat_encoding.aux_variable"),
            )
        })?;
        let aux_index = self
            .next_aux_variable
            .checked_sub(atom_count)
            .and_then(|value| value.checked_sub(1))
            .ok_or_else(|| {
                resource_rejection(
                    self.target,
                    RejectionLocation::new().with_field_path("sat_encoding.aux_variable"),
                )
            })?;
        if aux_index as usize >= self.context.limits.max_aux_variables {
            return Err(resource_rejection(
                self.target,
                RejectionLocation::new().with_field_path("sat_encoding.aux_variable"),
            ));
        }
        let variable = SatVariable(self.next_aux_variable);
        self.next_aux_variable = self.next_aux_variable.checked_add(1).ok_or_else(|| {
            resource_rejection(
                self.target,
                RejectionLocation::new().with_field_path("sat_encoding.aux_variable"),
            )
        })?;
        Ok(variable)
    }

    fn push_clause(
        &mut self,
        mut literals: Vec<SatLiteral>,
        location: RejectionLocation,
    ) -> SatEncodingResult<()> {
        if literals.is_empty() || literals.len() > self.context.limits.max_literals_per_clause {
            return Err(resource_rejection(self.target, location));
        }
        literals.sort();
        literals.dedup();
        self.total_literals = self
            .total_literals
            .checked_add(literals.len())
            .ok_or_else(|| resource_rejection(self.target, location.clone()))?;
        if self.total_literals > self.context.limits.max_literals {
            return Err(resource_rejection(self.target, location));
        }
        if self.clauses.len() >= self.context.limits.max_clauses {
            return Err(resource_rejection(self.target, location));
        }
        self.clauses.push(SatClause { literals });
        Ok(())
    }

    fn finish(self) -> Vec<SatClause> {
        self.clauses
    }
}

struct FormulaReplay<'a> {
    target: &'a TargetVcFingerprint,
    substitution_id: u32,
    validation_context: &'a ClauseValidationContext,
    binder_context: BinderContext,
    replacements: BTreeMap<VariableId, Term>,
    replacement_actual_variables: BTreeMap<VariableId, BTreeSet<VariableId>>,
    applied_formals: BTreeSet<VariableId>,
    active_bound_variables: Vec<VariableId>,
}

impl FormulaReplay<'_> {
    fn cache_actual_variables(&mut self) -> SatEncodingResult<()> {
        for (formal, actual) in &self.replacements {
            actual
                .validate_for_kernel(self.validation_context)
                .map_err(|error| {
                    clause_error_rejection(
                        self.target,
                        error,
                        RejectionLocation::new()
                            .with_substitution_id(self.substitution_id)
                            .with_field_path("substitution.payload.actual_term"),
                    )
                })?;
            let mut variables = BTreeSet::new();
            collect_term_free_variables(
                self.target,
                self.substitution_id,
                actual,
                &self.binder_context,
                &mut Vec::new(),
                &mut variables,
            )?;
            self.replacement_actual_variables.insert(*formal, variables);
        }
        Ok(())
    }

    fn instantiate_formula(
        &mut self,
        formula: &Formula,
        depth: usize,
    ) -> SatEncodingResult<Formula> {
        match formula {
            Formula::Atom(atom) => Ok(Formula::Atom(self.instantiate_atom(atom, depth)?)),
            Formula::Not(child) => Ok(Formula::Not(Box::new(self.instantiate_formula(
                child,
                next_depth(self.target, depth, "substitution.formula_depth")?,
            )?))),
            Formula::And(children) => {
                let mut instantiated = Vec::with_capacity(children.len());
                for child in children {
                    instantiated.push(self.instantiate_formula(
                        child,
                        next_depth(self.target, depth, "substitution.formula_depth")?,
                    )?);
                }
                Ok(Formula::And(instantiated))
            }
            Formula::Or(children) => {
                let mut instantiated = Vec::with_capacity(children.len());
                for child in children {
                    instantiated.push(self.instantiate_formula(
                        child,
                        next_depth(self.target, depth, "substitution.formula_depth")?,
                    )?);
                }
                Ok(Formula::Or(instantiated))
            }
        }
    }

    fn instantiate_atom(&mut self, atom: &Atom, depth: usize) -> SatEncodingResult<Atom> {
        let mut arguments = Vec::with_capacity(atom.arguments.len());
        for argument in &atom.arguments {
            arguments.push(self.instantiate_term(
                argument,
                next_depth(self.target, depth, "substitution.term_depth")?,
            )?);
        }
        Ok(Atom::with_arity(atom.symbol, atom.arity, arguments))
    }

    fn instantiate_term(&mut self, term: &Term, depth: usize) -> SatEncodingResult<Term> {
        match term {
            Term::Variable(variable) => {
                if self.active_bound_variables.contains(variable) {
                    return Ok(Term::Variable(*variable));
                }
                if let Some(actual) = self.replacements.get(variable) {
                    self.applied_formals.insert(*variable);
                    Ok(actual.clone())
                } else {
                    Ok(Term::Variable(*variable))
                }
            }
            Term::Application { symbol, arguments } => {
                let mut instantiated = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    instantiated.push(self.instantiate_term(
                        argument,
                        next_depth(self.target, depth, "substitution.term_depth")?,
                    )?);
                }
                Ok(Term::Application {
                    symbol: *symbol,
                    arguments: instantiated,
                })
            }
            Term::BinderNormalized { binder_id, body } => {
                let Some(bound_variable) = self.binder_context.variable_for(*binder_id) else {
                    return Err(invalid_substitution(
                        self.target,
                        self.substitution_id,
                        "substitution.binder_context",
                    ));
                };
                self.active_bound_variables.push(bound_variable);
                let mut applicable_formals = BTreeSet::new();
                self.collect_applicable_formals(body, &mut applicable_formals)?;
                let captures = applicable_formals.iter().any(|formal| {
                    self.replacement_actual_variables
                        .get(formal)
                        .is_some_and(|variables| variables.contains(&bound_variable))
                });
                if captures {
                    self.active_bound_variables.pop();
                    return Err(invalid_substitution(
                        self.target,
                        self.substitution_id,
                        "substitution.capture",
                    ));
                }
                let result = self.instantiate_term(
                    body,
                    next_depth(self.target, depth, "substitution.term_depth")?,
                );
                self.active_bound_variables.pop();
                Ok(Term::BinderNormalized {
                    binder_id: *binder_id,
                    body: Box::new(result?),
                })
            }
            Term::Malformed => Err(invalid_substitution(
                self.target,
                self.substitution_id,
                "substitution.term",
            )),
        }
    }

    fn collect_applicable_formals(
        &mut self,
        term: &Term,
        formals: &mut BTreeSet<VariableId>,
    ) -> SatEncodingResult<()> {
        match term {
            Term::Variable(variable) => {
                if !self.active_bound_variables.contains(variable)
                    && self.replacements.contains_key(variable)
                {
                    formals.insert(*variable);
                }
                Ok(())
            }
            Term::Application { arguments, .. } => {
                for argument in arguments {
                    self.collect_applicable_formals(argument, formals)?;
                }
                Ok(())
            }
            Term::BinderNormalized { binder_id, body } => {
                let Some(bound_variable) = self.binder_context.variable_for(*binder_id) else {
                    return Err(invalid_substitution(
                        self.target,
                        self.substitution_id,
                        "substitution.binder_context",
                    ));
                };
                self.active_bound_variables.push(bound_variable);
                let result = self.collect_applicable_formals(body, formals);
                self.active_bound_variables.pop();
                result
            }
            Term::Malformed => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct BinderContext {
    frames: BTreeMap<u32, BinderFrame>,
}

#[derive(Clone, Debug)]
struct BinderFrame {
    canonical_index: u32,
    variable_id: VariableId,
}

impl BinderContext {
    fn decode(
        target: &TargetVcFingerprint,
        substitution: &FormulaSubstitutionEvidence,
        canonical_variables: BTreeSet<VariableId>,
        context: &SatEncodingContext,
    ) -> SatEncodingResult<Self> {
        let bytes = &substitution.binder_context_encoding;
        if bytes.len() > context.limits.max_binder_context_bytes {
            return Err(resource_rejection(
                target,
                RejectionLocation::new()
                    .with_substitution_id(substitution.substitution_id)
                    .with_field_path("substitution.binder_context"),
            ));
        }
        let mut reader = BinderReader::new(bytes);
        let Some(schema_version) = reader.read_u16() else {
            return Err(invalid_substitution(
                target,
                substitution.substitution_id,
                "substitution.binder_context",
            ));
        };
        if schema_version != BINDER_CONTEXT_SCHEMA_VERSION {
            return Err(invalid_substitution(
                target,
                substitution.substitution_id,
                "substitution.binder_context.schema_version",
            ));
        }
        let frame_count = read_binder_count(
            target,
            substitution,
            &mut reader,
            context,
            "substitution.binder_context.frames",
        )?;
        ensure_remaining(
            target,
            substitution,
            &reader,
            frame_count,
            BINDER_FRAME_ENCODED_BYTES,
            "substitution.binder_context.frames",
        )?;
        let mut frames = BTreeMap::new();
        let mut seen_binders = BTreeSet::new();
        let mut seen_variables = BTreeSet::new();
        for expected_index in 0..frame_count {
            let Some(binder_id) = reader.read_u32() else {
                return Err(invalid_substitution(
                    target,
                    substitution.substitution_id,
                    "substitution.binder_context.frames",
                ));
            };
            let Some(canonical_index) = reader.read_u32() else {
                return Err(invalid_substitution(
                    target,
                    substitution.substitution_id,
                    "substitution.binder_context.frames",
                ));
            };
            let Some(variable_raw) = reader.read_u32() else {
                return Err(invalid_substitution(
                    target,
                    substitution.substitution_id,
                    "substitution.binder_context.frames",
                ));
            };
            let Some(binder_role) = reader.read_u8() else {
                return Err(invalid_substitution(
                    target,
                    substitution.substitution_id,
                    "substitution.binder_context.frames",
                ));
            };
            let variable_id = VariableId(variable_raw);
            if !(1..=5).contains(&binder_role)
                || !seen_binders.insert(binder_id)
                || !seen_variables.insert(variable_id)
                || !canonical_variables.contains(&variable_id)
            {
                return Err(invalid_substitution(
                    target,
                    substitution.substitution_id,
                    "substitution.binder_context.frames",
                ));
            }
            let expected_index = u32::try_from(expected_index).map_err(|_| {
                resource_rejection(
                    target,
                    RejectionLocation::new()
                        .with_substitution_id(substitution.substitution_id)
                        .with_field_path("substitution.binder_context.frames"),
                )
            })?;
            if canonical_index != expected_index {
                return Err(invalid_substitution(
                    target,
                    substitution.substitution_id,
                    "substitution.binder_context.frames",
                ));
            }
            frames.insert(
                binder_id,
                BinderFrame {
                    canonical_index,
                    variable_id,
                },
            );
        }
        validate_variable_list(
            target,
            substitution,
            &mut reader,
            &canonical_variables,
            context,
            "substitution.binder_context.free_variables",
        )?;
        validate_variable_list(
            target,
            substitution,
            &mut reader,
            &canonical_variables,
            context,
            "substitution.binder_context.schematic_variables",
        )?;
        if !reader.is_finished() {
            return Err(invalid_substitution(
                target,
                substitution.substitution_id,
                "substitution.binder_context.trailing_bytes",
            ));
        }
        Ok(Self { frames })
    }

    fn variable_for(&self, binder_id: u32) -> Option<VariableId> {
        self.frames.get(&binder_id).map(|frame| frame.variable_id)
    }

    fn frame_for(&self, binder_id: u32) -> Option<&BinderFrame> {
        self.frames.get(&binder_id)
    }

    fn binder_ids(&self) -> BTreeSet<u32> {
        self.frames.keys().copied().collect()
    }
}

struct BinderReader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> BinderReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    const fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.cursor)
    }

    fn read_u8(&mut self) -> Option<u8> {
        let byte = *self.bytes.get(self.cursor)?;
        self.cursor += 1;
        Some(byte)
    }

    fn read_u16(&mut self) -> Option<u16> {
        let bytes = self.read_exact(2)?;
        Some(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32(&mut self) -> Option<u32> {
        let bytes = self.read_exact(4)?;
        Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_exact(&mut self, len: usize) -> Option<&'a [u8]> {
        let end = self.cursor.checked_add(len)?;
        let bytes = self.bytes.get(self.cursor..end)?;
        self.cursor = end;
        Some(bytes)
    }

    const fn is_finished(&self) -> bool {
        self.cursor == self.bytes.len()
    }
}

fn read_binder_count(
    target: &TargetVcFingerprint,
    substitution: &FormulaSubstitutionEvidence,
    reader: &mut BinderReader<'_>,
    context: &SatEncodingContext,
    field: &'static str,
) -> SatEncodingResult<usize> {
    let Some(count) = reader.read_u32() else {
        return Err(invalid_substitution(
            target,
            substitution.substitution_id,
            field,
        ));
    };
    let count = count as usize;
    if count > context.limits.max_binder_frames {
        return Err(resource_rejection(
            target,
            RejectionLocation::new()
                .with_substitution_id(substitution.substitution_id)
                .with_field_path(field),
        ));
    }
    Ok(count)
}

fn ensure_remaining(
    target: &TargetVcFingerprint,
    substitution: &FormulaSubstitutionEvidence,
    reader: &BinderReader<'_>,
    count: usize,
    item_bytes: usize,
    field: &'static str,
) -> SatEncodingResult<()> {
    let required = count.checked_mul(item_bytes).ok_or_else(|| {
        resource_rejection(
            target,
            RejectionLocation::new()
                .with_substitution_id(substitution.substitution_id)
                .with_field_path(field),
        )
    })?;
    if required > reader.remaining() {
        return Err(invalid_substitution(
            target,
            substitution.substitution_id,
            field,
        ));
    }
    Ok(())
}

fn validate_variable_list(
    target: &TargetVcFingerprint,
    substitution: &FormulaSubstitutionEvidence,
    reader: &mut BinderReader<'_>,
    canonical_variables: &BTreeSet<VariableId>,
    context: &SatEncodingContext,
    field: &'static str,
) -> SatEncodingResult<()> {
    let count = read_binder_count(target, substitution, reader, context, field)?;
    ensure_remaining(
        target,
        substitution,
        reader,
        count,
        BINDER_VARIABLE_ENCODED_BYTES,
        field,
    )?;
    let mut previous = None;
    for _ in 0..count {
        let Some(variable_raw) = reader.read_u32() else {
            return Err(invalid_substitution(
                target,
                substitution.substitution_id,
                field,
            ));
        };
        let variable = VariableId(variable_raw);
        if previous.is_some_and(|previous| previous >= variable)
            || !canonical_variables.contains(&variable)
        {
            return Err(invalid_substitution(
                target,
                substitution.substitution_id,
                field,
            ));
        }
        previous = Some(variable);
    }
    Ok(())
}

fn validate_formula_atoms(
    target: &TargetVcFingerprint,
    substitution_id: u32,
    formula: &Formula,
    validation_context: &ClauseValidationContext,
    context: &SatEncodingContext,
    depth: usize,
    node_count: &mut usize,
) -> SatEncodingResult<()> {
    *node_count = node_count.checked_add(1).ok_or_else(|| {
        resource_rejection(
            target,
            RejectionLocation::new()
                .with_substitution_id(substitution_id)
                .with_field_path("substitution.instantiated_formula"),
        )
    })?;
    if *node_count > context.limits.max_formula_nodes {
        return Err(resource_rejection(
            target,
            RejectionLocation::new()
                .with_substitution_id(substitution_id)
                .with_field_path("substitution.instantiated_formula"),
        ));
    }
    match formula {
        Formula::Atom(atom) => Clause::normalize(
            vec![Literal::new(Polarity::Positive, atom.clone())],
            validation_context,
        )
        .map(|_| ())
        .map_err(|error| {
            clause_error_rejection(
                target,
                error,
                RejectionLocation::new()
                    .with_substitution_id(substitution_id)
                    .with_field_path("substitution.instantiated_formula"),
            )
        }),
        Formula::Not(child) => validate_formula_atoms(
            target,
            substitution_id,
            child,
            validation_context,
            context,
            next_depth(target, depth, "substitution.instantiated_formula")?,
            node_count,
        ),
        Formula::And(children) | Formula::Or(children) => {
            for child in children {
                validate_formula_atoms(
                    target,
                    substitution_id,
                    child,
                    validation_context,
                    context,
                    next_depth(target, depth, "substitution.instantiated_formula")?,
                    node_count,
                )?;
            }
            Ok(())
        }
    }
}

fn formula_validation_context(
    evidence: &ParsedKernelEvidence,
    context: &SatEncodingContext,
) -> ClauseValidationContext {
    let clause_profile = ClauseProfile::new(
        evidence.kernel_profile().clause_schema_version,
        evidence.kernel_profile().clause_encoding_version,
        match evidence.kernel_profile().clause_tautology_policy {
            ClauseTautologyPolicy::Reject => crate::clause::TautologyPolicy::Reject,
            ClauseTautologyPolicy::Marker => crate::clause::TautologyPolicy::Marker,
        },
    );
    let mut clause_context = ClauseValidationContext::new(clause_profile)
        .with_limits(usize::MAX, context.limits.max_term_encoding_bytes)
        .with_max_term_recursion_depth(context.limits.max_term_recursion_depth);
    for symbol in evidence.symbol_manifest() {
        clause_context = clause_context.with_known_symbol(symbol.symbol);
    }
    for variable in evidence.variable_manifest() {
        clause_context = clause_context.with_canonical_variable(variable.variable_id);
    }
    clause_context
}

fn collect_formula_binder_ids(
    target: &TargetVcFingerprint,
    substitution: &FormulaSubstitutionEvidence,
    formula: &Formula,
    binder_context: &BinderContext,
    used_binders: &mut BTreeSet<u32>,
) -> SatEncodingResult<()> {
    match formula {
        Formula::Atom(atom) => {
            for argument in &atom.arguments {
                let mut seen_binders = BTreeSet::new();
                let mut previous_canonical_index = None;
                validate_term_binders(
                    target,
                    substitution,
                    argument,
                    binder_context,
                    "substitution.binder_context",
                    &mut seen_binders,
                    &mut previous_canonical_index,
                )?;
                used_binders.extend(seen_binders);
            }
            Ok(())
        }
        Formula::Not(child) => {
            collect_formula_binder_ids(target, substitution, child, binder_context, used_binders)
        }
        Formula::And(children) | Formula::Or(children) => {
            for child in children {
                collect_formula_binder_ids(
                    target,
                    substitution,
                    child,
                    binder_context,
                    used_binders,
                )?;
            }
            Ok(())
        }
    }
}

fn validate_term_binders(
    target: &TargetVcFingerprint,
    substitution: &FormulaSubstitutionEvidence,
    term: &Term,
    binder_context: &BinderContext,
    field_path: &'static str,
    seen_binders: &mut BTreeSet<u32>,
    previous_canonical_index: &mut Option<u32>,
) -> SatEncodingResult<()> {
    match term {
        Term::Variable(_) => Ok(()),
        Term::Application { arguments, .. } => {
            for argument in arguments {
                validate_term_binders(
                    target,
                    substitution,
                    argument,
                    binder_context,
                    field_path,
                    seen_binders,
                    previous_canonical_index,
                )?;
            }
            Ok(())
        }
        Term::BinderNormalized { binder_id, body } => {
            let Some(frame) = binder_context.frame_for(*binder_id) else {
                return Err(invalid_substitution(
                    target,
                    substitution.substitution_id,
                    field_path,
                ));
            };
            if !seen_binders.insert(*binder_id) {
                return Err(invalid_substitution(
                    target,
                    substitution.substitution_id,
                    field_path,
                ));
            }
            if previous_canonical_index.is_some_and(|previous| previous >= frame.canonical_index) {
                return Err(invalid_substitution(
                    target,
                    substitution.substitution_id,
                    field_path,
                ));
            }
            *previous_canonical_index = Some(frame.canonical_index);
            validate_term_binders(
                target,
                substitution,
                body,
                binder_context,
                field_path,
                seen_binders,
                previous_canonical_index,
            )
        }
        Term::Malformed => Err(invalid_substitution(
            target,
            substitution.substitution_id,
            field_path,
        )),
    }
}

fn collect_term_free_variables(
    target: &TargetVcFingerprint,
    substitution_id: u32,
    term: &Term,
    binder_context: &BinderContext,
    active_bound_variables: &mut Vec<VariableId>,
    variables: &mut BTreeSet<VariableId>,
) -> SatEncodingResult<()> {
    match term {
        Term::Variable(variable) => {
            if !active_bound_variables.contains(variable) {
                variables.insert(*variable);
            }
            Ok(())
        }
        Term::Application { arguments, .. } => {
            for argument in arguments {
                collect_term_free_variables(
                    target,
                    substitution_id,
                    argument,
                    binder_context,
                    active_bound_variables,
                    variables,
                )?;
            }
            Ok(())
        }
        Term::BinderNormalized { binder_id, body } => {
            let Some(bound_variable) = binder_context.variable_for(*binder_id) else {
                return Err(invalid_substitution(
                    target,
                    substitution_id,
                    "substitution.payload.actual_term",
                ));
            };
            active_bound_variables.push(bound_variable);
            let result = collect_term_free_variables(
                target,
                substitution_id,
                body,
                binder_context,
                active_bound_variables,
                variables,
            );
            active_bound_variables.pop();
            result
        }
        Term::Malformed => Err(invalid_substitution(
            target,
            substitution_id,
            "substitution.payload.actual_term",
        )),
    }
}

fn formula_fingerprint(formula: &Formula) -> Result<Fingerprint, RejectionDetail> {
    let canonical = formula
        .canonical_hash_input()
        .map_err(formula_error_detail)?;
    Ok(Fingerprint::new(
        SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID,
        canonical,
    ))
}

fn canonical_problem_bytes(
    target_vc: &Fingerprint,
    atom_variables: &[SatAtomVariable],
    assertions: &[EncodedFormulaAssertion],
    clauses: &[SatClause],
) -> Result<Vec<u8>, RejectionDetail> {
    let mut bytes = Vec::from(SAT_PROBLEM_DOMAIN_SEPARATOR);
    bytes.extend_from_slice(&SAT_PROBLEM_SCHEMA_VERSION.to_be_bytes());
    bytes.extend_from_slice(&SAT_PROBLEM_ENCODING_VERSION.to_be_bytes());
    bytes.extend(fingerprint_bytes(target_vc)?);
    write_len(atom_variables.len(), &mut bytes)?;
    for atom in atom_variables {
        bytes.extend_from_slice(&atom.variable.0.to_be_bytes());
        write_len(atom.atom_canonical_bytes.len(), &mut bytes)?;
        bytes.extend_from_slice(&atom.atom_canonical_bytes);
    }
    write_len(assertions.len(), &mut bytes)?;
    for assertion in assertions {
        bytes.push(assertion.assertion_kind);
        bytes.push(u8::from(assertion.asserted_true));
        write_option_u32(assertion.source_formula_id, &mut bytes);
        write_option_u32(assertion.substitution_id, &mut bytes);
        bytes.extend(fingerprint_bytes(&assertion.formula_fingerprint)?);
        let formula_bytes = assertion
            .formula
            .canonical_hash_input()
            .map_err(formula_error_detail)?;
        write_len(formula_bytes.len(), &mut bytes)?;
        bytes.extend(formula_bytes);
    }
    write_len(clauses.len(), &mut bytes)?;
    for clause in clauses {
        write_len(clause.literals.len(), &mut bytes)?;
        for literal in &clause.literals {
            bytes.extend_from_slice(&literal.variable.0.to_be_bytes());
            bytes.push(u8::from(literal.positive));
        }
    }
    Ok(bytes)
}

fn fingerprint_bytes(fingerprint: &Fingerprint) -> Result<Vec<u8>, RejectionDetail> {
    let mut bytes = vec![fingerprint.algorithm_id];
    write_len(fingerprint.digest.len(), &mut bytes)?;
    bytes.extend_from_slice(&fingerprint.digest);
    Ok(bytes)
}

fn write_option_u32(value: Option<u32>, bytes: &mut Vec<u8>) {
    match value {
        Some(value) => {
            bytes.push(1);
            bytes.extend_from_slice(&value.to_be_bytes());
        }
        None => {
            bytes.push(0);
            bytes.extend_from_slice(&0u32.to_be_bytes());
        }
    }
}

fn write_len(len: usize, bytes: &mut Vec<u8>) -> Result<(), RejectionDetail> {
    let len = u32::try_from(len).map_err(|_| RejectionDetail::ResourceExhaustion)?;
    bytes.extend_from_slice(&len.to_be_bytes());
    Ok(())
}

fn formula_error_detail(error: FormulaEvidenceError) -> RejectionDetail {
    match error {
        FormulaEvidenceError::ResourceExhaustion => RejectionDetail::ResourceExhaustion,
        FormulaEvidenceError::Clause(error) => clause_error_detail(error),
    }
}

fn clause_error_detail(error: ClauseError) -> RejectionDetail {
    if matches!(
        error,
        ClauseError::LiteralCountExceeded { .. }
            | ClauseError::TermSizeExceeded { .. }
            | ClauseError::TermRecursionDepthExceeded { .. }
    ) {
        RejectionDetail::ResourceExhaustion
    } else {
        RejectionDetail::InvalidSatRefutation
    }
}

fn clause_error_rejection(
    target: &TargetVcFingerprint,
    error: ClauseError,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    rejection(
        target,
        RejectionCategory::KernelRejection,
        clause_error_detail(error),
        location,
    )
}

fn formula_error_rejection(
    target: &TargetVcFingerprint,
    error: FormulaEvidenceError,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    rejection(
        target,
        RejectionCategory::KernelRejection,
        formula_error_detail(error),
        location,
    )
}

fn invalid_substitution(
    target: &TargetVcFingerprint,
    substitution_id: u32,
    field: &'static str,
) -> Box<RejectionRecord> {
    rejection(
        target,
        RejectionCategory::KernelRejection,
        RejectionDetail::InvalidSubstitution,
        RejectionLocation::new()
            .with_substitution_id(substitution_id)
            .with_field_path(field),
    )
}

fn resource_rejection(
    target: &TargetVcFingerprint,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    rejection(
        target,
        RejectionCategory::KernelRejection,
        RejectionDetail::ResourceExhaustion,
        location,
    )
}

fn rejection(
    target: &TargetVcFingerprint,
    category: RejectionCategory,
    detail: RejectionDetail,
    location: RejectionLocation,
) -> Box<RejectionRecord> {
    Box::new(
        RejectionRecord::new(target.clone(), category, detail, location)
            .expect("sat encoding uses valid rejection detail mappings"),
    )
}

fn next_depth(
    target: &TargetVcFingerprint,
    depth: usize,
    field: &'static str,
) -> SatEncodingResult<usize> {
    depth
        .checked_add(1)
        .ok_or_else(|| resource_rejection(target, RejectionLocation::new().with_field_path(field)))
}

#[cfg(test)]
mod tests;
