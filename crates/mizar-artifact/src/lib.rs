//! Stable external artifact projections and publication support.
//!
//! This crate owns the published artifact boundary described by
//! [`00.crate_plan.md`]. Schema and store modules are added by later
//! task-scoped changes so that each behavior lands with its own spec and tests.
//!
//! [`00.crate_plan.md`]: ../../../../doc/design/mizar-artifact/en/00.crate_plan.md

/// Canonical artifact serialization, schema-version checks, and hash framing.
pub mod store;
