# Task CORE-SOURCE-PROPERTY-CARRIER-ITEM-CONTEXT-33I264: Task264 carrier item context

> Canonical language: English. Japanese companion:
> [../ja/CORE-SOURCE-PROPERTY-CARRIER-ITEM-CONTEXT-33I264.md](../ja/CORE-SOURCE-PROPERTY-CARRIER-ITEM-CONTEXT-33I264.md).

Status: complete pending the task-only commit. This is a representation-only,
zero-semantic, zero-credit Core-33 prerequisite. It changes no language
behavior, protected test intent, diagnostic, obligation, trace, metadata, or
coverage credit.

## Identity, authority, and readiness

| Field | Frozen value |
|---|---|
| Task | `CORE-SOURCE-PROPERTY-CARRIER-ITEM-CONTEXT-33I264` |
| Primary owner | `mizar-core::elaborator`, Core Task 33 |
| Owning plan | [`mizar-core` crate plan](../../mizar-core/en/00.crate_plan.md) |
| Required predecessor | Committed checker Task264C `3cb1b31c8727f244933c9750214101da333cf139` |
| Inputs | Prepared `CoreContext` plus exact Task264 `SourcePropertyImplementationHandoff` |
| Consumer | Core34 authenticated selector/type ownership, then Core35/36 |
| Coverage | Zero semantic, execution, trace, metadata, and coverage credit |

