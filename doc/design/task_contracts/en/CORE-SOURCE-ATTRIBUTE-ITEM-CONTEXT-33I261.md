# Task CORE-SOURCE-ATTRIBUTE-ITEM-CONTEXT-33I261: Task261 attribute item context

> Canonical language: English. Japanese companion:
> [../ja/CORE-SOURCE-ATTRIBUTE-ITEM-CONTEXT-33I261.md](../ja/CORE-SOURCE-ATTRIBUTE-ITEM-CONTEXT-33I261.md).

Status: implementation and verification complete. The exact task-only commit
and postcommit successor inventory are pending. This is the
user-selected Task-261-specific successor to completed Core Task 33I260. It is
zero-semantic and zero-credit and does not complete Core 33 or activate
`MT10-CIR-TE`.

## Identity, authority, and decision

| Field | Contract value |
|---|---|
| Task | `CORE-SOURCE-ATTRIBUTE-ITEM-CONTEXT-33I261` |
| Primary owner | `mizar-core::elaborator`, Core Task 33 |
| Owning plan | [`mizar-core` crate plan](../../mizar-core/en/00.crate_plan.md) |
| Checker dependency | Existing Task-248 Profile-B `SourceBindingContextHandoff` and active Task-261 `SourceAttributeDefinitionHandoff` |
| Core dependency | Completed `CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB`; completed Task-259/260 handoffs are protected precedents, not inputs |
| Prepared consumer | Future `MT10-CIR-TE`, only after complete Core 33--35 lowering produces one deterministic real `CoreIr` |
| User decision | Select the exact Task-261 family-specific one-row standalone handoff ahead of the also-ready Task-262 family |
| Coverage | Zero semantic/execution credit; Task277B remains not ready and receives zero credit |

Authority remains, in order, `doc/spec/en/`, existing `.miz` sources, trace
metadata, expectations, design, then source. Chapter 6 Sections 6.1, 6.2,
6.8.1, and 6.9 fix the ordinary attribute-definition form and predicate-style
identity. Chapters 11 and 12 fix current-module symbol identity, visibility,
source order, and activation only after the declaration item. Chapter 16
Sections 16.6 and 16.7.2 reserve attribute-specific correctness for
redefinition coherence; the exact ordinary Task-261 definition owns no initial
obligation.

The existing pass source and checker handoff authenticate one normal public
attribute definition inside one normal Task-248 Profile-B definition block.
Fresh post-Task-260 inventory found both Task 261 and Task 262 technically
ready, with no repository tie-break. The user's adoption supplies only that
ordering decision. There is no `spec_gap`: this is derived phase transport.
The missing Core association and private Core consumer are bounded
`design_drift` and `test_gap`; implementation closes only those gaps.
The remote baseline mismatch and the external Task-260-era remote-tracking
update remain report-only `repo_metadata_conflict`.

## Frozen public API and ownership

`crates/mizar-core/src/elaborator.rs` may add only:

- immutable `SourceAttributeCoreItemAssociation`, with getters
  `source_item()`, `definition()`, `symbol()`, and `core_item()`;
- immutable source-ordered `SourceAttributeCoreItemAssociationTable`, with
  `get(SourceAttributeDefinitionId)`, `iter()`, `len()`, and `is_empty()`;
- immutable `SourceAttributeCoreContextHandoff`, retaining by value the
  complete 33LB handoff, Task-248 source context, Task-261 checker handoff, and
  association table, with getters `source_id()`, `module_id()`, `context()`,
  `source_bindings()`, `source_context()`, `checker_owner()`, `items()`, and
  non-authoritative `debug_text()`;
- non-exhaustive `SourceAttributeCoreContextError`, in precedence order:
  `EnvironmentMismatch`, `InvalidSourceBindingContext`,
  `InvalidCheckerOwner`, `InvalidCoreContext`, and
  `InvalidItemAssociation`;
