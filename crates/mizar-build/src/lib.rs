//! Build planning and validation for package and workspace inputs.

/// Module-index construction and build-side provider contract.
pub mod module_index;

/// Package manifest validation and build-plan input models.
pub mod planner;

/// Modeled resource-budget admission and release accounting.
pub mod resource;

/// Deterministic synthetic task scheduling over task graphs.
pub mod scheduler;

/// Deterministic verification task graph construction.
pub mod task_graph;
