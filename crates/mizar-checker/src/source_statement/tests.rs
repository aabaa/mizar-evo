use super::*;
use crate::{
    binding_env::{
        BindingContextDraft, BindingContextTable, BindingDiagnosticTable, BindingEnvParts,
        BindingTable,
    },
    resolved_typed_ast::ResolvedTypedAstError,
};
use mizar_resolve::{
    env::{ContributionKind, SourceContributionIndex},
    resolved_ast::{ImportResolution, ResolvedImport, ResolvedImports},
};
use mizar_session::{
    BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator as _,
};

fn source_id(source_ordinal: usize) -> SourceId {
    let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "d9".repeat(32)
    ))
    .expect("snapshot");
    let allocator = InMemorySessionIdAllocator::new();
    (0..source_ordinal)
        .map(|_| allocator.next_source_id(snapshot).expect("source"))
        .last()
        .expect("positive source ordinal")
}

mod misc;
