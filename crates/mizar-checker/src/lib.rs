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
//! shapes. Later cluster traces, overload resolution, and resolved typed AST
//! APIs land in later task-scoped modules after their design specs are written.

pub mod binding_env;
pub mod registration_resolution;
pub mod type_checker;
pub mod typed_ast;
