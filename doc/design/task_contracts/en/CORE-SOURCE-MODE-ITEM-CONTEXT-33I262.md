# Task CORE-SOURCE-MODE-ITEM-CONTEXT-33I262: Task262 mode item context

> Canonical language: English. Japanese companion:
> [../ja/CORE-SOURCE-MODE-ITEM-CONTEXT-33I262.md](../ja/CORE-SOURCE-MODE-ITEM-CONTEXT-33I262.md).

Status: implementation and verification complete; independent pre-source and
post-source reviews ended with no findings after documentation repairs.
Task-only commit and postcommit inventory are pending. This zero-semantic/
zero-credit Core-33 prerequisite does not complete Core 33 or activate
`MT10-CIR-TE`.

## Identity, authority, and readiness

| Field | Contract value |
|---|---|
| Task | `CORE-SOURCE-MODE-ITEM-CONTEXT-33I262` |
| Primary owner | `mizar-core::elaborator`, Core Task 33 |
| Owning plan | [`mizar-core` crate plan](../../mizar-core/en/00.crate_plan.md) |
| Checker dependency | Exact Task-248 Profile-B `SourceBindingContextHandoff` and active Task-262 `SourceModeDefinitionHandoff` |
| Core dependency | Completed `CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB`; Tasks 33I259--261 are protected precedents, not inputs |
| Prepared consumer | Future `MT10-CIR-TE`, only after complete Core 33--35 lowering produces deterministic real `CoreIr` |
| Coverage | Zero semantic/execution credit; Task277B remains not-ready/zero-credit |

Authority remains `doc/spec/en/`, existing `.miz`, trace metadata,
expectations, design, then source. Chapter 7 Sections 7.1, 7.2, 7.7, 7.8,
and 7.9 fix ordinary mode identity, parameter order, the normalized RHS
inhabitation boundary, `sethood`, and predicate-style encoding. Chapters 11
and 12 fix current-module symbol identity, visibility, and source order;
Chapter 16 keeps correctness as an obligation boundary.

The active 141-byte Task-262 source, complete checker handoff, and private
runner already authenticate the required identity. Checker ordering names
Task 262 after Task 261, and the Task-261 contract records the user decision to
place Task 261 ahead of the then-also-ready Task 262. This makes Task 262 the
dependency-minimal family-specific successor. There is no `spec_gap`.
The missing Core association and Core consumer are bounded `design_drift` and
`test_gap`. Task 263's multi-structure inventory and Task 264's property-shell
identity remain later distinct tasks.

## Frozen public API and ownership

`crates/mizar-core/src/elaborator.rs` may add only:

- immutable `SourceModeCoreItemAssociation`, with getters `source_item()`,
  `definition()`, `symbol()`, and `core_item()`;
- immutable source-ordered `SourceModeCoreItemAssociationTable`, with
  `get(SourceModeDefinitionId)`, `iter()`, `len()`, and `is_empty()`;
- immutable `SourceModeCoreContextHandoff`, retaining by value the complete
  33LB handoff, Task-248 source context, Task-262 checker handoff, and
  association table, with getters `source_id()`, `module_id()`, `context()`,
  `source_bindings()`, `source_context()`, `checker_owner()`, `items()`, and
  non-authoritative `debug_text()`;
- non-exhaustive `SourceModeCoreContextError`, in precedence order:
  `EnvironmentMismatch`, `InvalidSourceBindingContext`,
  `InvalidCheckerOwner`, `InvalidCoreContext`, and
  `InvalidItemAssociation`;
- `SourceModeCoreContextProducer::build(SourceBindingCoreContextHandoff,
  SourceBindingContextHandoff, SourceModeDefinitionHandoff) ->
  Result<SourceModeCoreContextHandoff, SourceModeCoreContextError>`.

All fields are private. The producer consumes inputs by value and publishes
only after complete postvalidation. It adds no generic definition adapter,
constructor, installer, compatibility layer, unchecked admission,
`CoreContextInput`/`CoreContext`/`CoreIr` field, or Typed/Resolved slot. It does
not alter 33LB or Tasks 33I259--261.

## Exact profile, identity, and provenance

- Source: existing 141-byte final-LF fixture, SHA-256
  `3271f243670bd781c7167ff0d3bf463263a318abbe261aabdde1842c532a725e`.
- Task 248: exact Profile B `1/2/2/2/2/2/0`; `SourceItemId(0)` is one normal
  `DefinitionBlock`, shell 0, site 50, range `0..140`, context/local-context 1,
  local scope `[0]`; module site is node 53; parameter sites are 37/41.
