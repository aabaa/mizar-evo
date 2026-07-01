//! Driver front door for build requests, sessions, phase-service wiring, and
//! entry-point orchestration.
//!
//! The crate is introduced by
//! [`00.crate_plan.md`](../../../doc/design/mizar-driver/en/00.crate_plan.md).
//! Task-scoped modules are added only after their paired EN/JA design specs
//! land. Until then, this scaffold owns no phase semantics, proof authority,
//! cache compatibility decisions, artifact serialization, or LSP protocol
//! conversion.

#![forbid(unsafe_code)]

pub mod cli;
pub mod driver;
pub mod events;
pub mod registry;
pub mod request;