- `SourceAttributeCoreContextProducer::build(
  SourceBindingCoreContextHandoff,
  SourceBindingContextHandoff,
  SourceAttributeDefinitionHandoff,
  ) -> Result<SourceAttributeCoreContextHandoff,
  SourceAttributeCoreContextError>`.

All fields are private. The producer consumes every input by value and
publishes only after complete postvalidation. It adds no constructor, adapter,
installer, unchecked admission, compatibility layer, `CoreContextInput`/
`CoreContext`/`CoreIr` field, or Typed/Resolved slot. It does not alter the
33LB, 33I259, or 33I260 public API and does not introduce a public generic
definition-family abstraction.

## Cardinality, identity, order, and provenance

The admitted profile is exact:

- source bytes: the existing 116-byte final-LF Task-261 pass source, SHA-256
  `ffd4954aad628d7946aaf7afb1b472a6bdfca7bce5ba0cf09f5b284c9dda07bf`;
- Task 248: exact Profile B `1/2/2/2/2/2/0`, with one normal
  `DefinitionBlock` `SourceItemId(0)`, two ordered normal definition-parameter
  declarations/bindings, exact module/definition binding and local-type
  contexts, two context links, and no diagnostics;
- Task 261: definitions/parameters/subjects/definientia `1/2/1/1`, exact
  nonempty Task-248/249/252/256 fingerprints, and no initial-obligation input or
  projection;
- definition 0: `SourceAttributeDefinitionId(0)`, whole attribute symbol,
  resolver `DefinitionId(0)`, contribution 0, source ordinal 0,
  `BindingContextId(1)`, site node 40, inner range `45..110`, spelling
  `attr Task261AttributeDefinition: x is task261_marked means x = y;`, normal
  recovery, local origin range `45..110`, and structural path `[4,0,7,0]`;
- parameters 0/1 retain bindings 0/1, type applications 0/1, sites 27/31,
  ordinals 0/1, owner ranges `13..26`/`29..42`, declaration ranges
  `17..18`/`33..34`, context 1, and exact spellings;
- subject 0 retains binding 0, site 40, token range `78..79`, context 1, and
  spelling `x`; definiens 0 retains atomic formula 0, site 39, range
  `104..109`, context 1, and spelling `x = y`;
- Core: exactly one valid public `Attribute` item selected by the retained
  whole `SymbolId`, with no dependencies, diagnostics, imports, generated
  origins, or partial/recovered state; it has one pending `DefinitionalItem`
  boundary and one pending worklist entry.

The definition's exact context link selects `SourceItemId(0)`. The association
table has exactly one row keyed by the typed `SourceAttributeDefinitionId(0)`.
No checker, resolver, or Core numeric id is reinterpreted; the Core item is
selected only by exact whole-symbol registry lookup. No display name, FQN
alone, range alone, shell ordinal, seed order, map iteration, or worklist
iteration is a join key.

The Core item, source-map row, boundary, and worklist row use the inner
definition range `45..110`, never the outer block range `0..115`, and exactly
one checker provenance key:
`source-attribute-core-item-v1.definition.0`. `CoreItemStatus::Valid` records
only an authenticated item shell. The body remains `PendingBody`; the equality
formula remains checker-owned and is not lowered or interpreted here.

## Default-deny oracle

Validation rejects, without sorting, repair, inference, recovery, unchecked
admission, or partial publication:

1. source/module mismatch across retained handoffs, foreign inputs, or an
   unequal 33LB and Task-248 `BindingEnv`;
2. stale/nonexact Task-248 Profile-B item, declaration, binding, context,
   local-context, link, range, site, role, order, ownership, recovery, or
   diagnostic state;
3. stale/nonexact Task-261 cardinality, lower fingerprint, resolver identity,
   symbol/definition/contribution, origin, definition, parameter, subject, or
   definiens row;
4. a missing, `None`, foreign, or mismatched context link/source item;
5. missing, extra, duplicate, reordered, stale, mismatched, or orphan
   association rows;
