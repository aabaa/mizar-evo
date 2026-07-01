//! Stable diagnostic identity, records, aggregation, and presentation.
//!
//! This crate owns the diagnostics boundary described by
//! [`00.crate_plan.md`]. Task-scoped module specs add behavior in dependency
//! order so the initial scaffold exposes no registry, record, sink, adapter,
//! driver, LSP, or artifact integration surface.
//!
//! [`00.crate_plan.md`]: ../../../../doc/design/mizar-diagnostics/en/00.crate_plan.md

#![forbid(unsafe_code)]
