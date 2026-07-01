//! Structured diagnostic fix suggestions.

use std::{cmp::Ordering, error::Error, fmt};

use mizar_session::{BuildSnapshotId, Hash, SourceRange};

/// Stable producer-side identity for one fix suggestion attached to a diagnostic.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FixSuggestionId {
    identity: String,
}

impl FixSuggestionId {
    /// Creates a fix-suggestion identity.
    pub fn new(identity: impl Into<String>) -> Result<Self, FixSuggestionError> {
        let identity = identity.into();
        validate_identity(&identity).map_err(|_| FixSuggestionError::InvalidFixIdentity {
            identity: identity.clone(),
        })?;
        Ok(Self { identity })
    }

    /// Returns the stable identity string.
    pub fn identity(&self) -> &str {
        &self.identity
    }
}

/// Opaque command reference for a later owner.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FixCommandRef {
    identity: String,
}

impl FixCommandRef {
    /// Creates a command reference.
    pub fn new(identity: impl Into<String>) -> Result<Self, FixSuggestionError> {
        let identity = identity.into();
        validate_identity(&identity).map_err(|_| FixSuggestionError::InvalidCommandRef {
            identity: identity.clone(),
        })?;
        Ok(Self { identity })
    }

    /// Returns the stable command reference string.
    pub fn identity(&self) -> &str {
        &self.identity
    }
}

/// Whether a consumer may offer a fix mechanically or should require review.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum FixApplicability {
    /// The edit is expected to be mechanically correct when preconditions hold.
    MachineApplicable,
    /// The edit is plausible but should be reviewed.
    MaybeIncorrect,
    /// The replacement text contains placeholders the user must fill.
    HasPlaceholders,
    /// No direct edit is provided; the suggestion is informational help.
    Informational,
}

impl FixApplicability {
    /// Stable lowercase rendering for debug/test snapshots.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MachineApplicable => "machine_applicable",
            Self::MaybeIncorrect => "maybe_incorrect",
            Self::HasPlaceholders => "has_placeholders",
            Self::Informational => "informational",
        }
    }
}

impl fmt::Display for FixApplicability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Safety boundary for offering a fix.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum FixSafety {
    /// All edits target explicit source ranges.
    LocalTextEdit,
    /// The suggestion is valid only for the required snapshot.
    SnapshotBound,
    /// The suggestion depends on source/artifact hashes checked elsewhere.
    ArtifactAssisted,
    /// No text edit is provided; a later owner may interpret the command ref.
    CommandOnly,
}

impl FixSafety {
    /// Stable lowercase rendering for debug/test snapshots.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LocalTextEdit => "local_text_edit",
            Self::SnapshotBound => "snapshot_bound",
            Self::ArtifactAssisted => "artifact_assisted",
            Self::CommandOnly => "command_only",
        }
    }
}

impl fmt::Display for FixSafety {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One source edit carried by a fix suggestion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixEdit {
    range: SourceRange,
    replacement: String,
    expected_text: Option<String>,
}

impl FixEdit {
    /// Creates a validated text edit.
    pub fn new(
        range: SourceRange,
        replacement: impl Into<String>,
        expected_text: Option<String>,
    ) -> Result<Self, FixSuggestionError> {
        validate_range(range)?;
        Ok(Self {
            range,
            replacement: replacement.into(),
            expected_text,
        })
    }

    /// Returns the compiler-native source range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the replacement text.
    pub fn replacement(&self) -> &str {
        &self.replacement
    }

    /// Returns expected current text, if supplied.
    pub fn expected_text(&self) -> Option<&str> {
        self.expected_text.as_deref()
    }
}

/// Input used to create a structured fix suggestion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixSuggestionInput {
    /// Stable fix id within the diagnostic draft or record.
    pub id: FixSuggestionId,
    /// Optional producer-supplied identity string.
    pub producer_key: Option<String>,
    /// Human-facing title. This is not identity.
    pub title: String,
    /// Applicability level.
    pub applicability: FixApplicability,
    /// Safety classification.
    pub safety: FixSafety,
    /// Ordered source edits after validation.
    pub edits: Vec<FixEdit>,
    /// Optional command reference for a later owner.
    pub command: Option<FixCommandRef>,
    /// Snapshot precondition.
    pub required_snapshot: Option<BuildSnapshotId>,
    /// Text or artifact hash precondition.
    pub required_text_hash: Option<Hash>,
}

/// Structured advisory fix suggestion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixSuggestion {
    id: FixSuggestionId,
    producer_key: Option<String>,
    title: String,
    applicability: FixApplicability,
    safety: FixSafety,
    edits: Vec<FixEdit>,
    command: Option<FixCommandRef>,
    required_snapshot: Option<BuildSnapshotId>,
    required_text_hash: Option<Hash>,
}

