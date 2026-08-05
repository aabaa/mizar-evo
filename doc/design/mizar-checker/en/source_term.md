# Source Primary-Term Handoff

> Canonical language: English. Japanese companion:
> [../ja/source_term.md](../ja/source_term.md).

## Purpose And Authority

The public `source_term` module implements Checker Task 252. It transports
source occurrences of variable and local-constant references, `it`, numerals,
and transparent parentheses into the checker without importing raw syntax.
The canonical authority is Chapter 04 §§4.1-4.3, 4.4.1, and 4.6 and Chapter
13 §§13.1, 13.8.1-13.8.2, and 13.8.8. MC-G017 and MC-G020 track the broader
term and source-to-checker gaps.

The module is transport-only. It authenticates source shape, binding lookup,
and missing numeric-type requests. It does not choose a numeric type, create a
semantic term or formula, type a current definition result, publish a fact or
axiom, or create FOL/downstream IR.

## Public Model

`SourcePrimaryTermHandoffInput` carries one source/module transaction plus
three ordered input tables:

- `SourcePrimaryTermInput`;
- `SourcePrimaryTermReferenceInput`; and
- `SourceNumericTypeRequestInput`.

`SourcePrimaryTermProducer::build` authenticates those rows against a
syntax-free `BindingEnv` and `TypedArena`, then atomically publishes
`SourcePrimaryTermHandoff`. Its immutable
`SourcePrimaryTermTable`, `SourcePrimaryTermReferenceTable`, and
`SourceNumericTypeRequestTable` expose only borrowed lookup, source-ordered
iteration, length, and emptiness. Their dense identities are
`SourcePrimaryTermId`, `SourcePrimaryTermReferenceId`, and
`SourceNumericTypeRequestId`.

Term rows retain a node site, exact source range, dense pre-order source
ordinal, binding context, recovery, token-normalized spelling, kind, role,
and optional parent. Reference rows retain term and binding identity plus
role; lexical scope and use ordinal are producer-derived output. Numeric
requests retain the exact numeral term/site/range/spelling and a dense request
ordinal. `debug_text()` renders every table deterministically.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourcePrimaryTermKind` | `#[non_exhaustive]`; callers must tolerate later primary-term families. |
| `SourcePrimaryTermRole` | `#[non_exhaustive]`; callers must tolerate later source roles. |
| `SourcePrimaryTermReferenceRole` | `#[non_exhaustive]`; callers must tolerate later authenticated binding roles. |
| `SourcePrimaryTermRecovery` | `#[non_exhaustive]`; callers must tolerate later recovery classes. |
| `SourcePrimaryTermError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |
| `SourceProofLocalGivenUseTermError` | `#[non_exhaustive]`; callers must not exhaustively match Task 269GU dependency, input, or installation failures. |
| `SourceProofLocalGivenConditionUseTermError` | `#[non_exhaustive]`; callers must not exhaustively match Task 269GCU dependency, input, or installation failures. |

No exhaustive public enum exceptions are owned by this module.

## Validation And Atomicity

Term ids and `source_ordinal` are equal dense pre-order indices. Every site is
a unique `TypedSiteRef::Node` whose arena kind, range, and recovery exactly
match the row. Identifier references use nonempty binding-authenticated
spellings accepted by the canonical `mizar_lexer::is_identifier` predicate:
an ASCII alphabetic or `_` start, ASCII alphanumeric, `_`, or apostrophe
continuations, and rejection of reserved words. This reuses lexical vocabulary
without importing raw syntax. `it` is exactly `it`, numerals contain only ASCII
digits, and a parenthesis spelling is exactly `( <child spelling> )` with one
ASCII space between tokens.

Each parent is an earlier parenthesis in the same context whose range strictly
contains its one immediate child. Only parents own children. Roots and
siblings remain source ordered, and nested parents form a closed acyclic
pre-order tree over the five Task-252 kinds. The private runner extractor
excludes an entire parenthesized subtree if any descendant belongs to a later
term family.

Variable and constant rows have exactly one reference. On the generic profile, variables accept only
`ReservedVariable`, `LetBinding`, `QuantifierBinder`, or
`DefinitionParameter`; constants accept only `LocalAbbreviation`. `it`,
numerals, and parentheses have no binding reference. Every numeral has
exactly one numeric request, and no other term kind has one.

