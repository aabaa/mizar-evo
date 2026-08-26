# Task CORE-SOURCE-CONTEXT-33P: C4C8 Core-context prerequisite boundary

> Canonical language: English. Japanese companion:
> [../ja/CORE-SOURCE-CONTEXT-33P.md](../ja/CORE-SOURCE-CONTEXT-33P.md).

Status: complete documentation-only prerequisite. This task records the
dependency-minimal boundary selected by the user after C4C8 completion. It
does not authorize a Rust implementation or freeze the later association API.

## Identity, authority, and readiness

| Field | Frozen value |
|---|---|
| Task | `CORE-SOURCE-CONTEXT-33P` |
| Primary owner | `mizar-core` Task 33, through the Core [crate plan](../../mizar-core/en/00.crate_plan.md), [source-family decomposition](../../mizar-core/en/source_family_decomposition.md), and [TODO](../../mizar-core/en/todo.md) |
| Upstream owner | Checker Task [C4C8](CHECKER-FRAENKEL-NESTED-MULTI-CAPTURE-GRAPH-257C4C8.md), implemented by `c7595b60e7784728967cfbac9b02522f7290c942` and closed by `c5792708e5451701f86a72ac6123df99bc1d3687` |
| Authority | `doc/spec/en/`, the exact C4C7 `.miz`, its unchanged expectation and trace row, C4C8R, C4C8, then derived Core design/source inventory |
| User decision | Accept candidate 1: freeze a generic zero-semantic Core-33 prerequisite and reserve a standalone immutable C4C8 association seam before choosing an API |
| Readiness | This documentation task is uniquely ready. Actual Core33/C4C8 transport is **not ready** because owner association, destination API, allocator mapping, and complete corruption oracle are not uniquely determined. |
| Coverage | Zero semantic credit; Task 277B remains not ready and receives zero credit. |

The repository authority fixes the meaning of the C4C7 case: inner mapper
uses of `x` and `y` refer to their outer generators by resolved binding
identity, while inner generator `z` is local and is not captured. C4C8 retains
the exact resolver-owned identity graph with cardinalities `3/1/0/2/2`; its
captured subvector is `x,y` in private source/declaration order. The C4C4
outer-`x` projection remains by value with an empty `captured` vector.

Fresh inventory also fixes what is absent:

- C4C7 and C4C8 provide no authenticated checker `SourceItemId`, `SymbolId`,
  `DefinitionId`, or Core item identity for the containing functor;
- C4C8 is a standalone immutable, syntax-free and Core-ID-free checker graph;
  it is not installed in Typed or Resolved and has no installer;
- checker Task 248 accepts its closed source-context profiles, none of which
  represents nested comprehension generators or links their resolver binding
  identities to a containing functor symbol/definition;
- `CoreContextInput` presently receives caller-supplied item, variable,
  binder, and generated seeds. It supplies neither the missing authenticated
  owner bridge nor an allocator rule for this graph;
- existing checker/Core/resolver numeric IDs are distinct domains and cannot
  be reinterpreted as one another.

Accordingly, the missing Core association is `design_drift` and the absent
future executable source-derived slice is a `test_gap`. It is not a C4C8R or
C4C8 implementation defect and does not establish new language semantics.

## Frozen owner and soundness boundary

Core Task 33 remains the sole future owner of context/item/binder identity,
source and checker provenance, fresh snapshot-local `CoreVarId` allocation,
and any durable association between an authenticated Core item and the C4C8
graph. This task reserves that association as a standalone immutable seam; it
does not choose whether the eventual representation is private or public and
does not add it to `CoreContextInput`, `CoreContext`, Typed, or Resolved.

Core Task 34 owns type, attribute, evidence, coercion, and view lowering. Core
Task 35 may consume only a complete Task-33 association after Task 34 and then
owns term/formula/Fraenkel lowering and `GeneratedOrigin`. Task 35 must not
allocate, repair, infer, or recover the association. Generator-domain operands
remain distinct from captured parameter/argument subvectors; only the latter
may eventually participate in a positional one-to-one join in the C4C8 graph's
private order.

