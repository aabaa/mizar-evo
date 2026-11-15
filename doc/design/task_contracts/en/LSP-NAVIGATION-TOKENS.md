# Task LSP-NAVIGATION-TOKENS: syntax-only semantic tokens
Canonical language: English; [Japanese pointer](../ja/LSP-NAVIGATION-TOKENS.md).
Status: specified. Tier: full; first navigation implementation/public protocol surface.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), task15; consumer: editor clients.
Purpose: one syntax-only semantic-token increment; task15 remains open.

## Authority and readiness

Follow the frozen [navigation design](LSP-NAVIGATION-DESIGN.md), [feature sources](../../mizar-lsp/en/navigation.md#feature-sources), [freshness](../../mizar-lsp/en/navigation.md#snapshot-association-and-freshness), [projection boundary](../../mizar-lsp/en/navigation.md#projection-boundary), and [range conversion](../../mizar-lsp/en/range.md).
Existing lexer preprocessing, recoverable raw scanning and reserved-shell classification supply syntax input; Server owns exact current open-buffer text. Missing navigation serving is implementation_gap.
Published expression symbol strings do not establish definition targets or token roles; no reference index or current parsed outline is available in LSP. Those features and resolved token enrichment remain deferred.

## Scope and ownership

Serve `textDocument/semanticTokens/full` synchronously from one current document entry. Advertise full requests only, legend `keyword`, `operator`, `number`, `comment`, and no modifiers, range or delta support.
Use existing lexer preprocessing/maps, recoverable scanning and reserved-shell classification. Emit reserved words as keyword, reserved symbols as operator, numerals as number, and comment trivia as comment. Omit unclassified/error tokens and identifiers with unknown semantic roles; do not infer classes from spellings.
Map preprocessing ranges back to the same raw text; split multiline spans into nonempty single-line UTF-16 tokens excluding CR/LF. Sort by position and encode standard five-integer deltas. Never fabricate semantic metadata, pending work, a successful build or a full snapshot.
Validate normal JSON-RPC lifecycle and nonempty URI parameters. Closed/unknown documents yield null; open empty/unclassified text yields an empty data array. Accepted edits/reopen use their exact current text; ignored/invalid traffic preserves it. Terminal lifecycle permits no feature delivery.
No retained semantic artifact is used or labeled current; stale artifact contents/classes cannot influence current syntax tokens. Existing hover and publication behavior stays unchanged.
Writes: LSP source/tests, this contract pair, paired owner navigation/TODO/plan pointers, mechanically required documentation registration. No producer, reader, snapshot, schema, metadata field, index representation, raw IR read, protected spec/test/trace, task17+, or protocol/AGENTS changes. §23.6/SCA-003 completion credit stays unchanged.

## Test matrix and exit

Cap: six grouped regression cases, plus existing suites: (1) independent expected legend/delta oracle for shell classes and omitted identifiers; (2) Unicode comments, multiline comments, BOM and mixed newlines mapped to raw UTF-16 positions; (3) malformed/incomplete input preserves valid recognized tokens without fabricated classes; (4) accepted/ignored edits, close/reopen and empty/missing sources; (5) lifecycle/parameter/notification errors and no pending/full snapshot; (6) genuine retained publication becoming stale and replacement cannot enrich syntax tokens, with framed executable request coverage.
Independent specification/test-intent review before freeze; sufficiency, precision, volume/scope, consistency and full-tier assessment after implementation. Reuse checked-tree command logs.
Freeze runs both lint_policy suites (including link/fragment/pair checks). Implementation runs fmt, workspace all-target/all-feature Clippy with warnings denied, both lint_policy suites, all mizar-lsp tests and short framed stdio E2E. Lexer producer behavior is unchanged; relevant existing lexer tests run. Full workspace test follows cadence only.
Exit: freeze then implementation commits, one changed task; task15 remains unchecked and remaining increments are disclosed. Local only, no push.
