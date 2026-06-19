//! Module and symbol resolution for Mizar Evo.
//!
//! This crate owns phases 4-5 of the pipeline. It currently exposes the
//! resolver-owned `ResolvedAst` data shapes and reference tables while the
//! executable import, name, label, and symbol resolution passes land in
//! follow-on tasks.

/// Resolved AST data shapes and reference tables.
pub mod resolved_ast;
