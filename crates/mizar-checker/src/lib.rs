//! Type checking, registration resolution, and overload resolution for Mizar
//! Evo.
//!
//! This crate owns pipeline phases 6-8. Task 1 intentionally exposes only the
//! crate boundary; task 3 exposes the typed AST data shapes; task 5 exposes the
//! binding-environment data layer; task 7 exposes type-expression
//! normalization. Later type-checking slices, cluster traces, overload
//! resolution, and resolved typed AST APIs land in later task-scoped modules
//! after their design specs are written.

pub mod binding_env;
pub mod type_checker;
pub mod typed_ast;
