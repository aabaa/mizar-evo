//! Type checking, registration resolution, and overload resolution for Mizar
//! Evo.
//!
//! This crate owns pipeline phases 6-8. Task 1 intentionally exposes only the
//! crate boundary; task 3 exposes the typed AST data shapes; task 5 exposes the
//! binding-environment data layer; task 7 exposes type-expression
//! normalization; task 8 exposes declaration and local-binding checking over
//! checker-owned payloads; task 9 exposes term/formula inference over
//! checker-owned payloads; task 10 exposes coercion candidate and initial
//! obligation checking over checker-owned payloads; task 11 exposes
//! deterministic type-fact queries; task 14 exposes registration database data
//! shapes; task 16 exposes cluster closure and trace recording; task 19
//! exposes pending-registration validation and activation gating; task 20
//! exposes existential gates for attributed type use; task 22 exposes
//! checker-owned overload site and candidate collection; task 23 exposes
//! checker-owned template expansion over collected candidates; task 24 exposes
//! checker-owned viability filtering over explicit recorded-evidence payloads;
//! task 25 exposes per-site specificity graph construction over viable
//! candidates; task 26 exposes root selection, refinement-join validation,
//! inserted-view recording, and failed-site preservation; task 28 exposes
//! final resolved typed AST assembly over explicit checker-owned outputs.

pub mod binding_env;
pub mod cluster_trace;
pub mod overload_resolution;
pub mod registration_resolution;
pub mod resolved_typed_ast;
pub mod source_attribute;
pub mod source_context;
pub mod source_evidence;
pub mod source_term;
pub mod source_type;
pub mod type_checker;
pub mod typed_ast;

#[cfg(test)]
mod determinism_suite;