impl FixSuggestion {
    /// Creates and canonicalizes a structured fix suggestion.
    pub fn new(input: FixSuggestionInput) -> Result<Self, FixSuggestionError> {
        if let Some(producer_key) = &input.producer_key {
            validate_identity(producer_key).map_err(|_| {
                FixSuggestionError::InvalidProducerKey {
                    key: producer_key.clone(),
                }
            })?;
        }

        let mut edits = input.edits;
        edits.sort_by(compare_edits);
        validate_non_overlapping_edits(&edits)?;

        let suggestion = Self {
            id: input.id,
            producer_key: input.producer_key,
            title: input.title,
            applicability: input.applicability,
            safety: input.safety,
            edits,
            command: input.command,
            required_snapshot: input.required_snapshot,
            required_text_hash: input.required_text_hash,
        };
        suggestion.validate_shape()?;
        Ok(suggestion)
    }

    /// Creates an informational, non-edit fix suggestion.
    pub fn informational(
        id: FixSuggestionId,
        title: impl Into<String>,
    ) -> Result<Self, FixSuggestionError> {
        Self::new(FixSuggestionInput {
            id,
            producer_key: None,
            title: title.into(),
            applicability: FixApplicability::Informational,
            safety: FixSafety::CommandOnly,
            edits: Vec::new(),
            command: None,
            required_snapshot: None,
            required_text_hash: None,
        })
    }

    /// Creates a local text-edit suggestion.
    pub fn local_text_edit(
        id: FixSuggestionId,
        title: impl Into<String>,
        applicability: FixApplicability,
        edits: Vec<FixEdit>,
    ) -> Result<Self, FixSuggestionError> {
        Self::new(FixSuggestionInput {
            id,
            producer_key: None,
            title: title.into(),
            applicability,
            safety: FixSafety::LocalTextEdit,
            edits,
            command: None,
            required_snapshot: None,
            required_text_hash: None,
        })
    }

    /// Returns the stable fix id.
    pub const fn id(&self) -> &FixSuggestionId {
        &self.id
    }

    /// Returns the optional producer identity string.
    pub fn producer_key(&self) -> Option<&str> {
        self.producer_key.as_deref()
    }

    /// Returns the human-facing title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns applicability metadata.
    pub const fn applicability(&self) -> FixApplicability {
        self.applicability
    }

    /// Returns safety metadata.
    pub const fn safety(&self) -> FixSafety {
        self.safety
    }

    /// Returns canonicalized edits.
    pub fn edits(&self) -> &[FixEdit] {
        &self.edits
    }

    /// Returns the optional command reference.
    pub const fn command(&self) -> Option<&FixCommandRef> {
        self.command.as_ref()
    }

    /// Returns the required snapshot precondition.
    pub const fn required_snapshot(&self) -> Option<BuildSnapshotId> {
        self.required_snapshot
    }

    /// Returns the required text or artifact hash precondition.
    pub const fn required_text_hash(&self) -> Option<Hash> {
        self.required_text_hash
    }

    /// Returns a deterministic debug/test snapshot without a published handle.
    pub fn debug_snapshot(&self) -> String {
        self.debug_snapshot_with_diagnostic("unpublished")
    }

    /// Returns a deterministic debug/test snapshot with a caller-supplied
    /// diagnostic identity projection.
    pub fn debug_snapshot_with_diagnostic(&self, diagnostic: &str) -> String {
        let lines = vec![
            "kind=fix".to_owned(),
            format!("id={:?}", self.id.identity),
            format!(
                "producer_key={}",
                render_optional_string(self.producer_key.as_deref())
            ),
            format!("diagnostic={diagnostic}"),
            format!("title={:?}", self.title),
            format!("applicability={}", self.applicability),
            format!("safety={}", self.safety),
            format!("edits={}", render_edits(&self.edits)),
            format!(
                "command={}",
                self.command.as_ref().map_or_else(
                    || "none".to_owned(),
                    |command| format!("{:?}", command.identity)
                )
            ),
            format!(
                "required_snapshot={}",
                self.required_snapshot
                    .map_or_else(|| "none".to_owned(), render_snapshot)
            ),
            format!(
                "required_text_hash={}",
                self.required_text_hash
                    .map_or_else(|| "none".to_owned(), render_hash)
            ),
        ];
        let mut rendered = lines.join("\n");
        rendered.push('\n');
        rendered
    }

