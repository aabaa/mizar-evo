# Task STEP5A1-G5-QUA-STRUCTURE: root-reachable AST tree validation

> Canonical language: English. Japanese pointer:
> [../ja/STEP5A1-G5-QUA-STRUCTURE.md](../ja/STEP5A1-G5-QUA-STRUCTURE.md).

Owning plan: [mizar-syntax](../../mizar-syntax/en/00.crate_plan.md).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Ready; freeze precedes test/source edits |
| Tier | Full: production syntax trust path plus test-first `.miz` |
| Owner / consumers | `mizar-syntax::SurfaceAstBuilder::finish` / parser, frontend, parse-only runner |
| Dependencies | None; Step 5A frozen order selects 5A.1 first |
| Authority | [§13.6](../../../spec/en/13.term_expression.md#136-type-qualification-qua), Appendix [A.3](../../../spec/en/appendix_a.grammar_summary.md#a3-type-expressions)/[A.13](../../../spec/en/appendix_a.grammar_summary.md#a13-term-expressions), [Step 5A](../../todo.md#step-5a--frontend-gap-closure--) |
| Classification | `source_drift`, `test_gap`, bounded `design_drift`; no `spec_gap` |

The recorded [G5 reproducer](../../mizar-test/en/semantic_bridge_frontend_gaps.md#minimal-reproducers)
is valid syntax. The builder currently counts a root-unreachable speculative
node that reuses a selected-tree token and panics before rowan projection;
builtin targets avoid that path.

## Invariant and tests

With a root, `finish` validates unique non-root parentage over the union of the
root-reachable and optional expression-root-reachable graphs. An artifact
outside both graphs cannot invalidate that selection. With no root, including
`finish(None, Some(expression_root))`, whole-arena validation remains. Every raw
child edge counts: shared children and duplicate child entries remain rejected.
Structural-root-child collision, builder-local ids, ranges, token order,
recovery/trivia, dense views, rowan text, snapshots, and public API are stable.
Grammar, lexing, diagnostics, name resolution, types, and acceptance do not move.

Test-first adds exactly
`tests/miz/pass/parser/pass_parser_qua_structure_type_001.{miz,expect.toml}`.
The `.miz` is the byte-identical 178-byte/11-line invalid-narrowing source
shape (SHA-256 `68072cd3284de290660cc82dc90d6bc3306531c8bf4ab8c7dcc0b9c1be96ad52`).
The sidecar fields are `id=pass_parser_qua_structure_type_001`, `kind=pass`,
`stage=parse_only`, `domain=parser.qua_terms`, matching source basename,
`expected_outcome=pass`, `expected_phase=parse`, `diagnostic_codes=[]`,
`tags=["active_parse_only"]`, and only
`spec.en.13.qua_qualification.parser`. The sole permitted
[`spec_trace.toml`](../../../../tests/coverage/spec_trace.toml) edit appends that
sidecar to this requirement's `tests`; stage/status/coverage and all other rows
stay unchanged.

Exact new syntax tests are
`builder_allows_sharing_from_outside_selected_graph`,
`builder_rejects_rootless_shared_children`,
`builder_rejects_nested_structural_root_child`, and
`builder_rejects_duplicate_selected_parent_child_edges`. They prove unchanged
selected-parent rejection and no selected rowan-text duplication; the rootless
test catches both `finish(None, None)` and `finish(None, Some(expression_root))`.
Durable
invariant/test design belongs to [ast.md](../../mizar-syntax/en/ast.md#builder-boundary);
the [parse-only harness contract](../../mizar-test/en/expectation_schema.md#parse-only-expectations)
and its private [`parse_only.rs`](../../../../crates/mizar-test/src/runner/parse_only.rs)
own runner admission/execution.

## Scope and protected surfaces

The prerequisite commit contains only this pair and paired syntax plan/TODO.
The implementation commit is limited to:

```text
crates/mizar-syntax/src/ast.rs
crates/mizar-syntax/src/ast/tests.rs
tests/miz/pass/parser/pass_parser_qua_structure_type_001.miz
tests/miz/pass/parser/pass_parser_qua_structure_type_001.expect.toml
tests/coverage/spec_trace.toml
doc/design/mizar-syntax/en/ast.md
doc/design/mizar-syntax/ja/ast.md
doc/design/task_contracts/en/STEP5A1-G5-QUA-STRUCTURE.md
doc/design/mizar-syntax/en/todo.md
doc/design/mizar-syntax/ja/todo.md
doc/design/todo.md
doc/design/mizar-test/en/semantic_bridge_frontend_gaps.md
```

Do not edit `doc/spec`, existing `.miz`/expectations, oracle expectations,
[activation rows](../../../../tests/coverage/step5_activation_map.tsv), existing
trace status/coverage, diagnostics, parser/frontend/lexer production, public
API, Cargo metadata, or `doc/design/archive/`. Do not activate the two audit-1
semantic cases; owners remain 5C.7/5C.6. Coverage-audit impact is explicitly
none because the parse requirement is already covered.

## Gates, baseline, and exit

Independent pre-implementation spec/equivalence and boundary reviews precede
test/source edits; test-sufficiency, implementation, source/docs/API, nine hard
gates, and read-only score ≥90 follow. Focused commands are syntax/parser/
frontend tests and the parse-only CLI. Final commands add fmt, warnings-denied
workspace Clippy, full tests, metadata plan/lints, link/fragment checks,
protected hashes, exact staging, local commit, and clean postcommit proof.

Baseline cases/requirements/active-parse-only is `550/499/101`, syntax tests
`70`, contract pairs `125/125`; expected delta is `+1/+0/+1`, `+4`, and one
pair. `ast.rs` is 3,037 lines / `95c12195359d8d2a0cf19740c82ffdf48a54219d0a115b68c31ccdffd4b1b8fd`;
tests 11,397 / `fc3cbbe9895c4bd336cb2da577f028f6960a0d7ad1d3da056dfe34b7f03565a7`;
trace `940a23dcc6cdda46c653cc2bc7ff19b059ace8af81e036df8f8917dca071511e`.
Protected activation/gap-ledger/coverage-audit/archive-manifest hashes are
`e9a1c2e6b0b444a2caed6a081b6c4a2ff780ee39d4e357c0268d3f8d7215a34b`,
`a0d161dedd78450110fe25e11634f9cdde5f5f633d30965b887cd70aee383dd8`,
`9e75f8ea0f7a1ca81a88f47811f21a264803b16fb3101327caaed7c0925af285`,
and `934df58f26b3ea1903f9c476452055d7755001fa9e5786293c959e863da72160`.

Exit: the active byte-equivalent G5 fixture parses clean without panic. The
current synonym source has no `qua` and remains G4-blocked until 5A.5; 5A.1
discharges only its mapped G5 dependency. No inactive semantic case becomes an
active oracle. Completion evidence and final hashes replace this paragraph;
fresh inventory then selects 5A.2/G1.
