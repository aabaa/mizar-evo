# Task CORE-STRUCTURE-PROPERTY-DEFINITION-OWNER-IR264: authenticated structure-property definition owner

> Canonical language: English. Japanese companion:
> [../ja/CORE-STRUCTURE-PROPERTY-DEFINITION-OWNER-IR264.md](../ja/CORE-STRUCTURE-PROPERTY-DEFINITION-OWNER-IR264.md).

Status: complete on the task-only commit. This is a representation and
validation-only, zero-semantic, zero-credit CoreIR prerequisite. It changes no
language behavior, `.miz`/expectation/trace test intent, diagnostic,
proof/acceptance policy, metadata, or coverage credit. It intentionally adds
derived Rust validation-test intent for the selected owner representation.

## Identity, authority, and readiness

| Field | Frozen value |
|---|---|
| Task | `CORE-STRUCTURE-PROPERTY-DEFINITION-OWNER-IR264` |
| Primary owners | `mizar-core::core_ir` and the authenticated Task34I264 adapter in `mizar-core::elaborator` |
| Required predecessors | Task264C `3cb1b31c`; Task33I264 `0f61a860`; Task34I264 `85648a07` |
| Consumer | A later Core35/36 Task264 property-definition task |
| Owning plan | [mizar-core Task Index](../../mizar-core/en/00.crate_plan.md#task-index) |
| Coverage | Zero semantic, execution, trace, metadata, and coverage credit |

Authority remains `doc/spec/en/`, existing `.miz`, trace metadata,
expectations, design, then source. Chapter 5 defines a structure property as a
function symbol with a declared carrier/type guard and excludes it from
constructor arguments. Chapter 7 makes the implementation target that existing
property over its declared carrier domain. Those authorities do not prescribe
a CoreIR owner representation. There is no `spec_gap` or source contradiction.

The active discrepancy is derived `design_drift`: `CoreDefinition.item` can
name only a module-level `CoreItem`, while Task34I264 authenticates the existing
`marker` property selector under the `Task264Carrier` structure item and
explicitly authenticates no `carrier` field edge. Several derived
representations would be possible. Under the autonomous-design rule, this
contract selects the smallest fail-closed design that preserves ordinary item
ownership and lets only the authenticated adapter mint the new property form.
The missing Rust validation matrix is a bounded `test_gap` closed here without
new `.miz` intent.

## Frozen public and crate-private API

In `mizar-core::core_ir`, replace `CoreDefinition.item` with this owner value:

```rust
#[derive(Clone, PartialEq, Eq)]
pub struct CoreDefinitionOwner { /* private fields */ }

impl CoreDefinitionOwner {
    pub const fn for_item(item: CoreItemId) -> Self;
    pub const fn anchor_item(&self) -> CoreItemId;
    pub const fn item(&self) -> Option<CoreItemId>;
    pub const fn property_symbol(&self) -> Option<&SymbolId>;
}

pub struct CoreDefinition {
    pub owner: CoreDefinitionOwner,
    // every other existing field is unchanged
}
```

The owner privately stores the anchor item and, only for the structure-property
form, the authenticated source id, module id, and property symbol. `for_item`
is the only externally or crate-wide callable constructor and preserves every
existing item-owned definition. The structure-property initializer is private
to `core_ir.rs`; there is no public or crate-private generic member/field
constructor and no public mutator.

In `core_ir.rs`, add exactly this inherent method to the already immutable
Task34I264 handoff type:

```rust
impl SourcePropertySelectorTypeContextHandoff {
    pub fn definition_owner(&self) -> CoreDefinitionOwner;
}
```

Because the inherent implementation lives in the module that owns the private
`CoreDefinitionOwner` fields, it is the sole non-test property initializer. It
copies only `source_id`, `module_id`, `carrier_item`, and the sole validated
`marker` association symbol. It neither reconstructs nor accepts these values
from a caller. Other `mizar-core` modules cannot mint the property form.

Add one `CoreIrError` variant:

```rust
InvalidDefinitionOwner {
    definition: CoreDefinitionId,
    reason: &'static str,
}
```

The exact reasons are `property-anchor-not-valid-structure`,
`property-symbol-mismatch`, `property-environment-mismatch`, and
`property-symbol-aliases-anchor`. Invalid anchor indexes retain the existing
`InvalidReference { table: "item", ... }` error.

## Validation, compatibility, and debug invariants

For every definition, source/source-map validation remains first. Owner
validation then runs before binders, body, correctness seeds, and generated
dependencies:

1. Validate `anchor_item` against the item table.
2. For an ordinary item owner, stop owner validation and preserve current
   behavior exactly.
3. For a property owner, require a `Valid` `Structure` anchor.
4. Require the private property symbol to equal `CoreDefinition.symbol`.
5. Require the private source/module identity, anchor symbol module, and
   property symbol module to equal the enclosing `CoreIr` source/module.
6. Require the property symbol to differ from the anchor structure symbol.

The first failing check wins. Validation is atomic through `CoreIr::try_new`.
No owner identity is reconstructed from name, FQN alone, range, dense id,
source order, or debug text.

The existing Step-4 `DefinitionSeed.owner`, item-keyed `definition_map`,
obligation/proof/generated owners, definition IDs, source-map keys, unfolding
requests, VC consumers, and discharge behavior remain unchanged. Their four
direct `CoreDefinition` construction sites migrate mechanically to
`CoreDefinitionOwner::for_item`.

`core-ir-debug-v1` remains the internal test-facing grammar. Implement manual
`Debug` for `CoreDefinitionOwner` and `CoreDefinition`. Ordinary item-owned rows
remain byte-identical, including the legacy `item: CoreItemId(...)` field. A
property-owned row has exactly this standard `DebugStruct` grammar, with each
placeholder replaced by that value's existing `Debug` representation and the
remaining fields retaining their current order and representation:

```text
CoreDefinition { owner: StructureProperty { anchor_item: <CoreItemId>, source_id: <SourceId>, module_id: <ModuleId>, property_symbol: <SymbolId> }, symbol: <SymbolId>, params: <Vec<CoreBinder>>, body: <DefinitionBody>, expansion: <ExpansionPolicy>, correctness: <Vec<ObligationSeedId>>, generated_dependencies: <Vec<GeneratedOriginId>>, source: <CoreSourceRef> }
```

The direct `Debug` representation of an ordinary owner is
`Item(<CoreItemId>)`. Do not bump the debug version or change the active CoreIR
snapshot/expectations.

## Tests, scope, and forbidden behavior

Add exactly one CoreIR unit test covering a valid/replayed property owner with
no extra Core item plus invalid anchor id, non-Structure anchor, non-Valid
anchor, property/definition mismatch, foreign source/module, and selector equal
to anchor. Assert deterministic debug rendering and atomic rejection. Extend
the existing Step-4 positive test to prove it still emits an ordinary item
owner and retains its item-keyed map. Extend the existing Task264 positive
means/equals test to prove `definition_owner()` returns carrier item 0,
`item() == None`, and only the authenticated `marker`; existing cross-profile
and foreign tests remain the negative adapter oracle. Add no public test-only
constructor.

The CoreIR test may insert one non-production fixture definition solely to
validate this owner representation; it carries an existing trivial fixture
term body and grants no semantic or coverage credit. Otherwise do not create a
selector/member/property `CoreItem`, new item kind, selector alias, field
association, normalized type, binder, term, formula, source-derived/property
definition body, correctness/coherence obligation, diagnostic, production route,
snapshot, artifact, VC, proof, acceptance, or coverage credit. Do not extend
Core35/36 lowering, `DefinitionSeed`, `definition_map`, obligations, or
downstream owner types. Task263 is not an input. `doc/spec`, `.miz`, existing
expectations/snapshots, trace status/backlinks, metadata, and protected
artifacts remain unchanged.

## Artifacts, reviews, verification, and handoff

Source scope is exactly `crates/mizar-core/src/core_ir.rs`,
`crates/mizar-core/src/elaborator.rs`,
`crates/mizar-test/src/runner/tests/type_elaboration/source_property_implementation.rs`,
and the mechanical item-owner construction in
`crates/mizar-vc/src/generator/task180.rs`. Durable documentation owners are
this paired contract; paired mizar-core plan, CoreIR, elaborator,
source/spec/decomposition, TODO, ledger, bilingual, and boundary documents;
`doc/design/architecture/en/06.elaboration_and_core_ir.md` and its exact
`doc/design/architecture/ja/06.elaboration_and_core_ir.md` companion; the
paired mizar-test harness/bilingual/boundary records; and the central coverage
audit. The `mizar-vc` source edit is a mechanical existing-item-owner migration
and changes no VC-owned API, invariant, test intent, or documentation, so no
mizar-vc owner document changes. No checker, kernel, artifact, or Cargo owner
changes.

Completion replaces these current-state claims in place, in both languages:

- Core `elaborator.md` and mizar-test `harness.md` change “no definition owner”
  to “authenticated owner value only; no `CoreDefinition` row/body or semantic
  publication”;
- Core `source_spec_audit.md`, `source_family_decomposition.md`, `todo.md`, and
  `task_ledger.md` replace the pending owner-model prerequisite with this
  completed zero-credit owner value while retaining Core35/36 deferrals;
- Core bilingual/module-boundary audits record the public owner shape and
  final source inventory; and
- the central coverage audit advances follow-up ownership without changing
  trace or coverage status.

Stable owner sections will be
[CoreIR structure-property owners](../../mizar-core/en/core_ir.md#structure-property-definition-owners),
[Task264 authenticated owner factory](../../mizar-core/en/elaborator.md#task-ir264-authenticated-property-definition-owner),
and the existing [Task264 private probe](../../mizar-test/en/harness.md#core-task-34i264-private-task264-selectortype-probe).

Clean baseline is HEAD `85648a076ae40538dafabea93faaf63f7b516978`.
`core_ir.rs` is `4016 / 132375`, SHA-256
`4458bc2353c437d4427b39f96e0041bf1c321e19cff0ec4565c3f50084f83c4c`;
`elaborator.rs` is `23682 / 890332`, SHA-256
`a91e2456c279ffec9a2f67a18d9741f8885228c5a31d600e41622fcd1e03bfb9`;
the Task264 leaf is `1017 / 44370`, SHA-256
`23ad08e3ac46e36ee34121cee49873b90f796fd50a18ad632aeca032598e79b6`;
the VC Task180 source is `1323 / 50775`, SHA-256
`1e471e4058d091be83d865542d8d27467cc10fd09c6a2fc82ae80571d314436c`;
and the central audit is `7394 / 559472`, SHA-256
`84707772a1bee9acb4a8e713252db848f0aea2421c4f08937751c835d680f749`.
Contract trees project `119/119 -> 120/120`; Core library tests project
`163 -> 164`; mizar-test library tests remain `646`.

Independent specification/API and bilingual/boundary reviews precede source
editing. Test-sufficiency, implementation/default-deny, source/documentation,
bilingual/boundary, and final quality reviews follow. Exit requires focused
CoreIR/Step-4/Task264/VC probes, affected package tests and lint/metadata, fmt,
warnings-denied Clippy, offline metadata, all-feature workspace tests/doctests,
protected invariance, hard gates `9/9`, quality `>=90/100`, exact task-only
commit, and clean postcommit proof.

After this prerequisite, fresh inventory may define the smallest Core35
property domain/return-type inputs or a Core36 property body seed, but neither
is authorized by this task. Multiple implementations, field owners, property
value/correctness/coherence semantics, and Task277B remain deferred/not-ready.

## Completion evidence

Pre-source specification/API and bilingual/boundary reviews ended with no
findings after contract repair. Post-source test-sufficiency findings for
source-map precedence and independent environment operands were repaired and
re-reviewed with no remaining blocking/high/medium finding. Implementation,
source/documentation/API, bilingual/boundary, and final quality reviews ended
with no blocking/high/medium findings. The final independent result is hard
gates `9/9`, valid uncapped quality `100/100`.

Focused CoreIR owner, Step-4, Task264 means/equals `2/2`, and VC Task180 probes
pass. Package suites pass for mizar-core `164`, mizar-test `646` plus lint `15`
and metadata `137`, mizar-vc `105`, and mizar-checker `580` with its required
enlarged test stack. Formatting, offline metadata, warnings-denied all-target/
all-feature Clippy, and enlarged-stack all-feature workspace tests/doctests
pass. Contract trees are `120/120`; protected Task264 `.miz`, expectations,
trace, checker property/source-type inputs, and stash `f65cf4...` are unchanged;
both unstaged and staged diff checks are required to pass at commit.

Final measured artifacts are `core_ir.rs` `4393 / 146011`, SHA-256
`4e614a6ee98d0ef6b93dcd5d708728e41b79f613b16880269550051450793fd1`;
`elaborator.rs` `23685 / 890564`, SHA-256
`1d78d960032e2f4086f712d258a8ec247aa12daeff88f51c6afe8f4d880a7162`;
the Task264 leaf `1022 / 44699`, SHA-256
`e584e3a36d8c8911d4e5f49209128cb35e81d0c93d254419476b93557a86fdca`;
VC Task180 `1324 / 50836`, SHA-256
`1622fea0fdb24ac900ef22a9ac604ee5a45cb66a40eebaee0c540b600b71df61`;
and the central audit `7413 / 560700`, SHA-256
`a085dc14b0479cfab399ce5b594134b812b094b71d0885f948aa1ec1bea0f40a`.
