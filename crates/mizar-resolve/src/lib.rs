//! Module and symbol resolution for Mizar Evo.
//!
//! This crate owns phases 4-5 of the pipeline. It currently exposes the
//! resolver-owned `ResolvedAst` and `SymbolEnv` data shapes, the module-index
//! input seam, source-shaped import path and graph resolution, and
//! declaration-shell collection while the executable name, label, and symbol
//! resolution passes land in follow-on tasks.

/// Source-shaped declaration shell collection.
pub mod declarations;

/// Symbol environment data shapes and deterministic indexes.
pub mod env;

/// Semantic import graph construction and cycle rejection.
pub mod imports;

/// Resolver-side module-index phase input seam.
pub mod module_index;

/// Resolved AST data shapes and reference tables.
pub mod resolved_ast;
