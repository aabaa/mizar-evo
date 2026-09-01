# Task CORE-SOURCE-PROPERTY-PARAMETER-CONTEXT-33P264: Task264 parameter Core context

> Canonical language: English. Japanese companion:
> [../ja/CORE-SOURCE-PROPERTY-PARAMETER-CONTEXT-33P264.md](../ja/CORE-SOURCE-PROPERTY-PARAMETER-CONTEXT-33P264.md).

Status: complete on the task-only commit. Independent reviews, verification,
hard gates `9/9`, and the valid uncapped quality score `100/100` are complete.
This is a representation-only, zero-semantic, zero-credit Core-33
prerequisite. It changes no language behavior, protected test intent,
diagnostic, obligation, trace, metadata, or coverage credit.

## Identity, authority, and classification

| Field | Frozen value |
|---|---|
| Task | `CORE-SOURCE-PROPERTY-PARAMETER-CONTEXT-33P264` |
| Primary owner | `mizar-core::elaborator`, Core Task 33 |
| Required predecessors | Task33LB, Task264 checker transaction, Task33I264, Task34I264/34D264 |
| Inputs | Complete `SourcePropertySelectorTypeContextHandoff` and exact Task264 `SourceBindingContextHandoff` |
| Consumer | Separately reviewed Task264 Core35 means/equals body inputs |
| Owning plan | [`mizar-core` Task Index](../../mizar-core/en/00.crate_plan.md#task-index) |
| Coverage | Zero semantic, execution, trace, metadata, and coverage credit |

Authority remains `doc/spec/en/`, existing `.miz`, trace metadata,
expectations, design, then source. Chapters 4, 5, and 7 plus both existing
Task264 `.miz` sources fix one definition parameter `M` over
`Task264Carrier`. The checker transaction already authenticates property
parameter 0, binding 0, target subject 0, its exact source declaration/context,
and the domain type application/root. Task33LB already owns deterministic
checker-binding-to-free-term-Core-variable allocation.

There is no `spec_gap`. The absent branded property-parameter/Core-variable
join is bounded `design_drift` and `source_drift`; focused private assertions
are a bounded `test_gap`. All other gap classes are absent. This task selects
only a derived representation and does not change language or accepted test
intent.

## Frozen public API and representation

Add exactly this private-field public surface beside the existing Task264 Core
contexts:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyParameterCoreVariableAssociation { /* private fields */ }

impl SourcePropertyParameterCoreVariableAssociation {
    pub const fn parameter(&self) -> SourcePropertyParameterId;
    pub const fn binding(&self) -> BindingId;
    pub const fn core_var(&self) -> CoreVarId;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyParameterCoreContextHandoff { /* private fields */ }

impl SourcePropertyParameterCoreContextHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub const fn context(&self) -> &CoreContext;
    pub const fn selector_context(&self)
        -> &SourcePropertySelectorTypeContextHandoff;
    pub const fn source_context(&self) -> &SourceBindingContextHandoff;
    pub const fn source_bindings(&self) -> &SourceBindingCoreContextHandoff;
    pub const fn association(&self)
        -> &SourcePropertyParameterCoreVariableAssociation;
    pub fn debug_text(&self) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePropertyParameterCoreContextError {
    EnvironmentMismatch,
    InvalidSelectorContext,
    InvalidSourceContext,
    InvalidBindingContext,
    InvalidAssociation,
}

#[derive(Debug, Clone, Copy)]
pub struct SourcePropertyParameterCoreContextProducer;

impl SourcePropertyParameterCoreContextProducer {
    pub fn build(
        selector_context: SourcePropertySelectorTypeContextHandoff,
        source_context: SourceBindingContextHandoff,
    ) -> Result<
        SourcePropertyParameterCoreContextHandoff,
        SourcePropertyParameterCoreContextError,
    >;
}
```

The handoff retains the complete selector/type context, complete checker source
context, internally derived complete 33LB handoff, and one association by
value. Construction calls the existing `SourceBindingCoreContextProducer` on
an exact clone of `selector_context.carrier_context().context()` and the exact
source-context binding environment. Replay deterministically rebuilds that
33LB value and compares the complete result; it does not allocate through a
second algorithm or trust numeric IDs alone.

The exact association is `parameter/binding/core-var = 0/0/0` for both means
and equals. Allocation remains the existing checked max-plus-one rule; the
carrier context has no existing variable, so binding 0 deterministically maps
to `CoreVarId(0)`. The variable is free, term-sorted, role
`definition-parameter`, sourced at declaration range `125..126`, carries only
`source-binding-core-variable-v1.binding.0` checker provenance, and has an
empty type-fact list. No binder frame is created.

`debug_text()` is exactly
`source-property-parameter-core-context-v1|module=<package>.<path>|carrier-item=<id>|bindings=1|parameter=0:0:0`
with decimal IDs, no padding, and no final LF. It is deterministic evidence,
never an identity oracle.

## Exact validation and default-deny order

Validation order is environment, selector context, source context, derived
binding context, then association, followed by complete postvalidation.

- Both inputs and every retained nested handoff share one source/module.
- The complete selector context replays unchanged, including domain binding 0,
  application/root `0/0`, carrier item 0, and property return member/root `1/2`.
- Its checker owner's source-context fingerprint equals the complete supplied
  context; the context contains exactly one normal property-implementation
  item, one normal `M` declaration, two context/local-context links, one normal
  active definition-parameter binding, and no diagnostics.
- The source item, declaration, binding environment, checker parameter, target
  subject, and selector-context domain all join on binding 0 using typed IDs.
  Source item/declaration sites and ranges equal the checker-owned rows; the
  local scope remains `[4]` and declaration context remains 1.
- The retained 33LB handoff equals a fresh deterministic rebuild from the
  untouched carrier context and exact binding environment. Its context retains
  the sole carrier item/boundary/work item and adds only the one variable,
  binder-source row, and empty type-fact entry above.
- The private association reproduces the checker parameter, binding row, and
  retained 33LB variable exactly.

Mixed profiles, foreign transactions, incomplete/recovered source contexts,
stale fingerprints, altered carrier contexts, extra bindings/variables, and
association drift fail before publication. Branded private fields are the
replay integrity boundary.

## Scope, tests, and forbidden behavior

Rust edits are exactly `crates/mizar-core/src/elaborator.rs` and the existing
private Task264 test leaf. Add two private tests: one deterministic exact
means/equals association test, and one cross-profile/foreign/default-deny test.
Core unit count remains 164; mizar-test unit count projects `646 -> 648`.
Existing `.miz`, expectations, trace, snapshots, selection, and production
runner routes remain unchanged.

Do not add a generic adapter, new `CoreContextInput` field, Typed/Resolved/CoreIr
slot, property or field item, normalized type/guard/fact, binder frame, term,
formula, selector lowering, current-result `it`, definition body, correctness/
coherence seed, diagnostic, obligation, route, or coverage credit. Do not edit
`doc/spec`, existing `.miz`, expectations, trace metadata, snapshots, checker,
`core_ir.rs`, or VC. Task264D is not an input: it remains the equals-only later
selector-occurrence identity; means current-result representation remains a
separate prerequisite.

## Artifacts, baselines, reviews, and exit

Durable owners are this paired contract; paired Core plan, elaborator,
decomposition, TODO, source/spec, bilingual, and boundary records; paired
mizar-test harness/bilingual/boundary records; and the central coverage audit.
Stable owner sections are the elaborator
[Task33P264 API](../../mizar-core/en/elaborator.md#task-33p264-task264-parameter-core-context),
Core [decomposition entry](../../mizar-core/en/source_family_decomposition.md#task-33p264-task264-parameter-core-context),
and mizar-test [private probe](../../mizar-test/en/harness.md#core-task-33p264-private-task264-parameter-probe).

Clean baseline is HEAD `2a06adfb8172b497d28b75f0a03cbdb593831b0f`.
`elaborator.rs` is `23785 / 893870`, SHA-256
`f65c64b2a59ab68689b6a53e0c334b231545cfcfe33f6b5e178ce18ebe3d7928`;
the private Task264 leaf is `1143 / 49788`, SHA-256
`c5d746b3d16bca7088aedfc3a31a286a84737790f8936d1ff48488a95e0b196d`;
the central audit is `7444 / 562566`, SHA-256
`9da542688b7e83bdaa4204576ee061aef36080a4c2f49f52163b6bfe40ad9de5`.
Contract trees project `122/122 -> 123/123`; Core/checker/mizar-test unit
counts begin at `164/582/646`. Protected hashes and stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` remain unchanged.

The implemented source measures `24256 / 912957` for `elaborator.rs`, SHA-256
`d48f63f0a8427bc6e9c4290affcc8ae5e909c8b2af5391398256894d1334ff11`,
and `1377 / 58827` for the private Task264 leaf, SHA-256
`b7b443a4638ce10216b6ffc1f72c0324dd8806ab5a33f2d5c8b8e77ac788afc4`.
The contract trees measure `123/123`. The final central audit measures
`7459 / 563428`, SHA-256
`085f0836ac2a0047eb59d058aa93310e3c7f750fe090fbc230b32ca01e87b9cf`.

Independent pre-source specification/API and bilingual/boundary reviews must
end with no blocking finding. Post-source test sufficiency, implementation,
source/documentation/API, bilingual/boundary, and final read-only quality
reviews must end with no blocking/high/medium finding. Run focused Task264
tests, Core and mizar-test package/lint/metadata suites, fmt, offline metadata,
warnings-denied all-feature Clippy, and enlarged-stack all-feature workspace
tests/doctests. Exit requires hard gates `9/9`, quality at least `90/100`, a
task-only commit, clean postcommit proof, and fresh inventory for the next
Task264 Core35 prerequisite.

After completion, equals Core35 may consume this parameter context plus Task264D
to lower only the exact selector term in a separately frozen task. Means still
requires an explicit current-definition-result `it` representation. Core36 and
Task277B remain not-ready/zero-credit.

## Completion evidence

Independent pre-source specification/API and bilingual/boundary reviews ended
with no blocking/high/medium finding after the clean-HEAD baseline was
distinguished from the documentation worktree and the JA owning-plan backlink
was restored. Post-source test-sufficiency and implementation/default-deny
reviews ended with no finding. Source/documentation/API and bilingual/boundary
review initially found two medium lifecycle/measurement findings; synchronized
EN/JA status, closed-gap records, the public inventory, boundary measurement,
and post-source artifact evidence resolved both, and the finding-specific
re-review ended with no remaining finding.

Focused Task33P264 tests pass `2/2`, the complete Task264 private family passes
`10/10`, Core passes `164` unit, `2` determinism, and `12` lint tests, and
mizar-test passes `648` unit, `3` layout, `15` lint, `137` metadata, `2`
public-enum, and `21` snapshot tests. Formatting, offline metadata,
`git diff --check`, warnings-denied all-target/all-feature Clippy, and the
enlarged-stack all-feature workspace tests/doctests pass. The coverage plan is
unchanged at `430` cases, `396` requirements, zero errors, and `23` warnings.

Protected means/equals `.miz` hashes remain `cc90659f...`/`175135aa...`, their
expectations remain `bced7730...`/`c491d7ea...`, trace metadata remains
`17bba212...`, and protected stash remains `f65cf4a...`. No specification,
expectation, trace, snapshot, diagnostic, obligation, route, or semantic credit
changed. Parent and independent hard gates pass `9/9`. The independent final
read-only audit reported no finding, applied no score cap, and assigned a valid
`100/100` (`20/20/15/15/10/10/5/5`). Exact staging and the task-only commit are
prepared; clean postcommit proof and fresh successor inventory are recorded in
the operational handoff rather than this self-referential task commit.
