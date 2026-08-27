# Task CHECKER-FRAENKEL-CAPTURE-GRAPH-OWNER-33C: Opaque graph-owner receipt

> Canonical language: English. Japanese companion:
> [../ja/CHECKER-FRAENKEL-CAPTURE-GRAPH-OWNER-33C.md](../ja/CHECKER-FRAENKEL-CAPTURE-GRAPH-OWNER-33C.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index). Durable
owner sections: checker
[source formula composition](../../mizar-checker/en/source_formula_composition.md#task-33c-opaque-capture-graph-owner-receipt)
and test [harness](../../mizar-test/en/harness.md#checker-task-33c-private-graph-owner-probe).

## Status, decision, and readiness

**Status:** implementation, all reviews, broad verification, protected checks,
and final-quality scoring complete; staging and commit pending.

This is the dependency-minimal zero-semantic successor to completed C4C8 and
Task33R. Fresh read-only inventory leaves exactly one lower-stage composition:
the existing checker `source_formula_composition` owner may retain the
unchanged C4C8 graph and resolver Task33R owner receipt as one opaque scalar
receipt. Neither Typed/Resolved nor Core may own this prerequisite without
prematurely selecting an installation or semantic destination.

Repository authority did not previously choose the exact composite API and
oracle. The user adopted the reviewed recommendation and thereby authorizes
this narrow decision: one checker-owned immutable one-to-one receipt retains
both inputs by value; it has no table or dense id; it exposes only the two
receipts plus common source/module identity; and every dependency or
association mismatch fails closed. The absent composition is `design_drift`;
the missing replay, corruption, freshness, display-independence, and real-
fixture probes are a `test_gap`. There is no blocking `spec_gap` or
`repo_metadata_conflict`.

## Authority, dependencies, and fixed meaning

Authority remains, in order: `doc/spec/en/`; the exact existing
[C4C7 source](../../../../tests/miz/pass/types/pass_types_nested_comprehension_two_outer_generator_captures_001.miz);
the canonical [trace](../../../../tests/coverage/spec_trace.toml); its unchanged
[expectation](../../../../tests/miz/pass/types/pass_types_nested_comprehension_two_outer_generator_captures_001.expect.toml);
`doc/design/`; then non-normative source inventory. Required completed dependencies are C4C5
(`72662d38`), C4C7 (`3d28af5f`), C4C8R (`a710b4f1`), C4C8 plus closure
(`c7595b60`, `c5792708`), Core-33P (`332d752c`), and Task33R
(`e94f36cf9785b9d1ffe965045b19aa42b89caedc`).

The fixed meaning remains unchanged: inner mapper `x` and `y` refer to the
outer generator resolved binding identities; inner generator `z` is local and
is not captured; association never uses display spelling. C4C4 remains
by-value with an empty `captured` vector. Resolver, checker, and Core numeric
ids remain separate domains. C4C5/C4C6 remain the separate exact-one receipt
and Typed/Resolved installation; this task neither generalizes nor replaces
them.

## Frozen public API and ownership

The existing checker `source_formula_composition` module is sole owner of
exactly these new public items:

- `SourceNestedFraenkelCaptureGraphOwnerHandoff`;
- non-exhaustive `SourceNestedFraenkelCaptureGraphOwnerError`;
- `SourceNestedFraenkelCaptureGraphOwnerProducer`.

There is no new id, row, table, installer, adapter, builder, default, mutator,
or unchecked constructor. Only the producer constructs the handoff. Its exact
entry point consumes both immutable prerequisites by value:

```rust
pub fn build(
    graph: SourceNestedFraenkelCaptureGraphHandoff,
    owner: SourceNestedFraenkelFunctorOwnerHandoff,
) -> Result<SourceNestedFraenkelCaptureGraphOwnerHandoff,
            SourceNestedFraenkelCaptureGraphOwnerError>
```

The handoff is `#[derive(Clone, PartialEq, Eq)]`, has private fields, retains
one graph and one owner by value, and freezes this complete public getter
surface:

```rust
pub const fn source_id(&self) -> SourceId
pub const fn module_id(&self) -> &ModuleId
pub const fn graph(&self) -> &SourceNestedFraenkelCaptureGraphHandoff
pub const fn owner(&self) -> &SourceNestedFraenkelFunctorOwnerHandoff
pub fn debug_text(&self) -> String
```

Complete revalidation is crate-private because no public downstream consumer
is selected. It is invoked by the producer and checker-owned tests. The
handoff privately retains one scalar association snapshot containing only the
common `SourceId`, `ModuleId`, definition-block `ResolvedNodeId`, and functor-
definition `ResolvedNodeId`. The snapshot is derived from authenticated
inputs; it is not a second owner row and publishes no Core identity.

`debug_text()` is deterministic, diagnostics-free, and exactly:

```text
source-nested-fraenkel-capture-graph-owner-v1|module=<package>.<path>|captures=2|occurrences=2|symbol=<fully-qualified-name>|definition=<index>
```

Display spelling occurs only in this debug rendering through the already
authenticated final symbol. It is never an association or admission key.

## Frozen association and default-deny oracle

Validation precedence is exact:

1. `InvalidGraphDependency`: C4C8 `validate_complete()` rejects any retained
   graph dependency or graph-row failure;
2. `InvalidOwnerDependency`: Task33R `validate_resolver_collection()` rejects
   any owner failure or anything other than exact equality with the resolver
   snapshot retained privately by that graph;
3. `InvalidAssociation`: common source/module identity differs, the private
   scalar snapshot differs from fresh derivation, or any graph generator,
   mapper, or predicate row differs from the Task33R definition-block or
   functor-definition identity.

The producer executes the same precedence before publication. Cardinality is
exactly one graph and one owner because the handoff is scalar. It introduces
no observable collection order; C4C8's authenticated private order and exact
`3/1/0/2/2` shape remain unchanged. Association compares typed identities
directly. It never joins by display name or range and never reinterprets a
resolver, checker, or Core numeric id as another domain.

The error type has exactly these three fieldless variants, derives
`Debug, Clone, Copy, PartialEq, Eq`, is `#[non_exhaustive]`, and implements
`std::fmt::Display` and `std::error::Error` with these exact strings in the
same order:

```text
nested Fraenkel capture graph-owner graph dependency is invalid
nested Fraenkel capture graph-owner owner dependency is invalid
nested Fraenkel capture graph-owner association is invalid
```

`SourceNestedFraenkelCaptureGraphOwnerProducer` derives
`Debug, Clone, Copy`.

Missing, extra, duplicate, reordered, stale, foreign-source, foreign-module,
recovered, partial, mismatched, corrupted, or cross-snapshot input is rejected
atomically. No sort, repair, inference, display-name join, range join, numeric-
id reinterpretation, partial recovery, or unchecked admission is permitted.

## Exact scope, tests, and protected surfaces

This logical task may change exactly these 24 paths:

1. this paired EN/JA contract;
2. paired checker `00.crate_plan.md` Task Index rows;
3. paired checker `source_formula_composition.md` owner/API sections;
4. paired checker `todo.md` task sections;
5. paired checker `source_spec_audit.md` zero-credit correspondence sections;
6. paired checker `module_boundary_audit.md` boundary/inventory sections;
7. paired checker `bilingual_sync_audit.md` records;
8. paired mizar-test `00.crate_plan.md` Task Index rows;
9. paired mizar-test `harness.md` private-probe sections;
10. paired mizar-test `bilingual_sync_audit.md` records;
11. `doc/design/spec_coverage_audit.md` zero-credit owner mapping;
12. `crates/mizar-checker/src/source_formula_composition.rs`;
13. `crates/mizar-checker/tests/lint_policy.rs`; and
14. the existing private
    `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs`.

Checker adds exactly four tests:

- `task33c_builds_exact_graph_owner_handoff`;
- `task33c_rejects_graph_and_owner_dependencies_in_precedence`;
- `task33c_rejects_source_module_identity_and_retained_association_corruption`;
- `task33c_replays_immutably_and_rejects_stale_or_display_joined_pairs`.

The existing private mizar-test leaf adds exactly
`task33c_real_fixture_pairs_capture_graph_with_exact_functor_owner`. It uses
the unchanged C4C7 source, builds the exact resolver collection, C4C8 graph,
Task33R owner, and this receipt, and proves borrowed identity, exact common
owner ids, immutable replay, local-inner exclusion, unchanged import
augmentation, and zero semantic installation.

No Core source, Typed/Resolved field, C4C4 captured state, active runner,
diagnostic, Cargo manifest, `doc/spec`, existing `.miz`, expectation, or trace
metadata change is authorized. No parameter/argument order, `GeneratedOrigin`,
semantic result, active route, coverage credit, or Task277B readiness is
created.

## Baseline and expected impact

Entry HEAD and origin/main are both
`e94f36cf9785b9d1ffe965045b19aa42b89caedc`; divergence is `0/0`, worktree and
index are clean, and protected stash is
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`. Contract trees are `107/107` and
become `108/108`.

| Rust path | Baseline lines / bytes | Baseline SHA-256 |
|---|---:|---|
| checker source owner | `12132 / 472546` | `e7242ebf7344b1e89646fefe2dd9e1ad41d40be22b526c872327540ba7abad12` |
| checker lint policy | `1955 / 63228` | `f8c0c2c196e476b744716d51d8252a61f667536ef97a441246519b3b1a6dd2a0` |
| private mizar-test leaf | `932 / 37659` | `1b881477a9a17f5bab425e03d8ea59c656c1f28f56bdc57e211652b30a00d2c4` |

Checker library tests project `576 -> 580`; mizar-test library tests project
`626 -> 627`. Sorted baseline raw-list SHA-256 values are checker
`90263b289873fda7dd480010a8fbca2f7c491366c3afbd10f2cba471f5f112bd`
and mizar-test
`e54d5c97f46e65d4657d5e99b7efa609cd39a096020c4339b512fdbf039b0694`.

Protected C4C7 source, expectation, and trace hashes are respectively
`b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`,
`277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`,
and `17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`.

The checker public-enum inventory changes exactly `9 -> 10`. The central
coverage audit records only the new derived zero-credit owner and unchanged
follow-up boundary; Chapter 13 remains `partial`. No traceability count,
requirement status, fixture, expectation, diagnostic, or semantic coverage
changes. The schema-v2 compaction ledger is unchanged.

## Reviews, verification, exit, and handoff

Before Rust edits, independent specification/equivalence and bilingual/
boundary reviews must report **NO FINDINGS**. After implementation,
independent test-sufficiency, implementation, source/documentation/API, and
final-quality reviews must report **NO FINDINGS**, with finding-specific
re-review after every repair.

Required verification includes all four checker tests, the private real-
fixture probe, C4C8 and Task33R compatibility tests, checker and mizar-test
library/lint suites, mizar-test metadata, parser set-comprehension coverage,
formatting, offline Cargo metadata, workspace all-target/all-feature warnings-
denied Clippy, full workspace all-feature tests, `git diff --check`, exact
scope/count/hash/link checks, and protected invariance.

Exit requires all autonomous hard gates `9/9`, a valid score at least
`90/100`, exact task-only staging and commit, clean postcommit proof, and fresh
successor inventory. Core Task33 installation, Core Task34/35 transport,
free/generated-parameter order, `GeneratedOrigin`, actual semantics, active
route, and Task277B remain separately authority-gated and deferred.

The next-task handoff is: start from the clean Task33C commit with a fresh
read-only inventory; audit the dependency-minimal same-milestone successor;
freeze a new task only if authority uniquely fixes its owner, dependencies,
scope, public API, association, cardinality/order, complete fail-closed oracle,
and protected-state impact. Do not infer Core Task33/35 destination or free/
generated-parameter order, `GeneratedOrigin`, semantics, or an active route.
Use GPT-5.6 Sol `xhigh` for the parent authority and final hard-gate decision,
GPT-5.6 Luna `xhigh` only for bounded work after contract freeze, and escalate
cross-module precision to Terra `high` only when Luna evidence is insufficient.

## Precommit implementation evidence

Implementation changes exactly the frozen 24 paths: 21 documentation paths
and the three Rust paths. Contract trees are `108/108`; checker and mizar-test
library inventories are exactly `580` and `627` tests. Their final raw-list
SHA-256 values from `cargo test -q -p <crate> --lib -- --list | sha256sum`
are respectively
`269021fdb0a7b7d1f30bb4a82ffc4fa544d6224ed7ecfcd8bf27186eef254d7c`
and
`3612205e538fbba237f955aaa2a042f6874c31413311851bc94fa73d45a09744`.

Final Rust measurements are:

| Path | Lines / bytes | SHA-256 |
|---|---:|---|
| `crates/mizar-checker/src/source_formula_composition.rs` | `12475 / 486708` | `dcff2322170389f17d4ed01e00e47ea70a07008906d9ad4358dfeca2e232a7a8` |
| `crates/mizar-checker/tests/lint_policy.rs` | `1989 / 64304` | `3c726af3c41a0a28faf0c8ca0770a815293624ee1424ce31bd8575b97f299d30` |
| private mizar-test leaf | `1008 / 40967` | `c38e7c2c99d3b81fb8906edaf90244c7d08eca913c6758bc5f7064a10bfcbcd8` |

The four focused checker tests, private real-fixture probe, C4C8/Task33R and
parser-comprehension compatibility tests, checker/mizar-test library and lint
suites, mizar-test metadata, formatting, offline Cargo metadata, workspace
warnings-denied Clippy, full workspace all-feature tests including doctests,
recursive contract/link lint, and `git diff --check` pass. Independent test-
sufficiency, implementation, and source/documentation/API plus bilingual/
boundary reviews are **NO FINDINGS** after finding-specific repairs to the
combined precedence test and stale EN/JA audit tense. Exact 24-path accounting,
`108/108` contract parity, final Rust counts/hashes, and the three protected
C4C7 hashes pass. The sorted 24-path inventory SHA-256 is
`b2af48b0239f8f00570d5aa1d3f0fc89e2f03fae8543c99e53fd87d83acd3667`.
Independent final-quality review is **NO FINDINGS**: all `9/9` hard gates pass
with no score cap and the valid uncapped score is `100/100` (`20/20`
specification, `20/20` tests, `15/15` traceability, `15/15` implementation,
`10/10` synchronization, `10/10` boundaries, `5/5` verification, and `5/5`
handoff). Exact staging, commit, and postcommit inventory remain pending.
