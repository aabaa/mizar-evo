use std::panic::{AssertUnwindSafe, catch_unwind};

use batsat::{BasicSolver, Lit, SolverInterface, SolverOpts, lbool};

use crate::{
    rejection::{
        RejectionCategory, RejectionDetail, RejectionLocation, RejectionRecord, TargetVcFingerprint,
    },
    sat_encoding::{EncodedSatProblem, SatLiteral},
};

#[cfg(test)]
mod tests;

const DEFAULT_MAX_VARIABLES: usize = 2_000_000;
const DEFAULT_MAX_CLAUSES: usize = 4_000_000;
const DEFAULT_MAX_LITERALS: usize = 16_000_000;
const DEFAULT_MAX_LITERALS_PER_CLAUSE: usize = 65_536;
const DEFAULT_MAX_CANONICAL_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SatCheckContext {
    pub limits: SatCheckLimits,
}

impl SatCheckContext {
    #[must_use]
    pub fn v1() -> Self {
        Self {
            limits: SatCheckLimits::default(),
        }
    }

    #[must_use]
    pub const fn with_limits(mut self, limits: SatCheckLimits) -> Self {
        self.limits = limits;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SatCheckLimits {
    pub max_variables: usize,
    pub max_clauses: usize,
    pub max_literals: usize,
    pub max_literals_per_clause: usize,
    pub max_canonical_bytes: usize,
    pub max_conflicts: Option<u64>,
    pub max_propagations: Option<u64>,
}

impl Default for SatCheckLimits {
    fn default() -> Self {
        Self {
            max_variables: DEFAULT_MAX_VARIABLES,
            max_clauses: DEFAULT_MAX_CLAUSES,
            max_literals: DEFAULT_MAX_LITERALS,
            max_literals_per_clause: DEFAULT_MAX_LITERALS_PER_CLAUSE,
            max_canonical_bytes: DEFAULT_MAX_CANONICAL_BYTES,
            max_conflicts: None,
            max_propagations: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SatCheckReport {
    pub target_vc: TargetVcFingerprint,
    pub variables: usize,
    pub clauses: usize,
    pub literals: usize,
    pub canonical_bytes: usize,
    pub conflicts: u64,
    pub propagations: u64,
    pub decisions: u64,
    pub restarts: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SatCheckResult {
    Unsat(SatCheckReport),
    Sat(SatCheckReport),
    Rejected(RejectionRecord),
}

impl SatCheckResult {
    #[must_use]
    pub const fn is_unsat(&self) -> bool {
        matches!(self, Self::Unsat(_))
    }
}

pub fn check_sat_problem(problem: &EncodedSatProblem, context: &SatCheckContext) -> SatCheckResult {
    let target = TargetVcFingerprint::from_certificate_fingerprint(problem.target_vc());
    let counts = match validate_problem(problem, context, &target) {
        Ok(counts) => counts,
        Err(error) => return SatCheckResult::Rejected(*error),
    };
    let panic_target = target.clone();
    match catch_unwind(AssertUnwindSafe(|| run_batsat(problem, &target, counts))) {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => SatCheckResult::Rejected(*error),
        Err(_) => SatCheckResult::Rejected(invalid_sat_refutation(
            &panic_target,
            "sat_checker.dependency_panic",
        )),
    }
}

fn run_batsat(
    problem: &EncodedSatProblem,
    target: &TargetVcFingerprint,
    counts: ProblemCounts,
) -> Result<SatCheckResult, Box<RejectionRecord>> {
    let mut solver = BasicSolver::new(pinned_solver_options(), Default::default());
    let mut variables = Vec::with_capacity(counts.variables);
    for _ in 0..counts.variables {
        variables.push(solver.new_var_default());
    }

    for clause in problem.clauses() {
        let mut solver_clause = Vec::with_capacity(clause.literals.len());
        for literal in &clause.literals {
            solver_clause.push(to_batsat_literal(literal, &variables, target)?);
        }
        if !solver.add_clause_reuse(&mut solver_clause) {
            return Ok(SatCheckResult::Unsat(report(target, counts, &solver)));
        }
    }

    if !solver.is_ok() {
        return Ok(SatCheckResult::Unsat(report(target, counts, &solver)));
    }

    match solver.solve_limited(&[]) {
        status if status == lbool::FALSE => {
            Ok(SatCheckResult::Unsat(report(target, counts, &solver)))
        }
        status if status == lbool::TRUE => Ok(SatCheckResult::Sat(report(target, counts, &solver))),
        status if status == lbool::UNDEF => Err(Box::new(invalid_sat_refutation(
            target,
            "sat_checker.undefined_result",
        ))),
        _ => Err(Box::new(invalid_sat_refutation(
            target,
            "sat_checker.unexpected_result",
        ))),
    }
}

fn to_batsat_literal(
    literal: &SatLiteral,
    variables: &[batsat::Var],
    target: &TargetVcFingerprint,
) -> Result<Lit, Box<RejectionRecord>> {
    let id = literal.variable.0;
    if id == 0 {
        return Err(Box::new(invalid_sat_refutation(
            target,
            "sat_checker.literal_variable",
        )));
    }
    let index = usize::try_from(id - 1).map_err(|_| {
        Box::new(invalid_sat_refutation(
            target,
            "sat_checker.literal_variable",
        ))
    })?;
    let Some(variable) = variables.get(index).copied() else {
        return Err(Box::new(invalid_sat_refutation(
            target,
            "sat_checker.literal_variable",
        )));
    };
    Ok(Lit::new(variable, literal.positive))
}

#[derive(Clone, Copy)]
struct ProblemCounts {
    variables: usize,
    clauses: usize,
    literals: usize,
    canonical_bytes: usize,
}

fn validate_problem(
    problem: &EncodedSatProblem,
    context: &SatCheckContext,
    target: &TargetVcFingerprint,
) -> Result<ProblemCounts, Box<RejectionRecord>> {
    if context.limits.max_conflicts.is_some() || context.limits.max_propagations.is_some() {
        return Err(Box::new(resource_rejection(
            target,
            "sat_checker.unsupported_step_budget",
        )));
    }
    let canonical_bytes = problem.canonical_bytes().len();
    if canonical_bytes > context.limits.max_canonical_bytes {
        return Err(Box::new(resource_rejection(
            target,
            "sat_checker.canonical_bytes",
        )));
    }
    let clauses = problem.clauses().len();
    if clauses > context.limits.max_clauses {
        return Err(Box::new(resource_rejection(target, "sat_checker.clauses")));
    }

    let mut literals = 0usize;
    let mut max_variable_id = 0u32;
    for clause in problem.clauses() {
        if clause.literals.is_empty() {
            return Err(Box::new(invalid_sat_refutation(
                target,
                "sat_checker.clause",
            )));
        }
        if clause.literals.len() > context.limits.max_literals_per_clause {
            return Err(Box::new(resource_rejection(
                target,
                "sat_checker.clause_width",
            )));
        }
        literals = literals
            .checked_add(clause.literals.len())
            .ok_or_else(|| Box::new(resource_rejection(target, "sat_checker.literals")))?;
        if literals > context.limits.max_literals {
            return Err(Box::new(resource_rejection(target, "sat_checker.literals")));
        }
        for literal in &clause.literals {
            if literal.variable.0 == 0 {
                return Err(Box::new(invalid_sat_refutation(
                    target,
                    "sat_checker.literal_variable",
                )));
            }
            max_variable_id = max_variable_id.max(literal.variable.0);
        }
    }

    let variables = usize::try_from(max_variable_id)
        .map_err(|_| Box::new(resource_rejection(target, "sat_checker.variables")))?;
    if variables > context.limits.max_variables {
        return Err(Box::new(resource_rejection(
            target,
            "sat_checker.variables",
        )));
    }
    Ok(ProblemCounts {
        variables,
        clauses,
        literals,
        canonical_bytes,
    })
}

fn report(
    target: &TargetVcFingerprint,
    counts: ProblemCounts,
    solver: &BasicSolver,
) -> SatCheckReport {
    SatCheckReport {
        target_vc: target.clone(),
        variables: counts.variables,
        clauses: counts.clauses,
        literals: counts.literals,
        canonical_bytes: counts.canonical_bytes,
        conflicts: solver.num_conflicts(),
        propagations: solver.num_propagations(),
        decisions: solver.num_decisions(),
        restarts: solver.num_restarts(),
    }
}

fn pinned_solver_options() -> SolverOpts {
    let options = SolverOpts {
        var_decay: 0.95,
        clause_decay: 0.999,
        random_var_freq: 0.0,
        random_seed: 91648253.0,
        luby_restart: true,
        ccmin_mode: 2,
        phase_saving: 2,
        rnd_pol: false,
        rnd_init_act: false,
        garbage_frac: 0.20,
        min_learnts_lim: 0,
        restart_first: 100,
        restart_inc: 2.0,
        learntsize_factor: 1.0 / 3.0,
        learntsize_inc: 1.1,
    };
    debug_assert!(options.check());
    options
}

fn invalid_sat_refutation(
    target: &TargetVcFingerprint,
    field_path: &'static str,
) -> RejectionRecord {
    RejectionRecord::new(
        target.clone(),
        RejectionCategory::KernelRejection,
        RejectionDetail::InvalidSatRefutation,
        RejectionLocation::new().with_field_path(field_path),
    )
    .expect("invalid_sat_refutation is a valid kernel rejection detail")
}

fn resource_rejection(target: &TargetVcFingerprint, field_path: &'static str) -> RejectionRecord {
    RejectionRecord::new(
        target.clone(),
        RejectionCategory::KernelRejection,
        RejectionDetail::ResourceExhaustion,
        RejectionLocation::new().with_field_path(field_path),
    )
    .expect("resource_exhaustion is a valid kernel rejection detail")
}
