# Source Set/Choice/Qua-Term Transport

> Canonical language: English. Japanese companion:
> [../ja/source_set_term.md](../ja/source_set_term.md).

## Scope

Checker Task 255 owns a syntax-free immutable description of source set
enumerations, independent set comprehensions with zero or one frozen source
condition, choice terms, and `qua` terms. It transports source shape,
transparent wrappers, written comprehension generators, bare builtin
target-type sites, direct condition-wrapper provenance and spelling, ordered
child edges, and unresolved request intent only. It does not bind
comprehension variables, resolve capture or inner condition formulas, decide
sethood or nonemptiness, select a choice witness, establish `qua`
reachability, compute result types, publish facts, accept definitions, or
lower proofs and IR.

The canonical language requirements are Chapter 13 Sections 13.4-13.6,
Chapter 7 Section 7.8.1, Chapter 8 Section 8.2.2, and their Chapter 17/21
semantic dependencies. Task 252 owns primary children, Task 253 owns
application children, Task 254 owns structure-family children, and Task 255
links to their dense root IDs without copying rows. Task 257 retains
comprehension binding/capture; Tasks 256-257 retain conditioned formula
ownership; later semantic owners retain all request resolution.

## Public Transaction

`SourceSetTermProducer::build` consumes `SourceSetTermHandoffInput`,
`BindingEnv`, `SourcePrimaryTermHandoff`, optional
`SourceFunctorApplicationHandoff` and `SourceStructureHandoff` dependencies,
and `TypedArena`. The input has seven source-ordered vectors:

- set/choice/`qua` terms;
- transparent set-term wrappers;
- written comprehension generators;
- term- or generator-owned bare target-type sites;
- direct condition wrappers and their term-owned colon provenance;
- ordered enumeration-element, comprehension-mapper, and `qua`-base edges;
- unresolved result-type, generator-sethood, choice-nonempty, and
  `qua`-widening requests.

The producer publishes seven dense immutable tables only after the entire
transaction validates. Public IDs expose zero-based `new` and `index`;
tables expose `get`, source-ordered `iter`, `len`, and `is_empty`; validated
rows expose only the read-only accessors frozen in the crate plan.

