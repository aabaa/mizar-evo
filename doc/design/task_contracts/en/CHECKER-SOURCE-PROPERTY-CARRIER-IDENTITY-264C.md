# Task CHECKER-SOURCE-PROPERTY-CARRIER-IDENTITY-264C: Task264 carrier identity transport

> Canonical language: English. Japanese companion:
> [../ja/CHECKER-SOURCE-PROPERTY-CARRIER-IDENTITY-264C.md](../ja/CHECKER-SOURCE-PROPERTY-CARRIER-IDENTITY-264C.md).

Status: complete on the exact task-only commit containing this record.
Implementation, post-source reviews, and final read-only quality scoring have
no remaining blocking, high, or medium findings. This is a representation-only
checker prerequisite.
It changes no language behavior, protected test intent, diagnostic, semantics,
obligation, trace status, metadata, or coverage credit.

## Identity, authority, and classification

| Field | Frozen value |
|---|---|
| Task | `CHECKER-SOURCE-PROPERTY-CARRIER-IDENTITY-264C` |
| Primary owner | `mizar-checker::source_property_implementation` |
| Owning plan | [`mizar-checker` crate plan](../../mizar-checker/en/00.crate_plan.md) |
| Lower authority | Existing Task264 means/equals `.miz`, trace row, expectations, and completed checker route |
| Consumer | Future Task264 same-source Core-33 carrier context, then Core-34 selector/type ownership |
| Coverage | Zero new semantic, execution, trace, metadata, and coverage credit |

