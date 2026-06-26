#![forbid(unsafe_code)]

//! Trusted certificate checking boundary for Mizar Evo.
//!
//! This crate owns pipeline phase 14. Public semantic modules are added only
//! after their owning English/Japanese design specifications exist.
//!
//! The kernel checks evidence only. It must not perform proof search, premise
//! selection, overload resolution, cluster search, ATP search, implicit coercion
//! insertion, fallback inference, or hidden global-state lookup.

pub mod certificate_parser;
pub mod checker;
pub mod clause;
pub mod formula_evidence;
pub mod rejection;
pub mod resolution_trace;
pub mod sat_checker;
pub mod sat_encoding;
pub mod substitution_checker;
