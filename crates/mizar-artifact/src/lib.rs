//! Stable external artifact projections and publication support.
//!
//! This crate owns the published artifact boundary described by
//! [`00.crate_plan.md`]. Schema and store modules land through task-scoped
//! changes so that each behavior keeps its own spec and tests.
//!
//! [`00.crate_plan.md`]: ../../../../doc/design/mizar-artifact/en/00.crate_plan.md

/// Canonical artifact serialization, schema-version checks, and hash framing.
pub mod store;

/// Published module-summary schema and validating reader/writer.
pub mod module_summary;

/// Published registration-summary schema and validating reader/writer.
pub mod registration_summary;
