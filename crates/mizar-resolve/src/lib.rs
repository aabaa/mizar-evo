//! Module and symbol resolution for Mizar Evo.
//!
//! This crate owns phases 4-5 of the pipeline. It currently exposes the
//! resolver-owned `ResolvedAst` and `SymbolEnv` data shapes, the module-index
//! input seam, source-shaped import path and graph resolution,
//! declaration-shell collection, namespace lookup, and preliminary symbol-name
//! lookup while executable label, signature, and full symbol extraction passes
//! land in follow-on tasks.

/// Source-shaped declaration shell collection.
pub mod declarations;

/// Symbol environment data shapes and deterministic indexes.
pub mod env;

/// Semantic import graph construction and cycle rejection.
pub mod imports;

/// Resolver-side module-index phase input seam.
pub mod module_index;

/// Namespace and preliminary symbol-name resolution.
pub mod names;

/// Resolved AST data shapes and reference tables.
pub mod resolved_ast;
