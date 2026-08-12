# Task CHECKER-FRAENKEL-NESTED-MAPPER-PRIMARY-257C4C4: Nested Fraenkel Mapper Primary

> Canonical language: English. Japanese companion:
> [../ja/CHECKER-FRAENKEL-NESTED-MAPPER-PRIMARY-257C4C4.md](../ja/CHECKER-FRAENKEL-NESTED-MAPPER-PRIMARY-257C4C4.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).
Durable owner sections: checker
[source term](../../mizar-checker/en/source_term.md#task-257c4c4-nested-fraenkel-mapper-primary)
and test
[harness](../../mizar-test/en/harness.md#checker-task-257c4c4-private-nested-mapper-primary-probe).

## Status, purpose, and readiness

**Status:** complete.

Fresh read-only inventory at clean
`HEAD 5578f7e51f5acfb60494dbacb41640b976c9c55c` selects this task as the
dependency-minimal successor to completed C4C3. Chapter 13 and the human
semantic confirmation fix the inner mapper `x@94..95` as a use of the distinct
outer generator `x@136..137`. Completed C4C2 authenticates that resolver
relation, and completed C4C3 transports it to exact checker typed sites. This
task adds only the missing Task-252 primary occurrence and binding reference in
one C4C3-gated transaction.

There is no `spec_gap`. The absent specialized Task-252 forward-written
Fraenkel profile is `design_drift`; the absent exact checker and imported-
fixture occurrence regressions are a `test_gap`; after this contract is frozen,
the absent implementation is `source_drift`. Reusing exact-F5 C4A/C4B,
relaxing generic Task-252 declaration-order lookup, treating resolver binding
ID `1` as checker `BindingId(1)`, or creating capture/semantic state is a
`boundary_violation`.

The user-authorized intermediate split is dependency-minimal, zero-semantic,
and default-deny. Competing alternatives were: project both generator binders,
put the outer binding in the inner owner context, add a separate public binding
handoff before Task 252, or globally change Task-252 source-order rules. They
are rejected because the sole requested occurrence references only outer `x`,
outer ownership is already exact, another handoff adds unused public surface,
and generic behavior must remain unchanged.

## Authority, dependencies, and protected artifacts

Authority is, in order:

1. canonical [Chapter 13 §§13.4.2, 13.4.4, and 13.8.6](../../../spec/en/13.term_expression.md#134-set-expressions);
2. exact existing
   [`pass_types_nested_comprehension_outer_generator_capture_001.miz`](../../../../tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.miz);
3. its sole existing [trace row](../../../../tests/coverage/spec_trace.toml);
4. its inactive [expectation](../../../../tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.expect.toml);
5. completed [C4C2](RESOLVE-FRAENKEL-NESTED-CAPTURE-257C4C2.md) and
   [C4C3](CHECKER-FRAENKEL-NESTED-BINDER-USE-257C4C3.md), followed by derived
   design/source inventory.

The source, inactive sidecar, and trace remain byte-identical with SHA-256
`c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431`,
`9ed000a30c1d519bd665f338c636fb9e529e9848a285209bebe6728f19961b92`, and
`d4d817e83aac78d19e729702b26c62604fc57581eec18672a5c26ec44efe7a81`.
The sidecar stays inactive `advanced_semantics`, `pass/type_check`, with no
diagnostic codes or active tags. The trace remains test-intent-only and grants
no execution or semantic credit.

The sole lower dependency is one complete, internally valid
`SourceNestedFraenkelBinderUseHandoff`: one row, resolver use `0`, resolver
binding `1`, typed outer binder `x@136..137`, typed inner mapper use
`x@94..95`, and source ordinal `0`. The new producer must reauthenticate the
dependency's retained C4C2 resolver and typed snapshot through a crate-private
C4C3 validation method before projecting any Task-252 state. C4A/C4B are
negative compatibility profiles, not dependencies.

## Selected lower projection

The specialized transaction constructs exactly one immutable `BindingEnv`:

| Context | Owner | Parent/layer | Bindings | Visible |
|---:|---|---|---|---|
| `0` | `Module` | none / `Module` | `[]` | `[]` |
| `1` | `SourceComprehension { 90..157 }` | `0` / `Expression` | `[0]` | `[0]` |
| `2` | `SourceComprehension { 92..123 }` | `1` / `Expression` | `[]` | `[0]` |

All contexts have no lexical scope and normal recovery. The table has exactly
one binding and zero diagnostics. Checker binding `0` is spelling `x`, kind
`QuantifierBinder`, identity `SourceBound { context: 1, ordinal: 0 }`, owner
context `1`, declaration `136..137`, visibility ordinal `0`, type site
`Source(141..155)`, `Active`, normal recovery, empty captured identities, and
empty diagnostics. Its numeric checker ID is local to this projection; it is
not resolver binding ID `1`. The C4C3 dependency authenticates the explicit
relation between those identities.

The mapper query is exact context `2`, no lexical scope or resolver fallback,
spelling `x`, and specialized logical ordinal `1`. Lookup at ordinal `0` must
be `ForwardReference { candidates: [0] }`; lookup at ordinal `1` must be
`Local(0)`. The ordinal is a profile-scoped logical generator-visibility
coordinate, not a source byte order, C4C2 use ordinal, or capture ordinal.

The private Task-252 projection arena contains exactly one root node `0`:
kind `source.term.variable-reference`, anchor `94..95`, no resolved node, no
children, `Unknown`, normal recovery, and empty links. It is a syntax-free
Task-252 transport projection authenticated against C4C3's real typed
`Identifier`; it is not a replacement `TypedAst`, an installation, or a claim
that the two node IDs are interchangeable.

The resulting existing `SourcePrimaryTermHandoff` has exact cardinality
`1/1/0`:

| Table | ID | Exact row |
|---|---:|---|
| term | `0` | node `0`; `94..95`; ordinal `0`; context `2`; `Normal`; `x`; `VariableReference`; `Value`; no parent |
| reference | `0` | term `0`; checker binding `0`; `Variable`; no lexical scope; use ordinal `1` |
| numeric request | — | empty |

Inner generator `y` is authenticated by C4C3 as distinct resolver binding `0`
but is deliberately absent from this occurrence-only checker projection. No
spelling or numeric-ID coincidence may establish the outer relation.

## Frozen API, validation, and default deny

`crates/mizar-checker/src/source_term.rs` is the sole production owner. The
exact new public family is:

```rust
SourceNestedFraenkelMapperPrimaryHandoff
#[non_exhaustive] SourceNestedFraenkelMapperPrimaryError
SourceNestedFraenkelMapperPrimaryProducer
```

The handoff exposes exactly:

```rust
source_id() -> SourceId
module_id() -> &ModuleId
dependency() -> &SourceNestedFraenkelBinderUseHandoff
dependency_fingerprint() -> &str
binding_env() -> &BindingEnv
binding_fingerprint() -> &str
projection_arena() -> &TypedArena
source_term() -> &SourcePrimaryTermHandoff
source_term_fingerprint() -> &str
debug_text() -> String
```

The producer signature is exactly:

```rust
SourceNestedFraenkelMapperPrimaryProducer::build(
    dependency: SourceNestedFraenkelBinderUseHandoff,
) -> Result<SourceNestedFraenkelMapperPrimaryHandoff,
          SourceNestedFraenkelMapperPrimaryError>
```

The dependency is consumed and retained immutably. No public unchecked
constructor, mutable accessor, alternate input, optional profile selector,
installation API, or `Default` implementation is added. C4C3 adds only one
crate-private complete-validation entry point; it exposes no retained resolver
or `TypedAst` getter.

The debug grammar is exactly:

```text
source-nested-fraenkel-mapper-primary-debug-v1
module: <package>::<path>
dependency-fingerprint: <Debug quoted complete C4C3 debug text>
binding-fingerprint: <Debug quoted complete BindingEnv debug text>
projection: nodes=1 root=0
source-term-fingerprint: <Debug quoted complete source-primary-term debug text>
```

The non-exhaustive error enum has exactly these variants and public strings, in
this precedence:

```rust
InvalidDependency        // "nested Fraenkel mapper-primary dependency is invalid"
InvalidBindingEnvironment // "nested Fraenkel mapper-primary binding environment is invalid"
InvalidSourceTerm         // "nested Fraenkel mapper-primary source term is invalid"
```

Dependency environment/snapshot/resolver/typed/row/fingerprint corruption is
first. Exact context/binding/lookup/fingerprint corruption is second. Exact
arena/root/node and term/reference/request/fingerprint corruption is third.
Admission is atomic and default-deny; missing, extra, reordered, duplicated,
recovered, mismatched, or partially valid state publishes no handoff.

`SourcePrimaryTermBindingProfile` gains only a private exact nested-Fraenkel
case. It supplies logical use ordinal `1` and permits the authenticated outer
binder's textual declaration to follow this exact mapper. Public generic
`SourcePrimaryTermProducer::build`, every other private profile, the generic
declaration-derived ordinal rule, generic forward-reference rejection, role
allowlists, error behavior, and installed Task-252 transactions remain
unchanged.

## Scope, tests, and audit impact

Implementation changes exactly:

1. `crates/mizar-checker/src/source_formula_composition.rs` for the
   crate-private dependency revalidation seam;
2. `crates/mizar-checker/src/source_term.rs` for the sole production owner and
   checker tests; and
3. existing private
   `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs`
   for the sole current consumer.

The checker owner adds exactly four tests:

1. `task257c4c4_builds_exact_nested_mapper_primary_handoff`;
2. `task257c4c4_rejects_dependency_and_binding_projection_corruption`;
3. `task257c4c4_rejects_arena_term_reference_and_precedence_corruption`; and
4. `task257c4c4_replays_deterministically_and_preserves_generic_task252_rejection`.

The private leaf adds exactly
`task257c4c4_real_imported_fixture_builds_mapper_primary_handoff`. It calls the
producer directly after C4C1/C4C2/C4C3 construction, checks the exact public
dependency/binding/arena/term surface, and remains library-test-only.

Required corruption coverage includes dependency source/module/summary/row and
retained snapshot replay; context cardinality/order/owner/parent/range/scope/
recovery/visibility; x binding identity/owner/range/visibility/type/status/
capture/diagnostics/recovery; query forward/local results; arena root/count/
kind/range/resolution/children/typing/recovery/links; term/reference/request
cardinality and every row field; fingerprint replay; error precedence; generic
forward-order rejection; raw `Identifier` rejection; and deterministic clone
replay.

Baselines are `source_term.rs` `6451` lines / SHA-256
`e6f96b3fd83c77c06689d53e7efc6ddae27c744d5ffed79019ced2d2104d4602`,
`source_formula_composition.rs` `9358` /
`eed8c480a2ddeceafd529ee4c37c333f6e36f8f23e62f4b53f782bc9df651b6c`,
and the private leaf `248` /
`46bb3e63199d4b9794a9d56c214d76864a073cc35b0643ec64a8a1e412d5bb0a`.
Raw library test lists project checker `558 -> 562` and mizar-test `620 ->
621`; baseline hashes are respectively
`aa1eccf5bd93c9574082f7c888918ccb2bbc76167aa5ef0c672a6db931e42d8f`
and `95ff9e007bd474cad657e626f61424db408ec343f6f1a6c1b84d6fff50ee9a75`.
Contract trees project `93/93 -> 94/94`. Source line counts/content hashes will
change and are measured once here after implementation.

`doc/design/spec_coverage_audit.md` receives one zero-credit mapping because a
new durable Task-252 structural owner and private consumer become current.
Specification, `.miz`, expectation, trace row/status/backlink, active route,
diagnostic, semantic result, and executable/semantic coverage credit remain
unchanged.

## Forbidden behavior, reviews, and exit

This task creates no `CapturedFreeVariables` identity, semantic capture,
generated-core parameter, type or sethood answer, request/result, verdict,
diagnostic, proof/fact, `TypedAst`/`ResolvedTypedAst` installation, production
dispatch, runner activation, registry entry, sidecar/trace activation,
coverage credit, or Task-277B state. The structurally required binding field is
exactly empty. Task 277B remains not-ready and zero-credit.

Required independent reviews are specification/contract, test sufficiency,
implementation, source/documentation/API, bilingual/boundary, and final hard-
gate quality. Every material finding requires repair and re-review.
Verification includes focused tests, checker and mizar-test libraries, both
lint-policy suites, metadata tests, formatting, warnings-denied workspace
Clippy, full workspace tests, unchanged CLI replays, protected hashes, exact
scope/diff review, task-only staging, commit, and clean postcommit proof.

Exit requires **NO FINDINGS**, all `9/9` autonomous hard gates, valid quality
`>=90/100`, a task-only implementation commit, clean/stash-invariant
postcommit proof, and a fresh readiness inventory. Per the user's current
instruction, the agent stops after this one task even if that inventory finds
a ready successor; no successor implementation is authorized in this run.

Recommended routing: GPT-5.6 Sol at `xhigh` owns authority, public API,
boundary acceptance, finding disposition, and final scoring. Terra at `high`
or `xhigh` may perform frozen implementation and independent reviews.

## Completion evidence

The frozen documentation prerequisite committed as
`faa558276ff984ac20c8aef60caf0b7712e5554c`. Independent specification/API,
documentation-boundary, test-sufficiency, and implementation reviews report
**NO FINDINGS** after the exact corruption matrix was completed. The
implementation remains inside the exact three-Rust-path scope and preserves
generic Task-252 source-order rejection.

Final source measurements are:

| Path | Lines | SHA-256 |
|---|---:|---|
| `crates/mizar-checker/src/source_term.rs` | `7574` | `2ef60bd40d0ff147f1615d20bd3a9fff3980e916868da90f998b00c3b4d369fe` |
| `crates/mizar-checker/src/source_formula_composition.rs` | `9411` | `2b982a6ab418e63ee6996c428aea2f8d5a4b3fc6bb55c9e830043f07fec73e56` |
| `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs` | `416` | `7760e98cb9b6fb3ea26f232b34551119d6d084c0f4785cd11b3af7cf829be1f1` |

Checker production remains `32` paths / `196334` lines with unchanged path
hash `9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787`
and content-manifest hash
`783826a091b8a5892b88a7f5c34a2bfff55683befa3cc98248eaa6de19afc5c6`.
Focused tests pass `4/4` checker and `1/1` mizar-test; both full library suites
pass `562/562` and `621/621`. Their raw-list SHA-256 values are respectively
`2abdadafce718a2cd05d3a38240f4c3a96939b5ccec3174511481be7ee562e78`
and `a0d872b065dc98b8dd9caff1d964d8d58d8af372ef3a57cfec7455b31bce4f63`.
Warnings-denied checker+mizar-test Clippy,
formatting, and `git diff --check` pass. Contract trees remain `94/94`, and all
three protected authority hashes remain exact. Independent source/docs/API
review reports **NO FINDINGS**. The initial bilingual/boundary review found one
lifecycle-ordering mismatch; after that review completed, the contract and
TODO were synchronized. Full-workspace Clippy and `cargo test` pass.
Finding-specific bilingual/boundary re-review reports **NO FINDINGS**.
Independent final-quality review reports **NO FINDINGS**, all `9/9` hard gates
pass, and the valid uncapped quality score is `100/100`
(`20/20/15/15/10/10/5/5`). It independently confirms the exact 20-path scope
and authorized task-only staging. The five unchanged plan/parse/declaration/
type/proof CLI stdout
hashes are respectively
`2d2accef2c6fc32c1b3372530f6136af1299ac6ae7db6a0158798336b779c7e7`,
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`;
all retain the existing `23` warnings and zero errors. Checker and mizar-test
lint-policy suites pass `15/15`, metadata passes `137/137`, and no score cap
applies.

## Postcommit proof and fresh successor inventory

The reviewed task-only implementation committed as
`7ae41e91f6d7cfe0036651d14b31dc570722a274` over documentation prerequisite
`faa558276ff984ac20c8aef60caf0b7712e5554c`. `git show --check` passed; the
immediate worktree was clean; `origin/HEAD...HEAD` was `0/2`; and protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`, the canonical `.miz`, inactive
sidecar, trace, and all final measurements were observed unchanged.

The fresh read-only inventory finds no uniquely dependency-ready semantic
successor. This is not a Chapter-13 `spec_gap`: resolved-identity capture and
the inner generated-core parameter are specified. The missing single owner/API
across the Task-255 set-comprehension, Typed/Resolved installation, and core
`Comp_H`/generated-origin boundary is `design_drift`; the absent exact parameter
identity/order, membership/sethood, installation, and executable semantic
oracle is a `test_gap`. C4C4 intentionally supplies none of those results.
Task 277B remains not-ready and zero-credit. Independently, the user's explicit
one-task instruction requires STOP here even if a later inventory makes a
successor ready.