For each reference, the producer clones lexical scope from the term context
and derives `use_ordinal` as the number of binding rows whose declaration
ranges end no later than the term start. Previous references do not advance
that ordinal. Normal binding groups are source ordered singletons with
visibility equal to their dense index. An exact consecutive duplicate group
shares spelling, kind, owner context, `BinderIdentity`, range, and the final
group row's dense index as its visibility ordinal. This preserves the whole
group until `BindingEnv::lookup` can reject it as `Ambiguous`.

The producer constructs `BindingLookupSite::new` with no resolver payload and
requires the exact supplied local binding winner. Forward, ambiguous, missing
scope payload, unresolved, different-winner, and lookup-error results fail
closed. `Resolver` is structurally unreachable on this path. Inputs are never
sorted, repaired, or partially published.

## Ownership And Consumers

`TypedAst::with_source_term` installs one optional immutable handoff after
revalidating source/module and every arena node; replacement is rejected.
`ResolvedTypedAst` only clone-preserves the handoff and exposes
`source_term()`.

The private `mizar-test::runner::type_elaboration::source_term` leaf owns raw
`SurfaceAst` extraction. Its exact real selector is:

1. `fail_type_elaboration_term_formula_gap_001`;
2. `pass_type_elaboration_reserved_variable_equality_001`; and
3. `pass_type_elaboration_parenthesized_reserved_variable_equality_001`.

Their aggregate handoff is seven terms, four references, and two numeric
requests. Existing semantic outcomes and detail keys remain unchanged.
Synthetic tests exercise local constants, `it`, nested parentheses, and
mixed-family exclusion without adding semantic acceptance.

## Verification And Deferrals

Checker tests cover every kind and role, dense order, binding-event order,
shadow/forward/ambiguous/missing/unresolved lookup behavior, reference and
numeric-request cardinality, parent graphs, source/module/site/range/kind/
spelling/recovery/context corruption, deterministic rendering, and typed-AST
installation. Runner tests cover the exact real selector, 7/4/2 oracle,
synthetic dependency boundaries, isolation, corruption, deterministic replay,
and final resolved preservation.

The covered trace requirement is
`spec.en.checker.type_elaboration.source_primary_term_payload`. Applications,
structure/set/choice/comprehension/`qua` terms, formula graphs, definition
result semantics, real proof-local constant production, numeric responses,
accepted facts/declarations/proofs, downstream IR, Tasks 253+, and Steps 6/7
remain with their explicit owners.

## Task 257B1 Consumer Addendum

Task 257B1 adds one exact pass consumer with two additional
`VariableReference`/`Value` rows and two binding references. Both references
select the explicit quantifier's binding 0 in body context 1. Task 252 keeps
exclusive occurrence and lookup-winner ownership; the formula-composition
handoff records only binder-to-reference associations and does not repurpose
captured-free-variable metadata.

Task 257B2 reuses sixteen numeral rows and sixteen numeric-type requests in
body context 1. It intentionally creates zero references: the explicit `x`
binder is unused, captured identities remain empty, and the composition layer
does not invent bound-use rows.

## Task 257B3 Frozen Consumer Addendum

Task 257B3 reuses exactly six `VariableReference`/`Value` terms and six
Task-252 lookup-selected references. In source order, three `x` occurrences
select outer quantifier binding 1, one `y` selects binding 2, and two `r`
occurrences select inner quantifier binding 3 rather than reserved binding 0.
Terms/references 0-1 use context 1 and rows 2-5 use context 3. Terms are
`VariableReference`/`Value`/`Normal` with source ordinals `0..5`; references
retain the variable role.
Scope paths and local identities are source-derived resolver-shaped preflight
facts, while use ordinals are authenticated Task-252 producer output. Formula
composition records owning-edge associations only; Task 252 keeps occurrence,
reference, spelling, lexical-scope, and lookup-winner ownership.

Task 257B3 now executes this six-row reciprocal consumer and verifies binding
ids `1,1,3,2,1,3` with use ordinals `2,2,4,4,4,4`; Task 252 retains
every occurrence and reference.

## Task 257C1 Frozen Consumer Addendum

