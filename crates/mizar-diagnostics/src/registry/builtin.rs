//! Built-in diagnostic descriptors.

use super::{
    DiagnosticCode, DiagnosticDescriptor, DiagnosticSeverity, DiagnosticStatus, PhaseFamily,
};

const INITIAL_SINCE: &str = "spec-22.7-v1";
const FRONTEND_SINCE: &str = "spec-22-frontend-v1";
const INITIAL_DOC_URL: &str =
    "doc/spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference";

macro_rules! builtin_descriptor {
    ($severity:ident, $number:literal, $name:literal, $family:ident, $summary:literal) => {
        builtin_descriptor!($severity, $number, $name, $family, $summary, INITIAL_SINCE)
    };
    ($severity:ident, $number:literal, $name:literal, $family:ident, $summary:literal, $since:expr) => {
        DiagnosticDescriptor {
            code: DiagnosticCode::from_parts_unchecked(DiagnosticSeverity::$severity, $number),
            meaning_key: $name,
            semantic_name: $name,
            default_severity: DiagnosticSeverity::$severity,
            phase_family: PhaseFamily::$family,
            summary: $summary,
            doc_url: INITIAL_DOC_URL,
            status: DiagnosticStatus::Active,
            since: $since,
            retired_since: None,
            replacement_codes: &[],
            aliases: &[],
        }
    };
}

/// Built-in descriptors allocated by specification 22.
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
        13,
        "syntax.carriage_return",
        Syntax,
        "Carriage return remains in lexical input",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        14,
        "syntax.non_ascii_code",
        Syntax,
        "Non-ASCII character outside an allowed lexical region",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        15,
        "syntax.unterminated_multiline_comment",
        Syntax,
        "Multiline comment has no closing delimiter",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        16,
        "syntax.import_missing_module_path",
        Syntax,
        "Import prescan requires a module path",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        17,
        "syntax.import_empty_path_component",
        Syntax,
        "Import prescan finds an empty path component",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        18,
        "syntax.import_missing_alias",
        Syntax,
        "Import prescan requires an alias name",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        19,
        "syntax.import_missing_semicolon",
        Syntax,
        "Import prescan requires its terminating semicolon",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        20,
        "syntax.import_unexpected_token",
        Syntax,
        "Unexpected token during import prescan",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        21,
        "syntax.raw_import_scan_error",
        Syntax,
        "Raw scanning failed during import prescan",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        22,
        "syntax.unresolved_import",
        Syntax,
        "Import target unavailable to lexical-environment construction",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        23,
        "syntax.missing_lexical_summary",
        Syntax,
        "Resolved import has no usable lexical summary",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        24,
        "syntax.user_symbol_import_conflict",
        Syntax,
        "Imported user-symbol shapes conflict",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        25,
        "syntax.invalid_user_symbol_spelling",
        Syntax,
        "Dependency lexical summary has an invalid symbol spelling",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        26,
        "syntax.invalid_user_symbol_arity",
        Syntax,
        "Dependency lexical summary has invalid symbol arity",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        27,
        "syntax.reserved_word_collision",
        Syntax,
        "Dependency user-symbol spelling collides with a reserved word",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        28,
        "syntax.reserved_symbol_collision",
        Syntax,
        "Dependency user-symbol spelling collides with a reserved symbol",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        29,
        "syntax.raw_scan_error",
        Syntax,
        "Raw token scanning failed",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        30,
        "syntax.malformed_binder_list",
        Syntax,
        "Lexical scope scan finds a malformed binder list",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        31,
        "syntax.unsupported_binder_shape",
        Syntax,
        "Lexical scope scan cannot interpret the binder shape",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        32,
        "syntax.duplicate_binding_name",
        Syntax,
        "Lexical scope scan finds duplicate binding names",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        33,
        "syntax.unmatched_end",
        Syntax,
        "Lexical scope scan finds an end without a matching opener",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        34,
        "syntax.no_valid_token_candidate",
        Syntax,
        "No lexical token candidate is valid",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        35,
        "syntax.parser_context_rejected_candidate",
        Syntax,
        "Parser lexing context rejects otherwise valid candidates",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        36,
        "syntax.ambiguous_user_symbol",
        Syntax,
        "Lexical user-symbol candidates cannot be disambiguated",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        37,
        "syntax.unsupported_raw_token",
        Syntax,
        "Disambiguation cannot consume the raw token kind",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        38,
        "syntax.unexpected_error_token",
        Syntax,
        "Parser encounters a lexical recovery/error token",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        39,
        "syntax.dangling_operator",
        Syntax,
        "Operator lacks a required operand",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        40,
        "syntax.non_associative_operator_chain",
        Syntax,
        "Unparenthesized non-associative operator chain",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        41,
        "syntax.missing_semicolon",
        Syntax,
        "Parser requires a terminating semicolon",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        42,
        "syntax.missing_string_literal",
        Syntax,
        "String-required syntax lacks a string literal",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        43,
        "syntax.malformed_import",
        Syntax,
        "Parser finds malformed import syntax",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        44,
        "syntax.malformed_export",
        Syntax,
        "Parser finds malformed export syntax",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        45,
        "syntax.malformed_visibility",
        Syntax,
        "Parser finds malformed visibility syntax",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        46,
        "syntax.malformed_type_expression",
        Syntax,
        "Parser finds a malformed type expression",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        47,
        "syntax.malformed_term_expression",
        Syntax,
        "Parser finds a malformed term expression",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        48,
        "syntax.malformed_formula_expression",
        Syntax,
        "Parser finds a malformed formula",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        49,
        "syntax.malformed_justification",
        Syntax,
        "Parser finds malformed justification syntax",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        50,
        "syntax.malformed_annotation",
        Syntax,
        "Parser finds malformed annotation syntax",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        51,
        "syntax.unexpected_top_level_token",
        Syntax,
        "Token cannot start a top-level item",
        FRONTEND_SINCE
    ),
    builtin_descriptor!(
        Error,
        52,
        "syntax.unrecoverable_input",
        Syntax,
        "Parser cannot recover an AST from the input",
        FRONTEND_SINCE
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
        Error,
        600,
        "source.load_failed",
        SourceLoad,
        "Other source-loading failure",
        "spec-22-source-load-v1"
    ),
    builtin_descriptor!(
        Error,
        601,
        "source.invalid_utf8",
        SourceLoad,
        "Source bytes are not valid UTF-8",
        "spec-22-source-load-v1"
    ),
    builtin_descriptor!(
        Error,
        602,
        "source.unreadable_file",
        SourceLoad,
        "Source file cannot be read",
        "spec-22-source-load-v1"
    ),
    builtin_descriptor!(
        Error,
        603,
        "source.outside_package_root",
        SourceLoad,
        "Source path escapes the package root",
        "spec-22-source-load-v1"
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
