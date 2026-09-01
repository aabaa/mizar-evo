# Task CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-SEEDS-35E264: Task264 equals selector seeds

> Canonical language: English. Japanese companion:
> [../ja/CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-SEEDS-35E264.md](../ja/CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-SEEDS-35E264.md).

Status: complete for the task-only commit. Independent reviews, verification,
hard gates `9/9`, and valid uncapped quality `100/100` are complete. This is a
standalone, representation-only, zero-semantic, zero-credit Core-35 input
prerequisite.

## Identity, authority, and classification

| Field | Frozen value |
|---|---|
| Task | `CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-SEEDS-35E264` |
| Primary owner | `mizar-core::elaborator`, Core Task 35 input |
| Required predecessors | Task264D, Task33P264, IR264 |
| Inputs | Complete `SourcePropertyParameterCoreContextHandoff` and `SourcePropertyEqualsSelectorIdentityHandoff` |
| Consumer | Separately reviewed property-owner-aware Core35 lowering, then Core36 |
| Owning plan | [`mizar-core` Task Index](../../mizar-core/en/00.crate_plan.md#task-index) |
| Coverage | Zero semantic, execution, trace, metadata, and coverage credit |

Authority remains `doc/spec/en/`, existing `.miz`, trace metadata,
expectations, design, then source. Chapters 7 and 13 plus the protected equals
fixture fix a direct term definiens `M.carrier`. Task264D authenticates its
whole field `SymbolId`, exact source graph, and binding 0; Task33P264 maps that
binding to free term `CoreVarId(0)`; IR264 supplies the non-item property
`CoreDefinitionOwner`. There is no `spec_gap`.

The absent joint Core seed graph is bounded `design_drift` and `source_drift`;
two private assertions are a bounded `test_gap`. Direct lowering is not ready:
`TermAndFormulaLoweringInput` still requires a `CoreItemId`, while the property
owner deliberately has no item. This task must not substitute carrier item 0
as the definition owner or publish `CoreTermId`s.

## Frozen public API and exact representation

Add exactly this private-field public surface beside Task33P264:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyEqualsSelectorTermSeedAssociation { /* private */ }

impl SourcePropertyEqualsSelectorTermSeedAssociation {
    pub const fn parameter(&self) -> SourcePropertyParameterId;
    pub const fn binding(&self) -> BindingId;
    pub const fn core_var(&self) -> CoreVarId;
    pub const fn source_base(&self) -> SourcePrimaryTermId;
    pub const fn base_seed(&self) -> CoreTermSeedId;
    pub const fn source_selector(&self) -> SourceStructureTermId;
    pub const fn selector_seed(&self) -> CoreTermSeedId;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyEqualsSelectorTermSeedHandoff { /* private */ }

impl SourcePropertyEqualsSelectorTermSeedHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub const fn definition_owner(&self) -> &CoreDefinitionOwner;
    pub const fn parameter_context(&self)
        -> &SourcePropertyParameterCoreContextHandoff;
    pub const fn selector_identity(&self)
        -> &SourcePropertyEqualsSelectorIdentityHandoff;
    pub fn terms(&self) -> &[CoreTermSeed];
    pub const fn association(&self)
        -> &SourcePropertyEqualsSelectorTermSeedAssociation;
    pub fn debug_text(&self) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePropertyEqualsSelectorTermSeedError {
    EnvironmentMismatch,
    InvalidParameterContext,
    InvalidSelectorIdentity,
    InvalidDefinitionOwner,
    InvalidTermSeeds,
}

#[derive(Debug, Clone, Copy)]
pub struct SourcePropertyEqualsSelectorTermSeedProducer;

impl SourcePropertyEqualsSelectorTermSeedProducer {
    pub fn build(
        parameter_context: SourcePropertyParameterCoreContextHandoff,
        selector_identity: SourcePropertyEqualsSelectorIdentityHandoff,
    ) -> Result<
        SourcePropertyEqualsSelectorTermSeedHandoff,
        SourcePropertyEqualsSelectorTermSeedError,
    >;
}
```

The handoff retains both complete inputs, the Task34I264-derived property
definition owner, exactly two ordered `CoreTermSeed` rows, and one association
by value. The rows are exactly:

1. seed 0: `Var(CoreVarId(0))`, direct source range `173..174`, provenance
   phase `Checker` and key
   `source-property-equals-selector-term-seed-v1.base`;
2. seed 1: `Select { selector: <Task264D whole carrier-field SymbolId>, base:
   CoreTermSeedId(0) }`, direct source range `173..182`, provenance phase
   `Checker` and key
   `source-property-equals-selector-term-seed-v1.selector`.

The direct `CoreSourceRef` itself has no embedded provenance; checker-owned
provenance remains in `CoreTermSeed::provenance` for the later lowerer to merge.
The exact association is parameter/binding/Core-variable `0/0/0`, source base
0 to seed 0, and source selector 0 to seed 1. Seed ids are local dense graph
coordinates, not published `CoreTermId`s.

`debug_text()` is exactly
`source-property-equals-selector-term-seeds-v1|module=<package>.<path>|owner-anchor=0|property=<property-fqn>|selector=<field-fqn>|source=0:0|seed=0:1|parameter=0:0:0`
with no final LF.

## Validation and default-deny order

Validation order is environment, parameter context, selector identity,
definition owner, term seeds, followed by complete postvalidation.

- Inputs share exact source/module and the complete property handoff.
- Task33P264 replays unchanged and exposes only association `0/0/0` and one
  free term variable with no binder frame or type facts.
- Task264D remains equals-only and its exact `0/0/0/0/0/0/0/0/0` association,
  fingerprints, normal primary/reference/structure/member/edge/request rows,
  source sites/ranges, binding 0, and whole field symbol are reproduced.
- The retained definition owner equals
  `parameter_context.selector_context().definition_owner()`, has anchor item 0,
  no ordinary item, and the sole authenticated `marker` property symbol.
- The two rows, source anchors, checker provenance, local seed references, and
  association reproduce both retained receipts exactly.

Means, mixed/foreign transactions, stale fingerprints, wrong/nonzero ids,
different owners or symbols, extra/missing/reordered seeds, spelling-only
identity, and malformed source/provenance fail before publication. Private
fields and retained branded handoffs are the integrity boundary.

## Scope, tests, artifacts, and exit

Rust edits are exactly `crates/mizar-core/src/elaborator.rs` and
`crates/mizar-test/src/runner/tests/type_elaboration/source_property_implementation.rs`.
Add two private tests: deterministic exact seed graph and
mixed/foreign/default-deny rejection. Core unit count remains 164; mizar-test
projects `648 -> 650`.

Do not call `lower_term_and_formula_inputs`, construct
`TermAndFormulaLoweringInput`/`TermAndFormulaLoweringOutput`, substitute the
carrier item as property owner, or add `CoreTermId`, `CoreTermTable`,
`CoreSourceMap`, formula, field/type association, normalized type/fact/guard,
`it`, definition/body, correctness/coherence, diagnostic, obligation,
production route, snapshot, or coverage credit. Do not edit `doc/spec`, `.miz`,
expectations, trace metadata, checker, `core_ir.rs`, or VC.

Durable owners are this paired contract; paired Core plan, elaborator,
decomposition, TODO, source/spec, bilingual, and boundary records; paired
mizar-test harness/bilingual/boundary records; and the central coverage audit.
Stable owner sections are the Core [Task35E264 API](../../mizar-core/en/elaborator.md#task-35e264-task264-equals-selector-term-seeds),
[decomposition entry](../../mizar-core/en/source_family_decomposition.md#task-35e264-task264-equals-selector-seed-input),
and mizar-test [private probe](../../mizar-test/en/harness.md#core-task-35e264-private-task264-equals-seed-probe).
Audit impact is corrected Task264 readiness/follow-up ownership only, with zero
credit and unchanged `430/396/0/23` coverage-plan counts.

Clean baseline is HEAD `3d72789d1344df89bd17908415e10f550e1d2fc6`.
`elaborator.rs` is `24256 / 912957`, SHA-256
`d48f63f0a8427bc6e9c4290affcc8ae5e909c8b2af5391398256894d1334ff11`;
the private Task264 leaf is `1377 / 58827`, SHA-256
`b7b443a4638ce10216b6ffc1f72c0324dd8806ab5a33f2d5c8b8e77ac788afc4`;
the central audit is `7459 / 563428`, SHA-256
`085f0836ac2a0047eb59d058aa93310e3c7f750fe090fbc230b32ca01e87b9cf`.
Contract trees project `123/123 -> 124/124`; Core/checker/mizar-test unit
counts begin at `164/582/648`. Protected hashes and stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` remain unchanged.

The implemented source measures `24788 / 934127` for `elaborator.rs`, SHA-256
`10bde6f70141a7848e73278b23f3d66c866d158acbee65b6bab3093e7b5210d2`,
and `1633 / 68263` for the private Task264 leaf, SHA-256
`bd320b1ec77859417b13708412e5e44b5030609b6632773388655a1be57ef9ee`.
Contract trees measure `124/124`. The final central audit measures
`7474 / 564328`, SHA-256
`16a1f0fce5b0ec82f706f81a34154c24dec6e4a13d8022ce3052d513b997cb67`.

Independent pre/post-source reviews, all required verification, hard gates
`9/9`, quality `>=90/100`, task-only commit, clean postcommit proof, and fresh
inventory are required. Next work is property-owner-aware Core35 lowering for
this graph; means `it`, Core36, and Task277B remain separate/not-ready.

## Completion evidence

Independent pre-source specification/API and bilingual/boundary reviews ended
with no blocking/high/medium finding. Post-source implementation/default-deny
and test-sufficiency reviews ended with no finding. The source/documentation/
API/bilingual/boundary review found three medium lifecycle, gap-status, and
line-count findings; synchronized EN/JA status, closed-gap records, public API
inventory, and current boundary measurements resolved them, and its
finding-specific re-review reported no remaining finding.

Focused Task35E264 tests pass `2/2`, the complete Task264 private family passes
`12/12`, Core passes `164` unit, `2` determinism, and `12` lint tests, and
mizar-test passes `650` unit, `3` layout, `15` lint, `137` metadata, `2`
public-enum, and `21` snapshot tests. Formatting, offline metadata,
`git diff --check`, warnings-denied all-target/all-feature Clippy, and the
enlarged-stack all-feature workspace tests/doctests pass. The coverage plan is
unchanged at `430` cases, `396` requirements, zero errors, and `23` warnings.

Protected means/equals `.miz` hashes remain `cc90659f...`/`175135aa...`, their
expectations remain `bced7730...`/`c491d7ea...`, trace metadata remains
`17bba212...`, and protected stash remains `f65cf4a...`. No specification,
expectation, trace, snapshot, diagnostic, obligation, route, or semantic credit
changed. Parent and independent hard gates pass `9/9`. The final independent
read-only audit reported no finding, applied no score cap, and assigned valid
`100/100` (`20/20/15/15/10/10/5/5`). Exact staging and the task-only commit are
prepared; clean postcommit proof and fresh successor inventory belong to the
operational handoff rather than this self-referential task commit.
