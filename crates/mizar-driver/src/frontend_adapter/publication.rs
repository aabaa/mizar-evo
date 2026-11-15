use super::*;

// Allocate at the original first-field position; all stage validation remains at the caller.
pub(super) fn publish_native_output<T: Send + Sync + 'static>(
    publisher: &mizar_ir::publisher::PhaseOutputPublisher,
    snapshot: mizar_session::BuildSnapshotId,
    identity: (IrPipelinePhase, IrWorkUnit, OutputKind),
    input: impl FnOnce(
        mizar_ir::storage::PendingOutputSlot<T>,
        IrPipelinePhase,
        IrWorkUnit,
        OutputKind,
    ) -> PublishOutputInput<T>,
) -> Result<mizar_ir::storage::PhaseOutputRef<T>, mizar_ir::publisher::PublishError> {
    let (phase, work_unit, output_kind) = identity;
    let slot = publisher.allocate(
        snapshot,
        phase.clone(),
        work_unit.clone(),
        output_kind.clone(),
        SchemaVersion::new(1),
    );
    publisher.publish(input(slot, phase, work_unit, output_kind))
}