6. missing/extra Core items or wrong whole symbol, kind, visibility, status,
   inner source range, provenance, source-map row, worklist order/state,
   dependency, diagnostic, generated-origin, or boundary state;
7. any join by display name, spelling, FQN alone, range alone, numeric id,
   shell ordinal, seed order, map iteration, or worklist iteration.

The producer revalidates the complete immutable inputs and built association
before returning. Missing/extra/duplicate/reordered/stale/mismatch/recovered/
partial states fail closed.

## Installation boundary and semantic deferrals

Only the existing private Task-261 real-source test leaf may construct the one
Core item seed from the authenticated definition, prepare the Core context,
apply 33LB to the retained complete `BindingEnv`, and invoke the standalone
producer. It verifies retained inputs, the exact one-row association, full
item/source-map/boundary/worklist state, deterministic replay, and the
default-deny mutation matrix.

There is no production runner branch or installation into `TypedAst`,
`ResolvedTypedAst`, `CoreContext`, or `CoreIr`. No `.miz`, expectation, trace,
active result, diagnostic, metadata count, or coverage state changes.

Task-262--264 owner families, a generic/complete Core-33 item inventory, Core
34 attribute/type/evidence semantics, Core 35 formula semantics, Core 36
attribute-definition parameters/body/correctness lowering, attribute
applications, redefinition/coherence, initial obligations, proof or
acceptance, `GeneratedOrigin`, C4C8 composition, snapshots, `MT10-CIR-TE`,
diagnostics, and Task277B remain deferred. Task 261 earns zero Core credit.

## Affected artifacts and audit impact

Source changes are exactly:

1. `crates/mizar-core/src/elaborator.rs`;
2. `crates/mizar-test/src/runner/tests/type_elaboration/source_attribute_definition.rs`.

Derived documentation is limited to this paired contract; paired Core plan,
source-family decomposition, TODO, elaborator, source/spec audit, bilingual
audit, and task ledger; paired mizar-test harness and bilingual audit; and
`doc/design/spec_coverage_audit.md`. Checker documents remain unchanged
because Task 248 and Task 261 ownership/API do not change.

The central audit records only a zero-credit Core mapping and narrowed
follow-up ownership. Specification, existing test intent, trace status and
backlinks, and coverage credit do not change.

At freeze, `elaborator.rs` is `20805 / 775898`, SHA-256
`b8ca96a9ca86078b664a2f6f2581f45f820f13b9dff20ee624adbb32e04aa22e`;
the Task-261 test leaf is `1113 / 41268`, SHA-256
`6d7f492627f32f80df9a9dd17fb0548bae3a1107279837013d7f38556053766d`.
The paired task-contract trees are `112/112` and become exactly `113/113`.
The implementation adds exactly two Task-261 private tests; Core library tests
stay `163`, mizar-test library tests project `636 -> 638`, and metadata tests
stay `137`. Final source counts and hashes are measured once in this contract
before commit.

