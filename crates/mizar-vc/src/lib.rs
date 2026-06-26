//! Verification-condition generation, deterministic pre-ATP discharge, and
//! dependency-slice assembly.
//!
//! This crate owns pipeline phases 11-12. Public semantic modules are added
//! only after their owning English/Japanese design specifications exist.

pub mod dependency_slice;
pub mod discharge;
pub mod generator;
pub mod kernel_evidence_handoff;
pub mod vc_ir;
