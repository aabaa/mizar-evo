use std::{collections::HashSet, sync::Arc};

use mizar_ir::{
    identity::{
        ExprIdentityInput, ItemIdentityInput, ModuleIdentityInput, NamedInputHash, OutputKind,
        PipelinePhase, SnapshotHandleRegistry, VcIdentityInput, WorkUnit,
    },
    publisher::{
        AllowedWorkUnit, OutputOrigin, PhaseOutputPublisher, PublicationTarget, PublishError,
        PublishOutputInput,
    },
    storage::{
        AnyPhaseOutputRef, BlobDecodeError, BlobDecoder, CollectInput, IrSideTables,
        IrStorageService, PhaseOutputRef, RetainOwner, SchemaVersion, SideTableRecord,
        StorageError,
    },
};
use mizar_session::{BuildSnapshotId, Hash};

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

fn snapshot(seed: u8) -> BuildSnapshotId {
    let hex = [seed; Hash::BYTE_LEN]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    BuildSnapshotId::from_published_schema_str(&format!("mizar-session-build-snapshot-v1:{hex}"))
        .expect("test snapshot id is valid")
}

fn phase() -> PipelinePhase {
    PipelinePhase::new("resolve")
}

fn output_kind() -> OutputKind {
    OutputKind::new("ResolvedAst")
}

fn schema() -> SchemaVersion {
    SchemaVersion::new(1)
}

fn work_unit(value: impl Into<String>) -> WorkUnit {
    WorkUnit::new(value)
}

fn named(seed: u8) -> NamedInputHash {
    NamedInputHash {
        name: "source".to_owned(),
        domain: "test".to_owned(),
        digest: hash(seed),
    }
}

fn side_tables(seed: u8) -> IrSideTables {
    IrSideTables {
        source_maps: vec![SideTableRecord::new("source-map", "Main", hash(seed))],
        diagnostics: vec![SideTableRecord::new("diagnostic", "D001", hash(seed + 1))],
        explanation_refs: Vec::new(),
        documentation_attachments: Vec::new(),
    }
}

fn string_decoder() -> BlobDecoder<String> {
    BlobDecoder::new(|bytes| {
        String::from_utf8(bytes.to_vec()).map_err(|error| BlobDecodeError::new(error.to_string()))
    })
}

fn publisher(snapshot: BuildSnapshotId, units: &[String]) -> Arc<PhaseOutputPublisher> {
    let publisher = Arc::new(PhaseOutputPublisher::new(
        Arc::new(IrStorageService::new()),
        Arc::new(SnapshotHandleRegistry::new()),
    ));
    publisher.register_current_snapshot(snapshot);
    for unit in units {
        publisher.allow_work_unit(AllowedWorkUnit::new(
            phase(),
            output_kind(),
            work_unit(unit),
        ));
    }
    publisher
}

fn try_publish_text(
    publisher: &PhaseOutputPublisher,
    snapshot: BuildSnapshotId,
    unit: &str,
    payload: &str,
    parents: Vec<AnyPhaseOutputRef>,
    side_table_seed: u8,
) -> Result<PhaseOutputRef<String>, PublishError> {
    publisher.publish(PublishOutputInput {
        slot: publisher.allocate(snapshot, phase(), work_unit(unit), output_kind(), schema()),
        snapshot,
        phase: phase(),
        work_unit: work_unit(unit),
        output_kind: output_kind(),
        schema_version: schema(),
        payload: payload.to_owned(),
        canonical_payload: Some(payload.as_bytes().to_vec()),
        decode: string_decoder(),
        parents,
        named_input_hashes: vec![named(1)],
        side_tables: side_tables(side_table_seed),
        origin: OutputOrigin::PackageSource,
        target: PublicationTarget::CurrentPackage,
    })
}

fn publish_text(
    publisher: &PhaseOutputPublisher,
    snapshot: BuildSnapshotId,
    unit: &str,
    payload: &str,
    parents: Vec<AnyPhaseOutputRef>,
    side_table_seed: u8,
) -> PhaseOutputRef<String> {
    try_publish_text(publisher, snapshot, unit, payload, parents, side_table_seed)
        .expect("test output publishes")
}

