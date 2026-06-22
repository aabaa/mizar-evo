//! Type checking, registration resolution, and overload resolution for Mizar
//! Evo.
//!
//! This crate owns pipeline phases 6-8. Task 1 intentionally exposes only the
//! crate boundary; typed AST data, binding environments, type checking,
//! cluster traces, overload resolution, and resolved typed AST APIs land in
//! later task-scoped modules after their design specs are written.
