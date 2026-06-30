//! Compiler-internal IR storage, snapshot output handles, and projection
//! boundaries.
//!
//! `mizar-ir` owns sealed internal phase-output storage and typed handles. It
//! consumes snapshot identity from `mizar-session`; cache-key construction,
//! dependency fingerprints, proof-reuse validation, proof acceptance, trusted
//! status, policy selection, and kernel acceptance remain owned by their
//! dedicated crates.
//!
//! Downstream driver sessions, diagnostics integration, producer projection
//! tokens, and artifact publication tokens are external dependency gaps until
//! their owning crates expose real integration seams.
//!
//! Design task stream:
//! [`00.crate_plan.md`](../../../doc/design/mizar-ir/en/00.crate_plan.md).

#![forbid(unsafe_code)]

/// Snapshot-scoped IR identity tables and phase-output lineage.
pub mod identity;

/// Phase-output publisher for sealed handles.
pub mod publisher;

/// Immutable phase-output storage and typed handles.
pub mod storage;