fn identity_registry_values(snapshot: BuildSnapshotId) -> (Hash, Hash, Hash, Hash) {
    let registry = SnapshotHandleRegistry::new();
    registry.register_snapshot(snapshot);
    let module = registry
        .module_id(ModuleIdentityInput {
            snapshot,
            package_id: "pkg".to_owned(),
            module_path: "pkg/main".to_owned(),
            source_id: None,
            source_hash: hash(10),
        })
        .expect("module id derives");
    let item = registry
        .item_id(ItemIdentityInput {
            snapshot,
            module,
            item_kind: "theorem".to_owned(),
            origin_key: "Th1".to_owned(),
            declaration_order_key: "0001".to_owned(),
        })
        .expect("item id derives");
    let expr = registry
        .expr_id(ExprIdentityInput {
            snapshot,
            module,
            item: Some(item),
            expression_kind: "term".to_owned(),
            producer_path_key: "body/0".to_owned(),
        })
        .expect("expr id derives");
    let vc = registry
        .vc_id(VcIdentityInput {
            snapshot,
            module,
            item: Some(item),
            obligation_order_key: "goal/0".to_owned(),
            canonical_vc_fingerprint: Some(hash(11)),
        })
        .expect("vc id derives");

    (module.hash(), item.hash(), expr.hash(), vc.hash())
}

#[test]
fn identical_inputs_yield_identical_ids_hashes_and_lineage() {
    let snapshot = snapshot(1);
    assert_eq!(
        identity_registry_values(snapshot),
        identity_registry_values(snapshot)
    );

    let units = ["parent-a", "parent-b", "child"]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let left = publisher(snapshot, &units);
    let right = publisher(snapshot, &units);

    let left_parent_a = publish_text(&left, snapshot, "parent-a", "a", Vec::new(), 20);
    let left_parent_b = publish_text(&left, snapshot, "parent-b", "b", Vec::new(), 21);
    let right_parent_a = publish_text(&right, snapshot, "parent-a", "a", Vec::new(), 20);
    let right_parent_b = publish_text(&right, snapshot, "parent-b", "b", Vec::new(), 21);
    let left_child = publish_text(
        &left,
        snapshot,
        "child",
        "child",
        vec![left_parent_a.erase(), left_parent_b.erase()],
        22,
    );
    let right_child = publish_text(
        &right,
        snapshot,
        "child",
        "child",
        vec![right_parent_b.erase(), right_parent_a.erase()],
        22,
    );

    assert_eq!(left_child.output(), right_child.output());
    assert_eq!(left_child.content_hash(), right_child.content_hash());
    assert_eq!(left_child.side_table_hash(), right_child.side_table_hash());
    assert_eq!(left_child.lineage(), right_child.lineage());
}

#[test]
fn collected_handles_fail_closed_before_reuse() {
    let snapshot = snapshot(2);
    let units = ["parent", "child"]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let publisher = publisher(snapshot, &units);
    let parent = publish_text(&publisher, snapshot, "parent", "parent", Vec::new(), 30);

    let summary = publisher.storage().collect(CollectInput {
        snapshot,
        protected_outputs: Vec::new(),
    });
    assert_eq!(summary.outputs_dropped, 1);
    assert!(matches!(
        publisher.storage().get(&parent),
        Err(StorageError::CollectedOutput { output }) if output == parent.output()
    ));

    let error = try_publish_text(
        &publisher,
        snapshot,
        "child",
        "child",
        vec![parent.erase()],
        31,
    )
    .expect_err("collected parent fails before reuse");
    assert!(matches!(
        error,
        PublishError::Storage { error }
            if matches!(*error, StorageError::CollectedOutput { output } if output == parent.output())
    ));
}

#[test]
fn collection_is_idempotent_after_snapshot_replacement() {
    let old_snapshot = snapshot(3);
    let new_snapshot = snapshot(4);
    let units = ["old"].into_iter().map(str::to_owned).collect::<Vec<_>>();
    let publisher = publisher(old_snapshot, &units);
    let old = publish_text(&publisher, old_snapshot, "old", "old", Vec::new(), 40);

    publisher
        .replace_current_snapshot(old_snapshot, new_snapshot)
        .expect("snapshot replacement succeeds");
    let first = publisher.storage().collect(CollectInput {
        snapshot: old_snapshot,
        protected_outputs: Vec::new(),
    });
    let second = publisher.storage().collect(CollectInput {
        snapshot: old_snapshot,
        protected_outputs: Vec::new(),
    });

    assert_eq!(first.outputs_dropped, 1);
    assert_eq!(second.outputs_dropped, 0);
    assert_eq!(second.retained_outputs, 0);
    assert_eq!(second.protected_outputs, 0);
    assert!(matches!(
        publisher.storage().get(&old),
        Err(StorageError::CollectedOutput { output }) if output == old.output()
    ));
    assert!(matches!(
        publisher.validate_current_snapshot(old_snapshot),
        Err(PublishError::ObsoleteSnapshot { snapshot }) if snapshot == old_snapshot
    ));
    publisher
        .validate_current_snapshot(new_snapshot)
        .expect("replacement snapshot remains current");
}

