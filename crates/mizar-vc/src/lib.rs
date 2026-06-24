//! Verification-condition generation and deterministic pre-ATP discharge.
//!
//! This crate owns pipeline phases 11-12. Public semantic modules are added
//! only after their owning English/Japanese design specifications exist.

pub mod discharge;
pub mod generator;
pub mod vc_ir;