Protected values include the Task-261 source/expectation hashes
`ffd4954aad628d7946aaf7afb1b472a6bdfca7bce5ba0cf09f5b284c9dda07bf` /
`ed8bc242f86206a56d178ef1d665faaa36c24d4943e7ca70e53af3decbecf4d8`,
all frozen Task-260/259/248/reserve/C4C7 source and expectation hashes, trace
SHA-256 `17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`,
and stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4`.

Entry `HEAD` is `f8e9fc212f1c24a65b7fa1b2faa0e57e18927b9e`;
actual `origin/main` is `de42b58f7322128566326c8ee1d3d1e9a5fe4d77`
with divergence `0/1`. This differs from the originally requested remote
baseline and remains report-only `repo_metadata_conflict`. No fetch, push,
stash mutation, or metadata repair is authorized.

## Reviews, verification, exit, and handoff

Before source edits, independent specification/equivalence and bilingual/
boundary reviews must end with no findings. After implementation, independent
test-sufficiency, full implementation, and source/documentation/API reviews
must end with no findings after finding-specific repair.

Focused verification runs the two Task-261 Core-context tests, existing
Task-261 checker route, the protected Task-259/260/33LB probes, Core and
mizar-test lint policies, and metadata lint. Required final verification is
`cargo fmt --all -- --check`, offline Cargo metadata,
`cargo clippy --all-targets --all-features -- -D warnings`, and
`cargo test --all-features`, followed by protected hash/count/status and
`git diff --check` checks. Stable broad suites are not repeated after
documentation-only completion-evidence edits.

Exit requires all autonomous hard gates `9/9`, a valid parent score of at
least `90/100`, exact task-only staging/commit, clean postcommit proof,
protected invariance, Task277B not-ready/zero-credit, and a fresh read-only
successor inventory. The successor is not selected by this contract.

## Completion evidence

The standalone producer and the exact two-test private Task-261 consumer are
complete. Final source measurements are `elaborator.rs` `21540 / 805739`,
SHA-256 `68d9623412dc1f1186ded06eff762d498e6d5b5431eca0f018bcc55df28ea07a`,
and the Task-261 test leaf `1510 / 56394`, SHA-256
`f4bfcaa0fe0446b36a316b06763d39ca84a37bb1acc4e18b3e212de022341c0e`.
The paired task-contract trees are exactly `113/113`; Core library tests are
`163`, mizar-test library tests are `638` (`636 + 2`), and metadata tests are
`137`.

The pre-source specification/equivalence review had no findings. The
pre-source bilingual/boundary review found the five new public items missing
from the paired public-API inventories; both inventories were fixed and its
re-review had no findings. The post-source test-sufficiency review found
missing direct evidence that dependency summaries, Core diagnostics, and
generated origins remain empty; four exact assertions were added and
re-review had no findings. The implementation review found three Task-248
Profile-B exactness checks missing. Its first repair was accidentally applied
in part to the protected Task-259 validator; the reviewer caught that
regression, the Task-259 changes were removed, the checks were placed only in
Task 261, and final re-review had no findings. Nonempty Task-249/252/256
fingerprints remain the frozen checker-handoff trust boundary; adding lower
handoff inputs would violate the frozen API. The source/documentation/API
review found stale completion status and a missing Japanese return signature;
both are repaired in this completion update and final re-review is required
before staging.

Focused Task-261 Core-context tests pass `2/2`; protected Task-259 and Task-260
probes each pass `2/2`; Core tests pass `163/163`; mizar-test lint passes
`15/15`; metadata passes `137/137`. `cargo fmt --all -- --check`, offline Cargo
metadata, `cargo clippy --all-targets --all-features -- -D warnings`, and
`cargo test --all-features`, including integration tests and doctests, pass.
The protected Task-261 source/expectation and trace hashes match the frozen
contract, and protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` is unchanged.

Parent hard gates pass `9/9`. The valid uncapped score is `98/100`:
specification `20/20`, test contract `19/20`, traceability `15/15`,
implementation `14/15`, design/source synchronization `10/10`, boundary
discipline `10/10`, verification `5/5`, and handoff `5/5`; no cap applies.
Task 261 remains zero-credit. Core 34--36, `GeneratedOrigin`, production
installation, `MT10-CIR-TE`, diagnostics, coverage credit, and Task277B remain
deferred; Task277B is not-ready/zero-credit.

The report-only `repo_metadata_conflict` remains. Precommit `HEAD` is
`f8e9fc212f1c24a65b7fa1b2faa0e57e18927b9e`; actual `origin/main` is
`de42b58f7322128566326c8ee1d3d1e9a5fe4d77` with divergence `0/1`. This
agent performed no fetch, push, stash mutation, or metadata repair. The exact
task-only commit and fresh postcommit successor inventory are pending.
