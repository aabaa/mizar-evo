//! Stable diagnostic identity, records, aggregation, and presentation.
//!
//! This crate owns the diagnostics boundary described by
//! [`00.crate_plan.md`]. Task-scoped module specs add behavior in dependency
//! order. The current implementation exposes the diagnostic-code registry,
//! structured failure records, producer-side sinks, deterministic aggregation,
//! and CLI rendering; driver, LSP, and artifact integration are added by later
//! tasks.
//!
//! [`00.crate_plan.md`]: ../../../../doc/design/mizar-diagnostics/en/00.crate_plan.md

#![forbid(unsafe_code)]

pub mod aggregator;
pub mod failure_record;
pub mod registry;
pub mod render;
pub mod sink;