Term kinds are `Enumeration`, `Comprehension`, `Choice`, and `Qua`. Recovery
is `Normal` or `Degraded`. Type heads are bare `BuiltinSet` or
`BuiltinObject`. Targets are a Task-252 `Primary`, Task-253 root
`Application`, Task-254 root `Structure`, or later nested Task-255
`SetTerm`.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceSetTermKind` | `#[non_exhaustive]`; callers must tolerate later frozen set-family source kinds. |
| `SourceSetTermRecovery` | `#[non_exhaustive]`; callers must tolerate later recovery classes. |
| `SourceSetTypeOwner` | `#[non_exhaustive]`; callers must tolerate later target-site owners. |
| `SourceSetTypeRole` | `#[non_exhaustive]`; callers must tolerate later term-owned target roles. |
| `SourceSetTypeHead` | `#[non_exhaustive]`; callers must tolerate later frozen bare builtin heads. |
| `SourceSetEdgeRole` | `#[non_exhaustive]`; callers must tolerate later child-edge roles. |
| `SourceSetTarget` | `#[non_exhaustive]`; callers must tolerate later frozen cross-family targets. |
| `SourceSetRequestKind` | `#[non_exhaustive]`; callers must tolerate later unresolved request kinds. |
| `SourceSetTermError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Validation And Ownership

The producer authenticates source/module identity, dense source preorder,
context, ranges, recovery, exact typed-arena anchors, grouping, ordinals,
canonical token spelling, and single ownership. Arena keys are
`source.term.set.enumeration`, `.comprehension`, `.choice`, `.qua`,
`.parenthesized`, `.comprehension-generator`, `.target-type`, and
`.target-type-head`.

Canonical spelling is reconstructed recursively from authenticated rows.
Enumeration elements join with ` , ` inside `{ }`; a comprehension joins its
mapper, ` where `, and `identifier is type` generator fragments; choice is
`the type`; `qua` is `base qua type`; and every wrapper is
`format!("( {} )", contained_spelling)`. Generator spelling is one lexer
identifier. A bare type expression and head both spell exactly `set` or
`object`.

An enumeration owns zero or more element edges and one final result request.
A comprehension owns one or more generators, one bare type site and sethood
request per generator, one mapper edge, and one final result request. A
choice owns one target type site, one nonempty request, and one final result
request. A `qua` term owns one target type site, one base edge, one widening
request, and one final result request.

For every written child slot, validation computes the maximal effective
Task-252/253/254/255 occurrence after removing descendants. Exactly one
remaining occurrence must cover the complete child slot. A primary already
owned by Task 253 or 254 and an application already owned by Task 254 cannot
be targeted again. Nested Task-253, Task-254, and Task-255 descendants remain
with their nearest family owner. Reverse Task-253/254 parents containing a
Task-255 child, conditioned comprehensions, generator-referencing
comprehensions, non-bare targets, and all other frozen exclusions fail closed
without detached descendants.

## Derived Dependency Fingerprints

The output always derives `primary_term_fingerprint` from the exact Task-252
`debug_text()`. `application_fingerprint` and `structure_fingerprint` are the
exact dependency `debug_text()` values and are `Some` only when an edge
targets that family. Unrelated installed optional handoffs coexist with
`None` only when their effective occurrences are range-disjoint from all
Task-255 terms, wrappers, and targets.

`TypedAst::with_source_set_term` is one-shot and requires every targeted
dependency first. `with_source_application` and `with_source_structure`
revalidate an already installed Task-255 handoff, so installation order
cannot bypass ownership or fingerprint checks. `ResolvedTypedAst` revalidates
and clone-preserves the same association without rebuilding or renumbering
rows. Typed and resolved debug renderings include the handoff only when
present.

## Private Source Consumer

Raw `SurfaceAst`, source node IDs, and syntax kinds remain in
`mizar-test::runner::type_elaboration::source_set_term`. Production selects
only the four functor definientia in
`fail_type_elaboration_local_set_choice_qua_term_gap_001`. The leaf reuses
Task 248's real binding-context transaction and Task 252's primary producer;
it fabricates no comprehension `BindingId`.

The exact Task-255 term/wrapper/generator/type-site/edge/request oracle is
4/0/1/3/4/7. The shared arena contains the Task-252
primary/reference/numeric-request slice 4/0/4. The real route has no Task-253
or Task-254 row or fingerprint. After transport validation it retains the
Task-260 `type_elaboration.external_dependency.ast_payload_extraction`
boundary with no public diagnostic.

## Verification Boundary

Checker tests cover every table and enum, all arena keys, canonical spelling,
wrapper nesting, per-kind cardinality and request association, cross-family
nearest ownership, optional dependency fingerprints, installation orders,
corruption, determinism, clone preservation, and atomic failure. Runner tests
cover the exact consumer/oracle, real lower-stage shape, zero/many
enumerations, independent multiple/nested comprehensions, choice, `qua`,
wrappers, degraded transport, cross-family children, exclusions, mutation
isolation, deterministic replay, final ownership, and isolation from every
other active type-elaboration case.

The bounded trace row is
`spec.en.checker.type_elaboration.source_set_choice_qua_term_payload`.
Task 255 changes only executable source-transport coverage; generator/capture,
formula, typing, evidence, facts, proof, and Steps 6/7 semantics remain
unimplemented.

## Task 255C1 Frozen Condition-Bearing-Comprehension Extension

Task 255C1 extends this module from six to seven source-ordered tables only
for one independent conditioned comprehension. Canonical Chapters 10, 13,
and 14 plus the existing parser fixtures authorize the exact 191-byte source
and ranges frozen in the crate plan. The new exact profile is
term/wrapper/generator/type-site/condition/edge/request
`1/0/1/1/1/1/2`, over Task-253 `1/0/1/2/2` and the one immutable
Task-252 `4/0/4` handoff.

`SourceSetConditionInput` and its immutable row retain owner term/ordinal,
Task-255-owned colon site/range/spelling, direct condition-wrapper site,
condition range/spelling, and recovery. The colon uses typed-arena key
`source.term.set.comprehension-condition-colon`; `condition_site` anchors the
direct `FormulaExpression` with Task-255 association key
`source.term.set.comprehension-condition`. Task 255 authenticates that wrapper
as the subtree boundary but leaves the inner `BuiltinPredicateApplication`
formula site and row to Task 256. Context is derived from the owner term.
`SourceSetConditionId` exposes `new`/`index`; its table exposes
`get`/`iter`/`len`/`is_empty`; the row and handoff expose every frozen field
and `conditions()` read-only.

Conditions group densely, appear only zero-or-one per comprehension, follow
the final generator type, and contribute ` : condition` to canonical term
spelling. They create no Task-255 edge or request. Every lower-family
occurrence wholly inside an authenticated condition range is excluded from
Task-255 direct-child discovery. The exact C1 route retains Task-252 numerals
3 and 4 there without a Task-255 edge; Task 256 may later own the equality
edges, but no formula handoff is installed by this extension. Outside
condition ranges, every previous nearest-family and whole-subtree exclusion
remains unchanged.

Condition rows render after type sites and before edges:

```text
condition#<id> term=<term> ordinal=<n> colon_range=<s>..<e> colon_site=<site> colon_spelling=<quoted> condition_site=<site> range=<s>..<e> spelling=<quoted> recovery=<key>
```

Empty condition tables render nothing, preserving every legacy debug byte.
Sixteen existing input literals receive empty vectors; `to_input`
clone-preserves nonempty rows.

Checker corruption coverage explicitly rejects omitted, copied, out-of-range,
or wrong-kind condition sites; omitted, copied, or out-of-range condition
primaries; and condition-contained Task-253/254/255 descendants. It proves
that Task 255 owns the direct condition
`FormulaExpression` site but not its inner `BuiltinPredicateApplication`
formula site; and rechecks unchanged nearest-family ownership immediately
outside the condition range.

The private runner must call a reusable unwrapped imported-`++` Task-253
extractor/builder and install Task 252, Task 253, and Task 255 in one arena.
The exact future fail sidecar and covered trace row prove source transport
only. Generator binding/capture, inner condition-formula ownership/composition,
sethood/result answers, equality truth, definition acceptance, proof, and IR
remain deferred.

## Task 255C1 Implementation Result

The seven-table extension is implemented exactly as frozen. Condition rows
authenticate their direct term-owned colon and wrapper sites, recursively keep
the wrapper subtree inside the recorded range, require every contained
Task-252 primary, reject Task-253/254/255 descendants and condition-directed
edges, and exclude only authenticated condition-contained lower-family rows
from direct-child discovery. Full debug, legacy empty-table byte equality,
group/order/cardinality, dependency substitution, rollback, and final clone
tests pass.

## Task 257C2 Frozen Consumer Boundary

Task 257C2 consumes, without modifying, condition row 0 of the exact Task-255C1
profile. The row continues to own the colon and direct `FormulaExpression`
wrapper only. Its `177..182` range and `3 = 4` spelling must match the distinct
direct Task-256 equality row; the future Task-257C2 association owns no site.
Task-252 primaries 2 and 3 remain excluded from Task-255 edges and become only
Task-256 equality operands. No Task-255 table, debug byte, fingerprint,
request, or validation meaning changes. At the frozen pre-Task-256C1
baseline, the separate lower task had to authenticate this exact condition
containment in the Task-256 validator for both installation orders without
weakening arbitrary overlap rejection. Task 256C1 now does so; only fresh
Task-257C2 preflight and implementation remained at prerequisite exit. The
completed C2 route now consumes the condition row unchanged and owns only
the separate association.

## Task 256C1 Frozen Lower-Owner Boundary

Task 256C1 consumes the immutable Task-255C1 condition row only as validation
context. It neither changes nor reconstructs the seven Task-255 tables.
Term 0 remains a `Comprehension` at `139..184`; condition 0 remains ordinal 0
on that term with colon `175..176`, direct wrapper range `177..182`, spelling
`3 = 4`, normal recovery, and owner-term context 0. Its condition site must
directly contain the distinct Task-256 equality site, whose context must
equal that existing owner-term context.

The condition row intentionally gains no context or formula ID; Task 256C1
derives context from the immutable owner term. Task 255
continues to own only the colon and wrapper boundary, while Task 256 owns the
inner equality and Task 257C2 later owns their explicit association.
Task 256C1 grants no Task-255 edge, fingerprint, debug, request, semantic, or
validation-schema change.

## Task 256C1 Implementation Result

The lower-owner boundary remains unchanged. Task 256C1 now consumes the
already validated immutable condition row only while Task 256 checks the
overlapping equality; it does not add or rewrite any Task-255 row. The exact
conditioned profile remains `1/0/1/1/1/1/2`, its debug and fingerprints are
unchanged, and stale, wrong-owner, wrapped, or non-direct relations fail
closed in Task 256. Both installation orders now accept only the exact
authenticated relation.

## Task 258B3M2B2B3P Frozen Proof-Context Enumeration Reuse

After B2C implementation commit
`e8373c683448e524cb98edde83fdf8de83a125cd`, fresh inventory selects a
private Task-255 set-enumeration reuse prerequisite. Its exact final-LF
source is:

```text
reserve x for set;
theorem FormulaStatementSetEnumerationWitnessSmoke: x = x proof
  take {1, 2};
  thus x = x;
