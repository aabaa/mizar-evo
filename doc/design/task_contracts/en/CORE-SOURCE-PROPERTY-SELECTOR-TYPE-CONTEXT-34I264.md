# Task CORE-SOURCE-PROPERTY-SELECTOR-TYPE-CONTEXT-34I264: Task264 selector/type context

> Canonical language: English. Japanese companion:
> [../ja/CORE-SOURCE-PROPERTY-SELECTOR-TYPE-CONTEXT-34I264.md](../ja/CORE-SOURCE-PROPERTY-SELECTOR-TYPE-CONTEXT-34I264.md).

Status: complete on task commit. This is a representation-only,
zero-semantic, zero-credit Core-34 prerequisite. It changes no language
behavior, test intent, diagnostic, obligation, trace, metadata, or coverage
credit.

## Identity, authority, and readiness

| Field | Frozen value |
|---|---|
| Task | `CORE-SOURCE-PROPERTY-SELECTOR-TYPE-CONTEXT-34I264` |
| Primary owner | `mizar-core::elaborator`, Core Task 34 |
| Required predecessors | Task264C `3cb1b31c`; Task33I264 `0f61a860` |
| Inputs | Complete `SourcePropertyCarrierCoreContextHandoff` and exact `SourceTypeApplicationHandoff` |
| Consumer | A separately reviewed CoreIR definition-owner prerequisite, then Core35/36 |
| Owning plan | [mizar-core Task Index](../../mizar-core/en/00.crate_plan.md#task-index) |
| Coverage | Zero semantic, execution, trace, metadata, and coverage credit |

Authority remains `doc/spec/en/`, existing `.miz`, trace metadata,
expectations, design, then source. Chapter 5 makes `carrier` a field and
`marker` a virtual property, gives both selector/function identity and type
guards, and excludes properties from constructor arguments. Chapter 7 makes
the implementation target the existing property over its declared carrier
domain. Existing Task264 means/equals sources fix the exact same-source
carrier, two members, parameter type, and property return row.

There is no `spec_gap`. Task264C and Task33I264 provide the complete identity
and carrier/Core prerequisites. The missing authenticated property-target-to-
return-member association is bounded `design_drift` and `test_gap`. A separate derived
`design_drift` remains: current `CoreDefinition.item` cannot own the property
selector without a forbidden selector alias or a reviewed owner-model change.
This task deliberately does not claim to resolve that later prerequisite.

## Frozen public API

Add exactly this public surface to `mizar-core::elaborator`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertySelectorTypeAssociation { /* private fields */ }

impl SourcePropertySelectorTypeAssociation {
    pub const fn symbol(&self) -> &SymbolId;
    pub const fn member_type(&self) -> SourceTypeStructureMemberId;
    pub const fn root(&self) -> SourceTypeExpressionId;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertySelectorTypeContextHandoff { /* private fields */ }

impl SourcePropertySelectorTypeContextHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub const fn carrier_context(&self)
        -> &SourcePropertyCarrierCoreContextHandoff;
    pub const fn source_type(&self) -> &SourceTypeApplicationHandoff;
    pub const fn carrier_item(&self) -> CoreItemId;
    pub const fn association(&self) -> &SourcePropertySelectorTypeAssociation;
    pub fn debug_text(&self) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePropertySelectorTypeContextError {
    EnvironmentMismatch,
    InvalidCarrierContext,
    InvalidSourceType,
    InvalidAssociation,
}

#[derive(Debug, Clone, Copy)]
pub struct SourcePropertySelectorTypeContextProducer;

impl SourcePropertySelectorTypeContextProducer {
    pub fn build(
        carrier_context: SourcePropertyCarrierCoreContextHandoff,
        source_type: SourceTypeApplicationHandoff,
    ) -> Result<
        SourcePropertySelectorTypeContextHandoff,
        SourcePropertySelectorTypeContextError,
    >;
}
```

The handoff retains both complete immutable inputs by value and derives exactly
one private property-selector association. It does not publish an association
table type, a `CoreDefinition` owner, or a normalized Core type.
Definition and contribution identities remain typed in the retained checker
owner and are fully validated, but are not redundantly exposed through the
association; this keeps resolver-environment types outside the Core API.

The exact no-final-LF debug value is:

```text
source-property-selector-type-context-v1|module=<package>.<path>|carrier-item=<id>|property=<fqn>:2:0:1:2
```

Each suffix is `definition:contribution:member-type:root`.

## Exact validation oracle

Validation order is environment, retained carrier context, source-type
handoff, then derived associations. Source/module must agree across both
inputs. The carrier context must still pass its complete Task33I264
postvalidation. The source-type debug bytes must equal the checker owner's
retained fingerprint, but that comparison is replay evidence only: exact
typed rows below are the authority and are never reconstructed from the text.

The source-type profile is exactly applications/expressions/arguments/
definition-returns/mode-RHS/structure-members `1/3/0/0/0/2`. Application 0 is
binding 0, ordinal 0, root expression 0. Expression 0 is the bare whole
`Task264Carrier` symbol with contribution 0 at range `130..144`; expressions
1 and 2 are bare builtin `set` at `62..65` and `90..93`. Structure-member rows
0 and 1 have ordinals 0/1, ranges `45..66`/`71..94`, and roots 1/2. Exact
means typed-node coordinates are application/head `63/64`, expression/head
`55/54` and `58/57`, member `56/59`; equals coordinates are `45/46`, `37/36`
and `40/39`, member `38/41`. All rows are normal and retain their exact
spellings.

The checker parameter is row 0 with written type 0 and the carrier binding;
target row 0 is the retained `marker` property identity and explicitly has
return type member 1. The sole derived association follows that typed target
edge to member 1/root 2. The lower handoffs do not authenticate a corresponding
`carrier` field-to-member-0 edge, so this task deliberately publishes no field
association. The property symbol remains in the handoff module and the carrier
item remains exactly the Task33I264 item. Names, FQN alone, ranges, numeric ids,
debug fingerprints, and iteration order are never joins.

## Tests, scope, and forbidden behavior

Source edits are exactly `crates/mizar-core/src/elaborator.rs` and the existing
private Task264 assertion leaf. Add exactly two tests: (1) means/equals
positive construction, deterministic replay, retained inputs, the exact
property association, all exact source-type rows, and debug; (2) a
same-environment means/equals
cross-profile transaction reaches source-type validation and returns
`InvalidSourceType`, foreign carrier/type transactions return
`EnvironmentMismatch`, and valid transactions remain isolated. The lower
checker producers' private immutable construction is trusted for row-level
mutation safety; no public test-only mutator is added. Production runner
selection is unchanged.

Do not add a shell/member/property `CoreItem`, `CoreItemKind`, selector alias,
dependency, `CoreTypePredicate`, normalized type, binder, type fact, coercion,
view, term, formula, definition, diagnostic, obligation, installer, production
route, snapshot, or coverage credit. Do not lower means/equals bodies,
correctness, coherence, property values, facts, proofs, acceptance, or
discharge. Task263 is not an input. `doc/spec`, `.miz`, expectations, trace
status/backlinks, metadata, and protected artifacts remain unchanged.

## Artifacts, reviews, verification, and handoff

Derived owners are this paired contract; paired Core plan, elaborator,
source/spec, decomposition, TODO, bilingual, module-boundary, and ledger docs;
paired mizar-test harness, bilingual, and module-boundary docs; and the central
coverage audit. Checker source/design and `core_ir.rs`/`core_ir.md` are
read-only dependencies.

Stable owner sections are the
[elaborator API/invariants](../../mizar-core/en/elaborator.md#task-34i264-task264-selectortype-context),
[source/spec disposition](../../mizar-core/en/source_spec_audit.md#core-34-task264-selectortype-context-mapping),
and [private harness route](../../mizar-test/en/harness.md#core-task-34i264-private-task264-selectortype-probe).

Clean baseline is HEAD `0f61a86062707e2e6ec4c7ed611c03cc7b91ee00`.
`elaborator.rs` is `23335 / 877140`, SHA-256
`83d5884a24013345cb486d76b1df448a52ff860c89d9c491b201e70ba2eedd29`;
the private Task264 leaf is `699 / 31668`, SHA-256
`e45ac5bdbcbbab3fb0eeb4a281058dc2bad8330235db6590b432b76cb69c3d48`;
the central audit is `7373 / 558238`, SHA-256
`4888efe82c6900faaf132918c55f085449415e86a36be2476b39fed88e450240`.
Contract trees project `118/118 -> 119/119`; Core tests remain `163`, and
mizar-test library tests project `644 -> 646`.

Final measured artifacts are `elaborator.rs` `23682 / 890332`, SHA-256
`a91e2456c279ffec9a2f67a18d9741f8885228c5a31d600e41622fcd1e03bfb9`;
the private Task264 leaf `1017 / 44370`, SHA-256
`23ad08e3ac46e36ee34121cee49873b90f796fd50a18ad632aeca032598e79b6`;
and the central audit `7394 / 559472`, SHA-256
`84707772a1bee9acb4a8e713252db848f0aea2421c4f08937751c835d680f749`.
Contract trees are `119/119`; Core/checker/mizar-test library tests are
`163/580/646`.

Independent pre-source specification/API and bilingual/boundary reviews ended
with no findings after repair. Post-source test-sufficiency,
implementation/default-deny, source/documentation/API, and bilingual/boundary
reviews also ended with no blocking findings after the unauthenticated field
join was removed. Focused tests passed `2/2`; Core/checker/mizar-test package
tests and their lint/metadata suites passed; `cargo fmt --all -- --check`,
warnings-denied all-target/all-feature Clippy, offline metadata, and the
all-feature workspace test/doctest gate passed. Protected artifacts, checker
owners, `core_ir.rs`, and the protected stash remained invariant. Independent
final read-only review passed hard gates `9/9` with no blocking/high/medium
finding and assigned a valid uncapped quality score of `100/100`. Exact
task-only commit and clean postcommit proof close the task.

Fresh successor inventory must select the smallest fail-closed CoreIR
definition-owner representation for a structure member without a synthetic
selector item or alias. Only after that prerequisite may Core35/36 claim a
`CoreDefinition`-compatible property owner. Task277B remains not-ready.
