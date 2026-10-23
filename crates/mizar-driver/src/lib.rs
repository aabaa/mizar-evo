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
mod frontend_adapter;
pub mod registry;
pub mod request;

pub use frontend_adapter::native_algorithm_core_from_storage_bytes;
pub use frontend_adapter::native_algorithm_core_storage_bytes;
pub use frontend_adapter::native_algorithm_vc_from_storage_bytes;
pub use frontend_adapter::native_algorithm_vc_storage_bytes;
pub use frontend_adapter::native_theorem_core_from_storage_bytes;
pub use frontend_adapter::native_theorem_core_semantic_payload;
pub use frontend_adapter::native_theorem_core_storage_bytes;
pub use frontend_adapter::native_theorem_discharge_from_storage_bytes;
pub use frontend_adapter::native_theorem_discharge_storage_bytes;
pub use frontend_adapter::native_theorem_metadata_from_storage_bytes;
pub use frontend_adapter::native_theorem_metadata_storage_bytes;
pub use frontend_adapter::native_theorem_preparation_from_storage_bytes;
pub use frontend_adapter::native_theorem_preparation_storage_bytes;
pub use frontend_adapter::native_theorem_registration_closure_from_storage_bytes;
pub use frontend_adapter::native_theorem_registration_closure_storage_bytes;
pub use frontend_adapter::native_theorem_typed_ast_from_storage_bytes;
pub use frontend_adapter::native_theorem_typed_ast_storage_bytes;
pub use frontend_adapter::native_theorem_vc_from_storage_bytes;
pub use frontend_adapter::native_theorem_vc_storage_bytes;
