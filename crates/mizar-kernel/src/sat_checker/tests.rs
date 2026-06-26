use crate::{
    certificate_parser::Fingerprint,
    rejection::RejectionDetail,
    sat_encoding::{EncodedSatProblem, SatClause, SatLiteral, SatVariable},
};

use super::*;

#[test]
fn unsatisfiable_problem_returns_unsat_report() {
    let result = check_sat_problem(
        &problem(vec![vec![positive(1)], vec![negative(1)]], b"unsat"),
        &SatCheckContext::v1(),
    );

    let SatCheckResult::Unsat(report) = result else {
        panic!("contradictory unit clauses must be unsat");
    };
    assert_eq!(
        report.target_vc,
        TargetVcFingerprint::from_certificate_fingerprint(&target())
    );
    assert_eq!(report.variables, 1);
    assert_eq!(report.clauses, 2);
    assert_eq!(report.literals, 2);
    assert_eq!(report.canonical_bytes, 5);
}

#[test]
fn satisfiable_problem_returns_sat_without_acceptance_material() {
    let result = check_sat_problem(
        &problem(vec![vec![positive(1), positive(2)]], b"sat"),
        &SatCheckContext::v1(),
    );

    let SatCheckResult::Sat(report) = result else {
        panic!("single disjunctive clause must be satisfiable");
    };
    assert_eq!(report.variables, 2);
    assert_eq!(report.clauses, 1);
    assert_eq!(report.literals, 2);
    assert_eq!(report.canonical_bytes, 3);
}

#[test]
fn repeated_checks_are_deterministic() {
    let encoded = problem(vec![vec![positive(1)], vec![negative(1)]], b"repeat");

    let first = check_sat_problem(&encoded, &SatCheckContext::v1());
    let second = check_sat_problem(&encoded, &SatCheckContext::v1());

    assert_eq!(first, second);
    assert!(first.is_unsat());
}

#[test]
fn input_limits_reject_before_solver_construction() {
    let encoded = problem(vec![vec![positive(1), positive(2)]], b"canonical");
    let cases = [
        (
            SatCheckLimits {
                max_variables: 1,
                ..SatCheckLimits::default()
            },
            "sat_checker.variables",
        ),
        (
            SatCheckLimits {
                max_clauses: 0,
                ..SatCheckLimits::default()
            },
            "sat_checker.clauses",
        ),
        (
            SatCheckLimits {
                max_literals: 1,
                ..SatCheckLimits::default()
            },
            "sat_checker.literals",
        ),
        (
            SatCheckLimits {
                max_literals_per_clause: 1,
                ..SatCheckLimits::default()
            },
            "sat_checker.clause_width",
        ),
        (
            SatCheckLimits {
                max_canonical_bytes: 1,
                ..SatCheckLimits::default()
            },
            "sat_checker.canonical_bytes",
        ),
    ];

    for (limits, field_path) in cases {
        assert_rejected(
            check_sat_problem(&encoded, &SatCheckContext::v1().with_limits(limits)),
            RejectionDetail::ResourceExhaustion,
            field_path,
        );
    }
}

#[test]
fn unsupported_step_budgets_reject_without_solver_hook_accounting() {
    let encoded = problem(vec![vec![positive(1)]], b"budget");
    for limits in [
        SatCheckLimits {
            max_conflicts: Some(1),
            ..SatCheckLimits::default()
        },
        SatCheckLimits {
            max_propagations: Some(1),
            ..SatCheckLimits::default()
        },
    ] {
        assert_rejected(
            check_sat_problem(&encoded, &SatCheckContext::v1().with_limits(limits)),
            RejectionDetail::ResourceExhaustion,
            "sat_checker.unsupported_step_budget",
        );
    }
}

#[test]
fn unsupported_clause_shapes_reject_as_invalid_refutation() {
    let cases = [
        (problem(vec![Vec::new()], b"empty"), "sat_checker.clause"),
        (
            problem(vec![vec![SatLiteral::positive(SatVariable(0))]], b"zero"),
            "sat_checker.literal_variable",
        ),
    ];

    for (encoded, field_path) in cases {
        assert_rejected(
            check_sat_problem(&encoded, &SatCheckContext::v1()),
            RejectionDetail::InvalidSatRefutation,
            field_path,
        );
    }
}

#[test]
fn solver_options_are_pinned_to_audited_defaults() {
    let options = pinned_solver_options();

    assert!(options.check());
    assert_eq!(options.var_decay, 0.95);
    assert_eq!(options.clause_decay, 0.999);
    assert_eq!(options.random_var_freq, 0.0);
    assert_eq!(options.random_seed, 91648253.0);
    assert!(options.luby_restart);
    assert_eq!(options.ccmin_mode, 2);
    assert_eq!(options.phase_saving, 2);
    assert!(!options.rnd_pol);
    assert!(!options.rnd_init_act);
    assert_eq!(options.garbage_frac, 0.20);
    assert_eq!(options.min_learnts_lim, 0);
    assert_eq!(options.restart_first, 100);
    assert_eq!(options.restart_inc, 2.0);
    assert_eq!(options.learntsize_factor, 1.0 / 3.0);
    assert_eq!(options.learntsize_inc, 1.1);
}

fn problem(clauses: Vec<Vec<SatLiteral>>, canonical_bytes: &[u8]) -> EncodedSatProblem {
    EncodedSatProblem::from_test_parts(
        target(),
        Vec::new(),
        clauses
            .into_iter()
            .map(|literals| SatClause { literals })
            .collect(),
        canonical_bytes.to_vec(),
    )
}

fn positive(variable: u32) -> SatLiteral {
    SatLiteral::positive(SatVariable(variable))
}

fn negative(variable: u32) -> SatLiteral {
    SatLiteral::negative(SatVariable(variable))
}

fn target() -> Fingerprint {
    Fingerprint::new(9, b"target".to_vec())
}

fn assert_rejected(result: SatCheckResult, detail: RejectionDetail, field_path: &'static str) {
    let SatCheckResult::Rejected(rejection) = result else {
        panic!("expected rejection");
    };
    assert_eq!(rejection.detail(), detail);
    assert_eq!(rejection.location().field_path, Some(field_path));
}