Task 257C1 reuses exactly three Task-252 `Numeral`/`Value` primaries and
numeric requests, profile `3/0/3`, for source terms `1`, `2`, and `3`.
Primary 1 (`2`, `85..86`) is one occurrence: the new Task-256 shared-boundary
edge references it from both adjacent segment descriptions without duplicating
the term or request. Task 252 retains occurrence, spelling, range, arena, and
numeric-request ownership; predicate grouping and polarity remain Task 256.

Task 257C1 now exercises this frozen backlink in the active pass consumer.
The measured `3/0/3` profile and single middle-primary identity are preserved;
no Task-252 API or semantic numeric result changed.

## Task 255C1 Frozen Backlink

The exact conditioned-comprehension prerequisite builds one immutable
Task-252 `4/0/4` handoff. Primaries 0/1 are Task-253 mapper arguments;
primaries 2/3 are equality operands wholly inside the authenticated condition
range. The latter remain ordinary Task-252 occurrence/numeric-request rows
but have no Task-255 edge. This preserves the exact objects later Task 256
must target without granting formula or numeric semantics here.

## Task 255C1 Transport Result

The exact route now publishes that single `4/0/4` handoff. Copied, omitted,
or range-substituted condition primaries fail the complete Task-255
transaction, while both authentic condition operands remain available to
later Task-256 installation without a Task-255 edge.

## Task 257C2 Frozen Consumer Boundary

Task 257C2 reuses the same immutable `4/0/4` handoff. Task-256 equality edges
target primaries 2 and 3 directly; the Task-257C2 association targets only
the condition and formula IDs. No Task-252 row, request, parent, context,
fingerprint, debug byte, or numeric semantic meaning changes. At the frozen
pre-Task-256C1 baseline, the route was gated on that separate lower task.
Task 256C1 now passes both installation orders; Task 252 still requires no
compatibility edit, and the completed Task-257C2 route now reuses these exact
rows without modifying their fingerprints or debug bytes.

## Task 257C3 Frozen Downstream Consumer

Task 257C3 reauthenticates the existing Task-252 `3/0/3` numeral handoff only
through its exact fingerprint and the Task-256 shared boundary edge targeting
primary 1. It adds no term/reference/request row, parent edge, ownership, or
Task-252 API. This documentation prerequisite leaves every Task-252 byte and
test unchanged.

## Task 257C3 Downstream Consumption Result

The implementation reuses the exact immutable `3/0/3` handoff and its debug
fingerprint. It adds no Task-252 production API or row. A coherent two-term
test-only handoff on the same source/module/arena validates independently
and then fails only at the C3 exact-profile boundary.

## Task 258A Frozen Downstream Consumer

Task 258A reuses exactly two normal `VariableReference` / `Value` primaries
at `74..75` and `78..79` and their two independently lookup-authenticated
references to reserved binding 0. The profile is `2/2/0`; both Task-252
`SourcePrimaryTermReference::use_ordinal()` values are 1 because one binding
row is complete before either use. These are distinct from the runner's
upstream binding/use source-event lookup ordinals 1 and 2. The statement
input fact points to reference IDs `[0, 1]`
without copying their binding, spelling, range, lookup winner, or source
ordinal. Task 252 retains all occurrence/reference ownership and semantics.
This documentation prerequisite changes no Task-252 API, source, test, or
debug byte.

## Task 269GUP Source-term Exclusion