#[test]
fn randomized_retain_release_sequence_does_not_leak_lifetimes() {
    let snapshot = snapshot(5);
    let units = (0..12)
        .map(|index| format!("unit-{index}"))
        .collect::<Vec<_>>();
    let publisher = publisher(snapshot, &units);
    let handles = units
        .iter()
        .enumerate()
        .map(|(index, unit)| {
            publish_text(
                &publisher,
                snapshot,
                unit,
                &format!("payload-{index}"),
                Vec::new(),
                50 + index as u8,
            )
        })
        .collect::<Vec<_>>();
    let owners = (0..5)
        .map(|index| RetainOwner::new(format!("owner-{index}")))
        .collect::<Vec<_>>();
    let mut live = HashSet::<(usize, usize)>::new();
    let mut rng = DeterministicRng::new(0x5eed_f00d_cafe_babe);

    for _ in 0..128 {
        let output = rng.index(handles.len());
        let owner = rng.index(owners.len());
        if rng.next_bool() {
            publisher
                .storage()
                .retain(handles[output].output(), owners[owner].clone())
                .expect("retain succeeds before collection");
            live.insert((output, owner));
        } else {
            publisher
                .storage()
                .release(handles[output].output(), &owners[owner])
                .expect("release succeeds before collection");
            live.remove(&(output, owner));
        }
    }

    publisher
        .storage()
        .retain(handles[0].output(), owners[0].clone())
        .expect("at least one output remains live");
    live.insert((0, 0));
    for (owner_index, owner) in owners.iter().enumerate() {
        publisher
            .storage()
            .release(handles[handles.len() - 1].output(), owner)
            .expect("release succeeds before collection");
        live.remove(&(handles.len() - 1, owner_index));
    }

    let retained_outputs = retained_output_count(handles.len(), &live);
    let first = publisher.storage().collect(CollectInput {
        snapshot,
        protected_outputs: Vec::new(),
    });
    assert_eq!(first.retained_outputs, retained_outputs);
    assert_eq!(first.outputs_dropped, handles.len() - retained_outputs);

    for (index, handle) in handles.iter().enumerate() {
        if output_has_owner(index, &live) {
            assert_eq!(
                &*publisher
                    .storage()
                    .get(handle)
                    .expect("retained output survives collection"),
                &format!("payload-{index}")
            );
        } else {
            assert!(matches!(
                publisher.storage().get(handle),
                Err(StorageError::CollectedOutput { output }) if output == handle.output()
            ));
        }
    }

    for (output_index, owner_index) in live.iter().copied().collect::<Vec<_>>() {
        publisher
            .storage()
            .release(handles[output_index].output(), &owners[owner_index])
            .expect("release of retained owner succeeds");
    }
    let second = publisher.storage().collect(CollectInput {
        snapshot,
        protected_outputs: Vec::new(),
    });
    assert_eq!(second.outputs_dropped, retained_outputs);
    assert_eq!(second.retained_outputs, 0);
    for handle in &handles {
        assert!(matches!(
            publisher.storage().get(handle),
            Err(StorageError::CollectedOutput { output }) if output == handle.output()
        ));
    }
    let third = publisher.storage().collect(CollectInput {
        snapshot,
        protected_outputs: Vec::new(),
    });
    assert_eq!(third.outputs_dropped, 0);
    assert_eq!(third.retained_outputs, 0);
}

fn retained_output_count(output_count: usize, live: &HashSet<(usize, usize)>) -> usize {
    (0..output_count)
        .filter(|output| output_has_owner(*output, live))
        .count()
}

fn output_has_owner(output: usize, live: &HashSet<(usize, usize)>) -> bool {
    live.iter()
        .any(|(live_output, _owner)| *live_output == output)
}

struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 0
    }

    fn index(&mut self, len: usize) -> usize {
        (self.next_u64() as usize) % len
    }
}
