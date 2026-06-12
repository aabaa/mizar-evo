use crate::ast::SurfaceNodeId;
use mizar_session::{CommentKind, GeneratedSpanAnchor, SourceAnchor, SourceId, SourceRange};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceTrivia {
    source_id: SourceId,
    comments: Vec<CommentTrivia>,
    doc_comment_attachments: Vec<DocCommentAttachment>,
    skipped_token_ranges: Vec<SkippedTokenRange>,
    whitespace_hints: Vec<WhitespaceHint>,
}

impl SurfaceTrivia {
    pub fn empty(source_id: SourceId) -> Self {
        Self {
            source_id,
            comments: Vec::new(),
            doc_comment_attachments: Vec::new(),
            skipped_token_ranges: Vec::new(),
            whitespace_hints: Vec::new(),
        }
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub fn is_empty(&self) -> bool {
        self.comments.is_empty()
            && self.doc_comment_attachments.is_empty()
            && self.skipped_token_ranges.is_empty()
            && self.whitespace_hints.is_empty()
    }

    pub fn comments(&self) -> &[CommentTrivia] {
        &self.comments
    }

    pub fn doc_comment_attachments(&self) -> &[DocCommentAttachment] {
        &self.doc_comment_attachments
    }

    pub fn skipped_token_ranges(&self) -> &[SkippedTokenRange] {
        &self.skipped_token_ranges
    }

    pub fn whitespace_hints(&self) -> &[WhitespaceHint] {
        &self.whitespace_hints
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceTriviaBuilder {
    source_id: SourceId,
    comments: Vec<CommentTrivia>,
    doc_comment_attachments: Vec<DocCommentAttachment>,
    skipped_token_ranges: Vec<SkippedTokenRange>,
    whitespace_hints: Vec<WhitespaceHint>,
}

impl SurfaceTriviaBuilder {
    pub fn new(source_id: SourceId) -> Self {
        Self {
            source_id,
            comments: Vec::new(),
            doc_comment_attachments: Vec::new(),
            skipped_token_ranges: Vec::new(),
            whitespace_hints: Vec::new(),
        }
    }

    pub fn add_comment(&mut self, kind: CommentKind, range: SourceRange) {
        self.assert_same_source(range);
        self.comments.push(CommentTrivia { kind, range });
    }

    pub fn add_doc_comment_attachment(
        &mut self,
        range: SourceRange,
        target: TriviaAttachmentTarget,
        placement: TriviaPlacement,
    ) {
        self.assert_same_source(range);
        self.assert_same_source_for_target(&target);
        self.doc_comment_attachments.push(DocCommentAttachment {
            range,
            target,
            placement,
        });
    }

    pub fn add_skipped_token_range(
        &mut self,
        range: SourceRange,
        owner: Option<TriviaAttachmentTarget>,
        reason: SkippedTokenReason,
    ) {
        self.assert_same_source(range);
        if let Some(owner) = &owner {
            self.assert_same_source_for_target(owner);
        }
        self.skipped_token_ranges.push(SkippedTokenRange {
            range,
            owner,
            reason,
        });
    }

    pub fn add_whitespace_hint(&mut self, kind: WhitespaceHintKind, range: SourceRange) {
        self.assert_same_source(range);
        self.whitespace_hints.push(WhitespaceHint { kind, range });
    }

    pub fn finish(mut self) -> SurfaceTrivia {
        self.comments.sort_by_key(comment_sort_key);
        self.doc_comment_attachments
            .sort_by_key(doc_comment_attachment_sort_key);
        self.skipped_token_ranges
            .sort_by_key(skipped_token_range_sort_key);
        self.whitespace_hints.sort_by_key(whitespace_hint_sort_key);

        SurfaceTrivia {
            source_id: self.source_id,
            comments: self.comments,
            doc_comment_attachments: self.doc_comment_attachments,
            skipped_token_ranges: self.skipped_token_ranges,
            whitespace_hints: self.whitespace_hints,
        }
    }

    fn assert_same_source(&self, range: SourceRange) {
        assert_eq!(
            range.source_id, self.source_id,
            "surface trivia range must belong to the trivia source"
        );
    }

    fn assert_same_source_for_target(&self, target: &TriviaAttachmentTarget) {
        match target {
            TriviaAttachmentTarget::Node(target) | TriviaAttachmentTarget::Token(target) => {
                self.assert_same_source(target.range);
            }
            TriviaAttachmentTarget::Detached(anchor) => self.assert_same_source_for_anchor(anchor),
        }
    }

    fn assert_same_source_for_anchor(&self, anchor: &SourceAnchor) {
        match anchor {
            SourceAnchor::Range(range) => self.assert_same_source(*range),
            SourceAnchor::Point { source_id, .. } => {
                assert_eq!(
                    *source_id, self.source_id,
                    "surface trivia anchor must belong to the trivia source"
                );
            }
            SourceAnchor::Generated(origin) => {
                self.assert_same_source_for_generated_anchor(origin.anchor());
            }
            _ => {}
        }
    }

    fn assert_same_source_for_generated_anchor(&self, anchor: GeneratedSpanAnchor) {
        match anchor {
            GeneratedSpanAnchor::Range(range) => self.assert_same_source(range),
            GeneratedSpanAnchor::Point { source_id, .. } => {
                assert_eq!(
                    source_id, self.source_id,
                    "surface trivia generated anchor must belong to the trivia source"
                );
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentTrivia {
    pub kind: CommentKind,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocCommentAttachment {
    pub range: SourceRange,
    pub target: TriviaAttachmentTarget,
    pub placement: TriviaPlacement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedTokenRange {
    pub range: SourceRange,
    pub owner: Option<TriviaAttachmentTarget>,
    pub reason: SkippedTokenReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhitespaceHint {
    pub kind: WhitespaceHintKind,
    pub range: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriviaNodeTarget {
    pub id: SurfaceNodeId,
    pub range: SourceRange,
}

impl TriviaNodeTarget {
    pub const fn new(id: SurfaceNodeId, range: SourceRange) -> Self {
        Self { id, range }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriviaAttachmentTarget {
    Node(TriviaNodeTarget),
    Token(TriviaNodeTarget),
    Detached(SourceAnchor),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriviaPlacement {
    Leading,
    Trailing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkippedTokenReason {
    Recovery,
    MalformedAnnotation,
    UnexpectedToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhitespaceHintKind {
    RequiresSeparation,
    LineBreakBefore,
    LineBreakAfter,
    SyntheticBoundary,
}

pub(crate) fn write_trivia_snapshot(
    output: &mut String,
    trivia: &SurfaceTrivia,
    mut target_range: impl FnMut(SurfaceNodeId) -> Option<SourceRange>,
) {
    output.push_str("trivia:\n");
    if trivia.is_empty() {
        output.push_str("  <none>\n");
        return;
    }

    for comment in trivia.comments() {
        output.push_str("  Comment kind=");
        output.push_str(comment_kind_snapshot_name(comment.kind));
        write_range(output, comment.range);
        output.push('\n');
    }
    for attachment in trivia.doc_comment_attachments() {
        output.push_str("  DocComment");
        write_range(output, attachment.range);
        output.push_str(" placement=");
        output.push_str(placement_snapshot_name(attachment.placement));
        output.push_str(" target=");
        write_target(output, &attachment.target, &mut target_range);
        output.push('\n');
    }
    for skipped in trivia.skipped_token_ranges() {
        output.push_str("  SkippedTokens reason=");
        output.push_str(skipped_reason_snapshot_name(skipped.reason));
        write_range(output, skipped.range);
        output.push_str(" owner=");
        match &skipped.owner {
            Some(owner) => write_target(output, owner, &mut target_range),
            None => output.push_str("<none>"),
        }
        output.push('\n');
    }
    for hint in trivia.whitespace_hints() {
        output.push_str("  WhitespaceHint kind=");
        output.push_str(whitespace_hint_snapshot_name(hint.kind));
        write_range(output, hint.range);
        output.push('\n');
    }
}

fn write_range(output: &mut String, range: SourceRange) {
    output.push_str(" range=");
    output.push_str(&range.start.to_string());
    output.push_str("..");
    output.push_str(&range.end.to_string());
}

fn write_target(
    output: &mut String,
    target: &TriviaAttachmentTarget,
    target_range: &mut impl FnMut(SurfaceNodeId) -> Option<SourceRange>,
) {
    match target {
        TriviaAttachmentTarget::Node(node) => {
            output.push_str("node:");
            write_target_range(output, target_range(node.id));
        }
        TriviaAttachmentTarget::Token(token) => {
            output.push_str("token:");
            write_target_range(output, target_range(token.id));
        }
        TriviaAttachmentTarget::Detached(anchor) => write_anchor(output, anchor),
    }
}

fn write_target_range(output: &mut String, range: Option<SourceRange>) {
    match range {
        Some(range) => {
            output.push_str("range:");
            output.push_str(&range.start.to_string());
            output.push_str("..");
            output.push_str(&range.end.to_string());
        }
        None => output.push_str("<missing>"),
    }
}

fn write_anchor(output: &mut String, anchor: &SourceAnchor) {
    match anchor {
        SourceAnchor::Range(range) => {
            output.push_str("detached:range:");
            output.push_str(&range.start.to_string());
            output.push_str("..");
            output.push_str(&range.end.to_string());
        }
        SourceAnchor::Point { offset, .. } => {
            output.push_str("detached:point:");
            output.push_str(&offset.to_string());
        }
        SourceAnchor::Generated(_) => output.push_str("detached:generated"),
        _ => output.push_str("detached:unknown"),
    }
}

fn comment_sort_key(comment: &CommentTrivia) -> (usize, usize, u8) {
    (
        comment.range.start,
        comment.range.end,
        comment_kind_sort_key(comment.kind),
    )
}

fn doc_comment_attachment_sort_key(
    attachment: &DocCommentAttachment,
) -> (usize, usize, u8, String) {
    (
        attachment.range.start,
        attachment.range.end,
        placement_sort_key(attachment.placement),
        target_sort_key(&attachment.target),
    )
}

fn skipped_token_range_sort_key(skipped: &SkippedTokenRange) -> (usize, usize, u8, String) {
    (
        skipped.range.start,
        skipped.range.end,
        skipped_reason_sort_key(skipped.reason),
        skipped
            .owner
            .as_ref()
            .map_or_else(|| "9:none".to_owned(), target_sort_key),
    )
}

fn whitespace_hint_sort_key(hint: &WhitespaceHint) -> (usize, usize, u8) {
    (
        hint.range.start,
        hint.range.end,
        whitespace_hint_sort_key_value(hint.kind),
    )
}

fn target_sort_key(target: &TriviaAttachmentTarget) -> String {
    match target {
        TriviaAttachmentTarget::Node(node) => format!(
            "0:node:{:020}:{:020}:{:020}",
            node.id.index(),
            node.range.start,
            node.range.end
        ),
        TriviaAttachmentTarget::Token(token) => format!(
            "1:token:{:020}:{:020}:{:020}",
            token.id.index(),
            token.range.start,
            token.range.end
        ),
        TriviaAttachmentTarget::Detached(anchor) => detached_anchor_sort_key(anchor),
    }
}

fn detached_anchor_sort_key(anchor: &SourceAnchor) -> String {
    match anchor {
        SourceAnchor::Range(range) => {
            format!("2:detached:range:{:020}:{:020}", range.start, range.end)
        }
        SourceAnchor::Point { offset, .. } => {
            format!("2:detached:point:{offset:020}")
        }
        SourceAnchor::Generated(origin) => match origin.anchor() {
            GeneratedSpanAnchor::Range(range) => format!(
                "2:detached:generated:range:{:020}:{:020}:{}",
                range.start,
                range.end,
                origin.reason()
            ),
            GeneratedSpanAnchor::Point { offset, .. } => {
                format!(
                    "2:detached:generated:point:{offset:020}:{}",
                    origin.reason()
                )
            }
            _ => format!("2:detached:generated:unknown:{}", origin.reason()),
        },
        _ => "2:detached:unknown".to_owned(),
    }
}

fn comment_kind_snapshot_name(kind: CommentKind) -> &'static str {
    match kind {
        CommentKind::SingleLine => "SingleLine",
        CommentKind::MultiLine => "MultiLine",
        CommentKind::Documentation => "Documentation",
        _ => "Unknown",
    }
}

fn comment_kind_sort_key(kind: CommentKind) -> u8 {
    match kind {
        CommentKind::SingleLine => 0,
        CommentKind::MultiLine => 1,
        CommentKind::Documentation => 2,
        _ => u8::MAX,
    }
}

fn placement_snapshot_name(placement: TriviaPlacement) -> &'static str {
    match placement {
        TriviaPlacement::Leading => "Leading",
        TriviaPlacement::Trailing => "Trailing",
    }
}

fn placement_sort_key(placement: TriviaPlacement) -> u8 {
    match placement {
        TriviaPlacement::Leading => 0,
        TriviaPlacement::Trailing => 1,
    }
}

fn skipped_reason_snapshot_name(reason: SkippedTokenReason) -> &'static str {
    match reason {
        SkippedTokenReason::Recovery => "Recovery",
        SkippedTokenReason::MalformedAnnotation => "MalformedAnnotation",
        SkippedTokenReason::UnexpectedToken => "UnexpectedToken",
    }
}

fn skipped_reason_sort_key(reason: SkippedTokenReason) -> u8 {
    match reason {
        SkippedTokenReason::Recovery => 0,
        SkippedTokenReason::MalformedAnnotation => 1,
        SkippedTokenReason::UnexpectedToken => 2,
    }
}

fn whitespace_hint_snapshot_name(kind: WhitespaceHintKind) -> &'static str {
    match kind {
        WhitespaceHintKind::RequiresSeparation => "RequiresSeparation",
        WhitespaceHintKind::LineBreakBefore => "LineBreakBefore",
        WhitespaceHintKind::LineBreakAfter => "LineBreakAfter",
        WhitespaceHintKind::SyntheticBoundary => "SyntheticBoundary",
    }
}

fn whitespace_hint_sort_key_value(kind: WhitespaceHintKind) -> u8 {
    match kind {
        WhitespaceHintKind::RequiresSeparation => 0,
        WhitespaceHintKind::LineBreakBefore => 1,
        WhitespaceHintKind::LineBreakAfter => 2,
        WhitespaceHintKind::SyntheticBoundary => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SkippedTokenReason, SurfaceTriviaBuilder, TriviaAttachmentTarget, TriviaPlacement,
        WhitespaceHintKind,
    };
    use mizar_session::{
        BuildSnapshotId, CommentKind, GeneratedSpanAnchor, GeneratedSpanOrigin, Hash,
        InMemorySessionIdAllocator, SessionIdAllocator, SourceAnchor, SourceId, SourceRange,
    };

    #[test]
    fn trivia_builder_preserves_ownership_and_attachment_hints() {
        let source_id = source_id(1);
        let mut builder = SurfaceTriviaBuilder::new(source_id);

        builder.add_comment(CommentKind::SingleLine, range(source_id, 30, 39));
        builder.add_doc_comment_attachment(
            range(source_id, 0, 9),
            TriviaAttachmentTarget::Detached(SourceAnchor::Point {
                source_id,
                offset: 10,
            }),
            TriviaPlacement::Leading,
        );
        builder.add_whitespace_hint(WhitespaceHintKind::LineBreakAfter, range(source_id, 9, 10));

        let trivia = builder.finish();

        assert_eq!(trivia.source_id(), source_id);
        assert_eq!(trivia.comments().len(), 1);
        assert_eq!(trivia.comments()[0].kind, CommentKind::SingleLine);
        assert_eq!(trivia.comments()[0].range, range(source_id, 30, 39));
        assert_eq!(trivia.doc_comment_attachments().len(), 1);
        assert_eq!(
            trivia.doc_comment_attachments()[0].range,
            range(source_id, 0, 9)
        );
        assert_eq!(
            trivia.doc_comment_attachments()[0].target,
            TriviaAttachmentTarget::Detached(SourceAnchor::Point {
                source_id,
                offset: 10,
            })
        );
        assert_eq!(
            trivia.doc_comment_attachments()[0].placement,
            TriviaPlacement::Leading
        );
        assert_eq!(trivia.whitespace_hints().len(), 1);
        assert!(!trivia.is_empty());
    }

    #[test]
    fn skipped_ranges_are_preserved_with_source_ranges() {
        let source_id = source_id(2);
        let mut builder = SurfaceTriviaBuilder::new(source_id);

        builder.add_skipped_token_range(
            range(source_id, 20, 24),
            Some(TriviaAttachmentTarget::Detached(SourceAnchor::Range(
                range(source_id, 18, 25),
            ))),
            SkippedTokenReason::Recovery,
        );
        builder.add_skipped_token_range(
            range(source_id, 30, 34),
            None,
            SkippedTokenReason::MalformedAnnotation,
        );

        let trivia = builder.finish();

        assert_eq!(trivia.skipped_token_ranges().len(), 2);
        assert_eq!(
            trivia.skipped_token_ranges()[0].range,
            range(source_id, 20, 24)
        );
        assert_eq!(
            trivia.skipped_token_ranges()[0].owner,
            Some(TriviaAttachmentTarget::Detached(SourceAnchor::Range(
                range(source_id, 18, 25,)
            )))
        );
        assert_eq!(
            trivia.skipped_token_ranges()[0].reason,
            SkippedTokenReason::Recovery
        );
        assert_eq!(
            trivia.skipped_token_ranges()[1].range,
            range(source_id, 30, 34)
        );
        assert_eq!(trivia.skipped_token_ranges()[1].owner, None);
        assert_eq!(
            trivia.skipped_token_ranges()[1].reason,
            SkippedTokenReason::MalformedAnnotation
        );
    }

    #[test]
    #[should_panic(expected = "surface trivia range must belong to the trivia source")]
    fn generated_detached_anchor_must_match_trivia_source() {
        let ids = InMemorySessionIdAllocator::new();
        let source_id = ids.next_source_id(snapshot_id(3)).unwrap();
        let other_source_id = ids.next_source_id(snapshot_id(4)).unwrap();
        let mut builder = SurfaceTriviaBuilder::new(source_id);
        let origin = GeneratedSpanOrigin::new(
            GeneratedSpanAnchor::Range(range(other_source_id, 0, 1)),
            "generated fixture",
        )
        .unwrap();

        builder.add_doc_comment_attachment(
            range(source_id, 0, 9),
            TriviaAttachmentTarget::Detached(SourceAnchor::Generated(origin)),
            TriviaPlacement::Leading,
        );
    }

    fn source_id(byte: u8) -> SourceId {
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id(byte))
            .unwrap()
    }

    const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn snapshot_id(byte: u8) -> BuildSnapshotId {
        let hex = format!("{byte:02x}").repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }
}