- Task 262: definitions/parameters/applications/expansions/inhabitation
  requests/properties are exactly `1/2/1/1/1/1`; Task-248 fingerprint equals
  `source_context.debug_text()`, Task-249+249M fingerprint is nonempty, and
  `base_initial_obligation_count()` is zero.
- Definition 0: typed id 0, whole Mode symbol, resolver definition 0,
  contribution 0, site 49, inner range `45..135`, ordinal 0, context 1, normal,
  exact spelling, application/expansion/request/property `0/0/0/Some(0)`, and
  local origin `[4,0,10,0]`.
- Parameters 0/1: owner 0, ordinals/bindings/written-type applications 0/1,
  sites 37/41, owner ranges `13..26`/`29..42`, declaration ranges
  `17..18`/`33..34`, pattern ranges `86..87`/`89..90`, context 1, exact
  spellings, and normal recovery.
- Application 0: owner/ordinal 0, ordered parameter vector `[0,1]`, site 42,
  range `73..91`, context 1, spelling `Task262Mode [ x , y ]`.
- Expansion 0: owner/ordinal/RHS `0/0/0`, site 44, range `95..98`, context 1,
  spelling `set`; request 0 links expansion 0 with kind `Rhs` and the same
  site/range/context/spelling.
- Property 0: owner/ordinal 0, kind `Sethood`, site 48, range `102..135`,
  justification `113..134`, exact spelling, normal recovery, and retained
  checker `InitialObligationId(0)`.
- Core: exactly one valid public `Mode` item, no dependencies, diagnostics,
  imports, generated origins, obligation seeds, or partial/recovered state;
  one pending `DefinitionalItem` boundary and one pending worklist entry.

The exact definition context link selects `SourceItemId(0)`. The association
table has one row keyed by typed `SourceModeDefinitionId(0)`. The Core item is
selected only through exact whole-`SymbolId` registry lookup. Checker,
resolver, and Core numeric ids are never reinterpreted; display names, FQN
alone, ranges, shell/seed order, maps, and worklists are not joins.

The item, item source-map row, boundary, and worklist use inner range
`45..135`, not outer `0..140`, with exactly one checker provenance key:
`source-mode-core-item-v1.definition.0`. `Valid` authenticates only the item
shell. The RHS, inhabitation request, `sethood` property, computation
justification, and pending obligation stay checker-owned and are not lowered,
proved, activated, or converted to Core obligations here.

## Default-deny oracle

Validation rejects without sorting, repair, inference, recovery, unchecked
admission, or partial publication:

1. any source/module mismatch or unequal retained Task-248/33LB `BindingEnv`;
2. any nonexact Task-248 item, shell, declaration, binding, context,
   local-context, link, site, range, role, order, ownership, recovery, or
   diagnostic state;
3. nonexact Task-262 cardinality, base count, source-context fingerprint,
   empty source-type fingerprint, resolver identity/origin, or any definition,
   parameter, application, expansion, request, or property row;
4. missing/`None`/foreign/mismatched context link or source item;
5. missing/extra/duplicate/reordered/stale/mismatched/orphan association rows;
6. missing/extra Core items or wrong symbol, kind, visibility, status, inner
   source, provenance, source-map domain, dependency, diagnostic,
   generated-origin, boundary, or worklist state;
7. name-, range-, numeric-, seed-, map-, or worklist-order joins.

The source-type handoff and initial-obligation table are not producer inputs.
Their exact construction is the checker Task-262 trust boundary; this producer
validates the retained fingerprint/base count/property obligation reference
without inventing a second lower-stage slot or canonical payload.

## Installation boundary and deferrals

Only the existing private Task-262 real-source test leaf may derive the one
Core `Mode` seed from the authenticated definition, prepare the Core context,
apply 33LB to the retained complete `BindingEnv`, and call the standalone
producer. Exactly two new private tests verify retained inputs, association,
item/source-map/boundary/worklist state, deterministic replay, ten Core
mutations, and four foreign-environment combinations.

There is no production runner branch or installation into Typed, Resolved,
CoreContext, or CoreIr. No `.miz`, expectation, trace, active result,
diagnostic, metadata count, or coverage state changes. Task 263/264, generic
Core-33 inventory, Core 34 mode/RHS/type/inhabitation/sethood semantics, Core
35 formula semantics, Core 36 definition body/correctness/obligation lowering,
mode applications/redefinitions, proof/discharge/acceptance, `GeneratedOrigin`,
C4C8 composition, snapshots, `MT10-CIR-TE`, diagnostics, and Task277B remain
deferred. Task 262 earns zero Core credit.

