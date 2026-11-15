# Code completion
Canonical language: English; [Japanese companion](../ja/completion.md).

## Responsibility and inputs

Serve `textDocument/completion` under Task25 in the [crate plan](00.crate_plan.md#task-index)
from the current open buffer and one retained [publication](metadata.md#published-inputs).
The ordinary route provides syntax candidates; the metadata-host route adds
available owner-produced symbols, notation, labels and namespace access paths.
No build, resolution, AST/IR traversal or proof search occurs on a request.

## Context and candidates

Validate raw UTF-16 positions, treating CRLF as one newline and rejecting split
surrogates. Lex the current prefix with the existing lexer preprocessing; do
not complete comments or string contents. Scan significant tokens since the
last semicolon: any `by` selects citations, including comma-separated entries.
Otherwise the immediately preceding token selects the following bounded cues:
`be`, `being`, `->`, `qua` select types (keywords `object`, `set`);
`holds`, `that`, `not`, `and`, `or`, `implies`, `iff`, `:` select formulas
(keywords `ex`, `for`, `not`); `=`, `:=`, `equals` select terms (`the`).
Other incomplete expression positions
may offer the union of term/formula kinds; uncertain declaration positions
receive only syntactically justified keywords. This is candidate filtering,
never a claim of type applicability or successful name resolution.

Filter by the written prefix. Dotted access offers the next published namespace
segment or terminal name, inserting only the suffix after the last dot.
Type candidates use mode/structure/attribute kinds, terms functor/constructor/
selector kinds, formulas predicate/attribute kinds; aliases retain their
published notation kind and relation rather than inheriting guessed semantics.
Citation candidates are published theorem/definition labels, not ATP premises;
unpublished private and proof-local labels remain unavailable.
Local declarations become available at their authenticated completion boundary;
imported public surfaces and alias paths come from the resolver view. Missing
products stay unavailable. Standalone operator events update spelling metadata
after activation and do not introduce symbols. Preserve distinct overloads and
spelling pieces, exact arity and supplied signatures/statuses.

## Freshness and deterministic delivery

Use [snapshot freshness](snapshot.md#freshness-and-replacement) unchanged. Current
syntax always uses the current buffer. Semantic activation uses the same cursor
in the retained original source; if that coordinate is invalid, omit semantic
candidates. Stale candidates explicitly name `stale_artifact`, original URI and
version; their coordinates refer to that source. They supply no `textEdit`,
additional edit or current-buffer range. Invalid/unavailable metadata cannot
erase independent syntax candidates or establish pending work.

Rank exact prefix matches first, then local before imported before keywords,
then spelling and canonical identity. Use deterministic ties for overloads;
deduplicate only identical candidates. Emit matching `sortText` and ordinary
plain-text insertion. Repeated requests and input permutations produce the
same ordered items. Closed documents and terminal lifecycle states yield no
candidates through the existing protocol lifecycle rules.

## Test design

The contract owns the grouped matrix. Exercise genuine frontend/resolver views
for notation and imports and a genuine selected snapshot for host freshness;
native publication eligibility is unchanged. Framed stdio covers capability and
syntax fallback. Completion adds no §23.6 completion or SCA-003 audit credit.