    pub(crate) fn canonical_key(&self) -> FixSuggestionKey {
        FixSuggestionKey {
            id: self.id.identity.clone(),
            producer_key: self.producer_key.clone(),
            applicability: self.applicability,
            safety: self.safety,
            edits: self.edits.iter().map(FixEditKey::from_edit).collect(),
            command: self
                .command
                .as_ref()
                .map(|command| command.identity.clone()),
            required_snapshot: self.required_snapshot.map(render_snapshot),
            required_text_hash: self.required_text_hash.map(render_hash),
        }
    }

    fn validate_shape(&self) -> Result<(), FixSuggestionError> {
        let has_edits = !self.edits.is_empty();
        if self.applicability == FixApplicability::Informational && has_edits {
            return Err(FixSuggestionError::InformationalFixHasEdits {
                id: self.id.identity.clone(),
            });
        }
        if self.safety == FixSafety::CommandOnly && has_edits {
            return Err(FixSuggestionError::CommandOnlyFixHasEdits {
                id: self.id.identity.clone(),
            });
        }
        if self.command.is_some() && has_edits {
            return Err(FixSuggestionError::EditFixHasCommand {
                id: self.id.identity.clone(),
            });
        }
        if !has_edits
            && self.command.is_none()
            && self.applicability != FixApplicability::Informational
        {
            return Err(FixSuggestionError::ActionlessNonInformationalFix {
                id: self.id.identity.clone(),
            });
        }
        if !has_edits && self.safety != FixSafety::CommandOnly {
            return Err(FixSuggestionError::MissingEditsForEditSafety {
                id: self.id.identity.clone(),
                safety: self.safety,
            });
        }
        if self.safety == FixSafety::SnapshotBound && self.required_snapshot.is_none() {
            return Err(FixSuggestionError::SnapshotBoundMissingSnapshot {
                id: self.id.identity.clone(),
            });
        }
        if self.safety == FixSafety::ArtifactAssisted && self.required_text_hash.is_none() {
            return Err(FixSuggestionError::ArtifactAssistedMissingHash {
                id: self.id.identity.clone(),
            });
        }
        Ok(())
    }
}

/// Error returned while constructing structured fix suggestions.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FixSuggestionError {
    /// The fix identity did not use structured identity grammar.
    InvalidFixIdentity {
        /// Rejected identity.
        identity: String,
    },
    /// The producer key did not use structured identity grammar.
    InvalidProducerKey {
        /// Rejected producer key.
        key: String,
    },
    /// The command reference did not use structured identity grammar.
    InvalidCommandRef {
        /// Rejected command reference.
        identity: String,
    },
    /// Source range had `start > end`.
    InvalidRange {
        /// Range start.
        start: usize,
        /// Range end.
        end: usize,
    },
    /// Two edits in one suggestion overlap.
    OverlappingEdits {
        /// First overlapping range.
        first: SourceRange,
        /// Second overlapping range.
        second: SourceRange,
    },
    /// Informational fixes cannot carry source edits.
    InformationalFixHasEdits {
        /// Rejected fix id.
        id: String,
    },
    /// Command-only fixes cannot carry source edits.
    CommandOnlyFixHasEdits {
        /// Rejected fix id.
        id: String,
    },
    /// A text-edit fix carried a command reference.
    EditFixHasCommand {
        /// Rejected fix id.
        id: String,
    },
    /// A non-informational fix had neither edits nor a command.
    ActionlessNonInformationalFix {
        /// Rejected fix id.
        id: String,
    },
    /// A non-command safety class had no edits.
    MissingEditsForEditSafety {
        /// Rejected fix id.
        id: String,
        /// Rejected safety class.
        safety: FixSafety,
    },
    /// Snapshot-bound fixes require a snapshot precondition.
    SnapshotBoundMissingSnapshot {
        /// Rejected fix id.
        id: String,
    },
    /// Artifact-assisted fixes require a hash precondition.
    ArtifactAssistedMissingHash {
        /// Rejected fix id.
        id: String,
    },
}

impl fmt::Display for FixSuggestionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFixIdentity { identity } => {
                write!(formatter, "invalid fix identity `{identity}`")
            }
            Self::InvalidProducerKey { key } => {
                write!(formatter, "invalid fix producer key `{key}`")
            }
            Self::InvalidCommandRef { identity } => {
                write!(formatter, "invalid fix command reference `{identity}`")
            }
            Self::InvalidRange { start, end } => {
                write!(formatter, "fix edit range start {start} exceeds end {end}")
            }
            Self::OverlappingEdits { first, second } => write!(
                formatter,
                "fix edit ranges {}..{} and {}..{} overlap",
                first.start, first.end, second.start, second.end
            ),
            Self::InformationalFixHasEdits { id } => {
                write!(formatter, "informational fix `{id}` cannot carry edits")
            }
            Self::CommandOnlyFixHasEdits { id } => {
                write!(formatter, "command-only fix `{id}` cannot carry edits")
            }
            Self::EditFixHasCommand { id } => {
                write!(formatter, "text-edit fix `{id}` cannot carry a command ref")
            }
            Self::ActionlessNonInformationalFix { id } => write!(
                formatter,
                "non-informational fix `{id}` requires edits or a command ref"
            ),
            Self::MissingEditsForEditSafety { id, safety } => write!(
                formatter,
                "fix `{id}` with {safety} safety requires source edits"
            ),
            Self::SnapshotBoundMissingSnapshot { id } => {
                write!(formatter, "snapshot-bound fix `{id}` requires a snapshot")
            }
            Self::ArtifactAssistedMissingHash { id } => {
                write!(formatter, "artifact-assisted fix `{id}` requires a hash")
            }
        }
    }
}