## Artifacts, baselines, reviews, and exit

Source changes are exactly:

1. `crates/mizar-core/src/elaborator.rs`;
2. `crates/mizar-test/src/runner/tests/type_elaboration/source_mode_definition.rs`.

Derived docs are this paired contract; paired Core plan, decomposition, TODO,
elaborator, source/spec audit, bilingual audit, and ledger; paired mizar-test
harness and bilingual audit; and `doc/design/spec_coverage_audit.md`. Checker
docs remain unchanged. The central audit records zero-credit mapping and
follow-up ownership only.

At freeze, `elaborator.rs` is `21540 / 805739`, SHA-256
`68d9623412dc1f1186ded06eff762d498e6d5b5431eca0f018bcc55df28ea07a`;
the Task-262 test leaf is `1242 / 45711`, SHA-256
`7ae8f4d7cd6805d85afe92380cd4fc702bfafc7124ee01f3283e36e460b2b798`.
Task-contract trees are `113/113 -> 114/114`. Core library tests stay `163`;
mizar-test library tests project `638 -> 640`; metadata stays `137`.

Protected values include the Task-262 source/expectation hashes
`3271f243670bd781c7167ff0d3bf463263a318abbe261aabdde1842c532a725e` /
`046b5a686600f78e1598c515c05f8124ec19edef56a14385a2d05bced527601e`,
all prior protected hashes, trace
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`,
and stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4`.

Entry `HEAD` is `4c6ecafc2a9bee7a4eb6e3f27336733fc672bd57`;
`origin/main` is `de42b58f7322128566326c8ee1d3d1e9a5fe4d77` with divergence
`0/2`. The original requested baseline mismatch remains report-only
`repo_metadata_conflict`; no fetch, push, stash mutation, or metadata repair
is authorized.

Before source edits, independent specification/equivalence and bilingual/
boundary reviews must end with no findings. After source edits, independent
test-sufficiency, implementation, and source/documentation/API reviews must
end with no findings after repair. Verification runs the Task-262 probes,
checker route, protected 33LB/259--261 probes, lint/metadata, fmt, offline
metadata, full warnings-denied Clippy, and all-feature tests including
doctests. Exit requires hard gates `9/9`, parent score `>=90/100`, exact
task-only commit, clean postcommit proof, protected invariance, Task277B
not-ready/zero-credit, and fresh successor inventory.

## Completion evidence

The standalone producer and exactly two private Task-262 tests are complete.
Final source measurements are `elaborator.rs` `22350 / 839135`, SHA-256
`3fe6e32d621f6516b54a67fd7649e6504b619c3e5e570ed26143060b5e849510`, and the
Task-262 test leaf `1637 / 60702`, SHA-256
`87355decdec7f657bbe421190428b4aa4fd0e47e1420df3962e6063584644bc5`.
The paired task-contract trees are exactly `114/114`; Core library tests are
`163`, mizar-test library tests are `640` (`638 + 2`), and metadata tests are
`137`.

The pre-source specification/equivalence and bilingual/boundary reviews had no
findings. Post-source test-sufficiency, implementation, and
source/documentation/API reviews ended with no findings after repairing the
public API inventory and status drift; a Core-lint link failure was separately
repaired by adding the Japanese owning-plan link.

Focused Task-262 tests pass `2/2`; the Task-262 route passes `6/6`; protected
Task-259--262 item-context probes pass `8/8`; Core tests and integration/lint
pass `163/163`; mizar-test passes `640/640`; Core lint passes `12/12`;
mizar-test lint passes `15/15`; metadata passes `137/137`. Formatting, offline
metadata, full warnings-denied Clippy, and `cargo test --all-features`, including
integration tests and doctests, pass. `git diff --check` passes. Protected
Task-262 source/expectation/trace hashes match the frozen values, and the
protected stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4` is unchanged.

Parent review passes all hard gates `9/9` with an uncapped quality score of
`98/100`: specification `20/20`, test contract `20/20`, traceability `15/15`,
implementation `15/15`, design/source synchronization `10/10`, boundary
discipline `10/10`, verification `5/5`, and handoff `3/5`. Task 262 remains zero
Core credit. Core
34--36 mode/RHS/type/inhabitation/sethood semantics, `GeneratedOrigin`,
production installation, `MT10-CIR-TE`, diagnostics, coverage credit, and
Task277B remain deferred/not-ready. The exact task-only commit and fresh
postcommit successor inventory remain pending.