GUP creates only the exact sibling binding environment. Checker and runner
source-term code, Task-252 role allowlists, term/reference/request tables, and
all source-term tests remain byte-identical. The later leaves at `116..117`
and `120..121` are selector-only. Task 269GU, after GUPT, owns any future
`GivenWitness -> Variable` admission and occurrence payload.
Completion evidence: [central Task-269GUP historical contract](../../task_contracts/en/269GUP.md#completion-evidence).

## Task 269GU Frozen Proof-`given` Later-use Term/Reference Contract

### Selection, authority, and classification

Fresh clean HEAD `c529245138b6d40be65c590ba701fef4f4ea0881` contains the
committed GUPT source-type prerequisite and selects only Task 269GU. Canonical
Chapter 4 §4.6.1(5), Chapter 15 §§15.3.3 and 15.10, Chapter 16 §16.3.3
item 5a and §16.4.2 require a `given` witness to bind its declaration
conditions and then remain visible for the rest of its innermost enclosing
proof/reasoning block and inherited child blocks unless shadowed. The user
confirmed that exact block lifetime; parent and sibling blocks never inherit
the binding. Chapter 8 §8.1 supplies only the already implemented declared
type. Chapter 3 supplies only the ordinary in-scope variable interpretation,
and Chapter 13 §§13.1.1 and 13.8.1 identify an in-scope identifier occurrence
as a variable reference.

The exact parser/resolver-authenticated source already selected by GUP is:

```mizar
reserve x for set;
theorem FormulaStatementGivenSmoke: thesis proof
  given y being set such that G: thesis;
  thus y = y;
end;
```

It is 128 bytes with one final LF, source SHA-256
`ec15ded78ae96022840a8419a85d74643de3b37337e9a202cbda77ee97aa7c01`,
54 Surface nodes rooted at 53, and Surface snapshot SHA-256
`c64297ce72e380a2e4146276966e085d780f8b38f2528d5abaa440a50c67db6d`.
The two exact later leaves are unrecovered `TermReference` Surface nodes 41
and 43, `y@116..117` and `y@120..121`, under the conclusion subtree
`111..122`. GUP already authenticates the exact two declaration shells,
theorem symbol/definition/contribution, theorem `19..127`, proof `62..126`,
`given` `70..108`, segment `76..87`, name `76..77`, written type `84..87`,
and resolver-local identity scope `[0]`, ordinal 1. GUPT owns the copied exact
`2/2/0` typed binding environment and `2/2/0/0/0/0` source type.

There is no blocking `spec_gap`: canonical scope, exact AST leaves, GUP
lookup identity, and GUPT type dependency determine the transport uniquely.
Absent term/reference composition and focused tests are bounded `source_drift`
and `test_gap`; stale post-GUPT ledgers are `design_drift`. Admitting
`GivenWitness` through the generic Task-252 path, changing old GUP/GUPT
payloads, or publishing formula/proof semantics is a `boundary_violation`.
Origin `0/9` is report-only `repo_metadata_conflict` and must not be repaired.

### Public composite and exact payload

Task 269GU adds these syntax-free public siblings in `source_term.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenUseTermHandoff { /* private fields */ }

impl SourceProofLocalGivenUseTermHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub const fn dependency(&self) -> &SourceProofLocalGivenUseTypeHandoff;
    pub fn dependency_fingerprint(&self) -> &str;
    pub const fn source_term(&self) -> &SourcePrimaryTermHandoff;
    pub fn source_term_fingerprint(&self) -> &str;
    pub fn debug_text(&self) -> String;
}

pub struct SourceProofLocalGivenUseTermProducer;

impl SourceProofLocalGivenUseTermProducer {
    pub fn build(
        dependency: SourceProofLocalGivenUseTypeHandoff,
        input: SourcePrimaryTermHandoffInput,
        arena: &TypedArena,
    ) -> Result<SourceProofLocalGivenUseTermHandoff,
                SourceProofLocalGivenUseTermError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalGivenUseTermError {
    InvalidDependency,
    InvalidSourceTerm,
    InvalidInstallation,
}

impl std::error::Error for SourceProofLocalGivenUseTermError {}
```

The producer consumes one authenticated GUPT handoff by value. It preserves
the entire dependency and exact `dependency.debug_text()` fingerprint, then
builds exactly one `SourcePrimaryTermHandoff` with profile `2/2/0`:

| row | exact term | exact reference |
|---|---|---|
| 0 | site node 3; `116..117`; source ordinal 0; context 1; `Normal`; spelling `y`; `VariableReference`; `Value`; no parent | term 0; binding 1; `Variable`; derived scope `[0]`; derived use ordinal 2 |
| 1 | site node 4; `120..121`; source ordinal 1; context 1; `Normal`; spelling `y`; `VariableReference`; `Value`; no parent | term 1; binding 1; `Variable`; derived scope `[0]`; derived use ordinal 2 |

There are no numeric-type requests. Both references have use ordinal 2
because reserve binding 0 and witness binding 1 have completed before each
later leaf; the first occurrence does not increment a binding event. Exact
`BindingEnv::lookup` in context 1 and lexical scope `[0]` must return local
binding 1 for each row. This is occurrence/reference identity transport only:
the source-term rows do not type the equality, create an equality node, or
publish a fact.

The ordinary public `SourcePrimaryTermProducer::build` retains its current
binding-role allowlist byte-for-byte in behavior. A private producer profile
admits `BindingKind::GivenWitness` only for the exact GU composite and only as
`SourcePrimaryTermReferenceRole::Variable`; all non-GU generic, Let,
quantifier, definition-parameter, reserved-variable, abbreviation, and
constant behavior remains unchanged. This prevents the canonical decision
from becoming an unauthenticated global admission.

The exact six-node arena is distinct from the standalone three-node GUPT
arena while preserving its dependency prefix:

| node | kind | anchor | children |
|---|---|---|---|
| 0 | `source.proof-local.given-use.reserve-type` | `14..17` | `[]` |
| 1 | `source.proof-local.given-use.type` | `84..87` | `[]` |
| 2 | `source.proof-local.given-use.type-root` | `0..127` | `[0,1]` |
| 3 | `source.term.variable-reference` | `116..117` | `[]` |
| 4 | `source.term.variable-reference` | `120..121` | `[]` |
| 5 | `source.proof-local.given-use.term-root` | `0..127` | `[2,3,4]` |

Root is node 5. All nodes have no resolver node, `TypingState::Unknown`,
`NodeRecoveryState::Normal`, and empty links. Dependency reauthentication
checks every GUPT public component and fingerprint against nodes 0--2 without
weakening the standalone GUPT three-node installation contract. Nodes 3--4
are checked by the existing primary-term node validator. Any source/module,
dependency, binding/type payload, fingerprint, cardinality, row, reference,
lookup, prefix, node, root, child, range, kind, recovery, typing, or link
mismatch fails atomically.

The composite debug grammar is exactly:

```text
source-proof-local-given-use-term-debug-v1
module: <package>::<path>
dependency-fingerprint: <Debug quoted complete GUPT debug text>
source-term-fingerprint: <Debug quoted complete source-primary-term debug text>
```

The nested source-term fingerprint renders exactly two terms and two
references in dense order and zero numeric requests. The exact public error
strings are `source proof-local given-use term dependency is invalid`,
`source proof-local given-use source term is invalid`, and
`source proof-local given-use term installation is invalid`.
Validation precedence is dependency and GUPT-prefix reauthentication, exact
source-term input/profile validation, then exact full-arena and one-shot
installation validation. Every failure publishes no partial handoff or AST
owner.

### Typed/final ownership and runner consumer

`TypedAst` adds only boxed optional `source_proof_local_given_use_term`, its
getter, one-shot `with_source_proof_local_given_use_term`, and
`InvalidSourceProofLocalGivenUseTerm`, whose string is
`source proof-local given-use term handoff is invalid`. Installation owns the
composite rather than installing temporary direct GUPT, binding, source-type,
or source-term fields. It requires the exact six-node arena, excludes every
old proof-local/source/semantic owner in both installation orders, and keeps
contexts, types, facts, coercions, initial obligations, and diagnostics empty.

`ResolvedTypedAst` clones and revalidates only that boxed composite, exposes
the matching getter, and adds `InvalidSourceProofLocalGivenUseTerm`, whose
string is `resolved typed AST source proof-local given-use term handoff is
invalid`. All six nodes project one-for-one as source-preserved role
`source.proof-local.given-use.term`. Node-hint inputs and every semantic table
remain empty.

The dormant runner adds private
`SourceProofLocalGivenUseTermRouteOutput { typed_ast, resolved }` and
`SourceProofLocalGivenUseTermRouteMutation` with exact variants `None`,
`WrongDependencyModule`, `WrongTermRange`, `WrongReferenceBinding`,
`WrongArenaRoot`, and `WrongArenaKind`. Both selectors take the same five
arguments as GUPT; cfg-test `_with_mutation` appends the mutation. Mismatch is
`None`; selected failure is `Some(Err(_))`. The only route-local error is
`Task269GU GUPT dependency is missing`; all other strings come from the
frozen upstream routes or the new public error. The route reuses the exact
GUPT private output, clones its authenticated owned dependency, constructs
only the two AST-authenticated term/reference rows and six-node arena,
installs the GU composite into an otherwise empty TypedAst, assembles
ResolvedTypedAst, and remains unreachable from public dispatch.

### Scope, tests, impact, deferrals, and exit

Implementation ownership is exactly seven existing Rust files:
`crates/mizar-checker/src/source_term.rs`,
`crates/mizar-checker/src/typed_ast.rs`,
`crates/mizar-checker/src/resolved_typed_ast.rs`,
`crates/mizar-test/src/runner/type_elaboration/source_proof_local_declaration.rs`,
`crates/mizar-test/src/runner/type_elaboration.rs`,
`crates/mizar-test/src/runner.rs`, and
`crates/mizar-test/src/runner/tests/type_elaboration/source_proof_local_declaration.rs`.
The facade re-exports remain test-only. Checker `source_type.rs`,
`source_proof_local_declaration.rs`, and `binding_env.rs`; runner
`source_statement.rs`; parser; resolver; canonical specification; fixtures;
sidecars; expectations; trace; metadata; Cargo; diagnostics; public dispatch;
CLI; and active results must not change.

The exact checker tests are
`task269gu_exact_occurrence_references_and_fingerprints_are_stable`,
`task269gu_dependency_term_input_and_arena_corruption_fail_closed`,
`task269gu_typed_and_resolved_ownership_is_atomic`, and
`task269gu_generic_and_neighbor_routes_remain_isolated`. The exact runner
tests are
`task269gu_exact_term_reference_composition_and_replay_are_stable`,
`task269gu_dependency_input_and_arena_corruption_fail_closed`,
`task269gu_typed_and_resolved_owners_are_one_shot_and_semantically_empty`, and
`task269gu_near_miss_gupt_and_active_routes_remain_isolated`. They cover both
occurrences and derived lookups, complete fingerprints, every dependency/
input/arena field and validation precedence, clone replay, one-shot and
both-order same-identity exclusion, generic/Let/old-Given/GUP/GUPT isolation,
exact selector near misses, and exhaustive zero semantic publication.

Documentation ownership is 42 synchronized Markdown files: 28 paired checker
plan/todo/audit/owner documents including this new primary owner, 12 paired
runner documents, and two global ledgers. Baselines are checker/runner library
tests `506/568`, parser/resolver/syntax `226/148/59`, production
`30/174332` and `37/75074`, path hashes
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`,
and content hashes
`fc85ad8c271614a4474cab3ef6a6d212b168546d1f76d1bc3edb9fa4354378b0` /
`afef82f149a350314a9160685e094e4a1b580d772790cf1c9e2a7efd89d0c870`.
Raw/normalized test-list hashes are checker
`d9c3c7e10b836f1e5ab987bfc54b1c06eaf8af15e2d6f3532fad51a756fca140` /
`9342b51b7e26745f5e04770fe254b8954524dccd45a01ced475b5f097d941cb1`
and runner
`30fce970d193edf3a0a84607b6015e017e91f8e6c8f35fc9b10be88e16fdff93` /
`48261f74e202e4496db6e231c335f842942ab3049b61196884984b16cc997c99`.
Implementation projects `510/572`; production lines/content and test-list
hashes are remeasured, while path hashes remain fixed.

Cases/requirements stay `428/395`, pass/fail `235/193`, warnings/errors
`23/0`, stages `101/7/205/1`, type coverage `259=247+12`, and trace SHA-256
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.
All five CLI hashes and the parser and broad proof-local gap fixture/sidecar
hashes remain at the committed GUPT values. Task 269GU adds eight Rust unit
tests but no active `.miz`, sidecar, expectation, trace row/status/backlink,
metadata case, diagnostic, CLI output, dispatch branch, or coverage credit.

Task 269GU owns only later primary-term occurrence/reference transport. It
does not own the containing `BuiltinPredicateApplication`, equality formula,
proposition, `thus` conclusion, `such that` condition or label, condition/fact
lifetime, existential/Skolem operation, type guard or assumption, capture or
export checks, goal/thesis composition, initial obligations, proof discharge
or acceptance, theorem acceptance, CoreIr, ControlFlowIr, or VC. Those remain
semantic deferrals; capture/export is the next candidate only after fresh
post-GU inventory, and Task 270 remains separate.

Exit requires EN/JA specification review **NO FINDINGS**, docs-only nine hard
gates uncapped at `>=90/100`, an exact documentation prerequisite commit,
fresh lower-stage/count/hash preflight, the exact seven-file/eight-test
implementation, separate test-sufficiency/implementation/source-docs reviews
ending **NO FINDINGS**, all verification and count/hash gates, exact staging,
and a separate implementation commit leaving a clean tree with origin
divergence reported and protected stash unchanged. Fresh inventory must then
select the next dependency-ready task automatically.

### Task 269GU implemented term/reference transport

The exact two rows at `116..117` and `120..121`, both resolving to binding 1
at derived use ordinal 2, are implemented in the frozen six-node arena. The
profile-scoped `GivenWitness -> Variable` admission, dependency/source
fingerprints, full corruption/precedence matrix, immutable replay, one-shot
Typed/Resolved ownership, and old/generic/neighbor isolation are covered by the
four checker and four runner tests. Test-sufficiency and implementation reviews
are **NO FINDINGS**.

Libraries are `510/572`; production is `30/176258` and `37/75339`, with the
content and raw/normalized test-list hashes recorded in the crate plan. No
canonical artifact, active route, semantic table, or coverage credit changed.
The user-confirmed block lifetime remains authoritative, while condition and
descendant occurrence transport, shadow/capture/export realization, and every
formula/fact/goal/proof/obligation meaning remain explicit follow-ups.

## Task 269GCP Frozen Term Deferral

The two condition leaves at `107..108` and `111..112` are exact selector
evidence only. GCP adds no `SourcePrimaryTermHandoff`, no profile admission,
and no Typed/final term owner. Task 269GCU may transport them only after the
exact GC binding and GCT type dependencies exist by value.

### Task 269GCP implemented term deferral

Both condition leaves remain excluded from the private lower output and from
all Typed/final term owners. No source-term API or admission changed; GCU may
consume them only after the separate GC and GCT dependencies exist.

## Task 269GC Frozen Term Deferral

GC publishes no term, occurrence, reference, resolver provenance at a use
site, equality operand, or Typed/final term node. The GCP-authenticated
`107..108` and `111..112` leaves remain opaque. Only GCU may transport them,
after consuming the exact GCT composite; descendant uses remain later.

### Task 269GC implemented term deferral

No term/reference/use-site or Typed/final term owner was added. Both opaque
condition leaves, descendant occurrences, and all resolver-at-use provenance
remain deferred to GCU after the exact GCT dependency.

## Task 269GCT Frozen Term Deferral

The written type contains no term arguments, so the source-type argument table
is empty. GCT publishes no term, occurrence, reference, resolver use
provenance, equality operand, or Typed/final term node. The condition leaves
`107..108` and `111..112` remain excluded; only GCU may transport them after
consuming the exact GCT composite.

Completion evidence: [central Task-269GCT historical contract](../../task_contracts/en/269GCT.md#completion-evidence).

## Task 269GCU Frozen Given-condition Term/reference Composition

GCU consumes the exact `SourceProofLocalGivenConditionTypeHandoff` by value and
publishes only the two declaration-condition variable occurrences
`y@107..108` and `y@111..112`. The input has two dense term rows on typed nodes
3/4 with source ordinals 0/1, binding context 1, normal recovery, spelling `y`,
`VariableReference`, `Value`, and no parent; two dense reference rows map terms
0/1 to `BindingId(1)` as `Variable`; numeric-type requests are empty. The
common producer derives use ordinal 2 and must resolve both rows uniquely to
the GCT-owned `GivenWitness` with scope `[0]` and type site `Source(90..93)`.

The private `SourcePrimaryTermBindingProfile::ProofLocalGivenConditionUse`
admits only `GivenWitness -> Variable` for this family. Generic admission and
the older `ProofLocalGivenUse` profile remain unchanged and mutually isolated.
No resolver `SymbolId` is fabricated: complete theorem/resolver provenance is
retained only through the nested GCT/GC/GCP dependency fingerprint.

The six-node arena is exact and ordered:

1. `source.proof-local.given-condition.reserve-type@14..17`;
2. `source.proof-local.given-condition.type@90..93`;
3. `source.proof-local.given-condition.type-root@0..133`, children `[0,1]`;
4. `source.term.variable-reference@107..108`;
5. `source.term.variable-reference@111..112`;
6. `source.proof-local.given-condition.term-root@0..133`, children `[2,3,4]`.

Root is node 5. Every node is unresolved, unknown-typed, normal, and link-free.
Dependency validation extracts nodes 0--2 with root 2 and revalidates GCT
before checking source-term input or the complete arena.

The public ABI and field order are frozen exactly as follows:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenConditionUseTermHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    dependency: SourceProofLocalGivenConditionTypeHandoff,
    dependency_fingerprint: String,
    source_term: SourcePrimaryTermHandoff,
    source_term_fingerprint: String,
}

impl SourceProofLocalGivenConditionUseTermHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub const fn dependency(&self) -> &SourceProofLocalGivenConditionTypeHandoff;
    pub fn dependency_fingerprint(&self) -> &str;
    pub const fn source_term(&self) -> &SourcePrimaryTermHandoff;
    pub fn source_term_fingerprint(&self) -> &str;
    pub fn debug_text(&self) -> String;

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
    ) -> Result<(), SourceProofLocalGivenConditionUseTermError>;

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        arena: &TypedArena,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalGivenConditionUseTermError>;
}

pub struct SourceProofLocalGivenConditionUseTermProducer;

impl SourceProofLocalGivenConditionUseTermProducer {
    pub fn build(
        dependency: SourceProofLocalGivenConditionTypeHandoff,
        input: SourcePrimaryTermHandoffInput,
        arena: &TypedArena,
    ) -> Result<
        SourceProofLocalGivenConditionUseTermHandoff,
        SourceProofLocalGivenConditionUseTermError,
    >;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalGivenConditionUseTermError {
    InvalidDependency,
    InvalidSourceTerm,
    InvalidInstallation,
}
```

The enum implements `fmt::Display` and `std::error::Error`; it has no source or
additional public method. Exact display strings are respectively
`source proof-local given-condition-use term dependency is invalid`,
`source proof-local given-condition-use source term is invalid`, and
`source proof-local given-condition-use term installation is invalid`.

Exact debug grammar is:

```text
source-proof-local-given-condition-use-term-debug-v1
module: {package}::{module}
dependency-fingerprint: {Rust-debug-quoted complete GCT debug text}
source-term-fingerprint: {Rust-debug-quoted complete source-term debug text}
```

Every line, including the last fingerprint line, ends in exactly one LF; no
blank or extra terminal line exists. Producer and replay validate dependency
identity/fingerprint first, exact term input/common term handoff/fingerprint
second, exact complete arena third, and slot availability last. Multi-
corruption tests observe the stable three-tier error precedence.

GCU owns only identifier occurrence/reference transport. `G@104..105`, the
equality and all enclosing formula/condition nodes, label/fact/guard/proof/
obligation semantics, later or descendant occurrences, capture/export, generic
source-term publication, active dispatch, coverage credit, and downstream IR
remain excluded.

Completion evidence: [central Task-269GCU historical contract](../../task_contracts/en/269GCU.md#completion-evidence).

## Task 269SDP Term Deferral

The RHS leaves `y@118..119` and `z@133..134` are authenticated only as
`TermReference` Surface subtrees. SDP publishes no primary-term row,
reference winner, use ordinal, arena node, numeric request, type, or capture.
The descendant `y` occurrence is a later consumer; `z` closure replay remains
blocked on canonical `set` reconciliation.

Completion evidence: [central Task-269SDP historical contract](../../task_contracts/en/269SDP.md#completion-evidence).

## Task 269SDC Frozen Term/Occurrence Deferral

The context-2 lookup oracle proves only inherited visibility in abstract
`BindingEnv`. SDC creates no term arena, reference row, request, or source-site
mapping for `y@118..119`; therefore it earns no descendant occurrence or
capture credit. `z@114..115`, `z@133..134`, and `q@129..130` remain opaque.
An occurrence consumer follows only after a separately frozen Given type
owner, while all Set closure/capture work remains blocked by canonical
reconciliation.

Completion evidence: [central Task-269SDC historical contract](../../task_contracts/en/269SDC.md#completion-evidence).