impl Error for FixSuggestionError {}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct FixSuggestionKey {
    id: String,
    producer_key: Option<String>,
    applicability: FixApplicability,
    safety: FixSafety,
    edits: Vec<FixEditKey>,
    command: Option<String>,
    required_snapshot: Option<String>,
    required_text_hash: Option<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct FixEditKey {
    source: String,
    start: usize,
    end: usize,
    replacement: String,
    expected_text: Option<String>,
}

impl FixEditKey {
    fn from_edit(edit: &FixEdit) -> Self {
        Self {
            source: source_key(edit.range),
            start: edit.range.start,
            end: edit.range.end,
            replacement: edit.replacement.clone(),
            expected_text: edit.expected_text.clone(),
        }
    }
}

fn validate_identity(identity: &str) -> Result<(), ()> {
    if identity.is_empty() {
        return Err(());
    }
    for segment in identity.split('.') {
        validate_identity_segment(segment)?;
    }
    Ok(())
}

fn validate_identity_segment(segment: &str) -> Result<(), ()> {
    let bytes = segment.as_bytes();
    let Some((&first, rest)) = bytes.split_first() else {
        return Err(());
    };
    if !first.is_ascii_lowercase() {
        return Err(());
    }

    let mut previous_underscore = false;
    for byte in rest {
        if byte.is_ascii_lowercase() || byte.is_ascii_digit() {
            previous_underscore = false;
        } else if *byte == b'_' {
            if previous_underscore {
                return Err(());
            }
            previous_underscore = true;
        } else {
            return Err(());
        }
    }

    if previous_underscore {
        return Err(());
    }
    Ok(())
}

fn validate_range(range: SourceRange) -> Result<(), FixSuggestionError> {
    if range.start > range.end {
        return Err(FixSuggestionError::InvalidRange {
            start: range.start,
            end: range.end,
        });
    }
    Ok(())
}

fn validate_non_overlapping_edits(edits: &[FixEdit]) -> Result<(), FixSuggestionError> {
    for (left_index, left) in edits.iter().enumerate() {
        for right in &edits[left_index + 1..] {
            if left.range.source_id == right.range.source_id
                && ranges_conflict(left.range, right.range)
            {
                return Err(FixSuggestionError::OverlappingEdits {
                    first: left.range,
                    second: right.range,
                });
            }
        }
    }
    Ok(())
}

fn ranges_conflict(left: SourceRange, right: SourceRange) -> bool {
    if left.start == left.end && right.start == right.end {
        return left.start == right.start;
    }
    left.start < right.end && right.start < left.end
}

fn compare_edits(left: &FixEdit, right: &FixEdit) -> Ordering {
    FixEditKey::from_edit(left).cmp(&FixEditKey::from_edit(right))
}

fn render_edits(edits: &[FixEdit]) -> String {
    let rendered = edits.iter().map(render_edit).collect::<Vec<_>>();
    format!("[{}]", rendered.join(", "))
}

fn render_edit(edit: &FixEdit) -> String {
    format!(
        "{{range={}:{}..{},replacement={:?},expected_text={}}}",
        source_key(edit.range),
        edit.range.start,
        edit.range.end,
        edit.replacement,
        render_optional_string(edit.expected_text.as_deref())
    )
}

fn source_key(range: SourceRange) -> String {
    range
        .source_id
        .to_published_schema_string()
        .unwrap_or_else(|_| format!("{:?}", range.source_id))
}

fn render_snapshot(snapshot: BuildSnapshotId) -> String {
    snapshot
        .to_published_schema_string()
        .unwrap_or_else(|_| format!("{snapshot:?}"))
}

fn render_hash(hash: Hash) -> String {
    let mut rendered = String::with_capacity(Hash::BYTE_LEN * 2);
    for byte in hash.as_bytes() {
        rendered.push_str(&format!("{byte:02x}"));
    }
    rendered
}

fn render_optional_string(value: Option<&str>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| format!("{value:?}"))
}
