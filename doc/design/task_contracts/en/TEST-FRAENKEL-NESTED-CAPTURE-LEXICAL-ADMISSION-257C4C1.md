# Task TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1: Explicit-import lexical admission

> Canonical language: English. Japanese companion: [../ja/TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md](../ja/TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

Stable owner sections: checker [source/spec classification](../../mizar-checker/en/source_spec_audit.md#task-257c4c1-explicit-import-lexical-admission),
[TODO](../../mizar-checker/en/todo.md#task-257c4c1-explicit-import-lexical-admission),
and [bilingual record](../../mizar-checker/en/bilingual_sync_audit.md#task-257c4c1-frozen-contract-parity);
mizar-test [harness](../../mizar-test/en/harness.md#task-257c4c1-private-import-provider),
[boundary](../../mizar-test/en/module_boundary_audit.md#task-257c4c1-crate-local-testdata-boundary),
[corpus](../../mizar-test/en/miz_corpus.md#task-257c4c1-explicit-import-corpus-repair),
[traceability](../../mizar-test/en/traceability.md#task-257c4c1-inactive-trace-repair),
[TODO](../../mizar-test/en/todo.md#task-257c4c1-lexical-admission-prerequisite), and
[bilingual record](../../mizar-test/en/bilingual_sync_audit.md#task-257c4c1-frozen-contract-parity).

## Status, authority, and readiness

**Status:** dependency-ready documentation prerequisite. The user explicitly
authorizes only the dedicated explicit-import test-intent repair: define the two
required imported spellings in one separate fixture module, import that module
from the existing inactive C4C0 oracle, prove frontend lexical/parser admission
and preprocessed import provenance, and stop. This authority closes the former human-owned
choice without authorizing capture or semantic work.

Authority is, in order:

1. canonical [Chapter 2 §2.10](../../../spec/en/02.lexical_structure.md),
   [Chapter 3 §3.3](../../../spec/en/03.type_system.md),
   [Chapter 7 §7.2](../../../spec/en/07.modes.md),
   [Chapter 10 §10.1](../../../spec/en/10.functors.md),
   [Chapter 11 §§11.2.1 and 11.2.4](../../../spec/en/11.symbol_management.md),
   [Chapter 12 §§12.3 and 12.5](../../../spec/en/12.modules_and_namespaces.md),
   and [Chapter 13 §§13.4.2, 13.4.4, and 13.8.6](../../../spec/en/13.term_expression.md);
2. the approved exact imported `.miz` test intent below, derived from the
   completed C4C0 oracle;
3. the existing C4C0 trace row;
4. the existing inactive C4C0 expectation sidecar;
5. completed R2/C4A/C4B/C4C0 contracts, design records, and current private
   provider/source observations.

Chapter 12 makes an import prelude file-scoped and makes public definitions
part of a module signature. Chapter 13 requires the nested occurrence to refer
to the outer generator by resolved identity. The approved repair supplies only
the two imported lexical spellings needed to admit that already-frozen source.
The current six parser diagnostics are `source_drift`; the absent dedicated
support module/test is a `test_gap`; and the absent provider and owner record is
`design_drift`. There is no remaining blocking `spec_gap`, no expectation
rebaseline, and no public-API `boundary_violation` in this slice.

Dependencies are complete: C4C0 owns the inactive 124-byte historical oracle,
the canonical import/public-definition rules are explicit, and the current
frontend/provider seams already carry the required one-stub/one-summary
transaction. C4C1 is therefore ready after this exact synchronized
documentation prerequisite passes review and is committed separately.

## Exact support module and exported summary

The implementation adds exactly one crate-local testdata file outside ordinary
corpus discovery:

```text
crates/mizar-test/tests/testdata/parser/nested_capture_fixtures.miz
```

Its logical module identity is exactly `parser.nested_capture_fixtures`. Its
final-LF source is exactly:

```mizar
definition
  let S be set;
  public mode ElementDef: Element of S is set;
end;

definition
  public func NatDef: NAT -> set equals {};
end;
```

The file is `140` bytes with SHA-256
`dd721a48620f985d5612cc718a94aef576e87d616c239712b8deb2d65c84a11c`.
Exact declaration anchors are `S@17..18`, definition label
`ElementDef@41..51`, exported mode `Element@53..60`, locus `S@64..65`,
definition label `NatDef@105..111`, exported functor `NAT@113..116`, and body
`{}@131..133`. The separate definition blocks prevent the mode locus `S` from
contaminating the zero-arity `NAT` functor. The file is a test-owned support
module, not an ordinary corpus case; it has no expectation sidecar, trace row,
active route, or independent coverage credit.

The C4C1 harness associates that exact crate-local testdata source with the
synthetic `ModuleId("parser.nested_capture_fixtures")`, and the crate-private
provider recognizes only that exact module id. The physical source and summary
are cross-validated by the frozen tests. This is not package, MML, implicit-
prelude, production source-manifest, or general file-resolution behavior.

The sole import at stub ordinal `0` resolves one summary with fingerprint
`LexicalSummaryFingerprint(1)` and exactly these two ordered full-field shapes:

| rank | spelling | symbol id | source module | kind | arity | operator |
|---:|---|---|---|---|---|---|
| `0` | `Element` | `parser.nested_capture_fixtures#parse-only#Element` | `parser.nested_capture_fixtures` | `Mode` | exact `1` | `None` |
| `1` | `NAT` | `parser.nested_capture_fixtures#parse-only#NAT` | `parser.nested_capture_fixtures` | `Functor` | exact `0` | `None` |

`parser.type_fixtures` and every unrelated module retain their exact existing
summary bytes, ranks, identities, and fingerprints. The implementation must
not append these two shapes to `parser.type_fixtures`, synthesize multiple
modules, or broaden the provider for approximate spellings.

## Exact repaired C4C0 source and frontend provenance

The existing corpus source becomes exactly:

```mizar
import parser.nested_capture_fixtures;

definition
  func NestedCapture -> set equals
    { { x where y is Element of NAT }
      where x is Element of NAT };
end;
```

The final-LF file is `164` bytes with SHA-256
`c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431`.
It has exactly one import stub and one summary. Significant ranges are module
path `7..37`, `NestedCapture@58..71`, inner mapper `x@94..95`, inner generator
`y@102..103`, first `Element@107..114`, first `NAT@118..121`, outer generator
`x@136..137`, second `Element@141..148`, and second `NAT@152..155`.

Preprocessing must expose exactly one `ImportStub`: stub span `7..37`,
path spelling `parser.nested_capture_fixtures`, path span `7..37`, components
`parser` and `nested_capture_fixtures`, one raw source segment `7..37`, no
relative prefix, and no alias. The parsed AST retains component ranges
`parser@7..13` and `nested_capture_fixtures@14..37`, while the surrounding
import declaration is `0..38`. The provider resolves that stub at ordinal `0`
to the exact one-summary/
two-shape lexical environment above, after which the real frontend AST has zero
diagnostics and no recovery. This task does not run or extend resolver import
augmentation. Resolver imported identities, use-site name-reference
resolution, generator identity, capture readiness, type application, sethood,
and formula semantics remain separately deferred.

The historical no-import source remains frozen in the fourth regression as
the exact final-LF `124` bytes with SHA-256
`f1a35d2d7f6cb4a57ece3b1143a68c1a01ab9ac478960862057025cc9838cea7`.
It must continue to produce exactly six parser diagnostics, first at
`Element@67..74`, and must receive no imported-summary leakage. The repaired
corpus file supersedes those historical bytes only under the explicit test-
intent authority above; the historical regression prevents an implicit
prelude or builtin substitution from being inferred.

## Existing API ownership and exact implementation scope

No public API is added or changed. `mizar-frontend::lexical_env` already owns
`LexicalEnvironmentRequest`, `LexicalSummaryProvider::resolve_imports`,
`ResolvedImports`, provenance validation, and active-environment assembly.
`mizar-lexer` already owns `ModuleLexicalSummary`, `ExportedSymbolShape`,
`ExportRank`, `UserSymbolKind`, `UserSymbolArity`, and
`LexicalSummaryFingerprint`. C4C1 only extends the crate-private
`ParseOnlyImportProvider` summary branch. The existing
`augment_type_elaboration_import_summaries` allowlist and all resolver
augmentation behavior remain byte-identical.

After the documentation prerequisite commit, implementation changes exactly
these seven paths:

```text
crates/mizar-test/tests/testdata/parser/nested_capture_fixtures.miz
tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.miz
tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.expect.toml
tests/coverage/spec_trace.toml
doc/design/spec_coverage_audit.md
crates/mizar-test/src/runner/import_fixtures.rs
crates/mizar-test/src/runner/tests/parse_only.rs
```

There is no new test leaf and no `tests.rs` include. The exact four tests added
to the existing `parse_only.rs` are:

1. `task257c4c1_physical_fixture_declarations_are_exact`;
2. `task257c4c1_provider_summary_is_exact_and_unrelated_modules_are_isolated`;
3. `task257c4c1_canonical_imported_source_is_zero_diagnostic_with_exact_frontend_provenance`;
4. `task257c4c1_historical_no_import_source_retains_six_diagnostics_without_leakage`.

They respectively pin the physical file's bytes/hash/declaration ranges and
zero-diagnostic declaration surface; every summary field, one-stub/one-summary
fingerprint, and unrelated-module isolation; the repaired source bytes/hash,
zero frontend/parser diagnostics, exact preprocessed import stub/provider
summary/AST and no recovery; and the exact historical source/hash/six-
diagnostic/no-leakage baseline. Raw mizar-test library tests project
`614 -> 618`. No public runner command or case dispatch is introduced.

## Sidecar, trace, audit, and count impact

The existing sidecar keeps schema `1`, kind `pass`, stage
`advanced_semantics`, domain `set_expressions.nested_capture`, outcome
`pass/type_check`, empty diagnostic codes, the sole existing spec ref, no
active tags, and no failure fields. Only its note changes: explicit imported
lexical/parser admission is complete, while capture transport, execution, and
Task 277B remain deferred.

The existing trace requirement remains the sole
`spec.en.13.set_expressions.nested_capture.semantic` row and keeps
`advanced_semantics/covered/required/pass` plus the sole sidecar. Its sorted
dependency remains exactly `spec.en.13.set_expressions.parser`; no new
requirement or test backlink is added. The row note cites the Chapter-12 import
authority and records the lexical-admission boundary. The coverage audit changes
only the Chapter-13 row to record that the exact lexical/import blocker is
closed while resolver/checker capture and the
advanced-semantics runner remain follow-ups. No status or semantic credit
changes.

At clean baseline HEAD `d93fa7133e77e070d4ba0d016a1c3519ae80dd4b`,
`origin/main...HEAD` is `0/27`, the worktree is clean, and protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` is unchanged. Baselines are:

- corpus pairs `344/344`, unchanged because the support file is outside
  `tests/miz` and the existing C4C0 pair is modified in place;
- contract trees `90/90 -> 91/91` after this prerequisite;
- trace `5924` lines / SHA-256
  `d1df314665998fe5271a73d7102b6e6d6098fd6636d78e2a6ded779d5f44cbae`;
- coverage audit `7005` lines / SHA-256
  `99720173f84f1713ed2bf63e9806566b2aa6a904d18d6855b20544bab96928a5`;
- sidecar SHA-256
  `2c7d987baa988b9ea1ae179d6ed1a3b9c8df334694cdd9d43626342647d59701`;
- `import_fixtures.rs` `469` lines / SHA-256
  `991c400cedb084fe9cd4b59a17be068d843ce806f1fae74c49c22208970e400f`;
- `parse_only.rs` `111` lines / SHA-256
  `3cddce85155b72597cfc4c2ea5841dbf3fe5f88d0c8123d98ba9cb958f90a3a8`;
- mizar-test library `614`, metadata tests `137/137`, cases/requirements
  `429/396`, pass/fail `236/193`, active routes `101/7/205/1`, and aggregate
  warnings/errors `23/0`.

Implementation must remeasure the two Rust files, support/source/sidecar/
trace/audit hashes, raw test list, metadata, and all five CLI stdout hashes.
Only the test count and inactive artifact/document bytes are expected to
change; corpus pair, metadata, active route, warning/error, and executable
stage counts must remain unchanged. No count or unchanged-hash claim is
accepted without replay.

## Scope, prohibitions, reviews, and exit

This prerequisite changes exactly 24 Markdown paths: this synchronized
contract pair; both checker and mizar-test EN/JA crate plans; checker EN/JA
`source_spec_audit.md`, `todo.md`, and `bilingual_sync_audit.md`; and mizar-test
EN/JA `harness.md`, `module_boundary_audit.md`, `miz_corpus.md`,
`traceability.md`, `todo.md`, and `bilingual_sync_audit.md`. No Rust, artifact,
canonical specification, legacy manifest, protected audit/ledger, staging, or
commit belongs to this docs-only prerequisite.

Forbidden and deferred:

- no `doc/spec` change and no inference of language semantics from source;
- no frontend/parser/resolver/checker public API or production semantic route,
  and no Chapter-2/3/7/10/11/12 coverage credit;
- no new corpus case, sidecar, trace requirement, active tag, failure field,
  diagnostic expectation, warning/error credit, or route credit;
- no `SourcePrimaryTerm`, term/use row, Task 252, `CapturedFreeVariables`,
  role-enum duplication, binding/capture table, formula, type/sethood request,
  verdict, diagnostic, Typed/Resolved installation, dispatch, or execution;
- no edits or reinterpretation of F5, R2, C4A, C4B, protected legacy evidence,
  or schema-v2 compaction data;
- Task 277B remains not ready and receives zero semantic credit.

The documentation prerequisite exits only after exact24 scope review, EN/JA
parity and recursive-link review, both checker and mizar-test lint-policy
suites, `git diff --check`, protected-anchor/schema-v2 stability, and an
independent final-quality review with all nine hard gates passing and a valid
score of at least `90/100`. It then receives a task-only docs commit and clean
fresh preflight before the exact seven-path implementation.

The implementation exits only after all four exact tests pass, mizar-test
library/full workspace tests, format, package/full-workspace Clippy, metadata,
five CLI replay, both lint suites, diff/scope checks, independent test,
implementation, source-documentation, bilingual/boundary, and final-quality
reviews finish with no findings, and the inactive zero-credit status is
reproved. The immediate successor is fresh inventory only; capture semantics
must be separately selected and frozen.

Recommended routing: GPT-5.6 Sol at `xhigh` owns authority, contract
acceptance, scope expansion decisions, and final hard-gate scoring. After this
contract is frozen, GPT-5.6 Terra at `xhigh` may implement the bounded
crate-private provider/test/artifact slice and perform independent reviews. Any
different module identity, symbol shape/arity/rank, frontend provenance, diagnostic
profile, or requested semantic expansion returns to Sol before editing.