end;
```

The source is 117 bytes, SHA-256
`4f8ea5b9cadf763ea108b6f7deb6b481cb6f997dec2048b4351f07fd5dc38539`,
zero-diagnostic, and 57 nodes/root 56 with normal same-source recovery.
Set term 0 has site `Node(40)`, range `90..96`, source ordinal 0, proof
context 1, recovery `Normal`, spelling `{ 1 , 2 }`, and kind
`Enumeration`. `EnumerationElement` edge 0 has term 0, ordinal 0, and
target `Primary(2)` at node/range `36/91..92`; edge 1 has term 0, ordinal
1, and target `Primary(3)` at `38/94..95`. Request 0 has term 0, ordinal
0, kind `ResultType`, `generator = None`, and `type_site = None`. The
primary fingerprint equals the exact Task-252 handoff fingerprint;
application and structure fingerprints are absent. There are no
comprehension, choice, condition, or other Task-255 rows.

Task 48 supplies contexts 0 and proof context 1, one reserve binding, and no
diagnostics. Task 252 owns nodes `30/32/36/38/44/46`, with theorem and
conclusion references plus numeric requests 2/3. Task 255 owns only node 40.
Tasks 253/254/256/258 are empty, and the term-expression, witness,
statement, proof, theorem, item, compilation, and root containers are
unowned. There is no imported provenance.

B3P may add only an explicit-context private sibling in the runner
`source_set_term` path while preserving the existing context-0 helper
byte-for-byte. The exact two future compound tests are
`task258b3m2b2b3p_set_enumeration_proof_context_reuse_is_exact` and
`task258b3m2b2b3p_set_enumeration_corruption_replay_and_legacy_output_fail_closed`.
Together they mutate all 117 loaded-source bytes including final LF,
stripped/extra-LF variants, every kind/range/recovery/ordered-children field
of all 57 nodes and root identity, every local resolver field/substitution,
every Task-48 context/binding field, every Task-252 term/reference/numeric-
request field, and every Task-255 term/`EnumerationElement` edge/request/
fingerprint field. They assert the exact owned partition
`{30,32,36,38,40,44,46}` and its complement through node 56, explicit
validation precedence, stale-fingerprint replay, atomic rollback, clean
replay, and exact final typed/resolved clone.

They also assert empty Tasks 253/254/256/258, active and adjacent-family
isolation, and empty semantic/proof/goal outputs. Legacy context-0 stability
uses literal preimplementation Task-111 hashes: handoff
`30b72230bb7ff39464962133b58df212e23afccccc8f4e4788ab9a9d0481c43a`,
typed `1bb296c06ab62691684260aa94987adee23081baa4a35aac9e485d95370d2cb9`,
and resolved
`cdb4eaae9605f62269d6a74d64267a8fcb1e8d8008564d8b9e014037665df1e4`.
Old/new in-build equality alone is insufficient.

This lower task adds no checker row, API, statement witness, or semantic
claim. Upper B3A alone may later own
`SourceStatementWitness -> SetTerm(0)`. Empty, singleton, three-or-more,
nested, parenthesized, comprehension, choice, and `qua` terms; sethood,
element/result unification; existential/proof/goal/theorem behavior; and
Tasks 253/254/256/258 remain excluded.

## Task 258B3M2B2B3P Documentation Review Status

Specification/documentation, test-sufficiency, implementation-boundary, and
source/documentation consistency reviews all report **NO FINDINGS**. Exact
source/hash, lint, library, production/test-list/CLI hash, scope, diff, and
trace-no-op verification pass. The lower table/test oracle is frozen; future
private implementation `source_drift`/`test_gap` remains planned. Final
quality, commit, post-commit, and fresh inventory are pending.

## Task 258B3M2B2B3P Final Quality Status

Final quality has **NO FINDINGS**, all nine hard gates PASS, and valid
`98/100` (`20/20/15/14/10/10/5/4`). Only stage/commit, post-commit, and
fresh implementation inventory remain pending.

## Task 258B3M2B2B3P Implemented Proof-Context Enumeration Reuse

The frozen contract from
`285a1f11c310bb313c4c6b4feae914eb11f74754` is implemented in the four
authorized runner files. `source_set_term_output_with_source_term_in_context`
is a `pub(super)` explicit-context sibling; the pre-existing entry point is a
context-0 delegate. Context-0 compatibility remains independently fixed by
the literal handoff/typed/resolved hashes
`30b72230bb7ff39464962133b58df212e23afccccc8f4e4788ab9a9d0481c43a`,
`1bb296c06ab62691684260aa94987adee23081baa4a35aac9e485d95370d2cb9`,
and
`cdb4eaae9605f62269d6a74d64267a8fcb1e8d8008564d8b9e014037665df1e4`.

The exact two tests cover every source byte/final-LF variant, 57-node
surface field, 63-field resolver oracle, 39-field binding oracle, every
Task-252/255 field, coherent application/structure dependencies whose
non-None fingerprints are rejected by a shared fingerprint-only exact
subprofile, stale and simultaneous validation precedence, immediate clean
replay, typed/resolved clone rollback, and family/active isolation. Focused
`2/2`, runner library `446/446`, formatting, package Clippy, and diff check
pass; test-sufficiency and implementation reviews are **NO FINDINGS**.
Source/documentation consistency and documentation/boundary repeats are also
**NO FINDINGS**. Lint-policy `15/14`, metadata `137`, focused/library/fmt,
workspace Clippy/tests, five CLI and current manifest/test-list hashes, diff
check, and exact 30-file scope PASS.

No checker schema/API or semantic result was added. Independent final
quality reports **NO FINDINGS**; all nine hard gates PASS with valid
`98/100` (`20/20/15/14/10/10/5/4`). Only the implementation
commit/post-commit checks and fresh B3A inventory remain pending.

## Task 258B3M2B2B3A Frozen Set-Term Consumer Boundary

Task 255/B3P stays byte-exact. `SetTerm(0)` is node/range `40/90..96`,
context `1`, ordinal `0`, `Normal`, spelling `{ 1 , 2 }`, `Enumeration`;
two `EnumerationElement` edges target `Primary(2/3)`, one result-type
request exists, primary fingerprint is exact, and application/structure
fingerprints are absent. B3A consumes unchanged
`source_set_term_output_with_source_term_in_context`.

B3A adds no set-term producer behavior, only witness
`0 -> SetTerm(0) -> Primary(2/3)` and exact set fingerprint authentication.
No reverse/semantic edge exists. Both `source_set_term.rs` files, result
typing/sethood/element unification, and broader set forms are forbidden.

## Task 258B3M2B2B3A Implemented Consumer Closure

B3A consumes the unchanged Task-255/B3P handoff through the existing runner
seam and records its exact debug fingerprint in the statement-witness
handoff. Neither `source_set_term.rs` file changed. The two enumeration
edges and result-type request remain source transport only: no result
typing, sethood/element unification, imported/broader set form, or semantic
edge is credited. Focused/package and implementation reviews pass with
**NO FINDINGS**. The second source/documentation consistency repeat and
final documentation/boundary reread also report **NO FINDINGS**; parent
final verification listed in the crate plans passes, including exact
`39`-file scope. Independent final read-only quality review reports
**NO FINDINGS**. All nine hard gates PASS with no score cap; the valid score
is `98/100` (`20/20/15/14/10/10/5/4`). The stated semantic and coverage
deferrals remain unchanged as residual risk. Only the dedicated
implementation commit, post-commit invariant verification, and fresh
next-task inventory remain pending.

## Task 258B3M2B2B3B Reused Empty-Enumeration Lower Contract

B3B adds no Task-255 schema or producer behavior. The existing
explicit-context extractor must yield exactly one `Enumeration` term at
`33/95..97`, spelling `{ }`, context 1, with zero wrappers, generators,
type sites, conditions, and edges plus one `ResultType` request. Existing
zero-element producer tests remain the lower authority; no
`source_set_term.rs` source or test changes are authorized. B3B owns only
the separate upper statement/witness consumer and preserves choice,
comprehension, `qua`, and all semantic requests as later work.

## Task 258B3M2B2B3B Implemented Consumer Closure

B3B consumes the unchanged explicit-context Task-255 handoff: exactly one
`Enumeration`, zero wrappers/generators/type-sites/conditions/edges, and one
`ResultType` request. Neither `source_set_term.rs` owner nor its schema or
helper changed. The statement witness records only the existing set
fingerprint and `SetTerm(0)` target. Result typing, sethood, element
unification, choice/comprehension/`qua`, and all semantic credit remain
deferred.

Post-auth injection and stage-prefix/non-generic-guard assertions remain
upper-consumer tests only. All test-sufficiency repeats and the final
implementation repeat report **NO FINDINGS**; lower ownership remains a
source/test no-op.

## Task 258B3M2B2B3C Reuse Contract

No Task-255 source change is needed. The frozen handoff is
`1/0/0/1/0/0/2`: one `Choice` term at `35/82..89`, one `ChoiceTarget`
`BuiltinSet` type site at expression/head `34/33/86..89`, zero child edges,
then `ChoiceNonempty(type-site 0)` and `ResultType`. Context is proof context
`1`; application/structure fingerprints are absent. Future B3C tests mutate
all `39` safely mutable input fields, replay each result, and require Task-255
stage errors rather than a generic dependency failure.

## Task 258B3M2B2B3C Reused Choice Consumer

B3C consumes the unchanged exact Task-255 handoff: one `Choice`, one
`ChoiceTarget` builtin-set type site, zero wrappers/generators/conditions/
edges, and ordered `ChoiceNonempty` then `ResultType` requests. Neither
`source_set_term.rs` owner, schema, producer, or test changed. The upper
witness records only its existing set fingerprint and `SetTerm(0)` target;
the exact 39-field replay matrix confirms Task-255-owned error precedence.
Choice nonemptiness, stable symbols, type facts, and every semantic credit
remain deferred.

## Task 258B3M2B2B3D Qua Reuse Contract

No Task-255 source change is needed. The frozen handoff is
`1/0/0/1/0/1/2`: one `Qua` term at `37/79..88`, one term-owned
`QuaTarget` `BuiltinSet` type site at expression/head `36/35/85..88`,
one `QuaBase -> Primary(2)` edge, then ordered
`QuaWidening(type-site 0)` and `ResultType`. Context is proof context `1`;
application/structure fingerprints are absent. Future B3D tests mutate all
`44` safely mutable Task-255 fields, replay each result, and require
Task-255-owned errors. Both `source_set_term.rs` owners remain unchanged;
widening is not discharged.

## Task 258B3M2B2B3D Reused Qua Consumer

B3D consumes the existing Task-255 handoff without modifying either
`source_set_term.rs` owner. The installed profile remains exactly
`1/0/0/1/0/1/2`, with `QuaTarget` builtin-set expression/head
`36/35/85..88`, `QuaBase -> Primary(2)`, and ordered
`QuaWidening(type-site 0)`/`ResultType`. The upper witness stores only the
existing set fingerprint and `SetTerm(0)` target.

All 44 safely mutable Task-255 fields are replayed with `Task255:` ownership
and non-generic rejection; test-sufficiency review reports **NO FINDINGS**.
No reachability, widening discharge, type-view, result/numeric typing,
overload/coercion, fact, or proof semantics is added. Independent
implementation review reports **NO FINDINGS**. Source/documentation
consistency and boundary review also report **NO FINDINGS** after the
bounded review-state/family/qua-edge corrections. Both packages, formatting,
full Clippy, workspace tests, five CLIs, and count/hash reruns pass.
Independent final read-only quality review reports **NO FINDINGS**; all nine
hard gates PASS with no cap at valid `100/100`
(`20/20/15/15/10/10/5/5`). Only exact staging/cached-diff review,
implementation commit, and post-commit/fresh-next-task gates remain pending.