Stable owner links are the elaborator's
[Task264 carrier item API and invariants](../../mizar-core/en/elaborator.md#task-33i264-task264-carrier-item-context),
[source/specification boundary](../../mizar-core/en/source_spec_audit.md#task-33i264-sourcespecification-boundary),
and the mizar-test
[private probe boundary](../../mizar-test/en/harness.md#core-task-33i264-private-task264-carrier-probe).
This contract owns orchestration, exact task freeze, completion evidence, and
handoff; those sections own durable API, boundary, and test design.

Authority remains `doc/spec/en/`, existing `.miz`, trace metadata,
expectations, design, then source. Chapter 5 distinguishes the structure and
its field/property members. Chapter 7 §§7.4.1 and 7.8.2 make the property
implementation target the existing property over its declared carrier domain.
Chapters 11 and 12 preserve whole-symbol identity and source order. Committed
Task264C already authenticates the exact same-source `Task264Carrier`,
`carrier`, and `marker` resolver identities. The completed Task36P264
disposition remains authoritative that the context-only implementation shell
has no Core item.

There is no `spec_gap`. Task264C resolved the lower `source_drift`. The absent
carrier/Core association and private Core-context assertions are bounded
`design_drift` and `test_gap`. The stale current-state claim that the lower
carrier receipt is still missing is derived `design_drift` repaired by this
task's owner documents; completed historical contracts are not rewritten.
There is no `source_undocumented_behavior`, `test_expectation_drift`,
`boundary_violation`, or `repo_metadata_conflict`.

## Frozen API and representation

Add exactly this public surface to `mizar-core::elaborator`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyCarrierCoreContextHandoff { /* private fields */ }

impl SourcePropertyCarrierCoreContextHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub const fn context(&self) -> &CoreContext;
    pub const fn checker_owner(&self)
        -> &SourcePropertyImplementationHandoff;
    pub const fn carrier_item(&self) -> CoreItemId;
    pub fn debug_text(&self) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePropertyCarrierCoreContextError {
    EnvironmentMismatch,
    InvalidCheckerOwner,
    InvalidCoreContext,
    InvalidItemAssociation,
}

#[derive(Debug, Clone, Copy)]
pub struct SourcePropertyCarrierCoreContextProducer;

impl SourcePropertyCarrierCoreContextProducer {
    pub fn build(
        context: CoreContext,
        checker_owner: SourcePropertyImplementationHandoff,
    ) -> Result<
        SourcePropertyCarrierCoreContextHandoff,
        SourcePropertyCarrierCoreContextError,
    >;
}
```

The handoff retains the context and complete checker owner by value plus one
private `CoreItemId`. There is no public association row/table: the one scalar
`carrier_item()` and `checker_owner().carrier_identity()` form the exact
association without inventing a Core-owned source-definition ID. Fields are
private and publication follows complete postvalidation. Task263's two-row
structure API is a protected different-source precedent, not an input and not
generalized.

The exact no-blank-line debug value is:

```text
source-property-carrier-core-item-context-v1|module=<package>.<path>|carrier=<whole-fqn>:0:0|item=<core-item-id>
```

`<whole-fqn>` is `structure_symbol().fqn().as_str()`. Decimal IDs have no
padding. The string has no final LF and is non-authoritative.

## Exact input and default-deny oracle

The checker owner must retain one implementation/parameter/target/definiens,
and either the exact means correctness pair or no correctness rows for equals.
Required lower fingerprints are nonempty; optional structure/atomic-formula
fingerprints retain their existing profile separation. The three carrier
roles must have definition IDs `0/1/2`, contribution `0`, pairwise-distinct
whole symbols in the handoff module, normal local origins, and exact provenance:

| role | range | structural path |
|---|---|---|
| structure | `13..101` | `[4,0,11,0]` |
| field | `45..66` | `[4,0,11,0,18,0]` |
| property | `71..94` | `[4,0,11,0,19,1]` |

Target row 0 must equal the retained property symbol/definition/contribution/
origin. Core does not reauthenticate resolver spelling or the source-type head;
those are Task264C construction/replay guarantees protected by private fields.

The prepared context must match source/module and contain exactly one public,
valid `CoreItemKind::Structure` selected only by exact whole `SymbolId` lookup.
The item has the structure origin's `13..101` source range, no dependencies or
diagnostics, and sole checker provenance
`source-property-carrier-core-item-v1.structure`. The registry dependency row
is empty; source-map item ownership is exact; there is one pending
`DefinitionalItem` boundary and one pending item worklist entry. Binder state,
checker sites, dependency summaries, generated origins, diagnostics, other
Core items, terms, formulas, definitions, other source-map domains, external
or missing dependencies, and partial/recovered state are absent.

Validation order is environment, checker owner, Core context, association.
It rejects without sorting, repair, inference, unchecked admission, numeric-ID
reinterpretation, or partial publication. Names/FQN alone, ranges, source
order, seeds, maps, worklists, and Task263 are never joins.

## Tests, scope, and forbidden behavior

Source edits are exactly `crates/mizar-core/src/elaborator.rs` and the existing
private Task264 assertion leaf
`crates/mizar-test/src/runner/tests/type_elaboration/source_property_implementation.rs`.
Exactly two tests are added in that leaf: (1) means/equals positive construction,
deterministic replay, retained owner/scalar association, and exact Core item/
source-map/boundary/worklist assertions; (2) missing/extra/wrong Core shape,
dependency, provenance, boundary, binder/generated/checker-site mutations and
foreign context/owner rejection. The production runner route is unchanged.

No Core item is created for the property-implementation shell, `carrier`, or
`marker`; no selector alias, association, dependency, or `CoreItemKind` is
added. Field/property ownership remains Core34. Parameter/domain/type,
definiens, means/equals value, correctness/coherence, obligations, proof,
discharge, acceptance, facts, axioms, `CoreDefinition`, terms/formulas, CFG/VC,
diagnostics, production installation, Typed/Resolved/CoreIr slots,
`MT10-CIR-TE`, generic Core33, and Task277B remain deferred. Do not modify
`doc/spec`, `.miz`, expectations, trace backlinks/status, metadata, runner
selection, or coverage credit.

## Artifacts, audit impact, reviews, and exit

Derived owners are the paired contract; Core plan, elaborator, source/spec,
decomposition, TODO, bilingual, module-boundary, and completion ledger docs;
mizar-test harness, bilingual, and module-boundary docs; and the central
coverage audit. Checker docs and historical completed contracts remain
unchanged. The central audit records only the zero-credit mapping and advances
follow-up ownership from missing checker identity to Core34 selector/type
ownership; no chapter/trace/coverage status changes.

Clean baseline is HEAD `3cb1b31c8727f244933c9750214101da333cf139`.
`elaborator.rs` is `22947 / 862541`, SHA-256
`e9ea1d6eabb191d7d3b8c22fe1fc11626d2e0dab86690dee662f851bb487f85c`;
the Task264 test leaf is `258 / 13953`, SHA-256
`b5d86410fca9546872fb25ce644381284c97ad58f2e7f703319af99b14cd149a`;
contract trees project `117/117 -> 118/118`; Core library tests stay `163`,
and mizar-test library tests project `642 -> 644`. Protected Task264 source,
expectation, trace, and stash hashes remain those recorded by Task264C.

Independent specification/API and bilingual/boundary reviews must have no
blocking/high/medium findings before Rust edits. Then test-sufficiency,
implementation/default-deny, source/documentation/API, bilingual/boundary, and
final read-only quality reviews repeat after repairs. Exit requires focused
Task264/Core probes, Core/checker/mizar-test packages, lint/metadata, formatting,
warnings-denied Clippy, all-feature workspace tests including doctests,
protected invariance, hard gates `9/9`, quality `>=90/100`, exact task-only
commit, clean postcommit proof, and fresh Core34 inventory.

## Next handoff

After this task commits, fresh inventory may freeze only the smallest Core34
mapping from the retained `carrier`/`marker` identities plus existing Task264
type evidence to an authenticated selector/type owner compatible with
`CoreDefinition.item`. Core36 remains blocked until Core34 and applicable
Core35 inputs exist. No owner-model or semantic decision is implied here.

## Completion evidence

The frozen API and exact default-deny validation are implemented in
`elaborator.rs`, whose final size is `23335 / 877140` with SHA-256
`83d5884a24013345cb486d76b1df448a52ff860c89d9c491b201e70ba2eedd29`.
The existing private Task264 leaf grew to `699 / 31668` with SHA-256
`e45ac5bdbcbbab3fb0eeb4a281058dc2bad8330235db6590b432b76cb69c3d48`
and adds exactly the two frozen tests. Contract trees are `118/118`; Core
library tests remain `163`, and mizar-test library tests are `644`.

Independent pre-source specification/API and bilingual/boundary reviews,
post-source test-sufficiency and implementation/default-deny reviews, and
final source/documentation/API and bilingual/boundary reviews ended with no
blocking, high, or medium findings after the completion-state repair. Focused
Task264 tests pass `6/6`; checker Task264 tests pass `5/5`; Core, checker, and
mizar-test packages pass `163/580/644` library tests plus their integration,
lint, metadata, and doctests. Formatting, offline Cargo metadata, full
all-target/all-feature warnings-denied Clippy, and the all-feature workspace
including doctests pass. Metadata validation reports zero errors and the
unchanged 23 baseline warnings. Protected Task264 source/expectation/trace
hashes and protected stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4` are
unchanged. Final diff checks pass; no specification, `.miz`, expectation,
trace, metadata, production route, semantic behavior, or coverage credit
changed.

The final independent read-only review reports no blocking/high/medium
findings, hard gates `9/9`, and a valid uncapped quality score of `100/100`.
