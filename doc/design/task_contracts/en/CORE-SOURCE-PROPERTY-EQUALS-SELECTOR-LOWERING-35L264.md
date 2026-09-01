# Task CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-LOWERING-35L264: Task264 equals term lowering

> Canonical language: English. Japanese companion:
> [../ja/CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-LOWERING-35L264.md](../ja/CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-LOWERING-35L264.md).

Status: implemented and verified with hard gates `9/9` and valid independent
quality `100/100`. This is a specialized,
representation-only, unattached Core-35 lowering task with no language-behavior,
active-route, trace, metadata, or coverage-credit change.

## Identity, authority, and classification

| Field | Frozen value |
|---|---|
| Task | `CORE-SOURCE-PROPERTY-EQUALS-SELECTOR-LOWERING-35L264` |
| Primary owner | `mizar-core::elaborator`, Core Task 35 |
| Required predecessors | Task35E264, Task264D, Task33P264, IR264 |
| Input | One complete `SourcePropertyEqualsSelectorTermSeedHandoff` |
| Consumer | Separately reviewed Task264 Core36 property definition/body input |
| Owning plan | [`mizar-core` Task Index](../../mizar-core/en/00.crate_plan.md#task-index) |
| Coverage | Zero active execution, trace, metadata, and coverage credit |

Authority remains `doc/spec/en/`, existing `.miz`, trace metadata,
expectations, design, then source. Chapter 13 represents a selector application
as a term and the protected equals fixture fixes the direct term `M.carrier`.
Task35E264 already authenticates the exact `Var(0)` then `Select(field, seed0)`
graph, non-item property owner, source ranges, and provenance. There is no
`spec_gap` and no language or protected-test-intent decision.

The absent Task264-specific term table/source-map lowering is bounded
`design_drift` and `source_drift`; two private assertions are `test_gap`.
Changing generic `TermAndFormulaLoweringInput.owner` is not in scope: its
generated origins, diagnostics, and obligations require an ordinary
`CoreItemId` owner. This task must not invent property ownership for those
families or substitute carrier item 0.

## Frozen public API and exact representation

Add exactly this private-field public surface beside Task35E264:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyEqualsSelectorTermLoweringAssociation { /* private */ }

impl SourcePropertyEqualsSelectorTermLoweringAssociation {
    pub const fn base_seed(&self) -> CoreTermSeedId;
    pub const fn base_term(&self) -> CoreTermId;
    pub const fn selector_seed(&self) -> CoreTermSeedId;
    pub const fn selector_term(&self) -> CoreTermId;
    pub const fn root_term(&self) -> CoreTermId;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyEqualsSelectorTermLoweringHandoff { /* private */ }

impl SourcePropertyEqualsSelectorTermLoweringHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub const fn definition_owner(&self) -> &CoreDefinitionOwner;
    pub const fn seed_handoff(&self)
        -> &SourcePropertyEqualsSelectorTermSeedHandoff;
    pub const fn terms(&self) -> &CoreTermTable;
    pub const fn source_map(&self) -> &CoreSourceMap;
    pub const fn association(&self)
        -> &SourcePropertyEqualsSelectorTermLoweringAssociation;
    pub fn debug_text(&self) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePropertyEqualsSelectorTermLoweringError {
    InvalidSeedHandoff,
    InvalidTermLowering,
}

#[derive(Debug, Clone, Copy)]
pub struct SourcePropertyEqualsSelectorTermLoweringProducer;

impl SourcePropertyEqualsSelectorTermLoweringProducer {
    pub fn build(
        seed_handoff: SourcePropertyEqualsSelectorTermSeedHandoff,
    ) -> Result<
        SourcePropertyEqualsSelectorTermLoweringHandoff,
        SourcePropertyEqualsSelectorTermLoweringError,
    >;
}
```

The handoff retains the complete Task35E264 capability by value and delegates
its definition-owner getter; it does not duplicate or mint an owner. It owns
one local `CoreTermTable`, one term-only `CoreSourceMap`, and one association.
The rows are exactly:

1. local `CoreTermId(0)`: `CoreTermKind::Var(CoreVarId(0))`;
2. local `CoreTermId(1)`: `CoreTermKind::Select { selector: <Task264D whole
   carrier-field SymbolId>, base: CoreTermId(0) }`.

The association is seed 0 to term 0, seed 1 to term 1, and root term 1. These
term IDs are local dense coordinates, not globally installed `CoreIr` IDs.
The local root identifies the complete selector graph only; it is not yet a
definition body.

Each term source is the matching Task35E264 direct source with its checker-owned
seed provenance merged by the existing `source_with_provenance` rule. Thus term
0 has range `173..174` and key
`source-property-equals-selector-term-seed-v1.base`; term 1 has range
`173..182` and key
`source-property-equals-selector-term-seed-v1.selector`. Both phases are
`Checker`. `source_map.term_sources` contains exactly the same two sources; all
other source-map domains are empty.

`debug_text()` is exactly
`source-property-equals-selector-term-lowering-v1|module=<package>.<path>|owner-anchor=0|property=<property-fqn>|seed=0:1|term=0:1|root=1`
with no final LF.

## Validation and default-deny order

Validation order is seed handoff, definition owner, association, term table,
source map, followed by complete postvalidation.

- The complete Task35E264 input replays unchanged.
- The delegated owner equals the retained property owner, has anchor item 0,
  no ordinary item, and the sole authenticated `marker` property symbol.
- The association is exactly `0->0`, `1->1`, root 1.
- The two dense term rows reproduce both seed kinds and the whole selector
  symbol exactly; the selector base is term 0.
- Each normalized term source equals its term-source-map row and contains only
  the merged checker provenance above.
- The source map has exactly two term rows and no item, formula, definition,
  proof, algorithm, generated, obligation, or diagnostic ownership effect.

Private fields and the retained branded input are the integrity boundary.
Externally forged/reordered/extra terms or source rows are not constructible.
Independent valid transactions must remain source/module-local and must not
share or install tables.

## Scope, tests, artifacts, and exit

Rust edits are exactly `crates/mizar-core/src/elaborator.rs` and
`crates/mizar-test/src/runner/tests/type_elaboration/source_property_implementation.rs`.
Add two private tests: deterministic exact two-term lowering and unattached
foreign-transaction isolation. Core/checker unit counts remain `164/582`;
mizar-test projects `650 -> 652`, and the Task264 private family projects
`12 -> 14`.

Do not call or modify `lower_term_and_formula_inputs`, change
`TermAndFormulaLoweringInput`, attribute any output to carrier item 0, install
the local table into `CoreIr`, or add formula, definition row/body, field/type
association, normalized type/fact/guard, generated origin, diagnostic,
obligation, correctness/coherence, means `it`, production route, snapshot,
acceptance, fact, property value/axiom, or coverage credit. Do not edit
`doc/spec`, `.miz`, expectations, trace metadata, checker, `core_ir.rs`, VC,
Cargo manifests, or module topology. Task263 is not an input.

Durable owners are this paired contract; paired Core plan, elaborator,
decomposition, TODO, source/spec, bilingual, and boundary records; paired
mizar-test harness/bilingual/boundary records; and the central coverage audit.
Stable owner sections are the Core [Task35L264 API](../../mizar-core/en/elaborator.md#task-35l264-task264-equals-selector-term-lowering),
[decomposition entry](../../mizar-core/en/source_family_decomposition.md#task-35l264-task264-equals-selector-term-lowering),
and mizar-test [private probe](../../mizar-test/en/harness.md#core-task-35l264-private-task264-equals-term-lowering-probe).
Audit impact is closed bounded drift/test gaps and corrected Task264 follow-up
ownership only, with unchanged `430/396/0/23` coverage-plan counts and zero
credit.

Clean baseline is HEAD `112129671cfaefe5635676697baa3e9e028cb548`.
`elaborator.rs` is `24788 / 934127`, SHA-256
`10bde6f70141a7848e73278b23f3d66c866d158acbee65b6bab3093e7b5210d2`;
the private Task264 leaf is `1633 / 68263`, SHA-256
`bd320b1ec77859417b13708412e5e44b5030609b6632773388655a1be57ef9ee`;
the central audit is `7474 / 564328`, SHA-256
`16a1f0fce5b0ec82f706f81a34154c24dec6e4a13d8022ce3052d513b997cb67`.
Contract trees project `124/124 -> 125/125`. Protected hashes and stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` remain unchanged.

Independent pre/post-source reviews, all required verification, hard gates
`9/9`, quality `>=90/100`, task-only commit, clean postcommit proof, and fresh
inventory are required. Next work is a separately frozen Core36 property
definition/body input over this unattached root; means `it`, correctness/
coherence, production routing, and Task277B remain separate/not-ready.

## Implementation evidence

The specialized producer and two private tests are implemented in the exact
two Rust paths. Independent pre-source authority/API, implementability, and
bilingual/boundary reviews reported no finding. Post-source test-sufficiency,
implementation/default-deny, and source/documentation/bilingual/boundary
reviews reported no remaining finding after one stale measurement was fixed.

Focused Task35L264 assertions pass `2/2`, and the complete Task264 family passes
`14/14`. Core passes `164` unit, `2` determinism, and `12` lint tests. mizar-test
passes `652` unit, `3` layout, `15` lint, `137` metadata, `2` public-enum, and
`21` snapshot tests. Formatting, offline metadata, `git diff --check`, warnings-
denied all-target/all-feature Clippy, and enlarged-stack all-feature workspace
tests/doctests pass. The coverage plan remains `430/396/0/23`.

Post-source measurements: `elaborator.rs` is `25108 / 945613`, SHA-256
`55597e4a5e18fc13fe2909eaea504cab2be16d48bf526a7f1b1c93d82c7706b4`;
the private Task264 leaf is `1891 / 77732`, SHA-256
`92d664fab2dbe790433398606652ce2b8e65974c642d092d30411cb98fe1b437`.
Contract trees are `125/125`; final audit measurement and commit evidence are
`7490 / 565242`, SHA-256
`a1d534d9e533d9266744d8ff874adb7dd7119d9e2402970d4baeeb9138c2366f`.
Protected fixtures, expectations, trace metadata, and stash remain unchanged.
Parent and independent hard gates pass `9/9`. The final read-only audit reported
no finding, applied no score cap, and assigned valid `100/100`
(`20/20/15/15/10/10/5/5`). The exact task-only commit remains.
