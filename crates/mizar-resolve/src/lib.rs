//! Module and symbol resolution for Mizar Evo.
//!
//! This crate owns phases 4-5 of the pipeline. It currently exposes the
//! resolver-owned `ResolvedAst` and `SymbolEnv` data shapes, the module-index
//! input seam, source-shaped import path and graph resolution,
//! declaration-shell collection, namespace lookup, preliminary symbol-name
//! lookup, crate-local/internal name diagnostics, dot-chain finalization, and
//! executable label resolution while signature and full symbol extraction
//! passes land in follow-on tasks.

/// Source-shaped declaration shell collection.
pub mod declarations;

/// Symbol environment data shapes and deterministic indexes.
pub mod env;

/// Semantic import graph construction and cycle rejection.
pub mod imports;

/// Label declaration and citation resolution.
pub mod labels;

/// Resolver-side module-index phase input seam.
pub mod module_index;

/// Namespace, preliminary symbol-name resolution, internal diagnostics, and
/// dot-chain finalization.
pub mod names;

/// Resolved AST data shapes and reference tables.
pub mod resolved_ast;