The future association oracle must reject missing, extra, duplicate,
reordered, stale, foreign-owner, cross-module, recovered, partial, mismatched,
or orphan rows. It must not join by display name, reinterpret numeric IDs,
sort, repair, infer, or admit unchecked input. These are minimum default-deny
requirements, not a frozen API or a complete executable oracle.

This task creates no task-semantic implementation ID, Rust type, field,
adapter, installer, route, destination slot, allocator, item, variable,
parameter, argument, functor, generated origin, diagnostic, expectation, trace
credit, or active runner behavior.

## Candidate comparison and unresolved decision

| Candidate | Current disposition | Reason |
|---|---|---|
| Generic Core-33 base plus reserved standalone immutable association seam | Selected for this documentation prerequisite only | Preserves the already-assigned Core33 owner and zero-semantic/default-deny boundary without pretending the missing owner bridge exists. |
| C4C8-specific private Core-33 association | Deferred | Requires a complete private destination/API, owner key, allocator mapping, parameter order, and corruption oracle. |
| Public `CoreContextInput`/`CoreContext` extension | Deferred | Changes public API and ownership exposure before authority identifies the complete consumer contract. |
| Extend checker Task 248 or add a public checker route | Deferred | Changes the checker's closed profile/API boundary and needs separate checker authority. |
| Install a second Typed/Resolved receipt | Forbidden under the current contracts | C4C6 owns the exact existing receipt; C4C8 deliberately remained standalone. |
| Lower directly in Core Task 35 | Forbidden | Skips the Core33 association owner and would force Task35 to infer or allocate soundness-critical identity. |

Before an implementation successor can start, authority must uniquely choose
the authenticated containing-owner bridge, exact association destination and
visibility, identity-preserving allocator mapping, immutable API, captured
parameter/argument cardinality and order, and complete default-deny corruption
oracle. The minimum human decision, if repository authority remains unchanged,
is whether Core33 receives a new checker-authenticated containing-owner link
through a checker-private route or exposes a new public Core input. No choice
is made here.

## Exact documentation scope and protected surfaces

This task may change exactly these ten documentation paths:

1. `doc/design/task_contracts/en/CORE-SOURCE-CONTEXT-33P.md`;
2. `doc/design/task_contracts/ja/CORE-SOURCE-CONTEXT-33P.md`;
3. `doc/design/mizar-core/en/00.crate_plan.md`;
4. `doc/design/mizar-core/ja/00.crate_plan.md`;
5. `doc/design/mizar-core/en/source_family_decomposition.md`;
6. `doc/design/mizar-core/ja/source_family_decomposition.md`;
7. `doc/design/mizar-core/en/todo.md`;
8. `doc/design/mizar-core/ja/todo.md`;
9. `doc/design/task_contracts/en/CHECKER-FRAENKEL-NESTED-MULTI-CAPTURE-GRAPH-257C4C8.md`;
10. `doc/design/task_contracts/ja/CHECKER-FRAENKEL-NESTED-MULTI-CAPTURE-GRAPH-257C4C8.md`.

The central specification-coverage audit and Core source/spec audit remain
unchanged because this task changes no specification coverage, implementation
owner, API, source correspondence, trace state, test ownership, or deferred
coverage credit. All `doc/spec`, existing `.miz`, expectation, trace, source,
Cargo, diagnostics, C4C4 captured state, Typed/Resolved state, and active route
artifacts are protected.

At entry the eight existing scoped documents had these line/byte and SHA-256
baselines:

| Path | Lines / bytes | SHA-256 |
|---|---:|---|
| Core EN plan | `330 / 42980` | `644735ceaf6452f5aa7a496383efbee996634bdce7efc5f30cc9668efc128eb4` |
| Core JA plan | `312 / 45108` | `52b699187da370b20bb344d5c55750a8ef99abcc428f6949c4b217123b56b1fd` |
| Core EN decomposition | `215 / 21666` | `99da1bdc812f44d6f3c89c88b0bfcad533d510c0accd2c12964cffd4441ec413` |
| Core JA decomposition | `195 / 20965` | `14f1cc6f34b48e94250aab4cd8eace7f1fc1f6543a2cbbcaae55b208837b36ff` |
| Core EN TODO | `626 / 36734` | `6e3c3ca6633b36bfd8c9e250e664640bdcdab698e51591a076408bcca15637e7` |
| Core JA TODO | `599 / 40467` | `524bbd47144211cd13ac37bd8d7cd30390010044abaf5259c8bb27de29bcd4fd` |
| C4C8 EN contract | `442 / 21798` | `20d34dd22d255005dc37d1e8eee32417e45c26e175c5b10c90b1e36a7ca3d8ed` |
| C4C8 JA contract | `238 / 15528` | `87969ca5feb45105f1629cec7e827138cde4eab2d4c904b8582289c71a900d7d` |

The paired task-contract trees are `105/105` and become exactly `106/106`.
The protected C4C7 source, expectation, and trace hashes are respectively
`b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`,
`277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`,
and `17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`.
The protected stash is
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`. Entry HEAD is
`c5792708e5451701f86a72ac6123df99bc1d3687`, origin/main is
`481a599877803e855307381901b82ae38365ce4a`, and divergence is `0/2`.

## Review, verification, and exit

Independent specification/equivalence and bilingual/boundary reviews must end
with no findings. Because the implementation is documentation-only, the
post-freeze test-sufficiency, implementation-equivalence, and source/docs/API
reviews verify absence rather than authorize code. A final read-only review
must pass all nine autonomous hard gates and score at least `90/100`.

Required checks are Core and mizar-test lint-policy suites, recursive paired
contract/link validation, `cargo fmt --all -- --check`, offline Cargo metadata,
workspace all-target/all-feature warnings-denied Clippy, full workspace tests,
`git diff --check`, exact scope/count/hash/protected checks, exact task-only
staging, commit, and clean postcommit proof.

Exit requires the paired contract and four paired owner references to agree;
no non-documentation path to change; all protected hashes and stash to remain
unchanged; Task 277B to remain not ready/zero credit; and the next inventory to
stop actual implementation unless every owner/API/oracle decision above has
become unique.

## Precommit completion evidence

Independent specification/equivalence, bilingual/boundary, test-sufficiency,
and implementation/source-documentation/API reviews ended **NO FINDINGS**. The
only intermediate verification finding was mechanical: recursive contract lint
required the new Core-plan `Task Index` backlinks and the exact Japanese
`canonical English:` marker. Both were repaired within scope, the focused lint
was rerun, and independent review of the repaired state ended **NO FINDINGS**.

Core lint passed `12/12`; mizar-test lint, including recursive contract/link
validation, passed `15/15`; `cargo fmt --all -- --check`, offline Cargo
metadata, workspace all-target/all-feature warnings-denied Clippy, full
workspace all-feature tests and doctests, and `git diff --check` passed. The
working tree contains exactly the ten frozen documentation paths, with sorted
path-list SHA-256
`20cfc1f5339cc29760a37b3faaee19f5c25aa1c3f98f174ebffc31cd16d44084`.
The contract trees are exactly `106/106`; the three protected C4C7 hashes,
stash, origin, source/API/route state, and Task 277B status remain unchanged.
Independent final-quality review ended **NO FINDINGS**: all `9/9` hard gates
pass with no score cap and an uncapped `100/100` (`20/20` specification,
`20/20` tests, `15/15` traceability, `15/15` implementation equivalence,
`10/10` synchronization, `10/10` boundary discipline, `5/5` verification, and
`5/5` handoff). Exact commit identity, clean postcommit proof, and the fresh
successor inventory are recorded by the external final handoff because a
commit cannot contain its own hash.
