use mizar_diagnostics::fix::{
    FixApplicability, FixCommandRef, FixEdit, FixSafety, FixSuggestion, FixSuggestionError,
    FixSuggestionId, FixSuggestionInput,
};
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId, SourceRange,
};

#[test]
fn structured_fix_round_trip_orders_edits_and_renders_debug_snapshot() {
    let snapshot = snapshot_id(1);
    let (source_a, source_b) = source_pair(snapshot);
    let fix = FixSuggestion::new(FixSuggestionInput {
        id: FixSuggestionId::new("syntax.insert_token").expect("valid fix id"),
        producer_key: Some("parser.insert_token".to_owned()),
        title: "insert token".to_owned(),
        applicability: FixApplicability::MachineApplicable,
        safety: FixSafety::SnapshotBound,
        edits: vec![
            FixEdit::new(
                SourceRange {
                    source_id: source_b,
                    start: 4,
                    end: 6,
                },
                "B",
                Some("bb".to_owned()),
            )
            .expect("valid edit"),
            FixEdit::new(
                SourceRange {
                    source_id: source_a,
                    start: 1,
                    end: 1,
                },
                "A",
                None,
            )
            .expect("valid edit"),
        ],
        command: None,
        required_snapshot: Some(snapshot),
        required_text_hash: None,
    })
    .expect("valid structured fix");

    assert_eq!(fix.id().identity(), "syntax.insert_token");
    assert_eq!(fix.producer_key(), Some("parser.insert_token"));
    assert_eq!(fix.title(), "insert token");
    assert_eq!(fix.applicability(), FixApplicability::MachineApplicable);
    assert_eq!(fix.safety(), FixSafety::SnapshotBound);
    assert_eq!(fix.required_snapshot(), Some(snapshot));
    assert_eq!(fix.required_text_hash(), None);
    assert_eq!(fix.edits()[0].range().source_id, source_a);
    assert_eq!(fix.edits()[0].replacement(), "A");
    assert_eq!(fix.edits()[0].expected_text(), None);
    assert_eq!(fix.edits()[1].range().source_id, source_b);
    assert_eq!(fix.edits()[1].replacement(), "B");
    assert_eq!(fix.edits()[1].expected_text(), Some("bb"));
    assert_eq!(
        fix.debug_snapshot(),
        concat!(
            "kind=fix\n",
            "id=\"syntax.insert_token\"\n",
            "producer_key=\"parser.insert_token\"\n",
            "diagnostic=unpublished\n",
            "title=\"insert token\"\n",
            "applicability=machine_applicable\n",
            "safety=snapshot_bound\n",
            "edits=[{range=SourceId(OpaqueId(1)):1..1,replacement=\"A\",expected_text=none}, ",
            "{range=SourceId(OpaqueId(2)):4..6,replacement=\"B\",expected_text=\"bb\"}]\n",
            "command=none\n",
            "required_snapshot=mizar-session-build-snapshot-v1:",
            "0101010101010101010101010101010101010101010101010101010101010101\n",
            "required_text_hash=none\n",
        )
    );
}

#[test]
fn invalid_edit_ranges_and_overlaps_are_rejected() {
    let snapshot = snapshot_id(2);
    let source_id = source_id(snapshot);
    assert!(matches!(
        FixEdit::new(
            SourceRange {
                source_id,
                start: 9,
                end: 2,
            },
            "",
            None,
        ),
        Err(FixSuggestionError::InvalidRange { start: 9, end: 2 })
    ));

    let overlapping = FixSuggestion::local_text_edit(
        FixSuggestionId::new("syntax.overlap").expect("valid fix id"),
        "overlap",
        FixApplicability::MachineApplicable,
        vec![
            FixEdit::new(
                SourceRange {
                    source_id,
                    start: 0,
                    end: 3,
                },
                "abc",
                None,
            )
            .expect("valid first edit"),
            FixEdit::new(
                SourceRange {
                    source_id,
                    start: 2,
                    end: 4,
                },
                "cd",
                None,
            )
            .expect("valid second edit"),
        ],
    );
    assert!(matches!(
        overlapping,
        Err(FixSuggestionError::OverlappingEdits { .. })
    ));
}