Stable durable-owner links are the checker design's
[carrier identity transport](../../mizar-checker/en/source_property_implementation.md#carrier-identity-transport),
[exact handoff and debug API](../../mizar-checker/en/source_property_implementation.md#immutable-output-debug-and-producer-api),
and [consumer/test boundary](../../mizar-checker/en/source_property_implementation.md#dedicated-consumers-tests-and-write-scope)
sections. This contract owns the task freeze and completion evidence; those
sections own the lasting module API, invariants, and test design.

Authority remains `doc/spec/en/`, existing `.miz`, trace metadata,
expectations, design, then source. Chapter 7 §§7.4.1 and 7.8.2 require the
property implementation to target the existing structure property over its
declared carrier domain; Chapter 5 distinguishes the structure and its
members. The exact Task264 resolver transaction already authenticates local
public/exported `Task264Carrier`, `carrier`, and `marker` identities. Publishing
that already-authenticated tuple is derived transport, not new language or test
intent.

There is no `spec_gap`. The handoff's failure to retain the carrier/field
identity is `source_drift`; the absent public and replay assertions are a
bounded `test_gap`; the prior Core36 disposition already records the dependent
Core `design_drift`. There is no `source_undocumented_behavior`,
`test_expectation_drift`, `boundary_violation`, or `repo_metadata_conflict`.

## Frozen API and exact identity

Add exactly one immutable public value with private fields. Its derives keep
the existing public handoff's `Debug + Clone + PartialEq + Eq` contract:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePropertyCarrierIdentity { /* private fields */ }

impl SourcePropertyCarrierIdentity {
    pub fn structure_symbol(&self) -> &SymbolId;
    pub const fn structure_definition(&self) -> DefinitionId;
    pub const fn structure_contribution(&self) -> SourceContributionId;
    pub const fn structure_origin(&self) -> &SemanticOrigin;
    pub fn field_symbol(&self) -> &SymbolId;
    pub const fn field_definition(&self) -> DefinitionId;
    pub const fn field_contribution(&self) -> SourceContributionId;
    pub const fn field_origin(&self) -> &SemanticOrigin;
    pub fn property_symbol(&self) -> &SymbolId;
    pub const fn property_definition(&self) -> DefinitionId;
    pub const fn property_contribution(&self) -> SourceContributionId;
    pub const fn property_origin(&self) -> &SemanticOrigin;
}

impl SourcePropertyImplementationHandoff {
    pub const fn carrier_identity(&self) -> &SourcePropertyCarrierIdentity;
}
```

`SourcePropertyImplementationHandoff` replaces its private marker-only
resolver snapshot with this single value. The existing producer signature,
input, projection, tables, public errors, Typed/Resolved destination, and
installer remain unchanged. No role enum, second handoff slot, generic resolver
identity abstraction, Core type, or semantic result is added.

The tuple is derived only from the existing `SymbolEnv`:

| Role | Exact identity and provenance |
|---|---|
| structure | definition 0, `Structure`/`Structure`, `Task264Carrier`, contribution 0, `13..101`, path `[4,0,11,0]` |
| field | definition 1, `Selector`/`Selector`, `carrier`, contribution 0, `45..66`, path `[4,0,11,0,18,0]` |
| property | definition 2, `Selector`/`Selector`, `marker`, contribution 0, `71..94`, path `[4,0,11,0,19,1]` |

All three symbols are normal, local to the Task264 module, public/exported,
conflict-free, share the sole local-source contribution, and occur in its exact
three-symbol/three-definition effects. The parameter application head must
equal the retained structure symbol/contribution. Target row 0 must equal the
retained property identity. Any mismatch returns the existing
`InvalidResolverTarget { index: 0 }`.

Final replay has no `SymbolEnv`. Complete resolver authentication therefore
occurs only during construction. Replay checks the immutable snapshot's exact
role ids, normal origins, shared contribution/module, distinct whole symbols,
and property/target self-consistency; the retained source-type handoff
independently reauthenticates only the structure parameter head. It does not
claim an independent lower-handoff identity oracle for the field or property.
The fields are private, so an external caller cannot forge the snapshot. No
identity is reconstructed from a name, range, numeric id, map order, Task263,
or another source.

## Debug, tests, and forbidden behavior

`debug_text()` becomes `source-property-implementation-debug-v2`. Immediately
after the existing module line and seven fingerprint lines, and before the
existing `implementation#0` row, it prints this exact grammar; `<...-fqn>` is the Rust
`Debug` rendering of that retained whole symbol's `fqn().as_str()` value:

```text
carrier-identity#0 role=structure symbol=<structure-fqn> definition=0 contribution=0 origin_range=13..101 origin_path=[4, 0, 11, 0]
carrier-identity#1 role=field symbol=<field-fqn> definition=1 contribution=0 origin_range=45..66 origin_path=[4, 0, 11, 0, 18, 0]
carrier-identity#2 role=property symbol=<property-fqn> definition=2 contribution=0 origin_range=71..94 origin_path=[4, 0, 11, 0, 19, 1]
```

There is one ASCII space between fields, decimal ids/ranges have no padding,
paths use Rust `Debug` list punctuation, each row ends with LF, and no extra
blank line is inserted. Existing Task264 checker and
private runner tests are extended in place: positive assertions cover all 12
getters and exact debug output; construction mutations cover all three resolver
roles; replay mutations cover every snapshot invariant plus property-target and
structure-head linkage. A unique same-module field-symbol replacement has no
independent replay oracle and is protected by build-time resolver
authentication plus private-field immutability, not by an overstated replay
claim. Task264 remains five checker tests and four runner tests.

Forbidden behavior includes modifying `doc/spec`, any existing `.miz`, sidecar,
trace row/backlink/status, runner selection, diagnostic, obligation, accepted
semantics, property value, proof/discharge, Core item, `CoreItemKind`,
`CoreDefinition`, Typed/Resolved installation shape, or Task277B readiness.
The Task263 structure handoff is a different-source fixed profile and is not an
input. The new identity is transport only and grants zero coverage credit.

## Artifact index, baseline, reviews, and exit

Implementation-owned files are the checker source and its existing unit test,
plus the existing private mizar-test Task264 assertion file. Durable derived
owners are the paired checker property design, plan, TODO, source/API audit,
bilingual and module-boundary audits, the paired mizar-test runner-boundary
inventory, this contract, and the central coverage audit. The runner producer
itself is unchanged.

Baselines are checker source `2460 / 89030`, SHA-256
`82a9c45e8a7201e85afe961aefde74f35dd49dac359d4be51062d507294b08ee`;
checker Task264 support test `2004 / 71309`, SHA-256
`7c178ca3911c2c16b8ebf44f28a1128a562f68e2b3769840ee40f97c85bf755e`;
runner Task264 test `236 / 12697`, SHA-256
`602211a63cf51972f46141f4ac8c8b460aa056f19f06a325795eec5f9c6c0880`;
and contract trees `116/116 -> 117/117`. The protected means/equals source and
expectation hashes remain those recorded by Task36P264: sources
`cc90659f10cae4ef68890624df9b8b9d3f0e830dae5e20cc195dc8b263c5fa2b` /
`175135aaf40b9eab1a28e73ca1aae9f250e66278410d50575cdd279f6d7a2784` and
expectations `bced77302602f43f3237424aa2963e5522c1458e879e606c68d1a516cd737c3a` /
`c491d7ea65e1c096d869af4666a06a053a5a0b213d9e79483d13e5ec91b75b6e`.
Protected trace SHA-256 remains
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`.

Required reviews are specification/API and bilingual/boundary before source;
then test sufficiency, implementation/default-deny, source/documentation/API,
and final read-only quality. Exit requires no blocking findings, focused
Task264 tests, checker and mizar-test packages, metadata/lints, formatting,
Clippy with warnings denied, all-feature workspace tests, all nine hard gates,
quality at least `90/100`, task-only commit, clean postcommit state, and fresh
inventory for the same-source Core-33 carrier task.

## Completion evidence

Final Rust measurements are checker source `2625 / 94187`, SHA-256
`0a0f9d887aa6cda7ef11c18936cd27503326c587ad7c8bc3193565828e91fe58`;
checker Task264 support test `2152 / 77946`, SHA-256
`7a7ddb8730f0a39d5739a11a3a2ca094e3997a446fb5b8556b168e34cc48b54d`;
and private runner Task264 test `258 / 13953`, SHA-256
`b5d86410fca9546872fb25ce644381284c97ad58f2e7f703319af99b14cd149a`.
Contract trees are exactly `117/117`; protected source, expectation, trace,
and stash hashes remain unchanged.

Independent specification/API, bilingual/boundary, test-sufficiency,
implementation/default-deny, and source/documentation/API reviews have no
remaining findings after freezing derives/debug/replay scope, updating public
and module inventories, and adding the isolated foreign-module replay test.
Focused checker Task264 passes `5/5`; private runner Task264 passes `4/4`;
checker passes `580/580` plus lint `16/16`; mizar-test passes `642/642`, layout
`3/3`, lint `15/15`, metadata `137/137`, enum `2/2`, and snapshot `21/21`.
Workspace all-feature tests including doctests, warnings-denied all-target/
all-feature Clippy, formatting, offline metadata, recursive contract links, and
diff checks pass. The first default-stack checker-package attempt reached an
unrelated existing deep-test stack overflow; the complete unchanged suite
passes with `RUST_MIN_STACK=16777216` and no assertion failure.

The first final-quality pass found stale exact API/debug owner blocks and
missing stable owner-section links. EN/JA owner documents and contracts were
repaired. A subsequent bilingual/boundary pass found the paired mizar-test
runner-boundary inventory's stale `236`-line assertion-leaf count; it was
synchronized to the measured `258` lines. Both lint surfaces were rerun, and
finding-specific bilingual and final-quality reviews were repeated. The final
review reports **NO FINDINGS**, all hard gates `9/9` PASS, and a valid uncapped
quality score of `100/100`.

The exact twenty-path payload containing this record is the task-only commit.
Its clean postcommit proof and the fresh same-source Core-33 inventory are
read-only successor evidence and do not amend this completed task record.
