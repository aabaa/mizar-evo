//! Core elaboration and control-flow preparation for Mizar Evo.
//!
//! This crate owns pipeline phases 9-10. Public semantic modules are added only
//! after their owning English/Japanese design specifications exist.

pub mod binder_normalization;
pub mod control_flow;
pub mod core_ir;
pub mod elaborator;
