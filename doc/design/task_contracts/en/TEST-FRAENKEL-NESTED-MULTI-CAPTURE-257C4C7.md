# Task TEST-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C7: Two-capture inactive test intent

> Canonical language: English. Japanese companion:
> [../ja/TEST-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C7.md](../ja/TEST-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C7.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

Stable owner sections: checker
[source/spec classification](../../mizar-checker/en/source_spec_audit.md#task-257c4c7-two-capture-test-intent),
[future projection boundary](../../mizar-checker/en/source_formula_composition.md#task-257c4c7-multi-capture-projection-boundary),
[TODO](../../mizar-checker/en/todo.md#task-257c4c7-two-capture-prerequisite), and
[bilingual record](../../mizar-checker/en/bilingual_sync_audit.md#task-257c4c7-frozen-contract-parity);
mizar-test [corpus](../../mizar-test/en/miz_corpus.md#task-257c4c7-frozen-corpus-increment),
[traceability](../../mizar-test/en/traceability.md#task-257c4c7-inactive-trace-increment),
[TODO](../../mizar-test/en/todo.md#task-257c4c7-two-capture-inactive-oracle), and
[bilingual record](../../mizar-test/en/bilingual_sync_audit.md#task-257c4c7-frozen-contract-parity).

## Status, authority, and readiness

**Status:** implementation, reviews, verification, and final-quality scoring
complete; exact staging, commit, and postcommit proof remain. The
user accepted the parent authority decision summarized below. This task adds
only the exact inactive two-capture oracle, its metadata/backlink/audit records,
and mechanical global count-guard maintenance. It adds no capture
implementation or active route.

Authority is, in order:

1. canonical Chapter 13
   [§13.4.3](../../../spec/en/13.term_expression.md#1343-multiple-generators),
   [§13.4.4](../../../spec/en/13.term_expression.md#1344-nested-comprehensions),
   and [§13.8.6](../../../spec/en/13.term_expression.md#1386-set-expression-encoding);
2. the exact test-first `.miz` source frozen below;
3. the existing
   `spec.en.13.set_expressions.nested_capture.semantic` trace requirement and
   the two synchronized inactive sidecars after implementation;
4. completed C4C1 through C4C6 contracts and derived owner documents;
5. current parser, resolver, checker, and Core source observations, which are
   non-normative.

Chapter 13 explicitly permits multiple generators, requires nested capture by
resolved binder identity rather than display spelling, and defines generated
`params` as free variables captured from the surrounding context. Therefore
the exact two-outer-binder membership/cardinality oracle is fully derivable and
closes a `test_gap`; it does not decide capture order. The missing generalized
checker projection and corruption coverage are `design_drift` and a later
private-unit `test_gap`. Treating display names, resolver/checker/Core numeric
IDs, or the one-row C4C5 `source_ordinal` as a generalized join is a
`boundary_violation`. Making an order observable language semantics would be a
`spec_gap` and is forbidden here. There is no authority contradiction or
`repo_metadata_conflict` in this slice.

The exact imported `Element`/`NAT` lexical profile, the parser's independently
covered multiple-generator and nested-comprehension shapes, completed C4C6,
and the accepted owner/boundary decision make this test-first task uniquely
ready. Current C4C2 through C4C6 exact-one-capture validators cannot consume
the new witness and must remain unchanged.

Task 277B remains not ready and receives zero credit.

## Frozen owner and downstream boundary decision

This task records the accepted boundary for later work without creating its
API:

- `mizar-test` owns this inactive corpus/expectation/trace artifact. The
  existing checker Task-257C `source_formula_composition` family is the sole
  future lower owner of a complete, immutable, syntax-free, Core-ID-free
  multi-capture projection.
- Association is by authenticated resolved binder identity. The complete
  projection must cover generator declarations, mapper, optional predicate,
  capture occurrences, owners/contexts, and source provenance; exact fields
  and constructors require a fresh post-C4C7 contract.
- A distinct capture appears once even when it has multiple occurrences.
  Future transport may use authenticated outer generator declaration/source
  order as a private deterministic convention. That convention is
  alpha-invariant and must not become a language result, diagnostic contract,
  or assertion in the `.miz` sidecar.
- The complete projection is standalone. C4C6's boxed Typed/Resolved receipt
  remains installed and immutable; C4C7 does not reopen `TypedAst` or
  `ResolvedTypedAst`, add another slot, or replace C4C6.
- Missing, extra, duplicate, reordered, stale, recovered, partial, mismatched,
  or display-name-joined projection state must fail closed. Consumers must not
  sort, repair, infer, deduplicate unchecked input, or reinterpret numeric IDs.
- Core Task 33 later owns fresh snapshot-local `CoreVarId` allocation and a
  durable typed association from the checker projection. Core Task 35 must
  consume that association and owns actual Fraenkel lowering and generated
  origin/naming after its Task-34 dependency; it must not allocate or infer the
  association.
- Future Core inputs must distinguish semantic operands such as generator
  domain `S` from captured parameters/arguments. Only the captured-parameter
  and capture-argument subvectors are a positional one-to-one join in the
  checker's private deterministic order; whole `params.len() == args.len()` is
  neither required nor authorized.

No public/private Rust type, field, adapter, installer, Core route, or
GeneratedOrigin value is named or created by C4C7. A fresh inventory after the
artifact commit must freeze those details before implementation.

## Exact test-first source and sidecar

Add exactly this final-LF source at
`tests/miz/pass/types/pass_types_nested_comprehension_two_outer_generator_captures_001.miz`:

```mizar
import parser.nested_capture_fixtures;

definition
  func NestedCaptureTwo -> set equals
    { { [x, y] where z is Element of NAT }
      where x is Element of NAT, y is Element of NAT };
end;
```

It is `193` bytes with expected SHA-256
`b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`.
The inner mapper references exactly the two resolved outer generators `x` and
`y`; inner generator `z` is local and is not captured. The oracle asserts the
two-member capture set and no order.

Add the matching sidecar
`tests/miz/pass/types/pass_types_nested_comprehension_two_outer_generator_captures_001.expect.toml`
with schema `1`, matching id/source, `kind = "pass"`,
`stage = "advanced_semantics"`,
`domain = "set_expressions.nested_capture"`, pass/type-check outcome, empty
diagnostics, and sole spec reference
`spec.en.13.set_expressions.nested_capture.semantic`. Its exact note is:

```text
Inactive advanced_semantics pass oracle derived from Chapter 13 sections 13.4.3, 13.4.4, and 13.8.6: the inner mapper references both resolved outer generator identities x and y, while inner z remains local. It asserts capture membership/cardinality only, not generated-parameter/application-argument order. Frontend admission uses parser.nested_capture_fixtures; generalized resolver/checker capture transport, execution, Core lowering, and Task 277B remain deferred.
```

The complete final-LF sidecar is `885` bytes with expected SHA-256
`277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`.
It has no active tags or failure-only fields.

## Trace, audit, and exact implementation scope

The existing nested-capture requirement remains the sole requirement and keeps
its id, source/section, `advanced_semantics`, `covered`, `required = true`,
`coverage = "pass"`, and sole parser dependency. Append only the new sidecar
path to its sorted `tests` array and update its note to describe one- and
two-capture inactive seeds without executable or ordering credit. Do not add a
requirement or change status.

Add one dedicated C4C7 zero-credit mapping/follow-up section to
`doc/design/spec_coverage_audit.md`. The Chapter-13 summary row remains
`partial` because its status and broad follow-up are unchanged; the dedicated
section owns the exact second-oracle mapping and downstream boundary delta.
Resolver/checker generalized capture, active execution, Core lowering, and
Task 277B remain deferred. Historical numeric prose elsewhere in that audit
remains historical and is not rebaselined.

The documentation freeze changes exactly 22 paths: this paired contract, both
checker and mizar-test EN/JA Task Index plans, and the paired checker
source-spec/source-formula/TODO/bilingual plus mizar-test corpus/traceability/
TODO/bilingual owner records linked above. Artifact completion may update only
their task status/evidence in addition to the eight paths below. Thus the final
task scope is exactly 30 paths.

After the contract/owner records pass review, implementation may add or change
exactly these eight artifact/verification paths:

```text
tests/miz/pass/types/pass_types_nested_comprehension_two_outer_generator_captures_001.miz
tests/miz/pass/types/pass_types_nested_comprehension_two_outer_generator_captures_001.expect.toml
tests/coverage/spec_trace.toml
doc/design/spec_coverage_audit.md
crates/mizar-test/src/runner/tests/type_elaboration/source_attribute_definition.rs
crates/mizar-test/src/runner/tests/type_elaboration/source_functor_definition.rs
crates/mizar-test/src/runner/tests/type_elaboration/source_mode_definition.rs
crates/mizar-test/src/runner/tests/type_elaboration/source_property_implementation.rs
```

The four Rust test files may change only their global metadata tuples from
`(429, 396)` to `(430, 396)` and `(236, 193)` to `(237, 193)`. No production
Rust or other assertion changes are authorized.

## Baseline, expected impact, and protected state

Clean baseline HEAD is
`60dbe59e26659ccce16c7999f81760597b3ef2fd`, origin/main is
`ffc882675141a3e25bc78a47affc018bfe3685e1`, and divergence is `0/2`.
Protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` remains untouched.

- corpus pairs: `344/344 -> 345/345`;
- metadata cases/requirements: `429/396 -> 430/396`;
- pass/fail: `236/193 -> 237/193`;
- requirements and active route/stage/warning/error counts: unchanged;
- contract trees: `101/101 -> 102/102`;
- baseline trace: `5924` lines / `464057` bytes / SHA-256
  `d4d817e83aac78d19e729702b26c62604fc57581eec18672a5c26ec44efe7a81`;
- baseline coverage audit: `7088` lines / `540634` bytes / SHA-256
  `1ec5de8dbccdf3afee01c710ac22f00af933ee57ec749e930cf89f8936b27cfd`.

Protected existing one-capture source and sidecar hashes are respectively
`c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431`,
and `9ed000a30c1d519bd665f338c636fb9e529e9848a285209bebe6728f19961b92`.
The trace baseline above is pre-change evidence; only the frozen second
backlink and note delta are authorized. `doc/spec`, the existing `.miz` and
sidecar, every other trace field/row, C4C4 captured state, diagnostics, active
routes, checker/Core production, and Task 277B are protected.

## Reviews, verification, exit, and handoff

Before adding artifacts, independent reviews must report no blocking/high
findings for specification/equivalence and EN/JA/boundary parity. Afterward,
independent reviews cover test sufficiency, artifact/metadata implementation,
source/documentation/API consistency, and final quality; findings are repaired
and re-reviewed.

Required verification is:

```text
cargo test -q -p mizar-parser set_comprehensions
cargo test -q -p mizar-test --test metadata
cargo test -q -p mizar-test --test lint_policy
cargo test -q -p mizar-checker --test lint_policy
cargo test -q -p mizar-test --lib
cargo fmt --all --check
cargo metadata --offline --no-deps --format-version 1
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features --no-fail-fast
git diff --check
```

Recheck exact new/protected hashes, corpus/contract inventories, links and
fragments, CLI metadata counts, C4C4 empty captured state, zero production
diff, and Task-277B zero-credit status. Exit requires all `9/9` hard gates and
a valid score of at least `90/100`, exact task-only staging/commit, clean
postcommit proof, and fresh successor inventory.

The next candidate after completion is a separately frozen checker C4C8
contract for the standalone complete projection. It is ready only if fresh
inventory uniquely fixes the exact immutable API/fields, complete graph and
cardinality validator, private ordering oracle, destination/consumer, and
default-deny matrix without changing language semantics. Otherwise stop with
the classified gap; do not infer an implementation.

## Completion evidence

The exact 30-path implementation is complete. The new source is `7` lines /
`193` bytes / SHA-256
`b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`;
the sidecar is `13` lines / `885` bytes / SHA-256
`277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`.
Corpus pairs are `345/345`, contract trees are `102/102`, metadata is
`137/137`, cases/requirements are `430/396`, pass/fail is `237/193`, and
warnings/errors remain `23/0`. Requirements, stage coverage, architecture
matrix, and active route counts remain unchanged.

Final trace is `5925` lines / `464335` bytes / SHA-256
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`;
final coverage audit is `7107` lines / `541809` bytes / SHA-256
`18dbb5048e949461c03f8d59c61c2b0c63ac3bcea19d01b8d1fa2126dc8d8c39`.
The protected one-capture source/sidecar hashes remain exact. `doc/spec`, all
other existing corpus/trace rows, production Rust, C4C4 captured state,
diagnostics, active routes, Core state, and Task 277B are unchanged.

Independent specification/equivalence, bilingual/boundary, test-sufficiency,
artifact implementation, and source/documentation/API reviews all report
**NO FINDINGS**. Focused parser `1/1`, mizar-test library `623/623`, both
lint-policy suites `15/15`, metadata `137/137`, formatting, offline Cargo
metadata, full-workspace all-target/all-feature warnings-denied Clippy, full
workspace all-target/all-feature tests including the three frontend benchmark
cases, and `git diff --check` pass.

Independent final-quality review reports **NO FINDINGS**. Parent adjudication
confirms all `9/9` hard gates pass with no score cap at valid uncapped
`100/100` (`20/20/15/15/10/10/5/5`). The exact 30-path precommit status-path
hash is `38fe0671baff256460020a1b650a657f679d33690b0cf0b20e751c43d610e860`.
Exact task-only staging/cached review, commit, and clean postcommit/fresh-
successor proof remain the final procedural gates.
