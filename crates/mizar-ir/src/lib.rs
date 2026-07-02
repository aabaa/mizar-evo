//! Compiler-internal IR storage, snapshot output handles, and projection
//! boundaries.
//!
//! `mizar-ir` owns sealed internal phase-output storage and typed handles. It
//! consumes snapshot identity from `mizar-session`; cache-key construction,
//! dependency fingerprints, proof-reuse validation, proof acceptance, trusted
//! status, policy selection, and kernel acceptance remain owned by their
//! dedicated crates.
//!
//! Downstream driver and diagnostics crates exist, but real producer dispatch,
//! diagnostic rendering, producer projection tokens, and artifact publication
//! tokens remain classified as external dependency gaps until their owning
//! crates expose integration seams that `mizar-ir` can consume.
//!
//! Design task stream:
//! [`00.crate_plan.md`](../../../doc/design/mizar-ir/en/00.crate_plan.md).

#![forbid(unsafe_code)]

/// Cache-record adapter for sealed IR handles.
pub mod cache_adapter;

/// Phase input identities and sealed parent handles for scheduler dispatch.
pub mod dispatch_input;

/// Snapshot-scoped IR identity tables and phase-output lineage.
pub mod identity;

/// Phase-output publisher for sealed handles.
pub mod publisher;

/// Artifact projection from sealed handles to stable external schemas.
pub mod projection;

/// Immutable phase-output storage and typed handles.
pub mod storage;