#[test]
fn safety_rules_and_preconditions_are_validated_without_applying_edits() {
    let snapshot = snapshot_id(3);
    let source_id = source_id(snapshot);
    let edit = FixEdit::new(
        SourceRange {
            source_id,
            start: 1,
            end: 1,
        },
        "x",
        Some(String::new()),
    )
    .expect("valid edit");

    assert!(matches!(
        FixSuggestion::new(FixSuggestionInput {
            id: FixSuggestionId::new("syntax.bad_producer").expect("valid fix id"),
            producer_key: Some("bad-key".to_owned()),
            title: "bad producer".to_owned(),
            applicability: FixApplicability::Informational,
            safety: FixSafety::CommandOnly,
            edits: Vec::new(),
            command: None,
            required_snapshot: None,
            required_text_hash: None,
        }),
        Err(FixSuggestionError::InvalidProducerKey { key }) if key == "bad-key"
    ));
    assert!(matches!(
        FixCommandRef::new("bad command"),
        Err(FixSuggestionError::InvalidCommandRef { identity }) if identity == "bad command"
    ));
    assert!(matches!(
        FixSuggestion::new(FixSuggestionInput {
            id: FixSuggestionId::new("syntax.informational_edit").expect("valid fix id"),
            producer_key: None,
            title: "informational edit".to_owned(),
            applicability: FixApplicability::Informational,
            safety: FixSafety::LocalTextEdit,
            edits: vec![edit.clone()],
            command: None,
            required_snapshot: None,
            required_text_hash: None,
        }),
        Err(FixSuggestionError::InformationalFixHasEdits { id })
            if id == "syntax.informational_edit"
    ));
    assert!(matches!(
        FixSuggestion::new(FixSuggestionInput {
            id: FixSuggestionId::new("syntax.edit_command").expect("valid fix id"),
            producer_key: None,
            title: "edit command".to_owned(),
            applicability: FixApplicability::MachineApplicable,
            safety: FixSafety::LocalTextEdit,
            edits: vec![edit.clone()],
            command: Some(FixCommandRef::new("syntax.command").expect("valid command")),
            required_snapshot: None,
            required_text_hash: None,
        }),
        Err(FixSuggestionError::EditFixHasCommand { id }) if id == "syntax.edit_command"
    ));
    assert!(matches!(
        FixSuggestion::new(FixSuggestionInput {
            id: FixSuggestionId::new("syntax.actionless").expect("valid fix id"),
            producer_key: None,
            title: "actionless".to_owned(),
            applicability: FixApplicability::MaybeIncorrect,
            safety: FixSafety::CommandOnly,
            edits: Vec::new(),
            command: None,
            required_snapshot: None,
            required_text_hash: None,
        }),
        Err(FixSuggestionError::ActionlessNonInformationalFix { id })
            if id == "syntax.actionless"
    ));
    assert!(matches!(
        FixSuggestion::new(FixSuggestionInput {
            id: FixSuggestionId::new("syntax.missing_edit").expect("valid fix id"),
            producer_key: None,
            title: "missing edit".to_owned(),
            applicability: FixApplicability::Informational,
            safety: FixSafety::LocalTextEdit,
            edits: Vec::new(),
            command: None,
            required_snapshot: None,
            required_text_hash: None,
        }),
        Err(FixSuggestionError::MissingEditsForEditSafety {
            id,
            safety: FixSafety::LocalTextEdit
        }) if id == "syntax.missing_edit"
    ));
    assert!(matches!(
        FixSuggestion::new(FixSuggestionInput {
            id: FixSuggestionId::new("syntax.snapshot").expect("valid fix id"),
            producer_key: None,
            title: "snapshot".to_owned(),
            applicability: FixApplicability::MachineApplicable,
            safety: FixSafety::SnapshotBound,
            edits: vec![edit.clone()],
            command: None,
            required_snapshot: None,
            required_text_hash: None,
        }),
        Err(FixSuggestionError::SnapshotBoundMissingSnapshot { id })
            if id == "syntax.snapshot"
    ));
    assert!(matches!(
        FixSuggestion::new(FixSuggestionInput {
            id: FixSuggestionId::new("syntax.artifact").expect("valid fix id"),
            producer_key: None,
            title: "artifact".to_owned(),
            applicability: FixApplicability::MachineApplicable,
            safety: FixSafety::ArtifactAssisted,
            edits: vec![edit.clone()],
            command: None,
            required_snapshot: None,
            required_text_hash: None,
        }),
        Err(FixSuggestionError::ArtifactAssistedMissingHash { id })
            if id == "syntax.artifact"
    ));
    assert!(matches!(
        FixSuggestion::new(FixSuggestionInput {
            id: FixSuggestionId::new("syntax.command_edit").expect("valid fix id"),
            producer_key: None,
            title: "command with edit".to_owned(),
            applicability: FixApplicability::MachineApplicable,
            safety: FixSafety::CommandOnly,
            edits: vec![edit.clone()],
            command: Some(FixCommandRef::new("syntax.command").expect("valid command")),
            required_snapshot: None,
            required_text_hash: None,
        }),
        Err(FixSuggestionError::CommandOnlyFixHasEdits { id })
            if id == "syntax.command_edit"
    ));

    let command = FixSuggestion::new(FixSuggestionInput {
        id: FixSuggestionId::new("syntax.open_help").expect("valid fix id"),
        producer_key: None,
        title: "open help".to_owned(),
        applicability: FixApplicability::MaybeIncorrect,
        safety: FixSafety::CommandOnly,
        edits: Vec::new(),
        command: Some(FixCommandRef::new("syntax.open_command").expect("valid command")),
        required_snapshot: None,
        required_text_hash: None,
    })
    .expect("command-only fix is valid");
    assert_eq!(
        command.command().map(FixCommandRef::identity),
        Some("syntax.open_command")
    );

    let artifact = FixSuggestion::new(FixSuggestionInput {
        id: FixSuggestionId::new("syntax.artifact_ok").expect("valid fix id"),
        producer_key: None,
        title: "artifact assisted".to_owned(),
        applicability: FixApplicability::MachineApplicable,
        safety: FixSafety::ArtifactAssisted,
        edits: vec![edit],
        command: None,
        required_snapshot: None,
        required_text_hash: Some(Hash::from_bytes([7; Hash::BYTE_LEN])),
    })
    .expect("artifact-assisted fix has hash precondition");
    assert_eq!(
        artifact.debug_snapshot_with_diagnostic("snapshot#1"),
        concat!(
            "kind=fix\n",
            "id=\"syntax.artifact_ok\"\n",
            "producer_key=none\n",
            "diagnostic=snapshot#1\n",
            "title=\"artifact assisted\"\n",
            "applicability=machine_applicable\n",
            "safety=artifact_assisted\n",
            "edits=[{range=SourceId(OpaqueId(1)):1..1,replacement=\"x\",expected_text=\"\"}]\n",
            "command=none\n",
            "required_snapshot=none\n",
            "required_text_hash=0707070707070707070707070707070707070707070707070707070707070707\n",
        )
    );
}

fn snapshot_id(byte: u8) -> BuildSnapshotId {
    let hex = format!("{byte:02x}");
    BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        hex.repeat(32)
    ))
    .expect("test snapshot id is valid")
}

fn source_id(snapshot: BuildSnapshotId) -> SourceId {
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .expect("source id allocation succeeds")
}

fn source_pair(snapshot: BuildSnapshotId) -> (SourceId, SourceId) {
    let allocator = InMemorySessionIdAllocator::new();
    (
        allocator
            .next_source_id(snapshot)
            .expect("first source id allocation succeeds"),
        allocator
            .next_source_id(snapshot)
            .expect("second source id allocation succeeds"),
    )
}
