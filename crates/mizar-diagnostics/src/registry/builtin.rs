//! Built-in diagnostic descriptors allocated by the initial registry.

use super::{
    DiagnosticCode, DiagnosticDescriptor, DiagnosticSeverity, DiagnosticStatus, PhaseFamily,
};

const INITIAL_SINCE: &str = "spec-22.7-v1";
const INITIAL_DOC_URL: &str =
    "doc/spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference";

macro_rules! builtin_descriptor {
    ($severity:ident, $number:literal, $name:literal, $family:ident, $summary:literal) => {
        DiagnosticDescriptor {
            code: DiagnosticCode::from_parts_unchecked(DiagnosticSeverity::$severity, $number),
            meaning_key: $name,
            semantic_name: $name,
            default_severity: DiagnosticSeverity::$severity,
            phase_family: PhaseFamily::$family,
            summary: $summary,
            doc_url: INITIAL_DOC_URL,
            status: DiagnosticStatus::Active,
            since: INITIAL_SINCE,
            retired_since: None,
            replacement_codes: &[],
            aliases: &[],
        }
    };
}

/// Built-in descriptors allocated by the initial spec-22 registry.
pub const BUILTIN_DESCRIPTORS: &[DiagnosticDescriptor] = &[
    builtin_descriptor!(
        Error,
        1,
        "syntax.unexpected_token",
        Syntax,
        "Unexpected token in current syntactic context"
    ),
    builtin_descriptor!(
        Error,
        2,
        "syntax.malformed_literal",
        Syntax,
        "Numeric or string literal does not conform to lexical rules"
    ),
    builtin_descriptor!(
        Error,
        3,
        "syntax.unexpected_end_of_file",
        Syntax,
        "File ended with open construct pending"
    ),
    builtin_descriptor!(
        Error,
        10,
        "syntax.missing_end",
        Syntax,
        "Block opened without matching `end`"
    ),
    builtin_descriptor!(
        Error,
        11,
        "syntax.unmatched_delimiter",
        Syntax,
        "Parenthesis, bracket, or `do` without matching close"
    ),
    builtin_descriptor!(
        Error,
        12,
        "syntax.reserved_keyword_as_identifier",
        Syntax,
        "Reserved keyword used as identifier"
    ),
    builtin_descriptor!(
        Error,
        101,
        "type.mismatch",
        Type,
        "Expression type incompatible with required type"
    ),
    builtin_descriptor!(
        Error,
        102,
        "type.narrowing_requires_proof",
        Type,
        "Narrowing coercion without justification"
    ),
    builtin_descriptor!(
        Error,
        103,
        "type.sethood.missing",
        Type,
        "Fraenkel comprehension for type without `sethood`"
    ),
    builtin_descriptor!(
        Error,
        110,
        "type.inference_conflict",
        Type,
        "Conflicting type constraints within an expression"
    ),
    builtin_descriptor!(
        Error,
        120,
        "type.mode_mismatch",
        Type,
        "Mode incompatibility without registered widening"
    ),
    builtin_descriptor!(
        Error,
        121,
        "type.attribute_required",
        Type,
        "Required attribute not registered for the type"
    ),
    builtin_descriptor!(
        Error,
        122,
        "type.attribute_contradiction",
        Type,
        "Attribute combination rejected: mutually exclusive attributes or missing existential cluster"
    ),
    builtin_descriptor!(
        Error,
        201,
        "resolve.ambiguous_symbol",
        Resolution,
        "Two or more equally-ranked overload candidates"
    ),
    builtin_descriptor!(
        Error,
        202,
        "resolve.no_viable_overload",
        Resolution,
        "No candidate survives type-checking"
    ),
    builtin_descriptor!(
        Error,
        203,
        "template.argument_omitted_not_inferable",
        Resolution,
        "Template schema parameter cannot be inferred"
    ),
    builtin_descriptor!(
        Error,
        204,
        "resolve.incompatible_refinement_join",
        Resolution,
        "Same-root redefinitions expose incompatible joined facts"
    ),
    builtin_descriptor!(
        Error,
        301,
        "proof.by.search_exhausted",
        Proof,
        "`by` step: ATP exhausted resource budget"
    ),
    builtin_descriptor!(
        Error,
        302,
        "proof.by.missing_fact",
        Proof,
        "Goal likely provable but required lemma not in scope"
    ),
    builtin_descriptor!(
        Error,
        303,
        "proof.obligation.open",
        Proof,
        "Proof block closed with goals remaining"
    ),
    builtin_descriptor!(
        Error,
        310,
        "proof.counterexample.found",
        Proof,
        "Counterexample model found for the goal"
    ),
    builtin_descriptor!(
        Error,
        320,
        "proof.atp.timeout",
        Proof,
        "All ATP backends timed out"
    ),
    builtin_descriptor!(
        Error,
        321,
        "proof.atp.axiom_budget_exceeded",
        Proof,
        "Axiom set exceeds `max_axioms` limit for the obligation"
    ),
    builtin_descriptor!(
        Error,
        350,
        "proof.kernel.unsupported_evidence",
        Proof,
        "Legacy or backend proof material is unsupported under normal proof policy"
    ),
    builtin_descriptor!(
        Error,
        351,
        "proof.kernel.missing_provenance",
        Proof,
        "Kernel evidence is missing required provenance or context binding"
    ),
    builtin_descriptor!(
        Error,
        352,
        "proof.kernel.invalid_substitution",
        Proof,
        "Explicit substitution evidence failed kernel side conditions"
    ),
    builtin_descriptor!(
        Error,
        353,
        "proof.kernel.invalid_sat_refutation",
        Proof,
        "Kernel-derived SAT refutation check failed"
    ),
    builtin_descriptor!(
        Error,
        401,
        "logic.contradictory_axioms",
        Logic,
        "ATP derived `False` from declared axioms"
    ),
    builtin_descriptor!(
        Error,
        410,
        "logic.circular_definition",
        Logic,
        "Non-recursive definition refers to itself outside an algorithm block"
    ),
    builtin_descriptor!(
        Error,
        411,
        "logic.circular_cluster",
        Logic,
        "Cluster registration creates attribute inheritance cycle"
    ),
    builtin_descriptor!(
        Error,
        420,
        "vc.postcondition.return",
        Logic,
        "`ensures` not provable at `return` site"
    ),
    builtin_descriptor!(
        Error,
        421,
        "vc.assert.failed",
        Logic,
        "`assert` in algorithm body not provable"
    ),
    builtin_descriptor!(
        Error,
        422,
        "vc.precondition.call_site",
        Logic,
        "Callee `requires` clause not provable at call site"
    ),
    builtin_descriptor!(
        Error,
        423,
        "vc.loop.establish",
        Logic,
        "Loop invariant not provable before first iteration"
    ),
    builtin_descriptor!(
        Error,
        424,
        "vc.loop.maintain",
        Logic,
        "Loop invariant not provable to be preserved"
    ),
    builtin_descriptor!(
        Error,
        425,
        "vc.loop.decrease",
        Logic,
        "Termination measure not provably decreasing"
    ),
    builtin_descriptor!(
        Error,
        426,
        "vc.recursion.decrease",
        Logic,
        "Termination measure not provably decreasing at recursive call"
    ),
    builtin_descriptor!(
        Error,
        430,
        "logic.cluster.inconsistency",
        Logic,
        "Cluster registration creates contradiction"
    ),
    builtin_descriptor!(
        Warning,
        1,
        "warn.unused_variable",
        StructuralWarning,
        "Variable declared but never read"
    ),
    builtin_descriptor!(
        Warning,
        2,
        "warn.unused_definition",
        StructuralWarning,
        "Definition never referenced in package"
    ),
    builtin_descriptor!(
        Warning,
        3,
        "warn.unused_hypothesis",
        StructuralWarning,
        "Proof hypothesis never referenced in subsequent steps"
    ),
    builtin_descriptor!(
        Warning,
        10,
        "warn.deprecated_syntax",
        StructuralWarning,
        "Deprecated construct; replacement provided"
    ),
    builtin_descriptor!(
        Warning,
        101,
        "warn.redundant_hypothesis",
        ProofWarning,
        "`by` clause contains fact not used by accepted evidence"
    ),
    builtin_descriptor!(
        Warning,
        102,
        "warn.externally_attested_proof",
        ProofWarning,
        "External backend success without kernel-accepted evidence"
    ),
    builtin_descriptor!(
        Warning,
        103,
        "proof.citation.unused",
        ProofWarning,
        "Explicit citation absent from kernel-accepted `used_axioms`"
    ),
    builtin_descriptor!(
        Warning,
        201,
        "warn.unreachable_code",
        AlgorithmWarning,
        "Statement unreachable under static control-flow analysis"
    ),
    builtin_descriptor!(
        Warning,
        202,
        "warn.loop_may_not_terminate",
        AlgorithmWarning,
        "`terminating` algorithm with unverified loop measure"
    ),
    builtin_descriptor!(
        Warning,
        210,
        "warn.weakened_postcondition",
        AlgorithmWarning,
        "`ensures` weaker than what the verifier can prove"
    ),
    builtin_descriptor!(
        Warning,
        301,
        "compat.breaking_change",
        CompatibilityWarning,
        "Public API change requires a MAJOR bump"
    ),
    builtin_descriptor!(
        Warning,
        302,
        "compat.feature_addition",
        CompatibilityWarning,
        "Backward-compatible API addition requires a MINOR bump"
    ),
    builtin_descriptor!(
        Warning,
        303,
        "compat.overload_resolution_shift",
        CompatibilityWarning,
        "Registration, redefinition, or conditional-cluster change may shift overload/refinement resolution (heuristic MAJOR)"
    ),
    builtin_descriptor!(
        Warning,
        304,
        "compat.version_bump_insufficient",
        CompatibilityWarning,
        "Declared version bump smaller than required"
    ),
    builtin_descriptor!(
        Warning,
        305,
        "compat.edition_increase",
        CompatibilityWarning,
        "Package edition raised; MAJOR by default, review recommended"
    ),
];
