# Task EDITOR-LSP-COMPLETION: code completion
Canonical language: English; [Japanese pointer](../ja/EDITOR-LSP-COMPLETION.md).
Status: frozen. Tier: full. Owner: [lsp plan](../../mizar-lsp/en/00.crate_plan.md); consumers: editor clients.
Dependencies: mizar-lsp tasks 6-7 (open-buffer snapshots) and 12-15 (metadata and navigation).
Readiness: existing server/snapshot and approved [producer publication](../../mizar-lsp/en/producer_metadata_publication.md) supply inputs; unsupported products stay unavailable.
Authority: specification [§1](../../../spec/en/01.introduction.md), [§12.3–12.5](../../../spec/en/12.modules_and_namespaces.md#123-import-statements), [§16.5 citations](../../../spec/en/16.theorems_and_proofs.md), [§23.5–23.6](../../../spec/en/23.package_management_and_build_system.md#236-lsp-integration-and-ide-support); [architecture 12](../../architecture/en/12.diagnostics_and_lsp.md#lsp-reads-stable-snapshots); [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-bridge).
Gap: `design_drift`. The specification promises code completion, but no mizar-lsp task owns it.

## Scope

Serve completion items from the current snapshot and published metadata:
visible symbols and their notation, theorem and definition labels for `by`
citations, namespace path segments, and keywords valid at the position. Rank
deterministically, mark items from stale metadata as stale, and document the
module in a `completion.md` spec written in the same task.

Advertise and validate `textDocument/completion` on ordinary and metadata-host
routes. Use current raw UTF-16 buffer coordinates for syntax/prefix selection;
consume one retained publication for semantic candidates under the
[metadata freshness](../../mizar-lsp/en/metadata.md#snapshot-association-and-freshness)
rules. Stale candidates identify original source/version and use its coordinates
for activation; never supply edits or ranges purporting to target current text.
Local notation uses authenticated declaration boundaries and spelling-level
operator events; imported symbols/labels use resolver-owned access paths and
public closure. Labels absent from these projections, including unpublished
private/proof-local labels, remain unavailable. Preserve overloads and alias relations, without selecting a
semantic winner. Missing/incomplete products cannot be rebuilt in LSP.

The [completion owner](../../mizar-lsp/en/completion.md) defines bounded lexical
contexts, ranking and refusal. Affected files: LSP completion module/tests,
server/metadata routes, stdio tests and paired completion owner documents;
existing capability assertions may change. Producers, native acceptance,
Task39–41, schemas and SCA-003 credit remain unchanged.

## Forbidden

Semantic analysis inside the LSP server beyond published metadata and snapshot
data; proposing proof steps or premises (that belongs to the explanation and
agent interfaces); specification changes.

## Tests and exit

Case cap: six grouped cases, each requiring exact positive and refusal assertions.
1. With published mode M/functor F/predicate P: `be |` offers M/object/set, `= |` F/the, `holds |` P/ex/for/not; other named kinds excluded. `be M|` filters to M; comment/string/split-surrogate/out-of-line cursors yield none; raw BOM/CRLF positions work.
2. Genuine private/public local declarations and operator events: before/at activation excludes/includes spelling; multi-piece and same-spelling overloads retain identities/arity, latest active event wins metadata only. Unavailable notation adds none.
3. Genuine `import ... as Grp;` and facade: `Grp.|` offers resolver-owned next segment/terminal, unqualified prefix offers Grp; canonical origins unchanged. Duplicate paths deduplicate; conflicts/private/missing closure add none.
4. Published theorem T/definition D plus unpublished proof-local L: `by |` and `by T, |` offer T/D, exclude L/F/P and proof keywords; `by T; be |` resets to type candidates. Imported/alias label paths preserve signature/status.
5. Genuine selected publication: current/version-only edit/reopen/replacement preserve or replace candidate identity; edited recovered `be |` retains object/set. Stale items name original URI/version, omit edits/ranges, and invalid original coordinates add no semantic candidates.
6. Repeat/permutate rows: identical ordered JSON. JSON-RPC/stdio advertise completion; negative/noninteger coordinates or malformed context return InvalidParams; valid integers outside text yield none. Absent host yields syntax only, closed/terminal documents no items.
Use existing hover/snapshot/semantic-token fixtures as regression anchors.
Require independent specification, sufficiency, precision, volume and consistency reviews.
Run fmt, Clippy, both lint_policy suites, `cargo test -p mizar-lsp` and short framed stdio E2E; full `cargo test` only on a protocol cadence trigger.
Exit: completion requests return deterministic items drawn only from
published metadata and the current snapshot.
