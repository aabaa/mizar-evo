//! Stable diagnostic identity, records, aggregation, and presentation.
//!
//! This crate owns the diagnostics boundary described by
//! [`00.crate_plan.md`]. Task-scoped module specs add behavior in dependency
//! order. The current implementation exposes the diagnostic-code registry,
//! structured failure records, producer-side sinks, deterministic aggregation,
//! CLI rendering, structured fix suggestions, and lazy explanation handles;
//! driver, LSP, and artifact integration remain outside crate authority.
//!
//! [`00.crate_plan.md`]: ../../../../doc/design/mizar-diagnostics/en/00.crate_plan.md

#![forbid(unsafe_code)]

pub mod aggregator;
pub mod explain;
pub mod failure_record;
pub mod fix;
pub mod registry;
pub mod render;
pub mod sink;
